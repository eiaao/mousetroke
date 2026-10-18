//! Lua 5.4 脚本引擎（mlua）—— `MS.*` API 宿主。
//!
//! 入口兼容：优先调用 `ms_action(pos, wid, pid, exe, title, mode)`，
//! 若不存在则回退调用旧格式脚本的入口函数；
//! 并注册旧格式脚本使用的全局别名表——旧脚本里的 API 调用无需修改即可运行。
//!
//! 线程模型：每次手势在专用工作线程创建全新 VM（无状态残留、崩不进钩子线程）；
//! 内存上限 8MB。跨线程操作（暂停）通过托盘窗口消息桥接到引擎线程。

use mlua::{Function, Lua, Table, Value};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};

use crate::input;

/// 脚本执行上下文（来自手势目标与变量）。
#[derive(Debug, Clone)]
pub struct LuaCtx {
    pub pos: (i32, i32),
    pub wid: usize,
    pub pid: u32,
    pub exe: String,
    pub title: String,
    pub mode: String,
}

/// 运行一段 Lua 手势脚本（阻塞，在工作线程调用）。
pub fn run_lua(code: &str, ctx: &LuaCtx) {
    let lua = Lua::new();
    let _ = lua.set_memory_limit(8 * 1024 * 1024);

    if let Err(e) = install_ms_table(&lua, ctx) {
        crate::log!("MS API 注册失败: {e}");
        return;
    }

    let code = code.trim_start_matches('\u{feff}');
    if let Err(e) = lua.load(code).set_name("gesture_script").exec() {
        crate::log!("脚本加载错误: {e}");
        return;
    }

    let globals = lua.globals();
    let entry: Result<Function, _> = globals.get("ms_action");
    let func = match entry {
        Ok(f) => f,
        Err(_) => match globals.get::<Function>("wgAction") {
            // 旧格式脚本兼容入口
            Ok(f) => f,
            Err(_) => {
                crate::log!("脚本缺少入口函数 ms_action");
                return;
            }
        },
    };

    if let Err(e) = call_entry(&lua, &func, ctx) {
        crate::log!("脚本执行错误: {e}");
    }
}

fn call_entry(lua: &Lua, func: &Function, ctx: &LuaCtx) -> mlua::Result<()> {
    let pos = lua.create_table()?;
    pos.set("x", ctx.pos.0)?;
    pos.set("y", ctx.pos.1)?;
    func.call::<()>((pos, ctx.wid as i64, ctx.pid, ctx.exe.clone(), ctx.title.clone(), ctx.mode.clone()))
}

// ---------------------------------------------------------------- MS API

