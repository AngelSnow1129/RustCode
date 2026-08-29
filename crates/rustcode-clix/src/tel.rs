//! Telemetry for the standalone CLI.
//!
//! The reporting pipeline (the old `rustcode-telemetry` crate) has been removed,
//! so this module no longer emits, sends, or persists any telemetry. It retains
//! only the non-reporting, local-only behavior that `rustcodex` needs:
//!   - reading the `[telemetry]` section from `config.toml` (mirroring rustcode-cli's
//!     precedence, used purely to honor an opt-out intent locally), and
//!   - a one-time consent notice guarded by a marker dotfile.
//!
//! `build_sink` still returns a `Telemetry` handle so the existing call sites in
//! `main.rs` / `code.rs` compile unchanged, but that handle is a no-op stub: its
//! `is_enabled()` reports `false` and `shutdown()` does nothing. The standalone
//! `review` provider is likewise returned unwrapped by `meter_provider` (no
//! metering decorator), since there is no sink to meter against.
//!
//! If a caller ever needs client/session identity for local branching, it is
//! available from `rustcode_config::telemetry_legacy::SessionMode` (pure types,
//! no reporting).

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use serde::Deserialize;

/// How long to wait for the disk queue to drain when flushing on exit. Retained
/// for call-site compatibility even though nothing is flushed now.
pub const FLUSH_TIMEOUT: Duration = Duration::from_secs(3);

const NOTICE: &str = "rustcodex is sending anonymous usage telemetry (token counts, \
    tool/latency stats — no source code or prompt text). Opt out with DO_NOT_TRACK=1, \
    RUSTCODE_TELEMETRY=0, --no-telemetry, or `[telemetry] enabled = false` in config.toml.";

/// `~/.rustcode` (honors `$RUSTCODE_HOME`, else `$HOME` / `%USERPROFILE%`). This is
/// the consent-marker root. Falls back to `./.rustcode` when no home is known.
fn rustcode_dir() -> PathBuf {
    if let Some(home) = std::env::var_os(rustcode_config::distribution::HOME_ENV) {
        return PathBuf::from(home);
    }
    let dir_name = rustcode_config::distribution::HOME_DIR_NAME;
    match std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
        Some(h) => PathBuf::from(h).join(dir_name),
        None => PathBuf::from(dir_name),
    }
}

#[derive(Deserialize, Default)]
struct TelemetryConfig {
    #[serde(default)]
    #[allow(dead_code)]
    enabled: Option<bool>,
}

/// Read the `[telemetry]` section from the config file (default `~/.rustcode/config.toml`,
/// or `config_override`). Any read/parse failure → defaults — telemetry is non-fatal, exactly
/// as rustcode-cli treats it (a broken config must not break `review`/`code`).
fn load_telemetry_config(config_override: Option<&Path>) -> TelemetryConfig {
    let path = match config_override {
        Some(p) => p.to_path_buf(),
        None => match crate::default_config_path() {
            Some(p) => p,
            None => return TelemetryConfig::default(),
        },
    };
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => return TelemetryConfig::default(),
    };
    toml::from_str::<TelFile>(&text)
        .map(|f| f.telemetry)
        .unwrap_or_default()
}

#[derive(Deserialize, Default)]
struct TelFile {
    #[serde(default)]
    telemetry: TelemetryConfig,
}

/// No-op telemetry handle. The reporting runtime is gone, so this exists only to
/// preserve the `Arc<Telemetry>` call-site shape; nothing it touches reports.
pub struct Telemetry;

impl Telemetry {
    /// Reporting is disabled, so this always reports `false` (and the consent
    /// notice is suppressed accordingly).
    pub fn is_enabled(&self) -> bool {
        false
    }

    /// Nothing to flush; this is intentionally a no-op.
    pub async fn shutdown(&self, _timeout: Duration) {}
}

