use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex as StdMutex, OnceLock};

use rustcode_coding::{
    CodingRuntime, CodingRuntimeConfig, DriverCommand, RuntimeMode, RuntimePhase, UserInput,
};
use rustcode_kernel::message::SessionSnapshot;
use tokio::sync::Mutex;

use crate::live_hub::{HubError, LiveBinding, LiveJoin, LiveRuntimeControl, LiveViewHub};

static HUB: OnceLock<Arc<LiveViewHub>> = OnceLock::new();
static EMBEDDED_BINDING: StdMutex<Option<LiveBinding>> = StdMutex::new(None);
static HEADLESS: OnceLock<Mutex<Option<HeadlessRuntime>>> = OnceLock::new();
static REMOTE_COMMAND: StdMutex<Option<tokio::sync::mpsc::UnboundedSender<String>>> =
    StdMutex::new(None);

struct HeadlessRuntime {
    binding: LiveBinding,
    handle: rustcode_coding::CodingRuntimeHandle,
}

pub(crate) fn hub() -> &'static Arc<LiveViewHub> {
    HUB.get_or_init(|| Arc::new(LiveViewHub::new()))
}

fn headless() -> &'static Mutex<Option<HeadlessRuntime>> {
    HEADLESS.get_or_init(|| Mutex::new(None))
}

pub fn register_embedded_runtime(
    session_id: String,
    working_dir: PathBuf,
    provider: String,
    provider_fingerprint: String,
    snapshot: SessionSnapshot,
    control: Arc<dyn LiveRuntimeControl>,
) -> Result<LiveBinding, HubError> {
    let headless_owner = headless()
        .try_lock()
        .map_err(|_| HubError::RuntimeUnavailable)?;
    if headless_owner.is_some() {
        return Err(HubError::RuntimeUnavailable);
    }
    let binding = hub().bind_with_provider(
        session_id,
        working_dir,
        provider,
        provider_fingerprint,
        snapshot,
        control,
    )?;
    *EMBEDDED_BINDING
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = Some(binding.clone());
    // Seed the daemon's project state to the shared TUI's working dir so the webui footer
    // + session list match it. A no-op if the server hasn't started yet (DAEMON_PROJECT is
    // None) -- that case is covered by `init_project_state` reading the embedded binding at
    // startup; this call handles an ALREADY-running (persistent) daemon, where the embed
    // happens after init. `/cd` keeps it current afterward via the same `live_set_working_dir`.
    crate::live_set_working_dir(binding.working_dir.clone());
    Ok(binding)
}

pub fn embedded_binding() -> Option<LiveBinding> {
    EMBEDDED_BINDING
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clone()
}

pub fn unregister_embedded_runtime(binding: &LiveBinding) -> Result<(), HubError> {
    hub().unbind(binding)?;
    let mut embedded = EMBEDDED_BINDING
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    if embedded
        .as_ref()
        .is_some_and(|current| current.id == binding.id)
    {
        *embedded = None;
        *REMOTE_COMMAND
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = None;
    }
    Ok(())
}

pub fn register_remote_command_sink() -> tokio::sync::mpsc::UnboundedReceiver<String> {
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
    *REMOTE_COMMAND
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = Some(sender);
    receiver
}

pub fn send_remote_command(command: String) -> bool {
    REMOTE_COMMAND
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .as_ref()
        .is_some_and(|sender| sender.send(command).is_ok())
}

pub fn publish(
    binding: &LiveBinding,
    event: rustcode_coding::SequencedRuntimeEvent,
) -> Result<(), HubError> {
    hub().publish(binding, event)
}

pub fn publish_unsequenced(
    binding: &LiveBinding,
    event: rustcode_coding::CodingRuntimeEvent,
) -> Result<(), HubError> {
    hub().publish_unsequenced(binding, event)
}

pub fn seed_goal_progress(
    binding: &LiveBinding,
    progress: rustcode_coding::GoalProgress,
) -> Result<(), HubError> {
    hub().seed_goal_progress(binding, progress)
}

pub fn join() -> Result<LiveJoin, HubError> {
    hub().join()
}

pub fn join_for_provider(expected_session_id: Option<&str>) -> Result<LiveJoin, HubError> {
    hub().join_for_provider(expected_session_id)
}

