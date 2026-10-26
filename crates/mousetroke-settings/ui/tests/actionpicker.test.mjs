import assert from "node:assert/strict";
import { after, test } from "node:test";
import { createServer } from "vite";
import { act, createElement, useState } from "react";
import { readFileSync, mkdtempSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { JSDOM } from "jsdom";
import { renderToStaticMarkup } from "react-dom/server";

// 使用与产物相同的 Vite 转换，测试动作选择器的搜索、分类、键盘与参数校验。
const server = await createServer({ server: { middlewareMode: true, hmr: false, ws: false }, appType: "custom" });
after(() => server.close());
const platform = await server.ssrLoadModule("/src/platform.ts");
const actions = await server.ssrLoadModule("/src/actions.ts");
const keys = await server.ssrLoadModule("/src/gesture.ts");
const { t, initLang } = await server.ssrLoadModule("/src/i18n/index.ts");
const { ActionPicker } = await server.ssrLoadModule("/src/components/ActionPicker.tsx");
const { ActionEditor } = await server.ssrLoadModule("/src/components/ActionEditor.tsx");
const { Modal } = await server.ssrLoadModule("/src/components/Dialogs.tsx");
const { BindingList } = await server.ssrLoadModule("/src/components/BindingList.tsx");
const { persistedConfig, withBindingIds, updateBinding, updateScope, useConfigStore } = await server.ssrLoadModule("/src/store.ts");
const { api } = await server.ssrLoadModule("/src/api.ts");
const { Editor } = await server.ssrLoadModule("/src/components/Editor.tsx");

/** 与后端 action_catalog 序列化形状一致的迷你目录（真实词条键，t() 可取到译文）。 */
const fixture = {
  presets: [
    {
      id: "browser.close_tab", category: "browser",
      name_key: "preset.action.browser.close_tab.name", description_key: "preset.action.browser.close_tab.description", aliases_key: "preset.action.browser.close_tab.aliases",
      params: [],
      windows: { status: "implemented", value: { type: "keys", value: "^w" } },
      macos: { status: "implemented", value: { type: "keys", value: "#w" } },
    },
    {
      id: "text.type", category: "text",
      name_key: "preset.action.text.type.name", description_key: "preset.action.text.type.description", aliases_key: "preset.action.text.type.aliases",
      params: [
        { name: "text", label_key: "preset.param.text", kind: { type: "multiline_text" }, default: { kind: "literal", value: "" } },
        { name: "method", label_key: "preset.param.method", kind: { type: "choice", choices: [{ value: "paste", label_key: "preset.choice.paste" }, { value: "type", label_key: "preset.choice.type" }] }, default: { kind: "literal", value: "paste" } },
      ],
      windows: { status: "implemented", value: { type: "native", value: { type: "type_text" } } },
      macos: { status: "implemented", value: { type: "native", value: { type: "type_text" } } },
    },
    {
      id: "text.search", category: "text",
      name_key: "preset.action.text.search.name", description_key: "preset.action.text.search.description", aliases_key: "preset.action.text.search.aliases",
      params: [
        { name: "engine", label_key: "preset.param.engine", kind: { type: "choice", choices: [{ value: "google", label_key: "preset.choice.google" }, { value: "baidu", label_key: "preset.choice.baidu" }, { value: "custom", label_key: "preset.choice.custom" }] }, default: { kind: "search_engine" } },
        { name: "url", label_key: "preset.param.search_url", visible_when: { param: "engine", equals: "custom" }, kind: { type: "url" }, default: { kind: "literal", value: "https://www.google.com/search?q=%MS_SELECTED_TEXT%" } },
      ],
      windows: { status: "implemented", value: { type: "search" } },
      macos: { status: "implemented", value: { type: "search" } },
    },
    {
      id: "text.datetime", category: "text",
      name_key: "preset.action.text.datetime.name", description_key: "preset.action.text.datetime.description", aliases_key: "preset.action.text.datetime.aliases",
      params: [
        { name: "format", label_key: "preset.param.format", kind: { type: "choice", choices: [{ value: "date_iso", label_key: "preset.choice.date_iso" }, { value: "custom", label_key: "preset.choice.custom" }] }, default: { kind: "literal", value: "date_iso" } },
        { name: "custom_format", label_key: "preset.param.custom_format", visible_when: { param: "format", equals: "custom" }, kind: { type: "text" }, default: { kind: "literal", value: "YYYY-MM-DD" } },
        { name: "method", label_key: "preset.param.method", kind: { type: "choice", choices: [{ value: "paste", label_key: "preset.choice.paste" }, { value: "type", label_key: "preset.choice.type" }] }, default: { kind: "literal", value: "paste" } },
      ],
      windows: { status: "implemented", value: { type: "native", value: { type: "date_time" } } },
      macos: { status: "implemented", value: { type: "native", value: { type: "date_time" } } },
    },
    {
      id: "open.website", category: "open",
      name_key: "preset.action.open.website.name", description_key: "preset.action.open.website.description", aliases_key: "preset.action.open.website.aliases",
      params: [{ name: "url", label_key: "preset.param.url", kind: { type: "url" }, default: { kind: "literal", value: "https://mousetroke.com/" } }],
      windows: { status: "implemented", value: { type: "website" } },
      macos: { status: "implemented", value: { type: "website" } },
    },
    {
      id: "desktop.clipboard_history", category: "desktop",
      name_key: "preset.action.desktop.clipboard_history.name", description_key: "preset.action.desktop.clipboard_history.description", aliases_key: "preset.action.desktop.clipboard_history.aliases",
      params: [],
      windows: { status: "implemented", value: { type: "keys", value: "#v" } },
      macos: { status: "unsupported", value: { reason_key: "preset.status.windows_only" } },
    },
    {
      id: "mousetroke.cheatsheet", category: "mousetroke",
      name_key: "preset.action.mousetroke.cheatsheet.name", description_key: "preset.action.mousetroke.cheatsheet.description", aliases_key: "preset.action.mousetroke.cheatsheet.aliases",
      params: [],
      windows: { status: "pending", value: { reason_key: "preset.status.phase2" } },
      macos: { status: "pending", value: { reason_key: "preset.status.phase2" } },
    },
  ],
  defaults: {
    "text.type": { text: "", method: "paste" },
    "text.search": { engine: "google", url: "https://www.google.com/search?q=%MS_SELECTED_TEXT%" },
    "text.datetime": { format: "date_iso", custom_format: "YYYY-MM-DD", method: "paste" },
    "open.website": { url: "https://mousetroke.com/" },
  },
  commonPresetIds: ["browser.close_tab", "text.type", "open.website", "desktop.clipboard_history"],
  knownActionTypes: ["preset"],
  unknownActionKey: "preset.status.unknown",
};

for (const preset of fixture.presets) {
  preset.advanced_expansion = {
    windows: preset.windows.status === "implemented" && preset.windows.value.type !== "native",
    macos: preset.macos.status === "implemented" && preset.macos.value.type !== "native",
    reason_key: "preset.status.native_only",
  };
}

/** Editor 无 hooks，可直接调用取元素树（与 platform.test.mjs 遍历 UpdateStatus 同源）。 */
function findAll(element, pred, out = []) {
  if (!element || typeof element !== "object") return out;
  if (Array.isArray(element)) {
    for (const child of element) findAll(child, pred, out);
    return out;
  }
  if (pred(element)) out.push(element);
  findAll(element.props?.children, pred, out);
  return out;
}

const render = (element) => renderToStaticMarkup(element);

function setup() {
  platform.initPlatform("windows");
  actions.initActionCatalog(fixture);
}

// 事件与焦点使用真实 DOM 挂载；静态渲染无法验证 Escape 的冒泡顺序。
const dom = new JSDOM("<!doctype html><html><body></body></html>", { url: "http://localhost/" });
Object.assign(globalThis, { window: dom.window, document: dom.window.document, HTMLElement: dom.window.HTMLElement, HTMLInputElement: dom.window.HTMLInputElement, Event: dom.window.Event, location: dom.window.location, requestAnimationFrame: (fn) => setTimeout(fn, 0), IS_REACT_ACT_ENVIRONMENT: true });
Object.defineProperty(globalThis, "navigator", { value: dom.window.navigator, configurable: true });
// jsdom 不提供录制画布的尺寸监听和绘图；这些测试只验证编辑状态。
globalThis.ResizeObserver = class { observe() {} disconnect() {} };
dom.window.HTMLCanvasElement.prototype.getContext = () => null;
let scrolled;
dom.window.HTMLElement.prototype.scrollIntoView = function () { scrolled = this; };
const { createRoot } = await import("react-dom/client");
after(() => dom.window.close());
api.validateKeys = async () => null;
api.validatePreset = async () => {};

test("search matches names, aliases and shortcut spellings on the current platform", () => {
  setup();
  const close = actions.findPreset("browser.close_tab");
  for (const q of ["^w", "ctrl+w", "Ctrl + W", "control+w", "⌃w", "ctrlw"]) {
    assert.ok(actions.matchPreset(close, q), `windows: ${q} 应匹配关闭标签页`);
  }
  assert.ok(actions.matchPreset(close, t("preset.action.browser.close_tab.name")));
  assert.ok(!actions.matchPreset(close, "不存在的动作xyz"));
  assert.ok(actions.keyseqSearchVariants("^+t").includes("ctrl+shift+t"));
  platform.initPlatform("macos");
  for (const q of ["⌘w", "cmd+w", "command+w", "#w"]) {
    assert.ok(actions.matchPreset(close, q), `macos: ${q} 应匹配关闭标签页`);
  }
});

test("category filter shows common and registry categories, search ignores category", () => {
  setup();
  assert.deepEqual(actions.visiblePresets("common", "").map((p) => p.id), ["browser.close_tab", "text.type", "open.website", "desktop.clipboard_history"]);
  assert.deepEqual(actions.visiblePresets("browser", "").map((p) => p.id), ["browser.close_tab"]);
  assert.deepEqual(actions.visiblePresets("mousetroke", "").map((p) => p.id), ["mousetroke.cheatsheet"]);
  assert.deepEqual(actions.visiblePresets("advanced", ""), []);
  assert.deepEqual(actions.visiblePresets("browser", "网站").map((p) => p.id), ["open.website"]);
  assert.deepEqual(actions.visiblePresets("common", "搜索").map((p) => p.id), ["text.search"]);
});

test("keyboard navigation moves in a loop and skips unavailable rows (12)", () => {
  setup();
  const close = actions.findPreset("browser.close_tab");
  const pending = actions.findPreset("text.type");
  const cheat = actions.findPreset("mousetroke.cheatsheet");
  const sel = [close, pending, cheat, actions.findPreset("open.website")].map((p) => actions.presetPlatformStatus(p) === "implemented");
  assert.equal(actions.pickerNav(0, 4, "ArrowDown", sel), 1, "原生文字动作可选");
  assert.equal(actions.pickerNav(1, 4, "ArrowDown", sel), 3, "跳过第二期动作");
  assert.equal(actions.pickerNav(3, 4, "ArrowDown", sel), 0, "循环回可选");
  assert.equal(actions.pickerNav(0, 4, "ArrowUp", sel), 3);
  assert.equal(actions.pickerNav(0, 0, "ArrowDown", sel), null);
  assert.equal(actions.pickerNav(0, 4, "Enter", sel), null);
});

test("enter respects row states and required params (1, 3)", () => {
  setup();
  const close = actions.findPreset("browser.close_tab");
  const site = actions.findPreset("open.website");
  const cheat = actions.findPreset("mousetroke.cheatsheet");
  assert.equal(actions.presetPickResult(close, false, {}), "pick");
  assert.equal(actions.presetPickResult(site, false, {}), "form");
  assert.equal(actions.presetPickResult(site, true, { url: "https://a.com" }), "apply-form");
  assert.equal(actions.presetPickResult(cheat, false, {}), "blocked");
  assert.equal(actions.presetPickResult(cheat, true, {}), "blocked");
  // 切换高亮行时不能提交另一个动作：formOpenForIt=false 时带参数动作只展开表单
  assert.equal(actions.presetPickResult(site, false, { url: "https://a.com" }), "form", "高亮别的动作时 Enter 不提交");
});

test("unsupported and pending actions render unselectable with their reasons (12)", () => {
  platform.initPlatform("macos");
  actions.initActionCatalog(fixture);
  const clipboard = actions.findPreset("desktop.clipboard_history");
  assert.equal(actions.presetPlatformStatus(clipboard), "unsupported");
  assert.equal(actions.presetStatusReason(clipboard), "preset.status.windows_only");
  const html = render(createElement(ActionPicker, { initialId: "mousetroke.cheatsheet", onPick() {}, onClose() {} }));
  const commonHtml = render(createElement(ActionPicker, { onPick() {}, onClose() {} }));
  assert.ok(commonHtml.includes(t("preset.status.windows_only")), "unsupported 行显示原因");
  assert.ok(html.includes(t("ui.actpick.soon")), "pending 行显示即将支持");
  assert.ok((html.match(/disabled=""/g) ?? []).length >= 1, "灰显的行都不可选");
  assert.ok(html.includes('aria-live="polite"'), "原因通过 aria-live 读出");
  const card = render(createElement(ActionEditor, { action: { type: "preset", id: "text.type" }, onChange() {} }));
  assert.ok(!card.includes(`>${t("ui.actpick.to_advanced")}</button>`), "原生动作不能转为高级动作");
  assert.ok(card.includes(t("ui.actpick.change")));
  assert.ok(card.includes(t("preset.status.native_only")));
  assert.equal(actions.presetCanExpand(actions.findPreset("text.type")), false);
  assert.equal(actions.presetCanExpand(actions.findPreset("browser.close_tab")), true);
  assert.equal(actions.presetCanExpand(clipboard), false);
});

test("param form renders by kind and blocks empty required params (1)", () => {
  setup();
  const typePreset = actions.findPreset("text.type");
  assert.deepEqual(actions.presetParamValues(typePreset), { text: "", method: "paste" });
  assert.deepEqual(actions.missingRequiredParams(typePreset, { text: "", method: "paste" }), ["text"]);
  assert.deepEqual(actions.missingRequiredParams(typePreset, { text: "收到", method: "paste" }), []);
  const site = actions.findPreset("open.website");
  assert.deepEqual(actions.missingRequiredParams(site, { url: "https://mousetroke.com/" }), []);
  const html = render(createElement(ActionPicker, { initialId: "text.type", onPick() {}, onClose() {} }));
  assert.ok(html.includes(t("preset.param.text")));
  assert.ok(html.includes(t("preset.choice.paste")));
  assert.ok(html.includes(t("preset.choice.type")));
  assert.ok(html.includes(t("ui.actpick.fill_required")), "页脚提示填写必填项");
  assert.ok(html.includes(t("ui.actpick.required")), "字段旁显示必填");
  assert.match(html, /class="btn primary small" disabled/, "必填未填时不能确认");
  // 字符串参数提交前 trim
  assert.deepEqual(actions.cleanPresetParams(site, { url: "  https://a.com/  " }), { url: "https://a.com/" });
  assert.deepEqual(actions.cleanPresetParams(typePreset, { text: "  收到 ", method: "paste" }), { text: "  收到 ", method: "paste" });
  assert.deepEqual(actions.cleanPresetParams(typePreset, { text: "  ", method: "type" }), { text: "  ", method: "type" }, "保留字面空白");
});

test("searching across categories shows a hint (7)", () => {
  setup();
  const html = render(createElement(ActionPicker, { onPick() {}, onClose() {} }));
  assert.ok(!html.includes(t("ui.actpick.search_all_cats")), "无搜索词时不提示");
  assert.ok(html.includes(t("preset.category.common")) && html.includes(t("preset.category.mousetroke")), "分类 chips 存在");
  assert.ok(html.includes(t("ui.actpick.cat_advanced")));
});

test("initialId form stays open on mount and Esc would close form first (8, 17)", () => {
  setup();
  const html = render(createElement(ActionPicker, { initialId: "open.website", onPick() {}, onClose() {} }));
  assert.ok(html.includes(t("preset.param.url")), "initialId 直接展开表单（不被挂载 effect 清掉）");
  assert.ok(html.includes('value="https://mousetroke.com/"'), "默认值预填");
  // "更换"定位到当前动作分类
  const card = render(createElement(ActionEditor, { action: { type: "preset", id: "open.website", params: { url: "https://x.com" } }, onChange() {} }));
  assert.ok(card.includes(t("ui.actpick.change")), "卡片有更换按钮");
  assert.equal(actions.categoryOfPreset(actions.findPreset("open.website"), true), "common");
  assert.equal(actions.categoryOfPreset(actions.findPreset("mousetroke.cheatsheet"), false), "mousetroke");
});

test("choice params display option labels, not internal values (10)", () => {
  setup();
  const typePreset = actions.findPreset("text.type");
  assert.equal(actions.presetParamDisplay(typePreset, "method", "paste"), t("preset.choice.paste"));
  assert.equal(actions.presetParamDisplay(typePreset, "method", "type"), t("preset.choice.type"));
  assert.equal(actions.presetParamDisplay(typePreset, "text", "hello"), "hello");
});

test("empty gesture names are filled with the picked action name (A)", () => {
  setup();
  const binding = { name: "", pattern: [{ type: "press", key: "mouse:right" }], action: { type: "key_sequence", keys: "" }, execute_on_match: false, hide_label: false, secured: false };
  const apply = (b, action) => {
    let next = null;
    const tree = Editor({ binding: b, bindingKey: "k", conflictWith: [], otherBindings: [], onChange: (f) => (next = f(b)), onDuplicate() {}, onDelete() {}, onClose() {} });
    const ae = findAll(tree, (e) => e.type?.name === "ActionEditor").at(0);
    ae.props.onChange(action);
    return next;
  };
  const named = apply(binding, { type: "preset", id: "browser.close_tab" });
  assert.equal(named.name, t("preset.action.browser.close_tab.name"));
  assert.equal(named.lastActionName, t("preset.action.browser.close_tab.name"));
  assert.equal(named.action.id, "browser.close_tab");
  assert.equal(apply(binding, { type: "preset", id: "text.search" }).name.endsWith("…"), false);
  // 已有名字时不覆盖
  const kept = apply({ ...binding, name: "我的手势" }, { type: "preset", id: "browser.close_tab" });
  assert.equal(kept.name, "我的手势");
  assert.equal(kept.lastActionName, undefined);
  // 默认名也会被替换
  const def = apply({ ...binding, name: t("ui.app.new_gesture") }, { type: "preset", id: "browser.close_tab" });
  assert.equal(def.name, t("preset.action.browser.close_tab.name"));
  // 上一次自动填入的动作名跟随新动作
  const follow = apply({ ...named }, { type: "preset", id: "open.website" });
  assert.equal(follow.name, t("preset.action.open.website.name"));
  assert.equal(follow.lastActionName, t("preset.action.open.website.name"));
  // 列表摘要显示动作名而不是 ID
  assert.equal(keys.actionSummary({ type: "preset", id: "browser.close_tab" }), t("preset.action.browser.close_tab.name"));
});

test("advanced editor hides type bar behind a folded details (B)", () => {
  setup();
  const html = render(createElement(ActionEditor, { action: { type: "key_sequence", keys: "^c" }, onChange() {} }));
  assert.ok(html.includes(t("ui.actpick.change")), "顶部有更换动作按钮");
  assert.ok(html.includes(t("ui.actpick.change_type")), "类型条收进折叠区域");
  assert.ok(html.includes("<details"), "用 details/summary 折叠");
});

test("unselected action shows a prominent pick button and hint (A)", () => {
  setup();
  const html = render(createElement(ActionEditor, { action: { type: "key_sequence", keys: "" }, unselected: true, onChange() {} }));
  assert.ok(html.includes(t("ui.actpick.empty_hint")), "未选择时显示提示");
  assert.ok(html.includes(t("ui.actpick.choose")), "未选择时显示选择动作按钮");
  assert.ok(!html.includes(t("ui.actpick.change_type")), "未选择时不显示折叠的类型条");
});


async function mount(element) {
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  await act(async () => root.render(element));
  return {
    el: container,
    async render(element) { await act(async () => root.render(element)); },
    async close() { await act(async () => root.unmount()); container.remove(); },
  };
}
async function click(el) { assert.ok(el); await act(async () => el.click()); }
async function key(el, value, modifiers = {}) { assert.ok(el); await act(async () => el.dispatchEvent(new dom.window.KeyboardEvent("keydown", { key: value, bubbles: true, cancelable: true, ...modifiers }))); }
async function input(el, value) {
  await act(async () => {
    Object.getOwnPropertyDescriptor(dom.window.HTMLInputElement.prototype, "value").set.call(el, value);
    el.dispatchEvent(new dom.window.Event("input", { bubbles: true }));
  });
}
const button = (el, text) => [...el.querySelectorAll("button")].find((b) => b.textContent === text);

async function mountGestureEditor(binding = keys.newBinding(), fromDisk = false) {
  let store;
  function Harness() {
    store = useConfigStore({ scopes: [{ id: "g", bindings: fromDisk ? [binding] : [] }] });
    if (!store.config.scopes[0].bindings.length) return null;
    return createElement(Editor, {
      binding: store.config.scopes[0].bindings[0], bindingKey: "gesture", conflictWith: [], otherBindings: [],
      onChange: (f, coalesce) => store.edit((c) => updateBinding(c, 0, 0, f), coalesce),
      onDuplicate() {}, onDelete() {}, onClose() {},
    });
  }
  const view = await mount(createElement(Harness));
  if (!fromDisk) await act(async () => store.edit((c) => updateScope(c, 0, (s) => ({ ...s, bindings: [binding] }))));
  return { ...view, get store() { return store; }, get binding() { return store.config.scopes[0].bindings[0]; } };
}

test("draft completion requires a recorded pattern and a selected action", () => {
  const press = { type: "press", key: "mouse:right" };
  for (const pattern of [[], [press], [press, { type: "trace", points: [] }], [press, { type: "trace", points: [0, 0] }], [press, { type: "shape", templates: [[]] }]]) {
    assert.equal(keys.hasConflictPattern(pattern), false);
    const b = withBindingIds({ scopes: [{ bindings: [{ ...keys.newBinding(), pattern, actionUnselected: false }] }] }).scopes[0].bindings[0];
    assert.equal(b.draft, true);
    assert.equal(b.disabled, true);
  }
  for (const pattern of [[press, { type: "press", key: "kb:82" }], [press, { type: "wheel", horizontal: false }], [{ type: "edge", edge: 1 }], [press, { type: "trace", points: [0, 0, 1, 0] }], [press, { type: "shape", templates: [[0, 0, 1, 0]] }]]) {
    assert.equal(keys.hasConflictPattern(pattern), true);
    const config = { scopes: [{ bindings: [{ ...keys.newBinding(), pattern }] }] };
    assert.equal(withBindingIds(config).scopes[0].bindings[0].draft, true);
    config.scopes[0].bindings[0].actionUnselected = false;
    const b = withBindingIds(config).scopes[0].bindings[0];
    assert.equal(b.draft, undefined);
    assert.equal(b.disabled, undefined);
  }
});

test("editor warns on drafts until both the pattern and action are complete", async () => {
  setup();
  const view = await mountGestureEditor();
  try {
    assert.equal(view.el.querySelector('.pattern-editor + .field-warn')?.textContent, t("ui.editor.incomplete_pattern"));
    await click(button(view.el, t("ui.pattern.add_step")));
    await click(button(view.el, t("ui.pattern.wheel_up")));
    assert.equal(view.binding.draft, true);
    await act(async () => view.store.edit((c) => updateBinding(c, 0, 0, (b) => ({ ...b, actionUnselected: false }))));
    assert.equal(view.el.querySelector('.pattern-editor + .field-warn'), null);
    assert.equal(view.binding.disabled, undefined);
    assert.equal(view.binding.draft, undefined);
    await click(view.el.querySelector('.chip-x'));
    assert.equal(view.binding.disabled, true, "界面新建绑定移除最后一个有效步骤后重新进入草稿");
    assert.equal(view.binding.draft, true);
    assert.equal(view.el.querySelector('.pattern-editor + .field-warn')?.textContent, t("ui.editor.incomplete_pattern"));
  } finally { await view.close(); }
});

test("draft persists disabled across JSON reload and completing it enables without a manual toggle", async () => {
  setup();
  const config = { scopes: [{ id: "g", bindings: [{ ...keys.newBinding(), action: { type: "key_sequence", keys: "^v", activate_target: false }, actionUnselected: false, lastActionName: "action" }] }] };
  const before = structuredClone(config);
  const saved = JSON.parse(JSON.stringify(persistedConfig(config)));
  const binding = saved.scopes[0].bindings[0];
  assert.equal(binding.disabled, true);
  assert.equal(binding.draft, true);
  for (const field of ["uiId", "actionUnselected", "lastActionName", "loaded", "autoDisabled"]) assert.equal(field in binding, false);
  assert.deepEqual(config, before);
  const view = await mountGestureEditor(binding, true);
  try {
    assert.equal(view.binding.loaded, true);
    assert.equal(view.binding.disabled, true);
    assert.equal(view.binding.draft, true);
    await click(button(view.el, t("ui.pattern.add_step")));
    await click(button(view.el, t("ui.pattern.wheel_up")));
    assert.equal(view.binding.disabled, undefined);
    assert.equal(view.binding.draft, undefined);
    assert.equal(persistedConfig(view.store.config).scopes[0].bindings[0].disabled, undefined);
    await act(async () => view.store.undo());
    assert.equal(view.binding.disabled, true);
    assert.equal(view.binding.draft, true);
    await act(async () => view.store.redo());
    assert.equal(view.binding.disabled, undefined);
    assert.equal(view.binding.draft, undefined);
  } finally { await view.close(); }
});

test("manual disable in the editor stays off after pattern completion until manually enabled", async () => {
  setup();
  const view = await mountGestureEditor({ ...keys.newBinding(), action: { type: "preset", id: "browser.close_tab" }, actionUnselected: false });
  try {
    const toggle = view.el.querySelector('.editor-enable input');
    assert.equal(toggle.checked, true);
    await click(toggle);
    assert.equal(view.binding.disabled, true);
    assert.equal(view.binding.draft, undefined);
    await input(view.el.querySelector('.editor-name'), "Manually disabled");
    assert.equal(toggle.checked, false);
    assert.equal(view.binding.disabled, true);
    assert.equal(view.binding.draft, undefined);
    await click(button(view.el, t("ui.pattern.add_step")));
    await click(button(view.el, t("ui.pattern.wheel_up")));
    assert.equal(view.binding.disabled, true);
    assert.equal(view.binding.draft, undefined);
    assert.equal(toggle.checked, false);
    assert.equal(persistedConfig(view.store.config).scopes[0].bindings[0].disabled, true);
    await click(toggle);
    assert.equal(toggle.checked, true);
    assert.equal(view.binding.disabled, undefined);
    assert.equal(view.binding.draft, undefined);
    assert.equal(persistedConfig(view.store.config).scopes[0].bindings[0].disabled, undefined);
  } finally { await view.close(); }
});

for (const selectAction of [false, true]) test(`editor toggling an incomplete draft off and on restores its draft state with action selected: ${selectAction}`, async () => {
  setup();
  const view = await mountGestureEditor();
  try {
    if (selectAction) {
      await click(button(view.el, t("ui.actpick.choose")));
      await click([...document.querySelectorAll('.act-list button')].find((b) => b.textContent.startsWith(t("preset.action.browser.close_tab.name"))));
      assert.equal(view.binding.action.id, "browser.close_tab");
    }
    const toggle = view.el.querySelector('.editor-enable input');
    assert.equal(toggle.checked, true);
    await click(toggle);
    assert.equal(toggle.checked, false);
    assert.equal(view.binding.disabled, true);
    assert.equal(view.binding.draft, undefined);
    assert.equal(view.el.querySelector('.pattern-editor + .field-warn'), null);
    const disabled = persistedConfig(view.store.config).scopes[0].bindings[0];
    assert.equal(disabled.disabled, true);
    assert.equal(disabled.draft, undefined);
    await click(toggle);
    assert.equal(toggle.checked, true);
    assert.equal(view.binding.disabled, true);
    assert.equal(view.binding.draft, true);
    assert.equal(view.el.querySelector('.pattern-editor + .field-warn')?.textContent, t("ui.editor.incomplete_pattern"));
    const before = structuredClone(view.store.config);
    const saved = persistedConfig(view.store.config).scopes[0].bindings[0];
    assert.equal(saved.disabled, true);
    assert.equal(saved.draft, true);
    assert.equal("actionUnselected" in saved, false);
    assert.deepEqual(view.store.config, before);
  } finally { await view.close(); }
});

test("normalization and persistence protect incomplete UI bindings regardless of action selection", () => {
  setup();
  const press = { type: "press", key: "mouse:right" };
  for (const pattern of [[], [press], [press, { type: "trace", points: [] }], [press, { type: "shape", templates: [[]] }]]) {
    for (const [actionUnselected, action] of [
      [true, { type: "preset", id: "browser.close_tab" }],
      [undefined, { type: "key_sequence", keys: "" }],
      [false, { type: "key_sequence", keys: "" }],
      [false, { type: "preset", id: "browser.close_tab" }],
      [undefined, { type: "key_sequence", keys: "^v" }],
    ]) {
      const config = { scopes: [{ bindings: [{ ...keys.newBinding(), pattern, action, actionUnselected, disabled: undefined, draft: undefined }] }] };
      const before = structuredClone(config);
      const normalized = withBindingIds(config).scopes[0].bindings[0];
      assert.equal(normalized.disabled, true);
      assert.equal(normalized.draft, true);
      const saved = persistedConfig(config).scopes[0].bindings[0];
      assert.equal(saved.disabled, true);
      assert.equal(saved.draft, true);
      for (const field of ["uiId", "actionUnselected", "lastActionName", "loaded"]) assert.equal(field in saved, false);
      assert.deepEqual(config, before);
    }
  }
});

test("loaded single presses stay enabled and a new edge without an action stays draft", async () => {
  setup();
  for (const source of ["initial", "load"]) for (const key of ["mouse:right", "mouse:x1", "mouse:x2", "mouse:middle"]) {
    const binding = { ...keys.newBinding(), pattern: [{ type: "press", key }], action: { type: "key_sequence", keys: "^v" }, draft: undefined, disabled: undefined, actionUnselected: undefined };
    const before = structuredClone(binding);
    const view = await mountGestureEditor(binding, source === "initial");
    try {
      if (source === "load") await act(async () => view.store.load({ scopes: [{ id: "g", bindings: [binding] }] }));
      assert.equal(view.binding.loaded, true);
      assert.equal(view.binding.disabled, undefined);
      assert.equal(view.binding.draft, undefined);
      await act(async () => view.store.edit((c) => updateBinding(c, 0, 0, (b) => ({ ...b, name: "renamed" }))));
      assert.equal(view.binding.loaded, true);
      assert.equal(view.binding.disabled, undefined);
      assert.equal(view.binding.draft, undefined);
      assert.equal(withBindingIds(view.store.config).scopes[0].bindings[0].draft, undefined);
      assert.equal(view.el.querySelector('.field-warn'), null);
      const saved = persistedConfig(view.store.config).scopes[0].bindings[0];
      assert.ok(!saved.disabled);
      assert.ok(!saved.draft);
      assert.equal("loaded" in saved, false);
      assert.deepEqual(binding, before);
    } finally { await view.close(); }
  }
  const edge = { ...keys.newBinding(), pattern: [{ type: "edge", edge: 1 }] };
  const saved = persistedConfig(withBindingIds({ scopes: [{ bindings: [edge] }] })).scopes[0].bindings[0];
  assert.equal(saved.disabled, true);
  assert.equal(saved.draft, true);
  assert.equal(saved.action.keys, "");
  const reloaded = withBindingIds(JSON.parse(JSON.stringify({ scopes: [{ bindings: [saved] }] })));
  assert.equal(reloaded.scopes[0].bindings[0].actionUnselected, true);
  assert.equal(reloaded.scopes[0].bindings[0].draft, true);
  assert.equal(persistedConfig(reloaded).scopes[0].bindings[0].disabled, true);
});

test("current preset selects its actual row and scrolls into view (UI 1)", async () => {
  setup();
  const action = { type: "preset", id: "open.website", params: { url: "https://example.org/current" } };
  assert.ok(actions.pickerInitial(action).active > 0);
  const view = await mount(createElement(ActionPicker, { current: action, onPick() {}, onClose() {} }));
  assert.equal(view.el.querySelector('[aria-selected="true"] .act-name').textContent, t("preset.action.open.website.name") + "…");
  assert.equal(scrolled, view.el.querySelector('[aria-selected="true"]'));
  const other = view.el.querySelector('[role="option"]');
  await act(async () => other.dispatchEvent(new dom.window.MouseEvent("mouseover", { bubbles: true })));
  assert.equal(view.el.querySelector('[aria-selected="true"] .act-name').textContent, t("preset.action.open.website.name") + "…", "静止指针不能覆盖打开时的当前动作");
  assert.equal(view.el.querySelector('[aria-pressed="true"]').textContent, t("preset.category.common"));
  await view.close();
  const nonCommon = actions.pickerInitial({ type: "preset", id: "text.search" });
  assert.equal(nonCommon.category, "text");
});

test("bare key searches include English text matches and exact shortcuts together", async () => {
  setup();
  const view = await mount(createElement("div"));
  try {
    initLang({ setting: "en-US", url: "" });
    const close = actions.findPreset("browser.close_tab");
    const tab = { ...close, id: "browser.new_tab", name_key: "preset.action.browser.new_tab.name", aliases_key: "preset.action.browser.new_tab.aliases", windows: { status: "implemented", value: { type: "keys", value: "^t" } } };
    const snap = { ...close, id: "window.snap_left", name_key: "preset.action.window.snap_left.name", aliases_key: "preset.action.window.snap_left.aliases", windows: { status: "implemented", value: { type: "keys", value: "#{LEFT}" } } };
    const arrow = { ...close, id: "edit.left", name_key: "preset.action.edit.copy.name", aliases_key: "preset.action.edit.copy.aliases", windows: { status: "implemented", value: { type: "keys", value: "{LEFT}" } } };
    actions.initActionCatalog({ ...fixture, presets: [close, tab, snap, arrow] });
    assert.deepEqual(actions.visiblePresets("common", "tab").map((p) => p.id), [close.id, tab.id]);
    assert.deepEqual(actions.visiblePresets("common", "left").map((p) => p.id), [snap.id, arrow.id]);
    assert.deepEqual(actions.visiblePresets("common", "^t").map((p) => p.id), [tab.id]);
    assert.deepEqual(actions.visiblePresets("common", "win+left").map((p) => p.id), [snap.id]);
  } finally {
    initLang({ setting: "zh-CN", url: "" });
    await view.close();
    setup();
  }
});

test("shortcut matching compares complete modifier sets and main keys (UI 2)", () => {
  setup();
  const desktop = { ...actions.findPreset("browser.close_tab"), id: "desktop.prev", aliases_key: "preset.action.desktop.prev.aliases", windows: { status: "implemented", value: { type: "keys", value: "^#{LEFT}" } } };
  for (const query of ["ctrl+w", "ctrlw", "Ctrl + W"]) assert.equal(actions.matchPreset(desktop, query), false);
  for (const query of ["ctrl+win+left", "win+ctrl+left", "ctrl+win+←", "^#{LEFT}"]) assert.equal(actions.matchPreset(desktop, query), true);
  assert.equal(actions.matchPreset(desktop, "cmd+w"), false);
  const plain = { ...desktop, id: "plain", windows: { status: "implemented", value: { type: "keys", value: "{F5}" } } };
  const modified = { ...plain, id: "modified", windows: { status: "implemented", value: { type: "keys", value: "^{F5}" } } };
  assert.equal(actions.matchPreset(plain, "F5"), true);
  assert.equal(actions.matchPreset(modified, "F5"), false);
  const shifted = { ...plain, id: "shifted", windows: { status: "implemented", value: { type: "keys", value: "#+5" } } };
  for (const query of ["#+5", "win+shift+5", "shift+win+5"]) assert.equal(actions.matchPreset(shifted, query), true);
  assert.equal(actions.matchPreset(shifted, "#5"), false);
});

test("initialId and row activation merge existing params over defaults (C3)", async () => {
  setup();
  const current = { type: "preset", id: "text.type", params: { text: "my current text" } };
  assert.deepEqual(actions.presetParamValues(actions.findPreset(current.id), current), { text: "my current text", method: "paste" });
  const view = await mount(createElement(ActionPicker, { current, initialId: current.id, onPick() {}, onClose() {} }));
  assert.equal(view.el.querySelector("textarea").value, "my current text");
  await key(document.activeElement, "Escape");
  await click(view.el.querySelector('[aria-selected="true"] button'));
  assert.equal(view.el.querySelector("textarea").value, "my current text");
  await view.close();
});

test("hidden custom URL is absent from form, validation, summary and saved params (UI 5, 6)", async () => {
  setup();
  const preset = actions.findPreset("text.search");
  const values = { engine: "baidu", url: "https://stale.invalid/" };
  assert.deepEqual(actions.cleanPresetParams(preset, values), { engine: "baidu" });
  assert.deepEqual(actions.persistedAction({ type: "preset", id: preset.id, params: { ...values, future: [1] } }).params, { engine: "baidu", future: [1] });
  assert.deepEqual(actions.missingRequiredParams(preset, values), []);
  const required = { ...preset, params: preset.params.map((p) => p.name === "url" ? { ...p, default: { kind: "literal", value: "" } } : p) };
  assert.deepEqual(actions.missingRequiredParams(required, { engine: "baidu" }), []);
  assert.deepEqual(actions.missingRequiredParams(required, { engine: "custom" }), ["url"]);
  const current = { type: "preset", id: preset.id, params: values };
  const card = render(createElement(ActionEditor, { action: current, onChange() {} }));
  assert.ok(!card.includes("stale.invalid"));
  const picked = [];
  const view = await mount(createElement(ActionPicker, { current, initialId: preset.id, onPick: (a) => picked.push(a), onClose() {} }));
  assert.equal(view.el.querySelector('#param-url'), null);
  assert.equal(document.activeElement.textContent, t("preset.choice.baidu"));
  await click(button(view.el, t("ui.actpick.apply")));
  assert.deepEqual(picked[0].params, { engine: "baidu" });
  await click(button(view.el, t("preset.choice.custom")));
  assert.ok(view.el.querySelector('#param-url'));
  const insert = button(view.el, t("ui.actpick.insert_selected_text"));
  assert.ok(insert);
  const field = view.el.querySelector('#param-url');
  assert.equal(field.value, "https://stale.invalid/");
  await input(field, "https://example.org/?q=");
  field.setSelectionRange(field.value.length, field.value.length);
  await click(insert);
  await click(button(view.el, t("ui.actpick.apply")));
  assert.equal(picked[1].params.url, "https://example.org/?q=%MS_SELECTED_TEXT%");
  await view.close();
});

test("Escape first closes picker form, next closes picker; rerender preserves focus (C5)", async () => {
  setup();
  let closed = 0;
  const props = { initialId: "open.website", onPick() {}, onClose: () => closed++ };
  const view = await mount(createElement(ActionPicker, props));
  const field = view.el.querySelector('#param-url');
  field.focus(); field.setSelectionRange(3, 3);
  await view.render(createElement(ActionPicker, { ...props, onClose: () => closed++ }));
  assert.equal(document.activeElement, field);
  assert.equal(field.selectionStart, 3);
  await key(field, "Escape");
  assert.equal(closed, 0);
  assert.equal(view.el.querySelector('.act-form'), null);
  assert.equal(document.activeElement, view.el.querySelector('.search'));
  await key(document.activeElement, "Escape");
  assert.equal(closed, 1);
  await view.close();
  const modal = await mount(createElement(Modal, { title: "test", onClose: () => closed++ }, createElement("input")));
  await key(document.activeElement, "Escape");
  assert.equal(closed, 2);
  await modal.close();
});

test("Enter on category buttons never applies highlighted action (C8)", async () => {
  setup();
  let picked = 0;
  const view = await mount(createElement(ActionPicker, { onPick: () => picked++, onClose() {} }));
  const cat = button(view.el, t("preset.category.text"));
  cat.focus();
  await key(cat, "Enter");
  await key(cat, " ");
  assert.equal(picked, 0);
  // DOM 不合成浏览器默认点击；显式点击校验按钮自身的动作。
  await click(cat);
  assert.equal(cat.getAttribute("aria-pressed"), "true");
  view.el.querySelector('.search').focus();
  await input(document.activeElement, "^w");
  await key(document.activeElement, "Enter");
  assert.equal(picked, 1);
  await view.close();
});

test("hover and keyboard share one active row; search clears category selection (UI 10, 11)", async () => {
  setup();
  const view = await mount(createElement(ActionPicker, { onPick() {}, onClose() {} }));
  const search = view.el.querySelector('.search');
  await key(search, "ArrowDown");
  const row = view.el.querySelectorAll('[role="option"]')[2];
  await act(async () => row.dispatchEvent(new dom.window.MouseEvent("mousemove", { bubbles: true })));
  assert.equal(view.el.querySelectorAll('li.on').length, 1);
  assert.equal(view.el.querySelector('li.on'), row);
  await input(search, "website");
  assert.equal(view.el.querySelectorAll('.act-cat[aria-pressed="true"]').length, 0);
  await click(button(view.el, t("preset.category.text")));
  assert.equal(search.value, "");
  assert.equal(view.el.querySelector('.act-cat[aria-pressed="true"]').textContent, t("preset.category.text"));
  await view.close();
});

test("empty key sequence stays editable, including after clearing keys (C2)", async () => {
  setup();
  function Harness() {
    const [action, change] = useState({ type: "key_sequence", keys: "^c" });
    return createElement(ActionEditor, { action, onChange: change });
  }
  const view = await mount(createElement(Harness));
  const field = view.el.querySelector('.keys-input input');
  await input(field, "");
  assert.equal(view.el.querySelector('.keys-input input'), field);
  assert.equal(field.value, "");
  assert.ok(!view.el.textContent.includes(t("ui.actpick.empty_hint")));
  await view.close();
  function Unselected() {
    const [binding, update] = useState(keys.newBinding());
    return createElement(ActionEditor, { action: binding.action, unselected: binding.actionUnselected, autoPick: true, onChange: (action) => update({ ...binding, action, actionUnselected: false }) });
  }
  const fresh = await mount(createElement(Unselected));
  await click([...fresh.el.querySelectorAll('.act-list button')].find((b) => b.textContent.startsWith(t("action.key_sequence"))));
  assert.ok(fresh.el.querySelector('.keys-input input'));
  assert.ok(!fresh.el.querySelector('[role="dialog"]'));
  await fresh.close();
});

test("auto picker request is consumed once by stable binding identity (C9)", async () => {
  setup();
  const original = keys.newBinding();
  const later = keys.newBinding();
  const config = { scopes: [{ id: "g", bindings: [original, later] }] };
  const identified = withBindingIds(config);
  assert.notEqual(identified.scopes[0].bindings[0].uiId, identified.scopes[0].bindings[1].uiId);
  assert.equal(withBindingIds({ scopes: [{ id: "g", bindings: [later, { ...original, name: "edited" }] }] }).scopes[0].bindings[1].uiId, original.uiId);
  let shows = 0;
  function Harness({ binding }) {
    const [pending, setPending] = useState(original.uiId);
    return createElement(ActionEditor, { key: binding.uiId, action: binding.action, unselected: true, autoPick: pending === binding.uiId, onPickerShown: () => { shows++; setPending(null); }, onChange() {} });
  }
  const view = await mount(createElement(Harness, { binding: original }));
  assert.equal(shows, 1);
  assert.ok(view.el.querySelector('[role="dialog"]'));
  await key(document.activeElement, "Escape");
  await view.render(createElement(Harness, { binding: later }));
  await view.render(createElement(Harness, { binding: original }));
  assert.equal(shows, 1);
  assert.equal(view.el.querySelector('[role="dialog"]'), null);
  const persisted = persistedConfig(identified);
  assert.equal(persisted.scopes[0].bindings[0].uiId, undefined);
  assert.equal(persisted.scopes[0].bindings[0].actionUnselected, undefined);
  assert.deepEqual(persisted.scopes[0].bindings[0].action, { type: "key_sequence", keys: "", activate_target: false });
  await view.close();
});

test("unselected rows are muted and incomplete gestures do not conflict (C2, UI 7)", () => {
  setup();
  const binding = { ...keys.newBinding(), disabled: false, draft: undefined };
  assert.equal(keys.conflicts([binding, binding]).size, 0);
  const trace = { ...binding, pattern: [{ type: "press", key: "mouse:right" }, { type: "trace", points: [] }] };
  assert.equal(keys.conflicts([trace, trace]).size, 0);
  const edge = { ...binding, pattern: [{ type: "edge", edge: 1 }] };
  assert.equal(keys.conflicts([edge, edge]).size, 2);
  const html = render(createElement(BindingList, { scope: { id: "g", kind: "global", name: "g", bindings: [binding] }, tab: "gesture", filter: "all", selected: null, clash: new Map(), onSelect() {}, onTab() {}, onFilter() {}, onAdd() {}, onDelete() {}, onDuplicate() {}, onToggle() {}, onScopeChange() {}, onDeleteScope() {} }));
  assert.ok(html.includes('class="row-action-text muted"'));
  assert.ok(html.includes(t("ui.actpick.unselected")));
});

test("new gesture rows show no stroke yet with or without an action selected", () => {
  setup();
  const binding = keys.newBinding();
  assert.equal(binding.disabled, true);
  assert.equal(binding.draft, true);
  const props = { tab: "gesture", filter: "all", selected: null, clash: new Map(), onSelect() {}, onTab() {}, onFilter() {}, onAdd() {}, onDelete() {}, onDuplicate() {}, onToggle() {}, onScopeChange() {}, onDeleteScope() {} };
  for (const actionUnselected of [true, false]) {
    for (const pattern of [[], binding.pattern]) {
      for (const state of [{ disabled: true, draft: true }]) {
        const html = render(createElement(BindingList, { ...props, scope: { id: "g", kind: "global", name: "g", bindings: [{ ...binding, ...state, pattern, actionUnselected }] } }));
        assert.ok(html.includes('class="row-action-text muted"'));
        assert.ok(html.includes(t("ui.list.no_stroke")));
        assert.equal(html.includes(t("ui.actpick.unselected")), actionUnselected);
        assert.ok(!html.includes('filter-off'), "等待画轨迹与手动停用分开显示");
        const row = JSDOM.fragment(html).querySelector(".row");
        const badges = row.querySelectorAll(".row-name .badge");
        assert.deepEqual([...badges].map((badge) => badge.textContent), [t("ui.list.inactive_badge")]);
        assert.equal(badges[0].getAttribute("data-tip"), t("ui.list.inactive_tip"));
        assert.equal(row.querySelector(".switch").checked, true);
      }
    }
  }
  const disabled = render(createElement(BindingList, { ...props, scope: { id: "g", kind: "global", name: "g", bindings: [{ ...binding, draft: undefined }] } }));
  assert.ok(disabled.includes('filter-off'));
  const disabledBadges = JSDOM.fragment(disabled).querySelectorAll(".row-name .badge");
  assert.deepEqual([...disabledBadges].map((badge) => badge.textContent), [t("ui.list.off_badge")]);
  for (const step of [{ type: "press", key: "kb:82" }, { type: "trace", points: [0, 0, 1, 0] }]) {
    const html = render(createElement(BindingList, { ...props, scope: { id: "g", kind: "global", name: "g", bindings: [{ ...binding, disabled: false, draft: undefined, pattern: [...binding.pattern, step] }] } }));
    assert.ok(!html.includes(t("ui.list.no_stroke")));
    assert.equal(JSDOM.fragment(html).querySelector(".row-name .badge"), null);
  }
});


for (const recovery of ["defaults", "import"]) test(`unreadable config blocks all automatic and close-time saves until explicit ${recovery} recovery (C10)`, async () => {
  setup();
  const { App } = await server.ssrLoadModule("/src/App.tsx");
  const config = JSON.parse(readFileSync(new URL("../../../mousetroke-core/assets/default-config.json", import.meta.url), "utf8"));
  config.settings = JSON.parse(readFileSync(new URL("../../../../tools/uitest/config.json", import.meta.url), "utf8")).settings;
  config.scopes = config.scopes.slice(0, 1);
  const listeners = new Map();
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
  window.__TAURI_INTERNALS__ = {
    metadata: { currentWindow: { label: "main" }, currentWebview: { label: "main" } },
    transformCallback: (fn) => fn,
    invoke: async (cmd, args) => {
      if (cmd === "plugin:event|listen") { listeners.set(args.event, args.handler); return listeners.size; }
      return null;
    },
  };
  const originalApi = { ...api };
  const saved = [];
  const opened = [];
  let recovered = 0;
  const diskDir = mkdtempSync(join(tmpdir(), "mousetroke-ui-recovery-"));
  const diskPath = join(diskDir, "config.json");
  writeFileSync(diskPath, "broken JSON");
  const imported = structuredClone(config);
  imported.scopes[0].bindings[0].name = "Imported binding";
  Object.assign(api, {
    initState: async () => ({ platform: "windows", config, loadError: "line 2 column 14", configPath: "config.json", engine: { running: true, autostart: false, exeFound: true }, systemLangs: ["zh-CN"], actionCatalog: fixture }),
    actionCatalog: async () => fixture,
    setUiLang: async () => {},
    appIcons: async () => ({}),
    saveConfig: async (config) => saved.push(config),
    openPath: async (what) => opened.push(what),
    recoverConfig: async (incoming) => {
      recovered++;
      assert.equal(incoming === undefined, recovery === "defaults");
      writeFileSync(diskPath, JSON.stringify(persistedConfig(incoming ?? config)));
      return incoming ?? config;
    },
    pickFile: async () => "import.json",
    loadConfigFile: async () => imported,
  });
  const view = await mount(createElement(App));
  try {
    assert.ok(view.el.querySelector('.banner').textContent.includes("line 2 column 14"));
    await click(view.el.querySelector('.rows .row .switch'));
    await act(async () => new Promise((resolve) => setTimeout(resolve, 420)));
    assert.equal(saved.length, 0);
    await act(async () => listeners.get("tauri://close-requested")({ event: "tauri://close-requested", id: 1 }));
    assert.equal(saved.length, 0);
    await click(button(view.el, t("ui.app.open_config_folder")));
    assert.deepEqual(opened, ["config_dir"]);
    if (recovery === "defaults") await click(button(view.el, t("ui.app.recover_defaults")));
    else {
      await click([...view.el.querySelectorAll('.nav-item')].find((b) => b.textContent === t("ui.sidebar.data")));
      await click(button(view.el, t("ui.data.import_file")));
      assert.equal(JSON.parse(readFileSync(diskPath, "utf8")).scopes[0].bindings[0].name, "Imported binding");
      assert.ok(!button(view.el.querySelector('.toast'), t("common.undo")), "恢复导入不提供撤销通知");
    }
    assert.equal(recovered, 1);
    assert.equal(view.el.querySelector('.banner'), null);
    await click(view.el.querySelector('.rows .row .switch'));
    await act(async () => new Promise((resolve) => setTimeout(resolve, 420)));
    assert.ok(saved.length >= 1);
    const saves = saved.length;
    await click(view.el.querySelector('.rows .row .switch'));
    await act(async () => listeners.get("tauri://close-requested")({ event: "tauri://close-requested", id: 1 }));
    assert.equal(saved.length, saves + 1, "恢复后关闭窗口前可以保存");
  } finally {
    await view.close();
    rmSync(diskDir, { recursive: true, force: true });
    Object.assign(api, originalApi);
  }
});

for (const togglePath of ["row", "editor"]) for (const selectAction of [false, true]) test(`${togglePath} toggles preserve draft state and persistence safety with action selected: ${selectAction}`, async () => {
  setup();
  const { App } = await server.ssrLoadModule("/src/App.tsx");
  const config = JSON.parse(readFileSync(new URL("../../../mousetroke-core/assets/default-config.json", import.meta.url), "utf8"));
  config.settings = JSON.parse(readFileSync(new URL("../../../../tools/uitest/config.json", import.meta.url), "utf8")).settings;
  config.scopes = [{ ...config.scopes[0], bindings: [] }];
  const originalApi = { ...api };
  const originalInternals = window.__TAURI_INTERNALS__;
  const originalEvents = window.__TAURI_EVENT_PLUGIN_INTERNALS__;
  const saved = [], listeners = new Map();
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
  window.__TAURI_INTERNALS__ = {
    metadata: { currentWindow: { label: "main" }, currentWebview: { label: "main" } },
    transformCallback: (fn) => fn,
    invoke: async (cmd, args) => {
      if (cmd === "plugin:event|listen") { listeners.set(args.event, args.handler); return listeners.size; }
      if (cmd === "save_config") saved.push(args.config);
      return null;
    },
  };
  Object.assign(api, {
    initState: async () => ({ platform: "windows", config, loadError: null, configPath: "config.json", engine: { running: true, autostart: false, exeFound: true }, systemLangs: ["zh-CN"], actionCatalog: fixture }),
    actionCatalog: async () => fixture,
    setUiLang: async () => {},
    appIcons: async () => ({}),
  });
  const view = await mount(createElement(App));
  try {
    await click(button(view.el, t("ui.list.add_gesture")));
    assert.ok(view.el.querySelector('.row-action-text.muted').textContent.includes(t("ui.list.no_stroke")));
    if (selectAction) await click([...view.el.querySelectorAll('.act-list button')].find((b) => b.textContent.startsWith(t("preset.action.browser.close_tab.name"))));
    else await key(document.activeElement, "Escape");
    assert.ok(view.el.querySelector('.pattern-editor + .field-warn'));
    await act(async () => new Promise((resolve) => setTimeout(resolve, 420)));
    assert.ok(saved.length > 0);
    const binding = saved.at(-1).scopes[0].bindings[0];
    if (selectAction) assert.equal(binding.action.id, "browser.close_tab");
    else assert.equal(binding.action.keys, "");
    assert.equal(binding.pattern.length, 1);
    assert.equal(binding.disabled, true);
    assert.equal(binding.draft, true);
    for (const field of ["uiId", "actionUnselected", "lastActionName", "loaded"]) assert.equal(field in binding, false);
    const toggle = view.el.querySelector(togglePath === "row" ? '.rows .row .switch' : '.editor-enable input');
    const badge = () => view.el.querySelector('.rows .row .row-name .badge')?.textContent;
    assert.equal(toggle.checked, true);
    assert.equal(badge(), t("ui.list.inactive_badge"));
    await click(toggle);
    assert.equal(toggle.checked, false);
    assert.equal(badge(), t("ui.list.off_badge"));
    await act(async () => listeners.get("tauri://close-requested")({ event: "tauri://close-requested", id: 1 }));
    assert.equal(saved.at(-1).scopes[0].bindings[0].disabled, true);
    assert.ok(!saved.at(-1).scopes[0].bindings[0].draft);
    await click(toggle);
    assert.equal(toggle.checked, true);
    assert.equal(badge(), t("ui.list.inactive_badge"));
    assert.ok(view.el.querySelector('.pattern-editor + .field-warn'));
    const saves = saved.length;
    await act(async () => new Promise((resolve) => setTimeout(resolve, 420)));
    assert.ok(saved.length > saves);
    assert.equal(saved.at(-1).scopes[0].bindings[0].disabled, true);
    assert.equal(saved.at(-1).scopes[0].bindings[0].draft, true);
    await click(toggle);
    await click(toggle);
    assert.equal(badge(), t("ui.list.inactive_badge"));
    const autosaves = saved.length;
    await act(async () => listeners.get("tauri://close-requested")({ event: "tauri://close-requested", id: 1 }));
    assert.equal(saved.length, autosaves + 1);
    assert.equal(saved.at(-1).scopes[0].bindings[0].pattern.length, 1);
    assert.equal(saved.at(-1).scopes[0].bindings[0].disabled, true);
    assert.equal(saved.at(-1).scopes[0].bindings[0].draft, true);
    if (!selectAction) return;
    await click(toggle);
    await click(button(view.el, t("ui.pattern.add_step")));
    await click(button(view.el, t("ui.pattern.wheel_up")));
    assert.equal(view.el.querySelector('.pattern-editor + .field-warn'), null);
    assert.equal(toggle.checked, false);
    await act(async () => listeners.get("tauri://close-requested")({ event: "tauri://close-requested", id: 1 }));
    assert.equal(saved.at(-1).scopes[0].bindings[0].disabled, true);
    await click(toggle);
    assert.equal(toggle.checked, true);
    assert.equal(badge(), undefined);
    await act(async () => listeners.get("tauri://close-requested")({ event: "tauri://close-requested", id: 1 }));
    assert.ok(!saved.at(-1).scopes[0].bindings[0].disabled, "完整模式允许手动启用并落盘");
    assert.ok(!saved.at(-1).scopes[0].bindings[0].draft);
    await click(view.el.querySelector('.chip-x'));
    assert.equal(badge(), t("ui.list.inactive_badge"));
    assert.ok(view.el.querySelector('.pattern-editor + .field-warn'));
    await act(async () => listeners.get("tauri://close-requested")({ event: "tauri://close-requested", id: 1 }));
    assert.equal(saved.at(-1).scopes[0].bindings[0].disabled, true);
    assert.equal(saved.at(-1).scopes[0].bindings[0].draft, true);
  } finally {
    await view.close();
    Object.assign(api, originalApi);
    window.__TAURI_INTERNALS__ = originalInternals;
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = originalEvents;
  }
});


test("Folder and File parameters dispatch to distinct native pickers (C6)", async () => {
  setup();
  const template = actions.findPreset("open.website");
  const paths = ["folder", "file"].map((kind) => ({ ...template, id: `open.${kind}`, params: [{ name: "path", label_key: "preset.param.path", kind: { type: "path", kind }, default: { kind: "literal", value: "" } }] }));
  actions.initActionCatalog({ ...fixture, presets: [...fixture.presets, ...paths] });
  const originalFolder = api.pickFolder, originalFile = api.pickFile;
  const calls = [];
  api.pickFolder = async () => { calls.push("folder"); return "chosen folder"; };
  api.pickFile = async (kind) => { calls.push(kind); return "chosen file"; };
  try {
    for (const kind of ["folder", "file"]) {
      const view = await mount(createElement(ActionPicker, { initialId: `open.${kind}`, onPick() {}, onClose() {} }));
      await click(button(view.el, t("common.browse")));
      assert.equal(view.el.querySelector('#param-path').value, `chosen ${kind}`);
      await view.close();
    }
    assert.deepEqual(calls, ["folder", "any"]);
  } finally { api.pickFolder = originalFolder; api.pickFile = originalFile; }
});


test("clicking a nonactive parameter row activates it before opening its form", async () => {
  setup();
  const view = await mount(createElement(ActionPicker, { onPick() {}, onClose() {} }));
  await click([...view.el.querySelectorAll('.act-list button')].find((b) => b.textContent.startsWith(t("preset.action.text.type.name"))));
  assert.ok(view.el.querySelector('.act-form textarea'));
  assert.ok(view.el.querySelector('li.on').textContent.startsWith(t("preset.action.text.type.name")));
  await view.close();
});


test("preset keycaps and platform notes use the current platform (UI 3, 8, 9, 16)", () => {
  setup();
  const preset = actions.findPreset("browser.close_tab");
  assert.equal(actions.presetKeys(preset), "^w");
  const card = render(createElement(ActionEditor, { action: { type: "preset", id: preset.id }, onChange() {} }));
  assert.ok(card.includes("<kbd>Ctrl</kbd>"));
  const noted = { ...preset, description_key: "preset.action.window.center.description", platform_notes: { windows: null, macos: "preset.note.accessibility" } };
  assert.ok(!actions.presetDescription(noted).includes(t("preset.note.accessibility")));
  assert.equal(actions.presetTypeLabel(actions.findPreset("open.website")), t("action.open_url"));
  platform.initPlatform("macos");
  assert.equal(actions.presetKeys(preset), "#w");
  assert.ok(actions.presetDescription(noted).includes(t("preset.note.accessibility")));
});

test("hovering another row keeps an open form; search Enter applies the form action, not the row", async () => {
  setup();
  const picked = [];
  const view = await mount(createElement(ActionPicker, { onPick(a) { picked.push(a); }, onClose() {} }));
  const search = view.el.querySelector('.search');
  // 打开带参数动作（输入一段文字）的表单
  await click([...view.el.querySelectorAll('.act-list button')].find((b) => b.textContent.startsWith(t("preset.action.text.type.name"))));
  const formInput = view.el.querySelector('.act-form textarea[data-param="text"]');
  assert.ok(formInput, "param form visible");
  // 鼠标滑过其他行：表单保持打开，不被收起
  const otherRow = view.el.querySelector('[role=option]');
  await act(async () => otherRow.dispatchEvent(new dom.window.MouseEvent("mousemove", { bubbles: true })));
  assert.ok(view.el.querySelector('.act-form textarea[data-param="text"]'), "form still open after hover");
  // 填入文字后在搜索框按回车：提交表单的动作，而不是高亮行
  await act(async () => {
    formInput.focus();
    Object.getOwnPropertyDescriptor(dom.window.HTMLTextAreaElement.prototype, "value").set.call(formInput, "你好");
    formInput.dispatchEvent(new dom.window.Event("input", { bubbles: true }));
  });
  await key(search, "Enter");
  assert.equal(picked.length, 1);
  assert.equal(picked[0].id, "text.type");
  assert.equal(picked[0].params.text, "你好");
  await view.close();
});


test("custom search validates with the backend and clearing its URL stays required", async () => {
  setup();
  const original = api.validatePreset;
  const calls = [], picked = [];
  api.validatePreset = async (id, params, lang) => {
    calls.push({ id, params, lang });
    if (!params.url.includes("%MS_SELECTED_TEXT%")) throw { key: "preset.status.invalid_param", params: { param: "url" } };
  };
  const view = await mount(createElement(ActionPicker, { initialId: "text.search", current: { type: "preset", id: "text.search", params: { engine: "custom" } }, onPick: (a) => picked.push(a), onClose() {} }));
  try {
    const field = view.el.querySelector('[data-param="url"]');
    await input(field, "https://example.org/");
    await key(field, "Enter");
    assert.equal(picked.length, 0);
    assert.equal(view.el.querySelector('[role="alert"]').textContent, t("preset.status.invalid_param"));
    assert.ok(document.activeElement === field, "焦点应回到校验失败的参数框");
    assert.ok(view.el.querySelector(".act-form"));
    await input(field, "   ");
    assert.equal(field.getAttribute("aria-required"), "true");
    assert.equal(field.getAttribute("aria-invalid"), "true");
    assert.equal(button(view.el, t("ui.actpick.apply")).disabled, true);
    await key(field, "Enter");
    assert.equal(calls.length, 1);
    assert.equal(picked.length, 0);
    await input(field, "  https://example.org/?q=%MS_SELECTED_TEXT%  ");
    await key(field, "Enter");
    await key(field, "Enter");
    assert.equal(calls.length, 2);
    assert.equal(picked.length, 1);
    assert.equal(picked[0].params, calls[1].params);
    assert.deepEqual(picked[0].params, { engine: "custom", url: "https://example.org/?q=%MS_SELECTED_TEXT%" });
  } finally { await view.close(); api.validatePreset = original; }
});

test("custom datetime is required and backend format errors keep its form open", async () => {
  setup();
  const original = api.validatePreset;
  const picked = [], calls = [];
  api.validatePreset = async (id, params) => {
    calls.push({ id, params });
    if (params.custom_format.includes("'")) throw { key: "preset.status.invalid_param", params: { param: "custom_format" } };
  };
  const view = await mount(createElement(ActionPicker, { initialId: "text.datetime", current: { type: "preset", id: "text.datetime", params: { format: "custom" } }, onPick: (a) => picked.push(a), onClose() {} }));
  try {
    const field = view.el.querySelector('[data-param="custom_format"]');
    await input(field, "");
    await key(field, "Enter");
    assert.equal(field.getAttribute("aria-invalid"), "true");
    assert.equal(calls.length, 0);
    await input(field, "YYYY-MM-DD '");
    await click(button(view.el, t("ui.actpick.apply")));
    assert.equal(picked.length, 0);
    assert.ok(document.activeElement === field, "焦点应回到校验失败的参数框");
    assert.equal(view.el.querySelector('[role="alert"]').textContent, t("preset.status.invalid_param"));
    await input(field, "YYYY-MM-DD HH:mm");
    await key(field, "Enter");
    assert.equal(picked.length, 1);
    assert.equal(picked[0].params, calls[1].params);
  } finally { await view.close(); api.validatePreset = original; }
});

test("native presets validate too and textarea requires Ctrl or Cmd Enter", async () => {
  setup();
  const original = api.validatePreset;
  const calls = [], picked = [];
  api.validatePreset = async (id, params) => { calls.push({ id, params }); };
  const view = await mount(createElement(ActionPicker, { initialId: "text.type", current: { type: "preset", id: "text.type", params: { text: "hello" } }, onPick: (a) => picked.push(a), onClose() {} }));
  try {
    const field = view.el.querySelector("textarea");
    await key(field, "Enter");
    assert.equal(calls.length, 0);
    await key(field, "Enter", { ctrlKey: true });
    assert.equal(picked.length, 1);
    assert.equal(calls[0].id, "text.type");
    assert.equal(picked[0].params, calls[0].params);
    await key(field, "Escape");
    await click([...view.el.querySelectorAll('.act-list button')].find((b) => b.textContent.startsWith(t("preset.action.desktop.clipboard_history.name"))));
    assert.equal(calls[1].id, "desktop.clipboard_history");
    assert.deepEqual(calls[1].params, {});
    assert.equal(picked.length, 2);
  } finally { await view.close(); api.validatePreset = original; }
});

test("backend rejection keeps the form open and allows retry", async () => {
  setup();
  const original = api.validatePreset;
  const picked = [];
  api.validatePreset = async () => { throw new Error("validation unavailable"); };
  const view = await mount(createElement(ActionPicker, { initialId: "open.website", onPick: (a) => picked.push(a), onClose() {} }));
  try {
    await click(button(view.el, t("ui.actpick.apply")));
    assert.equal(view.el.querySelector('[role="alert"]').textContent, "validation unavailable");
    assert.equal(picked.length, 0);
    assert.ok(view.el.querySelector(".act-form"));
    api.validatePreset = async () => {};
    await click(button(view.el, t("ui.actpick.apply")));
    assert.equal(picked.length, 1);
  } finally { await view.close(); api.validatePreset = original; }
});

for (const change of ["switch", "edit", "escape", "unmount"]) test(`pending validation is guarded and discarded after ${change}`, async () => {
  setup();
  const original = api.validatePreset;
  let finish, calls = 0;
  const picked = [];
  api.validatePreset = () => { calls++; return new Promise((resolve) => { finish = resolve; }); };
  const view = await mount(createElement(ActionPicker, { initialId: "open.website", onPick: (a) => picked.push(a), onClose() {} }));
  let unmounted = false;
  try {
    const field = view.el.querySelector('[data-param="url"]');
    await key(field, "Enter");
    await key(field, "Enter");
    assert.equal(calls, 1);
    assert.equal(button(view.el, t("ui.actpick.apply")).disabled, true);
    if (change === "switch") await click([...view.el.querySelectorAll('.act-list button')].find((b) => b.textContent.startsWith(t("preset.action.text.type.name"))));
    if (change === "edit") await input(field, "https://changed.example/");
    if (change === "escape") await key(field, "Escape");
    if (change === "unmount") { await view.close(); unmounted = true; }
    await act(async () => finish());
    assert.equal(picked.length, 0);
  } finally { if (!unmounted) await view.close(); api.validatePreset = original; }
});
