//! 设置与引擎共用已签名的 bundle 和二进制；TCC 授权身份相同，不向开发用辅助程序授予权限。
use crate::{keys, Msg, MsgResult};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct PermissionStatus {
    accessibility: bool,
    input_monitoring: bool,
    engine_ready: Option<bool>,
}

#[tauri::command]
pub fn get_permissions() -> PermissionStatus {
    let status = mousetroke_mac::permissions::check();
    let engine_ready = mousetroke_mac::instance::input_state().map(|(_, active)| active);
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
    mousetroke_mac::permissions::request(matches!(permission, Permission::Accessibility))
        .map_err(|_| Msg::new(keys::MAC_SETTINGS_OPEN_FAILED, &[]))
}

#[tauri::command]
pub fn restart_engine() -> MsgResult<()> {
    mousetroke_mac::instance::request_restart()
        .map_err(|error| Msg::new(keys::ERROR_START_ENGINE, &[("err", error.to_string())]))
}
