//! 菜单对象和 action target 一直保留到应用退出；所有调用都来自主线程。
use std::cell::Cell;
use std::panic::{catch_unwind, AssertUnwindSafe};

use block2::RcBlock;
use mousetroke_core::i18n::{keys, t};
use objc2::rc::Retained;
use objc2::runtime::{Bool, Sel};
use objc2::{define_class, msg_send, sel, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSBezierPath, NSColor, NSEventModifierFlags, NSImage, NSLineCapStyle, NSMenu, NSMenuItem,
    NSStatusBar, NSStatusItem, NSWorkspace,
};
use objc2_foundation::{
    ns_string, NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize, NSString, NSURL,
};

use crate::permissions::Permissions;
thread_local! { static DIRTY: Cell<bool> = const { Cell::new(true) }; }
thread_local! { static SELECTED: Cell<Option<usize>> = const { Cell::new(None) }; }
thread_local! { static VISIBLE: Cell<bool> = const { Cell::new(true) }; }
pub fn set_visible(visible: bool) {
    VISIBLE.with(|flag| flag.set(visible));
    request_refresh();
}
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
        #[unsafe(method(selectAction:))]
        fn select_action(&self, sender: &NSMenuItem) {
            guard(|| SELECTED.with(|s| s.set(usize::try_from(sender.tag()).ok())));
        }
        #[unsafe(method(togglePause:))]
        fn toggle_pause(&self, _sender: Option<&NSObject>) {
            guard(|| { crate::app::request_toggle_pause(); crate::app::wake(); });
        }
        #[unsafe(method(grantAccessibility:))]
        fn grant_accessibility(&self, _sender: Option<&NSObject>) {
            guard(|| grant_permission(true));
        }
        #[unsafe(method(grantInputMonitoring:))]
        fn grant_input_monitoring(&self, _sender: Option<&NSObject>) {
            guard(|| grant_permission(false));
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
        #[unsafe(method(openSettings:))]
        fn open_settings(&self, _sender: Option<&NSObject>) { guard(|| crate::settings::open(false)); }
        #[unsafe(method(openAbout:))]
        fn open_about(&self, _sender: Option<&NSObject>) { guard(|| crate::settings::open(true)); }
    }
);

/// 必须在 tap 返回后的主线程调用；模态跟踪仍泵送 common-mode tap 和计时器。
pub fn popup(
    items: &[mousetroke_core::schema::MenuItem],
    pos: (i32, i32),
    mtm: MainThreadMarker,
) -> Option<mousetroke_core::schema::Action> {
    use mousetroke_core::schema::Action;
    fn build(
        items: &[mousetroke_core::schema::MenuItem],
        target: &MenuTarget,
        actions: &mut Vec<Action>,
        depth: usize,
        mtm: MainThreadMarker,
    ) -> Retained<NSMenu> {
        let menu = NSMenu::new(mtm);
        menu.setAutoenablesItems(false);
        if depth > 16 {
            return menu;
        }
        for item in items.iter().take(256) {
            if actions.len() >= 256 {
                break;
            }
            // 安全：action selector 在 MenuTarget 定义；target 保留至整个模态菜单返回。
            let entry = unsafe {
                NSMenuItem::initWithTitle_action_keyEquivalent(
                    NSMenuItem::alloc(mtm),
                    &NSString::from_str(&item.label),
                    Some(sel!(selectAction:)),
                    ns_string!(""),
                )
            };
            if let Action::Menu { items, .. } = &item.action {
                entry.setSubmenu(Some(&build(items, target, actions, depth + 1, mtm)));
            } else {
                entry.setTag(actions.len() as isize);
                unsafe { entry.setTarget(Some(target)) };
                actions.push(item.action.clone());
            }
            menu.addItem(&entry);
        }
        menu
    }
    // 安全：NSObject 初始化，在主线程持有 MenuTarget 到菜单关闭。
    let target: Retained<MenuTarget> = unsafe { msg_send![MenuTarget::alloc(mtm), init] };
    let mut actions = Vec::new();
    let menu = build(items, &target, &mut actions, 0, mtm);
    SELECTED.with(|s| s.set(None));
    let height = objc2_app_kit::NSScreen::screens(mtm)
        .iter()
        .next()?
        .frame()
        .size
        .height;
    let (x, y) = crate::helpers::menu_point(pos.0 as f64, pos.1 as f64, height);
    if !menu.popUpMenuPositioningItem_atLocation_inView(None, NSPoint::new(x, y), None) {
        return None;
    }
    SELECTED
        .with(|s| s.take())
        .and_then(|i| actions.get(i).cloned())
}

fn grant_permission(accessibility: bool) {
    if let Err(error) = crate::permissions::request(accessibility) {
        crate::log!("打开系统权限设置失败: {error}");
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
    settings: Retained<NSMenuItem>,
    about: Retained<NSMenuItem>,
    update: Retained<NSMenuItem>,
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
            NSEventModifierFlags::Control
                | NSEventModifierFlags::Option
                | NSEventModifierFlags::Command,
        );
        let accessibility = entry(Some(sel!(grantAccessibility:)));
        let input_monitoring = entry(Some(sel!(grantInputMonitoring:)));
        let open_log = entry(Some(sel!(openLog:)));
        let settings = entry(Some(sel!(openSettings:)));
        settings.setKeyEquivalent(ns_string!(","));
        settings.setKeyEquivalentModifierMask(NSEventModifierFlags::Command);
        let about = entry(Some(sel!(openAbout:)));
        let update = entry(Some(sel!(openAbout:)));
        update.setHidden(true);
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
        let image =
            NSImage::imageWithSize_flipped_drawingHandler(NSSize::new(22.0, 22.0), true, &drawing);
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
            settings,
            about,
            update,
            quit,
        }
    }

    pub fn set_update(&self, version: Option<&str>) {
        self.update.setHidden(version.is_none());
        if let Some(version) = version {
            self.update.setTitle(&NSString::from_str(&mousetroke_core::i18n::tf(keys::MAC_UPDATE_AVAILABLE, &[("version", version)])));
        }
    }

    pub fn refresh(
        &self,
        mtm: MainThreadMarker,
        permissions: Permissions,
        active: bool,
        paused: bool,
    ) {
        self.item.setVisible(VISIBLE.with(Cell::get));
        let key = if !permissions.ready() || !active {
            keys::MAC_STATUS_PERMISSION
        } else if paused {
            keys::MAC_STATUS_PAUSED
        } else {
            keys::MAC_STATUS_RUNNING
        };
        self.status.setTitle(&NSString::from_str(t(key)));
        self.pause.setTitle(&NSString::from_str(t(keys::TRAY_MENU_PAUSE)));
        self.pause.setState(if paused { 1 } else { 0 });
        self.accessibility
            .setTitle(&NSString::from_str(t(keys::MAC_GRANT_ACCESSIBILITY)));
        self.accessibility.setHidden(permissions.accessibility);
        self.input_monitoring
            .setTitle(&NSString::from_str(t(keys::MAC_GRANT_INPUT_MONITORING)));
        self.input_monitoring
            .setHidden(permissions.input_monitoring);
        self.open_log
            .setTitle(&NSString::from_str(t(keys::UI_ABOUT_OPEN_LOG)));
        self.settings.setTitle(&NSString::from_str(t(keys::TRAY_MENU_SETTINGS)));
        self.about.setTitle(&NSString::from_str(t(keys::MAC_MENU_ABOUT)));
        self.quit.setTitle(&NSString::from_str(t(keys::TRAY_MENU_QUIT)));
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
