//! 平台无关的手势状态机；系统查询与副作用通过 [`Host`] / [`Canvas`] 提供。
//! 引擎仅在安装输入钩子的线程使用，计时沿用标准库的单调时钟。

mod engine;
mod actions;
mod host;
pub mod log;
mod slot;

pub use engine::{push_toast, Engine, InputEvent, Reload, Verdict, PAUSED};
pub use actions::prepare_action;
pub use host::{Canvas, Host, Target};
pub use slot::{install, uninstall, with_engine};
