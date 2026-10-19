import type { EngineStatus, Settings } from "../types";
import { KeysInput } from "./KeysInput";
import { LANGS, t, useLang, type Lang, type LangSetting, type MsgKey } from "../i18n";

/** #RRGGBBAA ⇄ (#RRGGBB, alpha 0..1) */
function splitColor(c: string): [string, number] {
  const hex = c.replace("#", "");
  const rgb = `#${hex.slice(0, 6).padEnd(6, "0")}`;
  const a = hex.length >= 8 ? parseInt(hex.slice(6, 8), 16) / 255 : 1;
  return [rgb, a];
}
function joinColor(rgb: string, a: number): string {
  return `${rgb}${Math.round(a * 255).toString(16).padStart(2, "0")}`.toUpperCase();
}

function ColorField({ label, value, onChange }: { label: string; value: string; onChange: (v: string) => void }) {
  const [rgb, a] = splitColor(value);
  return (
    <div className="pref-row">
      <span className="pref-label">{label}</span>
      <span className="color-field">
        <span className="swatch" style={{ background: `rgba(${parseInt(rgb.slice(1, 3), 16)},${parseInt(rgb.slice(3, 5), 16)},${parseInt(rgb.slice(5, 7), 16)},${a})` }}>
          <input type="color" value={rgb} aria-label={`${label} · ${t("ui.prefs.color_aria")}`} onChange={(e) => onChange(joinColor(e.target.value, a))} />
        </span>
        <input type="range" min={0.1} max={1} step={0.05} value={a} aria-label={`${label} · ${t("ui.prefs.opacity_aria")}`} onChange={(e) => onChange(joinColor(rgb, Number(e.target.value)))} />
        <span className="num">{Math.round(a * 100)}%</span>
      </span>
    </div>
  );
}

function Toggle({ label, help, checked, onChange }: { label: string; help?: string; checked: boolean; onChange: (v: boolean) => void }) {
  return (
    <label className="pref-row toggle-row">
      <span className="pref-label">
        {label}
        {help && <small>{help}</small>}
      </span>
      <input type="checkbox" className="switch" checked={checked} onChange={(e) => onChange(e.target.checked)} />
    </label>
  );
}

function Slider({ label, help, value, min, max, step, unit, onChange }: { label: string; help?: string; value: number; min: number; max: number; step: number; unit: string; onChange: (v: number) => void }) {
  return (
    <div className="pref-row">
      <span className="pref-label">
        {label}
        {help && <small>{help}</small>}
      </span>
      <span className="slider">
        <input type="range" min={min} max={max} step={step} value={value} onChange={(e) => onChange(Number(e.target.value))} />
        <span className="num">
          {value}
          {unit}
        </span>
      </span>
    </div>
  );
}

/** 轨迹与标签的实时预览（手势名固定在屏幕中下方）。 */
function TrailPreview({ s }: { s: Settings }) {
  const [rgb, a] = splitColor(s.trail.color_normal);
  const [lrgb, la] = splitColor(s.label.color_normal);
  return (
    <div className="trail-preview" aria-hidden>
      <svg viewBox="0 0 240 90" width="240" height="90">
        {s.trail.show && (
          <polyline points="70,50 70,12 170,12 170,50" fill="none" stroke={rgb} strokeOpacity={a} strokeWidth={Math.max(1, s.trail.width)} strokeLinecap="round" strokeLinejoin="round" />
        )}
        {s.label.show && (
          <g>
            <rect x="76" y="60" width="88" height="24" rx="7" fill="rgba(20,24,28,.78)" />
            <text x="120" y="77" textAnchor="middle" fontSize="13" fontWeight="600" fill={lrgb} fillOpacity={la}>
              {t("ui.prefs.preview_label")}
            </text>
          </g>
        )}
      </svg>
    </div>
  );
}

/** 语言选项用它自己的语言显示名称（English 永远显示 English）。 */
const TAG_TO_NAME: Record<Lang, MsgKey> = {
  "zh-CN": "lang.name.zh_cn",
  "en-US": "lang.name.en_us",
  "es-ES": "lang.name.es_es",
  "fr-FR": "lang.name.fr_fr",
  "de-DE": "lang.name.de_de",
  "ja-JP": "lang.name.ja_jp",
  "pt-BR": "lang.name.pt_br",
  "ru-RU": "lang.name.ru_ru",
  "it-IT": "lang.name.it_it",
  "ko-KR": "lang.name.ko_kr",
  "qps-ploc": "lang.name.en_us",
};

function langSelfName(l: Lang): string {
  return t(TAG_TO_NAME[l]);
}

