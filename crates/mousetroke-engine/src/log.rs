//! 运行日志（两平台共用）：输入回调只把消息复制进固定大小缓冲并 try_send，
//! 时间戳、控制台输出、写盘与 1MB 轮转都在后台线程；平台只提供路径和时间戳格式。
use std::fmt::{self, Write as _};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::mpsc::{sync_channel, SyncSender};
use std::sync::{Mutex, OnceLock};
use std::thread::JoinHandle;

const MAX_LINE: usize = 2048;
const MAX_BYTES: u64 = 1 << 20;
const QUEUE: usize = 256;

static TX: OnceLock<SyncSender<Message>> = OnceLock::new();
static WORKER: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);
static OBSERVER: OnceLock<fn(&str)> = OnceLock::new();

struct Line {
    bytes: [u8; MAX_LINE],
    len: usize,
}
impl Line {
    fn new() -> Self {
        Self { bytes: [0; MAX_LINE], len: 0 }
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

// 固定大小内联日志使回调不必为每条日志分配；队列总内存有明确上限。
#[allow(clippy::large_enum_variant)]
enum Message {
    Line(Line),
    Shape(Vec<i32>),
    Stop,
}

/// 启动写日志线程（重复调用无效果）。`timestamp` 在后台线程调用。
pub fn init(path: PathBuf, timestamp: fn() -> String) -> io::Result<()> {
    if TX.get().is_some() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let open = |path: &PathBuf| std::fs::OpenOptions::new().create(true).append(true).open(path);
    let file = open(&path)?;
    let (tx, rx) = sync_channel(QUEUE);
    if TX.set(tx).is_err() {
        return Ok(());
    }
    let worker = std::thread::Builder::new().name("mousetroke-log".into()).spawn(move || {
        let mut file = Some(file);
        for message in rx {
            let text = match message {
                Message::Line(line) => line.text().to_owned(),
                Message::Shape(sample) => {
                    let values = sample.iter().map(i32::to_string).collect::<Vec<_>>().join(",");
                    format!("形状样本: [{values}]") // i18n:allow 校准采样的日志协议
                }
                Message::Stop => break,
            };
            if file.as_ref().and_then(|f| f.metadata().ok()).is_some_and(|m| m.len() > MAX_BYTES) {
                drop(file.take());
                let old = path.with_extension("log.old");
                let _ = std::fs::remove_file(&old);
                let _ = std::fs::rename(&path, &old);
                file = open(&path).ok();
            }
            if let Some(file) = file.as_mut() {
                let _ = writeln!(file, "{} {text}", timestamp());
            }
            // 终端或管道关闭时忽略错误，日志文件照常写。
            let _ = writeln!(io::stdout().lock(), "[mousetroke] {text}");
        }
        if let Some(file) = file.as_mut() {
            let _ = file.flush();
        }
    })?;
    if let Ok(mut slot) = WORKER.lock() {
        *slot = Some(worker);
    }
    Ok(())
}

/// 自测等诊断用：每条日志在调用线程同步回调一次（只读文本，须快速返回）。
pub fn set_observer(observer: fn(&str)) {
    let _ = OBSERVER.set(observer);
}

/// `log!` 的实现：不分配，队列满时丢弃；未初始化时只输出到控制台（命令行工具场景）。
pub fn write_args(args: fmt::Arguments<'_>) {
    let mut line = Line::new();
    let _ = line.write_fmt(args);
    if let Some(observe) = OBSERVER.get() {
        observe(line.text());
    }
    match TX.get() {
        Some(tx) => {
            let _ = tx.try_send(Message::Line(line));
        }
        None => println!("[mousetroke] {}", line.text()),
    }
}

pub fn write(text: &str) {
    write_args(format_args!("{text}"));
}

/// 校准样本（MOUSETROKE_SHAPE_SAMPLES=1）；格式化在后台，超长样本丢弃。
pub fn shape_sample(sample: Vec<i32>) {
    if sample.len() <= 512 {
        if let Some(tx) = TX.get() {
            let _ = tx.try_send(Message::Shape(sample));
        }
    }
}

/// 退出前清空队列；可等待后台线程，不可在输入回调里调用。
pub fn shutdown() {
    if let Some(tx) = TX.get() {
        let _ = tx.send(Message::Stop);
    }
    if let Some(worker) = WORKER.lock().ok().and_then(|mut slot| slot.take()) {
        let _ = worker.join();
    }
}

/// `log!("格式", 参数…)`：写入运行日志（不分配）。
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
