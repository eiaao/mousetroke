//! 原生配置 schema —— Mousetroke 自己的 snake_case JSON 格式。
//!
//! 与旧格式解耦：旧格式文件仅作为导入来源（[`crate::legacy`]），
//! 运行时只读写本格式。字段命名遵循本 crate 词汇表（Scope/Binding/Pattern/Action）。

use serde::{Deserialize, Serialize};
use serde::ser::SerializeStruct;

// ================================================================ 配置根

/// config.json 根结构。
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct Config {
    /// schema 版本，用于未来迁移。
    pub version: u32,
    pub scopes: Vec<Scope>,
    pub settings: Settings,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            scopes: Vec::new(),
            settings: Settings::default(),
        }
    }
}

// ================================================================ 作用域

/// 手势作用域。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Scope {
    /// 稳定 id（升级时保持绑定关系）。
    pub id: String,
    /// 展示名。
    pub name: String,
    /// 内置名称的词条键（如 "builtin.scope.global"）。存在且用户没改名时按当前语言显示；
    /// 改名即清空。`name` 始终保留一份文字，作为不认识该键时的回落。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name_key: Option<String>,
    pub kind: ScopeKind,
    /// `kind == app` 时必填：目标匹配规则。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app: Option<AppMatch>,
    /// `kind == special` 时必填：特殊区域。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub special: Option<SpecialArea>,
    /// 本作用域未命中时是否回落到通用（global）手势。默认是。
    #[serde(default = "yes", skip_serializing_if = "is_true")]
    pub inherit_global: bool,
    /// 在这个程序里禁用所有手势（含通用手势与屏幕边角）：鼠标键原样交给程序。
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub disabled: bool,
    pub bindings: Vec<Binding>,
}

fn yes() -> bool {
    true
}

fn is_true(v: &bool) -> bool {
    *v
}

fn default_language() -> String {
    "auto".into()
}

fn is_auto(v: &str) -> bool {
    v == "auto"
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScopeKind {
    Global,
    App,
    Special,
}

/// 应用匹配规则。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AppMatch {
    /// 目标 exe 绝对路径（大小写不敏感比较）。
    pub exe: String,
    #[serde(default)]
    pub win_class: Option<String>,
    #[serde(default)]
    pub win_title: Option<String>,
    #[serde(default)]
    pub regex: bool,
}

impl AppMatch {
    pub fn new(exe: impl Into<String>) -> Self {
        Self {
            exe: exe.into(),
            win_class: None,
            win_title: None,
            regex: false,
        }
    }
}

/// 特殊区域（桌面/任务栏）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpecialArea {
    Desktop,
    Taskbar,
}

// ================================================================ 绑定

/// 一条手势绑定。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Binding {
    pub name: String,
    /// 内置名称的词条键（如 "builtin.gesture.close_tab"）。存在且用户没改名时按当前语言
    /// 显示；改名即清空。`name` 始终保留一份当时语言的文字，作为不认识该键时的回落。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name_key: Option<String>,
    pub pattern: Vec<Step>,
    pub action: Action,
    /// 识别即执行（不等触发键释放）。
    #[serde(default)]
    pub execute_on_match: bool,
    /// 执行时不显示手势名标签。
    #[serde(default)]
    pub hide_label: bool,
    /// 敏感项：UI 中隐藏命令内容。
    #[serde(default)]
    pub secured: bool,
    /// 停用：保留在配置里但不参与识别（其触发键也不再拦截）。
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub disabled: bool,
    /// 草稿：尚未完成的绑定，读取时统一停用，完成后由设置界面启用。
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub draft: bool,
}

// ================================================================ 模式步骤

/// 手势模式的组成步骤（有序）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Step {
    /// 按下一个键并保持到手势结束。
    Press { key: PatternKey },
    /// 笔画：归一化绝对坐标点序列（偶数长度，值域 0~100，50 = 一格），
    /// 匹配时差分为位移段。编码已破解（见 recognizer 模块文档）。
    /// `alias = "dirs"` 兼容旧配置文件。
    Trace {
        #[serde(alias = "dirs")]
        points: Vec<i32>,
    },
    /// 曲线笔画：多个已归一化的 64 点样本，x/y 交替排列，坐标乘 1000 后取整。
    /// 与 Trace 占用相同的会话步骤位置，详见 [`crate::shape`]。
    Shape { templates: Vec<Vec<i32>> },
    /// 鼠标移至屏幕边缘/角落（位标志编码，见 [`crate::edge`]）。
    Edge { edge: u8 },
    /// 滚动滚轮。
    Wheel { horizontal: bool },
}

