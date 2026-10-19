//! 仅启动时请求权限，后续轮询不再弹窗。
use objc2_core_foundation::{CFBoolean, CFDictionary, CFString};

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrustedWithOptions(options: &CFDictionary) -> u8;
    static kAXTrustedCheckOptionPrompt: &'static CFString;
    fn CGPreflightListenEventAccess() -> bool;
    fn CGRequestListenEventAccess() -> bool;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Permissions {
    pub accessibility: bool,
    pub input_monitoring: bool,
}

impl Permissions {
    pub fn ready(self) -> bool {
        self.accessibility && self.input_monitoring
    }
}

pub fn check(prompt: bool) -> Permissions {
    // 安全：系统常量有静态生命周期；类型化字典保留键和值，只在调用期间借用。
    let accessibility = unsafe {
        let options = CFDictionary::from_slices(&[kAXTrustedCheckOptionPrompt], &[CFBoolean::new(prompt)]);
        AXIsProcessTrustedWithOptions(options.as_opaque()) != 0
    };
    // 安全：这两个公开 C API 不接收指针；请求结果只做记录，下次轮询重新查询。
    let input_monitoring = unsafe { CGPreflightListenEventAccess() };
    if prompt && !input_monitoring {
        let requested = unsafe { CGRequestListenEventAccess() };
        crate::log!("输入监控权限请求结果: {requested}");
    }
    Permissions {
        accessibility,
        input_monitoring,
    }
}
