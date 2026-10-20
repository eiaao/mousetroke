//! 默认每日及手动更新检查；网络、校验、文件替换均不在钩子路径运行。

mod http;

use std::path::{Path, PathBuf};
use std::sync::{mpsc, Mutex};
use std::time::Duration;

pub use mousetroke_core::update::{Check, Error, RELEASES_URL};
use mousetroke_core::update::{version, Automatic, Source, Release};
#[cfg(test)]
use mousetroke_core::update::{now, release_notes, UpdateState, API};

fn select_release(data: &[u8], source: &Source) -> Result<Release, Error> {
    mousetroke_core::update::select_release_for(data, source, "mousetroke.exe")
}

fn fetch_release(source: &Source) -> Result<Release, Error> {
    select_release(&http::get(source, &source.api, true, 1024 * 1024)?, source)
}

pub fn check() -> Result<Check, Error> {
    crate::log::init();
    crate::log!("检查更新：当前 {}", env!("CARGO_PKG_VERSION"));
    let result = fetch_release(&Source::configured()).map(|r| Check::from_release(r, None));
    match &result {
        Ok(r) => crate::log!("检查更新成功：最新 {}", r.latest),
        Err(_) => crate::log!("检查更新失败"),
    }
    result
}

fn checksum_line(text: &str) -> Result<String, Error> {
    mousetroke_core::update::checksum_for(text, "mousetroke.exe")
}

fn sha256(data: &[u8]) -> Result<String, Error> {
    use windows::Win32::Security::Cryptography::*;
    struct Algorithm(BCRYPT_ALG_HANDLE);
    impl Drop for Algorithm { fn drop(&mut self) { unsafe { let _ = BCryptCloseAlgorithmProvider(self.0, 0); } } }
    struct Hash(BCRYPT_HASH_HANDLE);
    impl Drop for Hash { fn drop(&mut self) { unsafe { let _ = BCryptDestroyHash(self.0); } } }
    unsafe {
        let mut algorithm = BCRYPT_ALG_HANDLE::default();
        BCryptOpenAlgorithmProvider(&mut algorithm, BCRYPT_SHA256_ALGORITHM, None, BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS(0)).ok().map_err(|_| Error::Checksum)?;
        let algorithm = Algorithm(algorithm);
        let mut hash = BCRYPT_HASH_HANDLE::default();
        BCryptCreateHash(algorithm.0, &mut hash, None, None, 0).ok().map_err(|_| Error::Checksum)?;
        let hash = Hash(hash);
        BCryptHashData(hash.0, data, 0).ok().map_err(|_| Error::Checksum)?;
        let mut digest = [0; 32];
        BCryptFinishHash(hash.0, &mut digest, 0).ok().map_err(|_| Error::Checksum)?;
        Ok(digest.iter().map(|b| format!("{b:02x}")).collect())
    }
}

fn sidecar(exe: &Path, suffix: &str) -> PathBuf {
    let mut name = exe.as_os_str().to_os_string();
    name.push(suffix);
    PathBuf::from(name)
}

struct Partial(PathBuf);
impl Drop for Partial { fn drop(&mut self) { let _ = std::fs::remove_file(&self.0); } }

/// 替换事务：失败时保留原程序；启动/退出请求失败也可显式回滚。
struct Replacement { exe: PathBuf, old: PathBuf }
impl Replacement {
    fn rollback(&self) -> Result<(), Error> {
        let new = sidecar(&self.exe, ".new");
        std::fs::rename(&self.exe, &new).map_err(|_| Error::Replace)?;
        if std::fs::rename(&self.old, &self.exe).is_err() {
            let _ = std::fs::rename(&new, &self.exe);
            return Err(Error::Replace);
        }
        let _ = std::fs::remove_file(new);
        Ok(())
    }
}

fn download_replace(source: &Source, release: &Release, exe: &Path) -> Result<Replacement, Error> {
    use std::io::Write;
    // 先试写，无法原地更新时立即让界面提供发布页。
    let new_path = sidecar(exe, ".new");
    // 上次更新中途断电或被结束时可能留下半个文件；不清掉会被误判为目录不可写。
    let _ = std::fs::remove_file(&new_path);
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&new_path).map_err(|_| Error::NotWritable)?;
    let new = Partial(new_path);
    let sums = http::get(source, &release.sums, false, 1024 * 1024)?;
    let expected = checksum_line(std::str::from_utf8(&sums).map_err(|_| Error::Checksum)?)?;
    let data = http::get(source, &release.exe, false, 64 * 1024 * 1024)?;
    if sha256(&data)? != expected { return Err(Error::Checksum); }
    crate::log!("更新下载及 SHA-256 校验成功：{}", release.latest);
    file.write_all(&data).and_then(|_| file.sync_all()).map_err(|_| Error::NotWritable)?;
    drop(file);
    let old = sidecar(exe, ".old");
    // 不覆盖仍可能被旧进程占用的备份。
    if old.exists() { std::fs::remove_file(&old).map_err(|_| Error::Replace)?; }
    std::fs::rename(exe, &old).map_err(|_| Error::Replace)?;
    if std::fs::rename(&new.0, exe).is_err() {
        std::fs::rename(&old, exe).map_err(|_| Error::Replace)?;
        return Err(Error::Replace);
    }
    Ok(Replacement { exe: exe.into(), old })
}

