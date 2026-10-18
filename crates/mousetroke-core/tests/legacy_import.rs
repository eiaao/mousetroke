//! 导入器测试：旧格式 gestures.json / prefs.json → 原生 Mousetroke 配置。
//! fixtures 是本仓库自己编写的旧格式样例（带 BOM），覆盖导入器的各条转换路径。

use std::path::PathBuf;

use mousetroke_core::legacy;
use mousetroke_core::keyseq;
use mousetroke_core::recognizer::{match_polyline, Segment};
use mousetroke_core::schema::{Action, PatternKey, ScopeKind, Step, MouseButton};

fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("读取 {path:?} 失败: {e}"))
}

fn import() -> mousetroke_core::schema::Config {
    legacy::import(&fixture("gestures.json"), Some(&fixture("prefs.json")))
        .expect("旧格式配置导入失败")
}

/// 导入会按当前语言写内置名（i18n::lang 是进程全局），相关测试串行执行。
static LANG_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
macro_rules! lock_lang {
    () => {
        let _g = crate::LANG_LOCK.lock().unwrap();
    };
}

#[test]
fn legacy_names_are_localized() {
    lock_lang!();
    let cfg = import();
    let names: Vec<&str> = cfg.scopes.iter().flat_map(|s| s.bindings.iter().map(|b| b.name.as_str())).collect();
    let english: Vec<&&str> = names.iter().filter(|n| n.chars().any(|c| c.is_ascii_alphabetic()) && !n.contains("Ctrl") && !n.contains("Esc")).collect();
    assert!(english.is_empty(), "仍有英文名: {english:?}");
    assert_eq!(cfg.scopes[0].name, "所有程序");
    // New 按场景区分
    assert!(names.contains(&"新建标签页") && names.contains(&"新建窗口") && names.contains(&"新开命令行窗口"));
    // 自定义名不动
    let mut cfg = cfg;
    cfg.scopes[0].bindings[0].name = "我的复制".into();
    cfg.migrate();
    assert_eq!(cfg.scopes[0].bindings[0].name, "我的复制");
}

#[test]
fn imports_scope_structure() {
    lock_lang!();
    let cfg = import();
    // global + explorer + cmd + desktop = 4 个作用域
    assert_eq!(cfg.scopes.len(), 4);
    assert_eq!(cfg.scopes.iter().filter(|s| s.kind == ScopeKind::Global).count(), 1);
    assert_eq!(cfg.scopes.iter().filter(|s| s.kind == ScopeKind::App).count(), 2);
    assert_eq!(cfg.scopes.iter().filter(|s| s.kind == ScopeKind::Special).count(), 1);

    let global = cfg.scopes.iter().find(|s| s.kind == ScopeKind::Global).unwrap();
    assert_eq!(global.bindings.len(), 23, "全局作用域应有 23 条绑定");

    let explorer = cfg
        .scopes
        .iter()
        .find(|s| s.name == "资源管理器")
        .expect("应有 Windows Explorer 作用域");
    assert_eq!(explorer.bindings.len(), 4);
    assert!(explorer
        .app
        .as_ref()
        .unwrap()
        .exe
        .eq_ignore_ascii_case("c:\\windows\\explorer.exe"));
}

