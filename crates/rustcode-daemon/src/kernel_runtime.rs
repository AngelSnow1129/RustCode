//! Daemon entry points for the unified native [`rustcode_coding::CodingRuntime`].

use rustcode_coding::config::CodingAgentConfig;
use rustcode_coding::parts::{PrepareOptions, SessionMode};
use rustcode_coding::runtime::CodingRuntimeEvent;
use rustcode_coding::CodingRuntimeConfig;
use tokio::sync::{mpsc, watch};

struct DaemonVlImagePreprocessor {
    working_dir: std::path::PathBuf,
}

#[async_trait::async_trait]
impl rustcode_coding::ImagePreprocessor for DaemonVlImagePreprocessor {
    async fn preprocess(
        &self,
        text: String,
        images: Vec<rustcode_coding::ImageContent>,
        supports_vision: bool,
        session_id: Option<String>,
    ) -> (
        rustcode_coding::UserInput,
        Option<rustcode_coding::VisionNotice>,
    ) {
        let config = match rustcode_config::config::Config::load(
            &rustcode_config::config::Config::default_path(),
        ) {
            Ok(config) => config,
            Err(_) => return (rustcode_coding::UserInput { text, images }, None),
        };
        crate::live_api::preprocess_image_input(
            &config,
            supports_vision,
            &self.working_dir,
            session_id.as_deref(),
            &text,
            &images,
        )
        .await
    }
}

fn daemon_image_preprocessor(
    cfg: &CodingRuntimeConfig,
    injected: Option<std::sync::Arc<dyn rustcode_coding::ImagePreprocessor>>,
) -> Option<std::sync::Arc<dyn rustcode_coding::ImagePreprocessor>> {
    injected.or_else(|| {
        Some(std::sync::Arc::new(DaemonVlImagePreprocessor {
            working_dir: cfg.working_dir.clone(),
        })
            as std::sync::Arc<dyn rustcode_coding::ImagePreprocessor>)
    })
}

/// Convert the shared runtime configuration into the native coding-agent config.
pub fn coding_config_from_runtime(cfg: &CodingRuntimeConfig) -> CodingAgentConfig {
    cfg.agent_config()
}

pub async fn start_native_runtime(
    cfg: CodingRuntimeConfig,
) -> Result<(rustcode_coding::CodingRuntime, CodingAgentConfig), rustcode_coding::RuntimeStartError>
{
    start_native_runtime_with_session(cfg, SessionMode::Fresh).await
}

pub async fn start_native_runtime_with_session(
    cfg: CodingRuntimeConfig,
    session: SessionMode,
) -> Result<(rustcode_coding::CodingRuntime, CodingAgentConfig), rustcode_coding::RuntimeStartError>
{
    start_native_runtime_with_session_bootstrap(
        cfg,
        session,
        rustcode_coding::ProviderBootstrap::Required,
        None,
    )
    .await
}

