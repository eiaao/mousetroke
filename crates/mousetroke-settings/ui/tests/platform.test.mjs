import assert from "node:assert/strict";
import { after, test } from "node:test";
import { readFileSync } from "node:fs";
import { createServer } from "vite";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";

// 使用与产物相同的 Vite 转换，不复制待测试的按键与平台实现。
const server = await createServer({ server: { middlewareMode: true }, appType: "custom" });
after(() => server.close());
const platform = await server.ssrLoadModule("/src/platform.ts");
const keys = await server.ssrLoadModule("/src/gesture.ts");
const { t } = await server.ssrLoadModule("/src/i18n/index.ts");
const { About } = await server.ssrLoadModule("/src/components/About.tsx");
const { ActionEditor } = await server.ssrLoadModule("/src/components/ActionEditor.tsx");
const { Guide, PermissionRows, permissionsReady } = await server.ssrLoadModule("/src/components/Guide.tsx");
const event = (code, mods = {}) => ({ code, ctrlKey: false, altKey: false, shiftKey: false, metaKey: false, ...mods });

test("Windows display and identities retain their existing behavior", () => {
  platform.initPlatform("windows");
  assert.deepEqual(keys.keyseqToCaps("^+t%{F4}#d"), [["Ctrl", "Shift", "T"], ["Alt", "F4"], ["Win", "D"]]);
  assert.equal(platform.appIdentity("C:\\Apps\\Chrome.exe"), "Chrome.exe");
  assert.equal(platform.primaryModifier(event("KeyZ", { ctrlKey: true })), true);
  assert.equal(keys.actionSummary({ type: "key_sequence", keys: "^c" }), "^c");
});

test("macOS records physical keys as shared VK/keyseq values", () => {
  platform.initPlatform("macos");
  assert.equal(keys.codeToVk(event("MetaLeft")), 0x5b);
  assert.equal(keys.codeToVk(event("AltLeft")), 0xa4);
  assert.equal(keys.codeToVk(event("ArrowLeft")), 0x25);
  assert.equal(keys.codeToVk(event("F12")), 0x7b);
  assert.equal(keys.comboToKeyseq(event("KeyT", { metaKey: true, shiftKey: true })), "+#t");
  assert.equal(keys.comboToKeyseq(event("ArrowLeft", { metaKey: true, altKey: true })), "%#{LEFT}");
  assert.equal(keys.comboToKeyseq(event("F5", { ctrlKey: true })), "^{F5}");
  assert.equal(keys.comboToKeyseq(event("MetaLeft", { metaKey: true })), null);
  assert.match(keys.vkName(0x5b), /⌘/);
  assert.match(keys.actionSummary({ type: "key_sequence", keys: "#c" }), /⌘/);
  assert.equal(platform.primaryModifier(event("KeyZ", { metaKey: true })), true);
});

test("macOS bundle identities and executable paths survive scope creation", () => {
  platform.initPlatform("macos");
  for (const identity of ["com.google.Chrome", "/opt/tools/custom executable"]) {
    assert.equal(platform.appIdentity(identity), identity);
  }
  assert.match(t("ui.guide.p4_body"), /菜单栏/);
  assert.match(t("ui.data.reveal"), /Finder/);
});

test("unsupported imported actions require an explicit repair", () => {
  platform.initPlatform("macos");
  const original = { type: "launch_app", exe: "com.apple.Safari", elevate: true, if_running: "send_keys", win_class: "test" };
  assert.equal(platform.unsupportedAction(original), true);
  const repaired = platform.supportedAction(original);
  assert.equal(original.elevate, true);
  assert.equal(platform.unsupportedAction(repaired), false);
  assert.equal(repaired.if_running, "activate");
  // 与 Rust 的 mousetroke_core::caps 共用用例，任何一端规则漂移都会失败
  const cases = JSON.parse(readFileSync(new URL("../../../mousetroke-core/tests/fixtures/mac-caps.json", import.meta.url), "utf8"));
  for (const { action, repaired } of cases) {
    assert.equal(platform.unsupportedAction(action), !!repaired, JSON.stringify(action));
    assert.deepEqual(platform.supportedAction(action), repaired ?? action);
  }
});

test("macOS About retains project links and enables manual update checks", () => {
  const props = { engine: { running: true, autostart: false, exeFound: true }, onShowGuide() {}, notify() {} };
  platform.initPlatform("windows");
  assert.match(renderToStaticMarkup(createElement(About, props)), /update-check/);
  platform.initPlatform("macos");
  const html = renderToStaticMarkup(createElement(About, props));
  assert.match(html, /update-check/);
  assert.doesNotMatch(html, /update-install/);
  for (const key of ["ui.about.homepage", "ui.about.source", "ui.about.star_button"]) {
    assert.ok(html.includes(t(key)), key);
  }
});

test("macOS editor hides elevation and disables topmost without altering Windows", () => {
  const render = (action) => renderToStaticMarkup(createElement(ActionEditor, { action, onChange() {} }));
  platform.initPlatform("windows");
  assert.ok(render({ type: "launch_app", exe: "" }).includes(t("ui.action.elevate")));
  platform.initPlatform("macos");
  assert.ok(!render({ type: "launch_app", exe: "" }).includes(t("ui.action.elevate")));
  assert.match(render({ type: "window_op", op: "minimize" }), /disabled=""/);
  assert.ok(render({ type: "launch_app", exe: "com.apple.Safari", elevate: true }).includes(t("mac.ui.action.repair")));
});

test("permission setup adds a Mac step and preserves the four Windows steps", () => {
  const props = { pauseHotkey: "^%#p", onDisableApps() {}, onClose() {} };
  const render = (extra) => renderToStaticMarkup(createElement(Guide, { ...props, ...extra }));
  platform.initPlatform("windows");
  const windows = render({});
  assert.equal(render({ setup: true, initialPermissions: { accessibility: false, input_monitoring: false } }), windows);
  assert.equal((windows.match(/role="tab"/g) ?? []).length, 4);
  platform.initPlatform("macos");
  const missing = render({ initialPermissions: { accessibility: true, input_monitoring: false } });
  assert.ok(missing.includes(t("mac.setup.permissions")));
  assert.equal((missing.match(/role="tab"/g) ?? []).length, 5);
  const granted = { accessibility: true, input_monitoring: true };
  assert.equal((render({ initialPermissions: granted }).match(/role="tab"/g) ?? []).length, 4);
  const setup = render({ setup: true, initialPermissions: granted });
  assert.ok(setup.includes(t("mac.setup.ready")));
  assert.equal((setup.match(/role="tab"/g) ?? []).length, 5);
});

test("permission rows distinguish each grant and require both before advancing", () => {
  platform.initPlatform("macos");
  const status = { accessibility: true, input_monitoring: false };
  const html = renderToStaticMarkup(createElement(PermissionRows, { status, requesting: null, onGrant() {} }));
  assert.ok(html.includes(t("mac.setup.accessibility_why")));
  assert.ok(html.includes(t("mac.setup.input_monitoring_why")));
  assert.ok(html.includes(t("mac.setup.granted")));
  assert.ok(html.includes(t("mac.setup.pending")));
  assert.equal((html.match(/disabled=""/g) ?? []).length, 1);
  for (const [accessibility, input_monitoring, ready] of [[false, false, false], [true, false, false], [false, true, false], [true, true, true]]) {
    assert.equal(permissionsReady({ accessibility, input_monitoring }), ready);
  }
});
