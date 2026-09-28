//! `rustcode im setup|add|list|check|remove` — CLI management of IM channels.
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

use std::io::{self, Write};

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

/// Credential keys required by a platform, in config-key spelling.
pub fn credential_fields_for(platform: ImPlatform) -> &'static [&'static str] {
    platform.required_credentials()
}

/// Fully validated setup input. Validation is side-effect free; in particular,
/// no config is opened until an instance of this type has been produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ValidatedImSetup {
    platform: ImPlatform,
    project: String,
    credentials: Vec<(String, String)>,
}

fn validate_setup_platform_project(
    platform: &str,
    project: &str,
) -> Result<(ImPlatform, String), String> {
    let parsed = ImPlatform::parse(platform)
        .ok_or_else(|| t(Msg::ImAdminUnknownPlatform { platform }).into_owned())?;
    let project = project.trim();
    rustcode::im::resolve_project(project)
        .map_err(|_| t(Msg::ImAdminSetupProjectInvalid { project }).into_owned())?;
    Ok((parsed, project.to_string()))
}

/// Validate and normalize every input needed by `im setup` without reading or
/// writing config. Callers can therefore reject bad paths or literal secrets
/// before delegating persistence to [`handle_im_add`].
pub(crate) fn validate_im_setup_inputs(
    platform: &str,
    project: &str,
    credentials: &[(String, String)],
) -> Result<ValidatedImSetup, String> {
    let (platform, project) = validate_setup_platform_project(platform, project)?;
    let mut normalized = Vec::with_capacity(credential_fields_for(platform).len());

    for &field in credential_fields_for(platform) {
        let raw = credentials
            .iter()
            .find(|(candidate, _)| candidate == field)
            .map(|(_, value)| value.as_str())
            .ok_or_else(|| t(Msg::ImAdminSetupCredentialRequired { field }).into_owned())?;
        let (value, _) = normalize_credential_input(field, raw)?;
        normalized.push((field.to_string(), value));
    }

    Ok(ValidatedImSetup {
        platform,
        project,
        credentials: normalized,
    })
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
/// [`ImPlatform::required_credentials`], exposed through
/// [`credential_fields_for`] so the setup wizard and validation share one list.
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

fn prompt_for_setup_value(prompt: &str) -> anyhow::Result<String> {
    print!("{prompt} ");
    io::stdout().flush()?;
    let mut value = String::new();
    io::stdin().read_line(&mut value)?;
    Ok(value.trim().to_string())
}

fn print_setup_credential_hint(platform: ImPlatform) {
    let message = match platform {
        ImPlatform::Dingtalk => Msg::ImAdminSetupCredentialHintDingtalk,
        ImPlatform::Feishu => Msg::ImAdminSetupCredentialHintFeishu,
        ImPlatform::Wecom => Msg::ImAdminSetupCredentialHintWecom,
    };
    println!("{}", t(message));
}

/// Parse a comma-separated sender allowlist into normalized entries.
///
/// `None` when the input is empty/whitespace ("no allowlist" = unrestricted).
/// Entries are split on commas, trimmed, and blanks dropped; the caller (config
/// `validate`) still flags duplicates, so normalization here only shapes what
/// the user typed.
pub(crate) fn normalize_allow_senders_input(
    raw: Option<&str>,
) -> Result<Option<Vec<String>>, String> {
    let Some(raw) = raw.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(None);
    };
    let senders: Vec<String> = raw
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    if senders.is_empty() {
        // "  ,  " shaped input: treat like empty.
        return Ok(None);
    }
    Ok(Some(senders))
}

