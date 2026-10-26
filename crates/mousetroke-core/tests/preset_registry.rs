use mousetroke_core::{caps::{self, Platform}, i18n::Lang, keyseq, presets::actions::*, schema::Action};
use serde_json::{json, Value};
use mousetroke_core::presets::native::{NativeAction, PreparedAction, TextMethod};
use std::collections::{BTreeMap, BTreeSet};

const LANGS: &[(&str, Lang)] = &[
    ("zh-CN", Lang::ZhCn), ("en-US", Lang::EnUs), ("es-ES", Lang::EsEs), ("fr-FR", Lang::FrFr),
    ("de-DE", Lang::DeDe), ("ja-JP", Lang::JaJp), ("pt-BR", Lang::PtBr), ("ru-RU", Lang::RuRu),
    ("it-IT", Lang::ItIt), ("ko-KR", Lang::KoKr),
];

// 仅第二期速查浮层保留占位。
const PENDING: &[&str] = &["mousetroke.cheatsheet"];

#[test]
fn registry_is_complete_localized_and_has_valid_defaults() {
    assert_eq!(REGISTRY.len(), 87);
    assert_eq!(REGISTRY.iter().map(|p| p.id).collect::<BTreeSet<_>>().len(), REGISTRY.len());
    let mut categories = BTreeMap::new();
    for preset in REGISTRY {
        *categories.entry(serde_json::to_value(preset.category).unwrap().as_str().unwrap().to_owned()).or_insert(0) += 1;
    }
    assert_eq!(categories, BTreeMap::from([
        ("browser".into(), 17), ("edit".into(), 18), ("text".into(), 6), ("window".into(), 11),
        ("desktop".into(), 20), ("media".into(), 8), ("open".into(), 4), ("mousetroke".into(), 3),
    ]));
    for (tag, lang) in LANGS {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../i18n").join(format!("{tag}.json"));
        let catalog: BTreeMap<String, String> = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let translated = |key: &str| assert!(catalog.get(key).is_some_and(|v| !v.trim().is_empty()), "{tag}: {key}");
        for preset in REGISTRY {
            for key in [preset.name_key, preset.description_key, preset.aliases_key] { translated(key); }
            for key in [preset.platform_notes.windows, preset.platform_notes.macos].into_iter().flatten() { translated(key); }
            let defaults = preset.default_params(*lang);
            validate_params(preset, &defaults).unwrap();
            assert_eq!(defaults.len(), preset.params.len());
            let mut names = BTreeSet::new();
            for param in preset.params {
                assert!(names.insert(param.name));
                translated(param.label_key);
                assert!(defaults[param.name].is_string());
                if let ParamKind::Choice { choices } = param.kind {
                    assert!(!choices.is_empty());
                    assert_eq!(choices.iter().map(|c| c.value).collect::<BTreeSet<_>>().len(), choices.len());
                    for choice in choices { translated(choice.label_key); }
                }
            }
            for platform in [Platform::Windows, Platform::Macos] {
                match preset.implementation(platform) {
                    Implementation::Implemented(_) => {},
                    Implementation::Unsupported { reason_key } => translated(reason_key),
                    Implementation::Pending { reason_key } => { assert!(PENDING.contains(&preset.id), "{}", preset.id); translated(reason_key); }
                }
            }
        }
    }
    assert_eq!(find("text.type").unwrap().default_params(Lang::EnUs)["method"], "paste");
    let ParamKind::Choice { choices } = find("text.type").unwrap().params[1].kind else { panic!() };
    assert_eq!(choices.iter().map(|c| c.value).collect::<Vec<_>>(), ["paste", "type"]);
}

