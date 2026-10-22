---
title: "Per-app rules"
description: "Add apps, override global mouse gestures, exclude apps that need mouse dragging, and understand fullscreen auto-pause."
order: 6
---

# Per-app rules

## Add an application

Click **Add app…** in the sidebar. Choose **Set app-specific gestures** or **Turn off gestures in it**, then:

- Select an entry under **Running**.
- Click **Pick a window**, then click the application's window; Esc or right-click cancels.
- Use **Browse…** to choose the application.
- Drop an application or its shortcut into the settings window. Mac accepts `.app` bundles and executable files; its scopes use bundle IDs when available.

Windows rules can match an executable name or full path, case-insensitively. Mac app identities usually look like `com.apple.Safari`. Use the picker rather than guessing the identity. The app scope appears in the sidebar and can have its own mouse gestures and screen edges.

## App-specific and global gestures

**All apps** contains global gestures. An app-specific gesture takes priority over the corresponding global gesture. With **Also use global gestures** checked, global bindings remain available when the app's bindings do not match. Uncheck it to use only that app's gestures.

For example, add a browser, create a right-button `↓→` gesture and assign a different action there. Other apps keep the global `↓→` action. To copy an existing setup, use [Duplicate](gestures.md#find-copy-and-turn-off) within its scope and edit the binding.

## Turn gestures off in an app

Select the app in the sidebar and enable **Turn off gestures in this app**. This disables its own gestures, global gestures and screen-edge triggers. Right, middle and side buttons pass through normally. The stored bindings remain; **Turn gestures back on** restores them. **Remove this app** removes the app-specific scope.

This is useful for remote desktop, virtual machines, 3D modeling, design tools and game editors that use right/middle-button dragging. The picker’s **Suggested to turn off** list includes known applications from these categories. Detected installed apps can be added together.

## Fullscreen and first run

Fullscreen auto-pause is automatic: the engine avoids starting gestures and edge triggers while the foreground app is detected as fullscreen. It covers fullscreen games, video and presentations; it is separate from the manual paused state. On Windows, detection checks a borderless window covering its monitor and excludes desktop/shell windows. Mac checks the foreground app's window against screen bounds. A fullscreen-looking app may therefore behave differently on the two platforms.

For a windowed game or remote desktop, add an explicit exclusion. To pause everywhere, use the [tray/menu-bar command or pause hotkey](getting-started.md#pause).

The first-run guide shows detected apps that commonly need their own mouse dragging. Checked entries are disabled when you finish. Review them before continuing. Mac's built-in configuration also includes disabled scopes for its suggested exclusion apps; these have no app-specific bindings. You can change exclusions later from the sidebar.

Next: [actions](actions.md), [configuration backup](config.md), [troubleshooting](troubleshooting.md).
