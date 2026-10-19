//! 手势识别：鼠标轨迹 → 网格量化折线 → 与 trace 点序列匹配。
//!
//! **trace 编码（已破解，2026-10-16）**：`points` 是偶数长度的 `i32` 数组，
//! 为**归一化绝对坐标点序列** `(x0,y0, x1,y1, …)`，值域 0~100（单位 50 = 一格）。
//! 匹配时取相邻点差分得到期望位移段，与量化轨迹段比对。
//!
//! 例：旧格式"截图"手势 `[0,50, 0,-50, 50,0, 0,50, -50,0]` = ↑ → ↓ ← 画方框。
//! 验证样本：刷新 = ←↑→、ESC = →↑←、最小化 = ↙ 对角线、新建标签 = →↓。

/// 量化后的轨迹段（网格步数，x 正=右、y 正=下）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Segment {
    pub x: i32,
    pub y: i32,
}

/// 轨迹追踪器：增量喂入鼠标点，按 `grid`（px）量化为网格步数段，同向段合并。
#[derive(Debug, Clone)]
pub struct PathTracker {
    grid: f64,
    anchor: (f64, f64),
    segments: Vec<Segment>,
}

impl PathTracker {
    pub fn new(grid: f64) -> Self {
        Self { grid: grid.max(1.0), anchor: (0.0, 0.0), segments: Vec::new() }
    }

    /// 手势开始时定位锚点并清空历史。
    pub fn start(&mut self, x: f64, y: f64) {
        self.anchor = (x, y);
        self.segments.clear();
    }

    pub fn add_point(&mut self, x: f64, y: f64) {
        let (dx, dy) = (x - self.anchor.0, y - self.anchor.1);
        if dx.hypot(dy) < self.grid {
            return; // 位移不足一格
        }
        let sx = (dx / self.grid).round() as i32;
        let sy = (dy / self.grid).round() as i32;
        if sx == 0 && sy == 0 {
            return;
        }
        match self.segments.last_mut() {
            Some(last) if last.x.signum() == sx.signum() && last.y.signum() == sy.signum() => {
                last.x += sx;
                last.y += sy;
            }
            _ => self.segments.push(Segment { x: sx, y: sy }),
        }
        self.anchor = (x, y);
    }

    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// 去噪后的段（识别用）。见 [`denoise`]。
    pub fn strokes(&self) -> Vec<Segment> {
        denoise(&self.segments)
    }
}

fn seg_len(s: &Segment) -> i32 {
    s.x.abs().max(s.y.abs())
}

/// 去除手抖产生的噪声段：
/// - 拐角过渡：1 格的斜段夹在两段 ≥2 格之间，且其 x/y 方向分别来自前后两段（如 ← ↖ ↑ 中的 ↖）
/// - 首尾钩子：起笔/收笔处 1 格的小段（最长段 ≥4 格时）
///
/// 之后合并同象限的相邻段。
pub fn denoise(segs: &[Segment]) -> Vec<Segment> {
    let longest = segs.iter().map(seg_len).max().unwrap_or(0);
    let n = segs.len();
    let mut kept: Vec<Segment> = Vec::with_capacity(n);
    for (i, s) in segs.iter().enumerate() {
        if seg_len(s) == 1 && n > 1 {
            let hook = (i == 0 || i == n - 1) && longest >= 4;
            let corner = i > 0 && i + 1 < n && s.x != 0 && s.y != 0 && {
                let (p, q) = (&segs[i - 1], &segs[i + 1]);
                let from = |a: i32, b: i32| a.signum() == b.signum() && b != 0;
                seg_len(p) >= 2
                    && seg_len(q) >= 2
                    && ((from(s.x, p.x) && from(s.y, q.y)) || (from(s.y, p.y) && from(s.x, q.x)))
            };
            if hook || corner {
                continue;
            }
        }
        match kept.last_mut() {
            Some(last) if last.x.signum() == s.x.signum() && last.y.signum() == s.y.signum() => {
                last.x += s.x;
                last.y += s.y;
            }
            _ => kept.push(*s),
        }
    }
    kept
}

/// 笔画的方向箭头表示（如 `→ ↓ →`），用于屏幕上"未识别"提示。
/// 每段按 8 方向吸附（次轴 < 主轴×tan22.5° 视为纯轴向）。
pub fn arrows(segs: &[Segment]) -> String {
    let tan = std::f64::consts::FRAC_PI_8.tan();
    segs.iter()
        .map(|s| {
            let (ax, ay) = (s.x.abs() as f64, s.y.abs() as f64);
            let (x, y) = if ay < ax * tan {
                (s.x.signum(), 0)
            } else if ax < ay * tan {
                (0, s.y.signum())
            } else {
                (s.x.signum(), s.y.signum())
            };
            match (x, y) {
                (1, 0) => '→',
                (-1, 0) => '←',
                (0, -1) => '↑',
                (0, 1) => '↓',
                (1, -1) => '↗',
                (-1, -1) => '↖',
                (1, 1) => '↘',
                _ => '↙',
            }
        })
        .map(String::from)
        .collect::<Vec<_>>()
        .join(" ")
}

