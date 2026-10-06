use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, RwLock};
use std::time::Duration;

use rustcode_capabilities::team::{
    role_by_id, validate_non_overlapping_worker_scopes, TeamEvent, TeamEventPayload, TeamMemberId,
    TeamPermission, TeamRunId, TeamTaskSpec,
};
use tokio::task::AbortHandle;
use tokio_util::sync::CancellationToken;

pub type TeamJob = Pin<Box<dyn Future<Output = TeamMemberOutcome> + Send + 'static>>;
/// `(activity_text, estimated_output_tokens)` -- the runner reports a live token
/// estimate alongside each activity so the panel matches the `task` subagent.
pub type TeamActivitySink = Arc<dyn Fn(String, u64) + Send + Sync>;
pub type TeamJobFactory =
    Arc<dyn Fn(TeamTaskSpec, CancellationToken, TeamActivitySink) -> TeamJob + Send + Sync>;
pub type TeamModelFactory = Arc<dyn Fn(&TeamTaskSpec) -> String + Send + Sync>;

#[derive(Clone, Debug)]
pub struct TeamRuntimeConfig {
    /// Max concurrent WRITE members (Worker permission). These edit the working tree
    /// and are scope-confined, so the lane stays conservative.
    pub max_concurrent: usize,
    /// Max concurrent READ-ONLY members (Explore permission). They never write and
    /// cannot conflict, so investigation roles fan out wider than workers. The two
    /// lanes use independent semaphores (see [`TeamRunManager::delegate`]).
    pub max_concurrent_explore: usize,
    pub cancel_grace: Duration,
    pub max_result_chars: usize,
    pub max_completed_runs: usize,
    /// Maximum delegation depth for hierarchical team dispatch.
    /// 0 = flat (no sub-dispatch), 2 = three-tier (root -> child -> grandchild).
    pub max_depth: u8,
}

