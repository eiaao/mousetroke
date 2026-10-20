"""对比页事实与双语文案；每条已核实事实附官方来源，空项不作推断。"""

CHECKED = "2026-10-21"


def bi(zh, en):
    return {"zh-CN": zh, "en": en}


SOURCES = {
    "strokeit": ("StrokeIt · Home", "https://www.tcbmi.com/strokeit/"),
    "strokeit-download": ("StrokeIt · Downloads", "https://www.tcbmi.com/strokeit/downloads.shtml"),
    "strokesplus": ("StrokesPlus.net · Archive", "https://www.strokesplus.net/"),
    "observer": ("Input.Observer · Features", "https://input.observer/"),
    "observer-story": ("Input.Observer · StrokesPlus history", "https://input.observer/strokesplus-story"),
    "observer-price": ("Input.Observer · Pricing & FAQ", "https://input.observer/pricing"),
    "observer-release": ("Input.Observer · Download 1.2.0", "https://input.observer/download"),
    "gesturesign": ("GestureSign · README & license", "https://github.com/TransposonY/GestureSign"),
    "gesturesign-release": ("GestureSign · v8.1", "https://github.com/TransposonY/GestureSign/releases/tag/v8.1"),
    "gesturesignv2": ("GestureSignv2 · README & license", "https://github.com/Tomclanc/GestureSignv2"),
    "gesturesignv2-release": ("GestureSignv2 · v18.4.1", "https://github.com/Tomclanc/GestureSignv2/releases/tag/v18.4.1"),
    "fastgestures": ("FastGestures · Official site", "https://fg.zhaokeli.com/"),
    "fastgestures-repo": ("FastGestures · Official repository", "https://github.com/zhaokeli/FastGestures"),
    "fastgestures-release": ("FastGestures · 2.2.75", "https://github.com/zhaokeli/FastGestures/releases/tag/2.2.75"),
    "mouseinc": ("MouseInc · Former product URL", "https://shuax.com/project/mouseinc/"),
    "mouseinc-settings": ("MouseInc.Settings · Archived settings repository", "https://github.com/shuax/MouseInc.Settings"),
    "aitiy": ("Aitiy · Official site & FAQ", "https://www.aitiy.com/"),
    "aitiy-docs": ("Aitiy · User guide", "https://www.aitiy.com/docs.html"),
    "gesturefy": ("Gesturefy · README & license", "https://github.com/Robbendebiene/Gesturefy"),
    "gesturefy-release": ("Gesturefy · v3.2.19", "https://github.com/Robbendebiene/Gesturefy/releases/tag/v3.2.19"),
    "crxmouse": ("CrxMouse · Official site", "https://crxmouse.com/"),
    "mousetroke": ("Mousetroke · README", "https://github.com/eiaao/mousetroke#readme"),
    "mousetroke-home": ("Mousetroke · Features", "https://mousetroke.pages.dev/en/"),
    "mousetroke-release": ("Mousetroke · Changelog", "https://github.com/eiaao/mousetroke/blob/main/CHANGELOG.md"),
}

ROWS = {
    "platform": bi("支持平台", "Platforms"),
    "license": bi("许可证与价格", "License & price"),
    "release": bi("维护与最近发布", "Maintenance / latest release"),
    "scope": bi("作用范围", "System-wide or browser-only"),
    "script": bi("脚本与动作", "Scripting & actions"),
    "edges": bi("屏幕边角", "Screen corners & edges"),
    "touch": bi("触控板与触屏", "Touchpad & touchscreen"),
    "install": bi("安装方式", "Installation"),
}


def fact(zh, en, *sources):
    return {"text": bi(zh, en), "sources": sources}


