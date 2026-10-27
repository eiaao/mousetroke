---
title: "Action library"
description: "Pick ready-to-use actions, set their parameters and check the full Windows and Mac catalogue."
order: 5.5
---

# Action library

Presets describe what you want to do, such as close a tab or center a window. Mousetroke stores the preset ID and parameters and chooses the Windows or Mac implementation when it runs. Browser and editor shortcuts still depend on the receiving app supporting them.

## Pick an action

1. Create or select a gesture, then click **Choose action** (or **Change** on an existing action).
2. Search by name, description, alias or shortcut spelling, such as `ctrl+w` on Windows or `cmd+w` / `⌘w` on Mac. Search covers all categories. Without a search, use **Common**, **Browser**, **Edit**, **Text**, **Window**, **Desktop & System**, **Media**, **Open** or **Mousetroke**.
3. Pick a result. Actions with parameters open a form: enter text, choose a date format or search service, or select a URL, file, folder or app. Click **Use this action** to confirm the form. Unavailable actions are disabled with a platform or phase-two reason.
4. Test the gesture outside settings. An empty gesture name follows the selected action name. Menu items use the same picker.

The search box receives focus when the picker opens. Use Up/Down to move through available results and Enter to select. Esc closes the parameter form first, then the picker. In multiline text, Ctrl+Enter or Command+Enter confirms the form.

**Convert to advanced action** replaces a preset with its current platform's editable keys, window operation, URL, app, command or system key. The confirmation explains that this is one way: the result no longer follows the preset's platform or language defaults. Native layout, power, literal-text, date/time, plain-copy, pause and settings actions keep their preset form; their cards explain why conversion is unavailable. The **Advanced** group at the end of the list offers send keys, commands, Lua and menus; the [advanced action reference](actions.md) also covers the existing editors.

## Text, windows and power

- **Type text** outputs the literal text, including spaces, line breaks, Chinese, emoji and characters such as `^+#{ENTER}`. **Paste** is the default: it temporarily uses the clipboard and restores its previous **text** after a short wait. If the clipboard contains images, HTML, file lists or other non-text data, or cannot be inspected, it types directly without changing the clipboard. `paste_delay_ms` is only visible for `paste`: the UI accepts 50–5000 ms, with a default of 150 ms. **Type** injects Unicode without using the clipboard. Both send input to the app receiving keyboard input.
- **Insert date/time** uses the local system clock and the chosen output method. Formats include ISO date, Chinese date, US date, time and date with time. A custom pattern accepts `YYYY`, `MM`, `DD`, `HH`, `mm` and `ss` (case sensitive). Other characters are literal; single quotes protect literal text and `''` inserts a quote. For example, `YYYY-MM-DD HH:mm:ss` gives a timestamp. Unclosed quotes are rejected.
- **Copy as plain text** sends Ctrl+C / Command+C, waits up to 500 ms for a clipboard version change, then replaces the copied content with its text only. If copying does not complete or has no text, it reports a failure and keeps the clipboard contents. Select text and keep that app focused before using it.
- Layout actions use the same gesture target as other window actions, following **Preferences → Gestures act on**. Windows uses native window positioning; Mac uses Accessibility and the screen's usable area. Minimum window sizes and app restrictions can prevent an exact half-screen layout. Exit Mac native fullscreen first. **Next monitor** cycles through screens in a stable order, preserves relative placement and maximization, and does nothing with one screen.
- **Turn off displays** lets input wake the displays again. **Sleep** requests normal system sleep and can be denied by the account or system policy. Failed native actions write one log line and show a brief on-screen message. **Pause / resume gestures** uses the same pause state as the tray and hotkey; use either of those to resume when gestures are paused. **Open settings** uses the regular settings window.

## Catalogue

The tables below are generated from the registry and include all 87 entries. ✓ means implemented on that platform; availability still depends on the app, hardware and required permissions. The gesture cheatsheet is reserved for phase two.

<!-- action-library:generated:start -->

### Browser

