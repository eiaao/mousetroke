//! 手势引擎状态机：输入事件 → 捕获会话 → 模式构建 → 匹配 → 执行。
//!
//! 支持两类触发源：
//! - **按键触发**：按住鼠标键（右/中/X1/X2）划线，释放时匹配
//! - **边缘触发**：鼠标撞向屏幕边/角（触发角/摩擦边）。纯边角绑定到达即执行；
//!   否则进入边缘捕获，等待后续按键/滚轮/沿边摩擦，离开边缘即取消
//!
//! 交互原则：
//! - 不属于任何手势前缀的输入一律放行（例如在屏幕上沿点浏览器标签页不会被吞）
//! - 滚轮步骤可连续触发（按住右键连续滚动切标签、上边缘连续滚动调音量）
//! - 执行过动作的捕获结束时不补发点击（不会弹出多余的右键菜单）
//! - 边角触发一次后需离开边缘才会再次触发（不会停在角落反复弹开始菜单）
//! - 按住触发键超时未移动 → 补发按下并放行，保留原生拖拽（一旦开始移动就不再超时）
//! - 没形成笔画（不足最小识别距离）都算点击，照常补发；不足最小识别距离一半的手滑不画轨迹。
//!   画出了笔画但没识别 → 不补发（不会弹出意料之外的右键菜单），屏幕提示「未识别」
//! - 在屏幕边缘附近按下触发键，既可匹配边缘组合手势，也可回落为普通按键手势
//! - 任务栏/托盘/系统菜单上不做手势
//! - 捕获中按 Esc 取消手势
//!
//! 匹配逻辑本身在 [`mousetroke_core::matcher`]（纯逻辑、可测）。

use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};

use mousetroke_core::edge;
use mousetroke_core::matcher::{self, Trigger};
use mousetroke_core::recognizer::{self, PathTracker, Segment};
use mousetroke_core::schema::{Action, Binding, Config, MouseButton, PatternKey, Scope, Step};
use mousetroke_core::shape;
use mousetroke_core::vars::Vars;

use crate::{Canvas, Host, Target};

/// 暂停状态的全局镜像（Lua 等跨线程读取）。
pub static PAUSED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Toast 队列（Lua show_toast → 引擎线程显示）。
static TOASTS: std::sync::Mutex<VecDeque<(String, Duration)>> = std::sync::Mutex::new(VecDeque::new());

/// 投递一条屏幕提示（任意线程）。
pub fn push_toast(msg: String, duration_ms: u64) {
    if let Ok(mut q) = TOASTS.lock() {
        q.push_back((msg, Duration::from_millis(duration_ms)));
    }
}

/// 边缘驻留时间：进入热区停留这么久才触发（防甩过角落误触）。
const EDGE_DWELL: Duration = Duration::from_millis(30);
/// 边缘手势的第一步是按键/滚轮时，光标须贴在边上（px）。往边上一推光标就停在第 0 行；
/// 不贴边的点击照常交给程序——最大化窗口的标题栏、浏览器标签栏都紧挨着屏幕上沿，
/// 宽松判定会让右键标签误触「上边+右键」、中键关标签误触「上边+中键」。
const EDGE_PRESS_PX: i32 = 1;
/// 边缘捕获中、尚无附加按键时，离开边缘超过此距离(px) → 取消。
const EDGE_LEAVE_PX: i32 = 32;
/// 手势结束后空闲多久归还工作集。
const TRIM_AFTER: Duration = Duration::from_secs(3);
/// 补发点击的回声窗口：其他鼠标工具（uTools 长按右键等）会截下我们补发的点击再重发，
/// 重发后我们的标记丢失、只剩系统的"注入"标志。窗口内匹配的注入事件视为回声，直接放行。
const ECHO_WINDOW: Duration = Duration::from_millis(500);
/// 命中标签停留时间。
const HIT_LABEL: Duration = Duration::from_millis(600);
/// 超长取消提示停留时间，松键不打断。
const TOO_LONG_LABEL: Duration = Duration::from_millis(1500);
/// 点击容差下限(px)：位移不超过 max(此值, 最小识别距离/3) 不算开始移动（仍可回落为原生拖拽）；
/// 轨迹要到最小识别距离的一半才开始画，点击时手滑不会闪出一小段线。
const CLICK_SLOP_MIN: i32 = 6;
/// 原始路径复用固定容量；满时原地隔点抽稀，钩子内不扩容。
const SHAPE_PATH_CAP: usize = 2048;
/// 内置手势最长仅 4 段；放宽到 100 段容纳复杂笔画，仍能及时截断无休止的涂画。
const MAX_STROKE_SEGMENTS: usize = 100;

/// 钩子回调送入的输入事件（坐标为屏幕坐标）。
#[derive(Debug, Clone, Copy)]
pub enum InputEvent {
    MouseDown { btn: MouseButton, x: i32, y: i32 },
    MouseUp { btn: MouseButton, x: i32, y: i32 },
    MouseMove { x: i32, y: i32 },
    /// 滚轮（旧格式约定：正=向下，单位格）。
    Wheel(i32),
    KeyDown(u16),
}

/// 事件处理裁决：吞下或放行。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Swallow,
    Pass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CaptureState {
    /// 正在识别。
    Active,
    /// 拖拽超时，down 已补发，之后的事件全部放行。
    Passthrough,
    /// 已执行（识别即执行）或已取消（Esc），等待按键释放，释放被吞且不补发。
    Done,
    /// 笔画过长，等待按键释放；停止追踪与绘制，释放被吞且不补发。
    Cancelled,
}

