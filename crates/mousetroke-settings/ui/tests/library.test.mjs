import assert from "node:assert/strict";
import { after, test } from "node:test";
import { createServer } from "vite";
import { act, createElement } from "react";
import { JSDOM } from "jsdom";

// 手势库：纯函数（library.ts）、store 不变式、编辑器的共享编辑会话、选择器与整理向导。
const server = await createServer({ server: { middlewareMode: true, hmr: false, ws: false }, appType: "custom" });
after(() => server.close());
const platform = await server.ssrLoadModule("/src/platform.ts");
const keys = await server.ssrLoadModule("/src/gesture.ts");
const lib = await server.ssrLoadModule("/src/library.ts");
const store = await server.ssrLoadModule("/src/store.ts");
const { t } = await server.ssrLoadModule("/src/i18n/index.ts");
const { api } = await server.ssrLoadModule("/src/api.ts");
const { Editor } = await server.ssrLoadModule("/src/components/Editor.tsx");
const { GestureLibrary, placeText, conflictLines } = await server.ssrLoadModule("/src/components/GestureLibrary.tsx");
const { GesturePicker } = await server.ssrLoadModule("/src/components/GesturePicker.tsx");
const { ImportWizard } = await server.ssrLoadModule("/src/components/ImportWizard.tsx");
const { mergeConfigs } = await server.ssrLoadModule("/src/components/Dialogs.tsx");

const dom = new JSDOM("<!doctype html><html><body></body></html>", { url: "http://localhost/" });
Object.assign(globalThis, { window: dom.window, document: dom.window.document, HTMLElement: dom.window.HTMLElement, HTMLInputElement: dom.window.HTMLInputElement, Event: dom.window.Event, location: dom.window.location, requestAnimationFrame: (fn) => setTimeout(fn, 0), IS_REACT_ACT_ENVIRONMENT: true });
Object.defineProperty(globalThis, "navigator", { value: dom.window.navigator, configurable: true });
globalThis.ResizeObserver = class { observe() {} disconnect() {} };
dom.window.HTMLCanvasElement.prototype.getContext = () => null;
dom.window.HTMLElement.prototype.scrollIntoView = () => {};
const { createRoot } = await import("react-dom/client");
after(() => dom.window.close());
platform.initPlatform("windows");
// 形状比较走后端；编辑器测试不涉及形状样本
api.shapeLimits = async () => ({ accept: 0.8, margin: 0.04 });
api.shapeCompare = async () => 0;

async function mount(element) {
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  await act(async () => root.render(element));
  return { el: container, async close() { await act(async () => root.unmount()); container.remove(); } };
}
async function click(el) { assert.ok(el); await act(async () => el.click()); }
const byText = (el, text) => [...el.querySelectorAll("button, label")].find((b) => b.textContent.includes(text));

const R = { type: "press", key: "mouse:right" };
const LEFT = [R, { type: "trace", points: [50, 0, 0, 0] }];
const UP = [R, { type: "trace", points: [0, 50, 0, 0] }];
const DOWN = [R, { type: "trace", points: [0, 0, 0, 50] }];
const bind = (name, pattern, extra = {}) => ({ name, pattern, action: { type: "key_sequence", keys: "^w" }, execute_on_match: false, hide_label: false, secured: false, ...extra });
const settings = { drag_timeout_ms: 0, drag_timeout_hint: false, min_stroke_px: 30, target_mode: "under_cursor", pause_hotkey: "", show_tray: true, trail: {}, label: {} };

/** 通用「后退」与 Chrome「上一页」共享 g1（←）；两边都有一条 ↑。 */
function shared() {
  return {
    version: 1,
    settings,
    gestures: [{ id: "g1", pattern: LEFT }],
    scopes: [
      { id: "general", name: "general", kind: "global", bindings: [bind("back", LEFT, { gesture: "g1" }), bind("top", UP)] },
      { id: "chrome", name: "Chrome", kind: "app", app: { exe: "chrome.exe" }, bindings: [bind("prev", LEFT, { gesture: "g1" }), bind("reopen", UP)] },
    ],
  };
}
const at = (c, s, b) => c.scopes[s].bindings[b];

// ================================================================ 纯函数

