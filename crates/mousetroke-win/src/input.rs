//! 输入注入与进程执行（SendInput / cmd / 窗口操作）。
//!
//! 自注入事件通过 `dw_extra_info` 打上 [`INJECT_TAG`] 标记，
//! 钩子回调据此放行，避免自触发递归。

use mousetroke_core::keyseq::{Action, Key, KeySeq, ModKey, MouseKey};
use windows::core::BOOL;
use windows::Win32::Foundation::{CloseHandle, HANDLE, HGLOBAL, HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::HMENU as HMENU_W;
use windows::Win32::System::Threading::{CreateProcessW, PROCESS_INFORMATION, STARTUPINFOW};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYEVENTF_KEYUP,
    KEYBD_EVENT_FLAGS, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN,
    MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL, MOUSEEVENTF_HWHEEL,
    MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP, MOUSE_EVENT_FLAGS, MOUSEINPUT, VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DestroyMenu, GetWindowTextW, PostMessageW, SetForegroundWindow, SetWindowPos, ShowWindow,
    HWND_NOTOPMOST, HWND_TOPMOST, MF_POPUP, MF_STRING, SW_SHOWNORMAL, SW_HIDE, SW_MAXIMIZE,
    SW_MINIMIZE, SW_RESTORE, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, WM_CLOSE, WS_EX_TOOLWINDOW,
    WS_OVERLAPPED,
};

use mousetroke_core::schema::{MouseButton, WindowOpKind};

/// 自注入事件标记（"MSTROKE"），钩子见此直接放行。
pub const INJECT_TAG: usize = 0x4D5F_5354_524F_4B45;

fn mouse_button_flags(btn: MouseButton, down: bool) -> (MOUSE_EVENT_FLAGS, u32) {
    match (btn, down) {
        (MouseButton::Left, true) => (MOUSEEVENTF_LEFTDOWN, 0),
        (MouseButton::Left, false) => (MOUSEEVENTF_LEFTUP, 0),
        (MouseButton::Right, true) => (MOUSEEVENTF_RIGHTDOWN, 0),
        (MouseButton::Right, false) => (MOUSEEVENTF_RIGHTUP, 0),
        (MouseButton::Middle, true) => (MOUSEEVENTF_MIDDLEDOWN, 0),
        (MouseButton::Middle, false) => (MOUSEEVENTF_MIDDLEUP, 0),
        (MouseButton::X1, down) => {
            let base = if down { MOUSEEVENTF_XDOWN } else { MOUSEEVENTF_XUP };
            (base, 0x0001) // XBUTTON1
        }
        (MouseButton::X2, down) => {
            let base = if down { MOUSEEVENTF_XDOWN } else { MOUSEEVENTF_XUP };
            (base, 0x0002) // XBUTTON2
        }
    }
}

/// 发送一次鼠标按下/释放。
pub fn send_mouse(btn: MouseButton, down: bool) {
    let (flags, data) = mouse_button_flags(btn, down);
    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: data,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: INJECT_TAG,
            },
        },
    };
    unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };
}

/// 发送滚轮。`delta` 为 Windows 原始单位（120 = 一格，正 = 向上/向右）。
pub fn send_wheel(delta: i32, horizontal: bool) {
    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: delta as u32,
                dwFlags: if horizontal { MOUSEEVENTF_HWHEEL } else { MOUSEEVENTF_WHEEL },
                time: 0,
                dwExtraInfo: INJECT_TAG,
            },
        },
    };
    unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };
}

fn send_vk(vk: u16, down: bool) {
    let input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: windows::Win32::UI::Input::KeyboardAndMouse::KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: 0,
                dwFlags: if down { KEYBD_EVENT_FLAGS(0) } else { KEYEVENTF_KEYUP },
                time: 0,
                dwExtraInfo: INJECT_TAG,
            },
        },
    };
    unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };
}

fn mod_vk(m: ModKey) -> u16 {
    match m {
        ModKey::Ctrl => 0x11,
        ModKey::Alt => 0x12,
        ModKey::Shift => 0x10,
        ModKey::Win => 0x5B,
    }
}

