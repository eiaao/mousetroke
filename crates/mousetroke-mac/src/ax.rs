//! AX 与 CF 的小型所有权封装。跨进程 AX 消息只在工作线程发送，不阻塞事件 tap。
use crate::{
    helpers::{covers, Window},
    overlay::geometry::Rect,
};
use mousetroke_core::schema::WindowOpKind;
use std::ffi::{c_char, c_void, CStr};
use std::ptr;

pub type Raw = *const c_void;
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFRelease(value: Raw);
    fn CFGetTypeID(value: Raw) -> usize;
    fn CFStringGetTypeID() -> usize;
    fn CFArrayGetTypeID() -> usize;
    fn CFDictionaryGetTypeID() -> usize;
    fn CFNumberGetTypeID() -> usize;
    fn CFBooleanGetTypeID() -> usize;
    fn CFStringCreateWithCString(allocator: Raw, text: *const c_char, encoding: u32) -> Raw;
    fn CFStringGetLength(value: Raw) -> isize;
    fn CFStringGetMaximumSizeForEncoding(length: isize, encoding: u32) -> isize;
    fn CFStringGetCString(value: Raw, buffer: *mut c_char, size: isize, encoding: u32) -> bool;
    fn CFArrayGetCount(value: Raw) -> isize;
    fn CFArrayGetValueAtIndex(value: Raw, index: isize) -> Raw;
    fn CFDictionaryGetValue(value: Raw, key: Raw) -> Raw;
    fn CFNumberGetValue(value: Raw, kind: isize, result: *mut c_void) -> bool;
    static kCFBooleanTrue: Raw;
    static kCFBooleanFalse: Raw;
}
#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXUIElementCreateApplication(pid: i32) -> Raw;
    fn AXUIElementCreateSystemWide() -> Raw;
    fn AXUIElementSetMessagingTimeout(element: Raw, timeout: f32) -> i32;
    fn AXUIElementCopyAttributeValue(element: Raw, attribute: Raw, value: *mut Raw) -> i32;
    fn AXUIElementSetAttributeValue(element: Raw, attribute: Raw, value: Raw) -> i32;
    fn AXUIElementPerformAction(element: Raw, action: Raw) -> i32;
    fn AXUIElementCopyElementAtPosition(element: Raw, x: f32, y: f32, value: *mut Raw) -> i32;
    fn AXUIElementGetPid(element: Raw, pid: *mut i32) -> i32;
    fn AXValueGetType(value: Raw) -> u32;
    fn AXValueGetTypeID() -> usize;
    fn AXValueGetValue(value: Raw, kind: u32, output: *mut c_void) -> bool;
}

pub struct Cf(Raw);
impl Drop for Cf {
    fn drop(&mut self) {
        // 安全：只接管 Create/Copy 成功返回的非空 +1 引用，恰好释放一次。
        unsafe { CFRelease(self.0) };
    }
}
impl Cf {
    /// 安全：调用方必须传入 Create/Copy 的 +1 返回值，或空指针。
    pub unsafe fn owned(raw: Raw) -> Option<Self> {
        if raw.is_null() {
            None
        } else {
            Some(Self(raw))
        }
    }
    pub fn raw(&self) -> Raw {
        self.0
    }
    fn key(name: &str) -> Self {
        let text = std::ffi::CString::new(name).expect("static AX attribute without NUL");
        // 安全：UTF-8 C 字符串有效，默认分配器，接管 Create 引用。
        unsafe {
            Self::owned(CFStringCreateWithCString(
                ptr::null(),
                text.as_ptr(),
                0x08000100,
            ))
            .expect("CFString allocation failed")
        }
    }
    pub fn string(&self) -> Option<String> {
        string(self.0)
    }
    fn attr(&self, name: &str) -> Option<Self> {
        let key = Self::key(name);
        let mut raw = ptr::null();
        // 安全：有效 AX 对象、属性字符串与输出地址；失败不读取输出。
        let status = unsafe { AXUIElementCopyAttributeValue(self.0, key.0, &mut raw) };
        if status != 0 {
            return None;
        }
        unsafe { Self::owned(raw) }
    }
    fn timeout(&self) {
        // 安全：有效 AX 对象；单次 IPC 最多 100ms，且始终在工作线程。
        if unsafe { AXUIElementSetMessagingTimeout(self.0, 0.1) } != 0 {
            crate::log!("设置 AX 消息超时失败");
        }
    }
    fn boolean(&self, name: &str) -> Option<bool> {
        let value = self.attr(name)?;
        // 安全：先校验 CFBoolean 类型，再比较系统单例地址。
        unsafe {
            (CFGetTypeID(value.0) == CFBooleanGetTypeID()).then_some(value.0 == kCFBooleanTrue)
        }
    }
    fn set_boolean(&self, name: &str, value: bool) -> i32 {
        let key = Self::key(name);
        // 安全：有效 AX 代理、属性字符串及系统 CFBoolean 常量。
        unsafe {
            AXUIElementSetAttributeValue(
                self.0,
                key.0,
                if value {
                    kCFBooleanTrue
                } else {
                    kCFBooleanFalse
                },
            )
        }
    }
    fn perform(&self, name: &str) -> i32 {
        let action = Self::key(name);
        // 安全：只执行公开 AX 动作，错误由目标应用返回。
        unsafe { AXUIElementPerformAction(self.0, action.0) }
    }
    fn press_button(&self, name: &str) -> Option<i32> {
        Some(self.attr(name)?.perform("AXPress"))
    }
}

