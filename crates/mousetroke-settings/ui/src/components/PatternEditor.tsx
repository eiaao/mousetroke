import * as React from "react";
import { useEffect, useRef, useState } from "react";
import type { Binding, MouseButton, Step } from "../types";
import { api, errText } from "../api";
import { EDGE, arrows, buttonName, buttonPhrase, codeToVk, compose, decompose, keyTagName, type PatternItem, type PatternModel } from "../gesture";
import { t } from "../i18n";
import { EdgePicker } from "./EdgePicker";
import { Recorder } from "./Recorder";
import { ShapeGlyph, TraceGlyph } from "./Glyph";
import { ShapeRecorder } from "./ShapeRecorder";

/** 触发方式候选（标签按当前语言取词条）。 */
const TRIGGERS: { id: string }[] = [
  { id: "right" },
  { id: "middle" },
  { id: "x1" },
  { id: "x2" },
  { id: "edge" },
];

/** 手势模式编辑：触发方式 + 有序的附加步骤（鼠标键/滚轮/键盘/轨迹）+ 录制画布。 */
export function PatternEditor({ pattern, onChange, conflictHint, others }: { pattern: Step[]; onChange: (p: Step[]) => void; conflictHint?: React.ReactNode; others: Binding[] }) {
  const model = decompose(pattern);
  const set = (m: PatternModel) => onChange(compose(m));
  const strokeIdx = model.items.findIndex((i) => i.kind === "trace" || i.kind === "shape");
  const stroke = model.items[strokeIdx];
  const trace = stroke?.kind === "trace" ? stroke.points : null;
  const [wantTrace, setWantTrace] = useState(false);
  const [lastDrawing, setLastDrawing] = useState<[number, number][] | null>(null);
  const [converting, setConverting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const patternRef = useRef(pattern);
  patternRef.current = pattern;
  const mounted = useRef(true);
  useEffect(() => {
    mounted.current = true;
    return () => { mounted.current = false; };
  }, []);
  const showRecorder = model.trigger.kind === "button" || stroke != null || wantTrace;

  const setTrigger = (id: string) => {
    if (id === "edge") set({ ...model, trigger: { kind: "edge", edge: model.trigger.kind === "edge" ? model.trigger.edge : EDGE.leftBottom } });
    else set({ ...model, trigger: { kind: "button", button: id as MouseButton } });
  };

  const setStroke = (item: Extract<PatternItem, { kind: "trace" | "shape" }>) => {
    const items = [...model.items];
    if (strokeIdx >= 0) items[strokeIdx] = item;
    else items.push(item);
    set({ ...model, items });
  };

  const removeItem = (i: number) => {
    if (model.items[i].kind !== "key") {
      setWantTrace(false);
      setLastDrawing(null);
    }
    set({ ...model, items: model.items.filter((_, j) => j !== i) });
  };

  const addKey = (key: string) => set({ ...model, items: [...model.items, { kind: "key", key }] });

  const triggerId = model.trigger.kind === "edge" ? "edge" : model.trigger.button;

  const switchDrawing = async () => {
    if (!lastDrawing || converting) return;
    const original = pattern;
    const asShape = stroke?.kind !== "shape";
    setConverting(true);
    setError(null);
    try {
      const points = asShape ? await api.shapeSample(lastDrawing) : await api.recordTrace(lastDrawing, 30);
      if (!mounted.current || patternRef.current !== original) return;
      if (points.length < 4) {
        setError(t("ui.recorder.too_short"));
        return;
      }
      // 整个类型转换只提交一次编辑，撤销可恢复全部形状样本。
      setStroke(asShape ? { kind: "shape", templates: [points] } : { kind: "trace", points });
    } catch (e) {
      if (mounted.current) setError(errText(e));
    } finally {
      if (mounted.current) setConverting(false);
    }
  };

  const feedbackLine = lastDrawing && stroke && (
    <div className="recorder-detection" role="status">
      <span>{t(stroke.kind === "shape" ? "ui.recorder.recognized_shape" : "ui.recorder.recognized_line")}</span>
      <button type="button" className="btn text" disabled={converting} onClick={switchDrawing}>
        {t(stroke.kind === "shape" ? stroke.templates.length > 1 ? "ui.recorder.use_line_drop_samples" : "ui.recorder.use_line" : "ui.recorder.use_shape")}
      </button>
    </div>
  );

  const emptyShape = () => {
    setLastDrawing(null);
    setError(null);
    setStroke({ kind: "shape", templates: [] });
  };

  return (
    <div className="pattern-editor">
      <div className="field">
        <label className="field-label">{t("ui.pattern.start_label")}</label>
        <div className="segmented" role="radiogroup">
          {TRIGGERS.map((tr) => (
            <button key={tr.id} type="button" role="radio" aria-checked={triggerId === tr.id} className={triggerId === tr.id ? "on" : ""} onClick={() => setTrigger(tr.id)}>
              {tr.id === "edge" ? t("ui.pattern.edge_trigger") : buttonPhrase(tr.id as MouseButton, "hold")}
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
            hasShape={stroke?.kind === "shape"}
            onTrace={() => {
              setLastDrawing(null);
              setError(null);
              if (stroke?.kind === "shape") setStroke({ kind: "trace", points: [] });
              setWantTrace(true);
            }}
            onShape={emptyShape}
            onKey={addKey}
            exclude={model.trigger.kind === "button" ? model.trigger.button : null}
          />
        </div>
      </div>

      {showRecorder && (
        <div className="field">
          {stroke?.kind === "shape" ? (
            <ShapeRecorder templates={stroke.templates} onChange={(templates, raw) => {
              if (raw) setLastDrawing(raw);
              setError(null);
              setStroke({ kind: "shape", templates });
            }} others={others} feedbackLine={feedbackLine} />
          ) : (
            <>
              <Recorder key="trace" points={trace} disabled={converting} onChange={(points, raw, kind) => {
                setLastDrawing(raw);
                setError(null);
                setStroke(kind === "shape" ? { kind: "shape", templates: [points] } : { kind: "trace", points });
              }} hint={conflictHint} />
              {feedbackLine}
              {trace && trace.length / 2 - 1 > 4 && (
                <div className="recorder-complex field-help">
                  <span>{t("ui.recorder.complex_hint")}</span>
                  <button type="button" className="btn text" onClick={emptyShape}>{t("ui.recorder.redraw_shape")}</button>
                </div>
              )}
            </>
          )}
          {error && <p className="field-warn recorder-error" role="status">{error}</p>}
        </div>
      )}
    </div>
  );
}

function Chip({ item, onRemove }: { item: PatternItem; onRemove: () => void }) {
  return (
    <span className={`chip ${item.kind !== "key" ? "chip-trace" : ""}`}>
      {item.kind === "trace" ? (
        <>
          <TraceGlyph points={item.points} size={18} stroke={1.6} />
          {t("ui.pattern.draw", { arrows: arrows(item.points) })}
        </>
      ) : item.kind === "shape" ? (
        <>
          {item.templates[0] && <ShapeGlyph template={item.templates[0]} size={18} stroke={1.6} />}
          {t("ui.pattern.draw_shape")}
        </>
      ) : item.key.startsWith("mouse:") ? (
        buttonPhrase(item.key.slice(6) as MouseButton, "click")
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
function AddStepMenu({ hasTrace, hasShape, onTrace, onShape, onKey, exclude }: { hasTrace: boolean; hasShape: boolean; onTrace: () => void; onShape: () => void; onKey: (k: string) => void; exclude: MouseButton | null }) {
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
            <button role="menuitem" data-step="trace" onClick={() => (onTrace(), setOpen(false))}>
              {t("ui.pattern.draw_trace")}
            </button>
          )}
          {!hasShape && (
            <button role="menuitem" data-step="shape" onClick={() => (onShape(), setOpen(false))}>
              {t("ui.pattern.draw_shape")}
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
