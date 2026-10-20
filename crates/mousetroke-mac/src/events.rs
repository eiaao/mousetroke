//! Session tap：只做有界转换和引擎调用；补发由主线程下一次计时器处理。
//! 普通移动走只监听的 tap（系统不等待回调，主线程忙时光标也不卡）；
//! 按键、滚轮、拖动走可拦截的 tap，保证拖动与按下/松开的顺序。
use std::cell::{Cell, RefCell};
use std::ffi::c_void;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr::{self, NonNull};
use std::time::{Duration, Instant};

use mousetroke_core::{keymap::kvk_to_vk, schema::MouseButton};
use mousetroke_engine::{InputEvent, Verdict};
use objc2_core_foundation::{kCFRunLoopCommonModes, CFMachPort, CFRetained, CFRunLoop, CFRunLoopSource};
use objc2_core_graphics::{
    CGEvent, CGEventField as Field, CGEventSource, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventTapProxy, CGEventType as Kind,
};

pub const INJECT_TAG: i64 = 0x4D53_5452;
thread_local! {
    static TAPS: RefCell<Option<[Tap; 2]>> = const { RefCell::new(None) };
    static HOTKEY_HELD: Cell<bool> = const { Cell::new(false) };
    /// 暂停热键：(kVK, 必须恰好按下的修饰键位)；来自 settings.pause_hotkey。
    static PAUSE_KEY: Cell<Option<(u16, u64)>> = const { Cell::new(None) };
    static FAULTED: Cell<bool> = const { Cell::new(false) };
    static SCROLL: Cell<f64> = const { Cell::new(0.0) };
    static EDGE_SETTLE: Cell<Option<(i32, i32, Instant)>> = const { Cell::new(None) };
}

struct Tap {
    port: CFRetained<CFMachPort>,
    source: CFRetained<CFRunLoopSource>,
}
impl Drop for Tap {
    fn drop(&mut self) {
        CGEvent::tap_enable(&self.port, false);
        self.source.invalidate();
        self.port.invalidate();
    }
}

/// 调试开关 MOUSETROKE_TRACE_EVENTS=1：记录 tap 收到的每个鼠标按下/抬起（启动时读一次）。
fn trace_events() -> bool {
    static TRACE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *TRACE.get_or_init(|| std::env::var_os("MOUSETROKE_TRACE_EVENTS").is_some_and(|v| v == "1"))
}

const FILTERED: &[Kind] = &[
    Kind::LeftMouseDown,
    Kind::LeftMouseUp,
    Kind::LeftMouseDragged,
    Kind::RightMouseDown,
    Kind::RightMouseUp,
    Kind::RightMouseDragged,
    Kind::OtherMouseDown,
    Kind::OtherMouseUp,
    Kind::OtherMouseDragged,
    Kind::ScrollWheel,
    Kind::KeyDown,
    Kind::KeyUp,
    Kind::FlagsChanged,
];
/// 引擎对移动一律放行，无需同步等待。
const LISTENED: &[Kind] = &[Kind::MouseMoved];

pub fn install() -> bool {
    if active() {
        return true;
    }
    remove();
    let (Some(filter), Some(listen)) = (
        create(CGEventTapOptions::Default, FILTERED),
        create(CGEventTapOptions::ListenOnly, LISTENED),
    ) else {
        return false;
    };
    TAPS.with(|slot| *slot.borrow_mut() = Some([filter, listen]));
    crate::log!("事件 tap 已启用（主 run loop）");
    true
}

fn create(options: CGEventTapOptions, kinds: &[Kind]) -> Option<Tap> {
    let mask = kinds.iter().fold(0, |mask, kind| mask | (1u64 << kind.0));
    // 安全：回调捕获所有 Rust unwind；user_info 不使用；source 只加入主 run loop。
    let port = unsafe {
        CGEvent::tap_create(
            CGEventTapLocation::SessionEventTap,
            CGEventTapPlacement::HeadInsertEventTap,
            options,
            mask,
            Some(callback),
            ptr::null_mut(),
        )
    }?;
    let Some(source) = CFMachPort::new_run_loop_source(None, Some(&port), 0) else {
        port.invalidate();
        return None;
    };
    let Some(run_loop) = CFRunLoop::main() else {
        port.invalidate();
        return None;
    };
    // 安全：系统提供的 common modes 常量始终有效；菜单跟踪期间也要处理 tap。
    let Some(mode) = (unsafe { kCFRunLoopCommonModes }) else {
        port.invalidate();
        return None;
    };
    run_loop.add_source(Some(&source), Some(mode));
    CGEvent::tap_enable(&port, true);
    let tap = Tap { port, source };
    (run_loop.contains_source(Some(&tap.source), Some(mode)) && CGEvent::tap_is_enabled(&tap.port))
        .then_some(tap)
}

