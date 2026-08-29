//! Non-telemetry data types that were historically defined in the
//! (now-deleted) `rustcode-telemetry` crate.
//!
//! This module re-homes ONLY the pure, side-effect-free types that other
//! crates still need after the reporting pipeline was removed: client/session
//! mode tagging, repo-origin detection, and the offline telemetry-consent
//! `CliOverride` shape. Nothing in this module emits, sends, or persists any
//! telemetry. All `Telemetry`/`Queue`/`track`/HTTP-sender behavior is gone.

use serde::{Deserialize, Serialize};

// ---------- SessionMode (client identity tagging, not reporting) ----------

/// Identifies which client/host surface launched a session. Used purely for
/// local behavior branching (e.g. IDE-specific flows), never for reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionMode {
    Headless,
    Tui,
    Ide,
    Vscode,
    Jetbrains,
    #[serde(rename = "webui")]
    Webui,
    #[serde(rename = "rustcode_desktop")]
    RustcodeAir,
    Channel,
}

impl Default for SessionMode {
    fn default() -> Self {
        SessionMode::Ide
    }
}

// ---------- CliOverride (local opt-out flag, no network) ----------

/// Local, in-process override of telemetry consent. Carried only so the
/// `--no-telemetry` flag (now a no-op) and IDE-launched daemons can express
/// intent without a live reporting runtime.
#[derive(Debug, Clone, Copy, Default)]
pub struct CliOverride {
    pub disabled: bool,
}

// ---------- repo-origin detection (pure helper) ----------

/// Best-effort detection of the hosting provider for the current git repo.
/// Pure string inspection; no network.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoOrigin {
    pub host: String,
    pub owner: String,
    pub name: String,
}

/// Inspect a git remote URL and classify its hosting origin. Returns `None`
/// when the URL cannot be parsed as a known host.
pub fn detect_repo_origin(remote_url: &str) -> Option<RepoOrigin> {
    let url = remote_url.trim();
    // Strip a leading SCP-style `git@host:owner/name` into `ssh://host/owner/name`.
    let normalized = if let Some(rest) = url.strip_prefix("git@") {
        format!("ssh://{}", rest.replacen(':', "/", 1))
    } else {
        url.to_string()
    };
    let without_scheme = normalized
        .split_once("://")
        .map(|(_, r)| r)
        .unwrap_or(&normalized);
    // Drop userinfo, path query/fragment.
    let authority = without_scheme.split(['/', '?', '#']).next().unwrap_or("");
    let host_userinfo = authority.rsplit_once('@').map(|(_, h)| h).unwrap_or(authority);
    let host = host_userinfo.split(':').next().unwrap_or(host_userinfo);
    let path = without_scheme.trim_start_matches(authority);
    let path = path.trim_start_matches('/');
    let mut parts = path.splitn(2, '/');
    let owner = parts.next()?;
    let name = parts.next()?;
    let name = name.strip_suffix(".git").unwrap_or(name);
    if owner.is_empty() || name.is_empty() {
        return None;
    }
    Some(RepoOrigin {
        host: host.to_string(),
        owner: owner.to_string(),
        name: name.to_string(),
    })
}