struct Capture {
    trigger: Trigger,
    /// 触发之外的附加按键（按时间序）。
    presses: Vec<PatternKey>,
    /// 轨迹开始时已有的附加键数（轨迹在步骤序列中的位置）。
    trace_at: Option<usize>,
    /// 捕获中被吞下、尚未释放的附加鼠标键（其抬起也要吞；捕获前按下的键则放行）。
    held: Vec<MouseButton>,
    /// 触发键按下时刻（拖拽超时从这里算）。
    started: Instant,
    /// 触发键按下位置（点击容差从这里算）。
    origin: (i32, i32),
    /// 已移动超过点击容差。
    moved: bool,
    /// 已移动到足以显示轨迹（最小识别距离的一半）。
    show_trail: bool,
    state: CaptureState,
    /// 捕获期间执行过动作（连续滚轮等）→ 结束时不补发点击。
    consumed: bool,
    /// 边缘捕获中第一个附加键是触发键 → 也可回落为该键的普通手势。
    as_button: Option<MouseButton>,
}

type Steps = Vec<mousetroke_core::schema::Step>;

impl Capture {
    /// 当前输入可构成的候选步骤序列（优先级从高到低）：
    /// 边缘捕获在 [`Self::as_button`] 存在时，额外给出"去掉边缘、以该键触发"的序列。
    fn candidates(&self, extra: Option<PatternKey>, with_trace_now: bool) -> Vec<Steps> {
        let mut presses = self.presses.clone();
        presses.extend(extra);
        let trace_at = self.trace_at.or(with_trace_now.then_some(self.presses.len()));
        let mut out = vec![matcher::build_steps(self.trigger, &presses, trace_at)];
        if let (Trigger::Edge(_), Some(b)) = (self.trigger, self.as_button) {
            if trace_at.is_none_or(|t| t >= 1) {
                out.push(matcher::build_steps(Trigger::Button(b), &presses[1..], trace_at.map(|t| t - 1)));
            }
        }
        out
    }

    /// 作为普通按键手势时的触发键。
    fn button(&self) -> Option<MouseButton> {
        match self.trigger {
            Trigger::Button(b) => Some(b),
            Trigger::Edge(_) => self.as_button,
        }
    }

    /// 只是按了一下触发键：没形成笔画、没附加步骤。
    /// 不要求完全没移动——边移动边点、手一滑都很常见，点击不能因此丢掉。
    fn is_plain_click(&self) -> bool {
        self.trace_at.is_none()
            && match self.trigger {
                Trigger::Button(_) => self.presses.is_empty(),
                Trigger::Edge(_) => self.as_button.is_some() && self.presses.len() == 1,
            }
    }
}

/// 热加载结果（托盘据此提示/重注册热键）。
pub enum Reload {
    Applied { scopes: usize, bindings: usize, hotkey_changed: Option<String> },
    Failed(String),
}

/// 配置文件监视：mtime + 长度作为指纹。
struct ConfigWatch {
    path: PathBuf,
    stamp: Option<(SystemTime, u64)>,
    /// 捕获进行中读到的新配置，等手势结束再换。
    pending: Option<Config>,
}

impl ConfigWatch {
    fn new(path: PathBuf) -> Self {
        let stamp = Self::stamp_of(&path);
        Self { path, stamp, pending: None }
    }

    fn stamp_of(path: &std::path::Path) -> Option<(SystemTime, u64)> {
        let m = std::fs::metadata(path).ok()?;
        Some((m.modified().ok()?, m.len()))
    }
}

/// 手势引擎。仅在引擎线程（安装钩子的线程）上使用。
pub struct Engine {
    cfg: Config,
    overlay: Box<dyn Canvas>,
    host: Box<dyn Host>,
    tracker: PathTracker,
    path: Vec<(f64, f64)>,
    /// 启动时读取一次；测试直接注入，不修改进程环境。
    shape_samples: bool,
    capture: Option<Capture>,
    target: Option<Target>,
    trigger_buttons: Vec<MouseButton>,
    /// 配置中出现的边缘/角落值。
    edge_triggers: Vec<u8>,
    paused: bool,
    live_label: Option<String>,
    /// 当前/最近一次手势的变量上下文。
    vars: Vars,
    last_pos: (i32, i32),
    label_deadline: Option<Instant>,
    /// 本次手势的标签位置（手势开始时按所在显示器确定，之后不动）。
    label_at: (i32, i32),
    /// 最近一次手势开始的时刻；None = 已归还工作集。
    active_at: Option<Instant>,
    /// 等待回来的补发事件：(键, 是否按下, 截止时刻)。
    echoes: Vec<(MouseButton, bool, Instant)>,
    /// 最近几次补发单击的时刻（回声识别失效时的熔断）。
    recent_replays: VecDeque<Instant>,
    edge_pending: Option<(u8, Instant)>,
    /// 边角触发/取消后需先离开所有边缘才重新武装。
    edge_disarmed: bool,
    watch: Option<ConfigWatch>,
}

impl Engine {
    /// 在引擎线程初始化；创建绘制面与固定路径缓冲后才能接收输入。
    pub fn new(cfg: Config, cfg_path: Option<PathBuf>, host: Box<dyn Host>) -> Option<Self> {
        let overlay = host.new_canvas(&cfg.settings.trail.color_normal, cfg.settings.trail.width)?;
        let shape_samples = std::env::var_os("MOUSETROKE_SHAPE_SAMPLES").as_deref() == Some(std::ffi::OsStr::new("1"));
        if shape_samples {
            host.init_shape_samples();
        }
        Some(Self::with_parts(cfg, cfg_path, overlay, host, shape_samples))
    }

