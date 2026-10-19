import { invoke } from "@tauri-apps/api/core";
import type { AppInfo, Config, EngineStatus, InitState, PresetApp, RunningApp } from "./types";
import { t, type MsgKey } from "./i18n";
import zh from "../../../../i18n/zh-CN.json";

/** 后端错误（词条键 + 参数），前端翻译。 */
export interface Msg {
  key: string;
  params: Record<string, string>;
}

// 后端命令的薄封装（命令定义见 src/main.rs）。
export const api = {
  initState: () => invoke<InitState>("init_state"),
  saveConfig: (config: Config) => invoke<void>("save_config", { config }),
  engineStatus: () => invoke<EngineStatus>("get_engine_status"),
  setUiLang: (tag: string) => invoke<void>("set_ui_lang", { tag }),
  /** 校验结果直接给出已翻译的错误文本；合法返回 null。 */
  validateKeys: (keys: string) =>
    invoke<Msg | null>("validate_keys", { keys }).then((m) => (m ? msgText(m) : null)),
  recordTrace: (points: [number, number][], grid: number) => invoke<number[]>("record_trace", { points, grid }),
  classifyStroke: (points: [number, number][]) => invoke<"trace" | "shape">("classify_stroke", { points }),
  shapeSample: (points: [number, number][]) => invoke<number[]>("shape_sample", { points }),
  shapeCompare: (a: number[], b: number[]) => invoke<number>("shape_compare", { a, b }),
  shapeLimits: (sample: number[]) => invoke<{ accept: number; margin: number }>("shape_limits", { sample }),
  importLegacy: (gestures: string, prefs: string | null) => invoke<Config>("import_legacy", { gestures, prefs }),
  loadConfigFile: (path: string) => invoke<Config>("load_config_file", { path }),
  exportConfig: (path: string, config: Config) => invoke<void>("export_config", { path, config }),
  pickFile: (kind: "json" | "exe" | "any", save: boolean, title: string) =>
    invoke<string | null>("pick_file", { kind, save, title }),
  runningApps: () => invoke<RunningApp[]>("running_apps"),
  resolveApp: (path: string) => invoke<string>("resolve_app", { path }),
  pickWindow: () => invoke<{ exe: string; title: string } | null>("pick_window"),
  presetApps: () => invoke<PresetApp[]>("preset_apps"),
  appInfo: () => invoke<AppInfo>("app_info"),
  checkUpdate: () => invoke<{ current: string; latest: string; newer: boolean; notes_url: string }>("check_update"),
  installUpdate: () => invoke<void>("install_update"),
  takeAboutRequest: () => invoke<boolean>("take_about_request"),
  openPath: (what: "log" | "config_dir" | "homepage" | "github" | "issues" | "releases") => invoke<void>("open_path", { what }),
  appIcons: (exes: string[]) => invoke<Record<string, string | null>>("app_icons", { exes }),
  setAutostart: (on: boolean) => invoke<EngineStatus>("set_autostart", { on }),
  startEngine: () => invoke<EngineStatus>("start_engine"),
  revealConfig: () => invoke<void>("reveal_config"),
};

/** Msg → 文本。`reason` 参数本身也是词条键（keyseq 错误），连同其余参数一起翻译。 */
export function msgText(m: Msg): string {
  const values: Record<string, string | number> = { ...m.params };
  const reason = m.params.reason;
  if (reason !== undefined && reason in zh) {
    values.reason = t(reason as MsgKey, values);
  }
  return t(m.key as MsgKey, values);
}

/** 后端错误 → 可显示文本：先识别 Msg 结构，再兼容旧的纯字符串。 */
export function errText(e: unknown): string {
  if (typeof e === "string") return e;
  if (e && typeof e === "object" && "key" in e && "params" in e) return msgText(e as Msg);
  return e instanceof Error ? e.message : JSON.stringify(e);
}
