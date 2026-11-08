//! 手势库：引用维护、内置建库、兼容性与整理导入分析。

use mousetroke_core::caps::Platform;
use mousetroke_core::library::{self, AdoptSource, GroupKind, LibraryError, Relation};
use mousetroke_core::schema::{Action, AppMatch, Binding, Config, Gesture, MouseButton, PatternKey, Scope, ScopeKind, Step};
use mousetroke_core::shape;
use std::collections::BTreeMap;

#[derive(serde::Deserialize)]
struct RealShapes {
    classes: BTreeMap<String, Vec<Vec<i32>>>,
}

fn real() -> BTreeMap<String, Vec<Vec<i32>>> {
    serde_json::from_str::<RealShapes>(include_str!("fixtures/real-shapes.json")).unwrap().classes
}

fn right() -> Step {
    Step::Press { key: PatternKey::Mouse(MouseButton::Right) }
}

fn trace(points: &[i32]) -> Vec<Step> {
    vec![right(), Step::Trace { points: points.to_vec() }]
}

fn shape_pattern(templates: &[Vec<i32>]) -> Vec<Step> {
    vec![right(), Step::Shape { templates: templates.to_vec() }]
}

fn binding(name: &str, pattern: Vec<Step>) -> Binding {
    Binding {
        name: name.into(),
        name_key: None,
        gesture: None,
        pattern,
        action: Action::Shell { script: name.into() },
        execute_on_match: false,
        hide_label: false,
        secured: false,
        disabled: false,
        draft: false,
    }
}

fn scope(id: &str, kind: ScopeKind, bindings: Vec<Binding>) -> Scope {
    Scope {
        id: id.into(),
        name: id.into(),
        name_key: None,
        kind,
        app: (kind == ScopeKind::App).then(|| AppMatch::new(format!("{id}.exe"))),
        special: None,
        inherit_global: true,
        disabled: false,
        bindings,
    }
}

fn config(scopes: Vec<Scope>) -> Config {
    Config { scopes, ..Config::default() }
}

const LEFT: [i32; 4] = [50, 0, 0, 0];
const UP: [i32; 4] = [0, 50, 0, 0];
const DOWN_RIGHT: [i32; 6] = [0, 0, 0, 50, 50, 50];
const DOWN_RIGHT_LONG: [i32; 6] = [0, 0, 0, 100, 50, 100];

// ================================================================ 兼容与引用维护

#[test]
fn config_without_library_serializes_without_gestures_field() {
    let cfg = config(vec![scope("general", ScopeKind::Global, vec![binding("back", trace(&LEFT))])]);
    let json = cfg.to_json();
    assert!(!json.contains("\"gestures\""), "{json}");
    assert!(!json.contains("\"gesture\""), "{json}");
    assert_eq!(Config::from_json(&json).unwrap(), cfg);
}

#[test]
fn library_round_trips_and_field_order_is_stable() {
    let mut cfg = config(vec![scope("general", ScopeKind::Global, vec![binding("back", trace(&LEFT))])]);
    library::adopt_exact(&mut cfg, AdoptSource::Global);
    let json = cfg.to_json();
    let (g, s) = (json.find("\"gestures\"").unwrap(), json.find("\"scopes\"").unwrap());
    assert!(g < s, "手势库写在分组之前");
    assert_eq!(Config::from_json(&json).unwrap(), cfg);
    assert_eq!(Config::from_json(&json).unwrap().to_json(), json);
}

#[test]
fn old_versions_ignore_library_and_keep_working_patterns() {
    let cfg = Config::builtin_for(Platform::Windows);
    let mut raw: serde_json::Value = serde_json::from_str(&cfg.to_json()).unwrap();
    raw.as_object_mut().unwrap().remove("gestures");
    for scope in raw["scopes"].as_array_mut().unwrap() {
        for b in scope["bindings"].as_array_mut().unwrap() {
            b.as_object_mut().unwrap().remove("gesture");
        }
    }
    let old = Config::from_json(&raw.to_string()).unwrap();
    assert!(old.gestures.is_empty());
    let patterns = |c: &Config| c.all_bindings().map(|b| b.pattern.clone()).collect::<Vec<_>>();
    assert_eq!(patterns(&old), patterns(&cfg));
}

