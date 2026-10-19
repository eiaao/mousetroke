#!/bin/bash
set -euo pipefail
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
source "$root/scripts/mac/messages.sh"
cd "$root"
version=$(awk '/^\[workspace.package\]/{section=1;next} /^\[/{section=0} section && /^version = / {gsub(/"/, "", $3); print $3; exit}' Cargo.toml)
test -n "$version"
print_plist() { sed "s/@VERSION@/$version/g" "$root/scripts/mac/Info.plist.in"; }
if [[ "${1:-}" == --print-plist ]]; then print_plist; exit 0; fi

# 尊重调用者的构建目录，最终 app 固定放在仓库的 target/mac。
build_target="${CARGO_TARGET_DIR:-$root/target}"
CARGO_TARGET_DIR="$build_target" CARGO_BUILD_JOBS=4 MACOSX_DEPLOYMENT_TARGET=11.0 cargo build --release -p mousetroke --locked
bundle="$root/target/mac/Mousetroke.app"
mkdir -p "$bundle/Contents/MacOS"
cp "$build_target/release/mousetroke" "$bundle/Contents/MacOS/mousetroke"
chmod +x "$bundle/Contents/MacOS/mousetroke"
print_plist > "$bundle/Contents/Info.plist"
plutil -lint "$bundle/Contents/Info.plist"
identity="${MOUSETROKE_SIGN_ID:-Mousetroke Dev}"
if [[ "$identity" == - ]] || ! security find-identity -v -p codesigning | grep -F -- "$identity" >/dev/null; then
    warning=$(mac_message mac.bundle.adhoc_warning)
    printf '%s\n' "${warning//\{identity\}/$identity}" >&2
    identity=-
fi
codesign --force --options runtime --sign "$identity" "$bundle"
codesign --verify --strict --verbose=2 "$bundle"
message=$(mac_message mac.bundle.done)
printf '%s\n' "${message//\{path\}/$bundle}"
