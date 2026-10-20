# Changelog

Each version lists its highlights in English, then in Chinese. The release page, the in-app update notes and the website changelog are all generated from this file.

## 0.2.1 — 2026-10-20

### English

- Mac: Update now downloads and verifies the new version, replaces the installed app and restarts it. No second "Open Anyway" is needed after this update, and permissions stay granted. If anything fails, the old version is kept; manual download remains available.
- Mac: first launch opens a short setup guide instead of two system prompts. Usually only Accessibility is needed; once it is on, gestures start right away without restarting.
- Mac: opening Mousetroke from the DMG offers to move it to Applications and reopen it. If an older copy is running, it can be quit and replaced; the old copy goes to the Trash.
- Mac: the DMG window shows where to drag the app and how to get past the first-open warning.
- Mac: if permissions are turned off while Mousetroke is running, it now pauses instead of slowing down the mouse.

### 中文

- Mac：支持应用内一键更新。下载并校验新版本后替换已安装的应用并重新打开，更新后不用再点"仍要打开"，已有权限保持不变。出错时保留旧版本，也可以改为手动下载。
- Mac：首次启动打开简短的设置引导，不再连续弹出两个系统授权框。通常只需打开辅助功能，打开后手势立即可用，无需重启。
- Mac：从 DMG 直接打开时，可一键移到"应用程序"并重新打开；旧版本正在运行时可退出并替换，旧副本移到废纸篓。
- Mac：DMG 窗口标明拖到哪里，并提示首次打开被拦截时怎么办。
- Mac：运行中关闭了权限时，Mousetroke 会自动暂停，不再拖慢鼠标。

## 0.2.0 — 2026-10-20

### English

- Mousetroke for Mac (beta): the same gestures, trails, screen corners and edges, per-app gestures and Lua scripts, in a menu-bar app for macOS 11 or later on Apple silicon and Intel. Defaults follow Mac conventions, such as Mission Control in the top-left corner and Spotlight in the bottom-left.
- The Mac settings window has the same interface, with ⌘⌥⌃⇧ key names, a window picker, launch at login and a warning when a macOS hot corner overlaps a Mousetroke corner.
- Windows: fixed the gesture trail sometimes not showing after the screen went to sleep or the display layout changed.
- Selecting a web address that starts with www. and using a search gesture now opens that address directly.
- Opening Settings again brings the existing window to the front instead of starting another one.
- Lower idle CPU and memory use, and a quieter log.

### 中文

- Mac 版（测试版）：同样的手势、轨迹、屏幕边角、按程序区分的手势和 Lua 脚本，常驻菜单栏，支持 macOS 11 及以上的 Apple 芯片和 Intel 机型。默认手势按 Mac 习惯设置，例如左上角打开调度中心、左下角打开聚焦搜索。
- Mac 设置窗口与 Windows 界面一致：按键显示为 ⌘⌥⌃⇧，支持点选窗口、登录时启动；系统触发角与 Mousetroke 边角冲突时会提醒。
- Windows：修复屏幕休眠唤醒或显示器布局变化后，手势轨迹偶尔不显示的问题。
- 选中以 www. 开头的网址再用搜索手势，会直接打开该网址。
- 再次打开设置时，直接切到已打开的窗口，不会再开一个。
- 降低空闲时的 CPU 与内存占用，日志更精简。

## 0.1.1 — 2026-10-19

### English

- Shape gestures: draw a curve such as G, C, O, S or 6 to run an action. Record the same shape 2–5 times and Mousetroke learns how you draw it.
- The recorder tells lines from curves by itself: a straight stroke becomes a line gesture, a curve becomes a shape. One click switches it if it guessed wrong.
- Recognition is tuned on real hand-drawn strokes: loose, wobbly curves are accepted, while straight gestures are never mistaken for shapes.
- Endless scribbling no longer keeps the gesture going: after 100 direction changes it is cancelled with a short message, and nothing runs.
- Check for updates: automatically once a day (can be turned off in Preferences) or from About. Updating takes one click: the new version is downloaded, its checksum verified, and Mousetroke restarts itself.
- About now links to the website, the source code and feedback.

### 中文

- 形状手势：画 G、C、O、S、6 这类曲线来执行动作。同一个形状录 2–5 次，Mousetroke 会记住你的画法。
- 录制时自动区分直线和曲线：直的录成轨迹，弯的录成形状；判断错了点一下就能切换。
- 识别阈值用真实手绘样本校准：画得随意、有点抖的曲线也能认出来，直线手势不会被误认成形状。
- 乱画不再无限持续：方向变化超过 100 次会自动取消并给个提示，不执行任何动作。
- 检查更新：默认每天自动检查一次（可在偏好设置里关闭），也可以在“关于”里手动检查。一键更新：自动下载、校验 SHA-256、替换并重启。
- “关于”页加入主页、源代码和反馈入口。

## 0.1.0 — 2026-10-18

### English

- First public release: mouse gestures with the right, middle or side buttons, about 50 built-in gestures, screen corners and edges, eight action types, per-app gestures, and an interface in ten languages.

### 中文

- 首个公开版本：右键、中键、侧键手势，约 50 个内置手势，屏幕边角触发，八类动作，按程序区分手势，界面支持十种语言。
