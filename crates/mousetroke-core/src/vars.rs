//! 变量系统：命令文本中的 `%MS_*%` 占位符（旧格式里写作 `%<旧前缀>_*%`，导入时转换）。
//!
//! 旧格式导入时占位符自动转换（[`convert_legacy_placeholders`]）：
//! 例如旧格式的选中文字、目标进程号占位符 → `%MS_SELECTED_TEXT%`、`%MS_PID%` …

use std::collections::BTreeMap;

/// 变量表。
pub type Vars = BTreeMap<String, String>;

/// 旧格式占位符 → 我们的变量名。
const LEGACY_TO_MS: &[(&str, &str)] = &[
    ("WG_TARGET_PID", "MS_PID"),
    ("WG_TARGET_EXE", "MS_EXE"),
    ("WG_TARGET_WID", "MS_WID"),
    ("WG_MOUSE_X", "MS_MOUSE_X"),
    ("WG_MOUSE_Y_FLIP", "MS_MOUSE_Y_FLIP"),
    ("WG_MOUSE_Y", "MS_MOUSE_Y"),
    ("WG_TARGET_APP_NAME", "MS_APP_NAME"),
    ("WG_TARGET_WIN_NAME", "MS_WIN_NAME"),
    ("WG_TARGET_MODE", "MS_TARGET_MODE"),
    ("WG_SELECTED_TEXT", "MS_SELECTED_TEXT"),
    ("WG_CLIPBOARD_TEXT", "MS_CLIPBOARD_TEXT"),
];

/// 把旧格式写法（`{<旧前缀>_X}` 或 `%<旧前缀>_X%`）转换为 `%MS_*%`。
pub fn convert_legacy_placeholders(input: &str) -> String {
    let mut out = input.to_string();
    for (old, ms) in LEGACY_TO_MS {
        out = out.replace(&format!("{{{old}}}"), &format!("%{ms}%"));
        out = out.replace(&format!("%{old}%"), &format!("%{ms}%"));
    }
    out
}

/// 展开 `%KEY%` 占位符（未定义的保留原样，便于发现拼写问题）。
pub fn expand(template: &str, vars: &Vars) -> String {
    let mut out = String::with_capacity(template.len());
    let bytes = template.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            if let Some(end) = template[i + 1..].find('%') {
                let name = &template[i + 1..i + 1 + end];
                if let Some(v) = vars.get(name) {
                    out.push_str(v);
                    i += end + 2;
                    continue;
                }
            }
        }
        // 按 UTF-8 字符推进
        let ch = template[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// URL 查询参数编码（非保留字符全部百分号编码，UTF-8 字节）。
pub fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// 展开后是否直接是一个网址（是则直接打开，不套模板——旧格式 WebSearch 语义）。
pub fn looks_like_url(s: &str) -> bool {
    let s = s.trim();
    s.starts_with("http://") || s.starts_with("https://") || s.starts_with("www.")
}

/// 网址动作：选中文字本身是网址就直接打开（`www.` 补 https://），否则把它编码后代入模板。
pub fn expand_url(template: &str, vars: &Vars) -> String {
    if let Some(text) = vars.get("MS_SELECTED_TEXT") {
        if looks_like_url(text) {
            let text = text.trim();
            return if text.starts_with("www.") {
                format!("https://{text}")
            } else {
                text.into()
            };
        }
        return expand(&template.replace("%MS_SELECTED_TEXT%", &url_encode(text)), vars);
    }
    expand(template, vars)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars() -> Vars {
        [("MS_PID".to_string(), "1234".to_string()),
         ("MS_SELECTED_TEXT".to_string(), "mousetroke 手势".to_string())]
            .into_iter()
            .collect()
    }

    #[test]
    fn converts_official_placeholders() {
        assert_eq!(
            convert_legacy_placeholders("https://baidu.com/s?wd={WG_SELECTED_TEXT}"),
            "https://baidu.com/s?wd=%MS_SELECTED_TEXT%"
        );
        assert_eq!(
            convert_legacy_placeholders("taskkill /F /PID %WG_TARGET_PID%"),
            "taskkill /F /PID %MS_PID%"
        );
    }

    #[test]
    fn expands_variables() {
        assert_eq!(expand("kill %MS_PID% now", &vars()), "kill 1234 now");
        // 未定义保留原样
        assert_eq!(expand("%MS_UNKNOWN%", &vars()), "%MS_UNKNOWN%");
        // 中文与孤立的 % 不受影响
        assert_eq!(expand("100% 完成", &vars()), "100% 完成");
    }

    #[test]
    fn url_encodes_utf8() {
        assert_eq!(url_encode("a b"), "a%20b");
        assert_eq!(url_encode("手势"), "%E6%89%8B%E5%8A%BF");
    }

    #[test]
    fn url_actions_encode_queries_and_open_selected_urls() {
        let mut values: Vars = [
            ("MS_SELECTED_TEXT".to_string(), "a b&中文".to_string()),
            ("MS_PID".to_string(), "42".to_string()),
        ]
        .into_iter()
        .collect();
        assert_eq!(
            expand_url("https://example.com?q=%MS_SELECTED_TEXT%&pid=%MS_PID%", &values),
            "https://example.com?q=a%20b%26%E4%B8%AD%E6%96%87&pid=42"
        );
        values.insert("MS_SELECTED_TEXT".into(), " https://example.com/?x=1 ".into());
        assert_eq!(expand_url("unused", &values), "https://example.com/?x=1");
        values.insert("MS_SELECTED_TEXT".into(), "www.example.com".into());
        assert_eq!(expand_url("unused", &values), "https://www.example.com");
    }

    #[test]
    fn detects_urls() {
        assert!(looks_like_url("https://example.com"));
        assert!(looks_like_url("www.example.com"));
        assert!(!looks_like_url("搜索词"));
    }
}
