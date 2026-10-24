//! Lua 只在工作线程运行；主线程只分片处理桥接请求，从不等待脚本结束。
use mousetroke_core::{
    keymap::vk_to_kvk,
    keyseq::{KeySeq, ModKey},
    schema::IfRunning,
};
use mousetroke_lua::{required_keys, KeyPhase, LuaCtx, LuaPlatform, RequiredKey};
use objc2_app_kit::{NSPasteboard, NSPasteboardTypeString, NSRunningApplication};
use objc2_core_graphics::{
    CGEvent, CGEventFlags as Flags, CGEventSource, CGEventSourceStateID, CGEventTapLocation,
    CGEventType,
};
use objc2_foundation::NSString;
use std::{
    cell::RefCell,
    collections::BTreeSet,
    sync::{mpsc, Arc, Mutex, Once, OnceLock},
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
    crate::app::wake_from_worker();
    // 仅 Lua 工作线程等待；主线程继续处理输入、计时器和动作队列。
    rx.recv().ok()
}

pub fn run(code: String, ctx: LuaCtx) {
    // 调用方已在采集 AX 标题的工作线程；VM 不会移动到主线程。
    mousetroke_lua::run_lua(&code, &ctx, Arc::new(MacPlatform));
}

fn accessibility() -> bool {
    if crate::permissions::check().accessibility {
        return true;
    }
    static WARNED: Once = Once::new();
    WARNED.call_once(|| crate::log!("Lua: 需要辅助功能权限，跳过按键状态或 AX 窗口调用"));
    false
}

fn window_by_id(wid: i64) -> Option<crate::helpers::Window> {
    let wid = usize::try_from(wid).ok().filter(|wid| *wid != 0)?;
    crate::targeting::lua_windows(false)
        .into_iter()
        .find(|w| w.wid == wid && w.layer == 0)
}

fn modifier_flag(modifier: ModKey) -> Flags {
    match modifier {
        ModKey::Ctrl => Flags::MaskControl,
        ModKey::Alt => Flags::MaskAlternate,
        ModKey::Shift => Flags::MaskShift,
        ModKey::Win => Flags::MaskCommand,
    }
}

fn keys_pressed(seq: &KeySeq, flags: Flags, pressed: impl Fn(u16) -> bool) -> bool {
    required_keys(seq).is_some_and(|keys| {
        keys.into_iter().all(|key| match key {
            RequiredKey::Modifier(modifier) => flags.contains(modifier_flag(modifier)),
            RequiredKey::Vk(vk) => match vk {
                0x10 => flags.contains(Flags::MaskShift),
                0x11 => flags.contains(Flags::MaskControl),
                0x12 => flags.contains(Flags::MaskAlternate),
                _ => vk_to_kvk(vk).is_some_and(&pressed),
            },
        })
    })
}

