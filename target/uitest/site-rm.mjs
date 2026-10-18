// 减少动画模式：所有内容应立即可见，没有自动演示
import puppeteer from "puppeteer-core";
const [url = "http://localhost:8790/"] = process.argv.slice(2);
const browser = await puppeteer.launch({ executablePath: "C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", headless: true });
const page = await browser.newPage();
await page.setViewport({ width: 1440, height: 900 });
await page.emulateMediaFeatures([{ name: "prefers-reduced-motion", value: "reduce" }]);
const errors = [];
page.on("pageerror", (e) => errors.push(e.message));
await page.goto(url, { waitUntil: "networkidle0" });
await new Promise((r) => setTimeout(r, 300));
const r = await page.evaluate(() => {
  const vis = (el) => +getComputedStyle(el).opacity;
  return {
    hiddenRv: [...document.querySelectorAll(".rv")].filter((e) => vis(e) < 1).length,
    stage: vis(document.querySelector(".stage")),
    h2: getComputedStyle(document.querySelector("#faq h2 .ln > span")).transform,
    url: document.querySelector(".url").textContent,
  };
});
if (r.hiddenRv || r.stage < 1 || r.h2 !== "none") errors.push("reduced motion still hides: " + JSON.stringify(r));
if (!r.url.endsWith("/today")) errors.push("auto demo ran under reduced motion: " + r.url);
console.log(JSON.stringify({ errors }));
await browser.close();
