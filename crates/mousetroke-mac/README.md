# Mousetroke macOS M2

Apple Silicon, macOS 11+. Install Xcode Command Line Tools and Rust. From the repository root:

```bash
export CARGO_BUILD_JOBS=4 CARGO_TARGET_DIR="$PWD/target" MACOSX_DEPLOYMENT_TARGET=11.0
cargo check -p mousetroke-mac -p mousetroke --locked
cargo clippy -p mousetroke-mac --all-targets -- -D warnings
cargo test -p mousetroke-core -p mousetroke-engine -p mousetroke-mac --locked
bash scripts/mac/make-dev-cert.sh
bash scripts/mac/bundle.sh
open target/mac/Mousetroke.app
```

The certificate script creates a **Mousetroke Dev** signing identity in a dedicated keychain. Follow its trust instructions and keep the identity and app path stable across rebuilds. Set `MOUSETROKE_SIGN_ID` for another identity. Without a matching identity, bundling warns and uses ad-hoc signing; rebuilding can then require Accessibility permission again. This development bundle is not notarized.

Grant **Accessibility** and **Input Monitoring** to `Mousetroke.app` in System Settings → Privacy & Security. The menu links to missing permissions. Permissions are checked every two seconds and the tap starts without restarting. Prompts are requested only on first launch, tracked by `permission-prompted` in Application Support. SSH tests require an active graphical login for the same user. Use the signed bundle for repeatable permission tests.

The menu icon dims while paused or awaiting permission. Toggle pause from the menu or with **⌃⌥⌘P**. Ordinary right clicks are replayed. Right-button drags recognize shared-engine gestures and send configured keys to the focused application. Frontmost-app metadata is sampled every 250 ms; window IDs remain zero. Fullscreen detection, target activation and automatic updates remain for later milestones.

## Overlay

Each `NSScreen` has a borderless, non-opaque, transparent `NSWindow` that ignores mouse events and cannot become key or main. The public `NSScreenSaverWindowLevel` (1000) is above the menu bar (24). `canJoinAllSpaces | fullScreenAuxiliary | stationary | ignoresCycle` accompanies full-screen applications without changing focus. Windows are ordered out while idle, then closed and released when the canvas is replaced or the app exits. System-protected surfaces and the login screen are outside the intended scope.

A layer-hosting content view owns one `CAShapeLayer` and a rounded translucent pill containing a `CATextLayer`. Trail color/width follow config, using Windows-compatible `#RRGGBBAA`, round caps/joins, 0.55 smoothing and quadratic midpoint curves. The tap appends at most 8192 segments per session; the existing 10 ms common-mode timer coalesces path submissions. No per-move CPU bitmap redraw occurs. All layer properties change under `CATransaction.setDisableActions(true)`.

Engine coordinates are Quartz global **points**, top-left origin. AppKit frames use the height of `NSScreen.screens[0]`, even when another screen is main: `cgY = primaryHeight - appkitY - height`. Layer coordinates are `(cgX - screenX, screenHeight - (cgY - screenY))`. Retina changes `contentsScale`, never point coordinates. `virtual_rect` is the union of screen frames. Labels use the containing screen's visible-frame horizontal center and 80% height from the top.

Trail and label fade together through an explicit 200 ms opacity animation; the timer then clears and orders out the window. A new session immediately removes old animation and path. No delayed completion can clear a newer session. A later toast also clears the fading trail. `NSApplicationDidChangeScreenParametersNotification` schedules `Engine::on_display_change()` on the main timer, including backing-scale changes; two-second screen snapshots remain a fallback.

## Send keys

`Host::execute` only queues. The next main-run-loop timer calls `mousetroke_core::keyseq::parse(&keys, false)`, converts VK through `vk_to_kvk`, and posts `CGEventCreateKeyboardEvent` events at the HID tap. Every event carries `INJECT_TAG`, so our session tap ignores its echo.

| Syntax | macOS modifier | kVK | CG flag |
| --- | --- | --- | --- |
| `^` Ctrl | Control ⌃ | `0x3b` | `MaskControl` |
| `+` Shift | Shift ⇧ | `0x38` | `MaskShift` |
| `%` Alt | Option ⌥ | `0x3a` | `MaskAlternate` |
| `#` Win | Command ⌘ | `0x37` | `MaskCommand` |

Chords release modifiers in reverse order, with matching flags. A whole chord is allocated before posting. Repeats are lazy, `{SLEEP ms}` uses a deadline, and each timer tick runs at most 32 executor steps. Parser `Text` items use `CGEventKeyboardSetUnicodeString`, one complete Unicode scalar (including surrogate pairs) per event pair. The existing `parse(..., false)` grammar produces US-layout presses for literal characters and rejects non-ASCII literals; the shared grammar is unchanged. Mouse items and unmapped keys are skipped with one content-free log per sequence. Non-`down_up` modes are skipped. `activate_target` is ignored with one log per process. Other action types log unsupported until M3.

