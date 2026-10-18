// 十种语言及伪语言冒烟检查（docs/I18N-DESIGN.md §7.3/7.4）。
// 用法：node smoke.mjs <语言标签> [url]
// 结构性选择器（不按文字查找），所有语言共用同一套交互；
// en-US 断言正文无 CJK（语言自名除外）；qps-ploc 断言正文出现伪语言标记（未标记即漏翻）。
import puppeteer from "puppeteer-core";
import { mkdir, readFile } from "node:fs/promises";

const lang = process.argv[2] ?? "zh-CN";
const url = process.argv[3] ?? `http://localhost:8766/?lang=${lang}`;
const out = `../uishots/${lang}`;
await mkdir(out, { recursive: true });
const catalog = lang === "qps-ploc" ? null : JSON.parse(await readFile(`../../i18n/${lang}.json`, "utf8"));
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
const shot = async (n) => {
  // 检查控件自身的内容尺寸；滚动容器中尚未滚到的控件不算文字裁剪。
  const clipped = await page.evaluate(() => {
    const selectors = ".action-types button, .tab, .filter, .chip, .nav-item, .nav-label, .guide .btn, .guide-app, .toast-action, .toast-text, .segmented button, .app-list .muted, .keys-row .btn";
    return [...document.querySelectorAll(selectors)].flatMap((el) => {
      if (!el.getClientRects().length) return [];
      const style = getComputedStyle(el);
      if (el.matches(".nav-label")) {
        const tip = el.closest("[data-tip]")?.dataset.tip;
        if (style.whiteSpace !== "nowrap" || style.textOverflow !== "ellipsis" || style.overflowX !== "hidden") return ["sidebar name must use single-line ellipsis"];
        if (!tip?.includes(el.textContent)) return ["sidebar tooltip must contain full name"];
        // 侧栏有意省略的文本不算裁剪；仍检查垂直溢出。
        return el.scrollHeight > el.clientHeight + 1 ? ["sidebar name wraps vertically"] : [];
      }
      if (style.wordBreak === "break-all" || style.overflowWrap === "anywhere") return [`arbitrary word breaks: ${el.className || el.tagName}`];
      return el.scrollWidth > el.clientWidth + 1 || el.scrollHeight > el.clientHeight + 1 ? [el.className || el.tagName] : [];
    });
  });
  if (clipped.length) errors.push(`clipped controls in ${n}: ${clipped.join(", ")}`);
  const docLang = await page.$eval("html", (el) => el.lang);
  if (docLang !== (lang === "qps-ploc" ? "en" : lang)) errors.push(`incorrect document language in ${n}: ${docLang}`);
  if (lang === "zh-CN" || lang === "en-US") {
    const wrapped = await page.$$eval(".keys-row .btn", (els) => els.some((el) => el.getBoundingClientRect().height > 34));
    if (wrapped) errors.push(`short record button wrapped in ${n}`);
  }
  await page.screenshot({ path: `${out}/${n}.png` });
};
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
const EXCEPT = ["简体中文", "日本語"];

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

// 长译文会把动作区推到滚动容器下方，单独滚动检查完整动作网格。
await page.$eval(".action-types", (el) => el.scrollIntoView({ block: "start" }));
await sleep(200); await shot("12-actions");

// 覆盖触发按钮、附加鼠标步骤和列表摘要，独立按钮名仍用于菜单及筛选。
await page.$eval(".editor-body", (el) => { el.scrollTop = 0; });
if (catalog) {
  const labels = await page.$$eval(".pattern-editor .segmented button", (els) => els.map((e) => e.textContent));
  for (const [i, b] of ["right", "middle", "x1", "x2"].entries()) {
    if (labels[i] !== catalog[`ui.pattern.hold_btn.${b}`]) errors.push(`incorrect hold phrase: ${b}`);
  }
  const summary = await page.$eval(".row.on .row-pattern", (el) => el.textContent);
  if (!summary.startsWith(catalog["ui.pattern.hold_btn.right"])) errors.push("incorrect summary hold phrase");
}
for (const [i, b] of ["left", "middle", "x1", "x2"].entries()) {
  await click(".chip-add");
  await clickAll(".popover-row button", i);
  if (catalog) {
    const labels = await page.$$eval(".chips .chip:not(.chip-add):not(.chip-trace)", (els) => els.map((e) => e.firstChild.textContent));
    if (!labels.includes(catalog[`ui.pattern.click_btn.${b}`])) errors.push(`incorrect click phrase: ${b}`);
  }
}
await shot("13-click-steps");

// 菜单项内的紧凑动作网格也必须容纳多行标签。
await clickAll(".action-types button", 6);
await click(".menu-fields > .btn");
await page.$eval(".action-types.compact", (el) => el.scrollIntoView({ block: "start" }));
await sleep(200); await shot("14-menu-actions");

// 语言切换必须同步更新 html lang，让自动断字跟随实际显示的语言。
await page.keyboard.press("Escape");
await clickAll(".nav-bottom .nav-item", 0);
for (const next of ["de-DE", "ru-RU", "fr-FR", "en-US"]) {
  await page.select(".lang-select", next);
  const docLang = await page.$eval("html", (el) => el.lang);
  if (docLang !== next) errors.push(`document language did not change to ${next}: ${docLang}`);
}

// 长用户名称与无名称时的 exe 回退：保持一行，并通过现有气泡显示完整内容。
await page.setRequestInterception(true);
const longName = "Custom application " + "long-name-".repeat(12);
const longExe = "C:\\Apps\\" + "long-executable-name-".repeat(10) + ".exe";
page.on("request", async (request) => {
  if (/\/config(?:-en)?\.json$/.test(request.url())) {
    const config = JSON.parse(await readFile("config-en.json", "utf8"));
    const apps = config.scopes.filter((s) => s.kind === "app");
    apps[0].name = longName;
    delete apps[0].name_key;
    apps[1].name = "";
    delete apps[1].name_key;
    apps[1].app.exe = longExe;
    await request.respond({ contentType: "application/json", body: JSON.stringify(config) });
  } else await request.continue();
});
await page.goto(url, { waitUntil: "networkidle0" });
await page.keyboard.press("Escape");
await sleep(300);
for (const name of [longName, longExe.split("\\").pop()]) {
  const labels = await page.$$(".nav-label");
  let found = false;
  for (const label of labels) {
    if (await label.evaluate((el) => el.textContent) !== name) continue;
    found = true;
    const valid = await label.evaluate((el) => {
      const style = getComputedStyle(el);
      const range = document.createRange();
      range.selectNodeContents(el);
      const lines = new Set([...range.getClientRects()].map((r) => r.top));
      return style.whiteSpace === "nowrap" && style.textOverflow === "ellipsis" && el.scrollWidth > el.clientWidth && lines.size === 1;
    });
    if (!valid) errors.push("long sidebar name must stay on one line with ellipsis");
    await label.hover();
    await page.waitForSelector(".tip", { timeout: 3000 });
    const tip = await page.$eval(".tip", (el) => el.textContent);
    if (!tip.includes(name)) errors.push("long sidebar name missing from tooltip");
    if (name === longExe.split("\\").pop() && !tip.includes(longExe)) errors.push("executable path missing from tooltip");
    await page.mouse.move(400, 20);
    await sleep(150);
  }
  if (!found) errors.push("long sidebar name missing");
}

console.log(JSON.stringify({ lang, errors }, null, 1));
await browser.close();
if (errors.length) process.exitCode = 1;
