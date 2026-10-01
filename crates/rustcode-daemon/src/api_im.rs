//! IM channel configuration API — the WebUI slice for managing IM channels.
//!
//! Mirrors `api_provider.rs`: each handler is thin, pushes validation and
//! persistence through [`update_config`] (the store's compare-and-write loop,
//! which already handles concurrent IDE/TUI writes), and reports problems as
//! localized `Msg` diagnostics so the WebUI renders them in the user's language.
//!
//! Route map (registered in `lib.rs` next to the other `/im` reads):
//!
//! ```text
//! GET    /im/channels            list configured channels (definitions, not secrets)
//! PUT    /im/channels            replace the whole channel list + enabled flag
//! DELETE /im/channels/:index     remove one channel by its list position
//! POST   /im/enabled             flip just the master switch
//! POST   /im/channels/test       run one channel's gateway handshake (verdict
//!                                only -- credentials and tickets never echoed)
//! ```
//!
//! Two deliberate constraints carried over from the CLI side:
//!
//! - **Credentials stay `env:` references.** The API stores whatever the UI
//!   sends verbatim (the UI sends `$VAR` spellings); `ImChannelConfig::credential`
//!   expands them at use time. The read handler returns the raw configured
//!   strings, never the expanded values, so expanded secrets never round-trip
//!   through the browser.
//! - **A broken channel list is rejected, not clipped.** Validation runs the same
//!   `ImConfig::validate()` the CLI serve path uses, so a channel the daemon
//!   would refuse to serve cannot be silently saved from the WebUI.

use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use rustcode_capabilities::im_probe::endpoint_host;
use rustcode_config::config::im::{ImChannelConfig, ImConfig, ImPlatform};
use rustcode_config::i18n::{t, Msg};
use serde::{Deserialize, Serialize};

use crate::{api_config::update_config, json_error};

/// One channel as the WebUI sees it: the configured (unexpanded) values.
#[derive(Debug, Clone, Serialize)]
pub struct ImChannelView {
    /// 0-based position in `channels`, for DELETE and stable UI keys.
    pub index: usize,
    pub platform: String,
    /// 1-based position, purely for display ("channel #3").
    pub position: usize,
    /// Recognized platform? Unknown spellings are preserved but flagged so the
    /// UI can render them as broken rather than pretending they are valid.
    pub known_platform: bool,
    /// Credential fields for the recognized platform, raw `env:` spellings.
    pub credentials: std::collections::BTreeMap<String, String>,
    /// Which credential keys the platform requires (so the UI can mark gaps).
    pub required: Vec<String>,
    pub project: String,
    pub enabled: bool,
    /// Sender allowlist echo (`env:`-free plain ids; empty = anyone). The UI
    /// must send it back on PUT or a whole-list replace silently empties it.
    pub allow_senders: Vec<String>,
}

/// `GET /im/channels` — configured channel definitions.
pub async fn get_im_channels() -> impl IntoResponse {
    im_channels_response().await
}

/// Shared body for the read path, so the mutating handlers can re-render the
/// resulting list with a concrete `Response` (an `impl IntoResponse` opaque
/// cannot be returned from a second match arm).
async fn im_channels_response() -> axum::response::Response {
    let config = match load_im_config() {
        Ok(config) => config,
        Err(error) => return json_error(StatusCode::INTERNAL_SERVER_ERROR, error).into_response(),
    };
    let channels: Vec<ImChannelView> = config
        .channels
        .iter()
        .enumerate()
        .map(|(index, channel)| view_channel(index, channel))
        .collect();
    Json(ImChannelsResponse {
        enabled: config.enabled,
        channels,
    })
    .into_response()
}

#[derive(Serialize)]
struct ImChannelsResponse {
    enabled: bool,
    channels: Vec<ImChannelView>,
}

fn load_im_config() -> Result<ImConfig, String> {
    Ok(rustcode_config::ConfigStore::default_store()
        .read()
        .map_err(|e| format!("Failed to read config: {e:#}"))?
        .config
        .im)
}

