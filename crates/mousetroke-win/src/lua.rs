//! Windows Lua 平台适配与旧 WG / User32 / luanet 兼容层。
use crate::input;
use mlua::{Lua, Table, Value};
use mousetroke_core::keyseq::{Key, ModKey};
pub use mousetroke_lua::LuaCtx;
use mousetroke_lua::{required_keys, KeyPhase, LuaPlatform, RequiredKey};
use windows::Win32::Foundation::HWND;

pub fn run_lua(code: &str, ctx: &LuaCtx) {
    mousetroke_lua::run_lua(code, ctx, std::sync::Arc::new(WindowsPlatform));
}

struct WindowsPlatform;

fn keys_pressed(seq: &mousetroke_core::keyseq::KeySeq, pressed: impl Fn(u16) -> bool) -> bool {
    required_keys(seq).is_some_and(|keys| {
        keys.into_iter().all(|key| match key {
            RequiredKey::Vk(vk) => pressed(vk),
            RequiredKey::Modifier(modifier) => match modifier {
                ModKey::Ctrl => pressed(0x11),
                ModKey::Alt => pressed(0x12),
                ModKey::Shift => pressed(0x10),
                ModKey::Win => pressed(0x5B) || pressed(0x5C),
            },
        })
    })
}

impl LuaPlatform for WindowsPlatform {
    fn log(&self, message: &str) {
        crate::log!("{message}");
    }
    fn install_extensions(&self, lua: &Lua, ms: &Table) -> mlua::Result<()> {
        install_legacy(lua, ms)
    }
    fn cursor_pos(&self) -> (i32, i32) {
        input::get_cursor_pos()
    }
    fn set_cursor(&self, x: i32, y: i32) {
        input::set_cursor_pos(x, y);
    }
    fn screen_of(&self, x: i32, y: i32) -> Option<(i32, i32, i32, i32)> {
        Some(input::screen_of(x, y))
    }
    fn send_keys(&self, seq: &mousetroke_core::keyseq::KeySeq, phase: KeyPhase) {
        match phase {
            KeyPhase::DownUp => input::inject_keyseq(seq),
            KeyPhase::Down => input::inject_keyseq_phase(seq, input::KeyPhase::Down),
            KeyPhase::Up => input::inject_keyseq_phase(seq, input::KeyPhase::Up),
        }
    }
    fn is_keys_pressed(&self, seq: &mousetroke_core::keyseq::KeySeq) -> Option<bool> {
        Some(keys_pressed(seq, |vk| input::is_vk_pressed(Key::Vk(vk))))
    }
    fn open_url(&self, url: &str) {
        input::open_url(url, None);
    }
    fn run_shell(&self, code: &str) {
        input::run_shell(code);
    }
    fn launch_app(&self, file: &str, args: Option<&str>) {
        input::launch_app(
            file,
            args,
            false,
            None,
            None,
            false,
            mousetroke_core::schema::IfRunning::RunAgain,
        );
    }
    fn is_paused(&self) -> bool {
        crate::engine::PAUSED.load(std::sync::atomic::Ordering::Relaxed)
    }
    fn set_paused(&self, on: bool) {
        crate::tray::post_engine_cmd(if on {
            crate::tray::CMD_PAUSE_ON
        } else {
            crate::tray::CMD_PAUSE_OFF
        });
    }
    fn show_settings(&self) {
        crate::tray::open_settings();
    }
    fn show_toast(&self, message: String, duration: u64) {
        crate::engine::push_toast(message, duration);
    }
    fn get_clipboard_text(&self) -> Option<String> {
        input::get_clipboard_text()
    }
    fn set_clipboard_text(&self, text: &str) {
        input::set_clipboard_text(text);
    }
    fn find_windows(
        &self,
        class: Option<&str>,
        title: Option<&str>,
    ) -> Option<Vec<(String, isize)>> {
        Some(input::find_windows_by(class, title, true))
    }
    fn focus_window(&self, wid: i64) {
        input::focus_window(HWND(wid as *mut core::ffi::c_void));
    }
    fn set_window_state(&self, wid: i64, state: &str) {
        input::set_window_state(HWND(wid as *mut core::ffi::c_void), state);
    }
    fn set_topmost(&self, wid: i64, top: bool) {
        input::set_topmost(HWND(wid as *mut core::ffi::c_void), top);
    }
    fn window_state(&self, wid: i64) -> Option<String> {
        use windows::Win32::UI::WindowsAndMessaging::{IsIconic, IsZoomed};
        let h = HWND(wid as *mut core::ffi::c_void);
        // 安全：保留原有 HWND 查询行为，失效句柄由 Win32 返回未置位状态。
        Some(
            unsafe {
                if IsIconic(h).as_bool() {
                    "min"
                } else if IsZoomed(h).as_bool() {
                    "max"
                } else {
                    "normal"
                }
            }
            .into(),
        )
    }
    fn get_brightness(&self, x: i32, y: i32) -> Option<i64> {
        crate::brightness::get(x, y).map(i64::from)
    }
    fn adjust_brightness(&self, x: i32, y: i32, delta: i32) -> Option<i64> {
        crate::brightness::adjust(x, y, delta).map(i64::from)
    }
}

