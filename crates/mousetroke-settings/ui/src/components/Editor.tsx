import { useEffect, useRef, useState } from "react";
import type { Action, Binding, Gesture, Step } from "../types";
import { displayName, t } from "../i18n";
import { presetDefinition, presetName } from "../actions";
import { gestureName, quoteJoin } from "../gesture";
import { ActionEditor } from "./ActionEditor";
import { PatternEditor } from "./PatternEditor";
import { GestureGlyph } from "./GestureLibrary";
import { NavIcon } from "./Icons";

/** 绑定引用的库手势及其共享情况（由上层从配置算出）。 */
export interface SharedGesture {
  gesture: Gesture;
  /** 引用它的绑定总数（含当前）。 */
  uses: number;
  /** 其他引用处的位置文字（「所有程序」「后退」…）。 */
  places: string[];
  /** 当前修改在其他分组造成的冲突。 */
  conflicts: string[];
}

export interface EditorProps {
  binding: Binding;
  /** 用于在切换绑定时重置内部状态。 */
  bindingKey: string;
  conflictWith: string[];
  otherBindings: Binding[];
  /** 新建手势首次打开编辑器时自动弹出动作选择器。 */
  autoPickAction?: boolean;
  onPickerShown?: () => void;
  onChange: (f: (b: Binding) => Binding, coalesce?: string) => void;
  /** 改画法的唯一入口：引用库手势时同步所有引用处。 */
  onPatternChange: (pattern: Step[]) => void;
  shared?: SharedGesture;
  /** 能否加入手势库（草稿和未录完的不能）。 */
  canAdopt: boolean;
  onOpenPicker: () => void;
  onPromote: () => void;
  onUseElsewhere: () => void;
  onShowInLibrary: (id: string) => void;
  /** 「只改这里」：把本次会话的修改收回到当前绑定，库与其他引用处恢复为 before。 */
  onKeepHere: (before: Step[]) => void;
  onDuplicate: () => void;
  onDelete: () => void;
  onClose: () => void;
}

/** 共享手势的编辑会话：进入绑定后第一次改动前的画法；离开或点「改好了」即结束。 */
interface Session {
  key: string;
  gesture: string;
  before: Step[];
  modified: boolean;
}

/** 手势的默认名（新建/新触发角）的当前译文。 */
function defaultBindingName(binding: Binding): string {
  return binding.pattern[0]?.type === "edge" ? t("ui.app.new_edge") : t("ui.app.new_gesture");
}

/** 当前动作是未知预置 ID（不在注册表里）。 */
function isUnknownPreset(b: Binding): boolean {
  return b.action.type === "preset" && !presetDefinition(b.action);
}