export function Prefs({
  settings,
  onChange,
  engine,
  onAutostart,
}: {
  settings: Settings;
  onChange: (f: (s: Settings) => Settings, coalesce?: string) => void;
  engine: EngineStatus;
  onAutostart: (on: boolean) => void;
}) {
  const s = settings;
  const [lang, setLang] = useLang();
  const onLanguage = (l: LangSetting) => {
    setLang(l);
    onChange((x) => ({ ...x, language: l }));
  };
  return (
    <section className="prefs" aria-label={t("ui.prefs.title")}>
      <h1>{t("ui.prefs.title")}</h1>

      <div className="pref-group">
        <h2>{t("ui.prefs.group_startup")}</h2>
        <div className="pref-row">
          <span className="pref-label">{t("ui.prefs.language")}</span>
          <span className="pref-control">
            <select className="lang-select" value={s.language ?? "auto"} onChange={(e) => onLanguage(e.target.value as LangSetting)}>
              <option value="auto">{t("lang.auto", { lang: langSelfName(lang) })}</option>
              {LANGS.map((l) => (
                <option key={l} value={l}>{langSelfName(l)}</option>
              ))}
            </select>
          </span>
        </div>
        <Toggle
          label={t("ui.prefs.autostart")}
          help={engine.exeFound ? undefined : t("ui.prefs.autostart_help")}
          checked={engine.autostart}
          onChange={onAutostart}
        />
        <Toggle label={t("ui.prefs.show_tray")} checked={s.show_tray} onChange={(v) => onChange((x) => ({ ...x, show_tray: v }))} />
        <div className="update-preference">
          <Toggle label={t("ui.prefs.update_check")} help={t("ui.prefs.update_check_help")} checked={s.update_check ?? true} onChange={(v) => onChange((x) => ({ ...x, update_check: v }))} />
        </div>
        <div className="pref-row">
          <span className="pref-label">
            {t("ui.prefs.pause_hotkey")}
            <small>{t("ui.prefs.pause_hotkey_help")}</small>
          </span>
          <span className="pref-control">
            <KeysInput single value={s.pause_hotkey} onChange={(v) => onChange((x) => ({ ...x, pause_hotkey: v }), "hotkey")} />
          </span>
        </div>
      </div>

      <div className="pref-group">
        <h2>{t("ui.prefs.group_recognition")}</h2>
        <Slider
          label={t("ui.prefs.drag_timeout")}
          help={t("ui.prefs.drag_timeout_help")}
          value={s.drag_timeout_ms}
          min={150}
          max={1500}
          step={50}
          unit=" ms"
          onChange={(v) => onChange((x) => ({ ...x, drag_timeout_ms: v }), "drag")}
        />
        <Slider
          label={t("ui.prefs.stroke_px")}
          help={t("ui.prefs.stroke_px_help")}
          value={s.min_stroke_px}
          min={10}
          max={80}
          step={2}
          unit=" px"
          onChange={(v) => onChange((x) => ({ ...x, min_stroke_px: v }), "stroke")}
        />
        <div className="pref-row">
          <span className="pref-label">{t("ui.prefs.target_mode")}</span>
          <div className="segmented" role="radiogroup">
            {(
              [
                ["under_cursor", "ui.prefs.target_under_cursor"],
                ["focused", "ui.prefs.target_focused"],
              ] as const
            ).map(([v, k]) => (
              <button key={v} type="button" role="radio" aria-checked={s.target_mode === v} className={s.target_mode === v ? "on" : ""} onClick={() => onChange((x) => ({ ...x, target_mode: v }))}>
                {t(k)}
              </button>
            ))}
          </div>
        </div>
      </div>

      <div className="pref-group">
        <h2>{t("ui.prefs.group_appearance")}</h2>
        <TrailPreview s={s} />
        <Toggle label={t("ui.prefs.show_trail")} checked={s.trail.show} onChange={(v) => onChange((x) => ({ ...x, trail: { ...x.trail, show: v } }))} />
        <Slider label={t("ui.prefs.trail_width")} value={s.trail.width} min={1} max={10} step={0.5} unit=" px" onChange={(v) => onChange((x) => ({ ...x, trail: { ...x.trail, width: v } }), "tw")} />
        <ColorField label={t("ui.prefs.trail_color")} value={s.trail.color_normal} onChange={(v) => onChange((x) => ({ ...x, trail: { ...x.trail, color_normal: v } }), "tc")} />
        <ColorField label={t("ui.prefs.trail_matched")} value={s.trail.color_matched} onChange={(v) => onChange((x) => ({ ...x, trail: { ...x.trail, color_matched: v } }), "tm")} />
        <Toggle label={t("ui.prefs.fade_out")} checked={s.trail.fade_out} onChange={(v) => onChange((x) => ({ ...x, trail: { ...x.trail, fade_out: v } }))} />
        <Toggle label={t("ui.prefs.show_label")} checked={s.label.show} onChange={(v) => onChange((x) => ({ ...x, label: { ...x.label, show: v } }))} />
        <ColorField label={t("ui.prefs.label_color")} value={s.label.color_normal} onChange={(v) => onChange((x) => ({ ...x, label: { ...x.label, color_normal: v } }), "lc")} />
      </div>
    </section>
  );
}
