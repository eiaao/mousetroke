//! 下载与验证先完成，最后才切换 bundle；失败清理仅针对本次创建的文件。
use super::{fetch_release, http, sha256::sha256, Error, Source, CAP};
use crate::update_helpers::{
    absent, bundle_root, designated_requirement, UpdatePaths, BUNDLE_ID, PAYLOAD,
};
use mousetroke_core::update::{checksum_for, version};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub(super) fn run(command: &mut Command, error: Error) -> Result<std::process::Output, Error> {
    let output = command.stdin(Stdio::null()).output().map_err(|_| error)?;
    if !output.status.success() {
        crate::log!(
            "更新命令失败：{:?}，{}",
            command.get_program(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
        return Err(error);
    }
    Ok(output)
}

pub(super) fn trash(path: &Path) -> Result<PathBuf, Error> {
    use objc2_foundation::{NSFileManager, NSString, NSURL};
    crate::log!("更新副本移到废纸篓：{}", path.display());
    let url = NSURL::fileURLWithPath(&NSString::from_str(path.to_str().ok_or(Error::Replace)?));
    let mut result = None;
    NSFileManager::defaultManager()
        .trashItemAtURL_resultingItemURL_error(&url, Some(&mut result))
        .map_err(|error| {
            crate::log!("移到废纸篓失败：{}", error.localizedDescription());
            Error::Replace
        })?;
    result
        .and_then(|url| url.path())
        .map(|s| PathBuf::from(s.to_string()))
        .ok_or(Error::Replace)
}

struct Staging(PathBuf);
impl Drop for Staging {
    fn drop(&mut self) {
        if std::fs::symlink_metadata(&self.0).is_ok() {
            // 下载或验证开始前仅预留了空目录，可直接释放；已有应用内容则只能移到废纸篓。
            if std::fs::remove_dir(&self.0).is_ok() {
                return;
            }
            if let Err(error) = trash(&self.0) {
                crate::log!("清理更新暂存副本失败，保留原位置：{error:?}");
            }
        }
    }
}

struct Workspace(PathBuf);
impl Workspace {
    fn new() -> Result<Self, Error> {
        use std::os::unix::fs::DirBuilderExt;
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Error::Replace)?
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("mousetroke-update-{}-{nonce}", std::process::id()));
        // create + 0700 保证不会复用其他进程准备的目录或符号链接。
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&path)
            .map_err(|_| Error::NotWritable)?;
        let workspace = Self(path);
        std::fs::create_dir(workspace.0.join("mounts")).map_err(|_| Error::Replace)?;
        Ok(workspace)
    }

    fn detach(&self) -> Result<(), Error> {
        let entries = match std::fs::read_dir(self.0.join("mounts")) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(_) => return Err(Error::Replace),
        };
        let mut failed = false;
        for entry in entries {
            let entry = entry.map_err(|_| Error::Replace)?;
            if !entry.file_type().map_err(|_| Error::Replace)?.is_dir() {
                continue;
            }
            let mount = entry.path();
            crate::log!("卸载更新镜像：{}", mount.display());
            if run(
                Command::new("/usr/bin/hdiutil").arg("detach").arg(&mount),
                Error::Replace,
            )
            .is_err()
                && run(
                    Command::new("/usr/bin/hdiutil")
                        .args(["detach", "-force"])
                        .arg(&mount),
                    Error::Replace,
                )
                .is_err()
            {
                failed = true;
            } else {
                // hdiutil 通常会删除随机挂载点；只尝试移除空目录。
                let _ = std::fs::remove_dir(&mount);
            }
        }
        if failed {
            Err(Error::Replace)
        } else {
            Ok(())
        }
    }

    fn mount(&self, image: &Path) -> Result<PathBuf, Error> {
        crate::log!("只读挂载更新镜像");
        // 守卫在 attach 前已经存在，即使 attach 部分成功后报错也会卸载。
        run(
            Command::new("/usr/bin/hdiutil")
                .args([
                    "attach",
                    "-nobrowse",
                    "-readonly",
                    "-noautoopen",
                    "-mountrandom",
                ])
                .arg(self.0.join("mounts"))
                .arg(image),
            Error::Replace,
        )?;
        let mut bundles = Vec::new();
        for entry in std::fs::read_dir(self.0.join("mounts")).map_err(|_| Error::Replace)? {
            let path = entry
                .map_err(|_| Error::Replace)?
                .path()
                .join("Mousetroke.app");
            if std::fs::symlink_metadata(&path).is_ok_and(|m| m.is_dir()) {
                bundles.push(path);
            }
        }
        if bundles.len() != 1 {
            return Err(Error::Signature);
        }
        Ok(bundles.remove(0))
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        if self.detach().is_ok() {
            // 只删除由本守卫独占创建的临时目录，绝不递归删除用户的 app。
            if let Err(error) = std::fs::remove_dir_all(&self.0) {
                crate::log!("清理更新临时文件失败：{error}");
            }
        } else {
            crate::log!(
                "镜像仍被占用，保留临时目录避免误删挂载内容：{}",
                self.0.display()
            );
        }
    }
}

