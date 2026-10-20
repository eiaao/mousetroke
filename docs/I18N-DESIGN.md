# Mousetroke 国际化（i18n）设计

> 状态：待实现。读者：负责实现的 agent。
> 目标语言：一期 **zh-CN（源语言）+ en-US**；架构须让后续加语言（ja-JP、zh-TW…）只需新增一个文件。
> 构建、测试方法见根目录 `README.md`。本文所有路径相对仓库根目录。

---

## 0. 结论先行

1. **一份共享词条库**：`i18n/zh-CN.json`、`i18n/en-US.json`。Rust（引擎、托盘、命令行、设置后端）与设置界面（TS）读同一批文件，不各维护一份。
2. **键名在编译期检查**：
   - Rust 侧由 `mousetroke-core/build.rs` 生成键名常量，写错键名就编译不过。
   - TS 侧用 `typeof zh` 推导键名类型，写错键名 tsc 报错。
   - 各语言的键集合与占位符必须一致，由测试保证。
3. **后端不再返回拼好的句子**：错误一律以 `{ key, params }` 的形式交给界面，由界面翻译。这样切换语言不需要后端参与。
4. **默认手势名不再改写进配置文件**：给绑定和作用域加一个可选的 `name_key`。显示时，有 `name_key` 且用户没改过名，就按当前语言翻译；用户一旦改名，就清掉 `name_key`。
5. **新设置项 `settings.language`**，取值 `auto` / `zh-CN` / `en-US`：
   - `auto` 时跟随 Windows 界面语言；
   - 设置界面里切换后立即生效；
   - 引擎通过现有的热加载在 1 秒内切换托盘菜单和屏幕提示的语言。
6. **防回归**：
   - 一个 lint 脚本禁止在源码字符串里写死中文（注释不算）；
   - 一个伪语言 `qps-ploc`（字符加重音、文本拉长约 40%）专门用来抓漏翻和布局溢出；
   - UI 冒烟测试分别用 zh-CN、en-US、`qps-ploc` 各跑一遍。

---

## 1. 范围

### 1.1 要翻译的

| 位置 | 内容 | 现状（文件，含中文的非注释行数） |
|---|---|---|
| 设置界面 | 全部界面文字、`data-tip` 提示气泡、`aria-label`、提示条、空状态、引导、关于页 | `ui/src/gesture.ts` 51、`Dialogs.tsx` 38、`BindingList.tsx` 34、`ActionEditor.tsx` 31、`Prefs.tsx` 27、`PatternEditor.tsx` 20、`Guide.tsx` 20、`About.tsx` 18、`Editor.tsx` 15、`App.tsx` 14、`Sidebar.tsx` 13，其余零星 |
| 设置界面后端 | 返回给界面的错误、窗口标题「Mousetroke 设置」 | `mousetroke-settings/src/lib.rs`、`picker.rs`（拖入非程序、快捷方式解析、点选提示条文字） |
| 引擎 | 托盘菜单、托盘悬停提示（szTip）、托盘气泡、屏幕上的 toast（「配置已更新」「亮度 N%」…）、手势名标签里的「未识别」 | `mousetroke-win/src/tray.rs` 20、`engine.rs`、`input.rs`（亮度提示）、`lua.rs`（show_toast 由脚本传入，不译） |
| 核心 | 按键序列语法错误、边/角名、默认手势名对照（`names.rs`）、内置建议禁用清单（`presets.rs`） | `keyseq.rs`、`edge.rs`、`names.rs` 56、`presets.rs` 20 |
| 命令行 | `--help`、`diag` 输出、错误提示 | `mousetroke/src/main.rs` 30 |
| exe 资源 | `FileDescription`（资源管理器里显示） | `mousetroke/build.rs`：保持英文，不做多语言资源 |

### 1.2 不翻译的（明确排除）

