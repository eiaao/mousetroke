import { useSyncExternalStore } from "react";
import { api } from "./api";

// 新版本提示的共享状态：侧栏圆点、顶部横幅、关于页都读同一份
let hint: string | null = null;
const listeners = new Set<() => void>();

function set(next: string | null) {
  if (next === hint) return;
  hint = next;
  listeners.forEach((fn) => fn());
}

function subscribe(fn: () => void) {
  listeners.add(fn);
  return () => { listeners.delete(fn); };
}

/** 当前应提示的新版本号；没有则为 null。 */
export function useUpdateHint(): string | null {
  return useSyncExternalStore(subscribe, () => hint, () => null);
}

/** 从后端本地记录重新读取提示（不联网），失败时保持原状。 */
export async function refreshUpdateHint(): Promise<void> {
  try { set(await api.updateHint()); } catch { /* 读取失败不影响界面 */ }
}

/** 忽略某个版本，然后刷新提示。 */
export async function ignoreUpdateVersion(version: string): Promise<void> {
  try { await api.ignoreUpdate(version); } catch { /* 忽略失败时下面的刷新会保留提示 */ }
  await refreshUpdateHint();
}
