//! 托盘图标、隐藏控制窗口、暂停热键与托盘菜单。
//!
//! 控制窗口与钩子同线程：wndproc 通过 [`crate::engine::with_engine`] 直接访问引擎。
//! 它是隐藏的顶层窗口（不是 message-only），以便收到 `WM_DISPLAYCHANGE` 与
//! `TaskbarCreated` 广播（Explorer 重启后需重新添加托盘图标）。
//!
//! 交互：左键单击图标 → 打开设置；右键 → 菜单（暂停/设置/配置文件夹/开机自启/退出）。
//! 暂停时图标变灰，提示文字同步更新。

use std::cell::RefCell;
use std::path::PathBuf;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::Graphics::Gdi::{CreateBitmap, CreateDIBSection, DeleteObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HGDIOBJ};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT, MOD_WIN,
};
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_INFO, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY, NOTIFYICONDATAW, NIN_BALLOONUSERCLICK,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreateIconIndirect, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyIcon, DestroyMenu,
    GetCursorPos, PostMessageW, PostQuitMessage, RegisterClassW, RegisterWindowMessageW, SetForegroundWindow,
    SetTimer, TrackPopupMenuEx, HICON, ICONINFO, MF_CHECKED, MF_SEPARATOR, MF_STRING, TPM_BOTTOMALIGN,
    TPM_RIGHTBUTTON, WM_APP, WM_COMMAND, WM_CONTEXTMENU, WM_DISPLAYCHANGE, WM_HOTKEY, WM_LBUTTONUP, WM_NULL, WM_RBUTTONUP,
    WM_TIMER, WNDCLASSW, WS_EX_TOOLWINDOW, WS_POPUP,
};

use crate::engine::with_engine;
use mousetroke_engine::Reload;
use mousetroke_core::i18n::{keys, t, tf};
use mousetroke_core::keyseq::{parse_hotkey, ModKey};

const WM_TRAY: u32 = WM_APP; // 托盘回调消息
const MENU_TOGGLE_PAUSE: u32 = 1;
const MENU_QUIT: u32 = 2;
const MENU_SETTINGS: u32 = 3;
const MENU_OPEN_DIR: u32 = 4;
const MENU_AUTOSTART: u32 = 5;
const HOTKEY_PAUSE_ID: i32 = 1;
const TOAST_TIMER_ID: usize = 2;
const CONFIG_TIMER_ID: usize = 3;

/// 引擎命令消息（工作线程 → 引擎线程，Lua set_paused 等用）。
pub const WM_ENGINE_CMD: u32 = WM_APP + 7;
pub const CMD_TOGGLE_PAUSE: usize = 1;
pub const CMD_PAUSE_ON: usize = 2;
pub const CMD_PAUSE_OFF: usize = 3;

/// 控制窗口句柄的全局镜像（跨线程 PostMessage 用）。
static TRAY_HWND_GLOBAL: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);
static UPDATE_BALLOON: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// 从任意线程向引擎线程发命令。
pub fn post_engine_cmd(cmd: usize) -> bool {
    let h = TRAY_HWND_GLOBAL.load(std::sync::atomic::Ordering::Relaxed);
    if h == 0 {
        return false;
    }
    unsafe { PostMessageW(Some(HWND(h as *mut core::ffi::c_void)), WM_ENGINE_CMD, WPARAM(cmd), LPARAM(0)).is_ok() }
}

thread_local! {
    static STATE: RefCell<Option<TrayState>> = const { RefCell::new(None) };
}

struct TrayState {
    hwnd: HWND,
    show_icon: bool,
    icon_on: HICON,
    icon_paused: HICON,
    taskbar_created: u32,
}

/// 托盘句柄（drop 时移除图标）。
pub struct Tray;

