---
title: "Lua API"
description: "Complete Lua 5.4 ms API reference with signatures, return values, examples and Windows/macOS support notes."
order: 9
---

# Lua API

These are all functions registered on `ms` by the shared runtime. Examples are statements to place inside `ms_action`, where `pos`, `wid`, `pid`, `exe`, `title` and `mode` are available. See [Lua basics](lua-basics.md) for a complete script and [recipes](lua-examples.md) for ready-to-use actions.

## Conventions

Lua version: **5.4** (`mlua` with `lua54` and `vendored`). `nil` means no value; `?` marks an optional parameter. Functions returning no values are shown as `-> nil`. An omitted optional argument is equivalent to nil. Pass a real boolean for boolean arguments. Integer parameters truncate finite fractions toward zero and reject values outside their underlying integer range; use integers in scripts.

Position tables use named fields `{x=..., y=...}`, not array entries. Unsupported Mac calls generally log once per function per process; getters return nil and setters do nothing. A no-value setter is not a success check. Lua API strings do not receive action-variable substitution.

## Flow and feedback

### `ms.log`

`ms.log(...) -> nil`

Accepts any Lua values. With a format string and more arguments, supports `%s`, `%d`, `%f`, `%q` and `%%`; otherwise joins values with spaces. `%q` does not add Lua quoting. Writes a `Lua:` line to the runtime log on both platforms. Returns no value.

```lua
ms.log("pid=%d app=%s", pid, exe)
```

### `ms.sleep`

`ms.sleep(milliseconds: integer) -> nil`

Waits for a nonnegative millisecond count (unsigned 64-bit). Blocks this script worker on both platforms; the gesture engine/main thread continues. No return value and no script timeout is installed.

```lua
ms.sleep(100)
```

### `ms.is_paused`

`ms.is_paused() -> boolean`

No parameters. Returns the manual engine pause flag on both platforms. It does not report automatic fullscreen suppression or an app exclusion.

```lua
ms.log(ms.is_paused())
```

### `ms.set_paused`

`ms.set_paused(paused: boolean) -> nil`

`true` pauses; `false` resumes. No return value. Supported on both platforms. Windows posts a command to the tray thread, so an immediate `is_paused()` can still see the old value. Mac applies through the main-thread bridge.

```lua
ms.set_paused(true)
```

### `ms.show_settings`

`ms.show_settings() -> nil`

No parameters or return value. Opens or brings forward settings on both platforms.

```lua
ms.show_settings()
```

### `ms.show_toast`

`ms.show_toast(message: string, duration_ms?: integer) -> nil`

`message` is the displayed text. Optional nonnegative duration is in milliseconds, default **1200**. Displays an overlay message on both platforms; returns no value.

```lua
ms.show_toast("Done", 1500)
```

## Keyboard input

### `ms.keys`

`ms.keys(sequence: string, text_by_default?: boolean) -> nil`

Parses and sends the [key sequence](key-sequences.md); text-first mode defaults to `false`. Returns after sending; malformed syntax raises a Lua error. Both platforms support ordinary keyboard/text input. Mac skips unsupported keys, mouse and wheel actions, and enforces its queue/text limits. It waits on its main-thread input queue, which may also contain other sequences.

```lua
ms.keys("Hello", true)
```

### `ms.keys_down`

`ms.keys_down(sequence: string) -> nil`

Windows only: parses in key mode and presses keys/modifiers without releasing. Repeats are not repeated in this phase; wheel actions fire once. Invalid syntax raises a Lua error. Pair with `keys_up` to release. Mac logs unsupported and does nothing. No return value.

```lua
ms.keys_down("^a")
ms.sleep(50)
ms.keys_up("^a")
```

### `ms.keys_up`

`ms.keys_up(sequence: string) -> nil`

Windows only: releases the parsed sequence in reverse action order, with modifiers released last. Sleep steps still wait. Syntax errors raise a Lua error. Mac logs unsupported and does nothing. No return value.

```lua
ms.keys_down("{SHIFT}")
ms.keys_up("{SHIFT}")
```

### `ms.is_keys_pressed`

`ms.is_keys_pressed(sequence: string) -> boolean | nil`

Both platforms check every keyboard key **and its prefix modifiers**: `"^a"` requires Control and A. Either side satisfies a prefix; explicit names such as `{RCTRL}` require that side. On Mac `^` = Control, `%` = Option, `+` = Shift and `#` = Command. Multiple steps mean all listed keys/modifiers must be down together, not that they were pressed in order; repeats add no requirement. Mouse/wheel, sleep or text actions make the test false; an empty sequence returns true. Invalid syntax returns false. Mac reads the combined session state and returns false without Accessibility permission, logging once.

```lua
if ms.is_keys_pressed("^a") then
  ms.show_toast("Ctrl and A are down")
end
```

## Clipboard and files

### `ms.get_clipboard_text`

`ms.get_clipboard_text() -> string | nil`

