# Mac 权限恢复与回调审计

## 授权模型

- macOS 27 的 Mac mini 实测：同时重置两项 TCC 权限后，仅开启辅助功能，**同一引擎进程**的 `CGPreflightListenEventAccess` 随即变为 true，事件 tap 启用、手势可用；Mousetroke 无需列在输入监控中。监听预检为 true 不代表用户单独授予了输入监控。
- 首次引导只显示辅助功能的一行和一个授权按钮，解释其用于看见鼠标手势、操作窗口和发送快捷键。辅助功能开启后，若新鲜预检仍缺监听权限，或引擎 tap 仍无法启用，持续四秒才显示输入监控兜底；菜单栏采用相同条件。权限恢复或辅助功能撤销后隐藏兜底项。
- **准备就绪只取决于引擎两个事件 tap 已启用**；预检均为 true 但 tap 未就绪、引擎快照缺失或重启期间的空快照，都不能作为成功。

## 控制与恢复

- 授权、重启及更新退出使用 `NSDistributedNotificationCenter`，通知名以 `io.github.eiaao.mousetroke.request.` 开头，object 携带目标引擎 PID。发送前核对实例锁 PID，接收端按 PID 过滤并再次验证；收到后唤醒现有主线程计时器，正常空闲响应预算约 250ms。不再创建或轮询 `restart-request` / `quit-request`。
- 辅助功能按钮让运行中的引擎调用 AX prompt，然后打开面板；只有用户点击兜底按钮时才调用 `CGRequestListenEventAccess`，额外尝试一次临时 listen-only tap 以登记 TCC 列表。仅没有引擎时由设置进程本地请求。
- TCC 的辅助功能与监听预检都可能受进程缓存影响。引导未就绪、引擎正常两秒轮询和 tap 禁用处理共用同一可执行文件的内部 `--permission-status` 短命进程，静默查询当前权限；不初始化配置、日志、窗口或实例锁，不触发授权。探针两秒超时后终止并回收。周期轮询同时最多一个在途探针，等待在后台线程完成，主线程仅非阻塞接收结果，不阻塞活动 tap。收到禁用通知或用户授权请求时丢弃之前启动的结果。
- 正常两秒轮询在 AX 已就绪时尝试安装两个 tap，允许首次安装时监听预检仍为 false；成功安装是可靠的可用证据。运行中任一新鲜预检缺失时移除 tap、清除捕获和补发队列，并走故障暂停恢复流程；不再用已有 tap 覆盖新鲜撤权结果。故障后必须通过新鲜权限检查才重新安装，直接复用本轮探针，不另启重复探针。
- 引导看到新鲜权限均为 true、引擎仍未就绪，先等 2.3 秒正常恢复，再自动重启一次。自动重启六秒后仍未成功才显示手动按钮；重启期间空快照不能视为成功。
- 禁用回调立即进入放行状态并记录类型／时刻；最迟下一空闲拍关闭两个 tap，再做新鲜预检。权限缺失、探针失败或 30 秒内第二次超时，会移除所有端口和 run-loop source，经 `events::take_fault` 取消捕获、清除待补发队列、暂停、刷新菜单并发布未就绪状态。两次超时有 30 秒冷却，冷却后仍须新鲜权限就绪才允许正常两秒轮询重装。权限缺失本身无需等待冷却；重新安装后恢复故障前的用户暂停选择。

## tap 回调审计

| 路径 | 发现与处理 |
| --- | --- |
| AX、WindowServer、应用查询 | 原有目标、窗口标题、全屏状态已使用快照；AX 动作及窗口列表在工作线程。本次保留缓存命中测试，不把权限检查放进 tap 回调。 |
| 覆盖层 | 原有 `set_color`、会话开始／结束、标签绘制／清除会调用 AppKit／Core Animation，包括窗口显示、文字测量和事务提交。现由 `DeferredCanvas` 存入预分配的有界命令队列，`overlay_tick` 才访问真实画布；过载清屏，文本截断，虚拟屏矩形只读缓存。 |
| 日志与校准样本 | 原有 `SyncSender::try_send` 虽不等待队列容量，仍可能争用共享发送端内部锁；诊断 observer 也同步执行。现 tap 仅写预分配的线程本地日志队列，下一拍才发送到日志线程并调用 observer，写盘／控制台继续留在日志线程。引擎运行期间也将默认 panic hook 的同步 stderr 输出改送日志缓冲，退出后恢复原 hook。 |
| 唤醒与暂停热键 | 原有 `wake` 在回调里修改 NSTimer／NSDate，暂停可触发画布清理。现回调仅记线程本地活动和待切换标志；下一拍切换暂停。调试环境开关在启动时预读。 |
| 输入判断与副作用 | 事件字段转换、滚动余数、快照读取和引擎吞／放决策仍同步执行；使用线程本地 RefCell 的非等待借用，不持有共享互斥锁。按键补发、动作、菜单、剪贴板和进程启动原已排队，继续在回调后处理。CGEvent 的字段／source 读取只访问当前事件数据。 |

