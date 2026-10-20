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
        "touch": fact("以鼠标按键拖动为主；原生多指触控板/触屏手势未验证", "Mouse-button strokes; native multi-finger touchpad/touchscreen gestures unverified", "mousetroke"),
        "install": fact("Windows 13.2 MB 便携 exe；Mac 8.1 MB DMG", "Windows: 13.2 MB portable exe; Mac: 8.1 MB DMG", "mousetroke-home"),
    }},
    "strokeit": {"name": "StrokeIt", "facts": {
        "platform": fact("Windows；官网提到 Win95–Win7，Windows 11 兼容性未验证", "Windows; site mentions Win95–Win7; Windows 11 compatibility unverified", "strokeit"),
        "license": fact("个人及符合条件的非营利慈善用途免费；商业、政府、教育用途需许可（截至 2026-10-21）；完整源码许可证未验证", "Free for individual and eligible nonprofit charity use; business, government and education require a license (as of 2026-10-21); full-source license unverified", "strokeit-download"),
        "release": fact("官网列出 0.9.7；发布日期和后续维护未验证", "Official site lists 0.9.7; release date and ongoing maintenance unverified", "strokeit", "strokeit-download"),
        "scope": fact("在 Windows 中画手势控制程序", "Mouse gestures to control Windows applications", "strokeit"),
        "script": fact("Professional 版提供 Lua 脚本", "Lua scripting in the Professional edition", "strokeit"),
        "install": fact("Windows exe 安装程序", "Windows exe installer", "strokeit", "strokeit-download"),
    }},
    "strokesplus": {"name": "StrokesPlus.net", "facts": {
        "platform": fact("Windows；具体最低版本未验证", "Windows; minimum version unverified", "observer-story"),
        "release": fact("原站为论坛与最终下载归档；作者转向 Input.Observer；最终版本号未验证", "Original site archives forums and final downloads; author moved to Input.Observer; final version number unverified", "strokesplus", "observer-story"),
        "scope": fact("Windows 程序中的鼠标手势与热键", "Mouse gestures and hotkeys across Windows apps", "strokesplus", "observer-price"),
        "script": fact("JavaScript（V8）；迁往后继应用时不能直接运行原脚本", "JavaScript (V8); scripts do not run unchanged in its successor", "observer-story", "observer-price"),
    }},
    "observer": {"name": "Input.Observer", "facts": {
        "platform": fact("Windows 10 1809+/11 x64；不支持 Windows ARM", "Windows 10 1809+/11 x64; Windows ARM unsupported", "observer-price"),
        "license": fact("1.x 个人许可一次购买；完整试用 30 天，部分旧社区成员可免费领取（截至 2026-10-21）；源码许可证未验证", "One-time personal license for 1.x; full 30-day trial, free keys for eligible former community members (as of 2026-10-21); source license unverified", "observer-price"),
        "release": fact("1.2.0 · 2026-10-20", "1.2.0 · 2026-10-20", "observer-release"),
        "scope": fact("全局与按程序手势、热键、文本扩展", "Global and per-app gestures, hotkeys and text expansion", "observer"),
        "script": fact("C 风格脚本、无代码步骤、宏录制、补全与内联帮助；可交给 AutoHotkey v2 执行", "C-like scripts, no-code steps, macro recorder, completion and inline help; AutoHotkey v2 handoff", "observer"),
        "install": fact("x64 安装程序；也提供 Microsoft Store 入口", "x64 installer; Microsoft Store option", "observer-release"),
    }},
    "gesturesign": {"name": "GestureSign", "facts": {
        "platform": fact("Windows 平板；具体系统版本未验证", "Windows tablets; specific OS versions unverified", "gesturesign"),
        "license": fact("GPL-2.0 开源；价格未验证", "GPL-2.0 open source; price unverified", "gesturesign"),
        "release": fact("v8.1 · 2022-02-03；不据此推断已停止维护", "v8.1 · 2022-02-03; no inference that maintenance has stopped", "gesturesign-release"),
        "scope": fact("Windows 窗口、键盘模拟及程序操作", "Windows window control, keyboard simulation and program actions", "gesturesign"),
        "script": fact("命令/程序启动和多种内置动作；脚本语言未验证", "Run commands/programs and built-in actions; scripting language unverified", "gesturesign"),
        "touch": fact("手指触屏和鼠标；触控板支持未验证", "Finger/touchscreen and mouse; touchpad support unverified", "gesturesign"),
    }},
    "gesturesignv2": {"name": "GestureSignv2", "facts": {
        "platform": fact("面向 Windows 11；x64、ARM64", "Windows 11 focused; x64 and ARM64", "gesturesignv2"),
        "license": fact("GPL-2.0 开源；价格未验证", "GPL-2.0 open source; price unverified", "gesturesignv2"),
        "release": fact("v18.4.1 · 2026-10-20", "v18.4.1 · 2026-10-20", "gesturesignv2-release"),
        "scope": fact("按应用管理手势和命令", "Per-app gesture and command management", "gesturesignv2"),
        "touch": fact("触控板、触屏、鼠标；TipTap 与触控边缘交互", "Touchpad, touchscreen and mouse; TipTap and touch edge interactions", "gesturesignv2"),
        "install": fact("MSI、便携 ZIP；需匹配的 .NET 10 与 Windows App Runtime", "MSI or portable ZIP; matching .NET 10 and Windows App Runtime required", "gesturesignv2", "gesturesignv2-release"),
    }},
    "fastgestures": {"name": "FastGestures", "facts": {
        "platform": fact("Windows 7/8/10/11", "Windows 7/8/10/11", "fastgestures"),
        "release": fact("GitHub 2.2.75 · 2026-09-26（UTC）；主页仍标 2.2.73", "GitHub 2.2.75 · 2026-09-26 (UTC); home page still labels 2.2.73", "fastgestures", "fastgestures-release"),
        "scope": fact("系统全局鼠标、触控板与触屏手势", "System-wide mouse, touchpad and touchscreen gestures", "fastgestures-repo"),
        "edges": fact("边缘滚动：音量、亮度等；角落支持未验证", "Edge scrolling for volume, brightness and more; corners unverified", "fastgestures"),
        "touch": fact("触控板、触屏；触控板三指拖动", "Touchpad and touchscreen; three-finger touchpad dragging", "fastgestures", "fastgestures-repo"),
    }},
    "mouseinc": {"name": "MouseInc", "facts": {}},
    "aitiy": {"name": "Aitiy", "facts": {
        "platform": fact("Windows 10/11 x64；macOS 11+，Intel 与 Apple silicon", "Windows 10/11 x64; macOS 11+, Intel and Apple silicon", "aitiy", "aitiy-docs"),
        "license": fact("个人用途免费（截至 2026-10-21）；源码及商业使用许可未验证", "Free for personal use (as of 2026-10-21); source and commercial-use license unverified", "aitiy"),
        "release": fact("官网显示 v1.0.13；发布日期未验证", "Official site displays v1.0.13; release date unverified", "aitiy"),
        "scope": fact("全局与按应用鼠标手势、热键、截图和 OCR", "Global/per-app mouse gestures, hotkeys, screenshots and OCR", "aitiy", "aitiy-docs"),
        "script": fact("Lua 动作引擎；编辑器提供补全", "Lua action engine with editor completion", "aitiy", "aitiy-docs"),
        "edges": fact("屏幕边缘与触发角", "Screen edges and hot corners", "aitiy-docs"),
        "install": fact("Windows 解压运行 exe；Mac 解压 app 后移至 Applications；可配置便携模式", "Windows: extract and run exe; Mac: extract app and move to Applications; configurable portable mode", "aitiy-docs"),
    }},
    "gesturefy": {"name": "Gesturefy", "facts": {
        "platform": fact("Firefox 扩展", "Firefox extension", "gesturefy"),
        "license": fact("GPL-3.0 开源；价格未验证", "GPL-3.0 open source; price unverified", "gesturefy"),
        "release": fact("v3.2.19 · 2026-09-25", "v3.2.19 · 2026-09-25", "gesturefy-release"),
        "scope": fact("浏览器网页；不能在 about:addons、Mozilla 受限页等处使用", "Browser web pages; excludes about:addons, restricted Mozilla pages and similar pages", "gesturefy"),
        "script": fact("提供 user script、多用途和跨扩展命令", "User script, multi-purpose and cross-addon commands", "gesturefy"),
        "touch": fact("官方说明面向鼠标，不适合触控板", "Official README says mice only, not suited to touchpads", "gesturefy"),
        "install": fact("Firefox 附加组件；无需独立桌面程序", "Firefox add-on; no separate desktop app", "gesturefy"),
    }},
    "crxmouse": {"name": "CrxMouse", "facts": {
        "platform": fact("桌面 Chrome 扩展；其他 Chromium 浏览器未验证", "Desktop Chrome extension; other Chromium browsers unverified", "crxmouse"),
        "scope": fact("浏览器导航、标签页与自定义手势", "Browser navigation, tabs and custom gestures", "crxmouse"),
        "install": fact("Chrome 浏览器扩展", "Chrome browser extension", "crxmouse"),
    }},
}