- **用户自己的数据**：用户起的手势名、程序名、按键序列、命令、Lua 代码、网址。
- **运行日志** `mousetroke.log`：面向排查问题，单一语言即可。保持现状，新增日志也不需要走词条。
- **Lua API**：`ms.*` / `WG.*` 的函数名，以及脚本自己传给 `show_toast` 的文字。
- **键名与符号**：`Ctrl`、`Alt`、`Shift`、`Win`、`Tab`、`Esc`、F1–F24、方向箭头 ←↑→↓ 通用，不进词条。
  - 但「空格」「小键盘 1」「滚轮上」「左键」这类描述性的名字要进词条。
- **测试里的断言文字**：测试直接断言中文或英文均可，跟随被测语言。
- 不支持从右往左书写的语言（RTL），也不做按地区的日期和数字格式（目前界面里没有日期，数字只有 ms、px、%）。

---

## 2. 词条文件

### 2.1 位置与格式

```
i18n/
  zh-CN.json     源语言，新增文案先写这里
  en-US.json
  README.md      写给翻译者：术语表、语气、占位符规则（内容见本文第 8 节）
```

- 格式是**扁平 JSON**：用点号分隔命名空间，值是字符串。不用嵌套对象，这样两端处理起来最简单，也方便 diff。
- 占位符写作 `{name}`，只允许 `[a-z_][a-z0-9_]*`。
- **复数**（只有英文需要）：同一个词条拆成 `key_one` / `key_other` 两条。调用方只写基础键 `key` 并传入 `count`，由运行时选择后缀。
  - zh-CN 只需提供 `key_other`（也可以不加后缀，直接写 `key`），运行时按“没有 `_one` 就用 `_other`，再没有就用基础键”的顺序回退。
- 文本里的强调用一对 `**…**` 标记，由界面渲染成 `<b>`，引导页正文要用到。其他 HTML 一律不允许。

```json
{
  "tray.menu.settings": "设置…",
  "tray.menu.pause": "暂停手势",
  "tray.menu.resume": "恢复手势",
  "toast.apps_disabled_other": "已在 {count} 个程序里禁用手势",
  "error.not_a_program": "「{file}」不是程序，只能添加 .exe 或它的快捷方式"
}
```

```json
{
  "toast.apps_disabled_one": "Gestures turned off in {count} app",
  "toast.apps_disabled_other": "Gestures turned off in {count} apps"
}
```

### 2.2 键名规范

`<区域>.<页面或组件>.<含义>`，全部小写加下划线。

| 前缀 | 用途 |
|---|---|
| `common.*` | 通用按钮与词：保存、取消、删除、撤销、关闭、浏览… |
| `tray.*` | 托盘菜单、托盘悬停提示、托盘气泡 |
| `overlay.*` | 屏幕上的标签与 toast（`overlay.unrecognized` = “未识别”） |
| `cli.*` | 命令行 |
| `ui.sidebar.*` `ui.list.*` `ui.editor.*` `ui.pattern.*` `ui.action.*` `ui.prefs.*` `ui.data.*` `ui.picker.*` `ui.guide.*` `ui.about.*` `ui.drop.*` | 设置界面，按组件划分 |
| `button.*` | 鼠标键名：`button.right` = “右键” |
| `edge.*` | 边与角：`edge.1` = “上边”…`edge.12` = “左下角”（数字是位标志，与 `mousetroke_core::edge` 一致） |
| `key.*` | 描述性的键名：`key.space`、`key.numpad`（“小键盘 {n}”）、`key.wheel_up`… |
| `keyseq.error.*` | 按键序列语法错误 |
| `error.*` | 后端返回给界面的其他错误 |
| `builtin.gesture.*` `builtin.scope.*` | 默认手势名、默认作用域名（见第 5 节） |
| `preset.app.<exe 小写>` `preset.category.*` | 内置建议禁用清单（见第 5.3 节） |

不要按“这句话在哪一行”命名，要按“这句话是什么意思”命名。同一个意思在多处出现，就共用一个键。

