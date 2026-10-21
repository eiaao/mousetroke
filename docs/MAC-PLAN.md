# Mousetroke macOS 版规划

> 状态：规划稿（2026-10-18，已按测试条件修订），待确认后按里程碑实施。
> 测试条件：Mac mini（M6，Apple 芯片，macOS 27.0，无内置屏）+ 普通三键鼠标；MacBook Pro 2013（Intel，内置触控板，最高 macOS 11 Big Sur）；暂无 Apple 开发者账号。
> 目标：在同一仓库里提供 macOS 版，手势体验与 Windows 版一致，配置格式、识别算法、界面、翻译共用。

## 1. 结论先行

- **同一仓库，按平台拆 crate**：共享 `mousetroke-core`（配置 / 识别 / 匹配 / 按键语法 / i18n）和新拆出的 `mousetroke-engine`（手势状态机），平台层 `mousetroke-win` 与新建的 `mousetroke-mac` 并列。
- **先做一次不改行为的重构**（M0）：把 `mousetroke-win/src/engine.rs` 里的状态机抽成平台无关的 crate。它现在通过 `Host` / `Canvas` 两个接口与系统交互，直接调用 Windows API 的只有 3 处，抽离成本低，且 27 个状态机测试可以原样带走，Mac 版一开始就有回归保护。
- **macOS 平台层**：事件拦截用 CGEventTap，轨迹用透明 NSWindow + Core Animation，窗口操作用 Accessibility（AX）API，菜单栏图标用 NSStatusItem。
- **签名分两阶段**：开发和测试版用**自签名代码签名证书**（免费，签名身份固定，"辅助功能"授权不会因重新编译或更新而失效）；正式版再用 Apple 开发者账号（$99/年）做 Developer ID 签名和公证，去掉首次打开时的系统拦截。
- **必须有一台真 Mac 做验收**：CI 能编译、跑纯逻辑测试，但事件拦截、权限弹窗、轨迹绘制只能在真机上验证。

## 2. 仓库结构（目标）

```
crates/
  mousetroke-core/       不变：schema、recognizer、shape、matcher、keyseq、i18n、presets…
  mousetroke-engine/     新：手势状态机（现 mousetroke-win/src/engine.rs 主体）+ Host/Canvas/Clock 接口 + 状态机测试
  mousetroke-win/        Windows 平台层：hooks、input、overlay、targeting、tray、autostart、brightness、lua 的平台部分
  mousetroke-mac/        新：macOS 平台层（同上各模块的 macOS 实现）
  mousetroke-settings/   Tauri 设置界面：icons.rs / picker.rs / 自启动按 cfg(target_os) 分平台实现
  mousetroke/            可执行入口：按平台选择平台层；Windows 产出 exe，macOS 产出 .app
```

规模参考（现有代码）：core 约 3500 行、engine 约 1200 行 + 650 行测试，可直接共享；需要为 macOS 重写的平台层约 2800 行，另有设置界面的 icons/picker 约 650 行。

## 3. 共享与平台差异

