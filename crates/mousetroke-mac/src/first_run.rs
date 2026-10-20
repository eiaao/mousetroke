//! 临时挂载位置只提供安装或退出；权限必须绑定到固定的安装路径。
pub fn needs_install(path: &std::path::Path) -> bool {
    let path = path.to_string_lossy();
    path.starts_with("/Volumes/") || path.contains("/AppTranslocation/")
}

/// 只接受真正的 .app 祖先目录，不把路径中碰巧含有 .app 的普通目录当作 bundle。
fn bundle_path(exe: &std::path::Path) -> Option<&std::path::Path> {
    exe.ancestors()
        .find(|path| path.extension().is_some_and(|ext| ext == "app"))
}

fn mounted_volume(path: &std::path::Path) -> Option<std::path::PathBuf> {
    let mut parts = path.components();
    if parts.next()? != std::path::Component::RootDir || parts.next()?.as_os_str() != "Volumes" {
        return None;
    }
    Some(std::path::Path::new("/Volumes").join(parts.next()?.as_os_str()))
}

#[cfg(target_os = "macos")]
pub fn handle_install(mtm: objc2::MainThreadMarker) -> bool {
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    if !needs_install(&exe) {
        return false;
    }
    use mousetroke_core::i18n::{keys, t};
    use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
    let app = NSApplication::sharedApplication(mtm);
    let _ = app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
    #[allow(deprecated)]
    app.activateIgnoringOtherApps(true);
    crate::log!("临时位置启动，先安装: {}", exe.display());
    if alert(
        mtm,
        t(keys::MAC_INSTALL_TITLE),
        t(keys::MAC_INSTALL_BODY),
        &[t(keys::MAC_INSTALL_MOVE), t(keys::TRAY_MENU_QUIT)],
    ) != 1000
    {
        crate::log!("用户取消安装，退出");
        return true;
    }
    let result = bundle_path(&exe)
        .ok_or_else(|| t(keys::MAC_INSTALL_NOT_BUNDLE).to_string())
        .and_then(install);
    match result {
        Ok(destination) => {
            // 此时尚未 acquire 引擎锁；open -n 的副本可以直接取得同一把锁。
            crate::log!("启动已安装副本: {}", destination.display());
            if let Err(error) = command(
                "/usr/bin/open",
                &[std::ffi::OsStr::new("-n"), destination.as_os_str()],
            ) {
                install_error(mtm, &error);
            } else if let Some(volume) = mounted_volume(&exe) {
                eject_later(&volume);
            }
        }
        Err(error) => {
            crate::log!("安装失败: {error}");
            install_error(mtm, &error);
        }
    }
    true
}

#[cfg(target_os = "macos")]
fn alert(mtm: objc2::MainThreadMarker, title: &str, body: &str, buttons: &[&str]) -> isize {
    use objc2_app_kit::NSAlert;
    use objc2_foundation::NSString;
    let alert = NSAlert::new(mtm);
    alert.setMessageText(&NSString::from_str(title));
    alert.setInformativeText(&NSString::from_str(body));
    for button in buttons {
        alert.addButtonWithTitle(&NSString::from_str(button));
    }
    alert.runModal()
}

#[cfg(target_os = "macos")]
fn install_error(mtm: objc2::MainThreadMarker, error: &str) {
    use mousetroke_core::i18n::{keys, t, tf};
    crate::log!("安装/启动失败: {error}");
    alert(
        mtm,
        t(keys::MAC_INSTALL_ERROR_TITLE),
        &tf(keys::MAC_INSTALL_ERROR_BODY, &[("error", error)]),
        &[t(keys::COMMON_OK)],
    );
}

#[cfg(target_os = "macos")]
fn command(program: &str, args: &[&std::ffi::OsStr]) -> Result<(), String> {
    use mousetroke_core::i18n::{keys, tf};
    crate::log!("执行安装命令: {program} {args:?}");
    let output = std::process::Command::new(program)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        let error = format!(
            "{}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        );
        Err(tf(
            keys::MAC_INSTALL_COMMAND_FAILED,
            &[("command", program), ("error", &error)],
        ))
    }
}

#[cfg(target_os = "macos")]
fn trash(path: &std::path::Path) -> Result<Option<std::path::PathBuf>, String> {
    use objc2_foundation::{NSFileManager, NSString, NSURL};
    crate::log!("移到废纸篓: {}", path.display());
    let url = NSURL::fileURLWithPath(&NSString::from_str(&path.to_string_lossy()));
    let mut result = None;
    NSFileManager::defaultManager()
        .trashItemAtURL_resultingItemURL_error(&url, Some(&mut result))
        .map_err(|error| error.localizedDescription().to_string())?;
    Ok(result
        .and_then(|url| url.path())
        .map(|p| p.to_string().into()))
}