---

## 3. Rust 侧

### 3.1 模块：`mousetroke_core::i18n`

```rust
/// 支持的语言。新增语言：加枚举值 + 加 i18n/<tag>.json，其余自动。
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Lang { ZhCn, EnUs /*, QpsPloc（仅 debug 构建） */ }

impl Lang {
    pub fn tag(self) -> &'static str;              // "zh-CN"
    pub fn from_tag(s: &str) -> Option<Lang>;      // 大小写不敏感；"zh"/"zh-Hans"/"zh-SG" → ZhCn
}

/// 当前语言（进程全局，原子量；引擎在 apply_config 时设置）。
pub fn set_lang(lang: Lang);
pub fn lang() -> Lang;

/// 取词条。缺失时回退到 zh-CN，再缺失返回键名本身（debug 构建同时 eprintln 一次）。
pub fn t(key: &'static str) -> &'static str;
/// 带参数：`tf(keys::TOAST_BRIGHTNESS, &[("value", "80")])`；含 `count` 参数时按复数规则选 `_one/_other`。
pub fn tf(key: &'static str, args: &[(&str, &str)]) -> String;
```

- 词条在编译期嵌入：`include_str!("../../../i18n/zh-CN.json")`，第一次使用时解析成 `HashMap<&'static str, &'static str>`（`OnceLock`，并且 `Box::leak` 成 `'static`，常驻内存约几十 KB）。
- **不引入新依赖**：现有的 `serde_json` 足够用。复数规则手写：en 为 `n == 1 → one`，zh 一律 `other`。
- 宏 `t!(KEY)` / `t!(KEY, name = value, …)` 是 `t` / `tf` 的语法糖，参数值用 `Display` 格式化。

### 3.2 编译期键名常量

`mousetroke-core/build.rs`（新建）：

1. 读 `i18n/zh-CN.json`，为每个键生成常量，写到 `OUT_DIR/i18n_keys.rs`：
   `pub const TRAY_MENU_SETTINGS: &str = "tray.menu.settings";`
   - 生成规则：点号和下划线都换成 `_`，转大写；复数词条只为基础键生成常量。
2. 由 `i18n.rs` 里的 `pub mod keys { include!(concat!(env!("OUT_DIR"), "/i18n_keys.rs")); }` 引入。
3. 加上 `cargo:rerun-if-changed=../../i18n`。

代码里只能写 `keys::TRAY_MENU_SETTINGS`，不写字符串字面量。这样写错键名会编译失败，删掉的键也会让引用它的代码编译失败。

### 3.3 语言的确定

`schema::Settings` 增加一个字段：

```rust
/// 界面语言：auto（跟随系统）| zh-CN | en-US。
#[serde(default = "auto", skip_serializing_if = "is_auto")]
pub language: String,
```

用字符串而不是枚举，原因是将来新增的语言写进配置后，旧版本程序读到不认识的值也不会解析失败。

`mousetroke_core::i18n::resolve(setting: &str, system: &[String]) -> Lang` 是纯函数，便于写测试：
- 设置值不是 `auto` 且认识，就用它；
- 否则依次取系统的首选界面语言列表，第一个认识的就用；
- 都不认识，回退到 en-US。
- `zh-TW`、`zh-HK` 暂时映射到 zh-CN；等加了 zh-TW 词条后改成映射到 zh-TW。

系统语言列表由 `mousetroke-win` 提供：`GetUserPreferredUILanguages(MUI_LANGUAGE_NAME)`，需要 windows crate 的 `Win32_Globalization` 功能集。

### 3.4 各进程如何接入

