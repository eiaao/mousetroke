//! Mousetroke 设置程序。
//!
//! 独立进程、按需打开、关掉即退（常驻引擎不含 WebView，保持低占用）。
//! 只读写原生 `config.json`（原子写入），引擎轮询 mtime 热加载——两个进程之间
//! 更新时通过托盘退出消息交接进程；命名事件用于定位关于页。配置语义复用 mousetroke-core。
//!
//! 以库的形式编进 `mousetroke.exe`（`mousetroke settings [配置路径]` 启动本界面进程）：
//! 只发布一个 exe，但界面仍是独立进程——常驻的引擎进程里没有 WebView。

mod icons;
mod picker;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use mousetroke_core::i18n::{keys, t};
use mousetroke_core::schema::{default_config_path, Config, Step};
use mousetroke_core::shape;
use serde::Serialize;
use tauri::{Manager, State, WebviewWindow};

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
    config_path: String,
    /// 文件不存在时为默认配置，`config_exists = false`。
    config: Config,
    config_exists: bool,
    /// 配置文件存在但解析失败时的错误（界面提示并允许重新导入/重置）。
    load_error: Option<String>,
    engine: EngineStatus,
    /// 系统首选界面语言列表（前端在 language = auto 时用；不用 navigator.language，
    /// WebView2 报告的语言与 Windows 界面语言不一定一致）。
    system_langs: Vec<String>,
    /// 后端按配置 + 系统语言解析出的语言标签。
    resolved_lang: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EngineStatus {
    running: bool,
    autostart: bool,
    /// 同目录下是否有 mousetroke.exe（可从界面启动/设置自启）。
    exe_found: bool,
}


#[derive(Serialize)]
struct RunningApp {
    exe: String,
    title: String,
}

/// 引擎 exe：合并版就是自己（`mousetroke.exe`）；开发时单独的设置程序则找同目录的 mousetroke.exe。
fn engine_exe() -> Option<PathBuf> {
    let me = std::env::current_exe().ok()?;
    if !mousetroke_core::matcher::file_name(&me.to_string_lossy()).eq_ignore_ascii_case("mousetroke-settings.exe") {
        return Some(me);
    }
    let p = me.parent()?.join("mousetroke.exe");
    p.exists().then_some(p)
}

fn engine_status() -> EngineStatus {
    EngineStatus {
        running: mousetroke_win::engine_running(),
        autostart: mousetroke_win::autostart::is_enabled(),
        exe_found: engine_exe().is_some(),
    }
}


fn read_text(path: &Path) -> Result<String, Msg> {
    std::fs::read_to_string(path).map_err(|e| Msg::new(keys::ERROR_READ_FILE, &[("path", path.display().to_string()), ("err", e.to_string())]))
}

// ---------------------------------------------------------------- 命令

#[tauri::command]
fn init_state(paths: State<AppPaths>) -> InitState {
    use mousetroke_core::i18n;
    let path = &paths.config;
    let exists = path.exists();
    let (config, load_error) = if exists {
        match Config::load(path) {
            Ok(c) => (c, None),
            Err(e) => (Config::default(), Some(e)),
        }
    } else {
        // 首次运行（引擎还没生成配置文件）：展示内置默认手势，保存时写盘
        (Config::builtin(), None)
    };
    let system_langs = mousetroke_win::system_langs();
    let resolved = i18n::resolve(&config.settings.language, &system_langs);
    // 后端自己需要的少量文字（窗口标题、点选提示条）也用这个语言
    i18n::set_lang(resolved);
    InitState {
        about: take_about_request(),
        config_path: path.to_string_lossy().into_owned(),
        config,
        config_exists: exists,
        load_error,
        engine: engine_status(),
        system_langs,
        resolved_lang: resolved.tag().into(),
    }
}

/// 界面切换语言后调用：后端 set_lang 并更新窗口标题。
#[tauri::command]
fn set_ui_lang(tag: String, window: WebviewWindow) {
    use mousetroke_core::i18n;
    if let Some(l) = i18n::Lang::from_tag(&tag) {
        i18n::set_lang(l);
    }
    let _ = window.set_title(i18n::t(i18n::keys::UI_WINDOW_TITLE));
}

