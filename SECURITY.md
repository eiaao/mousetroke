# Security policy / 安全策略

## Reporting a vulnerability / 报告漏洞

Please **do not** open a public issue for security problems. Use GitHub's private
reporting instead: **Security → Report a vulnerability** on this repository.
We aim to reply within 7 days.

请不要在公开 issue 里报告安全问题，使用本仓库的 **Security → Report a vulnerability**
私下提交，我们会在 7 天内回复。

## Supported versions / 支持的版本

Only the latest release receives security fixes. 只有最新版本会收到安全修复。

## What Mousetroke does on your machine / 程序会做什么

- Installs low-level mouse and keyboard hooks (`WH_MOUSE_LL`, `WH_KEYBOARD_LL`) to
  recognize gestures. Key presses are only compared against your gesture bindings;
  they are never stored or sent anywhere. The log file never contains key codes.
- No network access, no telemetry, no account.
- Runs actions you configure: key sequences, programs, URLs, shell commands and Lua
  scripts. **Shell and Lua actions run with your full user rights** — only import
  configuration files from people you trust.
- Files: `%APPDATA%\mousetroke\config.json` (settings) and `mousetroke.log`
  (diagnostics, rotated at 1 MB). Start with Windows writes one value under
  `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.

- 安装低级鼠标、键盘钩子来识别手势。按键只用来和手势比对，不保存、不外传，日志里不记录按键。
- 不联网，没有遥测，不需要账号。
- 执行你配置的动作。**"运行命令"和 Lua 脚本拥有你的全部用户权限**，只导入可信来源的配置。
- 文件：`%APPDATA%\mousetroke\config.json`（配置）和 `mousetroke.log`（诊断日志，1 MB 轮换）；
  开机自启会在 `HKCU\...\Run` 写一个值。
