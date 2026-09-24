//! Binary-side implementation of the IM agent runner.
//!
//! Lives in the **binary** rather than the `im` lib module because the pieces it
//! needs -- [`crate::spawn_native_cli_runtime`] and [`crate::run_native_headless`]
//! -- are `pub(crate)` inside this crate's binary target, and a lib module cannot
//! reach them. The lib owns the transport, chunking and session-continuity
//! logic; this file owns the process-level wiring.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use rustcode::im::{AgentRunner, AgentTurn, ApprovalPort, ImError};

/// Runs one agent turn in-process by driving the same headless bootstrap the
/// `-p` and `schedule run` paths use, so an IM turn behaves like any other
/// non-interactive turn.
pub struct CliAgentRunner;

/// Load the user's config. A missing/broken file falls back to defaults rather
/// than failing the turn: the very next thing that happens is a provider
/// resolution, which reports a missing provider far more usefully.
fn load_config() -> rustcode_config::config::Config {
    let path = rustcode_config::config::Config::default_path();
    if path.exists() {
        rustcode_config::config::Config::load(&path).unwrap_or_default()
    } else {
        rustcode_config::config::Config::default()
    }
}

#[async_trait]
impl AgentRunner for CliAgentRunner {
    async fn run_turn(
        &self,
        project: &Path,
        session_id: Option<&str>,
        prompt: &str,
        port: &mut dyn ApprovalPort,
    ) -> Result<AgentTurn, ImError> {
        use rustcode_capabilities::session::manager::SessionOrigin;
        use rustcode_capabilities::session::SessionManager;
        use rustcode_coding::ProviderBootstrap;

        let config = load_config();
        let cwd: PathBuf = project.to_path_buf();

        // IM turns never take a full bypass: `dangerously_skip_permissions=false`
        // and `interactive=false`, so the approval path stays fail-closed.
        let runtime_cfg = crate::runtime_config_from(
            &config, &cwd, None, false, // dangerously_skip_permissions
            false, // interactive
        );

        // `ProviderBootstrap::Required` mirrors `-p` / `schedule run`: fail fast
        // and legibly when no provider is configured, instead of "succeeding"
        // with an empty base URL and failing later inside reqwest.
        let (runtime, agent, _continued) = crate::spawn_native_cli_runtime(
            &runtime_cfg,
            session_id.map(|s| s.to_string()),
            ProviderBootstrap::Required,
            false, // ephemeral
            false, // no_tools
            false, // fork_on_session_in_use
            false, // round_cap_checkpoint (no interactive UI to answer it)
            rustcode_coding::RuntimeMode::Build,
        )
        .await
        .map_err(|e| ImError::Transport(format!("could not start the agent runtime: {e:#}")))?;

        // The session id is what gives the chat continuity; a runtime that
        // reports none cannot be resumed later, so surface that immediately
        // rather than writing a binding that can never continue.
        let session = runtime
            .session
            .as_ref()
            .map(|s| s.id.clone())
            .ok_or_else(|| ImError::Transport("runtime started without a session id".into()))?;

        // Tag the session so a chat-driven conversation is distinguishable from
        // one the user typed (and can be filtered in pickers later).
        let manager = SessionManager::for_project(&agent.working_dir);
        let _ = manager.update_meta(&session, |m| {
            m.origin = SessionOrigin::Im;
        });

        // `accept_edits` is the highest mode an unattended-but-supervised turn
        // may take. `auto` is deliberately not reachable from IM: the user is
        // not watching every step, so destructive calls must still be gated.
        runtime
            .handle
            .set_mode(rustcode_coding::RuntimeMode::AcceptEdits)
            .await
            .map_err(|e| ImError::Transport(format!("could not set the turn mode: {e}")))?;

        let provider_name = runtime_cfg.provider_name.clone();
        let model_name = runtime_cfg.model.clone();

        // `capture = true` accumulates the whole answer, which is what an IM
        // reply needs (there is no byte stream to consume on the chat side).
        // `strict_unattended = false`: a human *is* behind this channel, just
        // asynchronously. The approval port IS that human's voice: escalated
        // tool calls are rendered into the chat and answered there (P-IM2),
        // with the timeout fail-closing to Deny.
        let (exit_code, captured) = crate::run_native_headless(
            config.notifications.clone(),
            runtime,
            prompt.to_string(),
            provider_name,
            model_name,
            false, // verbose
            crate::HeadlessOutputFormat::Text,
            true, // capture
            cwd,
            false, // skip_permissions
            false, // is_admin
            false, // strict_unattended
            Some(port),
        )
        .await
        .map_err(|e| ImError::Transport(format!("the turn failed: {e:#}")))?;

        if exit_code != 0 {
            // Exit code 2 means at least one escalated call was denied (by the
            // user's explicit reply, or by the approval timeout). The agent has
            // already produced its explanation -- deliver THAT to the chat
            // instead of a bare error, so the user learns what was refused and
            // why rather than staring at a failure code.
            let explanation = captured.unwrap_or_default();
            if exit_code == 2 && !explanation.trim().is_empty() {
                return Ok(AgentTurn {
                    text: explanation,
                    session_id: session,
                });
            }
            return Err(ImError::Transport(format!(
                "the turn ended with exit code {exit_code}"
            )));
        }

        Ok(AgentTurn {
            text: captured.unwrap_or_default(),
            session_id: session,
        })
    }
}

