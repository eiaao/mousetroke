//! 炸弹：扔进来、落地、引爆、碎屏。
use crate::canvas::{Canvas, Rgb, Rng, Soft};
use crate::fireworks::Spark;
use crate::Effect;

// ---------------------------------------------------------------- 炸弹
//
// 时间线：
//   0 ~ THROW     炸弹从屏幕下角外被扔进来，翻滚着沿抛物线飞向鼠标处，引线一路冒火星；
//   ~ +BOUNCE     落地弹两下、压扁、滚一小段停住；
//   ~ FUSE        引线烧完，炸弹膨胀、闪红；
//   FUSE          引爆：全屏白闪 + 震屏；爆心一圈屏幕直接炸没，露出烧焦的桌面；
//   之后          近处碎片朝镜头飞出，外圈碎片先挂着裂纹、再依次掉落；火球翻滚变黑烟；
//   FADE_START~   整体淡出，桌面恢复。

/// 三角形的三个顶点（像素坐标）。
type Tri = [(f32, f32); 3];

struct Shard { src: [(f32, f32); 3], c: (f32, f32), vx: f32, vy: f32, w: f32, axis: [f32; 3], zv: f32, start: f32, hang: bool }

struct Flame { x: f32, y: f32, vx: f32, vy: f32, r: f32, grow: f32, life: f32, max: f32 }

struct Puff { x: f32, y: f32, r: f32, vx: f32, vy: f32, grow: f32, a: f32, born: f32 }

struct Fleck { x: f32, y: f32, vx: f32, vy: f32, life: f32 }

pub struct Bomb {
    s: f32,
    w: i32,
    h: i32,
    /// 截图；拿不到（如 Mac 未授权屏幕录制）时为 None，改为只画裂纹、不碎屏。
    tex: Option<Vec<u32>>,
    bg: Vec<u32>,
    at: (f32, f32),
    shards: Vec<Shard>,
    flames: Vec<Flame>,
    puffs: Vec<Puff>,
    embers: Vec<Spark>,
    flecks: Vec<Fleck>,
    rng: Rng,
    radius: f32,
    blown: bool,
    smoke: Soft,
    fire: Soft,
}

const THROW: f32 = 0.62;
const BOUNCE: f32 = 0.45;
const FUSE: f32 = 1.85;
const FADE_START: f32 = 4.35;
const BOMB_END: f32 = 4.95;
const SHARD_GRAVITY: f32 = 2200.0;

