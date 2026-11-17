//! 屏幕边/角编码与命中判定。
//!
//! 编码与旧格式配置中的 `RectEdge` / `RectCorner` 枚举值一致：
//! 边是位标志 `TOP=1 RIGHT=2 BOTTOM=4 LEFT=8`，角是两条边按位或：
//! `RIGHT_TOP=3 RIGHT_BOTTOM=6 LEFT_TOP=9 LEFT_BOTTOM=12`。
//! 因此旧格式配置里的 `EdgeCorner.Value` 可直接作为本模块的值使用。

pub const TOP: u8 = 1;
pub const RIGHT: u8 = 2;
pub const BOTTOM: u8 = 4;
pub const LEFT: u8 = 8;
pub const RIGHT_TOP: u8 = RIGHT | TOP;
pub const RIGHT_BOTTOM: u8 = RIGHT | BOTTOM;
pub const LEFT_TOP: u8 = LEFT | TOP;
pub const LEFT_BOTTOM: u8 = LEFT | BOTTOM;

/// 全部合法值（4 边 + 4 角）。
pub const ALL: [u8; 8] = [TOP, RIGHT, BOTTOM, LEFT, RIGHT_TOP, RIGHT_BOTTOM, LEFT_TOP, LEFT_BOTTOM];

/// 是否角落。
pub fn is_corner(v: u8) -> bool {
    v.count_ones() == 2
}

/// 人类可读名称（仅日志用；界面一侧用 i18n 的 `edge.*` 词条）。
// i18n:allow-begin 日志专用：运行日志不翻译
pub fn name(v: u8) -> &'static str {
    match v {
        TOP => "上边",
        RIGHT => "右边",
        BOTTOM => "下边",
        LEFT => "左边",
        RIGHT_TOP => "右上角",
        RIGHT_BOTTOM => "右下角",
        LEFT_TOP => "左上角",
        LEFT_BOTTOM => "左下角",
        _ => "未知边角",
    }
}
// i18n:allow-end

/// 角落判定半径（px）：距两条边都在此范围内算角。
pub const CORNER_PX: i32 = 6;
/// 边判定厚度（px）。
pub const EDGE_PX: i32 = 4;

/// 屏幕矩形 `(x, y, w, h)`，与鼠标事件使用同一坐标系。
pub type ScreenRect = (i32, i32, i32, i32);

/// 点 `(x, y)` 位于矩形 `(rx, ry, rw, rh)` 的哪条边/角；不在边上返回 `None`。角优先于边。
pub fn edge_at(x: i32, y: i32, rect: ScreenRect) -> Option<u8> {
    edge_at_sides(x, y, rect, |_| true)
}

/// 同 [`edge_at`]，但只考虑 `usable` 允许的边；角要求两条边都可用。
fn edge_at_sides(x: i32, y: i32, (rx, ry, rw, rh): ScreenRect, usable: impl Fn(u8) -> bool) -> Option<u8> {
    let sides = [(y - ry, TOP), (ry + rh - 1 - y, BOTTOM), (x - rx, LEFT), (rx + rw - 1 - x, RIGHT)];
    let sides = sides.into_iter().filter(|&(_, e)| usable(e));
    // 先按角半径收集
    let v = sides.clone().filter(|&(d, _)| d <= CORNER_PX).fold(0u8, |v, (_, e)| v | e);
    if is_corner(v) && ALL.contains(&v) {
        return Some(v);
    }
    // 非角：按边厚度判定（取最近的一条）
    sides.filter(|&(d, _)| d <= EDGE_PX).min_by_key(|&(d, _)| d).map(|(_, e)| e)
}

fn contains((rx, ry, rw, rh): ScreenRect, x: i32, y: i32) -> bool {
    x >= rx && x < rx + rw && y >= ry && y < ry + rh
}

/// 点所在的屏幕；不在任何屏幕内时取最近的一块（事件坐标可能短暂越界一两个像素）。
pub fn screen_at(x: i32, y: i32, screens: &[ScreenRect]) -> Option<ScreenRect> {
    let valid = screens.iter().copied().filter(|&(_, _, w, h)| w > 0 && h > 0);
    valid.clone().find(|&r| contains(r, x, y)).or_else(|| {
        valid.min_by_key(|&(rx, ry, rw, rh)| {
            let dx = (x - x.clamp(rx, rx + rw - 1)) as i64;
            let dy = (y - y.clamp(ry, ry + rh - 1)) as i64;
            dx * dx + dy * dy
        })
    })
}

