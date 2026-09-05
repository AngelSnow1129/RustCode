//! Client/session tagging and repo-origin detection helpers.
//!
//! This module is the migration target of the deleted `telemetry_legacy`
//! module. It re-homes ONLY the pure, side-effect-free types that survived the
//! removal of the reporting runtime. Nothing here emits, sends, or persists
//! any telemetry: there is no network client, no queue and no event sender in
//! this module or anywhere else in the workspace.
//!
//! Note on client identity: the `SessionMode` enum that used to live here is
//! now owned by the daemon as `rustcode_daemon::client_mode::ClientMode`. It
//! was moved because the daemon is its only consumer, and it was renamed to
//! match its actual purpose (identifying the calling client, not reporting).
//! The serde wire tags are byte-identical and MUST stay that way, because they
//! are a persisted/over-the-wire contract with the `extensions/` clients --
//! notably the `rustcode_desktop` tag on the desktop variant.

// ---------- repo-origin detection (pure helper, no network) ----------

/// Best-effort detection of the hosting provider for the current git repo.
///
/// Pure string inspection of a git remote URL; performs no network I/O.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoOrigin {
    pub host: String,
    pub owner: String,
    pub name: String,
}

/// Inspect a git remote URL and classify its hosting origin.
///
/// Returns `None` when the URL cannot be parsed into a non-empty
/// `owner`/`name` pair. Accepts `https://`, `ssh://`, bare
/// `host/owner/name` and SCP-style `git@host:owner/name` forms.
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
    let host_userinfo = authority
        .rsplit_once('@')
        .map(|(_, h)| h)
        .unwrap_or(authority);
    let host = host_userinfo.split(':').next().unwrap_or(host_userinfo);
    let path = without_scheme.trim_start_matches(authority);
    let path = path.trim_start_matches('/');
    let (owner, name) = path.split_once('/')?;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn origin(remote: &str) -> (String, String, String) {
        let o = detect_repo_origin(remote).expect("expected a parseable remote");
        (o.host, o.owner, o.name)
    }

    #[test]
    fn parses_https_remote() {
        assert_eq!(
            origin("https://github.com/acme/widgets.git"),
            (
                "github.com".to_string(),
                "acme".to_string(),
                "widgets".to_string()
            )
        );
    }

    #[test]
    fn parses_https_remote_without_git_suffix() {
        assert_eq!(
            origin("https://github.com/acme/widgets"),
            (
                "github.com".to_string(),
                "acme".to_string(),
                "widgets".to_string()
            )
        );
    }

    #[test]
    fn parses_scp_style_remote() {
        assert_eq!(
            origin("git@github.com:acme/widgets.git"),
            (
                "github.com".to_string(),
                "acme".to_string(),
                "widgets".to_string()
            )
        );
    }

    #[test]
    fn parses_ssh_scheme_remote() {
        assert_eq!(
            origin("ssh://git@github.com/acme/widgets.git"),
            (
                "github.com".to_string(),
                "acme".to_string(),
                "widgets".to_string()
            )
        );
    }

    #[test]
    fn strips_userinfo_and_port_from_authority() {
        assert_eq!(
            origin("https://user:token@git.example.com:8443/acme/widgets.git"),
            (
                "git.example.com".to_string(),
                "acme".to_string(),
                "widgets".to_string()
            )
        );
    }

    /// Quirk of the original parser, preserved verbatim: `?`/`#` are only
    /// trimmed while isolating the authority, so a query or fragment attached
    /// to the repo name is kept as part of that name.
    #[test]
    fn query_and_fragment_are_kept_in_the_repo_name() {
        assert_eq!(
            origin("https://github.com/acme/widgets.git?ref=main#frag"),
            (
                "github.com".to_string(),
                "acme".to_string(),
                "widgets.git?ref=main#frag".to_string()
            )
        );
    }

    #[test]
    fn parses_bare_host_slash_path_remote() {
        assert_eq!(
            origin("git.example.com/acme/widgets"),
            (
                "git.example.com".to_string(),
                "acme".to_string(),
                "widgets".to_string()
            )
        );
    }

    #[test]
    fn surrounding_whitespace_is_ignored() {
        assert_eq!(
            origin("  https://github.com/acme/widgets.git\n"),
            (
                "github.com".to_string(),
                "acme".to_string(),
                "widgets".to_string()
            )
        );
    }

    #[test]
    fn rejects_unparseable_remotes() {
        for remote in [
            "",
            "   ",
            "not-a-remote",
            "https://github.com",
            "https://github.com/",
            "https://github.com/acme/",
            "git@github.com:acme",
        ] {
            assert_eq!(
                detect_repo_origin(remote),
                None,
                "expected {remote:?} to be rejected"
            );
        }
    }

    #[test]
    fn keeps_dot_git_in_the_middle_of_a_name() {
        assert_eq!(
            origin("https://github.com/acme/widgets.git.bak"),
            (
                "github.com".to_string(),
                "acme".to_string(),
                "widgets.git.bak".to_string()
            )
        );
    }
}
