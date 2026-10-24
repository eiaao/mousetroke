---
title: "Lua API"
description: "Lua 5.4 的完整 ms 函数参考，包含签名、参数类型、返回值、示例以及 Windows 和 Mac 平台差异。"
order: 9
---

# Lua API

本页列出共享运行时注册到 `ms` 的全部函数。示例是放入 `ms_action` 的语句，函数中可使用 `pos`、`wid`、`pid`、`exe`、`title`、`mode`。完整脚本见 [Lua 入门](lua-basics.md)，可直接用作动作的脚本见[实用示例](lua-examples.md)。

## 约定

Lua 版本为 **5.4**（`mlua` 的 `lua54`、`vendored` 特性）。`nil` 表示无值，`?` 表示可选参数。无返回值的函数写作 `-> nil`，可选参数省略等同于 nil。布尔参数请传真正的布尔值。整数参数会将有限小数向零截断，超出底层整数范围会报错；脚本中建议使用整数。

位置表应使用命名字段 `{x=..., y=...}`，不是数组。Mac 不支持的调用通常每个函数、每个进程只记一次警告，查询返回 nil，设置函数无操作。无返回值不表示操作成功。Lua API 字符串不经过动作变量替换。

## 流程与反馈

### `ms.log`

`ms.log(...) -> nil`

接受任意 Lua 值。首参为格式串且还有其他参数时支持 `%s`、`%d`、`%f`、`%q`、`%%`；否则以空格连接各值。`%q` 不会额外加 Lua 引号。两平台均写入带 `Lua:` 前缀的运行日志，无返回值。

```lua
ms.log("pid=%d app=%s", pid, exe)
```

### `ms.sleep`

`ms.sleep(milliseconds: integer) -> nil`

等待非负毫秒数（无符号 64 位）。两平台均阻塞当前脚本工作线程，手势引擎／主线程继续运行。无返回值，运行时没有脚本超时。

```lua
ms.sleep(100)
```

### `ms.is_paused`

`ms.is_paused() -> boolean`

无参数。两平台均返回引擎手动暂停标志，不代表全屏自动抑制或程序排除状态。

```lua
ms.log(ms.is_paused())
```

### `ms.set_paused`

`ms.set_paused(paused: boolean) -> nil`

`true` 暂停，`false` 恢复，无返回值。两平台支持。Windows 向托盘线程投递命令，紧接着查询 `is_paused()` 可能仍是旧值；Mac 通过主线程桥接应用。

```lua
ms.set_paused(true)
```

### `ms.show_settings`

`ms.show_settings() -> nil`

无参数、无返回值。两平台均打开设置或将已有设置窗口带到前面。

```lua
ms.show_settings()
```

### `ms.show_toast`

`ms.show_toast(message: string, duration_ms?: integer) -> nil`

`message` 为提示文字；可选非负时长以毫秒计，默认 **1200**。两平台均显示屏幕提示，无返回值。

```lua
ms.show_toast("完成", 1500)
```

## 键盘输入

### `ms.keys`

`ms.keys(sequence: string, text_by_default?: boolean) -> nil`

解析并发送[按键序列](key-sequences.md)，文本优先模式默认为 `false`。发送后返回，无返回值，语法无效会抛出 Lua 错误。两平台支持普通键盘／文字输入。Mac 跳过不支持的键、鼠标和滚轮动作，并受队列／文字长度限制；它会等待主线程输入队列，该队列也可能包含其他序列。

```lua
ms.keys("你好", true)
```

### `ms.keys_down`

`ms.keys_down(sequence: string) -> nil`

仅 Windows：按按键模式解析，只按下按键／修饰键，不松开。该阶段不重复次数，滚轮动作发送一次。语法无效会抛出 Lua 错误。请配合 `keys_up` 释放。Mac 记录不支持并无操作，无返回值。

```lua
ms.keys_down("^a")
ms.sleep(50)
ms.keys_up("^a")
```

### `ms.keys_up`

