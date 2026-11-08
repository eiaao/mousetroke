// 手势库的纯函数（与 core::library 对应）。
//
// 不变式 I1：绑定的 gesture 指向库中存在的手势，且 pattern 与库中完全相同。
// 所有修改 pattern 的代码都必须经过 setBindingPattern / setGesturePattern，
// store 在每次编辑后用 checkRefs 检查（开发构建直接报错，正式构建解除引用兜底）。

import type { Binding, Config, Gesture, ImportGroup, ImportPlan, Step } from "./types";
import { conflicts, decompose, uid } from "./gesture";

export interface BindingRef {
  scope: number;
  binding: number;
}

/** JSON 值的结构相等（不受对象键顺序影响）。 */
export function deepEqual(a: unknown, b: unknown): boolean {
  if (a === b) return true;
  if (typeof a !== "object" || typeof b !== "object" || a === null || b === null) return false;
  if (Array.isArray(a) !== Array.isArray(b)) return false;
  if (Array.isArray(a)) {
    const bb = b as unknown[];
    return a.length === bb.length && a.every((v, i) => deepEqual(v, bb[i]));
  }
  const ka = Object.keys(a).filter((k) => (a as Record<string, unknown>)[k] !== undefined);
  const kb = Object.keys(b).filter((k) => (b as Record<string, unknown>)[k] !== undefined);
  return ka.length === kb.length && ka.every((k) => deepEqual((a as Record<string, unknown>)[k], (b as Record<string, unknown>)[k]));
}

export const gesturesOf = (c: Config): Gesture[] => c.gestures ?? [];
export const gestureById = (c: Config, id: string | undefined): Gesture | undefined => (id ? gesturesOf(c).find((g) => g.id === id) : undefined);

/** 笔画是否都已录完（折线至少两点、形状至少一个样本）。 */
export function strokesComplete(pattern: Step[]): boolean {
  return pattern.every((s) => (s.type === "trace" ? s.points.length >= 4 : s.type === "shape" ? s.templates.length > 0 : true));
}

/** 可以作为手势库条目：非空且笔画完整（单击手势也算）。 */
export const libraryReady = (pattern: Step[]): boolean => pattern.length > 0 && strokesComplete(pattern);

/** 草稿不建库条目（但可以引用已有的库手势）。 */
export const canAdopt = (b: Binding): boolean => !b.draft && !b.actionUnselected && libraryReady(b.pattern);

/** 手势库页面的分类：与程序列表的「鼠标手势 / 触发角」一致。 */
export const categoryOf = (pattern: Step[]): "gesture" | "edge" => (decompose(pattern).trigger.kind === "edge" ? "edge" : "gesture");

export function gestureUses(c: Config, id: string): BindingRef[] {
  const out: BindingRef[] = [];
  c.scopes.forEach((s, scope) => s.bindings.forEach((b, binding) => b.gesture === id && out.push({ scope, binding })));
  return out;
}

/** 界面新建的手势 id：`u` + 随机串，不与内置的 g<n> 冲突，也不复用已删除的 id。 */
export function newGestureId(c: Config): string {
  const used = new Set(gesturesOf(c).map((g) => g.id));
  let id = "";
  do id = `u${uid()}`;
  while (used.has(id));
  return id;
}

function mapBindings(c: Config, f: (b: Binding, ref: BindingRef) => Binding): Config {
  let changed = false;
  const scopes = c.scopes.map((s, scope) => {
    let local = false;
    const bindings = s.bindings.map((b, binding) => {
      const next = f(b, { scope, binding });
      if (next !== b) local = true;
      return next;
    });
    if (!local) return s;
    changed = true;
    return { ...s, bindings };
  });
  return changed ? { ...c, scopes } : c;
}

function updateAt(c: Config, ref: BindingRef, f: (b: Binding) => Binding): Config {
  return mapBindings(c, (b, r) => (r.scope === ref.scope && r.binding === ref.binding ? f(b) : b));
}

/** 改库中手势的画法，并同步所有引用它的绑定。 */
export function setGesturePattern(c: Config, id: string, pattern: Step[]): Config {
  if (!gestureById(c, id)) return c;
  const withLib = { ...c, gestures: gesturesOf(c).map((g) => (g.id === id ? { ...g, pattern } : g)) };
  return mapBindings(withLib, (b) => (b.gesture === id ? { ...b, pattern } : b));
}