pub fn active() -> bool {
    TAPS.with(|slot| {
        slot.borrow().as_ref().is_some_and(|taps| {
            taps.iter()
                .all(|tap| tap.port.is_valid() && CGEvent::tap_is_enabled(&tap.port))
        })
    })
}
pub fn remove() {
    EDGE_SETTLE.with(|slot| slot.set(None));
    TAPS.with(|slot| {
        slot.borrow_mut().take();
    });
}
pub fn take_fault() -> bool {
    FAULTED.with(|flag| flag.replace(false))
}

pub fn is_tagged(tag: i64) -> bool {
    tag == INJECT_TAG
}

fn reenable(kind: Kind) {
    TAPS.with(|slot| {
        if let Ok(slot) = slot.try_borrow() {
            for tap in slot.iter().flatten() {
                if !CGEvent::tap_is_enabled(&tap.port) {
                    CGEvent::tap_enable(&tap.port, true);
                    let enabled = CGEvent::tap_is_enabled(&tap.port);
                    crate::log!("事件 tap 被系统禁用: {}，重新启用: {enabled}", kind.0);
                }
            }
        }
    });
}

unsafe extern "C-unwind" fn callback(
    _proxy: CGEventTapProxy,
    kind: Kind,
    event: NonNull<CGEvent>,
    _info: *mut c_void,
) -> *mut CGEvent {
    // 安全：系统在回调期间借出 event；禁用通知先处理，不读取其事件字段。
    let result = catch_unwind(AssertUnwindSafe(|| {
        if kind == Kind::TapDisabledByTimeout || kind == Kind::TapDisabledByUserInput {
            reenable(kind);
            return event.as_ptr();
        }
        crate::app::wake();
        let event_ref = unsafe { event.as_ref() };
        if is_tagged(CGEvent::integer_value_field(
            Some(event_ref),
            Field::EventSourceUserData,
        )) {
            crate::selftest::tagged_echo(kind);
            return event.as_ptr();
        }
        if FAULTED.with(Cell::get) {
            return event.as_ptr();
        }
        let key = CGEvent::integer_value_field(Some(event_ref), Field::KeyboardEventKeycode) as u16;
        let pause = PAUSE_KEY.with(Cell::get).filter(|(code, _)| *code == key);
        if kind == Kind::KeyUp && pause.is_some() && HOTKEY_HELD.with(|held| held.replace(false)) {
            return ptr::null_mut();
        }
        let flags = CGEvent::flags(Some(event_ref)).bits();
        if kind == Kind::KeyDown
            && pause.is_some_and(|(_, mods)| HOTKEY_HELD.with(Cell::get) || pause_chord(flags, mods))
        {
            if !HOTKEY_HELD.with(|held| held.replace(true)) {
                crate::app::toggle_pause();
            }
            return ptr::null_mut();
        }
        let pos = CGEvent::location(Some(event_ref));
        let button = CGEvent::integer_value_field(Some(event_ref), Field::MouseEventButtonNumber);
        if kind == Kind::MouseMoved {
            crate::selftest::moved();
            EDGE_SETTLE.with(|slot| slot.set(Some((pos.x as i32, pos.y as i32, Instant::now()))));
        }
        if trace_events() && !matches!(kind, Kind::ScrollWheel | Kind::KeyDown | Kind::KeyUp | Kind::FlagsChanged) {
            crate::log!("tap 事件: {kind:?} 按钮={button} ({:.0},{:.0})", pos.x, pos.y);
        }
        let wheel = if kind == Kind::ScrollWheel {
            let continuous = CGEvent::integer_value_field(Some(event_ref), Field::ScrollWheelEventIsContinuous) != 0;
            // 连续滚动的 fixed-point 数据以像素计；从该事件的 source 读取每行像素数。
            let pixels_per_line = if continuous {
                CGEvent::new_source_from_event(Some(event_ref))
                    .map(|source| CGEventSource::pixels_per_line(Some(&source)))
                    .unwrap_or(0.0)
            } else {
                0.0
            };
            SCROLL.with(|remainder| {
                let mut rest = remainder.get();
                let delta = scroll_lines(
                    CGEvent::integer_value_field(Some(event_ref), Field::ScrollWheelEventDeltaAxis1),
                    CGEvent::integer_value_field(Some(event_ref), Field::ScrollWheelEventFixedPtDeltaAxis1),
                    continuous,
                    pixels_per_line,
                    &mut rest,
                );
                remainder.set(rest);
                delta
            })
        } else {
            0
        };
        let input = map_event(kind, button, key, flags, pos.x as i32, pos.y as i32, wheel);
        if input.and_then(|input| crate::app::with_engine(|engine| engine.handle(input))) == Some(Verdict::Swallow) {
            ptr::null_mut()
        } else {
            event.as_ptr()
        }
    }));
    result.unwrap_or_else(|_| {
        // 失败后先放行；下一次主线程计时器取消捕获并暂停，不让 unwind 越过 FFI。
        FAULTED.with(|flag| flag.set(true));
        crate::log!("事件回调发生 panic，已放行并请求暂停");
        event.as_ptr()
    })
}

