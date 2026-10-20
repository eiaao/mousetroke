//! 主线程覆盖层：输入只追加有界路径，主 run loop 合并提交到 Core Animation。
use std::cell::Cell;
use std::ptr;
use std::time::{Duration, Instant};

use mousetroke_engine::Canvas;
use objc2::{
    define_class, msg_send, rc::Retained, runtime::Bool, MainThreadMarker, MainThreadOnly,
};
use objc2_app_kit::{
    NSBackingStoreType, NSColor, NSFont, NSFontAttributeName, NSScreen, NSScreenSaverWindowLevel,
    NSStringDrawing, NSView, NSWindow, NSWindowCollectionBehavior, NSWindowStyleMask,
};
use objc2_core_foundation::{CFRetained, CFString, CGPoint, CGRect, CGSize};
use objc2_core_graphics::{CGColor, CGMutablePath};
use objc2_foundation::{ns_string, NSDictionary, NSNumber, NSObjectProtocol, NSString};
use objc2_quartz_core::{
    CABasicAnimation, CALayer, CAMediaTiming, CAShapeLayer, CATextLayer, CATransaction,
};

pub(crate) mod geometry;
use geometry::{rgba, Rect};

const FADE: Duration = Duration::from_millis(200);
const MAX_SEGMENTS: usize = 8192;
thread_local! {
    static STARTS: Cell<usize> = const { Cell::new(0) };
    static LINES: Cell<usize> = const { Cell::new(0) };
    /// 正在绘制或淡出；主循环据此决定是否保持快节奏。
    static ACTIVE: Cell<bool> = const { Cell::new(false) };
}

pub fn busy() -> bool {
    ACTIVE.with(Cell::get)
}

pub fn counters() -> (usize, usize) {
    (STARTS.with(Cell::get), LINES.with(Cell::get))
}

define_class!(
    // 安全：不添加 ivar，所有实例及重写方法均只在 AppKit 主线程使用。
    #[unsafe(super(NSWindow))]
    #[thread_kind = MainThreadOnly]
    struct OverlayWindow;
    impl OverlayWindow {
        #[unsafe(method(canBecomeKeyWindow))]
        fn can_become_key(&self) -> Bool { Bool::NO }
        #[unsafe(method(canBecomeMainWindow))]
        fn can_become_main(&self) -> Bool { Bool::NO }
    }
    unsafe impl NSObjectProtocol for OverlayWindow {}
);

/// 每次属性更新都关闭隐式动画；显式 opacity 动画另行添加。
struct Transaction;
impl Transaction {
    fn new() -> Self {
        CATransaction::begin();
        CATransaction::setDisableActions(true);
        Self
    }
}
impl Drop for Transaction {
    fn drop(&mut self) {
        CATransaction::commit();
    }
}

pub fn rect(frame: CGRect) -> Rect {
    Rect {
        x: frame.origin.x,
        y: frame.origin.y,
        w: frame.size.width,
        h: frame.size.height,
    }
}

fn color(value: &str) -> CFRetained<CGColor> {
    let [r, g, b, a] = rgba(value);
    CGColor::new_generic_rgb(r, g, b, a)
}

struct ScreenWindow {
    window: Retained<OverlayWindow>,
    frame: Rect,
    root: Retained<CALayer>,
    trail: Retained<CAShapeLayer>,
    pill: Retained<CALayer>,
    label: Retained<CATextLayer>,
    path: CFRetained<CGMutablePath>,
    visible: bool,
}

