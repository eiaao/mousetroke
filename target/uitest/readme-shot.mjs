// README 用的设置界面截图（内置默认配置）：node readme-shot.mjs <lang> <输出.png>
import puppeteer from "puppeteer-core";
const [lang = "en-US", out = "shot.png"] = process.argv.slice(2);
const browser = await puppeteer.launch({ executablePath: "C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", headless: true, args: [`--lang=${lang}`] });
const page = await browser.newPage();
await page.setViewport({ width: 1180, height: 760, deviceScaleFactor: 2 });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const errors = [];
page.on("pageerror", (e) => errors.push(e.message));
await page.goto(`http://localhost:8765/?lang=${lang}&cfg=default.json`, { waitUntil: "networkidle0" });
await sleep(800);
// 跳过使用引导
for (let i = 0; i < 6 && (await page.$(".guide")); i++) { await page.evaluate(() => document.querySelector(".guide .btn.primary")?.click()); await sleep(500); }
// 打开"后退 / Back"这条（列表里含 ← 的第一行）
await page.evaluate(() => [...document.querySelectorAll(".row")].find((r) => r.textContent.includes("←"))?.click() ?? document.querySelector(".row")?.click());
await sleep(900);
// 关掉引导结束时的"已关闭手势 / 撤销"提示条（禁用的程序仍留在侧栏里，正好展示这个功能）
await page.evaluate(() => document.querySelector(".toast-close")?.click());
await sleep(500);
await page.mouse.move(5, 5);
await sleep(300);
await page.screenshot({ path: out });
if (errors.length) console.log("ERR", errors);
await browser.close();