pub fn binding() -> Result<LiveBinding, HubError> {
    hub().binding()
}

pub fn submit(input: UserInput) -> Result<(), HubError> {
    hub().submit(input)
}

pub async fn submit_confirmed(
    input: UserInput,
) -> Result<rustcode_coding::SubmitReceipt, HubError> {
    hub().submit_confirmed(input).await
}

/// Submit `runtime_input` to the model while echoing `echo_input` to the live view
/// (see [`crate::live_hub::LiveViewHub::submit_confirmed_with_echo`]). Used by the
/// webui image path so the VL caption feeds the model but the user's original
/// message + image is what displays.
pub async fn submit_confirmed_with_echo(
    runtime_input: UserInput,
    echo_input: UserInput,
    client_input_id: Option<String>,
) -> Result<rustcode_coding::SubmitReceipt, HubError> {
    hub()
        .submit_confirmed_with_echo(runtime_input, echo_input, client_input_id)
        .await
}

pub fn accept_local_input(input: UserInput) -> Result<(), HubError> {
    hub().accept_local_input(input)
}

pub fn respond(
    id: rustcode_kernel::event::RequestId,
    value: serde_json::Value,
) -> Result<(), HubError> {
    hub().respond(id, value)
}

pub fn respond_pending_kind(kind: &str, value: serde_json::Value) -> Result<u64, HubError> {
    hub().respond_pending_kind(kind, value)
}

pub async fn respond_confirmed(
    id: rustcode_kernel::event::RequestId,
    value: serde_json::Value,
) -> Result<(), HubError> {
    hub().respond_confirmed(id, value).await
}

pub async fn resolve_policy_intervention(
    intervention_id: u64,
    action: rustcode_kernel::event::PolicyRecoveryAction,
) -> Result<(), HubError> {
    hub()
        .resolve_policy_intervention(intervention_id, action)
        .await
}

pub async fn respond_pending_kind_confirmed(
    kind: &str,
    value: serde_json::Value,
) -> Result<u64, HubError> {
    hub().respond_pending_kind_confirmed(kind, value).await
}

pub fn cancel() -> Result<(), HubError> {
    hub().cancel()
}

pub async fn cancel_confirmed() -> Result<(), HubError> {
    hub().cancel_confirmed().await
}

pub fn dispatch(command: DriverCommand) -> Result<(), HubError> {
    hub().dispatch(command)
}

pub async fn set_mode(mode: RuntimeMode) -> Result<(), HubError> {
    hub().set_mode(mode).await
}

pub async fn reload_provider(
    expected: &LiveBinding,
    next: rustcode_coding::CodingAgentConfig,
    provider_fingerprint: String,
) -> Result<rustcode_coding::RuntimeGeneration, HubError> {
    hub()
        .reload_provider(expected, next, provider_fingerprint)
        .await
}