impl ScreenWindow {
    fn new(mtm: MainThreadMarker, screen: &NSScreen, primary_height: f64) -> Option<Self> {
        let frame = screen.frame();
        // 安全：由 NSWindow 的指定初始化器初始化无 ivar 的子类，使用有效的屏幕点矩形。
        let window: Option<Retained<OverlayWindow>> = unsafe {
            msg_send![OverlayWindow::alloc(mtm), initWithContentRect: frame,
                styleMask: NSWindowStyleMask::Borderless, backing: NSBackingStoreType::Buffered,
                defer: false]
        };
        let window = window?;
        // 安全：窗口由 Retained 管理，close 时不能再由 AppKit 额外 release。
        unsafe { window.setReleasedWhenClosed(false) };
        window.setOpaque(false);
        window.setBackgroundColor(Some(&NSColor::clearColor()));
        window.setIgnoresMouseEvents(true);
        window.setHasShadow(false);
        window.setHidesOnDeactivate(false);
        // 公开的屏保级别 1000 高于菜单栏 24；配合全屏辅助空间可覆盖普通全屏应用。
        // 不使用私有 shielding 级别，也不抢占键盘焦点。
        window.setLevel(NSScreenSaverWindowLevel);
        window.setCollectionBehavior(
            NSWindowCollectionBehavior::CanJoinAllSpaces
                | NSWindowCollectionBehavior::FullScreenAuxiliary
                | NSWindowCollectionBehavior::Stationary
                | NSWindowCollectionBehavior::IgnoresCycle,
        );
        let bounds = CGRect::new(CGPoint::new(0.0, 0.0), frame.size);
        let view = NSView::initWithFrame(NSView::alloc(mtm), bounds);
        let root = CALayer::layer();
        root.setFrame(bounds);
        root.setMasksToBounds(true);
        // 先设自定义 layer 再 wantsLayer，使用 layer-hosting view，子层完全归我们管理。
        view.setLayer(Some(&root));
        view.setWantsLayer(true);
        window.setContentView(Some(&view));
        let trail = CAShapeLayer::layer();
        trail.setFrame(bounds);
        trail.setFillColor(None);
        trail.setLineCap(ns_string!("round"));
        trail.setLineJoin(ns_string!("round"));
        root.addSublayer(&trail);
        let pill = CALayer::layer();
        pill.setBackgroundColor(Some(&color("#14181cc8")));
        pill.setCornerRadius(9.0);
        pill.setMasksToBounds(true);
        pill.setHidden(true);
        root.addSublayer(&pill);
        let label = CATextLayer::layer();
        let font = NSFont::systemFontOfSize(24.0);
        let name = CFString::from_str(&font.fontName().to_string());
        // 安全：CATextLayer.font 文档允许 CFString 字体名；这里取系统字体的 PostScript 名。
        unsafe { label.setFont(Some(name.as_ref())) };
        label.setFontSize(24.0);
        label.setAlignmentMode(ns_string!("center"));
        label.setTruncationMode(ns_string!("end"));
        pill.addSublayer(&label);
        let scale = screen.backingScaleFactor();
        for layer in [&*root, &*trail, &*pill, &*label] {
            layer.setContentsScale(scale);
        }
        window.orderOut(None);
        Some(Self {
            window,
            frame: rect(frame).quartz(primary_height),
            root,
            trail,
            pill,
            label,
            path: CGMutablePath::new(),
            visible: false,
        })
    }

    fn show(&mut self) {
        if !self.visible {
            self.window.orderFrontRegardless();
            self.visible = true;
        }
    }

    fn clear(&mut self) {
        clear_layers(&self.root, &self.trail, &self.pill);
        self.path = CGMutablePath::new();
        if self.visible {
            self.window.orderOut(None);
            self.visible = false;
        }
    }
}

fn clear_layers(root: &CALayer, trail: &CAShapeLayer, pill: &CALayer) {
    root.removeAllAnimations();
    root.setOpacity(1.0);
    trail.setPath(None);
    pill.setHidden(true);
}

impl Drop for ScreenWindow {
    fn drop(&mut self) {
        self.window.orderOut(None);
        self.window.close();
    }
}

pub struct Overlay {
    windows: Vec<ScreenWindow>,
    virtual_rect: (i32, i32, i32, i32),
    previous: Option<(f64, f64)>,
    segments: usize,
    dirty: bool,
    fade_pending: bool,
    fade_deadline: Option<Instant>,
}

impl Overlay {
    pub fn new(mtm: MainThreadMarker, color: &str, width: f64) -> Option<Self> {
        let screens = NSScreen::screens(mtm);
        let primary_height = screens.firstObject()?.frame().size.height;
        let _tx = Transaction::new();
        let windows: Vec<_> = screens
            .iter()
            .map(|s| ScreenWindow::new(mtm, &s, primary_height))
            .collect::<Option<_>>()?;
        let union = windows.iter().map(|s| s.frame).reduce(Rect::union)?;
        let mut overlay = Self {
            windows,
            virtual_rect: (
                union.x as i32,
                union.y as i32,
                union.w as i32,
                union.h as i32,
            ),
            previous: None,
            segments: 0,
            dirty: false,
            fade_pending: false,
            fade_deadline: None,
        };
        overlay.set_color(color);
        overlay.set_width(width);
        Some(overlay)
    }

