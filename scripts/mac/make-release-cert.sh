#!/bin/bash
# 私钥只在系统临时目录出现，密码通过交互输入，不回显、不写入仓库。
set -euo pipefail
umask 077
script_dir=$(dirname "${BASH_SOURCE[0]}") || exit 1
root=$(cd "$script_dir/../.." && pwd -P) || exit 1
source "$root/scripts/mac/messages.sh" || exit 1
[[ $# -eq 1 && "$1" == /* && ! -e "$1" && ! -L "$1" ]] || { mac_message mac.release.cert_usage >&2; exit 2; }
parent=$(dirname "$1") || exit 1
parent=$(cd "$parent" && pwd -P) || exit 1
filename=$(basename "$1") || exit 1
output="$parent/$filename"
# worktree 和主仓库都拒绝；父目录先解析符号链接。
common=$(git -C "$root" rev-parse --path-format=absolute --git-common-dir) || exit 1
repository=$(cd "$common/.." && pwd -P) || exit 1
case "$output" in "$repository"|"$repository"/*|"$root"|"$root"/*) mac_message mac.release.cert_outside >&2; exit 2 ;; esac
if git -C "$parent" rev-parse --git-dir >/dev/null 2>&1; then
    mac_message mac.release.cert_outside >&2
    exit 2
fi
mac_message mac.release.cert_password >&2 || exit 1
IFS= read -r -s password || { printf 'Could not read password\n' >&2; exit 2; }
printf '\n' >&2
[[ -n "$password" ]] || { printf 'Empty password\n' >&2; exit 2; }
mac_message mac.release.cert_confirm >&2 || exit 1
IFS= read -r -s confirm || { printf 'Could not read password confirmation\n' >&2; exit 2; }
printf '\n' >&2
[[ "$password" == "$confirm" ]] || { printf 'Passwords do not match\n' >&2; exit 2; }
temporary=$(mktemp -d) || exit 1
trap 'status=$?; rm -rf "$temporary" || status=1; unset password confirm; exit "$status"' EXIT
# 不信任调用者的 TMPDIR 设置，生成密钥前再次检查临时目录。
if git -C "$temporary" rev-parse --git-dir >/dev/null 2>&1; then
    mac_message mac.release.cert_outside >&2
    exit 2
fi
cat > "$temporary/openssl.cnf" <<'CONFIG' || exit 1
[req]
distinguished_name = subject
x509_extensions = signing
prompt = no
[subject]
CN = Mousetroke Self-Signed
O = Mousetroke
[signing]
basicConstraints = critical,CA:FALSE
keyUsage = critical,digitalSignature
extendedKeyUsage = critical,codeSigning
subjectKeyIdentifier = hash
CONFIG
openssl req -new -newkey rsa:2048 -nodes -x509 -sha256 -days 3650 \
    -config "$temporary/openssl.cnf" -keyout "$temporary/key.pem" -out "$temporary/cert.pem" 2>/dev/null || exit 1
# 通过环境传给 OpenSSL，避免密码出现在命令行；显式兼容 macOS PKCS#12 导入算法。
MOUSETROKE_P12_PASSWORD="$password" openssl pkcs12 -export \
    -inkey "$temporary/key.pem" -in "$temporary/cert.pem" -name "Mousetroke Self-Signed" \
    -out "$output" -passout env:MOUSETROKE_P12_PASSWORD \
    -keypbe PBE-SHA1-3DES -certpbe PBE-SHA1-3DES -macalg sha1 || exit 1
mac_message mac.release.cert_secrets || exit 1
# 仅打印命令，不运行 base64、不打印密码。剪贴板内容由维护者粘贴到 Secrets。
printf 'base64 < %q | tr -d '\''\\n'\'' | pbcopy\n' "$output"
