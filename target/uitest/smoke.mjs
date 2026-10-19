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
// 项目链接只能交给后端打开，设置界面自身不应请求外站。
page.on("request", (request) => {
  const target = new URL(request.url());
  if (/^https?:$/.test(target.protocol) && target.origin !== new URL(url).origin) errors.push(`external request: ${target.href}`);
});
page.on("pageerror", (e) => errors.push("pageerror: " + e.message));
page.on("console", (m) => m.type() === "error" && !m.text().includes("404") && errors.push("console: " + m.text()));
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const shot = async (n) => {
  // 检查控件自身的内容尺寸；滚动容器中尚未滚到的控件不算文字裁剪。
  const clipped = await page.evaluate(() => {
    const selectors = ".action-types button, .tab, .filter, .chip, .nav-item, .nav-label, .guide .btn, .guide-app, .toast-action, .toast-text, .segmented button, .app-list .muted, .keys-row .btn, .shape-instruction, .shape-samples-head, .shape-feedback p, .recorder-error, .about .btn, .about .pref-label, .about-star, .about-star p";
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
if (await page.$(".toast-close")) await click(".toast-close");

// 编辑面板 + 动作类型
await clickAll(".row", 2);
await sleep(300); await check("editor"); await shot("02-editor");
await page.keyboard.press("Escape"); await sleep(200);

// 形状在鼠标手势列表中显示曲线字形与完整画法描述。
const fixture = JSON.parse(await readFile(lang === "zh-CN" ? "config.json" : "config-en.json", "utf8"));
// 引导完成时会按勾选项添加禁用程序，以进入形状编辑前的配置作为基准。
const beforeShapeEdits = await page.evaluate(() => window.__saved) ?? fixture;
const shapeBinding = fixture.scopes[0].bindings.find((b) => b.pattern.some((s) => s.type === "shape"));
const shapePattern = shapeBinding.pattern;
const sample = shapePattern.find((s) => s.type === "shape").templates[0];
if (sample.length !== 128) errors.push("shape fixture must contain 64 encoded points");
if (await page.$$(".row .shape-glyph").then((els) => els.length) !== 1) errors.push("shape row must render a glyph");
const shapeDescription = await page.$eval(".row:has(.shape-glyph) .row-pattern", (el) => el.textContent);
if (catalog && !shapeDescription.includes(catalog["ui.pattern.draw_shape"])) errors.push("shape row is missing its description");
const gestureCount = await page.$eval(".tab .tab-count", (el) => Number(el.textContent));
if (gestureCount !== fixture.scopes[0].bindings.filter((b) => b.pattern[0].type === "press").length) errors.push("shape must count as a mouse gesture");
await clickAll(".filters .filter", 1);
if (!await page.$(".row .shape-glyph")) errors.push("shape must appear in its mouse button filter");
await clickAll(".filters .filter", 0);
await page.type(".search", shapeDescription); await sleep(150);
if (await page.$$(".row:has(.shape-glyph)").then((els) => els.length) !== 1) errors.push("search must find the shape description");
await page.focus(".search");
await page.keyboard.down("Control"); await page.keyboard.press("a"); await page.keyboard.up("Control");
await page.keyboard.press("Backspace"); await sleep(150);
await click(".row:has(.shape-glyph)");
await page.waitForSelector(".shape-sample .shape-glyph");
await check("shape-editor"); await shot("02-shape-editor");

// 修改触发方式再改回来，确保真实经过 decompose/compose，配置仍无损往返。
await clickAll(".pattern-editor .segmented button", 1);
await clickAll(".pattern-editor .segmented button", 0);
await sleep(500);
const roundTrip = await page.evaluate(() => window.__saved);
if (JSON.stringify(roundTrip) !== JSON.stringify(beforeShapeEdits)) {
  const diff = (a, b, path = "config") => {
    if (JSON.stringify(a) === JSON.stringify(b)) return [];
    if (!a || !b || typeof a !== "object" || typeof b !== "object") return [`${path}: ${JSON.stringify(a)} != ${JSON.stringify(b)}`];
    return [...new Set([...Object.keys(a), ...Object.keys(b)])].flatMap((k) => diff(a[k], b[k], `${path}.${k}`));
  };
  errors.push(`editing the trigger must round-trip the whole config exactly: ${diff(roundTrip, beforeShapeEdits).join("; ") || "property order changed"}`);
}

// 退化绘制必须显示本地化错误，不能追加或保存无效样本。
const drawSample = async (reverse = false) => {
  const rect = await page.$eval(".shape-recorder canvas", (el) => { const r = el.getBoundingClientRect(); return { x: r.x, y: r.y, w: r.width, h: r.height }; });
  const pts = Array.from({ length: sample.length / 2 }, (_, i) => [rect.x + rect.w / 2 + sample[i * 2] * 0.14, rect.y + rect.h / 2 + sample[i * 2 + 1] * 0.14]);
  if (reverse) pts.reverse();
  await page.mouse.move(...pts[0]); await page.mouse.down();
  for (const p of pts.slice(1)) await page.mouse.move(...p);
  await page.mouse.up(); await sleep(450);
};
await page.click(".shape-recorder canvas");
await page.waitForSelector(".recorder-error");
if (catalog && await page.$eval(".recorder-error", (el) => el.textContent) !== catalog["ui.shape.too_short"]) errors.push("shape error must be localized");
if (await page.$$(".shape-sample").then((els) => els.length) !== 1) errors.push("invalid drawing must not add a sample");
await drawSample();
await page.waitForSelector(".shape-consistency");
if (await page.$$(".shape-sample").then((els) => els.length) !== 2) errors.push("drawing must add a shape sample");
const savedWithSample = await page.evaluate(() => window.__saved.scopes[0].bindings.find((b) => b.name === "G").pattern);
if (savedWithSample[1]?.type !== "shape" || savedWithSample[1].templates.length !== 2 || JSON.stringify(savedWithSample[1].templates[0]) !== JSON.stringify(sample)) errors.push("saving must keep the shape step and original sample intact");
await check("shape-samples"); await shot("02-shape-samples");

// 相反方向的样本给出友好提示；最多 5 个，移除后可继续添加。
await drawSample(true);
if (!await page.$(".shape-advice")) errors.push("inconsistent samples must show a hint");
await drawSample(); await drawSample();
if (await page.$$(".shape-sample").then((els) => els.length) !== 5) errors.push("shape must allow five samples");
const callsAtLimit = await page.evaluate(() => window.__calls.filter(([cmd]) => cmd === "shape_sample").length);
await drawSample();
if (await page.evaluate(() => window.__calls.filter(([cmd]) => cmd === "shape_sample").length) !== callsAtLimit) errors.push("shape must stop recording at five samples");
for (let i = 0; i < 4; i++) await click(".shape-sample:last-child .chip-x");

// 同作用域中的近似绑定会点名警告；停用绑定不参与比较。
await click(".editor-foot .btn");
await page.waitForSelector(".shape-conflict");
const conflict = await page.$eval(".shape-conflict", (el) => el.textContent);
if (!conflict.includes("G")) errors.push("shape conflict must name the other gesture");
await page.$eval(".shape-conflict", (el) => el.scrollIntoView({ block: "nearest" }));
await check("shape-conflict"); await shot("02-shape-conflict");
await click(".editor-enable input");
await page.keyboard.press("Escape"); await sleep(150);
await click(".row:has(.shape-glyph):not(.off)");
await sleep(150);
if (await page.$(".shape-conflict")) errors.push("disabled shapes must not cause a conflict");

// 移除最后一个样本不能写盘；补录后才恢复自动保存。
await sleep(500);
const savesBeforeEmpty = await page.evaluate(() => window.__calls.filter(([cmd]) => cmd === "save_config").length);
await click(".shape-sample .chip-x"); await sleep(500);
if (await page.evaluate(() => window.__calls.filter(([cmd]) => cmd === "save_config").length) !== savesBeforeEmpty) errors.push("empty shape must not be saved");
await drawSample();
await page.waitForFunction((n) => window.__calls.filter(([cmd]) => cmd === "save_config").length > n, {}, savesBeforeEmpty);

// trace 和 shape 互相替换，始终只保留一个笔画步骤，撤销恢复原来的整数模板。
const beforeSwitch = await page.evaluate(() => window.__saved);
const savesBeforeSwitch = await page.evaluate(() => window.__calls.filter(([cmd]) => cmd === "save_config").length);
await click(".chip-add"); await click('[data-step="trace"]');
if (await page.$(".shape-recorder")) errors.push("switching to trace must replace shape");
await sleep(500);
if (await page.evaluate(() => window.__calls.filter(([cmd]) => cmd === "save_config").length) !== savesBeforeSwitch) errors.push("switching to an empty trace must not save");
await click(".chip-add"); await click('[data-step="shape"]');
if (await page.$$(".chip-trace").then((els) => els.length) !== 1) errors.push("switching stroke types must not append a second stroke");
await page.keyboard.press("Escape");
await page.evaluate(() => document.activeElement?.blur());
await page.keyboard.down("Control"); await page.keyboard.press("z"); await page.keyboard.press("z"); await page.keyboard.up("Control");
await sleep(500);
if (JSON.stringify(await page.evaluate(() => window.__saved)) !== JSON.stringify(beforeSwitch)) errors.push("undoing stroke switches must restore the exact config");
await click(".row:has(.shape-glyph):not(.off)");
await page.waitForSelector(".shape-recorder");
await page.keyboard.press("Escape"); await sleep(150);

// 触发角标签页
await clickAll(".tab", 1); await sleep(200); await shot("03-edges");

// 偏好设置（语言下拉）、导入导出、关于
const navCount = await page.evaluate(() => document.querySelectorAll(".nav-group:not(.nav-bottom) .nav-item").length);
await clickAll(".nav-bottom .nav-item", 0); await check("prefs"); await shot("04-prefs");
await clickAll(".nav-bottom .nav-item", 1); await check("data"); await shot("05-data");
await clickAll(".nav-bottom .nav-item", 2); await check("about"); await shot("06-about");

// 项目组位于文件位置之前；每个按钮只传白名单名称，Star 复用 GitHub。
await page.waitForSelector('.about-project[role="group"]');
if (await page.$$(".about-project .pref-row").then((els) => els.length) !== 3) errors.push("about must show three project rows");
if (catalog) {
  const headings = await page.$$eval(".about .pref-group h2", (els) => els.map((el) => el.textContent));
  if (headings[0] !== catalog["ui.about.project_group"] || headings[1] !== catalog["ui.about.files_group"]) errors.push("project group must precede files");
  const projectText = await page.$eval(".about-project", (el) => el.textContent);
  for (const key of ["homepage", "homepage_address", "source", "source_address", "feedback", "feedback_help", "star_prompt", "star_button"]) {
    if (!projectText.includes(catalog[`ui.about.${key}`])) errors.push(`missing project text: ${key}`);
  }
}
if (!await page.$(".about-star .btn.primary svg[aria-hidden]")) errors.push("star button must have a decorative SVG icon");
const callsBeforeLinks = await page.evaluate(() => window.__calls.length);
for (const selector of [".about-project .pref-row:nth-of-type(1) .btn", ".about-project .pref-row:nth-of-type(2) .btn", ".about-project .pref-row:nth-of-type(3) .btn", ".about-star .btn"]) {
  await page.click(selector);
}
const linkCalls = await page.evaluate((n) => window.__calls.slice(n).filter(([cmd]) => cmd === "open_path"), callsBeforeLinks);
const expectedLinks = ["homepage", "github", "issues", "github"].map((what) => ["open_path", { what }]);
if (JSON.stringify(linkCalls) !== JSON.stringify(expectedLinks)) errors.push(`incorrect about links: ${JSON.stringify(linkCalls)}`);

// 最小窗口验证长译文：内容不横向溢出，按钮始终完整显示且不在词中换行。
await page.setViewport({ width: 900, height: 580 });
await page.$eval(".about-project", (el) => el.scrollIntoView({ block: "start" }));
await sleep(150);
const aboutLayout = await page.evaluate(() => {
  const about = document.querySelector(".about");
  const fits = about.scrollWidth <= about.clientWidth + 1;
  const buttonsFit = [...about.querySelectorAll(".btn")].every((el) => {
    const style = getComputedStyle(el), rect = el.getBoundingClientRect(), bounds = about.getBoundingClientRect();
    return style.whiteSpace === "nowrap" && rect.left >= bounds.left && rect.right <= bounds.right;
  });
  return fits && buttonsFit;
});
if (!aboutLayout) errors.push("about layout must fit the minimum window without wrapping buttons");
await check("about-min"); await shot("06-about-min");
await page.setViewport({ width: 1180, height: 760 });

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
