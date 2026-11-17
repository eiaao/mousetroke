---
title: "Per-app rules"
description: "Add apps, override global gestures, exclude apps, and configure global fullscreen pause with per-app exceptions."
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

**All apps** contains global gestures. An app-specific gesture takes priority over the corresponding global gesture. With **Also use global gestures** turned on, global bindings remain available when the app's bindings do not match. Turn it off to use only that app's gestures.

For example, add a browser, create a right-button `↓→` gesture and assign a different action there. Other apps keep the global `↓→` action. To copy an existing setup, use [Duplicate](gestures.md#find-copy-and-turn-off) within its scope and edit the binding. To record a drawing once and give it a different action in each app, use the [gesture library](gesture-library.md).

The gesture editor shows where the binding is stored. Right-click the gesture in the list and choose **Copy to / Move to…**, select destination apps, then click **Copy** or **Move**. You can target several added apps at once or add an app from the dialog. Copies have independent settings; moving removes the binding from its original scope. If a destination already has the same drawing, choose whether to skip or replace it. The whole operation can be undone from the toast. A binding copied into an app with gestures turned off is retained but will not run until gestures are enabled there.

## Turn gestures off in an app

Select the app in the sidebar and turn off **Use gestures**. This disables its own gestures, global gestures and screen-edge triggers. Right, middle and side buttons pass through normally. The stored bindings remain; turning **Use gestures** back on restores them. The trash-can button **Remove this app** at the top right of the app page removes the app-specific scope.

This is useful for remote desktop, virtual machines, 3D modeling, design tools and game editors that use right/middle-button dragging. The picker’s **Suggested to turn off** list includes known applications from these categories. Detected installed apps can be added together.

## Fullscreen and first run

By default, fullscreen auto-pause is enabled: the engine avoids starting gestures and edge triggers while the foreground app is detected as fullscreen. It covers fullscreen games, video and presentations; it is separate from the manual paused state. On Windows, detection checks a borderless window covering its monitor and excludes desktop/shell windows. Mac checks the foreground app's window against screen bounds. A fullscreen-looking app may therefore behave differently on the two platforms.

Turn off “Pause gestures in fullscreen” in Preferences to change the global default. Each app offers “Follow global”, “Keep enabled” and “Pause gestures” under “In fullscreen”; an explicit app choice takes priority. To keep global gestures working in fullscreen Chrome, add Chrome, keep global gesture inheritance enabled and choose “Keep enabled”. No app-specific gestures are needed. Manual pause and app exclusions still apply.

For a windowed game or remote desktop, add an explicit exclusion. To pause everywhere, use the [tray/menu-bar command or pause hotkey](getting-started.md#pause).

The first-run guide shows detected apps that commonly need their own mouse dragging. Checked entries are disabled when you finish. Review them before continuing. Mac's built-in configuration also includes disabled scopes for its suggested exclusion apps; these have no app-specific bindings. You can change exclusions later from the sidebar.

Next: [gesture library](gesture-library.md), [actions](actions.md), [configuration backup](config.md), [troubleshooting](troubleshooting.md).
