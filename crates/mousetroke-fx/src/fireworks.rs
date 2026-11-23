//! 放烟花。
use crate::canvas::{Canvas, Rgb, Rng};
use crate::Effect;

// ---------------------------------------------------------------- 烟花

pub(crate) struct Spark {
    pub x: f32,
    pub y: f32,
    pub px: f32,
    pub py: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub max: f32,
    pub color: Rgb,
    pub size: f32,
    pub seed: f32,
}

struct Rocket { at: f32, x: f32, y: f32, vy: f32, hue: f32, launched: bool, done: bool, kind: u8 }

pub struct Fireworks { s: f32, h: f32, rng: Rng, rockets: Vec<Rocket>, sparks: Vec<Spark>, flashes: Vec<(f32, f32, f32, Rgb)> }

const GRAVITY_ROCKET: f32 = 620.0;

impl Fireworks {
    pub fn new(w: i32, h: i32) -> Self {
        let mut rng = Rng::new();
        let s = h as f32 / 1080.0;
        let hues = [0.08, 0.0, 0.55, 0.92, 0.33, 0.75, 0.14];
        let rockets = [0.0, 0.28, 0.55, 0.85, 1.15, 1.35]
            .iter()
            .enumerate()
            .map(|(i, &at)| Rocket {
                at,
                x: w as f32 * rng.range(0.15, 0.85),
                y: h as f32 + 10.0,
                vy: 0.0,
                hue: hues[(i + (rng.f() * 7.0) as usize) % hues.len()],
                launched: false,
                done: false,
                kind: (rng.f() * 3.0) as u8,
            })
            .collect();
        Self { s, h: h as f32, rng, rockets, sparks: Vec::with_capacity(1500), flashes: Vec::new() }
    }

    fn explode(&mut self, x: f32, y: f32, hue: f32, kind: u8, t: f32) {
        let s = self.s;
        let n = 110 + (self.rng.f() * 50.0) as usize;
        let base = Rgb::hsv(hue, 0.85, 1.0);
        let alt = Rgb::hsv(hue + 0.12, 0.7, 1.0);
        for i in 0..n {
            let a = i as f32 / n as f32 * std::f32::consts::TAU + self.rng.range(-0.05, 0.05);
            // kind 0：实心球；1：圆环；2：两层（内金外彩）
            let sp = match kind {
                1 => self.rng.range(410.0, 440.0),
                2 if i % 2 == 0 => self.rng.range(150.0, 230.0),
                _ => 470.0 * self.rng.f().sqrt(),
            } * s;
            let color = match (kind, i % 2) {
                (2, 0) => Rgb(255.0, 214.0, 120.0),
                _ if self.rng.f() < 0.12 => Rgb(255.0, 255.0, 255.0),
                _ if self.rng.f() < 0.3 => alt,
                _ => base,
            };
            let max = self.rng.range(1.2, 1.9);
            self.sparks.push(Spark { x, y, px: x, py: y, vx: a.cos() * sp, vy: a.sin() * sp, life: max, max, color, size: self.rng.range(1.6, 2.4) * s, seed: self.rng.f() * 10.0 });
        }
        self.flashes.push((x, y, t, base.lerp(Rgb(255.0, 255.0, 255.0), 0.5)));
    }
}

impl Effect for Fireworks {
    fn frame(&mut self, c: &mut Canvas, t: f32, dt: f32) -> bool {
        c.clear(0);
        let s = self.s;
        // 火箭上升，拖一串暖色火星；速度降到 0 时在最高点炸开
        let mut bursts = Vec::new();
        for i in 0..self.rockets.len() {
            let r = &mut self.rockets[i];
            if r.done || t < r.at {
                continue;
            }
            if !r.launched {
                r.launched = true;
                let top = self.h * self.rng.range(0.16, 0.42);
                r.vy = -(2.0 * GRAVITY_ROCKET * s * (r.y - top)).sqrt();
            }
            r.y += r.vy * dt;
            r.vy += GRAVITY_ROCKET * s * dt;
            let (rx, ry) = (r.x + (t * 13.0 + i as f32).sin() * 1.5 * s, r.y);
            for _ in 0..2 {
                self.sparks.push(Spark { x: rx, y: ry, px: rx, py: ry, vx: self.rng.range(-30.0, 30.0) * s, vy: self.rng.range(20.0, 80.0) * s, life: 0.45, max: 0.45, color: Rgb(255.0, 190.0, 110.0), size: 1.4 * s, seed: 0.0 });
            }
            c.glow(rx, ry, 10.0 * s, Rgb(255.0, 220.0, 170.0), 0.8);
            c.disc(rx, ry, 2.2 * s, Rgb(255.0, 245.0, 225.0), 1.0, true);
            if r.vy >= 0.0 {
                r.done = true;
                bursts.push((rx, ry, r.hue, r.kind));
            }
        }
        for (x, y, hue, kind) in bursts {
            self.explode(x, y, hue, kind, t);
        }
        // 炸开瞬间的闪光
        self.flashes.retain(|&(x, y, at, col)| {
            let k = 1.0 - (t - at) / 0.35;
            if k > 0.0 {
                c.glow(x, y, 190.0 * s, col, 0.55 * k * k);
            }
            k > 0.0
        });
        // 火花：空气阻力 + 重力，最后一段闪烁
        let drag = (-1.7 * dt).exp();
        self.sparks.retain_mut(|p| {
            p.life -= dt;
            if p.life <= 0.0 {
                return false;
            }
            p.px = p.x;
            p.py = p.y;
            p.vx *= drag;
            p.vy = p.vy * drag + 230.0 * s * dt;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            let k = p.life / p.max;
            let mut a = k.powf(1.2);
            if k < 0.45 && p.seed > 0.0 {
                a *= 0.55 + 0.45 * (t * 38.0 + p.seed).sin();
            }
            if a > 0.02 {
                c.line((p.px, p.py), (p.x, p.y), p.size * 0.9, p.color, a * 0.5, true);
                c.glow(p.x, p.y, p.size * 4.5, p.color, a * 0.22);
                c.disc(p.x, p.y, p.size, p.color.lerp(Rgb(255.0, 255.0, 255.0), 0.35 * k), a, true);
            }
            true
        });
        !(self.rockets.iter().all(|r| r.done) && self.sparks.is_empty() && self.flashes.is_empty())
    }
}

