//! 运行日志：`~/Library/Logs/mousetroke/mousetroke.log`（超过 1MB 轮转为 `.log.old`）。
//! 队列、写盘与控制台输出见 `mousetroke_engine::log`；这里只提供路径和本地时间格式。
use std::path::PathBuf;

use objc2::rc::{autoreleasepool, Retained};
use objc2_foundation::{ns_string, NSDate, NSDateFormatter, NSLocale};

pub fn path() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| ".".into())
        .join("Library/Logs/mousetroke/mousetroke.log")
}

/// 只在写日志线程调用；格式器按线程缓存，每次调用释放临时 Objective-C 对象。
fn timestamp() -> String {
    thread_local! {
        static FORMATTER: Retained<NSDateFormatter> = {
            let formatter = NSDateFormatter::new();
            formatter.setLocale(Some(&NSLocale::localeWithLocaleIdentifier(ns_string!("en_US_POSIX"))));
            formatter.setDateFormat(Some(ns_string!("yyyy-MM-dd HH:mm:ss.SSS")));
            formatter
        };
    }
    autoreleasepool(|_| FORMATTER.with(|f| f.stringFromDate(&NSDate::date()).to_string()))
}

pub fn init() -> std::io::Result<()> {
    mousetroke_engine::log::init(path(), timestamp)
}

pub use mousetroke_engine::log::shutdown;

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => { $crate::__engine::log::write_args(format_args!($($arg)*)) };
}
