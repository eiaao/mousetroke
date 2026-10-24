import { isMac, unsupportedAction, supportedAction } from "../platform";
import { useRef } from "react";
import type { Action, AdvancedAction, MenuItem } from "../types";
import { isAdvancedAction, isPresetAction } from "../types";
import { presetDefinition, presetSummary } from "../actions";
import { ACTION_ORDER, SYSTEM_KEYS, WINDOW_OPS, actionName, defaultAction } from "../gesture";
import { t, type MsgKey } from "../i18n";
import { api } from "../api";
import { KeysInput } from "./KeysInput";

const VARS: { v: string; key: "ui.var.selected_text" | "ui.var.clipboard" | "ui.var.app_name" | "ui.var.pid" | "ui.var.win_title" }[] = [
  { v: "%MS_SELECTED_TEXT%", key: "ui.var.selected_text" },
  { v: "%MS_CLIPBOARD_TEXT%", key: "ui.var.clipboard" },
  { v: "%MS_APP_NAME%", key: "ui.var.app_name" },
  { v: "%MS_PID%", key: "ui.var.pid" },
  { v: "%MS_WIN_NAME%", key: "ui.var.win_title" },
];

/**
 * 动作编辑。切换类型时记住各类型上次的内容（切回来不丢），
 * 记忆按 `memoryKey`（通常是绑定标识）隔离。
 */
export function ActionEditor({ action, onChange, depth = 0 }: { action: Action; onChange: (a: Action, coalesce?: string) => void; depth?: number }) {
  const memory = useRef<Partial<Record<AdvancedAction["type"], AdvancedAction>>>({});
  if (!isAdvancedAction(action)) {
    const preset = presetDefinition(action);
    const implementation = preset && (isMac() ? preset.macos : preset.windows);
    return <div className="action-editor">
      <strong>{presetSummary(action)}</strong>
      {isPresetAction(action) && <p><code>{action.id}</code></p>}
      {preset && <p className="field-help">{t(preset.description_key as MsgKey)}</p>}
      {implementation && implementation.status !== "implemented" && <p className="field-warn">{t(implementation.value.reason_key as MsgKey)}</p>}
      <p className="field-help">{t("preset.ui.read_only")}</p>
      <textarea readOnly rows={5} aria-label={t("preset.ui.read_only")} value={JSON.stringify(action, null, 2)} />
    </div>;
  }
  const switchTo = (tp: AdvancedAction["type"]) => {
    if (tp === action.type) return;
    memory.current[action.type] = action;
    onChange(memory.current[tp] ?? defaultAction(tp));
  };
  return (
    <div className="action-editor">
      <div className={`action-types ${depth ? "compact" : ""}`} role="radiogroup" aria-label={t("ui.action.type_aria")}>
        {ACTION_ORDER.filter((tp) => !(depth && tp === "menu")).map((tp) => (
          <button key={tp} type="button" role="radio" aria-checked={action.type === tp} className={action.type === tp ? "on" : ""} onClick={() => switchTo(tp)}>
            {actionName(tp)}
          </button>
        ))}
      </div>
      {unsupportedAction(action) && <div className="field-warn" role="alert">
        <p>{t("mac.ui.action.unsupported")}</p>
        <button type="button" className="btn small" onClick={() => onChange(supportedAction(action))}>{t("mac.ui.action.repair")}</button>
      </div>}
      <ActionFields action={action} onChange={onChange} depth={depth} />
    </div>
  );
}

function VarChips({ onInsert }: { onInsert: (v: string) => void }) {
  return (
    <div className="var-chips">
      <span className="muted">{t("ui.action.insert")}</span>
      {VARS.map((x) => (
        <button key={x.v} type="button" className="chip chip-var" data-tip={x.v} onClick={() => onInsert(x.v)}>
          {t(x.key)}
        </button>
      ))}
    </div>
  );
}

/** 在光标处插入文本（无光标则追加）。 */
function insertAt(el: HTMLInputElement | HTMLTextAreaElement | null, value: string, ins: string): string {
  if (!el || el.selectionStart == null) return value + ins;
  const s = el.selectionStart, e = el.selectionEnd ?? s;
  requestAnimationFrame(() => {
    el.focus();
    el.setSelectionRange(s + ins.length, s + ins.length);
  });
  return value.slice(0, s) + ins + value.slice(e);
}

