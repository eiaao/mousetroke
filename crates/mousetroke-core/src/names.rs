//! 旧格式默认手势 / 作用域名 ↔ 内置名称词条键（`builtin.gesture.*`、`builtin.scope.*`）的对照。
//!
//! - **导入**（[`crate::legacy`]）：识别到旧格式默认英文名 → 写入 `name_key`，
//!   `name` 填当前语言的译文（[`localize_builtin`]）。
//! - **加载已有配置**（[`crate::schema::Config::migrate`] → [`attach_name_keys`]）：
//!   只补 `name_key`，`name` 原样不动。早期版本曾把默认名直接改写成中文存进配置，
//!   英文名与改写后的中文名在这里都认得回来，幂等。
//! - 用户自己改过的名字不在对照表里，不受影响。

use crate::i18n::{keys, try_t};
use crate::schema::{Action, Config};

/// 作用域名（旧格式英文默认名，或曾被改写进配置的中文名）→ 词条键。
fn scope_key(name: &str) -> Option<&'static str> {
    Some(match name {
        "General" | "所有程序" => keys::BUILTIN_SCOPE_GLOBAL,
        "Windows Explorer" | "资源管理器" => keys::BUILTIN_SCOPE_EXPLORER,
        "Cmd" | "命令提示符" => keys::BUILTIN_SCOPE_CMD,
        "Desktop" | "桌面" => keys::BUILTIN_SCOPE_DESKTOP,
        _ => return None,
    })
}

