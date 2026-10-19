# Mousetroke macOS M5a

Universal Apple silicon and Intel, macOS 11+. Install Xcode Command Line Tools and Rust. From the repository root:

```bash
export CARGO_BUILD_JOBS=4 CARGO_TARGET_DIR="$PWD/target" MACOSX_DEPLOYMENT_TARGET=11.0
npm --prefix crates/mousetroke-settings/ui ci
npm --prefix crates/mousetroke-settings/ui run build
cargo check -p mousetroke-settings -p mousetroke-mac -p mousetroke --locked
cargo clippy -p mousetroke-mac --all-targets -- -D warnings
cargo test -p mousetroke-core -p mousetroke-engine -p mousetroke-lua -p mousetroke-mac --locked
bash scripts/mac/make-dev-cert.sh
bash scripts/mac/bundle.sh
open target/mac/Mousetroke.app
```

The certificate script creates a **Mousetroke Dev** signing identity in a dedicated keychain. Follow its trust instructions and keep the identity and app path stable across rebuilds. Set `MOUSETROKE_SIGN_ID` for another identity. Without a matching identity, bundling warns and uses ad-hoc signing; rebuilding can then require Accessibility permission again. This development bundle is not notarized.

**Screen Recording is not required**: titles are obtained through Accessibility, never `kCGWindowName`. Grant **Accessibility** and **Input Monitoring** to `Mousetroke.app` in System Settings → Privacy & Security. The menu links to missing permissions. Permissions are checked every two seconds and the tap starts without restarting. Prompts are requested only on first launch, tracked by `permission-prompted` in Application Support. SSH tests require an active graphical login for the same user. Use the signed bundle for repeatable permission tests.

The menu icon dims while paused or awaiting permission. Toggle pause from the menu or with **⌃⌥⌘P**. Ordinary right clicks are replayed. Right-button drags recognize shared-engine gestures and send configured keys to the focused application. Window targeting and fullscreen pause use a roughly 250 ms snapshot. Target activation is supported. Settings run in a separate process; manual and daily update checks are supported; downloads open in the browser and installation is manual.

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

Chords release modifiers in reverse order, with matching flags. A whole chord is allocated before posting. Repeats are lazy, `{SLEEP ms}` uses a deadline, and each timer tick runs at most 32 executor steps. Parser `Text` items use `CGEventKeyboardSetUnicodeString`, one complete Unicode scalar (including surrogate pairs) per event pair. The existing `parse(..., false)` grammar produces US-layout presses for literal characters and rejects non-ASCII literals; the shared grammar is unchanged. Mouse items and unmapped keys are skipped with one content-free log per sequence. Non-`down_up` modes are skipped. `activate_target` activates the captured application and yields at least 30 ms before sending keys. Key sequences are serialized with the other action requests.

## Targeting, fullscreen pause and actions

A worker calls `CGWindowListCopyWindowInfo(OnScreenOnly | ExcludeDesktopElements)` in front-to-back order. The main timer resolves each owner PID through `NSRunningApplication` and caches targets by `(pid, CGWindowID)` for about 250 ms. A mouse-down performs only a layer-0 bounds hit-test in Quartz global points; the first covering window wins. Nothing under the cursor (desktop/menu area without a window) falls back to the frontmost application and its first layer-0 window. Own-app, Dock, SystemUIServer, Control Center and Notification Center bundle IDs pass through; visible shell surfaces above layer 0 are checked before underlying application windows. A slow `target_at` (>2 ms) is logged once.

`Target.exe` is the **bundle id**, such as `com.google.Chrome`, falling back to the full executable path for unbundled programs. App scopes use that same identity through the unchanged core matcher (case-insensitive). `Target.class` is the **bundle id**, such as `com.google.Chrome`; it filters system UI and is not a second scope matcher. `wid` is the CGWindowID, `pid` the owner PID. The current shared engine resolves scopes from `exe`, not title/class. Window titles are cached at `Host::window_title`; actions referencing `%MS_WIN_NAME%` lazily refresh them on an AX worker, matching AX windows by position and size. No Screen Recording prompt is needed. A moved/closed or inaccessible target is skipped rather than operating on an unrelated focused window.

The frontmost app's first layer-0 window is compared with each full screen frame (2-point tolerance). A matching fullscreen Space or presentation window pauses button and edge gestures. This follows the shared engine's existing automatic pause behavior; there is currently no separate `pause_fullscreen` setting. Detection may lag the snapshot by roughly one interval.

