//! 手势匹配：捕获会话 → 步骤序列 → 在作用域内找最优绑定。
//!
//! 纯逻辑（不依赖平台），由引擎在识别中/识别结束时调用。

use crate::recognizer::{best_score, Segment};
use crate::schema::{Action, Binding, Config, MouseButton, PatternKey, Scope, ScopeKind, Step};

/// 捕获会话的触发源。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    Button(MouseButton),
    /// 屏幕边/角（位标志编码，见 [`crate::edge`]）。
    Edge(u8),
}

/// 把捕获会话构建为模式步骤序列。
///
/// `trace_at`：轨迹开始时已记录的附加键数（轨迹占位插在这些键之后）；`None` = 无轨迹。
/// Trace 用空 points 占位，匹配时单独评分。
pub fn build_steps(trigger: Trigger, presses: &[PatternKey], trace_at: Option<usize>) -> Vec<Step> {
    let mut steps = Vec::with_capacity(presses.len() + 2);
    steps.push(match trigger {
        Trigger::Button(b) => Step::Press { key: PatternKey::Mouse(b) },
        Trigger::Edge(e) => Step::Edge { edge: e },
    });
    for (i, k) in presses.iter().enumerate() {
        if trace_at == Some(i) {
            steps.push(Step::Trace { points: vec![] });
        }
        steps.push(Step::Press { key: *k });
    }
    if trace_at.is_some_and(|t| t >= presses.len()) {
        steps.push(Step::Trace { points: vec![] });
    }
    steps
}

/// 模式与构建序列逐步比较（长度需相等）。返回模式中的轨迹点（若有）。
/// 模式含多于一个轨迹时视为不支持。
fn compare<'a>(pattern: &'a [Step], built: &[Step]) -> Option<Option<&'a [i32]>> {
    if pattern.len() != built.len() {
        return None;
    }
    let mut trace = None;
    for (p, b) in pattern.iter().zip(built) {
        match (p, b) {
            (Step::Trace { points }, Step::Trace { .. }) => {
                if trace.replace(points.as_slice()).is_some() {
                    return None;
                }
            }
            _ if step_equal(p, b) => {}
            _ => return None,
        }
    }
    Some(trace)
}

/// 在作用域内匹配手势。
///
/// - 无轨迹模式：完全相等即命中；同名重复时取最后一个（旧格式语义）。
/// - 带轨迹模式：其余步骤相等、轨迹位置一致且段数一致的候选中，取识别评分唯一最优者。
pub fn match_binding<'a>(scope: &'a Scope, built: &[Step], segs: &[Segment]) -> Option<&'a Binding> {
    let has_trace = built.iter().any(|s| matches!(s, Step::Trace { .. }));
    let mut cands: Vec<(&[i32], &[Segment])> = Vec::new();
    let mut feasible: Vec<&Binding> = Vec::new();
    let mut pure_hit: Option<&Binding> = None;

    for b in scope.bindings.iter().filter(|b| !b.disabled) {
        match compare(&b.pattern, built) {
            Some(None) if !has_trace => pure_hit = Some(b),
            Some(Some(points)) if !segs.is_empty() && points.len() / 2 == segs.len() + 1 => {
                cands.push((points, segs));
                feasible.push(b);
            }
            _ => {}
        }
    }
    if pure_hit.is_some() {
        return pure_hit;
    }
    best_score(&cands).map(|(idx, _)| feasible[idx])
}

/// 是否存在以 `built` 为前缀（且更长或等长）的绑定——用于判断某个输入是否值得继续捕获。
/// 前缀中的轨迹占位与模式中同位置的轨迹视为相容（不比较点）。
pub fn is_feasible_prefix(scope: &Scope, built: &[Step]) -> bool {
    scope.bindings.iter().filter(|b| !b.disabled).any(|b| {
        b.pattern.len() >= built.len()
            && b.pattern.iter().zip(built).all(|(p, q)| match (p, q) {
                (Step::Trace { .. }, Step::Trace { .. }) => true,
                _ => step_equal(p, q),
            })
    })
}