test("pattern edits on a shared gesture update the library and every reference", () => {
  const c = lib.setBindingPattern(shared(), { scope: 1, binding: 0 }, UP);
  assert.deepEqual(lib.gestureById(c, "g1").pattern, UP);
  assert.deepEqual(at(c, 0, 0).pattern, UP);
  assert.deepEqual(at(c, 1, 0).pattern, UP);
  assert.deepEqual(lib.checkRefs(c), []);
  // 冲突仍按分组计算：两个分组里都与 ↑ 冲突
  assert.deepEqual(lib.gestureConflicts(c, "g1").map((x) => x.use.scope), [0, 1]);
  // 专用手势只改自己
  const inline = lib.setBindingPattern(shared(), { scope: 0, binding: 1 }, DOWN);
  assert.deepEqual(at(inline, 0, 1).pattern, DOWN);
  assert.deepEqual(lib.gestureById(inline, "g1").pattern, LEFT);
});

test("keep change here detaches the binding and restores the others", () => {
  const ref = { scope: 1, binding: 0 };
  const changed = lib.setBindingPattern(shared(), ref, UP);
  const kept = lib.keepChangeHere(changed, ref, LEFT);
  assert.equal(at(kept, 1, 0).gesture, undefined);
  assert.deepEqual(at(kept, 1, 0).pattern, UP);
  assert.deepEqual(at(kept, 0, 0).pattern, LEFT);
  assert.deepEqual(lib.gestureById(kept, "g1").pattern, LEFT);
  assert.deepEqual(lib.checkRefs(kept), []);
});

test("drafts never create library entries but identical patterns reuse one", () => {
  const c = shared();
  c.scopes[0].bindings.push(bind("draft", DOWN, { draft: true }));
  assert.equal(lib.promote(c, { scope: 0, binding: 2 }), null);
  c.scopes[1].bindings.push(bind("same", LEFT));
  const same = lib.promote(c, { scope: 1, binding: 2 });
  assert.equal(same.id, "g1");
  assert.equal(lib.gesturesOf(same.config).length, 1);
  const added = lib.promote(c, { scope: 0, binding: 1 });
  assert.notEqual(added.id, "g1");
  assert.match(added.id, /^u/);
  assert.equal(at(added.config, 0, 1).gesture, added.id);
});

test("deleting a used gesture keeps or removes its bindings", () => {
  const kept = lib.deleteGesture(shared(), "g1", "detach");
  assert.equal(lib.gesturesOf(kept).length, 0);
  assert.equal(kept.scopes[0].bindings.length, 2);
  assert.equal(at(kept, 0, 0).gesture, undefined);
  assert.deepEqual(at(kept, 0, 0).pattern, LEFT);
  const gone = lib.deleteGesture(shared(), "g1", "cascade");
  assert.deepEqual(gone.scopes.map((s) => s.bindings.map((b) => b.name)), [["top"], ["reopen"]]);
});

test("merging another config remaps clashing library ids", () => {
  const incoming = {
    ...shared(),
    gestures: [{ id: "g1", pattern: DOWN }, { id: "g2", pattern: LEFT }],
    scopes: [{ id: "x", name: "x", kind: "app", app: { exe: "x.exe" }, bindings: [bind("down", DOWN, { gesture: "g1" }), bind("left", LEFT, { gesture: "g2" })] }],
  };
  const merged = mergeConfigs(shared(), incoming);
  assert.deepEqual(lib.checkRefs(merged), []);
  const x = merged.scopes.find((s) => s.id === "x");
  assert.equal(x.bindings[1].gesture, "g1", "同画法映射到已有手势");
  assert.notEqual(x.bindings[0].gesture, "g1", "id 冲突且画法不同时换新 id");
  assert.deepEqual(lib.gestureById(merged, x.bindings[0].gesture).pattern, DOWN);
});

