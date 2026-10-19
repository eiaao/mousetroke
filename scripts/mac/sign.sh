#!/bin/bash
# bundle 与 DMG 共用签名规则；发布模式不允许回退到临时身份。
sign_mac() {
    local path="$1" identity="${MOUSETROKE_SIGN_ID:-}" keychain="${MOUSETROKE_SIGN_KEYCHAIN:-}"
    local secret="$HOME/.config/mousetroke-dev/keychain-password"
    local sign_args=() requirement=() fingerprint
    if [[ -z "$identity" && -f "$HOME/Library/Keychains/mousetroke-dev.keychain-db" && -f "$secret" ]]; then
        keychain="$HOME/Library/Keychains/mousetroke-dev.keychain-db"
        security unlock-keychain -p "$(cat "$secret")" "$keychain"
        identity=$(security find-certificate -c "Mousetroke Dev" -Z "$keychain" | awk '/SHA-1 hash:/ {print $NF; exit}')
    fi
    identity="${identity:-Mousetroke Dev}"
    [[ -z "$keychain" ]] || sign_args=(--keychain "$keychain")
    if [[ "$identity" != - ]]; then
        if ! security find-identity -p codesigning ${keychain:+"$keychain"} | grep -F -- "$identity" >/dev/null; then
            if [[ "${MOUSETROKE_REQUIRE_SIGNING:-0}" == 1 ]]; then
                mac_message mac.release.sign_required >&2
                return 1
            fi
            local warning
            warning=$(mac_message mac.bundle.adhoc_warning)
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
        fingerprint=$(security find-identity -p codesigning ${keychain:+"$keychain"} | awk -v id="$identity" 'index($0, id) {print $2; exit}')
        [[ "$fingerprint" =~ ^[[:xdigit:]]{40}$ ]] || return 1
        requirement=(-r "=designated => identifier \"io.github.eiaao.mousetroke\" and certificate root = H\"$fingerprint\"")
        # 按指纹签名，避免重名证书歧义，也支持未加入系统信任的自签名身份。
        identity="$fingerprint"
    fi
    codesign --force --options runtime --timestamp=none --identifier io.github.eiaao.mousetroke --sign "$identity" \
        ${sign_args[@]+"${sign_args[@]}"} ${requirement[@]+"${requirement[@]}"} "$path"
    codesign --verify --strict --verbose=2 "$path"
}