/** 唯一的绑定 pattern 写入口：专用手势只改自己；引用库手势时改库并同步所有引用处。 */
export function setBindingPattern(c: Config, ref: BindingRef, pattern: Step[]): Config {
  const b = c.scopes[ref.scope]?.bindings[ref.binding];
  if (!b) return c;
  if (b.gesture && gestureById(c, b.gesture)) return setGesturePattern(c, b.gesture, pattern);
  return updateAt(c, ref, (x) => ({ ...x, gesture: undefined, pattern }));
}

/** 解除引用：绑定变回本程序专用，pattern 保持不变。 */
export function detach(c: Config, ref: BindingRef): Config {
  return updateAt(c, ref, (b) => (b.gesture ? { ...b, gesture: undefined } : b));
}

/** 让绑定引用库手势（pattern 改为库中的画法）。 */
export function link(c: Config, ref: BindingRef, id: string): Config {
  const g = gestureById(c, id);
  if (!g) return c;
  return updateAt(c, ref, (b) => ({ ...b, gesture: id, pattern: g.pattern }));
}

/** 新建库手势。 */
export function addGesture(c: Config, pattern: Step[], name = ""): { config: Config; id: string } {
  const id = newGestureId(c);
  return { config: { ...c, gestures: [...gesturesOf(c), { id, ...(name ? { name } : {}), pattern }] }, id };
}

export function renameGesture(c: Config, id: string, name: string): Config {
  return { ...c, gestures: gesturesOf(c).map((g) => (g.id === id ? { ...g, name: name || undefined } : g)) };
}

/**
 * 把绑定的手势加入手势库并引用它；库中已有完全相同的手势时直接引用。
 * 草稿或未录完的手势返回 null（草稿不建库条目）。
 */
export function promote(c: Config, ref: BindingRef): { config: Config; id: string } | null {
  const b = c.scopes[ref.scope]?.bindings[ref.binding];
  if (!b) return null;
  if (b.gesture && gestureById(c, b.gesture)) return { config: c, id: b.gesture };
  if (!canAdopt(b)) return null;
  const same = gesturesOf(c).find((g) => deepEqual(g.pattern, b.pattern));
  if (same) return { config: link(c, ref, same.id), id: same.id };
  const added = addGesture(c, b.pattern);
  return { config: link(added.config, ref, added.id), id: added.id };
}

/** 在某个分组末尾追加一条引用库手势的绑定（「在其他程序中使用」）。 */
export function assignGesture(c: Config, scope: number, id: string, binding: Binding): Config {
  const g = gestureById(c, id);
  if (!g || !c.scopes[scope]) return c;
  const b = { ...binding, gesture: id, pattern: g.pattern };
  return { ...c, scopes: c.scopes.map((s, i) => (i === scope ? { ...s, bindings: [...s.bindings, b] } : s)) };
}

/** 删除库手势：`detach` 保留引用它的绑定（改为各自专用），`cascade` 连同绑定一起删除。 */
export function deleteGesture(c: Config, id: string, mode: "detach" | "cascade"): Config {
  const gestures = gesturesOf(c).filter((g) => g.id !== id);
  const scopes = c.scopes.map((s) =>
    s.bindings.some((b) => b.gesture === id)
      ? { ...s, bindings: mode === "cascade" ? s.bindings.filter((b) => b.gesture !== id) : s.bindings.map((b) => (b.gesture === id ? { ...b, gesture: undefined } : b)) }
      : s,
  );
  return { ...c, gestures, scopes };
}

/**
 * 「只改这里」：把共享手势在本次编辑中的修改收回到当前绑定——
 * 当前绑定解除引用并保留修改后的画法，库与其他引用处恢复为 `before`。
 */
export function keepChangeHere(c: Config, ref: BindingRef, before: Step[]): Config {
  const b = c.scopes[ref.scope]?.bindings[ref.binding];
  if (!b?.gesture) return c;
  const mine = b.pattern;
  const restored = setGesturePattern(c, b.gesture, before);
  return updateAt(restored, ref, (x) => ({ ...x, gesture: undefined, pattern: mine }));
}

/** 违反 I1 的绑定位置。 */
export function checkRefs(c: Config): BindingRef[] {
  const out: BindingRef[] = [];
  c.scopes.forEach((s, scope) =>
    s.bindings.forEach((b, binding) => {
      if (!b.gesture) return;
      const g = gestureById(c, b.gesture);
      if (!g || !deepEqual(g.pattern, b.pattern)) out.push({ scope, binding });
    }),
  );
  return out;
}