/// Run the IM channel loop for one configured channel.
///
/// Resolves the channel from config (platform + project filter), builds the
/// adapter, and serves until the transport closes. Returns the process exit code
/// so the subcommand can surface failures the way `schedule run` does.
pub async fn run_im_command(
    platform_filter: Option<&str>,
    project_filter: Option<&str>,
) -> anyhow::Result<i32> {
    use rustcode::im::{dingtalk::DingTalkAdapter, resolve_project, serve_channel, RecentMessages};

    // Fail-closed on the master switch: an operator turning channels off in a
    // container must not have a stale shell still serving them.
    let config_path = rustcode_config::config::Config::default_path();
    let config = if config_path.exists() {
        rustcode_config::config::Config::load(&config_path).unwrap_or_default()
    } else {
        rustcode_config::config::Config::default()
    };

    if !rustcode_config::config::im::im_enabled_from(Some(&config.im)) {
        anyhow::bail!(
            "IM channels are disabled. Enable them with `[im] enabled = true` in {} \
             or set {} = 1.",
            config_path.display(),
            rustcode_config::endpoints::IM_ENABLED_ENV
        );
    }

    // Config diagnostics (missing credentials, duplicate channels, ...) are
    // surfaced at load time; refuse to start on a broken channel rather than
    // half-serving it.
    let problems = config.im.validate();
    if !problems.is_empty() {
        for problem in &problems {
            eprintln!("[!] {problem}");
        }
        anyhow::bail!("IM channel configuration is invalid; fix the entries above");
    }

    // Pick the channel to serve. Filters are optional so a single-channel setup
    // needs no flags.
    let channel = config
        .im
        .channels
        .iter()
        .find(|c| {
            if !rustcode_config::config::im::channel_is_active(&config.im, c) {
                return false;
            }
            if let Some(want) = platform_filter {
                if !c.platform.eq_ignore_ascii_case(want) {
                    return false;
                }
            }
            if let Some(want) = project_filter {
                if c.project != want {
                    return false;
                }
            }
            true
        })
        .ok_or_else(|| {
            anyhow::anyhow!(
                "no active IM channel matches (platform={:?}, project={:?})",
                platform_filter,
                project_filter
            )
        })?;

    let platform = channel.parsed_platform().ok_or_else(|| {
        anyhow::anyhow!(
            "channel has an unrecognized platform `{}`",
            channel.platform
        )
    })?;
    let project = resolve_project(&channel.project).map_err(|e| anyhow::anyhow!("{e}"))?;

    match platform {
        rustcode_config::config::im::ImPlatform::Dingtalk => {
            let client_id = channel
                .credential("client_id")
                .ok_or_else(|| anyhow::anyhow!("channel is missing `client_id`"))?;
            let client_secret = channel
                .credential("client_secret")
                .ok_or_else(|| anyhow::anyhow!("channel is missing `client_secret`"))?;

            let mut adapter = DingTalkAdapter::new(client_id, client_secret);
            if let Ok(gateway) = std::env::var("RUSTCODE_DINGTALK_GATEWAY") {
                if !gateway.trim().is_empty() {
                    adapter = adapter.with_gateway(gateway);
                }
            }

            println!(
                "[*] IM channel `dingtalk` serving project {} (long connection, no public endpoint needed)",
                project.display()
            );

            let mut dedupe = RecentMessages::new(1024);
            loop {
                match serve_channel(&mut adapter, &CliAgentRunner, &project, &mut dedupe).await {
                    // A clean stream end is a reconnect, not a failure: the
                    // platform rotates connections, and exiting here would leave
                    // the channel dead until an operator noticed.
                    Ok(()) => {
                        eprintln!("[*] IM stream closed; reconnecting");
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                    Err(error) => {
                        eprintln!("[!] IM stream error: {error}; reconnecting");
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    }
                }
            }
        }
        other => {
            anyhow::bail!(
                "platform `{}` is not implemented yet (only `dingtalk` Stream mode is)",
                other.as_str()
            )
        }
    }
}
