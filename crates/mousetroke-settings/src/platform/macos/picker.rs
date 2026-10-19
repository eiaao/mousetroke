//! 每个显示器各放一个透明点击窗；局部事件循环同时处理 Escape、右键和超时。
use crate::{keys, t, Msg, MsgResult};
use objc2::{define_class, msg_send, rc::Retained, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSApplication, NSBackingStoreType, NSColor, NSCursor, NSEvent, NSEventMask, NSEventType,
    NSScreen, NSTextField, NSView, NSWindow, NSWindowCollectionBehavior, NSWindowStyleMask,
};
use objc2_foundation::{
    NSBundle, NSDate, NSDefaultRunLoopMode, NSObjectProtocol, NSPoint, NSRect, NSSize, NSString,
};
use serde::Serialize;
use std::{
    path::Path,
    time::{Duration, Instant},
};

#[derive(Serialize, Clone)]
pub struct PickedApp {
    pub exe: String,
    pub title: String,
}

pub fn resolve_app(path: &str) -> MsgResult<String> {
    let p = Path::new(path);
    if p.extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("app"))
    {
        return NSBundle::bundleWithURL(&super::icons::bundle_url(p))
            .and_then(|bundle| bundle.bundleIdentifier())
            .map(|id| id.to_string())
            .filter(|id| !id.is_empty())
            .ok_or_else(|| Msg::new(keys::MAC_SETTINGS_NOT_APP, &[("file", path.into())]));
    }
    use std::os::unix::fs::PermissionsExt;
    if p.is_absolute()
        && p.is_file()
        && p.metadata()
            .is_ok_and(|m| m.permissions().mode() & 0o111 != 0)
    {
        return Ok(path.into());
    }
    Err(Msg::new(
        keys::MAC_SETTINGS_NOT_APP,
        &[("file", path.into())],
    ))
}

define_class!(
    // 安全：NSWindow 无额外子类约束，仅主线程创建，重写只返回固定布尔值。
    #[unsafe(super = NSWindow)]
    #[thread_kind = MainThreadOnly]
    struct PickerWindow;
    unsafe impl NSObjectProtocol for PickerWindow {}
    impl PickerWindow {
        #[unsafe(method(canBecomeKeyWindow))]
        fn can_become_key(&self) -> bool { true }
        #[unsafe(method(canBecomeMainWindow))]
        fn can_become_main(&self) -> bool { true }
    }
);

define_class!(
    // 安全：NSView 无额外子类约束，光标区域覆盖本窗口的整个内容区。
    #[unsafe(super = NSView)]
    #[thread_kind = MainThreadOnly]
    struct PickerView;
    unsafe impl NSObjectProtocol for PickerView {}
    impl PickerView {
        #[unsafe(method(resetCursorRects))]
        fn reset_cursor_rects(&self) {
            self.addCursorRect_cursor(self.bounds(), &NSCursor::crosshairCursor());
        }
    }
);

struct Capture {
    windows: Vec<Retained<PickerWindow>>,
}
impl Drop for Capture {
    fn drop(&mut self) {
        for window in &self.windows {
            window.orderOut(None);
            window.close();
        }
        // 安全：本次捕获恰好 push 一次，析构也恰好 pop 一次。
        NSCursor::pop_class();
    }
}

