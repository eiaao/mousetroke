import assert from "node:assert/strict";
import { after, test } from "node:test";
import { createServer } from "vite";
import { act, createElement } from "react";
import { JSDOM } from "jsdom";

const server = await createServer({ server: { middlewareMode: true, hmr: false, ws: false }, appType: "custom" });
after(() => server.close());
const stats = await server.ssrLoadModule("/src/stats.ts");
const { persistedConfig, withBindingIds } = await server.ssrLoadModule("/src/store.ts");
const { api } = await server.ssrLoadModule("/src/api.ts");
const { StatsPage } = await server.ssrLoadModule("/src/components/StatsPage.tsx");
const { t } = await server.ssrLoadModule("/src/i18n/index.ts");

const pattern = [{ type: "press", key: "mouse:right" }];
const binding = (name, extra = {}) => ({ name, pattern, action: { type: "key_sequence", keys: "^w" }, execute_on_match: false, hide_label: false, secured: false, ...extra });

test("summaries add up days, sort by count and keep 24 hours", () => {
  const data = {
    days: {
      "2026-11-17": { bindings: { a: 2, b: 5 }, apps: { "C:\\x.exe": 7 }, hours: [1, ...Array(22).fill(0), 6] },
      "2026-11-18": { bindings: { a: 4 }, apps: { "C:\\x.exe": 1, "C:\\y.exe": 3 }, hours: Array(24).fill(0) },
    },
    bindings: {},
  };
  const s = stats.summarize(data, ["2026-11-17", "2026-11-18", "2026-11-19"]);
  assert.equal(s.total, 11);
  assert.deepEqual(s.bindings, [["a", 6], ["b", 5]]);
  assert.deepEqual(s.apps, [["C:\\x.exe", 8], ["C:\\y.exe", 3]]);
  assert.equal(s.hours.length, 24);
  assert.equal(s.hours[23], 6);
  assert.equal(stats.dayTotal(data.days["2026-11-17"]), 7);
});

test("levels, calendar grids and selectable years", () => {
  assert.deepEqual([0, 1, 25, 26, 50, 100].map((n) => stats.level(n, 100)), [0, 1, 1, 2, 2, 4]);
  assert.equal(stats.level(3, 0), 0);
  // 2026-01-01 是周四：第一列前三格不属于该年
  const weeks = stats.yearWeeks(2026);
  assert.deepEqual(weeks[0].slice(0, 4).map((d) => d && stats.dayKey(d)), [null, null, null, "2026-01-01"]);
  assert.equal(weeks.flat().filter(Boolean).length, 365);
  assert.ok(weeks.every((w) => w.length === 7));
  const recent = stats.lastDays(new Date(2026, 10, 18), 30);
  assert.equal(stats.dayKey(recent[0]), "2026-10-20");
  assert.equal(stats.dayKey(recent[29]), "2026-11-18");
  const cal = stats.calendarWeeks(recent);
  assert.equal(cal.flat().filter(Boolean).length, 30);
  assert.equal(cal[0].findIndex(Boolean), 1, "10 月 20 日是周二");
  assert.deepEqual(stats.statsYears({ days: { "2024-05-01": {} }, bindings: {} }, new Date(2026, 0, 1)), [2024, 2025, 2026]);
  assert.deepEqual(stats.statsYears({ days: {}, bindings: {} }, new Date(2026, 0, 1)), [2026]);
});

test("binding ids persist from the UI id; copies keep their own new id", () => {
  const config = { version: 2, settings: {}, scopes: [{ id: "global", name: "All", kind: "global", bindings: [binding("a", { id: "b1" }), binding("copy", { id: "b1", uiId: "new" })] }] };
  const loaded = withBindingIds(config);
  assert.equal(loaded.scopes[0].bindings[0].uiId, "b1");
  const saved = persistedConfig(loaded);
  assert.deepEqual(saved.scopes[0].bindings.map((b) => b.id), ["b1", "new"]);
  assert.ok(saved.scopes[0].bindings.every((b) => !("uiId" in b)));
});