#[test]
fn imports_binding_semantics() {
    lock_lang!();
    let cfg = import();
    let global = cfg.scopes.iter().find(|s| s.kind == ScopeKind::Global).unwrap();

    // Copy = 按住右键 + 上划 → key_sequence ^(c)
    let copy = global.bindings.iter().find(|b| b.name == "复制").unwrap();
    assert_eq!(
        copy.pattern,
        vec![
            Step::Press { key: PatternKey::Mouse(MouseButton::Right) },
            Step::Trace { points: vec![0, 50, 0, 0] },
        ]
    );
    assert!(matches!(
        &copy.action,
        Action::KeySequence { keys, activate_target, .. }
            if keys == "^(c)" && !activate_target
    ));

    // Kill → shell（含旧格式变量名，变量替换 M4 接入）
    let kill = global.bindings.iter().find(|b| b.name == "强制结束程序").unwrap();
    assert!(matches!(&kill.action, Action::Shell { script } if script.contains("taskkill /F /PID %MS_PID%")));

    // 手势设置 → menu（嵌套 lua）
    let settings = global.bindings.iter().find(|b| b.name == "手势设置").unwrap();
    assert!(matches!(
        &settings.action,
        Action::Menu { columns, items } if *columns == 2 && items.len() == 2
    ));
    assert!(matches!(&settings.action, Action::Menu { items, .. }
        if matches!(&items[1].action, Action::Lua { .. })));

    // Web Search → open_url
    let search = global.bindings.iter().find(|b| b.name == "搜索选中文字").unwrap();
    assert!(matches!(
        &search.action,
        Action::OpenUrl { url_template, .. }
            if url_template.contains("bing.com") && url_template.contains("%MS_SELECTED_TEXT%")
    ));

    // Minimize → window_op
    let minimize = global.bindings.iter().find(|b| b.name == "最小化").unwrap();
    assert!(matches!(&minimize.action, Action::WindowOp { .. }));

    // 纯按键模式：X2 侧键 = Prev Tab（旧格式有两个同名绑定，取纯按键的那个）
    let prev_tab = global
        .bindings
        .iter()
        .find(|b| b.name == "上一个标签页" && b.pattern.len() == 1)
        .unwrap();
    assert_eq!(prev_tab.pattern, vec![Step::Press { key: PatternKey::Mouse(MouseButton::X2) }]);
    let next_tab = global
        .bindings
        .iter()
        .find(|b| b.name == "下一个标签页" && b.pattern.len() == 1)
        .unwrap();
    assert_eq!(next_tab.pattern, vec![Step::Press { key: PatternKey::Mouse(MouseButton::X1) }]);
}

#[test]
fn imports_prefs_to_settings() {
    lock_lang!();
    let cfg = import();
    let s = &cfg.settings;
    assert_eq!(s.drag_timeout_ms, 450);
    assert_eq!(s.min_stroke_px, 25);
    assert_eq!(s.pause_hotkey, "#({ESC})");
    assert_eq!(s.label.pos, 0.3);
    assert!(s.trail.fade_out);
    // 旧格式默认外观（1.5px、19% 黑，靠描边才看得见）→ 保留 Mousetroke 默认，免得导入后轨迹/手势名看不见
    let d = mousetroke_core::schema::Settings::default();
    assert_eq!(s.trail.width, d.trail.width);
    assert_eq!(s.trail.color_normal, d.trail.color_normal);
    assert_eq!(s.label.color_normal, d.label.color_normal);
    // 命中色是用户可见的强调色，照常导入
    assert_eq!(s.trail.color_matched, "#2E9BFFFF");
}

#[test]
fn all_imported_key_sequences_parse() {
    lock_lang!();
    let cfg = import();
    let mut count = 0;
    for binding in cfg.all_bindings() {
        binding.action.visit(&mut |action| {
            if let Action::KeySequence { keys, .. } = action {
                keyseq::parse(keys, false)
                    .unwrap_or_else(|e| panic!("按键序列解析失败 {keys:?}: {e:?}"));
                count += 1;
            }
        });
    }
    assert!(count >= 20, "旧格式 key_sequence 动作应 ≥ 20，实际 {count}");
}

#[test]
fn native_config_round_trips() {
    lock_lang!();
    let cfg = import();
    let json = serde_json::to_string_pretty(&cfg).unwrap();
    let re: mousetroke_core::schema::Config = serde_json::from_str(&json).unwrap();
    assert_eq!(cfg, re, "原生配置往返应无损");
}

#[test]
fn imported_bindings_recognize_paths() {
    lock_lang!();
    // 端到端：旧格式导入的绑定与量化折线匹配
    let cfg = import();
    let global = cfg.scopes.iter().find(|s| s.kind == ScopeKind::Global).unwrap();

    let copy = global.bindings.iter().find(|b| b.name == "复制").unwrap();
    let pts = trace_points(copy).expect("Copy 应含 trace");
    assert!(match_polyline(pts, &[Segment { x: 0, y: -1 }]).unwrap() < mousetroke_core::recognizer::EXACT_EPS); // ↑

    let snip = global.bindings.iter().find(|b| b.name == "截图").unwrap();
    let pts = trace_points(snip).expect("截图应含 trace");
    let box_path = [
        Segment { x: 0, y: -1 },
        Segment { x: 1, y: 0 },
        Segment { x: 0, y: 1 },
        Segment { x: -1, y: 0 },
    ];
    assert!(match_polyline(pts, &box_path).unwrap() < mousetroke_core::recognizer::EXACT_EPS); // ↑→↓← 画方框
}

