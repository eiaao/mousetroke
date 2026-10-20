#!/usr/bin/env python3
"""从 site/en/index.html 生成各语言页面。

每条替换是 (原文行, 译文行)，按整行精确匹配（保留缩进由译文自带），
重复行（如 data-on/data-off）一次性全换。生成后运行 site-check.py 校验结构。
用法：python gen.py es fr …（缺省生成全部已定义语言）
"""
import sys, pathlib, re, json

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

# Mac 新增片段的正文与 FAQ JSON-LD 必须使用同一份译文。
MAC_EN = {'windows': 'Download Windows version',
 'download': 'Download Mac version (beta)',
 'first_title': 'First open on Mac',
 'first_text': 'Drag Mousetroke into Applications and open it there. This beta is not notarized: if macOS blocks it, '
               'go to System Settings → Privacy & Security → Open Anyway (on older macOS, right-click → Open). Then '
               'grant Accessibility and Input Monitoring.',
 'faq_title': 'How do I uninstall the Mac version?',
 'faq_text': 'Quit from the menu bar, delete Mousetroke.app, ~/Library/Application Support/mousetroke and '
             '~/Library/Logs/mousetroke, and remove the Mousetroke login item in System Settings.'}

def mac_replacements(text):
    import html
    pairs = []
    for key, tail in [("download", " <small>"), ("first_title", "</h3>"), ("first_text", "</p>"), ("faq_title", "</summary>"), ("faq_text", "</p>")]:
        pairs.append((">" + html.escape(MAC_EN[key], quote=False) + tail, ">" + html.escape(text[key], quote=False) + tail))
    for key in ["faq_title", "faq_text"]:
        pairs.append((json.dumps(MAC_EN[key], ensure_ascii=False), json.dumps(text[key], ensure_ascii=False)))
    return pairs


# 各语言在 site/langs/<tag>.py（REPL = [(old, new), …]，base 由这里注入）
import pathlib as _p
for _f in sorted((_p.Path(__file__).parent / "langs").glob("*.py")):
    _ns = {"base": base}
    exec(compile(_f.read_text(encoding="utf-8"), str(_f), "exec"), _ns)
    LANGS[_f.stem] = _ns["REPL"] + mac_replacements(_ns["MAC"])

def gen(tag, repls):
    out = TPL
    missed = []
    for old, new in repls:
        if old not in out:
            missed.append(old[:80])
            continue
        out = out.replace(old, new)
        # 免费说明的结构化数据与可见回答保持同一译文。
        if '<summary>Is Mousetroke free?</summary>' in old:
            old_faq = re.search(r'<summary>(.*?)</summary>.*?<p>(.*?)</p>', old)
            new_faq = re.search(r'<summary>(.*?)</summary>.*?<p>(.*?)</p>', new)
            for source, translated in zip(old_faq.groups(), new_faq.groups()):
                out = out.replace(json.dumps(source, ensure_ascii=False), json.dumps(translated, ensure_ascii=False))
    d = ROOT / "site" / tag
    d.mkdir(exist_ok=True)
    (d / "index.html").write_text(out, encoding="utf-8")
    print(f"{tag}: written, {len(missed)} missed")
    for m in missed:
        print("  MISS:", m)

if __name__ == "__main__":
    tags = sys.argv[1:] or list(LANGS)
    for tag in tags:
        gen(tag, LANGS[tag])
    from gen_changelog import main as gen_changelog
    gen_changelog()
