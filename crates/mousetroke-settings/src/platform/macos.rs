//! AppKit 入口在 Tauri 主线程；后台命令通过 run_on_main_thread 回到此处。
use std::path::{Path, PathBuf};

use objc2::MainThreadMarker;
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy, NSWorkspace};
use objc2_foundation::{NSArray, NSString, NSURL};
use tauri::Manager;

use crate::{keys, EngineStatus, Msg, MsgResult, RunningApp};

mod autostart;
pub mod icons;
mod menu;
pub mod picker;
pub use mousetroke_mac::log::path as log_path;
pub use mousetroke_mac::{default_config_path, system_langs};

pub fn engine_exe() -> Option<PathBuf> {
    let me = std::env::current_exe().ok()?;
    if me.file_name()? != "mousetroke-settings" {
        return Some(me);
    }
    let engine = me.parent()?.join("mousetroke");
    engine.is_file().then_some(engine)
}

pub fn engine_status() -> EngineStatus {
    EngineStatus {
        running: mousetroke_mac::instance::engine_running(),
        autostart: autostart::is_enabled(),
        exe_found: engine_exe().is_some(),
    }
}

pub fn setup(app: &tauri::App) {
    if let Some(mtm) = MainThreadMarker::new() {
        // 引擎的 bundle 是 LSUIElement；独立设置进程仍需原生标题栏与正常键盘焦点。
        let native = NSApplication::sharedApplication(mtm);
        native.setActivationPolicy(NSApplicationActivationPolicy::Regular);
        #[allow(deprecated)]
        native.activateIgnoringOtherApps(true);
    }
    menu::install(app);
}

/// 被第二次启动唤醒时应用可能在后台；LSUIElement 引擎同 bundle，需要显式激活。
/// 当前引擎 PID；引擎重启后会变，用于重发录制画布位置。
pub fn engine_id() -> Option<isize> {
    mousetroke_mac::instance::engine_pid().map(|pid| pid as isize)
}
/// 录制画布的全局坐标（点，与引擎事件位置一致）。
pub fn send_canvas(engine: isize, rect: Option<(i32, i32, i32, i32)>) {
    let _ = mousetroke_mac::instance::send_canvas(engine as u32, rect);
}
/// 引擎事件坐标的单位：macOS 为点。
pub const ENGINE_USES_PHYSICAL_PIXELS: bool = false;

pub fn bring_to_front() {
    if let Some(mtm) = MainThreadMarker::new() {
        #[allow(deprecated)]
        NSApplication::sharedApplication(mtm).activateIgnoringOtherApps(true);
    }
}

pub fn refresh_texts(window: &tauri::WebviewWindow) {
    menu::refresh(window.app_handle());
}

/// bundle id → 访达显示名（作用域名称用）。
pub fn app_display_name(identity: &str) -> Option<String> {
    icons::display_name(identity)
}

/// mac 暂不列出已安装程序；界面只显示正在运行的程序。
pub fn installed_apps() -> Vec<RunningApp> { Vec::new() }

pub fn running_apps() -> Vec<RunningApp> {
    use objc2_app_kit::NSApplicationActivationPolicy;
    NSWorkspace::sharedWorkspace()
        .runningApplications()
        .iter()
        .filter(|app| {
            app.activationPolicy() == NSApplicationActivationPolicy::Regular
                && !app.isTerminated()
                && !app.bundleIdentifier().is_some_and(|id| {
                    mousetroke_mac::ignored_bundle(&id.to_string())
                })
        })
        .filter_map(|app| {
            let exe = app
                .bundleIdentifier()
                .map(|id| id.to_string())
                .filter(|id| !id.is_empty())
                .or_else(|| app.executableURL()?.path().map(|p| p.to_string()))?;
            Some(RunningApp {
                exe,
                title: app
                    .localizedName()
                    .map(|n| n.to_string())
                    .unwrap_or_default(),
            })
        })
        .collect()
}

pub fn set_autostart(on: bool, exe: &Path) -> Result<(), String> {
    autostart::set(on, exe)
}

pub fn start_engine(exe: &Path, config: &Path) -> std::io::Result<()> {
    // 自定义配置也交给独立引擎进程；常驻进程不承载 WebView。
    let mut command = std::process::Command::new(exe);
    if config != default_config_path() {
        command.arg("run").arg("--config").arg(config);
    }
    command.spawn()?;
    Ok(())
}

pub fn open_file(path: &Path) -> MsgResult<()> {
    open(&NSURL::fileURLWithPath(&NSString::from_str(
        &path.to_string_lossy(),
    )))
}
pub fn open_folder(path: &Path) -> MsgResult<()> {
    open_file(path)
}
pub fn open_url(url: &str) -> MsgResult<()> {
    let url = NSURL::URLWithString(&NSString::from_str(url))
        .ok_or_else(|| Msg::new(keys::ERROR_UNKNOWN_LOCATION, &[("what", url.into())]))?;
    open(&url)
}
fn open(url: &NSURL) -> MsgResult<()> {
    if NSWorkspace::sharedWorkspace().openURL(url) {
        Ok(())
    } else {
        Err(Msg::new(keys::MAC_SETTINGS_OPEN_FAILED, &[]))
    }
}
pub fn reveal_config(path: &Path) {
    let url = NSURL::fileURLWithPath(&NSString::from_str(&path.to_string_lossy()));
    NSWorkspace::sharedWorkspace().activateFileViewerSelectingURLs(&NSArray::from_slice(&[&*url]));
}
pub fn program_dialog(d: rfd::FileDialog) -> rfd::FileDialog {
    // 不加扩展名过滤，允许同时选择 .app 包与无扩展名可执行文件。
    d.set_directory("/Applications")
}
pub fn picked_file(path: String, kind: &str) -> Option<String> {
    if kind == "exe" {
        picker::resolve_app(&path).ok()
    } else {
        Some(path)
    }
}

pub fn pick_window(window: tauri::WebviewWindow) -> MsgResult<Option<picker::PickedApp>> {
    // 此函数由异步命令工作线程调用，整个 AppKit picker 生命周期固定在主线程。
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let handle = window.app_handle().clone();
    handle
        .run_on_main_thread(move || {
            let _ = window.hide();
            let result = picker::pick_window();
            let _ = window.show();
            let _ = window.set_focus();
            let _ = tx.send(result);
        })
        .map_err(|_| Msg::new(keys::ERROR_PICK_FAILED, &[]))?;
    rx.recv()
        .map_err(|_| Msg::new(keys::ERROR_PICK_FAILED, &[]))?
}

pub use mousetroke_mac::update;
