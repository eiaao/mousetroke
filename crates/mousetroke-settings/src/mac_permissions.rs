//! 设置与引擎共用已签名的 bundle 和二进制；TCC 授权身份相同，不向开发用辅助程序授予权限。
use crate::{keys, Msg, MsgResult};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct PermissionStatus {
    accessibility: bool,
    input_monitoring: bool,
    engine_ready: Option<bool>,
}

#[tauri::command(async)]
pub fn get_permissions() -> PermissionStatus {
    let engine_ready = mousetroke_mac::instance::input_state().map(|(_, active)| active);
    // 该同步函数作为异步命令在工作线程执行；短命探针让开着的引导也能看到新授权。
    let status = if engine_ready == Some(true) {
        mousetroke_mac::permissions::check()
    } else {
        mousetroke_mac::permissions::check_fresh().unwrap_or_else(mousetroke_mac::permissions::check)
    };
    // CoreGraphics 可能缓存当前进程的拒绝结果；引擎实际装好 tap 也是授权已生效的可靠证据。
    PermissionStatus {
        accessibility: status.accessibility || engine_ready == Some(true),
        input_monitoring: status.input_monitoring || engine_ready == Some(true),
        engine_ready,
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    Accessibility,
    InputMonitoring,
}

/// 同步 Tauri 命令在主线程执行；只请求用户点击的那一项，随后打开对应系统面板。
#[tauri::command]
pub fn request_permission(permission: Permission) -> MsgResult<()> {
    let accessibility = matches!(permission, Permission::Accessibility);
    // 有引擎时由真正持有事件 tap 的进程请求并打开面板；仅无引擎时本地回退。
    let result = if mousetroke_mac::instance::engine_running() {
        mousetroke_mac::instance::request_permission(accessibility).map_err(|error| error.to_string())
    } else {
        mousetroke_mac::permissions::request(accessibility)
    };
    result.map_err(|_| Msg::new(keys::MAC_SETTINGS_OPEN_FAILED, &[]))
}

#[tauri::command]
pub fn restart_engine() -> MsgResult<()> {
    mousetroke_mac::instance::request_restart()
        .map_err(|error| Msg::new(keys::ERROR_START_ENGINE, &[("err", error.to_string())]))
}
