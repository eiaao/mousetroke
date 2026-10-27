//! 原生预设沿用执行器锁定的目标；阻塞能力只在动作工作线程调用。
use crate::{preset_datetime, preset_layout, preset_power, preset_text, targeting::Target};
use mousetroke_core::presets::native::{LayoutOp, NativeAction, TextMethod};
use std::time::Duration;

pub(crate) fn execute(action: NativeAction, target: Option<&Target>) -> Result<(), String> {
    match action {
        NativeAction::Layout(op) => {
            let target = target.ok_or("Gesture target window is missing")?;
            if !mousetroke_core::action_geometry::layout_target_allowed(mousetroke_core::caps::Platform::Windows, target.is_desktop, &target.class, 0) {
                crate::log!("跳过桌面或外壳窗口的布局动作");
                return Ok(());
            }
            let hwnd = target.hwnd;
            match op {
                LayoutOp::SnapLeft => preset_layout::snap_left(hwnd),
                LayoutOp::SnapRight => preset_layout::snap_right(hwnd),
                LayoutOp::Center => preset_layout::center(hwnd),
                LayoutOp::NextMonitor => preset_layout::move_to_next_monitor(hwnd),
            }
        }
        NativeAction::TypeText { ref text, method, paste_delay_ms } => type_text(text, method, paste_delay_ms),
        NativeAction::DateTime { method, paste_delay_ms, .. } => type_text(
            &action.formatted_text(preset_datetime::local_now())?,
            method,
            paste_delay_ms,
        ),
        NativeAction::CopyPlain => preset_text::copy_plain(),
        NativeAction::DisplayOff => preset_power::display_off(),
        NativeAction::Sleep => preset_power::sleep(),
        NativeAction::TogglePause => {
            if crate::tray::post_engine_cmd(crate::tray::CMD_TOGGLE_PAUSE) {
                Ok(())
            } else {
                Err("Engine control window is unavailable".into())
            }
        }
        NativeAction::Settings => crate::tray::try_open_settings(),
    }
}

fn type_text(text: &str, method: TextMethod, paste_delay_ms: u32) -> Result<(), String> {
    match method {
        TextMethod::Paste => preset_text::type_text_via_paste_with_delay(text, Duration::from_millis(paste_delay_ms.into())),
        TextMethod::Type => preset_text::type_text_direct(text),
    }
}
