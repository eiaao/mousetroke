//! 按键序列语法解析（AutoIt/SendKeys 风格，兼容旧格式配置里的写法）。
//!
//! 语法：
//! - 修饰符：`^`Ctrl  `%`Alt  `+`Shift  `#`Win，作用于下一个键或 `(组)`
//! - 特殊键：`{NAME}`、`{NAME n}`（重复 n 次）、`{SLEEP ms}`（延时）
//! - 鼠标：`{MOUSE_LEFT}` … `{MOUSE_X2}`、`{MOUSE_WHEEL -120}`、`{MOUSE_HWHEEL 120}`
//! - 普通字符：默认视为按键；文本优先模式下视为文本输出，按键须写成 `{a}`
//!
//! 解析产出与执行解耦：注入由平台层（mousetroke-win，SendInput）完成。
//! 错误是 `keyseq.error.*` 词条键 + 参数（由界面翻译，见 docs/I18N-DESIGN.md §5）。

use crate::i18n::keys;

/// 修饰键。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModKey {
    Ctrl,
    Alt,
    Shift,
    Win,
}

/// 鼠标键/滚轮动作。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseKey {
    Left,
    Right,
    Middle,
    X1,
    X2,
    /// 滚轮（正=向下滚，负=向上滚，单位：格×120）。
    Wheel(i32),
    HWheel(i32),
}

/// 一个待注入的键。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    /// 虚拟键码（'A'=0x41、'0'=0x30 …）。
    Vk(u16),
    Mouse(MouseKey),
}

/// 序列中的一个动作。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Key {
        key: Key,
        mods: Vec<ModKey>,
        repeat: u32,
    },
    /// `{SLEEP ms}`。
    SleepMs(u32),
    /// 文本输出（文本优先模式下的普通字符）。
    Text(String),
}

/// 解析产物：有序动作序列。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KeySeq {
    pub actions: Vec<Action>,
}

/// 解析错误（含源串位置）。`key` 是 `keyseq.error.*` 词条键，`params` 是词条参数；
/// 由调用方（设置界面后端）转成 `Msg` 交给前端翻译。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeySeqError {
    /// 出错位置（0 起的字符下标；显示时 +1）。
    pub pos: usize,
    pub key: &'static str,
    pub params: Vec<(&'static str, String)>,
}

/// 从序列中提取一个全局热键 (修饰键, vk)——取第一个 Key 动作。
/// 用于暂停热键等（如 `#({ESC})` → ([Win], ESC)）。
pub fn parse_hotkey(input: &str) -> Option<(Vec<ModKey>, u16)> {
    let seq = parse(input, false).ok()?;
    for action in &seq.actions {
        if let Action::Key { key: Key::Vk(vk), mods, .. } = action {
            return Some((mods.clone(), *vk));
        }
    }
    None
}