| Action | macOS support / limits |
| --- | --- |
| KeySequence | Supported: Command/Option/Control/Shift, tagged events, timer-based sleep/repeat; only `down_up`, mapped keyboard keys and supported shared grammar |
| WindowOp Minimize | AXMinimized = true on the captured window |
| WindowOp ToggleMaximize | AXZoomButton + AXPress; minimized windows or absent buttons log and skip |
| WindowOp Close | AXCloseButton + AXPress on the captured window; normal unsaved-document prompts still apply |
| WindowOp ToggleTopmost | Unsupported public cross-app API: translated short toast and log |
| LaunchApp | .app path, bundle id or absolute executable via asynchronous NSWorkspace; if LaunchServices rejects an unbundled executable, launch it on a worker; args use quoted/escaped word splitting without shell expansion |
| LaunchApp if_running | Activate reuses/activates an existing app; RunAgain requests a new instance (app may refuse); SendKeys is treated as Activate and logged; elevate is ignored and logged |
| OpenUrl | NSWorkspace, optional browser .app path/bundle id; selected text is URL-encoded, or opened directly if it is already a URL |
| Shell | Detached `/bin/zsh -c`; null stdin/stdout/stderr; asynchronous exit-status log without script content |
| SystemKey | Play/pause, next, previous, mute, volume up/down, built-in brightness up/down; unknown keys log and skip; no external DDC |
| Menu | Native NSMenu at the captured cursor position; recursive submenus; actions queued after selection; columns ignored |
| Lua | Shared Lua 5.4 runtime on a worker; macOS backend and limits below; self-test only logs and never starts a VM |

`%MS_EXE%`, `%MS_PID%`, `%MS_WID%`, `%MS_WIN_NAME%`, `%MS_TARGET_MODE%`, mouse coordinates and app basename retain shared-engine semantics. `%MS_SELECTED_TEXT%` first reads AXSelectedText from the system-wide focused element. If unsupported, it snapshots all pasteboard items/types, posts tagged ⌘C, polls changeCount for up to 300 ms, reads the string and restores the previous contents. Backup is split across timer turns (up to 64 items / 256 types / 8 MiB / 300 ms); incomplete or promised-file backups skip capture without modifying the pasteboard. `%MS_CLIPBOARD_TEXT%` reads NSPasteboard before the copy fallback. There are no sleeps on the main thread. Avoid changing the clipboard during that short fallback: the system does not identify which writer changed it. Secure fields, app AX gaps and late copy responses can yield empty text. Logs never include keys, text, clipboard contents, scripts or URL query strings.

## macOS defaults

`Config::builtin()` selects `assets/default-config-macos.json` on macOS. The general-scope gestures share the configuration schema; macOS corner names and enabled rules are platform-specific; explorer/cmd scopes are omitted. Existing saved configurations, including M1 configs, are preserved. Self-test uses new defaults in memory. For interactive testing, quit the app, back up `~/Library/Application Support/mousetroke/config.json`, then copy `crates/mousetroke-core/assets/default-config-macos.json` there before restarting.

| Gestures | macOS decision |
| --- | --- |
| Back / forward | ⌘[ / ⌘] |
| New / reopen / close tab, refresh | ⌘T / ⌘⇧T / ⌘W / ⌘R |
| Previous / next tab (both patterns) | ⌃⇧Tab / ⌃Tab |
| Top / bottom | ⌘↑ / ⌘↓ |
| Copy / paste / cut / paste + Enter | ⌘C / ⌘V / ⌘X / ⌘V then Return |
| Enter / backspace / forward delete / Esc | Return / Delete / Forward Delete / Esc |
| Screenshot / minimize / full screen / close window | ⌘⇧4 / AXMinimized / ⌃⌘F / ⌘⇧W (browser window shortcut) |
| Previous / next desktop | ⌃← / ⌃→ |
| Corners / edge combinations | See the M3b table below; new defaults apply to fresh configurations |
| Zoom in / out / reset | ⌘= / ⌘− / ⌘0 |
| Maximize / minimize | Enabled: AX zoom button / AXMinimized on the captured window |
| Topmost | Disabled: no public cross-app API |
| Force kill / web search | Enabled: `/bin/kill -9 %MS_PID%` / selected-text search |
| Media / volume | Enabled: tagged NX system-defined events |
| Brightness | Supported for built-in displays; defaults remain disabled for the Mac mini external-display test setup; external DDC is out of scope |
| Task Manager / system settings | Enabled: `com.apple.ActivityMonitor` / `com.apple.systempreferences` |

System and application shortcuts can be reassigned. F11, Spaces and Mission Control depend on Keyboard Shortcuts settings. Maximize/restore is not replaced by full screen. macOS has no public API for keeping another application's window always on top. Labels and the Hot Corner warning use all ten translation catalogs.

## Screen corners, edges and system Hot Corners

The canvas uses the union of **full** `NSScreen.frame` rectangles, converted from AppKit bottom-left coordinates to Quartz global logical points with `screens[0]` as the origin reference. `visibleFrame` is used only for label anchors: neither the menu bar nor Dock is subtracted from edge hit testing. Retina scale changes rendering, not event coordinates. Negative screen origins and displays above/below the primary are included. As on Windows, edges belong to the virtual desktop bounding rectangle: internal display seams are not edges; an unoccupied corner of a staggered multi-display bounding rectangle cannot be reached with a physical pointer.

