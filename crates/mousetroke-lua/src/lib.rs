//! 共享 Lua 5.4 运行时；每个工作线程创建独立 VM，保留 Windows 原有库和内存限制。
// i18n:allow-begin Lua 诊断仅写运行日志，按键语法错误面向脚本作者
use mlua::{Function, Lua, Table, Value};
use std::sync::Arc;
mod platform;
use mousetroke_core::keyseq::{Action, Key, KeySeq, ModKey};
pub use platform::{KeyPhase, LuaPlatform};

/// 按键查询的要求；前缀修饰键接受任意一侧，显式左右键保留侧别。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequiredKey {
    Vk(u16),
    Modifier(ModKey),
}

/// 多步序列取所有键和修饰键的并集；非键盘动作不能查询按下状态。
pub fn required_keys(seq: &KeySeq) -> Option<Vec<RequiredKey>> {
    let mut required = Vec::new();
    for action in &seq.actions {
        let Action::Key {
            key: Key::Vk(vk),
            mods,
            ..
        } = action
        else {
            return None;
        };
        for key in mods
            .iter()
            .copied()
            .map(RequiredKey::Modifier)
            .chain(std::iter::once(RequiredKey::Vk(*vk)))
        {
            if !required.contains(&key) {
                required.push(key);
            }
        }
    }
    Some(required)
}

fn callback<A: mlua::FromLuaMulti, R: mlua::IntoLuaMulti>(
    lua: &Lua,
    platform: &Arc<dyn LuaPlatform>,
    f: impl Fn(&dyn LuaPlatform, &Lua, A) -> mlua::Result<R> + 'static,
) -> mlua::Result<Function> {
    let platform = platform.clone();
    lua.create_function(move |lua, args| f(platform.as_ref(), lua, args))
}

/// 脚本执行上下文（来自手势目标与变量）。
#[derive(Debug, Clone, Default)]
pub struct LuaCtx {
    pub pos: (i32, i32),
    pub wid: usize,
    pub pid: u32,
    pub exe: String,
    pub title: String,
    pub mode: String,
}

impl LuaCtx {
    /// 由手势目标与动作变量组装；缺失的变量取空串。
    pub fn from_vars(
        pos: (i32, i32),
        wid: usize,
        pid: u32,
        vars: &mousetroke_core::vars::Vars,
    ) -> Self {
        let get = |key: &str| vars.get(key).cloned().unwrap_or_default();
        LuaCtx {
            pos,
            wid,
            pid,
            exe: get("MS_EXE"),
            title: get("MS_WIN_NAME"),
            mode: get("MS_TARGET_MODE"),
        }
    }
}

/// 运行一段 Lua 手势脚本（阻塞，在工作线程调用）。
pub fn run_lua(code: &str, ctx: &LuaCtx, platform: Arc<dyn LuaPlatform>) {
    let lua = Lua::new();
    run_lua_in(&lua, code, ctx, &platform);
}

fn run_lua_in(lua: &Lua, code: &str, ctx: &LuaCtx, platform: &Arc<dyn LuaPlatform>) {
    let _ = lua.set_memory_limit(8 * 1024 * 1024);

    if let Err(e) = install_ms_table(lua, ctx, platform) {
        platform.log(&format!("MS API 注册失败: {e}"));
        return;
    }

    let code = code.trim_start_matches('\u{feff}');
    if let Err(e) = lua.load(code).set_name("gesture_script").exec() {
        platform.log(&format!("脚本加载错误: {e}"));
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
                platform.log("脚本缺少入口函数 ms_action");
                return;
            }
        },
    };

    if let Err(e) = call_entry(lua, &func, ctx) {
        platform.log(&format!("脚本执行错误: {e}"));
    }
}

fn call_entry(lua: &Lua, func: &Function, ctx: &LuaCtx) -> mlua::Result<()> {
    let pos = lua.create_table()?;
    pos.set("x", ctx.pos.0)?;
    pos.set("y", ctx.pos.1)?;
    func.call::<()>((
        pos,
        ctx.wid as i64,
        ctx.pid,
        ctx.exe.clone(),
        ctx.title.clone(),
        ctx.mode.clone(),
    ))
}

