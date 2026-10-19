//! 宿主快照由主线程计时器刷新，tap 回调内不跨进程查询前台应用。
use std::cell::RefCell;
use std::collections::VecDeque;

use mousetroke_core::{
    schema::{Action, MouseButton},
    vars::Vars,
};
use mousetroke_engine::{Canvas, Host, Target};
use objc2::MainThreadMarker;
use objc2_app_kit::{NSScreen, NSWorkspace};
use objc2_core_foundation::CGPoint;
use objc2_core_graphics::{CGEvent, CGEventField, CGEventTapLocation, CGEventType, CGMouseButton};
use objc2_foundation::NSLocale;

#[derive(Clone, Copy)]
struct Replay {
    button: MouseButton,
    click: bool,
}
thread_local! {
    static REPLAYS: RefCell<VecDeque<Replay>> = RefCell::new(VecDeque::with_capacity(64));
    static TARGET: RefCell<Target> = const { RefCell::new(Target {
        wid: 0, pid: 0, exe: None, class: String::new(), is_desktop: false,
    }) };
    static SCREEN: RefCell<Screen> = const { RefCell::new(Screen {
        rect: (0, 0, 1920, 1080), anchor: (960, 864), center: (960, 540),
    }) };
}
#[derive(Clone, Copy, PartialEq)]
struct Screen {
    rect: (i32, i32, i32, i32),
    anchor: (i32, i32),
    center: (i32, i32),
}

pub fn system_langs() -> Vec<String> {
    objc2::rc::autoreleasepool(|_| {
        NSLocale::preferredLanguages()
            .iter()
            .map(|lang| lang.to_string())
            .collect()
    })
}

pub fn refresh_target() {
    let front = NSWorkspace::sharedWorkspace().frontmostApplication();
    let target = front
        .map(|app| Target {
            wid: 0,
            pid: app.processIdentifier() as u32,
            exe: app
                .executableURL()
                .and_then(|url| url.path())
                .map(|path| path.to_string()),
            class: app.bundleIdentifier().map(|id| id.to_string()).unwrap_or_default(),
            is_desktop: false,
        })
        .unwrap_or(Target {
            wid: 0,
            pid: 0,
            exe: None,
            class: String::new(),
            is_desktop: false,
        });
    TARGET.with(|slot| *slot.borrow_mut() = target);
}

pub fn refresh_screen(mtm: MainThreadMarker) -> bool {
    let screens = NSScreen::screens(mtm);
    let Some(primary) = screens.iter().next() else {
        return false;
    };
    // AppKit 原点在主屏左下，Quartz 原点在主屏左上；两者均使用逻辑坐标点。
    let primary_height = primary.frame().size.height;
    let main = NSScreen::mainScreen(mtm).unwrap_or(primary);
    let frame = main.frame();
    let work = main.visibleFrame();
    let mut left = 0.0f64;
    let mut top = 0.0f64;
    let mut right = 0.0f64;
    let mut bottom = 0.0f64;
    for screen in screens.iter() {
        let r = screen.frame();
        left = left.min(r.origin.x);
        top = top.min(primary_height - r.origin.y - r.size.height);
        right = right.max(r.origin.x + r.size.width);
        bottom = bottom.max(primary_height - r.origin.y);
    }
    let new = Screen {
        rect: (left as i32, top as i32, (right - left) as i32, (bottom - top) as i32),
        anchor: (
            (work.origin.x + work.size.width / 2.0) as i32,
            (primary_height - work.origin.y - work.size.height + work.size.height * 0.8) as i32,
        ),
        center: (
            (frame.origin.x + frame.size.width / 2.0) as i32,
            (primary_height - frame.origin.y - frame.size.height / 2.0) as i32,
        ),
    };
    SCREEN.with(|slot| {
        let changed = *slot.borrow() != new;
        *slot.borrow_mut() = new;
        changed
    })
}

pub fn screen_center() -> (i32, i32) {
    SCREEN.with(|slot| slot.borrow().center)
}
pub fn cursor_pos() -> Option<CGPoint> {
    CGEvent::new(None).map(|event| CGEvent::location(Some(&event)))
}