/// 匹配分接受阈值（归一化后）。低于 [`EXACT_EPS`] 视为精确。
pub const EXACT_EPS: f64 = 0.25;
/// 模糊匹配接受上限（且要求唯一最优）。
pub const ACCEPT_MAX: f64 = 1.25;

/// 计算轨迹与 trace 点序列的匹配分（越小越像，归一化后）。
/// 返回 `None` 表示结构不符（段数不同 / 非法编码）。
///
/// 旧格式 P 是**归一化**坐标（格数是相对量），因此先把实际轨迹按
/// 总长度缩放到与期望一致，再做逐段比较；方向相反按段重罚。
pub fn match_polyline(points: &[i32], segments: &[Segment]) -> Option<f64> {
    if points.len() < 4 || !points.len().is_multiple_of(2) {
        return None;
    }
    let expected_len = points.len() / 2 - 1;
    if expected_len != segments.len() || segments.is_empty() {
        return None;
    }
    // 期望位移段（差分，格）
    let mut expected = Vec::with_capacity(expected_len);
    for i in 0..expected_len {
        let ex = (points[(i + 1) * 2] - points[i * 2]) / 50;
        let ey = (points[(i + 1) * 2 + 1] - points[i * 2 + 1]) / 50;
        expected.push((ex as f64, ey as f64));
    }
    // 按段主轴长度（L∞）归一化：既能容忍划得长/短，又能区分 ← 与 ↙
    let linf = |x: f64, y: f64| x.abs().max(y.abs());
    let exp_total: f64 = expected.iter().map(|(x, y)| linf(*x, *y)).sum();
    let act_total: f64 = segments.iter().map(|s| linf(s.x as f64, s.y as f64)).sum();
    if exp_total <= 0.0 || act_total <= 0.0 {
        return None;
    }
    let scale = exp_total / act_total;
    let mut score = 0.0;
    for (i, (ex, ey)) in expected.iter().enumerate() {
        let seg = segments[i];
        let (sx, sy) = (seg.x as f64 * scale, seg.y as f64 * scale);
        score += (ex - sx).abs() + (ey - sy).abs();
        // 方向相反：重罚（除非实际分量几乎为零）
        if *ex != 0.0 && sx.signum() != *ex && sx.abs() > 0.35 {
            score += 1.5;
        }
        if *ey != 0.0 && sy.signum() != *ey && sy.abs() > 0.35 {
            score += 1.5;
        }
    }
    Some(score)
}

/// 从多个候选中选最佳匹配：先找精确匹配（0 分），否则取唯一的最小分（≤ 1）。
pub fn best_score(candidates: &[(&[i32], &[Segment])]) -> Option<(usize, f64)> {
    let mut best: Option<(usize, f64)> = None;
    let mut ties = false;
    for (idx, (points, segs)) in candidates.iter().enumerate() {
        let Some(score) = match_polyline(points, segs) else { continue };
        match best {
            None => best = Some((idx, score)),
            Some((_, bs)) if score < bs - f64::EPSILON => {
                best = Some((idx, score));
                ties = false;
            }
            Some((_, bs)) if (score - bs).abs() <= f64::EPSILON => ties = true,
            _ => {}
        }
    }
    match best {
        Some((idx, s)) if s < EXACT_EPS => Some((idx, s)),
        Some((idx, s)) if s <= ACCEPT_MAX && !ties => Some((idx, s)),
        _ => None,
    }
}

