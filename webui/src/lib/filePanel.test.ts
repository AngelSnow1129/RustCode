// File-panel model tests: normalization, containment, breadcrumbs, dir rows,
// artifact pairing. Pure functions only — no fetch, no React, no DOM.
import { test } from 'node:test';
import assert from 'node:assert';
import {
  fsArtifactRows,
  fsBreadcrumbs,
  fsEntryRows,
  fsIsWithinWorkspace,
  fsJoinPath,
  fsNormalizePath,
  fsParentPath,
  fsRelativeToRoot,
} from './filePanel.ts';

test('normalize collapses separators, dots and trailing slashes', () => {
  assert.equal(fsNormalizePath('/a/b/'), '/a/b');
  assert.equal(fsNormalizePath('/a//b///c'), '/a/b/c');
  assert.equal(fsNormalizePath('/a/./b'), '/a/b');
  assert.equal(fsNormalizePath('/a/b/../c'), '/a/c');
  assert.equal(fsNormalizePath('a\\b\\c'), 'a/b/c');
  assert.equal(fsNormalizePath('  '), '');
  assert.equal(fsNormalizePath(''), '');
  assert.equal(fsNormalizePath('/'), '/');
});

test('normalize keeps a windows drive prefix', () => {
  assert.equal(fsNormalizePath('C:\\Users\\x'), 'C:/Users/x');
  // `x/..` cancels, leaving the USER dir -- not the drive root.
  assert.equal(fsNormalizePath('C:/Users/x/../y'), 'C:/Users/y');
  assert.equal(fsNormalizePath('c:/'), 'c:/');
});

test('normalize does not climb above a relative root', () => {
  // Climbing past the start of a relative path stays lexical.
  assert.equal(fsNormalizePath('../../etc'), '../../etc');
  assert.equal(fsNormalizePath('a/../../b'), '../b');
});

test('containment accepts the root itself and descendants', () => {
  assert.equal(fsIsWithinWorkspace('/w/proj', '/w/proj'), true);
  assert.equal(fsIsWithinWorkspace('/w/proj', '/w/proj/src/main.rs'), true);
  assert.equal(fsIsWithinWorkspace('/w/proj/', '/w/proj/src'), true);
  // A sibling sharing a textual prefix is NOT inside.
  assert.equal(fsIsWithinWorkspace('/w/proj', '/w/proj-other/x'), false);
});

test('containment rejects climbing out and unrelated paths', () => {
  assert.equal(fsIsWithinWorkspace('/w/proj', '/etc/passwd'), false);
  assert.equal(fsIsWithinWorkspace('/w/proj', '/w/other'), false);
  // `..` is resolved before the comparison, so a climb is rejected.
  assert.equal(fsIsWithinWorkspace('/w/proj', '/w/proj/../secret'), false);
  // Relative paths resolve against the root by construction.
  assert.equal(fsIsWithinWorkspace('/w/proj', 'src/main.rs'), true);
  assert.equal(fsIsWithinWorkspace('/w/proj', '../outside'), false);
});

test('containment is empty-safe', () => {
  assert.equal(fsIsWithinWorkspace('', '/a'), false);
  assert.equal(fsIsWithinWorkspace('/a', ''), false);
});

test('relative-to-root returns the suffix or null', () => {
  assert.equal(fsRelativeToRoot('/w/proj', '/w/proj/src/main.rs'), 'src/main.rs');
  assert.equal(fsRelativeToRoot('/w/proj', '/w/proj'), '');
  assert.equal(fsRelativeToRoot('/w/proj', '/etc/passwd'), null);
  assert.equal(fsRelativeToRoot('/w/proj', 'src/main.rs'), 'src/main.rs');
  assert.equal(fsRelativeToRoot('', '/a'), null);
});

