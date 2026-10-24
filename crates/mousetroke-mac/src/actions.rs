//! 动作上下文和惰性变量；每轮只推进一件任务。AppKit 留在主线程，AX/进程等待交给工作线程。
use crate::{
    ax,
    helpers::{self, Window},
};
use block2::RcBlock;
use mousetroke_core::{
    i18n::{keys, t},
    schema::{Action, IfRunning, WindowOpKind},
    vars::{self, Vars},
};
use mousetroke_engine::Target;
use objc2::{rc::Retained, runtime::ProtocolObject, MainThreadMarker};
use objc2_app_kit::{
    NSApplicationActivationOptions, NSEvent, NSEventModifierFlags, NSEventType, NSPasteboard,
    NSPasteboardItem, NSPasteboardTypeString, NSPasteboardWriting, NSRunningApplication,
    NSWorkspace, NSWorkspaceOpenConfiguration,
};
use objc2_core_graphics::{CGEvent, CGEventField, CGEventTapLocation};
use objc2_foundation::{NSArray, NSError, NSPoint, NSString, NSURL};
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

#[derive(Clone)]
struct Request {
    action: Action,
    target: Option<Target>,
    vars: Vars,
    pos: (i32, i32),
    window: Option<Window>,
}
struct AxResult {
    request: Request,
    title: Option<String>,
    selected: Option<String>,
    needs_selected: bool,
}
struct Backup {
    request: Request,
    saved: Vec<Retained<NSPasteboardItem>>,
    todo: VecDeque<(
        Retained<NSPasteboardItem>,
        Retained<NSString>,
        Retained<NSPasteboardItem>,
    )>,
    count: isize,
    bytes: usize,
    deadline: Instant,
}
enum Pending {
    Ax(mpsc::Receiver<AxResult>),
    Window(mpsc::Receiver<()>),
    Backup(Box<Backup>),
    Keys {
        action: Action,
        deadline: Instant,
    },
    Clipboard {
        request: Box<Request>,
        saved: Vec<Retained<NSPasteboardItem>>,
        count: isize,
        deadline: Instant,
    },
}
#[derive(Default)]
struct State {
    queue: VecDeque<Request>,
    pending: Option<Pending>,
    dry_run: bool,
}
thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
    static BUSY: Cell<bool> = const { Cell::new(false) };
}
pub fn initialize(dry_run: bool) {
    STATE.with(|s| {
        *s.borrow_mut() = State {
            dry_run,
            ..State::default()
        }
    });
}
pub fn enqueue(action: Action, target: Option<Target>, vars: Vars, pos: (i32, i32)) {
    let window = target.as_ref().and_then(crate::targeting::window);
    enqueue_request(Request {
        action,
        target,
        vars,
        pos,
        window,
    });
}
fn enqueue_request(request: Request) {
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        if state.queue.len() < 64 {
            state.queue.push_back(request);
        } else {
            crate::log!("动作队列已满，跳过动作");
        }
    });
}
struct BusyGuard;
impl Drop for BusyGuard {
    fn drop(&mut self) {
        BUSY.with(|b| b.set(false));
    }
}
/// 仍有排队动作或等待中的后台结果时，主循环保持快节奏。
pub fn busy() -> bool {
    STATE.with(|s| {
        let s = s.borrow();
        !s.queue.is_empty() || s.pending.is_some()
    })
}

