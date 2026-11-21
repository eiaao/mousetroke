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
    DestroyMenu, GetWindowTextW, IsIconic, PostMessageW, SetForegroundWindow, SetWindowPos, ShowWindow,
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

/// 虚拟键对应的扫描码与是否扩展键；`map` 为 VK→扫描码换算（`MapVirtualKeyW`）。
///
/// 浏览器按扫描码生成 `KeyboardEvent.code`，扫描码为 0 时 `code` 为空，依赖它的网页快捷键不响应（#84）。
/// 只给有固定物理位置的键补扫描码；媒体、浏览器、输入法等键换算结果可能撞上普通键（如音量+ 得到 B 的位置），保持 0。
/// 导航键与小键盘数字键共用扫描码，必须带扩展标志，否则会被当成小键盘键，NumLock 开启时 Shift+Home 还会被系统插入松开 Shift。
fn scan_code(vk: u16, map: impl FnOnce(u16) -> u32) -> (u16, bool) {
    let extended = matches!(
        vk,
        0x21..=0x28 // PageUp PageDown End Home ←↑→↓
            | 0x2C..=0x2E // PrintScreen Insert Delete
            | 0x5B..=0x5D // 左右 Win、菜单键
            | 0x6F // 小键盘 /
            | 0x90 // NumLock
            | 0xA3 | 0xA5 // 右 Ctrl、右 Alt
    );
    let physical = matches!(
        vk,
        0x08 | 0x09 | 0x0D // Backspace Tab Enter
            | 0x10..=0x12 | 0x14 | 0x1B | 0x20 // Shift Ctrl Alt CapsLock Esc 空格
            | 0x30..=0x39 | 0x41..=0x5A // 数字、字母
            | 0x60..=0x6B | 0x6D | 0x6E // 小键盘（不含分隔键）
            | 0x70..=0x87 // F1–F24
            | 0x91 | 0xA0..=0xA5 // ScrollLock、左右修饰键
            | 0xBA..=0xC0 | 0xDB..=0xDF | 0xE2 // 标点键
    );
    match vk {
        0x2C => (0x37, true), // PrintScreen 物理扫描码 E0 37；换算结果是 Alt+PrintScreen 的 SysRq
        _ if extended || physical => match map(vk) {
            // 只取低字节；换算失败时不标扩展，与原来一致
            0 => (0, false),
            sc => ((sc & 0xFF) as u16, extended),
        },
        _ => (0, false),
    }
}

fn key_input(vk: u16, down: bool) -> INPUT {
    use windows::Win32::UI::Input::KeyboardAndMouse::{MapVirtualKeyW, KEYEVENTF_EXTENDEDKEY, MAPVK_VK_TO_VSC};
    let (scan, extended) = scan_code(vk, |vk| unsafe { MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC) });
    let mut flags = if down { KEYBD_EVENT_FLAGS(0) } else { KEYEVENTF_KEYUP };
    if extended {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: windows::Win32::UI::Input::KeyboardAndMouse::KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: INJECT_TAG,
            },
        },
    }
}

fn send_vk(vk: u16, down: bool) {
    unsafe { SendInput(&[key_input(vk, down)], std::mem::size_of::<INPUT>() as i32) };
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
    let _ = send_text_checked(text);
}

pub(crate) fn send_text_checked(text: &str) -> Result<(), String> {
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
    if !inputs.is_empty() && unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) } as usize != inputs.len() {
        return Err("Unicode input failed; check target permissions".into());
    }
    Ok(())
}

