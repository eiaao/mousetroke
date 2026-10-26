import { useCallback, useReducer } from "react";
import type { Config } from "./types";
import { persistedAction } from "./actions";
import { hasConflictPattern, uid } from "./gesture";

// 配置编辑状态：不可变更新 + 撤销/重做。
// 同一字段的连续输入（相同 coalesce 键、间隔 < 1s）合并为一步撤销。

interface State {
  config: Config;
  past: Config[];
  future: Config[];
  /** 每次实际变更 +1，驱动自动保存。 */
  revision: number;
  lastKey: string | null;
  lastAt: number;
}

type Msg =
  | { t: "edit"; f: (c: Config) => Config; key?: string }
  | { t: "load"; config: Config; keepHistory: boolean }
  | { t: "undo" }
  | { t: "redo" };

const HISTORY = 100;

function reducer(s: State, m: Msg): State {
  switch (m.t) {
    case "edit": {
      const next = m.f(s.config);
      if (next === s.config) return s;
      const now = Date.now();
      const merge = m.key != null && m.key === s.lastKey && now - s.lastAt < 1000;
      return {
        config: withBindingIds(next),
        past: merge ? s.past : [...s.past.slice(-HISTORY + 1), s.config],
        future: [],
        revision: s.revision + 1,
        lastKey: m.key ?? null,
        lastAt: now,
      };
    }
    case "load":
      return {
        config: withBindingIds(m.config),
        past: m.keepHistory ? [...s.past.slice(-HISTORY + 1), s.config] : [],
        future: [],
        revision: m.keepHistory ? s.revision + 1 : s.revision,
        lastKey: null,
        lastAt: 0,
      };
    case "undo": {
      const prev = s.past[s.past.length - 1];
      if (!prev) return s;
      return { ...s, config: prev, past: s.past.slice(0, -1), future: [s.config, ...s.future], revision: s.revision + 1, lastKey: null };
    }
    case "redo": {
      const [next, ...rest] = s.future;
      if (!next) return s;
      return { ...s, config: next, past: [...s.past, s.config], future: rest, revision: s.revision + 1, lastKey: null };
    }
  }
}

export function useConfigStore(initial: Config) {
  const [s, dispatch] = useReducer(reducer, {
    config: withBindingIds(initial),
    past: [],
    future: [],
    revision: 0,
    lastKey: null,
    lastAt: 0,
  });
  const edit = useCallback((f: (c: Config) => Config, key?: string) => dispatch({ t: "edit", f, key }), []);
  const load = useCallback((config: Config, keepHistory = true) => dispatch({ t: "load", config, keepHistory }), []);
  const undo = useCallback(() => dispatch({ t: "undo" }), []);
  const redo = useCallback(() => dispatch({ t: "redo" }), []);
  return {
    config: s.config,
    revision: s.revision,
    canUndo: s.past.length > 0,
    canRedo: s.future.length > 0,
    edit,
    load,
    undo,
    redo,
  };
}

/** 浅拷贝修改某个作用域的某条绑定。 */
export function updateBinding(c: Config, scopeIdx: number, bindingIdx: number, f: (b: Config["scopes"][0]["bindings"][0]) => Config["scopes"][0]["bindings"][0]): Config {
  return updateScope(c, scopeIdx, (s) => ({ ...s, bindings: s.bindings.map((b, i) => (i === bindingIdx ? f(b) : b)) }));
}

export function updateScope(c: Config, scopeIdx: number, f: (s: Config["scopes"][0]) => Config["scopes"][0]): Config {
  return { ...c, scopes: c.scopes.map((s, i) => (i === scopeIdx ? f(s) : s)) };
}

/** 稳定标识跨编辑与排序保留；仅在模式和动作均已完成时启用草稿。 */
export function withBindingIds(config: Config): Config {
  return { ...config, scopes: config.scopes.map((scope) => ({ ...scope, bindings: scope.bindings.map((binding) => {
    // 未选择动作的空按键序列也会落盘；重载草稿时恢复这个仅界面标记。
    const actionUnselected = binding.actionUnselected ?? (binding.draft && binding.action.type === "key_sequence" && binding.action.keys === "" ? true : undefined);
    const b = { ...binding, actionUnselected, uiId: binding.uiId ?? uid() };
    return b.draft && hasConflictPattern(b.pattern) && !b.actionUnselected ? { ...b, draft: undefined, disabled: undefined } : b;
  }) })) };
}

export function persistedConfig(config: Config): Config {
  return { ...config, scopes: config.scopes.map((scope) => ({ ...scope, bindings: scope.bindings.map((binding) => {
    const { uiId: _id, actionUnselected: _unselected, lastActionName: _name, ...saved } = binding;
    return { ...saved, action: persistedAction(saved.action) };
  }) })) };
}
