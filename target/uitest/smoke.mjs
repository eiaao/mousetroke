// 三语言冒烟检查（docs/I18N-DESIGN.md §7.3/7.4）：zh-CN / en-US / qps-ploc。
// 用法：node smoke.mjs <zh-CN|en-US|qps-ploc> [url]
// 结构性选择器（不按文字查找），三种语言共用同一套交互；
// en-US 断言正文无 CJK（语言自名除外）；qps-ploc 断言正文出现伪语言标记（未标记即漏翻）。
import puppeteer from "puppeteer-core";

const lang = process.argv[2] ?? "zh-CN";
const url = process.argv[3] ?? `http://localhost:8765/?lang=${lang}`;
const out = `../uishots/${lang}`;
const browser = await puppeteer.launch({
  executablePath: "C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe",
  headless: true,
  args: [`--lang=${lang}`],
});
const page = await browser.newPage();
await page.setViewport({ width: 1180, height: 760, deviceScaleFactor: 1 });
const errors = [];
page.on("pageerror", (e) => errors.push("pageerror: " + e.message));
page.on("console", (m) => m.type() === "error" && !m.text().includes("404") && errors.push("console: " + m.text()));
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const shot = (n) => page.screenshot({ path: `${out}/${n}.png` });
const click = async (sel) => {
  const ok = await page.evaluate((sel) => {
    const el = document.querySelector(sel);
    if (!el) return false;
    el.click();
    return true;
  }, sel);
  if (!ok) errors.push(`not found ${sel}`);
  await sleep(250);
};
const clickAll = async (sel, i) => {
  const ok = await page.evaluate((sel, i) => {
    const el = document.querySelectorAll(sel)[i];
    if (!el) return false;
    el.click();
    return true;
  }, sel, i);
  if (!ok) errors.push(`not found ${sel}[${i}]`);
  await sleep(250);
};

const hasCJK = (s) => /[一-鿿　-〿＀-￯]/.test(s);
// 语言选项按设计用自身语言显示（简体中文永远是简体中文）
const EXCEPT = ["简体中文"];

const check = async (where) => {
  const text = await page.evaluate(() => document.body.innerText);
  const lines = text.split("\n");
  if (lang === "en-US") {
    const bad = lines.filter((l) => hasCJK(l) && !EXCEPT.some((x) => l.includes(x)));
    if (bad.length) errors.push(`CJK in ${where}: ${bad.slice(0, 4).join(" | ")}`);
  } else if (lang === "qps-ploc") {
    // 走词条的文本都带 [!! … !!] 标记；用户数据（手势名等）除外
    // 不走词条的：品牌名、用户数据（测试配置里的程序名）、键帽、路径、纯数字/符号
    const NOT_I18N = ["Mousetroke", "Google Chrome", "Custom lua test", "main.rs - mousetroke", "chrome.exe", "explorer.exe", "blender.exe", "mstsc.exe"];
    const plain = lines.filter((l) => {
      const x = l.trim();
      if (!x || hasCJK(x) || x.includes("[!!")) return false;
      if (NOT_I18N.some((n) => x === n || x.startsWith(n + " "))) return false;
      if (/^[A-Z][A-Za-z0-9]{0,5}$/.test(x)) return false; // 键帽 Ctrl / Win / ESC / F5
      if (/[\\/]/.test(x) || /\.exe$/i.test(x) || /^[0-9 .,%×–—+:-]+$/.test(x) || x.length <= 2) return false; // 路径、纯数字
      if (/^[0-9.]+ ?(ms|px)$/.test(x)) return false; // 通用单位
      if (/^(Ctrl|Alt|Shift|Win)\+/.test(x)) return false; // 快捷键
      return true;
    });
    if (plain.length) errors.push(`untranslated (no pseudo marker) in ${where}: ${plain.slice(0, 4).join(" | ")}`);
  }
};

await page.goto(url, { waitUntil: "networkidle0" });
await sleep(400);

// 引导：四步（结构点击，语言无关）
await sleep(1500); await shot("00-guide-1");
for (let i = 1; i <= 3; i++) { await click(".guide .btn.primary"); await sleep(1500); await shot(`00-guide-${i + 1}`); }
await click(".guide .btn.primary"); // 开始使用
await sleep(300);
await check("list"); await shot("01-list");

// 编辑面板 + 动作类型
await clickAll(".row", 2);
await sleep(300); await check("editor"); await shot("02-editor");
await page.keyboard.press("Escape"); await sleep(200);

// 触发角标签页
await clickAll(".tab", 1); await sleep(200); await shot("03-edges");

// 偏好设置（语言下拉）、导入导出、关于
const navCount = await page.evaluate(() => document.querySelectorAll(".nav-group:not(.nav-bottom) .nav-item").length);
await clickAll(".nav-bottom .nav-item", 0); await check("prefs"); await shot("04-prefs");
await clickAll(".nav-bottom .nav-item", 1); await check("data"); await shot("05-data");
await clickAll(".nav-bottom .nav-item", 2); await check("about"); await shot("06-about");

// 添加程序对话框（建议清单 + 禁用模式）
await clickAll(".nav-group:not(.nav-bottom) .nav-item", navCount - 1); // 添加程序…
await sleep(300);
await clickAll(".picker-mode button", 1); // 在它里面禁用手势
await sleep(300); await check("picker"); await shot("07-picker");
await page.keyboard.press("Escape"); await sleep(200);

// 各作用域（通用 / 桌面 / 应用）
await clickAll(".nav-group:not(.nav-bottom) .nav-item", 0); await sleep(150); await shot("08-global");
await clickAll(".nav-group:not(.nav-bottom) .nav-item", 1); await sleep(150); await shot("09-desktop");
await clickAll(".nav-group:not(.nav-bottom) .nav-item", 2); await sleep(150); await shot("10-app");

// 搜索按显示名匹配：内置手势在配置里存的是导入时的语言（"复制"），en-US 下显示并能搜到 "Copy"
await clickAll(".nav-group:not(.nav-bottom) .nav-item", 0);
await clickAll(".tab", 0);
const firstName = await page.$eval(".row .row-name", (e) => e.textContent.trim());
const term = firstName.replace(/^\[!! /, "").split(/\s+/)[0].slice(0, 4).toLowerCase();
await page.type(".search", term);
await sleep(250);
const hits = await page.$$eval(".row .row-name", (els) => els.map((e) => e.textContent.trim()));
if (!hits.includes(firstName)) errors.push(`search "${term}" should find "${firstName}", got: ${hits.slice(0, 4).join(" | ")}`);
await page.$eval(".search", (e) => { e.value = ""; e.dispatchEvent(new Event("input", { bubbles: true })); });
await page.click(".search", { clickCount: 3 }); await page.keyboard.press("Backspace"); await sleep(150);

// 窄窗口（编辑面板变抽屉）
await page.setViewport({ width: 980, height: 700 });
await clickAll(".nav-group:not(.nav-bottom) .nav-item", 0); // 所有程序
await clickAll(".tab", 0); // 鼠标手势
await clickAll(".row", 0); await sleep(300); await check("narrow"); await shot("11-narrow");

console.log(JSON.stringify({ lang, errors }, null, 1));
await browser.close();
