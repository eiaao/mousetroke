//! 添加程序的两种方式：拖入 exe/快捷方式（[`resolve_app`]），或在屏幕上点选窗口（[`pick_window`]）。
//!
//! 点选：专用线程装 WH_MOUSE_LL / WH_KEYBOARD_LL（比引擎晚装 → 先收到事件，点击被吞、
//! 不会激活目标窗口也不会触发手势），系统光标临时换成十字，悬停的窗口画高亮框，
//! 顶部居中显示提示。左键确认、右键或 Esc 取消、60 秒超时。光标在任何退出路径上都会还原。

use std::cell::RefCell;
use std::path::Path;
use std::time::{Duration, Instant};

use mousetroke_win::targeting;
use serde::Serialize;
use windows::core::{w, Interface, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, CreateRectRgn, CreateSolidBrush, CombineRgn, DeleteObject, DrawTextW, EndPaint, FillRect,
    GetMonitorInfoW, MonitorFromPoint, SelectObject, SetBkMode, SetTextColor, SetWindowRgn, DT_CENTER, DT_SINGLELINE,
    DT_VCENTER, HGDIOBJ, MONITORINFO, MONITOR_DEFAULTTONEAREST, PAINTSTRUCT, RGN_DIFF, TRANSPARENT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;

#[derive(Serialize, Clone)]
pub struct PickedApp {
    pub exe: String,
    pub title: String,
}

// ---------------------------------------------------------------- 拖入

/// 拖入的文件 → 程序完整路径。支持 .exe 与指向 exe 的 .lnk 快捷方式。
pub fn resolve_app(path: &str) -> Result<String, crate::Msg> {
    use crate::{keys, Msg};
    let p = Path::new(path);
    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    let target = match ext.as_str() {
        "exe" => path.to_string(),
        "lnk" => shortcut_target(path).ok_or_else(|| {
            Msg::new(keys::ERROR_SHORTCUT_TARGET, &[("file", file_name(path).to_string())])
        })?,
        _ => return Err(Msg::new(keys::ERROR_NOT_A_PROGRAM, &[("file", file_name(path).to_string())])),
    };
    if !target.to_ascii_lowercase().ends_with(".exe") {
        return Err(Msg::new(keys::ERROR_SHORTCUT_NOT_PROGRAM, &[("file", file_name(path).to_string())]));
    }
    Ok(target)
}

fn file_name(p: &str) -> &str {
    mousetroke_core::matcher::file_name(p)
}

/// .lnk 的目标路径（IShellLinkW）。
fn shortcut_target(lnk: &str) -> Option<String> {
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, IPersistFile, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
        STGM_READ,
    };
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink, SLGP_RAWPATH};
    unsafe {
        let init = CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_ok();
        let result = (|| {
            let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).ok()?;
            let wide: Vec<u16> = lnk.encode_utf16().chain([0]).collect();
            link.cast::<IPersistFile>().ok()?.Load(PCWSTR(wide.as_ptr()), STGM_READ).ok()?;
            let mut buf = [0u16; 1024];
            link.GetPath(&mut buf, std::ptr::null_mut(), SLGP_RAWPATH.0 as u32).ok()?;
            let n = buf.iter().position(|&c| c == 0).unwrap_or(0);
            let raw = String::from_utf16_lossy(&buf[..n]);
            (!raw.is_empty()).then(|| expand_env(&raw))
        })();
        if init {
            CoUninitialize();
        }
        result
    }
}

/// 展开 `%ProgramFiles%` 这类环境变量。
fn expand_env(s: &str) -> String {
    use windows::Win32::System::Environment::ExpandEnvironmentStringsW;
    let src: Vec<u16> = s.encode_utf16().chain([0]).collect();
    let mut buf = vec![0u16; 2048];
    let n = unsafe { ExpandEnvironmentStringsW(PCWSTR(src.as_ptr()), Some(&mut buf)) } as usize;
    if n == 0 || n > buf.len() {
        return s.to_string();
    }
    String::from_utf16_lossy(&buf[..n.saturating_sub(1)])
}

// ---------------------------------------------------------------- 点选窗口

const TIMEOUT: Duration = Duration::from_secs(60);
const FRAME: i32 = 4;
const ACCENT: COLORREF = COLORREF(0x00D9_9800); // #0098D9（BGR）

