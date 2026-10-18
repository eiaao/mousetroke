import { useEffect, useRef, useState } from "react";
import { api } from "../api";
import { arrows } from "../gesture";
import { t } from "../i18n";
import { Arrow, tracePath } from "./Glyph";

const GRID = 30; // 与引擎默认最小识别距离一致：画布上一格 = 30px

/**
 * 手势录制画布。用左键或右键按住拖动画出手势，松开后由引擎同一套算法
 * （mousetroke-core::recognizer::record_points）规整为标准轨迹并以动画展示。
 */
export function Recorder({ points, onChange, hint }: { points: number[] | null; onChange: (p: number[]) => void; hint?: React.ReactNode }) {
  const wrap = useRef<HTMLDivElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const raw = useRef<[number, number][]>([]);
  const [drawing, setDrawing] = useState(false);
  const [tooShort, setTooShort] = useState(false);
  const [animKey, setAnimKey] = useState(0);
  const [size, setSize] = useState({ w: 400, h: 200 });

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
    if (e.button !== 0 && e.button !== 2) return;
    e.preventDefault();
    canvas.current!.setPointerCapture(e.pointerId);
    raw.current = [local(e)];
    clearCanvas();
    setTooShort(false);
    setDrawing(true);
  };

  const onMove = (e: React.PointerEvent) => {
    if (!drawing) return;
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
    if (!drawing) return;
    setDrawing(false);
    canvas.current!.releasePointerCapture(e.pointerId);
    const recorded = await api.recordTrace(raw.current, GRID);
    if (recorded.length < 4) {
      setTooShort(true);
      return;
    }
    clearCanvas();
    setAnimKey((k) => k + 1);
    onChange(recorded);
  };

  // 规整后的标准轨迹（SVG，绘制动画）
  const shown = !drawing && points && points.length >= 4;
  const box = Math.min(size.w, size.h);
  const pts = shown ? tracePath(points!, box, box * 0.22) : [];
  const off = { x: (size.w - box) / 2, y: 0 };
  const len = pts.reduce((acc, p, i) => (i ? acc + Math.hypot(p[0] - pts[i - 1][0], p[1] - pts[i - 1][1]) : 0), 0);

  return (
    <div className="recorder">
      <div
        ref={wrap}
        className={`recorder-pad ${drawing ? "drawing" : ""}`}
        style={{ "--grid": `${GRID}px` } as React.CSSProperties}
        onContextMenu={(e) => e.preventDefault()}
      >
        <canvas ref={canvas} onPointerDown={onDown} onPointerMove={onMove} onPointerUp={onUp} onPointerCancel={onUp} />
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
            {tooShort ? t("ui.recorder.too_short") : points ? "" : t("ui.recorder.hint")}
          </div>
        )}
      </div>
      <div className="recorder-foot">
        <span className="recorder-arrows" aria-live="polite">
          {points && points.length >= 4 ? arrows(points) : "—"}
        </span>
        <span className="recorder-hint">{hint ?? (points ? t("ui.recorder.redraw") : "")}</span>
      </div>
    </div>
  );
}