impl Step {
    /// 从原始路径录制形状。忽略无效样本；没有有效样本时返回 `None`。
    pub fn shape_from_samples(samples: &[Vec<(f64, f64)>]) -> Option<Self> {
        let templates: Vec<_> = samples.iter().filter_map(|raw| crate::shape::normalize(raw)).map(|p| crate::shape::encode(&p)).collect();
        (!templates.is_empty()).then_some(Self::Shape { templates })
    }
}

/// 模式中可按下的键。序列化为短字符串标签：
/// `mouse:left|mouse:right|mouse:middle|mouse:x1|mouse:x2`、`kb:<VK 码>`、`wheel:<±格数>`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PatternKey {
    Mouse(MouseButton),
    Keyboard(u16),
    /// 滚轮（正=向下，负=向上，单位：格）。
    Wheel(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    X1,
    X2,
}

impl PatternKey {
    pub fn tag(&self) -> String {
        match self {
            Self::Mouse(b) => format!("mouse:{}", match b {
                MouseButton::Left => "left",
                MouseButton::Right => "right",
                MouseButton::Middle => "middle",
                MouseButton::X1 => "x1",
                MouseButton::X2 => "x2",
            }),
            Self::Keyboard(vk) => format!("kb:{vk}"),
            Self::Wheel(n) => format!("wheel:{n}"),
        }
    }
}

impl Serialize for PatternKey {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.tag())
    }
}

impl<'de> Deserialize<'de> for PatternKey {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let tag = String::deserialize(d)?;
        let (kind, val) = tag
            .split_once(':')
            .ok_or_else(|| serde::de::Error::custom(format!("非法键标签: {tag:?}")))?; // i18n:allow 配置损坏时的诊断信息（serde 原文混排，不进词条）
        // i18n:allow 配置损坏时的诊断信息（serde 原文混排，不进词条）
        Ok(match kind {
            "mouse" => Self::Mouse(match val {
                "left" => MouseButton::Left,
                "right" => MouseButton::Right,
                "middle" => MouseButton::Middle,
                "x1" => MouseButton::X1,
                "x2" => MouseButton::X2,
                _ => return Err(serde::de::Error::custom(format!("非法鼠标键: {val:?}"))), // i18n:allow 配置损坏时的诊断信息（serde 原文混排，不进词条）
            }),
            "kb" => Self::Keyboard(val.parse().map_err(|_| serde::de::Error::custom(format!("非法 VK 码: {val:?}")))?), // i18n:allow 配置损坏时的诊断信息（serde 原文混排，不进词条）
            "wheel" => Self::Wheel(val.parse().map_err(|_| serde::de::Error::custom(format!("非法滚轮格数: {val:?}")))?), // i18n:allow 配置损坏时的诊断信息（serde 原文混排，不进词条）
            _ => return Err(serde::de::Error::custom(format!("非法键类别: {kind:?}"))), // i18n:allow 配置损坏时的诊断信息（serde 原文混排，不进词条）
        })
    }
}

// ================================================================ 动作