#[derive(Default)]
struct PickState {
    frame: HWND,
    hint: HWND,
    hovered: Option<HWND>,
    /// 结束：Some(Some(窗口)) = 选中，Some(None) = 取消。
    done: Option<Option<HWND>>,
}

thread_local! {
    static STATE: RefCell<PickState> = RefCell::new(PickState::default());
}

/// 交互式点选一个窗口，返回它的程序。取消返回 Ok(None)。阻塞直到结束（在专用线程里跑）。
pub fn pick_window() -> Result<Option<PickedApp>, crate::Msg> {
    let picked = std::thread::spawn(pick_on_this_thread)
        .join()
        .map_err(|_| crate::Msg::new(crate::keys::ERROR_PICK_FAILED, &[]))?;
    picked.map_err(|_| crate::Msg::new(crate::keys::ERROR_PICK_FAILED, &[]))
}

/// 系统光标临时换成十字；析构时还原（任何退出路径都会执行）。
struct CrossCursor;

impl CrossCursor {
    fn set() -> Self {
        unsafe {
            if let Ok(cross) = LoadCursorW(None, IDC_CROSS) {
                for id in [OCR_NORMAL, OCR_IBEAM, OCR_HAND] {
                    if let Ok(copy) = CopyIcon(HICON(cross.0)) {
                        let _ = SetSystemCursor(HCURSOR(copy.0), id);
                    }
                }
            }
        }
        CrossCursor
    }
}

impl Drop for CrossCursor {
    fn drop(&mut self) {
        unsafe {
            let _ = SystemParametersInfoW(SPI_SETCURSORS, 0, None, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0));
        }
    }
}

fn pick_on_this_thread() -> Result<Option<PickedApp>, String> {
    unsafe {
        let hinst = GetModuleHandleW(None).map_err(|e| e.to_string())?;
        let class = w!("MousetrokePicker");
        let wc = WNDCLASSW { lpfnWndProc: Some(helper_proc), hInstance: hinst.into(), lpszClassName: class, ..Default::default() };
        RegisterClassW(&wc);
        let ex = WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE;
        let make = |title: PCWSTR| CreateWindowExW(ex, class, title, WS_POPUP, 0, 0, 1, 1, None, None, Some(hinst.into()), None);
        let frame = make(w!("frame")).map_err(|e| e.to_string())?;
        let hint = make(w!("hint")).map_err(|e| e.to_string())?;
        let _ = SetLayeredWindowAttributes(frame, COLORREF(0), 255, LWA_ALPHA);
        let _ = SetLayeredWindowAttributes(hint, COLORREF(0), 235, LWA_ALPHA);
        STATE.with(|s| *s.borrow_mut() = PickState { frame, hint, ..Default::default() });
        place_hint(hint);

        let _cursor = CrossCursor::set();
        let mouse = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook), None, 0).map_err(|e| e.to_string())?;
        let kbd = SetWindowsHookExW(WH_KEYBOARD_LL, Some(kbd_hook), None, 0).ok();
        SetTimer(None, 0, 100, None);

        // 先高亮当前光标下的窗口
        let mut pt = POINT::default();
        let _ = GetCursorPos(&mut pt);
        hover(pt);

        let start = Instant::now();
        let mut msg = MSG::default();
        while STATE.with(|s| s.borrow().done.is_none()) && start.elapsed() < TIMEOUT {
            if !GetMessageW(&mut msg, None, 0, 0).as_bool() {
                break;
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        let _ = UnhookWindowsHookEx(mouse);
        if let Some(k) = kbd {
            let _ = UnhookWindowsHookEx(k);
        }
        let _ = DestroyWindow(frame);
        let _ = DestroyWindow(hint);
        let picked = STATE.with(|s| s.borrow_mut().done.take()).flatten();
        Ok(picked.and_then(app_of))
    }
}

/// 根窗口 → 程序。桌面不算程序。
fn app_of(root: HWND) -> Option<PickedApp> {
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(root, Some(&mut pid)) };
    let exe = targeting::exe_of(pid)?;
    Some(PickedApp { exe, title: targeting::window_title(root) })
}

