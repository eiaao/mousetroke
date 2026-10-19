//! 旧格式配置导入器 —— 把 `gestures.json` / `prefs.json`
//! 转换为 Mousetroke 原生格式（[`crate::schema`]）。
//!
//! 本模块包含旧格式的 serde 原始结构（Newtonsoft `$type` 多态，PascalCase
//! 字段名照抄仅用于解析兼容，不进入运行时），以及到原生 schema 的映射。
//! 

use serde::Deserialize;
use std::collections::BTreeMap;

use crate::schema::*;

// ---------------------------------------------------------------- 旧格式原始结构

type UnknownFields = BTreeMap<String, serde_json::Value>;

#[derive(Debug, Deserialize)]
#[serde(tag = "$type")]
pub(crate) enum LegacyGestureElement {
    #[serde(rename_all = "PascalCase")]
    KeyDown { key: String },
    #[serde(rename_all = "PascalCase")]
    Stroke { p: Vec<i32> },
    #[serde(rename_all = "PascalCase")]
    MoveToEdgeCorner { edge_corner: LegacyEdgeCorner },
    #[serde(rename_all = "PascalCase")]
    Scroll { is_horizontal: bool },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct LegacyEdgeCorner {
    value: u32,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "$type")]
pub(crate) enum LegacyCommand {
    #[serde(rename_all = "PascalCase")]
    SendKeys {
        keys: String,
        mode: Option<String>,
        secured: Option<bool>,
        activate_target_window: Option<bool>,
    },
    #[serde(rename_all = "PascalCase")]
    Cmd { code: String },
    #[serde(rename_all = "PascalCase")]
    Script { code: String },
    #[serde(rename_all = "PascalCase")]
    WebSearch { browser: String, engine: String },
    #[serde(rename_all = "PascalCase")]
    Menu {
        columns: u32,
        commands: Vec<LegacyMenuItem>,
    },
    #[serde(rename_all = "PascalCase")]
    RunApp {
        #[serde(flatten)]
        fields: UnknownFields,
    },
    #[serde(rename_all = "PascalCase")]
    WindowControl {
        command: String,
        allow_child_window: Option<bool>,
    },
    #[serde(rename_all = "PascalCase")]
    SystemFunctionKey {
        #[serde(flatten)]
        fields: UnknownFields,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct LegacyMenuItem {
    name: String,
    command: LegacyCommand,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct LegacyIntent {
    name: String,
    #[serde(default)]
    execute_on_recognize: bool,
    #[serde(default)]
    dont_show_name: Option<bool>,
    gesture: Vec<LegacyGestureElement>,
    command: LegacyCommand,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct LegacyGeneralTarget {
    id: String,
    name: String,
    intents: Vec<LegacyIntent>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct LegacyAppTarget {
    path: String,
    id: String,
    name: String,
    intents: Vec<LegacyIntent>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "$type")]
pub(crate) enum LegacySpecialTarget {
    #[serde(rename = "WinDesktopTarget", rename_all = "PascalCase")]
    WinDesktop {
        id: String,
        name: String,
        intents: Vec<LegacyIntent>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct LegacyGestures {
    general: LegacyGeneralTarget,
    #[serde(default)]
    groups: Vec<LegacyAppTarget>,
    #[serde(default)]
    apps: Vec<LegacyAppTarget>,
    #[serde(default)]
    specials: Vec<LegacySpecialTarget>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default, rename_all = "PascalCase")]
pub(crate) struct LegacyPrefs {
    start_drag_timeout: Option<u32>,
    show_start_drag_timeout_indicator: Option<bool>,
    show_path: Option<bool>,
    show_gesture_name: Option<bool>,
    show_status_icon: Option<bool>,
    path_color_normal: Option<String>,
    path_color_recognized: Option<String>,
    label_color_normal: Option<String>,
    label_color_executed: Option<String>,
    target_mode: Option<String>,
    path_line_width: Option<f64>,
    gesture_name_pos: Option<f64>,
    skip_version: Option<String>,
    fade_out_executed: Option<bool>,
    pause_resume_hotkey: Option<String>,
    stroke_min_move: Option<u32>,
    label_border_width: Option<f64>,
    path_border_width: Option<f64>,
}

// ---------------------------------------------------------------- 转换

/// 解析（容忍 BOM）。
fn from_json_str<T: serde::de::DeserializeOwned>(raw: &str) -> Result<T, String> {
    serde_json::from_str(raw.trim_start_matches('\u{feff}')).map_err(|e| e.to_string())
}

/// 旧格式 gestures.json + prefs.json → 原生 Config。
/// 旧格式默认的普通轨迹/标签颜色（#RRGGBBAA：19% 黑）与线宽。
const LEGACY_DEFAULT_NORMAL_COLOR: &str = "#00000030";
const LEGACY_DEFAULT_LINE_WIDTH: f64 = 1.5;

pub fn import(gestures_raw: &str, prefs_raw: Option<&str>) -> Result<Config, String> {
    let wg: LegacyGestures = from_json_str(gestures_raw)?;
    let prefs = prefs_raw
        .map(from_json_str::<LegacyPrefs>)
        .transpose()?
        .unwrap_or_default();

    let mut scopes = Vec::new();
    scopes.push(Scope {
        id: wg.general.id,
        name: wg.general.name,
        name_key: None,
        kind: ScopeKind::Global,
        app: None,
        special: None,
        inherit_global: true,
        disabled: false,
        bindings: wg.general.intents.iter().map(convert_intent).collect(),
    });
    // 分组（旧格式 Groups）作为 app 作用域导入（分组语义 M2 细化）。
    for t in wg.groups.iter().chain(wg.apps.iter()) {
        scopes.push(Scope {
            id: t.id.clone(),
            name: t.name.clone(),
            name_key: None,
            kind: ScopeKind::App,
            app: Some(AppMatch::new(&t.path)),
            special: None,
            inherit_global: true,
            disabled: false,
            bindings: t.intents.iter().map(convert_intent).collect(),
        });
    }
    for t in &wg.specials {
        let (id, name, intents) = match t {
            LegacySpecialTarget::WinDesktop { id, name, intents } => (id, name, intents),
        };
        let area = match name.as_str() {
            "Desktop" => SpecialArea::Desktop,
            "Taskbar" => SpecialArea::Taskbar,
            _ => SpecialArea::Desktop, // 未知特殊目标按桌面处理（M2 细化）
        };
        scopes.push(Scope {
            id: id.clone(),
            name: name.clone(),
            name_key: None,
            kind: ScopeKind::Special,
            app: None,
            special: Some(area),
            inherit_global: true,
            disabled: false,
            bindings: intents.iter().map(convert_intent).collect(),
        });
    }

    let mut settings = Settings::default();
    if let Some(v) = prefs.start_drag_timeout { settings.drag_timeout_ms = v; }
    if let Some(v) = prefs.show_start_drag_timeout_indicator { settings.drag_timeout_hint = v; }
    if let Some(v) = prefs.stroke_min_move { settings.min_stroke_px = v; }
    if let Some(v) = prefs.show_status_icon { settings.show_tray = v; }
    if let Some(v) = prefs.pause_resume_hotkey { settings.pause_hotkey = v; }
    if let Some(v) = prefs.target_mode.as_deref() {
        settings.target_mode = match v {
            "Focused" => TargetMode::Focused,
            _ => TargetMode::UnderCursor,
        };
    }
    if let Some(v) = prefs.show_path { settings.trail.show = v; }
    // 旧格式默认外观（细线 + 19% 黑 + 白描边）依赖描边才看得见；Mousetroke 不描边，
    // 遇到旧格式默认值就保留自己的默认外观，用户改过的值照常导入
    let custom = |v: &String| !v.eq_ignore_ascii_case(LEGACY_DEFAULT_NORMAL_COLOR);
    if let Some(v) = prefs.path_line_width.filter(|w| *w != LEGACY_DEFAULT_LINE_WIDTH) { settings.trail.width = v; }
    if let Some(v) = prefs.path_border_width { settings.trail.border = v as u32; }
    if let Some(v) = prefs.fade_out_executed { settings.trail.fade_out = v; }
    if let Some(v) = prefs.path_color_normal.filter(custom) { settings.trail.color_normal = v; }
    if let Some(v) = prefs.path_color_recognized { settings.trail.color_matched = v; }
    if let Some(v) = prefs.show_gesture_name { settings.label.show = v; }
    if let Some(v) = prefs.gesture_name_pos { settings.label.pos = v; }
    if let Some(v) = prefs.label_border_width { settings.label.border = v as u32; }
    if let Some(v) = prefs.label_color_normal.filter(custom) { settings.label.color_normal = v; }
    if let Some(v) = prefs.label_color_executed { settings.label.color_matched = v; }

    let mut cfg = Config { version: 1, scopes, settings };
    cfg.migrate();
    // 旧格式默认英文名 → name_key + 当前语言的译文（migrate 只补 key 不动 name）
    crate::names::localize_builtin(&mut cfg);
    Ok(cfg)
}

fn convert_intent(i: &LegacyIntent) -> Binding {
    Binding {
        name: i.name.clone(),
        name_key: None,
        pattern: i.gesture.iter().map(convert_element).collect(),
        action: convert_command(&i.command),
        execute_on_match: i.execute_on_recognize,
        hide_label: i.dont_show_name.unwrap_or(false),
        secured: false,
        disabled: false,
    }
}

fn convert_element(e: &LegacyGestureElement) -> Step {
    match e {
        LegacyGestureElement::KeyDown { key } => Step::Press { key: parse_wg_key(key) },
        LegacyGestureElement::Stroke { p } => Step::Trace { points: p.clone() },
        LegacyGestureElement::MoveToEdgeCorner { edge_corner } => Step::Edge {
            edge: edge_corner.value.min(u8::MAX as u32) as u8,
        },
        LegacyGestureElement::Scroll { is_horizontal } => Step::Wheel { horizontal: *is_horizontal },
    }
}

/// 旧格式 Key 字符串 → PatternKey：`MOUSE:0..4`、`KEYBOARD:<VK>`、`VSCROLL:<±120>`。
fn parse_wg_key(raw: &str) -> PatternKey {
    let (kind, val) = raw.split_once(':').unwrap_or(("mouse", ""));
    match kind {
        "MOUSE" => PatternKey::Mouse(match val {
            "0" => MouseButton::Left,
            "1" => MouseButton::Right,
            "2" => MouseButton::Middle,
            "3" => MouseButton::X1,
            _ => MouseButton::X2,
        }),
        "KEYBOARD" => val.parse().map(PatternKey::Keyboard).unwrap_or(PatternKey::Keyboard(0)),
        "VSCROLL" => PatternKey::Wheel(val.parse::<i32>().unwrap_or(0).signum()),
        _ => PatternKey::Keyboard(0),
    }
}

fn convert_command(c: &LegacyCommand) -> Action {
    match c {
        LegacyCommand::SendKeys { keys, mode, secured, activate_target_window } => Action::KeySequence {
            keys: keys.clone(),
            mode: match mode.as_deref() {
                Some("Mapping") => SendMode::Mapping,
                Some("Toggle") => SendMode::Toggle,
                Some("Down") => SendMode::Down,
                Some("Up") => SendMode::Up,
                _ => SendMode::DownUp,
            },
            secured: *secured == Some(true),
            activate_target: *activate_target_window == Some(true),
        },
        LegacyCommand::Cmd { code } => Action::Shell { script: crate::vars::convert_legacy_placeholders(code) },
        LegacyCommand::Script { code } => Action::Lua { code: code.clone() },
        LegacyCommand::WebSearch { browser, engine } => Action::OpenUrl {
            url_template: crate::vars::convert_legacy_placeholders(engine),
            browser: if browser == "default" { None } else { Some(browser.clone()) },
        },
        LegacyCommand::Menu { columns, commands } => Action::Menu {
            columns: *columns,
            items: commands
                .iter()
                .map(|m| MenuItem { label: m.name.clone(), action: convert_command(&m.command) })
                .collect(),
        },
        LegacyCommand::RunApp { fields } => Action::LaunchApp {
            exe: fields.get("Path").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            args: fields.get("Args").and_then(|v| v.as_str()).map(str::to_string),
            elevate: fields.get("RunAsAdmin").and_then(|v| v.as_bool()).unwrap_or(false),
            win_class: fields.get("MatchWinClass").and_then(|v| v.as_str()).map(str::to_string),
            win_title: fields.get("MatchWinTitle").and_then(|v| v.as_str()).map(str::to_string),
            regex: fields.get("IsRegex").and_then(|v| v.as_bool()).unwrap_or(false),
            if_running: IfRunning::default(),
        },
        LegacyCommand::WindowControl { command, allow_child_window } => Action::WindowOp {
            op: match command.as_str() {
                "WINDOW_TOGGLE_MAXIMIZE" => WindowOpKind::ToggleMaximize,
                "WINDOW_MINIMIZE" => WindowOpKind::Minimize,
                "WINDOW_CLOSE" => WindowOpKind::Close,
                "WINDOW_TOGGLE_TOPMOST" => WindowOpKind::ToggleTopmost,
                _ => WindowOpKind::ToggleMaximize,
            },
            allow_child: *allow_child_window == Some(true),
        },
        LegacyCommand::SystemFunctionKey { fields } => Action::SystemKey {
            key: fields
                .iter()
                .find(|(k, _)| k != &"$type")
                .map(|(_, v)| v.to_string().trim_matches('"').to_string())
                .unwrap_or_default(),
        },
    }
}