| 进程或入口 | 什么时候调用 `set_lang` |
|---|---|
| 引擎 | `Engine::apply_config`（启动和热加载时都会经过它）。语言变化时还要调用 `tray::refresh_texts()`：重新设置托盘悬停提示 `szTip`。托盘菜单每次弹出时现建，自然就是新语言。 |
| 设置界面进程 | 启动时按配置决定一次，用于后端自己需要的少量文字（窗口标题、点选窗口时屏幕顶部的提示条）。界面切换语言后调用命令 `set_ui_lang(tag)`：后端 `set_lang`，并调用 `window.set_title(...)`。 |
| 命令行 | `main()` 开头：读到配置就用配置里的设置，否则用系统语言。 |

### 3.5 字体

覆盖层标签（`overlay.rs` 里的 `CreateFontW`）、点选窗口时的提示条（`picker.rs`）都按语言选字体：

| 语言 | 字体 |
|---|---|
| zh-CN | Microsoft YaHei UI |
| zh-TW（将来） | Microsoft JhengHei UI |
| ja-JP（将来） | Yu Gothic UI |
| 其他 | Segoe UI |

做成 `i18n::ui_font(lang) -> &'static str` 一个函数，不要散落在各处判断。字体在覆盖层的暂存资源里是懒创建的，语言变化时要丢弃重建。

---

## 4. 设置界面（TS）

### 4.1 运行时：`ui/src/i18n/index.ts`

```ts
import zh from "../../../../../i18n/zh-CN.json";   // vite 原生支持 JSON；tsconfig 打开 resolveJsonModule
import en from "../../../../../i18n/en-US.json";

type Catalog = typeof zh;
/** 去掉复数后缀后的基础键（"toast.apps_disabled"），以及非复数键。 */
export type MsgKey = BaseKey<keyof Catalog>;

export function t(key: MsgKey, params?: Record<string, string | number>): string;
export function useLang(): [Lang, (l: Lang | "auto") => void];   // React 订阅当前语言
export function T({ k, params }: { k: MsgKey; params?: … }): JSX.Element;  // 渲染 **粗体**，用于引导/横幅这类富文本
```

- 词条文件在前端工程目录之外：`npm run build` 不受影响；用 `vite dev` 调试时要在 `vite.config.ts` 里加上 `server.fs.allow: ["../../.."]`（指向仓库根目录），否则开发服务器拒绝读取。
- en-US 的类型标注为 `satisfies Record<keyof Catalog, string>`：少键就编译不过。多出的键由第 7.1 节的校验测试抓出来。
- `t()` 内部按当前语言取词条，缺失时回退到 zh-CN。复数用 `Intl.PluralRules(lang)` 选择 `_one` / `_other`。
- 语言状态放在一个很小的模块级 store 里（`useSyncExternalStore`），**不要**把 `t` 一层层当 props 往下传。
- 切换语言时同步更新 `document.documentElement.lang`，因为 CSS 字体栈和换行规则都依赖它。

### 4.2 语言的确定

- 设置界面启动时，`init_state` 增加两个返回字段：`systemLangs: string[]`（后端用第 3.3 节的同一个函数取到的系统语言列表）和 `resolvedLang`。前端只认 `config.settings.language` 加上这两个值。**不要**用 `navigator.language`：WebView2 报告的语言与 Windows 界面语言不一定一致。
- 「偏好设置 → 启动」组的第一行加一个「语言」下拉框，选项：跟随系统（当前：简体中文）/ 简体中文 / English。
  - 每个语言选项都用它**自己的语言**显示名称，比如永远显示 English，而不是“英语”；
  - 选择后立即重新渲染界面，并写进配置，由现有的自动保存负责保存；
  - 引擎会自动热加载新设置。

### 4.3 改造要点（不只是替换字符串）

1. **句子要用模板，不要拼接。** 现在 `gesture.ts` 里 `describePattern` 的拼法是 `` `按住${button}` + " + " + `画 ${arrows}` ``，英文语序不同，没法这样拼。改为词条：
   - `ui.pattern.hold`: `按住{button}` / `Hold {button}`
   - `ui.pattern.move_to`: `移到{edge}` / `Push to {edge}`
   - `ui.pattern.draw`: `画 {arrows}` / `draw {arrows}`
   - `ui.pattern.join`: ` + `（两种语言相同，也放进词条，留给将来别的语言调整）
