import { useEffect, useState } from "react";
import type { AppInfo, EngineStatus } from "../types";
import { api, errText } from "../api";
import { t } from "../i18n";

/** 设置界面里的键盘操作（关于页列出，便于发现）。 */
const SHORTCUTS: [string, keyof typeof KEY_LABELS][] = [
  ["Ctrl+F", "kbd_search"],
  ["↑ / ↓", "kbd_move"],
  ["SPACE_KEY", "kbd_toggle"],
  ["Delete", "kbd_delete"],
  ["Ctrl+D", "kbd_duplicate"],
  ["Ctrl+Z / Ctrl+Y", "kbd_undo"],
  ["Esc", "kbd_esc"],
];

const KEY_LABELS = {
  kbd_search: "ui.about.kbd_search",
  kbd_move: "ui.about.kbd_move",
  kbd_toggle: "ui.about.kbd_toggle",
  kbd_delete: "ui.about.kbd_delete",
  kbd_duplicate: "ui.about.kbd_duplicate",
  kbd_undo: "ui.about.kbd_undo",
  kbd_esc: "ui.about.kbd_esc",
} as const;

export function About({ engine, onShowGuide, notify }: { engine: EngineStatus; onShowGuide: () => void; notify: (msg: string) => void }) {
  const [info, setInfo] = useState<AppInfo | null>(null);
  useEffect(() => {
    api.appInfo().then(setInfo, () => {});
  }, []);
  const open = (what: "log" | "config_dir") => api.openPath(what).catch((e) => notify(errText(e)));

  return (
    <section className="prefs about" aria-label={t("ui.about.aria")}>
      <div className="about-hero">
        <svg className="about-logo" width="64" height="64" viewBox="0 0 32 32" aria-hidden>
          <rect width="32" height="32" rx="8" fill="var(--trail)" />
          <polyline className="about-logo-line" points="7,11 13.5,21 19,13 25,21" fill="none" stroke="#fff" strokeWidth="3.2" strokeLinecap="round" strokeLinejoin="round" pathLength="100" />
        </svg>
        <div>
          <h1>Mousetroke</h1>
          <p className="muted">
            {t("ui.about.subtitle", {
              version: info?.version ?? "…",
              status: engine.running ? t("ui.sidebar.engine_on") : t("ui.sidebar.engine_off"),
            })}
          </p>
        </div>
        <button type="button" className="btn primary about-guide" onClick={onShowGuide}>
          {t("ui.about.show_guide")}
        </button>
      </div>

      <div className="pref-group">
        <h2>{t("ui.about.files_group")}</h2>
        <div className="pref-row">
          <span className="pref-label">
            {t("ui.about.config_file")}
            <small className="path">{info?.configPath}</small>
          </span>
          <button type="button" className="btn small" onClick={() => open("config_dir")}>
            {t("ui.about.open_folder")}
          </button>
        </div>
        <div className="pref-row">
          <span className="pref-label">
            {t("ui.about.log_file")}
            <small className="path">{info?.logPath}</small>
            <small>{t("ui.about.log_help")}</small>
          </span>
          <button type="button" className="btn small" onClick={() => open("log")}>
            {t("ui.about.open_log")}
          </button>
        </div>
        <div className="pref-row">
          <span className="pref-label">
            {t("ui.about.program")}
            <small className="path">{info?.exePath}</small>
          </span>
        </div>
      </div>

      <div className="pref-group">
        <h2>{t("ui.about.keys_group")}</h2>
        <dl className="shortcuts">
          {SHORTCUTS.map(([k, v]) => (
            <div key={k}>
              <dt>
                {k === "SPACE_KEY" ? (
                  <kbd>{t("ui.about.key_space")}</kbd>
                ) : (
                  k.split(" / ").map((x, i) => (
                    <span key={x}>
                      {i > 0 && " / "}
                      {x.split("+").map((p, j) => (
                        <span key={p}>
                          {j > 0 && "+"}
                          <kbd>{p}</kbd>
                        </span>
                      ))}
                    </span>
                  ))
                )}
              </dt>
              <dd>{t(KEY_LABELS[v])}</dd>
            </div>
          ))}
        </dl>
      </div>
    </section>
  );
}