/// 撞到物理边界后可能不再有移动事件；补一次驻留检查，不投递系统事件或重复执行动作。
pub fn settle_edges(now: Instant) {
    EDGE_SETTLE.with(|slot| {
        let Some((x, y, since)) = slot.get() else { return };
        if now.saturating_duration_since(since) >= Duration::from_millis(40) {
            slot.set(None);
            if active() && !FAULTED.with(Cell::get) && crate::host::cursor_pos()
                .is_some_and(|p| (p.x as i32, p.y as i32) == (x, y)) {
                crate::app::with_engine(|engine| engine.settle_edges(x, y));
            }
        }
    });
}

const SHIFT: u64 = 1 << 17;
const CONTROL: u64 = 1 << 18;
const OPTION: u64 = 1 << 19;
const COMMAND: u64 = 1 << 20;

/// 修饰键必须恰好一致（多按 Shift 不算），与 Windows RegisterHotKey 的语义相同。
pub fn settle_pending() -> bool {
    EDGE_SETTLE.with(|slot| slot.get().is_some())
}

pub fn pause_chord(flags: u64, required: u64) -> bool {
    flags & (SHIFT | CONTROL | OPTION | COMMAND) == required
}

/// 解析设置里的暂停热键（Win 对应 ⌘）；无法映射到 Mac 按键时禁用热键。
pub fn hotkey(text: &str) -> Option<(u16, u64)> {
    use mousetroke_core::keyseq::ModKey;
    let (mods, vk) = mousetroke_core::keyseq::parse_hotkey(text)?;
    let key = mousetroke_core::keymap::vk_to_kvk(vk)?;
    let mask = mods.iter().fold(0, |mask, m| {
        mask | match m {
            ModKey::Shift => SHIFT,
            ModKey::Ctrl => CONTROL,
            ModKey::Alt => OPTION,
            ModKey::Win => COMMAND,
        }
    });
    Some((key, mask))
}

pub fn set_pause_hotkey(text: &str) {
    let parsed = hotkey(text);
    if parsed.is_none() && !text.is_empty() {
        crate::log!("暂停热键无法解析: {text:?}");
    }
    PAUSE_KEY.with(|slot| slot.set(parsed));
    HOTKEY_HELD.with(|held| held.set(false));
}

/// Apple 正值向上；16.16 像素先按 source 的 pixels-per-line 换成行，再累计小数。
pub fn scroll_lines(discrete: i64, fixed: i64, continuous: bool, pixels_per_line: f64, remainder: &mut f64) -> i32 {
    let precise = continuous && pixels_per_line.is_finite() && pixels_per_line > 0.0 && fixed != 0;
    let lines = if precise {
        fixed as f64 / 65536.0 / pixels_per_line
    } else {
        discrete as f64
    };
    let total = -lines + if precise { *remainder } else { 0.0 };
    let result = total.clamp(i32::MIN as f64, i32::MAX as f64).trunc() as i32;
    *remainder = if precise { total - total.trunc() } else { 0.0 };
    result
}

fn modifier_pressed(key: u16, flags: u64) -> bool {
    // 设备相关位来自 IOKit/hidsystem/IOLLEvent.h，保留同时按住左右修饰键时的抬起语义。
    let bit = match key {
        0x38 => 0x2,
        0x3C => 0x4,
        0x3B => 0x1,
        0x3E => 0x2000,
        0x3A => 0x20,
        0x3D => 0x40,
        0x37 => 0x8,
        0x36 => 0x10,
        0x39 => 1 << 16,
        _ => return false,
    };
    flags & bit != 0
}

