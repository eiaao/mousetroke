//! 低级钩子安装与引擎线程消息循环。
//!
//! 关键事实：`WH_MOUSE_LL` / `WH_KEYBOARD_LL` 的回调在**安装钩子的线程**
//! 的消息循环中执行——因此回调与引擎通过 `thread_local!` 交互，无锁、零同步。
//! 托盘消息窗口也驻留本线程（见 [`crate::tray`]）。

use std::cell::Cell;
use std::sync::atomic::{AtomicU32, Ordering};

use mousetroke_core::schema::Config;
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::System::SystemInformation::GetTickCount;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, HHOOK,
    KBDLLHOOKSTRUCT, MSLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN,
    WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL,
    WM_RBUTTONDOWN, WM_RBUTTONUP, LLMHF_INJECTED, WM_SYSKEYDOWN, WM_XBUTTONDOWN, WM_XBUTTONUP, XBUTTON1,
};

use mousetroke_engine::{InputEvent, Verdict};
use crate::input::INJECT_TAG;
use crate::tray::Tray;

/// 启动引擎线程并阻塞（进程退出即卸载钩子）。返回 Err 表示初始化失败。
pub fn run(cfg: Config, cfg_path: Option<std::path::PathBuf>) -> Result<(), String> {
    // 单实例：命名互斥体
    single_instance_guard()?;
    crate::update::cleanup();
    let (tx, rx) = std::sync::mpsc::channel::<Result<(), String>>();
    let handle = std::thread::Builder::new()
        .name("mousetroke-engine".into())
        .spawn(move || {
            let init = engine_main(cfg, cfg_path);
            let _ = tx.send(init);
        })
        .map_err(|e| e.to_string())?;

    match rx.recv() {
        Ok(result) => {
            let _ = handle.join();
            result
        }
        Err(_) => Err(mousetroke_core::i18n::t(mousetroke_core::i18n::keys::CLI_THREAD_DIED).into()),
    }
}

/// 引擎是否在运行（检测单实例互斥体，任意进程可调用）。
pub fn engine_running() -> bool {
    use windows::Win32::System::Threading::{OpenMutexW, SYNCHRONIZATION_SYNCHRONIZE};
    let name: Vec<u16> = "Mousetroke::single-instance".encode_utf16().chain([0]).collect();
    unsafe {
        match OpenMutexW(SYNCHRONIZATION_SYNCHRONIZE, false, windows::core::PCWSTR(name.as_ptr())) {
            Ok(h) => {
                let _ = windows::Win32::Foundation::CloseHandle(h);
                true
            }
            Err(_) => false,
        }
    }
}

fn single_instance_guard() -> Result<(), String> {
    use windows::Win32::System::Threading::CreateMutexW;
    use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
    let mut name: Vec<u16> = "Mousetroke::single-instance".encode_utf16().chain([0]).collect();
    unsafe {
        let _handle = CreateMutexW(None, false, windows::core::PCWSTR(name.as_mut_ptr()))
            .unwrap_or_default();
        if GetLastError() == ERROR_ALREADY_EXISTS {
            return Err(mousetroke_core::i18n::t(mousetroke_core::i18n::keys::CLI_SINGLE_INSTANCE).into());
        }
    }
    Ok(())
}