pub fn pick_window() -> MsgResult<Option<PickedApp>> {
    let mtm = MainThreadMarker::new().ok_or_else(|| Msg::new(keys::ERROR_PICK_FAILED, &[]))?;
    let screens = NSScreen::screens(mtm);
    let Some(primary) = screens.firstObject() else {
        return Ok(None);
    };
    let primary_height = primary.frame().size.height;
    let frames: Vec<_> = screens.iter().map(|screen| screen.frame()).collect();
    let app = NSApplication::sharedApplication(mtm);
    let mut capture = Capture {
        windows: Vec::new(),
    };
    NSCursor::crosshairCursor().push();
    for screen in screens.iter() {
        // 安全：PickerWindow 继承 NSWindow，初始化返回同一个子类并保留至退出。
        let window: Retained<PickerWindow> = unsafe {
            msg_send![PickerWindow::alloc(mtm),
            initWithContentRect: screen.frame(), styleMask: NSWindowStyleMask::Borderless,
            backing: NSBackingStoreType::Buffered, defer: false]
        };
        // 安全：Retained 管理唯一拥有关系，关闭窗口时不允许 AppKit 额外 release。
        unsafe { window.setReleasedWhenClosed(false) };
        window.setOpaque(false);
        // 极低透明度保证整屏都能收到鼠标点击，同时仍看得见底下的应用。
        window.setBackgroundColor(Some(&NSColor::colorWithWhite_alpha(0.0, 0.015)));
        window.setHasShadow(false);
        window.setLevel(1000);
        window.setCollectionBehavior(
            NSWindowCollectionBehavior::CanJoinAllSpaces
                | NSWindowCollectionBehavior::FullScreenAuxiliary,
        );
        let view: Retained<PickerView> = unsafe {
            msg_send![PickerView::alloc(mtm), initWithFrame: NSRect::new(NSPoint::new(0.0, 0.0), screen.frame().size)]
        };
        window.setContentView(Some(&view));
        let hint =
            NSTextField::labelWithString(&NSString::from_str(t(keys::UI_PICKER_LIVE_HINT)), mtm);
        hint.setTextColor(Some(&NSColor::whiteColor()));
        hint.setBackgroundColor(Some(&NSColor::colorWithWhite_alpha(0.12, 0.95)));
        hint.setDrawsBackground(true);
        hint.setAlignment(objc2_app_kit::NSTextAlignment::Center);
        let bounds = screen.frame().size;
        hint.setFrame(NSRect::new(
            NSPoint::new((bounds.width - 680.0).max(0.0) / 2.0, bounds.height - 90.0),
            NSSize::new(bounds.width.min(680.0), 44.0),
        ));
        if let Some(view) = window.contentView() {
            view.addSubview(&hint);
        }
        window.makeKeyAndOrderFront(None);
        capture.windows.push(window);
    }
    let deadline = Instant::now() + Duration::from_secs(60);
    let mut clicked: Option<(NSEventType, Option<PickedApp>)> = None;
    while Instant::now() < deadline {
        if NSScreen::screens(mtm)
            .iter()
            .map(|screen| screen.frame())
            .collect::<Vec<_>>()
            != frames
        {
            return Ok(None);
        }
        let until = NSDate::dateWithTimeIntervalSinceNow(0.05);
        let Some(event) = (unsafe {
            app.nextEventMatchingMask_untilDate_inMode_dequeue(
                NSEventMask::Any,
                Some(&until),
                NSDefaultRunLoopMode,
                true,
            )
        }) else {
            continue;
        };
        NSCursor::crosshairCursor().set();
        match event.r#type() {
            NSEventType::RightMouseDown => clicked = Some((NSEventType::RightMouseUp, None)),
            NSEventType::KeyDown if event.keyCode() == 53 => return Ok(None),
            NSEventType::LeftMouseDown => {
                let point = NSEvent::mouseLocation();
                let result = mousetroke_mac::pick_app_at(
                    point.x,
                    primary_height - point.y,
                    std::process::id(),
                );
                // 连抬起事件也吞掉，避免把半次点击漏给底下的窗口。
                clicked = Some((
                    NSEventType::LeftMouseUp,
                    result.map(|(exe, title)| PickedApp { exe, title }),
                ));
            }
            kind if clicked.as_ref().is_some_and(|(up, _)| *up == kind) => {
                return Ok(clicked.take().and_then(|(_, result)| result))
            }
            _ => {
                app.sendEvent(&event);
                NSCursor::crosshairCursor().set();
            }
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    #[test]
    fn bundle_selection_returns_identity_and_executables_keep_their_path() {
        let root =
            std::env::temp_dir().join(format!("mousetroke-app-picker-{}", std::process::id()));
        let app = root.join("Example with spaces.app");
        let contents = app.join("Contents");
        std::fs::create_dir_all(&contents).unwrap();
        let plist = contents.join("Info.plist");
        std::fs::write(
            &plist,
            r#"<?xml version="1.0"?><plist version="1.0"><dict>
            <key>CFBundleIdentifier</key><string>org.mousetroke.picker-test</string>
            <key>CFBundlePackageType</key><string>APPL</string>
            </dict></plist>"#,
        )
        .unwrap();
        assert_eq!(
            super::resolve_app(&app.to_string_lossy()).unwrap(),
            "org.mousetroke.picker-test"
        );
        assert_eq!(
            super::resolve_app("/usr/bin/true").unwrap(),
            "/usr/bin/true"
        );
        assert!(super::resolve_app(&plist.to_string_lossy()).is_err());
        std::fs::remove_file(plist).unwrap();
        std::fs::remove_dir(contents).unwrap();
        std::fs::remove_dir(app).unwrap();
        std::fs::remove_dir(root).unwrap();
    }
}
