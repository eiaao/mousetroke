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

use crate::overlay::{geometry::Rect, rect, Overlay};

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
        center: (960, 540), displays: Vec::new(),
    }) };
}
#[derive(PartialEq)]
struct Screen {
    center: (i32, i32),
    displays: Vec<Display>,
}
#[derive(PartialEq)]
struct Display {
    frame: Rect,
    visible: Rect,
    scale: f64,
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
            class: app
                .bundleIdentifier()
                .map(|id| id.to_string())
                .unwrap_or_default(),
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
    let new = Screen {
        displays: screens
            .iter()
            .map(|screen| Display {
                frame: rect(screen.frame()).quartz(primary_height),
                visible: rect(screen.visibleFrame()).quartz(primary_height),
                scale: screen.backingScaleFactor(),
            })
            .collect(),
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
        crate::input::enqueue(action);
    }
    fn label_anchor(&self, x: i32, y: i32) -> (i32, i32) {
        SCREEN.with(|slot| {
            let screen = slot.borrow();
            screen
                .displays
                .iter()
                .find(|s| s.frame.contains(x as f64, y as f64))
                .or_else(|| screen.displays.first())
                .map(|s| s.visible.anchor())
                .unwrap_or(screen.center)
        })
    }
    fn new_canvas(&self, color: &str, width: f64) -> Option<Box<dyn Canvas>> {
        Some(Box::new(Overlay::new(
            MainThreadMarker::new()?,
            color,
            width,
        )?))
    }
    fn init_shape_samples(&self) {}
    fn shape_sample(&self, encoded: Vec<i32>) {
        crate::log::shape_sample(encoded);
    }
    fn cursor_pos(&self) -> (i32, i32) {
        cursor_pos()
            .map(|p| (p.x as i32, p.y as i32))
            .unwrap_or((0, 0))
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
    CGEvent::set_integer_value_field(
        Some(&event),
        CGEventField::MouseEventButtonNumber,
        number as i64,
    );
    CGEvent::set_integer_value_field(Some(&event), CGEventField::MouseEventClickState, 1);
    CGEvent::set_integer_value_field(Some(&event), CGEventField::EventSourceUserData, tag);
    Some(event)
}