PAGES = {
    "strokeit": {
        "entities": ["strokeit"], "sources": ["strokeit", "strokeit-download"], "related": ["strokesplus", "mouseinc"],
        "title": bi("StrokeIt 替代方案：免费开源的 Mousetroke", "StrokeIt alternative: free, MIT-licensed Mousetroke"),
        "h1": bi("想换一款 StrokeIt 替代工具？从 Mousetroke 开始", "A free, open-source StrokeIt alternative for Windows 11"),
        "description": bi("寻找 StrokeIt 替代方案？Mousetroke 以 MIT 许可证免费开源，支持 Windows 11 和 Mac 测试版，用屏幕边角和按程序手势完成日常操作。对照官网核实的平台、许可与 Lua 支持，了解 Windows 便携 exe、Mac 首次打开说明和触控限制，再决定保留原有配置还是重新设置。", "Compare StrokeIt with free MIT-licensed Mousetroke for Windows 11 and Mac beta: screen corners, per-app gestures, portable setup and honest limitations."),
        "intro": bi("保留按住鼠标画手势的习惯，把常用操作放到屏幕边角，并在 Windows 与 Mac 测试版之间使用同一套设置思路。", "Keep mouse strokes for everyday actions, add screen corners, and use the same approach on Windows and Mac beta."),
        "ours": bi("你需要明确支持 Windows 11 的 MIT 开源工具，以及屏幕边角和按程序设置。", "You want explicit Windows 11 support, MIT-licensed source, screen corners and per-app settings."),
        "theirs": bi("你已有稳定工作的 StrokeIt 配置，或依赖它的插件与 Professional 版 Lua 工作流。", "Your StrokeIt configuration already works well, or you depend on its plugins and Professional Lua workflow."),
        "paragraphs": bi([
            "StrokeIt 官网仍列出 0.9.7，并提到修复 Win95 至 Win7 的问题。这不足以证明它不能运行于 Windows 11，也不能用来承诺新系统兼容性。先在自己的应用中验证原配置，再决定是否迁移。",
            "StrokeIt 的 Professional 版提供 Lua 脚本；许多插件以 MIT 开源不代表整个产品采用 MIT。Mousetroke 则以 MIT 发布完整项目，支持方向与形状手势、按程序规则及全屏自动暂停。两者脚本和配置不能据此视为直接兼容。"], [
            "StrokeIt still lists 0.9.7 and fixes for Win95 through Win7. That does not establish that it fails on Windows 11, or promise compatibility with current systems. Check your existing setup in your own apps before migrating.",
            "StrokeIt Professional includes Lua. MIT-licensed plugins do not establish an MIT license for the whole product. Mousetroke publishes its project under MIT and offers directional and shape gestures, per-app rules and automatic fullscreen pause. This does not imply configuration or script compatibility." ]),
        "limitation": bi(("Windows 下载会出现 SmartScreen 提示吗？", "可能会。exe 尚未代码签名，首次运行 SmartScreen 可能警告；README 的操作是“更多信息 → 仍要运行”。请从官方发布页下载并核对 SHA-256。"), ("Can the Windows download trigger SmartScreen?", "Yes. The exe is not code-signed yet, so SmartScreen may warn on first run. The README says to choose More info → Run anyway. Download from the official release and verify SHA-256.")),
    },
    "strokesplus": {
        "entities": ["strokesplus", "observer"], "sources": ["strokesplus", "observer", "observer-story", "observer-price", "observer-release"], "related": ["strokeit", "fastgestures"],
        "title": bi("StrokesPlus.net 替代：Mousetroke 与 Input.Observer", "StrokesPlus.net alternatives: Mousetroke & Input.Observer"),
        "h1": bi("StrokesPlus.net 之后，选哪款替代工具？", "Choosing a StrokesPlus.net alternative for Windows 11"),
        "description": bi("比较 StrokesPlus.net 后续选择与免费 MIT 开源的 Mousetroke。面向 Windows 11，也提供 Mac 测试版，用屏幕边角和按程序手势完成日常操作。了解原作者后继应用的脚本编辑、宏录制与迁移限制，对照已核实的发布、许可和安装信息，按现有配置的复杂程度和日常操作来选择。", "Compare StrokesPlus.net options with free MIT-licensed Mousetroke for Windows 11 and Mac beta: per-app gestures, screen corners and clear migration limits."),
        "intro": bi("日常鼠标操作可以从轻量设置开始；重度脚本用户也值得了解原作者的后继应用。", "Start with focused gesture settings for daily use, and consider the original author's successor for script-heavy automation."),
        "ours": bi("你更看重开源、Windows 与 Mac 测试版，以及鼠标手势和边角操作的直接配置。", "You value open source, Windows plus Mac beta, and direct setup for mouse gestures and screen corners."),
        "theirs": bi("你依赖宏录制、文本扩展、脚本补全，或希望由 Input.Observer 导入旧配置，再手动处理不能转换的部分。", "You need macro recording, text expansion and script completion, or want Input.Observer's importer and can review untranslated parts."),
        "paragraphs": bi([
            "StrokesPlus.net 的原作者把 Input.Observer 定位为重新构建的后继应用；旧站保留论坛与最终下载归档。后者提供 C 风格脚本、无代码步骤、宏录制和 AutoHotkey v2 交接，适合把自动化当作完整工作环境的人。",
            "StrokesPlus.net 原来的 JavaScript 不能原样在后继应用运行。官方导入器会尽可能转换，无法转换的内容需要人工处理。Mousetroke 提供 Lua 与结构化动作，但这里没有承诺兼容旧脚本或自动迁移配置。"], [
            "The StrokesPlus.net author describes Input.Observer as a ground-up successor, while the old site keeps a forum and final-download archive. The successor offers C-like scripts, no-code steps, macro recording and AutoHotkey v2 handoff for a broader automation workflow.",
            "StrokesPlus.net JavaScript does not run unchanged in the successor. Its importer translates what it can, and unsupported parts need manual work. Mousetroke offers Lua and structured actions, without a promise to import old configurations or run their scripts." ]),
        "limitation": bi(("Mousetroke 能直接运行旧的自动化脚本吗？", "不能假设可以。Mousetroke 使用 Lua 与自己的动作结构；请重新配置并逐个测试。需要宏录制和带补全的脚本编辑环境时，可先评估 Input.Observer。"), ("Can Mousetroke run my existing automation scripts unchanged?", "Do not assume compatibility. Mousetroke uses Lua and its own action structure; rebuild and test each action. Evaluate Input.Observer if macro recording and a script editor with completion are central to your workflow.")),
    },
    "gesturesign": {
        "entities": ["gesturesign", "gesturesignv2"], "sources": ["gesturesign", "gesturesign-release", "gesturesignv2", "gesturesignv2-release"], "related": ["fastgestures", "browser-extensions"],
        "title": bi("GestureSign 替代：鼠标还是触控优先？", "GestureSign alternative: mouse gestures or touch first?"),
        "h1": bi("寻找 GestureSign 替代方案，先看输入设备", "A GestureSign alternative for mouse-first workflows"),
        "description": bi("寻找 GestureSign 替代工具？免费 MIT 开源的 Mousetroke 支持 Windows 11 与 Mac 测试版，提供屏幕边角、按程序规则和 Lua 动作。对照原版与社区后继项目的官方触控支持、许可证及发布记录，区分鼠标按键拖动和多指触控需求，并了解便携下载、首次打开和系统运行时差异。", "Compare GestureSign with free MIT-licensed Mousetroke for Windows 11 and Mac beta: screen corners, Lua actions, and honest mouse-versus-touch tradeoffs."),
        "intro": bi("鼠标按键拖动与多指触控是不同需求；按设备选择，比只看动作数量更实用。", "Mouse-button strokes and multi-finger input are different needs. Start with the device you actually use."),
        "ours": bi("你主要用鼠标，需要 MIT 开源、屏幕边角，并希望在 Windows 与 Mac 测试版使用相似操作。", "You mainly use a mouse and want MIT source, screen corners and similar workflows on Windows and Mac beta."),
        "theirs": bi("你优先使用触屏，或需要社区 GestureSignv2 明确提供的触控板、多指 TipTap 和触控边缘交互。", "Touchscreen input comes first, or you need the community GestureSignv2 project's touchpad, TipTap and touch edge interactions."),
        "paragraphs": bi([
            "GestureSign 的官方说明以 Windows 平板为对象，支持用手指或鼠标画手势。其发布页最新列出 v8.1（2022-02-03）；单凭这个日期不能推断所有维护活动已经结束。",
            "GestureSignv2 是基于原项目、面向 Windows 11 的社区重构，明确列出触控板、触屏、鼠标与 TipTap。两个项目都列出 GPL-2.0。Mousetroke 的文档以鼠标按键手势为主，原生多指触控能力未验证，不能用网页演示的触屏交互代替产品支持说明。"], [
            "GestureSign describes itself as software for Windows tablets, with gestures drawn using fingers or a mouse. Its latest listed release is v8.1, dated 2022-02-03; that date alone does not establish that all maintenance has ended.",
            "GestureSignv2 is a community rebuild based on the original project and focused on Windows 11. It explicitly lists touchpad, touchscreen, mouse and TipTap support. Both projects list GPL-2.0. Mousetroke's documentation centers on mouse-button strokes; native multi-finger input is unverified, and the website's touch demo is not evidence of product support." ]),
        "limitation": bi(("Mousetroke 能替代原生多指触控手势吗？", "目前不能这样承诺。已核实的主要输入方式是按住鼠标键画手势。触控板或触屏优先时，请先评估明确列出这些输入方式的工具，并在自己的硬件上测试。"), ("Can Mousetroke replace native multi-finger touch gestures?", "That is not established. The verified primary input is drawing while holding a mouse button. For touchpad or touchscreen use, first evaluate a tool that explicitly documents those inputs and test it on your hardware.")),
    },
    "fastgestures": {
        "entities": ["fastgestures"], "sources": ["fastgestures", "fastgestures-repo", "fastgestures-release"], "related": ["gesturesign", "strokesplus"],
        "title": bi("FastGestures 替代：开源鼠标手势与边角操作", "FastGestures alternative: open-source mouse gestures"),
        "h1": bi("需要开源的 FastGestures 替代工具？", "A free, open-source FastGestures alternative"),
        "description": bi("比较 FastGestures 与免费 MIT 开源的 Mousetroke：支持 Windows 11 和 Mac 测试版，用屏幕边角与按程序手势减少重复操作。根据官网核实的 Windows 与触控支持选择输入方式，查看最新发布差异、安装说明和限制；未核实的价格、脚本语言及许可不作推断，便于逐项判断。", "Compare FastGestures with free MIT-licensed Mousetroke for Windows 11 and Mac beta: screen corners, per-app gestures and practical mouse-versus-touch choices."),
        "intro": bi("如果你主要用鼠标，先比较边角操作和按程序设置；触控板用户则应优先核对硬件交互。", "For mouse use, compare screen edges and per-app settings. For touchpads, check the input features your hardware needs first."),
        "ours": bi("你需要完整 MIT 源码、Windows 与 Mac 测试版，以及方向手势、形状手势和屏幕边角。", "You want MIT-licensed source, Windows and Mac beta, directional and shape gestures, and screen corners."),
        "theirs": bi("你在 Windows 上优先使用触控板或触屏，尤其需要官方列出的三指拖动与边缘滚动。", "You prioritize Windows touchpad or touchscreen input, especially the documented three-finger dragging and edge scrolling."),
        "paragraphs": bi([
            "FastGestures 官网明确列出 Windows 7/8/10/11、触控板与触屏，以及边缘滚动和三指拖动。这些是评估触控场景的具体依据，不能因为另一款产品也叫手势工具就认为输入能力相同。",
            "FastGestures 主页标注 2.2.73，而官方 GitHub 发布页更新到 2.2.75；表格分别说明了这两个来源。价格、源码许可、Lua 和安装方式未从本次可读官方资料确认，因此保留未验证。Mousetroke 则适合希望直接查看源码、配置鼠标手势与边角的用户。"], [
            "FastGestures explicitly lists Windows 7/8/10/11, touchpad and touchscreen input, edge scrolling and three-finger dragging. Those are useful specifics for touch workflows; two products using the word gestures do not necessarily accept the same input.",
            "The FastGestures home page labels 2.2.73, while the official GitHub release page lists 2.2.75. The table preserves that distinction. Price, source license, Lua and installation details remain unverified in the official material read for this comparison. Mousetroke suits users who want to inspect the source and configure mouse strokes and screen edges." ]),
        "limitation": bi(("Mousetroke 有触控板三指拖动吗？", "本次依据的产品资料没有确认这项能力。需要三指拖动时，应优先测试明确提供它的工具；Mousetroke 的已核实定位是鼠标按键手势和屏幕边角。"), ("Does Mousetroke offer three-finger touchpad dragging?", "The product sources used here do not establish that feature. If three-finger dragging is essential, test a tool that documents it. Mousetroke's verified focus is mouse-button gestures and screen corners and edges.")),
    },
    "mouseinc": {
        "entities": ["mouseinc", "aitiy"], "sources": ["mouseinc", "mouseinc-settings", "aitiy", "aitiy-docs"], "related": ["strokeit", "strokesplus"],
        "title": bi("MouseInc 替代方案：Mousetroke 与 Aitiy", "MouseInc alternatives: Mousetroke and Aitiy"),
        "h1": bi("寻找 MouseInc 替代工具，从日常需求选起", "Looking for a MouseInc alternative for Windows 11?"),
        "description": bi("寻找 MouseInc 替代方案？免费 MIT 开源的 Mousetroke 支持 Windows 11 与 Mac 测试版，用鼠标手势、屏幕边角和按程序规则处理日常操作。也可了解另一款桌面工具的截图、OCR 与 Lua 能力；对照官方来源、首次运行说明和未验证项，按所需功能、输入设备和安装要求选择。", "Explore MouseInc alternatives with free MIT-licensed Mousetroke for Windows 11 and Mac beta: screen corners, per-app gestures and transparent source code."),
        "intro": bi("先确定你需要专注的鼠标手势，还是把截图、OCR 与桌面工具也放进同一个应用。", "Decide whether you want focused mouse gestures or a broader app combining screenshots, OCR and desktop utilities."),
        "ours": bi("你需要可审阅的 MIT 源码、免安装 Windows exe、Mac 测试版，以及手势和边角的直接设置。", "You want inspectable MIT source, a portable Windows exe, Mac beta, and direct gesture and screen-edge settings."),
        "theirs": bi("原有 MouseInc 配置在你的电脑上稳定工作；或你愿意评估 Aitiy 的截图、OCR、Lua 动作和桌面工具。", "Your existing MouseInc setup works reliably, or you want to evaluate Aitiy's screenshots, OCR, Lua actions and desktop tools."),
        "paragraphs": bi([
            "MouseInc 旧产品地址本次指向其设置界面仓库，该仓库标记为归档。设置界面的归档不足以确认核心程序的最终版本、许可证或停止维护的时间，因此表格没有把这些信息写成事实。",
            "MouseInc 用户也可了解 Aitiy：官网确认 Windows 与 macOS、个人用途免费、Lua、截图和 OCR。关于两者是否存在正式后继关系，本次官方资料未验证；也不承诺配置兼容。Mousetroke 以 MIT 开源，适合主要想配置鼠标手势、边角和按程序规则的人。"], [
            "The former MouseInc product URL pointed to its settings-interface repository when checked, and that repository is archived. Archiving a settings interface does not establish the core product's final version, license or end-of-maintenance date, so those remain unverified.",
            "MouseInc users can also evaluate Aitiy. Its site confirms Windows and macOS, free personal use, Lua, screenshots and OCR. A formal successor relationship was not verified in the official material read here, and configuration compatibility is not promised. Mousetroke provides MIT-licensed source for users focused on gestures, screen edges and per-app rules." ]),
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
            "Gesturefy 是 Firefox 鼠标手势扩展，支持用户脚本等命令，但明确说明不适合触控板，也不能在 about:addons 等受限页面使用。选择浏览器内方案时，这些限制比动作列表长短更值得先确认。",
            "CrxMouse 官网确认桌面 Chrome、标签页控制和自定义手势；本页没有把它推广为所有 Chromium 浏览器都兼容。Mousetroke 在系统层处理鼠标手势，可按程序配置，但首次运行涉及桌面应用的签名提示与权限设置。"], [
            "Gesturefy is a Firefox mouse gesture add-on with commands including user scripts. It explicitly says it is not suited to touchpads and cannot operate on restricted pages such as about:addons. Check those boundaries before choosing a browser-only workflow.",
            "CrxMouse confirms desktop Chrome, tab controls and custom gestures on its official site. This page does not assume compatibility with every Chromium browser. Mousetroke handles mouse strokes at the system level with per-app rules; first use also involves desktop-app signing prompts and permission setup." ]),
        "limitation": bi(("只需要浏览器手势，也值得装全局工具吗？", "不一定。只在网页内操作时，浏览器扩展可能更合适。需要文件管理器、其他桌面应用或屏幕边角时，再考虑 Mousetroke，并按程序禁用有冲突的手势。"), ("Do I need a system-wide app just for browser gestures?", "Not necessarily. A browser extension may be the better fit if you only act on web pages. Consider Mousetroke for a file manager, other desktop apps or screen corners, and exclude apps where gestures conflict.")),
    },
}

