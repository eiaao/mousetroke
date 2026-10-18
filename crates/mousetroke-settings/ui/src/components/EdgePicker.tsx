import { EDGE, edgeName } from "../gesture";
import { t } from "../i18n";

// 屏幕示意图上的 8 个可点击热区（4 角 + 4 边），键盘可用方向键切换。

const ZONES: { edge: number; style: React.CSSProperties }[] = [
  { edge: EDGE.leftTop, style: { left: 0, top: 0, width: "16%", height: "22%" } },
  { edge: EDGE.top, style: { left: "16%", top: 0, width: "68%", height: "14%" } },
  { edge: EDGE.rightTop, style: { right: 0, top: 0, width: "16%", height: "22%" } },
  { edge: EDGE.left, style: { left: 0, top: "22%", width: "10%", height: "56%" } },
  { edge: EDGE.right, style: { right: 0, top: "22%", width: "10%", height: "56%" } },
  { edge: EDGE.leftBottom, style: { left: 0, bottom: 0, width: "16%", height: "22%" } },
  { edge: EDGE.bottom, style: { left: "16%", bottom: 0, width: "68%", height: "14%" } },
  { edge: EDGE.rightBottom, style: { right: 0, bottom: 0, width: "16%", height: "22%" } },
];

export function EdgePicker({ value, onChange }: { value: number; onChange: (edge: number) => void }) {
  return (
    <div className="edge-picker">
      <div className="edge-screen" role="radiogroup" aria-label={t("ui.pattern.edge_aria")}>
        {ZONES.map((z) => (
          <button
            key={z.edge}
            type="button"
            role="radio"
            aria-checked={value === z.edge}
            aria-label={edgeName(z.edge)}
            data-tip={edgeName(z.edge)}
            className={`edge-zone ${value === z.edge ? "on" : ""} ${[3, 6, 9, 12].includes(z.edge) ? "corner" : ""}`}
            style={z.style}
            onClick={() => onChange(z.edge)}
          />
        ))}
        <span className="edge-caption">{edgeName(value) || t("ui.pattern.pick_edge")}</span>
      </div>
    </div>
  );
}
