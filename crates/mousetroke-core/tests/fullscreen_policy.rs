use mousetroke_core::matcher::pause_in_fullscreen;
use mousetroke_core::schema::{Config, FullscreenPolicy, ScopeKind};

fn config() -> Config {
    Config::from_json(r#"{"scopes":[{"id":"chrome","name":"Chrome","kind":"app","app":{"exe":"chrome.exe"},"bindings":[]}]}"#).unwrap()
}

#[test]
fn defaults() {
    assert!(Config::from_json("{}").unwrap().settings.pause_on_fullscreen);
    let cfg = config();
    assert!(cfg.settings.pause_on_fullscreen);
    assert_eq!(cfg.scopes[0].fullscreen_policy, FullscreenPolicy::Inherit);
    assert!(!cfg.to_json().contains("fullscreen_policy"));
}

#[test]
fn roundtrip() {
    for policy in [FullscreenPolicy::Inherit, FullscreenPolicy::Allow, FullscreenPolicy::Pause] {
        let mut cfg = config();
        cfg.settings.pause_on_fullscreen = false;
        cfg.scopes[0].fullscreen_policy = policy;
        let saved = cfg.to_json();
        assert_eq!(Config::from_json(&saved).unwrap(), cfg);
        if policy != FullscreenPolicy::Inherit {
            let value: serde_json::Value = serde_json::from_str(&saved).unwrap();
            assert_eq!(value["scopes"][0]["fullscreen_policy"], if policy == FullscreenPolicy::Allow { "allow" } else { "pause" });
        }
    }
}

#[test]
fn policy_matrix() {
    for global in [true, false] {
        for (policy, expected) in [(FullscreenPolicy::Inherit, global), (FullscreenPolicy::Allow, false), (FullscreenPolicy::Pause, true)] {
            let mut cfg = config();
            cfg.settings.pause_on_fullscreen = global;
            cfg.scopes[0].fullscreen_policy = policy;
            assert_eq!(pause_in_fullscreen(&cfg, Some(r"C:\Apps\CHROME.EXE")), expected);
            assert_eq!(pause_in_fullscreen(&cfg, Some("other.exe")), global);
            assert_eq!(pause_in_fullscreen(&cfg, None), global);
            let mut duplicate = cfg.scopes[0].clone();
            duplicate.fullscreen_policy = if expected { FullscreenPolicy::Allow } else { FullscreenPolicy::Pause };
            cfg.scopes.push(duplicate);
            assert_eq!(pause_in_fullscreen(&cfg, Some("chrome.exe")), expected);
            cfg.scopes.truncate(1);
            for kind in [ScopeKind::Global, ScopeKind::Special] {
                cfg.scopes[0].kind = kind;
                assert_eq!(pause_in_fullscreen(&cfg, Some("chrome.exe")), global);
            }
        }
    }
}