fn install_ms_table(lua: &Lua, ctx: &LuaCtx) -> mlua::Result<()> {
    let ms = lua.create_table()?;
    let _ = ctx; // 部分回调不需要上下文

    // ---- 日志/流程 ----
    ms.set(
        "log",
        lua.create_function(|_l, args: mlua::MultiValue| {
            let vals: Vec<Value> = args.into_iter().collect();
            crate::log!("Lua: {}", format_lua(&vals));
            Ok(())
        })?,
    )?;
    ms.set(
        "sleep",
        lua.create_function(|_l, ms: u64| {
            std::thread::sleep(std::time::Duration::from_millis(ms));
            Ok(())
        })?,
    )?;

    // ---- 暂停/状态（跨线程经托盘窗口消息桥接） ----
    ms.set(
        "is_paused",
        lua.create_function(|_l, ()| Ok(crate::engine::PAUSED.load(std::sync::atomic::Ordering::Relaxed)))?,
    )?;
    ms.set(
        "set_paused",
        lua.create_function(|_l, on: bool| {
            crate::tray::post_engine_cmd(if on { crate::tray::CMD_PAUSE_ON } else { crate::tray::CMD_PAUSE_OFF });
            Ok(())
        })?,
    )?;
    ms.set(
        "show_settings",
        lua.create_function(|_l, ()| {
            crate::tray::open_settings();
            Ok(())
        })?,
    )?;
    ms.set(
        "show_toast",
        lua.create_function(|_l, (msg, duration): (String, Option<u64>)| {
            crate::engine::push_toast(msg, duration.unwrap_or(1200));
            Ok(())
        })?,
    )?;

    // ---- 按键 ----
    ms.set(
        "keys",
        lua.create_function(|_l, (keys, text_by_default): (String, Option<bool>)| {
            match mousetroke_core::keyseq::parse(&keys, text_by_default.unwrap_or(false)) {
                Ok(seq) => {
                    input::inject_keyseq(&seq);
                    Ok(())
                }
                Err(e) => Err(mlua::Error::external(format!("按键序列错误: {e:?}"))) // i18n:allow Lua 运行时错误，面向脚本作者,
            }
        })?,
    )?;
    ms.set(
        "is_keys_pressed",
        lua.create_function(|_l, keys: String| {
            match mousetroke_core::keyseq::parse(&keys, false) {
                Ok(seq) => Ok(seq.actions.iter().all(|a| matches!(a, mousetroke_core::keyseq::Action::Key { key, .. } if input::is_vk_pressed(*key)))),
                Err(_) => Ok(false),
            }
        })?,
    )?;

    // ---- 剪贴板 ----
    ms.set(
        "get_clipboard_text",
        lua.create_function(|_l, ()| Ok(input::get_clipboard_text()))?,
    )?;
    ms.set(
        "set_clipboard_text",
        lua.create_function(|_l, text: String| {
            input::set_clipboard_text(&text);
            Ok(())
        })?,
    )?;

    // ---- 窗口 ----
    ms.set(
        "find_win_by_class",
        lua.create_function(|l, pattern: String| {
            let wins = input::find_windows_by(Some(&pattern), None, true);
            wins_to_table(l, wins)
        })?,
    )?;
    ms.set(
        "find_win_by_title",
        lua.create_function(|l, pattern: String| {
            let wins = input::find_windows_by(None, Some(&pattern), true);
            wins_to_table(l, wins)
        })?,
    )?;
    ms.set(
        "focus_win",
        lua.create_function(|_l, wid: i64| {
            input::focus_window(HWND(wid as *mut core::ffi::c_void));
            Ok(())
        })?,
    )?;
    ms.set(
        "set_win_state",
        lua.create_function(|_l, (wid, state): (i64, String)| {
            input::set_window_state(HWND(wid as *mut core::ffi::c_void), &state);
            Ok(())
        })?,
    )?;
    ms.set(
        "set_win_top",
        lua.create_function(|_l, (wid, top): (i64, bool)| {
            input::set_topmost(HWND(wid as *mut core::ffi::c_void), top);
            Ok(())
        })?,
    )?;

    // ---- 鼠标/屏幕 ----
    ms.set(
        "get_mouse_pos",
        lua.create_function(|l, ()| {
            let (x, y) = input::get_cursor_pos();
            let t = l.create_table()?;
            t.set("x", x)?;
            t.set("y", y)?;
            Ok(t)
        })?,
    )?;
    ms.set(
        "set_mouse_pos",
        lua.create_function(|_l, (x, y): (i32, i32)| {
            input::set_cursor_pos(x, y);
            Ok(())
        })?,
    )?;
    ms.set(
        "get_screen",
        lua.create_function(|l, (x, y): (i32, i32)| {
            let (left, top, right, bottom) = input::screen_of(x, y);
            let t = l.create_table()?;
            t.set("left", left)?;
            t.set("top", top)?;
            t.set("right", right)?;
            t.set("bottom", bottom)?;
            Ok(t)
        })?,
    )?;

    // ---- 文件/进程/URL ----
    ms.set(
        "open",
        lua.create_function(|_l, url: String| {
            input::open_url(&url, None);
            Ok(())
        })?,
    )?;
    ms.set(
        "run",
        lua.create_function(|_l, (file, args): (String, Option<String>)| {
            input::launch_app(&file, args.as_deref(), false, None, None, false, mousetroke_core::schema::IfRunning::RunAgain);
            Ok(())
        })?,
    )?;
    ms.set(
        "cmd",
        lua.create_function(|_l, code: String| {
            input::run_shell(&code);
            Ok(())
        })?,
    )?;
    ms.set(
        "read_text",
        lua.create_function(|_l, path: String| Ok(std::fs::read_to_string(path).ok()))?,
    )?;

    // ---- 亮度 / 窗口状态 / 分段按键 ----
    ms.set(
        "get_brightness",
        lua.create_function(|_l, pos: Option<Table>| {
            let (x, y) = table_pos(pos);
            Ok(crate::brightness::get(x, y).map_or(-1, |v| v as i64))
        })?,
    )?;
    ms.set(
        "set_brightness",
        lua.create_function(|_l, (pos, delta): (Option<Table>, i32)| {
            let (x, y) = table_pos(pos);
            Ok(crate::brightness::adjust(x, y, delta).map_or(-1, |v| v as i64))
        })?,
    )?;
    ms.set(
        "get_win_state",
        lua.create_function(|_l, wid: i64| {
            use windows::Win32::UI::WindowsAndMessaging::{IsIconic, IsZoomed};
            let h = HWND(wid as *mut core::ffi::c_void);
            Ok(unsafe {
                if IsIconic(h).as_bool() {
                    "min"
                } else if IsZoomed(h).as_bool() {
                    "max"
                } else {
                    "normal"
                }
            })
        })?,
    )?;
    for (name, phase) in [("keys_down", input::KeyPhase::Down), ("keys_up", input::KeyPhase::Up)] {
        ms.set(
            name,
            lua.create_function(move |_l, keys: String| match mousetroke_core::keyseq::parse(&keys, false) {
                Ok(seq) => {
                    input::inject_keyseq_phase(&seq, phase);
                    Ok(())
                }
                Err(e) => Err(mlua::Error::external(format!("按键序列错误: {e:?}"))) // i18n:allow Lua 运行时错误，面向脚本作者,
            })?,
        )?;
    }

    let _ = lua.globals().set("ms", ms.clone());

    // 兼容表：旧格式 PascalCase API 名映射到 snake_case 实现
    const LEGACY_NAMES: &[(&str, &str)] = &[
        ("Log", "log"),
        ("Sleep", "sleep"),
        ("IsPaused", "is_paused"),
        ("SetPaused", "set_paused"),
        ("ShowSettings", "show_settings"),
        ("ShowToast", "show_toast"),
        ("IsWinTop", "is_win_top"),
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
        ("GetBrightness", "get_brightness"),
        ("SetBrightness", "set_brightness"),
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
    // 旧格式脚本能引用但暂未实现的方法 → 记录日志的空操作
    for stub in ["GetTriggerState", "SetTriggerState", "GetExplorerPath", "ToggleKeys",
                 "GetClipboardData", "SetClipboardData"] {
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
    for stub in ["GetWindowText", "MessageBox", "PostMessage", "GetForegroundWindow"] {
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
    let _ = luanet.set(
        "enum",
        lua.create_function(|_l, (_t, name): (Value, String)| Ok(name))?,
    )?;
    let _ = lua.globals().set("luanet", luanet);
    let _ = lua.globals().set(
        "import",
        lua.create_function(|_l, _m: String| Ok(()))?,
    )?;
    Ok(())
}

/// printf 风格格式化（%s/%d/%f/%q/%%），首参为格式串；单参直接打印。
fn format_lua(vals: &[Value]) -> String {
    if vals.len() >= 2 {
        if let Some(Value::String(fmt)) = vals.first() {
            let fmt = fmt.to_string_lossy().to_string();
            let rest = &vals[1..];
            let mut out = String::with_capacity(fmt.len());
            let mut it = rest.iter();
            let mut chars = fmt.chars().peekable();
            while let Some(c) = chars.next() {
                if c != '%' {
                    out.push(c);
                    continue;
                }
                match chars.next() {
                    Some('%') => out.push('%'),
                    Some(spec) if "sdfq".contains(spec) => {
                        let rendered = match (it.next(), spec) {
                            (Some(Value::String(s)), _) => s.to_string_lossy().to_string(),
                            (Some(Value::Integer(i)), 'f') => format!("{i:.6}"),
                            (Some(Value::Integer(i)), _) => i.to_string(),
                            (Some(Value::Number(n)), 'f') => format!("{n:.6}"),
                            (Some(Value::Number(n)), _) => n.to_string(),
                            (Some(Value::Boolean(b)), _) => b.to_string(),
                            (Some(Value::Nil), _) => "nil".into(),
                            (other, _) => format!("{other:?}"),
                        };
                        out.push_str(&rendered);
                    }
                    Some(other) => {
                        out.push('%');
                        out.push(other);
                    }
                    None => out.push('%'),
                }
            }
            return out;
        }
    }
    vals.iter()
        .map(|v| match v {
            Value::String(s) => s.to_string_lossy().to_string(),
            Value::Integer(i) => i.to_string(),
            Value::Number(n) => n.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Nil => "nil".into(),
            other => format!("{other:?}"),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// 旧格式 pos 参数是 {x=, y=} 表；缺省用当前鼠标位置。
fn table_pos(pos: Option<Table>) -> (i32, i32) {
    pos.and_then(|t| Some((t.get::<i32>("x").ok()?, t.get::<i32>("y").ok()?)))
        .unwrap_or_else(input::get_cursor_pos)
}

fn wins_to_table(lua: &Lua, wins: Vec<(String, isize)>) -> mlua::Result<Table> {
    let t = lua.create_table()?;
    for (i, (text, hwnd)) in wins.into_iter().enumerate() {
        let item = lua.create_table()?;
        item.set("text", text)?;
        item.set("hwnd", hwnd)?;
        t.set(i + 1, item)?;
    }
    Ok(t)
}

/// 编译期消息参数检查。
#[allow(dead_code)]
fn _wp_lp_check(w: WPARAM, l: LPARAM) -> (WPARAM, LPARAM) {
    (w, l)
}