/// 解析按键序列。`text_by_default` 对应旧格式 UseTextByDefaultSyntax。
pub fn parse(input: &str, text_by_default: bool) -> Result<KeySeq, KeySeqError> {
    let mut p = Parser {
        chars: input.chars().collect(),
        pos: 0,
    };
    let (actions, _) = p.parse_group(text_by_default)?;
    Ok(KeySeq { actions })
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    fn err(&self, key: &'static str, params: &[(&'static str, String)]) -> KeySeqError {
        KeySeqError { pos: self.pos, key, params: params.to_vec() }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    /// 解析一层序列，返回 (动作, 未消费的修饰符)。
    fn parse_group(
        &mut self,
        text_by_default: bool,
    ) -> Result<(Vec<Action>, Vec<ModKey>), KeySeqError> {
        let mut actions: Vec<Action> = Vec::new();
        let mut mods: Vec<ModKey> = Vec::new();
        while let Some(c) = self.peek() {
            match c {
                '^' => { self.bump(); mods.push(ModKey::Ctrl); }
                '%' => { self.bump(); mods.push(ModKey::Alt); }
                '+' => { self.bump(); mods.push(ModKey::Shift); }
                '#' => { self.bump(); mods.push(ModKey::Win); }
                '(' => {
                    self.bump();
                    let (inner, _) = self.parse_group(text_by_default)?;
                    if self.peek() != Some(')') {
                        return Err(self.err(keys::KEYSEQ_ERROR_MISSING_PAREN, &[]));
                    }
                    self.bump();
                    // 组内每个按键挂上外层修饰符：^(c) → Ctrl+C。
                    for action in inner {
                        actions.push(apply_mods(action, &mods));
                    }
                    mods.clear();
                }
                ')' => break, // 由上层消费
                '{' => {
                    self.bump();
                    let action = self.parse_special()?;
                    actions.push(apply_mods(action, &mods));
                    mods.clear();
                }
                ch => {
                    self.bump();
                    if text_by_default {
                        match actions.last_mut() {
                            Some(Action::Text(s)) => s.push(ch),
                            _ => actions.push(Action::Text(ch.to_string())),
                        }
                        mods.clear();
                    } else {
                        let mut m = std::mem::take(&mut mods);
                        actions.push(char_key(ch, &mut m, 1).map_err(|BadChar(c)| {
                            self.err(keys::KEYSEQ_ERROR_BAD_CHAR, &[("char", c.to_string())])
                        })?);
                    }
                }
            }
        }
        Ok((actions, mods))
    }

    /// 已消费 '{'，解析到匹配 '}'。
    fn parse_special(&mut self) -> Result<Action, KeySeqError> {
        let mut body = String::new();
        loop {
            match self.bump() {
                Some('}') => return self.resolve_special(&body),
                Some(c) => body.push(c),
                None => return Err(self.err(keys::KEYSEQ_ERROR_MISSING_BRACE, &[])),
            }
        }
    }

    fn resolve_special(&self, body: &str) -> Result<Action, KeySeqError> {
        let (name, arg): (&str, Option<i64>) = match body.split_once(' ') {
            Some((n, a)) => match a.trim().parse::<i64>() {
                Ok(v) => (n, Some(v)),
                Err(_) => {
                    return Err(self.err(keys::KEYSEQ_ERROR_BAD_ARG, &[("arg", a.trim().to_string())]));
                }
            },
            None => (body, None),
        };
        let upper = name.to_ascii_uppercase();

        if upper == "SLEEP" {
            return match arg {
                Some(ms) if ms >= 0 => Ok(Action::SleepMs(ms as u32)),
                _ => Err(self.err(keys::KEYSEQ_ERROR_SLEEP_NEEDS_MS, &[])),
            };
        }

        let key = match upper.as_str() {
            "MOUSE_LEFT" => Key::Mouse(MouseKey::Left),
            "MOUSE_RIGHT" => Key::Mouse(MouseKey::Right),
            "MOUSE_MIDDLE" => Key::Mouse(MouseKey::Middle),
            "MOUSE_X1" => Key::Mouse(MouseKey::X1),
            "MOUSE_X2" => Key::Mouse(MouseKey::X2),
            "MOUSE_WHEEL" | "MOUSE_HWHEEL" => {
                return match arg {
                    Some(delta) => Ok(Action::Key {
                        key: Key::Mouse(if upper == "MOUSE_WHEEL" {
                            MouseKey::Wheel(delta as i32)
                        } else {
                            MouseKey::HWheel(delta as i32)
                        }),
                        mods: vec![],
                        repeat: 1,
                    }),
                    None => Err(self.err(
                        keys::KEYSEQ_ERROR_WHEEL_NEEDS_ARG,
                        &[("name", upper.to_string())],
                    )),
                }
            }
            _ => match lookup_vk(&upper) {
                Some(vk) => Key::Vk(vk),
                None => {
                    if name.chars().count() == 1 {
                        let ch = name.chars().next().unwrap();
                        let repeat = arg.filter(|n| *n > 0).map_or(1, |n| n as u32);
                        return char_key(ch, &mut Vec::new(), repeat).map_err(|BadChar(c)| {
                            self.err(keys::KEYSEQ_ERROR_BAD_CHAR, &[("char", c.to_string())])
                        });
                    }
                    return Err(self.err(keys::KEYSEQ_ERROR_UNKNOWN_KEY, &[("name", name.to_string())]));
                }
            },
        };

        let repeat = match arg {
            Some(n) if n > 0 => n as u32,
            _ => 1,
        };
        Ok(Action::Key { key, mods: vec![], repeat })
    }
}