/// 按键注入阶段：完整按下抬起 / 只按下 / 只抬起（Lua KeysDown/KeysUp 用）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyPhase {
    DownUp,
    Down,
    Up,
}

/// 注入完整按键序列（含 SLEEP 延时）。阻塞调用——在手势完成后由工作线程执行。
pub fn inject_keyseq(seq: &KeySeq) {
    inject_keyseq_phase(seq, KeyPhase::DownUp);
}

fn key_event(key: &Key, down: bool) {
    match key {
        Key::Vk(vk) => send_vk(*vk, down),
        Key::Mouse(m) => {
            let btn = match m {
                MouseKey::Left => MouseButton::Left,
                MouseKey::Right => MouseButton::Right,
                MouseKey::Middle => MouseButton::Middle,
                MouseKey::X1 => MouseButton::X1,
                MouseKey::X2 => MouseButton::X2,
                // 滚轮没有按下/抬起之分：只在"按下"阶段发送一次
                MouseKey::Wheel(d) => return if down { send_wheel(*d, false) },
                MouseKey::HWheel(d) => return if down { send_wheel(*d, true) },
            };
            send_mouse(btn, down);
        }
    }
}

/// 按阶段注入按键序列。`Up` 阶段逆序释放，保证修饰键最后抬起。
pub fn inject_keyseq_phase(seq: &KeySeq, phase: KeyPhase) {
    let actions: Vec<&Action> = if phase == KeyPhase::Up { seq.actions.iter().rev().collect() } else { seq.actions.iter().collect() };
    for action in actions {
        match action {
            Action::SleepMs(ms) => std::thread::sleep(std::time::Duration::from_millis(*ms as u64)),
            Action::Text(text) => {
                if phase != KeyPhase::Up {
                    send_text(text);
                }
            }
            Action::Key { key, mods, repeat } => match phase {
                KeyPhase::DownUp => {
                    mods.iter().for_each(|m| send_vk(mod_vk(*m), true));
                    for _ in 0..*repeat {
                        key_event(key, true);
                        key_event(key, false);
                    }
                    mods.iter().rev().for_each(|m| send_vk(mod_vk(*m), false));
                }
                KeyPhase::Down => {
                    mods.iter().for_each(|m| send_vk(mod_vk(*m), true));
                    key_event(key, true);
                }
                KeyPhase::Up => {
                    key_event(key, false);
                    mods.iter().rev().for_each(|m| send_vk(mod_vk(*m), false));
                }
            },
        }
    }
}

/// 以 Unicode 方式输入文本（不受键盘布局/输入法影响，中文与符号都可）。
pub fn send_text(text: &str) {
    use windows::Win32::UI::Input::KeyboardAndMouse::{KEYBDINPUT, KEYEVENTF_UNICODE};
    let mut inputs: Vec<INPUT> = Vec::with_capacity(text.len() * 2);
    for unit in text.encode_utf16() {
        for up in [false, true] {
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(0),
                        wScan: unit,
                        dwFlags: if up { KEYEVENTF_UNICODE | KEYEVENTF_KEYUP } else { KEYEVENTF_UNICODE },
                        time: 0,
                        dwExtraInfo: INJECT_TAG,
                    },
                },
            });
        }
    }
    if !inputs.is_empty() {
        unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    }
}

/// 把被吞掉的触发键点击补发（超时放行场景）。
/// 补发按下（拖拽超时放行：之后真实的抬起事件自然放行，保留原生拖拽）。
pub fn replay_down(btn: MouseButton) {
    send_mouse(btn, true);
}

pub fn replay_click(btn: MouseButton) {
    send_mouse(btn, true);
    send_mouse(btn, false);
}

