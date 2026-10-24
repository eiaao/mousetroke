use mousetroke_core::caps::Platform;
use mousetroke_core::schema::{Action, Config};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!("mousetroke-preset-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn config(&self) -> PathBuf { self.0.join("config.json") }
}
impl Drop for Temp {
    fn drop(&mut self) {
        for file in std::fs::read_dir(&self.0).unwrap() { std::fs::remove_file(file.unwrap().path()).unwrap(); }
        std::fs::remove_dir(&self.0).unwrap();
    }
}

fn config_with(action: Value) -> Config {
    Config::from_json(&json!({"version":1,"scopes":[{"id":"g","name":"G","kind":"global","bindings":[{"name":"B","pattern":[],"action":action}]}]}).to_string()).unwrap()
}

#[test]
fn presets_and_future_actions_round_trip_including_nested_menus() {
    let entries = json!([
        {"label":"known","action":{"type":"preset","id":"text.search","params":{"engine":"google","future_option":[1,true,{"x":"y"}]}}},
        {"label":"future","action":{"type":"future_action","payload":{"type":"anything","data":[null,123,"^%+#"]},"enabled":false}},
        {"label":"future preset","action":{"type":"preset","id":"future.preset","params":{"complex":{"a":[1,2]}},"future_field":{"revision":3}}},
        {"label":"future empty params","action":{"type":"preset","id":"future.empty","params":{}}},
        {"label":"empty params","action":{"type":"preset","id":"edit.copy"}}
    ]);
    let cfg = config_with(json!({"type":"menu","columns":1,"items":entries}));
    let dir = Temp::new();
    cfg.save(&dir.config()).unwrap();
    let loaded = Config::load(&dir.config()).unwrap();
    let raw: Value = serde_json::from_str(&loaded.to_json()).unwrap();
    assert_eq!(raw["scopes"][0]["bindings"][0]["action"]["items"], entries);
    assert_eq!(raw["version"], 2);
    let Action::Menu { items, .. } = &loaded.scopes[0].bindings[0].action else { panic!() };
    assert!(matches!(&items[1].action, Action::Unknown(_)));
    assert!(matches!(&items[2].action, Action::Unknown(raw) if raw["id"] == "future.preset"));
    assert_eq!(Config::from_json(&loaded.to_json()).unwrap(), loaded);
}

#[test]
fn an_unknown_preset_alone_also_requires_version_two() {
    let cfg = config_with(json!({"type":"preset","id":"future.preset","params":{},"future_field":[1,2]}));
    assert_eq!(cfg.saved_version(), 2);
    assert_eq!(serde_json::to_value(&cfg).unwrap()["scopes"][0]["bindings"][0]["action"], json!({"type":"preset","id":"future.preset","params":{},"future_field":[1,2]}));
}

#[test]
fn malformed_known_actions_and_invalid_tags_still_fail() {
    for action in [json!({"type":"key_sequence","keys":42}), json!({"type":"preset","id":"edit.copy","params":[]}), json!({"type":"preset"}), json!({"type":9}), json!({})] {
        assert!(serde_json::from_value::<Action>(action.clone()).is_err(), "{action}");
    }
}

#[test]
fn known_action_json_shapes_remain_compatible() {
    let canonical = json!([
        {"type":"key_sequence","keys":"^c","mode":"down_up","secured":false,"activate_target":false},
        {"type":"shell","script":"echo hello"}, {"type":"lua","code":"function ms_action() end"},
        {"type":"open_url","url_template":"https://example.com/","browser":null},
        {"type":"menu","columns":1,"items":[{"label":"Copy","action":{"type":"key_sequence","keys":"#c","mode":"down_up","secured":false,"activate_target":true}}]},
        {"type":"launch_app","exe":"editor.exe","args":null,"elevate":false,"win_class":null,"win_title":null,"regex":false,"if_running":"activate"},
        {"type":"window_op","op":"close","allow_child":false}, {"type":"system_key","key":"PLAY_PAUSE"}
    ]);
    for raw in canonical.as_array().unwrap() {
        let action: Action = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(serde_json::to_value(action).unwrap(), *raw);
    }
}

#[test]
fn broken_configs_are_never_overwritten_or_renamed() {
    for bytes in [b"invalid JSON".as_slice(), br#"{"scopes":[{"kind":"broken"}]}"#, &[0xff, 0xfe]] {
        let dir = Temp::new();
        let path = dir.config();
        std::fs::write(&path, bytes).unwrap();
        assert!(Config::load(&path).is_err());
        let (fallback, warning) = Config::load_or_init(&path).unwrap();
        assert!(warning.is_some());
        assert!(fallback.all_bindings().count() >= 40);
        assert!(fallback.save(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert_eq!(std::fs::read_dir(&dir.0).unwrap().count(), 1);
    }
}

#[test]
fn v1_is_preserved_until_a_preset_save_and_backup_is_created_once() {
    let dir = Temp::new();
    let path = dir.config();
    let backup = dir.0.join("config.v1.bak.json");
    let original = "\u{feff}{ \"version\": 1, \"scopes\": [] }\n";
    std::fs::write(&path, original).unwrap();
    let (cfg, warning) = Config::load_or_init(&path).unwrap();
    assert!(warning.is_none());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
    cfg.save(&path).unwrap();
    assert_eq!(Config::load(&path).unwrap().version, 1);
    assert!(!backup.exists());
    std::fs::write(&path, original).unwrap();
    let mut upgraded = config_with(json!({"type":"preset","id":"edit.copy"}));
    upgraded.save(&path).unwrap();
    assert_eq!(std::fs::read_to_string(&backup).unwrap(), original);
    assert_eq!(Config::load(&path).unwrap().version, 2);
    upgraded.settings.language = "en-US".into();
    upgraded.save(&path).unwrap();
    assert_eq!(std::fs::read_to_string(&backup).unwrap(), original);
    // 已有备份在第二次 v1→v2 保存时同样保留。
    std::fs::write(&path, "{\"version\":1}").unwrap();
    upgraded.save(&path).unwrap();
    assert_eq!(std::fs::read_to_string(&backup).unwrap(), original);
}

#[test]
fn explicit_v2_upgrade_and_failed_backup_are_safe() {
    let dir = Temp::new();
    let path = dir.config();
    std::fs::write(&path, "{}").unwrap();
    let cfg = Config { version: 2, ..Config::default() };
    let backup = dir.0.join("config.v1.bak.json");
    std::fs::create_dir(&backup).unwrap();
    assert!(cfg.save(&path).is_err());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "{}");
    std::fs::remove_dir(&backup).unwrap();
    cfg.save(&path).unwrap();
    assert_eq!(std::fs::read_to_string(dir.0.join("config.v1.bak.json")).unwrap(), "{}");
    let newer = Config { version: 7, ..Config::default() };
    assert_eq!(serde_json::to_value(newer).unwrap()["version"], 7);
}

#[test]
fn unified_defaults_initialize_both_platforms() {
    for platform in [Platform::Windows, Platform::Macos] {
        let dir = Temp::new();
        let cfg = Config::builtin_for(platform);
        cfg.save(&dir.config()).unwrap();
        assert_eq!(Config::load(&dir.config()).unwrap(), cfg);
        assert!(!dir.0.join("config.v1.bak.json").exists());
    }
    let dir = Temp::new();
    let (cfg, warning) = Config::load_or_init(&dir.config()).unwrap();
    assert!(warning.is_none());
    assert_eq!(cfg, Config::load(&dir.config()).unwrap());
}
