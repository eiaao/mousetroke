---
title: "Configuration, backup and updates"
description: "Find Mousetroke config and logs, export and restore settings, reload changes, update the app or uninstall it."
order: 11
---

# Configuration, backup and updates

## File locations

| File | Windows | Mac |
|---|---|---|
| Configuration | `%APPDATA%\mousetroke\config.json` | `~/Library/Application Support/mousetroke/config.json` |
| Runtime log | `%APPDATA%\mousetroke\mousetroke.log` | `~/Library/Logs/mousetroke/mousetroke.log` |

Logs rotate to `mousetroke.log.old` after exceeding 1 MB. **About → File locations** displays the actual paths. **Open folder** opens the config directory; **Open log** opens the runtime log. Under **Import & export**, **Show in File Explorer** on Windows or **Show in Finder** on Mac reveals the configuration.

## Export, import and restore

Under **Import & export → Mousetroke config file**, click **Export…** and save a JSON backup. **Import from file…** reads a Mousetroke config JSON and replaces the current configuration, including preferences. It does not merge that file into your current bindings. There is no native-config merge option in this workflow. You can undo the import with Ctrl+Z on Windows or ⌘+Z on Mac while settings is open.

Before editing JSON manually, export a backup. For a direct restore, quit the engine and close settings, replace `config.json` with the backup, then reopen Mousetroke. Keeping settings closed avoids its autosave overwriting the restored file.

## Reload and reset

The engine watches the configuration's file timestamp. Windows checks about once per second; Mac checks in the maintenance branch scheduled every two seconds. A change received during a gesture is applied after capture ends. Invalid JSON leaves the previous active configuration in use and records an error in the log.

The current settings UI has no reset-to-defaults button. To start with built-in defaults, quit Mousetroke and close settings, back up and remove or rename `config.json`, then reopen. A missing file is initialized with platform-specific defaults. A malformed file is not a request to reset.

## Updates

**Preferences → Check for updates automatically** is enabled by default and checks GitHub once a day. Turn it off there to stop automatic checks. Use **About → Check for updates** for a manual check and **What’s new** for release notes. A new-version menu item also opens About.

On Windows, **Update now** downloads the executable, verifies its SHA-256, replaces the running copy and restarts. **Open download page** is available for manual installation.

On Mac, **Update now** downloads the DMG, checks SHA-256, bundle identity, version and the signing requirement, replaces the app and restarts. The old bundle goes to Trash after shutdown; failed replacement or restart handoff restores it. **Download manually** is available if the installation folder is not writable or the update fails. In-app updates avoid another browser-quarantine prompt and preserve the installation identity used for permissions. A fresh browser download may need **Open Anyway** again.

## Uninstall

Windows: turn off **Enable gestures automatically at startup** in Preferences, **Quit** from the tray, close settings, then delete the executable and `%APPDATA%\mousetroke` if you no longer need settings or logs.

Mac: **Quit** from the menu bar, close settings and delete `Mousetroke.app`. To remove data too, delete `~/Library/Application Support/mousetroke` and `~/Library/Logs/mousetroke`. Remove Mousetroke from login items in System Settings. Keep an exported backup if you may reinstall.

See [troubleshooting](troubleshooting.md) for configuration errors and missing logs.
