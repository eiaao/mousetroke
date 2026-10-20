import puppeteer from "puppeteer-core";
const dirs = ["", "en/", "es/", "fr/", "de/", "ja/", "pt/", "ru/", "it/", "ko/"];
const browser = await puppeteer.launch({ executablePath: "C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", headless: true });
const out = [];
for (const d of dirs) {
  for (const [w, h] of [[1280, 800], [390, 844]]) {
    const page = await browser.newPage();
    await page.setViewport({ width: w, height: h });
    const errs = [];
    page.on("pageerror", (e) => errs.push(e.message));
    page.on("console", (m) => m.type() === "error" && !m.text().includes("404") && errs.push(m.text()));
    await page.goto(`http://localhost:8790/${d}index.html`, { waitUntil: "networkidle0" });
    await new Promise((r) => setTimeout(r, 400));
    const over = await page.evaluate(() => document.documentElement.scrollWidth - innerWidth);
    let menu = "-";
    if (w === 1280) {
      await page.click(".langmenu summary");
      await new Promise((r) => setTimeout(r, 150));
      menu = await page.evaluate(() => {
        const dd = document.querySelector(".langdrop");
        const r = dd.getBoundingClientRect();
        return `${document.querySelectorAll(".langdrop a").length} items, visible=${r.height > 0}, inViewport=${r.right <= innerWidth && r.left >= 0}`;
      });
      await page.screenshot({ path: `../siteshots/lang-${d.replace("/", "") || "zh"}.png` });
    }
    out.push(`${(d || "zh/").padEnd(4)} ${w}px overflow=${over} errors=${errs.length}${errs.length ? " " + errs[0] : ""} menu=${menu}`);
    await page.close();
  }
}
// 跳转：从日语页菜单点俄语
const p = await browser.newPage();
await p.goto("http://localhost:8790/ja/index.html", { waitUntil: "networkidle0" });
await p.click(".langmenu summary");
await Promise.all([p.waitForNavigation(), p.click('.langdrop a[hreflang="ru"]')]);
out.push("nav ja→ru: " + p.url() + " lang=" + (await p.evaluate(() => document.documentElement.lang)));
console.log(out.join("\n"));
await browser.close();
