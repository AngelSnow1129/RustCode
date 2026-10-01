// IM records browser model (platform -> project -> session_id).
//
// Pure shapes + selectors only: no React, no fetch. The sidebar popover walks
// the three levels with these, and the tests exercise them directly.
//
// Read-only by design — there is no create/delete action anywhere in this tree.

import type { ImBindingInfo, ImBindingsInfo, ImPlatformNode, ImProjectNode } from '../api';

/** Which level of the 3-level breadcrumb the popover is showing.
 *  0 = platform list, 1 = project list (inside one platform), 2 = session list. */
export type ImRecordLevel = 0 | 1 | 2;

/** The current (platform, project) selection; `null` means "not drilled in". */
export interface ImRecordSelection {
  platform: string | null;
  project: string | null;
}

export const IM_ROOT_SELECTION: ImRecordSelection = { platform: null, project: null };

/** One row of the level-0 list: every platform the daemon knows about, INCLUDING
 *  configured-but-unused ones (binding_count 0) — those must still be listed so
 *  the user can see a channel that is wired up but has no conversation yet. */
export interface ImPlatformRow {
  platform: string;
  binding_count: number;
  project_count: number;
  known_platform: boolean;
  configured: boolean;
  channel_enabled: boolean;
  last_active_at: number;
}

/** One row of the level-1 list (projects inside the selected platform). */
export interface ImProjectRow {
  project: string;
  binding_count: number;
  session_count: number;
  last_active_at: number;
}

/** One row of the level-2 list: a bound session, i.e. a clickable leaf. */
export interface ImSessionRow {
  session_id: string;
  chat_id: string;
  project: string;
  /** Binding creation time (the "bound since" meta line). */
  created_at: number;
  /** Last activity in that IM conversation (the "last active" meta line). */
  updated_at: number;
}

/** Normalizes a possibly-null timestamp to a number (0 when absent). */
function ts(value: number | null | undefined): number {
  return typeof value === 'number' && Number.isFinite(value) ? value : 0;
}

/** Platforms shown at level 0. Sorted by most-recently-active, then by name so
 *  the order is stable when timestamps tie (or are all zero). */
export function imPlatformRows(tree: ImBindingsInfo | null | undefined): ImPlatformRow[] {
  const platforms = tree?.platforms ?? [];
  return platforms
    .map((p) => ({
      platform: p.platform,
      binding_count: p.binding_count ?? 0,
      project_count: p.project_count ?? p.projects?.length ?? 0,
      known_platform: !!p.known_platform,
      configured: !!p.configured,
      channel_enabled: !!p.channel_enabled,
      last_active_at: ts(p.last_active_at),
    }))
    .sort((a, b) => b.last_active_at - a.last_active_at || a.platform.localeCompare(b.platform));
}

/** Roll-up status of one platform row for the records popover: a platform with
 *  no channel configuration cannot receive messages even when bindings from an
 *  older config remain on disk. */
export type ImPlatformStatus = 'enabled' | 'disabled' | 'unconfigured';

export function imPlatformStatus(
  row: Pick<ImPlatformRow, 'configured' | 'channel_enabled'>,
): ImPlatformStatus {
  if (!row.configured) return 'unconfigured';
  return row.channel_enabled ? 'enabled' : 'disabled';
}

/** Projects of the selected platform (level 1). Empty when nothing is selected. */
export function imProjectRows(
  tree: ImBindingsInfo | null | undefined,
  selection: ImRecordSelection,
): ImProjectRow[] {
  const platform = selection.platform ? imFindPlatform(tree, selection.platform) : null;
  if (!platform) return [];
  const projects = platform.projects ?? [];
  return projects
    .map((pr) => ({
      project: pr.project,
      binding_count: pr.binding_count ?? pr.bindings?.length ?? 0,
      session_count: pr.session_count ?? pr.bindings?.length ?? 0,
      last_active_at: ts(pr.last_active_at) || latestBindingTs(pr.bindings),
    }))
    .sort((a, b) => b.last_active_at - a.last_active_at || a.project.localeCompare(b.project));
}

/** Bound sessions of the selected platform + project (level 2), newest first. */
export function imSessionRows(
  tree: ImBindingsInfo | null | undefined,
  selection: ImRecordSelection,
): ImSessionRow[] {
  const project = selection.project ? imFindProject(tree, selection.platform, selection.project) : null;
  if (!project) return [];
  const bindings = project.bindings ?? [];
  return bindings
    .map((b) => ({
      session_id: b.session_id,
      chat_id: b.chat_id,
      project: b.project || project.project,
      created_at: ts(b.created_at),
      updated_at: ts(b.updated_at) || ts(b.created_at),
    }))
    .sort((a, b) => b.updated_at - a.updated_at || a.session_id.localeCompare(b.session_id));
}

