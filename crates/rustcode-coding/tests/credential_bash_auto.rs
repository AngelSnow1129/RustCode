//! Auto mode is absolute trust: the credential shell gate must ALLOW instead of
//! terminating the turn -- even under `[coding] shell_guard_policy = "strict"`.
//!
//! Driven through the FULL assembly (`prepare` + `assemble`) so the wiring from
//! `CodingParts::bypass_mode` into the gate is covered, not just the gate in
//! isolation. The test driver answers every approval with "allow", so the
//! credential gate is the only thing that can stop this call.

use std::sync::Arc;
use std::time::Duration;

use rustcode_coding::{assemble, prepare, CodingAgentConfig, PrepareOptions, SessionMode};
use rustcode_kernel::event::{AgentCommand, AgentEvent};
use rustcode_kernel::stream::StreamEvent;
use rustcode_kernel::testkit::RecordingProvider;
use rustcode_kernel::tool::ToolCall;

#[ctor::ctor]
fn _isolate_rustcode_home() {
    rustcode_kernel::test_support::isolate_home();
}

/// A credential-literal detection (real-looking value, not a synthetic test one).
const CRED_BASH: &str = r#"{"command":"echo 'Authorization: Bearer real-looking-token'"}"#;

/// Run one turn whose only tool call is a credential-shaped `bash`, with the
/// runtime in Auto mode (`auto_mode`) or not. Returns
/// `(tool_result_is_error, tool_result_content, saw_policy_intervention)`.
async fn run_credential_bash(auto_mode: bool) -> (bool, String, bool) {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    std::env::set_var("RUSTCODE_HOME", home.path());

    let mut cfg = CodingAgentConfig::new("k", "http://unused", "test-model", project.path());
    cfg.stream_timeout = Duration::from_secs(5);
    cfg.request_timeout = Some(Duration::from_millis(100));
    cfg.credential_shell_policy = rustcode_capabilities::tools::CredentialShellPolicy::Strict;
    let opts = PrepareOptions {
        session: SessionMode::Disabled,
        tools: true,
        skill_dirs: Some(vec![project.path().join("skills")]),
        plugin_skill_dirs: Vec::new(),
        mcp: false,
        extra_mcp_servers: Vec::new(),
        external_subagents: Vec::new(),
        memory: false,
        web: false,
        review: false,
        subagents: rustcode_coding::SubagentPolicy::Disabled,
        request_user_input: true,
        rate_limit_source: None,
    };
    let mut parts = prepare(&cfg, opts).await.unwrap();
    if auto_mode {
        // Exactly what `SetMode(RuntimeMode::Auto)` does in the live runtime.
        parts
            .bypass_mode
            .store(true, std::sync::atomic::Ordering::Release);
    }

    // Round 1: the model shells out with a credential literal. Round 2: it stops.
    let provider = Arc::new(RecordingProvider::new(vec![
        vec![
            StreamEvent::ToolCall(ToolCall {
                id: "c1".into(),
                name: "bash".into(),
                arguments: CRED_BASH.into(),
            }),
            StreamEvent::Done { truncated: false },
        ],
        vec![
            StreamEvent::TextDelta("done".into()),
            StreamEvent::Done { truncated: false },
        ],
    ]));

    let mut h = assemble(&mut parts, &cfg, provider).unwrap().spawn();
    h.commands
        .send(AgentCommand::SendMessage {
            text: "echo the token".into(),
            images: vec![],
        })
        .unwrap();

    let mut observed: Option<(bool, String)> = None;
    let mut intervention = false;
    while let Some(ev) = h.events.recv().await {
        match ev {
            AgentEvent::Request { id, .. } => {
                let _ = h.commands.send(AgentCommand::Respond {
                    id,
                    value: serde_json::json!({ "decision": "allow" }),
                });
            }
            AgentEvent::PolicyIntervention { .. } => intervention = true,
            AgentEvent::ToolResult { result } => {
                observed = Some((result.is_error, result.content));
            }
            AgentEvent::TurnComplete { .. } => break,
            _ => {}
        }
    }
    h.commands.send(AgentCommand::Shutdown).unwrap();
    let _ = h.task.await;

    let (is_error, content) = observed.expect("bash must produce a tool result");
    (is_error, content, intervention)
}

#[tokio::test]
async fn strict_credential_bash_terminates_the_turn() {
    let (is_error, content, intervention) = run_credential_bash(false).await;
    assert!(is_error, "a denied credential read must be an error result");
    assert!(
        content.contains("credentials must not be extracted"),
        "the block must be the credential-gate denial; got: {content:?}"
    );
    assert!(intervention, "strict must emit a policy intervention");
}

#[tokio::test]
async fn auto_mode_allows_the_same_credential_bash() {
    let (is_error, content, intervention) = run_credential_bash(true).await;
    assert!(!intervention, "Auto must not emit a policy intervention");
    assert!(
        !is_error,
        "Auto must let the command run, not deny it; got: {content:?}"
    );
    assert!(
        content.contains("real-looking-token"),
        "the command must actually have executed; got: {content:?}"
    );
}