test('join handles roots, empty sides and trailing slashes', () => {
  assert.equal(fsJoinPath('/a/b', 'c'), '/a/b/c');
  assert.equal(fsJoinPath('/a/b/', 'c'), '/a/b/c');
  assert.equal(fsJoinPath('/', 'c'), '/c');
  assert.equal(fsJoinPath('', 'c'), 'c');
  assert.equal(fsJoinPath('/a/b', ''), '/a/b');
  assert.equal(fsJoinPath('', ''), '');
});

test('parent walks up and stops at the root', () => {
  assert.equal(fsParentPath('/a/b/c'), '/a/b');
  assert.equal(fsParentPath('/a/b'), '/a');
  assert.equal(fsParentPath('/a'), '/');
  assert.equal(fsParentPath('/'), null);
  assert.equal(fsParentPath(''), null);
  assert.equal(fsParentPath('relative'), null);
});

test('breadcrumbs anchor at the filesystem root', () => {
  assert.deepEqual(fsBreadcrumbs('/w/proj/src'), [
    { label: '/', path: '/' },
    { label: 'w', path: '/w' },
    { label: 'proj', path: '/w/proj' },
    { label: 'src', path: '/w/proj/src' },
  ]);
  assert.deepEqual(fsBreadcrumbs('/'), [{ label: '/', path: '/' }]);
  assert.deepEqual(fsBreadcrumbs(''), []);
});

test('breadcrumbs keep a windows drive as the anchor', () => {
  assert.deepEqual(fsBreadcrumbs('C:/Users/x'), [
    { label: 'C:', path: 'C:/' },
    { label: 'Users', path: 'C:/Users' },
    { label: 'x', path: 'C:/Users/x' },
  ]);
});

test('breadcrumbs for a relative path have no root anchor', () => {
  assert.deepEqual(fsBreadcrumbs('a/b'), [
    { label: 'a', path: 'a' },
    { label: 'b', path: 'a/b' },
  ]);
});

test('entry rows put directories first then sort each group', () => {
  const rows = fsEntryRows('/w', ['zeta', 'alpha'], ['readme.md', 'Beta.rs']);
  assert.deepEqual(
    rows.map((r) => [r.name, r.isDir]),
    [
      ['alpha', true],
      ['zeta', true],
      ['Beta.rs', false],
      ['readme.md', false],
    ],
  );
  assert.equal(rows[0].path, '/w/alpha');
  assert.equal(rows[2].path, '/w/Beta.rs');
});

test('entry rows tolerate missing file lists', () => {
  assert.deepEqual(fsEntryRows('/w', ['d'], []).map((r) => r.name), ['d']);
  assert.deepEqual(fsEntryRows('/w', [], []), []);
});

test('artifact rows mark in-workspace paths previewable', () => {
  const rows = fsArtifactRows('/w/proj', [
    { path: '/w/proj/src/main.rs', label: 'main.rs' },
    { path: '/w/outside/other.rs', label: 'other.rs' },
  ]);
  assert.equal(rows.length, 2);
  assert.equal(rows[0].previewable, true);
  assert.equal(rows[0].requestPath, 'src/main.rs');
  // Out-of-workspace artifacts are still listed (the agent may have written
  // them) but are not offered as preview links.
  assert.equal(rows[1].previewable, false);
  assert.equal(rows[1].requestPath, '/w/outside/other.rs');
});

test('artifact rows dedupe and fall back to the basename', () => {
  const rows = fsArtifactRows('/w', [
    { path: '/w/a/x.rs', label: '' },
    { path: '/w/a/x.rs', label: 'dup' },
    { path: '', label: 'empty' },
  ]);
  assert.equal(rows.length, 1, 'duplicates and empty paths collapse');
  assert.equal(rows[0].label, 'x.rs', 'empty label falls back to the basename');
  // A relative artifact resolves against the working directory.
  const rel = fsArtifactRows('/w/proj', [{ path: 'src/lib.rs', label: 'lib.rs' }]);
  assert.equal(rel[0].previewable, true);
  assert.equal(rel[0].requestPath, 'src/lib.rs');
});