impl Tray {
    /// 创建托盘（含热键注册与定时器）。`pause_hotkey` 为 keyseq 语法（如 `#({ESC})`）。
    pub fn create(show_icon: bool, pause_hotkey: &str, cfg_path: Option<PathBuf>) -> Option<Self> {
        unsafe {
            let hinst = GetModuleHandleW(None).ok()?;
            let classname = wide("MousetrokeTray");
            let wc = WNDCLASSW {
                lpfnWndProc: Some(tray_wndproc),
                hInstance: hinst.into(),
                lpszClassName: PCWSTR(classname.as_ptr()),
                ..Default::default()
            };
            RegisterClassW(&wc);
            // 隐藏顶层窗口：从不 ShowWindow，但能收到广播消息
            let hwnd = CreateWindowExW(
                WS_EX_TOOLWINDOW,
                PCWSTR(classname.as_ptr()),
                None,
                WS_POPUP,
                0,
                0,
                0,
                0,
                None,
                None,
                Some(hinst.into()),
                None,
            )
            .ok()?;
            TRAY_HWND_GLOBAL.store(hwnd.0 as isize, std::sync::atomic::Ordering::Relaxed);
            let _ = SetTimer(Some(hwnd), TOAST_TIMER_ID, 250, None);
            if cfg_path.is_some() {
                let _ = SetTimer(Some(hwnd), CONFIG_TIMER_ID, 1000, None);
            }
            let taskbar_created = RegisterWindowMessageW(PCWSTR(wide("TaskbarCreated").as_ptr()));

            if let Ok(mut g) = CFG_PATH.lock() {
                g.clone_from(&cfg_path);
            }
            STATE.with(|s| {
                *s.borrow_mut() = Some(TrayState {
                    hwnd,
                    show_icon,
                    icon_on: make_icon(false),
                    icon_paused: make_icon(true),
                    taskbar_created,
                })
            });
            register_pause_hotkey(hwnd, pause_hotkey);
            if show_icon && !shell_icon(NIM_ADD, false) {
                crate::log!("托盘图标创建失败（继续无图标运行）");
            }
            Some(Self)
        }
    }
}

impl Drop for Tray {
    fn drop(&mut self) {
        shell_icon(NIM_DELETE, false);
        STATE.with(|s| {
            if let Some(st) = s.borrow_mut().take() {
                unsafe {
                    let _ = DestroyIcon(st.icon_on);
                    let _ = DestroyIcon(st.icon_paused);
                }
            }
        });
    }
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain([0]).collect()
}

fn copy_wide(dst: &mut [u16], s: &str) {
    let src: Vec<u16> = s.encode_utf16().take(dst.len() - 1).collect();
    dst[..src.len()].copy_from_slice(&src);
    dst[src.len()] = 0;
}

fn base_nid(st: &TrayState) -> NOTIFYICONDATAW {
    NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: st.hwnd,
        uID: 1,
        ..Default::default()
    }
}

/// 添加/更新/删除托盘图标（图标与提示随暂停状态变化）。
fn shell_icon(op: windows::Win32::UI::Shell::NOTIFY_ICON_MESSAGE, paused: bool) -> bool {
    STATE.with(|s| {
        let s = s.borrow();
        let Some(st) = s.as_ref() else { return false };
        if !st.show_icon {
            return true;
        }
        let mut nid = base_nid(st);
        if op != NIM_DELETE {
            nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
            nid.uCallbackMessage = WM_TRAY;
            nid.hIcon = if paused { st.icon_paused } else { st.icon_on };
            copy_wide(&mut nid.szTip, if paused { t(keys::TRAY_TIP_PAUSED) } else { t(keys::TRAY_TIP) });
        }
        unsafe { Shell_NotifyIconW(op, &nid).as_bool() }
    })
}

/// 语言（或暂停状态）变化后刷新托盘悬停提示；菜单每次弹出时现建，自然就是新语言。
pub fn refresh_texts() {
    shell_icon(NIM_MODIFY, is_paused());
}

/// 托盘气泡通知。
pub fn notify(text: &str) -> bool {
    UPDATE_BALLOON.store(false, std::sync::atomic::Ordering::Relaxed);
    STATE.with(|s| {
        let s = s.borrow();
        let Some(st) = s.as_ref() else { return false };
        if !st.show_icon {
            return false;
        }
        let mut nid = base_nid(st);
        nid.uFlags = NIF_INFO;
        copy_wide(&mut nid.szInfo, text);
        copy_wide(&mut nid.szInfoTitle, "Mousetroke");
        unsafe {
            Shell_NotifyIconW(NIM_MODIFY, &nid).as_bool()
        }
    })
}

