---
title: "Lua recipes"
description: "Copy ready-to-use mouse gesture Lua scripts for text, clipboard paste, dates, web search, windows, brightness and pause."
order: 10
---

# Lua recipes

Paste one complete block into a gesture's **Lua script** action. Each block defines `ms_action`. Try text recipes in a disposable text document first. [Lua basics](lua-basics.md) explains the editor and [Lua API](lua-api.md) lists platform support.

Cross-platform recipes use `package.config` to choose `^` (Control on Windows) or `#` (Command on Mac). Clipboard recipes preserve **text only**, not images or other formats. Their 200 ms waits are examples, not an application guarantee; increase them if the destination is slow. They replace clipboard contents temporarily and restore the saved text, even if you copy something else during that interval.

## Type fixed text

Purpose: insert a reusable sentence with one mouse gesture.

```lua
function ms_action()
  ms.keys("Hello from Mousetroke.", true)
end
```

Text-first mode supports Unicode on both platforms. It still parses syntax symbols; use the clipboard recipe for arbitrary punctuation such as `%`, `+` or braces.

## Paste and restore clipboard text

Purpose: paste literal text, then restore the text that was on the clipboard.

```lua
function ms_action()
  local mod = package.config:sub(1, 1) == "\\" and "^" or "#"
  local saved = ms.get_clipboard_text()
  ms.set_clipboard_text("Price: 50% + tax {example}")
  ms.keys(mod .. "v")
  ms.sleep(200)
  ms.set_clipboard_text(saved or "")
end
```

This bypasses key-sequence escaping for the inserted text. If there was no clipboard text, the final clipboard contains empty text; other original formats cannot be restored with these APIs.

## Different text per app

Purpose: use the gesture target's executable identity to choose a phrase.

```lua
function ms_action(pos, wid, pid, exe)
  local name = string.lower(exe)
  local text = "Hello from Mousetroke"
  if string.find(name, "firefox", 1, true) then
    text = "Hello Firefox"
  elseif name == "com.apple.safari" then
    text = "Hello Safari"
  end
  ms.keys(text, true)
end
```

Windows identities are normally full executable paths; Mac usually supplies a bundle ID. This checks the captured target, while typing goes to the active app. Use `ms.focus_win(wid)` first if you want to activate the target app.

## Insert today's date

Purpose: insert the local date as year-month-day.

```lua
function ms_action()
  ms.keys(os.date("%Y-%m-%d"), true)
end
```

`os.date` is available in the runtime. The `%` characters are Lua date-format syntax; its resulting digits and hyphens are safe for text-first input.

## Search selected text on a site

Purpose: copy the selection and open a UTF-8 encoded Google query.

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
    ms.show_toast("No selected text copied")
  end
end
```

Select text in the active app before triggering. Clearing the clipboard first avoids searching old clipboard text if Copy does nothing. Lua has no automatic selected-text substitution; the simpler [Open a URL action](actions.md#open-a-url) can collect the selection for you.

## Wrap selected text in quotes or brackets

Purpose: replace the selected text with a quoted version in an editable field.

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

For brackets, replace the wrapping expression with `"[" .. text .. "]"`. Copy must leave the text selected so Paste replaces it. Read-only pages cannot be edited this way.

## Toggle always-on-top

Purpose: toggle a Windows window's topmost state. Choose **Window actions → Always on top / off** for this recipe: that action can query and toggle the actual state. The Lua API only sets a requested state; it has no topmost getter and each gesture gets a fresh VM. Mac does not support cross-app topmost control.

If you want a separate “pin this window” Lua gesture on Windows, use:

```lua
function ms_action(pos, wid)
  if package.config:sub(1, 1) ~= "\\" then
    ms.show_toast("Always-on-top is unavailable on Mac")
    return
  end
  if wid ~= 0 then ms.set_win_top(wid, true) end
end
```

Use `false` instead of `true` for a separate unpin gesture. A Lua global boolean would reset on every execution and would not implement a toggle.

## Maximize or restore

Purpose: toggle the target window's maximized state on Windows.

```lua
function ms_action(pos, wid)
  if package.config:sub(1, 1) ~= "\\" then
    ms.show_toast("Use Window actions: Maximize / restore on Mac")
    return
  end
  if wid == 0 then return end
  local state = ms.get_win_state(wid)
  ms.set_win_state(wid, state == "max" and "normal" or "max")
end
```

On Mac choose **Window actions → Maximize / restore**; the built-in action uses Accessibility, while the Lua state APIs are unsupported. There is no Lua window-position/size API for moving a window to another monitor.

## Brightness under the cursor

Purpose: raise the cursor's screen brightness by 5 percentage points on Windows.

```lua
function ms_action()
  local p = ms.get_mouse_pos()
  local value = ms.set_brightness(p, 5)
  if value and value >= 0 then
    ms.show_toast("Brightness: " .. value)
  else
    ms.show_toast("Brightness API unavailable for this screen/platform")
  end
end
```

Change `5` to `-5` for a second, dimmer gesture. Windows needs compatible DDC/CI or WMI hardware; WMI falls back to its first brightness device, which may be a different screen. Mac returns nil here; use **Media & system → Brightness up / Brightness down** for built-in-screen brightness keys, without cursor-screen selection or external DDC/CI.

## Open a folder

Purpose: open the current user's Downloads folder with the platform's file browser.

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.cmd('explorer.exe "%USERPROFILE%\\Downloads"')
  else
    ms.cmd('/usr/bin/open "$HOME/Downloads"')
  end
end
```

These environment variables are expanded by the operating system shell, not Mousetroke's action variables. This assumes the usual Downloads location.

## Run a program with arguments

Purpose: pass an argument string to a program.

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.run("cmd.exe", "/K echo Hello from Mousetroke")
  else
    ms.run("/usr/bin/say", '"Hello from Mousetroke"')
  end
end
```

Windows opens a command window displaying a message. Mac speaks the phrase using `say`. `ms.run` accepts one argument string, not a table, and returns no process handle.

## Open Task Manager or Activity Monitor

Purpose: open the platform's process viewer.

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.keys("^+{ESC}")
  else
    ms.run("com.apple.ActivityMonitor")
  end
end
```

Windows uses Ctrl+Shift+Esc. Mac launches Activity Monitor by bundle ID.

## Lock the screen

Purpose: lock the current session using its system shortcut.

```lua
function ms_action()
  if package.config:sub(1, 1) == "\\" then
    ms.keys("#l")
  else
    ms.keys("^#q")
  end
end
```

Mac uses Control+Command+Q. Windows uses Win+L; Ctrl+Alt+Del cannot be synthesized as an ordinary key sequence.

## Pause gestures for ten minutes

Purpose: temporarily pause the engine, then resume it while Mousetroke remains running.

```lua
function ms_action()
  ms.set_paused(true)
  ms.show_toast("Gestures paused for ten minutes", 2000)
  ms.sleep(10 * 60 * 1000)
  ms.set_paused(false)
  ms.show_toast("Gestures resumed", 1500)
end
```

This works on both platforms and keeps one Lua worker asleep for ten minutes. It is not a persistent timer: quitting or restarting ends it. You can resume manually during the wait, but the script still issues Resume when its wait ends. Avoid starting multiple copies. See [runtime limits](lua-basics.md#runtime-and-limits).
