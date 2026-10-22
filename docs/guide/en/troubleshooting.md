---
title: "Troubleshooting"
description: "Fix mouse gestures that do not trigger, macOS permissions, missing shortcuts, IME text input and Lua errors."
order: 12
---

# Troubleshooting

## Gestures do not trigger

First test a simple built-in gesture in a normal, non-fullscreen browser window.

- Check the tray/menu-bar menu. On Windows select **Resume gestures** if offered; on Mac click **Pause gestures** again if it is checked. Check **Preferences → Pause / resume hotkey** if a shortcut paused the engine.
- Leave fullscreen. Automatic fullscreen suppression can stop both mouse gestures and edge actions even when manual pause is off.
- Select the app in settings and check **Turn off gestures in this app**. Also check the individual binding's **On** / **Off** switch, **Also use global gestures**, the trigger button and any extra steps.
- Move promptly after pressing the button and draw a longer stroke. Tune **Hold-still time before treating it as a click** and **Stroke sensitivity** under Preferences.
- If feedback is missing, check **Show the trail** and **Show the gesture name while drawing**. A hidden trail does not itself mean recognition is off.

On Windows, input uses `SendInput`. A normally launched instance may be unable to send keys/clicks into an elevated app because of Windows UIPI. Test against a non-elevated app first; if you need to control an administrator window, run Mousetroke at the corresponding elevation. This does not enable sending to the secure desktop.

On Mac, check **Accessibility** for the installed copy. If setup still asks for **Input Monitoring**, grant it too. Permission changes can take a few seconds to be detected. Use **Restart Mousetroke** when offered if input still does not start. Screen Recording is not required. Moving or replacing a copy through the browser can require first-open approval again.

## SmartScreen or Gatekeeper blocks first open

Windows: get the official executable from the release page, verify the checksum, then use **More info → Run anyway**. Mac: install in Applications, then use **System Settings → Privacy & Security → Open Anyway**; older macOS also supports **right-click → Open**. See [installation](getting-started.md) for the complete sequence.

## An application does not receive keys

Try the shortcut on the physical keyboard. Check whether the application is active, or enable **Activate the window under the cursor before sending**. Confirm the gesture target preference and use [correct syntax](key-sequences.md): `^` is Control, while `#` is Command on Mac. Separate dependent steps with a short `{SLEEP 100}` if the app needs time to open a menu or dialog.

Mac key sequences skip mouse/wheel tokens, media/volume key tokens, Insert, Print Screen, Break and F21–F24. Use **Media & system** for media/volume actions. Unsupported options are marked in the editor; inspect the log too.

Ctrl+Alt+Del is a Windows Secure Attention Sequence and cannot be sent as ordinary synthetic input. Use Ctrl+Shift+Esc for Task Manager, Win+L for locking, or `shutdown /l` to sign out. Details are in [send-keys syntax](key-sequences.md#why-ctrlaltdel-cannot-be-sent).

## IME or text output is wrong

Ordinary send-keys characters represent US keyboard positions, not arbitrary Unicode text. Capital letters alone do not add Shift. Lua `ms.keys(text, true)` emits Unicode text but still interprets syntax characters such as `^` and `{`.

For punctuation-heavy text, non-Latin text or apps with unusual IME handling, use the [clipboard paste-and-restore recipe](lua-examples.md#paste-and-restore-clipboard-text). Restoring with these APIs preserves text only, not images or other clipboard formats. Some apps need a longer delay before clipboard restoration.

## Lua errors

Make sure the code defines `function ms_action(...) ... end`. The runtime writes load and execution errors to the [runtime log](config.md#file-locations). Open it through **About → Open log**. Use `ms.log` for values and `ms.show_toast` for a brief on-screen checkpoint. An unsupported Mac function may only log a warning and return `nil`; do not treat a missing value as success.

The Lua allocation limit is 8 MiB. There is no script execution timeout; avoid infinite loops and see [Lua basics](lua-basics.md#runtime-and-limits) for blocking behavior.

## Remote desktop and games

Remote desktop, virtual machines and 3D/game applications may need the same mouse buttons for dragging or may handle synthetic input differently. Add a [per-app exclusion](per-app.md) or pause while using them. Windowed games may need an explicit exclusion because fullscreen detection does not apply. Test shortcuts in an ordinary local app to separate a key-sequence problem from application input handling.

## Report a bug

Use **About → Feedback → Open** or [GitHub Issues](https://github.com/eiaao/mousetroke/issues). Include the app version, OS version, affected application, gesture and action, expected result and steps to reproduce. Attach the relevant log from **About → Open log** and a minimal exported configuration if useful. Review attachments for private text, window titles, paths or command arguments before sharing them.
