#!/bin/bash
# 在专用钥匙串里创建本地开发签名证书。专用钥匙串的密码保存在仅本人可读的文件里，
# 所以 SSH 会话也能解锁并签名（登录钥匙串在 SSH 下无法交互解锁）。
# 证书不需要被系统信任：macOS 权限按"标识符 + 证书叶哈希"识别，只要证书不变，重建后授权仍有效。
set -euo pipefail
script_dir=$(dirname "${BASH_SOURCE[0]}") || exit 1
root=$(cd "$script_dir/../.." && pwd) || exit 1
source "$root/scripts/mac/messages.sh" || exit 1
identity="Mousetroke Dev"
keychain="$HOME/Library/Keychains/mousetroke-dev.keychain-db"
secret_dir="$HOME/.config/mousetroke-dev"
secret="$secret_dir/keychain-password"
if [[ -f "$keychain" && -f "$secret" ]]; then
    mac_message mac.cert.exists || exit 1
    exit 0
fi
mac_message mac.cert.instructions || exit 1
mkdir -p "$secret_dir" || exit 1
chmod 700 "$secret_dir" || exit 1
temporary=$(mktemp -d) || exit 1
trap 'status=$?; rm -f "$temporary/key.pem" "$temporary/cert.pem" "$temporary/cert.p12" "$temporary/openssl.cnf" || status=1; rmdir "$temporary" || status=1; exit "$status"' EXIT
chmod 700 "$temporary" || exit 1
# 系统临时目录若被调用者指到仓库，拒绝在其中生成私钥。
command -v git >/dev/null 2>&1 || { printf 'git is required to check the temporary directory\n' >&2; exit 1; }
if git -C "$temporary" rev-parse --git-dir >/dev/null 2>&1; then
    mac_message mac.release.cert_outside >&2
    exit 2
fi
password=$(openssl rand -hex 24) || exit 1
[[ -n "$password" ]] || { printf 'Empty keychain password\n' >&2; exit 1; }
(umask 077; printf '%s' "$password" > "$secret") || exit 1
security create-keychain -p "$password" "$keychain" || exit 1
# 不自动上锁；bundle.sh 每次签名前仍会显式解锁
security set-keychain-settings "$keychain" || exit 1
security unlock-keychain -p "$password" "$keychain" || exit 1
cat > "$temporary/openssl.cnf" <<'CONFIG' || exit 1
[req]
distinguished_name = subject
x509_extensions = signing
prompt = no
[subject]
CN = Mousetroke Dev
O = Mousetroke Local Development
[signing]
basicConstraints = critical,CA:FALSE
keyUsage = critical,digitalSignature
extendedKeyUsage = critical,codeSigning
subjectKeyIdentifier = hash
CONFIG
openssl req -new -newkey rsa:2048 -nodes -x509 -days 3650 \
    -config "$temporary/openssl.cnf" -keyout "$temporary/key.pem" -out "$temporary/cert.pem" || exit 1
p12=$(openssl rand -hex 24) || exit 1
[[ -n "$p12" ]] || { printf 'Empty PKCS#12 password\n' >&2; exit 1; }
# 显式使用 macOS security import 也能读取的 PKCS#12 算法，兼容 OpenSSL 3。
openssl pkcs12 -export -inkey "$temporary/key.pem" -in "$temporary/cert.pem" \
    -name "$identity" -out "$temporary/cert.p12" -passout "pass:$p12" \
    -keypbe PBE-SHA1-3DES -certpbe PBE-SHA1-3DES -macalg sha1 || exit 1
security import "$temporary/cert.p12" -k "$keychain" -P "$p12" -T /usr/bin/codesign >/dev/null || exit 1
# 允许 codesign 无提示使用私钥
security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$password" "$keychain" >/dev/null || exit 1
# 加入用户钥匙串搜索列表（保留原有项）
existing=()
keychains=$(security list-keychains -d user) || exit 1
while IFS= read -r line; do
    line=${line#*\"}; line=${line%\"*}
    if [[ -n "$line" && "$line" != "$keychain" ]]; then existing+=("$line"); fi
done <<< "$keychains" || exit 1
security list-keychains -d user -s "$keychain" ${existing[@]+"${existing[@]}"} || exit 1
mac_message mac.cert.created
