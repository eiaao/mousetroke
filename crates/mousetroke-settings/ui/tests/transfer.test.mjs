import assert from "node:assert/strict";
import { after, test } from "node:test";
import { createServer } from "vite";
import { act, createElement } from "react";
import { JSDOM } from "jsdom";

const server = await createServer({ server: { middlewareMode: true, hmr: false, ws: false }, appType: "custom" });
after(() => server.close());
const { transferBinding, transferConflicts, transferTargets } = await server.ssrLoadModule("/src/transfer.ts");
const { reducer } = await server.ssrLoadModule("/src/store.ts");
const { TransferDialog } = await server.ssrLoadModule("/src/components/TransferDialog.tsx");
const { BindingList } = await server.ssrLoadModule("/src/components/BindingList.tsx");

const pattern = [{ type: "press", key: "mouse:right" }, { type: "press", key: "wheel:1" }];
const binding = (name, extra = {}) => ({ name, pattern, action: { type: "key_sequence", keys: "{PGDN}" }, execute_on_match: true, hide_label: true, secured: false, uiId: name, ...extra });
const fixture = () => ({
  version: 1,
  gestures: [{ id: "g1", pattern }],
  settings: {},
  scopes: [
    { id: "global", name: "All", kind: "global", bindings: [binding("source", { gesture: "g1" })] },
    { id: "chrome", name: "Chrome", kind: "app", app: { exe: "chrome.exe" }, bindings: [] },
    { id: "firefox", name: "Firefox", kind: "app", app: { exe: "firefox.exe" }, bindings: [] },
  ],
});

test("copying to several apps preserves action and options but makes independent drawings", () => {
  const original = fixture();
  const copied = transferBinding(original, { scope: 0, binding: 0 }, [1, 2], "copy", {});
  assert.equal(original.scopes[1].bindings.length, 0);
  assert.equal(copied.scopes[0].bindings.length, 1);
  for (const scope of copied.scopes.slice(1)) {
    const b = scope.bindings[0];
    assert.deepEqual(b.action, original.scopes[0].bindings[0].action);
    assert.equal(b.execute_on_match, true);
    assert.equal(b.hide_label, true);
    assert.equal(b.gesture, undefined);
    assert.notEqual(b.uiId, "source");
  }
  copied.scopes[1].bindings[0].pattern[1].key = "wheel:-1";
  assert.equal(copied.scopes[2].bindings[0].pattern[1].key, "wheel:1");
});

test("move skips conflicts unless replaced and keeps source when every target is skipped", () => {
  const original = fixture();
  original.scopes[1].bindings.push(binding("existing"));
  assert.deepEqual(transferConflicts(original.scopes[1], original.scopes[0].bindings[0]), [0]);
  assert.equal(transferBinding(original, { scope: 0, binding: 0 }, [1], "move", {}), original);
  const moved = transferBinding(original, { scope: 0, binding: 0 }, [1, 2], "move", { 1: "replace" });
  assert.equal(moved.scopes[0].bindings.length, 0);
  assert.deepEqual(moved.scopes[1].bindings.map((b) => b.name), ["source"]);
  assert.deepEqual(moved.scopes[2].bindings.map((b) => b.name), ["source"]);
  assert.equal(moved.scopes[1].bindings[0].gesture, undefined);
});

test("the destination count uses applied scopes, including duplicates and skipped conflicts", () => {
  const original = fixture();
  original.scopes[1].bindings.push(binding("existing"));
  const source = { scope: 0, binding: 0 };
  assert.deepEqual(transferTargets(original, source, [1, 1, 2, 2], {}), [2]);
  const moved = transferBinding(original, source, [1, 1, 2, 2], "move", {});
  assert.deepEqual(moved.scopes[1].bindings.map((b) => b.name), ["existing"]);
  assert.equal(moved.scopes[2].bindings[0].gesture, "g1");
  assert.equal(moved.scopes[0].bindings.length, 0);
});

test("single-destination move retains the library reference and is one undo step", () => {
  const original = fixture();
  const state = { config: original, past: [], future: [], revision: 0, lastKey: null, lastAt: 0, repairs: 0 };
  const edited = reducer(state, { t: "edit", f: (c) => transferBinding(c, { scope: 0, binding: 0 }, [1], "move", {}) });
  assert.equal(edited.config.scopes[1].bindings[0].gesture, "g1");
  assert.equal(edited.past.length, 1);
  assert.deepEqual(reducer(edited, { t: "undo" }).config, original);
});

