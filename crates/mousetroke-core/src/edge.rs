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

/// 点 `(x, y)` 位于矩形 `(rx, ry, rw, rh)` 的哪条边/角；不在边上返回 `None`。角优先于边。
pub fn edge_at(x: i32, y: i32, (rx, ry, rw, rh): (i32, i32, i32, i32)) -> Option<u8> {
    let dl = x - rx;
    let dr = rx + rw - 1 - x;
    let dt = y - ry;
    let db = ry + rh - 1 - y;
    let mut v = 0u8;
    // 先按角半径收集
    if dt <= CORNER_PX {
        v |= TOP;
    }
    if db <= CORNER_PX {
        v |= BOTTOM;
    }
    if dl <= CORNER_PX {
        v |= LEFT;
    }
    if dr <= CORNER_PX {
        v |= RIGHT;
    }
    if is_corner(v) && ALL.contains(&v) {
        return Some(v);
    }
    // 非角：按边厚度判定（取最近的一条）
    [(dt, TOP), (db, BOTTOM), (dl, LEFT), (dr, RIGHT)]
        .into_iter()
        .filter(|(d, _)| *d <= EDGE_PX)
        .min_by_key(|(d, _)| *d)
        .map(|(_, e)| e)
}

/// 点离边/角 `v` 的"离开距离"：对其包含的每条边求点到该边的距离，取最大值。
/// 角落时离开任一条边都会使该值变大。
pub fn distance_from(v: u8, x: i32, y: i32, (rx, ry, rw, rh): (i32, i32, i32, i32)) -> i32 {
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
}