2. **名称表改成函数。** `BUTTON_NAMES`、`EDGE_NAMES`、`VK_NAMES` 里描述性的那几项、`ACTION_NAMES`，统统改成按需调用 `t()` 的函数，比如 `buttonName(b)`。模块加载时就求值的常量表不会随语言切换更新。
3. **排序。** `localeCompare(..., "zh-CN")` 改成用当前语言（`Sidebar.tsx`）。
4. **数字和单位。** `500 ms`、`30 px`、`19%` 保持现有写法，不走 `Intl.NumberFormat`；单位符号两种语言一样。
5. **键帽。** `keyseqToCaps` 只翻译“描述性”的键名（空格、滚轮、鼠标键、等待 N ms），`Ctrl`、`Enter`、`F5` 等保持原样。
6. **后端错误。** 所有返回 `Err(String)` 的命令改为返回下面的结构，前端统一用 `t(e.key, e.params)` 显示。`api.ts` 里的 `errText` 跟着改：先识别这个结构，再兼容旧的字符串。

   ```rust
   #[derive(Serialize)]
   pub struct Msg { pub key: &'static str, pub params: BTreeMap<&'static str, String> }
   ```

   `validate_keys` 同理：返回 `Option<Msg>`，词条为 `keyseq.error.near: "第 {pos} 个字符附近：{reason}"`，其中 `reason` 本身也是一个词条键，由前端翻译。
7. **布局要给英文留空间。** 英文普遍比中文长 30%～80%。重点检查这几处：
   - `.segmented` 分段控件（触发方式那排五个按钮、`picker-mode`、目标模式）：允许换行，或在窄宽度下改成纵向；
   - 分类标签 `.tabs` 与新建按钮同在一行：标签区允许收缩；
   - 编辑面板宽度固定在 400–460px：字段标签和帮助文字允许换行；
   - 动作类型 4×2 网格 `.action-types`：按钮文字允许两行；
   - 托盘菜单和覆盖层标签是原生控件，会按文字自适应宽度，不用处理。
8. **字体栈** 改为按语言选择：

   ```css
   :root { --font: "Segoe UI Variable Text", "Segoe UI", "Microsoft YaHei UI", system-ui, sans-serif; }
   :root:lang(zh-CN) { --font: "Segoe UI Variable Text", "Microsoft YaHei UI", "Segoe UI", system-ui, sans-serif; }
   ```

   中文界面让雅黑优先，避免中英混排时英文用 Segoe、中文回落到宋体。

---

## 5. 内置名称的数据模型

### 5.1 问题

`core/names.rs` 的 `localize()` 在导入和加载配置时，把旧格式的英文默认名**直接改写成中文**存进配置文件。这样带来两个问题：
- 英文用户也会看到中文名；
- 一旦改写，就丢失了“这是一个默认名”的信息，以后无法再切换语言。

### 5.2 方案：`name_key`

```rust
pub struct Binding {
    pub name: String,
    /// 内置名称的词条键（如 "builtin.gesture.close_tab"）。存在时显示 t(name_key)；
    /// 用户改名即清空。`name` 始终保留一份当时语言的文字，作为不认识该键时的回落。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name_key: Option<String>,
    …
}
// Scope 同样加 name_key（"builtin.scope.global" / "builtin.scope.explorer" …）
```

- **显示名**：`display_name(b) = b.name_key.and_then(|k| try_t(k)).unwrap_or(&b.name)`。
  - Rust 侧（引擎在屏幕上显示的手势名）和 TS 侧（列表、编辑面板）各实现一份同样逻辑的函数。
  - `try_t` 遇到不认识的键返回 None，就回落到 `name`。
- **改名**：编辑面板改名时，同时清掉 `name_key`。
  - 改名输入框的初始值是 `display_name`，不是 `name`。否则一个英文用户打开编辑面板，会看到中文。
