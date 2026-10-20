//! 唯一的主线程状态；输入回调只借用 ENGINE，不借用包含 AppKit 对象的 Runtime。
use std::cell::{Cell, RefCell};
use std::ffi::OsStr;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use block2::RcBlock;
use mousetroke_core::{
    i18n::{keys, t},
    schema::{default_config_path, Config},
};
use mousetroke_engine::{Engine, Reload};
use objc2::{
    rc::{autoreleasepool, Retained},
    runtime::ProtocolObject,
    MainThreadMarker,
};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy,
    NSApplicationDidChangeScreenParametersNotification, NSEvent, NSEventModifierFlags, NSEventType,
};
use objc2_foundation::{
    NSDate, NSNotificationCenter, NSObjectProtocol, NSPoint, NSRunLoop, NSRunLoopCommonModes,
    NSTimer,
};

use crate::{
    events, host, input, instance, log,
    menu::{self, Menu},
    permissions::{self, Permissions},
    selftest::SelfTest,
};

thread_local! {
    static RUNTIME: RefCell<Option<Runtime>> = const { RefCell::new(None) };
    static QUIT: Cell<Option<i32>> = const { Cell::new(None) };
    static TIMER: RefCell<Option<Retained<NSTimer>>> = const { RefCell::new(None) };
    /// 最近一次输入或后台结果后保持 10ms 节奏的截止时间。
    static AWAKE_UNTIL: Cell<Option<Instant>> = const { Cell::new(None) };
    static SLOW: Cell<bool> = const { Cell::new(false) };
}

/// 有活动时 10ms 一拍；空闲后每 250ms 一拍，只做权限、配置、提示等低频工作。
const FAST: f64 = 0.01;
const SLOW_INTERVAL: Duration = Duration::from_millis(250);
const LINGER: Duration = Duration::from_secs(1);

/// 主线程：输入事件或后台结果到达时恢复快节奏，并至少保持 LINGER。
pub(crate) fn wake() {
    AWAKE_UNTIL.with(|until| until.set(Some(Instant::now() + LINGER)));
    if SLOW.with(|slow| slow.replace(false)) {
        crate::targeting::invalidate();
        TIMER.with(|timer| {
            if let Some(timer) = timer.borrow().as_ref() {
                timer.setFireDate(&NSDate::now());
            }
        });
    }
}

/// 任意线程：把 wake 交给主队列执行，不直接触碰主线程状态。
pub(crate) fn wake_from_worker() {
    extern "C" fn on_main(_: *mut std::ffi::c_void) {
        wake();
    }
    extern "C" {
        static _dispatch_main_q: u8;
        fn dispatch_async_f(
            queue: *const u8,
            context: *mut std::ffi::c_void,
            work: extern "C" fn(*mut std::ffi::c_void),
        );
    }
    // 安全：主队列是进程生命周期内的全局对象；回调不使用 context。
    unsafe { dispatch_async_f(&raw const _dispatch_main_q, std::ptr::null_mut(), on_main) };
}

pub(crate) fn awake() -> bool {
    !SLOW.with(Cell::get)
}

/// 本拍结束后若无待办工作，把下一拍推迟到 250ms 后。
fn settle_rhythm(now: Instant, busy: bool) {
    if busy || AWAKE_UNTIL.with(Cell::get).is_some_and(|until| now < until) {
        return;
    }
    SLOW.with(|slow| slow.set(true));
    TIMER.with(|timer| {
        if let Some(timer) = timer.borrow().as_ref() {
            timer.setFireDate(&NSDate::dateWithTimeIntervalSinceNow(SLOW_INTERVAL.as_secs_f64()));
        }
    });
}

static DISPLAY_CHANGED: AtomicBool = AtomicBool::new(false);
static RESTART: AtomicBool = AtomicBool::new(false);

