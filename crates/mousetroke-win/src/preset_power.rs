//! 电源能力只供后续预设执行器显式调用。
use windows::Win32::{
    Foundation::{CloseHandle, GetLastError, ERROR_NOT_ALL_ASSIGNED, HANDLE, LPARAM, LUID, WPARAM},
    Security::{
        AdjustTokenPrivileges, LookupPrivilegeValueW, LUID_AND_ATTRIBUTES, SE_PRIVILEGE_ENABLED,
        SE_SHUTDOWN_NAME, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TOKEN_QUERY,
    },
    System::Threading::{GetCurrentProcess, OpenProcessToken},
    UI::WindowsAndMessaging::{PostMessageW, HWND_BROADCAST, SC_MONITORPOWER, WM_SYSCOMMAND},
};

struct ShutdownPrivilege {
    token: HANDLE,
    previous: TOKEN_PRIVILEGES,
}

impl ShutdownPrivilege {
    fn enable() -> Result<Self, String> {
        let mut token = HANDLE::default();
        unsafe {
            OpenProcessToken(
                GetCurrentProcess(),
                TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
                &mut token,
            )
        }
        .map_err(|e| format!("Open power action token: {e}"))?;
        let mut privilege = Self {
            token,
            previous: TOKEN_PRIVILEGES::default(),
        };
        let mut luid = LUID::default();
        unsafe { LookupPrivilegeValueW(None, SE_SHUTDOWN_NAME, &mut luid) }
            .map_err(|e| format!("Find shutdown privilege: {e}"))?;
        let requested = TOKEN_PRIVILEGES {
            PrivilegeCount: 1,
            Privileges: [LUID_AND_ATTRIBUTES {
                Luid: luid,
                Attributes: SE_PRIVILEGE_ENABLED,
            }],
        };
        let mut size = 0;
        // 只临时启用已有权限；不提权、不弹 UAC，返回后恢复原 token 状态。
        unsafe {
            AdjustTokenPrivileges(
                token,
                false,
                Some(&requested),
                std::mem::size_of::<TOKEN_PRIVILEGES>() as u32,
                Some(&mut privilege.previous),
                Some(&mut size),
            )
        }
        .map_err(|e| format!("Enable shutdown privilege: {e}"))?;
        if unsafe { GetLastError() } == ERROR_NOT_ALL_ASSIGNED {
            return Err("The current account does not have shutdown privilege".into());
        }
        Ok(privilege)
    }
}

impl Drop for ShutdownPrivilege {
    fn drop(&mut self) {
        unsafe {
            if self.previous.PrivilegeCount != 0 {
                let _ =
                    AdjustTokenPrivileges(self.token, false, Some(&self.previous), 0, None, None);
            }
            let _ = CloseHandle(self.token);
        }
    }
}

pub fn display_off() -> Result<(), String> {
    // 广播投递立即返回，不同步等待其他应用的窗口过程。
    unsafe {
        PostMessageW(
            Some(HWND_BROADCAST),
            WM_SYSCOMMAND,
            WPARAM(SC_MONITORPOWER as usize),
            LPARAM(2),
        )
    }
    .map_err(|e| format!("Post display-off request: {e}"))
}

pub fn sleep() -> Result<(), String> {
    static SLEEP_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _lock = SLEEP_LOCK
        .lock()
        .map_err(|_| "Power action lock poisoned")?;
    let _privilege = ShutdownPrivilege::enable()?;
    // 工作区未启用 Win32_System_Power，直接链接同一 API，保持依赖清单不变。
    #[link(name = "powrprof")]
    unsafe extern "system" {
        fn SetSuspendState(hibernate: u8, force: u8, disable_wake_events: u8) -> u8;
    }
    if unsafe { SetSuspendState(0, 0, 0) } != 0 {
        Ok(())
    } else {
        Err(format!(
            "Request system sleep: {}",
            windows::core::Error::from_thread()
        ))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore] // 在真实桌面单独运行；显示器关闭后移动鼠标唤醒。
    fn desktop_display_off() {
        super::display_off().unwrap();
    }

    #[test]
    #[ignore] // 保存工作后单独运行；系统会睡眠，需要手动唤醒。
    fn desktop_sleep() {
        super::sleep().unwrap();
    }
}
