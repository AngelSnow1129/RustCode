// Right-hand file panel: browse a directory, see this turn's changed files, and
// preview text files inline.
//
// Scope boundaries (deliberate, see docs/plans/2026-09-25-file-surface-security.md):
//   * Preview goes through `GET /fs/read`, which the daemon confines to the
//     SESSION WORKING DIRECTORY, rejects sensitive paths, caps size and refuses
//     binary content -- and refuses outright when the webui runs without auth.
//     The panel therefore shows the server's message verbatim instead of trying
//     to re-implement the boundary in the browser (a browser cannot resolve
//     symlinks, so it could not enforce it correctly anyway).
//   * Browsing uses `/fs/list`, which is NOT confined (it backs the project
//     picker). Files outside the working directory are listed but are shown as
//     not previewable rather than being hidden -- the agent may legitimately have
//     written them, and hiding them would look like a bug.
//   * Read-only: no create, rename, delete, upload or download here. Downloads
//     are explicitly deferred by user decision.

import { useCallback, useEffect, useRef, useState } from 'preact/hooks';
import { listDir, readFile, type FsListResult } from '../api';
import { useT } from '../settings';
import {
  fsArtifactRows,
  fsBreadcrumbs,
  fsEntryRows,
  fsParentPath,
  type FsEntryRow,
} from '../lib/filePanel';

export interface FilePanelArtifact {
  path: string;
  label: string;
}

interface FilePanelProps {
  /** Session working directory: the containment root for previews. */
  cwd: string;
  /** Files the current turn changed (from `turnArtifacts`). */
  artifacts: FilePanelArtifact[];
  sessionId?: string;
  onClose: () => void;
  /** Open on the host with its default app (`/fs/open`); no content involved. */
  onOpenExternal?: (path: string) => void;
}

/** One request in flight at a time; a late response must not overwrite a newer
 *  selection (the same "latest wins" guard the model picker uses). */
type PreviewState =
  | { kind: 'idle' }
  | { kind: 'loading'; path: string }
  | { kind: 'ready'; path: string; content: string }
  | { kind: 'error'; path: string; message: string };