fn register_pause_hotkey(hwnd: HWND, hotkey: &str) {
    unsafe {
        let _ = UnregisterHotKey(Some(hwnd), HOTKEY_PAUSE_ID);
        let Some((mods, vk)) = parse_hotkey(hotkey) else {
            if !hotkey.is_empty() {
                crate::log!("暂停热键无法解析: {hotkey:?}");
            }
            return;
        };
        let mut flags = MOD_NOREPEAT;
        for m in &mods {
            flags |= match m {
                ModKey::Ctrl => MOD_CONTROL,
                ModKey::Alt => MOD_ALT,
                ModKey::Shift => MOD_SHIFT,
                ModKey::Win => MOD_WIN,
            };
        }
        let _: HOT_KEY_MODIFIERS = flags;
        if RegisterHotKey(Some(hwnd), HOTKEY_PAUSE_ID, flags, vk as u32).is_err() {
            crate::log!("暂停热键注册失败（可能被其他程序占用）: {hotkey}");
        }
    }
}

fn is_paused() -> bool {
    with_engine(|e| e.is_paused()).unwrap_or(false)
}

/// 暂停状态变化后同步图标与提示。
fn sync_pause_ui(announce: bool) {
    let paused = is_paused();
    shell_icon(NIM_MODIFY, paused);
    if announce {
        // 日志保持中文（不翻译）；屏幕 toast 走词条
        if paused {
            crate::log!("已暂停（再次按暂停热键恢复）");
            crate::engine::push_toast(t(keys::TRAY_PAUSED_TOAST).into(), 1200);
        } else {
            crate::log!("已恢复");
            crate::engine::push_toast(t(keys::TRAY_RESUMED_TOAST).into(), 1200);
        }
    }
}

/// 配置路径的全局镜像（工作线程也能打开设置）。
static CFG_PATH: std::sync::Mutex<Option<PathBuf>> = std::sync::Mutex::new(None);

fn cfg_path() -> Option<PathBuf> {
    CFG_PATH.lock().ok().and_then(|g| g.clone())
}

/// 打开设置：以 `settings` 参数再启动一个自己（界面是独立进程，引擎进程里不放 WebView）。
/// 任意线程可调用（Lua ShowSettings 在工作线程）。
pub fn open_settings() {
    open_settings_page(false);
}

fn open_settings_page(about: bool) {
    let Ok(exe) = std::env::current_exe() else { return };
    // 直接创建进程：ShellExecute 会把整套外壳组件加载进常驻的引擎进程（实测工作集 +12MB）
    let mut cmd = std::process::Command::new(exe);
    cmd.arg("settings").args(cfg_path());
    if about { cmd.arg("--about"); }
    if let Err(e) = cmd.spawn() {
        crate::log!("打开设置失败: {e}");
    }
}

fn open_config_dir() {
    if let Some(dir) = cfg_path().and_then(|p| p.parent().map(|d| d.to_path_buf())) {
        let _ = std::process::Command::new(crate::input::system_exe("explorer.exe")).arg(dir).spawn(); // 同上：不用 ShellExecute
    }
}

fn poll_config(hwnd: HWND) {
    match with_engine(|e| e.poll_config()).flatten() {
        Some(Reload::Applied { scopes, bindings, hotkey_changed }) => {
            crate::log!("配置已热加载: {scopes} 作用域 / {bindings} 绑定");
            crate::engine::push_toast(t(keys::TOAST_CONFIG_UPDATED).into(), 1200);
            if let Some(hk) = hotkey_changed {
                register_pause_hotkey(hwnd, &hk);
            }
        }
        Some(Reload::Failed(e)) => {
            crate::log!("配置热加载失败（保持旧配置）: {e}");
            notify(t(keys::TRAY_RELOAD_FAILED));
        }
        None => {}
    }
}