impl Bomb {
    /// tex：播放前截下的这块屏幕（不透明 ARGB）；at：落点 / 爆点（屏幕内坐标）。
    pub fn new(w: i32, h: i32, tex: Option<Vec<u32>>, at: (f32, f32)) -> Self {
        let mut rng = Rng::new();
        let s = h as f32 / 1080.0;
        let (ax, ay) = at;
        let far = [(0.0, 0.0), (w as f32, 0.0), (0.0, h as f32), (w as f32, h as f32)]
            .iter()
            .map(|&(x, y): &(f32, f32)| ((x - ax).powi(2) + (y - ay).powi(2)).sqrt())
            .fold(0.0, f32::max)
            * 1.08;
        // 放射状裂纹：扇区 × 环，每格切成两个三角形；越靠近爆心越碎
        let sectors = 20;
        let rings = 9;
        let angles: Vec<f32> = (0..sectors).map(|j| (j as f32 + rng.range(-0.35, 0.35)) / sectors as f32 * std::f32::consts::TAU).collect();
        let mut verts = vec![vec![(ax, ay); sectors]; rings + 1];
        for (k, ring) in verts.iter_mut().enumerate().skip(1) {
            for (j, v) in ring.iter_mut().enumerate() {
                let base = far * (k as f32 / rings as f32).powf(1.6);
                let r = if k == rings { far } else { base * rng.range(0.82, 1.18) };
                let a = angles[j] + rng.range(-0.08, 0.08) * (k as f32 / rings as f32);
                *v = (ax + a.cos() * r, ay + a.sin() * r);
            }
        }
        let hole = far * 0.13;
        let mut shards = Vec::new();
        for k in 0..rings {
            for j in 0..sectors {
                let j2 = (j + 1) % sectors;
                let (a, b, c2, d) = (verts[k][j], verts[k][j2], verts[k + 1][j2], verts[k + 1][j]);
                for tri in [[a, b, c2], [a, c2, d]] {
                    let area = ((tri[1].0 - tri[0].0) * (tri[2].1 - tri[0].1) - (tri[1].1 - tri[0].1) * (tri[2].0 - tri[0].0)).abs();
                    if area < 4.0 {
                        continue;
                    }
                    let c = ((tri[0].0 + tri[1].0 + tri[2].0) / 3.0, (tri[0].1 + tri[1].1 + tri[2].1) / 3.0);
                    let (dx, dy) = (c.0 - ax, c.1 - ay);
                    let dist = (dx * dx + dy * dy).sqrt().max(1.0);
                    if dist < hole {
                        continue; // 爆心直接炸没
                    }
                    let rel = (dist / far).min(1.0);
                    let (ux, uy) = (dx / dist, dy / dist);
                    if rel < 0.42 {
                        // 近处：被气浪掀起，朝外、朝镜头飞
                        let sp = rng.range(900.0, 1700.0) * s * (1.25 - rel);
                        shards.push(Shard {
                            src: tri, c,
                            vx: ux * sp + rng.range(-80.0, 80.0) * s,
                            vy: uy * sp - rng.range(100.0, 400.0) * s,
                            w: rng.range(5.0, 12.0) * if rng.f() < 0.5 { -1.0 } else { 1.0 },
                            axis: norm3([rng.range(-1.0, 1.0), rng.range(-1.0, 1.0), rng.range(-0.4, 0.4)]),
                            zv: rng.range(250.0, 1100.0) * s * (1.0 - rel),
                            start: FUSE + rel * 0.12,
                            hang: false,
                        });
                    } else {
                        // 外圈：先裂着挂一会儿，再从里往外陆续塌落
                        shards.push(Shard {
                            src: tri, c,
                            vx: ux * rng.range(40.0, 220.0) * s,
                            vy: uy * rng.range(20.0, 120.0) * s - rng.range(0.0, 160.0) * s,
                            w: rng.range(1.5, 4.5) * if rng.f() < 0.5 { -1.0 } else { 1.0 },
                            axis: norm3([rng.range(-1.0, 1.0), rng.range(-1.0, 1.0), rng.range(-0.3, 0.3)]),
                            zv: rng.range(-60.0, 140.0) * s,
                            start: FUSE + 0.35 + (rel - 0.42) * 1.2 + rng.range(0.0, 0.45),
                            hang: true,
                        });
                    }
                }
            }
        }
        // 背景：烧焦的桌面（暗、偏红褐）+ 爆心一圈余烬发光
        let hot = far * 0.5;
        let bg = match &tex {
            Some(tex) => tex
                .iter()
                .enumerate()
                .map(|(i, &p)| {
                    let (x, y) = ((i as i32 % w) as f32, (i as i32 / w) as f32);
                    let lum = ((p >> 16 & 255) as f32 * 0.3 + (p >> 8 & 255) as f32 * 0.59 + (p & 255) as f32 * 0.11) * 0.14;
                    let d = ((x - ax).powi(2) + (y - ay).powi(2)).sqrt();
                    let k = (1.0 - d / hot).max(0.0).powi(2);
                    let r = 14.0 + lum * 1.15 + 150.0 * k;
                    let g = 11.0 + lum * 0.85 + 55.0 * k * k;
                    let b = 10.0 + lum * 0.7 + 10.0 * k * k;
                    0xFF00_0000 | (r.min(255.0) as u32) << 16 | (g.min(255.0) as u32) << 8 | b.min(255.0) as u32
                })
                .collect(),
            // 没有截图：半透明的焦痕盖在真实桌面上，越靠近爆心越黑、越红
            None => (0..w * h)
                .map(|i| {
                    let (x, y) = ((i % w) as f32, (i / w) as f32);
                    let d = ((x - ax).powi(2) + (y - ay).powi(2)).sqrt();
                    let a = (1.0 - d / (hot * 1.3)).max(0.0).powf(1.4) * 0.92;
                    let k = (1.0 - d / hot).max(0.0).powi(2);
                    let (r, g, b) = ((18.0 + 150.0 * k) * a, (12.0 + 50.0 * k * k) * a, 10.0 * a);
                    ((a * 255.0) as u32) << 24 | (r.min(255.0) as u32) << 16 | (g.min(255.0) as u32) << 8 | b.min(255.0) as u32
                })
                .collect(),
        };
        Self { s, w, h, tex, bg, at, shards, flames: Vec::new(), puffs: Vec::new(), embers: Vec::new(), flecks: Vec::new(), rng, radius: far, blown: false, smoke: Soft::new(w, h, 4), fire: Soft::new(w, h, 4) }
    }