fn trace_points(b: &mousetroke_core::schema::Binding) -> Option<&[i32]> {
    b.pattern.iter().find_map(|s| match s {
        Step::Trace { points } => Some(points.as_slice()),
        _ => None,
    })
}

// ---------------------------------------------------------------- 边角 + 组合手势

mod gestures {
    use super::import;
    use mousetroke_core::edge;
    use mousetroke_core::matcher::{build_steps, is_feasible_prefix, match_binding, Trigger};
    use mousetroke_core::recognizer::PathTracker;
    use mousetroke_core::schema::{MouseButton, PatternKey, Scope, ScopeKind};

    fn global() -> Scope {
        import().scopes.into_iter().find(|s| s.kind == ScopeKind::Global).unwrap()
    }

    fn trace(points: &[(f64, f64)]) -> Vec<mousetroke_core::recognizer::Segment> {
        let mut t = PathTracker::new(30.0);
        t.start(points[0].0, points[0].1);
        for w in points.windows(2) {
            for i in 1..=10 {
                let k = i as f64 / 10.0;
                t.add_point(w[0].0 + (w[1].0 - w[0].0) * k, w[0].1 + (w[1].1 - w[0].1) * k);
            }
        }
        t.segments().to_vec()
    }

    #[test]
    fn corners_map_to_default_semantics() {
        lock_lang!();
        let g = global();
        let pure = |e| match_binding(&g, &build_steps(Trigger::Edge(e), &[], None), &[]).map(|b| b.name.clone());
        assert_eq!(pure(edge::LEFT_BOTTOM).as_deref(), Some("开始菜单"));
        assert_eq!(pure(edge::RIGHT_BOTTOM).as_deref(), Some("显示桌面"));
        assert_eq!(pure(edge::LEFT_TOP).as_deref(), Some("切换程序"));
        assert_eq!(pure(edge::RIGHT_TOP).as_deref(), Some("任务切换器"));
    }

    #[test]
    fn top_edge_wheel_is_volume_and_clicks_pass_through() {
        lock_lang!();
        let g = global();
        let top = |p: &[PatternKey]| build_steps(Trigger::Edge(edge::TOP), p, None);
        // 旧格式约定：负 = 向上
        assert_eq!(match_binding(&g, &top(&[PatternKey::Wheel(-1)]), &[]).unwrap().name, "音量加");
        assert_eq!(match_binding(&g, &top(&[PatternKey::Wheel(1)]), &[]).unwrap().name, "音量减");
        // 上边缘点左键（点浏览器标签页）不属于任何手势 → 引擎放行
        assert!(!is_feasible_prefix(&g, &top(&[PatternKey::Mouse(MouseButton::Left)])));
    }

    #[test]
    fn right_edge_rub_opens_task_manager() {
        lock_lang!();
        let g = global();
        assert!(is_feasible_prefix(&g, &build_steps(Trigger::Edge(edge::RIGHT), &[], Some(0))));
        let segs = trace(&[(1919.0, 500.0), (1919.0, 400.0), (1919.0, 500.0), (1919.0, 400.0), (1919.0, 500.0)]);
        let b = match_binding(&g, &build_steps(Trigger::Edge(edge::RIGHT), &[], Some(0)), &segs).unwrap();
        assert_eq!(b.name, "任务管理器");
        assert!(b.execute_on_match);
    }