纯决策测试在 Windows 也运行，覆盖缺权、用户禁用、孤立超时、30 秒边界、第二次超时及过期恢复；另有画布延迟与队列过载、日志有界与 unwind 恢复测试。UI 单测与浏览器测试覆盖单行授权、仅辅助功能恢复、延迟显示输入监控、撤权后隐藏、预检全真但 tap 未就绪、空快照不算就绪、单次自动重启、失败、手动兜底、关闭清理和十种语言正常／兜底布局。Windows 检查不能替代真实 TCC／分布式通知验收。

## Mac mini（macOS 27）审查步骤

1. 使用正常证书签名并安装 `/Applications/Mousetroke.app`，同时重置 `Accessibility` 与 `ListenEvent` 后启动。记录 `~/Library/Logs/mousetroke/mousetroke.log` 中的引擎 PID。引导只应显示辅助功能，菜单也不显示输入监控。点击唯一的授权按钮，日志应在约 0.3 秒内出现对应 PID 的“引擎处理权限请求”，打开辅助功能面板。
2. **只开启辅助功能**，不授予输入监控、不重启；应在同一 PID 下启用两个 tap、显示就绪并进入使用引导。尝试右键画 `←`；输入监控列表没有 Mousetroke 也是正常结果。
3. 保持引擎运行并关闭设置窗口，在终端执行 `tccutil reset Accessibility io.github.eiaao.mousetroke`，先不做手势，观察正常两秒轮询也能记录辅助功能为 false、移除 tap 和故障暂停。重新打开引导应只请求辅助功能。撤权后持续普通左键点击、文字选择和拖动至少 50 秒，鼠标必须一直可用，不得反复盲目启用 tap；重新授权后正常安装或单次自动重启恢复，并保留设置窗口。
4. 在旧 macOS 或开发副本模拟 AX 为 true、监听为 false：前四秒不显示输入监控，之后引导和菜单才出现兜底。点击 Input Monitoring 的 Grant，应把 Mousetroke 登记到系统列表，无需按 `+` 查找；授权恢复后隐藏兜底。另测预检均为 true 但 tap 安装持续失败时，兜底按钮仍可点击。macOS 27 单独重置 `ListenEvent` 若 AX 仍使监听有效，不应强制显示输入监控。
5. 验证一般右键菜单、右键拖动手势、滚轮、暂停热键、全屏及轨迹／标签无残留。分别在故障前主动暂停和正常运行，确认正常恢复保留用户选择。
6. 自动重启失败路径可在开发副本暂时让 tap 安装返回失败：每次打开引导最多自动重启一次，六秒后才出现手动按钮；关闭引导不再触发重启。另可对引擎 `SIGSTOP` 超过 tap 超时阈值再 `SIGCONT`，30 秒内重复两次，检查第二次禁用进入冷却并完全移除端口，期间点击仍可用。
7. 引擎恢复后，发送携带旧 PID 的分布式重启通知，当前引擎不得退出；再从正常设置流程重启和更新退出，确认当前 PID 的控制请求仍能生效。检查应用目录不产生两个旧请求文件。

## Windows 侧验证

构建环境：`CARGO_TARGET_DIR=<target-dir>`，`CARGO_BUILD_JOBS=4`，沿用该 target 目录中的 Zig 交叉 C 工具链。`<target-dir>` 可替换为仓库外的任意目录。

| 检查 | 结果 |
| --- | --- |
| `cargo check --workspace --target aarch64-apple-darwin --all-targets --locked` | 通过 |
| `cargo clippy --workspace --target aarch64-apple-darwin --all-targets --locked -- -D warnings` | 通过 |
| `cargo test --workspace --locked` | 通过，223 项，另有 6 项原有忽略 |
| UI `npm test` / `npm run typecheck` / `npm run build` | 通过，10 项单测；构建保留原有包体积提示 |
| `node tools/uitest/mac-setup.mjs` | 通过，真实构建 UI 的单行授权、延迟兜底及十种语言正常／兜底布局 |
| `python scripts/i18n_lint.py --check` | 通过；更新全部十种语言的授权与恢复文案 |
| `python -m unittest discover -s scripts -p test_mac_packaging.py` | 通过，33 项 |
| `python site/gen.py` / `python site/check.py --allow-mac-size-placeholder` | 八种翻译均为 0 missed，十语言页面一致；沿用未测量 DMG 大小占位符 |
