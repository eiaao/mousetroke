//! 轨迹覆盖层：全虚拟屏、透明、点击穿透的分层窗口 + 软件光栅（预乘 ARGB DIB）。
//!
//! **性能铁律（钩子回调内零阻塞）**：
//! - 会话开始不做任何系统调用（无清屏、无提交），首次绘制才显示窗口
//! - 提交一律走 `UpdateLayeredWindowIndirect` 的**脏矩形**路径
//! - 淡出动画由 `WM_TIMER` 驱动（消息循环泵送，不在回调里 sleep）
//! - 只对"内容包围盒"做像素操作，绝不整屏遍历
//!
//! 以后若换 Direct2D 渲染，架构不变，只替换渲染后端。

use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM};
use windows::core::PCWSTR;
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, DeleteObject, GetTextExtentPoint32W, SelectObject, TextOutW,
    AC_SRC_ALPHA, AC_SRC_OVER, BI_RGB, BLENDFUNCTION, BITMAPINFO, BITMAPINFOHEADER,
    DIB_RGB_COLORS, HGDIOBJ, HDC, HBITMAP,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, GetSystemMetrics, KillTimer, RegisterClassW, SetTimer,
    ShowWindow, UpdateLayeredWindow, UpdateLayeredWindowIndirect, CS_HREDRAW, CS_VREDRAW,
    SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
    SW_SHOWNOACTIVATE, UPDATELAYEREDWINDOWINFO, WM_TIMER, WNDCLASSW, WS_EX_LAYERED,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
};

const CLASS_NAME: &str = "MousetrokeOverlay";
const FADE_TIMER_ID: usize = 1;
const FADE_FRAMES: u8 = 5;
const FADE_INTERVAL_MS: u32 = 16;

/// "#RRGGBBAA"（旧格式，如 #00000030 = 黑色 19% 透明）→ (a, r, g, b)。
/// 6 位 "#RRGGBB" 视为不透明。
pub fn parse_argb(s: &str) -> (u8, u8, u8, u8) {
    let hex = s.trim_start_matches('#');
    let v = u64::from_str_radix(hex, 16).unwrap_or(0x000000_30);
    match hex.len() {
        8 => (
            (v & 0xff) as u8,
            ((v >> 24) & 0xff) as u8,
            ((v >> 16) & 0xff) as u8,
            ((v >> 8) & 0xff) as u8,
        ),
        6 => (0xff, ((v >> 16) & 0xff) as u8, ((v >> 8) & 0xff) as u8, (v & 0xff) as u8),
        _ => (0x30, 0, 0, 0),
    }
}

/// 覆盖层。所有方法须在引擎线程（安装钩子的线程）调用。
pub struct Overlay {
    hwnd: HWND,
    mem_dc: HDC,
    _dib: HBITMAP, // 选入 mem_dc 的位图
    pixels: *mut u32,
    width: i32,
    height: i32,
    vx: i32,
    vy: i32,
    visible: bool,
    /// 上一个原始输入点（平滑曲线的控制点）。
    prev: Option<(f64, f64)>,
    /// 已画到的位置（上一段的中点）。
    mid: Option<(f64, f64)>,
    half_width: f64,
    color: (u8, u8, u8, u8),
    last_present: Option<std::time::Instant>,
    ulw_warned: bool,
    /// 被跳过（节流）的待提交区域并集。
    pending_dirty: Option<RECT>,
    /// 本次会话绘制内容的包围盒（淡出/清屏只处理此区域）。
    content_bounds: Option<RECT>,
    fade_frames: u8,
    // 标签（手势名）渲染的暂存资源与位置
    label_rect: Option<(i32, i32, i32, i32)>, // 本地坐标
    /// 标签层（预乘 ARGB，按 label_rect 行优先）与其下方原有内容。
    /// 清除标签时还原下方内容，不会把轨迹擦掉一块。淡出时丢弃（整体一起淡出）。
    label_layer: Vec<u32>,
    label_under: Vec<u32>,
    scratch_dc: HDC,
    scratch_ptr: *mut u32,
    _scratch_dib: HBITMAP,
    font: windows::Win32::Graphics::Gdi::HFONT,
    /// 标签字体创建时的语言标签（语言切换后丢弃重建）。
    font_tag: &'static str,
}

