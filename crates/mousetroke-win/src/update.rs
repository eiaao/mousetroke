//! 默认每日及手动更新检查；网络、校验、文件替换均不在钩子路径运行。

mod http;

use mousetroke_core::i18n::keys;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const RELEASES_URL: &str = "https://github.com/eiaao/mousetroke/releases/latest";
const API: &str = "https://api.github.com/repos/eiaao/mousetroke/releases/latest";
const DAY: u64 = 24 * 60 * 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error { Network, Checksum, NotWritable, Replace, Restart, Busy, Current }

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
fn version(tag: &str) -> Option<[u64; 3]> {
    let parts: Vec<_> = tag.strip_prefix('v').unwrap_or(tag).split('.').collect();
    if parts.len() != 3 { return None; }
    let mut out = [0; 3];
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) { return None; }
        out[i] = part.parse().ok()?;
    }
    Some(out)
}

#[derive(Debug)]
struct Source {
    api: String,
    local: Option<String>,
}

impl Source {
    fn production() -> Self { Self { api: API.into(), local: None } }

    fn configured() -> Self {
        #[cfg(debug_assertions)]
        if let Ok(base) = std::env::var("MOUSETROKE_UPDATE_BASE") {
            if let Some(source) = Self::local(&base) { return source; }
        }
        Self::production()
    }

    #[cfg(any(test, debug_assertions))]
    fn local(base: &str) -> Option<Self> {
        let base = base.trim_end_matches('/');
        let url = http::parse_url(base)?;
        if url.secure || url.host != "127.0.0.1" || base.contains('?') { return None; }
        let authority = base.strip_prefix("http://")?.split('/').next()?;
        if !authority.contains(':') { return None; }
        Some(Self { api: format!("{base}/repos/eiaao/mousetroke/releases/latest"), local: Some(format!("http://{authority}")) })
    }

    fn allowed(&self, text: &str, api: bool) -> bool {
        let Some(url) = http::parse_url(text) else { return false };
        if let Some(origin) = &self.local {
            return !url.secure && (text == origin || text.strip_prefix(origin).is_some_and(|s| s.starts_with('/')));
        }
        url.secure && url.port == 443 && if api { url.host.eq_ignore_ascii_case("api.github.com") } else {
            ["github.com", "objects.githubusercontent.com", "release-assets.githubusercontent.com"].iter().any(|h| url.host.eq_ignore_ascii_case(h))
        }
    }
}

#[derive(Debug)]
struct Release { latest: String, notes_url: String, body: String, exe: String, sums: String }

fn select_release(data: &[u8], source: &Source) -> Result<Release, Error> {
    let v: serde_json::Value = serde_json::from_slice(data).map_err(|_| Error::Network)?;
    if v["draft"].as_bool() != Some(false) || v["prerelease"].as_bool() != Some(false) { return Err(Error::Network); }
    let tag = v["tag_name"].as_str().filter(|s| version(s).is_some()).ok_or(Error::Network)?;
    let notes = v["html_url"].as_str().ok_or(Error::Network)?;
    if !source.allowed(notes, false) { return Err(Error::Network); }
    let assets = v["assets"].as_array().ok_or(Error::Network)?;
    let asset = |name: &str| -> Result<String, Error> {
        let mut found = assets.iter().filter(|a| a["name"].as_str() == Some(name));
        let url = found.next().and_then(|a| a["browser_download_url"].as_str()).ok_or(Error::Network)?;
        if found.next().is_some() || !source.allowed(url, false) { return Err(Error::Network); }
        Ok(url.into())
    };
    Ok(Release { latest: tag.trim_start_matches('v').into(), notes_url: notes.into(), body: v["body"].as_str().unwrap_or_default().into(), exe: asset("mousetroke.exe")?, sums: asset("SHA256SUMS.txt")? })
}