    fn with_parts(cfg: Config, cfg_path: Option<PathBuf>, overlay: Box<dyn Canvas>, host: Box<dyn Host>, shape_samples: bool) -> Self {
        let mut engine = Self {
            host,
            tracker: PathTracker::new(cfg.settings.min_stroke_px as f64),
            path: Vec::with_capacity(SHAPE_PATH_CAP),
            shape_samples,
            overlay,
            cfg: Config::default(),
            capture: None,
            target: None,
            trigger_buttons: Vec::new(),
            edge_triggers: Vec::new(),
            paused: false,
            live_label: None,
            vars: Vars::new(),
            last_pos: (0, 0),
            label_deadline: None,
            label_at: (0, 0),
            active_at: None,
            echoes: Vec::new(),
            recent_replays: VecDeque::new(),
            edge_pending: None,
            edge_disarmed: false,
            watch: cfg_path.map(ConfigWatch::new),
        };
        engine.apply_config(cfg);
        engine
    }

    /// 替换配置并重算派生状态。须在无捕获时调用。
    fn apply_config(&mut self, cfg: Config) {
        self.host.set_automatic_updates(cfg.settings.update_check);
        // 语言设置也随配置走：启动与热加载都在这里切换（托盘/屏幕提示 ≤1s 跟着换）
        let lang = mousetroke_core::i18n::resolve(&cfg.settings.language, &self.host.system_langs());
        let lang_changed = lang != mousetroke_core::i18n::lang();
        mousetroke_core::i18n::set_lang(lang);
        if lang_changed {
            self.host.refresh_texts();
        }
        self.trigger_buttons = matcher::trigger_buttons(&cfg);
        self.edge_triggers = matcher::edge_triggers(&cfg);
        self.tracker = PathTracker::new(cfg.settings.min_stroke_px as f64);
        self.overlay.set_color(&cfg.settings.trail.color_normal);
        self.overlay.set_width(cfg.settings.trail.width);
        self.host.config_applied(&cfg);
        self.cfg = cfg;
    }

