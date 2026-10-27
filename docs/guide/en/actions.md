---
title: "Actions"
description: "Pick a preset from the action library, or edit advanced keys, windows, apps, URLs, commands, menus and Lua."
order: 5
---

# Actions

Select or create a gesture, then click **Choose action** under **Action**. Search for what you want to do, or browse a category; fill any parameters and click **Use this action**. The [action library](action-library.md) lists all presets and their Windows/Mac availability, including literal text, date/time, window layout and power actions.

Existing actions show **Change**. Presets that map to an editable advanced action also offer **Convert to advanced action**; native presets explain why conversion is unavailable. The **Advanced** category opens send keys, commands, Lua and menus. The editors below remain available, including through **Change action type**.

## Send keys

Enter a [key sequence](key-sequences.md), or click **Record**, press the shortcut and click **Done**. For example, `^c` copies on Windows; `#c` copies on Mac. **Syntax help** opens the syntax guide.

**Activate the window under the cursor before sending** activates the gesture target first. Without it, shortcuts go to whichever application receives keyboard input. The target also follows **Preferences → Gestures act on**. For literal text, choose **Type text** from the [action library](action-library.md), or use [text-first input](key-sequences.md#text-and-escaping) through Lua.

## Window actions

Choose one operation, applied to the gesture target:

| Operation | Behavior |
|---|---|
| Minimize | Minimize the window. |
| Maximize / restore | Toggle between maximized and restored size. |
| Close | Ask the window to close. |
| Always on top / off | Toggle topmost on Windows; unsupported and disabled in the Mac editor. |

Mac window control uses Accessibility. **Maximize / restore** presses the app's Zoom button, so the resulting size follows that app's behavior; it does not request native fullscreen and skips already minimized windows. Applications can differ in which window operations they expose.

## Open an app

**App** accepts an executable on Windows; **Browse…** selects it. On Mac use an app bundle ID such as `com.apple.Safari`, an `.app` path, or an executable path. **Arguments** is optional. On Mac, arguments are split with quote handling; use full paths for command-line executables.

**If already running** offers **Switch to it** or **Open another one**. Windows also offers **Run as administrator**. Mac does not support elevation or window-filter launch options. There are no class/title/regex fields in the current action editor.

## Open a URL

Fill **URL** and optionally **Browser**. Leave Browser empty for the default browser. Windows accepts a browser executable; Mac accepts a browser bundle ID or app path. The **Insert** buttons add selected text, clipboard text, app name, process ID or window title placeholders.

For selected-text web search, set URL to:

```text
https://www.google.com/search?q=%MS_SELECTED_TEXT%
```

Select text in an application, then perform the gesture. Selected text is URL-encoded within the template, so search and translation templates keep using the chosen service even when the selection is an address. Only when the URL field contains `%MS_SELECTED_TEXT%` alone does Mousetroke open selected addresses starting with `http://`, `https://` or `www.` directly; `www.` receives an `https://` prefix.

## Run a command

Enter the command in the text area. **Insert** adds variables. Windows runs it through a hidden `cmd.exe /C`; Mac uses `/bin/zsh -c`. This is shell syntax, so quote arguments correctly. Substituted text is not shell-escaped; avoid feeding untrusted selected text into commands.

For example, the following Windows command opens Task Manager:

```text
taskmgr.exe
```

On Mac, use **Open an app** with `com.apple.ActivityMonitor` instead.

## Media & system

The editor offers **Play / pause**, **Next track**, **Previous track**, **Volume up**, **Volume down**, **Mute**, **Brightness up** and **Brightness down**.

Windows brightness adjusts by 10 percentage points using DDC/CI with a WMI fallback. Mac brightness sends brightness keys for the built-in screen; external DDC/CI is unsupported. These Mac system actions are available even though Lua brightness functions are unsupported. See [screen edges](screen-edges.md).

## Show a menu

Click **+ Add a menu item**, enter **Menu item name**, then choose an action inside each item. Use **Move up**, **Move down** and **Delete the menu item** to organize it. The menu appears at the gesture position. The editor does not expose a column-count field and does not offer another menu type within an item. Mac supports a single column; multi-column settings are flagged as unsupported.

## Lua script

Paste a complete script defining `ms_action`. The editor links to [Lua basics](lua-basics.md), the [API reference](lua-api.md) and [examples](lua-examples.md). The text area accepts Tab as two spaces.

<a id="variables"></a>

## Variables

Use these placeholders in **Open a URL → URL**, **Run a command**, and **Open an app → App / Arguments**. They are not expanded in send-key strings, Lua source, menu names or Browser.

| Variable | Value |
|---|---|
| `%MS_SELECTED_TEXT%` | Selection captured when the action needs it; empty if unavailable. |
| `%MS_CLIPBOARD_TEXT%` | Current clipboard text, captured on demand. |
| `%MS_PID%` | Target process ID. |
| `%MS_EXE%` | Target executable identity: Windows path; Mac bundle ID or executable path. |
| `%MS_APP_NAME%` | File-name portion of the identity, which can be a bundle ID on Mac. |
| `%MS_WID%` | Platform window identifier as decimal text. |
| `%MS_WIN_NAME%` | Target window title; Mac fetches it through Accessibility when needed. |
| `%MS_MOUSE_X%`, `%MS_MOUSE_Y%` | Gesture-start screen coordinates. |
| `%MS_MOUSE_Y_FLIP%` | Virtual-screen bottom coordinate minus the gesture-start Y coordinate. |
| `%MS_TARGET_MODE%` | `UnderCursor` or `Focused`, according to the target preference. |

Unavailable target variables may be absent; unknown placeholders remain unchanged. Only selected text receives automatic URL encoding in a URL template. Other substitutions are inserted as-is. Lua uses [entry arguments](lua-basics.md#entry-arguments) and `ms` functions instead of placeholder expansion.
