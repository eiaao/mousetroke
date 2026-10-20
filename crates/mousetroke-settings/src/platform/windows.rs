//! Windows 设置后端；原有 Win32 行为保留在此平台边界内。
use std::path::{Path, PathBuf};
use crate::{keys, t, MsgResult, EngineStatus, RunningApp};

pub mod icons;
pub mod picker;
pub use mousetroke_win::{system_langs, update};
pub use mousetroke_win::log::path as log_path;
pub use mousetroke_core::schema::default_config_path;
pub use mousetroke_win::engine_running;

pub fn engine_exe() -> Option<PathBuf> {
    let me = std::env::current_exe().ok()?;
    if !mousetroke_core::matcher::file_name(&me.to_string_lossy()).eq_ignore_ascii_case("mousetroke-settings.exe") {
        return Some(me);
    }
    let p = me.parent()?.join("mousetroke.exe");
    p.exists().then_some(p)
}

pub fn engine_status() -> EngineStatus {
    EngineStatus {
        running: mousetroke_win::engine_running(),
        autostart: mousetroke_win::autostart::is_enabled(),
        exe_found: engine_exe().is_some(),
    }
}

pub fn bring_to_front() {}
pub fn setup(_app: &tauri::App) {}
pub fn refresh_texts(_window: &tauri::WebviewWindow) {}
pub fn running_apps() -> Vec<RunningApp> {
    mousetroke_win::targeting::running_apps().into_iter().map(|(exe, title)| RunningApp { exe, title }).collect()
}
pub fn set_autostart(on: bool, exe: &Path) -> Result<(), String> {
    mousetroke_win::autostart::set_for(on, &exe.to_string_lossy()).map(|_| ())
}
pub fn start_engine(exe: &Path, config: &Path) -> std::io::Result<()> {
    std::process::Command::new(exe).arg("run").arg("--config").arg(config).spawn()?;
    for _ in 0..20 {
        std::thread::sleep(std::time::Duration::from_millis(50));
        if engine_running() { break; }
    }
    Ok(())
}
pub fn open_file(path: &Path) -> MsgResult<()> {
    mousetroke_win::input::shell_execute("open", &path.to_string_lossy(), "");
    Ok(())
}
pub fn open_folder(path: &Path) -> MsgResult<()> {
    let _ = std::process::Command::new(mousetroke_win::input::system_exe("explorer.exe")).arg(path).spawn();
    Ok(())
}
pub fn open_url(url: &str) -> MsgResult<()> {
    mousetroke_win::input::shell_execute("open", url, "");
    Ok(())
}
pub fn reveal_config(path: &Path) {
    let arg = format!("/select,\"{}\"", path.display());
    mousetroke_win::input::shell_execute("open", &mousetroke_win::input::system_exe("explorer.exe").to_string_lossy(), &arg);
}
pub fn program_dialog(d: rfd::FileDialog) -> rfd::FileDialog {
    d.add_filter(t(keys::UI_PICKER_FILTER_PROGRAM), &["exe", "lnk", "bat", "cmd"])
}
pub fn picked_file(path: String, _kind: &str) -> Option<String> { Some(path) }
pub fn pick_window(window: tauri::WebviewWindow) -> MsgResult<Option<picker::PickedApp>> {
    let _ = window.minimize();
    std::thread::sleep(std::time::Duration::from_millis(250));
    let picked = picker::pick_window();
    let _ = window.unminimize();
    let _ = window.set_focus();
    picked
}
