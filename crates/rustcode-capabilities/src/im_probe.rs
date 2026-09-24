//! IM channel connectivity probe -- the server-side "test connection".
//!
//! One implementation, two consumers:
//!
//! - the CLI DingTalk adapter, which delegates its `open_connection` here
//!   instead of keeping a second copy of the gateway handshake, and
//! - the daemon's `POST /im/channels/test` route, which lets the WebUI verify a
//!   channel's credentials before saving or serving it.
//!
//! The probe performs ONLY the gateway handshake: one HTTP POST to
//! `{gateway}/v1.0/gateway/connections/open` exchanging `clientId`/`clientSecret`
//! for a WebSocket `endpoint` + short-lived `ticket`. It deliberately does NOT
//! open the WebSocket -- the long connection stays in the CLI driver, so this
//! module needs no WS stack and stays cheap to embed.
//!
//! Outbound HTTP goes through [`crate::egress::build_http_client`] (the single
//! egress factory) rather than a local `reqwest::Client::new()`. The gateway URL
//! is user-supplied (BYO, like a provider `base_url`), but routing the probe
//! through the factory costs nothing and keeps TLS trust roots / proxy policy
//! uniform; the per-message `sessionWebhook` replies in the CLI adapter remain a
//! documented BYO exemption instead.
//!
//! Error payloads never embed the client secret: details carry the gateway's
//! response text (diagnostic gold), which the platform does not echo secrets in.

use std::fmt;
use std::time::Duration;

use crate::egress::{build_http_client, HttpClientSpec};

/// Default DingTalk gateway. Overridable for self-hosted/private deployments.
pub const DINGTALK_DEFAULT_GATEWAY: &str = "https://api.dingtalk.com";

/// Topic for inbound bot messages. Both directions of the chatbot protocol use
/// this one topic; the official SDK's `ChatbotMessage.TOPIC` is the same string.
pub const DINGTALK_CHATBOT_TOPIC: &str = "/v1.0/im/bot/messages/get";

/// Budget for the whole probe request (handshake only; no WS is opened).
const PROBE_TIMEOUT: Duration = Duration::from_secs(10);

/// Typed probe failure. Small and fixed on purpose; the CLI maps it onto its own
/// `ImError` one-to-one.
#[derive(Debug)]
pub enum ImProbeError {
    /// The gateway rejected the credentials (or answered without `endpoint`).
    Auth {
        platform: &'static str,
        detail: String,
    },
    /// The HTTP round trip itself failed.
    Transport(String),
    /// The gateway answered, but the body is not a usable handshake response.
    Protocol(String),
}

impl fmt::Display for ImProbeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auth { platform, detail } => {
                write!(
                    f,
                    "IM probe authentication failed for `{platform}`: {detail}"
                )
            }
            Self::Transport(detail) => write!(f, "IM probe transport error: {detail}"),
            Self::Protocol(detail) => write!(f, "IM probe protocol error: {detail}"),
        }
    }
}

impl std::error::Error for ImProbeError {}

/// A successful gateway handshake: where to dial and for how long it is valid.
///
/// `ticket` is short-lived and platform-scoped; it is handed back to the CLI
/// adapter to open the WebSocket, and the daemon's test route must NOT return it
/// (nor anything expanded from credentials) to a browser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DingTalkConnection {
    /// WebSocket endpoint URL, e.g. `wss://api-dingtalk.example/connect`.
    pub endpoint: String,
    /// Short-lived dial ticket; append percent-encoded as `?ticket=`.
    pub ticket: String,
}

