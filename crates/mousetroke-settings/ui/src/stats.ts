/** 使用统计的汇总（纯函数）：引擎按本地日期写 stats.json，这里按视图范围汇总。 */

export interface StatsDay {
  bindings?: Record<string, number>;
  apps?: Record<string, number>;
  hours?: number[];
}

export interface UsageStats {
  days: Record<string, StatsDay>;
  /** 绑定 id → 最近一次触发时的名称与作用域（绑定删除后仍能显示）。 */
  bindings: Record<string, { name: string; name_key?: string; scope: string }>;
}

export interface Summary {
  total: number;
  /** 按次数从多到少。 */
  bindings: [string, number][];
  apps: [string, number][];
  hours: number[];
}

/** 本地日期键 YYYY-MM-DD（与引擎写入的格式一致）。 */
export function dayKey(d: Date): string {
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

export function addDays(d: Date, n: number): Date {
  return new Date(d.getFullYear(), d.getMonth(), d.getDate() + n);
}

/** 截至 today（含）的最近 n 天，从早到晚。 */
export function lastDays(today: Date, n: number): Date[] {
  return Array.from({ length: n }, (_, i) => addDays(today, i - n + 1));
}

export function dayTotal(day: StatsDay | undefined): number {
  return Object.values(day?.bindings ?? {}).reduce((a, b) => a + b, 0);
}

export function summarize(stats: UsageStats, keys: Iterable<string>): Summary {
  const bindings = new Map<string, number>();
  const apps = new Map<string, number>();
  const hours = Array<number>(24).fill(0);
  let total = 0;
  for (const key of keys) {
    const day = stats.days[key];
    if (!day) continue;
    for (const [id, n] of Object.entries(day.bindings ?? {})) {
      bindings.set(id, (bindings.get(id) ?? 0) + n);
      total += n;
    }
    for (const [app, n] of Object.entries(day.apps ?? {})) apps.set(app, (apps.get(app) ?? 0) + n);
    (day.hours ?? []).slice(0, 24).forEach((n, h) => (hours[h] += n));
  }
  const sorted = (m: Map<string, number>) => [...m].sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]));
  return { total, bindings: sorted(bindings), apps: sorted(apps), hours };
}

/** 色阶 0–4：0 = 没有触发；其余按占最大值的比例分四档。 */
export function level(count: number, max: number): number {
  if (count <= 0 || max <= 0) return 0;
  return Math.min(4, Math.max(1, Math.ceil((count / max) * 4)));
}

/** 年视图（GitHub 式）：每列一周、周一在上；不在该年的格子为 null。 */
export function yearWeeks(year: number): (Date | null)[][] {
  const first = new Date(year, 0, 1);
  const start = addDays(first, -((first.getDay() + 6) % 7));
  const weeks: (Date | null)[][] = [];
  for (let d = start; d.getFullYear() <= year; ) {
    const week: (Date | null)[] = [];
    for (let i = 0; i < 7; i++, d = addDays(d, 1)) week.push(d.getFullYear() === year ? d : null);
    weeks.push(week);
  }
  return weeks;
}

/** 30 天视图（日历式）：每行一周、周一在左；范围外的格子为 null。 */
export function calendarWeeks(days: Date[]): (Date | null)[][] {
  if (!days.length) return [];
  const lead = (days[0].getDay() + 6) % 7;
  const cells: (Date | null)[] = [...Array<null>(lead).fill(null), ...days];
  while (cells.length % 7) cells.push(null);
  return Array.from({ length: cells.length / 7 }, (_, i) => cells.slice(i * 7, i * 7 + 7));
}

/** 有数据的年份到今年（年视图可切换的范围）。 */
export function statsYears(stats: UsageStats, today: Date): number[] {
  const years = Object.keys(stats.days).map((k) => Number(k.slice(0, 4))).filter((y) => y > 1970);
  const from = Math.min(today.getFullYear(), ...years);
  return Array.from({ length: today.getFullYear() - from + 1 }, (_, i) => from + i);
}