/// 运行 cmd 脚本（隐藏窗口）。
pub fn run_shell(script: &str) {
    use std::os::windows::ffi::OsStrExt;
    let cmdline = format!("cmd.exe /C {script}");
    let mut wide: Vec<u16> = cmdline.encode_utf16().chain([0]).collect();
    // 程序名必须给完整路径：留空时 CreateProcess 会在本程序目录和当前目录里找 cmd.exe
    let app: Vec<u16> = system_exe(r"System32\cmd.exe").as_os_str().encode_wide().chain([0]).collect();
    let si = STARTUPINFOW {
        cb: std::mem::size_of::<STARTUPINFOW>() as u32,
        dwFlags: windows::Win32::System::Threading::STARTUPINFOW_FLAGS(0x0000_0001), // STARTF_USESHOWWINDOW
        wShowWindow: SW_HIDE.0 as u16,
        ..Default::default()
    };
    let mut pi = PROCESS_INFORMATION::default();
    let ok = unsafe {
        CreateProcessW(
            windows::core::PCWSTR(app.as_ptr()),
            Some(windows::core::PWSTR(wide.as_mut_ptr())),
            None,
            None,
            false,
            windows::Win32::System::Threading::CREATE_NO_WINDOW,
            None,
            None,
            &si,
            &mut pi,
        )
    };
    if let Ok(()) = ok {
        unsafe {
            let _ = CloseHandle(pi.hProcess);
            let _ = CloseHandle(pi.hThread);
        }
    } else {
        // 不记命令全文：展开后可能含选中文字、剪贴板内容
        crate::log!("命令启动失败（{} 字符）", script.chars().count());
    }
}

/// 窗口操作。
pub fn apply_window_op(hwnd: HWND, op: WindowOpKind, _allow_child: bool) {
    unsafe {
        match op {
            WindowOpKind::Minimize => {
                let _ = ShowWindow(hwnd, SW_MINIMIZE);
            }
            WindowOpKind::ToggleMaximize => {
                let placement = windows::Win32::UI::WindowsAndMessaging::WINDOWPLACEMENT {
                    length: std::mem::size_of::<
                        windows::Win32::UI::WindowsAndMessaging::WINDOWPLACEMENT,
                    >() as u32,
                    ..Default::default()
                };
                let mut p = placement;
                if windows::Win32::UI::WindowsAndMessaging::GetWindowPlacement(hwnd, &mut p).is_ok()
                    && p.showCmd != SW_MAXIMIZE.0 as u32
                {
                    let _ = ShowWindow(hwnd, SW_MAXIMIZE);
                } else {
                    let _ = ShowWindow(hwnd, SW_RESTORE);
                }
            }
            WindowOpKind::Close => {
                let _ = PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
            }
            WindowOpKind::ToggleTopmost => {
                set_topmost(hwnd, !is_topmost(hwnd));
            }
        }
    }
}

/// 窗口当前是否置顶。
pub fn is_topmost(hwnd: HWND) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::{GetWindowLongPtrW, GWL_EXSTYLE, WS_EX_TOPMOST};
    unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) & WS_EX_TOPMOST.0 as isize != 0 }
}

/// 强制置顶 / 取消置顶。
pub fn set_topmost(hwnd: HWND, on: bool) {
    let after = if on { HWND_TOPMOST } else { HWND_NOTOPMOST };
    unsafe {
        let _ = SetWindowPos(hwnd, Some(after), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
    }
}

/// 激活窗口（执行命令前）。
pub fn focus_window(hwnd: HWND) {
    unsafe {
        let _ = SetForegroundWindow(hwnd);
    }
}

/// 句柄检查（编译期占位，避免未使用告警）。
#[allow(dead_code)]
fn _handle_type_check(h: HANDLE) -> HANDLE {
    h
}


// ================================================================ M4：剪贴板 / URL / 应用 / 菜单

use mousetroke_core::schema::{IfRunning, MenuItem};
use windows::Win32::System::DataExchange::{CloseClipboard, GetClipboardData, OpenClipboard};
use windows::Win32::System::Memory::{GlobalLock, GlobalUnlock};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, DestroyWindow, EnumWindows, IsWindowVisible, TrackPopupMenuEx, WM_NULL,
    TPM_RETURNCMD, TPM_RIGHTBUTTON,
};

