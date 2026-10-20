# 更新实现与本机回归

更新由 `mousetroke-win/src/update.rs` 和 `update/http.rs` 实现，不使用 Tauri updater。
默认每天向 GitHub 询问一次是否有新版本，可在偏好设置中关闭；不发送其他信息，无遥测。
关于页打开时只读取本地版本，手动点击调用 `check_update`。配置 `settings.update_check`
默认为 true；引擎的独立更新线程检查，每次尝试前将时间
写入配置旁的 `update-state.json`，失败也等待 24 小时。配置热加载只唤醒该线程。
通知成功提交给托盘后记录版本；点击通知通过 `settings --about` 启动/聚焦设置，
`Mousetroke::settings-about` 自动复位命名事件使已有窗口也能定位关于页。

设置命令通过 `spawn_blocking` 执行同步 WinHTTP，钩子回调不参与网络、文件或进程工作。
发布正文使用同一次 API 响应的 `body`，按界面语言选择中文或英文条目，缺中文时回退英文；
剥离 Markdown/HTML 后最多显示 12 条、每条 300 字符，不额外请求正文。
API 只连接 HTTPS 的 `api.github.com`；下载及其每次跳转只允许 HTTPS 的 `github.com`、
`objects.githubusercontent.com`、`release-assets.githubusercontent.com`，端口 443。
禁用自动重定向及自动登录，逐次检查 Location，不打印完整 URL。API/校验文件上限
1 MiB，程序上限 64 MiB；单次网络操作超时约 10 秒。下载在内存中完成后使用 BCrypt
验证 `SHA256SUMS.txt` 中精确的 `mousetroke.exe` 条目，再写入同目录 `.new`。

替换顺序：原程序 → `.old`，`.new` → 原程序名。移动失败尝试恢复原程序。
设置进程启动 `mousetroke.exe --after-update <引擎 PID，缺省 0> <设置 PID> --config <路径>`，
然后查找 `MousetrokeTray` 隐藏控制窗口，发送与托盘「退出」完全相同的
`WM_COMMAND/MENU_QUIT`。引擎卸载钩子并结束消息循环，设置命令成功后退出设置进程。
启动失败或退出消息发送失败会终止继任者并回滚。继任者最多等待旧进程合计 10 秒，
退出后才取得 `Mousetroke::single-instance`；超时不启动第二个引擎。
引擎取得互斥体后尝试清除 `.old`，失败忽略。不可写目录返回结构化错误，界面打开固定发布页。

校验和与程序来自同一 GitHub release：SHA-256 检测损坏/不匹配，不能代替独立签名。
断电、杀毒软件锁文件、进程挂起或 UIPI 阻止退出消息等情况仍需在真实桌面回归。
更新不会请求管理员权限。自动通知受 Windows 通知设置及托盘图标可用性影响。

## 本地端到端回归（交互式 Windows 桌面）

以下 PowerShell 命令从此工作树根目录执行；使用独立测试配置，避免修改日常配置。`<target-dir>` 可替换为仓库外的任意目录。
先从托盘退出已有引擎和设置窗口。仅 debug 构建读取 `MOUSETROKE_UPDATE_BASE`；
release 完全忽略它。测试服务只接受 `http://127.0.0.1:<端口>`，请求与跳转必须留在同一源。

```powershell
$env:CARGO_TARGET_DIR='<target-dir>'
$env:CARGO_BUILD_JOBS='2'
cargo build -p mousetroke --locked
New-Item -ItemType Directory -Force target/update-e2e | Out-Null
Copy-Item "$env:CARGO_TARGET_DIR/debug/mousetroke.exe" target/update-e2e/mousetroke.exe
python scripts/update-test-server.py --exe "$env:CARGO_TARGET_DIR/debug/mousetroke.exe" --version 999.0.0 --port 8770
```

保持服务运行，在第二个 PowerShell 窗口进入工作树根目录：

```powershell
$env:MOUSETROKE_UPDATE_BASE='http://127.0.0.1:8770'
& ./target/update-e2e/mousetroke.exe run --config "$PWD/target/update-e2e/config.json"
```

1. 首次启动及进入关于页时服务应没有请求。点击「检查更新」应显示 999.0.0。
2. 点击「立即更新」，确认旧引擎/设置退出、新引擎启动、手势与托盘正常，`.old` 已清除。
   测试服务发布同一个 debug 程序但宣称 999.0.0，只验证交接，不改变内置版本号。
3. 重启服务并加 `--bad-checksum`，再次更新：应显示校验失败，旧引擎继续工作且无 `.new`。
4. 停止服务后检查：应显示网络错误，可重试及打开下载页。以普通用户在只读目录测试
   更新：应打开 GitHub 发布页，原程序保持可用。
5. 保持有效服务，打开自动检查，约一次后台检查后出现通知；点击应定位关于页。
   关闭设置窗口后及已打开其他设置页时各测试一次。重启引擎不应重复请求或通知。
   查看独立配置旁的 `update-state.json`，测试日界线可在引擎退出后将 `last_check` 减少 86400。
   同一版本应不再通知。关闭开关后等待、重启均不应产生请求。
6. release 程序设置同一环境变量，确认仍只使用正式 HTTPS API，不能连接此 HTTP 服务。
   完成后从托盘退出测试引擎，Ctrl+C 停止服务，移除测试用环境变量。

不依赖桌面的文件替换回归：

```powershell
$env:CARGO_TARGET_DIR='<target-dir>'
$env:CARGO_BUILD_JOBS='2'
cargo test -p mousetroke-win --release -- --include-ignored --skip new_session_clears_interrupted_fade
```