#[tauri::command]
fn save_config(paths: State<AppPaths>, config: Config) -> MsgResult<()> {
    validate_strokes(&config)?;
    config.save(&paths.config).map_err(|e| Msg::new(keys::ERROR_SAVE, &[("path", paths.config.display().to_string()), ("err", e)]))
}

#[tauri::command]
fn get_engine_status() -> EngineStatus {
    engine_status()
}

/// 校验按键序列语法；返回错误词条（含位置），合法返回 None。
/// `reason` 参数本身也是一个词条键，由前端翻译。
#[tauri::command]
fn validate_keys(keys: String) -> Option<Msg> {
    mousetroke_core::keyseq::parse(&keys, false).err().map(|e| {
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
    validate_strokes(&config)?;
    config.save(Path::new(&path)).map_err(|e| Msg::new(keys::ERROR_SAVE, &[("path", path), ("err", e)]))
}

/// 文件选择。`kind`：json | exe | any；`save` 为 true 时是保存对话框。
#[tauri::command]
async fn pick_file(window: WebviewWindow, kind: String, save: bool, title: String) -> Option<String> {
    let mut d = rfd::FileDialog::new().set_title(&title).set_parent(&window);
    d = match kind.as_str() {
        "json" => d.add_filter(t(keys::UI_PICKER_FILTER_JSON), &["json"]),
        "exe" => d.add_filter(t(keys::UI_PICKER_FILTER_PROGRAM), &["exe", "lnk", "bat", "cmd"]),
        _ => d,
    };
    let picked = if save { d.set_file_name("mousetroke-config.json").save_file() } else { d.pick_file() };
    picked.map(|p| p.to_string_lossy().into_owned())
}

#[tauri::command]
fn running_apps() -> Vec<RunningApp> {
    let mut apps: Vec<RunningApp> = mousetroke_win::targeting::running_apps()
        .into_iter()
        .map(|(exe, title)| RunningApp { exe, title })
        .collect();
    apps.sort_by_key(|a| mousetroke_core::matcher::file_name(&a.exe).to_lowercase());
    apps
}

/// 一批程序的图标（data URL；取不到为 null）。在线程池里跑，不卡界面。
#[tauri::command(async)]
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
#[tauri::command(async)]
fn preset_apps() -> Vec<PresetApp> {
    use mousetroke_core::i18n::t;
    mousetroke_core::presets::DISABLE_SUGGESTIONS
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
        log_path: mousetroke_win::log::path().to_string_lossy().into_owned(),
    }
}

const HOMEPAGES: [(&str, &str); 10] = [
    ("zh-CN", "https://mousetroke.pages.dev/"),
    ("en-US", "https://mousetroke.pages.dev/en/"),
    ("es-ES", "https://mousetroke.pages.dev/es/"),
    ("fr-FR", "https://mousetroke.pages.dev/fr/"),
    ("de-DE", "https://mousetroke.pages.dev/de/"),
    ("ja-JP", "https://mousetroke.pages.dev/ja/"),
    ("pt-BR", "https://mousetroke.pages.dev/pt/"),
    ("ru-RU", "https://mousetroke.pages.dev/ru/"),
    ("it-IT", "https://mousetroke.pages.dev/it/"),
    ("ko-KR", "https://mousetroke.pages.dev/ko/"),
];
const GITHUB_URL: &str = "https://github.com/eiaao/mousetroke";
const ISSUES_URL: &str = "https://github.com/eiaao/mousetroke/issues";

/// 链接仅由后端白名单决定；没有对应首页的语言回退到英文。
fn project_url(what: &str, lang: &str) -> Option<&'static str> {
    match what {
        "homepage" => Some(HOMEPAGES.iter().find(|(tag, _)| *tag == lang).unwrap_or(&HOMEPAGES[1]).1),
        "github" => Some(GITHUB_URL),
        "issues" => Some(ISSUES_URL),
        "releases" => Some(mousetroke_win::update::RELEASES_URL),
        _ => None,
    }
}

