// 截图：node shot.mjs <url> <输出.png> [宽] [高] [dark] [脚本]
import puppeteer from "puppeteer-core";
import { readFileSync } from "node:fs";
const [url, out, w = "1180", h = "760", dark = "light", script = ""] = process.argv.slice(2);
const browser = await puppeteer.launch({ executablePath: "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe", headless: true, args: ["--lang=zh-CN"] });
const page = await browser.newPage();
await page.setViewport({ width: +w, height: +h, deviceScaleFactor: 1 });
if (dark === "dark") await page.emulateMediaFeatures([{ name: "prefers-color-scheme", value: "dark" }]);
const errors = [];
page.on("pageerror", (e) => errors.push(e.message));
await page.goto(url, { waitUntil: "networkidle0" });
await new Promise((r) => setTimeout(r, 400));
if (script) { await page.evaluate(`(async () => { ${script.endsWith(".js") ? readFileSync(script, "utf8") : script} })()`); await new Promise((r) => setTimeout(r, 400)); }
await page.screenshot({ path: out });
if (errors.length) console.log("ERR", errors.join("\n"));
await browser.close();
