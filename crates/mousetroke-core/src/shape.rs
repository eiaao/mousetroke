//! 曲线手势：等距重采样与有界旋转的 Protractor 模板匹配。
//!
//! 保留起点、绘制方向和长宽比；只消除平移、等比缩放及少量旋转。
//! 模板保存归一化坐标的千分位整数，匹配时直接读取，不在样本循环内分配内存。

use std::f64::consts::FRAC_PI_2;

/// 每条笔画重采样后的点数。
pub const SAMPLE_POINTS: usize = 64;
/// 最低相似度。0.81 对应向量夹角约 17.1°，允许手抖与小钩子，同时拒绝普通折线。
/// 提高此值可减少误触发，但需要用户录制更接近实际画法的样本。
pub const ACCEPT: f64 = 0.81;
/// 最佳绑定必须领先其他绑定的分数。0.04 用于拒绝近似形状之间的歧义；
/// 同一绑定的多个样本不互相竞争。减小此值会增加相似绑定之间的误识别。
pub const MARGIN: f64 = 0.04;
/// 允许补偿的最大旋转（弧度），不做起始角度对齐。
pub const ROTATION_LIMIT: f64 = 20.0 * std::f64::consts::PI / 180.0;

/// 原始像素路径 → 64 个等距点 → 质心归零 → 按包围盒长边等比缩放。
/// 少于两点、总长不足一像素、非有限坐标或退化路径返回 `None`。
pub fn normalize(raw: &[(f64, f64)]) -> Option<Vec<(f64, f64)>> {
    if raw.len() < 2 || raw.iter().any(|(x, y)| !x.is_finite() || !y.is_finite()) {
        return None;
    }
    let length: f64 = raw.windows(2).map(|p| (p[1].0 - p[0].0).hypot(p[1].1 - p[0].1)).sum();
    if !length.is_finite() || length < 1.0 {
        return None;
    }
    let origin = raw[0];
    let mut sampled = Vec::with_capacity(SAMPLE_POINTS);
    sampled.push((0.0, 0.0));
    let mut traversed = 0.0;
    for pair in raw.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let segment = (b.0 - a.0).hypot(b.1 - a.1);
        if segment == 0.0 {
            continue;
        }
        while sampled.len() < SAMPLE_POINTS - 1 {
            let target = length * (sampled.len() as f64 / (SAMPLE_POINTS - 1) as f64);
            if target > traversed + segment {
                break;
            }
            let t = ((target - traversed) / segment).clamp(0.0, 1.0);
            sampled.push((a.0 - origin.0 + t * (b.0 - a.0), a.1 - origin.1 + t * (b.1 - a.1)));
        }
        traversed += segment;
    }
    let last = raw.last()?;
    sampled.push((last.0 - origin.0, last.1 - origin.1));
    if sampled.len() != SAMPLE_POINTS {
        return None;
    }
    let (mut cx, mut cy) = (0.0, 0.0);
    let (mut min_x, mut max_x, mut min_y, mut max_y) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    for &(x, y) in &sampled {
        cx += x / SAMPLE_POINTS as f64;
        cy += y / SAMPLE_POINTS as f64;
        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_y = min_y.min(y);
        max_y = max_y.max(y);
    }
    let size = (max_x - min_x).max(max_y - min_y);
    if !size.is_finite() || size <= f64::EPSILON {
        return None;
    }
    for (x, y) in &mut sampled {
        *x = (*x - cx) / size;
        *y = (*y - cy) / size;
    }
    Some(sampled)
}

/// 将已归一化的模板编码为交替排列的 x/y 千分位整数。
pub fn encode(points: &[(f64, f64)]) -> Vec<i32> {
    points.iter().flat_map(|&(x, y)| [(x * 1000.0).round() as i32, (y * 1000.0).round() as i32]).collect()
}

fn valid_encoding(encoded: &[i32]) -> bool {
    encoded.len() == SAMPLE_POINTS * 2 && encoded.iter().all(|v| (-1000..=1000).contains(v))
}

