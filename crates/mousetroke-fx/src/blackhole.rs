//! 黑洞：鼠标处张开一个黑洞，整个屏幕旋转着被吸进去，坍缩一闪，再弹回原样。
//! 有截图时逐像素反向映射扭曲桌面；没有截图时退化为变暗 + 粒子被吸入，不动桌面本身。
use crate::canvas::{Canvas, Rgb, Rng};
use crate::Effect;

const OPEN: f32 = 0.35;
const SUCK_END: f32 = 1.9;
const COLLAPSE_END: f32 = 2.15;
const SPIT_END: f32 = 2.95;
const END: f32 = 3.25;

struct Mote {
    r: f32,
    a: f32,
    w: f32,
    size: f32,
    color: Rgb,
    pr: f32,
    pa: f32,
    out: f32,
}

pub struct BlackHole {
    s: f32,
    w: i32,
    h: i32,
    tex: Option<Vec<u32>>,
    at: (f32, f32),
    far: f32,
    motes: Vec<Mote>,
}

/// 吸入程度：0 为原样，1 为全部吸进去；弹回时带一点过冲（负值 = 略微放大）。
fn pull(t: f32) -> f32 {
    if t < OPEN {
        0.08 * t / OPEN
    } else if t < SUCK_END {
        let u = (t - OPEN) / (SUCK_END - OPEN);
        0.08 + 0.92 * u.powf(2.2)
    } else if t < COLLAPSE_END {
        1.0
    } else if t < SPIT_END {
        let u = (t - COLLAPSE_END) / (SPIT_END - COLLAPSE_END);
        (-6.0 * u).exp() * (9.0 * u).cos() * (1.0 - u)
    } else {
        0.0
    }
}

/// 视界半径。
fn horizon(t: f32, r0: f32) -> f32 {
    if t < OPEN {
        let u = t / OPEN;
        r0 * (1.0 - (1.0 - u).powi(3))
    } else if t < SUCK_END {
        r0 * (1.0 + 0.45 * (t - OPEN) / (SUCK_END - OPEN))
    } else if t < COLLAPSE_END {
        r0 * 1.45 * (1.0 - (t - SUCK_END) / (COLLAPSE_END - SUCK_END)).powi(2)
    } else {
        0.0
    }
}

#[inline]
fn hash(x: i32, y: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x8da6_b343) ^ (y as u32).wrapping_mul(0xd816_3841);
    h ^= h >> 13;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^ (h >> 16)
}

/// 吸进去之后露出的“太空”：深蓝黑底 + 固定的星点。
#[inline]
fn space(x: i32, y: i32) -> u32 {
    let hsh = hash(x, y);
    if hsh & 511 == 0 {
        let v = 120 + (hsh >> 24) % 136;
        0xFF00_0000 | v << 16 | v << 8 | (v + 20).min(255)
    } else {
        0xFF06_0610
    }
}

impl BlackHole {
    pub fn new(w: i32, h: i32, tex: Option<Vec<u32>>, at: (f32, f32)) -> Self {
        let s = h as f32 / 1080.0;
        let far = [(0.0, 0.0), (w as f32, 0.0), (0.0, h as f32), (w as f32, h as f32)]
            .iter()
            .map(|&(x, y): &(f32, f32)| ((x - at.0).powi(2) + (y - at.1).powi(2)).sqrt())
            .fold(1.0, f32::max);
        let mut rng = Rng::new();
        let n = if tex.is_some() { 90 } else { 260 };
        let motes = (0..n)
            .map(|_| {
                let r = rng.range(0.15, 1.0).sqrt() * far;
                let hue = rng.range(0.55, 0.85);
                Mote {
                    r,
                    a: rng.range(0.0, std::f32::consts::TAU),
                    w: rng.range(0.6, 1.4),
                    size: rng.range(1.0, 2.6) * s,
                    color: if rng.f() < 0.5 { Rgb(255.0, 200.0, 140.0) } else { Rgb::hsv(hue, 0.35, 1.0) },
                    pr: r,
                    pa: 0.0,
                    out: rng.range(0.6, 1.4),
                }
            })
            .collect();
        Self { s, w, h, tex, at, far, motes }
    }

