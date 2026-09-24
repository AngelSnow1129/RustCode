//! `rustcode im add|list|check|remove` — CLI management of IM channels.
//!
//! Lives beside `im_runner.rs` in the binary: these handlers are driver logic
//! (argument parsing, process output, exit codes). The pure helpers are kept
//! separate and unit-tested — they encode the credential model, which is the
//! one rule a typo here would silently break.
//!
//! # Credential model (the load-bearing rule)
//!
//! Config stores `env:`-style **references** (`$VAR` / `${VAR}`), never literal
//! secrets — the same model the provider layer uses for `api_key`. To make the
//! common case effortless without weakening the rule,
//! [`normalize_credential_input`] accepts:
//!
//! - `$VAR` / `${VAR}` — kept as-is;
//! - a bare valid environment-variable name (`DINGTALK_CLIENT_SECRET`) —
//!   prefixed with `$` automatically;
//! - anything else (a pasted secret, a URL, prose) — **rejected**, because
//!   accepting it would write a plaintext credential into config.toml.
//!
//! # `add` deliberately skips full validation
//!
//! `ImConfig::validate()` checks that every credential *expands* to non-empty,
//! which requires the environment variables to be set in *this* shell. `add` is
//! a staging step — the user may only set the variables in their shell profile
//! later — so `add` runs structural checks only (platform known, project
//! non-empty, no duplicate `(platform, project)`), and credentials are verified
//! for real by `rustcode im check`, which shares the gateway probe with the
//! daemon's `/im/channels/test` route.

use rustcode_config::config::im::{ImChannelConfig, ImConfig, ImPlatform};
use rustcode_config::config::Config;
use rustcode_config::i18n::{t, Msg};
use rustcode_config::store::ConfigStore;

/// Result of credential normalization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialInput {
    /// Already a `$VAR`/`${VAR}` reference — stored verbatim.
    Reference,
    /// A bare environment-variable name — stored with a `$` prefix.
    PrefixedFromBareName,
}

/// Normalize one credential CLI argument into a config-storable reference.
///
/// `field` is only used for the error message (config-key spelling).
pub fn normalize_credential_input(
    field: &str,
    raw: &str,
) -> Result<(String, CredentialInput), String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(t(Msg::ImAdminCredentialEmpty { field }).into_owned());
    }
    if let Some(inner) = trimmed.strip_prefix('$') {
        let inner = inner.trim_start_matches('{').trim_end_matches('}');
        // `${VAR:-default}` (and the non-expanding `$VAR:-default` typo) embed a
        // literal fallback value; for a credential that fallback IS the
        // plaintext secret, so the form is denied outright instead of stored.
        if inner.contains(":-") {
            return Err(t(Msg::ImAdminCredentialDefaultDenied { field }).into_owned());
        }
        if is_env_name(inner) {
            return Ok((trimmed.to_string(), CredentialInput::Reference));
        }
        return Err(t(Msg::ImAdminCredentialBadVarName { field }).into_owned());
    }
    if is_env_name(trimmed) {
        return Ok((format!("${trimmed}"), CredentialInput::PrefixedFromBareName));
    }
    // Anything else is a literal — a pasted secret, a URL, prose. Writing it
    // into config would defeat the whole credential model, so this is a hard
    // rejection with the fix spelled out, not a warning.
    Err(t(Msg::ImAdminCredentialMustBeEnvRef { field }).into_owned())
}

