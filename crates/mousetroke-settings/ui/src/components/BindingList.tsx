import { primaryModifier } from "../platform";
import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import type { Binding, FullscreenPolicy, MouseButton, Scope, Step } from "../types";
import { actionKeys, presetDefinition, presetTypeLabel } from "../actions";
import { EDGE, actionSummary, buttonName, decompose, describePattern, edgeName, fileName, hasConflictPattern } from "../gesture";
import { currentLang, displayName, t } from "../i18n";
import { PatternGlyph } from "./Glyph";
import { AppIcon, EdgeIcon, MouseIcon, TrashIcon } from "./Icons";
import { Caps } from "./KeysInput";
import { scopeTitle } from "./Sidebar";

/** 列表分类：鼠标手势（按住某个键画）/ 触发角（推到屏幕边角）。 */
export type ListTab = "gesture" | "edge";
/** 筛选："all" | `btn:<键>` | `edge:<位标志>` */
export type ListFilter = string;

const BUTTON_ORDER: MouseButton[] = ["right", "middle", "x1", "x2", "left"];
const EDGE_ORDER = [EDGE.leftTop, EDGE.top, EDGE.rightTop, EDGE.left, EDGE.right, EDGE.leftBottom, EDGE.bottom, EDGE.rightBottom];

/** 绑定的分类与筛选键。 */
function classify(b: Binding): { tab: ListTab; key: string } {
  const t0 = decompose(b.pattern).trigger;
  return t0.kind === "edge" ? { tab: "edge", key: `edge:${t0.edge}` } : { tab: "gesture", key: `btn:${t0.button}` };
}