/// 录制：把一段原始轨迹（屏幕/画布像素点）转为 trace 点编码。
///
/// 步骤：按 `grid` 量化为段 → 每段吸附到 8 方向（次轴 < 主轴×0.414 视为纯轴向）→
/// 去噪（[`denoise`]）→ 合并同向段 → 每段记为一格（50）累加 → 平移使最小坐标为 0。
/// 空轨迹返回空数组。
pub fn record_points(raw: &[(f64, f64)], grid: f64) -> Vec<i32> {
    let Some(&(x0, y0)) = raw.first() else { return Vec::new() };
    let mut t = PathTracker::new(grid);
    t.start(x0, y0);
    for &(x, y) in &raw[1..] {
        t.add_point(x, y);
    }
    let snap = |s: &Segment| -> (i32, i32) {
        let (ax, ay) = (s.x.abs() as f64, s.y.abs() as f64);
        let tan = std::f64::consts::FRAC_PI_8.tan(); // 22.5°
        if ay < ax * tan {
            (s.x.signum(), 0)
        } else if ax < ay * tan {
            (0, s.y.signum())
        } else {
            (s.x.signum(), s.y.signum())
        }
    };
    let mut dirs: Vec<(i32, i32)> = Vec::new();
    for s in t.strokes() {
        let d = snap(&s);
        if dirs.last() != Some(&d) {
            dirs.push(d);
        }
    }
    if dirs.is_empty() {
        return Vec::new();
    }
    let mut pts = vec![(0i32, 0i32)];
    for (dx, dy) in dirs {
        let (px, py) = *pts.last().unwrap();
        pts.push((px + dx * 50, py + dy * 50));
    }
    let minx = pts.iter().map(|p| p.0).min().unwrap();
    let miny = pts.iter().map(|p| p.1).min().unwrap();
    pts.into_iter().flat_map(|(x, y)| [x - minx, y - miny]).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(from: (f64, f64), to: (f64, f64)) -> Vec<(f64, f64)> {
        (0..=20).map(|i| {
            let k = i as f64 / 20.0;
            (from.0 + (to.0 - from.0) * k, from.1 + (to.1 - from.1) * k)
        }).collect()
    }

    #[test]
    fn arrows_text() {
        let s = |x, y| Segment { x, y };
        assert_eq!(arrows(&[s(5, 0), s(1, 6), s(4, 0)]), "→ ↓ →");
        assert_eq!(arrows(&[s(-3, -3), s(0, -2)]), "↖ ↑");
        assert_eq!(arrows(&[]), "");
    }

    #[test]
    fn denoise_corner_and_hooks() {
        let s = |x, y| Segment { x, y };
        assert_eq!(denoise(&[s(-6, 0), s(-1, -1), s(0, -6), s(6, 0)]), vec![s(-6, 0), s(0, -6), s(6, 0)]);
        assert_eq!(denoise(&[s(1, 0), s(0, -5), s(-1, 0)]), vec![s(0, -5)]);
        // 真实的短段（不是斜向过渡）保留
        assert_eq!(denoise(&[s(0, 3), s(1, 0), s(0, 3)]).len(), 3);
        // 短手势不受首尾规则影响
        assert_eq!(denoise(&[s(0, -1), s(1, 0)]), vec![s(0, -1), s(1, 0)]);
    }

    #[test]
    fn record_square_and_copy() {
        // 截图：↑ → ↓ ←
        let mut raw = line((0.0, 300.0), (0.0, 0.0));
        raw.extend(line((0.0, 0.0), (300.0, 0.0)));
        raw.extend(line((300.0, 0.0), (300.0, 300.0)));
        raw.extend(line((300.0, 300.0), (0.0, 300.0)));
        assert_eq!(record_points(&raw, 30.0), vec![0, 50, 0, 0, 50, 0, 50, 50, 0, 50]);
        // 复制：↑（略带偏斜仍吸附为纯轴向）
        assert_eq!(record_points(&line((0.0, 200.0), (25.0, 0.0)), 30.0), vec![0, 50, 0, 0]);
        // ↙ 对角
        assert_eq!(record_points(&line((200.0, 0.0), (0.0, 200.0)), 30.0), vec![50, 0, 0, 50]);
        assert!(record_points(&[(0.0, 0.0), (5.0, 5.0)], 30.0).is_empty());
    }

    #[test]
    fn recorded_points_match_their_own_stroke() {
        let mut raw = line((0.0, 0.0), (-250.0, 0.0));
        raw.extend(line((-250.0, 0.0), (-250.0, -250.0)));
        raw.extend(line((-250.0, -250.0), (0.0, -250.0)));
        let pts = record_points(&raw, 30.0);
        assert_eq!(pts, vec![50, 50, 0, 50, 0, 0, 50, 0]); // 刷新 ←↑→，与旧格式编码一致
        let mut t = PathTracker::new(30.0);
        t.start(0.0, 0.0);
        raw.iter().for_each(|&(x, y)| t.add_point(x, y));
        assert!(match_polyline(&pts, &t.strokes()).unwrap_or_else(|| panic!("{:?}", t.strokes())) < EXACT_EPS);
    }

    fn tracker(points: &[(f64, f64)]) -> Vec<Segment> {
        let mut t = PathTracker::new(30.0);
        let (x0, y0) = points[0];
        t.start(x0, y0);
        for &(x, y) in &points[1..] {
            t.add_point(x, y);
        }
        t.segments().to_vec()
    }

    #[test]
    fn quantizes_axis_and_diagonal() {
        // 上（y 负）
        assert_eq!(tracker(&[(0.0, 0.0), (0.0, -100.0)]), vec![Segment { x: 0, y: -3 }]);
        // 右下对角
        assert_eq!(tracker(&[(0.0, 0.0), (70.0, 70.0)]), vec![Segment { x: 2, y: 2 }]);
        // 左
        assert_eq!(tracker(&[(0.0, 0.0), (-90.0, 0.0)]), vec![Segment { x: -3, y: 0 }]);
    }

    #[test]
    fn ignores_sub_grid_moves() {
        assert!(tracker(&[(0.0, 0.0), (0.0, -25.0)]).is_empty());
    }

    #[test]
    fn merges_consecutive_same_direction() {
        let pts: Vec<(f64, f64)> = (0..=10).map(|i| (0.0, -(i as f64) * 20.0)).collect();
        assert_eq!(tracker(&pts), vec![Segment { x: 0, y: -5 }]);
    }

    #[test]
    fn multi_segment_path() {
        let pts = [(0.0, 0.0), (0.0, -100.0), (0.0, -200.0), (-100.0, -200.0)];
        // 两段同向（上）会被合并为一段
        assert_eq!(tracker(&pts), vec![Segment { x: 0, y: -6 }, Segment { x: -3, y: 0 }]);
    }

    #[test]
    fn official_copy_matches_upward_stroke() {
        // 旧格式 Copy: P=[0,50,0,0] → 期望位移 (0,-1)（格，归一化）
        // 自然划 5 格长也应命中（总长度归一化）
        let long = [Segment { x: 0, y: -5 }];
        assert!(match_polyline(&[0, 50, 0, 0], &long).unwrap() < EXACT_EPS);
        assert!(match_polyline(&[0, 50, 0, 0], &[Segment { x: 0, y: -1 }]).unwrap() < EXACT_EPS);
    }

    #[test]
    fn official_screenshot_matches_box_path() {
        // 截图：↑ → ↓ ←（画方框）；P 为旧格式绝对坐标 (0,50)(0,0)(50,0)(50,50)(0,50)
        let points = [0, 50, 0, 0, 50, 0, 50, 50, 0, 50];
        let segs = [
            Segment { x: 0, y: -1 },
            Segment { x: 1, y: 0 },
            Segment { x: 0, y: 1 },
            Segment { x: -1, y: 0 },
        ];
        assert!(match_polyline(&points, &segs).unwrap() < EXACT_EPS);
    }

    #[test]
    fn official_refresh_and_esc() {
        // 刷新 ← ↑ →（旧格式 P 起点 50,50）
        let segs = [Segment { x: -1, y: 0 }, Segment { x: 0, y: -1 }, Segment { x: 1, y: 0 }];
        assert!(match_polyline(&[50, 50, 0, 50, 0, 0, 50, 0], &segs).unwrap() < EXACT_EPS);
        // ESC → ↑ ←
        let segs2 = [Segment { x: 1, y: 0 }, Segment { x: 0, y: -1 }, Segment { x: -1, y: 0 }];
        assert!(match_polyline(&[0, 50, 50, 50, 50, 0, 0, 0], &segs2).unwrap() < EXACT_EPS);
    }

    #[test]
    fn official_minimize_diagonal() {
        // 最小化 ↙ = (-1,+1)；划长对角线也应命中（归一化）
        let long = [Segment { x: -3, y: 3 }];
        assert!(match_polyline(&[50, 0, 0, 50], &long).unwrap() < EXACT_EPS);
        // 纯左划 (−1,0) 与 ↙ 有分差（1 分）——由 best_score 的"唯一最优"规则控制误匹配
        let s = match_polyline(&[50, 0, 0, 50], &[Segment { x: -1, y: 0 }]).unwrap();
        assert!((EXACT_EPS..=ACCEPT_MAX).contains(&s), "实际 {s}");
    }

    #[test]
    fn wrong_direction_does_not_match() {
        let s = match_polyline(&[0, 50, 0, 0], &[Segment { x: 0, y: 1 }]).unwrap();
        assert!(s > ACCEPT_MAX, "方向相反应被重罚, 实际 {s}");
    }

    #[test]
    fn segment_count_must_match() {
        assert!(match_polyline(&[0, 50, 0, 0], &[Segment { x: 0, y: -1 }, Segment { x: 1, y: 0 }]).is_none());
    }

    #[test]
    fn best_score_prefers_exact_then_unique() {
        let segs = [Segment { x: -1, y: 0 }];
        let cands: Vec<(&[i32], &[Segment])> = vec![
            (&[0i32, 50, 0, 0], &segs),          // 2 分
            (&[50, 0, 0, 50], &segs),            // 1 分（↙）
        ];
        assert_eq!(best_score(&cands).map(|(i, _)| i), Some(1));
        // 出现并列 → 放弃模糊匹配（↙ [50,0,0,50] 与 ↖ [50,50,0,0] 对左划段同为 1 分）
        let cands2: Vec<(&[i32], &[Segment])> = vec![
            (&[50, 0, 0, 50], &segs),
            (&[50, 50, 0, 0], &segs),
        ];
        assert!(best_score(&cands2).is_none());
    }
}
