//! 手势库：可被多个绑定共享的手势定义。
//!
//! 不变式（I1）：绑定的 `gesture = Some(id)` 时，库中存在 `id`，且绑定的 `pattern` 与库中完全相同。
//! 引擎与旧版本只读绑定自己的 `pattern`，手势库对它们只是可选信息。
//!
//! - 加载：[`check_gesture_refs`] 把不一致的引用改为内联，保留绑定自己的 pattern，从不改变任何手势的行为。
//! - 内置配置：[`adopt_exact`] 从「所有程序」建库，只合并完全相同的手势。
//! - 整理导入：[`analyze`] 给出"相同 / 方向相同的折线 / 相似形状"的分组建议，由设置界面展示并按用户选择应用。
//! - 保存：[`validate`] 拒绝违反 I1 或含未录完手势的手势库。

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::schema::{Binding, Config, Gesture, ScopeKind, Step};
use crate::shape;

/// 每个形状手势最多保存的样本数（与设置界面的录制上限一致）。
pub const MAX_SAMPLES: usize = 5;
/// 相似形状组默认改用其他成员样本所需的一致度领先幅度。
const CHOICE_LEAD: f64 = 0.03;

fn valid_sample(sample: &[i32]) -> bool {
    shape::decode(sample).is_some_and(|p| shape::similarity(&p, &p) > 0.0)
}

/// 笔画是否都已录完：折线至少两个点、形状至少一个有效样本。
pub fn strokes_complete(pattern: &[Step]) -> bool {
    pattern.iter().all(|step| match step {
        Step::Trace { points } => points.len() >= 4,
        Step::Shape { templates } => templates.iter().any(|t| valid_sample(t)),
        _ => true,
    })
}

/// 可以作为手势库条目的 pattern：非空且笔画完整（单击手势也算）。
pub fn library_ready(pattern: &[Step]) -> bool {
    !pattern.is_empty() && strokes_complete(pattern)
}

/// 绑定能否被收入手势库：草稿不建库条目（但可以引用已有的库手势）。
pub fn can_adopt(binding: &Binding) -> bool {
    !binding.draft && library_ready(&binding.pattern)
}

/// 引用某个手势的绑定位置（作用域下标, 绑定下标）。
pub fn uses<'a>(cfg: &'a Config, id: &'a str) -> impl Iterator<Item = (usize, usize)> + 'a {
    cfg.scopes.iter().enumerate().flat_map(move |(si, scope)| {
        scope.bindings.iter().enumerate().filter(move |(_, b)| b.gesture.as_deref() == Some(id)).map(move |(bi, _)| (si, bi))
    })
}

/// 下一个 `g<n>` 形式的 id（取现有最大序号 + 1，确定性）。
pub fn next_gesture_id(cfg: &Config) -> String {
    let max = cfg.gestures.iter().filter_map(|g| g.id.strip_prefix('g')?.parse::<u32>().ok()).max().unwrap_or(0);
    format!("g{}", max + 1)
}

/// 加载时维护 I1：丢弃空的、未录完的和重复 id 的库条目；引用缺失或与库不一致的绑定改为内联
/// （保留绑定自己的 pattern）。返回被解除引用的绑定数。幂等。
pub fn check_gesture_refs(cfg: &mut Config) -> usize {
    let mut seen = HashSet::new();
    cfg.gestures.retain(|g| !g.id.is_empty() && library_ready(&g.pattern) && seen.insert(g.id.clone()));
    let gestures = &cfg.gestures;
    let mut detached = 0;
    for binding in cfg.scopes.iter_mut().flat_map(|s| s.bindings.iter_mut()) {
        let Some(id) = &binding.gesture else { continue };
        if !gestures.iter().any(|g| &g.id == id && g.pattern == binding.pattern) {
            binding.gesture = None;
            detached += 1;
        }
    }
    detached
}

/// 保存前的手势库校验失败原因（携带相关手势 id）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LibraryError {
    /// 手势库中有空的或未录完的手势。
    Incomplete(String),
    /// 手势库中 id 重复。
    DuplicateId(String),
    /// 绑定引用了不存在的手势，或 pattern 与库中不一致。
    BrokenRef(String),
}