The shared engine requires a 30 ms dwell. macOS can stop emitting mouse moves when the pointer hits a boundary, so the main timer supplies one settle check after 40 ms. It uses the last delivered position, starts only an idle edge capture, or re-evaluates an existing button-free edge trace after its final segment has grown past the denoising threshold. It does not post another event, append trace points or affect held-button gestures. Pure corners fire once until the pointer leaves the edge region; wheel combinations can repeat while held at the edge; the first click/wheel must be within one point of the edge. The shared 100-segment maximum and swallowed-button-release handling remain in effect.

| Position / input | Fresh macOS default |
| --- | --- |
| Top-left | Mission Control, ⌃↑ |
| Top-right | Unbound (the old template is disabled) |
| Bottom-right | Show desktop, F11 |
| Bottom-left | Spotlight, ⌘Space |
| Top + wheel up / down | Volume up / down |
| Top + middle / right click | Mute / play-pause |
| Bottom + wheel | Brightness templates disabled: Dock overlap and external-display limitations |
| Right + ↑↓↑↓ rubbing | Activity Monitor (`com.apple.ActivityMonitor`), execute on match without a button release |

Top-right stays unbound because Notification Center has no stable public scripting entry, and Launchpad/Apps availability and shortcuts differ across macOS versions. No private API or fixed application path is assumed. Saved configurations are preserved; changing built-in defaults does not rewrite an existing user's rules.

At startup and after each **successful** config reload, the host refreshes and reads `com.apple.dock` preferences through `CFPreferencesCopyAppValue`: `wvous-tl-corner` → 9, `wvous-tr-corner` → 3, `wvous-bl-corner` → 12, `wvous-br-corner` → 6. Missing/non-numeric values and 0/1 mean no action. A value >1 conflicts only with an enabled binding beginning with that corner in an enabled scope (including corner combinations and app scopes). Each conflicting corner is logged once per check; one short translated toast names all conflicts. It does not alter system preferences or Mousetroke bindings. Modifier-based system Hot Corners are conservatively reported too. This check is skipped in self-test, and is not a continuous preferences watcher.

## Shared Lua runtime

`crates/mousetroke-lua` owns VM creation, safe standard-library setup (`Lua::new`), the 8 MiB memory limit, BOM handling, `ms_action(pos, wid, pid, exe, title, mode)` entry dispatch, integer compatibility conversions, formatting, the `ms` table and the migrated neutral tests. `LuaPlatform` receives plain Rust values. The Windows crate keeps the original Win32 implementation and WG/User32/luanet/import shims; the macOS crate implements supported operations using M3a helpers. `wgAction` remains an entry fallback, but macOS does not install Windows legacy tables.

Each gesture starts a fresh VM on a worker, after obtaining the captured target's title through AX on that worker. `wid` is CGWindowID; `exe` is the captured bundle id (or executable-path fallback); `pid`, title and mode describe that captured target. AppKit requests go through a bounded queue (64 requests, at most 8 drained per timer turn). Only the Lua worker waits for a read result; the main thread never waits for Lua. `ms.keys` retains parsed text mode and returns after the timer-driven sequence completes; pauses and mouse input remain responsive during sequence delays and `ms.sleep`.

| Lua API | macOS support |
| --- | --- |
| `ms.log`, `ms.sleep`, `ms.read_text` | Supported on the worker; explicit logs/errors may include script-provided text |
| `ms.get_mouse_pos`, `ms.set_mouse_pos` | Quartz global points; posted moves carry the recursion-suppression tag |
| `ms.get_screen(x, y)` | Full containing display bounds `{left, top, right, bottom}`, exclusive right/bottom; primary fallback outside all screens |
| `ms.keys(sequence, text_by_default?)` | Shared parser, mapped keyboard chords/text, sleep/repeat; mouse/unmapped items skipped like M3a KeySequence |
| `ms.open`, `ms.run`, `ms.cmd` | M3a NSWorkspace URL/app launch and `/bin/zsh -c`; run args use the existing quoted word parser |
| `ms.is_paused`, `ms.set_paused`, `ms.show_toast` | Supported; state writes are dispatched to the main thread; toast uses the engine queue |
| `ms.get_clipboard_text`, `ms.set_clipboard_text` | String pasteboard on the main thread; set replaces current contents |
| `ms.focus_win(wid)` | Activates the captured application for the captured CGWindowID; does not raise a particular AX window |
| `ms.find_win_by_class`, `ms.find_win_by_title`, `ms.set_win_state`, `ms.get_win_state`, `ms.set_win_top` | Unsupported in the Lua backend; nil + one log per API per process (normal WindowOp actions still use M3a AX) |
| `ms.get_brightness`, `ms.set_brightness` | Unsupported Lua brightness reads/adjustments; nil + one log; media brightness actions remain separate |
| `ms.show_settings` | Opens the independent settings process using the current engine config |
| `ms.is_keys_pressed`, `ms.keys_down`, `ms.keys_up` | Unsupported; nil + one log |
| `WG`, `User32`, `luanet`, `import` | Windows-only shims, absent on macOS |