/// 点 (x, y) 下的可选根窗口（排除桌面与本程序自己的窗口）。
fn root_at(pt: POINT) -> Option<HWND> {
    unsafe {
        let hwnd = WindowFromPoint(pt);
        if hwnd.0.is_null() {
            return None;
        }
        let root = GetAncestor(hwnd, GA_ROOT);
        let root = if root.0.is_null() { hwnd } else { root };
        let class = targeting::class_name(root);
        let mut pid = 0u32;
        GetWindowThreadProcessId(root, Some(&mut pid));
        let desktop = matches!(class.as_str(), "Progman" | "WorkerW");
        (!desktop && pid != std::process::id()).then_some(root)
    }
}

fn hover(pt: POINT) {
    let target = root_at(pt);
    let (frame, changed) = STATE.with(|s| {
        let mut s = s.borrow_mut();
        let changed = s.hovered != target;
        s.hovered = target;
        (s.frame, changed)
    });
    if !changed {
        return;
    }
    unsafe {
        let Some(target) = target else {
            let _ = ShowWindow(frame, SW_HIDE);
            return;
        };
        let mut r = RECT::default();
        if DwmGetWindowAttribute(target, DWMWA_EXTENDED_FRAME_BOUNDS, (&mut r as *mut RECT).cast(), std::mem::size_of::<RECT>() as u32).is_err() {
            let _ = GetWindowRect(target, &mut r);
        }
        let (w, h) = (r.right - r.left, r.bottom - r.top);
        // 只留四条边：外框减内框
        let outer = CreateRectRgn(0, 0, w, h);
        let inner = CreateRectRgn(FRAME, FRAME, w - FRAME, h - FRAME);
        CombineRgn(Some(outer), Some(outer), Some(inner), RGN_DIFF);
        let _ = DeleteObject(HGDIOBJ(inner.0));
        let _ = SetWindowPos(frame, Some(HWND_TOPMOST), r.left, r.top, w, h, SWP_NOACTIVATE | SWP_SHOWWINDOW);
        SetWindowRgn(frame, Some(outer), true); // 区域归窗口所有，不再删除
    }
}

/// 提示条：光标所在显示器顶部居中。
fn place_hint(hint: HWND) {
    unsafe {
        let mut pt = POINT::default();
        let _ = GetCursorPos(&mut pt);
        let mut mi = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
        let _ = GetMonitorInfoW(MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST), &mut mi);
        let wa = mi.rcWork;
        let (w, h) = (460, 44);
        let x = (wa.left + wa.right) / 2 - w / 2;
        let _ = SetWindowPos(hint, Some(HWND_TOPMOST), x, wa.top + 24, w, h, SWP_NOACTIVATE | SWP_SHOWWINDOW);
        let rgn = windows::Win32::Graphics::Gdi::CreateRoundRectRgn(0, 0, w + 1, h + 1, 14, 14);
        SetWindowRgn(hint, Some(rgn), true);
    }
}

unsafe extern "system" fn helper_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let dc = BeginPaint(hwnd, &mut ps);
            let mut rc = RECT::default();
            let _ = GetClientRect(hwnd, &mut rc);
            let is_hint = STATE.with(|s| s.borrow().hint == hwnd);
            let brush = CreateSolidBrush(if is_hint { COLORREF(0x0028_1C18) } else { ACCENT });
            FillRect(dc, &rc, brush);
            let _ = DeleteObject(HGDIOBJ(brush.0));
            if is_hint {
                let lang = mousetroke_core::i18n::lang();
                let face: Vec<u16> = mousetroke_core::i18n::ui_font(lang).encode_utf16().chain([0]).collect();
                let font = CreateFontW(-15, 0, 0, 0, 600, 0, 0, 0, windows::Win32::Graphics::Gdi::FONT_CHARSET(1),
                    Default::default(), Default::default(), windows::Win32::Graphics::Gdi::FONT_QUALITY(5), 0, PCWSTR(face.as_ptr()));
                let old = SelectObject(dc, HGDIOBJ(font.0));
                SetBkMode(dc, TRANSPARENT);
                SetTextColor(dc, COLORREF(0x00FF_FFFF));
                let hint = mousetroke_core::i18n::t(mousetroke_core::i18n::keys::UI_PICKER_LIVE_HINT);
                let mut text: Vec<u16> = hint.encode_utf16().collect();
                DrawTextW(dc, &mut text, &mut rc, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
                SelectObject(dc, old);
                let _ = DeleteObject(HGDIOBJ(font.0));
            }
            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn finish(result: Option<HWND>) {
    STATE.with(|s| s.borrow_mut().done = Some(result));
    unsafe { wake() };
}

/// 唤醒消息循环检查结束标志。
unsafe fn wake() {
    let _ = PostThreadMessageW(windows::Win32::System::Threading::GetCurrentThreadId(), WM_NULL, WPARAM(0), LPARAM(0));
}

unsafe extern "system" fn mouse_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code < 0 {
        return CallNextHookEx(None, code, wparam, lparam);
    }
    let info = &*(lparam.0 as *const MSLLHOOKSTRUCT);
    match wparam.0 as u32 {
        WM_MOUSEMOVE => {
            hover(info.pt);
            CallNextHookEx(None, code, wparam, lparam)
        }
        WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN => LRESULT(1), // 吞掉，不激活目标窗口
        WM_LBUTTONUP => {
            finish(root_at(info.pt));
            LRESULT(1)
        }
        WM_RBUTTONUP | WM_MBUTTONUP => {
            finish(None);
            LRESULT(1)
        }
        _ => CallNextHookEx(None, code, wparam, lparam),
    }
}

