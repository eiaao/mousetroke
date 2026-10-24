//! Windows 引擎适配：线程局部单例、绘制面、系统查询与异步动作执行。

use std::path::PathBuf;

use mousetroke_core::matcher;
use mousetroke_core::presets::native::PreparedAction;
use mousetroke_core::schema::{Action, Config, MouseButton};
use mousetroke_core::vars::{self, Vars};
use mousetroke_engine::{Canvas, Engine, Host, Target};

use crate::input;
use crate::overlay::Overlay;
use crate::targeting;

pub use mousetroke_engine::{push_toast, PAUSED};

/// 手势名显示在显示器工作区的高度比例（0 = 顶，1 = 底）：中下方，固定不动。
const LABEL_Y: f64 = 0.8;

/// 初始化共享引擎；宿主与日志接收器均由 Windows 层提供。
pub fn new(cfg: Config, cfg_path: Option<PathBuf>) -> Option<Engine> {
    Engine::new(cfg, cfg_path, Box::new(SystemHost))
}

impl Canvas for Overlay {
    fn set_color(&mut self, color: &str) {
        Overlay::set_color(self, color)
    }
    fn set_width(&mut self, width_px: f64) {
        Overlay::set_width(self, width_px)
    }
    fn start_session(&mut self, x: i32, y: i32) {
        Overlay::start_session(self, x, y)
    }
    fn line_to(&mut self, x: i32, y: i32) {
        Overlay::line_to(self, x, y)
    }
    fn finish(&mut self) {
        Overlay::finish(self)
    }
    fn fade_out(&mut self) {
        Overlay::fade_out(self)
    }
    fn tick_fade(&mut self) {
        Overlay::tick_fade(self)
    }
    fn draw_label(&mut self, text: &str, x: i32, y: i32, color: &str) {
        Overlay::draw_label(self, text, x, y, color)
    }
    fn clear_label(&mut self) {
        Overlay::clear_label(self)
    }
    fn virtual_rect(&self) -> (i32, i32, i32, i32) {
        Overlay::virtual_rect(self)
    }
}

pub(crate) struct SystemHost;

impl Host for SystemHost {
    fn target_at(&self, x: i32, y: i32) -> Target {
        targeting::resolve_under_cursor(x, y).into()
    }
    fn ignore_target(&self, target: &Target) -> bool {
        is_own_settings(target) || targeting::is_shell_ui_class(&target.class)
    }
    fn window_title(&self, wid: usize) -> String {
        targeting::window_title(windows::Win32::Foundation::HWND(wid as *mut _))
    }
    fn foreground_fullscreen(&self) -> bool {
        targeting::is_foreground_fullscreen()
    }
    fn replay(&self, btn: MouseButton, click: bool) {
        std::thread::spawn(move || if click { input::replay_click(btn) } else { input::replay_down(btn) });
    }
    fn execute(&self, action: Action, target: Option<Target>, vars: Vars, pos: (i32, i32)) {
        std::thread::spawn(move || execute(action, target.map(Into::into), vars, pos));
    }
    fn label_anchor(&self, x: i32, y: i32) -> (i32, i32) {
        use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST};
        let mut mi = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
        unsafe {
            let mon = MonitorFromPoint(windows::Win32::Foundation::POINT { x, y }, MONITOR_DEFAULTTONEAREST);
            if !GetMonitorInfoW(mon, &mut mi).as_bool() {
                return (x, y - 40);
            }
        }
        let w = mi.rcWork;
        ((w.left + w.right) / 2, w.top + ((w.bottom - w.top) as f64 * LABEL_Y) as i32)
    }
    fn new_canvas(&self, color: &str, width: f64) -> Option<Box<dyn Canvas>> {
        Overlay::new(color, width).map(|o| Box::new(o) as Box<dyn Canvas>)
    }
    fn trim_memory(&self) {
        use windows::Win32::System::Threading::{GetCurrentProcess, SetProcessWorkingSetSize};
        unsafe {
            let _ = SetProcessWorkingSetSize(GetCurrentProcess(), usize::MAX, usize::MAX);
        }
    }
    fn cursor_pos(&self) -> (i32, i32) {
        input::get_cursor_pos()
    }
    fn system_langs(&self) -> Vec<String> {
        crate::system_langs()
    }
    fn refresh_texts(&self) {
        crate::tray::refresh_texts();
    }
    fn shape_sample(&self, encoded: Vec<i32>) {
        mousetroke_engine::log::shape_sample(encoded);
    }
}

/// LL 钩子回调与托盘消息都在引擎线程执行。
pub(crate) use mousetroke_engine::with_engine;

/// 目标是否 Mousetroke 自己的设置界面（与引擎同一个 exe 的另一个进程；开发版为单独的设置程序）。
fn is_own_settings(t: &Target) -> bool {
    static OWN: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    let own = OWN.get_or_init(targeting::own_exe_name);
    t.exe.as_deref().is_some_and(|e| {
        let name = matcher::file_name(e);
        name.eq_ignore_ascii_case("mousetroke-settings.exe") || name.eq_ignore_ascii_case(own)
    })
}

/// 显示器休眠/唤醒时的布局通知可能带过渡尺寸，之后不再通知；
/// 定时与按下时核对，覆盖层与当前虚拟屏不一致就重建（否则轨迹画在屏幕外、边角失灵）。
pub fn sync_display() {
    let current = crate::overlay::system_virtual_rect();
    with_engine(|e| {
        if e.virtual_rect() != current {
            crate::log!("覆盖层与当前屏幕不一致，重建");
            e.on_display_change();
        }
    });
}

