//! 独立窗口布局能力；目标沿用 WindowOp 的 HWND，不查当前焦点，也不发送系统快捷键。
use mousetroke_core::action_geometry::{self as geometry, FrameInsets, Monitor, Rect};
use windows::core::BOOL;
use windows::Win32::{
    Foundation::{HWND, LPARAM, RECT},
    Graphics::{
        Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS},
        Gdi::{
            EnumDisplayMonitors, GetMonitorInfoW, MonitorFromWindow, HDC, HMONITOR, MONITORINFO,
            MONITOR_DEFAULTTONEAREST,
        },
    },
    UI::{
        HiDpi::{
            SetThreadDpiAwarenessContext, DPI_AWARENESS_CONTEXT,
            DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        },
        WindowsAndMessaging::{
            GetWindowRect, IsIconic, IsWindow, IsZoomed, SetWindowPos, ShowWindow, SWP_NOACTIVATE,
            SWP_NOZORDER, SW_MAXIMIZE, SW_RESTORE,
        },
    },
};

struct DpiContext(DPI_AWARENESS_CONTEXT);
impl DpiContext {
    fn enter() -> Result<Self, String> {
        // DWM 始终返回物理像素；临时切换线程，避免 GetWindowRect 被 DPI 虚拟化。
        let previous =
            unsafe { SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
        if previous.0.is_null() {
            Err(format!(
                "Set layout DPI context: {}",
                windows::core::Error::from_thread()
            ))
        } else {
            Ok(Self(previous))
        }
    }
}
impl Drop for DpiContext {
    fn drop(&mut self) {
        unsafe {
            SetThreadDpiAwarenessContext(self.0);
        }
    }
}

fn rect(r: RECT) -> Rect {
    Rect {
        x: r.left as f64,
        y: r.top as f64,
        width: r.right as f64 - r.left as f64,
        height: r.bottom as f64 - r.top as f64,
    }
}

fn monitor(handle: HMONITOR) -> Result<Monitor, String> {
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    unsafe { GetMonitorInfoW(handle, &mut info) }
        .ok()
        .map_err(|e| format!("Read monitor work area: {e}"))?;
    let result = Monitor {
        bounds: rect(info.rcMonitor),
        work_area: rect(info.rcWork),
    };
    if result.bounds.valid() && result.work_area.valid() {
        Ok(result)
    } else {
        Err("Invalid monitor work area".into())
    }
}

fn monitors() -> Result<Vec<(HMONITOR, Monitor)>, String> {
    struct Context {
        rows: Vec<(HMONITOR, Monitor)>,
        error: Option<String>,
    }
    unsafe extern "system" fn collect(
        handle: HMONITOR,
        _: HDC,
        _: *mut RECT,
        data: LPARAM,
    ) -> BOOL {
        // 安全：EnumDisplayMonitors 同步调用，data 指向本次调用的独占上下文。
        let context = unsafe { &mut *(data.0 as *mut Context) };
        match monitor(handle) {
            Ok(info) => {
                context.rows.push((handle, info));
                BOOL(1)
            }
            Err(error) => {
                context.error = Some(error);
                BOOL(0)
            }
        }
    }
    let mut context = Context {
        rows: Vec::new(),
        error: None,
    };
    let status = unsafe {
        EnumDisplayMonitors(
            None,
            None,
            Some(collect),
            LPARAM((&mut context as *mut Context) as isize),
        )
    };
    if let Some(error) = context.error {
        return Err(error);
    }
    status
        .ok()
        .map_err(|e| format!("Enumerate monitors: {e}"))?;
    if context.rows.is_empty() {
        Err("No monitors available".into())
    } else {
        Ok(context.rows)
    }
}

fn frame(hwnd: HWND) -> Result<(Rect, FrameInsets), String> {
    let mut outer = RECT::default();
    unsafe { GetWindowRect(hwnd, &mut outer) }
        .map_err(|e| format!("Read target window bounds: {e}"))?;
    let mut visible = RECT::default();
    let outer = rect(outer);
    let has_dwm = unsafe {
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            (&mut visible as *mut RECT).cast(),
            std::mem::size_of::<RECT>() as u32,
        )
    }
    .is_ok();
    let visible = if has_dwm && rect(visible).valid() {
        rect(visible)
    } else {
        outer
    };
    let insets = FrameInsets::between(outer, visible).ok_or("Invalid target window frame")?;
    Ok((visible, insets))
}

fn validate(hwnd: HWND) -> Result<(), String> {
    if unsafe { IsWindow(Some(hwnd)) }.as_bool() {
        Ok(())
    } else {
        Err("Gesture target window is no longer valid".into())
    }
}

fn restore(hwnd: HWND) -> Result<(), String> {
    if unsafe { IsZoomed(hwnd).as_bool() || IsIconic(hwnd).as_bool() } {
        unsafe {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }
        if unsafe { IsZoomed(hwnd).as_bool() || IsIconic(hwnd).as_bool() } {
            return Err("Target window could not be restored".into());
        }
    }
    Ok(())
}