export function BindingList({
  scope,
  pauseOnFullscreen,
  tab,
  onTab,
  filter,
  onFilter,
  selected,
  onSelect,
  onAdd,
  onDelete,
  onDuplicate,
  onTransfer,
  onToggle,
  onScopeChange,
  onDeleteScope,
  clash,
}: {
  scope: Scope;
  pauseOnFullscreen: boolean;
  tab: ListTab;
  onTab: (t: ListTab) => void;
  filter: ListFilter;
  onFilter: (f: ListFilter) => void;
  selected: number | null;
  onSelect: (i: number | null) => void;
  /** 新建：按当前分类/筛选给出初始模式。 */
  onAdd: (pattern: Step[]) => void;
  onDelete: (i: number) => void;
  onDuplicate: (i: number) => void;
  onTransfer: (i: number) => void;
  onToggle: (i: number) => void;
  onScopeChange: (f: (s: Scope) => Scope, coalesce?: string) => void;
  onDeleteScope: () => void;
  /** 冲突表（由上层计算一次，编辑面板与列表共用）。 */
  clash: Map<number, number[]>;
}) {
  const [query, setQuery] = useState("");
  const [context, setContext] = useState<{ index: number; x: number; y: number } | null>(null);
  const search = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLUListElement>(null);
  const contextRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!context) return;
    contextRef.current?.querySelector<HTMLButtonElement>("button")?.focus();
    const close = (e: MouseEvent) => { if (!contextRef.current?.contains(e.target as Node)) setContext(null); };
    const escape = (e: KeyboardEvent) => { if (e.key === "Escape") { e.stopPropagation(); setContext(null); } };
    window.addEventListener("mousedown", close);
    window.addEventListener("keydown", escape, true);
    return () => { window.removeEventListener("mousedown", close); window.removeEventListener("keydown", escape, true); };
  }, [context]);

  const classes = useMemo(() => scope.bindings.map(classify), [scope.bindings]);
  const tabCount = (t0: ListTab) => classes.filter((c) => c.tab === t0).length;

  // 当前分类下出现过的筛选项及数量（按固定顺序）
  const filters = useMemo(() => {
    const count = new Map<string, number>();
    classes.forEach((c) => c.tab === tab && count.set(c.key, (count.get(c.key) ?? 0) + 1));
    const keys = tab === "gesture" ? BUTTON_ORDER.map((b) => `btn:${b}`) : EDGE_ORDER.map((e) => `edge:${e}`);
    const off = scope.bindings.filter((b, i) => b.disabled && !b.draft && classes[i].tab === tab).length;
    return [...keys.filter((k) => count.has(k)).map((k) => ({ key: k, n: count.get(k)! })), ...(off ? [{ key: "off", n: off }] : [])];
  }, [classes, tab, scope.bindings]);
  const activeFilter = filters.some((f) => f.key === filter) ? filter : "all";

  const lang = currentLang();
  const visible = useMemo(() => {
    const q = query.trim().toLowerCase();
    return scope.bindings
      .map((b, i) => ({ b, i }))
      .filter(({ b, i }) => classes[i].tab === tab && (activeFilter === "all" || classes[i].key === activeFilter || (activeFilter === "off" && !!b.disabled && !b.draft)))
      // 按界面上显示的名字搜（内置名随语言翻译，b.name 只是导入时那种语言的文字）
      .filter(
        ({ b }) =>
          !q ||
          displayName(b.name, b.name_key).toLowerCase().includes(q) ||
          describePattern(b.pattern).toLowerCase().includes(q) ||
          actionSummary(b.action).toLowerCase().includes(q),
      );
    // lang：画法描述、动作摘要、内置名都随语言变化，切换语言后要重新匹配
  }, [scope.bindings, classes, tab, activeFilter, query, lang]);

  const add = () => {
    const [kind, val] = activeFilter.split(":");
    if (tab === "edge") onAdd([{ type: "edge", edge: kind === "edge" ? Number(val) : EDGE.leftBottom }]);
    else onAdd([{ type: "press", key: `mouse:${kind === "btn" ? val : "right"}` }]);
  };
  const addLabel = tab === "edge" ? t("ui.list.add_edge") : t("ui.list.add_gesture");

  // Ctrl+F 聚焦搜索
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (primaryModifier(e) && e.key.toLowerCase() === "f") {
        e.preventDefault();
        search.current?.focus();
        search.current?.select();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  // 选中项滚动到可见
  useEffect(() => {
    listRef.current?.querySelector(".row.on")?.scrollIntoView({ block: "nearest" });
  }, [selected]);

  const onListKey = (e: React.KeyboardEvent) => {
    const pos = visible.findIndex((v) => v.i === selected);
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      const next = visible[Math.min(visible.length - 1, Math.max(0, pos + (e.key === "ArrowDown" ? 1 : -1)))];
      if (next) onSelect(next.i);
    } else if (e.key === " " && selected != null) {
      e.preventDefault();
      onToggle(selected);
    } else if (e.key === "Delete" && selected != null) {
      e.preventDefault();
      onDelete(selected);
    } else if (e.key.toLowerCase() === "d" && primaryModifier(e) && selected != null) {
      e.preventDefault();
      onDuplicate(selected);
    }
  };

  const isApp = scope.kind !== "global";

  // 分类标签的滑块：量出选中标签的位置，用 transform 平滑滑过去
  const tabsRef = useRef<HTMLDivElement>(null);
  const [pill, setPill] = useState<{ x: number; w: number } | null>(null);
  useLayoutEffect(() => {
    const el = tabsRef.current?.querySelector<HTMLElement>(".tab.on");
    if (el) setPill({ x: el.offsetLeft, w: el.offsetWidth });
  }, [tab, classes.length]);
  const off = isApp && !!scope.disabled;
  const where = scope.kind === "app" ? fileName(scope.app?.exe ?? "") : t("scope.desktop");
  const setOff = (v: boolean) => onScopeChange((s) => ({ ...s, disabled: v ? true : undefined }));

  return (
    <section className="list-pane" aria-label={t("ui.list.aria")}>
      <header className="list-head">
        {scope.kind === "app" && (
          <span className="list-app-icon">
            <AppIcon exe={scope.app?.exe} size={32} />
          </span>
        )}
        <div className="list-title">
          {scope.kind === "app" ? (
            <input
              className="title-input"
              value={displayName(scope.name, scope.name_key)}
              aria-label={t("ui.list.name_aria")}
              onChange={(e) => onScopeChange((s) => ({ ...s, name: e.target.value, name_key: undefined }), "scope-name")}
            />
          ) : (
            <h1>{scopeTitle(scope)}</h1>
          )}
          <p className="list-sub">
            {scope.kind === "global"
              ? t("ui.list.sub_global")
              : scope.kind === "special"
                ? t("ui.list.sub_special")
                : off
                  ? t("ui.list.sub_off", { where })
                  : t("ui.list.sub_app", { where })}
          </p>
        </div>
        {scope.kind === "app" && (
          <button type="button" className="btn icon subtle remove-scope" aria-label={t("ui.list.remove_app")} data-tip={t("ui.list.remove_app")} onClick={onDeleteScope}>
            <TrashIcon />
          </button>
        )}
      </header>

      {isApp && (
        <div className={`scope-props ${off ? "off" : ""}`}>
          <label className="scope-prop scope-prop-main">
            <span className="scope-prop-text">
              <b>{t("ui.list.use_gestures")}</b>
              <small>{off ? t("ui.list.use_gestures_off", { where }) : t("ui.list.use_gestures_on")}</small>
            </span>
            <input type="checkbox" className="switch" checked={!off} onChange={(e) => setOff(!e.target.checked)} />
          </label>
          <label className="scope-prop">
            <span className="scope-prop-text">
              <b>{t("ui.list.inherit_global")}</b>
              <small>{t("ui.list.inherit_global_help")}</small>
            </span>
            <input
              type="checkbox"
              className="switch"
              disabled={off}
              checked={scope.inherit_global !== false}
              onChange={(e) => onScopeChange((s) => ({ ...s, inherit_global: e.target.checked }))}
            />
          </label>
          {scope.kind === "app" && (
            <label className="scope-prop">
              <span className="scope-prop-text">
                <b>{t("ui.list.fullscreen")}</b>
                <small>{t(scope.fullscreen_policy ? "ui.list.fullscreen_help_custom" : "ui.list.fullscreen_help_inherit")}</small>
              </span>
              <select
                className="scope-select"
                disabled={off}
                value={scope.fullscreen_policy ?? "inherit"}
                onChange={(e) => {
                  const policy = e.target.value as FullscreenPolicy;
                  onScopeChange((s) => ({ ...s, fullscreen_policy: policy === "inherit" ? undefined : policy }));
                }}
              >
                <option value="inherit">{t("ui.list.fullscreen_inherit", { state: t(pauseOnFullscreen ? "ui.list.fullscreen_state_pause" : "ui.list.fullscreen_state_allow") })}</option>
                <option value="allow">{t("ui.list.fullscreen_allow")}</option>
                <option value="pause">{t("ui.list.fullscreen_pause")}</option>
              </select>
            </label>
          )}
        </div>
      )}

      <div className={`list-toolbar ${off ? "dimmed" : ""}`}>
        <div className="tabs" role="tablist" aria-label={t("ui.list.tabs_aria")} ref={tabsRef}>
          {pill && <span className="tab-pill" style={{ transform: `translateX(${pill.x}px)`, width: pill.w }} aria-hidden />}
          {(["gesture", "edge"] as const).map((tt) => (
            <button
              key={tt}
              type="button"
              role="tab"
              aria-selected={tab === tt}
              className={`tab ${tab === tt ? "on" : ""}`}
              onClick={() => {
                onTab(tt);
                onSelect(null);
              }}
            >
              {tt === "gesture" ? <MouseIcon /> : <EdgeIcon edge={EDGE.leftTop} />}
              {tt === "gesture" ? t("ui.list.tab_gestures") : t("ui.list.tab_edges")}
              <span className="tab-count">{tabCount(tt)}</span>
            </button>
          ))}
        </div>
        <input ref={search} className="search" type="search" placeholder={t("ui.list.search_ph")} value={query} onChange={(e) => setQuery(e.target.value)} />
        <button type="button" className="btn primary toolbar-add" onClick={add}>
          {addLabel}
        </button>
      </div>

      {filters.length > 1 && (
      <div className={`filter-bar ${off ? "dimmed" : ""}`}>
        <div className="filters" role="radiogroup" aria-label={tab === "edge" ? t("ui.list.filter_edges") : t("ui.list.filter_buttons")}>
          <button type="button" role="radio" aria-checked={activeFilter === "all"} className={`filter ${activeFilter === "all" ? "on" : ""}`} onClick={() => onFilter("all")}>
            {t("common.all")}
            <span className="filter-count">{tabCount(tab)}</span>
          </button>
          {filters.map(({ key, n }) => {
            const [kind, val] = key.split(":");
            if (kind === "off")
              return (
                <button key={key} type="button" role="radio" aria-checked={activeFilter === key} className={`filter filter-off ${activeFilter === key ? "on" : ""}`} onClick={() => onFilter(key)}>
                  {t("ui.list.off_filter")}
                  <span className="filter-count">{n}</span>
                </button>
              );
            return (
              <button key={key} type="button" role="radio" aria-checked={activeFilter === key} className={`filter ${activeFilter === key ? "on" : ""}`} onClick={() => onFilter(key)}>
                {kind === "btn" ? <MouseIcon button={val as MouseButton} /> : <EdgeIcon edge={Number(val)} />}
                {kind === "btn" ? buttonName(val as MouseButton) : edgeName(Number(val))}
                <span className="filter-count">{n}</span>
              </button>
            );
          })}
        </div>
      </div>
      )}

      {visible.length === 0 && !query ? (
        <div className="empty">
          <p>
            {tab === "edge"
              ? t("ui.list.empty_edges")
              : scope.kind === "global"
                ? t("ui.list.empty_gestures")
                : t("ui.list.empty_app", { scope: scopeTitle(scope) })}
          </p>
          <p className="muted">
            {tab === "edge"
              ? t("ui.list.empty_edges_hint")
              : scope.kind === "global"
                ? t("ui.list.empty_gestures_hint")
                : t("ui.list.empty_app_hint")}
          </p>
          <button type="button" className="btn primary" onClick={add}>
            {addLabel}
          </button>
        </div>
      ) : visible.length === 0 ? (
        <div className="empty">
          <p>{t("ui.list.no_match", { query })}</p>
        </div>
      ) : (
        <ul key={`${tab}-${activeFilter}`} ref={listRef} className={`rows ${off ? "dimmed" : ""}`} role="listbox" aria-label={t("ui.list.rows_aria")} tabIndex={0} onKeyDown={onListKey}>
          {visible.map(({ b, i }, n) => (
            <Row key={i} n={n} b={b} on={selected === i} clash={clash.has(i)} onClick={() => onSelect(i)} onToggle={() => onToggle(i)}
              onContextMenu={(x, y) => {
                onSelect(i);
                setContext({ index: i, x: Math.max(8, Math.min(x, window.innerWidth - 230)), y: Math.max(8, Math.min(y, window.innerHeight - 90)) });
              }} />
          ))}
        </ul>
      )}
      {context && createPortal(
        <div ref={contextRef} className="gesture-context-menu popover" role="menu" style={{ position: "fixed", left: context.x, top: context.y, zIndex: 1000 }}>
          <button type="button" role="menuitem" onClick={() => { onTransfer(context.index); setContext(null); }}>{t("ui.transfer.open")}</button>
        </div>, document.body,
      )}
    </section>
  );
}

