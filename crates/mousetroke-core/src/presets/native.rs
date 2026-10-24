//! 原生预设的执行目标；不写入配置，也不冒充高级动作。
use crate::{
    action_datetime::{self, LocalDateTime},
    schema::Action,
};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LayoutOp {
    SnapLeft,
    SnapRight,
    Center,
    NextMonitor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum NativePreset {
    Layout(LayoutOp),
    TypeText,
    DateTime,
    CopyPlain,
    DisplayOff,
    Sleep,
    TogglePause,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextMethod {
    Paste,
    Type,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NativeAction {
    Layout(LayoutOp),
    TypeText { text: String, method: TextMethod },
    DateTime { pattern: String, method: TextMethod },
    CopyPlain,
    DisplayOff,
    Sleep,
    TogglePause,
    Settings,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PreparedAction {
    Advanced(Action),
    Native(NativeAction),
}

impl NativeAction {
    pub fn id(&self) -> &'static str {
        match self {
            Self::Layout(LayoutOp::SnapLeft) => "window.snap_left",
            Self::Layout(LayoutOp::SnapRight) => "window.snap_right",
            Self::Layout(LayoutOp::Center) => "window.center",
            Self::Layout(LayoutOp::NextMonitor) => "window.next_monitor",
            Self::TypeText { .. } => "text.type",
            Self::DateTime { .. } => "text.datetime",
            Self::CopyPlain => "text.copy_plain",
            Self::DisplayOff => "system.display_off",
            Self::Sleep => "system.sleep",
            Self::TogglePause => "mousetroke.toggle_pause",
            Self::Settings => "mousetroke.settings",
        }
    }

    pub fn formatted_text(&self, now: LocalDateTime) -> Result<String, String> {
        match self {
            Self::DateTime { pattern, .. } => action_datetime::format_pattern(pattern, now),
            Self::TypeText { text, .. } => Ok(text.clone()),
            _ => Err("Not a text action".into()),
        }
    }
}
