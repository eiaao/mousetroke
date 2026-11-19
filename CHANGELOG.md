# Changelog

Each version lists its highlights in English, then in Chinese. The release page, the in-app update notes and the website changelog are all generated from this file.

## 0.3.5 — 2026-11-19

### English

- Mac: apps in the sidebar and settings now show their name as in Finder (for example "Windows App") instead of a bundle identifier such as `com.microsoft.rdc.macos`. Apps added earlier are renamed when settings opens; names you changed yourself are kept. (#108)
- Moving a gesture to a single other app with **Move to…** keeps its usage stats together. Previously the stats split into two rows and the old one showed as deleted. (#109)

### 中文

- Mac：侧栏和设置中的程序显示访达里的名称（例如“Windows App”），不再显示 `com.microsoft.rdc.macos` 这类程序标识。之前添加的程序在打开设置时自动改名，你自己改过的名称保持不变。（#108）
- 用“移动到…”把手势移到另一个程序时，使用统计保持为同一条；之前会拆成两条，旧的一条显示为“已删除”。（#109）

## 0.3.4 — 2026-11-18

### English

- New **Stats** page in settings: a yearly calendar of how often gestures, screen edges and key triggers ran (each square is a day, darker means more), plus your most used gestures, apps and hours of the day. Click a day to see just that day. Counts are only kept on your computer. (#106)
- Traditional Chinese (Taiwan) interface and user guide. **System default** picks it for Taiwan, Hong Kong, Macau and Traditional Chinese system languages, and documentation links open the Traditional Chinese guide on the website. Contributed by Amppa. (#88, #96)
- Right-click a gesture in the list and choose **Copy to / Move to…** to put it into one or more other apps. If an app already has the same drawing, choose to skip or replace it; the whole operation can be undone. **Duplicate** moved into the same right-click menu (Ctrl+D / ⌘+D still works). Contributed by Amppa. (#90)
- **Preferences → Appearance** adds **Interface theme** (System, Light or Dark) and **Prompt text size** for the gesture name shown while drawing (16–96 px, default 24 px). Contributed by Amppa. (#89, #92)
- The gesture editor now floats over the list instead of narrowing it; click outside the editor to close it. Contributed by Amppa. (#91)
- **Show gesture list** now works: it pops up the gestures available in the current app with their drawings, and choosing one runs it. (#102)
- **Open or switch to app** can be set to **Run as administrator** (Windows), directly in the action list or in the gesture details. A **Choose from list** button picks the app from running programs and Start menu programs. (#104)
- When an administrator window comes to the front and Mousetroke is not running as administrator, a system notification explains why gestures don't work there and offers **Restart as administrator**. While Mousetroke runs as administrator, apps, commands and links it opens still run with normal rights unless the action asks for administrator. (#101)
- If Windows silently removes Mousetroke's input hooks (for example after the system was busy), they are reinstalled automatically and gestures keep working. (#101)
- Right-clicking a gesture in the list only opens the menu; it no longer opens the details panel. (#103)
- Lua: on Windows, `GetExplorerPath`, `ToggleKeys` and clipboard formats other than text (HTML, files) now work; on Mac, `KeysDown` / `KeysUp` and built-in display brightness. (#99, #100)
- Solid buttons have better contrast in the dark theme. **Undo** on a notification only undoes that operation and disappears once anything else changes. (#98)

### 中文

- 设置新增“统计”页：按年显示触发日历（每格一天，颜色越深次数越多），以及最常用的手势、按程序和按小时的统计；点某一天可只看当天。统计只保存在本机。（#106）
- 新增繁体中文（台湾）界面和使用指南。系统语言为台湾、香港、澳门或繁体中文时，“跟随系统”会选用繁体；文档链接打开网站上的繁体指南。由 Amppa 贡献。（#88、#96）
- 在手势列表中右键单击手势，选择“复制到／移动到…”，可放到一个或多个其他程序。目标程序已有相同画法时可选择跳过或取代，整次操作可撤销。“复制一份”移入同一右键菜单（仍可按 Ctrl+D／⌘+D）。由 Amppa 贡献。（#90）
- “偏好设置 → 外观”新增“界面主题”（跟随系统、浅色、深色）和画手势时手势名的“提示文字大小”（16–96 px，默认 24 px）。由 Amppa 贡献。（#89、#92）
- 手势编辑面板改为浮在列表上方，不再挤窄列表；点击面板外即可收起。由 Amppa 贡献。（#91）
- “显示手势列表”动作可以用了：弹出当前程序可用的手势及画法，选中即执行。（#102）
- “打开或切换到程序”可设置“以管理员身份运行”（Windows），在动作列表或手势详情里都能勾选；新增“从列表选择”，可从正在运行的程序和开始菜单程序中选取。（#104）
- Mousetroke 未以管理员身份运行时，管理员窗口来到前台会弹出系统通知，说明手势在其中无法使用，并提供“以管理员身份重启”。Mousetroke 以管理员身份运行时，它打开的程序、命令和网址仍以普通权限运行，除非动作勾选了“以管理员身份运行”。（#101）
- Windows 静默摘除 Mousetroke 的输入钩子时（例如系统繁忙之后）会自动重装，手势不再失效。（#101）
- 在手势列表中右键单击只弹出菜单，不再打开详情面板。（#103）
- Lua：Windows 支持 `GetExplorerPath`、`ToggleKeys` 及文本以外的剪贴板格式（HTML、文件）；Mac 支持 `KeysDown`／`KeysUp` 与内建屏幕亮度。（#99、#100）
- 深色主题下实心按钮的对比度更高。提示条上的“撤销”只撤销对应的那次操作，之后有任何其他修改即收起。（#98）

## 0.3.3 — 2026-11-16

### English

- New versions are easier to notice: the settings sidebar shows a dot on **About**, a banner appears at the top of the settings window, and the tray menu (menu bar on Mac) lists the new version. On Windows, a system notification appears once per new version; turn it off in Preferences or choose **Ignore this version**.
- Screen edges and corners now work on every display in multi-monitor setups with different resolutions or offsets. Only outer edges where the pointer stops count; the boundary between two displays, and corners that touch another display, do not trigger. (#81)
- Keys sent by Mousetroke on Windows now carry hardware scan codes, so web pages and browser extensions that read `KeyboardEvent.code` respond to them. Home, arrows and other navigation keys are no longer mistaken for numpad keys. (#84)
- Gestures now work in the settings window, except inside the gesture recording area, which still records your stroke. Gestures no longer interfere with Mousetroke's own tray menu.
- The About page links to the author's X account.

### 中文

- 新版本更容易发现：设置侧栏的“关于”上显示圆点，设置窗口顶部出现提示条，托盘菜单（Mac 为菜单栏）列出新版本。Windows 上每个新版本只弹出一次系统通知，可在偏好设置中关闭，或选择“忽略此版本”。
- 多显示器分辨率或位置不同时，每块显示器的屏幕边角都能触发。只认鼠标会停住的外侧边缘；两块显示器的交界处、紧挨另一块屏的角不触发。（#81）
- Windows 上 Mousetroke 发送的按键带上硬件扫描码，依赖 `KeyboardEvent.code` 的网页和浏览器扩展可以正常响应；Home、方向键等导航键不再被识别为小键盘键。（#84）
- 设置窗口中也能使用手势，只有手势录制区域内仍用于录制笔画；手势不再干扰 Mousetroke 自己的托盘菜单。
- 关于页加入作者的 X 账号链接。

## 0.3.2 — 2026-11-15

### English

- Fullscreen pause can now be turned off globally or overridden per app. Mouse gestures and screen-edge triggers share the same policy; existing configurations keep pausing in fullscreen by default. (#79)
- The app page header is redesigned: **Use gestures**, **Also use global gestures** and **In fullscreen** sit together in one settings bar, and search shares the toolbar row with the tabs.

### 中文

- 全屏暂停支持全局关闭和按程序设置例外，鼠标手势与触发角共用同一策略；已有配置仍默认在全屏时暂停。（#79）
- 程序页顶部重新设计：“使用手势”“也使用通用手势”“全屏时”集中到一条设置栏，搜索框与分类标签同在一行。

## 0.3.1 — 2026-11-14

### English

- Fixed vertical or horizontal strokes incorrectly triggering diagonal gestures when the corresponding straight gestures were disabled. Diagonal minimize and maximize gestures now require the matching direction while retaining tolerance for small angle and length variations. (#73)

### 中文

- 修复停用对应的上下左右手势后，直划可能误触发斜向手势的问题。斜向最小化、最大化等手势现在要求方向一致，同时保留轻微角度偏差和长度变化的容差。（#73）

## 0.3.0 — 2026-11-08

### English

- Automatic update checks now read a small `latest.txt` version marker before querying GitHub release information. The release API remains authoritative; manual checks continue to use only the API.
- Gesture library: define a gesture once and use it in several apps with different actions. Re-recording a shared gesture updates every app at once, with one set of samples. Editing a shared gesture shows where it is used and any new conflicts; **Only change this one** keeps the edit to the current app. Existing setups are left untouched until you run **Organize and import**, which merges identical gestures and lets you confirm similar shapes; new installs start with the built-in gestures already in the library. Older versions still read the configuration normally.
- The action picker has a **Send a shortcut** entry: record a key combination such as Ctrl+Shift+K right in the picker and use it as the action.

### 中文

- 自动更新检查先读取小型 `latest.txt` 版本标记，再查询 GitHub 发布信息；始终以发布 API 为准，手动检查仍只请求 API。
- 手势库：同一个手势定义一次，即可在多个程序里执行不同动作；重录共享手势时所有程序同时生效，样本只有一份。修改共享手势时会显示用到它的位置和新产生的冲突，「只改这里」可把修改只留给当前程序。已有配置在运行「整理并导入」之前保持原样：相同的手势自动合并，相似的形状由你确认；新安装的内置手势直接放在手势库中。旧版本仍能正常读取配置。
- 动作选择器新增「发送快捷键」：在选择器里直接录制一个组合键（如 Ctrl+Shift+K）作为动作。

## 0.2.2 — 2026-10-27

### English

- Action library: a searchable picker with about 87 presets, including window layout, text and power actions; shared platform defaults and config v2 upgrades with automatic backup.
- New gestures stay inactive until their recording and action are complete; imported and existing single-click bindings keep working.
- Text actions protect non-text clipboard data and offer a configurable wait before restoring clipboard text. Preset forms validate parameters before saving, and shortcut searches also match action names, descriptions and aliases.
- Startup stops with an error if configuration cannot be read; search and translation URL templates are respected. Windows app switching restores minimized windows, skips desktop and shell windows, and preserves administrator launches. Mac actions no longer wait indefinitely.
- User guides in all ten interface languages: German, Spanish, French, Italian, Japanese, Korean, Brazilian Portuguese and Russian join English and Simplified Chinese. In-app help links follow the interface language.
- Website moved to [mousetroke.com](https://mousetroke.com/).
- Lua: add Mac key-state queries and AX window discovery/state control; check shortcut modifiers on both platforms and return nil on brightness failure (legacy Windows wrappers retain -1). Mac topmost and numeric brightness remain unsupported.
- Complete English and Simplified Chinese user guides, with Lua basics, API reference, examples, send-keys syntax and variable help links in the action editor.

### 中文

- 动作库：支持搜索的动作选择器，约 87 项预置动作，包含窗口布局、文字和电源操作；合并两端默认配置，配置升级到 v2 并自动备份。
- 新建手势在录制和动作完整前不会生效；导入和已有的单击绑定继续正常工作。
- 文字动作保护剪贴板中的非文本数据，可设置恢复剪贴板文本前的等待时间。预置动作表单在保存前校验参数，快捷键搜索也会匹配动作名称、说明和别名。
- 配置无法读取时，启动会报错并停止；搜索和翻译正确使用指定的网址模板。Windows 切换程序时恢复最小化窗口，跳过桌面和系统外壳窗口，并保留管理员启动行为。Mac 动作不再无限等待。
- 使用指南补齐全部十种界面语言：在英文、简体中文之外新增德语、西班牙语、法语、意大利语、日语、韩语、巴西葡萄牙语和俄语；应用内帮助链接跟随界面语言跳转。
- 网站迁至 [mousetroke.com](https://mousetroke.com/)。
- Lua：补齐 Mac 按键状态查询及 AX 窗口查找／状态控制，两平台检查快捷键修饰键，亮度失败统一返回 nil（Windows 旧接口保留 -1）；Mac 置顶和数值亮度控制仍不支持。
- 新增完整的简体中文和英文使用指南，并在动作编辑器中加入 Lua 入门、API 参考、示例、按键语法说明和变量说明链接。

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