fn pixels(r: Rect) -> Result<(i32, i32, i32, i32), String> {
    let edges = [
        r.x.round(),
        r.y.round(),
        r.right().round(),
        r.bottom().round(),
    ];
    if !r.valid()
        || edges
            .iter()
            .any(|&e| e < i32::MIN as f64 || e > i32::MAX as f64)
    {
        return Err("Window layout exceeds system coordinate range".into());
    }
    let [x, y, right, bottom] = edges.map(|v| v as i32);
    let width = right
        .checked_sub(x)
        .filter(|&v| v > 0)
        .ok_or("Invalid layout width")?;
    let height = bottom
        .checked_sub(y)
        .filter(|&v| v > 0)
        .ok_or("Invalid layout height")?;
    Ok((x, y, width, height))
}

fn place(hwnd: HWND, visible: Rect) -> Result<(), String> {
    // 跨 DPI 屏幕后边框可能改变；第二次按新边框修正可见边缘。
    for _ in 0..2 {
        let (current, insets) = frame(hwnd)?;
        if pixels(current)? == pixels(visible)? {
            return Ok(());
        }
        let outer = insets
            .outer_for_visible(visible)
            .ok_or("Invalid DWM frame compensation")?;
        let (x, y, width, height) = pixels(outer)?;
        unsafe {
            SetWindowPos(
                hwnd,
                None,
                x,
                y,
                width,
                height,
                SWP_NOACTIVATE | SWP_NOZORDER,
            )
        }
        .map_err(|e| format!("Set target window layout: {e}"))?;
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    if pixels(frame(hwnd)?.0)? == pixels(visible)? {
        Ok(())
    } else {
        Err("Target window rejected layout (minimum size or frame restrictions)".into())
    }
}

fn windows_target_class(hwnd: HWND) -> String {
    let mut name = [0u16; 256];
    let n = unsafe { windows::Win32::UI::WindowsAndMessaging::GetClassNameW(hwnd, &mut name) };
    String::from_utf16_lossy(&name[..n as usize])
}

fn excluded(class: String) -> bool {
    let excluded = !geometry::layout_target_allowed(mousetroke_core::caps::Platform::Windows, false, &class, 0);
    if excluded { crate::log!("跳过桌面或外壳窗口的布局动作"); }
    excluded
}

fn layout(hwnd: HWND, target: impl FnOnce(Rect, Rect) -> Option<Rect>) -> Result<(), String> {
    if excluded(windows_target_class(hwnd)) { return Ok(()); }
    let _dpi = DpiContext::enter()?;
    validate(hwnd)?;
    let work = monitor(unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) })?.work_area;
    restore(hwnd)?;
    let (visible, _) = frame(hwnd)?;
    place(
        hwnd,
        target(visible, work).ok_or("Cannot calculate target layout")?,
    )
}

pub fn snap_left(w: HWND) -> Result<(), String> {
    layout(w, |_, work| geometry::halves(work).map(|(left, _)| left))
}

pub fn snap_right(w: HWND) -> Result<(), String> {
    layout(w, |_, work| geometry::halves(work).map(|(_, right)| right))
}

pub fn center(w: HWND) -> Result<(), String> {
    layout(w, geometry::centered)
}

pub fn move_to_next_monitor(w: HWND) -> Result<(), String> {
    if excluded(windows_target_class(w)) { return Ok(()); }
    let _dpi = DpiContext::enter()?;
    validate(w)?;
    let rows = monitors()?;
    let current_handle = unsafe { MonitorFromWindow(w, MONITOR_DEFAULTTONEAREST) };
    let current = rows
        .iter()
        .position(|(handle, _)| *handle == current_handle)
        .ok_or("Target monitor disconnected")?;
    let geometry: Vec<_> = rows.iter().map(|(_, info)| *info).collect();
    let Some(next) = geometry::next_monitor(&geometry, current) else {
        return Ok(());
    };
    let maximized = unsafe { IsZoomed(w).as_bool() };
    restore(w)?;
    let result = (|| {
        let visible = frame(w)?.0;
        let target =
            geometry::proportional(visible, rows[current].1.work_area, rows[next].1.work_area)
                .ok_or("Cannot calculate cross-monitor layout")?;
        place(w, target)
    })();
    if maximized {
        unsafe {
            let _ = ShowWindow(w, SW_MAXIMIZE);
        }
        if result.is_ok()
            && (!unsafe { IsZoomed(w).as_bool() }
                || unsafe { MonitorFromWindow(w, MONITOR_DEFAULTTONEAREST) } != rows[next].0)
        {
            return Err("Target window could not be maximized on the next monitor".into());
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pixel_conversion_rejects_overflow_and_rounds_edges_together() {
        assert_eq!(
            pixels(Rect {
                x: -10.5,
                y: 20.4,
                width: 100.2,
                height: 50.2
            })
            .unwrap(),
            (-11, 20, 101, 51)
        );
        assert!(pixels(Rect {
            x: i32::MAX as f64,
            y: 0.0,
            width: 10.0,
            height: 10.0
        })
        .is_err());
        assert!(snap_left(HWND::default()).is_err());
    }

    #[test]
    #[ignore] // 单独运行后 3 秒内聚焦普通窗口；会改变该窗口布局。
    fn desktop_layout() {
        std::thread::sleep(std::time::Duration::from_secs(3));
        let w = unsafe { windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow() };
        snap_left(w).unwrap();
        snap_right(w).unwrap();
        center(w).unwrap();
        move_to_next_monitor(w).unwrap();
    }
}