async fn start_native_runtime_with_session_bootstrap(
    cfg: CodingRuntimeConfig,
    session: SessionMode,
    bootstrap: rustcode_coding::ProviderBootstrap,
    image_preprocessor: Option<std::sync::Arc<dyn rustcode_coding::ImagePreprocessor>>,
) -> Result<(rustcode_coding::CodingRuntime, CodingAgentConfig), rustcode_coding::RuntimeStartError>
{
    let image_preprocessor = daemon_image_preprocessor(&cfg, image_preprocessor);
    let coding_cfg = coding_config_from_runtime(&cfg);
    let (session, imported_lease) = match session {
        SessionMode::Resume(id) => {
            let manager = rustcode_capabilities::session::SessionManager::for_project(
                &coding_cfg.working_dir,
            );
            let lease = manager.acquire_lease(&id).map_err(|error| {
                rustcode_coding::RuntimeStartError::Prepare(std::io::Error::from(error))
            })?;
            crate::legacy_convert::converge_session(&manager, &lease).map_err(|error| {
                rustcode_coding::RuntimeStartError::Prepare(std::io::Error::other(error))
            })?;
            (SessionMode::Resume(id), Some(lease))
        }
        SessionMode::ExternalSnapshot { id, snapshot } => {
            let manager = rustcode_capabilities::session::SessionManager::for_project(
                &coding_cfg.working_dir,
            );
            let lease = manager.acquire_lease(&id).map_err(|error| {
                rustcode_coding::RuntimeStartError::Prepare(std::io::Error::from(error))
            })?;
            let has_existing = [
                manager.meta_path(&id),
                manager.snapshot_path(&id),
                manager.legacy_path(&id),
            ]
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| {
                rustcode_coding::RuntimeStartError::Prepare(std::io::Error::from(error))
            })?
            .iter()
            .any(|path| path.exists());
            if has_existing {
                crate::legacy_convert::converge_session(&manager, &lease).map_err(|error| {
                    rustcode_coding::RuntimeStartError::Prepare(std::io::Error::other(error))
                })?;
            } else {
                let now = rustcode_capabilities::session::now_ms();
                let mut meta = rustcode_capabilities::session::SessionMeta::new(
                    &id,
                    coding_cfg.working_dir.to_string_lossy(),
                    now,
                );
                meta.owner = rustcode_capabilities::session::StorageOwner::Native;
                meta.message_count = u32::try_from(snapshot.messages.len()).map_err(|_| {
                    rustcode_coding::RuntimeStartError::Prepare(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "external snapshot has too many messages",
                    ))
                })?;
                manager
                    .commit_native_import(
                        &lease,
                        Some(&snapshot),
                        Some(&rustcode_capabilities::session::PresentationFile::default()),
                        &meta,
                    )
                    .map_err(|error| {
                        rustcode_coding::RuntimeStartError::Prepare(std::io::Error::from(error))
                    })?;
            }
            (SessionMode::Resume(id), Some(lease))
        }
        other => (other, None),
    };
    let prepare = PrepareOptions {
        subagents: rustcode_coding::SubagentPolicy::Enabled,
        request_user_input: true,
        session,
        tools: true,
        skill_dirs: None,
        plugin_skill_dirs: crate::gather_plugin_skill_dirs_for(&cfg.working_dir),
        mcp: cfg.mcp,
        extra_mcp_servers: Vec::new(),
        // External-agent subagents (Claude Code / Codex) from
        // `[[subagent.external]]` + the built-in parallel template
        // (explorer / builder / reviewer) when `subagent.parallel_template`
        // is on (default). Daemon runs are headless: never allow the
        // dangerous `bypass` mode (fail-closed).
        external_subagents: cfg
            .subagent_config
            .as_ref()
            .map(|c| rustcode_coding::parts::resolve_external_subagents(&c.subagent, false))
            .unwrap_or_default(),
        memory: true,
        web: true,
        review: true,
        rate_limit_source: None,
    };
    let start = rustcode_coding::CodingRuntimeStart {
        agent: coding_cfg.clone(),
        prepare,
        provider_factory: crate::coding_provider_factory(),
        plugin_hooks: crate::installed_plugin_hook_source(),
        // Normal daemon submits may preprocess in live_api before reaching the
        // runtime, while request_user_input responses enter through Respond.
        // Installing the same adapter here covers both without double-processing
        // inputs whose images were already stripped upstream.
        image_preprocessor,
        task_id: None,
    };
    let runtime = match imported_lease {
        Some(lease) => {
            rustcode_coding::CodingRuntime::start_with_session_lease(start, bootstrap, lease)
                .await?
        }
        None => rustcode_coding::CodingRuntime::start_with_bootstrap(start, bootstrap).await?,
    };
    Ok((runtime, coding_cfg))
}

/// Start a restored runtime asynchronously while immediately returning its native
/// command and event channels to the TUI session-switch path.
pub fn spawn_native_runtime_for_session_deferred(
    cfg: CodingRuntimeConfig,
    id: String,
    snapshot: rustcode_kernel::message::SessionSnapshot,
) -> (
    mpsc::UnboundedSender<rustcode_coding::DriverCommand>,
    mpsc::UnboundedReceiver<rustcode_coding::SequencedRuntimeEvent>,
    watch::Receiver<rustcode_coding::DeferredRuntimeState>,
) {
    spawn_native_runtime_for_session_deferred_with_preprocessor(cfg, id, snapshot, None)
}

