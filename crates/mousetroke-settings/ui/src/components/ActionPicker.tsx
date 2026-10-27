import { useEffect, useMemo, useRef, useState } from "react";
import type { Action, ActionPreset } from "../types";
import {
  ADVANCED_PICKS,
  CATEGORY_ORDER,
  cleanPresetParams,
  findPreset,
  pickerInitial,
  presetName,
  presetKeys,
  presetDescription,
  visibleParams,
  matchAdvancedPick,
  missingRequiredParams,
  pickerNav,
  presetParamValues,
  presetParamRequired,
  presetPickResult,
  presetPlatformStatus,
  presetStatusReason,
  visiblePresets,
  type PickerCategory,
} from "../actions";
import { defaultAction } from "../gesture";
import { currentLang, t, type MsgKey } from "../i18n";
import { api, errText, type Msg } from "../api";
import { Modal } from "./Dialogs";
import { Caps } from "./KeysInput";

/**
 * 动作选择器：按"我想做什么"选动作。搜索框自动聚焦；
 * 上下键移动、回车确认、Esc 关闭；带参数的动作在面板下方展开参数表单。
 */
export function ActionPicker({
  current,
  depth = 0,
  initialId,
  onPick,
  onClose,
}: {
  /** 当前动作：是同 id 的预置动作时，参数表单预填现有值。 */
  current?: Action;
  /** 弹出菜单里不能再嵌套弹出菜单（depth > 0 时"高级"隐藏 menu）。 */
  depth?: number;
  /** 打开时直接展开某个带参数动作的表单（测试与"更换"流程用）。 */
  initialId?: string;
  onPick: (a: Action) => void;
  onClose: () => void;
}) {
  const initial = pickerInitial(current, initialId);
  const [q, setQ] = useState("");
  const [cat, setCat] = useState<PickerCategory>(initial.category);
  const [active, setActive] = useState(initial.active);
  const [formId, setFormId] = useState<string | null>(initialId && initial.preset?.params.length ? initial.preset.id : null);
  const [values, setValues] = useState<Record<string, string>>(() => initial.preset ? presetParamValues(initial.preset, current) : {});
  const [announce, setAnnounce] = useState("");
  const [formError, setFormError] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const submission = useRef<{ id: string } | null>(null);
  const listRef = useRef<HTMLUListElement>(null);
  const bodyRef = useRef<HTMLDivElement>(null);
  const searchRef = useRef<HTMLInputElement>(null);
  // 首次挂载（含 initialId）不重置分类/表单
  const mounted = useRef(false);

  const advanced = useMemo(() => ADVANCED_PICKS.filter((a) => !(depth > 0 && a.type === "menu")), [depth]);
  const presets = useMemo(() => visiblePresets(cat, q), [cat, q]);
  const searching = q.trim() !== "";
  const advancedVisible = searching
    ? advanced.filter((a) => matchAdvancedPick(a.nameKey, a.descKey, q))
    : advanced;
  const rowCount = presets.length + advancedVisible.length;
  const selectable = useMemo(() => {
    return [...presets.map((p) => presetPlatformStatus(p) === "implemented"), ...advancedVisible.map(() => true)];
  }, [presets, advancedVisible]);
  const activeRow = Math.min(active, Math.max(0, rowCount - 1));
  const formPreset = formId ? findPreset(formId) : undefined;
  const missing = formPreset ? missingRequiredParams(formPreset, values) : [];

  const invalidateValidation = () => {
    submission.current = null;
    setSubmitting(false);
    setFormError("");
  };
  const closeForm = () => {
    invalidateValidation();
    setFormId(null);
    setValues({});
  };
  const closePicker = () => {
    invalidateValidation();
    onClose();
  };

  // 打开时显式聚焦搜索框（不依赖 Modal 的默认聚焦）
  useEffect(() => {
    searchRef.current?.focus();
    return () => { submission.current = null; };
  }, []);

  // 换分类或改搜索词时回到第一行，收起参数表单（跳过首次挂载）
  useEffect(() => {
    if (!mounted.current) {
      mounted.current = true;
      return;
    }
    setActive(0);
    closeForm();
  }, [cat, q]);

  // 高亮行滚动到可见
  useEffect(() => {
    listRef.current?.querySelector(`#actpick-opt-${activeRow}`)?.scrollIntoView({ block: "nearest" });
  }, [activeRow, cat, q]);

  const openForm = (p: ActionPreset) => {
    invalidateValidation();
    const base = presetParamValues(p, current);
    setFormId(p.id);
    setValues(base);
    setAnnounce(t("ui.actpick.form_opened", { name: presetName(p) }));
  };

  const announceBlocked = (p: ActionPreset) => {
    const reason = presetStatusReason(p);
    setAnnounce(reason ? t(reason) : t("preset.status.unknown"));
  };

  const applyForm = async (p: ActionPreset) => {
    if (submission.current?.id === p.id) return;
    const missing = missingRequiredParams(p, values);
    if (missing.length) {
      // 校验不通过：就地提示并聚焦第一个缺项
      const first = missing[0];
      const el = bodyRef.current?.querySelector<HTMLElement>(`[data-param="${first}"]`);
      el?.focus();
      return;
    }
    const params = cleanPresetParams(p, values);
    const attempt = { id: p.id };
    submission.current = attempt;
    setSubmitting(true);
    setFormError("");
    try {
      await api.validatePreset(p.id, params, currentLang());
    } catch (error) {
      if (submission.current !== attempt) return;
      submission.current = null;
      setSubmitting(false);
      setFormError(errText(error));
      const param = error && typeof error === "object" && "params" in error ? (error as Msg).params?.param : undefined;
      if (param) Array.from(bodyRef.current?.querySelectorAll<HTMLElement>("[data-param]") ?? []).find((el) => el.dataset.param === param)?.focus();
      return;
    }
    // 换表单、修改输入或关闭后，晚到的校验结果不能再选用旧动作。
    if (submission.current !== attempt) return;
    setSubmitting(false);
    onPick({ type: "preset", id: p.id, ...(p.params.length ? { params } : {}) });
  };

  /** 确认一行：无参数的动作直接选中；带参数的展开表单，再按一次回车/点"选用"确认。 */
  const activate = (index: number) => {
    setActive(index);
    if (index < presets.length) {
      const p = presets[index];
      switch (presetPickResult(p, formId === p.id, values)) {
        case "pick":
          if (formId) closeForm();
          applyForm(p);
          break;
        case "form":
          openForm(p);
          break;
        case "apply-form":
          applyForm(p);
          break;
        case "invalid":
          setFormId(p.id);
          applyForm(p);
          break;
        case "blocked":
          announceBlocked(p);
          break;
      }
    } else {
      const adv = advancedVisible[index - presets.length];
      if (adv) { invalidateValidation(); onPick(defaultAction(adv.type)); }
    }
  };

  const onKeyDown = (e: React.KeyboardEvent) => {
    const target = e.target as HTMLElement;
    const inForm = !!target.closest(".act-form");
    if (e.key === "Escape") {
      e.stopPropagation();
      // 有展开的表单时先收起表单并把焦点回到搜索框；没有表单时才关闭
      if (formId) {
        closeForm();
        searchRef.current?.focus();
      } else {
        closePicker();
      }
      return;
    }
    if (e.key === "Enter") {
      if (inForm) {
        if (target.tagName === "TEXTAREA") {
          if ((e.ctrlKey || e.metaKey) && formPreset) {
            e.preventDefault();
            applyForm(formPreset);
          }
          return;
        }
        if (target.tagName === "SELECT") return;
        if (target.tagName === "INPUT") {
          e.preventDefault();
          if (formPreset) applyForm(formPreset);
          return;
        }
        return;
      }
      // 搜索框/列表区域的回车：表单打开时提交表单（避免误选高亮行），否则激活高亮行
      if (target === searchRef.current || target === listRef.current) {
        e.preventDefault();
        if (formPreset) applyForm(formPreset);
        else activate(activeRow);
      }
      return;
    }
    // 参数表单内的多行文字框里上下键是编辑文本；choice 的 radiogroup 用左右键切换
    if (inForm && (target.tagName === "TEXTAREA" || target.tagName === "SELECT" || target.closest("[role=radiogroup]"))) return;
    if (target !== searchRef.current && target !== listRef.current && !target.closest(".act-list")) return;
    const next = pickerNav(activeRow, rowCount, e.key, selectable);
    if (next !== null && next !== activeRow) {
      e.preventDefault();
      setActive(next);
      setAnnounce("");
    }
  };

  // 打开参数表单后聚焦表单里的第一个输入
  useEffect(() => {
    if (formId) bodyRef.current?.querySelector<HTMLElement>('.act-form [data-param]:not([role="radio"]), .act-form [role="radio"][aria-checked="true"]')?.focus();
  }, [formId]);

  return (
    <Modal title={t("ui.actpick.title")} onClose={closePicker} wide onEscape={() => {
      if (!formId) return false;
      closeForm(); searchRef.current?.focus(); return true;
    }}>
      <div ref={bodyRef} className="modal-body act-pick" onKeyDown={onKeyDown}>
        <input
          ref={searchRef}
          className="search"
          type="search"
          role="combobox"
          aria-expanded="true"
          aria-controls="actpick-list"
          aria-activedescendant={rowCount > 0 ? `actpick-opt-${activeRow}` : undefined}
          aria-label={t("ui.actpick.search_ph")}
          placeholder={t("ui.actpick.search_ph")}
          value={q}
          onChange={(e) => { invalidateValidation(); setQ(e.target.value); }}
        />
        <div className="act-cats" aria-label={t("ui.actpick.cat_aria")}>
          {CATEGORY_ORDER.map((c) => (
            <button
              key={c}
              type="button"
              aria-pressed={!searching && cat === c}
              className={`act-cat ${!searching && cat === c ? "on" : ""}`}
              onClick={() => { invalidateValidation(); setQ(""); setCat(c); }}
            >
              {t(categoryKey(c))}
            </button>
          ))}
        </div>
        <p className="act-search-hint muted" aria-live="polite">{searching ? t("ui.actpick.search_all_cats") : "\u00a0"}</p>
        <ul ref={listRef} id="actpick-list" className="act-list" role="listbox" tabIndex={0} aria-label={t("ui.actpick.list_aria")}>
          {presets.map((p, i) => {
            const status = presetPlatformStatus(p);
            const reason = presetStatusReason(p);
            const active = i === activeRow;
            return (
              <li key={p.id} id={`actpick-opt-${i}`} className={active ? "on" : ""} role="option" aria-selected={active} onMouseMove={() => setActive(i)}>
                <button
                  type="button"
                  tabIndex={-1}
                  disabled={status !== "implemented"}
                  title={presetDescription(p)}
                  onClick={() => activate(i)}
                >
                  <span className="act-name">{presetName(p)}{p.params.length ? "…" : ""}</span>
                  <span className="act-desc">
                    {status === "pending" ? t("ui.actpick.soon") : reason ? t(reason) : presetKeys(p) ? <Caps keys={presetKeys(p)!} compact /> : null}
                  </span>
                </button>
              </li>
            );
          })}
          {advancedVisible.length > 0 && <li className="act-group" role="presentation">{t("ui.actpick.cat_advanced")}</li>}
          {advancedVisible.map((a, j) => {
            const i = presets.length + j;
            const active = i === activeRow;
            return (
              <li key={a.type} id={`actpick-opt-${i}`} className={active ? "on" : ""} role="option" aria-selected={active} onMouseMove={() => setActive(i)}>
                <button type="button" tabIndex={-1} onClick={() => activate(i)}>
                  <span className="act-name">{t(a.nameKey)}</span>
                  <span className="act-desc">{t(a.descKey)}</span>
                </button>
              </li>
            );
          })}
          {rowCount === 0 && <li className="muted act-none">{t("ui.actpick.no_match")}</li>}
        </ul>
        <p className="act-detail muted">{presets[activeRow] ? presetDescription(presets[activeRow]) : "\u00a0"}</p>
        {formError && <p className="field-error" role="alert">{formError}</p>}
        {formPreset && (
          <div className="act-form">
            {visibleParams(formPreset, values).map((param) => (
              <ParamField
                key={param.name}
                param={param}
                value={values[param.name] ?? ""}
                onChange={(v) => { invalidateValidation(); setValues((old) => ({ ...old, [param.name]: v })); }}
              />
            ))}
            <div className="act-form-foot">
              {missing.length > 0 && <span className="field-error">{t("ui.actpick.fill_required")}</span>}
              <button type="button" className="btn primary small" disabled={submitting || missing.length > 0} onClick={() => applyForm(formPreset)}>
                {t("ui.actpick.apply")}
              </button>
            </div>
          </div>
        )}
        <div aria-live="polite" className="sr-only">{announce}</div>
      </div>
    </Modal>
  );
}