#[test]
fn broken_references_become_inline_without_changing_patterns() {
    let mut cfg = config(vec![scope("general", ScopeKind::Global, vec![binding("a", trace(&LEFT)), binding("b", trace(&UP)), binding("c", trace(&LEFT))])]);
    cfg.gestures.push(Gesture { id: "g1".into(), name: String::new(), pattern: trace(&LEFT) });
    let bindings = &mut cfg.scopes[0].bindings;
    bindings[0].gesture = Some("g1".into()); // 一致
    bindings[1].gesture = Some("g1".into()); // pattern 被手动改过
    bindings[2].gesture = Some("missing".into()); // 引用缺失
    let json = serde_json::to_string(&cfg).unwrap();
    let loaded = Config::from_json(&json).unwrap();
    assert_eq!(loaded.detached_gesture_refs, 2);
    let b = &loaded.scopes[0].bindings;
    assert_eq!(b[0].gesture.as_deref(), Some("g1"));
    assert_eq!((b[1].gesture.as_deref(), &b[1].pattern), (None, &trace(&UP)));
    assert_eq!((b[2].gesture.as_deref(), &b[2].pattern), (None, &trace(&LEFT)));
    // 幂等：再次加载不再解除
    assert_eq!(Config::from_json(&loaded.to_json()).unwrap().detached_gesture_refs, 0);
}

#[test]
fn invalid_library_entries_are_dropped_on_load() {
    let mut cfg = config(vec![scope("general", ScopeKind::Global, vec![binding("a", trace(&LEFT))])]);
    cfg.gestures = vec![
        Gesture { id: "g1".into(), name: "first".into(), pattern: trace(&LEFT) },
        Gesture { id: "g1".into(), name: "dup".into(), pattern: trace(&UP) },
        Gesture { id: "g2".into(), name: String::new(), pattern: vec![right(), Step::Trace { points: vec![] }] },
        Gesture { id: "g3".into(), name: String::new(), pattern: vec![] },
    ];
    let loaded = Config::from_json(&serde_json::to_string(&cfg).unwrap()).unwrap();
    assert_eq!(loaded.gestures.len(), 1);
    assert_eq!(loaded.gestures[0].name, "first");
}

#[test]
fn validation_rejects_broken_library() {
    let mut cfg = config(vec![scope("general", ScopeKind::Global, vec![binding("a", trace(&LEFT))])]);
    library::adopt_exact(&mut cfg, AdoptSource::Global);
    assert_eq!(library::validate(&cfg), Ok(()));

    let mut broken = cfg.clone();
    broken.scopes[0].bindings[0].pattern = trace(&UP);
    assert_eq!(library::validate(&broken), Err(LibraryError::BrokenRef("g1".into())));

    let mut dup = cfg.clone();
    dup.gestures.push(Gesture { id: "g1".into(), name: String::new(), pattern: trace(&UP) });
    assert_eq!(library::validate(&dup), Err(LibraryError::DuplicateId("g1".into())));

    let mut incomplete = cfg.clone();
    incomplete.gestures.push(Gesture { id: "g2".into(), name: String::new(), pattern: shape_pattern(&[]) });
    assert_eq!(library::validate(&incomplete), Err(LibraryError::Incomplete("g2".into())));
}

// ================================================================ 内置配置

#[test]
fn builtin_library_is_built_from_global_scope_without_unused_entries() {
    for platform in [Platform::Windows, Platform::Macos] {
        let cfg = Config::builtin_for(platform);
        let global = cfg.scopes.iter().find(|s| s.kind == ScopeKind::Global).unwrap();
        let mut unique: Vec<&Vec<Step>> = Vec::new();
        for b in &global.bindings {
            if !unique.contains(&&b.pattern) {
                unique.push(&b.pattern);
            }
        }
        assert_eq!(cfg.gestures.len(), unique.len(), "{platform:?}");
        for (i, g) in cfg.gestures.iter().enumerate() {
            assert_eq!(g.id, format!("g{}", i + 1));
            assert!(library::uses(&cfg, &g.id).next().is_some(), "{platform:?} 未使用的内置手势 {}", g.id);
        }
        assert!(global.bindings.iter().all(|b| b.gesture.is_some()));
        assert_eq!(library::validate(&cfg), Ok(()));
        assert_eq!(Config::builtin_for(platform), cfg, "确定性");
        assert_eq!(Config::from_json(&cfg.to_json()).unwrap(), cfg);
    }
}