#[cfg(target_os = "macos")]
fn install(source: &std::path::Path) -> Result<std::path::PathBuf, String> {
    use mousetroke_core::i18n::{keys, t, tf};
    use objc2_app_kit::NSWorkspace;
    use objc2_foundation::{NSFileManager, NSString};
    use std::{ffi::OsStr, path::PathBuf};
    let files = NSFileManager::defaultManager();
    let directory = if files.isWritableFileAtPath(&NSString::from_str("/Applications")) {
        PathBuf::from("/Applications")
    } else {
        let home =
            std::env::var_os("HOME").ok_or_else(|| t(keys::MAC_ERROR_STARTUP).to_string())?;
        let directory = PathBuf::from(home).join("Applications");
        crate::log!("/Applications 不可写，改用 {}", directory.display());
        std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
        directory
    };
    let destination = directory.join("Mousetroke.app");
    // 同一 bundle 的引擎和设置进程都必须退出；不替换仍在运行或有未保存设置的副本。
    let ensure_stopped = || -> Result<(), String> {
        let running = NSWorkspace::sharedWorkspace()
            .runningApplications()
            .iter()
            .find_map(|app| {
                if app.isTerminated() || app.processIdentifier() == std::process::id() as i32 {
                    return None;
                }
                app.bundleURL().and_then(|url| url.path()).and_then(|path| {
                    let path = PathBuf::from(path.to_string());
                    let same_identity = app
                        .bundleIdentifier()
                        .is_some_and(|id| id.to_string() == "io.github.eiaao.mousetroke");
                    (same_identity
                        || path == destination
                        || path
                            .canonicalize()
                            .ok()
                            .zip(destination.canonicalize().ok())
                            .is_some_and(|(a, b)| a == b))
                    .then_some(path)
                })
            });
        if let Some(running) = running {
            Err(tf(
                keys::MAC_INSTALL_RUNNING,
                &[("path", &running.to_string_lossy())],
            ))
        } else {
            Ok(())
        }
    };
    ensure_stopped()?;
    let staging = directory.join(format!(".Mousetroke-install-{}.app", std::process::id()));
    if staging.exists() {
        trash(&staging)?;
    }
    let prepare = || -> Result<(), String> {
        command("/usr/bin/ditto", &[source.as_os_str(), staging.as_os_str()])?;
        // 仅处理刚复制的副本；没有 quarantine 时不把 xattr 的“属性不存在”当作安装失败。
        let attributes = std::process::Command::new("/usr/bin/xattr")
            .arg("-r")
            .arg(&staging)
            .output()
            .map_err(|e| e.to_string())?;
        if !attributes.status.success() {
            return Err(String::from_utf8_lossy(&attributes.stderr).into_owned());
        }
        if String::from_utf8_lossy(&attributes.stdout).contains("com.apple.quarantine") {
            command(
                "/usr/bin/xattr",
                &[
                    OsStr::new("-dr"),
                    OsStr::new("com.apple.quarantine"),
                    staging.as_os_str(),
                ],
            )?;
        }
        crate::log!("副本就绪，quarantine 已检查: {}", staging.display());
        Ok(())
    };
    let result = prepare().and_then(|()| {
        // 复制期间用户可能重新打开旧副本，替换前再检查一次。
        ensure_stopped()?;
        let previous = if destination.exists() {
            trash(&destination)?
        } else {
            None
        };
        if let Err(error) = std::fs::rename(&staging, &destination) {
            if let Some(previous) = previous {
                let restored = std::fs::rename(&previous, &destination);
                crate::log!("安装失败，恢复旧副本: {restored:?}");
            }
            return Err(error.to_string());
        }
        crate::log!("安装完成: {}", destination.display());
        Ok(destination.clone())
    });
    if result.is_err() && staging.exists() {
        if let Err(cleanup) = trash(&staging) {
            crate::log!("清理安装暂存副本失败: {cleanup}");
        }
    }
    result
}

#[cfg(target_os = "macos")]
fn eject_later(volume: &std::path::Path) {
    use std::os::unix::process::CommandExt;
    use std::process::{Command, Stdio};
    crate::log!("安排三秒后推出安装卷: {}", volume.display());
    // 独立进程组，父进程退出后仍运行；卷路径作为参数传递，绝不插入 shell 代码。
    let result = Command::new("/bin/sh")
        .args([
            "-c",
            "sleep 3; /usr/bin/hdiutil detach \"$1\"",
            "mousetroke-eject",
        ])
        .arg(volume)
        .process_group(0)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    if let Err(error) = result {
        crate::log!("安排推出安装卷失败: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bundle_and_volume_use_path_components() {
        use std::path::{Path, PathBuf};
        let exe = Path::new("/Volumes/Mousetroke beta/Mousetroke.app/Contents/MacOS/mousetroke");
        assert_eq!(
            bundle_path(exe),
            Some(Path::new("/Volumes/Mousetroke beta/Mousetroke.app"))
        );
        assert_eq!(
            mounted_volume(exe),
            Some(PathBuf::from("/Volumes/Mousetroke beta"))
        );
        assert!(mounted_volume(Path::new("/Applications/Mousetroke.app")).is_none());
        assert!(bundle_path(Path::new("/tmp/a.app.backup/mousetroke")).is_none());
    }
    #[test]
    fn temporary_locations_require_copying_to_applications() {
        for path in ["/Volumes/Mousetroke/Mousetroke.app/Contents/MacOS/mousetroke",
            "/private/var/folders/x/AppTranslocation/UUID/d/Mousetroke.app/Contents/MacOS/mousetroke"] {
            assert!(needs_install(std::path::Path::new(path)));
        }
        for path in [
            "/Applications/Mousetroke.app/Contents/MacOS/mousetroke",
            "/Users/me/Applications/Mousetroke.app/Contents/MacOS/mousetroke",
            "/Users/me/Volumes/example",
            "/tmp/AppTranslocation-backup/mousetroke",
        ] {
            assert!(!needs_install(std::path::Path::new(path)));
        }
    }
}