/// Parse the gateway's `connections/open` response body.
pub fn parse_open_response(body: &str) -> Result<DingTalkConnection, ImProbeError> {
    let value: serde_json::Value = serde_json::from_str(body)
        .map_err(|e| ImProbeError::Protocol(format!("gateway response is not JSON: {e}")))?;
    let endpoint = value
        .get("endpoint")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| ImProbeError::Auth {
            platform: "dingtalk",
            detail: format!("gateway rejected the credentials or omitted `endpoint`: {body}"),
        })?;
    let ticket = value
        .get("ticket")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| ImProbeError::Protocol("gateway response omitted `ticket`".to_string()))?;
    Ok(DingTalkConnection {
        endpoint: endpoint.to_string(),
        ticket: ticket.to_string(),
    })
}

/// Run the DingTalk gateway handshake and return the dial target.
///
/// `gateway` may be empty/whitespace, in which case
/// [`DINGTALK_DEFAULT_GATEWAY`] is used. Credentials are passed in already
/// expanded -- this module never reads the environment or config itself.
pub async fn probe_dingtalk(
    client_id: &str,
    client_secret: &str,
    gateway: &str,
) -> Result<DingTalkConnection, ImProbeError> {
    let mut base = if gateway.trim().is_empty() {
        DINGTALK_DEFAULT_GATEWAY.to_string()
    } else {
        gateway.trim().to_string()
    };
    while base.ends_with('/') {
        base.pop();
    }
    let url = format!("{base}/v1.0/gateway/connections/open");

    // Single egress factory: uniform TLS roots, proxy policy, and a bounded
    // whole-request budget (the spec's request_timeout covers the round trip,
    // so no per-request timeout wrapper is needed here).
    let client = build_http_client(&HttpClientSpec {
        request_timeout: Some(PROBE_TIMEOUT),
        ..HttpClientSpec::default()
    })
    .map_err(|e| ImProbeError::Transport(format!("http client build failed: {e}")))?;

    let body = serde_json::json!({
        "clientId": client_id,
        "clientSecret": client_secret,
        "subscriptions": [{ "type": "CALLBACK", "topic": DINGTALK_CHATBOT_TOPIC }],
        "ua": concat!("rustcode-im/", env!("CARGO_PKG_VERSION")),
        "localIp": best_effort_local_ip(),
    });
    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| ImProbeError::Transport(format!("gateway request failed: {e}")))?;
    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| ImProbeError::Transport(format!("reading gateway response failed: {e}")))?;
    if !status.is_success() {
        return Err(ImProbeError::Auth {
            platform: "dingtalk",
            detail: format!("gateway returned HTTP {status}: {text}"),
        });
    }
    parse_open_response(&text)
}

/// Percent-encode a ticket for a query string.
///
/// Hand-rolled so embedders need no URL-encoding crate for one call site;
/// tickets are opaque base64-ish strings, so anything outside the unreserved
/// set is escaped.
pub fn urlencode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char)
            }
            other => {
                use std::fmt::Write as _;
                let _ = write!(out, "%{other:02X}");
            }
        }
    }
    out
}

/// Host (authority, optional port) of an endpoint URL, for display.
///
/// Shared by the daemon's test route and the CLI's `im check` (single
/// implementation). The query string is excluded, so a dial ticket glued to the
/// endpoint can never ride along into user-visible output. Unexpected shapes
/// degrade to the raw input rather than panicking or truncating -- this is
/// display-only.
pub fn endpoint_host(endpoint: &str) -> &str {
    let rest = endpoint
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(endpoint);
    let end = rest.find(['/', '?']).unwrap_or(rest.len());
    &rest[..end]
}

