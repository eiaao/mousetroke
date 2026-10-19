//! 按目标平台选择入口；Windows 命令处理保留原实现。
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

#[cfg(windows)]
mod windows;
#[cfg(windows)]
fn main() { windows::main(); }

#[cfg(target_os = "macos")]
fn main() {
    use mousetroke_core::i18n::{self, keys, t, tf};
    let setting = std::fs::read_to_string(mousetroke_core::schema::default_config_path()).ok()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(text.trim_start_matches('\u{feff}')).ok())
        .and_then(|v| v.get("settings")?.get("language")?.as_str().map(str::to_string))
        .unwrap_or_default();
    i18n::set_lang(i18n::resolve(&setting, &mousetroke_mac::system_langs()));
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        None | Some("run") if args.len() <= 1 => match mousetroke_mac::run() {
            Ok(code) => std::process::exit(code),
            Err(error) => {
                eprintln!("{}", tf(keys::CLI_RUN_FAILED, &[("err", &error)]));
                std::process::exit(1);
            }
        },
        Some("--help") | Some("-h") => println!("{}", tf(keys::MAC_CLI_HELP, &[("version", env!("CARGO_PKG_VERSION"))])),
        _ => { eprintln!("{}", t(keys::MAC_CLI_UNSUPPORTED)); std::process::exit(2); }
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
fn main() {}
