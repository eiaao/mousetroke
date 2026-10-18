//! i18n 词条：扁平 JSON（点号命名空间，值一律为字符串）。
//!
//! - **新增文案先写 `zh-CN.json`（源语言）**，再补其他语言；两边的键集合与
//!   占位符由 `crates/mousetroke-core/tests/i18n_catalog.rs` 校验，缺了测试就红。
//! - 程序里不要写字符串字面量：Rust 用 `mousetroke_core::i18n::keys::*` 常量
//!   （build.rs 生成，写错键名编译不过）；前端用 `t("…")`（键名由类型检查）。
//! - 占位符 `{name}`，只允许 `[a-z_][a-z0-9_]*`。复数拆 `key_one` / `key_other`，
//!   调用方写基础键并传 `count`。强调用一对 `**…**`（由界面渲染成粗体），其他 HTML 不允许。
//! - 伪语言 `qps-ploc`（重音 + 拉长 40%）由运行时生成，不提交文件。
//! - `button.*` / `edge.*` 是独立标签；带动词的句子用完整词条
//!   `ui.pattern.hold_btn.*`、`ui.pattern.click_btn.*`、`ui.pattern.move_to_edge.*`，
//!   不把独立名称插入句子，以免丢失冠词、性别或格变化。
//!   同一语言的按住、点击、绘制和移动步骤保持一致语气（法/西/意/葡使用不定式）。
//!
//! 术语与语气见 `docs/I18N-DESIGN.md` 第 8 节，要点：
//! - “停用（单个手势）”= Turn off；“禁用手势（在某个程序里）”= Turn off gestures
//!   in this app。两个概念，**不能共用词条**。
//!   状态标签同理：手势“已停用”= Off；程序“已禁用”= Gestures off（别都写成 Off，并排出现时分不清）。
//! - 英文用 sentence case，撇号用 `’`；中文全角标点只出现在 zh 词条。
//! - 不翻译：用户自己的数据、运行日志、Lua API、Ctrl/F5 这类键名。
