//! The per-channel serve loop: inbound message -> agent turn -> chunked reply.
//!
//! # Why the agent call is injected
//!
//! Actually running a turn needs `spawn_native_cli_runtime` +
//! `run_native_headless`, which are `pub(crate)` inside the **binary** target.
//! A `lib` module cannot call them. Rather than duplicate the process-launch
//! wiring (or reach across the bin/lib boundary), the loop takes an
//! [`AgentRunner`] and the binary injects the real implementation. That keeps
//! this file's logic -- session continuity, dedupe, chunking, error surfacing --
//! fully testable against a fake runner, which is where the bugs actually live.
//!
//! # Session continuity
//!
//! The first message from a chat mints a session; every later message **reuses**
//! it (see `rustcode_config::im_store::upsert_preserving_session`). That is what
//! makes an IM conversation feel like a conversation rather than a series of
//! unrelated one-shot questions. A grouping key of `chat_id` -- not `sender_id`
//! -- is deliberate: in a group chat every sender must land in the *same*
//! session, otherwise the agent sees fragments of a discussion.

use std::path::{Path, PathBuf};

use async_trait::async_trait;

use super::bridge::{BridgedReply, RecentMessages};
use super::{ApprovalPort, ImAdapter, ImApprovalRelay, ImError, APPROVAL_WAIT};

/// Result of one agent turn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentTurn {
    /// The agent's final text. Empty means "nothing to say" -- the caller then
    /// sends nothing rather than an empty bubble.
    pub text: String,
    /// Session the turn ran in, so the binding can be refreshed.
    pub session_id: String,
}

/// Runs one agent turn. Implemented by the binary; faked in tests.
#[async_trait]
pub trait AgentRunner: Send + Sync {
    /// Run `prompt` in `project`.
    ///
    /// `session_id` is `Some` for a continuing conversation and `None` for a
    /// chat's first message. Implementations must treat a failed resume as an
    /// error rather than silently starting a fresh session -- a silent reset
    /// would look to the user like the agent "forgot" for no reason.
    ///
    /// `port` is the approval broker: when the agent escalates a tool call,
    /// the implementation surfaces it through this port (rendered by the IM
    /// relay as a chat card) instead of fail-closing silently. Implementations
    /// may ignore it only when their turn mode can never escalate.
    async fn run_turn(
        &self,
        project: &Path,
        session_id: Option<&str>,
        prompt: &str,
        port: &mut dyn ApprovalPort,
    ) -> Result<AgentTurn, ImError>;
}

/// What the loop did with one message, for logging and tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DispatchOutcome {
    /// A turn ran and the reply was sent (possibly in several chunks).
    Replied { chunks: usize },
    /// The turn produced no text, so nothing was sent.
    SilentlyCompleted,
    /// The message was a redelivery and was skipped.
    Duplicate,
    /// The platform delivered a message with no usable text.
    IgnoredEmpty,
}

/// Handle exactly one inbound message: dedupe, resolve the session, run the
/// agent, chunk and send the reply.
///
/// Kept separate from [`serve_channel`] so a single message can be driven in a
/// test without standing up a stream.
pub async fn handle_message(
    adapter: &mut dyn ImAdapter,
    runner: &dyn AgentRunner,
    project: &Path,
    dedupe: &mut RecentMessages,
    message: &super::ImMessage,
) -> Result<DispatchOutcome, ImError> {
    // Redelivery guard first: a platform retry must not run a second turn (it
    // would cost tokens and could repeat a side-effecting tool call).
    if !dedupe.observe(&message.message_id) {
        return Ok(DispatchOutcome::Duplicate);
    }

    let prompt = message.text.trim();
    if prompt.is_empty() {
        return Ok(DispatchOutcome::IgnoredEmpty);
    }

    // Resolve (or mint) the session for this chat. An existing binding keeps its
    // session so the conversation continues.
    let existing = rustcode_config::im_store::load(&message.platform, &message.chat_id)
        .ok()
        .flatten();
    let prior_session = existing.as_ref().map(|b| b.session_id.clone());

    // Approval broker for this turn: escalated tool calls are rendered into the
    // chat and answered there (see approval.rs). The relay borrows the adapter
    // for the duration of the turn; once the turn returns, the borrow ends and
    // the adapter is used again below for the reply.
    let mut relay = ImApprovalRelay::new(
        &mut *adapter,
        &message.chat_id,
        message.reply_token.as_deref(),
        APPROVAL_WAIT,
    );

    let turn = runner
        .run_turn(project, prior_session.as_deref(), prompt, &mut relay)
        .await?;

    // Record the binding *after* a successful turn: writing it before would
    // leave a binding pointing at a session that never produced anything.
    let _ = rustcode_config::im_store::upsert_preserving_session(
        &message.platform,
        &message.chat_id,
        &project.to_string_lossy(),
        &turn.session_id,
    );

    let Some(reply) = BridgedReply::new(
        &message.platform,
        &message.chat_id,
        message.reply_token.clone(),
        &turn.text,
    ) else {
        return Ok(DispatchOutcome::SilentlyCompleted);
    };

    let chunks = reply.chunks.len();
    for chunk in &reply.chunks {
        adapter
            .send_text(&message.chat_id, message.reply_token.as_deref(), chunk)
            .await?;
    }
    Ok(DispatchOutcome::Replied { chunks })
}

