//! 运行日志：`%APPDATA%\mousetroke\mousetroke.log`（超过 1MB 轮转为 `.log.old`）。
//!
//! 钩子回调里只投递到通道，由后台线程写盘——绝不在回调里做文件 IO。
//! 从终端启动时同时输出到控制台。

use std::io::Write;
use std::sync::mpsc::{channel, sync_channel, Sender, SyncSender};
use std::sync::OnceLock;

static TX: OnceLock<Sender<String>> = OnceLock::new();
static SHAPE_TX: OnceLock<SyncSender<Vec<i32>>> = OnceLock::new();
const MAX_BYTES: u64 = 1 << 20;

/// 仅在启动时开启校准开关后调用；格式化和控制台输出均在后台进行。
pub(crate) fn init_shape_samples() {
    SHAPE_TX.get_or_init(|| {
        let (tx, rx) = sync_channel::<Vec<i32>>(16);
        std::thread::Builder::new().name("mousetroke-shape-samples".into()).spawn(move || {
            use std::fmt::Write as _;
            for sample in rx {
                let mut json = String::with_capacity(768);
                json.push('[');
                for (i, value) in sample.iter().enumerate() {
                    if i != 0 {
                        json.push(',');
                    }
                    let _ = write!(json, "{value}");
                }
                json.push(']');
                crate::log!("形状样本: {json}");
            }
        }).ok();
        tx
    });
}

/// 钩子只投递固定大小的样本；队列满或线程退出时丢弃，绝不等待写盘。
pub(crate) fn shape_sample(encoded: Vec<i32>) {
    if let Some(tx) = SHAPE_TX.get() {
        let _ = tx.try_send(encoded);
    }
}

/// 日志文件路径（与配置同目录）。
pub fn path() -> std::path::PathBuf {
    mousetroke_core::schema::default_config_path().with_file_name("mousetroke.log")
}

/// 启动写日志线程（重复调用无效果）。
pub fn init() {
    TX.get_or_init(|| {
        let (tx, rx) = channel::<String>();
        let path = path();
        std::thread::Builder::new()
            .name("mousetroke-log".into())
            .spawn(move || {
                if let Some(dir) = path.parent() {
                    let _ = std::fs::create_dir_all(dir);
                }
                let open = || std::fs::OpenOptions::new().create(true).append(true).open(&path).ok();
                let mut file = open();
                for line in rx {
                    if file.as_ref().and_then(|f| f.metadata().ok()).is_some_and(|m| m.len() > MAX_BYTES) {
                        drop(file.take());
                        let _ = std::fs::rename(&path, path.with_extension("log.old"));
                        file = open();
                    }
                    if let Some(f) = file.as_mut() {
                        let _ = writeln!(f, "{line}");
                    }
                }
            })
            .ok();
        tx
    });
}

/// 写一行（带本地时间戳）。未 init 时只输出到控制台。
pub fn write(msg: &str) {
    let t = unsafe { windows::Win32::System::SystemInformation::GetLocalTime() };
    let line = format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03} {msg}",
        t.wYear, t.wMonth, t.wDay, t.wHour, t.wMinute, t.wSecond, t.wMilliseconds
    );
    println!("[mousetroke] {msg}");
    if let Some(tx) = TX.get() {
        let _ = tx.send(line);
    }
}

/// `log!("格式", 参数…)`：写入运行日志。
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        $crate::log::write(&format!($($arg)*))
    };
}
