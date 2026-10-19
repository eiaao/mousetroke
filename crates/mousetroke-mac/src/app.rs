//! 唯一的主线程状态；输入回调只借用 ENGINE，不借用包含 AppKit 对象的 Runtime。
use std::cell::{Cell, RefCell};
use std::ffi::OsStr;
use std::fs::OpenOptions;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::{Duration, Instant};

use block2::RcBlock;
use mousetroke_core::{
    i18n::{keys, t},
    schema::{default_config_path, Config},
};
use mousetroke_engine::{Engine, Reload};
use objc2::{
    rc::{autoreleasepool, Retained},
    MainThreadMarker,
};
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy, NSEvent, NSEventModifierFlags, NSEventType};
use objc2_foundation::{NSPoint, NSRunLoop, NSRunLoopCommonModes, NSTimer};

use crate::{
    events, host, instance, log,
    menu::{self, Menu},
    permissions::{self, Permissions},
    selftest::SelfTest,
};

thread_local! {
    static ENGINE: RefCell<Option<Engine>> = const { RefCell::new(None) };
    static RUNTIME: RefCell<Option<Runtime>> = const { RefCell::new(None) };
    static QUIT: Cell<Option<i32>> = const { Cell::new(None) };
}

pub(crate) fn with_engine<R>(f: impl FnOnce(&mut Engine) -> R) -> Option<R> {
    ENGINE.with(|slot| slot.try_borrow_mut().ok().and_then(|mut slot| slot.as_mut().map(f)))
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
}

impl Runtime {
    fn tick(&mut self, mtm: MainThreadMarker) {
        let now = Instant::now();
        if events::take_fault() {
            with_engine(|engine| engine.set_paused(true));
            menu::request_refresh();
        }
        host::drain_replays();
        if now >= self.next_engine_tick {
            self.next_engine_tick = now + Duration::from_millis(250);
            host::refresh_target();
            with_engine(Engine::toast_tick);
        }
        if now >= self.next_permissions {
            self.next_permissions = now + Duration::from_secs(2);
            let permissions = permissions::check(false);
            if permissions != self.permissions {
                if !permissions.ready() {
                    // 系统可能先禁用 tap 再让轮询发现撤权；无论 tap 是否仍 enabled 都取消旧捕获。
                    with_engine(|engine| {
                        let paused = engine.is_paused();
                        engine.set_paused(true);
                        engine.set_paused(paused);
                    });
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
            if host::refresh_screen(mtm) {
                with_engine(Engine::on_display_change);
            }
            if let Some(Some(reload)) = with_engine(Engine::poll_config) {
                match reload {
                    Reload::Applied { scopes, bindings, .. } => {
                        crate::log!("配置已更新: scopes={scopes}, bindings={bindings}")
                    }
                    Reload::Failed(error) => crate::log!("配置更新失败: {error}"),
                }
            }
        }
        if let Some(test) = self.selftest.as_mut() {
            test.tick(events::active());
        }
        if menu::take_refresh() {
            let paused = with_engine(|engine| engine.is_paused()).unwrap_or(false);
            self.menu.refresh(mtm, self.permissions, events::active(), paused);
        }
        if QUIT.with(Cell::get).is_some() {
            stop(mtm);
        }
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
    let mtm = MainThreadMarker::new().ok_or_else(|| t(keys::MAC_ERROR_MAIN_THREAD).to_string())?;
    log::init().map_err(|error| error.to_string())?;
    mousetroke_engine::log::set_sink(log::write);
    let result = autoreleasepool(|_| run_main(mtm));
    // 启动失败也写进日志：从 Finder/open 启动时看不到 stderr。
    if let Err(error) = &result {
        mousetroke_engine::log!("运行失败：{error}");
    }
    log::shutdown();
    result
}

fn run_main(mtm: MainThreadMarker) -> Result<i32, String> {
    let selftest = std::env::var_os("MOUSETROKE_SELFTEST").as_deref() == Some(OsStr::new("1"));
    let path = default_config_path();
    let dir = path
        .parent()
        .ok_or_else(|| t(keys::MAC_ERROR_STARTUP).to_string())?
        .to_path_buf();
    let Some(_lock) = instance::acquire(&dir).map_err(|error| error.to_string())? else {
        crate::log!("已有实例运行，退出第二个实例");
        return Ok(if selftest { 1 } else { 0 });
    };
    let config = if selftest {
        let config = Config::builtin();
        if !path.exists() {
            config.save(&path)?;
        }
        config
    } else {
        match Config::load_existing(&path)? {
            Some(config) => config,
            None => {
                let config = Config::builtin();
                config.save(&path)?;
                config
            }
        }
    };
    let expected = config
        .all_bindings()
        .find(|binding| binding.name_key.as_deref() == Some(keys::BUILTIN_GESTURE_CLOSE_TAB))
        .map(|binding| binding.name.clone())
        .unwrap_or_default();
    let app = NSApplication::sharedApplication(mtm);
    // Info.plist 已声明 LSUIElement 时策略本来就是 Accessory，此调用可能返回 NO；不作为启动失败。
    if !app.setActivationPolicy(NSApplicationActivationPolicy::Accessory) {
        mousetroke_engine::log!("{}", t(keys::MAC_ERROR_ACTIVATION));
    }
    host::refresh_target();
    host::refresh_screen(mtm);
    let engine = Engine::new(config, (!selftest).then_some(path), Box::new(host::SystemHost))
        .ok_or_else(|| t(keys::MAC_ERROR_ENGINE).to_string())?;
    ENGINE.with(|slot| *slot.borrow_mut() = Some(engine));
    let prompt = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dir.join("permission-prompted"))
    {
        Ok(_) => true,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => false,
        Err(error) => {
            crate::log!("无法记录首次权限请求: {error}");
            false
        }
    };
    let permissions = permissions::check(prompt);
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
            selftest: selftest.then(|| SelfTest::new(expected)),
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
    let timer: Retained<NSTimer> = unsafe { NSTimer::timerWithTimeInterval_repeats_block(0.01, true, &block) };
    unsafe { NSRunLoop::mainRunLoop().addTimer_forMode(&timer, NSRunLoopCommonModes) };
    crate::log!("Mousetroke macOS M1 启动");
    app.run();
    timer.invalidate();
    events::remove();
    RUNTIME.with(|slot| {
        slot.borrow_mut().take();
    });
    ENGINE.with(|slot| {
        slot.borrow_mut().take();
    });
    let code = QUIT.with(|slot| slot.take()).unwrap_or(0);
    crate::log!("Mousetroke 退出: {code}");
    Ok(code)
}
