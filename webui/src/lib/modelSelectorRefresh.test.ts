import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const read = (relative: string) =>
  readFileSync(fileURLToPath(new URL(relative, import.meta.url)), 'utf8');

const selector = () => read('../components/ModelSelector.tsx');
const chat = () => read('../components/Chat.tsx');
const app = () => read('../app.tsx');

test('opening the picker always re-fetches the model list', () => {
  // The core bug: a model added in the settings dialog was invisible in an
  // already-open conversation until a full page reload.
  const src = selector();
  assert.match(src, /const toggleModels = \(\) => \{[\s\S]*?if \(next\) void loadModels\(true\)/);
  assert.match(src, /const toggleEffort = \(\) => \{[\s\S]*?if \(next\) void loadModels\(true\)/);
});

test('interactive refresh bypasses the hands-off guard, background refresh keeps it', () => {
  // Background path must not shift the list under the cursor mid-selection;
  // the user-initiated path must, or a new model stays unselectable.
  const src = selector();
  assert.match(src, /if \(!interactive && \(openRef\.current \|\| effortOpenRef\.current\)\) return;/);
});

test('a stale in-flight response cannot overwrite a newer list', () => {
  const src = selector();
  assert.match(src, /const seq = seqRef\.current \+ 1;/);
  assert.match(src, /if \(seq !== seqRef\.current\) return;/);
});

test('no polling loop remains: refresh is event- and interaction-driven', () => {
  const src = selector();
  assert.doesNotMatch(src, /setInterval/);
  assert.doesNotMatch(src, /setTimeout\(\(\) => void loadModels/);
});

test('the fetch effect is stable: it does not re-run on every parent render', () => {
  // Parent handlers are captured in a ref, so the callbacks keep a stable
  // identity and the effect only fires on mount + `refreshSignal`.
  const src = selector();
  assert.match(src, /const onDefaultChangeRef = useRef\(onDefaultChange\);/);
  assert.match(src, /\}, \[\]\);[\s\S]*?const loadModels = useCallback/);
  assert.match(src, /\}, \[loadModels, refreshSignal\]\);/);
});

test('unmount stops an in-flight refresh from touching state', () => {
  const src = selector();
  assert.match(src, /if \(!activeRef\.current\) return;/);
  assert.match(src, /activeRef\.current = false;/);
});

test('a failed refresh keeps the last known list and stays retryable', () => {
  const src = selector();
  assert.match(src, /\} catch \{[\s\S]*?\/\/ Keep the last known list/);
  // The explicit retry lives at the bottom of the dropdown.
  assert.match(src, /onClick=\{\(\) => void loadModels\(true\)\}/);
});

test('the refresh affordance is a footer, not a fake model entry', () => {
  const src = selector();
  const css = read('../styles/app.css');
  assert.match(src, /class="model-dropdown-footer"/);
  assert.match(css, /\.model-dropdown-footer\s*\{/);
  // The footer must sit AFTER the model list, so it cannot be read as a model.
  const listIdx = src.indexOf('model-item');
  const footerIdx = src.indexOf('model-dropdown-footer');
  assert.ok(listIdx !== -1 && footerIdx > listIdx, 'footer renders below the entries');
});

test('settings dialog closing bumps the refresh signal all the way to the picker', () => {
  assert.match(app(), /modelsVersion=\{modelsVersion\}/);
  assert.match(app(), /ModelConfigDialog onClose=\{[\s\S]*?setModelsVersion\(\(v\) => v \+ 1\)/);
  assert.match(chat(), /refreshSignal=\{modelRefreshSignal\}/);
  assert.match(chat(), /setModelRefreshSignal\(\(n\) => n \+ 1\)/);
});

test('the signal bump is edge-triggered, so mount does not double-fetch', () => {
  const src = chat();
  assert.match(src, /if \(modelsVersion === modelsVersionRef\.current\) return;/);
  assert.match(src, /modelsVersionRef\.current = modelsVersion;/);
});

test('i18n carries the refresh labels in both zh and en', () => {
  const i18n = read('../i18n.ts');
  for (const key of ['model.refresh', 'model.refreshing']) {
    const pattern = new RegExp(`'${key.replace('.', '\\.')}':`, 'g');
    assert.equal((i18n.match(pattern) ?? []).length, 2, `${key} missing from a table`);
  }
});

test('the css adds the footer without touching the shared dropdown shell', () => {
  const css = read('../styles/app.css');
  assert.match(css, /\.model-refresh-action\s*\{/);
  assert.match(css, /\.model-refreshing\s*\{/);
});
