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
    /// 监听访问的预检结果；辅助功能也能提供访问，不代表单独授予了输入监控。
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

/// 引导、两秒轮询与 tap 禁用处理共用短命同 bundle 进程，绕过 TCC 的正／负进程缓存。
/// 不读取配置、不请求权限、不创建文件；调用者必须在 tap 回调外调用。
pub fn check_fresh() -> Option<Permissions> {
    use std::{process::{Command, Stdio}, time::{Duration, Instant}};
    let mut child = Command::new(std::env::current_exe().ok()?)
        .arg("--permission-status")
        .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
        .spawn().ok()?;
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Ok(Some(status)) = child.try_wait() {
            let code = status.code()?;
            return (0..=3).contains(&code).then_some(Permissions {
                accessibility: code & 1 != 0,
                input_monitoring: code & 2 != 0,
            });
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// 两个可执行入口都先处理内部探针，避免开发用设置程序意外递归打开窗口。
pub fn exit_if_probe() {
    if std::env::args().skip(1).eq(["--permission-status"]) {
        let status = check();
        std::process::exit(i32::from(status.accessibility) | (i32::from(status.input_monitoring) << 1));
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
        // 某些系统仅请求 API 不会登记列表；真实监听 tap 的创建也触发 TCC 登记。
        crate::events::register_input_monitoring();
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
