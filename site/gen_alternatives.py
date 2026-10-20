#!/usr/bin/env python3
"""从已核实的双语数据和单一模板生成对比页；不在构建时访问网络。"""
from html import escape
from pathlib import Path
from string import Template
import json
import posixpath
import re
import sys
import xml.etree.ElementTree as ET
sys.dont_write_bytecode = True

from alternatives_data import CHECKED, ENTITIES, HUB, PAGES, ROWS, SCENARIOS, SOURCES
from gen import faq_entities

ROOT = Path(__file__).resolve().parent
ORIGIN = "https://mousetroke.pages.dev"
LANGUAGES = {"zh-CN": "简体中文", "en": "English"}


def directory(lang, slug=""):
    return ("/" if lang == "zh-CN" else "/en/") + "alternatives/" + (slug + "/" if slug else "")


def relative(current, target):
    """保留目录尾斜杠，使主站和镜像子路径都能解析。"""
    result = posixpath.relpath(target, current)
    return (result + "/") if target.endswith("/") else result


def render():
    template = Template((ROOT / "alternatives_template.html").read_text(encoding="utf-8"))
    outputs = {}
    for lang in LANGUAGES:
        home = (ROOT / ("index.html" if lang == "zh-CN" else "en/index.html")).read_text(encoding="utf-8")
        brand = re.search(r'<a class="brand".*?</a>', home, re.S).group()
        download = re.search(r'<div class="split">.*?</details>\s*</div>', home, re.S).group()
        for slug, data in [("", HUB), *PAGES.items()]:
            current = directory(lang, slug)
            home_url = relative(current, "/" if lang == "zh-CN" else "/en/")
            hub_url = relative(current, directory(lang))
            text = lambda value: value[lang]
            tr = lambda zh, en: zh if lang == "zh-CN" else en
            link = lambda path, label: f'<a href="{relative(current, path)}">{escape(label)}</a>'
            source_keys = list(dict.fromkeys([
                *(data["sources"] if slug else [key for page in PAGES.values() for key in page["sources"]]),
                "mousetroke", "mousetroke-home", "mousetroke-release",
            ]))
            numbers = {key: i + 1 for i, key in enumerate(source_keys)}

            def citations(keys):
                return ''.join(f'<sup><a href="#source-{numbers[key]}" aria-label="{tr("来源", "Source")} {numbers[key]}">[{numbers[key]}]</a></sup>' for key in keys)

            def value(entity, row):
                item = ENTITIES[entity]["facts"].get(row)
                return escape(text(item["text"])) + citations(item["sources"]) if item else "—"

            table_note = f'<p class="alt-note">{escape(text(HUB["table_note"]))}</p>'

            def table(entities, rows, caption):
                heading = '<th scope="col">' + tr("项目", "Feature") + '</th>' + ''.join(f'<th scope="col">{escape(ENTITIES[key]["name"])}</th>' for key in entities)
                body = ''.join('<tr><th scope="row">' + escape(text(ROWS[row])) + '</th>' + ''.join(f'<td>{value(key, row)}</td>' for key in entities) + '</tr>' for row in rows)
                return f'<div class="alt-table-scroll" role="region" tabindex="0" aria-label="{escape(caption)}"><table><caption>{escape(caption)}</caption><thead><tr>{heading}</tr></thead><tbody>{body}</tbody></table></div>' + table_note

            def section(ident, title, body):
                return f'<section id="{ident}" class="alt-section" aria-labelledby="h-{ident}"><h2 id="h-{ident}">{escape(title)}</h2>{body}</section>'

            if slug:
                entities = ["mousetroke", *data["entities"]]
                content = section("facts", tr("逐项比较", "Compare the facts"), table(entities, ROWS, tr("Mousetroke 与其他选择", "Mousetroke and the other options")))
                others = " / ".join(ENTITIES[key]["name"] for key in data["entities"])
                content += section("choose", tr("按你的需求选择", "Choose for your needs"), f'<div class="alt-choices"><div><h3>{tr("适合选择 Mousetroke 的情况", "Choose Mousetroke if…")}</h3><p>{escape(text(data["ours"]))}</p></div><div><h3>{escape(tr("适合选择 " + others + " 的情况", "Choose " + others + " if…"))}</h3><p>{escape(text(data["theirs"]))}</p></div></div>')
                content += section("differences", tr("实际使用中的差别", "Differences in daily use"), ''.join(f'<p>{escape(p)}</p>' for p in text(data["paragraphs"])))
                image = "settings-zh.webp" if lang == "zh-CN" else "settings-en.webp"
                alt = tr("Mousetroke 中文设置：手势列表、轨迹预览与动作绑定", "Mousetroke English settings showing gesture rules, stroke previews and action bindings")
                # 图片尺寸由同一张原始设置截图优化后保留。
                content += f'<figure class="alt-screenshot"><img src="{relative(current, "/assets/" + image)}" alt="{alt}" width="2360" height="1520" loading="lazy"><figcaption>{tr("Mousetroke 设置：按程序管理手势、查看轨迹与配置动作。", "Mousetroke settings: organize per-app gestures, inspect strokes and configure actions.")}</figcaption></figure>'
                related = ''.join('<li>' + link(directory(lang, other), text(PAGES[other]["title"])) + '</li>' for other in data["related"])
                content += section("related", tr("继续比较", "Related comparisons"), f'<ul class="alt-related">{related}<li>{link(directory(lang), tr("全部对比与选择指南", "All comparisons and the selection guide"))}</li></ul>')
                name = ENTITIES[data["entities"][0]]["name"]
                first_question = tr(f"Windows 11 上有什么好的 {name} 替代工具？", f"What is a good {name} alternative for Windows 11?")
                first_answer = tr("如果你主要用鼠标，需要全局手势、屏幕边角和可查看的 MIT 源码，可以考虑 Mousetroke。若需求是深度宏编辑、原生多指触控或只在浏览器内操作，请结合本页的选择建议评估。", "Consider Mousetroke if you mainly use a mouse and want system-wide gestures, screen corners and inspectable MIT source. For deep macro editing, native multi-finger input or browser-only use, follow the choices on this page.")
                faqs = [(first_question, first_answer),
                        (tr("Mousetroke 免费且开源吗？", "Is Mousetroke free and open source?"), tr("是，采用 MIT 许可证，没有广告或账号，无遥测。默认每天向 GitHub 检查一次更新，可在偏好设置中关闭。", "Yes. It uses the MIT license, with no ads, account or telemetry. It checks GitHub for updates once a day by default; turn that off in Preferences if you prefer.")),
                        text(data["limitation"])]
            else:
                routes = []
                for target, title, explanation in SCENARIOS:
                    route_link = link(directory(lang, target), tr("查看对比", "See comparison")) if target else link("/" if lang == "zh-CN" else "/en/", tr("了解 Mousetroke", "Explore Mousetroke"))
                    if target == "fastgestures":
                        route_link += " · " + link(directory(lang, "gesturesign"), "GestureSign / GestureSignv2")
                    routes.append(f'<li><h3>{escape(text(title))}</h3><p>{escape(text(explanation))}</p>{route_link}</li>')
                content = section("scenarios", tr("从使用场景开始", "Start with your scenario"), '<ol class="alt-scenarios">' + ''.join(routes) + '</ol>')
                cards = ''.join(f'<a class="alt-card" href="{relative(current, directory(lang, key))}"><h3>{escape(" / ".join(ENTITIES[e]["name"] for e in page["entities"]))}</h3><p>{escape(text(page["intro"]))}</p><span>{tr("查看对比", "Compare options")} →</span></a>' for key, page in PAGES.items())
                content += section("pages", tr("六组工具对比", "Six comparisons to explore"), '<div class="alt-grid">' + cards + '</div>')
                # 总览表按工具逐行列出，移动端只在表格容器内横向滚动。
                body = ''.join(f'<tr><th scope="row">{escape(entity["name"])}</th><td>{value(key, "platform")}</td><td>{value(key, "scope")}</td><td>{value(key, "license")}</td></tr>' for key, entity in ENTITIES.items())
                content += section("summary", tr("一张表看清范围与许可", "Scope and licensing at a glance"), f'<div class="alt-table-scroll" role="region" tabindex="0" aria-label="{tr("工具总览", "Tool overview")}"><table><caption>{tr("工具总览", "Tool overview")}</caption><thead><tr><th scope="col">{tr("工具", "Tool")}</th><th scope="col">{text(ROWS["platform"])}</th><th scope="col">{text(ROWS["scope"])}</th><th scope="col">{text(ROWS["license"])}</th></tr></thead><tbody>{body}</tbody></table></div>' + table_note)
                faqs = [
                    (tr("如何选择鼠标手势工具？", "How do I choose a mouse gesture tool?"), tr("先确认鼠标、触控板或触屏，再决定只用于浏览器还是全局。之后比较脚本深度、按程序规则、许可证和安装要求，并在自己的硬件与常用应用中测试。", "Start with mouse, touchpad or touchscreen input, then decide between browser-only and system-wide use. Compare scripting depth, per-app rules, licensing and installation requirements, and test on your hardware and daily apps.")),
                    (tr("什么时候选择 Mousetroke？", "When should I choose Mousetroke?"), tr("当你需要免费 MIT 开源的全局鼠标手势、屏幕边角和按程序规则，使用 Windows 10 1809+/11 x64，或愿意试用 macOS 11+ 测试版时。", "When you want free MIT-licensed system-wide mouse gestures, screen corners and per-app rules on Windows 10 1809+/11 x64, or want to try the macOS 11+ beta.")),
                    (tr("全局工具总比浏览器扩展好吗？", "Is a system-wide app always better than an extension?"), tr("不是。只操作网页时扩展可能已经足够；需要控制其他应用或屏幕边角时，再选择全局工具。触控优先时应选择明确说明对应设备支持的工具。", "No. An extension may be enough for web pages. Choose a system-wide tool for other apps or screen edges. For touch-first use, pick a tool that explicitly documents the relevant input device.")),
                ]
            faq = '\n'.join(f'<details open><summary>{escape(q)}</summary><div class="ans"><p>{escape(a)}</p></div></details>' for q, a in faqs)
            breadcrumbs = [link("/" if lang == "zh-CN" else "/en/", tr("首页", "Home"))]
            breadcrumbs.append(link(directory(lang), tr("工具对比", "Alternatives")) if slug else f'<span aria-current="page">{tr("工具对比", "Alternatives")}</span>')
            if slug:
                breadcrumbs.append(f'<span aria-current="page">{escape(" / ".join(ENTITIES[e]["name"] for e in data["entities"]))}</span>')
            variables = dict(
                lang=lang, title=escape(text(data["title"])), description=escape(text(data["description"])),
                canonical=ORIGIN + current, zh_url=ORIGIN + directory("zh-CN", slug), en_url=ORIGIN + directory("en", slug),
                assets=relative(current, "/assets/"), locale="zh_CN" if lang == "zh-CN" else "en_US",
                og_image=ORIGIN + ("/og.png" if lang == "zh-CN" else "/og-en.png"), structured_data="",
                skip=tr("跳到正文", "Skip to content"), brand=re.sub(r'href="[^"]+"', f'href="{home_url}"', brand, count=1),
                language_label=tr("语言", "Language"), language=LANGUAGES[lang],
                languages=''.join('<a' + (' class="cur"' if key == lang else '') + f' href="{relative(current, directory(key, slug))}" hreflang="{key}" lang="{key}">{label}</a>' for key, label in LANGUAGES.items()),
                breadcrumb_label=tr("面包屑", "Breadcrumb"), breadcrumbs=' <span aria-hidden="true">/</span> '.join(breadcrumbs),
                h1=escape(text(data["h1"])), intro=escape(text(data["intro"])), download=download,
                secondary_url=hub_url if slug else "#pages", secondary_label=tr("全部工具对比", "All comparisons") if slug else tr("浏览六组对比", "Browse six comparisons"),
                meta=tr("v0.2.1 · MIT 开源 · Windows 10 1809+/11 x64 · macOS 11+ 测试版", "v0.2.1 · MIT source · Windows 10 1809+/11 x64 · macOS 11+ beta"),
                content=content, faq_label=tr("常见问题", "Frequently asked questions"), faq=faq,
                bottom_title=tr("从一个常用手势开始", "Start with one everyday gesture"),
                bottom_text=tr("后退、关标签页、管理窗口，或从屏幕边角执行动作。Windows 常驻约 19 MB，Mac 约 13 MB。", "Go back, close a tab, manage a window or trigger an action from a screen corner. About 19 MB resident on Windows and 13 MB on Mac."),
                first_open=tr("Windows exe 尚未代码签名，SmartScreen 首次运行可能警告：更多信息 → 仍要运行。Mac 测试版自签名且未公证；拖入“应用程序”，如被拦截到“隐私与安全性 → 仍要打开”，再按引导授予辅助功能权限。", "The Windows exe is not code-signed yet, so SmartScreen may warn on first run: More info → Run anyway. The Mac beta is self-signed and not notarized: drag to Applications, use Privacy & Security → Open Anyway if blocked, then grant Accessibility in setup."),
                sources_label=tr("来源", "Sources"), checked_label=tr("最后核查", "Last checked"), checked=CHECKED,
                sources=''.join(f'<li id="source-{numbers[key]}"><a href="{escape(SOURCES[key][1])}">{escape(SOURCES[key][0])}</a></li>' for key in source_keys),
                license_label=tr("MIT 许可证", "MIT license"), footer_label=tr("页脚", "Footer"), home_url=home_url, home_label=tr("首页", "Home"), hub_url=hub_url, hub_label=tr("鼠标手势工具对比", "Compare alternatives"), llms_url=relative(current, "/llms.txt"),
            )
            page = template.substitute(variables)
            crumb_items = [(tr("首页", "Home"), ORIGIN + ("/" if lang == "zh-CN" else "/en/")), (tr("工具对比", "Alternatives"), ORIGIN + directory(lang))]
            if slug:
                crumb_items.append((" / ".join(ENTITIES[e]["name"] for e in data["entities"]), ORIGIN + current))
            schema = {"@context": "https://schema.org", "@graph": [
                {"@type": "WebPage", "@id": ORIGIN + current + "#page", "url": ORIGIN + current, "name": text(data["title"]), "description": text(data["description"]), "dateModified": CHECKED, "inLanguage": lang, "isPartOf": {"@type": "WebSite", "@id": ORIGIN + "/#site", "name": "Mousetroke", "url": ORIGIN + "/"}},
                {"@type": "FAQPage", "mainEntity": faq_entities(page)},
                {"@type": "BreadcrumbList", "itemListElement": [{"@type": "ListItem", "position": i + 1, "name": name, "item": url} for i, (name, url) in enumerate(crumb_items)]},
            ]}
            variables["structured_data"] = json.dumps(schema, ensure_ascii=False, indent=2).replace("<", "\\u003c")
            outputs[ROOT / current.lstrip("/") / "index.html"] = template.substitute(variables)
    return outputs


