//! 多语言词条运行时（设计见 `docs/I18N-DESIGN.md`）。
//!
//! - 词条在编译期嵌入（`include_str!`），首次取用时才解析（`OnceLock` + `Box::leak`，
//!   常驻内存约几十 KB；10 种语言合计几百 KB）。
//! - 当前语言是进程全局原子量：引擎在 `apply_config`、设置界面在启动与切换语言、
//!   命令行在 `main` 开头调用 [`set_lang`]。
//! - 取词条缺失时回退 zh-CN，再缺失返回键名本身（debug 构建同时 eprintln 一次）。
//! - 不引入新依赖：复数规则按 CLDR 基数规则手写（见 [`plural_category`]），
//!   回退顺序 `_<类别>` → `_other` → 基础键。
//! - 新增语言：加枚举值 + [`TAGS`]/[`SOURCES`] 对应项 + [`from_tag`] 映射 +
//!   [`plural_category`]/[`ui_font`] 分支 + `i18n/<tag>.json`，其余（目录一致性测试、
//!   键名常量、界面下拉框）自动。

use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::OnceLock;

/// 支持的语言。RTL 语言（阿拉伯语、希伯来语）与复杂文字系统（印地语）一期不做。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Lang {
    ZhCn,
    EnUs,
    EsEs,
    FrFr,
    DeDe,
    JaJp,
    PtBr,
    RuRu,
    ItIt,
    KoKr,
    /// 伪语言（仅 debug 构建认识设置值）：重音 + 拉长 40%，抓漏翻与布局溢出。
    /// 词条由 en-US 在运行时生成，不提交文件。
    QpsPloc,
}

/// 枚举值个数（含伪语言；必须与 [`Lang`] 的变体数一致）。
const LANG_COUNT: usize = 11;

const TAGS: [&str; LANG_COUNT] = [
    "zh-CN", "en-US", "es-ES", "fr-FR", "de-DE", "ja-JP", "pt-BR", "ru-RU", "it-IT", "ko-KR", "qps-ploc",
];

/// 各语言的词条源（编译期嵌入；伪语言由 en-US 生成，为空）。
const SOURCES: [Option<&str>; LANG_COUNT] = [
    Some(include_str!("../../../i18n/zh-CN.json")),
    Some(include_str!("../../../i18n/en-US.json")),
    Some(include_str!("../../../i18n/es-ES.json")),
    Some(include_str!("../../../i18n/fr-FR.json")),
    Some(include_str!("../../../i18n/de-DE.json")),
    Some(include_str!("../../../i18n/ja-JP.json")),
    Some(include_str!("../../../i18n/pt-BR.json")),
    Some(include_str!("../../../i18n/ru-RU.json")),
    Some(include_str!("../../../i18n/it-IT.json")),
    Some(include_str!("../../../i18n/ko-KR.json")),
    None,
];

/// 当前语言（进程全局；默认 zh-CN，各入口在启动/热加载时设置）。
static LANG: AtomicU8 = AtomicU8::new(0);

pub fn set_lang(lang: Lang) {
    LANG.store(lang as u8, Ordering::Relaxed);
}

pub fn lang() -> Lang {
    match LANG.load(Ordering::Relaxed) {
        n if n < LANG_COUNT as u8 => TAGS_BY_INDEX[n as usize],
        _ => Lang::ZhCn,
    }
}

/// 索引 → 枚举（`lang()` 用；保持与声明顺序一致）。
const TAGS_BY_INDEX: [Lang; LANG_COUNT] = [
    Lang::ZhCn, Lang::EnUs, Lang::EsEs, Lang::FrFr, Lang::DeDe, Lang::JaJp, Lang::PtBr, Lang::RuRu,
    Lang::ItIt, Lang::KoKr, Lang::QpsPloc,
];