/// 手势识别后执行的动作；未知类型保留原始 JSON。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    /// 按键/文字序列（语法见 [`crate::keyseq`]）。
    KeySequence {
        keys: String,
        #[serde(default)]
        mode: SendMode,
        #[serde(default)]
        secured: bool,
        /// 执行前先激活目标窗口。
        #[serde(default)]
        activate_target: bool,
    },
    /// Shell 脚本（cmd /C，支持 `%MS_*%` 变量，M4）。
    Shell { script: String },
    /// Lua 脚本，入口 `function ms_action(...)`（M4，mlua）。
    Lua { code: String },
    /// 打开 URL 模板（`{MS_SELECTED_TEXT}` 等占位符，M4）。
    OpenUrl {
        url_template: String,
        #[serde(default)]
        browser: Option<String>,
    },
    /// 弹出自定义菜单（可递归嵌套）。
    Menu {
        columns: u32,
        items: Vec<MenuItem>,
    },
    /// 运行/激活应用程序（M4）。
    LaunchApp {
        exe: String,
        #[serde(default)]
        args: Option<String>,
        #[serde(default)]
        elevate: bool,
        #[serde(default)]
        win_class: Option<String>,
        #[serde(default)]
        win_title: Option<String>,
        #[serde(default)]
        regex: bool,
        #[serde(default)]
        if_running: IfRunning,
        /// 仅预置“打开应用”按程序身份激活；旧动作仍按窗口条件查找。
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        activate_by_identity: bool,
    },
    /// 控制目标窗口。
    WindowOp {
        op: WindowOpKind,
        #[serde(default)]
        allow_child: bool,
    },
    /// 系统功能键（亮度/媒体等）。
    SystemKey { key: String },
    /// 预置动作在执行时按平台和界面语言展开。
    Preset {
        id: String,
        #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
        params: serde_json::Map<String, serde_json::Value>,
    },
    /// 较新版本的动作；存盘时不增加包装，也不迁移其内容。
    #[serde(untagged)]
    Unknown(serde_json::Value),
}

// 只反序列化已知标签；新增类型时与 Action 字段、KNOWN_TYPES 一起维护。
#[derive(Deserialize)]
#[serde(remote = "Action", tag = "type", rename_all = "snake_case")]
enum KnownAction {
    KeySequence {
        keys: String,
        #[serde(default)]
        mode: SendMode,
        #[serde(default)]
        secured: bool,
        /// 执行前先激活目标窗口。
        #[serde(default)]
        activate_target: bool,
    },
    Shell { script: String },
    Lua { code: String },
    OpenUrl {
        url_template: String,
        #[serde(default)]
        browser: Option<String>,
    },
    Menu {
        columns: u32,
        items: Vec<MenuItem>,
    },
    LaunchApp {
        exe: String,
        #[serde(default)]
        args: Option<String>,
        #[serde(default)]
        elevate: bool,
        #[serde(default)]
        win_class: Option<String>,
        #[serde(default)]
        win_title: Option<String>,
        #[serde(default)]
        regex: bool,
        #[serde(default)]
        if_running: IfRunning,
        #[serde(default)]
        activate_by_identity: bool,
    },
    WindowOp {
        op: WindowOpKind,
        #[serde(default)]
        allow_child: bool,
    },
    SystemKey { key: String },
    Preset {
        id: String,
        #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
        params: serde_json::Map<String, serde_json::Value>,
    },
}

impl<'de> Deserialize<'de> for Action {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = serde_json::Value::deserialize(d)?;
        let tag = raw.get("type").and_then(serde_json::Value::as_str)
            .ok_or_else(|| serde::de::Error::custom("action.type must be a string"))?;
        // 未收录的预置动作也保留完整对象，避免丢失未来版本增加的字段。
        if tag == "preset" && raw.get("id").and_then(serde_json::Value::as_str).is_some_and(|id| crate::presets::actions::find(id).is_none()) {
            return Ok(Self::Unknown(raw));
        }
        if Self::KNOWN_TYPES.contains(&tag) {
            KnownAction::deserialize(raw).map_err(serde::de::Error::custom)
        } else {
            Ok(Self::Unknown(raw))
        }
    }
}

