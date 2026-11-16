//! 同一登录会话内的推送控制；通知 object 是目标 PID，不读写请求文件。
use std::cell::Cell;
use std::io;
use std::panic::{catch_unwind, AssertUnwindSafe};

use objc2::{define_class, msg_send, rc::Retained, sel, MainThreadMarker, MainThreadOnly};
use objc2_foundation::{
    ns_string, NSDistributedNotificationCenter, NSNotification, NSNotificationSuspensionBehavior,
    NSObject, NSObjectProtocol, NSString,
};

#[derive(Clone, Copy)]
pub(crate) enum Request {
    Accessibility = 1,
    InputMonitoring = 2,
    Restart = 4,
    Quit = 8,
}

impl Request {
    fn name(self) -> &'static NSString {
        match self {
            Self::Accessibility => ns_string!("io.github.eiaao.mousetroke.request.accessibility"),
            Self::InputMonitoring => {
                ns_string!("io.github.eiaao.mousetroke.request.input-monitoring")
            }
            Self::Restart => ns_string!("io.github.eiaao.mousetroke.request.restart"),
            Self::Quit => ns_string!("io.github.eiaao.mousetroke.request.quit"),
        }
    }
}
const REQUESTS: [Request; 4] = [
    Request::Accessibility,
    Request::InputMonitoring,
    Request::Restart,
    Request::Quit,
];
thread_local! {
    static PENDING: Cell<u8> = const { Cell::new(0) };
    /// 设置进程最近一次上报的录制画布（外层 Some 表示有未处理的更新）。
    static CANVAS: Cell<Option<Option<Rect>>> = const { Cell::new(None) };
}

/// 屏幕矩形：左、上、右、下（全局坐标，单位为点，与事件位置一致）。
pub type Rect = (i32, i32, i32, i32);

/// 录制画布通知：object 为 `"<引擎 PID>:<左>,<上>,<右>,<下>"`，矩形为空表示清除。
/// 矩形放在 object 里而不是 userInfo，接收端仍先按 PID 核验。
fn canvas_name() -> &'static NSString {
    ns_string!("io.github.eiaao.mousetroke.canvas")
}

fn encode_canvas(pid: u32, rect: Option<Rect>) -> String {
    match rect {
        Some((l, t, r, b)) => format!("{pid}:{l},{t},{r},{b}"),
        None => format!("{pid}:"),
    }
}

fn decode_canvas(object: &str, pid: u32) -> Option<Option<Rect>> {
    let (owner, rect) = object.split_once(':')?;
    if owner.parse::<u32>().ok()? != pid {
        return None;
    }
    if rect.is_empty() {
        return Some(None);
    }
    let v: Vec<i32> = rect.split(',').map(str::parse).collect::<Result<_, _>>().ok()?;
    let &[l, t, r, b] = v.as_slice() else { return None };
    Some(Some((l, t, r, b)))
}

define_class!(
    // 安全：无额外 ivar，分布式通知仅在主线程交付，selector 签名与通知中心一致。
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    struct Receiver;
    unsafe impl NSObjectProtocol for Receiver {}
    impl Receiver {
        #[unsafe(method(receive:))]
        fn receive(&self, notification: &NSNotification) {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                let object = notification.object();
                let object = object.as_deref().and_then(|value| value.downcast_ref::<NSString>());
                if &*notification.name() == canvas_name() {
                    if let Some(rect) = object.and_then(|o| decode_canvas(&o.to_string(), std::process::id())) {
                        CANVAS.with(|slot| slot.set(Some(rect)));
                        crate::app::wake();
                    }
                    return;
                }
                // 注册时也按 object 过滤；二次核验避免旧 PID 通知影响新实例。
                let pid = NSString::from_str(&std::process::id().to_string());
                if object != Some(&*pid) {
                    return;
                }
                for request in REQUESTS {
                    if &*notification.name() == request.name() {
                        PENDING.with(|pending| pending.set(pending.get() | request as u8));
                        crate::app::wake();
                    }
                }
            }));
        }
    }
);

pub(crate) struct Observer(Retained<Receiver>);
impl Observer {
    pub(crate) fn new(mtm: MainThreadMarker) -> Self {
        // 安全：NSObject 的指定初始化器，接收对象存活到注销完成。
        let receiver: Retained<Receiver> = unsafe { msg_send![Receiver::alloc(mtm), init] };
        let center = NSDistributedNotificationCenter::defaultCenter();
        let pid = NSString::from_str(&std::process::id().to_string());
        for request in REQUESTS {
            unsafe {
                center.addObserver_selector_name_object_suspensionBehavior(
                    &receiver,
                    sel!(receive:),
                    Some(request.name()),
                    Some(&pid),
                    NSNotificationSuspensionBehavior::DeliverImmediately,
                );
            }
        }
        // 画布通知的 object 带矩形，无法按 PID 精确注册，收到后再核验。
        unsafe {
            center.addObserver_selector_name_object_suspensionBehavior(
                &receiver,
                sel!(receive:),
                Some(canvas_name()),
                None,
                NSNotificationSuspensionBehavior::DeliverImmediately,
            );
        }
        Self(receiver)
    }
}
impl Drop for Observer {
    fn drop(&mut self) {
        // 安全：先注销本对象全部通知，再释放接收对象。
        unsafe { NSDistributedNotificationCenter::defaultCenter().removeObserver(&self.0) };
    }
}

pub(crate) fn send(pid: u32, request: Request) -> io::Result<()> {
    if crate::instance::engine_pid() != Some(pid) {
        return Err(io::ErrorKind::NotConnected.into());
    }
    let pid = NSString::from_str(&pid.to_string());
    // 安全：无 userInfo；名称和 object 都是可序列化字符串，只发给当前登录会话。
    unsafe {
        NSDistributedNotificationCenter::defaultCenter()
            .postNotificationName_object_userInfo_deliverImmediately(
                request.name(),
                Some(&pid),
                None,
                true,
            );
    }
    Ok(())
}

/// 设置进程调用：把录制画布位置发给当前引擎。
pub(crate) fn send_canvas(pid: u32, rect: Option<Rect>) -> io::Result<()> {
    if crate::instance::engine_pid() != Some(pid) {
        return Err(io::ErrorKind::NotConnected.into());
    }
    let object = NSString::from_str(&encode_canvas(pid, rect));
    // 安全：无 userInfo；名称和 object 都是字符串，只发给当前登录会话。
    unsafe {
        NSDistributedNotificationCenter::defaultCenter()
            .postNotificationName_object_userInfo_deliverImmediately(canvas_name(), Some(&object), None, true);
    }
    Ok(())
}

/// 引擎主循环取走未处理的画布更新。
pub(crate) fn take_canvas() -> Option<Option<Rect>> {
    CANVAS.with(Cell::take)
}

pub(crate) fn take(request: Request) -> bool {
    PENDING.with(|pending| {
        let bits = pending.get();
        pending.set(bits & !(request as u8));
        bits & request as u8 != 0
    })
}

#[cfg(test)]
mod tests {
    use super::{decode_canvas, encode_canvas};

    #[test]
    fn canvas_notification_round_trips_and_checks_pid() {
        for rect in [Some((10, 20, 300, 400)), Some((-1440, -100, -1000, 200)), None] {
            assert_eq!(decode_canvas(&encode_canvas(42, rect), 42), Some(rect));
        }
        assert_eq!(decode_canvas(&encode_canvas(42, Some((1, 2, 3, 4))), 7), None);
        assert_eq!(decode_canvas("42:1,2,3", 42), None);
        assert_eq!(decode_canvas("garbage", 42), None);
    }
}
