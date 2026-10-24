//! Mousetroke 设置程序。
//!
//! 独立进程、按需打开、关掉即退（常驻引擎不含 WebView，保持低占用）。
//! 只读写原生 `config.json`（原子写入），引擎轮询 mtime 热加载——两个进程之间
//! Windows 更新通过托盘消息交接，macOS 通过 PID 退出请求交接；关于页请求由平台层传递。
//! 配置语义复用 mousetroke-core。
//!
//! 以库的形式编进 `mousetroke`（`mousetroke settings [配置路径]` 启动本界面进程）：
//! 只发布一个主程序，但界面仍是独立进程——常驻的引擎进程里没有 WebView。

mod platform;
#[cfg(target_os = "macos")]
mod mac_permissions;
use platform::{engine_exe, engine_status, icons, picker};

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use mousetroke_core::i18n::{keys, t};
use mousetroke_core::schema::{Action, Config, Step};
use mousetroke_core::presets::actions::{self as action_presets, Params};
use mousetroke_core::shape;
use mousetroke_core::caps;
use serde::Serialize;
use tauri::{Emitter, Manager, State, WebviewWindow};

/// 启动参数或其他实例请求打开关于页；界面初始化时取一次，之后改用 show-about 事件。
static ABOUT: AtomicBool = AtomicBool::new(false);
#[cfg(target_os = "macos")]
static SETUP: AtomicBool = AtomicBool::new(false);

/// 打开关于页并把窗口带到前台（第二次启动、Mac 应用菜单共用）。
pub(crate) fn show_window(app: &tauri::AppHandle, about: bool) {
    if about {
        ABOUT.store(true, Ordering::Release);
        let _ = app.emit("show-about", ());
    } else {
        #[cfg(target_os = "macos")]
        let _ = app.emit("show-settings", ());
    }
    focus_window(app);
}

fn focus_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        platform::bring_to_front();
    }
}

/// 后端返回给界面的错误：词条键 + 参数，由前端 `t(key, params)` 翻译。
/// 切换语言不需要后端参与（docs/I18N-DESIGN.md §3.4）。
#[derive(Serialize, Clone, Debug)]
pub struct Msg {
    pub key: &'static str,
    pub params: BTreeMap<&'static str, String>,
}

impl Msg {
    fn new(key: &'static str, params: &[(&'static str, String)]) -> Self {
        Self { key, params: params.iter().cloned().collect() }
    }
}

type MsgResult<T> = Result<T, Msg>;

/// 当前编辑的配置文件路径（命令行第一个参数，否则默认路径）。
struct AppPaths {
    config: PathBuf,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InitState {
    about: bool,
    #[cfg(target_os = "macos")]
    setup: bool,
    #[cfg(target_os = "macos")]
    permissions: mac_permissions::PermissionStatus,
    platform: &'static str,
    config_path: String,
    /// 文件不存在时为默认配置，`config_exists = false`。
    config: Config,
    config_exists: bool,
    /// 配置文件存在但解析失败时的错误（界面提示并允许重新导入/重置）。
    load_error: Option<String>,
    engine: EngineStatus,
    /// 系统首选界面语言列表（前端在 language = auto 时用，不依赖 WebView 或浏览器语言）。
    system_langs: Vec<String>,
    /// 后端按配置 + 系统语言解析出的语言标签。
    resolved_lang: String,
    action_catalog: ActionCatalog,
}

/// 未知动作的原始 JSON 仍在 config 内；此处给选择器与只读展示提供判定和词条。
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ActionCatalog {
    presets: &'static [action_presets::Preset],
    defaults: BTreeMap<&'static str, Params>,
    known_action_types: &'static [&'static str],
    unknown_action_key: &'static str,
}

#[tauri::command]
fn action_catalog() -> ActionCatalog {
    ActionCatalog {
        presets: action_presets::REGISTRY,
        defaults: action_presets::REGISTRY.iter().map(|p| (p.id, p.default_params(mousetroke_core::i18n::lang()))).collect(),
        known_action_types: Action::KNOWN_TYPES,
        unknown_action_key: "preset.status.unknown",
    }
}