ENTITIES = {
    "mousetroke": {"name": "Mousetroke", "facts": {
        "platform": fact("Windows 10 1809+/11 x64；macOS 11+（Apple silicon、Intel，测试版）", "Windows 10 1809+/11 x64; macOS 11+ (Apple silicon & Intel, beta)", "mousetroke"),
        "license": fact("免费，MIT 开源", "Free, MIT open source", "mousetroke"),
        "release": fact("v0.2.1 · 2026-10-20", "v0.2.1 · 2026-10-20", "mousetroke-release"),
        "scope": fact("全局；可按程序设置或禁用", "System-wide; per-app rules and exclusions", "mousetroke-home"),
        "script": fact("Lua；发送按键、窗口操作、打开程序、打开网址、运行命令、媒体与系统、弹出菜单、Lua 脚本，共 8 类动作", "Lua; 8 action types: send keys, window actions, open an app, open a URL, run a command, media & system, show a menu, Lua script", "mousetroke-home"),
        "edges": fact("支持屏幕角落和边缘动作", "Screen corner and edge actions", "mousetroke-home"),
        "touch": fact("以鼠标按键拖动为主", "Mouse-button strokes", "mousetroke"),
        "install": fact("Windows 13.2 MB 便携 exe；Mac 8.1 MB DMG", "Windows: 13.2 MB portable exe; Mac: 8.1 MB DMG", "mousetroke-home"),
    }},
    "strokeit": {"name": "StrokeIt", "facts": {
        "platform": fact("Windows；官网提到 Win95–Win7", "Windows; site mentions Win95–Win7", "strokeit"),
        "license": fact("个人及符合条件的非营利慈善用途免费；商业、政府、教育用途需许可（截至 2026-10-21）", "Free for individual and eligible nonprofit charity use; business, government and education require a license (as of 2026-10-21)", "strokeit-download"),
        "release": fact("官网列出 0.9.7", "Official site lists 0.9.7", "strokeit", "strokeit-download"),
        "scope": fact("在 Windows 中画手势控制程序", "Mouse gestures to control Windows applications", "strokeit"),
        "script": fact("Professional 版提供 Lua 脚本", "Lua scripting in the Professional edition", "strokeit"),
        "install": fact("Windows exe 安装程序", "Windows exe installer", "strokeit", "strokeit-download"),
    }},
    "strokesplus": {"name": "StrokesPlus.net", "facts": {
        "platform": fact("Windows", "Windows", "observer-story"),
        "release": fact("原站保留论坛与最终下载归档；作者转向 Input.Observer", "Original site archives forums and final downloads; author moved to Input.Observer", "strokesplus", "observer-story"),
        "scope": fact("Windows 程序中的鼠标手势与热键", "Mouse gestures and hotkeys across Windows apps", "strokesplus", "observer-price"),
        "script": fact("JavaScript（V8）；迁往后继应用时不能直接运行原脚本", "JavaScript (V8); scripts do not run unchanged in its successor", "observer-story", "observer-price"),
    }},
    "observer": {"name": "Input.Observer", "facts": {
        "platform": fact("Windows 10 1809+/11 x64；不支持 Windows ARM", "Windows 10 1809+/11 x64; Windows ARM unsupported", "observer-price"),
        "license": fact("1.x 个人许可一次购买；完整试用 30 天，部分旧社区成员可免费领取（截至 2026-10-21）", "One-time personal license for 1.x; full 30-day trial, free keys for eligible former community members (as of 2026-10-21)", "observer-price"),
        "release": fact("1.2.0 · 2026-10-20", "1.2.0 · 2026-10-20", "observer-release"),
        "scope": fact("全局与按程序手势、热键、文本扩展", "Global and per-app gestures, hotkeys and text expansion", "observer"),
        "script": fact("C 风格脚本、无代码步骤、宏录制、补全与内联帮助；可交给 AutoHotkey v2 执行", "C-like scripts, no-code steps, macro recorder, completion and inline help; AutoHotkey v2 handoff", "observer"),
        "install": fact("x64 安装程序；也提供 Microsoft Store 入口", "x64 installer; Microsoft Store option", "observer-release"),
    }},
    "gesturesign": {"name": "GestureSign", "facts": {
        "platform": fact("Windows 平板", "Windows tablets", "gesturesign"),
        "license": fact("GPL-2.0 开源", "GPL-2.0 open source", "gesturesign"),
        "release": fact("v8.1 · 2022-02-03", "v8.1 · 2022-02-03", "gesturesign-release"),
        "scope": fact("Windows 窗口、键盘模拟及程序操作", "Windows window control, keyboard simulation and program actions", "gesturesign"),
        "script": fact("命令/程序启动和多种内置动作", "Run commands/programs and built-in actions", "gesturesign"),
        "touch": fact("手指触屏和鼠标", "Touchscreen gestures with fingers; mouse input", "gesturesign"),
    }},
    "gesturesignv2": {"name": "GestureSignv2", "facts": {
        "platform": fact("面向 Windows 11；x64、ARM64", "Windows 11 focused; x64 and ARM64", "gesturesignv2"),
        "license": fact("GPL-2.0 开源", "GPL-2.0 open source", "gesturesignv2"),
        "release": fact("v18.4.1 · 2026-10-20", "v18.4.1 · 2026-10-20", "gesturesignv2-release"),
        "scope": fact("按应用管理手势和命令", "Per-app gesture and command management", "gesturesignv2"),
        "touch": fact("触控板、触屏、鼠标；TipTap 与触控边缘交互", "Touchpad, touchscreen and mouse; TipTap and touch edge interactions", "gesturesignv2"),
        "install": fact("MSI、便携 ZIP；需匹配的 .NET 10 与 Windows App Runtime", "MSI or portable ZIP; matching .NET 10 and Windows App Runtime required", "gesturesignv2", "gesturesignv2-release"),
    }},
    "fastgestures": {"name": "FastGestures", "facts": {
        "platform": fact("Windows 7/8/10/11", "Windows 7/8/10/11", "fastgestures"),
        "release": fact("GitHub 2.2.75 · 2026-09-26（UTC）；主页仍标 2.2.73", "GitHub 2.2.75 · 2026-09-26 (UTC); home page still labels 2.2.73", "fastgestures", "fastgestures-release"),
        "scope": fact("系统全局鼠标、触控板与触屏手势", "System-wide mouse, touchpad and touchscreen gestures", "fastgestures-repo"),
        "edges": fact("边缘滚动：音量、亮度等", "Edge scrolling for volume, brightness and more", "fastgestures"),
        "touch": fact("触控板、触屏；触控板三指拖动", "Touchpad and touchscreen; three-finger touchpad dragging", "fastgestures", "fastgestures-repo"),
    }},
    "mouseinc": {"name": "MouseInc", "facts": {}},
    "aitiy": {"name": "Aitiy", "facts": {
        "platform": fact("Windows 10/11 x64；macOS 11+，Intel 与 Apple silicon", "Windows 10/11 x64; macOS 11+, Intel and Apple silicon", "aitiy", "aitiy-docs"),
        "license": fact("个人用途免费（截至 2026-10-21）", "Free for personal use (as of 2026-10-21)", "aitiy"),
        "release": fact("官网显示 v1.0.13", "Official site displays v1.0.13", "aitiy"),
        "scope": fact("全局与按应用鼠标手势、热键、截图和 OCR", "Global/per-app mouse gestures, hotkeys, screenshots and OCR", "aitiy", "aitiy-docs"),
        "script": fact("Lua 动作引擎；编辑器提供补全", "Lua action engine with editor completion", "aitiy", "aitiy-docs"),
        "edges": fact("屏幕边缘与触发角", "Screen edges and hot corners", "aitiy-docs"),
        "install": fact("Windows 解压运行 exe；Mac 解压 app 后移至 Applications；可配置便携模式", "Windows: extract and run exe; Mac: extract app and move to Applications; configurable portable mode", "aitiy-docs"),
    }},
    "gesturefy": {"name": "Gesturefy", "facts": {
        "platform": fact("Firefox 扩展", "Firefox extension", "gesturefy"),
        "license": fact("GPL-3.0 开源", "GPL-3.0 open source", "gesturefy"),
        "release": fact("v3.2.19 · 2026-09-25", "v3.2.19 · 2026-09-25", "gesturefy-release"),
        "scope": fact("浏览器网页；不能在 about:addons、Mozilla 受限页等处使用", "Browser web pages; excludes about:addons, restricted Mozilla pages and similar pages", "gesturefy"),
        "script": fact("提供 user script、多用途和跨扩展命令", "User script, multi-purpose and cross-addon commands", "gesturefy"),
        "touch": fact("官方说明面向鼠标，不适合触控板", "Official README says mice only, not suited to touchpads", "gesturefy"),
        "install": fact("Firefox 附加组件；无需独立桌面程序", "Firefox add-on; no separate desktop app", "gesturefy"),
    }},
    "crxmouse": {"name": "CrxMouse", "facts": {
        "platform": fact("桌面 Chrome 扩展", "Desktop Chrome extension", "crxmouse"),
        "scope": fact("浏览器导航、标签页与自定义手势", "Browser navigation, tabs and custom gestures", "crxmouse"),
        "install": fact("Chrome 浏览器扩展", "Chrome browser extension", "crxmouse"),
    }},
}

