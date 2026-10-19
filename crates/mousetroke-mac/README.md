# Mousetroke macOS M1

Apple Silicon, macOS 11+. Install Xcode Command Line Tools and Rust. From the repository root:

```bash
export CARGO_BUILD_JOBS=4 CARGO_TARGET_DIR="$PWD/target"
cargo build -p mousetroke --locked
cargo test -p mousetroke-core -p mousetroke-engine -p mousetroke-mac --locked
bash scripts/mac/make-dev-cert.sh
bash scripts/mac/bundle.sh
open target/mac/Mousetroke.app
```

The certificate script creates and trusts a local **Mousetroke Dev** signing identity in the login keychain. macOS may ask for the keychain password or permission to use the private key; allow `codesign`. Keep this identity and the app path stable across rebuilds. Set `MOUSETROKE_SIGN_ID` to use another identity. The bundle script falls back to ad-hoc signing if the identity is unavailable, with a warning: rebuilding then changes the identity and may require Accessibility permission again. This is a development bundle, without notarization.

Grant **Accessibility** and **Input Monitoring** in System Settings → Privacy & Security to `Mousetroke.app`. The menu links to any missing permission. Permissions are checked every two seconds; the event tap starts without restarting. The system permission prompts are requested only on the first launch (tracked by `permission-prompted` in Application Support). An SSH session needs an active graphical login for this user. If testing the raw executable, macOS may attribute permission to the launching terminal; use the signed bundle for repeatable permission tests.

The menu bar icon dims while paused or awaiting permission. Toggle pause from the menu or with **⌃⌥⌘P**. Test ordinary right clicks and right-button drags in another application. M1 logs recognition and action types; actions and the overlay are stubs. Frontmost-app metadata is sampled every 250 ms; window IDs remain zero. Fullscreen detection and automatic updates are stubs.

For an end-to-end SSH test, quit the running app, then run the signed executable directly so its exit status is available:

```bash
MOUSETROKE_SELFTEST=1 target/mac/Mousetroke.app/Contents/MacOS/mousetroke
echo "$?"
tail -n 60 "$HOME/Library/Logs/mousetroke/mousetroke.log"
```

Self-test uses the built-in config in memory, preserving the saved config. Once the tap is active it posts **untagged** right-button ↓ then → drags (200 points per leg, 30 steps per leg, at least 10 ms apart), then a plain right click. Avoid moving the physical mouse during the test. It waits one second, reports the engine’s `手势命中: …` entry (close tab) and tagged replay down/up echoes, restores the cursor and exits **0** on success (**1** on failure or a 120-second timeout). M1 never sends the configured close-tab keys.

Config: `~/Library/Application Support/mousetroke/config.json` (built-in defaults saved on first run). Log: `~/Library/Logs/mousetroke/mousetroke.log`, rotated at 1 MiB to `.log.old`. Set `MOUSETROKE_SHAPE_SAMPLES=1` to log normalized shape samples. A nonblocking exclusive `flock` prevents a second instance from running.

AppKit, the engine, tap callbacks, timers and menu actions all run on the main thread. Tap callbacks only enqueue replay into a fixed-capacity queue; the next main-run-loop timer posts tagged events. Logs use a bounded background queue. Rust panics are caught at callback boundaries.
