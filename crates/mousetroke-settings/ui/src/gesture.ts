import { isMac } from "./platform";
import type { Action, Binding, KeyTag, MouseButton, Step } from "./types";
import { t } from "./i18n";
import type { MsgKey } from "./i18n";

/** 键盘事件中录制需要的字段（DOM 与 React 事件通用）。 */
type KeyLike = Pick<KeyboardEvent, "code" | "ctrlKey" | "altKey" | "shiftKey" | "metaKey">;

// ---------------------------------------------------------------- 边角

/** 与 mousetroke-core::edge 一致的位标志编码。 */
export const EDGE = { top: 1, right: 2, bottom: 4, left: 8, rightTop: 3, rightBottom: 6, leftTop: 9, leftBottom: 12 } as const;

/** 边/角名（词条键就是位标志数字，与 mousetroke-core::edge 一致）。 */
export function edgeName(edge: number): string {
  return t(`edge.${edge}` as MsgKey);
}

// ---------------------------------------------------------------- 键

/** 鼠标键名（按需取词条：模块加载时就求值的常量表不会随语言切换更新）。 */
export function buttonName(b: MouseButton): string {
  return t(`button.${b}` as MsgKey);
}

/** 动词短语单独翻译，避免把独立按钮名直接塞进需要变格或冠词的句子。 */
export function buttonPhrase(b: MouseButton, action: "hold" | "click"): string {
  return t(`ui.pattern.${action}_btn.${b}`);
}

/** VK → 显示名。通用符号（Ctrl、F5、←…）不进词条；描述性的（空格、小键盘）要进。 */
export function vkName(vk: number): string {
  if (isMac()) {
    if ([0x5b, 0x5c].includes(vk)) return t("mac.key.command");
    if ([0x10, 0xa0, 0xa1].includes(vk)) return t("mac.key.shift");
    if ([0x11, 0xa2, 0xa3].includes(vk)) return t("mac.key.control");
    if ([0x12, 0xa4, 0xa5].includes(vk)) return t("mac.key.option");
    const macNamed: Partial<Record<number, string>> = { 0x08: "⌫", 0x0d: "↩", 0x1b: "⎋", 0x25: "←", 0x26: "↑", 0x27: "→", 0x28: "↓", 0x2e: "⌦", 0x09: "⇥" };
    if (macNamed[vk]) return macNamed[vk]!;
  }
  const named: Partial<Record<number, MsgKey>> = {
    0x20: "key.space", 0x5c: "key.win_right", 0xa0: "key.shift_left", 0xa1: "key.shift_right",
    0xa2: "key.ctrl_left", 0xa3: "key.ctrl_right", 0xa4: "key.alt_left", 0xa5: "key.alt_right",
  };
  if (named[vk]) return t(named[vk]!);
  if ((vk >= 0x30 && vk <= 0x39) || (vk >= 0x41 && vk <= 0x5a)) return String.fromCharCode(vk);
  if (vk >= 0x70 && vk <= 0x87) return `F${vk - 0x6f}`;
  if (vk >= 0x60 && vk <= 0x69) return t("key.numpad", { n: vk - 0x60 });
  return t("key.vk", { vk });
}

export function keyTagName(tag: KeyTag): string {
  const [kind, val] = tag.split(":");
  if (kind === "mouse") return val in { left: 1, right: 1, middle: 1, x1: 1, x2: 1 } ? buttonName(val as MouseButton) : val;
  if (kind === "kb") return vkName(Number(val));
  if (kind === "wheel") return Number(val) < 0 ? t("key.wheel_up") : t("key.wheel_down");
  return tag;
}

/** KeyboardEvent.code → VK（录制"附加键"用）。 */
export function codeToVk(e: KeyLike): number | null {
  const c = e.code;
  if (isMac() && c === "NumpadEnter") return 0x0d;
  if (/^Key[A-Z]$/.test(c)) return c.charCodeAt(3);
  if (/^Digit[0-9]$/.test(c)) return c.charCodeAt(5);
  if (/^F([1-9]|1[0-9]|2[0-4])$/.test(c)) return 0x6f + Number(c.slice(1));
  if (/^Numpad[0-9]$/.test(c)) return 0x60 + Number(c.slice(6));
  const map: Record<string, number> = {
    Backspace: 0x08, Tab: 0x09, Enter: 0x0d, Escape: 0x1b, Space: 0x20, PageUp: 0x21, PageDown: 0x22,
    End: 0x23, Home: 0x24, ArrowLeft: 0x25, ArrowUp: 0x26, ArrowRight: 0x27, ArrowDown: 0x28,
    Insert: 0x2d, Delete: 0x2e, ShiftLeft: 0xa0, ShiftRight: 0xa1, ControlLeft: 0xa2, ControlRight: 0xa3,
    AltLeft: 0xa4, AltRight: 0xa5, MetaLeft: 0x5b, MetaRight: 0x5c, Semicolon: 0xba, Equal: 0xbb,
    Comma: 0xbc, Minus: 0xbd, Period: 0xbe, Slash: 0xbf, Backquote: 0xc0, BracketLeft: 0xdb,
    Backslash: 0xdc, BracketRight: 0xdd, Quote: 0xde, CapsLock: 0x14, PrintScreen: 0x2c, Pause: 0x13,
  };
  return map[c] ?? null;
}