- **导入**：`legacy.rs` 识别到默认英文名时，设置 `name_key`，`name` 填当前语言的译文。
- **迁移旧配置**：`names.rs` 的对照表从“英文 → 中文”扩展为“英文名或已经改写过的中文名 → 键”。
  - 加载配置时，如果 `name` 恰好等于表里的英文名或中文名，而且没有 `name_key`，就补上 `name_key`。
  - 这一步是幂等的，并保持 `name` 原样不动。
  - 同名 `New` 按动作区分成三个键的逻辑保留。
  - `localize()` 改名为 `attach_name_keys()`。
- `names.rs` 里的中文文字搬进 `i18n/zh-CN.json` 的 `builtin.gesture.*`，表里只留对照关系。

### 5.3 内置建议禁用清单

`presets.rs` 的元组从 `(exe, 显示名, 类别)` 改成 `(exe, 类别枚举)`：
- 显示名的键是 `preset.app.<exe 小写去掉 .exe>`；
- 类别的键是 `preset.category.<remote|design|gamedev>`。

产品名（ToDesk、Blender、AutoCAD…）在两种语言里通常都一样，但仍然进词条，因为“远程桌面连接”这种是系统组件的本地化名称。

添加程序时传入的中文显示名（`addApps(..., names)`）改为写入 `name_key = preset.app.*`，`name` 填当前语言的译文。

---

## 6. 实施步骤

每一步结束时都要满足：`cargo test --workspace` 通过、`tsc` 通过、UI 冒烟测试通过，并且每一步都能单独提交。

| 步骤 | 内容 | 涉及文件 | 完成标准 |
|---|---|---|---|
| **P0 基础设施** | `i18n/` 目录与两个 JSON 文件（先只放 `common.*`）；`core::i18n`（`t`/`tf`/`t!`、`resolve`、`ui_font`）；`build.rs` 生成键名常量；`Settings.language`；`mousetroke-win` 取系统语言；TS 端 `i18n/index.ts`、`useLang`、`<T>`；lint 脚本（第 7.2 节）先只报告不拦截 | 新建 `i18n/*`、`core/src/i18n.rs`、`core/build.rs`；改 `schema.rs`、`ui/tsconfig.json` | `resolve` 单测覆盖 auto / 显式指定 / 不认识的值 / zh-TW 映射；词条一致性测试（第 7.1 节）通过 |
| **P1 设置界面** | 按组件逐个抽取文字。`gesture.ts` 的名称表改成函数、`describePattern` 改用模板；`data-tip`、`aria-label`、placeholder 一并处理；语言下拉框 | `ui/src/**` | lint 对 `ui/src` 零报告；切到 en-US 后整个界面没有中文（冒烟测试断言） |
| **P2 后端错误结构化** | `Msg` 结构体；`validate_keys`、`resolve_app`、`open_path`、`start_engine`、`set_autostart`、导入相关命令全部改造；`keyseq` 的错误改成 `reason` 键 | `settings/src/lib.rs`、`picker.rs`、`core/src/keyseq.rs`、`ui/src/api.ts` | 界面里切换语言后，错误提示也随之切换 |
| **P3 引擎、托盘、命令行** | 托盘菜单、托盘提示、托盘气泡；覆盖层的「未识别」、各种 toast；亮度提示；`diag` 与帮助；按语言选择覆盖层字体 | `win/src/tray.rs`、`engine.rs`、`input.rs`、`overlay.rs`、`picker.rs`、`mousetroke/src/main.rs` | 引擎测试分别断言 zh 与 en 下的「未识别」文字；改配置里的语言后 1 秒内托盘菜单变成新语言（手动验证，写进交接文档） |
| **P4 内置名称** | `name_key`（Binding、Scope）；导入时设置；加载时补上；改名时清掉；`display_name` 两端实现；内置建议清单改造 | `core/src/schema.rs`、`legacy.rs`、`names.rs`、`presets.rs`、`ui/src/components/Editor.tsx`、`BindingList.tsx`、`Sidebar.tsx`、`App.tsx` | 导入测试：en-US 下默认名显示英文，zh-CN 下显示中文；用户改过的名字不受语言影响；旧配置（已改写成中文名）加载后能补上 `name_key` |
| **P5 英文翻译与验收** | 补全 `en-US.json`（按第 8 节的术语表）；伪语言；三种语言的冒烟测试与截图；lint 改为拦截 | `i18n/en-US.json`、`tools/uitest/*` | 伪语言截图里没有溢出或截断；lint 在 `cargo test` 中执行，有违规就失败 |