fn step_equal(pattern: &Step, built: &Step) -> bool {
    match (pattern, built) {
        (Step::Press { key: a }, Step::Press { key: b }) => a == b,
        // 旧格式 Scroll 元素（垂直）视为任意方向的滚轮 press
        (Step::Wheel { horizontal: false }, Step::Press { key: PatternKey::Wheel(_) }) => true,
        (Step::Edge { edge: a }, Step::Edge { edge: b }) => a == b,
        _ => false,
    }
}

/// 根据目标窗口选择作用域链（按优先级）：桌面 → special；exe 命中 → app；最后 global。
/// 特定作用域 `inherit_global = false` 时不回落到 global。
pub fn resolve_scopes<'a>(cfg: &'a Config, is_desktop: bool, exe: Option<&str>) -> Vec<&'a Scope> {
    let specific = is_desktop
        .then(|| cfg.scopes.iter().find(|s| s.kind == ScopeKind::Special))
        .flatten()
        .or_else(|| {
            let exe = exe?;
            cfg.scopes
                .iter()
                .find(|s| s.kind == ScopeKind::App && s.app.as_ref().is_some_and(|m| exe_matches(&m.exe, exe)))
        });
    let global = cfg.scopes.iter().find(|s| s.kind == ScopeKind::Global);
    match specific {
        Some(s) if s.disabled => vec![], // 该程序禁用了手势
        Some(s) if !s.inherit_global => vec![s],
        Some(s) => std::iter::once(s).chain(global).collect(),
        None => global.into_iter().collect(),
    }
}

/// 目标程序（或桌面）是否被设为"禁用所有手势"。
pub fn gestures_off(cfg: &Config, is_desktop: bool, exe: Option<&str>) -> bool {
    resolve_scopes(cfg, is_desktop, exe).is_empty() && cfg.scopes.iter().any(|s| s.disabled)
}

/// 在作用域链上依次匹配，返回 (命中绑定, 所在作用域)。
pub fn match_chain<'a>(scopes: &[&'a Scope], built: &[Step], segs: &[Segment]) -> Option<(&'a Binding, &'a Scope)> {
    scopes.iter().find_map(|s| match_binding(s, built, segs).map(|b| (b, *s)))
}

/// exe 匹配：规则含路径分隔符时比较完整路径，否则只比较文件名（均大小写不敏感）。
pub fn exe_matches(rule: &str, exe: &str) -> bool {
    if rule.contains(['\\', '/']) {
        rule.eq_ignore_ascii_case(exe)
    } else {
        file_name(exe).eq_ignore_ascii_case(rule)
    }
}

/// 路径的文件名部分。
pub fn file_name(path: &str) -> &str {
    path.rsplit(['\\', '/']).next().unwrap_or(path)
}

/// 从配置推导按键触发：任意启用绑定首个 press 步骤中的鼠标键。
/// 左键不允许单独画线（会破坏拖选），故排除。
pub fn trigger_buttons(cfg: &Config) -> Vec<MouseButton> {
    let mut buttons = Vec::new();
    for b in cfg.all_bindings().filter(|b| !b.disabled) {
        if let Some(Step::Press { key: PatternKey::Mouse(btn) }) = b.pattern.first() {
            if *btn != MouseButton::Left && !buttons.contains(btn) {
                buttons.push(*btn);
            }
        }
    }
    buttons
}

/// 从配置推导启用的边缘/角落值。
pub fn edge_triggers(cfg: &Config) -> Vec<u8> {
    let mut edges = Vec::new();
    for b in cfg.all_bindings().filter(|b| !b.disabled) {
        if let Some(Step::Edge { edge }) = b.pattern.first() {
            if !edges.contains(edge) {
                edges.push(*edge);
            }
        }
    }
    edges
}