test("transfer dialog requires an explicit choice for a conflicting destination", async () => {
  const dom = new JSDOM("<!doctype html><html><body></body></html>", { url: "http://localhost/" });
  Object.assign(globalThis, { window: dom.window, document: dom.window.document, HTMLElement: dom.window.HTMLElement, location: dom.window.location, IS_REACT_ACT_ENVIRONMENT: true });
  const { createRoot } = await import("react-dom/client");
  const config = fixture();
  config.scopes[1].bindings.push(binding("existing"));
  config.scopes[2].disabled = true;
  const host = document.createElement("div");
  document.body.append(host);
  const root = createRoot(host);
  let applied;
  await act(async () => root.render(createElement(TransferDialog, {
    config, source: { scope: 0, binding: 0 },
    onApply: (...args) => { applied = args; }, onAddApp: () => {}, onClose: () => {},
  })));
  const chrome = [...host.querySelectorAll("label")].find((label) => label.textContent.includes("Chrome"));
  await act(async () => chrome.querySelector("input").click());
  const [copyButton, moveButton, cancelButton] = host.querySelectorAll(".modal-foot button");
  const disabledScope = [...host.querySelectorAll(".transfer-destination label")].find((label) => label.textContent.includes("Firefox"));
  assert.equal(disabledScope.querySelector(".transfer-destination-off").previousElementSibling.classList.contains("transfer-destination-name"), true);
  assert.equal(host.querySelector(".segmented"), null);
  assert.equal(copyButton.classList.contains("primary"), true);
  assert.equal(moveButton.classList.contains("primary"), true);
  assert.equal(copyButton.disabled, true);
  assert.equal(moveButton.disabled, true);
  assert.equal(cancelButton.disabled, false);
  const choice = host.querySelector(".transfer-conflict select");
  await act(async () => {
    choice.value = "replace";
    choice.dispatchEvent(new dom.window.Event("change", { bubbles: true }));
  });
  assert.equal(copyButton.disabled, false);
  assert.equal(moveButton.disabled, false);
  await act(async () => copyButton.click());
  assert.deepEqual(applied, ["copy", [1], { 1: "replace" }]);
  await act(async () => moveButton.click());
  assert.deepEqual(applied, ["move", [1], { 1: "replace" }]);
  await act(async () => root.unmount());
  host.remove();
  dom.window.close();
});

test("right-click offers one transfer action without making the row draggable", async () => {
  const dom = new JSDOM("<!doctype html><html><body></body></html>", { url: "http://localhost/" });
  Object.assign(globalThis, { window: dom.window, document: dom.window.document, HTMLElement: dom.window.HTMLElement, location: dom.window.location, IS_REACT_ACT_ENVIRONMENT: true });
  dom.window.HTMLElement.prototype.scrollIntoView = () => {};
  const { createRoot } = await import("react-dom/client");
  const config = fixture();
  let transferred;
  const host = document.createElement("div");
  document.body.append(host);
  const root = createRoot(host);
  const noop = () => {};
  await act(async () => root.render(createElement(BindingList, {
      scope: config.scopes[0], pauseOnFullscreen: false, tab: "gesture", onTab: noop, filter: "all", onFilter: noop,
      selected: null, onSelect: noop, onAdd: noop, onDelete: noop, onDuplicate: noop, onToggle: noop,
      onTransfer: (i) => { transferred = i; },
      onScopeChange: noop, onDeleteScope: noop, clash: new Map(),
  })));
  const row = host.querySelector(".row");
  assert.equal(row.hasAttribute("draggable"), false);
  await act(async () => row.dispatchEvent(new dom.window.MouseEvent("contextmenu", { bubbles: true, clientX: 10, clientY: 10 })));
  const menu = document.body.querySelector(".gesture-context-menu");
  assert.ok(menu);
  assert.equal(menu.querySelectorAll("button").length, 1);
  await act(async () => menu.querySelector("button").click());
  assert.equal(transferred, 0);
  await act(async () => root.unmount());
  host.remove();
  dom.window.close();
});