Compatibility deliberately retains the existing Windows execution policy: **there is no wall-clock timeout or cancellation hook**. Load/entry errors stop that VM invocation, memory errors respect the 8 MiB limit, and sleep blocks only its worker. An infinite loop can consume a worker until process exit; changing this policy on both platforms is separate work. `Lua::new` is not an untrusted-code security boundary. In self-test, Lua actions log their dry-run kind before any AX, clipboard or VM work.

## Self-test and real-Mac verification

Quit the running instance. With the same graphical user logged in, run the signed executable directly over SSH:

```bash
MOUSETROKE_SELFTEST=1 target/mac/Mousetroke.app/Contents/MacOS/mousetroke
result=$?
tail -n 80 "$HOME/Library/Logs/mousetroke/mousetroke.log"
test "$result" -eq 0
```

Self-test preserves existing saved config. Once its warm-up move reaches the tap, it posts **untagged** right-button ↓ then → drags (200 points per leg, 30 steps per leg, at least 10 ms apart), then a plain right click. Avoid physical mouse input. After that, it posts one untagged move into the virtual top-left corner, waits for the idle settle check, verifies one corner match, then moves away. It requires one expected close-tab match plus one top-left match, exactly one tagged replay down/up pair, positive real-overlay `start_session`/`line_to` counters, two expected KeySequence dispatches in dry-run mode, and non-empty bundle IDs resolved from the mouse-down target. A system Hot Corner can still react to the physical cursor move; clear it before an isolated self-test. The executor logs what it would post and **never posts ⌘W or other action keys**. The cursor is restored. Exit **0** means all checks passed; **1** means failure or a 120-second timeout. Overlay counters verify calls, not WindowServer pixels.

Then start normally with the macOS defaults and check:

1. In a browser with history, draw ←/→; verify back/forward, trail, translated label and unchanged focus. Try ↓→ on a disposable tab; it must close that tab.
2. Start another gesture within 200 ms of release, repeatedly. No old trail should remain or erase the new trail. Try an unrecognized gesture and verify its toast disappears.
3. Repeat on Retina/1x screens, screens left/above/below primary, across screen boundaries and in a full-screen browser (gestures must pass through while fullscreen pause is active). Verify visible-frame label anchors, crisp text and configured colors/width.
4. While idle and drawing, connect/disconnect a display, change resolution/scaling or rearrange screens. The old canvas must close, capture must cancel, and the next gesture must use the new layout.
5. Test copy/paste, tab navigation, Option/Control/Command chords and `#c{SLEEP 500}#v` in a disposable document. Confirm no recursion or stuck modifiers; clicks and pause should respond during delays.

Config: `~/Library/Application Support/mousetroke/config.json`. Log: `~/Library/Logs/mousetroke/mousetroke.log`, rotated at 1 MiB to `.log.old`. `MOUSETROKE_SHAPE_SAMPLES=1` enables normalized shape samples. A nonblocking exclusive `flock` prevents duplicate instances.

AppKit, engine, tap, overlay, timers, menus and the key executor all run on the main thread. Lua VMs run on workers. WindowServer queries, AX IPC and process launch/wait run on workers with plain data sent back to the main timer. Replay and action injection occur after the tap returns. Logs use a bounded background queue; callback boundaries catch Rust panics. Windows config and CLI behavior are unchanged. Pure coordinate/color tests also run on Windows.

## M3a reviewer checklist (real Mac)

Use a signed build with both permissions granted, an active graphical session, and fresh macOS defaults copied as described above. Run the self-test first (exit 0; resolved bundle id non-empty; no action side effects). Use disposable documents/windows and a disposable process for destructive actions. For action cases without a built-in binding, temporarily assign the JSON action to a known gesture in the config and let hot reload apply it.