pub fn provider_fingerprint(
    config: &rustcode_config::config::Config,
    provider_name: &str,
) -> Result<String, String> {
    use sha2::{Digest, Sha256};

    if !config.selection_exists(provider_name) {
        return Err(format!("provider {provider_name:?} not found"));
    }
    let mut normalized = config.clone();
    normalized.default_provider = provider_name.to_string();
    // Serialize through Value so map keys are canonicalized before hashing;
    // Config contains HashMaps whose iteration order differs across processes.
    let canonical = serde_json::to_value(&normalized)
        .map_err(|error| format!("serialize provider configuration failed: {error}"))?;
    let bytes = serde_json::to_vec(&canonical)
        .map_err(|error| format!("serialize provider configuration failed: {error}"))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

pub async fn resume_session(
    session_id: String,
) -> Result<rustcode_coding::SessionChanged, HubError> {
    let binding = hub().binding()?;
    if binding.session_id == session_id {
        return Ok(rustcode_coding::SessionChanged {
            generation: rustcode_coding::RuntimeGeneration(binding.generation),
            session_id: Some(binding.session_id),
            working_dir: binding.working_dir,
        });
    }
    let project_bucket =
        rustcode_capabilities::session::SessionManager::project_hash(&binding.working_dir);
    let prepared = match crate::legacy_convert::prepare_catalog_session_resume_in_project(
        &project_bucket,
        &session_id,
    ) {
        Ok(Some(prepared)) => prepared,
        _ => crate::legacy_convert::prepare_catalog_session_resume_any_project(&session_id)
            .map_err(|error| HubError::RuntimeRejected(error.to_string()))?
            .ok_or_else(|| {
                HubError::RuntimeRejected(format!("session {session_id:?} not found in catalog"))
            })?,
    };
    let target_dir = PathBuf::from(&prepared.view.meta.working_dir);
    hub()
        .resume_session_with_lease(session_id, target_dir, prepared.lease)
        .await
}

/// Move the bound runtime to a fresh staged session. This is the only safe way
/// for the daemon to release the current idle session's lease before deleting
/// that session from disk.
pub async fn fresh_session(
    expected: &LiveBinding,
) -> Result<crate::live_hub::FreshSessionOutcome, HubError> {
    hub().fresh_session(expected).await
}

pub async fn change_directory(
    working_dir: PathBuf,
) -> Result<rustcode_coding::SessionChanged, HubError> {
    hub().change_directory(working_dir).await
}

pub async fn reload_capabilities() -> Result<rustcode_coding::SessionChanged, HubError> {
    hub().reload_capabilities().await
}

pub fn publish_command_output(text: String) -> Result<(), HubError> {
    hub().publish_command_output(text)
}

pub fn replace_snapshot(
    binding: &LiveBinding,
    session_id: String,
    working_dir: PathBuf,
    snapshot: SessionSnapshot,
) -> Result<LiveBinding, HubError> {
    let next = hub().replace_snapshot(binding, session_id, working_dir, snapshot)?;
    let mut embedded = EMBEDDED_BINDING
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    if embedded
        .as_ref()
        .is_some_and(|current| current.id == binding.id)
    {
        *embedded = Some(next.clone());
    }
    Ok(next)
}

pub fn commit_runtime_snapshot(
    binding: &LiveBinding,
    session_id: String,
    working_dir: PathBuf,
    snapshot: SessionSnapshot,
) -> Result<LiveBinding, HubError> {
    let next = hub().commit_runtime_snapshot(binding, session_id, working_dir, snapshot)?;
    let mut embedded = EMBEDDED_BINDING
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    if embedded
        .as_ref()
        .is_some_and(|current| current.id == binding.id)
    {
        *embedded = Some(next.clone());
    }
    Ok(next)
}

fn load_snapshot(working_dir: &Path, session_id: &str) -> Result<SessionSnapshot, String> {
    let bucket = rustcode_capabilities::session::SessionManager::project_hash(working_dir);
    crate::legacy_convert::load_catalog_session_view_in_project(&bucket, session_id)
        .map_err(|error| error.to_string())?
        .map(|session| session.snapshot)
        .ok_or_else(|| format!("session {session_id:?} not found"))
}

async fn bind_after_mcp_ready<T, E>(
    readiness: impl std::future::Future<Output = Result<(), E>>,
    bind: impl FnOnce() -> Result<T, String>,
) -> Result<T, String>
where
    E: std::fmt::Debug,
{
    readiness
        .await
        .map_err(|error| format!("MCP readiness wait failed: {error:?}"))?;
    bind()
}

pub async fn ensure_headless_runtime(
    working_dir: PathBuf,
    provider_name: String,
    mode: RuntimeMode,
    requested_session_id: Option<String>,
) -> Result<LiveJoin, String> {
    if let Some(binding) = embedded_binding() {
        if requested_session_id
            .as_deref()
            .is_some_and(|requested| requested != binding.session_id)
        {
            return Err(format!(
                "embedded runtime is bound to session {:?}, requested {:?}",
                binding.session_id,
                requested_session_id.as_deref().unwrap_or_default()
            ));
        }
        return join().map_err(|error| format!("live hub join failed: {error:?}"));
    }

    let mut owner = headless().lock().await;
    let can_reuse = owner.is_some()
        && join().is_ok_and(|current| {
            current.binding.working_dir == working_dir
                && requested_session_id
                    .as_deref()
                    .is_none_or(|requested| requested == current.binding.session_id)
        });
    if can_reuse {
        return join().map_err(|error| format!("live hub join failed: {error:?}"));
    }

    if let Some(old) = owner.take() {
        if matches!(
            old.handle.status().phase,
            RuntimePhase::InTurn | RuntimePhase::WaitingApproval | RuntimePhase::Reconfiguring
        ) {
            *owner = Some(old);
            return Err("cannot replace an active live runtime".into());
        }
        old.handle
            .shutdown()
            .await
            .map_err(|_| "failed to stop previous live runtime".to_string())?;
        let _ = hub().unbind(&old.binding);
    }

    let config =
        rustcode_config::config::Config::load(&rustcode_config::config::Config::default_path())
            .map_err(|error| error.to_string())?;
    if !config.selection_exists(&provider_name) {
        return Err(format!("provider {provider_name:?} not found"));
    }
    let provider_fingerprint = provider_fingerprint(&config, &provider_name)?;
    let runtime_config: CodingRuntimeConfig =
        crate::live_api::live_runtime_config(&config, &provider_name, &working_dir);
    let (session_mode, initial_snapshot) = match requested_session_id {
        Some(id) => {
            let snapshot = load_snapshot(&working_dir, &id)?;
            (
                rustcode_coding::SessionMode::ExternalSnapshot {
                    id,
                    snapshot: snapshot.clone(),
                },
                snapshot,
            )
        }
        None => (
            rustcode_coding::SessionMode::Fresh,
            SessionSnapshot::new(Vec::new()),
        ),
    };
    let (runtime, _) = crate::start_native_runtime_with_session(runtime_config, session_mode)
        .await
        .map_err(|error| error.to_string())?;
    let CodingRuntime {
        handle,
        mut events,
        task,
        session,
        ..
    } = runtime;
    handle
        .set_mode(mode)
        .await
        .map_err(|error| format!("failed to set live mode: {error}"))?;

    // Wait for initial MCP tools to be published to the mounted kernel catalog
    // before the first turn. Without this, a headless
    // runtime created by `rustcode.exe webui` (which has no pre-existing
    // CodingRuntime from the TUI) would start its first turn before background
    // MCP connections complete, making MCP tools invisible to the agent even
    // though `/mcp/status` shows them as connected.
    // Timeout prevents a stalled MCP server from blocking the first message.
    let session_id = session
        .map(|session| session.id)
        .ok_or_else(|| "live runtime started without a persistent session".to_string())?;
    let binding = bind_after_mcp_ready(
        handle.wait_mcp_ready(rustcode_capabilities::mcp::CONNECT_TIMEOUT),
        || {
            hub()
                .bind_with_provider(
                    session_id.clone(),
                    working_dir.clone(),
                    provider_name,
                    provider_fingerprint,
                    initial_snapshot,
                    Arc::new(handle.clone()),
                )
                .map_err(|error| format!("live hub bind failed: {error:?}"))
        },
    )
    .await?;
    let event_binding = binding.clone();
    let event_handle = handle.clone();
    tokio::spawn(async move {
        while let Some(event) = events.recv().await {
            let session_change = match &event.event {
                rustcode_coding::CodingRuntimeEvent::SessionChanged(changed) => {
                    Some((changed.session_id.clone(), changed.working_dir.clone()))
                }
                _ => None,
            };
            match hub().publish(&event_binding, event) {
                Ok(()) => {}
                Err(HubError::StaleEvent) => {
                    tracing::warn!("discarded stale live runtime event");
                    continue;
                }
                Err(error) => {
                    tracing::warn!("stopping live event forwarding: {error:?}");
                    break;
                }
            }
            if let Some((Some(session_id), working_dir)) = session_change {
                match event_handle.snapshot().await {
                    Ok(snapshot) => {
                        if let Err(error) = hub().commit_runtime_snapshot(
                            &event_binding,
                            session_id,
                            working_dir,
                            snapshot.as_ref().clone(),
                        ) {
                            tracing::warn!("live session snapshot commit failed: {error:?}");
                            break;
                        }
                    }
                    Err(error) => {
                        tracing::warn!("live runtime session snapshot unavailable: {error}");
                    }
                }
            }
        }
        let _ = task.await;
    });
    *owner = Some(HeadlessRuntime { binding, handle });
    drop(owner);
    join().map_err(|error| format!("live hub join failed: {error:?}"))
}

// --- Multi-task support ---
//
// Each task runs in its own `LiveViewHub` + `CodingRuntime`, fully isolated
// from the interactive `/live` binding. This lets external programs submit
// one-shot prompts and stream progress via `POST /live/tasks` without
// interfering with the shared interactive session.

static TASK_RUNTIMES: OnceLock<Mutex<HashMap<String, TaskRuntime>>> = OnceLock::new();

struct TaskRuntime {
    hub: Arc<LiveViewHub>,
    binding: LiveBinding,
    handle: rustcode_coding::CodingRuntimeHandle,
    event_forwarder: tokio::task::JoinHandle<()>,
    created_at: i64,
    prompt: String,
    session_id: String,
    working_dir: PathBuf,
    provider: String,
}

#[derive(serde::Serialize, Clone)]
pub struct TaskInfo {
    pub task_id: String,
    pub session_id: String,
    pub working_dir: String,
    pub provider: String,
    pub status: String,
    pub created_at: i64,
    pub prompt: String,
}

fn task_runtimes() -> &'static Mutex<HashMap<String, TaskRuntime>> {
    TASK_RUNTIMES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn generate_task_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("task-{now}")
}