/// 单个字符 → 按键（美式键盘布局）：字母/数字直接映射，标点映射到 OEM 键码，
/// 需要 Shift 的符号（如 `?` `+`）自动追加 Shift。
struct BadChar(char);

fn char_key(ch: char, mods: &mut Vec<ModKey>, repeat: u32) -> Result<Action, BadChar> {
    const UNSHIFTED: &str = "`-=[]\\;',./";
    const SHIFTED: &str = "~_+{}|:\"<>?";
    const OEM: [u16; 11] = [0xC0, 0xBD, 0xBB, 0xDB, 0xDD, 0xDC, 0xBA, 0xDE, 0xBC, 0xBE, 0xBF];
    const SHIFT_DIGITS: &str = ")!@#$%^&*(";
    let (vk, shift) = if ch.is_ascii_alphanumeric() {
        (ch.to_ascii_uppercase() as u16, false)
    } else if let Some(vk) = match ch {
        ' ' => Some(0x20),
        '\t' => Some(0x09),
        '\n' | '\r' => Some(0x0D),
        '\u{1b}' => Some(0x1B),
        '\u{8}' => Some(0x08),
        _ => None,
    } {
        (vk, false)
    } else if let Some(i) = UNSHIFTED.find(ch) {
        (OEM[i], false)
    } else if let Some(i) = SHIFTED.find(ch) {
        (OEM[i], true)
    } else if let Some(i) = SHIFT_DIGITS.find(ch) {
        (b'0' as u16 + i as u16, true)
    } else {
        return Err(BadChar(ch));
    };
    if shift && !mods.contains(&ModKey::Shift) {
        mods.push(ModKey::Shift);
    }
    Ok(Action::Key { key: Key::Vk(vk), mods: std::mem::take(mods), repeat })
}

/// 将待应用的修饰符并入动作（仅对 Key 动作有效）。
fn apply_mods(action: Action, mods: &[ModKey]) -> Action {
    match action {
        Action::Key { key, mods: mut own, repeat } => {
            own.extend_from_slice(mods);
            Action::Key { key, mods: own, repeat }
        }
        other => other,
    }
}

