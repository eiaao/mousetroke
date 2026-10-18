//! 开机自启动（HKCU\Software\Microsoft\Windows\CurrentVersion\Run）。

use windows::core::PCWSTR;
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_SAM_FLAGS, REG_VALUE_TYPE, REG_SZ,
};

const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const VALUE_NAME: &str = "Mousetroke";

fn open_run_key(access: REG_SAM_FLAGS) -> Result<HKEY, String> {
    let mut wide: Vec<u16> = RUN_KEY.encode_utf16().chain([0]).collect();
    let mut hkey = HKEY::default();
    let err = unsafe {
        RegOpenKeyExW(HKEY_CURRENT_USER, PCWSTR(wide.as_mut_ptr()), Some(0), access, &mut hkey)
    };
    if err.0 != 0 {
        return Err(mousetroke_core::i18n::tf(mousetroke_core::i18n::keys::ERROR_AUTOSTART_OPEN, &[("err", &err.0.to_string())]));
    }
    Ok(hkey)
}

fn exe_path() -> Result<String, String> {
    std::env::current_exe()
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(|e| e.to_string())
}

pub fn is_enabled() -> bool {
    let Ok(hkey) = open_run_key(KEY_READ) else {
        return false;
    };
    let mut name: Vec<u16> = VALUE_NAME.encode_utf16().chain([0]).collect();
    let mut ty = REG_VALUE_TYPE(0);
    let mut cb = 0u32;
    let err = unsafe {
        RegQueryValueExW(
            hkey,
            PCWSTR(name.as_mut_ptr()),
            None,
            Some(&mut ty),
            None,
            Some(&mut cb),
        )
    };
    let _ = unsafe { RegCloseKey(hkey) };
    err.0 == 0 && cb > 0
}

/// 以当前进程 exe 设置自启动。
pub fn set(enabled: bool) -> Result<bool, String> {
    set_for(enabled, &exe_path()?)
}

/// 以指定 exe 设置自启动（设置程序替引擎注册时用）。
pub fn set_for(enabled: bool, exe: &str) -> Result<bool, String> {
    let hkey = open_run_key(KEY_WRITE)?;
    let mut name: Vec<u16> = VALUE_NAME.encode_utf16().chain([0]).collect();
    let result = if enabled {
        let path = format!("\"{exe}\" run");
        let data: Vec<u8> = path.encode_utf16().chain([0]).flat_map(|c| c.to_le_bytes()).collect();
        let err = unsafe {
            RegSetValueExW(hkey, PCWSTR(name.as_mut_ptr()), None, REG_SZ, Some(&data))
        };
        if err.0 != 0 {
            Err(mousetroke_core::i18n::tf(mousetroke_core::i18n::keys::ERROR_AUTOSTART_WRITE, &[("err", &err.0.to_string())]))
        } else {
            Ok(true)
        }
    } else {
        let err = unsafe { RegDeleteValueW(hkey, PCWSTR(name.as_mut_ptr())) };
        if err.0 != 0 && err.0 != 2 {
            // 2 = 值不存在，视为已关闭
            Err(mousetroke_core::i18n::tf(mousetroke_core::i18n::keys::ERROR_AUTOSTART_DELETE, &[("err", &err.0.to_string())]))
        } else {
            Ok(false)
        }
    };
    let _ = unsafe { RegCloseKey(hkey) };
    result
}
