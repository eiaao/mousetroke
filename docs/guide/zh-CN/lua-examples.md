---
title: "Lua 实用示例"
description: "直接复制的鼠标手势 Lua 脚本：输出文本、剪贴板粘贴、日期、选区搜索、窗口、亮度、程序与定时暂停。"
order: 10
---

# Lua 实用示例

将一个完整代码块粘贴到手势的“Lua 脚本”动作中，每块都定义了 `ms_action`。文字示例先在临时文档中试用。编辑方法见 [Lua 入门](lua-basics.md)，平台支持见 [Lua API](lua-api.md)。

跨平台示例用 `package.config` 选择 Windows 的 `^`（Control）或 Mac 的 `#`（Command）。剪贴板示例只恢复**文字**，不保留图片等格式。200 ms 等待只是示例，不保证每个程序都足够，较慢时请增加。示例会临时替换剪贴板，之后恢复保存的文字；期间新复制的内容也会被恢复操作覆盖。

## 输出固定文本

用途：用一次鼠标手势输入常用句子。

```lua
function ms_action()
  ms.keys("你好，来自 Mousetroke 的问候。", true)
end
```

两平台的文本优先模式都支持 Unicode，但仍会解析语法字符。任意包含 `%`、`+`、花括号的文字，建议用下面的剪贴板方法。

## 粘贴后恢复剪贴板文字

用途：按原样粘贴文字，再恢复剪贴板之前的文字。

```lua
function ms_action()
  local mod = package.config:sub(1, 1) == "\\" and "^" or "#"
  local saved = ms.get_clipboard_text()
  ms.set_clipboard_text("价格：50% + 税费 {示例}")
  ms.keys(mod .. "v")
  ms.sleep(200)
  ms.set_clipboard_text(saved or "")
end
```

这样粘贴的内容不经过按键序列转义。原来没有文字时，最终剪贴板变为空文字；这些 API 无法恢复原来的其他格式。

## 按程序输出不同文字

用途：根据手势目标的程序身份选择句子。

```lua
function ms_action(pos, wid, pid, exe)
  local name = string.lower(exe)
  local text = "来自 Mousetroke 的问候"
  if string.find(name, "firefox", 1, true) then
    text = "你好 Firefox"
  elseif name == "com.apple.safari" then
    text = "你好 Safari"
  end
  ms.keys(text, true)
end
```

Windows 通常传完整可执行文件路径，Mac 通常传 bundle id。这里检查的是手势目标，输入仍由活动程序接收；需要先激活目标程序时，可先调用 `ms.focus_win(wid)`。

## 插入今天日期

用途：按年-月-日格式插入本地日期。

```lua
function ms_action()
  ms.keys(os.date("%Y-%m-%d"), true)
end
```

运行时可以使用 `os.date`。这里 `%` 是 Lua 日期格式语法，最终输出的数字和连字符适用于文本优先输入。

## 在网站搜索选中文字

用途：复制选区，按 UTF-8 编码后打开 Google 搜索。

```lua
local function url_encode(text)
  return (text:gsub("([^A-Za-z0-9%-._~])", function(c)
    return string.format("%%%02X", string.byte(c))
  end))
end

function ms_action()
  local mod = package.config:sub(1, 1) == "\\" and "^" or "#"
  local saved = ms.get_clipboard_text()
  ms.set_clipboard_text("")
  ms.keys(mod .. "c")
  ms.sleep(200)
  local text = ms.get_clipboard_text() or ""
  ms.set_clipboard_text(saved or "")
  if text ~= "" then
    ms.open("https://www.google.com/search?q=" .. url_encode(text))
  else
    ms.show_toast("没有复制到选中文字")
  end
end
```

