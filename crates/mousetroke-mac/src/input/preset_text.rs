//! 作为 input 子模块复用 Action::Text 的 Unicode 事件和完整修饰键释放组。
use crate::preset_actions::on_main;
use mousetroke_core::{
    action_text::{self, CopyBackend, PasteBackend},
    keyseq::ModKey,
};
use objc2_app_kit::{NSPasteboard, NSPasteboardTypeString};
use objc2_foundation::NSString;
use std::{sync::Mutex, time::Duration};

static PASTE_LOCK: Mutex<()> = Mutex::new(());
struct Backend;
impl PasteBackend for Backend {
    fn read_text(&mut self) -> Result<Option<String>, String> {
        // 沿用 Lua 的 NSPasteboard 字符串读写实现，所有 AppKit 操作回主线程。
        on_main(|| {
            Ok(
                unsafe { NSPasteboard::generalPasteboard().stringForType(NSPasteboardTypeString) }
                    .map(|s| s.to_string()),
            )
        })
    }
    fn write_text(&mut self, text: Option<&str>) -> Result<(), String> {
        let text = text.map(str::to_owned);
        on_main(move || {
            let board = NSPasteboard::generalPasteboard();
            board.clearContents();
            if let Some(text) = text {
                if !unsafe {
                    board.setString_forType(&NSString::from_str(&text), NSPasteboardTypeString)
                } {
                    return Err("Write clipboard text failed".into());
                }
            }
            Ok(())
        })
    }
    fn paste(&mut self) -> Result<(), String> {
        on_main(|| {
            if super::post_chord(0x56, &[ModKey::Win], false) {
                Ok(())
            } else {
                Err("Create complete Cmd+V event group failed".into())
            }
        })
    }
    fn wait(&mut self, delay: Duration) {
        std::thread::sleep(delay);
    }
}

impl CopyBackend for Backend {
    fn change_count(&mut self) -> Result<u64, String> {
        on_main(|| Ok(NSPasteboard::generalPasteboard().changeCount() as u64))
    }
    fn copy(&mut self) -> Result<(), String> {
        on_main(|| {
            if super::copy_selection() {
                Ok(())
            } else {
                Err("Create complete Cmd+C event group failed".into())
            }
        })
    }
    fn replace_with_plain_text(&mut self, expected: u64) -> Result<(), String> {
        on_main(move || {
            let board = NSPasteboard::generalPasteboard();
            if board.changeCount() as u64 != expected {
                return Err("Clipboard changed again; plain-text copy cancelled".into());
            }
            let text = unsafe { board.stringForType(NSPasteboardTypeString) }
                .ok_or("Copied clipboard does not contain text")?;
            if board.changeCount() as u64 != expected {
                return Err("Clipboard changed while reading; plain-text copy cancelled".into());
            }
            board.clearContents();
            if unsafe { board.setString_forType(&text, NSPasteboardTypeString) } {
                Ok(())
            } else {
                Err("Write plain clipboard text failed".into())
            }
        })
    }
    fn wait(&mut self, delay: Duration) {
        std::thread::sleep(delay);
    }
}

pub fn copy_plain() -> Result<(), String> {
    if objc2::MainThreadMarker::new().is_some() {
        return Err("Copy wait must run on a worker thread".into());
    }
    let _lock = PASTE_LOCK.lock().map_err(|_| "Text paste lock poisoned")?;
    action_text::copy_plain(&mut Backend)
}

/// 仅保存文本；原先没有文本时恢复为空。必须从工作线程调用，主 run loop 需保持运行。
pub fn type_text_via_paste(text: &str) -> Result<(), String> {
    type_text_via_paste_with_delay(text, action_text::DEFAULT_PASTE_DELAY)
}

pub fn type_text_via_paste_with_delay(text: &str, delay: Duration) -> Result<(), String> {
    if objc2::MainThreadMarker::new().is_some() {
        return Err("Paste delay must run on a worker thread".into());
    }
    let _lock = PASTE_LOCK.lock().map_err(|_| "Text paste lock poisoned")?;
    action_text::paste_with_restore(&mut Backend, text, delay)
}

/// 与 Action::Text 共用 post_text；CGEvent 可在工作线程投递，不占用主 run loop。
pub fn type_text_direct(text: &str) -> Result<(), String> {
    for ch in text.chars() {
        if !super::post_text(ch, false) {
            return Err("Create Unicode text event failed".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore] // 授予测试终端辅助功能权限；运行后 3 秒内聚焦空白编辑器。
    fn desktop_direct_text() {
        std::thread::sleep(std::time::Duration::from_secs(3));
        super::type_text_direct("直接 😀 ^+#{ENTER}\n").unwrap();
    }
}
