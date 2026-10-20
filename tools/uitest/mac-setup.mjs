// Mac 设置引导的浏览器交互测试；仅模拟原生传输，渲染已构建的真实 UI。
// 用法：npm ci --prefix tools/uitest && node tools/uitest/mac-setup.mjs [Chrome/Edge 路径]
import assert from "node:assert/strict";
import { createServer } from "node:http";
import { readFile, mkdir } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";
import puppeteer from "puppeteer-core";

const root = fileURLToPath(new URL("../../", import.meta.url));
const dist = path.join(root, "crates/mousetroke-settings/ui/dist");
const config = JSON.parse(await readFile(path.join(root, "tools/uitest/config-en.json"), "utf8"));
const shots = path.join(root, "target/mac-setup-shots");
await mkdir(shots, { recursive: true });
const server = createServer(async (request, response) => {
  const pathname = new URL(request.url, "http://localhost").pathname;
  const filename = path.resolve(dist, "." + (pathname === "/" ? "/index.html" : pathname));
  if (!filename.startsWith(dist + path.sep)) { response.writeHead(403).end(); return; }
  try {
    const data = await readFile(filename);
    response.setHeader("Content-Type", filename.endsWith(".js") ? "text/javascript" : filename.endsWith(".css") ? "text/css" : "text/html");
    response.end(data);
  } catch { response.writeHead(404).end(); }
});
await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
const url = `http://127.0.0.1:${server.address().port}/`;
const browser = await puppeteer.launch({ executablePath: process.argv[2] ?? process.env.CHROME_PATH ?? "C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", headless: true });
const errors = [];
const ready = { accessibility: true, input_monitoring: true, engine_ready: true };
const missing = { accessibility: false, input_monitoring: false, engine_ready: false };
const delay = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

async function open(options = {}) {
  const page = await browser.newPage();
  await page.setViewport({ width: 900, height: 580, deviceScaleFactor: 1 });
  page.on("pageerror", (error) => errors.push(error.message));
  await page.evaluateOnNewDocument((config, options) => {
    localStorage.setItem("mousetroke.guide.seen", "1");
    const callbacks = new Map(), listeners = new Map();
    let next = 1;
    const status = window.__status = { ...(options.permissions ?? { accessibility: true, input_monitoring: true, engine_ready: true }) };
    const calls = window.__calls = [];
    window.__setupRequest = !!options.earlyRequest;
    window.__emit = (event) => {
      if (event === "show-setup") window.__setupRequest = true;
      for (const [id, entry] of listeners) if (entry.event === event) callbacks.get(entry.handler)?.({ event, id, payload: null });
    };
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: (_event, id) => listeners.delete(id) };
    window.__TAURI_INTERNALS__ = {
      metadata: { currentWindow: { label: "main" }, currentWebview: { label: "main", windowLabel: "main" } },
      transformCallback(callback) { const id = next++; callbacks.set(id, callback); return id; },
      unregisterCallback(id) { callbacks.delete(id); },
      async invoke(command, args) {
        calls.push({ command, args, time: performance.now() });
        switch (command) {
          case "init_state": return { platform: options.platform ?? "macos", setup: !!options.setup, configPath: "/Users/test/Library/Application Support/mousetroke/config.json", config: { ...config, settings: { ...config.settings, language: options.lang ?? "en-US", pause_hotkey: "^%#p" } }, configExists: true, loadError: null, engine: { running: true, exeFound: true, autostart: false }, systemLangs: [options.lang ?? "en-US"], resolvedLang: options.lang ?? "en-US", permissions: { ...status } };
          case "get_permissions": return { ...status };
          case "request_permission":
            if (window.__grantError) throw { key: "mac.settings.open_failed", params: {} };
            return null;
          case "take_setup_request": { const request = window.__setupRequest; window.__setupRequest = false; return request; }
          case "take_about_request": return false;
          case "restart_engine":
            if (options.restartError) throw { key: "error.start_engine", params: { err: "test" } };
            status.engine_ready = options.failAutomatic && calls.filter((call) => call.command === "restart_engine").length === 1 ? (options.emptyDuringRestart ? null : false) : true;
            return null;
          case "get_engine_status": return { running: true, exeFound: true, autostart: false };
          case "preset_apps": return [];
          case "app_icons": return {};
          case "app_info": return { version: "0.2.0", exePath: "/Applications/Mousetroke.app/Contents/MacOS/mousetroke", configPath: "/tmp/config.json", logPath: "/tmp/mousetroke.log" };
          case "plugin:event|listen": { const id = next++; listeners.set(id, args); return id; }
          case "plugin:event|unlisten": listeners.delete(args.eventId); return null;
          default: return null;
        }
      },
    };
  }, config, options);
  await page.goto(url, { waitUntil: "networkidle0" });
  await page.waitForSelector(".app");
  return page;
}

