import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { assistantTurnActions } from './turnActions.ts';

const read = (relative: string) =>
  readFileSync(fileURLToPath(new URL(relative, import.meta.url)), 'utf8');

/** Index of every message whose assistant turn ENDS at that row — mirrors
 *  `assistantTurnEndFlags`, which the caller feeds in alongside the messages. */
function endsFor(roles: string[]): boolean[] {
  return roles.map((role, i) => role === 'assistant' && roles[i + 1] !== 'assistant');
}

test('each assistant turn maps to the user prompt that opened it, numbered 1-based', () => {
  const messages = [
    { role: 'user', parts: [{ kind: 'text', text: 'first' }] },
    { role: 'assistant', parts: [{ kind: 'text', text: 'a1' }] },
    { role: 'user', parts: [{ kind: 'text', text: 'second' }] },
    { role: 'assistant', parts: [{ kind: 'text', text: 'a2' }] },
  ];
  const map = assistantTurnActions(messages, endsFor(messages.map((m) => m.role)));
  assert.equal(map.get(1)?.promptN, 1);
  assert.equal(map.get(1)?.promptText, 'first');
  assert.equal(map.get(3)?.promptN, 2);
  assert.equal(map.get(3)?.promptText, 'second');
});

test('only the newest turn is regenerable in place', () => {
  const messages = [
    { role: 'user', parts: [{ kind: 'text', text: 'first' }] },
    { role: 'assistant', parts: [{ kind: 'text', text: 'a1' }] },
    { role: 'user', parts: [{ kind: 'text', text: 'second' }] },
    { role: 'assistant', parts: [{ kind: 'text', text: 'a2' }] },
  ];
  const map = assistantTurnActions(messages, endsFor(messages.map((m) => m.role)));
  assert.equal(map.get(1)?.isLastTurn, false);
  assert.equal(map.get(3)?.isLastTurn, true);
});

test('intermediate tool rounds inside one turn get no action row', () => {
  // assistant -> assistant: only the LAST assistant row of the turn is a
  // turn-end, so the intermediate tool round must not grow its own toolbar.
  const messages = [
    { role: 'user', parts: [{ kind: 'text', text: 'q' }] },
    { role: 'assistant', parts: [{ kind: 'tool', text: '' }] },
    { role: 'assistant', parts: [{ kind: 'text', text: 'done' }] },
  ];
  const map = assistantTurnActions(messages, endsFor(messages.map((m) => m.role)));
  assert.equal(map.has(1), false);
  assert.equal(map.get(2)?.isLastTurn, true);
});

test('system notices never open a turn, so they get no actions', () => {
  const messages = [
    { role: 'system', parts: [{ kind: 'text', text: 'notice' }] },
    { role: 'user', parts: [{ kind: 'text', text: 'q' }] },
    { role: 'assistant', parts: [{ kind: 'text', text: 'a' }] },
  ];
  const map = assistantTurnActions(messages, endsFor(messages.map((m) => m.role)));
  assert.equal(map.has(0), false);
  // The first real prompt is still N=1 despite the leading system row.
  assert.equal(map.get(2)?.promptN, 1);
});

test('prompt images are carried so a regenerate resends them too', () => {
  const image = { data: 'base64', mime: 'image/png' } as never;
  const messages = [
    { role: 'user', parts: [{ kind: 'text', text: 'look' }], images: [image] },
    { role: 'assistant', parts: [{ kind: 'text', text: 'ok' }] },
  ];
  const map = assistantTurnActions(messages, endsFor(messages.map((m) => m.role)));
  assert.deepEqual(map.get(1)?.promptImages, [image]);
});

test('the toolbar renders restore/regenerate next to the copy button', () => {
  // Guards the wiring the pure helpers cannot see: AssistantMessageView must
  // actually place the two actions in the assistant action row.
  const chat = read('../components/Chat.tsx');
  assert.match(chat, /msg-actions msg-actions-left[\s\S]*?\{copyBtn\}[\s\S]*?\{restoreBtn\}[\s\S]*?\{regenerateBtn\}/);
  assert.match(chat, /onRestore=\{\(target\) => onRestore\?\.\(target\)\}|onClick=\{\(\) => onRestore\?\.\(turnAction\)\}/);
  assert.match(chat, /onClick=\{\(\) => onRegenerate\?\.\(turnAction\)\}/);
});

test('restore and regenerate are gated on idle and non-error turns', () => {
  const chat = read('../components/Chat.tsx');
  assert.match(chat, /turnAction && !streaming && !isError/);
  assert.match(chat, /turnAction\?\.isLastTurn && !streaming && !isError/);
});

test('destructive restore on a non-last turn still asks for confirmation', () => {
  const chat = read('../components/Chat.tsx');
  assert.match(chat, /if \(!target\.isLastTurn && !window\.confirm\(t\('msg\.restoreConfirm'\)\)\) return;/);
});

test('restore/regenerate refuse to run while busy or in sync mode', () => {
  const chat = read('../components/Chat.tsx');
  assert.match(chat, /if \(busyRef\.current\) \{ pushCommandNotice\(t\('cmd\.session\.busy'\)\); return false; \}/);
  assert.match(chat, /if \(sync\) \{ pushCommandNotice\(t\('cmd\.session\.syncUnsupported'\)\); return false; \}/);
});

test('i18n carries both restore/regenerate labels in zh and en', () => {
  const i18n = read('../i18n.ts');
  for (const key of ['msg.restore', 'msg.regenerate', 'msg.restoreConfirm']) {
    const zh = new RegExp(`'${key.replace('.', '\\.')}':`);
    assert.match(i18n, zh);
    assert.equal((i18n.match(new RegExp(`'${key.replace('.', '\\.')}':`, 'g')) ?? []).length, 2);
  }
});
