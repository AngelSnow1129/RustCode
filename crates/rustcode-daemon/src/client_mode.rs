//! Client identity for the daemon HTTP surface.
//!
//! The daemon tags every request with the client that issued it. The tag is
//! read from the `x-rustcode-client` request header (per request) and from the
//! startup flag `--client <value>` (process default). Both funnel into
//! [`ClientMode`], which is then inserted into the axum request extensions by
//! the client-mode middleware so handlers can read it via
//! `axum::Extension<ClientMode>`.

use serde::{Deserialize, Serialize};

/// Daemon-side client identity, parsed from the `x-rustcode-client` request
/// header (startup: `--client <value>`).
///
/// NOTE: this is NOT telemetry. `ClientMode` drives real product behaviour:
/// `client_interactive_permission()` (token/permission enforcement) and the
/// Webui interactive-input path. It replaces the deleted upstream
/// `SessionMode` type; the wire tag strings it parses are kept stable on
/// purpose so existing `extensions/` clients keep working.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum ClientMode {
    Headless,
    Tui,
    #[default]
    Ide,
    Vscode,
    Jetbrains,
    #[serde(rename = "webui")]
    Webui,
    #[serde(rename = "rustcode_desktop")]
    RustcodeAir,
    Channel,
}

/// Map an `x-rustcode-client` header value to [`ClientMode`].
///
/// Known wire tags are matched verbatim so existing `extensions/` clients keep
/// working; unknown values fall back to [`ClientMode::Ide`].
pub fn resolve_client_mode(header: &str) -> ClientMode {
    match header {
        "headless" => ClientMode::Headless,
        "tui" => ClientMode::Tui,
        "ide" => ClientMode::Ide,
        "channel" => ClientMode::Channel,
        "vscode" => ClientMode::Vscode,
        "jetbrains" => ClientMode::Jetbrains,
        "webui" => ClientMode::Webui,
        "rustcode-air" => ClientMode::RustcodeAir,
        _ => ClientMode::Ide,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_wire_tags_are_stable() {
        assert_eq!(resolve_client_mode("headless"), ClientMode::Headless);
        assert_eq!(resolve_client_mode("tui"), ClientMode::Tui);
        assert_eq!(resolve_client_mode("ide"), ClientMode::Ide);
        assert_eq!(resolve_client_mode("channel"), ClientMode::Channel);
        assert_eq!(resolve_client_mode("vscode"), ClientMode::Vscode);
        assert_eq!(resolve_client_mode("jetbrains"), ClientMode::Jetbrains);
        assert_eq!(resolve_client_mode("webui"), ClientMode::Webui);
        assert_eq!(resolve_client_mode("rustcode-air"), ClientMode::RustcodeAir);
    }

    #[test]
    fn unknown_wire_tag_falls_back_to_ide() {
        assert_eq!(resolve_client_mode("nope"), ClientMode::Ide);
        assert_eq!(resolve_client_mode(""), ClientMode::Ide);
        assert_eq!(ClientMode::default(), ClientMode::Ide);
    }
}
