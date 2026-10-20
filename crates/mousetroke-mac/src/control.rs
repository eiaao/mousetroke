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
thread_local! { static PENDING: Cell<u8> = const { Cell::new(0) }; }

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
                // 注册时也按 object 过滤；二次核验避免旧 PID 通知影响新实例。
                let pid = NSString::from_str(&std::process::id().to_string());
                if notification.object().as_deref().and_then(|value| value.downcast_ref::<NSString>()) != Some(&*pid) {
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

pub(crate) fn take(request: Request) -> bool {
    PENDING.with(|pending| {
        let bits = pending.get();
        pending.set(bits & !(request as u8));
        bits & request as u8 != 0
    })
}
