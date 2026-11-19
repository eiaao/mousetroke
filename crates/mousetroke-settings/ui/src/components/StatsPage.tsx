import { useEffect, useMemo, useState } from "react";
import type { Config } from "../types";
import { api } from "../api";
import { fileName } from "../gesture";
import { currentLang, displayName, t } from "../i18n";
import { appIdentity } from "../platform";
import { calendarWeeks, dayKey, dayTotal, lastDays, level, statsYears, summarize, yearWeeks, type UsageStats } from "../stats";
import { AppIcon } from "./Icons";
import { PatternGlyph } from "./Glyph";
import { scopeTitle } from "./Sidebar";

/** 引擎攒几秒写一次盘；页面开着时定期重读。 */
const REFRESH_MS = 5000;
const TOP = 10;

type CalendarView = { kind: "recent" } | { kind: "year"; year: number };

export function StatsPage({ config, onOpenBinding }: { config: Config; onOpenBinding: (scope: number, binding: number) => void }) {
  const [stats, setStats] = useState<UsageStats | null>(null);
  const [view, setView] = useState<CalendarView>({ kind: "recent" });
  const [day, setDay] = useState<string | null>(null);
  useEffect(() => {
    const load = () => api.usageStats().then(setStats, () => setStats({ days: {}, bindings: {} }));
    load();
    const timer = setInterval(load, REFRESH_MS);
    return () => clearInterval(timer);
  }, []);

  const today = new Date();
  const todayKey = dayKey(today);
  const recent = lastDays(today, 30);
  const fullFmt = useMemo(() => new Intl.DateTimeFormat(currentLang(), { year: "numeric", month: "short", day: "numeric", weekday: "short" }), []);

  if (!stats) return <section className="prefs stats" aria-busy />;
  const allKeys = Object.keys(stats.days);
  if (!allKeys.some((k) => dayTotal(stats.days[k]) > 0)) {
    return (
      <section className="prefs stats">
        <h1>{t("ui.stats.title")}</h1>
        <div className="pref-group stats-empty"><p className="muted">{t("ui.stats.empty")}</p></div>
      </section>
    );
  }

  // 柱状图的范围：选中的那一天，否则跟随日历视图（最近 30 天 / 所选年份）
  const rangeKeys = day ? [day] : view.kind === "recent" ? recent.map(dayKey) : allKeys.filter((k) => k.startsWith(`${view.year}-`));
  const rangeLabel = day ? fullFmt.format(parseKey(day)) : view.kind === "recent" ? t("ui.stats.last30") : t("ui.stats.year", { year: view.year });
  const range = summarize(stats, rangeKeys);
  const overall = summarize(stats, allKeys);
  const top = overall.bindings[0];

  // 绑定 id → 当前配置里的位置（已删除的用统计里记下的名字）
  const where = new Map<string, [number, number]>();
  config.scopes.forEach((s, si) => s.bindings.forEach((b, bi) => b.uiId && where.set(b.uiId, [si, bi])));
  const gesture = (id: string) => {
    const at = where.get(id);
    if (at) {
      const [si, bi] = at;
      const b = config.scopes[si].bindings[bi];
      return { name: displayName(b.name, b.name_key) || t("ui.list.unnamed"), scope: scopeTitle(config.scopes[si]), pattern: b.pattern, at };
    }
    const seen = stats.bindings[id];
    return { name: seen ? displayName(seen.name, seen.name_key) : id, scope: t("ui.stats.deleted"), pattern: null, at: null };
  };
  const appName = (exe: string) => {
    const scope = config.scopes.find((s) => s.kind === "app" && s.app && appIdentity(s.app.exe).toLowerCase() === appIdentity(exe).toLowerCase());
    return scope ? scopeTitle(scope) : fileName(exe) || exe;
  };

  const years = statsYears(stats, today);
  const year = view.kind === "year" ? view.year : today.getFullYear();
  const yearIdx = years.indexOf(year);
  const weeks = view.kind === "recent" ? calendarWeeks(recent) : yearWeeks(year);
  const shown = weeks.flat().filter((d): d is Date => !!d);
  const max = Math.max(0, ...shown.map((d) => dayTotal(stats.days[dayKey(d)])));
  const cell = (d: Date | null, i: number) => {
    if (!d) return <span key={i} className="cal-cell blank" aria-hidden />;
    const key = dayKey(d);
    const n = dayTotal(stats.days[key]);
    const future = key > todayKey;
    const tip = `${fullFmt.format(d)} · ${t("ui.stats.count", { count: n })}`;
    return (
      <button
        key={i}
        type="button"
        className={`cal-cell l${future ? 0 : level(n, max)} ${day === key ? "on" : ""} ${key === todayKey ? "today" : ""}`}
        disabled={future}
        aria-pressed={day === key}
        aria-label={tip}
        data-tip={future ? undefined : tip}
        onClick={() => setDay(day === key ? null : key)}
      >
        {view.kind === "recent" && <span className="cal-num">{d.getDate()}</span>}
      </button>
    );
  };
  const weekdays = Array.from({ length: 7 }, (_, i) => new Intl.DateTimeFormat(currentLang(), { weekday: "narrow" }).format(new Date(2024, 0, 1 + i)));
  const monthFmt = new Intl.DateTimeFormat(currentLang(), { month: "short" });

  return (
    <section className="prefs stats" aria-label={t("ui.stats.title")}>
      <h1>{t("ui.stats.title")}</h1>
      <div className="stats-tiles">
        <Tile label={t("ui.stats.today")} value={dayTotal(stats.days[todayKey])} />
        <Tile label={t("ui.stats.last30")} value={summarize(stats, recent.map(dayKey)).total} />
        <Tile label={t("ui.stats.total")} value={overall.total} />
        <div className="stats-tile">
          <span className="muted">{t("ui.stats.top")}</span>
          <strong className="stats-tile-name">{top ? gesture(top[0]).name : "—"}</strong>
        </div>
      </div>

      <div className="pref-group stats-card">
        <div className="stats-head">
          <h2>{t("ui.stats.calendar")}</h2>
          {view.kind === "year" && (
            <div className="stats-year">
              <button type="button" className="btn icon subtle" aria-label={t("ui.stats.prev_year")} disabled={yearIdx <= 0} onClick={() => { setView({ kind: "year", year: years[yearIdx - 1] }); setDay(null); }}>‹</button>
              <span>{year}</span>
              <button type="button" className="btn icon subtle" aria-label={t("ui.stats.next_year")} disabled={yearIdx >= years.length - 1} onClick={() => { setView({ kind: "year", year: years[yearIdx + 1] }); setDay(null); }}>›</button>
            </div>
          )}
          <div className="segmented" role="radiogroup" aria-label={t("ui.stats.calendar")}>
            <button type="button" role="radio" aria-checked={view.kind === "recent"} className={view.kind === "recent" ? "on" : ""} onClick={() => { setView({ kind: "recent" }); setDay(null); }}>
              {t("ui.stats.last30")}
            </button>
            <button type="button" role="radio" aria-checked={view.kind === "year"} className={view.kind === "year" ? "on" : ""} onClick={() => { setView({ kind: "year", year: today.getFullYear() }); setDay(null); }}>
              {t("ui.stats.by_year")}
            </button>
          </div>
        </div>
        {view.kind === "recent" ? (
          <div className="cal-month">
            {weekdays.map((w, i) => <span key={`w${i}`} className="cal-weekday">{w}</span>)}
            {weeks.flat().map(cell)}
          </div>
        ) : (
          <div className="cal-year-wrap">
            <div className="cal-year" style={{ gridTemplateColumns: `auto repeat(${weeks.length}, var(--cal-size))` }}>
              <span />
              {weeks.map((w, i) => {
                // 含某月 1 日的那一列标月份
                const first = w.find((d) => d?.getDate() === 1);
                return <span key={`m${i}`} className="cal-month-label">{first ? monthFmt.format(first) : ""}</span>;
              })}
              {Array.from({ length: 7 }, (_, row) => [
                <span key={`r${row}`} className="cal-weekday">{row % 2 === 0 ? weekdays[row] : ""}</span>,
                ...weeks.map((w, col) => cell(w[row], row * 100 + col)),
              ])}
            </div>
          </div>
        )}
        <div className="cal-legend muted">
          <span>{t("ui.stats.less")}</span>
          {[0, 1, 2, 3, 4].map((l) => <span key={l} className={`cal-cell l${l}`} aria-hidden />)}
          <span>{t("ui.stats.more")}</span>
        </div>
      </div>

      <div className="stats-range">
        <span>{rangeLabel}</span>
        <strong>{t("ui.stats.count", { count: range.total })}</strong>
        {day && <button type="button" className="link" onClick={() => setDay(null)}>{t("ui.stats.clear_day")}</button>}
      </div>

      <div className="stats-grid">
        <div className="pref-group stats-card">
          <h2>{t("ui.stats.top_gestures")}</h2>
          <Bars
            rows={range.bindings.slice(0, TOP).map(([id, n]) => {
              const g = gesture(id);
              return {
                key: id, n,
                icon: g.pattern ? <PatternGlyph pattern={g.pattern} size={22} /> : <span className="stats-glyph-blank" />,
                label: g.name, sub: g.scope,
                onClick: g.at ? () => onOpenBinding(...g.at!) : undefined,
              };
            })}
          />
        </div>
        <div className="pref-group stats-card">
          <h2>{t("ui.stats.by_app")}</h2>
          <Bars rows={byApp(range.apps).slice(0, TOP).map(([exe, n]) => ({ key: exe, n, icon: <AppIcon exe={exe} size={20} />, label: appName(exe) }))} />
        </div>
      </div>

      <div className="pref-group stats-card">
        <h2>{t("ui.stats.by_hour")}</h2>
        <Hours hours={range.hours} />
      </div>
    </section>
  );
}