export function Editor({ binding, bindingKey, conflictWith, otherBindings, autoPickAction, onPickerShown, onChange, onPatternChange, shared, canAdopt, onOpenPicker, onPromote, onUseElsewhere, onShowInLibrary, onKeepHere, onDuplicate, onDelete, onClose }: EditorProps) {
  // 草稿保留启用意图，用户仍可在完成前手动停用。
  const enabled = !binding.disabled || !!binding.draft;
  const conflict = conflictWith.length ? (
    <span className="warn">{t("ui.editor.conflict", { names: quoteJoin(conflictWith.map((n) => n)) })}</span>
  ) : null;
  return (
    <section className="editor" aria-label={t("ui.editor.aria")} key={bindingKey}>
      <header className="editor-head">
        <input
          className="editor-name"
          value={displayName(binding.name, binding.name_key)}
          aria-label={t("ui.editor.name_aria")}
          placeholder={t("ui.editor.name_ph")}
          onChange={(e) => onChange((b) => ({ ...b, name: e.target.value, name_key: undefined }), "name")}
        />
        <label className="editor-enable" data-tip={enabled ? t("ui.editor.on_tip") : t("ui.editor.off_tip")}>
          <span>{enabled ? t("ui.editor.on_state") : t("ui.editor.off_state")}</span>
          <input
            type="checkbox"
            className="switch small"
            checked={enabled}
            onChange={(e) => onChange((b) => ({ ...b, disabled: e.target.checked ? undefined : true, draft: undefined }))}
          />
        </label>
        <button type="button" className="icon-btn" data-tip={t("ui.editor.close_tip")} aria-label={t("ui.editor.close_aria")} onClick={onClose}>
          ×
        </button>
      </header>

      <div className="editor-body">
        <h3 className="editor-section">{t("ui.editor.section_gesture")}</h3>
        <GestureSection
          binding={binding}
          bindingKey={bindingKey}
          otherBindings={otherBindings}
          shared={shared}
          canAdopt={canAdopt}
          onPatternChange={onPatternChange}
          onOpenPicker={onOpenPicker}
          onPromote={onPromote}
          onUseElsewhere={onUseElsewhere}
          onShowInLibrary={onShowInLibrary}
          onKeepHere={onKeepHere}
        />
        {binding.draft && <p className="field-warn" role="status">{t("ui.editor.incomplete_pattern")}</p>}
        {conflict && <p className="field-warn">{conflict}</p>}

        <h3 className="editor-section">{t("ui.editor.section_action")}</h3>
        <ActionEditor
          key={`a-${bindingKey}`}
          action={binding.action}
          autoPick={autoPickAction}
          unselected={!!binding.actionUnselected}
          onPickerShown={onPickerShown}
          onChange={(action: Action, c?: string) =>
            onChange((b) => {
              const preset = presetDefinition(action);
              const next: Binding = { ...b, action, actionUnselected: undefined };
              if (preset) {
                const name = presetName(preset);
                // 空名、默认名、或等于上一次自动填入的动作名时，跟随新动作名；手动改过的名字不动
                const auto = !b.name || b.name === defaultBindingName(b) || b.name === b.lastActionName;
                if (auto) {
                  next.name = name;
                  next.lastActionName = name;
                } else {
                  next.lastActionName = undefined;
                }
              } else if (isUnknownPreset(b)) {
                next.lastActionName = undefined;
              }
              return next;
            }, c && `action-${c}`)
          }
        />

        <h3 className="editor-section">{t("ui.editor.section_options")}</h3>
        <div className="fields">
          {binding.pattern.some((s) => s.type === "shape") ? (
            <p className="field-help">{t("ui.shape.release")}</p>
          ) : (
            <label className="check">
              <input type="checkbox" checked={binding.execute_on_match} onChange={(e) => onChange((b) => ({ ...b, execute_on_match: e.target.checked }))} />
              <span>
                {t("ui.editor.execute_now")}
                <small>{t("ui.editor.execute_now_help")}</small>
              </span>
            </label>
          )}
          <label className="check">
            <input type="checkbox" checked={binding.hide_label} onChange={(e) => onChange((b) => ({ ...b, hide_label: e.target.checked }))} />
            <span>{t("ui.editor.hide_label")}</span>
          </label>
        </div>
      </div>

      <footer className="editor-foot">
        <button type="button" className="btn" onClick={onDuplicate}>
          {t("ui.editor.duplicate")}
        </button>
        <button type="button" className="btn danger" onClick={onDelete}>
          {t("common.delete")}
        </button>
      </footer>
    </section>
  );
}

/** 专用手势的更多操作：加入手势库 / 在其他程序中使用（草稿不可用）。 */
function MoreMenu({ canAdopt, onPromote, onUseElsewhere }: { canAdopt: boolean; onPromote: () => void; onUseElsewhere: () => void }) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!open) return;
    const close = (e: MouseEvent) => !ref.current?.contains(e.target as Node) && setOpen(false);
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.stopPropagation();
        setOpen(false);
      }
    };
    window.addEventListener("mousedown", close);
    window.addEventListener("keydown", onKey, true);
    return () => {
      window.removeEventListener("mousedown", close);
      window.removeEventListener("keydown", onKey, true);
    };
  }, [open]);
  const run = (f: () => void) => () => {
    setOpen(false);
    f();
  };
  return (
    <div className="more-menu" ref={ref}>
      <button type="button" className="icon-btn" aria-label={t("ui.gestures.more")} aria-expanded={open} data-tip={t("ui.gestures.more")} onClick={() => setOpen((o) => !o)}>⋯</button>
      {open && (
        <div className="popover" role="menu">
          {!canAdopt && <div className="popover-group">{t("ui.gestures.draft_hint")}</div>}
          <button role="menuitem" disabled={!canAdopt} onClick={run(onPromote)}>{t("ui.gestures.add_to_library")}</button>
          <button role="menuitem" disabled={!canAdopt} onClick={run(onUseElsewhere)}>{t("ui.gestures.assign")}</button>
        </div>
      )}
    </div>
  );
}