impl Default for TeamRuntimeConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 3,
            max_concurrent_explore: 8,
            cancel_grace: Duration::from_secs(2),
            max_result_chars: 12_000,
            max_completed_runs: 32,
            max_depth: 2,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenerationTeamEvent {
    pub generation: u64,
    pub event: TeamEvent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TeamMemberOutcome {
    pub success: bool,
    pub stop: String,
    pub output: String,
    /// Final accumulated output-token estimate for the member's whole run, so the
    /// closing round (which surfaces no activity) is still reflected in the panel.
    pub output_tokens: u64,
}

impl TeamMemberOutcome {
    pub fn completed(output: impl Into<String>) -> Self {
        Self {
            success: true,
            stop: "completed".into(),
            output: output.into(),
            output_tokens: 0,
        }
    }

    pub fn failed(output: impl Into<String>) -> Self {
        Self {
            success: false,
            stop: "failed".into(),
            output: output.into(),
            output_tokens: 0,
        }
    }

    fn stopped(reason: impl Into<String>) -> Self {
        Self {
            success: false,
            stop: "stopped".into(),
            output: reason.into(),
            output_tokens: 0,
        }
    }
}

#[derive(Clone)]
pub struct TeamRunManager {
    inner: Arc<Inner>,
}

struct Inner {
    store: Mutex<TeamRunStore>,
    generation: AtomicU64,
    generation_root: Mutex<CancellationToken>,
    event_tx: RwLock<Option<tokio::sync::mpsc::UnboundedSender<GenerationTeamEvent>>>,
    /// Serializes seq assignment with the channel send so the receiver observes
    /// events in seq order. Concurrent members share one per-run seq counter; if a
    /// member reserved a lower seq (`fetch_add`) but lost the `send` race, the
    /// consumer's monotonic filter (`seq <= previous`) would drop it forever.
    emit_lock: Mutex<()>,
    /// Generation captured when a synchronous legacy `task` batch starts.
    /// Late events are ignored after `begin_generation` clears this map.
    external_runs: Mutex<BTreeMap<String, u64>>,
    config: TeamRuntimeConfig,
    run_counter: AtomicU64,
    /// Dispatch depth of this manager (0 = root, 1 = child, 2 = grandchild).
    depth: u8,
    /// Cascading chain: child managers built via [`TeamRunManager::child_manager`].
    /// Holding strong refs keeps the whole subtree alive while the root lives, and
    /// lets `stop_all`/`begin_generation`/`set_event_sender` recurse downward so
    /// cancellation and generation switches reach every tier (F3-HIGH-2).
    children: Mutex<Vec<Arc<Inner>>>,
}

impl Drop for Inner {
    fn drop(&mut self) {
        self.generation_root
            .get_mut()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .cancel();
        let store = self
            .store
            .get_mut()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        for run in store.runs.values() {
            for member in run.members.values() {
                if let Some(abort) = &member.abort {
                    abort.abort();
                }
            }
        }
    }
}

impl TeamRunManager {
    pub fn new(config: TeamRuntimeConfig) -> Self {
        Self::with_depth(0, config)
    }

    pub fn with_depth(depth: u8, config: TeamRuntimeConfig) -> Self {
        Self {
            inner: Arc::new(Inner {
                store: Mutex::new(TeamRunStore::default()),
                generation: AtomicU64::new(0),
                generation_root: Mutex::new(CancellationToken::new()),
                event_tx: RwLock::new(None),
                emit_lock: Mutex::new(()),
                external_runs: Mutex::new(BTreeMap::new()),
                config,
                run_counter: AtomicU64::new(1),
                depth,
                children: Mutex::new(Vec::new()),
            }),
        }
    }

    pub fn depth(&self) -> u8 {
        self.inner.depth
    }

    pub fn max_depth(&self) -> u8 {
        self.inner.config.max_depth
    }

    /// Create a child TeamRunManager for sub-dispatch at depth+1.
    /// Returns None if depth+1 > max_depth.
    ///
    /// The child's cancellation root is a [`CancellationToken::child_token`] of the
    /// parent's *current* generation root, so cancelling or regenerating the parent
    /// cascades down (F3-HIGH-2). The child is also registered in the parent's
    /// `children` chain so `stop_all`/`begin_generation`/`set_event_sender` can
    /// recurse. The event channel is bridged so child events reach the same
    /// consumer.
    pub fn child_manager(&self) -> Option<Self> {
        let next_depth = self.inner.depth + 1;
        if next_depth > self.inner.config.max_depth {
            return None;
        }
        let child = Self::with_depth(next_depth, self.inner.config.clone());
        // Cancel-cascade: child root is a child token of the parent's live root.
        let parent_root = self
            .inner
            .generation_root
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone();
        *child
            .inner
            .generation_root
            .lock()
            .unwrap_or_else(|p| p.into_inner()) = parent_root.child_token();
        // Bridge event channel so child events propagate to the same consumer.
        if let Some(tx) = self
            .inner
            .event_tx
            .read()
            .unwrap_or_else(|p| p.into_inner())
            .as_ref()
        {
            let _ = child
                .inner
                .event_tx
                .write()
                .unwrap_or_else(|p| p.into_inner())
                .insert(tx.clone());
        }
        // Register in the parent's cascade chain.
        self.inner
            .children
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .push(Arc::clone(&child.inner));
        Some(child)
    }

    pub fn generation(&self) -> u64 {
        self.inner.generation.load(Ordering::Acquire)
    }

    pub fn set_event_sender(
        &self,
        sender: tokio::sync::mpsc::UnboundedSender<GenerationTeamEvent>,
    ) {
        *self
            .inner
            .event_tx
            .write()
            .unwrap_or_else(|p| p.into_inner()) = Some(sender.clone());
        // Cascade to child managers so every tier emits to the same consumer
        // (F3-HIGH-2).
        let children = self
            .inner
            .children
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone();
        for child in children {
            *child
                .event_tx
                .write()
                .unwrap_or_else(|p| p.into_inner()) = Some(sender.clone());
        }
    }

    /// Starts a new runtime generation. Existing work is first made ineligible to
    /// publish, then cancelled; callers must await [`stop_all`] before replacement.
    pub fn begin_generation(&self, generation: u64) {
        self.inner.generation.store(generation, Ordering::Release);
        let new_root = {
            let mut root = self
                .inner
                .generation_root
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            root.cancel();
            let new_root = CancellationToken::new();
            *root = new_root.clone();
            new_root
        };
        self.inner
            .external_runs
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clear();
        // Cascade: re-derive every child's root from the new parent root so a
        // generation switch invalidates all tiers and re-hooks them to the live
        // generation (F3-HIGH-2).
        self.renew_child_roots(&new_root, generation);
    }

    /// Recursively re-derive `child.generation_root` as a child token of
    /// `parent_root` and propagate the event sender. Called by `begin_generation`
    /// so the cancellation tree and event channel reach every tier.
    fn renew_child_roots(&self, parent_root: &CancellationToken, generation: u64) {
        let sender = self
            .inner
            .event_tx
            .read()
            .unwrap_or_else(|p| p.into_inner())
            .clone();
        let children = self
            .inner
            .children
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone();
        for child in children {
            child
                .generation
                .store(generation, Ordering::Release);
            let child_new_root = parent_root.child_token();
            *child
                .generation_root
                .lock()
                .unwrap_or_else(|p| p.into_inner()) = child_new_root.clone();
            if let Some(tx) = &sender {
                *child
                    .event_tx
                    .write()
                    .unwrap_or_else(|p| p.into_inner()) = Some(tx.clone());
            }
            TeamRunManager {
                inner: Arc::clone(&child),
            }
            .renew_child_roots(&child_new_root, generation);
        }
    }

    /// Accept a typed lifecycle event from the synchronous `task` tool. The
    /// run's generation is captured at RunStarted, so a late child event cannot
    /// be mislabeled as belonging to a replacement runtime generation.
    pub fn publish_external(&self, event: TeamEvent) {
        let run_key = event.run_id.to_string();
        let generation = {
            let mut runs = self
                .inner
                .external_runs
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            match &event.payload {
                TeamEventPayload::RunStarted { .. } => {
                    let generation = self.generation();
                    runs.insert(run_key.clone(), generation);
                    generation
                }
                _ => match runs.get(&run_key).copied() {
                    Some(generation) => generation,
                    None => return,
                },
            }
        };
        if generation != self.generation() {
            return;
        }
        let sender = self
            .inner
            .event_tx
            .read()
            .unwrap_or_else(|p| p.into_inner())
            .clone();
        if let Some(sender) = sender {
            let _ = sender.send(GenerationTeamEvent {
                generation,
                event: event.clone(),
            });
        }
        if matches!(event.payload, TeamEventPayload::RunFinished { .. }) {
            self.inner
                .external_runs
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .remove(&run_key);
        }
    }

    pub async fn delegate(
        &self,
        tasks: Vec<TeamTaskSpec>,
        factory: TeamJobFactory,
        models: TeamModelFactory,
    ) -> Result<TeamRunId, String> {
        if self.inner.depth >= self.inner.config.max_depth {
            return Err(format!(
                "team delegation depth limit reached (max_depth={}, current depth={}); \
                 subagents at this depth cannot further dispatch",
                self.inner.config.max_depth, self.inner.depth
            ));
        }
        // F2-MINOR-1: normalize scope like legacy `workers_missing_scope` -- trim
        // each entry and drop empties so a whitespace-only scope fails the
        // non-empty check, and a padded prefix never reaches the path gate.
        let mut tasks = tasks;
        for task in &mut tasks {
            task.scope = task
                .scope
                .iter()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        validate_tasks(&tasks)?;
        let generation = self.generation();
        let root = self
            .inner
            .generation_root
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone();
        let ordinal = self.inner.run_counter.fetch_add(1, Ordering::Relaxed);
        let run_id = TeamRunId::new(format!("team-{generation}-{ordinal}"));
        let seq = Arc::new(AtomicU64::new(1));
        let notify = Arc::new(tokio::sync::Notify::new());
        {
            let mut store = self.lock_store();
            store.runs.insert(
                run_id.to_string(),
                TeamRunState {
                    run_id: run_id.clone(),
                    generation,
                    ordinal,
                    total: tasks.len(),
                    seq: Arc::clone(&seq),
                    notify: Arc::clone(&notify),
                    ..TeamRunState::default()
                },
            );
        }
        self.emit(
            generation,
            &run_id,
            &seq,
            TeamEventPayload::RunStarted { total: tasks.len() },
        );

        // Two independent scheduling lanes: read-only Explore members fan out wider
        // (they cannot conflict), while write Workers stay bounded by the conservative
        // worker budget. A broad investigation sweep therefore is not throttled behind
        // the small write budget, and a worker is never stuck behind a read queue.
        let worker_sem = Arc::new(tokio::sync::Semaphore::new(
            self.inner.config.max_concurrent.max(1),
        ));
        let explore_sem = Arc::new(tokio::sync::Semaphore::new(
            self.inner.config.max_concurrent_explore.max(1),
        ));
        for (index, task) in tasks.into_iter().enumerate() {
            let member_id = TeamMemberId::new(format!("{}#{}", task.role, index + 1));
            let model = models(&task);
            let child_cancel = root.child_token();
            self.insert_member(
                &run_id,
                TeamMemberRuntime {
                    id: member_id.clone(),
                    role: task.role.to_string(),
                    model: model.clone(),
                    description: task.description.clone(),
                    status: TeamMemberStatus::Queued,
                    result: String::new(),
                    output_tokens: 0,
                    cancel: child_cancel.clone(),
                    abort: None,
                },
            )?;
            self.emit(
                generation,
                &run_id,
                &seq,
                TeamEventPayload::MemberQueued {
                    member_id: member_id.clone(),
                    role: task.role,
                    model,
                    description: task.description.clone(),
                },
            );
            let manager = self.clone();
            let run = run_id.clone();
            let member = member_id.clone();
            let semaphore = if task.permission == TeamPermission::Worker {
                Arc::clone(&worker_sem)
            } else {
                Arc::clone(&explore_sem)
            };
            let factory = Arc::clone(&factory);
            let handle = tokio::spawn(async move {
                let permit = tokio::select! {
                    permit = semaphore.acquire_owned() => permit.ok(),
                    _ = child_cancel.cancelled() => None,
                };
                if permit.is_none() {
                    manager.finish_member(
                        &run,
                        &member,
                        TeamMemberOutcome::stopped("stopped before start"),
                    );
                    return;
                }
                if !manager.mark_started(&run, &member) {
                    return;
                }
                let activity_manager = manager.clone();
                let activity_run = run.clone();
                let activity_member = member.clone();
                let activity: TeamActivitySink = Arc::new(move |activity, tokens| {
                    activity_manager.member_activity(
                        &activity_run,
                        &activity_member,
                        activity,
                        tokens,
                    );
                });
                let outcome = tokio::select! {
                    outcome = factory(task, child_cancel.clone(), activity) => outcome,
                    _ = child_cancel.cancelled() => TeamMemberOutcome::stopped("cancelled"),
                };
                manager.finish_member(&run, &member, outcome);
            });
            self.set_abort(&run_id, &member_id, handle.abort_handle())?;
        }
        Ok(run_id)
    }

    pub async fn wait(
        &self,
        run_id: &TeamRunId,
        timeout: Duration,
    ) -> Result<TeamWaitOutcome, String> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            let notify = {
                let store = self.lock_store();
                let run = store
                    .runs
                    .get(run_id.as_str())
                    .ok_or_else(|| format!("unknown team run: {run_id}"))?;
                Arc::clone(&run.notify)
            };
            let notified = notify.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if self.run_is_terminal(run_id)? {
                return Ok(TeamWaitOutcome { terminal: true });
            }
            if tokio::time::timeout_at(deadline, notified).await.is_err() {
                return Ok(TeamWaitOutcome {
                    terminal: self.run_is_terminal(run_id)?,
                });
            }
        }
    }

    pub async fn stop(&self, run_id: &TeamRunId) -> Result<(), String> {
        let members = self.cancel_members(Some(run_id))?;
        self.finish_cancelled_after_grace(members).await;
        Ok(())
    }

    pub async fn stop_all(&self) {
        self.inner
            .generation_root
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .cancel();
        let members = self.cancel_members(None).unwrap_or_default();
        self.finish_cancelled_after_grace(members).await;
        // Cascade: stop every child tier so sub-dispatched members cannot outlive
        // a root `stop_all` (F3-HIGH-2). Children are dropped from the lock before
        // awaiting to avoid holding it across the recursion.
        let children = self
            .inner
            .children
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone();
        for child in children {
            Box::pin(
                TeamRunManager {
                    inner: Arc::clone(&child),
                }
                .stop_all(),
            )
            .await;
        }
    }

    pub fn snapshot(&self, run_id: Option<&TeamRunId>) -> Option<TeamSnapshot> {
        self.lock_store().snapshot(run_id)
    }

    /// Snapshot a single run as a tree node, tagged with this manager's depth.
    /// `children` is empty for leaf managers; a hierarchical driver may populate
    /// it from child managers built via [`TeamRunManager::child_manager`].
    pub fn snapshot_tree(&self, run_id: &TeamRunId) -> Option<TreeSnapshot> {
        let snapshot = self.snapshot(Some(run_id))?;
        let base = snapshot.runs.into_iter().next()?;
        Some(TreeSnapshot {
            base,
            depth: self.inner.depth,
            children: vec![],
        })
    }

    async fn finish_cancelled_after_grace(
        &self,
        members: Vec<(TeamRunId, TeamMemberId, Option<AbortHandle>)>,
    ) {
        if members.is_empty() {
            return;
        }
        let deadline = tokio::time::Instant::now() + self.inner.config.cancel_grace;
        loop {
            let all_terminal = {
                let store = self.lock_store();
                members.iter().all(|(run_id, member_id, _)| {
                    store
                        .runs
                        .get(run_id.as_str())
                        .and_then(|run| run.members.get(member_id.as_str()))
                        .is_none_or(|member| member.status.is_terminal())
                })
            };
            if all_terminal {
                return;
            }
            let now = tokio::time::Instant::now();
            if now >= deadline {
                break;
            }
            tokio::time::sleep(
                deadline
                    .saturating_duration_since(now)
                    .min(Duration::from_millis(10)),
            )
            .await;
        }
        for (run_id, member_id, abort) in members {
            let still_running = {
                let store = self.lock_store();
                store
                    .runs
                    .get(run_id.as_str())
                    .and_then(|run| run.members.get(member_id.as_str()))
                    .is_some_and(|member| !member.status.is_terminal())
            };
            if still_running {
                if let Some(abort) = abort {
                    abort.abort();
                }
                self.finish_member(
                    &run_id,
                    &member_id,
                    TeamMemberOutcome::stopped("aborted after cancellation grace"),
                );
            }
        }
    }

    fn cancel_members(
        &self,
        only: Option<&TeamRunId>,
    ) -> Result<Vec<(TeamRunId, TeamMemberId, Option<AbortHandle>)>, String> {
        let mut store = self.lock_store();
        if let Some(run_id) = only {
            if !store.runs.contains_key(run_id.as_str()) {
                return Err(format!("unknown team run: {run_id}"));
            }
        }
        let mut members = Vec::new();
        for run in store
            .runs
            .values_mut()
            .filter(|run| only.is_none_or(|id| id.as_str() == run.run_id.as_str()))
        {
            for member in run
                .members
                .values_mut()
                .filter(|member| !member.status.is_terminal())
            {
                member.status = TeamMemberStatus::Cancelling;
                member.cancel.cancel();
                members.push((run.run_id.clone(), member.id.clone(), member.abort.clone()));
            }
        }
        Ok(members)
    }

    fn mark_started(&self, run_id: &TeamRunId, member_id: &TeamMemberId) -> bool {
        let (generation, seq, payload) = {
            let mut store = self.lock_store();
            let Some(run) = store.runs.get_mut(run_id.as_str()) else {
                return false;
            };
            let Some(member) = run.members.get_mut(member_id.as_str()) else {
                return false;
            };
            if member.status != TeamMemberStatus::Queued {
                return false;
            }
            member.status = TeamMemberStatus::Running;
            (
                run.generation,
                Arc::clone(&run.seq),
                TeamEventPayload::MemberStarted {
                    member_id: member.id.clone(),
                    role: member
                        .role
                        .parse()
                        .unwrap_or(rustcode_capabilities::team::TeamRoleId::Explorer),
                    model: member.model.clone(),
                    description: member.description.clone(),
                },
            )
        };
        self.emit(generation, run_id, &seq, payload);
        true
    }

    fn member_activity(
        &self,
        run_id: &TeamRunId,
        member_id: &TeamMemberId,
        activity: String,
        output_tokens: u64,
    ) {
        if activity.trim().is_empty() {
            return;
        }
        let (generation, seq) = {
            let mut store = self.lock_store();
            let Some(run) = store.runs.get_mut(run_id.as_str()) else {
                return;
            };
            let Some(member) = run.members.get_mut(member_id.as_str()) else {
                return;
            };
            if member.status != TeamMemberStatus::Running {
                return;
            }
            // Estimates are monotonic within a run; keep the max so a late,
            // reordered activity can't roll the displayed count backward.
            member.output_tokens = member.output_tokens.max(output_tokens);
            (run.generation, Arc::clone(&run.seq))
        };
        self.emit(
            generation,
            run_id,
            &seq,
            TeamEventPayload::MemberActivity {
                member_id: member_id.clone(),
                activity,
                output_tokens,
                // The Team runtime does not yet track per-member tool counts;
                // the `task` fan-out path populates this. 0 here = "unknown".
                tool_uses: 0,
            },
        );
    }

    fn finish_member(
        &self,
        run_id: &TeamRunId,
        member_id: &TeamMemberId,
        outcome: TeamMemberOutcome,
    ) {
        let completed_status = if outcome.stop == "stopped" {
            TeamMemberStatus::Stopped
        } else if outcome.success {
            TeamMemberStatus::Completed
        } else {
            TeamMemberStatus::Failed
        };
        let (generation, seq, member_event, run_event, notify) = {
            let mut store = self.lock_store();
            let Some(run) = store.runs.get_mut(run_id.as_str()) else {
                return;
            };
            let Some(member) = run.members.get_mut(member_id.as_str()) else {
                return;
            };
            if member.status.is_terminal() {
                return;
            }
            member.status = completed_status;
            member.result = truncate_chars(&outcome.output, self.inner.config.max_result_chars);
            // The outcome carries the run's final token total (incl. the closing
            // round that emitted no activity); keep the larger of it and the live max.
            member.output_tokens = member.output_tokens.max(outcome.output_tokens);
            let member_event = TeamEventPayload::MemberFinished {
                member_id: member.id.clone(),
                success: completed_status == TeamMemberStatus::Completed,
                stop: outcome.stop,
                summary: member
                    .result
                    .lines()
                    .next()
                    .unwrap_or_default()
                    .chars()
                    .take(160)
                    .collect(),
                output_tokens: member.output_tokens,
            };
            run.recount();
            let run_event = run.is_terminal().then_some(TeamEventPayload::RunFinished {
                total: run.total,
                completed: run.completed,
                failed: run.failed,
            });
            (
                run.generation,
                Arc::clone(&run.seq),
                member_event,
                run_event,
                Arc::clone(&run.notify),
            )
        };
        self.emit(generation, run_id, &seq, member_event);
        if let Some(event) = run_event {
            self.emit(generation, run_id, &seq, event);
            notify.notify_waiters();
            self.lock_store()
                .prune_completed(self.inner.config.max_completed_runs.max(1));
        }
    }

    fn emit(
        &self,
        generation: u64,
        run_id: &TeamRunId,
        seq: &AtomicU64,
        payload: TeamEventPayload,
    ) {
        if self.generation() != generation {
            return;
        }
        let sender = self
            .inner
            .event_tx
            .read()
            .unwrap_or_else(|p| p.into_inner())
            .clone();
        if let Some(sender) = sender {
            // Assign the seq and send it under one lock so the channel receives
            // events in seq order; otherwise a lower-seq event that lost the send
            // race is dropped by the consumer's monotonic filter. The send is
            // non-blocking (unbounded), so the critical section stays short.
            let _emit_guard = self
                .inner
                .emit_lock
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            let event =
                TeamEvent::new(run_id.clone(), seq.fetch_add(1, Ordering::Relaxed), payload)
                    .with_depth(self.inner.depth);
            let _ = sender.send(GenerationTeamEvent { generation, event });
        }
    }

    fn insert_member(&self, run_id: &TeamRunId, member: TeamMemberRuntime) -> Result<(), String> {
        self.lock_store()
            .runs
            .get_mut(run_id.as_str())
            .ok_or_else(|| format!("unknown team run: {run_id}"))?
            .members
            .insert(member.id.to_string(), member);
        Ok(())
    }

    fn set_abort(
        &self,
        run_id: &TeamRunId,
        member_id: &TeamMemberId,
        abort: AbortHandle,
    ) -> Result<(), String> {
        self.lock_store()
            .runs
            .get_mut(run_id.as_str())
            .and_then(|run| run.members.get_mut(member_id.as_str()))
            .ok_or_else(|| format!("unknown team member: {member_id}"))?
            .abort = Some(abort);
        Ok(())
    }

    fn lock_store(&self) -> MutexGuard<'_, TeamRunStore> {
        self.inner.store.lock().unwrap_or_else(|p| p.into_inner())
    }

    fn run_is_terminal(&self, run_id: &TeamRunId) -> Result<bool, String> {
        self.lock_store()
            .runs
            .get(run_id.as_str())
            .map(TeamRunState::is_terminal)
            .ok_or_else(|| format!("unknown team run: {run_id}"))
    }
}

