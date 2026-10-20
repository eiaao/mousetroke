//! 单独的设置程序入口——只用于开发调试（`cargo tauri dev` 等）。
//! 发布版只有一个 `mousetroke.exe`，用 `mousetroke settings [配置路径]` 打开同一个界面。

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(target_os = "macos")]
    mousetroke_mac::permissions::exit_if_probe();
    mousetroke_settings::run(std::env::args().nth(1).map(Into::into));
}
