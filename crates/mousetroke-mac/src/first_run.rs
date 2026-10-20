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

/// 取消不发送退出请求；确认后等待全部实例正常退出，超时返回仍存活的实例。
fn quit_before_replace<A>(
    mut running: impl FnMut() -> Vec<A>,
    confirm: impl FnOnce() -> bool,
    mut terminate: impl FnMut(&A),
    mut wait: impl FnMut(std::time::Duration),
    timeout: std::time::Duration,
) -> Result<bool, A> {
    let apps = running();
    if apps.is_empty() {
        return Ok(true);
    }
    if !confirm() {
        return Ok(false);
    }
    let deadline = std::time::Instant::now() + timeout;
    for app in &apps {
        terminate(app);
    }
    loop {
        let Some(app) = running().into_iter().next() else {
            return Ok(true);
        };
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        if remaining.is_zero() {
            return Err(app);
        }
        wait(remaining.min(std::time::Duration::from_millis(100)));
    }
}

fn matches_install_instance(
    pid: i32,
    current_pid: i32,
    terminated: bool,
    bundle_id: Option<&str>,
    path: Option<&std::path::Path>,
    destination: &std::path::Path,
) -> bool {
    pid != current_pid
        && !terminated
        && (bundle_id == Some("io.github.eiaao.mousetroke")
            || path.is_some_and(|path| {
                path == destination
                    || path
                        .canonicalize()
                        .ok()
                        .zip(destination.canonicalize().ok())
                        .is_some_and(|(a, b)| a == b)
            }))
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
        .and_then(|source| install(mtm, source));
    match result {
        Ok(Some(destination)) => {
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
        Ok(None) => crate::log!("用户取消替换正在运行的副本，退出"),
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
fn running_instances(
    destination: &std::path::Path,
) -> Vec<objc2::rc::Retained<objc2_app_kit::NSRunningApplication>> {
    use objc2_app_kit::NSWorkspace;
    NSWorkspace::sharedWorkspace()
        .runningApplications()
        .iter()
        .filter(|app| {
            let id = app.bundleIdentifier().map(|id| id.to_string());
            let path = app
                .bundleURL()
                .and_then(|url| url.path())
                .map(|path| std::path::PathBuf::from(path.to_string()));
            matches_install_instance(
                app.processIdentifier(),
                std::process::id() as i32,
                app.isTerminated(),
                id.as_deref(),
                path.as_deref(),
                destination,
            )
        })
        .collect()
}

#[cfg(target_os = "macos")]
fn running_error(
    app: &objc2_app_kit::NSRunningApplication,
    destination: &std::path::Path,
) -> String {
    use mousetroke_core::i18n::{keys, tf};
    let path = app
        .bundleURL()
        .and_then(|url| url.path())
        .map(|path| std::path::PathBuf::from(path.to_string()));
    tf(
        keys::MAC_INSTALL_RUNNING,
        &[(
            "path",
            &path.as_deref().unwrap_or(destination).to_string_lossy(),
        )],
    )
}

#[cfg(target_os = "macos")]
fn install(
    mtm: objc2::MainThreadMarker,
    source: &std::path::Path,
) -> Result<Option<std::path::PathBuf>, String> {
    use mousetroke_core::i18n::{keys, t};
    use objc2_foundation::{NSDate, NSFileManager, NSRunLoop, NSString};
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
    // 引擎与 Tauri 设置进程共享 bundle id，二者都接收 AppKit 的正常退出请求。
    // 设置页未拦截 ExitRequested；仍在运行或拒绝退出时绝不替换、也不强杀。
    if !quit_before_replace(
        || running_instances(&destination),
        || {
            alert(
                mtm,
                t(keys::MAC_INSTALL_QUIT_TITLE),
                t(keys::MAC_INSTALL_QUIT_BODY),
                &[t(keys::MAC_INSTALL_QUIT_REPLACE), t(keys::COMMON_CANCEL)],
            ) == 1000
        },
        |app| {
            let accepted = app.terminate();
            crate::log!(
                "请求旧副本正常退出: pid={}, accepted={accepted}",
                app.processIdentifier()
            );
        },
        |duration| {
            // NSRunningApplication 的动态属性随主 run loop 刷新，不能只 sleep。
            NSRunLoop::mainRunLoop().runUntilDate(&NSDate::dateWithTimeIntervalSinceNow(
                duration.as_secs_f64(),
            ));
        },
        std::time::Duration::from_secs(10),
    )
    .map_err(|app| running_error(&app, &destination))?
    {
        return Ok(None);
    }
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
        // ditto 阻塞期间用户可能重新打开旧副本；先刷新主 run loop，再检查进程列表。
        NSRunLoop::mainRunLoop().runUntilDate(&NSDate::dateWithTimeIntervalSinceNow(0.01));
        if let Some(app) = running_instances(&destination).first() {
            return Err(running_error(app, &destination));
        }
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
        Ok(Some(destination.clone()))
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
    use std::{cell::RefCell, time::Duration};

    #[test]
    fn replacement_cancel_sends_no_quit_requests() {
        let result = quit_before_replace(
            || vec![1, 2],
            || false,
            |_| panic!("cancel must not terminate an instance"),
            |_| panic!("cancel must not wait"),
            Duration::ZERO,
        );
        assert_eq!(result, Ok(false));
        assert_eq!(
            quit_before_replace(
                Vec::<i32>::new,
                || panic!("no instances need confirmation"),
                |_| unreachable!(),
                |_| unreachable!(),
                Duration::ZERO,
            ),
            Ok(true)
        );
    }

    #[test]
    fn replacement_requests_both_engine_and_settings_to_quit_and_waits_for_both() {
        let apps = RefCell::new(vec![1, 2]);
        let requested = RefCell::new(Vec::new());
        let result = quit_before_replace(
            || apps.borrow().clone(),
            || true,
            |app| requested.borrow_mut().push(*app),
            |interval| {
                assert!(interval <= Duration::from_millis(100));
                // 引擎先退出，设置进程稍后退出；必须等到两者都停止。
                apps.borrow_mut().remove(0);
            },
            Duration::from_secs(10),
        );
        assert_eq!(result, Ok(true));
        assert_eq!(*requested.borrow(), vec![1, 2]);
        assert!(apps.borrow().is_empty());
    }

    #[test]
    fn replacement_timeout_preserves_the_running_copy() {
        let apps = RefCell::new(vec![1, 2]);
        let requested = RefCell::new(Vec::new());
        let result = quit_before_replace(
            || apps.borrow().clone(),
            || true,
            |app| {
                requested.borrow_mut().push(*app);
                if *app == 1 {
                    apps.borrow_mut().retain(|pid| *pid != 1);
                }
            },
            |_| panic!("expired deadline must not wait again"),
            Duration::ZERO,
        );
        assert_eq!(result, Err(2));
        assert_eq!(*requested.borrow(), vec![1, 2]);
        assert_eq!(*apps.borrow(), vec![2]);
    }

    #[test]
    fn replacement_matches_old_bundle_identity_and_destination_but_excludes_self() {
        use std::path::Path;
        let destination = Path::new("/Applications/Mousetroke.app");
        let same_id = Some("io.github.eiaao.mousetroke");
        let old_copy = Some(Path::new("/Volumes/Older/Mousetroke.app"));
        assert!(matches_install_instance(
            1,
            3,
            false,
            same_id,
            old_copy,
            destination
        ));
        assert!(matches_install_instance(
            2,
            3,
            false,
            same_id,
            None,
            destination
        ));
        assert!(matches_install_instance(
            2,
            3,
            false,
            None,
            Some(destination),
            destination
        ));
        assert!(!matches_install_instance(
            3,
            3,
            false,
            same_id,
            old_copy,
            destination
        ));
        assert!(!matches_install_instance(
            1,
            3,
            true,
            same_id,
            old_copy,
            destination
        ));
        assert!(!matches_install_instance(
            1,
            3,
            false,
            Some("other.app"),
            old_copy,
            destination
        ));
    }

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
