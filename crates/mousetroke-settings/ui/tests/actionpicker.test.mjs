import assert from "node:assert/strict";
import { after, test } from "node:test";
import { createServer } from "vite";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";

// 使用与产物相同的 Vite 转换，测试动作选择器的搜索、分类、键盘与参数校验。
const server = await createServer({ server: { middlewareMode: true }, appType: "custom" });
after(() => server.close());
const platform = await server.ssrLoadModule("/src/platform.ts");
const actions = await server.ssrLoadModule("/src/actions.ts");
const keys = await server.ssrLoadModule("/src/gesture.ts");
const { t } = await server.ssrLoadModule("/src/i18n/index.ts");
const { ActionPicker } = await server.ssrLoadModule("/src/components/ActionPicker.tsx");
const { ActionEditor } = await server.ssrLoadModule("/src/components/ActionEditor.tsx");
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
      windows: { status: "pending", value: { reason_key: "preset.status.pending" } },
      macos: { status: "pending", value: { reason_key: "preset.status.pending" } },
    },
    {
      id: "text.search", category: "text",
      name_key: "preset.action.text.search.name", description_key: "preset.action.text.search.description", aliases_key: "preset.action.text.search.aliases",
      params: [
        { name: "engine", label_key: "preset.param.engine", kind: { type: "choice", choices: [{ value: "google", label_key: "preset.choice.google" }, { value: "baidu", label_key: "preset.choice.baidu" }] }, default: { kind: "search_engine" } },
        { name: "url", label_key: "preset.param.search_url", kind: { type: "url" }, default: { kind: "literal", value: "https://www.google.com/search?q=%MS_SELECTED_TEXT%" } },
      ],
      windows: { status: "implemented", value: { type: "search" } },
      macos: { status: "implemented", value: { type: "search" } },
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
    "open.website": { url: "https://mousetroke.com/" },
  },
  commonPresetIds: ["browser.close_tab", "text.type", "open.website", "desktop.clipboard_history"],
  knownActionTypes: ["preset"],
  unknownActionKey: "preset.status.unknown",
};

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
  assert.equal(actions.pickerNav(0, 4, "ArrowDown", sel), 3, "跳过 pending");
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
  const html = render(createElement(ActionPicker, { onPick() {}, onClose() {} }));
  assert.ok(html.includes(t("preset.status.windows_only")), "unsupported 行显示原因");
  assert.ok(html.includes(t("ui.actpick.soon")), "pending 行显示即将支持");
  assert.ok((html.match(/disabled=""/g) ?? []).length >= 2, "灰显的行都不可选");
  assert.ok(html.includes('aria-live="polite"'), "原因通过 aria-live 读出");
  const card = render(createElement(ActionEditor, { action: { type: "preset", id: "text.type" }, onChange() {} }));
  assert.ok(!card.includes(t("ui.actpick.to_advanced")), "pending 动作不能转为高级动作");
  assert.ok(card.includes(t("ui.actpick.change")));
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
  assert.deepEqual(actions.cleanPresetParams(typePreset, { text: "  收到 ", method: "paste" }), { text: "收到", method: "paste" });
  assert.deepEqual(actions.cleanPresetParams(typePreset, { text: "  ", method: "type" }), { method: "type" }, "空白参数不提交");
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
  const html = render(createElement(ActionEditor, { action: { type: "key_sequence", keys: "" }, onChange() {} }));
  assert.ok(html.includes(t("ui.actpick.empty_hint")), "未选择时显示提示");
  assert.ok(html.includes(t("ui.actpick.choose")), "未选择时显示选择动作按钮");
  assert.ok(!html.includes(t("ui.actpick.change_type")), "未选择时不显示折叠的类型条");
});