fn plist(bundle: &Path, key: &str) -> Result<String, Error> {
    let output = run(
        Command::new("/usr/libexec/PlistBuddy")
            .args(["-c", &format!("Print :{key}")])
            .arg(bundle.join("Contents/Info.plist")),
        Error::Signature,
    )?;
    String::from_utf8(output.stdout)
        .map(|s| s.trim_end_matches(['\r', '\n']).into())
        .map_err(|_| Error::Signature)
}

fn read_requirement(bundle: &Path) -> Result<String, Error> {
    let output = run(
        Command::new("/usr/bin/codesign")
            .args(["-d", "-r-"])
            .arg(bundle),
        Error::Signature,
    )?;
    designated_requirement(&format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ))
}

fn validate(bundle: &Path, latest: &str, requirement: &str) -> Result<(), Error> {
    crate::log!("验证更新包标识、版本与签名：{}", bundle.display());
    if plist(bundle, "CFBundleIdentifier")? != BUNDLE_ID
        || plist(bundle, "CFBundleShortVersionString")? != latest
        || plist(bundle, "CFBundleExecutable")? != "mousetroke"
        || !std::fs::symlink_metadata(bundle.join("Contents/MacOS/mousetroke"))
            .is_ok_and(|m| m.is_file())
    {
        return Err(Error::Signature);
    }
    run(
        Command::new("/usr/bin/codesign")
            .args(["--verify", "--deep", "--strict", "--all-architectures"])
            .arg(bundle),
        Error::Signature,
    )?;
    run(
        Command::new("/usr/bin/codesign")
            .args(["-v", "--all-architectures", &format!("-R={requirement}")])
            .arg(bundle),
        Error::Signature,
    )?;
    // 同证书还不够：新包也必须声明相同的证书绑定要求，避免 TCC 因新 DR 变弱而重新授权。
    if !read_requirement(bundle)?.eq_ignore_ascii_case(requirement) {
        return Err(Error::Signature);
    }
    Ok(())
}

fn strip_quarantine(staging: &Path) -> Result<(), Error> {
    let query = || {
        run(
            Command::new("/usr/bin/xattr").arg("-r").arg(staging),
            Error::Replace,
        )
    };
    if String::from_utf8_lossy(&query()?.stdout).contains("com.apple.quarantine") {
        run(
            Command::new("/usr/bin/xattr")
                .args(["-dr", "com.apple.quarantine"])
                .arg(staging),
            Error::Replace,
        )?;
        if String::from_utf8_lossy(&query()?.stdout).contains("com.apple.quarantine") {
            return Err(Error::Replace);
        }
    }
    crate::log!("更新暂存包已确认无 quarantine");
    Ok(())
}

fn engine_pid(bundle: &Path) -> Result<u32, Error> {
    if !crate::instance::engine_running() {
        return Ok(0);
    }
    let (pid, _) = crate::instance::input_state().ok_or(Error::Restart)?;
    let engine =
        objc2_app_kit::NSRunningApplication::runningApplicationWithProcessIdentifier(pid as i32)
            .ok_or(Error::Restart)?;
    let path = engine
        .bundleURL()
        .and_then(|url| url.path())
        .ok_or(Error::Restart)?;
    if Path::new(&path.to_string())
        .canonicalize()
        .map_err(|_| Error::Restart)?
        != bundle
    {
        return Err(Error::Restart);
    }
    Ok(pid)
}

static INSTALL: Mutex<()> = Mutex::new(());