test("import plans apply user choices in one config change", () => {
  const inline = { version: 1, settings, scopes: [
    { id: "general", name: "general", kind: "global", bindings: [bind("back", LEFT), bind("close", [R, { type: "trace", points: [0, 0, 0, 50, 50, 50] }])] },
    { id: "explorer", name: "Explorer", kind: "app", app: { exe: "explorer.exe" }, bindings: [bind("back", LEFT), bind("close", [R, { type: "trace", points: [0, 0, 0, 100, 50, 100] }])] },
  ] };
  const plan = { source: "global", notes: [], groups: [
    { kind: "identical", pattern: LEFT, score: 1, checked: true, members: [{ scope: 0, binding: 0, relation: "seed", score: 1, strong: true }, { scope: 1, binding: 0, relation: "identical", score: 1, strong: true }] },
    { kind: "equivalent", pattern: at(inline, 0, 1).pattern, score: 1, checked: true, members: [{ scope: 0, binding: 1, relation: "seed", score: 1, strong: true }, { scope: 1, binding: 1, relation: "equivalent", score: 1, strong: true }] },
  ] };
  const merged = lib.applyImportPlan(inline, plan, lib.defaultChoices(plan));
  assert.deepEqual(merged.stats, { added: 2, linked: 4, replaced: 1 });
  assert.deepEqual(lib.checkRefs(merged.config), []);
  assert.deepEqual(at(merged.config, 1, 1).pattern, at(inline, 0, 1).pattern);
  // 不勾选：来源为「所有程序」时其他程序的成员保持专用
  const off = lib.applyImportPlan(inline, plan, [{ checked: true, choice: "rep" }, { checked: false, choice: "rep" }]);
  assert.equal(at(off.config, 1, 1).gesture, undefined);
  assert.deepEqual(off.stats, { added: 2, linked: 3, replaced: 0 });
  // 以其他成员为准
  const theirs = lib.applyImportPlan(inline, plan, [{ checked: true, choice: "rep" }, { checked: true, choice: 1 }]);
  assert.deepEqual(at(theirs.config, 0, 1).pattern, at(inline, 1, 1).pattern);
  // 来源为全部分组时，不合并的成员各自入库
  const all = lib.applyImportPlan(inline, { ...plan, source: "all" }, [{ checked: true, choice: "rep" }, { checked: false, choice: "rep" }]);
  assert.equal(all.stats.added, 3);
  assert.ok(at(all.config, 1, 1).gesture);
  // 原配置不被修改
  assert.equal(at(inline, 0, 0).gesture, undefined);
});

// ================================================================ store

test("the store refuses edits that bypass the gesture library in development", () => {
  const state = { config: shared(), past: [], future: [], revision: 0, lastKey: null, lastAt: 0, repairs: 0 };
  assert.throws(() => store.reducer(state, { t: "edit", f: (c) => store.updateBinding(c, 1, 0, (b) => ({ ...b, pattern: UP })) }), /out of sync/);
  const ok = store.reducer(state, { t: "edit", f: (c) => lib.setBindingPattern(c, { scope: 1, binding: 0 }, UP) });
  assert.equal(ok.repairs, 0);
  assert.deepEqual(at(ok.config, 0, 0).pattern, UP);
});

test("loading repairs broken references and saving omits an empty library", () => {
  const broken = shared();
  broken.scopes[1].bindings[0].pattern = UP;
  const state = { config: shared(), past: [], future: [], revision: 0, lastKey: null, lastAt: 0, repairs: 0 };
  const loaded = store.reducer(state, { t: "load", config: broken, keepHistory: false });
  assert.equal(at(loaded.config, 1, 0).gesture, undefined);
  assert.equal(at(loaded.config, 0, 0).gesture, "g1");
  const saved = store.persistedConfig({ ...loaded.config, gestures: [] });
  assert.equal("gestures" in saved, false);
  const named = store.persistedConfig({ ...loaded.config, gestures: [{ id: "g1", name: "", pattern: LEFT, extra: 1 }] });
  assert.deepEqual(named.gestures, [{ id: "g1", pattern: LEFT }]);
});

// ================================================================ 组件

function EditorHarness({ get, edit, sel }) {
  const config = get();
  const binding = at(config, sel.scope, sel.binding);
  const g = lib.gestureById(config, binding.gesture);
  const uses = g ? lib.gestureUses(config, g.id) : [];
  const sharedInfo = g && { gesture: g, uses: uses.length, places: uses.filter((u) => !(u.scope === sel.scope && u.binding === sel.binding)).map((u) => placeText(config, u)), conflicts: conflictLines(config, g.id, sel.scope) };
  return createElement(Editor, {
    binding: { ...binding, uiId: "k" }, bindingKey: "k", conflictWith: [], otherBindings: [],
    onChange() {}, onPatternChange: (p) => edit((c) => lib.setBindingPattern(c, sel, p)),
    shared: sharedInfo, canAdopt: lib.canAdopt(binding), onOpenPicker() {}, onPromote() {}, onUseElsewhere() {}, onShowInLibrary() {},
    onKeepHere: (before) => edit((c) => lib.keepChangeHere(c, sel, before)),
    onDuplicate() {}, onDelete() {}, onClose() {},
  });
}

