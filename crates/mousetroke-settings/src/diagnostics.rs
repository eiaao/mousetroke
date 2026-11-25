//! 诊断信息：版本、系统、显示器、设置摘要与最近日志，反馈问题时一并附上。
//! 内容给开发者看，标签不翻译；路径里的用户名替换掉，其余由用户提交前自行查看删改。
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use mousetroke_core::schema::{Config, ScopeKind, Step};

/// 最多附上的日志行数与字节数（取末尾）。
const LOG_LINES: usize = 400;
const LOG_BYTES: usize = 160 * 1024;

/// 生成报告所需的环境信息（由调用方从平台与窗口收集）。
pub struct Env {
    pub version: String,
    pub os: String,
    pub arch: String,
    pub engine_running: bool,
    pub autostart: bool,
    /// 每个显示器一行：名称、物理尺寸、位置、缩放。
    pub monitors: Vec<String>,
}

impl Env {
    /// 一行环境摘要，用于预填 issue 和邮件。
    pub fn summary(&self) -> String {
        format!("Mousetroke {} · {} ({}) · {} monitor(s): {}", self.version, self.os, self.arch, self.monitors.len(), self.monitors.join("; "))
    }
}

/// 拼出完整报告。config 读不出来时附上错误。
pub fn render(env: &Env, config: Result<&Config, &str>, log: &str, generated: &str) -> String {
    let mut out = String::new();
    out.push_str("== Mousetroke diagnostics ==\n");
    out.push_str(&format!("generated: {generated} (UTC)\n"));
    out.push_str(&format!("version: {}\nos: {}\narch: {}\n", env.version, env.os, env.arch));
    out.push_str(&format!("engine running: {}\nautostart: {}\n", env.engine_running, env.autostart));
    out.push_str("\n== Monitors ==\n");
    for m in &env.monitors {
        out.push_str(m);
        out.push('\n');
    }
    out.push_str("\n== Config ==\n");
    match config {
        Ok(c) => out.push_str(&config_summary(c)),
        Err(e) => out.push_str(&format!("failed to load: {e}\n")),
    }
    out.push_str("\n== Log (latest lines) ==\n");
    out.push_str(log);
    if !log.ends_with('\n') {
        out.push('\n');
    }
    out
}

/// 设置原样列出；手势与规则只给数量和开关，不含用户写的名称与脚本。
fn config_summary(c: &Config) -> String {
    let mut out = String::new();
    out.push_str(&format!("settings: {}\n", serde_json::to_string(&c.settings).unwrap_or_default()));
    out.push_str(&format!("gestures: {}, scopes: {}\n", c.gestures.len(), c.scopes.len()));
    for s in &c.scopes {
        let what = match s.kind {
            ScopeKind::Global => "global".to_string(),
            ScopeKind::App => format!("app {}", s.app.as_ref().map(|a| a.exe.as_str()).unwrap_or("?")),
            ScopeKind::Special => format!("special {}", s.special.as_ref().map(|a| format!("{a:?}").to_lowercase()).unwrap_or_default()),
        };
        let total = s.bindings.len();
        let off = s.bindings.iter().filter(|b| b.disabled).count();
        let edge = s.bindings.iter().filter(|b| matches!(b.pattern.first(), Some(Step::Edge { .. }))).count();
        let edge_on = s.bindings.iter().filter(|b| !b.disabled && matches!(b.pattern.first(), Some(Step::Edge { .. }))).count();
        out.push_str(&format!(
            "- {what}: disabled={}, inherit_global={}, fullscreen={:?}, bindings={total} (off {off}), edge triggers={edge} (on {edge_on})\n",
            s.disabled,
            s.inherit_global,
            s.fullscreen_policy,
        ));
    }
    out
}

/// 当前日志与轮转出的旧日志合起来，取末尾若干行。
pub fn log_tail(current: &str, old: &str) -> String {
    let joined;
    let text = if current.lines().count() >= LOG_LINES || old.is_empty() {
        current
    } else {
        joined = format!("{old}{}{current}", if old.ends_with('\n') { "" } else { "\n" });
        &joined
    };
    let mut start = text.len().saturating_sub(LOG_BYTES);
    while !text.is_char_boundary(start) {
        start += 1;
    }
    let text = &text[start..];
    let lines: Vec<&str> = text.lines().collect();
    // 字节截断可能切在行中，丢掉不完整的第一行
    let skip = usize::from(start > 0 && lines.len() > 1);
    let from = lines.len().saturating_sub(LOG_LINES).max(skip);
    lines[from..].join("\n")
}

/// 把用户主目录换成 `~`，其余路径里的用户名换成 `<user>`（不区分大小写）。
pub fn redact(text: &str, home: Option<&str>, user: Option<&str>) -> String {
    let mut out = text.to_string();
    if let Some(home) = home.filter(|h| h.len() > 3) {
        out = replace_ci(&out, home, "~", false);
        out = replace_ci(&out, &home.replace('\\', "/"), "~", false);
        out = replace_ci(&out, &home.replace('\\', "\\\\"), "~", false);
    }
    if let Some(user) = user.filter(|u| !u.is_empty()) {
        for sep in ['\\', '/'] {
            out = replace_ci(&out, &format!("{sep}{user}"), &format!("{sep}<user>"), true);
        }
    }
    out
}