`ms.keys_up(sequence: string) -> nil`

仅 Windows：按动作逆序释放解析出的序列，修饰键最后释放；延时步骤仍会等待。语法错误会抛出 Lua 错误。Mac 记录不支持并无操作，无返回值。

```lua
ms.keys_down("{SHIFT}")
ms.keys_up("{SHIFT}")
```

### `ms.is_keys_pressed`

`ms.is_keys_pressed(sequence: string) -> boolean | nil`

两平台均检查每个键盘键**及其前缀修饰键**，`"^a"` 要求 Control 与 A 同时按下。前缀接受任意一侧，`{RCTRL}` 等显式名称要求指定侧。Mac 上 `^` = Control、`%` = Option、`+` = Shift、`#` = Command。多步序列要求所有列出的键与修饰键同时按下，不检查按下顺序；重复次数不增加要求。鼠标／滚轮、延时或文本动作使结果为假；空序列为真。无效语法返回 false。Mac 读取合并会话状态，缺少辅助功能权限时返回 false，并仅记录一次日志。

```lua
if ms.is_keys_pressed("^a") then
  ms.show_toast("Ctrl 和 A 均已按下")
end
```

## 剪贴板与文件

### `ms.get_clipboard_text`

`ms.get_clipboard_text() -> string | nil`

无参数。返回剪贴板文字，无可用文字时为 `nil`。两平台支持，不会自动读取选区。

```lua
local text = ms.get_clipboard_text()
ms.log(text or "没有文字")
```

### `ms.set_clipboard_text`

`ms.set_clipboard_text(text: string) -> nil`

两平台均用文字替换剪贴板内容，无返回值或成功标志，不保留其他剪贴板格式。参见[剪贴板示例](lua-examples.md)。

```lua
ms.set_clipboard_text("来自 Mousetroke 的问候")
```

### `ms.read_text`

`ms.read_text(path: string) -> string | nil`

两平台均完整读取 UTF-8 文件，读取或 UTF-8 解码失败返回 `nil`。相对路径基于进程工作目录，没有约定的脚本文件夹。读取内容受可用内存约束。

```lua
local text = ms.read_text("config.json")
ms.log(text and #text or 0)
```

## 窗口

### `ms.find_win_by_class`

`ms.find_win_by_class(pattern: string) -> table | nil`

使用 **Rust 正则表达式**查找可见顶层窗口：Windows 匹配窗口类名，Mac 匹配应用 **bundle ID**（例如 Lua 字符串 `^com\\.apple\\.Safari$`）。返回从 1 开始的数组，最多 32 项，每项 `{text=标题, hwnd=窗口标识}`；未命中或正则无效时为空表。默认区分大小写，可用 `(?i)` 等标志，不使用 Lua 模式。Mac 通过辅助功能读取标题，缺少权限返回 nil 并仅记录一次日志；`hwnd` 是 `focus_win` 和窗口状态 API 接受的 CGWindowID。Mac 仅搜索当前屏幕上可见的窗口。

```lua
local wins = ms.find_win_by_class("^Notepad$")
if wins and wins[1] then ms.focus_win(wins[1].hwnd) end
```

### `ms.find_win_by_title`

`ms.find_win_by_title(pattern: string) -> table | nil`

两平台按标题的 Rust 正则表达式查找窗口，返回与 `find_win_by_class` 相同的、从 1 开始的 `{text, hwnd}` 数组，最多 32 项。无效正则为空表。Mac 使用 AX 窗口标题，需要辅助功能权限，仅搜索当前屏幕上可见的窗口。

```lua
local wins = ms.find_win_by_title("(?i)notepad")
if wins then ms.log(#wins) end
```

### `ms.focus_win`

`ms.focus_win(wid: integer) -> nil`

激活窗口／程序，无返回值或成功标志。Windows 接受 HWND。Mac 接受手势目标或查找得到的 CGWindowID，取消最小化并提升对应 AX 窗口，再经主线程桥接激活所属应用。Mac AX 调用在脚本工作线程执行，复用内建动作按 PID／边界定位窗口的逻辑，需要辅助功能权限；忽略失效 ID 或不可用的 AX 窗口。边界完全相同的重叠窗口可能存在定位歧义。

