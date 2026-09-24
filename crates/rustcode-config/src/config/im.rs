//! IM (instant-messaging) channel configuration.
//!
//! Lets a local RustCode project be driven from an IM chat: a user messages a
//! bot they created on the platform, and the message is forwarded to the agent
//! bound to that project, with the reply posted back to the same conversation.
//!
//! Shape (TOML, array-of-tables so one project may have several platforms while
//! each `(project, platform)` pair stays unique):
//!
//! ```toml
//! [im]
//! enabled = false
//!
//! [[im.channels]]
//! platform = "dingtalk"
//! project = "/abs/path/to/workdir"
//! client_id = "$DINGTALK_CLIENT_ID"
//! client_secret = "$DINGTALK_CLIENT_SECRET"
//! ```
//!
//! Two deliberate constraints, both aligned with existing repo behaviour:
//!
//! - **Credentials are bring-your-own and never stored literally.** Values go
//!   through [`crate::config::provider::expand_env_vars`] (the same `$VAR` /
//!   `${VAR}` / `${VAR:-default}` mechanism provider `api_key` uses), so a
//!   config file can be committed without leaking a bot secret. There is
//!   deliberately no second credential syntax.
//! - **The master switch defaults to off**, mirroring the
//!   `HOSTED_RELAY_ENABLED = false` gate style: an unconfigured build behaves
//!   exactly as it does today.

use serde::{Deserialize, Serialize};

/// Supported IM platforms.
///
/// Each maps to a bot the *user* creates on that platform and whose credentials
/// they supply -- this fork ships no hosted bot and no central account, the same
/// BYO posture the provider layer takes for model endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImPlatform {
    /// DingTalk (钉钉). Inbound uses the Stream long connection: the client
    /// dials out over WebSocket, so **no public endpoint is required**.
    Dingtalk,
    /// Feishu / Lark (飞书). Long connection or webhook event subscription.
    Feishu,
    /// WeCom (企业微信). Inbound is a webhook callback, so reaching it from
    /// outside requires a public endpoint (e.g. the bundled reverse tunnel).
    Wecom,
}

impl ImPlatform {
    /// Wire/TOML spelling. Kept stable: it is written into user config files.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dingtalk => "dingtalk",
            Self::Feishu => "feishu",
            Self::Wecom => "wecom",
        }
    }

    /// Parse a TOML spelling. Unknown values are rejected rather than silently
    /// coerced to a default -- a typo'd platform should fail loudly.
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "dingtalk" => Some(Self::Dingtalk),
            "feishu" | "lark" => Some(Self::Feishu),
            "wecom" | "weixin" | "qywx" => Some(Self::Wecom),
            _ => None,
        }
    }

    /// Credential field names this platform requires, in config-key spelling.
    ///
    /// Used by validation so a half-configured channel fails at config load
    /// with a named missing field instead of failing later with an auth error.
    pub fn required_credentials(self) -> &'static [&'static str] {
        match self {
            Self::Dingtalk => &["client_id", "client_secret"],
            Self::Feishu => &["app_id", "app_secret"],
            Self::Wecom => &["bot_id", "secret"],
        }
    }

    /// Whether inbound traffic arrives over a client-initiated long connection
    /// (no public endpoint / tunnel needed) as opposed to a webhook callback.
    ///
    /// Only DingTalk's Stream mode is currently verified against upstream
    /// documentation; the other two are not yet confirmed and must not be
    /// assumed. This predicate is advisory (it drives setup guidance only) and
    /// never gates whether a channel may be enabled.
    pub fn has_verified_long_connection(self) -> bool {
        matches!(self, Self::Dingtalk)
    }
}

/// One bound IM channel: a bot on one platform wired to one project.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ImChannelConfig {
    /// Platform spelling; see [`ImPlatform::parse`]. Empty means "not set".
    pub platform: String,
    /// Absolute working directory this channel is bound to. This is the
    /// "agent identity" for the channel, since the fork has no hosted-agent
    /// concept: the project *is* the agent.
    pub project: String,
    /// Per-channel switch, so one platform can be paused without editing the
    /// table out. Effective only when the `[im] enabled` master switch is on.
    pub enabled: bool,

    // -- DingTalk (client_id + client_secret) --
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,

    // -- Feishu / Lark (app_id + app_secret) --
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_secret: Option<String>,

    // -- WeCom (bot_id + secret) --
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
}

impl Default for ImChannelConfig {
    fn default() -> Self {
        Self {
            platform: String::new(),
            project: String::new(),
            enabled: true,
            client_id: None,
            client_secret: None,
            app_id: None,
            app_secret: None,
            bot_id: None,
            secret: None,
        }
    }
}

impl ImChannelConfig {
    /// Parsed platform, or `None` when unset/unrecognized.
    pub fn parsed_platform(&self) -> Option<ImPlatform> {
        ImPlatform::parse(&self.platform)
    }