    fn detonate(&mut self) {
        let s = self.s;
        let (ax, ay) = self.at;
        let rng = &mut self.rng;
        for _ in 0..26 {
            let a = rng.range(0.0, std::f32::consts::TAU);
            let sp = rng.range(80.0, 520.0) * s;
            let max = rng.range(0.55, 1.1);
            self.flames.push(Flame { x: ax, y: ay, vx: a.cos() * sp, vy: a.sin() * sp - 60.0 * s, r: rng.range(40.0, 90.0) * s, grow: rng.range(140.0, 300.0) * s, life: max, max });
        }
        for _ in 0..160 {
            let a = rng.range(0.0, std::f32::consts::TAU);
            let sp = rng.range(400.0, 2000.0) * s;
            let max = rng.range(0.5, 1.6);
            let hot = rng.f();
            self.embers.push(Spark {
                x: ax, y: ay, px: ax, py: ay,
                vx: a.cos() * sp, vy: a.sin() * sp - 250.0 * s,
                life: max, max,
                color: Rgb(255.0, 150.0 + 100.0 * hot, 40.0 + 160.0 * hot * hot),
                size: rng.range(1.2, 3.2) * s,
                seed: 0.0,
            });
        }
    }

    /// 引爆前：3D 炸弹 + 燃烧的引线。背景透明。
    fn draw_bomb(&mut self, c: &mut Canvas, t: f32, dt: f32) {
        let s = self.s;
        let (gx, gy) = self.at; // 落点
        let (w, h) = (self.w as f32, self.h as f32);
        // 从离落点较远的那个下角外面扔进来
        let dir = if gx > w * 0.5 { -1.0 } else { 1.0 }; // 飞行方向（+1 向右）
        let roll = 70.0 * s; // 落地后再滚一小段
        let land = (gx - dir * roll, gy);
        let start = (if dir > 0.0 { -90.0 * s } else { w + 90.0 * s }, h * 0.92);
        let peak = (land.1.min(start.1) - h * 0.55).max(-h * 0.15);
        let (ax, ay, scale0, tilt0, kx, ky, height);
        if t < THROW {
            // 二次贝塞尔：水平匀速、竖直抛物线
            let u = t / THROW;
            let ctrl = ((start.0 + land.0) * 0.5, peak);
            let m = 1.0 - u;
            ax = m * m * start.0 + 2.0 * m * u * ctrl.0 + u * u * land.0;
            ay = m * m * start.1 + 2.0 * m * u * ctrl.1 + u * u * land.1;
            scale0 = 1.0 + 0.35 * (std::f32::consts::PI * u).sin(); // 弧顶离镜头更近
            tilt0 = dir * std::f32::consts::TAU * 1.7 * u;
            (kx, ky) = (1.0, 1.0);
            height = gy - ay;
        } else {
            // 弹两下、边滚边停
            let v = ((t - THROW) / BOUNCE).min(1.0);
            let roll_k = 1.0 - (1.0 - v).powi(2);
            ax = land.0 + dir * roll * roll_k;
            let (hop, sq) = if v < 0.6 {
                let q = v / 0.6;
                (34.0 * s * (std::f32::consts::PI * q).sin(), (1.0 - q / 0.12).max(0.0))
            } else if v < 0.9 {
                let q = (v - 0.6) / 0.3;
                (10.0 * s * (std::f32::consts::PI * q).sin(), (1.0 - q / 0.15).max(0.0) * 0.6)
            } else {
                (0.0, (1.0 - (v - 0.9) / 0.1).max(0.0) * 0.3)
            };
            ay = gy - hop;
            scale0 = 1.0;
            // 滚动角度与位移对应（r≈50s），停下后保留最后的角度
            tilt0 = dir * (std::f32::consts::TAU * 1.7 + roll * roll_k / (50.0 * s));
            (kx, ky) = (1.0 + 0.16 * sq, 1.0 - 0.16 * sq);
            height = hop;
        }
        let swell = ((t - (FUSE - 0.28)) / 0.28).clamp(0.0, 1.0);
        let settle = ((t - THROW - BOUNCE) / 0.15).clamp(0.0, 1.0);
        let scale = scale0 * (1.0 + 0.2 * swell * swell) * (1.0 + 0.03 * swell * (t * 70.0).sin());
        let tilt = tilt0 + 0.08 * settle * (t * 7.0).sin() * (1.0 - swell) + 0.06 * swell * (t * 55.0).sin();
        let r = 50.0 * s * scale;
        let ay = ay + (ky - 1.0) * -r; // 压扁时底部贴地
        let (sn, cs) = tilt.sin_cos();
        let rot = |dx: f32, dy: f32| {
            let (dx, dy) = (dx * kx, dy * ky);
            (ax + dx * cs - dy * sn, ay + dx * sn + dy * cs)
        };
        // 引线形状（先算出来，火花位置要用来打光）
        let burn = (t / FUSE).min(1.0);
        let len = 1.0 - 0.85 * burn;
        let segs = 40;
        let path: Vec<(f32, f32)> = (0..=segs)
            .map(|i| {
                let u = i as f32 / segs as f32 * len;
                rot(r * (0.5 + 0.62 * u), -r * (1.12 + 0.7 * u) + (u * 6.5).sin() * r * 0.16 * u)
            })
            .collect();
        let tip = *path.last().unwrap();
        let flick = if t > 0.0 { 0.75 + 0.25 * (t * 63.0).sin() * (t * 23.0).cos() } else { 0.0 };
        c.clear(0);
        // 地面阴影（压扁的椭圆）
        // 离地越高，影子越淡越大
        let lift = (height / (h * 0.5)).clamp(0.0, 1.0);
        let sr = 50.0 * s * (1.0 + 0.6 * lift);
        for k in -3..=3 {
            c.soft(ax + k as f32 * sr * 0.22 + 10.0 * s, gy + 50.0 * s * 1.02, sr * 0.55, Rgb(0.0, 0.0, 0.0), 0.10 * (1.0 - 0.8 * lift));
        }
        // 弹体：逐像素球面光照
        let heat = swell * (0.55 + 0.45 * (t * 38.0).sin().abs());
        let light = norm3([-0.55, -0.65, 0.55]);
        let half = norm3([light[0], light[1], light[2] + 1.0]);
        let tip3 = [(tip.0 - ax) / r, (tip.1 - ay) / r, 0.6];
        let rr = r * kx.max(ky);
        let (x0, x1) = (((ax - rr - 1.0) as i32).max(0), ((ax + rr + 1.0) as i32).min(c.w - 1));
        let (y0, y1) = (((ay - rr - 1.0) as i32).max(0), ((ay + rr + 1.0) as i32).min(c.h - 1));
        for py in y0..=y1 {
            for px in x0..=x1 {
                let (u, v) = ((px as f32 + 0.5 - ax) / (r * kx), (py as f32 + 0.5 - ay) / (r * ky));
                let d2 = u * u + v * v;
                if d2 >= 1.0 + 2.0 / r {
                    continue;
                }
                let nz = (1.0 - d2).max(0.0).sqrt();
                let n = [u, v, nz];
                let dif = dot3(n, light).max(0.0);
                let spec = dot3(n, half).max(0.0).powi(48);
                let rim = (1.0 - nz).powi(3);
                // 引线火花当作点光源
                let lf = norm3([tip3[0] - u, tip3[1] - v, tip3[2] - nz]);
                let fire = dot3(n, lf).max(0.0) * flick;
                // 铸铁颗粒
                let hsh = ((px.wrapping_mul(73856093) ^ py.wrapping_mul(19349663)) & 255) as f32 / 255.0 - 0.5;
                let base = Rgb(42.0 + hsh * 8.0, 44.0 + hsh * 8.0, 52.0 + hsh * 8.0).lerp(Rgb(170.0, 30.0, 15.0), heat);
                let col = Rgb(
                    base.0 * (0.25 + 1.1 * dif) + 230.0 * spec + 70.0 * rim + 120.0 * fire + 160.0 * heat * nz,
                    base.1 * (0.25 + 1.1 * dif) + 235.0 * spec + 90.0 * rim + 70.0 * fire + 30.0 * heat * nz,
                    base.2 * (0.25 + 1.1 * dif) + 245.0 * spec + 120.0 * rim + 25.0 * fire,
                );
                let cov = ((1.0 - d2.sqrt()) * r + 0.5).clamp(0.0, 1.0);
                c.over((py * c.w + px) as usize, col, cov);
            }
        }
        // 引信口：短金属圆柱（横向圆柱光照 + 顶面椭圆）
        let (kx, ky) = rot(r * 0.42, -r * 0.86);
        let ang = tilt + 0.48;
        let (ks, kc) = ang.sin_cos();
        let (hw, hh) = (r * 0.27, r * 0.2);
        let (bx0, bx1) = (((kx - r * 0.4) as i32).max(0), ((kx + r * 0.4) as i32).min(c.w - 1));
        let (by0, by1) = (((ky - r * 0.4) as i32).max(0), ((ky + r * 0.4) as i32).min(c.h - 1));
        for py in by0..=by1 {
            for px in bx0..=bx1 {
                let (dx, dy) = (px as f32 + 0.5 - kx, py as f32 + 0.5 - ky);
                let (lx, ly) = (dx * kc + dy * ks, -dx * ks + dy * kc);
                let top = ((lx / hw).powi(2) + ((ly + hh) / (hh * 0.38)).powi(2)) < 1.0;
                let body = lx.abs() < hw && ly.abs() < hh;
                if !top && !body {
                    continue;
                }
                let col = if top {
                    Rgb(150.0, 152.0, 160.0).lerp(Rgb(60.0, 58.0, 56.0), ((lx / hw) + 1.0) * 0.3)
                } else {
                    let nx = lx / hw;
                    let dif = (-0.6 * nx + 0.8 * (1.0 - nx * nx).max(0.0).sqrt()).max(0.0);
                    let band = (-((nx + 0.45) * 7.0).powi(2)).exp();
                    let v = 45.0 + 120.0 * dif + 150.0 * band;
                    Rgb(v * 0.95, v * 0.95, v)
                };
                let cov = (hw - lx.abs() + 0.5).clamp(0.0, 1.0);
                c.over((py * c.w + px) as usize, col, if top { 1.0 } else { cov });
            }
        }
        // 引线：一节节小圆管，带编织纹，越靠近火越焦黑
        for (i, &p) in path.iter().enumerate() {
            let u = i as f32 / segs as f32;
            let braid = if (i / 2) % 2 == 0 { 1.0 } else { 0.8 };
            let char_ = ((u - 0.85) / 0.15).clamp(0.0, 1.0);
            let col = Rgb(190.0 * braid, 150.0 * braid, 100.0 * braid).lerp(Rgb(40.0, 30.0, 25.0), char_);
            lit_ball(c, p.0, p.1, 3.4 * s * scale.max(0.3), col, light);
        }
        // 火花
        if t > 0.0 {
            c.glow(tip.0, tip.1, 44.0 * s * flick, Rgb(255.0, 150.0, 50.0), 0.8);
            c.glow(tip.0, tip.1, 14.0 * s, Rgb(255.0, 250.0, 220.0), 1.0);
            for _ in 0..5 {
                let a = self.rng.range(0.0, std::f32::consts::TAU);
                let sp = self.rng.range(90.0, 300.0) * s;
                self.flecks.push(Fleck { x: tip.0, y: tip.1, vx: a.cos() * sp, vy: a.sin() * sp - 80.0 * s, life: self.rng.range(0.15, 0.4) });
            }
        }
        let g = 800.0 * s * dt;
        self.flecks.retain_mut(|f| {
            f.life -= dt;
            let (ox, oy) = (f.x, f.y);
            f.vy += g;
            f.x += f.vx * dt;
            f.y += f.vy * dt;
            c.thin((ox, oy), (f.x, f.y), Rgb(255.0, 215.0, 120.0), (f.life * 4.0).min(1.0), true);
            f.life > 0.0
        });
    }

