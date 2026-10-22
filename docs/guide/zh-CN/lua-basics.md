---
title: "Lua 入门"
description: "从零编写鼠标手势 Lua 脚本，理解 ms_action 参数、常用语法、日志调试和运行限制。"
order: 8
---

# Lua 入门

## 什么时候用 Lua

单个快捷键、窗口操作、网址、程序或命令，优先用[简单动作](actions.md)。需要按程序判断、加工文字或组合多步时，选择 Lua。Mousetroke 内置 **Lua 5.4**。

## 第一个脚本

1. 在设置中新建或选中手势。
2. 在“执行”中选择“Lua 脚本”，将整个文本框替换成下列代码。
3. 等待已保存提示，再到设置窗口外触发。

```lua
function ms_action(pos, wid, pid, exe, title, mode)
  ms.log("Target: %s, mode: %s", exe, mode)
  ms.show_toast("我的第一个 Lua 手势", 1500)
end
```

`function` 开始一个函数，`end` 结束函数。Mousetroke 加载代码后调用一次 `ms_action`。把要执行的操作放在函数里。括号中的名称接收手势上下文，不用的参数可以省略。`ms.log` 写运行日志，`ms.show_toast` 在屏幕上显示 1500 毫秒的提示。

编辑器的“Lua 帮助”打开本页。想让鼠标手势输出文本、通过剪贴板粘贴，可继续看 [Lua 实用示例](lua-examples.md)。

## 入口参数

完整入口是 `ms_action(pos, wid, pid, exe, title, mode)`：

| 参数 | Lua 类型 | 含义 |
|---|---|---|
| `pos` | 表 `{x=整数, y=整数}` | 分派动作时最后记录的鼠标坐标，其他显示器上可能为负数。 |
| `wid` | 整数 | 目标窗口标识，Windows 是 HWND，Mac 是窗口 ID。原样传给相应 API；没有目标时可能为 0。 |
| `pid` | 整数 | 目标进程号，没有时为 0。 |
| `exe` | 字符串 | Windows 程序身份，通常为完整路径；Mac 是 bundle id 或可执行文件路径，无法取得时为空。 |
| `title` | 字符串 | 目标窗口标题，无法取得时为空。Mac 在工作线程通过辅助功能读取。 |
| `mode` | 字符串 | 根据“偏好设置 → 手势作用于”，取固定值 `UnderCursor` 或 `Focused`，不代表鼠标按钮或手势名称。 |

目标在手势上下文中确定，`pos` 可能不同于动作变量中的手势开始坐标。键盘输入仍发给活动程序，除非先激活目标。Mac 的 `ms.focus_win(wid)` 只能激活本次手势的目标程序。Lua 不展开 `%MS_PID%` 之类占位符，请直接使用 `pid`。

## 五分钟语法

`local` 声明变量，字符串用单引号或双引号，`..` 拼接字符串。`--` 开始注释。`if`、`elseif`、`else`、`end` 用于分支。`nil` 表示没有值，条件中只有 `false` 和 `nil` 为假，0 和空字符串都为真。

```lua
function ms_action(pos, wid, pid, exe)
  local message = "你好"
  if string.find(string.lower(exe), "firefox", 1, true) then
    message = "你好，Firefox"
  else
    message = message .. "，来自 Mousetroke"
  end
  ms.show_toast(message)
end
```

`string.find(s, needle, 1, true)` 按字面查找，返回位置或 `nil`，其中 `true` 表示不把标点当模式。`string.match` 使用 Lua 模式，规则与正则表达式不同。表可以存放命名字段，也可以存放从 1 开始编号的列表。辅助函数可以返回结果：

```lua
local function app_name(identity)
  return string.match(identity, "([^/\\]+)$") or identity
end

function ms_action(pos, wid, pid, exe)
  local target = {name = app_name(exe), process = pid}
  local lines = {target.name, tostring(target.process)}
  ms.show_toast(table.concat(lines, " / "))
end
```

Lua 字符串会先处理转义，再交给按键解析器。`"\\"` 表示一个反斜杠，`"\n"` 表示换行；`[[文字]]` 这样的长括号字符串会保留反斜杠。[按键语法](key-sequences.md)介绍第二层语法。

## 调试

用 `ms.log("pid=%d title=%s", pid, title)` 记录值，再从“关于 → 打开日志”查看：

- Windows：`%APPDATA%\mousetroke\mousetroke.log`。
- Mac：`~/Library/Logs/mousetroke/mousetroke.log`。

加载错误和执行错误会写入日志，代码块名称为 `gesture_script`。`ms.show_toast("执行到这里")` 可以快速观察进度。可选返回值可能为 `nil`，先判断再做运算或读取字段。不支持的 Mac 调用见 [API 参考](lua-api.md)。

## 运行环境与限制

每次执行都会在工作线程创建独立 Lua VM，**Lua 分配内存上限 8 MiB**。全局变量不会跨手势保留。运行时没有指令数量限制或执行超时，请避免无限循环。`ms.sleep(ms)` 阻塞本次脚本的工作线程，不阻塞输入钩子或主界面线程。`ms.keys` 会等序列发送完再返回；Mac 由主线程计时器推进输入，Lua 工作线程等待。

可用标准库包括基础函数、`coroutine`、`table`、`io`、`os`、`string`、`utf8`、`math` 和 `package`。没有加载 `debug`，mlua 安全模式禁用原生 C 模块加载。可以用 `os.date` 取日期。这并不是操作系统安全沙箱：`io`、`os` 和进程 API 可使用当前用户权限访问文件、运行命令。请只运行可信脚本和配置。

继续阅读：[API 参考](lua-api.md)、[可复制示例](lua-examples.md)、[常见问题](troubleshooting.md)。
