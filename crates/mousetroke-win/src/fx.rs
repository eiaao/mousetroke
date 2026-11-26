//! 趣味特效的 Windows 播放器：在鼠标所在显示器上盖一层透明、点击穿透、不抢焦点的置顶分层窗口，
//! 独立线程按 60fps 用 `mousetroke-fx` 绘制并 `UpdateLayeredWindow` 提交。同一时间只播一个。
use mousetroke_core::presets::native::FunEffect;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use windows::core::w;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, SIZE, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GdiFlush, GetDC, GetMonitorInfoW, MonitorFromPoint, ReleaseDC,
    SelectObject, AC_SRC_ALPHA, AC_SRC_OVER, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, BLENDFUNCTION, CAPTUREBLT, DIB_RGB_COLORS,
    HBITMAP, HDC, HGDIOBJ, MONITORINFO, MONITOR_DEFAULTTONEAREST, ROP_CODE, SRCCOPY,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetCursorPos, PeekMessageW, RegisterClassW, ShowWindow,
    TranslateMessage, UpdateLayeredWindow, MSG, PM_REMOVE, SW_SHOWNOACTIVATE, ULW_ALPHA, WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
};

static PLAYING: AtomicBool = AtomicBool::new(false);
const FRAME: Duration = Duration::from_micros(16_667);
/// 兜底：任何特效最多播这么久。
const MAX_DURATION: Duration = Duration::from_secs(15);

/// 在 at（屏幕物理坐标；None 为当前鼠标位置）所在的显示器上播放。正在播放时忽略本次。
pub fn play(effect: FunEffect, at: Option<(i32, i32)>) {
    if PLAYING.swap(true, Ordering::AcqRel) {
        crate::log!("特效 {} 正在播放，忽略新的请求", effect.name());
        return;
    }
    let spawned = std::thread::Builder::new().name("fx".into()).spawn(move || {
        // 线程异常退出也要复位，免得之后再也播不了
        struct Reset;
        impl Drop for Reset {
            fn drop(&mut self) {
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
    let (x, y) = at.unwrap_or_else(cursor);
    let rect = monitor_rect(x, y);
    let mut surface = Surface::new(rect).ok_or("Failed to create the effect window")?;
    let (w, h) = (surface.w, surface.h);
    let name = effect.name();
    let capture = mousetroke_fx::wants_capture(name).then(|| surface.capture_screen());
    let mut fx = mousetroke_fx::create(name, w, h, ((x - rect.0) as f32, (y - rect.1) as f32), capture).ok_or("Unknown effect")?;
    let start = Instant::now();
    let mut last = start;
    loop {
        let now = Instant::now();
        let t = (now - start).as_secs_f32();
        let dt = (now - last).as_secs_f32().min(0.05);
        last = now;
        let more = fx.frame(&mut mousetroke_fx::Canvas { px: surface.pixels(), w, h }, t, dt);
        surface.present();
        if !more || now - start > MAX_DURATION {
            return Ok(());
        }
        if let Some(rest) = FRAME.checked_sub(now.elapsed()) {
            std::thread::sleep(rest);
        }
    }
}

fn cursor() -> (i32, i32) {
    let mut p = POINT::default();
    unsafe {
        let _ = GetCursorPos(&mut p);
    }
    (p.x, p.y)
}

/// 点所在显示器的矩形（物理像素）。
fn monitor_rect(x: i32, y: i32) -> (i32, i32, i32, i32) {
    unsafe {
        let mon = MonitorFromPoint(POINT { x, y }, MONITOR_DEFAULTTONEAREST);
        let mut mi = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
        let _ = GetMonitorInfoW(mon, &mut mi);
        let r = mi.rcMonitor;
        (r.left, r.top, r.right - r.left, r.bottom - r.top)
    }
}

unsafe extern "system" fn wndproc(h: HWND, m: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    DefWindowProcW(h, m, w, l)
}

struct Surface {
    hwnd: HWND,
    dc: HDC,
    dib: HBITMAP,
    bits: *mut u32,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    shown: bool,
}

impl Surface {
    fn new((x, y, w, h): (i32, i32, i32, i32)) -> Option<Self> {
        if w <= 0 || h <= 0 {
            return None;
        }
        unsafe {
            let hinst = GetModuleHandleW(None).ok()?;
            let class = w!("MousetrokeFx");
            let wc = WNDCLASSW { lpfnWndProc: Some(wndproc), hInstance: hinst.into(), lpszClassName: class, ..Default::default() };
            // 重复注册会失败，无妨
            RegisterClassW(&wc);
            let hwnd = CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                class,
                None,
                WS_POPUP,
                x,
                y,
                w,
                h,
                None,
                None,
                Some(hinst.into()),
                None,
            )
            .ok()?;
            crate::overlay::mark_not_fullscreen(hwnd);
            let dc = CreateCompatibleDC(None);
            let bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: w,
                    biHeight: -h,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut bits = std::ptr::null_mut();
            let Ok(dib) = CreateDIBSection(Some(dc), &bmi, DIB_RGB_COLORS, &mut bits, None, 0) else {
                let _ = DeleteDC(dc);
                crate::overlay::unmark_not_fullscreen(hwnd);
                let _ = DestroyWindow(hwnd);
                return None;
            };
            SelectObject(dc, HGDIOBJ(dib.0));
            Some(Self { hwnd, dc, dib, bits: bits as *mut u32, x, y, w, h, shown: false })
        }
    }

    fn pixels(&mut self) -> &mut [u32] {
        // SAFETY: bits 指向 CreateDIBSection 分配的 w*h 个 32 位像素，生命周期与 self 相同。
        unsafe { std::slice::from_raw_parts_mut(self.bits, (self.w * self.h) as usize) }
    }

    /// 窗口尚未显示时调用：截到的是桌面本身。不透明度补成 255。
    fn capture_screen(&mut self) -> Vec<u32> {
        unsafe {
            let screen = GetDC(None);
            let _ = BitBlt(self.dc, 0, 0, self.w, self.h, Some(screen), self.x, self.y, ROP_CODE(SRCCOPY.0 | CAPTUREBLT.0));
            ReleaseDC(None, screen);
            // 直接读 DIB 像素前先让 GDI 把排队的绘制做完
            let _ = GdiFlush();
        }
        let tex = self.pixels().iter().map(|p| p | 0xFF00_0000).collect();
        self.pixels().fill(0);
        tex
    }

    fn present(&mut self) {
        unsafe {
            let size = SIZE { cx: self.w, cy: self.h };
            let pos = POINT { x: self.x, y: self.y };
            let src = POINT { x: 0, y: 0 };
            let blend = BLENDFUNCTION { BlendOp: AC_SRC_OVER as u8, BlendFlags: 0, SourceConstantAlpha: 255, AlphaFormat: AC_SRC_ALPHA as u8 };
            let _ = UpdateLayeredWindow(self.hwnd, None, Some(&pos), Some(&size), Some(self.dc), Some(&src), COLORREF(0), Some(&blend), ULW_ALPHA);
            if !self.shown {
                let _ = ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
                self.shown = true;
            }
            let mut msg = MSG::default();
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            crate::overlay::unmark_not_fullscreen(self.hwnd);
            let _ = DestroyWindow(self.hwnd);
            let _ = DeleteDC(self.dc);
            let _ = DeleteObject(HGDIOBJ(self.dib.0));
        }
    }
}
