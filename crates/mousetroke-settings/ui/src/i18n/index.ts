import { isMac } from "../platform";
// i18n 运行时（设计见 docs/I18N-DESIGN.md §4）。
//
// - 词条与 Rust 侧共用仓库根目录的 i18n/*.json（vite 原生支持 JSON 导入），
//   十种语言全部内嵌（gzip 后约 100 KB，设置界面按需打开，不值得按需加载）。
// - 键名由 `typeof zh` 推导：写错键名 tsc 报错；en-US 标注为 Record<keyof Catalog, string>，
//   少键编译不过，多出的键由 Rust 侧的目录一致性测试抓出来。
// - 语言状态是模块级小 store（useSyncExternalStore 订阅）；不要把 t 一层层当 props 传，
//   在根组件 useLang() 一次，切换语言时整棵树重渲染。
import { createElement, Fragment, useSyncExternalStore } from "react";
import zh from "../../../../../i18n/zh-CN.json";
import en from "../../../../../i18n/en-US.json";
import es from "../../../../../i18n/es-ES.json";
import fr from "../../../../../i18n/fr-FR.json";
import de from "../../../../../i18n/de-DE.json";
import ja from "../../../../../i18n/ja-JP.json";
import pt from "../../../../../i18n/pt-BR.json";
import ru from "../../../../../i18n/ru-RU.json";
import it from "../../../../../i18n/it-IT.json";
import ko from "../../../../../i18n/ko-KR.json";

type Catalog = typeof zh; // zh-CN 是源语言，键集合的权威来源

/** 去掉复数后缀（CLDR：_one / _other / _few / _many）后的基础键。 */
type StripPlural<K> = K extends `${infer B}_one` | `${infer B}_other` | `${infer B}_few` | `${infer B}_many` ? B : K;
export type MsgKey = StripPlural<keyof Catalog>;

export type Lang =
  | "zh-CN"
  | "en-US"
  | "es-ES"
  | "fr-FR"
  | "de-DE"
  | "ja-JP"
  | "pt-BR"
  | "ru-RU"
  | "it-IT"
  | "ko-KR"
  | "qps-ploc";
/** 设置值：auto（跟随系统）或具体语言。 */
export type LangSetting = "auto" | Exclude<Lang, "qps-ploc">;

/** 界面语言下拉框的顺序（语言自名永远用它自己的语言显示）。 */
export const LANGS: readonly Exclude<Lang, "qps-ploc">[] = [
  "zh-CN", "en-US", "es-ES", "fr-FR", "de-DE", "ja-JP", "pt-BR", "ru-RU", "it-IT", "ko-KR",
];

// en-US：少键在这里就编译不过（其余语言由 Rust 侧目录测试保证）
const enFull: Record<keyof Catalog, string> = en;

const BASE: Record<Exclude<Lang, "qps-ploc">, Record<string, string>> = {
  "zh-CN": zh,
  "en-US": enFull,
  "es-ES": es,
  "fr-FR": fr,
  "de-DE": de,
  "ja-JP": ja,
  "pt-BR": pt,
  "ru-RU": ru,
  "it-IT": it,
  "ko-KR": ko,
};

// ---------------------------------------------------------------- 语言状态

let resolved: Lang = "zh-CN";
let setting: LangSetting = "auto";
let systemLangs: string[] = [];
const listeners = new Set<() => void>();

const emit = () => listeners.forEach((f) => f());
const subscribe = (f: () => void) => (listeners.add(f), () => listeners.delete(f));

function applyDocLang() {
  // CSS 字体栈与换行规则依赖 document lang
  document.documentElement.lang = resolved === "qps-ploc" ? "en" : resolved;
}

/** 主标签 → 基准语言；地区变体归并（zh-TW 词条就绪前 → zh-CN）。 */
const PRIMARY: Partial<Record<string, Lang>> = {
  zh: "zh-CN", en: "en-US", es: "es-ES", fr: "fr-FR", de: "de-DE", ja: "ja-JP", pt: "pt-BR", ru: "ru-RU", it: "it-IT", ko: "ko-KR",
};

function fromTag(s: string): Lang | null {
  const [p] = s.trim().toLowerCase().split("-");
  return PRIMARY[p] ?? null;
}

function resolveLang(setting: LangSetting, system: string[]): Lang {
  if (setting !== "auto") return setting;
  for (const tag of system) {
    const l = fromTag(tag);
    if (l) return l;
  }
  return "en-US";
}

/** 启动时初始化（App 在 init_state 之后调用一次）。`url` 用于冒烟测试强制语言。 */
export function initLang(o: { setting?: string; systemLangs?: string[]; url?: string }): Lang {
  systemLangs = o.systemLangs ?? [];
  const s = o.setting ?? "auto";
  setting = s === "auto" || fromTag(s) ? (s as LangSetting) : "auto";
  resolved = resolveLang(setting, systemLangs);
  // URL 参数强制（qps-ploc 只在这里认：设置值不暴露伪语言）
  const q = new URLSearchParams(o.url ?? location.search).get("lang");
  const forced = q === "qps-ploc" ? ("qps-ploc" as const) : q !== null ? fromTag(q) : null;
  if (forced) resolved = forced;
  applyDocLang();
  emit();
  return resolved;
}