```lua
if wid ~= 0 then ms.focus_win(wid) end
```

### `ms.set_win_state`

`ms.set_win_state(wid: integer, state: string) -> nil`

Windows：`"min"` 最小化，`"max"` 最大化；**其他任意字符串都执行还原**，包括 `"normal"` 和 `"norm"`。Mac：`"min"` 使用 AXMinimized；`"max"` 取消最小化／退出全屏，窗口未填满屏幕工作区时按下与内建动作相同的 AXZoomButton。`"normal"`、`"norm"`、`"restore"` 取消最小化／退出全屏，并撤销填满工作区的缩放。Mac 另支持 `"fullscreen"`，通过 AXFullScreen 请求应用全屏（取决于应用支持）；其他名称忽略并仅记录一次日志。值区分大小写，无返回值。Mac 需要辅助功能权限；AX 在工作线程执行，经主线程桥接读取屏幕几何信息。状态转换可能异步完成。

Mac 的缩放行为由应用决定：按内容大小缩放后可能仍报告 `"normal"`，此时本 API 无法可靠还原，重复请求 `"max"` 可能切换缩放。此类应用请用内建“最大化 / 还原”切换动作。公开 AX 接口没有统一的缩放状态标志。

```lua
if wid ~= 0 then ms.set_win_state(wid, "max") end
```

### `ms.get_win_state`

`ms.get_win_state(wid: integer) -> string | nil`

Windows 返回 `"min"`、`"max"` 或 `"normal"`，不查询置顶，也不校验失效句柄，失效句柄可能表现为 normal。Mac 返回 `"min"`、`"fullscreen"`、`"max"`（边界与某屏工作区的差不超过 2 个逻辑点）或 `"normal"`。公开 AX 接口没有通用的缩放状态查询，按内容大小缩放的窗口报告 normal。Mac 缺少权限、AX 窗口／状态不可用或 ID 失效时返回 nil；权限拒绝仅记录一次日志。

```lua
if wid ~= 0 then ms.log(ms.get_win_state(wid)) end
```

### `ms.set_win_top`

`ms.set_win_top(wid: integer, top: boolean) -> nil`

Windows 用 `true` 设置、`false` 取消置顶，不移动或改变窗口大小，无返回值。这是设置状态，不是切换，也不会返回原状态。Mac 无操作并仅记录一次日志：系统没有公开的跨应用窗口置顶 API，内建置顶动作同样不支持。需要切换时，Windows 编辑器选择“窗口操作 → 置顶 / 取消置顶”。

```lua
if wid ~= 0 then ms.set_win_top(wid, true) end
```

## 鼠标与屏幕

### `ms.get_mouse_pos`

`ms.get_mouse_pos() -> table`

无参数，两平台均返回当前鼠标位置 `{x=整数, y=整数}`，可能与入口 `pos` 不同。Windows 使用屏幕像素，Mac 使用 Quartz 全局逻辑点，Y 向下增加。Mac 查询失败时返回 `{x=0,y=0}`。

```lua
local p = ms.get_mouse_pos()
ms.log("%d,%d", p.x, p.y)
```

### `ms.set_mouse_pos`

`ms.set_mouse_pos(x: integer, y: integer) -> nil`

两平台均将鼠标移动到全局屏幕坐标，无返回值，坐标可以为负数。Mac 发送带标记的移动事件，避免再次触发手势。

```lua
local p = ms.get_mouse_pos()
ms.set_mouse_pos(p.x + 20, p.y)
```

### `ms.get_screen`

`ms.get_screen(x: integer, y: integer) -> table | nil`

