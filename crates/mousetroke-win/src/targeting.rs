//! 目标解析：鼠标下窗口 → 根窗口 → 进程 exe 路径 / 桌面识别。

use windows::Win32::Foundation::{CloseHandle, HWND, POINT};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
    PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetAncestor, GetClassNameW, GetDesktopWindow, GetWindowThreadProcessId, GA_ROOT,
};

/// 手势开始时锁定的目标上下文。
#[derive(Debug, Clone)]
pub struct Target {
    pub hwnd: HWND,
    pub pid: u32,
    /// 目标进程 exe 完整路径（可能取不到）。
    pub exe: Option<String>,
    pub is_desktop: bool,
    /// 根窗口类名。
    pub class: String,
}

/// 系统外壳界面：任务栏、通知区域及其溢出面板、开始菜单、弹出菜单等。
/// 这些地方右键必须原样交给系统（托盘图标菜单、任务栏跳转列表），不做手势。
const SHELL_UI_CLASSES: &[&str] = &[
    "Shell_TrayWnd",
    "Shell_SecondaryTrayWnd",
    "NotifyIconOverflowWindow",
    "TopLevelWindowForOverflowXamlIsland",
    "Xaml_WindowedPopupClass",
    "XamlExplorerHostIslandWindow",
    "Windows.UI.Core.CoreWindow",
    "#32768",
];

impl Target {
    pub fn is_shell_ui(&self) -> bool {
        SHELL_UI_CLASSES.contains(&self.class.as_str())
    }
}

/// 解析 (x, y) 处的目标。
pub fn resolve_under_cursor(x: i32, y: i32) -> Target {
    unsafe {
        let mut hwnd = windows::Win32::UI::WindowsAndMessaging::WindowFromPoint(POINT { x, y });
        if hwnd.0.is_null() {
            hwnd = GetDesktopWindow();
        }
        let mut root = GetAncestor(hwnd, GA_ROOT);
        if root.0.is_null() {
            root = hwnd;
        }
        let desktop = GetDesktopWindow();

        let class = class_name(root);
        let is_desktop = root == desktop
            || class == "Progman"
            || class == "WorkerW"
            || class == "SHELLDLL_DefView";

        let mut pid = 0u32;
        GetWindowThreadProcessId(root, Some(&mut pid));

        Target {
            hwnd: root,
            pid,
            exe: if is_desktop { None } else { exe_of(pid) },
            is_desktop,
            class,
        }
    }
}

/// 窗口类名。
pub fn class_name(hwnd: HWND) -> String {
    let mut buf = [0u16; 256];
    let n = unsafe { GetClassNameW(hwnd, &mut buf) };
    if n == 0 {
        return String::new();
    }
    String::from_utf16_lossy(&buf[..n as usize])
}

/// 进程 exe 路径。
pub fn exe_of(pid: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 1024];
        let mut size = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, windows::core::PWSTR(buf.as_mut_ptr()), &mut size);
        let _ = CloseHandle(handle);
        ok.ok()?;
        Some(String::from_utf16_lossy(&buf[..size as usize]))
    }
}

// HWND 内部为裸指针；这里仅传递句柄值到工作线程（Win32 句柄跨线程合法）。
unsafe impl Send for Target {}

/// 可见的顶层窗口：(句柄, exe, 标题)（诊断用）。
pub fn visible_windows() -> Vec<(HWND, String)> {
    unsafe extern "system" fn proc(hwnd: HWND, lparam: windows::Win32::Foundation::LPARAM) -> windows::Win32::Foundation::BOOL {
        let out = &mut *(lparam.0 as *mut Vec<(HWND, String)>);
        if windows::Win32::UI::WindowsAndMessaging::IsWindowVisible(hwnd).as_bool() {
            out.push((hwnd, window_title(hwnd)));
        }
        true.into()
    }
    let mut out = Vec::new();
    unsafe {
        let _ = windows::Win32::UI::WindowsAndMessaging::EnumWindows(Some(proc), windows::Win32::Foundation::LPARAM(&mut out as *mut _ as isize));
    }
    out
}

