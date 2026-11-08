import { useEffect, useMemo, useRef, useState } from "react";
import type { Action, Binding, Config, Gesture, MouseButton, Step } from "../types";
import { EDGE, actionSummary, buttonName, decompose, describePattern, edgeName, gestureName, hasConflictPattern, patternKey, quote, quoteJoin } from "../gesture";
import { currentLang, displayName, t } from "../i18n";
import { categoryOf, gestureById, gestureConflicts, gestureUses, gesturesOf, libraryReady, type BindingRef } from "../library";
import { PatternGlyph } from "./Glyph";
import { AppIcon, EdgeIcon, MouseIcon, NavIcon } from "./Icons";
import { PatternEditor } from "./PatternEditor";
import { scopeTitle } from "./Sidebar";
import { Modal } from "./Dialogs";
import { ActionPicker } from "./ActionPicker";
import { Caps } from "./KeysInput";
import { actionKeys } from "../actions";

export type LibraryTab = "gesture" | "edge";
/** 筛选："all" | "unused" | `btn:<键>` | `edge:<位标志>` */
export type LibraryFilter = string;

const BUTTON_ORDER: MouseButton[] = ["right", "middle", "x1", "x2", "left"];
const EDGE_ORDER = [EDGE.leftTop, EDGE.top, EDGE.rightTop, EDGE.left, EDGE.right, EDGE.leftBottom, EDGE.bottom, EDGE.rightBottom];

function filterKey(pattern: Step[]): string {
  const tr = decompose(pattern).trigger;
  return tr.kind === "edge" ? `edge:${tr.edge}` : `btn:${tr.button}`;
}

/** 字形 + 触发键角标（缩略图对不同触发键是一样的，靠角标区分）。 */
export function GestureGlyph({ pattern, size = 40 }: { pattern: Step[]; size?: number }) {
  const tr = decompose(pattern).trigger;
  return (
    <span className="gesture-glyph">
      <PatternGlyph pattern={pattern} size={size} />
      {tr.kind === "button" && (
        <span className="gesture-badge" data-tip={buttonName(tr.button)}>
          <MouseIcon button={tr.button} size={12} />
        </span>
      )}
    </span>
  );
}

/** 绑定所在位置的文字：zh「所有程序」「后退」，en “Back” in “All apps”。 */
export function placeText(config: Config, ref: BindingRef): string {
  const scope = config.scopes[ref.scope];
  const b = scope?.bindings[ref.binding];
  if (!scope || !b) return "";
  return t("ui.gestures.place", { scope: quote(scopeTitle(scope)), name: quote(displayName(b.name, b.name_key)) });
}

/** 这个手势在其他分组里的冲突提示（当前分组的冲突由编辑器自己提示）。 */
export function conflictLines(config: Config, id: string, exceptScope?: number): string[] {
  return gestureConflicts(config, id)
    .filter(({ use }) => use.scope !== exceptScope)
    .map(({ use, others }) => {
      const scope = config.scopes[use.scope];
      const b = scope.bindings[use.binding];
      return t("ui.gestures.conflict_in", {
        scope: quote(scopeTitle(scope)),
        name: quote(displayName(b.name, b.name_key)),
        others: quoteJoin(others.map((j) => displayName(scope.bindings[j].name, scope.bindings[j].name_key))),
      });
    });
}

