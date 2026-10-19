//! Lua 只在工作线程运行；主线程只分片处理桥接请求，从不等待脚本结束。
use mousetroke_core::{keyseq::KeySeq, schema::IfRunning};
use mousetroke_lua::{KeyPhase, LuaCtx, LuaPlatform};
use objc2_app_kit::{NSPasteboard, NSPasteboardTypeString};
use objc2_core_graphics::{CGEvent, CGEventTapLocation, CGEventType};
use objc2_foundation::NSString;
use std::{
    cell::RefCell,
    collections::BTreeSet,
    sync::{mpsc, Arc, Mutex, OnceLock},
    time::Duration,
};

type Job = Box<dyn FnOnce() + Send>;
static SENDER: OnceLock<mpsc::SyncSender<Job>> = OnceLock::new();
thread_local! { static RECEIVER: RefCell<Option<mpsc::Receiver<Job>>> = const { RefCell::new(None) }; }

pub fn initialize() {
    let (tx, rx) = mpsc::sync_channel(64);
    if SENDER.set(tx).is_ok() {
        RECEIVER.with(|s| *s.borrow_mut() = Some(rx));
    }
}
pub fn tick() {
    for _ in 0..8 {
        let job = RECEIVER.with(|s| s.borrow().as_ref().and_then(|rx| rx.try_recv().ok()));
        let Some(job) = job else { break };
        job();
    }
}
pub fn shutdown() {
    RECEIVER.with(|s| s.borrow_mut().take());
}

fn on_main<R: Send + 'static>(f: impl FnOnce() -> R + Send + 'static) -> Option<R> {
    let (tx, rx) = mpsc::sync_channel(1);
    let job: Job = Box::new(move || {
        let _ = tx.send(f());
    });
    if SENDER.get()?.try_send(job).is_err() {
        crate::log!("Lua 主线程请求队列已满或关闭，跳过调用");
        return None;
    }
    // 仅 Lua 工作线程等待；主线程继续处理输入、计时器和动作队列。
    rx.recv().ok()
}

pub fn run(code: String, ctx: LuaCtx) {
    // 调用方已在采集 AX 标题的工作线程；VM 不会移动到主线程。
    mousetroke_lua::run_lua(
        &code,
        &ctx,
        Arc::new(MacPlatform {
            wid: ctx.wid,
            pid: ctx.pid,
        }),
    );
}

struct MacPlatform {
    wid: usize,
    pid: u32,
}
impl LuaPlatform for MacPlatform {
    fn log(&self, message: &str) {
        crate::log!("{message}");
    }
    fn unsupported(&self, name: &str) {
        static WARNED: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());
        if WARNED.lock().is_ok_and(|mut seen| seen.insert(name.into())) {
            crate::log!("Lua {name}: not supported on macOS");
        }
    }
    fn cursor_pos(&self) -> (i32, i32) {
        crate::host::cursor_pos()
            .map(|p| (p.x as i32, p.y as i32))
            .unwrap_or((0, 0))
    }
    fn set_cursor(&self, x: i32, y: i32) {
        // CoreGraphics 事件 API 可在线程间使用；打标记避免脚本移动再次触发手势。
        if let Some(event) = crate::host::mouse_event(
            CGEventType::MouseMoved,
            0,
            objc2_core_foundation::CGPoint::new(x as f64, y as f64),
            crate::events::INJECT_TAG,
        ) {
            CGEvent::post(CGEventTapLocation::HIDEventTap, Some(&event));
        }
    }
    fn screen_of(&self, x: i32, y: i32) -> Option<(i32, i32, i32, i32)> {
        on_main(move || crate::host::screen_rect(x, y)).flatten()
    }
    fn send_keys(&self, seq: &KeySeq, phase: KeyPhase) {
        if !matches!(phase, KeyPhase::DownUp) {
            self.unsupported(if matches!(phase, KeyPhase::Down) {
                "keys_down"
            } else {
                "keys_up"
            });
            return;
        }
        let seq = seq.clone();
        if on_main(move || crate::input::enqueue_parsed(seq)).is_none() {
            return;
        }
        // 与 Windows 一样，keys 返回时序列已发完；等待发生在工作线程，SLEEP 仍由主计时器推进。
        while on_main(crate::input::busy) == Some(true) {
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    fn open_url(&self, url: &str) {
        let url = url.to_owned();
        on_main(move || crate::actions::open_url(&url, None));
    }
    fn run_shell(&self, code: &str) {
        crate::actions::run_shell(code.to_owned());
    }
    fn launch_app(&self, file: &str, args: Option<&str>) {
        let (file, args) = (file.to_owned(), args.unwrap_or_default().to_owned());
        on_main(move || crate::actions::launch(&file, &args, IfRunning::RunAgain));
    }
    fn is_paused(&self) -> bool {
        mousetroke_engine::PAUSED.load(std::sync::atomic::Ordering::Relaxed)
    }
    fn set_paused(&self, paused: bool) {
        on_main(move || {
            crate::app::with_engine(|engine| engine.set_paused(paused));
            crate::menu::request_refresh();
        });
    }
    fn show_settings(&self) {
        crate::settings::open(false);
    }
    fn show_toast(&self, message: String, duration: u64) {
        mousetroke_engine::push_toast(message, duration);
    }
    fn get_clipboard_text(&self) -> Option<String> {
        on_main(|| {
            // 安全：系统 pasteboard 类型常量有效；此闭包始终在主线程执行。
            unsafe { NSPasteboard::generalPasteboard().stringForType(NSPasteboardTypeString) }
                .map(|s| s.to_string())
        })
        .flatten()
    }
    fn set_clipboard_text(&self, text: &str) {
        let text = text.to_owned();
        on_main(move || {
            let board = NSPasteboard::generalPasteboard();
            board.clearContents();
            // 安全：系统类型常量有效，字符串在同步调用期间存活。
            unsafe { board.setString_forType(&NSString::from_str(&text), NSPasteboardTypeString) };
        });
    }
    fn focus_window(&self, wid: i64) {
        if wid as usize != self.wid || self.pid == 0 {
            return;
        }
        let pid = self.pid;
        on_main(move || crate::actions::activate(pid));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bridge_yields_to_main_timer_and_disconnects_on_shutdown() {
        initialize();
        let worker = std::thread::spawn(|| on_main(|| 42));
        while !worker.is_finished() {
            tick();
            std::thread::yield_now();
        }
        assert_eq!(worker.join().unwrap(), Some(42));
        shutdown();
        assert_eq!(on_main(|| 99), None);
    }
}