struct DisplayObserver(Retained<ProtocolObject<dyn NSObjectProtocol>>);
impl DisplayObserver {
    fn new() -> Self {
        // 不捕获 AppKit 对象；通知即使从别的线程到达，也只设置标志，下一轮主线程再重建。
        let block = RcBlock::new(
            |_notification: std::ptr::NonNull<objc2_foundation::NSNotification>| {
                DISPLAY_CHANGED.store(true, Ordering::Relaxed);
            },
        );
        // 安全：系统通知名有效，object/queue 均为空，block 可在任意发送线程安全调用。
        Self(unsafe {
            NSNotificationCenter::defaultCenter().addObserverForName_object_queue_usingBlock(
                Some(NSApplicationDidChangeScreenParametersNotification),
                None,
                None,
                &block,
            )
        })
    }
}
impl Drop for DisplayObserver {
    fn drop(&mut self) {
        // 安全：移除本实例注册并保留的 observer token，先注销再释放所有权。
        let token: &ProtocolObject<dyn NSObjectProtocol> = &self.0;
        unsafe { NSNotificationCenter::defaultCenter().removeObserver(token.as_ref()) };
    }
}

pub(crate) use mousetroke_engine::with_engine;
/// 启动与热加载成功后同步平台侧设置：菜单栏、暂停热键、自动更新、触发角冲突提示。
fn apply_settings(config: &Config, selftest: bool) {
    menu::set_visible(config.settings.show_tray);
    events::set_pause_hotkey(&config.settings.pause_hotkey);
    crate::update::set_automatic(config.settings.update_check);
    if !selftest {
        crate::hot_corners::check(config);
    }
}

pub(crate) fn toggle_pause() {
    if let Some(paused) = with_engine(|engine| {
        engine.toggle_paused();
        engine.is_paused()
    }) {
        crate::log!("手势暂停状态: {paused}");
        menu::request_refresh();
    }
}
pub(crate) fn request_quit(code: i32) {
    QUIT.with(|slot| slot.set(Some(code)));
}

struct Runtime {
    menu: Menu,
    permissions: Permissions,
    next_permissions: Instant,
    next_engine_tick: Instant,
    tap_failed: bool,
    selftest: Option<SelfTest>,
    _displays: DisplayObserver,
    update_version: Option<String>,
}

impl Runtime {
    fn tick(&mut self, mtm: MainThreadMarker) {
        let now = Instant::now();
        if events::take_fault() {
            with_engine(|engine| engine.set_paused(true));
            menu::request_refresh();
        }
        if DISPLAY_CHANGED.swap(false, Ordering::Relaxed) {
            host::refresh_screen(mtm);
            with_engine(Engine::on_display_change);
        }
        host::drain_replays();
        host::refresh_target();
        events::settle_edges(now);
        input::tick();
        with_engine(Engine::overlay_tick);
        if now >= self.next_engine_tick {
            self.next_engine_tick = now + Duration::from_millis(250);
            with_engine(Engine::toast_tick);
        }
        if now >= self.next_permissions {
            self.next_permissions = now + Duration::from_secs(2);
            let permissions = permissions::check();
            if permissions != self.permissions {
                if !permissions.ready() {
                    // 系统可能先禁用 tap 再让轮询发现撤权；无论 tap 是否仍 enabled 都取消旧捕获。
                    with_engine(Engine::cancel_capture);
                    events::remove();
                    self.tap_failed = false;
                }
                crate::log!(
                    "权限状态: 辅助功能={}, 输入监控={}",
                    permissions.accessibility,
                    permissions.input_monitoring
                );
                self.permissions = permissions;
                menu::request_refresh();
            }
            if self.permissions.ready() && !events::active() {
                let installed = events::install();
                if !installed && !self.tap_failed {
                    crate::log!("创建事件 tap 失败；两秒后重试，请检查系统权限与图形登录会话");
                }
                self.tap_failed = !installed;
                menu::request_refresh();
            }
            instance::publish_input_state(events::active());
            if host::refresh_screen(mtm) {
                with_engine(Engine::on_display_change);
            }
            if let Some(Some(reload)) = with_engine(Engine::poll_config) {
                match reload {
                    Reload::Applied {
                        scopes, bindings, ..
                    } => {
                        crate::log!("配置已更新: scopes={scopes}, bindings={bindings}");
                        let selftest = self.selftest.is_some();
                        with_engine(|engine| apply_settings(engine.config(), selftest));
                        mousetroke_engine::push_toast(t(keys::TOAST_CONFIG_UPDATED).into(), 1200);
                    }
                    Reload::Failed(error) => {
                        crate::log!("配置更新失败: {error}");
                        mousetroke_engine::push_toast(t(keys::TRAY_RELOAD_FAILED).into(), 3000);
                    }
                }
            }
            if instance::take_quit_request() {
                crate::log!("设置页请求更新退出引擎");
                request_quit(0);
            } else if instance::take_restart_request() {
                crate::log!("设置页请求重启引擎");
                RESTART.store(true, Ordering::Relaxed);
                request_quit(0);
            }
        }
        if let Some(test) = self.selftest.as_mut() {
            test.tick(events::active());
        }
        if let Some((version, ack)) = crate::update::take_notice() {
            crate::log!("有新版本：{version}");
            self.update_version = Some(version);
            let _ = ack.send(true);
            menu::request_refresh();
        }
        if menu::take_refresh() {
            self.menu.set_update(self.update_version.as_deref());
            let paused = with_engine(|engine| engine.is_paused()).unwrap_or(false);
            self.menu
                .refresh(mtm, self.permissions, events::active(), paused);
        }
        if QUIT.with(Cell::get).is_some() {
            stop(mtm);
            return;
        }
        let busy = self.selftest.is_some()
            || crate::overlay::busy()
            || input::busy()
            || crate::actions::busy()
            || events::settle_pending();
        settle_rhythm(now, busy);
    }
}

