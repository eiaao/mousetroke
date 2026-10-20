#!/usr/bin/env python3
"""从 site/en/index.html 生成各语言页面。

每条替换是 (原文行, 译文行)，按整行精确匹配（保留缩进由译文自带），
重复行（如 data-on/data-off）一次性全换。生成后运行 site-check.py 校验结构。
用法：python gen.py es fr …（缺省生成全部已定义语言）
"""
import sys, pathlib, re, json, html

ROOT = pathlib.Path(__file__).resolve().parent.parent
TPL = (ROOT / "site/en/index.html").read_text(encoding="utf-8")

# 每语言的机械替换 + 文本替换
def base(d, lang_tag, og_locale, summary_label, self_name):
    """机械部分（canonical/og/url/lang 等），返回 list[(old,new)]。"""
    return [
        ('<html lang="en">', f'<html lang="{lang_tag}">'),
        ('<link rel="canonical" href="https://mousetroke.pages.dev/en/">',
         f'<link rel="canonical" href="https://mousetroke.pages.dev/{d}/">'),
        ('<meta property="og:locale" content="en_US">', f'<meta property="og:locale" content="{og_locale}">'),
        ('<meta property="og:url" content="https://mousetroke.pages.dev/en/">',
         f'<meta property="og:url" content="https://mousetroke.pages.dev/{d}/">'),
        ('<summary aria-label="Language">English ▾</summary>',
         f'<summary aria-label="{summary_label}">{self_name} ▾</summary>'),
        # 模板里 English 是当前项：改回普通链接；本语言那一项改成当前项
        ('<a class="cur" href="index.html" hreflang="en" lang="en">English</a>',
         '<a href="../en/index.html" hreflang="en" lang="en">English</a>'),
        (f'<a href="../{d}/index.html" hreflang="{lang_tag}" lang="{lang_tag}">{self_name}</a>',
         f'<a class="cur" href="index.html" hreflang="{lang_tag}" lang="{lang_tag}">{self_name}</a>'),
        ('"url": "https://mousetroke.pages.dev/en/",', f'"url": "https://mousetroke.pages.dev/{d}/",'),
        ('"@id": "https://mousetroke.pages.dev/en/#site",', f'"@id": "https://mousetroke.pages.dev/{d}/#site",'),
        ('"inLanguage": "en"', f'"inLanguage": "{lang_tag.split("-")[0]}"'),
    ]

LANGS = {}

# Mac 下载与首次打开说明共用译文；FAQ 统一从可见正文生成 JSON-LD。
MAC_EN = {'download': 'Download Mac version (beta)',
 'first_title': 'First open on Mac',
 'first_text': 'Drag Mousetroke into Applications and open it there. This beta is not notarized: if macOS blocks it, '
               'go to System Settings → Privacy & Security → Open Anyway (on older macOS, right-click → Open). Then '
               'grant Accessibility and Input Monitoring.'}

def mac_replacements(text):
    pairs = []
    for key, tail in [("download", " <small>"), ("first_title", "</h3>"), ("first_text", "</p>")]:
        pairs.append((">" + html.escape(MAC_EN[key], quote=False) + tail, ">" + html.escape(text[key], quote=False) + tail))
    return pairs


def faq_entities(page):
    """FAQ 正文含两平台的完整回答；JSON-LD 不按访客系统隐藏内容。"""
    def plain(text):
        # 相邻的平台 span 在 JSON-LD 中以空格分隔，code/kbd 保留原文。
        text = text.replace('</span><span', '</span> <span')
        return html.unescape(re.sub(r'<[^>]+>', '', text))
    return [
        {"@type": "Question", "name": plain(question),
         "acceptedAnswer": {"@type": "Answer", "text": plain(answer)}}
        for question, answer in re.findall(
            r'<details(?: open)?><summary>(.*?)</summary><div class="ans"><p>(.*?)</p></div></details>', page)
    ]


def sync_faq(page):
    entities = faq_entities(page)
    if not entities:
        raise ValueError('Missing visible FAQ')
    body = ',\n'.join('        ' + json.dumps(item, ensure_ascii=False) for item in entities)
    return re.sub(r'("mainEntity": \[\n).*?(\n      \])',
                  lambda m: m[1] + body + m[2], page, count=1, flags=re.S)


# 文件大小的小数点与单位（未列出的语言写 "8.1 MB"）
SIZE_LOCALE = {"de": (",", "MB"), "fr": (",", "Mo"), "it": (",", "MB"), "pt": (",", "MB"), "ru": (",", "МБ")}

# 下载组合按钮右侧 ▾ 的无障碍名称
OTHER_PLATFORMS = {"de": "Andere Plattformen", "es": "Otras plataformas", "fr": "Autres plateformes",
                   "it": "Altre piattaforme", "ja": "他のプラットフォーム", "ko": "다른 플랫폼",
                   "pt": "Outras plataformas", "ru": "Другие платформы"}

# 各语言在 site/langs/<tag>.py（REPL = [(old, new), …]，base 由这里注入）
import pathlib as _p
for _f in sorted((_p.Path(__file__).parent / "langs").glob("*.py")):
    _ns = {"base": base}
    exec(compile(_f.read_text(encoding="utf-8"), str(_f), "exec"), _ns)
    LANGS[_f.stem] = _ns["REPL"] + mac_replacements(_ns["MAC"]) + [
        ('aria-label="Other platforms"', f'aria-label="{OTHER_PLATFORMS[_f.stem]}"')]

def gen(tag, repls):
    out = TPL
    missed = []
    for old, new in repls:
        if old not in out:
            missed.append(old[:80])
            continue
        out = out.replace(old, new)
    # Mac 下载按钮的大小与 Windows 一样按语言写小数点和单位
    if tag in SIZE_LOCALE:
        sep, unit = SIZE_LOCALE[tag]
        out = re.sub(r'<small>(\d+)\.(\d) MB', lambda m: f'<small>{m[1]}{sep}{m[2]} {unit}', out)
    out = sync_faq(out)
    d = ROOT / "site" / tag
    d.mkdir(exist_ok=True)
    (d / "index.html").write_text(out, encoding="utf-8")
    print(f"{tag}: written, {len(missed)} missed")
    for m in missed:
        print("  MISS:", m)

if __name__ == "__main__":
    # 中文手写、英文模板的 FAQ 也以正文为准，避免两处改动遗漏。
    for path in [ROOT / 'site/index.html', ROOT / 'site/en/index.html']:
        path.write_text(sync_faq(path.read_text(encoding='utf-8')), encoding='utf-8')
    TPL = (ROOT / 'site/en/index.html').read_text(encoding='utf-8')
    tags = sys.argv[1:] or list(LANGS)
    for tag in tags:
        gen(tag, LANGS[tag])
    from gen_changelog import main as gen_changelog
    gen_changelog()
