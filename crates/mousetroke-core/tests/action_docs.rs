//! 注册表是目录的唯一来源；UPDATE_ACTION_DOCS=1 时只重写两个标记之间的内容。
use mousetroke_core::{
    i18n::Lang,
    presets::actions::{Implementation, ParamDefault, ParamKind, Preset, REGISTRY},
};
use std::{collections::BTreeMap, fmt::Write, path::Path};

const START: &str = "<!-- action-library:generated:start -->";
const END: &str = "<!-- action-library:generated:end -->";

struct DocText {
    headings: &'static str,
    required: &'static str,
    default: &'static str,
    search_default: &'static str,
    visible_when: &'static str,
}

fn localized_text(lang: Lang) -> DocText {
    let (headings, required, default, search_default, visible_when) = match lang {
        Lang::EnUs | Lang::QpsPloc => (
            "| Action / ID | What it does / where it works | Parameters | Windows | Mac |\n",
            "required", "default", "Baidu for Simplified Chinese UI; Google otherwise",
            "; only when {field} is {choice}",
        ),
        Lang::ZhCn => (
            "| 动作 / ID | 用途与适用范围 | 参数 | Windows | Mac |\n",
            "必填", "默认", "简体中文界面默认百度，其他语言默认 Google",
            "；仅当{field}为{choice}时",
        ),
        Lang::DeDe => (
            "| Aktion / ID | Funktion und Einsatzbereich | Parameter | Windows | Mac |\n",
            "erforderlich", "Standard:", "Baidu bei vereinfachter chinesischer Oberfläche, sonst Google",
            "; nur bei {field}: {choice}",
        ),
        Lang::EsEs => (
            "| Acción / ID | Función y ámbito de uso | Parámetros | Windows | Mac |\n",
            "obligatorio", "predeterminado:", "Baidu con la interfaz en chino simplificado; Google en los demás idiomas",
            "; solo si {field} es {choice}",
        ),
        Lang::FrFr => (
            "| Action / ID | Fonction et contexte d’utilisation | Paramètres | Windows | Mac |\n",
            "obligatoire", "par défaut :", "Baidu avec l’interface en chinois simplifié ; Google sinon",
            " ; uniquement si {field} vaut {choice}",
        ),
        Lang::ItIt => (
            "| Azione / ID | Funzione e ambito d’uso | Parametri | Windows | Mac |\n",
            "obbligatorio", "predefinito:", "Baidu con l’interfaccia in cinese semplificato; Google nelle altre lingue",
            "; solo se {field} è {choice}",
        ),
        Lang::JaJp => (
            "| 動作 / ID | 機能と利用範囲 | パラメーター | Windows | Mac |\n",
            "必須", "既定値:", "簡体字中国語の画面では Baidu、それ以外では Google が既定",
            "；{field}が{choice}の場合のみ",
        ),
        Lang::KoKr => (
            "| 동작 / ID | 기능과 사용 범위 | 매개변수 | Windows | Mac |\n",
            "필수", "기본값:", "중국어 간체 UI는 Baidu, 나머지 언어는 Google이 기본값",
            "; {field} 항목이 {choice}일 때만 표시",
        ),
        Lang::PtBr => (
            "| Ação / ID | Função e onde usar | Parâmetros | Windows | Mac |\n",
            "obrigatório", "padrão:", "Baidu na interface em chinês simplificado; Google nos demais idiomas",
            "; somente se {field} for {choice}",
        ),
        Lang::RuRu => (
            "| Действие / ID | Назначение и область применения | Параметры | Windows | Mac |\n",
            "обязательно", "по умолчанию:", "Baidu для интерфейса на упрощённом китайском, иначе Google",
            "; только при выборе «{choice}» в поле «{field}»",
        ),
    };
    DocText { headings, required, default, search_default, visible_when }
}

fn cell(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('|', "&#124;")
        .replace('\n', "<br>")
}