export function FilePanel({
  cwd,
  artifacts,
  sessionId,
  onClose,
  onOpenExternal,
}: FilePanelProps) {
  const t = useT();
  const [dir, setDir] = useState(cwd);
  const [listing, setListing] = useState<FsListResult | null>(null);
  const [dirError, setDirError] = useState<string | null>(null);
  const [dirLoading, setDirLoading] = useState(false);
  const [preview, setPreview] = useState<PreviewState>({ kind: 'idle' });
  const seqRef = useRef(0);

  // Follow the session's working directory when it changes (e.g. `/cd`), unless
  // the user has navigated somewhere else in the meantime.
  const followedCwdRef = useRef(cwd);
  useEffect(() => {
    if (followedCwdRef.current === cwd) return;
    followedCwdRef.current = cwd;
    setDir(cwd);
    setPreview({ kind: 'idle' });
  }, [cwd]);

  const loadDir = useCallback(
    (path: string) => {
      const seq = ++seqRef.current;
      setDirLoading(true);
      setDirError(null);
      listDir(path)
        .then((result) => {
          if (seq !== seqRef.current) return; // superseded
          setListing(result);
          // The server canonicalizes, so adopt its spelling: the next request
          // (and the breadcrumb) must agree with what the daemon resolved.
          if (result.path) setDir(result.path);
        })
        .catch((error: unknown) => {
          if (seq !== seqRef.current) return;
          setListing(null);
          setDirError(error instanceof Error ? error.message : String(error));
        })
        .finally(() => {
          if (seq === seqRef.current) setDirLoading(false);
        });
    },
    [],
  );

  useEffect(() => {
    loadDir(dir);
  }, [dir, loadDir]);

  const openPreview = useCallback(
    (path: string) => {
      const seq = ++seqRef.current;
      setPreview({ kind: 'loading', path });
      readFile(path, sessionId)
        .then((result) => {
          if (seq !== seqRef.current) return;
          setPreview({ kind: 'ready', path, content: result.content });
        })
        .catch((error: unknown) => {
          if (seq !== seqRef.current) return;
          // Server-side refusals (outside the working directory, sensitive,
          // binary, too large, or no-auth) surface here verbatim.
          setPreview({
            kind: 'error',
            path,
            message: error instanceof Error ? error.message : String(error),
          });
        });
    },
    [sessionId],
  );

  const crumbs = fsBreadcrumbs(dir);
  const rows: FsEntryRow[] = listing
    ? fsEntryRows(listing.path || dir, listing.dirs ?? [], listing.files ?? [])
    : [];
  const artifactRows = fsArtifactRows(cwd, artifacts);
  const parent = fsParentPath(dir);
  const currentPath =
    preview.kind === 'idle' ? null : preview.path;

  return (
    <aside class="file-panel" aria-label={t('filePanel.title')}>
      <header class="file-panel-header">
        <span class="file-panel-title">{t('filePanel.title')}</span>
        <button
          type="button"
          class="file-panel-icon-btn"
          title={t('filePanel.refresh')}
          aria-label={t('filePanel.refresh')}
          onClick={() => loadDir(dir)}
        >
          [R]
        </button>
        <button
          type="button"
          class="file-panel-icon-btn"
          title={t('filePanel.hide')}
          aria-label={t('filePanel.hide')}
          onClick={onClose}
        >
          [X]
        </button>
      </header>

      {/* Turn artifacts: the files this turn changed, previewable when inside
          the working directory. Shown first because "what did it just change?"
          is the primary question this panel answers. */}
      <section class="file-panel-section">
        <div class="file-panel-section-label">{t('filePanel.recent')}</div>
        {artifactRows.length === 0 ? (
          <div class="file-panel-note">{t('filePanel.noRecent')}</div>
        ) : (
          <ul class="file-panel-artifacts">
            {artifactRows.map((row) => (
              <li key={row.path}>
                <button
                  type="button"
                  class={
                    // Artifacts are previewed by their request path (relative when
                    // inside the workspace, absolute otherwise), while files
                    // clicked in the browser use their absolute path -- accept
                    // either so the two entry points agree on the highlight.
                    currentPath === row.requestPath || currentPath === row.path
                      ? 'file-panel-artifact active'
                      : 'file-panel-artifact'
                  }
                  title={row.previewable ? row.path : t('filePanel.outside')}
                  onClick={() => {
                    if (row.previewable) openPreview(row.requestPath);
                  }}
                  disabled={!row.previewable}
                >
                  <span class="file-panel-artifact-label">{row.label}</span>
                  {!row.previewable && (
                    <span class="file-panel-artifact-tag">{t('filePanel.outside')}</span>
                  )}
                </button>
              </li>
            ))}
          </ul>
        )}
      </section>

      {/* Directory browser, rooted at the working directory. */}
      <section class="file-panel-section file-panel-browse">
        <div class="file-panel-section-label">{t('filePanel.browse')}</div>
        <nav class="file-panel-crumbs" aria-label={t('filePanel.browse')}>
          {crumbs.map((crumb, index) => (
            <span key={crumb.path} class="file-panel-crumb">
              {index > 0 && <span class="file-panel-crumb-sep">/</span>}
              <button
                type="button"
                class="file-panel-crumb-btn"
                onClick={() => setDir(crumb.path)}
              >
                {crumb.label}
              </button>
            </span>
          ))}
        </nav>

        {dirError && <div class="file-panel-error">{dirError}</div>}
        {dirLoading && <div class="file-panel-note">{t('filePanel.loading')}</div>}

        <ul class="file-panel-entries">
          {parent && (
            <li>
              <button
                type="button"
                class="file-panel-entry dir"
                onClick={() => setDir(parent)}
              >
                <span class="file-panel-entry-icon">[D]</span>
                <span class="file-panel-entry-name">{t('filePanel.up')}</span>
              </button>
            </li>
          )}
          {!dirLoading && rows.length === 0 && !dirError && (
            <li class="file-panel-note">{t('filePanel.emptyDir')}</li>
          )}
          {rows.map((row) => (
            <li key={row.path}>
              <button
                type="button"
                class={row.isDir ? 'file-panel-entry dir' : 'file-panel-entry'}
                title={row.path}
                onClick={() => {
                  if (row.isDir) setDir(row.path);
                  else openPreview(row.path);
                }}
              >
                <span class="file-panel-entry-icon">{row.isDir ? '[D]' : '[F]'}</span>
                <span class="file-panel-entry-name">{row.name}</span>
              </button>
            </li>
          ))}
        </ul>
      </section>

      {/* Preview: text only, server-enforced. */}
      <section class="file-panel-preview">
        <div class="file-panel-preview-head">
          <span class="file-panel-section-label">{t('filePanel.preview')}</span>
          {preview.kind !== 'idle' && (
            <button
              type="button"
              class="file-panel-icon-btn"
              title={t('filePanel.closePreview')}
              aria-label={t('filePanel.closePreview')}
              onClick={() => setPreview({ kind: 'idle' })}
            >
              [X]
            </button>
          )}
          {onOpenExternal && preview.kind === 'ready' && (
            <button
              type="button"
              class="file-panel-icon-btn"
              title={t('filePanel.openExternal')}
              aria-label={t('filePanel.openExternal')}
              onClick={() => onOpenExternal(preview.path)}
            >
              [O]
            </button>
          )}
        </div>
        {preview.kind === 'idle' && (
          <div class="file-panel-note">{t('filePanel.previewHint')}</div>
        )}
        {preview.kind === 'loading' && (
          <div class="file-panel-note">{t('filePanel.loading')}</div>
        )}
        {preview.kind === 'error' && (
          <div class="file-panel-error">
            <div class="file-panel-preview-path">{preview.path}</div>
            {preview.message}
          </div>
        )}
        {preview.kind === 'ready' && (
          <>
            <div class="file-panel-preview-path" title={preview.path}>
              {preview.path}
            </div>
            <pre class="file-panel-code">
              <code>{preview.content}</code>
            </pre>
          </>
        )}
      </section>
    </aside>
  );
}
