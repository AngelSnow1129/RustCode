import { test } from 'node:test';
import assert from 'node:assert';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

const root = process.cwd();

test('im channels dialog keeps its layout contract (wide modal + flex head)', () => {
  const tsx = readFileSync(join(root, 'src/components/SettingsDialogs.tsx'), 'utf8');
  const css = readFileSync(join(root, 'src/styles/app.css'), 'utf8');

  // The channels dialog must opt into the wide modal (380px card-sm cannot fit it).
  assert.match(tsx, /cardClass="im-channels-modal"/);
  assert.match(css, /\.modal-card\.im-channels-modal\s*\{[^}]*width:\s*min\(760px,\s*100%\);/s);

  // The row head must be a flex container — without display:flex the five
  // children (position, select, toggle, test, delete) stack vertically.
  assert.match(css, /\.im-channel-head\s*\{[^}]*display:\s*flex;/s);

  // Every class the channels dialog emits must have a CSS definition.
  for (const cls of [
    'im-channel',
    'im-channel-head',
    'im-channel-pos',
    'im-channel-enabled',
    'im-channel-field',
    'im-saved',
    'im-channel-missing',
  ]) {
    assert.match(css, new RegExp(`\\.${cls.replace(/-/g, '\\-')}\\s*\\{`), `missing .${cls} in app.css`);
  }

  // Text inputs inside channel fields must use the shared menu-input chrome
  // (Tailwind Preflight strips border/padding from bare inputs).
  assert.doesNotMatch(tsx, /class="im-channel-field"[^>]*>\s*<span>[^<]*<\/span>\s*<input\s+type="text"\s+placeholder/s);
});