    fn reset(&mut self) {
        self.previous = None;
        self.segments = 0;
        self.dirty = false;
        self.fade_pending = false;
        self.fade_deadline = None;
        for screen in &mut self.windows {
            screen.clear();
        }
        self.note_activity();
    }

    fn note_activity(&self) {
        let active = self.previous.is_some()
            || self.dirty
            || self.fade_pending
            || self.fade_deadline.is_some();
        ACTIVE.with(|slot| slot.set(active));
    }
}

impl Canvas for Overlay {
    fn set_color(&mut self, value: &str) {
        let _tx = Transaction::new();
        let color = color(value);
        for screen in &self.windows {
            screen.trail.setStrokeColor(Some(&color));
        }
    }

    fn set_width(&mut self, width: f64) {
        let _tx = Transaction::new();
        let width = if width.is_finite() {
            width.clamp(1.0, 200.0)
        } else {
            3.0
        };
        for screen in &self.windows {
            screen.trail.setLineWidth(width);
        }
    }

    fn start_session(&mut self, x: i32, y: i32) {
        STARTS.with(|n| n.set(n.get().saturating_add(1)));
        let _tx = Transaction::new();
        // 即使上一轮尚在淡出也立即移除动画和旧 path，不依赖旧动画的 completion。
        self.reset();
        self.previous = Some((x as f64, y as f64));
        self.note_activity();
        for screen in &self.windows {
            let (x, y) = screen.frame.layer_point(x as f64, y as f64);
            // 安全：path 仅由主线程持有；null 表示不附加变换。
            unsafe { CGMutablePath::move_to_point(Some(&screen.path), ptr::null(), x, y) };
        }
    }

    fn line_to(&mut self, x: i32, y: i32) {
        let Some(previous) = self.previous else {
            return;
        };
        if self.segments >= MAX_SEGMENTS {
            return;
        }
        LINES.with(|n| n.set(n.get().saturating_add(1)));
        // 与 Windows 相同：0.55 指数平滑，再以旧点为控制点连接到新中点。
        let point = (
            previous.0 + (x as f64 - previous.0) * 0.55,
            previous.1 + (y as f64 - previous.1) * 0.55,
        );
        let midpoint = ((previous.0 + point.0) * 0.5, (previous.1 + point.1) * 0.5);
        for screen in &self.windows {
            let control = screen.frame.layer_point(previous.0, previous.1);
            let to = screen.frame.layer_point(midpoint.0, midpoint.1);
            // 安全：有效的独占主线程 path；控制点和终点均为窗口内逻辑坐标。
            unsafe {
                CGMutablePath::add_quad_curve_to_point(
                    Some(&screen.path),
                    ptr::null(),
                    control.0,
                    control.1,
                    to.0,
                    to.1,
                )
            };
        }
        self.previous = Some(point);
        self.segments += 1;
        self.dirty = true;
    }

    fn finish(&mut self) {
        let _tx = Transaction::new();
        self.reset();
    }

    fn fade_out(&mut self) {
        self.previous = None;
        self.fade_pending = true;
        self.note_activity();
    }