#[test]
fn builtin_links_identical_gestures_in_other_scopes() {
    let cfg = Config::builtin_for(Platform::Windows);
    let linked_elsewhere = cfg.scopes.iter().filter(|s| s.kind != ScopeKind::Global).flat_map(|s| &s.bindings).filter(|b| b.gesture.is_some()).count();
    let identical_elsewhere = cfg
        .scopes
        .iter()
        .filter(|s| s.kind != ScopeKind::Global)
        .flat_map(|s| &s.bindings)
        .filter(|b| cfg.gestures.iter().any(|g| g.pattern == b.pattern))
        .count();
    assert_eq!(linked_elsewhere, identical_elsewhere);
}

// ================================================================ 整理导入分析

fn split_class(name: &str) -> (Vec<Vec<i32>>, Vec<Vec<i32>>) {
    let samples = real().remove(name).unwrap();
    let (a, b) = samples.split_at(samples.len() / 2);
    (a.to_vec(), b.to_vec())
}

/// 老用户配置：通用 + Chrome + 资源管理器，覆盖相同、等效折线、相似形状、不同形状、同程序区分、草稿。
fn old_user() -> Config {
    let (o1, o2) = split_class("O");
    let (c1, _) = split_class("C");
    let (s1, s2) = split_class("S");
    let mut draft = binding("draft", trace(&UP));
    draft.draft = true;
    config(vec![
        scope("explorer", ScopeKind::App, vec![binding("close", trace(&DOWN_RIGHT_LONG)), binding("up", trace(&UP))]),
        scope("general", ScopeKind::Global, vec![binding("back", trace(&LEFT)), binding("close", trace(&DOWN_RIGHT)), binding("menu", shape_pattern(&o1))]),
        scope(
            "chrome",
            ScopeKind::App,
            vec![
                binding("back", trace(&LEFT)),
                binding("new tab", shape_pattern(&o2)),
                binding("c", shape_pattern(&c1)),
                binding("s1", shape_pattern(&s1)),
                binding("s2", shape_pattern(&s2)),
                draft,
            ],
        ),
    ])
}

fn find_group<'a>(plan: &'a library::ImportPlan, cfg: &Config, scope: &str, name: &str) -> Option<&'a library::ImportGroup> {
    let si = cfg.scopes.iter().position(|s| s.id == scope).unwrap();
    let bi = cfg.scopes[si].bindings.iter().position(|b| b.name == name).unwrap();
    plan.groups.iter().find(|g| g.members.iter().any(|m| m.scope == si && m.binding == bi))
}

#[test]
fn analyze_from_global_groups_identical_equivalent_and_similar() {
    let cfg = old_user();
    let plan = library::analyze(&cfg, AdoptSource::Global);
    // 通用分组的 3 个手势各自成组，且通用优先作为组代表
    assert_eq!(plan.groups.len(), 3);
    let back = find_group(&plan, &cfg, "general", "back").unwrap();
    assert_eq!(back.kind, GroupKind::Identical);
    assert_eq!(back.members.iter().map(|m| m.relation).collect::<Vec<_>>(), [Relation::Seed, Relation::Identical]);

    let close = find_group(&plan, &cfg, "general", "close").unwrap();
    assert_eq!(close.kind, GroupKind::Equivalent);
    assert!(close.checked);
    assert_eq!(close.pattern, trace(&DOWN_RIGHT), "组代表来自「所有程序」");

    let menu = find_group(&plan, &cfg, "general", "menu").unwrap();
    assert_eq!(menu.kind, GroupKind::Similar);
    assert!(menu.checked && menu.score >= shape::ACCEPT_CURVE + shape::MARGIN, "{}", menu.score);
    let merged = menu.merged.as_ref().unwrap();
    assert!(merged.templates.len() <= library::MAX_SAMPLES);

    // C 与 O 不相似；资源管理器的 ↑ 在通用中没有对应 → 保持专用；草稿不参与
    assert!(find_group(&plan, &cfg, "chrome", "c").is_none());
    assert!(find_group(&plan, &cfg, "explorer", "up").is_none());
    assert!(find_group(&plan, &cfg, "chrome", "draft").is_none());
    assert!(plan.notes.is_empty());
}

