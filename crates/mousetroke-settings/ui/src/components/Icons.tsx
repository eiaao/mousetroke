import { useEffect, useState } from "react";
import type { MouseButton } from "../types";
import { api } from "../api";

/** 关于页的 GitHub Star 图标。 */
export function StarIcon() {
  return (
    <svg className="ico" width="16" height="16" viewBox="0 0 16 16" aria-hidden>
      <path className="ico-line" d="m8 1.5 2 4.1 4.5.7-3.3 3.2.8 4.5-4-2.1L4 14l.8-4.5L1.5 6.3l4.5-.7Z" />
    </svg>
  );
}

/** 小鼠标图标，高亮指定按键（筛选标签、触发方式用）。 */
export function MouseIcon({ button, size = 16 }: { button?: MouseButton; size?: number }) {
  return (
    <svg className="ico" width={size} height={size} viewBox="0 0 16 16" aria-hidden>
      {button === "left" && <path className="ico-hot" d="M8 1.5H7.2A4.7 4.7 0 0 0 3.3 6.2V7H8Z" />}
      {button === "right" && <path className="ico-hot" d="M8 1.5h.8a4.7 4.7 0 0 1 3.9 4.7V7H8Z" />}
      {button === "middle" && <rect className="ico-hot" x="7" y="3" width="2" height="3.4" rx="1" />}
      {button === "x1" && <rect className="ico-hot" x="1.4" y="8.2" width="2.2" height="2.4" rx="1" />}
      {button === "x2" && <rect className="ico-hot" x="1.4" y="5.4" width="2.2" height="2.4" rx="1" />}
      <rect className="ico-line" x="3.3" y="1.5" width="9.4" height="13" rx="4.7" />
      <path className="ico-line" d="M3.3 7h9.4M8 1.5V7" />
    </svg>
  );
}

/** 屏幕缩略图，高亮指定边/角（位标志：上1 右2 下4 左8）。 */
export function EdgeIcon({ edge, size = 16 }: { edge?: number; size?: number }) {
  const e = edge ?? 0;
  const corner = [3, 6, 9, 12].includes(e);
  const hot: React.ReactNode[] = [];
  if (corner) {
    const x = e & 2 ? 10.5 : 1.5;
    const y = e & 4 ? 9.5 : 2.5;
    hot.push(<rect key="c" className="ico-hot" x={x} y={y} width="4" height="4" rx="1" />);
  } else if (e === 1) hot.push(<rect key="t" className="ico-hot" x="1.5" y="2.5" width="13" height="2.2" rx="1" />);
  else if (e === 4) hot.push(<rect key="b" className="ico-hot" x="1.5" y="11.3" width="13" height="2.2" rx="1" />);
  else if (e === 8) hot.push(<rect key="l" className="ico-hot" x="1.5" y="2.5" width="2.2" height="11" rx="1" />);
  else if (e === 2) hot.push(<rect key="r" className="ico-hot" x="12.3" y="2.5" width="2.2" height="11" rx="1" />);
  return (
    <svg className="ico" width={size} height={size} viewBox="0 0 16 16" aria-hidden>
      {hot}
      <rect className="ico-line" x="1.5" y="2.5" width="13" height="11" rx="2" />
    </svg>
  );
}

/** 侧栏图标。 */
export function NavIcon({ kind }: { kind: "global" | "desktop" | "app" | "prefs" | "data" | "add" | "pick" | "about" }) {
  const p = {
    global: (
      <>
        <circle className="ico-line" cx="8" cy="8" r="6.2" />
        <path className="ico-line" d="M1.8 8h12.4M8 1.8c2 2 2 10.4 0 12.4M8 1.8c-2 2-2 10.4 0 12.4" />
      </>
    ),
    desktop: (
      <>
        <rect className="ico-line" x="1.5" y="2.5" width="13" height="9" rx="1.6" />
        <path className="ico-line" d="M5.5 14h5M8 11.5V14" />
      </>
    ),
    app: (
      <>
        <rect className="ico-line" x="1.5" y="2.5" width="13" height="11" rx="2" />
        <path className="ico-line" d="M1.5 5.5h13" />
      </>
    ),
    prefs: (
      <>
        <path className="ico-line" d="M2 4.5h7M12 4.5h2M2 11.5h2M7 11.5h7" />
        <circle className="ico-line" cx="10.5" cy="4.5" r="1.6" />
        <circle className="ico-line" cx="5.5" cy="11.5" r="1.6" />
      </>
    ),
    data: <path className="ico-line" d="M8 2v8M4.8 6.8 8 10l3.2-3.2M2.5 11.5v1.2c0 .7.6 1.3 1.3 1.3h8.4c.7 0 1.3-.6 1.3-1.3v-1.2" />,
    add: <path className="ico-line" d="M8 3v10M3 8h10" />,
    about: (
      <>
        <circle className="ico-line" cx="8" cy="8" r="6.2" />
        <path className="ico-line" d="M8 7.2v4M8 4.9v.1" />
      </>
    ),
    pick: (
      <>
        <circle className="ico-line" cx="8" cy="8" r="4.6" />
        <path className="ico-line" d="M8 1v3.2M8 11.8V15M1 8h3.2M11.8 8H15" />
      </>
    ),
  }[kind];
  return (
    <svg className="ico" width="16" height="16" viewBox="0 0 16 16" aria-hidden>
      {p}
    </svg>
  );
}

// ---------------------------------------------------------------- 程序图标

/** 已取到的图标（键 = 小写 exe；null = 取不到，用通用图标）。 */
const iconCache = new Map<string, string | null>();
const iconWaiters = new Map<string, ((v: string | null) => void)[]>();
let iconBatch: string[] = [];

/** 同一帧内的请求合并成一次后端调用。 */
function loadIcon(exe: string): Promise<string | null> {
  const key = exe.toLowerCase();
  if (iconCache.has(key)) return Promise.resolve(iconCache.get(key)!);
  return new Promise((resolve) => {
    const waiting = iconWaiters.get(key);
    if (waiting) return void waiting.push(resolve);
    iconWaiters.set(key, [resolve]);
    if (iconBatch.push(exe) === 1) {
      setTimeout(() => {
        const batch = iconBatch;
        iconBatch = [];
        const settle = (res: Record<string, string | null>) =>
          batch.forEach((e) => {
            const k = e.toLowerCase();
            iconCache.set(k, res[e] ?? null);
            iconWaiters.get(k)?.forEach((r) => r(res[e] ?? null));
            iconWaiters.delete(k);
          });
        api.appIcons(batch).then(settle, () => settle({}));
      }, 0);
    }
  });
}

/** 程序自己的图标；取不到时显示通用窗口图标。 */
export function AppIcon({ exe, size = 16 }: { exe?: string | null; size?: number }) {
  const key = exe?.toLowerCase() ?? "";
  const [src, setSrc] = useState<string | null | undefined>(() => (exe ? iconCache.get(key) : null));
  useEffect(() => {
    if (!exe) return setSrc(null);
    let live = true;
    setSrc(iconCache.get(key));
    loadIcon(exe).then((v) => live && setSrc(v));
    return () => {
      live = false;
    };
  }, [exe, key]);
  if (src) return <img className="app-ico" src={src} width={size} height={size} alt="" draggable={false} />;
  return (
    <span className="app-ico app-ico-generic" style={{ width: size, height: size }}>
      <NavIcon kind="app" />
    </span>
  );
}