struct MacPlatform;
impl LuaPlatform for MacPlatform {
    fn log(&self, message: &str) {
        crate::log!("{message}");
    }
    fn unsupported(&self, name: &str) {
        static WARNED: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());
        if WARNED.lock().is_ok_and(|mut seen| seen.insert(name.into())) {
            // i18n:allow-begin 仅写运行日志的 Lua 能力诊断，不用于界面。
            let reason = match name {
                "set_win_top" => "macOS 没有公开的跨应用置顶 API",
                "get_brightness" | "set_brightness" => "现有 macOS 亮度后端仅发送内建屏幕亮度键，不能读取百分比或按百分点调整；不支持外接 DDC",
                _ => "not supported on macOS",
            };
            // i18n:allow-end
            crate::log!("Lua {name}: {reason}");
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
    fn is_keys_pressed(&self, seq: &KeySeq) -> Option<bool> {
        if !accessibility() {
            return Some(false);
        }
        let state = CGEventSourceStateID::CombinedSessionState;
        let flags = CGEventSource::flags_state(state);
        Some(keys_pressed(seq, flags, |code| {
            CGEventSource::key_state(state, code)
        }))
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
    fn find_windows(
        &self,
        class: Option<&str>,
        title: Option<&str>,
    ) -> Option<Vec<(String, isize)>> {
        if !accessibility() {
            return None;
        }
        let matcher = |pattern: Option<&str>| pattern.map(regex::Regex::new).transpose();
        let (Ok(class), Ok(title)) = (matcher(class), matcher(title)) else {
            return Some(Vec::new());
        };
        let windows = crate::targeting::lua_windows(true);
        let candidates = on_main(move || {
            windows
                .into_iter()
                .filter(|w| w.layer == 0)
                .filter_map(|w| {
                    let app = NSRunningApplication::runningApplicationWithProcessIdentifier(
                        w.pid as i32,
                    )?;
                    let bundle = app
                        .bundleIdentifier()
                        .map(|s| s.to_string())
                        .unwrap_or_default();
                    class
                        .as_ref()
                        .is_none_or(|pattern| pattern.is_match(&bundle))
                        .then_some(w)
                })
                .collect::<Vec<_>>()
        })?;
        Some(
            candidates
                .into_iter()
                .filter_map(|w| {
                    let text = crate::ax::lua_window_title(&w)?;
                    title
                        .as_ref()
                        .is_none_or(|pattern| pattern.is_match(&text))
                        .then_some((text, w.wid as isize))
                })
                .take(32)
                .collect(),
        )
    }
    fn focus_window(&self, wid: i64) {
        if !accessibility() {
            return;
        }
        let Some(window) = window_by_id(wid) else {
            return;
        };
        if crate::ax::lua_focus_window(&window) {
            on_main(move || crate::actions::activate(window.pid));
        }
    }
    fn set_window_state(&self, wid: i64, state: &str) {
        if !matches!(
            state,
            "min" | "max" | "normal" | "norm" | "restore" | "fullscreen"
        ) {
            self.unsupported("set_win_state: unknown macOS state");
            return;
        }
        if !accessibility() {
            return;
        }
        let Some(window) = window_by_id(wid) else {
            return;
        };
        let Some(work_areas) = on_main(crate::host::work_areas) else {
            return;
        };
        crate::ax::lua_set_window_state(&window, state, &work_areas);
    }
    fn window_state(&self, wid: i64) -> Option<String> {
        if !accessibility() {
            return None;
        }
        let window = window_by_id(wid)?;
        let work_areas = on_main(crate::host::work_areas)?;
        crate::ax::lua_window_state(&window, &work_areas)
    }
    fn set_topmost(&self, _wid: i64, _top: bool) {
        self.unsupported("set_win_top");
    }
    fn get_brightness(&self, _x: i32, _y: i32) -> Option<i64> {
        self.unsupported("get_brightness");
        None
    }
    fn adjust_brightness(&self, _x: i32, _y: i32, _delta: i32) -> Option<i64> {
        self.unsupported("set_brightness");
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn key_queries_distinguish_control_command_and_explicit_sides() {
        let query = |text: &str, flags, codes: &[u16]| {
            keys_pressed(
                &mousetroke_core::keyseq::parse(text, false).unwrap(),
                flags,
                |code| codes.contains(&code),
            )
        };
        assert!(!query("^a", Flags::empty(), &[0x00]));
        assert!(query("^a", Flags::MaskControl, &[0x00]));
        assert!(!query("^a", Flags::MaskCommand, &[0x00]));
        assert!(query("#a", Flags::MaskCommand, &[0x00]));
        assert!(query(
            "%+a",
            Flags::MaskAlternate | Flags::MaskShift,
            &[0x00]
        ));
        assert!(query("{CTRL}", Flags::MaskControl, &[]));
        assert!(query("{RCTRL}a", Flags::MaskControl, &[0x3E, 0x00]));
        assert!(!query("{RCTRL}a", Flags::MaskControl, &[0x3B, 0x00]));
        assert!(!query("^ab", Flags::MaskControl, &[0x00]));
        assert!(query("^ab", Flags::MaskControl, &[0x00, 0x0B]));
        assert!(!query("{INSERT}", Flags::empty(), &[]));
        assert!(!query("a{SLEEP 1}", Flags::empty(), &[0x00]));
        assert!(query("", Flags::empty(), &[]));
    }

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