unsafe impl Send for Overlay {}

impl Overlay {
    /// 创建（窗口隐藏状态，缓冲区全透明）。
    pub fn new(color_normal: &str, width_px: f64) -> Option<Self> {
        unsafe {
            let hinst = GetModuleHandleW(None).ok()?;
            let mut classname: Vec<u16> = CLASS_NAME.encode_utf16().chain([0]).collect();

            let wc = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(overlay_wndproc),
                hInstance: hinst.into(),
                lpszClassName: windows::core::PCWSTR(classname.as_mut_ptr()),
                ..Default::default()
            };
            let atom = RegisterClassW(&wc);
            if atom == 0 && windows::core::Error::from_win32().code().0 != 1410 {
                // 1410 = ERROR_CLASS_ALREADY_EXISTS，重复创建时忽略
                return None;
            }

            let (vx, vy, vw, vh) = (
                GetSystemMetrics(SM_XVIRTUALSCREEN),
                GetSystemMetrics(SM_YVIRTUALSCREEN),
                GetSystemMetrics(SM_CXVIRTUALSCREEN).max(1),
                GetSystemMetrics(SM_CYVIRTUALSCREEN).max(1),
            );

            let hwnd = CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW
                    | WS_EX_NOACTIVATE,
                windows::core::PCWSTR(classname.as_mut_ptr()),
                None,
                WS_POPUP,
                vx,
                vy,
                vw,
                vh,
                None,
                None,
                Some(hinst.into()),
                None,
            )
            .ok()?;

            // 32bpp 顶向下 DIB，预乘 ARGB
            let mem_dc = CreateCompatibleDC(None);
            let bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: vw,
                    biHeight: -vh, // 顶向下
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
            let dib = CreateDIBSection(Some(mem_dc), &bmi, DIB_RGB_COLORS, &mut bits, None, 0)
                .map_err(|e| crate::log!("DIB 创建失败: {e}"))
                .ok()?;
            if bits.is_null() {
                return None;
            }
            let _old = SelectObject(mem_dc, HGDIOBJ(dib.0));
            std::ptr::write_bytes(bits as *mut u8, 0, (vw * vh * 4) as usize);

