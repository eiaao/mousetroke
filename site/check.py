#!/usr/bin/env python3
"""校验 10 个语言页面：结构与英文模板一致、JSON 合法、元数据自指、无英文残留。"""
import json, re, sys, pathlib
sys.dont_write_bytecode = True
from gen_changelog import render as render_changelog
from gen import faq_entities
from html.parser import HTMLParser
from urllib.parse import unquote, urljoin, urlsplit
import xml.etree.ElementTree as ET
from gen_alternatives import render as render_alternatives, directory, ORIGIN
from alternatives_data import CHECKED, PAGES as ALTERNATIVES
ROOT = pathlib.Path(__file__).resolve().parent
PAGES = {"zh-CN": "index.html", "en": "en/index.html", "es": "es/index.html", "fr": "fr/index.html",
         "de": "de/index.html", "ja": "ja/index.html", "pt-BR": "pt/index.html", "ru": "ru/index.html",
         "it": "it/index.html", "ko": "ko/index.html"}
DIRS = {"zh-CN": "", "en": "en", "es": "es", "fr": "fr", "de": "de", "ja": "ja", "pt-BR": "pt", "ru": "ru", "it": "it", "ko": "ko"}
# 结构指纹：这些属性/标签的出现次数必须与英文页一致
MARKS = ['data-p="', 'data-act="', 'class="row', '<details', 'class="rule', 'data-edge="', 'class="sw"', '<kbd>', '<dt>', 'id="h-']
import argparse
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--allow-mac-size-placeholder", action="store_true", help="预发布时允许尚未测量的 Mac DMG 大小")
args = parser.parse_args()
errors = []
tpl = (ROOT / "en/index.html").read_text(encoding="utf-8")
ref = {m: tpl.count(m) for m in MARKS}


class PlatformDefaults(HTMLParser):
    """无 JS 时 Windows 可见、Mac 隐藏；其他平台下载菜单列出全部平台。"""
    def __init__(self):
        super().__init__()
        self.menu_depth = 0
        self.problems = []
        self.counts = {"os-win": 0, "os-mac": 0}

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        classes = attrs.get("class", "").split()
        if tag == "div" and "split-menu" in classes:
            self.menu_depth += 1
        for platform in self.counts:
            if platform in classes:
                self.counts[platform] += 1
                if platform == "os-mac" and not self.menu_depth and "hidden" not in attrs:
                    self.problems.append("Mac variant missing hidden")
                if platform == "os-win" and "hidden" in attrs:
                    self.problems.append("Windows variant hidden by default")

    def handle_endtag(self, tag):
        if tag == "div" and self.menu_depth:
            self.menu_depth -= 1