impl Lang {
    /// BCP-47 标签（"zh-CN"）。
    pub fn tag(self) -> &'static str {
        TAGS[self as usize]
    }

    /// 解析标签（大小写不敏感），地区变体映射到基准语言：
    /// `zh` / `zh-Hans` / `zh-SG` → ZhCn；`zh-TW` / `zh-HK` 在 zh-TW 词条就绪前也映射到
    /// ZhCn；`es-*` → EsEs、`fr-*` → FrFr、`de-*` → DeDe、`pt-*` → PtBr、`ru-*` → RuRu、
    /// `it-*` → ItIt、`ko-*` → KoKr；`qps-ploc` 仅 debug 构建认识。
    pub fn from_tag(s: &str) -> Option<Lang> {
        let norm = s.trim().to_lowercase();
        let (primary, region) = match norm.split_once('-') {
            Some((p, r)) => (p, Some(r)),
            None => (norm.as_str(), None),
        };
        match primary {
            // zh-TW 词条就绪后把 tw/hk/hant 分支改成返回 ZhTw
            "zh" => Some(Lang::ZhCn),
            "en" => Some(Lang::EnUs),
            "es" => Some(Lang::EsEs),
            "fr" => Some(Lang::FrFr),
            "de" => Some(Lang::DeDe),
            "ja" => Some(Lang::JaJp),
            "pt" => Some(Lang::PtBr),
            "ru" => Some(Lang::RuRu),
            "it" => Some(Lang::ItIt),
            "ko" => Some(Lang::KoKr),
            "qps" if region.is_some_and(|r| r == "ploc") => {
                cfg!(debug_assertions).then_some(Lang::QpsPloc)
            }
            _ => None,
        }
    }
}

/// 语言设置（`settings.language`）+ 系统首选界面语言列表 → 实际语言。纯函数，便于测试。
/// 设置值不是 `auto` 且认识就用它；否则依次看系统列表；都不认识回退 en-US。
pub fn resolve(setting: &str, system: &[String]) -> Lang {
    let setting = setting.trim();
    if !setting.is_empty() && !setting.eq_ignore_ascii_case("auto") {
        if let Some(l) = Lang::from_tag(setting) {
            return l;
        }
    }
    for tag in system {
        if let Some(l) = Lang::from_tag(tag) {
            return l;
        }
    }
    Lang::EnUs
}

// ---------------------------------------------------------------- 词条

type Catalog = HashMap<&'static str, &'static str>;

fn leak(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn catalog(lang: Lang) -> &'static Catalog {
    static CATALOGS: [OnceLock<Catalog>; LANG_COUNT] = [
        OnceLock::new(), OnceLock::new(), OnceLock::new(), OnceLock::new(), OnceLock::new(), OnceLock::new(),
        OnceLock::new(), OnceLock::new(), OnceLock::new(), OnceLock::new(), OnceLock::new(),
    ];
    CATALOGS[lang as usize].get_or_init(|| match SOURCES[lang as usize] {
        Some(src) => parse(src),
        // 伪语言由 en-US 逐条生成（仅 debug 构建会走到这里）。
        // 注意是对每条的值变换，不能对整个文件文本变换——那样键名也会被加重音、JSON 也不再合法
        None => SOURCES[Lang::EnUs as usize]
            .map(parse)
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| (k, leak(pseudo(v))))
            .collect(),
    })
}

fn parse(src: &'static str) -> Catalog {
    serde_json::from_str::<HashMap<String, String>>(src)
        .unwrap_or_default()
        .into_iter()
        .map(|(k, v)| (leak(k), leak(v)))
        .collect()
}

/// 取词条。缺失时回退 zh-CN，再缺失返回键名本身（debug 构建打一条日志）。
pub fn t(key: &'static str) -> &'static str {
    try_t(key).unwrap_or_else(|| {
        #[cfg(debug_assertions)]
        eprintln!("[i18n] 缺少词条: {key}"); // i18n:allow 调试输出
        key
    })
}

/// 取词条，缺失返回 `None`（内置名称的回落判断用）。
pub fn try_t(key: &str) -> Option<&'static str> {
    catalog(lang())
        .get(key)
        .copied()
        .or_else(|| catalog(Lang::ZhCn).get(key).copied())
}

/// 带参数取词条：`tf(keys::OVERLAY_BRIGHTNESS, &[("value", "80")])`。
/// 含 `count` 参数时按该语言的复数规则选变体，回退顺序 `_<类别>` → `_other` → 基础键。
pub fn tf(key: &'static str, args: &[(&str, &str)]) -> String {
    let templ = lookup_plural(key, args).unwrap_or_else(|| t(key));
    interpolate(templ, args)
}

/// 内置名称的显示名：有 `name_key` 且词条存在 → 当前语言译文；否则用户起的名字。
pub fn display_name<'a>(name: &'a str, name_key: &'a Option<String>) -> &'a str {
    name_key.as_deref().and_then(try_t).unwrap_or(name)
}

