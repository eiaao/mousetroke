//! 尚未接入预设注册表的平台能力。AX 和等待在工作线程，屏幕快照在主线程。
pub use crate::ax::preset_layout::{
    center, move_to_next_monitor, snap_left, snap_right, WindowTarget,
};
pub use crate::helpers::Window as GestureWindow;
pub use crate::input::preset_text::{
    type_text_direct, type_text_via_paste, type_text_via_paste_with_delay,
};
pub use mousetroke_core::action_geometry::Monitor;

mod datetime;
mod main_thread;
mod power;
pub use datetime::local_now;
pub(crate) use main_thread::on_main;
pub use power::{display_off, sleep};

/// 将此快照与 WindowOp 已有的 pid/window/pos 一起交给 AX 工作线程。
pub fn monitors(mtm: objc2::MainThreadMarker) -> Result<Vec<Monitor>, String> {
    use mousetroke_core::action_geometry::{from_bottom_left, Rect};
    use objc2_app_kit::NSScreen;
    objc2::rc::autoreleasepool(|_| {
        let screens = NSScreen::screens(mtm);
        let primary = screens.firstObject().ok_or("No screens available")?;
        let primary_top = primary.frame().origin.y + primary.frame().size.height;
        let rect = |r: objc2_foundation::NSRect| {
            from_bottom_left(
                Rect {
                    x: r.origin.x,
                    y: r.origin.y,
                    width: r.size.width,
                    height: r.size.height,
                },
                primary_top,
            )
            .ok_or_else(|| "Invalid screen coordinates".to_string())
        };
        screens
            .iter()
            .map(|screen| {
                Ok(Monitor {
                    bounds: rect(screen.frame())?,
                    work_area: rect(screen.visibleFrame())?,
                })
            })
            .collect()
    })
}
