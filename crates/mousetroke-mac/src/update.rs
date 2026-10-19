//! 系统 curl 只获取 GitHub API；下载交给浏览器，不在应用内安装。
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Mutex, OnceLock,
};
use std::time::Duration;

use mousetroke_core::update::{
    now, release_notes, select_release_for, version, Source, UpdateState, API,
};
pub use mousetroke_core::update::{Check, Error, RELEASES_URL};

const CAP: usize = 1024 * 1024;

fn fetch_api(url: &str) -> Result<Vec<u8>, Error> {
    if !Source::production().allowed(url, true) {
        return Err(Error::Network);
    }
    // -q 必须是首参数，禁止用户 curlrc 加入重定向、凭证或其他协议。
    // 无 -L，且只接受 200；即使 API 重定向也不会连接新主机。
    let mut child = Command::new("/usr/bin/curl")
        .args([
            "-q",
            "--proto",
            "=https",
            "--proto-redir",
            "=https",
            "--max-redirs",
            "0",
            "--connect-timeout",
            "10",
            "--max-time",
            "30",
            "--max-filesize",
            "1048576",
            "--silent",
            "--fail",
            "--header",
            "Accept: application/vnd.github+json",
            "--user-agent",
            concat!("Mousetroke/", env!("CARGO_PKG_VERSION")),
            "--write-out",
            "\n%{http_code}",
            "--url",
            url,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| Error::Network)?;
    let mut data = Vec::new();
    let read = child
        .stdout
        .take()
        .ok_or(Error::Network)?
        .take((CAP + 6) as u64)
        .read_to_end(&mut data);
    if read.is_err() || data.len() > CAP + 4 {
        let _ = child.kill();
        let _ = child.wait();
        return Err(Error::Network);
    }
    if !child.wait().map_err(|_| Error::Network)?.success() || !data.ends_with(b"\n200") {
        return Err(Error::Network);
    }
    data.truncate(data.len() - 4);
    Ok(data)
}

pub fn check() -> Result<Check, Error> {
    let _ = crate::log::init();
    crate::log!("检查更新：当前 {}", env!("CARGO_PKG_VERSION"));
    let result = (|| {
        let release =
            select_release_for(&fetch_api(API)?, &Source::production(), "Mousetroke.dmg")?;
        Ok(Check {
            current: env!("CARGO_PKG_VERSION"),
            newer: version(&release.latest) > version(env!("CARGO_PKG_VERSION")),
            latest: release.latest,
            notes: release_notes(&release.body, mousetroke_core::i18n::lang().tag()),
            notes_url: release.notes_url,
            download_url: Some(release.exe),
        })
    })();
    match &result {
        Ok(value) => crate::log!("检查更新成功：最新 {}", value.latest),
        Err(_) => crate::log!("检查更新失败"),
    }
    result
}

// 前端只传操作名；下载前再次解析发布，避免接受任意 WebView URL。
pub fn download() -> Result<String, Error> {
    check()?.download_url.ok_or(Error::Network)
}

pub fn install(_exe: &Path, _config: &Path) -> Result<(), Error> {
    Err(Error::Current)
}

static ENABLED: AtomicBool = AtomicBool::new(false);
static WORKER: OnceLock<std::thread::Thread> = OnceLock::new();
static NOTICE: Mutex<Option<(String, mpsc::Sender<bool>)>> = Mutex::new(None);

pub fn set_automatic(enabled: bool) {
    ENABLED.store(enabled, Ordering::Release);
    if let Some(thread) = WORKER.get() {
        thread.unpark();
    }
}

pub fn start_automatic(config: PathBuf, enabled: bool) {
    set_automatic(enabled);
    let _ = std::thread::Builder::new()
        .name("mousetroke-update".into())
        .spawn(move || {
            let _ = WORKER.set(std::thread::current());
            let path = config.with_file_name("update-state.json");
            loop {
                if ENABLED.load(Ordering::Acquire) {
                    let mut state = UpdateState::load(&path);
                    if state.due(now()) {
                        state.last_check = Some(now());
                        // 请求前持久化尝试时间；失败和重启同样受每日限制。
                        if state.save(&path).is_ok() {
                            if let Ok(result) = check() {
                                if result.newer
                                    && state.last_notified.as_ref() != Some(&result.latest)
                                    && ENABLED.load(Ordering::Acquire)
                                {
                                    let (tx, rx) = mpsc::channel();
                                    if let Ok(mut notice) = NOTICE.lock() {
                                        *notice = Some((result.latest.clone(), tx));
                                    }
                                    if rx.recv_timeout(Duration::from_secs(5)) == Ok(true) {
                                        state.last_notified = Some(result.latest);
                                        let _ = state.save(&path);
                                    }
                                }
                            }
                        } else {
                            crate::log!("更新检查状态写入失败，跳过自动检查");
                        }
                    }
                }
                std::thread::park_timeout(Duration::from_secs(60));
            }
        });
}

pub fn take_notice() -> Option<(String, mpsc::Sender<bool>)> {
    let notice = NOTICE.lock().ok()?.take()?;
    if ENABLED.load(Ordering::Acquire) {
        Some(notice)
    } else {
        let _ = notice.1.send(false);
        None
    }
}
