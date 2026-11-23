//! 爱心：鼠标处先“怦”地跳出一颗大心，随后一串大小不一的爱心左右摇摆着往上飘。
use crate::canvas::{Canvas, Rgb, Rng};
use crate::Effect;

struct Heart {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    size: f32,
    color: Rgb,
    born: f32,
    life: f32,
    phase: f32,
    wobble: f32,
}

pub struct Hearts {
    s: f32,
    at: (f32, f32),
    rng: Rng,
    hearts: Vec<Heart>,
    spawned: usize,
}

const COUNT: usize = 34;
const SPAWN_END: f32 = 1.7;
const BIG_END: f32 = 1.5;
const END: f32 = 4.4;

/// 弹出时的放大曲线：0 → 1.18 → 1。
fn pop(x: f32) -> f32 {
    if x >= 1.0 {
        return 1.0;
    }
    let (c1, c3) = (1.9, 2.9);
    1.0 + c3 * (x - 1.0).powi(3) + c1 * (x - 1.0).powi(2)
}

impl Hearts {
    pub fn new(_w: i32, h: i32, at: (f32, f32)) -> Self {
        Self { s: h as f32 / 1080.0, at, rng: Rng::new(), hearts: Vec::new(), spawned: 0 }
    }

    fn spawn(&mut self, t: f32) {
        let s = self.s;
        let rng = &mut self.rng;
        // 粉到红之间，偶尔一颗偏紫
        let hue = if rng.f() < 0.15 { rng.range(0.86, 0.9) } else { rng.range(0.94, 1.0) };
        self.hearts.push(Heart {
            x: self.at.0 + rng.range(-18.0, 18.0) * s,
            y: self.at.1 + rng.range(-10.0, 10.0) * s,
            vx: rng.range(-150.0, 150.0) * s,
            vy: -rng.range(140.0, 300.0) * s,
            size: rng.range(12.0, 34.0) * s,
            color: Rgb::hsv(hue, rng.range(0.55, 0.85), 1.0),
            born: t,
            life: rng.range(1.8, 2.6),
            phase: rng.range(0.0, std::f32::consts::TAU),
            wobble: rng.range(1.8, 3.2),
        });
    }
}

impl Effect for Hearts {
    fn frame(&mut self, c: &mut Canvas, t: f32, dt: f32) -> bool {
        c.clear(0);
        let s = self.s;
        // 中间的大心：弹出、跳两下（像心跳），再淡出
        if t < BIG_END {
            let beat = 1.0 + 0.09 * (-((t - 0.45) * 14.0).powi(2)).exp() + 0.07 * (-((t - 0.75) * 14.0).powi(2)).exp();
            let k = pop(t / 0.3) * beat;
            let a = (1.0 - ((t - 1.1) / 0.4).max(0.0)).clamp(0.0, 1.0);
            let (x, y) = (self.at.0, self.at.1 - 30.0 * s * (t / BIG_END));
            c.glow(x, y, 120.0 * s * k, Rgb(255.0, 90.0, 140.0), 0.35 * a);
            c.heart(x, y, 64.0 * s * k, 0.0, Rgb(244.0, 52.0, 96.0), a);
        }
        let want = if t < 0.25 { 0 } else { (((t - 0.25) / (SPAWN_END - 0.25)).min(1.0) * COUNT as f32) as usize };
        while self.spawned < want {
            self.spawn(t);
            self.spawned += 1;
        }
        for h in &mut self.hearts {
            h.vx *= (-1.6 * dt).exp();
            h.vy *= (-0.4 * dt).exp();
            h.phase += h.wobble * dt;
            h.x += (h.vx + h.phase.sin() * 40.0 * s) * dt;
            h.y += h.vy * dt;
        }
        self.hearts.retain(|h| t - h.born < h.life);
        for h in &self.hearts {
            let age = t - h.born;
            let k = pop(age / 0.28);
            let a = (1.0 - ((age - (h.life - 0.6)) / 0.6).max(0.0)).clamp(0.0, 1.0);
            c.heart(h.x, h.y, h.size * k, h.phase.sin() * 0.25, h.color, a);
        }
        t < END && (t < SPAWN_END || !self.hearts.is_empty())
    }
}
