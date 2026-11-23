//! 预乘 ARGB（u32 = A<<24 | R<<16 | G<<8 | B）上的软件绘制：发光点、柔和圆、旋转矩形、贴图三角形。
pub struct Canvas<'a> {
    pub px: &'a mut [u32],
    pub w: i32,
    pub h: i32,
}

#[derive(Clone, Copy)]
pub struct Rgb(pub f32, pub f32, pub f32);

impl Rgb {
    pub fn hsv(h: f32, s: f32, v: f32) -> Self {
        let h = h.rem_euclid(1.0) * 6.0;
        let (i, f) = (h.floor() as i32, h.fract());
        let (p, q, t) = (v * (1.0 - s), v * (1.0 - s * f), v * (1.0 - s * (1.0 - f)));
        let (r, g, b) = match i { 0 => (v, t, p), 1 => (q, v, p), 2 => (p, v, t), 3 => (p, q, v), 4 => (t, p, v), _ => (v, p, q) };
        Rgb(r * 255.0, g * 255.0, b * 255.0)
    }
    /// 各分量限制在 0~255：高光叠加后可能超出，直接混合会得到“比不透明还亮”的非法预乘像素。
    pub fn clamped(self) -> Rgb {
        Rgb(self.0.clamp(0.0, 255.0), self.1.clamp(0.0, 255.0), self.2.clamp(0.0, 255.0))
    }
    pub fn lerp(self, o: Rgb, t: f32) -> Rgb {
        Rgb(self.0 + (o.0 - self.0) * t, self.1 + (o.1 - self.1) * t, self.2 + (o.2 - self.2) * t)
    }
}

#[inline]
fn unpack(p: u32) -> [f32; 4] {
    [(p >> 24 & 255) as f32, (p >> 16 & 255) as f32, (p >> 8 & 255) as f32, (p & 255) as f32]
}

#[inline]
fn pack(a: f32, r: f32, g: f32, b: f32) -> u32 {
    let c = |v: f32| v.clamp(0.0, 255.0) as u32;
    c(a) << 24 | c(r) << 16 | c(g) << 8 | c(b)
}

