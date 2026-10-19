//! 极小的日志转发接口；未注册时丢弃，不直接访问控制台或磁盘。

use std::sync::OnceLock;

static SINK: OnceLock<fn(&str)> = OnceLock::new();

/// 平台在安装钩子前注册一次；后续注册无效果。接收器在调用线程同步执行，
/// 须遵守输入回调约束，格式化、时间戳与落盘策略由平台保持。
pub fn set_sink(sink: fn(&str)) {
    let _ = SINK.set(sink);
}

/// 转发已格式化的原始消息，不添加前缀、换行或时间戳。
pub fn write(msg: &str) {
    if let Some(sink) = SINK.get() {
        sink(msg);
    }
}

/// 保留平台日志原有的格式化与分配次数。
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        $crate::log::write(&format!($($arg)*))
    };
}
