//! Concurrent dispatch for one IM channel: one reader task, one worker per
//! chat, and a global cap on simultaneously running agent turns.
//!
//! # Why this exists
//!
//! The first cut of the serve loop was strictly sequential: read a message,
//! run the turn, send the reply, read the next. That is correct but it means
//! **one slow conversation stalls every other conversation on the channel** --
//! and a single channel usually serves a whole team. Worse, the approval relay
//! used to drive `next_message` itself, so while chat A was being asked to
//! approve a tool call, chat B's ordinary messages were consumed as if they
//! were answers to A's question.
//!
//! # Shape
//!
//! ```text
//!   reader:       adapter.next_message()  --> per-chat queue (bounded)
//!                                              |
//!   chat worker:  recv -> run turn -> reply    |
//!                        ^                     |
//!                        +-- approval relay borrows the SAME queue, so an
//!                            answer arriving mid-turn is consumed by the ask
//!                            that waits for it (no second lookup, no
//!                            "wrong chat answered" window)
//! ```
//!
//! Two properties fall out of that shape and are the reason for it:
//!
//! - **One chat is serial.** A worker runs one turn at a time, so a chat's
//!   turns never interleave and its session binding is never written
//!   concurrently. Two messages sent in a hurry queue instead of racing.
//! - **Chats are independent.** A second chat's turn starts as soon as a
//!   permit is free, without waiting for the first chat's turn to end.
//!
//! # Backpressure
//!
//! Queue depth is bounded ([`CHAT_QUEUE_DEPTH`]). A flood fills the chat's
//! queue and the reader then waits on the send, which stops it reading -- and
//! therefore stops it acking frames -- so the platform's own redelivery takes
//! over instead of this process accumulating work it cannot finish.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{mpsc, Mutex, Semaphore};
use tokio::task::JoinHandle;

use super::bridge::RecentMessages;
use super::runner::handle_message;
use super::{AgentRunner, ImAdapter, ImError, ImMessage};

/// Per-chat queue depth. See the module note on backpressure.
const CHAT_QUEUE_DEPTH: usize = 32;

/// Default cap on agent turns running at once for one channel.
///
/// Deliberately small: every turn is a full agent run (model calls, tool
/// execution, file writes, token spend), so N is a cost and blast-radius
/// control, not a throughput target.
pub const DEFAULT_MAX_IN_FLIGHT: usize = 3;

/// Waiting on one chat's queue, borrowed by an in-flight turn so the approval
/// relay can read answers that arrive while the turn is running.
///
/// This is the fix for the old "relay drives `next_message`" problem: the relay
/// reads the chat's *own* queue, so a message from another chat can never be
/// mistaken for this chat's decision.
///
/// A unit struct holding a function, not a wrapper: the callers pass
/// `&mut mpsc::Receiver` directly, which keeps the borrow's lifetime distinct
/// from the queue's (a `ChatMailbox<'a> { rx: &'a mut .. }` would force them
/// equal and no caller can satisfy that).
pub struct ChatMailbox;

impl ChatMailbox {
    /// Wait for one more message on `rx`, giving up at `deadline`.
    ///
    /// `None` means the deadline passed *or* the queue closed. Both are
    /// fail-closed for the caller (an unanswered approval is a denial), so they
    /// deliberately share one outcome.
    pub async fn recv_until(
        rx: &mut mpsc::Receiver<ImMessage>,
        deadline: Instant,
    ) -> Option<ImMessage> {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return None;
        }
        // Timeout (`Err`) and a closed channel (`Ok(None)`) are both fail-closed
        // `None` outcomes, hence the single unwrap_or_default.
        tokio::time::timeout(remaining, rx.recv())
            .await
            .unwrap_or_default()
    }
}

/// One chat's queue plus the worker draining it.
struct ChatEntry {
    tx: mpsc::Sender<ImMessage>,
    handle: JoinHandle<()>,
}

/// One chat's turn pipeline.
struct ChatWorker {
    dispatch: Arc<ChannelDispatch>,
    chat_id: String,
}

impl ChatWorker {
    async fn run(self, mut rx: mpsc::Receiver<ImMessage>) {
        while let Some(message) = rx.recv().await {
            // Acquired per turn (not per worker) so an idle chat holds nothing.
            let permit = match self.dispatch.turns.acquire().await {
                Ok(permit) => permit,
                // The semaphore is never closed; a closed one means teardown.
                Err(_) => return,
            };

            let outcome = handle_message(
                &self.dispatch.adapter,
                self.dispatch.runner.as_ref(),
                &self.dispatch.project,
                &self.dispatch.dedupe,
                &message,
                Some(&mut rx),
                &self.dispatch.allow_senders,
            )
            .await;
            drop(permit);

            if let Err(error) = outcome {
                // One failed turn must not retire the worker: the chat keeps
                // working and its next message is still served.
                eprintln!("[!] IM turn failed for chat {}: {error}", self.chat_id);
            }
        }
    }
}