| 能力 | Windows 现状 | macOS 方案 | 备注 |
|---|---|---|---|
| 全局拦截鼠标/键盘 | WH_MOUSE_LL / WH_KEYBOARD_LL | `CGEventTapCreate`（session tap，可修改事件），回调返回 NULL 吞掉事件 | 回调同样必须零阻塞；被系统因超时禁用时（`kCGEventTapDisabledByTimeout`）要立即重新启用 |
| 补发普通右键 | SendInput + INJECT_TAG | `CGEventPost`，用 `kCGEventSourceUserData` 字段打标记识别回声 | 回声抑制、熔断逻辑在 engine 里共用 |
| 轨迹与手势名 | 分层窗口 + GDI 软渲染 | 每块屏幕一个透明无边框 `NSWindow`（`ignoresMouseEvents`、加入所有桌面空间与全屏辅助），`CAShapeLayer` 画线、`CATextLayer` 画标签 | GPU 合成，Retina 按 `backingScaleFactor` 缩放；AppKit 只能在主线程调用 |
| 鼠标下的窗口/程序 | WindowFromPoint + 进程路径 | `CGWindowListCopyWindowInfo` 命中测试 + `NSRunningApplication`（bundle id、可执行路径） | 程序匹配在 macOS 上用 bundle id（如 `com.google.Chrome`） |
| 全屏自动暂停 | 窗口覆盖整个显示器 | 前台窗口尺寸等于屏幕且位于全屏空间；演示模式用 `NSApplication.presentationOptions` 判断 | |
| 发送按键 | SendInput（VK 码） | `CGEventCreateKeyboardEvent`（kVK 虚拟键码），VK→kVK 映射表放在 core | 配置里继续存 VK 码作为平台无关的"逻辑键" |
| 窗口操作 | ShowWindow / SetWindowPos | AX：`AXMinimized`、全屏属性、缩放按钮、`AXRaise`、关闭按钮 `AXPress` | **"窗口置顶"在 macOS 没有对其他程序可用的公开 API**，标为不支持 |
| 强制结束 | taskkill / TerminateProcess | `NSRunningApplication.forceTerminate` | |
| 打开程序 / 网址 / 命令 | CreateProcess / ShellExecute | `NSWorkspace` 打开应用和 URL；命令用 `/bin/zsh -c`（完整路径） | 沿用"系统程序用完整路径"的安全规则 |
| 媒体键 | VK_MEDIA_* | 系统定义事件（`NSEventTypeSystemDefined`，子类型 8，`NX_KEYTYPE_*`） | 音量也可直接走 CoreAudio |
| 屏幕亮度 | DDC/CI + WMI | 外接屏：Apple 芯片上通过 IOAVService 发 DDC/CI（MonitorControl、m1ddc 等开源工具的做法，私有接口）；内建屏：DisplayServices（私有框架） | Mac mini 没有内建屏，用户全用外接显示器，外接屏 DDC 进入 M3；均为"尽力而为"，失败时提示 |
| 选中文字变量 | 模拟 Ctrl+C 读剪贴板 | 先读 AX `kAXSelectedTextAttribute`，取不到再模拟 ⌘C 并恢复剪贴板 | |
| 弹出菜单 | TrackPopupMenu | `NSMenu.popUpMenuPositioningItem` 在光标处弹出 | |
| 托盘 | 通知区域图标 | 菜单栏 `NSStatusItem`（模板图标，跟随深浅色）；`LSUIElement` 不显示 Dock 图标 | |
| 开机自启 | 注册表 Run | macOS 13+ 用 `SMAppService.mainApp.register()`；macOS 11–12 回退到 `~/Library/LaunchAgents` 下的 LaunchAgent plist | 为覆盖 2013 MBP 测试机，最低支持 macOS 11 |
| 暂停快捷键 | RegisterHotKey | 在 event tap 里识别，或 Carbon `RegisterEventHotKey` | 默认 ⌃⌥⌘P（⌥⌘Esc 是系统"强制退出"，不能用） |
| Lua 脚本 | mlua | 同一份 mlua；`ms.*` 平台相关函数（窗口句柄、类名查找）在 macOS 上换成 CGWindowID / 程序名 | 少数 Windows 专属函数返回"不支持" |
| 配置与日志位置 | `%APPDATA%\mousetroke` | `~/Library/Application Support/mousetroke/`、日志 `~/Library/Logs/mousetroke/` | |

Rust 生态：`objc2` + `objc2-app-kit` / `objc2-foundation` / `objc2-core-graphics`（官方维护的类型安全绑定），Accessibility 用少量手写 FFI。

## 4. 权限（macOS 特有，决定首次体验）