/// 发布正文只转成纯文本，不渲染远程 HTML 或 Markdown。
fn plain_note(input: &str) -> String {
    let mut text = String::new();
    let mut rest = input;
    while let Some(start) = rest.find('<') {
        text.push_str(&rest[..start]);
        let tag = &rest[start..];
        if tag.starts_with("<!--") {
            rest = tag.find("-->").map(|end| &tag[end + 3..]).unwrap_or_default();
            continue;
        }
        let Some(end) = tag.find('>') else { rest = ""; break };
        let name = tag[1..end].split_whitespace().next().unwrap_or_default().to_ascii_lowercase();
        rest = &tag[end + 1..];
        if name == "script" || name == "style" {
            let closing = format!("</{name}>");
            rest = rest.to_ascii_lowercase().find(&closing).map(|end| &rest[end + closing.len()..]).unwrap_or_default();
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
        if c == '!' && chars.get(i + 1) == Some(&'[') { i += 1; continue; }
        if c == '[' {
            if let Some(end) = (i + 1..chars.len()).find(|&j| chars[j] == ']') {
                if let Some(&open @ ('(' | '[')) = chars.get(end + 1) {
                    let close = if open == '(' { ')' } else { ']' };
                    let mut depth = 1;
                    let mut next = end + 2;
                    while next < chars.len() && depth > 0 {
                        if chars[next] == open { depth += 1; }
                        if chars[next] == close { depth -= 1; }
                        next += 1;
                    }
                    if depth == 0 {
                        out.extend(chars[i + 1..end].iter().filter(|c| !matches!(c, '*' | '_' | '`' | '~')));
                        i = next;
                        continue;
                    }
                }
            }
        }
        if !matches!(c, '*' | '_' | '`' | '~') { out.push(c); }
        i += 1;
    }
    let out = out.split_whitespace().collect::<Vec<_>>().join(" ");
    if out.chars().count() > 300 { out.chars().take(299).chain(['…']).collect() } else { out }
}

fn release_notes(body: &str, language: &str) -> Vec<String> {
    let section = |heading: &str| {
        let mut active = false;
        let mut bullets: Vec<String> = Vec::new();
        for line in body.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                active = trimmed == heading;
            } else if active {
                if let Some(bullet) = ["- ", "* ", "+ "].iter().find_map(|prefix| trimmed.strip_prefix(prefix)) {
                    bullets.push(bullet.into());
                } else if line.starts_with("  ") && !trimmed.is_empty() {
                    if let Some(last) = bullets.last_mut() { last.push(' '); last.push_str(trimmed); }
                }
            }
        }
        bullets.iter().map(|bullet| plain_note(bullet)).filter(|bullet| !bullet.is_empty()).take(12).collect::<Vec<_>>()
    };
    if language.to_ascii_lowercase().split('-').next() == Some("zh") {
        let notes = section("### 中文"); // i18n:allow 更新日志格式标记，非界面文案。
        if !notes.is_empty() { return notes; }
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
}

fn fetch_release(source: &Source) -> Result<Release, Error> {
    select_release(&http::get(source, &source.api, true, 1024 * 1024)?, source)
}

pub fn check() -> Result<Check, Error> {
    crate::log::init();
    crate::log!("检查更新：当前 {}", env!("CARGO_PKG_VERSION"));
    let result = fetch_release(&Source::configured()).map(|r| Check {
        notes: release_notes(&r.body, mousetroke_core::i18n::lang().tag()),
        current: env!("CARGO_PKG_VERSION"), newer: version(&r.latest) > version(env!("CARGO_PKG_VERSION")), latest: r.latest, notes_url: r.notes_url,
    });
    match &result {
        Ok(r) => crate::log!("检查更新成功：最新 {}", r.latest),
        Err(_) => crate::log!("检查更新失败"),
    }
    result
}