/** 兜底修复：违反 I1 的绑定改为本程序专用（保留它自己的 pattern）。 */
export function repairRefs(c: Config): Config {
  const broken = checkRefs(c);
  if (!broken.length) return c;
  return mapBindings(c, (b, r) => (broken.some((x) => x.scope === r.scope && x.binding === r.binding) ? { ...b, gesture: undefined } : b));
}

/** 这个手势在各个使用它的分组里与哪些绑定冲突（冲突仍按分组计算）。 */
export function gestureConflicts(c: Config, id: string): { use: BindingRef; others: number[] }[] {
  const out: { use: BindingRef; others: number[] }[] = [];
  const cache = new Map<number, Map<number, number[]>>();
  for (const use of gestureUses(c, id)) {
    if (!cache.has(use.scope)) cache.set(use.scope, conflicts(c.scopes[use.scope].bindings));
    const others = cache.get(use.scope)!.get(use.binding);
    if (others?.length) out.push({ use, others });
  }
  return out;
}

/**
 * 合并另一份配置的手势库：同画法的映射到已有 id，id 冲突且画法不同的换新 id。
 * 返回合并后的库，以及 incoming 中 id → 合并后 id 的映射。
 */
export function mergeLibraries(base: Config, incoming: Config): { gestures: Gesture[]; remap: Map<string, string> } {
  const gestures = [...gesturesOf(base)];
  const remap = new Map<string, string>();
  for (const g of gesturesOf(incoming)) {
    const same = gestures.find((x) => deepEqual(x.pattern, g.pattern));
    if (same) {
      remap.set(g.id, same.id);
      continue;
    }
    const id = gestures.some((x) => x.id === g.id) ? newGestureId({ ...base, gestures }) : g.id;
    gestures.push({ ...g, id });
    remap.set(g.id, id);
  }
  return { gestures, remap };
}

// ================================================================ 整理并导入

/** 用户对每组的选择：是否合并、以谁为准（组代表 / 某个成员 / 合并样本）；`skip` = 这组不导入（绑定保持专用）。 */
export interface GroupChoice {
  checked: boolean;
  choice: "rep" | number | "merge";
  skip?: boolean;
}

export const defaultChoices = (plan: ImportPlan): GroupChoice[] => plan.groups.map((g) => ({ checked: g.checked, choice: g.choice ?? "rep" }));

/** 组合并后的画法。 */
export function groupPattern(c: Config, g: ImportGroup, ch: GroupChoice): Step[] {
  if (!ch.checked || ch.choice === "rep") return g.pattern;
  if (ch.choice === "merge" && g.merged) return g.pattern.map((s) => (s.type === "shape" ? { ...s, templates: g.merged!.templates } : s));
  if (typeof ch.choice === "number") {
    const m = g.members[ch.choice];
    return c.scopes[m.scope]?.bindings[m.binding]?.pattern ?? g.pattern;
  }
  return g.pattern;
}

export interface ImportStats {
  added: number;
  linked: number;
  replaced: number;
}

/** 按分析结果与用户选择建库、关联（纯函数，一次 edit 应用；也用于预览数字）。 */
export function applyImportPlan(c: Config, plan: ImportPlan, choices: GroupChoice[]): { config: Config; stats: ImportStats } {
  const stats: ImportStats = { added: 0, linked: 0, replaced: 0 };
  // 先记下所有成员原来的画法（下标基于分析时的配置）
  const original = (r: BindingRef) => c.scopes[r.scope]?.bindings[r.binding]?.pattern;
  let config = c;
  plan.groups.forEach((g, gi) => {
    const ch = choices[gi] ?? { checked: g.checked, choice: "rep" };
    if (ch.skip) return;
    const pattern = groupPattern(c, g, ch);
    let id = g.existing;
    if (!id) {
      const added = addGesture(config, pattern);
      config = added.config;
      id = added.id;
      stats.added++;
    } else if (ch.checked && !deepEqual(gestureById(config, id)?.pattern, pattern)) {
      config = setGesturePattern(config, id, pattern);
    }
    for (const m of g.members) {
      if (m.relation === "linked") continue;
      const before = original(m);
      if (!before) continue;
      const join = m.relation === "seed" || m.relation === "identical" || ch.checked;
      if (join) {
        if (!deepEqual(before, pattern)) stats.replaced++;
        config = link(config, m, id);
        stats.linked++;
      } else if (plan.source === "all") {
        const own = addGesture(config, before);
        config = link(own.config, m, own.id);
        stats.added++;
        stats.linked++;
      }
    }
  });
  return { config, stats };
}
