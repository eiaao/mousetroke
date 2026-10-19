//! AppKit 入口在 Tauri 主线程；后台命令通过 run_on_main_thread 回到此处。
use std::path::{Path, PathBuf};

use objc2::MainThreadMarker;
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy, NSWorkspace};
use objc2_foundation::{NSArray, NSString, NSURL};
use tauri::Manager;

use crate::{keys, EngineStatus, Msg, MsgResult, RunningApp};

mod autostart;
pub mod icons;
mod instance;
mod menu;
pub mod picker;
pub use instance::{enter_instance, take_about_request};
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
    instance::listen(app.handle().clone());
    menu::install(app);
}

pub fn refresh_texts(window: &tauri::WebviewWindow) {
    menu::refresh(window.app_handle());
}

pub fn unsupported_keys(keys: &mousetroke_core::keyseq::KeySeq) -> bool {
    use mousetroke_core::keyseq::{Action, Key};
    keys.actions.iter().any(|action| match action {
        Action::Key {
            key: Key::Vk(vk), ..
        } => mousetroke_core::keymap::vk_to_kvk(*vk).is_none(),
        Action::Key {
            key: Key::Mouse(_), ..
        } => true,
        _ => false,
    })
}

pub fn running_apps() -> Vec<RunningApp> {
    use objc2_app_kit::NSApplicationActivationPolicy;
    NSWorkspace::sharedWorkspace()
        .runningApplications()
        .iter()
        .filter(|app| {
            app.activationPolicy() == NSApplicationActivationPolicy::Regular && !app.isTerminated()
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