/**
 * 键盘组合 → 按键序列语法（keyseq）。只按了修饰键时返回 null。
 * 例：Ctrl+Shift+T → `^+t`，Alt+F4 → `%{F4}`，Win+D → `#d`。
 */
export function comboToKeyseq(e: KeyLike): string | null {
  const c = e.code;
  if (/^(Shift|Control|Alt|Meta)(Left|Right)$/.test(c)) return null;
  let key: string | null = null;
  if (/^Key[A-Z]$/.test(c)) key = c.slice(3).toLowerCase();
  else if (/^Digit[0-9]$/.test(c)) key = c.slice(5);
  else if (/^F([1-9]|1[0-9]|2[0-4])$/.test(c)) key = `{${c}}`;
  else if (/^Numpad[0-9]$/.test(c)) key = `{N${c.slice(6)}}`;
  else if (isMac() && c === "NumpadEnter") key = "{ENTER}";
  else {
    const map: Record<string, string> = {
      Backspace: "{BACKSPACE}", Tab: "{TAB}", Enter: "{ENTER}", Escape: "{ESC}", Space: "{SPACE}",
      PageUp: "{PGUP}", PageDown: "{PGDN}", End: "{END}", Home: "{HOME}", ArrowLeft: "{LEFT}",
      ArrowUp: "{UP}", ArrowRight: "{RIGHT}", ArrowDown: "{DOWN}", Insert: "{INS}", Delete: "{DEL}",
      PrintScreen: "{PRTSC}", CapsLock: "{CAPSLOCK}", Semicolon: ";", Equal: "=", Comma: ",", Minus: "-",
      Period: ".", Slash: "/", Backquote: "`", BracketLeft: "[", Backslash: "\\", BracketRight: "]", Quote: "'",
    };
    key = map[c] ?? null;
  }
  if (!key) return null;
  const mods = (e.ctrlKey ? "^" : "") + (e.altKey ? "%" : "") + (e.shiftKey ? "+" : "") + (e.metaKey ? "#" : "");
  return mods + key;
}

/** 把按键序列拆成可读的键帽组（仅用于展示，不做完整解析）。
 *  只翻译"描述性"的键名（空格、滚轮、鼠标键、等待）；Ctrl、Enter、F5 等保持原样。 */
export function keyseqToCaps(keys: string): string[][] {
  const out: string[][] = [];
  let mods: string[] = [];
  const modName: Record<string, string> = isMac()
    ? { "^": t("mac.key.control"), "%": t("mac.key.option"), "+": t("mac.key.shift"), "#": t("mac.key.command") }
    : { "^": "Ctrl", "%": "Alt", "+": "Shift", "#": "Win" };
  const nice = (k: string) => {
    const u = k.toUpperCase();
    const named: Partial<Record<string, MsgKey>> = {
      SPACE: "key.space",
      MOUSE_LEFT: "button.left", MOUSE_RIGHT: "button.right", MOUSE_MIDDLE: "button.middle",
      MEDIA_NEXT: "syskey.next", MEDIA_PREV: "syskey.previous",
      MEDIA_PLAY_PAUSE: "syskey.play_pause", MEDIA_PLAY: "syskey.play_pause",
      VOLUME_UP: "syskey.volume_up", VOLUME_DOWN: "syskey.volume_down", VOLUME_MUTE: "syskey.mute",
    };
    if (isMac()) {
      const macNames: Record<string, string> = { WIN: t("mac.key.command"), LWIN: t("mac.key.command"), RWIN: t("mac.key.command"), ALT: t("mac.key.option"), CTRL: t("mac.key.control"), SHIFT: t("mac.key.shift"), LEFT: "←", RIGHT: "→", UP: "↑", DOWN: "↓", BACKSPACE: "⌫", DEL: "⌦", DELETE: "⌦", ENTER: "↩", RETURN: "↩", TAB: "⇥", ESC: "⎋" };
      if (macNames[u]) return macNames[u];
    }
    if (named[u]) return t(named[u]!);
    if (u.startsWith("SLEEP")) return t("keyseq.wait_ms", { ms: u.slice(5).trim() });
    if (k === "\t") return "Tab";
    return k.length === 1 ? k.toUpperCase() : k;
  };
  let i = 0;
  while (i < keys.length) {
    const ch = keys[i];
    if (modName[ch]) {
      mods.push(modName[ch]);
      i++;
    } else if (ch === "{") {
      const end = keys.indexOf("}", i + 2);
      const body = end < 0 ? keys.slice(i + 1) : keys.slice(i + 1, end);
      // {键名 次数}：音量+ ×2（SLEEP 的参数是毫秒，不是次数）
      const m = /^(\S+)\s+(\d+)$/.exec(body);
      out.push([...mods, m && !/^sleep$/i.test(m[1]) ? `${nice(m[1])}${Number(m[2]) > 1 ? ` ×${m[2]}` : ""}` : nice(body)]);
      mods = [];
      i = end < 0 ? keys.length : end + 1;
    } else if (ch === "(") {
      // 找匹配的右括号（支持嵌套，跳过 {…} 内的字符）
      let depth = 0, end = -1;
      for (let j = i; j < keys.length; j++) {
        const c = keys[j];
        if (c === "{") {
          const close = keys.indexOf("}", j + 2);
          if (close < 0) break;
          j = close;
        } else if (c === "(") depth++;
        else if (c === ")" && --depth === 0) {
          end = j;
          break;
        }
      }
      const inner = end < 0 ? keys.slice(i + 1) : keys.slice(i + 1, end);
      for (const g of keyseqToCaps(inner)) out.push([...mods, ...g]);
      mods = [];
      i = end < 0 ? keys.length : end + 1;
    } else {
      out.push([...mods, nice(ch)]);
      mods = [];
      i++;
    }
  }
  return out;
}

