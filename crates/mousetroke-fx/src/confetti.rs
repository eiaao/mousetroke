//! 礼花。
use crate::canvas::{Canvas, Rgb, Rng};
use crate::Effect;

// ---------------------------------------------------------------- 礼花

struct Paper { x: f32, y: f32, vx: f32, vy: f32, rot: f32, vrot: f32, flip: f32, vflip: f32, hw: f32, hh: f32, color: Rgb, phase: f32 }

pub struct Confetti { s: f32, w: f32, h: f32, rng: Rng, papers: Vec<Paper>, fired: u8 }

const PALETTE: [Rgb; 8] = [
    Rgb(255.0, 94.0, 91.0), Rgb(255.0, 210.0, 63.0), Rgb(59.0, 206.0, 172.0), Rgb(14.0, 173.0, 105.0),
    Rgb(77.0, 157.0, 224.0), Rgb(225.0, 92.0, 237.0), Rgb(255.0, 140.0, 66.0), Rgb(255.0, 255.0, 255.0),
];
const CONFETTI_END: f32 = 3.8;

impl Confetti {
    pub fn new(w: i32, h: i32) -> Self {
        Self { s: h as f32 / 1080.0, w: w as f32, h: h as f32, rng: Rng::new(), papers: Vec::with_capacity(500), fired: 0 }
    }

    /// 从左下、右下两角朝屏幕中上方喷出一批纸片。
    fn burst(&mut self, n: usize, power: f32) {
        let s = self.s;
        for side in [0.0f32, 1.0] {
            for _ in 0..n {
                let ang = self.rng.range(52.0, 80.0).to_radians();
                let dir = if side == 0.0 { 1.0 } else { -1.0 };
                let sp = self.rng.range(1400.0, 2400.0) * s * power;
                let strip = self.rng.f() < 0.25;
                self.papers.push(Paper {
                    x: self.w * (0.02 + side * 0.96),
                    y: self.h + 6.0,
                    vx: ang.cos() * sp * dir,
                    vy: -ang.sin() * sp,
                    rot: self.rng.range(0.0, 6.3),
                    vrot: self.rng.range(-7.0, 7.0),
                    flip: self.rng.range(0.0, 6.3),
                    vflip: self.rng.range(5.0, 13.0),
                    hw: if strip { 2.2 } else { self.rng.range(4.0, 6.5) } * s,
                    hh: if strip { 9.0 } else { self.rng.range(2.5, 4.0) } * s,
                    color: PALETTE[(self.rng.f() * PALETTE.len() as f32) as usize % PALETTE.len()],
                    phase: self.rng.range(0.0, 6.3),
                });
            }
        }
    }
}

impl Effect for Confetti {
    fn frame(&mut self, c: &mut Canvas, t: f32, dt: f32) -> bool {
        if self.fired == 0 {
            self.burst(150, 1.0);
            self.fired = 1;
        }
        if self.fired == 1 && t > 0.32 {
            self.burst(70, 0.85);
            self.fired = 2;
        }
        c.clear(0);
        let s = self.s;
        let drag = (-2.0 * dt).exp();
        let fade = ((CONFETTI_END - t) / 0.7).clamp(0.0, 1.0);
        let h = self.h;
        for p in self.papers.iter_mut() {
            p.vx *= drag;
            p.vy = p.vy * drag + 1250.0 * s * dt;
            // 纸片下落有终端速度，并左右飘
            let terminal = 170.0 * s;
            if p.vy > terminal {
                p.vy = terminal + (p.vy - terminal) * (-7.0 * dt).exp();
            }
            p.x += (p.vx + (t * 3.2 + p.phase).sin() * 55.0 * s) * dt;
            p.y += p.vy * dt;
            p.rot += p.vrot * dt;
            p.flip += p.vflip * dt;
            if p.y > h + 30.0 && p.vy > 0.0 {
                continue;
            }
            // 翻面：宽度按 cos 缩放，背面颜色暗一点
            let f = p.flip.cos();
            let color = if f < 0.0 { p.color.lerp(Rgb(0.0, 0.0, 0.0), 0.28) } else { p.color };
            c.rect(p.x, p.y, (p.hw * f.abs().max(0.12), p.hh), p.rot, color, 0.95 * fade);
        }
        t < CONFETTI_END
    }
}