#[tauri::command]
fn expand_preset(id: String, params: Params, ui_lang: String) -> MsgResult<Action> {
    let lang = mousetroke_core::i18n::Lang::from_tag(&ui_lang).unwrap_or_else(mousetroke_core::i18n::lang);
    action_presets::expand(&id, caps::Platform::CURRENT, &params, lang).map_err(|e| Msg::new(e.reason_key(), &[]))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EngineStatus {
    running: bool,
    autostart: bool,
    /// 是否找到可启动的引擎程序（合并版即当前可执行文件）。
    exe_found: bool,
}

#[derive(Serialize)]
struct RunningApp {
    exe: String,
    title: String,
}

fn read_text(path: &Path) -> Result<String, Msg> {
    std::fs::read_to_string(path).map_err(|e| Msg::new(keys::ERROR_READ_FILE, &[("path", path.display().to_string()), ("err", e.to_string())]))
}

// ---------------------------------------------------------------- 命令

#[tauri::command]
fn init_state(paths: State<AppPaths>) -> InitState {
    use mousetroke_core::i18n;
    let path = &paths.config;
    let (config, exists, load_error) = config_snapshot(path);
    let system_langs = platform::system_langs();
    let resolved = i18n::resolve(&config.settings.language, &system_langs);
    // 后端自己需要的少量文字（窗口标题、点选提示条）也用这个语言
    i18n::set_lang(resolved);
    InitState {
        about: take_about_request(),
        #[cfg(target_os = "macos")]
        setup: take_setup_request(),
        #[cfg(target_os = "macos")]
        permissions: mac_permissions::get_permissions(),
        platform: if cfg!(target_os = "macos") { "macos" } else { "windows" },
        config_path: path.to_string_lossy().into_owned(),
        config,
        config_exists: exists,
        load_error,
        engine: engine_status(),
        system_langs,
        resolved_lang: resolved.tag().into(),
        action_catalog: action_catalog(),
    }
}

/// 不创建原生窗口的配置快照，也用于 CI 冒烟检查实际命令的读写路径。
fn config_snapshot(path: &Path) -> (Config, bool, Option<String>) {
    let exists = path.exists();
    let (config, load_error) = if exists {
        match Config::load(path) {
            Ok(c) => (c, None),
            Err(e) => {
                eprintln!("{e}");
                (Config::builtin(), Some(e))
            }
        }
    } else {
        // 首次运行（引擎还没生成配置文件）：展示内置默认手势，保存时写盘
        (Config::builtin(), None)
    };
    (config, exists, load_error)
}

/// 界面切换语言后调用：后端 set_lang 并更新窗口标题。
#[tauri::command]
fn set_ui_lang(tag: String, window: WebviewWindow) {
    use mousetroke_core::i18n;
    if let Some(l) = i18n::Lang::from_tag(&tag) {
        i18n::set_lang(l);
    }
    let _ = window.set_title(i18n::t(i18n::keys::UI_WINDOW_TITLE));
    platform::refresh_texts(&window);
}

#[tauri::command]
fn save_config(paths: State<AppPaths>, config: Config) -> MsgResult<()> {
    save_config_at(&paths.config, &config)
}

fn save_config_at(path: &Path, config: &Config) -> MsgResult<()> {
    validate_strokes(config)?;
    config.save(path).map_err(|e| Msg::new(keys::ERROR_SAVE, &[("path", path.display().to_string()), ("err", e)]))
}

#[tauri::command]
fn get_engine_status() -> EngineStatus {
    engine_status()
}

/// 校验按键序列语法；返回错误词条（含位置），合法返回 None。
/// `reason` 参数本身也是一个词条键，由前端翻译。
#[tauri::command]
fn validate_keys(keys: String) -> Option<Msg> {
    let parsed = mousetroke_core::keyseq::parse(&keys, false);
    if parsed.as_ref().is_ok_and(|seq| !caps::keys_supported(seq, caps::Platform::CURRENT)) {
        return Some(Msg::new(keys::MAC_SETTINGS_KEYS_UNSUPPORTED, &[]));
    }
    parsed.err().map(|e| {
        let mut params: Vec<(&'static str, String)> = vec![("pos", (e.pos + 1).to_string()), ("reason", e.key.to_string())];
        params.extend(e.params);
        Msg::new(mousetroke_core::i18n::keys::KEYSEQ_ERROR_NEAR, &params)
    })
}

/// 录制器：画布上的原始点 → trace 点编码。`grid` 为画布上的一格像素数。
#[tauri::command]
fn record_trace(points: Vec<(f64, f64)>, grid: f64) -> Vec<i32> {
    mousetroke_core::recognizer::record_points(&points, grid)
}

/// 自动建议录制类型；实际的样本或轨迹编码仍由对应命令生成。
#[tauri::command]
fn classify_stroke(points: Vec<(f64, f64)>) -> &'static str {
    match shape::classify_stroke(&points) {
        shape::StrokeKind::Polyline => "trace",
        shape::StrokeKind::Shape => "shape",
    }
}

/// 形状录制与比较始终复用核心算法，前端只负责采点与展示。
#[tauri::command]
fn shape_sample(points: Vec<(f64, f64)>) -> MsgResult<Vec<i32>> {
    shape::normalize(&points).map(|p| shape::encode(&p)).ok_or_else(|| Msg::new(keys::UI_SHAPE_TOO_SHORT, &[]))
}

#[tauri::command]
fn shape_compare(a: Vec<i32>, b: Vec<i32>) -> f64 {
    match (shape::decode(&a), shape::decode(&b)) {
        (Some(a), Some(b)) => shape::similarity(&a, &b),
        _ => 0.0,
    }
}

#[derive(Serialize)]
struct ShapeLimits {
    accept: f64,
    margin: f64,
}

/// 样本作为绘制输入时的核心阈值；混合曲线/折线的提示须逐样本查询。
#[tauri::command]
fn shape_limits(sample: Vec<i32>) -> ShapeLimits {
    let accept = shape::decode(&sample).map_or(shape::ACCEPT, |path| shape::acceptance_threshold(&path));
    ShapeLimits { accept, margin: shape::MARGIN }
}

/// 空笔画是尚未录完的编辑状态，不能覆盖磁盘上的有效配置。
fn validate_strokes(config: &Config) -> MsgResult<()> {
    for binding in config.scopes.iter().flat_map(|s| &s.bindings) {
        for step in &binding.pattern {
            if matches!(step, Step::Trace { points } if points.len() < 4) {
                return Err(Msg::new(keys::UI_RECORDER_TOO_SHORT, &[]));
            }
            if let Step::Shape { templates } = step {
                let valid = templates.iter().any(|s| shape::decode(s).is_some_and(|p| shape::similarity(&p, &p) > 0.0));
                if !valid {
                    return Err(Msg::new(keys::UI_SHAPE_NEEDS_SAMPLE, &[]));
                }
            }
        }
    }
    Ok(())
}

#[tauri::command]
fn import_legacy(gestures: String, prefs: Option<String>) -> MsgResult<Config> {
    let g = read_text(Path::new(&gestures))?;
    // prefs.json 可选：不存在时只导入手势
    let p = match prefs.filter(|p| !p.is_empty() && Path::new(p).exists()) {
        Some(p) => Some(read_text(Path::new(&p))?),
        None => None,
    };
    mousetroke_core::legacy::import(&g, p.as_deref())
        .map_err(|e| Msg::new(keys::ERROR_PARSE, &[("err", e)]))
}

#[tauri::command]
fn load_config_file(path: String) -> MsgResult<Config> {
    Config::load(Path::new(&path))
        .map_err(|e| Msg::new(keys::ERROR_PARSE, &[("err", e)]))
}

#[tauri::command]
fn export_config(path: String, config: Config) -> MsgResult<()> {
    save_config_at(Path::new(&path), &config)
}

/// 文件选择。`kind`：json | exe | any；`save` 为 true 时是保存对话框。
#[tauri::command]
async fn pick_file(window: WebviewWindow, kind: String, save: bool, title: String) -> Option<String> {
    let mut d = rfd::FileDialog::new().set_title(&title).set_parent(&window);
    d = match kind.as_str() {
        "json" => d.add_filter(t(keys::UI_PICKER_FILTER_JSON), &["json"]),
        "exe" => platform::program_dialog(d),
        _ => d,
    };
    let picked = if save { d.set_file_name("mousetroke-config.json").save_file() } else { d.pick_file() };
    picked.and_then(|p| platform::picked_file(p.to_string_lossy().into_owned(), &kind))
}

#[tauri::command]
fn running_apps() -> Vec<RunningApp> {
    let mut apps = platform::running_apps();
    apps.sort_by_key(|a| mousetroke_core::matcher::file_name(&a.exe).to_lowercase());
    apps
}

/// 一批程序的图标（data URL；取不到为 null）。在线程池里跑，不卡界面。
#[cfg_attr(windows, tauri::command(async))]
#[cfg_attr(target_os = "macos", tauri::command)]
fn app_icons(exes: Vec<String>) -> std::collections::HashMap<String, Option<String>> {
    exes.into_iter().map(|e| {
        let icon = icons::icon_data_url(&e);
        (e, icon)
    }).collect()
}

/// 字段保持 snake_case（与 types.ts 的 PresetApp、作用域的 name_key 一致）。
#[derive(Serialize)]
struct PresetApp {
    exe: String,
    /// 后端当前语言的显示名（回落用；界面按 name_key 自己翻译）。
    name: String,
    /// 显示名词条键（preset.app.*）：添加程序时写入作用域的 name_key。
    name_key: String,
    /// 后端当前语言的类别名（回落用）。
    category: String,
    /// 类别词条键（preset.category.*）。
    category_key: String,
    /// 在这台电脑上找到的完整路径（没找到为 null）。
    path: Option<String>,
}

/// 内置的"建议禁用手势"程序，并标出本机已安装的。
#[cfg_attr(windows, tauri::command(async))]
#[cfg_attr(target_os = "macos", tauri::command)]
fn preset_apps() -> Vec<PresetApp> {
    use mousetroke_core::i18n::t;
    mousetroke_core::presets::disable_suggestions()
        .iter()
        .map(|(exe, name_key, category)| PresetApp {
            exe: exe.to_string(),
            name: t(name_key).to_string(),
            name_key: name_key.to_string(),
            category: t(category.key()).to_string(),
            category_key: category.key().to_string(),
            path: icons::resolve_exe(exe).map(|p| p.to_string_lossy().into_owned()),
        })
        .collect()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppInfo {
    version: String,
    exe_path: String,
    config_path: String,
    log_path: String,
}

/// 关于页：版本与文件位置。
#[tauri::command]
fn app_info(paths: State<AppPaths>) -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").into(),
        exe_path: engine_exe().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default(),
        config_path: paths.config.to_string_lossy().into_owned(),
        log_path: platform::log_path().to_string_lossy().into_owned(),
    }
}

