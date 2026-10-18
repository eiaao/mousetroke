#!/usr/bin/env bash
# 准备 UI 冒烟静态目录（git-bash / Windows 版）：ui/dist + mock.js（注入到 index.html）+ config.json
# 用法：bash stage.sh [语言]   （语言会写进 mock 的 init_state 返回值；缺省 zh-CN）
set -e
R="$(cd "$(dirname "$0")/../.." && pwd)"
D="$R/target/uistage"
LANG_TAG="${1:-zh-CN}"
rm -rf "$D"
mkdir -p "$D"
cp -r "$R/crates/mousetroke-settings/ui/dist/." "$D/"
cp "$R/target/uitest/mock.js" "$D/"
cp "$R/target/uitest/config.json" "$R/target/uitest/config-en.json" "$D/"
grep -q mock.js "$D/index.html" || sed -i "s#<head>#<head><script src=\"mock.js\"></script>#" "$D/index.html"
# 语言通过 URL 参数注入（i18n.initLang 支持 ?lang=），config 固定一份
echo "staged $D (lang=$LANG_TAG)"