/// Serve one channel until the stream ends.
///
/// Returns `Ok(())` when the transport closed cleanly (the caller decides
/// whether to reconnect). A per-message failure is logged and **skipped** rather
/// than fatal: one bad message must not take down a working channel. A single
/// failed *reply* is reported back to the chat instead of vanishing, so the user
/// is not left staring at silence.
pub async fn serve_channel(
    adapter: &mut dyn ImAdapter,
    runner: &dyn AgentRunner,
    project: &Path,
    dedupe: &mut RecentMessages,
) -> Result<(), ImError> {
    while let Some(message) = adapter.next_message().await? {
        match handle_message(&mut *adapter, runner, project, dedupe, &message).await {
            Ok(_) => {}
            Err(error) => {
                // Tell the user their message failed, best-effort. Failing to
                // report a failure must not escalate into a channel teardown.
                let notice = format!("[error] {error}");
                let _ = adapter
                    .send_text(&message.chat_id, message.reply_token.as_deref(), &notice)
                    .await;
            }
        }
    }
    Ok(())
}

/// Project path for a channel, validated at startup.
///
/// Rejects an empty or non-absolute path: a relative path would silently resolve
/// against whatever directory the daemon happened to be launched from, which is
/// exactly the kind of "it worked on my machine" misconfiguration that is hard
/// to debug later.
pub fn resolve_project(configured: &str) -> Result<PathBuf, ImError> {
    let trimmed = configured.trim();
    if trimmed.is_empty() {
        return Err(ImError::Protocol(
            "IM channel has no `project` configured".into(),
        ));
    }
    let path = PathBuf::from(trimmed);
    if !path.is_absolute() {
        return Err(ImError::Protocol(format!(
            "IM channel `project` must be an absolute path, got `{trimmed}`"
        )));
    }
    if !path.is_dir() {
        return Err(ImError::Protocol(format!(
            "IM channel `project` is not a directory: `{trimmed}`"
        )));
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::im::ImMessage;
    use std::sync::Mutex;

    fn msg(id: &str, text: &str) -> ImMessage {
        ImMessage {
            platform: "dingtalk".into(),
            chat_id: "chat-1".into(),
            sender_id: "user-1".into(),
            text: text.into(),
            message_id: id.into(),
            reply_token: Some("https://hook.example/x".into()),
        }
    }

    /// Fake adapter: hands out a fixed script and records what was sent.
    struct FakeAdapter {
        inbound: Vec<Option<ImMessage>>,
        sent: Mutex<Vec<String>>,
    }

    impl FakeAdapter {
        fn with(inbound: Vec<ImMessage>) -> Self {
            let mut queue: Vec<Option<ImMessage>> = inbound.into_iter().map(Some).collect();
            queue.push(None); // terminate the stream
            Self {
                inbound: queue,
                sent: Mutex::new(Vec::new()),
            }
        }
        fn sent(&self) -> Vec<String> {
            self.sent.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl ImAdapter for FakeAdapter {
        fn platform(&self) -> &'static str {
            "dingtalk"
        }
        async fn next_message(&mut self) -> Result<Option<ImMessage>, ImError> {
            if self.inbound.is_empty() {
                return Ok(None);
            }
            Ok(self.inbound.remove(0))
        }
        async fn send_text(
            &self,
            _chat_id: &str,
            _reply_token: Option<&str>,
            text: &str,
        ) -> Result<(), ImError> {
            self.sent.lock().unwrap().push(text.to_string());
            Ok(())
        }
    }

    /// Fake runner: records the session it was given, answers from a script.
    struct FakeRunner {
        answer: String,
        minted_session: String,
        calls: Mutex<Vec<Option<String>>>,
        fail: bool,
    }

    impl FakeRunner {
        fn ok(answer: &str) -> Self {
            Self {
                answer: answer.into(),
                minted_session: "sess-minted".into(),
                calls: Mutex::new(Vec::new()),
                fail: false,
            }
        }
        fn failing() -> Self {
            Self {
                fail: true,
                ..Self::ok("")
            }
        }
        fn calls(&self) -> Vec<Option<String>> {
            self.calls.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl AgentRunner for FakeRunner {
        async fn run_turn(
            &self,
            _project: &Path,
            session_id: Option<&str>,
            _prompt: &str,
            _port: &mut dyn ApprovalPort,
        ) -> Result<AgentTurn, ImError> {
            self.calls
                .lock()
                .unwrap()
                .push(session_id.map(|s| s.to_string()));
            if self.fail {
                return Err(ImError::Transport("provider exploded".into()));
            }
            Ok(AgentTurn {
                text: self.answer.clone(),
                session_id: session_id.unwrap_or(&self.minted_session).to_string(),
            })
        }
    }

    /// The store writes to `$RUSTCODE_HOME`; point it at a throwaway dir.
    struct HomeGuard(Option<String>);
    impl HomeGuard {
        fn new(dir: &Path) -> Self {
            let old = std::env::var("RUSTCODE_HOME").ok();
            std::env::set_var("RUSTCODE_HOME", dir);
            Self(old)
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
    async fn a_message_runs_a_turn_and_sends_the_answer() {
        let tmp = tempfile::tempdir().unwrap();
        let _home = HomeGuard::new(tmp.path());
        let mut adapter = FakeAdapter::with(vec![]);
        let runner = FakeRunner::ok("the answer");

        let outcome = handle_message(
            &mut adapter,
            &runner,
            tmp.path(),
            &mut RecentMessages::new(8),
            &msg("m1", "hello"),
        )
        .await
        .unwrap();

        assert_eq!(outcome, DispatchOutcome::Replied { chunks: 1 });
        assert_eq!(adapter.sent(), vec!["the answer"]);
        // A first message has no session to resume.
        assert_eq!(runner.calls(), vec![None]);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn a_second_message_resumes_the_same_session() {
        // This is the whole point of the binding store: continuity.
        let tmp = tempfile::tempdir().unwrap();
        let _home = HomeGuard::new(tmp.path());
        let mut adapter = FakeAdapter::with(vec![]);
        let runner = FakeRunner::ok("ok");
        let mut dedupe = RecentMessages::new(8);

        handle_message(
            &mut adapter,
            &runner,
            tmp.path(),
            &mut dedupe,
            &msg("m1", "first"),
        )
        .await
        .unwrap();
        handle_message(
            &mut adapter,
            &runner,
            tmp.path(),
            &mut dedupe,
            &msg("m2", "second"),
        )
        .await
        .unwrap();

        assert_eq!(
            runner.calls(),
            vec![None, Some("sess-minted".to_string())],
            "the second turn must resume the minted session"
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn a_redelivered_message_does_not_run_a_second_turn() {
        let tmp = tempfile::tempdir().unwrap();
        let _home = HomeGuard::new(tmp.path());
        let mut adapter = FakeAdapter::with(vec![]);
        let runner = FakeRunner::ok("ok");
        let mut dedupe = RecentMessages::new(8);

        let first = handle_message(
            &mut adapter,
            &runner,
            tmp.path(),
            &mut dedupe,
            &msg("same-id", "hi"),
        )
        .await
        .unwrap();
        let second = handle_message(
            &mut adapter,
            &runner,
            tmp.path(),
            &mut dedupe,
            &msg("same-id", "hi"),
        )
        .await
        .unwrap();

        assert_eq!(first, DispatchOutcome::Replied { chunks: 1 });
        assert_eq!(second, DispatchOutcome::Duplicate);
        assert_eq!(runner.calls().len(), 1, "the agent must run only once");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn empty_text_is_ignored_without_calling_the_agent() {
        let tmp = tempfile::tempdir().unwrap();
        let _home = HomeGuard::new(tmp.path());
        let mut adapter = FakeAdapter::with(vec![]);
        let runner = FakeRunner::ok("ok");
        let outcome = handle_message(
            &mut adapter,
            &runner,
            tmp.path(),
            &mut RecentMessages::new(8),
            &msg("m1", "   "),
        )
        .await
        .unwrap();
        assert_eq!(outcome, DispatchOutcome::IgnoredEmpty);
        assert!(runner.calls().is_empty());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn a_long_answer_is_sent_in_several_chunks() {
        let tmp = tempfile::tempdir().unwrap();
        let _home = HomeGuard::new(tmp.path());
        let mut adapter = FakeAdapter::with(vec![]);
        // Longer than the WeCom limit, but the platform here is dingtalk; use a
        // length past DingTalk's cap to force chunking.
        let runner = FakeRunner::ok(&"z".repeat(crate::im::bridge::limits::DINGTALK + 50));

        let outcome = handle_message(
            &mut adapter,
            &runner,
            tmp.path(),
            &mut RecentMessages::new(8),
            &msg("m1", "give me a long answer"),
        )
        .await
        .unwrap();

        match outcome {
            DispatchOutcome::Replied { chunks } => assert!(chunks >= 2, "expected chunking"),
            other => panic!("expected a chunked reply, got {other:?}"),
        }
        assert_eq!(adapter.sent().len(), 2);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn an_empty_answer_sends_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let _home = HomeGuard::new(tmp.path());
        let mut adapter = FakeAdapter::with(vec![]);
        let runner = FakeRunner::ok("   ");
        let outcome = handle_message(
            &mut adapter,
            &runner,
            tmp.path(),
            &mut RecentMessages::new(8),
            &msg("m1", "hi"),
        )
        .await
        .unwrap();
        assert_eq!(outcome, DispatchOutcome::SilentlyCompleted);
        assert!(adapter.sent().is_empty(), "no empty bubble");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn a_failing_turn_reports_back_and_does_not_kill_the_channel() {
        let tmp = tempfile::tempdir().unwrap();
        let _home = HomeGuard::new(tmp.path());
        let mut adapter = FakeAdapter::with(vec![msg("m1", "boom")]);
        let runner = FakeRunner::failing();

        // The loop must finish cleanly (stream end), having reported the error.
        serve_channel(
            &mut adapter,
            &runner,
            tmp.path(),
            &mut RecentMessages::new(8),
        )
        .await
        .unwrap();

        let sent = adapter.sent();
        assert_eq!(sent.len(), 1);
        assert!(sent[0].contains("[error]"), "got {sent:?}");
        assert!(sent[0].contains("provider exploded"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn serve_channel_processes_every_message_then_returns() {
        let tmp = tempfile::tempdir().unwrap();
        let _home = HomeGuard::new(tmp.path());
        let mut adapter = FakeAdapter::with(vec![msg("m1", "one"), msg("m2", "two")]);
        let runner = FakeRunner::ok("pong");

        serve_channel(
            &mut adapter,
            &runner,
            tmp.path(),
            &mut RecentMessages::new(8),
        )
        .await
        .unwrap();

        assert_eq!(adapter.sent(), vec!["pong", "pong"]);
        assert_eq!(runner.calls().len(), 2);
    }

    #[test]
    fn resolve_project_requires_an_absolute_existing_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let ok = tmp.path().to_string_lossy().to_string();
        assert_eq!(resolve_project(&ok).unwrap(), tmp.path());

        // Empty / relative / nonexistent all fail loudly rather than silently
        // resolving against an unexpected cwd.
        assert!(resolve_project("").is_err());
        assert!(resolve_project("   ").is_err());
        assert!(resolve_project("relative/path").is_err());
        assert!(resolve_project("/definitely/not/a/real/dir/xyzzy").is_err());
    }

    #[test]
    fn agent_turn_keeps_text_and_session() {
        let t = AgentTurn {
            text: "hi".into(),
            session_id: "s".into(),
        };
        assert_eq!(t.text, "hi");
        assert_eq!(t.session_id, "s");
    }
}