fn install_legacy(lua: &Lua, ms: &Table) -> mlua::Result<()> {
    // 兼容表：旧格式 PascalCase API 名映射到 snake_case 实现
    const LEGACY_NAMES: &[(&str, &str)] = &[
        ("Log", "log"),
        ("Sleep", "sleep"),
        ("IsPaused", "is_paused"),
        ("SetPaused", "set_paused"),
        ("ShowSettings", "show_settings"),
        ("ShowToast", "show_toast"),
        ("SetWinTop", "set_win_top"),
        ("ReadText", "read_text"),
        ("Keys", "keys"),
        ("IsKeysPressed", "is_keys_pressed"),
        ("FindWinByClass", "find_win_by_class"),
        ("FindWinByTitle", "find_win_by_title"),
        ("FocusWin", "focus_win"),
        ("SetWinState", "set_win_state"),
        ("GetScreen", "get_screen"),
        ("GetMousePos", "get_mouse_pos"),
        ("SetMousePos", "set_mouse_pos"),
        ("GetClipboardText", "get_clipboard_text"),
        ("SetClipboardText", "set_clipboard_text"),
        ("Open", "open"),
        ("Run", "run"),
        ("Cmd", "cmd"),
        ("GetWinState", "get_win_state"),
        ("KeysDown", "keys_down"),
        ("KeysUp", "keys_up"),
    ];
    let legacy = lua.create_table()?;
    for (old_name, ms_name) in LEGACY_NAMES {
        if let Ok(v) = ms.get::<Value>(*ms_name) {
            let _ = legacy.set(*old_name, v);
        }
    }
    // 仅旧格式亮度包装保留失败值 -1，ms 表统一使用 nil。
    for (old_name, ms_name) in [
        ("GetBrightness", "get_brightness"),
        ("SetBrightness", "set_brightness"),
    ] {
        let function: mlua::Function = ms.get(ms_name)?;
        legacy.set(
            old_name,
            lua.create_function(move |_, args: mlua::MultiValue| {
                Ok(function.call::<Option<i64>>(args)?.unwrap_or(-1))
            })?,
        )?;
    }
    // 旧格式脚本能引用但暂未实现的方法 → 记录日志的空操作
    for stub in [
        "GetTriggerState",
        "SetTriggerState",
        "GetExplorerPath",
        "ToggleKeys",
        "GetClipboardData",
        "SetClipboardData",
    ] {
        let name = stub.to_string();
        let _ = legacy.set(
            stub,
            lua.create_function(move |_l, _args: mlua::MultiValue| {
                crate::log!("Lua: 旧格式 API {name} 尚未实现");
                Ok(())
            })?,
        );
    }
    let _ = lua.globals().set("WG", legacy); // 旧格式脚本里的全局表名，必须保持原样

    // User32 / luanet / import：旧格式 NLua 互操作的最小桩
    let user32 = lua.create_table()?;
    for stub in [
        "GetWindowText",
        "MessageBox",
        "PostMessage",
        "GetForegroundWindow",
    ] {
        let name = stub.to_string();
        let _ = user32.set(
            stub,
            lua.create_function(move |_l, _args: mlua::MultiValue| {
                crate::log!("Lua: User32.{name} 互操作暂未实现");
                Ok(())
            })?,
        );
    }
    let _ = lua.globals().set("User32", user32);
    let luanet = lua.create_table()?;
    luanet.set(
        "enum",
        lua.create_function(|_l, (_t, name): (Value, String)| Ok(name))?,
    )?;
    let _ = lua.globals().set("luanet", luanet);
    lua.globals()
        .set("import", lua.create_function(|_l, _m: String| Ok(()))?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn read_only_ms_helpers_use_the_windows_backend() {
        let lua = Lua::new();
        let ctx = LuaCtx::default();
        let platform: std::sync::Arc<dyn LuaPlatform> = std::sync::Arc::new(WindowsPlatform);
        mousetroke_lua::install_ms_table(&lua, &ctx, &platform).unwrap();
        let paused: bool = lua.load("return ms.is_paused()").eval().unwrap();
        assert_eq!(
            paused,
            crate::engine::PAUSED.load(std::sync::atomic::Ordering::Relaxed)
        );
        let read: mlua::Function = lua
            .globals()
            .get::<Table>("ms")
            .unwrap()
            .get("read_text")
            .unwrap();
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml");
        assert_eq!(
            read.call::<String>(path).unwrap(),
            include_str!("../Cargo.toml")
        );
        assert!(matches!(
            read.call::<Value>(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml/missing"))
                .unwrap(),
            Value::Nil
        ));
    }
    #[test]
    fn legacy_aliases_share_functions_without_calling_them() {
        let lua = Lua::new();
        let ctx = LuaCtx::default();
        let platform: std::sync::Arc<dyn LuaPlatform> = std::sync::Arc::new(WindowsPlatform);
        mousetroke_lua::install_ms_table(&lua, &ctx, &platform).unwrap();
        lua.load(
            r#"
            local names = {
                Log = 'log', Sleep = 'sleep', IsPaused = 'is_paused',
                SetPaused = 'set_paused', ShowSettings = 'show_settings',
                ShowToast = 'show_toast', SetWinTop = 'set_win_top',
                ReadText = 'read_text', Keys = 'keys', IsKeysPressed = 'is_keys_pressed',
                FindWinByClass = 'find_win_by_class', FindWinByTitle = 'find_win_by_title',
                FocusWin = 'focus_win', SetWinState = 'set_win_state', GetScreen = 'get_screen',
                GetMousePos = 'get_mouse_pos', SetMousePos = 'set_mouse_pos',
                GetClipboardText = 'get_clipboard_text', SetClipboardText = 'set_clipboard_text',
                Open = 'open', Run = 'run', Cmd = 'cmd', GetWinState = 'get_win_state',
                KeysDown = 'keys_down', KeysUp = 'keys_up',
            }
            assert(type(ms) == 'table' and type(WG) == 'table')
            for old, name in pairs(names) do
                assert(type(ms[name]) == 'function', name)
                assert(rawequal(WG[old], ms[name]), old)
            end
            assert(ms.is_win_top == nil and WG.IsWinTop == nil)
            for _, name in ipairs({
                'GetTriggerState', 'SetTriggerState', 'GetExplorerPath',
                'ToggleKeys', 'GetClipboardData', 'SetClipboardData',
            }) do
                assert(type(WG[name]) == 'function', name)
            end
        "#,
        )
        .exec()
        .unwrap();
    }

    #[test]
    fn pressed_sequences_require_the_union_of_keys_and_modifiers() {
        use mousetroke_core::keyseq::parse;
        use RequiredKey::{Modifier, Vk};
        for (sequence, expected) in [
            ("^a", vec![Modifier(ModKey::Ctrl), Vk(0x41)]),
            (
                "^%+#a",
                vec![
                    Modifier(ModKey::Ctrl),
                    Modifier(ModKey::Alt),
                    Modifier(ModKey::Shift),
                    Modifier(ModKey::Win),
                    Vk(0x41),
                ],
            ),
            (
                "^(ab){a 3}",
                vec![Modifier(ModKey::Ctrl), Vk(0x41), Vk(0x42)],
            ),
            (
                "^{+}",
                vec![Modifier(ModKey::Shift), Modifier(ModKey::Ctrl), Vk(0xBB)],
            ),
            ("{RCTRL}a", vec![Vk(0xA3), Vk(0x41)]),
            ("", vec![]),
        ] {
            assert_eq!(
                required_keys(&parse(sequence, false).unwrap()),
                Some(expected),
                "{sequence}"
            );
        }
        let seq = parse("^a", false).unwrap();
        assert!(!keys_pressed(&seq, |vk| vk == 0x41));
        assert!(keys_pressed(&seq, |vk| [0x11, 0x41].contains(&vk)));
        assert!(!keys_pressed(&parse("^ab", false).unwrap(), |vk| [
            0x11, 0x41
        ]
        .contains(&vk)));
        // Win 没有通用 VK，前缀必须分别接受左侧和右侧。
        for side in [0x5B, 0x5C] {
            assert!(keys_pressed(&parse("#a", false).unwrap(), |vk| [
                side, 0x41
            ]
            .contains(&vk)));
        }
        assert!(!keys_pressed(&parse("{RCTRL}a", false).unwrap(), |vk| [
            0xA2, 0x41
        ]
        .contains(&vk)));
        assert!(keys_pressed(&parse("", false).unwrap(), |_| false));
        for sequence in ["a{SLEEP 1}", "{MOUSE_LEFT}", "{MOUSE_WHEEL 120}"] {
            assert!(!keys_pressed(&parse(sequence, false).unwrap(), |_| true));
        }
        assert!(!keys_pressed(&parse("text", true).unwrap(), |_| true));
    }

    #[test]
    fn brightness_legacy_wrappers_alone_keep_negative_failure_values() {
        struct Brightness;
        impl LuaPlatform for Brightness {
            fn log(&self, _: &str) {}
            fn cursor_pos(&self) -> (i32, i32) {
                (0, 0)
            }
            fn install_extensions(&self, lua: &Lua, ms: &Table) -> mlua::Result<()> {
                install_legacy(lua, ms)
            }
            fn get_brightness(&self, x: i32, _: i32) -> Option<i64> {
                (x == 1).then_some(0)
            }
            fn adjust_brightness(&self, _: i32, _: i32, delta: i32) -> Option<i64> {
                (delta >= 0).then_some(i64::from(delta).min(100))
            }
        }
        let lua = Lua::new();
        let platform: std::sync::Arc<dyn LuaPlatform> = std::sync::Arc::new(Brightness);
        mousetroke_lua::install_ms_table(&lua, &LuaCtx::default(), &platform).unwrap();
        lua.load(
            r#"
            assert(ms.get_brightness() == nil)
            assert(WG.GetBrightness() == -1)
            assert(ms.get_brightness({x=1, y=0}) == 0)
            assert(WG.GetBrightness({x=1, y=0}) == 0)
            assert(ms.set_brightness(nil, -1) == nil)
            assert(WG.SetBrightness(nil, -1) == -1)
            for _, value in ipairs({0, 55, 100}) do
                assert(ms.set_brightness(nil, value) == value)
                assert(WG.SetBrightness(nil, value) == value)
            end
        "#,
        )
        .exec()
        .unwrap();
    }
}