function ActionFields({ action, onChange, depth }: { action: AdvancedAction; onChange: (a: Action, coalesce?: string) => void; depth: number }) {
  const textRef = useRef<HTMLTextAreaElement & HTMLInputElement>(null);
  switch (action.type) {
    case "key_sequence":
      return (
        <div className="fields">
          <KeysInput value={action.keys} onChange={(keys) => onChange({ ...action, keys }, "keys")} />
          <p className="field-help">
            <button type="button" className="link" onClick={() => api.openPath("key_sequences")}>{t("ui.keys.syntax_help")}</button>
          </p>
          <label className="check">
            <input type="checkbox" checked={!!action.activate_target} onChange={(e) => onChange({ ...action, activate_target: e.target.checked })} />
            {t("ui.action.activate_first")}
          </label>
        </div>
      );
    case "window_op":
      return (
        <div className="fields">
          <div className="segmented wrap" role="radiogroup">
            {WINDOW_OPS.map((w) => (
              <button key={w.op} disabled={isMac() && w.op === "toggle_topmost"} data-tip={isMac() && w.op === "toggle_topmost" ? t("mac.action.topmost_unsupported") : undefined} type="button" role="radio" aria-checked={action.op === w.op} className={action.op === w.op ? "on" : ""} onClick={() => onChange({ ...action, op: w.op as never })}>
                {t(w.key)}
              </button>
            ))}
          </div>
          <p className="field-help">{t("ui.action.window_op_help")}</p>
        </div>
      );
    case "system_key":
      return (
        <div className="fields">
          <div className="segmented wrap" role="radiogroup">
            {SYSTEM_KEYS.map((k) => (
              <button key={k.key} type="button" role="radio" aria-checked={action.key.toUpperCase() === k.key} className={action.key.toUpperCase() === k.key ? "on" : ""} onClick={() => onChange({ ...action, key: k.key })}>
                {t(k.i18n)}
              </button>
            ))}
          </div>
        </div>
      );
    case "launch_app":
      return (
        <div className="fields">
          <div className="field">
            <label className="field-label">{t("ui.action.exe_label")}</label>
            <div className="input-row">
              <input value={action.exe} spellCheck={false} placeholder={isMac() ? t("mac.ui.action.app_ph") : "C:\Program Files\…\app.exe"} onChange={(e) => onChange({ ...action, exe: e.target.value }, "exe")} />
              <button
                type="button"
                className="btn small"
                onClick={async () => {
                  const p = await api.pickFile("exe", false, t("ui.action.pick_exe"));
                  if (p) onChange({ ...action, exe: p });
                }}
              >
                {t("common.browse")}
              </button>
            </div>
          </div>
          <div className="field">
            <label className="field-label">{t("ui.action.args_label")}</label>
            <input value={action.args ?? ""} spellCheck={false} placeholder={t("ui.action.optional")} onChange={(e) => onChange({ ...action, args: e.target.value || null }, "args")} />
          </div>
          <div className="field">
            <label className="field-label">{t("ui.action.if_running")}</label>
            <div className="segmented" role="radiogroup">
              {(
                [
                  ["activate", "ui.action.activate"],
                  ["run_again", "ui.action.run_again"],
                ] as const
              ).map(([v, k]) => (
                <button key={v} type="button" role="radio" aria-checked={(action.if_running ?? "activate") === v} className={(action.if_running ?? "activate") === v ? "on" : ""} onClick={() => onChange({ ...action, if_running: v })}>
                  {t(k)}
                </button>
              ))}
            </div>
          </div>
          {!isMac() && <label className="check">
            <input type="checkbox" checked={!!action.elevate} onChange={(e) => onChange({ ...action, elevate: e.target.checked })} />
            {t("ui.action.elevate")}
          </label>}
        </div>
      );
    case "open_url":
      return (
        <div className="fields">
          <div className="field">
            <label className="field-label">{t("ui.action.url_label")}</label>
            <input ref={textRef} value={action.url_template} spellCheck={false} onChange={(e) => onChange({ ...action, url_template: e.target.value }, "url")} />
            <VarChips onInsert={(v) => onChange({ ...action, url_template: insertAt(textRef.current, action.url_template, v) })} />
            <p className="field-help">
              {t("ui.action.url_help")}{" "}
              <button type="button" className="link" onClick={() => api.openPath("variables")}>{t("ui.action.variables_docs")}</button>
            </p>
          </div>
          <div className="field">
            <label className="field-label">{t("ui.action.browser_label")}</label>
            <input value={action.browser ?? ""} spellCheck={false} placeholder={t("ui.action.browser_ph")} onChange={(e) => onChange({ ...action, browser: e.target.value || null }, "browser")} />
          </div>
        </div>
      );
    case "shell":
      return (
        <div className="fields">
          <textarea
            ref={textRef}
            className="code"
            rows={4}
            spellCheck={false}
            value={action.script}
            placeholder={isMac() ? t("mac.ui.action.shell_ph") : "taskkill /F /PID %MS_PID%"}
            onChange={(e) => onChange({ ...action, script: e.target.value }, "script")}
          />
          <VarChips onInsert={(v) => onChange({ ...action, script: insertAt(textRef.current, action.script, v) })} />
          <p className="field-help">
            {t("ui.action.shell_help")}{" "}
            <button type="button" className="link" onClick={() => api.openPath("variables")}>{t("ui.action.variables_docs")}</button>
          </p>
        </div>
      );
    case "lua":
      return (
        <div className="fields">
          <textarea
            className="code"
            rows={10}
            spellCheck={false}
            value={action.code}
            onKeyDown={(e) => {
              if (e.key === "Tab" && !e.shiftKey) {
                e.preventDefault();
                const el = e.currentTarget;
                onChange({ ...action, code: insertAt(el, action.code, "  ") }, "lua");
              }
            }}
            onChange={(e) => onChange({ ...action, code: e.target.value }, "lua")}
          />
          <p className="field-help">
            {t("ui.action.lua_help")}{" "}
            <button type="button" className="link" onClick={() => api.openPath("lua_basics")}>{t("ui.action.lua_docs")}</button>
            {" · "}
            <button type="button" className="link" onClick={() => api.openPath("lua_api")}>{t("ui.action.lua_api_docs")}</button>
            {" · "}
            <button type="button" className="link" onClick={() => api.openPath("lua_examples")}>{t("ui.action.lua_examples_docs")}</button>
          </p>
        </div>
      );
    case "menu":
      return <MenuFields action={action} onChange={onChange} depth={depth} />;
  }
}

