//! 跨平台发布协议；网络、签名摘要与原子文件写入由平台层完成。
use crate::i18n::keys;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

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
    #[cfg(unix)]
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

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
#[path = "update_tests.rs"]
mod tests;
