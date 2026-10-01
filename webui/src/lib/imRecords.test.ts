// IM records browser: tree flattening/filtering + breadcrumb segments.
// Pure helpers only — no fetch, no React, no DOM.
import { test } from 'node:test';
import assert from 'node:assert';
import {
  IM_ROOT_SELECTION,
  imBreadcrumbs,
  imFindProject,
  imFindSession,
  imFlattenBindings,
  imHasRecords,
  imPlatformRows,
  imPlatformStatus,
  imProjectRows,
  imSessionRows,
  imTotalBindings,
  type ImRecordSelection,
} from './imRecords.ts';

const binding = (over: Partial<{
  platform: string;
  chat_id: string;
  project: string;
  session_id: string;
  created_at: number;
  updated_at: number;
}> = {}) => ({
  platform: 'dingtalk',
  chat_id: 'chat-1',
  project: '/w/a',
  session_id: 'sess-1',
  created_at: 100,
  updated_at: 200,
  ...over,
});

const tree = {
  enabled: true,
  total: 3,
  platforms: [
    {
      platform: 'dingtalk',
      known_platform: true,
      configured: true,
      channel_enabled: true,
      binding_count: 2,
      project_count: 1,
      last_active_at: 300,
      projects: [
        {
          project: '/w/a',
          binding_count: 2,
          session_count: 2,
          last_active_at: 300,
          bindings: [
            binding({ session_id: 'sess-1', chat_id: 'chat-1', created_at: 100, updated_at: 200 }),
            binding({ session_id: 'sess-2', chat_id: 'chat-2', created_at: 110, updated_at: 300 }),
          ],
        },
      ],
    },
    {
      // Configured but never used: no bindings at all. Must still be listed.
      platform: 'feishu',
      known_platform: true,
      configured: true,
      channel_enabled: false,
      binding_count: 0,
      project_count: 0,
      last_active_at: 0,
      projects: [],
    },
  ],
};

test('im records: an empty (or missing) tree yields no rows and no records', () => {
  assert.deepEqual(imPlatformRows(null), []);
  assert.deepEqual(imPlatformRows({ enabled: false, total: 0, platforms: [] }), []);
  assert.equal(imHasRecords(null), false);
  assert.equal(imHasRecords({ enabled: false, total: 0, platforms: [] }), false);
  assert.equal(imTotalBindings(null), 0);
  assert.deepEqual(imProjectRows(null, IM_ROOT_SELECTION), []);
  assert.deepEqual(imSessionRows({ enabled: true, total: 0, platforms: [] }, { platform: 'x', project: 'y' }), []);
});

test('im records: configured-but-unused platform (binding_count 0) is still listed', () => {
  const rows = imPlatformRows(tree);
  assert.deepEqual(rows.map((r) => r.platform), ['dingtalk', 'feishu']);
  const feishu = rows.find((r) => r.platform === 'feishu')!;
  assert.equal(feishu.binding_count, 0);
  assert.equal(feishu.configured, true);
  assert.equal(feishu.channel_enabled, false);
  // Drilling into it shows an empty project list, not a crash.
  assert.deepEqual(imProjectRows(tree, { platform: 'feishu', project: null }), []);
});

test('im records: project grouping stays inside the selected platform', () => {
  const multi = {
    enabled: true,
    total: 2,
    platforms: [
      {
        platform: 'wecom',
        known_platform: true,
        configured: true,
        channel_enabled: true,
        binding_count: 2,
        project_count: 2,
        last_active_at: 0,
        projects: [
          {
            project: '/w/a',
            binding_count: 1,
            session_count: 1,
            last_active_at: 0,
            bindings: [binding({ platform: 'wecom', project: '/w/a', session_id: 'wa', updated_at: 10 })],
          },
          {
            project: '/w/b',
            binding_count: 1,
            session_count: 1,
            last_active_at: 0,
            bindings: [binding({ platform: 'wecom', project: '/w/b', session_id: 'wb', updated_at: 20 })],
          },
        ],
      },
    ],
  };
  const projRows = imProjectRows(multi, { platform: 'wecom', project: null });
  assert.deepEqual(projRows.map((p) => p.project), ['/w/b', '/w/a'], 'newest project first');
  assert.deepEqual(imProjectRows(multi, { platform: 'dingtalk', project: null }), [], 'other platform has no leak');
  // A project selected on the wrong platform resolves to nothing.
  assert.equal(imFindProject(multi, 'dingtalk', '/w/a'), null);
  assert.equal(imFindProject(multi, 'wecom', '/w/b')?.project, '/w/b');
});

