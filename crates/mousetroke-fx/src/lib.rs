//! 趣味特效：放烟花、礼花、炸弹、红包雨、爱心、黑洞、下雪、裂屏、流星雨。
//!
//! 与平台无关：每帧在一块预乘 ARGB（`A<<24 | R<<16 | G<<8 | B`）像素缓冲上作画，
//! 由平台层盖一层透明、点击穿透的置顶窗口显示出来。尺寸按屏幕高度缩放（1080p 为 1.0）。
mod blackhole;
mod bomb;
pub mod canvas;
mod confetti;
mod crack;
mod fireworks;
mod hearts;
mod meteors;
mod money;
mod snow;

pub use canvas::Canvas;

pub trait Effect: Send {
    /// 画第 t 秒的一帧（dt 为距上一帧的秒数）；返回 false 表示播完。
    fn frame(&mut self, c: &mut Canvas, t: f32, dt: f32) -> bool;
}

/// 全部特效的标识，顺序即动作库中的顺序。
pub const ALL: &[&str] = &["fireworks", "confetti", "bomb", "money_rain", "hearts", "black_hole", "snow", "crack", "meteors"];

/// 是否要用播放前的屏幕截图（碎屏、黑洞扭曲）。截不到时仍可播放，只是退化为不动桌面的版本。
pub fn wants_capture(name: &str) -> bool {
    matches!(name, "bomb" | "black_hole")
}

/// 按标识创建特效。w、h 为显示器像素尺寸；at 为鼠标位置（显示器内坐标）；
/// capture 为该显示器的截图（不透明 ARGB，长度 w*h），只有 [`wants_capture`] 的特效会用到。
pub fn create(name: &str, w: i32, h: i32, at: (f32, f32), capture: Option<Vec<u32>>) -> Option<Box<dyn Effect>> {
    if w <= 0 || h <= 0 {
        return None;
    }
    let capture = capture.filter(|c| c.len() == (w * h) as usize);
    Some(match name {
        "fireworks" => Box::new(fireworks::Fireworks::new(w, h)),
        "confetti" => Box::new(confetti::Confetti::new(w, h)),
        "bomb" => Box::new(bomb::Bomb::new(w, h, capture, at)),
        "money_rain" => Box::new(money::MoneyRain::new(w, h)),
        "hearts" => Box::new(hearts::Hearts::new(w, h, at)),
        "black_hole" => Box::new(blackhole::BlackHole::new(w, h, capture, at)),
        "snow" => Box::new(snow::Snow::new(w, h)),
        "crack" => Box::new(crack::Crack::new(w, h, at)),
        "meteors" => Box::new(meteors::Meteors::new(w, h)),
        _ => return None,
    })
}

#[cfg(test)]
mod tests;
