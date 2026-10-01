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

test('im UI additions keep their CSS contracts (grouped credentials, status pills, back affordance)', () => {
  const tsx = readFileSync(join(root, 'src/components/SettingsDialogs.tsx'), 'utf8');
  const sidebar = readFileSync(join(root, 'src/components/Sidebar.tsx'), 'utf8');
  const css = readFileSync(join(root, 'src/styles/app.css'), 'utf8');

  // Credential fields render inside one dashed sub-card; every class the
  // section emits must exist in app.css.
  assert.match(tsx, /class="im-cred-section"/);
  for (const cls of ['im-cred-section', 'im-cred-title', 'im-test-mark', 'im-test-host']) {
    assert.match(css, new RegExp(`\\.${cls.replace(/-/g, '\\-')}\\s*\\{`), `missing .${cls} in app.css`);
  }

  // Records popover: back button (32px hit target + visible focus ring),
  // channel summary line, and status pills for all three health states.
  assert.match(sidebar, /class="im-back"/);
  assert.match(sidebar, /im-menu-summary/);
  for (const cls of ['im-back', 'im-menu-summary', 'im-status-enabled', 'im-status-disabled', 'im-status-unconfigured']) {
    assert.match(css, new RegExp(`\\.${cls.replace(/-/g, '\\-')}\\s*\\{`), `missing .${cls} in app.css`);
  }
  assert.match(css, /\.im-back\s*\{[^}]*width:\s*32px;/s);
  assert.match(css, /\.im-back:focus-visible,\s*\n?\.im-crumb:focus-visible\s*\{\s*\n?\s*outline:/s);

  // Unread dot: rendered from the imUnseen flag in both expanded and rail IM
  // entries, styled with the brand accent.
  assert.match(sidebar, /im-unseen-dot/);
  assert.match(css, /\.im-unseen-dot\s*\{[^}]*var\(--app-brand\)/s);
});