    /// 碎片的 3D 姿态：返回投影后的三个顶点、深度、明暗、高光。
    fn shard_pose(&self, sh: &Shard, t: f32, (ox, oy): (f32, f32)) -> Option<(Tri, f32, f32, f32)> {
        let dt = (t - sh.start).max(0.0);
        let g = if dt <= 0.0 { 0.0 } else { 0.5 * SHARD_GRAVITY * self.s * dt * dt };
        let pos = [sh.c.0 + sh.vx * dt + ox, sh.c.1 + sh.vy * dt + g + oy, sh.zv * dt];
        let ang = sh.w * dt;
        let rotv = |p: [f32; 3]| rodrigues(p, sh.axis, ang);
        let n = rotv([0.0, 0.0, 1.0]);
        let (w, h) = (self.w as f32, self.h as f32);
        let f = h * 1.4; // 焦距
        let (cx, cy) = (w * 0.5, h * 0.5);
        let mut d = [(0.0, 0.0); 3];
        let mut zsum = 0.0;
        for (k, &(x, y)) in sh.src.iter().enumerate() {
            let p = rotv([x - sh.c.0, y - sh.c.1, 0.0]);
            let z = pos[2] + p[2];
            if z > f * 0.85 {
                return None; // 飞过镜头了
            }
            let k2 = f / (f - z);
            d[k] = (cx + (pos[0] + p[0] - cx) * k2, cy + (pos[1] + p[1] - cy) * k2);
            zsum += z;
        }
        let off = d.iter().all(|p| p.1 > h + 4.0) || d.iter().all(|p| p.1 < -4.0) || d.iter().all(|p| p.0 > w + 4.0) || d.iter().all(|p| p.0 < -4.0);
        if off {
            return None;
        }
        // 光照：静止时（法线朝镜头）亮度正好为 1
        let light = norm3([-0.35, -0.55, 0.76]);
        let front = n[2] >= 0.0;
        let nn = if front { n } else { [-n[0], -n[1], -n[2]] };
        let ndl = dot3(nn, light).max(0.0);
        let mut shade = (0.3 + 0.7 * ndl / light[2]).min(1.12);
        if !front {
            shade *= 0.55; // 背面：隔着玻璃看，暗一些
        }
        let r = [2.0 * nn[2] * nn[0], 2.0 * nn[2] * nn[1], 2.0 * nn[2] * nn[2] - 1.0];
        let glint = if dt > 0.0 { dot3(r, light).max(0.0).powi(30) * 0.85 } else { 0.0 };
        Some((d, zsum / 3.0, shade, glint))
    }