fn parameters(preset: &Preset, lang: Lang, catalog: &BTreeMap<String, String>) -> String {
    let defaults = preset.default_params(lang);
    let text = localized_text(lang);
    preset
        .params
        .iter()
        .map(|param| {
            let label = cell(&catalog[param.label_key]);
            let default = defaults[param.name].as_str().unwrap();
            let choices = if let ParamKind::Choice { choices } = param.kind {
                choices
                    .iter()
                    .map(|choice| {
                        format!("{} (`{}`)", cell(&catalog[choice.label_key]), choice.value)
                    })
                    .collect::<Vec<_>>()
                    .join(" / ")
            } else {
                String::new()
            };
            let default_label = if let ParamKind::Choice { choices } = param.kind {
                cell(
                    &catalog[choices
                        .iter()
                        .find(|choice| choice.value == default)
                        .unwrap()
                        .label_key],
                )
            } else {
                format!("`{}`", cell(default))
            };
            let detail = if default.is_empty() {
                text.required.into()
            } else if matches!(param.default, ParamDefault::SearchEngine) {
                text.search_default.into()
            } else {
                format!("{} {default_label}", text.default)
            };
            let condition = param.visible_when.map(|condition| {
                let field = preset.params.iter().find(|p| p.name == condition.param).unwrap();
                let ParamKind::Choice { choices } = field.kind else { panic!() };
                let choice = choices.iter().find(|c| c.value == condition.equals).unwrap();
                text.visible_when.replace("{field}", &cell(&catalog[field.label_key])).replace("{choice}", &cell(&catalog[choice.label_key]))
            }).unwrap_or_default();
            format!(
                "{label} (`{}`): {}{detail}{condition}",
                param.name,
                if choices.is_empty() {
                    String::new()
                } else {
                    format!("{choices}; ")
                }
            )
        })
        .collect::<Vec<_>>()
        .join("<br>")
}

fn availability(implementation: Implementation, note: Option<&str>, catalog: &BTreeMap<String, String>) -> String {
    let availability: String = match implementation {
        Implementation::Implemented(_) => "✓".into(),
        Implementation::Unsupported { reason_key } | Implementation::Pending { reason_key } => {
            cell(&catalog[reason_key])
        }
    };
    match note { Some(key) if matches!(implementation, Implementation::Implemented(_)) => format!("{availability}<br>{}", cell(&catalog[key])), _ => availability }
}

fn catalogue(lang: Lang, catalog: &BTreeMap<String, String>) -> String {
    let mut out = String::new();
    let text = localized_text(lang);
    let mut previous = None;
    for preset in REGISTRY {
        if previous != Some(preset.category) {
            let category = serde_json::to_value(preset.category).unwrap();
            writeln!(
                out,
                "\n### {}\n",
                catalog[&format!("preset.category.{}", category.as_str().unwrap())]
            )
            .unwrap();
            out.push_str(text.headings);
            out.push_str("|---|---|---|---|---|\n");
            previous = Some(preset.category);
        }
        writeln!(
            out,
            "| {}<br>`{}` | {} | {} | {} | {} |",
            cell(&catalog[preset.name_key]),
            preset.id,
            cell(&catalog[preset.description_key]),
            if preset.params.is_empty() {
                "—".into()
            } else {
                parameters(preset, lang, catalog)
            },
            availability(preset.windows, preset.platform_notes.windows, catalog),
            availability(preset.macos, preset.platform_notes.macos, catalog)
        )
        .unwrap();
    }
    out.push('\n');
    out
}

#[test]
fn action_library_tables_match_registry() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (dir, tag, lang) in [
        ("en", "en-US", Lang::EnUs), ("zh-CN", "zh-CN", Lang::ZhCn),
        ("de", "de-DE", Lang::DeDe), ("es", "es-ES", Lang::EsEs),
        ("fr", "fr-FR", Lang::FrFr), ("it", "it-IT", Lang::ItIt),
        ("ja", "ja-JP", Lang::JaJp), ("ko", "ko-KR", Lang::KoKr),
        ("pt", "pt-BR", Lang::PtBr), ("ru", "ru-RU", Lang::RuRu),
    ] {
        let catalog: BTreeMap<String, String> = serde_json::from_str(
            &std::fs::read_to_string(root.join(format!("i18n/{tag}.json"))).unwrap(),
        )
        .unwrap();
        let path = root.join(format!("docs/guide/{dir}/action-library.md"));
        let doc = std::fs::read_to_string(&path)
            .unwrap()
            .replace("\r\n", "\n");
        assert_eq!(doc.matches(START).count(), 1);
        assert_eq!(doc.matches(END).count(), 1);
        let (before, rest) = doc.split_once(START).unwrap();
        let (_, after) = rest.split_once(END).unwrap();
        let expected = format!("{before}{START}\n{}{END}{after}", catalogue(lang, &catalog));
        if std::env::var("UPDATE_ACTION_DOCS").as_deref() == Ok("1") {
            std::fs::write(&path, expected).unwrap();
        } else {
            assert_eq!(doc, expected, "{dir}/action-library.md is stale; run UPDATE_ACTION_DOCS=1 cargo test -p mousetroke-core --test action_docs");
        }
    }
}
