import { useEffect, type RefObject } from "react";
import { api } from "./api";

/** 当前上报给引擎的画布；同时挂载多个时只上报一个，卸载后由其余画布接管。 */
let owner: object | null = null;

/**
 * 把录制画布的可见区域告诉引擎：在画布里按触发键画手势时，引擎不拦截。
 * 尺寸变化、滚动时立即上报；其余布局变化（面板展开、引擎重启）和丢失的消息由定时上报兜底。
 */
export function useRecorderArea(ref: RefObject<HTMLElement | null>) {
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const token = {};
    let frame = 0;
    const send = () => {
      frame = 0;
      if (owner && owner !== token) return;
      owner = token;
      const r = el.getBoundingClientRect();
      let left = Math.max(0, r.left), top = Math.max(0, r.top);
      let right = Math.min(window.innerWidth, r.right), bottom = Math.min(window.innerHeight, r.bottom);
      // 滚动容器会裁掉画布被卷走的部分，那里显示的是别的控件
      for (let p = el.parentElement; p; p = p.parentElement) {
        if (getComputedStyle(p).overflow === "visible") continue;
        const c = p.getBoundingClientRect();
        left = Math.max(left, c.left); top = Math.max(top, c.top);
        right = Math.min(right, c.right); bottom = Math.min(bottom, c.bottom);
      }
      const x = left, y = top, w = right - x, h = bottom - y;
      api.setRecorderArea(w > 0 && h > 0 ? { x, y, w, h } : null).catch(() => {});
    };
    const schedule = () => { frame ||= window.setTimeout(send, 50); };
    const ro = new ResizeObserver(schedule);
    ro.observe(el);
    window.addEventListener("scroll", schedule, true);
    window.addEventListener("resize", schedule);
    const timer = setInterval(schedule, 2000);
    schedule();
    return () => {
      ro.disconnect();
      window.removeEventListener("scroll", schedule, true);
      window.removeEventListener("resize", schedule);
      clearInterval(timer);
      clearTimeout(frame);
      if (owner === token) {
        owner = null;
        api.setRecorderArea(null).catch(() => {});
      }
    };
  }, [ref]);
}
