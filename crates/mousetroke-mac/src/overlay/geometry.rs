//! 不依赖 AppKit 的坐标与颜色转换，Windows 上也能验证多屏布局。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    /// AppKit 全局左下原点 → Quartz 全局左上原点；基准始终是 screens[0]。
    pub fn quartz(self, primary_height: f64) -> Self {
        Self {
            y: primary_height - self.y - self.h,
            ..self
        }
    }

    /// 默认 CALayer 使用左下原点，输入的 Quartz 坐标是逻辑点而非 Retina 像素。
    pub fn layer_point(self, x: f64, y: f64) -> (f64, f64) {
        (x - self.x, self.h - (y - self.y))
    }

    pub fn contains(self, x: f64, y: f64) -> bool {
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
    }

    pub fn anchor(self) -> (i32, i32) {
        (
            (self.x + self.w * 0.5) as i32,
            (self.y + self.h * 0.8) as i32,
        )
    }

    pub fn union(self, other: Self) -> Self {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        Self {
            x,
            y,
            w: (self.x + self.w).max(other.x + other.w) - x,
            h: (self.y + self.h).max(other.y + other.h) - y,
        }
    }
}

/// 与 Windows 相同：#RRGGBBAA，六位颜色按不透明处理，错误值回退半透明黑色。
pub fn rgba(s: &str) -> [f64; 4] {
    let hex = s.trim_start_matches('#');
    let v = u64::from_str_radix(hex, 16).unwrap_or(0x30);
    let bytes = match hex.len() {
        8 => [(v >> 24) as u8, (v >> 16) as u8, (v >> 8) as u8, v as u8],
        6 => [(v >> 16) as u8, (v >> 8) as u8, v as u8, 255],
        _ => [0, 0, 0, 0x30],
    };
    bytes.map(|b| b as f64 / 255.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quartz_and_layer_coordinates_cover_left_above_and_below_screens() {
        for (appkit, global, local) in [
            (
                Rect {
                    x: 0.0,
                    y: 0.0,
                    w: 1920.0,
                    h: 1080.0,
                },
                (120.0, 80.0),
                (120.0, 1000.0),
            ),
            (
                Rect {
                    x: -1280.0,
                    y: 100.0,
                    w: 1280.0,
                    h: 800.0,
                },
                (-1200.0, 200.0),
                (80.0, 780.0),
            ),
            (
                Rect {
                    x: 0.0,
                    y: 1080.0,
                    w: 1440.0,
                    h: 900.0,
                },
                (100.0, -850.0),
                (100.0, 850.0),
            ),
            (
                Rect {
                    x: 200.0,
                    y: -900.0,
                    w: 1440.0,
                    h: 900.0,
                },
                (300.0, 1200.0),
                (100.0, 780.0),
            ),
        ] {
            let cg = appkit.quartz(1080.0);
            assert!(cg.contains(global.0, global.1));
            assert_eq!(cg.layer_point(global.0, global.1), local);
            // contentsScale 不参与几何换算，1x/2x 显示器共用点坐标。
            assert_eq!(cg.layer_point(cg.x, cg.y), (0.0, cg.h));
            assert_eq!(cg.layer_point(cg.x + cg.w, cg.y + cg.h), (cg.w, 0.0));
        }
    }

    #[test]
    fn union_and_visible_frame_anchor_do_not_assume_zero_origin() {
        let left = Rect {
            x: -1280.0,
            y: -200.0,
            w: 1280.0,
            h: 800.0,
        };
        let right = Rect {
            x: 0.0,
            y: 0.0,
            w: 1920.0,
            h: 1080.0,
        };
        assert_eq!(
            left.union(right),
            Rect {
                x: -1280.0,
                y: -200.0,
                w: 3200.0,
                h: 1280.0
            }
        );
        let work = Rect {
            x: -1280.0,
            y: 100.0,
            w: 1280.0,
            h: 700.0,
        };
        assert_eq!(work.anchor(), (-640, 660));
        assert!(!left.contains(0.0, 0.0));
    }

    #[test]
    fn color_alpha_is_last_and_six_digit_color_is_opaque() {
        assert_eq!(
            rgba("#12345680"),
            [18.0 / 255.0, 52.0 / 255.0, 86.0 / 255.0, 128.0 / 255.0]
        );
        assert_eq!(rgba("#ff0000"), [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(rgba("bad"), [0.0, 0.0, 0.0, 48.0 / 255.0]);
    }

    #[test]
    fn edges_use_full_frames_including_menu_bar_dock_and_multidisplay_union() {
        use mousetroke_core::edge::{self, LEFT_TOP, RIGHT_BOTTOM, TOP, BOTTOM};
        let full = Rect { x: 0.0, y: 0.0, w: 2560.0, h: 1440.0 }.quartz(1440.0);
        let visible = Rect { x: 0.0, y: 80.0, w: 2560.0, h: 1330.0 }.quartz(1440.0);
        let bounds = |r: Rect| (r.x as i32, r.y as i32, r.w as i32, r.h as i32);
        assert_eq!(edge::edge_at(0, 0, bounds(full)), Some(LEFT_TOP));
        assert_eq!(edge::edge_at(1280, 0, bounds(full)), Some(TOP));
        assert_eq!(edge::edge_at(1280, 1439, bounds(full)), Some(BOTTOM));
        assert_eq!(edge::edge_at(2559, 1439, bounds(full)), Some(RIGHT_BOTTOM));
        assert!(!visible.contains(1280.0, 0.0));
        assert!(!visible.contains(1280.0, 1439.0));
        let left = Rect { x: -1920.0, y: 0.0, w: 1920.0, h: 1080.0 }.quartz(1440.0);
        let above = Rect { x: 0.0, y: 1440.0, w: 2560.0, h: 1440.0 }.quartz(1440.0);
        let union = full.union(left).union(above);
        assert_eq!(bounds(union), (-1920, -1440, 4480, 2880));
        assert_eq!(edge::edge_at(2559, 1439, bounds(union)), Some(RIGHT_BOTTOM));
        assert_eq!(edge::edge_at(0, 500, bounds(union)), None, "相邻屏幕接缝不是虚拟桌面的边");
    }
}