function MenuFields({ action, onChange, depth }: { action: Extract<Action, { type: "menu" }>; onChange: (a: Action, c?: string) => void; depth: number }) {
  const setItems = (items: MenuItem[], c?: string) => onChange({ ...action, items }, c);
  const move = (i: number, d: number) => {
    const items = [...action.items];
    const [x] = items.splice(i, 1);
    items.splice(i + d, 0, x);
    setItems(items);
  };
  return (
    <div className="fields menu-fields">
      {action.items.length === 0 && <p className="muted">{t("ui.action.menu_empty")}</p>}
      {action.items.map((it, i) => (
        <details key={i} className="menu-item" open={action.items.length <= 2}>
          <summary>
            <input
              value={it.label}
              placeholder={t("ui.action.menu_item_ph")}
              onClick={(e) => e.preventDefault()}
              onKeyDown={(e) => e.key === " " && e.stopPropagation()}
              onChange={(e) => setItems(action.items.map((x, j) => (j === i ? { ...x, label: e.target.value } : x)), `menu-label-${i}`)}
            />
            <span className="menu-item-tools">
              <button type="button" className="icon-btn" aria-label={t("ui.action.move_up")} disabled={i === 0} onClick={(e) => (e.preventDefault(), move(i, -1))}>
                ↑
              </button>
              <button type="button" className="icon-btn" aria-label={t("ui.action.move_down")} disabled={i === action.items.length - 1} onClick={(e) => (e.preventDefault(), move(i, 1))}>
                ↓
              </button>
              <button type="button" className="icon-btn danger" aria-label={t("ui.action.delete_item")} onClick={(e) => (e.preventDefault(), setItems(action.items.filter((_, j) => j !== i)))}>
                ×
              </button>
            </span>
          </summary>
          <ActionEditor
            action={it.action}
            depth={depth + 1}
            onChange={(a, c) => setItems(action.items.map((x, j) => (j === i ? { ...x, action: a } : x)), c && `menu-${i}-${c}`)}
          />
        </details>
      ))}
      <button type="button" className="btn small" onClick={() => setItems([...action.items, { label: t("ui.action.menu_item", { n: action.items.length + 1 }), action: defaultAction("key_sequence") }])}>
        {t("ui.action.add_menu_item")}
      </button>
    </div>
  );
}
