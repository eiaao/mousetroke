//! 跨平台发布协议与每日自动检查；网络与签名摘要由平台层完成。
use crate::i18n::keys;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct Url<'a> {
    pub host: &'a str,
    pub port: u16,
    pub path: &'a str,
    pub secure: bool,
}

pub fn parse_url(text: &str) -> Option<Url<'_>> {
    if text.bytes().any(|b| b <= 32 || b == 127) || text.contains(['\\', '#', '@']) {
        return None;
    }
    let (secure, rest) = if let Some(s) = text.strip_prefix("https://") {
        (true, s)
    } else {
        (false, text.strip_prefix("http://")?)
    };
    let (authority, path) = rest
        .split_once('/')
        .map(|(a, _)| (a, &rest[a.len()..]))
        .unwrap_or((rest, "/"));
    let (host, port) = match authority.split_once(':') {
        Some((h, p)) if !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()) => {
            (h, p.parse().ok()?)
        }
        Some(_) => return None,
        None => (authority, if secure { 443 } else { 80 }),
    };
    if host.is_empty()
        || port == 0
        || !host
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
    {
        return None;
    }
    Some(Url {
        host,
        port,
        path,
        secure,
    })
}

pub const RELEASES_URL: &str = "https://github.com/eiaao/mousetroke/releases/latest";
pub const API: &str = "https://api.github.com/repos/eiaao/mousetroke/releases/latest";
pub const MARKER_CAP: usize = 64;
pub const DAY: u64 = 24 * 60 * 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Network,
    Checksum,
    NotWritable,
    Replace,
    Restart,
    Busy,
    Current,
    Signature,
}

impl Error {
    pub fn key(self) -> &'static str {
        match self {
            Self::Network => keys::UPDATE_ERROR_NETWORK,
            Self::Checksum => keys::UPDATE_ERROR_CHECKSUM,
            Self::NotWritable => keys::UPDATE_ERROR_NOT_WRITABLE,
            Self::Replace => keys::UPDATE_ERROR_REPLACE,
            Self::Restart => keys::UPDATE_ERROR_RESTART,
            Self::Busy => keys::UPDATE_ERROR_BUSY,
            Self::Current => keys::UI_UPDATE_CURRENT,
            Self::Signature => keys::MAC_UPDATE_ERROR_SIGNATURE,
        }
    }
}

/// 严格接受三个十进制分量，拒绝预发布、溢出及尾随字符。
pub fn version(tag: &str) -> Option<[u64; 3]> {
    let parts: Vec<_> = tag.strip_prefix('v').unwrap_or(tag).split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let mut out = [0; 3];
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        out[i] = part.parse().ok()?;
    }
    Some(out)
}

pub fn marker_version(data: &[u8]) -> Option<[u64; 3]> {
    if data.len() > MARKER_CAP {
        return None;
    }
    let text = std::str::from_utf8(data).ok()?;
    let tag = text
        .strip_suffix("\r\n")
        .or_else(|| text.strip_suffix('\n'))
        .unwrap_or(text);
    if !tag.starts_with('v') {
        return None;
    }
    version(tag)
}

#[derive(Debug)]
pub struct Source {
    pub api: String,
    pub local: Option<String>,
}

impl Source {
    pub fn production() -> Self {
        Self {
            api: API.into(),
            local: None,
        }
    }

    pub fn configured() -> Self {
        #[cfg(debug_assertions)]
        if let Ok(base) = std::env::var("MOUSETROKE_UPDATE_BASE") {
            if let Some(source) = Self::local(&base) {
                return source;
            }
        }
        Self::production()
    }

    /// 仅调试配置入口调用；生产配置始终使用固定 API。
    pub fn local(base: &str) -> Option<Self> {
        let base = base.trim_end_matches('/');
        let url = parse_url(base)?;
        if url.secure || url.host != "127.0.0.1" || base.contains('?') {
            return None;
        }
        let authority = base.strip_prefix("http://")?.split('/').next()?;
        if !authority.contains(':') {
            return None;
        }
        Some(Self {
            api: format!("{base}/repos/eiaao/mousetroke/releases/latest"),
            local: Some(format!("http://{authority}")),
        })
    }