/// stop 不会唤醒 AppKit 正在等待的下一件事件，投递 application-defined 事件使 run 返回。
fn stop(mtm: MainThreadMarker) {
    let app = NSApplication::sharedApplication(mtm);
    app.stop(None);
    // ApplicationDefined 事件不携带窗口或上下文，所有标量均按 AppKit 文档传入。
    let wake = NSEvent::otherEventWithType_location_modifierFlags_timestamp_windowNumber_context_subtype_data1_data2(
        NSEventType::ApplicationDefined,
        NSPoint::new(0.0, 0.0),
        NSEventModifierFlags::empty(),
        0.0,
        0,
        None,
        0,
        0,
        0,
    );
    if let Some(wake) = wake {
        app.postEvent_atStart(&wake, true);
    } else {
        crate::log!("创建退出唤醒事件失败");
    }
}

pub fn run() -> Result<i32, String> {
    run_with_config(None)
}

pub fn run_with_config(config: Option<std::path::PathBuf>) -> Result<i32, String> {
    let mtm = MainThreadMarker::new().ok_or_else(|| t(keys::MAC_ERROR_MAIN_THREAD).to_string())?;
    log::init().map_err(|error| error.to_string())?;
    mousetroke_engine::log::set_observer(crate::selftest::observe);
    let result = autoreleasepool(|_| run_main(mtm, config));
    // 启动失败也写进日志：从 Finder/open 启动时看不到 stderr。
    if let Err(error) = &result {
        mousetroke_engine::log!("运行失败：{error}");
    }
    log::shutdown();
    result
}

