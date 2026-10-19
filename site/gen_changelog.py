#!/usr/bin/env python3
"""用共享解析器生成中英文更新日志页；render 供站点检查比较生成结果。"""
from html import escape
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT.parent / "scripts"))
from changelog import load

ORIGIN = "https://mousetroke.pages.dev/"
GITHUB = "https://github.com/eiaao/mousetroke"


def render() -> dict[Path, str]:
    releases = load()
    template = (ROOT / "en/index.html").read_text(encoding="utf-8")
    brand = re.search(r'<a class="brand".*?</a>', template, re.S).group()
    pages = {}
    for chinese in (True, False):
        language = "中文" if chinese else "English"
        tag = "zh-CN" if chinese else "en"
        prefix = "" if chinese else "../"
        relative = "changelog.html" if chinese else "en/changelog.html"
        title = "更新日志" if chinese else "Changelog"
        description = "Mousetroke 各版本的新功能、改进和修复。" if chinese else "New features, improvements and fixes in each Mousetroke release."
        download = "下载" if chinese else "Download"
        release_link = "在 GitHub 查看此版本" if chinese else "View this release on GitHub"
        license_text = "MIT 许可证" if chinese else "MIT license"
        home = brand.replace('aria-label="Mousetroke home"', 'aria-label="Mousetroke 首页"') if chinese else brand
        articles = []
        for release in releases:
            bullets = "\n".join(f"        <li>{escape(bullet)}</li>" for bullet in release.sections[language])
            articles.append(f'''    <article class="changelog-release" aria-labelledby="v{release.version}">
      <h2 id="v{release.version}">v{release.version} <time datetime="{release.date}">{release.date}</time></h2>
      <ul>
{bullets}
      </ul>
      <a class="changelog-release-link" href="{GITHUB}/releases/tag/v{release.version}">{release_link} →</a>
    </article>''')
        body = "\n".join(articles)
        pages[ROOT / relative] = f'''<!doctype html>
<html lang="{tag}">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{title} · Mousetroke</title>
  <meta name="description" content="{description}">
  <link rel="canonical" href="{ORIGIN}{relative}">
  <link rel="alternate" hreflang="zh-CN" href="{ORIGIN}changelog.html">
  <link rel="alternate" hreflang="en" href="{ORIGIN}en/changelog.html">
  <link rel="icon" type="image/svg+xml" href="{prefix}assets/icon.svg">
  <link rel="stylesheet" href="{prefix}assets/site.css">
</head>
<body class="changelog-page">
<header class="top">
  <div class="wrap">
    {home}
    <a class="gh" href="{GITHUB}">GitHub</a>
    <a class="btn primary small" href="{GITHUB}/releases/latest/download/mousetroke.exe">{download}</a>
  </div>
</header>
<main class="wrap changelog-main">
  <h1>{title}</h1>
  <p class="changelog-intro">{description}</p>
{body}
</main>
<footer>
  <div class="wrap">
    <span>Mousetroke · {license_text}</span>
    <nav aria-label="{'页脚' if chinese else 'Footer'}">
      <a href="{prefix}changelog.html" hreflang="zh-CN" lang="zh-CN">简体中文</a>
      <a href="{prefix}en/changelog.html" hreflang="en" lang="en">English</a>
      <a href="{GITHUB}">GitHub</a>
    </nav>
  </div>
</footer>
</body>
</html>
'''
    return pages


def main():
    for path, content in render().items():
        path.write_text(content, encoding="utf-8")
        print(f"{path.relative_to(ROOT)}: written")


if __name__ == "__main__":
    main()