/// Build the WebUI view of one channel. Raw credential spellings only.
fn view_channel(index: usize, channel: &ImChannelConfig) -> ImChannelView {
    let mut credentials = std::collections::BTreeMap::new();
    for field in [
        "client_id",
        "client_secret",
        "app_id",
        "app_secret",
        "bot_id",
        "secret",
    ] {
        if let Some(value) = channel.credential_raw_for_ui(field) {
            credentials.insert(field.to_string(), value);
        }
    }
    let known = channel.parsed_platform();
    ImChannelView {
        index,
        position: index + 1,
        platform: channel.platform.clone(),
        known_platform: known.is_some(),
        credentials,
        required: known
            .map(|p| {
                p.required_credentials()
                    .iter()
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default(),
        project: channel.project.clone(),
        enabled: channel.enabled,
        allow_senders: channel.allow_senders.clone(),
    }
}

/// `PUT /im/channels` — replace the channel list wholesale.
///
/// Whole-list replacement (rather than per-channel PATCH) keeps the model
/// identical to the config file the CLI reads: the UI edits a document, the
/// server validates the result. Concurrent edits are serialized by the
/// `ConfigStore::update` loop, so two tabs cannot interleave half a list.
pub async fn put_im_channels(Json(req): Json<PutImChannelsRequest>) -> impl IntoResponse {
    let mut config = ImConfig {
        enabled: req.enabled,
        channels: req.channels,
    };
    if let Err(problem) = validate_channels(&mut config) {
        return json_error(StatusCode::BAD_REQUEST, problem).into_response();
    }

    let result = update_config(|cfg| {
        cfg.im = config.clone();
        Ok(())
    });
    match result {
        Ok(_) => im_channels_response().await,
        Err(error) => json_error(StatusCode::INTERNAL_SERVER_ERROR, error).into_response(),
    }
}

#[derive(Deserialize)]
pub struct PutImChannelsRequest {
    pub enabled: bool,
    /// Full replacement list, in order.
    pub channels: Vec<ImChannelConfig>,
}

/// `POST /im/enabled` — flip only the master switch, leaving channels alone.
pub async fn post_im_enabled(Json(req): Json<PostImEnabledRequest>) -> impl IntoResponse {
    let result = update_config(|cfg| {
        cfg.im.enabled = req.enabled;
        Ok(())
    });
    match result {
        Ok(_) => im_channels_response().await,
        Err(error) => json_error(StatusCode::INTERNAL_SERVER_ERROR, error).into_response(),
    }
}

#[derive(Deserialize)]
pub struct PostImEnabledRequest {
    pub enabled: bool,
}

/// `DELETE /im/channels/:index` — remove one channel by list position.
///
/// Index-based (not id-based) because channel definitions have no stable id in
/// config. The handler re-reads under the update lock and validates the index
/// against the *current* list, so a stale UI cannot delete the wrong row after
/// another client reordered the list.
pub async fn delete_im_channel(Path(index): Path<usize>) -> impl IntoResponse {
    let result = update_config(|cfg| {
        if index >= cfg.im.channels.len() {
            return Err(anyhow::anyhow!(
                "{}",
                t(Msg::DaemonImChannelIndexOutOfRange { index }).into_owned()
            ));
        }
        cfg.im.channels.remove(index);
        Ok(())
    });
    match result {
        Ok(_) => im_channels_response().await,
        Err(error) => json_error(StatusCode::BAD_REQUEST, error).into_response(),
    }
}

// ---------------------------------------------------------------------------
// Connectivity test: POST /im/channels/test
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct TestImChannelRequest {
    /// Position of the channel in the configured list (same addressing as
    /// DELETE). The channel is read from the *saved* config, so the WebUI tests
    /// exactly what will be served, not what a form currently holds.
    pub index: usize,
}

/// Verdict of one connectivity test.
///
/// `message` is the localized human-readable summary. On success `endpoint_host`
/// names the host the gateway issued (for display only) -- the dial ticket and
/// the expanded credentials are NEVER included: this response goes to a browser.
#[derive(Debug, Clone, Serialize)]
pub struct ImTestResult {
    pub ok: bool,
    pub platform: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint_host: Option<String>,
}

/// `POST /im/channels/test` — run the platform's gateway handshake for one
/// configured channel and report whether its credentials work.
///
/// Status-code contract:
///
/// - `404` — no channel at that index (the list changed under the UI);
/// - `400` — unknown platform spelling, or a required credential that is unset
///   or expands to empty (the caller must fix the channel first);
/// - `200` with `ok: false` — the route answered but the *test* failed (probe
///   error, or a platform whose probe is not implemented yet).
///
/// A disabled channel is still testable on purpose: verifying credentials
/// before flipping the master switch is the normal sequence. The gateway
/// override honors the same `RUSTCODE_DINGTALK_GATEWAY` env the CLI serve path
/// uses, so the test exercises the endpoint the channel will actually dial.
pub async fn test_im_channel(Json(req): Json<TestImChannelRequest>) -> impl IntoResponse {
    use rustcode_capabilities::im_probe::probe_dingtalk;

    let config = match load_im_config() {
        Ok(config) => config,
        Err(error) => return json_error(StatusCode::INTERNAL_SERVER_ERROR, error).into_response(),
    };
    let Some(channel) = config.channels.get(req.index) else {
        return json_error(
            StatusCode::NOT_FOUND,
            t(Msg::DaemonImChannelIndexOutOfRange { index: req.index }).into_owned(),
        )
        .into_response();
    };

    let Some(platform) = channel.parsed_platform() else {
        return json_error(
            StatusCode::BAD_REQUEST,
            t(Msg::DaemonImUnknownPlatform {
                position: req.index + 1,
                platform: &channel.platform,
            })
            .into_owned(),
        )
        .into_response();
    };
    let platform_str = platform.as_str();

    // Only DingTalk has a probe today. Feishu/WeCom report "unsupported"
    // plainly instead of pretending the channel is broken.
    let probe = match platform {
        ImPlatform::Dingtalk => {
            let Some(client_id) = channel.credential("client_id") else {
                return test_missing_credential_response(req.index, platform_str, "client_id");
            };
            let Some(client_secret) = channel.credential("client_secret") else {
                return test_missing_credential_response(req.index, platform_str, "client_secret");
            };
            let gateway = std::env::var("RUSTCODE_DINGTALK_GATEWAY").unwrap_or_default();
            Some(probe_dingtalk(&client_id, &client_secret, &gateway).await)
        }
        ImPlatform::Feishu | ImPlatform::Wecom => None,
    };

    let result = match probe {
        Some(Ok(conn)) => {
            let host = endpoint_host(&conn.endpoint);
            ImTestResult {
                ok: true,
                platform: platform_str.to_string(),
                message: t(Msg::DaemonImTestOk {
                    platform: platform_str,
                    endpoint_host: host,
                })
                .into_owned(),
                endpoint_host: Some(host.to_string()),
            }
        }
        Some(Err(error)) => {
            // The probe's Display carries the gateway's response text for
            // diagnosis; it never contains the credentials (they are only in
            // the request body, which the gateway does not echo).
            let detail = error.to_string();
            ImTestResult {
                ok: false,
                platform: platform_str.to_string(),
                message: t(Msg::DaemonImTestFailed {
                    platform: platform_str,
                    error: &detail,
                })
                .into_owned(),
                endpoint_host: None,
            }
        }
        None => ImTestResult {
            ok: false,
            platform: platform_str.to_string(),
            message: t(Msg::DaemonImTestUnsupported {
                platform: platform_str,
            })
            .into_owned(),
            endpoint_host: None,
        },
    };
    Json(result).into_response()
}

/// 400 for a channel whose required credential is unset or expands to empty.
fn test_missing_credential_response(
    index: usize,
    platform: &str,
    field: &'static str,
) -> axum::response::Response {
    json_error(
        StatusCode::BAD_REQUEST,
        t(Msg::CfgDiagImMissingCredential {
            position: index + 1,
            platform,
            field,
        })
        .into_owned(),
    )
    .into_response()
}

/// Shared validation: normalize platforms, then run the config-layer checks.
///
/// Runs the same `ImConfig::validate()` used by the CLI serve path, so the WebUI
/// cannot save a list the daemon would refuse to serve.
fn validate_channels(config: &mut ImConfig) -> Result<(), String> {
    // Normalize/verify each platform spelling up front so the error names the
    // offending row instead of a generic "validation failed".
    for (position, channel) in config.channels.iter_mut().enumerate() {
        let trimmed = channel.platform.trim().to_ascii_lowercase();
        if ImPlatform::parse(&trimmed).is_none() {
            return Err(t(Msg::DaemonImUnknownPlatform {
                position: position + 1,
                platform: &channel.platform,
            })
            .into_owned());
        }
        channel.platform = trimmed;
    }
    let problems = config.validate();
    if problems.is_empty() {
        return Ok(());
    }
    // One problem per response keeps the UI actionable (fix this, retry).
    Err(problems.into_iter().next().unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustcode_config::config::im::ImChannelConfig;

    fn channel(platform: &str, project: &str) -> ImChannelConfig {
        ImChannelConfig {
            platform: platform.into(),
            project: project.into(),
            ..ImChannelConfig::default()
        }
    }

    /// Point every credential at a set variable so `validate()` passes.
    ///
    /// Serialized: env mutation is process-global and these tests run in
    /// parallel with the secret-echo test, which also touches env.
    fn with_credentials(mut config: ImConfig) -> ImConfig {
        for channel in &mut config.channels {
            // Case-insensitive: `validate_channels` normalizes spelling AFTER
            // credentials are filled, so "DingTalk" must match "dingtalk" here.
            match channel.platform.to_ascii_lowercase().as_str() {
                "dingtalk" => {
                    channel.client_id = Some("$RUSTCODE_TEST_IM_ID".into());
                    channel.client_secret = Some("$RUSTCODE_TEST_IM_SECRET".into());
                }
                "feishu" => {
                    channel.app_id = Some("$RUSTCODE_TEST_IM_ID".into());
                    channel.app_secret = Some("$RUSTCODE_TEST_IM_SECRET".into());
                }
                "wecom" => {
                    channel.bot_id = Some("$RUSTCODE_TEST_IM_ID".into());
                    channel.secret = Some("$RUSTCODE_TEST_IM_SECRET".into());
                }
                _ => {}
            }
        }
        config
    }

    #[test]
    #[serial_test::serial]
    fn validation_accepts_a_well_formed_list() {
        std::env::set_var("RUSTCODE_TEST_IM_ID", "id-value");
        std::env::set_var("RUSTCODE_TEST_IM_SECRET", "secret-value");
        let mut config = with_credentials(ImConfig {
            enabled: true,
            channels: vec![channel("dingtalk", "/tmp/a"), channel("feishu", "/tmp/b")],
        });
        assert!(validate_channels(&mut config).is_ok());
        // Normalization lowercases the platform spelling.
        assert_eq!(config.channels[0].platform, "dingtalk");
    }

    #[test]
    fn validation_rejects_an_unknown_platform_and_names_the_row() {
        let mut config = ImConfig {
            enabled: true,
            channels: vec![channel("dingtalk", "/tmp/a"), channel("slack", "/tmp/b")],
        };
        let problem = validate_channels(&mut config).unwrap_err();
        assert!(problem.contains("#2"), "error must name the row: {problem}");
        assert!(problem.contains("slack"));
    }

    #[test]
    fn validation_rejects_a_channel_missing_credentials() {
        // DingTalk needs client_id + client_secret; neither is set here.
        let mut config = ImConfig {
            enabled: true,
            channels: vec![channel("dingtalk", "/tmp/a")],
        };
        let problem = validate_channels(&mut config).unwrap_err();
        assert!(!problem.is_empty());
    }

    #[test]
    #[serial_test::serial]
    fn validation_rejects_a_duplicate_pair() {
        std::env::set_var("RUSTCODE_TEST_IM_ID", "id-value");
        std::env::set_var("RUSTCODE_TEST_IM_SECRET", "secret-value");
        // Same platform (after normalization) and same project => duplicate.
        // Both channels get credentials so the only failure the assertion sees
        // is the duplicate itself.
        let mut config = with_credentials(ImConfig {
            enabled: true,
            channels: vec![channel("dingtalk", "/tmp/a"), channel("DingTalk", "/tmp/a")],
        });
        let problem = validate_channels(&mut config).unwrap_err();
        assert!(
            problem.to_lowercase().contains("duplicate") || problem.contains("重复"),
            "got: {problem}"
        );
    }

    #[test]
    fn view_never_exposes_an_expanded_secret() {
        // The whole point of `credential_raw_for_ui`: the raw `env:` spelling is
        // echoed, the expanded value never is. Simulate expansion succeeding by
        // setting the variable, then assert the view still shows the reference.
        std::env::set_var("RUSTCODE_TEST_UI_SECRET", "live-secret-value");
        let mut c = channel("dingtalk", "/tmp/a");
        c.client_id = Some("$RUSTCODE_TEST_UI_SECRET".into());
        let view = view_channel(0, &c);
        assert_eq!(
            view.credentials.get("client_id").map(String::as_str),
            Some("$RUSTCODE_TEST_UI_SECRET"),
            "the view must echo the raw reference"
        );
        let rendered = format!("{view:?}");
        assert!(
            !rendered.contains("live-secret-value"),
            "expanded secret leaked into the view"
        );
        std::env::remove_var("RUSTCODE_TEST_UI_SECRET");
    }

    #[test]
    fn view_flags_an_unknown_platform_without_dropping_it() {
        let mut c = channel("slack", "/tmp/a");
        c.client_id = Some("$X".into());
        let view = view_channel(0, &c);
        assert!(!view.known_platform);
        assert!(
            view.required.is_empty(),
            "unknown platform has no known requirements"
        );
        // Credentials are still echoed so the user can see what was configured.
        assert_eq!(
            view.credentials.get("client_id").map(String::as_str),
            Some("$X")
        );
    }

    #[test]
    fn view_indexes_are_stable_and_one_based_for_display() {
        let config = ImConfig {
            enabled: false,
            channels: vec![channel("dingtalk", "/a"), channel("wecom", "/b")],
        };
        let views: Vec<ImChannelView> = config
            .channels
            .iter()
            .enumerate()
            .map(|(i, c)| view_channel(i, c))
            .collect();
        assert_eq!(views[0].index, 0);
        assert_eq!(views[0].position, 1);
        assert_eq!(views[1].index, 1);
        assert_eq!(views[1].position, 2);
        assert_eq!(views[1].platform, "wecom");
        assert!(views[1].known_platform);
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
        // endpoint URL cannot ride along into the WebUI-visible response.
        let host = endpoint_host("wss://gw.example/connect?ticket=SECRET-TICKET");
        assert!(
            !host.contains("SECRET-TICKET"),
            "ticket leaked via host: {host}"
        );
    }
}
