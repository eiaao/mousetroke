//! Mousetroke 主程序。
//!
//! 用法：
//! - `mousetroke import <gestures.json> [prefs.json] [-o 输出路径]`
//!   把旧格式的手势配置转换为原生格式（默认输出到 %APPDATA%\mousetroke\config.json）
//! - `mousetroke [run] [--config 路径]`
//!   加载配置并启动手势引擎（无参数即 run，便于双击/开机自启）
//! - `mousetroke settings [配置路径]`
//!   打开设置界面（独立进程：常驻的引擎进程里没有 WebView）
//!
//! 输出语言：读到配置就用配置里的设置，否则用系统语言（docs/I18N-DESIGN.md §3.4）。
//! release 构建为 GUI 子系统（常驻不弹黑框）；从终端启动时挂接父控制台输出日志。

use std::path::PathBuf;

use mousetroke_core::i18n::{keys, t, tf};
use mousetroke_core::schema::{default_config_path, Config};

pub(super) fn main() {
    mousetroke_win::attach_parent_console();
    init_lang();
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        None => cmd_run(&[]),
        Some("--config") => cmd_run(&args),
        Some("--after-update") => {
            let pids = args.get(1..3).and_then(|p| p.iter().map(|s| s.parse::<u32>().ok()).collect::<Option<Vec<_>>>());
            if let Some(pids) = pids {
                if mousetroke_win::update::wait_for_previous(&pids).is_ok() { cmd_run(&args[3..]); }
                else { eprintln!("{}", t(keys::UPDATE_ERROR_RESTART)); }
            }
        }
        Some("import") => cmd_import(&args[1..]),
        Some("run") => cmd_run(&args[1..]),
        Some("autostart") => cmd_autostart(&args[1..]),
        Some("lua") => cmd_lua(&args[1..]),
        Some("settings") | Some("--settings") => mousetroke_settings::run(args.get(1).filter(|s| !s.starts_with("--")).map(PathBuf::from)),
        Some("diag") => cmd_diag(),
        Some("-h") | Some("--help") => print_help(),
        Some(other) => {
            eprintln!("{}", tf(keys::CLI_UNKNOWN_COMMAND, &[("cmd", other)]));
            print_help();
            std::process::exit(2);
        }
    }
}

/// 命令行输出语言：配置里的设置优先，否则跟随系统（不阻塞：读不到配置就用系统语言）。
fn init_lang() {
    let setting = std::fs::read_to_string(default_config_path())
        .ok()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(text.trim_start_matches('\u{feff}')).ok())
        .and_then(|v| {
            v.get("settings").and_then(|s| s.get("language")).and_then(|l| l.as_str()).map(str::to_string)
        })
        .unwrap_or_default();
    let langs = mousetroke_win::system_langs();
    mousetroke_core::i18n::set_lang(mousetroke_core::i18n::resolve(&setting, &langs));
}

fn print_help() {
    println!("{}", tf(keys::CLI_HELP, &[("version", env!("CARGO_PKG_VERSION"))]));
}

fn cmd_import(args: &[String]) {
    let mut gestures: Option<PathBuf> = None;
    let mut prefs: Option<PathBuf> = None;
    let mut out = default_config_path();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-o" => {
                i += 1;
                out = PathBuf::from(args.get(i).cloned().unwrap_or_default());
            }
            p if p.ends_with("prefs.json") => prefs = Some(PathBuf::from(p)),
            p => gestures = Some(PathBuf::from(p)),
        }
        i += 1;
    }
    let Some(gestures) = gestures else {
        eprintln!("{}", t(keys::CLI_IMPORT_MISSING_ARG));
        std::process::exit(2);
    };

    let read = |p: &PathBuf| -> String {
        std::fs::read_to_string(p).unwrap_or_else(|e| {
            eprintln!("{}", tf(keys::ERROR_READ_FILE, &[("path", &p.display().to_string()), ("err", &e.to_string())]));
            std::process::exit(1);
        })
    };
    let g = read(&gestures);
    let p = prefs.map(|path| read(&path));

    let cfg = mousetroke_core::legacy::import(&g, p.as_deref())
        .unwrap_or_else(|e| {
            eprintln!("{}", tf(keys::CLI_IMPORT_FAILED, &[("err", &e)]));
            std::process::exit(1);
        });
    if let Err(e) = cfg.save(&out) {
        eprintln!("{e}");
        std::process::exit(1);
    }
    println!(
        "{}",
        tf(keys::CLI_IMPORT_DONE, &[
            ("scopes", &cfg.scopes.len().to_string()),
            ("bindings", &cfg.all_bindings().count().to_string()),
            ("path", &out.display().to_string()),
        ])
    );
}