/// 多显示器下点 `(x, y)` 所在的边/角，同时返回所在屏幕。
///
/// 只认光标真正会停住的外边界：某条边外侧 1 像素若落在另一块屏幕上，
/// 这段是屏幕交界，不算边；角要求两条边都是外边界，交界处的角降级为另一条边。
pub fn edge_at_screens(x: i32, y: i32, screens: &[ScreenRect]) -> Option<(u8, ScreenRect)> {
    let rect @ (rx, ry, rw, rh) = screen_at(x, y, screens)?;
    let (cx, cy) = (x.clamp(rx, rx + rw - 1), y.clamp(ry, ry + rh - 1));
    let outer = |e: u8| {
        let beyond = match e {
            TOP => (cx, ry - 1),
            BOTTOM => (cx, ry + rh),
            LEFT => (rx - 1, cy),
            _ => (rx + rw, cy),
        };
        !screens.iter().any(|&r| contains(r, beyond.0, beyond.1))
    };
    edge_at_sides(x, y, rect, outer).map(|e| (e, rect))
}

/// 点离边/角 `v` 的"离开距离"：对其包含的每条边求点到该边的距离，取最大值。
/// 角落时离开任一条边都会使该值变大。
pub fn distance_from(v: u8, x: i32, y: i32, (rx, ry, rw, rh): ScreenRect) -> i32 {
    let mut d = 0;
    if v & TOP != 0 {
        d = d.max(y - ry);
    }
    if v & BOTTOM != 0 {
        d = d.max(ry + rh - 1 - y);
    }
    if v & LEFT != 0 {
        d = d.max(x - rx);
    }
    if v & RIGHT != 0 {
        d = d.max(rx + rw - 1 - x);
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;
    const R: (i32, i32, i32, i32) = (0, 0, 1920, 1080);

    #[test]
    fn official_values() {
        assert_eq!((RIGHT_TOP, RIGHT_BOTTOM, LEFT_TOP, LEFT_BOTTOM), (3, 6, 9, 12));
    }

    #[test]
    fn corners() {
        assert_eq!(edge_at(0, 0, R), Some(LEFT_TOP));
        assert_eq!(edge_at(1919, 0, R), Some(RIGHT_TOP));
        assert_eq!(edge_at(1919, 1079, R), Some(RIGHT_BOTTOM));
        assert_eq!(edge_at(0, 1079, R), Some(LEFT_BOTTOM));
        assert_eq!(edge_at(5, 1074, R), Some(LEFT_BOTTOM));
    }

    #[test]
    fn edges() {
        assert_eq!(edge_at(960, 0, R), Some(TOP));
        assert_eq!(edge_at(1919, 500, R), Some(RIGHT));
        assert_eq!(edge_at(960, 1079, R), Some(BOTTOM));
        assert_eq!(edge_at(0, 500, R), Some(LEFT));
        assert_eq!(edge_at(960, 500, R), None);
        assert_eq!(edge_at(5, 500, R), None); // 超出边厚度
    }

    #[test]
    fn leave_distance() {
        assert_eq!(distance_from(TOP, 500, 40, R), 40);
        assert_eq!(distance_from(RIGHT, 1919, 10, R), 0); // 沿右边摩擦不算离开
        assert_eq!(distance_from(LEFT_BOTTOM, 3, 1079 - 50, R), 50);
    }

    #[test]
    fn negative_origin() {
        // 副屏在左侧的虚拟桌面
        let r = (-1280, 0, 3200, 1080);
        assert_eq!(edge_at(-1280, 0, r), Some(LEFT_TOP));
        assert_eq!(edge_at(1919, 1079, r), Some(RIGHT_BOTTOM));
    }

    /// 报告 #81 的布局：左 1080p，右 2K，大屏上下各多出 180。
    const SMALL: ScreenRect = (0, 0, 1920, 1080);
    const BIG: ScreenRect = (1920, -180, 2560, 1440);
    const PAIR: [ScreenRect; 2] = [SMALL, BIG];

    fn at(x: i32, y: i32, screens: &[ScreenRect]) -> Option<u8> {
        edge_at_screens(x, y, screens).map(|(e, _)| e)
    }

    #[test]
    fn smaller_monitor_has_its_own_outer_edges_and_corners() {
        assert_eq!(at(500, 0, &PAIR), Some(TOP));
        assert_eq!(at(500, 1079, &PAIR), Some(BOTTOM));
        assert_eq!(at(0, 500, &PAIR), Some(LEFT));
        assert_eq!(at(0, 0, &PAIR), Some(LEFT_TOP));
        assert_eq!(at(0, 1079, &PAIR), Some(LEFT_BOTTOM));
        assert_eq!(edge_at_screens(500, 0, &PAIR).map(|(_, r)| r), Some(SMALL));
    }

    #[test]
    fn larger_monitor_corners_and_exposed_left_edge() {
        assert_eq!(at(1920, -180, &PAIR), Some(LEFT_TOP));
        assert_eq!(at(1920, 1259, &PAIR), Some(LEFT_BOTTOM));
        assert_eq!(at(4479, -180, &PAIR), Some(RIGHT_TOP));
        assert_eq!(at(4479, 1259, &PAIR), Some(RIGHT_BOTTOM));
        // 大屏左侧高出、低于小屏的部分光标会停住，算左边
        assert_eq!(at(1920, -100, &PAIR), Some(LEFT));
        assert_eq!(at(1920, 1200, &PAIR), Some(LEFT));
    }

    #[test]
    fn seam_between_monitors_is_not_an_edge() {
        assert_eq!(at(1919, 500, &PAIR), None);
        assert_eq!(at(1920, 500, &PAIR), None);
        assert_eq!(at(1917, 500, &PAIR), None);
    }

    #[test]
    fn seam_corners_fall_back_to_the_outer_edge() {
        assert_eq!(at(1919, 0, &PAIR), Some(TOP));
        assert_eq!(at(1919, 1079, &PAIR), Some(BOTTOM));
        assert_eq!(at(1915, 3, &PAIR), Some(TOP));
    }

    #[test]
    fn aligned_monitors_keep_the_combined_top_edge_without_middle_corners() {
        let screens = [(0, 0, 1920, 1080), (1920, 0, 1920, 1080)];
        assert_eq!(at(1919, 0, &screens), Some(TOP));
        assert_eq!(at(1920, 0, &screens), Some(TOP));
        assert_eq!(at(0, 0, &screens), Some(LEFT_TOP));
        assert_eq!(at(3839, 1079, &screens), Some(RIGHT_BOTTOM));
    }

    #[test]
    fn stacked_and_three_monitor_layouts() {
        // 上下叠放：交界的上下边都不算
        let stacked = [(0, 0, 1920, 1080), (0, 1080, 1920, 1080)];
        assert_eq!(at(500, 1079, &stacked), None);
        assert_eq!(at(500, 1080, &stacked), None);
        assert_eq!(at(0, 1080, &stacked), Some(LEFT));
        // 三屏：中间屏只有上下边
        let three = [(-1920, 0, 1920, 1080), (0, 0, 1920, 1080), (1920, 0, 1920, 1080)];
        assert_eq!(at(0, 500, &three), None);
        assert_eq!(at(1919, 500, &three), None);
        assert_eq!(at(0, 0, &three), Some(TOP));
        assert_eq!(at(-1920, 0, &three), Some(LEFT_TOP));
    }

    #[test]
    fn coordinates_slightly_outside_use_the_nearest_monitor() {
        assert_eq!(at(-1, 500, &PAIR), Some(LEFT));
        assert_eq!(at(-2, -1, &PAIR), Some(LEFT_TOP));
        assert_eq!(edge_at_screens(10, 10, &[]), None);
    }

    fn bbox(screens: &[ScreenRect]) -> ScreenRect {
        let x0 = screens.iter().map(|r| r.0).min().unwrap();
        let y0 = screens.iter().map(|r| r.1).min().unwrap();
        let x1 = screens.iter().map(|r| r.0 + r.2).max().unwrap();
        let y1 = screens.iter().map(|r| r.1 + r.3).max().unwrap();
        (x0, y0, x1 - x0, y1 - y0)
    }

    /// 在每块屏幕边缘附近逐点比较新旧判定。
    fn near_edges(screens: &[ScreenRect], mut check: impl FnMut(i32, i32)) {
        for &(rx, ry, rw, rh) in screens {
            for x in rx..rx + rw {
                for y in (ry..ry + 10).chain(ry + rh - 10..ry + rh) {
                    check(x, y);
                }
            }
            for y in ry..ry + rh {
                for x in (rx..rx + 10).chain(rx + rw - 10..rx + rw) {
                    check(x, y);
                }
            }
        }
    }

    #[test]
    fn rectangular_layouts_behave_exactly_like_before() {
        for screens in [
            vec![(0, 0, 1920, 1080)],
            vec![(0, 0, 1920, 1080), (1920, 0, 1920, 1080)],
            vec![(-2560, 0, 2560, 1440), (0, 0, 2560, 1440)],
            vec![(0, 0, 1920, 1080), (0, 1080, 1920, 1080)],
            vec![(-1920, 0, 1920, 1080), (0, 0, 1920, 1080), (1920, 0, 1920, 1080)],
        ] {
            let all = bbox(&screens);
            near_edges(&screens, |x, y| assert_eq!(at(x, y, &screens), edge_at(x, y, all), "{screens:?} ({x}, {y})"));
        }
    }

    #[test]
    fn every_previous_trigger_point_still_triggers_on_the_same_side() {
        for screens in [
            PAIR.to_vec(),
            vec![(0, 0, 2560, 1440), (2560, 360, 1920, 1080)],
            vec![(0, 0, 1920, 1080), (300, 1080, 1280, 1024)],
            vec![(-1280, 200, 1280, 1024), (0, 0, 1920, 1080), (1920, -400, 1080, 1920)],
        ] {
            let all = bbox(&screens);
            near_edges(&screens, |x, y| {
                if let Some(old) = edge_at(x, y, all) {
                    let new = at(x, y, &screens);
                    // 以前能触发的位置现在一定也触发，且包含原来那条边（可能升级为角）
                    assert!(new.is_some_and(|n| n & old != 0), "{screens:?} ({x}, {y}) {old} -> {new:?}");
                    if is_corner(old) {
                        assert_eq!(new, Some(old), "{screens:?} ({x}, {y})");
                    }
                }
            });
        }
    }
}
