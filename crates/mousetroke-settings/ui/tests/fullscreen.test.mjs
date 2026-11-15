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

test("app choice updates only that scope", async () => {
  let scope = { id: "chrome", name: "Chrome", kind: "app", app: { exe: "chrome.exe" }, inherit_global: false, bindings: [] };
  const original = structuredClone(scope);
  let pauseOnFullscreen = true;
  const props = () => ({ scope, pauseOnFullscreen, tab: "gesture", filter: "all", selected: null, clash: new Map(), onTab() {}, onFilter() {}, onSelect() {}, onAdd() {}, onDelete() {}, onDuplicate() {}, onToggle() {}, onDeleteScope() {}, onScopeChange: (f) => { scope = f(scope); } });
  const view = await mount(createElement(BindingList, props()));
  try {
    const buttons = () => [...view.el.querySelectorAll('.list-fullscreen button')];
    assert.equal(buttons()[0].getAttribute("aria-pressed"), "true");
    assert.ok(buttons()[0].textContent.includes(t("ui.list.fullscreen_state_pause")));
    for (const [index, policy] of [[1, "allow"], [2, "pause"], [0, undefined]]) {
      await act(async () => buttons()[index].click());
      assert.deepEqual(scope, { ...original, fullscreen_policy: policy });
      await view.render(createElement(BindingList, props()));
      assert.equal(buttons()[index].getAttribute("aria-pressed"), "true");
    }
    pauseOnFullscreen = false;
    await view.render(createElement(BindingList, props()));
    assert.ok(buttons()[0].textContent.includes(t("ui.list.fullscreen_state_allow")));
    scope = { ...scope, disabled: true, fullscreen_policy: "allow" };
    await view.render(createElement(BindingList, props()));
    assert.ok(buttons().every((b) => b.disabled));
    await act(async () => buttons()[2].click());
    assert.equal(scope.fullscreen_policy, "allow");
    for (const kind of ["global", "special"]) {
      scope = { ...original, kind };
      await view.render(createElement(BindingList, props()));
      assert.equal(buttons().length, 0);
    }
  } finally { await view.close(); }
});
