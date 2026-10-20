// 设置界面交互冒烟：无头 Edge + 模拟后端（mock.js）。用法: node ui.mjs <url> <输出目录>
import puppeteer from "puppeteer-core";
const [url, out] = process.argv.slice(2);
const browser = await puppeteer.launch({
  executablePath: "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe",
  headless: true,
  args: ["--lang=zh-CN"],
});
const page = await browser.newPage();
await page.setViewport({ width: 1180, height: 760, deviceScaleFactor: 1 });
const errors = [];
page.on("pageerror", (e) => errors.push("pageerror: " + e.message));
page.on("console", (m) => m.type() === "error" && errors.push("console: " + m.text()));
const shot = (n) => page.screenshot({ path: `${out}\\${n}.png` });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const clickText = async (sel, text) => {
  const ok = await page.evaluate((sel, text) => {
    const el = [...document.querySelectorAll(sel)].find((e) => e.textContent.trim().includes(text));
    if (!el) return false;
    el.click();
    return true;
  }, sel, text);
  if (!ok) errors.push(`未找到 ${sel} "${text}"`);
  await sleep(250);
};

await page.goto(url, { waitUntil: "networkidle0" });
await sleep(300);
// ---- 首次打开：使用引导
if (!(await page.$(".guide"))) errors.push("首次打开应显示使用引导");
for (let i = 1; i <= 4; i++) {
  await sleep(1700); // 让插图动画走到中段
  await shot(`00-guide-${i}`);
  if (i < 4) await clickText(".guide .btn.primary", "下一步");
}
await page.keyboard.press("Escape");
await sleep(300);
if (await page.$(".guide")) errors.push("Esc 应关闭引导");
await shot("01-list");

// 选中一条带折返的手势 → 编辑面板
await clickText(".row", "搜索选中文字");
await shot("02-editor");

// 新建手势 → 在录制画布上画 ↑→↓ 
await clickText("button", "新建手势");
const pad = await page.$(".recorder-pad canvas");
const b = await pad.boundingBox();
const p0 = [b.x + b.width / 2 - 60, b.y + b.height - 30];
await page.mouse.move(...p0);
await page.mouse.down();
const path = [[0, -140], [120, -140], [120, 0]];
let cur = [0, 0];
for (const [tx, ty] of path) {
  for (let i = 1; i <= 12; i++) {
    await page.mouse.move(p0[0] + cur[0] + ((tx - cur[0]) * i) / 12, p0[1] + cur[1] + ((ty - cur[1]) * i) / 12);
  }
  cur = [tx, ty];
}
await page.mouse.up();
await sleep(700);
await shot("03-recorded");

// 名称输入 + 按键录制
await page.click(".editor-name", { clickCount: 3 });
await page.keyboard.type("关闭标签页");
await clickText(".keys-input button", "录制");
await page.keyboard.down("Control");
await page.keyboard.press("KeyW");
await page.keyboard.up("Control");
await clickText(".keys-input button", "完成");
await sleep(400);
await shot("04-keys");

// 添加一步菜单
await clickText(".chip-add", "添加一步");
await shot("05-addstep");
await page.keyboard.press("Escape");
await page.mouse.click(5, 5);

// 屏幕边角触发
await clickText(".segmented button", "屏幕边角");
await sleep(200);
await shot("06-edge");

// 删除 → 撤销提示
await clickText(".editor-foot button", "删除");
await shot("07-deleted-toast");
await clickText(".toast button", "撤销");
await sleep(200);

// 偏好设置（深色模式）
await page.emulateMediaFeatures([{ name: "prefers-color-scheme", value: "dark" }]);
await clickText(".nav-item", "偏好设置");
await shot("08-prefs-dark");
await page.emulateMediaFeatures([{ name: "prefers-color-scheme", value: "light" }]);

// 添加程序
await clickText(".nav-item", "添加程序");
await sleep(200);
await shot("09-apppicker");
await clickText(".app-list button", "chrome.exe");
await sleep(200);
await shot("10-app-scope");

// 导入导出
await clickText(".nav-item", "导入与导出");
await shot("11-data");

// 窄窗口：编辑面板变抽屉
await page.setViewport({ width: 980, height: 700 });
await clickText(".nav-item", "所有程序");
await clickText(".tab", "鼠标手势");
await clickText(".row", "复制");
await shot("12-narrow");