#[test]
fn analyze_all_keeps_same_scope_shapes_apart_and_reports_it() {
    let cfg = old_user();
    let plan = library::analyze(&cfg, AdoptSource::All);
    let s1 = find_group(&plan, &cfg, "chrome", "s1").unwrap();
    let s2 = find_group(&plan, &cfg, "chrome", "s2").unwrap();
    assert!(!std::ptr::eq(s1, s2), "同一程序中的两个相似形状不合并");
    assert_eq!(plan.notes.len(), 1);
    let chrome = cfg.scopes.iter().position(|s| s.id == "chrome").unwrap();
    assert_eq!(plan.notes[0].scope, chrome);
    // 资源管理器的 ↑ 自成一组
    assert!(find_group(&plan, &cfg, "explorer", "up").is_some());
    // 结果与运行次数无关
    assert_eq!(serde_json::to_string(&plan).unwrap(), serde_json::to_string(&library::analyze(&cfg, AdoptSource::All)).unwrap());
}

#[test]
fn analyze_reuses_existing_library_and_reports_nothing_when_done() {
    let mut cfg = old_user();
    library::adopt_exact(&mut cfg, AdoptSource::All);
    // 精确建库后，只剩等效折线和相似形状可关联
    let plan = library::analyze(&cfg, AdoptSource::Global);
    assert!(plan.groups.iter().all(|g| g.existing.is_some()));
    assert!(plan.groups.iter().all(|g| g.kind != GroupKind::Identical));

    // 全部关联后不再有建议
    let mut linked = config(vec![scope("general", ScopeKind::Global, vec![binding("back", trace(&LEFT))])]);
    library::adopt_exact(&mut linked, AdoptSource::Global);
    assert!(library::analyze(&linked, AdoptSource::All).groups.is_empty());
}

#[test]
fn similar_shape_detection_matches_real_classes() {
    let classes = real();
    for (a, sa) in &classes {
        let (x, y) = sa.split_at(sa.len() / 2);
        assert!(shape::cross_similarity(x, y) >= shape::pair_threshold(x, y) + shape::MARGIN, "同类 {a} 应明显相似");
        for (b, sb) in &classes {
            if a != b {
                assert!(shape::cross_similarity(sa, sb) < shape::pair_threshold(sa, sb), "{a} 与 {b} 不应相似");
            }
        }
    }
}

#[test]
fn merged_samples_are_capped_and_deduplicated() {
    let classes = real();
    let o = &classes["O"];
    let merged = library::merge_samples([o.as_slice(), o.as_slice(), &o[..2]]);
    assert_eq!(merged.len(), library::MAX_SAMPLES.min(o.len()));
    let mut many = o.clone();
    many.extend(classes["C"].iter().cloned());
    let capped = library::merge_samples([many.as_slice()]);
    assert_eq!(capped.len(), library::MAX_SAMPLES);
}

#[test]
fn drafts_and_incomplete_strokes_are_never_adopted() {
    let mut draft = binding("draft", vec![right()]);
    draft.draft = true;
    let empty_trace = binding("empty", vec![right(), Step::Trace { points: vec![] }]);
    let single_click = binding("click", vec![Step::Press { key: PatternKey::Mouse(MouseButton::X1) }]);
    let mut cfg = config(vec![scope("general", ScopeKind::Global, vec![draft, empty_trace, single_click])]);
    assert_eq!(library::adopt_exact(&mut cfg, AdoptSource::Global), 1, "只有单击手势入库");
    assert_eq!(cfg.scopes[0].bindings[2].gesture.as_deref(), Some("g1"));
    assert!(cfg.scopes[0].bindings[0].gesture.is_none());
}

#[test]
fn shape_consistency_is_lowest_pairwise_similarity() {
    let o = &real()["O"];
    let c = shape::consistency(o).unwrap();
    let lowest = (0..o.len())
        .flat_map(|i| (i + 1..o.len()).map(move |j| (i, j)))
        .map(|(i, j)| shape::similarity(&shape::decode(&o[i]).unwrap(), &shape::decode(&o[j]).unwrap()))
        .fold(1.0f64, f64::min);
    assert!((c - lowest).abs() < 1e-12);
    assert_eq!(shape::consistency(&o[..1]), None);
}
