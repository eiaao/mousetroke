# Mousetroke macOS M3b

Apple Silicon, macOS 11+. Install Xcode Command Line Tools and Rust. From the repository root:

```bash
export CARGO_BUILD_JOBS=4 CARGO_TARGET_DIR="$PWD/target" MACOSX_DEPLOYMENT_TARGET=11.0
cargo check -p mousetroke-mac -p mousetroke --locked
cargo clippy -p mousetroke-mac --all-targets -- -D warnings
cargo test -p mousetroke-core -p mousetroke-engine -p mousetroke-lua -p mousetroke-mac --locked
bash scripts/mac/make-dev-cert.sh
bash scripts/mac/bundle.sh
open target/mac/Mousetroke.app
```

The certificate script creates a **Mousetroke Dev** signing identity in a dedicated keychain. Follow its trust instructions and keep the identity and app path stable across rebuilds. Set `MOUSETROKE_SIGN_ID` for another identity. Without a matching identity, bundling warns and uses ad-hoc signing; rebuilding can then require Accessibility permission again. This development bundle is not notarized.

**Screen Recording is not required**: titles are obtained through Accessibility, never `kCGWindowName`. Grant **Accessibility** and **Input Monitoring** to `Mousetroke.app` in System Settings → Privacy & Security. The menu links to missing permissions. Permissions are checked every two seconds and the tap starts without restarting. Prompts are requested only on first launch, tracked by `permission-prompted` in Application Support. SSH tests require an active graphical login for the same user. Use the signed bundle for repeatable permission tests.

The menu icon dims while paused or awaiting permission. Toggle pause from the menu or with **⌃⌥⌘P**. Ordinary right clicks are replayed. Right-button drags recognize shared-engine gestures and send configured keys to the focused application. Window targeting and fullscreen pause use a roughly 250 ms snapshot. Target activation is supported; automatic updates remain for later milestones.

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

`Target.exe` is the **.app bundle path**, falling back to the executable path for unbundled programs. Existing per-app scopes match either `/Applications/Google Chrome.app` or the basename `Google Chrome.app` through the unchanged core matcher (case-insensitive). `Target.class` is the **bundle id**, such as `com.google.Chrome`; it filters system UI and is not a second scope matcher. `wid` is the CGWindowID, `pid` the owner PID. The current shared engine resolves scopes from `exe`, not title/class. Window titles are cached at `Host::window_title`; actions referencing `%MS_WIN_NAME%` lazily refresh them on an AX worker, matching AX windows by position and size. No Screen Recording prompt is needed. A moved/closed or inaccessible target is skipped rather than operating on an unrelated focused window.

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

Each gesture starts a fresh VM on a worker, after obtaining the captured target's title through AX on that worker. `wid` is CGWindowID; `exe` is the captured .app bundle path (or executable fallback); `pid`, title and mode describe that captured target. AppKit requests go through a bounded queue (64 requests, at most 8 drained per timer turn). Only the Lua worker waits for a read result; the main thread never waits for Lua. `ms.keys` retains parsed text mode and returns after the timer-driven sequence completes; pauses and mouse input remain responsive during sequence delays and `ms.sleep`.

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
| `ms.is_keys_pressed`, `ms.keys_down`, `ms.keys_up`, `ms.show_settings` | Unsupported in M3b; nil + one log; settings integration is M4 |
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

- Target/scopes: overlap two apps, keep A frontmost but draw on B's exposed window; B's `/Applications/…app` scope must win; repeat with `…app` basename, a disabled scope, and two windows of the same app.
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