    /// Raw configured value for a credential field, before env expansion.
    fn credential_raw(&self, field: &str) -> Option<&str> {
        let value = match field {
            "client_id" => self.client_id.as_deref(),
            "client_secret" => self.client_secret.as_deref(),
            "app_id" => self.app_id.as_deref(),
            "app_secret" => self.app_secret.as_deref(),
            "bot_id" => self.bot_id.as_deref(),
            "secret" => self.secret.as_deref(),
            _ => None,
        };
        value.filter(|v| !v.trim().is_empty())
    }

    /// Raw configured spelling for the WebUI: exactly what the config file
    /// holds (an `env:` reference), never the expanded secret.
    ///
    /// Expanding here would leak credentials into API responses, so callers that
    /// need the real value go through [`Self::credential`] instead.
    pub fn credential_raw_for_ui(&self, field: &str) -> Option<String> {
        self.credential_raw(field).map(str::to_string)
    }

    /// Resolve a credential after environment expansion.
    ///
    /// Returns `None` when the field is unset or expands to empty, so a caller
    /// can fail closed instead of authenticating with a blank secret.
    pub fn credential(&self, field: &str) -> Option<String> {
        let raw = self.credential_raw(field)?;
        let expanded = crate::config::provider::expand_env_vars(raw);
        let trimmed = expanded.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    }
}

/// Effective enablement of a channel: both the master switch and the channel's
/// own switch must be on.
pub fn channel_is_active(cfg: &ImConfig, channel: &ImChannelConfig) -> bool {
    cfg.enabled && channel.enabled
}

/// `[im]` section: master switch plus the list of bound channels.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ImConfig {
    /// Master switch for every IM channel. Defaults to off, so an existing
    /// config that never mentions `[im]` is completely unaffected.
    pub enabled: bool,
    /// Bound channels. Empty is the default and is not an error.
    pub channels: Vec<ImChannelConfig>,
}

impl ImConfig {
    /// `true` when the master switch is on and at least one channel is
    /// configured and active.
    pub fn has_active_channels(&self) -> bool {
        self.channels.iter().any(|c| channel_is_active(self, c))
    }

    /// Whether nothing is configured. Drives `skip_serializing_if` so writing
    /// the config back never materializes an empty `[im]` table for users who
    /// never opted in.
    pub fn is_empty(&self) -> bool {
        !self.enabled && self.channels.is_empty()
    }

    /// Validate the section. Returns one localized diagnostic per problem
    /// (empty ⇒ valid). Pure -- does not mutate and does not read the network.
    ///
    /// Checks, in order: unknown platform spelling, missing `project`, a
    /// required credential that is unset or expands to empty, and a duplicate
    /// `(project, platform)` pair. Duplicates are rejected rather than
    /// last-wins: silently keeping one would let a user believe channel A is
    /// running while channel B actually is.
    pub fn validate(&self) -> Vec<String> {
        use crate::i18n::{t, Msg};
        let mut diags = Vec::new();
        let mut seen: Vec<(String, ImPlatform)> = Vec::new();

        for (index, channel) in self.channels.iter().enumerate() {
            // Address channels by 1-based position: an id does not exist yet at
            // validation time, and the index is what the user counts in the file.
            let position = index + 1;

            let Some(platform) = channel.parsed_platform() else {
                diags.push(
                    t(Msg::CfgDiagImUnknownPlatform {
                        position,
                        platform: &channel.platform,
                    })
                    .into_owned(),
                );
                continue; // credentials cannot be checked without a platform
            };

            if channel.project.trim().is_empty() {
                diags.push(
                    t(Msg::CfgDiagImMissingProject {
                        position,
                        platform: platform.as_str(),
                    })
                    .into_owned(),
                );
            }

            for field in platform.required_credentials() {
                if channel.credential(field).is_none() {
                    diags.push(
                        t(Msg::CfgDiagImMissingCredential {
                            position,
                            platform: platform.as_str(),
                            field,
                        })
                        .into_owned(),
                    );
                }
            }

            let project = channel.project.trim().to_string();
            if seen
                .iter()
                .any(|(p, plat)| *plat == platform && *p == project)
            {
                diags.push(
                    t(Msg::CfgDiagImDuplicateChannel {
                        platform: platform.as_str(),
                        project: &project,
                    })
                    .into_owned(),
                );
            } else {
                seen.push((project, platform));
            }
        }
        diags
    }
}

/// Environment variable name for the IM master switch.
///
/// The authoritative spelling lives in [`crate::endpoints`] alongside every
/// other `RUSTCODE_*` name so a distribution rename touches one place; this is
/// a re-export, not a second literal.
pub use crate::endpoints::IM_ENABLED_ENV;

