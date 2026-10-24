//! 动作库：稳定 ID、选择器元数据与平台展开共用此表。新平台能力只需替换 Pending 项。
use crate::caps::Platform;
use crate::i18n::Lang;
use crate::schema::{Action, IfRunning, SendMode, WindowOpKind};
use serde::Serialize;
use serde_json::{Map, Value};

pub type Params = Map<String, Value>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Category { Browser, Edit, Text, Window, Desktop, Media, Open, Mousetroke }

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Choice {
    pub value: &'static str,
    pub label_key: &'static str,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PathKind { Folder, File }

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ParamKind {
    Text,
    MultilineText,
    Choice { choices: &'static [Choice] },
    Url,
    Path { kind: PathKind },
    App,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum ParamDefault { Literal(&'static str), SearchEngine }

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Param {
    pub name: &'static str,
    pub label_key: &'static str,
    pub kind: ParamKind,
    pub default: ParamDefault,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Expansion {
    Keys(&'static str),
    Window(WindowOpKind),
    SystemKey(&'static str),
    Launch(&'static str),
    Shell(&'static str),
    Search,
    Translate,
    SelectedUrl,
    Website,
    OpenPath,
    App,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(tag = "status", content = "value", rename_all = "snake_case")]
pub enum Implementation {
    Implemented(Expansion),
    Unsupported { reason_key: &'static str },
    Pending { reason_key: &'static str },
}

#[derive(Debug, Serialize)]
pub struct Preset {
    pub id: &'static str,
    pub category: Category,
    pub name_key: &'static str,
    pub description_key: &'static str,
    pub aliases_key: &'static str,
    pub params: &'static [Param],
    pub windows: Implementation,
    pub macos: Implementation,
}

impl Preset {
    pub fn implementation(&self, platform: Platform) -> Implementation {
        match platform { Platform::Windows => self.windows, Platform::Macos => self.macos }
    }

    pub fn default_params(&self, ui_lang: Lang) -> Params {
        self.params.iter().map(|p| {
            let value = match p.default {
                ParamDefault::Literal(value) => value,
                ParamDefault::SearchEngine => if ui_lang == Lang::ZhCn { "baidu" } else { "google" },
            };
            (p.name.into(), Value::String(value.into()))
        }).collect()
    }
}

macro_rules! choices {
    ($($value:literal),+ $(,)?) => { &[$(Choice { value: $value, label_key: concat!("preset.choice.", $value) }),+] };
}

const METHOD: Param = Param { name: "method", label_key: "preset.param.method", kind: ParamKind::Choice { choices: choices!["paste", "type"] }, default: ParamDefault::Literal("paste") };
const TYPE_PARAMS: &[Param] = &[
    Param { name: "text", label_key: "preset.param.text", kind: ParamKind::MultilineText, default: ParamDefault::Literal("") }, METHOD,
];
const DATE_PARAMS: &[Param] = &[
    Param { name: "format", label_key: "preset.param.format", kind: ParamKind::Choice { choices: choices!["date_iso", "date_local", "date_us", "time", "datetime", "custom"] }, default: ParamDefault::Literal("date_iso") },
    Param { name: "custom_format", label_key: "preset.param.custom_format", kind: ParamKind::Text, default: ParamDefault::Literal("%Y-%m-%d") }, METHOD,
];
const SEARCH_PARAMS: &[Param] = &[
    Param { name: "engine", label_key: "preset.param.engine", kind: ParamKind::Choice { choices: choices!["bing", "google", "baidu", "duckduckgo", "custom"] }, default: ParamDefault::SearchEngine },
    Param { name: "url", label_key: "preset.param.search_url", kind: ParamKind::Url, default: ParamDefault::Literal("https://www.google.com/search?q=%MS_SELECTED_TEXT%") },
];
const TRANSLATE_PARAMS: &[Param] = &[
    Param { name: "service", label_key: "preset.param.service", kind: ParamKind::Choice { choices: &[
        Choice { value: "google", label_key: "preset.choice.google_translate" }, Choice { value: "deepl", label_key: "preset.choice.deepl" }, Choice { value: "bing", label_key: "preset.choice.bing_translate" },
    ] }, default: ParamDefault::Literal("google") },
    Param { name: "target_language", label_key: "preset.param.target_language", kind: ParamKind::Choice { choices: &[
        Choice { value: "ui", label_key: "preset.choice.ui" },
        Choice { value: "zh-CN", label_key: "lang.name.zh_cn" }, Choice { value: "en-US", label_key: "lang.name.en_us" },
        Choice { value: "es-ES", label_key: "lang.name.es_es" }, Choice { value: "fr-FR", label_key: "lang.name.fr_fr" },
        Choice { value: "de-DE", label_key: "lang.name.de_de" }, Choice { value: "ja-JP", label_key: "lang.name.ja_jp" },
        Choice { value: "pt-BR", label_key: "lang.name.pt_br" }, Choice { value: "ru-RU", label_key: "lang.name.ru_ru" },
        Choice { value: "it-IT", label_key: "lang.name.it_it" }, Choice { value: "ko-KR", label_key: "lang.name.ko_kr" },
    ] }, default: ParamDefault::Literal("ui") },
];
const WEBSITE_PARAMS: &[Param] = &[Param { name: "url", label_key: "preset.param.url", kind: ParamKind::Url, default: ParamDefault::Literal("https://mousetroke.com/") }];
const FOLDER_PARAMS: &[Param] = &[Param { name: "path", label_key: "preset.param.path", kind: ParamKind::Path { kind: PathKind::Folder }, default: ParamDefault::Literal("") }];
const FILE_PARAMS: &[Param] = &[Param { name: "path", label_key: "preset.param.path", kind: ParamKind::Path { kind: PathKind::File }, default: ParamDefault::Literal("") }];
const APP_PARAMS: &[Param] = &[Param { name: "app", label_key: "preset.param.app", kind: ParamKind::App, default: ParamDefault::Literal("") }];

const fn keys(value: &'static str) -> Implementation { Implementation::Implemented(Expansion::Keys(value)) }
const fn window(value: WindowOpKind) -> Implementation { Implementation::Implemented(Expansion::Window(value)) }
const fn system(value: &'static str) -> Implementation { Implementation::Implemented(Expansion::SystemKey(value)) }
const fn launch(value: &'static str) -> Implementation { Implementation::Implemented(Expansion::Launch(value)) }
const fn shell(value: &'static str) -> Implementation { Implementation::Implemented(Expansion::Shell(value)) }
const fn implemented(value: Expansion) -> Implementation { Implementation::Implemented(value) }
const PENDING: Implementation = Implementation::Pending { reason_key: "preset.status.pending" };
const PHASE2: Implementation = Implementation::Pending { reason_key: "preset.status.phase2" };
const WINDOWS_ONLY: Implementation = Implementation::Unsupported { reason_key: "preset.status.windows_only" };
const MAC_ONLY: Implementation = Implementation::Unsupported { reason_key: "preset.status.mac_only" };

macro_rules! preset {
    ($id:literal, $category:ident, $win:expr, $mac:expr $(, $params:expr)?) => {
        Preset {
            id: $id, category: Category::$category,
            name_key: concat!("preset.action.", $id, ".name"),
            description_key: concat!("preset.action.", $id, ".description"),
            aliases_key: concat!("preset.action.", $id, ".aliases"),
            windows: $win, macos: $mac, params: preset!(@params $($params)?),
        }
    };
    (@params $params:expr) => { $params };
    (@params) => { &[] };
}

/// 表的顺序也是选择器与文档的默认顺序；不要按本地化名称排序。
pub static REGISTRY: &[Preset] = &[
    preset!("browser.back", Browser, keys("%{LEFT}"), keys("#[")),
    preset!("browser.forward", Browser, keys("%{RIGHT}"), keys("#]")),
    preset!("browser.reload", Browser, keys("{F5}"), keys("#r")),
    preset!("browser.hard_reload", Browser, keys("^{F5}"), keys("#+r")),
    preset!("browser.new_tab", Browser, keys("^t"), keys("#t")),
    preset!("browser.close_tab", Browser, keys("^w"), keys("#w")),
    preset!("browser.reopen_tab", Browser, keys("^+t"), keys("#+t")),
    preset!("browser.prev_tab", Browser, keys("^+{TAB}"), keys("^+{TAB}")),
    preset!("browser.next_tab", Browser, keys("^{TAB}"), keys("^{TAB}")),
    preset!("browser.new_window", Browser, keys("^n"), keys("#n")),
    preset!("browser.private_window", Browser, keys("^+n"), keys("#+n")),
    preset!("browser.address_bar", Browser, keys("^l"), keys("#l")),
    preset!("browser.find", Browser, keys("^f"), keys("#f")),
    preset!("browser.bookmark", Browser, keys("^d"), keys("#d")),
    preset!("view.zoom_in", Browser, keys("^="), keys("#=")),
    preset!("view.zoom_out", Browser, keys("^-"), keys("#-")),
    preset!("view.zoom_reset", Browser, keys("^0"), keys("#0")),
    preset!("view.top", Edit, keys("^{HOME}"), keys("#{UP}")),
    preset!("view.bottom", Edit, keys("^{END}"), keys("#{DOWN}")),
    preset!("view.page_up", Edit, keys("{PGUP}"), keys("{PGUP}")),
    preset!("view.page_down", Edit, keys("{PGDN}"), keys("{PGDN}")),
    preset!("edit.copy", Edit, keys("^c"), keys("#c")),
    preset!("edit.cut", Edit, keys("^x"), keys("#x")),
    preset!("edit.paste", Edit, keys("^v"), keys("#v")),
    preset!("edit.paste_enter", Edit, keys("^v{ENTER}"), keys("#v{ENTER}")),
    preset!("edit.paste_plain", Edit, keys("^+v"), keys("#+%v")),
    preset!("edit.select_all", Edit, keys("^a"), keys("#a")),
    preset!("edit.undo", Edit, keys("^z"), keys("#z")),
    preset!("edit.redo", Edit, keys("^y"), keys("#+z")),
    preset!("edit.save", Edit, keys("^s"), keys("#s")),
    preset!("key.enter", Edit, keys("{ENTER}"), keys("{ENTER}")),
    preset!("key.backspace", Edit, keys("{BACKSPACE}"), keys("{BACKSPACE}")),
    preset!("key.delete", Edit, keys("{DELETE}"), keys("{DELETE}")),
    preset!("key.escape", Edit, keys("{ESC}"), keys("{ESC}")),
    preset!("view.fullscreen", Edit, keys("{F11}"), keys("^#f")),
    preset!("text.type", Text, PENDING, PENDING, TYPE_PARAMS),
    preset!("text.datetime", Text, PENDING, PENDING, DATE_PARAMS),
    preset!("text.search", Text, implemented(Expansion::Search), implemented(Expansion::Search), SEARCH_PARAMS),
    preset!("text.translate", Text, implemented(Expansion::Translate), implemented(Expansion::Translate), TRANSLATE_PARAMS),
    preset!("text.open_selected_url", Text, implemented(Expansion::SelectedUrl), implemented(Expansion::SelectedUrl)),
    preset!("text.copy_plain", Text, PENDING, PENDING),
    preset!("window.minimize", Window, window(WindowOpKind::Minimize), window(WindowOpKind::Minimize)),
    preset!("window.toggle_maximize", Window, window(WindowOpKind::ToggleMaximize), window(WindowOpKind::ToggleMaximize)),
    preset!("window.close", Window, window(WindowOpKind::Close), window(WindowOpKind::Close)),
    preset!("app.quit", Window, keys("%{F4}"), keys("#q")),
    preset!("app.kill", Window, shell("taskkill /F /PID %MS_PID%"), shell("/bin/kill -9 %MS_PID%")),
    preset!("window.toggle_topmost", Window, window(WindowOpKind::ToggleTopmost), WINDOWS_ONLY),
    preset!("window.snap_left", Window, PENDING, PENDING),
    preset!("window.snap_right", Window, PENDING, PENDING),
    preset!("window.center", Window, PENDING, PENDING),
    preset!("window.next_monitor", Window, PENDING, PENDING),
    preset!("app.hide", Window, MAC_ONLY, keys("#h")),
    preset!("desktop.switch_app", Desktop, keys("%{TAB}"), keys("#{TAB}")),
    preset!("desktop.task_view", Desktop, keys("#{TAB}"), keys("^{UP}")),
    preset!("desktop.show", Desktop, keys("#d"), keys("{F11}")),
    preset!("desktop.prev", Desktop, keys("^#{LEFT}"), keys("^{LEFT}")),
    preset!("desktop.next", Desktop, keys("^#{RIGHT}"), keys("^{RIGHT}")),
    preset!("desktop.launcher", Desktop, keys("^{ESC}"), keys("#{SPACE}")),
    preset!("desktop.files", Desktop, keys("#e"), launch("com.apple.finder")),
    preset!("capture.region", Desktop, keys("#+s"), keys("#+4")),
    preset!("capture.screen", Desktop, keys("#{PRTSC}"), keys("#+3")),
    preset!("capture.tool", Desktop, launch("SnippingTool.exe"), keys("#+5")),
    preset!("desktop.clipboard_history", Desktop, keys("#v"), WINDOWS_ONLY),
    preset!("desktop.emoji", Desktop, keys("#."), keys("^#{SPACE}")),
    preset!("desktop.notifications", Desktop, keys("#n"), WINDOWS_ONLY),
    preset!("system.lock", Desktop, keys("#l"), keys("^#q")),
    preset!("system.task_manager", Desktop, keys("^+{ESC}"), launch("com.apple.ActivityMonitor")),
    preset!("system.force_quit_dialog", Desktop, MAC_ONLY, keys("#%{ESC}")),
    preset!("system.settings", Desktop, keys("#i"), launch("com.apple.systempreferences")),
    preset!("system.calculator", Desktop, launch("calc.exe"), launch("com.apple.calculator")),
    preset!("system.display_off", Desktop, PENDING, PENDING),
    preset!("system.sleep", Desktop, PENDING, PENDING),
    preset!("media.play_pause", Media, system("PLAY_PAUSE"), system("PLAY_PAUSE")),
    preset!("media.previous", Media, system("PREVIOUS"), system("PREVIOUS")),
    preset!("media.next", Media, system("NEXT"), system("NEXT")),
    preset!("media.volume_up", Media, system("VOLUME_UP"), system("VOLUME_UP")),
    preset!("media.volume_down", Media, system("VOLUME_DOWN"), system("VOLUME_DOWN")),
    preset!("media.mute", Media, system("MUTE"), system("MUTE")),
    preset!("display.brightness_up", Media, system("BRIGHTNESS_UP"), system("BRIGHTNESS_UP")),
    preset!("display.brightness_down", Media, system("BRIGHTNESS_DOWN"), system("BRIGHTNESS_DOWN")),
    preset!("open.website", Open, implemented(Expansion::Website), implemented(Expansion::Website), WEBSITE_PARAMS),
    preset!("open.folder", Open, implemented(Expansion::OpenPath), implemented(Expansion::OpenPath), FOLDER_PARAMS),
    preset!("open.file", Open, implemented(Expansion::OpenPath), implemented(Expansion::OpenPath), FILE_PARAMS),
    preset!("open.app", Open, implemented(Expansion::App), implemented(Expansion::App), APP_PARAMS),
    preset!("mousetroke.toggle_pause", Mousetroke, PENDING, PENDING),
    preset!("mousetroke.settings", Mousetroke, PENDING, PENDING),
    preset!("mousetroke.cheatsheet", Mousetroke, PHASE2, PHASE2),
];

pub fn find(id: &str) -> Option<&'static Preset> { REGISTRY.iter().find(|entry| entry.id == id) }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PresetError {
    UnknownId,
    Unsupported(&'static str),
    Pending(&'static str),
    InvalidParam { name: &'static str, reason_key: &'static str },
}

impl PresetError {
    pub fn reason_key(&self) -> &'static str {
        match self {
            Self::UnknownId => "preset.status.unknown",
            Self::Unsupported(key) | Self::Pending(key) => key,
            Self::InvalidParam { reason_key, .. } => reason_key,
        }
    }
}

fn invalid(name: &'static str) -> PresetError { PresetError::InvalidParam { name, reason_key: "preset.status.invalid_param" } }

/// 空文本是表单的合法初始值；真正执行路径、程序等时再检查是否填写。
pub fn validate_params(preset: &Preset, params: &Params) -> Result<(), PresetError> {
    for schema in preset.params {
        let Some(value) = params.get(schema.name) else { continue };
        let text = value.as_str().ok_or_else(|| invalid(schema.name))?;
        if let ParamKind::Choice { choices } = schema.kind {
            if !choices.iter().any(|c| c.value == text) { return Err(invalid(schema.name)); }
        }
    }
    Ok(())
}

fn required<'a>(params: &'a Params, name: &'static str) -> Result<&'a str, PresetError> {
    params.get(name).and_then(Value::as_str).filter(|s| !s.trim().is_empty() && !s.contains('\0')).ok_or_else(|| invalid(name))
}

fn launch_action(exe: &str, args: Option<String>) -> Action {
    Action::LaunchApp { exe: exe.into(), args, elevate: false, win_class: None, win_title: None, regex: false, if_running: IfRunning::Activate }
}

fn url_action(url: impl Into<String>) -> Action { Action::OpenUrl { url_template: url.into(), browser: None } }

/// 不改写配置；语言依赖参数在每次展开时解析，切换界面语言即可生效。
pub fn expand(id: &str, platform: Platform, params: &Params, ui_lang: Lang) -> Result<Action, PresetError> {
    let preset = find(id).ok_or(PresetError::UnknownId)?;
    let expansion = match preset.implementation(platform) {
        Implementation::Implemented(expansion) => expansion,
        Implementation::Unsupported { reason_key } => return Err(PresetError::Unsupported(reason_key)),
        Implementation::Pending { reason_key } => return Err(PresetError::Pending(reason_key)),
    };
    validate_params(preset, params)?;
    let mut values = preset.default_params(ui_lang);
    values.extend(params.clone());
    Ok(match expansion {
        Expansion::Keys(keys) => Action::KeySequence { keys: keys.into(), mode: SendMode::DownUp, secured: false, activate_target: false },
        Expansion::Window(op) => Action::WindowOp { op, allow_child: false },
        Expansion::SystemKey(key) => Action::SystemKey { key: key.into() },
        Expansion::Launch(exe) => launch_action(exe, None),
        Expansion::Shell(script) => Action::Shell { script: script.into() },
        Expansion::SelectedUrl => url_action("%MS_SELECTED_TEXT%"),
        Expansion::Website => url_action(required(&values, "url")?),
        Expansion::Search => {
            let url = match required(&values, "engine")? {
                "bing" => "https://www.bing.com/search?q=%MS_SELECTED_TEXT%",
                "google" => "https://www.google.com/search?q=%MS_SELECTED_TEXT%",
                "baidu" => "https://www.baidu.com/s?wd=%MS_SELECTED_TEXT%",
                "duckduckgo" => "https://duckduckgo.com/?q=%MS_SELECTED_TEXT%",
                "custom" => {
                    let url = required(&values, "url")?;
                    if !url.contains("%MS_SELECTED_TEXT%") || !(url.starts_with("https://") || url.starts_with("http://")) { return Err(invalid("url")); }
                    url
                }
                _ => return Err(invalid("engine")),
            };
            url_action(url)
        }
        Expansion::Translate => {
            let target = required(&values, "target_language")?;
            let lang = if target == "ui" { ui_lang } else { Lang::from_tag(target).ok_or_else(|| invalid("target_language"))? };
            let lang = if lang == Lang::QpsPloc { Lang::EnUs } else { lang };
            let primary = lang.tag().split('-').next().unwrap_or("en");
            let url = match required(&values, "service")? {
                "google" => format!("https://translate.google.com/?sl=auto&tl={}&text=%MS_SELECTED_TEXT%&op=translate", if lang == Lang::ZhCn { "zh-CN" } else { primary }),
                "deepl" => format!("https://www.deepl.com/translator#auto/{primary}/%MS_SELECTED_TEXT%"),
                "bing" => format!("https://www.bing.com/translator?from=auto-detect&to={}&text=%MS_SELECTED_TEXT%", if lang == Lang::ZhCn { "zh-Hans" } else { primary }),
                _ => return Err(invalid("service")),
            };
            url_action(url)
        }
        Expansion::OpenPath => {
            let path = required(&values, "path")?;
            // 现有 LaunchApp 直接运行可执行文件；文件关联交给 Explorer / open。
            match platform {
                Platform::Windows => {
                    if path.contains('"') { return Err(invalid("path")); }
                    let trailing = path.chars().rev().take_while(|c| *c == '\\').count();
                    let mut action = launch_action("explorer.exe", Some(format!("\"{path}{}\"", "\\".repeat(trailing))));
                    if let Action::LaunchApp { if_running, .. } = &mut action { *if_running = IfRunning::RunAgain; }
                    action
                }
                Platform::Macos => {
                    let quoted = path.replace('\'', "'\\''");
                    let mut action = launch_action("/usr/bin/open", Some(format!("-- '{quoted}'")));
                    if let Action::LaunchApp { if_running, .. } = &mut action { *if_running = IfRunning::RunAgain; }
                    action
                }
            }
        }
        Expansion::App => launch_action(required(&values, "app")?, None),
    })
}