unsafe extern "system" fn kbd_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let info = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
        if info.vkCode == 0x1B {
            if wparam.0 as u32 == WM_KEYUP {
                finish(None);
            }
            return LRESULT(1);
        }
    }
    CallNextHookEx(None, code, wparam, lparam)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_programs() {
        assert!(resolve_app(r"C:\x\readme.txt").is_err());
        assert_eq!(resolve_app(r"C:\Apps\Foo.EXE").unwrap(), r"C:\Apps\Foo.EXE");
    }

    #[test]
    fn resolves_start_menu_shortcut() {
        // 系统自带的快捷方式（不同机器可能缺失，找到一个即可）
        let dirs = [r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs"];
        let lnk = dirs.iter().flat_map(|d| walk(Path::new(d))).find(|p| {
            p.extension().is_some_and(|e| e.eq_ignore_ascii_case("lnk")) && shortcut_target(&p.to_string_lossy()).is_some_and(|t| t.to_ascii_lowercase().ends_with(".exe"))
        });
        let lnk = lnk.expect("开始菜单里应至少有一个指向 exe 的快捷方式");
        let exe = resolve_app(&lnk.to_string_lossy()).unwrap();
        assert!(Path::new(&exe).exists(), "{lnk:?} → {exe}");
    }

    fn walk(dir: &Path) -> Vec<std::path::PathBuf> {
        let mut out = Vec::new();
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    out.extend(walk(&p));
                } else {
                    out.push(p);
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod live {
    /// 手动：`cargo test -p mousetroke-settings pick_live -- --ignored --nocapture`。
    /// 在光标当前位置注入一次左键（点选器的钩子先收到并吞掉），打印识别出的程序；再测 Esc 取消。
    #[test]
    #[ignore]
    fn pick_live() {
        use windows::Win32::UI::Input::KeyboardAndMouse::*;
        let click = || {
            std::thread::sleep(std::time::Duration::from_millis(400));
            let mk = |f| INPUT { r#type: INPUT_MOUSE, Anonymous: INPUT_0 { mi: MOUSEINPUT { dwFlags: f, ..Default::default() } } };
            unsafe { SendInput(&[mk(MOUSEEVENTF_LEFTDOWN), mk(MOUSEEVENTF_LEFTUP)], std::mem::size_of::<INPUT>() as i32) };
        };
        std::thread::spawn(click);
        let t0 = std::time::Instant::now();
        let picked = super::pick_window().unwrap();
        println!("点选结果: {:?}（{}ms）", picked.as_ref().map(|p| (&p.exe, &p.title)), t0.elapsed().as_millis());
        assert!(picked.is_some());

        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_millis(400));
            let k = |f| INPUT { r#type: INPUT_KEYBOARD, Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: VK_ESCAPE, dwFlags: f, ..Default::default() } } };
            unsafe { SendInput(&[k(KEYBD_EVENT_FLAGS(0)), k(KEYEVENTF_KEYUP)], std::mem::size_of::<INPUT>() as i32) };
        });
        assert!(super::pick_window().unwrap().is_none(), "Esc 应取消");
        println!("Esc 取消: ok");
    }
}
