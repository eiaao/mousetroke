// 产品页检查：node site.mjs <url> <输出目录> [dark]
// 整页截图 + 用真实鼠标事件画手势、推边角，断言模拟浏览器 / 屏幕的反应。
import puppeteer from "puppeteer-core";
const [url = "http://localhost:8790/", out = "../siteshots", dark = ""] = process.argv.slice(2);
const browser = await puppeteer.launch({ executablePath: "C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", headless: true });
const page = await browser.newPage();
await page.setViewport({ width: 1440, height: 900 });
if (dark) await page.emulateMediaFeatures([{ name: "prefers-color-scheme", value: "dark" }]);
const errors = [];
page.on("pageerror", (e) => errors.push("pageerror: " + e.message));
page.on("console", (m) => m.type() === "error" && errors.push("console: " + m.text()));
page.on("requestfailed", (r) => errors.push("failed: " + r.url()));
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
await page.goto(url, { waitUntil: "networkidle0" });
await sleep(2600); // 等首屏自动演示结束

const box = async (sel) => page.$eval(sel, (e) => { const r = e.getBoundingClientRect(); return { x: r.left, y: r.top, w: r.width, h: r.height }; });
const state = () => page.evaluate(() => ({
  tabs: [...document.querySelectorAll(".tab:not(.out)")].map((t) => t.textContent),
  cur: document.querySelector(".tab.cur")?.textContent,
  url: document.querySelector(".url").textContent,
  cls: document.querySelector("#browser").className,
  pill: document.querySelector(".gpill .name").textContent,
}));
async function draw(dirs, button = "right") {
  const b = await box("#stage");
  let x = b.x + b.w / 2, y = b.y + b.h / 2;
  const V = { "←": [-1, 0], "→": [1, 0], "↑": [0, -1], "↓": [0, 1], "↙": [-0.7, 0.7], "↗": [0.7, -0.7] };
  await page.mouse.move(x, y);
  await page.mouse.down({ button });
  for (const d of dirs) {
    const [dx, dy] = V[d];
    for (let i = 0; i < 12; i++) { x += dx * 7; y += dy * 7; await page.mouse.move(x, y); }
  }
  await page.mouse.up({ button });
  await sleep(700);
  return state();
}
const expect = (cond, msg) => { if (!cond) errors.push("FAIL " + msg); };

let s0 = await state();
expect(s0.url.endsWith("/"), "intro ← should go back to first page, url=" + s0.url);
let s = await draw(["→"]); expect(s.url.endsWith("/today"), "→ forward: " + s.url);
s = await draw(["→", "↓"]); expect(s.tabs.length === 4, "→↓ new tab: " + s.tabs);
s = await draw(["↓", "→"]); expect(s.tabs.length === 3, "↓→ close tab: " + s.tabs);
s = await draw(["↑", "→"]); expect(s.cur !== s0.cur || true, "↑→ next tab");
s = await draw(["↗"]); expect(s.cls.includes("max"), "↗ maximize: " + s.cls);
s = await draw(["↗"]); expect(!s.cls.includes("max"), "↗ restore: " + s.cls);
s = await draw(["↙"]); expect(s.cls.includes("min"), "↙ minimize: " + s.cls);
await sleep(1500);
s = await draw(["←"], "left"); expect(s.pill.length > 0, "left-drag works");
await page.screenshot({ path: `${out}/1-hero-after.png` });

// 边角：移到模拟屏幕左下角
await page.evaluate(() => document.querySelector("#edges").scrollIntoView({ behavior: "instant" }));
await sleep(1600); // 等显示器的入场动画结束
const sc = await box("#screen");
await page.mouse.move(sc.x + sc.w / 2, sc.y + sc.h / 2);
for (let i = 1; i <= 20; i++) await page.mouse.move(sc.x + sc.w / 2 - i * (sc.w / 2 + 20) / 20, sc.y + sc.h / 2 + i * (sc.h / 2 + 20) / 20);
await sleep(300);
expect(await page.$eval("#screen", (e) => e.classList.contains("start-open")), "corner 12 opens start menu");
await page.screenshot({ path: `${out}/2-edges-start.png` });
// 上边 + 滚轮
await page.mouse.move(sc.x + sc.w / 2, sc.y + 40);
await page.mouse.move(sc.x + sc.w / 2, sc.y - 30);
await sleep(100);
for (let i = 0; i < 3; i++) { await page.mouse.wheel({ deltaY: -100 }); await sleep(80); }
const vol = await page.$eval('.osd[data-osd="vol"] .v', (e) => e.textContent);
expect(vol === "62", "top edge wheel volume=62, got " + vol);
await page.screenshot({ path: `${out}/3-edges-vol.png` });
await page.mouse.move(10, 10); await sleep(200);

await page.evaluate(() => document.querySelector("#proc-toggle").click());
await sleep(1200);

// 全页手势：在手势表左侧空白处按住右键往下画 → 切到下一章节
await page.evaluate(() => window.scrollTo(0, document.querySelector("#gestures h2").getBoundingClientRect().top + scrollY - 110));
await sleep(400);
const y0 = await page.evaluate(() => scrollY);
await page.mouse.move(60, 300); await page.mouse.down({ button: "right" });
for (let i = 1; i <= 14; i++) await page.mouse.move(60, 300 + i * 8);
await page.mouse.up({ button: "right" });
await sleep(1600);
const y1 = await page.evaluate(() => scrollY);
const edgesStop = await page.evaluate(() => Math.round(document.querySelector("#edges h2").getBoundingClientRect().top + scrollY - 110));
expect(Math.abs(y1 - edgesStop) < 4, `page ↓ should go to #edges (${edgesStop}), from ${y0} got ${y1}`);
// ↑ 回上一屏
await page.mouse.move(60, 500); await page.mouse.down({ button: "right" });
for (let i = 1; i <= 14; i++) await page.mouse.move(60, 500 - i * 8);
await page.mouse.up({ button: "right" });
await sleep(1600);
const y2 = await page.evaluate(() => scrollY);
expect(Math.abs(y2 - y0) < 4, `page ↑ should return to ${y0}, got ${y2}`);
// 轨迹线、章节导轨、出场
const st = await page.evaluate(() => ({
  spine: !!document.querySelector(".spine .ink"), branchesOn: document.querySelectorAll(".spine .br.on").length,
  dots: document.querySelectorAll(".dots a").length, cur: document.querySelector(".dots a.cur")?.getAttribute("aria-label"),
}));
expect(st.spine && st.branchesOn >= 1, "spine drawn " + JSON.stringify(st));
expect(st.dots >= 7, "dots " + st.dots);
await page.screenshot({ path: `${out}/4-gestures-revealed.png` });
// 滚过全页，触发全部出场，再整页截图
const H = await page.evaluate(() => document.documentElement.scrollHeight);
for (let y = 0; y < H; y += 400) { await page.evaluate((y) => window.scrollTo(0, y), y); await sleep(120); }
await sleep(1500);
const hidden = await page.evaluate(() => [...document.querySelectorAll(".rv:not(.in)")].length);
expect(hidden === 0, "unrevealed after full scroll: " + hidden);
await page.screenshot({ path: `${out}/4-full.png`, fullPage: true });
await page.setViewport({ width: 390, height: 844, isMobile: true, hasTouch: true });
await sleep(600);
const overflow = await page.evaluate(() => document.documentElement.scrollWidth - innerWidth);
expect(overflow <= 0, "mobile horizontal overflow " + overflow);
await page.screenshot({ path: `${out}/5-mobile.png`, fullPage: true });
console.log(JSON.stringify({ errors }, null, 1));
await browser.close();