platform_ref = PlatformDefaults()
platform_ref.feed(tpl)
for lang, rel in PAGES.items():
    s = (ROOT / rel).read_text(encoding="utf-8")
    platforms = PlatformDefaults()
    platforms.feed(s)
    errors.extend(f"{lang}: {problem}" for problem in platforms.problems)
    if platforms.counts != platform_ref.counts:
        errors.append(f"{lang}: platform variant counts differ from template")
    if 'dl-win' in s or 'dl-mac' in s:
        errors.append(f"{lang}: obsolete download-only platform classes")
    for m in MARKS:
        if lang != "zh-CN" and s.count(m) != ref[m]:
            errors.append(f"{lang}: {m!r} {s.count(m)} != {ref[m]}")
    if "@@MAC_DMG_SIZE@@" in s and not args.allow_mac_size_placeholder:
        errors.append(f"{lang}: fill @@MAC_DMG_SIZE@@ with the measured release DMG size")
    if 'releases/latest/download/Mousetroke.dmg' not in s or 'macOS 11+' not in s:
        errors.append(f"{lang}: missing universal Mac download")
    # JSON-LD 与演示文案 JSON
    ld = None
    for name, pat in [("ld+json", r'<script type="application/ld\+json">(.*?)</script>'),
                      ("demo-text", r'<script type="application/json" id="demo-text">(.*?)</script>')]:
        m = re.search(pat, s, re.S)
        if not m:
            errors.append(f"{lang}: missing {name}")
            continue
        try:
            data = json.loads(m.group(1))
            if name == "ld+json":
                ld = data
        except Exception as e:
            errors.append(f"{lang}: {name} invalid JSON: {e}")
    # 自指元数据
    want = "https://mousetroke.pages.dev/" + (DIRS[lang] + "/" if DIRS[lang] else "")
    if f'<link rel="canonical" href="{want}">' not in s:
        errors.append(f"{lang}: canonical != {want}")
    if s.count('rel="alternate" hreflang=') != 11:
        errors.append(f"{lang}: hreflang count {s.count('rel=\"alternate\" hreflang=')}")
    if not re.search(rf'<html lang="{lang}"', s):
        errors.append(f"{lang}: html lang wrong")
    # 语言菜单：10 项、互不重复、每项链接指向自己的目录
    for block in re.findall(r'<div class="langdrop">(.*?)</div>', s, re.S):
        items = re.findall(r'<a(?: class="cur")? href="([^"]+)" hreflang="([^"]+)"', block)
        if len(items) != 10 or len({h for _, h in items}) != 10:
            errors.append(f"{lang}: language menu items {[h for _, h in items]}")
        for href, h in items:
            target = "index.html" if h == lang else (("../" if DIRS[lang] else "") + (DIRS[h] + "/" if DIRS[h] else "") + "index.html")
            if href != target:
                errors.append(f"{lang}: menu {h} -> {href} (want {target})")
    cur = re.findall(r'<a class="cur" href="[^"]+" hreflang="([^"]+)"', s)
    if cur != [lang, lang]:
        errors.append(f"{lang}: cur markers {cur}")
    # 非英文页：正文里不应残留模板英文句子（取英文页的几个标志句）
    if lang != "en":
        for probe in ["Draw this, get that", "Push into a corner", "More than shortcuts", "Barely there",
                      "Download, double-click, done", "Skip to content", "Not recognized", "Stays out of the way"]:
            if probe in s:
                errors.append(f"{lang}: English residue {probe!r}")
    # 下载区与页脚都必须能找到对应的更新日志。
    target = "changelog.html" if lang == "zh-CN" else "../en/changelog.html"
    if len(re.findall(rf'<a[^>]*href="{re.escape(target)}"', s)) != 2:
        errors.append(f"{lang}: expected download and footer changelog links")
    alternative_target = "alternatives/" if lang == "zh-CN" else "../en/alternatives/"
    if f'href="{alternative_target}"' not in s:
        errors.append(f"{lang}: missing alternatives footer link")
    # FAQ 的 JSON-LD 包含两个平台；每个可见分支都必须原样覆盖。
    visible_faq = faq_entities(s)
    schema_faq = next((item.get('mainEntity') for item in (ld or {}).get('@graph', [])
                       if item.get('@type') == 'FAQPage'), None)
    if len(visible_faq) != 8 or schema_faq != visible_faq:
        errors.append(f"{lang}: visible FAQ differs from JSON-LD")
    if not any('~/Library/Logs/mousetroke' in item['acceptedAnswer']['text'] for item in visible_faq):
        errors.append(f"{lang}: missing Mac uninstall/configuration paths")
try:
    for path, content in render_changelog().items():
        if not path.exists() or path.read_text(encoding="utf-8") != content:
            errors.append(f"{path.relative_to(ROOT)}: stale; run python site/gen_changelog.py")
        # Cloudflare Pages 会把 .html 308 到无后缀地址，对外 URL 一律不带后缀
        if f"<loc>https://mousetroke.pages.dev/{path.relative_to(ROOT).as_posix().removesuffix('.html')}</loc>" not in (ROOT / "sitemap.xml").read_text(encoding="utf-8"):
            errors.append(f"{path.name}: missing from sitemap")
except (ValueError, OSError) as error:
    errors.append(f"changelog: {error}")


class PageMetadata(HTMLParser):
    """收集实际属性，避免校验依赖属性顺序或 HTML 的缩进。"""
    def __init__(self, source):
        super().__init__(convert_charrefs=True)
        self.tags = []
        self.feed(source)

    def handle_starttag(self, tag, attrs):
        self.tags.append((tag, dict(attrs)))

    handle_startendtag = handle_starttag

    def attrs(self, tag, **match):
        return [attrs for name, attrs in self.tags if name == tag and all(attrs.get(k) == v for k, v in match.items())]


def public_url(path):
    rel = path.relative_to(ROOT).as_posix()
    return ORIGIN + "/" + (rel.removesuffix("index.html") if rel.endswith("index.html") else rel.removesuffix(".html"))