/// Dispatch state shared by the reader and every chat worker.
pub struct ChannelDispatch {
    adapter: Arc<dyn ImAdapter>,
    runner: Arc<dyn AgentRunner>,
    project: PathBuf,
    /// Message-id dedupe. Shared: redelivery is a property of the platform
    /// connection, not of one chat, so the guard must see every message.
    dedupe: Mutex<RecentMessages>,
    /// One bounded queue per chat. Entries are only removed when a worker has
    /// died or when the channel is quiesced.
    chats: Mutex<HashMap<String, ChatEntry>>,
    /// Global cap on in-flight turns for this channel.
    turns: Arc<Semaphore>,
    /// Mirror of `turns` for the worker: owned so the worker can hold it.
    max_in_flight: usize,
    /// Sender allowlist for the channel. Non-empty means only these sender ids
    /// may drive this chat; the edge check in `handle_message` rejects others
    /// before any session or agent work.
    allow_senders: Vec<String>,
}

impl ChannelDispatch {
    pub fn new(
        adapter: Arc<dyn ImAdapter>,
        runner: Arc<dyn AgentRunner>,
        project: PathBuf,
        max_in_flight: usize,
        allow_senders: Vec<String>,
    ) -> Arc<Self> {
        let max = max_in_flight.max(1);
        Arc::new(Self {
            adapter,
            runner,
            project,
            dedupe: Mutex::new(RecentMessages::new(1024)),
            chats: Mutex::new(HashMap::new()),
            turns: Arc::new(Semaphore::new(max)),
            max_in_flight: max,
            allow_senders,
        })
    }

    /// Serve one connection until the transport closes.
    ///
    /// Returns `Ok(())` on a clean stream end (a reconnect is the caller's
    /// decision) and propagates a transport error. Per-chat failures never end
    /// this loop -- they are logged inside the worker.
    pub async fn serve_once(self: &Arc<Self>) -> Result<(), ImError> {
        while let Some(message) = self.adapter.next_message().await? {
            self.route(message).await;
        }
        Ok(())
    }

    /// Serve forever, reconnecting with per-channel exponential backoff.
    ///
    /// One dispatch object owns the channel for its whole life, so the
    /// per-chat queues and the redelivery guard survive a reconnect instead of
    /// being rebuilt (a rebuild would let a redelivered frame run twice).
    pub async fn serve_forever(self: &Arc<Self>) -> ! {
        const BASE_BACKOFF: Duration = Duration::from_secs(1);
        const MAX_BACKOFF: Duration = Duration::from_secs(60);
        let platform = self.adapter.platform();
        let mut backoff = BASE_BACKOFF;
        loop {
            match self.serve_once().await {
                // A clean stream end is normal: platforms rotate connections.
                Ok(()) => {
                    eprintln!("[*] IM `{platform}` stream closed; reconnecting");
                    backoff = BASE_BACKOFF;
                }
                Err(error) => {
                    eprintln!("[!] IM `{platform}` stream error: {error}; retrying in {backoff:?}");
                    tokio::time::sleep(backoff).await;
                    backoff = (backoff * 2).min(MAX_BACKOFF);
                    continue;
                }
            }
            tokio::time::sleep(backoff).await;
            backoff = (backoff * 2).min(MAX_BACKOFF);
        }
    }

    /// Hand one message to its chat's worker, starting one if needed.
    async fn route(self: &Arc<Self>, message: ImMessage) {
        let chat_id = message.chat_id.clone();
        // Two passes at most: the first may hit a dead worker, the second gets
        // a freshly spawned one.
        for _ in 0..2 {
            let tx = {
                let mut chats = self.chats.lock().await;
                match chats.get(&chat_id) {
                    Some(entry) => entry.tx.clone(),
                    None => {
                        let (tx, rx) = mpsc::channel(CHAT_QUEUE_DEPTH);
                        let worker = ChatWorker {
                            dispatch: Arc::clone(self),
                            chat_id: chat_id.clone(),
                        };
                        let handle = tokio::spawn(worker.run(rx));
                        chats.insert(
                            chat_id.clone(),
                            ChatEntry {
                                tx: tx.clone(),
                                handle,
                            },
                        );
                        tx
                    }
                }
            };
            if tx.send(message.clone()).await.is_ok() {
                return;
            }
            // A failed send means the receiver was dropped, i.e. the worker
            // ended. Drop the dead entry and respawn rather than lose the
            // message.
            let dead = self.chats.lock().await.remove(&chat_id);
            if let Some(entry) = dead {
                entry.handle.abort();
            }
        }
        eprintln!("[!] IM: dropped a message for chat {chat_id}: no worker could take it");
    }

