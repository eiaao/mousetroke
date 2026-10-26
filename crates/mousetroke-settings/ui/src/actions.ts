import type { Action, ActionCatalog, ActionPreset, AdvancedAction } from "./types";
import { isMac } from "./platform";
import { isPresetAction } from "./types";
import { currentLang, t, type MsgKey } from "./i18n";

let catalog: ActionCatalog | undefined;
let presets = new Map<string, ActionPreset>();
let commonIds = new Set<string>();
// 搜索索引随界面语言和平台变化（名称/说明/别名是词条，按键写法看当前平台），缓存键一并带上。
let blobCacheKey = "";
let blobs = new Map<string, string>();

export function initActionCatalog(c?: ActionCatalog) {
  catalog = c;
  presets = new Map(c?.presets.map((p) => [p.id, p]));
  commonIds = new Set(c?.commonPresetIds ?? []);
  blobCacheKey = "";
  blobs = new Map();
}

export function actionCatalogCommonIds(): ReadonlySet<string> {
  return commonIds;
}

export function isCommonPreset(id: string): boolean {
  return commonIds.has(id);
}

export function actionCatalog() {
  return catalog;
}

export function findPreset(id: string): ActionPreset | undefined {
  return presets.get(id);
}

export function presetDefinition(action: Action): ActionPreset | undefined {
  return isPresetAction(action) ? presets.get(action.id) : undefined;
}

export function presetSummary(action: Action): string {
  const preset = presetDefinition(action);
  return preset ? presetName(preset) : t("preset.status.unknown");
}

/** 省略号只在选择器中表示需要填写参数，不能成为动作名。 */
export function presetName(p: ActionPreset): string {
  return t(p.name_key as MsgKey).replace(/(?:…|\.\.\.)\s*$/, "");
}

export function presetKeys(p: ActionPreset): string | undefined {
  const impl = isMac() ? p.macos : p.windows;
  return impl.status === "implemented" && impl.value.type === "keys" && typeof impl.value.value === "string" ? impl.value.value : undefined;
}

export function actionKeys(action: Action): string | undefined {
  if (action.type === "key_sequence" && typeof action.keys === "string") return action.keys;
  const p = presetDefinition(action);
  return p ? presetKeys(p) : undefined;
}

export function presetDescription(p: ActionPreset): string {
  const note = p.platform_notes?.[isMac() ? "macos" : "windows"];
  return [t(p.description_key as MsgKey), note ? t(note as MsgKey) : ""].filter(Boolean).join(" ");
}

/** 动作名与手势名重复时，用实际执行类型补充信息。 */
export function presetTypeLabel(p: ActionPreset): string {
  const impl = isMac() ? p.macos : p.windows;
  if (impl.status === "implemented") {
    switch (impl.value.type) {
      case "window": return t("action.window_op");
      case "website": case "search": case "translate": case "selected_url": return t("action.open_url");
      case "launch": case "app": case "open_path": return t("action.launch_app");
      case "system_key": return t("action.system_key");
      case "shell": return t("action.shell");
      case "native": if (p.category === "window") return t("action.window_op");
    }
  }
  return t(`preset.category.${p.category}` as MsgKey);
}

/** 当前平台的实现状态：implemented / unsupported / pending。 */
export function presetPlatformStatus(p: ActionPreset): "implemented" | "unsupported" | "pending" {
  return (isMac() ? p.macos : p.windows).status;
}

/** 能执行与能转为高级动作分别由后端给出，原生能力保留为预置动作。 */
export function presetCanExpand(p: ActionPreset): boolean {
  return presetPlatformStatus(p) === "implemented" && p.advanced_expansion[isMac() ? "macos" : "windows"];
}

/** 不可用时显示的原因词条键（unsupported/pending）。 */
export function presetStatusReason(p: ActionPreset): MsgKey | undefined {
  const impl = isMac() ? p.macos : p.windows;
  return impl.status === "implemented" ? undefined : (impl.value.reason_key as MsgKey);
}

