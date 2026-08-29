use std::future::Future;

use rustcode_config::telemetry_legacy::SessionMode;

use crate::AppState;

/// Runs `f` (the handler body). Historically this wrapped the closure in a
/// telemetry `CurrentContext::scope` that set `mode`, `repo_origin` and a
/// per-request `session_id`. The reporting runtime (`rustcode-telemetry`) was
/// removed, so this is now a plain pass-through — it simply invokes the closure.
/// The first three arguments are retained for call-site stability but ignored.
pub async fn daemon_scope<F, Fut, R>(
    _state: &AppState,
    _session_id: Option<uuid::Uuid>,
    _mode: SessionMode,
    f: F,
) -> R
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = R>,
{
    f().await
}
