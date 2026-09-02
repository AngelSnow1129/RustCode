// i18n 目录守卫：zh/en 键集必须一致，且无 React 依赖的查表器
// （translate/resolveI18n/readStoredLang）行为正确。
import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  i18nKeyMismatches,
  resolveI18n,
  readStoredLang,
  translate,
  DEFAULT_LANG,
  LANG_STORAGE_KEY,
  messages,
  type MsgKey,
} from './i18n.ts';

test('i18n: zh and en catalogs have identical key sets', () => {
  const { onlyZh, onlyEn } = i18nKeyMismatches();
  assert.deepEqual(onlyZh, [], `keys only in zh: ${onlyZh.join(', ')}`);
  assert.deepEqual(onlyEn, [], `keys only in en: ${onlyEn.join(', ')}`);
});

test('i18n: product default is Simplified Chinese (mirrors Rust Locale::default)', () => {
  assert.equal(DEFAULT_LANG, 'zh');
});

test('i18n: resolveI18n interpolates {name} placeholders in both locales', () => {
  assert.equal(resolveI18n('en', 'notify.title.done'), 'RustCode done');
  assert.equal(resolveI18n('zh', 'notify.title.done'), 'RustCode 已完成');
  // 带占位符的既有键
  assert.match(resolveI18n('en', 'cmd.cost.body', { tokens: '1k', turns: 2 }), /1k tokens/);
});

test('i18n: translate() falls back to default lang without localStorage', () => {
  // node --test 环境无 localStorage：readStoredLang 必须回落到 DEFAULT_LANG。
  assert.equal(readStoredLang(), DEFAULT_LANG);
  assert.equal(translate('notify.status.done'), messages[DEFAULT_LANG]['notify.status.done']);
});

test('i18n: readStoredLang honors the persisted rustcode.lang preference', () => {
  const store = new Map<string, string>();
  (globalThis as Record<string, unknown>).localStorage = {
    getItem: (k: string) => (store.has(k) ? store.get(k)! : null),
    setItem: (k: string, v: unknown) => void store.set(k, String(v)),
    removeItem: (k: string) => void store.delete(k),
  };
  try {
    assert.equal(readStoredLang(), DEFAULT_LANG);
    store.set(LANG_STORAGE_KEY, 'en');
    assert.equal(readStoredLang(), 'en');
    assert.equal(translate('notify.status.done'), 'Done');
    store.set(LANG_STORAGE_KEY, 'zh');
    assert.equal(translate('notify.status.done'), '已完成');
    // 未知值回落到默认语言
    store.set(LANG_STORAGE_KEY, 'fr');
    assert.equal(readStoredLang(), DEFAULT_LANG);
  } finally {
    delete (globalThis as Record<string, unknown>).localStorage;
  }
});

test('i18n: every value is a non-empty string', () => {
  for (const lang of ['zh', 'en'] as const) {
    for (const [key, value] of Object.entries(messages[lang]) as [MsgKey, string][]) {
      assert.equal(typeof value, 'string', `${lang}.${key} is not a string`);
      assert.ok(value.length > 0, `${lang}.${key} is empty`);
    }
  }
});
