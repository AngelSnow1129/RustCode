//! File-surface boundary predicates for the daemon `/fs/*` endpoints.
//!
//! This module is the SINGLE place that answers "may this webui request touch
//! this path?". It exists because the file surface grew endpoint by endpoint and
//! each site grew its own ad-hoc check (`resolve_workspace_file` only checked
//! `is_file`; `tools/mod.rs`, `open_file.rs`, `lsp_tool.rs` and `skills/render.rs`
//! each wrote their own `starts_with(root)`), which is exactly the drift pattern
//! this repo already guards against elsewhere (`fallback_eligible`, the
//! credential bash gate's shared bypass cell).
//!
//! Design and分期: `docs/plans/2026-09-25-file-surface-security.md`.
//!
//! Two rules that MUST survive any refactor here:
//!
//! 1. **Canonicalize first, then compare prefixes.** A symlink inside the
//!    workspace that points at `/etc/passwd` resolves to a path *outside* the
//!    root, so a prefix test done before canonicalization is trivially bypassed.
//! 2. **Content-exposing operations fail closed when auth is off.** With
//!    `webui_no_auth` (or any `enforce_token == false` deployment) the daemon is
//!    reachable by anyone who can route to the port, so `Read` / `Download` /
//!    `Package` must refuse rather than degrade.
//!
//! This module deliberately produces NO user-facing strings: L1 cannot depend on
//! `rustcode-config`, so every [`FileDeny`] is a stable, machine-shaped variant
//! that the daemon edge maps onto an i18n `Msg`. Callers must not invent their
//! own messages either.

use std::path::{Path, PathBuf};

use crate::pathnorm;
use crate::tools::looks_binary;
use crate::tools::sensitive_path::path_is_sensitive;

/// Default cap for a single file handed to the browser (preview or download).
/// Previews are meant for source files, not for dumping a build artifact through
/// the webui, so 1 MiB is generous and still bounded.
pub const DEFAULT_MAX_FILE_BYTES: u64 = 1024 * 1024;

/// Which `/fs/*` operation is being authorized. Drives two decisions: whether the
/// call exposes file CONTENT to the caller (the irreversible part), and whether
/// it mutates the filesystem.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileOp {
    /// `GET /fs/list`
    List,
    /// `GET /fs/search`
    Search,
    /// `POST /fs/mkdir`
    Mkdir,
    /// `POST /fs/open` (launches the host GUI opener; returns no content)
    Open,
    /// `GET /fs/read` (text preview)
    Read,
    /// `GET /fs/download`
    Download,
    /// Folder packaging (zip). Per-entry checks still apply on top of this.
    Package,
}

impl FileOp {
    /// True for operations that hand file CONTENT to the caller. These are the
    /// ones that must fail closed when authentication was turned off.
    pub fn exposes_content(self) -> bool {
        matches!(self, Self::Read | Self::Download | Self::Package)
    }

    /// True for operations that write to the filesystem.
    pub fn is_write(self) -> bool {
        matches!(self, Self::Mkdir)
    }
}

/// Per-request policy, resolved by the caller (the daemon edge) from server state.
#[derive(Clone, Copy, Debug)]
pub struct FilePolicy {
    /// `AppState.webui_no_auth` -- the user explicitly dropped authentication.
    pub no_auth: bool,
    /// Size cap for [`FileOp::Read`] / [`FileOp::Download`].
    pub max_bytes: u64,
}

impl Default for FilePolicy {
    fn default() -> Self {
        Self {
            no_auth: false,
            max_bytes: DEFAULT_MAX_FILE_BYTES,
        }
    }
}

/// Why access was refused. Machine-shaped on purpose (see the module docs); the
/// daemon maps each variant to a localized `Msg`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileDeny {
    /// Empty / whitespace-only path.
    EmptyPath,
    /// Content-exposing or write operation while auth is switched off.
    NoAuthMode,
    /// Missing, unresolvable, or outside the workspace root. Existence failures
    /// collapse into this variant on purpose: a webui caller must not be able to
    /// probe which paths exist outside the workspace.
    EscapesRoot,
    /// Resolved into a protected location or a secret file (see
    /// `sensitive_path::path_is_sensitive`).
    Sensitive,
    /// Larger than the caller's cap.
    TooLarge,
    /// Binary content -- previews are text-only.
    Binary,
    /// The operation needs a regular file but the path is not one.
    NotRegularFile,
    /// The operation needs a directory but the path is not one.
    NotADirectory,
    /// Could not be opened or read.
    Unreadable,
}

