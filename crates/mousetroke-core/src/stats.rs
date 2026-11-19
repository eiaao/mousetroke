//! 使用统计：每次触发（画手势、屏幕边角、按键）记一次，按本地日期累计。
//! 只存本机（与配置同目录的 `stats.json`），只记次数，不记内容。
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// 统计文件放在配置旁边。
pub fn path_for(config: &Path) -> PathBuf {
    config.with_file_name("stats.json")
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Stats {
    /// 本地日期（`YYYY-MM-DD`）→ 当天的计数。
    #[serde(default)]
    pub days: BTreeMap<String, Day>,
    /// 绑定 id → 最近一次触发时的名称与所属作用域；绑定删除后界面仍能显示名字。
    #[serde(default)]
    pub bindings: BTreeMap<String, Seen>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Day {
    /// 绑定 id → 次数。
    #[serde(default)]
    pub bindings: BTreeMap<String, u32>,
    /// 触发时所在程序（完整路径；mac 为 bundle id）→ 次数。
    #[serde(default)]
    pub apps: BTreeMap<String, u32>,
    /// 0–23 点各小时的次数（总是 24 项）。
    #[serde(default)]
    pub hours: Vec<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Seen {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name_key: Option<String>,
    pub scope: String,
}

/// 一次触发。
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub date: String,
    pub hour: u8,
    pub binding: String,
    pub seen: Seen,
    pub app: Option<String>,
}

impl Stats {
    pub fn record(&mut self, e: &Event) {
        let day = self.days.entry(e.date.clone()).or_default();
        *day.bindings.entry(e.binding.clone()).or_default() += 1;
        if let Some(app) = e.app.as_ref().filter(|a| !a.is_empty()) {
            *day.apps.entry(app.clone()).or_default() += 1;
        }
        day.hours.resize(24, 0);
        day.hours[usize::from(e.hour.min(23))] += 1;
        self.bindings.insert(e.binding.clone(), e.seen.clone());
    }

    /// 文件不存在或损坏时从空白开始（统计丢失不影响手势）。
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path).ok().and_then(|text| serde_json::from_str(text.trim_start_matches('\u{feff}')).ok()).unwrap_or_default()
    }

    /// 先写临时文件再改名，设置界面不会读到半截内容。
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_vec(self).map_err(std::io::Error::other)?)?;
        std::fs::rename(&tmp, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(date: &str, hour: u8, binding: &str, app: Option<&str>) -> Event {
        Event { date: date.into(), hour, binding: binding.into(), seen: Seen { name: binding.to_uppercase(), name_key: None, scope: "global".into() }, app: app.map(Into::into) }
    }

    #[test]
    fn counts_per_day_binding_app_and_hour() {
        let mut s = Stats::default();
        s.record(&event("2026-11-18", 9, "a", Some("C:\\chrome.exe")));
        s.record(&event("2026-11-18", 9, "a", None));
        s.record(&event("2026-11-18", 23, "b", Some("C:\\chrome.exe")));
        s.record(&event("2026-11-19", 0, "a", Some("")));
        let d = &s.days["2026-11-18"];
        assert_eq!(d.bindings["a"], 2);
        assert_eq!(d.bindings["b"], 1);
        assert_eq!(d.apps["C:\\chrome.exe"], 2);
        assert_eq!((d.hours.len(), d.hours[9], d.hours[23]), (24, 2, 1));
        assert!(s.days["2026-11-19"].apps.is_empty());
        assert_eq!(s.bindings["b"].name, "B");
    }

    #[test]
    fn saves_atomically_and_tolerates_missing_or_corrupt_files() {
        let dir = std::env::temp_dir().join(format!("mousetroke-stats-{}", std::process::id()));
        let path = path_for(&dir.join("config.json"));
        assert_eq!(Stats::load(&path), Stats::default());
        let mut s = Stats::default();
        s.record(&event("2026-11-18", 9, "a", None));
        s.save(&path).unwrap();
        assert_eq!(Stats::load(&path), s);
        assert!(!path.with_extension("json.tmp").exists());
        std::fs::write(&path, "{broken").unwrap();
        assert_eq!(Stats::load(&path), Stats::default());
        let _ = std::fs::remove_dir_all(dir);
    }
}
