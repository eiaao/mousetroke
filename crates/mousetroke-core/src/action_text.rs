//! 文本动作的字面序列和临时剪贴板流程；平台实现读写、注入与等待。
use crate::keyseq::{Action, Key, KeySeq, ModKey};
use std::time::Duration;

pub const DEFAULT_PASTE_DELAY: Duration = Duration::from_millis(150);
pub const COPY_POLL_DELAY: Duration = Duration::from_millis(10);
pub const COPY_POLL_LIMIT: usize = 50;

pub fn literal_text(text: &str) -> KeySeq {
    KeySeq {
        actions: vec![Action::Text(text.into())],
    }
}

pub fn paste_shortcut(modifier: ModKey) -> KeySeq {
    KeySeq {
        actions: vec![Action::Key {
            key: Key::Vk(0x56),
            mods: vec![modifier],
            repeat: 1,
        }],
    }
}

pub trait PasteBackend {
    fn clipboard_has_non_text(&mut self) -> Result<bool, String>;
    fn read_text(&mut self) -> Result<Option<String>, String>;
    fn write_text(&mut self, text: Option<&str>) -> Result<(), String>;
    fn paste(&mut self) -> Result<(), String>;
    fn wait(&mut self, delay: Duration);
}

pub trait CopyBackend {
    fn change_count(&mut self) -> Result<u64, String>;
    fn copy(&mut self) -> Result<(), String>;
    /// 核对版本并替换；没有新文本或期间再次变化时保留剪贴板并报错。
    fn replace_with_plain_text(&mut self, expected: u64) -> Result<(), String>;
    fn wait(&mut self, delay: Duration);
}

/// 用版本号识别复制完成，同样的文字也可能从富文本重新复制；最多等待 500ms。
pub fn copy_plain(backend: &mut impl CopyBackend) -> Result<(), String> {
    let previous = backend.change_count()?;
    backend.copy()?;
    for _ in 0..COPY_POLL_LIMIT {
        backend.wait(COPY_POLL_DELAY);
        let current = backend.change_count()?;
        if current != previous {
            return backend.replace_with_plain_text(current);
        }
    }
    Err("Copy timed out; clipboard was not changed".into())
}

/// 非文本剪贴板无法完整恢复，检测失败时也直接输入，避免覆盖原内容。
pub fn paste_or_type(
    backend: &mut impl PasteBackend,
    text: &str,
    delay: Duration,
    type_direct: impl FnOnce(&str) -> Result<(), String>,
) -> Result<(), String> {
    if backend.clipboard_has_non_text().unwrap_or(true) {
        type_direct(text)
    } else {
        paste_with_restore(backend, text, delay)
    }
}