/// 按键输出模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SendMode {
    #[default]
    DownUp,
    /// 触发键释放时同步释放。
    Mapping,
    Toggle,
    Down,
    Up,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IfRunning {
    #[default]
    Activate,
    SendKeys,
    RunAgain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowOpKind {
    ToggleMaximize,
    Minimize,
    Close,
    ToggleTopmost,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MenuItem {
    pub label: String,
    pub action: Action,
}

impl Action {
    /// 带参数或提权的启动必须交给程序处理，不能只聚焦已有窗口。
    pub fn activates_app_identity(&self) -> bool {
        matches!(self, Self::LaunchApp { activate_by_identity: true, elevate: false, args, win_class: None, win_title: None, if_running: IfRunning::Activate, .. } if args.is_none())
    }

    pub const KNOWN_TYPES: &'static [&'static str] = &[
        "key_sequence", "shell", "lua", "open_url", "menu", "launch_app", "window_op", "system_key", "preset",
    ];

    /// 深度优先访问本动作及全部嵌套动作（Menu 递归）。
    pub fn visit(&self, f: &mut impl FnMut(&Action)) {
        f(self);
        if let Action::Menu { items, .. } = self {
            for item in items {
                item.action.visit(f);
            }
        }
    }

    /// 可变版本的 [`Action::visit`]。
    pub fn visit_mut(&mut self, f: &mut impl FnMut(&mut Action)) {
        f(self);
        if let Action::Menu { items, .. } = self {
            for item in items {
                item.action.visit_mut(f);
            }
        }
    }
}

// ================================================================ 设置

/// settings（对应旧格式 prefs.json，字段重命名为 snake_case）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct Settings {
    /// 界面语言：`auto`（跟随系统）| `zh-CN` | `en-US`。
    /// 用字符串而不是枚举：将来新增的语言写进配置后，旧版本读到不认识的值也不会解析失败。
    #[serde(default = "default_language", skip_serializing_if = "is_auto")]
    pub language: String,
    /// 按下触发键后超此时间(ms)未移动则放行为普通点击。
    pub drag_timeout_ms: u32,
    pub drag_timeout_hint: bool,
    /// 轨迹最小识别距离(px)。
    pub min_stroke_px: u32,
    /// under_cursor（鼠标下窗口）| focused（活动窗口）。
    pub target_mode: TargetMode,
    /// 暂停/恢复热键（keyseq 语法）。
    pub pause_hotkey: String,
    pub show_tray: bool,
    /// 默认每天检查更新，可在偏好设置关闭；检查记录另存，不混入配置。
    #[serde(default = "yes")]
    pub update_check: bool,
    pub trail: Trail,
    pub label: Label,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetMode {
    #[default]
    UnderCursor,
    Focused,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct Trail {
    pub show: bool,
    pub width: f64,
    pub border: u32,
    /// 执行后淡出。
    pub fade_out: bool,
    /// "#RRGGBBAA"（6 位视为不透明）。
    pub color_normal: String,
    pub color_matched: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct Label {
    pub show: bool,
    /// 标签在轨迹上的位置（0..1）。
    pub pos: f64,
    pub border: u32,
    pub color_normal: String,
    pub color_matched: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: default_language(),
            drag_timeout_ms: 500,
            drag_timeout_hint: true,
            min_stroke_px: 30,
            target_mode: TargetMode::UnderCursor,
            pause_hotkey: "#({ESC})".into(),
            show_tray: true,
            update_check: true,
            trail: Trail {
                show: true,
                width: 2.0,
                border: 1,
                fade_out: true,
                // #RRGGBBAA：绘制中亮青蓝、命中橙色
                color_normal: "#00C2FFE0".into(),
                color_matched: "#FF7A00F2".into(),
            },
            label: Label {
                show: true,
                pos: 0.25,
                border: 1,
                color_normal: "#FFFFFFE6".into(),
                color_matched: "#FF7A00F2".into(),
            },
        }
    }
}

impl Default for Trail {
    fn default() -> Self {
        Settings::default().trail
    }
}

impl Default for Label {
    fn default() -> Self {
        Settings::default().label
    }
}

impl Config {
    /// 遍历全部作用域下的所有绑定。
    pub fn all_bindings(&self) -> impl Iterator<Item = &Binding> {
        self.scopes.iter().flat_map(|s| s.bindings.iter())
    }

    /// 从 JSON 文本解析（容忍 UTF-8 BOM），并做幂等迁移。
    pub fn from_json(text: &str) -> Result<Self, String> {
        let mut cfg: Self = serde_json::from_str(text.trim_start_matches('\u{feff}')).map_err(|e| e.to_string())?;
        cfg.migrate();
        Ok(cfg)
    }

    /// 幂等迁移：早期导入器未转换的旧格式占位符（`{<旧前缀>_SELECTED_TEXT}` 等）→ `%MS_*%`；
    /// 曾被改写进配置的内置中文名 → 补上 `name_key`（`name` 原样不动）。
    pub fn migrate(&mut self) {
        use crate::vars::convert_legacy_placeholders as conv;
        crate::names::attach_name_keys(self);
        for scope in &mut self.scopes {
            for b in &mut scope.bindings {
                if b.draft {
                    b.disabled = true;
                }
                b.action.visit_mut(&mut |a| match a {
                    Action::Shell { script } => *script = conv(script),
                    Action::OpenUrl { url_template, .. } => *url_template = conv(url_template),
                    Action::LaunchApp { args: Some(args), .. } => *args = conv(args),
                    _ => {}
                });
            }
        }
    }

    /// 菜单里的预置动作也要求 v2；保留未来版本号。
    pub fn saved_version(&self) -> u32 {
        let mut version = self.version;
        for binding in self.all_bindings() {
            binding.action.visit(&mut |a| {
                if matches!(a, Action::Preset { .. }) || matches!(a, Action::Unknown(raw) if raw.get("type").and_then(serde_json::Value::as_str) == Some("preset")) {
                    version = version.max(2);
                }
            });
        }
        version
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("Config 序列化不会失败") // i18n:allow 配置损坏时的诊断信息（serde 原文混排，不进词条）
    }

    /// 内置默认配置：首次运行（还没有配置文件）时使用并写盘。
    /// 两平台共用一份手势；名称按当前界面语言本地化。
    pub fn builtin() -> Self {
        Self::builtin_for(crate::caps::Platform::CURRENT)
    }

    /// 指定平台的内置默认配置（设置界面与测试可取另一平台的默认值）。
    pub fn builtin_for(platform: crate::caps::Platform) -> Self {
        let mut cfg = Self::from_json(include_str!("../assets/default-config.json")).expect("内置配置必须能解析"); // i18n:allow 内置资源损坏属于编译期错误，测试覆盖
        if platform == crate::caps::Platform::Macos {
            cfg.scopes.retain(|scope| !matches!(scope.id.as_str(), "explorer" | "cmd"));
            cfg.settings.pause_hotkey = "^%#p".into();
            for binding in cfg.scopes.iter_mut().flat_map(|scope| &mut scope.bindings) {
                match binding.name_key.as_deref() {
                    Some("builtin.gesture.topmost" | "builtin.gesture.task_switcher" | "builtin.gesture.brighter" | "builtin.gesture.darker") => binding.disabled = true,
                    Some("builtin.gesture.switch_app") => {
                        binding.name_key = Some("builtin.gesture.mission_control".into());
                        binding.action = Action::Preset { id: "desktop.task_view".into(), params: Default::default() };
                    }
                    Some("builtin.gesture.start") => binding.name_key = Some("builtin.gesture.spotlight".into()),
                    Some("builtin.gesture.task_manager") => binding.execute_on_match = true,
                    _ => {}
                }
            }
        } else {
            cfg.scopes.retain(|scope| !scope.app.as_ref().is_some_and(|app| crate::presets::MAC_DISABLE_SUGGESTIONS.iter().any(|(identity, _, _)| app.exe == *identity)));
        }
        crate::names::localize_builtin(&mut cfg);
        cfg
    }

    /// 读取配置文件；文件不存在时返回 `Ok(None)`（首次运行）。
    pub fn load_existing(path: &std::path::Path) -> Result<Option<Self>, String> {
        if !path.exists() {
            return Ok(None);
        }
        Self::load(path).map(Some)
    }

    /// 读取配置；首次运行（文件不存在）写入内置默认值，设置界面随后读到的也是这一份。
    /// 读取失败直接返回错误；第二项是解析失败或首次写盘失败的原因，调用方须通知用户并暂停设置写盘。
    pub fn load_or_init(path: &std::path::Path) -> Result<(Self, Option<String>), String> {
        let bytes = match std::fs::read(path) {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(format!("读取 {} 失败: {error}", path.display())), // i18n:allow 配置损坏时的诊断信息
        };
        if let Some(bytes) = bytes {
            // 已读到的损坏内容留给用户恢复；只在内存回落，由调用方显示诊断。
            let parsed = std::str::from_utf8(&bytes).map_err(|e| e.to_string()).and_then(Self::from_json);
            return match parsed {
                Ok(cfg) => Ok((cfg, None)),
                Err(error) => Ok((Self::builtin(), Some(format!("解析 {} 失败: {error}", path.display())))), // i18n:allow 配置损坏时的诊断信息
            };
        }
        let cfg = Self::builtin();
        let saved = cfg.save_live(path).err();
        Ok((cfg, saved))
    }

    /// 读取配置文件。
    pub fn load(path: &std::path::Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("读取 {} 失败: {e}", path.display()))?; // i18n:allow 配置损坏时的诊断信息（serde 原文混排，不进词条）
        Self::from_json(&text).map_err(|e| format!("解析 {} 失败: {e}", path.display())) // i18n:allow 配置损坏时的诊断信息（serde 原文混排，不进词条）
    }

    /// 原子写入配置文件（先写临时文件再改名，避免引擎热加载读到半截内容）。
    pub fn save(&self, path: &std::path::Path) -> Result<(), String> {
        self.write_atomic(path)
    }

    /// 引擎与设置的当前配置才需要损坏保护和 v1 备份；导出可覆盖普通文件。
    pub fn save_live(&self, path: &std::path::Path) -> Result<(), String> {
        if path.exists() {
            let original = std::fs::read(path).map_err(|e| e.to_string())?;
            let text = std::str::from_utf8(&original).map_err(|e| e.to_string())?;
            Self::from_json(text)?;
            let raw: serde_json::Value = serde_json::from_str(text.trim_start_matches('\u{feff}')).map_err(|e| e.to_string())?;
            if raw.get("version").and_then(serde_json::Value::as_u64).unwrap_or(1) == 1 && self.saved_version() >= 2 {
                backup_v1(path, &original)?;
            }
        }
        self.write_atomic(path)
    }

    fn write_atomic(&self, path: &std::path::Path) -> Result<(), String> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("创建目录 {} 失败: {e}", dir.display()))?; // i18n:allow 配置损坏时的诊断信息（serde 原文混排，不进词条）
        }
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, self.to_json()).map_err(|e| format!("写入 {} 失败: {e}", tmp.display()))?; // i18n:allow 配置损坏时的诊断信息（serde 原文混排，不进词条）
        std::fs::rename(&tmp, path).map_err(|e| format!("替换 {} 失败: {e}", path.display())) // i18n:allow 配置损坏时的诊断信息（serde 原文混排，不进词条）
    }

    /// 显式恢复：先完整保留坏文件，再写默认值；失败时恢复原文件。
    pub fn recover_defaults(path: &std::path::Path, now: std::time::SystemTime) -> Result<Self, String> {
        Self::recover_to(path, Self::builtin(), now)
    }

    /// 显式恢复为指定配置：先完整备份坏文件，写入失败时回滚。
    pub fn recover_to(path: &std::path::Path, mut cfg: Self, now: std::time::SystemTime) -> Result<Self, String> {
        if Self::load(path).is_ok() {
            return Err(crate::i18n::t("config.recovery_readable").into());
        }
        cfg.migrate();
        let backup = path.with_file_name(format!("config.broken-{}.json", backup_timestamp(now)?));
        // 独占创建避免同一秒内的第二次恢复覆盖上次保留的原件。
        let reservation = std::fs::OpenOptions::new().write(true).create_new(true).open(&backup).map_err(|e| e.to_string())?;
        drop(reservation);
        if let Err(error) = std::fs::rename(path, &backup) {
            let _ = std::fs::remove_file(&backup);
            return Err(error.to_string());
        }
        if let Err(error) = cfg.write_atomic(path) {
            let _ = std::fs::rename(&backup, path);
            return Err(error);
        }
        Ok(cfg)
    }
}

