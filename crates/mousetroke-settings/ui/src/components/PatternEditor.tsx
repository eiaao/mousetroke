import * as React from "react";
import { useEffect, useRef, useState } from "react";
import type { MouseButton, Step } from "../types";
import { EDGE, arrows, buttonName, codeToVk, compose, decompose, keyTagName, type PatternItem, type PatternModel } from "../gesture";
import { t } from "../i18n";
import { EdgePicker } from "./EdgePicker";
import { Recorder } from "./Recorder";
import { TraceGlyph } from "./Glyph";

/** 触发方式候选（标签按当前语言取词条）。 */
const TRIGGERS: { id: string }[] = [
  { id: "right" },
  { id: "middle" },
  { id: "x1" },
  { id: "x2" },
  { id: "edge" },
];

/** 手势模式编辑：触发方式 + 有序的附加步骤（鼠标键/滚轮/键盘/轨迹）+ 录制画布。 */
export function PatternEditor({ pattern, onChange, conflictHint }: { pattern: Step[]; onChange: (p: Step[]) => void; conflictHint?: React.ReactNode }) {
  const model = decompose(pattern);
  const set = (m: PatternModel) => onChange(compose(m));
  const traceIdx = model.items.findIndex((i) => i.kind === "trace");
  const trace = traceIdx >= 0 ? (model.items[traceIdx] as { kind: "trace"; points: number[] }).points : null;
  const [wantTrace, setWantTrace] = useState(false);
  const showRecorder = model.trigger.kind === "button" || trace !== null || wantTrace;

  const setTrigger = (id: string) => {
    if (id === "edge") set({ ...model, trigger: { kind: "edge", edge: model.trigger.kind === "edge" ? model.trigger.edge : EDGE.leftBottom } });
    else set({ ...model, trigger: { kind: "button", button: id as MouseButton } });
  };

  const setTrace = (points: number[]) => {
    const items = [...model.items];
    if (traceIdx >= 0) items[traceIdx] = { kind: "trace", points };
    else items.push({ kind: "trace", points });
    set({ ...model, items });
  };

  const removeItem = (i: number) => {
    if (model.items[i].kind === "trace") setWantTrace(false);
    set({ ...model, items: model.items.filter((_, j) => j !== i) });
  };

  const addKey = (key: string) => set({ ...model, items: [...model.items, { kind: "key", key }] });

  const triggerId = model.trigger.kind === "edge" ? "edge" : model.trigger.button;

  return (
    <div className="pattern-editor">
      <div className="field">
        <label className="field-label">{t("ui.pattern.start_label")}</label>
        <div className="segmented" role="radiogroup">
          {TRIGGERS.map((tr) => (
            <button key={tr.id} type="button" role="radio" aria-checked={triggerId === tr.id} className={triggerId === tr.id ? "on" : ""} onClick={() => setTrigger(tr.id)}>
              {tr.id === "edge" ? t("ui.pattern.edge_trigger") : t("ui.pattern.hold", { button: buttonName(tr.id as MouseButton) })}
            </button>
          ))}
        </div>
        {model.trigger.kind === "edge" && (
          <>
            <EdgePicker value={model.trigger.edge} onChange={(edge) => set({ ...model, trigger: { kind: "edge", edge } })} />
            <p className="field-help">{t("ui.pattern.edge_help")}</p>
          </>
        )}
      </div>

      <div className="field">
        <label className="field-label">{t("ui.pattern.then_label")}</label>
        <div className="chips">
          {model.items.length === 0 && <span className="muted chips-empty">{model.trigger.kind === "button" ? t("ui.pattern.draw_below") : t("ui.pattern.none_execute")}</span>}
          {model.items.map((it, i) => (
            <Chip key={i} item={it} onRemove={() => removeItem(i)} />
          ))}
          <AddStepMenu
            hasTrace={trace !== null}
            onTrace={() => setWantTrace(true)}
            onKey={addKey}
            exclude={model.trigger.kind === "button" ? model.trigger.button : null}
          />
        </div>
      </div>

      {showRecorder && (
        <div className="field">
          <Recorder points={trace} onChange={setTrace} hint={conflictHint} />
        </div>
      )}
    </div>
  );
}

function Chip({ item, onRemove }: { item: PatternItem; onRemove: () => void }) {
  return (
    <span className={`chip ${item.kind === "trace" ? "chip-trace" : ""}`}>
      {item.kind === "trace" ? (
        <>
          <TraceGlyph points={item.points} size={18} stroke={1.6} />
          {t("ui.pattern.draw", { arrows: arrows(item.points) })}
        </>
      ) : item.key.startsWith("mouse:") ? (
        t("ui.pattern.click", { key: keyTagName(item.key) })
      ) : (
        keyTagName(item.key)
      )}
      <button type="button" className="chip-x" aria-label={t("ui.pattern.remove_step")} onClick={onRemove}>
        ×
      </button>
    </span>
  );
}

/** "添加一步"弹出菜单。键盘键通过"按下任意键"捕获。 */
function AddStepMenu({ hasTrace, onTrace, onKey, exclude }: { hasTrace: boolean; onTrace: () => void; onKey: (k: string) => void; exclude: MouseButton | null }) {
  const [open, setOpen] = useState(false);
  const [capturing, setCapturing] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const close = (e: MouseEvent) => !ref.current?.contains(e.target as Node) && (setOpen(false), setCapturing(false));
    // Esc 只关闭弹出菜单（捕获阶段拦截，不让它继续关掉编辑面板）；正在捕获按键时 Esc 也算一个可选的键
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape" && !capturing) {
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
  }, [open, capturing]);

  const pick = (k: string) => {
    onKey(k);
    setOpen(false);
  };

  const buttons: MouseButton[] = ["left", "right", "middle", "x1", "x2"];

  return (
    <div className="add-step" ref={ref}>
      <button type="button" className="chip chip-add" aria-expanded={open} onClick={() => setOpen((o) => !o)}>
        {t("ui.pattern.add_step")}
      </button>
      {open && (
        <div className="popover" role="menu">
          {!hasTrace && (
            <button role="menuitem" onClick={() => (onTrace(), setOpen(false))}>
              {t("ui.pattern.draw_trace")}
            </button>
          )}
          <div className="popover-group">{t("ui.pattern.click_mouse")}</div>
          <div className="popover-row">
            {buttons
              .filter((b) => b !== exclude)
              .map((b) => (
                <button key={b} role="menuitem" onClick={() => pick(`mouse:${b}`)}>
                  {buttonName(b)}
                </button>
              ))}
          </div>
          <div className="popover-group">{t("ui.pattern.scroll_wheel")}</div>
          <div className="popover-row">
            <button role="menuitem" onClick={() => pick("wheel:-1")}>
              {t("ui.pattern.wheel_up")}
            </button>
            <button role="menuitem" onClick={() => pick("wheel:1")}>
              {t("ui.pattern.wheel_down")}
            </button>
          </div>
          <button
            role="menuitem"
            className={capturing ? "capturing" : ""}
            onClick={() => setCapturing(true)}
            onKeyDown={(e) => {
              if (!capturing) return;
              e.preventDefault();
              const vk = codeToVk(e);
              if (vk) {
                pick(`kb:${vk}`);
                setCapturing(false);
              }
            }}
          >
            {capturing ? t("ui.pattern.press_key_hint") : t("ui.pattern.hold_key")}
          </button>
        </div>
      )}
    </div>
  );
}
