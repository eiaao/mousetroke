//! 动作只在下一轮主线程计时器执行；延时和重复均分片推进，不阻塞 tap 或 run loop。
use std::cell::RefCell;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

use mousetroke_core::{
    keymap::vk_to_kvk,
    keyseq::{self, Action as Step, Key, ModKey},
    schema::{Action, SendMode},
};
use objc2_core_foundation::CFRetained;
use objc2_core_graphics::{CGEvent, CGEventField, CGEventFlags as Flags, CGEventTapLocation};

const CAPACITY: usize = 64;
const MAX_KEYS_BYTES: usize = 16384;
const TICK_BUDGET: usize = 32;
thread_local! {
    static EXECUTOR: RefCell<Executor> = RefCell::new(Executor::default());
}

#[derive(Default)]
struct Executor {
    queue: VecDeque<Queued>,
    active: Option<Job>,
    dry_run: bool,
}

enum Queued { Action(Action), Parsed(keyseq::KeySeq) }

struct Repeat {
    vk: u16,
    mods: Vec<ModKey>,
    remaining: u32,
}

struct Job {
    steps: std::vec::IntoIter<Step>,
    repeat: Option<Repeat>,
    text: std::vec::IntoIter<char>,
    resume: Instant,
    warned_unsupported: bool,
}

pub fn initialize(dry_run: bool) {
    crate::actions::initialize(dry_run);
    EXECUTOR.with(|slot| {
        *slot.borrow_mut() = Executor {
            dry_run,
            ..Executor::default()
        }
    });
}

pub fn enqueue(action: Action) {
    EXECUTOR.with(|slot| {
        let mut executor = slot.borrow_mut();
        if executor.queue.len() >= CAPACITY {
            crate::log!("动作队列已满，丢弃本次动作");
            return;
        }
        if matches!(&action, Action::KeySequence { keys, .. } if keys.len() > MAX_KEYS_BYTES) {
            crate::log!("按键序列过长，跳过动作");
            return;
        }
        executor.queue.push_back(Queued::Action(action));
    });
}

/// Lua 已按 text_by_default 解析，不能再经过字符串重解析丢失文本语义。
pub fn enqueue_parsed(sequence: keyseq::KeySeq) {
    EXECUTOR.with(|slot| {
        let mut executor = slot.borrow_mut();
        if executor.queue.len() >= CAPACITY || sequence.actions.len() > MAX_KEYS_BYTES
            || sequence.actions.iter().any(|s| matches!(s, Step::Text(t) if t.len() > MAX_KEYS_BYTES)) {
            crate::log!("Lua 按键队列已满或序列过长，跳过调用");
            return;
        }
        executor.queue.push_back(Queued::Parsed(sequence));
    });
}

/// 此函数仅由 AppKit 主线程计时器调用，不能从事件 tap 调用。
pub fn tick() {
    EXECUTOR.with(|slot| slot.borrow_mut().tick(Instant::now()));
}

pub fn busy() -> bool {
    EXECUTOR.with(|s| {
        let s = s.borrow();
        s.active.is_some() || !s.queue.is_empty()
    })
}

impl Executor {
    fn tick(&mut self, now: Instant) {
        for _ in 0..TICK_BUDGET {
            if self.active.is_none() {
                let Some(action) = self.queue.pop_front() else {
                    break;
                };
                self.active = match action {
                    Queued::Action(action) => self.prepare(action, now),
                    Queued::Parsed(sequence) => Some(Self::job(sequence, now)),
                };
                continue;
            }
            let job = self.active.as_mut().unwrap();
            if now < job.resume {
                break;
            }
            if let Some(repeat) = job.repeat.as_mut() {
                if !post_chord(repeat.vk, &repeat.mods, self.dry_run) {
                    crate::log!("创建按键事件失败，取消本次序列");
                    self.active = None;
                    continue;
                }
                repeat.remaining -= 1;
                if repeat.remaining == 0 {
                    job.repeat = None;
                }
                continue;
            }
            if let Some(ch) = job.text.next() {
                if !post_text(ch, self.dry_run) {
                    crate::log!("创建文字事件失败，取消本次序列");
                    self.active = None;
                }
                continue;
            }
            match job.steps.next() {
                Some(Step::SleepMs(ms)) => job.resume = now + Duration::from_millis(ms as u64),
                Some(Step::Text(text)) => job.text = text.chars().collect::<Vec<_>>().into_iter(),
                Some(Step::Key {
                    key: Key::Vk(vk),
                    mods,
                    repeat,
                }) if vk_to_kvk(vk).is_some() => {
                    if repeat > 0 {
                        job.repeat = Some(Repeat {
                            vk,
                            mods,
                            remaining: repeat,
                        });
                    }
                }
                Some(Step::Key { .. }) => {
                    if !job.warned_unsupported {
                        crate::log!("按键序列含未支持的项目，已跳过（不记录内容）");
                        job.warned_unsupported = true;
                    }
                }
                None => self.active = None,
            }
        }
    }