/// 手势名。同名但含义随场景不同的（New）按动作区分。
fn binding_key(name: &str, action: &Action) -> Option<&'static str> {
    Some(match name {
        "Copy" | "复制" => keys::BUILTIN_GESTURE_COPY,
        "Paste" | "粘贴" => keys::BUILTIN_GESTURE_PASTE,
        "Cut" | "剪切" => keys::BUILTIN_GESTURE_CUT,
        "WG Settings" | "手势设置" => keys::BUILTIN_GESTURE_SETTINGS,
        "Web Search" | "搜索选中文字" => keys::BUILTIN_GESTURE_WEB_SEARCH,
        "Enter" | "回车" => keys::BUILTIN_GESTURE_ENTER,
        "Paste & Enter" | "粘贴并回车" => keys::BUILTIN_GESTURE_PASTE_ENTER,
        "Forward" | "前进" => keys::BUILTIN_GESTURE_FORWARD,
        "Back" | "后退" => keys::BUILTIN_GESTURE_BACK,
        // 旧格式的 New 按动作区分；已改写过的中文名各归各位
        "New" => match action {
            Action::Shell { .. } => keys::BUILTIN_GESTURE_NEW_CMD,
            Action::KeySequence { keys, .. } if keys.contains('n') => keys::BUILTIN_GESTURE_NEW_WINDOW,
            _ => keys::BUILTIN_GESTURE_NEW_TAB,
        },
        "新开命令行窗口" => keys::BUILTIN_GESTURE_NEW_CMD,
        "新建窗口" => keys::BUILTIN_GESTURE_NEW_WINDOW,
        "新建标签页" => keys::BUILTIN_GESTURE_NEW_TAB,
        "Reopen Tab" | "恢复关闭的标签页" => keys::BUILTIN_GESTURE_REOPEN_TAB,
        "Refresh" | "刷新" => keys::BUILTIN_GESTURE_REFRESH,
        "Close" | "关闭标签页" => keys::BUILTIN_GESTURE_CLOSE_TAB,
        "Quit" | "关闭窗口" => keys::BUILTIN_GESTURE_QUIT,
        "Kill" | "强制结束程序" => keys::BUILTIN_GESTURE_KILL,
        "Start" | "开始菜单" => keys::BUILTIN_GESTURE_START,
        "Show Desktop" | "显示桌面" => keys::BUILTIN_GESTURE_SHOW_DESKTOP,
        "Switch App" | "切换程序" => keys::BUILTIN_GESTURE_SWITCH_APP,
        "Task Switcher" | "任务切换器" => keys::BUILTIN_GESTURE_TASK_SWITCHER,
        "Play/Pause" | "播放/暂停" => keys::BUILTIN_GESTURE_PLAY_PAUSE,
        "Volume +" | "音量加" => keys::BUILTIN_GESTURE_VOLUME_UP,
        "Volume -" | "音量减" => keys::BUILTIN_GESTURE_VOLUME_DOWN,
        "Brighter" | "调亮屏幕" => keys::BUILTIN_GESTURE_BRIGHTER,
        "Darker" | "调暗屏幕" => keys::BUILTIN_GESTURE_DARKER,
        "Mute" | "静音" => keys::BUILTIN_GESTURE_MUTE,
        "Prev Track" | "上一曲" => keys::BUILTIN_GESTURE_PREV_TRACK,
        "Next Track" | "下一曲" => keys::BUILTIN_GESTURE_NEXT_TRACK,
        "Task Manager" | "任务管理器" => keys::BUILTIN_GESTURE_TASK_MANAGER,
        "ESC" | "取消（Esc）" => keys::BUILTIN_GESTURE_ESC,
        "Minimize" | "最小化" => keys::BUILTIN_GESTURE_MINIMIZE,
        "Maximize" | "最大化/还原" => keys::BUILTIN_GESTURE_MAXIMIZE,
        "Fullscreen" | "全屏" => keys::BUILTIN_GESTURE_FULLSCREEN,
        "Topmost" | "窗口置顶" => keys::BUILTIN_GESTURE_TOPMOST,
        "Prev Tab" | "上一个标签页" => keys::BUILTIN_GESTURE_PREV_TAB,
        "Next Tab" | "下一个标签页" => keys::BUILTIN_GESTURE_NEXT_TAB,
        "Home" | "回到顶部" => keys::BUILTIN_GESTURE_HOME,
        "End" | "跳到底部" => keys::BUILTIN_GESTURE_END,
        "System Settings" | "系统设置" => keys::BUILTIN_GESTURE_SYSTEM_SETTINGS,
        "Hibernate" | "休眠" => keys::BUILTIN_GESTURE_HIBERNATE,
        "Zoom In" | "放大" => keys::BUILTIN_GESTURE_ZOOM_IN,
        "Zoom Out" | "缩小" => keys::BUILTIN_GESTURE_ZOOM_OUT,
        "Zoom Reset" | "重置缩放" => keys::BUILTIN_GESTURE_ZOOM_RESET,
        "Backspace" | "退格" => keys::BUILTIN_GESTURE_BACKSPACE,
        "Delete" | "删除" => keys::BUILTIN_GESTURE_DELETE,
        "Snip & Sketch" | "截图" => keys::BUILTIN_GESTURE_SNIP,
        "Next Desktop" | "下一个虚拟桌面" => keys::BUILTIN_GESTURE_NEXT_DESKTOP,
        "Prev Desktop" | "上一个虚拟桌面" => keys::BUILTIN_GESTURE_PREV_DESKTOP,
        "Rename" | "重命名" => keys::BUILTIN_GESTURE_RENAME,
        "Open In New Win" | "在新窗口中打开" => keys::BUILTIN_GESTURE_OPEN_IN_NEW_WIN,
        "Ctrl Drag" | "按住 Ctrl 拖动" => keys::BUILTIN_GESTURE_CTRL_DRAG,
        _ => return None,
    })
}

/// 加载配置时补 `name_key`（幂等；`name` 原样不动）。
pub fn attach_name_keys(cfg: &mut Config) {
    for scope in &mut cfg.scopes {
        if scope.name_key.is_none() {
            scope.name_key = scope_key(&scope.name).map(str::to_string);
        }
        for b in &mut scope.bindings {
            if b.name_key.is_none() {
                b.name_key = binding_key(&b.name, &b.action).map(str::to_string);
            }
        }
    }
}

/// 导入时把内置名刷新为当前语言的译文（`name_key` 需已由 [`attach_name_keys`] 补好）。
pub fn localize_builtin(cfg: &mut Config) {
    for scope in &mut cfg.scopes {
        if let Some(text) = scope.name_key.as_deref().and_then(try_t) {
            scope.name = text.into();
        }
        for b in &mut scope.bindings {
            if let Some(text) = b.name_key.as_deref().and_then(try_t) {
                b.name = text.into();
            }
        }
    }
}