function Row({ n, b, on, clash, onClick, onToggle, onContextMenu }: { n: number; b: Binding; on: boolean; clash: boolean; onClick: () => void; onToggle: () => void; onContextMenu: (x: number, y: number) => void }) {
  const name = displayName(b.name, b.name_key);
  const enabled = !b.disabled || !!b.draft;
  const incomplete = !!b.draft && !hasConflictPattern(b.pattern);
  return (
    <li
      role="option"
      aria-selected={on}
      className={`row ${on ? "on" : ""} ${!enabled ? "off" : ""}`}
      style={{ "--i": Math.min(n, 12) } as React.CSSProperties}
      onClick={onClick}
      onContextMenu={(e) => { e.preventDefault(); if (!b.draft) onContextMenu(e.clientX, e.clientY); }}
    >
      <span className="row-glyph">
        <PatternGlyph pattern={b.pattern} size={36} />
      </span>
      <span className="row-main">
        <span className="row-name">
          {name || <em className="muted">{t("ui.list.unnamed")}</em>}
          {!enabled && <span className="badge">{t("ui.list.off_badge")}</span>}
          {b.draft && (
            <span className="badge" data-tip={t("ui.list.inactive_tip")}>
              {t("ui.list.inactive_badge")}
            </span>
          )}
          {clash && (
            <span className="badge warn" data-tip={t("ui.list.dup_tip")}>
              {t("ui.list.dup_badge")}
            </span>
          )}
        </span>
        <span className="row-pattern">{describePattern(b.pattern)}</span>
      </span>
      <span className="row-action">
        {incomplete || b.actionUnselected ? <span className="row-action-text muted">{[incomplete && t("ui.list.no_stroke"), b.actionUnselected && t("ui.actpick.unselected")].filter(Boolean).join(" · ")}</span> : actionKeys(b.action) !== undefined && !b.secured ? <Caps keys={actionKeys(b.action)!} compact /> : <span className="row-action-text">{b.secured ? "••••" : actionSummary(b.action) === name && presetDefinition(b.action) ? presetTypeLabel(presetDefinition(b.action)!) : actionSummary(b.action)}</span>}
      </span>
      <input
        type="checkbox"
        className="switch small"
        checked={enabled}
        data-tip={enabled ? t("ui.list.on_toggle_tip") : t("ui.list.off_toggle_tip")}
        aria-label={enabled ? t("ui.list.disable_aria", { name }) : t("ui.list.enable_aria", { name })}
        onClick={(e) => e.stopPropagation()}
        onChange={onToggle}
      />
    </li>
  );
}
