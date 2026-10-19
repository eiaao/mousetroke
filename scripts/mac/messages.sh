#!/bin/bash
# 脚本提示复用十种语言词条；只读取没有 JSON 转义的单行脚本文案。
mac_message() {
    local language="${LC_ALL:-${LC_MESSAGES:-${LANG:-en_US}}}"
    case "$language" in
        zh*) language=zh-CN ;; es*) language=es-ES ;; fr*) language=fr-FR ;;
        de*) language=de-DE ;; ja*) language=ja-JP ;; pt*) language=pt-BR ;;
        ru*) language=ru-RU ;; it*) language=it-IT ;; ko*) language=ko-KR ;;
        *) language=en-US ;;
    esac
    local message
    message=$(awk -v key="$1" '$1 == "\"" key "\":" { sub(/^[^:]*: "/, ""); sub(/",?$/, ""); print; exit }' "$root/i18n/$language.json") || return 1
    [[ -n "$message" ]] || { printf 'Missing macOS message: %s\n' "$1" >&2; return 1; }
    printf '%s\n' "$message"
}