    /// 配置热加载轮询（托盘 WM_TIMER 驱动，约每秒一次）。只在文件变化时读盘。
    pub fn poll_config(&mut self) -> Option<Reload> {
        let watch = self.watch.as_mut()?;
        let stamp = ConfigWatch::stamp_of(&watch.path);
        if stamp.is_some() && stamp != watch.stamp {
            watch.stamp = stamp;
            match Config::load(&watch.path) {
                Ok(cfg) => watch.pending = Some(cfg),
                Err(e) => return Some(Reload::Failed(e)),
            }
        }
        if self.capture.is_some() {
            return None; // 捕获中不换配置
        }
        let cfg = self.watch.as_mut()?.pending.take()?;
        let hotkey_changed =
            (cfg.settings.pause_hotkey != self.cfg.settings.pause_hotkey).then(|| cfg.settings.pause_hotkey.clone());
        let (scopes, bindings) = (cfg.scopes.len(), cfg.all_bindings().count());
        self.apply_config(cfg);
        Some(Reload::Applied { scopes, bindings, hotkey_changed })
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// 平台在光标停稳后补一次驻留/摩擦识别；不追加轨迹点，不影响按键捕获。
    pub fn settle_edges(&mut self, x: i32, y: i32) {
        if self.paused || self.last_pos != (x, y) { return; }
        match &self.capture {
            None => self.detect_edge(x, y),
            Some(cap) if cap.state == CaptureState::Active && matches!(cap.trigger, Trigger::Edge(_))
                && cap.held.is_empty() && cap.presses.is_empty() && cap.trace_at.is_some() => {
                // 分段数未变时末段仍可能从去噪短钩长成有效笔画；停稳后重新识别完整摩擦。
                self.after_input();
            }
            _ => {}
        }
    }

    /// 淡出动画一帧（overlay 窗口 WM_TIMER 驱动）。
    pub fn overlay_tick(&mut self) {
        self.overlay.tick_fade();
    }

    pub fn set_paused(&mut self, on: bool) {
        if on != self.paused {
            self.paused = on;
            PAUSED.store(on, std::sync::atomic::Ordering::Relaxed);
        }
        if self.paused {
            self.abort_capture();
        }
    }

    pub fn toggle_paused(&mut self) {
        self.set_paused(!self.paused);
    }

    /// 显示器布局变化（WM_DISPLAYCHANGE）：重建覆盖层以匹配新的虚拟屏。
    pub fn on_display_change(&mut self) {
        self.abort_capture();
        let s = &self.cfg.settings.trail;
        crate::log!("显示器布局变化，重建覆盖层");
        if let Some(o) = self.host.new_canvas(&s.color_normal, s.width) {
            self.overlay = o;
        }
    }

    /// Toast/标签到期轮询（托盘 WM_TIMER 驱动，250ms）。
    pub fn toast_tick(&mut self) {
        let now = Instant::now();
        if self.capture.is_some() {
            self.active_at = Some(now);
        } else if self.active_at.is_some_and(|t| now.duration_since(t) > TRIM_AFTER) && self.label_deadline.is_none() {
            self.active_at = None;
            crate::log!("空闲，归还工作集");
            self.host.trim_memory();
        }
        let next = TOASTS.lock().ok().and_then(|mut q| q.pop_front());
        if let Some((msg, dur)) = next {
            self.label_deadline = Some(now + dur);
            let (x, y) = self.host.cursor_pos();
            let (lx, ly) = self.host.label_anchor(x, y);
            self.overlay.draw_label(&msg, lx, ly, &self.cfg.settings.label.color_normal);
        } else if self.label_deadline.is_some_and(|d| now >= d) {
            self.label_deadline = None;
            if self.capture.as_ref().is_none_or(|c| c.state == CaptureState::Cancelled) || self.live_label.is_none() {
                self.overlay.clear_label();
                self.live_label = None;
            }
        }
    }

    /// 钩子回调入口。返回 [`Verdict::Swallow`] 表示拦截该事件。
    pub fn handle(&mut self, ev: InputEvent) -> Verdict {
        self.handle_input(ev, false)
    }

    /// 同 [`Self::handle`]，`injected` = 系统标记的注入事件（非本程序标记的那种）。
    pub fn handle_input(&mut self, ev: InputEvent, injected: bool) -> Verdict {
        if injected {
            let (btn, down) = match ev {
                InputEvent::MouseDown { btn, .. } => (Some(btn), true),
                InputEvent::MouseUp { btn, .. } => (Some(btn), false),
                _ => (None, false),
            };
            if let Some(btn) = btn {
                let now = Instant::now();
                self.echoes.retain(|e| e.2 > now);
                if let Some(i) = self.echoes.iter().position(|e| e.0 == btn && e.1 == down) {
                    self.echoes.remove(i);
                    return Verdict::Pass; // 我们补发的点击被别的工具重发回来
                }
            }
        }
        if self.paused {
            return Verdict::Pass;
        }
        match ev {
            InputEvent::MouseDown { btn, x, y } => self.on_mouse_down(btn, x, y),
            InputEvent::MouseUp { btn, .. } => self.on_mouse_up(btn),
            InputEvent::MouseMove { x, y } => self.on_mouse_move(x, y),
            InputEvent::Wheel(delta) => self.on_wheel(delta),
            InputEvent::KeyDown(vk) => self.on_key_down(vk),
        }
    }

    // ------------------------------------------------------------ 输入事件

    /// 无捕获时按下鼠标键：是触发键就开始按键手势。
    fn try_start_button(&mut self, btn: MouseButton, x: i32, y: i32) -> Verdict {
        if !self.trigger_buttons.contains(&btn) || self.host.foreground_fullscreen() {
            return Verdict::Pass;
        }
        let target = self.host.target_at(x, y);
        // 设置程序的录制画布要用右键画手势；任务栏/托盘的右键菜单交给系统
        if self.host.ignore_target(&target) {
            return Verdict::Pass;
        }
        // 用户在这个程序里禁用了手势：完全不拦截
        if matcher::gestures_off(&self.cfg, target.is_desktop, target.exe.as_deref()) {
            return Verdict::Pass;
        }
        crate::log!("{btn:?} 按下 ({x},{y})，开始捕获：{}", target.exe.as_deref().map(matcher::file_name).unwrap_or(&target.class));
        self.start_capture_at(Trigger::Button(btn), x, y, target);
        Verdict::Swallow
    }

    fn on_mouse_down(&mut self, btn: MouseButton, x: i32, y: i32) -> Verdict {
        let Some(cap) = &mut self.capture else {
            return self.try_start_button(btn, x, y);
        };
        if cap.trigger == Trigger::Button(btn) {
            // 触发键还没抬起就又按下 → 上次的抬起丢了（安全桌面、其他钩子吞掉等）。
            // 丢弃卡住的会话重新开始，而不是把它当成附加键让手势一直失灵
            crate::log!("{btn:?} 未抬起又按下（上次抬起丢失），重置捕获");
            self.abort_capture();
            return self.try_start_button(btn, x, y);
        }
        match (cap.state, cap.trigger) {
            (CaptureState::Passthrough, _) => Verdict::Pass,
            (CaptureState::Done | CaptureState::Cancelled, Trigger::Button(_)) => {
                cap.held.push(btn);
                Verdict::Swallow
            }
            (CaptureState::Done | CaptureState::Cancelled, Trigger::Edge(_)) => {
                if cap.held.is_empty() {
                    self.end_edge_capture();
                    self.try_start_button(btn, x, y)
                } else {
                    cap.held.push(btn);
                    Verdict::Swallow
                }
            }
            (CaptureState::Active, Trigger::Button(_)) => {
                cap.presses.push(PatternKey::Mouse(btn));
                cap.held.push(btn);
                self.after_input();
                Verdict::Swallow
            }
            (CaptureState::Active, Trigger::Edge(_)) => {
                // 边缘捕获只吞"能延续某个边缘手势"的按键；否则结束边缘捕获，
                // 按普通方式处理这次按下（贴着屏幕上沿右键划线照样是普通手势）
                let first = cap.presses.is_empty() && cap.trace_at.is_none();
                let steps = cap.candidates(Some(PatternKey::Mouse(btn)), false);
                if (first && !self.pressed_at_edge(x, y)) || !self.feasible(&steps[..1]) {
                    self.end_edge_capture();
                    return self.try_start_button(btn, x, y);
                }
                let as_button = (first && self.trigger_buttons.contains(&btn)).then_some(btn);
                let cap = self.capture.as_mut().unwrap();
                if first {
                    // 从这次按下起算轨迹、点击容差和拖拽超时
                    cap.origin = (x, y);
                    cap.moved = false;
                    cap.show_trail = false;
                    cap.started = Instant::now();
                    cap.as_button = as_button;
                    self.start_path(x, y);
                    self.overlay.start_session(x, y);
                }
                let cap = self.capture.as_mut().unwrap();
                cap.presses.push(PatternKey::Mouse(btn));
                cap.held.push(btn);
                self.after_input();
                Verdict::Swallow
            }
        }
    }

    fn on_mouse_up(&mut self, btn: MouseButton) -> Verdict {
        let Some(cap) = &mut self.capture else {
            return Verdict::Pass;
        };
        if cap.state == CaptureState::Passthrough {
            if cap.button() == Some(btn) {
                self.capture = None;
                self.overlay.finish();
            }
            return Verdict::Pass;
        }
        match cap.trigger {
            Trigger::Button(b) => {
                if b != btn {
                    // 附加键释放：按下时已记录；捕获前就按着的键照常放行
                    return match cap.held.iter().position(|h| *h == btn) {
                        Some(i) => {
                            cap.held.remove(i);
                            Verdict::Swallow
                        }
                        None => Verdict::Pass,
                    };
                }
                let cap = self.capture.take().unwrap();
                if cap.state == CaptureState::Active {
                    self.finish_gesture(cap);
                } else if cap.state == CaptureState::Done {
                    self.log_shape_sample(&cap);
                    self.end_visuals(true);
                }
                Verdict::Swallow
            }
            Trigger::Edge(_) => {
                let Some(i) = cap.held.iter().position(|h| *h == btn) else {
                    return Verdict::Pass; // 不是我们吞下的键
                };
                cap.held.remove(i);
                if cap.held.is_empty() {
                    let cap = self.capture.take().unwrap();
                    if cap.state == CaptureState::Active {
                        self.finish_gesture(cap);
                    } else if cap.state == CaptureState::Done {
                        self.log_shape_sample(&cap);
                        self.end_visuals(true);
                    }
                    self.edge_disarmed = true;
                }
                Verdict::Swallow
            }
        }
    }

    fn on_mouse_move(&mut self, x: i32, y: i32) -> Verdict {
        if self.capture.as_ref().is_some_and(|c| c.state == CaptureState::Cancelled) {
            return Verdict::Pass;
        }
        self.last_pos = (x, y);
        let slop = self.click_slop();
        let trail_slop = (self.cfg.settings.min_stroke_px as i32 / 2).max(slop);
        let drag_timeout = Duration::from_millis(self.cfg.settings.drag_timeout_ms as u64);
        let Some(cap) = &mut self.capture else {
            self.detect_edge(x, y);
            return Verdict::Pass;
        };
        match cap.state {
            CaptureState::Passthrough | CaptureState::Cancelled => return Verdict::Pass,
            CaptureState::Done => {
                // 已执行：轨迹继续画（视觉连贯），不再识别
                if self.cfg.settings.trail.show && cap.trace_at.is_some() {
                    self.overlay.line_to(x, y);
                }
                return Verdict::Pass;
            }
            CaptureState::Active => {}
        }

        // 按住触发键不动超时 → 补发按下，此后放行（保留原生拖拽）。已开始移动则不再超时。
        if !cap.moved && cap.is_plain_click() && cap.started.elapsed() > drag_timeout {
            let b = cap.button().unwrap();
            crate::log!("{b:?} 按住不动超过 {}ms，交还原生拖拽", drag_timeout.as_millis());
            cap.state = CaptureState::Passthrough;
            if let Trigger::Edge(_) = cap.trigger {
                cap.held.clear();
                self.edge_disarmed = true;
            }
            self.overlay.finish();
            self.target = None;
            self.replay(b, false);
            return Verdict::Pass;
        }
        let (dx, dy) = ((x - cap.origin.0).abs(), (y - cap.origin.1).abs());
        cap.moved |= dx > slop || dy > slop;
        cap.show_trail |= dx.max(dy) > trail_slop;

        match cap.trigger {
            Trigger::Button(_) => self.track(x, y, true),
            Trigger::Edge(e) => {
                let away = edge::distance_from(e, x, y, self.overlay.virtual_rect());
                if cap.held.is_empty() && cap.presses.is_empty() && away > EDGE_LEAVE_PX {
                    self.end_edge_capture();
                    return Verdict::Pass;
                }
                // 只有存在"边缘 + 轨迹"类绑定时才记录轨迹（沿边摩擦等）
                let (draw, tracing, cands) = (!cap.presses.is_empty(), cap.trace_at.is_some(), cap.candidates(None, true));
                if tracing || self.feasible(&cands) {
                    self.track(x, y, draw);
                }
            }
        }
        Verdict::Pass
    }

    fn on_wheel(&mut self, delta: i32) -> Verdict {
        let Some(cap) = &self.capture else {
            return Verdict::Pass;
        };
        match cap.state {
            CaptureState::Passthrough => return Verdict::Pass,
            CaptureState::Done | CaptureState::Cancelled => return Verdict::Swallow,
            CaptureState::Active => {}
        }
        // 边缘手势的第一步是滚轮：须贴边（离开一点就是普通滚动）
        if matches!(cap.trigger, Trigger::Edge(_)) && cap.presses.is_empty() && cap.trace_at.is_none() {
            let (x, y) = self.last_pos;
            if !self.pressed_at_edge(x, y) {
                self.end_edge_capture();
                return Verdict::Pass;
            }
        }
        let Some(cap) = &self.capture else { return Verdict::Pass };
        let key = PatternKey::Wheel(delta.signum());
        // 滚轮完成某个手势 → 立即执行且不记入序列，可连续滚动反复触发
        let steps = cap.candidates(Some(key), false);
        let path = cap.trace_at.is_some().then_some(self.path.as_slice());
        // 含 Shape 的滚轮组合也必须等释放，避免绕过 after_input 的提前执行保护。
        if let Some(b) = self.match_steps(&steps, &self.tracker.strokes(), path)
            .filter(|b| !b.pattern.iter().any(|s| matches!(s, Step::Shape { .. }))) {
            if let Some(cap) = &mut self.capture {
                cap.consumed = true;
            }
            self.show_label(&b, true);
            self.run_action(b.action);
            return Verdict::Swallow;
        }
        if self.feasible(&steps) {
            self.capture.as_mut().unwrap().presses.push(key);
            self.after_input();
            return Verdict::Swallow;
        }
        match cap.trigger {
            Trigger::Button(_) => Verdict::Swallow,
            Trigger::Edge(_) => {
                self.end_edge_capture();
                Verdict::Pass
            }
        }
    }

    fn on_key_down(&mut self, vk: u16) -> Verdict {
        let Some(cap) = &mut self.capture else {
            return Verdict::Pass;
        };
        if cap.state != CaptureState::Active || vk >= 0xFF {
            return Verdict::Pass;
        }
        // 捕获中按 Esc：取消手势（不执行、不补发）
        if vk == 0x1B {
            cap.state = CaptureState::Done;
            cap.consumed = true;
            if let Trigger::Edge(_) = cap.trigger {
                if cap.held.is_empty() {
                    self.end_edge_capture();
                    return Verdict::Pass;
                }
            }
            self.overlay.finish();
            self.overlay.clear_label();
            self.live_label = None;
            return Verdict::Swallow;
        }
        let key = PatternKey::Keyboard(vk);
        let (is_edge, steps) = (matches!(cap.trigger, Trigger::Edge(_)), cap.candidates(Some(key), false));
        if is_edge && !self.feasible(&steps) {
            return Verdict::Pass;
        }
        self.capture.as_mut().unwrap().presses.push(key);
        self.after_input();
        Verdict::Pass
    }

    /// 无捕获时的屏幕边/角检测（驻留防误触；触发后离开边缘才重新武装）。
    fn detect_edge(&mut self, x: i32, y: i32) {
        if self.edge_triggers.is_empty() {
            return;
        }
        let at = edge::edge_at(x, y, self.overlay.virtual_rect());
        if self.edge_disarmed {
            if at.is_none() {
                self.edge_disarmed = false;
            }
            return;
        }
        let now = Instant::now();
        match (at.filter(|e| self.edge_triggers.contains(e)), self.edge_pending) {
            (Some(e), Some((pending, since))) if pending == e => {
                if now.duration_since(since) >= EDGE_DWELL && !self.host.foreground_fullscreen() {
                    self.edge_pending = None;
                    self.start_edge_capture(e, x, y);
                }
            }
            (Some(e), _) => self.edge_pending = Some((e, now)),
            (None, _) => self.edge_pending = None,
        }
    }

    // ------------------------------------------------------------ 捕获生命周期

    fn start_capture(&mut self, trigger: Trigger, x: i32, y: i32) {
        self.start_capture_at(trigger, x, y, self.host.target_at(x, y));
    }

    fn start_capture_at(&mut self, trigger: Trigger, x: i32, y: i32, target: Target) {
        self.start_path(x, y);
        self.target = Some(target);
        self.vars = self.build_vars(x, y);
        self.last_pos = (x, y);
        self.label_at = self.host.label_anchor(x, y);
        self.active_at = Some(Instant::now());
        self.overlay.set_color(&self.cfg.settings.trail.color_normal);
        self.overlay.start_session(x, y);
        self.live_label = None;
        self.capture = Some(Capture {
            trigger,
            presses: Vec::new(),
            trace_at: None,
            held: Vec::new(),
            started: Instant::now(),
            origin: (x, y),
            moved: false,
            show_trail: false,
            state: CaptureState::Active,
            consumed: false,
            as_button: None,
        });
    }

    /// 边缘触发开始：纯边角绑定立即执行（如 左下角=开始菜单）；
    /// 否则若存在以该边角开头的组合手势，进入边缘捕获。
    fn start_edge_capture(&mut self, e: u8, x: i32, y: i32) {
        self.start_capture(Trigger::Edge(e), x, y);
        let steps = vec![matcher::build_steps(Trigger::Edge(e), &[], None)];
        if let Some(b) = self.match_steps(&steps, &[], None) {
            crate::log!("手势命中: {}（{}）", b.name, edge::name(e));
            self.show_label(&b, true);
            self.run_action(b.action);
            self.end_edge_capture();
        } else if !self.feasible(&steps) {
            self.end_edge_capture(); // 该作用域没有这个边角的手势
        }
    }

    /// 结束边缘捕获（取消/完成），离开边缘前不再触发。
    fn end_edge_capture(&mut self) {
        self.capture = None;
        self.overlay.finish();
        self.target = None;
        self.edge_disarmed = true;
        self.edge_pending = None;
    }

    /// 终止进行中的捕获（不执行、不补发）。
    fn abort_capture(&mut self) {
        self.overlay.finish();
        self.overlay.clear_label();
        self.capture = None;
        self.live_label = None;
        self.target = None;
    }

    /// 超长笔画立即清除；已吞下的按键仍等到释放，避免把孤立的抬起交给应用。
    fn cancel_long_stroke(&mut self) {
        let Some(cap) = &mut self.capture else { return };
        cap.state = CaptureState::Cancelled;
        if matches!(cap.trigger, Trigger::Edge(_)) && cap.held.is_empty() {
            self.end_edge_capture();
        } else {
            self.overlay.finish();
            self.target = None;
        }
        self.overlay.clear_label();
        self.live_label = None;
        self.label_deadline = None;
        if self.cfg.settings.label.show {
            let text = mousetroke_core::i18n::t(mousetroke_core::i18n::keys::OVERLAY_GESTURE_TOO_LONG);
            let color = self.cfg.settings.label.color_normal.clone();
            self.show_text(text, &color, Some(TOO_LONG_LABEL));
        }
        crate::log!("手势过长（超过 {MAX_STROKE_SEGMENTS} 段），已取消");
    }

    /// 与量化追踪器同时起笔，包括边缘捕获首次按键时的重置。
    fn start_path(&mut self, x: i32, y: i32) {
        self.tracker.start(x as f64, y as f64);
        self.path.clear();
        self.path.push((x as f64, y as f64));
    }

    fn record_point(&mut self, x: i32, y: i32) {
        let point = (x as f64, y as f64);
        if self.path.last().is_some_and(|&(px, py)| (point.0 - px).powi(2) + (point.1 - py).powi(2) < 4.0) {
            return;
        }
        if self.path.len() == SHAPE_PATH_CAP {
            for i in 0..SHAPE_PATH_CAP / 2 {
                self.path[i] = self.path[i * 2];
            }
            self.path.truncate(SHAPE_PATH_CAP / 2);
        }
        self.path.push(point);
    }

    /// 轨迹追踪；分段变化时重新匹配。
    fn track(&mut self, x: i32, y: i32, draw: bool) {
        let before = self.tracker.segments().len();
        self.tracker.add_point(x as f64, y as f64);
        let after = self.tracker.segments().len();
        if after > MAX_STROKE_SEGMENTS {
            self.cancel_long_stroke();
            return;
        }
        self.record_point(x, y);
        // 手抖不画轨迹：超过点击容差后才开始画（第一笔从按下点连过来）
        let shown = self.capture.as_ref().is_some_and(|c| c.show_trail);
        if draw && shown && self.cfg.settings.trail.show {
            self.overlay.line_to(x, y);
        }
        if after != before {
            if let Some(cap) = &mut self.capture {
                cap.trace_at.get_or_insert(cap.presses.len());
            }
            self.after_input();
        }
    }

    /// 捕获序列变化后：更新实时标签；命中"识别即执行"的绑定则立刻执行。
    fn after_input(&mut self) {
        let Some(cap) = &self.capture else { return };
        let steps = cap.candidates(None, false);
        let segs = self.tracker.strokes();
        let path = cap.trace_at.is_some().then_some(self.path.as_slice());
        let hit = self.match_steps(&steps, &segs, path);
        if let Some(b) = &hit {
            // Shape 必须等到释放：尚未画完的 G 可能像 C，忽略其「识别即执行」设置。
            if b.execute_on_match && !b.pattern.iter().any(|s| matches!(s, Step::Shape { .. })) {
                crate::log!("手势命中: {}（识别即执行）", b.name);
                let cap = self.capture.as_mut().unwrap();
                cap.state = CaptureState::Done;
                let edge_idle = matches!(cap.trigger, Trigger::Edge(_)) && cap.held.is_empty();
                self.overlay.set_color(&self.cfg.settings.trail.color_matched);
                self.show_label(b, true);
                self.run_action(b.action.clone());
                if edge_idle {
                    // 沿边摩擦类：没有按住的键，直接结束
                    if let Some(cap) = &self.capture {
                        self.log_shape_sample(cap);
                    }
                    self.capture = None;
                    self.end_visuals(true);
                    self.edge_disarmed = true;
                }
                return;
            }
        }
        // 没命中时显示已识别出的方向（→ ↓ →），让人知道引擎"看到"了什么。
        // 边缘捕获在按下附加键之前只是沿边移动，不提示
        let drawing = cap.moved && (matches!(cap.trigger, Trigger::Button(_)) || !cap.presses.is_empty());
        match hit {
            Some(b) => self.show_label(&b, false),
            None if drawing && !segs.is_empty() && self.cfg.settings.label.show => {
                let text = recognizer::arrows(&segs);
                if self.live_label.as_deref() != Some(text.as_str()) {
                    let color = self.cfg.settings.label.color_normal.clone();
                    self.show_text(&text, &color, None);
                }
            }
            None if self.live_label.is_some() => {
                self.overlay.clear_label();
                self.live_label = None;
            }
            None => {}
        }
    }

    /// 在光标上方显示一段文字；`hold` = 保留多久后自动清除（None = 跟随捕获）。
    fn show_text(&mut self, text: &str, color: &str, hold: Option<Duration>) {
        let (x, y) = self.label_at;
        self.overlay.draw_label(text, x, y, color);
        self.live_label = Some(text.to_string());
        if let Some(d) = hold {
            self.label_deadline = Some(Instant::now() + d);
        }
    }

    /// 显示手势名标签。`matched` = 已执行（用命中色并定时消失）。
    /// 内置名称（有 name_key 且未被改名）按当前语言显示。
    fn show_label(&mut self, b: &Binding, matched: bool) {
        use mousetroke_core::i18n::display_name;
        if !self.cfg.settings.label.show || b.hide_label {
            return;
        }
        let name = display_name(&b.name, &b.name_key);
        if !matched && self.live_label.as_deref() == Some(name) {
            return;
        }
        let l = &self.cfg.settings.label;
        let color = if matched { &l.color_matched } else { &l.color_normal };
        let (x, y) = self.label_at;
        self.overlay.draw_label(name, x, y, color);
        self.live_label = Some(name.to_string());
        if matched {
            self.label_deadline = Some(Instant::now() + HIT_LABEL);
        }
    }

    /// 结束一次捕获的视觉：命中 → 命中色淡出；否则淡出/清除并清掉标签。
    fn end_visuals(&mut self, matched: bool) {
        if matched {
            self.overlay.set_color(&self.cfg.settings.trail.color_matched);
        }
        if self.cfg.settings.trail.fade_out {
            self.overlay.fade_out();
        } else {
            self.overlay.finish();
        }
        if !matched || self.label_deadline.is_none() {
            self.overlay.clear_label();
            self.live_label = None;
        }
        self.target = None;
    }

    fn finish_gesture(&mut self, cap: Capture) {
        self.log_shape_sample(&cap);
        let segs = self.tracker.strokes();
        let steps = cap.candidates(None, false);
        let path = cap.trace_at.is_some().then_some(self.path.as_slice());
        match self.match_steps(&steps, &segs, path) {
            // 连续滚轮执行过后松开：只剩触发本身，不再匹配一次
            Some(b) if !(cap.consumed && segs.is_empty() && cap.presses.len() <= cap.as_button.is_some() as usize) => {
                crate::log!("手势命中: {}（作用域 {}）", b.name, self.scope_name());
                self.show_label(&b, true);
                self.run_action(b.action);
                self.end_visuals(true);
            }
            _ => {
                if cap.consumed {
                    // 期间执行过动作（连续滚轮等）：静默结束
                } else if cap.is_plain_click() {
                    // 纯点击 → 工作线程补发，保留右键菜单等正常语义
                    let b = cap.button().unwrap();
                    crate::log!("{b:?} 单击，补发（按下 {}ms）", cap.started.elapsed().as_millis());
                    self.replay(b, true);
                } else {
                    // 画了但没识别：不补发（否则会弹出意料之外的右键菜单），屏幕上提示一下
                    crate::log!("手势未识别（{} 段轨迹，{} 个附加键）", segs.len(), cap.presses.len());
                    self.end_visuals(false);
                    if self.cfg.settings.label.show {
                        let text = match recognizer::arrows(&segs) {
                            a if a.is_empty() => mousetroke_core::i18n::t(mousetroke_core::i18n::keys::OVERLAY_UNRECOGNIZED).to_string(),
                            a => mousetroke_core::i18n::tf(mousetroke_core::i18n::keys::OVERLAY_UNRECOGNIZED_ARROWS, &[("arrows", &a)]),
                        };
                        let color = self.cfg.settings.label.color_normal.clone();
                        self.show_text(&text, &color, Some(HIT_LABEL));
                    }
                    return;
                }
                self.end_visuals(cap.consumed);
            }
        }
    }

    // ------------------------------------------------------------ 匹配辅助

    fn log_shape_sample(&self, cap: &Capture) {
        if self.shape_samples && cap.trace_at.is_some() {
            if let Some(normalized) = shape::normalize(&self.path) {
                self.host.shape_sample(shape::encode(&normalized));
            }
        }
    }

    /// 按候选顺序匹配，返回第一个命中。
    fn match_steps(&self, candidates: &[Steps], segs: &[Segment], path: Option<&[(f64, f64)]>) -> Option<Binding> {
        let scopes = self.active_scopes();
        candidates.iter().find_map(|steps| matcher::match_chain_path(&scopes, steps, segs, path).map(|(b, _)| b.clone()))
    }

    /// 任一候选能延续某个手势。
    fn feasible(&self, candidates: &[Steps]) -> bool {
        let scopes = self.active_scopes();
        candidates.iter().any(|steps| scopes.iter().any(|s| matcher::is_feasible_prefix(s, steps)))
    }

    /// 边缘捕获中，(x, y) 是否贴着该边/角（边缘手势的第一步按键/滚轮要求）。
    fn pressed_at_edge(&self, x: i32, y: i32) -> bool {
        match self.capture.as_ref().map(|c| c.trigger) {
            Some(Trigger::Edge(e)) => edge::distance_from(e, x, y, self.overlay.virtual_rect()) <= EDGE_PRESS_PX,
            _ => false,
        }
    }

    fn click_slop(&self) -> i32 {
        (self.cfg.settings.min_stroke_px as i32 / 3).max(CLICK_SLOP_MIN)
    }

    /// 补发触发键并登记回声。熔断：1 秒内补发单击超过 6 次（人手做不到）
    /// 说明回声识别失效、正在和别的鼠标工具互相重发 → 停止补发，打断循环。
    fn replay(&mut self, btn: MouseButton, click: bool) {
        let now = Instant::now();
        if click {
            self.recent_replays.retain(|t| now.duration_since(*t) < Duration::from_secs(1));
            if self.recent_replays.len() >= 6 {
                crate::log!("{btn:?} 补发过于频繁（疑似与其他鼠标工具互相重发），本次不补发");
                return;
            }
            self.recent_replays.push_back(now);
        }
        self.echoes.push((btn, true, now + ECHO_WINDOW));
        if click {
            self.echoes.push((btn, false, now + ECHO_WINDOW));
        }
        self.host.replay(btn, click);
    }

    /// 异步执行动作（工作线程；钩子回调里绝不阻塞）。
    fn run_action(&self, action: Action) {
        let target = self.target.clone();
        let (vars, pos) = (self.vars.clone(), self.last_pos);
        self.host.execute(action, target, vars, pos);
    }

    fn active_scopes(&self) -> Vec<&Scope> {
        let t = self.target.as_ref();
        matcher::resolve_scopes(&self.cfg, t.is_some_and(|t| t.is_desktop), t.and_then(|t| t.exe.as_deref()))
    }

    fn scope_name(&self) -> String {
        self.active_scopes().first().map(|s| s.name.clone()).unwrap_or_default()
    }
}

// ---------------------------------------------------------------- 变量与执行

impl Engine {
    /// 手势开始时构建变量上下文（惰性变量在执行时再取）。
    fn build_vars(&self, x: i32, y: i32) -> Vars {
        let mut v = Vars::new();
        if let Some(t) = &self.target {
            v.insert("MS_PID".into(), t.pid.to_string());
            if let Some(exe) = &t.exe {
                v.insert("MS_EXE".into(), exe.clone());
                v.insert("MS_APP_NAME".into(), matcher::file_name(exe).to_string());
            }
            v.insert("MS_WID".into(), t.wid.to_string());
            v.insert("MS_WIN_NAME".into(), self.host.window_title(t.wid));
        }
        v.insert("MS_MOUSE_X".into(), x.to_string());
        v.insert("MS_MOUSE_Y".into(), y.to_string());
        let (_, _, _, vh) = self.overlay.virtual_rect();
        v.insert("MS_MOUSE_Y_FLIP".into(), (vh - y).to_string());
        v.insert(
            "MS_TARGET_MODE".into(),
            match self.cfg.settings.target_mode {
                mousetroke_core::schema::TargetMode::UnderCursor => "UnderCursor".into(),
                mousetroke_core::schema::TargetMode::Focused => "Focused".into(),
            },
        );
        v
    }
}

#[cfg(test)]
#[path = "engine_tests.rs"]
mod tests;