// mlua 0.12.2 拒绝小数转整数；旧脚本仍按 0.10 的约定截断，并保留范围检查。
pub struct LuaInt<T>(pub T);

impl<T: mlua::FromLua> mlua::FromLua for LuaInt<T> {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        match T::from_lua(value.clone(), lua) {
            Ok(n) => Ok(Self(n)),
            Err(error) => match lua.coerce_number(value)? {
                Some(n) if n.is_finite() && n.fract() != 0.0 => {
                    T::from_lua(Value::Number(n.trunc()), lua)
                        .map(Self)
                        .map_err(|_| error)
                }
                _ => Err(error),
            },
        }
    }
}

// ---------------------------------------------------------------- MS API

pub fn install_ms_table(
    lua: &Lua,
    ctx: &LuaCtx,
    platform: &Arc<dyn LuaPlatform>,
) -> mlua::Result<()> {
    let ms = lua.create_table()?;
    let _ = ctx; // 部分回调不需要上下文

    // ---- 日志/流程 ----
    ms.set(
        "log",
        callback(lua, platform, |platform, _l, args: mlua::MultiValue| {
            let vals: Vec<Value> = args.into_iter().collect();
            platform.log(&format!("Lua: {}", format_lua(&vals)));
            Ok(())
        })?,
    )?;
    ms.set(
        "sleep",
        callback(lua, platform, |platform, _l, LuaInt(ms): LuaInt<u64>| {
            platform.sleep(ms);
            Ok(())
        })?,
    )?;

    // ---- 暂停/状态（跨线程经托盘窗口消息桥接） ----
    ms.set(
        "is_paused",
        callback(lua, platform, |platform, _l, ()| Ok(platform.is_paused()))?,
    )?;
    ms.set(
        "set_paused",
        callback(lua, platform, |platform, _l, on: bool| {
            platform.set_paused(on);
            Ok(())
        })?,
    )?;
    ms.set(
        "show_settings",
        callback(lua, platform, |platform, _l, ()| {
            platform.show_settings();
            Ok(())
        })?,
    )?;
    ms.set(
        "show_toast",
        callback(
            lua,
            platform,
            |platform, _l, (msg, duration): (String, Option<LuaInt<u64>>)| {
                platform.show_toast(msg, duration.map(|v| v.0).unwrap_or(1200));
                Ok(())
            },
        )?,
    )?;

    // ---- 按键 ----
    ms.set(
        "keys",
        callback(
            lua,
            platform,
            |platform, _l, (keys, text_by_default): (String, Option<bool>)| {
                match mousetroke_core::keyseq::parse(&keys, text_by_default.unwrap_or(false)) {
                    Ok(seq) => {
                        platform.send_keys(&seq, KeyPhase::DownUp);
                        Ok(())
                    }
                    Err(e) => Err(mlua::Error::external(format!("按键序列错误: {e:?}"))), // i18n:allow Lua 运行时错误，面向脚本作者,
                }
            },
        )?,
    )?;
    ms.set(
        "is_keys_pressed",
        callback(
            lua,
            platform,
            |platform, _l, keys: String| match mousetroke_core::keyseq::parse(&keys, false) {
                Ok(seq) => Ok(platform.is_keys_pressed(&seq)),
                Err(_) => Ok(Some(false)),
            },
        )?,
    )?;

    // ---- 剪贴板 ----
    ms.set(
        "get_clipboard_text",
        callback(lua, platform, |platform, _l, ()| {
            Ok(platform.get_clipboard_text())
        })?,
    )?;
    ms.set(
        "set_clipboard_text",
        callback(lua, platform, |platform, _l, text: String| {
            platform.set_clipboard_text(&text);
            Ok(())
        })?,
    )?;

    // ---- 窗口 ----
    ms.set(
        "find_win_by_class",
        callback(lua, platform, |platform, l, pattern: String| {
            let wins = platform.find_windows(Some(&pattern), None);
            wins.map(|wins| wins_to_table(l, wins)).transpose()
        })?,
    )?;
    ms.set(
        "find_win_by_title",
        callback(lua, platform, |platform, l, pattern: String| {
            let wins = platform.find_windows(None, Some(&pattern));
            wins.map(|wins| wins_to_table(l, wins)).transpose()
        })?,
    )?;
    ms.set(
        "focus_win",
        callback(lua, platform, |platform, _l, LuaInt(wid): LuaInt<i64>| {
            platform.focus_window(wid);
            Ok(())
        })?,
    )?;
    ms.set(
        "set_win_state",
        callback(
            lua,
            platform,
            |platform, _l, (LuaInt(wid), state): (LuaInt<i64>, String)| {
                platform.set_window_state(wid, &state);
                Ok(())
            },
        )?,
    )?;
    ms.set(
        "set_win_top",
        callback(
            lua,
            platform,
            |platform, _l, (LuaInt(wid), top): (LuaInt<i64>, bool)| {
                platform.set_topmost(wid, top);
                Ok(())
            },
        )?,
    )?;

    // ---- 鼠标/屏幕 ----
    ms.set(
        "get_mouse_pos",
        callback(lua, platform, |platform, l, ()| {
            let (x, y) = platform.cursor_pos();
            let t = l.create_table()?;
            t.set("x", x)?;
            t.set("y", y)?;
            Ok(t)
        })?,
    )?;
    ms.set(
        "set_mouse_pos",
        callback(
            lua,
            platform,
            |platform, _l, (LuaInt(x), LuaInt(y)): (LuaInt<i32>, LuaInt<i32>)| {
                platform.set_cursor(x, y);
                Ok(())
            },
        )?,
    )?;
    ms.set(
        "get_screen",
        callback(
            lua,
            platform,
            |platform, l, (LuaInt(x), LuaInt(y)): (LuaInt<i32>, LuaInt<i32>)| {
                let Some((left, top, right, bottom)) = platform.screen_of(x, y) else {
                    return Ok(None);
                };
                let t = l.create_table()?;
                t.set("left", left)?;
                t.set("top", top)?;
                t.set("right", right)?;
                t.set("bottom", bottom)?;
                Ok(Some(t))
            },
        )?,
    )?;

    // ---- 文件/进程/URL ----
    ms.set(
        "open",
        callback(lua, platform, |platform, _l, url: String| {
            platform.open_url(&url);
            Ok(())
        })?,
    )?;
    ms.set(
        "run",
        callback(
            lua,
            platform,
            |platform, _l, (file, args): (String, Option<String>)| {
                platform.launch_app(&file, args.as_deref());
                Ok(())
            },
        )?,
    )?;
    ms.set(
        "cmd",
        callback(lua, platform, |platform, _l, code: String| {
            platform.run_shell(&code);
            Ok(())
        })?,
    )?;
    ms.set(
        "read_text",
        callback(lua, platform, |_platform, _l, path: String| {
            Ok(std::fs::read_to_string(path).ok())
        })?,
    )?;

    // ---- 亮度 / 窗口状态 / 分段按键 ----
    ms.set(
        "get_brightness",
        callback(lua, platform, |platform, _l, pos: Option<Table>| {
            let (x, y) = table_pos(pos, || platform.cursor_pos());
            Ok(platform.get_brightness(x, y))
        })?,
    )?;
    ms.set(
        "set_brightness",
        callback(
            lua,
            platform,
            |platform, _l, (pos, LuaInt(delta)): (Option<Table>, LuaInt<i32>)| {
                let (x, y) = table_pos(pos, || platform.cursor_pos());
                Ok(platform.adjust_brightness(x, y, delta))
            },
        )?,
    )?;
    ms.set(
        "get_win_state",
        callback(lua, platform, |platform, _l, LuaInt(wid): LuaInt<i64>| {
            Ok(platform.window_state(wid))
        })?,
    )?;
    for (name, phase) in [("keys_down", KeyPhase::Down), ("keys_up", KeyPhase::Up)] {
        ms.set(
            name,
            callback(lua, platform, move |platform, _l, keys: String| {
                match mousetroke_core::keyseq::parse(&keys, false) {
                    Ok(seq) => {
                        platform.send_keys(&seq, phase);
                        Ok(())
                    }
                    Err(e) => Err(mlua::Error::external(format!("按键序列错误: {e:?}"))), // i18n:allow Lua 运行时错误，面向脚本作者,
                }
            })?,
        )?;
    }

    let _ = lua.globals().set("ms", ms.clone());

    platform.install_extensions(lua, &ms)?;
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
pub fn table_pos(pos: Option<Table>, cursor: impl FnOnce() -> (i32, i32)) -> (i32, i32) {
    pos.and_then(|t| {
        Some((
            t.get::<LuaInt<i32>>("x").ok()?.0,
            t.get::<LuaInt<i32>>("y").ok()?.0,
        ))
    })
    .unwrap_or_else(cursor)
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

#[cfg(test)]
mod tests {
    use super::*;
    struct FakePlatform;
    impl LuaPlatform for FakePlatform {
        fn log(&self, _: &str) {}
        fn cursor_pos(&self) -> (i32, i32) {
            (0, 0)
        }
    }
    fn platform() -> Arc<dyn LuaPlatform> {
        Arc::new(FakePlatform)
    }

    fn context() -> LuaCtx {
        LuaCtx {
            pos: (-120, 345),
            wid: 0x1234,
            pid: 42,
            exe: "test.exe".into(),
            title: "测试窗口".into(),
            mode: "gesture".into(),
        }
    }

    fn run_script(code: &str) -> Lua {
        let lua = Lua::new();
        run_lua_in(&lua, code, &context(), &platform());
        lua
    }

    #[test]
    fn memory_limit_stops_allocating_script_before_entry() {
        let lua = run_script("before = true; local huge = string.rep('x', 9 * 1024 * 1024); function ms_action() called = true end");
        assert!(lua.globals().get::<bool>("before").unwrap());
        assert!(matches!(
            lua.globals().get::<Value>("called").unwrap(),
            Value::Nil
        ));
    }

    #[test]
    fn platform_calls_preserve_text_mode_positions_and_nil_fallbacks() {
        #[derive(Default)]
        struct Recording(std::sync::Mutex<Vec<String>>);
        impl LuaPlatform for Recording {
            fn log(&self, _: &str) {}
            fn cursor_pos(&self) -> (i32, i32) {
                (-100, 200)
            }
            fn set_cursor(&self, x: i32, y: i32) {
                self.0.lock().unwrap().push(format!("cursor:{x},{y}"));
            }
            fn send_keys(&self, seq: &mousetroke_core::keyseq::KeySeq, _: KeyPhase) {
                assert!(
                    matches!(&seq.actions[0], mousetroke_core::keyseq::Action::Text(t) if t == "中文")
                );
                self.0.lock().unwrap().push("keys".into());
            }
            fn show_toast(&self, msg: String, ms: u64) {
                self.0.lock().unwrap().push(format!("toast:{msg}:{ms}"));
            }
            fn get_clipboard_text(&self) -> Option<String> {
                Some("clipboard".into())
            }
        }
        let platform = Arc::new(Recording::default());
        let lua = Lua::new();
        let adapter: Arc<dyn LuaPlatform> = platform.clone();
        install_ms_table(&lua, &context(), &adapter).unwrap();
        lua.load(
            r#"
            assert(WG == nil and User32 == nil and luanet == nil and import == nil)
            local pos = ms.get_mouse_pos()
            assert(pos.x == -100 and pos.y == 200)
            assert(ms.get_clipboard_text() == 'clipboard')
            ms.set_mouse_pos(-12.75, 3.25)
            ms.keys('中文', true)
            ms.show_toast('message', 42.75)
            assert(ms.get_brightness() == nil and ms.set_brightness(nil, 5) == nil)
            assert(ms.get_win_state(1) == nil)
            assert(ms.find_win_by_class('x') == nil and ms.find_win_by_title('x') == nil)
            assert(ms.is_keys_pressed('a') == nil)
        "#,
        )
        .exec()
        .unwrap();
        assert_eq!(
            *platform.0.lock().unwrap(),
            ["cursor:-12,3", "keys", "toast:message:42"]
        );
    }

    #[test]
    fn entries_receive_context_and_run_once() {
        for entry in ["ms_action", "wgAction"] {
            let lua = run_script(&format!(
                r#"
                calls = 0
                function {entry}(...)
                    calls = calls + 1
                    argc = select('#', ...)
                    args = {{...}}
                    return "ignored", 123
                end
            "#
            ));
            let globals = lua.globals();
            assert_eq!(globals.get::<i32>("calls").unwrap(), 1);
            assert_eq!(globals.get::<i32>("argc").unwrap(), 6);
            let args: Table = globals.get("args").unwrap();
            let pos: Table = args.get(1).unwrap();
            let ctx = context();
            assert_eq!(table_pos(Some(pos), || (0, 0)), ctx.pos);
            assert_eq!(args.get::<i64>(2).unwrap(), ctx.wid as i64);
            assert_eq!(args.get::<u32>(3).unwrap(), ctx.pid);
            assert_eq!(args.get::<String>(4).unwrap(), ctx.exe);
            assert_eq!(args.get::<String>(5).unwrap(), ctx.title);
            assert_eq!(args.get::<String>(6).unwrap(), ctx.mode);
        }
    }

    #[test]
    fn primary_entry_takes_precedence() {
        let lua = run_script(
            r#"
            function ms_action() selected = "ms" end
            function wgAction() selected = "WG" end
        "#,
        );
        assert_eq!(lua.globals().get::<String>("selected").unwrap(), "ms");
    }

    #[test]
    fn non_function_primary_entry_uses_legacy_fallback() {
        let lua = run_script(
            r#"
            ms_action = false
            function wgAction() called = true end
        "#,
        );
        assert!(lua.globals().get::<bool>("called").unwrap());
    }

    #[test]
    fn utf8_bom_is_ignored() {
        let lua = run_script("\u{feff}function ms_action() called = true end");
        assert!(lua.globals().get::<bool>("called").unwrap());
    }

    #[test]
    fn syntax_error_is_ignored_without_running_script() {
        let code = "loaded = true; function ms_action(";
        run_lua(code, &context(), platform());
        let lua = run_script(code);
        assert!(matches!(
            lua.globals().get::<Value>("loaded").unwrap(),
            Value::Nil
        ));
    }

    #[test]
    fn load_runtime_error_prevents_entry_call() {
        let lua = run_script(
            r#"
            loaded = true
            function ms_action() called = true end
            error('expected load failure')
        "#,
        );
        assert!(lua.globals().get::<bool>("loaded").unwrap());
        assert!(matches!(
            lua.globals().get::<Value>("called").unwrap(),
            Value::Nil
        ));
    }

    #[test]
    fn entry_runtime_errors_are_ignored() {
        for entry in ["ms_action", "wgAction"] {
            let code = format!(
                r#"
                function {entry}()
                    started = true
                    error('expected entry failure')
                    finished = true
                end
            "#
            );
            run_lua(&code, &context(), platform());
            let lua = run_script(&code);
            assert!(lua.globals().get::<bool>("started").unwrap());
            assert!(matches!(
                lua.globals().get::<Value>("finished").unwrap(),
                Value::Nil
            ));
        }
    }

    #[test]
    fn missing_entry_is_ignored() {
        let lua = run_script("loaded = true");
        assert!(lua.globals().get::<bool>("loaded").unwrap());
    }

    #[test]
    fn read_only_ms_helpers_return_expected_values() {
        let lua = Lua::new();
        install_ms_table(&lua, &context(), &platform()).unwrap();
        let paused: bool = lua.load("return ms.is_paused()").eval().unwrap();
        assert!(!paused);

        let read: Function = lua
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
        let missing = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml/missing");
        assert!(matches!(read.call::<Value>(missing).unwrap(), Value::Nil));
    }

    #[test]
    fn explicit_positions_preserve_numeric_conversions() {
        let lua = Lua::new();
        for (code, expected) in [
            ("{x = -120, y = 345}", (-120, 345)),
            ("{x = 12.75, y = -3.25}", (12, -3)),
            ("{x = '12.75', y = '-3.25'}", (12, -3)),
            ("{x = -2147483648, y = 2147483647}", (i32::MIN, i32::MAX)),
        ] {
            let pos: Table = lua.load(code).eval().unwrap();
            assert_eq!(table_pos(Some(pos), || (0, 0)), expected);
        }
    }

    #[test]
    fn integer_callback_arguments_preserve_coercion_and_bounds() {
        let lua = Lua::new();
        // 用无副作用的回调固定坐标、窗口句柄和可选时长的转换约定。
        let echo = lua
            .create_function(
                |_l,
                 (LuaInt(x), LuaInt(wid), duration): (
                    LuaInt<i32>,
                    LuaInt<i64>,
                    Option<LuaInt<u64>>,
                )| { Ok((x, wid, duration.map(|v| v.0))) },
            )
            .unwrap();
        lua.globals().set("echo", echo).unwrap();
        for (code, expected) in [
            ("return echo(12.75, -3.25, 4.75)", (12, -3, Some(4))),
            ("return echo('12.75', '-3.25', '4.75')", (12, -3, Some(4))),
            (
                "return echo(-2147483648, math.maxinteger)",
                (i32::MIN, i64::MAX, None),
            ),
            (
                "return echo(2147483647, math.mininteger, -0.5)",
                (i32::MAX, i64::MIN, Some(0)),
            ),
            (
                "return echo(0, 9007199254740993, 18446744073709549568.0)",
                (0, 9007199254740993, Some(18446744073709549568)),
            ),
        ] {
            assert_eq!(
                lua.load(code).eval::<(i32, i64, Option<u64>)>().unwrap(),
                expected
            );
        }
        for args in [
            "2147483648, 0",
            "-2147483649, 0",
            "0, 9223372036854775808.0",
            "0, 0, -1",
            "0, 0, 18446744073709551616.0",
            "0/0, 0",
            "math.huge, 0",
            "false, 0",
            "'invalid', 0",
            "nil, 0",
        ] {
            assert!(
                lua.load(format!("return echo({args})"))
                    .eval::<Value>()
                    .is_err(),
                "{args}"
            );
        }
    }

    #[test]
    fn window_results_are_one_based_tables() {
        let lua = Lua::new();
        let wins =
            wins_to_table(&lua, vec![("first".into(), 123), ("second".into(), -456)]).unwrap();
        assert_eq!(wins.raw_len(), 2);
        assert!(matches!(wins.get::<Value>(0).unwrap(), Value::Nil));
        let first: Table = wins.get(1).unwrap();
        assert_eq!(first.get::<String>("text").unwrap(), "first");
        assert_eq!(first.get::<isize>("hwnd").unwrap(), 123);
        let second: Table = wins.get(2).unwrap();
        assert_eq!(second.get::<String>("text").unwrap(), "second");
        assert_eq!(second.get::<isize>("hwnd").unwrap(), -456);
        assert_eq!(wins_to_table(&lua, Vec::new()).unwrap().raw_len(), 0);
    }

    #[test]
    fn log_formatting_preserves_multi_value_order_without_logging() {
        let lua = Lua::new();
        let vals: mlua::MultiValue = lua
            .load(r#"return '%s %d %f %q %%', 'sample', 7, 1.25, 'quoted'"#)
            .eval()
            .unwrap();
        assert_eq!(
            format_lua(&vals.into_iter().collect::<Vec<_>>()),
            "sample 7 1.250000 quoted %"
        );
        let vals: mlua::MultiValue = lua.load("return 7, true, nil, 'last'").eval().unwrap();
        assert_eq!(
            format_lua(&vals.into_iter().collect::<Vec<_>>()),
            "7 true nil last"
        );
        assert_eq!(format_lua(&[]), "");
    }
}

// i18n:allow-end