pub struct SystemHost;
impl Host for SystemHost {
    fn target_at(&self, _x: i32, _y: i32) -> Target {
        TARGET.with(|slot| slot.borrow().clone())
    }
    fn ignore_target(&self, target: &Target) -> bool {
        target.pid == std::process::id() || target.class == "io.github.eiaao.mousetroke"
    }
    fn window_title(&self, _wid: usize) -> String {
        String::new()
    }
    fn foreground_fullscreen(&self) -> bool {
        false
    }
    fn replay(&self, button: MouseButton, click: bool) {
        REPLAYS.with(|slot| {
            if let Ok(mut queue) = slot.try_borrow_mut() {
                if queue.len() < 64 {
                    queue.push_back(Replay { button, click });
                } else {
                    crate::log!("补发队列已满，丢弃本次补发");
                }
            }
        });
    }
    fn execute(&self, action: Action, _target: Option<Target>, _vars: Vars, _pos: (i32, i32)) {
        // 只记录枚举类型，绝不 Debug 整个动作，避免泄露脚本、网址、按键内容。
        let kind = match action {
            Action::KeySequence { .. } => "key_sequence",
            Action::Shell { .. } => "shell",
            Action::Lua { .. } => "lua",
            Action::OpenUrl { .. } => "open_url",
            Action::Menu { .. } => "menu",
            Action::LaunchApp { .. } => "launch_app",
            Action::WindowOp { .. } => "window_op",
            Action::SystemKey { .. } => "system_key",
        };
        crate::log!("动作暂未支持（M3）: {kind}");
    }
    fn label_anchor(&self, _x: i32, _y: i32) -> (i32, i32) {
        SCREEN.with(|slot| slot.borrow().anchor)
    }
    fn new_canvas(&self, _color: &str, _width: f64) -> Option<Box<dyn Canvas>> {
        Some(Box::new(NullCanvas(SCREEN.with(|slot| slot.borrow().rect))))
    }
    fn init_shape_samples(&self) {}
    fn shape_sample(&self, encoded: Vec<i32>) {
        crate::log::shape_sample(encoded);
    }
    fn cursor_pos(&self) -> (i32, i32) {
        cursor_pos().map(|p| (p.x as i32, p.y as i32)).unwrap_or((0, 0))
    }
    fn set_automatic_updates(&self, _enabled: bool) {}
    fn system_langs(&self) -> Vec<String> {
        system_langs()
    }
    fn refresh_texts(&self) {
        crate::menu::request_refresh();
    }
}

/// 在 tap 回调返回后的下一次主 run loop 计时器执行，队列上限固定，不同步重入引擎。
pub fn drain_replays() {
    for _ in 0..64 {
        let replay = REPLAYS.with(|slot| slot.borrow_mut().pop_front());
        let Some(replay) = replay else {
            break;
        };
        let Some(pos) = cursor_pos() else {
            crate::log!("读取补发位置失败");
            continue;
        };
        let (down, up, number) = match replay.button {
            MouseButton::Left => (CGEventType::LeftMouseDown, CGEventType::LeftMouseUp, 0),
            MouseButton::Right => (CGEventType::RightMouseDown, CGEventType::RightMouseUp, 1),
            MouseButton::Middle => (CGEventType::OtherMouseDown, CGEventType::OtherMouseUp, 2),
            MouseButton::X1 => (CGEventType::OtherMouseDown, CGEventType::OtherMouseUp, 3),
            MouseButton::X2 => (CGEventType::OtherMouseDown, CGEventType::OtherMouseUp, 4),
        };
        // 先创建两件事件，避免创建 up 失败后留下永久按下状态。
        let first = mouse_event(down, number, pos, crate::events::INJECT_TAG);
        let last = replay
            .click
            .then(|| mouse_event(up, number, pos, crate::events::INJECT_TAG));
        if first.is_none() || last.as_ref().is_some_and(Option::is_none) {
            crate::log!("创建补发事件失败");
            continue;
        }
        CGEvent::post(CGEventTapLocation::HIDEventTap, first.as_deref());
        if let Some(Some(last)) = last {
            CGEvent::post(CGEventTapLocation::HIDEventTap, Some(&last));
        }
        crate::log!("点击补发: {:?}, click={}", replay.button, replay.click);
    }
}

pub fn mouse_event(
    kind: CGEventType,
    number: u32,
    pos: CGPoint,
    tag: i64,
) -> Option<objc2_core_foundation::CFRetained<CGEvent>> {
    let event = CGEvent::new_mouse_event(None, kind, pos, CGMouseButton(number))?;
    CGEvent::set_integer_value_field(Some(&event), CGEventField::MouseEventButtonNumber, number as i64);
    CGEvent::set_integer_value_field(Some(&event), CGEventField::MouseEventClickState, 1);
    CGEvent::set_integer_value_field(Some(&event), CGEventField::EventSourceUserData, tag);
    Some(event)
}

struct NullCanvas((i32, i32, i32, i32));
impl Canvas for NullCanvas {
    fn set_color(&mut self, _: &str) {}
    fn set_width(&mut self, _: f64) {}
    fn start_session(&mut self, _: i32, _: i32) {}
    fn line_to(&mut self, _: i32, _: i32) {}
    fn finish(&mut self) {}
    fn fade_out(&mut self) {}
    fn tick_fade(&mut self) {}
    fn draw_label(&mut self, _: &str, _: i32, _: i32, _: &str) {}
    fn clear_label(&mut self) {}
    fn virtual_rect(&self) -> (i32, i32, i32, i32) {
        self.0
    }
}
