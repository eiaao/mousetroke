//! 引擎状态机测试：假绘制面 + 假宿主，记录补发/执行/画线，不碰真实输入。

use std::cell::RefCell;
use std::rc::Rc;

use mousetroke_core::edge::{LEFT_BOTTOM, TOP};
use mousetroke_core::schema::{Action, Binding, Config, MouseButton as B, PatternKey, Scope, ScopeKind, Step};

use super::*;

#[derive(Default)]
struct Log {
    replays: Vec<(B, bool)>,
    executed: Vec<String>,
    lines: usize,
    finishes: usize,
    fades: usize,
    label_clears: usize,
    labels: Vec<String>,
    samples: Vec<Vec<i32>>,
}

type Shared = Rc<RefCell<Log>>;

struct FakeCanvas(Shared);

impl Canvas for FakeCanvas {
    fn set_color(&mut self, _: &str) {}
    fn set_width(&mut self, _: f64) {}
    fn start_session(&mut self, _: i32, _: i32) {}
    fn line_to(&mut self, _: i32, _: i32) {
        self.0.borrow_mut().lines += 1;
    }
    fn finish(&mut self) {
        self.0.borrow_mut().finishes += 1;
    }
    fn fade_out(&mut self) {
        self.0.borrow_mut().fades += 1;
    }
    fn tick_fade(&mut self) {}
    fn draw_label(&mut self, text: &str, _: i32, _: i32, _: &str) {
        self.0.borrow_mut().labels.push(text.into());
    }
    fn clear_label(&mut self) {
        self.0.borrow_mut().label_clears += 1;
    }
    fn virtual_rect(&self) -> (i32, i32, i32, i32) {
        (0, 0, 1920, 1080)
    }
}

/// 鼠标下的窗口：(类名, exe)。
type Under = Rc<RefCell<(String, Option<String>)>>;

struct FakeHost {
    log: Shared,
    under: Under,
}

impl Host for FakeHost {
    fn target_at(&self, _: i32, _: i32) -> Target {
        let (class, exe) = self.under.borrow().clone();
        Target { wid: 0, pid: 0, exe, is_desktop: false, class }
    }
    fn ignore_target(&self, target: &Target) -> bool {
        // 假宿主按 Windows 用例标记需要放行的窗口；平台判定留在宿主。
        ["Shell_TrayWnd", "NotifyIconOverflowWindow", "TopLevelWindowForOverflowXamlIsland", "#32768"].contains(&target.class.as_str())
            || target.exe.as_deref().is_some_and(|e| matcher::file_name(e).eq_ignore_ascii_case("mousetroke-settings.exe"))
    }
    fn window_title(&self, _: usize) -> String {
        String::new()
    }
    fn cursor_pos(&self) -> (i32, i32) {
        (960, 540)
    }
    fn system_langs(&self) -> Vec<String> {
        Vec::new()
    }
    fn refresh_texts(&self) {}
    fn foreground_fullscreen(&self) -> bool {
        false
    }
    fn replay(&self, btn: B, click: bool) {
        self.log.borrow_mut().replays.push((btn, click));
    }
    fn execute(&self, action: Action, _: Option<Target>, _: Vars, _: (i32, i32)) {
        if let Action::Shell { script } = action {
            self.log.borrow_mut().executed.push(script);
        }
    }
    fn label_anchor(&self, _: i32, _: i32) -> (i32, i32) {
        (960, 864)
    }
    fn new_canvas(&self, _: &str, _: f64) -> Option<Box<dyn Canvas>> {
        Some(Box::new(FakeCanvas(self.log.clone())))
    }
    fn shape_sample(&self, encoded: Vec<i32>) {
        self.log.borrow_mut().samples.push(encoded);
    }
}

fn bind(name: &str, pattern: Vec<Step>) -> Binding {
    Binding {
        name: name.into(),
        name_key: None,
        pattern,
        action: Action::Shell { script: name.into() },
        execute_on_match: false,
        hide_label: false,
        secured: false,
        disabled: false,
    }
}

fn press(k: PatternKey) -> Step {
    Step::Press { key: k }
}
const RIGHT: Step = Step::Press { key: PatternKey::Mouse(B::Right) };
fn trace(points: &[i32]) -> Step {
    Step::Trace { points: points.to_vec() }
}
const DOWN: [i32; 4] = [0, 0, 0, 50];
const DOWN_RIGHT: [i32; 6] = [0, 0, 0, 50, 50, 50];

/// 测试配置（接近旧格式默认的几类手势）。
fn config(with_edge_combo: bool) -> Config {
    let mut bindings = vec![
        bind("down", vec![RIGHT, trace(&DOWN)]),
        bind("next_tab", vec![RIGHT, press(PatternKey::Wheel(1))]),
        bind("vol_up", vec![Step::Edge { edge: TOP }, press(PatternKey::Wheel(-1))]),
        bind("start", vec![Step::Edge { edge: LEFT_BOTTOM }]),
    ];
    if with_edge_combo {
        // 旧格式「系统设置」：上边 + 右键 + ↓ + 左键
        bindings.push(bind(
            "settings",
            vec![Step::Edge { edge: TOP }, RIGHT, trace(&DOWN), press(PatternKey::Mouse(B::Left))],
        ));
    }
    let mut cfg = Config::default();
    cfg.settings.language = "zh-CN".into(); // 语言相关的断言不依赖系统设置
    cfg.settings.drag_timeout_ms = 60;
    cfg.scopes = vec![Scope {
        id: "global".into(),
        name: "所有程序".into(),
        name_key: None,
        kind: ScopeKind::Global,
        app: None,
        special: None,
        inherit_global: true,
        disabled: false,
        bindings,
    }];
    cfg
}

/// 当前语言是进程全局，而每个引擎在 apply_config 时都会 set_lang：
/// T 在整个生命周期里持有这把锁，引擎测试因此串行，语言断言不会被并行测试改掉。
static LANG_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

struct T {
    e: Engine,
    log: Shared,
    under: Under,
    pos: (i32, i32),
    _lang: std::sync::MutexGuard<'static, ()>,
}

impl T {
    /// 直接设置当前语言（绕过 apply_config 的解析，测试专用）。
    fn set_lang(&self, l: mousetroke_core::i18n::Lang) {
        mousetroke_core::i18n::set_lang(l);
    }

    fn new(with_edge_combo: bool) -> Self {
        Self::with_config(config(with_edge_combo))
    }

    fn with_config(cfg: Config) -> Self {
        Self::with_samples(cfg, false)
    }