- **辅助功能（Accessibility）**：可修改事件的 event tap 和 AX 窗口操作都需要。用 `AXIsProcessTrustedWithOptions` 检测并弹出系统提示。
- **输入监控（Input Monitoring）**：部分系统版本对键盘事件单独要求，用 `CGPreflightListenEventAccess` / `CGRequestListenEventAccess`。
- **引导流程**：首次启动的使用引导里加一步"授权"，用深链接 `x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility` 直接打开对应设置页；授权后自动检测、继续，不要求重启。没有授权时菜单栏图标显示提示状态。
- **授权跟代码签名绑定**：签名身份变了授权就失效，第一个公开测试版起复用同一自签名证书；M5b 再迁移 Developer ID。M5a 固定 designated requirement 为 bundle identifier + 自签名 certificate root 哈希，更新时用 `codesign -d -r-` 比较。迁移证书需要重新授权。

## 5. 产品层面的差异（需要单独设计）

1. **修饰键**：Windows 默认手势发 Ctrl+C、Ctrl+W；macOS 应该是 ⌘C、⌘W。方案：macOS 用单独的内置默认配置 `default-config-macos.json`；按键语法本身不变（字面量就是字面量）。跨平台导入配置时，提示"其中 N 个按键动作含 Ctrl，是否换成 ⌘"。
2. **触控板用户**：很多 Mac 用户没有鼠标，触控板很难"按住右键拖动"。鼠标（右键 / 中键 / 侧键）先在 M1–M2 做稳；引擎的"触发方式"做成可扩展，**M3 加入触控板触发**（候选方案：按住修饰键如 ⌥ + 移动光标），在 2013 MBP 的内置触控板上验证。测试用普通三键鼠标；妙控鼠标没有物理中键，不适合做主力测试设备；罗技 Options+ 等驱动可能接管侧键。
3. **屏幕边角**：macOS 自带"触发角"，菜单栏在上、程序坞在下。默认边角规则要重排（例如左上角调度中心、右下角显示桌面、上边加滚轮调音量），检测到系统已经设置了触发角时提示冲突。
4. **默认禁用程序清单**：换成 macOS 上需要右键/中键拖动的软件（Parallels、VMware Fusion、Windows App 远程桌面、Blender、Cinema 4D、Maya、Unity、Unreal 等）。
5. **界面文字**：快捷键显示用 ⌘⌥⌃⇧ 符号，"Win" 相关文案按平台切换；i18n 增加少量平台专属词条，10 种语言都要补。

## 6. 设置界面（Tauri）

- Tauri 2 在 macOS 上用 WKWebView，React 界面基本可以直接用。
- 平台相关后端命令按 `cfg(target_os)` 实现：程序图标（`NSWorkspace.icon(forFile:)` → PNG）、点选窗口（CGWindowList + 高亮浮层）、拖入 `.app`、自启动开关。
- 进程模型保持一致：常驻引擎很轻，设置窗口是同一个程序以 `settings` 参数启动的独立进程，关掉即退。

## 7. 打包与发布

- 产物：通用二进制（arm64 + x86_64，`lipo` 合并）打成 `Mousetroke.app`，再做成 DMG。x86_64 在 2013 MBP（Intel，macOS 11）上实测。
- **测试版（无账号）**：CI 用存放在 GitHub Secrets 里的同一张自签名证书签名（授权可跨版本保留）。用户首次打开会被 Gatekeeper 拦截，需要到"系统设置 → 隐私与安全性"点"仍要打开"，每个新下载的版本都要再点一次；README 写清步骤。
- **正式版（有账号后）**：Developer ID Application 证书 + 加固运行时（hardened runtime）→ `notarytool` 公证 → `stapler` 装订，双击即可打开。只改 CI，不改代码。
- CI：在 `macos-latest` 上跑 core / engine 测试和构建；发布流程增加 macOS 任务，证书和公证凭据放 GitHub Secrets（证书 p12、密码、Team ID、App Store Connect API 密钥）。
- 后续：Homebrew Cask；自动更新用 Sparkle（可放到 1.0 之后）。
- 独立网站仓库的产品页、llms.txt，以及本仓库的 README 增加 macOS 下载与系统要求。

## 8. 里程碑