    pub fn marker_url(&self) -> String {
        match &self.local {
            Some(origin) => format!("{origin}/latest.txt"),
            None => format!("{RELEASES_URL}/download/latest.txt"),
        }
    }

    pub fn allowed(&self, text: &str, api: bool) -> bool {
        let Some(url) = parse_url(text) else {
            return false;
        };
        if let Some(origin) = &self.local {
            return !url.secure
                && (text == origin
                    || text
                        .strip_prefix(origin)
                        .is_some_and(|s| s.starts_with('/')));
        }
        url.secure
            && url.port == 443
            && if api {
                url.host.eq_ignore_ascii_case("api.github.com")
            } else {
                [
                    "github.com",
                    "objects.githubusercontent.com",
                    "release-assets.githubusercontent.com",
                ]
                .iter()
                .any(|h| url.host.eq_ignore_ascii_case(h))
            }
    }
}

#[derive(Debug)]
pub struct Release {
    pub latest: String,
    pub notes_url: String,
    pub body: String,
    pub exe: String,
    pub sums: String,
}

pub fn select_release_for(data: &[u8], source: &Source, payload: &str) -> Result<Release, Error> {
    let v: serde_json::Value = serde_json::from_slice(data).map_err(|_| Error::Network)?;
    if v["draft"].as_bool() != Some(false) || v["prerelease"].as_bool() != Some(false) {
        return Err(Error::Network);
    }
    let tag = v["tag_name"]
        .as_str()
        .filter(|s| version(s).is_some())
        .ok_or(Error::Network)?;
    let notes = v["html_url"].as_str().ok_or(Error::Network)?;
    if !source.allowed(notes, false) {
        return Err(Error::Network);
    }
    let assets = v["assets"].as_array().ok_or(Error::Network)?;
    let asset = |name: &str| -> Result<String, Error> {
        let mut found = assets.iter().filter(|a| a["name"].as_str() == Some(name));
        let url = found
            .next()
            .and_then(|a| a["browser_download_url"].as_str())
            .ok_or(Error::Network)?;
        if found.next().is_some() || !source.allowed(url, false) {
            return Err(Error::Network);
        }
        Ok(url.into())
    };
    Ok(Release {
        latest: tag.trim_start_matches('v').into(),
        notes_url: notes.into(),
        body: v["body"].as_str().unwrap_or_default().into(),
        exe: asset(payload)?,
        sums: asset("SHA256SUMS.txt")?,
    })
}

/// 发布正文只转成纯文本，不渲染远程 HTML 或 Markdown。
fn plain_note(input: &str) -> String {
    let mut text = String::new();
    let mut rest = input;
    while let Some(start) = rest.find('<') {
        text.push_str(&rest[..start]);
        let tag = &rest[start..];
        if tag.starts_with("<!--") {
            rest = tag
                .find("-->")
                .map(|end| &tag[end + 3..])
                .unwrap_or_default();
            continue;
        }
        let Some(end) = tag.find('>') else {
            rest = "";
            break;
        };
        let name = tag[1..end]
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();
        rest = &tag[end + 1..];
        if name == "script" || name == "style" {
            let closing = format!("</{name}>");
            rest = rest
                .to_ascii_lowercase()
                .find(&closing)
                .map(|end| &rest[end + closing.len()..])
                .unwrap_or_default();
        }
    }
    text.push_str(rest);
    let chars: Vec<_> = text.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\\' && i + 1 < chars.len() && chars[i + 1].is_ascii_punctuation() {
            out.push(chars[i + 1]);
            i += 2;
            continue;
        }
        if c == '!' && chars.get(i + 1) == Some(&'[') {
            i += 1;
            continue;
        }
        if c == '[' {
            if let Some(end) = (i + 1..chars.len()).find(|&j| chars[j] == ']') {
                if let Some(&open @ ('(' | '[')) = chars.get(end + 1) {
                    let close = if open == '(' { ')' } else { ']' };
                    let mut depth = 1;
                    let mut next = end + 2;
                    while next < chars.len() && depth > 0 {
                        if chars[next] == open {
                            depth += 1;
                        }
                        if chars[next] == close {
                            depth -= 1;
                        }
                        next += 1;
                    }
                    if depth == 0 {
                        out.extend(
                            chars[i + 1..end]
                                .iter()
                                .filter(|c| !matches!(c, '*' | '_' | '`' | '~')),
                        );
                        i = next;
                        continue;
                    }
                }
            }
        }
        if !matches!(c, '*' | '_' | '`' | '~') {
            out.push(c);
        }
        i += 1;
    }
    let out = out.split_whitespace().collect::<Vec<_>>().join(" ");
    if out.chars().count() > 300 {
        out.chars().take(299).chain(['…']).collect()
    } else {
        out
    }
}