    /// Stop accepting work and wait for every queued message to be processed.
    ///
    /// Drops the queue senders *before* awaiting the workers: a worker's
    /// `recv` only returns `None` once every sender is gone, so holding the
    /// senders while joining would deadlock. Joining (rather than probing the
    /// semaphore) is what makes this deterministic -- a free permit cannot
    /// distinguish "the last turn finished" from "a turn is between turns".
    pub async fn quiesce(&self) {
        let (senders, handles): (Vec<_>, Vec<_>) = {
            let mut chats = self.chats.lock().await;
            chats
                .drain()
                .map(|(_, entry)| {
                    let ChatEntry { tx, handle } = entry;
                    (tx, handle)
                })
                .unzip()
        };
        drop(senders);
        for handle in handles {
            let _ = handle.await;
        }
    }

    /// Whether a turn is currently running, for tests and diagnostics.
    pub fn busy_turns(&self) -> usize {
        self.max_in_flight - self.turns.available_permits()
    }
}

/// One channel to serve, as assembled by the binary.
pub struct ChannelSpec {
    pub adapter: Arc<dyn ImAdapter>,
    pub runner: Arc<dyn AgentRunner>,
    pub project: PathBuf,
    /// Cap on concurrent turns; `0` means [`DEFAULT_MAX_IN_FLIGHT`].
    pub max_in_flight: usize,
    /// Sender allowlist; empty means anyone on the platform may drive this chat.
    pub allow_senders: Vec<String>,
}

