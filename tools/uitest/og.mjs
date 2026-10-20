// 生成社交分享图：node og.mjs <输出.png> <zh|en>（文案写在这里，避免 cmd 引号转义问题）
import puppeteer from "puppeteer-core";
const [out, lang = "zh"] = process.argv.slice(2);
const TEXT = {
  zh: ["按住右键，<br>画一笔。", "免费开源的 Windows 鼠标手势，常驻内存约 19 MB", "关闭标签页"],
  en: ["Hold right-click.<br>Draw a line.", "Free, open-source mouse gestures for Windows", "Close tab"],
};
const [title, sub, pillText] = TEXT[lang];
const browser = await puppeteer.launch({ executablePath: "C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", headless: true });
const page = await browser.newPage();
await page.setViewport({ width: 1200, height: 630 });
await page.setContent(`<!doctype html><html><head><link href="https://fonts.googleapis.com/css2?family=Schibsted+Grotesk:wght@600;800&display=swap" rel="stylesheet"><style>
body{margin:0;width:1200px;height:630px;background:#e9edf2;font-family:"Schibsted Grotesk","Microsoft YaHei UI",sans-serif;color:#10151f;position:relative;overflow:hidden;
background-image:radial-gradient(circle at 1px 1px,rgba(16,21,31,.13) 1px,transparent 0);background-size:26px 26px}
.b{position:absolute;left:80px;top:72px;display:flex;align-items:center;gap:16px;font-weight:800;font-size:34px}
h1{position:absolute;left:80px;top:170px;margin:0;font-size:96px;line-height:1.02;letter-spacing:-.035em;font-weight:800;max-width:720px}
p{position:absolute;left:80px;bottom:72px;margin:0;font-size:30px;color:#3a4352;font-weight:600}
svg.t{position:absolute;right:-10px;top:110px;width:440px;height:420px;overflow:visible}
.pill{position:absolute;right:150px;bottom:92px;background:#1c222c;color:#f4f7fb;font-size:30px;font-weight:600;padding:14px 30px;border-radius:999px}
</style></head><body>
<div class="b"><svg width="52" height="52" viewBox="0 0 32 32"><rect width="32" height="32" rx="8" fill="#0098d9"/><polyline points="7,11 13.5,21 19,13 25,21" fill="none" stroke="#fff" stroke-width="3.2" stroke-linecap="round" stroke-linejoin="round"/></svg>Mousetroke</div>
<h1>${title}</h1><p>${sub}</p>
<svg class="t" viewBox="0 0 440 420"><path d="M120 60 C 118 140, 122 220, 124 270 C 126 300, 150 306, 200 304 L 360 300" fill="none" stroke="#0098d9" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/><circle cx="120" cy="60" r="14" fill="#0098d9"/></svg>
<div class="pill">↓→ ${pillText}</div>
</body></html>`, { waitUntil: "networkidle0" });
await page.screenshot({ path: out });
await browser.close();