/// 正在运行、有可见标题窗口的程序（按 exe 去重，设置界面"添加应用"用）。
/// 返回 (exe 绝对路径, 窗口标题)。
pub fn running_apps() -> Vec<(String, String)> {
    unsafe extern "system" fn proc(hwnd: HWND, lparam: windows::Win32::Foundation::LPARAM) -> windows::Win32::Foundation::BOOL {
        let out = &mut *(lparam.0 as *mut Vec<(String, String)>);
        use windows::Win32::UI::WindowsAndMessaging::{GetWindow, IsWindowVisible, GW_OWNER, GetWindowLongPtrW, GWL_EXSTYLE, WS_EX_TOOLWINDOW};
        if !IsWindowVisible(hwnd).as_bool()
            || GetWindow(hwnd, GW_OWNER).is_ok_and(|h| !h.0.is_null())
            || GetWindowLongPtrW(hwnd, GWL_EXSTYLE) & WS_EX_TOOLWINDOW.0 as isize != 0
        {
            return true.into();
        }
        let title = window_title(hwnd);
        if title.trim().is_empty() {
            return true.into();
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == std::process::id() {
            return true.into();
        }
        if let Some(exe) = exe_of(pid) {
            if !out.iter().any(|(e, _)| e.eq_ignore_ascii_case(&exe)) {
                out.push((exe, title));
            }
        }
        true.into()
    }
    let mut out: Vec<(String, String)> = Vec::new();
    unsafe {
        let _ = windows::Win32::UI::WindowsAndMessaging::EnumWindows(
            Some(proc),
            windows::Win32::Foundation::LPARAM(&mut out as *mut _ as isize),
        );
    }
    out
}

/// 前台窗口是否真全屏（游戏、视频、演示）——全屏自动暂停用（FR-13）。
///
/// 条件：没有标题栏 + 完整覆盖所在显示器 + 不是桌面/外壳窗口。
/// 只看覆盖面积是不行的：普通最大化窗口含不可见边框后覆盖率约 96%，
/// 会被误判成全屏而让手势整体失效（右键直接弹菜单）。
pub fn is_foreground_fullscreen() -> bool {
    is_fullscreen_window(unsafe { windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow() })
}

/// 见 [`is_foreground_fullscreen`]。
pub fn is_fullscreen_window(hwnd: HWND) -> bool {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST};
    use windows::Win32::UI::WindowsAndMessaging::{GetWindowLongPtrW, GetWindowRect, GWL_STYLE, WS_CAPTION};
    unsafe {
        if hwnd.0.is_null() {
            return false;
        }
        let class = class_name(hwnd);
        if matches!(class.as_str(), "Progman" | "WorkerW") || SHELL_UI_CLASSES.contains(&class.as_str()) {
            return false;
        }
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
        if style & WS_CAPTION.0 == WS_CAPTION.0 {
            return false; // 有标题栏：普通窗口（含最大化）
        }
        let mut rc = RECT::default();
        if GetWindowRect(hwnd, &mut rc).is_err() {
            return false;
        }
        let mut mi = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
        if !GetMonitorInfoW(MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST), &mut mi).as_bool() {
            return false;
        }
        let m = mi.rcMonitor;
        rc.left <= m.left && rc.top <= m.top && rc.right >= m.right && rc.bottom >= m.bottom
    }
}

/// 窗口标题。
pub fn window_title(hwnd: HWND) -> String {
    let mut buf = [0u16; 512];
    let n = unsafe {
        windows::Win32::UI::WindowsAndMessaging::GetWindowTextW(hwnd, &mut buf)
    };
    if n == 0 {
        return String::new();
    }
    String::from_utf16_lossy(&buf[..n as usize])
}

/// 是否有名为 `exe_name` 的进程在运行（不区分大小写，只比较文件名）。
pub fn process_running(exe_name: &str) -> bool {
    !pids_of(exe_name).is_empty()
}

/// 正在运行的同名进程的完整 exe 路径（取第一个能打开的）。
pub fn running_exe_path(exe_name: &str) -> Option<String> {
    pids_of(exe_name).into_iter().find_map(exe_of)
}

/// 名为 `exe_name` 的进程 id 列表。
fn pids_of(exe_name: &str) -> Vec<u32> {
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
    };
    let mut out = Vec::new();
    unsafe {
        let Ok(snap) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else { return out };
        let mut e = PROCESSENTRY32W { dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32, ..Default::default() };
        let mut ok = Process32FirstW(snap, &mut e).is_ok();
        while ok {
            let n = e.szExeFile.iter().position(|&c| c == 0).unwrap_or(e.szExeFile.len());
            if String::from_utf16_lossy(&e.szExeFile[..n]).eq_ignore_ascii_case(exe_name) {
                out.push(e.th32ProcessID);
            }
            ok = Process32NextW(snap, &mut e).is_ok();
        }
        let _ = CloseHandle(snap);
    }
    out
}

/// 请求指定程序的所有顶层窗口关闭（WM_CLOSE，让它自己收尾保存）。返回发出的窗口数。
pub fn close_app_windows(exe_name: &str) -> usize {
    use windows::Win32::UI::WindowsAndMessaging::{GetWindow, PostMessageW, GW_OWNER, WM_CLOSE};
    let mut n = 0;
    for hwnd in all_top_windows() {
        let mut pid = 0u32;
        unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
        let owned = unsafe { GetWindow(hwnd, GW_OWNER) }.is_ok_and(|h| !h.0.is_null());
        if owned || pid == 0 || pid == std::process::id() {
            continue;
        }
        let matches = exe_of(pid).is_some_and(|e| {
            e.rsplit(['\\', '/']).next().is_some_and(|f| f.eq_ignore_ascii_case(exe_name))
        });
        if matches && !window_title(hwnd).is_empty() {
            unsafe {
                let _ = PostMessageW(Some(hwnd), WM_CLOSE, windows::Win32::Foundation::WPARAM(0), windows::Win32::Foundation::LPARAM(0));
            }
            n += 1;
        }
    }
    n
}

/// 全部顶层窗口（含隐藏）。
fn all_top_windows() -> Vec<HWND> {
    unsafe extern "system" fn proc(hwnd: HWND, lparam: windows::Win32::Foundation::LPARAM) -> windows::Win32::Foundation::BOOL {
        let out = &mut *(lparam.0 as *mut Vec<HWND>);
        out.push(hwnd);
        true.into()
    }
    let mut out = Vec::new();
    unsafe {
        let _ = windows::Win32::UI::WindowsAndMessaging::EnumWindows(Some(proc), windows::Win32::Foundation::LPARAM(&mut out as *mut _ as isize));
    }
    out
}

/// 本程序的 exe 文件名（设置界面与引擎是同一个 exe）。
pub fn own_exe_name() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "mousetroke.exe".into())
}
