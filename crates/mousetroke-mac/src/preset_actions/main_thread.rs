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
    match rx.recv_timeout(Duration::from_secs(5)) {
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
                rx.recv()
                    .map_err(|_| "Main-thread text action disconnected")?
            }
        }
    }
}