await sleep(600);
const saved = await page.evaluate(() => {
  const c = window.__saved;
  return c && { scopes: c.scopes.length, global: c.scopes.find((s) => s.kind === "global").bindings.length, apps: c.scopes.filter((s) => s.kind === "app").map((s) => s.app.exe) };
});
// ---- 分类与筛选
await page.evaluate(() => document.querySelector('.nav-item').click());
await sleep(200);
await clickText(".tab", "触发角");
const edgeRows = await page.$$eval(".row-pattern", (els) => els.map((e) => e.textContent));
if (!edgeRows.length || !edgeRows.every((t) => t.startsWith("移到"))) errors.push("触发角标签页里出现了非边角手势: " + edgeRows.join("|"));
await clickText(".tab", "鼠标手势");
await clickText(".filter", "中键");
const midRows = await page.$$eval(".row-pattern", (els) => els.map((e) => e.textContent));
if (!midRows.length || !midRows.every((t) => t.includes("中键"))) errors.push("中键筛选结果不对: " + midRows.join("|"));
await shot("13-filter-middle");
await clickText(".btn.primary", "新建手势");
const newPat = await page.$eval(".segmented button.on", (e) => e.textContent);
if (!newPat.includes("中键")) errors.push("在中键筛选下新建的手势应默认按住中键，实际: " + newPat);
await clickText(".tab", "触发角");
const tabAfter = await page.$eval(".tab.on", (e) => e.textContent);
await clickText(".btn.primary", "新建触发角");
await shot("14-new-edge");
const edgeSeg = await page.$eval(".segmented button.on", (e) => e.textContent);
if (!edgeSeg.includes("屏幕边角")) errors.push("新建触发角应默认为屏幕边角，实际: " + edgeSeg + " / " + tabAfter);

// ---- 停用
await page.setViewport({ width: 1180, height: 760 });
await page.keyboard.press("Escape");
await clickText(".nav-item", "所有程序");
await clickText(".tab", "鼠标手势");
await clickText(".filter", "全部");
await page.evaluate(() => document.querySelectorAll(".row .switch")[1].click());
await sleep(600);
const dis1 = await page.evaluate(() => window.__saved.scopes.find((s) => s.kind === "global").bindings.filter((b) => b.disabled).map((b) => b.name));
if (dis1.length !== 1) errors.push("点开关后应有 1 个停用: " + dis1);
const rowOff = await page.evaluate(() => document.querySelectorAll(".row")[1].classList.contains("off"));
if (!rowOff) errors.push("停用的行应变淡");
await page.evaluate(() => document.querySelectorAll(".row")[2].click());
await page.focus(".rows");
await page.keyboard.press("Space");
await sleep(600);
const dis2 = await page.evaluate(() => window.__saved.scopes.find((s) => s.kind === "global").bindings.filter((b) => b.disabled).length);
if (dis2 !== 2) errors.push("空格切换后应有 2 个停用: " + dis2);
await clickText(".filter", "已停用");
const offRows = await page.$$eval(".row", (els) => els.length);
if (offRows !== 2) errors.push("「已停用」筛选应显示 2 行: " + offRows);
await shot("15-disabled");
await page.evaluate(() => document.querySelectorAll(".row")[0].click());
await sleep(200);
await shot("16-disabled-editor");

// ---- 添加程序：点选窗口 / 拖入
await page.keyboard.press("Escape");
await clickText(".nav-item", "添加程序");
await shot("17-picker");
await clickText(".modal .btn", "点选窗口");
await sleep(300);
const title1 = await page.$eval(".list-title input", (e) => e.value).catch(() => "");
if (title1 !== "firefox") errors.push("点选窗口后应跳到 firefox，实际: " + title1);
await page.evaluate(() => window.__emit("tauri://drag-enter", { paths: ["C:\\x\\a.lnk"], position: { x: 500, y: 300 } }));
await sleep(150);
const overlay = await page.$(".drop-overlay");
if (!overlay) errors.push("拖入时应显示提示层");
await shot("18-dropping");
await page.evaluate(() => window.__emit("tauri://drag-drop", { paths: ["C:\\x\\Notepad++.lnk", "C:\\x\\readme.txt"], position: { x: 500, y: 300 } }));
await sleep(400);
const title2 = await page.$eval(".list-title input", (e) => e.value).catch(() => "");
if (title2 !== "notepad++") errors.push("拖入快捷方式后应跳到 notepad++，实际: " + title2);
const toastText = await page.$eval(".toast", (e) => e.textContent).catch(() => "");
if (!toastText.includes("不是程序")) errors.push("拖入非程序文件应提示，实际: " + toastText);
if (await page.$(".drop-overlay")) errors.push("松开后提示层应消失");
await shot("19-dropped");