#[test]
fn app_identity_activation_is_explicit_and_never_skips_launch_arguments() {
    for platform in [Platform::Windows, Platform::Macos] {
        let params = serde_json::from_value(json!({"app":"editor"})).unwrap();
        let mut preset = expand("open.app", platform, &params, Lang::EnUs).unwrap();
        assert!(preset.activates_app_identity());
        if let Action::LaunchApp { args, .. } = &mut preset { *args = Some("document.txt".into()); }
        assert!(!preset.activates_app_identity());
        if let Action::LaunchApp { args, .. } = &mut preset { *args = Some(String::new()); }
        assert!(!preset.activates_app_identity());
    }
    for exe in ["explorer.exe", "notepad.exe", "code.exe", "com.apple.TextEdit"] {
        for args in [Value::Null, json!("document.txt")] {
            let legacy: Action = serde_json::from_value(json!({"type":"launch_app","exe":exe,"args":args})).unwrap();
            assert!(!legacy.activates_app_identity());
            assert!(serde_json::to_value(legacy).unwrap().get("activate_by_identity").is_none());
        }
    }
}

#[test]
fn conditional_params_only_validate_for_their_visible_option() {
    let search = find("text.search").unwrap();
    let hidden: Params = serde_json::from_value(json!({"engine":"baidu","url":false})).unwrap();
    assert!(validate_params(search, &hidden).is_ok());
    assert!(expand(search.id, Platform::Windows, &hidden, Lang::EnUs).is_ok());
    let visible: Params = serde_json::from_value(json!({"engine":"custom","url":false})).unwrap();
    assert!(validate_params(search, &visible).is_err());
    let date = find("text.datetime").unwrap();
    assert!(!date.params[1].visible(&date.default_params(Lang::EnUs)));
    assert_eq!(find("text.translate").unwrap().params.iter().filter(|p| p.visible_when.is_some()).count(), 0);
    assert_eq!(find("text.type").unwrap().params.iter().filter(|p| p.visible_when.is_some()).count(), 0);
}

#[test]
fn layout_excludes_both_shells_without_blocking_normal_app_windows() {
    use mousetroke_core::action_geometry::layout_target_allowed as allowed;
    for class in ["Progman", "WorkerW", "SHELLDLL_DefView", "Shell_TrayWnd", "Shell_SecondaryTrayWnd"] {
        assert!(!allowed(Platform::Windows, false, class, 0));
    }
    assert!(!allowed(Platform::Windows, true, "", 0));
    assert!(allowed(Platform::Windows, false, "CabinetWClass", 0));
    assert!(!allowed(Platform::Macos, false, "com.apple.dock", 0));
    assert!(!allowed(Platform::Macos, false, "com.apple.finder", -2147483623));
    assert!(!allowed(Platform::Macos, true, "com.apple.finder", 0));
    assert!(allowed(Platform::Macos, false, "com.apple.finder", 0));
    assert!(allowed(Platform::Macos, false, "com.apple.TextEdit", 0));
}