pub fn tick() {
    if BUSY.with(|b| b.replace(true)) {
        return;
    }
    let _guard = BusyGuard;
    let pending = STATE.with(|s| s.borrow_mut().pending.take());
    if let Some(pending) = pending {
        poll(pending);
        return;
    }
    if crate::input::busy() {
        return;
    }
    let (request, dry_run) = STATE.with(|s| {
        let mut s = s.borrow_mut();
        (s.queue.pop_front(), s.dry_run)
    });
    let Some(request) = request else {
        return;
    };
    let Some(action) = mousetroke_engine::prepare_action(request.action, mousetroke_core::caps::Platform::Macos, mousetroke_core::i18n::lang()) else { return };
    let request = Request { action, ..request };
    if dry_run {
        if matches!(request.action, Action::KeySequence { .. }) {
            crate::input::enqueue(request.action);
        } else {
            crate::log!(
                "自测动作 dry_run：{}",
                mousetroke_core::matcher::action_kind(&request.action)
            );
        }
        return;
    }
    let template = match &request.action {
        Action::Shell { script } => script.clone(),
        Action::OpenUrl { url_template, .. } => url_template.clone(),
        Action::LaunchApp { exe, args, .. } => {
            format!("{exe} {}", args.as_deref().unwrap_or_default())
        }
        _ => String::new(),
    };
    let mut request = request;
    if template.contains("%MS_CLIPBOARD_TEXT%") && !request.vars.contains_key("MS_CLIPBOARD_TEXT") {
        request
            .vars
            .insert("MS_CLIPBOARD_TEXT".into(), clipboard_text());
    }
    let needs_selected =
        template.contains("%MS_SELECTED_TEXT%") && !request.vars.contains_key("MS_SELECTED_TEXT");
    let needs_title = template.contains("%MS_WIN_NAME%");
    if needs_selected || needs_title {
        let (tx, rx) = mpsc::sync_channel(1);
        STATE.with(|s| s.borrow_mut().pending = Some(Pending::Ax(rx)));
        std::thread::spawn(move || {
            let title = needs_title.then(|| {
                request
                    .target
                    .as_ref()
                    .map(|t| ax::title(t.pid, request.window.as_ref(), request.pos))
                    .unwrap_or_default()
            });
            let selected = needs_selected.then(ax::selected_text).flatten();
            let _ = tx.send(AxResult {
                request,
                title,
                selected,
                needs_selected,
            });
        });
    } else {
        dispatch(request);
    }
}
fn poll(pending: Pending) {
    match pending {
        Pending::Keys { action, deadline } => {
            if Instant::now() >= deadline {
                crate::input::enqueue(action);
            } else {
                STATE.with(|s| s.borrow_mut().pending = Some(Pending::Keys { action, deadline }));
            }
        }
        Pending::Backup(backup) => backup_step(backup),
        Pending::Window(rx) => {
            if matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)) {
                STATE.with(|s| s.borrow_mut().pending = Some(Pending::Window(rx)));
            }
        }
        Pending::Ax(rx) => match rx.try_recv() {
            Ok(mut result) => {
                if let Some(title) = result.title {
                    if let Some(target) = &result.request.target {
                        crate::targeting::remember_title(target, title.clone());
                    }
                    result.request.vars.insert("MS_WIN_NAME".into(), title);
                }
                if result.needs_selected && result.selected.is_none() {
                    start_copy(result.request);
                } else {
                    if let Some(text) = result.selected {
                        result.request.vars.insert("MS_SELECTED_TEXT".into(), text);
                    }
                    dispatch(result.request);
                }
            }
            Err(mpsc::TryRecvError::Empty) => {
                STATE.with(|s| s.borrow_mut().pending = Some(Pending::Ax(rx)))
            }
            Err(mpsc::TryRecvError::Disconnected) => crate::log!("AX 工作线程提前结束，跳过动作"),
        },
        Pending::Clipboard {
            mut request,
            saved,
            count,
            deadline,
        } => {
            let pasteboard = NSPasteboard::generalPasteboard();
            let changed = pasteboard.changeCount() != count;
            if changed || Instant::now() >= deadline {
                let text = if changed {
                    clipboard_text()
                } else {
                    String::new()
                };
                // 未发生复制时剪贴板原封不动；发生复制则恢复所有已保存 item 的所有数据类型。
                if changed {
                    restore(&pasteboard, &saved);
                }
                request.vars.insert("MS_SELECTED_TEXT".into(), text);
                dispatch(*request);
            } else {
                STATE.with(|s| {
                    s.borrow_mut().pending = Some(Pending::Clipboard {
                        request,
                        saved,
                        count,
                        deadline,
                    })
                });
            }
        }
    }
}
fn clipboard_text() -> String {
    // 安全：系统提供的字符串 pasteboard 类型常量有效；AppKit 仅在主线程调用。
    unsafe { NSPasteboard::generalPasteboard().stringForType(NSPasteboardTypeString) }
        .map(|s| s.to_string())
        .unwrap_or_default()
}
fn start_copy(request: Request) {
    let pasteboard = NSPasteboard::generalPasteboard();
    let count = pasteboard.changeCount();
    let mut saved = Vec::new();
    let mut complete = true;
    let mut todo = VecDeque::new();
    if let Some(items) = pasteboard.pasteboardItems() {
        if items.len() > 64 {
            empty_selection(request);
            return;
        }
        for item in items.iter() {
            let copy = NSPasteboardItem::new();
            for kind in item.types().iter() {
                // 文件 promise 可能等待原程序兑现；不在主线程请求这类内容。
                if kind.to_string().to_ascii_lowercase().contains("promise") || todo.len() >= 256 {
                    complete = false;
                    break;
                }
                todo.push_back((item.clone(), kind, copy.clone()));
            }
            saved.push(copy);
        }
    } else if pasteboard.types().is_some_and(|types| !types.is_empty()) {
        // 旧式剪贴板有格式但没有可备份 item 时，不能把它当作空剪贴板清除。
        complete = false;
    }
    if !complete || pasteboard.changeCount() != count {
        empty_selection(request);
        return;
    }
    // 每轮仅复制一个格式，限制总大小和总用时，防止大型剪贴板占满一轮 run loop。
    STATE.with(|s| {
        s.borrow_mut().pending = Some(Pending::Backup(Box::new(Backup {
            request,
            saved,
            todo,
            count,
            bytes: 0,
            deadline: Instant::now() + Duration::from_millis(300),
        })))
    });
}
fn empty_selection(mut request: Request) {
    crate::log!("无法安全采集选区，使用空文本");
    request
        .vars
        .insert("MS_SELECTED_TEXT".into(), String::new());
    dispatch(request);
}
fn backup_step(mut backup: Box<Backup>) {
    let pasteboard = NSPasteboard::generalPasteboard();
    if pasteboard.changeCount() != backup.count || Instant::now() >= backup.deadline {
        empty_selection(backup.request);
        return;
    }
    if let Some((item, kind, copy)) = backup.todo.pop_front() {
        let Some(data) = item.dataForType(&kind) else {
            empty_selection(backup.request);
            return;
        };
        backup.bytes = backup.bytes.saturating_add(data.length());
        if backup.bytes > 8 * 1024 * 1024 || !copy.setData_forType(&data, &kind) {
            empty_selection(backup.request);
            return;
        }
        STATE.with(|s| s.borrow_mut().pending = Some(Pending::Backup(backup)));
    } else if crate::input::copy_selection() {
        STATE.with(|s| {
            s.borrow_mut().pending = Some(Pending::Clipboard {
                request: Box::new(backup.request),
                saved: backup.saved,
                count: backup.count,
                deadline: Instant::now() + Duration::from_millis(300),
            })
        });
    } else {
        empty_selection(backup.request);
    }
}
pub fn shutdown() {
    if let Some(Pending::Clipboard { saved, count, .. }) =
        STATE.with(|s| s.borrow_mut().pending.take())
    {
        let pasteboard = NSPasteboard::generalPasteboard();
        if pasteboard.changeCount() != count {
            restore(&pasteboard, &saved);
        }
    }
}
fn restore(pasteboard: &NSPasteboard, saved: &[Retained<NSPasteboardItem>]) {
    pasteboard.clearContents();
    if saved.is_empty() {
        return;
    }
    let objects: Vec<&ProtocolObject<dyn NSPasteboardWriting>> = saved
        .iter()
        .map(|item| ProtocolObject::from_ref(&**item))
        .collect();
    if !pasteboard.writeObjects(&NSArray::from_slice(&objects)) {
        crate::log!("恢复剪贴板失败");
    }
}
fn dispatch(request: Request) {
    let Request {
        action,
        target,
        vars,
        pos,
        window,
    } = &request;
    match action {
        Action::KeySequence {
            activate_target, ..
        } => {
            if *activate_target {
                if let Some(target) = target {
                    activate(target.pid);
                }
            }
            let mut action = action.clone();
            if let Action::KeySequence {
                activate_target, ..
            } = &mut action
            {
                *activate_target = false;
            }
            if *activate_target && target.is_some() {
                STATE.with(|s| {
                    s.borrow_mut().pending = Some(Pending::Keys {
                        action,
                        deadline: Instant::now() + Duration::from_millis(30),
                    })
                });
            } else {
                crate::input::enqueue(action);
            }
        }
        Action::WindowOp {
            op: WindowOpKind::ToggleTopmost,
            ..
        } => {
            mousetroke_engine::push_toast(t(keys::MAC_ACTION_TOPMOST_UNSUPPORTED).into(), 1600);
            crate::log!("macOS 没有公开的跨应用置顶 API");
        }
        Action::WindowOp { op, .. } => {
            if let Some(target) = target {
                let (pid, window, pos, op) = (target.pid, window.clone(), *pos, *op);
                let (tx, rx) = mpsc::sync_channel(1);
                STATE.with(|s| s.borrow_mut().pending = Some(Pending::Window(rx)));
                std::thread::spawn(move || {
                    ax::apply(pid, window.as_ref(), pos, op);
                    let _ = tx.send(());
                });
            }
        }
        Action::Shell { script } => {
            run_shell(vars::expand(script, vars));
        }
        Action::LaunchApp {
            exe,
            args,
            if_running,
            ..
        } => {
            if !mousetroke_core::caps::action_supported(action, mousetroke_core::caps::Platform::Macos) {
                // 规则见 core::caps：提权、按窗口查找、SendKeys 均按普通激活处理。
                crate::log!("启动应用含 macOS 不支持的选项，按普通启动/激活处理");
            }
            launch(
                &vars::expand(exe, vars),
                &vars::expand(args.as_deref().unwrap_or_default(), vars),
                *if_running,
            );
        }
        Action::OpenUrl {
            url_template,
            browser,
        } => open_url(&vars::expand_url(url_template, vars), browser.as_deref()),
        Action::SystemKey { key } => system_key(key),
        Action::Menu { items, .. } => {
            if let Some(mtm) = MainThreadMarker::new() {
                if let Some(action) = crate::menu::popup(items, *pos, mtm) {
                    enqueue_request(Request { action, ..request });
                }
            }
        }
        Action::Lua { code } => {
            let ctx = mousetroke_lua::LuaCtx::from_vars(
                *pos,
                target.as_ref().map_or(0, |t| t.wid),
                target.as_ref().map_or(0, |t| t.pid),
                vars,
            );
            // AX 标题只在工作线程读取；主线程不会等待 Lua 完成。
            let (code, window) = (code.clone(), window.clone());
            std::thread::spawn(move || {
                let mut ctx = ctx;
                if ctx.pid != 0 { ctx.title = ax::title(ctx.pid, window.as_ref(), ctx.pos); }
                crate::lua::run(code, ctx);
            });
        }
        // 队列消费时已展开；未识别的动作不进入平台执行器。
        Action::Preset { .. } | Action::Unknown(_) => {}
    }
}
#[allow(deprecated)] // macOS 11 起的兼容激活入口；新激活 API 不能覆盖最低系统版本。
pub(crate) fn activate(pid: u32) {
    if !NSRunningApplication::runningApplicationWithProcessIdentifier(pid as i32).is_some_and(
        |app| app.activateWithOptions(NSApplicationActivationOptions::ActivateIgnoringOtherApps),
    ) {
        crate::log!("激活目标应用失败");
    }
}
fn application_url(exe: &str) -> Option<Retained<NSURL>> {
    if exe.starts_with('/') {
        Some(NSURL::fileURLWithPath(&NSString::from_str(
            exe.trim_end_matches('/'),
        )))
    } else {
        NSWorkspace::sharedWorkspace()
            .URLForApplicationWithBundleIdentifier(&NSString::from_str(exe))
    }
}
fn completion() -> RcBlock<dyn Fn(*mut NSRunningApplication, *mut NSError)> {
    // 不读取错误描述，其中可能含命令参数或 URL 查询；仅记录完成状态。
    RcBlock::new(|app: *mut NSRunningApplication, error: *mut NSError| {
        crate::log!(
            "NSWorkspace 动作完成: success={}",
            error.is_null() && !app.is_null()
        );
    })
}
pub(crate) fn launch(exe: &str, args: &str, if_running: IfRunning) {
    let Some(args) = helpers::arguments(args) else {
        crate::log!("应用参数引号不完整，跳过动作");
        return;
    };
    let Some(url) = application_url(exe) else {
        crate::log!("找不到应用 bundle，跳过动作");
        return;
    };
    let config = NSWorkspaceOpenConfiguration::configuration();
    config.setActivates(true);
    config.setCreatesNewApplicationInstance(if_running == IfRunning::RunAgain);
    let strings: Vec<_> = args.iter().map(|a| NSString::from_str(a)).collect();
    config.setArguments(&NSArray::from_retained_slice(&strings));
    // 先统一交给 LaunchServices；无 bundle 的命令行可执行文件可能被拒绝，才在工作线程回退。
    let executable = (exe.starts_with('/') && !exe.trim_end_matches('/').ends_with(".app"))
        .then(|| exe.to_owned());
    let completed = RcBlock::new(move |app: *mut NSRunningApplication, error: *mut NSError| {
        if !error.is_null() {
            if let Some(exe) = &executable {
                let (exe, args) = (exe.clone(), args.clone());
                process(
                    move || {
                        let mut cmd = Command::new(exe);
                        cmd.args(args);
                        cmd
                    },
                    false,
                );
                return;
            }
        }
        let app = if error.is_null() { unsafe { app.as_ref() } } else { None };
        crate::log!("NSWorkspace 启动完成: success={}", app.is_some());
        // 协作式激活下，后台菜单栏程序发起的启动常停在后台；交给 open 再激活一次。
        if let Some(path) = app
            .filter(|app| if_running != IfRunning::RunAgain && !app.isActive())
            .and_then(|app| app.bundleURL())
            .and_then(|url| url.path())
        {
            let path = path.to_string();
            process(
                move || {
                    let mut cmd = Command::new("/usr/bin/open");
                    cmd.arg("-a").arg(path);
                    cmd
                },
                false,
            );
        }
    });
    NSWorkspace::sharedWorkspace().openApplicationAtURL_configuration_completionHandler(
        &url,
        &config,
        Some(&completed),
    );
}
pub(crate) fn open_url(text: &str, browser: Option<&str>) {
    let Some(url) = NSURL::URLWithString(&NSString::from_str(text)) else {
        crate::log!("URL 无效，跳过动作");
        return;
    };
    let workspace = NSWorkspace::sharedWorkspace();
    if let Some(browser) = browser {
        let Some(app) = application_url(browser) else {
            crate::log!("找不到浏览器 bundle，跳过动作");
            return;
        };
        let config = NSWorkspaceOpenConfiguration::configuration();
        workspace.openURLs_withApplicationAtURL_configuration_completionHandler(
            &NSArray::from_slice(&[&*url]),
            &app,
            &config,
            Some(&completion()),
        );
    } else if !workspace.openURL(&url) {
        crate::log!("打开 URL 失败");
    }
}
static PROCESSES: AtomicUsize = AtomicUsize::new(0);
/// 普通 Shell 动作先展开变量；Lua cmd 直接运行传入脚本，与 Windows API 一致。
pub(crate) fn run_shell(script: String) {
    process(move || {
        let mut cmd = Command::new("/bin/zsh");
        cmd.args(["-c", &script]);
        cmd
    }, true);
}
fn process(build: impl FnOnce() -> Command + Send + 'static, shell: bool) {
    if PROCESSES.fetch_add(1, Ordering::Relaxed) >= 16 {
        PROCESSES.fetch_sub(1, Ordering::Relaxed);
        crate::log!("后台进程已达上限，跳过动作");
        return;
    }
    std::thread::spawn(move || {
        use std::os::unix::process::CommandExt;
        let result = build()
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .process_group(0)
            .spawn()
            .and_then(|mut child| child.wait());
        match result {
            Ok(status) if shell => crate::log!("ran shell action: exit_status={status}"),
            Ok(status) => crate::log!("应用进程结束: exit_status={status}"),
            Err(_) => crate::log!("后台进程启动或等待失败"),
        }
        PROCESSES.fetch_sub(1, Ordering::Relaxed);
    });
}
fn system_key(key: &str) {
    let Some(code) = helpers::media_code(key) else {
        crate::log!("未知系统键，跳过动作");
        return;
    };
    if code == 2 || code == 3 {
        crate::log!("亮度键仅支持内建屏幕，外接 DDC 不在本阶段范围");
    }
    let events: Option<Vec<_>> = [true, false].into_iter().map(|down| {
        let event = NSEvent::otherEventWithType_location_modifierFlags_timestamp_windowNumber_context_subtype_data1_data2(NSEventType::SystemDefined, NSPoint::new(0.0, 0.0), NSEventModifierFlags::empty(), 0.0, 0, None, 8, helpers::media_data1(code, down), 0)?;
        let cg = event.CGEvent()?;
        CGEvent::set_integer_value_field(Some(&cg), CGEventField::EventSourceUserData, crate::events::INJECT_TAG);
        Some(cg)
    }).collect();
    let Some(events) = events else {
        crate::log!("创建系统键事件失败");
        return;
    };
    for event in events {
        CGEvent::post(CGEventTapLocation::HIDEventTap, Some(&event));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dry_run_discards_side_effects_before_ax_pasteboard_or_appkit() {
        initialize(true);
        for action in [
            Action::WindowOp {
                op: WindowOpKind::Close,
                allow_child: false,
            },
            Action::WindowOp {
                op: WindowOpKind::ToggleTopmost,
                allow_child: false,
            },
            Action::Shell {
                script: "%MS_SELECTED_TEXT% %MS_WIN_NAME%".into(),
            },
            Action::OpenUrl {
                url_template: "%MS_SELECTED_TEXT%".into(),
                browser: None,
            },
            Action::SystemKey { key: "PLAY".into() },
            Action::Menu {
                columns: 2,
                items: Vec::new(),
            },
            Action::LaunchApp {
                exe: "com.apple.ActivityMonitor".into(),
                args: None,
                elevate: true,
                win_class: None,
                win_title: None,
                regex: false,
                if_running: IfRunning::RunAgain,
            },
            Action::Lua {
                code: "unused".into(),
            },
        ] {
            enqueue(action, None, Vars::new(), (0, 0));
            tick();
            STATE.with(|s| {
                let s = s.borrow();
                assert!(s.queue.is_empty() && s.pending.is_none());
            });
        }
        assert_eq!(PROCESSES.load(Ordering::Relaxed), 0);
    }
}
