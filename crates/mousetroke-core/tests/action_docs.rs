//! 注册表是目录的唯一来源；UPDATE_ACTION_DOCS=1 时只重写两个标记之间的内容。
use mousetroke_core::{
    i18n::Lang,
    presets::actions::{Implementation, ParamDefault, ParamKind, Preset, REGISTRY},
};
use std::{collections::BTreeMap, fmt::Write, path::Path};

const START: &str = "<!-- action-library:generated:start -->";
const END: &str = "<!-- action-library:generated:end -->";

fn cell(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('|', "&#124;")
        .replace('\n', "<br>")
}

fn parameters(preset: &Preset, lang: Lang, catalog: &BTreeMap<String, String>) -> String {
    let defaults = preset.default_params(lang);
    let zh = lang == Lang::ZhCn;
    preset
        .params
        .iter()
        .map(|param| {
            let label = cell(&catalog[param.label_key]);
            let default = defaults[param.name].as_str().unwrap();
            let choices = if let ParamKind::Choice { choices } = param.kind {
                choices
                    .iter()
                    .map(|choice| {
                        format!("{} (`{}`)", cell(&catalog[choice.label_key]), choice.value)
                    })
                    .collect::<Vec<_>>()
                    .join(" / ")
            } else {
                String::new()
            };
            let default_label = if let ParamKind::Choice { choices } = param.kind {
                cell(
                    &catalog[choices
                        .iter()
                        .find(|choice| choice.value == default)
                        .unwrap()
                        .label_key],
                )
            } else {
                format!("`{}`", cell(default))
            };
            let detail = if default.is_empty() {
                if zh {
                    "必填".into()
                } else {
                    "required".into()
                }
            } else if matches!(param.default, ParamDefault::SearchEngine) {
                if zh {
                    "简体中文界面默认百度，其他语言默认 Google".into()
                } else {
                    "Baidu for Simplified Chinese UI; Google otherwise".into()
                }
            } else if zh {
                format!("默认 {default_label}")
            } else {
                format!("default {default_label}")
            };
            let condition = param.visible_when.map(|condition| {
                let field = preset.params.iter().find(|p| p.name == condition.param).unwrap();
                let ParamKind::Choice { choices } = field.kind else { panic!() };
                let choice = choices.iter().find(|c| c.value == condition.equals).unwrap();
                if zh { format!("；仅当{}为{}时", cell(&catalog[field.label_key]), cell(&catalog[choice.label_key])) }
                else { format!("; only when {} is {}", cell(&catalog[field.label_key]), cell(&catalog[choice.label_key])) }
            }).unwrap_or_default();
            format!(
                "{label} (`{}`): {}{detail}{condition}",
                param.name,
                if choices.is_empty() {
                    String::new()
                } else {
                    format!("{choices}; ")
                }
            )
        })
        .collect::<Vec<_>>()
        .join("<br>")
}

fn availability(implementation: Implementation, note: Option<&str>, catalog: &BTreeMap<String, String>) -> String {
    let availability: String = match implementation {
        Implementation::Implemented(_) => "✓".into(),
        Implementation::Unsupported { reason_key } | Implementation::Pending { reason_key } => {
            cell(&catalog[reason_key])
        }
    };
    match note { Some(key) if matches!(implementation, Implementation::Implemented(_)) => format!("{availability}<br>{}", cell(&catalog[key])), _ => availability }
}

fn catalogue(lang: Lang, catalog: &BTreeMap<String, String>) -> String {
    let mut out = String::new();
    let mut previous = None;
    for preset in REGISTRY {
        if previous != Some(preset.category) {
            let category = serde_json::to_value(preset.category).unwrap();
            writeln!(
                out,
                "\n### {}\n",
                catalog[&format!("preset.category.{}", category.as_str().unwrap())]
            )
            .unwrap();
            if lang == Lang::ZhCn {
                out.push_str("| 动作 / ID | 用途与适用范围 | 参数 | Windows | Mac |\n|---|---|---|---|---|\n");
            } else {
                out.push_str("| Action / ID | What it does / where it works | Parameters | Windows | Mac |\n|---|---|---|---|---|\n");
            }
            previous = Some(preset.category);
        }
        writeln!(
            out,
            "| {}<br>`{}` | {} | {} | {} | {} |",
            cell(&catalog[preset.name_key]),
            preset.id,
            cell(&catalog[preset.description_key]),
            if preset.params.is_empty() {
                "—".into()
            } else {
                parameters(preset, lang, catalog)
            },
            availability(preset.windows, preset.platform_notes.windows, catalog),
            availability(preset.macos, preset.platform_notes.macos, catalog)
        )
        .unwrap();
    }
    out.push('\n');
    out
}

#[test]
fn action_library_tables_match_registry() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (dir, tag, lang) in [("en", "en-US", Lang::EnUs), ("zh-CN", "zh-CN", Lang::ZhCn)] {
        let catalog: BTreeMap<String, String> = serde_json::from_str(
            &std::fs::read_to_string(root.join(format!("i18n/{tag}.json"))).unwrap(),
        )
        .unwrap();
        let path = root.join(format!("docs/guide/{dir}/action-library.md"));
        let doc = std::fs::read_to_string(&path)
            .unwrap()
            .replace("\r\n", "\n");
        assert_eq!(doc.matches(START).count(), 1);
        assert_eq!(doc.matches(END).count(), 1);
        let (before, rest) = doc.split_once(START).unwrap();
        let (_, after) = rest.split_once(END).unwrap();
        let expected = format!("{before}{START}\n{}{END}{after}", catalogue(lang, &catalog));
        if std::env::var("UPDATE_ACTION_DOCS").as_deref() == Ok("1") {
            std::fs::write(&path, expected).unwrap();
        } else {
            assert_eq!(doc, expected, "{dir}/action-library.md is stale; run UPDATE_ACTION_DOCS=1 cargo test -p mousetroke-core --test action_docs");
        }
    }
}