/// 按 ASCII 大小写不敏感替换（非 ASCII 字符须完全一致）。whole 为真时，匹配后面不能紧跟字母数字
/// （用户名 `ad` 不会误伤 `\Adobe`）。
fn replace_ci(text: &str, from: &str, to: &str, whole: bool) -> String {
    if from.is_empty() {
        return text.to_string();
    }
    let lower = text.to_ascii_lowercase();
    let needle = from.to_ascii_lowercase();
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for (i, _) in lower.match_indices(&needle) {
        let end = i + needle.len();
        if i < last || whole && text[end..].chars().next().is_some_and(char::is_alphanumeric) {
            continue;
        }
        out.push_str(&text[last..i]);
        out.push_str(to);
        last = end;
    }
    out.push_str(&text[last..]);
    out
}

/// UTC 时间：(`2026-11-24 07:30:05`, `20261124-073005`)。
pub fn timestamp(now: SystemTime) -> (String, String) {
    let secs = now.duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let (days, rem) = ((secs / 86_400) as i64, secs % 86_400);
    let (h, m, s) = (rem / 3600, rem / 60 % 60, rem % 60);
    // Howard Hinnant 的 civil_from_days
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(mo <= 2);
    (format!("{y:04}-{mo:02}-{d:02} {h:02}:{m:02}:{s:02}"), format!("{y:04}{mo:02}{d:02}-{h:02}{m:02}{s:02}"))
}

/// 读文本文件，失败或不存在时为空（日志可能含非 UTF-8 字节，按有损解码）。
pub fn read_lossy(path: &Path) -> String {
    std::fs::read(path).map(|b| String::from_utf8_lossy(&b).into_owned()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_formats_utc() {
        let t = UNIX_EPOCH + std::time::Duration::from_secs(1_795_505_405);
        assert_eq!(timestamp(t).0, "2026-11-24 07:30:05");
        assert_eq!(timestamp(t).1, "20261124-073005");
        assert_eq!(timestamp(UNIX_EPOCH).0, "1970-01-01 00:00:00");
    }

    #[test]
    fn redact_hides_home_and_user_name() {
        let text = r"config C:\Users\Alice\AppData\Roaming\mousetroke and c:/users/alice/x and D:\work\ALICE\y, alice wrote";
        let out = redact(text, Some(r"C:\Users\Alice"), Some("Alice"));
        assert_eq!(out, r"config ~\AppData\Roaming\mousetroke and ~/x and D:\work\<user>\y, alice wrote");
        assert_eq!(redact(r"D:\ad\x D:\Adobe", None, Some("ad")), r"D:\<user>\x D:\Adobe");
    }

    #[test]
    fn log_tail_keeps_latest_lines_and_uses_old_log_when_short() {
        let current: String = (0..10).map(|i| format!("new {i}\n")).collect();
        let old: String = (0..1000).map(|i| format!("old {i}\n")).collect();
        let tail = log_tail(&current, &old);
        let lines: Vec<&str> = tail.lines().collect();
        assert_eq!(lines.len(), LOG_LINES);
        assert_eq!(lines.last(), Some(&"new 9"));
        assert_eq!(lines[LOG_LINES - 11], "old 999");
        assert_eq!(log_tail("a\nb\n", ""), "a\nb");
    }

    #[test]
    fn log_tail_caps_bytes_without_partial_first_line() {
        let long: String = (0..300).map(|i| format!("{i:04} {}\n", "x".repeat(1000))).collect();
        let tail = log_tail(&long, "");
        assert!(tail.len() <= LOG_BYTES);
        assert!(tail.lines().all(|l| l.len() == 1005));
        assert!(tail.ends_with(&format!("0299 {}", "x".repeat(1000))));
    }

    #[test]
    fn report_lists_scopes_without_names() {
        let config = Config::default();
        let env = Env { version: "1.2.3".into(), os: "TestOS".into(), arch: "x86_64".into(), engine_running: true, autostart: false, monitors: vec!["m1 1920x1080 @ (0,0) scale 1.5".into()] };
        let report = render(&env, Ok(&config), "line", "2026-01-01 00:00:00");
        assert!(report.contains("version: 1.2.3"));
        assert!(report.contains("m1 1920x1080"));
        assert!(report.contains("settings: {"));
        assert!(report.ends_with("== Log (latest lines) ==\nline\n"));
        for s in &config.scopes {
            assert!(s.name.is_empty() || !report.contains(&format!("{}:", s.name)));
        }
        assert!(env.summary().starts_with("Mousetroke 1.2.3 · TestOS (x86_64) · 1 monitor(s)"));
    }
}