    fn with_samples(cfg: Config, shape_samples: bool) -> Self {
        // 某个测试失败导致锁中毒时照样拿锁，别把后面的测试连带弄红
        let lang = LANG_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let log = Shared::default();
        let under: Under = Rc::new(RefCell::new(("Chrome_WidgetWin_1".into(), Some(r"C:\app\chrome.exe".into()))));
        let host = FakeHost { log: log.clone(), under: under.clone() };
        let e = Engine::with_parts(cfg, None, Box::new(FakeCanvas(log.clone())), Box::new(host), shape_samples);
        Self { e, log, under, pos: (960, 540), _lang: lang }
    }
    fn under(&self, class: &str, exe: Option<&str>) {
        *self.under.borrow_mut() = (class.into(), exe.map(Into::into));
    }
    fn down(&mut self, btn: B) -> Verdict {
        let (x, y) = self.pos;
        self.e.handle(InputEvent::MouseDown { btn, x, y })
    }
    fn up(&mut self, btn: B) -> Verdict {
        let (x, y) = self.pos;
        self.e.handle(InputEvent::MouseUp { btn, x, y })
    }
    fn to(&mut self, x: i32, y: i32) -> Verdict {
        self.pos = (x, y);
        self.e.handle(InputEvent::MouseMove { x, y })
    }
    /// 每 10px 一个移动事件，平滑地拖到 (x, y)。
    fn drag(&mut self, x: i32, y: i32) {
        let (x0, y0) = self.pos;
        let n = ((x - x0).abs().max((y - y0).abs()) / 10).max(1);
        for i in 1..=n {
            self.to(x0 + (x - x0) * i / n, y0 + (y - y0) * i / n);
        }
    }
    fn wheel(&mut self, d: i32) -> Verdict {
        self.e.handle(InputEvent::Wheel(d))
    }
    /// 到达边/角并驻留，触发边缘检测。
    fn dwell(&mut self, x: i32, y: i32) {
        self.to(x, y);
        std::thread::sleep(EDGE_DWELL + Duration::from_millis(10));
        self.to(x, y);
    }
    fn executed(&self) -> Vec<String> {
        self.log.borrow().executed.clone()
    }
    fn replays(&self) -> Vec<(B, bool)> {
        self.log.borrow().replays.clone()
    }
    fn lines(&self) -> usize {
        self.log.borrow().lines
    }
    /// 每次反向移动形成一段，按键和沿边摩擦共用。
    fn zigzag(&mut self, segments: usize) {
        let (x, y) = self.pos;
        for i in 0..segments {
            self.to(if i % 2 == 0 { x + 120 } else { x }, y);
        }
    }
    fn draw_path(&mut self, path: &[(i32, i32)]) {
        self.to(path[0].0, path[0].1);
        assert_eq!(self.down(B::Right), Verdict::Swallow);
        for &(x, y) in &path[1..] {
            self.to(x, y);
        }
    }
}