No parameters. Returns clipboard text, or `nil` if text is unavailable. Supported on both platforms; this does not fetch the selection automatically.

```lua
local text = ms.get_clipboard_text()
ms.log(text or "No text")
```

### `ms.set_clipboard_text`

`ms.set_clipboard_text(text: string) -> nil`

Replaces clipboard contents with text on both platforms; no return value or success flag. Other clipboard formats are not preserved. See [clipboard recipes](lua-examples.md).

```lua
ms.set_clipboard_text("Hello from Mousetroke")
```

### `ms.read_text`

`ms.read_text(path: string) -> string | nil`

Reads the entire file as UTF-8 on both platforms. Returns `nil` on a read or UTF-8 decoding error. Relative paths use the process working directory, not a documented scripts folder. Reads count toward available memory.

```lua
local text = ms.read_text("config.json")
ms.log(text and #text or 0)
```

## Windows

### `ms.find_win_by_class`

`ms.find_win_by_class(pattern: string) -> table | nil`

Searches visible top-level windows using a **Rust regular expression**, on Windows class names or Mac application **bundle IDs** (for example `^com\\.apple\\.Safari$` in a Lua string). Returns up to 32 entries in a 1-based array: `{text=title, hwnd=window_id}`; no match or an invalid regex gives an empty table. Matching is case-sensitive unless the pattern uses a flag such as `(?i)`; these are not Lua patterns. Mac reads titles through Accessibility, returning nil without permission and logging once. Its `hwnd` is a CGWindowID accepted by `focus_win` and the window-state APIs. Only currently on-screen windows are searched on Mac.

```lua
local wins = ms.find_win_by_class("^Notepad$")
if wins and wins[1] then ms.focus_win(wins[1].hwnd) end
```

### `ms.find_win_by_title`

`ms.find_win_by_title(pattern: string) -> table | nil`

Searches window titles on both platforms using a Rust regular expression, returning the same 1-based `{text, hwnd}` entries as `find_win_by_class`, at most 32. An invalid regex gives an empty table. Mac uses AX window titles, requires Accessibility permission and searches only currently on-screen windows.

```lua
local wins = ms.find_win_by_title("(?i)notepad")
if wins then ms.log(#wins) end
```

### `ms.focus_win`

`ms.focus_win(wid: integer) -> nil`

Activates a window/app; no return value or success flag. Windows takes an HWND. Mac accepts the gesture target or a discovered CGWindowID, unminimizes and raises its AX window, then activates the owning app through the main-thread bridge. Mac AX calls run on the script worker, reuse the built-in window lookup by PID/bounds and require Accessibility permission; stale IDs or unavailable AX windows are ignored. Apps with identical overlapping window bounds may be ambiguous.

```lua
if wid ~= 0 then ms.focus_win(wid) end
```

### `ms.set_win_state`

`ms.set_win_state(wid: integer, state: string) -> nil`

Windows: `"min"` minimizes, `"max"` maximizes; **every other string restores**, including `"normal"` and `"norm"`. Mac: `"min"` uses AXMinimized; `"max"` unminimizes/exits fullscreen and presses the same AXZoomButton as the built-in action when the window does not fill its screen's work area. `"normal"`, `"norm"` and `"restore"` unminimize/exit fullscreen and undo work-area zoom. Mac also accepts `"fullscreen"` through AXFullScreen where the app supports it; every other name is ignored and logged once. Values are case-sensitive. No return value. Mac requires Accessibility permission; AX runs on the worker and screen geometry comes through the main-thread bridge. Transitions can be asynchronous.

Mac Zoom is app-defined: a content-sized zoom may still report `"normal"` and cannot be reliably restored with this API; repeated `"max"` requests may toggle it. For such apps use the built-in **Maximize / restore** toggle. There is no public universal AX zoom-state flag.

```lua
if wid ~= 0 then ms.set_win_state(wid, "max") end
```

### `ms.get_win_state`

`ms.get_win_state(wid: integer) -> string | nil`

Windows returns `"min"`, `"max"` or `"normal"`. This is not a topmost query and does not validate a stale handle; an invalid handle can appear normal. Mac returns `"min"`, `"fullscreen"`, `"max"` (frame matches a screen work area within 2 logical points) or `"normal"`. Mac has no universal public AX zoom-state getter, so content-sized zoom reports normal. It returns nil for missing permission, an unavailable AX window/state or a stale ID; permission denial logs once.

```lua
if wid ~= 0 then ms.log(ms.get_win_state(wid)) end
```

### `ms.set_win_top`

`ms.set_win_top(wid: integer, top: boolean) -> nil`

Windows sets (`true`) or clears (`false`) always-on-top without moving/resizing the window; no return value. This sets a state, does not toggle it or return the previous state. Mac does nothing and logs once: macOS has no public API for pinning another app's window, and the built-in topmost action is also unsupported. For a toggle, choose **Window actions → Always on top / off** in the Windows editor.

