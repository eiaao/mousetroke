#!/bin/bash
set -euo pipefail
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
source "$root/scripts/mac/messages.sh"
identity="Mousetroke Dev"
if security find-identity -v -p codesigning | grep -F -- "\"$identity\"" >/dev/null; then
    mac_message mac.cert.exists
    exit 0
fi
mac_message mac.cert.instructions
keychain=$(security login-keychain | sed 's/^[[:space:]]*"//;s/"[[:space:]]*$//')
test -f "$keychain"
mkdir -p "$root/target/mac"
temporary=$(mktemp -d "$root/target/mac/dev-cert.XXXXXX")
chmod 700 "$temporary"
trap 'rm -f "$temporary/key.pem" "$temporary/cert.pem" "$temporary/cert.p12" "$temporary/openssl.cnf"; rmdir "$temporary"' EXIT
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
authorityKeyIdentifier = keyid:always
CONFIG
openssl req -new -newkey rsa:2048 -nodes -x509 -days 3650 \
    -config "$temporary/openssl.cnf" -keyout "$temporary/key.pem" -out "$temporary/cert.pem"
password=$(openssl rand -hex 24)
# 显式使用 macOS security import 也能读取的 PKCS#12 算法，兼容 OpenSSL 3。
openssl pkcs12 -export -inkey "$temporary/key.pem" -in "$temporary/cert.pem" \
    -name "$identity" -out "$temporary/cert.p12" -passout "pass:$password" \
    -keypbe PBE-SHA1-3DES -certpbe PBE-SHA1-3DES -macalg sha1
mac_message mac.cert.password
security import "$temporary/cert.p12" -k "$keychain" -P "$password" -T /usr/bin/codesign -T /usr/bin/security
security add-trusted-cert -r trustRoot -p codeSign -k "$keychain" "$temporary/cert.pem"
security find-identity -v -p codesigning | grep -F -- "\"$identity\""
mac_message mac.cert.created
