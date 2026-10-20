#!/bin/bash
# bundle 与 DMG 共用签名与架构校验规则；发布模式不允许回退到临时身份。

# 通用程序必须恰好包含 arm64 与 x86_64；CommandLineTools 的 -verify_arch 每次只接受一个架构。
verify_universal() {
    local binary="$1" architecture architectures
    for architecture in arm64 x86_64; do
        lipo "$binary" -verify_arch "$architecture" || return 1
    done
    architectures=$(lipo "$binary" -archs) || return 1
    [[ "$architectures" =~ ^[[:space:]]*(arm64[[:space:]]+x86_64|x86_64[[:space:]]+arm64)[[:space:]]*$ ]] || {
        printf 'Expected exactly arm64 and x86_64; got: %s\n' "$architectures" >&2
        return 1
    }
}

sign_mac() {
    local path="$1" identity="${MOUSETROKE_SIGN_ID:-}" keychain="${MOUSETROKE_SIGN_KEYCHAIN:-}"
    local secret="$HOME/.config/mousetroke-dev/keychain-password"
    local sign_args=() requirement=() fingerprint password identities certificate
    if [[ -z "$identity" && -f "$HOME/Library/Keychains/mousetroke-dev.keychain-db" && -f "$secret" ]]; then
        keychain="$HOME/Library/Keychains/mousetroke-dev.keychain-db"
        password=$(cat "$secret") || return 1
        security unlock-keychain -p "$password" "$keychain" || return 1
        unset password
        certificate=$(security find-certificate -c "Mousetroke Dev" -Z "$keychain") || return 1
        identity=$(awk '/SHA-1 hash:/ {print $NF; exit}' <<< "$certificate") || return 1
        [[ -n "$identity" ]] || { printf 'Missing development certificate fingerprint\n' >&2; return 1; }
    fi
    identity="${identity:-Mousetroke Dev}"
    if [[ -n "$keychain" ]]; then sign_args=(--keychain "$keychain"); fi
    if [[ "$identity" != - ]]; then
        # 先检查查询状态，再解析结果；避免条件管道吞掉失败或提前退出引发 SIGPIPE。
        identities=$(security find-identity -p codesigning ${keychain:+"$keychain"}) || return 1
        if ! grep -F -- "$identity" <<< "$identities" >/dev/null; then
            if [[ "${MOUSETROKE_REQUIRE_SIGNING:-0}" == 1 ]]; then
                mac_message mac.release.sign_required >&2
                return 1
            fi
            local warning
            warning=$(mac_message mac.bundle.adhoc_warning) || return 1
            printf '%s\n' "${warning//\{identity\}/$identity}" >&2
            identity=-
        fi
    fi
    if [[ "${MOUSETROKE_REQUIRE_SIGNING:-0}" == 1 && "$identity" == - ]]; then
        mac_message mac.release.sign_required >&2
        return 1
    fi
    if [[ "$identity" != - ]]; then
        # 按实际证书哈希固定 root，而非可重建的证书名称；自签名时 root 即 leaf。
        # identity 可为 SHA-1 或名称，find-identity 的精确行用于解析证书指纹。
        fingerprint=$(awk -v id="$identity" 'index($0, id) {print $2; exit}' <<< "$identities") || return 1
        [[ "$fingerprint" =~ ^[[:xdigit:]]{40}$ ]] || { printf 'Invalid signing certificate fingerprint\n' >&2; return 1; }
        requirement=(-r "=designated => identifier \"io.github.eiaao.mousetroke\" and certificate root = H\"$fingerprint\"")
        # 按指纹签名，避免重名证书歧义，也支持未加入系统信任的自签名身份。
        identity="$fingerprint"
    fi
    codesign --force --options runtime --timestamp=none --identifier io.github.eiaao.mousetroke --sign "$identity" \
        ${sign_args[@]+"${sign_args[@]}"} ${requirement[@]+"${requirement[@]}"} "$path" || return 1
    codesign --verify --strict --verbose=2 "$path" || return 1
}
