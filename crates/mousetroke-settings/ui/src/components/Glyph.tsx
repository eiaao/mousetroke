import type { Step } from "../types";
import { decompose, traceOf } from "../gesture";

/** 轨迹点 → SVG 路径坐标（等比缩放进 size×size，留边距）。 */
export function tracePath(points: number[], size: number, pad: number): [number, number][] {
  const xs = points.filter((_, i) => i % 2 === 0);
  const ys = points.filter((_, i) => i % 2 === 1);
  const minX = Math.min(...xs), maxX = Math.max(...xs), minY = Math.min(...ys), maxY = Math.max(...ys);
  const span = Math.max(maxX - minX, maxY - minY, 1);
  const k = (size - pad * 2) / span;
  const ox = pad + ((size - pad * 2) - (maxX - minX) * k) / 2;
  const oy = pad + ((size - pad * 2) - (maxY - minY) * k) / 2;
  const out: [number, number][] = [];
  for (let i = 0; i + 1 < points.length; i += 2) out.push([ox + (points[i] - minX) * k, oy + (points[i + 1] - minY) * k]);
  // 折返（↑↓、→← 等）会与上一段重叠成一条线：在拐点处做一个小 U 形错位
  const gap = Math.max(2.5, size * 0.09);
  const res: [number, number][] = [out[0]];
  let sx = 0, sy = 0;
  for (let i = 1; i < out.length; i++) {
    const p: [number, number] = [out[i][0] + sx, out[i][1] + sy];
    const next = out[i + 1];
    if (next) {
      const [ax, ay] = [out[i][0] - out[i - 1][0], out[i][1] - out[i - 1][1]];
      const [bx, by] = [next[0] - out[i][0], next[1] - out[i][1]];
      const la = Math.hypot(ax, ay), lb = Math.hypot(bx, by);
      if (la && lb && (ax * bx + ay * by) / (la * lb) < -0.99) {
        res.push(p);
        sx += (-ay / la) * gap;
        sy += (ax / la) * gap;
        res.push([out[i][0] + sx, out[i][1] + sy]);
        continue;
      }
    }
    res.push(p);
  }
  return res;
}

export function Arrow({ pts, w }: { pts: [number, number][]; w: number }) {
  if (pts.length < 2) return null;
  const [x1, y1] = pts[pts.length - 2];
  const [x2, y2] = pts[pts.length - 1];
  const a = Math.atan2(y2 - y1, x2 - x1);
  const l = w * 3.2;
  const p = (t: number) => `${x2 - l * Math.cos(a + t)},${y2 - l * Math.sin(a + t)}`;
  return <polygon className="glyph-head" points={`${x2},${y2} ${p(0.5)} ${p(-0.5)}`} />;
}

/** 笔画字形：起点实心圆 + 折线 + 终点箭头。 */
export function TraceGlyph({ points, size = 40, stroke = 2.4, className }: { points: number[]; size?: number; stroke?: number; className?: string }) {
  const pts = tracePath(points, size, size * 0.18);
  return (
    <svg className={`glyph ${className ?? ""}`} width={size} height={size} viewBox={`0 0 ${size} ${size}`} aria-hidden>
      <polyline className="glyph-line" points={pts.map((p) => p.join(",")).join(" ")} strokeWidth={stroke} />
      <circle className="glyph-start" cx={pts[0][0]} cy={pts[0][1]} r={stroke * 1.15} />
      <Arrow pts={pts} w={stroke} />
    </svg>
  );
}

/** 屏幕缩略图，高亮某个边/角。 */
export function EdgeGlyph({ edge, size = 40 }: { edge: number; size?: number }) {
  const w = size, h = size * 0.72, y0 = (size - h) / 2;
  const t = 3.2;
  const rects: [number, number, number, number][] = [];
  const top = edge & 1, right = edge & 2, bottom = edge & 4, left = edge & 8;
  const corner = [top, right, bottom, left].filter(Boolean).length === 2;
  if (corner) {
    const cx = left ? 2 : w - 2 - 10, cy = top ? y0 + 2 : y0 + h - 2 - 10;
    rects.push([cx, cy, 10, 10]);
  } else {
    if (top) rects.push([2, y0 + 2, w - 4, t]);
    if (bottom) rects.push([2, y0 + h - 2 - t, w - 4, t]);
    if (left) rects.push([2, y0 + 2, t, h - 4]);
    if (right) rects.push([w - 2 - t, y0 + 2, t, h - 4]);
  }
  return (
    <svg className="glyph" width={size} height={size} viewBox={`0 0 ${size} ${size}`} aria-hidden>
      <rect className="glyph-screen" x={1} y={y0 + 1} width={w - 2} height={h - 2} rx={3} />
      {rects.map(([x, y, rw, rh], i) => (
        <rect key={i} className="glyph-hot" x={x} y={y} width={rw} height={rh} rx={corner ? 3 : 1.5} />
      ))}
    </svg>
  );
}

/** 鼠标示意（无轨迹的纯按键/滚轮手势）。 */
export function MouseGlyph({ size = 40, wheel }: { size?: number; wheel?: boolean }) {
  const w = size * 0.5, h = size * 0.74, x = (size - w) / 2, y = (size - h) / 2;
  return (
    <svg className="glyph" width={size} height={size} viewBox={`0 0 ${size} ${size}`} aria-hidden>
      <rect className="glyph-screen" x={x} y={y} width={w} height={h} rx={w / 2} />
      <line className="glyph-screen" x1={size / 2} y1={y} x2={size / 2} y2={y + h * 0.38} />
      {wheel && <rect className="glyph-hot" x={size / 2 - 1.6} y={y + h * 0.12} width={3.2} height={h * 0.2} rx={1.6} />}
    </svg>
  );
}

/** 列表里的手势图标：优先轨迹，其次边角，最后鼠标。 */
export function PatternGlyph({ pattern, size = 40 }: { pattern: Step[]; size?: number }) {
  const trace = traceOf(pattern);
  if (trace && trace.length >= 4) return <TraceGlyph points={trace} size={size} />;
  const m = decompose(pattern);
  if (m.trigger.kind === "edge") return <EdgeGlyph edge={m.trigger.edge} size={size} />;
  return <MouseGlyph size={size} wheel={m.items.some((i) => i.kind === "key" && i.key.startsWith("wheel"))} />;
}
