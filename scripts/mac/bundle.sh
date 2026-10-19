#!/bin/bash
set -euo pipefail
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
source "$root/scripts/mac/messages.sh"
source "$root/scripts/mac/sign.sh"
cd "$root"
version=$(awk '/^\[workspace.package\]/{section=1;next} /^\[/{section=0} section && /^version = / {gsub(/"/, "", $3); print $3; exit}' Cargo.toml)
test -n "$version"
print_plist() { sed "s/@VERSION@/$version/g" "$root/scripts/mac/Info.plist.in"; }
if [[ "${1:-}" == --print-plist ]]; then print_plist; exit 0; fi
[[ $# -eq 0 || ( $# -eq 1 && "$1" == --universal ) ]] || exit 2

# 构建开始前清除旧包；任何后续失败也不能留下可被 dmg.sh 复用的半成品。
bundle="$root/target/mac/Mousetroke.app"
rm -rf "$bundle"
trap 'rm -rf "$bundle"' EXIT

# 尊重调用者的构建目录，最终 app 固定放在仓库的 target/mac。
# 先在 crates/mousetroke-settings/ui 执行 npm ci && npm run build；custom-protocol 将 ui/dist 内嵌进主程序。
build_target="${CARGO_TARGET_DIR:-$root/target}"
export CARGO_TARGET_DIR="$build_target" CARGO_BUILD_JOBS=4 MACOSX_DEPLOYMENT_TARGET=11.0
if [[ "${1:-}" == --universal ]]; then
    for architecture in aarch64-apple-darwin x86_64-apple-darwin; do
        cargo build --release -p mousetroke --locked --target "$architecture"
    done
    mkdir -p "$build_target/universal/release"
    binary="$build_target/universal/release/mousetroke"
    lipo "$build_target/aarch64-apple-darwin/release/mousetroke" \
        "$build_target/x86_64-apple-darwin/release/mousetroke" -create -output "$binary"
    lipo "$binary" -verify_arch arm64 x86_64
else
    cargo build --release -p mousetroke --locked
    binary="$build_target/release/mousetroke"
fi
mkdir -p "$bundle/Contents/MacOS"
cp "$binary" "$bundle/Contents/MacOS/mousetroke"
chmod +x "$bundle/Contents/MacOS/mousetroke"
print_plist > "$bundle/Contents/Info.plist"
plutil -lint "$bundle/Contents/Info.plist"
sign_mac "$bundle"
trap - EXIT
message=$(mac_message mac.bundle.done)
printf '%s\n' "${message//\{path\}/$bundle}"