/// 直接运行一段 Lua 手势脚本（测试用）。
fn cmd_lua(args: &[String]) {
    let Some(path) = args.first() else {
        eprintln!("{}", t(keys::CLI_LUA_USAGE));
        std::process::exit(2);
    };
    let code = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("{}", tf(keys::ERROR_READ_FILE, &[("path", path), ("err", &e.to_string())]));
        std::process::exit(1);
    });
    let ctx = mousetroke_win::lua::LuaCtx {
        pos: (0, 0),
        wid: 0,
        pid: 0,
        exe: "console".into(),
        title: "lua-test".into(),
        mode: "UnderCursor".into(),
    };
    mousetroke_win::lua::run_lua(&code, &ctx);
}

fn cmd_autostart(args: &[String]) {
    match args.first().map(String::as_str) {
        Some("on") => match mousetroke_win::autostart::set(true) {
            Ok(_) => println!("{}", t(keys::CLI_AUTOSTART_ON)),
            Err(e) => {
                eprintln!("{}", tf(keys::ERROR_AUTOSTART, &[("err", &e)]));
                std::process::exit(1);
            }
        },
        Some("off") => match mousetroke_win::autostart::set(false) {
            Ok(_) => println!("{}", t(keys::CLI_AUTOSTART_OFF)),
            Err(e) => {
                eprintln!("{}", tf(keys::ERROR_AUTOSTART, &[("err", &e)]));
                std::process::exit(1);
            }
        },
        _ => println!(
            "{}",
            tf(keys::CLI_AUTOSTART_STATUS, &[(
                "state",
                t(if mousetroke_win::autostart::is_enabled() { keys::CLI_STATE_ON } else { keys::CLI_STATE_OFF })
            )])
        ),
    }
}

fn cmd_run(args: &[String]) {
    let mut path = default_config_path();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--config" {
            i += 1;
            path = PathBuf::from(args.get(i).cloned().unwrap_or_default());
        }
        i += 1;
    }
    let cfg = match Config::load_existing(&path) {
        Ok(Some(cfg)) => cfg,
        // 首次运行：写入内置默认手势，设置界面打开时读到的也是这一份
        Ok(None) => {
            let cfg = Config::builtin();
            if let Err(e) = cfg.save(&path) {
                eprintln!("{}", tf(keys::CLI_RUN_LOAD_FAILED, &[("err", &e)]));
            }
            cfg
        }
        Err(e) => {
            eprintln!("{}", tf(keys::CLI_RUN_LOAD_FAILED, &[("err", &e)]));
            std::process::exit(1);
        }
    };
    println!(
        "{}",
        tf(keys::CLI_RUN_LOADED, &[
            ("scopes", &cfg.scopes.len().to_string()),
            ("bindings", &cfg.all_bindings().count().to_string()),
            ("path", &path.display().to_string()),
        ])
    );
    if let Err(e) = mousetroke_win::run(cfg, Some(path)) {
        eprintln!("{}", tf(keys::CLI_RUN_FAILED, &[("err", &e)]));
        std::process::exit(1);
    }
}

/// 诊断信息：排查"手势不触发/弹出右键菜单"等问题。
fn cmd_diag() {
    use mousetroke_win::targeting;
    let yn = |b: bool| t(if b { keys::CLI_DIAG_YES } else { keys::CLI_DIAG_NO });
    println!("{}", tf(keys::CLI_DIAG_ENGINE_RUNNING, &[("state", yn(mousetroke_win::engine_running()))]));
    // 这些工具也会截取/重发鼠标事件；Mousetroke 能识别它们重发的回声，列出来便于排查
    let tools = ["uTools.exe", "Quicker.exe", "AutoHotkey.exe", "AutoHotkey64.exe", "XMouseButtonControl.exe", "StrokesPlus.net.exe", "PowerToys.MouseWithoutBorders.exe"];
    let running: Vec<&str> = tools.into_iter().filter(|tool| targeting::process_running(tool)).collect();
    let tools_str = if running.is_empty() {
        t(keys::CLI_DIAG_NONE).to_string()
    } else {
        running.join(t(keys::CLI_DIAG_TOOLS_SEP))
    };
    println!("{}", tf(keys::CLI_DIAG_TOOLS, &[("tools", &tools_str)]));
    println!("{}", tf(keys::CLI_DIAG_LOG, &[("path", &mousetroke_win::log::path().display().to_string())]));
    let (x, y) = mousetroke_win::input::get_cursor_pos();
    let target = targeting::resolve_under_cursor(x, y);
    println!(
        "{}",
        tf(keys::CLI_DIAG_UNDER_CURSOR, &[
            ("x", &x.to_string()),
            ("y", &y.to_string()),
            ("class", &target.class),
            ("exe", target.exe.as_deref().unwrap_or("-")),
            ("desktop", yn(target.is_desktop)),
            ("shell", yn(target.is_shell_ui())),
        ])
    );
    println!("{}", tf(keys::CLI_DIAG_FULLSCREEN, &[("state", yn(targeting::is_foreground_fullscreen()))]));
    println!("{}", t(keys::CLI_DIAG_FULLSCREEN_WINDOWS));
    for (hwnd, title) in targeting::visible_windows() {
        if targeting::is_fullscreen_window(hwnd) {
            println!("  [{}] {}", targeting::class_name(hwnd), title);
        }
    }
}