// ---- 在某个程序里禁用手势
await clickText(".nav-item", "添加程序");
await clickText(".picker-mode button", "在它里面禁用手势");
await clickText(".app-list button", "Code.exe");
await sleep(600);
const codeScope = await page.evaluate(() => window.__saved.scopes.find((s) => s.app && /code\.exe/i.test(s.app.exe)));
if (!codeScope || codeScope.disabled !== true) errors.push("以禁用模式添加的程序应 disabled=true: " + JSON.stringify(codeScope));
if (!(await page.$(".off-banner"))) errors.push("禁用的程序页应显示横幅");
const navOff = await page.$$eval(".nav-item.off", (els) => els.map((e) => e.textContent));
if (!navOff.some((t) => t.includes("Code") && t.includes("已禁用"))) errors.push("侧栏应标「已禁用」: " + navOff);
await shot("20-app-disabled");
await clickText(".off-banner .btn", "恢复手势");
await sleep(600);
const codeScope2 = await page.evaluate(() => window.__saved.scopes.find((s) => s.app && /code\.exe/i.test(s.app.exe)));
if (codeScope2.disabled) errors.push("恢复后 disabled 应清除");
// 已添加的程序在禁用模式下也能选，改为禁用
await clickText(".nav-item", "添加程序");
await clickText(".picker-mode button", "在它里面禁用手势");
await clickText(".app-list button", "explorer.exe");
await sleep(600);
const ex = await page.evaluate(() => window.__saved.scopes.find((s) => s.app && /explorer\.exe/i.test(s.app.exe)));
if (!ex?.disabled || !ex.bindings.length) errors.push("已有程序改为禁用时应保留手势并 disabled=true: " + JSON.stringify(ex && { d: ex.disabled, n: ex.bindings.length }));

// ---- 关于页 + 再看一遍引导（完成时禁用勾选的已安装程序）
await page.keyboard.press("Escape");
await clickText(".nav-item", "关于");
await sleep(500);
await shot("21-about");
await clickText(".about .btn", "查看使用引导");
for (let i = 0; i < 3; i++) await clickText(".guide .btn.primary", "下一步");
await clickText(".guide .btn.primary", "开始使用");
await sleep(600);
const offApps = await page.evaluate(() => window.__saved.scopes.filter((s) => s.disabled).map((s) => s.app.exe));
const offNames = await page.evaluate(() => window.__saved.scopes.filter((s) => s.disabled).map((s) => s.name));
if (!offNames.includes("远程桌面连接")) errors.push("内置清单添加的程序应使用中文显示名: " + offNames);
if (!(offApps.includes("mstsc.exe") && offApps.includes("blender.exe"))) errors.push("引导完成后应禁用勾选的已安装程序: " + offApps);
// 提示条：带撤销 → 有关闭按钮，点关闭后消失
await sleep(200);
if (!(await page.$(".toast .toast-close"))) errors.push("提示条应有关闭按钮");
await shot("22-toast");
await page.click(".toast .toast-close");
await sleep(400);
if (await page.$(".toast")) errors.push("点关闭后提示条应消失");
// 普通提示 4 秒自动关闭（期间界面重渲染不应重置计时）
await clickText(".nav-item", "添加程序");
await clickText(".picker-mode button", "在它里面禁用手势");
await shot("23-picker-suggest");
await page.keyboard.press("Escape");

// ---- 提示条自动关闭：期间引擎状态轮询（3s）会让界面重渲染，不能重置倒计时
await page.mouse.move(600, 200); // 指针不能停在提示条上（悬停会暂停倒计时）
await page.evaluate(() => window.__emit("tauri://drag-drop", { paths: ["C:\\x\\a.txt"], position: { x: 1, y: 1 } }));
await sleep(500);
if (!(await page.$(".toast"))) errors.push("拖入非程序应出现提示");
await sleep(4600);
if (await page.$(".toast")) errors.push("普通提示 4 秒后应自动关闭（曾因重渲染重置计时而永不消失）");
// 悬停时暂停
await page.evaluate(() => window.__emit("tauri://drag-drop", { paths: ["C:\\x\\b.txt"], position: { x: 1, y: 1 } }));
await sleep(300);
const box = await page.$eval(".toast", (e) => { const r = e.getBoundingClientRect(); return [r.x + 30, r.y + r.height / 2]; });
await page.mouse.move(box[0], box[1]);
await sleep(5000);
if (!(await page.$(".toast"))) errors.push("鼠标悬停时提示条应暂停倒计时、不关闭");
await page.mouse.move(600, 200);
await sleep(4600);
if (await page.$(".toast")) errors.push("移开后应继续倒计时并关闭");

console.log(JSON.stringify({ saved, calls: await page.evaluate(() => [...new Set(window.__calls.map((c) => c[0]))]), errors }, null, 1));
await browser.close();