- Target/scopes: overlap two apps, keep A frontmost but draw on B's exposed window; B's bundle-id scope (for example `com.google.Chrome`) must win; repeat after moving the .app, a disabled scope, and two windows of the same app.
- Fallback/ignore: right-click the desktop, Dock, menu bar, Control Center, Notification Center and Mousetroke menu; normal system menus must work without an accidental gesture.
- Fullscreen pause: enter a browser fullscreen Space, then a presentation covering an entire display; gestures pass through, resume after leaving; a merely maximized window with the menu bar visible must not pause; repeat on a secondary screen.
- KeySequence: back/forward and `#c{SLEEP 500}#v`; observe correct keys, responsive pause/clicks during the delay, no recursive gestures or stuck modifiers; `activate_target:true` must activate the captured app first.
- Minimize: draw on an exposed background window; only that window enters the Dock, the other app stays open.
- ToggleMaximize: draw twice on a resizable window; zoom then restore; a window without a zoom button logs and skips; minimize it before dispatch to verify the minimized skip.
- Close: draw on a disposable background window; only the captured window closes (or asks to save).
- ToggleTopmost: enable a temporary binding; observe a short translated unsupported toast and log, no window stacking change; repeat after changing UI language.
- LaunchApp Activate: launch `com.apple.ActivityMonitor`, then its .app path; the second gesture activates the existing app; quoted args such as `--args 'two words'` must remain separate arguments when tested with a suitable app.
- LaunchApp RunAgain/SendKeys/elevate: test a disposable app supporting multiple instances; RunAgain requests a second instance, SendKeys activates without typing args, elevate logs unsupported without prompting for root.
- LaunchApp executable: launch an absolute test executable such as `/usr/bin/true`; a completion status appears and mouse input remains responsive.
- OpenUrl: select `a b&中文` and invoke search; the query is encoded exactly once; select `https://example.com/?x=1` and observe direct navigation; repeat with `browser:com.apple.Safari` and a browser .app path.
- Shell/variables: use `/bin/sleep 2; /usr/bin/printf '%s\n' '%MS_EXE%' '%MS_PID%' '%MS_WID%' '%MS_WIN_NAME%' '%MS_TARGET_MODE%' '%MS_CLIPBOARD_TEXT%' > /tmp/mousetroke-m3a-vars`; input stays responsive, file values describe the captured target, status is logged later, no values appear in the log.
- Force kill: open a disposable app and draw the kill gesture on its window; `/bin/kill -9` ends only the captured PID.
- Selected text AX: select text in TextEdit and search; the selected text is used while an existing rich clipboard remains unchanged.
- Selected text fallback: in an app that omits AXSelectedText, select text and search; ⌘C works, the previous text/rich/image pasteboard items are restored; with no selection or a copy timeout the action completes using empty text and input stays responsive.
- SystemKey play/next/previous: with a media app open, trigger each action; playback toggles and tracks move as expected, with no echo gesture.
- SystemKey mute/up/down: observe system volume changes; on fixed-volume external audio outputs macOS may ignore them.
- SystemKey brightness up/down: on a MacBook built-in panel brightness changes; on the Mac mini external display only the scope-limit log is expected; unknown key logs and skips.
- Menu: invoke a multi-item menu with a nested menu; choose an action and observe it after menu dismissal, Escape cancels; repeat with monitors left/above/below primary and mixed Retina scale to verify cursor placement.
- Lua: assign `function ms_action(pos, wid, pid, exe, title, mode) ms.log("target %d %s", pid, title); ms.keys("#c{SLEEP 500}"); ms.sleep(500); ms.show_toast("Lua OK"); end`; use a disposable document, check captured context, responsive input/pause during delays, and no recursive gestures. Test text-mode keys and clipboard set/get; call an unsupported API twice and verify nil plus only one unsupported log.
- System Settings: invoke the default gesture; the existing settings app activates or opens using its bundle id.
- Permissions/privacy: keep Screen Recording denied, revoke/regrant Accessibility and Input Monitoring separately; prompts/recovery work, AX title has no Screen Recording dependency, no sensitive action content appears in the log.

Remaining risks requiring hardware validation: WindowServer/LaunchServices/AX behavior on macOS 27 and macOS 11, approximately 250 ms stale targeting/fullscreen snapshots, applications with incomplete AX support or identical overlapping window bounds, the unavoidable race with unrelated or late clipboard writes, and media/brightness hardware policy. Synchronous individual AppKit pasteboard calls still depend on the pasteboard server/provider responding promptly; timer slicing bounds our own work, not a misbehaving external provider.

## M3b reviewer checks (real Mac)

- Build/check/clippy on the Mac with the system `cc` (vendored Lua compiles from source); run core/engine/lua/mac tests and the signed self-test; require exit 0 and two dry-run dispatches with no action key posting.
- On the 2560×1440 display, push into each enabled corner once, remain there, move away and return. Mission Control, F11 and Spotlight depend on the user's system shortcut assignments. Top-right should do nothing.
- At the literal menu-bar top row, test repeated wheel volume and middle/right clicks. Just below the edge, native clicks/scrolling must pass. The Dock must keep working; brightness defaults remain disabled.
- Rub ↑↓↑↓ along the right edge without any mouse button: Activity Monitor opens once. More rubbing must not fire until leaving/re-entering. For the scribble limit, use a temporary non-executing long edge-trace binding and exceed 100 segments, with/without an additional held button; release must not leak.
- Set each system Hot Corner in turn, reload the configuration, and verify one named warning only for enabled Mousetroke corner bindings; disable the binding/scope and reload to verify no warning. Confirm neither setting is changed. Repeat after changing UI language.
- Repeat edge/corner input on left/above/below display layouts, mixed scaling, and after hot-plug/rearrangement; test fullscreen pause. Internal seams and physically unreachable bounding corners follow the Windows virtual-desktop behavior.
- Run the Lua cases above, plus URL/app/shell actions, captured-app focus, unsupported calls and load/runtime errors. Restart after deliberately testing a runaway script; there is no script cancellation UI.


