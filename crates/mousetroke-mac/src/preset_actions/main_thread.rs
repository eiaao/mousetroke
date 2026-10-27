use block2::RcBlock;
use objc2::{rc::autoreleasepool, MainThreadMarker};
use objc2_foundation::NSOperationQueue;
use std::{
    sync::{mpsc, Arc, Mutex},
    time::Duration,
};

/// 仅用于短暂的剪贴板或事件投递；等待粘贴完成始终留在调用方工作线程。
pub(crate) fn on_main<R: Send + 'static>(
    f: impl FnOnce() -> Result<R, String> + Send + 'static,
) -> Result<R, String> {
    if MainThreadMarker::new().is_some() {
        return autoreleasepool(|_| f());
    }
    let (tx, rx) = mpsc::sync_channel(1);
    let task = Arc::new(Mutex::new(Some(f)));
    let queued = Arc::clone(&task);
    let block = RcBlock::new(move || {
        let f = queued.lock().ok().and_then(|mut task| task.take());
        if let Some(f) = f {
            let _ = tx.send(autoreleasepool(|_| f()));
        }
    });
    // 安全：捕获内容可 Send，队列复制 block；闭包恰好执行一次且只在主线程运行。
    unsafe {
        NSOperationQueue::mainQueue().addOperationWithBlock(&block);
    }
    wait_for_task(rx, &task, Duration::from_secs(5))
}

fn wait_for_task<R>(rx: mpsc::Receiver<Result<R, String>>, task: &Mutex<Option<impl Sized>>, timeout: Duration) -> Result<R, String> {
    match rx.recv_timeout(timeout) {
        Ok(result) => result,
        Err(_) => {
            // 尚未开始的操作可取消，避免超时返回后才改写剪贴板。
            let pending = task
                .lock()
                .map_err(|_| "Main-thread action lock poisoned")?
                .take();
            if pending.is_some() {
                Err("Main thread did not respond; text action cancelled".into())
            } else {
                // 已开始的操作不能取消；丢弃晚到的结果并释放工作线程，避免主线程卡住时队列永久死锁。
                Err("Main-thread text action timed out; late result discarded".into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_timeout_cancels_the_task() {
        let (_tx, rx) = mpsc::sync_channel::<Result<(), String>>(1);
        let task = Mutex::new(Some(()));
        assert!(wait_for_task(rx, &task, Duration::ZERO).unwrap_err().contains("cancelled"));
        assert!(task.lock().unwrap().is_none());
    }

    #[test]
    fn started_timeout_releases_worker_and_discards_late_result() {
        let (tx, rx) = mpsc::sync_channel::<Result<(), String>>(1);
        let task = Mutex::new(Some(()));
        task.lock().unwrap().take();
        assert!(wait_for_task(rx, &task, Duration::ZERO).unwrap_err().contains("timed out"));
        assert!(tx.send(Ok(())).is_err());
    }

    #[test]
    fn completed_task_returns_its_result() {
        let (tx, rx) = mpsc::sync_channel(1);
        tx.send(Ok(42)).unwrap();
        assert_eq!(wait_for_task(rx, &Mutex::new(None::<()>), Duration::ZERO), Ok(42));
    }
}
