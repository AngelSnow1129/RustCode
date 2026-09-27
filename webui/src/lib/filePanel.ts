// File-panel model: path normalization, workspace containment, breadcrumbs and
// directory-row shaping.
//
// Pure helpers only — no React, no fetch, no DOM. The panel renders whatever
// these return, and the tests exercise them directly.
//
// IMPORTANT: `fsIsWithinWorkspace` is a UX judgement, NOT the security boundary.
// The authoritative check is server-side (`/fs/read` canonicalizes, resolves
// symlinks, confines to the session working directory, rejects sensitive paths,
// and refuses entirely when the webui runs without auth). A browser cannot
// resolve symlinks at all, so this lexical test can disagree with the server in
// both directions — which is acceptable precisely because the server refuses.
// Never treat a truthy result here as permission to skip the server's answer.

/** Normalize a path for comparison/display: `/` separators, no trailing slash,
 *  and `.`/`..` segments resolved lexically. Empty input yields `''`. */
export function fsNormalizePath(path: string): string {
  const raw = (path ?? '').replace(/\\/g, '/').trim();
  if (!raw) return '';
  const isAbsolute = raw.startsWith('/');
  // A Windows drive prefix (`C:/...`) must survive normalization.
  const drive = /^([A-Za-z]:)(\/|$)/.exec(raw);
  const afterDrive = drive ? raw.slice(drive[1].length) : raw;
  const out: string[] = [];
  for (const segment of afterDrive.split('/')) {
    if (!segment || segment === '.') continue;
    if (segment === '..') {
      // Climbing past the root is a no-op rather than an error: the result is
      // only used for a prefix comparison.
      if (out.length > 0 && out[out.length - 1] !== '..') out.pop();
      else if (!isAbsolute && !drive) out.push('..');
      continue;
    }
    out.push(segment);
  }
  const body = out.join('/');
  if (drive) return body ? `${drive[1]}/${body}` : `${drive[1]}/`;
  return isAbsolute ? `/${body}` : body;
}

/** True when `child` is `root` itself or lives underneath it (lexically). */
export function fsIsWithinWorkspace(root: string, child: string): boolean {
  const normRoot = fsNormalizePath(root);
  const normChild = fsNormalizePath(child);
  if (!normRoot || !normChild) return false;
  if (normChild === normRoot) return true;
  // Relative child paths resolve against the root, so they are inside by
  // construction (the server still validates `..` climbs).
  if (!normChild.startsWith('/')) return !normChild.startsWith('../');
  const withSlash = normRoot.endsWith('/') ? normRoot : normRoot + '/';
  if (normChild.startsWith(withSlash)) return true;
  // Secondary case-insensitive check: Windows/macOS filesystems fold case, so a
  // differing spelling still refers to the same file. Erring toward "inside"
  // only ever produces a link the server may still refuse (it re-checks), which
  // is the fail-safe direction; hiding a valid link is the worse failure.
  return normChild.toLowerCase().startsWith(withSlash.toLowerCase());
}

/**
 * `child` expressed relative to `root`, or `null` when it is outside.
 * Returns `''` when `child` IS the root.
 */
export function fsRelativeToRoot(root: string, child: string): string | null {
  const normRoot = fsNormalizePath(root);
  const normChild = fsNormalizePath(child);
  if (!normRoot || !normChild) return null;
  if (!normChild.startsWith('/')) return normChild === '..' ? null : normChild;
  if (normChild === normRoot) return '';
  const withSlash = normRoot.endsWith('/') ? normRoot : normRoot + '/';
  if (normChild.startsWith(withSlash)) return normChild.slice(withSlash.length);
  const lowerRoot = withSlash.toLowerCase();
  if (normChild.toLowerCase().startsWith(lowerRoot)) return normChild.slice(withSlash.length);
  return null;
}

/** Join a directory and a child name for the `/fs/list` request. */
export function fsJoinPath(dir: string, name: string): string {
  const base = fsNormalizePath(dir);
  const leaf = fsNormalizePath(name);
  if (!base) return leaf;
  if (!leaf) return base;
  return base.endsWith('/') ? base + leaf : `${base}/${leaf}`;
}

