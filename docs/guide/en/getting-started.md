---
title: "Getting started"
description: "Install Mousetroke on Windows or Mac, draw your first mouse gesture, open settings and pause gestures."
order: 2
---

# Getting started

## Install and open

On Windows, download `mousetroke.exe` from the [latest release](https://github.com/eiaao/mousetroke/releases/latest). It is a portable executable: save it somewhere you want to keep it and double-click it. Compare the download's SHA-256 with `SHA256SUMS.txt` on that release page.

Windows 10 1809 or later and Windows 11, 64-bit, are supported. Settings uses Microsoft Edge WebView2, included in Windows 11. The executable is not code-signed yet. If SmartScreen blocks the first launch, choose **More info → Run anyway** after checking that you downloaded the official release.

On Mac, download `Mousetroke.dmg`, open it and drag **Mousetroke** into **Applications**. The universal app supports Apple silicon and Intel on macOS 11 or later. If opened inside the DMG, the app offers **Move to Applications** and reopens the installed copy.

The Mac beta is self-signed and not notarized. If blocked, use **System Settings → Privacy & Security → Open Anyway**; on older macOS, use **right-click → Open**. In setup, click **Grant…** and enable Mousetroke under **Accessibility**. Wait for **Ready — you can use gestures now**. Screen Recording permission is not required. Setup offers **Input Monitoring** if listening still fails on systems that need it; enable it when requested.

## Open settings and finish the guide

On Windows, left-click the notification-area icon to open settings. Right-click it for **Settings…**, **Pause gestures** / **Resume gestures**, **Start automatically**, **Open config folder** and **Quit**. A new-version item can also appear when an update is available.

On Mac, use the menu-bar icon to open **Settings…**, pause/resume or **Quit**. Permission and update items appear when relevant. **About Mousetroke** opens About.

The first-run guide explains drawing, screen edges, app exclusions and pause. Review the checked apps before finishing: those apps will have gestures turned off. You can reopen it with **About → View the getting-started guide**. Changes in settings save automatically; an incomplete stroke must be recorded before it can save.

## Draw your first gesture

1. Open a browser page with navigation history.
2. Hold the right mouse button, move left to draw `←`, then release. This goes back.
3. Try `↓→` on a disposable tab to close it, or `↙` to minimize the window.

The built-in actions follow the platform: browser shortcuts use Ctrl on Windows and Command on Mac where appropriate. The full editable list is under **All apps → Mouse gestures**. Some built-ins are disabled, especially actions unavailable on Mac.

A plain right-click still opens the application's context menu. Holding still long enough also passes the button through for normal clicking or dragging; adjust that time in [Preferences](gestures.md#recognition-and-appearance).

## Create a gesture

Open **All apps → Mouse gestures → New gesture**, choose the trigger and record the drawing. Click **Choose action** to search or browse the [action library](action-library.md), select a preset and fill its parameters. Changes save automatically once the gesture is valid; try it outside settings. See [Mouse gestures](gestures.md) for recording details.

## Pause

On Windows choose **Pause gestures**, then **Resume gestures** to restore them. On Mac, **Pause gestures** stays the same label: its checkmark indicates manual pause, and clicking it again resumes. The default pause/resume hotkey is **Win+Esc** on Windows and **Control+Option+Command+P** on Mac. Change it under **Preferences → Pause / resume hotkey**.

Fullscreen games, videos and presentations automatically suppress gestures while the foreground window is detected as fullscreen. See [per-app rules](per-app.md) for exclusions and [troubleshooting](troubleshooting.md) if nothing triggers.