/// UTC 时间戳，避免依赖平台时区 API；公历换算以 400 年为周期。
fn backup_timestamp(now: std::time::SystemTime) -> Result<String, String> {
    let seconds = now.duration_since(std::time::UNIX_EPOCH).map_err(|e| e.to_string())?.as_secs();
    let days = i64::try_from(seconds / 86400).map_err(|e| e.to_string())? + 719468;
    let era = days / 146097;
    let day = days - era * 146097;
    let year_in_era = (day - day / 1460 + day / 36524 - day / 146096) / 365;
    let day_of_year = day - (365 * year_in_era + year_in_era / 4 - year_in_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let date = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = month_index + if month_index < 10 { 3 } else { -9 };
    let year = year_in_era + era * 400 + i64::from(month <= 2);
    Ok(format!("{year:04}{month:02}{date:02}-{:02}{:02}{:02}", seconds / 3600 % 24, seconds / 60 % 60, seconds % 60))
}

impl Serialize for Config {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Config", 3)?;
        state.serialize_field("version", &self.saved_version())?;
        state.serialize_field("scopes", &self.scopes)?;
        state.serialize_field("settings", &self.settings)?;
        state.end()
    }
}

/// 使用独占创建，已有备份绝不覆盖；保留原文件的字节（含 BOM 和格式）。
fn backup_v1(path: &std::path::Path, original: &[u8]) -> Result<(), String> {
    use std::io::Write;
    let backup = path.with_file_name("config.v1.bak.json");
    let file = std::fs::OpenOptions::new().write(true).create_new(true).open(&backup);
    let mut file = match file {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists && backup.is_file() => return Ok(()),
        Err(e) => return Err(e.to_string()),
    };
    if let Err(e) = file.write_all(original).and_then(|()| file.sync_all()) {
        drop(file);
        let _ = std::fs::remove_file(&backup);
        return Err(e.to_string());
    }
    Ok(())
}

