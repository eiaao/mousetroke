//! 作为 ax 子模块复用原 WindowOp 的目标解析和 CF 所有权，不改动现有动作。
use super::{Cf, Raw};
use crate::helpers::Window;
use mousetroke_core::action_geometry::{self as geometry, Monitor, Rect};
use std::{
    ffi::c_void,
    time::{Duration, Instant},
};

/// 与 ax::apply 相同的手势目标信息，另附主线程采集的 NSScreen 工作区快照。
/// window 必须为 Some；保留原上下文的 Option 类型，缺少快照时返回错误。
#[derive(Clone, Copy)]
pub struct WindowTarget<'a> {
    pub pid: u32,
    pub window: Option<&'a Window>,
    pub pos: (i32, i32),
    pub monitors: &'a [Monitor],
}

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXValueCreate(kind: u32, value: *const c_void) -> Raw;
}

fn frame(window: &Cf) -> Result<Rect, String> {
    let r = super::bounds(window).ok_or("Read target AX position/size failed")?;
    let result = Rect {
        x: r.x,
        y: r.y,
        width: r.w,
        height: r.h,
    };
    if result.valid() {
        Ok(result)
    } else {
        Err("Invalid target AX window size".into())
    }
}

fn resolve(w: WindowTarget<'_>) -> Result<Cf, String> {
    if objc2::MainThreadMarker::new().is_some() {
        return Err("AX layout must run on a worker thread".into());
    }
    // 布局必须有手势窗口快照，不能走原解析器的 focused-window 兜底。
    if w.window.is_none() {
        return Err("Gesture target window snapshot is required for layout".into());
    }
    if w.window.is_some_and(|window| window.pid != w.pid) {
        return Err("Gesture target PID/window mismatch".into());
    }
    let window = super::window(w.pid, w.window, w.pos)
        .ok_or("Gesture target AX window not found; check Accessibility permission")?;
    if window.boolean("AXFullScreen") == Some(true) {
        return Err("Exit the target window's full-screen mode first".into());
    }
    Ok(window)
}

fn work_area(w: WindowTarget<'_>, frame: Rect) -> Result<usize, String> {
    geometry::monitor_for_rect(w.monitors, frame)
        .ok_or_else(|| "No valid target screen work area".into())
}

fn same_frame(a: Rect, b: Rect) -> bool {
    (a.x - b.x).abs() <= 2.0
        && (a.y - b.y).abs() <= 2.0
        && (a.width - b.width).abs() <= 2.0
        && (a.height - b.height).abs() <= 2.0
}

fn zoomed(window: &Cf, work: Rect) -> Result<bool, String> {
    // AX 没有所有应用统一提供的缩放状态；优先使用应用属性，再沿用现有工作区匹配。
    Ok(window
        .boolean("AXZoomed")
        .unwrap_or(same_frame(frame(window)?, work)))
}

fn check_status(status: i32, operation: &str) -> Result<(), String> {
    if status == 0 {
        Ok(())
    } else {
        Err(format!("{operation}: AXError {status}"))
    }
}

fn zoom_button(window: &Cf) -> Result<(), String> {
    check_status(
        window
            .press_button("AXZoomButton")
            .ok_or("Target window has no AX zoom button")?,
        "Press window zoom button",
    )
}