    fn prepare(&mut self, action: Action, now: Instant) -> Option<Job> {
        let Action::KeySequence { keys, mode, .. } = action else {
            crate::log!("非按键动作误入按键队列，已跳过");
            return None;
        };
        if mode != SendMode::DownUp {
            crate::log!("macOS 暂不支持此按键模式，已跳过（不记录内容）");
            return None;
        }
        let Ok(sequence) = keyseq::parse(&keys, false) else {
            crate::log!("按键序列解析失败，已跳过（不记录内容）");
            return None;
        };
        crate::selftest::key_sequence_dispatched(&keys, self.dry_run);
        crate::log!("按键动作已派发: dry_run={}", self.dry_run);
        Some(Self::job(sequence, now))
    }

    fn job(sequence: keyseq::KeySeq, now: Instant) -> Job {
        Job {
            steps: sequence.actions.into_iter(),
            repeat: None,
            text: Vec::new().into_iter(),
            resume: now,
            warned_unsupported: false,
        }
    }
}

fn modifier(m: ModKey) -> (u16, Flags) {
    match m {
        ModKey::Ctrl => (0x3B, Flags::MaskControl),
        ModKey::Shift => (0x38, Flags::MaskShift),
        ModKey::Alt => (0x3A, Flags::MaskAlternate),
        ModKey::Win => (0x37, Flags::MaskCommand),
    }
}

fn key_modifier(vk: u16) -> Flags {
    match vk {
        0x10 | 0xA0 | 0xA1 => Flags::MaskShift,
        0x11 | 0xA2 | 0xA3 => Flags::MaskControl,
        0x12 | 0xA4 | 0xA5 => Flags::MaskAlternate,
        0x5B | 0x5C => Flags::MaskCommand,
        0x14 => Flags::MaskAlphaShift,
        _ => Flags::empty(),
    }
}

/// 真实键盘上 F 键与导航键带 fn 位、方向键与小键盘另带 NumericPad 位；
/// 缺这些位时系统快捷键（如 ⌃↑ 调度中心、F11 显示桌面）不会匹配。
fn hardware_flags(vk: u16) -> Flags {
    match vk {
        0x25..=0x28 => Flags::MaskSecondaryFn | Flags::MaskNumericPad,
        0x70..=0x83 | 0x21..=0x24 | 0x2E | 0x2F => Flags::MaskSecondaryFn,
        0x60..=0x6F | 0x0C | 0x90 | 0x92 => Flags::MaskNumericPad,
        _ => Flags::empty(),
    }
}

/// 修饰键按下累加 flags，按相反顺序抬起时先去掉相应位；两侧通用键不重复注入。
fn chord(vk: u16, mods: &[ModKey]) -> Option<Vec<(u16, bool, Flags)>> {
    let key = vk_to_kvk(vk)?;
    let mut result = Vec::with_capacity(10);
    let mut held = Vec::with_capacity(4);
    let mut flags = Flags::empty();
    for m in mods {
        let (code, flag) = modifier(*m);
        if !flags.contains(flag) && flag != key_modifier(vk) {
            flags.insert(flag);
            held.push((code, flag));
            result.push((code, true, flags));
        }
    }
    result.push((key, true, flags | key_modifier(vk) | hardware_flags(vk)));
    result.push((key, false, flags | hardware_flags(vk)));
    for (code, flag) in held.into_iter().rev() {
        flags.remove(flag);
        result.push((code, false, flags));
    }
    Some(result)
}

fn event(code: u16, down: bool, flags: Flags) -> Option<CFRetained<CGEvent>> {
    let event = CGEvent::new_keyboard_event(None, code, down)?;
    CGEvent::set_flags(Some(&event), flags);
    CGEvent::set_integer_value_field(
        Some(&event),
        CGEventField::EventSourceUserData,
        crate::events::INJECT_TAG,
    );
    Some(event)
}

fn post_chord(vk: u16, mods: &[ModKey], dry_run: bool) -> bool {
    let Some(chord) = chord(vk, mods) else {
        return false;
    };
    if dry_run {
        crate::log!(
            "自测按键：将投递 {} 件带标记的 HID 事件（未实际投递）",
            chord.len()
        );
        return true;
    }
    // 先创建完整的按下/释放组，任何分配失败时都不投递，避免修饰键卡住。
    let Some(events) = chord
        .into_iter()
        .map(|(code, down, flags)| event(code, down, flags))
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    for event in events {
        CGEvent::post(CGEventTapLocation::HIDEventTap, Some(&event));
    }
    true
}

/// 仅由异步选区状态机调用，沿用普通按键的注入标记与完整释放组。
pub fn copy_selection() -> bool {
    post_chord(0x43, &[ModKey::Win], false)
}

fn post_text(ch: char, dry_run: bool) -> bool {
    if dry_run {
        crate::log!("自测文字：将投递一对 Unicode HID 事件（未实际投递）");
        return true;
    }
    let Some(events) = unicode_events(ch) else {
        return false;
    };
    for event in events {
        CGEvent::post(CGEventTapLocation::HIDEventTap, Some(&event));
    }
    true
}