def sync_sitemap():
    """保留现有条目，只更新内容已改的首页日期和新增目录页。"""
    path = ROOT / "sitemap.xml"
    content = path.read_text(encoding="utf-8")
    content = re.sub(r'(<loc>https://mousetroke\.pages\.dev/(?:[a-z]{2}/)?</loc>\s*<lastmod>)[^<]+', rf'\g<1>{CHECKED}', content)
    ns = {"s": "http://www.sitemaps.org/schemas/sitemap/0.9"}
    existing = {node.text for node in ET.fromstring(content).findall("s:url/s:loc", ns)}
    additions = []
    for slug in ["", *PAGES]:
        for lang in LANGUAGES:
            url = ORIGIN + directory(lang, slug)
            if url not in existing:
                links = '\n'.join(f'    <xhtml:link rel="alternate" hreflang="{key}" href="{ORIGIN + directory("en" if key == "x-default" else key, slug)}"/>' for key in [*LANGUAGES, "x-default"])
                additions.append(f'  <url>\n    <loc>{url}</loc>\n    <lastmod>{CHECKED}</lastmod>\n{links}\n  </url>\n')
    path.write_text(content.replace('</urlset>', ''.join(additions) + '</urlset>'), encoding="utf-8")


def main():
    for path, content in render().items():
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")
        print(f"{path.relative_to(ROOT)}: written")
    sync_sitemap()


if __name__ == "__main__":
    main()