const WEBSITE_ORIGIN: &str = "https://mousetroke.com";
const HOMEPAGE_PATHS: [(&str, &str); 10] = [
    ("zh-CN", "/"),
    ("en-US", "/en/"),
    ("es-ES", "/es/"),
    ("fr-FR", "/fr/"),
    ("de-DE", "/de/"),
    ("ja-JP", "/ja/"),
    ("pt-BR", "/pt/"),
    ("ru-RU", "/ru/"),
    ("it-IT", "/it/"),
    ("ko-KR", "/ko/"),
];
const GITHUB_URL: &str = "https://github.com/eiaao/mousetroke";
const ISSUES_URL: &str = "https://github.com/eiaao/mousetroke/issues";

/// 链接仅由后端白名单决定；没有对应首页的语言回退到英文。
fn project_url(what: &str, lang: &str) -> Option<String> {
    let path = match what {
        "homepage" => HOMEPAGE_PATHS.iter().find(|(tag, _)| *tag == lang).unwrap_or(&HOMEPAGE_PATHS[1]).1,
        "github" => return Some(GITHUB_URL.into()),
        "issues" => return Some(ISSUES_URL.into()),
        "releases" => return Some(platform::update::RELEASES_URL.into()),
        "lua_basics" => if lang == "zh-CN" {
            "/docs/lua-basics/"
        } else {
            "/en/docs/lua-basics/"
        },
        "lua_api" => if lang == "zh-CN" {
            "/docs/lua-api/"
        } else {
            "/en/docs/lua-api/"
        },
        "lua_examples" => if lang == "zh-CN" {
            "/docs/lua-examples/"
        } else {
            "/en/docs/lua-examples/"
        },
        "variables" => if lang == "zh-CN" {
            "/docs/actions/#variables"
        } else {
            "/en/docs/actions/#variables"
        },
        "key_sequences" => if lang == "zh-CN" {
            "/docs/key-sequences/"
        } else {
            "/en/docs/key-sequences/"
        },
        _ => return None,
    };
    Some(format!("{WEBSITE_ORIGIN}{path}"))
}