    /// 没有截图时：只画外圈碎片的裂纹，慢慢淡去。
    fn draw_cracks(&self, c: &mut Canvas, t: f32, (ox, oy): (f32, f32)) {
        let k = (1.0 - (t - FUSE - 1.2) / 0.8).clamp(0.0, 1.0);
        if k <= 0.0 {
            return;
        }
        for sh in self.shards.iter().filter(|sh| sh.hang) {
            for i in 0..3 {
                let (a, b) = (sh.src[i], sh.src[(i + 1) % 3]);
                let (a, b) = ((a.0 + ox, a.1 + oy), (b.0 + ox, b.1 + oy));
                c.thin(a, b, Rgb(8.0, 8.0, 10.0), 0.75 * k, false);
                c.thin((a.0 + 1.0, a.1 + 1.0), (b.0 + 1.0, b.1 + 1.0), Rgb(225.0, 235.0, 255.0), 0.6 * k, true);
            }
        }
    }

    fn draw_shards(&self, c: &mut Canvas, t: f32, shake: (f32, f32)) {
        let Some(tex) = &self.tex else {
            self.draw_cracks(c, t, shake);
            return;
        };
        let (w, h) = (self.w, self.h);
        let mut poses: Vec<(Tri, Tri, f32, f32, f32, bool)> = self
            .shards
            .iter()
            .filter_map(|sh| {
                let (d, z, shade, glint) = self.shard_pose(sh, t, shake)?;
                Some((d, sh.src, z, shade, glint, t <= sh.start))
            })
            .collect();
        // 远的先画
        poses.sort_by(|a, b| a.2.total_cmp(&b.2));
        let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).min(8) as i32;
        let band = (h + threads - 1) / threads;
        let poses = &poses;
        std::thread::scope(|scope| {
            for (i, chunk) in c.px.chunks_mut((band * w) as usize).enumerate() {
                scope.spawn(move || {
                    let row0 = i as i32 * band;
                    let rows = (row0, (row0 + band).min(h));
                    let mut part = Canvas { px: chunk, w, h };
                    for (d, src, _, shade, glint, _) in poses {
                        part.textured_tri(*d, *src, tex, w, h, *shade, *glint, rows, row0);
                    }
                });
            }
        });
        // 碎片边缘：暗缝 + 玻璃反光，靠近爆心的边被火光染橙；飞散中的淡一些
        let (ax, ay) = self.at;
        for (d, _, _, _, glint, still) in poses {
            let k_edge = if *still { 1.0 } else { 0.5 + glint };
            for k in 0..3 {
                let (a, b) = (d[k], d[(k + 1) % 3]);
                c.thin(a, b, Rgb(6.0, 6.0, 8.0), 0.7 * k_edge.min(1.0), false);
                let mid = ((a.0 + b.0) * 0.5 - ax, (a.1 + b.1) * 0.5 - ay);
                let near = (1.0 - (mid.0 * mid.0 + mid.1 * mid.1).sqrt() / self.radius).clamp(0.0, 1.0);
                let col = Rgb(220.0, 235.0, 255.0).lerp(Rgb(255.0, 170.0, 80.0), near);
                c.thin((a.0 + 1.0, a.1 + 1.0), (b.0 + 1.0, b.1 + 1.0), col, 0.6 * k_edge, true);
            }
        }
    }
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn norm3(a: [f32; 3]) -> [f32; 3] {
    let l = dot3(a, a).sqrt().max(1e-6);
    [a[0] / l, a[1] / l, a[2] / l]
}