/// 执行动作（工作线程）。
fn execute(action: Action, target: Option<targeting::Target>, vars: Vars, pos: (i32, i32)) {
    let Some(action) = mousetroke_engine::prepare_action(action, mousetroke_core::caps::Platform::Windows, mousetroke_core::i18n::lang()) else { return };
    let action = match action {
        PreparedAction::Advanced(action) => action,
        PreparedAction::Native(action) => {
            let id = action.id();
            if let Err(error) = crate::preset_actions::execute(action, target.as_ref()) {
                mousetroke_engine::report_action_failure(id, &error);
            }
            return;
        }
    };
    match action {
        Action::KeySequence { keys, activate_target, .. } => {
            if activate_target {
                if let Some(t) = &target {
                    input::focus_window(t.hwnd);
                    std::thread::sleep(std::time::Duration::from_millis(30));
                }
            }
            match mousetroke_core::keyseq::parse(&keys, false) {
                Ok(seq) => input::inject_keyseq(&seq),
                Err(e) => crate::log!("按键序列解析失败 {keys:?}: {e:?}"),
            }
        }
        Action::Shell { script } => {
            let mut v = vars.clone();
            fill_lazy_vars(&script, &mut v);
            input::run_shell(&vars::expand(&script, &v));
        }
        Action::WindowOp { op, allow_child } => {
            if let Some(t) = &target {
                input::apply_window_op(t.hwnd, op, allow_child);
            }
        }
        Action::OpenUrl { url_template, browser } => {
            let mut v = vars.clone();
            fill_lazy_vars(&url_template, &mut v);
            input::open_url(&vars::expand_url(&url_template, &v), browser.as_deref());
        }
        Action::LaunchApp { exe, args, elevate, win_class, win_title, regex, if_running } => {
            let mut v = vars.clone();
            fill_lazy_vars(&exe, &mut v);
            fill_lazy_vars(args.as_deref().unwrap_or(""), &mut v);
            input::launch_app(
                &vars::expand(&exe, &v),
                Some(vars::expand(args.as_deref().unwrap_or(""), &v)).filter(|s| !s.is_empty()).as_deref(),
                elevate,
                win_class.as_deref(),
                win_title.as_deref(),
                regex,
                if_running,
            );
        }
        Action::Menu { items, .. } => {
            // 模态菜单须在专用线程（不能阻塞钩子，也不占引擎线程）
            std::thread::spawn(move || {
                if let Some(item) = input::show_context_menu(&items, pos.0, pos.1) {
                    execute(item.action, target, vars, pos);
                }
            });
        }
        Action::SystemKey { key } => input::inject_system_key(&key),
        Action::Lua { code } => {
            let ctx = crate::lua::LuaCtx::from_vars(
                pos,
                target.as_ref().map_or(0, |t| t.hwnd.0 as usize),
                target.as_ref().map_or(0, |t| t.pid),
                &vars,
            );
            std::thread::spawn(move || crate::lua::run_lua(&code, &ctx));
        }
        // prepare_action 已处理；保守跳过，避免将未来动作送入旧执行器。
        Action::Preset { .. } | Action::Unknown(_) => {}
    }
}

/// 惰性变量：模板引用了但上下文没有 → 现场采集。
fn fill_lazy_vars(template: &str, vars: &mut Vars) {
    if template.contains("%MS_SELECTED_TEXT%") && !vars.contains_key("MS_SELECTED_TEXT") {
        vars.insert("MS_SELECTED_TEXT".into(), input::capture_selected_text().unwrap_or_default());
    }
    if template.contains("%MS_CLIPBOARD_TEXT%") && !vars.contains_key("MS_CLIPBOARD_TEXT") {
        let text = input::get_clipboard_text().unwrap_or_default();
        vars.insert("MS_CLIPBOARD_TEXT".into(), text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_host_preserves_ignored_targets() {
        let mut target = Target { wid: 0, pid: 0, exe: None, is_desktop: false, class: String::new() };
        for class in [
            "Shell_TrayWnd", "Shell_SecondaryTrayWnd", "NotifyIconOverflowWindow", "TopLevelWindowForOverflowXamlIsland",
            "Xaml_WindowedPopupClass", "XamlExplorerHostIslandWindow", "Windows.UI.Core.CoreWindow", "#32768",
        ] {
            target.class = class.into();
            assert!(SystemHost.ignore_target(&target), "{class}");
        }
        target.class = "Chrome_WidgetWin_1".into();
        assert!(!SystemHost.ignore_target(&target));
        target.exe = Some(r"D:\x\MOUSETROKE-SETTINGS.EXE".into());
        assert!(SystemHost.ignore_target(&target));
        target.exe = Some(targeting::own_exe_name());
        assert!(SystemHost.ignore_target(&target));
        target.exe = Some(r"C:\app\chrome.exe".into());
        assert!(!SystemHost.ignore_target(&target));
    }

    #[test]
    fn target_conversion_preserves_handle_and_context() {
        for wid in [0, 1, usize::MAX] {
            let target = targeting::Target {
                hwnd: windows::Win32::Foundation::HWND(wid as *mut _),
                pid: 42,
                exe: Some(r"C:\app\chrome.exe".into()),
                is_desktop: true,
                class: "WorkerW".into(),
            };
            let shared: Target = target.clone().into();
            assert_eq!(shared.wid, wid);
            let restored: targeting::Target = shared.into();
            assert_eq!(restored.hwnd, target.hwnd);
            assert_eq!(restored.pid, target.pid);
            assert_eq!(restored.exe, target.exe);
            assert_eq!(restored.is_desktop, target.is_desktop);
            assert_eq!(restored.class, target.class);
        }
    }
}