// ---------------------------------------------------------------- 搜索

/** 修饰键的检索别名：用户可能输入 ctrl/control/⌃ 等任意写法。 */
const MOD_ALIASES: Record<string, string[]> = {
  "^": ["ctrl", "control", "⌃"],
  "%": ["alt", "option", "⌥"],
  "+": ["shift", "⇧"],
  "#": ["win", "windows", "cmd", "command", "⌘"],
};

/** 按键序列 → 各组按键的写法变体（修饰键别名 × 键名，如 "ctrl+shift+t"、"⌘w"）。
 *  只服务搜索匹配，不解析按键语法细节；{…} 整体视为一个键。 */
export function keyseqSearchVariants(keys: string): string[] {
  const out: string[] = [];
  let mods: string[] = [];
  let i = 0;
  while (i < keys.length) {
    const ch = keys[i];
    if (MOD_ALIASES[ch]) {
      mods.push(ch);
      i += 1;
      continue;
    }
    let key: string;
    if (ch === "{") {
      const end = keys.indexOf("}", i + 2);
      key = end < 0 ? keys.slice(i + 1) : keys.slice(i + 1, end);
      i = end < 0 ? keys.length : end + 1;
    } else {
      key = ch;
      i += 1;
    }
    let variants = [key.toLowerCase()];
    // 后写的修饰键先组合，保证输出是 "ctrl+shift+t" 这样的常规顺序
    for (const m of [...mods].reverse()) {
      const next: string[] = [];
      for (const v of variants) {
        for (const a of MOD_ALIASES[m]) next.push(`${a}+${v}`, `${a}${v}`);
      }
      variants = next;
    }
    out.push(...variants);
    mods = [];
  }
  return out;
}

/** 归一化：小写并去掉空白，"Ctrl + W" 与 "ctrl+w" 相同。 */
function normalizeQuery(q: string): string {
  return q.trim().toLowerCase().replace(/\s+/g, "");
}

