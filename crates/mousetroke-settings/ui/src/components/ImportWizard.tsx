import { useMemo, useState } from "react";
import type { Config, ImportGroup, ImportPlan, ImportSource } from "../types";
import { api, errText } from "../api";
import { arrows, describePattern, quote, traceOf } from "../gesture";
import { displayName, t } from "../i18n";
import { applyImportPlan, defaultChoices, deepEqual, groupPattern, type GroupChoice } from "../library";
import { Modal } from "./Dialogs";
import { ShapeGlyph } from "./Glyph";
import { GestureGlyph } from "./GestureLibrary";
import { scopeTitle } from "./Sidebar";

const pct = (v: number | undefined) => Math.round((v ?? 0) * 100);

/**
 * 整理并导入手势：第 1 步选来源，第 2 步展示后端分析结果（相同 / 方向相同的折线 / 相似形状），
 * 用户逐组决定后一次应用（可撤销）。分析前后都不修改配置。
 */
export function ImportWizard({ config, onApply, onClose }: { config: Config; onApply: (plan: ImportPlan, choices: GroupChoice[]) => void; onClose: () => void }) {
  const [source, setSource] = useState<ImportSource>("global");
  const [plan, setPlan] = useState<ImportPlan | null>(null);
  const [choices, setChoices] = useState<GroupChoice[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showDirect, setShowDirect] = useState(false);

  const analyze = async () => {
    setBusy(true);
    setError(null);
    try {
      const p = await api.analyzeGestureImport(config, source);
      setPlan(p);
      setChoices(defaultChoices(p));
    } catch (e) {
      setError(errText(e));
    } finally {
      setBusy(false);
    }
  };

  const stats = useMemo(() => (plan ? applyImportPlan(config, plan, choices).stats : null), [config, plan, choices]);
  const setChoice = (gi: number, patch: Partial<GroupChoice>) => setChoices((cs) => cs.map((c, i) => (i === gi ? { ...c, ...patch } : c)));

  if (!plan) {
    const option = (value: ImportSource, title: string, help: string) => (
      <label className={`import-source ${source === value ? "on" : ""}`}>
        <input type="radio" name="import-source" checked={source === value} onChange={() => setSource(value)} />
        <span>
          {title}
          <small>{help}</small>
        </span>
      </label>
    );
    return (
      <Modal title={t("ui.gestures.wizard_title")} onClose={onClose} wide>
        <div className="modal-body">
          <p className="field-help">{t("ui.gestures.wizard_intro")}</p>
          {option("global", t("ui.gestures.source_global"), t("ui.gestures.source_global_help"))}
          {option("all", t("ui.gestures.source_all"), t("ui.gestures.source_all_help"))}
          {error && <p className="field-warn">{error}</p>}
        </div>
        <footer className="modal-foot btn-row">
          <button type="button" className="btn" onClick={onClose}>{t("common.cancel")}</button>
          <button type="button" className="btn primary" disabled={busy} onClick={analyze}>{busy ? t("ui.gestures.analyzing") : t("ui.gestures.next")}</button>
        </footer>
      </Modal>
    );
  }

  const indexed = plan.groups.map((g, i) => ({ g, i }));
  const direct = indexed.filter(({ g }) => g.kind === "identical");
  const equivalent = indexed.filter(({ g }) => g.kind === "equivalent");
  const similar = indexed.filter(({ g }) => g.kind === "similar");
  const nothing = plan.groups.length === 0;
  const directPicked = direct.filter(({ i }) => !choices[i]?.skip).length;
  const setSkip = (rows: { i: number }[], skip: boolean) => setChoices((cs) => cs.map((c, i) => (rows.some((r) => r.i === i) ? { ...c, skip } : c)));
  const empty = !!stats && stats.added + stats.linked === 0;
  const memberText = (scope: number, binding: number) => {
    const s = config.scopes[scope];
    const b = s.bindings[binding];
    return { scope: scopeTitle(s), name: displayName(b.name, b.name_key) };
  };

  const groupCard = (g: ImportGroup, gi: number) => {
    const ch = choices[gi];
    const final = groupPattern(config, g, ch);
    const radio = (value: GroupChoice["choice"], label: string) => (
      <label className="import-choice">
        <input type="radio" name={`import-${gi}`} disabled={!ch.checked} checked={ch.choice === value} onChange={() => setChoice(gi, { choice: value })} />
        {label}
      </label>
    );
    const shape = g.kind === "similar";
    return (
      <div key={gi} className={`import-group ${ch.checked ? "" : "off"}`}>
        <div className="import-group-head">
          <input type="checkbox" checked={ch.checked} aria-label={describePattern(final)} onChange={(e) => setChoice(gi, { checked: e.target.checked })} />
          <GestureGlyph pattern={final} size={30} />
          <span className="import-group-title">{describePattern(final)}</span>
          {shape ? (
            <>
              <span className="muted">{t("ui.gestures.similarity", { percent: pct(g.score) })}</span>
              {!g.checked && <span className="import-low">{t("ui.gestures.low")}</span>}
            </>
          ) : (
            <span className="muted">{arrows(traceOf(g.pattern) ?? [])} · {t("ui.gestures.length_only")}</span>
          )}
        </div>
        {g.existing && (
          <div className="import-member">
            <GestureGlyph pattern={g.pattern} size={36} />
            <span className="import-who">{t("ui.gestures.in_library")}</span>
            {radio("rep", shape ? t("ui.gestures.use_samples") + (g.consistency != null ? ` · ${t("ui.shape.consistency", { percent: pct(g.consistency) })}` : "") : t("ui.gestures.use_this"))}
          </div>
        )}
        {g.members.map((m, mi) => {
          const who = memberText(m.scope, m.binding);
          const pattern = config.scopes[m.scope].bindings[m.binding].pattern;
          const changes = ch.checked && m.relation !== "linked" && !deepEqual(pattern, final);
          const isRep = !g.existing && mi === 0;
          return (
            <div key={mi} className="import-member">
              <GestureGlyph pattern={pattern} size={36} />
              <span className="import-who">
                {who.scope}
                <small>{who.name}{m.relation === "linked" ? ` · ${t("ui.gestures.in_library")}` : ""}</small>
                {changes && <small className="import-change">{t("ui.gestures.will_change")}</small>}
              </span>
              {m.relation !== "linked" && radio(isRep ? "rep" : mi, shape ? t("ui.gestures.use_samples") + (m.consistency != null ? ` · ${t("ui.shape.consistency", { percent: pct(m.consistency) })}` : "") : t("ui.gestures.use_this"))}
            </div>
          );
        })}
        {shape && g.merged && (
          <div className="import-member import-merge">
            <span className="import-samples">
              {g.merged.templates.map((tpl, i) => <ShapeGlyph key={i} template={tpl} size={26} stroke={1.6} />)}
            </span>
            <span className="import-who">
              {t("ui.gestures.samples", { count: g.merged.templates.length })}
              {g.merged.consistency != null && <small>{t("ui.shape.consistency", { percent: pct(g.merged.consistency) })}</small>}
            </span>
            {radio("merge", t("ui.gestures.merge_samples"))}
          </div>
        )}
      </div>
    );
  };

  return (
    <Modal title={`${t("ui.gestures.wizard_title")} · ${t("ui.gestures.review")}`} onClose={onClose} wide>
      <div className="modal-body import-review">
        <p className="field-help">{plan.source === "global" ? t("ui.gestures.source_global") : t("ui.gestures.source_all")} · {t("ui.gestures.untouched")}</p>
        {nothing ? (
          <p className="import-nothing">{t("ui.gestures.nothing")}</p>
        ) : (
          <>
            {stats && (
              <p className="import-summary" role="status">
                <b>{t("ui.gestures.will_add", { count: stats.added })}</b>
                <span> · </span>
                <b>{t("ui.gestures.will_link", { count: stats.linked })}</b>
                {stats.replaced > 0 && (
                  <>
                    <span> · </span>
                    <span className="import-change">{t("ui.gestures.will_replace", { count: stats.replaced })}</span>
                  </>
                )}
              </p>
            )}
            {direct.length > 0 && (
              <section className="import-section">
                <div className="import-section-row">
                  <button type="button" className="import-section-head" aria-expanded={showDirect} onClick={() => setShowDirect((v) => !v)}>
                    <span className="import-twist">{showDirect ? "▾" : "▸"}</span>
                    {t("ui.gestures.direct")}
                    <span className="muted"> {t("ui.gestures.direct_count", { picked: directPicked, total: direct.length })}</span>
                  </button>
                  {showDirect && (
                    <span className="import-bulk">
                      <button type="button" className="link" disabled={directPicked === direct.length} onClick={() => setSkip(direct, false)}>{t("ui.gestures.select_all")}</button>
                      <button type="button" className="link" disabled={directPicked === 0} onClick={() => setSkip(direct, true)}>{t("ui.gestures.select_none")}</button>
                    </span>
                  )}
                </div>
                <p className="field-help">{showDirect ? t("ui.gestures.direct_pick_help") : t("ui.gestures.direct_help")}</p>
                {showDirect && (
                  <ul className="import-direct">
                    {direct.map(({ g, i }) => {
                      const on = !choices[i]?.skip;
                      const where = g.members.map((m) => { const w = memberText(m.scope, m.binding); return `${w.scope} · ${w.name}`; });
                      return (
                        <li key={i} className={on ? "" : "off"}>
                          <label>
                            <input type="checkbox" checked={on} onChange={(e) => setChoice(i, { skip: !e.target.checked })} />
                            <GestureGlyph pattern={g.pattern} size={26} />
                            <span className="import-direct-name">{describePattern(g.pattern)}</span>
                            <span className="import-direct-where muted" title={where.join("\n")}>{where.join(t("ui.gestures.list_sep"))}</span>
                          </label>
                        </li>
                      );
                    })}
                  </ul>
                )}
              </section>
            )}
            {equivalent.length > 0 && (
              <section className="import-section">
                <h3>{t("ui.gestures.equivalent")}<span className="muted"> ({equivalent.length})</span></h3>
                <p className="field-help">{t("ui.gestures.equivalent_help")}</p>
                {equivalent.map(({ g, i }) => groupCard(g, i))}
              </section>
            )}
            {similar.length > 0 && (
              <section className="import-section">
                <h3>{t("ui.gestures.similar")}<span className="muted"> ({similar.length})</span></h3>
                <p className="field-help">{t("ui.gestures.similar_help")}</p>
                {similar.map(({ g, i }) => groupCard(g, i))}
              </section>
            )}
            {plan.notes.map((n, i) => {
              const s = config.scopes[n.scope];
              const a = s.bindings[n.first];
              const b = s.bindings[n.second];
              return (
                <p key={i} className="import-note">
                  {t("ui.gestures.kept_apart", { scope: quote(scopeTitle(s)), first: quote(displayName(a.name, a.name_key)), second: quote(displayName(b.name, b.name_key)) })}
                </p>
              );
            })}
          </>
        )}
      </div>
      <footer className="modal-foot btn-row">
        {!nothing && <span className="muted modal-foot-hint">{t("ui.gestures.undo_hint")}</span>}
        <button type="button" className="btn" onClick={() => setPlan(null)}>{t("ui.gestures.back")}</button>
        <button type="button" className="btn" onClick={onClose}>{t("common.cancel")}</button>
        {!nothing && <button type="button" className="btn primary" disabled={empty} onClick={() => onApply(plan, choices)}>{t("ui.gestures.do_import")}</button>}
      </footer>
    </Modal>
  );
}
