/**
 * Utility functions for formatting quota timestamps, countdowns, and progress bars
 */

export interface FormattedCountdown {
  timeStr: string;
  dateStr: string;
  fullStr: string;
}

/**
 * Formats a Unix timestamp (epoch seconds or ms or ISO string) into human-readable countdown & timestamp
 * e.g. "4h 31m (10/04 19:40)" or "6d 18h (10/11 10:03)"
 */
export function formatResetCountdown(resetAt: number | string | null | undefined): FormattedCountdown {
  if (resetAt === null || resetAt === undefined || resetAt === "") {
    return {
      timeStr: "--",
      dateStr: "",
      fullStr: "无限制",
    };
  }

  let ts: number;
  if (typeof resetAt === "string") {
    const num = Number(resetAt);
    if (!isNaN(num)) {
      ts = num;
    } else {
      const parsed = Date.parse(resetAt);
      ts = isNaN(parsed) ? 0 : Math.floor(parsed / 1000);
    }
  } else {
    ts = Number(resetAt);
  }

  if (!ts || isNaN(ts) || ts <= 0) {
    return {
      timeStr: "--",
      dateStr: "",
      fullStr: "无限制",
    };
  }

  // If ts is in milliseconds (> 100 billion, i.e. > year 1973 in ms)
  if (ts > 100_000_000_000) {
    ts = Math.floor(ts / 1000);
  }

  const now = Math.floor(Date.now() / 1000);
  const diff = ts - now;

  const resetDate = new Date(ts * 1000);
  if (isNaN(resetDate.getTime())) {
    return {
      timeStr: "--",
      dateStr: "",
      fullStr: "无限制",
    };
  }

  const mm = String(resetDate.getMonth() + 1).padStart(2, "0");
  const dd = String(resetDate.getDate()).padStart(2, "0");
  const hh = String(resetDate.getHours()).padStart(2, "0");
  const min = String(resetDate.getMinutes()).padStart(2, "0");
  const dateStr = `${mm}/${dd} ${hh}:${min}`;

  if (diff <= 0) {
    return {
      timeStr: "已就绪",
      dateStr,
      fullStr: `已就绪 (${dateStr})`,
    };
  }

  const days = Math.floor(diff / 86400);
  const hours = Math.floor((diff % 86400) / 3600);
  const mins = Math.floor((diff % 3600) / 60);

  let timeStr = "";
  if (days >= 2) {
    timeStr = hours > 0 ? `${days}d ${hours}h` : `${days}d`;
  } else if (days === 1) {
    timeStr = hours > 0 ? `1d ${hours}h` : `1d`;
  } else if (hours > 0) {
    timeStr = mins > 0 ? `${hours}h ${mins}m` : `${hours}h`;
  } else {
    timeStr = `${Math.max(1, mins)}m`;
  }

  return {
    timeStr,
    dateStr,
    fullStr: `${timeStr} (${dateStr})`,
  };
}

/**
 * Returns color classes for progress bar
 * >= 50% : Emerald Green
 * 20% - 49% : Warm Amber Orange
 * < 20% : Coral Rose Red
 */
export function getProgressColor(percent: number | string | null | undefined): string {
  if (percent === null || percent === undefined) return "bg-slate-200";
  const p = Number(percent);
  if (isNaN(p)) return "bg-slate-200";
  if (p >= 50) return "bg-emerald-500";
  if (p >= 20) return "bg-amber-500";
  return "bg-rose-500";
}

/**
 * Returns text color for percentage values
 */
export function getPercentTextColor(percent: number | string | null | undefined): string {
  if (percent === null || percent === undefined) return "text-slate-400";
  const p = Number(percent);
  if (isNaN(p)) return "text-slate-400";
  if (p >= 50) return "text-emerald-600";
  if (p >= 20) return "text-amber-600";
  return "text-rose-600";
}

/**
 * Normalizes model name by removing difficulty tier qualifiers like (High), (Medium), (Low)
 */
export function normalizeModelName(name: string): string {
  if (!name) return "";
  return name.replace(/\s*\((High|Medium|Low)\)/gi, "").trim();
}

/**
 * Merges individual models that share the same base name (ignoring difficulty tiers).
 * Merges quota by taking the bottle-necked remaining percentage and reset time so the user
 * gets an accurate, consolidated view of each unique model.
 */
export function getMergedIndividualModels<T extends { 
  id?: string;
  model_name: string; 
  remaining_percent?: number | null; 
  remaining_value?: number | null;
  reset_at?: number | string | null;
}>(
  quotas: T[]
): T[] {
  if (!Array.isArray(quotas)) return [];

  const summaryNames = new Set(["Claude (5h)", "Claude (Weekly)", "Gemini (5h)", "Gemini (Weekly)", "AI Credits"]);
  const map = new Map<string, T>();

  for (const q of quotas) {
    if (!q || typeof q.model_name !== "string") continue;
    if (summaryNames.has(q.model_name)) continue;

    const baseName = normalizeModelName(q.model_name);
    if (!baseName) continue;

    const existing = map.get(baseName);

    if (!existing) {
      map.set(baseName, {
        ...q,
        model_name: baseName,
      });
      continue;
    }

    const pNew = q.remaining_percent !== null && q.remaining_percent !== undefined ? Number(q.remaining_percent) : null;
    const pOld = existing.remaining_percent !== null && existing.remaining_percent !== undefined ? Number(existing.remaining_percent) : null;

    let mergedPercent = existing.remaining_percent;
    let mergedResetAt = existing.reset_at;

    if (pNew !== null && pOld !== null) {
      // Pick the lowest quota (bottleneck constraint)
      if (pNew < pOld) {
        mergedPercent = pNew;
        mergedResetAt = q.reset_at;
      } else {
        mergedPercent = pOld;
      }
    } else if (pNew !== null) {
      mergedPercent = pNew;
      mergedResetAt = q.reset_at;
    }

    map.set(baseName, {
      ...existing,
      model_name: baseName,
      remaining_percent: mergedPercent,
      reset_at: mergedResetAt,
    });
  }

  return Array.from(map.values()).sort((a, b) => a.model_name.localeCompare(b.model_name));
}