impl Canvas<'_> {
    pub fn clear(&mut self, p: u32) {
        self.px.fill(p);
    }

    /// 普通叠加（预乘 over）。
    #[inline]
    pub fn over(&mut self, i: usize, c: Rgb, a: f32) {
        let (c, a) = (c.clamped(), a.clamp(0.0, 1.0));
        let [da, dr, dg, db] = unpack(self.px[i]);
        let k = 1.0 - a;
        self.px[i] = pack(a * 255.0 + da * k, c.0 * a + dr * k, c.1 * a + dg * k, c.2 * a + db * k);
    }

    /// 加色（光）：颜色相加，透明度取较大者，叠多了会发白发亮。
    #[inline]
    pub fn add(&mut self, i: usize, c: Rgb, a: f32) {
        let (c, a) = (c.clamped(), a.clamp(0.0, 1.0));
        let [da, dr, dg, db] = unpack(self.px[i]);
        self.px[i] = pack((da + a * 255.0).min(255.0), dr + c.0 * a, dg + c.1 * a, db + c.2 * a);
    }

    fn bbox(&self, x: f32, y: f32, r: f32) -> Option<(i32, i32, i32, i32)> {
        let (x0, y0) = ((x - r).floor().max(0.0) as i32, (y - r).floor().max(0.0) as i32);
        let (x1, y1) = (((x + r).ceil() as i32).min(self.w - 1), ((y + r).ceil() as i32).min(self.h - 1));
        (x0 <= x1 && y0 <= y1).then_some((x0, y0, x1, y1))
    }

    /// 实心圆（边缘抗锯齿）。
    pub fn disc(&mut self, x: f32, y: f32, r: f32, c: Rgb, a: f32, additive: bool) {
        let Some((x0, y0, x1, y1)) = self.bbox(x, y, r + 1.0) else { return };
        for py in y0..=y1 {
            for px in x0..=x1 {
                let d = ((px as f32 + 0.5 - x).powi(2) + (py as f32 + 0.5 - y).powi(2)).sqrt();
                let cov = (r - d + 0.5).clamp(0.0, 1.0) * a;
                if cov > 0.003 {
                    let i = (py * self.w + px) as usize;
                    if additive { self.add(i, c, cov) } else { self.over(i, c, cov) }
                }
            }
        }
    }

    /// 柔光：中心最亮、向外平方衰减。
    pub fn glow(&mut self, x: f32, y: f32, r: f32, c: Rgb, a: f32) {
        let Some((x0, y0, x1, y1)) = self.bbox(x, y, r) else { return };
        let inv = 1.0 / r;
        for py in y0..=y1 {
            for px in x0..=x1 {
                let d = ((px as f32 + 0.5 - x).powi(2) + (py as f32 + 0.5 - y).powi(2)).sqrt() * inv;
                if d < 1.0 {
                    let f = (1.0 - d) * (1.0 - d) * a;
                    self.add((py * self.w + px) as usize, c, f);
                }
            }
        }
    }

    /// 柔和的烟团：普通叠加、边缘平方衰减。
    pub fn soft(&mut self, x: f32, y: f32, r: f32, c: Rgb, a: f32) {
        let Some((x0, y0, x1, y1)) = self.bbox(x, y, r) else { return };
        let inv = 1.0 / r;
        for py in y0..=y1 {
            for px in x0..=x1 {
                let d = ((px as f32 + 0.5 - x).powi(2) + (py as f32 + 0.5 - y).powi(2)).sqrt() * inv;
                if d < 1.0 {
                    self.over((py * self.w + px) as usize, c, (1.0 - d) * (1.0 - d) * a);
                }
            }
        }
    }

    /// 圆环（冲击波）。
    pub fn ring(&mut self, x: f32, y: f32, r: f32, width: f32, c: Rgb, a: f32) {
        let Some((x0, y0, x1, y1)) = self.bbox(x, y, r + width) else { return };
        for py in y0..=y1 {
            for px in x0..=x1 {
                let d = ((px as f32 + 0.5 - x).powi(2) + (py as f32 + 0.5 - y).powi(2)).sqrt();
                let t = 1.0 - ((d - r).abs() / width);
                if t > 0.0 {
                    self.add((py * self.w + px) as usize, c, t * t * a);
                }
            }
        }
    }

    /// 线段：沿线盖小圆点（点距半个半径）。
    pub fn line(&mut self, (ax, ay): (f32, f32), (bx, by): (f32, f32), r: f32, c: Rgb, a: f32, additive: bool) {
        let len = ((bx - ax).powi(2) + (by - ay).powi(2)).sqrt();
        let steps = (len / (r * 0.6).max(0.5)).ceil().max(1.0) as i32;
        for s in 0..=steps {
            let t = s as f32 / steps as f32;
            self.disc(ax + (bx - ax) * t, ay + (by - ay) * t, r, c, a / (1.0 + steps as f32 * 0.02), additive);
        }
    }

    /// 细线（约 1 像素宽，Wu 抗锯齿）：沿主轴逐像素走，每步写相邻两个像素。比 line 便宜得多。
    pub fn thin(&mut self, (ax, ay): (f32, f32), (bx, by): (f32, f32), c: Rgb, a: f32, additive: bool) {
        let (dx, dy) = (bx - ax, by - ay);
        let steep = dy.abs() > dx.abs();
        let (n, len) = if steep { (dy.abs().ceil() as i32, dy) } else { (dx.abs().ceil() as i32, dx) };
        if n == 0 {
            return;
        }
        let slope = if steep { dx / len } else { dy / len };
        let (major0, minor0, dir) = if steep { (ay, ax, dy.signum()) } else { (ax, ay, dx.signum()) };
        let (lim_major, lim_minor) = if steep { (self.h, self.w) } else { (self.w, self.h) };
        for i in 0..=n {
            let m = (major0 + dir * i as f32).floor() as i32;
            if m < 0 || m >= lim_major {
                continue;
            }
            let f = minor0 + slope * dir * i as f32 - 0.5;
            let base = f.floor();
            let frac = f - base;
            for (k, cov) in [(base as i32, 1.0 - frac), (base as i32 + 1, frac)] {
                if k < 0 || k >= lim_minor || cov < 0.01 {
                    continue;
                }
                let idx = if steep { (m * self.w + k) as usize } else { (k * self.w + m) as usize };
                if additive { self.add(idx, c, cov * a) } else { self.over(idx, c, cov * a) }
            }
        }
    }

    /// 全屏加一层颜色（整数运算）。
    pub fn add_all(&mut self, c: Rgb, a: f32) {
        let (c, a) = (c.clamped(), a.clamp(0.0, 1.0));
        let (r, g, b) = ((c.0 * a) as u32, (c.1 * a) as u32, (c.2 * a) as u32);
        let al = (a * 255.0) as u32;
        for p in self.px.iter_mut() {
            let v = *p;
            let na = ((v >> 24 & 255) + al).min(255);
            let nr = ((v >> 16 & 255) + r).min(255);
            let ng = ((v >> 8 & 255) + g).min(255);
            let nb = ((v & 255) + b).min(255);
            *p = na << 24 | nr << 16 | ng << 8 | nb;
        }
    }

    /// 旋转矩形（礼花纸片）。half=(半宽, 半高)，角度为弧度；边缘抗锯齿。
    pub fn rect(&mut self, cx: f32, cy: f32, (hw, hh): (f32, f32), angle: f32, c: Rgb, a: f32) {
        let r = (hw * hw + hh * hh).sqrt();
        let Some((x0, y0, x1, y1)) = self.bbox(cx, cy, r + 1.0) else { return };
        let (s, co) = angle.sin_cos();
        for py in y0..=y1 {
            for px in x0..=x1 {
                let (dx, dy) = (px as f32 + 0.5 - cx, py as f32 + 0.5 - cy);
                let (lx, ly) = (dx * co + dy * s, -dx * s + dy * co);
                let cov = (hw - lx.abs() + 0.5).clamp(0.0, 1.0) * (hh - ly.abs() + 0.5).clamp(0.0, 1.0) * a;
                if cov > 0.003 {
                    self.over((py * self.w + px) as usize, c, cov);
                }
            }
        }
    }

    /// 旋转椭圆（金币、红包上的圆章）。rx/ry 为两个半轴，边缘抗锯齿。
    #[allow(clippy::too_many_arguments)]
    pub fn ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, angle: f32, c: Rgb, a: f32) {
        if rx < 0.05 || ry < 0.05 {
            return;
        }
        let Some((x0, y0, x1, y1)) = self.bbox(cx, cy, rx.max(ry) + 1.0) else { return };
        let (s, co) = angle.sin_cos();
        let m = rx.min(ry);
        for py in y0..=y1 {
            for px in x0..=x1 {
                let (dx, dy) = (px as f32 + 0.5 - cx, py as f32 + 0.5 - cy);
                let (lx, ly) = (dx * co + dy * s, -dx * s + dy * co);
                let d = ((lx / rx).powi(2) + (ly / ry).powi(2)).sqrt();
                let cov = ((1.0 - d) * m + 0.5).clamp(0.0, 1.0) * a;
                if cov > 0.003 {
                    self.over((py * self.w + px) as usize, c, cov);
                }
            }
        }
    }

    /// 爱心：隐函数 (x²+y²-1)³ - x²y³ ≤ 0，用梯度估算到边缘的距离做抗锯齿；上浅下深，左上一点高光。
    /// size 约为半宽（像素）。
    #[allow(clippy::too_many_arguments)]
    pub fn heart(&mut self, cx: f32, cy: f32, size: f32, angle: f32, c: Rgb, a: f32) {
        if size < 0.5 {
            return;
        }
        let Some((x0, y0, x1, y1)) = self.bbox(cx, cy, size * 1.45 + 1.0) else { return };
        let (s, co) = angle.sin_cos();
        let inv = 1.0 / size;
        let light = Rgb(c.0 + (255.0 - c.0) * 0.55, c.1 + (255.0 - c.1) * 0.55, c.2 + (255.0 - c.2) * 0.55);
        let dark = Rgb(c.0 * 0.62, c.1 * 0.5, c.2 * 0.55);
        for py in y0..=y1 {
            for px in x0..=x1 {
                let (dx, dy) = (px as f32 + 0.5 - cx, py as f32 + 0.5 - cy);
                let (lx, ly) = (dx * co + dy * s, -dx * s + dy * co);
                // 图形坐标：y 朝上，整体上移一点让视觉中心落在 (cx, cy)
                let (x, y) = (lx * inv * 1.15, -ly * inv * 1.15 + 0.12);
                let q = x * x + y * y - 1.0;
                let f = q * q * q - x * x * y * y * y;
                let gx = 6.0 * x * q * q - 2.0 * x * y * y * y;
                let gy = 6.0 * y * q * q - 3.0 * x * x * y * y;
                let g = (gx * gx + gy * gy).sqrt().max(1e-4);
                let dist = f / g * size / 1.15; // 近似像素距离，内部为负
                let cov = (0.5 - dist).clamp(0.0, 1.0) * a;
                if cov <= 0.003 {
                    continue;
                }
                let shade = ((y + 1.0) / 2.3).clamp(0.0, 1.0);
                let mut col = dark.lerp(c, (shade * 1.6).min(1.0));
                let hl = (1.0 - ((x + 0.45).powi(2) + (y - 0.55).powi(2)) / 0.09).max(0.0);
                col = col.lerp(light, hl * 0.8);
                self.over((py * self.w + px) as usize, col, cov);
            }
        }
    }

    /// 贴图三角形：dst 三个顶点对应 src（截图）里的三个顶点，逐像素反向映射取色（不透明）。
    /// 只画 [row0, row1) 这些行，供多线程分带绘制。shade 用于压暗（模拟碎片翻转背光）。
    #[allow(clippy::too_many_arguments)]
    pub fn textured_tri(&mut self, dst: [(f32, f32); 3], src: [(f32, f32); 3], tex: &[u32], tw: i32, th: i32, shade: f32, glint: f32, rows: (i32, i32), row_base: i32) {
        let (x0, x1) = (dst.iter().map(|p| p.0).fold(f32::MAX, f32::min), dst.iter().map(|p| p.0).fold(f32::MIN, f32::max));
        let (y0, y1) = (dst.iter().map(|p| p.1).fold(f32::MAX, f32::min), dst.iter().map(|p| p.1).fold(f32::MIN, f32::max));
        let (x0, x1) = ((x0.floor() as i32).max(0), (x1.ceil() as i32).min(self.w - 1));
        let (y0, y1) = ((y0.floor() as i32).max(rows.0), (y1.ceil() as i32).min(rows.1 - 1));
        if x0 > x1 || y0 > y1 {
            return;
        }
        let [(ax, ay), (bx, by), (cx, cy)] = dst;
        let area = (bx - ax) * (cy - ay) - (by - ay) * (cx - ax);
        if area.abs() < 1e-3 {
            return;
        }
        let inv = 1.0 / area;
        for py in y0..=y1 {
            let fy = py as f32 + 0.5;
            for px in x0..=x1 {
                let fx = px as f32 + 0.5;
                // 重心坐标
                let w0 = ((bx - fx) * (cy - fy) - (by - fy) * (cx - fx)) * inv;
                let w1 = ((cx - fx) * (ay - fy) - (cy - fy) * (ax - fx)) * inv;
                let w2 = 1.0 - w0 - w1;
                if w0 < -0.002 || w1 < -0.002 || w2 < -0.002 {
                    continue;
                }
                let sx = (w0 * src[0].0 + w1 * src[1].0 + w2 * src[2].0) as i32;
                let sy = (w0 * src[0].1 + w1 * src[1].1 + w2 * src[2].1) as i32;
                let p = tex[(sy.clamp(0, th - 1) * tw + sx.clamp(0, tw - 1)) as usize];
                let p = if shade != 1.0 || glint > 0.0 {
                    let [_, r, g, b] = unpack(p);
                    let gl = glint * 255.0;
                    pack(255.0, r * shade + gl, g * shade + gl, b * shade + gl)
                } else {
                    p
                };
                self.px[((py - row_base) * self.w + px) as usize] = p;
            }
        }
    }

    /// 整体淡出：预乘格式下所有通道同乘一个系数。
    pub fn fade(&mut self, k: f32) {
        let k = (k.clamp(0.0, 1.0) * 256.0) as u32;
        for p in self.px.iter_mut() {
            let v = *p;
            *p = (((v >> 24 & 255) * k) >> 8) << 24 | (((v >> 16 & 255) * k) >> 8) << 16 | (((v >> 8 & 255) * k) >> 8) << 8 | (((v & 255) * k) >> 8);
        }
    }
}

