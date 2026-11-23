//! 流星雨：屏幕慢慢暗成夜空、星星闪烁，几颗流星拖着长尾从右上划向左下。
use crate::canvas::{Canvas, Rgb, Rng};
use crate::Effect;

struct Meteor {
    at: f32,
    x0: f32,
    y0: f32,
    dx: f32,
    dy: f32,
    speed: f32,
    tail: f32,
    life: f32,
    width: f32,
    tint: Rgb,
}

struct Star {
    x: f32,
    y: f32,
    r: f32,
    phase: f32,
    freq: f32,
}

pub struct Meteors {
    s: f32,
    w: i32,
    h: i32,
    meteors: Vec<Meteor>,
    stars: Vec<Star>,
    /// 每行夜色的不透明度（上浓下淡），预先算好。
    rows: Vec<f32>,
}

const FADE_IN: f32 = 0.5;
const FADE_START: f32 = 4.1;
const END: f32 = 4.8;
const NIGHT: Rgb = Rgb(6.0, 10.0, 30.0);

impl Meteors {
    pub fn new(w: i32, h: i32) -> Self {
        let s = h as f32 / 1080.0;
        let mut rng = Rng::new();
        let (wf, hf) = (w as f32, h as f32);
        let n = 9;
        let meteors = (0..n)
            .map(|i| {
                // 大致同一方向（右上 → 左下），角度略有差别
                let ang = std::f32::consts::PI * rng.range(0.70, 0.78);
                Meteor {
                    at: 0.35 + i as f32 / n as f32 * 3.0 + rng.range(-0.12, 0.12),
                    x0: rng.range(0.25, 1.25) * wf,
                    y0: rng.range(-0.12, 0.25) * hf,
                    dx: ang.cos(),
                    dy: ang.sin(),
                    speed: rng.range(1300.0, 2100.0) * s,
                    tail: rng.range(220.0, 420.0) * s,
                    life: rng.range(0.55, 0.85),
                    width: rng.range(1.3, 2.4) * s.max(0.7),
                    tint: if rng.f() < 0.3 { Rgb(200.0, 230.0, 255.0) } else { Rgb(255.0, 250.0, 235.0) },
                }
            })
            .collect();
        let stars = (0..(140.0 * wf / hf) as usize)
            .map(|_| Star { x: rng.range(0.0, wf), y: rng.f().powf(1.5) * hf * 0.75, r: rng.range(0.6, 1.6) * s.max(0.7), phase: rng.range(0.0, 6.3), freq: rng.range(2.0, 6.0) })
            .collect();
        let rows = (0..h).map(|y| 0.62 - 0.4 * (y as f32 / hf)).collect();
        Self { s, w, h, meteors, stars, rows }
    }
}

impl Effect for Meteors {
    fn frame(&mut self, c: &mut Canvas, t: f32, _dt: f32) -> bool {
        let s = self.s;
        let env = if t < FADE_IN { t / FADE_IN } else if t > FADE_START { (1.0 - (t - FADE_START) / (END - FADE_START)).max(0.0) } else { 1.0 };
        // 夜色：整行同色，直接填
        for y in 0..self.h {
            let a = self.rows[y as usize] * env;
            let p = ((a * 255.0) as u32) << 24 | ((NIGHT.0 * a) as u32) << 16 | ((NIGHT.1 * a) as u32) << 8 | (NIGHT.2 * a) as u32;
            c.px[(y * self.w) as usize..((y + 1) * self.w) as usize].fill(p);
        }
        for st in &self.stars {
            let tw = 0.55 + 0.45 * (t * st.freq + st.phase).sin();
            c.disc(st.x, st.y, st.r, Rgb(230.0, 236.0, 255.0), tw * env, true);
        }
        for m in &self.meteors {
            let age = t - m.at;
            if age < 0.0 || age > m.life {
                continue;
            }
            let u = age / m.life;
            let bright = (u / 0.15).min(1.0) * (1.0 - ((u - 0.7) / 0.3).max(0.0));
            let d = m.speed * age;
            let head = (m.x0 + m.dx * d, m.y0 + m.dy * d);
            let tail = m.tail * (0.4 + 0.6 * (u / 0.3).min(1.0));
            // 尾巴：分段画，越往后越细越淡
            let n = 22;
            for i in 0..n {
                let (k0, k1) = (i as f32 / n as f32, (i + 1) as f32 / n as f32);
                let p0 = (head.0 - m.dx * tail * k0, head.1 - m.dy * tail * k0);
                let p1 = (head.0 - m.dx * tail * k1, head.1 - m.dy * tail * k1);
                let fade = (1.0 - k0).powi(2);
                c.line(p0, p1, m.width * (1.0 - k0 * 0.7), m.tint, 0.9 * fade * bright, true);
            }
            c.glow(head.0, head.1, 16.0 * s, m.tint, 0.9 * bright);
            c.disc(head.0, head.1, m.width * 1.3, Rgb(255.0, 255.0, 255.0), bright, true);
        }
        t < END
    }
}