HUB = {
    "title": bi("鼠标手势工具对比：按场景选择替代方案", "Compare mouse gesture tools and alternatives"),
    "h1": bi("鼠标手势工具，按你的用法选", "Choose a mouse gesture tool for the way you work"),
    "description": bi("按场景比较鼠标手势工具与替代方案：仅浏览器、深度脚本、触控板或全局操作。Mousetroke 以 MIT 许可证免费开源，支持 Windows 11 与 Mac 测试版，提供屏幕边角和按程序手势。查看六组中英对比页、官方来源及核查日期，先确认输入设备、作用范围和安装要求，再选择适合自己日常使用的工具。", "Compare mouse gesture alternatives by browser, touch and scripting needs. Free MIT-licensed Mousetroke adds screen corners on Windows 11 and Mac beta."),
    "intro": bi("先看输入设备和作用范围，再比较脚本、许可和安装方式。下面的选择路线与表格只采用已核实的官方资料。", "Start with your input device and where gestures need to work. Then compare scripts, licensing and setup using verified official sources."),
}

SCENARIOS = [
    ("browser-extensions", bi("只在浏览器里用", "Only in the browser"), bi("先看 Gesturefy / CrxMouse：网页导航与标签页操作，无需独立桌面手势程序。", "Start with Gesturefy / CrxMouse for web navigation and tabs, without a separate desktop gesture app.")),
    ("strokesplus", bi("深度脚本与宏编辑", "Deep scripting and macros"), bi("先看 Input.Observer：原作者的后继应用提供脚本补全、步骤与宏录制。", "Look at Input.Observer: the original author's successor offers script completion, steps and macro recording.")),
    ("fastgestures", bi("触控板或触屏优先", "Touchpad or touchscreen first"), bi("先看 FastGestures 和 GestureSign / GestureSignv2，核对输入设备与系统要求。", "Evaluate FastGestures and GestureSign / GestureSignv2, checking device and OS requirements.")),
    (None, bi("轻量全局鼠标手势与边角", "Focused system-wide gestures and corners"), bi("Mousetroke：MIT 开源，Windows 与 Mac 测试版，按程序设置，支持屏幕边角。", "Mousetroke: MIT source, Windows and Mac beta, per-app settings and screen edges.")),
]