## Settings (M4)

Build the embedded UI **before** invoking `scripts/mac/bundle.sh`:

```bash
cd crates/mousetroke-settings/ui
npm ci
npm run typecheck
npm test
npm run build
cd ../../..
CARGO_TARGET_DIR=target CARGO_BUILD_JOBS=4 scripts/mac/bundle.sh
```

The default `custom-protocol` feature embeds `ui/dist` into the executable. The bundle remains a single `Contents/MacOS/mousetroke` executable. `mousetroke settings [config] [--about]` runs Tauri on that process's main thread. The menu bar has **Settings… (⌘,)** and **About Mousetroke**; both spawn a separate settings process with the engine's config path. Closing settings exits that process; engine config hot reload continues unchanged. `ms.show_settings()` uses the same launch path. `mousetroke [run] [--config path]` also supports an explicit engine config.

Engine and settings hold separate nonblocking `flock` files in Application Support. The locked files are never replaced or unlinked during normal use. A second settings launch atomically publishes a focus/about request, and the existing process restores, shows and activates its native window. Requests also work while the first window is still starting. No title search, Screen Recording permission or PID-based signaling is involved. A single settings instance edits the config selected by its first launch; later launches focus it without changing that config.

The settings backend lives in `platform/windows.rs` and `platform/macos.rs`, with platform-specific icon and picker modules. macOS uses `NSLocale.preferredLanguages`, the same config/log locations as the engine, regular `NSWorkspace.runningApplications`, and `NSWorkspace.iconForFile` → TIFF → `NSBitmapImageRep` PNG with a bounded cache. Every app selector, drop and `.app` resolution returns the **bundle id**, falling back to an absolute executable path for non-bundle programs. App scopes, `%MS_EXE%` and Lua `exe` follow that identity. Existing path-based app scopes must be reselected to use the bundle id; saved configs are not silently rewritten.

The picker places a transparent click-catching window on every display, uses a crosshair and translated hint, and handles left-click, Escape, right-click and a 60-second timeout. It converts AppKit points to Quartz coordinates, including negative display origins, and selects the first normal layer-zero window under the point, excluding Mousetroke and system UI. It reads bounds and owner PID only, never window titles, so **Screen Recording is not required**. Fullscreen Spaces, cursor appearance and display rearrangement still need real-Mac validation.

Login launch uses **SMAppService.mainAppService on macOS 13+**, discovered only after a runtime version check; no 13+ class symbol is imported on Big Sur. Enabled state comes from the service status, including user approval/disabled states. Approval-needed registration shows a translated instruction and stays unchecked. Run the signed `.app`, rather than a standalone development binary, for this path. On **11–12**, a per-user `~/Library/LaunchAgents/io.github.eiaao.mousetroke.plist` contains the absolute executable path as one ProgramArguments entry, with RunAtLoad. `launchctl enable/bootstrap/disable` applies changes without stopping a running engine when login launch is disabled, and status checks the plist, loaded service and launchd disabled override. No administrator privileges are needed. Moving the app requires re-enabling login launch.

macOS retains the native title bar/traffic lights and light/dark appearance. The backend platform value drives Command/Option/Control/Shift names, physical-key recording and ⌘F/⌘D/⌘Z/⌘⇧Z UI shortcuts; stored keys remain shared Windows-VK logical values. Finder, menu bar, login launch and application examples have ten-language variants. Unsupported imported action options get an explicit warning and repair button; unmapped or mouse keys in key sequences get a translated validation error; topmost cannot be selected, elevation is hidden, and Windows Lua shims are called out. **One-click installation remains Windows-only**. Mac About supports shared update checks and release notes, with browser DMG downloads; Preferences controls daily checking. About also exposes the homepage, source/Star and feedback links.

Fresh macOS defaults disable gestures in Parallels Desktop, VMware Fusion, Windows App, Screen Sharing, VirtualBox VM, Blender, Cinema 4D, Maya, Unity and Unreal Editor. Remote/VM apps need their mouse buttons passed through to another OS; 3D/game editors need right/middle drag for viewport navigation. These bundle-id entries also populate the existing recommended-disable picker and onboarding list; installation detection uses LaunchServices. Existing configurations remain intact.

CI builds/tests the UI on macOS and Windows, builds the embedded settings library and binary, and runs the Rust command tests plus `settings_config_round_trip` without a window server or permissions. The smoke test exercises missing config defaults, validated atomic save, reload, export/import and invalid-config reporting.