    fn tick_fade(&mut self) {
        if !self.dirty && !self.fade_pending && self.fade_deadline.is_none() {
            return;
        }
        let _tx = Transaction::new();
        if self.dirty {
            for screen in &mut self.windows {
                screen.trail.setPath(Some(&screen.path));
                screen.show();
            }
            self.dirty = false;
        }
        if self.fade_pending {
            self.fade_pending = false;
            if self.windows.iter().any(|s| s.visible) {
                let animation = CABasicAnimation::animationWithKeyPath(Some(ns_string!("opacity")));
                // 安全：opacity 的动画值按 Apple 文档要求使用 NSNumber。
                unsafe {
                    animation.setFromValue(Some(&NSNumber::new_f32(1.0)));
                    animation.setToValue(Some(&NSNumber::new_f32(0.0)));
                }
                animation.setDuration(FADE.as_secs_f64());
                for screen in &self.windows {
                    screen.root.setOpacity(0.0);
                    screen
                        .root
                        .addAnimation_forKey(&animation, Some(ns_string!("fade")));
                }
                self.fade_deadline = Some(Instant::now() + FADE);
            }
        }
        if self
            .fade_deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            self.reset();
        }
        self.note_activity();
    }

    fn draw_label(&mut self, text: &str, x: i32, y: i32, value: &str) {
        let _tx = Transaction::new();
        // 未识别提示在 end_visuals 之后绘制；取消旧淡出时必须同时移除旧轨迹。
        if self.fade_pending || self.fade_deadline.is_some() {
            self.reset();
        }
        let text = NSString::from_str(text);
        let font = NSFont::systemFontOfSize(24.0);
        // 安全：字典的 NSFontAttributeName 对应 NSFont，满足 NSStringDrawing 的类型约定。
        let attributes =
            unsafe { NSDictionary::from_slices(&[NSFontAttributeName], &[font.as_ref()]) };
        let size = unsafe { text.sizeWithAttributes(Some(&attributes)) };
        for screen in &mut self.windows {
            screen.pill.setHidden(true);
            if !screen.frame.contains(x as f64, y as f64) {
                if self.segments == 0 && screen.visible {
                    screen.window.orderOut(None);
                    screen.visible = false;
                }
                continue;
            }
            // 新 toast 可以在旧轨迹淡出后到来，先恢复不透明度，避免标签不可见。
            screen.root.removeAllAnimations();
            screen.root.setOpacity(1.0);
            let center = screen.frame.layer_point(x as f64, y as f64);
            let width = (size.width.ceil() + 24.0).min(screen.frame.w);
            let height = size.height.ceil() + 10.0;
            let origin = CGPoint::new(
                (center.0 - width * 0.5).clamp(0.0, screen.frame.w - width),
                (center.1 - height * 0.5).clamp(0.0, (screen.frame.h - height).max(0.0)),
            );
            screen
                .pill
                .setFrame(CGRect::new(origin, CGSize::new(width, height)));
            screen.label.setFrame(CGRect::new(
                CGPoint::new(12.0, 5.0),
                CGSize::new((width - 24.0).max(0.0), height - 10.0),
            ));
            // 安全：CATextLayer.string 接受 NSString，setter 会复制字符串。
            unsafe { screen.label.setString(Some(&text)) };
            screen.label.setForegroundColor(Some(&color(value)));
            screen.pill.setHidden(false);
            screen.show();
        }
        self.fade_pending = false;
        self.fade_deadline = None;
    }

    fn clear_label(&mut self) {
        let _tx = Transaction::new();
        for screen in &mut self.windows {
            screen.pill.setHidden(true);
            if self.segments == 0 && screen.visible {
                screen.window.orderOut(None);
                screen.visible = false;
            }
        }
    }

    fn virtual_rect(&self) -> (i32, i32, i32, i32) {
        self.virtual_rect
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_session_clears_interrupted_fade() {
        let _tx = Transaction::new();
        let root = CALayer::layer();
        let trail = CAShapeLayer::layer();
        let pill = CALayer::layer();
        let path = CGMutablePath::new();
        // 安全：只创建一段测试路径，空变换指针表示恒等变换。
        unsafe {
            CGMutablePath::move_to_point(Some(&path), ptr::null(), 1.0, 2.0);
            CGMutablePath::add_line_to_point(Some(&path), ptr::null(), 3.0, 4.0);
        }
        trail.setPath(Some(&path));
        root.setOpacity(0.0);
        root.addAnimation_forKey(
            &CABasicAnimation::animationWithKeyPath(Some(ns_string!("opacity"))),
            Some(ns_string!("fade")),
        );
        pill.setHidden(false);
        clear_layers(&root, &trail, &pill);
        assert!(trail.path().is_none());
        assert_eq!(root.opacity(), 1.0);
        assert!(root.animationKeys().is_none_or(|keys| keys.is_empty()));
        assert!(pill.isHidden());
        // 无窗口时仍执行真实 Canvas 生命周期，覆盖计时器待提交与已开始的淡出。
        for pending in [false, true] {
            let mut canvas = Overlay {
                windows: Vec::new(),
                virtual_rect: (0, 0, 1, 1),
                previous: None,
                segments: 10,
                dirty: true,
                fade_pending: pending,
                fade_deadline: Some(Instant::now() + FADE),
            };
            canvas.start_session(10, 20);
            assert_eq!(canvas.previous, Some((10.0, 20.0)));
            assert_eq!(canvas.segments, 0);
            assert!(!canvas.dirty && !canvas.fade_pending);
            assert!(canvas.fade_deadline.is_none());
        }
    }
}
