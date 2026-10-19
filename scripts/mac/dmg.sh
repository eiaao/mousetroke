#!/bin/bash
set -euo pipefail
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
source "$root/scripts/mac/messages.sh"
source "$root/scripts/mac/sign.sh"
bundle="$root/target/mac/Mousetroke.app"
test -d "$bundle"
codesign --verify --strict "$bundle"
temporary=$(mktemp -d)
trap 'rm -rf "$temporary"' EXIT
ditto "$bundle" "$temporary/Mousetroke.app"
ln -s /Applications "$temporary/Applications"
image="$root/target/mac/Mousetroke.dmg"
hdiutil create -ov -format UDZO -volname Mousetroke -srcfolder "$temporary" "$image"
sign_mac "$image"
hdiutil verify "$image"
