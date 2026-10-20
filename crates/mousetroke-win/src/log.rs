//! 运行日志：`%APPDATA%\mousetroke\mousetroke.log`（超过 1MB 轮转为 `.log.old`）。
//! 队列、写盘与控制台输出见 `mousetroke_engine::log`；这里只提供路径和本地时间格式。

/// 日志文件路径（与配置同目录）。
pub fn path() -> std::path::PathBuf {
    mousetroke_core::schema::default_config_path().with_file_name("mousetroke.log")
}

fn timestamp() -> String {
    let t = unsafe { windows::Win32::System::SystemInformation::GetLocalTime() };
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}",
        t.wYear, t.wMonth, t.wDay, t.wHour, t.wMinute, t.wSecond, t.wMilliseconds
    )
}

/// 启动写日志线程（重复调用无效果；失败时日志只输出到控制台）。
pub fn init() {
    let _ = mousetroke_engine::log::init(path(), timestamp);
}

/// `log!("格式", 参数…)`：写入运行日志（不分配，钩子回调可用）。
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => { $crate::__engine::log::write_args(format_args!($($arg)*)) };
}