fn validate_tasks(tasks: &[TeamTaskSpec]) -> Result<(), String> {
    if tasks.is_empty() {
        return Err("team delegate requires at least one task".into());
    }
    for task in tasks {
        let profile = role_by_id(task.role.as_str())
            .ok_or_else(|| format!("unknown team role: {}", task.role))?;
        if task.permission != profile.permission {
            return Err(format!(
                "team role {} requires {:?} permission",
                task.role, profile.permission
            ));
        }
        if task.permission == TeamPermission::Worker
            && task.scope.iter().all(|s| s.trim().is_empty())
        {
            return Err(format!(
                "team worker role {} requires a non-empty scope",
                task.role
            ));
        }
    }
    validate_non_overlapping_worker_scopes(tasks)
}

fn truncate_chars(value: &str, max: usize) -> String {
    if value.chars().count() <= max {
        value.to_string()
    } else {
        value.chars().take(max).collect::<String>() + "..."
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TeamMemberStatus {
    Queued,
    Running,
    Cancelling,
    Completed,
    Failed,
    Stopped,
}
impl TeamMemberStatus {
    fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Stopped)
    }
}

struct TeamMemberRuntime {
    id: TeamMemberId,
    role: String,
    model: String,
    description: String,
    status: TeamMemberStatus,
    result: String,
    output_tokens: u64,
    cancel: CancellationToken,
    abort: Option<AbortHandle>,
}