/// 绕单位轴 k 转 ang 弧度。
fn rodrigues(v: [f32; 3], k: [f32; 3], ang: f32) -> [f32; 3] {
    let (s, c) = ang.sin_cos();
    let kv = [k[1] * v[2] - k[2] * v[1], k[2] * v[0] - k[0] * v[2], k[0] * v[1] - k[1] * v[0]];
    let kd = dot3(k, v) * (1.0 - c);
    [v[0] * c + kv[0] * s + k[0] * kd, v[1] * c + kv[1] * s + k[1] * kd, v[2] * c + kv[2] * s + k[2] * kd]
}

/// 有光照的小球（引线的一节）。
fn lit_ball(c: &mut Canvas, x: f32, y: f32, r: f32, col: Rgb, light: [f32; 3]) {
    let (x0, x1) = (((x - r - 1.0) as i32).max(0), ((x + r + 1.0) as i32).min(c.w - 1));
    let (y0, y1) = (((y - r - 1.0) as i32).max(0), ((y + r + 1.0) as i32).min(c.h - 1));
    for py in y0..=y1 {
        for px in x0..=x1 {
            let (u, v) = ((px as f32 + 0.5 - x) / r, (py as f32 + 0.5 - y) / r);
            let d2 = u * u + v * v;
            if d2 >= 1.0 + 1.5 / r {
                continue;
            }
            let n = [u, v, (1.0 - d2).max(0.0).sqrt()];
            let k = 0.3 + 0.9 * dot3(n, light).max(0.0);
            let cov = ((1.0 - d2.sqrt()) * r + 0.5).clamp(0.0, 1.0);
            c.over((py * c.w + px) as usize, Rgb(col.0 * k, col.1 * k, col.2 * k), cov);
        }
    }
}