            Some(Self {
                hwnd,
                mem_dc,
                _dib: dib,
                pixels: bits as *mut u32,
                width: vw,
                height: vh,
                vx,
                vy,
                visible: false,
                prev: None,
                mid: None,
                half_width: half_width(width_px),
                color: parse_argb(color_normal),
                last_present: None,
                ulw_warned: false,
                pending_dirty: None,
                content_bounds: None,
                fade_frames: 0,
                label_rect: None,
                label_layer: Vec::new(),
                label_under: Vec::new(),
                scratch_dc: HDC(std::ptr::null_mut()),
                scratch_ptr: std::ptr::null_mut(),
                _scratch_dib: HBITMAP(std::ptr::null_mut()),
                font: windows::Win32::Graphics::Gdi::HFONT::default(),
                font_tag: "",
            })
        }
    }

    pub fn set_color(&mut self, color: &str) {
        self.color = parse_argb(color);
    }

    pub fn set_width(&mut self, width_px: f64) {
        self.half_width = half_width(width_px);
    }

    /// 手势会话开始：重置状态并**强制提交清掉上次残留**（区域级，代价小）。
    /// 若窗口从未显示则零系统调用。
    pub fn start_session(&mut self, x: i32, y: i32) {
        self.stop_fade();
        // 先记下上次的内容区（轨迹 ∪ 标签），清零后再提交——不能先 take 走 content_bounds，
        // 否则 zero_content 拿不到区域，淡出被打断的半透明轨迹会永远留在屏幕上
        let label = self.label_rect.map(|(l, t, r, b)| RECT { left: l, top: t, right: r, bottom: b });
        let stale = match (self.content_bounds, label) {
            (Some(c), Some(l)) => Some(union_rect(&c, &l)),
            (c, l) => c.or(l),
        };
        self.zero_content();
        if let Some(b) = stale {
            // 上次会话的内容可能仍在窗口表面上（淡出未跑完/中断），强制清一次
            if self.visible {
                self.present_dirty(&b, true);
            }
        }
        self.prev = Some(self.to_local(x, y));
        self.mid = None;
    }

    /// 手势中：画到 (x, y) 并脏矩形提交（首次绘制才显示窗口）。
    ///
    /// 中点平滑：从上一段中点到本段中点画二次贝塞尔（控制点为上一个输入点），
    /// 再叠加轻度指数平滑，鼠标事件的整数抖动和折线拐角都会被抹平。
    pub fn line_to(&mut self, x: i32, y: i32) {
        let raw = self.to_local(x, y);
        let Some(c) = self.prev else {
            return;
        };
        // 轻度指数平滑：滤掉逐像素抖动，滞后约半个事件
        let p = (c.0 + (raw.0 - c.0) * 0.55, c.1 + (raw.1 - c.1) * 0.55);
        self.prev = Some(p);
        let from = self.mid.unwrap_or(c);
        let to = ((c.0 + p.0) / 2.0, (c.1 + p.1) / 2.0);
        self.mid = Some(to);
        let len = (c.0 - from.0).hypot(c.1 - from.1) + (to.0 - c.0).hypot(to.1 - c.1);
        let n = (len / 3.0).ceil().clamp(1.0, 48.0) as i32;
        let mut dirty = RECT { left: i32::MAX, top: i32::MAX, right: i32::MIN, bottom: i32::MIN };
        let mut a = from;
        for i in 1..=n {
            let t = i as f64 / n as f64;
            let u = 1.0 - t;
            let b = (
                u * u * from.0 + 2.0 * u * t * c.0 + t * t * to.0,
                u * u * from.1 + 2.0 * u * t * c.1 + t * t * to.1,
            );
            dirty = union_rect(&dirty, &self.stroke(a, b));
            a = b;
        }
        if dirty.left < dirty.right {
            self.union_bounds(&dirty);
            self.present_dirty(&dirty, false);
        }
    }

    /// 手势结束：清内容区域并提交（窗口保持显示——全透明位图即不可见，
    /// 避免隐藏/重显导致的表面残留）。
    pub fn finish(&mut self) {
        self.prev = None;
        self.mid = None;
        self.stop_fade();
        let bounds = self.content_bounds.map(|b| b);
        self.zero_content();
        if let Some(b) = bounds {
            if self.visible {
                self.present_dirty(&b, true);
            }
        }
    }

    /// 手势结束：淡出（**非阻塞**——WM_TIMER 驱动，回调立即返回）。
    pub fn fade_out(&mut self) {
        self.prev = None;
        self.mid = None;
        // 标签与轨迹一起淡出：不再单独还原
        self.label_layer.clear();
        self.label_under.clear();
        if self.content_bounds.is_none() {
            return; // 没画过东西
        }
        if !self.visible {
            self.finish();
            return;
        }
        self.fade_frames = FADE_FRAMES;
        unsafe {
            SetTimer(Some(self.hwnd), FADE_TIMER_ID, FADE_INTERVAL_MS, None);
        }
    }

    /// 淡出动画的一帧（由 overlay wndproc 的 WM_TIMER 调用）。
    pub fn tick_fade(&mut self) {
        if self.fade_frames == 0 {
            return;
        }
        if let Some(b) = self.content_bounds {
            let (l, t, r, bo) = (b.left.max(0), b.top.max(0), b.right.min(self.width), b.bottom.min(self.height));
            if r > l && bo > t {
                unsafe {
                    for y in t..bo {
                        let row = self.pixels.add(y as usize * self.width as usize);
                        for x in l..r {
                            let p = row.add(x as usize);
                            let v = *p;
                            *p = (((v >> 24) & 0xff) * 13 / 18 << 24)
                                | (((v >> 16) & 0xff) * 13 / 18 << 16)
                                | (((v >> 8) & 0xff) * 13 / 18 << 8)
                                | ((v & 0xff) * 13 / 18);
                        }
                    }
                }
            }
            let mut rect = b;
            // 标签区域一并纳入
            if let Some((lx, ly, lr, lb)) = self.label_rect {
                rect.left = rect.left.min(lx);
                rect.top = rect.top.min(ly);
                rect.right = rect.right.max(lr);
                rect.bottom = rect.bottom.max(lb);
            }
            self.present_dirty(&rect, true);
        }
        self.fade_frames -= 1;
        if self.fade_frames == 0 {
            self.stop_fade();
            let bounds = self.content_bounds;
            self.zero_content();
            if let Some(b) = bounds {
                self.present_dirty(&b, true);
            }
        }
    }

    fn stop_fade(&mut self) {
        if self.fade_frames > 0 {
            self.fade_frames = 0;
            unsafe {
                let _ = KillTimer(Some(self.hwnd), FADE_TIMER_ID);
            }
        }
    }

    /// 清空内容包围盒区域（含标签），像素级、只碰画过的部分。
    fn zero_content(&mut self) {
        let mut bounds = self.content_bounds.take();
        if let Some((lx, ly, lr, lb)) = self.label_rect.take() {
            bounds = Some(match bounds {
                Some(b) => RECT {
                    left: b.left.min(lx),
                    top: b.top.min(ly),
                    right: b.right.max(lr),
                    bottom: b.bottom.max(lb),
                },
                None => RECT { left: lx, top: ly, right: lr, bottom: lb },
            });
        }
        if let Some(b) = bounds {
            let (l, t, r, bo) = (b.left.max(0), b.top.max(0), b.right.min(self.width), b.bottom.min(self.height));
            if r > l && bo > t {
                unsafe {
                    for y in t..bo {
                        let row = self.pixels.add(y as usize * self.width as usize);
                        std::ptr::write_bytes(row.add(l as usize), 0, (r - l) as usize * 4);
                    }
                }
            }
        }
        self.content_bounds = None;
        self.label_rect = None;
        self.label_layer.clear();
        self.label_under.clear();
        self.pending_dirty = None;
    }

    /// 仅首次显示（之后窗口常驻：全透明位图 = 不可见，避免隐藏/重显的表面残留）。
    fn show(&mut self, on: bool) {
        if on && !self.visible {
            unsafe {
                let _ = ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
            }
            self.visible = true;
        }
    }

    /// 虚拟屏矩形 (x, y, w, h)——边缘/角落热区检测用。
    pub fn virtual_rect(&self) -> (i32, i32, i32, i32) {
        (self.vx, self.vy, self.width, self.height)
    }

    fn to_local(&self, x: i32, y: i32) -> (f64, f64) {
        ((x - self.vx) as f64, (y - self.vy) as f64)
    }

    fn union_bounds(&mut self, rect: &RECT) {
        let clamped = RECT {
            left: rect.left.clamp(-8, self.width + 8),
            top: rect.top.clamp(-8, self.height + 8),
            right: rect.right.clamp(-8, self.width + 8),
            bottom: rect.bottom.clamp(-8, self.height + 8),
        };
        self.content_bounds = Some(match self.content_bounds {
            Some(b) => RECT {
                left: b.left.min(clamped.left),
                top: b.top.min(clamped.top),
                right: b.right.max(clamped.right),
                bottom: b.bottom.max(clamped.bottom),
            },
            None => clamped,
        });
    }

    /// 画一小段（胶囊形）：按像素到线段的距离算覆盖度——核心 + 中层 + 光晕三档，
    /// 与已有像素**取最大**而不是叠加，拐角、端点、重叠处浓淡一致。返回脏矩形。
    fn stroke(&mut self, a: (f64, f64), b: (f64, f64)) -> RECT {
        let hw = self.half_width;
        let reach = hw * 4.0 + 1.5;
        let x0 = (a.0.min(b.0) - reach).floor().max(0.0) as i32;
        let y0 = (a.1.min(b.1) - reach).floor().max(0.0) as i32;
        let x1 = (a.0.max(b.0) + reach).ceil().min(self.width as f64 - 1.0) as i32;
        let y1 = (a.1.max(b.1) + reach).ceil().min(self.height as f64 - 1.0) as i32;
        if x1 < x0 || y1 < y0 {
            return RECT { left: i32::MAX, top: i32::MAX, right: i32::MIN, bottom: i32::MIN };
        }
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let l2 = dx * dx + dy * dy;
        let ca = self.color.0 as f64 / 255.0;
        for py in y0..=y1 {
            for px in x0..=x1 {
                let (cx, cy) = (px as f64, py as f64);
                let t = if l2 > 0.0 { (((cx - a.0) * dx + (cy - a.1) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
                let d = (cx - a.0 - t * dx).hypot(cy - a.1 - t * dy);
                let cov = |r: f64| (r + 0.5 - d).clamp(0.0, 1.0);
                let v = (255.0 * cov(hw)).max(110.0 * cov(hw * 2.0)).max(36.0 * cov(hw * 4.0));
                if v >= 1.0 {
                    self.trail_pixel(px, py, (v * ca) as u8);
                }
            }
        }
        RECT { left: x0, top: y0, right: x1 + 1, bottom: y1 + 1 }
    }

    /// 轨迹像素：透明度取最大。落在标签区域内时写入标签下层并重新合成。
    fn trail_pixel(&mut self, x: i32, y: i32, a: u8) {
        let (_, r, g, b) = self.color;
        let v = premul(a, r, g, b);
        unsafe {
            let p = self.pixels.add(y as usize * self.width as usize + x as usize);
            if let (Some((l, t, rr, bb)), false) = (self.label_rect, self.label_under.is_empty()) {
                if x >= l && x < rr && y >= t && y < bb {
                    let j = ((y - t) * (rr - l) + (x - l)) as usize;
                    if a as u32 > self.label_under[j] >> 24 {
                        self.label_under[j] = v;
                        *p = over(self.label_layer[j], v);
                    }
                    return;
                }
            }
            if a as u32 > *p >> 24 {
                *p = v;
            }
        }
    }

    /// 以 (cx, cy)（屏幕坐标）为中心绘制手势名：深色半透明圆角底板 + 文字
    /// （GDI 灰度抗锯齿 → alpha 掩码）。底板保证白底网页上也看得清。
    pub fn draw_label(&mut self, text: &str, cx: i32, cy: i32, color: &str) {
        const LW: i32 = 1024;
        const LH: i32 = 64;
        unsafe {
            // 懒初始化暂存资源；语言变化时字体丢弃重建（字体按语言选择）
            let lang = mousetroke_core::i18n::lang();
            if self.scratch_ptr.is_null() {
                let dc = CreateCompatibleDC(None);
                let bmi = BITMAPINFO {
                    bmiHeader: BITMAPINFOHEADER {
                        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                        biWidth: LW,
                        biHeight: -LH,
                        biPlanes: 1,
                        biBitCount: 32,
                        biCompression: BI_RGB.0,
                        ..Default::default()
                    },
                    ..Default::default()
                };
                let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
                let dib = CreateDIBSection(Some(dc), &bmi, DIB_RGB_COLORS, &mut bits, None, 0)
                    .unwrap_or_default();
                let _old = SelectObject(dc, HGDIOBJ(dib.0));
                self.scratch_dc = dc;
                self.scratch_ptr = bits as *mut u32;
                self._scratch_dib = dib;
            }
            if self.font_tag != lang.tag() {
                use windows::Win32::Graphics::Gdi as gdi;
                let face: Vec<u16> = mousetroke_core::i18n::ui_font(lang).encode_utf16().chain([0]).collect();
                let font = gdi::CreateFontW(
                    -24,
                    0,
                    0,
                    0,
                    700,
                    0,
                    0,
                    0,
                    gdi::FONT_CHARSET(1),
                    gdi::FONT_OUTPUT_PRECISION(0),
                    gdi::FONT_CLIP_PRECISION(0),
                    gdi::FONT_QUALITY(4), // ANTIALIASED：灰度抗锯齿才能当 alpha 掩码（ClearType 会有彩边）
                    0,
                    PCWSTR(face.as_ptr()),
                );
                if !self.font.is_invalid() {
                    let _ = DeleteObject(HGDIOBJ(self.font.0));
                }
                SelectObject(self.scratch_dc, HGDIOBJ(font.0));
                self.font = font;
                self.font_tag = lang.tag();
            }
            // 还原旧标签下方的内容（之后一并提交）
            let old = self.label_rect;
            self.clear_label_inner();

            let wide: Vec<u16> = text.encode_utf16().collect();
            let mut size = windows::Win32::Foundation::SIZE::default();
            if !GetTextExtentPoint32W(self.scratch_dc, &wide, &mut size).as_bool() {
                return;
            }
            if size.cx > LW || size.cy > LH {
                return;
            }
            // 白字黑底 → 亮度即 alpha 掩码
            std::ptr::write_bytes(self.scratch_ptr as *mut u8, 0, (LW * LH * 4) as usize);
            let _ = windows::Win32::Graphics::Gdi::SetTextColor(
                self.scratch_dc,
                COLORREF(0x00FFFFFF),
            );
            let _ = windows::Win32::Graphics::Gdi::SetBkColor(self.scratch_dc, COLORREF(0));
            let _ = TextOutW(self.scratch_dc, 0, 0, &wide);

            let (a, r, g, b) = parse_argb(color);
            // 底板几何（本地坐标）：文字四周留白，圆角
            const PAD_X: i32 = 12;
            const PAD_Y: i32 = 5;
            const RADIUS: f32 = 9.0;
            const BG: (u8, u8, u8, u8) = (0xC8, 20, 24, 28);
            let (bw, bh) = (size.cx + PAD_X * 2, size.cy + PAD_Y * 2);
            if bw > self.width || bh > self.height {
                return;
            }
            let bx = ((cx - self.vx) - bw / 2).clamp(0, self.width - bw);
            let by = ((cy - self.vy) - bh / 2).clamp(0, self.height - bh);

            // 标签层：底板（圆角覆盖率抗锯齿）+ 文字
            let mut layer = vec![0u32; (bw * bh) as usize];
            for row in 0..bh {
                for col in 0..bw {
                    let cov = rounded_cover(col as f32 + 0.5, row as f32 + 0.5, bw as f32, bh as f32, RADIUS);
                    if cov > 0.0 {
                        layer[(row * bw + col) as usize] = premul((BG.0 as f32 * cov) as u8, BG.1, BG.2, BG.3);
                    }
                }
            }
            for row in 0..size.cy {
                for col in 0..size.cx {
                    let v = *self.scratch_ptr.add(row as usize * LW as usize + col as usize);
                    let lum = (v >> 16) & 0xff; // 白字：R 通道即亮度
                    if lum >= 8 {
                        let k = ((row + PAD_Y) * bw + col + PAD_X) as usize;
                        layer[k] = over(premul((a as u32 * lum / 255) as u8, r, g, b), layer[k]);
                    }
                }
            }
            // 保存下方内容，合成上屏
            let mut under = Vec::with_capacity(layer.len());
            for row in 0..bh {
                let base = self.pixels.add((by + row) as usize * self.width as usize + bx as usize);
                for col in 0..bw {
                    let p = base.add(col as usize);
                    under.push(*p);
                    *p = over(layer[(row * bw + col) as usize], *p);
                }
            }
            self.label_layer = layer;
            self.label_under = under;
            self.label_rect = Some((bx, by, bx + bw, by + bh));
            let mut rect = RECT { left: bx - 1, top: by - 1, right: bx + bw + 1, bottom: by + bh + 1 };
            if let Some((l, t, r, b)) = old {
                rect = union_rect(&rect, &RECT { left: l, top: t, right: r, bottom: b });
            }
            self.union_bounds(&rect);
            self.present_dirty(&rect, false);
        }
    }

    /// 清除当前标签（不提交）：还原下方内容；淡出中（下层已丢弃）则清零。
    fn clear_label_inner(&mut self) {
        let Some((l, t, r, b)) = self.label_rect.take() else { return };
        let under = std::mem::take(&mut self.label_under);
        self.label_layer.clear();
        let w = (r - l) as usize;
        unsafe {
            for y in t.max(0)..b.min(self.height) {
                let row = self.pixels.add(y as usize * self.width as usize + l as usize);
                if under.len() == w * (b - t) as usize {
                    std::ptr::copy_nonoverlapping(under.as_ptr().add((y - t) as usize * w), row, w);
                } else {
                    std::ptr::write_bytes(row, 0, w);
                }
            }
        }
    }

    /// 清除当前标签并提交。
    pub fn clear_label(&mut self) {
        self.clear_label_inner();
        let rect = self.content_bounds.unwrap_or(RECT {
            left: 0,
            top: 0,
            right: 8,
            bottom: 8,
        });
        self.present_dirty(&rect, true);
    }

    // ------------------------------------------------------------ 提交

    /// 脏矩形提交（`UpdateLayeredWindowIndirect` + prcDirty）。
    /// `force = true` 忽略节流（动画帧）。被节流的区域并入 pending，
    /// 随后任何一次提交都会覆盖。
    fn present_dirty(&mut self, dirty: &RECT, force: bool) {
        if dirty.left >= dirty.right || dirty.top >= dirty.bottom {
            return;
        }
        // 节流：距上次提交 <6ms 则暂存（缓冲区内容是最新的，下次提交带出）
        if !force {
            if let Some(t) = self.last_present {
                if t.elapsed().as_millis() < 6 {
                    self.pending_dirty = Some(match self.pending_dirty.take() {
                        Some(p) => union_rect(&p, dirty),
                        None => *dirty,
                    });
                    return;
                }
            }
        }
        let rect = match self.pending_dirty.take() {
            Some(p) => union_rect(&p, dirty),
            None => *dirty,
        };
        unsafe {
            let size = SIZE { cx: self.width, cy: self.height };
            let src = POINT { x: 0, y: 0 };
            let blend = BLENDFUNCTION {
                BlendOp: AC_SRC_OVER as u8,
                BlendFlags: 0,
                SourceConstantAlpha: 255,
                AlphaFormat: AC_SRC_ALPHA as u8,
            };
            let info = UPDATELAYEREDWINDOWINFO {
                cbSize: std::mem::size_of::<UPDATELAYEREDWINDOWINFO>() as u32,
                hdcDst: HDC(std::ptr::null_mut()),
                pptDst: std::ptr::null(),
                psize: &size,
                hdcSrc: self.mem_dc,
                pptSrc: &src,
                crKey: COLORREF(0),
                pblend: &blend,
                dwFlags: windows::Win32::UI::WindowsAndMessaging::ULW_ALPHA,
                prcDirty: &rect,
            };
            if !UpdateLayeredWindowIndirect(self.hwnd, &info).as_bool() {
                // 回退整窗提交（只报一次）
                if !self.ulw_warned {
                    self.ulw_warned = true;
                    let e = windows::core::Error::from_win32();
                    crate::log!("UpdateLayeredWindowIndirect 失败: {e}，回退整窗");
                }
                let _ = UpdateLayeredWindow(
                    self.hwnd,
                    None,
                    None,
                    Some(&size),
                    Some(self.mem_dc),
                    Some(&src),
                    COLORREF(0),
                    Some(&blend),
                    windows::Win32::UI::WindowsAndMessaging::ULW_ALPHA,
                );
            }
        }
        if !self.visible {
            self.show(true); // 首次提交成功后再显示
        }
        self.last_present = Some(std::time::Instant::now());
    }
}

fn union_rect(a: &RECT, b: &RECT) -> RECT {
    RECT {
        left: a.left.min(b.left),
        top: a.top.min(b.top),
        right: a.right.max(b.right),
        bottom: a.bottom.max(b.bottom),
    }
}

unsafe extern "system" fn overlay_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_TIMER && wparam.0 == FADE_TIMER_ID {
        // 淡出动画帧（引擎线程消息循环泵送，不阻塞钩子回调）
        crate::engine::with_engine(|e| e.overlay_tick());
        return LRESULT(0);
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

/// 预乘 ARGB 像素。
fn premul(a: u8, r: u8, g: u8, b: u8) -> u32 {
    let a32 = a as u32;
    (a32 << 24) | ((r as u32 * a32 / 255) << 16) | ((g as u32 * a32 / 255) << 8) | (b as u32 * a32 / 255)
}

/// 预乘 ARGB 合成：src over dst。
fn over(src: u32, dst: u32) -> u32 {
    let k = 255 - (src >> 24);
    let ch = |sh: u32| (((src >> sh) & 0xff) + ((dst >> sh) & 0xff) * k / 255).min(255) << sh;
    ch(24) | ch(16) | ch(8) | ch(0)
}

/// 轨迹半宽（px）。
fn half_width(width_px: f64) -> f64 {
    (width_px / 2.0 + 0.4).max(1.0)
}

/// 圆角矩形 (0,0,w,h) 在点 (x,y) 处的覆盖率（0~1，边缘 1px 抗锯齿）。
fn rounded_cover(x: f32, y: f32, w: f32, h: f32, r: f32) -> f32 {
    let qx = (x - w / 2.0).abs() - (w / 2.0 - r);
    let qy = (y - h / 2.0).abs() - (h / 2.0 - r);
    let outside = qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - r;
    (0.5 - outside).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 淡出途中开始新手势：上次的轨迹必须清零（曾因先 take 走包围盒而残留在屏幕上）。
    #[test]
    fn new_session_clears_interrupted_fade() {
        let mut o = Overlay::new("#00C2FF08", 2.0).expect("overlay");
        let (vx, vy) = (o.vx, o.vy);
        o.start_session(vx + 100, vy + 100);
        for i in 1..20 {
            o.line_to(vx + 100 + i * 3, vy + 100);
        }
        o.draw_label("测试", vx + 300, vy + 300, "#FFFFFF08");
        o.fade_out();
        o.tick_fade(); // 淡出到一半被打断
        o.start_session(vx + 900, vy + 900);
        let px = |o: &Overlay, x: i32, y: i32| unsafe { *o.pixels.add(y as usize * o.width as usize + x as usize) };
        for y in 90..110 {
            for x in 90..180 {
                assert_eq!(px(&o, x, y), 0, "轨迹残留于 ({x},{y})");
            }
        }
        for y in 270..330 {
            for x in 240..360 {
                assert_eq!(px(&o, x, y), 0, "标签残留于 ({x},{y})");
            }
        }
        o.finish();
    }

    /// 手动查看轨迹与标签渲染：`cargo test -p mousetroke-win label_render -- --ignored`，
    /// 输出 BMP 到 MT_LABEL_OUT。左半白底、右半黑底。
    /// 轨迹为带抖动的 →↓→；中途在路径上显示标签再清除（轨迹不应被擦掉）；最后显示标签。
    #[test]
    #[ignore]
    fn label_render() {
        let mut o = Overlay::new("#00C2FFE0", 2.0).expect("overlay");
        let (w, h) = (640i32, 400i32);
        let (vx, vy) = (o.vx, o.vy);
        let jitter = |i: i32| ((i * 7919) % 5 - 2) as i32;
        let mut pts = Vec::new();
        for i in 0..40 { pts.push((60 + i * 6, 60 + jitter(i) / 2 + i / 8)); }
        for i in 0..45 { pts.push((300 + jitter(i) - i / 6, 65 + i * 6)); }
        for i in 0..40 { pts.push((293 + i * 6, 335 + jitter(i) / 2 + i / 10)); }
        o.start_session(vx + pts[0].0, vy + pts[0].1);
        for (k, &(x, y)) in pts.iter().enumerate() {
            o.line_to(vx + x, vy + y);
            if k == 70 {
                o.draw_label("Enter", vx + x, vy + y - 34, "#FFFFFFE6"); // 盖在竖线上
            }
            if k == 90 {
                o.clear_label();
            }
        }
        if std::env::var("MT_LABEL_MATCHED").is_ok() {
            o.draw_label("关闭标签页 Ctrl+W", vx + 520, vy + 330, "#FF7A00F2");
        } else {
            o.draw_label("未识别  → ↓ →", vx + 520, vy + 330, "#FFFFFFE6");
        }
        let mut bmp = Vec::new();
        let row = (w * 3 + 3) & !3;
        let size = 54 + row * h;
        bmp.extend(b"BM");
        bmp.extend((size as u32).to_le_bytes());
        bmp.extend([0u8; 4]);
        bmp.extend(54u32.to_le_bytes());
        bmp.extend(40u32.to_le_bytes());
        bmp.extend(w.to_le_bytes());
        bmp.extend(h.to_le_bytes());
        bmp.extend(1u16.to_le_bytes());
        bmp.extend(24u16.to_le_bytes());
        bmp.extend([0u8; 24]);
        for y in (0..h).rev() {
            for x in 0..w {
                let p = unsafe { *o.pixels.add(y as usize * o.width as usize + x as usize) };
                let a = (p >> 24) & 0xff;
                let bg = if x < w / 2 { 255 } else { 0 };
                let c = |sh: u32| ((p >> sh) & 0xff) + bg * (255 - a) / 255;
                bmp.extend([c(0) as u8, c(8) as u8, c(16) as u8]);
            }
            bmp.extend(std::iter::repeat_n(0u8, (row - w * 3) as usize));
        }
        std::fs::write(std::env::var("MT_LABEL_OUT").unwrap_or("label.bmp".into()), bmp).unwrap();
    }
}
