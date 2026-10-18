import { useEffect, useState } from "react";
import type { PresetApp } from "../types";
import { api } from "../api";
import { T, displayName, t } from "../i18n";
import { AppIcon } from "./Icons";
import { Caps } from "./KeysInput";

const SEEN_KEY = "mousetroke.guide.seen";

/** 是否还没看过使用引导（本机浏览器存储；读不到时当作看过，不打扰）。 */
export function guideUnseen(): boolean {
  try {
    return localStorage.getItem(SEEN_KEY) !== "1";
  } catch {
    return false;
  }
}

function markSeen() {
  try {
    localStorage.setItem(SEEN_KEY, "1");
  } catch {
    /* 存不了就下次再显示 */
  }
}

/**
 * 使用引导：四步，每步一个循环播放的小动画。
 * 第三步列出本机找到的"建议禁用"程序（默认勾选），完成时一并禁用。
 */
export function Guide({ pauseHotkey, onDisableApps, onClose }: { pauseHotkey: string; onDisableApps: (apps: PresetApp[]) => void; onClose: () => void }) {
  const [step, setStep] = useState(0);
  const [dir, setDir] = useState<1 | -1>(1);
  const [presets, setPresets] = useState<PresetApp[] | null>(null);
  const [chosen, setChosen] = useState<Set<string>>(new Set());

  useEffect(() => {
    api.presetApps().then(
      (list) => {
        const installed = list.filter((p) => p.path);
        setPresets(installed);
        setChosen(new Set(installed.map((p) => p.exe)));
      },
      () => setPresets([]),
    );
  }, []);

  const finish = () => {
    markSeen();
    if (chosen.size) onDisableApps((presets ?? []).filter((p) => chosen.has(p.exe)));
    onClose();
  };
  const skip = () => {
    markSeen();
    onClose();
  };
  const go = (to: number) => {
    setDir(to > step ? 1 : -1);
    setStep(to);
  };

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") (e.stopPropagation(), skip());
      else if (e.key === "ArrowRight" && step < STEPS - 1) go(step + 1);
      else if (e.key === "ArrowLeft" && step > 0) go(step - 1);
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  });

  const pages = [
    {
      art: <GestureArt />,
      title: t("ui.guide.p1_title"),
      body: <T k="ui.guide.p1_body" />,
    },
    {
      art: <EdgeArt />,
      title: t("ui.guide.p2_title"),
      body: <T k="ui.guide.p2_body" />,
    },
    {
      art: <AppsArt presets={presets} />,
      title: t("ui.guide.p3_title"),
      body:
        presets && presets.length > 0 ? (
          <>
            <p>{t("ui.guide.p3_body_installed")}</p>
            <div className="guide-apps">
              {presets.map((p) => (
                <label key={p.exe} className="guide-app">
                  <input
                    type="checkbox"
                    checked={chosen.has(p.exe)}
                    onChange={(e) =>
                      setChosen((s) => {
                        const n = new Set(s);
                        e.target.checked ? n.add(p.exe) : n.delete(p.exe);
                        return n;
                      })
                    }
                  />
                  <AppIcon exe={p.path ?? p.exe} size={20} />
                  {displayName(p.name, p.name_key)}
                </label>
              ))}
            </div>
          </>
        ) : (
          <p>{t("ui.guide.p3_body_none")}</p>
        ),
    },
    {
      art: <TrayArt />,
      title: t("ui.guide.p4_title"),
      body: (
        <>
          <p>{t("ui.guide.p4_body")}</p>
          <div className="guide-keys">
            <Caps keys={pauseHotkey} />
          </div>
          <p className="muted">{t("ui.guide.p4_note")}</p>
        </>
      ),
    },
  ];
  const STEPS = pages.length;
  const page = pages[step];
  const last = step === STEPS - 1;

  return (
    <div className="modal-backdrop guide-backdrop">
      <div className="guide" role="dialog" aria-modal aria-label={t("ui.guide.aria")}>
        <div className="guide-art" key={`art-${step}`}>
          {page.art}
        </div>
        <div className={`guide-text ${dir > 0 ? "fwd" : "back"}`} key={`text-${step}`}>
          <div className="guide-step">{t("ui.guide.step", { step: step + 1, total: STEPS })}</div>
          <h2>{page.title}</h2>
          {page.body}
        </div>
        <footer className="guide-foot">
          <div className="guide-dots" role="tablist" aria-label={t("ui.guide.steps_aria")}>
            {pages.map((p, i) => (
              <button key={i} type="button" role="tab" aria-selected={i === step} aria-label={p.title} className={i === step ? "on" : ""} onClick={() => go(i)} />
            ))}
          </div>
          <button type="button" className="btn subtle" onClick={step === 0 ? skip : () => go(step - 1)}>
            {step === 0 ? t("ui.guide.skip") : t("ui.guide.prev")}
          </button>
          <button type="button" className="btn primary" onClick={last ? finish : () => go(step + 1)} autoFocus>
            {last ? (chosen.size ? t("ui.guide.finish_disable", { count: chosen.size }) : t("ui.guide.finish")) : t("ui.guide.next")}
          </button>
        </footer>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------- 插图（纯 SVG + CSS 动画，循环播放）

function GestureArt() {
  return (
    <svg viewBox="0 0 320 180" className="art art-gesture" aria-hidden>
      <rect className="art-window" x="20" y="14" width="280" height="152" rx="10" />
      <rect className="art-bar" x="20" y="14" width="280" height="22" rx="10" />
      <circle className="art-dot" cx="36" cy="25" r="3" />
      <circle className="art-dot" cx="46" cy="25" r="3" />
      <circle className="art-dot" cx="56" cy="25" r="3" />
      <polyline className="art-trail" points="128,56 128,110 196,110" pathLength="100" />
      <circle className="art-start" cx="128" cy="56" r="4.5" />
      <g className="art-label">
        <rect x="110" y="132" width="100" height="24" rx="8" />
        <text x="160" y="148" textAnchor="middle">
          {t("ui.guide.art_close")}
        </text>
      </g>
    </svg>
  );
}

function EdgeArt() {
  return (
    <svg viewBox="0 0 320 180" className="art art-edge" aria-hidden>
      <rect className="art-window" x="20" y="14" width="280" height="152" rx="10" />
      <path className="art-corner" d="M20 136 v20 a10 10 0 0 0 10 10 h20" />
      <path className="art-cursor" d="M0 0 L0 15 L4 11 L7 18 L10 17 L7 10 L12 10 Z" />
      <g className="art-label">
        <rect x="110" y="70" width="100" height="24" rx="8" />
        <text x="160" y="86" textAnchor="middle">
          {t("ui.guide.art_start_menu")}
        </text>
      </g>
    </svg>
  );
}

function AppsArt({ presets }: { presets: PresetApp[] | null }) {
  const shown = (presets ?? []).slice(0, 3);
  return (
    <div className="art art-apps" aria-hidden>
      {(shown.length ? shown : [null, null, null]).map((p, i) => (
        <div key={i} className="art-app" style={{ "--i": i } as React.CSSProperties}>
          {p ? <AppIcon exe={p.path ?? p.exe} size={28} /> : <span className="art-app-blank" />}
          <span className="art-switch" />
        </div>
      ))}
    </div>
  );
}

function TrayArt() {
  return (
    <svg viewBox="0 0 320 180" className="art art-tray" aria-hidden>
      <rect className="art-taskbar" x="20" y="138" width="280" height="28" rx="6" />
      <rect className="art-tray-icon" x="246" y="145" width="14" height="14" rx="3.5" />
      <g className="art-menu">
        <rect x="186" y="62" width="104" height="70" rx="8" />
        <text x="200" y="84">{t("tray.menu.settings")}</text>
        <text x="200" y="104">{t("tray.menu.pause")}</text>
        <text x="200" y="124">{t("tray.menu.quit")}</text>
      </g>
    </svg>
  );
}
