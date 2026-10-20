#!/bin/bash
set -euo pipefail
script_dir=$(dirname "${BASH_SOURCE[0]}") || exit 1
root=$(cd "$script_dir/../.." && pwd) || exit 1
source "$root/scripts/mac/messages.sh" || exit 1
source "$root/scripts/mac/sign.sh" || exit 1
bundle="$root/target/mac/Mousetroke.app"
image="$root/target/mac/Mousetroke.dmg"
# 先删除旧镜像，即使输入包缺失或校验失败也不能遗留上次的发布产物。
rm -f "$image" || exit 1
[[ -d "$bundle" ]] || { printf 'Missing bundle: %s\n' "$bundle" >&2; exit 1; }
binary="$bundle/Contents/MacOS/mousetroke"
verify_universal "$binary" || exit 1
codesign --verify --strict --all-architectures "$bundle" || exit 1
if [[ "${MOUSETROKE_REQUIRE_SIGNING:-0}" == 1 ]]; then
    # display 默认只看宿主架构；两片都必须包含证书身份，ad-hoc 没有 Authority。
    for architecture in arm64 x86_64; do
        signature=$(codesign --display --verbose=4 --architecture "$architecture" "$bundle" 2>&1) || exit 1
        if ! grep -q '^Authority=' <<< "$signature"; then
            mac_message mac.release.sign_required >&2
            exit 1
        fi
    done
fi
temporary=$(mktemp -d) || exit 1
mountpoint="$temporary/mount"
attached=0
complete=0
cleanup() {
    local status=$?
    if [[ "$attached" == 1 ]]; then
        if hdiutil detach "$mountpoint" || hdiutil detach -force "$mountpoint"; then
            attached=0
        else
            status=1
        fi
    fi
    # 卸载失败时保留挂载目录，绝不递归删除仍挂载的卷。
    if [[ "$attached" == 0 ]]; then rm -rf "$temporary" || status=1; fi
    if [[ "$complete" != 1 || "$status" != 0 ]]; then rm -f "$image" || status=1; fi
    exit "$status"
}
trap cleanup EXIT
ditto "$bundle" "$temporary/Mousetroke.app" || exit 1
# dmgbuild 创建固定布局、Applications 链接和隐藏的 Retina 背景；先暂存一次以保留原包。
python3 "$root/scripts/mac/build_dmg.py" "$root/scripts/mac/dmg_settings.py" "$temporary/Mousetroke.app" "$image" || exit 1
# 复制回调早于 Finder 布局处理；必须重挂最终镜像，检查真正交付的 app。
mkdir "$mountpoint" || exit 1
attached=1
hdiutil attach -readonly -nobrowse -mountpoint "$mountpoint" "$image" || exit 1
mounted_app="$mountpoint/Mousetroke.app"
codesign --verify --deep --strict --all-architectures "$mounted_app" || exit 1
# codesign 允许 quarantine 等属性；递归检查明确拒绝 FinderInfo 和资源叉。
attributes=$(xattr -r "$mounted_app") || exit 1
while IFS= read -r attribute; do
    case "${attribute##*: }" in
        com.apple.FinderInfo|com.apple.ResourceFork)
            printf 'Forbidden extended attributes in DMG app:\n%s\n' "$attributes" >&2
            exit 1
            ;;
    esac
done <<< "$attributes"
hdiutil detach "$mountpoint" || exit 1
attached=0
sign_mac "$image" || exit 1
hdiutil verify "$image" || exit 1
complete=1
