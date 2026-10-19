// 双语更新日志：file:// 下的桌面/手机截图、资源错误与横向溢出检查。
// 用法：在 target/uitest 运行 node changelog.mjs。
import puppeteer from "puppeteer-core";
import { mkdir } from "node:fs/promises";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

const out = resolve("../changelogshots");
await mkdir(out, { recursive: true });
const browser = await puppeteer.launch({
  executablePath: "C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe",
  headless: true,
  userDataDir: resolve("../uibrowsers/changelog"),
});
const errors = [];
try {
  const page = await browser.newPage();
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("console", (message) => message.type() === "error" && errors.push(message.text()));
  page.on("requestfailed", (request) => errors.push(`failed: ${request.url()}`));
  for (const [lang, directory] of [["zh", ""], ["en", "en/"]]) {
    for (const width of [1280, 390]) {
      await page.setViewport({ width, height: 900, deviceScaleFactor: 1 });
      await page.goto(pathToFileURL(resolve(`../../site/${directory}index.html`)).href, { waitUntil: "networkidle0" });
      const baseline = await page.evaluate(() => document.documentElement.scrollWidth - innerWidth);
      await page.goto(pathToFileURL(resolve(`../../site/${directory}changelog.html`)).href, { waitUntil: "networkidle0" });
      const layout = await page.evaluate(() => ({
        overflow: document.documentElement.scrollWidth - innerWidth,
        outside: [...document.querySelectorAll("header a, h1, h2, li, footer a")].some((element) => {
          const rect = element.getBoundingClientRect();
          return rect.left < -1 || rect.right > innerWidth + 1;
        }),
        versions: document.querySelectorAll(".changelog-release").length,
      }));
      if (layout.overflow > Math.max(1, baseline) || layout.outside) errors.push(`${lang} ${width}: horizontal overflow ${JSON.stringify(layout)}`);
      if (layout.versions !== 2) errors.push(`${lang} ${width}: expected both releases`);
      await page.screenshot({ path: `${out}/${lang}-${width}.png`, fullPage: true });
      console.log(`${lang} ${width}: overflow=${layout.overflow}px, main page=${baseline}px`);
    }
  }
} finally {
  await browser.close();
}
console.log(errors.length ? errors.join("\n") : `OK: changelog screenshots in ${out}`);
process.exitCode = errors.length ? 1 : 0;
