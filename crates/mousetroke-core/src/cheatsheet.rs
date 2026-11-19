//! 「显示手势列表」：把当前目标可用的手势列成弹出菜单，选中即执行该手势的动作。

use crate::i18n::{display_name, t, try_t};
use crate::schema::{Action, MenuItem, MouseButton, PatternKey, Scope, Step};

pub const PRESET_ID: &str = "mousetroke.cheatsheet";

/// Windows 菜单把 Tab 后的文字右对齐成一列；其他平台用空格分隔。
const SEPARATOR: &str = if cfg!(windows) { "\t" } else { "  " };

fn is_cheatsheet(action: &Action) -> bool {
    matches!(action, Action::Preset { id, .. } if id == PRESET_ID)
}

/// 按作用域链（见 `matcher::resolve_scopes`）列出可用手势：跳过停用和草稿，
/// 后面的作用域里与前面同模式的绑定已被覆盖，不再列出；本身含手势列表的绑定也不列出。
pub fn items(scopes: &[&Scope]) -> Vec<MenuItem> {
    let mut out = Vec::new();
    let mut covered: Vec<&[Step]> = Vec::new();
    for scope in scopes {
        let enabled: Vec<_> = scope.bindings.iter().filter(|b| !b.disabled && !b.draft).collect();
        for b in &enabled {
            let mut nested = false;
            b.action.visit(&mut |a| nested |= is_cheatsheet(a));
            if nested || covered.contains(&b.pattern.as_slice()) {
                continue;
            }
            let name = display_name(&b.name, &b.name_key);
            let label = match (name.is_empty(), pattern_text(&b.pattern)) {
                (false, Some(pattern)) => format!("{name}{SEPARATOR}{pattern}"),
                (true, Some(pattern)) => pattern,
                (false, None) => name.to_string(),
                (true, None) => t("ui.pattern.draw_shape").to_string(),
            };
            out.push(MenuItem { label, action: b.action.clone() });
        }
        covered.extend(enabled.iter().map(|b| b.pattern.as_slice()));
    }
    out
}

/// 把动作中的「显示手势列表」换成手势菜单。没有可列出的手势时返回 `false`（调用方提示，不执行）。
pub fn expand(action: &mut Action, scopes: &[&Scope]) -> bool {
    let mut found = false;
    action.visit(&mut |a| found |= is_cheatsheet(a));
    if !found {
        return true;
    }
    let items = items(scopes);
    if items.is_empty() {
        return false;
    }
    action.visit_mut(&mut |a| {
        if is_cheatsheet(a) {
            *a = Action::Menu { columns: 1, items: items.clone() };
        }
    });
    true
}

/// 紧凑的模式文字，如「右键 ↓→」，键名规则与设置界面一致。含形状时返回 `None`（只显示名称）。
fn pattern_text(pattern: &[Step]) -> Option<String> {
    let mut parts = Vec::new();
    for step in pattern {
        parts.push(match step {
            Step::Press { key: PatternKey::Mouse(b) } => t(match b {
                MouseButton::Left => "button.left",
                MouseButton::Right => "button.right",
                MouseButton::Middle => "button.middle",
                MouseButton::X1 => "button.x1",
                MouseButton::X2 => "button.x2",
            }).to_string(),
            Step::Edge { edge } => try_t(&format!("edge.{edge}"))?.to_string(),
            Step::Press { key: PatternKey::Keyboard(vk) } => match *vk {
                0x30..=0x39 | 0x41..=0x5a => char::from(*vk as u8).to_string(),
                0x70..=0x87 => format!("F{}", vk - 0x6f),
                _ => crate::i18n::tf("key.vk", &[("vk", &vk.to_string())]),
            },
            Step::Press { key: PatternKey::Wheel(n) } if *n < 0 => t("key.wheel_up").to_string(),
            Step::Press { key: PatternKey::Wheel(_) } | Step::Wheel { .. } => t("key.wheel_down").to_string(),
            Step::Trace { points } => arrows(points),
            Step::Shape { .. } => return None,
        });
    }
    Some(parts.join(" "))
}