fn restore(window: &Cf, work: Rect) -> Result<(), String> {
    if window.boolean("AXMinimized") == Some(true) {
        check_status(
            window.set_boolean("AXMinimized", false),
            "Restore minimized window",
        )?;
    }
    if zoomed(window, work)? {
        let before = frame(window)?;
        zoom_button(window)?;
        let deadline = Instant::now() + Duration::from_millis(500);
        while same_frame(frame(window)?, before) {
            if Instant::now() >= deadline {
                return Err("Timed out restoring target window".into());
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    Ok(())
}

fn set_value(window: &Cf, name: &str, kind: u32, components: [f64; 2]) -> Result<(), String> {
    // 安全：AX 的 CGPoint/CGSize 都由两个 64 位 CGFloat 构成；Create 返回 +1 引用。
    let value = unsafe { Cf::owned(AXValueCreate(kind, components.as_ptr().cast())) }
        .ok_or("Create AX geometry value failed")?;
    let key = Cf::key(name);
    check_status(
        unsafe { super::AXUIElementSetAttributeValue(window.raw(), key.raw(), value.raw()) },
        name,
    )
}

fn place(window: &Cf, target: Rect) -> Result<(), String> {
    if !target.valid() {
        return Err("Invalid target AX layout".into());
    }
    // 部分应用会按旧屏尺寸限制首次调整；移动后再应用一次尺寸与位置。
    for _ in 0..2 {
        set_value(window, "AXSize", 2, [target.width, target.height])?;
        set_value(window, "AXPosition", 1, [target.x, target.y])?;
    }
    let deadline = Instant::now() + Duration::from_millis(500);
    loop {
        if same_frame(frame(window)?, target) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(
                "Target application rejected layout (minimum size or AX restrictions)".into(),
            );
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn layout(
    w: WindowTarget<'_>,
    calculate: impl FnOnce(Rect, Rect) -> Option<Rect>,
) -> Result<(), String> {
    let window = resolve(w)?;
    let work = w.monitors[work_area(w, frame(&window)?)?].work_area;
    restore(&window, work)?;
    let target = calculate(frame(&window)?, work).ok_or("Cannot calculate target layout")?;
    place(&window, target)
}

pub fn snap_left(w: WindowTarget<'_>) -> Result<(), String> {
    layout(w, |_, work| geometry::halves(work).map(|(left, _)| left))
}
pub fn snap_right(w: WindowTarget<'_>) -> Result<(), String> {
    layout(w, |_, work| geometry::halves(work).map(|(_, right)| right))
}
pub fn center(w: WindowTarget<'_>) -> Result<(), String> {
    layout(w, geometry::centered)
}

pub fn move_to_next_monitor(w: WindowTarget<'_>) -> Result<(), String> {
    let window = resolve(w)?;
    let source = work_area(w, frame(&window)?)?;
    let Some(next) = geometry::next_monitor(w.monitors, source) else {
        return Ok(());
    };
    let source_work = w.monitors[source].work_area;
    let target_work = w.monitors[next].work_area;
    let maximized = zoomed(&window, source_work)?;
    restore(&window, source_work)?;
    let target = geometry::proportional(frame(&window)?, source_work, target_work)
        .ok_or("Cannot calculate cross-screen layout")?;
    place(&window, target)?;
    if maximized {
        zoom_button(&window)?;
        // 智能缩放不保证铺满工作区，补上明确的几何尺寸。
        place(&window, target_work)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fake_screens_use_window_overlap_and_stable_next_order() {
        let left = Rect {
            x: -1440.0,
            y: 180.0,
            width: 1440.0,
            height: 900.0,
        };
        let right = Rect {
            x: 0.0,
            y: 0.0,
            width: 1920.0,
            height: 1080.0,
        };
        let screens = [
            Monitor {
                bounds: right,
                work_area: Rect {
                    y: 25.0,
                    height: 1000.0,
                    ..right
                },
            },
            Monitor {
                bounds: left,
                work_area: Rect {
                    width: 1380.0,
                    ..left
                },
            },
        ];
        let w = WindowTarget {
            pid: 0,
            window: None,
            pos: (0, 0),
            monitors: &screens,
        };
        assert_eq!(work_area(w, left).unwrap(), 1);
        assert_eq!(geometry::next_monitor(&screens, 1), Some(0));
    }

    #[test]
    #[ignore] // 真实桌面：设置 TEST_PID、TEST_AREAS 和 TEST_LAYOUT（均加 MOUSESTROKE_ 前缀），单独运行。
    fn desktop_layout() {
        let pid: u32 = std::env::var("MOUSESTROKE_TEST_PID")
            .expect("Set target PID")
            .parse()
            .unwrap();
        // 每屏一行 x,y,w,h,work_x,work_y,work_w,work_h，用分号连接，单位为 AX 逻辑点。
        let areas = std::env::var("MOUSESTROKE_TEST_AREAS")
            .expect("Set monitor bounds and visible work areas");
        let monitors: Vec<_> = areas
            .split(';')
            .map(|record| {
                let values: Vec<f64> = record
                    .split(',')
                    .map(|value| value.trim().parse().unwrap())
                    .collect();
                assert_eq!(values.len(), 8);
                let bounds = Rect {
                    x: values[0],
                    y: values[1],
                    width: values[2],
                    height: values[3],
                };
                let work_area = Rect {
                    x: values[4],
                    y: values[5],
                    width: values[6],
                    height: values[7],
                };
                assert!(bounds.valid() && work_area.valid());
                Monitor { bounds, work_area }
            })
            .collect();
        let wid = std::env::var("MOUSESTROKE_TEST_WINDOW")
            .ok()
            .map(|value| value.parse::<usize>().unwrap());
        let window = crate::targeting::lua_windows(true)
            .into_iter()
            .find(|window| {
                window.pid == pid && window.layer == 0 && wid.is_none_or(|wid| window.wid == wid)
            })
            .expect("Open the target window first");
        let target = WindowTarget {
            pid,
            window: Some(&window),
            pos: (window.bounds.x as i32, window.bounds.y as i32),
            monitors: &monitors,
        };
        match std::env::var("MOUSESTROKE_TEST_LAYOUT")
            .as_deref()
            .unwrap_or("snap_left")
        {
            "snap_left" => snap_left(target),
            "snap_right" => snap_right(target),
            "center" => center(target),
            "next" => move_to_next_monitor(target),
            _ => panic!("Expected snap_left, snap_right, center or next"),
        }
        .unwrap();
    }
}