pub fn map_event(kind: Kind, number: i64, key: u16, flags: u64, x: i32, y: i32, wheel: i32) -> Option<InputEvent> {
    let button = match kind {
        Kind::LeftMouseDown | Kind::LeftMouseUp => Some(MouseButton::Left),
        Kind::RightMouseDown | Kind::RightMouseUp => Some(MouseButton::Right),
        Kind::OtherMouseDown | Kind::OtherMouseUp => match number {
            2 => Some(MouseButton::Middle),
            3 => Some(MouseButton::X1),
            4 => Some(MouseButton::X2),
            _ => None,
        },
        _ => None,
    };
    match kind {
        Kind::LeftMouseDown | Kind::RightMouseDown | Kind::OtherMouseDown => {
            button.map(|btn| InputEvent::MouseDown { btn, x, y })
        }
        Kind::LeftMouseUp | Kind::RightMouseUp | Kind::OtherMouseUp => {
            button.map(|btn| InputEvent::MouseUp { btn, x, y })
        }
        Kind::MouseMoved | Kind::LeftMouseDragged | Kind::RightMouseDragged | Kind::OtherMouseDragged => {
            Some(InputEvent::MouseMove { x, y })
        }
        Kind::ScrollWheel if wheel != 0 => Some(InputEvent::Wheel(wheel)),
        Kind::KeyDown => kvk_to_vk(key).map(InputEvent::KeyDown),
        Kind::FlagsChanged if modifier_pressed(key, flags) => kvk_to_vk(key).map(InputEvent::KeyDown),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn buttons_drags_and_keys_map_without_unknown_buttons() {
        for (number, expected) in [(2, MouseButton::Middle), (3, MouseButton::X1), (4, MouseButton::X2)] {
            assert!(matches!(map_event(Kind::OtherMouseDown, number, 0, 0, -12, 34, 0),
                Some(InputEvent::MouseDown { btn, x: -12, y: 34 }) if btn == expected));
            assert!(matches!(map_event(Kind::OtherMouseUp, number, 0, 0, 0, 0, 0),
                Some(InputEvent::MouseUp { btn, .. }) if btn == expected));
        }
        assert!(map_event(Kind::OtherMouseDown, 5, 0, 0, 0, 0, 0).is_none());
        assert!(matches!(
            map_event(Kind::RightMouseDragged, 1, 0, 0, 10, 20, 0),
            Some(InputEvent::MouseMove { x: 10, y: 20 })
        ));
        assert!(matches!(
            map_event(Kind::KeyDown, 0, 0x23, 0, 0, 0, 0),
            Some(InputEvent::KeyDown(0x50))
        ));
        assert!(matches!(
            map_event(Kind::FlagsChanged, 0, 0x38, 0x2, 0, 0, 0),
            Some(InputEvent::KeyDown(0x10))
        ));
        assert!(map_event(Kind::FlagsChanged, 0, 0x38, 0x4 | (1 << 17), 0, 0, 0).is_none());
    }
    #[test]
    fn scroll_converts_lines_and_accumulates_fractional_lines() {
        let mut rest = 0.0;
        assert_eq!(scroll_lines(3, 0, false, 10.0, &mut rest), -3);
        assert_eq!(scroll_lines(-2, 0, false, 10.0, &mut rest), 2);
        for _ in 0..3 {
            assert_eq!(scroll_lines(0, 163840, true, 10.0, &mut rest), 0);
        }
        assert_eq!(scroll_lines(0, 163840, true, 10.0, &mut rest), -1);
        assert_eq!(scroll_lines(i64::MIN, 0, false, 10.0, &mut rest), i32::MAX);
        assert_eq!(scroll_lines(0, 0, true, 10.0, &mut rest), 0);
        assert_eq!(scroll_lines(-2, 163840, true, 0.0, &mut rest), 2);
        assert_eq!(scroll_lines(-2, 163840, true, f64::NAN, &mut rest), 2);
    }
    #[test]
    fn tag_is_exact_and_hotkey_excludes_shift() {
        assert!(is_tagged(INJECT_TAG));
        assert!(!is_tagged(INJECT_TAG + 1));
        assert!(!is_tagged(0));
        let (key, chord) = hotkey("^%#p").unwrap();
        assert_eq!((key, chord), (0x23, CONTROL | OPTION | COMMAND));
        assert!(pause_chord(chord, chord));
        assert!(!pause_chord(chord | SHIFT, chord));
        assert!(!pause_chord(chord & !CONTROL, chord));
        assert_eq!(hotkey("+{F5}").map(|(_, m)| m), Some(SHIFT));
        assert_eq!(hotkey(""), None);
    }
}
