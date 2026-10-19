#!/bin/bash
# 在专用钥匙串里创建本地开发签名证书。专用钥匙串的密码保存在仅本人可读的文件里，
# 所以 SSH 会话也能解锁并签名（登录钥匙串在 SSH 下无法交互解锁）。
# 证书不需要被系统信任：macOS 权限按"标识符 + 证书叶哈希"识别，只要证书不变，重建后授权仍有效。
set -euo pipefail
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
source "$root/scripts/mac/messages.sh"
identity="Mousetroke Dev"
keychain="$HOME/Library/Keychains/mousetroke-dev.keychain-db"
secret_dir="$HOME/.config/mousetroke-dev"
secret="$secret_dir/keychain-password"
if [[ -f "$keychain" && -f "$secret" ]]; then
    mac_message mac.cert.exists
    exit 0
fi
mac_message mac.cert.instructions
mkdir -p "$secret_dir" "$root/target/mac"
chmod 700 "$secret_dir"
temporary=$(mktemp -d "$root/target/mac/dev-cert.XXXXXX")
chmod 700 "$temporary"
trap 'rm -f "$temporary/key.pem" "$temporary/cert.pem" "$temporary/cert.p12" "$temporary/openssl.cnf"; rmdir "$temporary"' EXIT
password=$(openssl rand -hex 24)
(umask 077; printf '%s' "$password" > "$secret")
security create-keychain -p "$password" "$keychain"
# 不自动上锁；bundle.sh 每次签名前仍会显式解锁
security set-keychain-settings "$keychain"
security unlock-keychain -p "$password" "$keychain"
cat > "$temporary/openssl.cnf" <<'CONFIG'
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
    -config "$temporary/openssl.cnf" -keyout "$temporary/key.pem" -out "$temporary/cert.pem"
p12=$(openssl rand -hex 24)
# 显式使用 macOS security import 也能读取的 PKCS#12 算法，兼容 OpenSSL 3。
openssl pkcs12 -export -inkey "$temporary/key.pem" -in "$temporary/cert.pem" \
    -name "$identity" -out "$temporary/cert.p12" -passout "pass:$p12" \
    -keypbe PBE-SHA1-3DES -certpbe PBE-SHA1-3DES -macalg sha1
security import "$temporary/cert.p12" -k "$keychain" -P "$p12" -T /usr/bin/codesign >/dev/null
# 允许 codesign 无提示使用私钥
security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$password" "$keychain" >/dev/null
# 加入用户钥匙串搜索列表（保留原有项）
existing=()
while IFS= read -r line; do
    line=${line#*\"}; line=${line%\"*}
    [[ -n "$line" && "$line" != "$keychain" ]] && existing+=("$line")
done < <(security list-keychains -d user)
security list-keychains -d user -s "$keychain" "${existing[@]}"
mac_message mac.cert.created