// ---------------------------------------------------------------- 模式

export type TriggerSpec = { kind: "button"; button: MouseButton } | { kind: "edge"; edge: number };

/** 触发之后的有序步骤；保留非 press 步骤的原始编码，编辑其他步骤时无损往返。 */
export type PatternItem =
  | { kind: "key"; key: KeyTag; step?: Extract<Step, { type: "edge" | "wheel" }> }
  | { kind: "trace"; points: number[] }
  | { kind: "shape"; templates: number[][] };

export interface PatternModel {
  trigger: TriggerSpec;
  items: PatternItem[];
}

export function decompose(pattern: Step[]): PatternModel {
  const [first, ...rest] = pattern;
  let trigger: TriggerSpec = { kind: "button", button: "right" };
  if (first?.type === "edge") trigger = { kind: "edge", edge: first.edge };
  else if (first?.type === "press" && first.key.startsWith("mouse:"))
    trigger = { kind: "button", button: first.key.slice(6) as MouseButton };
  const items: PatternItem[] = rest.map((s) => {
    switch (s.type) {
      case "trace": return { kind: "trace", points: s.points };
      case "shape": return { kind: "shape", templates: s.templates };
      case "press": return { kind: "key", key: s.key };
      case "wheel": return { kind: "key", key: "wheel:1", step: s };
      case "edge": return { kind: "key", key: `edge:${s.edge}`, step: s };
    }
  });
  return { trigger, items };
}

export function compose(m: PatternModel): Step[] {
  const first: Step =
    m.trigger.kind === "edge" ? { type: "edge", edge: m.trigger.edge } : { type: "press", key: `mouse:${m.trigger.button}` };
  return [first, ...m.items.map((it): Step => {
    switch (it.kind) {
      case "trace": return { type: "trace", points: it.points };
      case "shape": return { type: "shape", templates: it.templates };
      case "key": return it.step ?? { type: "press", key: it.key };
    }
  })];
}

export function traceOf(pattern: Step[]): number[] | null {
  const t = pattern.find((s) => s.type === "trace");
  return t && t.type === "trace" ? t.points : null;
}

/** 轨迹点 → 方向箭头串（↑→↓←↗…）。 */
export function arrows(points: number[]): string {
  const dirs: string[] = [];
  for (let i = 2; i + 1 < points.length; i += 2) {
    const dx = Math.sign(points[i] - points[i - 2]);
    const dy = Math.sign(points[i + 1] - points[i - 1]);
    const a = ({ "0,-1": "↑", "1,-1": "↗", "1,0": "→", "1,1": "↘", "0,1": "↓", "-1,1": "↙", "-1,0": "←", "-1,-1": "↖" } as Record<string, string>)[`${dx},${dy}`];
    if (a) dirs.push(a);
  }
  return dirs.join("");
}

/** 模式的文字描述（列表次要信息、冲突提示用）。按钮和边角使用完整短语。 */
export function describePattern(pattern: Step[]): string {
  const m = decompose(pattern);
  const head = m.trigger.kind === "edge" ? t(`ui.pattern.move_to_edge.${m.trigger.edge}` as MsgKey) : buttonPhrase(m.trigger.button, "hold");
  const parts = m.items.map((it) => (it.kind === "trace" ? t("ui.pattern.draw", { arrows: arrows(it.points) || t("ui.pattern.stroke") }) : it.kind === "shape" ? t("ui.pattern.draw_shape") : keyTagName(it.key)));
  return [head, ...parts].join(t("ui.pattern.join"));
}