unsafe extern "system" fn tray_wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_TRAY => {
            match (lparam.0 & 0xffff) as u32 {
                WM_LBUTTONUP => open_settings(),
                NIN_BALLOONUSERCLICK if UPDATE_BALLOON.swap(false, std::sync::atomic::Ordering::Relaxed) => open_settings_page(true),
                // WM_CONTEXTMENU：键盘选中托盘图标后按菜单键/Shift+F10
                WM_RBUTTONUP | WM_CONTEXTMENU => show_menu(hwnd),
                _ => {}
            }
            LRESULT(0)
        }
        WM_COMMAND => {
            match (wparam.0 & 0xffff) as u32 {
                MENU_TOGGLE_PAUSE => {
                    with_engine(|e| e.toggle_paused());
                    sync_pause_ui(true);
                }
                MENU_SETTINGS => open_settings(),
                MENU_OPEN_DIR => open_config_dir(),
                MENU_AUTOSTART => {
                    let on = !crate::autostart::is_enabled();
                    match crate::autostart::set(on) {
                        Ok(_) => { notify(t(if on { keys::TRAY_AUTOSTART_ON } else { keys::TRAY_AUTOSTART_OFF })); }
                        Err(e) => { notify(&tf(keys::ERROR_AUTOSTART, &[("err", &e)])); }
                    }
                }
                MENU_QUIT => {
                    // 退出时设置程序一起关（它会先保存未写盘的修改）
                    crate::targeting::close_app_windows(&crate::targeting::own_exe_name());
                    crate::targeting::close_app_windows("mousetroke-settings.exe"); // 开发版的独立设置程序
                    PostQuitMessage(0)
                }
                _ => {}
            }
            LRESULT(0)
        }
        WM_HOTKEY if wparam.0 as i32 == HOTKEY_PAUSE_ID => {
            with_engine(|e| e.toggle_paused());
            sync_pause_ui(true);
            LRESULT(0)
        }
        WM_ENGINE_CMD => {
            with_engine(|e| match wparam.0 {
                CMD_TOGGLE_PAUSE => e.toggle_paused(),
                CMD_PAUSE_ON => e.set_paused(true),
                CMD_PAUSE_OFF => e.set_paused(false),
                _ => {}
            });
            sync_pause_ui(false);
            LRESULT(0)
        }
        WM_TIMER if wparam.0 == TOAST_TIMER_ID => {
            if let Some((version, ack)) = crate::update::take_notice() {
                let shown = notify(&tf(keys::UPDATE_AVAILABLE, &[("version", &version)]));
                UPDATE_BALLOON.store(shown, std::sync::atomic::Ordering::Relaxed);
                let _ = ack.send(shown);
            }
            with_engine(|e| e.toast_tick());
            LRESULT(0)
        }
        WM_TIMER if wparam.0 == CONFIG_TIMER_ID => {
            crate::engine::sync_display();
            poll_config(hwnd);
            LRESULT(0)
        }
        WM_DISPLAYCHANGE => {
            with_engine(|e| e.on_display_change());
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        _ if STATE.with(|s| s.borrow().as_ref().is_some_and(|st| st.taskbar_created == msg)) => {
            // Explorer 重启：重新添加图标
            shell_icon(NIM_ADD, is_paused());
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

/// 设置进程查找引擎控制窗口；窗口 PID 用于更新后的有界等待。
pub fn engine_window() -> Option<(HWND, u32)> {
    use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, GetWindowThreadProcessId};
    unsafe {
        let hwnd = FindWindowW(PCWSTR(wide("MousetrokeTray").as_ptr()), PCWSTR::null()).ok()?;
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        (pid != 0).then_some((hwnd, pid))
    }
}

/// 跨进程复用托盘「退出」，由消息循环卸载钩子并关闭设置窗口。
pub fn request_exit(hwnd: HWND) -> bool {
    unsafe { PostMessageW(Some(hwnd), WM_COMMAND, WPARAM(MENU_QUIT as usize), LPARAM(0)).is_ok() }
}

unsafe fn show_menu(hwnd: HWND) {
    let paused = is_paused();
    let Ok(menu) = CreatePopupMenu() else { return };
    let item = |id: u32, text: &str, checked: bool| {
        let t = wide(text);
        let flags = if checked { MF_STRING | MF_CHECKED } else { MF_STRING };
        let _ = AppendMenuW(menu, flags, id as usize, PCWSTR(t.as_ptr()));
    };
    let sep = || {
        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
    };
    item(MENU_SETTINGS, t(keys::TRAY_MENU_SETTINGS), false);
    item(MENU_TOGGLE_PAUSE, t(if paused { keys::TRAY_MENU_RESUME } else { keys::TRAY_MENU_PAUSE }), false);
    sep();
    item(MENU_AUTOSTART, t(keys::TRAY_MENU_AUTOSTART), crate::autostart::is_enabled());
    item(MENU_OPEN_DIR, t(keys::TRAY_MENU_OPEN_DIR), false);
    sep();
    item(MENU_QUIT, t(keys::TRAY_MENU_QUIT), false);

    let mut pt = POINT::default();
    let _ = GetCursorPos(&mut pt);
    let _ = SetForegroundWindow(hwnd);
    let _ = TrackPopupMenuEx(menu, TPM_RIGHTBUTTON.0 | TPM_BOTTOMALIGN.0, pt.x, pt.y, hwnd, None);
    let _ = PostMessageW(Some(hwnd), WM_NULL, WPARAM(0), LPARAM(0));
    let _ = DestroyMenu(menu);
}

// ---------------------------------------------------------------- 图标绘制

/// 程序化绘制 32×32 托盘图标：圆角方块底 + 白色手势折线（↘↗ 形）。暂停态为灰色。
fn make_icon(paused: bool) -> HICON {
    const N: i32 = 32;
    let bg: (f32, f32, f32) = if paused { (0.55, 0.57, 0.60) } else { (0.0, 0.62, 0.95) };
    // 手势折线（像素坐标）
    let path = [(7.0f32, 11.0f32), (13.5, 21.0), (19.0, 13.0), (25.0, 21.0)];
    let mut px = vec![0u32; (N * N) as usize];
    for y in 0..N {
        for x in 0..N {
            let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
            // 圆角矩形覆盖度（半径 7）
            let r = 7.0;
            let (cx, cy) = (fx.clamp(r, N as f32 - r), fy.clamp(r, N as f32 - r));
            let d_bg = ((fx - cx).powi(2) + (fy - cy).powi(2)).sqrt() - r;
            let a_bg = (0.5 - d_bg).clamp(0.0, 1.0);
            // 折线覆盖度（半宽 1.8）
            let d_line = path
                .windows(2)
                .map(|w| seg_dist((fx, fy), w[0], w[1]))
                .fold(f32::MAX, f32::min);
            let a_line = (2.3 - d_line).clamp(0.0, 1.0) * a_bg;
            let (r_, g_, b_) = (
                bg.0 * (1.0 - a_line) + a_line,
                bg.1 * (1.0 - a_line) + a_line,
                bg.2 * (1.0 - a_line) + a_line,
            );
            // 预乘 BGRA
            let a = a_bg;
            let c = |v: f32| ((v * a) * 255.0).round() as u32;
            px[(y * N + x) as usize] = ((a * 255.0).round() as u32) << 24 | c(r_) << 16 | c(g_) << 8 | c(b_);
        }
    }
    unsafe {
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: N,
                biHeight: -N,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
        let Ok(color) = CreateDIBSection(None, &bmi, DIB_RGB_COLORS, &mut bits, None, 0) else {
            return HICON::default();
        };
        std::ptr::copy_nonoverlapping(px.as_ptr(), bits as *mut u32, px.len());
        let mask = CreateBitmap(N, N, 1, 1, None);
        let info = ICONINFO { fIcon: true.into(), xHotspot: 0, yHotspot: 0, hbmMask: mask, hbmColor: color };
        let icon = CreateIconIndirect(&info).unwrap_or_default();
        let _ = DeleteObject(HGDIOBJ(color.0));
        let _ = DeleteObject(HGDIOBJ(mask.0));
        icon
    }
}

fn seg_dist(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let (abx, aby) = (b.0 - a.0, b.1 - a.1);
    let t = (((p.0 - a.0) * abx + (p.1 - a.1) * aby) / (abx * abx + aby * aby)).clamp(0.0, 1.0);
    let (qx, qy) = (a.0 + abx * t, a.1 + aby * t);
    ((p.0 - qx).powi(2) + (p.1 - qy).powi(2)).sqrt()
}
