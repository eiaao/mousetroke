import type { EngineStatus, Scope } from "../types";
import { fileName } from "../gesture";
import { currentLang, displayName, t } from "../i18n";
import { AppIcon, NavIcon } from "./Icons";

export type View = { kind: "scope"; index: number } | { kind: "prefs" } | { kind: "data" } | { kind: "about" };

export function scopeTitle(s: Scope): string {
  if (s.kind === "global") return t("scope.global");
  if (s.kind === "special") return s.special === "taskbar" ? t("scope.taskbar") : t("scope.desktop");
  return displayName(s.name, s.name_key) || fileName(s.app?.exe ?? "") || t("scope.unnamed_app");
}

function Logo() {
  return (
    <svg width="22" height="22" viewBox="0 0 32 32" aria-hidden>
      <rect width="32" height="32" rx="7" fill="var(--trail)" />
      <polyline points="7,11 13.5,21 19,13 25,21" fill="none" stroke="#fff" strokeWidth="3.4" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

export function Sidebar({
  scopes,
  view,
  onView,
  onAddApp,
  engine,
  onStartEngine,
  saveState,
}: {
  scopes: Scope[];
  view: View;
  onView: (v: View) => void;
  onAddApp: () => void;
  engine: EngineStatus;
  onStartEngine: () => void;
  saveState: "idle" | "saving" | "saved" | "error";
}) {
  // 通用在最前，其次特殊区域，再按名称排列应用（排序跟随当前语言）
  const order = scopes
    .map((s, i) => ({ s, i }))
    .sort((a, b) => {
      const rank = (s: Scope) => (s.kind === "global" ? 0 : s.kind === "special" ? 1 : 2);
      return rank(a.s) - rank(b.s) || scopeTitle(a.s).localeCompare(scopeTitle(b.s), currentLang());
    });
  return (
    <nav className="sidebar" aria-label={t("ui.sidebar.aria")}>
      <div className="brand">
        <Logo />
        <span>Mousetroke</span>
      </div>

      <div className="nav-group">
        <div className="nav-heading">{t("ui.sidebar.where")}</div>
        {order.map(({ s, i }) => (
          <button
            key={s.id}
            type="button"
            className={`nav-item ${view.kind === "scope" && view.index === i ? "on" : ""} ${s.disabled ? "off" : ""}`}
            onClick={() => onView({ kind: "scope", index: i })}
            data-tip={s.app?.exe}
          >
            {s.kind === "app" ? <AppIcon exe={s.app?.exe} /> : <NavIcon kind={s.kind === "global" ? "global" : "desktop"} />}
            <span className="nav-label">{scopeTitle(s)}</span>
            {s.disabled ? <span className="nav-count nav-off">{t("ui.sidebar.disabled_badge")}</span> : <span className="nav-count">{s.bindings.length}</span>}
          </button>
        ))}
        <button type="button" className="nav-item nav-add" onClick={onAddApp}>
          <NavIcon kind="add" />
          <span className="nav-label">{t("ui.sidebar.add_app")}</span>
        </button>
      </div>

      <div className="nav-group nav-bottom">
        <button type="button" className={`nav-item ${view.kind === "prefs" ? "on" : ""}`} onClick={() => onView({ kind: "prefs" })}>
          <NavIcon kind="prefs" />
          {t("ui.sidebar.prefs")}
        </button>
        <button type="button" className={`nav-item ${view.kind === "data" ? "on" : ""}`} onClick={() => onView({ kind: "data" })}>
          <NavIcon kind="data" />
          {t("ui.sidebar.data")}
        </button>
        <button type="button" className={`nav-item ${view.kind === "about" ? "on" : ""}`} onClick={() => onView({ kind: "about" })}>
          <NavIcon kind="about" />
          {t("ui.sidebar.about")}
        </button>
      </div>

      <div className="status">
        <div className={`status-engine ${engine.running ? "ok" : "off"}`}>
          <span className="dot" aria-hidden />
          {engine.running ? t("ui.sidebar.engine_on") : t("ui.sidebar.engine_off")}
          {!engine.running && engine.exeFound && (
            <button type="button" className="link" onClick={onStartEngine}>
              {t("ui.sidebar.start_engine")}
            </button>
          )}
        </div>
        <div className="status-save" aria-live="polite">
          {saveState === "saving" ? t("ui.sidebar.saving") : saveState === "saved" ? t("ui.sidebar.saved") : saveState === "error" ? t("ui.sidebar.save_failed") : ""}
        </div>
      </div>
    </nav>
  );
}
