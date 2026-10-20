// 动效抽帧：node site-frames.mjs <url> <输出目录>
// 首屏入场按时间抽帧；再在章节切换（全页手势 ↓）过程中抽帧。
import puppeteer from "puppeteer-core";
const [url = "http://localhost:8790/", out = "../siteshots/frames"] = process.argv.slice(2);
const browser = await puppeteer.launch({ executablePath: "C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", headless: true });
const page = await browser.newPage();
await page.setViewport({ width: 1440, height: 900 });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
await page.goto(url, { waitUntil: "domcontentloaded" });
const t0 = Date.now();
for (const t of [250, 600, 1000, 1600, 2600, 3200]) { await sleep(Math.max(0, t - (Date.now() - t0))); await page.screenshot({ path: `${out}/hero-${t}.png` }); }
await page.evaluate(() => window.scrollTo(0, document.querySelector("#actions h2").getBoundingClientRect().top + scrollY - 700));
await sleep(300);
await page.mouse.move(60, 300); await page.mouse.down({ button: "right" });
for (let i = 1; i <= 14; i++) await page.mouse.move(60, 300 + i * 8);
await page.screenshot({ path: `${out}/sw-0-drawing.png` });
await page.mouse.up({ button: "right" });
for (const t of [250, 600, 1100, 1800]) { await sleep(t === 250 ? 250 : t - [250, 600, 1100, 1800][[250, 600, 1100, 1800].indexOf(t) - 1]); await page.screenshot({ path: `${out}/sw-${t}.png` }); }
await browser.close();