Real-Mac hand test (signed bundle; macOS 27 on Apple silicon, repeat login fallback on 11/12):

1. Open Settings from the menu and CLI with a config path containing spaces; reopen normally and with `--about`, including while minimized. There must be one settings window and the engine must keep responding after closing it.
2. Edit a gesture, pause hotkey, language and trail setting; verify save/reload, live engine behavior, undo/redo and light/dark appearance. Record Command/Option/Control/Shift, arrows and function keys.
3. Select running apps, browse/drop a `.app` from /Applications, and select an executable. Verify bundle ids/full executable paths in the saved scope and matching behavior in the engine. Check icons and the ten default-disabled apps.
4. With Screen Recording denied, pick an overlapping background app; cancel with Escape/right-click. Repeat on the 2560×1440 screen, fullscreen Spaces and secondary screens at negative origins; no click should reach the target app.
5. Enable/disable login launch, verify the actual System Settings state, log out/in, and repeat with user approval denied. Confirm engine status/start and Finder/log/project links.
6. Open About, check current/latest version and release notes, and download the newer DMG in the browser; import unsupported action options and check their warning and explicit repair. Run onboarding in each of the ten languages and check native traffic lights and focus.


## Universal beta release

The reviewer Mac needs Xcode Command Line Tools, Rust, Python 3 and both Rust targets.
There is no Node dependency in these scripts: copy the already built
`crates/mousetroke-settings/ui/dist` from Windows before building. CI builds it with Node 22.

```bash
export CARGO_BUILD_JOBS=4 CARGO_TARGET_DIR="$PWD/target" MACOSX_DEPLOYMENT_TARGET=11.0
rustup target add aarch64-apple-darwin x86_64-apple-darwin
cargo test -p mousetroke-core -p mousetroke-engine -p mousetroke-mac -p mousetroke-settings --locked
# 快速主机架构开发包；开发证书缺失时有明确警告并回退 ad-hoc。
bash scripts/mac/make-dev-cert.sh
bash scripts/mac/bundle.sh
# CI 冒烟明确选择 ad-hoc，不能用于公开发布。
MOUSETROKE_SIGN_ID=- bash scripts/mac/bundle.sh --universal
MOUSETROKE_SIGN_ID=- bash scripts/mac/dmg.sh
```

Create the release certificate **once**, in a private directory outside every repository
and worktree. The script refuses existing output, repository paths and paths resolving
through symlinks into the repository. Enter a nonempty password twice without echo.
The ten-year certificate is RSA 2048 with the codeSigning EKU and common name
**Mousetroke Self-Signed**. The private key is removed from the system temporary directory
on exit. Only the password-protected p12 remains, with owner-only permissions.

```bash
mkdir -p "$HOME/.config/mousetroke-release"
chmod 700 "$HOME/.config/mousetroke-release"
bash scripts/mac/make-release-cert.sh "$HOME/.config/mousetroke-release/signing.p12"
# 脚本只打印此命令；执行后从剪贴板粘贴到 GitHub Secret，不把内容输出到日志。
base64 < "$HOME/.config/mousetroke-release/signing.p12" | tr -d '\n' | pbcopy
```

Set repository Actions secrets `MAC_SIGN_P12_BASE64` to that clipboard value and
`MAC_SIGN_P12_PASSWORD` to the entered password. Back up the p12 and password securely;
**do not regenerate the certificate for each release**. Do not put either in git or logs.
The release job fails if either secret is absent or import/signing fails. It creates an
unlocked temporary keychain, imports the identity, allows codesign access, signs with
`--options runtime`, then restores the keychain list and deletes temporary signing material
with an `always()` step. No notarization is performed in M5a.

For a real signed local build over SSH, import that same p12 into a temporary unlocked
keychain. Run these commands in a subshell so cleanup also happens on failure. Enter the
P12 password interactively; do not substitute its literal value into a command.

```bash
(
set -euo pipefail
umask 077
temporary=$(mktemp -d)
keychain="$temporary/release.keychain-db"
trap 'security delete-keychain "$keychain" >/dev/null 2>&1 || true; rm -rf "$temporary"; unset p12_password keychain_password' EXIT
read -r -s -p 'P12 password: ' p12_password; printf '\n'
keychain_password=$(openssl rand -hex 32)
security create-keychain -p "$keychain_password" "$keychain"
security set-keychain-settings -lut 21600 "$keychain"
security unlock-keychain -p "$keychain_password" "$keychain"
security import "$HOME/.config/mousetroke-release/signing.p12" -k "$keychain" -P "$p12_password" -T /usr/bin/codesign >/dev/null 2>&1
security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$keychain_password" "$keychain" >/dev/null 2>&1
export MOUSETROKE_SIGN_KEYCHAIN="$keychain"
export MOUSETROKE_SIGN_ID='Mousetroke Self-Signed' MOUSETROKE_REQUIRE_SIGNING=1
bash scripts/mac/bundle.sh --universal
bash scripts/mac/dmg.sh
lipo -archs target/mac/Mousetroke.app/Contents/MacOS/mousetroke
codesign --verify --strict --verbose=2 target/mac/Mousetroke.app
codesign --verify --strict --verbose=2 target/mac/Mousetroke.dmg
codesign -d -r- target/mac/Mousetroke.app
hdiutil verify target/mac/Mousetroke.dmg
)
```

