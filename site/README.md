# Mousetroke 产品页

纯静态站点，没有构建步骤，可以直接部署到任何静态托管（GitHub Pages、Cloudflare Pages、Nginx 等）。

```
index.html          中文页（默认）
en/index.html       英文页（其余 8 种语言页面的模板）
changelog.html / en/changelog.html  由 gen_changelog.py 从根目录 CHANGELOG.md 生成
es fr de ja pt ru it ko/index.html   由 gen.py 从英文页生成，不要手改
404.html            中英双语错误页，兼容主站与镜像的路径
gen.py langs/*.py   生成器与各语言替换表（原文行 → 译文行）
check.py            校验 10 个页面：结构与英文页一致、JSON 合法、canonical/hreflang/语言菜单自指
_worker.js          Cloudflare 入口：访问首页 / 时按 mt_lang cookie（语言菜单点选写入）或浏览器语言 302 到对应语言版本；GitHub Pages 部署时删除
assets/site.css     10 种语言共用的样式
assets/site.js      共用交互与系统检测（文案都在 HTML 里，脚本只管行为）
assets/icon.svg     站点图标
og.png / og-en.png  社交分享图（tools/uitest/og.mjs 生成）
llms.txt            给 AI 搜索 / Agent 读的纯文本摘要
robots.txt sitemap.xml
```

## 部署

- **Cloudflare Pages 是主站**：<https://mousetroke.pages.dev/>。推送到 `main` 且改动了 `site/` 时，
  `.github/workflows/cloudflare.yml` 自动发布；修改工作流本身也会触发，也可手动运行。不需要构建步骤。
  在仓库 Settings → Secrets and variables → Actions 中设置 `CLOUDFLARE_API_TOKEN`
  （权限 Account → Cloudflare Pages → Edit）和 `CLOUDFLARE_ACCOUNT_ID`。
- **GitHub Pages 是镜像**：<https://eiaao.github.io/mousetroke/>。`.github/workflows/pages.yml`
  继续在 `main` 的 `site/` 或该工作流有改动时自动发布；镜像的 canonical 等绝对地址统一指向 Cloudflare 主站。
- Windows 下载指向 `releases/latest/download/mousetroke.exe`，Mac 下载指向 `releases/latest/download/Mousetroke.dmg`（均在 GitHub 的 `eiaao/mousetroke` 仓库），永远是最新版本。版本号、大小写在下载按钮、资源说明、JSON-LD 和 `llms.txt` 里；发布者在发版时统一核对。
- 换主站域名时，在仓库根目录用 Bash（如 Git Bash）执行一次递归替换，覆盖 `site/` 下的
  canonical / hreflang / og / JSON-LD / sitemap / robots / llms，以及 `site/gen.py`、
  `site/langs/*.py` 替换表的两侧和 `site/check.py`；同时同步根目录的产品主页链接：

  ```bash
  grep -rlZF 'https://mousetroke.pages.dev' site/ README.md README.zh-CN.md Cargo.toml | xargs -0 sed -i 's#https://mousetroke.pages.dev#https://新域名#g'
  python site/gen.py
  python site/check.py
  ```

  替换时域名末尾不带 `/`，保留原有路径；每种语言必须显示 `0 missed`，校验必须通过。
  新域名需先在托管平台配置好，GitHub Pages 镜像地址保留不变。
- 产品页面内链接都是相对路径，部署在根目录或子目录、或直接双击打开 HTML 都能正常显示。
  `404.html` 使用内联最小样式，并按请求路径是否以 `/mousetroke/` 开头设置首页和图标的基准路径，兼容两个站点。

## 改内容时注意

- 中文页手写；英文页是其余 8 种语言的模板。改英文页后跑 `python gen.py`，有 `MISS` 说明对应语言的替换表要补译文，再跑 `python check.py`。`gen.py` 也会生成双语更新日志；只改根目录 `CHANGELOG.md` 时可单独跑 `python site/gen_changelog.py`。发布正文与网站共用 `scripts/changelog.py`，`check.py` 会检查日志页是否过期。
- 改中文页时同步英文页。FAQ 正文是唯一文案来源：`gen.py` 会从中文、英文和生成页的可见 FAQ 同步 JSON-LD，合并两个平台的回答；替换表只翻译正文。`check.py` 会核对全部 8 个 FAQ，包含免费说明和合并后的卸载说明。
- 语言菜单（页头 `details.langmenu`）和页脚语言列表在 10 个页面里都有，新增语言时 hreflang 块、`sitemap.xml`、两个列表都要加。
- 手势表每行的 `data-p` 是手势折线编码（与引擎配置相同），`data-act` 决定首屏演示里模拟浏览器的反应；带 `data-mod`/`data-btn` 的行只展示、不参与演示识别。
- 产品数据（内存、大小、默认手势、边角规则）要和真实程序一致，改了程序行为记得回来改这里。