/**
 * 手势段：库手势卡片 / 专用手势操作 + 共享编辑会话 + 画法编辑器。
 * 共享手势"先改后决定"：改动立即同步到所有引用处并显示影响范围，离开前可「只改这里」收回到当前绑定。
 */
function GestureSection({
  binding,
  bindingKey,
  otherBindings,
  shared,
  canAdopt,
  onPatternChange,
  onOpenPicker,
  onPromote,
  onUseElsewhere,
  onShowInLibrary,
  onKeepHere,
}: Pick<EditorProps, "binding" | "bindingKey" | "otherBindings" | "shared" | "canAdopt" | "onPatternChange" | "onOpenPicker" | "onPromote" | "onUseElsewhere" | "onShowInLibrary" | "onKeepHere">) {
  const [stored, setSession] = useState<Session | null>(null);
  const gid = shared?.gesture.id;
  // 切换绑定、更换手势后会话自动失效；撤销「只改这里」后引用恢复，会话随之恢复
  const session = stored && stored.key === bindingKey && stored.gesture === gid ? stored : null;
  const multi = !!shared && shared.uses > 1;
  const changePattern = (pattern: Step[]) => {
    if (multi && gid && !session?.modified) setSession({ key: bindingKey, gesture: gid, before: binding.pattern, modified: true });
    onPatternChange(pattern);
  };
  const barRef = useRef<HTMLDivElement>(null);
  const modified = multi && !!session?.modified;
  useEffect(() => {
    if (modified) barRef.current?.animate?.([{ boxShadow: "0 0 0 4px color-mix(in srgb, var(--hit) 40%, transparent)" }, { boxShadow: "0 0 0 0 transparent" }], { duration: 600, easing: "ease-out" });
  }, [modified]);
  return (
    <>
    {shared ? (
      <div className="gesture-ref">
        <GestureGlyph pattern={shared.gesture.pattern} size={34} />
        <span className="gesture-ref-main">
          <span className="gesture-ref-name">{gestureName(shared.gesture)}</span>
          <span className="gesture-ref-sub">
            {t("ui.gestures.library_tag")}
            {" · "}
            {multi ? (
              <button type="button" className="link" onClick={() => onShowInLibrary(shared.gesture.id)}>{t("ui.gestures.shared_uses", { count: shared.uses })}</button>
            ) : (
              t("ui.gestures.only_here_tag")
            )}
          </span>
        </span>
        <button type="button" className="btn small" onClick={onOpenPicker}>{t("ui.gestures.change")}</button>
        <button type="button" className="icon-btn" data-tip={t("ui.gestures.view_in_library")} aria-label={t("ui.gestures.view_in_library")} onClick={() => onShowInLibrary(shared.gesture.id)}>↗</button>
      </div>
    ) : (
      <div className="gesture-inline">
        <NavIcon kind="gestures" />
        <span className="gesture-inline-text">
          {t("ui.gestures.inline")}
          <small>{t("ui.gestures.inline_hint")}</small>
        </span>
        <button type="button" className="btn small" onClick={onOpenPicker}>{t("ui.gestures.pick_from_library")}</button>
        <MoreMenu canAdopt={canAdopt} onPromote={onPromote} onUseElsewhere={onUseElsewhere} />
      </div>
    )}
    {multi && !modified && <p className="share-tag">{t("ui.gestures.shared_list", { list: shared.places.join(t("ui.gestures.list_sep")) })}</p>}
    {multi && modified && session && (
      <div className="share-bar" role="status" ref={barRef}>
        <p className="share-bar-title">{t("ui.gestures.synced", { count: shared.uses })}</p>
        <ul>{shared.places.map((p) => <li key={p}>{p}</li>)}</ul>
        {shared.conflicts.map((c) => <p key={c} className="share-bar-conflict">{c}</p>)}
        <div className="btn-row">
          <button type="button" className="btn small" onClick={() => onKeepHere(session.before)}>{t("ui.gestures.only_here")}</button>
          <button type="button" className="btn small" onClick={() => setSession({ key: bindingKey, gesture: shared.gesture.id, before: binding.pattern, modified: false })}>{t("ui.gestures.done")}</button>
        </div>
      </div>
    )}
    <PatternEditor
      key={`p-${bindingKey}`}
      pattern={binding.pattern}
      others={otherBindings}
      onChange={changePattern}
    />
    </>
  );
}