fn engine_main(cfg: Config, cfg_path: Option<std::path::PathBuf>) -> Result<(), String> {
    crate::log::init();
    crate::log!("Mousetroke {} 启动（pid {}）", env!("CARGO_PKG_VERSION"), std::process::id());
    // 每显示器 DPI 感知：钩子坐标与虚拟屏度量保持物理像素一致
    unsafe {
        let _ = windows::Win32::UI::HiDpi::SetProcessDpiAwarenessContext(
            windows::Win32::UI::HiDpi::DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        );
    }

    let pause_hotkey = cfg.settings.pause_hotkey.clone();
    let show_tray = cfg.settings.show_tray;
    let (update_check, update_notify) = (cfg.settings.update_check, cfg.settings.update_notify);
    let engine = crate::engine::new(cfg, cfg_path.clone()).ok_or_else(|| mousetroke_core::i18n::t(mousetroke_core::i18n::keys::CLI_OVERLAY_INIT_FAILED).to_string())?;
    mousetroke_engine::install(engine);
    if cfg_path.as_ref().is_some_and(|path| path.exists() && Config::load(path).is_err()) {
        crate::engine::push_toast(mousetroke_core::i18n::t("config.unreadable_notice").into(), 8000);
    }

    // 托盘 + 暂停热键
    let _tray = Tray::create(show_tray, &pause_hotkey, cfg_path.clone());
    if let Some(path) = cfg_path { crate::update::start_automatic(path, update_check, update_notify); }

    install_hooks()?;
    crate::log!("引擎已启动：钩子/托盘/热键就绪（托盘菜单或 Ctrl+C 退出）");
    unsafe {
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    unhook();
    Ok(())
}

/// 钩子最近一次收到事件的系统 tick（鼠标、键盘共用）。
/// 共用意味着只丢了鼠标钩子、而用户一直在打字时看不出缺口；一旦有鼠标输入，
/// 系统输入就比钩子新，缺口出现，约 1 秒后重装。手势本来就要用鼠标，只是恢复晚一点，不会一直失效。
static LAST_HOOK_TICK: AtomicU32 = AtomicU32::new(0);

thread_local! {
    /// 当前钩子句柄（鼠标, 键盘），只在引擎线程读写。
    static HOOKS: Cell<Option<(HHOOK, HHOOK)>> = const { Cell::new(None) };
    static LAST_INSTALL_TICK: Cell<u32> = const { Cell::new(0) };
    /// 第一次发现"系统输入比钩子新"的时刻；缺口消失时清空。
    static GAP_SINCE: Cell<Option<u32>> = const { Cell::new(None) };
}

/// 安装两个 LL 钩子；必须在跑消息循环的引擎线程调用。
fn install_hooks() -> Result<(), String> {
    use mousetroke_core::i18n::{keys, tf};
    unsafe {
        let mouse = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), None, 0)
            .map_err(|e| tf(keys::CLI_HOOK_MOUSE_FAILED, &[("err", &e.to_string())]))?;
        let kbd = match SetWindowsHookExW(WH_KEYBOARD_LL, Some(kbd_proc), None, 0) {
            Ok(h) => h,
            Err(e) => {
                let _ = UnhookWindowsHookEx(mouse);
                return Err(tf(keys::CLI_HOOK_KBD_FAILED, &[("err", &e.to_string())]));
            }
        };
        HOOKS.set(Some((mouse, kbd)));
        let now = GetTickCount();
        LAST_HOOK_TICK.store(now, Ordering::Relaxed);
        LAST_INSTALL_TICK.set(now);
    }
    Ok(())
}

fn unhook() {
    if let Some((mouse, kbd)) = HOOKS.take() {
        unsafe {
            let _ = UnhookWindowsHookEx(mouse);
            let _ = UnhookWindowsHookEx(kbd);
        }
    }
}

/// "系统输入比钩子新"持续这么久 → 判定钩子已被系统摘除（短暂的回调排队不算）。
const HOOK_SETTLE_MS: u32 = 1000;
/// 两次重装的最小间隔。
const HOOK_RETRY_MS: u32 = 30_000;

/// 系统有比钩子更新的输入（钩子没见到）。tick 为 `GetTickCount` 的 u32 毫秒，按回绕差值比较。
fn input_unseen(last_hook: u32, last_input: u32) -> bool {
    (last_input.wrapping_sub(last_hook) as i32) > 0
}

