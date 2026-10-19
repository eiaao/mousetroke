//! 低级钩子安装与引擎线程消息循环。
//!
//! 关键事实：`WH_MOUSE_LL` / `WH_KEYBOARD_LL` 的回调在**安装钩子的线程**
//! 的消息循环中执行——因此回调与引擎通过 `thread_local!` 交互，无锁、零同步。
//! 托盘消息窗口也驻留本线程（见 [`crate::tray`]）。

use mousetroke_core::schema::Config;
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage,
    KBDLLHOOKSTRUCT, MSLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN,
    WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL,
    WM_RBUTTONDOWN, WM_RBUTTONUP, LLMHF_INJECTED, WM_SYSKEYDOWN, WM_XBUTTONDOWN, WM_XBUTTONUP, XBUTTON1,
};

use crate::engine::{Engine, InputEvent, Verdict};
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
    let update_check = cfg.settings.update_check;
    let engine = Engine::new(cfg, cfg_path.clone()).ok_or_else(|| mousetroke_core::i18n::t(mousetroke_core::i18n::keys::CLI_OVERLAY_INIT_FAILED).to_string())?;
    crate::engine::ENGINE.with(|e| *e.borrow_mut() = Some(engine));

    // 托盘 + 暂停热键
    let _tray = Tray::create(show_tray, &pause_hotkey, cfg_path.clone());
    if let Some(path) = cfg_path { crate::update::start_automatic(path, update_check); }

    unsafe {
        let mouse_hook = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), None, 0)
            .map_err(|e| mousetroke_core::i18n::tf(mousetroke_core::i18n::keys::CLI_HOOK_MOUSE_FAILED, &[("err", &e.to_string())]))?;
        let kbd_hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(kbd_proc), None, 0)
            .map_err(|e| mousetroke_core::i18n::tf(mousetroke_core::i18n::keys::CLI_HOOK_KBD_FAILED, &[("err", &e.to_string())]))?;

        crate::log!("引擎已启动：钩子/托盘/热键就绪（托盘菜单或 Ctrl+C 退出）");

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        let _ = windows::Win32::UI::WindowsAndMessaging::UnhookWindowsHookEx(mouse_hook);
        let _ = windows::Win32::UI::WindowsAndMessaging::UnhookWindowsHookEx(kbd_hook);
    }
    Ok(())
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
    forward(ev, info.flags & LLMHF_INJECTED != 0, code, wparam, lparam)
}

unsafe extern "system" fn kbd_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
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