#[test]
fn each_platform_has_only_explicit_unsupported_or_pending_actions() {
    for (platform, implemented, unsupported) in [
        (Platform::Windows, 84, vec!["app.hide", "system.force_quit_dialog"]),
        (Platform::Macos, 83, vec!["window.toggle_topmost", "desktop.clipboard_history", "desktop.notifications"]),
    ] {
        let mut count = 0;
        for preset in REGISTRY {
            let mut params = preset.default_params(Lang::EnUs);
            if matches!(preset.id, "open.folder" | "open.file") { params.insert("path".into(), json!("folder with spaces")); }
            if preset.id == "open.app" { params.insert("app".into(), json!(if platform == Platform::Windows { "editor.exe" } else { "com.apple.TextEdit" })); }
            if preset.id == "text.type" { params.insert("text".into(), json!("sample")); }
            match preset.implementation(platform) {
                Implementation::Implemented(_) => {
                    count += 1;
                    let prepared = resolve(preset.id, platform, &params, Lang::EnUs).unwrap();
                    let can_expand = match platform { Platform::Windows => preset.advanced_expansion.windows, Platform::Macos => preset.advanced_expansion.macos };
                    assert_eq!(can_expand, matches!(prepared, PreparedAction::Advanced(_)));
                    let action = match prepared {
                        PreparedAction::Advanced(action) => action,
                        PreparedAction::Native(action) => {
                            assert_eq!(action.id(), preset.id);
                            assert_eq!(expand(preset.id, platform, &params, Lang::EnUs), Err(PresetError::NativeOnly));
                            continue;
                        }
                    };
                    assert!(!matches!(action, Action::Preset { .. } | Action::Unknown(_) | Action::Lua { .. }));
                    if let Action::KeySequence { keys, .. } = &action {
                        let seq = keyseq::parse(keys, false).unwrap();
                        assert!(caps::keys_supported(&seq, platform), "{}: {keys}", preset.id);
                    }
                    if matches!(preset.id, "text.search" | "text.translate" | "text.open_selected_url") {
                        assert!(matches!(action, Action::OpenUrl { url_template, .. } if url_template.contains("%MS_SELECTED_TEXT%")));
                    }
                }
                Implementation::Unsupported { reason_key } => {
                    assert!(unsupported.contains(&preset.id), "{}", preset.id);
                    assert_eq!(expand(preset.id, platform, &params, Lang::EnUs), Err(PresetError::Unsupported(reason_key)));
                }
                Implementation::Pending { reason_key } => {
                    assert!(PENDING.contains(&preset.id), "{}", preset.id);
                    assert_eq!(expand(preset.id, platform, &params, Lang::EnUs), Err(PresetError::Pending(reason_key)));
                }
            }
        }
        assert_eq!(count, implemented);
        for id in PENDING { assert!(matches!(find(id).unwrap().implementation(platform), Implementation::Pending { .. })); }
    }
}

#[test]
fn locale_defaults_and_translation_targets_follow_ui_language() {
    for (tag, lang) in LANGS {
        let search = expand("text.search", Platform::Windows, &Params::new(), *lang).unwrap();
        let Action::OpenUrl { url_template, .. } = search else { panic!() };
        assert!(url_template.contains(if *lang == Lang::ZhCn { "www.baidu.com/s?wd=" } else { "www.google.com/search?q=" }));
        for service in ["google", "deepl", "bing"] {
            let params = serde_json::from_value(json!({"service":service})).unwrap();
            let translated = expand("text.translate", Platform::Macos, &params, *lang).unwrap();
            let Action::OpenUrl { url_template, .. } = translated else { panic!() };
            let target = match (service, lang) { ("google", Lang::ZhCn) => "zh-CN", ("bing", Lang::ZhCn) => "zh-Hans", _ => tag.split('-').next().unwrap() };
            assert!(url_template.contains(&format!("/{target}/")) || url_template.contains(&format!("={target}&")), "{url_template}");
        }
    }
    let params = serde_json::from_value(json!({"service":"bing", "target_language":"ja-JP"})).unwrap();
    assert!(matches!(expand("text.translate", Platform::Windows, &params, Lang::EnUs).unwrap(), Action::OpenUrl { url_template, .. } if url_template.contains("to=ja&")));
}

#[test]
fn invalid_parameters_are_rejected_without_losing_future_fields() {
    for (id, params) in [
        ("text.search", json!({"engine":"invalid"})), ("text.search", json!({"engine":true})),
        ("text.search", json!({"engine":"custom", "url":"https://example.com/"})),
        ("text.search", json!({"engine":"custom", "url":"file:///%MS_SELECTED_TEXT%"})),
        ("text.translate", json!({"target_language":"unknown"})),
        ("open.folder", json!({"path":""})), ("open.file", json!({"path":"bad\u{0000}path"})),
        ("open.app", json!({"app":""})),
        ("text.type", json!({"text":""})), ("text.type", json!({"text":"bad\u{0000}text"})),
        ("text.type", json!({"text":"ok", "method":"invalid"})),
        ("text.datetime", json!({"format":"invalid"})),
        ("text.datetime", json!({"format":"custom", "custom_format":"'YYYY"})),
    ] {
        let params = serde_json::from_value(params).unwrap();
        assert!(matches!(resolve(id, Platform::Windows, &params, Lang::EnUs), Err(PresetError::InvalidParam { .. })), "{id}");
    }
    let params = serde_json::from_value(json!({"future":[1,2,3]})).unwrap();
    assert!(expand("edit.copy", Platform::Macos, &params, Lang::EnUs).is_ok());
    assert_eq!(expand("future.preset", Platform::Windows, &params, Lang::EnUs), Err(PresetError::UnknownId));
}

