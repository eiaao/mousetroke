# Mac 权限恢复与回调审计

## 控制与恢复

- 授权、重启及更新退出使用 `NSDistributedNotificationCenter`，通知名以 `io.github.eiaao.mousetroke.request.` 开头，object 携带目标引擎 PID。发送前核对实例锁 PID，接收端按 PID 过滤并再次验证；收到后唤醒现有主线程计时器，正常空闲响应预算约 250ms。不再创建或轮询 `restart-request` / `quit-request`。
- 授权按钮让运行中的引擎调用 AX prompt／CGRequestListenEventAccess，然后打开面板；输入监控额外尝试一次临时 listen-only tap 以登记 TCC 列表。仅没有引擎时由设置进程本地请求。
- `CGPreflightListenEventAccess` 可能缓存正负结果。引导未就绪时，以同一可执行文件的内部 `--permission-status` 短命进程做静默预检；探针不初始化配置、日志、窗口或实例锁。设置查询在异步命令线程执行，探针两秒超时后终止。正常工作中的引擎不定期启动探针。
- 正常两秒轮询在 AX 已就绪时尝试安装两个 tap，即使输入监控预检仍为 false；成功安装是可靠的可用证据。引导看到新鲜权限均为 true、引擎仍未就绪，先等 2.3 秒正常恢复，再自动重启一次。自动重启六秒后仍未成功才显示手动按钮；重启期间空快照不能视为成功。
- 禁用回调立即进入放行状态并记录类型／时刻；最迟下一空闲拍关闭两个 tap，再做新鲜预检。权限缺失、探针失败或 30 秒内第二次超时，会移除所有端口和 run-loop source，经 `events::take_fault` 取消捕获、清除待补发队列、暂停、刷新菜单并发布未就绪状态。两次超时有 30 秒冷却，冷却后仍须新鲜权限就绪才允许正常两秒轮询重装。权限缺失本身无需等待冷却；重新安装后恢复故障前的用户暂停选择。

## tap 回调审计

| 路径 | 发现与处理 |
| --- | --- |
| AX、WindowServer、应用查询 | 原有目标、窗口标题、全屏状态已使用快照；AX 动作及窗口列表在工作线程。本次保留缓存命中测试，不把权限检查放进 tap 回调。 |
| 覆盖层 | 原有 `set_color`、会话开始／结束、标签绘制／清除会调用 AppKit／Core Animation，包括窗口显示、文字测量和事务提交。现由 `DeferredCanvas` 存入预分配的有界命令队列，`overlay_tick` 才访问真实画布；过载清屏，文本截断，虚拟屏矩形只读缓存。 |
| 日志与校准样本 | 原有 `SyncSender::try_send` 虽不等待队列容量，仍可能争用共享发送端内部锁；诊断 observer 也同步执行。现 tap 仅写预分配的线程本地日志队列，下一拍才发送到日志线程并调用 observer，写盘／控制台继续留在日志线程。引擎运行期间也将默认 panic hook 的同步 stderr 输出改送日志缓冲，退出后恢复原 hook。 |
| 唤醒与暂停热键 | 原有 `wake` 在回调里修改 NSTimer／NSDate，暂停可触发画布清理。现回调仅记线程本地活动和待切换标志；下一拍切换暂停。调试环境开关在启动时预读。 |
| 输入判断与副作用 | 事件字段转换、滚动余数、快照读取和引擎吞／放决策仍同步执行；使用线程本地 RefCell 的非等待借用，不持有共享互斥锁。按键补发、动作、菜单、剪贴板和进程启动原已排队，继续在回调后处理。CGEvent 的字段／source 读取只访问当前事件数据。 |

纯决策测试在 Windows 也运行，覆盖缺权、用户禁用、孤立超时、30 秒边界、第二次超时及过期恢复；另有画布延迟与队列过载、日志有界与 unwind 恢复测试。浏览器测试覆盖自动恢复、单次自动重启、失败、空快照、手动兜底、关闭清理和十种语言布局。Windows 检查不能替代真实 TCC／分布式通知验收。

## Mac mini（macOS 27）审查步骤

1. 使用正常证书签名并安装 `/Applications/Mousetroke.app`，启动引擎与设置引导。观察 `~/Library/Logs/mousetroke/mousetroke.log`，记录引擎 PID。引擎空闲后点击两种授权按钮，日志应在约 0.3 秒内出现对应 PID 的“引擎处理权限请求”；打开相应系统面板。关闭设置窗口后，菜单授权仍应可用。
2. 正常授权并确认手势可用；保持引擎运行，在终端执行 `tccutil reset ListenEvent io.github.eiaao.mousetroke`。持续做普通左键点击、文字选择和拖动，至少观察 50 秒，系统鼠标必须一直可用。日志应显示禁用后的即时权限、移除 tap 和故障暂停，不应反复盲目重新启用。菜单应显示未就绪。
3. 撤权后从引导点击 Input Monitoring 的 Grant。System Settings → Privacy & Security → Input Monitoring 应列出 Mousetroke；用户只需打开开关，无需按 `+` 手动寻找应用。若系统先显示自己的授权确认框，按正常系统流程确认。
4. 打开开关并返回引导，不点击 Restart：应由正常安装或单次自动重启带起手势，显示就绪并自动进入使用引导。记录新 PID（若重启）和 tap 启用日志；设置窗口保持打开。辅助功能也做一次撤权／重新授权的同样检查。
5. 验证一般右键菜单、右键拖动手势、滚轮、暂停热键、全屏及轨迹／标签无残留。分别在故障前主动暂停和正常运行，确认正常恢复保留用户选择。
6. 自动重启失败路径可在开发副本暂时让 tap 安装返回失败：每次打开引导最多自动重启一次，六秒后才出现手动按钮；关闭引导不再触发重启。另可对引擎 `SIGSTOP` 超过 tap 超时阈值再 `SIGCONT`，30 秒内重复两次，检查第二次禁用进入冷却并完全移除端口，期间点击仍可用。
7. 引擎恢复后，发送携带旧 PID 的分布式重启通知，当前引擎不得退出；再从正常设置流程重启和更新退出，确认当前 PID 的控制请求仍能生效。检查应用目录不产生两个旧请求文件。

## Windows 侧验证

构建环境：`CARGO_TARGET_DIR=D:\project\reaWGestures2\.claude\worktrees\target-mac-install`，`CARGO_BUILD_JOBS=4`，沿用该 target 目录中的 Zig 交叉 C 工具链。

| 检查 | 结果 |
| --- | --- |
| `cargo check --workspace --target aarch64-apple-darwin --all-targets --locked` | 通过 |
| `cargo clippy --workspace --target aarch64-apple-darwin --all-targets --locked -- -D warnings` | 通过 |
| `cargo test --workspace --locked` | 通过，223 项，另有 6 项原有忽略 |
| UI `npm test` / `npm run typecheck` / `npm run build` | 通过，9 项单测；构建保留原有包体积提示 |
| `node tools/uitest/mac-setup.mjs` | 通过，真实构建 UI 的浏览器交互与十种语言布局 |
| `python scripts/i18n_lint.py --check` | 通过；复用现有十种语言词条，无新增界面字符串 |
| `python -m unittest discover -s scripts -p test_mac_packaging.py` | 通过，33 项 |