/// True iff `target` is `root` itself or lives underneath it.
///
/// Delegates to [`pathnorm::path_within_root`] -- the predicate lives there so
/// capability features that do not enable `tools` (`lsp`, `skills`) can share the
/// same implementation instead of writing another `starts_with(root)`.
/// Canonicalization happens FIRST there (a symlink inside the workspace that
/// points outside resolves out of the root, so a lexical prefix test is trivially
/// bypassed), and any canonicalization failure is `false` -- fail closed.
pub fn path_within_root(root: &Path, target: &Path) -> bool {
    pathnorm::path_within_root(root, target)
}

/// The ONE predicate for "may this request touch this path?".
///
/// Returns the canonical, verbatim-stripped absolute path on success so the
/// caller can use it directly (no second resolution step that could disagree).
///
/// `root` is the session's working directory (decision Q4 of the design). Empty
/// requested paths are rejected before anything touches the filesystem.
pub fn authorize_file_access(
    root: &Path,
    requested: &str,
    op: FileOp,
    policy: &FilePolicy,
) -> Result<PathBuf, FileDeny> {
    authorize_inner(root, requested, op, policy).map(|(path, _)| path)
}

/// Authorize a text preview AND return the content, in ONE resolution + ONE read.
///
/// Prefer this over calling [`authorize_file_access`] and then reading the path
/// yourself: a second resolution could disagree with the first, and reading twice
/// gives a window for the file to change between the check and the use.
///
/// Decoding is lossy on purpose. [`looks_binary`] is the binary gate; a file that
/// passes it but is not valid UTF-8 (latin-1 text, say) is still worth showing,
/// and replacement characters beat a blanket "unreadable" for a preview pane.
pub fn read_text_file(
    root: &Path,
    requested: &str,
    policy: &FilePolicy,
) -> Result<String, FileDeny> {
    let (_, bytes) = authorize_inner(root, requested, FileOp::Read, policy)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// Authorize a directory CREATION.
///
/// Deliberately does **not** apply root confinement: `/fs/mkdir` backs the project
/// picker, which browses and creates directories anywhere (user decision, see §0
/// of `docs/plans/2026-09-25-file-surface-security.md`). What it does enforce is
/// the auth posture and the sensitive-path guard.
///
/// The target usually does not exist yet, so [`pathnorm::canonicalize`] cannot
/// resolve it directly -- this walks up to the nearest existing ancestor and
/// canonicalizes THAT. Doing so also resolves any `..` climb, so a request for
/// `<sensitive-dir>/../sibling` is judged by its real location rather than its
/// spelling.
pub fn authorize_directory_creation(
    requested: &str,
    policy: &FilePolicy,
) -> Result<PathBuf, FileDeny> {
    let requested = requested.trim();
    if requested.is_empty() {
        return Err(FileDeny::EmptyPath);
    }
    if policy.no_auth {
        return Err(FileDeny::NoAuthMode);
    }
    let lexical = Path::new(requested);
    if path_is_sensitive(lexical) {
        return Err(FileDeny::Sensitive);
    }
    let mut cur: Option<&Path> = Some(lexical);
    while let Some(candidate) = cur {
        if let Ok(real) = pathnorm::canonicalize(candidate) {
            if path_is_sensitive(&real) {
                return Err(FileDeny::Sensitive);
            }
            // An existing non-directory in the path cannot become a directory.
            if !real.is_dir() {
                return Err(FileDeny::NotADirectory);
            }
            return Ok(real);
        }
        cur = candidate.parent();
    }
    // Nothing along the path could be resolved (e.g. a nonexistent root).
    Err(FileDeny::Unreadable)
}

fn authorize_inner(
    root: &Path,
    requested: &str,
    op: FileOp,
    policy: &FilePolicy,
) -> Result<(PathBuf, Vec<u8>), FileDeny> {
    let requested = requested.trim();
    if requested.is_empty() {
        return Err(FileDeny::EmptyPath);
    }

    // Checked before any filesystem work: with auth off there is no caller to
    // blame, so the cheap decisive refusal goes first.
    if (op.exposes_content() || op.is_write()) && policy.no_auth {
        return Err(FileDeny::NoAuthMode);
    }

    let root = pathnorm::canonicalize(root).map_err(|_| FileDeny::EscapesRoot)?;
    let unresolved = Path::new(requested);
    let unresolved = if unresolved.is_absolute() {
        unresolved.to_path_buf()
    } else {
        root.join(unresolved)
    };
    let target = pathnorm::canonicalize(&unresolved).map_err(|_| FileDeny::EscapesRoot)?;

    if !path_within_root(&root, &target) {
        return Err(FileDeny::EscapesRoot);
    }
    if path_is_sensitive(&target) {
        return Err(FileDeny::Sensitive);
    }

    match op {
        FileOp::Read | FileOp::Download => {
            if !target.is_file() {
                return Err(FileDeny::NotRegularFile);
            }
            let bytes = read_capped(&target, policy.max_bytes)?;
            if looks_binary(&bytes) {
                return Err(FileDeny::Binary);
            }
            return Ok((target, bytes));
        }
        FileOp::Open => {
            if !target.is_file() {
                return Err(FileDeny::NotRegularFile);
            }
        }
        FileOp::Package => {
            if !target.is_dir() {
                return Err(FileDeny::NotADirectory);
            }
        }
        // List / Search / Mkdir are bounded by the root + sensitive checks above.
        FileOp::List | FileOp::Search | FileOp::Mkdir => {}
    }

    Ok((target, Vec::new()))
}

/// Read at most `max` bytes, reporting [`FileDeny::TooLarge`] instead of
/// truncating silently -- a caller that cannot tell "whole file" from "prefix"
/// cannot honestly render a preview.
fn read_capped(path: &Path, max: u64) -> Result<Vec<u8>, FileDeny> {
    use std::io::Read;
    let file = std::fs::File::open(path).map_err(|_| FileDeny::Unreadable)?;
    let mut buf = Vec::new();
    file.take(max.saturating_add(1))
        .read_to_end(&mut buf)
        .map_err(|_| FileDeny::Unreadable)?;
    if buf.len() as u64 > max {
        return Err(FileDeny::TooLarge);
    }
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workspace() -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        std::fs::create_dir(d.path().join("ws")).unwrap();
        d
    }

    fn policy() -> FilePolicy {
        FilePolicy::default()
    }

    #[test]
    fn empty_and_whitespace_paths_are_rejected() {
        let d = workspace();
        let ws = d.path().join("ws");
        std::fs::write(ws.join("a.txt"), "hi").unwrap();
        for bad in ["", "   ", "\t"] {
            assert_eq!(
                authorize_file_access(&ws, bad, FileOp::Read, &policy()),
                Err(FileDeny::EmptyPath),
                "should reject {bad:?}"
            );
        }
    }

    #[test]
    fn dotdot_escape_is_rejected() {
        let d = workspace();
        let ws = d.path().join("ws");
        std::fs::write(d.path().join("outside.txt"), "secret").unwrap();
        // A relative climb out of the workspace, and an absolute path outside it.
        let rel = "../outside.txt";
        assert_eq!(
            authorize_file_access(&ws, rel, FileOp::Read, &policy()),
            Err(FileDeny::EscapesRoot)
        );
        let abs = d.path().join("outside.txt");
        assert_eq!(
            authorize_file_access(&ws, &abs.to_string_lossy(), FileOp::Read, &policy()),
            Err(FileDeny::EscapesRoot)
        );
    }

    /// The whole point of the module: canonicalize BEFORE the prefix test, or a
    /// symlink inside the workspace silently hands over files outside it.
    #[cfg(unix)]
    #[test]
    fn symlink_pointing_outside_the_workspace_is_rejected() {
        let d = workspace();
        let ws = d.path().join("ws");
        let outside = d.path().join("secrets");
        std::fs::create_dir(&outside).unwrap();
        std::fs::write(outside.join("id_rsa"), "PRIVATE KEY").unwrap();
        std::os::unix::fs::symlink(&outside, ws.join("link")).unwrap();

        assert_eq!(
            authorize_file_access(&ws, "link/id_rsa", FileOp::Read, &policy()),
            Err(FileDeny::EscapesRoot)
        );
        // Direct helper-level pin: the link itself is "in" the workspace only
        // lexically, so the predicate must say no.
        assert!(!path_within_root(&ws, &ws.join("link").join("id_rsa")));
    }

    #[test]
    fn missing_and_unresolvable_paths_are_rejected() {
        let d = workspace();
        let ws = d.path().join("ws");
        // Nonexistent target, and a root that does not exist at all.
        assert_eq!(
            authorize_file_access(&ws, "nope.txt", FileOp::Read, &policy()),
            Err(FileDeny::EscapesRoot)
        );
        assert_eq!(
            authorize_file_access(
                &d.path().join("no-such-root"),
                "a.txt",
                FileOp::Read,
                &policy(),
            ),
            Err(FileDeny::EscapesRoot)
        );
        assert!(!path_within_root(&d.path().join("no-such-root"), &ws));
    }

    /// The Windows verbatim prefix would make a naive `starts_with` fail, so the
    /// canonical form must have it stripped (`pathnorm::canonicalize` does).
    #[test]
    fn verbatim_prefixed_roots_still_match() {
        let d = workspace();
        let ws = d.path().join("ws");
        assert!(path_within_root(&ws, &ws));
        // Directories are returned verbatim for List.
        assert_eq!(
            authorize_file_access(&ws, ".", FileOp::List, &policy()),
            Ok(pathnorm::canonicalize(&ws).unwrap())
        );
    }

    #[test]
    fn in_workspace_text_file_is_authorized() {
        let d = workspace();
        let ws = d.path().join("ws");
        std::fs::write(ws.join("main.rs"), "fn main() {}").unwrap();
        let got = authorize_file_access(&ws, "main.rs", FileOp::Read, &policy()).unwrap();
        assert_eq!(got, pathnorm::canonicalize(&ws.join("main.rs")).unwrap());
        // Absolute in-workspace paths are fine too.
        let abs = ws.join("main.rs");
        assert_eq!(
            authorize_file_access(&ws, &abs.to_string_lossy(), FileOp::Read, &policy()),
            Ok(got.clone())
        );
    }

    /// Sensitive paths are refused even when they sit INSIDE the workspace
    /// (a checked-in `.ssh` dir, or the user's config with provider api keys).
    #[test]
    fn sensitive_paths_are_refused_even_inside_the_workspace() {
        let d = workspace();
        let ws = d.path().join("ws");
        let ssh = ws.join(".ssh");
        std::fs::create_dir(&ssh).unwrap();
        std::fs::write(ssh.join("id_rsa"), "PRIVATE KEY").unwrap();

        assert_eq!(
            authorize_file_access(&ws, ".ssh/id_rsa", FileOp::Read, &policy()),
            Err(FileDeny::Sensitive)
        );
    }

    #[test]
    fn content_and_write_ops_fail_closed_when_auth_is_off() {
        let d = workspace();
        let ws = d.path().join("ws");
        std::fs::write(ws.join("a.txt"), "hi").unwrap();
        let no_auth = FilePolicy {
            no_auth: true,
            ..policy()
        };
        // Checked BEFORE any filesystem work, so it wins even for a valid file.
        assert_eq!(
            authorize_file_access(&ws, "a.txt", FileOp::Read, &no_auth),
            Err(FileDeny::NoAuthMode)
        );
        assert_eq!(
            authorize_file_access(&ws, "a.txt", FileOp::Download, &no_auth),
            Err(FileDeny::NoAuthMode)
        );
        assert_eq!(
            authorize_file_access(&ws, ".", FileOp::Mkdir, &no_auth),
            Err(FileDeny::NoAuthMode)
        );
        // Metadata-only ops stay available with auth off (decision Q1).
        assert!(authorize_file_access(&ws, ".", FileOp::List, &no_auth).is_ok());
    }

    #[test]
    fn oversized_files_are_refused_not_truncated() {
        let d = workspace();
        let ws = d.path().join("ws");
        std::fs::write(ws.join("big.txt"), vec![b'a'; 4096]).unwrap();
        let tight = FilePolicy {
            max_bytes: 1024,
            ..policy()
        };
        assert_eq!(
            authorize_file_access(&ws, "big.txt", FileOp::Read, &tight),
            Err(FileDeny::TooLarge)
        );
        // Exactly at the cap is fine.
        let exact = FilePolicy {
            max_bytes: 4096,
            ..policy()
        };
        assert!(authorize_file_access(&ws, "big.txt", FileOp::Read, &exact).is_ok());
    }

    #[test]
    fn binary_files_are_refused_for_text_preview() {
        let d = workspace();
        let ws = d.path().join("ws");
        // NUL byte -> `looks_binary` flags it.
        std::fs::write(ws.join("blob.bin"), [0u8, 1, 2, 3]).unwrap();
        assert_eq!(
            authorize_file_access(&ws, "blob.bin", FileOp::Read, &policy()),
            Err(FileDeny::Binary)
        );
    }

    #[test]
    fn operation_shape_is_enforced() {
        let d = workspace();
        let ws = d.path().join("ws");
        std::fs::write(ws.join("a.txt"), "hi").unwrap();
        std::fs::create_dir(ws.join("sub")).unwrap();
        // A directory is not previewable text; a file is not a package root.
        assert_eq!(
            authorize_file_access(&ws, "sub", FileOp::Read, &policy()),
            Err(FileDeny::NotRegularFile)
        );
        assert_eq!(
            authorize_file_access(&ws, "a.txt", FileOp::Package, &policy()),
            Err(FileDeny::NotADirectory)
        );
        assert!(authorize_file_access(&ws, "sub", FileOp::Package, &policy()).is_ok());
    }

    /// `/fs/mkdir` keeps its wide reach (the project picker creates directories
    /// anywhere) but must refuse when auth is off -- it is a write primitive on a
    /// daemon that may be bound to 0.0.0.0 with no token.
    #[test]
    fn directory_creation_refuses_when_auth_is_off() {
        let d = workspace();
        let target = d.path().join("ws").join("new");
        let no_auth = FilePolicy {
            no_auth: true,
            ..policy()
        };
        assert_eq!(
            authorize_directory_creation(&target.to_string_lossy(), &no_auth),
            Err(FileDeny::NoAuthMode)
        );
        assert!(authorize_directory_creation(&target.to_string_lossy(), &policy()).is_ok());
    }

    /// Protected SYSTEM prefixes are refused without touching the filesystem (the
    /// lexical check runs first), so this needs no write privileges.
    #[cfg(unix)]
    #[test]
    fn directory_creation_refuses_system_protected_prefixes() {
        assert_eq!(
            authorize_directory_creation("/etc/rustcode-mkdir-must-not-exist", &policy()),
            Err(FileDeny::Sensitive)
        );
    }

    /// HONEST BOUNDARY: `path_is_sensitive` marks a `.ssh`/`.aws`/`.gnupg` directory
    /// only when it sits under the REAL home, plus system prefixes, plus secret
    /// FILE names/extensions. A directory merely *named* `.ssh` inside an arbitrary
    /// project is therefore NOT flagged -- which is why the earlier `Read` test
    /// passes on `.ssh/id_rsa` by matching the FILE name `id_rsa`, not the
    /// directory. Do not "fix" this locally: the definition is shared with write
    /// approval, and a second definition here is exactly the drift this module
    /// exists to prevent.
    #[test]
    fn directory_creation_does_not_flag_a_plain_dot_ssh_directory() {
        let d = workspace();
        let ws = d.path().join("ws");
        let ssh = ws.join(".ssh");
        std::fs::create_dir(&ssh).unwrap();
        assert!(
            authorize_directory_creation(&ssh.join("nested").to_string_lossy(), &policy()).is_ok(),
            "a workspace-local `.ssh` dir is not in the shared sensitive set"
        );
        // A secret FILE name in the final component IS flagged.
        assert_eq!(
            authorize_directory_creation(&ws.join("id_rsa").to_string_lossy(), &policy()),
            Err(FileDeny::Sensitive)
        );
    }

    /// The target usually does not exist, so the check resolves the nearest
    /// EXISTING ancestor; that must still reject a `..` climb into a sensitive
    /// directory, and must not reject a plain new directory.
    #[test]
    fn directory_creation_resolves_the_existing_ancestor() {
        let d = workspace();
        let ws = d.path().join("ws");
        std::fs::create_dir(ws.join("real")).unwrap();

        // Deep new directory under an existing one: allowed.
        let deep = ws.join("real").join("a").join("b");
        assert!(authorize_directory_creation(&deep.to_string_lossy(), &policy()).is_ok());

        // `..` climb out of a sensitive dir is judged by where it really lands.
        let ssh = ws.join(".ssh");
        std::fs::create_dir(&ssh).unwrap();
        let escape = ssh.join("..").join("plain-new");
        // `.ssh/..` resolves to the workspace itself, which is NOT sensitive.
        assert!(authorize_directory_creation(&escape.to_string_lossy(), &policy()).is_ok());

        // Empty is always rejected.
        assert_eq!(
            authorize_directory_creation("   ", &policy()),
            Err(FileDeny::EmptyPath)
        );
    }

    #[test]
    fn directory_creation_refuses_an_existing_file_ancestor() {
        let d = workspace();
        let ws = d.path().join("ws");
        std::fs::write(ws.join("a.txt"), "hi").unwrap();
        assert_eq!(
            authorize_directory_creation(
                &ws.join("a.txt").join("under").to_string_lossy(),
                &policy()
            ),
            Err(FileDeny::NotADirectory)
        );
    }

    #[test]
    fn op_classification_matches_the_design() {
        assert!(FileOp::Read.exposes_content());
        assert!(FileOp::Download.exposes_content());
        assert!(FileOp::Package.exposes_content());
        assert!(!FileOp::List.exposes_content());
        assert!(!FileOp::Open.exposes_content());
        assert!(FileOp::Mkdir.is_write());
        assert!(!FileOp::Read.is_write());
    }
}
