//! 菜单对象和 action target 一直保留到应用退出；所有调用都来自主线程。
use std::cell::Cell;
use std::panic::{catch_unwind, AssertUnwindSafe};

use block2::RcBlock;
use mousetroke_core::i18n::{keys, t};
use objc2::rc::Retained;
use objc2::runtime::{Bool, Sel};
use objc2::{define_class, msg_send, sel, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSBezierPath, NSColor, NSEventModifierFlags, NSImage, NSLineCapStyle, NSMenu, NSMenuItem, NSStatusBar,
    NSStatusItem, NSWorkspace,
};
use objc2_foundation::{ns_string, NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize, NSString, NSURL};

use crate::permissions::Permissions;
thread_local! { static DIRTY: Cell<bool> = const { Cell::new(true) }; }
pub fn request_refresh() {
    DIRTY.with(|flag| flag.set(true));
}
pub fn take_refresh() -> bool {
    DIRTY.with(|flag| flag.replace(false))
}

fn guard(action: impl FnOnce()) {
    if catch_unwind(AssertUnwindSafe(action)).is_err() {
        crate::log!("菜单回调发生 panic");
    }
}

define_class!(
    // 安全：NSObject 无额外子类约束；对象限定主线程，selector 签名与 AppKit action 一致。
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    struct MenuTarget;
    unsafe impl NSObjectProtocol for MenuTarget {}
    impl MenuTarget {
        #[unsafe(method(togglePause:))]
        fn toggle_pause(&self, _sender: Option<&NSObject>) { guard(crate::app::toggle_pause); }
        #[unsafe(method(grantAccessibility:))]
        fn grant_accessibility(&self, _sender: Option<&NSObject>) {
            guard(|| open_url("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"));
        }
        #[unsafe(method(grantInputMonitoring:))]
        fn grant_input_monitoring(&self, _sender: Option<&NSObject>) {
            guard(|| open_url("x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent"));
        }
        #[unsafe(method(openLog:))]
        fn open_log(&self, _sender: Option<&NSObject>) {
            guard(|| {
                let url = NSURL::fileURLWithPath(&NSString::from_str(&crate::log::path().to_string_lossy()));
                if !NSWorkspace::sharedWorkspace().openURL(&url) { crate::log!("打开日志失败"); }
            });
        }
        #[unsafe(method(quit:))]
        fn quit(&self, _sender: Option<&NSObject>) { guard(|| crate::app::request_quit(0)); }
    }
);

fn open_url(text: &str) {
    if !NSURL::URLWithString(&NSString::from_str(text)).is_some_and(|url| NSWorkspace::sharedWorkspace().openURL(&url))
    {
        crate::log!("打开系统权限设置失败");
    }
}

pub struct Menu {
    item: Retained<NSStatusItem>,
    _menu: Retained<NSMenu>,
    _target: Retained<MenuTarget>,
    status: Retained<NSMenuItem>,
    pause: Retained<NSMenuItem>,
    accessibility: Retained<NSMenuItem>,
    input_monitoring: Retained<NSMenuItem>,
    open_log: Retained<NSMenuItem>,
    quit: Retained<NSMenuItem>,
}

impl Menu {
    pub fn new(mtm: MainThreadMarker) -> Self {
        // 安全：MenuTarget 的唯一初始化路径；NSObject init 的返回类型与所有权正确。
        let target: Retained<MenuTarget> = unsafe { msg_send![MenuTarget::alloc(mtm), init] };
        let menu = NSMenu::new(mtm);
        menu.setAutoenablesItems(false);
        let entry = |action: Option<Sel>| {
            // 安全：selector 均由上面类定义，target 被 Menu 保留，菜单在主线程使用。
            let item = unsafe {
                NSMenuItem::initWithTitle_action_keyEquivalent(
                    NSMenuItem::alloc(mtm),
                    ns_string!(""),
                    action,
                    ns_string!(""),
                )
            };
            unsafe { item.setTarget(Some(&target)) };
            menu.addItem(&item);
            item
        };
        let status = entry(None);
        status.setEnabled(false);
        let pause = entry(Some(sel!(togglePause:)));
        pause.setKeyEquivalent(ns_string!("p"));
        pause.setKeyEquivalentModifierMask(
            NSEventModifierFlags::Control | NSEventModifierFlags::Option | NSEventModifierFlags::Command,
        );
        let accessibility = entry(Some(sel!(grantAccessibility:)));
        let input_monitoring = entry(Some(sel!(grantInputMonitoring:)));
        let open_log = entry(Some(sel!(openLog:)));
        let quit = entry(Some(sel!(quit:)));
        let item = NSStatusBar::systemStatusBar().statusItemWithLength(24.0);
        let drawing = RcBlock::new(|rect: NSRect| -> Bool {
            if MainThreadMarker::new().is_none() {
                return Bool::NO;
            }
            // 系统调用绘图 block 时也在主线程；用 flipped 坐标保留原始 logo 折线方向。
            let result = catch_unwind(AssertUnwindSafe(|| {
                NSColor::blackColor().set();
                let path = NSBezierPath::bezierPath();
                let scale = rect.size.width / 32.0;
                path.setLineWidth(3.0 * scale);
                path.setLineCapStyle(NSLineCapStyle::Round);
                path.moveToPoint(NSPoint::new(7.0 * scale, 11.0 * scale));
                for (x, y) in [(13.5, 21.0), (19.0, 13.0), (25.0, 21.0)] {
                    path.lineToPoint(NSPoint::new(x * scale, y * scale));
                }
                path.stroke();
            }));
            Bool::new(result.is_ok())
        });
        let image = NSImage::imageWithSize_flipped_drawingHandler(NSSize::new(22.0, 22.0), true, &drawing);
        image.setTemplate(true);
        if let Some(button) = item.button(mtm) {
            button.setImage(Some(&image));
        } else {
            crate::log!("创建菜单栏按钮失败");
        }
        item.setMenu(Some(&menu));
        Self {
            item,
            _menu: menu,
            _target: target,
            status,
            pause,
            accessibility,
            input_monitoring,
            open_log,
            quit,
        }
    }

    pub fn refresh(&self, mtm: MainThreadMarker, permissions: Permissions, active: bool, paused: bool) {
        let key = if !permissions.ready() || !active {
            keys::MAC_STATUS_PERMISSION
        } else if paused {
            keys::MAC_STATUS_PAUSED
        } else {
            keys::MAC_STATUS_RUNNING
        };
        self.status.setTitle(&NSString::from_str(t(key)));
        self.pause.setTitle(&NSString::from_str(t(keys::MAC_PAUSE)));
        self.pause.setState(if paused { 1 } else { 0 });
        self.accessibility
            .setTitle(&NSString::from_str(t(keys::MAC_GRANT_ACCESSIBILITY)));
        self.accessibility.setHidden(permissions.accessibility);
        self.input_monitoring
            .setTitle(&NSString::from_str(t(keys::MAC_GRANT_INPUT_MONITORING)));
        self.input_monitoring.setHidden(permissions.input_monitoring);
        self.open_log.setTitle(&NSString::from_str(t(keys::MAC_OPEN_LOG)));
        self.quit.setTitle(&NSString::from_str(t(keys::MAC_QUIT)));
        if let Some(button) = self.item.button(mtm) {
            button.setAppearsDisabled(paused || !permissions.ready() || !active);
            button.setToolTip(Some(&NSString::from_str(t(key))));
        }
    }
}

impl Drop for Menu {
    fn drop(&mut self) {
        NSStatusBar::systemStatusBar().removeStatusItem(&self.item);
    }
}