/// 默认配置文件路径：Windows 使用 APPDATA；macOS 使用 Library/Application Support。
pub fn default_config_path() -> std::path::PathBuf {
    #[cfg(windows)]
    let base = std::env::var_os("APPDATA")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".config")))
        .unwrap_or_else(|| ".".into());
    #[cfg(target_os = "macos")]
    let base = std::env::var_os("HOME")
        .map(|h| std::path::PathBuf::from(h).join("Library/Application Support"))
        .unwrap_or_else(|| ".".into());
    #[cfg(not(any(windows, target_os = "macos")))]
    let base = std::env::var_os("HOME")
        .map(|h| std::path::PathBuf::from(h).join(".config"))
        .unwrap_or_else(|| ".".into());
    base.join("mousetroke").join("config.json")
}

#[cfg(test)]
mod builtin_tests {
    use super::*;

    #[test]
    fn both_platforms_share_defaults_and_keep_platform_preferences() {
        use crate::caps::Platform;
        use crate::presets::actions::{expand, find, Params};
        for platform in [Platform::Windows, Platform::Macos] {
            let cfg = Config::builtin_for(platform);
            assert_eq!(cfg.version, 2);
            assert_eq!(cfg.scopes.len(), if platform == Platform::Macos { 1 + crate::presets::MAC_DISABLE_SUGGESTIONS.len() } else { 3 });
            if platform == Platform::Macos {
                for (identity, _, _) in crate::presets::MAC_DISABLE_SUGGESTIONS {
                    assert!(cfg.scopes.iter().any(|scope| scope.disabled && scope.app.as_ref().is_some_and(|app| app.exe == *identity)));
                }
                assert!(cfg.scopes.iter().all(|scope| !matches!(scope.id.as_str(), "explorer" | "cmd")));
            }
            for binding in cfg.all_bindings() {
                binding.action.visit(&mut |action| {
                    if let Action::Preset { id, params } = action {
                        assert!(find(id).is_some(), "{id}");
                        let expanded = expand(id, platform, params, crate::i18n::Lang::EnUs);
                        if let Ok(Action::KeySequence { keys, .. }) = expanded {
                            let seq = crate::keyseq::parse(&keys, false).unwrap();
                            assert!(crate::caps::keys_supported(&seq, platform));
                        }
                    }
                });
            }
            let close = expand("browser.close_tab", platform, &Params::new(), crate::i18n::Lang::EnUs).unwrap();
            assert!(matches!(close, Action::KeySequence { keys, .. } if keys == if platform == Platform::Macos { "#w" } else { "^w" }));
            let rubbing = cfg.all_bindings().find(|b| b.name_key.as_deref() == Some("builtin.gesture.task_manager")).unwrap();
            assert!(!rubbing.disabled);
            assert_eq!(rubbing.execute_on_match, platform == Platform::Macos);
            assert_eq!(rubbing.pattern, [Step::Edge { edge: crate::edge::RIGHT }, Step::Trace { points: vec![50,50,50,0,50,50,50,0,50,50] }]);
            let corner = |edge| cfg.all_bindings().find(|b| b.pattern == [Step::Edge { edge }]).unwrap();
            assert_eq!(corner(crate::edge::RIGHT_TOP).disabled, platform == Platform::Macos);
            assert_eq!(cfg.settings.pause_hotkey, if platform == Platform::Macos { "^%#p" } else { "#({ESC})" });
            let expected = if platform == Platform::Macos { "desktop.task_view" } else { "desktop.switch_app" };
            assert!(matches!(&corner(crate::edge::LEFT_TOP).action, Action::Preset { id, .. } if id == expected));
        }
    }