struct TeamRunState {
    run_id: TeamRunId,
    generation: u64,
    ordinal: u64,
    total: usize,
    completed: usize,
    failed: usize,
    stopped: usize,
    members: BTreeMap<String, TeamMemberRuntime>,
    notify: Arc<tokio::sync::Notify>,
    seq: Arc<AtomicU64>,
}

impl Default for TeamRunState {
    fn default() -> Self {
        Self {
            run_id: TeamRunId::new(""),
            generation: 0,
            ordinal: 0,
            total: 0,
            completed: 0,
            failed: 0,
            stopped: 0,
            members: BTreeMap::new(),
            notify: Arc::new(tokio::sync::Notify::new()),
            seq: Arc::new(AtomicU64::new(1)),
        }
    }
}

impl TeamRunState {
    fn is_terminal(&self) -> bool {
        self.completed + self.failed + self.stopped == self.total
    }
    fn recount(&mut self) {
        self.completed = self
            .members
            .values()
            .filter(|m| m.status == TeamMemberStatus::Completed)
            .count();
        self.failed = self
            .members
            .values()
            .filter(|m| m.status == TeamMemberStatus::Failed)
            .count();
        self.stopped = self
            .members
            .values()
            .filter(|m| m.status == TeamMemberStatus::Stopped)
            .count();
    }
}