/// 钩子是否已被系统摘除（回调超过 LowLevelHooksTimeout 时 Windows 会静默卸掉 LL 钩子）。
/// 从第一次发现缺口算起已过去足够久（钩子活着早该收到），并距上次安装够久。
/// 按缺口出现的时刻计时，而不是按最近一次输入：持续操作不会把重装一直往后推。
fn hooks_lost(gap_since: u32, now: u32, last_install: u32) -> bool {
    now.wrapping_sub(gap_since) >= HOOK_SETTLE_MS && now.wrapping_sub(last_install) >= HOOK_RETRY_MS
}

/// 前台是管理员窗口（而本程序不是）时返回其进程号：系统不调用我们的钩子，钩子没见到输入是正常的。
fn elevated_foreground() -> Option<u32> {
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};
    let mut pid = 0;
    unsafe { GetWindowThreadProcessId(GetForegroundWindow(), Some(&mut pid)) };
    (pid != 0 && crate::engine::elevated_beyond_us(pid)).then_some(pid)
}

/// 定时调用（引擎线程）：钩子被系统摘除时重新安装。
pub fn check_hooks() {
    let mut info = LASTINPUTINFO { cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32, dwTime: 0 };
    if unsafe { !GetLastInputInfo(&mut info).as_bool() } {
        return;
    }
    // 管理员窗口在前台时钩子收不到按键，手势无从触发：它到前台时提示原因（每个进程一次）
    let elevated = elevated_foreground();
    if let Some(pid) = elevated {
        crate::engine::warn_if_elevated(pid);
    }
    let now = unsafe { GetTickCount() };
    let last_hook = LAST_HOOK_TICK.load(Ordering::Relaxed);
    if elevated.is_some() || !input_unseen(last_hook, info.dwTime) {
        GAP_SINCE.set(None);
        return;
    }
    let gap_since = GAP_SINCE.get().unwrap_or(now);
    GAP_SINCE.set(Some(gap_since));
    if !hooks_lost(gap_since, now, LAST_INSTALL_TICK.get()) {
        return;
    }
    unhook();
    GAP_SINCE.set(None);
    match install_hooks() {
        Ok(()) => {
            // 丢钩子时可能正在画手势（松开没收到）：清掉残留的捕获，下一次按下从头开始
            crate::engine::with_engine(|e| e.cancel_capture());
            crate::log!("钩子疑似被系统摘除（{}ms 未收到输入），已重新安装", now.wrapping_sub(last_hook))
        }
        Err(e) => {
            LAST_INSTALL_TICK.set(now);
            crate::log!("钩子疑似被系统摘除，重新安装失败：{e}");
        }
    }
}

/// 写日志用的事件描述：键盘事件不带按键码。日志是明文落盘的，
/// 记下按了哪个键就等于键盘记录，排查问题也用不到具体是哪个键。
fn for_log(ev: &InputEvent) -> String {
    match ev {
        InputEvent::KeyDown(_) => "KeyDown".into(),
        other => format!("{other:?}"),
    }
}

/// 引擎事件转发（吞下 → 1，放行 → 链上）。
fn forward(ev: InputEvent, injected: bool, code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // try_borrow_mut：若在回调内被系统重入（如 SendInput 同步分发），
    // 直接放行该事件，绝不在 extern "system" 里 panic（会 abort 整个进程）。
    let t0 = std::time::Instant::now();
    let verdict = crate::engine::with_engine(|e| e.handle_input(ev, injected));
    let swallow = verdict == Some(Verdict::Swallow);
    // 回调太慢会被系统判超时：事件不等我们的结果就放行（右键菜单/点击错乱的根源），记下来
    let ms = t0.elapsed().as_millis();
    if ms >= 20 {
        crate::log!("钩子回调耗时 {ms}ms：{} → {verdict:?}", for_log(&ev));
    }
    if verdict.is_none() {
        crate::log!("钩子重入，事件直接放行：{}", for_log(&ev));
    }
    if swallow {
        LRESULT(1)
    } else {
        unsafe { CallNextHookEx(None, code, wparam, lparam) }
    }
}

unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    LAST_HOOK_TICK.store(GetTickCount(), Ordering::Relaxed);
    if code < 0 {
        return CallNextHookEx(None, code, wparam, lparam);
    }
    let info = &*(lparam.0 as *const MSLLHOOKSTRUCT);
    if info.dwExtraInfo == INJECT_TAG {
        return CallNextHookEx(None, code, wparam, lparam);
    }

    use mousetroke_core::schema::MouseButton as B;
    let (x, y) = (info.pt.x, info.pt.y);
    // LL 钩子的 wparam 只是消息号；XBUTTON 编号与滚轮 delta 在 mouseData 高字
    let hi = (info.mouseData >> 16) as u16;
    let xbtn = || if hi == XBUTTON1 { B::X1 } else { B::X2 };
    let ev = match wparam.0 as u32 {
        WM_MOUSEMOVE => InputEvent::MouseMove { x, y },
        WM_LBUTTONDOWN => InputEvent::MouseDown { btn: B::Left, x, y },
        WM_LBUTTONUP => InputEvent::MouseUp { btn: B::Left, x, y },
        WM_RBUTTONDOWN => InputEvent::MouseDown { btn: B::Right, x, y },
        WM_RBUTTONUP => InputEvent::MouseUp { btn: B::Right, x, y },
        WM_MBUTTONDOWN => InputEvent::MouseDown { btn: B::Middle, x, y },
        WM_MBUTTONUP => InputEvent::MouseUp { btn: B::Middle, x, y },
        WM_XBUTTONDOWN => InputEvent::MouseDown { btn: xbtn(), x, y },
        WM_XBUTTONUP => InputEvent::MouseUp { btn: xbtn(), x, y },
        // 正值 = 向上滚（远离用户）；引擎约定 正=向下，故取反
        WM_MOUSEWHEEL => InputEvent::Wheel(-(hi as i16 as i32).signum()),
        _ => return CallNextHookEx(None, code, wparam, lparam),
    };
    if matches!(ev, InputEvent::MouseDown { .. }) {
        crate::engine::sync_display();
    }
    forward(ev, info.flags & LLMHF_INJECTED != 0, code, wparam, lparam)
}

unsafe extern "system" fn kbd_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    LAST_HOOK_TICK.store(GetTickCount(), Ordering::Relaxed);
    if code < 0 {
        return CallNextHookEx(None, code, wparam, lparam);
    }
    let info = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
    if info.dwExtraInfo == INJECT_TAG {
        return CallNextHookEx(None, code, wparam, lparam);
    }
    if (wparam.0 as u32) == WM_KEYDOWN || (wparam.0 as u32) == WM_SYSKEYDOWN {
        return forward(
            InputEvent::KeyDown(info.vkCode as u16),
            false,
            code,
            wparam,
            lparam,
        );
    }
    CallNextHookEx(None, code, wparam, lparam)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_unseen_compares_ticks_with_wraparound() {
        let t = 100_000;
        assert!(!input_unseen(t, t));
        assert!(!input_unseen(t + 10, t));
        assert!(input_unseen(t, t + 10));
        // 输入在回绕之后：仍判定为更新；钩子 tick 在回绕之后：钩子见到了
        assert!(input_unseen(u32::MAX - 100, 50));
        assert!(!input_unseen(50, u32::MAX - 100));
    }

    #[test]
    fn hooks_lost_counts_from_when_the_gap_started() {
        let t = 100_000;
        // 缺口刚出现：钩子回调可能还在排队
        assert!(!hooks_lost(t, t + 500, 0));
        // 缺口持续 1s 以上（期间是否还有新输入不影响）
        assert!(hooks_lost(t, t + 1000, 0));
        // 距上次安装不足 30s：限频
        assert!(!hooks_lost(t, t + 1000, t + 1000 - 29_999));
        assert!(hooks_lost(t, t + 1000, t + 1000 - 30_000));
        // 跨回绕
        assert!(hooks_lost(u32::MAX - 500, 600, u32::MAX - 40_000));
        assert!(!hooks_lost(u32::MAX - 500, 400, u32::MAX - 40_000));
    }
}