/// 动作类别名（日志/UI 用）。
pub fn action_kind(a: &Action) -> &'static str {
    match a {
        Action::KeySequence { .. } => "key_sequence",
        Action::Shell { .. } => "shell",
        Action::Lua { .. } => "lua",
        Action::OpenUrl { .. } => "open_url",
        Action::Menu { .. } => "menu",
        Action::LaunchApp { .. } => "launch_app",
        Action::WindowOp { .. } => "window_op",
        Action::SystemKey { .. } => "system_key",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edge;
    use crate::recognizer::PathTracker;
    use crate::schema::{AppMatch, WindowOpKind};

    fn binding(name: &str, pattern: Vec<Step>) -> Binding {
        Binding {
            name: name.into(),
            name_key: None,
            pattern,
            action: Action::WindowOp { op: WindowOpKind::Close, allow_child: false },
            execute_on_match: false,
            hide_label: false,
            secured: false,
            disabled: false,
        }
    }

    fn scope(kind: ScopeKind, bindings: Vec<Binding>) -> Scope {
        Scope { id: "s".into(), name: "s".into(), name_key: None, kind, app: None, special: None, inherit_global: true, disabled: false, bindings }
    }

    const RIGHT: Step = Step::Press { key: PatternKey::Mouse(MouseButton::Right) };

    fn down_trace() -> Vec<Segment> {
        let mut t = PathTracker::new(30.0);
        t.start(0.0, 0.0);
        for i in 1..=10 {
            t.add_point(0.0, i as f64 * 20.0);
        }
        t.segments().to_vec()
    }

    const LEFT_KEY: PatternKey = PatternKey::Mouse(MouseButton::Left);
    const RIGHT_KEY: PatternKey = PatternKey::Mouse(MouseButton::Right);

    #[test]
    fn build_places_trace_between_presses() {
        let b = build_steps(Trigger::Edge(edge::TOP), &[RIGHT_KEY, LEFT_KEY], Some(1));
        assert_eq!(
            b,
            vec![
                Step::Edge { edge: edge::TOP },
                Step::Press { key: RIGHT_KEY },
                Step::Trace { points: vec![] },
                Step::Press { key: LEFT_KEY },
            ]
        );
        assert_eq!(build_steps(Trigger::Edge(edge::TOP), &[RIGHT_KEY], Some(1)).len(), 3);
    }

    #[test]
    fn trace_in_middle_matches() {
        // 旧格式「系统设置」：上边 + 按右键 + 下划 + 点左键
        let s = scope(
            ScopeKind::Global,
            vec![
                binding(
                    "settings",
                    vec![
                        Step::Edge { edge: edge::TOP },
                        Step::Press { key: RIGHT_KEY },
                        Step::Trace { points: vec![0, 0, 0, 50] },
                        Step::Press { key: LEFT_KEY },
                    ],
                ),
                binding("play", vec![Step::Edge { edge: edge::TOP }, Step::Press { key: RIGHT_KEY }]),
            ],
        );
        let segs = down_trace();
        let b = build_steps(Trigger::Edge(edge::TOP), &[RIGHT_KEY, LEFT_KEY], Some(1));
        assert_eq!(match_binding(&s, &b, &segs).unwrap().name, "settings");
        let b = build_steps(Trigger::Edge(edge::TOP), &[RIGHT_KEY], None);
        assert_eq!(match_binding(&s, &b, &[]).unwrap().name, "play");
    }

    #[test]
    fn feasibility() {
        let s = scope(
            ScopeKind::Global,
            vec![
                binding("vol", vec![Step::Edge { edge: edge::TOP }, Step::Press { key: PatternKey::Wheel(-1) }]),
                binding("tm", vec![Step::Edge { edge: edge::RIGHT }, Step::Trace { points: vec![50, 50, 50, 0] }]),
            ],
        );
        let top = |p: &[PatternKey], t| is_feasible_prefix(&s, &build_steps(Trigger::Edge(edge::TOP), p, t));
        assert!(top(&[PatternKey::Wheel(-1)], None));
        assert!(!top(&[LEFT_KEY], None), "上边缘点左键不应被捕获");
        assert!(!top(&[], Some(0)), "上边缘没有纯轨迹绑定");
        assert!(is_feasible_prefix(&s, &build_steps(Trigger::Edge(edge::RIGHT), &[], Some(0))));
    }

    #[test]
    fn trace_match_picks_direction() {
        let s = scope(
            ScopeKind::Global,
            vec![
                binding("right", vec![RIGHT, Step::Trace { points: vec![0, 0, 50, 0] }]),
                binding("down", vec![RIGHT, Step::Trace { points: vec![0, 0, 0, 50] }]),
            ],
        );
        let built = build_steps(Trigger::Button(MouseButton::Right), &[], Some(0));
        assert_eq!(match_binding(&s, &built, &down_trace()).unwrap().name, "down");
    }

    #[test]
    fn pure_edge_and_wheel() {
        let s = scope(
            ScopeKind::Global,
            vec![
                binding("start", vec![Step::Edge { edge: edge::LEFT_BOTTOM }]),
                binding("vol", vec![Step::Edge { edge: edge::LEFT }, Step::Wheel { horizontal: false }]),
            ],
        );
        let b = build_steps(Trigger::Edge(edge::LEFT_BOTTOM), &[], None);
        assert_eq!(match_binding(&s, &b, &[]).unwrap().name, "start");
        let b = build_steps(Trigger::Edge(edge::LEFT), &[PatternKey::Wheel(-1)], None);
        assert_eq!(match_binding(&s, &b, &[]).unwrap().name, "vol");
        let b = build_steps(Trigger::Edge(edge::TOP), &[], None);
        assert!(match_binding(&s, &b, &[]).is_none());
    }

    #[test]
    fn duplicate_pure_takes_last() {
        let s = scope(
            ScopeKind::Global,
            vec![
                binding("a", vec![RIGHT, Step::Press { key: PatternKey::Mouse(MouseButton::Left) }]),
                binding("b", vec![RIGHT, Step::Press { key: PatternKey::Mouse(MouseButton::Left) }]),
            ],
        );
        let b = build_steps(Trigger::Button(MouseButton::Right), &[LEFT_KEY], None);
        assert_eq!(match_binding(&s, &b, &[]).unwrap().name, "b");
    }

    #[test]
    fn scope_resolution() {
        let mut app = scope(ScopeKind::App, vec![]);
        app.name = "chrome".into();
        app.app = Some(AppMatch::new(r"C:\Apps\Chrome\chrome.exe"));
        let mut bare = scope(ScopeKind::App, vec![]);
        bare.name = "code".into();
        bare.app = Some(AppMatch::new("Code.exe"));
        let mut desk = scope(ScopeKind::Special, vec![]);
        desk.name = "desk".into();
        let mut glob = scope(ScopeKind::Global, vec![]);
        glob.name = "global".into();
        let cfg = Config { scopes: vec![app, bare, desk, glob], ..Default::default() };

        let chain = |d, e| resolve_scopes(&cfg, d, e).iter().map(|s| s.name.clone()).collect::<Vec<_>>();
        assert_eq!(chain(false, Some(r"c:\apps\chrome\CHROME.EXE")), ["chrome", "global"]);
        assert_eq!(chain(false, Some(r"D:\x\code.exe")), ["code", "global"]);
        assert_eq!(chain(true, None), ["desk", "global"]);
        assert_eq!(chain(false, Some(r"C:\other.exe")), ["global"]);
        assert_eq!(chain(false, None), ["global"]);
        let mut cfg2 = cfg.clone();
        cfg2.scopes[1].inherit_global = false;
        assert_eq!(resolve_scopes(&cfg2, false, Some("code.exe")).len(), 1);
    }

    #[test]
    fn triggers_exclude_left() {
        let cfg = Config {
            scopes: vec![scope(
                ScopeKind::Global,
                vec![
                    binding("l", vec![Step::Press { key: PatternKey::Mouse(MouseButton::Left) }]),
                    binding("r", vec![RIGHT]),
                    binding("r2", vec![RIGHT]),
                    binding("e", vec![Step::Edge { edge: edge::RIGHT_TOP }]),
                ],
            )],
            ..Default::default()
        };
        assert_eq!(trigger_buttons(&cfg), vec![MouseButton::Right]);
        assert_eq!(edge_triggers(&cfg), vec![edge::RIGHT_TOP]);
    }
}
