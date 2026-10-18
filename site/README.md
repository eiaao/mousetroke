# Mousetroke 产品页

纯静态站点，没有构建步骤，可以直接部署到任何静态托管（GitHub Pages、Cloudflare Pages、Nginx 等）。

```
index.html          中文页（默认）
en/index.html       英文页（其余 8 种语言页面的模板）
es fr de ja pt ru it ko/index.html   由 gen.py 从英文页生成，不要手改
gen.py langs/*.py   生成器与各语言替换表（原文行 → 译文行）
check.py            校验 10 个页面：结构与英文页一致、JSON 合法、canonical/hreflang/语言菜单自指
assets/site.css     两页共用的样式
assets/site.js      两页共用的交互（文案都在 HTML 里，脚本只管行为）
assets/icon.svg     站点图标
og.png / og-en.png  社交分享图（target/uitest/og.mjs 生成）
llms.txt            给 AI 搜索 / Agent 读的纯文本摘要
robots.txt sitemap.xml
download/           发布时把 dist/mousetroke.exe 放进来
```

## 上线前要改的

1. **域名**：所有绝对地址都写的是占位域名 `https://mousetroke.example`，替换成真实域名：
   `grep -rl mousetroke.example . | xargs sed -i 's#https://mousetroke.example#https://你的域名#g'`
2. **下载文件**：把 `dist/mousetroke.exe` 复制到 `download/mousetroke.exe`。版本号、大小（12.8 MB）写在两页的 CTA、JSON-LD 和 `llms.txt` 里，发版时一起改。
3. 页面内链接都是相对路径，部署在根目录或子目录、或直接双击打开 HTML 都能正常显示；但 canonical / hreflang / og 等元信息用的是绝对地址，部署地址变了要一起改。

## 改内容时注意

- 中文页手写；英文页是其余 8 种语言的模板。改英文页后跑 `python gen.py`，有 `MISS` 说明对应语言的替换表要补译文，再跑 `python check.py`。
- 改中文页时同步英文页；FAQ 同时出现在页面正文和 `<script type="application/ld+json">` 里，两处都要改（替换表里也是两处）。
- 语言菜单（页头 `details.langmenu`）和页脚语言列表在 10 个页面里都有，新增语言时 hreflang 块、`sitemap.xml`、两个列表都要加。
- 手势表每行的 `data-p` 是手势折线编码（与引擎配置相同），`data-act` 决定首屏演示里模拟浏览器的反应；带 `data-mod`/`data-btn` 的行只展示、不参与演示识别。
- 产品数据（内存、大小、默认手势、边角规则）要和真实程序一致，改了程序行为记得回来改这里。

## 本地预览与检查

```
cd site && python3 -m http.server 8790
cd target/uitest && node site.mjs http://localhost:8790/ ../siteshots      # 中文：真实鼠标画手势、推边角、手机宽度无横向滚动
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
