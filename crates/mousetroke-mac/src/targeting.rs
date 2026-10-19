//! 250ms 窗口快照：WindowServer 在工作线程查询，tap 只做有界命中测试与缓存读取。
use crate::{
    ax::{self, Cf, Raw},
    helpers::{self, Window},
    overlay::geometry::Rect,
};
use mousetroke_engine::Target;
use objc2_app_kit::{NSRunningApplication, NSWorkspace};
use objc2_core_foundation::CGRect;
use std::{
    cell::RefCell,
    collections::HashMap,
    sync::mpsc,
    time::{Duration, Instant},
};

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGWindowListCopyWindowInfo(options: u32, relative_to: u32) -> Raw;
    fn CGRectMakeWithDictionaryRepresentation(dict: Raw, rect: *mut CGRect) -> bool;
    static kCGWindowLayer: Raw;
    static kCGWindowOwnerPID: Raw;
    static kCGWindowNumber: Raw;
    static kCGWindowBounds: Raw;
}
#[derive(Default)]
struct Snapshot {
    rows: Vec<Window>,
    targets: HashMap<(u32, usize), Target>,
    front: Option<Target>,
    fullscreen: bool,
    pending: Option<mpsc::Receiver<Vec<Window>>>,
    titles: HashMap<(u32, usize), String>,
    warned_slow: bool,
    next_query: Option<Instant>,
    /// 各屏 Quartz 逻辑坐标，用于识别铺满整屏的透明壳窗口。
    screens: Vec<Rect>,
}
thread_local! { static SNAPSHOT: RefCell<Snapshot> = RefCell::new(Snapshot::default()); }