def resolve_url(url):
    """静态目录、实际文件及托管平台的无后缀 HTML 路径都要存在。"""
    path = ROOT / unquote(urlsplit(url).path).lstrip("/")
    if path.is_dir():
        path /= "index.html"
    elif not path.suffix and path.with_suffix(".html").is_file():
        path = path.with_suffix(".html")
    return path


# 禁用名称拆成字符码，保证校验器自身和字节码也不含目标串。
forbidden = bytes([119, 103, 101, 115, 116, 117, 114, 101, 115])
for path in ROOT.rglob("*"):
    if path.is_file() and forbidden in path.read_bytes().lower():
        errors.append(f"{path.relative_to(ROOT)}: forbidden product name")

html_pages = sorted(path for path in ROOT.rglob("*.html") if path.name != "alternatives_template.html")
metadata = {}
titles, descriptions = {}, {}
for path in html_pages:
    rel = path.relative_to(ROOT).as_posix()
    source = path.read_text(encoding="utf-8")
    parsed = PageMetadata(source)
    metadata[public_url(path)] = parsed
    title = re.search(r"<title>(.*?)</title>", source, re.S)
    description = parsed.attrs("meta", name="description")
    for label, value, seen in [("title", title[1].strip() if title else "", titles),
                               ("description", description[0].get("content", "") if len(description) == 1 else "", descriptions)]:
        if not value and path.name == "404.html" and label == "description":
            continue
        if not value:
            errors.append(f"{rel}: missing or repeated {label}")
        elif value in seen:
            errors.append(f"{rel}: duplicate {label} with {seen[value]}")
        seen[value] = rel
    if path.name != "404.html":
        if parsed.attrs("link", rel="canonical") != [{"rel": "canonical", "href": public_url(path)}]:
            errors.append(f"{rel}: canonical is not the public self URL")
    for raw in re.findall(r'<script\b[^>]*type="application/ld\+json"[^>]*>(.*?)</script>', source, re.S):
        try:
            json.loads(raw)
        except ValueError as error:
            errors.append(f"{rel}: invalid JSON-LD: {error}")
    for tag, attrs in parsed.tags:
        if tag == "img":
            if not attrs.get("alt", "").strip() or not all(attrs.get(key, "").isdigit() and int(attrs[key]) > 0 for key in ("width", "height")):
                errors.append(f"{rel}: image needs descriptive alt and positive width/height")
        for attr in (["href"] if tag in ("a", "link") else ["src"] if tag in ("img", "script") else []):
            href = attrs.get(attr, "")
            absolute = urljoin(public_url(path), href)
            parts = urlsplit(absolute)
            if parts.scheme not in ("http", "https") or parts.netloc != urlsplit(ORIGIN).netloc:
                continue
            target = resolve_url(absolute)
            if not target.is_file() or not target.resolve().is_relative_to(ROOT):
                errors.append(f"{rel}: unresolved internal {attr}: {href}")
            elif parts.fragment and target.suffix == ".html":
                # 404 的首页片段指向生成页，其余静态链接也要能定位。
                target_source = target.read_text(encoding="utf-8")
                if not re.search(r'\bid=[\"\']' + re.escape(unquote(parts.fragment)) + r'[\"\']', target_source):
                    errors.append(f"{rel}: missing fragment target: {href}")