/// 设置进程的阻塞工作线程调用；只从当前可执行文件解析安装位置，不接受 WebView 路径或 URL。
pub fn install(config: &Path) -> Result<(), Error> {
    let _ = crate::log::init();
    let _guard = INSTALL.try_lock().map_err(|_| Error::Busy)?;
    let result = objc2::rc::autoreleasepool(|_| {
        let lock_path = crate::default_config_path();
        let _lock = crate::instance::acquire_named(
            lock_path.parent().ok_or(Error::NotWritable)?,
            "update.lock",
        )
        .map_err(|_| Error::NotWritable)?
        .ok_or(Error::Busy)?;
        let exe = std::env::current_exe().map_err(|_| Error::NotWritable)?;
        // 原路径和真实路径都检查，不能通过链接绕过挂载卷或 AppTranslocation 限制。
        if crate::first_run::needs_install(&exe) {
            return Err(Error::NotWritable);
        }
        let exe = exe.canonicalize().map_err(|_| Error::NotWritable)?;
        if crate::first_run::needs_install(&exe) {
            return Err(Error::NotWritable);
        }
        let paths = UpdatePaths::new(bundle_root(&exe)?)?;
        crate::log!("开始应用内更新：{}", paths.bundle.display());
        let pid = engine_pid(&paths.bundle)?;
        absent(&paths.old)?;
        absent(&paths.staging)?;
        // 实际创建目录探测写权限；失败时 UI 显示不可写并提供手动下载。
        std::fs::create_dir(&paths.staging).map_err(|_| Error::NotWritable)?;
        let _staging = Staging(paths.staging.clone());
        let source = Source::configured();
        crate::log!("重新获取最新发布信息");
        let release = fetch_release(&source)?;
        if version(&release.latest) <= version(env!("CARGO_PKG_VERSION")) {
            return Err(Error::Current);
        }
        run(
            Command::new("/usr/bin/codesign")
                .args(["--verify", "--deep", "--strict", "--all-architectures"])
                .arg(&paths.bundle),
            Error::Signature,
        )?;
        crate::log!("读取当前应用的证书绑定签名要求");
        let requirement = read_requirement(&paths.bundle)?;
        let workspace = Workspace::new()?;
        crate::log!("下载更新校验清单");
        let sums = http::get(&source, &release.sums, false, CAP)?;
        let expected = checksum_for(
            std::str::from_utf8(&sums).map_err(|_| Error::Checksum)?,
            PAYLOAD,
        )?;
        crate::log!("下载更新镜像，上限 64 MiB");
        let data = http::get(&source, &release.exe, false, 64 * 1024 * 1024)?;
        if sha256(&data)? != expected {
            return Err(Error::Checksum);
        }
        crate::log!("更新 SHA-256 校验成功：{}", release.latest);
        let image = workspace.0.join(PAYLOAD);
        std::fs::write(&image, data).map_err(|_| Error::NotWritable)?;
        let mounted = workspace.mount(&image)?;
        validate(&mounted, &release.latest, &requirement)?;
        crate::log!("复制到同目录更新暂存包");
        run(
            Command::new("/usr/bin/ditto")
                .arg(&mounted)
                .arg(&paths.staging),
            Error::Replace,
        )?;
        strip_quarantine(&paths.staging)?;
        validate(&paths.staging, &release.latest, &requirement)?;
        workspace.detach()?;
        // 复制期间引擎可能已退出或换了实例；不同 PID 时停止本次交接。
        if engine_pid(&paths.bundle)? != pid {
            return Err(Error::Restart);
        }
        crate::log!("切换新旧应用包");
        paths.swap()?;
        let handoff = start_helper(&paths, pid, config, &workspace.0.join("ready"));
        if let Err(error) = handoff {
            crate::log!("更新交接失败，恢复旧应用包：{error:?}");
            paths.rollback()?;
            return Err(error);
        }
        crate::log!("更新替换成功，等待设置进程退出后重启：{}", release.latest);
        Ok(())
    });
    if let Err(error) = result {
        crate::log!("安装更新失败：{error:?}");
    }
    result
}

fn start_helper(
    paths: &UpdatePaths,
    engine: u32,
    config: &Path,
    ready: &Path,
) -> Result<(), Error> {
    use std::os::unix::process::CommandExt;
    let mut child = Command::new(paths.bundle.join("Contents/MacOS/mousetroke"))
        .arg("update-relaunch")
        .arg(engine.to_string())
        .arg(std::process::id().to_string())
        .arg(config)
        .arg(ready)
        .process_group(0)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| Error::Restart)?;
    let handoff = (|| {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if std::fs::read_to_string(ready).is_ok_and(|s| s == child.id().to_string()) {
                break;
            }
            if child.try_wait().map_err(|_| Error::Restart)?.is_some() || Instant::now() >= deadline
            {
                return Err(Error::Restart);
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        crate::log!("更新重启助手已就绪：{}", child.id());
        if engine != 0 {
            crate::instance::request_quit(engine).map_err(|_| Error::Restart)?;
        }
        Ok(())
    })();
    if handoff.is_err() {
        let _ = child.kill();
        let _ = child.wait();
    }
    handoff
}