#[derive(Default)]
struct TeamRunStore {
    runs: BTreeMap<String, TeamRunState>,
}
impl TeamRunStore {
    fn snapshot(&self, selected: Option<&TeamRunId>) -> Option<TeamSnapshot> {
        let runs = self
            .runs
            .values()
            .filter(|run| selected.is_none_or(|id| id == &run.run_id))
            .map(TeamRunSnapshot::from)
            .collect::<Vec<_>>();
        (!runs.is_empty()).then_some(TeamSnapshot { runs })
    }

    fn prune_completed(&mut self, keep: usize) {
        let mut completed = self
            .runs
            .values()
            .filter(|run| run.is_terminal())
            .map(|run| (run.ordinal, run.run_id.to_string()))
            .collect::<Vec<_>>();
        if completed.len() <= keep {
            return;
        }
        completed.sort_by_key(|(ordinal, _)| *ordinal);
        let remove = completed.len() - keep;
        for (_, run_id) in completed.into_iter().take(remove) {
            self.runs.remove(&run_id);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TeamSnapshot {
    pub runs: Vec<TeamRunSnapshot>,
}

/// A hierarchical snapshot node for tree-style team dispatch. Each node carries
/// the dispatch depth of its originating manager; `children` holds sub-trees
/// produced by deeper child managers (depth+1, depth+2, ...).
#[derive(Debug, Clone)]
pub struct TreeSnapshot {
    pub base: TeamRunSnapshot,
    pub depth: u8,
    pub children: Vec<TreeSnapshot>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TeamRunSnapshot {
    pub run_id: TeamRunId,
    pub generation: u64,
    pub total: usize,
    pub completed: usize,
    pub failed: usize,
    pub stopped: usize,
    pub members: Vec<TeamMemberSnapshot>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TeamMemberSnapshot {
    pub id: TeamMemberId,
    pub role: String,
    pub status: TeamMemberStatus,
    pub result: String,
    pub output_tokens: u64,
}

impl From<&TeamRunState> for TeamRunSnapshot {
    fn from(run: &TeamRunState) -> Self {
        Self {
            run_id: run.run_id.clone(),
            generation: run.generation,
            total: run.total,
            completed: run.completed,
            failed: run.failed,
            stopped: run.stopped,
            members: run
                .members
                .values()
                .map(|member| TeamMemberSnapshot {
                    id: member.id.clone(),
                    role: member.role.clone(),
                    status: member.status,
                    result: member.result.clone(),
                    output_tokens: member.output_tokens,
                })
                .collect(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TeamWaitOutcome {
    pub terminal: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustcode_capabilities::team::{TeamDifficulty, TeamRoleId};

    fn task(role: TeamRoleId, permission: TeamPermission, scope: Vec<String>) -> TeamTaskSpec {
        TeamTaskSpec {
            description: "work".into(),
            prompt: "do work".into(),
            role,
            permission,
            difficulty: TeamDifficulty::Simple,
            scope,
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn read_only_lane_fans_out_independently_of_worker_budget() {
        // Worker lane is pinned to 1, read-only lane to 4. Four explorers share a
        // barrier that only releases when ALL FOUR run at once. If read-only members
        // wrongly shared the worker semaphore (1 permit), this deadlocks and the
        // outer timeout fires -- proving the two lanes are scheduled independently.
        let manager = TeamRunManager::new(TeamRuntimeConfig {
            max_concurrent: 1,
            max_concurrent_explore: 4,
            cancel_grace: Duration::from_millis(10),
            ..TeamRuntimeConfig::default()
        });
        manager.begin_generation(1);
        let barrier = Arc::new(tokio::sync::Barrier::new(4));
        let factory: TeamJobFactory = Arc::new(move |_task, _cancel, _activity| {
            let barrier = Arc::clone(&barrier);
            Box::pin(async move {
                tokio::time::timeout(Duration::from_secs(10), barrier.wait())
                    .await
                    .expect("explorers must run concurrently on the read-only lane");
                TeamMemberOutcome::completed("done")
            })
        });
        let models: TeamModelFactory = Arc::new(|_| "fast".to_string());
        let tasks: Vec<TeamTaskSpec> = (0..4)
            .map(|_| task(TeamRoleId::Explorer, TeamPermission::Explore, vec![]))
            .collect();
        let run = manager.delegate(tasks, factory, models).await.unwrap();
        let wait = tokio::time::timeout(
            Duration::from_secs(20),
            manager.wait(&run, Duration::from_secs(30)),
        )
        .await
        .expect("read-only team run did not finish in time")
        .unwrap();
        assert!(wait.terminal);
        let snap = manager.snapshot(Some(&run)).unwrap();
        let run_snap = &snap.runs[0];
        assert_eq!(run_snap.completed, 4);
        assert_eq!(run_snap.failed + run_snap.stopped, 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn worker_lane_is_bounded_independently_of_read_lane() {
        // Read-only lane is wide (8) but the worker lane is 1: three workers must
        // never overlap, even though they try to. The semaphore makes this exact.
        let manager = TeamRunManager::new(TeamRuntimeConfig {
            max_concurrent: 1,
            max_concurrent_explore: 8,
            cancel_grace: Duration::from_millis(10),
            ..TeamRuntimeConfig::default()
        });
        manager.begin_generation(1);
        let active = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let peak = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let peak_out = Arc::clone(&peak);
        let factory: TeamJobFactory = Arc::new(move |_task, _cancel, _activity| {
            let active = Arc::clone(&active);
            let peak = Arc::clone(&peak);
            Box::pin(async move {
                let now = active.fetch_add(1, Ordering::SeqCst) + 1;
                peak.fetch_max(now, Ordering::SeqCst);
                tokio::time::sleep(Duration::from_millis(20)).await;
                active.fetch_sub(1, Ordering::SeqCst);
                TeamMemberOutcome::completed("edited")
            })
        });
        let models: TeamModelFactory = Arc::new(|_| "cap".to_string());
        let tasks: Vec<TeamTaskSpec> = (1..=3)
            .map(|i| {
                task(
                    TeamRoleId::Rust,
                    TeamPermission::Worker,
                    vec![format!("w{i}/**")],
                )
            })
            .collect();
        let run = manager.delegate(tasks, factory, models).await.unwrap();
        manager.wait(&run, Duration::from_secs(30)).await.unwrap();
        let snap = manager.snapshot(Some(&run)).unwrap();
        assert_eq!(snap.runs[0].completed, 3);
        assert_eq!(
            peak_out.load(Ordering::SeqCst),
            1,
            "write workers must be serialized by the worker lane"
        );
    }

    fn manager() -> TeamRunManager {
        let manager = TeamRunManager::new(TeamRuntimeConfig {
            cancel_grace: Duration::from_millis(10),
            ..TeamRuntimeConfig::default()
        });
        manager.begin_generation(7);
        manager
    }

    fn models() -> TeamModelFactory {
        Arc::new(|_| "test-model".to_string())
    }

    #[test]
    fn worker_scopes_reject_identical_file_lane() {
        let tasks = vec![
            task(
                TeamRoleId::Implementer,
                TeamPermission::Worker,
                vec!["src/lib.rs".into()],
            ),
            task(
                TeamRoleId::Implementer,
                TeamPermission::Worker,
                vec!["./src/lib.rs".into()],
            ),
        ];
        let error = validate_tasks(&tasks).unwrap_err();
        assert!(error.contains("worker scopes overlap"), "{error}");
    }

    #[test]
    fn worker_scope_rejects_whitespace_only_lane() {
        // F2-MINOR-1: whitespace-only scope must fail the non-empty check,
        // matching legacy `workers_missing_scope` trim semantics.
        let tasks = vec![task(
            TeamRoleId::Implementer,
            TeamPermission::Worker,
            vec!["  ".into(), "\t".into()],
        )];
        let error = validate_tasks(&tasks).unwrap_err();
        assert!(error.contains("non-empty scope"), "{error}");
    }

    #[test]
    fn worker_scopes_reject_recursive_parent_lane() {
        let tasks = vec![
            task(
                TeamRoleId::Implementer,
                TeamPermission::Worker,
                vec!["src/**".into()],
            ),
            task(
                TeamRoleId::Implementer,
                TeamPermission::Worker,
                vec!["src/auth/login.rs".into()],
            ),
        ];
        let error = validate_tasks(&tasks).unwrap_err();
        assert!(error.contains("worker scopes overlap"), "{error}");
    }

    #[test]
    fn worker_scopes_allow_disjoint_file_lanes() {
        let tasks = vec![
            task(
                TeamRoleId::Implementer,
                TeamPermission::Worker,
                vec!["src/a.rs".into()],
            ),
            task(
                TeamRoleId::Implementer,
                TeamPermission::Worker,
                vec!["src/b.rs".into()],
            ),
        ];
        validate_tasks(&tasks).unwrap();
    }

    #[tokio::test]
    async fn delegate_completes_each_member_once() {
        let manager = manager();
        let factory: TeamJobFactory =
            Arc::new(|_, _, _| Box::pin(async { TeamMemberOutcome::completed("done") }));
        let run = manager
            .delegate(
                vec![task(TeamRoleId::Explorer, TeamPermission::Explore, vec![])],
                factory,
                models(),
            )
            .await
            .unwrap();
        assert!(
            manager
                .wait(&run, Duration::from_secs(1))
                .await
                .unwrap()
                .terminal
        );
        let snap = manager.snapshot(Some(&run)).unwrap();
        assert_eq!(snap.runs[0].completed, 1);
        assert_eq!(snap.runs[0].failed + snap.runs[0].stopped, 0);
    }

    #[tokio::test]
    async fn wait_observes_terminal_state_without_missing_completion() {
        let manager = manager();
        let release = Arc::new(tokio::sync::Notify::new());
        let child_release = Arc::clone(&release);
        let factory: TeamJobFactory = Arc::new(move |_, _, _| {
            let release = Arc::clone(&child_release);
            Box::pin(async move {
                release.notified().await;
                TeamMemberOutcome::completed("done")
            })
        });
        let run = manager
            .delegate(
                vec![task(TeamRoleId::Explorer, TeamPermission::Explore, vec![])],
                factory,
                models(),
            )
            .await
            .unwrap();
        let waiting = {
            let manager = manager.clone();
            let run = run.clone();
            tokio::spawn(async move { manager.wait(&run, Duration::from_secs(1)).await })
        };
        tokio::task::yield_now().await;
        release.notify_waiters();
        assert!(waiting.await.unwrap().unwrap().terminal);
    }

    #[tokio::test]
    async fn completed_run_retention_is_bounded_and_keeps_running_work() {
        let manager = TeamRunManager::new(TeamRuntimeConfig {
            max_completed_runs: 2,
            ..TeamRuntimeConfig::default()
        });
        manager.begin_generation(7);
        let factory: TeamJobFactory =
            Arc::new(|_, _, _| Box::pin(async { TeamMemberOutcome::completed("done") }));
        let mut runs = Vec::new();
        for _ in 0..3 {
            let run = manager
                .delegate(
                    vec![task(TeamRoleId::Explorer, TeamPermission::Explore, vec![])],
                    Arc::clone(&factory),
                    models(),
                )
                .await
                .unwrap();
            assert!(
                manager
                    .wait(&run, Duration::from_secs(1))
                    .await
                    .unwrap()
                    .terminal
            );
            runs.push(run);
        }
        let snapshot = manager.snapshot(None).unwrap();
        assert_eq!(snapshot.runs.len(), 2);
        assert!(manager.snapshot(Some(&runs[0])).is_none());
        assert!(manager.snapshot(Some(&runs[1])).is_some());
        assert!(manager.snapshot(Some(&runs[2])).is_some());
    }

    #[tokio::test]
    async fn started_event_carries_model_and_child_activity_is_forwarded() {
        let manager = manager();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        manager.set_event_sender(tx);
        let factory: TeamJobFactory = Arc::new(|_, _, activity| {
            Box::pin(async move {
                activity("using read_file".to_string(), 512);
                TeamMemberOutcome::completed("done")
            })
        });
        let run = manager
            .delegate(
                vec![task(TeamRoleId::Explorer, TeamPermission::Explore, vec![])],
                factory,
                models(),
            )
            .await
            .unwrap();
        assert!(
            manager
                .wait(&run, Duration::from_secs(1))
                .await
                .unwrap()
                .terminal
        );

        let events = std::iter::from_fn(|| rx.try_recv().ok()).collect::<Vec<_>>();
        assert!(events.iter().any(|event| matches!(
            &event.event.payload,
            TeamEventPayload::MemberQueued { model, .. } if model == "test-model"
        )));
        assert!(events.iter().any(|event| matches!(
            &event.event.payload,
            TeamEventPayload::MemberStarted { model, .. } if model == "test-model"
        )));
        assert!(events.iter().any(|event| matches!(
            &event.event.payload,
            TeamEventPayload::MemberActivity { activity, output_tokens, .. }
                if activity == "using read_file" && *output_tokens == 512
        )));
        // The final event carries the accumulated token estimate.
        assert!(events.iter().any(|event| matches!(
            &event.event.payload,
            TeamEventPayload::MemberFinished { output_tokens, .. } if *output_tokens == 512
        )));
    }

    #[tokio::test]
    async fn stop_all_cancels_and_forces_uncooperative_members() {
        let manager = manager();
        let factory: TeamJobFactory = Arc::new(|_, _, _| Box::pin(std::future::pending()));
        let run = manager
            .delegate(
                vec![task(TeamRoleId::Explorer, TeamPermission::Explore, vec![])],
                factory,
                models(),
            )
            .await
            .unwrap();
        tokio::task::yield_now().await;
        manager.stop_all().await;
        assert!(
            manager
                .wait(&run, Duration::from_secs(1))
                .await
                .unwrap()
                .terminal
        );
        assert_eq!(manager.snapshot(Some(&run)).unwrap().runs[0].stopped, 1);
    }

    #[tokio::test]
    async fn generation_change_suppresses_late_events() {
        let manager = manager();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        manager.set_event_sender(tx);
        let gate = Arc::new(tokio::sync::Notify::new());
        let wait = Arc::clone(&gate);
        let factory: TeamJobFactory = Arc::new(move |_, _, _| {
            let wait = Arc::clone(&wait);
            Box::pin(async move {
                wait.notified().await;
                TeamMemberOutcome::completed("late")
            })
        });
        let run = manager
            .delegate(
                vec![task(TeamRoleId::Explorer, TeamPermission::Explore, vec![])],
                factory,
                models(),
            )
            .await
            .unwrap();
        tokio::task::yield_now().await;
        while rx.try_recv().is_ok() {}
        manager.begin_generation(8);
        gate.notify_waiters();
        manager.stop_all().await;
        assert!(rx.try_recv().is_err());
        let run = &manager.snapshot(Some(&run)).unwrap().runs[0];
        assert_eq!(run.completed + run.failed + run.stopped, 1);
    }

    #[test]
    fn external_task_events_keep_their_start_generation() {
        let manager = manager();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        manager.set_event_sender(tx);
        let run = TeamRunId::new("task-1");
        manager.publish_external(TeamEvent::new(
            run.clone(),
            1,
            TeamEventPayload::RunStarted { total: 1 },
        ));
        let initial_generation = manager.generation();
        assert_eq!(rx.try_recv().unwrap().generation, initial_generation);

        manager.begin_generation(initial_generation + 1);
        manager.publish_external(TeamEvent::new(
            run,
            2,
            TeamEventPayload::RunFinished {
                total: 1,
                completed: 1,
                failed: 0,
            },
        ));
        assert!(
            rx.try_recv().is_err(),
            "late external event must be dropped"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_member_events_arrive_in_strict_seq_order() {
        // Regression guard for the emit reorder: many members share one per-run seq
        // counter and emit concurrently. The receiver must observe events in seq
        // order, or the consumer's `seq <= previous` filter drops the loser forever.
        let manager = TeamRunManager::new(TeamRuntimeConfig {
            max_concurrent: 16,
            cancel_grace: Duration::from_millis(10),
            ..TeamRuntimeConfig::default()
        });
        manager.begin_generation(7);
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        manager.set_event_sender(tx);
        let factory: TeamJobFactory = Arc::new(|_, _, activity| {
            Box::pin(async move {
                for i in 0..20 {
                    activity(format!("step {i}"), (i as u64 + 1) * 4);
                }
                TeamMemberOutcome::completed("done")
            })
        });
        let tasks = (0..12)
            .map(|_| task(TeamRoleId::Explorer, TeamPermission::Explore, vec![]))
            .collect::<Vec<_>>();
        let run = manager.delegate(tasks, factory, models()).await.unwrap();
        assert!(
            manager
                .wait(&run, Duration::from_secs(5))
                .await
                .unwrap()
                .terminal
        );

        let mut last: BTreeMap<String, u64> = BTreeMap::new();
        while let Ok(event) = rx.try_recv() {
            let previous = last.entry(event.event.run_id.to_string()).or_insert(0);
            assert!(
                event.event.seq > *previous,
                "out-of-order seq {} arrived after {}",
                event.event.seq,
                *previous
            );
            *previous = event.event.seq;
        }
    }

    #[tokio::test]
    async fn worker_requires_scope() {
        let manager = manager();
        let factory: TeamJobFactory =
            Arc::new(|_, _, _| Box::pin(async { TeamMemberOutcome::completed("unused") }));
        let error = manager
            .delegate(
                vec![task(
                    TeamRoleId::Implementer,
                    TeamPermission::Worker,
                    vec![],
                )],
                factory,
                models(),
            )
            .await
            .unwrap_err();
        assert!(error.contains("non-empty scope"));
    }

    #[tokio::test]
    async fn delegate_rejects_empty_tasks() {
        let manager = manager();
        let factory: TeamJobFactory =
            Arc::new(|_, _, _| Box::pin(async { TeamMemberOutcome::completed("unused") }));
        let error = manager
            .delegate(vec![], factory, models())
            .await
            .unwrap_err();
        assert!(error.contains("at least one task"));
    }

    #[tokio::test]
    async fn delegate_rejects_permission_mismatch() {
        let manager = manager();
        let factory: TeamJobFactory =
            Arc::new(|_, _, _| Box::pin(async { TeamMemberOutcome::completed("unused") }));
        // Explorer 角色必须用 Explore 权限；错配 Worker 权限应被拒绝。
        let mut spec = task(TeamRoleId::Explorer, TeamPermission::Explore, vec![]);
        spec.permission = TeamPermission::Worker;
        let error = manager
            .delegate(vec![spec], factory, models())
            .await
            .unwrap_err();
        assert!(error.contains("requires"));
    }

    #[tokio::test]
    async fn stop_cancels_only_the_target_run() {
        let manager = manager();
        let factory: TeamJobFactory = Arc::new(|_, _, _| Box::pin(std::future::pending()));
        let run_a = manager
            .delegate(
                vec![task(TeamRoleId::Explorer, TeamPermission::Explore, vec![])],
                Arc::clone(&factory),
                models(),
            )
            .await
            .unwrap();
        let run_b = manager
            .delegate(
                vec![task(TeamRoleId::Explorer, TeamPermission::Explore, vec![])],
                Arc::clone(&factory),
                models(),
            )
            .await
            .unwrap();
        tokio::task::yield_now().await;
        manager.stop(&run_a).await.unwrap();
        assert!(
            manager
                .wait(&run_a, Duration::from_secs(1))
                .await
                .unwrap()
                .terminal
        );
        // run_b 不受影响，仍在运行（非终态）。
        let snap_b = manager.snapshot(Some(&run_b)).unwrap();
        assert!(
            snap_b.runs[0].completed + snap_b.runs[0].failed + snap_b.runs[0].stopped
                < snap_b.runs[0].total
        );
        manager.stop_all().await; // 清理
    }

    #[tokio::test]
    async fn wait_times_out_without_terminal() {
        let manager = manager();
        let factory: TeamJobFactory = Arc::new(|_, _, _| Box::pin(std::future::pending()));
        let run = manager
            .delegate(
                vec![task(TeamRoleId::Explorer, TeamPermission::Explore, vec![])],
                factory,
                models(),
            )
            .await
            .unwrap();
        let outcome = manager.wait(&run, Duration::from_millis(20)).await.unwrap();
        assert!(!outcome.terminal);
        manager.stop_all().await; // 清理
    }

    #[tokio::test]
    async fn wait_unknown_run_errors() {
        let manager = manager();
        let error = manager
            .wait(&TeamRunId::new("missing"), Duration::from_millis(1))
            .await
            .unwrap_err();
        assert!(error.contains("unknown team run"));
    }

    #[tokio::test]
    async fn finish_member_is_idempotent() {
        let manager = manager();
        let factory: TeamJobFactory =
            Arc::new(|_, _, _| Box::pin(async { TeamMemberOutcome::completed("done") }));
        let run = manager
            .delegate(
                vec![task(TeamRoleId::Explorer, TeamPermission::Explore, vec![])],
                factory,
                models(),
            )
            .await
            .unwrap();
        assert!(
            manager
                .wait(&run, Duration::from_secs(1))
                .await
                .unwrap()
                .terminal
        );
        // 已终态成员再次 finish：不得改变计数或状态。
        let member = manager.snapshot(Some(&run)).unwrap().runs[0].members[0].clone();
        manager.finish_member(&run, &member.id, TeamMemberOutcome::failed("dup"));
        let snap = manager.snapshot(Some(&run)).unwrap();
        assert_eq!(snap.runs[0].completed, 1);
        assert_eq!(snap.runs[0].failed, 0);
    }

    #[tokio::test]
    async fn activity_ignored_outside_running() {
        let manager = manager();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        manager.set_event_sender(tx);
        let factory: TeamJobFactory =
            Arc::new(|_, _, _| Box::pin(async { TeamMemberOutcome::completed("done") }));
        let run = manager
            .delegate(
                vec![task(TeamRoleId::Explorer, TeamPermission::Explore, vec![])],
                factory,
                models(),
            )
            .await
            .unwrap();
        assert!(
            manager
                .wait(&run, Duration::from_secs(1))
                .await
                .unwrap()
                .terminal
        );
        while rx.try_recv().is_ok() {} // 排空
                                       // 成员已终态（Completed），activity 必须被忽略：不发布事件、不更新 token。
        let member = manager.snapshot(Some(&run)).unwrap().runs[0].members[0].clone();
        manager.member_activity(&run, &member.id, "late".into(), 999);
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn truncate_chars_honors_max_result_chars() {
        assert_eq!(truncate_chars("abc", 5), "abc");
        assert_eq!(truncate_chars("abcdef", 5), "abcde...");
        assert_eq!(truncate_chars("你好世界", 3), "你好世...");
    }

    #[test]
    fn publish_external_runfinished_cleans_map() {
        let manager = manager();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        manager.set_event_sender(tx);
        let run = TeamRunId::new("task-1");
        manager.publish_external(TeamEvent::new(
            run.clone(),
            1,
            TeamEventPayload::RunStarted { total: 1 },
        ));
        assert!(rx.try_recv().is_ok());
        manager.publish_external(TeamEvent::new(
            run.clone(),
            2,
            TeamEventPayload::RunFinished {
                total: 1,
                completed: 1,
                failed: 0,
            },
        ));
        assert!(rx.try_recv().is_ok());
        // RunFinished 已清理 external_runs 映射：后续非 RunStarted 事件应被丢弃。
        manager.publish_external(TeamEvent::new(
            run,
            3,
            TeamEventPayload::RunFinished {
                total: 1,
                completed: 1,
                failed: 0,
            },
        ));
        assert!(rx.try_recv().is_err());
    }

    /// Build a manager chain of length max_depth+1 via `child_manager`, mirroring
    /// the production assembly in `parts.rs`.
    fn build_chain(max_depth: u8) -> Vec<TeamRunManager> {
        let cfg = TeamRuntimeConfig {
            max_depth,
            ..TeamRuntimeConfig::default()
        };
        let mut chain = vec![TeamRunManager::new(cfg)];
        while let Some(c) = chain.last().unwrap().child_manager() {
            chain.push(c);
        }
        chain
    }

    #[test]
    fn manager_chain_caps_dispatch_depth() {
        // F3-HIGH-1: the manager chain is exactly max_depth+1 long, and the deepest
        // tier cannot spawn a deeper one -- dispatch depth increments strictly so
        // there is no infinite fan-out.
        let chain = build_chain(2);
        assert_eq!(chain.len(), 3, "chain length must equal max_depth + 1");
        assert!(
            chain.last().unwrap().child_manager().is_none(),
            "deepest manager must not fan out further"
        );
    }

    #[test]
    fn begin_generation_cascades_to_children() {
        // F3-HIGH-2: a generation switch on the root re-derives every child's
        // generation, so stale-tier members are invalidated across the whole tree.
        let chain = build_chain(2);
        assert_eq!(chain[1].generation(), 0);
        assert_eq!(chain[2].generation(), 0);
        chain[0].begin_generation(42);
        assert_eq!(chain[0].generation(), 42);
        assert_eq!(chain[1].generation(), 42);
        assert_eq!(chain[2].generation(), 42);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn deepest_manager_rejects_further_dispatch() {
        // F3-HIGH-1: the deepest tier's `delegate` is blocked by the max_depth
        // guard even though its members carry a `team` tool.
        let chain = build_chain(2);
        let deepest = chain.last().unwrap().clone();
        let factory: TeamJobFactory =
            Arc::new(|_, _, _| Box::pin(async { TeamMemberOutcome::completed("ok") }));
        let models: TeamModelFactory = Arc::new(|_| "m".to_string());
        let err = deepest
            .delegate(
                vec![task(TeamRoleId::Explorer, TeamPermission::Explore, vec![])],
                factory,
                models,
            )
            .await
            .unwrap_err();
        assert!(err.contains("depth limit reached"), "{err}");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn stop_all_cascades_to_child_members() {
        // F3-HIGH-2: cancelling the root also stops members dispatched on a child
        // tier -- a child member must never outlive a root stop_all.
        let chain = build_chain(2);
        let root = &chain[0];
        let child = &chain[1];
        let factory: TeamJobFactory = Arc::new(|_, cancel, _| {
            Box::pin(async move {
                cancel.cancelled().await;
                TeamMemberOutcome::stopped("cancelled")
            })
        });
        let models: TeamModelFactory = Arc::new(|_| "m".to_string());
        let run_id = child
            .delegate(
                vec![task(TeamRoleId::Explorer, TeamPermission::Explore, vec![])],
                factory,
                models,
            )
            .await
            .unwrap();
        tokio::task::yield_now().await;
        root.stop_all().await;
        let terminal = child
            .wait(&run_id, Duration::from_secs(5))
            .await
            .unwrap()
            .terminal;
        assert!(
            terminal,
            "child run must become terminal after root stop_all cascades"
        );
    }
}
