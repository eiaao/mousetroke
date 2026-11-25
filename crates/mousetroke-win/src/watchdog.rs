//! 卡顿与异常退出记录：引擎线程长时间不响应时写日志（附最后在做什么），
//! Rust panic 与未处理的系统异常在进程结束前同步写进日志。
//! 只做记录，不尝试恢复：用户反馈时日志能说明卡在哪一步、为什么退出。
use std::io::Write;
use std::sync::atomic::{AtomicIsize, AtomicU32, AtomicU8, Ordering};
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{SendMessageTimeoutW, SMTO_ABORTIFHUNG, WM_NULL};

/// 引擎线程当前所处的步骤（卡住时写进日志）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Stage {
    Idle,
    MouseHook,
    KeyboardHook,
    SyncDisplay,
    TrayTimer,
    CheckHooks,
    ConfigTimer,
    DisplayChange,
    TrailFade,
    /// 覆盖层操作；附加信息 1 开始 2 画线 3 结束 4 淡出 5 淡出帧 6 标签 7 清标签。
    Overlay,
    Screens,
    Target,
    Fullscreen,
    Effect,
    TrimMemory,
}

impl Stage {
    fn name(self) -> &'static str {
        match self {
            Stage::Idle => "idle",
            Stage::MouseHook => "mouse hook",
            Stage::KeyboardHook => "keyboard hook",
            Stage::SyncDisplay => "display sync",
            Stage::TrayTimer => "tray timer",
            Stage::CheckHooks => "hook check",
            Stage::ConfigTimer => "config poll",
            Stage::DisplayChange => "display change",
            Stage::TrailFade => "trail fade",
            Stage::Overlay => "overlay",
            Stage::Screens => "monitor query",
            Stage::Target => "window under cursor",
            Stage::Fullscreen => "fullscreen check",
            Stage::Effect => "fun effect",
            Stage::TrimMemory => "trim memory",
        }
    }

    fn from_u8(v: u8) -> Self {
        const ALL: [Stage; 15] = [
            Stage::Idle,
            Stage::MouseHook,
            Stage::KeyboardHook,
            Stage::SyncDisplay,
            Stage::TrayTimer,
            Stage::CheckHooks,
            Stage::ConfigTimer,
            Stage::DisplayChange,
            Stage::TrailFade,
            Stage::Overlay,
            Stage::Screens,
            Stage::Target,
            Stage::Fullscreen,
            Stage::Effect,
            Stage::TrimMemory,
        ];
        ALL.get(v as usize).copied().unwrap_or(Stage::Idle)
    }
}

static STAGE: AtomicU8 = AtomicU8::new(Stage::Idle as u8);
/// 步骤的附加信息（如鼠标消息号）。
static DETAIL: AtomicU32 = AtomicU32::new(0);
static ENGINE_HWND: AtomicIsize = AtomicIsize::new(0);

/// 进入某一步骤，返回的守卫在离开时恢复上一步骤（可嵌套）。只做两次原子写，钩子回调里可用。
pub fn enter(stage: Stage, detail: u32) -> StageGuard {
    let prev = STAGE.swap(stage as u8, Ordering::Relaxed);
    let prev_detail = DETAIL.swap(detail, Ordering::Relaxed);
    StageGuard(prev, prev_detail)
}

pub struct StageGuard(u8, u32);
impl Drop for StageGuard {
    fn drop(&mut self) {
        STAGE.store(self.0, Ordering::Relaxed);
        DETAIL.store(self.1, Ordering::Relaxed);
    }
}

/// 卡住多久开始记录、之后每隔多久再记一次。
const HUNG_AFTER: Duration = Duration::from_secs(2);
const REPEAT_EVERY: Duration = Duration::from_secs(10);

/// 启动看门狗线程：每秒向引擎线程的窗口发一条空消息，超时即视为卡住。
pub fn start(engine_hwnd: HWND) {
    ENGINE_HWND.store(engine_hwnd.0 as isize, Ordering::Relaxed);
    let _ = std::thread::Builder::new().name("watchdog".into()).spawn(|| {
        let mut hung_since: Option<Instant> = None;
        let mut last_report = Instant::now();
        loop {
            std::thread::sleep(Duration::from_secs(1));
            let hwnd = HWND(ENGINE_HWND.load(Ordering::Relaxed) as *mut _);
            if hwnd.0.is_null() {
                return;
            }
            let asked = Instant::now();
            let mut result = 0usize;
            // 安全：WM_NULL 无参数；超时或对方挂起时返回 0。
            let ok = unsafe {
                SendMessageTimeoutW(hwnd, WM_NULL, WPARAM(0), LPARAM(0), SMTO_ABORTIFHUNG, HUNG_AFTER.as_millis() as u32, Some(&mut result))
            };
            if ok.0 != 0 {
                if let Some(since) = hung_since.take() {
                    crate::log!("引擎线程恢复响应（卡了 {:.1}s）", since.elapsed().as_secs_f32());
                }
                continue;
            }
            if ENGINE_HWND.load(Ordering::Relaxed) == 0 {
                return; // 正在退出，窗口已销毁
            }
            let stage = Stage::from_u8(STAGE.load(Ordering::Relaxed));
            let detail = DETAIL.load(Ordering::Relaxed);
            match hung_since {
                None => {
                    hung_since = Some(asked);
                    last_report = Instant::now();
                    crate::log!("引擎线程无响应超过 {}s，正在：{} (0x{detail:X})", HUNG_AFTER.as_secs(), stage.name());
                }
                Some(since) if last_report.elapsed() >= REPEAT_EVERY => {
                    last_report = Instant::now();
                    crate::log!("引擎线程仍无响应（{:.0}s），正在：{} (0x{detail:X})", since.elapsed().as_secs_f32(), stage.name());
                }
                Some(_) => {}
            }
        }
    });
}

