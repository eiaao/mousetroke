//! 原生配置 schema —— Mousetroke 自己的 snake_case JSON 格式。
//!
//! 与旧格式解耦：旧格式文件仅作为导入来源（[`crate::legacy`]），
//! 运行时只读写本格式。字段命名遵循本 crate 词汇表（Scope/Binding/Pattern/Action）。

use serde::{Deserialize, Serialize};

// ================================================================ 配置根

/// config.json 根结构。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

/// 手势识别后执行的动作（8 种）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    },
    /// 控制目标窗口。
    WindowOp {
        op: WindowOpKind,
        #[serde(default)]
        allow_child: bool,
    },
    /// 系统功能键（亮度/媒体等）。
    SystemKey { key: String },
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
                b.action.visit_mut(&mut |a| match a {
                    Action::Shell { script } => *script = conv(script),
                    Action::OpenUrl { url_template, .. } => *url_template = conv(url_template),
                    Action::LaunchApp { args: Some(args), .. } => *args = conv(args),
                    _ => {}
                });
            }
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("Config 序列化不会失败") // i18n:allow 配置损坏时的诊断信息（serde 原文混排，不进词条）
    }

    /// 内置默认配置：首次运行（还没有配置文件）时使用并写盘。
    /// Windows 与 macOS 各自使用内置资源；名称按当前界面语言本地化。
    pub fn builtin() -> Self {
        Self::builtin_for(crate::caps::Platform::CURRENT)
    }

    /// 指定平台的内置默认配置（设置界面与测试可取另一平台的默认值）。
    pub fn builtin_for(platform: crate::caps::Platform) -> Self {
        let source = match platform {
            crate::caps::Platform::Macos => include_str!("../assets/default-config-macos.json"),
            crate::caps::Platform::Windows => include_str!("../assets/default-config.json"),
        };
        let mut cfg = Self::from_json(source).expect("内置配置必须能解析"); // i18n:allow 内置资源损坏属于编译期错误，测试覆盖
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

    /// 读取配置文件。
    pub fn load(path: &std::path::Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("读取 {} 失败: {e}", path.display()))?; // i18n:allow 配置损坏时的诊断信息（serde 原文混排，不进词条）
        Self::from_json(&text).map_err(|e| format!("解析 {} 失败: {e}", path.display())) // i18n:allow 配置损坏时的诊断信息（serde 原文混排，不进词条）
    }

    /// 原子写入配置文件（先写临时文件再改名，避免引擎热加载读到半截内容）。
    pub fn save(&self, path: &std::path::Path) -> Result<(), String> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("创建目录 {} 失败: {e}", dir.display()))?; // i18n:allow 配置损坏时的诊断信息（serde 原文混排，不进词条）
        }
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, self.to_json()).map_err(|e| format!("写入 {} 失败: {e}", tmp.display()))?; // i18n:allow 配置损坏时的诊断信息（serde 原文混排，不进词条）
        std::fs::rename(&tmp, path).map_err(|e| format!("替换 {} 失败: {e}", path.display())) // i18n:allow 配置损坏时的诊断信息（serde 原文混排，不进词条）
    }
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
    fn mac_defaults_enable_supported_actions_and_keep_topmost_disabled() {
        let source = include_str!("../assets/default-config-macos.json");
        let cfg = Config::from_json(source).unwrap();
        assert_eq!(cfg.scopes.len(), 1 + crate::presets::MAC_DISABLE_SUGGESTIONS.len());
        for (identity, _, _) in crate::presets::MAC_DISABLE_SUGGESTIONS {
            assert!(cfg.scopes.iter().any(|scope| scope.disabled && scope.app.as_ref().is_some_and(|app| app.exe == *identity)));
        }
        assert_eq!(cfg.settings.pause_hotkey, "^%#p");
        assert!(cfg.all_bindings().count() >= 40);
        for scope in &cfg.scopes {
            assert!(!matches!(scope.id.as_str(), "explorer" | "cmd"));
            if scope.app.is_some() {
                assert!(scope.disabled && scope.bindings.is_empty());
            }
            for binding in &scope.bindings {
                assert!(crate::i18n::try_t(binding.name_key.as_deref().unwrap()).is_some());
                binding.action.visit(&mut |action| {
                    if let Action::KeySequence { keys, .. } = action {
                        let seq = crate::keyseq::parse(keys, false).unwrap();
                        for step in seq.actions {
                            if let crate::keyseq::Action::Key { key: crate::keyseq::Key::Vk(vk), .. } = step {
                                assert!(crate::keymap::vk_to_kvk(vk).is_some());
                            }
                        }
                    } else if matches!(action, Action::WindowOp { op: WindowOpKind::ToggleTopmost, .. }) {
                        assert!(binding.disabled);
                    } else {
                        assert!(!matches!(action, Action::Lua { .. }));
                        if !matches!(action, Action::SystemKey { key } if key.starts_with("BRIGHTNESS_")) {
                            assert!(!binding.disabled);
                        }
                    }
                });
            }
        }
        let lower = source.to_ascii_lowercase();
        for windows_only in [".exe", "c:\\", "taskkill", "start cmd", "explorer"] {
            assert!(!lower.contains(windows_only), "{windows_only}");
        }
        let key = |name: &str| cfg.all_bindings().find(|b| b.name_key.as_deref() == Some(name)).unwrap().action.clone();
        for (name, expected) in [("back", "#["), ("forward", "#]"), ("close_tab", "#w"),
            ("home", "#{UP}"), ("end", "#{DOWN}"),
            ("prev_tab", "^+{TAB}"), ("next_tab", "^{TAB}")] {
            assert!(matches!(key(&format!("builtin.gesture.{name}")), Action::KeySequence { keys, .. } if keys == expected));
        }
        assert!(matches!(key("builtin.gesture.minimize"), Action::WindowOp { op: WindowOpKind::Minimize, .. }));
        assert!(matches!(key("builtin.gesture.maximize"), Action::WindowOp { op: WindowOpKind::ToggleMaximize, .. }));
        assert!(matches!(key("builtin.gesture.kill"), Action::Shell { script } if script == "/bin/kill -9 %MS_PID%"));
        assert!(matches!(key("builtin.gesture.task_manager"), Action::LaunchApp { exe, .. } if exe == "com.apple.ActivityMonitor"));
        assert!(matches!(key("builtin.gesture.system_settings"), Action::LaunchApp { exe, .. } if exe == "com.apple.systempreferences"));
        let corner = |edge| cfg.all_bindings().find(|b| b.pattern == [Step::Edge { edge }]).unwrap();
        for (edge, name, keys) in [
            (crate::edge::LEFT_TOP, "builtin.gesture.mission_control", "^{UP}"),
            (crate::edge::LEFT_BOTTOM, "builtin.gesture.spotlight", "#{SPACE}"),
            (crate::edge::RIGHT_BOTTOM, "builtin.gesture.show_desktop", "{F11}"),
        ] {
            let binding = corner(edge);
            assert!(!binding.disabled);
            assert_eq!(binding.name_key.as_deref(), Some(name));
            assert!(matches!(&binding.action, Action::KeySequence { keys: actual, .. } if actual == keys));
        }
        assert!(corner(crate::edge::RIGHT_TOP).disabled);
        for (name, edge, modifier, system) in [
            ("volume_up", crate::edge::TOP, PatternKey::Wheel(-1), "VOLUME_UP"),
            ("volume_down", crate::edge::TOP, PatternKey::Wheel(1), "VOLUME_DOWN"),
            ("mute", crate::edge::TOP, PatternKey::Mouse(MouseButton::Middle), "MUTE"),
            ("play_pause", crate::edge::TOP, PatternKey::Mouse(MouseButton::Right), "PLAY_PAUSE"),
            ("brighter", crate::edge::BOTTOM, PatternKey::Wheel(-1), "BRIGHTNESS_UP"),
            ("darker", crate::edge::BOTTOM, PatternKey::Wheel(1), "BRIGHTNESS_DOWN"),
        ] {
            let name = format!("builtin.gesture.{name}");
            let binding = cfg.all_bindings().find(|b| b.name_key.as_deref() == Some(&name)).unwrap();
            assert_eq!(binding.pattern, [Step::Edge { edge }, Step::Press { key: modifier }]);
            assert!(matches!(&binding.action, Action::SystemKey { key } if key == system));
            assert_eq!(binding.disabled, edge == crate::edge::BOTTOM);
        }
        let rubbing = cfg.all_bindings().find(|b| b.name_key.as_deref() == Some("builtin.gesture.task_manager")).unwrap();
        assert!(!rubbing.disabled);
        assert!(rubbing.execute_on_match, "无按键的摩擦手势必须在匹配时执行");
        assert_eq!(rubbing.pattern, [Step::Edge { edge: crate::edge::RIGHT }, Step::Trace { points: vec![50,50,50,0,50,50,50,0,50,50] }]);
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