/// A valid environment-variable name: `[A-Za-z_][A-Za-z0-9_]*`.
fn is_env_name(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && value.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Store one credential on a channel, keyed by config-key spelling.
fn set_credential(channel: &mut ImChannelConfig, field: &str, value: String) {
    match field {
        "client_id" => channel.client_id = Some(value),
        "client_secret" => channel.client_secret = Some(value),
        "app_id" => channel.app_id = Some(value),
        "app_secret" => channel.app_secret = Some(value),
        "bot_id" => channel.bot_id = Some(value),
        "secret" => channel.secret = Some(value),
        _ => {}
    }
}

/// Insert or update one channel, keyed by the normalized `(platform, project)`.
///
/// The credential field names a CLI user may set for each platform are exactly
/// [`ImPlatform::required_credentials`] -- no wrapper is kept here so there is
/// never a second list to drift.
///
/// Returns `true` when an existing channel was replaced (update), `false` when
/// a new one was appended. The platform spelling is normalized (`DingTalk` ->
/// `dingtalk`) so an upsert cannot create a duplicate that differs only by
/// case — `ImConfig::validate()` would then refuse the whole list.
pub fn upsert_channel(config: &mut ImConfig, mut channel: ImChannelConfig) -> bool {
    if let Some(platform) = channel.parsed_platform() {
        channel.platform = platform.as_str().to_string();
    }
    if let Some(existing) = config
        .channels
        .iter_mut()
        .find(|c| c.platform == channel.platform && c.project == channel.project)
    {
        *existing = channel;
        true
    } else {
        config.channels.push(channel);
        false
    }
}

/// Find a channel by platform (case-insensitive) and optional project filter.
pub fn find_channel_index(
    config: &ImConfig,
    platform: &str,
    project: Option<&str>,
) -> Option<usize> {
    let wanted = platform.trim().to_ascii_lowercase();
    config.channels.iter().position(|c| {
        c.platform.eq_ignore_ascii_case(&wanted) && project.is_none_or(|p| c.project == p)
    })
}

/// Every channel matching the filter, as `(index, channel)` pairs.
pub fn matching_channels<'a>(
    config: &'a ImConfig,
    platform: Option<&str>,
    project: Option<&str>,
) -> Vec<(usize, &'a ImChannelConfig)> {
    config
        .channels
        .iter()
        .enumerate()
        .filter(|(_, c)| {
            platform.is_none_or(|p| c.platform.eq_ignore_ascii_case(p.trim()))
                && project.is_none_or(|p| c.project == p)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Handlers (process output + exit codes)
// ---------------------------------------------------------------------------

fn load_config() -> Result<Config, String> {
    ConfigStore::default_store()
        .read()
        .map(|snapshot| snapshot.config)
        .map_err(|e| format!("failed to read config: {e:#}"))
}

/// `rustcode im add` — upsert one channel from CLI arguments.
///
/// Structural checks only (see the module doc for why credentials are not
/// expansion-checked here). Exits 0 on success, 1 on rejection.
pub async fn handle_im_add(
    platform: &str,
    project: &str,
    credentials: &[(String, String)],
) -> anyhow::Result<i32> {
    let parsed = ImPlatform::parse(platform).ok_or_else(|| {
        anyhow::anyhow!(
            "{}",
            t(Msg::ImAdminUnknownPlatform { platform }).into_owned()
        )
    })?;
    if project.trim().is_empty() {
        anyhow::bail!("{}", t(Msg::ImAdminProjectRequired).into_owned());
    }

    // Normalize every credential up front so a rejected one never leaves a
    // half-written channel behind.
    let mut normalized: Vec<(String, String)> = Vec::with_capacity(credentials.len());
    for (field, raw) in credentials {
        let (value, _) = normalize_credential_input(field, raw)
            .map_err(|problem| anyhow::anyhow!("{problem}"))?;
        normalized.push((field.clone(), value));
    }

    let mut channel = ImChannelConfig {
        platform: parsed.as_str().to_string(),
        project: project.trim().to_string(),
        enabled: true,
        ..ImChannelConfig::default()
    };
    for (field, value) in normalized {
        set_credential(&mut channel, &field, value);
    }

    let store = ConfigStore::default_store();
    // `ConfigStore::update`'s closure returns `Result<()>`, so results are
    // captured through the closure environment rather than returned.
    let mut updated = false;
    let mut platform_out = String::new();
    store
        .update(|config| {
            updated = upsert_channel(&mut config.im, channel.clone());
            platform_out = channel.platform.clone();
            Ok(())
        })
        .map_err(|e| anyhow::anyhow!("failed to save config: {e:#}"))?;

    if updated {
        println!(
            "{}",
            t(Msg::ImAdminUpdated {
                platform: &platform_out,
                project: project.trim(),
            })
            .into_owned()
        );
    } else {
        println!(
            "{}",
            t(Msg::ImAdminAdded {
                platform: &platform_out,
                project: project.trim(),
            })
            .into_owned()
        );
    }
    Ok(0)
}

/// `rustcode im list` — print configured channels (raw `env:` spellings only).
pub async fn handle_im_list() -> anyhow::Result<i32> {
    let config = load_config().map_err(|e| anyhow::anyhow!("{e}"))?;
    if config.im.channels.is_empty() {
        println!("{}", t(Msg::ImAdminListEmpty).into_owned());
        return Ok(0);
    }
    for (position, channel) in config.im.channels.iter().enumerate() {
        println!(
            "{}",
            t(Msg::ImAdminChannelHeader {
                position: position + 1
            })
            .into_owned()
        );
        // Config key names and values are not translated (repo rule); only the
        // header line above is prose. Values are the RAW spellings — expanding
        // them here would print secrets into the terminal.
        println!("  platform: {}", channel.platform);
        println!("  project: {}", channel.project);
        println!("  enabled: {}", channel.enabled);
        for field in [
            "client_id",
            "client_secret",
            "app_id",
            "app_secret",
            "bot_id",
            "secret",
        ] {
            if let Some(value) = channel.credential_raw_for_ui(field) {
                println!("  {field}: {value}");
            }
        }
    }
    Ok(0)
}

/// `rustcode im check` — run the gateway probe against matching channels.
///
/// Shares the probe with the daemon's `/im/channels/test` route (single
/// implementation). Disabled channels are checked too: verifying credentials
/// before flipping the master switch is the normal sequence. Exits 0 only when
/// every checked channel passes.
pub async fn handle_im_check(platform: Option<&str>, project: Option<&str>) -> anyhow::Result<i32> {
    use rustcode_capabilities::im_probe::probe_dingtalk;

    let config = load_config().map_err(|e| anyhow::anyhow!("{e}"))?;
    let targets = matching_channels(&config.im, platform, project);
    if targets.is_empty() {
        println!("{}", t(Msg::ImAdminNoMatch).into_owned());
        return Ok(1);
    }

    let mut all_ok = true;
    for (index, channel) in targets {
        let Some(parsed) = channel.parsed_platform() else {
            all_ok = false;
            println!(
                "{}",
                t(Msg::ImAdminCheckFailed {
                    position: index + 1,
                    error: &t(Msg::ImAdminUnknownPlatform {
                        platform: &channel.platform,
                    }),
                })
                .into_owned()
            );
            continue;
        };
        let platform_str = parsed.as_str();

        let outcome = match parsed {
            ImPlatform::Dingtalk => {
                let Some(client_id) = channel.credential("client_id") else {
                    println!(
                        "{}",
                        t(Msg::ImAdminCheckFailed {
                            position: index + 1,
                            error: &t(Msg::CfgDiagImMissingCredential {
                                position: index + 1,
                                platform: platform_str,
                                field: "client_id",
                            }),
                        })
                        .into_owned()
                    );
                    all_ok = false;
                    continue;
                };
                let Some(client_secret) = channel.credential("client_secret") else {
                    println!(
                        "{}",
                        t(Msg::ImAdminCheckFailed {
                            position: index + 1,
                            error: &t(Msg::CfgDiagImMissingCredential {
                                position: index + 1,
                                platform: platform_str,
                                field: "client_secret",
                            }),
                        })
                        .into_owned()
                    );
                    all_ok = false;
                    continue;
                };
                let gateway = std::env::var("RUSTCODE_DINGTALK_GATEWAY").unwrap_or_default();
                Some(probe_dingtalk(&client_id, &client_secret, &gateway).await)
            }
            ImPlatform::Feishu | ImPlatform::Wecom => None,
        };

        match outcome {
            Some(Ok(conn)) => {
                println!(
                    "{}",
                    t(Msg::ImAdminCheckOk {
                        position: index + 1,
                        platform: platform_str,
                        endpoint_host: rustcode_capabilities::im_probe::endpoint_host(
                            &conn.endpoint
                        ),
                    })
                    .into_owned()
                );
            }
            Some(Err(error)) => {
                all_ok = false;
                let detail = error.to_string();
                println!(
                    "{}",
                    t(Msg::ImAdminCheckFailed {
                        position: index + 1,
                        error: &detail,
                    })
                    .into_owned()
                );
            }
            None => {
                all_ok = false;
                println!(
                    "{}",
                    t(Msg::ImAdminCheckUnsupported {
                        platform: platform_str,
                    })
                    .into_owned()
                );
            }
        }
    }
    Ok(if all_ok { 0 } else { 1 })
}

/// `rustcode im remove` — delete one channel by `(platform[, project])`.
pub async fn handle_im_remove(platform: &str, project: Option<&str>) -> anyhow::Result<i32> {
    let store = ConfigStore::default_store();
    // `ConfigStore::update`'s closure returns `Result<()>`, so the removed
    // channel's identity is captured through the closure environment.
    let mut removed: Option<(String, String)> = None;
    store
        .update(|config| {
            let index = find_channel_index(&config.im, platform, project).ok_or_else(|| {
                anyhow::anyhow!("{}", t(Msg::ImAdminNotFound { platform }).into_owned())
            })?;
            let channel = config.im.channels.remove(index);
            removed = Some((channel.platform, channel.project));
            Ok(())
        })
        .map_err(|e| anyhow::anyhow!("{e:#}"))?;
    let (removed_platform, removed_project) =
        removed.ok_or_else(|| anyhow::anyhow!("nothing was removed"))?;

    println!(
        "{}",
        t(Msg::ImAdminRemoved {
            platform: &removed_platform,
            project: &removed_project,
        })
        .into_owned()
    );
    Ok(0)
}

/// `rustcode im register` — install the OS service for one channel.
///
/// The project is resolved before anything is written: an explicit `--project`
/// wins, otherwise the first active channel for this platform in config (the
/// same selection rule `serve` uses, so the registered unit serves the same
/// channel a foreground run would). The path must be an existing absolute
/// directory — a unit pointing at a missing directory would crash-loop at
/// every login, which is exactly the failure this check prevents.
pub async fn handle_im_register(platform: &str, project: Option<&str>) -> anyhow::Result<i32> {
    let platform = crate::im_service_os::checked_platform(platform)?;
    let config = load_config().map_err(|e| anyhow::anyhow!("{e}"))?;
    let project = match project {
        Some(p) => p.trim().to_string(),
        None => config
            .im
            .channels
            .iter()
            .find(|c| {
                c.platform.eq_ignore_ascii_case(&platform)
                    && rustcode_config::config::im::channel_is_active(&config.im, c)
            })
            .map(|c| c.project.clone())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "{}",
                    t(Msg::ImAdminNotFound {
                        platform: &platform
                    })
                    .into_owned()
                )
            })?,
    };
    // Fails loudly on empty / relative / nonexistent paths (see module doc of
    // the runner): the unit would otherwise crash-loop at every login.
    rustcode::im::resolve_project(&project).map_err(|e| anyhow::anyhow!("{e}"))?;

    let spec = crate::im_service_os::ImServiceSpec { platform, project };
    let registrar = crate::im_service_os::current()?;
    // Report a re-registration as such: installing over an existing service is
    // legitimate (config changed), but silently succeeding would hide the
    // fact that a unit was already running.
    let was_installed =
        registrar.status(&spec.platform) == crate::schedule_os::InstallState::Installed;
    registrar.install(&spec)?;
    if was_installed {
        println!(
            "{}",
            t(Msg::ImAdminReRegistered {
                platform: &spec.platform
            })
            .into_owned()
        );
    } else {
        println!(
            "{}",
            t(Msg::ImAdminRegistered {
                platform: &spec.platform
            })
            .into_owned()
        );
    }
    Ok(0)
}

