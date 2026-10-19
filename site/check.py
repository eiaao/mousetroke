#!/usr/bin/env python3
"""校验 10 个语言页面：结构与英文模板一致、JSON 合法、元数据自指、无英文残留。"""
import json, re, sys, pathlib
from gen_changelog import render as render_changelog
ROOT = pathlib.Path(__file__).resolve().parent
PAGES = {"zh-CN": "index.html", "en": "en/index.html", "es": "es/index.html", "fr": "fr/index.html",
         "de": "de/index.html", "ja": "ja/index.html", "pt-BR": "pt/index.html", "ru": "ru/index.html",
         "it": "it/index.html", "ko": "ko/index.html"}
DIRS = {"zh-CN": "", "en": "en", "es": "es", "fr": "fr", "de": "de", "ja": "ja", "pt-BR": "pt", "ru": "ru", "it": "it", "ko": "ko"}
# 结构指纹：这些属性/标签的出现次数必须与英文页一致
MARKS = ['data-p="', 'data-act="', 'class="row', '<details', 'class="rule', 'data-edge="', 'class="sw"', '<kbd>', '<dt>', 'id="h-']
errors = []
tpl = (ROOT / "en/index.html").read_text(encoding="utf-8")
ref = {m: tpl.count(m) for m in MARKS}
for lang, rel in PAGES.items():
    s = (ROOT / rel).read_text(encoding="utf-8")
    for m in MARKS:
        if lang != "zh-CN" and s.count(m) != ref[m]:
            errors.append(f"{lang}: {m!r} {s.count(m)} != {ref[m]}")
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
    faq = re.search(r'<summary>[^<]*</summary><div class="ans"><p>([^<]*GitHub[^<]*)</p>', s)
    # 免费说明不能在 JSON-LD 中保留旧的隐私承诺。
    if faq and ld is not None and json.dumps(faq.group(1), ensure_ascii=False) not in json.dumps(ld, ensure_ascii=False):
        errors.append(f"{lang}: visible privacy FAQ differs from JSON-LD")
try:
    for path, content in render_changelog().items():
        if not path.exists() or path.read_text(encoding="utf-8") != content:
            errors.append(f"{path.relative_to(ROOT)}: stale; run python site/gen_changelog.py")
        if f"<loc>https://mousetroke.pages.dev/{path.relative_to(ROOT).as_posix()}</loc>" not in (ROOT / "sitemap.xml").read_text(encoding="utf-8"):
            errors.append(f"{path.name}: missing from sitemap")
except (ValueError, OSError) as error:
    errors.append(f"changelog: {error}")
print("\n".join(errors) if errors else f"OK: {len(PAGES)} pages consistent")
sys.exit(1 if errors else 0)