PAGES = {
    "strokeit": {
        "entities": ["strokeit"], "sources": ["strokeit", "strokeit-download"], "related": ["strokesplus", "mouseinc"],
        "title": bi("StrokeIt 替代方案：免费开源的 Mousetroke", "StrokeIt alternative: free, MIT-licensed Mousetroke"),
        "h1": bi("想换一款 StrokeIt 替代工具？从 Mousetroke 开始", "A free, open-source StrokeIt alternative for Windows 11"),
        "description": bi("寻找 StrokeIt 替代方案？Mousetroke 以 MIT 许可证免费开源，支持 Windows 11 和 Mac 测试版，用屏幕边角和按程序手势完成日常操作。比较两款工具的平台、许可与 Lua 动作，了解 Windows 便携 exe 和 Mac 首次打开步骤。从后退、关标签页等常用操作开始，逐项重建手势。", "Compare StrokeIt with free MIT-licensed Mousetroke for Windows 11 and Mac beta: screen corners, per-app gestures, portable setup and rebuilding your actions."),
        "intro": bi("保留按住鼠标画手势的习惯，把常用操作放到屏幕边角，并在 Windows 与 Mac 测试版之间使用同一套设置思路。", "Keep mouse strokes for everyday actions, add screen corners, and use the same approach on Windows and Mac beta."),
        "ours": bi("你需要明确支持 Windows 11 的 MIT 开源工具，以及屏幕边角和按程序设置。", "You want explicit Windows 11 support, MIT-licensed source, screen corners and per-app settings."),
        "theirs": bi("你已有稳定工作的 StrokeIt 配置，或依赖它的插件与 Professional 版 Lua 工作流。", "Your StrokeIt configuration already works well, or you depend on its plugins and Professional Lua workflow."),
        "paragraphs": bi([
            "StrokeIt 用鼠标手势控制 Windows 程序，Professional 版还提供 Lua 脚本。如果你已有顺手的配置，日常操作也都正常，可以继续沿用。它通过 exe 安装程序安装。",
            "Mousetroke 的 Windows 版下载后即可运行，也提供 Mac 测试版。迁移时，先把常用手势逐项设为按键、窗口操作或 Lua 动作，再添加按程序规则和屏幕边角。若依赖 StrokeIt 的插件或 Professional 工作流，把这些操作列为迁移时的重点测试项。"], [
            "StrokeIt uses mouse gestures to control Windows apps, with Lua scripting in the Professional edition. If your current setup handles your daily tasks well, keeping it saves you from rebuilding those actions. It installs through an exe installer.",
            "Mousetroke runs as a portable Windows exe and also offers a Mac beta. Start the move by setting up your usual shortcuts, window actions or Lua actions, then add per-app rules and screen corners. If you rely on StrokeIt plugins or a Professional workflow, test those tasks first." ]),
        "limitation": bi(("Windows 下载会出现 SmartScreen 提示吗？", "可能会。exe 尚未代码签名，首次运行 SmartScreen 可能警告；README 的操作是“更多信息 → 仍要运行”。请从官方发布页下载并核对 SHA-256。"), ("Can the Windows download trigger SmartScreen?", "Yes. The exe is not code-signed yet, so SmartScreen may warn on first run. The README says to choose More info → Run anyway. Download from the official release and verify SHA-256.")),
    },
    "strokesplus": {
        "entities": ["strokesplus", "observer"], "sources": ["strokesplus", "observer", "observer-story", "observer-price", "observer-release"], "related": ["strokeit", "fastgestures"],
        "title": bi("StrokesPlus.net 替代：Mousetroke 与 Input.Observer", "StrokesPlus.net alternatives: Mousetroke & Input.Observer"),
        "h1": bi("StrokesPlus.net 之后，选哪款替代工具？", "Choosing a StrokesPlus.net alternative for Windows 11"),
        "description": bi("比较 StrokesPlus.net 后续选择与免费 MIT 开源的 Mousetroke。面向 Windows 11，也提供 Mac 测试版，用屏幕边角和按程序手势完成日常操作。了解原作者后继应用的脚本编辑、宏录制与迁移限制，对照已核实的发布、许可和安装信息，按现有配置的复杂程度和日常操作来选择。", "Compare StrokesPlus.net options with free MIT-licensed Mousetroke for Windows 11 and Mac beta: per-app gestures, screen corners and clear migration limits."),
        "intro": bi("日常鼠标操作可以从轻量设置开始；重度脚本用户也值得了解原作者的后继应用。", "Start with focused gesture settings for daily use, and consider the original author's successor for script-heavy automation."),
        "ours": bi("你更看重开源、Windows 与 Mac 测试版，以及鼠标手势和边角操作的直接配置。", "You value open source, Windows plus Mac beta, and direct setup for mouse gestures and screen corners."),
        "theirs": bi("你需要宏录制、文本扩展和脚本补全，或想用 Input.Observer 导入旧配置，再手动调整未能转换的部分。", "You need macro recording, text expansion and script completion, or want to use Input.Observer's importer and adjust the parts it cannot translate."),
        "paragraphs": bi([
            "StrokesPlus.net 的旧站保留论坛和最终下载归档。原作者的后继应用 Input.Observer 提供 C 风格脚本、无代码步骤、宏录制，还能调用 AutoHotkey v2。经常编排多步操作的话，这些工具值得优先试用。",
            "StrokesPlus.net 的 JavaScript 不能直接在后继应用中运行。导入器会转换能处理的内容，其余部分需要手动调整。选择 Mousetroke 时，可以从最常用的几个手势开始，用内置动作或 Lua 重新配置；需要宏录制和文本扩展时，优先考虑 Input.Observer。"], [
            "The StrokesPlus.net site keeps its forums and final downloads in an archive. The original author's successor, Input.Observer, offers C-like scripts, no-code steps, macro recording and AutoHotkey v2 handoff. Those tools suit workflows with many steps.",
            "StrokesPlus.net JavaScript does not run unchanged in the successor. Its importer converts what it can; the rest needs manual adjustment. For a move to Mousetroke, rebuild your most-used gestures with built-in actions or Lua. Choose Input.Observer if macro recording and text expansion are central to your day." ]),
        "limitation": bi(("从旧脚本迁移到 Mousetroke，怎么开始？", "先挑几个最常用的操作，用内置动作或 Lua 重新配置，再逐个测试。若你每天都需要宏录制和带补全的脚本编辑器，可以先试用 Input.Observer。"), ("How do I move my scripted actions to Mousetroke?", "Start with your most-used actions. Set them up with built-in actions or Lua, then test each one. Try Input.Observer first if you use macro recording and a script editor with completion every day.")),
    },
    "gesturesign": {
        "entities": ["gesturesign", "gesturesignv2"], "sources": ["gesturesign", "gesturesign-release", "gesturesignv2", "gesturesignv2-release"], "related": ["fastgestures", "browser-extensions"],
        "title": bi("GestureSign 替代：鼠标还是触控优先？", "GestureSign alternative: mouse gestures or touch first?"),
        "h1": bi("寻找 GestureSign 替代方案，先看输入设备", "A GestureSign alternative for mouse-first workflows"),
        "description": bi("寻找 GestureSign 替代工具？免费 MIT 开源的 Mousetroke 支持 Windows 11 与 Mac 测试版，提供屏幕边角、按程序规则和 Lua 动作。比较原版与社区后继项目的触控方式、许可证和发布记录，按鼠标、触控板或触屏选择。了解便携下载、首次打开步骤和系统运行时要求，从常用操作开始配置。", "Compare GestureSign with free MIT-licensed Mousetroke for Windows 11 and Mac beta: screen corners, Lua actions, touch input and practical setup differences."),
        "intro": bi("鼠标按键拖动与多指触控是不同需求；按设备选择，比只看动作数量更实用。", "Mouse-button strokes and multi-finger input are different needs. Start with the device you actually use."),
        "ours": bi("你主要用鼠标，需要 MIT 开源、屏幕边角，并希望在 Windows 与 Mac 测试版使用相似操作。", "You mainly use a mouse and want MIT source, screen corners and similar workflows on Windows and Mac beta."),
        "theirs": bi("你优先使用触屏，或需要社区 GestureSignv2 明确提供的触控板、多指 TipTap 和触控边缘交互。", "Touchscreen input comes first, or you need the community GestureSignv2 project's touchpad, TipTap and touch edge interactions."),
        "paragraphs": bi([
            "GestureSign 面向 Windows 平板，可以用手指或鼠标画手势。社区重构版 GestureSignv2 面向 Windows 11，支持触控板、触屏、鼠标和 TipTap。你若经常离开鼠标操作，这类输入方式更值得先看。",
            "GestureSignv2 提供 MSI 和便携 ZIP，安装前需准备匹配的 .NET 10 与 Windows App Runtime。Mousetroke 的 Windows 版是便携 exe，也有 Mac 测试版，主要通过按住鼠标键画手势。换用时，从常用按键和窗口动作开始，再设置按程序规则与屏幕边角；多指触控优先的用户可先试用社区重构版。"], [
            "GestureSign is built for Windows tablets, with gestures drawn by finger or mouse. The community rebuild, GestureSignv2, targets Windows 11 and supports touchpads, touchscreens, mice and TipTap. Those inputs are useful if you often work away from a mouse.",
            "GestureSignv2 offers an MSI and a portable ZIP, with matching .NET 10 and Windows App Runtime installations required. Mousetroke provides a portable Windows exe and a Mac beta, centered on strokes drawn while holding a mouse button. Set up your usual shortcuts and window actions first, then add per-app rules and screen corners. For multi-finger workflows, try the community rebuild first." ]),
        "limitation": bi(("我主要用触控板或触屏，该先看哪款？", "先看 GestureSignv2 的触控板、触屏和 TipTap，或 FastGestures 的触控与三指拖动功能。主要用鼠标时，可以用 Mousetroke 配置按键手势和屏幕边角。"), ("Which tool should I try for a touchpad or touchscreen?", "Start with GestureSignv2 for touchpad, touchscreen and TipTap input, or FastGestures for touch input and three-finger dragging. For mouse use, Mousetroke offers mouse-button strokes and screen corners.")),
    },
    "fastgestures": {
        "entities": ["fastgestures"], "sources": ["fastgestures", "fastgestures-repo", "fastgestures-release"], "related": ["gesturesign", "strokesplus"],
        "title": bi("FastGestures 替代：开源鼠标手势与边角操作", "FastGestures alternative: open-source mouse gestures"),
        "h1": bi("需要开源的 FastGestures 替代工具？", "A free, open-source FastGestures alternative"),
        "description": bi("比较 FastGestures 与免费 MIT 开源的 Mousetroke：支持 Windows 11 和 Mac 测试版，用屏幕边角与按程序手势减少重复操作。看看触控板三指拖动、触屏手势和边缘滚动是否适合你的习惯，再了解鼠标方向手势、形状手势与 Lua 动作。结合平台要求、发布信息和便携下载步骤，选择更顺手的工具。", "Compare FastGestures with free MIT-licensed Mousetroke for Windows 11 and Mac beta: screen corners, per-app gestures and practical mouse-versus-touch choices."),
        "intro": bi("如果你主要用鼠标，先比较边角操作和按程序设置；触控板用户则应优先核对硬件交互。", "For mouse use, compare screen edges and per-app settings. For touchpads, check the input features your hardware needs first."),
        "ours": bi("你需要完整 MIT 源码、Windows 与 Mac 测试版，以及方向手势、形状手势和屏幕边角。", "You want MIT-licensed source, Windows and Mac beta, directional and shape gestures, and screen corners."),
        "theirs": bi("你在 Windows 上优先使用触控板或触屏，尤其需要官方列出的三指拖动与边缘滚动。", "You prioritize Windows touchpad or touchscreen input, especially the documented three-finger dragging and edge scrolling."),
        "paragraphs": bi([
            "FastGestures 支持 Windows 7/8/10/11，提供触控板三指拖动、触屏手势和边缘滚动。习惯用触控板拖窗口，或在屏幕边缘调音量、亮度的话，这些功能很贴近日常操作。下载时留意版本：主页标注 2.2.73，官方发布页已有 2.2.75。",
            "Mousetroke 适合主要用鼠标、想自己设置手势和查看 MIT 源码的用户，Windows 版下载后即可运行，也有 Mac 测试版。换用时，先为常用手势绑定按键或窗口动作，再添加屏幕边角和按程序规则。若你最依赖的是三指拖动，可优先试用 FastGestures。"], [
            "FastGestures supports Windows 7/8/10/11 with three-finger touchpad dragging, touchscreen gestures and edge scrolling. Those features suit dragging windows from a touchpad or adjusting volume and brightness at the screen edge. Check the version when downloading: the home page labels 2.2.73, while the official releases list 2.2.75.",
            "Mousetroke suits mouse users who want to configure strokes and inspect MIT-licensed source. Its Windows exe runs without installation, and a Mac beta is also available. Start by binding your usual strokes to shortcuts or window actions, then add screen corners and per-app rules. If three-finger dragging is your priority, try FastGestures first." ]),
        "limitation": bi(("需要触控板三指拖动，选哪款更合适？", "FastGestures 列有触控板三指拖动，建议先在自己的硬件上试用。Mousetroke 主要用鼠标按键画手势，也提供屏幕边角动作。"), ("Which tool should I try for three-finger touchpad dragging?", "FastGestures lists three-finger touchpad dragging, so try it on your hardware first. Mousetroke centers on mouse-button strokes and screen corner and edge actions.")),
    },
    "mouseinc": {
        "entities": ["mouseinc", "aitiy"], "sources": ["mouseinc", "mouseinc-settings", "aitiy", "aitiy-docs"], "related": ["strokeit", "strokesplus"],
        "title": bi("MouseInc 替代方案：Mousetroke 与 Aitiy", "MouseInc alternatives: Mousetroke and Aitiy"),
        "h1": bi("寻找 MouseInc 替代工具，从日常需求选起", "Looking for a MouseInc alternative for Windows 11?"),
        "description": bi("寻找 MouseInc 替代方案？免费 MIT 开源的 Mousetroke 支持 Windows 11 与 Mac 测试版，用鼠标手势、屏幕边角和按程序规则处理日常操作。也可了解 Aitiy 的截图、OCR 与 Lua 功能。比较两款工具的平台、安装方式和首次打开步骤，先配置几个最常用的动作，再按日常需要补齐设置。", "Explore MouseInc alternatives with free MIT-licensed Mousetroke for Windows 11 and Mac beta: screen corners, per-app gestures and transparent source code."),
        "intro": bi("先确定你需要专注的鼠标手势，还是把截图、OCR 与桌面工具也放进同一个应用。", "Decide whether you want focused mouse gestures or a broader app combining screenshots, OCR and desktop utilities."),
        "ours": bi("你需要可审阅的 MIT 源码、免安装 Windows exe、Mac 测试版，以及手势和边角的直接设置。", "You want inspectable MIT source, a portable Windows exe, Mac beta, and direct gesture and screen-edge settings."),
        "theirs": bi("原有 MouseInc 配置在你的电脑上稳定工作；或你愿意评估 Aitiy 的截图、OCR、Lua 动作和桌面工具。", "Your existing MouseInc setup works reliably, or you want to evaluate Aitiy's screenshots, OCR, Lua actions and desktop tools."),
        "paragraphs": bi([
            "MouseInc 用户若想把截图、OCR 和鼠标手势放在一个应用里，可以看看个人用途免费的 Aitiy。它支持 Windows 与 macOS，提供 Lua 动作和编辑补全。Windows 解压后运行 exe；Mac 解压后把 app 移到“应用程序”。",
            "若你换掉 MouseInc 主要是为了配置手势和边角，Mousetroke 提供 MIT 源码、Windows 便携 exe 和 Mac 测试版。先列出最常用的按键、窗口和程序启动操作，再逐项配置。选择时看日常任务：截图和 OCR 可优先试用 Aitiy，全局鼠标手势和边角则可以从 Mousetroke 开始。"], [
            "MouseInc users who want screenshots, OCR and mouse gestures in one app can look at Aitiy, which is free for personal use. It supports Windows and macOS, with Lua actions and editor completion. On Windows, extract and run the exe; on Mac, extract the app and move it to Applications.",
            "If you are leaving MouseInc mainly to set up gestures and screen edges, Mousetroke offers MIT source, a portable Windows exe and a Mac beta. List your usual shortcuts, window tasks and app launches, then configure them one at a time. Try Aitiy first for screenshots and OCR, or start with Mousetroke for system-wide mouse strokes and corners." ]),
        "limitation": bi(("Mac 测试版首次打开需要什么？", "DMG 中的应用需拖入“应用程序”。测试版自签名且未公证；被拦截时，按 README 到“系统设置 → 隐私与安全性 → 仍要打开”，再跟随引导授予辅助功能权限。仅在引导要求时开启输入监控。"), ("What does the Mac beta need on first open?", "Drag the app from the DMG to Applications. The beta is self-signed and not notarized. If blocked, follow the README: System Settings → Privacy & Security → Open Anyway, then grant Accessibility in setup. Enable Input Monitoring only if setup asks for it.")),
    },
    "browser-extensions": {
        "entities": ["gesturefy", "crxmouse"], "sources": ["gesturefy", "gesturefy-release", "crxmouse"], "related": ["gesturesign", "strokeit"],
        "title": bi("浏览器手势扩展替代：何时需要全局鼠标手势？", "Browser gesture extension alternatives: go system-wide?"),
        "h1": bi("Gesturefy、CrxMouse 的全局替代方案", "A system-wide alternative to browser gesture extensions"),
        "description": bi("比较 Gesturefy 与免费 MIT 开源的 Mousetroke：从浏览器手势走向 Windows 11 全局操作，也可试用 Mac 测试版。了解屏幕边角、按程序设置和扩展受限页面的差别；仅管理网页与标签页时也可保留扩展，结合官方来源、安装方式、适用范围和鼠标输入限制，决定是否需要桌面级操作。", "Compare Gesturefy with free MIT-licensed Mousetroke for Windows 11 and Mac beta: system-wide gestures and screen corners beyond browser-only navigation."),
        "intro": bi("只在网页内后退和关标签页，扩展就可能够用；想控制桌面应用和屏幕边角，再考虑全局工具。", "An extension may be enough for back and close-tab actions on web pages. Consider a system-wide tool for desktop apps and screen edges."),
        "ours": bi("你希望同一套鼠标习惯用于浏览器、文件管理器与其他应用，并使用屏幕边角动作。", "You want the same mouse habits across a browser, file manager and other apps, plus screen corner actions."),
        "theirs": bi("你只操作网页和标签页，希望直接在 Firefox 或 Chrome 中添加扩展，不额外安装桌面手势程序。", "You only need web-page and tab actions, and prefer a Firefox or Chrome add-on without a separate desktop gesture app."),
        "paragraphs": bi([
            "Gesturefy 直接装在 Firefox 中，提供鼠标手势、用户脚本和跨扩展命令。它不适合触控板，也不能在 about:addons 等受限页面使用。CrxMouse 则用于桌面 Chrome，提供浏览器导航、标签页控制和自定义手势；只处理网页操作时，这类扩展就很方便。",
            "Mousetroke 能把鼠标手势用到文件管理器和其他桌面应用，也能设置屏幕边角。换用时，先重建后退、关标签页等常用动作，再按程序调整或禁用冲突规则。你需要运行独立桌面程序，首次打开会涉及签名提示，Mac 上还需授予辅助功能权限。"], [
            "Gesturefy installs in Firefox and offers mouse gestures, user scripts and cross-addon commands. It is not suited to touchpads and cannot work on restricted pages such as about:addons. CrxMouse adds navigation, tab controls and custom gestures to desktop Chrome. These extensions are a convenient fit for web-page tasks.",
            "Mousetroke extends mouse strokes to a file manager and other desktop apps, and adds screen corners. Rebuild familiar actions such as Back and Close Tab, then adjust per-app rules or exclude apps where gestures conflict. You will run a separate desktop app, handle signing prompts on first open, and grant Accessibility permission on Mac." ]),
        "limitation": bi(("只需要浏览器手势，也值得装全局工具吗？", "不一定。只在网页内操作时，浏览器扩展可能更合适。需要文件管理器、其他桌面应用或屏幕边角时，再考虑 Mousetroke，并按程序禁用有冲突的手势。"), ("Do I need a system-wide app just for browser gestures?", "Not necessarily. A browser extension may be the better fit if you only act on web pages. Consider Mousetroke for a file manager, other desktop apps or screen corners, and exclude apps where gestures conflict.")),
    },
}

