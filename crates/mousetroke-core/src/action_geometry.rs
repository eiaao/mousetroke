//! 预设动作使用的纯几何；坐标均为左上原点，同一次运算必须使用相同单位。

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn valid(self) -> bool {
        [
            self.x,
            self.y,
            self.width,
            self.height,
            self.right(),
            self.bottom(),
        ]
        .into_iter()
        .all(f64::is_finite)
            && self.width > 0.0
            && self.height > 0.0
    }

    pub fn right(self) -> f64 {
        self.x + self.width
    }
    pub fn bottom(self) -> f64 {
        self.y + self.height
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Monitor {
    pub bounds: Rect,
    pub work_area: Rect,
}

/// 保留逻辑点的小数；Windows 在最终设置窗口时统一舍入边缘。
pub fn halves(work: Rect) -> Option<(Rect, Rect)> {
    if !work.valid() {
        return None;
    }
    let left_width = work.width / 2.0;
    if left_width == 0.0 {
        return None;
    }
    Some((
        Rect {
            width: left_width,
            ..work
        },
        Rect {
            x: work.x + left_width,
            width: work.width - left_width,
            ..work
        },
    ))
}

pub fn centered(window: Rect, work: Rect) -> Option<Rect> {
    if !window.valid() || !work.valid() {
        return None;
    }
    let width = window.width.min(work.width);
    let height = window.height.min(work.height);
    Some(Rect {
        x: work.x + (work.width - width) / 2.0,
        y: work.y + (work.height - height) / 2.0,
        width,
        height,
    })
}

/// 位置和尺寸分别按工作区比例映射，保留原本跨屏或部分越界的比例。
pub fn proportional(window: Rect, source: Rect, target: Rect) -> Option<Rect> {
    if !window.valid() || !source.valid() || !target.valid() {
        return None;
    }
    let width = window.width / source.width * target.width;
    let height = window.height / source.height * target.height;
    let result = Rect {
        x: target.x + (window.x - source.x) / source.width * target.width,
        y: target.y + (window.y - source.y) / source.height * target.height,
        width,
        height,
    };
    result.valid().then_some(result)
}

/// 按完整屏幕的 x、y 排序，避免任务栏位置改变顺序；相同布局保持原顺序。
pub fn monitor_order(monitors: &[Monitor]) -> Vec<usize> {
    let mut order: Vec<_> = (0..monitors.len())
        .filter(|&i| monitors[i].bounds.valid() && monitors[i].work_area.valid())
        .collect();
    order.sort_by(|&a, &b| {
        monitors[a]
            .bounds
            .x
            .total_cmp(&monitors[b].bounds.x)
            .then(monitors[a].bounds.y.total_cmp(&monitors[b].bounds.y))
    });
    order
}

pub fn next_monitor(monitors: &[Monitor], current: usize) -> Option<usize> {
    let order = monitor_order(monitors);
    if order.len() <= 1 {
        return None;
    }
    let index = order.iter().position(|&i| i == current)?;
    Some(order[(index + 1) % order.len()])
}

/// 取交叠面积最大的屏幕；完全离屏时取矩形距离最近的屏幕。
pub fn monitor_for_rect(monitors: &[Monitor], window: Rect) -> Option<usize> {
    if !window.valid() {
        return None;
    }
    let mut best = None;
    let mut best_area = -1.0;
    let mut best_distance = f64::INFINITY;
    for i in monitor_order(monitors) {
        let r = monitors[i].bounds;
        let area = (r.right().min(window.right()) - r.x.max(window.x)).max(0.0)
            * (r.bottom().min(window.bottom()) - r.y.max(window.y)).max(0.0);
        let dx = (r.x - window.right()).max(window.x - r.right()).max(0.0);
        let dy = (r.y - window.bottom()).max(window.y - r.bottom()).max(0.0);
        let distance = dx.hypot(dy);
        if area > best_area || (area == best_area && distance < best_distance) {
            best = Some(i);
            best_area = area;
            best_distance = distance;
        }
    }
    best
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameInsets {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

impl FrameInsets {
    /// outer 为 GetWindowRect，visible 为 DWM 扩展边框；保留有符号差值。
    pub fn between(outer: Rect, visible: Rect) -> Option<Self> {
        if !outer.valid() || !visible.valid() {
            return None;
        }
        Some(Self {
            left: visible.x - outer.x,
            top: visible.y - outer.y,
            right: outer.right() - visible.right(),
            bottom: outer.bottom() - visible.bottom(),
        })
    }

    pub fn outer_for_visible(self, visible: Rect) -> Option<Rect> {
        let outer = Rect {
            x: visible.x - self.left,
            y: visible.y - self.top,
            width: visible.width + self.left + self.right,
            height: visible.height + self.top + self.bottom,
        };
        (visible.valid() && outer.valid()).then_some(outer)
    }
}

/// AppKit 全局左下原点转 AX/Quartz 左上原点；primary_top 取主屏 frame 的顶部。
pub fn from_bottom_left(rect: Rect, primary_top: f64) -> Option<Rect> {
    let result = Rect {
        y: primary_top - rect.y - rect.height,
        ..rect
    };
    result.valid().then_some(result)
}

/// 桌面与系统外壳不是可调整布局的普通窗口；Finder 的普通窗口仍可操作。
pub fn layout_target_allowed(platform: crate::caps::Platform, is_desktop: bool, class: &str, layer: i32) -> bool {
    if is_desktop { return false; }
    match platform {
        crate::caps::Platform::Windows => !matches!(class, "Progman" | "WorkerW" | "SHELLDLL_DefView" | "Shell_TrayWnd" | "Shell_SecondaryTrayWnd"),
        crate::caps::Platform::Macos => class != "com.apple.dock" && !(class == "com.apple.finder" && layer != 0),
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn r(x: f64, y: f64, width: f64, height: f64) -> Rect {
        Rect {
            x,
            y,
            width,
            height,
        }
    }
    fn m(bounds: Rect, work_area: Rect) -> Monitor {
        Monitor { bounds, work_area }
    }

    #[test]
    fn halves_respect_left_and_top_taskbars_and_odd_width() {
        let work = r(-1870.0, -960.0, 1871.0, 1040.0);
        let (left, right) = halves(work).unwrap();
        assert_eq!(left, r(-1870.0, -960.0, 935.5, 1040.0));
        assert_eq!(right, r(-934.5, -960.0, 935.5, 1040.0));
        assert_eq!(right.right(), work.right());
        assert_eq!(left.right(), right.x);
    }

    #[test]
    fn centering_preserves_size_and_shrinks_each_oversized_axis() {
        let work = r(-1200.0, 40.0, 1200.0, 760.0);
        assert_eq!(
            centered(r(99.0, 99.0, 400.0, 200.0), work),
            Some(r(-800.0, 320.0, 400.0, 200.0))
        );
        assert_eq!(centered(r(0.0, 0.0, 1800.0, 900.0), work), Some(work));
        assert_eq!(
            centered(r(0.0, 0.0, 1800.0, 200.0), work),
            Some(r(-1200.0, 320.0, 1200.0, 200.0))
        );
    }

    #[test]
    fn mapping_scales_position_and_size_across_different_work_areas() {
        let source = r(-1600.0, 40.0, 1600.0, 960.0);
        let target = r(80.0, -1000.0, 2400.0, 1200.0);
        let window = r(-1200.0, 280.0, 800.0, 480.0);
        let mapped = proportional(window, source, target).unwrap();
        assert_eq!(mapped, r(680.0, -700.0, 1200.0, 600.0));
        assert_eq!(proportional(mapped, target, source), Some(window));
        assert_eq!(
            proportional(r(-2000.0, -200.0, 2000.0, 1200.0), source, target),
            Some(r(-520.0, -1300.0, 3000.0, 1500.0))
        );
    }

    #[test]
    fn ordering_uses_full_bounds_then_y_and_wraps() {
        let rows = [
            m(r(0.0, 0.0, 1920.0, 1080.0), r(80.0, 0.0, 1840.0, 1080.0)),
            m(
                r(-1600.0, 0.0, 1600.0, 900.0),
                r(-1600.0, 30.0, 1600.0, 870.0),
            ),
            m(
                r(0.0, -1200.0, 1920.0, 1200.0),
                r(0.0, -1200.0, 1920.0, 1160.0),
            ),
        ];
        assert_eq!(monitor_order(&rows), vec![1, 2, 0]);
        assert_eq!(next_monitor(&rows, 0), Some(1));
        assert_eq!(next_monitor(&rows, 1), Some(2));
        assert_eq!(next_monitor(&rows[..1], 0), None);
        assert_eq!(next_monitor(&[], 0), None);
        assert_eq!(next_monitor(&rows, 99), None);
        assert_eq!(
            monitor_for_rect(&rows, r(-100.0, 100.0, 900.0, 500.0)),
            Some(0)
        );
        assert_eq!(
            monitor_for_rect(&rows, r(-3000.0, 100.0, 100.0, 100.0)),
            Some(1)
        );
    }

    #[test]
    fn dwm_insets_align_visible_edges_and_support_borderless_windows() {
        let visible = r(-1920.0, 30.0, 960.0, 1050.0);
        let insets =
            FrameInsets::between(r(92.0, 100.0, 816.0, 608.0), r(100.0, 100.0, 800.0, 600.0))
                .unwrap();
        assert_eq!(
            insets.outer_for_visible(visible),
            Some(r(-1928.0, 30.0, 976.0, 1058.0))
        );
        assert_eq!(
            FrameInsets::default().outer_for_visible(visible),
            Some(visible)
        );
        let signed =
            FrameInsets::between(r(1.0, 1.0, 98.0, 98.0), r(0.0, 0.0, 100.0, 100.0)).unwrap();
        assert_eq!(
            signed.outer_for_visible(r(0.0, 0.0, 100.0, 100.0)),
            Some(r(1.0, 1.0, 98.0, 98.0))
        );
    }

    #[test]
    fn appkit_flip_handles_screens_above_below_and_negative_x() {
        assert_eq!(
            from_bottom_left(r(-1440.0, 0.0, 1440.0, 900.0), 1080.0),
            Some(r(-1440.0, 180.0, 1440.0, 900.0))
        );
        assert_eq!(
            from_bottom_left(r(0.0, 1080.0, 1920.0, 1200.0), 1080.0),
            Some(r(0.0, -1200.0, 1920.0, 1200.0))
        );
        assert_eq!(
            from_bottom_left(r(0.0, -900.0, 1440.0, 875.0), 1080.0),
            Some(r(0.0, 1105.0, 1440.0, 875.0))
        );
    }

    #[test]
    fn invalid_rectangles_are_rejected_without_division_or_clamp_panics() {
        let valid = r(0.0, 0.0, 100.0, 100.0);
        for bad in [
            r(0.0, 0.0, 0.0, 1.0),
            r(f64::NAN, 0.0, 1.0, 1.0),
            r(0.0, 0.0, -1.0, 1.0),
        ] {
            assert_eq!(halves(bad), None);
            assert_eq!(centered(valid, bad), None);
            assert_eq!(proportional(valid, bad, valid), None);
            assert_eq!(monitor_for_rect(&[], bad), None);
        }
    }
}
