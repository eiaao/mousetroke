//! 新版本系统通知：Windows 通知中心的"提醒"类通知，带"查看更新""忽略此版本"按钮。
//! 未打包的程序需要先在注册表登记应用身份（AUMID）才能发通知；失败时由调用方回退到托盘气泡。
use mousetroke_core::i18n::{keys, t, tf};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use windows::core::{Interface, HSTRING};
use windows::Data::Xml::Dom::XmlDocument;
use windows::Foundation::TypedEventHandler;
use windows::UI::Notifications::{
    ToastActivatedEventArgs, ToastDismissalReason, ToastDismissedEventArgs, ToastNotification, ToastNotificationManager,
};
use windows::Win32::System::Registry::{RegSetKeyValueW, HKEY_CURRENT_USER, REG_SZ};

const AUMID: &str = "io.github.eiaao.mousetroke";
/// 提醒类通知会一直停留，到时从屏幕收起。
const ON_SCREEN: Duration = Duration::from_secs(60);
/// 通知中心副本等待用户处理的检查间隔。
const KEEP_ALIVE_POLL: Duration = Duration::from_secs(5);

/// 通知按钮的结果，交给托盘线程处理。
pub enum Choice {
    View,
    Ignore(String),
}

fn register() -> windows::core::Result<()> {
    let key = HSTRING::from(format!("Software\\Classes\\AppUserModelId\\{AUMID}"));
    let name: Vec<u16> = "Mousetroke".encode_utf16().chain([0]).collect();
    unsafe {
        RegSetKeyValueW(
            HKEY_CURRENT_USER,
            &key,
            &HSTRING::from("DisplayName"),
            REG_SZ.0,
            Some(name.as_ptr().cast()),
            (name.len() * 2) as u32,
        )
        .ok()
    }
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn xml(version: &str) -> String {
    format!(
        r#"<toast scenario="reminder" launch="view"><visual><binding template="ToastGeneric"><text>Mousetroke</text><text>{body}</text></binding></visual><actions><action content="{view}" arguments="view" activationType="foreground"/><action content="{ignore}" arguments="ignore" activationType="foreground"/></actions><audio src="ms-winsoundevent:Notification.Default"/></toast>"#,
        body = escape(&tf(keys::UPDATE_AVAILABLE, &[("version", version)])),
        view = escape(t(keys::UI_UPDATE_VIEW)),
        ignore = escape(t(keys::UI_UPDATE_IGNORE)),
    )
}

type OnChoice = std::sync::Arc<dyn Fn(Choice) + Send + Sync>;

/// 创建通知并挂上按钮回调；用户点过按钮或手动关闭后 `handled` 置位。
fn create(version: &str, on_choice: &OnChoice, handled: &Arc<AtomicBool>) -> windows::core::Result<ToastNotification> {
    let doc = XmlDocument::new()?;
    doc.LoadXml(&HSTRING::from(xml(version)))?;
    let toast = ToastNotification::CreateToastNotification(&doc)?;
    let (version, on_choice, done) = (version.to_string(), on_choice.clone(), handled.clone());
    toast.Activated(&TypedEventHandler::<ToastNotification, windows::core::IInspectable>::new(move |_, args| {
        done.store(true, Ordering::Release);
        let arguments = args.as_ref().and_then(|a| a.cast::<ToastActivatedEventArgs>().ok()).and_then(|a| a.Arguments().ok());
        on_choice(if arguments.is_some_and(|a| a == "ignore") { Choice::Ignore(version.clone()) } else { Choice::View });
        Ok(())
    }))?;
    let done = handled.clone();
    toast.Dismissed(&TypedEventHandler::<ToastNotification, ToastDismissedEventArgs>::new(move |_, args| {
        if args.as_ref().and_then(|a| a.Reason().ok()) == Some(ToastDismissalReason::UserCanceled) {
            done.store(true, Ordering::Release);
        }
        Ok(())
    }))?;
    Ok(toast)
}

/// 发出新版本通知；`on_choice` 在通知线程上调用。
/// 提醒类通知会一直停留：60 秒后从屏幕收起（收起会连同通知中心记录一起删除），
/// 用户尚未处理时再静默放一份到通知中心。
pub fn show_update(version: &str, on_choice: impl Fn(Choice) + Send + Sync + 'static) -> windows::core::Result<()> {
    register()?;
    let on_choice: OnChoice = Arc::new(on_choice);
    let handled = Arc::new(AtomicBool::new(false));
    let toast = create(version, &on_choice, &handled)?;
    let notifier = ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(AUMID))?;
    notifier.Show(&toast)?;
    let version = version.to_string();
    std::thread::spawn(move || {
        std::thread::sleep(ON_SCREEN);
        if handled.load(Ordering::Acquire) {
            return;
        }
        let _ = notifier.Hide(&toast);
        let quiet = create(&version, &on_choice, &handled).and_then(|copy| {
            copy.SetSuppressPopup(true)?;
            notifier.Show(&copy)?;
            Ok(copy)
        });
        match quiet {
            // 按钮回调挂在通知对象上：未注册 COM 激活器时，对象释放后从通知中心点击不会回调本进程，
            // 所以持有副本直到用户处理（点击或关闭），进程退出时随之结束。
            Ok(_copy) => {
                while !handled.load(Ordering::Acquire) {
                    std::thread::sleep(KEEP_ALIVE_POLL);
                }
            }
            Err(e) => crate::log!("通知转入通知中心失败: {e}"),
        }
    });
    Ok(())
}

/// 忽略此版本：写入共用状态文件，菜单提示随后刷新。
/// 旧通知只在它的版本仍是最新版本时生效，不覆盖对更新版本的忽略。
pub fn ignore(config: Option<PathBuf>, version: String) {
    let Some(config) = config else { return };
    let path = mousetroke_core::update::state_path(&config);
    if mousetroke_core::update::UpdateState::update(&path, |s| {
        if s.latest.as_ref() == Some(&version) {
            s.ignored = Some(version);
        }
    }).is_err() {
        crate::log!("忽略版本写入失败");
    }
    crate::update::wake();
}
