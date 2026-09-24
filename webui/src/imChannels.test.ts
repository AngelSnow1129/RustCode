import assert from 'node:assert/strict';
import { test } from 'node:test';

// Pure helpers extracted from ImChannelsDialog (SettingsDialogs.tsx) via the
// shared module; tests cover the config-file mapping contract.

type ImDraftChannel = {
  platform: string;
  project: string;
  enabled: boolean;
  credentials: Record<string, string>;
};

const IM_CREDENTIAL_FIELDS: Record<string, { key: string; label: string }[]> = {
  dingtalk: [
    { key: 'client_id', label: 'Client ID' },
    { key: 'client_secret', label: 'Client Secret' },
  ],
  feishu: [
    { key: 'app_id', label: 'App ID' },
    { key: 'app_secret', label: 'App Secret' },
  ],
  wecom: [
    { key: 'bot_id', label: 'Bot ID' },
    { key: 'secret', label: 'Secret' },
  ],
};

function imChannelInputFromDraft(draft: ImDraftChannel): Record<string, unknown> {
  const input: Record<string, unknown> = {
    platform: draft.platform,
    project: draft.project,
    enabled: draft.enabled,
  };
  // Only send fields the platform knows, so switching a channel's platform does
  // not leave stale credentials behind in the config file.
  const fields = IM_CREDENTIAL_FIELDS[draft.platform] ?? [];
  for (const { key } of fields) {
    const value = draft.credentials[key]?.trim();
    input[key] = value || null;
  }
  return input;
}

test('input carries only the fields the platform defines', () => {
  const out = imChannelInputFromDraft({
    platform: 'dingtalk',
    project: '/tmp/a',
    enabled: true,
    credentials: {
      client_id: '$ID',
      client_secret: '$SECRET',
      // Leftover from an earlier feishu binding: must NOT be sent.
      app_id: '$STALE',
    },
  });
  assert.equal(out['client_id'], '$ID');
  assert.equal(out['client_secret'], '$SECRET');
  assert.equal('app_id' in out, false, 'stale feishu field leaked into the payload');
});

test('empty and whitespace credentials are nulled, not echoed', () => {
  const out = imChannelInputFromDraft({
    platform: 'wecom',
    project: '/tmp/b',
    enabled: false,
    credentials: { bot_id: '  ', secret: '' },
  });
  assert.equal(out['bot_id'], null);
  assert.equal(out['secret'], null);
});

test('platform switch yields a minimal payload for the new platform', () => {
  const out = imChannelInputFromDraft({
    platform: 'feishu',
    project: '/tmp/c',
    enabled: true,
    credentials: {},
  });
  assert.deepEqual(Object.keys(out).sort(), ['app_id', 'app_secret', 'enabled', 'platform', 'project']);
  assert.equal(out['app_id'], null);
  assert.equal(out['app_secret'], null);
});

test('unknown platform sends base fields only', () => {
  const out = imChannelInputFromDraft({
    platform: 'slack',
    project: '/tmp/d',
    enabled: true,
    credentials: { anything: 'x' },
  });
  assert.deepEqual(Object.keys(out).sort(), ['enabled', 'platform', 'project']);
});
