import puppeteer from "puppeteer-core";
import { readFileSync } from "node:fs";
const browser = await puppeteer.launch({ executablePath: "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe", headless: true });
const page = await browser.newPage();
await page.goto(process.argv[2], { waitUntil: "networkidle0" });
await new Promise((r) => setTimeout(r, 1700));
await page.evaluate(readFileSync(process.argv[3], "utf8"));
console.log(await page.title());
await browser.close();
