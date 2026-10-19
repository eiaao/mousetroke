import { useEffect, useState } from "react";
import { api, errText } from "../api";
import { displayName, t } from "../i18n";
import type { Binding } from "../types";
import { ShapeGlyph } from "./Glyph";
import { Recorder } from "./Recorder";

const MAX_SAMPLES = 5;

/** 仅展示核心返回的样本与分数；不在前端实现归一化或形状匹配。 */
export function ShapeRecorder({ templates, onChange, others, feedbackLine }: { templates: number[][]; onChange: (templates: number[][], raw?: [number, number][]) => void; others: Binding[]; feedbackLine?: React.ReactNode }) {
  const [feedback, setFeedback] = useState<{ lowest: number | null; inconsistent: boolean; conflict: Binding | null } | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    setFeedback(null);
    setError(null);
    const compare = async () => {
      const candidates = others.filter((b) => !b.disabled && b.pattern.some((s) => s.type === "shape"));
      const otherSamples = candidates.map((binding) => binding.pattern.flatMap((s) => s.type === "shape" ? s.templates : []));
      const allSamples = [...templates, ...otherSamples.flat()];
      // 每个样本作为绘制输入时的阈值由核心决定，前端不推断曲线/折线。
      const limits = new Map(await Promise.all(allSamples.map(async (sample) => [sample, await api.shapeLimits(sample)] as const)));
      const comparePair = async (a: number[], b: number[]) => {
        const score = await api.shapeCompare(a, b);
        const left = limits.get(a)!;
        const right = limits.get(b)!;
        return { score, inconsistent: score < Math.max(left.accept, right.accept), conflict: score >= Math.min(left.accept - left.margin, right.accept - right.margin) };
      };
      const pairs = templates.flatMap((a, i) => templates.slice(i + 1).map((b) => comparePair(a, b)));
      const [scores, matches] = await Promise.all([
        Promise.all(pairs),
        Promise.all(candidates.map(async (binding, i) => {
          const scores = await Promise.all(templates.flatMap((a) => otherSamples[i].map((b) => comparePair(a, b))));
          return { binding, best: Math.max(0, ...scores.filter((s) => s.conflict).map((s) => s.score)), conflict: scores.some((s) => s.conflict) };
        })),
      ]);
      if (!active) return;
      const best = matches.filter((m) => m.conflict).reduce<(typeof matches)[number] | null>((a, b) => !a || b.best > a.best ? b : a, null);
      setFeedback({ lowest: scores.length ? Math.min(...scores.map((s) => s.score)) : null, inconsistent: scores.some((s) => s.inconsistent), conflict: best?.binding ?? null });
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
          <p className={`field-help shape-consistency ${feedback.inconsistent ? "warn" : ""}`}>
            {t("ui.shape.consistency", { percent: Math.round(feedback.lowest * 100) })}
            {feedback.inconsistent && <span className="shape-advice">{t("ui.shape.differ")}</span>}
          </p>
        )}
        {feedback?.conflict && <p className="field-warn shape-conflict">{t("ui.shape.conflict", { name: displayName(feedback.conflict.name, feedback.conflict.name_key) })}</p>}
        {error && <p className="field-warn">{error}</p>}
      </div>
    </div>
  );
}