Expect `arm64 x86_64` (order may differ). The designated requirement must retain
`identifier "io.github.eiaao.mousetroke" and certificate root = H"<same SHA-1>"`
across independently rebuilt versions. This explicitly avoids a requirement based on the
binary's changing code hash. SHA-1 here identifies the certificate, not the asset checksum.
Keep the app in the same Applications path. Certificate changes, moving the app or macOS
policy changes may still require granting permissions again; verify on both macOS 27 and 11.

`dmg.sh` creates `target/mac/Mousetroke.dmg` with `hdiutil` UDZO, volume name
**Mousetroke**, the signed app and an `/Applications` symlink, and signs the DMG with
the same identity. `release.yml` has `windows`, `macos` and `publish` jobs. The first two
upload `windows-release` (exe + notes) and `macos-release` (DMG) artifacts. Only `publish`
has contents-write permission; it waits for both, generates and verifies one
`SHA256SUMS.txt` listing `mousetroke.exe` and `Mousetroke.dmg`, then creates one release.

## First open, updates and uninstall

Open the DMG, drag Mousetroke to **Applications**, then launch it there. The beta is
self-signed and **not notarized**: if blocked, go to **System Settings → Privacy & Security →
Open Anyway** (older macOS: **right-click → Open**). Each newly downloaded version may
need this again. Grant **Accessibility** and **Input Monitoring**; no Screen Recording.
Launching from `/Volumes/` or `/AppTranslocation/` logs the location problem and queues one
translated eight-second notice per engine launch. It does not modify quarantine attributes.

About reuses the Windows version comparison and plain-text, language-selected release-note
renderer. The transport is `/usr/bin/curl`, already present on macOS 11, avoiding another
network crate or Objective-C session lifecycle. It disables curlrc, restricts protocols to
HTTPS, validates the exact `api.github.com` host, follows no redirects, requires HTTP 200,
and limits connect/total time to 10/30 seconds and response body to 1 MiB. Only API metadata
is fetched by the app; the checked `Mousetroke.dmg` URL is opened in the browser after an
explicit download action. No one-click installation runs on Mac. The API must provide one
exact DMG and one exact sums asset; malformed/duplicate/untrusted assets are rejected.

The daily worker follows `settings.update_check` and its live reload. It saves the attempt
before networking in `update-state.json` beside the config, checks at most every 24 hours,
and suppresses repeat menu notices for the same version. Rollback of the clock postpones
checking. A version menu item opens settings About. Disabled checks do not read state or
make requests; an already active request may finish, but its notice is suppressed.
Windows retains WinHTTP, BCrypt SHA-256, file replacement/rollback and restart behavior;
mixed Windows/Mac asset and checksum fixtures exercise the actual Windows selectors.

To uninstall, quit from the menu bar; delete the app,
`~/Library/Application Support/mousetroke` and `~/Library/Logs/mousetroke`; remove its login
item in System Settings.

## M5a acceptance checklist

- Build twice with the same release p12; compare `codesign -d -r-` requirements, verify
  both slices and hardened runtime, and run each host architecture (`arch -x86_64` requires
  Rosetta on Apple silicon). Run natively on the Intel macOS 11 machine too.
- Mount the DMG read-only, verify volume name, app and Applications symlink; launch there
  and from a quarantined/translocated downloaded copy and confirm one installation notice
  plus log. Copy to Applications and confirm no notice.
- Download through a browser to test real quarantine/Gatekeeper; follow Open Anyway,
  grant both permissions, verify gestures/settings and then replace with the second signed
  build at the same path. Confirm grants and gestures survive and login launch still works.
- About: test current/newer/error responses, Chinese/English release-note selection and
  fallback, retry, browser DMG download, no in-app installation. Test the ten UI languages.
- With a newer real release, enable daily checks and confirm one menu notice opening About;
  restart and confirm no second request that day. Disable/re-enable and live-reload the
  preference; inspect the state/log. Invalid/future state and offline networking must not
  block event processing.
- Run `bash -n scripts/mac/*.sh` with a loop and shellcheck if available; CI also runs
  universal + ad-hoc DMG smoke tests. Locally remove either release secret in a test workflow
  and confirm the mac job fails and publish never starts (do not create a public test tag).
- Measure the final signed DMG size, fill `@@MAC_DMG_SIZE@@` in the site sources, regenerate
  all pages and require `python site/check.py` without the pre-release allowance before
  deploying the site. Keep version/changelog updates for the actual release.