/// 读取剪贴板文本。
pub fn get_clipboard_text() -> Option<String> {
    unsafe {
        if OpenClipboard(None).is_err() {
            return None;
        }
        let read = || -> Option<String> {
            let handle = GetClipboardData(13 /* CF_UNICODETEXT */).ok()?;
            if handle.0.is_null() {
                return None;
            }
            let ptr = GlobalLock(HGLOBAL(handle.0)) as *const u16;
            if ptr.is_null() {
                return None;
            }
            let mut len = 0usize;
            while *ptr.add(len) != 0 {
                len += 1;
            }
            let text = String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len));
            let _ = GlobalUnlock(HGLOBAL(handle.0));
            Some(text)
        };
        let result = read();
        let _ = CloseClipboard();
        result
    }
}

/// 通过 Ctrl+C 采集当前选中文字（会覆盖剪贴板，与旧格式语义一致）。
pub fn capture_selected_text() -> Option<String> {
    if let Ok(seq) = mousetroke_core::keyseq::parse("^{C}", false) {
        inject_keyseq(&seq);
        std::thread::sleep(std::time::Duration::from_millis(120));
    }
    get_clipboard_text().filter(|s| !s.trim().is_empty())
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain([0]).collect()
}

/// Windows 自带程序的完整路径（如 `system_exe("explorer.exe")`、
/// `system_exe(r"System32\WindowsPowerShell\v1.0\powershell.exe")`）。
/// 只写文件名时，CreateProcess 会先在本程序所在目录里找：用户把 exe 放在
/// "下载"文件夹运行时，同目录一个同名恶意程序就会被执行。
pub fn system_exe(rel: &str) -> std::path::PathBuf {
    let root = std::env::var_os("SystemRoot")
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "C:\\Windows".into());
    std::path::Path::new(&root).join(rel)
}

/// ShellExecuteW 封装。verb: "open" / "runas"。成功返回 true。
pub fn shell_execute(verb: &str, target: &str, params: &str) -> bool {
    unsafe {
        let mut v = wide(verb);
        let mut t = wide(target);
        let mut p = wide(params);
        let r = ShellExecuteW(
            None,
            windows::core::PCWSTR(v.as_mut_ptr()),
            windows::core::PCWSTR(t.as_mut_ptr()),
            windows::core::PCWSTR(p.as_mut_ptr()),
            None,
            SW_SHOWNORMAL,
        );
        r.0 as isize > 32
    }
}

/// 用浏览器打开 URL（browser=None 系统默认）。
pub fn open_url(url: &str, browser: Option<&str>) {
    match browser {
        Some(path) if !path.is_empty() => {
            if !shell_execute("open", path, url) {
                crate::log!("浏览器启动失败: {path}");
            }
        }
        _ => {
            if !shell_execute("open", url, "") {
                crate::log!("打开 URL 失败: {url}");
            }
        }
    }
}

/// 枚举回调上下文。
struct FindCtx {
    class: Option<regex::Regex>,
    title: Option<regex::Regex>,
    found: Option<HWND>,
}

/// 枚举可见顶层窗口，按类名/标题（可正则）查找第一个匹配。
pub fn find_window_by(win_class: Option<&str>, win_title: Option<&str>, use_regex: bool) -> Option<HWND> {
    let to_matcher = |pat: &str| -> Option<regex::Regex> {
        if use_regex {
            regex::Regex::new(pat).ok()
        } else {
            regex::Regex::new(&regex::escape(pat)).ok()
        }
    };
    let mut ctx = FindCtx {
        class: win_class.and_then(to_matcher),
        title: win_title.and_then(to_matcher),
        found: None,
    };
    unsafe {
        let ctx_ptr: *mut FindCtx = &mut ctx;
        let _ = EnumWindows(
            Some(enum_proc),
            LPARAM(ctx_ptr as *mut core::ffi::c_void as isize),
        );
    }
    ctx.found
}

unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let ctx = &mut *(lparam.0 as *mut FindCtx);
    if !IsWindowVisible(hwnd).as_bool() {
        return BOOL(1);
    }
    let mut buf = [0u16; 256];
    if let Some(re) = &ctx.class {
        let n = windows::Win32::UI::WindowsAndMessaging::GetClassNameW(hwnd, &mut buf);
        let name = String::from_utf16_lossy(&buf[..n as usize]);
        if !re.is_match(&name) {
            return BOOL(1);
        }
    }
    if let Some(re) = &ctx.title {
        let n = GetWindowTextW(hwnd, &mut buf);
        let title = String::from_utf16_lossy(&buf[..n as usize]);
        if !re.is_match(&title) {
            return BOOL(1);
        }
    }
    if ctx.class.is_some() || ctx.title.is_some() {
        ctx.found = Some(hwnd);
        return BOOL(0); // 停止枚举
    }
    BOOL(1)
}

