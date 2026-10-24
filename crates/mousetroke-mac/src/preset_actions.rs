//! 原生预设的平台能力。AX 和等待在工作线程，屏幕快照在主线程。
pub use crate::ax::preset_layout::{
    center, move_to_next_monitor, snap_left, snap_right, WindowTarget,
};
pub use crate::helpers::Window as GestureWindow;
pub use crate::input::preset_text::{
    copy_plain, type_text_direct, type_text_via_paste, type_text_via_paste_with_delay,
};
pub use mousetroke_core::action_geometry::Monitor;

mod datetime;
mod main_thread;
mod power;
pub use datetime::local_now;
pub(crate) use main_thread::on_main;
pub use power::{display_off, sleep};

use mousetroke_core::presets::native::{LayoutOp, NativeAction, TextMethod};

pub(crate) fn execute(
    action: NativeAction,
    target: Option<mousetroke_engine::Target>,
    window: Option<GestureWindow>,
    pos: (i32, i32),
    monitors: Vec<Monitor>,
) -> Result<(), String> {
    match action {
        NativeAction::Layout(op) => {
            let target = target.ok_or("Gesture target window is missing")?;
            let w = WindowTarget {
                pid: target.pid,
                window: window.as_ref(),
                pos,
                monitors: &monitors,
            };
            match op {
                LayoutOp::SnapLeft => snap_left(w),
                LayoutOp::SnapRight => snap_right(w),
                LayoutOp::Center => center(w),
                LayoutOp::NextMonitor => move_to_next_monitor(w),
            }
        }
        NativeAction::TypeText { ref text, method } => type_text(text, method),
        NativeAction::DateTime { method, .. } => {
            type_text(&action.formatted_text(local_now()?)?, method)
        }
        NativeAction::CopyPlain => copy_plain(),
        NativeAction::DisplayOff => display_off(),
        NativeAction::Sleep => sleep(),
        NativeAction::TogglePause => on_main(|| {
            crate::app::request_toggle_pause();
            crate::app::wake();
            Ok(())
        }),
        NativeAction::Settings => crate::settings::open_checked(),
    }
}

fn type_text(text: &str, method: TextMethod) -> Result<(), String> {
    match method {
        TextMethod::Paste => type_text_via_paste(text),
        TextMethod::Type => type_text_direct(text),
    }
}

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