/// 用默认程序打开日志、配置文件夹或白名单中的项目链接。
#[tauri::command]
fn open_path(what: String, paths: State<AppPaths>) -> MsgResult<()> {
    match what.as_str() {
        "log" => {
            let log = platform::log_path();
            if !log.exists() {
                return Err(Msg::new(keys::ERROR_NO_LOG_YET, &[]));
            }
            platform::open_file(&log)?;
        }
        "config_dir" => {
            let dir = paths.config.parent().map(Path::to_path_buf).unwrap_or_default();
            platform::open_folder(&dir)?;
        }
        _ => {
            let Some(url) = project_url(&what, mousetroke_core::i18n::lang().tag()) else {
                return Err(Msg::new(keys::ERROR_UNKNOWN_LOCATION, &[("what", what)]));
            };
            platform::open_url(&url)?;
        }
    }
    Ok(())
}

/// 拖入的文件 → 平台应用身份（Windows 路径 / macOS bundle id 或可执行文件路径）。
#[tauri::command]
fn resolve_app(path: String) -> MsgResult<String> {
    picker::resolve_app(&path)
}

/// 在屏幕上点选一个窗口，平台层负责让开并还原设置窗口。
#[tauri::command(async)]
fn pick_window(window: WebviewWindow) -> MsgResult<Option<picker::PickedApp>> {
    platform::pick_window(window)
}