pub fn string(raw: Raw) -> Option<String> {
    // 安全：raw 来自仍存活的 CF 容器或 Cf；先检查类型，再分配输出缓冲。
    unsafe {
        if raw.is_null() || CFGetTypeID(raw) != CFStringGetTypeID() {
            return None;
        }
        let size =
            CFStringGetMaximumSizeForEncoding(CFStringGetLength(raw), 0x08000100).checked_add(1)?;
        let mut bytes = vec![0u8; usize::try_from(size).ok()?];
        if !CFStringGetCString(raw, bytes.as_mut_ptr().cast(), size, 0x08000100) {
            return None;
        }
        Some(
            CStr::from_ptr(bytes.as_ptr().cast())
                .to_string_lossy()
                .into_owned(),
        )
    }
}
pub fn array(raw: Raw) -> Vec<Raw> {
    // 安全：检查 CF 类型后读取元素；调用方在使用这些借用指针期间保留原数组。
    unsafe {
        if raw.is_null() || CFGetTypeID(raw) != CFArrayGetTypeID() {
            return Vec::new();
        }
        (0..CFArrayGetCount(raw))
            .map(|i| CFArrayGetValueAtIndex(raw, i))
            .collect()
    }
}
pub fn dict(raw: Raw, key: Raw) -> Raw {
    // 安全：键为系统常量，值借用自仍存活的字典。
    unsafe {
        if raw.is_null() || CFGetTypeID(raw) != CFDictionaryGetTypeID() {
            return ptr::null();
        }
        CFDictionaryGetValue(raw, key)
    }
}
pub fn number(raw: Raw) -> Option<i64> {
    let mut value = 0i64;
    // 安全：先检查 CFNumber 类型，类型 4 为 kCFNumberSInt64Type。
    unsafe {
        if raw.is_null() || CFGetTypeID(raw) != CFNumberGetTypeID() {
            return None;
        }
        CFNumberGetValue(raw, 4, (&mut value as *mut i64).cast()).then_some(value)
    }
}

fn application(pid: u32) -> Option<Cf> {
    // 安全：pid 只用于创建 AX 代理，失效进程由后续 AX 调用返回错误。
    let app = unsafe { Cf::owned(AXUIElementCreateApplication(pid.try_into().ok()?)) }?;
    app.timeout();
    Some(app)
}
fn system() -> Option<Cf> {
    // 安全：Create 返回独立拥有的系统级 AX 代理。
    let system = unsafe { Cf::owned(AXUIElementCreateSystemWide()) }?;
    system.timeout();
    Some(system)
}
fn bounds(window: &Cf) -> Option<Rect> {
    let position = window.attr("AXPosition")?;
    let size = window.attr("AXSize")?;
    let mut point = [0f64; 2];
    let mut dimensions = [0f64; 2];
    // 安全：AXValue 的 CGPoint/CGSize 布局均为两个 CGFloat（64 位）；先检查类型。
    unsafe {
        if CFGetTypeID(position.0) != AXValueGetTypeID()
            || CFGetTypeID(size.0) != AXValueGetTypeID()
            || AXValueGetType(position.0) != 1
            || AXValueGetType(size.0) != 2
            || !AXValueGetValue(position.0, 1, point.as_mut_ptr().cast())
            || !AXValueGetValue(size.0, 2, dimensions.as_mut_ptr().cast())
        {
            return None;
        }
    }
    Some(Rect {
        x: point[0],
        y: point[1],
        w: dimensions[0],
        h: dimensions[1],
    })
}
fn window(pid: u32, expected: Option<&Window>, pos: (i32, i32)) -> Option<Cf> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(500);
    let app = application(pid)?;
    if let Some(rows) = app.attr("AXWindows") {
        for raw in array(rows.0) {
            if std::time::Instant::now() >= deadline {
                return None;
            }
            // 安全：临时包装不拥有数组元素；ManuallyDrop 防止释放借用引用。
            let candidate = std::mem::ManuallyDrop::new(Cf(raw));
            if expected.is_some_and(|w| bounds(&candidate).is_some_and(|b| covers(b, w.bounds))) {
                // 保留数组内窗口的独立引用，数组释放后仍可安全使用。
                return retain(raw);
            }
        }
    }
    // 已知 CG 窗口不能回退到另一个 focused 窗口，避免关闭错误窗口。
    let system = system()?;
    let mut raw = ptr::null();
    // 安全：坐标为 Quartz 全局点；Copy 成功后接管返回引用。
    if unsafe { AXUIElementCopyElementAtPosition(system.0, pos.0 as f32, pos.1 as f32, &mut raw) }
        == 0
    {
        if let Some(mut element) = unsafe { Cf::owned(raw) } {
            for _ in 0..32 {
                if std::time::Instant::now() >= deadline {
                    return None;
                }
                let mut owner = 0;
                if unsafe { AXUIElementGetPid(element.0, &mut owner) } != 0 || owner as u32 != pid {
                    break;
                }
                if element.attr("AXRole").and_then(|v| v.string()).as_deref() == Some("AXWindow") {
                    if expected
                        .is_none_or(|w| bounds(&element).is_some_and(|b| covers(b, w.bounds)))
                    {
                        return Some(element);
                    }
                    break;
                }
                let Some(parent) = element.attr("AXParent") else {
                    break;
                };
                element = parent;
            }
        }
    }
    if expected.is_none() {
        app.attr("AXFocusedWindow")
    } else {
        None
    }
}
fn retain(raw: Raw) -> Option<Cf> {
    unsafe extern "C" {
        fn CFRetain(value: Raw) -> Raw;
    }
    // 安全：raw 为存活的数组元素；获取独立引用后数组可以释放。
    unsafe { Cf::owned(CFRetain(raw)) }
}