fn run_main(mtm: MainThreadMarker, config_path: Option<std::path::PathBuf>) -> Result<i32, String> {
    let selftest = std::env::var_os("MOUSETROKE_SELFTEST").as_deref() == Some(OsStr::new("1"));
    // 安装对话框先于锁、配置写盘和任何权限相关初始化；重启的已安装副本不会被本实例阻挡。
    if !selftest && crate::first_run::handle_install(mtm) {
        return Ok(0);
    }
    let path = config_path.unwrap_or_else(default_config_path);
    crate::settings::initialize(path.clone());
    // 引擎单实例锁固定在应用目录，设置中的自定义配置不会绕过检测。
    let lock_path = default_config_path();
    let dir = lock_path
        .parent()
        .ok_or_else(|| t(keys::MAC_ERROR_STARTUP).to_string())?
        .to_path_buf();
    let Some(_lock) = instance::acquire(&dir).map_err(|error| error.to_string())? else {
        crate::log!("已有实例运行，退出第二个实例");
        return Ok(if selftest { 1 } else { 0 });
    };
    let first_run = !path.exists();
    let config = if selftest {
        let config = Config::builtin();
        if !path.exists() {
            config.save(&path)?;
        }
        config
    } else {
        let (config, saved) = Config::load_or_init(&path)?;
        if let Some(error) = saved {
            crate::log!("首次写入默认配置失败: {error}");
        }
        config
    };
    let expected = config
        .all_bindings()
        .find(|binding| binding.name_key.as_deref() == Some(keys::BUILTIN_GESTURE_CLOSE_TAB))
        .map(|binding| {
            (
                binding.name.clone(),
                match &binding.action {
                    mousetroke_core::schema::Action::KeySequence { keys, .. } => keys.clone(),
                    _ => String::new(),
                },
            )
        })
        .unwrap_or_default();
    let app = NSApplication::sharedApplication(mtm);
    // Info.plist 已声明 LSUIElement 时策略本来就是 Accessory，此调用会返回 NO（真机验证），忽略即可。
    let _ = app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
    host::refresh_target();
    host::refresh_screen(mtm);
    input::initialize(selftest);
    crate::lua::initialize();
    let displays = DisplayObserver::new();
    let automatic_updates = config.settings.update_check;
    let restart_path = path.clone();
    let update_path = path.clone();
    let engine = Engine::new(
        config,
        (!selftest).then_some(path),
        Box::new(host::SystemHost),
    )
    .ok_or_else(|| t(keys::MAC_ERROR_ENGINE).to_string())?;
    apply_settings(engine.config(), selftest);
    mousetroke_engine::install(engine);
    if !selftest {
        crate::update::start_automatic(update_path, automatic_updates);
    }
    let permissions = permissions::check();
    if !selftest && (first_run || !permissions.ready()) {
        crate::settings::open_setup();
    }
    crate::log!(
        "权限状态: 辅助功能={}, 输入监控={}",
        permissions.accessibility,
        permissions.input_monitoring
    );
    let menu = Menu::new(mtm);
    let now = Instant::now();
    RUNTIME.with(|slot| {
        *slot.borrow_mut() = Some(Runtime {
            menu,
            permissions,
            next_permissions: now,
            next_engine_tick: now,
            tap_failed: false,
            selftest: selftest.then(|| SelfTest::new(expected.0, expected.1)),
            _displays: displays,
            update_version: None,
        })
    });
    let block = RcBlock::new(move |_timer: std::ptr::NonNull<NSTimer>| {
        // 计时器和 ObjC action 同样不能把 Rust panic 传播到系统调用栈。
        if catch_unwind(AssertUnwindSafe(|| {
            autoreleasepool(|_| {
                RUNTIME.with(|slot| {
                    if let Ok(mut slot) = slot.try_borrow_mut() {
                        if let Some(runtime) = slot.as_mut() {
                            runtime.tick(mtm);
                        }
                    }
                });
                crate::actions::tick();
                crate::lua::tick();
            })
        }))
        .is_err()
        {
            crate::log!("主线程计时器发生 panic，退出");
            request_quit(1);
            stop(mtm);
        }
    });
    // 安全：只在主 run loop common modes 调用 block；timer 在退出前失效，捕获的 mtm 也仅在主线程使用。
    let timer: Retained<NSTimer> =
        unsafe { NSTimer::timerWithTimeInterval_repeats_block(FAST, true, &block) };
    unsafe { NSRunLoop::mainRunLoop().addTimer_forMode(&timer, NSRunLoopCommonModes) };
    TIMER.with(|slot| *slot.borrow_mut() = Some(timer.clone()));
    wake();
    crate::log!("Mousetroke macOS M3b 启动");
    app.run();
    timer.invalidate();
    TIMER.with(|slot| slot.borrow_mut().take());
    crate::actions::shutdown();
    crate::lua::shutdown();
    events::remove();
    RUNTIME.with(|slot| {
        slot.borrow_mut().take();
    });
    drop(mousetroke_engine::uninstall());
    let code = QUIT.with(|slot| slot.take()).unwrap_or(0);
    crate::log!("Mousetroke 退出: {code}");
    if RESTART.swap(false, Ordering::Relaxed) {
        // 清理完成并显式释放 flock 后再启动；新进程运行引擎，设置窗口继续保留。
        drop(_lock);
        let exe = std::env::current_exe().map_err(|error| error.to_string())?;
        std::process::Command::new(exe).arg("run").arg("--config").arg(restart_path)
            .spawn().map_err(|error| error.to_string())?;
    }
    Ok(code)
}
