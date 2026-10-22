---
title: "Lua basics"
description: "Write your first mouse gesture Lua script, learn ms_action arguments, debug with logs and understand runtime limits."
order: 8
---

# Lua basics

## When to use Lua

Use a [simple action](actions.md) for one shortcut, window operation, URL, application or command. Choose Lua when a gesture must branch by application, transform text or combine operations. Mousetroke embeds **Lua 5.4**.

## Your first script

1. Create or select a gesture in settings.
2. Under **Action**, choose **Lua script**. Replace the whole text area with the following code.
3. Wait for the saved indication, then trigger it outside settings.

```lua
function ms_action(pos, wid, pid, exe, title, mode)
  ms.log("Target: %s, mode: %s", exe, mode)
  ms.show_toast("My first Lua gesture", 1500)
end
```

`function` begins a function and `end` finishes it. Mousetroke loads the code, then calls `ms_action` once. Put your work inside that function. The names in parentheses receive context; you can omit unused arguments. `ms.log` writes to the runtime log and `ms.show_toast` displays a message for 1500 milliseconds.

The editor's **Lua help** opens this page. Follow [Lua recipes](lua-examples.md) to make a mouse gesture output text or paste through the clipboard.

## Entry arguments

The full entry is `ms_action(pos, wid, pid, exe, title, mode)`:

| Argument | Lua type | Meaning |
|---|---|---|
| `pos` | table `{x=integer, y=integer}` | Last cursor position recorded when the action is dispatched. Coordinates may be negative on another monitor. |
| `wid` | integer | Target window identifier; Windows HWND, Mac window ID. Treat it as an opaque identifier; it may be 0 without a target. |
| `pid` | integer | Target process ID, or 0. |
| `exe` | string | Windows executable identity, normally a full path; Mac bundle ID or executable path. Empty if unavailable. |
| `title` | string | Target window title, or an empty string. Mac fetches the title through Accessibility on the worker. |
| `mode` | string | Exactly `UnderCursor` or `Focused`, from **Preferences → Gestures act on**. It is not the trigger button or gesture name. |

The target is captured for the gesture. `pos` can differ from the gesture-start coordinates in the action variables. Keyboard input still goes to the active app unless you activate the target. On Mac, `ms.focus_win(wid)` can activate only this gesture's target app. Placeholder text such as `%MS_PID%` is not expanded in Lua; use `pid` directly.

## A five-minute Lua primer

`local` creates a variable. Strings use single or double quotes; `..` joins strings. `--` begins a comment. `if`, `elseif`, `else`, `end` choose a branch. `nil` means no value; only `false` and `nil` are false in conditions (0 and `""` are true).

```lua
function ms_action(pos, wid, pid, exe)
  local message = "Hello"
  if string.find(string.lower(exe), "firefox", 1, true) then
    message = "Hello, Firefox"
  else
    message = message .. " from Mousetroke"
  end
  ms.show_toast(message)
end
```

Here `string.find(s, needle, 1, true)` searches literally and returns a position or `nil`. The `true` avoids interpreting punctuation as a pattern. `string.match` uses Lua patterns, which differ from regular expressions. A table can store named fields or a list, whose first index is 1. A helper function can return a value:

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

Lua strings interpret escapes before key-sequence parsing. `"\\"` represents one backslash and `"\n"` a newline. Long-bracket strings such as `[[text]]` preserve backslashes. The [send-keys guide](key-sequences.md) explains the second layer of syntax.

## Debugging

Use `ms.log("pid=%d title=%s", pid, title)` and inspect **About → Open log**:

- Windows: `%APPDATA%\mousetroke\mousetroke.log`.
- Mac: `~/Library/Logs/mousetroke/mousetroke.log`.

Load errors and execution errors appear there with `gesture_script` as the chunk name. `ms.show_toast("Reached this step")` is a quick visual check. Optional API results can be `nil`; test them before arithmetic or table access. See the [API reference](lua-api.md) for unsupported Mac calls.

## Runtime and limits

Each script execution creates a fresh Lua VM on a worker thread, with an **8 MiB Lua allocation limit**. Globals do not persist between gestures. The runtime sets no instruction-count or wall-clock timeout. Avoid endless loops. `ms.sleep(ms)` blocks that script's worker, not the input hook or main UI thread. `ms.keys` waits for its sequence to finish; on Mac, input is advanced by the main-thread timer while the Lua worker waits.

Available standard libraries are the base functions, `coroutine`, `table`, `io`, `os`, `string`, `utf8`, `math` and `package`. `debug` is not loaded; native C module loading is disabled by mlua's safe mode. `os.date` is available. This is not an operating-system security sandbox: `io`, `os` and the process APIs can access files or run commands with your user rights. Only run scripts and configurations you trust.

Next: [API reference](lua-api.md), [copy-and-paste recipes](lua-examples.md), [troubleshooting](troubleshooting.md).