/** 当前语言（React 组件里用 useLang 订阅）。 */
export function currentLang(): Lang {
  return resolved;
}

/** 当前设置值（auto 或具体语言），写回 config.settings.language 用。 */
export function langSetting(): LangSetting {
  return setting;
}

/** 订阅当前语言。放在界面根组件：语言切换时整棵树重渲染。 */
export function useLang(): [Lang, (l: LangSetting) => void] {
  const lang = useSyncExternalStore(subscribe, currentLang);
  return [lang, setLangSetting];
}

function setLangSetting(l: LangSetting) {
  setting = l;
  resolved = resolveLang(l, systemLangs);
  applyDocLang();
  emit();
}

// ---------------------------------------------------------------- 词条

let pseudo: Record<string, string> | null = null;

function catalogOf(lang: Lang): Record<string, string> {
  if (lang === "qps-ploc") return (pseudo ??= pseudoCatalog(enFull));
  return BASE[lang];
}

function lookup(lang: Lang, key: string): string | undefined {
  return catalogOf(lang)[key] ?? catalogOf("zh-CN")[key];
}

function interpolate(templ: string, params?: Record<string, string | number>): string {
  if (!params) return templ;
  return templ.replace(/\{([a-z_][a-z0-9_]*)\}/g, (all, name: string) =>
    params[name] !== undefined ? String(params[name]) : all,
  );
}

/** 取词条。缺失回退 zh-CN，再缺失返回键名。`count` 按当前语言的复数规则选变体，
 *  回退顺序 `_<类别>` → `_other` → 基础键。 */
export function t(key: MsgKey, params?: Record<string, string | number>): string {
  if (isMac() && `mac.${key}` in zh) key = `mac.${key}` as MsgKey;
  const lang = resolved;
  let templ: string | undefined;
  if (params && params.count != null) {
    const n = Number(params.count);
    if (Number.isFinite(n)) {
      templ = lookup(lang, `${key}_${pluralOf(lang, n)}`) ?? lookup(lang, `${key}_other`);
    }
  }
  templ ??= lookup(lang, key) ?? key;
  return interpolate(templ, params);
}

/** CLDR 基数复数类别，交给 Intl（qps-ploc 借用英文规则）。 */
function pluralOf(lang: Lang, n: number): string {
  const l = lang === "qps-ploc" ? "en" : lang;
  return new Intl.PluralRules(l).select(n);
}

/** 渲染带 `**强调**` 的词条（引导、横幅这类富文本），其余一律用 t()。 */
export function T({ k, params }: { k: MsgKey; params?: Record<string, string | number> }) {
  const parts = t(k, params).split(/\*\*(.+?)\*\*/gs);
  return createElement(
    Fragment,
    null,
    ...parts.map((p, i) => (i % 2 ? createElement("b", { key: i }, p) : p)),
  );
}

/** 取词条，缺失（或不是合法键）时返回原值。 */
export function tryT(key: string): string {
  return key in zh ? t(key as MsgKey) : key;
}

/** 内置名称的显示名：有 name_key 且词条存在 → 当前语言译文；否则用户起的名字。
 *  与 mousetroke_core::i18n::display_name 同一逻辑。 */
export function displayName(name: string, nameKey?: string): string {
  return nameKey && nameKey in zh ? t(nameKey as MsgKey) : name;
}

// ---------------------------------------------------------------- 伪语言

// 与 Rust 侧 i18n::pseudo 同一规则：字母加重音、首尾 [!! … !!]、按长度补约 40% 填充、
// 占位符 {name} 原样保留。由 en-US 生成（拉丁字母才能加重音，长度也接近最长的真实语言）。
function pseudoCatalog(src: Record<string, string>): Record<string, string> {
  const out: Record<string, string> = {};
  for (const [k, v] of Object.entries(src)) out[k] = pseudoText(v);
  return out;
}

const ACCENT: Record<string, string> = {
  a: "á", e: "é", i: "í", o: "ó", u: "ú", y: "ý", n: "ñ", c: "ç",
  A: "Á", E: "É", I: "Í", O: "Ó", U: "Ú", Y: "Ý", N: "Ñ", C: "Ç",
};

function pseudoText(text: string): string {
  const accent = (s: string) => s.replace(/[aeiouyncAEIOUYNC]/g, (c) => ACCENT[c] ?? c);
  // 占位符整段跳过
  let body = "";
  let rest = text;
  for (;;) {
    const i = rest.indexOf("{");
    if (i < 0) break;
    body += accent(rest.slice(0, i));
    const j = rest.indexOf("}", i);
    if (j < 0) {
      body += accent(rest.slice(i));
      rest = "";
      break;
    }
    body += rest.slice(i, j + 1);
    rest = rest.slice(j + 1);
  }
  body += accent(rest);
  const pad = Math.ceil(body.length * 0.4);
  const filler = Array.from({ length: pad }, (_, i) => String(i % 10)).join("");
  return `[!! ${body} ${filler} !!]`;
}