function stepKey(s: Step): string {
  switch (s.type) {
    case "press":
      return s.key;
    case "edge":
      return `edge:${s.edge}`;
    case "wheel":
      return "wheel:any";
    case "trace":
      return `trace:${arrows(s.points)}`;
    case "shape":
      return `shape:${JSON.stringify(s.templates)}`;
  }
}

/** 模式指纹：相同指纹 = 互相冲突（同一作用域内后者生效）。 */
export function patternKey(pattern: Step[]): string {
  return pattern.map(stepKey).join(" ");
}

/** 作用域内的冲突：指纹 → 绑定下标列表（仅含 ≥2 个的）。 */
export function conflicts(bindings: Binding[]): Map<number, number[]> {
  const groups = new Map<string, number[]>();
  bindings.forEach((b, i) => {
    if (b.disabled) return; // 停用的不参与，也就不算重复
    const k = patternKey(b.pattern);
    groups.set(k, [...(groups.get(k) ?? []), i]);
  });
  const out = new Map<number, number[]>();
  for (const idxs of groups.values()) if (idxs.length > 1) for (const i of idxs) out.set(i, idxs.filter((j) => j !== i));
  return out;
}

// ---------------------------------------------------------------- 动作

/** 动作类型的显示名（按需取词条）。 */
export function actionName(type: Action["type"]): string {
  return t(`action.${type}` as MsgKey);
}

export const ACTION_ORDER: Action["type"][] = [
  "key_sequence",
  "window_op",
  "launch_app",
  "open_url",
  "shell",
  "system_key",
  "menu",
  "lua",
];

export function defaultAction(type: Action["type"]): Action {
  switch (type) {
    case "key_sequence":
      return { type, keys: "", activate_target: false };
    case "window_op":
      return { type, op: "minimize", allow_child: false };
    case "launch_app":
      return { type, exe: "", args: null, elevate: false, regex: false, if_running: "activate" };
    case "open_url":
      return { type, url_template: "https://www.bing.com/search?q=%MS_SELECTED_TEXT%", browser: null };
    case "shell":
      return { type, script: "" };
    case "system_key":
      return { type, key: "PLAY_PAUSE" };
    case "menu":
      return { type, columns: 1, items: [] };
    case "lua":
      return { type, code: "function ms_action()\n  ms.show_toast(\"Hello\", 1000)\nend\n" };
  }
}

export const WINDOW_OPS: { op: string; key: MsgKey }[] = [
  { op: "minimize", key: "window_op.minimize" },
  { op: "toggle_maximize", key: "window_op.toggle_maximize" },
  { op: "close", key: "window_op.close" },
  { op: "toggle_topmost", key: "window_op.toggle_topmost" },
];

export const SYSTEM_KEYS: { key: string; i18n: MsgKey }[] = [
  { key: "PLAY_PAUSE", i18n: "syskey.play_pause" },
  { key: "NEXT", i18n: "syskey.next" },
  { key: "PREVIOUS", i18n: "syskey.previous" },
  { key: "VOLUME_UP", i18n: "syskey.volume_up" },
  { key: "VOLUME_DOWN", i18n: "syskey.volume_down" },
  { key: "MUTE", i18n: "syskey.mute" },
  { key: "BRIGHTNESS_UP", i18n: "syskey.brightness_up" },
  { key: "BRIGHTNESS_DOWN", i18n: "syskey.brightness_down" },
];

/** 动作的一行摘要。 */
export function actionSummary(a: Action): string {
  switch (a.type) {
    case "key_sequence":
      return isMac() ? keyseqToCaps(a.keys).map((group) => group.join("+")).join(" ") : a.keys;
    case "window_op":
      return t(WINDOW_OPS.find((w) => w.op === a.op)?.key ?? "action.window_op");
    case "launch_app":
      return fileName(a.exe) || t("ui.action.no_app");
    case "open_url":
      return a.url_template;
    case "shell":
      return a.script.split("\n")[0];
    case "system_key":
      return t(SYSTEM_KEYS.find((k) => k.key === a.key.toUpperCase())?.i18n ?? "action.system_key");
    case "menu":
      return t("ui.action.menu_items", { count: a.items.length });
    case "lua":
      return t("ui.action.lua_summary");
  }
}

export function fileName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

export function newBinding(): Binding {
  return {
    name: t("ui.app.new_gesture"),
    pattern: [{ type: "press", key: "mouse:right" }],
    action: defaultAction("key_sequence"),
    execute_on_match: false,
    hide_label: false,
    secured: false,
  };
}

export function uid(): string {
  return Math.random().toString(36).slice(2, 10) + Date.now().toString(36);
}