/** Parent directory, or `null` at/above the filesystem root. */
export function fsParentPath(path: string): string | null {
  const norm = fsNormalizePath(path);
  if (!norm || norm === '/') return null;
  const idx = norm.lastIndexOf('/');
  if (idx < 0) return null;
  return idx === 0 ? '/' : norm.slice(0, idx);
}

/** One breadcrumb segment with the full path it navigates to. */
export interface FsCrumb {
  label: string;
  path: string;
}

/**
 * Breadcrumbs from the filesystem root down to `path`. The root segment is
 * labelled `/` (or the drive, e.g. `C:`) so a deep path never loses its anchor.
 */
export function fsBreadcrumbs(path: string): FsCrumb[] {
  const norm = fsNormalizePath(path);
  if (!norm) return [];
  const driveMatch = /^([A-Za-z]:)(\/|$)/.exec(norm);
  // A drive prefix is an anchor in its own right -- `norm` does NOT start with
  // `/` here, so testing for that would skip this branch and fall through to the
  // relative path, yielding a crumb path of bare `C:` (a drive-RELATIVE path,
  // which is not what clicking the anchor should request).
  if (driveMatch) {
    const rest = norm.slice(driveMatch[1].length + 1);
    const crumbs: FsCrumb[] = [{ label: driveMatch[1], path: driveMatch[1] + '/' }];
    let acc = driveMatch[1];
    for (const segment of rest.split('/').filter(Boolean)) {
      acc += '/' + segment;
      crumbs.push({ label: segment, path: acc });
    }
    return crumbs;
  }
  if (!norm.startsWith('/')) {
    // Relative path: no root anchor to show, just the segments.
    const crumbs: FsCrumb[] = [];
    let acc = '';
    for (const segment of norm.split('/').filter(Boolean)) {
      acc = acc ? `${acc}/${segment}` : segment;
      crumbs.push({ label: segment, path: acc });
    }
    return crumbs;
  }
  const crumbs: FsCrumb[] = [{ label: '/', path: '/' }];
  let acc = '';
  for (const segment of norm.slice(1).split('/').filter(Boolean)) {
    acc += '/' + segment;
    crumbs.push({ label: segment, path: acc });
  }
  return crumbs;
}

/** One row in the directory browser. */
export interface FsEntryRow {
  name: string;
  path: string;
  isDir: boolean;
}

/**
 * Shape a `/fs/list` response into display rows: directories first, then files,
 * each alphabetically (case-insensitive so `README` and `readme` order stably).
 * The server returns names only, in two separate arrays.
 */
export function fsEntryRows(dir: string, dirs: string[], files: string[]): FsEntryRow[] {
  const byName = (a: FsEntryRow, b: FsEntryRow) =>
    a.name.toLowerCase().localeCompare(b.name.toLowerCase()) || a.name.localeCompare(b.name);
  const dirRows: FsEntryRow[] = (dirs ?? []).map((name) => ({
    name,
    path: fsJoinPath(dir, name),
    isDir: true,
  }));
  const fileRows: FsEntryRow[] = (files ?? []).map((name) => ({
    name,
    path: fsJoinPath(dir, name),
    isDir: false,
  }));
  return [...dirRows.sort(byName), ...fileRows.sort(byName)];
}

/** A turn artifact paired with whether the panel can offer a preview link. */
export interface FsArtifactRow {
  path: string;
  label: string;
  /** False when the path is lexically outside the working directory. */
  previewable: boolean;
  /** Path to request from `/fs/read` (relative form when possible). */
  requestPath: string;
}

/**
 * Pair the turn's file mutations with the working directory so the panel knows
 * which ones it can preview. Artifacts may arrive absolute (the agent writes
 * absolute paths) or relative; both are handled.
 */
export function fsArtifactRows(
  cwd: string,
  artifacts: { path: string; label: string }[],
): FsArtifactRow[] {
  const seen = new Set<string>();
  const rows: FsArtifactRow[] = [];
  for (const artifact of artifacts ?? []) {
    const norm = fsNormalizePath(artifact.path);
    if (!norm || seen.has(norm)) continue;
    seen.add(norm);
    const rel = fsRelativeToRoot(cwd, norm);
    const previewable = rel !== null && rel !== '';
    rows.push({
      path: norm,
      label: artifact.label || norm.split('/').pop() || norm,
      previewable,
      requestPath: rel ?? norm,
    });
  }
  return rows;
}
