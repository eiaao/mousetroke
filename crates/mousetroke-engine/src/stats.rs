//! 统计写盘线程：钩子线程只发送事件（不阻塞），这里累计后攒一小段再原子写盘。
use std::path::PathBuf;
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

use mousetroke_core::stats::{Event, Stats};

/// 连续触发只写一次；进程退出最多丢这么久的计数，设置页也最多晚这么久看到。
const FLUSH: Duration = Duration::from_secs(3);

pub struct Recorder {
    tx: Sender<Event>,
}

impl Recorder {
    pub fn spawn(path: PathBuf) -> Option<Self> {
        let (tx, rx) = mpsc::channel::<Event>();
        std::thread::Builder::new()
            .name("mousetroke-stats".into())
            .spawn(move || {
                let mut stats = Stats::load(&path);
                while let Ok(e) = rx.recv() {
                    stats.record(&e);
                    let deadline = Instant::now() + FLUSH;
                    let mut closed = false;
                    loop {
                        match rx.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
                            Ok(e) => stats.record(&e),
                            Err(RecvTimeoutError::Timeout) => break,
                            Err(RecvTimeoutError::Disconnected) => {
                                closed = true;
                                break;
                            }
                        }
                    }
                    if let Err(err) = stats.save(&path) {
                        crate::log!("写入使用统计失败: {err}");
                    }
                    if closed {
                        return;
                    }
                }
            })
            .ok()?;
        Some(Self { tx })
    }

    #[cfg(test)]
    pub(crate) fn from_sender(tx: Sender<Event>) -> Self {
        Self { tx }
    }

    pub fn record(&self, e: Event) {
        let _ = self.tx.send(e);
    }
}