function categoryKey(c: PickerCategory): MsgKey {
  return (c === "advanced" ? "ui.actpick.cat_advanced" : c === "common" ? "preset.category.common" : `preset.category.${c}`) as MsgKey;
}

/** 按参数类型渲染表单字段；路径带"浏览"，程序带"浏览/点选窗口"。 */
function ParamField({ param, value, onChange }: { param: ActionPreset["params"][number]; value: string; onChange: (v: string) => void }) {
  const inputRef = useRef<HTMLInputElement>(null);
  const label = t(param.label_key as MsgKey);
  const selectionLabel = `{${t("ui.var.selected_text")}}`;
  const customUrl = param.label_key === "preset.param.search_url";
  const displayValue = customUrl ? value.replaceAll("%MS_SELECTED_TEXT%", selectionLabel) : value;
  const required = presetParamRequired(param);
  const empty = required && !(param.kind.type === "multiline_text" ? value : value.trim());
  const error = empty ? <span className="field-error">{t("ui.actpick.required")}</span> : null;
  switch (param.kind.type) {
    case "number":
      return (
        <div className="field">
          <label className="field-label" htmlFor={`param-${param.name}`}>{label}</label>
          <input
            id={`param-${param.name}`}
            type="number"
            min={param.kind.min}
            max={param.kind.max}
            step={param.kind.step}
            value={value}
            aria-required={required}
            aria-invalid={empty}
            data-param={param.name}
            onChange={(e) => onChange(e.target.value)}
          />
          {error}
        </div>
      );
    case "choice":
      return (
        <div className="field">
          <span className="field-label" id={`param-label-${param.name}`}>{label}</span>
          <div className="segmented wrap" role="radiogroup" aria-labelledby={`param-label-${param.name}`}>
            {param.kind.choices.map((c) => (
              <button
                key={c.value}
                type="button"
                role="radio"
                aria-checked={value === c.value}
                className={value === c.value ? "on" : ""}
                data-param={param.name}
                onClick={() => onChange(c.value)}
              >
                {t(c.label_key as MsgKey)}
              </button>
            ))}
          </div>
        </div>
      );
    case "multiline_text":
      return (
        <div className="field">
          <label className="field-label" htmlFor={`param-${param.name}`}>{label}</label>
          <textarea
            id={`param-${param.name}`}
            rows={3}
            value={value}
            spellCheck={false}
            aria-required={required}
            aria-invalid={empty}
            data-param={param.name}
            onChange={(e) => onChange(e.target.value)}
          />
          {error}
        </div>
      );
    case "path":
      return (
        <div className="field">
          <label className="field-label" htmlFor={`param-${param.name}`}>{label}</label>
          <div className="input-row">
            <input
              id={`param-${param.name}`}
              value={value}
              spellCheck={false}
              aria-required={required}
              aria-invalid={empty}
              data-param={param.name}
              onChange={(e) => onChange(e.target.value)}
            />
            <button
              type="button"
              className="btn small"
              onClick={async () => {
                const p = param.kind.type === "path" && param.kind.kind === "folder" ? await api.pickFolder(label) : await api.pickFile("any", false, label);
                if (p) onChange(p);
              }}
            >
              {t("common.browse")}
            </button>
          </div>
          {error}
        </div>
      );
    case "app":
      return (
        <div className="field">
          <label className="field-label" htmlFor={`param-${param.name}`}>{label}</label>
          <div className="input-row">
            <input
              id={`param-${param.name}`}
              value={value}
              spellCheck={false}
              aria-required={required}
              aria-invalid={empty}
              data-param={param.name}
              onChange={(e) => onChange(e.target.value)}
            />
            <button
              type="button"
              className="btn small"
              onClick={async () => {
                const p = await api.pickFile("exe", false, label);
                if (p) onChange(p);
              }}
            >
              {t("common.browse")}
            </button>
            <button
              type="button"
              className="btn small"
              onClick={async () => {
                const p = await api.pickWindow();
                if (p) onChange(p.exe);
              }}
            >
              {t("ui.picker.pick_window")}
            </button>
          </div>
          {error}
        </div>
      );
    default:
      // text / url
      return (
        <div className="field">
          <label className="field-label" htmlFor={`param-${param.name}`}>{label}</label>
          <input
            ref={inputRef}
            id={`param-${param.name}`}
            value={displayValue}
            spellCheck={false}
            aria-required={required}
            aria-invalid={empty}
            data-param={param.name}
            onChange={(e) => onChange(customUrl ? e.target.value.replaceAll(selectionLabel, "%MS_SELECTED_TEXT%") : e.target.value)}
          />
          {customUrl && <div className="field-help">
            <p>{t("ui.actpick.selected_text_hint", { selection: selectionLabel })}</p>
            <button type="button" className="chip chip-var" onClick={() => {
              const el = inputRef.current;
              const start = el?.selectionStart ?? displayValue.length, end = el?.selectionEnd ?? start;
              const variable = selectionLabel;
              onChange((displayValue.slice(0, start) + variable + displayValue.slice(end)).replaceAll(selectionLabel, "%MS_SELECTED_TEXT%"));
              requestAnimationFrame(() => { el?.focus(); el?.setSelectionRange(start + variable.length, start + variable.length); });
            }}>{t("ui.actpick.insert_selected_text")}</button>
          </div>}
          {error}
        </div>
      );
  }
}
