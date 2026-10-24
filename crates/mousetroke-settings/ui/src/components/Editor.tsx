import type { Action, Binding, Step } from "../types";
import { displayName, t, type MsgKey } from "../i18n";
import { presetDefinition } from "../actions";
import { ActionEditor } from "./ActionEditor";
import { PatternEditor } from "./PatternEditor";

export interface EditorProps {
  binding: Binding;
  /** 用于在切换绑定时重置内部状态。 */
  bindingKey: string;
  conflictWith: string[];
  otherBindings: Binding[];
  /** 新建手势首次打开编辑器时自动弹出动作选择器。 */
  autoPickAction?: boolean;
  onChange: (f: (b: Binding) => Binding, coalesce?: string) => void;
  onDuplicate: () => void;
  onDelete: () => void;
  onClose: () => void;
}

/** 冲突名单按语言加引号并连接：zh「a」「b」，en “a”, “b”（引号与分隔符来自词条）。 */
function quoteJoin(names: string[]): string {
  return names.map((n) => `${t("ui.editor.q_open")}${n}${t("ui.editor.q_close")}`).join(t("ui.editor.q_sep"));
}

/** 手势的默认名（新建/新触发角）的当前译文。 */
function defaultBindingName(binding: Binding): string {
  return binding.pattern[0]?.type === "edge" ? t("ui.app.new_edge") : t("ui.app.new_gesture");
}

/** 当前动作是未知预置 ID（不在注册表里）。 */
function isUnknownPreset(b: Binding): boolean {
  return b.action.type === "preset" && !presetDefinition(b.action);
}

export function Editor({ binding, bindingKey, conflictWith, otherBindings, autoPickAction, onChange, onDuplicate, onDelete, onClose }: EditorProps) {
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
        <label className="editor-enable" data-tip={binding.disabled ? t("ui.editor.off_tip") : t("ui.editor.on_tip")}>
          <span>{binding.disabled ? t("ui.editor.off_state") : t("ui.editor.on_state")}</span>
          <input
            type="checkbox"
            className="switch small"
            checked={!binding.disabled}
            onChange={(e) => onChange((b) => ({ ...b, disabled: e.target.checked ? undefined : true }))}
          />
        </label>
        <button type="button" className="icon-btn" data-tip={t("ui.editor.close_tip")} aria-label={t("ui.editor.close_aria")} onClick={onClose}>
          ×
        </button>
      </header>

      <div className="editor-body">
        <h3 className="editor-section">{t("ui.editor.section_gesture")}</h3>
        <PatternEditor
          key={`p-${bindingKey}`}
          pattern={binding.pattern}
          others={otherBindings}
          onChange={(pattern: Step[]) => onChange((b) => ({ ...b, pattern }))}
        />
        {conflict && <p className="field-warn">{conflict}</p>}

        <h3 className="editor-section">{t("ui.editor.section_action")}</h3>
        <ActionEditor
          key={`a-${bindingKey}`}
          action={binding.action}
          autoPick={autoPickAction}
          onChange={(action: Action, c?: string) =>
            onChange((b) => {
              const preset = presetDefinition(action);
              const next: Binding = { ...b, action };
              if (preset) {
                const name = t(preset.name_key as MsgKey);
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