## 平台文案与下载

共同事实使用简短的双平台文案；SEO、SoftwareApplication 和 FAQ JSON-LD 都覆盖 Windows 和 Mac，不按访客切换。不同事实使用 `os-win` / `os-mac`，行内写法：

```html
<span class="os-win">Windows 文案</span><span class="os-mac" hidden>Mac 文案</span>
```

也可直接给段落、列表项、下载链接或整块插图加类。Windows 分支默认不带 `hidden`，Mac 分支必须带 `hidden`；CSS 的全局 `[hidden] { display: none !important; }` 保证 flex/grid 不会使隐藏分支重新显示。`site.js` 沿用 `isMac` 检测（排除报告 MacIntel 的 iPad），统一切换这些元素。脚本不可用时显示 Windows 文案，爬虫仍可读取两平台的 HTML 与 JSON-LD。

下载组合按钮的主按钮显示当前系统，▾ 菜单反向显示其他系统。菜单内两种链接都不预设 `hidden`，无脚本时仍可下载两个版本。修改分支后同步中文、英文和 `langs/*.py`，运行生成器及检查器；检查器还会核对分支数量和无脚本默认状态。

Windows 资源示意图保留实测 WebView2 数据，只在 Windows 显示；Mac 显示活动监视器物理内存约 13 MB 的说明。首屏浏览器、任务栏、边角动画与 `demo-text` 是示意演示，保留 Windows 风格；旁边的默认边角列表和产品说明按平台显示。

## Mac 测试版发布前

下载区同时提供 Windows exe 与 `releases/latest/download/Mousetroke.dmg`。Mac 最低 11，通用包包含 Apple silicon / Intel；首次打开的共用译文在 `langs/*.py` 的 `MAC` 中维护，其他平台文案在 `REPL` 中维护。卸载已合并为一个 FAQ，JSON-LD 自动包含两平台的完整回答。

`@@MAC_DMG_SIZE@@` 是尚未测量的 DMG 大小，不能估计。审查期间运行 `python site/check.py --allow-mac-size-placeholder`；发布者测量最终签名 DMG 后，在中文页、英文模板和 `gen.py`（如有大小引用）填入实际大小，再运行 `python site/gen.py`、`python site/check.py`。默认检查必须通过后才能部署站点，部署工作流不允许占位符。

## 本地预览与检查

```
cd site && python3 -m http.server 8790
cd tools/uitest && node site.mjs http://localhost:8790/ ../siteshots      # 中文：真实鼠标画手势、推边角、手机宽度无横向滚动
node site.mjs http://localhost:8790/en/ ../siteshots/en                   # 英文
node site-sec.mjs http://localhost:8790/ ../siteshots [dark]              # 分节截图（等出场动画结束）
node site-frames.mjs http://localhost:8790/ ../siteshots/frames            # 首屏入场、章节切换过程抽帧
node site-rm.mjs http://localhost:8790/                                    # 减少动画模式：内容全部直接可见
```

## 动效约定

- 所有动效只在 `<html class="js">` 下生效（head 里一行脚本加上），脚本没跑时页面完整可读；`prefers-reduced-motion` 时全部关闭。
- 出场：脚本给元素挂 `.rv`（可加 `from-l` / `from-r` / `zoom`），进入视野加 `.in`；同组元素用 `--i` 错开。
- 全页手势（在页面任意处按住右键画 ↓ ↑ ←↑ ←↓ 切换章节）只在 Windows 上启用：其他系统右键菜单在按下时就弹出，没法和手势共存。
- 会做描线动画的 SVG 路径（`pathLength="1"` + dash）必须一笔画完，不能用 `M` 另起子路径，否则动画在子路径之间断开。