HUB = {
    "title": bi("鼠标手势工具对比：按场景选择替代方案", "Compare mouse gesture tools and alternatives"),
    "h1": bi("鼠标手势工具，按你的用法选", "Choose a mouse gesture tool for the way you work"),
    "description": bi("按场景比较鼠标手势工具与替代方案：仅浏览器、深度脚本、触控板或全局操作。Mousetroke 以 MIT 许可证免费开源，支持 Windows 11 与 Mac 测试版，提供屏幕边角和按程序手势。查看六组中英对比页、官方来源及核查日期，先确认输入设备、作用范围和安装要求，再选择适合自己日常使用的工具。", "Compare mouse gesture alternatives by browser, touch and scripting needs. Free MIT-licensed Mousetroke adds screen corners on Windows 11 and Mac beta."),
    "intro": bi("你主要用鼠标、触控板，还是触屏？手势只在浏览器里用，还是也要控制桌面应用？从这两个问题开始，再比较脚本、许可和安装方式。", "Start with your input device and where you need gestures: web pages, desktop apps or both. Then compare scripting, licensing and setup."),
    "table_note": bi("依据各官网信息，核对于 2026-10-21；“—”表示官网未说明。", 'Based on the official sites, checked 2026-10-21. "—" = not stated there.'),
}

SCENARIOS = [
    ("browser-extensions", bi("只在浏览器里用", "Only in the browser"), bi("先看 Gesturefy / CrxMouse：网页导航与标签页操作，无需独立桌面手势程序。", "Start with Gesturefy / CrxMouse for web navigation and tabs, without a separate desktop gesture app.")),
    ("strokesplus", bi("深度脚本与宏编辑", "Deep scripting and macros"), bi("先看 Input.Observer：原作者的后继应用提供脚本补全、步骤与宏录制。", "Look at Input.Observer: the original author's successor offers script completion, steps and macro recording.")),
    ("fastgestures", bi("触控板或触屏优先", "Touchpad or touchscreen first"), bi("先看 FastGestures 和 GestureSign / GestureSignv2，核对输入设备与系统要求。", "Evaluate FastGestures and GestureSign / GestureSignv2, checking device and OS requirements.")),
    (None, bi("轻量全局鼠标手势与边角", "Focused system-wide gestures and corners"), bi("Mousetroke：MIT 开源，Windows 与 Mac 测试版，按程序设置，支持屏幕边角。", "Mousetroke: MIT source, Windows and Mac beta, per-app settings and screen edges.")),
]
