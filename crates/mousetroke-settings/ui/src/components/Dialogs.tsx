import { appIdentity, isMac } from "../platform";
import { useEffect, useMemo, useRef, useState } from "react";
import { AppIcon, NavIcon } from "./Icons";
import type { Config, PresetApp, RunningApp, Scope } from "../types";
import { api, errText } from "../api";
import { fileName } from "../gesture";
import { displayName, t } from "../i18n";

/** 通用模态框：Esc 关闭、点遮罩关闭、打开时聚焦第一个输入。 */
export function Modal({ title, onClose, children, wide }: { title: string; onClose: () => void; children: React.ReactNode; wide?: boolean }) {
  const box = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && (e.stopPropagation(), onClose());
    window.addEventListener("keydown", onKey, true);
    // 优先聚焦输入框（关闭按钮排在前面，但打开后多半是要输入）
    ((box.current?.querySelector("input") ?? box.current?.querySelector("button")) as HTMLElement | null)?.focus();
    return () => window.removeEventListener("keydown", onKey, true);
  }, [onClose]);
  return (
    <div className="modal-backdrop" onMouseDown={(e) => e.target === e.currentTarget && onClose()}>
      <div ref={box} className={`modal ${wide ? "wide" : ""}`} role="dialog" aria-modal aria-label={title}>
        <header className="modal-head">
          <h2>{title}</h2>
          <button type="button" className="icon-btn" aria-label={t("common.close")} onClick={onClose}>
            ×
          </button>
        </header>
        {children}
      </div>
    </div>
  );
}