/// 保存前校验：库条目完整、id 唯一、所有引用满足 I1。
pub fn validate(cfg: &Config) -> Result<(), LibraryError> {
    let mut seen = HashSet::new();
    for g in &cfg.gestures {
        if !library_ready(&g.pattern) {
            return Err(LibraryError::Incomplete(g.id.clone()));
        }
        if !seen.insert(g.id.as_str()) {
            return Err(LibraryError::DuplicateId(g.id.clone()));
        }
    }
    for binding in cfg.all_bindings() {
        let Some(id) = &binding.gesture else { continue };
        if !cfg.gestures.iter().any(|g| &g.id == id && g.pattern == binding.pattern) {
            return Err(LibraryError::BrokenRef(id.clone()));
        }
    }
    Ok(())
}

/// 建库 / 整理导入的来源。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdoptSource {
    /// 只从「所有程序」收入；其他程序中相同（或经确认相似）的手势关联过来。
    Global,
    /// 所有程序的手势都收入。
    All,
}

impl AdoptSource {
    fn seeds(self, kind: ScopeKind) -> bool {
        self == AdoptSource::All || kind == ScopeKind::Global
    }
}

/// 遍历顺序：「所有程序」在前，其余按配置顺序。分组结果只取决于这一顺序。
fn scope_order(cfg: &Config) -> Vec<usize> {
    let global = (0..cfg.scopes.len()).filter(|&i| cfg.scopes[i].kind == ScopeKind::Global);
    let rest = (0..cfg.scopes.len()).filter(|&i| cfg.scopes[i].kind != ScopeKind::Global);
    global.chain(rest).collect()
}

/// 只按完全相同的 pattern 建库并关联（内置配置使用，无需用户确认）。返回新增的手势数。
pub fn adopt_exact(cfg: &mut Config, source: AdoptSource) -> usize {
    let before = cfg.gestures.len();
    for si in scope_order(cfg) {
        if !source.seeds(cfg.scopes[si].kind) {
            continue;
        }
        for bi in 0..cfg.scopes[si].bindings.len() {
            let binding = &cfg.scopes[si].bindings[bi];
            if binding.gesture.is_some() || !can_adopt(binding) || cfg.gestures.iter().any(|g| g.pattern == binding.pattern) {
                continue;
            }
            let gesture = Gesture { id: next_gesture_id(cfg), name: String::new(), pattern: binding.pattern.clone() };
            cfg.gestures.push(gesture);
        }
    }
    let gestures = &cfg.gestures;
    for binding in cfg.scopes.iter_mut().flat_map(|s| s.bindings.iter_mut()) {
        if binding.gesture.is_none() && can_adopt(binding) {
            if let Some(g) = gestures.iter().find(|g| g.pattern == binding.pattern) {
                binding.gesture = Some(g.id.clone());
            }
        }
    }
    cfg.gestures.len() - before
}

// ================================================================ 整理导入分析

