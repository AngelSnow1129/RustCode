//! Shared path resolution for every capability that touches the rustcode config
//! tree (`mcp` / `session` / `memory`) -- ONE home for the rule, one place to
//! document its single known divergence from production.

use std::path::PathBuf;

/// The rustcode config/data root: `$RUSTCODE_HOME` if set & non-empty, else
/// `~/.rustcode`. Mirrors `rustcode_core::config::Config::config_dir` so everything
/// L1 persists (`sessions/`, `memory.md`, MCP OAuth tokens) lands in the SAME tree
/// as production's.
///
/// KNOWN DIVERGENCE (deliberate L1 simplification): the core helper additionally
/// resolves `$SUDO_USER` via getpwnam, so under `sudo` WITHOUT `$RUSTCODE_HOME` set
/// production resolves the invoking user's home while this resolves root's -- the
/// two stacks would then read/write parallel trees. Setting `$RUSTCODE_HOME`
/// (checked first, byte-identical to production) keeps them aligned.
pub(crate) fn config_dir() -> PathBuf {
    if let Ok(p) = std::env::var("RUSTCODE_HOME") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".rustcode")
}

// NO unit test here ON PURPOSE: testing this means mutating the process-global
// `RUSTCODE_HOME`, and libtest runs the lib's unit tests in parallel threads -- any
// future unit test touching `config_dir()` (memory.md, session paths, OAuth token
// store) would race it nondeterministically. The `$RUSTCODE_HOME`-wins behavior is
// exercised by the env-isolating INTEGRATION binaries (each its own process):
// capabilities `tests/session.rs` + `tests/mcp.rs`, coding `tests/full_assembly.rs`.