## macOS defaults

`Config::builtin()` selects `assets/default-config-macos.json` on macOS. All 47 general-scope gestures retain their patterns, names and translation keys; explorer/cmd scopes are omitted. Existing saved configurations, including M1 configs, are preserved. Self-test uses new defaults in memory. For interactive testing, quit the app, back up `~/Library/Application Support/mousetroke/config.json`, then copy `crates/mousetroke-core/assets/default-config-macos.json` there before restarting.

| Gestures | macOS decision |
| --- | --- |
| Back / forward | ⌘[ / ⌘] |
| New / reopen / close tab, refresh | ⌘T / ⌘⇧T / ⌘W / ⌘R |
| Previous / next tab (both patterns) | ⌃⇧Tab / ⌃Tab |
| Top / bottom | ⌘↑ / ⌘↓ |
| Copy / paste / cut / paste + Enter | ⌘C / ⌘V / ⌘X / ⌘V then Return |
| Enter / backspace / forward delete / Esc | Return / Delete / Forward Delete / Esc |
| Screenshot / minimize / full screen / close window | ⌘⇧4 / ⌘M / ⌃⌘F / ⌘⇧W (browser window shortcut) |
| Previous / next desktop | ⌃← / ⌃→ |
| Start / show desktop / switch app / task switcher | Spotlight ⌘Space / F11 / ⌘Tab / Mission Control ⌃↑ |
| Zoom in / out / reset | ⌘= / ⌘− / ⌘0 |
| Maximize / topmost / force kill / web search | Retained, disabled pending M3; kill command uses `/bin/kill` |
| Media, volume, brightness (8 gestures) | Retained, disabled pending M3 |
| Task Manager / system settings | Activity Monitor / System Settings app paths, disabled pending M3 |

System and application shortcuts can be reassigned. F11, Spaces and Mission Control depend on Keyboard Shortcuts settings. Maximize/restore is not replaced by full screen. macOS has no public API for keeping another application's window always on top. Labels reuse all ten existing translation catalogs.

## Self-test and real-Mac verification

Quit the running instance. With the same graphical user logged in, run the signed executable directly over SSH:

```bash
MOUSETROKE_SELFTEST=1 target/mac/Mousetroke.app/Contents/MacOS/mousetroke
result=$?
tail -n 80 "$HOME/Library/Logs/mousetroke/mousetroke.log"
test "$result" -eq 0
```

Self-test preserves existing saved config. Once its warm-up move reaches the tap, it posts **untagged** right-button ↓ then → drags (200 points per leg, 30 steps per leg, at least 10 ms apart), then a plain right click. Avoid physical mouse input. After one second it requires exactly one expected close-tab match, exactly one tagged replay down/up pair, positive real-overlay `start_session`/`line_to` counters and exactly one expected KeySequence dispatch in dry-run mode. The executor logs what it would post and **never posts ⌘W or other action keys**. The cursor is restored. Exit **0** means all checks passed; **1** means failure or a 120-second timeout. Overlay counters verify calls, not WindowServer pixels.

Then start normally with the macOS defaults and check:

1. In a browser with history, draw ←/→; verify back/forward, trail, translated label and unchanged focus. Try ↓→ on a disposable tab; it must close that tab.
2. Start another gesture within 200 ms of release, repeatedly. No old trail should remain or erase the new trail. Try an unrecognized gesture and verify its toast disappears.
3. Repeat on Retina/1x screens, screens left/above/below primary, across screen boundaries and in a full-screen browser. Verify visible-frame label anchors, crisp text and configured colors/width.
4. While idle and drawing, connect/disconnect a display, change resolution/scaling or rearrange screens. The old canvas must close, capture must cancel, and the next gesture must use the new layout.
5. Test copy/paste, tab navigation, Option/Control/Command chords and `#c{SLEEP 500}#v` in a disposable document. Confirm no recursion or stuck modifiers; clicks and pause should respond during delays.

Config: `~/Library/Application Support/mousetroke/config.json`. Log: `~/Library/Logs/mousetroke/mousetroke.log`, rotated at 1 MiB to `.log.old`. `MOUSETROKE_SHAPE_SAMPLES=1` enables normalized shape samples. A nonblocking exclusive `flock` prevents duplicate instances.

AppKit, engine, tap, overlay, timers, menus and executor all run on the main thread. Replay and action injection occur after the tap returns. Logs use a bounded background queue; callback boundaries catch Rust panics. Windows config and CLI behavior are unchanged. Pure coordinate/color tests also run on Windows.
