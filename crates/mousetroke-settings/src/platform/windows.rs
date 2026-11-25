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

pub fn play_effect(effect: mousetroke_core::presets::native::FunEffect) {
    mousetroke_win::fx::play(effect, None);
}
/// 当前引擎控制窗口；引擎重启后会变，用于重发录制画布位置。
pub fn engine_id() -> Option<isize> {
    mousetroke_win::tray::engine_window().map(|(hwnd, _)| hwnd.0 as isize)
}
/// 引擎钩子坐标的单位：Windows 为物理像素（引擎按每显示器 DPI 感知运行）。
pub const ENGINE_USES_PHYSICAL_PIXELS: bool = true;
/// 录制画布的屏幕矩形（物理像素，与引擎钩子坐标一致）。
pub fn send_canvas(engine: isize, rect: Option<(i32, i32, i32, i32)>) {
    mousetroke_win::tray::send_settings_canvas(engine, rect);
}
pub fn setup(_app: &tauri::App) {}
pub fn refresh_texts(_window: &tauri::WebviewWindow) {}
pub fn running_apps() -> Vec<RunningApp> {
    mousetroke_win::targeting::running_apps().into_iter().map(|(exe, title)| RunningApp { exe, title }).collect()
}
/// Windows 的作用域名称直接用文件名，不另取显示名。
pub fn app_display_name(_identity: &str) -> Option<String> {
    None
}

pub fn installed_apps() -> Vec<RunningApp> {
    picker::installed_apps().into_iter().map(|(exe, title)| RunningApp { exe, title }).collect()
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

/// 系统版本，如 `Windows 11 Pro 24H2 (build 26100.2894)`。
pub fn os_version() -> String {
    use windows::core::{w, PCWSTR};
    use windows::Win32::System::Registry::{RegGetValueW, HKEY_LOCAL_MACHINE, RRF_RT_REG_DWORD, RRF_RT_REG_SZ};
    const KEY: PCWSTR = w!(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion");
    let text = |name: PCWSTR| {
        let mut buf = [0u16; 128];
        let mut len = std::mem::size_of_val(&buf) as u32;
        // 安全：缓冲区与长度一致，RRF_RT_REG_SZ 保证以 0 结尾。
        unsafe { RegGetValueW(HKEY_LOCAL_MACHINE, KEY, name, RRF_RT_REG_SZ, None, Some(buf.as_mut_ptr().cast()), Some(&mut len)) }.ok().ok()?;
        let n = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        Some(String::from_utf16_lossy(&buf[..n]))
    };
    let mut ubr = 0u32;
    let mut len = 4u32;
    // 安全：DWORD 输出到 4 字节变量。
    let _ = unsafe { RegGetValueW(HKEY_LOCAL_MACHINE, KEY, w!("UBR"), RRF_RT_REG_DWORD, None, Some((&mut ubr as *mut u32).cast()), Some(&mut len)) };
    let build = text(w!("CurrentBuildNumber")).unwrap_or_default();
    let mut name = text(w!("ProductName")).unwrap_or_else(|| "Windows".into());
    // Windows 11 的注册表 ProductName 仍写 “Windows 10”
    if build.parse::<u32>().is_ok_and(|b| b >= 22000) {
        name = name.replacen("Windows 10", "Windows 11", 1);
    }
    let display = text(w!("DisplayVersion")).map(|v| format!(" {v}")).unwrap_or_default();
    format!("{name}{display} (build {build}.{ubr})")
}

/// 把文本放进剪贴板。
pub fn copy_text(text: &str) {
    mousetroke_win::input::set_clipboard_text(text);
}