/** Finds a platform node by exact name (null when absent). */
export function imFindPlatform(
  tree: ImBindingsInfo | null | undefined,
  platform: string | null | undefined,
): ImPlatformNode | null {
  if (!platform) return null;
  return (tree?.platforms ?? []).find((p) => p.platform === platform) ?? null;
}

/** Finds a project node inside a platform (null when either level is missing). */
export function imFindProject(
  tree: ImBindingsInfo | null | undefined,
  platform: string | null | undefined,
  project: string | null | undefined,
): ImProjectNode | null {
  if (!project) return null;
  const node = imFindPlatform(tree, platform);
  if (!node) return null;
  return (node.projects ?? []).find((p) => p.project === project) ?? null;
}

/** Looks up one session leaf by its (possibly short) session id, anywhere in the
 *  currently selected platform+project. Returns null when there is no match. */
export function imFindSession(
  tree: ImBindingsInfo | null | undefined,
  selection: ImRecordSelection,
  sessionId: string | null | undefined,
): ImSessionRow | null {
  if (!sessionId) return null;
  return (
    imSessionRows(tree, selection).find(
      (row) => row.session_id === sessionId || row.session_id.startsWith(sessionId),
    ) ?? null
  );
}

/** Total binding count across the tree — the sidebar badge. Prefers the backend
 *  `total`, falling back to summing the platform counts when it is missing. */
export function imTotalBindings(tree: ImBindingsInfo | null | undefined): number {
  if (!tree) return 0;
  if (typeof tree.total === 'number' && Number.isFinite(tree.total)) return tree.total;
  return (tree.platforms ?? []).reduce((sum, p) => sum + (p.binding_count ?? 0), 0);
}

/** Flattens the tree down to the raw bindings reachable under a selection:
 *  no selection -> every binding of every platform (a platform with zero
 *  bindings contributes nothing); platform -> its bindings; platform+project
 *  -> that project's bindings. Order follows the tree, so callers that need a
 *  stable sort should use `imSessionRows` instead. */
export function imFlattenBindings(
  tree: ImBindingsInfo | null | undefined,
  selection: ImRecordSelection = IM_ROOT_SELECTION,
): ImBindingInfo[] {
  const platforms = tree?.platforms ?? [];
  const scoped = selection.platform
    ? platforms.filter((p) => p.platform === selection.platform)
    : platforms;
  const out: ImBindingInfo[] = [];
  for (const platform of scoped) {
    const projects = selection.project
      ? (platform.projects ?? []).filter((p) => p.project === selection.project)
      : platform.projects ?? [];
    for (const project of projects) {
      for (const binding of project.bindings ?? []) {
        if (binding) out.push(binding);
      }
    }
  }
  return out;
}

/** Whether the tree has anything to browse at all (any platform node). */
export function imHasRecords(tree: ImBindingsInfo | null | undefined): boolean {
  return (tree?.platforms ?? []).length > 0;
}

export interface ImCrumb {
  label: string;
  /** Which level clicking this crumb jumps back to. */
  level: ImRecordLevel;
  /** The selection to apply when the crumb is clicked. */
  selection: ImRecordSelection;
}

/**
 * Breadcrumb segments for the current drill-down:
 *   level 0 -> [root]
 *   level 1 -> [root, platform]
 *   level 2 -> [root, platform, project]
 * The last segment is the current level (rendered non-clickable by the UI).
 */
export function imBreadcrumbs(
  selection: ImRecordSelection,
  labels?: { root: string },
): ImCrumb[] {
  const root: ImCrumb = {
    label: labels?.root ?? '',
    level: 0,
    selection: { platform: null, project: null },
  };
  const crumbs: ImCrumb[] = [root];
  if (selection.platform) {
    crumbs.push({
      label: selection.platform,
      level: 1,
      selection: { platform: selection.platform, project: null },
    });
  }
  // A project without a platform can't be reached, so ignore it — the caller
  // always drills platform-first.
  if (selection.platform && selection.project) {
    crumbs.push({
      label: selection.project,
      level: 2,
      selection: { platform: selection.platform, project: selection.project },
    });
  }
  return crumbs;
}

function latestBindingTs(bindings: ImBindingInfo[] | undefined): number {
  let latest = 0;
  for (const b of bindings ?? []) {
    const v = ts(b?.updated_at) || ts(b?.created_at);
    if (v > latest) latest = v;
  }
  return latest;
}