/** 选择程序：列出正在运行的窗口，或浏览 exe。 */
export function AppPicker({
  scopes,
  onPick,
  onPickMany,
  onClose,
}: {
  scopes: Scope[];
  onPick: (exe: string, disable: boolean, name?: string) => void;
  /** 一次禁用多个（建议清单「全部禁用」）。 */
  onPickMany: (apps: PresetApp[]) => void;
  onClose: () => void;
}) {
  // 添加后的用途：单独设置手势，还是在里面禁用手势（远程桌面、3D 软件常用）
  const [disable, setDisable] = useState(false);
  const [apps, setApps] = useState<RunningApp[] | null>(null);
  const [presets, setPresets] = useState<PresetApp[] | null>(null);
  const [q, setQ] = useState("");
  const [picking, setPicking] = useState(false);
  const [pickError, setPickError] = useState<string | null>(null);
  useEffect(() => {
    api.runningApps().then(setApps, () => setApps([]));
    api.presetApps().then(setPresets, () => setPresets([]));
  }, []);
  const pickWindow = async () => {
    setPicking(true);
    setPickError(null);
    try {
      const p = await api.pickWindow();
      if (p) onPick(p.exe, disable);
    } catch (e) {
      setPickError(errText(e));
    } finally {
      setPicking(false);
    }
  };
  const k = q.trim().toLowerCase();
  const shown = useMemo(() => (apps ?? []).filter((a) => !k || a.exe.toLowerCase().includes(k) || a.title.toLowerCase().includes(k)), [apps, k]);
  const scopeOf = (exe: string) => scopes.find((s) => s.kind === "app" && appIdentity(s.app?.exe ?? "").toLowerCase() === appIdentity(exe).toLowerCase());
  const taken = (exe: string) => !!scopeOf(exe);
  const isOff = (exe: string) => !!scopeOf(exe)?.disabled;
  // 建议清单：已安装的在前；搜索时一起过滤
  const suggestions = useMemo(
    () =>
      (presets ?? [])
        .filter((p) => !k || displayName(p.name, p.name_key).toLowerCase().includes(k) || p.exe.toLowerCase().includes(k))
        .sort((a, b) => Number(!a.path) - Number(!b.path)),
    [presets, k],
  );
  const installedToDisable = suggestions.filter((p) => p.path && !isOff(p.exe));
  return (
    <Modal title={t("ui.picker.title")} onClose={onClose}>
      <div className="modal-body">
        <div className="segmented picker-mode" role="radiogroup" aria-label={t("ui.picker.mode_aria")}>
          <button type="button" role="radio" aria-checked={!disable} className={!disable ? "on" : ""} onClick={() => setDisable(false)}>
            {t("ui.picker.mode_set")}
          </button>
          <button type="button" role="radio" aria-checked={disable} className={disable ? "on" : ""} onClick={() => setDisable(true)}>
            {t("ui.picker.mode_disable")}
          </button>
        </div>
        <input className="search" type="search" placeholder={disable ? t("ui.picker.search_off") : t("ui.picker.search_on")} value={q} onChange={(e) => setQ(e.target.value)} />
        <ul className="app-list">
          {disable && suggestions.length > 0 && (
            <>
              <li className="app-group">
                <span>{t("ui.picker.suggest_group")}</span>
                <span className="muted">{t("ui.picker.suggest_hint")}</span>
                {installedToDisable.length > 1 && (
                  <button type="button" className="link" onClick={() => onPickMany(installedToDisable)}>
                    {t("ui.picker.disable_all_installed", { count: installedToDisable.length })}
                  </button>
                )}
              </li>
              {suggestions.map((p) => (
                <li key={p.exe} className={p.path ? "" : "not-installed"}>
                  <button type="button" disabled={isOff(p.exe)} onClick={() => onPick(p.exe, true, p.name_key)}>
                    <AppIcon exe={p.path ?? p.exe} size={24} />
                    <strong>{displayName(p.name, p.name_key)}</strong>
                    <span className="muted">
                      {isOff(p.exe) ? t("ui.picker.already_off") : p.path ? t("ui.picker.cat_installed", { category: displayName(p.category, p.category_key) }) : t("ui.picker.not_installed")}
                    </span>
                  </button>
                </li>
              ))}
              <li className="app-group">
                <span>{t("ui.picker.running_group")}</span>
              </li>
            </>
          )}
          {apps === null && <li className="muted">{t("ui.picker.loading")}</li>}
          {apps !== null && shown.length === 0 && <li className="muted">{t("ui.picker.no_match")}</li>}
          {shown.map((a) => (
            <li key={a.exe}>
              <button type="button" disabled={disable ? isOff(a.exe) : taken(a.exe)} onClick={() => onPick(a.exe, disable)}>
                <AppIcon exe={a.exe} size={24} />
                <strong>{isMac() ? a.title || a.exe : fileName(a.exe)}</strong>
                <span className="muted">
                  {disable && isOff(a.exe) ? t("ui.picker.already_off") : taken(a.exe) ? (disable ? t("ui.picker.added_switch_off") : t("ui.picker.added")) : a.title}
                </span>
              </button>
            </li>
          ))}
        </ul>
      </div>
      <footer className="modal-foot picker-foot">
        <span className="muted picker-hint">{pickError ?? t("ui.picker.drop_hint")}</span>
        <button type="button" className="btn" disabled={picking} onClick={pickWindow} data-tip={t("ui.picker.pick_tip")}>
          <NavIcon kind="pick" />
          {picking ? t("ui.picker.picking") : t("ui.picker.pick_window")}
        </button>
        <button
          type="button"
          className="btn"
          onClick={async () => {
            const p = await api.pickFile("exe", false, t("ui.picker.title"));
            if (p) onPick(p, disable);
          }}
        >
          {t("common.browse")}
        </button>
      </footer>
    </Modal>
  );
}

