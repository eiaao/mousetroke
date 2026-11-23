//! 红包雨：红包和金币从屏幕上方落下，边落边翻转；落到底部弹一下再掉出去；金币正对镜头时闪一下。
use crate::canvas::{Canvas, Rgb, Rng};
use crate::Effect;

struct Item {
    coin: bool,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    rot: f32,
    vrot: f32,
    flip: f32,
    vflip: f32,
    size: f32,
    sway: f32,
    bounced: bool,
    born: f32,
}

struct Glint {
    x: f32,
    y: f32,
    life: f32,
    size: f32,
}

pub struct MoneyRain {
    s: f32,
    w: f32,
    h: f32,
    rng: Rng,
    items: Vec<Item>,
    glints: Vec<Glint>,
    spawned: usize,
    total: usize,
}

const SPAWN_END: f32 = 2.6;
const FADE_START: f32 = 4.0;
const END: f32 = 4.6;

const RED: Rgb = Rgb(222.0, 32.0, 38.0);
const RED_DARK: Rgb = Rgb(160.0, 18.0, 26.0);
const GOLD: Rgb = Rgb(255.0, 200.0, 60.0);
const GOLD_DARK: Rgb = Rgb(196.0, 130.0, 20.0);

impl MoneyRain {
    pub fn new(w: i32, h: i32) -> Self {
        let s = h as f32 / 1080.0;
        // 宽屏多一些，保持密度
        let total = (110.0 * (w as f32 / h as f32) / (16.0 / 9.0)).clamp(60.0, 200.0) as usize;
        Self { s, w: w as f32, h: h as f32, rng: Rng::new(), items: Vec::new(), glints: Vec::new(), spawned: 0, total }
    }

    fn spawn(&mut self, t: f32) {
        let s = self.s;
        let rng = &mut self.rng;
        let coin = rng.f() < 0.45;
        let size = if coin { rng.range(14.0, 22.0) } else { rng.range(22.0, 34.0) } * s;
        self.items.push(Item {
            coin,
            x: rng.range(-0.02, 1.02) * self.w,
            y: -size * 2.5 - rng.range(0.0, 60.0) * s,
            vx: rng.range(-40.0, 40.0) * s,
            vy: rng.range(150.0, 380.0) * s,
            rot: rng.range(-0.5, 0.5),
            vrot: rng.range(-2.0, 2.0),
            flip: rng.range(0.0, std::f32::consts::TAU),
            vflip: rng.range(3.0, 9.0) * if rng.f() < 0.5 { -1.0 } else { 1.0 },
            size,
            sway: rng.range(0.0, std::f32::consts::TAU),
            bounced: false,
            born: t,
        });
    }

    fn draw_envelope(c: &mut Canvas, it: &Item, a: f32) {
        let k = it.flip.cos();
        let hw = it.size * 0.62 * k.abs().max(0.06);
        let hh = it.size;
        let front = k >= 0.0;
        let (sn, cs) = it.rot.sin_cos();
        // 局部坐标 (lx, ly) 转屏幕
        let at = |lx: f32, ly: f32| (it.x + lx * cs - ly * sn, it.y + lx * sn + ly * cs);
        c.rect(it.x, it.y, (hw, hh), it.rot, if front { RED } else { RED_DARK }, a);
        if front {
            // 封口：上部一条稍深的带子
            let (fx, fy) = at(0.0, -hh * 0.52);
            c.rect(fx, fy, (hw, hh * 0.48), it.rot, Rgb(196.0, 24.0, 32.0), a);
            // 金色圆章
            let (gx, gy) = at(0.0, -hh * 0.06);
            let r = it.size * 0.26;
            c.ellipse(gx, gy, r * k.abs().max(0.06), r, it.rot, GOLD, a);
            c.ellipse(gx, gy, r * 0.62 * k.abs().max(0.06), r * 0.62, it.rot, GOLD_DARK, a * 0.9);
        }
    }

