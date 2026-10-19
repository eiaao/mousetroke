//! 真机端到端自测；用主线程计时器逐件投递未打标事件，绝不 sleep 或直接调用引擎。
use objc2_core_foundation::CGPoint;
use objc2_core_graphics::{CGEvent, CGEventTapLocation, CGEventType as Kind};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

static ENABLED: AtomicBool = AtomicBool::new(false);
static MATCHES: AtomicUsize = AtomicUsize::new(0);
static ECHO_DOWN: AtomicUsize = AtomicUsize::new(0);
static ECHO_UP: AtomicUsize = AtomicUsize::new(0);
static EXPECTED_PREFIX: OnceLock<String> = OnceLock::new();
static EXPECTED_HIT: AtomicBool = AtomicBool::new(false);
const MATCH_PREFIX: &str = "手势命中:"; // i18n:allow 共享引擎的运行日志协议

pub fn observe(text: &str) {
    if ENABLED.load(Ordering::Relaxed) && text.starts_with(MATCH_PREFIX) {
        MATCHES.fetch_add(1, Ordering::Relaxed);
        if EXPECTED_PREFIX.get().is_some_and(|prefix| text.starts_with(prefix)) {
            EXPECTED_HIT.store(true, Ordering::Relaxed);
        }
    }
}
pub fn tagged_echo(kind: Kind) {
    if ENABLED.load(Ordering::Relaxed) {
        if kind == Kind::RightMouseDown {
            ECHO_DOWN.fetch_add(1, Ordering::Relaxed);
        }
        if kind == Kind::RightMouseUp {
            ECHO_UP.fetch_add(1, Ordering::Relaxed);
        }
    }
}

pub struct SelfTest {
    launched: Instant,
    next: Instant,
    step: Option<u32>,
    origin: (i32, i32),
    original_cursor: Option<CGPoint>,
    done: bool,
}

impl SelfTest {
    pub fn new(expected_name: String) -> Self {
        let _ = EXPECTED_PREFIX.set(format!("{MATCH_PREFIX} {expected_name}"));
        EXPECTED_HIT.store(false, Ordering::Relaxed);
        ENABLED.store(true, Ordering::Relaxed);
        MATCHES.store(0, Ordering::Relaxed);
        ECHO_DOWN.store(0, Ordering::Relaxed);
        ECHO_UP.store(0, Ordering::Relaxed);
        crate::log!("自测等待事件 tap；只使用内置默认配置，不执行动作");
        crate::log!("自测预期手势: {expected_name}");
        let now = Instant::now();
        Self {
            launched: now,
            next: now,
            step: None,
            origin: (0, 0),
            original_cursor: None,
            done: false,
        }
    }

    pub fn tick(&mut self, active: bool) {
        if self.done {
            return;
        }
        let now = Instant::now();
        if now.duration_since(self.launched) > Duration::from_secs(120) {
            crate::log!("自测失败：120 秒内未完成，请检查权限和图形登录会话");
            self.finish(1);
            return;
        }
        if self.step.is_none() {
            if !active {
                return;
            }
            let center = crate::host::screen_center();
            self.origin = (center.0 - 100, center.1 - 100);
            self.original_cursor = crate::host::cursor_pos();
            self.step = Some(0);
            // 启用后等一轮 run loop，确保 tap source 先开始接收事件。
            self.next = now + Duration::from_millis(100);
            crate::log!("自测开始：右键 ↓→，随后普通右键单击");
        }
        if now < self.next {
            return;
        }
        let step = self.step.unwrap_or(0);
        if step == 64 {
            let matched = MATCHES.load(Ordering::Relaxed);
            let expected = EXPECTED_HIT.load(Ordering::Relaxed);
            let down = ECHO_DOWN.load(Ordering::Relaxed);
            let up = ECHO_UP.load(Ordering::Relaxed);
            crate::log!(
                "自测结果：手势命中次数={matched}，预期手势命中={expected}，点击已补发={}，带标记回声 down={down}, up={up}",
                down == 1 && up == 1
            );
            self.finish(if matched == 1 && expected && down == 1 && up == 1 {
                0
            } else {
                1
            });
            return;
        }
        let (kind, x, y) = test_event(step, self.origin);
        let Some(event) = crate::host::mouse_event(kind, 1, CGPoint::new(x as f64, y as f64), 0) else {
            crate::log!("自测失败：创建鼠标事件失败");
            self.finish(1);
            return;
        };
        // 从 HID 位置投递，确保下游 Session tap 能看见；tag=0，不绕过自己的 tap。
        CGEvent::post(CGEventTapLocation::HIDEventTap, Some(&event));
        self.step = Some(step + 1);
        self.next = now
            + Duration::from_millis(match step {
                61 => 200,
                63 => 1000,
                _ => 10,
            });
    }

    fn finish(&mut self, code: i32) {
        self.done = true;
        ENABLED.store(false, Ordering::Relaxed);
        if code != 0 && self.step.is_some_and(|step| (1..=63).contains(&step)) {
            // 创建事件失败或超时时，尽力释放合成右键，避免留下按下状态。
            if let Some(pos) = crate::host::cursor_pos() {
                if let Some(event) = crate::host::mouse_event(Kind::RightMouseUp, 1, pos, crate::events::INJECT_TAG) {
                    CGEvent::post(CGEventTapLocation::HIDEventTap, Some(&event));
                }
            }
        }
        if let Some(pos) = self.original_cursor {
            if let Some(event) = crate::host::mouse_event(Kind::MouseMoved, 0, pos, crate::events::INJECT_TAG) {
                CGEvent::post(CGEventTapLocation::HIDEventTap, Some(&event));
            }
        }
        crate::app::request_quit(code);
    }
}

fn test_event(step: u32, (x, y): (i32, i32)) -> (Kind, i32, i32) {
    match step {
        0 => (Kind::RightMouseDown, x, y),
        1..=30 => (Kind::RightMouseDragged, x, y + step as i32 * 200 / 30),
        31..=60 => (Kind::RightMouseDragged, x + (step - 30) as i32 * 200 / 30, y + 200),
        61 | 63 => (Kind::RightMouseUp, x + 200, y + 200),
        _ => (Kind::RightMouseDown, x + 200, y + 200),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selftest_draws_down_then_right_and_clicks_at_the_endpoint() {
        assert_eq!(test_event(0, (10, 20)), (Kind::RightMouseDown, 10, 20));
        assert_eq!(test_event(30, (10, 20)), (Kind::RightMouseDragged, 10, 220));
        assert_eq!(test_event(60, (10, 20)), (Kind::RightMouseDragged, 210, 220));
        assert_eq!(test_event(61, (10, 20)), (Kind::RightMouseUp, 210, 220));
        assert_eq!(test_event(62, (10, 20)), (Kind::RightMouseDown, 210, 220));
        assert_eq!(test_event(63, (10, 20)), (Kind::RightMouseUp, 210, 220));
    }
}