/// 特殊键名 → 虚拟键码。
fn lookup_vk(name: &str) -> Option<u16> {
    let vk = match name {
        "ENTER" | "RETURN" => 0x0D,
        "ESC" => 0x1B,
        "TAB" => 0x09,
        "BACKSPACE" => 0x08,
        "SPACE" => 0x20,
        "CAPSLOCK" => 0x14,
        "INSERT" | "INS" => 0x2D,
        "DELETE" | "DEL" => 0x2E,
        "HOME" => 0x24,
        "END" => 0x23,
        "PGUP" => 0x21,
        "PGDN" => 0x22,
        "UP" => 0x26,
        "DOWN" => 0x28,
        "LEFT" => 0x25,
        "RIGHT" => 0x27,
        "PRTSC" => 0x2C,
        "BREAK" => 0x03,
        "MEDIA_NEXT" => 0xB0,
        "MEDIA_PREV" => 0xB1,
        "MEDIA_STOP" => 0xB2,
        "MEDIA_PLAY" | "MEDIA_PLAY_PAUSE" => 0xB3,
        "VOLUME_MUTE" => 0xAD,
        "VOLUME_DOWN" => 0xAE,
        "VOLUME_UP" => 0xAF,
        "CTRL" => 0x11,
        "ALT" => 0x12,
        "SHIFT" => 0x10,
        "LCTRL" => 0xA2,
        "RCTRL" => 0xA3,
        "LSHIFT" => 0xA0,
        "RSHIFT" => 0xA1,
        "LALT" => 0xA4,
        "RALT" => 0xA5,
        "LWIN" | "WIN" => 0x5B,
        "RWIN" => 0x5C,
        _ => {
            if let Some(rest) = name.strip_prefix('F') {
                if let Ok(n) = rest.parse::<u16>() {
                    if (1..=24).contains(&n) {
                        return Some(0x70 + n - 1);
                    }
                }
            }
            if let Some(rest) = name.strip_prefix('N') {
                if rest.len() == 1 {
                    if let Some(d) = rest.chars().next().unwrap().to_digit(10) {
                        return Some(0x60 + d as u16);
                    }
                }
            }
            return None;
        }
    };
    Some(vk)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(input: &str) -> Vec<Action> {
        parse(input, false).unwrap().actions
    }

    #[test]
    fn punctuation_keys() {
        // Ctrl+- 缩小：'-' 是 VK_OEM_MINUS，不是 0x2D(Insert)
        assert_eq!(keys("^-"), vec![Action::Key { key: Key::Vk(0xBD), mods: vec![ModKey::Ctrl], repeat: 1 }]);
        // {+} 是 Shift+'='
        assert_eq!(keys("^{+}"), vec![Action::Key { key: Key::Vk(0xBB), mods: vec![ModKey::Shift, ModKey::Ctrl], repeat: 1 }]);
        assert_eq!(keys("{a 3}"), vec![Action::Key { key: Key::Vk(b'A' as u16), mods: vec![], repeat: 3 }]);
    }

    #[test]
    fn ctrl_c() {
        assert_eq!(
            keys("^(c)"),
            vec![Action::Key { key: Key::Vk(b'C' as u16), mods: vec![ModKey::Ctrl], repeat: 1 }]
        );
    }

    #[test]
    fn plain_letters_and_sequence() {
        assert_eq!(
            keys("^v{ENTER}"),
            vec![
                Action::Key { key: Key::Vk(b'V' as u16), mods: vec![ModKey::Ctrl], repeat: 1 },
                Action::Key { key: Key::Vk(0x0D), mods: vec![], repeat: 1 },
            ]
        );
    }

    #[test]
    fn repeat_and_sleep() {
        assert_eq!(
            keys("+{F10}{SLEEP 50}e{SLEEP 50}{ENTER}"),
            vec![
                Action::Key { key: Key::Vk(0x79), mods: vec![ModKey::Shift], repeat: 1 },
                Action::SleepMs(50),
                Action::Key { key: Key::Vk(b'E' as u16), mods: vec![], repeat: 1 },
                Action::SleepMs(50),
                Action::Key { key: Key::Vk(0x0D), mods: vec![], repeat: 1 },
            ]
        );
    }

    #[test]
    fn volume_repeat_2() {
        assert_eq!(
            keys("{VOLUME_UP 2}"),
            vec![Action::Key { key: Key::Vk(0xAF), mods: vec![], repeat: 2 }]
        );
    }

    #[test]
    fn win_group_esc() {
        assert_eq!(
            keys("#({ESC})"),
            vec![Action::Key { key: Key::Vk(0x1B), mods: vec![ModKey::Win], repeat: 1 }]
        );
    }

    #[test]
    fn ctrl_click_mapping() {
        assert_eq!(
            keys("^({MOUSE_LEFT})"),
            vec![Action::Key {
                key: Key::Mouse(MouseKey::Left),
                mods: vec![ModKey::Ctrl],
                repeat: 1
            }]
        );
    }

    #[test]
    fn wheel_with_delta() {
        assert_eq!(
            keys("{MOUSE_WHEEL -120}"),
            vec![Action::Key { key: Key::Mouse(MouseKey::Wheel(-120)), mods: vec![], repeat: 1 }]
        );
    }

    #[test]
    fn text_by_default_syntax() {
        let seq = parse("abc{d}", true).unwrap();
        assert_eq!(
            seq.actions,
            vec![
                Action::Text("abc".into()),
                Action::Key { key: Key::Vk(b'D' as u16), mods: vec![], repeat: 1 },
            ]
        );
    }

    #[test]
    fn unknown_key_is_error() {
        assert!(parse("{NOT_A_KEY}", false).is_err());
        assert!(parse("{SLEEP}", false).is_err());
    }
}
