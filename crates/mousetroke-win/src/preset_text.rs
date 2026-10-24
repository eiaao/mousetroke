//! 临时剪贴板仅保存文本；图片、HTML、文件列表等非文本格式不能恢复。
//! 调用方应在动作工作线程串行执行；150ms 是默认值，慢应用可增加等待时间。
use crate::input;
use mousetroke_core::{
    action_text::{self, PasteBackend},
    keyseq::ModKey,
};
use std::{sync::Mutex, time::Duration};
use windows::{
    core::w,
    Win32::{
        Foundation::{GlobalFree, HANDLE, HGLOBAL, HWND},
        System::{
            DataExchange::{
                CloseClipboard, EmptyClipboard, IsClipboardFormatAvailable, OpenClipboard,
                SetClipboardData,
            },
            Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE},
        },
        UI::WindowsAndMessaging::{
            CreateWindowExW, DestroyWindow, HWND_MESSAGE, WINDOW_EX_STYLE, WINDOW_STYLE,
        },
    },
};

static PASTE_LOCK: Mutex<()> = Mutex::new(());
const TEXT_FORMAT: u32 = 13;

struct Clipboard;
impl Clipboard {
    fn open(owner: Option<HWND>) -> Result<Self, String> {
        let mut last = String::new();
        for _ in 0..20 {
            match unsafe { OpenClipboard(owner) } {
                Ok(()) => return Ok(Self),
                Err(error) => {
                    last = error.to_string();
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        }
        Err(format!("Open clipboard: {last}"))
    }
}
impl Drop for Clipboard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseClipboard();
        }
    }
}

struct Owner(HWND);
impl Owner {
    fn new() -> Result<Self, String> {
        // 使用本线程的隐藏消息窗口作 owner；NULL owner 在 EmptyClipboard 后不能可靠写入。
        unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("STATIC"),
                w!(""),
                WINDOW_STYLE(0),
                0,
                0,
                0,
                0,
                Some(HWND_MESSAGE),
                None,
                None,
                None,
            )
        }
        .map(Self)
        .map_err(|e| format!("Create clipboard owner window: {e}"))
    }
}
impl Drop for Owner {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.0);
        }
    }
}

struct Memory(HGLOBAL);
impl Drop for Memory {
    fn drop(&mut self) {
        unsafe {
            let _ = GlobalFree(Some(self.0));
        }
    }
}

struct Backend {
    owner: Owner,
}
impl PasteBackend for Backend {
    fn read_text(&mut self) -> Result<Option<String>, String> {
        {
            let _clipboard = Clipboard::open(Some(self.owner.0))?;
            if unsafe { IsClipboardFormatAvailable(TEXT_FORMAT) }.is_err() {
                return Ok(None);
            }
        }
        // 沿用 Lua 的读取实现；已有文本却读不到时停止，避免把访问失败当作空剪贴板。
        input::get_clipboard_text()
            .map(Some)
            .ok_or_else(|| "Read original clipboard text failed".into())
    }

    fn write_text(&mut self, text: Option<&str>) -> Result<(), String> {
        let memory = if let Some(text) = text {
            if text.contains('\0') {
                return Err("Clipboard text cannot contain NUL".into());
            }
            let wide: Vec<u16> = text.encode_utf16().chain([0]).collect();
            let memory = Memory(
                unsafe { GlobalAlloc(GMEM_MOVEABLE, wide.len() * 2) }
                    .map_err(|e| format!("Allocate clipboard text: {e}"))?,
            );
            let dst = unsafe { GlobalLock(memory.0) } as *mut u16;
            if dst.is_null() {
                return Err("Lock clipboard text memory failed".into());
            }
            unsafe {
                std::ptr::copy_nonoverlapping(wide.as_ptr(), dst, wide.len());
                let _ = GlobalUnlock(memory.0);
            }
            Some(memory)
        } else {
            None
        };
        let _clipboard = Clipboard::open(Some(self.owner.0))?;
        unsafe { EmptyClipboard() }.map_err(|e| format!("Clear clipboard: {e}"))?;
        if let Some(memory) = memory {
            unsafe { SetClipboardData(TEXT_FORMAT, Some(HANDLE(memory.0 .0))) }
                .map_err(|e| format!("Write clipboard text: {e}"))?;
            std::mem::forget(memory); // SetClipboardData 成功后系统接管内存。
        }
        Ok(())
    }

    fn paste(&mut self) -> Result<(), String> {
        let sequence = action_text::paste_shortcut(ModKey::Ctrl);
        struct Release<'a>(&'a mousetroke_core::keyseq::KeySeq);
        impl Drop for Release<'_> {
            fn drop(&mut self) {
                input::inject_keyseq_phase(self.0, input::KeyPhase::Up);
            }
        }
        let _release = Release(&sequence);
        input::inject_keyseq(&sequence);
        Ok(())
    }
    fn wait(&mut self, delay: Duration) {
        std::thread::sleep(delay);
    }
}

pub fn type_text_via_paste(text: &str) -> Result<(), String> {
    type_text_via_paste_with_delay(text, action_text::DEFAULT_PASTE_DELAY)
}

pub fn type_text_via_paste_with_delay(text: &str, delay: Duration) -> Result<(), String> {
    if text.contains('\0') {
        return Err("Clipboard text cannot contain NUL".into());
    }
    let _lock = PASTE_LOCK.lock().map_err(|_| "Text paste lock poisoned")?;
    let mut backend = Backend {
        owner: Owner::new()?,
    };
    action_text::paste_with_restore(&mut backend, text, delay)
}

/// 直接构造 Action::Text，语法字符也作为 Unicode 文本注入；沿用 SendInput 的权限限制。
pub fn type_text_direct(text: &str) {
    input::inject_keyseq(&action_text::literal_text(text));
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore] // 运行后 3 秒内聚焦空白编辑器；检查中文、emoji、语法字符和剪贴板恢复。
    fn desktop_text_output() {
        std::thread::sleep(std::time::Duration::from_secs(3));
        super::type_text_via_paste("粘贴 😀 ^+#{ENTER}\n").unwrap();
        super::type_text_direct("直接 😀 ^+#{ENTER}\n");
    }
}