/** 按修饰键集合与完整主键比较，避免 Ctrl+W 命中 Ctrl+Win+Left。 */
function shortcutIdentity(query: string): string | undefined {
  const mainKey = (value: string) => {
    const key = value.replace(/^\{|\}$/g, "");
    return ({ "←": "left", "→": "right", "↑": "up", "↓": "down", "return": "enter", "escape": "esc", "pageup": "pgup", "pagedown": "pgdn" } as Record<string, string>)[key] ?? key;
  };
  let rest = normalizeQuery(query);
  if (/^[\^%+#]/.test(rest)) {
    const native = rest.match(/^([\^%+#]+)(.+)$/);
    if (native) return [...new Set(native[1])].sort().join("") + ":" + mainKey(native[2]);
  }
  const mods = new Set<string>();
  const aliases = Object.entries(MOD_ALIASES).flatMap(([mod, names]) => [mod, ...names].map((name) => ({ mod, name }))).sort((a, b) => b.name.length - a.name.length);
  while (rest) {
    const alias = aliases.find(({ name }) => rest.startsWith(name));
    if (!alias) break;
    mods.add(alias.mod);
    rest = rest.slice(alias.name.length);
    if (rest.startsWith("+")) rest = rest.slice(1);
  }
  if (!rest) return undefined;
  const key = mainKey(rest);
  if (!mods.size && !/^(?:f\d{1,2}|left|right|up|down|home|end|tab|enter|backspace|delete|esc|space|pgup|pgdn|prtsc)$/.test(key)) return undefined;
  if (!/^(?:.|f\d{1,2}|left|right|up|down|home|end|tab|enter|backspace|delete|esc|space|pgup|pgdn|prtsc)$/.test(key)) return undefined;
  return [...mods].sort().join("") + ":" + key;
}

function shortcutIdentities(keys: string): string[] {
  return keyseqSearchVariants(keys).map(shortcutIdentity).filter((v): v is string => !!v);
}

/** 打开时按当前动作定位；带参数的指定入口才直接展开表单。 */
export function pickerInitial(current?: Action, initialId?: string) {
  const preset = initialId ? findPreset(initialId) : current ? presetDefinition(current) : undefined;
  const category: PickerCategory = preset ? categoryOfPreset(preset, isCommonPreset(preset.id)) : "common";
  const active = preset ? Math.max(0, visiblePresets(category, "").findIndex((p) => p.id === preset.id)) : 0;
  return { preset, category, active };
}

/** 文案检索串；快捷键单独按完整组合匹配，不混入子串索引。 */
export function presetSearchBlob(p: ActionPreset): string {
  const cacheKey = `${currentLang()}|${isMac() ? "mac" : "win"}`;
  if (blobCacheKey !== cacheKey) {
    blobCacheKey = cacheKey;
    blobs = new Map();
  }
  const hit = blobs.get(p.id);
  if (hit !== undefined) return hit;
  // 带修饰键的别名由完整快捷键匹配处理，避免 Ctrl+W 成为 Ctrl+Win+Left 的子串。
  const aliases = t(p.aliases_key as MsgKey).split(/\s+/).filter((word) => {
    const identity = shortcutIdentity(word);
    return !identity || identity.startsWith(":");
  }).join(" ");
  const parts = [t(p.name_key as MsgKey), t(p.description_key as MsgKey), aliases];
  const blob = parts.join(" ").toLowerCase().replace(/\s+/g, "");
  blobs.set(p.id, blob);
  return blob;
}

export function matchPreset(p: ActionPreset, query: string): boolean {
  const q = normalizeQuery(query);
  if (!q) return true;
  const keys = presetKeys(p);
  const shortcut = shortcutIdentity(q);
  return presetSearchBlob(p).includes(q) || (!!shortcut && !!keys && shortcutIdentities(keys).includes(shortcut));
}

// ---------------------------------------------------------------- 分类与参数

/** 注册表分类（选择器分类 chips 的中间段）。 */
export const PRESET_CATEGORIES = ["browser", "edit", "text", "window", "desktop", "media", "open", "mousetroke"] as const;

/** 选择器分类："常用" + 注册表分类 + "高级"。 */
export type PickerCategory = "common" | (typeof PRESET_CATEGORIES)[number] | "advanced";

export const CATEGORY_ORDER: PickerCategory[] = ["common", ...PRESET_CATEGORIES];

/** 某个分类下可见的动作；搜索时忽略分类，匹配所有动作。 */
export function visiblePresets(category: PickerCategory, query: string): ActionPreset[] {
  const all = catalog?.presets ?? [];
  const q = normalizeQuery(query);
  if (q) return all.filter((p) => matchPreset(p, q));
  if (category === "common") return all.filter((p) => commonIds.has(p.id));
  if (category === "advanced") return [];
  return all.filter((p) => p.category === category);
}

/** "高级"分类的底层动作入口（弹出菜单里不能再嵌套弹出菜单）。 */
export const ADVANCED_PICKS: { type: AdvancedAction["type"]; nameKey: MsgKey; descKey: MsgKey }[] = [
  { type: "key_sequence", nameKey: "action.key_sequence", descKey: "ui.actpick.adv_key_sequence" },
  { type: "shell", nameKey: "action.shell", descKey: "ui.actpick.adv_shell" },
  { type: "lua", nameKey: "action.lua", descKey: "ui.actpick.adv_lua" },
  { type: "menu", nameKey: "action.menu", descKey: "ui.actpick.adv_menu" },
];

export function matchAdvancedPick(nameKey: MsgKey, descKey: MsgKey, query: string): boolean {
  const q = normalizeQuery(query);
  return !q || normalizeQuery(`${t(nameKey)} ${t(descKey)}`).includes(q);
}

/** 参数初始值：注册表默认值（搜索引擎默认值跟界面语言，切换语言后目录会刷新）。 */
export function presetParamValues(p: ActionPreset, current?: Action): Record<string, string> {
  const defaults = catalog?.defaults?.[p.id] ?? {};
  const out: Record<string, string> = {};
  for (const param of p.params) {
    const v = current && isPresetAction(current) && current.id === p.id && typeof current.params?.[param.name] === "string" ? current.params[param.name] : defaults[param.name];
    out[param.name] = typeof v === "string" ? v : "";
  }
  return out;
}

export function visibleParams(p: ActionPreset, values: Record<string, string>): ActionPreset["params"] {
  return p.params.filter((param) => !param.visible_when || values[param.visible_when.param] === param.visible_when.equals);
}

/** 必填参数：默认值是空串的文字类参数（选项类总有值）。 */
export function missingRequiredParams(p: ActionPreset, values: Record<string, string>): string[] {
  return visibleParams(p, values)
    .filter((param) => param.default.kind === "literal" && param.default.value === "" && !(param.kind.type === "multiline_text" ? values[param.name] : (values[param.name] ?? "").trim()))
    .map((param) => param.name);
}

/** 保存时也清理旧配置中的隐藏参数；未知字段和未知动作原样保留。 */
export function persistedAction(action: Action): Action {
  if (action.type === "menu" && Array.isArray(action.items)) {
    return { ...action, items: action.items.map((item) => ({ ...item, action: persistedAction(item.action) })) };
  }
  const preset = presetDefinition(action);
  if (!preset || !isPresetAction(action) || !action.params) return action;
  const values = presetParamValues(preset, action);
  const params = { ...action.params };
  const visible = new Set(visibleParams(preset, values).map((param) => param.name));
  for (const param of preset.params) if (!visible.has(param.name)) delete params[param.name];
  return { ...action, params };
}

/** 路径与网址去掉两端空格；输入文字保留缩进、换行与尾部空格。 */
export function cleanPresetParams(p: ActionPreset, values: Record<string, string>): Record<string, string> {
  const out: Record<string, string> = {};
  for (const param of visibleParams(p, values)) {
    const v = param.kind.type === "multiline_text" ? (values[param.name] ?? "") : (values[param.name] ?? "").trim();
    if (v) out[param.name] = v;
  }
  return out;
}

// ---------------------------------------------------------------- 选择器交互（纯逻辑，组件与测试共用）

/** 上下键移动高亮：在行数内循环，跳过不可选行，空列表不动。 */
export function pickerNav(active: number, rows: number, key: string, selectable: boolean[]): number | null {
  if (rows === 0) return null;
  const step = key === "ArrowDown" ? 1 : key === "ArrowUp" ? -1 : 0;
  if (step === 0) return null;
  let pos = active;
  for (let n = 0; n < rows; n++) {
    pos = (pos + step + rows) % rows;
    if (selectable[pos]) return pos;
  }
  return null;
}

/** 回车/点击一行的结果：灰显不可选；无参数直接选中；带参数的展开表单，再确认一次才选中。 */
export function presetPickResult(p: ActionPreset, formOpenForIt: boolean, values: Record<string, string>): "blocked" | "pick" | "form" | "apply-form" | "invalid" {
  if (presetPlatformStatus(p) !== "implemented") return "blocked";
  if (p.params.length === 0) return "pick";
  if (!formOpenForIt) return "form";
  return missingRequiredParams(p, values).length ? "invalid" : "apply-form";
}

/** 动作在当前平台的检索分类（用于"更换"时定位分类 chips）。 */
export function categoryOfPreset(p: ActionPreset, common: boolean): PickerCategory {
  return common ? "common" : p.category;
}

/** choice 参数的显示文案（选项文案，不是内部值）。 */
export function presetParamDisplay(p: ActionPreset, name: string, value: string): string {
  const param = p.params.find((x) => x.name === name);
  if (param?.kind.type === "choice") {
    const choice = param.kind.choices.find((c) => c.value === value);
    if (choice) return t(choice.label_key as MsgKey);
  }
  return value;
}
