#!/bin/bash
set -euo pipefail
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
source "$root/scripts/mac/messages.sh"
source "$root/scripts/mac/sign.sh"
bundle="$root/target/mac/Mousetroke.app"
image="$root/target/mac/Mousetroke.dmg"
# 先删除旧镜像，即使输入包缺失或校验失败也不能遗留上次的发布产物。
rm -f "$image"
test -d "$bundle"
lipo "$bundle/Contents/MacOS/mousetroke" -verify_arch arm64 x86_64
codesign --verify --strict --all-architectures "$bundle"
if [[ "${MOUSETROKE_REQUIRE_SIGNING:-0}" == 1 ]]; then
    # display 默认只看宿主架构；两片都必须包含证书身份，ad-hoc 没有 Authority。
    for architecture in arm64 x86_64; do
        signature=$(codesign --display --verbose=4 --architecture "$architecture" "$bundle" 2>&1)
        if ! grep -q '^Authority=' <<< "$signature"; then
            mac_message mac.release.sign_required >&2
            exit 1
        fi
    done
fi
temporary=$(mktemp -d)
trap 'rm -rf "$temporary"; rm -f "$image"' EXIT
ditto "$bundle" "$temporary/Mousetroke.app"
ln -s /Applications "$temporary/Applications"
hdiutil create "$image" -format UDZO -volname Mousetroke -srcfolder "$temporary"
sign_mac "$image"
hdiutil verify "$image"
trap 'rm -rf "$temporary"' EXIT