/// 用默认程序打开日志、配置文件夹或白名单中的项目链接。
#[tauri::command]
fn open_path(what: String, paths: State<AppPaths>) -> MsgResult<()> {
    match what.as_str() {
        "log" => {
            let log = mousetroke_win::log::path();
            if !log.exists() {
                return Err(Msg::new(keys::ERROR_NO_LOG_YET, &[]));
            }
            mousetroke_win::input::shell_execute("open", &log.to_string_lossy(), "");
        }
        "config_dir" => {
            let dir = paths.config.parent().map(Path::to_path_buf).unwrap_or_default();
            let _ = std::process::Command::new(mousetroke_win::input::system_exe("explorer.exe")).arg(dir).spawn();
        }
        _ => {
            let Some(url) = project_url(&what, mousetroke_core::i18n::lang().tag()) else {
                return Err(Msg::new(keys::ERROR_UNKNOWN_LOCATION, &[("what", what)]));
            };
            mousetroke_win::input::shell_execute("open", url, "");
        }
    }
    Ok(())
}

/// 拖入的文件 → 程序完整路径（.exe / 指向 exe 的 .lnk）。
#[tauri::command]
fn resolve_app(path: String) -> MsgResult<String> {
    picker::resolve_app(&path)
}

/// 在屏幕上点选一个窗口，返回它的程序。点选期间设置窗口先最小化让开，结束后还原。
#[tauri::command(async)]
fn pick_window(window: WebviewWindow) -> MsgResult<Option<picker::PickedApp>> {
    let _ = window.minimize();
    std::thread::sleep(std::time::Duration::from_millis(250)); // 等最小化动画让开
    let picked = picker::pick_window();
    let _ = window.unminimize();
    let _ = window.set_focus();
    picked
}

#[tauri::command]
fn set_autostart(on: bool) -> MsgResult<EngineStatus> {
    let exe = engine_exe().ok_or_else(|| Msg::new(keys::ERROR_ENGINE_NOT_FOUND, &[]))?;
    mousetroke_win::autostart::set_for(on, &exe.to_string_lossy())
        .map_err(|e| Msg::new(keys::ERROR_AUTOSTART, &[("err", e)]))?;
    Ok(engine_status())
}

#[tauri::command]
fn start_engine(paths: State<AppPaths>) -> MsgResult<EngineStatus> {
    let exe = engine_exe().ok_or_else(|| Msg::new(keys::ERROR_ENGINE_NOT_FOUND, &[]))?;
    std::process::Command::new(exe)
        .arg("run")
        .arg("--config")
        .arg(&paths.config)
        .spawn()
        .map_err(|e| Msg::new(keys::ERROR_START_ENGINE, &[("err", e.to_string())]))?;
    // 给引擎一点时间创建单实例互斥体
    for _ in 0..20 {
        std::thread::sleep(std::time::Duration::from_millis(50));
        if mousetroke_win::engine_running() {
            break;
        }
    }
    Ok(engine_status())
}

/// 在资源管理器中显示配置文件。
#[tauri::command]
fn reveal_config(paths: State<AppPaths>) {
    let arg = format!("/select,\"{}\"", paths.config.display());
    mousetroke_win::input::shell_execute("open", &mousetroke_win::input::system_exe("explorer.exe").to_string_lossy(), &arg);
}

// ---------------------------------------------------------------- 单实例

/// 自动复位事件跨进程传递定位请求；已有设置窗口也能跳到关于页。
fn about_event() -> windows::Win32::Foundation::HANDLE {
    static EVENT: std::sync::OnceLock<isize> = std::sync::OnceLock::new();
    let value = *EVENT.get_or_init(|| unsafe {
        windows::Win32::System::Threading::CreateEventW(None, false, false, windows::core::w!("Mousetroke::settings-about"))
            .map(|h| h.0 as isize).unwrap_or(0)
    });
    windows::Win32::Foundation::HANDLE(value as *mut core::ffi::c_void)
}

#[tauri::command]
fn take_about_request() -> bool {
    unsafe { windows::Win32::System::Threading::WaitForSingleObject(about_event(), 0) == windows::Win32::Foundation::WAIT_OBJECT_0 }
}

#[tauri::command]
async fn check_update() -> MsgResult<mousetroke_win::update::Check> {
    tauri::async_runtime::spawn_blocking(mousetroke_win::update::check).await
        .map_err(|_| Msg::new(keys::UPDATE_ERROR_NETWORK, &[]))?
        .map_err(|e| Msg::new(e.key(), &[]))
}