/// 解码一个归一化模板；错误长度或超出归一化值域的坐标返回 `None`。
pub fn decode(encoded: &[i32]) -> Option<Vec<(f64, f64)>> {
    valid_encoding(encoded).then(|| encoded.chunks_exact(2).map(|p| (p[0] as f64 / 1000.0, p[1] as f64 / 1000.0)).collect())
}

fn energy(points: &[(f64, f64)]) -> Option<f64> {
    if points.len() != SAMPLE_POINTS || points.iter().any(|(x, y)| !x.is_finite() || !y.is_finite()) {
        return None;
    }
    let value: f64 = points.iter().map(|(x, y)| x * x + y * y).sum();
    (value.is_finite() && value > f64::EPSILON).then_some(value)
}

fn score(dot: f64, cross: f64, a_energy: f64, b_energy: f64) -> f64 {
    if !dot.is_finite() || !cross.is_finite() || !b_energy.is_finite() || b_energy <= f64::EPSILON {
        return 0.0;
    }
    // dot*cos(θ) + cross*sin(θ) 的极大值；atan2 保留象限，随后限制补偿角。
    let angle = cross.atan2(dot).clamp(-ROTATION_LIMIT, ROTATION_LIMIT);
    let (sin, cos) = angle.sin_cos();
    let cosine = ((dot * cos + cross * sin) / a_energy.sqrt() / b_energy.sqrt()).clamp(-1.0, 1.0);
    if cosine >= 1.0 - 1e-14 {
        1.0
    } else {
        (1.0 - cosine.acos() / FRAC_PI_2).clamp(0.0, 1.0)
    }
}

/// 两个归一化路径的相似度（0..=1），1 表示完全一致；只补偿 ±20° 的旋转。
/// 非有限值、退化向量或点数错误记为 0 分。
pub fn similarity(a: &[(f64, f64)], b: &[(f64, f64)]) -> f64 {
    let (Some(a_energy), Some(b_energy)) = (energy(a), energy(b)) else { return 0.0 };
    let (mut dot, mut cross) = (0.0, 0.0);
    for (&(ax, ay), &(bx, by)) in a.iter().zip(b) {
        dot += ax * bx + ay * by;
        cross += ax * by - ay * bx;
    }
    score(dot, cross, a_energy, b_energy)
}

fn encoded_similarity(path: &[(f64, f64)], path_energy: f64, template: &[i32]) -> f64 {
    if !valid_encoding(template) {
        return 0.0;
    }
    let (mut dot, mut cross, mut template_energy) = (0.0, 0.0, 0.0);
    for (&(ax, ay), p) in path.iter().zip(template.chunks_exact(2)) {
        // 向量夹角不受统一倍数影响，因此无需解码或除以 1000。
        let (bx, by) = (p[0] as f64, p[1] as f64);
        dot += ax * bx + ay * by;
        cross += ax * by - ay * bx;
        template_energy += bx * bx + by * by;
    }
    score(dot, cross, path_energy, template_energy)
}

/// 每个绑定取其所有样本的最高分；达标且领先其他 id 至少 [`MARGIN`] 才接受。
/// id 可用绑定索引或稳定标识；同一 id 的样本即使分列多项也不参与歧义竞争。
/// 时间复杂度 O(模板数 × 64)，样本循环不分配内存。
pub fn best_match<Id: Copy + Eq>(path_norm: &[(f64, f64)], candidates: &[(Id, &[Vec<i32>])]) -> Option<(Id, f64)> {
    let path_energy = energy(path_norm)?;
    let mut best = None;
    let mut runner_up = 0.0f64;
    for &(id, samples) in candidates {
        let candidate_score = samples.iter().map(|s| encoded_similarity(path_norm, path_energy, s)).fold(0.0f64, f64::max);
        match best {
            None => best = Some((id, candidate_score)),
            Some((best_id, best_score)) if id == best_id => best = Some((id, best_score.max(candidate_score))),
            Some((_, best_score)) if candidate_score > best_score => {
                runner_up = best_score;
                best = Some((id, candidate_score));
            }
            _ => runner_up = runner_up.max(candidate_score),
        }
    }
    best.filter(|&(_, value)| value >= ACCEPT && value - runner_up >= MARGIN)
}