test("editing a shared gesture shows its reach and can be kept to this binding", async () => {
  let config = shared();
  const history = [];
  const sel = { scope: 1, binding: 0 };
  const edit = (f) => { history.push(config); config = f(config); };
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  const draw = () => act(async () => root.render(createElement(EditorHarness, { get: () => config, edit, sel })));
  await draw();
  try {
    assert.match(container.querySelector(".share-tag").textContent, /back/);
    assert.equal(container.querySelector(".share-bar"), null);
    // 改触发键（中键）= 修改共享手势：两处同步，出现提示条与新冲突
    await click(byText(container, keys.buttonPhrase("middle", "hold")));
    await draw();
    assert.equal(at(config, 0, 0).pattern[0].key, "mouse:middle");
    const bar = container.querySelector(".share-bar");
    assert.ok(bar);
    assert.match(bar.textContent, new RegExp(t("ui.gestures.synced", { count: 2 })));
    // 「只改这里」：当前绑定独立，库与通用恢复原画法
    await click(byText(container, t("ui.gestures.only_here")));
    await draw();
    assert.equal(at(config, 1, 0).gesture, undefined);
    assert.equal(at(config, 1, 0).pattern[0].key, "mouse:middle");
    assert.deepEqual(at(config, 0, 0).pattern, LEFT);
    assert.deepEqual(lib.checkRefs(config), []);
    assert.ok(container.querySelector(".gesture-inline"));
    // 撤销「只改这里」：引用恢复，会话仍处于已修改状态
    config = history.pop();
    await draw();
    assert.ok(container.querySelector(".share-bar"));
    await click(byText(container, t("ui.gestures.done")));
    await draw();
    assert.equal(container.querySelector(".share-bar"), null);
  } finally {
    await act(async () => root.unmount());
    container.remove();
  }
});

test("the picker greys out patterns already used here and offers other apps' gestures", async () => {
  const c = shared();
  c.scopes[0].bindings.push(bind("down", DOWN));
  c.scopes[0].bindings.push(bind("draft", [R], { draft: true }));
  const picked = [];
  const view = await mount(createElement(GesturePicker, { config: c, target: { scope: 1, binding: 1 }, onPickGesture: (id) => picked.push(id), onPickInline: (r) => picked.push(r), onRecordNew() {}, onClose() {} }));
  try {
    const cards = [...view.el.querySelectorAll(".gesture-card")];
    // 库里的 ←（本程序「上一页」已在用 → 置灰并提示冲突）+ 通用里的专用手势 ↑、↓；草稿不列出
    assert.equal(cards.length, 3);
    assert.equal(cards.filter((c) => c.classList.contains("dim")).length, 1);
    await click(cards[0].querySelector("button"));
    assert.deepEqual(picked, ["g1"]);
  } finally {
    await view.close();
  }
});

test("library page lists usage, filters unused and shows the onboarding when empty", async () => {
  const c = shared();
  c.gestures.push({ id: "u1", name: "spare", pattern: DOWN });
  const props = { config: c, selected: null, onSelect() {}, onNew() {}, onImport() {}, tab: "gesture", onTab() {}, onFilter() {} };
  const view = await mount(createElement(GestureLibrary, { ...props, filter: "all" }));
  try {
    assert.equal(view.el.querySelectorAll(".gesture-card").length, 2);
    assert.match(view.el.textContent, new RegExp(t("ui.gestures.unused_badge")));
  } finally {
    await view.close();
  }
  const unused = await mount(createElement(GestureLibrary, { ...props, filter: "unused" }));
  try {
    assert.deepEqual([...unused.el.querySelectorAll(".gesture-card-name")].map((e) => e.textContent), ["spare"]);
  } finally {
    await unused.close();
  }
  const empty = await mount(createElement(GestureLibrary, { ...props, config: { ...c, gestures: [] }, filter: "all" }));
  try {
    assert.ok(byText(empty.el, t("ui.gestures.empty_import")));
  } finally {
    await empty.close();
  }
});

test("the import wizard previews counts and applies the chosen groups", async () => {
  const inline = { version: 1, settings, scopes: [
    { id: "general", name: "general", kind: "global", bindings: [bind("back", LEFT)] },
    { id: "chrome", name: "Chrome", kind: "app", app: { exe: "chrome.exe" }, bindings: [bind("prev", LEFT), bind("long", [R, { type: "trace", points: [100, 0, 0, 0] }])] },
  ] };
  const plan = { source: "global", notes: [], groups: [
    { kind: "equivalent", pattern: LEFT, score: 1, checked: true, members: [{ scope: 0, binding: 0, relation: "seed", score: 1, strong: true }, { scope: 1, binding: 0, relation: "identical", score: 1, strong: true }, { scope: 1, binding: 1, relation: "equivalent", score: 1, strong: true }] },
  ] };
  api.analyzeGestureImport = async () => plan;
  let applied = null;
  const view = await mount(createElement(ImportWizard, { config: inline, onApply: (p, ch) => (applied = lib.applyImportPlan(inline, p, ch)), onClose() {} }));
  try {
    await click(byText(view.el, t("ui.gestures.next")));
    const summary = () => view.el.querySelector(".import-summary").textContent;
    assert.match(summary(), new RegExp(t("ui.gestures.will_link", { count: 3 })));
    assert.match(summary(), new RegExp(t("ui.gestures.will_replace", { count: 1 })));
    // 取消勾选：不再替换
    await act(async () => view.el.querySelector(".import-group-head input[type=checkbox]").click());
    assert.match(summary(), new RegExp(t("ui.gestures.will_link", { count: 2 })));
    await click(byText(view.el, t("ui.gestures.do_import")));
    assert.equal(applied.stats.linked, 2);
    assert.equal(at(applied.config, 1, 1).gesture, undefined);
  } finally {
    await view.close();
  }
});