/** 同一程序（不同路径）合并成一行，图标取次数最多的那个路径。 */
function byApp(apps: [string, number][]): [string, number][] {
  const merged = new Map<string, [string, number]>();
  for (const [exe, n] of apps) {
    const id = appIdentity(exe).toLowerCase();
    const prev = merged.get(id);
    merged.set(id, prev ? [prev[0], prev[1] + n] : [exe, n]);
  }
  return [...merged.values()].sort((a, b) => b[1] - a[1]);
}

function parseKey(key: string): Date {
  const [y, m, d] = key.split("-").map(Number);
  return new Date(y, m - 1, d);
}

function Tile({ label, value }: { label: string; value: number }) {
  return (
    <div className="stats-tile">
      <span className="muted">{label}</span>
      <strong>{value.toLocaleString(currentLang())}</strong>
    </div>
  );
}

type BarRow = { key: string; n: number; icon: React.ReactNode; label: string; sub?: string; onClick?: () => void };

/** 横向条形图：单一系列，条尾直接标次数（排名列表，不需要图例）。 */
function Bars({ rows }: { rows: BarRow[] }) {
  if (!rows.length) return <p className="muted stats-none">{t("ui.stats.none")}</p>;
  const max = rows[0].n;
  return (
    <ul className="stats-bars">
      {rows.map((r) => {
        const body = (
          <>
            <span className="stats-bar-icon">{r.icon}</span>
            <span className="stats-bar-label">
              <span className="stats-bar-name">{r.label}</span>
              {r.sub && <span className="muted stats-bar-sub">{r.sub}</span>}
            </span>
            <span className="stats-bar-track"><span className="stats-bar-fill" style={{ width: `${Math.max(2, (r.n / max) * 100)}%` }} /></span>
            <span className="stats-bar-n">{r.n.toLocaleString(currentLang())}</span>
          </>
        );
        return (
          <li key={r.key} data-tip={t("ui.stats.count", { count: r.n })}>
            {r.onClick ? <button type="button" className="stats-bar" onClick={r.onClick}>{body}</button> : <div className="stats-bar">{body}</div>}
          </li>
        );
      })}
    </ul>
  );
}

/** 24 小时柱状图：每根柱自带悬停提示；隔 6 小时标一次刻度。 */
function Hours({ hours }: { hours: number[] }) {
  const max = Math.max(...hours);
  if (max <= 0) return <p className="muted stats-none">{t("ui.stats.none")}</p>;
  return (
    <div className="stats-hours">
      {hours.map((n, h) => (
        <div key={h} className="stats-hour" data-tip={`${String(h).padStart(2, "0")}:00 · ${t("ui.stats.count", { count: n })}`} aria-label={`${h}:00 ${t("ui.stats.count", { count: n })}`}>
          <span className="stats-hour-plot"><span className="stats-hour-bar" style={{ height: `${n ? Math.max(4, (n / max) * 100) : 0}%` }} /></span>
          <span className="stats-hour-label muted">{h % 6 === 0 ? h : ""}</span>
        </div>
      ))}
    </div>
  );
}
