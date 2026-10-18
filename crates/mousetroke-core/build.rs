//! 从 i18n/zh-CN.json 生成键名常量（`i18n::keys`）。
//!
//! 代码里只写 `keys::TRAY_MENU_SETTINGS` 这样的常量，不写字符串字面量：
//! 写错键名、或词条删除后仍有代码引用，都会编译失败。
//! 复数词条（`_one` / `_other` 后缀）只为基础键生成常量。

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let dir = Path::new(env::var("CARGO_MANIFEST_DIR").unwrap().as_str()).join("../../i18n");
    println!("cargo:rerun-if-changed={}", dir.display());
    let zh = fs::read_to_string(dir.join("zh-CN.json")).expect("读取 i18n/zh-CN.json");
    let map: BTreeMap<String, String> = serde_json::from_str(&zh).expect("解析 i18n/zh-CN.json（应为扁平的字符串映射）");

    let mut out = String::from("// 由 build.rs 从 i18n/zh-CN.json 生成，勿手改。\n");
    // 复数变体（CLDR 后缀 _one/_other/_few/_many）共享基础键，先归并
    let bases: std::collections::BTreeSet<&str> = map
        .keys()
        .map(|k| {
            k.strip_suffix("_one")
                .or_else(|| k.strip_suffix("_other"))
                .or_else(|| k.strip_suffix("_few"))
                .or_else(|| k.strip_suffix("_many"))
                .unwrap_or(k.as_str())
        })
        .collect();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for base in bases {
        let const_name: String = base
            .chars()
            .map(|c| if c == '.' || c == '_' { '_' } else { c.to_ascii_uppercase() })
            .collect();
        assert!(
            seen.insert(const_name.clone()),
            "i18n 键名常量冲突：{base:?}（点号/下划线归一后与其他键相同）"
        );
        out.push_str(&format!("pub const {const_name}: &str = \"{base}\";\n"));
    }
    let out_dir = env::var("OUT_DIR").unwrap();
    fs::write(Path::new(&out_dir).join("i18n_keys.rs"), out).expect("写入 i18n_keys.rs");
}