| Action / ID | What it does / where it works | Parameters | Windows | Mac |
|---|---|---|---|---|
| Back<br>`browser.back` | Works in web browsers. | — | ✓ | ✓ |
| Forward<br>`browser.forward` | Works in web browsers. | — | ✓ | ✓ |
| Refresh<br>`browser.reload` | Works in web browsers. | — | ✓ | ✓ |
| Reload without cache<br>`browser.hard_reload` | Works in web browsers. | — | ✓ | ✓ |
| New tab<br>`browser.new_tab` | Browsers, editors and other apps that support this shortcut. | — | ✓ | ✓ |
| Close tab<br>`browser.close_tab` | Browsers, editors and other apps that support this shortcut. | — | ✓ | ✓ |
| Reopen closed tab<br>`browser.reopen_tab` | Works in web browsers. | — | ✓ | ✓ |
| Previous tab<br>`browser.prev_tab` | Browsers, editors and other apps that support this shortcut. | — | ✓ | ✓ |
| Next tab<br>`browser.next_tab` | Browsers, editors and other apps that support this shortcut. | — | ✓ | ✓ |
| New window<br>`browser.new_window` | Browsers, editors and other apps that support this shortcut. | — | ✓ | ✓ |
| New private window<br>`browser.private_window` | Chrome, Edge and Safari; Firefox uses a different shortcut. | — | ✓ | ✓ |
| Focus address bar<br>`browser.address_bar` | Works in web browsers. | — | ✓ | ✓ |
| Find on page<br>`browser.find` | Browsers, editors and other apps that support this shortcut. | — | ✓ | ✓ |
| Bookmark this page<br>`browser.bookmark` | Works in web browsers. | — | ✓ | ✓ |
| Zoom in<br>`view.zoom_in` | Browsers, editors and other apps that support this shortcut. | — | ✓ | ✓ |
| Zoom out<br>`view.zoom_out` | Browsers, editors and other apps that support this shortcut. | — | ✓ | ✓ |
| Reset zoom<br>`view.zoom_reset` | Browsers, editors and other apps that support this shortcut. | — | ✓ | ✓ |

### Edit

| Action / ID | What it does / where it works | Parameters | Windows | Mac |
|---|---|---|---|---|
| Scroll to top<br>`view.top` | Browsers, editors and other apps that support this shortcut. | — | ✓ | ✓ |
| Scroll to bottom<br>`view.bottom` | Browsers, editors and other apps that support this shortcut. | — | ✓ | ✓ |
| Page up<br>`view.page_up` | Browsers, editors and other apps that support this shortcut. | — | ✓ | ✓ |
| Page down<br>`view.page_down` | Browsers, editors and other apps that support this shortcut. | — | ✓ | ✓ |
| Copy<br>`edit.copy` | Browsers, editors and other apps that support this shortcut. | — | ✓ | ✓ |
| Cut<br>`edit.cut` | Browsers, editors and other apps that support this shortcut. | — | ✓ | ✓ |
| Paste<br>`edit.paste` | Browsers, editors and other apps that support this shortcut. | — | ✓ | ✓ |
| Paste &amp; Enter<br>`edit.paste_enter` | Browsers, editors and other apps that support this shortcut. | — | ✓ | ✓ |
| Paste as plain text<br>`edit.paste_plain` | Apps with plain-text paste; shortcuts vary by app. | — | ✓ | ✓ |
| Select all<br>`edit.select_all` | Browsers, editors and other apps that support this shortcut. | — | ✓ | ✓ |
| Undo<br>`edit.undo` | Browsers, editors and other apps that support this shortcut. | — | ✓ | ✓ |
| Redo<br>`edit.redo` | Browsers, editors and other apps that support this shortcut. | — | ✓ | ✓ |
| Save<br>`edit.save` | Browsers, editors and other apps that support this shortcut. | — | ✓ | ✓ |
| Enter<br>`key.enter` | Browsers, editors and other apps that support this shortcut. | — | ✓ | ✓ |
| Backspace<br>`key.backspace` | Browsers, editors and other apps that support this shortcut. | — | ✓ | ✓ |
| Delete<br>`key.delete` | Browsers, editors and other apps that support this shortcut. | — | ✓ | ✓ |
| Cancel (Esc)<br>`key.escape` | Browsers, editors and other apps that support this shortcut. | — | ✓ | ✓ |
| Fullscreen<br>`view.fullscreen` | Browsers, editors and other apps that support this shortcut. | — | ✓ | ✓ |

### Text

