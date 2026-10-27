import assert from "node:assert/strict";
import { after, test } from "node:test";
import { readFileSync } from "node:fs";
import { createServer } from "vite";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";

// 使用与产物相同的 Vite 转换，不复制待测试的按键与平台实现。
const server = await createServer({ server: { middlewareMode: true, hmr: false, ws: false }, appType: "custom" });
after(() => server.close());
const platform = await server.ssrLoadModule("/src/platform.ts");
const keys = await server.ssrLoadModule("/src/gesture.ts");
const { t } = await server.ssrLoadModule("/src/i18n/index.ts");
const { About, UpdateStatus } = await server.ssrLoadModule("/src/components/About.tsx");
const { ActionEditor } = await server.ssrLoadModule("/src/components/ActionEditor.tsx");
const { initActionCatalog } = await server.ssrLoadModule("/src/actions.ts");
const { Guide, PermissionRows, permissionsReady } = await server.ssrLoadModule("/src/components/Guide.tsx");
const event = (code, mods = {}) => ({ code, ctrlKey: false, altKey: false, shiftKey: false, metaKey: false, ...mods });

test("known presets render as a changeable card and future actions stay read-only", () => {
  initActionCatalog({
    presets: [{ id: "edit.copy", advanced_expansion: { windows: true, macos: true, reason_key: "preset.status.native_only" }, name_key: "preset.action.edit.copy.name", description_key: "preset.action.edit.copy.description", params: [], windows: { status: "implemented", value: { type: "keys", value: "^c" } }, macos: { status: "implemented", value: { type: "keys", value: "#c" } } }],
    defaults: {},
    commonPresetIds: [],
    knownActionTypes: ["preset"],
    unknownActionKey: "preset.status.unknown",
  });
  // 已知预置动作：动作卡片，可"更换/转为高级动作"，不再是只读
  const known = { type: "preset", id: "edit.copy", params: { future: [1, true] } };
  const html = renderToStaticMarkup(createElement(ActionEditor, { action: known, onChange: () => assert.fail("known preset card must not change by itself") }));
  assert.ok(!/readonly/i.test(html));
  assert.ok(html.includes(t("preset.action.edit.copy.name")));
  assert.ok(html.includes(t("preset.action.edit.copy.description")));
  assert.ok(html.includes(t("ui.actpick.change")));
  assert.ok(html.includes(t("ui.actpick.to_advanced")));
  assert.equal(keys.actionSummary(known), t("preset.action.edit.copy.name"));
  // 未知动作（含未收录的预置 ID）：只读保留原始 JSON，提供"更换"
  for (const action of [
    { type: "preset", id: "future.preset", params: { service: "future" } },
    { type: "future_action", payload: { nested: ["^%+#", null, 42] } },
  ]) {
    const before = structuredClone(action);
    const h = renderToStaticMarkup(createElement(ActionEditor, { action, onChange: () => assert.fail("read-only action must not change") }));
    assert.match(h, /readonly/i);
    assert.ok(h.includes(t("preset.status.unknown")));
    assert.ok(h.includes(t("ui.actpick.change")));
    assert.deepEqual(action, before);
    assert.equal(keys.actionSummary(action), t("preset.status.unknown"));
  }
});

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

test("Mac and Windows updates share installation and busy states with a Mac manual fallback", () => {
  const update = { current: "0.2.0", latest: "0.2.1", newer: true, notes_url: "", notes: ["<script>plain text</script>"] };
  for (const os of ["windows", "macos"]) {
    platform.initPlatform(os);
    let installed = 0, downloaded = 0;
    const props = { update, busy: null, error: null, onInstall() { installed++; }, onDownload() { downloaded++; }, onNotes() {} };
    const render = (extra = {}) => renderToStaticMarkup(createElement(UpdateStatus, { ...props, ...extra }));
    const html = render();
    assert.match(html, /update-install/);
    assert.ok(html.includes(t("ui.update.install")));
    assert.match(html, /&lt;script&gt;plain text&lt;\/script&gt;/);
    assert.equal(html.includes("update-download"), os === "macos");
    // 遍历真实组件返回的按钮，确认主要操作与手动下载的回调没有接反。
    const buttons = [];
    function visit(element) {
      if (!element || typeof element !== "object") return;
      if (element.type === "button") buttons.push(element);
      for (const child of [element.props?.children].flat(Infinity)) visit(child);
    }
    visit(UpdateStatus(props));
    buttons.find(b => b.props.className.includes("update-install")).props.onClick();
    assert.equal(installed, 1);
    assert.equal(downloaded, 0);
    if (os === "macos") {
      buttons.find(b => b.props.className.includes("update-download")).props.onClick();
      assert.equal(downloaded, 1);
    }
    assert.ok(render({ busy: "install" }).includes(t("ui.update.installing")));
    assert.doesNotMatch(render({ busy: "install" }), /update-install|update-download/);
    assert.match(render({ busy: "check" }), /disabled=""/);
    for (const key of ["update.error.not_writable", "update.error.replace", "mac.update.error.signature"]) {
      const failed = render({ error: { key, params: {} } });
      assert.ok(failed.includes(t(key)));
      assert.match(failed, /update-download/);
      assert.doesNotMatch(failed, /update-install/);
    }
    assert.ok(render({ update: { ...update, newer: false } }).includes(t("ui.update.current")));
  }
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
  const granted = { accessibility: true, input_monitoring: true, engine_ready: true };
  assert.equal((render({ initialPermissions: granted }).match(/role="tab"/g) ?? []).length, 4);
  assert.equal((render({ initialPermissions: { ...granted, engine_ready: false } }).match(/role="tab"/g) ?? []).length, 5);
  const setup = render({ setup: true, initialPermissions: granted });
  assert.ok(setup.includes(t("mac.setup.ready")));
  assert.equal((setup.match(/role="tab"/g) ?? []).length, 5);
});

test("permission setup asks only for Accessibility and waits for an active tap", () => {
  platform.initPlatform("macos");
  const status = { accessibility: true, input_monitoring: false };
  const html = renderToStaticMarkup(createElement(PermissionRows, { status, requesting: null, onGrant() {} }));
  assert.ok(html.includes(t("mac.setup.accessibility_why")));
  assert.ok(!html.includes(t("mac.setup.input_monitoring_why")));
  assert.ok(html.includes(t("mac.setup.granted")));
  assert.equal((html.match(/<button/g) ?? []).length, 1);
  assert.equal((html.match(/disabled=""/g) ?? []).length, 1);
  for (const accessibility of [false, true]) for (const input_monitoring of [false, true]) {
    for (const engine_ready of [undefined, null, false, true]) {
      assert.equal(permissionsReady({ accessibility, input_monitoring, engine_ready }), engine_ready === true);
    }
  }
});

test("Input Monitoring fallback remains actionable even when preflight succeeds but tap fails", () => {
  for (const input_monitoring of [false, true]) {
    const html = renderToStaticMarkup(createElement(PermissionRows, {
      status: { accessibility: true, input_monitoring, engine_ready: false },
      showInputMonitoring: true, requesting: null, onGrant() {},
    }));
    assert.equal((html.match(/<button/g) ?? []).length, 2);
    assert.equal((html.match(/disabled=""/g) ?? []).length, 1);
    assert.ok(html.includes(t("mac.setup.input_monitoring_why")));
    assert.ok(html.includes(t(input_monitoring ? "mac.setup.checking" : "mac.setup.pending")));
  }
});