fn task_status(handle: &rustcode_coding::CodingRuntimeHandle) -> String {
    match handle.status().phase {
        RuntimePhase::Ready => "ready".into(),
        RuntimePhase::InTurn => "running".into(),
        RuntimePhase::WaitingApproval => "waiting_approval".into(),
        RuntimePhase::Reconfiguring => "reconfiguring".into(),
        RuntimePhase::ShuttingDown => "shutting_down".into(),
        RuntimePhase::Stopped => "stopped".into(),
        RuntimePhase::Failed => "failed".into(),
        RuntimePhase::AwaitingProvider => "awaiting_provider".into(),
    }
}

/// Create an isolated background task. The prompt is auto-submitted in
/// `Build` mode (auto-approve safe operations). Progress is streamed via
/// `GET /live/tasks/{id}/stream`.
pub async fn create_task(
    working_dir: PathBuf,
    provider_name: String,
    prompt: String,
    session_id: Option<String>,
) -> Result<TaskInfo, String> {
    let config =
        rustcode_config::config::Config::load(&rustcode_config::config::Config::default_path())
            .map_err(|error| error.to_string())?;
    if !config.selection_exists(&provider_name) {
        return Err(format!("provider {provider_name:?} not found"));
    }
    let provider_fingerprint = provider_fingerprint(&config, &provider_name)?;
    let runtime_config =
        crate::live_api::live_runtime_config(&config, &provider_name, &working_dir);
    let (session_mode, initial_snapshot) = match &session_id {
        Some(id) => {
            let snapshot = load_snapshot(&working_dir, id)?;
            (
                rustcode_coding::SessionMode::ExternalSnapshot {
                    id: id.clone(),
                    snapshot: snapshot.clone(),
                },
                snapshot,
            )
        }
        None => (
            rustcode_coding::SessionMode::Fresh,
            SessionSnapshot::new(Vec::new()),
        ),
    };
    let (runtime, _) = crate::start_native_runtime_with_session(runtime_config, session_mode)
        .await
        .map_err(|error| error.to_string())?;
    let CodingRuntime {
        handle,
        mut events,
        task,
        session,
        ..
    } = runtime;
    handle
        .set_mode(RuntimeMode::Build)
        .await
        .map_err(|error| format!("failed to set task mode: {error}"))?;
    let resolved_session_id = session
        .map(|session| session.id)
        .ok_or_else(|| "task runtime started without a persistent session".to_string())?;
    let task_hub = Arc::new(LiveViewHub::new());
    let binding = bind_after_mcp_ready(
        handle.wait_mcp_ready(rustcode_capabilities::mcp::CONNECT_TIMEOUT),
        || {
            task_hub
                .bind_with_provider(
                    resolved_session_id.clone(),
                    working_dir.clone(),
                    provider_name.clone(),
                    provider_fingerprint.clone(),
                    initial_snapshot,
                    Arc::new(handle.clone()),
                )
                .map_err(|error| format!("task hub bind failed: {error:?}"))
        },
    )
    .await?;
    let event_binding = binding.clone();
    let event_hub = Arc::clone(&task_hub);
    let event_handle = handle.clone();
    let event_forwarder = tokio::spawn(async move {
        while let Some(event) = events.recv().await {
            let session_change = match &event.event {
                rustcode_coding::CodingRuntimeEvent::SessionChanged(changed) => {
                    Some((changed.session_id.clone(), changed.working_dir.clone()))
                }
                _ => None,
            };
            match event_hub.publish(&event_binding, event) {
                Ok(()) => {}
                Err(HubError::StaleEvent) => {
                    continue;
                }
                Err(error) => {
                    tracing::warn!("stopping task event forwarding: {error:?}");
                    break;
                }
            }
            if let Some((Some(sid), wdir)) = session_change {
                match event_handle.snapshot().await {
                    Ok(snapshot) => {
                        if let Err(error) = event_hub.commit_runtime_snapshot(
                            &event_binding,
                            sid,
                            wdir,
                            snapshot.as_ref().clone(),
                        ) {
                            tracing::warn!("task snapshot commit failed: {error:?}");
                            break;
                        }
                    }
                    Err(error) => {
                        tracing::warn!("task snapshot unavailable: {error}");
                    }
                }
            }
        }
        let _ = task.await;
    });
    let input = UserInput {
        text: prompt.clone(),
        images: Vec::new(),
    };
    task_hub
        .submit_confirmed(input)
        .await
        .map_err(|error| format!("task prompt submission failed: {error:?}"))?;
    let task_id = generate_task_id();
    let created_at = rustcode_capabilities::session::now_ms();
    let info = TaskInfo {
        task_id: task_id.clone(),
        session_id: resolved_session_id.clone(),
        working_dir: working_dir.to_string_lossy().to_string(),
        provider: provider_name.clone(),
        status: task_status(&handle),
        created_at,
        prompt: prompt.clone(),
    };
    task_runtimes().lock().await.insert(
        task_id.clone(),
        TaskRuntime {
            hub: task_hub,
            binding,
            handle,
            event_forwarder,
            created_at,
            prompt,
            session_id: resolved_session_id,
            working_dir,
            provider: provider_name,
        },
    );
    Ok(info)
}

