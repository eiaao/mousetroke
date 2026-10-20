//! 系统 curl 只获取 GitHub API；下载交给浏览器，不在应用内安装。
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;

use mousetroke_core::update::{select_release_for, Automatic, Source, API};
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
    let result = fetch_api(API)
        .and_then(|data| select_release_for(&data, &Source::production(), "Mousetroke.dmg"))
        .map(|release| {
            let dmg = release.exe.clone();
            Check::from_release(release, Some(dmg))
        });
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

static AUTOMATIC: Automatic = Automatic::new();

pub fn set_automatic(enabled: bool) {
    AUTOMATIC.set(enabled);
}

pub fn start_automatic(config: PathBuf, enabled: bool) {
    AUTOMATIC.start(config, enabled, check, |message| crate::log!("{message}"));
}

pub fn take_notice() -> Option<(String, mpsc::Sender<bool>)> {
    AUTOMATIC.take_notice()
}
