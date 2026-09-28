//! The per-message pipeline: one inbound message -> agent turn -> chunked reply.
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
//! # Concurrency
//!
//! This file handles exactly *one* message; the reader/worker fan-out that
//! decides which messages run in parallel lives in [`super::dispatch`]. In
//! particular a chat that already has a turn running must not start a second
//! one -- that ordering is the dispatcher's job (one worker per chat), not
//! something this function re-checks.
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
use std::sync::Arc;

use async_trait::async_trait;
use rustcode_config::i18n::{t, Msg};

use super::bridge::{BridgedReply, RecentMessages};
use super::commands;
use super::{ApprovalPort, ImAdapter, ImApprovalRelay, ImError, ImMessage, APPROVAL_WAIT};

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
    /// A chat command was handled without consulting the agent.
    CommandHandled { command: &'static str },
    /// The message was a redelivery and was skipped.
    Duplicate,
    /// The platform delivered a message with no usable text.
    IgnoredEmpty,
    /// The message came from a sender not on the channel's allowlist. A
    /// localized notice was sent; no session, agent turn, or token spend
    /// happened.
    SenderNotAllowed,
}

/// Chunk and send one reply to the conversation it came from.
///
/// Blank text sends nothing (an empty bubble is worse than silence). Shared by
/// turn answers and command answers so chunking has exactly one implementation.
async fn send_chunked(
    adapter: &dyn ImAdapter,
    message: &ImMessage,
    text: &str,
) -> Result<DispatchOutcome, ImError> {
    let Some(reply) = BridgedReply::new(
        &message.platform,
        &message.chat_id,
        message.reply_token.clone(),
        text,
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

/// Handle exactly one inbound message: dedupe, maybe answer a command, resolve
/// the session, run the agent, chunk and send the reply.
///
/// Called from a per-chat worker (see [`super::dispatch`]), which is what makes
/// "one chat is serial" true without this function holding a lock.
///
/// `allow_senders` is the channel's sender allowlist (empty = anyone on the
/// platform may drive this chat). When non-empty, a message whose `sender_id`
/// is not listed is dropped with a localized notice *before* any session or
/// agent work -- rejecting at the edge keeps a disallowed sender out of the
/// session store and the token spend entirely. The sender id is never echoed
/// (it is untrusted network input).
///
/// `dedupe` is shared across every chat and is locked **only** for the
/// observation itself. It must not be held across `run_turn`: a turn can last
/// minutes, and holding the guard that long would serialize the whole channel
/// (the redelivery guard is about message ids, not about turn ordering).
///
/// `replies` is the chat's inbox, handed to the approval relay so an answer to
/// "may I run this?" is read from *this* conversation's stream. `None` (tests,
/// and any caller without a live reader) makes approvals time out and
/// fail closed instead of consulting the transport.
pub async fn handle_message(
    adapter: &Arc<dyn ImAdapter>,
    runner: &dyn AgentRunner,
    project: &Path,
    dedupe: &tokio::sync::Mutex<RecentMessages>,
    message: &ImMessage,
    replies: Option<&mut tokio::sync::mpsc::Receiver<ImMessage>>,
    allow_senders: &[String],
) -> Result<DispatchOutcome, ImError> {
    // Sender allowlist edge check. Done first so a disallowed message touches
    // nothing -- no session binding, no agent turn, no token spend.
    if !allow_senders.is_empty() && !allow_senders.iter().any(|s| s == &message.sender_id) {
        adapter
            .send_text(
                &message.chat_id,
                message.reply_token.as_deref(),
                &t(Msg::ImSenderNotAllowed),
            )
            .await?;
        return Ok(DispatchOutcome::SenderNotAllowed);
    }

    // Redelivery guard next: a platform retry must not run a second turn (it
    // would cost tokens and could repeat a side-effecting tool call). The guard
    // is released immediately -- see the function note.
    if !dedupe.lock().await.observe(&message.message_id) {
        return Ok(DispatchOutcome::Duplicate);
    }

    let prompt = message.text.trim();
    if prompt.is_empty() {
        return Ok(DispatchOutcome::IgnoredEmpty);
    }

    // Resolve the session for this chat. An existing binding keeps its session
    // so the conversation continues; commands need it too (`/status` reports
    // which session the chat is driving).
    let existing = rustcode_config::im_store::load(&message.platform, &message.chat_id)
        .ok()
        .flatten();
    let prior_session = existing.as_ref().map(|b| b.session_id.clone());

    if let Some(command) = commands::parse_chat_command(prompt) {
        if command.resets_session() {
            // `/new`: drop the binding so the *next* message mints a fresh
            // session. Applied here rather than out of band because this is the
            // chat's worker -- the same task that would be running a turn -- so
            // the clearing cannot race an in-flight turn's own binding write.
            let _ = rustcode_config::im_store::remove(&message.platform, &message.chat_id);
            send_chunked(adapter.as_ref(), message, &commands::new_session_ack()).await?;
            return Ok(DispatchOutcome::CommandHandled {
                command: command.machine_token(),
            });
        }

        let context = commands::CommandContext {
            project: &project.to_string_lossy(),
            session_id: prior_session.as_deref(),
        };
        if let Some(answer) = commands::answer_for(&command, &context) {
            send_chunked(adapter.as_ref(), message, &answer).await?;
            return Ok(DispatchOutcome::CommandHandled {
                command: command.machine_token(),
            });
        }
    }

    // Approval broker for this turn: escalated tool calls are rendered into the
    // chat and answered there (see approval.rs). The relay borrows the adapter
    // and, when available, this chat's inbox; both borrows end when the turn
    // returns, so the adapter is free again for the reply below.
    //
    // A "processing" note is sent only if the turn runs past a quiet grace
    // period. Fast answers must not be preceded by chatter; a slow one must not
    // leave the user staring at silence wondering if anything happened.
    //
    // The note task is spawned with *owned* copies of the send target: `adapter`
    // and `message` are borrowed for the turn, but `tokio::spawn` requires
    // `'static`, so the task cannot borrow them. The reply below uses the same
    // target, so the shapes match.
    const PROCESSING_GRACE_SECS: u64 = 8;
    let note_chat = message.chat_id.clone();
    let note_token = message.reply_token.clone();
    let note_adapter = adapter.clone();
    let processing_task = {
        let timer = tokio::time::sleep(std::time::Duration::from_secs(PROCESSING_GRACE_SECS));
        tokio::spawn(async move {
            timer.await;
            let _ = note_adapter
                .send_text(
                    &note_chat,
                    note_token.as_deref(),
                    &t(Msg::ImTurnProcessing {
                        seconds: PROCESSING_GRACE_SECS,
                    }),
                )
                .await;
        })
    };

    let turn = {
        let relay = ImApprovalRelay::new(
            adapter.as_ref(),
            &message.chat_id,
            message.reply_token.as_deref(),
            APPROVAL_WAIT,
        );
        let mut relay = match replies {
            Some(mailbox) => relay.with_mailbox(mailbox),
            None => relay,
        };
        match runner
            .run_turn(project, prior_session.as_deref(), prompt, &mut relay)
            .await
        {
            Ok(turn) => {
                // The turn finished within the grace window: cancel the pending
                // "working on it" note so it never arrives after the real answer.
                processing_task.abort();
                Ok(turn)
            }
            Err(error) => {
                // Cancel the note on both paths: a fast-failing turn must not
                // emit "working on it" after the fact, and a slow one already
                // had its note fire (abort is then a no-op). The failure itself
                // is reported back to the chat so the user is not left with
                // silence.
                processing_task.abort();
                let notice = t(Msg::ImTurnFailed {
                    detail: &error.to_string(),
                });
                let _ = adapter
                    .send_text(&message.chat_id, message.reply_token.as_deref(), &notice)
                    .await;
                Err(error)
            }
        }
    }?;

    // Record the binding *after* a successful turn: writing it before would
    // leave a binding pointing at a session that never produced anything.
    let _ = rustcode_config::im_store::upsert_preserving_session(
        &message.platform,
        &message.chat_id,
        &project.to_string_lossy(),
        &turn.session_id,
    );

    send_chunked(adapter.as_ref(), message, &turn.text).await
}

/// A single channel to serve, as assembled by the binary.
///
/// Re-exported here so `runner` stays the module a caller reads to understand
/// the loop; the implementation lives in [`super::dispatch`].
pub use super::dispatch::ChannelSpec;

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
        inbound: Mutex<Vec<Option<ImMessage>>>,
        sent: Mutex<Vec<String>>,
    }

    impl FakeAdapter {
        fn with(inbound: Vec<ImMessage>) -> Self {
            let mut queue: Vec<Option<ImMessage>> = inbound.into_iter().map(Some).collect();
            queue.push(None); // terminate the stream
            Self {
                inbound: Mutex::new(queue),
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
        async fn next_message(&self) -> Result<Option<ImMessage>, ImError> {
            let mut queue = self.inbound.lock().unwrap();
            if queue.is_empty() {
                return Ok(None);
            }
            Ok(queue.remove(0))
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

    /// Drive one message through `handle_message` with no live reader.
    ///
    /// No mailbox means an escalated approval times out and fails closed --
    /// which is exactly what the single-message tests want to exercise.
    ///
    /// Generic over the concrete adapter so tests keep calling `sent()` on the
    /// same handle they pass in; the `Arc` is widened to `&Arc<dyn ImAdapter>`
    /// only at the `handle_message` boundary.
    async fn dispatch_one<A: ImAdapter + 'static>(
        adapter: &Arc<A>,
        runner: &FakeRunner,
        project: &Path,
        dedupe: &tokio::sync::Mutex<RecentMessages>,
        message: &ImMessage,
        allow_senders: &[String],
    ) -> Result<DispatchOutcome, ImError> {
        let dyn_adapter: Arc<dyn ImAdapter> = adapter.clone();
        handle_message(
            &dyn_adapter,
            runner,
            project,
            dedupe,
            message,
            None,
            allow_senders,
        )
        .await
    }

    /// Drive a whole script through the real dispatch fan-out, then wait for
    /// every chat's in-flight turn to finish before asserting.
    ///
    /// Returns the chunks sent, in order, so a test can assert on replies the
    /// same way the single-message helper does.
    async fn serve_script(
        adapter: &std::sync::Arc<FakeAdapter>,
        runner: &std::sync::Arc<FakeRunner>,
        project: &Path,
        max_in_flight: usize,
        allow_senders: Vec<String>,
    ) -> Vec<String> {
        let dyn_adapter: Arc<dyn ImAdapter> = adapter.clone();
        let dispatch = crate::im::dispatch::ChannelDispatch::new(
            dyn_adapter,
            runner.clone(),
            project.to_path_buf(),
            max_in_flight,
            allow_senders,
        );
        dispatch.serve_once().await.unwrap();
        dispatch.quiesce().await;
        adapter.sent()
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn a_message_runs_a_turn_and_sends_the_answer() {
        let tmp = tempfile::tempdir().unwrap();
        let _home = HomeGuard::new(tmp.path());
        let adapter = std::sync::Arc::new(FakeAdapter::with(vec![]));
        let runner = FakeRunner::ok("the answer");

        let outcome = dispatch_one(
            &adapter,
            &runner,
            tmp.path(),
            &tokio::sync::Mutex::new(RecentMessages::new(8)),
            &msg("m1", "hello"),
            &Vec::new(),
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
        let adapter = std::sync::Arc::new(FakeAdapter::with(vec![]));
        let runner = FakeRunner::ok("ok");
        let dedupe = tokio::sync::Mutex::new(RecentMessages::new(8));

        dispatch_one(
            &adapter,
            &runner,
            tmp.path(),
            &dedupe,
            &msg("m1", "first"),
            &Vec::new(),
        )
        .await
        .unwrap();
        dispatch_one(
            &adapter,
            &runner,
            tmp.path(),
            &dedupe,
            &msg("m2", "second"),
            &Vec::new(),
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
        let adapter = std::sync::Arc::new(FakeAdapter::with(vec![]));
        let runner = FakeRunner::ok("ok");
        let dedupe = tokio::sync::Mutex::new(RecentMessages::new(8));

        let first = dispatch_one(
            &adapter,
            &runner,
            tmp.path(),
            &dedupe,
            &msg("same-id", "hi"),
            &Vec::new(),
        )
        .await
        .unwrap();
        let second = dispatch_one(
            &adapter,
            &runner,
            tmp.path(),
            &dedupe,
            &msg("same-id", "hi"),
            &Vec::new(),
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
        let adapter = std::sync::Arc::new(FakeAdapter::with(vec![]));
        let runner = FakeRunner::ok("ok");
        let outcome = dispatch_one(
            &adapter,
            &runner,
            tmp.path(),
            &tokio::sync::Mutex::new(RecentMessages::new(8)),
            &msg("m1", "   "),
            &Vec::new(),
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
        let adapter = std::sync::Arc::new(FakeAdapter::with(vec![]));
        // Longer than the WeCom limit, but the platform here is dingtalk; use a
        // length past DingTalk's cap to force chunking.
        let runner = FakeRunner::ok(&"z".repeat(crate::im::bridge::limits::DINGTALK + 50));

        let outcome = dispatch_one(
            &adapter,
            &runner,
            tmp.path(),
            &tokio::sync::Mutex::new(RecentMessages::new(8)),
            &msg("m1", "give me a long answer"),
            &Vec::new(),
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
        let adapter = std::sync::Arc::new(FakeAdapter::with(vec![]));
        let runner = FakeRunner::ok("   ");
        let outcome = dispatch_one(
            &adapter,
            &runner,
            tmp.path(),
            &tokio::sync::Mutex::new(RecentMessages::new(8)),
            &msg("m1", "hi"),
            &Vec::new(),
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
        let adapter = std::sync::Arc::new(FakeAdapter::with(vec![msg("m1", "boom")]));
        let runner = std::sync::Arc::new(FakeRunner::failing());

        // The channel must finish cleanly (stream end) without the failed turn
        // taking the worker down. The failure is reported back to the chat as a
        // localized notice (P4: failure visibility) -- the user is never left
        // with silence, and no fast-failing turn leaks a "working on it" note.
        let sent = serve_script(&adapter, &runner, tmp.path(), 2, Vec::new()).await;
        assert_eq!(sent.len(), 1, "exactly the failure notice, got {sent:?}");
        assert!(
            sent[0].contains("provider exploded"),
            "the raw error detail must reach the user, got {sent:?}"
        );
        assert!(!sent[0].contains("正在处理"), "no stale processing note");
        assert_eq!(runner.calls().len(), 1, "the turn was attempted once");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn every_message_in_the_script_is_served() {
        let tmp = tempfile::tempdir().unwrap();
        let _home = HomeGuard::new(tmp.path());
        let adapter =
            std::sync::Arc::new(FakeAdapter::with(vec![msg("m1", "one"), msg("m2", "two")]));
        let runner = std::sync::Arc::new(FakeRunner::ok("pong"));

        let sent = serve_script(&adapter, &runner, tmp.path(), 2, Vec::new()).await;

        assert_eq!(sent, vec!["pong", "pong"]);
        assert_eq!(runner.calls().len(), 2);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn a_chat_command_is_answered_without_running_the_agent() {
        let tmp = tempfile::tempdir().unwrap();
        let _home = HomeGuard::new(tmp.path());
        let adapter = std::sync::Arc::new(FakeAdapter::with(vec![]));
        let runner = FakeRunner::ok("should not appear");

        let outcome = dispatch_one(
            &adapter,
            &runner,
            tmp.path(),
            &tokio::sync::Mutex::new(RecentMessages::new(8)),
            &msg("m1", "/status"),
            &Vec::new(),
        )
        .await
        .unwrap();

        assert_eq!(
            outcome,
            DispatchOutcome::CommandHandled { command: "status" }
        );
        assert!(
            runner.calls().is_empty(),
            "a command must not spend tokens on the agent"
        );
        let sent = adapter.sent();
        assert_eq!(sent.len(), 1);
        assert!(sent[0].contains(&tmp.path().to_string_lossy().to_string()));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn new_command_clears_the_binding_so_the_next_turn_starts_fresh() {
        let tmp = tempfile::tempdir().unwrap();
        let _home = HomeGuard::new(tmp.path());
        let adapter = std::sync::Arc::new(FakeAdapter::with(vec![]));
        let runner = FakeRunner::ok("ok");
        let dedupe = tokio::sync::Mutex::new(RecentMessages::new(8));

        // First message mints and persists a session binding.
        dispatch_one(
            &adapter,
            &runner,
            tmp.path(),
            &dedupe,
            &msg("m1", "hello"),
            &Vec::new(),
        )
        .await
        .unwrap();
        assert_eq!(
            runner.calls(),
            vec![None],
            "the first turn has nothing to resume"
        );

        // `/new` drops it, so the next turn must again start without a session.
        let outcome = dispatch_one(
            &adapter,
            &runner,
            tmp.path(),
            &dedupe,
            &msg("m2", "/new"),
            &Vec::new(),
        )
        .await
        .unwrap();
        assert_eq!(outcome, DispatchOutcome::CommandHandled { command: "new" });

        dispatch_one(
            &adapter,
            &runner,
            tmp.path(),
            &dedupe,
            &msg("m3", "hello again"),
            &Vec::new(),
        )
        .await
        .unwrap();
        assert_eq!(
            runner.calls(),
            vec![None, None],
            "`/new` must make the next turn start a fresh session"
        );
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

    #[tokio::test]
    #[serial_test::serial]
    async fn a_disallowed_sender_is_rejected_before_any_agent_work() {
        // Empty allowlist means everyone is served; a non-empty list must keep
        // anyone not on it out -- and the rejection must happen at the edge, so
        // the disallowed sender never reaches the session store or the token
        // spend, and gets a localized notice instead.
        let tmp = tempfile::tempdir().unwrap();
        let _home = HomeGuard::new(tmp.path());
        let adapter = std::sync::Arc::new(FakeAdapter::with(vec![]));
        let runner = FakeRunner::ok("should not appear");

        // A message from a sender not on the list is dropped with a notice.
        let outcome = dispatch_one(
            &adapter,
            &runner,
            tmp.path(),
            &tokio::sync::Mutex::new(RecentMessages::new(8)),
            &msg("m1", "hello from stranger"),
            &["alice".to_string()],
        )
        .await
        .unwrap();

        assert_eq!(outcome, DispatchOutcome::SenderNotAllowed);
        assert!(
            runner.calls().is_empty(),
            "no agent turn for a blocked sender"
        );
        let sent = adapter.sent();
        assert_eq!(sent.len(), 1);
        // The sender id is never echoed back (untrusted network input).
        assert!(!sent[0].contains("stranger"));

        // A message from a listed sender is served normally.
        let outcome = dispatch_one(
            &adapter,
            &runner,
            tmp.path(),
            &tokio::sync::Mutex::new(RecentMessages::new(8)),
            &msg_allowed("m2", "alice", "hello from alice"),
            &["alice".to_string()],
        )
        .await
        .unwrap();

        assert_eq!(outcome, DispatchOutcome::Replied { chunks: 1 });
        assert_eq!(runner.calls().len(), 1);
    }

    /// A message whose sender is explicitly allowed, so the allowlist test can
    /// drive a real turn without being blocked.
    fn msg_allowed(chat: &str, sender: &str, text: &str) -> ImMessage {
        ImMessage {
            platform: "dingtalk".into(),
            chat_id: chat.into(),
            sender_id: sender.into(),
            text: text.into(),
            message_id: chat.into(),
            reply_token: Some("https://hook.example/x".into()),
        }
    }
}
