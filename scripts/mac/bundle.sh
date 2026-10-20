#!/bin/bash
set -euo pipefail
script_dir=$(dirname "${BASH_SOURCE[0]}") || exit 1
root=$(cd "$script_dir/../.." && pwd) || exit 1
source "$root/scripts/mac/messages.sh" || exit 1
source "$root/scripts/mac/sign.sh" || exit 1
cd "$root" || exit 1
version=$(awk '/^\[workspace.package\]/{section=1;next} /^\[/{section=0} section && /^version = / {gsub(/"/, "", $3); print $3; exit}' Cargo.toml) || exit 1
[[ -n "$version" ]] || { printf 'Missing workspace version\n' >&2; exit 1; }
print_plist() { sed "s/@VERSION@/$version/g" "$root/scripts/mac/Info.plist.in" || return 1; }
if [[ "${1:-}" == --print-plist ]]; then print_plist || exit 1; exit 0; fi
[[ $# -eq 0 || ( $# -eq 1 && "$1" == --universal ) ]] || { printf 'Usage: bundle.sh [--universal|--print-plist]\n' >&2; exit 2; }

# 构建开始前清除旧包；任何后续失败也不能留下可被 dmg.sh 复用的半成品。
bundle="$root/target/mac/Mousetroke.app"
rm -rf "$bundle" || exit 1
trap 'status=$?; rm -rf "$bundle" || status=1; exit "$status"' EXIT

# 尊重调用者的构建目录，最终 app 固定放在仓库的 target/mac。
# 先在 crates/mousetroke-settings/ui 执行 npm ci && npm run build；custom-protocol 将 ui/dist 内嵌进主程序。
build_target="${CARGO_TARGET_DIR:-$root/target}"
export CARGO_TARGET_DIR="$build_target" CARGO_BUILD_JOBS=4 MACOSX_DEPLOYMENT_TARGET=11.0
if [[ "${1:-}" == --universal ]]; then
    for architecture in aarch64-apple-darwin x86_64-apple-darwin; do
        cargo build --release -p mousetroke --locked --target "$architecture" || exit 1
    done
    mkdir -p "$build_target/universal/release" || exit 1
    binary="$build_target/universal/release/mousetroke"
    lipo "$build_target/aarch64-apple-darwin/release/mousetroke" \
        "$build_target/x86_64-apple-darwin/release/mousetroke" -create -output "$binary" || exit 1
    verify_universal "$binary" || exit 1
else
    cargo build --release -p mousetroke --locked || exit 1
    binary="$build_target/release/mousetroke"
fi
mkdir -p "$bundle/Contents/MacOS" || exit 1
cp "$binary" "$bundle/Contents/MacOS/mousetroke" || exit 1
chmod +x "$bundle/Contents/MacOS/mousetroke" || exit 1
print_plist > "$bundle/Contents/Info.plist" || exit 1
plutil -lint "$bundle/Contents/Info.plist" || exit 1
sign_mac "$bundle" || exit 1
message=$(mac_message mac.bundle.done) || exit 1
trap - EXIT
printf '%s\n' "${message//\{path\}/$bundle}"