/// 成员与组代表的关系。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Relation {
    /// 自成一组的第一个成员（组代表）。
    Seed,
    /// 已经引用该库手势。
    Linked,
    /// pattern 完全相同。
    Identical,
    /// 方向序列相同的折线（只差段长）。
    Equivalent,
    /// 交叉相似度达到接受阈值的形状。
    Similar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupKind {
    Identical,
    Equivalent,
    Similar,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportMember {
    pub scope: usize,
    pub binding: usize,
    pub relation: Relation,
    /// 与组代表的相似度（相同 / 等效为 1）。
    pub score: f64,
    /// 是否明显相似（≥ 阈值 + MARGIN）；全部明显相似的组默认勾选合并。
    pub strong: bool,
    /// 形状样本一致度（样本少于两个时为空）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consistency: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MergedSamples {
    pub templates: Vec<Vec<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consistency: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportGroup {
    /// 已在手势库中的手势 id（再次整理时作为现成的组代表）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub existing: Option<String>,
    pub kind: GroupKind,
    /// 组代表的 pattern：已有库手势或第一个成员。
    pub pattern: Vec<Step>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consistency: Option<f64>,
    /// 组内最低的相似度。
    pub score: f64,
    /// 建议是否合并不完全相同的成员。
    pub checked: bool,
    /// 建议以哪个成员的 pattern 为准；`None` = 组代表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub choice: Option<usize>,
    pub members: Vec<ImportMember>,
    /// 相似形状组：合并全部样本（最多 [`MAX_SAMPLES`] 个）的结果。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merged: Option<MergedSamples>,
}

/// 同一程序中两个相似但不同的手势：视为用户有意区分，不提议合并。
#[derive(Debug, Clone, Serialize)]
pub struct ImportNote {
    pub scope: usize,
    pub first: usize,
    pub second: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportPlan {
    pub source: AdoptSource,
    pub groups: Vec<ImportGroup>,
    pub notes: Vec<ImportNote>,
}

/// 折线的方向序列（与设置界面的箭头描述同口径：逐段取坐标差的符号）。
fn directions(points: &[i32]) -> Vec<(i32, i32)> {
    points.chunks_exact(2).collect::<Vec<_>>().windows(2).map(|w| ((w[1][0] - w[0][0]).signum(), (w[1][1] - w[0][1]).signum())).collect()
}

/// 两个 pattern 的关系：非笔画部分必须完全相同、笔画类型相同，只比较那一步笔画。
fn relate(a: &[Step], b: &[Step]) -> Option<(Relation, f64, bool)> {
    if a.len() != b.len() {
        return None;
    }
    let mut stroke = None;
    for (x, y) in a.iter().zip(b) {
        match (x, y) {
            (Step::Trace { .. }, Step::Trace { .. }) | (Step::Shape { .. }, Step::Shape { .. }) => {
                if stroke.replace((x, y)).is_some() {
                    return None;
                }
            }
            _ if x == y => {}
            _ => return None,
        }
    }
    match stroke? {
        (Step::Trace { points: p }, Step::Trace { points: q }) => {
            let d = directions(p);
            (!d.is_empty() && d == directions(q)).then_some((Relation::Equivalent, 1.0, true))
        }
        (Step::Shape { templates: p }, Step::Shape { templates: q }) => {
            let score = shape::cross_similarity(p, q);
            let threshold = shape::pair_threshold(p, q);
            (score >= threshold).then_some((Relation::Similar, score, score >= threshold + shape::MARGIN))
        }
        _ => None,
    }
}

fn shape_templates(pattern: &[Step]) -> Option<&[Vec<i32>]> {
    pattern.iter().find_map(|s| match s {
        Step::Shape { templates } => Some(templates.as_slice()),
        _ => None,
    })
}

fn member(scope: usize, binding: usize, relation: Relation, score: f64, strong: bool) -> ImportMember {
    ImportMember { scope, binding, relation, score, strong, consistency: None }
}

fn new_group(existing: Option<String>, pattern: Vec<Step>, members: Vec<ImportMember>) -> ImportGroup {
    ImportGroup { existing, kind: GroupKind::Identical, pattern, consistency: None, score: 1.0, checked: true, choice: None, members, merged: None }
}

/// 合并样本：去重后若超过上限，保留与其余样本平均相似度最高的几个，保持原有顺序。
pub fn merge_samples<'a>(sets: impl IntoIterator<Item = &'a [Vec<i32>]>) -> Vec<Vec<i32>> {
    let mut all: Vec<Vec<i32>> = Vec::new();
    for set in sets {
        for t in set {
            if valid_sample(t) && !all.contains(t) {
                all.push(t.clone());
            }
        }
    }
    if all.len() <= MAX_SAMPLES {
        return all;
    }
    let paths: Vec<_> = all.iter().map(|t| shape::decode(t).expect("已校验")).collect(); // i18n:allow 内部不变式
    let mut ranked: Vec<(usize, f64)> = (0..paths.len())
        .map(|i| (i, (0..paths.len()).filter(|&j| j != i).map(|j| shape::similarity(&paths[i], &paths[j])).sum::<f64>()))
        .collect();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    let mut keep: Vec<usize> = ranked.into_iter().take(MAX_SAMPLES).map(|(i, _)| i).collect();
    keep.sort_unstable();
    keep.into_iter().map(|i| all[i].clone()).collect()
}

fn finish(cfg: &Config, group: &mut ImportGroup) {
    let rels: Vec<_> = group.members.iter().map(|m| m.relation).collect();
    group.kind = if rels.contains(&Relation::Similar) {
        GroupKind::Similar
    } else if rels.contains(&Relation::Equivalent) {
        GroupKind::Equivalent
    } else {
        GroupKind::Identical
    };
    group.score = group.members.iter().filter(|m| m.relation == Relation::Similar).map(|m| m.score).fold(1.0, f64::min);
    group.checked = match group.kind {
        GroupKind::Similar => group.members.iter().filter(|m| m.relation == Relation::Similar).all(|m| m.strong),
        _ => true,
    };
    if group.kind != GroupKind::Similar {
        return;
    }
    let pattern_of = |m: &ImportMember| &cfg.scopes[m.scope].bindings[m.binding].pattern;
    group.consistency = shape_templates(&group.pattern).and_then(shape::consistency);
    for m in &mut group.members {
        m.consistency = shape_templates(&cfg.scopes[m.scope].bindings[m.binding].pattern).and_then(shape::consistency);
    }
    // 默认以组代表为准；其他成员的一致度明显更高时改选它。
    let mut best = group.consistency.unwrap_or(0.0) + CHOICE_LEAD;
    for (i, m) in group.members.iter().enumerate() {
        if m.relation != Relation::Linked && pattern_of(m) != &group.pattern && m.consistency.unwrap_or(0.0) > best {
            best = m.consistency.unwrap_or(0.0);
            group.choice = Some(i);
        }
    }
    let sets = std::iter::once(&group.pattern).chain(group.members.iter().map(pattern_of)).filter_map(|p| shape_templates(p));
    let templates = merge_samples(sets);
    let consistency = shape::consistency(&templates);
    group.merged = Some(MergedSamples { templates, consistency });
}

/// 整理导入分析（不修改配置）：
/// 1. 已在库中的手势作为现成的组代表；
/// 2. 按「所有程序」优先的顺序遍历可收入的绑定：先找完全相同的组，再找方向相同的折线或相似的形状中得分最高的组；
/// 3. 同一程序里已有另一个相似但不同的成员时不加入该组，记一条说明；
/// 4. 找不到组且来源允许时自成一组。
pub fn analyze(cfg: &Config, source: AdoptSource) -> ImportPlan {
    let mut groups: Vec<ImportGroup> = cfg
        .gestures
        .iter()
        .map(|g| new_group(Some(g.id.clone()), g.pattern.clone(), uses(cfg, &g.id).map(|(si, bi)| member(si, bi, Relation::Linked, 1.0, true)).collect()))
        .collect();
    let mut notes = Vec::new();
    for si in scope_order(cfg) {
        let scope = &cfg.scopes[si];
        for (bi, binding) in scope.bindings.iter().enumerate() {
            if binding.gesture.is_some() || !can_adopt(binding) {
                continue;
            }
            if let Some(group) = groups.iter_mut().find(|g| g.pattern == binding.pattern) {
                group.members.push(member(si, bi, Relation::Identical, 1.0, true));
                continue;
            }
            let mut best: Option<(usize, Relation, f64, bool)> = None;
            let mut blocked: Option<usize> = None;
            for (gi, group) in groups.iter().enumerate() {
                let Some((relation, score, strong)) = relate(&group.pattern, &binding.pattern) else { continue };
                let same_scope = group.members.iter().find(|m| m.scope == si && cfg.scopes[si].bindings[m.binding].pattern != binding.pattern);
                if let Some(other) = same_scope {
                    blocked.get_or_insert(other.binding);
                    continue;
                }
                if best.is_none_or(|(_, _, s, _)| score > s) {
                    best = Some((gi, relation, score, strong));
                }
            }
            if let Some((gi, relation, score, strong)) = best {
                groups[gi].members.push(member(si, bi, relation, score, strong));
                continue;
            }
            if let Some(first) = blocked {
                notes.push(ImportNote { scope: si, first, second: bi });
            }
            if source.seeds(scope.kind) {
                groups.push(new_group(None, binding.pattern.clone(), vec![member(si, bi, Relation::Seed, 1.0, true)]));
            }
        }
    }
    groups.retain(|g| g.existing.is_none() || g.members.iter().any(|m| m.relation != Relation::Linked));
    for group in &mut groups {
        finish(cfg, group);
    }
    ImportPlan { source, groups, notes }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directions_follow_segment_signs() {
        assert_eq!(directions(&[0, 0, 0, 50, 50, 50]), vec![(0, 1), (1, 0)]);
        assert_eq!(directions(&[0, 0, 0, 100, 50, 100]), directions(&[0, 0, 0, 50, 50, 50]));
        assert!(directions(&[0, 0]).is_empty());
    }

    #[test]
    fn next_id_skips_used_numbers_and_ignores_other_ids() {
        let mut cfg = Config::default();
        assert_eq!(next_gesture_id(&cfg), "g1");
        cfg.gestures.push(Gesture { id: "g7".into(), name: String::new(), pattern: vec![Step::Edge { edge: 1 }] });
        cfg.gestures.push(Gesture { id: "u1x9".into(), name: String::new(), pattern: vec![Step::Edge { edge: 2 }] });
        assert_eq!(next_gesture_id(&cfg), "g8");
    }
}