| 里程碑 | 内容 | 执行 / 验证 | 完成标准 |
|---|---|---|---|
| M0 | 抽出 `mousetroke-engine`（Windows 行为不变）；VK→kVK 映射与平台无关的屏幕/时钟接口放进共享层 | Codex / Claude：Windows 真机冒烟 + 全部测试 | Windows 版行为、性能不变；状态机测试在共享 crate 里跑 |
| M1 | macOS 骨架：菜单栏程序、权限检测与引导、event tap、普通右键补发、日志 | Codex / 你的 Mac 真机验收 | 右键单击菜单正常，右键拖动被拦截并在日志里看到轨迹 |
| M2 | 轨迹与手势名浮层、端到端识别、发送按键动作；触控板方案试验 | 同上 | 画 ← 在浏览器里后退，轨迹流畅、无残留 |
| M3 | 其余动作（窗口、程序、网址、命令、媒体、菜单、Lua）、屏幕边角、全屏暂停、按程序规则 | 同上 | 功能对照表逐项通过 |
| M4 | 设置界面在 macOS 上可用：图标、点选窗口、默认禁用清单、平台文案（10 种语言） | 同上 | 设置冒烟测试在 macOS CI 通过；真机走一遍引导 |
| M5a | 通用二进制、自签名、DMG、CI 发布测试版、产品页与 README 更新 | Codex / Claude：发布流程验证 | 下载 DMG，按说明"仍要打开"一次后正常运行；更新后授权保留 |
| M5b | 购买开发者账号后：Developer ID 签名 + 公证 | Codex / Claude | 双击打开无安全警告 |

每个里程碑一个或多个 PR，审查方式和 Windows 版的依赖升级一致：Codex 在独立 worktree 实现，Claude 审查并跑测试，真机部分由你在 Mac 上按清单验收。

**Mac 上的验证通道**（建议）：Mac mini 装 Xcode 命令行工具、Rust、git，并打开"远程登录"（SSH）。Claude 通过 SSH 在 Mac 上编译、跑测试，并用 `CGEventPost` 合成鼠标拖动做端到端测试；辅助功能授权由你在 Mac 上点一次。手感类验收（流畅度、误触）由你按清单完成。M0 不需要 Mac，可以先做。

## 9. 风险

| 风险 | 影响 | 应对 |
|---|---|---|
| 两台测试机系统跨度大（11 与 27） | 新接口在老系统上不可用、老接口在新系统上被弃用 | 新接口一律做版本判断加回退；两台机器都跑验收清单 |
| 暂无开发者账号 | 测试版首次打开有系统拦截，影响转化 | 自签名证书保证授权跨版本保留；正式推广前购买账号做公证（M5b） |
| event tap 被系统超时禁用 | 手势失效或输入阻塞 | 回调只记录禁用；先关闭两个 tap 并检查新鲜权限，撤权或 30 秒内第二次超时则移除并暂停，权限就绪后再恢复；见 MAC-PERMISSIONS.md |
| 亮度等私有 API | 系统更新后失效 | 标为尽力而为，失败时提示，不影响其他功能 |
| 触控板体验 | 大量 Mac 用户用不起来 | M2 真机试验修饰键触发方案 |
| 与系统手势、触发角冲突 | 误触或双触发 | 默认边角避开系统触发角，检测到冲突时提示 |

## 10. 已确认 / 待确认

已确认（2026-10-18）：
- 暂无 Apple 开发者账号 → 先自签名发测试版，正式版再公证（M5a / M5b）。
- 主测试机 Mac mini M6（macOS 27.0），配三键鼠标 → 外接屏 DDC 亮度进 M3。
- 副测试机 MacBook Pro 2013（Intel、触控板，最高 macOS 11）→ **最低支持 macOS 11 Big Sur**；Intel 与触控板在它上面验证，触控板触发进 M3。

- 暂停 / 恢复快捷键默认 **⌃⌥⌘P**（⌥⌘Esc 是系统"强制退出"，不能用）。
- 验证通道：两台 Mac 通过 Tailscale 与开发机互通，开启"远程登录"（SSH）后由 Claude 远程编译、测试。
