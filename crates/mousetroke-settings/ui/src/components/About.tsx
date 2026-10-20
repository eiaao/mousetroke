import { isMac } from "../platform";
import { useEffect, useState } from "react";
import type { AppInfo, EngineStatus } from "../types";
import { api, errText, type Msg } from "../api";
import { t } from "../i18n";
import { StarIcon } from "./Icons";

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
  const [update, setUpdate] = useState<Awaited<ReturnType<typeof api.checkUpdate>> | null>(null);
  const [busy, setBusy] = useState<"check" | "install" | null>(null);
  const [error, setError] = useState<unknown>(null);
  useEffect(() => {
    api.appInfo().then(setInfo, () => {});
  }, []);
  const open = (what: Parameters<typeof api.openPath>[0]) => api.openPath(what).catch((e) => notify(errText(e)));
  const checkUpdate = async () => {
    setBusy("check"); setError(null);
    try { setUpdate(await api.checkUpdate()); }
    catch (e) { setError(e); }
    finally { setBusy(null); }
  };
  const installUpdate = async () => {
    setBusy("install"); setError(null);
    try { await api.installUpdate(); }
    catch (e) {
      setError(e);
      if (!isMac() && (e as Msg)?.key === "update.error.not_writable") await open("releases");
    } finally { setBusy(null); }
  };
  const downloadUpdate = async () => {
    setBusy("check");
    try { await api.downloadUpdate(); }
    catch (e) { setError(e); }
    finally { setBusy(null); }
  };

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

      <div className="pref-group update-group" aria-busy={busy !== null}>
        <div className="pref-row">
          <span className="pref-label">{t("ui.update.version")}<small>{info?.version ?? "…"}</small></span>
          <button type="button" className="btn small update-check" disabled={busy !== null} onClick={checkUpdate}>
            {busy === "check" && <span className="update-spinner" aria-hidden />}
            {t(busy === "check" ? "ui.update.checking" : error ? "ui.update.retry" : "ui.update.check")}
          </button>
        </div>
        <UpdateStatus update={update} busy={busy} error={error} onInstall={installUpdate}
          onDownload={() => isMac() && !error ? downloadUpdate() : open("releases")} onNotes={() => open("releases")} />
      </div>

      <div className="pref-group about-project" role="group" aria-labelledby="about-project-title">
        <h2 id="about-project-title">{t("ui.about.project_group")}</h2>
        <div className="pref-row">
          <span className="pref-label" id="about-homepage">
            {t("ui.about.homepage")}
            <small>{t("ui.about.homepage_address")}</small>
          </span>
          <button type="button" className="btn small" aria-describedby="about-homepage" onClick={() => open("homepage")}>
            {t("ui.about.open_link")}
          </button>
        </div>
        <div className="pref-row">
          <span className="pref-label" id="about-source">
            {t("ui.about.source")}
            <small>{t("ui.about.source_address")}</small>
          </span>
          <button type="button" className="btn small" aria-describedby="about-source" onClick={() => open("github")}>
            {t("ui.about.open_link")}
          </button>
        </div>
        <div className="pref-row">
          <span className="pref-label" id="about-feedback">
            {t("ui.about.feedback")}
            <small>{t("ui.about.feedback_help")}</small>
          </span>
          <button type="button" className="btn small" aria-describedby="about-feedback" onClick={() => open("issues")}>
            {t("ui.about.open_link")}
          </button>
        </div>
        <div className="about-star">
          <p>{t("ui.about.star_prompt")}</p>
          <button type="button" className="btn primary" onClick={() => open("github")}>
            <StarIcon />
            {t("ui.about.star_button")}
          </button>
        </div>
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
          {SHORTCUTS.map(([original, v]) => {
            const k = isMac() ? original === "Ctrl+Z / Ctrl+Y" ? "⌘+Z / ⌘+⇧+Z" : original.replaceAll("Ctrl", "⌘") : original;
            return (
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
          ); })}
        </dl>
      </div>
    </section>
  );
}

/** 两个平台共用更新状态和主要操作，Mac 另外保留手动下载。 */
export function UpdateStatus({ update, busy, error, onInstall, onDownload, onNotes }: {
  update: Awaited<ReturnType<typeof api.checkUpdate>> | null;
  busy: "check" | "install" | null;
  error: unknown;
  onInstall: () => void;
  onDownload: () => void;
  onNotes: () => void;
}) {
  return <div className="update-status" role="status" aria-live="polite">
    {isMac() && update && <p>{t("mac.ui.update.latest", { version: update.latest })}</p>}
    {busy === "install" ? <p><span className="update-spinner" aria-hidden />{t("ui.update.installing")}</p> : error ? <>
      <p className="update-error">{errText(error)}</p>
      <button type="button" className="btn small update-download" disabled={isMac() && busy !== null} onClick={onDownload}>{t(isMac() ? "mac.ui.update.download" : "ui.update.download")}</button>
    </> : update && (update.newer ? <>
      <p>{t("update.available", { version: update.latest })}</p>
      {update.notes.length > 0 && <ul className="update-note-list" tabIndex={0}>
        {update.notes.map((note, index) => <li key={index}>{note}</li>)}
      </ul>}
      <div className="update-actions">
        <button type="button" className="btn small update-notes" disabled={busy !== null} onClick={onNotes}>{t("ui.update.notes")}</button>
        {isMac() && <button type="button" className="btn small update-download" disabled={busy !== null} onClick={onDownload}>{t("mac.ui.update.download")}</button>}
        <button type="button" className="btn primary update-install" disabled={busy !== null} onClick={onInstall}>{t("ui.update.install")}</button>
      </div>
    </> : <p>{t("ui.update.current")}</p>)}
  </div>;
}