test('im records: session leaves are newest-first inside the selected project', () => {
  const sel: ImRecordSelection = { platform: 'dingtalk', project: '/w/a' };
  const rows = imSessionRows(tree, sel);
  assert.deepEqual(rows.map((r) => r.session_id), ['sess-2', 'sess-1']);
  assert.equal(rows[0].chat_id, 'chat-2');
  assert.equal(rows[0].project, '/w/a');
  // Without a project selection there is no leaf list yet.
  assert.deepEqual(imSessionRows(tree, { platform: 'dingtalk', project: null }), []);
});

test('im records: a session leaf is found by full or short id', () => {
  const sel: ImRecordSelection = { platform: 'dingtalk', project: '/w/a' };
  assert.equal(imFindSession(tree, sel, 'sess-1')?.session_id, 'sess-1');
  assert.equal(imFindSession(tree, sel, 'sess')?.session_id, 'sess-2', 'short id matches the newest prefix hit');
  assert.equal(imFindSession(tree, sel, 'nope'), null);
  assert.equal(imFindSession(tree, sel, null), null);
});

test('im records: flattening honors the (platform, project) filter', () => {
  assert.equal(imFlattenBindings(tree).length, 2, 'no selection -> every binding');
  assert.equal(imFlattenBindings(tree, { platform: 'dingtalk', project: null }).length, 2);
  assert.equal(imFlattenBindings(tree, { platform: 'feishu', project: null }).length, 0);
  assert.equal(imFlattenBindings(tree, { platform: 'dingtalk', project: '/w/a' }).length, 2);
  assert.equal(imFlattenBindings(tree, { platform: 'dingtalk', project: '/w/nope' }).length, 0);
});

test('im records: platform status roll-up for the records popover', () => {
  // From the shared fixture: dingtalk is configured + enabled, feishu is
  // configured but its channel switch is off.
  const rows = imPlatformRows(tree);
  assert.deepEqual(
    rows.map((r) => imPlatformStatus(r)),
    ['enabled', 'disabled'],
  );
  // A platform without config can never receive messages, regardless of the
  // channel switch or leftover bindings from an older config file.
  assert.equal(imPlatformStatus({ configured: false, channel_enabled: true }), 'unconfigured');
  assert.equal(imPlatformStatus({ configured: false, channel_enabled: false }), 'unconfigured');
});

test('im records: totals come from the backend count', () => {
  assert.equal(imTotalBindings(tree), 3);
  // Missing total falls back to summing platform counts.
  assert.equal(imTotalBindings({ enabled: true, platforms: tree.platforms } as never), 2);
});

test('im records: breadcrumbs grow one segment per drilled level', () => {
  const root = imBreadcrumbs(IM_ROOT_SELECTION, { root: 'IM' });
  assert.equal(root.length, 1);
  assert.deepEqual(root[0], { label: 'IM', level: 0, selection: { platform: null, project: null } });

  const lvl1 = imBreadcrumbs({ platform: 'dingtalk', project: null }, { root: 'IM' });
  assert.deepEqual(lvl1.map((c) => c.label), ['IM', 'dingtalk']);
  assert.deepEqual(lvl1.map((c) => c.level), [0, 1]);
  assert.deepEqual(lvl1[1].selection, { platform: 'dingtalk', project: null });

  const lvl2 = imBreadcrumbs({ platform: 'dingtalk', project: '/w/a' }, { root: 'IM' });
  assert.deepEqual(lvl2.map((c) => c.label), ['IM', 'dingtalk', '/w/a']);
  assert.deepEqual(lvl2.map((c) => c.level), [0, 1, 2]);
  assert.deepEqual(lvl2[2].selection, { platform: 'dingtalk', project: '/w/a' });

  // A project without a platform is unreachable: no orphan segment.
  assert.deepEqual(imBreadcrumbs({ platform: null, project: '/w/a' }, { root: 'IM' }).map((c) => c.label), ['IM']);
});