pub fn title(pid: u32, expected: Option<&Window>, pos: (i32, i32)) -> String {
    window(pid, expected, pos)
        .and_then(|w| w.attr("AXTitle"))
        .and_then(|v| v.string())
        .unwrap_or_default()
}
pub fn selected_text() -> Option<String> {
    system()?
        .attr("AXFocusedUIElement")?
        .attr("AXSelectedText")?
        .string()
}
pub fn apply(pid: u32, expected: Option<&Window>, pos: (i32, i32), op: WindowOpKind) {
    let Some(window) = window(pid, expected, pos) else {
        crate::log!("找不到目标 AX 窗口，跳过动作");
        return;
    };
    let status = match op {
        WindowOpKind::Minimize => window.set_boolean("AXMinimized", true),
        WindowOpKind::ToggleMaximize | WindowOpKind::Close => {
            if op == WindowOpKind::ToggleMaximize {
                let minimized = window.attr("AXMinimized");
                // 安全：CFBoolean 为规范化的单例，直接比较常量地址。
                if minimized.is_some_and(|v| v.0 == unsafe { kCFBooleanTrue }) {
                    crate::log!("目标窗口已最小化，跳过缩放");
                    return;
                }
            }
            let Some(status) = window.press_button(if op == WindowOpKind::Close {
                "AXCloseButton"
            } else {
                "AXZoomButton"
            }) else {
                crate::log!("目标窗口没有所需 AX 按钮，跳过动作");
                return;
            };
            status
        }
        WindowOpKind::ToggleTopmost => return,
    };
    crate::log!("AX 窗口动作完成: status={status}");
}

fn lua_window(expected: &Window) -> Option<Cf> {
    window(
        expected.pid,
        Some(expected),
        (expected.bounds.x as i32, expected.bounds.y as i32),
    )
}
pub(crate) fn lua_window_title(expected: &Window) -> Option<String> {
    let window = lua_window(expected)?;
    Some(
        window
            .attr("AXTitle")
            .and_then(|v| v.string())
            .unwrap_or_default(),
    )
}
fn zoomed(window: &Cf, work_areas: &[Rect]) -> Option<bool> {
    let frame = bounds(window)?;
    Some(work_areas.iter().any(|area| covers(frame, *area)))
}
/// 与内建动作共用 AX 窗口定位和按钮操作；所有跨进程调用留在 Lua 工作线程。
pub(crate) fn lua_window_state(expected: &Window, work_areas: &[Rect]) -> Option<String> {
    let window = lua_window(expected)?;
    Some(
        if window.boolean("AXMinimized")? {
            "min"
        } else if window.boolean("AXFullScreen") == Some(true) {
            "fullscreen"
        } else if zoomed(&window, work_areas)? {
            "max"
        } else {
            "normal"
        }
        .into(),
    )
}
pub(crate) fn lua_set_window_state(expected: &Window, state: &str, work_areas: &[Rect]) {
    let Some(window) = lua_window(expected) else {
        return;
    };
    let status = match state {
        "min" => Some(window.set_boolean("AXMinimized", true)),
        "fullscreen" => Some(window.set_boolean("AXFullScreen", true)),
        "max" | "normal" | "norm" | "restore" => {
            if window.boolean("AXFullScreen") == Some(true)
                && window.set_boolean("AXFullScreen", false) != 0
            {
                return;
            }
            if window.boolean("AXMinimized") == Some(true)
                && window.set_boolean("AXMinimized", false) != 0
            {
                return;
            }
            match zoomed(&window, work_areas) {
                Some(is_max) if is_max != (state == "max") => window.press_button("AXZoomButton"),
                _ => None,
            }
        }
        _ => return,
    };
    if let Some(status) = status {
        crate::log!("Lua AX 窗口状态操作完成: status={status}");
    }
}
pub(crate) fn lua_focus_window(expected: &Window) -> bool {
    let Some(window) = lua_window(expected) else {
        return false;
    };
    if window.boolean("AXMinimized") == Some(true) && window.set_boolean("AXMinimized", false) != 0
    {
        return false;
    }
    window.perform("AXRaise") == 0
}