    fn draw_coin(c: &mut Canvas, it: &Item, a: f32) {
        let k = it.flip.cos();
        let r = it.size;
        let rx = r * k.abs().max(0.08);
        // 侧边厚度：翻到侧面时露出一条深色边
        c.ellipse(it.x + (1.0 - k.abs()) * r * 0.12, it.y, rx + r * 0.1 * (1.0 - k.abs()), r, it.rot, GOLD_DARK, a);
        c.ellipse(it.x, it.y, rx, r, it.rot, if k >= 0.0 { GOLD } else { Rgb(240.0, 180.0, 50.0) }, a);
        c.ellipse(it.x, it.y, rx * 0.72, r * 0.72, it.rot, Rgb(236.0, 168.0, 34.0), a);
        // 方孔
        c.rect(it.x, it.y, (r * 0.2 * k.abs().max(0.08), r * 0.2), it.rot, Rgb(150.0, 92.0, 12.0), a);
        // 高光
        let (sn, cs) = it.rot.sin_cos();
        let (lx, ly) = (-rx * 0.45, -r * 0.45);
        c.disc(it.x + lx * cs - ly * sn, it.y + lx * sn + ly * cs, r * 0.16 * k.abs(), Rgb(255.0, 250.0, 220.0), a * 0.8, false);
    }
}

impl Effect for MoneyRain {
    fn frame(&mut self, c: &mut Canvas, t: f32, dt: f32) -> bool {
        c.clear(0);
        let s = self.s;
        // 按时间均匀投放（前密后疏）
        let want = ((t / SPAWN_END).min(1.0).powf(0.8) * self.total as f32) as usize;
        while self.spawned < want {
            self.spawn(t);
            self.spawned += 1;
        }
        let g = 900.0 * s;
        let floor = self.h;
        let fade = if t > FADE_START { 1.0 - (t - FADE_START) / (END - FADE_START) } else { 1.0 };
        for it in &mut self.items {
            // 空气阻力：越快阻力越大，趋于收尾速度
            let vmax = if it.coin { 820.0 } else { 560.0 } * s;
            it.vy += g * dt;
            it.vy -= (it.vy / vmax).powi(2) * g * dt * it.vy.signum();
            it.sway += dt * 2.2;
            let sway = if it.coin { 0.0 } else { it.sway.sin() * 70.0 * s };
            it.x += (it.vx + sway) * dt;
            it.y += it.vy * dt;
            it.rot += it.vrot * dt;
            it.flip += it.vflip * dt;
            if !it.bounced && it.y + it.size > floor {
                it.bounced = true;
                it.y = floor - it.size;
                it.vy = -it.vy * 0.38;
                it.vrot *= -1.4;
            }
        }
        // 金币翻到正面时闪一下
        for it in self.items.iter().filter(|it| it.coin && it.flip.cos() > 0.985 && it.y > 0.0) {
            if self.glints.len() < 40 {
                self.glints.push(Glint { x: it.x - it.size * 0.3, y: it.y - it.size * 0.4, life: 0.25, size: it.size * 0.9 });
            }
        }
        self.items.retain(|it| it.y - it.size * 2.0 < self.h);
        for it in &self.items {
            // 刚出现的一小段时间淡入，避免从顶边突然冒出
            let a = ((t - it.born) / 0.12).min(1.0) * fade;
            if it.coin { Self::draw_coin(c, it, a) } else { Self::draw_envelope(c, it, a) }
        }
        self.glints.retain_mut(|gl| {
            gl.life -= dt;
            let k = (gl.life / 0.25).max(0.0);
            let r = gl.size * (0.4 + 0.6 * k);
            c.glow(gl.x, gl.y, r * 0.6, Rgb(255.0, 240.0, 180.0), 0.7 * k * fade);
            c.thin((gl.x - r, gl.y), (gl.x + r, gl.y), Rgb(255.0, 255.0, 230.0), k * fade, true);
            c.thin((gl.x, gl.y - r), (gl.x, gl.y + r), Rgb(255.0, 255.0, 230.0), k * fade, true);
            gl.life > 0.0
        });
        t < END && (t < SPAWN_END || !self.items.is_empty())
    }
}