/// 即使写入或粘贴失败也尝试恢复；恢复失败必须报告，不能悄悄覆盖旧内容。
pub fn paste_with_restore(
    backend: &mut impl PasteBackend,
    text: &str,
    delay: Duration,
) -> Result<(), String> {
    let saved = backend.read_text()?;
    let result = backend
        .write_text(Some(text))
        .and_then(|()| backend.paste());
    if result.is_ok() {
        backend.wait(delay);
    }
    let restored = backend.write_text(saved.as_deref());
    match (result, restored) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) => Err(error),
        (Ok(()), Err(error)) => Err(format!("Restore clipboard failed: {error}")),
        (Err(error), Err(restore)) => Err(format!("{error}; restore clipboard failed: {restore}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct CopyFake {
        polls: usize,
        changed_at: usize,
        copied: bool,
        replaced: bool,
        replacement_fails: bool,
    }
    impl CopyBackend for CopyFake {
        fn change_count(&mut self) -> Result<u64, String> {
            Ok(if self.copied && self.polls >= self.changed_at {
                2
            } else {
                1
            })
        }
        fn copy(&mut self) -> Result<(), String> {
            self.copied = true;
            Ok(())
        }
        fn replace_with_plain_text(&mut self, expected: u64) -> Result<(), String> {
            assert_eq!(expected, 2);
            if self.replacement_fails {
                Err("No text or clipboard changed again".into())
            } else {
                self.replaced = true;
                Ok(())
            }
        }
        fn wait(&mut self, delay: Duration) {
            assert_eq!(delay, COPY_POLL_DELAY);
            self.polls += 1;
        }
    }

    #[test]
    fn plain_copy_waits_for_a_new_clipboard_version_and_preserves_it_on_failure() {
        for (changed_at, replacement_fails, success, polls) in [
            (3, false, true, 3),
            (usize::MAX, false, false, COPY_POLL_LIMIT),
            (1, true, false, 1),
        ] {
            let mut backend = CopyFake {
                polls: 0,
                changed_at,
                copied: false,
                replaced: false,
                replacement_fails,
            };
            assert_eq!(copy_plain(&mut backend).is_ok(), success);
            assert_eq!(backend.replaced, success);
            assert_eq!(backend.polls, polls);
        }
    }
    #[derive(Default)]
    struct Fake {
        saved: Option<String>,
        calls: Vec<String>,
        fail_read: bool,
        non_text: bool,
        fail_check: bool,
        fail_write: bool,
        fail_paste: bool,
        fail_restore: bool,
    }
    impl PasteBackend for Fake {
        fn clipboard_has_non_text(&mut self) -> Result<bool, String> {
            if self.fail_check { Err("check failed".into()) } else { Ok(self.non_text) }
        }
        fn read_text(&mut self) -> Result<Option<String>, String> {
            self.calls.push("read".into());
            if self.fail_read {
                Err("read failed".into())
            } else {
                Ok(self.saved.clone())
            }
        }
        fn write_text(&mut self, text: Option<&str>) -> Result<(), String> {
            self.calls.push(format!("write:{text:?}"));
            if (self.calls.len() == 2 && self.fail_write)
                || (self.calls.len() > 2 && self.fail_restore)
            {
                Err("write failed".into())
            } else {
                Ok(())
            }
        }
        fn paste(&mut self) -> Result<(), String> {
            self.calls.push("paste".into());
            if self.fail_paste {
                Err("paste failed".into())
            } else {
                Ok(())
            }
        }
        fn wait(&mut self, delay: Duration) {
            self.calls.push(format!("wait:{}", delay.as_millis()));
        }
    }
    #[test]
    fn non_text_or_failed_detection_types_without_reading_or_writing_clipboard() {
        for (non_text, fail_check) in [(true, false), (false, true)] {
            let mut f = Fake { non_text, fail_check, ..Fake::default() };
            let mut typed = None;
            paste_or_type(&mut f, "new", DEFAULT_PASTE_DELAY, |text| {
                typed = Some(text.to_owned());
                Ok(())
            }).unwrap();
            assert_eq!(typed.as_deref(), Some("new"));
            assert!(f.calls.is_empty(), "剪贴板不应被读取或写入");
        }
    }

    #[test]
    fn text_or_empty_clipboard_keeps_paste_restore_and_delay() {
        for saved in [Some("old".into()), Some(String::new()), None] {
            let mut f = Fake { saved: saved.clone(), ..Fake::default() };
            paste_or_type(&mut f, "new", DEFAULT_PASTE_DELAY, |_| panic!("must paste")).unwrap();
            assert_eq!(f.calls, vec!["read".into(), "write:Some(\"new\")".into(), "paste".into(), "wait:150".into(), format!("write:{:?}", saved.as_deref())]);
        }
    }

    #[test]
    fn text_never_parses_syntax_and_chords_have_one_complete_pair() {
        let text = "中😀 ^%+#({ENTER})\n";
        assert_eq!(literal_text(text).actions, vec![Action::Text(text.into())]);
        for modifier in [ModKey::Ctrl, ModKey::Win] {
            assert_eq!(
                paste_shortcut(modifier).actions,
                vec![Action::Key {
                    key: Key::Vk(0x56),
                    mods: vec![modifier],
                    repeat: 1
                }]
            );
        }
    }
    #[test]
    fn saves_waits_then_restores_text_or_clears_missing_text() {
        for saved in [Some("old".into()), Some(String::new()), None] {
            let mut f = Fake {
                saved: saved.clone(),
                ..Fake::default()
            };
            paste_with_restore(&mut f, "new", DEFAULT_PASTE_DELAY).unwrap();
            assert_eq!(
                f.calls,
                vec![
                    "read".into(),
                    "write:Some(\"new\")".into(),
                    "paste".into(),
                    "wait:150".into(),
                    format!("write:{:?}", saved.as_deref())
                ]
            );
        }
    }
    #[test]
    fn errors_preserve_restore_attempt_and_report_both_failures() {
        for (write, paste, restore) in [
            (true, false, false),
            (false, true, false),
            (false, false, true),
            (false, true, true),
        ] {
            let mut f = Fake {
                saved: Some("old".into()),
                fail_write: write,
                fail_paste: paste,
                fail_restore: restore,
                ..Fake::default()
            };
            let error = paste_with_restore(&mut f, "new", Duration::ZERO).unwrap_err();
            assert_eq!(f.calls.last().unwrap(), "write:Some(\"old\")");
            assert_eq!(
                error.to_lowercase().contains("restore clipboard failed"),
                restore
            );
        }
        let mut f = Fake {
            fail_read: true,
            ..Fake::default()
        };
        assert!(paste_with_restore(&mut f, "new", Duration::ZERO).is_err());
        assert_eq!(f.calls, vec!["read"]);
    }
}