返回完整屏幕矩形 `{left, top, right, bottom}`，字段为全局整数坐标，不是扣除任务栏等区域的工作区。宽为 `right-left`，高为 `bottom-top`。Windows 在点不属于任何屏幕时选最近显示器；Mac 选包含该点的显示器，否则回退到缓存的第一台，无屏幕或桥接结果时可能为 `nil`。两平台支持。

```lua
local p = ms.get_mouse_pos()
local s = ms.get_screen(p.x, p.y)
if s then ms.log(s.right - s.left, s.bottom - s.top) end
```

### `ms.get_brightness`

`ms.get_brightness(pos?: table) -> integer | nil`

可选 `{x=整数,y=整数}` 指定屏幕，省略／nil 或位置不完整时使用当前鼠标。两平台统一为**成功返回 0–100 的亮度百分比，失败返回 nil**。Windows 使用 DDC/CI，失败回退 WMI；WMI 使用第一个亮度设备，并未按鼠标所在显示器匹配。Mac 当前返回 nil 并仅记录一次日志：现有亮度后端仅发送内建屏幕亮度键，不能读取百分比，也不支持外接 DDC。仅 Windows 旧接口 `WG.GetBrightness` 保留失败返回 -1。

```lua
local value = ms.get_brightness()
if value ~= nil then ms.show_toast(tostring(value)) end
```

### `ms.set_brightness`

`ms.set_brightness(pos: table | nil, delta: integer) -> integer | nil`

虽然叫 set，但它是增减 `delta` 个百分点，不是设置绝对亮度。第一参传命名字段表 `{x=...,y=...}`，或**明确传 nil** 使用当前鼠标。成功返回限制在 **0–100 的新亮度百分比**，两平台失败均为 **nil**。Windows 的 DDC/CI 与 WMI 设备选择同 `get_brightness`；仅 Windows 旧接口 `WG.SetBrightness` 保留失败返回 -1。Mac 返回 nil 并仅记录一次日志：现有“媒体与系统”后端只发送内建屏幕亮度键，没有数值读回或按百分点控制能力。本 Lua 调用不改变亮度；亮度键请使用内建动作，外接 DDC 仍不支持。

```lua
local value = ms.set_brightness(nil, 5)
ms.log(value)
```

## 网址与进程

### `ms.open`

`ms.open(url: string) -> nil`

使用默认处理程序打开 URL，无返回值或成功标志。Windows 使用 Shell 打开，也接受文件／文件夹路径。Mac 需要 URL，文件可用正确编码的 `file://` URL，或通过 `ms.cmd` 用 `/usr/bin/open` 打开路径。不做变量替换。

```lua
ms.open("https://mousetroke.pages.dev/")
```

### `ms.run`

`ms.run(file: string, args?: string) -> nil`

可选参数为**字符串**，不是 Lua 数组，无返回值或进程号。Windows 普通启动、不提权，采用再次运行行为。Mac 接受 bundle id、应用路径或可执行文件绝对路径，按引号规则拆分参数；LaunchServices 拒绝命令行可执行文件时可回退后台进程，并请求新实例。不展开变量，需要时用 Lua 拼接，不返回进程输出。

```lua
if package.config:sub(1, 1) == "\\" then
  ms.run("cmd.exe", "/K echo Hello")
else
  ms.run("/usr/bin/say", '"Hello from Mousetroke"')
end
```

### `ms.cmd`

`ms.cmd(command: string) -> nil`

Windows 通过隐藏的 `cmd.exe /C`，Mac 通过 `/bin/zsh -c` 执行 Shell 代码。无返回值、输出或退出码，不等待进程结束。Mac 管理的后台进程并发上限为 16，结束时记日志。按原字符串执行，此 Lua API 不展开 `%MS_*%` 占位符，Shell 参数需自行加引号。

```lua
if package.config:sub(1, 1) == "\\" then
  ms.cmd("taskmgr.exe")
else
  ms.cmd('/usr/bin/open -a "Activity Monitor"')
end
```

继续阅读：[按键语法](key-sequences.md)、[实用示例](lua-examples.md)、[日志位置](config.md#文件位置)。
