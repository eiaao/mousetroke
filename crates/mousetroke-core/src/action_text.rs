//! 文本动作的字面序列和临时剪贴板流程；平台实现读写、注入与等待。
use crate::keyseq::{Action, Key, KeySeq, ModKey};
use std::time::Duration;

pub const DEFAULT_PASTE_DELAY: Duration = Duration::from_millis(150);

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
    fn read_text(&mut self) -> Result<Option<String>, String>;
    fn write_text(&mut self, text: Option<&str>) -> Result<(), String>;
    fn paste(&mut self) -> Result<(), String>;
    fn wait(&mut self, delay: Duration);
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
    #[derive(Default)]
    struct Fake {
        saved: Option<String>,
        calls: Vec<String>,
        fail_read: bool,
        fail_write: bool,
        fail_paste: bool,
        fail_restore: bool,
    }
    impl PasteBackend for Fake {
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