/// 运行/激活应用程序。
pub fn launch_app(
    exe: &str,
    args: Option<&str>,
    elevate: bool,
    win_class: Option<&str>,
    win_title: Option<&str>,
    use_regex: bool,
    if_running: IfRunning,
) {
    // 已在运行 → 按策略处理
    if let Some(hwnd) = find_window_by(win_class, win_title, use_regex) {
        match if_running {
            IfRunning::Activate => {
                focus_window(hwnd);
                return;
            }
            IfRunning::RunAgain => { /* 继续往下再启动一个 */ }
            IfRunning::SendKeys => {
                focus_window(hwnd); // keys 字段 M3 设置界面补充
                return;
            }
        }
    }
    let ok = if elevate {
        shell_execute("runas", exe, args.unwrap_or(""))
    } else {
        run_app_process(exe, args.unwrap_or(""))
    };
    if !ok {
        crate::log!("应用启动失败: {exe}");
    }
}

fn run_app_process(exe: &str, args: &str) -> bool {
    let cmdline = if args.is_empty() {
        format!("\"{exe}\"")
    } else {
        format!("\"{exe}\" {args}")
    };
    let mut wide_cmd: Vec<u16> = cmdline.encode_utf16().chain([0]).collect();
    let si = STARTUPINFOW {
        cb: std::mem::size_of::<STARTUPINFOW>() as u32,
        ..Default::default()
    };
    let mut pi = PROCESS_INFORMATION::default();
    let ok = unsafe {
        CreateProcessW(
            windows::core::PCWSTR::null(),
            Some(windows::core::PWSTR(wide_cmd.as_mut_ptr())),
            None,
            None,
            false,
            windows::Win32::System::Threading::CREATE_NO_WINDOW,
            None,
            None,
            &si,
            &mut pi,
        )
    };
    if ok.is_ok() {
        unsafe {
            let _ = CloseHandle(pi.hProcess);
            let _ = CloseHandle(pi.hThread);
        }
        true
    } else {
        false
    }
}

/// 注入系统功能键（媒体键）。
pub fn inject_system_key(key: &str) {
    let upper = key.to_ascii_uppercase();
    let vk: u16 = match upper.as_str() {
        "PREVIOUS" => 0xB1,
        "PLAY" | "PLAY_PAUSE" | "MEDIA_PLAY" => 0xB3,
        "NEXT" => 0xB0,
        "MUTE" => 0xAD,
        "SOUND_DOWN" | "VOLUME_DOWN" => 0xAE,
        "SOUND_UP" | "VOLUME_UP" => 0xAF,
        "BRIGHTNESS_UP" | "BRIGHTNESS_DOWN" => {
            let (x, y) = get_cursor_pos();
            let delta = if upper == "BRIGHTNESS_UP" { 10 } else { -10 };
            use mousetroke_core::i18n::{keys, t, tf};
            match crate::brightness::adjust(x, y, delta) {
                Some(v) => crate::engine::push_toast(tf(keys::OVERLAY_BRIGHTNESS, &[("value", &v.to_string())]), 900),
                None => crate::engine::push_toast(t(keys::OVERLAY_BRIGHTNESS_UNSUPPORTED).into(), 1500),
            }
            return;
        }
        _ => {
            crate::log!("未知系统功能键: {key}");
            return;
        }
    };
    send_vk(vk, true);
    send_vk(vk, false);
}