try:
    generated = render_alternatives()
    for path, expected in generated.items():
        rel = path.relative_to(ROOT).as_posix()
        if not path.exists():
            errors.append(f"{rel}: stale; run python site/gen_alternatives.py")
            continue
        source = path.read_text(encoding="utf-8")
        if source != expected:
            errors.append(f"{rel}: stale; run python site/gen_alternatives.py")
        parsed = metadata[public_url(path)]
        lang = "en" if rel.startswith("en/") else "zh-CN"
        slug = path.parent.name if path.parent.name != "alternatives" else ""
        want = {key: ORIGIN + directory("en" if key == "x-default" else key, slug) for key in ("zh-CN", "en", "x-default")}
        alternates = parsed.attrs("link", rel="alternate")
        if len(alternates) != 3 or {a.get("hreflang"): a.get("href") for a in alternates} != want:
            errors.append(f"{rel}: hreflang pair or x-default incorrect")
        for key in ("zh-CN", "en"):
            other = metadata.get(want[key])
            if not other or not any(a.get("hreflang") == lang and a.get("href") == public_url(path) for a in other.attrs("link", rel="alternate")):
                errors.append(f"{rel}: non-reciprocal hreflang {key}")
        menus = re.findall(r'<div class="langdrop">(.*?)</div>', source, re.S)
        if len(menus) != 1 or len(PageMetadata(menus[0]).attrs("a")) != 2:
            errors.append(f"{rel}: expected a two-language menu")
        else:
            links = PageMetadata(menus[0]).attrs("a")
            if {a.get("hreflang"): urljoin(public_url(path), a.get("href", "")) for a in links} != {k: want[k] for k in ("zh-CN", "en")}:
                errors.append(f"{rel}: language switch does not target counterpart")
        schema = json.loads(re.search(r'<script type="application/ld\+json">(.*?)</script>', source, re.S)[1])
        graph = {item["@type"]: item for item in schema["@graph"]}
        visible = faq_entities(source)
        if len(visible) < 3 or graph.get("FAQPage", {}).get("mainEntity") != visible:
            errors.append(f"{rel}: visible FAQ differs from JSON-LD")
        webpage = graph.get("WebPage", {})
        if len(webpage.get("name", "")) > 60 or not 150 <= len(webpage.get("description", "")) <= 160:
            errors.append(f"{rel}: title exceeds 60 characters or description is outside 150–160")
        if webpage.get("dateModified") != CHECKED or webpage.get("inLanguage") != lang or webpage.get("url") != public_url(path) or not webpage.get("isPartOf"):
            errors.append(f"{rel}: incorrect WebPage metadata")
        if "BreadcrumbList" not in graph or "AggregateRating" in source or '"Review"' in source:
            errors.append(f"{rel}: missing breadcrumb or unsupported review schema")
        for key, expected_value in [("og:url", public_url(path)), ("og:title", webpage["name"]), ("og:description", webpage["description"]), ("og:image", ORIGIN + ("/og-en.png" if lang == "en" else "/og.png"))]:
            if [item.get("content") for item in parsed.attrs("meta", property=key)] != [expected_value]:
                errors.append(f"{rel}: incorrect {key}")
        defaults = PlatformDefaults()
        defaults.feed(source)
        errors.extend(f"{rel}: {problem}" for problem in defaults.problems)
        if defaults.counts != {"os-win": 4, "os-mac": 4}:
            errors.append(f"{rel}: expected hero and bottom download split buttons")
        if not parsed.attrs("time", datetime=CHECKED):
            errors.append(f"{rel}: missing visible checked date")
except (ValueError, OSError, KeyError, AttributeError) as error:
    errors.append(f"alternatives: {error}")

try:
    ns = {"s": "http://www.sitemaps.org/schemas/sitemap/0.9", "x": "http://www.w3.org/1999/xhtml"}
    tree = ET.parse(ROOT / "sitemap.xml")
    entries = tree.findall("s:url", ns)
    locations = [entry.findtext("s:loc", namespaces=ns) for entry in entries]
    expected_urls = {public_url(path) for path in html_pages if path.name != "404.html"}
    if len(set(locations)) != len(locations) or set(locations) != expected_urls:
        errors.append(f"sitemap: pages differ; missing={sorted(expected_urls - set(locations))}, extra={sorted(set(locations) - expected_urls)}")
    for entry in entries:
        url = entry.findtext("s:loc", namespaces=ns)
        if not resolve_url(url).is_file():
            errors.append(f"sitemap: nonexistent URL {url}")
        if "/alternatives/" in url:
            parsed = metadata.get(url)
            links = {a.get("hreflang"): a.get("href") for a in entry.findall("x:link", ns)}
            if entry.findtext("s:lastmod", namespaces=ns) != CHECKED or not parsed or links != {a["hreflang"]: a["href"] for a in parsed.attrs("link", rel="alternate")}:
                errors.append(f"sitemap: incorrect alternatives date or hreflang {url}")
except (ET.ParseError, OSError) as error:
    errors.append(f"sitemap: {error}")
print("\n".join(errors) if errors else f"OK: {len(PAGES)} home pages, {len(ALTERNATIVES) * 2 + 2} alternatives pages, changelogs, links, images and sitemap consistent")
sys.exit(1 if errors else 0)
