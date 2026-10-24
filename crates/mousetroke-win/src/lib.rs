//! mousetroke-win —— Windows 平台层。
//!
//! - [`engine`]：共享手势引擎的 Windows 宿主适配
//! - [`hooks`]：WH_MOUSE_LL / WH_KEYBOARD_LL 安装与回调
//! - [`overlay`]：透明穿透轨迹覆盖层（分层窗口 + 软件光栅）
//! - [`input`]：SendInput 注入 / cmd / 窗口操作
//! - [`targeting`]：鼠标下窗口 → exe / 桌面识别
//! - [`brightness`]：显示器亮度（DDC/CI + WMI 回落）
//! - [`tray`]：托盘、暂停热键、配置热加载定时器
//!
//! 本 crate 刻意不依赖 WebView：常驻进程保持低占用。

#![cfg(windows)]

/// 供导出的 `log!` 宏引用，调用方无需直接依赖引擎 crate。
#[doc(hidden)]
pub use mousetroke_engine as __engine;

pub mod autostart;
pub mod brightness;
pub mod engine;
pub mod hooks;
pub mod input;
pub mod log;
pub mod lua;
pub mod overlay;
pub mod preset_datetime;
pub mod preset_layout;
pub mod preset_power;
pub mod preset_text;
pub mod targeting;
pub mod tray;
pub mod update;

use mousetroke_core::schema::Config;

/// 启动手势引擎（阻塞直至消息循环退出）。`cfg_path` 给出时监视该文件并热加载。
pub fn run(cfg: Config, cfg_path: Option<std::path::PathBuf>) -> Result<(), String> {
    hooks::run(cfg, cfg_path)
}

pub use hooks::engine_running;

/// GUI 子系统下挂接父进程控制台（从终端启动时 CLI 输出可见；双击/自启时无操作）。
pub fn attach_parent_console() {
    use windows::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
    unsafe {
        let _ = AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

/// 当前用户的首选界面语言列表（如 `["zh-CN", "en-US"]`），供 [`mousetroke_core::i18n::resolve`]
/// 在 `language = auto` 时按顺序取第一个认识的语言。
pub fn system_langs() -> Vec<String> {
    use windows::core::PWSTR;
    use windows::Win32::Globalization::{GetUserPreferredUILanguages, MUI_LANGUAGE_NAME};
    unsafe {
        let mut n = 0u32;
        let mut len = 0u32;
        let _ = GetUserPreferredUILanguages(MUI_LANGUAGE_NAME, &mut n, Some(PWSTR::null()), &mut len);
        if len == 0 {
            return Vec::new();
        }
        let mut buf = vec![0u16; len as usize];
        if GetUserPreferredUILanguages(MUI_LANGUAGE_NAME, &mut n, Some(PWSTR(buf.as_mut_ptr())), &mut len).is_err() {
            return Vec::new();
        }
        // 空字符串分隔、双空结尾
        buf[..len as usize]
            .split(|&c| c == 0)
            .filter(|s| !s.is_empty())
            .map(String::from_utf16_lossy)
            .collect()
    }
}