fn unicode_events(ch: char) -> Option<[CFRetained<CGEvent>; 2]> {
    let events = [
        event(0, true, Flags::empty())?,
        event(0, false, Flags::empty())?,
    ];
    let mut buffer = [0u16; 2];
    let units = ch.encode_utf16(&mut buffer);
    for event in &events {
        // 安全：UTF-16 切片包含完整标量（含代理对），调用期间指针有效，长度以 u16 为单位。
        unsafe {
            CGEvent::keyboard_set_unicode_string(Some(event), units.len() as _, units.as_ptr())
        };
    }
    Some(events)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enqueue_defers_parsing_and_dispatch_to_tick() {
        initialize(true);
        enqueue(Action::KeySequence {
            keys: "#w".into(),
            mode: SendMode::DownUp,
            secured: false,
            activate_target: false,
        });
        EXECUTOR.with(|slot| {
            let executor = slot.borrow();
            assert_eq!(executor.queue.len(), 1);
            assert!(executor.active.is_none());
        });
        tick();
        EXECUTOR.with(|slot| {
            let executor = slot.borrow();
            assert!(executor.queue.is_empty() && executor.active.is_none());
        });
    }

    #[test]
    fn keyboard_and_unicode_events_keep_flags_tag_and_surrogate_pairs() {
        let key = event(0x0D, true, Flags::MaskCommand).unwrap();
        assert_eq!(CGEvent::flags(Some(&key)), Flags::MaskCommand);
        for event in std::iter::once(key).chain(unicode_events('😀').unwrap()) {
            assert_eq!(
                CGEvent::integer_value_field(Some(&event), CGEventField::EventSourceUserData),
                crate::events::INJECT_TAG
            );
        }
        for event in unicode_events('😀').unwrap() {
            let mut buffer = [0u16; 2];
            let mut length = 0;
            // 安全：两个 UTF-16 单元的输出缓冲和长度指针在调用期间有效，不投递事件。
            unsafe {
                CGEvent::keyboard_get_unicode_string(
                    Some(&event),
                    2,
                    &mut length,
                    buffer.as_mut_ptr(),
                )
            };
            assert_eq!(length, 2);
            assert_eq!(String::from_utf16(&buffer).unwrap(), "😀");
        }
    }

    #[test]
    fn chords_use_mac_flags_and_release_modifiers_in_reverse_order() {
        let steps = chord(0x57, &[ModKey::Win, ModKey::Shift, ModKey::Win]).unwrap();
        let both = Flags::MaskCommand | Flags::MaskShift;
        assert_eq!(
            steps,
            vec![
                (0x37, true, Flags::MaskCommand),
                (0x38, true, both),
                (0x0D, true, both),
                (0x0D, false, both),
                (0x38, false, Flags::MaskCommand),
                (0x37, false, Flags::empty()),
            ]
        );
        assert_eq!(modifier(ModKey::Ctrl), (0x3B, Flags::MaskControl));
        let up = chord(0x26, &[ModKey::Ctrl]).unwrap();
        let arrow = Flags::MaskControl | Flags::MaskSecondaryFn | Flags::MaskNumericPad;
        assert_eq!(up[1], (0x7E, true, arrow));
        assert_eq!(up[2], (0x7E, false, arrow));
        assert_eq!(chord(0x7A, &[]).unwrap()[0], (0x67, true, Flags::MaskSecondaryFn));
        assert_eq!(modifier(ModKey::Alt), (0x3A, Flags::MaskAlternate));
        assert!(chord(0xB3, &[]).is_none());
        assert_eq!(chord(0x11, &[ModKey::Ctrl]).unwrap().len(), 2);
    }

    #[test]
    fn delays_and_large_repeats_yield_to_the_run_loop() {
        let mut executor = Executor {
            dry_run: true,
            ..Executor::default()
        };
        let now = Instant::now();
        executor.active = Some(Job {
            steps: vec![
                Step::SleepMs(100),
                Step::Key {
                    key: Key::Vk(0x41),
                    mods: vec![],
                    repeat: u32::MAX,
                },
                Step::Text("你好😀".into()),
            ]
            .into_iter(),
            repeat: None,
            text: Vec::new().into_iter(),
            resume: now,
            warned_unsupported: false,
        });
        executor.tick(now);
        assert_eq!(
            executor.active.as_ref().unwrap().resume,
            now + Duration::from_millis(100)
        );
        assert!(executor.active.as_ref().unwrap().repeat.is_none());
        executor.tick(now + Duration::from_millis(100));
        let remaining = executor
            .active
            .as_ref()
            .unwrap()
            .repeat
            .as_ref()
            .unwrap()
            .remaining;
        assert_eq!(remaining, u32::MAX - (TICK_BUDGET as u32 - 1));
    }
}
