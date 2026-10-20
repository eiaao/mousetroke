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
for architecture in arm64 x86_64; do
    lipo "$binary" -verify_arch "$architecture" || exit 1
done
architectures=$(lipo "$binary" -archs) || exit 1
[[ "$architectures" =~ ^[[:space:]]*(arm64[[:space:]]+x86_64|x86_64[[:space:]]+arm64)[[:space:]]*$ ]] || {
    printf 'Expected exactly arm64 and x86_64; got: %s\n' "$architectures" >&2
    exit 1
}
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
trap 'status=$?; rm -rf "$temporary" || status=1; rm -f "$image" || status=1; exit "$status"' EXIT
ditto "$bundle" "$temporary/Mousetroke.app" || exit 1
ln -s /Applications "$temporary/Applications" || exit 1
hdiutil create "$image" -format UDZO -volname Mousetroke -srcfolder "$temporary" || exit 1
sign_mac "$image" || exit 1
hdiutil verify "$image" || exit 1
trap 'status=$?; rm -rf "$temporary" || status=1; exit "$status"' EXIT