#[tauri::command]
fn set_autostart(on: bool) -> MsgResult<EngineStatus> {
    let exe = engine_exe().ok_or_else(|| Msg::new(keys::ERROR_ENGINE_NOT_FOUND, &[]))?;
    platform::set_autostart(on, &exe)
        .map_err(|e| Msg::new(keys::ERROR_AUTOSTART, &[("err", e)]))?;
    Ok(engine_status())
}

#[tauri::command]
fn start_engine(paths: State<AppPaths>) -> MsgResult<EngineStatus> {
    let exe = engine_exe().ok_or_else(|| Msg::new(keys::ERROR_ENGINE_NOT_FOUND, &[]))?;
    platform::start_engine(&exe, &paths.config)
        .map_err(|e| Msg::new(keys::ERROR_START_ENGINE, &[("err", e.to_string())]))?;
    Ok(engine_status())
}

/// 在系统文件管理器中显示配置文件。
#[tauri::command]
fn reveal_config(paths: State<AppPaths>) {
    platform::reveal_config(&paths.config);
}

// ---------------------------------------------------------------- 单实例

/// 消费平台传来的关于页请求。
#[tauri::command]
fn take_about_request() -> bool { ABOUT.swap(false, Ordering::AcqRel) }

#[cfg(target_os = "macos")]
#[tauri::command]
fn take_setup_request() -> bool { SETUP.swap(false, Ordering::AcqRel) }

#[cfg(target_os = "macos")]
fn show_setup(app: &tauri::AppHandle) {
    SETUP.store(true, Ordering::Release);
    let _ = app.emit("show-setup", ());
    focus_window(app);
}

#[tauri::command]
async fn check_update() -> MsgResult<platform::update::Check> {
    tauri::async_runtime::spawn_blocking(platform::update::check).await
        .map_err(|_| Msg::new(keys::UPDATE_ERROR_NETWORK, &[]))?
        .map_err(|e| Msg::new(e.key(), &[]))
}

/// macOS 的手动下载后备入口；安装入口仍在原生侧重新解析发布信息。
#[tauri::command]
async fn download_update() -> MsgResult<()> {
    #[cfg(target_os = "macos")]
    {
        let url = tauri::async_runtime::spawn_blocking(platform::update::download).await
            .map_err(|_| Msg::new(keys::UPDATE_ERROR_NETWORK, &[]))?
            .map_err(|e| Msg::new(e.key(), &[]))?;
        platform::open_url(&url)
    }
    #[cfg(not(target_os = "macos"))]
    Err(Msg::new(keys::UPDATE_ERROR_NETWORK, &[]))
}

