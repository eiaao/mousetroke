//! 13+ 动态查找 SMAppService；11–12 使用 launchd，不引用新系统符号以免旧系统加载失败。
use crate::{keys, t};
use objc2::{
    msg_send,
    rc::Retained,
    runtime::{AnyClass, AnyObject, Bool},
};
use objc2_foundation::{NSBundle, NSError, NSProcessInfo};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[link(name = "ServiceManagement", kind = "framework")]
extern "C" {}
extern "C" {
    fn getuid() -> u32;
}
const LABEL: &str = "io.github.eiaao.mousetroke";

fn service() -> Option<Retained<AnyObject>> {
    if NSProcessInfo::processInfo()
        .operatingSystemVersion()
        .majorVersion
        < 13
    {
        return None;
    }
    let class = AnyClass::get(c"SMAppService")?;
    // 安全：仅 13+ 有此类，mainAppService 返回框架管理的对象，由 Retained 保留。
    Some(unsafe { msg_send![class, mainAppService] })
}

fn plist() -> PathBuf {
    let base = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default();
    base.join("Library/LaunchAgents")
        .join(format!("{LABEL}.plist"))
}
fn domain() -> String {
    // 安全：getuid 无参数，返回当前登录用户的实际 uid。
    format!("gui/{}", unsafe { getuid() })
}
fn enabled_legacy() -> bool {
    if !plist().is_file() {
        return false;
    }
    let domain = domain();
    let disabled = Command::new("/bin/launchctl")
        .args(["print-disabled", &domain])
        .output();
    if disabled.is_ok_and(|out| {
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .any(|line| line.contains(&format!("\"{LABEL}\"")) && line.contains("=> true"))
    }) {
        return false;
    }
    Command::new("/bin/launchctl")
        .args(["print", &format!("{domain}/{LABEL}")])
        .output()
        .is_ok_and(|out| out.status.success())
}

pub fn is_enabled() -> bool {
    if let Some(service) = service() {
        // 安全：status 是 macOS 13+ SMAppService 的 NSInteger 属性；1 为 Enabled。
        let status: isize = unsafe { msg_send![&*service, status] };
        status == 1
    } else {
        enabled_legacy()
    }
}

pub fn set(on: bool, exe: &Path) -> Result<(), String> {
    if let Some(service) = service() {
        if NSBundle::mainBundle().bundleIdentifier().is_none() {
            return Err(t(keys::MAC_AUTOSTART_BUNDLE_REQUIRED).into());
        }
        let mut error: Option<Retained<NSError>> = None;
        // 安全：两个 selector 仅在 13+ 调用，NSError** 的写回由 objc2 管理所有权。
        let ok: Bool = unsafe {
            if on {
                msg_send![&*service, registerAndReturnError: &mut error]
            } else {
                msg_send![&*service, unregisterAndReturnError: &mut error]
            }
        };
        if !ok.as_bool() {
            return Err(error
                .map(|e| e.localizedDescription().to_string())
                .unwrap_or_else(|| t(keys::MAC_AUTOSTART_FAILED).into()));
        }
        if on && !is_enabled() {
            return Err(t(keys::MAC_AUTOSTART_APPROVAL).into());
        }
        return Ok(());
    }
    let path = plist();
    let domain = domain();
    if !on {
        // 禁用后续登录启动，不停止由这个 LaunchAgent 启动、仍在处理手势的引擎。
        let disabled = Command::new("/bin/launchctl")
            .args(["disable", &format!("{domain}/{LABEL}")])
            .output()
            .map_err(|e| e.to_string())?;
        if !disabled.status.success() {
            return Err(t(keys::MAC_AUTOSTART_FAILED).into());
        }
        if path.exists() {
            std::fs::remove_file(path).map_err(|e| e.to_string())?;
        }
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let text = launch_agent(&exe.to_string_lossy());
    let temp = path.with_extension("plist.tmp");
    std::fs::write(&temp, text)
        .and_then(|_| std::fs::rename(&temp, &path))
        .map_err(|e| e.to_string())?;
    let enable = Command::new("/bin/launchctl")
        .args(["enable", &format!("{domain}/{LABEL}")])
        .output()
        .map_err(|e| e.to_string())?;
    if !enable.status.success() {
        return Err(t(keys::MAC_AUTOSTART_FAILED).into());
    }
    if is_enabled() {
        return Ok(());
    }
    let output = Command::new("/bin/launchctl")
        .args(["bootstrap", &domain])
        .arg(&path)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() || !is_enabled() {
        return Err(t(keys::MAC_AUTOSTART_FAILED).into());
    }
    Ok(())
}

fn launch_agent(exe: &str) -> String {
    let exe = exe
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict><key>Label</key><string>{LABEL}</string>
<key>ProgramArguments</key><array><string>{exe}</string></array>
<key>RunAtLoad</key><true/></dict></plist>
"#
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn launch_agent_preserves_spaces_and_escapes_xml() {
        let xml = super::launch_agent("/Applications/A & B's.app/Contents/MacOS/mousetroke");
        assert!(xml.contains("A &amp; B&apos;s.app"));
        assert!(xml.contains("<array><string>"));
        assert!(!xml.contains("settings"));
    }
}