test("stats page shows tiles, calendar and bars; bars follow a selected day", async () => {
  const dom = new JSDOM("<!doctype html><html><body></body></html>", { url: "http://localhost/" });
  Object.assign(globalThis, { window: dom.window, document: dom.window.document, HTMLElement: dom.window.HTMLElement, location: dom.window.location, IS_REACT_ACT_ENVIRONMENT: true });
  const { createRoot } = await import("react-dom/client");
  const today = stats.dayKey(new Date());
  const yesterday = stats.dayKey(stats.addDays(new Date(), -1));
  const original = { usage: api.usageStats, icons: api.appIcons };
  api.usageStats = async () => ({
    days: {
      [today]: { bindings: { b1: 3 }, apps: { "C:\\app\\chrome.exe": 3 }, hours: Array(24).fill(0).map((_, h) => (h === 9 ? 3 : 0)) },
      [yesterday]: { bindings: { gone: 5 }, apps: {}, hours: Array(24).fill(0) },
    },
    bindings: { gone: { name: "Old gesture", scope: "global" } },
  });
  api.appIcons = async () => ({});
  const opened = [];
  const config = { version: 2, settings: {}, scopes: [{ id: "global", name: "All", kind: "global", bindings: [binding("Close tab", { uiId: "b1" })] }] };
  const host = document.createElement("div");
  document.body.append(host);
  const root = createRoot(host);
  try {
    await act(async () => root.render(createElement(StatsPage, { config, onOpenBinding: (s, b) => opened.push([s, b]) })));
    const tiles = [...host.querySelectorAll(".stats-tile strong")].map((e) => e.textContent);
    assert.deepEqual(tiles, ["3", "8", "8", "Old gesture"]);
    assert.equal(host.querySelectorAll(".cal-month button.cal-cell").length, 30);
    const names = () => [...host.querySelectorAll(".stats-bars")][0].querySelectorAll(".stats-bar-name");
    assert.deepEqual([...names()].map((e) => e.textContent), ["Old gesture", "Close tab"]);
    // 已删除的手势不能跳转；现有的跳到它所在的位置
    assert.equal([...host.querySelectorAll(".stats-bars")][0].querySelectorAll("button.stats-bar").length, 1);
    await act(async () => host.querySelector("button.stats-bar").click());
    assert.deepEqual(opened, [[0, 0]]);
    // 点今天：柱状图只看今天，再点取消
    const todayCell = host.querySelector(".cal-cell.today");
    await act(async () => todayCell.click());
    assert.deepEqual([...names()].map((e) => e.textContent), ["Close tab"]);
    await act(async () => host.querySelector(".cal-cell.today").click());
    assert.equal(names().length, 2);
    // 年视图
    const yearBtn = [...host.querySelectorAll(".segmented button")].find((b) => b.textContent === t("ui.stats.by_year"));
    await act(async () => yearBtn.click());
    assert.ok(host.querySelector(".cal-year"));
    assert.ok(host.querySelectorAll(".cal-year button.cal-cell").length >= 365);
  } finally {
    await act(async () => root.unmount());
    api.usageStats = original.usage;
    api.appIcons = original.icons;
  }
});

test("stats page explains when nothing is recorded", async () => {
  const { createRoot } = await import("react-dom/client");
  const original = api.usageStats;
  api.usageStats = async () => ({ days: {}, bindings: {} });
  const host = document.createElement("div");
  const root = createRoot(host);
  try {
    await act(async () => root.render(createElement(StatsPage, { config: { version: 2, settings: {}, scopes: [] }, onOpenBinding() {} })));
    assert.equal(host.querySelector(".stats-empty p").textContent, t("ui.stats.empty"));
  } finally {
    await act(async () => root.unmount());
    api.usageStats = original;
  }
});