/// 轨迹点（x/y 交替）→ 方向箭头串，与设置界面的 `arrows` 一致。
fn arrows(points: &[i32]) -> String {
    points.chunks_exact(2).collect::<Vec<_>>().windows(2).filter_map(|w| {
        match ((w[1][0] - w[0][0]).signum(), (w[1][1] - w[0][1]).signum()) {
            (0, -1) => Some('↑'),
            (1, -1) => Some('↗'),
            (1, 0) => Some('→'),
            (1, 1) => Some('↘'),
            (0, 1) => Some('↓'),
            (-1, 1) => Some('↙'),
            (-1, 0) => Some('←'),
            (-1, -1) => Some('↖'),
            _ => None,
        }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{Binding, ScopeKind, WindowOpKind};

    const RIGHT: Step = Step::Press { key: PatternKey::Mouse(MouseButton::Right) };

    fn trace(points: &[i32]) -> Step {
        Step::Trace { points: points.to_vec() }
    }

    fn binding(name: &str, pattern: Vec<Step>, action: Action) -> Binding {
        Binding { id: String::new(), name: name.into(), name_key: None, gesture: None, pattern, action, execute_on_match: false, hide_label: false, secured: false, disabled: false, draft: false }
    }

    fn close(name: &str, pattern: Vec<Step>) -> Binding {
        binding(name, pattern, Action::WindowOp { op: WindowOpKind::Close, allow_child: false })
    }

    fn scope(kind: ScopeKind, bindings: Vec<Binding>) -> Scope {
        Scope { id: "s".into(), name: "s".into(), name_key: None, kind, app: None, special: None, inherit_global: true, fullscreen_policy: Default::default(), disabled: false, bindings }
    }

    fn cheatsheet() -> Action {
        Action::Preset { id: PRESET_ID.into(), params: Default::default() }
    }

    fn labels(items: &[MenuItem]) -> Vec<&str> {
        items.iter().map(|i| i.label.split(SEPARATOR).next().unwrap()).collect()
    }

    #[test]
    fn lists_app_then_uncovered_global_bindings() {
        let down = vec![RIGHT, trace(&[50, 0, 50, 50])];
        let left = vec![RIGHT, trace(&[50, 0, 0, 0])];
        let mut disabled = close("Disabled", vec![RIGHT, trace(&[0, 0, 50, 0])]);
        disabled.disabled = true;
        let mut draft = close("Draft", vec![RIGHT, trace(&[0, 0, 0, 50, 50, 50])]);
        draft.draft = true;
        let app = scope(ScopeKind::App, vec![close("App down", down.clone()), disabled, draft]);
        let global = scope(ScopeKind::Global, vec![
            close("Global down", down),
            close("Global left", left),
            binding("List", vec![RIGHT, trace(&[0, 0, 0, 50])], cheatsheet()),
        ]);
        let items = items(&[&app, &global]);
        assert_eq!(labels(&items), ["App down", "Global left"]);
        assert!(items[0].label.ends_with(" ↓"));
        assert!(matches!(items[0].action, Action::WindowOp { .. }));
        // 不继承通用手势：作用域链只有该程序
        assert_eq!(labels(&super::items(&[&app])), ["App down"]);
    }

    #[test]
    fn labels_fall_back_to_pattern_or_name() {
        // 标签取当前语言；其他测试会临时切换全局语言
        let _g = crate::i18n::tests::LANG_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let shape = Step::Shape { templates: vec![vec![0, 0]] };
        let global = scope(ScopeKind::Global, vec![
            close("", vec![RIGHT, trace(&[0, 0, 0, 50, 50, 50])]),
            close("Circle", vec![RIGHT, shape.clone()]),
            close("", vec![RIGHT, shape]),
            close("Corner", vec![Step::Edge { edge: crate::edge::RIGHT_TOP }]),
        ]);
        let items = items(&[&global]);
        assert!(items[0].label.ends_with(" ↓→") && !items[0].label.contains(SEPARATOR));
        assert_eq!(items[1].label, "Circle");
        assert_eq!(items[2].label, t("ui.pattern.draw_shape"));
        assert!(items[3].label.starts_with(&format!("Corner{SEPARATOR}")));
    }

    #[test]
    fn expand_replaces_nested_cheatsheet_or_reports_empty() {
        let global = scope(ScopeKind::Global, vec![close("Close", vec![RIGHT, trace(&[0, 0, 0, 50])])]);
        let mut action = Action::Menu { columns: 1, items: vec![MenuItem { label: "x".into(), action: cheatsheet() }] };
        assert!(expand(&mut action, &[&global]));
        let Action::Menu { items, .. } = &action else { panic!() };
        assert!(matches!(&items[0].action, Action::Menu { items, .. } if labels(items) == ["Close"]));

        let mut plain = Action::Shell { script: "x".into() };
        assert!(expand(&mut plain, &[]));
        assert_eq!(plain, Action::Shell { script: "x".into() });
        assert!(!expand(&mut cheatsheet(), &[]));
    }
}