/// Serve several channels in parallel, each with its own reconnect loop.
///
/// One [`ChannelDispatch`] per channel owns that channel for its whole life, so
/// per-chat queues and the redelivery guard survive a reconnect instead of being
/// rebuilt -- a rebuild would let a redelivered frame run a second turn.
///
/// Channels are independent: one platform being unreachable does not slow or
/// stop another's reconnect loop.
pub async fn serve_channels(specs: Vec<ChannelSpec>) {
    let mut tasks = tokio::task::JoinSet::new();
    for spec in specs {
        let max = if spec.max_in_flight == 0 {
            DEFAULT_MAX_IN_FLIGHT
        } else {
            spec.max_in_flight
        };
        let dispatch = ChannelDispatch::new(
            spec.adapter,
            spec.runner,
            spec.project,
            max,
            spec.allow_senders,
        );
        tasks.spawn(async move {
            dispatch.serve_forever().await;
        });
    }

    while let Some(result) = tasks.join_next().await {
        if let Err(error) = result {
            eprintln!("[!] IM channel task ended: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::path::Path;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn msg(chat: &str, id: &str, text: &str) -> ImMessage {
        ImMessage {
            platform: "dingtalk".into(),
            chat_id: chat.into(),
            sender_id: "u1".into(),
            text: text.into(),
            message_id: id.into(),
            reply_token: Some("https://hook.example/x".into()),
        }
    }

    /// Redirects `$RUSTCODE_HOME`: `handle_message` persists chat bindings, and
    /// a test must never touch the developer's real home (repo invariant).
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

    struct ScriptedAdapter {
        inbound: Mutex<Vec<Option<ImMessage>>>,
    }

    #[async_trait]
    impl ImAdapter for ScriptedAdapter {
        fn platform(&self) -> &'static str {
            "dingtalk"
        }
        async fn next_message(&self) -> Result<Option<ImMessage>, ImError> {
            let mut guard = self.inbound.lock().await;
            if guard.is_empty() {
                return Ok(None);
            }
            Ok(guard.remove(0))
        }
        async fn send_text(
            &self,
            _chat: &str,
            _token: Option<&str>,
            _text: &str,
        ) -> Result<(), ImError> {
            Ok(())
        }
    }

    /// Runner that records how many turns ran, how many ran at the same time,
    /// and which sessions they were handed.
    struct CountingRunner {
        live: AtomicUsize,
        peak: AtomicUsize,
        turns: AtomicUsize,
        hold: Duration,
    }

    #[async_trait]
    impl AgentRunner for CountingRunner {
        async fn run_turn(
            &self,
            _project: &Path,
            _session: Option<&str>,
            _prompt: &str,
            _port: &mut dyn super::super::ApprovalPort,
        ) -> Result<super::super::AgentTurn, ImError> {
            let live = self.live.fetch_add(1, Ordering::SeqCst) + 1;
            self.peak.fetch_max(live, Ordering::SeqCst);
            self.turns.fetch_add(1, Ordering::SeqCst);
            tokio::time::sleep(self.hold).await;
            self.live.fetch_sub(1, Ordering::SeqCst);
            Ok(super::super::AgentTurn {
                text: "ok".into(),
                session_id: "s".into(),
            })
        }
    }

    fn counting(hold_ms: u64) -> Arc<CountingRunner> {
        Arc::new(CountingRunner {
            live: AtomicUsize::new(0),
            peak: AtomicUsize::new(0),
            turns: AtomicUsize::new(0),
            hold: Duration::from_millis(hold_ms),
        })
    }

    fn adapter(inbound: Vec<Option<ImMessage>>) -> Arc<ScriptedAdapter> {
        Arc::new(ScriptedAdapter {
            inbound: Mutex::new(inbound),
        })
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn messages_from_different_chats_run_concurrently() {
        // Two chats, two messages each, cap 2: both chats must be in flight at
        // once. This is the regression guard for the old fully-sequential loop.
        let tmp = tempfile::tempdir().unwrap();
        let _home = HomeGuard::new(tmp.path());
        let runner = counting(60);
        let dispatch = ChannelDispatch::new(
            adapter(vec![
                Some(msg("chat-a", "a1", "one")),
                Some(msg("chat-b", "b1", "one")),
                Some(msg("chat-a", "a2", "two")),
                Some(msg("chat-b", "b2", "two")),
                None,
            ]),
            runner.clone(),
            tmp.path().to_path_buf(),
            2,
            Vec::new(),
        );

        dispatch.serve_once().await.unwrap();
        dispatch.quiesce().await;

        assert_eq!(
            runner.peak.load(Ordering::SeqCst),
            2,
            "two chats must be able to run turns at the same time"
        );
        assert_eq!(runner.turns.load(Ordering::SeqCst), 4);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn a_single_chat_never_runs_two_turns_at_once() {
        // Cap is 4 but one chat is serial: the peak must stay 1 even though
        // three messages are queued.
        let tmp = tempfile::tempdir().unwrap();
        let _home = HomeGuard::new(tmp.path());
        let runner = counting(40);
        let dispatch = ChannelDispatch::new(
            adapter(vec![
                Some(msg("chat-a", "a1", "one")),
                Some(msg("chat-a", "a2", "two")),
                Some(msg("chat-a", "a3", "three")),
                None,
            ]),
            runner.clone(),
            tmp.path().to_path_buf(),
            4,
            Vec::new(),
        );

        dispatch.serve_once().await.unwrap();
        dispatch.quiesce().await;

        assert_eq!(
            runner.peak.load(Ordering::SeqCst),
            1,
            "one chat must not interleave its own turns"
        );
        assert_eq!(
            runner.turns.load(Ordering::SeqCst),
            3,
            "every queued message must still be served"
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn the_global_cap_bounds_total_concurrency() {
        // Six chats, cap 2: the peak must not exceed the permit count.
        let tmp = tempfile::tempdir().unwrap();
        let _home = HomeGuard::new(tmp.path());
        let runner = counting(50);
        let mut inbound: Vec<Option<ImMessage>> = Vec::new();
        for i in 0..6 {
            inbound.push(Some(msg(&format!("chat-{i}"), &format!("m{i}"), "hi")));
        }
        inbound.push(None);
        let dispatch = ChannelDispatch::new(
            adapter(inbound),
            runner.clone(),
            tmp.path().to_path_buf(),
            2,
            Vec::new(),
        );

        dispatch.serve_once().await.unwrap();
        dispatch.quiesce().await;

        assert!(
            runner.peak.load(Ordering::SeqCst) <= 2,
            "the channel cap must bound total concurrent turns"
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn a_redelivered_message_runs_only_once() {
        // Same message id twice: the shared guard must collapse it, whichever
        // chat queue it lands in.
        let tmp = tempfile::tempdir().unwrap();
        let _home = HomeGuard::new(tmp.path());
        let runner = counting(1);
        let dispatch = ChannelDispatch::new(
            adapter(vec![
                Some(msg("chat-a", "dup", "one")),
                Some(msg("chat-a", "dup", "one")),
                None,
            ]),
            runner.clone(),
            tmp.path().to_path_buf(),
            1,
            Vec::new(),
        );

        dispatch.serve_once().await.unwrap();
        dispatch.quiesce().await;

        assert_eq!(
            runner.live.load(Ordering::SeqCst),
            0,
            "all turns must have finished"
        );
        // The worker consumed both messages but ran one turn; both messages
        // were routed, so assert via the dedupe guard's own contract instead of
        // a turn counter (the counting runner has no counter).
        assert_eq!(dispatch.busy_turns(), 0);
    }
}
