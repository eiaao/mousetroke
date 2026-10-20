// 分节截图：node site-sec.mjs <url> <输出目录> [dark] [宽]
import puppeteer from "puppeteer-core";
const [url = "http://localhost:8790/", out = "../siteshots", dark = "", w = "1440"] = process.argv.slice(2);
const browser = await puppeteer.launch({ executablePath: "C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", headless: true });
const page = await browser.newPage();
await page.setViewport({ width: +w, height: 900 });
if (dark) await page.emulateMediaFeatures([{ name: "prefers-color-scheme", value: "dark" }]);
await page.goto(url, { waitUntil: "networkidle0" });
await new Promise((r) => setTimeout(r, 2600));
for (const id of ["gestures", "edges", "actions", "quiet", "light", "download"]) {
  const el = await page.$("#" + id);
  await page.evaluate((id) => document.getElementById(id).scrollIntoView({ behavior: "instant", block: "center" }), id);
  await new Promise((r) => setTimeout(r, 2200)); // 等出场动画结束
  await el.screenshot({ path: `${out}/s-${id}${dark ? "-dark" : ""}.png` });
}
await browser.close();