/// 在 (x, y) 弹出自定义菜单（**必须在专用工作线程调用**——模态循环），
/// 返回选中的菜单项。TPM_RETURNCMD 模式，支持递归子菜单。
pub fn show_context_menu(items: &[MenuItem], x: i32, y: i32) -> Option<MenuItem> {
    unsafe {
        // 菜单需要前台宿主窗口
        let hinst = windows::Win32::System::LibraryLoader::GetModuleHandleW(None).ok()?;
        let mut class: Vec<u16> = "MousetrokeMenuHost".encode_utf16().chain([0]).collect();
        let wc = windows::Win32::UI::WindowsAndMessaging::WNDCLASSW {
            lpfnWndProc: Some(menu_wndproc),
            hInstance: hinst.into(),
            lpszClassName: windows::core::PCWSTR(class.as_mut_ptr()),
            ..Default::default()
        };
        let _ = windows::Win32::UI::WindowsAndMessaging::RegisterClassW(&wc);
        let hwnd = windows::Win32::UI::WindowsAndMessaging::CreateWindowExW(
            WS_EX_TOOLWINDOW,
            windows::core::PCWSTR(class.as_mut_ptr()),
            None,
            WS_OVERLAPPED,
            0, 0, 0, 0,
            None, None,
            Some(hinst.into()),
            None,
        )
        .ok()?;

        let mut collected: Vec<MenuItem> = Vec::new();
        let menu = windows::Win32::UI::WindowsAndMessaging::CreatePopupMenu().ok()?;
        build_menu(menu, items, &mut collected);

        let _ = SetForegroundWindow(hwnd);
        let cmd = TrackPopupMenuEx(
            menu,
            TPM_RETURNCMD.0 | TPM_RIGHTBUTTON.0,
            x,
            y,
            hwnd,
            None,
        );
        let _ = PostMessageW(Some(hwnd), WM_NULL, WPARAM(0), LPARAM(0));
        let _ = DestroyMenu(menu);
        let _ = DestroyWindow(hwnd);

        if cmd.0 > 0 && (cmd.0 as usize) <= collected.len() {
            Some(collected[(cmd.0 - 1) as usize].clone())
        } else {
            None
        }
    }
}

unsafe extern "system" fn menu_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    windows::Win32::UI::WindowsAndMessaging::DefWindowProcW(hwnd, msg, wparam, lparam)
}

unsafe fn build_menu(menu: HMENU_W, items: &[MenuItem], collected: &mut Vec<MenuItem>) {
    for item in items {
        if let mousetroke_core::schema::Action::Menu { items: sub, .. } = &item.action {
            let sub_menu = windows::Win32::UI::WindowsAndMessaging::CreatePopupMenu().unwrap_or_default();
            build_menu(sub_menu, sub, collected);
            let mut label = wide(&item.label);
            let _ = AppendMenuW(
                menu,
                MF_POPUP,
                sub_menu.0 as usize,
                windows::core::PCWSTR(label.as_mut_ptr()),
            );
        } else {
            collected.push(item.clone());
            let id = collected.len();
            let mut label = wide(&item.label);
            let _ = AppendMenuW(menu, MF_STRING, id, windows::core::PCWSTR(label.as_mut_ptr()));
        }
    }
}

/// 编译期句柄类型引用（HMENU 别名检查）。
#[allow(dead_code)]
fn _menu_handle(h: HMENU_W) -> HMENU_W {
    h
}


// ---------------------------------------------------------------- Lua API 支撑

use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::System::Memory::{GlobalAlloc, GMEM_MOVEABLE};
use windows::Win32::System::DataExchange::{EmptyClipboard, SetClipboardData};
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, SetCursorPos, SW_MAXIMIZE as SW_MAX, SW_MINIMIZE as SW_MIN, SW_RESTORE as SW_RST,
    SW_SHOWMAXIMIZED,
};

/// 写入剪贴板文本。
pub fn set_clipboard_text(text: &str) {
    unsafe {
        if OpenClipboard(None).is_err() {
            return;
        }
        let _ = EmptyClipboard();
        let wide: Vec<u16> = text.encode_utf16().chain([0]).collect();
        let bytes = wide.len() * 2;
        if let Ok(hglobal) = GlobalAlloc(GMEM_MOVEABLE, bytes) {
            let dst = GlobalLock(hglobal) as *mut u16;
            if !dst.is_null() {
                std::ptr::copy_nonoverlapping(wide.as_ptr(), dst, wide.len());
                let _ = GlobalUnlock(hglobal);
                let _ = SetClipboardData(13 /* CF_UNICODETEXT */, Some(HANDLE(hglobal.0)));
            }
        }
        let _ = CloseClipboard();
    }
}

