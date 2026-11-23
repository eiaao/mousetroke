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

/// 趣味特效（全屏动画，不操作任何窗口）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FunEffect {
    Fireworks,
    Confetti,
    Bomb,
    MoneyRain,
    Hearts,
    BlackHole,
    Snow,
    Crack,
    Meteors,
}

impl FunEffect {
    pub const ALL: [FunEffect; 9] = [
        Self::Fireworks, Self::Confetti, Self::Bomb, Self::MoneyRain, Self::Hearts,
        Self::BlackHole, Self::Snow, Self::Crack, Self::Meteors,
    ];

    /// 特效标识，与 `mousetroke-fx` 的名称一致。
    pub fn name(self) -> &'static str {
        match self {
            Self::Fireworks => "fireworks",
            Self::Confetti => "confetti",
            Self::Bomb => "bomb",
            Self::MoneyRain => "money_rain",
            Self::Hearts => "hearts",
            Self::BlackHole => "black_hole",
            Self::Snow => "snow",
            Self::Crack => "crack",
            Self::Meteors => "meteors",
        }
    }

    /// 由预置动作 id 反查。
    pub fn from_preset_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|effect| effect.preset_id() == id)
    }

    /// 对应的预置动作 id。
    pub fn preset_id(self) -> &'static str {
        match self {
            Self::Fireworks => "fun.fireworks",
            Self::Confetti => "fun.confetti",
            Self::Bomb => "fun.bomb",
            Self::MoneyRain => "fun.money_rain",
            Self::Hearts => "fun.hearts",
            Self::BlackHole => "fun.black_hole",
            Self::Snow => "fun.snow",
            Self::Crack => "fun.crack",
            Self::Meteors => "fun.meteors",
        }
    }
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
    Cheatsheet,
    Effect(FunEffect),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextMethod {
    Paste,
    Type,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NativeAction {
    Layout(LayoutOp),
    TypeText { text: String, method: TextMethod, paste_delay_ms: u32 },
    DateTime { pattern: String, method: TextMethod, paste_delay_ms: u32 },
    CopyPlain,
    DisplayOff,
    Sleep,
    TogglePause,
    Settings,
    /// 引擎执行前已按当前目标展开为手势菜单（见 [`crate::cheatsheet`]），平台不会收到。
    Cheatsheet,
    /// 在鼠标所在的显示器上播放全屏特效。
    Effect(FunEffect),
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
            Self::Cheatsheet => "mousetroke.cheatsheet",
            Self::Effect(effect) => effect.preset_id(),
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
