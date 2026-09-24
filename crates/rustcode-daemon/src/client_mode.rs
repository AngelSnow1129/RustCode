//! Client identity for the daemon HTTP surface.
//!
//! The daemon tags every request with the client that issued it. The tag is
//! read from the `x-rustcode-client` request header (per request) and from the
//! startup flag `--client <value>` (process default). Both funnel into
//! [`ClientMode`], which is then inserted into the axum request extensions by
//! the client-mode middleware so handlers can read it via
//! `axum::Extension<ClientMode>`.

use serde::{Deserialize, Serialize};

/// Request header carrying the client tag.
///
/// The spelling lives here so the middleware that *reads* it and any client that
/// *sends* it cannot drift apart. Header names are matched case-insensitively by
/// axum, so the lowercase form is canonical.
pub const CLIENT_HEADER: &str = "x-rustcode-client";

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

impl ClientMode {
    /// Reverse of [`resolve_client_mode`]: the wire tag a client should send in
    /// the [`CLIENT_HEADER`] request header.
    ///
    /// Kept beside the parser so the two directions cannot diverge.
    pub fn wire(self) -> &'static str {
        match self {
            Self::Headless => "headless",
            Self::Tui => "tui",
            Self::Ide => "ide",
            Self::Channel => "channel",
            Self::Vscode => "vscode",
            Self::Jetbrains => "jetbrains",
            Self::Webui => "webui",
            Self::RustcodeAir => "rustcode-air",
        }
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

    /// The parser and `wire()` must stay exact inverses, or a client would send
    /// a tag the daemon resolves to a different mode (silently downgrading an IM
    /// channel to `Ide`, which changes approval behaviour).
    #[test]
    fn wire_is_the_exact_inverse_of_parsing() {
        for mode in [
            ClientMode::Headless,
            ClientMode::Tui,
            ClientMode::Ide,
            ClientMode::Channel,
            ClientMode::Vscode,
            ClientMode::Jetbrains,
            ClientMode::Webui,
            ClientMode::RustcodeAir,
        ] {
            assert_eq!(
                resolve_client_mode(mode.wire()),
                mode,
                "`{}` did not round-trip",
                mode.wire()
            );
        }
    }

    #[test]
    fn channel_mode_is_conveyed_by_its_own_wire_tag() {
        // IM adapters must tag themselves `channel` so the daemon treats the
        // request as interactive-capable (there is a human, just an async one)
        // instead of falling back to the default `ide`.
        assert_eq!(ClientMode::Channel.wire(), "channel");
        assert_ne!(ClientMode::Channel.wire(), ClientMode::default().wire());
    }

    #[test]
    fn unknown_wire_tag_falls_back_to_ide() {
        assert_eq!(resolve_client_mode("nope"), ClientMode::Ide);
        assert_eq!(resolve_client_mode(""), ClientMode::Ide);
        assert_eq!(ClientMode::default(), ClientMode::Ide);
    }
}
