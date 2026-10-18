import { useEffect, useRef, useState } from "react";
import { api, errText } from "../api";
import { arrows } from "../gesture";
import { t } from "../i18n";
import { Arrow, tracePath } from "./Glyph";

const GRID = 30; // 与引擎默认最小识别距离一致：画布上一格 = 30px

/**
 * 手势录制画布。用左键或右键按住拖动画出手势，松开后由引擎同一套算法
 * 规整为标准轨迹或形状样本并以动画展示（两种模式都由 Rust 核心处理）。
 */
export function Recorder({ points, onChange, hint, mode = "trace", disabled = false }: { points: number[] | null; onChange: (p: number[]) => void; hint?: React.ReactNode; mode?: "trace" | "shape"; disabled?: boolean }) {
  const wrap = useRef<HTMLDivElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const raw = useRef<[number, number][]>([]);
  const [drawing, setDrawing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const pointer = useRef<number | null>(null);
  const mounted = useRef(true);
  const onChangeRef = useRef(onChange);
  onChangeRef.current = onChange;
  const [animKey, setAnimKey] = useState(0);
  const [size, setSize] = useState({ w: 400, h: 200 });

  useEffect(() => {
    mounted.current = true;
    return () => { mounted.current = false; };
  }, []);

  useEffect(() => {
    const el = wrap.current;
    if (!el) return;
    const ro = new ResizeObserver(([e]) => setSize({ w: e.contentRect.width, h: e.contentRect.height }));
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  // 画布像素与 CSS 尺寸、DPI 对齐
  useEffect(() => {
    const c = canvas.current;
    if (!c) return;
    const dpr = window.devicePixelRatio || 1;
    c.width = Math.round(size.w * dpr);
    c.height = Math.round(size.h * dpr);
    c.getContext("2d")?.setTransform(dpr, 0, 0, dpr, 0, 0);
  }, [size]);

  const clearCanvas = () => {
    const c = canvas.current;
    c?.getContext("2d")?.clearRect(0, 0, c.width, c.height);
  };

  const local = (e: React.PointerEvent): [number, number] => {
    const r = canvas.current!.getBoundingClientRect();
    return [e.clientX - r.left, e.clientY - r.top];
  };

  const onDown = (e: React.PointerEvent) => {
    if (disabled || busy || pointer.current !== null || (e.button !== 0 && e.button !== 2)) return;
    e.preventDefault();
    canvas.current!.setPointerCapture(e.pointerId);
    pointer.current = e.pointerId;
    raw.current = [local(e)];
    clearCanvas();
    setError(null);
    setDrawing(true);
  };

  const onMove = (e: React.PointerEvent) => {
    if (pointer.current !== e.pointerId) return;
    const p = local(e);
    const pts = raw.current;
    const last = pts[pts.length - 1];
    pts.push(p);
    const ctx = canvas.current!.getContext("2d")!;
    ctx.strokeStyle = getComputedStyle(canvas.current!).getPropertyValue("--trail").trim() || "#0098d9";
    ctx.lineWidth = 3;
    ctx.lineCap = "round";
    ctx.lineJoin = "round";
    ctx.beginPath();
    ctx.moveTo(last[0], last[1]);
    ctx.lineTo(p[0], p[1]);
    ctx.stroke();
  };

  const onUp = async (e: React.PointerEvent) => {
    if (pointer.current !== e.pointerId) return;
    pointer.current = null;
    setDrawing(false);
    canvas.current!.releasePointerCapture(e.pointerId);
    setBusy(true);
    try {
      const recorded = mode === "shape" ? await api.shapeSample([...raw.current, local(e)]) : await api.recordTrace(raw.current, GRID);
      if (!mounted.current) return;
      if (recorded.length < 4) {
        setError(t("ui.recorder.too_short"));
        return;
      }
      setAnimKey((k) => k + 1);
      onChangeRef.current(recorded);
    } catch (e) {
      if (mounted.current) setError(errText(e));
    } finally {
      if (mounted.current) {
        clearCanvas();
        setBusy(false);
      }
    }
  };

  const onCancel = (e: React.PointerEvent) => {
    if (pointer.current !== e.pointerId) return;
    pointer.current = null;
    setDrawing(false);
    clearCanvas();
  };

  // 规整后的标准轨迹（SVG，绘制动画）
  const shown = !drawing && points && points.length >= 4;
  const box = Math.min(size.w, size.h);
  const pts = shown ? tracePath(points!, box, box * 0.22, mode === "shape") : [];
  const off = { x: (size.w - box) / 2, y: 0 };
  const len = pts.reduce((acc, p, i) => (i ? acc + Math.hypot(p[0] - pts[i - 1][0], p[1] - pts[i - 1][1]) : 0), 0);

  return (
    <div className="recorder">
      <div
        ref={wrap}
        className={`recorder-pad ${drawing ? "drawing" : ""} ${disabled || busy ? "disabled" : ""}`}
        style={{ "--grid": `${GRID}px` } as React.CSSProperties}
        onContextMenu={(e) => e.preventDefault()}
      >
        <canvas ref={canvas} aria-label={t(mode === "shape" ? "ui.pattern.draw_shape" : "ui.pattern.draw_trace")} aria-disabled={disabled || busy} onPointerDown={onDown} onPointerMove={onMove} onPointerUp={onUp} onPointerCancel={onCancel} />
        {shown && (
          <svg key={animKey} className="recorder-result" width={size.w} height={size.h} aria-hidden>
            <g transform={`translate(${off.x},${off.y})`}>
              <polyline
                className="glyph-line draw-in"
                points={pts.map((p) => p.join(",")).join(" ")}
                strokeWidth={4}
                style={{ strokeDasharray: len, strokeDashoffset: len } as React.CSSProperties}
              />
              <circle className="glyph-start" cx={pts[0][0]} cy={pts[0][1]} r={5} />
              <g className="head-in">
                <Arrow pts={pts} w={4} />
              </g>
            </g>
          </svg>
        )}
        {!shown && !drawing && (
          <div className="recorder-empty">
            {t("ui.recorder.hint")}
          </div>
        )}
      </div>
      {mode === "trace" && (
        <div className="recorder-foot">
          <span className="recorder-arrows" aria-live="polite">
            {points && points.length >= 4 ? arrows(points) : "—"}
          </span>
          <span className="recorder-hint">{hint ?? (points ? t("ui.recorder.redraw") : "")}</span>
        </div>
      )}
      {error && <p className="field-warn recorder-error" role="status">{error}</p>}
    </div>
  );
}