/// `rustcode im unregister` — remove the OS service for a platform.
///
/// Idempotent by registrar contract: removing an absent service is success.
pub async fn handle_im_unregister(platform: &str) -> anyhow::Result<i32> {
    let platform = crate::im_service_os::checked_platform(platform)?;
    let registrar = crate::im_service_os::current()?;
    registrar.uninstall(&platform)?;
    println!(
        "{}",
        t(Msg::ImAdminUnregistered {
            platform: &platform
        })
        .into_owned()
    );
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn channel(platform: &str, project: &str) -> ImChannelConfig {
        ImChannelConfig {
            platform: platform.into(),
            project: project.into(),
            ..ImChannelConfig::default()
        }
    }

    // -- normalize_credential_input -----------------------------------------

    #[test]
    fn bare_env_names_get_a_dollar_prefix() {
        let (value, kind) = normalize_credential_input("client_id", "DINGTALK_CLIENT_ID").unwrap();
        assert_eq!(value, "$DINGTALK_CLIENT_ID");
        assert_eq!(kind, CredentialInput::PrefixedFromBareName);
    }

    #[test]
    fn existing_references_are_kept_verbatim() {
        for raw in ["$VAR", "${VAR}"] {
            let (value, kind) = normalize_credential_input("client_secret", raw).unwrap();
            assert_eq!(value, raw, "{raw} must round-trip");
            assert_eq!(kind, CredentialInput::Reference);
        }
    }

    #[test]
    fn default_value_syntax_is_denied_not_stored() {
        // `${VAR:-fallback}` embeds a literal fallback in the config file; for a
        // credential that fallback IS the plaintext secret, so the form must be
        // rejected rather than stored.
        for raw in ["${VAR:-fallback}", "$VAR:-fallback"] {
            assert!(
                normalize_credential_input("client_secret", raw).is_err(),
                "{raw:?} must be denied"
            );
        }
    }

    #[test]
    fn whitespace_around_the_input_is_trimmed() {
        let (value, _) = normalize_credential_input("client_id", "  MY_VAR  ").unwrap();
        assert_eq!(value, "$MY_VAR");
    }

    #[test]
    fn literal_secrets_are_rejected_not_stored() {
        // The whole point: a pasted secret must never reach config.toml.
        for raw in [
            "sk-live-abc123",
            "https://hook.example/x",
            "let me in",
            "1nvalid",
        ] {
            assert!(
                normalize_credential_input("client_secret", raw).is_err(),
                "literal {raw:?} must be rejected"
            );
        }
    }

    #[test]
    fn empty_credential_is_rejected() {
        assert!(normalize_credential_input("client_id", "").is_err());
        assert!(normalize_credential_input("client_id", "   ").is_err());
    }

    #[test]
    fn malformed_var_references_are_rejected() {
        // `$` followed by nothing name-like is a typo, not a reference.
        assert!(normalize_credential_input("client_id", "$").is_err());
        assert!(normalize_credential_input("client_id", "${}").is_err());
        assert!(normalize_credential_input("client_id", "$1VAR").is_err());
    }

    // -- upsert_channel ------------------------------------------------------

    #[test]
    fn upsert_appends_new_and_replaces_existing() {
        let mut config = ImConfig::default();
        assert!(!upsert_channel(&mut config, channel("dingtalk", "/a")));
        assert_eq!(config.channels.len(), 1);
        // Same (platform, project) -> replace, not append.
        assert!(upsert_channel(&mut config, channel("dingtalk", "/a")));
        assert_eq!(config.channels.len(), 1);
        // Different project -> new channel.
        assert!(!upsert_channel(&mut config, channel("dingtalk", "/b")));
        assert_eq!(config.channels.len(), 2);
    }

    #[test]
    fn upsert_normalizes_platform_spelling_so_case_differences_cannot_duplicate() {
        let mut config = ImConfig::default();
        upsert_channel(&mut config, channel("dingtalk", "/a"));
        // "DingTalk" normalizes to "dingtalk" -> replaces, not a second entry
        // that validate() would then reject as a duplicate pair.
        assert!(upsert_channel(&mut config, channel("DingTalk", "/a")));
        assert_eq!(config.channels.len(), 1);
        assert_eq!(config.channels[0].platform, "dingtalk");
    }

    #[test]
    fn upsert_replaces_credentials_not_merges_them() {
        let mut config = ImConfig::default();
        let mut first = channel("dingtalk", "/a");
        first.client_id = Some("$OLD".into());
        upsert_channel(&mut config, first);

        let mut second = channel("dingtalk", "/a");
        second.client_secret = Some("$NEW".into());
        upsert_channel(&mut config, second);

        // Replacement semantics keep the config file honest: what the user last
        // passed is what is stored, with no stale fields from the previous add.
        assert_eq!(config.channels[0].client_id, None);
        assert_eq!(config.channels[0].client_secret.as_deref(), Some("$NEW"));
    }

    // -- find / matching -----------------------------------------------------

    #[test]
    fn find_channel_matches_case_insensitively_and_honours_project() {
        let config = ImConfig {
            channels: vec![channel("dingtalk", "/a"), channel("feishu", "/b")],
            ..ImConfig::default()
        };
        assert_eq!(find_channel_index(&config, "DINGTALK", None), Some(0));
        assert_eq!(find_channel_index(&config, "feishu", Some("/b")), Some(1));
        assert_eq!(find_channel_index(&config, "feishu", Some("/a")), None);
        assert_eq!(find_channel_index(&config, "wecom", None), None);
    }

    #[test]
    fn matching_channels_returns_all_hits_with_their_indices() {
        let config = ImConfig {
            channels: vec![
                channel("dingtalk", "/a"),
                channel("dingtalk", "/b"),
                channel("feishu", "/c"),
            ],
            ..ImConfig::default()
        };
        let hits = matching_channels(&config, Some("dingtalk"), None);
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].0, 0);
        assert_eq!(hits[1].0, 1);
        assert!(matching_channels(&config, Some("wecom"), None).is_empty());
        assert_eq!(matching_channels(&config, None, None).len(), 3);
    }

    // -- credential field wiring ----------------------------------------------

    #[test]
    fn set_credential_writes_the_matching_field() {
        let mut c = channel("dingtalk", "/a");
        set_credential(&mut c, "client_id", "$ID".into());
        set_credential(&mut c, "client_secret", "$SECRET".into());
        assert_eq!(c.client_id.as_deref(), Some("$ID"));
        assert_eq!(c.client_secret.as_deref(), Some("$SECRET"));
        // Unknown field is a no-op (clap only offers known fields, so this is
        // defence in depth, not a reachable path).
        set_credential(&mut c, "nope", "$X".into());
        assert_eq!(c.bot_id, None);
    }

    #[test]
    fn platform_contract_lists_the_config_key_spelling() {
        // Must stay in lockstep with what `im add` accepts as flags and what
        // validation and the daemon test route both consume: the required
        // credential keys ARE the config-key spellings.
        assert_eq!(
            ImPlatform::Dingtalk.required_credentials(),
            &["client_id", "client_secret"]
        );
        assert_eq!(
            ImPlatform::Feishu.required_credentials(),
            &["app_id", "app_secret"]
        );
        assert_eq!(
            ImPlatform::Wecom.required_credentials(),
            &["bot_id", "secret"]
        );
    }

    // -- locale-independence guard --------------------------------------------

    #[test]
    fn list_output_never_expands_credentials() {
        // The raw spelling guard is enforced by construction in handle_im_list
        // (credential_raw_for_ui, not credential). This test pins the pure
        // half: raw lookup returns the reference, expansion does not happen.
        std::env::set_var("RUSTCODE_TEST_ADMIN_SECRET", "live-secret");
        let mut c = channel("dingtalk", "/a");
        c.client_secret = Some("$RUSTCODE_TEST_ADMIN_SECRET".into());
        assert_eq!(
            c.credential_raw_for_ui("client_secret").as_deref(),
            Some("$RUSTCODE_TEST_ADMIN_SECRET")
        );
        std::env::remove_var("RUSTCODE_TEST_ADMIN_SECRET");
    }
}
