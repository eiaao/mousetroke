//! 词条一致性测试（docs/I18N-DESIGN.md §7.1）：
//! - 每个语言文件都是合法 JSON 的扁平“字符串 → 字符串”映射；
//! - 键集合与 zh-CN 完全相同（复数键按基础键合并后比较）；
//! - 同一个键在各语言里的占位符集合相同；
//! - 没有空字符串；`_one` / `_other` 后缀的键必须对应 zh-CN 里的 `<base>_other`。

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

type Catalog = BTreeMap<String, String>;

fn i18n_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../i18n")
}

fn load(name: &str) -> Catalog {
    let text = std::fs::read_to_string(i18n_dir().join(name)).unwrap_or_else(|e| panic!("读取 {name}: {e}"));
    let v: serde_json::Value = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{name} 不是合法 JSON: {e}"));
    let serde_json::Value::Object(map) = v else {
        panic!("{name} 顶层应是对象");
    };
    let mut out = Catalog::new();
    for (k, v) in map {
        let serde_json::Value::String(s) = v else {
            panic!("{name} 的 {k:?} 值应是字符串（扁平结构，不嵌套）");
        };
        out.insert(k, s);
    }
    out
}

/// 复数键合并为基础键：`toast.apps_disabled_other` → `toast.apps_disabled`。
/// 后缀为 CLDR 基数类别（one / other / few / many）。
fn base_key(k: &str) -> &str {
    k.strip_suffix("_one")
        .or_else(|| k.strip_suffix("_other"))
        .or_else(|| k.strip_suffix("_few"))
        .or_else(|| k.strip_suffix("_many"))
        .unwrap_or(k)
}

/// 词条里的 `{name}` 占位符集合。只把 `[a-z_][a-z0-9_]*` 视为占位符；
/// `{F4}`、`{ESC}` 这类（按键序列语法的字面示例）不算。
fn placeholders(s: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut rest = s;
    while let Some(i) = rest.find('{') {
        let after = &rest[i + 1..];
        match after.find('}') {
            Some(j) => {
                let name = &after[..j];
                if name.starts_with(|c: char| c.is_ascii_lowercase() || c == '_')
                    && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
                {
                    out.insert(name.to_string());
                }
                rest = &after[j + 1..];
            }
            None => panic!(" '{{' 没有配对的 '}}': {s:?}"),
        }
    }
    out
}

fn language_files() -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(i18n_dir())
        .expect("i18n 目录存在")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".json"))
        .collect();
    names.sort();
    assert!(names.iter().any(|n| n == "zh-CN.json"), "必须有源语言 zh-CN.json");
    names
}

/// 允许空字符串的键（连字符这类"空也是一种翻译"的占位）。
const MAY_BE_EMPTY: &[&str] = &["ui.editor.q_sep"];

#[test]
fn catalogs_are_consistent() {
    let zh = load("zh-CN.json");
    for (k, v) in &zh {
        assert!(!v.is_empty() || MAY_BE_EMPTY.contains(&k.as_str()), "zh-CN 的 {k:?} 是空字符串");
    }
    // zh-CN 自己也要通过占位符合法性检查
    for v in zh.values() {
        placeholders(v);
    }

    for name in language_files() {
        let cat = load(&name);
        let zh_bases: BTreeSet<&str> = zh.keys().map(|k| base_key(k)).collect();
        let cat_bases: BTreeSet<&str> = cat.keys().map(|k| base_key(k)).collect();
        assert_eq!(
            zh_bases, cat_bases,
            "{name} 与 zh-CN 的键集合不一致：多出 {:?}，缺少 {:?}",
            cat_bases.difference(&zh_bases).collect::<Vec<_>>(),
            zh_bases.difference(&cat_bases).collect::<Vec<_>>(),
        );

        for (k, v) in &cat {
            assert!(!v.is_empty() || MAY_BE_EMPTY.contains(&k.as_str()), "{name} 的 {k:?} 是空字符串");
            // 复数后缀的键必须对应 zh-CN 里的 <base>_other（复数规则按 _one/_other 回退）
            if base_key(k) != k.as_str() {
                assert!(
                    zh.contains_key(&format!("{}_other", base_key(k))),
                    "{name} 的复数键 {k:?} 在 zh-CN 里缺少对应的 <base>_other"
                );
            }
            // 占位符按基础键合并后比较
            let zh_ph: BTreeSet<String> = zh
                .iter()
                .filter(|(zk, _)| base_key(zk) == base_key(k))
                .flat_map(|(_, zv)| placeholders(zv))
                .collect();
            let cat_ph: BTreeSet<String> = cat
                .iter()
                .filter(|(zk, _)| base_key(zk) == base_key(k))
                .flat_map(|(_, zv)| placeholders(zv))
                .collect();
            assert_eq!(zh_ph, cat_ph, "{name} 的 {k:?} 占位符与 zh-CN 不一致");
        }
    }
}

/// build.rs 生成的键名常量覆盖全部词条（写一个未知键的引用会直接编译失败，
/// 这里再保证没有“词条存在却生成不出常量”的漏网）。
#[test]
fn generated_constants_cover_catalog() {
    let zh = load("zh-CN.json");
    // 从生成的常量文件反推所有常量值，与词条基础键比较
    let generated = include_str!(concat!(env!("OUT_DIR"), "/i18n_keys.rs"));
    let consts: BTreeSet<&str> = generated
        .lines()
        .filter_map(|l| l.strip_prefix("pub const "))
        .filter_map(|l| l.split_once(": &str = "))
        .map(|(_, v)| v.trim().trim_end_matches(';').trim().trim_matches('"'))
        .collect();
    let bases: BTreeSet<&str> = zh.keys().map(|k| base_key(k)).collect();
    assert_eq!(consts, bases, "build.rs 生成的键名常量与词条不一致");
}

/// 源码里不许写死中文（scripts/i18n_lint.py --check，docs/I18N-DESIGN.md §7.2）。
/// 机器上没有 Python 时跳过并提示（npm run build 前也会跑一次）。
#[test]
fn no_hardcoded_chinese() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let script = root.join("scripts/i18n_lint.py");
    let out = ["python", "python3", "py"]
        .iter()
        .find_map(|py| std::process::Command::new(py).arg(&script).arg("--check").current_dir(&root).output().ok());
    let Some(out) = out else {
        eprintln!("跳过 i18n lint：找不到 Python");
        return;
    };
    assert!(
        out.status.success(),
        "i18n lint 失败：\n{}",
        String::from_utf8_lossy(&out.stdout)
    );
}