/// 两个平台均在阻塞工作线程完成验证和替换，再退出设置进程交给助手重启。
#[tauri::command]
async fn install_update(paths: State<'_, AppPaths>, app: tauri::AppHandle) -> MsgResult<()> {
    #[cfg(windows)]
    {
        let exe = engine_exe().ok_or_else(|| Msg::new(keys::ERROR_ENGINE_NOT_FOUND, &[]))?;
        let config = paths.config.clone();
        tauri::async_runtime::spawn_blocking(move || platform::update::install(&exe, &config)).await
            .map_err(|_| Msg::new(keys::UPDATE_ERROR_REPLACE, &[]))?
            .map_err(|e| Msg::new(e.key(), &[]))?;
        // 新进程等待本 PID 退出；引擎也通过托盘同一退出路径停止。
        app.exit(0);
        Ok(())
    }
    #[cfg(target_os = "macos")]
    {
        let config = paths.config.clone();
        tauri::async_runtime::spawn_blocking(move || platform::update::install(&config)).await
            .map_err(|_| Msg::new(keys::UPDATE_ERROR_REPLACE, &[]))?
            .map_err(|e| Msg::new(e.key(), &[]))?;
        app.exit(0);
        Ok(())
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let _ = (paths, app);
        Err(Msg::new(keys::UPDATE_ERROR_REPLACE, &[]))
    }
}

