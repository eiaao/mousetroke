//! 各平台对动作的支持差异：设置界面的提示与修复、按键校验、运行时共用这一份规则。
//! 设置界面的 `ui/src/platform.ts` 镜像同一规则，两端都用 `tests/fixtures/mac-caps.json` 校验。
use crate::keyseq::{self, Key, KeySeq};
use crate::schema::{Action, IfRunning, SendMode, WindowOpKind};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Windows,
    Macos,
}

impl Platform {
    pub const CURRENT: Platform = if cfg!(target_os = "macos") {
        Platform::Macos
    } else {
        Platform::Windows
    };
}

/// 按键序列能否在该平台完整发送：macOS 没有鼠标/滚轮键，也发不出无对应键码的键。
pub fn keys_supported(seq: &KeySeq, platform: Platform) -> bool {
    platform == Platform::Windows
        || seq.actions.iter().all(|action| match action {
            keyseq::Action::Key { key: Key::Vk(vk), .. } => crate::keymap::vk_to_kvk(*vk).is_some(),
            keyseq::Action::Key { key: Key::Mouse(_), .. } => false,
            _ => true,
        })
}

fn named(value: &Option<String>) -> bool {
    value.as_deref().is_some_and(|s| !s.is_empty())
}

/// 动作的全部选项在该平台都有效。无法解析的按键文本交给按键校验报告，这里不算不支持。
pub fn action_supported(action: &Action, platform: Platform) -> bool {
    if let Action::Preset { id, .. } = action {
        return crate::presets::actions::find(id).is_some_and(|p| matches!(p.implementation(platform), crate::presets::actions::Implementation::Implemented(_)));
    }
    if matches!(action, Action::Unknown(_)) { return false; }
    if platform == Platform::Windows {
        return true;
    }
    match action {
        Action::WindowOp { op, allow_child } => *op != WindowOpKind::ToggleTopmost && !allow_child,
        Action::LaunchApp { elevate, if_running, win_class, win_title, regex, .. } => {
            !elevate && *if_running != IfRunning::SendKeys && !named(win_class) && !named(win_title) && !regex
        }
        Action::KeySequence { keys, mode, .. } => {
            *mode == SendMode::DownUp
                && keyseq::parse(keys, false).map_or(true, |seq| keys_supported(&seq, platform))
        }
        Action::Menu { columns, .. } => *columns <= 1,
        _ => true,
    }
}

/// 去掉平台不支持的选项，换成最接近的可用行为；由用户在设置里明确点击后才应用。
pub fn repair(action: Action, platform: Platform) -> Action {
    if platform == Platform::Windows {
        return action;
    }
    match action {
        Action::WindowOp { op, .. } => Action::WindowOp {
            op: if op == WindowOpKind::ToggleTopmost { WindowOpKind::Minimize } else { op },
            allow_child: false,
        },
        Action::LaunchApp { exe, args, if_running, activate_by_identity, .. } => Action::LaunchApp {
            exe,
            args,
            activate_by_identity,
            elevate: false,
            win_class: None,
            win_title: None,
            regex: false,
            if_running: if if_running == IfRunning::SendKeys { IfRunning::Activate } else { if_running },
        },
        Action::KeySequence { keys, secured, activate_target, .. } => Action::KeySequence {
            keys: strip_mouse_keys(&keys),
            mode: SendMode::DownUp,
            secured,
            activate_target,
        },
        Action::Menu { items, .. } => Action::Menu { columns: 1, items },
        other => other,
    }
}

/// 删除 `{MOUSE_…}` 与 `{WHEEL…}`/`{HWHEEL…}`/`{VWHEEL…}` 记号（不区分大小写）。
fn strip_mouse_keys(keys: &str) -> String {
    let mut out = String::with_capacity(keys.len());
    let mut rest = keys;
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        let token = &rest[start..];
        let Some(end) = token.find('}') else {
            out.push_str(token);
            return out;
        };
        let name = token[1..end].to_ascii_uppercase();
        let mouse = ["MOUSE_", "WHEEL", "HWHEEL", "VWHEEL"].iter().any(|p| name.starts_with(p));
        if !mouse {
            out.push_str(&token[..=end]);
        }
        rest = &token[end + 1..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(serde::Deserialize)]
    struct Case {
        action: Action,
        repaired: Option<Action>,
    }

    #[test]
    fn mac_rules_match_the_shared_fixture() {
        let cases: Vec<Case> =
            serde_json::from_str(include_str!("../tests/fixtures/mac-caps.json")).unwrap();
        for case in cases {
            let expected = case.repaired.clone().unwrap_or_else(|| case.action.clone());
            assert_eq!(action_supported(&case.action, Platform::Macos), case.repaired.is_none(), "{:?}", case.action);
            assert_eq!(repair(case.action.clone(), Platform::Macos), expected);
            assert!(action_supported(&expected, Platform::Macos), "{expected:?}");
            assert!(action_supported(&case.action, Platform::Windows));
            assert_eq!(repair(case.action.clone(), Platform::Windows), case.action);
        }
    }

    #[test]
    fn enabled_mac_defaults_only_use_supported_actions() {
        let cfg = crate::schema::Config::builtin_for(Platform::Macos);
        for binding in cfg.all_bindings().filter(|b| !b.disabled) {
            assert!(action_supported(&binding.action, Platform::Macos), "{}", binding.name);
        }
    }

    #[test]
    fn mac_rejects_mouse_keys_and_strips_them_on_repair() {
        let mouse = keyseq::parse("{MOUSE_LEFT}", false).unwrap();
        assert!(keys_supported(&mouse, Platform::Windows));
        assert!(!keys_supported(&mouse, Platform::Macos));
        assert!(keys_supported(&keyseq::parse("#+t", false).unwrap(), Platform::Macos));
        assert_eq!(strip_mouse_keys("a{mouse_left}b{Enter}{vwheel 2}{"), "ab{Enter}{");
    }
}