/// CLDR 基数复数类别（cardinal）。UI 里 count 都是小整数，只实现整数规则。
fn plural_category(lang: Lang, n: i64) -> &'static str {
    let (n10, n100) = (n % 10, n % 100);
    match lang {
        // 亚洲语言无复数
        Lang::ZhCn | Lang::JaJp | Lang::KoKr => "other",
        // fr / pt-BR：0 和 1 都是单数
        Lang::FrFr | Lang::PtBr => {
            if (0..=1).contains(&n) {
                "one"
            } else {
                "other"
            }
        }
        // es / de / it / en（意语大数的 many 忽略，UI 用不到）
        Lang::EsEs | Lang::DeDe | Lang::ItIt | Lang::EnUs | Lang::QpsPloc => {
            if n == 1 {
                "one"
            } else {
                "other"
            }
        }
        // 俄语：one=以 1 结尾（除 11）；few=以 2-4 结尾（除 12-14）；many=其余
        Lang::RuRu => {
            if n10 == 1 && n100 != 11 {
                "one"
            } else if (2..=4).contains(&n10) && !(12..=14).contains(&n100) {
                "few"
            } else {
                "many"
            }
        }
    }
}

fn lookup_plural(key: &str, args: &[(&str, &str)]) -> Option<&'static str> {
    let count = args
        .iter()
        .find(|(n, _)| *n == "count")
        .and_then(|(_, v)| v.trim().parse::<i64>().ok());
    let mut cands: Vec<String> = Vec::new();
    if let Some(n) = count {
        let cat = plural_category(lang(), n);
        for s in [format!("_{cat}"), "_other".to_string()] {
            cands.push(format!("{key}{s}"));
        }
    }
    cands.push(key.to_string());
    let cur = catalog(lang());
    let zh = catalog(Lang::ZhCn);
    cands
        .iter()
        .find_map(|c| cur.get(c.as_str()).copied().or_else(|| zh.get(c.as_str()).copied()))
}

/// `{name}` 占位符替换。未知占位符原样保留。
fn interpolate(templ: &str, args: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(templ.len() + 16);
    let mut rest = templ;
    while let Some(i) = rest.find('{') {
        out.push_str(&rest[..i]);
        let after = &rest[i + 1..];
        match after.find('}') {
            Some(j) => {
                let name = &after[..j];
                match args.iter().find(|(n, _)| *n == name) {
                    Some((_, v)) => out.push_str(v),
                    None => {
                        out.push('{');
                        out.push_str(name);
                        out.push('}');
                    }
                }
                rest = &after[j + 1..];
            }
            None => {
                out.push_str(&rest[i..]);
                return out;
            }
        }
    }
    out.push_str(rest);
    out
}

// ---------------------------------------------------------------- 字体

/// 覆盖层标签、点选提示条等原生绘制文字的字体（按语言选择，不要散落判断）。
pub fn ui_font(lang: Lang) -> &'static str {
    match lang {
        Lang::ZhCn => "Microsoft YaHei UI",
        Lang::JaJp => "Yu Gothic UI",
        Lang::KoKr => "Malgun Gothic",
        // zh-TW → Microsoft JhengHei UI（加语言时在这里补）；西文与西里尔 Segoe UI 都覆盖
        _ => "Segoe UI",
    }
}

// ---------------------------------------------------------------- 伪语言

/// 伪语言变换（单条词条的值）：字母加重音、首尾 `[!! … !!]`、按长度补约 40% 填充；
/// 占位符 `{name}` 原样保留。由 en-US 生成（拉丁字母才能加重音；长度也更接近最长的真实语言）。
#[cfg(debug_assertions)]
fn pseudo(src: &str) -> String {
    const FILLER: &[u8] = b"0123456789";
    let accent = |c: char| -> char {
        match c {
            'a' => 'á', 'e' => 'é', 'i' => 'í', 'o' => 'ó', 'u' => 'ú', 'y' => 'ý', 'n' => 'ñ', 'c' => 'ç',
            'A' => 'Á', 'E' => 'É', 'I' => 'Í', 'O' => 'Ó', 'U' => 'Ú', 'Y' => 'Ý', 'N' => 'Ñ', 'C' => 'Ç',
            other => other,
        }
    };
    // 占位符整段跳过：{name}
    let mut body = String::new();
    let mut rest = src;
    while let Some(i) = rest.find('{') {
        body.extend(rest[..i].chars().map(accent));
        let after = &rest[i..];
        match after.find('}') {
            Some(j) => {
                body.push_str(&after[..=j]);
                rest = &after[j + 1..];
            }
            None => {
                body.extend(after.chars().map(accent));
                rest = "";
            }
        }
    }
    body.extend(rest.chars().map(accent));
    let pad = (body.chars().count() as f64 * 0.4).ceil() as usize;
    let filler: String = (0..pad).map(|i| FILLER[i % FILLER.len()] as char).collect();
    format!("[!! {body} {filler} !!]")
}