export function GestureLibrary({
  config,
  selected,
  onSelect,
  onNew,
  onImport,
  tab,
  onTab,
  filter,
  onFilter,
}: {
  config: Config;
  selected: string | null;
  onSelect: (id: string | null) => void;
  onNew: (tab: LibraryTab) => void;
  onImport: () => void;
  tab: LibraryTab;
  onTab: (tab: LibraryTab) => void;
  filter: LibraryFilter;
  onFilter: (f: LibraryFilter) => void;
}) {
  const [query, setQuery] = useState("");
  const gestures = gesturesOf(config);
  const lang = currentLang();
  const useCount = useMemo(() => new Map(gestures.map((g) => [g.id, gestureUses(config, g.id).length])), [config, gestures]);
  const inTab = useMemo(() => gestures.filter((g) => categoryOf(g.pattern) === tab), [gestures, tab]);
  const filters = useMemo(() => {
    const count = new Map<string, number>();
    inTab.forEach((g) => count.set(filterKey(g.pattern), (count.get(filterKey(g.pattern)) ?? 0) + 1));
    const keys = tab === "gesture" ? BUTTON_ORDER.map((b) => `btn:${b}`) : EDGE_ORDER.map((e) => `edge:${e}`);
    const unused = inTab.filter((g) => !useCount.get(g.id)).length;
    return [...keys.filter((k) => count.has(k)).map((k) => ({ key: k, n: count.get(k)! })), ...(unused ? [{ key: "unused", n: unused }] : [])];
  }, [inTab, tab, useCount]);
  const activeFilter = filters.some((f) => f.key === filter) ? filter : "all";

  const visible = useMemo(() => {
    const q = query.trim().toLowerCase();
    return inTab.filter((g) => {
      if (activeFilter === "unused" ? useCount.get(g.id) : activeFilter !== "all" && filterKey(g.pattern) !== activeFilter) return false;
      return gestureMatches(config, g, q);
    });
    // lang：自动名称与画法描述随语言变化
  }, [inTab, activeFilter, query, config, useCount, lang]);

  if (!gestures.length) {
    return (
      <section className="list-pane library" aria-label={t("ui.gestures.title")}>
        <header className="list-head">
          <div className="list-title">
            <h1>{t("ui.gestures.title")}</h1>
            <p className="list-sub">{t("ui.gestures.sub")}</p>
          </div>
        </header>
        <div className="empty library-empty">
          <div className="library-empty-art" aria-hidden>
            <PatternGlyph pattern={[{ type: "press", key: "mouse:right" }, { type: "trace", points: [50, 0, 0, 0] }]} size={44} />
            <PatternGlyph pattern={[{ type: "press", key: "mouse:right" }, { type: "trace", points: [0, 0, 0, 50, 50, 50] }]} size={44} />
            <PatternGlyph pattern={[{ type: "edge", edge: EDGE.leftTop }]} size={44} />
          </div>
          <p className="library-empty-title">{t("ui.gestures.empty_title")}</p>
          <p className="muted">{t("ui.gestures.empty_body")}</p>
          <p className="muted">{t("ui.gestures.empty_existing")}</p>
          <div className="btn-row">
            <button type="button" className="btn primary" onClick={onImport}>{t("ui.gestures.empty_import")}</button>
            <button type="button" className="btn" onClick={() => onNew(tab)}>{t("ui.gestures.new")}</button>
          </div>
        </div>
      </section>
    );
  }

  const tabCount = (tt: LibraryTab) => gestures.filter((g) => categoryOf(g.pattern) === tt).length;
  return (
    <section className="list-pane library" aria-label={t("ui.gestures.title")}>
      <header className="list-head">
        <div className="list-title">
          <h1>{t("ui.gestures.title")}</h1>
          <p className="list-sub">{t("ui.gestures.sub")}</p>
        </div>
        <div className="list-scope-tools">
          <button type="button" className="btn small subtle" onClick={onImport}>{t("ui.gestures.import")}</button>
        </div>
      </header>

      <div className="list-toolbar">
        <div className="tabs" role="tablist">
          {(["gesture", "edge"] as const).map((tt) => (
            <button key={tt} type="button" role="tab" aria-selected={tab === tt} className={`tab ${tab === tt ? "on" : ""}`} onClick={() => onTab(tt)}>
              {tt === "gesture" ? <MouseIcon /> : <EdgeIcon edge={EDGE.leftTop} />}
              {tt === "gesture" ? t("ui.list.tab_gestures") : t("ui.list.tab_edges")}
              <span className="tab-count">{tabCount(tt)}</span>
            </button>
          ))}
        </div>
        <button type="button" className="btn primary toolbar-add" onClick={() => onNew(tab)}>{t("ui.gestures.new")}</button>
      </div>

      <div className="filter-bar">
        {filters.length > 0 && (
          <div className="filters" role="radiogroup">
            <button type="button" role="radio" aria-checked={activeFilter === "all"} className={`filter ${activeFilter === "all" ? "on" : ""}`} onClick={() => onFilter("all")}>
              {t("common.all")}
              <span className="filter-count">{inTab.length}</span>
            </button>
            {filters.map(({ key, n }) => {
              const [kind, val] = key.split(":");
              return (
                <button key={key} type="button" role="radio" aria-checked={activeFilter === key} className={`filter ${kind === "unused" ? "filter-off" : ""} ${activeFilter === key ? "on" : ""}`} onClick={() => onFilter(key)}>
                  {kind === "btn" && <MouseIcon button={val as MouseButton} />}
                  {kind === "edge" && <EdgeIcon edge={Number(val)} />}
                  {kind === "unused" ? t("ui.gestures.filter_unused") : kind === "btn" ? buttonName(val as MouseButton) : edgeName(Number(val))}
                  <span className="filter-count">{n}</span>
                </button>
              );
            })}
          </div>
        )}
        <input className="search" type="search" placeholder={t("ui.gestures.search_ph")} value={query} onChange={(e) => setQuery(e.target.value)} />
      </div>

      {visible.length === 0 ? (
        <div className="empty"><p>{t("ui.gestures.no_match")}</p></div>
      ) : (
        <ul className="gesture-grid" role="listbox" aria-label={t("ui.gestures.title")}>
          {visible.map((g) => {
            const n = useCount.get(g.id) ?? 0;
            return (
              <li key={g.id} role="option" aria-selected={selected === g.id} className={`gesture-card ${selected === g.id ? "on" : ""}`}>
                <button type="button" onClick={() => onSelect(g.id)}>
                  <GestureGlyph pattern={g.pattern} size={46} />
                  <span className="gesture-card-name" title={gestureName(g)}>{gestureName(g)}</span>
                  <span className="gesture-card-uses">{n ? t("ui.gestures.uses", { count: n }) : <span className="badge">{t("ui.gestures.unused_badge")}</span>}</span>
                </button>
              </li>
            );
          })}
        </ul>
      )}
    </section>
  );
}