pub fn release_notes(body: &str, language: &str) -> Vec<String> {
    let section = |heading: &str| {
        let mut active = false;
        let mut bullets: Vec<String> = Vec::new();
        for line in body.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                active = trimmed == heading;
            } else if active {
                if let Some(bullet) = ["- ", "* ", "+ "]
                    .iter()
                    .find_map(|prefix| trimmed.strip_prefix(prefix))
                {
                    bullets.push(bullet.into());
                } else if line.starts_with("  ") && !trimmed.is_empty() {
                    if let Some(last) = bullets.last_mut() {
                        last.push(' ');
                        last.push_str(trimmed);
                    }
                }
            }
        }
        bullets
            .iter()
            .map(|bullet| plain_note(bullet))
            .filter(|bullet| !bullet.is_empty())
            .take(12)
            .collect::<Vec<_>>()
    };
    if language.to_ascii_lowercase().split('-').next() == Some("zh") {
        let notes = section("### 中文"); // i18n:allow 更新日志格式标记，非界面文案。
        if !notes.is_empty() {
            return notes;
        }
    }
    section("### English")
}

#[derive(Serialize)]
pub struct Check {
    pub current: &'static str,
    pub latest: String,
    pub newer: bool,
    pub notes_url: String,
    pub notes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
}

impl Check {
    /// 由发布信息生成检查结果；`download_url` 只在需要浏览器下载的平台提供。
    pub fn from_release(release: Release, download_url: Option<String>) -> Self {
        Check {
            current: env!("CARGO_PKG_VERSION"),
            newer: version(&release.latest) > version(env!("CARGO_PKG_VERSION")),
            notes: release_notes(&release.body, crate::i18n::lang().tag()),
            latest: release.latest,
            notes_url: release.notes_url,
            download_url,
        }
    }
}

/// 版本标记只用于冗余检查；始终顺序请求 API，并原样返回 API 检查结果。
pub fn automatic_check(
    fetch_marker: impl FnOnce() -> Result<Vec<u8>, Error>,
    check: impl FnOnce() -> Result<Check, Error>,
    log: impl Fn(&str),
) -> Result<Check, Error> {
    let marker = match fetch_marker() {
        Ok(data) => {
            let parsed = marker_version(&data);
            if parsed.is_none() {
                log("更新版本标记无效"); // i18n:allow 日志
            }
            parsed
        }
        Err(_) => {
            log("更新版本标记请求失败"); // i18n:allow 日志
            None
        }
    };
    let result = check();
    if let (Some(marker), Ok(result)) = (marker, &result) {
        if Some(marker) != version(&result.latest) {
            log("更新版本标记与 API 不一致，以 API 为准"); // i18n:allow 日志
        }
    }
    result
}

