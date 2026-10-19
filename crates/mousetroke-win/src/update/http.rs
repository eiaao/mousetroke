//! WinHTTP 同步下载，只能在更新工作线程调用；每次跳转先校验再连接。

use super::{Error, Source};
use std::ffi::c_void;
use windows::core::PCWSTR;
use windows::Win32::Networking::WinHttp::*;

pub(super) struct Url<'a> {
    pub host: &'a str,
    pub port: u16,
    pub path: &'a str,
    pub secure: bool,
}

pub(super) fn parse_url(text: &str) -> Option<Url<'_>> {
    if text.bytes().any(|b| b <= 32 || b == 127) || text.contains(['\\', '#', '@']) { return None; }
    let (secure, rest) = if let Some(s) = text.strip_prefix("https://") { (true, s) }
        else { (false, text.strip_prefix("http://")?) };
    let (authority, path) = rest.split_once('/').map(|(a, _)| (a, &rest[a.len()..])).unwrap_or((rest, "/"));
    let (host, port) = match authority.split_once(':') {
        Some((h, p)) if !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()) => (h, p.parse().ok()?),
        Some(_) => return None,
        None => (authority, if secure { 443 } else { 80 }),
    };
    if host.is_empty() || port == 0 || !host.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-') { return None; }
    Some(Url { host, port, path, secure })
}

struct Internet(*mut c_void);
impl Internet {
    fn new(p: *mut c_void) -> Result<Self, Error> {
        if p.is_null() { Err(Error::Network) } else { Ok(Self(p)) }
    }
}
impl Drop for Internet {
    fn drop(&mut self) { unsafe { let _ = WinHttpCloseHandle(self.0); } }
}

fn wide(s: &str) -> Vec<u16> { s.encode_utf16().chain([0]).collect() }

fn location(request: &Internet) -> Result<String, Error> {
    let mut buf = [0u16; 8192];
    let mut bytes = std::mem::size_of_val(&buf) as u32;
    unsafe { WinHttpQueryHeaders(request.0, WINHTTP_QUERY_LOCATION, PCWSTR::null(), Some(buf.as_mut_ptr().cast()), &mut bytes, std::ptr::null_mut()) }
        .map_err(|_| Error::Network)?;
    String::from_utf16(&buf[..(bytes as usize / 2).min(buf.len())]).map(|s| s.trim_end_matches('\0').to_string()).map_err(|_| Error::Network)
}

pub(super) fn get(source: &Source, url: &str, api: bool, cap: usize) -> Result<Vec<u8>, Error> {
    let mut url = url.to_string();
    let agent = wide(concat!("Mousetroke/", env!("CARGO_PKG_VERSION")));
    unsafe {
        // 本地测试绕过系统代理；正式请求沿用 Windows 代理配置，不发送登录凭证。
        let access = if source.local.is_some() { WINHTTP_ACCESS_TYPE_NO_PROXY } else { WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY };
        let session = Internet::new(WinHttpOpen(PCWSTR(agent.as_ptr()), access, PCWSTR::null(), PCWSTR::null(), 0))?;
        WinHttpSetTimeouts(session.0, 10_000, 10_000, 10_000, 10_000).map_err(|_| Error::Network)?;
        for _ in 0..=5 {
            if !source.allowed(&url, api) { return Err(Error::Network); }
            let parsed = parse_url(&url).ok_or(Error::Network)?;
            let host = wide(parsed.host);
            let path = wide(parsed.path);
            let connection = Internet::new(WinHttpConnect(session.0, PCWSTR(host.as_ptr()), parsed.port, 0))?;
            let flags = if parsed.secure { WINHTTP_FLAG_SECURE } else { WINHTTP_OPEN_REQUEST_FLAGS(0) };
            let request = Internet::new(WinHttpOpenRequest(connection.0, windows::core::w!("GET"), PCWSTR(path.as_ptr()), PCWSTR::null(), PCWSTR::null(), std::ptr::null(), flags))?;
            WinHttpSetOption(Some(request.0), WINHTTP_OPTION_REDIRECT_POLICY, Some(&WINHTTP_OPTION_REDIRECT_POLICY_NEVER.to_ne_bytes())).map_err(|_| Error::Network)?;
            // 禁止 WinHTTP 自动认证，更新请求没有用户身份信息。
            WinHttpSetOption(Some(request.0), WINHTTP_OPTION_AUTOLOGON_POLICY, Some(&WINHTTP_AUTOLOGON_SECURITY_LEVEL_HIGH.to_ne_bytes())).map_err(|_| Error::Network)?;
            WinHttpSendRequest(request.0, None, None, 0, 0, 0).map_err(|_| Error::Network)?;
            WinHttpReceiveResponse(request.0, std::ptr::null_mut()).map_err(|_| Error::Network)?;
            let mut status = 0u32;
            let mut size = 4;
            WinHttpQueryHeaders(request.0, WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER, PCWSTR::null(), Some((&mut status as *mut u32).cast()), &mut size, std::ptr::null_mut()).map_err(|_| Error::Network)?;
            if matches!(status, 301 | 302 | 303 | 307 | 308) {
                let next = location(&request)?;
                url = if next.starts_with('/') && !next.starts_with("//") {
                    let origin = url.find("://").unwrap() + 3;
                    let end = url[origin..].find('/').map(|i| origin + i).unwrap_or(url.len());
                    format!("{}{next}", &url[..end])
                } else { next };
                continue;
            }
            if status != 200 { return Err(Error::Network); }
            let mut data = Vec::new();
            let mut chunk = [0u8; 32 * 1024];
            // 超时防慢响应，大小上限防止错误页面耗尽内存。
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
            loop {
                let mut read = 0;
                WinHttpReadData(request.0, chunk.as_mut_ptr().cast(), chunk.len() as u32, &mut read).map_err(|_| Error::Network)?;
                if read == 0 { return Ok(data); }
                if data.len() + read as usize > cap || std::time::Instant::now() > deadline { return Err(Error::Network); }
                data.extend_from_slice(&chunk[..read as usize]);
            }
        }
    }
    Err(Error::Network)
}
