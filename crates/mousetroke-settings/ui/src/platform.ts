import type { Action } from "./types";
import { isAdvancedAction } from "./types";

export type Platform = "windows" | "macos";
// 只接受 init_state 的平台字段；Windows 作为旧版冒烟桥的兼容默认值。
let platform: Platform = "windows";
export function initPlatform(value: Platform) { platform = value; }
export function isMac() { return platform === "macos"; }
export function primaryModifier(e: { ctrlKey: boolean; metaKey: boolean }) { return isMac() ? e.metaKey : e.ctrlKey; }
export function appIdentity(value: string) { return isMac() ? value : value.split(/[\\/]/).pop() ?? value; }

/** 保留导入的原值并明确提示，用户点击修复后才移除不支持的选项。
 *  规则以 mousetroke_core::caps 为准，两端共用 mac-caps.json 用例校验。 */
export function unsupportedAction(a: Action): boolean {
  if (!isAdvancedAction(a)) return false;
  if (!isMac()) return false;
  switch (a.type) {
    case "window_op": return a.op === "toggle_topmost" || !!a.allow_child;
    case "launch_app": return !!(a.elevate || a.if_running === "send_keys" || a.win_class || a.win_title || a.regex);
    case "key_sequence": return !!(a.mode && a.mode !== "down_up") || /\{(?:MOUSE_|[HV]?WHEEL)/i.test(a.keys);
    case "menu": return a.columns > 1;
    default: return false;
  }
}
export function supportedAction(a: Action): Action {
  if (!isAdvancedAction(a)) return a;
  switch (a.type) {
    case "window_op": return { ...a, op: a.op === "toggle_topmost" ? "minimize" : a.op, allow_child: false };
    case "launch_app": return { ...a, elevate: false, if_running: a.if_running === "send_keys" ? "activate" : a.if_running, win_class: null, win_title: null, regex: false };
    case "key_sequence": return { ...a, mode: "down_up", keys: a.keys.replace(/\{(?:MOUSE_|[HV]?WHEEL)[^}]*\}/gi, "") };
    case "menu": return { ...a, columns: 1 };
    default: return a;
  }
}