/// `rustcode im setup` — interactively collect, save, and verify one channel.
///
/// Every value is validated before [`handle_im_add`] is called, so a rejected
/// literal credential or invalid project cannot cause a partial config write.
/// The optional arguments make the same flow usable without a TTY.
///
/// `allow_senders` is the optional raw allowlist spelling (comma-separated);
/// when absent the wizard prompts once, accepting a blank line as "anyone".
pub async fn handle_im_setup(
    platform: Option<&str>,
    project: Option<&str>,
    credentials: &[(String, String)],
    allow_senders: Option<&str>,
) -> anyhow::Result<i32> {
    println!("{}", t(Msg::ImAdminSetupStepOne));

    let platform = match platform {
        Some(platform) => platform.trim().to_string(),
        None => {
            let prompt = t(Msg::ImAdminSetupPlatformPrompt);
            prompt_for_setup_value(prompt.as_ref())?
        }
    };
    let project = match project {
        Some(project) => project.to_string(),
        None => match std::env::current_dir() {
            Ok(project) => {
                let project = project.to_string_lossy().into_owned();
                println!(
                    "{}",
                    t(Msg::ImAdminSetupProjectDefault { project: &project })
                );
                project
            }
            Err(error) => {
                let error = error.to_string();
                eprintln!("{}", t(Msg::CliSetupCwdError { error: &error }));
                return Ok(1);
            }
        },
    };

    // Validate step 1 before asking for credentials. The complete validation is
    // repeated below over all collected values and remains the write barrier.
    let parsed = match validate_setup_platform_project(&platform, &project) {
        Ok((parsed, _)) => parsed,
        Err(problem) => {
            eprintln!("{problem}");
            return Ok(1);
        }
    };

    println!("{}", t(Msg::ImAdminSetupStepTwo));
    print_setup_credential_hint(parsed);
    let mut collected = Vec::with_capacity(credential_fields_for(parsed).len());
    for &field in credential_fields_for(parsed) {
        let value = if let Some((_, value)) =
            credentials.iter().find(|(candidate, _)| candidate == field)
        {
            value.clone()
        } else {
            let prompt = t(Msg::ImAdminSetupCredentialPrompt { field });
            prompt_for_setup_value(prompt.as_ref())?
        };
        collected.push((field.to_string(), value));
    }

    // Sender allowlist: use the flag spelling when given, otherwise ask once.
    // A blank answer means "anyone" (None), the same as never configuring it.
    let allow_senders =
        match normalize_allow_senders_input(allow_senders).map_err(anyhow::Error::msg)? {
            Some(list) => Some(list),
            None => {
                let prompt = t(Msg::ImAdminSetupAllowSendersPrompt);
                let raw = prompt_for_setup_value(prompt.as_ref())?;
                normalize_allow_senders_input(Some(&raw)).map_err(anyhow::Error::msg)?
            }
        };

    run_im_setup(&platform, &project, &collected, allow_senders).await
}