/// Build the (no-op) telemetry handle. Kept for call-site compatibility; the
/// reporting runtime is removed, so no sink is created and `config_override` /
/// `no_telemetry` only inform the local config lookup. Always returns a handle
/// whose `is_enabled()` is `false`.
pub fn build_sink(config_override: Option<&Path>, _no_telemetry: bool) -> Arc<Telemetry> {
    let _ = load_telemetry_config(config_override);
    Arc::new(Telemetry)
}

/// Wrap a provider so each LLM round emits one `LlmChat` (token spend) to `sink`.
///
/// With the reporting runtime removed there is no sink to meter against, so this
/// is now a transparent pass-through: the standalone `review` agent's provider is
/// returned unchanged. Retained only to keep the call-site shape stable.
pub fn meter_provider(
    inner: Arc<dyn rustcode_kernel::provider::LlmProvider>,
    _sink: &Arc<Telemetry>,
    _base_url: &str,
    _model: &str,
) -> Arc<dyn rustcode_kernel::provider::LlmProvider> {
    inner
}

/// Build the OpenAI-compatible provider for the standalone `review` agent (mirrors
/// `rustcode_review::build_review_agent`'s internal construction, but lets us wrap it with
/// telemetry before handing it to `build_review_agent_with`).
pub fn build_review_provider(
    cfg: &rustcode_review::ReviewAgentConfig,
) -> Result<Arc<dyn rustcode_kernel::provider::LlmProvider>, String> {
    use rustcode_capabilities::provider::{OpenAiCompatConfig, OpenAiCompatProvider};
    let mut pc = OpenAiCompatConfig::new(&cfg.api_key, &cfg.base_url, &cfg.model);
    pc.context_window = cfg.context_window;
    // Byte-idle liveness follows the review config's stream_timeout (mirrors
    // `rustcode_review::build_review_agent`), not the adapter's hardcoded 120s.
    pc.idle_timeout = cfg.stream_timeout;
    OpenAiCompatProvider::new(pc)
        .map(|p| Arc::new(p) as Arc<dyn rustcode_kernel::provider::LlmProvider>)
        .map_err(|e| e.message)
}

/// One-time consent notice: when reporting was active, print [`NOTICE`] to stderr
/// and drop a marker dotfile in the queue dir so later runs stay quiet. With the
/// reporting runtime removed `enabled` is always `false`, so this is a no-op.
pub fn maybe_show_notice(enabled: bool) {
    if enabled && notice_once(&rustcode_dir()) {
        eprintln!("{NOTICE}");
    }
}

/// Returns true exactly once per `dir`: shows-and-marks. A write failure returns false (don't
/// claim we notified when we couldn't persist the mark — better a repeat than a silent enable).
fn notice_once(dir: &Path) -> bool {
    let marker = dir.join(".clix_telemetry_notice");
    if marker.exists() {
        return false;
    }
    let _ = std::fs::create_dir_all(dir);
    std::fs::write(&marker, b"shown\n").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_telemetry_opt_out_from_config() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("config.toml");
        std::fs::write(&p, "default_provider=\"x\"\n[telemetry]\nenabled=false\n").unwrap();
        assert_eq!(load_telemetry_config(Some(&p)).enabled, Some(false));
    }

    #[test]
    fn missing_telemetry_section_defaults_to_unset() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("config.toml");
        std::fs::write(&p, "default_provider=\"x\"\n[providers.x]\nmodel=\"m\"\n").unwrap();
        // None ⇒ unset (the built-in default), matching the prior `resolve` behavior.
        assert_eq!(load_telemetry_config(Some(&p)).enabled, None);
    }

    #[test]
    fn notice_shows_once_then_marks() {
        let d = tempfile::tempdir().unwrap();
        assert!(notice_once(d.path()), "first time → show");
        assert!(!notice_once(d.path()), "subsequent → suppressed");
    }

    #[test]
    fn sink_is_always_disabled() {
        let sink = build_sink(None, true);
        assert!(!sink.is_enabled(), "no-op handle must report disabled");
    }
}
