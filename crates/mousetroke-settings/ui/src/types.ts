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

export type AdvancedAction =
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

export interface PresetAction { type: "preset"; id: string; params?: Record<string, unknown> }
/** 原始未来类型直接往返；界面通过类型守卫展示只读卡片，不增加 JSON 包装。 */
export interface UnknownAction { type: string; [key: string]: unknown }
export type Action = AdvancedAction | PresetAction | UnknownAction;

export const ADVANCED_ACTION_TYPES = ["key_sequence", "shell", "lua", "open_url", "menu", "launch_app", "window_op", "system_key"] as const;
export function isAdvancedAction(action: Action): action is AdvancedAction {
  return (ADVANCED_ACTION_TYPES as readonly string[]).includes(action.type);
}
export function isPresetAction(action: Action): action is PresetAction {
  return action.type === "preset" && "id" in action && typeof action.id === "string";
}

export type PresetParamKind =
  | { type: "text" | "multiline_text" | "url" | "app" }
  | { type: "choice"; choices: { value: string; label_key: string }[] }
  | { type: "path"; kind: "folder" | "file" };
export type PresetImplementation =
  | { status: "implemented"; value: { type: string; value?: unknown } }
  | { status: "unsupported" | "pending"; value: { reason_key: string } };
export interface ActionPreset {
  id: string;
  category: "browser" | "edit" | "text" | "window" | "desktop" | "media" | "open" | "mousetroke";
  name_key: string;
  description_key: string;
  aliases_key: string;
  params: { name: string; label_key: string; kind: PresetParamKind; default: { kind: "literal"; value: string } | { kind: "search_engine" } }[];
  windows: PresetImplementation;
  macos: PresetImplementation;
  advanced_expansion: { windows: boolean; macos: boolean; reason_key: string };
}
export interface ActionCatalog {
  presets: ActionPreset[];
  defaults: Record<string, Record<string, unknown>>;
  /** "常用"分类：内置默认配置用到的预置动作，加上输入文字、搜索选中文字、打开网站。 */
  commonPresetIds: string[];
  knownActionTypes: string[];
  unknownActionKey: string;
}

export interface MenuItem {
  label: string;
  action: Action;
}

export interface Binding {
  name: string;
  /** 内置名称的词条键；存在且未被用户改名时按当前语言显示。 */
  name_key?: string;
  /** 上一次自动填入的动作名（用户没改名时跟随新动作）。 */
  lastActionName?: string;
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
  setup?: boolean;
  permissions?: PermissionStatus;
  configPath: string;
  config: Config;
  configExists: boolean;
  loadError: string | null;
  engine: EngineStatus;
  /** 系统首选界面语言列表（后端用 GetUserPreferredUILanguages 取得）。 */
  systemLangs: string[];
  /** 后端按 language + 系统语言解析出的语言标签。 */
  resolvedLang: string;
  actionCatalog?: ActionCatalog;
}

export interface PermissionStatus {
  accessibility: boolean;
  // 监听访问可由辅助功能提供，不等同于单独授予输入监控。
  input_monitoring: boolean;
  engine_ready?: boolean | null;
}

export type PermissionName = "accessibility" | "input_monitoring";

export interface RunningApp {
  exe: string;
  title: string;
}
