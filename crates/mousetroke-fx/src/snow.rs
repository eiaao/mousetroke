//! 下雪：远近三层雪花（远小慢淡、近大快亮）随风飘落，约 10 秒后停。
use crate::canvas::{Canvas, Rgb, Rng};
use crate::Effect;

struct Flake {
    x: f32,
    y: f32,
    z: f32,
    phase: f32,
    freq: f32,
}

pub struct Snow {
    s: f32,
    w: f32,
    h: f32,
    rng: Rng,
    flakes: Vec<Flake>,
    budget: f32,
}

const SPAWN_END: f32 = 7.0;
const FADE_START: f32 = 9.2;
const END: f32 = 10.0;

impl Snow {
    pub fn new(w: i32, h: i32) -> Self {
        Self { s: h as f32 / 1080.0, w: w as f32, h: h as f32, rng: Rng::new(), flakes: Vec::new(), budget: 0.0 }
    }

    fn spawn(&mut self, top: bool) {
        let rng = &mut self.rng;
        let z = rng.f().powf(1.6); // 远处的多，近处的少
        let y = if top { -10.0 * self.s - rng.range(0.0, 40.0) * self.s } else { rng.range(-0.1, 0.6) * self.h };
        self.flakes.push(Flake { x: rng.range(-0.05, 1.05) * self.w, y, z, phase: rng.range(0.0, std::f32::consts::TAU), freq: rng.range(0.6, 1.6) });
    }
}

impl Effect for Snow {
    fn frame(&mut self, c: &mut Canvas, t: f32, dt: f32) -> bool {
        c.clear(0);
        let s = self.s;
        // 每秒生成量随屏幕宽度变化；开头先在上半屏撒一些，免得要等雪落下来
        if self.flakes.is_empty() && t < 0.1 {
            for _ in 0..(60.0 * self.w / self.h) as usize {
                self.spawn(false);
            }
        }
        if t < SPAWN_END {
            let ramp = (t / 1.5).min(1.0) * (1.0 - ((t - SPAWN_END + 1.5) / 1.5).max(0.0));
            self.budget += dt * 70.0 * (self.w / self.h) * ramp;
            while self.budget >= 1.0 {
                self.budget -= 1.0;
                self.spawn(true);
            }
        }
        let wind = 18.0 * s + 14.0 * s * (t * 0.5).sin();
        let fade = if t > FADE_START { 1.0 - (t - FADE_START) / (END - FADE_START) } else { (t / 0.6).min(1.0) };
        for f in &mut self.flakes {
            f.phase += f.freq * dt;
            let fall = (35.0 + 110.0 * f.z) * s;
            f.y += fall * dt;
            f.x += (wind * (0.4 + f.z) + f.phase.sin() * (12.0 + 30.0 * f.z) * s) * dt;
        }
        self.flakes.retain(|f| f.y < self.h + 20.0 * s && f.x > -40.0 * s && f.x < self.w + 40.0 * s);
        for f in &self.flakes {
            let r = (1.1 + 3.6 * f.z) * s.max(0.6);
            let a = (0.45 + 0.55 * f.z) * fade;
            if f.z > 0.65 {
                c.glow(f.x, f.y, r * 3.0, Rgb(220.0, 235.0, 255.0), 0.25 * a);
                c.disc(f.x, f.y, r, Rgb(250.0, 252.0, 255.0), a, false);
            } else {
                c.disc(f.x, f.y, r, Rgb(240.0, 246.0, 255.0), a * 0.85, false);
            }
        }
        t < END && (t < SPAWN_END || !self.flakes.is_empty())
    }
}