/// 极简随机数（xorshift），演示用。
pub struct Rng(u64);

impl Default for Rng {
    fn default() -> Self {
        Self::new()
    }
}

impl Rng {
    pub fn new() -> Self {
        let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(1);
        Rng(seed | 1)
    }
    pub fn f(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 40) as f32 / (1u64 << 24) as f32
    }
    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.f()
    }
}

/// 低分辨率图层（每边缩小 f 倍），专门画烟、火这类柔和大团，最后双线性放大合成。
/// 存预乘 RGBA（0~255 浮点）。
pub struct Soft {
    pub f: i32,
    lw: i32,
    lh: i32,
    buf: Vec<[f32; 4]>,
    used: bool,
}

impl Soft {
    pub fn new(w: i32, h: i32, f: i32) -> Self {
        let (lw, lh) = (w / f + 2, h / f + 2);
        Self { f, lw, lh, buf: vec![[0.0; 4]; (lw * lh) as usize], used: false }
    }

    pub fn clear(&mut self) {
        self.buf.fill([0.0; 4]);
        self.used = false;
    }

    /// 在图层上画一团：additive=true 为发光相加，否则为普通叠加（烟）。坐标为全分辨率。
    pub fn blob(&mut self, x: f32, y: f32, r: f32, c: Rgb, a: f32, additive: bool) {
        let f = self.f as f32;
        let (x, y, r) = (x / f, y / f, (r / f).max(0.5));
        let (x0, y0) = (((x - r).floor() as i32).max(0), ((y - r).floor() as i32).max(0));
        let (x1, y1) = (((x + r).ceil() as i32).min(self.lw - 1), ((y + r).ceil() as i32).min(self.lh - 1));
        let inv = 1.0 / r;
        for py in y0..=y1 {
            let dy = (py as f32 - y) * inv;
            for px in x0..=x1 {
                let dx = (px as f32 - x) * inv;
                let d2 = dx * dx + dy * dy;
                if d2 >= 1.0 {
                    continue;
                }
                let d = d2.sqrt();
                let k = (1.0 - d) * (1.0 - d) * a;
                let p = &mut self.buf[(py * self.lw + px) as usize];
                if additive {
                    *p = [(p[0] + k * 255.0).min(255.0), p[1] + c.0 * k, p[2] + c.1 * k, p[3] + c.2 * k];
                } else {
                    let m = 1.0 - k;
                    *p = [k * 255.0 + p[0] * m, c.0 * k + p[1] * m, c.1 * k + p[2] * m, c.2 * k + p[3] * m];
                }
                self.used = true;
            }
        }
    }

