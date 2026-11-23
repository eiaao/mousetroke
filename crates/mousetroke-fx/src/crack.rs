//! 裂屏：鼠标处“咔”地一下，蛛网状裂纹向外蔓延；停两秒后从外往里愈合，恢复原样。
//! 只在透明层上画裂纹，不需要截图。
use crate::canvas::{Canvas, Rgb, Rng};
use crate::Effect;

/// 一段裂纹：两端点、离撞击点的距离（用于蔓延和愈合的先后）、粗细。
struct Seg {
    a: (f32, f32),
    b: (f32, f32),
    d0: f32,
    d1: f32,
    width: f32,
}

struct Chip {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    rot: f32,
    vrot: f32,
    size: f32,
}

pub struct Crack {
    s: f32,
    at: (f32, f32),
    segs: Vec<Seg>,
    chips: Vec<Chip>,
    reach_max: f32,
}

const GROW: f32 = 0.22;
const HEAL_START: f32 = 2.3;
const HEAL_END: f32 = 2.95;
const END: f32 = 3.1;

impl Crack {
    pub fn new(w: i32, h: i32, at: (f32, f32)) -> Self {
        let s = h as f32 / 1080.0;
        let mut rng = Rng::new();
        let (ax, ay) = at;
        let dist = |p: (f32, f32)| ((p.0 - ax).powi(2) + (p.1 - ay).powi(2)).sqrt();
        let mut segs = Vec::new();
        let spokes = 13;
        let mut rays: Vec<Vec<(f32, f32)>> = Vec::new();
        let max_len = (w.max(h) as f32) * 0.62;
        for i in 0..spokes {
            let mut ang = (i as f32 + rng.range(-0.3, 0.3)) / spokes as f32 * std::f32::consts::TAU;
            let len = rng.range(0.45, 1.0) * max_len;
            let mut p = (ax, ay);
            let mut pts = vec![p];
            let mut l = 0.0;
            while l < len {
                let step = rng.range(16.0, 42.0) * s * (1.0 + l / (300.0 * s)).min(3.0);
                ang += rng.range(-0.22, 0.22);
                let q = (p.0 + ang.cos() * step, p.1 + ang.sin() * step);
                segs.push(Seg { a: p, b: q, d0: dist(p), d1: dist(q), width: (1.9 - l / (250.0 * s)).max(0.6) * s.max(0.7) });
                // 偶尔分叉
                if rng.f() < 0.16 {
                    let mut bang = ang + rng.range(0.35, 0.8) * if rng.f() < 0.5 { -1.0 } else { 1.0 };
                    let mut bp = q;
                    for _ in 0..rng.range(2.0, 5.0) as i32 {
                        bang += rng.range(-0.25, 0.25);
                        let st = rng.range(14.0, 34.0) * s;
                        let bq = (bp.0 + bang.cos() * st, bp.1 + bang.sin() * st);
                        segs.push(Seg { a: bp, b: bq, d0: dist(bp), d1: dist(bq), width: 0.7 * s.max(0.7) });
                        bp = bq;
                    }
                }
                p = q;
                pts.push(p);
                l += step;
            }
            rays.push(pts);
        }
        // 同心的“蛛网”：相邻两条放射线上、半径相近的点用略折的线连起来
        for ring in [26.0, 62.0, 115.0, 190.0, 290.0] {
            let r = ring * s * rng.range(0.85, 1.15);
            let pick = |ray: &Vec<(f32, f32)>| ray.iter().copied().min_by(|p, q| (dist(*p) - r).abs().total_cmp(&(dist(*q) - r).abs()));
            for i in 0..spokes {
                if rng.f() < 0.25 {
                    continue; // 留些缺口，更自然
                }
                let (Some(p), Some(q)) = (pick(&rays[i]), pick(&rays[(i + 1) % spokes])) else { continue };
                if (dist(p) - r).abs() > r * 0.5 || (dist(q) - r).abs() > r * 0.5 {
                    continue;
                }
                let mid = ((p.0 + q.0) * 0.5 + rng.range(-6.0, 6.0) * s, (p.1 + q.1) * 0.5 + rng.range(-6.0, 6.0) * s);
                let w = 0.9 * s.max(0.7);
                segs.push(Seg { a: p, b: mid, d0: dist(p), d1: dist(mid), width: w });
                segs.push(Seg { a: mid, b: q, d0: dist(mid), d1: dist(q), width: w });
            }
        }
        let reach_max = segs.iter().map(|g| g.d0.max(g.d1)).fold(1.0, f32::max);
        let chips = (0..26)
            .map(|_| {
                let a = rng.range(0.0, std::f32::consts::TAU);
                let sp = rng.range(60.0, 260.0) * s;
                Chip { x: ax, y: ay, vx: a.cos() * sp, vy: a.sin() * sp - 120.0 * s, rot: rng.range(0.0, 6.0), vrot: rng.range(-12.0, 12.0), size: rng.range(1.5, 4.0) * s }
            })
            .collect();
        Self { s, at, segs, chips, reach_max }
    }
}