fn checksum_line(text: &str) -> Result<String, Error> {
    let mut found = None;
    for line in text.trim_start_matches('\u{feff}').lines() {
        let parts: Vec<_> = line.split_whitespace().collect();
        if parts.len() != 2 || parts[1].trim_start_matches('*') != "mousetroke.exe" { continue; }
        if found.is_some() || parts[0].len() != 64 || !parts[0].bytes().all(|b| b.is_ascii_hexdigit()) { return Err(Error::Checksum); }
        found = Some(parts[0].to_ascii_lowercase());
    }
    found.ok_or(Error::Checksum)
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

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
struct UpdateState { last_check: Option<u64>, last_notified: Option<String> }
impl UpdateState {
    fn due(&self, now: u64) -> bool { self.last_check.is_none_or(|last| now >= last && now - last >= DAY) }
    fn load(path: &Path) -> Self {
        std::fs::read(path).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
    }
    fn save(&self, path: &Path) -> std::io::Result<()> {
        use std::io::Write;
        let tmp = sidecar(path, ".tmp");
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(&serde_json::to_vec(self)?)?;
        file.sync_all()?;
        drop(file);
        // Windows 的 rename 不覆盖目标文件，使用系统的原子替换。
        use windows::Win32::Storage::FileSystem::{MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH};
        let wide = |p: &Path| p.as_os_str().to_string_lossy().encode_utf16().chain([0]).collect::<Vec<_>>();
        unsafe { MoveFileExW(windows::core::PCWSTR(wide(&tmp).as_ptr()), windows::core::PCWSTR(wide(path).as_ptr()), MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH) }.map_err(std::io::Error::other)
    }
}

fn now() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() }

static ENABLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static WORKER: std::sync::OnceLock<std::thread::Thread> = std::sync::OnceLock::new();
static NOTICE: Mutex<Option<(String, mpsc::Sender<bool>)>> = Mutex::new(None);

pub fn set_automatic(enabled: bool) {
    ENABLED.store(enabled, std::sync::atomic::Ordering::Release);
    if let Some(thread) = WORKER.get() { thread.unpark(); }
}

/// 仅从引擎启动调用。关闭时线程不读状态、不联网；热加载只切换开关并唤醒线程。
pub fn start_automatic(config: PathBuf, enabled: bool) {
    set_automatic(enabled);
    let _ = std::thread::Builder::new().name("mousetroke-update".into()).spawn(move || {
        let _ = WORKER.set(std::thread::current());
        let path = config.with_file_name("update-state.json");
        loop {
            if ENABLED.load(std::sync::atomic::Ordering::Acquire) {
                let mut state = UpdateState::load(&path);
                if state.due(now()) {
                    state.last_check = Some(now());
                    // 先持久化尝试时间，网络失败或程序重启都不突破一天一次的限制。
                    if state.save(&path).is_ok() {
                        if let Ok(result) = check() {
                            if result.newer && state.last_notified.as_ref() != Some(&result.latest) && ENABLED.load(std::sync::atomic::Ordering::Acquire) {
                                let (tx, rx) = mpsc::channel();
                                if let Ok(mut notice) = NOTICE.lock() { *notice = Some((result.latest.clone(), tx)); }
                                if rx.recv_timeout(Duration::from_secs(5)) == Ok(true) {
                                    state.last_notified = Some(result.latest);
                                    let _ = state.save(&path);
                                }
                            }
                        }
                    } else { crate::log!("更新检查状态写入失败，跳过自动检查"); }
                }
            }
            std::thread::park_timeout(Duration::from_secs(60));
        }
    });
}

/// 托盘定时器取出通知；只有成功提交给托盘才记录已通知版本。
pub fn take_notice() -> Option<(String, mpsc::Sender<bool>)> {
    let notice = NOTICE.lock().ok()?.take()?;
    if ENABLED.load(std::sync::atomic::Ordering::Acquire) { Some(notice) }
    else { let _ = notice.1.send(false); None }
}

#[cfg(test)]
#[path = "update_tests.rs"]
mod tests;