#[test]
fn representative_expansions_match_snapshots() {
    let cases: Vec<Value> = serde_json::from_str(include_str!("fixtures/preset-expansions.json")).unwrap();
    for case in cases {
        let id = case["id"].as_str().unwrap();
        let platform = if case["platform"] == "macos" { Platform::Macos } else { Platform::Windows };
        let params = serde_json::from_value(case["params"].clone()).unwrap();
        let lang = Lang::from_tag(case["lang"].as_str().unwrap()).unwrap();
        let expanded = match expand(id, platform, &params, lang) {
            Ok(action) => serde_json::to_value(action).unwrap(),
            Err(error) => json!({"error":error.reason_key()}),
        };
        assert_eq!(expanded, case["result"], "{id} {platform:?} {params:?}");
    }
}

#[test]
fn native_text_methods_dates_and_custom_patterns_are_resolved_without_lua() {
    let now = mousetroke_core::action_datetime::LocalDateTime { year: 2026, month: 10, day: 24, hour: 14, minute: 30, second: 7 };
    for platform in [Platform::Windows, Platform::Macos] {
        for method in ["paste", "type"] {
            let params = serde_json::from_value(json!({"text":"  中文 😀 ^+#{ENTER}\n ","method":method})).unwrap();
            let PreparedAction::Native(action) = resolve("text.type", platform, &params, Lang::EnUs).unwrap() else { panic!() };
            assert_eq!(action.formatted_text(now).unwrap(), "  中文 😀 ^+#{ENTER}\n ");
            assert!(matches!(action, NativeAction::TypeText { method: actual, .. } if actual == if method == "paste" { TextMethod::Paste } else { TextMethod::Type }));
            for (format, custom, expected) in [
                ("date_iso", "", "2026-10-24"), ("date_local", "", "2026年10月24日"),
                ("date_us", "", "10/24/2026"), ("time", "", "14:30"), ("datetime", "", "2026-10-24 14:30"),
                ("custom", "YYYY-MM-DD HH:mm:ss", "2026-10-24 14:30:07"),
                ("custom", "'YYYY' YYYY ''", "YYYY 2026 '"), ("custom", "date", "date"),
            ] {
                let params = serde_json::from_value(json!({"format":format,"custom_format":custom,"method":method})).unwrap();
                let PreparedAction::Native(action) = resolve("text.datetime", platform, &params, Lang::EnUs).unwrap() else { panic!() };
                assert_eq!(action.formatted_text(now).unwrap(), expected);
                assert!(matches!(action, NativeAction::DateTime { method: actual, .. } if actual == if method == "paste" { TextMethod::Paste } else { TextMethod::Type }));
            }
        }
        assert!(matches!(resolve("text.datetime", platform, &Params::new(), Lang::EnUs).unwrap(), PreparedAction::Native(NativeAction::DateTime { method: TextMethod::Paste, .. })));
    }
}

#[test]
fn windows_system_program_presets_use_explicit_system_root_paths() {
    for (id, expected) in [
        ("capture.tool", r"%SystemRoot%\System32\SnippingTool.exe"),
        ("system.calculator", r"%SystemRoot%\System32\calc.exe"),
        ("open.folder", r"%SystemRoot%\explorer.exe"),
    ] {
        let mut params = find(id).unwrap().default_params(Lang::EnUs);
        if id == "open.folder" { params.insert("path".into(), json!("folder")); }
        let Action::LaunchApp { exe, .. } = expand(id, Platform::Windows, &params, Lang::EnUs).unwrap() else { panic!("{id}: expected launch") };
        assert_eq!(exe, expected);
        assert!(exe.contains("%SystemRoot%"));
    }
}