/// 与 core 的 G 样本相同：逆时针 C 弧线，向上回钩，再向左收笔。
fn g_path(count: usize, noise: bool) -> Vec<(i32, i32)> {
    use std::f64::consts::PI;
    let arc = |t: f64| {
        let a = PI / 4.0 + 1.5 * PI * t;
        (a.cos(), -a.sin())
    };
    let lerp = |a: (f64, f64), b: (f64, f64), t: f64| (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
    (0..=count).map(|i| {
        let t = i as f64 / count as f64;
        let (x, y) = if t < 0.78 { arc(t / 0.78) }
            else if t < 0.89 { lerp(arc(1.0), (0.82, 0.0), (t - 0.78) / 0.11) }
            else { lerp((0.82, 0.0), (0.05, 0.0), (t - 0.89) / 0.11) };
        let (jx, jy) = if noise { ((i as f64 * 1.73).sin() * 3.0, (i as f64 * 2.91).sin() * 3.0) } else { (0.0, 0.0) };
        ((960.0 + x * 160.0 + jx).round() as i32, (540.0 + y * 160.0 + jy).round() as i32)
    }).collect()
}

fn shape_step(path: &[(i32, i32)]) -> Step {
    let raw = path.iter().map(|&(x, y)| (x as f64, y as f64)).collect::<Vec<_>>();
    Step::shape_from_samples(&[raw]).unwrap()
}

fn shape_config() -> Config {
    let mut cfg = config(false);
    cfg.scopes[0].bindings.extend([
        bind("G", vec![RIGHT, shape_step(&g_path(160, false))]),
        bind("close", vec![RIGHT, trace(&DOWN_RIGHT)]),
    ]);
    cfg
}

#[test]
fn shape_g_straight_scribble_and_plain_click() {
    let mut t = T::with_config(shape_config());
    t.draw_path(&g_path(80, true));
    assert!(t.executed().is_empty());
    t.up(B::Right);
    assert_eq!(t.executed(), ["G"]);
    assert_eq!(t.log.borrow().labels.last().map(String::as_str), Some("G"));

    t.to(960, 540);
    t.down(B::Right);
    t.drag(960, 740);
    t.drag(1160, 740);
    t.up(B::Right);
    assert_eq!(t.executed(), ["G", "close"]);

    let scribble: Vec<_> = (0..150).map(|i| {
        ((960.0 + (i as f64 * 1.73).sin() * 150.0) as i32, (540.0 + (i as f64 * 2.91).sin() * 150.0) as i32)
    }).collect();
    t.draw_path(&scribble);
    t.up(B::Right);
    assert_eq!(t.executed(), ["G", "close"]);
    assert_eq!(t.log.borrow().labels.last().map(String::as_str), Some("画嗨了吧？这一笔不算～"));
    assert!(t.replays().is_empty());

    t.down(B::Right);
    t.to(t.pos.0 + 1, t.pos.1 + 1);
    t.up(B::Right);
    assert_eq!(t.replays(), [(B::Right, true)]);
    assert_eq!(echo_back(&mut t, true), 1);
    assert_eq!(t.executed(), ["G", "close"]);
}

#[test]
fn shape_execute_on_match_waits_for_release() {
    let mut cfg = shape_config();
    cfg.scopes[0].bindings.iter_mut().find(|b| b.name == "G").unwrap().execute_on_match = true;
    let mut t = T::with_config(cfg);
    t.draw_path(&g_path(160, false));
    t.e.after_input();
    assert!(t.executed().is_empty());
    assert_eq!(t.e.capture.as_ref().unwrap().state, CaptureState::Active);
    assert_eq!(t.log.borrow().labels.last().map(String::as_str), Some("G"));
    t.up(B::Right);
    assert_eq!(t.executed(), ["G"]);
    assert!(t.replays().is_empty());
}

#[test]
fn partial_g_does_not_execute_a_c_shape() {
    let g = g_path(200, false);
    let mut cfg = shape_config();
    let mut c = bind("C", vec![RIGHT, shape_step(&g[..=156])]);
    c.execute_on_match = true;
    cfg.scopes[0].bindings.push(c);
    let mut t = T::with_config(cfg);
    t.draw_path(&g[..=156]);
    t.e.after_input();
    assert_eq!(t.log.borrow().labels.last().map(String::as_str), Some("C"));
    assert!(t.executed().is_empty());
    for &(x, y) in &g[157..] { t.to(x, y); }
    t.up(B::Right);
    assert_eq!(t.executed(), ["G"]);
}

#[test]
fn shape_with_wheel_waits_for_release() {
    let mut cfg = shape_config();
    cfg.scopes[0].bindings.retain(|b| b.name != "G");
    let mut b = bind("G-wheel", vec![RIGHT, shape_step(&g_path(160, false)), press(PatternKey::Wheel(1))]);
    b.execute_on_match = true;
    cfg.scopes[0].bindings.push(b);
    let mut t = T::with_config(cfg);
    t.draw_path(&g_path(80, true));
    assert_eq!(t.wheel(1), Verdict::Swallow);
    assert!(t.executed().is_empty(), "滚轮不能让形状在笔画结束前执行");
    t.up(B::Right);
    assert_eq!(t.executed(), ["G-wheel"]);
}

#[test]
fn app_shape_wheel_precedes_global_straight_wheel() {
    let mut cfg = shape_config();
    cfg.scopes[0].bindings.push(bind("global-wheel", vec![RIGHT, trace(&DOWN_RIGHT), press(PatternKey::Wheel(1))]));
    let mut app = cfg.scopes[0].clone();
    app.kind = ScopeKind::App;
    app.app = Some(mousetroke_core::schema::AppMatch { exe: "chrome.exe".into(), win_class: None, win_title: None, regex: false });
    app.bindings = vec![bind("app-wheel", vec![RIGHT, shape_step(&[(960, 540), (960, 740), (1160, 740)]), press(PatternKey::Wheel(1))])];
    cfg.scopes.push(app);
    let mut t = T::with_config(cfg);
    t.down(B::Right);
    t.drag(960, 740);
    t.drag(1160, 740);
    assert_eq!(t.wheel(1), Verdict::Swallow);
    assert!(t.executed().is_empty());
    t.up(B::Right);
    assert_eq!(t.executed(), ["app-wheel"]);
}

#[test]
fn shape_capture_still_allows_drag_timeout_and_escape() {
    let mut t = T::with_samples(shape_config(), true);
    t.down(B::Right);
    t.e.capture.as_mut().unwrap().started = Instant::now() - Duration::from_millis(100);
    assert_eq!(t.to(961, 540), Verdict::Pass);
    assert_eq!(t.replays(), [(B::Right, false)]);
    assert_eq!(t.up(B::Right), Verdict::Pass);
    assert!(t.log.borrow().samples.is_empty());
    t.draw_path(&g_path(160, false));
    assert_eq!(t.e.handle(InputEvent::KeyDown(0x1B)), Verdict::Swallow);
    t.up(B::Right);
    assert!(t.executed().is_empty());
    assert_eq!(t.replays(), [(B::Right, false)]);
}

#[test]
fn app_shape_precedes_global_straight_and_allows_fallback() {
    let mut cfg = shape_config();
    let down_right = [(960, 540), (960, 740), (1160, 740)];
    let mut app = cfg.scopes[0].clone();
    app.kind = ScopeKind::App;
    app.app = Some(mousetroke_core::schema::AppMatch { exe: "chrome.exe".into(), win_class: None, win_title: None, regex: false });
    app.bindings = vec![bind("app-shape", vec![RIGHT, shape_step(&down_right)])];
    cfg.scopes.push(app);
    let mut t = T::with_config(cfg);
    t.to(960, 540);
    t.down(B::Right);
    t.drag(960, 740);
    t.drag(1160, 740);
    t.up(B::Right);
    assert_eq!(t.executed(), ["app-shape"], "应用形状优先于全局精确折线");

    t.e.cfg.scopes[1].bindings[0].pattern[1] = shape_step(&g_path(160, false));
    t.to(960, 540);
    t.down(B::Right);
    t.drag(960, 740);
    t.drag(1160, 740);
    t.up(B::Right);
    assert_eq!(t.executed(), ["app-shape", "close"]);
    t.draw_path(&g_path(80, true));
    t.up(B::Right);
    assert_eq!(t.executed(), ["app-shape", "close", "app-shape"]);
}

#[test]
fn disabled_shape_and_app_gestures_off_are_respected() {
    let mut cfg = shape_config();
    cfg.scopes[0].bindings.iter_mut().find(|b| b.name == "G").unwrap().disabled = true;
    let mut app = cfg.scopes[0].clone();
    app.kind = ScopeKind::App;
    app.app = Some(mousetroke_core::schema::AppMatch { exe: "game.exe".into(), win_class: None, win_title: None, regex: false });
    app.disabled = true;
    cfg.scopes.push(app);
    let mut t = T::with_config(cfg);
    t.draw_path(&g_path(160, false));
    t.up(B::Right);
    assert!(t.executed().is_empty());
    assert!(t.log.borrow().labels.last().unwrap().starts_with("未识别"));
    t.under("Game", Some(r"C:\game.exe"));
    assert_eq!(t.down(B::Right), Verdict::Pass);
    for (x, y) in g_path(80, true) { t.to(x, y); }
    assert_eq!(t.up(B::Right), Verdict::Pass);
    assert!(t.executed().is_empty());
    assert!(t.replays().is_empty());
}

#[test]
fn shape_edge_button_reset_and_plain_click_fallback() {
    let mut cfg = shape_config();
    cfg.scopes[0].bindings.push(bind("edge-G", vec![Step::Edge { edge: TOP }, RIGHT, shape_step(&g_path(160, false))]));
    let mut t = T::with_config(cfg);
    t.dwell(960, 0);
    // 首次按键之前的边缘轨迹不能混入按下后的形状。
    t.to(961, 0);
    t.down(B::Right);
    assert_eq!(t.e.path, [(961.0, 0.0)]);
    t.up(B::Right);
    assert_eq!(t.replays(), [(B::Right, true)]);

    let g = g_path(160, false);
    t.to(900, 300);
    t.dwell(g[0].0, 0);
    t.down(B::Right);
    let y0 = g[0].1;
    for &(x, y) in &g[1..] { t.to(x, y - y0); }
    assert!(t.executed().is_empty());
    t.up(B::Right);
    assert_eq!(t.executed(), ["edge-G"]);
}

#[test]
fn raw_path_skips_nearby_points_decimates_and_resets() {
    let mut t = T::with_config(shape_config());
    t.down(B::Right);
    let storage = t.e.path.as_ptr();
    t.to(961, 541);
    assert_eq!(t.e.path.len(), 1);
    t.to(962, 540);
    assert_eq!(t.e.path.len(), 2);
    for i in 2..SHAPE_PATH_CAP { t.to(960 + i as i32 * 2, 540); }
    assert_eq!(t.e.path.len(), SHAPE_PATH_CAP);
    let before = t.e.path.clone();
    t.to(960 + SHAPE_PATH_CAP as i32 * 2, 540);
    assert_eq!(t.e.path.len(), SHAPE_PATH_CAP / 2 + 1);
    for (kept, original) in t.e.path.iter().zip(before.iter().step_by(2)) { assert_eq!(kept, original); }
    assert_eq!(t.e.path.as_ptr(), storage);
    t.up(B::Right);
    t.draw_path(&g_path(160, false));
    assert_eq!(t.e.path.first(), Some(&(g_path(160, false)[0].0 as f64, g_path(160, false)[0].1 as f64)));
    assert_eq!(t.e.path.as_ptr(), storage);
    t.up(B::Right);
    assert_eq!(t.executed(), ["G"]);
}

#[test]
fn raw_path_stays_bounded_for_50000_moves_and_finishes() {
    let mut t = T::with_config(shape_config());
    t.down(B::Right);
    let storage = t.e.path.as_ptr();
    for i in 1..=50_000 {
        t.to(960 + i * 3, 540);
        assert!(t.e.path.len() <= SHAPE_PATH_CAP);
        assert_eq!(t.e.path.capacity(), SHAPE_PATH_CAP);
        assert_eq!(t.e.path.as_ptr(), storage);
    }
    for (x, y) in g_path(160, false) { t.to(x + 150_000, y); }
    assert!(t.e.path.len() <= SHAPE_PATH_CAP);
    let start = Instant::now();
    t.up(B::Right);
    assert!(start.elapsed() < Duration::from_millis(100));
    assert!(t.e.capture.is_none());
    assert!(t.replays().is_empty());
}

#[test]
fn shape_samples_switch_logs_once_only_for_strokes() {
    for enabled in [false, true] {
        let mut t = T::with_samples(shape_config(), enabled);
        t.down(B::Right);
        t.to(962, 541);
        t.up(B::Right);
        assert!(t.log.borrow().samples.is_empty());
        t.draw_path(&g_path(160, false));
        assert!(t.log.borrow().samples.is_empty());
        let expected = shape::encode(&shape::normalize(&t.e.path).unwrap());
        t.up(B::Right);
        assert_eq!(t.log.borrow().samples, if enabled { vec![expected] } else { vec![] });
        t.up(B::Right);
        assert_eq!(t.log.borrow().samples.len(), enabled as usize);
        // 未识别的笔画也用于校准，不只记录命中的 G。
        t.to(960, 540);
        t.down(B::Right);
        t.drag(1200, 540);
        t.up(B::Right);
        assert_eq!(t.log.borrow().samples.len(), 2 * enabled as usize);
    }
}

#[test]
#[ignore = "本地 release 校准：满路径与 100 个形状模板的完整释放处理"]
fn shape_finish_timing_100_templates() {
    use mousetroke_core::shape::{self, StrokeKind};
    for (kind, g) in [(StrokeKind::Shape, g_path(160, false)), (StrokeKind::Polyline, vec![(0, 0), (320, 0)])] {
        let normalized = shape::normalize(&g.iter().map(|&(x, y)| (x as f64, y as f64)).collect::<Vec<_>>()).unwrap();
        assert_eq!(shape::classify_normalized_stroke(&normalized), kind);
        let mut cfg = shape_config();
        cfg.scopes[0].bindings = (0..100).map(|i| bind(&format!("shape-{i}"), vec![RIGHT, shape_step(&g)])).collect();
        let mut t = T::with_config(cfg);
        let full_path: Vec<_> = (0..SHAPE_PATH_CAP).map(|i| {
            let position = i as f64 * (g.len() - 1) as f64 / (SHAPE_PATH_CAP - 1) as f64;
            let index = (position as usize).min(g.len() - 2);
            let (a, b, fraction) = (g[index], g[index + 1], position - index as f64);
            (((a.0 as f64 + (b.0 - a.0) as f64 * fraction) * 10.0).round(),
                ((a.1 as f64 + (b.1 - a.1) as f64 * fraction) * 10.0).round())
        }).collect();
        for accepted in [false, true] {
            if accepted {
                let mut b = bind("100-samples", vec![RIGHT, shape_step(&g)]);
                if let Step::Shape { templates } = &mut b.pattern[1] { templates.resize(100, templates[0].clone()); }
                t.e.cfg.scopes[0].bindings = vec![b];
            }
            let mut times = Vec::new();
            for _ in 0..101 {
                // 计时包含归一化、64 点分类与完整释放回调；满路径无重复点，100 个模板均参与评分。
                t.draw_path(&g);
                t.e.path.clone_from(&full_path);
                let start = Instant::now();
                std::hint::black_box(t.up(B::Right));
                times.push(start.elapsed());
            }
            times.sort();
            println!("full-cap path + classification + 100 shape templates ({kind:?}, accepted={accepted}): median {:.3} us, maximum {:.3} us (101 releases)", times[50].as_secs_f64() * 1e6, times[100].as_secs_f64() * 1e6);
            assert!(times[100] < Duration::from_millis(1), "释放回调应远低于 1ms: {:?}", times[100]);
            assert_eq!(t.executed().len(), if accepted { 101 } else { 0 });
        }
    }
}

#[test]
fn right_gesture_executes_and_swallows() {
    let mut t = T::new(false);
    assert_eq!(t.down(B::Right), Verdict::Swallow);
    t.drag(960, 700);
    assert_eq!(t.up(B::Right), Verdict::Swallow);
    assert_eq!(t.executed(), ["down"]);
    assert!(t.replays().is_empty());
    assert!(t.lines() > 0);
}

#[test]
fn overlong_stroke_cancels_without_replay_and_next_gesture_works() {
    let mut cfg = config(false);
    cfg.scopes[0].bindings.push(bind("close_tab", vec![RIGHT, trace(&DOWN_RIGHT)]));
    let mut t = T::with_samples(cfg, true);
    assert_eq!(t.down(B::Right), Verdict::Swallow);
    t.zigzag(101);
    assert_eq!(t.e.live_label.as_deref(), Some("画嗨了吧？这一笔不算～"));
    let deadline = t.e.label_deadline.unwrap();
    assert!(deadline > Instant::now() + Duration::from_secs(1));
    assert!(deadline <= Instant::now() + Duration::from_millis(1500));
    assert_eq!(t.log.borrow().finishes, 1, "取消应立即清除轨迹");
    assert_eq!(t.log.borrow().fades, 0);
    assert!(t.executed().is_empty());
    let labels = t.log.borrow().labels.clone();
    assert_eq!(t.up(B::Right), Verdict::Swallow);
    assert!(t.e.capture.is_none());
    assert!(t.e.target.is_none());
    assert!(t.executed().is_empty());
    assert!(t.replays().is_empty());
    assert!(t.log.borrow().samples.is_empty(), "取消的笔画不用于校准");
    assert_eq!(t.log.borrow().labels, labels, "释放不应显示未识别");
    assert_eq!(t.e.label_deadline, Some(deadline));
    assert_eq!(t.log.borrow().finishes, 1, "释放不再操作绘制面");
    assert_eq!(t.log.borrow().fades, 0);

    t.to(960, 540);
    assert_eq!(t.down(B::Right), Verdict::Swallow);
    t.drag(960, 700);
    t.drag(1160, 700);
    assert_eq!(t.up(B::Right), Verdict::Swallow);
    assert_eq!(t.executed(), ["close_tab"]);
    assert!(t.replays().is_empty());
}

#[test]
fn overlong_stroke_stops_tracking_drawing_and_live_labels_until_release() {
    let mut t = T::new(false);
    t.down(B::Right);
    t.zigzag(100);
    // 第 101 段是去噪会删掉的短收笔：必须按原始段数取消。
    t.to(t.pos.0, t.pos.1 + t.e.cfg.settings.min_stroke_px as i32);
    assert_eq!(t.e.tracker.segments().len(), 101);
    assert_eq!(t.e.tracker.strokes().len(), 100);
    let segments = t.e.tracker.segments().to_vec();
    let path = t.e.path.clone();
    let lines = t.lines();
    let labels = t.log.borrow().labels.clone();
    let finishes = t.log.borrow().finishes;
    for i in 0..10_000 {
        assert_eq!(t.to(960 + i % 200, 540 + i % 160), Verdict::Pass);
    }
    assert_eq!(t.wheel(1), Verdict::Swallow);
    assert_eq!(t.e.handle(InputEvent::KeyDown(0x41)), Verdict::Pass);
    assert_eq!(t.e.tracker.segments(), segments);
    assert_eq!(t.e.path, path);
    assert_eq!(t.lines(), lines);
    assert_eq!(t.log.borrow().labels, labels);
    assert_eq!(t.log.borrow().finishes, finishes);
    assert!(t.executed().is_empty());
    assert!(t.replays().is_empty());

    // 不必等松键，提示也应按时消失；后续移动不能把它重新画出来。
    let clears = t.log.borrow().label_clears;
    t.e.label_deadline = Some(Instant::now() - Duration::from_millis(1));
    t.e.toast_tick();
    assert_eq!(t.log.borrow().label_clears, clears + 1);
    assert!(t.e.live_label.is_none());
    assert!(t.e.label_deadline.is_none());
    t.to(1200, 800);
    assert_eq!(t.log.borrow().labels, labels);
    assert_eq!(t.up(B::Right), Verdict::Swallow);
    assert!(t.e.capture.is_none());
    assert!(t.replays().is_empty());
}

#[test]
fn overlong_stroke_label_follows_language_and_visibility() {
    for show in [true, false] {
        let mut cfg = config(false);
        cfg.settings.language = "en-US".into();
        cfg.settings.label.show = show;
        let mut t = T::with_config(cfg);
        t.down(B::Right);
        t.zigzag(101);
        assert_eq!(t.e.live_label.as_deref(), show.then_some("Nice doodle! Let’s skip this one."));
        assert_eq!(t.e.label_deadline.is_some(), show);
        assert_eq!(t.up(B::Right), Verdict::Swallow);
        assert!(t.e.capture.is_none());
        assert!(t.executed().is_empty());
        assert!(t.replays().is_empty());
    }
}

#[test]
fn stroke_at_segment_limit_still_matches_on_release() {
    let mut points = vec![0, 0];
    for i in 0..100 {
        points.extend([if i % 2 == 0 { 50 } else { 0 }, 0]);
    }
    let mut cfg = config(false);
    cfg.scopes[0].bindings.push(bind("hundred_segments", vec![RIGHT, trace(&points)]));
    let mut t = T::with_config(cfg);
    t.down(B::Right);
    t.zigzag(100);
    assert_eq!(t.e.tracker.segments().len(), 100);
    assert_eq!(t.e.capture.as_ref().unwrap().state, CaptureState::Active);
    assert_eq!(t.log.borrow().finishes, 0);
    assert_eq!(t.e.live_label.as_deref(), Some("hundred_segments"));
    assert_eq!(t.up(B::Right), Verdict::Swallow);
    assert_eq!(t.executed(), ["hundred_segments"]);
    assert!(t.replays().is_empty());
}

#[test]
fn overlong_edge_rubbing_ends_and_stays_disarmed_until_leaving() {
    let mut cfg = config(false);
    cfg.scopes[0].bindings.push(bind("rub", vec![Step::Edge { edge: TOP }, trace(&[0, 0, 50, 0, 0, 0])]));
    let mut t = T::with_config(cfg);
    t.dwell(960, 0);
    assert!(t.e.capture.is_some());
    t.zigzag(101);
    assert!(t.e.capture.is_none());
    assert!(t.e.edge_disarmed);
    assert!(t.e.edge_pending.is_none());
    assert_eq!(t.e.live_label.as_deref(), Some("画嗨了吧？这一笔不算～"));
    assert_eq!(t.log.borrow().finishes, 1);
    assert_eq!(t.lines(), 0);
    assert!(t.executed().is_empty());
    assert!(t.replays().is_empty());
    let segments = t.e.tracker.segments().to_vec();
    let path = t.e.path.clone();
    t.zigzag(4);
    t.dwell(960, 0);
    assert!(t.e.capture.is_none());
    assert_eq!(t.e.tracker.segments(), segments);
    assert_eq!(t.e.path, path);
    t.to(960, 100);
    assert!(!t.e.edge_disarmed);
    t.dwell(960, 0);
    assert!(t.e.capture.is_some());
    assert!(t.e.tracker.is_empty());
    t.zigzag(2);
    assert_eq!(t.e.live_label.as_deref(), Some("rub"));
}

#[test]
fn overlong_edge_stroke_with_button_waits_for_swallowed_release() {
    let mut t = T::new(true);
    t.dwell(960, 0);
    assert_eq!(t.down(B::Right), Verdict::Swallow);
    t.zigzag(101);
    assert!(t.e.capture.is_some());
    assert_eq!(t.e.live_label.as_deref(), Some("画嗨了吧？这一笔不算～"));
    let lines = t.lines();
    t.to(960, 500);
    assert_eq!(t.lines(), lines);
    assert_eq!(t.up(B::Right), Verdict::Swallow);
    assert!(t.e.capture.is_none());
    assert!(t.e.edge_disarmed);
    assert!(t.executed().is_empty());
    assert!(t.replays().is_empty());
    assert_eq!(t.log.borrow().finishes, 1);
    assert_eq!(t.log.borrow().fades, 0);
}

#[test]
fn plain_click_replays_menu_and_draws_nothing() {
    let mut t = T::new(false);
    t.down(B::Right);
    t.to(962, 541); // 手抖
    t.to(963, 543);
    t.up(B::Right);
    assert_eq!(t.replays(), [(B::Right, true)]);
    assert_eq!(t.lines(), 0, "点击时的手抖不应画出轨迹");
    assert!(t.executed().is_empty());
}

#[test]
fn click_with_slip_is_not_lost() {
    for btn in [B::Right, B::Middle] {
        let mut t = T::new(false);
        t.e.cfg.scopes[0].bindings.push(bind("mid", vec![Step::Press { key: PatternKey::Mouse(B::Middle) }, trace(&DOWN)]));
        t.e.apply_config(t.e.cfg.clone());
        t.down(btn);
        t.drag(985, 540); // 手滑 25px：不足一格
        t.up(btn);
        assert_eq!(t.replays(), [(btn, true)], "{btn:?}：没形成笔画的点击必须补发");
    }
}

#[test]
fn drawn_but_unrecognized_does_not_pop_menu() {
    let mut t = T::new(false);
    t.down(B::Right);
    t.drag(1100, 540); // 一段 →，没有对应手势
    t.up(B::Right);
    assert!(t.replays().is_empty(), "画了没识别不应补发右键（会弹出菜单）");
    assert!(t.executed().is_empty());
}

#[test]
fn slow_start_is_not_mistaken_for_drag() {
    let mut t = T::new(false);
    t.down(B::Right);
    t.drag(960, 555); // 先慢慢移动一点
    std::thread::sleep(Duration::from_millis(100)); // 超过拖拽超时
    t.drag(960, 700);
    t.up(B::Right);
    assert!(t.replays().is_empty(), "已经开始移动就不应回落为原生拖拽");
    assert_eq!(t.executed(), ["down"]);
}

#[test]
fn holding_still_falls_back_to_native_drag() {
    let mut t = T::new(false);
    t.down(B::Right);
    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(t.to(961, 540), Verdict::Pass);
    assert_eq!(t.replays(), [(B::Right, false)], "只补发按下，交还原生拖拽");
    t.drag(960, 700);
    assert_eq!(t.up(B::Right), Verdict::Pass);
    assert!(t.executed().is_empty());
    // 之后的新手势不受影响
    t.to(960, 540);
    t.down(B::Right);
    t.drag(960, 700);
    t.up(B::Right);
    assert_eq!(t.executed(), ["down"]);
}

#[test]
fn taskbar_and_tray_are_left_to_the_shell() {
    let mut t = T::new(false);
    for class in ["Shell_TrayWnd", "NotifyIconOverflowWindow", "TopLevelWindowForOverflowXamlIsland", "#32768"] {
        t.under(class, Some(r"C:\Windows\explorer.exe"));
        assert_eq!(t.down(B::Right), Verdict::Pass, "{class}");
        t.drag(970, 560);
        assert_eq!(t.up(B::Right), Verdict::Pass, "{class}");
    }
    assert_eq!(t.lines(), 0);
    assert!(t.replays().is_empty());
}

#[test]
fn own_settings_window_is_left_alone() {
    let mut t = T::new(false);
    t.under("Chrome_WidgetWin_1", Some(r"D:\x\mousetroke-settings.exe"));
    assert_eq!(t.down(B::Right), Verdict::Pass);
    assert_eq!(t.up(B::Right), Verdict::Pass);
}

#[test]
fn wheel_repeats_and_release_is_silent() {
    let mut t = T::new(false);
    t.down(B::Right);
    assert_eq!(t.wheel(1), Verdict::Swallow);
    assert_eq!(t.wheel(1), Verdict::Swallow);
    t.up(B::Right);
    assert_eq!(t.executed(), ["next_tab", "next_tab"]);
    assert!(t.replays().is_empty(), "连续滚轮后松开不应弹右键菜单");
}

#[test]
fn esc_cancels_capture() {
    let mut t = T::new(false);
    t.down(B::Right);
    t.drag(960, 700);
    assert_eq!(t.e.handle(InputEvent::KeyDown(0x1B)), Verdict::Swallow);
    t.up(B::Right);
    assert!(t.executed().is_empty());
    assert!(t.replays().is_empty());
}

#[test]
fn right_gesture_near_top_edge_still_works() {
    for combo in [false, true] {
        let mut t = T::new(combo);
        t.dwell(960, 0); // 进入上边缘捕获（存在 上边+滚轮 手势）
        t.to(960, 8);
        assert_eq!(t.down(B::Right), Verdict::Swallow, "combo={combo}");
        t.drag(960, 200);
        t.up(B::Right);
        assert_eq!(t.executed(), ["down"], "combo={combo}：贴着上沿划线应回落为普通手势");
        assert!(t.replays().is_empty(), "combo={combo}");
        assert!(t.lines() > 0, "combo={combo}");
    }
}

#[test]
fn right_click_near_top_edge_keeps_context_menu() {
    for combo in [false, true] {
        let mut t = T::new(combo);
        t.dwell(960, 0);
        t.to(960, 6);
        t.down(B::Right);
        t.up(B::Right);
        assert_eq!(t.replays(), [(B::Right, true)], "combo={combo}");
        assert!(t.executed().is_empty(), "combo={combo}");
    }
}

#[test]
fn top_edge_combo_and_wheel() {
    let mut t = T::new(true);
    t.dwell(960, 0);
    assert_eq!(t.wheel(-1), Verdict::Swallow);
    assert_eq!(t.wheel(-1), Verdict::Swallow);
    assert_eq!(t.executed(), ["vol_up", "vol_up"]);

    // 上边 + 右键 + ↓ + 左键 = 系统设置
    t.down(B::Right);
    t.drag(960, 200);
    assert_eq!(t.down(B::Left), Verdict::Swallow);
    assert_eq!(t.up(B::Left), Verdict::Swallow);
    assert_eq!(t.up(B::Right), Verdict::Swallow);
    assert_eq!(t.executed(), ["vol_up", "vol_up", "settings"]);
    assert!(t.replays().is_empty());

    // 贴着上沿按住右键滚轮 → 回落为 右键+滚轮
    drop(t); // 先释放语言锁：重新绑定不会提前 drop 旧值，同一线程再拿锁会死锁
    let mut t = T::new(true);
    t.dwell(960, 0);
    t.down(B::Right);
    t.wheel(1);
    t.up(B::Right);
    assert_eq!(t.executed(), ["next_tab"]);
    assert!(t.replays().is_empty());
}

#[test]
fn left_click_at_top_edge_passes() {
    let mut t = T::new(true);
    t.dwell(960, 0);
    assert_eq!(t.down(B::Left), Verdict::Pass, "点浏览器标签不应被吞");
    assert_eq!(t.up(B::Left), Verdict::Pass);
}

#[test]
fn corner_fires_once_until_leaving() {
    let mut t = T::new(false);
    t.dwell(0, 1079);
    assert_eq!(t.executed(), ["start"]);
    t.dwell(1, 1078);
    t.dwell(0, 1079);
    assert_eq!(t.executed(), ["start"], "停在角落不应反复触发");
    t.drag(300, 700);
    t.drag(0, 1079);
    t.dwell(0, 1079);
    assert_eq!(t.executed(), ["start", "start"]);
}

#[test]
fn stationary_corner_settle_respects_dwell_pause_and_rearm() {
    let mut t = T::new(false);
    t.to(0, 1079);
    t.e.settle_edges(0, 1079);
    assert!(t.executed().is_empty());
    t.e.edge_pending.as_mut().unwrap().1 = Instant::now() - EDGE_DWELL;
    t.e.settle_edges(1, 1079);
    assert!(t.executed().is_empty(), "过期坐标不能触发");
    t.e.set_paused(true);
    t.e.settle_edges(0, 1079);
    assert!(t.executed().is_empty());
    t.e.set_paused(false);
    t.e.settle_edges(0, 1079);
    assert_eq!(t.executed(), ["start"]);
    t.e.settle_edges(0, 1079);
    assert_eq!(t.executed(), ["start"]);
    t.to(300, 700);
    t.to(0, 1079);
    t.e.edge_pending.as_mut().unwrap().1 = Instant::now() - EDGE_DWELL;
    t.e.settle_edges(0, 1079);
    assert_eq!(t.executed(), ["start", "start"]);
}

#[test]
fn mac_default_right_edge_rubbing_executes_without_a_button_release() {
    let mut cfg = Config::builtin_for(mousetroke_core::caps::Platform::Macos);
    let binding = cfg.scopes[0].bindings.iter_mut().find(|b| b.name_key.as_deref() == Some("builtin.gesture.task_manager")).unwrap();
    assert!(matches!(&binding.action, Action::Preset { id, .. } if id == "system.task_manager"));
    binding.action = Action::Shell { script: "activity_monitor".into() };
    let mut t = T::with_config(cfg);
    t.dwell(1919, 600);
    assert!(t.e.capture.is_some());
    for y in [400, 600, 400, 600] { t.drag(1919, y); }
    t.e.settle_edges(1919, 600);
    assert_eq!(t.executed(), ["activity_monitor"], "segments={:?}, strokes={:?}, label={:?}, capture={:?}", t.e.tracker.segments(), t.e.tracker.strokes(), t.e.live_label, t.e.capture.as_ref().map(|c| (c.trigger, c.state)));
    assert!(t.e.capture.is_none());
    for y in [400, 600, 400, 600] { t.drag(1919, y); }
    t.e.settle_edges(1919, 600);
    assert_eq!(t.executed(), ["activity_monitor"], "segments={:?}, strokes={:?}, label={:?}, capture={:?}", t.e.tracker.segments(), t.e.tracker.strokes(), t.e.live_label, t.e.capture.as_ref().map(|c| (c.trigger, c.state)));
    t.to(1500, 600);
    t.dwell(1919, 600);
    for y in [400, 600, 400, 600] { t.drag(1919, y); }
    t.e.settle_edges(1919, 600);
    assert_eq!(t.executed(), ["activity_monitor", "activity_monitor"]);
}

#[test]
fn lost_release_does_not_wedge_the_engine() {
    let mut t = T::new(false);
    t.down(B::Right);
    t.drag(960, 700);
    // 抬起事件丢失；用户再次按下右键画手势
    t.to(960, 300);
    assert_eq!(t.down(B::Right), Verdict::Swallow);
    t.drag(960, 500);
    t.up(B::Right);
    assert_eq!(t.executed(), ["down"]);
}

#[test]
fn unrecognized_gesture_gives_feedback() {
    let mut t = T::new(false);
    t.down(B::Right);
    t.drag(1160, 540);
    t.drag(1160, 740);
    t.drag(1360, 740);
    t.up(B::Right);
    assert!(t.executed().is_empty());
    assert!(t.replays().is_empty());
    let labels = t.log.borrow().labels.clone();
    assert!(labels.contains(&"→ ↓".to_string()), "画的过程中显示识别出的方向: {labels:?}");
    assert_eq!(labels.last().map(String::as_str), Some("未识别  → ↓ →"));
}

#[test]
fn matched_gesture_shows_its_name() {
    let mut t = T::new(false);
    t.down(B::Right);
    t.drag(960, 700);
    t.up(B::Right);
    assert_eq!(t.log.borrow().labels.last().map(String::as_str), Some("down"));
}


#[test]
fn unrecognized_label_follows_language() {
    let mut t = T::new(false);
    t.set_lang(mousetroke_core::i18n::Lang::EnUs);
    t.down(B::Right);
    t.drag(1160, 540);
    t.drag(1160, 740);
    t.drag(1360, 740);
    t.up(B::Right);
    assert_eq!(t.log.borrow().labels.last().map(String::as_str), Some("Unrecognized  → ↓ →"));
    mousetroke_core::i18n::set_lang(mousetroke_core::i18n::Lang::ZhCn);
}

#[test]
fn builtin_name_key_label_is_translated() {
    let mut t = T::new(false);
    // 内置名（有 name_key 且未改名）按当前语言显示；zh 下 name 存的是英文也不影响
    let mut b = bind("Close", vec![RIGHT, trace(&DOWN_RIGHT)]);
    b.name_key = Some("builtin.gesture.close_tab".into());
    t.e.cfg.scopes[0].bindings.push(b);
    t.e.apply_config(t.e.cfg.clone());
    t.set_lang(mousetroke_core::i18n::Lang::ZhCn);
    t.down(B::Right);
    t.drag(960, 700);
    t.drag(1160, 700);
    t.up(B::Right);
    assert_eq!(t.log.borrow().labels.last().map(String::as_str), Some("关闭标签页"));
    t.set_lang(mousetroke_core::i18n::Lang::EnUs);
    t.to(960, 540); // 回到起点（pos 跨手势保留）
    t.down(B::Right);
    t.drag(960, 700);
    t.drag(1160, 700);
    t.up(B::Right);
    assert_eq!(t.log.borrow().labels.last().map(String::as_str), Some("Close tab"));
    mousetroke_core::i18n::set_lang(mousetroke_core::i18n::Lang::ZhCn);
}

#[test]
fn disabled_bindings_are_ignored_and_release_their_button() {
    let mut t = T::new(false);
    let mut cfg = t.e.cfg.clone();
    cfg.scopes[0].bindings.push(Binding { disabled: true, ..bind("mid", vec![Step::Press { key: PatternKey::Mouse(B::Middle) }, trace(&DOWN)]) });
    cfg.scopes[0].bindings[0].disabled = true; // 「down」停用
    t.e.apply_config(cfg);
    // 中键只有一条停用的手势 → 完全不拦截
    t.pos = (960, 540);
    assert_eq!(t.down(B::Middle), Verdict::Pass);
    assert_eq!(t.up(B::Middle), Verdict::Pass);
    // 右键还有别的启用手势 → 照常捕获，但停用的「down」不会执行
    t.down(B::Right);
    t.drag(960, 700);
    t.up(B::Right);
    assert!(t.executed().is_empty());
}

#[test]
fn clicks_just_below_top_edge_do_not_fire_edge_combos() {
    // 旧格式默认：上边+右键=播放/暂停，上边+中键=静音，上边+滚轮=音量
    let mut t = T::new(false);
    let mut cfg = t.e.cfg.clone();
    cfg.scopes[0].bindings.push(bind("play", vec![Step::Edge { edge: TOP }, RIGHT]));
    cfg.scopes[0].bindings.push(bind("mute", vec![Step::Edge { edge: TOP }, press(PatternKey::Mouse(B::Middle))]));
    t.e.apply_config(cfg);

    // 碰到上沿后回到标签栏（y=12）右键标签 → 普通右键菜单
    t.dwell(960, 0);
    t.to(960, 12);
    assert_eq!(t.down(B::Right), Verdict::Swallow);
    assert_eq!(t.up(B::Right), Verdict::Swallow);
    assert_eq!(t.replays(), [(B::Right, true)]);
    // 中键关标签 → 原样放行
    t.to(900, 60); // 先离开边缘（重新武装）
    t.dwell(900, 0);
    assert!(t.e.capture.is_some(), "应已进入上边缘捕获");
    t.to(900, 12);
    assert_eq!(t.down(B::Middle), Verdict::Pass);
    assert_eq!(t.up(B::Middle), Verdict::Pass);
    // 标签栏上滚轮 → 普通滚动
    t.to(800, 60); // 先离开边缘（重新武装）
    t.dwell(800, 0);
    t.to(800, 12);
    assert_eq!(t.wheel(-1), Verdict::Pass);
    assert!(t.executed().is_empty(), "{:?}", t.executed());

    // 真的贴在上沿：照常触发
    t.to(700, 60); // 先离开边缘（重新武装）
    t.dwell(700, 0);
    t.down(B::Right);
    t.up(B::Right);
    t.to(600, 60); // 先离开边缘（重新武装）
    t.dwell(600, 0);
    t.wheel(-1);
    assert_eq!(t.executed(), ["play", "vol_up"]);
}

#[test]
fn slipping_click_draws_no_trail() {
    let mut t = T::new(false);
    t.down(B::Right);
    t.drag(972, 540); // 手滑 12px
    t.up(B::Right);
    assert_eq!(t.lines(), 0, "点击时手滑不应闪出轨迹");
    assert_eq!(t.replays(), [(B::Right, true)]);
}

/// 模拟 uTools 这类工具：截下我们补发的点击，去掉标记后重发（系统仍标"注入"）。
fn echo_back(t: &mut T, injected: bool) -> usize {
    let mut rounds = 0;
    for _ in 0..50 {
        let pending: Vec<(B, bool)> = std::mem::take(&mut t.log.borrow_mut().replays);
        if pending.is_empty() {
            break;
        }
        rounds += 1;
        for (btn, click) in pending {
            let (x, y) = t.pos;
            t.e.handle_input(InputEvent::MouseDown { btn, x, y }, injected);
            if click {
                t.e.handle_input(InputEvent::MouseUp { btn, x, y }, injected);
            }
        }
    }
    rounds
}

#[test]
fn replayed_click_echoed_by_another_tool_does_not_loop() {
    let mut t = T::new(false);
    t.down(B::Right);
    t.up(B::Right);
    assert_eq!(t.replays(), [(B::Right, true)]);
    assert_eq!(echo_back(&mut t, true), 1, "回声应被识别并放行，不再补发");
    // 之后的真实手势照常
    t.down(B::Right);
    t.drag(960, 700);
    t.up(B::Right);
    assert_eq!(t.executed(), ["down"]);
}

#[test]
fn replay_loop_is_broken_even_without_injected_flag() {
    let mut t = T::new(false);
    t.down(B::Right);
    t.up(B::Right);
    // 最坏情况：重发的事件连"注入"标志都没有 → 靠熔断打断
    let rounds = echo_back(&mut t, false);
    assert!(rounds <= 7, "补发循环应在几次内被熔断，实际 {rounds} 轮");
}

#[test]
fn app_with_gestures_disabled_is_left_alone() {
    let mut t = T::new(false);
    let mut cfg = t.e.cfg.clone();
    cfg.scopes.push(Scope {
        id: "game".into(),
        name: "game".into(),
        name_key: None,
        kind: ScopeKind::App,
        app: Some(mousetroke_core::schema::AppMatch { exe: "game.exe".into(), win_class: None, win_title: None, regex: false }),
        special: None,
        inherit_global: true,
        disabled: true,
        bindings: vec![],
    });
    t.e.apply_config(cfg);

    t.under("UnityWndClass", Some(r"D:\Games\game.exe"));
    assert_eq!(t.down(B::Right), Verdict::Pass, "禁用的程序里右键原样放行");
    t.drag(960, 700);
    assert_eq!(t.up(B::Right), Verdict::Pass);
    t.dwell(0, 1079); // 左下角
    assert!(t.executed().is_empty(), "禁用的程序里不画轨迹、不触发边角");
    assert_eq!(t.lines(), 0);

    // 其他程序照常
    t.under("Chrome_WidgetWin_1", Some(r"C:\app\chrome.exe"));
    t.to(960, 540);
    t.down(B::Right);
    t.drag(960, 700);
    t.up(B::Right);
    assert_eq!(t.executed(), ["down"]);
}

/// 内置默认手势在真实识别流程里能被画出来（首次运行的用户拿到的就是这一套）。
#[test]
fn builtin_defaults_are_drawable() {
    use mousetroke_core::i18n::{keys, t as tr};
    let mut cfg = Config::builtin();
    cfg.settings.language = "zh-CN".into();
    cfg.settings.drag_timeout_ms = 60;
    let mut t = T::with_config(cfg);
    // (轨迹折点, 期望的手势)
    let cases: [(&[(i32, i32)], &str); 6] = [
        (&[(760, 540)], keys::BUILTIN_GESTURE_BACK),
        (&[(960, 740), (1160, 740)], keys::BUILTIN_GESTURE_CLOSE_TAB),
        (&[(1160, 540), (1160, 740)], keys::BUILTIN_GESTURE_NEW_TAB),
        (&[(800, 700)], keys::BUILTIN_GESTURE_MINIMIZE),
        (&[(960, 340)], keys::BUILTIN_GESTURE_COPY),
        (&[(760, 540), (760, 340), (960, 340)], keys::BUILTIN_GESTURE_REFRESH),
    ];
    for (i, (path, want)) in cases.iter().enumerate() {
        t.pos = (960, 540);
        t.to(960, 540);
        t.down(B::Right);
        for &(x, y) in path.iter() {
            t.drag(x, y);
        }
        t.up(B::Right);
        // 假宿主只记录 shell 动作，这里看命中后显示的手势名
        assert_eq!(t.log.borrow().labels.last().map(String::as_str), Some(tr(want)), "第 {i} 个手势");
        assert!(t.replays().is_empty(), "第 {i} 个手势被当成了普通点击");
    }
}