/// Single resolution point for "are IM channels on?".
///
/// Mirrors [`crate::config::webui_no_auth_enabled`]: the environment variable
/// wins over config so an operator can flip channels off in a container without
/// editing (or mounting over) the user's file. An explicit `=0` therefore
/// overrides a config that says `enabled = true`.
///
/// Every entry point must call this instead of reading the env var or the field
/// directly, so there is never a second source of truth.
pub fn im_enabled_from(config: Option<&ImConfig>) -> bool {
    if let Some(raw) = std::env::var(IM_ENABLED_ENV)
        .ok()
        .filter(|v| !v.trim().is_empty())
    {
        return matches!(
            raw.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "on" | "yes"
        );
    }
    config.is_some_and(|c| c.enabled)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn channel(platform: &str, project: &str) -> ImChannelConfig {
        ImChannelConfig {
            platform: platform.into(),
            project: project.into(),
            ..Default::default()
        }
    }

    #[test]
    fn platform_parsing_is_strict_and_accepts_aliases() {
        assert_eq!(ImPlatform::parse("dingtalk"), Some(ImPlatform::Dingtalk));
        assert_eq!(ImPlatform::parse("  DingTalk "), Some(ImPlatform::Dingtalk));
        assert_eq!(ImPlatform::parse("lark"), Some(ImPlatform::Feishu));
        assert_eq!(ImPlatform::parse("feishu"), Some(ImPlatform::Feishu));
        assert_eq!(ImPlatform::parse("qywx"), Some(ImPlatform::Wecom));
        // A typo must not silently become some default platform.
        assert_eq!(ImPlatform::parse("dingtalks"), None);
        assert_eq!(ImPlatform::parse(""), None);
    }

    #[test]
    fn platform_spellings_roundtrip() {
        for p in [ImPlatform::Dingtalk, ImPlatform::Feishu, ImPlatform::Wecom] {
            assert_eq!(ImPlatform::parse(p.as_str()), Some(p));
        }
    }

    #[test]
    fn only_dingtalk_is_marked_as_verified_long_connection() {
        // The other two are unverified upstream; they must not claim it.
        assert!(ImPlatform::Dingtalk.has_verified_long_connection());
        assert!(!ImPlatform::Feishu.has_verified_long_connection());
        assert!(!ImPlatform::Wecom.has_verified_long_connection());
    }

    #[test]
    fn credential_is_env_expanded_not_stored_literally() {
        std::env::set_var("RUSTCODE_TEST_IM_SECRET", "s3cr3t");
        let mut c = channel("dingtalk", "/tmp/proj");
        c.client_secret = Some("$RUSTCODE_TEST_IM_SECRET".into());
        assert_eq!(c.credential("client_secret").as_deref(), Some("s3cr3t"));
        // An unset variable must not authenticate with an empty secret.
        c.client_secret = Some("$RUSTCODE_TEST_IM_UNSET_VAR".into());
        assert_eq!(c.credential("client_secret"), None);
        // Unset field -> None, never a blank string.
        assert_eq!(c.credential("client_id"), None);
        std::env::remove_var("RUSTCODE_TEST_IM_SECRET");
    }

    #[test]
    fn channel_is_active_requires_both_switches() {
        let mut cfg = ImConfig {
            enabled: true,
            channels: vec![],
        };
        let on = channel("dingtalk", "/tmp/proj");
        assert!(channel_is_active(&cfg, &on));

        let mut off = channel("dingtalk", "/tmp/proj");
        off.enabled = false;
        assert!(!channel_is_active(&cfg, &off));

        cfg.enabled = false;
        assert!(!channel_is_active(&cfg, &on));
        assert!(!cfg.has_active_channels());
    }

    #[test]
    fn im_defaults_to_off_and_empty() {
        let cfg = ImConfig::default();
        assert!(!cfg.enabled, "master switch must default to off");
        assert!(cfg.channels.is_empty());
        assert!(!cfg.has_active_channels());
    }

    #[test]
    fn toml_roundtrips_channels_and_keeps_unused_credentials_out() {
        let toml_src = r#"
            enabled = true
            [[channels]]
            platform = "dingtalk"
            project = "/tmp/proj"
            client_id = "$X"
            client_secret = "$Y"
        "#;
        let cfg: ImConfig = toml::from_str(toml_src).unwrap();
        assert!(cfg.enabled);
        assert_eq!(cfg.channels.len(), 1);
        assert_eq!(
            cfg.channels[0].parsed_platform(),
            Some(ImPlatform::Dingtalk)
        );
        // Fields for other platforms stay absent rather than serializing nulls.
        let out = toml::to_string(&cfg).unwrap();
        assert!(out.contains("dingtalk"));
        assert!(!out.contains("app_id"), "unused platform credential leaked");
        assert!(!out.contains("bot_id"), "unused platform credential leaked");
    }
}
