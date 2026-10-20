//! 禁用 curl 配置和自动重定向；每一跳在连接之前验证协议、端口和主机。
use super::{Error, Source};
use crate::update_helpers::{redirect, response};
use std::io::Read;
use std::process::{Command, Stdio};

pub(super) fn get(source: &Source, url: &str, api: bool, cap: usize) -> Result<Vec<u8>, Error> {
    let mut url = redirect(source, url, api)?;
    for hop in 0..=5 {
        let protocol = if source.local.is_some() {
            "=http"
        } else {
            "=https"
        };
        let mut command = Command::new("/usr/bin/curl");
        // -q 必须是首参数；不使用 -L，防止重定向绕过逐跳校验。
        command.args([
            "-q",
            "--globoff",
            "--proto",
            protocol,
            "--proto-redir",
            protocol,
            "--max-redirs",
            "0",
            "--connect-timeout",
            "10",
            "--max-time",
            if api { "30" } else { "120" },
            "--max-filesize",
            &cap.to_string(),
            "--silent",
            "--fail",
            "--user-agent",
            concat!("Mousetroke/", env!("CARGO_PKG_VERSION")),
            "--write-out",
            "\n%{http_code}\n%{redirect_url}",
        ]);
        if api {
            command.args(["--header", "Accept: application/vnd.github+json"]);
        }
        if source.local.is_some() {
            command.args(["--noproxy", "*"]);
        }
        let mut child = command
            .args(["--url", &url])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| Error::Network)?;
        let mut data = Vec::new();
        // 即使无 Content-Length 或使用分块编码，也限制实际读取；尾字段另外预留有界空间。
        let limit = cap + 8192 + 16;
        let read = child
            .stdout
            .take()
            .ok_or(Error::Network)?
            .take(limit as u64)
            .read_to_end(&mut data);
        if read.is_err() || data.len() >= limit {
            let _ = child.kill();
            let _ = child.wait();
            return Err(Error::Network);
        }
        if !child.wait().map_err(|_| Error::Network)?.success() {
            return Err(Error::Network);
        }
        let (body, status, next) = response(&data, cap)?;
        if status == 200 {
            return Ok(body.to_vec());
        }
        if api || hop == 5 || !matches!(status, 301 | 302 | 303 | 307 | 308) {
            return Err(Error::Network);
        }
        // redirect_url 由 curl 解析为绝对地址，包含相对 Location 的情况；日志不写签名 URL。
        url = redirect(source, next, false)?;
        crate::log!("更新资源重定向：第 {} 跳，目标已通过白名单", hop + 1);
    }
    Err(Error::Network)
}
