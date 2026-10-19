import { useEffect, useState } from "react";
import { api, errText } from "../api";
import { displayName, t } from "../i18n";
import type { Binding } from "../types";
import { ShapeGlyph } from "./Glyph";
import { Recorder } from "./Recorder";

const MAX_SAMPLES = 5;

/** 仅展示核心返回的样本与分数；不在前端实现归一化或形状匹配。 */
export function ShapeRecorder({ templates, onChange, others, feedbackLine }: { templates: number[][]; onChange: (templates: number[][], raw?: [number, number][]) => void; others: Binding[]; feedbackLine?: React.ReactNode }) {
  const [feedback, setFeedback] = useState<{ lowest: number | null; accept: number; conflict: Binding | null } | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    setFeedback(null);
    setError(null);
    const compare = async () => {
      const pairs = templates.flatMap((a, i) => templates.slice(i + 1).map((b) => api.shapeCompare(a, b)));
      const candidates = others.filter((b) => !b.disabled && b.pattern.some((s) => s.type === "shape"));
      const [limits, scores, matches] = await Promise.all([
        api.shapeLimits(),
        Promise.all(pairs),
        Promise.all(candidates.map(async (binding) => {
          const samples = binding.pattern.flatMap((s) => s.type === "shape" ? s.templates : []);
          const scores = await Promise.all(templates.flatMap((a) => samples.map((b) => api.shapeCompare(a, b))));
          return { binding, best: Math.max(0, ...scores) };
        })),
      ]);
      if (!active) return;
      const best = matches.reduce<(typeof matches)[number] | null>((a, b) => !a || b.best > a.best ? b : a, null);
      setFeedback({ lowest: scores.length ? Math.min(...scores) : null, accept: limits.accept, conflict: best && best.best >= limits.accept - limits.margin ? best.binding : null });
    };
    void compare().catch((e) => { if (active) setError(errText(e)); });
    // 切换绑定、删除样本或撤销后，旧的异步比较结果不能覆盖新状态。
    return () => { active = false; };
  }, [templates, others]);

  return (
    <div className="shape-recorder">
      <p className="field-help shape-instruction">{t(templates.length === 1 ? "ui.shape.more_samples" : "ui.shape.instruction")}</p>
      <Recorder
        mode="shape"
        points={templates.at(-1) ?? null}
        disabled={templates.length >= MAX_SAMPLES}
        onChange={(sample, raw) => onChange([...templates, sample].slice(0, MAX_SAMPLES), raw)}
      />
      {feedbackLine}
      <div className="shape-samples-head">
        <span>{t("ui.shape.samples", { count: templates.length, max: MAX_SAMPLES })}</span>
        {templates.length >= MAX_SAMPLES && <span className="muted">{t("ui.shape.limit")}</span>}
      </div>
      <div className="shape-samples">
        {templates.map((sample, i) => (
          <div className="shape-sample" key={i}>
            <ShapeGlyph template={sample} size={48} stroke={2} />
            <button type="button" className="chip-x" aria-label={t("ui.shape.remove_sample", { n: i + 1 })} onClick={() => onChange(templates.filter((_, j) => i !== j))}>×</button>
          </div>
        ))}
      </div>
      <div className="shape-feedback" aria-live="polite">
        {templates.length === 0 && <p className="field-warn">{t("ui.shape.needs_sample")}</p>}
        {feedback?.lowest != null && (
          <p className={`field-help shape-consistency ${feedback.lowest < feedback.accept ? "warn" : ""}`}>
            {t("ui.shape.consistency", { percent: Math.round(feedback.lowest * 100) })}
            {feedback.lowest < feedback.accept && <span className="shape-advice">{t("ui.shape.differ")}</span>}
          </p>
        )}
        {feedback?.conflict && <p className="field-warn shape-conflict">{t("ui.shape.conflict", { name: displayName(feedback.conflict.name, feedback.conflict.name_key) })}</p>}
        {error && <p className="field-warn">{error}</p>}
      </div>
    </div>
  );
}
