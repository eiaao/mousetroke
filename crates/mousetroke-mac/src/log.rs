//! 有界日志队列；回调只复制最多 2048 字节并 try_send，时间格式化、控制台与磁盘均在后台。
use std::fmt::{self, Write as _};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::mpsc::{sync_channel, SyncSender};
use std::sync::{Mutex, OnceLock};
use std::thread::JoinHandle;

use objc2::rc::autoreleasepool;
use objc2_foundation::{ns_string, NSDate, NSDateFormatter, NSLocale};

const MAX_LINE: usize = 2048;
const MAX_BYTES: u64 = 1 << 20;
static TX: OnceLock<SyncSender<Message>> = OnceLock::new();
static WORKER: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);

struct Line {
    bytes: [u8; MAX_LINE],
    len: usize,
}
impl Line {
    fn new() -> Self {
        Self {
            bytes: [0; MAX_LINE],
            len: 0,
        }
    }
    fn text(&self) -> &str {
        std::str::from_utf8(&self.bytes[..self.len]).unwrap_or_default()
    }
}
impl fmt::Write for Line {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let mut len = text.len().min(MAX_LINE - self.len);
        while !text.is_char_boundary(len) {
            len -= 1;
        }
        self.bytes[self.len..self.len + len].copy_from_slice(&text.as_bytes()[..len]);
        self.len += len;
        Ok(())
    }
}

// 固定大小内联日志使 tap 不需要为每条日志再分配 Box；队列总内存仍有明确上限。
#[allow(clippy::large_enum_variant)]
enum Message {
    Line(Line),
    Shape(Vec<i32>),
    Stop,
}

pub fn path() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| ".".into())
        .join("Library/Logs/mousetroke/mousetroke.log")
}

pub fn init() -> io::Result<()> {
    if TX.get().is_some() {
        return Ok(());
    }
    let path = path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = std::fs::OpenOptions::new().create(true).append(true).open(&path)?;
    let (tx, rx) = sync_channel(256);
    let worker = std::thread::Builder::new()
        .name("mousetroke-log".into())
        .spawn(move || {
            autoreleasepool(|_| {
                // Foundation 可在工作线程使用；每条日志释放临时 Objective-C 对象。
                let formatter = NSDateFormatter::new();
                formatter.setLocale(Some(&NSLocale::localeWithLocaleIdentifier(ns_string!("en_US_POSIX"))));
                formatter.setDateFormat(Some(ns_string!("yyyy-MM-dd HH:mm:ss.SSS")));
                let mut file = Some(file);
                for message in rx {
                    let text = match message {
                        Message::Line(line) => line.text().to_owned(),
                        Message::Shape(sample) => {
                            let values = sample.iter().map(i32::to_string).collect::<Vec<_>>().join(",");
                            format!("形状样本: [{values}]") // i18n:allow 运行日志与 Windows 采样格式一致
                        }
                        Message::Stop => break,
                    };
                    autoreleasepool(|_| {
                        let timestamp = formatter.stringFromDate(&NSDate::date());
                        let line = format!("{timestamp} {text}\n");
                        if file
                            .as_ref()
                            .and_then(|f| f.metadata().ok())
                            .is_some_and(|m| m.len() > MAX_BYTES)
                        {
                            drop(file.take());
                            let old = path.with_extension("log.old");
                            if old.exists() {
                                let _ = std::fs::remove_file(&old);
                            }
                            if let Err(error) = std::fs::rename(&path, &old) {
                                eprintln!("{error}");
                            }
                            file = std::fs::OpenOptions::new().create(true).append(true).open(&path).ok();
                        }
                        if let Some(file) = file.as_mut() {
                            if let Err(error) = file.write_all(line.as_bytes()) {
                                eprintln!("{error}");
                            }
                        }
                        // SSH 管道关闭时不 panic，仍继续写日志文件。
                        let _ = writeln!(std::io::stdout().lock(), "[mousetroke] {text}");
                    });
                }
                if let Some(file) = file.as_mut() {
                    let _ = file.flush();
                }
            })
        })?;
    let _ = TX.set(tx);
    if let Ok(mut slot) = WORKER.lock() {
        *slot = Some(worker);
    }
    Ok(())
}

pub fn write(text: &str) {
    crate::selftest::observe(text);
    write_args(format_args!("{text}"));
}

pub(crate) fn write_args(args: fmt::Arguments<'_>) {
    if let Some(tx) = TX.get() {
        let mut line = Line::new();
        let _ = line.write_fmt(args);
        let _ = tx.try_send(Message::Line(line));
    }
}

pub(crate) fn shape_sample(sample: Vec<i32>) {
    if sample.len() <= 512 {
        if let Some(tx) = TX.get() {
            let _ = tx.try_send(Message::Shape(sample));
        }
    }
}

/// 仅在 run loop 结束后调用，可等待后台清空队列，不可在 tap 回调里调用。
pub fn shutdown() {
    if let Some(tx) = TX.get() {
        let _ = tx.send(Message::Stop);
    }
    if let Some(worker) = WORKER.lock().ok().and_then(|mut slot| slot.take()) {
        let _ = worker.join();
    }
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => { $crate::log::write_args(format_args!($($arg)*)) };
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn callback_log_is_bounded_and_preserves_utf8() {
        let mut line = Line::new();
        line.write_str(&"界".repeat(1000)).unwrap();
        assert!(line.len <= MAX_LINE);
        assert_eq!(line.text().len(), 2046);
    }
}
