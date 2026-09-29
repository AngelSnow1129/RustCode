//! Conformance gates for the `session` capability: the `recall` tool and the
//! persistence/injection hooks must each satisfy the kernel seam contracts
//! (must-not-panic, bounded, meta-preserving, return-shape) — the same gates a
//! third-party provider/tool/hook is held to.
#![cfg(feature = "session")]

use std::sync::Arc;

use atomcode_capabilities::session::{
    RecallTool, SessionManager, SnapshotHook, StatusReminderHook,
};
use atomcode_capabilities::ProductDirs;
use atomcode_kernel::conformance;
use atomcode_kernel::hook::LifecycleHooks;
use atomcode_kernel::tool::Tool;

#[tokio::test]
async fn recall_tool_passes_kernel_tool_conformance() {
    // An empty temp sessions tree, handed in — nothing reads the environment.
    let home = tempfile::tempdir().unwrap();
    let tool: Arc<dyn Tool> = Arc::new(RecallTool::new(home.path().join("sessions")));
    conformance::tool::check(tool, &[r#"{"query":"oauth refresh"}"#, "{\"query\":\"\"}"])
        .await
        .assert_conformant();
}

#[tokio::test]
async fn snapshot_hook_passes_lifecycle_conformance() {
    let dir = tempfile::tempdir().unwrap();
    let mgr = Arc::new(SessionManager::with_root(dir.path()));
    let dirs = ProductDirs::new(dir.path().join("tree"), ".ours");
    let h: Arc<dyn LifecycleHooks> = Arc::new(SnapshotHook::new(mgr, "s1", "/proj", &dirs));
    conformance::hooks::check(h).await.assert_conformant();
}

#[tokio::test]
async fn status_reminder_hook_passes_lifecycle_conformance() {
    let h: Arc<dyn LifecycleHooks> = Arc::new(StatusReminderHook::new());
    conformance::hooks::check(h).await.assert_conformant();
}