    /// 有体积感的烟团：按球面法线打光（左上亮），朝下的一面被火光映成暖色。普通叠加。
    pub fn smoke_ball(&mut self, x: f32, y: f32, r: f32, c: Rgb, a: f32, warm: f32) {
        let f = self.f as f32;
        let (x, y, r) = (x / f, y / f, (r / f).max(0.5));
        let (x0, y0) = (((x - r).floor() as i32).max(0), ((y - r).floor() as i32).max(0));
        let (x1, y1) = (((x + r).ceil() as i32).min(self.lw - 1), ((y + r).ceil() as i32).min(self.lh - 1));
        let inv = 1.0 / r;
        let (lx, ly, lz) = (-0.45f32, -0.6f32, 0.66f32);
        for py in y0..=y1 {
            let dy = (py as f32 - y) * inv;
            for px in x0..=x1 {
                let dx = (px as f32 - x) * inv;
                let d2 = dx * dx + dy * dy;
                if d2 >= 1.0 {
                    continue;
                }
                let nz = (1.0 - d2).sqrt();
                let lit = 0.45 + 0.9 * (dx * lx + dy * ly + nz * lz).max(0.0);
                let under = dy.max(0.0) * warm;
                let col = Rgb(c.0 * lit + 210.0 * under, c.1 * lit + 90.0 * under, c.2 * lit + 30.0 * under);
                // 边缘软化：用 1-d² 的平方，比线性更“蓬”
                let k = (1.0 - d2) * (1.0 - d2) * a;
                let p = &mut self.buf[(py * self.lw + px) as usize];
                let m = 1.0 - k;
                *p = [k * 255.0 + p[0] * m, col.0 * k + p[1] * m, col.1 * k + p[2] * m, col.2 * k + p[3] * m];
                self.used = true;
            }
        }
    }