/** 导入与导出页。 */
export function DataPage({
  config,
  configPath,
  onReplace,
  notify,
}: {
  config: Config;
  configPath: string;
  onReplace: (c: Config, message: string) => void;
  notify: (msg: string) => void;
}) {
  const [busy, setBusy] = useState(false);
  const run = async (f: () => Promise<void>) => {
    setBusy(true);
    try {
      await f();
    } catch (e) {
      notify(errText(e));
    } finally {
      setBusy(false);
    }
  };

  /** 导入旧格式（gestures.json + 同目录可选的 prefs.json）。 */
  const pickLegacy = (mode: "replace" | "merge") =>
    run(async () => {
      const gestures = await api.pickFile("json", false, t("ui.data.pick_gestures"));
      if (!gestures) return;
      // 同目录的 prefs.json 存在就一并导入（后端会忽略不存在的文件）
      const incoming = await api.importLegacy(gestures, gestures.replace(/gestures\.json$/i, "prefs.json"));
      const n = incoming.scopes.reduce((a, s) => a + s.bindings.length, 0);
      if (mode === "replace") onReplace(incoming, t("ui.data.imported", { count: n }));
      else onReplace(mergeConfigs(config, incoming), t("ui.data.merged", { count: n }));
    });

  return (
    <section className="prefs" aria-label={t("ui.data.title")}>
      <h1>{t("ui.data.title")}</h1>

      <div className="pref-group">
        <h2>{t("ui.data.legacy_group")}</h2>
        <p>{t("ui.data.legacy_help")}</p>
        <div className="btn-row">
          <button type="button" className="btn primary" disabled={busy} onClick={() => pickLegacy("replace")}>
            {t("ui.data.import_replace")}
          </button>
          <button type="button" className="btn" disabled={busy} onClick={() => pickLegacy("merge")}>
            {t("ui.data.import_merge")}
          </button>
        </div>
        <p className="field-help">{t("ui.data.undo_help")}</p>
      </div>

      <div className="pref-group">
        <h2>{t("ui.data.config_group")}</h2>
        <p className="muted path">{configPath}</p>
        <div className="btn-row">
          <button
            type="button"
            className="btn"
            disabled={busy}
            onClick={() =>
              run(async () => {
                const p = await api.pickFile("json", true, t("ui.data.pick_export"));
                if (!p) return;
                await api.exportConfig(p, config);
                notify(t("ui.data.exported", { path: p }));
              })
            }
          >
            {t("ui.data.export")}
          </button>
          <button
            type="button"
            className="btn"
            disabled={busy}
            onClick={() =>
              run(async () => {
                const p = await api.pickFile("json", false, t("ui.data.pick_import"));
                if (!p) return;
                const c = await api.loadConfigFile(p);
                onReplace(c, t("ui.data.config_imported"));
              })
            }
          >
            {t("ui.data.import_file")}
          </button>
          <button type="button" className="btn subtle" onClick={() => api.revealConfig()}>
            {t("ui.data.reveal")}
          </button>
        </div>
      </div>
    </section>
  );
}

/** 合并：同名作用域（按 kind + exe）追加绑定，其余作用域整体加入；设置保持当前。 */
export function mergeConfigs(base: Config, incoming: Config): Config {
  const keyOf = (s: Config["scopes"][0]) => `${s.kind}|${fileName(s.app?.exe ?? "").toLowerCase()}|${s.special ?? ""}`;
  const scopes = base.scopes.map((s) => ({ ...s, bindings: [...s.bindings] }));
  for (const s of incoming.scopes) {
    const hit = scopes.find((x) => keyOf(x) === keyOf(s));
    if (hit) hit.bindings.push(...s.bindings);
    else scopes.push(s);
  }
  return { ...base, scopes };
}

/** 底部提示条（可带"撤销"）。 */
/**
 * 底部提示条。带操作（撤销）的 15 秒、普通提示 4 秒后自动关闭；右侧关闭按钮外圈是倒计时圆环。
 * 倒计时就是圆环的 CSS 动画本身（animationend 即关闭），鼠标悬停时动画与计时一起暂停。
 */
export function Toast({ text, action, onAction, onDone }: { text: string; action?: string; onAction?: () => void; onDone: () => void }) {
  const [leaving, setLeaving] = useState(false);
  // 父组件每次渲染都会给新的 onDone：放进 ref，不让它重置倒计时（曾因此永远不消失）
  const done = useRef(onDone);
  done.current = onDone;
  const close = () => setLeaving(true);
  const seconds = action ? 15 : 4;
  return (
    <div
      className={`toast ${leaving ? "leaving" : ""}`}
      role="status"
      style={{ "--toast-life": `${seconds}s` } as React.CSSProperties}
      onAnimationEnd={(e) => e.animationName === "toast-out" && done.current()}
    >
      <span className="toast-text">{text}</span>
      {action && (
        <button
          type="button"
          className="toast-action"
          onClick={() => {
            onAction?.();
            close();
          }}
        >
          {action}
        </button>
      )}
      <button type="button" className="toast-close" aria-label={t("ui.toast.close_aria", { seconds })} onClick={close}>
        <svg viewBox="0 0 28 28" width="28" height="28" aria-hidden>
          <circle className="toast-ring-track" cx="14" cy="14" r="12" />
          <circle className="toast-ring" cx="14" cy="14" r="12" pathLength="100" onAnimationEnd={close} />
          <path className="toast-x" d="M10.5 10.5l7 7M17.5 10.5l-7 7" />
        </svg>
      </button>
    </div>
  );
}