---

## 7. 质量保障

### 7.1 词条一致性测试（`mousetroke-core/tests/i18n_catalog.rs`）

对每个语言文件检查：
- 是合法 JSON，并且是扁平的“字符串到字符串”映射；
- 键集合与 zh-CN 完全相同，复数键按“基础键”合并后再比较；
- 同一个键在各语言里的**占位符集合相同**，比如 zh 用了 `{count}`，en 也必须用；
- 没有空字符串；
- 只有以 `_one` / `_other` 结尾的键可以出现在复数词条里。

### 7.2 禁止写死中文：`scripts/i18n_lint.py`（只用标准库）

- 扫描 `crates/**/src/**/*.rs` 与 `ui/src/**/*.{ts,tsx}` 中的**字符串字面量**和 **JSX 文本**，出现 CJK 字符（`一-鿿`、`　-〿`、`＀-￯`）就报告。
- 注释不算。
- 白名单：
  - `mousetroke-win/src/log.rs` 的调用方：凡是 `log!(…)` 宏的参数都放行，日志不翻译；
  - 测试文件：`*_tests.rs`、`tests/`、`#[cfg(test)]` 模块；
  - `names.rs` 里迁移用的中文对照；
  - 词条文件本身。
- 用 `cargo test` 里的一个测试调用这个脚本，P5 起有违规就让测试失败。前端在 `npm run build` 之前也跑一次。

### 7.3 伪语言 `qps-ploc`（仅调试构建）

- 由 zh-CN 或 en-US 在运行时自动生成，不提交文件。
  - 规则：每个字母换成带重音的同形字（a→á、e→é…）；首尾加 `[!! … !!]`；按长度补 40% 的填充字符；占位符原样保留。
- 用途：
  - 一眼看出哪里漏翻（没有方括号的就是写死的文字）；
  - 检查长文本会不会溢出或被截断。
- 开启方式：设置值 `language: "qps-ploc"` 只在 `cfg(debug_assertions)` 下才认；界面也可以用 URL 参数 `?lang=qps-ploc` 开启，方便冒烟测试使用。

### 7.4 UI 冒烟测试

- `tools/uitest/ui.mjs` 增加 `--lang <tag>` 参数，模拟后端在 `init_state` 里返回对应的 `resolvedLang`。
- zh-CN、en-US、qps-ploc 各跑一遍，截图分别保存到 `uishots/<lang>/`。
- en-US 运行时追加一条断言：`document.body.innerText` 里不含 CJK 字符。
  - 例外：用户数据里的中文名可以出现，测试用的配置文件也要准备一份英文版。

### 7.5 引擎测试

- `engine_tests.rs` 加一个测试：`set_lang(EnUs)` 后，未识别手势时的标签是 `Unrecognized  → ↓ →`。
- 再加一个测试：带 `name_key` 的绑定命中时，标签显示的是译文。

---

## 8. 翻译规范（同时写进 `i18n/README.md`）

### 8.1 术语表

