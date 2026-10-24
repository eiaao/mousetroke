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
    std::thread::spawn(move || {
        if launch(mode).and_then(|mut child| child.wait()).is_err() {
            crate::log!("启动设置进程失败");
        }
    });
}

pub(crate) fn open_checked() -> Result<(), String> {
    let mut child = launch(None).map_err(|error| format!("Open settings: {error}"))?;
    // 只等待启动结果；回收进程交给独立线程，动作队列不等待设置窗口关闭。
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

fn launch(mode: Option<&'static str>) -> std::io::Result<std::process::Child> {
    let config = CONFIG
        .get()
        .cloned()
        .unwrap_or_else(crate::default_config_path);
    let mut cmd = std::process::Command::new(std::env::current_exe()?);
    cmd.arg("settings").arg(config);
    if let Some(mode) = mode {
        cmd.arg(mode);
    }
    cmd.spawn()
}