    #[test]
    fn config_path_uses_the_platform_directory() {
        use std::path::PathBuf;
        #[cfg(windows)]
        let base = std::env::var_os("APPDATA").map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
            .unwrap_or_else(|| ".".into());
        #[cfg(target_os = "macos")]
        let base = std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support"))
            .unwrap_or_else(|| ".".into());
        #[cfg(not(any(windows, target_os = "macos")))]
        let base = std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config"))
            .unwrap_or_else(|| ".".into());
        assert_eq!(default_config_path(), base.join("mousetroke/config.json"));
    }

    #[test]
    fn builtin_config_is_complete_and_valid() {
        let cfg = Config::builtin();
        let bindings: Vec<&Binding> = cfg.all_bindings().collect();
        assert!(bindings.len() >= 40, "内置手势太少：{}", bindings.len());
        for b in &bindings {
            let key = b.name_key.as_deref().unwrap_or_else(|| panic!("{} 缺少 name_key", b.name));
            assert!(crate::i18n::try_t(key).is_some(), "缺少词条 {key}");
            assert!(!b.pattern.is_empty(), "{} 没有手势", b.name);
            b.action.visit(&mut |a| {
                if let Action::KeySequence { keys, .. } = a {
                    crate::keyseq::parse(keys, false).unwrap_or_else(|e| panic!("{} 的按键序列 {keys:?} 无效：{e:?}", b.name));
                }
            });
        }
        // 同一作用域里不能有两条一模一样的手势（后一条永远不会触发）
        for scope in &cfg.scopes {
            for (i, a) in scope.bindings.iter().enumerate() {
                for b in &scope.bindings[i + 1..] {
                    assert!(a.pattern != b.pattern, "{} 与 {} 手势重复", a.name, b.name);
                }
            }
        }
    }

