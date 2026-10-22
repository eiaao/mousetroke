# Mousetroke

[![CI](https://github.com/eiaao/mousetroke/actions/workflows/ci.yml/badge.svg)](https://github.com/eiaao/mousetroke/actions/workflows/ci.yml) [![Release](https://img.shields.io/github/v/release/eiaao/mousetroke?include_prereleases)](https://github.com/eiaao/mousetroke/releases/latest) [![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE) ![Windows 10/11](https://img.shields.io/badge/platform-Windows%2010%20%7C%2011-0098d9)

**Free, open-source mouse gestures for Windows and macOS (beta).** Hold the right button and draw — `←` goes
back, `↓→` closes the tab, `↙` minimizes the window. Push the cursor into a screen corner for
the Start menu, or scroll along the top edge to change the volume. A plain right-click still
opens the context menu.

**[Website](https://mousetroke.pages.dev/) · [Download](https://github.com/eiaao/mousetroke/releases/latest) · [简体中文](README.zh-CN.md)**

Documentation / 文档: [User guide](docs/guide/en/index.md) · [使用指南](docs/guide/zh-CN/index.md)

![Mousetroke settings](docs/images/settings-en.png)

- About 50 built-in gestures, all editable; record your own by drawing it once
- Screen corners and edges: Start menu, show desktop, volume, brightness (DDC/CI for external monitors)
- Eight action types: send keys, window actions, open an app, open a URL, run a command,
  media & system keys, popup menu, Lua script
- Per-app rules, or turn gestures off entirely in remote-desktop / 3D / design apps
- Pauses automatically for fullscreen games, video and presentations; `Win+Esc` pauses any time
- About 19 MB of memory; a single portable exe (≈13 MB); 10 interface languages

## Install

1. Download `mousetroke.exe` from the [latest release](https://github.com/eiaao/mousetroke/releases/latest) and check its
   SHA-256 against `SHA256SUMS.txt` on the same page.
2. Double-click it. An icon appears in the notification area: left-click for settings,
   right-click to pause or quit. A short guide runs the first time.

Requirements: Windows 10 1809+ or Windows 11, 64-bit. The settings window uses Microsoft Edge
WebView2 (built into Windows 11). The exe is not code-signed yet, so SmartScreen may warn on
first run — choose **More info → Run anyway**.

Uninstall: turn off *Start with Windows* in preferences, quit from the tray menu, then delete
`mousetroke.exe` and `%APPDATA%\mousetroke`.

## Build from source

Needs Rust (stable, MSVC toolchain), Node.js 20.19+ and Python 3, on Windows.

```bat
scripts\build.cmd            :: UI + release exe → dist\mousetroke.exe
cargo test --workspace       :: engine, importer, i18n and settings tests
```

| Path | What it is |
|---|---|
| `crates/mousetroke-core` | Platform-independent logic: config schema, recognizer, matcher, key-sequence syntax, built-in defaults, legacy-format importer |
| `crates/mousetroke-win` | Windows layer: low-level hooks, capture state machine, trail overlay, input injection, tray |
| `crates/mousetroke-settings` | Settings window (Tauri 2 + React); runs as a separate process only while open |
| `crates/mousetroke` | The `mousetroke.exe` entry point and CLI |
| `i18n/` | Shared UI strings for every language (see `i18n/README.md`) |

The [website](https://mousetroke.pages.dev/) is maintained and deployed from a separate repository.
Its changelog reads `CHANGELOG.md` from this repository's `main` branch on raw.githubusercontent.com.

## macOS beta

[Download Mac beta](https://github.com/eiaao/mousetroke/releases/latest/download/Mousetroke.dmg):
macOS 11+, Apple silicon and Intel in one universal app. Compare its SHA-256 with the
same release's `SHA256SUMS.txt` (which also lists the Windows exe).

1. Open the DMG, drag **Mousetroke** into **Applications**, then open it. If opened from the
   DMG, choose **Move to Applications** to install and reopen it there (or in `~/Applications`
   if the shared folder is not writable). Quit an older copy before replacing it.
2. The beta is self-signed and **not notarized**. If macOS blocks it, use **System Settings →
   Privacy & Security → Open Anyway**; on older macOS, **right-click → Open**. A newly
   browser-downloaded copy may need this step again. In-app updates avoid the repeated prompt.
3. Follow setup and click **Grant…** to allow **Accessibility**. This lets Mousetroke see
   mouse gestures, control windows and send keyboard shortcuts. Setup shows **Ready** once
   the engine’s event taps are active, then a short guide.
   Try holding the right button and drawing `←` in a window. You can skip and grant later
   from the menu bar. **Screen Recording is not required**.

The engine checks permissions with a fresh, silent probe every two seconds, including while
gestures work, so revoking access is detected too. If Accessibility is enabled but listening
or event taps still fail after a few seconds, setup and the menu bar offer **Input Monitoring**
as a fallback for older macOS versions. Enable it only when asked. Setup can restart the engine
once automatically and offers **Restart Mousetroke** if needed, keeping settings open.

About supports manual update checks, release notes and **Update now**: the app downloads the
DMG, verifies SHA-256, bundle identity, version and the current certificate's designated
requirement, then replaces the running bundle and restarts. The old app goes to Trash after
the engine and settings exit. Failed replacement or restart handoff restores the old bundle.
In-app downloads have no browser quarantine, so updates avoid a second Gatekeeper prompt.
The signing identity and installation path stay the same to retain permissions.
**Download manually** remains available if the folder is not writable or updating fails;
browser downloads may require Open Anyway again. Preferences controls daily checking,
and a new-version menu item opens About. See the [Mac update acceptance recipe](crates/mousetroke-mac/README.md#in-app-update-acceptance-on-the-mac-mini).

Uninstall: quit from the menu bar, delete `Mousetroke.app`,
`~/Library/Application Support/mousetroke` and `~/Library/Logs/mousetroke`, then remove
Mousetroke from login items in System Settings.

See the [macOS guide](crates/mousetroke-mac/README.md) for build/sign/DMG commands,
release secrets, supported actions and the real-Mac acceptance checklist.

## Security

Shell and Lua actions run with your user rights — only import configurations you trust.
See [SECURITY.md](SECURITY.md) for what the program does and how to report a vulnerability.

## Contributing

Issues, translations and pull requests are welcome — see [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE)