    #[test]
    fn top_edge_sequence_with_trace_in_middle() {
        lock_lang!();
        let g = global();
        let (r, l) = (PatternKey::Mouse(MouseButton::Right), PatternKey::Mouse(MouseButton::Left));
        let segs = trace(&[(800.0, 0.0), (800.0, 200.0)]);
        let steps = build_steps(Trigger::Edge(edge::TOP), &[r, l], Some(1));
        assert_eq!(match_binding(&g, &steps, &segs).unwrap().name, "系统设置");
        let steps = build_steps(Trigger::Edge(edge::TOP), &[r], None);
        assert_eq!(match_binding(&g, &steps, &[]).unwrap().name, "播放/暂停");
    }
}

#[test]
fn load_migrates_legacy_placeholders() {
    lock_lang!();
    let legacy = r#"{"version":1,"scopes":[{"id":"g","name":"G","kind":"global","bindings":[
        {"name":"s","pattern":[{"type":"press","key":"mouse:right"}],
         "action":{"type":"open_url","url_template":"https://baidu.com/s?wd={WG_SELECTED_TEXT}"}}]}]}"#;
    let cfg = mousetroke_core::schema::Config::from_json(legacy).unwrap();
    match &cfg.scopes[0].bindings[0].action {
        Action::OpenUrl { url_template, .. } => assert_eq!(url_template, "https://baidu.com/s?wd=%MS_SELECTED_TEXT%"),
        other => panic!("{other:?}"),
    }
}

/// 内置名称的语言行为（docs/I18N-DESIGN.md §5）：
/// - 导入时 zh 下默认名是中文、en 下是英文（由 name_key 翻译）；
/// - 用户改过的名字不受语言影响（没有 name_key）；
/// - 早期被改写成中文名、无 name_key 的旧配置，加载时补回 name_key（name 原样不动）。
#[test]
fn builtin_names_follow_language_and_backfill() {
    lock_lang!();
    use mousetroke_core::i18n::{self, Lang};
    use mousetroke_core::schema::Config;

    // en-US 下导入：默认名是英文，且带 name_key
    i18n::set_lang(Lang::EnUs);
    let cfg = import();
    assert_eq!(cfg.scopes[0].name, "All apps");
    assert_eq!(cfg.scopes[0].name_key.as_deref(), Some("builtin.scope.global"));
    let copy = cfg.scopes[0].bindings.iter().find(|b| b.name == "Copy").unwrap();
    assert_eq!(copy.name_key.as_deref(), Some("builtin.gesture.copy"));

    // zh-CN 下导入：同样的默认名变成中文
    i18n::set_lang(Lang::ZhCn);
    let mut cfg = import();
    assert_eq!(cfg.scopes[0].name, "所有程序");
    let copy = cfg.scopes[0].bindings.iter().find(|b| b.name == "复制").unwrap();
    assert_eq!(copy.name_key.as_deref(), Some("builtin.gesture.copy"));

    // 用户改过的名字：加载/导入都不动它，也没有 name_key（显示不随语言变）
    let snapshot = cfg.clone(); // 留给下面的旧配置测试
    let i = cfg.scopes[0].bindings.iter().position(|b| b.name == "复制").unwrap();
    cfg.scopes[0].bindings[i].name = "我的手势".into();
    cfg.scopes[0].bindings[i].name_key = None;
    let reloaded = Config::from_json(&cfg.to_json()).unwrap();
    let mine = &reloaded.scopes[0].bindings[i];
    assert_eq!(mine.name, "我的手势");
    assert!(mine.name_key.is_none());

    // 旧配置：默认名已被早期版本改写成中文、无 name_key → 加载时补回
    let mut old: serde_json::Value = serde_json::from_str(&snapshot.to_json()).unwrap();
    for s in old["scopes"].as_array_mut().unwrap() {
        s.as_object_mut().unwrap().remove("name_key");
        for b in s["bindings"].as_array_mut().unwrap() {
            b.as_object_mut().unwrap().remove("name_key");
        }
    }
    let migrated = Config::from_json(&serde_json::to_string(&old).unwrap()).unwrap();
    let copy2 = migrated.scopes[0].bindings.iter().find(|b| b.name == "复制").unwrap();
    assert_eq!(copy2.name_key.as_deref(), Some("builtin.gesture.copy"), "旧配置应补回 name_key");
    assert_eq!(copy2.name, "复制", "补 key 时 name 原样不动");
}