static INSTALL: Mutex<()> = Mutex::new(());

/// 设置进程调用：启动继任者后向托盘发送与「退出」相同的消息。
/// 新进程等待两个旧进程退出，避免互斥体竞争及 .old 仍被设置 WebView 占用。
pub fn install(exe: &Path, config: &Path) -> Result<(), Error> {
    crate::log::init();
    let _guard = INSTALL.try_lock().map_err(|_| Error::Busy)?;
    let result = (|| {
        let source = Source::configured();
        let release = fetch_release(&source)?;
        if version(&release.latest) <= version(env!("CARGO_PKG_VERSION")) { return Err(Error::Current); }
        let engine = crate::tray::engine_window();
        if crate::engine_running() && engine.is_none() { return Err(Error::Restart); }
        let replaced = download_replace(&source, &release, exe)?;
        let pid = engine.map(|(_, pid)| pid).unwrap_or(0);
        let child = std::process::Command::new(exe).arg("--after-update").arg(pid.to_string())
            .arg(std::process::id().to_string()).arg("--config").arg(config).spawn();
        let mut child = match child {
            Ok(c) => c,
            Err(_) => { replaced.rollback()?; return Err(Error::Restart); }
        };
        if engine.is_some_and(|(hwnd, _)| !crate::tray::request_exit(hwnd)) {
            let _ = child.kill();
            let _ = child.wait();
            replaced.rollback()?;
            return Err(Error::Restart);
        }
        crate::log!("更新替换成功，重启：{}", release.latest);
        Ok(())
    })();
    if let Err(e) = result { crate::log!("安装更新失败：{e:?}"); }
    result
}

/// 只在引擎取得单实例后清理；普通 settings 进程不能抢删正在使用的备份。
pub fn cleanup() {
    if let Ok(exe) = std::env::current_exe() { let _ = std::fs::remove_file(sidecar(&exe, ".old")); }
}

/// 有界等待旧进程，超时或无权观察时不启动第二个引擎。
pub fn wait_for_previous(pids: &[u32]) -> Result<(), Error> {
    use windows::Win32::Foundation::{CloseHandle, ERROR_INVALID_PARAMETER, WAIT_OBJECT_0};
    use windows::Win32::System::Threading::{OpenProcess, WaitForSingleObject, SYNCHRONIZATION_SYNCHRONIZE};
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    for &pid in pids.iter().filter(|&&p| p != 0) {
        unsafe {
            match OpenProcess(windows::Win32::System::Threading::PROCESS_ACCESS_RIGHTS(SYNCHRONIZATION_SYNCHRONIZE.0), false, pid) {
                Ok(handle) => {
                    let left = deadline.saturating_duration_since(std::time::Instant::now()).as_millis() as u32;
                    let status = WaitForSingleObject(handle, left);
                    let _ = CloseHandle(handle);
                    if status != WAIT_OBJECT_0 { return Err(Error::Restart); }
                }
                Err(e) if e.code() == windows::core::HRESULT::from_win32(ERROR_INVALID_PARAMETER.0) => {}
                Err(_) => return Err(Error::Restart),
            }
        }
    }
    Ok(())
}

static AUTOMATIC: Automatic = Automatic::new();

pub fn set_automatic(enabled: bool) { AUTOMATIC.set(enabled); }

/// 仅从引擎启动调用；每日限制与通知握手见 core。
pub fn start_automatic(config: PathBuf, enabled: bool) {
    AUTOMATIC.start(config, enabled, check, |message| crate::log!("{message}"));
}

/// 托盘定时器取出通知；只有成功提交给托盘才记录已通知版本。
pub fn take_notice() -> Option<(String, mpsc::Sender<bool>)> { AUTOMATIC.take_notice() }

#[cfg(test)]
#[path = "update_tests.rs"]
mod tests;