impl Effect for Crack {
    fn frame(&mut self, c: &mut Canvas, t: f32, dt: f32) -> bool {
        c.clear(0);
        let s = self.s;
        let (ax, ay) = self.at;
        // 蔓延：先快后慢；愈合：从外往里收回
        let reach = if t < GROW {
            self.reach_max * (1.0 - (1.0 - t / GROW).powi(3))
        } else if t < HEAL_START {
            self.reach_max
        } else {
            self.reach_max * (1.0 - ((t - HEAL_START) / (HEAL_END - HEAL_START)).min(1.0)).powf(1.5)
        };
        let healing = t >= HEAL_START;
        // 撞击点：一圈发白的碎粉
        let impact = if t < HEAL_START { 1.0 } else { (1.0 - (t - HEAL_START) / (HEAL_END - HEAL_START)).max(0.0) };
        c.soft(ax, ay, 34.0 * s, Rgb(235.0, 240.0, 248.0), 0.55 * impact);
        c.soft(ax, ay, 14.0 * s, Rgb(255.0, 255.0, 255.0), 0.7 * impact);
        for g in &self.segs {
            // 只画离撞击点 reach 以内的部分；两端谁近就从谁那头截（同心线的两端远近不定）
            if reach < 0.5 || g.d0.min(g.d1) > reach {
                continue;
            }
            let (near, far, dn, df) = if g.d0 <= g.d1 { (g.a, g.b, g.d0, g.d1) } else { (g.b, g.a, g.d1, g.d0) };
            let k = if df > reach { ((reach - dn) / (df - dn).max(0.5)).clamp(0.0, 1.0) } else { 1.0 };
            let (a, b) = (near, (near.0 + (far.0 - near.0) * k, near.1 + (far.1 - near.1) * k));
            if (b.0 - a.0).abs() + (b.1 - a.1).abs() < 0.5 {
                continue;
            }
            let alpha = if healing { 0.9 } else { 1.0 };
            if g.width > 1.2 {
                c.line(a, b, g.width * 0.55, Rgb(14.0, 15.0, 18.0), 0.75 * alpha, false);
            } else {
                c.thin(a, b, Rgb(14.0, 15.0, 18.0), 0.8 * alpha, false);
            }
            c.thin((a.0 + 1.0, a.1 + 1.0), (b.0 + 1.0, b.1 + 1.0), Rgb(235.0, 242.0, 255.0), 0.65 * alpha, true);
        }
        // 愈合的前沿：一圈细碎闪光
        if healing && reach > 2.0 {
            for (i, g) in self.segs.iter().enumerate().filter(|(_, g)| (g.d1 - reach).abs() < 10.0 * s).take(40) {
                let tw = 0.5 + 0.5 * ((t * 40.0) + i as f32).sin();
                c.glow(g.b.0, g.b.1, 7.0 * s, Rgb(220.0, 240.0, 255.0), 0.7 * tw);
            }
        }
        // 崩落的玻璃渣
        let g = 1400.0 * s;
        for ch in &mut self.chips {
            ch.vy += g * dt;
            ch.x += ch.vx * dt;
            ch.y += ch.vy * dt;
            ch.rot += ch.vrot * dt;
            c.rect(ch.x, ch.y, (ch.size, ch.size * 0.6), ch.rot, Rgb(225.0, 235.0, 250.0), 0.8 * impact);
        }
        t < END
    }
}