    /// 放大合成到画布：additive 为相加，否则为预乘 over。
    pub fn composite(&self, c: &mut Canvas, additive: bool) {
        if !self.used {
            return;
        }
        let f = self.f as f32;
        let (w, h) = (c.w, c.h);
        let xs: Vec<(usize, f32)> = (0..w).map(|x| {
            let u = (x as f32 + 0.5) / f - 0.5;
            let i = u.floor().max(0.0);
            (i as usize, (u - i).clamp(0.0, 1.0))
        }).collect();
        for y in 0..h {
            let v = (y as f32 + 0.5) / f - 0.5;
            let j = v.floor().max(0.0);
            let fy = (v - j).clamp(0.0, 1.0);
            let r0 = &self.buf[(j as i32 * self.lw) as usize..][..self.lw as usize];
            let r1 = &self.buf[((j as i32 + 1).min(self.lh - 1) * self.lw) as usize..][..self.lw as usize];
            let row = &mut c.px[(y * w) as usize..][..w as usize];
            for (x, &(i, fx)) in xs.iter().enumerate() {
                let i1 = (i + 1).min(self.lw as usize - 1);
                let (a, b, cc, d) = (r0[i], r0[i1], r1[i], r1[i1]);
                if a[0] + b[0] + cc[0] + d[0] < 0.5 {
                    continue;
                }
                let mix = |k: usize| {
                    let top = a[k] + (b[k] - a[k]) * fx;
                    let bot = cc[k] + (d[k] - cc[k]) * fx;
                    top + (bot - top) * fy
                };
                let (sa, sr, sg, sb) = (mix(0), mix(1), mix(2), mix(3));
                let p = row[x];
                let (da, dr, dg, db) = ((p >> 24 & 255) as f32, (p >> 16 & 255) as f32, (p >> 8 & 255) as f32, (p & 255) as f32);
                let (na, nr, ng, nb) = if additive {
                    (da + sa, dr + sr, dg + sg, db + sb)
                } else {
                    let m = 1.0 - sa / 255.0;
                    (sa + da * m, sr + dr * m, sg + dg * m, sb + db * m)
                };
                // 发光叠多了颜色会超过不透明度；限制在不透明度以内，保持合法的预乘值
                let na = na.clamp(0.0, 255.0);
                let cl = |v: f32| v.clamp(0.0, na) as u32;
                row[x] = (na as u32) << 24 | cl(nr) << 16 | cl(ng) << 8 | cl(nb);
            }
        }
    }
}