/// 原生文字动作用完整的一组按下/抬起事件，并把注入失败交给调用方报告。
pub(crate) fn send_shortcut_checked(vk: u16, modifier: ModKey) -> Result<(), String> {
    let modifier = mod_vk(modifier);
    let inputs = [(modifier, true), (vk, true), (vk, false), (modifier, false)].map(|(key, down)| key_input(key, down));
    if unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) } == inputs.len() as u32 {
        Ok(())
    } else {
        send_vk(vk, false);
        send_vk(modifier, false);
        Err("Shortcut input failed; check target permissions".into())
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
    let ok = spawn(windows::core::PCWSTR(app.as_ptr()), &mut wide, si);
    if !ok {
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
        if IsIconic(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }
        let _ = SetForegroundWindow(hwnd);
    }
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
/// 只认这次复制写入的内容：剪贴板序号在时限内没有变化（没有选中文字、复制失败或太慢）就返回 None，
/// 绝不把剪贴板里原有的内容当成选中文字（否则可能把它发给搜索或翻译网站）。
pub fn capture_selected_text() -> Option<String> {
    use std::time::{Duration, Instant};
    use windows::Win32::System::DataExchange::GetClipboardSequenceNumber;
    let seq = mousetroke_core::keyseq::parse("^{C}", false).ok()?;
    // 先读一次：上一位所有者"延迟提供"的文字会在此刻补写（补写也会让序号前进），
    // 免得它在等待期间补写、被误当成这次复制的结果
    let _ = get_clipboard_text();
    let before = unsafe { GetClipboardSequenceNumber() };
    inject_keyseq(&seq);
    let deadline = Instant::now() + Duration::from_millis(500);
    loop {
        std::thread::sleep(Duration::from_millis(20));
        // 序号变了也可能刚清空、还没写入文字：读到为止或到时限
        if unsafe { GetClipboardSequenceNumber() } != before {
            if let Some(text) = get_clipboard_text().filter(|s| !s.trim().is_empty()) {
                return Some(text);
            }
        }
        if Instant::now() >= deadline {
            return None;
        }
    }
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
    // 管理员身份下 ShellExecute 打开的程序也会是管理员：改由外壳的子进程打开
    if verb != "runas" && crate::targeting::self_elevated() {
        return if params.is_empty() {
            run_app_process(&system_exe("explorer.exe").to_string_lossy(), &format!("\"{target}\""))
        } else {
            run_app_process(target, params)
        };
    }
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

/// 展开用户与预置路径中的环境变量；未知变量按 Windows 语义原样保留。
fn expand_environment(s: &str) -> String {
    use windows::{core::PCWSTR, Win32::System::Environment::ExpandEnvironmentStringsW};
    let source: Vec<u16> = s.encode_utf16().chain([0]).collect();
    let mut required = unsafe { ExpandEnvironmentStringsW(PCWSTR(source.as_ptr()), None) };
    loop {
        if required == 0 {
            return s.into();
        }
        let mut buffer = vec![0u16; required as usize];
        let written = unsafe { ExpandEnvironmentStringsW(PCWSTR(source.as_ptr()), Some(&mut buffer)) };
        if written == 0 {
            return s.into();
        }
        if written as usize <= buffer.len() {
            return String::from_utf16_lossy(&buffer[..written as usize - 1]);
        }
        // 两次调用间环境变化时按最新长度重试，不受 MAX_PATH 限制。
        required = written;
    }
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
    let running = find_window_by(win_class, win_title, use_regex);
    if let Some(hwnd) = running {
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
    let exe = expand_environment(exe);
    let ok = if elevate {
        shell_execute("runas", &exe, args.unwrap_or(""))
    } else {
        run_app_process(&exe, args.unwrap_or(""))
    };
    if !ok {
        crate::log!("应用启动失败: {exe}");
    }
}

/// 仅由预置打开应用的无参数路径调用。
pub(crate) fn activate_app_by_identity(exe: &str) -> bool {
    let exe = expand_environment(exe);
    let running = crate::targeting::visible_windows().into_iter().find_map(|(hwnd, _)| {
        if !app_activation_class_eligible(&crate::targeting::class_name(hwnd)) { return None; }
        let mut pid = 0;
        unsafe { windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId(hwnd, Some(&mut pid)); }
        crate::targeting::exe_of(pid).filter(|path| app_exe_matches(&exe, path)).map(|_| hwnd)
    });
    if let Some(hwnd) = running { focus_window(hwnd); true } else { false }
}

/// 桌面与系统外壳不能代替应用窗口；资源管理器的文件窗口仍可激活。
fn app_activation_class_eligible(class: &str) -> bool {
    !matches!(class, "Progman" | "WorkerW" | "SHELLDLL_DefView") && !crate::targeting::is_shell_ui_class(class)
}

fn app_exe_matches(requested: &str, running: &str) -> bool {
    if requested.contains(['/', '\\']) {
        requested.replace('/', "\\").eq_ignore_ascii_case(&running.replace('/', "\\"))
    } else {
        requested.eq_ignore_ascii_case(mousetroke_core::matcher::file_name(running))
    }
}

fn run_app_process(exe: &str, args: &str) -> bool {
    let cmdline = if args.is_empty() {
        format!("\"{exe}\"")
    } else {
        format!("\"{exe}\" {args}")
    };
    let mut wide_cmd: Vec<u16> = cmdline.encode_utf16().chain([0]).collect();
    let si = STARTUPINFOW { cb: std::mem::size_of::<STARTUPINFOW>() as u32, ..Default::default() };
    spawn(windows::core::PCWSTR::null(), &mut wide_cmd, si)
}

/// 创建进程（不等待）。本程序以管理员身份运行时，把桌面外壳（Explorer）设为父进程，
/// 新进程继承外壳的普通权限：只有勾选了“以管理员身份运行”的动作才提权。
fn spawn(app: windows::core::PCWSTR, cmdline: &mut [u16], si: STARTUPINFOW) -> bool {
    use windows::Win32::System::Threading::{
        DeleteProcThreadAttributeList, InitializeProcThreadAttributeList, OpenProcess, UpdateProcThreadAttribute,
        CREATE_NO_WINDOW, EXTENDED_STARTUPINFO_PRESENT, LPPROC_THREAD_ATTRIBUTE_LIST, PROCESS_CREATE_PROCESS,
        PROC_THREAD_ATTRIBUTE_PARENT_PROCESS, STARTUPINFOEXW,
    };
    let mut pi = PROCESS_INFORMATION::default();
    let shell = crate::targeting::self_elevated().then(shell_process).flatten();
    let ok = unsafe {
        match shell {
            None => CreateProcessW(app, Some(windows::core::PWSTR(cmdline.as_mut_ptr())), None, None, false, CREATE_NO_WINDOW, None, None, &si, &mut pi),
            Some(pid) => {
                let Ok(parent) = OpenProcess(PROCESS_CREATE_PROCESS, false, pid) else { return false };
                let mut size = 0;
                let _ = InitializeProcThreadAttributeList(None, 1, None, &mut size);
                let mut buf = vec![0u8; size];
                let list = LPPROC_THREAD_ATTRIBUTE_LIST(buf.as_mut_ptr().cast());
                let result = InitializeProcThreadAttributeList(Some(list), 1, None, &mut size)
                    .and_then(|()| {
                        let r = UpdateProcThreadAttribute(
                            list,
                            0,
                            PROC_THREAD_ATTRIBUTE_PARENT_PROCESS as usize,
                            Some((&parent as *const HANDLE).cast()),
                            std::mem::size_of::<HANDLE>(),
                            None,
                            None,
                        );
                        let six = STARTUPINFOEXW { StartupInfo: STARTUPINFOW { cb: std::mem::size_of::<STARTUPINFOEXW>() as u32, ..si }, lpAttributeList: list };
                        r.and_then(|()| {
                            CreateProcessW(app, Some(windows::core::PWSTR(cmdline.as_mut_ptr())), None, None, false,
                                CREATE_NO_WINDOW | EXTENDED_STARTUPINFO_PRESENT, None, None, &six.StartupInfo, &mut pi)
                        })
                    });
                DeleteProcThreadAttributeList(list);
                let _ = CloseHandle(parent);
                result
            }
        }
    };
    if ok.is_ok() {
        unsafe {
            let _ = CloseHandle(pi.hProcess);
            let _ = CloseHandle(pi.hThread);
        }
    }
    ok.is_ok()
}

/// 桌面外壳（Explorer）的进程号。
fn shell_process() -> Option<u32> {
    use windows::Win32::UI::WindowsAndMessaging::{GetShellWindow, GetWindowThreadProcessId};
    let mut pid = 0;
    unsafe { GetWindowThreadProcessId(GetShellWindow(), Some(&mut pid)) };
    (pid != 0).then_some(pid)
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



// ---------------------------------------------------------------- Lua API 支撑

use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, MonitorFromPoint, HDC, HMONITOR, MONITORINFO, MONITOR_DEFAULTTONEAREST,
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

/// 全部显示器的完整矩形 (x, y, w, h)，虚拟屏坐标。
pub fn monitor_rects() -> Vec<(i32, i32, i32, i32)> {
    unsafe extern "system" fn collect(_: HMONITOR, _: HDC, rect: *mut RECT, data: LPARAM) -> BOOL {
        // 安全：EnumDisplayMonitors 同步调用；rect 为该显示器矩形，data 指向下面的 Vec。
        let (rects, r) = unsafe { (&mut *(data.0 as *mut Vec<(i32, i32, i32, i32)>), *rect) };
        rects.push((r.left, r.top, r.right - r.left, r.bottom - r.top));
        BOOL(1)
    }
    let mut rects = Vec::new();
    let _ = unsafe { EnumDisplayMonitors(None, None, Some(collect), LPARAM(&mut rects as *mut _ as isize)) };
    rects
}

/// 按键是否处于按下状态。
pub fn is_vk_pressed(key: mousetroke_core::keyseq::Key) -> bool {
    let vk: i32 = match key {
        mousetroke_core::keyseq::Key::Vk(v) => v as i32,
        _ => return false,
    };
    (unsafe { GetAsyncKeyState(vk) } as u16 & 0x8000) != 0
}

/// 对序列里的每个键（含修饰键）取反：按下则抬起，未按下则按下（Lua ToggleKeys 用）。
pub fn toggle_keyseq(seq: &KeySeq) {
    for action in &seq.actions {
        if let Action::Key { key: Key::Vk(vk), mods, .. } = action {
            for m in mods {
                send_vk(mod_vk(*m), !is_vk_pressed(Key::Vk(mod_vk(*m))));
            }
            send_vk(*vk, !is_vk_pressed(Key::Vk(*vk)));
        }
    }
}

/// 构造 CF_HTML 数据（含按字节偏移的头部）。
pub fn build_cf_html(fragment: &str) -> Vec<u8> {
    const PREFIX: &str = "<html><body>\r\n<!--StartFragment-->";
    const SUFFIX: &str = "<!--EndFragment-->\r\n</body></html>";
    let header_len = "Version:0.9\r\nStartHTML:0000000000\r\nEndHTML:0000000000\r\nStartFragment:0000000000\r\nEndFragment:0000000000\r\n".len();
    let start_fragment = header_len + PREFIX.len();
    let end_fragment = start_fragment + fragment.len();
    let end_html = end_fragment + SUFFIX.len();
    format!(
        "Version:0.9\r\nStartHTML:{header_len:010}\r\nEndHTML:{end_html:010}\r\nStartFragment:{start_fragment:010}\r\nEndFragment:{end_fragment:010}\r\n{PREFIX}{fragment}{SUFFIX}"
    )
    .into_bytes()
}

/// 从 CF_HTML 数据取出 StartFragment/EndFragment 之间的片段。
pub fn parse_cf_html(data: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(data);
    let field = |name: &str| -> Option<usize> {
        let rest = &text[text.find(name)? + name.len()..];
        rest[..rest.find(['\r', '\n'])?].trim().parse().ok()
    };
    let (start, end) = (field("StartFragment:")?, field("EndFragment:")?);
    std::str::from_utf8(data.get(start..end)?).ok().map(str::to_owned)
}

/// 构造 CF_HDROP 数据：DROPFILES 头（宽字符）+ 以双 NUL 结尾的路径列表。
pub fn build_dropfiles(paths: &[String]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&20u32.to_le_bytes()); // pFiles
    out.extend_from_slice(&[0; 8]); // pt
    out.extend_from_slice(&0u32.to_le_bytes()); // fNC
    out.extend_from_slice(&1u32.to_le_bytes()); // fWide
    for p in paths {
        p.encode_utf16().chain([0]).for_each(|u| out.extend_from_slice(&u.to_le_bytes()));
    }
    out.extend_from_slice(&0u16.to_le_bytes());
    out
}

/// 解析 CF_HDROP 数据；仅支持宽字符。
pub fn parse_dropfiles(data: &[u8]) -> Option<Vec<String>> {
    let offset = u32::from_le_bytes(data.get(0..4)?.try_into().ok()?) as usize;
    if u32::from_le_bytes(data.get(16..20)?.try_into().ok()?) == 0 {
        return None;
    }
    let units: Vec<u16> = data.get(offset..)?.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
    Some(units.split(|u| *u == 0).take_while(|s| !s.is_empty()).map(String::from_utf16_lossy).collect())
}

fn clipboard_format(name: &str) -> u32 {
    let wide: Vec<u16> = name.encode_utf16().chain([0]).collect();
    unsafe { windows::Win32::System::DataExchange::RegisterClipboardFormatW(windows::core::PCWSTR(wide.as_ptr())) }
}

fn get_clipboard_bytes(format: u32) -> Option<Vec<u8>> {
    unsafe {
        OpenClipboard(None).ok()?;
        let read = || -> Option<Vec<u8>> {
            let handle = HGLOBAL(GetClipboardData(format).ok()?.0);
            let ptr = GlobalLock(handle) as *const u8;
            if ptr.is_null() {
                return None;
            }
            let data = std::slice::from_raw_parts(ptr, windows::Win32::System::Memory::GlobalSize(handle)).to_vec();
            let _ = GlobalUnlock(handle);
            Some(data)
        };
        let result = read();
        let _ = CloseClipboard();
        result
    }
}

fn set_clipboard_bytes(format: u32, data: &[u8]) -> bool {
    unsafe {
        if OpenClipboard(None).is_err() {
            return false;
        }
        let _ = EmptyClipboard();
        let mut ok = false;
        if let Ok(hglobal) = GlobalAlloc(GMEM_MOVEABLE, data.len()) {
            let dst = GlobalLock(hglobal) as *mut u8;
            if !dst.is_null() {
                std::ptr::copy_nonoverlapping(data.as_ptr(), dst, data.len());
                let _ = GlobalUnlock(hglobal);
                ok = SetClipboardData(format, Some(HANDLE(hglobal.0))).is_ok();
            }
        }
        let _ = CloseClipboard();
        ok
    }
}

/// 读取 CF_HTML 片段。
pub fn get_clipboard_html() -> Option<String> {
    parse_cf_html(&get_clipboard_bytes(clipboard_format("HTML Format"))?)
}

/// 写入 CF_HTML 片段。
pub fn set_clipboard_html(fragment: &str) -> bool {
    let mut data = build_cf_html(fragment);
    data.push(0);
    set_clipboard_bytes(clipboard_format("HTML Format"), &data)
}

/// 读取剪贴板里的文件列表（CF_HDROP）。
pub fn get_clipboard_files() -> Option<Vec<String>> {
    parse_dropfiles(&get_clipboard_bytes(15 /* CF_HDROP */)?)
}

/// 把文件列表写入剪贴板（CF_HDROP）。
pub fn set_clipboard_files(paths: &[String]) -> bool {
    set_clipboard_bytes(15 /* CF_HDROP */, &build_dropfiles(paths))
}

/// 文件资源管理器窗口当前显示的文件夹路径；非资源管理器或虚拟文件夹返回 None。
pub fn explorer_path(hwnd: HWND) -> Option<String> {
    use windows::core::Interface;
    use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoUninitialize, IServiceProvider, CLSCTX_ALL, COINIT_APARTMENTTHREADED};
    use windows::Win32::System::Variant::VARIANT;
    use windows::Win32::UI::Shell::{
        IFolderView, IPersistFolder2, IShellBrowser, IShellWindows, IWebBrowserApp, SHGetPathFromIDListW, ShellWindows,
    };
    unsafe {
        // 工作线程默认没有 COM；S_FALSE 也要配对 Uninitialize，线程模式冲突时则不能
        let init = CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_ok();
        let find = || -> Option<String> {
            let windows: IShellWindows = CoCreateInstance(&ShellWindows, None, CLSCTX_ALL).ok()?;
            for i in 0..windows.Count().ok()? {
                let Ok(item) = windows.Item(&VARIANT::from(i)) else { continue };
                let Ok(app) = item.cast::<IWebBrowserApp>() else { continue };
                if app.HWND().map(|h| h.0).ok() != Some(hwnd.0 as isize) {
                    continue;
                }
                let Ok(provider) = app.cast::<IServiceProvider>() else { continue };
                let Ok(browser) = provider.QueryService::<IShellBrowser>(&windows::Win32::UI::Shell::SID_STopLevelBrowser) else { continue };
                let Ok(view) = browser.QueryActiveShellView() else { continue };
                // 多标签页共用顶层 HWND，只取当前可见的那一页
                if !view.GetWindow().is_ok_and(|w| windows::Win32::UI::WindowsAndMessaging::IsWindowVisible(w).as_bool()) {
                    continue;
                }
                let folder = view.cast::<IFolderView>().ok()?.GetFolder::<IPersistFolder2>().ok()?;
                let pidl = folder.GetCurFolder().ok()?;
                let mut buf = [0u16; 260];
                let ok = SHGetPathFromIDListW(pidl, &mut buf).as_bool();
                windows::Win32::System::Com::CoTaskMemFree(Some(pidl as *const _));
                let len = buf.iter().position(|c| *c == 0).unwrap_or(buf.len());
                return ok.then(|| String::from_utf16_lossy(&buf[..len]));
            }
            None
        };
        let result = find();
        if init {
            CoUninitialize();
        }
        result
    }
}

#[cfg(test)]
mod clipboard_format_tests {
    use super::*;

    #[test]
    fn cf_html_offsets_point_at_the_fragment() {
        let data = build_cf_html("<b>héllo</b>");
        let text = String::from_utf8(data.clone()).unwrap();
        let num = |name: &str| -> usize {
            let rest = &text[text.find(name).unwrap() + name.len()..];
            rest[..10].parse().unwrap()
        };
        assert_eq!(&data[num("StartFragment:")..num("EndFragment:")], "<b>héllo</b>".as_bytes());
        assert_eq!(num("EndHTML:"), data.len());
        assert!(text[num("StartHTML:")..].starts_with("<html>"));
        assert_eq!(parse_cf_html(&data).as_deref(), Some("<b>héllo</b>"));
    }

    #[test]
    #[ignore = "需要已打开的资源管理器窗口"]
    fn explorer_path_of_open_windows() {
        for (title, h) in find_windows_by(Some("CabinetWClass"), None, false) {
            eprintln!("{title}: {:?}", explorer_path(HWND(h as *mut core::ffi::c_void)));
        }
    }

    #[test]
    fn dropfiles_layout_is_wide_and_double_nul_terminated() {
        let data = build_dropfiles(&["C:\\a".into(), "D:\\b".into()]);
        assert_eq!(&data[..4], &20u32.to_le_bytes());
        assert_eq!(&data[16..20], &1u32.to_le_bytes());
        assert_eq!(data.len(), 20 + (5 + 5 + 1) * 2);
        assert_eq!(&data[data.len() - 4..], &[0, 0, 0, 0]);
        assert_eq!(parse_dropfiles(&data), Some(vec!["C:\\a".to_string(), "D:\\b".to_string()]));
    }
}

#[cfg(test)]
mod app_identity_tests {
    use super::{app_activation_class_eligible, app_exe_matches, expand_environment};

    #[test]
    fn environment_expansion_handles_system_paths_mixed_text_and_unknown_variables() {
        let root = std::env::var("SystemRoot").unwrap();
        assert_eq!(expand_environment("%SystemRoot%"), root);
        let mixed = format!("prefix {} %SystemRoot% suffix", "x".repeat(300));
        assert_eq!(expand_environment(&mixed), mixed.replace("%SystemRoot%", &root));
        assert_eq!(expand_environment("%MOUSESTROKE_UNKNOWN_ENV_829A%"), "%MOUSESTROKE_UNKNOWN_ENV_829A%");
        assert_eq!(expand_environment("plain text"), "plain text");
        assert_eq!(expand_environment(""), "");
        let expanded = expand_environment(r"%SystemRoot%\explorer.exe");
        assert!(app_exe_matches(&expanded, &format!(r"{root}\explorer.exe")));
        assert!(!app_exe_matches(&expanded, "portable/explorer.exe"));
    }

    #[test]
    fn activation_matches_names_and_full_paths_without_confusing_apps() {
        assert!(app_exe_matches("EDITOR.EXE", "tools/editor.exe"));
        assert!(app_exe_matches("tools/editor.exe", "tools\\EDITOR.EXE"));
        assert!(!app_exe_matches("editor.exe", "tools/other-editor.exe"));
        assert!(!app_exe_matches("tools/editor.exe", "other/editor.exe"));
    }
    #[test]
    fn shell_classes_are_not_app_activation_candidates() {
        for class in ["Progman", "WorkerW", "SHELLDLL_DefView", "Shell_TrayWnd", "Shell_SecondaryTrayWnd", "NotifyIconOverflowWindow", "TopLevelWindowForOverflowXamlIsland", "Xaml_WindowedPopupClass", "XamlExplorerHostIslandWindow", "Windows.UI.Core.CoreWindow", "#32768"] {
            assert!(!app_activation_class_eligible(class), "{class}");
        }
        for class in ["CabinetWClass", "Notepad", "Chrome_WidgetWin_1"] {
            assert!(app_activation_class_eligible(class), "{class}");
        }
    }
}

#[cfg(test)]
mod scan_code_tests {
    use super::scan_code;

    /// 美式布局下 MapVirtualKeyW(MAPVK_VK_TO_VSC) 的典型结果。
    fn us(vk: u16) -> u32 {
        match vk {
            0x24 => 0x47, // Home 与小键盘 7 同码
            0x25 => 0x4B,
            0x2E => 0x53,
            0x08 => 0x0E,
            0x11 => 0x1D,
            0x52 => 0x13,
            0x5B => 0x5B,
            0xA5 => 0x38,
            0x67 => 0x47,
            0xAF => 0x30, // 音量+ 换算成 B 的位置
            0x2C => 0x54,
            _ => 0,
        }
    }

    #[test]
    fn navigation_keys_are_extended_and_keep_their_scan_code() {
        assert_eq!(scan_code(0x24, us), (0x47, true));
        assert_eq!(scan_code(0x25, us), (0x4B, true));
        assert_eq!(scan_code(0x2E, us), (0x53, true));
        assert_eq!(scan_code(0x5B, us), (0x5B, true));
        assert_eq!(scan_code(0xA5, us), (0x38, true));
    }

    #[test]
    fn ordinary_keys_get_plain_scan_codes() {
        assert_eq!(scan_code(0x52, us), (0x13, false)); // R
        assert_eq!(scan_code(0x08, us), (0x0E, false)); // Backspace
        assert_eq!(scan_code(0x11, us), (0x1D, false)); // Ctrl
        assert_eq!(scan_code(0x67, us), (0x47, false)); // 小键盘 7 与 Home 区分只靠扩展标志
    }

    #[test]
    fn media_and_unknown_keys_keep_scan_code_zero() {
        assert_eq!(scan_code(0xAF, us), (0, false));
        assert_eq!(scan_code(0xB3, us), (0, false));
        assert_eq!(scan_code(0x15, us), (0, false)); // 输入法键
        assert_eq!(scan_code(0x41, |_| 0), (0, false)); // 换算失败
    }

    #[test]
    fn print_screen_uses_its_physical_scan_code() {
        assert_eq!(scan_code(0x2C, us), (0x37, true));
    }

    #[test]
    fn real_keyboard_layout_maps_navigation_keys() {
        use windows::Win32::UI::Input::KeyboardAndMouse::{MapVirtualKeyW, MAPVK_VK_TO_VSC};
        let map = |vk: u16| unsafe { MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC) };
        let (home, ext) = scan_code(0x24, map);
        assert!(home != 0 && ext);
        assert_eq!(scan_code(0x24, map).0, scan_code(0x67, map).0);
        assert!(scan_code(0x52, map).0 != 0);
    }
}

#[cfg(test)]
mod selection_capture_tests {
    /// 真机复现：没有选中文字时，不应把剪贴板里原有的内容当成选中文字。
    /// 只在记事本确实是前台窗口时注入按键，避免把 Ctrl+C 发给别的窗口。
    /// 运行：cargo test -p mousetroke-win stale_clipboard -- --ignored --nocapture
    #[test]
    #[ignore = "真机：会打开记事本并临时改写剪贴板（结束后恢复）"]
    fn stale_clipboard_is_not_taken_as_selection() {
        use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};
        // 已有记事本在运行就跳过：只对本测试打开的记事本输入和关闭，不碰用户的文档
        let running = std::process::Command::new("tasklist").args(["/FI", "IMAGENAME eq notepad.exe", "/NH"]).output().map(|o| String::from_utf8_lossy(&o.stdout).to_ascii_lowercase().contains("notepad.exe")).unwrap_or(true);
        if running {
            eprintln!("已有记事本在运行，跳过");
            return;
        }
        // Win11 记事本会恢复上次未保存的标签页：有保存的标签页（可能是用户的）就跳过，免得粘贴进去
        let tabs = std::env::var_os("LOCALAPPDATA").map(|d| std::path::Path::new(&d).join(r"Packages\Microsoft.WindowsNotepad_8wekyb3d8bbwe\LocalState\TabState"));
        // 目录不存在 = 没有标签页；读不了就返回 None（不当成空，宁可跳过）
        let listing = |dir: &Option<std::path::PathBuf>| -> Option<Vec<std::path::PathBuf>> {
            let Some(dir) = dir else { return Some(Vec::new()) };
            if !dir.exists() {
                return Some(Vec::new());
            }
            std::fs::read_dir(dir).ok()?.map(|e| e.map(|e| e.path())).collect::<Result<Vec<_>, _>>().ok()
        };
        if listing(&tabs).is_none_or(|t| !t.is_empty()) {
            eprintln!("记事本有保存的标签页（或无法确认），跳过");
            return;
        }
        // 只在能原样恢复时才动剪贴板：有文字就必须读得到；有图片等非文字内容则跳过
        use windows::Win32::System::DataExchange::{CountClipboardFormats, IsClipboardFormatAvailable};
        let has_text = unsafe { IsClipboardFormatAvailable(13 /* CF_UNICODETEXT */) }.is_ok();
        let saved = if has_text {
            let Some(text) = (0..5).find_map(|_| super::get_clipboard_text().or_else(|| { std::thread::sleep(std::time::Duration::from_millis(100)); None })) else {
                eprintln!("读不到剪贴板文字，跳过");
                return;
            };
            Some(text)
        } else if unsafe { CountClipboardFormats() } > 0 {
            eprintln!("剪贴板里有非文字内容，跳过");
            return;
        } else {
            None
        };
        super::set_clipboard_text("MT-OLD-CLIPBOARD");
        let mut child = std::process::Command::new(super::system_exe(r"System32\notepad.exe")).spawn().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(2500));
        let fg = unsafe { GetForegroundWindow() };
        let mut pid = 0u32;
        unsafe { GetWindowThreadProcessId(fg, Some(&mut pid)) };
        let exe = crate::targeting::exe_of(pid).unwrap_or_default();
        let result = exe.to_ascii_lowercase().ends_with("notepad.exe").then(|| {
            // 先在正文区点一下，确保输入焦点在编辑区（Win11 记事本打开时焦点可能在标签栏）
            let mut r = windows::Win32::Foundation::RECT::default();
            unsafe { let _ = windows::Win32::UI::WindowsAndMessaging::GetWindowRect(fg, &mut r); }
            let (x, y) = ((r.left + r.right) / 2, (r.top + r.bottom) / 2);
            unsafe { let _ = windows::Win32::UI::WindowsAndMessaging::SetCursorPos(x, y); }
            super::send_mouse(mousetroke_core::schema::MouseButton::Left, true);
            super::send_mouse(mousetroke_core::schema::MouseButton::Left, false);
            std::thread::sleep(std::time::Duration::from_millis(300));
            let none = super::capture_selected_text();
            // 正常情况：放入一段文字、全选后能取到。用粘贴而不是逐字输入：
            // 中文输入法会把逐字输入的字母当成拼音拦下，粘贴和快捷键不受影响
            super::set_clipboard_text("mtselected");
            super::inject_keyseq(&mousetroke_core::keyseq::parse("^v", false).unwrap());
            std::thread::sleep(std::time::Duration::from_millis(300));
            super::set_clipboard_text("MT-OLD-CLIPBOARD");
            super::inject_keyseq(&mousetroke_core::keyseq::parse("^a", false).unwrap());
            std::thread::sleep(std::time::Duration::from_millis(300));
            (none, super::capture_selected_text())
        });
        let _ = child.kill();
        let _ = child.wait();
        let _ = std::process::Command::new("taskkill").args(["/F", "/IM", "Notepad.exe", "/FI", &format!("PID eq {}", pid)]).output();
        // 只删本次测试产生的标签页（开始前为空）
        std::thread::sleep(std::time::Duration::from_millis(500));
        for path in listing(&tabs).unwrap_or_default() {
            let _ = std::fs::remove_file(path);
        }
        match saved {
            Some(text) => super::set_clipboard_text(&text),
            None => super::set_clipboard_text(""),
        }
        let Some(captured) = result else {
            eprintln!("记事本没有成为前台窗口（前台是 {exe}），跳过");
            return;
        };
        eprintln!("没有选中文字 / 全选后取到的内容: {captured:?}");
        assert_eq!(captured.0, None, "把剪贴板里的旧内容当成了选中文字");
        assert_eq!(captured.1.as_deref(), Some("mtselected"), "全选后应取到刚放进去的文字");
    }
}
