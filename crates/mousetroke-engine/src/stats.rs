//! 统计写盘线程：钩子线程只发送事件（不阻塞），这里累计后攒一小段再原子写盘。
use std::path::PathBuf;
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

use mousetroke_core::stats::{Event, Stats};

/// 连续触发只写一次；进程退出最多丢这么久的计数，设置页也最多晚这么久看到。
const FLUSH: Duration = Duration::from_secs(3);
/// 写盘失败后的重试间隔。
const RETRY: Duration = Duration::from_secs(30);

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
                // dirty：有还没写进文件的计数（写失败时定时重试，不必等下一次触发）
                let mut dirty = false;
                loop {
                    let next = if dirty { rx.recv_timeout(RETRY) } else { rx.recv().map_err(|_| RecvTimeoutError::Disconnected) };
                    match next {
                        Ok(e) => {
                            stats.record(&e);
                            dirty = true;
                        }
                        Err(RecvTimeoutError::Timeout) => {}
                        Err(RecvTimeoutError::Disconnected) => {
                            if dirty {
                                let _ = stats.save(&path);
                            }
                            return;
                        }
                    }
                    // 攒一小段再写：连续触发只写一次
                    let deadline = Instant::now() + FLUSH;
                    loop {
                        match rx.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
                            Ok(e) => stats.record(&e),
                            Err(RecvTimeoutError::Timeout) => break,
                            Err(RecvTimeoutError::Disconnected) => {
                                let _ = stats.save(&path);
                                return;
                            }
                        }
                    }
                    match stats.save(&path) {
                        Ok(()) => dirty = false,
                        Err(err) => crate::log!("写入使用统计失败（稍后重试）: {err}"),
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
