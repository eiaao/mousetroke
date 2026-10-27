//! 系统 curl 下载，CommonCrypto 校验，按当前签名身份原地替换并交接重启。
mod http;
mod install;
mod relaunch;
mod sha256;

use std::path::PathBuf;
use std::sync::mpsc;

pub use install::install;
use mousetroke_core::update::{select_release_for, Automatic, Release, Source, MARKER_CAP};
pub use mousetroke_core::update::{Check, Error, RELEASES_URL};
pub use relaunch::relaunch;

const CAP: usize = 1024 * 1024;

fn fetch_release(source: &Source) -> Result<Release, Error> {
    select_release_for(
        &http::get(source, &source.api, true, CAP)?,
        source,
        crate::update_helpers::PAYLOAD,
    )
}

pub fn check() -> Result<Check, Error> {
    check_source(&Source::configured(), false)
}

fn check_automatic() -> Result<Check, Error> {
    check_source(&Source::configured(), true)
}

fn check_source(source: &Source, automatic: bool) -> Result<Check, Error> {
    let _ = crate::log::init();
    crate::log!("检查更新：当前 {}", env!("CARGO_PKG_VERSION"));
    let check = || {
        fetch_release(source).map(|release| {
            let dmg = release.exe.clone();
            Check::from_release(release, Some(dmg))
        })
    };
    let result = if automatic {
        mousetroke_core::update::automatic_check(
            || http::get(source, &source.marker_url(), false, MARKER_CAP),
            check,
            |message| crate::log!("{message}"),
        )
    } else {
        check()
    };
    match &result {
        Ok(value) => crate::log!("检查更新成功：最新 {}", value.latest),
        Err(error) => crate::log!("检查更新失败：{error:?}"),
    }
    result
}

// 前端只传操作名；手动下载也再次解析发布，避免接受任意 WebView URL。
pub fn download() -> Result<String, Error> {
    check()?.download_url.ok_or(Error::Network)
}

static AUTOMATIC: Automatic = Automatic::new();

pub fn set_automatic(enabled: bool) {
    AUTOMATIC.set(enabled);
}

pub fn start_automatic(config: PathBuf, enabled: bool) {
    AUTOMATIC.start(config, enabled, check_automatic, |message| {
        crate::log!("{message}")
    });
}

pub fn take_notice() -> Option<(String, mpsc::Sender<bool>)> {
    AUTOMATIC.take_notice()
}
