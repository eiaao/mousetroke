// 与 mousetroke-core::schema 一一对应（snake_case JSON）。

/** 内置的"建议禁用手势"程序。`path` = 本机找到的位置（未安装为 null）。 */
export interface PresetApp {
  exe: string;
  name: string;
  category: string;
  /** 显示名的词条键（preset.app.*）；界面按它翻译，name 只作回落。 */
  name_key?: string;
  /** 类别的词条键（preset.category.*）。 */
  category_key?: string;
  path: string | null;
}

export interface AppInfo {
  version: string;
  exePath: string;
  configPath: string;
  logPath: string;
}

export type MouseButton = "left" | "right" | "middle" | "x1" | "x2";

/** 序列化的键标签：`mouse:right` / `kb:<VK>` / `wheel:<±1>` */
export type KeyTag = string;

export type Step =
  | { type: "press"; key: KeyTag }
  | { type: "trace"; points: number[] }
  | { type: "shape"; templates: number[][] }
  | { type: "edge"; edge: number }
  | { type: "wheel"; horizontal: boolean };

export type SendMode = "down_up" | "mapping" | "toggle" | "down" | "up";
export type IfRunning = "activate" | "send_keys" | "run_again";
export type WindowOpKind = "toggle_maximize" | "minimize" | "close" | "toggle_topmost";

export type Action =
  | { type: "key_sequence"; keys: string; mode?: SendMode; secured?: boolean; activate_target?: boolean }
  | { type: "shell"; script: string }
  | { type: "lua"; code: string }
  | { type: "open_url"; url_template: string; browser?: string | null }
  | { type: "menu"; columns: number; items: MenuItem[] }
  | {
      type: "launch_app";
      exe: string;
      args?: string | null;
      elevate?: boolean;
      win_class?: string | null;
      win_title?: string | null;
      regex?: boolean;
      if_running?: IfRunning;
    }
  | { type: "window_op"; op: WindowOpKind; allow_child?: boolean }
  | { type: "system_key"; key: string };

export type ActionType = Action["type"];

export interface MenuItem {
  label: string;
  action: Action;
}

export interface Binding {
  name: string;
  /** 内置名称的词条键；存在且未被用户改名时按当前语言显示。 */
  name_key?: string;
  pattern: Step[];
  action: Action;
  execute_on_match: boolean;
  hide_label: boolean;
  secured: boolean;
  /** 停用：保留但不参与识别。缺省 = 启用 */
  disabled?: boolean;
}

export type ScopeKind = "global" | "app" | "special";

export interface AppMatch {
  exe: string;
  win_class?: string | null;
  win_title?: string | null;
  regex?: boolean;
}

export interface Scope {
  id: string;
  name: string;
  /** 内置名称的词条键（builtin.scope.*）。 */
  name_key?: string;
  kind: ScopeKind;
  app?: AppMatch;
  special?: "desktop" | "taskbar";
  /** 缺省 = true */
  inherit_global?: boolean;
  /** 在这个程序里禁用所有手势（含通用手势与屏幕边角）。缺省 = false */
  disabled?: boolean;
  bindings: Binding[];
}

export interface Settings {
  update_check?: boolean;
  /** auto | zh-CN | en-US */
  language?: string;
  drag_timeout_ms: number;
  drag_timeout_hint: boolean;
  min_stroke_px: number;
  target_mode: "under_cursor" | "focused";
  pause_hotkey: string;
  show_tray: boolean;
  trail: { show: boolean; width: number; border: number; fade_out: boolean; color_normal: string; color_matched: string };
  label: { show: boolean; pos: number; border: number; color_normal: string; color_matched: string };
}

export interface Config {
  version: number;
  scopes: Scope[];
  settings: Settings;
}

export interface EngineStatus {
  running: boolean;
  autostart: boolean;
  exeFound: boolean;
}

export interface InitState {
  platform: "windows" | "macos";
  about?: boolean;
  configPath: string;
  config: Config;
  configExists: boolean;
  loadError: string | null;
  engine: EngineStatus;
  /** 系统首选界面语言列表（后端用 GetUserPreferredUILanguages 取得）。 */
  systemLangs: string[];
  /** 后端按 language + 系统语言解析出的语言标签。 */
  resolvedLang: string;
}

export interface RunningApp {
  exe: string;
  title: string;
}