const count = (page, command) => page.evaluate((command) => window.__calls.filter((call) => call.command === command).length, command);
async function expectStopped(page) {
  // 给正在返回的调用一次完成机会，再观察是否仍安排了下一轮。
  await delay(100);
  const before = await count(page, "get_permissions");
  await delay(1300);
  assert.equal(await count(page, "get_permissions"), before);
}

try {
  const windows = await open({ platform: "windows", setup: true, permissions: missing });
  assert.equal(await count(windows, "get_permissions"), 0);
  assert.equal(await windows.$(".guide-permissions"), null);
  await windows.close();

  const page = await open({ permissions: missing });
  await page.waitForSelector(".guide-permissions");
  assert.equal((await page.$$(".permission-row")).length, 1);
  assert.equal(await count(page, "request_permission"), 0);
  await page.click(".permission-row:first-child button");
  assert.deepEqual(await page.evaluate(() => window.__calls.filter((call) => call.command === "request_permission").map((call) => call.args.permission)), ["accessibility"]);
  await page.evaluate(() => { window.__status.accessibility = true; window.__status.input_monitoring = true; });
  await page.waitForSelector(".permission-row:first-child button:disabled");
  assert.notEqual(await page.$(".guide-permissions"), null);
  assert.equal(await page.$(".permission-ready"), null);
  assert.equal((await page.$$(".permission-row")).length, 1);
  await page.evaluate(() => { window.__status.engine_ready = true; });
  await page.waitForSelector(".permission-ready");
  await page.waitForFunction(() => !document.querySelector(".guide-permissions"));
  for (let i = 0; i < 3; i++) await page.click(".guide-foot .btn.primary");
  await page.waitForSelector(".guide-try");
  assert.ok((await page.$eval(".guide-try", (element) => element.textContent)).includes("←"));
  await page.click(".guide-foot .btn.primary");
  await page.waitForFunction(() => !document.querySelector(".guide"));
  await expectStopped(page);
  await page.evaluate(() => window.__emit("show-about"));
  await page.waitForSelector(".about-guide");
  await page.click(".about-guide");
  await page.waitForSelector(".guide");
  assert.equal((await page.$$(".guide-dots button")).length, 4);
  await page.click(".guide-foot .btn.subtle");
  await page.evaluate(() => window.__emit("show-setup"));
  await page.waitForSelector(".guide-permissions");
  assert.equal((await page.$$(".guide")).length, 1);
  await page.close();

  const early = await open({ earlyRequest: true, permissions: ready });
  await early.waitForSelector(".guide-permissions");
  await early.close();

  const skip = await open({ setup: true, permissions: missing });
  await skip.click(".guide-foot .btn.subtle");
  await expectStopped(skip);
  assert.equal(await skip.$(".guide"), null);
  await skip.evaluate(() => window.__emit("show-settings"));
  await skip.waitForSelector(".guide-permissions");
  await skip.evaluate(() => { window.__grantError = true; });
  await skip.click(".permission-row:first-child button");
  await skip.waitForSelector(".guide [role=alert]");
  await delay(1300);
  assert.notEqual(await skip.$(".guide [role=alert]"), null);
  await skip.close();

  const automatic = await open({ permissions: { ...ready, engine_ready: false } });
  await automatic.waitForFunction(() => window.__calls.some((call) => call.command === "restart_engine"));
  await automatic.waitForFunction(() => !document.querySelector(".guide-permissions"));
  assert.equal(await count(automatic, "restart_engine"), 1);
  assert.equal(await automatic.$(".permission-restart"), null);
  await automatic.close();

  const recovered = await open({ setup: true, permissions: { ...ready, engine_ready: false } });
  await recovered.evaluate(() => { window.__status.engine_ready = true; });
  await recovered.waitForFunction(() => !document.querySelector(".guide-permissions"));
  assert.equal(await count(recovered, "restart_engine"), 0);
  await recovered.close();

  for (const emptyDuringRestart of [false, true]) {
    const fallback = await open({ setup: true, permissions: { ...ready, engine_ready: false }, failAutomatic: true, emptyDuringRestart });
    await fallback.waitForSelector(".permission-restart", { timeout: 14000 });
    assert.notEqual(await fallback.$(".guide-permissions"), null);
    if (!emptyDuringRestart) {
      assert.equal((await fallback.$$(".permission-row")).length, 2);
      assert.equal(await fallback.$eval(".permission-row:last-child button", (button) => button.disabled), false);
    }
    assert.equal(await count(fallback, "restart_engine"), 1);
    await delay(2400);
    assert.equal(await count(fallback, "restart_engine"), 1);
    await fallback.click(".permission-restart button");
    assert.equal(await count(fallback, "restart_engine"), 2);
    await fallback.waitForFunction(() => !document.querySelector(".permission-restart"));
    await fallback.close();
  }

  const failed = await open({ setup: true, permissions: { ...ready, engine_ready: false }, restartError: true });
  await failed.waitForSelector(".guide [role=alert]", { timeout: 7000 });
  await failed.waitForSelector(".permission-restart", { timeout: 9000 });
  assert.equal(await count(failed, "restart_engine"), 1);
  await failed.close();

  const partial = await open({ setup: true, permissions: { ...missing, accessibility: true } });
  await delay(2700);
  assert.equal(await count(partial, "restart_engine"), 0);
  assert.equal((await partial.$$(".permission-row")).length, 1);
  await partial.waitForFunction(() => document.querySelectorAll(".permission-row").length === 2);
  await partial.click(".permission-row:last-child button");
  assert.deepEqual(await partial.evaluate(() => window.__calls.filter((call) => call.command === "request_permission").map((call) => call.args.permission)), ["input_monitoring"]);
  await partial.evaluate(() => { window.__status.accessibility = false; });
  await partial.waitForFunction(() => document.querySelectorAll(".permission-row").length === 1);
  await delay(4200);
  assert.equal((await partial.$$(".permission-row")).length, 1);
  await partial.close();

  const active = await open({ setup: true, permissions: { ...ready, input_monitoring: false } });
  await active.waitForSelector(".permission-ready");
  await active.waitForFunction(() => !document.querySelector(".guide-permissions"));
  assert.equal(await count(active, "request_permission"), 0);
  assert.equal(await count(active, "restart_engine"), 0);
  await active.close();

  const unknown = await open({ setup: true, permissions: { ...ready, engine_ready: null } });
  await delay(4500);
  assert.notEqual(await unknown.$(".guide-permissions"), null);
  assert.equal(await unknown.$(".permission-ready"), null);
  assert.equal((await unknown.$$(".permission-row")).length, 1);
  await unknown.close();

  const cancelled = await open({ setup: true, permissions: { ...ready, engine_ready: false } });
  await cancelled.click(".guide-foot .btn.subtle");
  await delay(2700);
  assert.equal(await count(cancelled, "restart_engine"), 0);
  await expectStopped(cancelled);
  await cancelled.close();

  for (const lang of ["zh-CN", "en-US", "es-ES", "fr-FR", "de-DE", "ja-JP", "pt-BR", "ru-RU", "it-IT", "ko-KR"]) {
    const localized = await open({ setup: true, permissions: missing, lang });
    await localized.waitForSelector(".guide-permissions");
    assert.equal((await localized.$$(".permission-row")).length, 1, lang);
    assert.equal(await localized.$eval(".guide", (element) => element.scrollWidth > element.clientWidth), false, lang);
    assert.equal(await localized.$eval(".guide-foot", (element) => element.getBoundingClientRect().bottom > window.innerHeight), false, lang);
    assert.equal(await localized.$eval(".permission-row [role=status]", (element) => element.getBoundingClientRect().bottom > document.querySelector(".guide-text").getBoundingClientRect().bottom), false, `${lang}: Accessibility status must be visible`);
    await localized.screenshot({ path: path.join(shots, `${lang}.png`) });
    await localized.evaluate(() => { window.__status.accessibility = true; });
    await localized.waitForFunction(() => document.querySelectorAll(".permission-row").length === 2);
    assert.equal(await localized.$eval(".guide", (element) => element.scrollWidth > element.clientWidth), false, `${lang}: fallback width`);
    assert.equal(await localized.$eval(".guide-foot", (element) => element.getBoundingClientRect().bottom > window.innerHeight), false, `${lang}: fallback footer`);
    assert.equal(await localized.$eval(".permission-row:last-child button", (element) => element.getBoundingClientRect().bottom > document.querySelector(".guide-text").getBoundingClientRect().bottom), false, `${lang}: fallback button must be visible`);
    await localized.screenshot({ path: path.join(shots, `${lang}-fallback.png`) });
    await localized.close();
  }
  assert.deepEqual(errors, []);
  console.log("OK: Accessibility-only setup, delayed Input Monitoring fallback and revocation, active-tap readiness, polling cleanup, auto-advance, skip, setup routing/race, automatic restart, one-shot/error/empty-snapshot/manual fallback, Windows isolation, 10-language normal/fallback layout");
} finally {
  await browser.close();
  await new Promise((resolve) => server.close(resolve));
}