pub fn checksum_for(text: &str, asset: &str) -> Result<String, Error> {
    let mut found = None;
    for line in text.trim_start_matches('\u{feff}').lines() {
        let parts: Vec<_> = line.split_whitespace().collect();
        if parts.len() != 2 || parts[1].trim_start_matches('*') != asset {
            continue;
        }
        if found.is_some()
            || parts[0].len() != 64
            || !parts[0].bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(Error::Checksum);
        }
        found = Some(parts[0].to_ascii_lowercase());
    }
    found.ok_or(Error::Checksum)
}

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub struct UpdateState {
    pub last_check: Option<u64>,
    pub last_notified: Option<String>,
}
impl UpdateState {
    pub fn due(&self, now: u64) -> bool {
        self.last_check
            .is_none_or(|last| now >= last && now - last >= DAY)
    }
    pub fn load(path: &Path) -> Self {
        std::fs::read(path)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default()
    }
    /// 写临时文件后原子替换；Windows 上 std 的 rename 同样覆盖已有目标。
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        use std::io::Write;
        let tmp = path.with_extension("json.tmp");
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(&serde_json::to_vec(self)?)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(tmp, path)
    }
}

/// 每日自动检查：平台只提供 `check` 与日志，开关、每日限制和通知握手共用。
pub struct Automatic {
    enabled: AtomicBool,
    worker: OnceLock<std::thread::Thread>,
    notice: Mutex<Option<(String, mpsc::Sender<bool>)>>,
}

impl Automatic {
    pub const fn new() -> Self {
        Automatic {
            enabled: AtomicBool::new(false),
            worker: OnceLock::new(),
            notice: Mutex::new(None),
        }
    }

    pub fn set(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Release);
        if let Some(thread) = self.worker.get() {
            thread.unpark();
        }
    }

    fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Acquire)
    }

    /// 仅从引擎启动调用。关闭时线程不读状态、不联网；热加载只切换开关并唤醒线程。
    pub fn start(
        &'static self,
        config: PathBuf,
        enabled: bool,
        check: fn() -> Result<Check, Error>,
        log: fn(&str),
    ) {
        self.set(enabled);
        let _ = std::thread::Builder::new()
            .name("mousetroke-update".into())
            .spawn(move || {
                let _ = self.worker.set(std::thread::current());
                let path = config.with_file_name("update-state.json");
                loop {
                    if self.enabled() {
                        self.run_once(&path, check, log);
                    }
                    std::thread::park_timeout(Duration::from_secs(60));
                }
            });
    }

    fn run_once(&self, path: &Path, check: fn() -> Result<Check, Error>, log: fn(&str)) {
        let mut state = UpdateState::load(path);
        if !state.due(now()) {
            return;
        }
        state.last_check = Some(now());
        // 先持久化尝试时间，网络失败或程序重启都不突破一天一次的限制。
        if state.save(path).is_err() {
            log("更新检查状态写入失败，跳过自动检查"); // i18n:allow 日志
            return;
        }
        let Ok(result) = check() else { return };
        if !result.newer || state.last_notified.as_ref() == Some(&result.latest) || !self.enabled()
        {
            return;
        }
        let (tx, rx) = mpsc::channel();
        if let Ok(mut notice) = self.notice.lock() {
            *notice = Some((result.latest.clone(), tx));
        }
        if rx.recv_timeout(Duration::from_secs(5)) == Ok(true) {
            state.last_notified = Some(result.latest);
            let _ = state.save(path);
        }
    }

    /// 主线程定时器取出通知；只有成功提交给界面才记录已通知版本。
    pub fn take_notice(&self) -> Option<(String, mpsc::Sender<bool>)> {
        let notice = self.notice.lock().ok()?.take()?;
        if self.enabled() {
            Some(notice)
        } else {
            let _ = notice.1.send(false);
            None
        }
    }
}

impl Default for Automatic {
    fn default() -> Self {
        Self::new()
    }
}

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
#[path = "update_tests.rs"]
mod tests;