/** 「在其他程序中使用」的目标：已选分组后弹出动作选择器。 */
export interface AssignTarget {
  scope: number;
}

/** 搜索手势：名称、画法描述、用在哪个程序、对应的动作名（`q` 已转小写）。 */
export function gestureMatches(config: Config, g: Gesture, q: string): boolean {
  if (!q) return true;
  const uses = gestureUses(config, g.id).flatMap((r) => {
    const s = config.scopes[r.scope];
    const b = s.bindings[r.binding];
    return [scopeTitle(s), displayName(b.name, b.name_key)];
  });
  return [gestureName(g), describePattern(g.pattern), ...uses].some((s) => s.toLowerCase().includes(q));
}

export function GestureDetail({
  config,
  id,
  assignScope,
  startAssign,
  onAssignStarted,
  onPattern,
  onRename,
  onCreate,
  highlight,
  onJump,
  onAssign,
  onAssignCancel,
  onAssignScope,
  onAddApp,
  onDelete,
  onClose,
}: {
  config: Config;
  /** 库中手势 id；"new" = 新建（录完才入库）。 */
  id: string;
  /** 「在其他程序中使用」已选定的分组（添加程序后由上层直接给出）。 */
  assignScope: number | null;
  /** 打开时直接弹出程序选择（来自编辑器的「在其他程序中使用」）。 */
  startAssign?: boolean;
  onAssignStarted?: () => void;
  onPattern: (pattern: Step[]) => void;
  onRename: (name: string) => void;
  onCreate: (pattern: Step[], name: string) => void;
  /** 刚分配的绑定（uiId）：用法表里高亮并滚到可见。 */
  highlight?: string | null;
  onJump: (ref: BindingRef) => void;
  onAssign: (scope: number, action: Action) => void;
  onAssignCancel: () => void;
  onAssignScope: (scope: number) => void;
  onAddApp: () => void;
  onDelete: (mode: "detach" | "cascade") => void;
  onClose: () => void;
}) {
  const gesture = id === "new" ? undefined : gestureById(config, id);
  const [draft, setDraft] = useState<Step[]>([{ type: "press", key: "mouse:right" }]);
  const [newName, setNewName] = useState("");
  const [choosingScope, setChoosingScope] = useState(!!startAssign);
  useEffect(() => {
    if (startAssign) onAssignStarted?.();
  }, []); // eslint-disable-line react-hooks/exhaustive-deps
  const [confirmDelete, setConfirmDelete] = useState(false);
  const nameRef = useRef(newName);
  nameRef.current = newName;

  const uses = useMemo(() => (gesture ? gestureUses(config, gesture.id) : []), [config, gesture]);
  // ShapeRecorder 的冲突提示：同分组里的其他绑定
  const others = useMemo(() => {
    const scopes = new Set(uses.map((u) => u.scope));
    const out: Binding[] = [];
    scopes.forEach((s) => config.scopes[s].bindings.forEach((b) => b.gesture !== gesture?.id && out.push(b)));
    return out;
  }, [config, uses, gesture]);

  if (id === "new") {
    return (
      <section className="editor" aria-label={t("ui.gestures.new")}>
        <header className="editor-head">
          <input className="editor-name" value={newName} placeholder={t("ui.gestures.new_name_ph")} aria-label={t("ui.gestures.name_aria")} onChange={(e) => setNewName(e.target.value)} />
          <button type="button" className="icon-btn" aria-label={t("ui.editor.close_aria")} onClick={onClose}>×</button>
        </header>
        <div className="editor-body">
          <h3 className="editor-section">{t("ui.editor.section_gesture")}</h3>
          <PatternEditor
            pattern={draft}
            others={[]}
            onChange={(p) => {
              // 录完（有笔画/边角/滚轮且笔画完整）才入库，避免空手势进入配置
              if (hasConflictPattern(p) && libraryReady(p)) onCreate(p, nameRef.current.trim());
              else setDraft(p);
            }}
          />
          <p className="field-help">{t("ui.gestures.new_help")}</p>
        </div>
      </section>
    );
  }
  if (!gesture) return null;

  const conflicts = conflictLines(config, gesture.id);
  const pickScope = (scope: number) => {
    setChoosingScope(false);
    onAssignScope(scope);
  };

  return (
    <section className="editor" aria-label={gestureName(gesture)}>
      <header className="editor-head">
        <input className="editor-name" value={gesture.name ?? ""} placeholder={describePattern(gesture.pattern)} aria-label={t("ui.gestures.name_aria")} onChange={(e) => onRename(e.target.value)} />
        <button type="button" className="icon-btn" aria-label={t("ui.editor.close_aria")} onClick={onClose}>×</button>
      </header>
      <div className="editor-body">
        <h3 className="editor-section">{t("ui.editor.section_gesture")}</h3>
        {uses.length > 1 && <p className="share-tag">{t("ui.gestures.shared_note", { count: uses.length })}</p>}
        <PatternEditor key={`g-${gesture.id}`} pattern={gesture.pattern} others={others} onChange={onPattern} />

        <h3 className="editor-section">
          {t("ui.gestures.used_in")}
          {uses.length > 0 && <span className="muted section-hint">{t("ui.gestures.used_in_hint")}</span>}
        </h3>
        {uses.length === 0 ? (
          <p className="field-help">{t("ui.gestures.not_used")}</p>
        ) : (
          <ul className="gesture-uses">
            {uses.map((ref) => {
              const scope = config.scopes[ref.scope];
              const b = scope.bindings[ref.binding];
              const on = !b.disabled || !!b.draft;
              return (
                <li key={`${ref.scope}-${ref.binding}`} className={highlight && b.uiId === highlight ? "just-added" : undefined} ref={highlight && b.uiId === highlight ? (el) => el?.scrollIntoView({ block: "nearest" }) : undefined}>
                  <button type="button" onClick={() => onJump(ref)}>
                    {scope.kind === "app" ? <AppIcon exe={scope.app?.exe} /> : <NavIcon kind={scope.kind === "global" ? "global" : "desktop"} />}
                    <span className="use-scope">{scopeTitle(scope)}</span>
                    <span className="use-name">{displayName(b.name, b.name_key)}</span>
                    <span className="use-action muted">{b.secured ? "••••" : actionKeys(b.action) ? <Caps keys={actionKeys(b.action)!} compact /> : actionKeys(b.action) === "" ? t("ui.action.no_keys") : actionSummary(b.action)}</span>
                    <span className={`use-dot ${on ? "on" : ""}`} data-tip={on ? t("ui.editor.on_state") : t("ui.editor.off_state")} />
                  </button>
                </li>
              );
            })}
          </ul>
        )}
        {conflicts.map((c) => <p key={c} className="field-warn">{c}</p>)}
        <div className="btn-row">
          <button type="button" className="btn" onClick={() => setChoosingScope(true)}>{t("ui.gestures.assign")}</button>
        </div>
      </div>
      <footer className="editor-foot">
        <span className="spacer" />
        <button type="button" className="btn danger" onClick={() => (uses.length ? setConfirmDelete(true) : onDelete("detach"))}>{t("ui.gestures.delete")}</button>
      </footer>

      {choosingScope && (
        <Modal title={t("ui.gestures.assign_pick")} onClose={() => setChoosingScope(false)}>
          <div className="modal-body">
            <ul className="app-list assign-list">
              {config.scopes.map((scope, i) => {
                const used = scope.bindings.some((b) => b.gesture === gesture.id);
                const clash = !used && scope.bindings.some((b) => !b.disabled && hasConflictPattern(b.pattern) && patternKey(b.pattern) === patternKey(gesture.pattern));
                const hint = used ? t("ui.gestures.assign_used") : scope.disabled ? t("ui.gestures.assign_off") : clash ? t("ui.gestures.assign_clash") : "";
                return (
                  <li key={scope.id}>
                    <button type="button" disabled={used} onClick={() => pickScope(i)}>
                      {scope.kind === "app" ? <AppIcon exe={scope.app?.exe} /> : <NavIcon kind={scope.kind === "global" ? "global" : "desktop"} />}
                      <span className="app-name">{scopeTitle(scope)}</span>
                      {hint && <span className={`assign-hint ${clash || scope.disabled ? "warn" : ""}`}>{hint}</span>}
                    </button>
                  </li>
                );
              })}
              <li>
                <button type="button" onClick={() => { setChoosingScope(false); onAddApp(); }}>
                  <NavIcon kind="add" />
                  <span className="app-name">{t("ui.sidebar.add_app")}</span>
                </button>
              </li>
            </ul>
          </div>
          <footer className="modal-foot">
            <button type="button" className="btn" onClick={() => setChoosingScope(false)}>{t("common.cancel")}</button>
          </footer>
        </Modal>
      )}
      {assignScope != null && <ActionPicker onPick={(a) => onAssign(assignScope, a)} onClose={onAssignCancel} />}
      {confirmDelete && (
        <Modal title={t("ui.gestures.delete_title", { name: gestureName(gesture) })} onClose={() => setConfirmDelete(false)}>
          <div className="modal-body">
            <p>{t("ui.gestures.delete_used", { count: uses.length })}</p>
            <ul className="gesture-uses compact">
              {uses.map((ref) => (
                <li key={`${ref.scope}-${ref.binding}`}><span className="use-line">{placeText(config, ref)}</span></li>
              ))}
            </ul>
          </div>
          <footer className="modal-foot btn-row">
            <button type="button" className="btn" onClick={() => setConfirmDelete(false)}>{t("common.cancel")}</button>
            <button type="button" className="btn danger" onClick={() => { setConfirmDelete(false); onDelete("cascade"); }}>{t("ui.gestures.delete_cascade")}</button>
            <button type="button" className="btn primary" onClick={() => { setConfirmDelete(false); onDelete("detach"); }}>{t("ui.gestures.delete_keep")}</button>
          </footer>
        </Modal>
      )}
    </section>
  );
}
