// imUnseen: client-side unread detection for the sidebar IM entry.
// Pure helpers + a localStorage stub; each node:test file runs in its own
// process, so swapping globalThis.localStorage here cannot leak elsewhere.
import { test } from 'node:test';
import assert from 'node:assert';
import {
  IM_LAST_SEEN_KEY,
  imHasUnseen,
  imLatestActivity,
  markImSeen,
  normalizeImTs,
  readImLastSeen,
} from './imUnseen.ts';

class MemStorage {
  private map = new Map<string, string>();
  getItem(k: string): string | null {
    return this.map.has(k) ? (this.map.get(k) as string) : null;
  }
  setItem(k: string, v: string): void {
    this.map.set(k, String(v));
  }
  removeItem(k: string): void {
    this.map.delete(k);
  }
  clear(): void {
    this.map.clear();
  }
}

const tree = (bindings: { created: number; updated: number }[]) => ({
  enabled: true,
  total: bindings.length,
  platforms: [
    {
      platform: 'dingtalk',
      known_platform: true,
      configured: true,
      channel_enabled: true,
      binding_count: bindings.length,
      project_count: 1,
      last_active_at: Math.max(0, ...bindings.map((b) => b.updated)),
      projects: [
        {
          project: '/w/a',
          binding_count: bindings.length,
          session_count: bindings.length,
          last_active_at: Math.max(0, ...bindings.map((b) => b.updated)),
          bindings: bindings.map((b, i) => ({
            platform: 'dingtalk',
            chat_id: `c${i}`,
            project: '/w/a',
            session_id: `s${i}`,
            created_at: b.created,
            updated_at: b.updated,
          })),
        },
      ],
    },
  ],
});

test('normalizeImTs: seconds to ms, ms passthrough, junk to 0', () => {
  assert.equal(normalizeImTs(300), 300_000);
  assert.equal(normalizeImTs(1.7e13), 1.7e13);
  assert.equal(normalizeImTs(0), 0);
  assert.equal(normalizeImTs(-5), 0);
  assert.equal(normalizeImTs(Number.NaN), 0);
  assert.equal(normalizeImTs(null), 0);
  assert.equal(normalizeImTs(undefined), 0);
});

test('imLatestActivity: max across bindings, 0 for an empty tree', () => {
  assert.equal(imLatestActivity(null), 0);
  assert.equal(imLatestActivity({ enabled: true, total: 0, platforms: [] }), 0);
  assert.equal(
    imLatestActivity(tree([
      { created: 100, updated: 200 },
      { created: 150, updated: 900 },
    ])),
    900_000,
  );
  // Bindings with a zero/absent updated_at fall back to created_at.
  const t = tree([{ created: 12345, updated: 0 }]);
  (t.platforms[0].projects[0].bindings[0] as { updated_at: number }).updated_at = 0;
  assert.equal(imLatestActivity(t), 12_345_000);
});

test('imHasUnseen: strictly newer activity counts, equal does not', () => {
  const t = tree([{ created: 100, updated: 200 }]);
  const latest = imLatestActivity(t);
  assert.equal(imHasUnseen(t, latest), false);
  assert.equal(imHasUnseen(t, latest - 1), true);
  assert.equal(imHasUnseen(t, 0), true);
});

test('read/mark roundtrip via localStorage; corrupt values read as 0', () => {
  (globalThis as { localStorage?: unknown }).localStorage = new MemStorage();
  assert.equal(readImLastSeen(), 0);
  const now = markImSeen(1_700_000_000_000);
  assert.equal(now, 1_700_000_000_000);
  assert.equal(readImLastSeen(), 1_700_000_000_000);
  localStorage.setItem(IM_LAST_SEEN_KEY, 'not-a-number');
  assert.equal(readImLastSeen(), 0);
  localStorage.setItem(IM_LAST_SEEN_KEY, '-3');
  assert.equal(readImLastSeen(), 0);
});

test('storage-less environments: read 0, mark still returns the timestamp', () => {
  (globalThis as { localStorage?: unknown }).localStorage = undefined;
  assert.equal(readImLastSeen(), 0);
  assert.equal(markImSeen(1_700_000_000_001), 1_700_000_000_001);
});