/// Deferred restored-runtime constructor for drivers that own image preprocessing.
/// Daemon/live callers use [`spawn_native_runtime_for_session_deferred`]; the local
/// TUI injects its VL adapter so a session replacement preserves the initial runtime's
/// image-input behavior.
pub fn spawn_native_runtime_for_session_deferred_with_preprocessor(
    cfg: CodingRuntimeConfig,
    id: String,
    snapshot: rustcode_kernel::message::SessionSnapshot,
    image_preprocessor: Option<std::sync::Arc<dyn rustcode_coding::ImagePreprocessor>>,
) -> (
    mpsc::UnboundedSender<rustcode_coding::DriverCommand>,
    mpsc::UnboundedReceiver<rustcode_coding::SequencedRuntimeEvent>,
    watch::Receiver<rustcode_coding::DeferredRuntimeState>,
) {
    let (control_tx, mut control_rx) = mpsc::unbounded_channel();
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let (state_tx, state_rx) = watch::channel(rustcode_coding::DeferredRuntimeState::Starting);
    tokio::spawn(async move {
        let mut output_sequence = 0u64;
        #[allow(clippy::result_large_err)]
        let send_event =
            |event_tx: &mpsc::UnboundedSender<rustcode_coding::SequencedRuntimeEvent>,
             output_sequence: &mut u64,
             generation: u64,
             event: CodingRuntimeEvent| {
                let result = event_tx.send(rustcode_coding::SequencedRuntimeEvent {
                    generation,
                    sequence: *output_sequence,
                    event,
                });
                *output_sequence = output_sequence.wrapping_add(1);
                result
            };
        let bootstrap = if cfg.model.is_empty() {
            rustcode_coding::ProviderBootstrap::Unavailable(
                rustcode_coding::ProviderUnavailableReason::NotConfigured,
            )
        } else {
            rustcode_coding::ProviderBootstrap::RecoverAuthentication
        };
        let runtime = start_native_runtime_with_session_bootstrap(
            cfg,
            SessionMode::ExternalSnapshot { id, snapshot },
            bootstrap,
            image_preprocessor,
        )
        .await;
        let (runtime, _) = match runtime {
            Ok(runtime) => runtime,
            Err(error) => {
                let message = error.to_string();
                state_tx.send_replace(rustcode_coding::DeferredRuntimeState::Failed(
                    message.clone(),
                ));
                let _ = send_event(
                    &event_tx,
                    &mut output_sequence,
                    0,
                    CodingRuntimeEvent::Agent(rustcode_kernel::event::AgentEvent::Error {
                        message: message.clone(),
                        http_status: None,
                        code: None,
                        retryable: None,
                    }),
                );
                while let Some(control) = control_rx.recv().await {
                    if matches!(control, rustcode_coding::DriverCommand::Shutdown) {
                        break;
                    }
                    let _ = send_event(
                        &event_tx,
                        &mut output_sequence,
                        0,
                        CodingRuntimeEvent::Agent(rustcode_kernel::event::AgentEvent::Error {
                            message: format!("runtime unavailable: {message}"),
                            http_status: None,
                            code: None,
                            retryable: None,
                        }),
                    );
                }
                return;
            }
        };
        let rustcode_coding::CodingRuntime {
            handle,
            mut events,
            task,
            ..
        } = runtime;
        state_tx.send_replace(rustcode_coding::DeferredRuntimeState::Ready(handle.clone()));
        loop {
            tokio::select! {
                control = control_rx.recv() => {
                    let Some(control) = control else {
                        let _ = handle.shutdown().await;
                        break;
                    };
                    let event = match control {
                        rustcode_coding::DriverCommand::UndoToPrompt(nth) => Some(
                            CodingRuntimeEvent::UndoFinished(handle.undo_to_prompt(nth).await),
                        ),
                        rustcode_coding::DriverCommand::RefreshContextStats => Some(
                            CodingRuntimeEvent::ContextStatsRefreshed(handle.context_stats().await),
                        ),
                        rustcode_coding::DriverCommand::RestoreSnapshotCorrelated {
                            snapshot,
                            correlation_id,
                        } => {
                            let result = async {
                                handle.restore_snapshot(snapshot).await?;
                                handle.snapshot().await
                            }
                            .await;
                            Some(CodingRuntimeEvent::SnapshotRestoreFinished {
                                correlation_id,
                                result,
                            })
                        }
                        rustcode_coding::DriverCommand::ReloadProvider(next) => Some(
                            CodingRuntimeEvent::ProviderReloadFinished(
                                handle.reassemble_provider(next).await,
                            ),
                        ),
                        rustcode_coding::DriverCommand::DeactivateProvider(reason) => Some(
                            CodingRuntimeEvent::ProviderDeactivationFinished(
                                handle.deactivate_provider(reason).await,
                            ),
                        ),
                        control => {
                            if let Err(error) = handle.dispatch(control) {
                                let _ = send_event(
                                    &event_tx,
                                    &mut output_sequence,
                                    handle.status().generation,
                                    CodingRuntimeEvent::Agent(rustcode_kernel::event::AgentEvent::Error {
                                        message: error.to_string(),
                                        http_status: None,
                                        code: None,
                                        retryable: None,
                                    }),
                                );
                            }
                            None
                        }
                    };
                    if let Some(event) = event {
                        let _ = send_event(
                            &event_tx,
                            &mut output_sequence,
                            handle.status().generation,
                            event,
                        );
                    }
                }
                event = events.recv() => match event {
                    Some(event) => {
                        if send_event(
                            &event_tx,
                            &mut output_sequence,
                            event.generation,
                            event.event,
                        ).is_err() {
                            let _ = handle.shutdown().await;
                            break;
                        }
                    }
                    None => break,
                }
            }
        }
        let _ = task.await;
    });
    (control_tx, event_rx, state_rx)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct RecordingImagePreprocessor {
        called: std::sync::Arc<std::sync::atomic::AtomicBool>,
    }

    #[async_trait::async_trait]
    impl rustcode_coding::ImagePreprocessor for RecordingImagePreprocessor {
        async fn preprocess(
            &self,
            text: String,
            _images: Vec<rustcode_coding::ImageContent>,
            _supports_vision: bool,
            _session_id: Option<String>,
        ) -> (
            rustcode_coding::UserInput,
            Option<rustcode_coding::VisionNotice>,
        ) {
            self.called
                .store(true, std::sync::atomic::Ordering::Release);
            (
                rustcode_coding::UserInput {
                    text: format!("recognized: {text}"),
                    images: Vec::new(),
                },
                None,
            )
        }
    }

    struct ScopedHome {
        _lock: std::sync::MutexGuard<'static, ()>,
        previous: Option<std::ffi::OsString>,
        _dir: tempfile::TempDir,
    }

    impl ScopedHome {
        fn new() -> Self {
            let lock = crate::rustcode_home_test_lock()
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            let previous = std::env::var_os("RUSTCODE_HOME");
            let dir = tempfile::tempdir().unwrap();
            std::env::set_var("RUSTCODE_HOME", dir.path());
            Self {
                _lock: lock,
                previous,
                _dir: dir,
            }
        }
    }

    impl Drop for ScopedHome {
        fn drop(&mut self) {
            match self.previous.take() {
                Some(value) => std::env::set_var("RUSTCODE_HOME", value),
                None => std::env::remove_var("RUSTCODE_HOME"),
            }
        }
    }

    #[test]
    fn daemon_runtime_installs_default_image_preprocessor() {
        let home = ScopedHome::new();
        let config = rustcode_config::config::Config::default();
        let cfg = CodingRuntimeConfig::from_config(&config, home._dir.path(), None, false, true);

        assert!(daemon_image_preprocessor(&cfg, None).is_some());
    }

    #[tokio::test]
    async fn deferred_runtime_publishes_authoritative_awaiting_provider_handle() {
        let _home = ScopedHome::new();
        let working_dir = tempfile::tempdir().unwrap();
        let config = rustcode_config::config::Config::default();
        let cfg = CodingRuntimeConfig::from_config(&config, working_dir.path(), None, false, false);
        let (control_tx, mut event_rx, mut state_rx) = spawn_native_runtime_for_session_deferred(
            cfg,
            "deferred-test".into(),
            rustcode_kernel::message::SessionSnapshot::new(Vec::new()),
        );
        let _: &mut mpsc::UnboundedReceiver<rustcode_coding::SequencedRuntimeEvent> = &mut event_rx;

        tokio::time::timeout(std::time::Duration::from_secs(5), state_rx.changed())
            .await
            .unwrap()
            .unwrap();
        let state = state_rx.borrow().clone();
        let rustcode_coding::DeferredRuntimeState::Ready(handle) = state else {
            panic!("deferred runtime did not publish a ready handle");
        };
        assert_eq!(
            handle.status().phase,
            rustcode_coding::RuntimePhase::AwaitingProvider
        );

        control_tx
            .send(rustcode_coding::DriverCommand::Shutdown)
            .unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(5), control_tx.closed())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn deferred_tui_resume_keeps_image_preprocessor() {
        let _home = ScopedHome::new();
        let working_dir = tempfile::tempdir().unwrap();
        let config = rustcode_config::config::Config::default();
        let mut cfg =
            CodingRuntimeConfig::from_config(&config, working_dir.path(), None, false, false);
        cfg.provider_name = "main".into();
        cfg.api_key = "test".into();
        cfg.base_url = "http://127.0.0.1:9/v1".into();
        cfg.model = "glm-5.2".into();
        let called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let preprocessor = std::sync::Arc::new(RecordingImagePreprocessor {
            called: called.clone(),
        });
        let (control_tx, _event_rx, mut state_rx) =
            spawn_native_runtime_for_session_deferred_with_preprocessor(
                cfg,
                "deferred-image-test".into(),
                rustcode_kernel::message::SessionSnapshot::new(Vec::new()),
                Some(preprocessor),
            );

        tokio::time::timeout(std::time::Duration::from_secs(5), state_rx.changed())
            .await
            .unwrap()
            .unwrap();
        assert!(matches!(
            *state_rx.borrow(),
            rustcode_coding::DeferredRuntimeState::Ready(_)
        ));

        control_tx
            .send(rustcode_coding::DriverCommand::Submit(
                rustcode_coding::UserInput {
                    text: "inspect".into(),
                    images: vec![rustcode_coding::ImageContent {
                        media_type: "image/png".into(),
                        data: "AAAA".into(),
                    }],
                },
            ))
            .unwrap();

        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            while !called.load(std::sync::atomic::Ordering::Acquire) {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("deferred resumed runtime dropped the image preprocessor");

        control_tx
            .send(rustcode_coding::DriverCommand::Shutdown)
            .unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(5), control_tx.closed())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn deferred_runtime_reports_same_session_conflict() {
        let _home = ScopedHome::new();
        let working_dir = tempfile::tempdir().unwrap();
        let config = rustcode_config::config::Config::default();
        let cfg =
            || CodingRuntimeConfig::from_config(&config, working_dir.path(), None, false, false);
        let snapshot = || rustcode_kernel::message::SessionSnapshot::new(Vec::new());
        let (first_tx, _first_events, mut first_state) = spawn_native_runtime_for_session_deferred(
            cfg(),
            "same-deferred-session".into(),
            snapshot(),
        );
        tokio::time::timeout(std::time::Duration::from_secs(5), first_state.changed())
            .await
            .unwrap()
            .unwrap();
        assert!(matches!(
            *first_state.borrow(),
            rustcode_coding::DeferredRuntimeState::Ready(_)
        ));

        let (second_tx, _second_events, mut second_state) =
            spawn_native_runtime_for_session_deferred(
                cfg(),
                "same-deferred-session".into(),
                snapshot(),
            );
        tokio::time::timeout(std::time::Duration::from_secs(5), second_state.changed())
            .await
            .unwrap()
            .unwrap();
        assert!(matches!(
            &*second_state.borrow(),
            rustcode_coding::DeferredRuntimeState::Failed(message)
                if message.contains("already in use")
        ));

        first_tx
            .send(rustcode_coding::DriverCommand::Shutdown)
            .unwrap();
        second_tx
            .send(rustcode_coding::DriverCommand::Shutdown)
            .unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(5), first_tx.closed())
            .await
            .unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(5), second_tx.closed())
            .await
            .unwrap();
    }
}