/// 运行设置界面（阻塞到窗口关闭）。`config` 缺省为默认配置路径。已有设置窗口时把它带到前台后返回。
pub fn run(config: Option<PathBuf>) {
    // 启动时按配置确定语言：窗口标题等少量原生文字依赖它
    let config = config.unwrap_or_else(platform::default_config_path);
    mousetroke_core::i18n::init_from_config(&config, &platform::system_langs());
    ABOUT.store(std::env::args().any(|s| s == "--about"), Ordering::Release);
    #[cfg(target_os = "macos")]
    SETUP.store(std::env::args().any(|s| s == "--setup"), Ordering::Release);
    tauri::Builder::default()
        // 必须最先注册：已有实例时把参数转交过去并立即退出，不再创建窗口
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            #[cfg(target_os = "macos")]
            if argv.iter().any(|a| a == "--setup") {
                show_setup(app);
                return;
            }
            show_window(app, argv.iter().any(|a| a == "--about"));
        }))
        .manage(AppPaths { config })
        .invoke_handler(tauri::generate_handler![
            init_state,
            action_catalog,
            expand_preset,
            save_config,
            get_engine_status,
            set_ui_lang,
            validate_keys,
            record_trace,
            classify_stroke,
            shape_sample,
            shape_compare,
            shape_limits,
            import_legacy,
            load_config_file,
            export_config,
            pick_file,
            running_apps,
            app_icons,
            resolve_app,
            preset_apps,
            app_info,
            check_update,
            install_update,
            download_update,
            take_about_request,
            #[cfg(target_os = "macos")]
            take_setup_request,
            #[cfg(target_os = "macos")]
            mac_permissions::get_permissions,
            #[cfg(target_os = "macos")]
            mac_permissions::request_permission,
            #[cfg(target_os = "macos")]
            mac_permissions::restart_engine,
            open_path,
            pick_window,
            set_autostart,
            start_engine,
            reveal_config,
        ])
        .setup(|app| {
            platform::setup(app);
            // 窗口标题在显示前就按当前语言设好（tauri.conf.json 里是中文占位，
            // 前端 init 后的 set_ui_lang 才来不及闪）；兜底：1.5s 后仍未显示则强制显示，避免"打不开"
            let w = app.get_webview_window("main").expect("main 窗口"); // i18n:allow 内部断言
            let _ = w.set_title(t(keys::UI_WINDOW_TITLE));
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(1500));
                let _ = w.show();
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("设置程序启动失败"); // i18n:allow 内部断言
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_never_saves_fallback_defaults_over_an_unparseable_file() {
        let dir = std::env::temp_dir().join(format!("mousetroke-settings-protection-{}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("config.json");
        let original = "{ invalid JSON";
        std::fs::write(&path, original).unwrap();
        let (config, exists, error) = config_snapshot(&path);
        assert!(exists && error.is_some());
        assert!(config.all_bindings().count() >= 40);
        assert!(save_config_at(&path, &config).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
        std::fs::remove_file(&path).unwrap();
        std::fs::remove_dir(&dir).unwrap();
    }

    #[test]
    fn settings_catalog_exposes_future_action_presentation_and_expansion() {
        let catalog = serde_json::to_value(action_catalog()).unwrap();
        assert_eq!(catalog["presets"].as_array().unwrap().len(), 87);
        assert_eq!(catalog["unknownActionKey"], "preset.status.unknown");
        assert!(catalog["knownActionTypes"].as_array().unwrap().contains(&serde_json::json!("preset")));
        assert!(expand_preset("edit.copy".into(), Params::new(), "en-US".into()).is_ok());
        assert_eq!(expand_preset("future.preset".into(), Params::new(), "en-US".into()).unwrap_err().key, "preset.status.unknown");
        assert_eq!(expand_preset("window.snap_left".into(), Params::new(), "en-US".into()).unwrap_err().key, "preset.status.pending");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn mac_key_validation_rejects_items_the_engine_cannot_execute() {
        for keys in ["{MOUSE_LEFT}", "{F24}", "{MEDIA_STOP}", "{INS}"] {
            assert_eq!(validate_keys(keys.into()).unwrap().key, keys::MAC_SETTINGS_KEYS_UNSUPPORTED);
        }
        for keys in ["#c", "%{LEFT}", "^+{F12}", "#c{SLEEP 500}#v"] {
            assert!(validate_keys(keys.into()).is_none());
        }
    }

    #[test]
    fn settings_config_round_trip() {
        let dir = std::env::temp_dir().join(format!("mousetroke-settings-smoke-{}-{:?}", std::process::id(), std::thread::current().id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config with spaces.json");
        let (mut config, exists, error) = config_snapshot(&path);
        assert!(!exists);
        assert!(error.is_none());
        config.settings.pause_hotkey = "^%#p".into();
        config.settings.language = "ja-JP".into();
        assert!(validate_keys(config.settings.pause_hotkey.clone()).is_none());
        save_config_at(&path, &config).unwrap();
        let (loaded, exists, error) = config_snapshot(&path);
        assert!(exists);
        assert!(error.is_none());
        assert_eq!(serde_json::to_value(&loaded).unwrap(), serde_json::to_value(&config).unwrap());
        let exported = dir.join("export.json");
        export_config(exported.to_string_lossy().into(), loaded).unwrap();
        let imported = load_config_file(exported.to_string_lossy().into()).unwrap();
        assert_eq!(serde_json::to_value(imported).unwrap(), serde_json::to_value(config).unwrap());
        std::fs::write(&path, "invalid json").unwrap();
        assert!(config_snapshot(&path).2.is_some());
        std::fs::remove_file(path).unwrap();
        std::fs::remove_file(exported).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }

    #[test]
    fn project_urls_follow_ui_language_and_only_allow_fixed_destinations() {
        for (tag, suffix) in [
            ("zh-CN", ""), ("en-US", "en/"), ("es-ES", "es/"), ("fr-FR", "fr/"),
            ("de-DE", "de/"), ("ja-JP", "ja/"), ("pt-BR", "pt/"), ("ru-RU", "ru/"),
            ("it-IT", "it/"), ("ko-KR", "ko/"), ("qps-ploc", "en/"), ("unknown", "en/"),
        ] {
            assert_eq!(project_url("homepage", tag).as_deref(), Some(format!("https://mousetroke.com/{suffix}").as_str()));
            assert_eq!(project_url("github", tag).as_deref(), Some("https://github.com/eiaao/mousetroke"));
            assert_eq!(project_url("issues", tag).as_deref(), Some("https://github.com/eiaao/mousetroke/issues"));
            assert_eq!(project_url("releases", tag).as_deref(), Some(platform::update::RELEASES_URL));
            let docs_prefix = if tag == "zh-CN" { "" } else { "en/" };
            for (what, destination) in [
                ("lua_basics", "lua-basics/"), ("lua_api", "lua-api/"),
                ("lua_examples", "lua-examples/"), ("variables", "actions/#variables"),
                ("key_sequences", "key-sequences/"),
            ] {
                assert_eq!(project_url(what, tag).as_deref(), Some(format!("https://mousetroke.com/{docs_prefix}docs/{destination}").as_str()));
            }
        }
        for what in ["", "https://example.com", "https://mousetroke.com/", "file:///C:/Windows", "Homepage", "github/issues", "lua_api/../", "variables#other"] {
            assert_eq!(project_url(what, "en-US"), None);
        }
    }

    #[test]
    fn shape_commands_use_core_and_reject_degenerate_drawings() {
        let raw = vec![(10.0, 20.0), (50.0, 10.0), (70.0, 70.0)];
        let sample = shape_sample(raw.clone()).unwrap();
        assert_eq!(sample, shape::encode(&shape::normalize(&raw).unwrap()));
        assert_eq!(shape_compare(sample.clone(), sample.clone()), 1.0);
        assert_eq!(shape_compare(vec![], sample.clone()), 0.0);
        assert_eq!(shape_compare(vec![0; shape::SAMPLE_POINTS * 2], sample), 0.0);
        assert_eq!(classify_stroke(vec![(0.0, 0.0), (100.0, 0.0)]), "trace");
        let circle: Vec<_> = (0..=64).map(|i| {
            let angle = std::f64::consts::TAU * i as f64 / 64.0;
            (angle.cos() * 100.0, angle.sin() * 100.0)
        }).collect();
        let circle_sample = shape_sample(circle.clone()).unwrap();
        assert_eq!(classify_stroke(circle), "shape");
        for raw in [vec![], vec![(0.0, 0.0)], vec![(1.0, 1.0); 3], vec![(0.0, 0.0), (0.1, 0.1)], vec![(0.0, 0.0), (f64::NAN, 1.0)]] {
            assert_eq!(shape_sample(raw).unwrap_err().key, keys::UI_SHAPE_TOO_SHORT);
        }
        let limits = shape_limits(shape_sample(vec![(0.0, 0.0), (100.0, 0.0)]).unwrap());
        assert_eq!(limits.accept, shape::ACCEPT);
        assert_eq!(limits.margin, shape::MARGIN);
        assert_eq!(shape_limits(circle_sample).accept, shape::ACCEPT_CURVE);
        assert_eq!(shape_limits(vec![]).accept, shape::ACCEPT);
        assert_eq!(shape_limits(vec![0; 128]).accept, shape::ACCEPT);
        let fixture: serde_json::Value = serde_json::from_str(include_str!("../../mousetroke-core/tests/fixtures/real-shapes.json")).unwrap();
        for (class, samples) in fixture["classes"].as_object().unwrap() {
            for sample in samples.as_array().unwrap() {
                let encoded: Vec<i32> = serde_json::from_value(sample.clone()).unwrap();
                let expected = if class == "Z" { shape::ACCEPT } else { shape::ACCEPT_CURVE };
                assert_eq!(shape_limits(encoded).accept, expected, "{class}");
            }
        }
    }

    #[test]
    fn shape_fixtures_are_core_encoded_and_empty_shapes_cannot_save() {
        // 与核心识别测试的 G 相同：先画圆弧，再向上、向左收笔。
        let arc = |t: f64| {
            let angle = std::f64::consts::PI / 4.0 + 1.5 * std::f64::consts::PI * t;
            (angle.cos() * 160.0, -angle.sin() * 160.0)
        };
        let raw: Vec<_> = (0..=160).map(|i| {
            let t = i as f64 / 160.0;
            if t < 0.78 {
                arc(t / 0.78)
            } else {
                let (a, b, f) = if t < 0.89 { (arc(1.0), (131.2, 0.0), (t - 0.78) / 0.11) }
                    else { ((131.2, 0.0), (8.0, 0.0), (t - 0.89) / 0.11) };
                (a.0 + (b.0 - a.0) * f, a.1 + (b.1 - a.1) * f)
            }
        }).collect();
        let expected = shape_sample(raw).unwrap();
        for text in [include_str!("../../../tools/uitest/config.json"), include_str!("../../../tools/uitest/config-en.json")] {
            let mut config: Config = serde_json::from_str(text).unwrap();
            assert!(validate_strokes(&config).is_ok());
            let step = config.scopes[0].bindings.iter_mut().flat_map(|b| &mut b.pattern).find(|s| matches!(s, Step::Shape { .. })).unwrap();
            let Step::Shape { templates } = step else { unreachable!() };
            assert_eq!(templates, &vec![expected.clone()]);
            templates.clear();
            assert_eq!(validate_strokes(&config).unwrap_err().key, keys::UI_SHAPE_NEEDS_SAMPLE);
            config.scopes[0].bindings.last_mut().unwrap().pattern = vec![Step::Trace { points: vec![] }];
            assert_eq!(validate_strokes(&config).unwrap_err().key, keys::UI_RECORDER_TOO_SHORT);
        }
    }
}
