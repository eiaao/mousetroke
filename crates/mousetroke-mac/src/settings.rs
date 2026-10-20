//! 只启动独立界面进程，常驻引擎不创建 WebView，也不等待界面退出。
use std::{path::PathBuf, sync::OnceLock};
static CONFIG: OnceLock<PathBuf> = OnceLock::new();

pub fn initialize(config: PathBuf) {
    let _ = CONFIG.set(config);
}
pub fn open(about: bool) {
    spawn(if about { Some("--about") } else { None });
}
pub fn open_setup() {
    spawn(Some("--setup"));
}
fn spawn(mode: Option<&'static str>) {
    let config = CONFIG
        .get()
        .cloned()
        .unwrap_or_else(crate::default_config_path);
    std::thread::spawn(move || {
        let result = std::env::current_exe().and_then(|exe| {
            let mut cmd = std::process::Command::new(exe);
            cmd.arg("settings").arg(config);
            if let Some(mode) = mode {
                cmd.arg(mode);
            }
            cmd.spawn()?.wait()
        });
        if result.is_err() {
            crate::log!("启动设置进程失败");
        }
    });
}