/// 引擎退出前调用：看门狗不再把关窗当成卡住。
pub fn stop() {
    ENGINE_HWND.store(0, Ordering::Relaxed);
}

/// 安装 panic 记录与未处理异常记录。两者都同步追加到日志文件：随后进程可能立刻结束，
/// 后台写日志线程来不及写盘。
pub fn install_crash_logging() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let message = info
            .payload()
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| info.payload().downcast_ref::<String>().map(String::as_str))
            .unwrap_or("non-string payload");
        let thread = std::thread::current();
        let location = info.location().map(|l| format!("{}:{}", l.file(), l.line())).unwrap_or_default();
        append_now(&format!("Rust panic（线程 {}）：{message}，位置 {location}", thread.name().unwrap_or("?"))); // i18n:allow 运行日志（与 log! 相同），同步写盘
        previous(info);
    }));
    // 安全：注册进程级回调；回调只读异常记录并写文件。
    unsafe {
        windows::Win32::System::Diagnostics::Debug::SetUnhandledExceptionFilter(Some(unhandled));
    }
}

unsafe extern "system" fn unhandled(info: *const windows::Win32::System::Diagnostics::Debug::EXCEPTION_POINTERS) -> i32 {
    // 安全：系统传入的指针在回调期间有效；为空时只记一句。
    let record = unsafe { info.as_ref().and_then(|p| p.ExceptionRecord.as_ref()) };
    match record {
        Some(r) => {
            let addr = r.ExceptionAddress as usize;
            let base = unsafe { windows::Win32::System::LibraryLoader::GetModuleHandleW(None) }.map(|h| h.0 as usize).unwrap_or(0);
            let offset = if base != 0 && addr >= base { format!("mousetroke+0x{:X}", addr - base) } else { format!("0x{addr:X}") };
            append_now(&format!("未处理的系统异常 0x{:08X}，地址 {offset}，进程即将退出", r.ExceptionCode.0 as u32)); // i18n:allow 运行日志（与 log! 相同），同步写盘
        }
        None => append_now("未处理的系统异常，进程即将退出"), // i18n:allow 运行日志（与 log! 相同），同步写盘
    }
    0 // EXCEPTION_CONTINUE_SEARCH：交给系统按默认方式结束进程
}

/// 绕过后台队列，直接追加一行到日志文件。
fn append_now(text: &str) {
    let t = unsafe { windows::Win32::System::SystemInformation::GetLocalTime() };
    let line = format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03} {text}\n",
        t.wYear, t.wMonth, t.wDay, t.wHour, t.wMinute, t.wSecond, t.wMilliseconds
    );
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(crate::log::path()) {
        let _ = f.write_all(line.as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_guard_restores_previous_stage() {
        {
            let _outer = enter(Stage::MouseHook, 0x201);
            {
                let _inner = enter(Stage::SyncDisplay, 0);
                assert_eq!(Stage::from_u8(STAGE.load(Ordering::Relaxed)), Stage::SyncDisplay);
            }
            assert_eq!(Stage::from_u8(STAGE.load(Ordering::Relaxed)), Stage::MouseHook);
            assert_eq!(DETAIL.load(Ordering::Relaxed), 0x201);
        }
        assert_eq!(Stage::from_u8(STAGE.load(Ordering::Relaxed)), Stage::Idle);
        assert_eq!(Stage::from_u8(200), Stage::Idle);
    }

    /// 手动验证：cargo test -p mousetroke-win watchdog_reports -- --ignored --nocapture，
    /// 控制台应出现“无响应…正在：trail fade”和“恢复响应”两行。
    #[test]
    #[ignore]
    fn watchdog_reports_hung_window_and_recovery() {
        use windows::Win32::UI::WindowsAndMessaging::{CreateWindowExW, DispatchMessageW, PeekMessageW, HWND_MESSAGE, MSG, PM_REMOVE, WINDOW_EX_STYLE, WINDOW_STYLE};
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || unsafe {
            let hwnd = CreateWindowExW(WINDOW_EX_STYLE(0), windows::core::w!("STATIC"), None, WINDOW_STYLE(0), 0, 0, 0, 0, Some(HWND_MESSAGE), None, None, None).unwrap();
            tx.send(hwnd.0 as isize).unwrap();
            let pump = |secs: f32| {
                let end = Instant::now() + Duration::from_secs_f32(secs);
                while Instant::now() < end {
                    let mut msg = MSG::default();
                    while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                        DispatchMessageW(&msg);
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
            };
            pump(1.5);
            let _stage = enter(Stage::TrailFade, 7);
            std::thread::sleep(Duration::from_secs(4));
            drop(_stage);
            pump(3.0);
        });
        start(HWND(rx.recv().unwrap() as *mut _));
        std::thread::sleep(Duration::from_secs(9));
        stop();
    }
}
