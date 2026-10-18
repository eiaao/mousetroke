import puppeteer from "puppeteer-core";
const browser = await puppeteer.launch({ executablePath: "C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", headless: true });
const page = await browser.newPage();
await page.setViewport({ width: 390, height: 844 });
await page.goto("http://localhost:8791/en/index.html", { waitUntil: "networkidle0" });
await new Promise((r) => setTimeout(r, 400));
const r = await page.evaluate(() => {
  const out = [];
  for (const el of document.querySelectorAll("body *")) {
    const b = el.getBoundingClientRect();
    if (b.right > innerWidth + 1 && b.width > 0) out.push(`${el.tagName.toLowerCase()}.${el.className} right=${Math.round(b.right)} w=${Math.round(b.width)}`);
  }
  return out.slice(0, 12);
});
console.log(r.join("\n"));
await browser.close();
