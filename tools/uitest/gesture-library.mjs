// 手势库界面冒烟：无头 Edge + 模拟后端（mock.js）。用法: node gesture-library.mjs <url> <输出目录>
// 流程：空库引导 → 整理并导入 → 手势库 → 在其他程序中使用 → 共享编辑与「只改这里」→ 选择器 → 删除确认。
import puppeteer from "puppeteer-core";
const [url, out] = process.argv.slice(2);
const browser = await puppeteer.launch({ executablePath: "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe", headless: true, args: ["--lang=zh-CN"] });
const page = await browser.newPage();
await page.setViewport({ width: 1280, height: 800, deviceScaleFactor: 1 });
const errors = [];
page.on("pageerror", (e) => errors.push("pageerror: " + e.message));
page.on("console", (m) => m.type() === "error" && errors.push("console: " + m.text()));
const shot = (n) => page.screenshot({ path: `${out}\\gl-${n}.png` });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const clickText = async (sel, text, exact = false) => {
  const ok = await page.evaluate((sel, text, exact) => {
    const el = [...document.querySelectorAll(sel)].find((e) => (exact ? e.textContent.trim() === text : e.textContent.trim().includes(text)));
    if (!el) return false;
    el.click();
    return true;
  }, sel, text, exact);
  if (!ok) errors.push(`未找到 ${sel} "${text}"`);
  await sleep(300);
};
const text = (sel) => page.$eval(sel, (e) => e.textContent).catch(() => null);
const check = (cond, msg) => cond || errors.push(msg);
/** 已保存配置的手势库不变式：引用存在且画法一致。 */
const savedOk = () =>
  page.evaluate(() => {
    const c = window.__saved;
    if (!c) return "未保存";
    const lib = new Map((c.gestures ?? []).map((g) => [g.id, JSON.stringify(g.pattern)]));
    for (const s of c.scopes) for (const b of s.bindings) if (b.gesture && lib.get(b.gesture) !== JSON.stringify(b.pattern)) return `引用不一致：${b.name}`;
    return "ok";
  });

await page.goto(url, { waitUntil: "networkidle0" });
await sleep(300);
if (await page.$(".guide")) {
  await page.keyboard.press("Escape");
  await sleep(300);
}

// ---- 空库引导
check(await text(".nav-item .nav-new"), "老用户的「手势」入口应显示新标记");
await clickText(".nav-item .nav-label", "手势", true);
await shot("01-empty");
check(!(await page.$(".nav-item .nav-new")), "进入手势库后新标记应消失");

// ---- 整理并导入
await clickText(".library-empty .btn.primary", "整理并导入手势");
await shot("02-wizard-source");
await clickText(".modal .btn.primary", "下一步");
await sleep(300);
await shot("03-wizard-review");
check(await text(".import-summary"), "检查结果应显示预览数字");
await clickText(".modal .btn.primary", "导入");
await sleep(600);
await shot("04-library");
const cards = await page.$$(".gesture-grid .gesture-card");
check(cards.length > 0, "导入后手势库应有手势");
check((await savedOk()) === "ok", `导入后保存：${await savedOk()}`);

// ---- 手势详情（选一个被多处使用的）
const sharedIdx = await page.evaluate(() => [...document.querySelectorAll(".gesture-grid .gesture-card")].findIndex((c) => /^[2-9]/.test(c.querySelector(".gesture-card-uses")?.textContent ?? "")));
check(sharedIdx >= 0, "应有被多处使用的手势");
await (await page.$$(".gesture-grid .gesture-card > button"))[Math.max(0, sharedIdx)].click();
await sleep(300);
await shot("05-detail");

// ---- 在其他程序中使用：选一个还没用它的程序 → 动作选择器 → 选第一个动作
await clickText(".editor .btn", "在其他程序中使用");
await shot("06-assign-scope");
const picked = await page.evaluate(() => {
  const b = [...document.querySelectorAll(".assign-list li > button")].find((x) => !x.disabled && !x.querySelector(".assign-hint") && !x.textContent.includes("添加程序"));
  b?.click();
  return !!b;
});
check(picked, "应能选择一个程序");
await sleep(400);
await shot("07-assign-action");
await page.evaluate(() => document.querySelector(".act-pick .act-row:not(:disabled), .act-pick li button")?.click());
await sleep(400);
await page.keyboard.press("Enter");
await sleep(500);
await shot("08-assigned");
check((await savedOk()) === "ok", `分配后保存：${await savedOk()}`);

// ---- 从用法表跳到使用它的绑定 → 共享编辑
await page.evaluate(() => document.querySelector(".gesture-uses li > button")?.click());
await sleep(400);
await shot("09-editor-shared");
check(await page.$(".gesture-ref"), "引用库手势的绑定应显示手势卡片");
check(await page.$(".share-tag"), "共享手势应显示同步范围");
await clickText(".pattern-editor .segmented button", "按住中键");
await sleep(300);
await shot("10-editor-modified");
check(await page.$(".share-bar"), "修改共享手势后应显示提示条");
await clickText(".share-bar .btn", "只改这里");
await sleep(400);
await shot("11-only-here");
check(await page.$(".gesture-inline"), "只改这里后应变为本程序专用");
check((await savedOk()) === "ok", `只改这里后保存：${await savedOk()}`);

// ---- 选择器
await clickText(".gesture-inline .btn", "从手势库选择");
await shot("12-picker");
check(await page.$(".gesture-picker .gesture-card"), "选择器应列出手势");
await page.keyboard.press("Escape");
await sleep(300);

// ---- 删除确认
await clickText(".nav-item .nav-label", "手势", true);
await (await page.$$(".gesture-grid .gesture-card > button"))[0].click();
await sleep(300);
await clickText(".editor-foot .btn.danger", "删除手势");
await shot("13-delete");
await page.keyboard.press("Escape");

// ---- 深色模式
await page.emulateMediaFeatures([{ name: "prefers-color-scheme", value: "dark" }]);
await sleep(300);
await shot("14-dark");

console.log(errors.length ? "ERRORS:\n" + errors.join("\n") : "NO ERRORS");
await browser.close();