#[tauri::command]
async fn install_update(paths: State<'_, AppPaths>, app: tauri::AppHandle) -> MsgResult<()> {
    let exe = engine_exe().ok_or_else(|| Msg::new(keys::ERROR_ENGINE_NOT_FOUND, &[]))?;
    let config = paths.config.clone();
    tauri::async_runtime::spawn_blocking(move || mousetroke_win::update::install(&exe, &config)).await
        .map_err(|_| Msg::new(keys::UPDATE_ERROR_REPLACE, &[]))?
        .map_err(|e| Msg::new(e.key(), &[]))?;
    // 新进程等待本 PID 退出；引擎也通过托盘同一退出路径停止。
    app.exit(0);
    Ok(())
}

/// 设置窗口已开着 → 把它带到前台并退出本进程。
/// 标题可能已被切换成其他语言，把已知的都找一遍。
fn focus_existing() -> bool {
    use windows::core::PCWSTR;
    use windows::Win32::System::Threading::CreateMutexW;
    use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, IsIconic, SetForegroundWindow, ShowWindow, SW_RESTORE};
    let name: Vec<u16> = "Mousetroke::settings".encode_utf16().chain([0]).collect();
    unsafe {
        let _ = CreateMutexW(None, false, PCWSTR(name.as_ptr()));
        if windows::Win32::Foundation::GetLastError() != windows::Win32::Foundation::ERROR_ALREADY_EXISTS {
            return false;
        }
        for title in ["Mousetroke 设置", "Mousetroke Settings", t(keys::UI_WINDOW_TITLE)] { /* i18n:allow 已知语言的窗口标题兜底匹配 */
            let wide: Vec<u16> = title.encode_utf16().chain([0]).collect();
            if let Ok(h) = FindWindowW(PCWSTR::null(), PCWSTR(wide.as_ptr())) {
                if IsIconic(h).as_bool() {
                    let _ = ShowWindow(h, SW_RESTORE);
                }
                let _ = SetForegroundWindow(h);
                break;
            }
        }
    }
    true
}

/// 运行设置界面（阻塞到窗口关闭）。`config` 缺省为默认配置路径。已有设置窗口时把它带到前台后返回。
pub fn run(config: Option<PathBuf>) {
    let event = about_event();
    if std::env::args().any(|s| s == "--about") {
        unsafe { let _ = windows::Win32::System::Threading::SetEvent(event); }
    }
    // 启动时按配置确定语言：focus_existing 找窗口、托盘/气泡等少量原生文字都依赖它
    let config = config.unwrap_or_else(default_config_path);
    let langs = mousetroke_win::system_langs();
    let lang = Config::load(&config)
        .map(|c| mousetroke_core::i18n::resolve(&c.settings.language, &langs))
        .unwrap_or_else(|_| mousetroke_core::i18n::resolve("auto", &langs));
    mousetroke_core::i18n::set_lang(lang);
    if focus_existing() {
        return;
    }
    tauri::Builder::default()
        .manage(AppPaths { config })
        .invoke_handler(tauri::generate_handler![
            init_state,
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
            take_about_request,
            open_path,
            pick_window,
            set_autostart,
            start_engine,
            reveal_config,
        ])
        .setup(|app| {
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
    fn project_urls_follow_ui_language_and_only_allow_fixed_destinations() {
        for (tag, suffix) in [
            ("zh-CN", ""), ("en-US", "en/"), ("es-ES", "es/"), ("fr-FR", "fr/"),
            ("de-DE", "de/"), ("ja-JP", "ja/"), ("pt-BR", "pt/"), ("ru-RU", "ru/"),
            ("it-IT", "it/"), ("ko-KR", "ko/"), ("qps-ploc", "en/"), ("unknown", "en/"),
        ] {
            assert_eq!(project_url("homepage", tag), Some(format!("https://mousetroke.pages.dev/{suffix}").as_str()));
            assert_eq!(project_url("github", tag), Some("https://github.com/eiaao/mousetroke"));
            assert_eq!(project_url("issues", tag), Some("https://github.com/eiaao/mousetroke/issues"));
            assert_eq!(project_url("releases", tag), Some(mousetroke_win::update::RELEASES_URL));
        }
        for what in ["", "https://example.com", "file:///C:/Windows", "Homepage", "github/issues"] {
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
        for text in [include_str!("../../../target/uitest/config.json"), include_str!("../../../target/uitest/config-en.json")] {
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
