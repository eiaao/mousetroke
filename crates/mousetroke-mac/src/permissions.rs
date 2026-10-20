//! 轮询始终静默；只有用户点击对应授权按钮时才请求该项权限。
use mousetroke_core::i18n::{keys, t};
use objc2_app_kit::NSWorkspace;
use objc2_core_foundation::{CFBoolean, CFDictionary, CFString};
use objc2_foundation::{NSString, NSURL};

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

pub fn check() -> Permissions {
    // 安全：系统常量有静态生命周期；类型化字典保留键和值，只在调用期间借用。
    let accessibility = unsafe {
        let options =
            CFDictionary::from_slices(&[kAXTrustedCheckOptionPrompt], &[CFBoolean::new(false)]);
        AXIsProcessTrustedWithOptions(options.as_opaque()) != 0
    };
    // 安全：这两个公开 C API 不接收指针；请求结果只做记录，下次轮询重新查询。
    let input_monitoring = unsafe { CGPreflightListenEventAccess() };
    Permissions {
        accessibility,
        input_monitoring,
    }
}

/// 两种进程来自同一个已签名 bundle、同一个可执行文件，TCC 使用相同的 designated requirement。
/// 设置界面的原生命令和菜单回调均在主线程调用，系统弹窗不会在后台线程创建。
pub fn request(accessibility: bool) -> Result<(), String> {
    let pane = if accessibility {
        let options = unsafe {
            CFDictionary::from_slices(&[kAXTrustedCheckOptionPrompt], &[CFBoolean::new(true)])
        };
        let granted = unsafe { AXIsProcessTrustedWithOptions(options.as_opaque()) != 0 };
        crate::log!("用户请求辅助功能权限: {granted}");
        "Privacy_Accessibility"
    } else {
        let granted = unsafe { CGRequestListenEventAccess() };
        crate::log!("用户请求输入监控权限: {granted}");
        "Privacy_ListenEvent"
    };
    // 旧版安全性面板 URL 也由 macOS 13+ 的 System Settings 处理；拒绝 URL 时回退到隐私总页。
    for url in [
        format!("x-apple.systempreferences:com.apple.preference.security?{pane}"),
        "x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension".into(),
        "x-apple.systempreferences:com.apple.preference.security".into(),
    ] {
        if NSURL::URLWithString(&NSString::from_str(&url))
            .is_some_and(|url| NSWorkspace::sharedWorkspace().openURL(&url))
        {
            return Ok(());
        }
        crate::log!("打开权限面板失败: {url}");
    }
    Err(t(keys::MAC_SETTINGS_OPEN_FAILED).into())
}