/// 按类名/标题（可正则）枚举全部匹配的可见顶层窗口（Lua find_win_* 用）。
/// 枚举收集上下文（全部匹配）。
struct ListCtx {
    class: Option<regex::Regex>,
    title: Option<regex::Regex>,
    out: Vec<(String, isize)>,
}

pub fn find_windows_by(
    win_class: Option<&str>,
    win_title: Option<&str>,
    use_regex: bool,
) -> Vec<(String, isize)> {
    let to_matcher = |pat: &str| -> Option<regex::Regex> {
        if use_regex {
            regex::Regex::new(pat).ok()
        } else {
            regex::Regex::new(&regex::escape(pat)).ok()
        }
    };
    let mut ctx = ListCtx {
        class: win_class.and_then(to_matcher),
        title: win_title.and_then(to_matcher),
        out: Vec::new(),
    };
    unsafe {
        let ctx_ptr: *mut ListCtx = &mut ctx;
        let _ = EnumWindows(
            Some(list_enum_proc),
            LPARAM(ctx_ptr as *mut core::ffi::c_void as isize),
        );
    }
    ctx.out
}

unsafe extern "system" fn list_enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let ctx = &mut *(lparam.0 as *mut ListCtx);
    if !IsWindowVisible(hwnd).as_bool() || ctx.out.len() >= 32 {
        return BOOL(1);
    }
    let mut buf = [0u16; 256];
    if let Some(re) = &ctx.class {
        let n = windows::Win32::UI::WindowsAndMessaging::GetClassNameW(hwnd, &mut buf);
        if !re.is_match(&String::from_utf16_lossy(&buf[..n as usize])) {
            return BOOL(1);
        }
    }
    let n = GetWindowTextW(hwnd, &mut buf);
    let title = String::from_utf16_lossy(&buf[..n as usize]);
    if let Some(re) = &ctx.title {
        if !re.is_match(&title) {
            return BOOL(1);
        }
    }
    if ctx.class.is_some() || ctx.title.is_some() {
        ctx.out.push((title, hwnd.0 as isize));
    }
    BOOL(1)
}

/// 设置窗口状态：min / max / norm。
pub fn set_window_state(hwnd: HWND, state: &str) {
    unsafe {
        match state {
            "min" => {
                let _ = ShowWindow(hwnd, SW_MIN);
            }
            "max" => {
                let _ = ShowWindow(hwnd, SW_SHOWMAXIMIZED);
            }
            _ => {
                let _ = ShowWindow(hwnd, SW_RST);
            }
        }
    }
    let _ = (SW_MAX, SW_MIN, SW_RST);
}

/// 当前光标位置。
pub fn get_cursor_pos() -> (i32, i32) {
    let mut pt = windows::Win32::Foundation::POINT::default();
    unsafe {
        let _ = GetCursorPos(&mut pt);
    }
    (pt.x, pt.y)
}

/// 移动光标。
pub fn set_cursor_pos(x: i32, y: i32) {
    unsafe {
        let _ = SetCursorPos(x, y);
    }
}

/// 点所在屏幕矩形。
pub fn screen_of(x: i32, y: i32) -> (i32, i32, i32, i32) {
    unsafe {
        let mon = MonitorFromPoint(
            windows::Win32::Foundation::POINT { x, y },
            MONITOR_DEFAULTTONEAREST,
        );
        let mut mi = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if GetMonitorInfoW(mon, &mut mi).as_bool() {
            let r = mi.rcMonitor;
            (r.left, r.top, r.right, r.bottom)
        } else {
            (0, 0, 0, 0)
        }
    }
}

/// 按键是否处于按下状态。
pub fn is_vk_pressed(key: mousetroke_core::keyseq::Key) -> bool {
    let vk: i32 = match key {
        mousetroke_core::keyseq::Key::Vk(v) => v as i32,
        _ => return false,
    };
    (unsafe { GetAsyncKeyState(vk) } as u16 & 0x8000) != 0
}