    /// 有截图：反向映射扭曲桌面。映射（源坐标 + 明暗）只在每 GRID 像素的网格点上精确计算，
    /// 中间双线性插值，再逐像素取色；两步都按行分带多线程。
    fn warp(&self, c: &mut Canvas, tex: &[u32], k: f32, hole: f32) {
        const GRID: i32 = 4;
        let (w, h) = (self.w, self.h);
        let (cx, cy) = self.at;
        let far = self.far;
        let keep = (1.0 - k).max(0.0005);
        let swirl = k * 5.0;
        let (gw, gh) = (w / GRID + 2, h / GRID + 2);
        let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).min(8) as i32;
        // 网格：(源 x, 源 y, 亮度)
        let mut grid = vec![(0.0f32, 0.0f32, 1.0f32); (gw * gh) as usize];
        let gband = (gh + threads - 1) / threads;
        std::thread::scope(|scope| {
            for (i, chunk) in grid.chunks_mut((gband * gw) as usize).enumerate() {
                scope.spawn(move || {
                    for (j, cell) in chunk.iter_mut().enumerate() {
                        let (gx, gy) = (j as i32 % gw, i as i32 * gband + j as i32 / gw);
                        let (fx, fy) = ((gx * GRID) as f32 - cx, (gy * GRID) as f32 - cy);
                        let r = (fx * fx + fy * fy).sqrt();
                        let shrink = keep.powf(1.0 + 2.0 * (-r / (0.35 * far)).exp());
                        let rs = r / shrink;
                        let th = fy.atan2(fx) + swirl * (-r / (0.45 * far)).exp();
                        let (sn, cs) = th.sin_cos();
                        // 视界附近变暗（光被吞掉）
                        let dim = if k > 0.0 && hole > 0.0 { (1.0 - 0.9 * (-(r - hole).max(0.0) / (hole * 1.4)).exp()).clamp(0.0, 1.0) } else { 1.0 };
                        *cell = (cx + rs * cs, cy + rs * sn, dim);
                    }
                });
            }
        });
        let grid = &grid;
        let band = (h + threads - 1) / threads;
        let inv = 1.0 / GRID as f32;
        std::thread::scope(|scope| {
            for (i, chunk) in c.px.chunks_mut((band * w) as usize).enumerate() {
                scope.spawn(move || {
                    let row0 = i as i32 * band;
                    for (ry, row) in chunk.chunks_mut(w as usize).enumerate() {
                        let y = row0 + ry as i32;
                        let (gy, fy) = (y / GRID, (y % GRID) as f32 * inv);
                        let dy = y as f32 + 0.5 - cy;
                        for (x, out) in row.iter_mut().enumerate() {
                            let x = x as i32;
                            let dx = x as f32 + 0.5 - cx;
                            if dx * dx + dy * dy < hole * hole {
                                *out = 0xFF00_0000;
                                continue;
                            }
                            let (gx, fx) = (x / GRID, (x % GRID) as f32 * inv);
                            let o = (gy * gw + gx) as usize;
                            let (a, b, cc, d) = (grid[o], grid[o + 1], grid[o + gw as usize], grid[o + gw as usize + 1]);
                            let lerp = |p: f32, q: f32, r: f32, s: f32| {
                                let top = p + (q - p) * fx;
                                top + (r + (s - r) * fx - top) * fy
                            };
                            let (sx, sy, dim) = (lerp(a.0, b.0, cc.0, d.0) as i32, lerp(a.1, b.1, cc.1, d.1) as i32, lerp(a.2, b.2, cc.2, d.2));
                            let base = if sx >= 0 && sy >= 0 && sx < w && sy < h { tex[(sy * w + sx) as usize] } else { space(x, y) };
                            *out = if dim < 0.999 {
                                let m = (dim * 256.0) as u32;
                                0xFF00_0000 | (((base >> 16 & 255) * m) >> 8) << 16 | (((base >> 8 & 255) * m) >> 8) << 8 | (((base & 255) * m) >> 8)
                            } else {
                                base | 0xFF00_0000
                            };
                        }
                    }
                });
            }
        });
    }

    /// 没有截图：越吸越暗的半透明“太空”盖在桌面上，视界处全黑（按行分带多线程）。
    fn veil(&self, c: &mut Canvas, k: f32, hole: f32) {
        let (w, h) = (self.w, self.h);
        let (cx, cy) = self.at;
        let a = k.clamp(0.0, 1.0).powf(1.3) * 0.9;
        let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).min(8) as i32;
        let band = (h + threads - 1) / threads;
        // 视界外 8 倍半径以内才需要算“越近越暗”
        let near_r2 = (hole * 8.0).powi(2);
        std::thread::scope(|scope| {
            for (i, chunk) in c.px.chunks_mut((band * w) as usize).enumerate() {
                scope.spawn(move || {
                    let row0 = i as i32 * band;
                    for (ry, row) in chunk.chunks_mut(w as usize).enumerate() {
                        let y = row0 + ry as i32;
                        let fy = y as f32 + 0.5 - cy;
                        for (x, out) in row.iter_mut().enumerate() {
                            let fx = x as f32 + 0.5 - cx;
                            let r2 = fx * fx + fy * fy;
                            if r2 < hole * hole {
                                *out = 0xFF00_0000;
                                continue;
                            }
                            let al = if hole > 0.0 && r2 < near_r2 {
                                let near = (-(r2.sqrt() - hole) / (hole * 2.0)).exp() * 0.9;
                                (a + (1.0 - a) * near).min(1.0)
                            } else {
                                a
                            };
                            if al < 0.004 {
                                *out = 0;
                                continue;
                            }
                            let sp = space(x as i32, y);
                            let m = (al * 256.0) as u32;
                            *out = ((al * 255.0) as u32) << 24 | (((sp >> 16 & 255) * m) >> 8) << 16 | (((sp >> 8 & 255) * m) >> 8) << 8 | (((sp & 255) * m) >> 8);
                        }
                    }
                });
            }
        });
    }
}

