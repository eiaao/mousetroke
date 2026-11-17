import assert from "node:assert/strict";
import { after, test } from "node:test";
import { readFileSync } from "node:fs";
import { createServer } from "vite";
import { act, createElement } from "react";
import { JSDOM } from "jsdom";

const server = await createServer({ server: { middlewareMode: true, hmr: false, ws: false }, appType: "custom" });
after(() => server.close());
const { Prefs } = await server.ssrLoadModule("/src/components/Prefs.tsx");
const { BindingList } = await server.ssrLoadModule("/src/components/BindingList.tsx");
const { initLang, t } = await server.ssrLoadModule("/src/i18n/index.ts");
const { api } = await server.ssrLoadModule("/src/api.ts");
const { initPlatform } = await server.ssrLoadModule("/src/platform.ts");
api.appIcons = async () => ({});
api.validateKeys = async () => null;
initPlatform("windows");

const dom = new JSDOM("<!doctype html><html><body></body></html>", { url: "http://localhost/" });
Object.assign(globalThis, { window: dom.window, document: dom.window.document, HTMLElement: dom.window.HTMLElement, HTMLInputElement: dom.window.HTMLInputElement, Event: dom.window.Event, location: dom.window.location, IS_REACT_ACT_ENVIRONMENT: true });
initLang({ setting: "en-US" });
Object.defineProperty(globalThis, "navigator", { value: dom.window.navigator, configurable: true });
dom.window.HTMLElement.prototype.scrollIntoView = () => {};
const { createRoot } = await import("react-dom/client");
after(() => dom.window.close());

async function mount(element) {
  const el = document.createElement("div");
  document.body.append(el);
  const root = createRoot(el);
  const render = async (value) => act(async () => root.render(value));
  await render(element);
  return { el, render, async close() { await act(async () => root.unmount()); el.remove(); } };
}

test("global toggle updates settings", async () => {
  let settings = JSON.parse(readFileSync(new URL("../../../../tools/uitest/config.json", import.meta.url), "utf8")).settings;
  delete settings.pause_on_fullscreen;
  const original = structuredClone(settings);
  const props = () => ({ settings, onChange: (f) => { settings = f(settings); }, engine: { running: false, autostart: false, exeFound: true }, onAutostart() {} });
  const view = await mount(createElement(Prefs, props()));
  try {
    const toggle = [...view.el.querySelectorAll("label")].find((el) => el.textContent.includes(t("ui.prefs.pause_on_fullscreen"))).querySelector("input");
    assert.equal(toggle.checked, true);
    await act(async () => toggle.click());
    assert.deepEqual(settings, { ...original, pause_on_fullscreen: false });
    await view.render(createElement(Prefs, props()));
    assert.equal(toggle.checked, false);
    await act(async () => toggle.click());
    assert.deepEqual(settings, { ...original, pause_on_fullscreen: true });
  } finally { await view.close(); }
});

test("appearance theme defaults to system and updates immediately", async () => {
  let settings = JSON.parse(readFileSync(new URL("../../../../tools/uitest/config.json", import.meta.url), "utf8")).settings;
  delete settings.theme;
  const props = () => ({ settings, onChange: (f) => { settings = f(settings); }, engine: { running: false, autostart: false, exeFound: true }, onAutostart() {} });
  const view = await mount(createElement(Prefs, props()));
  try {
    const row = [...view.el.querySelectorAll(".pref-row")].find((el) => el.textContent.includes(t("ui.prefs.theme")));
    const buttons = () => [...row.querySelectorAll("button")];
    assert.equal(buttons()[0].getAttribute("aria-checked"), "true");
    await act(async () => buttons()[2].click());
    assert.equal(settings.theme, "dark");
    await view.render(createElement(Prefs, props()));
    assert.equal(buttons()[2].getAttribute("aria-checked"), "true");
    await act(async () => buttons()[0].click());
    assert.equal(settings.theme, undefined);
  } finally { await view.close(); }
});

test("app settings bar updates only that scope", async () => {
  let scope = { id: "chrome", name: "Chrome", kind: "app", app: { exe: "chrome.exe" }, inherit_global: false, bindings: [] };
  const original = structuredClone(scope);
  let pauseOnFullscreen = true;
  const props = () => ({ scope, pauseOnFullscreen, tab: "gesture", filter: "all", selected: null, clash: new Map(), onTab() {}, onFilter() {}, onSelect() {}, onAdd() {}, onDelete() {}, onDuplicate() {}, onToggle() {}, onDeleteScope() {}, onScopeChange: (f) => { scope = f(scope); } });
  const view = await mount(createElement(BindingList, props()));
  const rerender = () => view.render(createElement(BindingList, props()));
  const select = () => view.el.querySelector(".scope-props select");
  const switches = () => [...view.el.querySelectorAll(".scope-props .switch")];
  const choose = async (value) => act(async () => {
    const el = select();
    Object.getOwnPropertyDescriptor(dom.window.HTMLSelectElement.prototype, "value").set.call(el, value);
    el.dispatchEvent(new dom.window.Event("change", { bubbles: true }));
  });
  try {
    assert.equal(select().value, "inherit");
    assert.equal(select().options[0].textContent, t("ui.list.fullscreen_inherit", { state: t("ui.list.fullscreen_state_pause") }));
    for (const policy of ["allow", "pause", "inherit"]) {
      await choose(policy);
      assert.deepEqual(scope, { ...original, fullscreen_policy: policy === "inherit" ? undefined : policy });
      await rerender();
      assert.equal(select().value, policy);
    }
    pauseOnFullscreen = false;
    await rerender();
    assert.equal(select().options[0].textContent, t("ui.list.fullscreen_inherit", { state: t("ui.list.fullscreen_state_allow") }));

    // 使用手势开关：关掉即 disabled=true，其余两项置灰但保留值；再打开即恢复。
    const [use, inherit] = switches();
    assert.equal(use.checked, true);
    assert.equal(inherit.checked, false);
    scope = { ...scope, fullscreen_policy: "allow" };
    await rerender();
    await act(async () => use.click());
    assert.equal(scope.disabled, true);
    await rerender();
    assert.ok(view.el.textContent.includes(t("ui.list.use_gestures_off", { where: "chrome.exe" })));
    assert.equal(switches()[1].disabled, true);
    assert.equal(select().disabled, true);
    assert.equal(select().value, "allow");
    assert.equal(view.el.querySelector(".off-banner"), null);
    await act(async () => switches()[0].click());
    assert.equal(scope.disabled, undefined);
    assert.equal(scope.fullscreen_policy, "allow");

    // 桌面只有前两项；“所有程序”没有属性栏。
    scope = { ...original, kind: "special", special: "desktop", app: undefined };
    await rerender();
    assert.equal(switches().length, 2);
    assert.equal(select(), null);
    scope = { ...original, kind: "global", app: undefined };
    await rerender();
    assert.equal(view.el.querySelector(".scope-props"), null);
  } finally { await view.close(); }
});