/// Complete setup from already-collected values, without reading stdin.
///
/// The side-effect-free validation pass is the write barrier: only after every
/// project and credential check succeeds do we call the shared add/check
/// handlers.
async fn run_im_setup(
    platform: &str,
    project: &str,
    credentials: &[(String, String)],
    allow_senders: Option<Vec<String>>,
) -> anyhow::Result<i32> {
    let validated = match validate_im_setup_inputs(platform, project, credentials) {
        Ok(validated) => validated,
        Err(problem) => {
            eprintln!("{problem}");
            return Ok(1);
        }
    };

    println!("{}", t(Msg::ImAdminSetupStepThree));
    let platform = validated.platform.as_str();
    let add_code = match handle_im_add(
        platform,
        &validated.project,
        &validated.credentials,
        allow_senders,
    )
    .await
    {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{error:#}");
            return Ok(1);
        }
    };
    if add_code != 0 {
        return Ok(1);
    }

    let check_code = match handle_im_check(Some(platform), Some(&validated.project)).await {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{error:#}");
            return Ok(1);
        }
    };
    if check_code == 0 {
        println!(
            "{}",
            t(Msg::ImAdminSetupSuccess {
                platform,
                project: &validated.project,
            })
        );
        Ok(0)
    } else {
        eprintln!(
            "{}",
            t(Msg::ImAdminSetupCheckRemediation {
                platform,
                project: &validated.project,
            })
        );
        Ok(1)
    }
}

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
///
/// `allow_senders` is the already-normalized allowlist; `None` (or an empty
/// list) means anyone may drive the agent. It is applied verbatim — duplicates
/// are left for config validation to flag.
pub async fn handle_im_add(
    platform: &str,
    project: &str,
    credentials: &[(String, String)],
    allow_senders: Option<Vec<String>>,
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
        allow_senders: allow_senders.unwrap_or_default(),
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
        // Same non-translated key/value style as the lines above; the empty
        // list means unrestricted, shown explicitly so "anyone" is not a guess.
        if channel.allow_senders.is_empty() {
            println!("  allow_senders: (anyone)");
        } else {
            println!("  allow_senders: {}", channel.allow_senders.join(", "));
        }
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

    // -- setup validation ----------------------------------------------------

    fn dingtalk_setup_credentials(secret: &str) -> Vec<(String, String)> {
        vec![
            ("client_id".into(), "DINGTALK_CLIENT_ID".into()),
            ("client_secret".into(), secret.into()),
        ]
    }

    #[test]
    fn setup_validation_rejects_literal_before_execution() {
        let project = std::env::current_dir().unwrap();
        let error = validate_im_setup_inputs(
            "dingtalk",
            &project.to_string_lossy(),
            &dingtalk_setup_credentials("pasted-secret"),
        )
        .unwrap_err();
        assert!(error.contains("client_secret"));
    }

    #[test]
    fn setup_validation_rejects_relative_and_missing_projects() {
        let credentials = dingtalk_setup_credentials("DINGTALK_CLIENT_SECRET");
        assert!(validate_im_setup_inputs("dingtalk", "relative/project", &credentials).is_err());

        let missing = std::env::current_dir()
            .unwrap()
            .join("rustcode-im-setup-path-that-must-not-exist");
        assert!(
            validate_im_setup_inputs("dingtalk", &missing.to_string_lossy(), &credentials).is_err()
        );
    }

    #[test]
    fn setup_validation_normalizes_only_platform_credentials() {
        let project = std::env::current_dir().unwrap();
        let mut credentials = dingtalk_setup_credentials("$DINGTALK_CLIENT_SECRET");
        credentials.push(("app_secret".into(), "irrelevant-literal".into()));

        let validated =
            validate_im_setup_inputs("DingTalk", &project.to_string_lossy(), &credentials).unwrap();
        assert_eq!(validated.platform, ImPlatform::Dingtalk);
        assert_eq!(
            validated.credentials,
            vec![
                ("client_id".into(), "$DINGTALK_CLIENT_ID".into()),
                ("client_secret".into(), "$DINGTALK_CLIENT_SECRET".into()),
            ]
        );
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

    // -- normalize_allow_senders_input ---------------------------------------

    #[test]
    fn allowlist_none_for_blank_input_means_anyone() {
        assert_eq!(normalize_allow_senders_input(None).unwrap(), None);
        assert_eq!(normalize_allow_senders_input(Some("   ")).unwrap(), None);
        // "  ,  " shaped input has no real entries either.
        assert_eq!(normalize_allow_senders_input(Some(" ,  ,")).unwrap(), None);
    }

    #[test]
    fn allowlist_splits_trims_and_keeps_order() {
        let list = normalize_allow_senders_input(Some(" u1 ,u2, u3 "))
            .unwrap()
            .unwrap();
        assert_eq!(list, vec!["u1", "u2", "u3"]);
    }

    #[test]
    fn allowlist_keeps_duplicates_for_validation_to_flag() {
        // Duplicates survive normalization on purpose: config validation owns
        // the duplicate diagnosis, just like the empty-entry checks.
        let list = normalize_allow_senders_input(Some("u1,u1"))
            .unwrap()
            .unwrap();
        assert_eq!(list, vec!["u1", "u1"]);
    }

    // -- handle_im_add persists the allowlist end-to-end ----------------------

    /// The store writes to `$RUSTCODE_HOME`; point it at a throwaway dir.
    struct HomeGuard(Option<String>);
    impl HomeGuard {
        fn new(dir: &std::path::Path) -> Self {
            let old = std::env::var("RUSTCODE_HOME").ok();
            std::env::set_var("RUSTCODE_HOME", dir);
            HomeGuard(old)
        }
    }
    impl Drop for HomeGuard {
        fn drop(&mut self) {
            match &self.0 {
                Some(v) => std::env::set_var("RUSTCODE_HOME", v),
                None => std::env::remove_var("RUSTCODE_HOME"),
            }
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn add_persists_the_sender_allowlist_and_clear_writes_mean_anyone() {
        let tmp = tempfile::tempdir().unwrap();
        let _home = HomeGuard::new(tmp.path());
        let project = std::env::current_dir().unwrap();
        let project = project.to_string_lossy();
        let credentials = dingtalk_setup_credentials("DINGTALK_CLIENT_SECRET");

        // First add: restricted to two senders.
        let code = handle_im_add(
            "dingtalk",
            &project,
            &credentials,
            Some(vec!["u1".into(), "u2".into()]),
        )
        .await
        .unwrap();
        assert_eq!(code, 0);
        let saved = load_config().unwrap();
        assert_eq!(
            saved.im.channels[0].allow_senders,
            vec!["u1".to_string(), "u2".to_string()]
        );

        // Upsert with no allowlist clears it back to "anyone" (replacement
        // semantics, same as the credential fields).
        let code = handle_im_add("dingtalk", &project, &credentials, None)
            .await
            .unwrap();
        assert_eq!(code, 0);
        let saved = load_config().unwrap();
        assert!(saved.im.channels[0].allow_senders.is_empty());
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