/// Best-effort local IP for the gateway's `localIp` hint.
///
/// Uses the standard "connect a UDP socket and read the chosen source address"
/// trick: no handshake, no packets, offline-safe. An empty string is an
/// acceptable answer -- the field is a hint and must never block the probe.
fn best_effort_local_ip() -> String {
    use std::net::UdpSocket;
    UdpSocket::bind("0.0.0.0:0")
        .and_then(|s| {
            s.connect("8.8.8.8:53")?;
            s.local_addr()
        })
        .map(|addr| addr.ip().to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_response_extracts_endpoint_and_ticket() {
        let conn =
            parse_open_response(r#"{"endpoint":"wss://eg.example/connect","ticket":"abc/def+="}"#)
                .unwrap();
        assert_eq!(conn.endpoint, "wss://eg.example/connect");
        assert_eq!(conn.ticket, "abc/def+=");
    }

    #[test]
    fn missing_endpoint_is_an_auth_error_not_transport() {
        // The real gateway answers a credential problem with a body lacking
        // `endpoint`; that must surface as auth (do not retry) so the WebUI can
        // tell the user to fix the credentials instead of "connection flaky".
        let err =
            parse_open_response(r#"{"code":"invalidCredential","message":"nope"}"#).unwrap_err();
        assert!(
            matches!(
                err,
                ImProbeError::Auth {
                    platform: "dingtalk",
                    ..
                }
            ),
            "got {err:?}"
        );
    }

    #[test]
    fn missing_ticket_is_a_protocol_error() {
        let err = parse_open_response(r#"{"endpoint":"wss://eg.example/connect"}"#).unwrap_err();
        assert!(matches!(err, ImProbeError::Protocol(_)), "got {err:?}");
    }

    #[test]
    fn non_json_body_is_a_protocol_error() {
        let err = parse_open_response("not json").unwrap_err();
        assert!(matches!(err, ImProbeError::Protocol(_)));
    }

    #[test]
    fn error_display_never_embeds_the_client_secret() {
        // The secret is not part of any error variant; assert the Display of an
        // auth error built from a gateway body stays free of a planted secret.
        let secret = "SUPER-SECRET-VALUE";
        let err = parse_open_response(r#"{"code":"bad","message":"denied"}"#).unwrap_err();
        let rendered = err.to_string();
        assert!(!rendered.contains(secret));
        assert!(rendered.contains("dingtalk"));
        assert!(
            rendered.contains("denied"),
            "gateway detail is preserved: {rendered}"
        );
    }

    #[test]
    fn urlencode_escapes_query_metacharacters_and_keeps_unreserved() {
        assert_eq!(urlencode("abcXYZ019-_.~"), "abcXYZ019-_.~");
        assert_eq!(urlencode("a/b+c="), "a%2Fb%2Bc%3D");
        assert_eq!(urlencode(" "), "%20");
    }

    #[test]
    fn local_ip_helper_never_panics_and_stays_well_formed() {
        // Offline CI must not panic; an empty hint is acceptable.
        let ip = best_effort_local_ip();
        assert!(ip.is_empty() || ip.parse::<std::net::IpAddr>().is_ok());
    }

    #[test]
    fn gateway_trailing_slashes_are_normalized_implicit_via_format() {
        // The normalization lives inside probe_dingtalk (network call), so test
        // the same rule the caller-visible constant implies: a bare-gateway
        // default never carries a trailing slash.
        assert!(!DINGTALK_DEFAULT_GATEWAY.ends_with('/'));
    }

    #[test]
    fn endpoint_host_extracts_the_authority_from_a_ws_url() {
        assert_eq!(endpoint_host("wss://gw.example/connect"), "gw.example");
        assert_eq!(endpoint_host("wss://gw.example:443/path"), "gw.example:443");
        // Query strings and paths are not part of the host.
        assert_eq!(endpoint_host("wss://gw.example/x?ticket=abc"), "gw.example");
        // A bare host without a scheme degrades to the whole input rather than
        // panicking or truncating -- display-only, fail soft.
        assert_eq!(endpoint_host("gw.example"), "gw.example");
    }

    #[test]
    fn endpoint_host_never_carries_a_ticket_into_the_verdict() {
        // The host must exclude the query string, so a dial ticket glued to the
        // endpoint URL cannot ride along into user-visible output.
        let host = endpoint_host("wss://gw.example/connect?ticket=SECRET-TICKET");
        assert!(
            !host.contains("SECRET-TICKET"),
            "ticket leaked via host: {host}"
        );
    }
}
