//! 趣味特效的 macOS 播放器：在鼠标所在屏幕上盖一个透明、忽略鼠标的置顶窗口。
//! 工作线程用 `mousetroke-fx` 按逻辑点分辨率绘制，最新一帧放进“信箱”，主线程取出做成 CGImage
//! 设为图层内容；主线程忙时直接丢弃旧帧。不截屏：炸弹和黑洞使用不动桌面的版本，免去屏幕录制权限。
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use block2::RcBlock;
use mousetroke_core::presets::native::FunEffect;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{msg_send, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{NSBackingStoreType, NSColor, NSScreen, NSScreenSaverWindowLevel, NSView, NSWindow, NSWindowCollectionBehavior, NSWindowStyleMask};
use objc2_core_foundation::{CFData, CGPoint, CGRect};
use objc2_core_graphics::{CGBitmapInfo, CGColorRenderingIntent, CGColorSpace, CGDataProvider, CGImage, CGImageAlphaInfo, CGImageByteOrderInfo};
use objc2_foundation::NSOperationQueue;
use objc2_quartz_core::{CALayer, CATransaction};

static PLAYING: AtomicBool = AtomicBool::new(false);
const FRAME: Duration = Duration::from_micros(16_667);
const MAX_DURATION: Duration = Duration::from_secs(15);

thread_local! {
    /// 主线程上当前特效的窗口与图层。
    static CURRENT: RefCell<Option<(Retained<NSWindow>, Retained<CALayer>)>> = const { RefCell::new(None) };
}

/// 最新一帧（像素、宽、高）；主线程取走后置空。
type Mailbox = Arc<Mutex<Option<(Vec<u32>, usize, usize)>>>;

/// 在 at（Quartz 全局坐标，逻辑点；None 为当前鼠标位置）所在屏幕上播放；任意线程可调用，立即返回。
/// 正在播放时忽略本次。
pub fn play(effect: FunEffect, at: Option<(i32, i32)>) {
    if PLAYING.swap(true, Ordering::AcqRel) {
        crate::log!("特效 {} 正在播放，忽略新的请求", effect.name());
        return;
    }
    let spawned = std::thread::Builder::new().name("fx".into()).spawn(move || {
        struct Reset;
        impl Drop for Reset {
            fn drop(&mut self) {
                // 无论成功与否都关窗、复位。关窗排进主队列、不等待也不会被取消：
                // 主线程卡住时窗口晚点关；下一次播放的 open 也会先关掉旧窗口。
                let block = RcBlock::new(|| objc2::rc::autoreleasepool(|_| close()));
                // 安全：块不捕获任何数据，只在主线程执行一次。
                unsafe { NSOperationQueue::mainQueue().addOperationWithBlock(&block) };
                PLAYING.store(false, Ordering::Release);
            }
        }
        let _reset = Reset;
        if let Err(e) = run(effect, at) {
            crate::log!("特效 {} 播放失败: {e}", effect.name());
        }
    });
    if spawned.is_err() {
        PLAYING.store(false, Ordering::Release);
    }
}

fn run(effect: FunEffect, at: Option<(i32, i32)>) -> Result<(), String> {
    let (w, h, local) = crate::preset_actions::on_main(move || open(at))?;
    let mut fx = mousetroke_fx::create(effect.name(), w, h, local, None).ok_or("Unknown effect")?;
    let mailbox: Mailbox = Arc::new(Mutex::new(None));
    let mut px = vec![0u32; (w * h) as usize];
    let start = Instant::now();
    let mut last = start;
    loop {
        let now = Instant::now();
        let t = (now - start).as_secs_f32();
        let dt = (now - last).as_secs_f32().min(0.05);
        last = now;
        let more = fx.frame(&mut mousetroke_fx::Canvas { px: &mut px, w, h }, t, dt);
        post(&mailbox, &px, w as usize, h as usize);
        if !more || now - start > MAX_DURATION {
            return Ok(());
        }
        if let Some(rest) = FRAME.checked_sub(now.elapsed()) {
            std::thread::sleep(rest);
        }
    }
}

/// 放进信箱；信箱原本为空时才排一个主线程任务（已有任务会取到这帧）。
fn post(mailbox: &Mailbox, px: &[u32], w: usize, h: usize) {
    let Ok(mut slot) = mailbox.lock() else { return };
    let queued = slot.is_some();
    *slot = Some((px.to_vec(), w, h));
    drop(slot);
    if queued {
        return;
    }
    let mailbox = Arc::clone(mailbox);
    let block = RcBlock::new(move || {
        let frame = mailbox.lock().ok().and_then(|mut slot| slot.take());
        if let Some((px, w, h)) = frame {
            objc2::rc::autoreleasepool(|_| show(&px, w, h));
        }
    });
    // 安全：捕获的 Arc<Mutex<..>> 可跨线程；块只在主线程执行一次。
    unsafe { NSOperationQueue::mainQueue().addOperationWithBlock(&block) };
}

/// 主线程：在 at 所在屏幕上建窗口，返回绘制尺寸（逻辑点）与屏幕内坐标。
fn open(at: Option<(i32, i32)>) -> Result<(i32, i32, (f32, f32)), String> {
    let mtm = MainThreadMarker::new().ok_or("Effect window requires the main thread")?;
    close();
    let screens = NSScreen::screens(mtm);
    let primary = screens.firstObject().ok_or("No screens available")?;
    let primary_height = primary.frame().size.height;
    let (x, y) = match at {
        Some((x, y)) => (x as f64, y as f64),
        None => crate::host::cursor_pos().map(|p| (p.x, p.y)).unwrap_or((0.0, 0.0)),
    };
    let screen = screens
        .iter()
        .find(|s| {
            let r = crate::overlay::rect(s.frame()).quartz(primary_height);
            x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h
        })
        .unwrap_or(primary);
    let frame = screen.frame();
    let quartz = crate::overlay::rect(frame).quartz(primary_height);
    let (w, h) = (frame.size.width.round() as i32, frame.size.height.round() as i32);
    if w <= 0 || h <= 0 {
        return Err("Empty screen".into());
    }
    // 安全：NSWindow 指定初始化器，参数为有效的屏幕点矩形。
    let window: Retained<NSWindow> = unsafe {
        msg_send![NSWindow::alloc(mtm), initWithContentRect: frame,
            styleMask: NSWindowStyleMask::Borderless, backing: NSBackingStoreType::Buffered, defer: false]
    };
    // 安全：窗口由 Retained 管理，close 时不能再由 AppKit 额外 release。
    unsafe { window.setReleasedWhenClosed(false) };
    window.setOpaque(false);
    window.setBackgroundColor(Some(&NSColor::clearColor()));
    window.setIgnoresMouseEvents(true);
    window.setHasShadow(false);
    window.setHidesOnDeactivate(false);
    window.setLevel(NSScreenSaverWindowLevel);
    window.setCollectionBehavior(
        NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::FullScreenAuxiliary
            | NSWindowCollectionBehavior::Stationary
            | NSWindowCollectionBehavior::IgnoresCycle,
    );
    let bounds = CGRect::new(CGPoint::new(0.0, 0.0), frame.size);
    let view = NSView::initWithFrame(NSView::alloc(mtm), bounds);
    let layer = CALayer::layer();
    layer.setFrame(bounds);
    view.setLayer(Some(&layer));
    view.setWantsLayer(true);
    window.setContentView(Some(&view));
    CURRENT.with(|c| *c.borrow_mut() = Some((window, layer)));
    Ok((w, h, ((x - quartz.x) as f32, (y - quartz.y) as f32)))
}

/// 主线程：把一帧设为图层内容，首帧时显示窗口。
fn show(px: &[u32], w: usize, h: usize) {
    CURRENT.with(|c| {
        let c = c.borrow();
        let Some((window, layer)) = c.as_ref() else { return };
        let Some(image) = image(px, w, h) else { return };
        CATransaction::begin();
        CATransaction::setDisableActions(true);
        // 安全：CGImage 与 id 免费桥接，图层持有自己的引用。
        unsafe {
            let obj: &AnyObject = &*(&*image as *const CGImage as *const AnyObject);
            layer.setContents(Some(obj));
        }
        CATransaction::commit();
        if !window.isVisible() {
            window.orderFrontRegardless();
        }
    });
}

/// 预乘 ARGB（小端内存为 BGRA）→ CGImage。
fn image(px: &[u32], w: usize, h: usize) -> Option<objc2_core_foundation::CFRetained<CGImage>> {
    // 安全：u32 切片按字节重新解释，长度为 4 倍，生命周期不超过本函数（CFData 会复制）。
    let bytes = unsafe { std::slice::from_raw_parts(px.as_ptr().cast::<u8>(), std::mem::size_of_val(px)) };
    let data = CFData::from_bytes(bytes);
    let provider = CGDataProvider::with_cf_data(Some(&data))?;
    let space = CGColorSpace::new_device_rgb()?;
    let info = CGBitmapInfo::from_bits_retain(CGImageAlphaInfo::PremultipliedFirst.0 | CGImageByteOrderInfo::Order32Little.0);
    // 安全：尺寸与数据长度一致；decode 为空指针。
    unsafe { CGImage::new(w, h, 8, 32, w * 4, Some(&space), info, Some(&provider), std::ptr::null(), false, CGColorRenderingIntent::RenderingIntentDefault) }
}

/// 主线程：关掉当前特效窗口。
fn close() {
    if let Some((window, _)) = CURRENT.with(|c| c.borrow_mut().take()) {
        window.orderOut(None);
        window.close();
    }
}