    #[test]
    fn elevated_launch_cannot_activate_by_identity() {
        let mut action: Action = serde_json::from_str(r#"{"type":"launch_app","exe":"editor.exe","activate_by_identity":true,"elevate":false}"#).unwrap();
        assert!(action.activates_app_identity());
        if let Action::LaunchApp { elevate, .. } = &mut action { *elevate = true; }
        assert!(!action.activates_app_identity());
        if let Action::LaunchApp { elevate, .. } = &mut action { *elevate = false; }
        assert!(action.activates_app_identity());
    }

    #[test]
    fn missing_file_is_first_run() {
        let p = std::env::temp_dir().join("mousetroke-no-such-config.json");
        assert_eq!(Config::load_existing(&p), Ok(None));
    }
}

#[cfg(test)]
mod update_schema_tests {
    use super::*;
    #[test]
    fn update_check_defaults_on_and_explicit_false_round_trips() {
        let mut cfg: Config = serde_json::from_str("{}").unwrap();
        assert!(cfg.settings.update_check);
        assert!(Config::builtin().settings.update_check);
        assert!(serde_json::from_str::<Config>(r#"{"settings":{}}"#).unwrap().settings.update_check);
        cfg.settings.update_check = false;
        let copy: Config = serde_json::from_str(&serde_json::to_string(&cfg).unwrap()).unwrap();
        assert!(!copy.settings.update_check);
        assert!(!serde_json::from_str::<Config>(r#"{"settings":{"update_check":false}}"#).unwrap().settings.update_check);
    }
}
