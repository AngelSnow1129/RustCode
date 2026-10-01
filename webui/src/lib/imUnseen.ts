// Client-side "new IM activity" detection for the sidebar IM entry.
//
// The daemon's read-only records tree already carries per-binding
// `updated_at` stamps (unix seconds); this module compares the latest one
// against a last-seen mark persisted in localStorage. Pure browser-local
// state: nothing is sent to the daemon and no server-side unread store
// exists. Opening the IM menu marks everything as seen.

import type { ImBindingsInfo } from '../api';

/** localStorage key for the last-seen mark (unix milliseconds). */
export const IM_LAST_SEEN_KEY = 'rustcode.im.lastSeen';

/** Normalizes a backend timestamp to unix milliseconds (0 when absent).
 *  Backend stamps are unix seconds; anything >= 1e12 is already ms. */
export function normalizeImTs(value: number | null | undefined): number {
  if (typeof value !== 'number' || !Number.isFinite(value) || value <= 0) return 0;
  return value < 1e12 ? value * 1000 : value;
}

/** Latest activity across the whole records tree (0 for an empty tree). */
export function imLatestActivity(tree: ImBindingsInfo | null | undefined): number {
  let latest = 0;
  for (const platform of tree?.platforms ?? []) {
    for (const project of platform.projects ?? []) {
      for (const binding of project.bindings ?? []) {
        const v = normalizeImTs(binding?.updated_at) || normalizeImTs(binding?.created_at);
        if (v > latest) latest = v;
      }
    }
  }
  return latest;
}

/** Whether any IM conversation changed after the last-seen mark (ms). */
export function imHasUnseen(tree: ImBindingsInfo | null | undefined, lastSeenMs: number): boolean {
  return imLatestActivity(tree) > lastSeenMs;
}

/** Reads the persisted last-seen mark (0 when storage is unavailable/empty). */
export function readImLastSeen(): number {
  try {
    if (typeof localStorage === 'undefined') return 0;
    const raw = localStorage.getItem(IM_LAST_SEEN_KEY);
    if (raw === null) return 0;
    const v = Number(raw);
    return Number.isFinite(v) && v > 0 ? v : 0;
  } catch {
    return 0;
  }
}

/** Persists the last-seen mark and returns it. Storage failures are ignored:
 *  the in-memory mark still clears the dot for this session. */
export function markImSeen(now = Date.now()): number {
  try {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(IM_LAST_SEEN_KEY, String(now));
    }
  } catch {
    /* storage unavailable: fall back to the in-memory mark only */
  }
  return now;
}