test("skipped import groups stay inline and the wizard lets you uncheck direct gestures", async () => {
  const inline = { version: 1, settings, scopes: [
    { id: "general", name: "general", kind: "global", bindings: [bind("back", LEFT), bind("down", DOWN)] },
    { id: "chrome", name: "Chrome", kind: "app", app: { exe: "chrome.exe" }, bindings: [bind("prev", LEFT)] },
  ] };
  const plan = { source: "global", notes: [], groups: [
    { kind: "identical", pattern: LEFT, score: 1, checked: true, members: [{ scope: 0, binding: 0, relation: "seed", score: 1, strong: true }, { scope: 1, binding: 0, relation: "identical", score: 1, strong: true }] },
    { kind: "identical", pattern: DOWN, score: 1, checked: true, members: [{ scope: 0, binding: 1, relation: "seed", score: 1, strong: true }] },
  ] };
  const skipped = lib.applyImportPlan(inline, plan, [{ checked: true, choice: "rep", skip: true }, { checked: true, choice: "rep" }]);
  assert.deepEqual(skipped.stats, { added: 1, linked: 1, replaced: 0 });
  assert.equal(at(skipped.config, 0, 0).gesture, undefined);
  assert.equal(at(skipped.config, 1, 0).gesture, undefined);
  assert.ok(at(skipped.config, 0, 1).gesture);

  api.analyzeGestureImport = async () => plan;
  let applied = null;
  const view = await mount(createElement(ImportWizard, { config: inline, onApply: (p, ch) => (applied = lib.applyImportPlan(inline, p, ch)), onClose() {} }));
  try {
    await click(byText(view.el, t("ui.gestures.next")));
    const summary = () => view.el.querySelector(".import-summary").textContent;
    assert.match(summary(), new RegExp(t("ui.gestures.will_add", { count: 2 })));
    await click(view.el.querySelector(".import-section-head"));
    const boxes = [...view.el.querySelectorAll(".import-direct input[type=checkbox]")];
    assert.equal(boxes.length, 2);
    await click(boxes[0]);
    assert.match(summary(), new RegExp(t("ui.gestures.will_add", { count: 1 })));
    assert.ok(view.el.textContent.includes(t("ui.gestures.direct_count", { picked: 1, total: 2 })));
    // 全不选：没有可导入的，导入按钮不可用
    await click(byText(view.el, t("ui.gestures.select_none")));
    assert.ok(byText(view.el, t("ui.gestures.do_import")).disabled);
    await click(byText(view.el, t("ui.gestures.select_all")));
    await click(view.el.querySelectorAll(".import-direct input[type=checkbox]")[1]);
    await click(byText(view.el, t("ui.gestures.do_import")));
    assert.ok(at(applied.config, 1, 0).gesture, "← 已导入并关联");
    assert.equal(at(applied.config, 0, 1).gesture, undefined, "↓ 被排除");
  } finally {
    await view.close();
  }
});

test("the gesture picker searches by name, drawing and where a gesture is used", async () => {
  const c = shared();
  c.scopes[0].bindings.push(bind("down", DOWN));
  const view = await mount(createElement(GesturePicker, { config: c, target: { scope: 1, binding: 1 }, onPickGesture() {}, onPickInline() {}, onRecordNew() {}, onClose() {} }));
  try {
    const search = view.el.querySelector(".picker-top .search");
    assert.equal(document.activeElement, search);
    const count = () => view.el.querySelectorAll(".gesture-card").length;
    assert.equal(count(), 3);
    await typeIn(search, "down");
    assert.equal(count(), 1);
    await typeIn(search, "no such gesture");
    assert.equal(count(), 0);
    assert.ok(view.el.textContent.includes(t("ui.gestures.picker_no_match")));
  } finally {
    await view.close();
  }
});

async function typeIn(el, value) {
  const setter = Object.getOwnPropertyDescriptor(dom.window.HTMLInputElement.prototype, "value").set;
  await act(async () => {
    setter.call(el, value);
    el.dispatchEvent(new dom.window.Event("input", { bubbles: true }));
  });
}