```lua
if wid ~= 0 then ms.set_win_top(wid, true) end
```

## Mouse and screen

### `ms.get_mouse_pos`

`ms.get_mouse_pos() -> table`

No parameters. Both platforms return `{x=integer, y=integer}` for the current cursor position, which can differ from entry `pos`. Windows uses screen pixels; Mac uses Quartz global logical points, with Y increasing downwards. Mac returns `{x=0,y=0}` if the cursor query fails.

```lua
local p = ms.get_mouse_pos()
ms.log("%d,%d", p.x, p.y)
```

### `ms.set_mouse_pos`

`ms.set_mouse_pos(x: integer, y: integer) -> nil`

Moves the pointer to global screen coordinates on both platforms; no return value. Coordinates may be negative. Mac posts a tagged movement event so it does not start another gesture.

```lua
local p = ms.get_mouse_pos()
ms.set_mouse_pos(p.x + 20, p.y)
```

### `ms.get_screen`

`ms.get_screen(x: integer, y: integer) -> table | nil`

Returns the full screen rectangle `{left, top, right, bottom}` as integer global coordinates, not the usable work area. Width is `right-left`; height is `bottom-top`. Windows chooses the nearest monitor if the point is outside all monitors. Mac chooses the containing display, or falls back to the first cached display; it can return `nil` if no screen/bridge result is available. Both platforms support this call.

```lua
local p = ms.get_mouse_pos()
local s = ms.get_screen(p.x, p.y)
if s then ms.log(s.right - s.left, s.bottom - s.top) end
```

### `ms.get_brightness`

`ms.get_brightness(pos?: table) -> integer | nil`

Optional `{x=integer,y=integer}` chooses the screen; omitted/nil or an incomplete position falls back to the current cursor. Returns **0–100 percent on success, nil on failure** on either platform. Windows uses DDC/CI with WMI fallback. WMI uses its first brightness device, not a cursor-matched monitor. Mac currently returns nil and logs once: its existing brightness backend only sends built-in-display brightness key events, which cannot read a percentage. External DDC is unsupported. Only the Windows legacy `WG.GetBrightness` wrapper retains -1 on failure.

```lua
local value = ms.get_brightness()
if value ~= nil then ms.show_toast(tostring(value)) end
```

### `ms.set_brightness`

`ms.set_brightness(pos: table | nil, delta: integer) -> integer | nil`

Despite the name, this adjusts by `delta` percentage points; it does not set an absolute percentage. Pass a named `{x=...,y=...}` table or **nil as the first argument** to use the cursor. Success returns the new **0–100 percentage**, clamped to that range; failure returns **nil on both platforms**. Windows uses the same DDC/CI and WMI selection as `get_brightness`. Only the Windows legacy `WG.SetBrightness` wrapper retains -1 on failure. Mac returns nil and logs once: the existing Media & system backend only sends built-in-display brightness keys, with no numeric readback or percentage-point control. No brightness change is made by this Lua call; use those built-in actions for brightness keys. External DDC remains unsupported.

```lua
local value = ms.set_brightness(nil, 5)
ms.log(value)
```

## URLs and processes

### `ms.open`

`ms.open(url: string) -> nil`

Opens a URL with the default handler; no return value or success flag. Windows uses shell open and also accepts a file/folder path. Mac expects a URL; use a properly encoded `file://` URL for a file, or `ms.cmd` to invoke `/usr/bin/open` on a path. No variable substitution is performed.

```lua
ms.open("https://mousetroke.pages.dev/")
```

### `ms.run`

`ms.run(file: string, args?: string) -> nil`

Launches with optional argument **string**, not a Lua array; returns no value or PID. Windows uses ordinary non-elevated launch with RunAgain behavior. Mac accepts a bundle ID, app path or absolute executable path; arguments are split with quote handling. Executable launch can fall back to a background process if LaunchServices rejects it. It requests a new instance. No variable expansion; use Lua concatenation. Process output is not returned.

```lua
if package.config:sub(1, 1) == "\\" then
  ms.run("cmd.exe", "/K echo Hello")
else
  ms.run("/usr/bin/say", '"Hello from Mousetroke"')
end
```

### `ms.cmd`

`ms.cmd(command: string) -> nil`

Runs shell code via hidden `cmd.exe /C` on Windows or `/bin/zsh -c` on Mac. Returns no value, output or exit code, and does not wait for process completion. Mac limits concurrently managed background processes to 16 and logs completion. Strings execute as supplied; `%MS_*%` placeholders are not expanded by this Lua API. Quote shell arguments yourself.

```lua
if package.config:sub(1, 1) == "\\" then
  ms.cmd("taskmgr.exe")
else
  ms.cmd('/usr/bin/open -a "Activity Monitor"')
end
```

Continue with [send-keys syntax](key-sequences.md), [recipes](lua-examples.md) or [log locations](config.md#file-locations).