fn list() -> Vec<Window> {
    // 安全：公开选项 OnScreenOnly(1)|ExcludeDesktopElements(16)，接管 Copy 引用。
    let Some(list) = (unsafe { Cf::owned(CGWindowListCopyWindowInfo(1 | 16, 0)) }) else {
        crate::log!("读取 CG 窗口快照失败");
        return Vec::new();
    };
    ax::array(list.raw())
        .into_iter()
        .take(4096)
        .filter_map(|row| {
            // 安全：键来自 CoreGraphics；不读取需要屏幕录制权限的 kCGWindowName。
            unsafe {
                let layer = ax::number(ax::dict(row, kCGWindowLayer))?.try_into().ok()?;
                let pid = ax::number(ax::dict(row, kCGWindowOwnerPID))?
                    .try_into()
                    .ok()?;
                let wid = ax::number(ax::dict(row, kCGWindowNumber))?
                    .try_into()
                    .ok()?;
                let bounds = ax::dict(row, kCGWindowBounds);
                if bounds.is_null() {
                    return None;
                }
                let mut rect = CGRect::default();
                if !CGRectMakeWithDictionaryRepresentation(bounds, &mut rect) {
                    return None;
                }
                Some(Window {
                    pid,
                    wid,
                    layer,
                    bounds: Rect {
                        x: rect.origin.x,
                        y: rect.origin.y,
                        w: rect.size.width,
                        h: rect.size.height,
                    },
                })
            }
        })
        .collect()
}
fn target(app: &NSRunningApplication, wid: usize) -> Target {
    let path = |url: Option<objc2::rc::Retained<objc2_foundation::NSURL>>| {
        url.and_then(|u| u.path()).map(|p| p.to_string())
    };
    let (exe, class) = helpers::identity(
        path(app.bundleURL()),
        path(app.executableURL()),
        app.bundleIdentifier()
            .map(|v| v.to_string())
            .unwrap_or_default(),
    );
    Target {
        pid: app.processIdentifier() as u32,
        wid,
        exe,
        class,
        is_desktop: false,
    }
}
pub fn ignored(target: &Target) -> bool {
    target.pid == std::process::id()
        || matches!(
            target.class.as_str(),
            "io.github.eiaao.mousetroke"
                | "io.github.eiaao.mousetroke-settings"
                | "com.apple.dock"
                | "com.apple.controlcenter"
                | "com.apple.notificationcenterui"
                | "com.apple.systemuiserver"
        )
}
pub fn refresh(screens: &[Rect]) {
    SNAPSHOT.with(|slot| {
        let mut snapshot = slot.borrow_mut();
        if snapshot.screens != screens {
            snapshot.screens = screens.to_vec();
        }
        let received = snapshot.pending.as_ref().map(|rx| rx.try_recv());
        let now = Instant::now();
        if matches!(received, None | Some(Err(mpsc::TryRecvError::Empty)))
            && snapshot.next_query.is_some_and(|next| now < next)
        {
            return;
        }
        if let Some(Ok(rows)) = received {
            snapshot.rows = rows;
            snapshot.pending = None;
            let mut apps = HashMap::new();
            snapshot.targets = snapshot
                .rows
                .iter()
                .filter_map(|w| {
                    let app = apps.entry(w.pid).or_insert_with(|| {
                        NSRunningApplication::runningApplicationWithProcessIdentifier(w.pid as i32)
                    });
                    app.as_ref().map(|app| ((w.pid, w.wid), target(app, w.wid)))
                })
                .collect();
            let Snapshot {
                targets, titles, ..
            } = &mut *snapshot;
            titles.retain(|key, _| targets.contains_key(key));
        } else if matches!(received, Some(Err(mpsc::TryRecvError::Disconnected))) {
            snapshot.pending = None;
        }
        snapshot.front = NSWorkspace::sharedWorkspace()
            .frontmostApplication()
            .map(|app| {
                let wid = snapshot
                    .rows
                    .iter()
                    .find(|w| w.pid == app.processIdentifier() as u32 && w.layer == 0)
                    .map(|w| w.wid)
                    .unwrap_or(0);
                target(&app, wid)
            });
        snapshot.fullscreen = snapshot
            .front
            .as_ref()
            .and_then(|app| {
                snapshot
                    .rows
                    .iter()
                    .find(|w| w.layer == 0 && w.pid == app.pid)
            })
            .is_some_and(|w| {
                screens
                    .iter()
                    .any(|screen| helpers::covers(w.bounds, *screen))
            });
        if snapshot.pending.is_none() && snapshot.next_query.is_none_or(|next| now >= next) {
            snapshot.next_query = Some(now + Duration::from_millis(250));
            let (tx, rx) = mpsc::sync_channel(1);
            snapshot.pending = Some(rx);
            std::thread::spawn(move || {
                let _ = tx.send(list());
            });
        }
    });
}
pub fn at(x: i32, y: i32) -> Target {
    let start = Instant::now();
    SNAPSHOT.with(|slot| {
        let mut snapshot = slot.borrow_mut();
        // 菜单栏、Dock 等壳窗口常在非零层；先识别这些遮挡，避免透过去捕获底下应用。
        let shell = snapshot
            .rows
            .iter()
            .take_while(|w| w.layer != 0 || !w.bounds.contains(x as f64, y as f64))
            .find(|w| {
                w.bounds.contains(x as f64, y as f64)
                    && w.layer < 1000 // 穿透式轨迹窗不能遮住后续手势目标。
                    // Dock 有一扇铺满整屏的透明窗（真机 layer=20），不能当成遮挡，否则整屏都被忽略。
                    && !covers_screen(&w.bounds, &snapshot.screens)
                    && snapshot.targets.get(&(w.pid, w.wid)).is_some_and(ignored)
            });
        let row = shell.or_else(|| helpers::hit_test(&snapshot.rows, x as f64, y as f64));
        let target = row
            .and_then(|w| snapshot.targets.get(&(w.pid, w.wid)))
            .cloned()
            .or_else(|| snapshot.front.clone())
            .unwrap_or(Target {
                pid: 0,
                wid: 0,
                exe: None,
                class: String::new(),
                is_desktop: false,
            });
        let elapsed = start.elapsed();
        if elapsed.as_micros() > 2000 && !snapshot.warned_slow {
            snapshot.warned_slow = true;
            crate::log!(
                "target_at 超过 2ms（仅报告一次）: {}us",
                elapsed.as_micros()
            );
        }
        crate::selftest::resolved_target(&target.class);
        target
    })
}
pub fn fullscreen() -> bool {
    SNAPSHOT.with(|s| s.borrow().fullscreen)
}
pub fn window(target: &Target) -> Option<Window> {
    SNAPSHOT.with(|s| {
        s.borrow()
            .rows
            .iter()
            .find(|w| w.pid == target.pid && w.wid == target.wid)
            .cloned()
    })
}
/// tap 中只返回已有 AX 标题；变量实际引用时由动作工作线程刷新。
pub fn title(wid: usize) -> String {
    SNAPSHOT.with(|s| {
        s.borrow()
            .titles
            .iter()
            .find(|((_, id), _)| *id == wid)
            .map(|(_, text)| text.clone())
            .unwrap_or_default()
    })
}
pub fn remember_title(target: &Target, title: String) {
    SNAPSHOT.with(|s| {
        s.borrow_mut()
            .titles
            .insert((target.pid, target.wid), title);
    });
}

/// 窗口是否几乎铺满它所在的某块屏幕（≥90% 宽和高）。
fn covers_screen(bounds: &Rect, screens: &[Rect]) -> bool {
    screens.iter().any(|screen| {
        bounds.x <= screen.x + 1.0
            && bounds.y <= screen.y + 1.0
            && bounds.w >= screen.w * 0.9
            && bounds.h >= screen.h * 0.9
    })
}

#[cfg(test)]
mod shell_tests {
    use super::*;

    #[test]
    fn full_screen_shell_window_is_not_an_occluder() {
        let screen = Rect { x: 0.0, y: 0.0, w: 2560.0, h: 1440.0 };
        assert!(covers_screen(&Rect { x: 0.0, y: 0.0, w: 2560.0, h: 1440.0 }, &[screen]));
        // 菜单栏、Dock 条只占一部分，仍算遮挡
        assert!(!covers_screen(&Rect { x: 0.0, y: 0.0, w: 2560.0, h: 30.0 }, &[screen]));
        assert!(!covers_screen(&Rect { x: 600.0, y: 1360.0, w: 1300.0, h: 80.0 }, &[screen]));
    }
}