先在活动程序中选中文字再触发。复制前清空剪贴板，避免复制无效时搜索旧内容。Lua 不自动替换选区变量；更简单的[打开网址动作](actions.md#打开网址)可以自动采集选区。

## 给选中文字加引号或括号

用途：在可编辑文本框中，将选区替换为带引号的文字。

```lua
function ms_action()
  local mod = package.config:sub(1, 1) == "\\" and "^" or "#"
  local saved = ms.get_clipboard_text()
  ms.set_clipboard_text("")
  ms.keys(mod .. "c")
  ms.sleep(200)
  local text = ms.get_clipboard_text() or ""
  if text ~= "" then
    ms.set_clipboard_text('"' .. text .. '"')
    ms.keys(mod .. "v")
    ms.sleep(200)
  end
  ms.set_clipboard_text(saved or "")
end
```

改成方括号时，将包裹表达式替换为 `"[" .. text .. "]"`。复制后需要保留选中状态，粘贴才能覆盖选区；只读网页不能这样编辑。

## 切换窗口置顶

用途：切换 Windows 窗口是否置顶。此需求请选“窗口操作 → 置顶 / 取消置顶”，该动作会查询实际状态并切换。Lua API 只能设置指定状态，没有置顶查询函数，每次手势的 VM 也是新建的。Mac 不支持跨程序置顶。

Windows 如需单独的“固定此窗口”Lua 手势，可以用：

```lua
function ms_action(pos, wid)
  if package.config:sub(1, 1) ~= "\\" then
    ms.show_toast("Mac 暂不支持窗口置顶")
    return
  end
  if wid ~= 0 then ms.set_win_top(wid, true) end
end
```

另建取消置顶手势时将 `true` 改为 `false`。使用 Lua 全局布尔变量会在每次执行时重置，不能实现切换。

## 最大化或还原

用途：在 Windows 切换目标窗口的最大化状态。

```lua
function ms_action(pos, wid)
  if package.config:sub(1, 1) ~= "\\" then
    ms.show_toast("Mac 请使用窗口操作中的最大化 / 还原")
    return
  end
  if wid == 0 then return end
  local state = ms.get_win_state(wid)
  ms.set_win_state(wid, state == "max" and "normal" or "max")
end
```

Mac 请选“窗口操作 → 最大化 / 还原”，内置动作使用辅助功能，Lua 窗口状态 API 尚不支持。当前没有可将窗口移动到另一显示器的 Lua 位置／大小 API。

## 调整鼠标所在屏幕亮度

用途：Windows 将鼠标所在屏幕调亮 5 个百分点。

```lua
function ms_action()
  local p = ms.get_mouse_pos()
  local value = ms.set_brightness(p, 5)
  if value and value >= 0 then
    ms.show_toast("亮度：" .. value)
  else
    ms.show_toast("当前屏幕或平台无法使用亮度 API")
  end
end
```

将 `5` 改成 `-5` 可另建调暗手势。Windows 需要可用的 DDC/CI 或 WMI 硬件支持，WMI 回退会选择第一个亮度设备，可能是另一块屏幕。Mac 此处返回 nil；可用“媒体与系统 → 亮度 +”／“亮度 −”发送内置屏幕亮度键，不支持按鼠标选择屏幕或外接 DDC/CI。

## 打开文件夹

用途：用系统文件管理器打开当前用户的下载文件夹。

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.cmd('explorer.exe "%USERPROFILE%\\Downloads"')
  else
    ms.cmd('/usr/bin/open "$HOME/Downloads"')
  end
end
```

这些环境变量由系统 Shell 展开，不是 Mousetroke 动作变量。示例假设下载文件夹位于通常位置。

## 带参数运行程序

用途：向程序传递参数字符串。

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.run("cmd.exe", "/K echo Hello from Mousetroke")
  else
    ms.run("/usr/bin/say", '"Hello from Mousetroke"')
  end
end
```

Windows 打开命令窗口显示文字；Mac 用 `say` 朗读。`ms.run` 的参数是一段字符串，不是表，也不返回进程句柄。

## 打开任务管理器或活动监视器

用途：打开当前平台的进程查看工具。

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.keys("^+{ESC}")
  else
    ms.run("com.apple.ActivityMonitor")
  end
end
```

Windows 使用 Ctrl+Shift+Esc；Mac 按 bundle id 启动活动监视器。

## 锁屏

用途：使用系统快捷键锁定当前会话。

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.keys("#l")
  else
    ms.keys("^#q")
  end
end
```

Mac 是 Control+Command+Q，Windows 是 Win+L。Ctrl+Alt+Del 不能作为普通模拟按键序列发送。

## 暂停十分钟

用途：临时暂停引擎，Mousetroke 持续运行时到时恢复。

```lua
function ms_action()
  ms.set_paused(true)
  ms.show_toast("手势暂停十分钟", 2000)
  ms.sleep(10 * 60 * 1000)
  ms.set_paused(false)
  ms.show_toast("手势已恢复", 1500)
end
```

两平台均可用，会保留一个休眠十分钟的 Lua 工作线程。这不是持久计时器，退出或重启就会结束。期间可以手动恢复，但等待结束时脚本仍会再发恢复命令。避免启动多个副本。详见[运行限制](lua-basics.md#运行环境与限制)。