| Action / ID | What it does / where it works | Parameters | Windows | Mac |
|---|---|---|---|---|
| Insert text<br>`text.type` | Text fields in apps; pastes and restores the clipboard by default. | Text (`text`): required<br>Input method (`method`): Paste and restore clipboard (`paste`) / Type characters (`type`); default Paste and restore clipboard<br>Wait after paste (ms) (`paste_delay_ms`): default `150`; only when Input method is Paste and restore clipboard | ✓ | ✓ |
| Insert date and time<br>`text.datetime` | Text fields in apps; uses local date and time. | Date and time format (`format`): 2026-10-24 (`date_iso`) / 2026年10月24日 (`date_local`) / 10/24/2026 (`date_us`) / 14:30 (`time`) / 2026-10-24 14:30 (`datetime`) / Custom (`custom`); default 2026-10-24<br>Custom format (`custom_format`): default `YYYY-MM-DD`; only when Date and time format is Custom<br>Input method (`method`): Paste and restore clipboard (`paste`) / Type characters (`type`); default Paste and restore clipboard<br>Wait after paste (ms) (`paste_delay_ms`): default `150`; only when Input method is Paste and restore clipboard | ✓ | ✓ |
| Web search<br>`text.search` | Uses the default browser; first select text in the current app. | Search engine (`engine`): Bing (`bing`) / Google (`google`) / Baidu (`baidu`) / DuckDuckGo (`duckduckgo`) / Custom (`custom`); Baidu for Simplified Chinese UI; Google otherwise<br>Custom URL (`url`): default `https://www.google.com/search?q=%MS_SELECTED_TEXT%`; only when Search engine is Custom | ✓ | ✓ |
| Translate selected text<br>`text.translate` | Uses the default browser; first select text in the current app. | Translation service (`service`): Google Translate (`google`) / DeepL (`deepl`) / Bing Translator (`bing`); default Google Translate<br>Target language (`target_language`): Follow UI language (`ui`) / 简体中文 (`zh-CN`) / English (`en-US`) / español (`es-ES`) / Français (`fr-FR`) / Deutsch (`de-DE`) / 日本語 (`ja-JP`) / Português (Brasil) (`pt-BR`) / Русский (`ru-RU`) / Italiano (`it-IT`) / 한국어 (`ko-KR`); default Follow UI language | ✓ | ✓ |
| Open selected URL<br>`text.open_selected_url` | Uses the default browser; first select text in the current app. | — | ✓ | ✓ |
| Copy selection as plain text<br>`text.copy_plain` | Apps that can copy text; removes clipboard formatting. | — | ✓ | ✓ |

### Window

| Action / ID | What it does / where it works | Parameters | Windows | Mac |
|---|---|---|---|---|
| Minimize<br>`window.minimize` | Applies to the gesture target window. | — | ✓ | ✓<br>Requires Accessibility permission. |
| Maximize / restore<br>`window.toggle_maximize` | Applies to the gesture target window. | — | ✓ | ✓<br>Requires Accessibility permission. |
| Close window<br>`window.close` | Applies to the gesture target window. | — | ✓ | ✓<br>Requires Accessibility permission. |
| Quit app<br>`app.quit` | Applies to the gesture target window. | — | ✓ | ✓ |
| Kill process<br>`app.kill` | Force-quits the target app; unsaved work will be lost. | — | ✓ | ✓ |
| Always on top<br>`window.toggle_topmost` | Toggle always on top for the gesture target window. | — | ✓ | This action is only supported on Windows. |
| Snap to left half<br>`window.snap_left` | Applies to the gesture target window. | — | ✓ | ✓<br>Requires Accessibility permission. |
| Snap to right half<br>`window.snap_right` | Applies to the gesture target window. | — | ✓ | ✓<br>Requires Accessibility permission. |
| Center window<br>`window.center` | Applies to the gesture target window. | — | ✓ | ✓<br>Requires Accessibility permission. |
| Move to next display<br>`window.next_monitor` | Applies to the gesture target window. | — | ✓ | ✓<br>Requires Accessibility permission. |
| Hide current app<br>`app.hide` | Hide the current application. | — | This action is only supported on macOS. | ✓ |

### Desktop & System