pub async fn list_tasks() -> Vec<TaskInfo> {
    task_runtimes()
        .lock()
        .await
        .iter()
        .map(|(id, rt)| TaskInfo {
            task_id: id.clone(),
            session_id: rt.session_id.clone(),
            working_dir: rt.working_dir.to_string_lossy().to_string(),
            provider: rt.provider.clone(),
            status: task_status(&rt.handle),
            created_at: rt.created_at,
            prompt: rt.prompt.clone(),
        })
        .collect()
}

pub async fn stop_task(task_id: &str) -> Result<(), String> {
    let runtime = {
        let mut map = task_runtimes().lock().await;
        map.remove(task_id).ok_or("task not found")?
    };
    let _ = runtime.handle.shutdown().await;
    let _ = runtime.hub.unbind(&runtime.binding);
    let _ = runtime.event_forwarder.await;
    Ok(())
}

/// Acquire a `LiveJoin` for SSE streaming of a specific task's events.
pub async fn task_join(task_id: &str) -> Option<LiveJoin> {
    let hub = {
        let map = task_runtimes().lock().await;
        map.get(task_id).map(|rt| Arc::clone(&rt.hub))
    };
    hub?.join().ok()
}

#[cfg(test)]
mod tests {
    use super::bind_after_mcp_ready;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn headless_bind_waits_for_mcp_catalog_readiness() {
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel::<()>();
        let (waiting_tx, waiting_rx) = tokio::sync::oneshot::channel::<()>();
        let bound = Arc::new(AtomicBool::new(false));
        let bound_in_task = Arc::clone(&bound);

        let bind_task = tokio::spawn(async move {
            bind_after_mcp_ready(
                async move {
                    waiting_tx.send(()).unwrap();
                    ready_rx.await.expect("readiness sender must stay alive");
                    Ok::<(), &'static str>(())
                },
                || {
                    bound_in_task.store(true, Ordering::Release);
                    Ok(())
                },
            )
            .await
        });

        waiting_rx.await.unwrap();
        assert!(
            !bound.load(Ordering::Acquire),
            "the live hub must remain unbound while MCP tools are unpublished"
        );

        ready_tx.send(()).unwrap();
        bind_task.await.unwrap().unwrap();
        assert!(
            bound.load(Ordering::Acquire),
            "the live hub should bind after MCP tools reach the catalog"
        );
    }
}