| 中文 | English | 说明 |
|---|---|---|
| 手势 / 鼠标手势 | gesture / mouse gesture | |
| 画 | draw | “画 ↓→” → “draw ↓→” |
| 触发角 | Screen edges | 标签页名；单指某个角时用 corner：“左下角” → “bottom-left corner” |
| 按住右键 | Hold right button | |
| 所有程序 | All apps | 全局作用域 |
| 为某个程序单独设置 | App-specific gestures | |
| 停用（单个手势） | Turn off | 手势级的开关：“已停用” → “Off” |
| 禁用手势（在某个程序里） | Turn off gestures in this app | 程序级。**和上一条是两个不同的概念，不能共用词条** |
| 未识别 | Unrecognized | |
| 识别即执行 | Run as soon as recognized | |
| 暂停 / 恢复 | Pause / Resume | |
| 托盘 / 通知区域 | system tray / notification area | |
| 撤销 / 重做 | Undo / Redo | |
| 导入旧格式的手势配置 | Import legacy gesture settings | |
| 点选窗口 | Pick a window | |

### 8.2 语气与格式

- 中文：沿用现有文案风格。动作用动词开头，比如“导入并替换当前手势”；错误直接说明怎么解决，不用“抱歉”。
- 英文：
  - 按钮和标题用 sentence case（“Import and replace”），不用 Title Case；
  - 不缩写成 “pls” 一类口语写法；
  - 撇号用 `’`。
- 中文全角标点「」、…、：只出现在 zh 词条里。en 用 `“ ”`、`…`、`:`。
- 占位符两侧的空格由各语言自己决定：中文“已在 {count} 个程序里”，英文“in {count} apps”。

### 8.3 不要翻译的

- 键名：Ctrl、Alt、Shift、Win、Esc、Tab、Enter、F1–F24；
- 文件名：`gestures.json`、`prefs.json`、`config.json`；
- 产品名：Mousetroke 以及第三方软件名；
- Lua 函数名。

---

## 9. 已知取舍与风险

1. **共享词条，体积会略增**：引擎会嵌入整份词条，包括它用不到的 `ui.*` 部分，两种语言合计大约 60–90KB。
   - 引擎的常驻内存目标（工作集小于 40MB，现状 19MB）不受影响，因为解析只在首次取词条时进行。
   - 如果在意这部分体积，可以在 `build.rs` 里按前缀给引擎生成一份只含 `tray./overlay./cli./edge./button./keyseq./builtin.` 的子集。一期不做。
2. **`name_key` 让配置格式多了一个字段**：旧版本程序读新配置时会忽略它，因为 serde 默认忽略未知字段，所以不需要提升配置版本号。新版本读旧配置时，由第 5.2 节的迁移补上。
3. **语言切换不需要重启**：界面即时切换，引擎通过热加载切换。屏幕上正在显示的那一个标签不会立刻刷新，下一次显示时生效，这是可以接受的。
4. **WebView2 自带的界面**（右键菜单、拼写检查）跟随系统语言，不跟随我们的设置。现状下这些已基本被禁用，不处理。

---

## 10. 交付物清单

- [ ] `i18n/zh-CN.json`、`i18n/en-US.json`、`i18n/README.md`
- [ ] `crates/mousetroke-core/src/i18n.rs`、`crates/mousetroke-core/build.rs`、`crates/mousetroke-core/tests/i18n_catalog.rs`
- [ ] `Settings.language`；`Binding.name_key`、`Scope.name_key` 及迁移逻辑
- [ ] `crates/mousetroke-settings/ui/src/i18n/index.ts`（含 `t`、`useLang`、`<T>`、伪语言生成）
- [ ] 偏好设置里的「语言」下拉框；`set_ui_lang` 命令；`init_state.systemLangs` 与 `resolvedLang`
- [ ] 后端错误改为 `Msg { key, params }`
- [ ] `scripts/i18n_lint.py`，并接入 `cargo test`
- [ ] 冒烟测试支持 `--lang` 参数，三种语言的截图
- [x] i18n 维护说明：见 `i18n/README.md`（怎么加新文案、怎么加新语言、怎么跑伪语言测试）