impl Effect for Bomb {
    fn frame(&mut self, c: &mut Canvas, t: f32, dt: f32) -> bool {
        let s = self.s;
        let (ax, ay) = self.at;
        if t < FUSE {
            c.clear(0);
            self.draw_bomb(c, t, dt);
            return true;
        }
        if !self.blown {
            self.blown = true;
            self.detonate();
        }
        let tb = t - FUSE;
        // 震屏：快速衰减的抖动
        let amp = 22.0 * s * (-tb * 7.0).exp();
        let shake = (amp * (t * 97.0).sin(), amp * (t * 83.0 + 1.3).cos());
        // 背景（随震屏平移）
        let (w, h) = (self.w, self.h);
        let (sx, sy) = (shake.0.round() as i32, shake.1.round() as i32);
        for y in 0..h {
            let row = &mut c.px[(y * w) as usize..((y + 1) * w) as usize];
            let src_y = (y - sy).clamp(0, h - 1);
            let src = &self.bg[(src_y * w) as usize..((src_y + 1) * w) as usize];
            if sx >= 0 {
                row[sx as usize..].copy_from_slice(&src[..(w - sx) as usize]);
                row[..sx as usize].fill(src[0]);
            } else {
                let k = (-sx) as usize;
                row[..w as usize - k].copy_from_slice(&src[k..]);
                row[w as usize - k..].fill(src[w as usize - 1]);
            }
        }
        // 黑烟（在碎片后面）
        self.smoke.clear();
        for p in &mut self.puffs {
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.r += p.grow * dt;
            p.grow *= (-dt).exp();
            p.vy -= 40.0 * s * dt;
            let fade_in = ((t - p.born) / 0.25).min(1.0);
            p.a *= (-0.55 * dt).exp();
            let warm = (1.0 - (t - FUSE) / 1.6).clamp(0.0, 1.0) * 0.9;
            self.smoke.smoke_ball(p.x + shake.0, p.y + shake.1, p.r, Rgb(46.0, 42.0, 40.0), p.a * fade_in * 1.3, warm);
        }
        self.smoke.composite(c, false);
        self.smoke.clear();
        self.draw_shards(c, t, shake);
        // 火球：一团团火焰，白 → 黄 → 橙 → 红，烧完变黑烟
        let mut new_puffs = Vec::new();
        for f in &mut self.flames {
            f.life -= dt;
            let drag = (-3.0 * dt).exp();
            f.vx *= drag;
            f.vy = f.vy * drag - 120.0 * s * dt;
            f.x += f.vx * dt;
            f.y += f.vy * dt;
            f.r += f.grow * dt;
            f.grow *= (-2.0 * dt).exp();
            let age = 1.0 - (f.life / f.max).max(0.0);
            let col = if age < 0.25 {
                Rgb(255.0, 250.0, 225.0).lerp(Rgb(255.0, 200.0, 70.0), age / 0.25)
            } else if age < 0.6 {
                Rgb(255.0, 200.0, 70.0).lerp(Rgb(240.0, 90.0, 25.0), (age - 0.25) / 0.35)
            } else {
                Rgb(240.0, 90.0, 25.0).lerp(Rgb(120.0, 25.0, 10.0), (age - 0.6) / 0.4)
            };
            if f.life > 0.0 {
                self.fire.blob(f.x + shake.0, f.y + shake.1, f.r, col, 0.75 * (1.0 - age * age), true);
            } else if f.max > 0.0 {
                f.max = 0.0; // 只转一次
                new_puffs.push(Puff { x: f.x, y: f.y, r: f.r * 0.9, vx: f.vx * 0.3, vy: f.vy * 0.3 - 30.0 * s, grow: 90.0 * s, a: 0.55, born: t - 0.25 });
            }
        }
        self.flames.retain(|f| f.max > 0.0);
        self.fire.composite(c, true);
        self.fire.clear();
        self.puffs.extend(new_puffs);
        // 爆心白热核
        let core = (1.0 - tb / 0.35).max(0.0);
        if core > 0.0 {
            c.glow(ax + shake.0, ay + shake.1, (90.0 + 380.0 * tb) * s, Rgb(255.0, 245.0, 220.0), core);
        }
        // 冲击波：亮环 + 外侧淡环
        let wave = (tb / 0.6).min(1.0);
        if wave < 1.0 {
            let r = self.radius * 1.05 * (1.0 - (1.0 - wave).powi(2));
            c.ring(ax + shake.0, ay + shake.1, r, 18.0 * s, Rgb(255.0, 235.0, 200.0), 0.8 * (1.0 - wave));
            c.ring(ax + shake.0, ay + shake.1, r * 0.82, 40.0 * s, Rgb(255.0, 150.0, 60.0), 0.35 * (1.0 - wave));
        }
        // 火星
        let drag = (-1.2 * dt).exp();
        let g = 1100.0 * s * dt;
        self.embers.retain_mut(|p| {
            p.life -= dt;
            p.px = p.x;
            p.py = p.y;
            p.vx *= drag;
            p.vy = p.vy * drag + g;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            let a = (p.life / p.max).max(0.0);
            c.line((p.px, p.py), (p.x, p.y), p.size, p.color, a, true);
            p.life > 0.0
        });
        // 全屏闪光：先白后橙
        let flash = (1.0 - tb / 0.18).max(0.0);
        if flash > 0.0 {
            let col = Rgb(255.0, 255.0, 255.0).lerp(Rgb(255.0, 180.0, 90.0), 1.0 - flash);
            c.add_all(col, flash * 0.85);
        }
        if t > FADE_START {
            c.fade(1.0 - (t - FADE_START) / (BOMB_END - FADE_START));
        }
        t < BOMB_END
    }
}
