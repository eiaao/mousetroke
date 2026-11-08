import { useState } from "react";
import type { Config } from "../types";
import { describePattern, gestureName, hasConflictPattern, patternKey, quote } from "../gesture";
import { displayName, t } from "../i18n";
import { canAdopt, categoryOf, deepEqual, gestureUses, gesturesOf, type BindingRef } from "../library";
import { Modal } from "./Dialogs";
import { GestureGlyph, gestureMatches } from "./GestureLibrary";
import { scopeTitle } from "./Sidebar";

/**
 * 为一条绑定选择手势：手势库（同分类在前，本程序已用的画法置灰并提示冲突）+
 * 其他程序里的专用手势（选中即入库并与原绑定共享）+ 录制新的专用手势。
 */
export function GesturePicker({
  config,
  target,
  onPickGesture,
  onPickInline,
  onRecordNew,
  onClose,
}: {
  config: Config;
  target: BindingRef;
  onPickGesture: (id: string) => void;
  onPickInline: (source: BindingRef) => void;
  onRecordNew: () => void;
  onClose: () => void;
}) {
  const [query, setQuery] = useState("");
  const q = query.trim().toLowerCase();
  const scope = config.scopes[target.scope];
  const binding = scope.bindings[target.binding];
  const cat = categoryOf(binding.pattern);
  // 本程序其他绑定已占用的画法（与分组冲突检查同口径）
  const taken = new Set(scope.bindings.filter((b, i) => i !== target.binding && !b.disabled && hasConflictPattern(b.pattern)).map((b) => patternKey(b.pattern)));
  const library = gesturesOf(config).filter((g) => gestureMatches(config, g, q)).sort(
    (a, b) => Number(categoryOf(a.pattern) !== cat) - Number(categoryOf(b.pattern) !== cat) || Number(taken.has(patternKey(a.pattern))) - Number(taken.has(patternKey(b.pattern))),
  );
  // 其他程序里的专用手势，按画法去重；库里已有的画法不再重复列出
  const others: BindingRef[] = [];
  const seen = gesturesOf(config).map((g) => g.pattern);
  config.scopes.forEach((s, si) => {
    if (si === target.scope) return;
    s.bindings.forEach((b, bi) => {
      if (b.gesture || !canAdopt(b) || seen.some((p) => deepEqual(p, b.pattern))) return;
      seen.push(b.pattern);
      if (q && ![describePattern(b.pattern), scopeTitle(s), displayName(b.name, b.name_key)].some((x) => x.toLowerCase().includes(q))) return;
      others.push({ scope: si, binding: bi });
    });
  });
  others.sort((a, b) => Number(categoryOf(config.scopes[a.scope].bindings[a.binding].pattern) !== cat) - Number(categoryOf(config.scopes[b.scope].bindings[b.binding].pattern) !== cat));

  const name = displayName(binding.name, binding.name_key);
  return (
    <Modal title={t("ui.gestures.picker_title", { name: quote(name) })} onClose={onClose} wide>
      <div className="modal-body gesture-picker">
        <p className="field-help">{t("ui.gestures.picker_help")}</p>
        <div className="picker-top">
          <input className="search" type="search" autoFocus placeholder={t("ui.gestures.search_ph")} aria-label={t("ui.gestures.search_ph")} value={query} onChange={(e) => setQuery(e.target.value)} />
          <button type="button" className="btn" onClick={onRecordNew}>{t("ui.gestures.picker_record")}</button>
        </div>
        <h3 className="picker-section">{t("ui.gestures.title")}</h3>
        {library.length === 0 ? (
          <p className="field-help">{q ? t("ui.gestures.picker_no_match") : t("ui.gestures.picker_empty")}</p>
        ) : (
          <ul className="gesture-grid compact">
            {library.map((g) => {
              const current = binding.gesture === g.id;
              const clash = !current && taken.has(patternKey(g.pattern));
              return (
                <li key={g.id} className={`gesture-card ${current ? "on" : ""} ${clash ? "dim" : ""}`}>
                  <button type="button" disabled={current} onClick={() => onPickGesture(g.id)}>
                    <GestureGlyph pattern={g.pattern} size={38} />
                    <span className="gesture-card-name" title={gestureName(g)}>{gestureName(g)}</span>
                    <span className={`gesture-card-uses ${clash ? "warn" : ""}`}>
                      {current ? t("ui.gestures.picker_current") : clash ? t("ui.gestures.picker_clash") : t("ui.gestures.uses", { count: gestureUses(config, g.id).length })}
                    </span>
                  </button>
                </li>
              );
            })}
          </ul>
        )}
        {others.length > 0 && (
          <>
            <h3 className="picker-section">
              {t("ui.gestures.picker_others")}
              <span className="muted section-hint">{t("ui.gestures.picker_others_help")}</span>
            </h3>
            <ul className="gesture-grid compact">
              {others.map((ref) => {
                const s = config.scopes[ref.scope];
                const b = s.bindings[ref.binding];
                const clash = taken.has(patternKey(b.pattern));
                return (
                  <li key={`${ref.scope}-${ref.binding}`} className={`gesture-card ${clash ? "dim" : ""}`}>
                    <button type="button" onClick={() => onPickInline(ref)}>
                      <GestureGlyph pattern={b.pattern} size={38} />
                      <span className="gesture-card-name" title={describePattern(b.pattern)}>{describePattern(b.pattern)}</span>
                      <span className={`gesture-card-uses ${clash ? "warn" : ""}`}>{clash ? t("ui.gestures.picker_clash") : `${scopeTitle(s)} · ${displayName(b.name, b.name_key)}`}</span>
                    </button>
                  </li>
                );
              })}
            </ul>
          </>
        )}
      </div>
      <footer className="modal-foot">
        <button type="button" className="btn" onClick={onClose}>{t("common.cancel")}</button>
      </footer>
    </Modal>
  );
}