| Action / ID | What it does / where it works | Parameters | Windows | Mac |
|---|---|---|---|---|
| Switch app<br>`desktop.switch_app` | System action; some behavior depends on system shortcut settings. | — | ✓ | ✓ |
| Task View / Mission Control<br>`desktop.task_view` | System action; some behavior depends on system shortcut settings. | — | ✓ | ✓ |
| Show desktop<br>`desktop.show` | System action; some behavior depends on system shortcut settings. | — | ✓ | ✓ |
| Previous desktop<br>`desktop.prev` | System action; some behavior depends on system shortcut settings. | — | ✓ | ✓ |
| Next desktop<br>`desktop.next` | System action; some behavior depends on system shortcut settings. | — | ✓ | ✓ |
| Start menu / Spotlight<br>`desktop.launcher` | System action; some behavior depends on system shortcut settings. | — | ✓ | ✓ |
| Open file manager<br>`desktop.files` | System action; some behavior depends on system shortcut settings. | — | ✓ | ✓ |
| Snip<br>`capture.region` | System action; some behavior depends on system shortcut settings. | — | ✓ | ✓ |
| Capture and save screen<br>`capture.screen` | System action; some behavior depends on system shortcut settings. | — | ✓ | ✓ |
| Screenshot and recording tools<br>`capture.tool` | System action; some behavior depends on system shortcut settings. | — | ✓ | ✓ |
| Clipboard history<br>`desktop.clipboard_history` | Open system clipboard history. | — | ✓ | This action is only supported on Windows. |
| Emoji and symbols<br>`desktop.emoji` | System action; some behavior depends on system shortcut settings. | — | ✓ | ✓ |
| Notification center<br>`desktop.notifications` | Open Notification Center. | — | ✓ | This action is only supported on Windows. |
| Lock screen<br>`system.lock` | System action; some behavior depends on system shortcut settings. | — | ✓ | ✓ |
| Task Manager / Activity Monitor<br>`system.task_manager` | System action; some behavior depends on system shortcut settings. | — | ✓ | ✓ |
| Force Quit dialog<br>`system.force_quit_dialog` | Open the Force Quit Applications dialog. | — | This action is only supported on macOS. | ✓ |
| System settings<br>`system.settings` | System action; some behavior depends on system shortcut settings. | — | ✓ | ✓ |
| Calculator<br>`system.calculator` | System action; some behavior depends on system shortcut settings. | — | ✓ | ✓ |
| Turn off display<br>`system.display_off` | System action; some behavior depends on system shortcut settings. | — | ✓ | ✓ |
| Sleep<br>`system.sleep` | System action; some behavior depends on system shortcut settings. | — | ✓ | ✓ |

### Media

| Action / ID | What it does / where it works | Parameters | Windows | Mac |
|---|---|---|---|---|
| Play / pause<br>`media.play_pause` | System media keys; requires player support. | — | ✓ | ✓ |
| Previous track<br>`media.previous` | System media keys; requires player support. | — | ✓ | ✓ |
| Next track<br>`media.next` | System media keys; requires player support. | — | ✓ | ✓ |
| Volume up<br>`media.volume_up` | System media keys; requires player support. | — | ✓ | ✓ |
| Volume down<br>`media.volume_down` | System media keys; requires player support. | — | ✓ | ✓ |
| Mute<br>`media.mute` | System media keys; requires player support. | — | ✓ | ✓ |
| Brighter<br>`display.brightness_up` | System brightness control; depends on display and driver support. | — | ✓ | ✓ |
| Darker<br>`display.brightness_down` | System brightness control; depends on display and driver support. | — | ✓ | ✓ |

### Open

| Action / ID | What it does / where it works | Parameters | Windows | Mac |
|---|---|---|---|---|
| Open website<br>`open.website` | Opens the specified URL in the default browser. | URL (`url`): default `https://mousetroke.com/` | ✓ | ✓ |
| Open folder<br>`open.folder` | Opens the specified path with the system default app. | Path (`path`): required | ✓ | ✓ |
| Open file<br>`open.file` | Opens the specified path with the system default app. | Path (`path`): required | ✓ | ✓ |
| Open or switch to app<br>`open.app` | Opens the app, or activates its existing window. | App (`app`): required | ✓ | ✓ |

### Mousetroke

| Action / ID | What it does / where it works | Parameters | Windows | Mac |
|---|---|---|---|---|
| Pause / resume gestures<br>`mousetroke.toggle_pause` | Pauses or resumes gesture recognition in Mousetroke. | — | ✓ | ✓ |
| Open settings<br>`mousetroke.settings` | Opens Mousetroke settings. | — | ✓ | ✓ |
| Show gesture list<br>`mousetroke.cheatsheet` | Lists gestures for the current app; planned for phase 2. | — | This action is planned for phase 2. | This action is planned for phase 2. |

<!-- action-library:generated:end -->

To refresh all ten catalogues after changing the registry or translations, set `UPDATE_ACTION_DOCS=1` and run `cargo test -p mousetroke-core --test action_docs`. Run the same command without that variable to check that the committed tables are current.