#[cfg(not(debug_assertions))]
fn pseudo(_src: &str) -> String {
    String::new()
}

/// 键名常量（build.rs 从 i18n/zh-CN.json 生成；写错键名编译不过）。
pub mod keys {
    include!(concat!(env!("OUT_DIR"), "/i18n_keys.rs"));
}

/// `t!(KEY)` / `t!(KEY, name = value, …)`：[`t`] / [`tf`] 的语法糖，参数值用 `Display` 格式化。
#[macro_export]
macro_rules! t {
    ($key:expr) => {
        $crate::i18n::t($key)
    };
    ($key:expr, $($name:ident = $value:expr),+ $(,)?) => {
        $crate::i18n::tf($key, &[$((stringify!($name), &($value).to_string())),+])
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// set_lang 是进程全局，动它的测试串行执行，避免与并行测试互相干扰。
    static LANG_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn resolve_setting_and_system() {
        // 显式设置优先
        assert_eq!(resolve("zh-CN", &["en-US".into()]), Lang::ZhCn);
        assert_eq!(resolve("EN-US", &[]), Lang::EnUs);
        assert_eq!(resolve("pt-BR", &[]), Lang::PtBr);
        // auto：跟随系统列表里第一个认识的
        assert_eq!(resolve("auto", &["en-US".into(), "zh-CN".into()]), Lang::EnUs);
        assert_eq!(resolve("auto", &["fr-FR".into(), "zh-Hans".into()]), Lang::FrFr);
        assert_eq!(resolve("auto", &["fr-CA".into(), "en-US".into()]), Lang::FrFr);
        assert_eq!(resolve("auto", &["ru-RU".into(), "en-US".into()]), Lang::RuRu);
        // 不认识的设置值当 auto 处理；系统也全不认识 → en-US
        assert_eq!(resolve("xx-YY", &["en-US".into()]), Lang::EnUs);
        assert_eq!(resolve("auto", &["xx-YY".into()]), Lang::EnUs);
        assert_eq!(resolve("", &[]), Lang::EnUs);
        // 空白容忍
        assert_eq!(resolve(" zh-CN ", &[]), Lang::ZhCn);
    }

    #[test]
    fn tag_aliases() {
        assert_eq!(Lang::from_tag("zh"), Some(Lang::ZhCn));
        assert_eq!(Lang::from_tag("ZH-cn"), Some(Lang::ZhCn));
        assert_eq!(Lang::from_tag("zh-Hans"), Some(Lang::ZhCn));
        assert_eq!(Lang::from_tag("zh-SG"), Some(Lang::ZhCn));
        // zh-TW 词条就绪前映射到 zh-CN
        assert_eq!(Lang::from_tag("zh-TW"), Some(Lang::ZhCn));
        assert_eq!(Lang::from_tag("zh-HK"), Some(Lang::ZhCn));
        assert_eq!(Lang::from_tag("en"), Some(Lang::EnUs));
        assert_eq!(Lang::from_tag("en-GB"), Some(Lang::EnUs));
        // 新语言与地区变体
        assert_eq!(Lang::from_tag("es"), Some(Lang::EsEs));
        assert_eq!(Lang::from_tag("es-MX"), Some(Lang::EsEs));
        assert_eq!(Lang::from_tag("fr-CA"), Some(Lang::FrFr));
        assert_eq!(Lang::from_tag("de-AT"), Some(Lang::DeDe));
        assert_eq!(Lang::from_tag("ja"), Some(Lang::JaJp));
        assert_eq!(Lang::from_tag("pt"), Some(Lang::PtBr));
        assert_eq!(Lang::from_tag("pt-PT"), Some(Lang::PtBr));
        assert_eq!(Lang::from_tag("ru-KZ"), Some(Lang::RuRu));
        assert_eq!(Lang::from_tag("it"), Some(Lang::ItIt));
        assert_eq!(Lang::from_tag("ko"), Some(Lang::KoKr));
        assert_eq!(Lang::from_tag("hi-IN"), None);
        assert_eq!(Lang::from_tag("ar-SA"), None, "RTL 一期不支持");
        assert_eq!(Lang::from_tag("qps-ploc"), cfg!(debug_assertions).then_some(Lang::QpsPloc));
        assert_eq!(Lang::ZhCn.tag(), "zh-CN");
        assert_eq!(Lang::EnUs.tag(), "en-US");
        assert_eq!(Lang::RuRu.tag(), "ru-RU");
    }

    /// CLDR 基数复数规则的边界（UI 里 count 都是小整数）。
    #[test]
    fn plural_categories() {
        use Lang::*;
        // 无复数
        assert_eq!(plural_category(ZhCn, 1), "other");
        assert_eq!(plural_category(JaJp, 1), "other");
        assert_eq!(plural_category(KoKr, 3), "other");
        // n == 1 → one
        assert_eq!(plural_category(EnUs, 1), "one");
        assert_eq!(plural_category(EnUs, 2), "other");
        assert_eq!(plural_category(EsEs, 1), "one");
        assert_eq!(plural_category(DeDe, 1), "one");
        assert_eq!(plural_category(ItIt, 1), "one");
        // 0 和 1 都是单数
        assert_eq!(plural_category(FrFr, 0), "one");
        assert_eq!(plural_category(FrFr, 1), "one");
        assert_eq!(plural_category(FrFr, 2), "other");
        assert_eq!(plural_category(PtBr, 0), "one");
        // 俄语三式
        assert_eq!(plural_category(RuRu, 1), "one");
        assert_eq!(plural_category(RuRu, 21), "one");
        assert_eq!(plural_category(RuRu, 101), "one");
        assert_eq!(plural_category(RuRu, 11), "many");
        assert_eq!(plural_category(RuRu, 2), "few");
        assert_eq!(plural_category(RuRu, 4), "few");
        assert_eq!(plural_category(RuRu, 22), "few");
        assert_eq!(plural_category(RuRu, 12), "many");
        assert_eq!(plural_category(RuRu, 5), "many");
        assert_eq!(plural_category(RuRu, 0), "many");
        assert_eq!(plural_category(RuRu, 100), "many");
    }

    #[test]
    fn fallback_and_missing() {
        let _g = LANG_LOCK.lock().unwrap();
        set_lang(Lang::EnUs);
        assert_eq!(t("common.ok"), "OK");
        set_lang(Lang::ZhCn);
        assert_eq!(t("common.ok"), "确定");
        // 缺失：返回键名本身
        assert_eq!(t("no.such.key"), "no.such.key");
        assert_eq!(try_t("no.such.key"), None);
    }

    #[test]
    fn interpolate_placeholders() {
        assert_eq!(interpolate("已在 {count} 个程序里禁用手势", &[("count", "3")]), "已在 3 个程序里禁用手势");
        // 未知占位符原样保留；孤立 '{' 不吞
        assert_eq!(interpolate("{a} {b}", &[("a", "1")]), "1 {b}");
        assert_eq!(interpolate("x {", &[]), "x {");
        assert_eq!(interpolate("无占位符", &[]), "无占位符");
    }

    #[test]
    fn ui_font_per_lang() {
        assert_eq!(ui_font(Lang::ZhCn), "Microsoft YaHei UI");
        assert_eq!(ui_font(Lang::EnUs), "Segoe UI");
        assert_eq!(ui_font(Lang::RuRu), "Segoe UI");
        assert_eq!(ui_font(Lang::JaJp), "Yu Gothic UI");
        assert_eq!(ui_font(Lang::KoKr), "Malgun Gothic");
    }

    #[test]
    #[cfg(debug_assertions)]
    fn pseudo_locale_transforms_each_value() {
        let _g = LANG_LOCK.lock().unwrap();
        set_lang(Lang::QpsPloc);
        let ok = t("common.ok");
        let with_param = tf("toast.apps_disabled", &[("count", "3")]);
        set_lang(Lang::ZhCn);
        // 键名不受影响、值被变换（曾对整个文件变换 → 解析失败 → 全部回退成中文）
        assert!(ok.starts_with("[!! ") && ok.ends_with(" !!]"), "{ok}");
        assert!(ok.contains('Ó'), "{ok}");
        // 占位符原样保留并被替换；复数按英文规则选 _other
        assert!(with_param.starts_with("[!! ") && with_param.contains(" 3 ápps"), "{with_param}");
        assert!(!with_param.contains("{count}"), "{with_param}");
    }

    #[test]
    fn display_name_fallback() {
        let _g = LANG_LOCK.lock().unwrap();
        set_lang(Lang::ZhCn);
        assert_eq!(display_name("复制", &Some("common.ok".into())), "确定");
        assert_eq!(display_name("我的手势", &Some("no.such.key".into())), "我的手势");
        assert_eq!(display_name("我的手势", &None), "我的手势");
    }
}
