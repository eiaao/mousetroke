//! flock 保证单实例；原子替换的小请求文件负责唤醒和定位关于页，不依赖窗口标题或 PID 重用。
use crate::{keys, Msg, MsgResult};
use std::{
    fs::File,
    sync::{
        atomic::{AtomicBool, Ordering},
        OnceLock,
    },
    time::Duration,
};
use tauri::Manager;

static LOCK: OnceLock<File> = OnceLock::new();
static ABOUT: AtomicBool = AtomicBool::new(false);

fn request_path() -> std::path::PathBuf {
    super::default_config_path().with_file_name("settings.request")
}

pub fn enter_instance(about: bool) -> MsgResult<bool> {
    let path = request_path();
    let dir = path
        .parent()
        .ok_or_else(|| Msg::new(keys::MAC_SETTINGS_INSTANCE_FAILED, &[]))?;
    let lock = mousetroke_mac::instance::acquire_named(dir, "settings.lock")
        .map_err(|_| Msg::new(keys::MAC_SETTINGS_INSTANCE_FAILED, &[]))?;
    if let Some(lock) = lock {
        let _ = LOCK.set(lock);
        ABOUT.store(about, Ordering::Release);
        return Ok(false);
    }
    // 每个发送者写独立临时文件，rename 后接收者不会读到半条请求。
    // 关于与唤醒使用不同旗标，稍后的普通启动不能覆盖尚未消费的关于请求。
    let path = if about {
        path.with_extension("about-request")
    } else {
        path
    };
    let temp = path.with_extension(format!("request.{}", std::process::id()));
    std::fs::write(&temp, if about { "about" } else { "focus" })
        .and_then(|_| std::fs::rename(&temp, &path))
        .map_err(|_| Msg::new(keys::MAC_SETTINGS_INSTANCE_FAILED, &[]))?;
    Ok(true)
}

pub fn take_about_request() -> bool {
    ABOUT.swap(false, Ordering::AcqRel)
}
pub fn request_about() {
    ABOUT.store(true, Ordering::Release);
}

pub fn listen(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        let path = request_path();
        loop {
            let mut focus = false;
            for (request, about) in [
                (&path, false),
                (&path.with_extension("about-request"), true),
            ] {
                let processing = request.with_extension(if about {
                    "about.processing"
                } else {
                    "focus.processing"
                });
                // 先取走请求再读取，新到的请求会留给下一轮。
                if std::fs::rename(request, &processing).is_ok() {
                    if about {
                        ABOUT.store(true, Ordering::Release);
                    }
                    let _ = std::fs::remove_file(&processing);
                    focus = true;
                }
            }
            if focus {
                let handle = app.clone();
                let _ = app.run_on_main_thread(move || {
                    if let Some(window) = handle.get_webview_window("main") {
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_focus();
                        if let Some(mtm) = objc2::MainThreadMarker::new() {
                            #[allow(deprecated)]
                            objc2_app_kit::NSApplication::sharedApplication(mtm)
                                .activateIgnoringOtherApps(true);
                        }
                    }
                });
            }
            std::thread::sleep(Duration::from_millis(150));
        }
    });
}
