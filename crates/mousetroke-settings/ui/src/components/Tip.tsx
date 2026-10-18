import { useEffect, useLayoutEffect, useRef, useState } from "react";

/**
 * 统一的提示气泡：给任意元素加 `data-tip="…"` 即可（替代原生 title 的系统小黄框）。
 * 悬停 0.4 秒出现，已有气泡时移到相邻元素立即切换；键盘聚焦也会显示。
 * 默认在上方，贴近窗口顶部时翻到下方；左右贴边时自动内收（箭头仍指向元素）。
 */
export function TipLayer() {
  const [tip, setTip] = useState<{ text: string; x: number; y: number; below: boolean } | null>(null);
  const box = useRef<HTMLDivElement>(null);
  const [shift, setShift] = useState(0);

  useEffect(() => {
    let current: HTMLElement | null = null;
    let timer = 0;
    let visible = false;
    const place = (el: HTMLElement) => {
      const text = el.dataset.tip;
      if (!text) return;
      const r = el.getBoundingClientRect();
      const below = r.top < 64;
      visible = true;
      setTip({ text, x: r.left + r.width / 2, y: below ? r.bottom : r.top, below });
    };
    const hide = () => {
      clearTimeout(timer);
      current = null;
      visible = false;
      setTip(null);
    };
    const target = (e: Event) => (e.target as Element | null)?.closest?.("[data-tip]") as HTMLElement | null;
    const onOver = (e: Event) => {
      const el = target(e);
      if (el === current) return;
      clearTimeout(timer);
      current = el;
      if (!el) {
        timer = window.setTimeout(hide, 80);
        return;
      }
      timer = window.setTimeout(() => place(el), visible ? 0 : 420);
    };
    const onFocus = (e: Event) => {
      const el = target(e);
      if (el && (e.target as HTMLElement).matches(":focus-visible")) {
        current = el;
        place(el);
      }
    };
    document.addEventListener("mouseover", onOver);
    document.addEventListener("focusin", onFocus);
    document.addEventListener("focusout", hide);
    document.addEventListener("mousedown", hide, true);
    document.addEventListener("keydown", hide, true);
    window.addEventListener("scroll", hide, true);
    window.addEventListener("blur", hide);
    return () => {
      clearTimeout(timer);
      document.removeEventListener("mouseover", onOver);
      document.removeEventListener("focusin", onFocus);
      document.removeEventListener("focusout", hide);
      document.removeEventListener("mousedown", hide, true);
      document.removeEventListener("keydown", hide, true);
      window.removeEventListener("scroll", hide, true);
      window.removeEventListener("blur", hide);
    };
  }, []);

  // 贴边内收：量出气泡宽度后算水平偏移
  useLayoutEffect(() => {
    const el = box.current;
    if (!el || !tip) return;
    const half = el.offsetWidth / 2;
    const margin = 8;
    const left = tip.x - half;
    const right = tip.x + half;
    setShift(left < margin ? margin - left : right > window.innerWidth - margin ? window.innerWidth - margin - right : 0);
  }, [tip]);

  if (!tip) return null;
  return (
    <div
      ref={box}
      key={`${tip.text}-${tip.x}-${tip.y}`}
      className={`tip ${tip.below ? "below" : ""}`}
      role="tooltip"
      style={{ left: tip.x + shift, top: tip.y, "--arrow": `${-shift}px` } as React.CSSProperties}
    >
      {tip.text}
    </div>
  );
}
