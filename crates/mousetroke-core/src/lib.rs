//! mousetroke-core —— Mousetroke 核心 crate。
//!
//! 命名约定：
//! - **Scope**：手势的作用域（global / app / special）
//! - **Binding**：一条手势绑定 = `Pattern + Action`
//! - **Pattern**：手势模式，由若干 `Step` 组成（press / trace / edge / wheel）
//! - **Action**：识别成功后执行的动作（8 种）
//!
//! 模块：
//! - [`schema`]：原生配置 schema（我们自己的 snake_case JSON 格式）
//! - [`edge`]：屏幕边/角编码（与旧格式 RectEdge/RectCorner 一致）与命中判定
//! - [`legacy`]：旧格式 gestures/prefs.json → 原生配置的导入器（迁移用）
//! - [`keyseq`]：按键序列语法解析（AutoIt/SendKeys 风格语法）
//! - [`matcher`]：捕获会话 → 步骤序列 → 作用域选择与绑定匹配
//! - [`recognizer`]：轨迹量化与折线匹配
//! - [`vars`]：`%MS_*%` 变量系统（含旧格式占位符转换）

pub mod edge;
pub mod i18n;
pub mod legacy;
pub mod keyseq;
pub mod matcher;
pub mod names;
pub mod presets;
pub mod recognizer;
pub mod schema;
pub mod vars;
