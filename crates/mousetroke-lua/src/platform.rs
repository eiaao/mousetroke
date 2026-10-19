//! 仅传递普通 Rust 数据；平台实现负责线程切换，不得让主线程等待 Lua。
use mlua::{Lua, Table};
use mousetroke_core::keyseq::KeySeq;

#[derive(Clone, Copy)]
pub enum KeyPhase {
    DownUp,
    Down,
    Up,
}

pub trait LuaPlatform: Send + Sync + 'static {
    fn log(&self, message: &str);
    fn unsupported(&self, name: &str) {
        self.log(&format!("Lua API unavailable: {name}"));
    }
    fn install_extensions(&self, _lua: &Lua, _ms: &Table) -> mlua::Result<()> {
        Ok(())
    }
    fn cursor_pos(&self) -> (i32, i32);
    fn set_cursor(&self, _x: i32, _y: i32) {
        self.unsupported("set_mouse_pos");
    }
    fn screen_of(&self, _x: i32, _y: i32) -> Option<(i32, i32, i32, i32)> {
        self.unsupported("get_screen");
        None
    }
    fn send_keys(&self, _seq: &KeySeq, _phase: KeyPhase) {
        self.unsupported("keys");
    }
    fn is_keys_pressed(&self, _seq: &KeySeq) -> Option<bool> {
        self.unsupported("is_keys_pressed");
        None
    }
    fn open_url(&self, _url: &str) {
        self.unsupported("open");
    }
    fn run_shell(&self, _code: &str) {
        self.unsupported("cmd");
    }
    fn launch_app(&self, _file: &str, _args: Option<&str>) {
        self.unsupported("run");
    }
    fn is_paused(&self) -> bool {
        false
    }
    fn set_paused(&self, _paused: bool) {
        self.unsupported("set_paused");
    }
    fn show_settings(&self) {
        self.unsupported("show_settings");
    }
    fn show_toast(&self, _message: String, _duration: u64) {
        self.unsupported("show_toast");
    }
    fn get_clipboard_text(&self) -> Option<String> {
        self.unsupported("get_clipboard_text");
        None
    }
    fn set_clipboard_text(&self, _text: &str) {
        self.unsupported("set_clipboard_text");
    }
    fn sleep(&self, millis: u64) {
        std::thread::sleep(std::time::Duration::from_millis(millis));
    }
    fn find_windows(
        &self,
        class: Option<&str>,
        _title: Option<&str>,
    ) -> Option<Vec<(String, isize)>> {
        self.unsupported(if class.is_some() {
            "find_win_by_class"
        } else {
            "find_win_by_title"
        });
        None
    }
    fn focus_window(&self, _wid: i64) {
        self.unsupported("focus_win");
    }
    fn set_window_state(&self, _wid: i64, _state: &str) {
        self.unsupported("set_win_state");
    }
    fn set_topmost(&self, _wid: i64, _top: bool) {
        self.unsupported("set_win_top");
    }
    fn window_state(&self, _wid: i64) -> Option<String> {
        self.unsupported("get_win_state");
        None
    }
    fn get_brightness(&self, _x: i32, _y: i32) -> Option<i64> {
        self.unsupported("get_brightness");
        None
    }
    fn adjust_brightness(&self, _x: i32, _y: i32, _delta: i32) -> Option<i64> {
        self.unsupported("set_brightness");
        None
    }
}