impl Effect for BlackHole {
    fn frame(&mut self, c: &mut Canvas, t: f32, dt: f32) -> bool {
        let s = self.s;
        let (cx, cy) = self.at;
        let k = pull(t);
        let r0 = 60.0 * s;
        let hole = horizon(t, r0);
        match self.tex.as_deref() {
            Some(tex) => self.warp(c, tex, k, hole),
            None => self.veil(c, k, hole),
        }
        // 吸积盘：视界外一圈旋转的光环
        if hole > 0.0 {
            let glow = (t / OPEN).min(1.0);
            c.glow(cx, cy, hole * 3.2, Rgb(255.0, 120.0, 40.0), 0.35 * glow);
            c.ring(cx, cy, hole * 1.18, 7.0 * s, Rgb(255.0, 210.0, 150.0), 0.9 * glow);
            c.ring(cx, cy, hole * 1.5, 16.0 * s, Rgb(170.0, 90.0, 255.0), 0.25 * glow);
            c.disc(cx, cy, hole, Rgb(0.0, 0.0, 0.0), 1.0, false);
        }
        // 粒子：吸入时螺旋向内，弹回时向外散开
        let sucking = t < COLLAPSE_END;
        let spit = (COLLAPSE_END..SPIT_END + 0.2).contains(&t);
        let far = self.far;
        for m in &mut self.motes {
            m.pr = m.r;
            m.pa = m.a;
            if sucking {
                let speed = (0.25 + 1.6 * k) * far * 0.55 * m.w;
                m.r = (m.r - speed * dt * (0.4 + 0.6 * (far / (m.r + 40.0 * s)).min(4.0))).max(0.0);
                m.a += dt * m.w * (1.2 + 900.0 * s / (m.r + 60.0 * s));
            } else if spit {
                m.r += far * 1.8 * m.out * dt;
                m.a -= dt * m.w * 0.8;
            }
        }
        if sucking || spit {
            let fade = if spit { 1.0 - (t - COLLAPSE_END) / (SPIT_END + 0.2 - COLLAPSE_END) } else { (t / 0.3).min(1.0) };
            for m in &self.motes {
                if m.r <= hole * 1.05 && sucking {
                    continue;
                }
                let p0 = (cx + m.pr * m.pa.cos(), cy + m.pr * m.pa.sin());
                let p1 = (cx + m.r * m.a.cos(), cy + m.r * m.a.sin());
                c.line(p0, p1, m.size, m.color, 0.85 * fade, true);
            }
        }
        // 坍缩的一闪
        let flash = 1.0 - ((t - COLLAPSE_END).abs() / 0.12);
        if flash > 0.0 {
            c.glow(cx, cy, far * 0.5 * flash + 40.0 * s, Rgb(255.0, 255.0, 255.0), flash);
            c.add_all(Rgb(255.0, 245.0, 255.0), flash * 0.5);
        }
        // 收尾：没有截图时盖层慢慢消失；有截图时映射已回到原样，淡出看不出接缝
        if t > SPIT_END {
            c.fade(1.0 - (t - SPIT_END) / (END - SPIT_END));
        }
        t < END
    }
}
