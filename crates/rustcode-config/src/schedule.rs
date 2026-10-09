use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Schedule {
    Daily { time: String },               // "HH:MM"
    Weekly { weekday: u8, time: String }, // weekday 1..=7 (1=Mon)
    Hourly,
    Interval { every_minutes: u32 },
    Cron { expr: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduleTask {
    pub id: String,
    pub title: String,
    pub prompt: String,
    pub cwd: String,
    pub schedule: Schedule,
    #[serde(default = "default_mode")]
    pub permission_mode: String, // "plan" | "accept_edits" | "auto"
    #[serde(default = "default_notify")]
    pub notify: String, // "off" | "important" | "all"
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub last_run_at: Option<i64>,
    #[serde(default)]
    pub last_status: Option<String>,
    /// Most recent run's id (key into the task's `runs/` ledger). `#[serde(default)]`
    /// so existing on-disk task JSON without the field still parses unchanged.
    #[serde(default)]
    pub last_run_id: Option<String>,
    /// Ids of tasks that must have succeeded before this one may fire — the
    /// P2.5 static DAG. Empty (the default) means "no dependencies", so every
    /// task written before this field existed behaves exactly as it does today:
    /// the graph is strictly opt-in.
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Event names that fire this task in addition to its own schedule (P2.5).
    /// Empty means "schedule only". An event-driven task with no `schedule` is
    /// pure event trigger; see [`tasks_triggered_by`].
    #[serde(default)]
    pub triggers: Vec<String>,
}

fn default_mode() -> String {
    "plan".into()
}
fn default_notify() -> String {
    "important".into()
}
fn default_true() -> bool {
    true
}

pub fn schedules_root() -> PathBuf {
    crate::config::Config::config_dir().join("schedules")
}

/// A task id is used verbatim to build its on-disk `<id>.json` path, so it must
/// not be able to escape the schedules directory. Ids minted by `build_task`
/// (slug + uuid) are always in-shape, but `load`/`remove`/`enable`/`disable`/`run`
/// take an id straight from an untrusted CLI argument. Accept only a conservative
/// filename-safe alphabet and explicitly reject path separators and `..`.
fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id != ".."
        && !id.contains("..")
        && !id.contains('/')
        && !id.contains('\\')
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

fn task_path_in(root: &std::path::Path, id: &str) -> std::io::Result<PathBuf> {
    if !valid_id(id) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("invalid scheduled task id {id:?}"),
        ));
    }
    Ok(root.join(format!("{id}.json")))
}

fn save_in(root: &std::path::Path, task: &ScheduleTask) -> std::io::Result<()> {
    let path = task_path_in(root, &task.id)?;
    std::fs::create_dir_all(root)?;
    let bytes = serde_json::to_vec_pretty(task)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, bytes)
}

fn load_in(root: &std::path::Path, id: &str) -> std::io::Result<ScheduleTask> {
    let bytes = std::fs::read(task_path_in(root, id)?)?;
    serde_json::from_slice(&bytes)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

fn list_in(root: &std::path::Path) -> Vec<ScheduleTask> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(root) else {
        return out;
    };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        if let Ok(bytes) = std::fs::read(&p) {
            if let Ok(t) = serde_json::from_slice::<ScheduleTask>(&bytes) {
                out.push(t); // corrupt files are skipped
            }
        }
    }
    out.sort_by_key(|a| a.created_at);
    out
}

fn remove_in(root: &std::path::Path, id: &str) -> std::io::Result<()> {
    match std::fs::remove_file(task_path_in(root, id)?) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn save(task: &ScheduleTask) -> std::io::Result<()> {
    save_in(&schedules_root(), task)
}
pub fn load(id: &str) -> std::io::Result<ScheduleTask> {
    load_in(&schedules_root(), id)
}
pub fn list() -> Vec<ScheduleTask> {
    list_in(&schedules_root())
}
pub fn remove(id: &str) -> std::io::Result<()> {
    remove_in(&schedules_root(), id)
}

// ─────────────────────────── wakeup registry (P3) ───────────────────────────
//
// Persists "the next execution intent" so a `schedule_wakeup` survives a process
// restart. Unlike the in-process `tokio::spawn` timer (the fast path in
// `runtime.rs`), a registered wakeup is durable: on restart the daemon tick
// claims due wakeups and fires them as runs. Lives under
// `<schedules_root>/wakeups/<id>.json`, a sibling of the schedule/run/im trees,
// reusing the same `valid_id` guard and file layout as tasks.

pub fn wakeups_root() -> std::path::PathBuf {
    schedules_root().join("wakeups")
}

/// Monotonic "runtime generation" that binds a [`ScheduledWakeup`] to the runtime
/// instance that created it. On the next tick, a wakeup whose `generation` does
/// not match the current one is stale (its creating runtime was replaced) and is
/// skipped so it can never fire. The counter is persisted beside the schedule
/// store so the registering runtime and the `schedule tick` subprocess observe
/// the same value. Defaults to 0 when absent, which keeps `generation: None`
/// wakeups (legacy / unbound) claimable for backward compatibility.
pub fn current_generation() -> u64 {
    generation_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0)
}

/// Advance and persist the runtime generation. Called once per coding runtime
/// instance start (see `ScheduleWakeupTool`); returns the new value.
pub fn bump_generation() -> u64 {
    let next = current_generation().saturating_add(1);
    if let Some(p) = generation_path() {
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(&p, next.to_string());
    }
    next
}

fn generation_path() -> Option<std::path::PathBuf> {
    Some(schedules_root().join("runtime_generation"))
}

/// A persisted "resume the loop with this prompt at/after `due_at`" intent.
///
/// `due_at` is an absolute epoch-seconds timestamp (NOT a relative delay): a
/// relative delay is meaningless once the process exits, whereas `due_at`
/// compares directly against wall-clock time on recovery. `generation` binds the
/// wakeup to the runtime generation that created it so a stale wakeup from a
/// replaced runtime can be dropped without side effects (see design §6).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduledWakeup {
    pub id: String,
    /// Task this wakeup is tied to (P3). `None` when the wakeup was produced
    /// outside a `schedule_task`-driven `/loop` (or before the `task_id`
    /// injection chain was wired); such wakeups degrade to consume-only on
    /// redemption (cannot start a run without a task context).
    #[serde(default)]
    pub task_id: Option<String>,
    pub due_at: i64,
    pub prompt: String,
    pub reason: String,
    pub created_at: i64,
    #[serde(default)]
    pub generation: Option<u64>,
    #[serde(default)]
    pub consumed: bool,
}

/// Mint a directory-safe id for a wakeup (lexical order tracks creation order),
/// mirroring [`mint_run_id`].
pub fn mint_wakeup_id(started_at_epoch_secs: i64, subsec_nanos: u32) -> String {
    format!("{started_at_epoch_secs}-{subsec_nanos:09}")
}

/// Persist a wakeup. `wakeup.id` must pass [`valid_id`]; callers mint it via
/// [`mint_wakeup_id`]. Best-effort like the rest of the store: a failure here
/// must never fail the turn that scheduled the wakeup.
pub fn register_wakeup(wakeup: &ScheduledWakeup) -> std::io::Result<()> {
    register_wakeup_in(&wakeups_root(), wakeup)
}

fn register_wakeup_in(root: &std::path::Path, wakeup: &ScheduledWakeup) -> std::io::Result<()> {
    let path = task_path_in(root, &wakeup.id)?;
    std::fs::create_dir_all(root)?;
    let bytes = serde_json::to_vec_pretty(wakeup)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, bytes)
}

fn list_wakeups_in(root: &std::path::Path) -> Vec<ScheduledWakeup> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(root) else {
        return out;
    };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        if let Ok(bytes) = std::fs::read(&p) {
            if let Ok(w) = serde_json::from_slice::<ScheduledWakeup>(&bytes) {
                out.push(w); // corrupt files are skipped
            }
        }
    }
    out.sort_by_key(|w| w.created_at);
    out
}

/// Wakeups whose `due_at` has arrived and that are not yet consumed. The caller
/// decides whether to fire them (subject to the catch-up window) and must call
/// [`consume_wakeup`] after a successful fire to guarantee exactly-once delivery
/// across concurrent ticks (defense in depth on top of the run ledger's
/// single-flight lock).
pub fn claim_due_wakeups(now_epoch_secs: i64, current_generation: u64) -> Vec<ScheduledWakeup> {
    claim_due_wakeups_in(&wakeups_root(), now_epoch_secs, current_generation)
}

fn claim_due_wakeups_in(
    root: &std::path::Path,
    now_epoch_secs: i64,
    current_generation: u64,
) -> Vec<ScheduledWakeup> {
    list_wakeups_in(root)
        .into_iter()
        .filter(|w| !w.consumed && w.due_at <= now_epoch_secs)
        // Drop wakeups from a previous runtime generation: they were created by a
        // runtime that has since been replaced, so firing them would pollute the
        // current session. Legacy `None` wakeups are unbound and always claimed.
        .filter(|w| match w.generation {
            None => true,
            Some(g) => g == current_generation,
        })
        .collect()
}

/// Every registered wakeup (consumed or not), for status display and recovery.
pub fn list_wakeups() -> Vec<ScheduledWakeup> {
    list_wakeups_in(&wakeups_root())
}

/// Mark a wakeup consumed (single delivery). Best-effort: a missing/corrupt file
/// is treated as already-consumed and reported as success so a tick never loops
/// on a dead entry.
pub fn consume_wakeup(id: &str) -> std::io::Result<()> {
    consume_wakeup_in(&wakeups_root(), id)
}

fn consume_wakeup_in(root: &std::path::Path, id: &str) -> std::io::Result<()> {
    let bytes = match std::fs::read(task_path_in(root, id)?) {
        Ok(b) => b,
        Err(_) => return Ok(()),
    };
    let mut w: ScheduledWakeup = match serde_json::from_slice(&bytes) {
        Ok(w) => w,
        Err(_) => return Ok(()),
    };
    if w.consumed {
        return Ok(());
    }
    w.consumed = true;
    let path = task_path_in(root, id)?;
    let bytes = serde_json::to_vec_pretty(&w)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, bytes)
}

/// Best-effort cleanup: drop consumed wakeups older than `older_than_epoch_secs`
/// so the registry does not grow without bound. Never fails the caller.
pub fn prune_wakeups(older_than_epoch_secs: i64) -> std::io::Result<()> {
    let root = wakeups_root();
    for w in list_wakeups_in(&root) {
        if w.consumed && w.due_at < older_than_epoch_secs {
            let _ = remove_in(&root, &w.id);
        }
    }
    Ok(())
}

// ─────────────────────────── run ledger (P1) ───────────────────────────

/// How many run records a task keeps by default when nothing else is configured.
/// Pruning is caller-driven (`prune_runs(keep)`) so tests and future config can
/// pick their own bound.
pub const DEFAULT_MAX_RUN_HISTORY: usize = 20;

/// State of one recorded run. `Running` is the transient state written before
/// the task actually starts; a record still `Running` on disk means the process
/// died before reaching a terminal state (crash / power loss) -- consumers
/// should treat it as "outcome unknown", not as success.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Running,
    Success,
    Error,
    Cancelled,
    /// The run was deliberately not executed: it was already in flight
    /// elsewhere, or its scheduled instant fell outside the catch-up window.
    /// A terminal state, never `Running` (see the P2 catch-up semantics in
    /// `docs/plans/2026-09-23-continuous-agent-design.md` §5.3).
    Skipped,
}

impl RunStatus {
    /// The string previously stored in `ScheduleTask::last_status` ("ok" /
    /// "error") stays the CLI-facing vocabulary; this is the ledger's own.
    pub fn as_str(self) -> &'static str {
        match self {
            RunStatus::Running => "running",
            RunStatus::Success => "success",
            RunStatus::Error => "error",
            RunStatus::Cancelled => "cancelled",
            RunStatus::Skipped => "skipped",
        }
    }
}

/// What fired the run.
///
/// `Manual` covers both a user invocation and the OS scheduler's cold start —
/// they share one command line, so they cannot be told apart (registered
/// residual in AGENTS.md). `Daemon` is the P2 tick (`schedule tick`), which can
/// distinguish itself. `OsScheduler` is reserved for a future explicit marker.
/// `Im` is a chat message driving the agent through an IM channel; its ledger
/// lives under the IM tree (`im/runs/`), not the schedules tree, so the
/// `task_id` field of those records carries the channel's hashed directory stem
/// rather than a schedule id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunTrigger {
    Manual,
    OsScheduler,
    Daemon,
    Im,
    /// Fired because a named event reached a task through the dependency graph
    /// (P2.5), rather than because its own schedule came due.
    Event,
    /// Fired by a persisted wakeup registry entry coming due (P3), rather than
    /// by the task's own schedule, an event, or a manual/daemon invocation.
    Wakeup,
}

/// One run of one task, persisted under `<task-id>/runs/<run_id>.json`.
///
/// Written best-effort: a ledger failure must never fail the task itself (the
/// OS scheduler judges runs by the process exit code, not by bookkeeping).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunRecord {
    pub run_id: String,
    pub task_id: String,
    pub status: RunStatus,
    pub trigger: RunTrigger,
    /// Unix seconds when the run started.
    pub started_at: i64,
    /// Unix seconds when the run reached a terminal state; `None` while `Running`.
    #[serde(default)]
    pub finished_at: Option<i64>,
    /// Process exit code; `None` while `Running` or when the launcher could not
    /// observe one.
    #[serde(default)]
    pub exit_code: Option<i32>,
    /// Agent session bound to this run, when one was created.
    #[serde(default)]
    pub session_id: Option<String>,
    /// Short human-readable outcome note (e.g. the last error line).
    #[serde(default)]
    pub summary: Option<String>,
}

/// Mint a run id whose lexical order matches start order: `<secs>-<nanos(9)>`.
/// Two starts of the same task cannot share a nanosecond, so consecutive runs
/// always get distinct ids. Passes `valid_id` (digits + `-`), so the ledger
/// path helpers accept it unchanged.
pub fn mint_run_id(started_at_epoch_secs: i64, subsec_nanos: u32) -> String {
    format!("{started_at_epoch_secs}-{subsec_nanos:09}")
}

/// `<task-id>/runs/` for a task id (decision D2). The id is untrusted (it comes
/// off the CLI surface just like `load`/`remove`), so the same `valid_id` guard
/// applies -- a crafted id must not escape the schedules directory.
fn runs_dir_in(root: &std::path::Path, task_id: &str) -> std::io::Result<PathBuf> {
    if !valid_id(task_id) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("invalid scheduled task id {task_id:?}"),
        ));
    }
    Ok(root.join(task_id).join("runs"))
}

/// Persist one run record (`<task-id>/runs/<run_id>.json`). The run id is also
/// untrusted once it travels through callers, so it gets the same guard.
pub fn save_run(task_id: &str, record: &RunRecord) -> std::io::Result<()> {
    save_run_in(&schedules_root(), task_id, record)
}

fn save_run_in(root: &std::path::Path, task_id: &str, record: &RunRecord) -> std::io::Result<()> {
    if !valid_id(&record.run_id) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("invalid run id {:?}", record.run_id),
        ));
    }
    let dir = runs_dir_in(root, task_id)?;
    std::fs::create_dir_all(&dir)?;
    let bytes = serde_json::to_vec_pretty(record)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(dir.join(format!("{}.json", record.run_id)), bytes)
}

/// All run records for a task, **newest first** (started_at desc, run_id desc
/// for stable ties). Corrupt files are skipped, mirroring `list_in`.
pub fn list_runs(task_id: &str) -> Vec<RunRecord> {
    list_runs_in(&schedules_root(), task_id)
}

fn list_runs_in(root: &std::path::Path, task_id: &str) -> Vec<RunRecord> {
    let Ok(dir) = runs_dir_in(root, task_id) else {
        return Vec::new();
    };
    let Ok(rd) = std::fs::read_dir(&dir) else {
        return Vec::new(); // no ledger yet -> empty history (not an error)
    };
    let mut out = Vec::new();
    for entry in rd.flatten() {
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        if let Ok(bytes) = std::fs::read(&p) {
            if let Ok(r) = serde_json::from_slice::<RunRecord>(&bytes) {
                out.push(r);
            }
        }
    }
    out.sort_by(|a, b| {
        b.started_at
            .cmp(&a.started_at)
            .then(b.run_id.cmp(&a.run_id))
    });
    out
}

/// The most recent run record, if any.
pub fn latest_run(task_id: &str) -> Option<RunRecord> {
    list_runs(task_id).into_iter().next()
}

/// Trim the ledger to the newest `keep` records. Returns how many files were
/// removed. Unparseable files are left alone (their age is unknown), so a
/// corrupt record cannot silently delete newer history.
pub fn prune_runs(task_id: &str, keep: usize) -> std::io::Result<usize> {
    prune_runs_in(&schedules_root(), task_id, keep)
}

fn prune_runs_in(root: &std::path::Path, task_id: &str, keep: usize) -> std::io::Result<usize> {
    let dir = runs_dir_in(root, task_id)?;
    let runs = list_runs_in(root, task_id);
    if runs.len() <= keep {
        return Ok(0);
    }
    let mut removed = 0;
    for record in &runs[keep..] {
        if std::fs::remove_file(dir.join(format!("{}.json", record.run_id))).is_ok() {
            removed += 1;
        }
    }
    Ok(removed)
}

// ─────────────────── IM message ledger ───────────────────

/// Root of the IM message run ledger: `<RUSTCODE_HOME>/im/runs/`.
///
/// The IM ledger is deliberately a **sibling** of the schedules tree rather than
/// a child: its per-channel directories are keyed by a hash of the chat id
/// (network input), and mixing that key space into `schedules/` would let a
/// crafted channel directory shadow a real task's ledger.
pub fn im_runs_root() -> PathBuf {
    crate::config::Config::config_dir().join("im").join("runs")
}

/// Per-channel directory stem: `<platform>-<sha256(chat_id) first 16 hex>`.
///
/// The chat id comes off the network, so it is never used verbatim in a path --
/// the same sha256-truncation the binding store uses. This is the parallel of
/// `im_store::binding_stem_in`, kept here so the schedule module owns the whole
/// ledger path story without exporting binding-store internals.
pub fn im_channel_stem(platform: &str, chat_id: &str) -> std::io::Result<String> {
    use sha2::{Digest, Sha256};
    if !crate::im_store::valid_platform(platform) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("invalid IM platform {platform:?}"),
        ));
    }
    if chat_id.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "empty IM chat id".to_string(),
        ));
    }
    let digest = Sha256::digest(chat_id.as_bytes());
    let mut hex = String::with_capacity(16);
    for byte in digest.iter().take(8) {
        use std::fmt::Write as _;
        let _ = write!(hex, "{byte:02x}");
    }
    Ok(format!("{platform}-{hex}"))
}

/// Persist one IM message run record under
/// `<RUSTCODE_HOME>/im/runs/<platform>-<sha256(chat_id)[..16]>/runs/<run_id>.json`.
///
/// Same best-effort contract as [`save_run`]: the caller decides whether a
/// ledger failure may fail the turn (for IM it must not).
pub fn save_im_run(platform: &str, chat_id: &str, record: &RunRecord) -> std::io::Result<()> {
    let stem = im_channel_stem(platform, chat_id)?;
    save_run_in(&im_runs_root(), &stem, record)
}

/// All run records for one IM chat, **newest first** (same ordering as
/// [`list_runs`]). Unknown chat -> empty history, not an error.
pub fn list_im_runs(platform: &str, chat_id: &str) -> Vec<RunRecord> {
    match im_channel_stem(platform, chat_id) {
        Ok(stem) => list_runs_in(&im_runs_root(), &stem),
        Err(_) => Vec::new(),
    }
}

/// Trim one IM chat's ledger to the newest `keep` records.
pub fn prune_im_runs(platform: &str, chat_id: &str, keep: usize) -> std::io::Result<usize> {
    let stem = im_channel_stem(platform, chat_id)?;
    prune_runs_in(&im_runs_root(), &stem, keep)
}

// ─────────────────── daemon tick support (P2) ───────────────────

/// Summary stored on a record reaped from a dead `Running` state.
pub const STALE_RUN_SUMMARY: &str =
    "interrupted: the process exited before recording a terminal state";

/// Summary stored on a run deliberately not executed because its scheduled
/// instant fell outside the catch-up window.
pub const CATCH_UP_MISSED_SUMMARY: &str = "missed run outside the catch-up window";

/// Is a missed run still allowed to execute?
///
/// `window_secs == 0` means "no window" (always eligible). This is deliberately
/// the **only** definition of the catch-up rule — the later persistent-wakeup
/// restoring path (P3, design §5.5 D3) must call this rather than grow a second
/// comparison, otherwise "how late is too late" becomes two truths.
pub fn within_catch_up_window(due_at: i64, now: i64, window_secs: u64) -> bool {
    if window_secs == 0 {
        return true;
    }
    now.saturating_sub(due_at) <= window_secs as i64
}

// ---------------------------------------------------------------------------
// P2.5 static dependency graph + event triggers
// ---------------------------------------------------------------------------

/// A problem found while validating the dependency graph.
///
/// Every problem is reported rather than repaired: silently dropping an unknown
/// dependency would let a user believe B runs after A when B never runs at all,
/// and silently breaking a cycle would impose an arbitrary order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GraphError {
    /// `depends_on` names a task that does not exist.
    UnknownDependency { task: String, dep: String },
    /// A task lists itself as a dependency.
    SelfDependency { task: String },
    /// A dependency cycle; the ids are the cycle itself, in order.
    Cycle { tasks: Vec<String> },
    /// Two tasks share one id, so a dependency edge is ambiguous.
    DuplicateId { id: String },
}

impl std::fmt::Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownDependency { task, dep } => {
                write!(f, "task `{task}` depends on `{dep}`, which does not exist")
            }
            Self::SelfDependency { task } => write!(f, "task `{task}` depends on itself"),
            Self::Cycle { tasks } => write!(f, "dependency cycle: {}", tasks.join(" -> ")),
            Self::DuplicateId { id } => write!(f, "duplicate task id `{id}`"),
        }
    }
}

impl std::error::Error for GraphError {}

/// Validate the dependency graph. Empty means the graph is usable.
///
/// Pure — no IO, no mutation — so it can run at load time and in tests alike.
/// Cycles are found with a three-colour DFS (unvisited / on-stack / done),
/// which is what distinguishes a back edge (a real cycle) from a cross edge
/// into an already-finished node (not one).
pub fn validate_graph(tasks: &[ScheduleTask]) -> Vec<GraphError> {
    use std::collections::HashMap;

    let mut errors = Vec::new();
    let mut by_id: HashMap<&str, usize> = HashMap::new();
    for (idx, task) in tasks.iter().enumerate() {
        if by_id.insert(task.id.as_str(), idx).is_some() {
            errors.push(GraphError::DuplicateId {
                id: task.id.clone(),
            });
        }
    }

    for task in tasks {
        for dep in &task.depends_on {
            if dep == &task.id {
                errors.push(GraphError::SelfDependency {
                    task: task.id.clone(),
                });
            } else if !by_id.contains_key(dep.as_str()) {
                errors.push(GraphError::UnknownDependency {
                    task: task.id.clone(),
                    dep: dep.clone(),
                });
            }
        }
    }

    // Walk known-good edges only: an unknown dependency has already been
    // reported above and must not also be walked into a spurious cycle.
    const UNVISITED: u8 = 0;
    const ON_STACK: u8 = 1;
    const DONE: u8 = 2;

    fn walk(
        idx: usize,
        tasks: &[ScheduleTask],
        by_id: &HashMap<&str, usize>,
        state: &mut Vec<u8>,
        stack: &mut Vec<usize>,
        reported: &mut HashSet<Vec<String>>,
        errors: &mut Vec<GraphError>,
    ) {
        match state[idx] {
            DONE => return,
            ON_STACK => {
                // Back edge: the cycle is the current stack from `idx` on.
                let start = stack.iter().position(|s| *s == idx).unwrap_or(0);
                let mut cycle: Vec<String> = stack[start..]
                    .iter()
                    .map(|i| tasks[*i].id.clone())
                    .collect();
                // One cycle is reachable from each of its members; report it
                // once, in a canonical rotation, so the error count is stable.
                let mut shift = 0;
                for (i, id) in cycle.iter().enumerate() {
                    if id < &cycle[shift] {
                        shift = i;
                    }
                }
                cycle.rotate_left(shift);
                if reported.insert(cycle.clone()) {
                    errors.push(GraphError::Cycle { tasks: cycle });
                }
                return;
            }
            _ => {}
        }
        state[idx] = ON_STACK;
        stack.push(idx);
        for dep in &tasks[idx].depends_on {
            // A self edge is already reported as `SelfDependency`; walking it
            // would additionally report a one-element cycle, which is the same
            // problem twice.
            if dep == &tasks[idx].id {
                continue;
            }
            if let Some(&next) = by_id.get(dep.as_str()) {
                walk(next, tasks, by_id, state, stack, reported, errors);
            }
        }
        stack.pop();
        state[idx] = DONE;
    }

    let mut state = vec![UNVISITED; tasks.len()];
    let mut stack: Vec<usize> = Vec::new();
    let mut reported: HashSet<Vec<String>> = HashSet::new();
    for idx in 0..tasks.len() {
        walk(
            idx,
            tasks,
            &by_id,
            &mut state,
            &mut stack,
            &mut reported,
            &mut errors,
        );
    }
    errors
}

/// Whether every dependency of `task` has recorded a successful run.
///
/// Pure: the caller supplies the set of ids whose latest run ended in
/// [`RunStatus::Success`]. A task with no dependencies is always ready — this
/// is what keeps every pre-P2.5 task on exactly its existing behaviour.
pub fn dependencies_ready(task: &ScheduleTask, succeeded: &HashSet<String>) -> bool {
    task.depends_on.iter().all(|dep| succeeded.contains(dep))
}

/// Tasks that fire when `event` occurs, regardless of their own schedule.
///
/// Disabled tasks are excluded: an event must not resurrect a task the user
/// switched off.
pub fn tasks_triggered_by<'a>(tasks: &'a [ScheduleTask], event: &str) -> Vec<&'a ScheduleTask> {
    tasks
        .iter()
        .filter(|t| t.enabled && t.triggers.iter().any(|e| e == event))
        .collect()
}

/// Ids of tasks whose most recent run ended in [`RunStatus::Success`].
///
/// Only ids that some task actually names as a dependency are looked up, so a
/// store full of dependency-free tasks costs no extra IO.
fn succeeded_ids_in(root: &std::path::Path, tasks: &[ScheduleTask]) -> HashSet<String> {
    let mut needed: Vec<&str> = Vec::new();
    for task in tasks {
        for dep in &task.depends_on {
            if !needed.contains(&dep.as_str()) {
                needed.push(dep.as_str());
            }
        }
    }
    let mut out = HashSet::new();
    for dep in needed {
        if let Some(record) = list_runs_in(root, dep).into_iter().next() {
            if record.status == RunStatus::Success {
                out.insert(dep.to_string());
            }
        }
    }
    out
}

/// Tasks whose next fire time has arrived, paired with that fire time.
///
/// The "last fired" anchor is `last_run_at`, falling back to `created_at`, so a
/// brand-new task first fires at its scheduled instant rather than being
/// treated as overdue from the epoch. Disabled tasks are never due. Pure apart
/// from reading the store — no clock read, no mutation — so the CLI `--once`
/// path and the daemon tick share exactly one definition of "due".
///
/// `Interval` intentionally yields a single due instant no matter how long the
/// task has been idle (a 30-minute task that has not run for two hours is due
/// once, not four times).
///
/// P2.5: a task whose dependencies have not **all** succeeded is held back even
/// once its own fire time arrives. Dependency-free tasks are unaffected, so
/// this stays additive for every task written before the graph existed.
pub fn due_tasks_in(root: &std::path::Path, now_epoch_secs: i64) -> Vec<(ScheduleTask, i64)> {
    let tasks = list_in(root);
    let succeeded = succeeded_ids_in(root, &tasks);
    let mut out = Vec::new();
    for task in tasks {
        if !task.enabled {
            continue;
        }
        if !dependencies_ready(&task, &succeeded) {
            continue;
        }
        let anchor = task.last_run_at.unwrap_or(task.created_at);
        let anchor = if anchor <= 0 { now_epoch_secs } else { anchor };
        if let Some(due_at) = next_run(&task.schedule, anchor) {
            if due_at <= now_epoch_secs {
                out.push((task, due_at));
            }
        }
    }
    out
}

/// [`due_tasks_in`] against the real schedules directory.
pub fn due_tasks(now_epoch_secs: i64) -> Vec<(ScheduleTask, i64)> {
    due_tasks_in(&schedules_root(), now_epoch_secs)
}

/// `<task-id>/.lock` — the single-flight lock file. The id is untrusted (it
/// reaches here from the store and the CLI), so it gets the same `valid_id`
/// guard as every other path helper.
fn lock_path_in(root: &std::path::Path, task_id: &str) -> std::io::Result<PathBuf> {
    if !valid_id(task_id) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("invalid scheduled task id {task_id:?}"),
        ));
    }
    Ok(root.join(task_id).join(".lock"))
}

/// Held for the duration of one run. Dropping it releases the advisory lock,
/// letting the next tick (or the OS scheduler's cold start) claim the task.
pub struct RunGuard {
    file: std::fs::File,
}

impl Drop for RunGuard {
    fn drop(&mut self) {
        // Best-effort: the OS releases the lock when the descriptor closes anyway.
        let _ = fs2::FileExt::unlock(&self.file);
    }
}

fn try_claim_run_in(root: &std::path::Path, task_id: &str) -> std::io::Result<Option<RunGuard>> {
    let path = lock_path_in(root, task_id)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = std::fs::OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        // Never truncate: the lock file carries no payload, and truncation
        // would be a write to a file another process may be holding.
        .truncate(false)
        .open(&path)?;
    match fs2::FileExt::try_lock_exclusive(&file) {
        Ok(()) => Ok(Some(RunGuard { file })),
        // Held elsewhere (or the platform refused) -> fail closed: report "not
        // ours" rather than risk two concurrent runs of the same task.
        Err(_) => Ok(None),
    }
}

/// Claim the single-flight lock for one run of `task_id`. `None` means another
/// process (or another tick) is already running it — the caller must skip, not
/// queue, so a long run is never re-entered.
pub fn try_claim_run(task_id: &str) -> Option<RunGuard> {
    try_claim_run_in(&schedules_root(), task_id).ok().flatten()
}

/// Probe (and immediately release) whether some process currently holds the run
/// lock. The daemon tick uses this to avoid spawning a child on every tick
/// while a long run is still in flight. Fail-closed: an invalid id or an I/O
/// error reports "held", so a doubtful lock never causes a double trigger.
pub fn run_lock_is_held(task_id: &str) -> bool {
    run_lock_is_held_in(&schedules_root(), task_id)
}

fn run_lock_is_held_in(root: &std::path::Path, task_id: &str) -> bool {
    match try_claim_run_in(root, task_id) {
        Ok(Some(guard)) => {
            drop(guard);
            false
        }
        _ => true,
    }
}

/// Turn records still stuck in `Running` into terminal `Error` records.
///
/// A `Running` record only legitimately exists while its owner holds the run
/// lock, so one that is `Running` **while the lock is free** means the previous
/// process died before it could record a terminal state (crash / power loss).
/// Leaving it would make `schedule history` show a permanently-running task,
/// violating "every accepted run reaches a terminal state".
///
/// Takes the lock itself: if a live runner holds it, there is nothing stale to
/// reap and this returns 0. Returns how many records were rewritten.
pub fn reap_stale_running(task_id: &str, now_epoch_secs: i64) -> std::io::Result<usize> {
    reap_stale_running_in(&schedules_root(), task_id, now_epoch_secs)
}

fn reap_stale_running_in(
    root: &std::path::Path,
    task_id: &str,
    now_epoch_secs: i64,
) -> std::io::Result<usize> {
    let Some(guard) = try_claim_run_in(root, task_id)? else {
        return Ok(0); // a live runner owns the task; nothing is stale
    };
    let mut reaped = 0;
    for mut record in list_runs_in(root, task_id) {
        if record.status != RunStatus::Running {
            continue;
        }
        record.status = RunStatus::Error;
        record.finished_at = Some(now_epoch_secs);
        record.summary = Some(STALE_RUN_SUMMARY.to_string());
        if save_run_in(root, task_id, &record).is_ok() {
            reaped += 1;
        }
    }
    drop(guard);
    Ok(reaped)
}

/// Record a run that was deliberately not executed because it fell outside the
/// catch-up window, and advance the task's anchor so the same missed instant is
/// not re-evaluated on every tick.
///
/// Returns the written record. The write stays *inside* the single-flight lock
/// (taken here) so two ticks cannot both mint a `Skipped` row for one miss.
/// `None` means the task was busy or the store was unavailable.
pub fn record_catch_up_skip(task_id: &str, now_epoch_secs: i64) -> Option<RunRecord> {
    record_catch_up_skip_in(&schedules_root(), task_id, now_epoch_secs)
}

fn record_catch_up_skip_in(
    root: &std::path::Path,
    task_id: &str,
    now_epoch_secs: i64,
) -> Option<RunRecord> {
    let guard = try_claim_run_in(root, task_id).ok().flatten()?;
    let mut task = load_in(root, task_id).ok()?;
    let run_id = mint_run_id(now_epoch_secs, 0);
    let record = RunRecord {
        run_id: run_id.clone(),
        task_id: task_id.to_string(),
        status: RunStatus::Skipped,
        trigger: RunTrigger::Daemon,
        started_at: now_epoch_secs,
        finished_at: Some(now_epoch_secs),
        exit_code: None,
        session_id: None,
        // The design's separate `skipped_because_of` field is folded into
        // `summary` here (recorded deviation, design §4.1): the reason is
        // human-auditable and this keeps `RunRecord`'s additive surface at zero.
        summary: Some(CATCH_UP_MISSED_SUMMARY.to_string()),
    };
    let _ = save_run_in(root, task_id, &record);
    // Advance the anchor past the miss, otherwise every tick would re-evaluate
    // (and re-record) the same old instant forever.
    task.last_run_at = Some(now_epoch_secs);
    task.last_status = Some(RunStatus::Skipped.as_str().to_string());
    task.last_run_id = Some(run_id);
    let _ = save_in(root, &task);
    drop(guard);
    Some(record)
}

/// Days-from-civil / civil-from-days conversion (Howard Hinnant's algorithm).
/// Needed because `next_run` must align `Weekly` and `Cron` to a real calendar
/// weekday, and naive `epoch / 86400` arithmetic cannot tell which weekday a
/// day is. Days are counted from the Unix epoch; the returned day-of-week is
/// `0 = Sunday .. 6 = Saturday` (the cron convention).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    (if m <= 2 { y + 1 } else { y }, m as u32, d as u32)
}

/// Day-of-week for a day index: `0 = Sunday .. 6 = Saturday`.
fn day_of_week(day: i64) -> u32 {
    (day + 4).rem_euclid(7) as u32
}

/// How many days ahead `next_run` may look before giving up. Eight years is
/// enough to reach a 29 February under the 100-year leap rule (2096 -> 2104),
/// which is the only case that legitimately needs more than four years.
const MAX_SEARCH_DAYS: i64 = 366 * 8;

/// One parsed cron field: the sorted, de-duplicated set of matching values plus
/// whether the field was restricted at all (i.e. not `*`). The `restricted`
/// flag is what drives the day-of-month / day-of-week OR rule (see
/// [`next_cron`]).
struct CronField {
    values: Vec<u32>,
    restricted: bool,
}

/// Parse one cron field into its expanded value set. Supports `*`, `a`, `a-b`,
/// `a,b` and `*/n` (plus `a-b/n`, which follows from composing step + range).
/// Macros (`@daily`) and names (`MON`) are deliberately **not** supported --
/// see decision D1 in `docs/plans/2026-09-23-continuous-agent-design.md`: this
/// only has to drive the list display and the tick decision, not full cron.
fn parse_cron_field(field: &str, min: u32, max: u32) -> Option<CronField> {
    let mut values: Vec<u32> = Vec::new();
    for part in field.split(',') {
        let part = part.trim();
        if part.is_empty() {
            return None;
        }
        let (range, step) = match part.split_once('/') {
            Some((r, s)) => (r.trim(), s.trim().parse::<u32>().ok()?),
            None => (part, 1),
        };
        if step == 0 {
            return None;
        }
        let (lo, hi) = if range == "*" {
            (min, max)
        } else if let Some((a, b)) = range.split_once('-') {
            (a.trim().parse::<u32>().ok()?, b.trim().parse::<u32>().ok()?)
        } else {
            let v = range.parse::<u32>().ok()?;
            // `5/15` means "from 5 to the end of the range, every 15".
            (v, if step == 1 { v } else { max })
        };
        if lo < min || hi > max || lo > hi {
            return None;
        }
        let mut v = lo;
        while v <= hi {
            values.push(v);
            v += step;
        }
    }
    values.sort_unstable();
    values.dedup();
    if values.is_empty() {
        return None;
    }
    Some(CronField {
        values,
        restricted: field.trim() != "*",
    })
}

/// Next fire time for a five-field cron expression (`min hour dom month dow`),
/// strictly after `now`. Returns `None` for a malformed expression or one that
/// cannot fire within [`MAX_SEARCH_DAYS`] (e.g. `0 0 30 2 *`, 30 February), so
/// every caller keeps its existing "unknown" display path.
///
/// Day-of-month and day-of-week follow the standard Vixie rule: when **both**
/// are restricted, a day matches if **either** matches; otherwise both must
/// match. Ignoring this would make the common `0 9 1 * 1` (first of month *or*
/// every Monday) silently fire far less often than the user asked for.
fn next_cron(expr: &str, now_epoch_secs: i64) -> Option<i64> {
    let fields: Vec<&str> = expr.split_whitespace().collect();
    if fields.len() != 5 {
        return None;
    }
    let minutes = parse_cron_field(fields[0], 0, 59)?;
    let hours = parse_cron_field(fields[1], 0, 23)?;
    let dom = parse_cron_field(fields[2], 1, 31)?;
    let month = parse_cron_field(fields[3], 1, 12)?;
    let mut dow = parse_cron_field(fields[4], 0, 7)?;
    // Cron accepts both 0 and 7 for Sunday; normalize to the 0..=6 form.
    if dow.values.contains(&7) {
        dow.values.push(0);
        dow.values.retain(|&v| v <= 6);
        dow.values.sort_unstable();
        dow.values.dedup();
    }
    let dow_or_dom = dom.restricted && dow.restricted;

    let first = now_epoch_secs + 1; // strictly after `now`
    let start_day = first.div_euclid(86400);
    for day in start_day..start_day + MAX_SEARCH_DAYS {
        let (_, m, d) = civil_from_days(day);
        let matches = if dow_or_dom {
            dom.values.contains(&d) || dow.values.contains(&day_of_week(day))
        } else {
            dom.values.contains(&d) && dow.values.contains(&day_of_week(day))
        };
        if month.values.contains(&m) && matches {
            // `hours` / `minutes` are sorted, so the first hit is the earliest.
            for &h in &hours.values {
                for &mi in &minutes.values {
                    let ts = day * 86400 + i64::from(h) * 3600 + i64::from(mi) * 60;
                    if ts >= first {
                        return Some(ts);
                    }
                }
            }
        }
    }
    None
}

/// Next fire time (epoch secs) for a schedule, strictly after `now`.
///
/// Uses UTC day/hour arithmetic; there is no DST handling (acceptable for the
/// list display and the tick decision -- exact firing remains the OS
/// scheduler's job, which uses its own local calendar). Returns `None` when the
/// schedule is malformed or cannot fire within [`MAX_SEARCH_DAYS`]; display
/// callers render that as `-`.
pub fn next_run(schedule: &Schedule, now_epoch_secs: i64) -> Option<i64> {
    fn hhmm(s: &str) -> Option<(i64, i64)> {
        let (h, m) = s.split_once(':')?;
        Some((h.parse().ok()?, m.parse().ok()?))
    }
    match schedule {
        Schedule::Interval { every_minutes } => {
            (*every_minutes > 0).then(|| now_epoch_secs + (*every_minutes as i64) * 60)
        }
        Schedule::Hourly => {
            let secs_into_hour = now_epoch_secs.rem_euclid(3600);
            Some(now_epoch_secs + (3600 - secs_into_hour))
        }
        Schedule::Daily { time } => {
            let (h, m) = hhmm(time)?;
            let day = now_epoch_secs.div_euclid(86400) * 86400;
            let target = day + h * 3600 + m * 60;
            Some(if target > now_epoch_secs {
                target
            } else {
                target + 86400
            })
        }
        Schedule::Weekly { weekday, time } => {
            // Align to the requested weekday. The previous "phase 1
            // approximation" ignored `weekday` entirely and returned the next
            // day-boundary with that time, which made `schedule list` claim a
            // weekly Monday task would fire tomorrow.
            if *weekday == 0 || *weekday > 7 {
                return None;
            }
            let (h, m) = hhmm(time)?;
            let want = u32::from(*weekday % 7); // 1..=6 -> Mon..Sat, 7 -> 0 (Sunday)
            let first = now_epoch_secs + 1;
            let start_day = first.div_euclid(86400);
            for day in start_day..start_day + 8 {
                let ts = day * 86400 + h * 3600 + m * 60;
                if day_of_week(day) == want && ts >= first {
                    return Some(ts);
                }
            }
            None
        }
        Schedule::Cron { expr } => next_cron(expr, now_epoch_secs),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> ScheduleTask {
        ScheduleTask {
            id: "t1".into(),
            title: "Daily brief".into(),
            prompt: "summarize".into(),
            cwd: "/tmp/proj".into(),
            schedule: Schedule::Daily {
                time: "09:00".into(),
            },
            permission_mode: "plan".into(),
            notify: "important".into(),
            enabled: true,
            created_at: 0,
            last_run_at: None,
            last_status: None,
            last_run_id: None,
            depends_on: Vec::new(),
            triggers: Vec::new(),
        }
    }

    #[test]
    fn task_json_roundtrips() {
        let t = sample();
        let json = serde_json::to_string(&t).unwrap();
        let back: ScheduleTask = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "t1");
        assert!(matches!(back.schedule, Schedule::Daily { .. }));
        assert_eq!(back.permission_mode, "plan");
    }

    #[test]
    fn store_save_load_list_remove_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        save_in(root, &sample()).unwrap();
        assert_eq!(load_in(root, "t1").unwrap().title, "Daily brief");
        assert_eq!(list_in(root).len(), 1);
        remove_in(root, "t1").unwrap();
        assert!(list_in(root).is_empty());
    }

    #[test]
    fn store_rejects_ids_that_escape_the_schedules_dir() {
        use std::io::ErrorKind;
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        for bad in ["../evil", "a/b", "a\\b", "foo/../bar", "..", ""] {
            // remove/enable/disable/run and save all flow through these helpers,
            // so a crafted id must be rejected before it can touch the filesystem.
            assert_eq!(
                remove_in(root, bad).unwrap_err().kind(),
                ErrorKind::InvalidInput,
                "remove should reject id {bad:?}"
            );
            assert_eq!(
                load_in(root, bad).unwrap_err().kind(),
                ErrorKind::InvalidInput,
                "load should reject id {bad:?}"
            );
            let mut task = sample();
            task.id = bad.to_string();
            assert_eq!(
                save_in(root, &task).unwrap_err().kind(),
                ErrorKind::InvalidInput,
                "save should reject id {bad:?}"
            );
        }
        // A traversal id must not have written anything outside the store.
        assert!(!tmp.path().parent().unwrap().join("evil.json").exists());
    }

    #[test]
    fn store_accepts_slug_and_uuid_shaped_ids() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let mut task = sample();
        task.id = "daily-brief_2026.01-9f8e7d6c".into();
        save_in(root, &task).unwrap();
        assert_eq!(load_in(root, &task.id).unwrap().id, task.id);
        remove_in(root, &task.id).unwrap();
    }

    #[test]
    fn next_run_daily_is_today_or_tomorrow_at_time() {
        // 2026-07-31 08:00:00 UTC = 1785657600 ; daily 09:00 -> same day 09:00
        let now = 1785657600;
        let nr = next_run(
            &Schedule::Daily {
                time: "09:00".into(),
            },
            now,
        )
        .unwrap();
        assert!(nr > now && nr - now <= 24 * 3600);
    }

    /// 2026-09-25 08:00:00 UTC = 1790323200 (a Friday).
    const FRI_0800: i64 = 1790323200;

    fn cron_at(expr: &str, now: i64) -> Option<i64> {
        next_run(&Schedule::Cron { expr: expr.into() }, now)
    }

    #[test]
    fn next_run_cron_weekday_range_skips_the_weekend() {
        // `0 9 * * 1-5` from Friday 08:00 -> later that morning (Fri is a
        // weekday). From Friday 10:00 -> next Monday 09:00.
        assert_eq!(cron_at("0 9 * * 1-5", FRI_0800), Some(1790326800)); // Fri 09:00
        assert_eq!(
            cron_at("0 9 * * 1-5", 1790332200), // Fri 10:30
            Some(1790586000)                    // Mon 2026-09-28 09:00
        );
    }

    #[test]
    fn next_run_cron_is_idempotent() {
        let first = cron_at("0 9 * * 1-5", FRI_0800).unwrap();
        // Re-asking from the instant *before* the answer must yield the same
        // answer; the tick loop relies on this not drifting.
        assert_eq!(cron_at("0 9 * * 1-5", first - 1), Some(first));
    }

    #[test]
    fn next_run_cron_is_strictly_after_now() {
        let now = 1790326800; // Fri 09:00 exactly
        let next = cron_at("0 9 * * 1-5", now).unwrap();
        assert!(next > now, "must not re-fire the instant that just passed");
    }

    #[test]
    fn next_run_cron_supports_star_steps_lists_and_ranges() {
        // Every 15 minutes.
        assert_eq!(cron_at("*/15 * * * *", 1790323200), Some(1790324100)); // 08:15
                                                                           // Explicit list.
        assert_eq!(cron_at("0 9,17 * * *", 1790323200), Some(1790326800)); // 09:00
                                                                           // Range with a step (`8-11/2` -> 08:00, 10:00); 08:00 has passed.
        assert_eq!(cron_at("0 8-11/2 * * *", 1790323200), Some(1790330400)); // 10:00
    }

    #[test]
    fn next_run_cron_treats_sunday_0_and_7_alike() {
        // Both spellings must reach the same Sunday 2026-09-27 09:00.
        let sun = 1790499600;
        assert_eq!(cron_at("0 9 * * 0", FRI_0800), Some(sun));
        assert_eq!(cron_at("0 9 * * 7", FRI_0800), Some(sun));
    }

    #[test]
    fn next_run_cron_or_rule_applies_when_dom_and_dow_are_both_restricted() {
        // `0 9 1 * 1` = first of month OR every Monday. From Fri 2026-09-25
        // the next hit is Mon 2026-09-28, not 2026-10-01.
        assert_eq!(cron_at("0 9 1 * 1", FRI_0800), Some(1790586000));
    }

    #[test]
    fn next_run_cron_and_rule_applies_when_only_dom_is_restricted() {
        // `0 9 1 * *` = first of month only -> Thu 2026-10-01 09:00.
        assert_eq!(cron_at("0 9 1 * *", FRI_0800), Some(1790845200));
    }

    #[test]
    fn next_run_cron_rejects_malformed_expressions() {
        // Wrong field count, out-of-range values, and macros/names (D1: not
        // supported) all fall back to None so callers keep showing `-`.
        for bad in [
            "0 9 * *",
            "0 9 * * * *",
            "99 9 * * *",
            "0 9 * * 9",
            "0 9 * * 1-5-6",
            "@daily",
            "0 9 * * MON",
            "",
        ] {
            assert_eq!(cron_at(bad, FRI_0800), None, "should reject {bad:?}");
        }
    }

    #[test]
    fn next_run_cron_returns_none_when_the_date_can_never_occur() {
        // 30 February never happens; the search must terminate, not hang.
        assert_eq!(cron_at("0 9 30 2 *", FRI_0800), None);
    }

    // -- IM message ledger ---------------------------------------------------

    fn im_run_record(run_id: &str, started_at: i64) -> RunRecord {
        RunRecord {
            run_id: run_id.into(),
            task_id: "im".into(),
            status: RunStatus::Running,
            trigger: RunTrigger::Im,
            started_at,
            finished_at: None,
            exit_code: None,
            session_id: None,
            summary: None,
        }
    }

    #[test]
    fn im_channel_stem_hashes_the_chat_id_never_verbatim() {
        // The chat id is network input; the directory name must carry only the
        // truncated sha256, never the raw id (no separators, no traversal).
        let stem = im_channel_stem("dingtalk", "../../evil/chat id").unwrap();
        assert!(stem.starts_with("dingtalk-"));
        let hex = stem.trim_start_matches("dingtalk-");
        assert_eq!(hex.len(), 16);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(!stem.contains('/'));

        // Same chat id on different platforms does not collide.
        assert_ne!(
            im_channel_stem("feishu", "c1").unwrap(),
            im_channel_stem("dingtalk", "c1").unwrap()
        );
        // Bad platform / empty chat id are rejected, not coerced into a path.
        assert!(im_channel_stem("bad platform", "c1").is_err());
        assert!(im_channel_stem("dingtalk", "").is_err());
    }

    #[test]
    fn im_run_ledger_roundtrips_and_prunes_per_chat() {
        struct HomeGuard(Option<String>);
        impl HomeGuard {
            fn new(dir: &std::path::Path) -> Self {
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
        let tmp = tempfile::tempdir().unwrap();
        let _guard = HomeGuard::new(tmp.path());
        save_im_run("dingtalk", "chat-1", &im_run_record("1000-000000001", 1000)).unwrap();
        save_im_run("dingtalk", "chat-1", &im_run_record("2000-000000002", 2000)).unwrap();
        save_im_run("dingtalk", "chat-2", &im_run_record("1000-000000003", 1000)).unwrap();

        let stem = im_channel_stem("dingtalk", "chat-1").unwrap();
        let dir = im_runs_root().join(&stem).join("runs");
        assert!(dir.join("1000-000000001.json").exists());
        assert!(dir.join("2000-000000002.json").exists());

        let runs = list_im_runs("dingtalk", "chat-1");
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].run_id, "2000-000000002");
        assert!(matches!(runs[0].trigger, RunTrigger::Im));
        // The other chat's history is separate.
        assert_eq!(list_im_runs("dingtalk", "chat-2").len(), 1);
        assert!(list_im_runs("feishu", "chat-1").is_empty());

        // Pruning keeps the newest.
        prune_im_runs("dingtalk", "chat-1", 1).unwrap();
        let runs = list_im_runs("dingtalk", "chat-1");
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].run_id, "2000-000000002");

        // Unknown chat -> empty history, not an error.
        assert!(list_im_runs("dingtalk", "never-seen").is_empty());
    }

    #[test]
    fn next_run_weekly_respects_weekday() {
        // Friday 08:00, weekly Monday 09:00 -> next Monday 09:00, NOT the very
        // next 09:00. The old "phase 1 approximation" ignored `weekday`.
        let next = next_run(
            &Schedule::Weekly {
                weekday: 1,
                time: "09:00".into(),
            },
            FRI_0800,
        )
        .unwrap();
        assert_eq!(next, 1790586000); // Mon 2026-09-28 09:00
        assert_eq!(
            day_of_week(next.div_euclid(86400)),
            1,
            "the result must actually be a Monday"
        );
    }

    #[test]
    fn next_run_weekly_same_day_is_today_when_the_time_has_not_passed() {
        // Monday 2026-09-21 08:00, weekly Monday 09:00 -> today 09:00.
        assert_eq!(
            next_run(
                &Schedule::Weekly {
                    weekday: 1,
                    time: "09:00".into(),
                },
                1789977600, // Mon 08:00
            ),
            Some(1789981200) // Mon 09:00
        );
    }

    #[test]
    fn next_run_weekly_accepts_7_as_sunday_and_rejects_invalid_weekdays() {
        // Friday 08:00 -> Sunday 2026-09-27 09:00.
        assert_eq!(
            next_run(
                &Schedule::Weekly {
                    weekday: 7,
                    time: "09:00".into(),
                },
                FRI_0800,
            ),
            Some(1790499600)
        );
        for bad in [0u8, 8, 255] {
            assert_eq!(
                next_run(
                    &Schedule::Weekly {
                        weekday: bad,
                        time: "09:00".into(),
                    },
                    FRI_0800,
                ),
                None,
                "weekday {bad} is out of range and must not fire"
            );
        }
    }

    // ───────────────────────── run ledger (P1) ─────────────────────────

    fn run_record(task_id: &str, run_id: &str, started_at: i64) -> RunRecord {
        RunRecord {
            run_id: run_id.into(),
            task_id: task_id.into(),
            status: RunStatus::Running,
            trigger: RunTrigger::Manual,
            started_at,
            finished_at: None,
            exit_code: None,
            session_id: None,
            summary: None,
        }
    }

    #[test]
    fn run_ledger_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // Two runs of the same task; the second starts later.
        let mut first = run_record("t1", "1000-000000001", 1000);
        first.status = RunStatus::Success;
        first.finished_at = Some(1060);
        first.exit_code = Some(0);
        let mut second = run_record("t1", "2000-000000002", 2000);
        second.status = RunStatus::Error;
        second.finished_at = Some(2050);
        second.exit_code = Some(1);
        second.summary = Some("cwd missing".into());

        save_run_in(root, "t1", &first).unwrap();
        save_run_in(root, "t1", &second).unwrap();

        // Newest first, regardless of insertion order.
        let runs = list_runs_in(root, "t1");
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].run_id, "2000-000000002");
        assert_eq!(runs[1].run_id, "1000-000000001");
        // Terminal shape survives the roundtrip.
        assert_eq!(runs[0].status, RunStatus::Error);
        assert_eq!(runs[0].exit_code, Some(1));
        assert_eq!(runs[0].finished_at, Some(2050));
        assert_eq!(runs[0].summary.as_deref(), Some("cwd missing"));
        assert_eq!(runs[1].status, RunStatus::Success);
        assert_eq!(runs[1].exit_code, Some(0));
        // Unknown task -> empty history, not an error.
        assert!(list_runs_in(root, "no-such-task").is_empty());
    }

    /// KEY compatibility regression (design §4.1 / D2): the ledger lives INSIDE
    /// the task's `<task-id>/` directory. The existing `list_in()` must keep
    /// reading only `<root>/<id>.json` task files and never treat the `runs/`
    /// subdirectory (or anything in it) as a task definition.
    #[test]
    fn task_list_does_not_read_the_runs_subdirectory() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        save_in(root, &sample()).unwrap(); // <root>/t1.json
        save_run_in(root, "t1", &run_record("t1", "1000-000000001", 1000)).unwrap();

        // Exactly one task, and its content is the task -- not the run record.
        let tasks = list_in(root);
        assert_eq!(tasks.len(), 1, "runs/ must not surface as a task");
        assert_eq!(tasks[0].id, "t1");
        assert!(tasks[0].last_run_id.is_none() || tasks[0].last_run_id.is_some()); // shape only
                                                                                   // And the ledger is still reachable through its own reader.
        assert_eq!(list_runs_in(root, "t1").len(), 1);
    }

    #[test]
    fn prune_runs_keeps_the_newest_n() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        for i in 0..5 {
            save_run_in(
                root,
                "t1",
                &run_record("t1", &format!("1{i:03}-00000000{i}"), 1000 + i),
            )
            .unwrap();
        }
        assert_eq!(list_runs_in(root, "t1").len(), 5);

        // Nothing to do when under the cap.
        assert_eq!(prune_runs_in(root, "t1", 10).unwrap(), 0);
        // Trim to 3: the two OLDEST records are removed.
        assert_eq!(prune_runs_in(root, "t1", 3).unwrap(), 2);
        let kept: Vec<i64> = list_runs_in(root, "t1")
            .iter()
            .map(|r| r.started_at)
            .collect();
        assert_eq!(kept, vec![1004, 1003, 1002]);
        // Idempotent.
        assert_eq!(prune_runs_in(root, "t1", 3).unwrap(), 0);
    }

    /// A Running record is the pre-start state; its optional fields stay empty
    /// and survive the roundtrip unchanged (serde defaults on the way out).
    #[test]
    fn running_record_has_no_terminal_fields() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        save_run_in(root, "t1", &run_record("t1", "1000-000000001", 1000)).unwrap();
        let run = &list_runs_in(root, "t1")[0];
        assert_eq!(run.status, RunStatus::Running);
        assert_eq!(run.finished_at, None);
        assert_eq!(run.exit_code, None);
        assert_eq!(run.session_id, None);
        assert_eq!(RunStatus::Running.as_str(), "running");
        assert_eq!(RunStatus::Cancelled.as_str(), "cancelled");
    }

    #[test]
    fn mint_run_ids_are_distinct_and_lexicographically_ordered() {
        let a = mint_run_id(1790000000, 1);
        let b = mint_run_id(1790000000, 999_999_999);
        let c = mint_run_id(1790000001, 0);
        assert_ne!(a, b, "same second, different nanos must differ");
        assert!(a < b && b < c, "lexical order must follow start order");
        // Minted ids pass the same guard the ledger applies (digits + '-').
        let tmp = tempfile::tempdir().unwrap();
        save_run_in(tmp.path(), "t1", &run_record("t1", &a, 1790000000)).unwrap();
    }

    #[test]
    fn run_ledger_rejects_untrusted_task_and_run_ids() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // A traversal task id must not create directories outside the store.
        assert_eq!(
            save_run_in(root, "../evil", &run_record("x", "1000-000000001", 1000))
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::InvalidInput
        );
        assert!(!tmp.path().parent().unwrap().join("evil").exists());
        // A traversal run id must not escape the ledger directory either.
        assert_eq!(
            save_run_in(root, "t1", &run_record("t1", "../../evil", 1000))
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::InvalidInput
        );
    }

    // ───────────────────── daemon tick support (P2) ─────────────────────

    /// 2026-09-25 08:00:00 UTC (a Friday), used as a stable `now`.
    const NOW: i64 = FRI_0800;

    fn interval_task(id: &str, created_at: i64, last_run_at: Option<i64>) -> ScheduleTask {
        let mut task = sample();
        task.id = id.into();
        task.schedule = Schedule::Interval { every_minutes: 30 };
        task.created_at = created_at;
        task.last_run_at = last_run_at;
        task
    }

    #[test]
    fn catch_up_window_boundaries() {
        // 0 = no window: every miss is still eligible.
        assert!(within_catch_up_window(NOW - 86_400, NOW, 0));
        // Exactly at the boundary counts as within (inclusive), a second later
        // does not -- otherwise a tick landing on the edge would flip-flop.
        assert!(within_catch_up_window(NOW - 3600, NOW, 3600));
        assert!(!within_catch_up_window(NOW - 3601, NOW, 3600));
        // A future `due_at` (clock skew) is never "too late".
        assert!(within_catch_up_window(NOW + 60, NOW, 1));
    }

    #[test]
    fn due_tasks_anchors_on_last_run_then_created_at() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // Never run: due 30 min after creation, and that instant has passed.
        save_in(root, &interval_task("fresh", NOW - 3600, None)).unwrap();
        // Already run one minute ago -> next fire is in the future, not due.
        save_in(root, &interval_task("recent", NOW - 86_400, Some(NOW - 60))).unwrap();
        // Disabled tasks are never due, even when overdue.
        let mut off = interval_task("off", NOW - 86_400, None);
        off.enabled = false;
        save_in(root, &off).unwrap();

        let due = due_tasks_in(root, NOW);
        assert_eq!(due.len(), 1, "only the never-run overdue task is due");
        assert_eq!(due[0].0.id, "fresh");
        assert_eq!(
            due[0].1,
            NOW - 3600 + 1800,
            "due instant = created_at + 30m"
        );
    }

    #[test]
    fn due_tasks_ignores_a_task_whose_first_slot_is_still_ahead() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // Created just now -> the first slot is 30 minutes away.
        save_in(root, &interval_task("brand-new", NOW, None)).unwrap();
        assert!(due_tasks_in(root, NOW).is_empty());
    }

    #[test]
    fn try_claim_is_exclusive_and_released_on_drop() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let first = try_claim_run_in(root, "t1").unwrap();
        assert!(first.is_some(), "the first claim must win");
        assert!(
            try_claim_run_in(root, "t1").unwrap().is_none(),
            "a second claim for the same task must fail (single flight)"
        );
        assert!(run_lock_is_held_in(root, "t1"), "the lock reads as held");
        // A different task is unaffected.
        assert!(try_claim_run_in(root, "t2").unwrap().is_some());

        drop(first);
        assert!(!run_lock_is_held_in(root, "t1"), "drop must release it");
        assert!(try_claim_run_in(root, "t1").unwrap().is_some());
    }

    #[test]
    fn claim_rejects_untrusted_task_ids() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        assert!(try_claim_run_in(root, "../evil").is_err());
        // Fail-closed probe: an unusable id reports "held", never "free".
        assert!(run_lock_is_held_in(root, "../evil"));
        assert!(!tmp.path().parent().unwrap().join("evil").exists());
    }

    /// P2 acceptance: a miss older than the catch-up window must NOT run; it is
    /// recorded as `Skipped` and the anchor advances so it is not re-evaluated
    /// on every tick (otherwise the ledger would grow one row per tick).
    #[test]
    fn catch_up_outside_window_records_skipped_and_advances_the_anchor() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // Due 90 minutes ago (30-minute interval + 60-minute creation lag).
        save_in(root, &interval_task("t1", NOW - 3600 - 3600, None)).unwrap();
        let (_, due_at) = due_tasks_in(root, NOW).pop().unwrap();
        assert_eq!(due_at, NOW - 3600 - 1800);
        assert!(!within_catch_up_window(due_at, NOW, 3600));

        let record = record_catch_up_skip_in(root, "t1", NOW).unwrap();
        assert_eq!(record.status, RunStatus::Skipped);
        assert_eq!(RunStatus::Skipped.as_str(), "skipped");
        assert_eq!(record.finished_at, Some(NOW));
        assert_eq!(record.summary.as_deref(), Some(CATCH_UP_MISSED_SUMMARY));

        let runs = list_runs_in(root, "t1");
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].status, RunStatus::Skipped);

        let task = load_in(root, "t1").unwrap();
        assert_eq!(task.last_run_at, Some(NOW), "anchor advances past the miss");
        assert_eq!(task.last_status.as_deref(), Some("skipped"));
        assert_eq!(task.last_run_id.as_deref(), Some(record.run_id.as_str()));
        // And the same missing instant is no longer due.
        assert!(due_tasks_in(root, NOW).is_empty());
    }

    #[test]
    fn catch_up_skip_refuses_when_the_task_is_running() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        save_in(root, &interval_task("t1", NOW - 7200, None)).unwrap();
        let guard = try_claim_run_in(root, "t1").unwrap().unwrap();
        assert!(
            record_catch_up_skip_in(root, "t1", NOW).is_none(),
            "a busy task must not have a Skipped row minted behind its back"
        );
        assert!(list_runs_in(root, "t1").is_empty());
        drop(guard);
    }

    /// P2 acceptance: a `Running` record left by a crashed process is reaped to
    /// `Error`, but only when no live runner holds the lock.
    #[test]
    fn stale_running_run_is_reaped_only_when_the_lock_is_free() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        save_in(root, &interval_task("t1", NOW - 7200, None)).unwrap();
        save_run_in(root, "t1", &run_record("t1", "1000-000000001", 1000)).unwrap();

        // While a runner holds the lock, the Running record is legitimate.
        let guard = try_claim_run_in(root, "t1").unwrap().unwrap();
        assert_eq!(reap_stale_running_in(root, "t1", NOW).unwrap(), 0);
        assert_eq!(list_runs_in(root, "t1")[0].status, RunStatus::Running);
        drop(guard);

        // With the lock free, the same record is provably abandoned.
        assert_eq!(reap_stale_running_in(root, "t1", NOW).unwrap(), 1);
        let run = &list_runs_in(root, "t1")[0];
        assert_eq!(run.status, RunStatus::Error);
        assert_eq!(run.finished_at, Some(NOW));
        assert_eq!(run.summary.as_deref(), Some(STALE_RUN_SUMMARY));
        // Terminal records are left alone on the next pass.
        assert_eq!(reap_stale_running_in(root, "t1", NOW).unwrap(), 0);
    }
}

#[cfg(test)]
mod graph_tests {
    use super::*;

    fn task(id: &str, depends_on: &[&str], triggers: &[&str]) -> ScheduleTask {
        ScheduleTask {
            id: id.to_string(),
            title: id.to_string(),
            prompt: "p".into(),
            cwd: "/tmp".into(),
            schedule: Schedule::Interval { every_minutes: 1 },
            permission_mode: "plan".into(),
            notify: "important".into(),
            enabled: true,
            created_at: 0,
            last_run_at: None,
            last_status: None,
            last_run_id: None,
            depends_on: depends_on.iter().map(|s| s.to_string()).collect(),
            triggers: triggers.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn succeeded(ids: &[&str]) -> HashSet<String> {
        ids.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn dependency_free_graphs_are_valid() {
        assert!(validate_graph(&[]).is_empty());
        assert!(validate_graph(&[task("a", &[], &[]), task("b", &[], &[])]).is_empty());
        // A well-formed chain is valid.
        assert!(validate_graph(&[task("a", &[], &[]), task("b", &["a"], &[])]).is_empty());
    }

    #[test]
    fn unknown_dependency_is_reported_not_ignored() {
        let errors = validate_graph(&[task("a", &["ghost"], &[])]);
        assert_eq!(
            errors,
            vec![GraphError::UnknownDependency {
                task: "a".into(),
                dep: "ghost".into()
            }]
        );
    }

    #[test]
    fn self_dependency_is_reported() {
        let errors = validate_graph(&[task("a", &["a"], &[])]);
        assert_eq!(
            errors,
            vec![GraphError::SelfDependency { task: "a".into() }]
        );
    }

    #[test]
    fn cycle_is_reported_exactly_once() {
        // a -> b -> c -> a. The cycle is reachable from each member, but it is
        // one cycle and must be reported once.
        let tasks = vec![
            task("a", &["b"], &[]),
            task("b", &["c"], &[]),
            task("c", &["a"], &[]),
        ];
        let errors = validate_graph(&tasks);
        let cycles: Vec<_> = errors
            .iter()
            .filter(|e| matches!(e, GraphError::Cycle { .. }))
            .collect();
        assert_eq!(
            cycles.len(),
            1,
            "one cycle must be reported once, got {errors:?}"
        );
    }

    #[test]
    fn duplicate_id_is_reported() {
        let errors = validate_graph(&[task("a", &[], &[]), task("a", &[], &[])]);
        assert_eq!(errors, vec![GraphError::DuplicateId { id: "a".into() }]);
    }

    #[test]
    fn dependencies_ready_requires_every_dependency() {
        let t = task("b", &["a", "c"], &[]);
        assert!(dependencies_ready(&t, &succeeded(&["a", "c"])));
        // One missing dependency holds the task: partial readiness is not ready.
        assert!(!dependencies_ready(&t, &succeeded(&["a"])));
        assert!(!dependencies_ready(&t, &HashSet::new()));
    }

    #[test]
    fn a_task_without_dependencies_is_always_ready() {
        // This is the backward-compatibility guarantee for every task written
        // before the graph existed.
        let t = task("a", &[], &[]);
        assert!(dependencies_ready(&t, &HashSet::new()));
    }

    #[test]
    fn tasks_triggered_by_matches_only_named_events() {
        let tasks = vec![
            task("a", &[], &["deploy"]),
            task("b", &[], &["deploy", "rollback"]),
            task("c", &[], &[]),
        ];
        let hit: Vec<&str> = tasks_triggered_by(&tasks, "deploy")
            .iter()
            .map(|t| t.id.as_str())
            .collect();
        assert_eq!(hit, vec!["a", "b"]);
        assert!(tasks_triggered_by(&tasks, "nope").is_empty());
    }

    #[test]
    fn a_disabled_task_is_never_event_triggered() {
        let mut t = task("a", &[], &["deploy"]);
        t.enabled = false;
        assert!(tasks_triggered_by(&[t], "deploy").is_empty());
    }

    #[test]
    fn a_due_task_is_held_until_its_dependency_succeeds() {
        // The real wiring: `due_tasks_in` must hold a task back while its
        // dependency has no successful run, then release it once it does.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let now = 1_000_000i64;

        let mut a = task("a", &[], &[]);
        a.created_at = now - 3_600;
        let mut b = task("b", &["a"], &[]);
        b.created_at = now - 3_600;
        save_in(root, &a).unwrap();
        save_in(root, &b).unwrap();

        let due: Vec<String> = due_tasks_in(root, now)
            .into_iter()
            .map(|(t, _)| t.id)
            .collect();
        assert_eq!(due, vec!["a".to_string()], "`b` must wait for `a`");

        // Record a *successful* run for `a`.
        save_run_in(
            root,
            "a",
            &RunRecord {
                run_id: "1-000000000".into(),
                task_id: "a".into(),
                status: RunStatus::Success,
                trigger: RunTrigger::Daemon,
                started_at: now - 60,
                finished_at: Some(now - 30),
                exit_code: Some(0),
                session_id: None,
                summary: None,
            },
        )
        .unwrap();

        // Sorted: `list_in` walks a directory, whose order is not guaranteed.
        let mut due: Vec<String> = due_tasks_in(root, now)
            .into_iter()
            .map(|(t, _)| t.id)
            .collect();
        due.sort();
        assert_eq!(due, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn a_failed_dependency_does_not_release_the_task() {
        // Only Success counts: an Error run must not unblock a dependent task.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let now = 1_000_000i64;

        let mut a = task("a", &[], &[]);
        a.created_at = now - 3_600;
        let mut b = task("b", &["a"], &[]);
        b.created_at = now - 3_600;
        save_in(root, &a).unwrap();
        save_in(root, &b).unwrap();

        save_run_in(
            root,
            "a",
            &RunRecord {
                run_id: "1-000000000".into(),
                task_id: "a".into(),
                status: RunStatus::Error,
                trigger: RunTrigger::Daemon,
                started_at: now - 60,
                finished_at: Some(now - 30),
                exit_code: Some(1),
                session_id: None,
                summary: None,
            },
        )
        .unwrap();

        let due: Vec<String> = due_tasks_in(root, now)
            .into_iter()
            .map(|(t, _)| t.id)
            .collect();
        assert_eq!(
            due,
            vec!["a".to_string()],
            "a failed dependency must still hold `b`"
        );
    }
}

#[test]
fn wakeup_register_claim_and_consume() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let id_due = mint_wakeup_id(now, 1);
    let id_future = mint_wakeup_id(now, 2);

    register_wakeup(&ScheduledWakeup {
        id: id_due.clone(),
        task_id: None,
        due_at: now - 10,
        prompt: "resume".into(),
        reason: "r".into(),
        created_at: now,
        generation: None,
        consumed: false,
    })
    .unwrap();
    register_wakeup(&ScheduledWakeup {
        id: id_future.clone(),
        task_id: None,
        due_at: now + 10_000,
        prompt: "later".into(),
        reason: "r".into(),
        created_at: now,
        generation: None,
        consumed: false,
    })
    .unwrap();

    // A path-escape id must be rejected (valid_id guard), exactly like tasks.
    let escaped = register_wakeup(&ScheduledWakeup {
        id: "../escape".into(),
        task_id: None,
        due_at: now,
        prompt: String::new(),
        reason: String::new(),
        created_at: now,
        generation: None,
        consumed: false,
    });
    assert!(
        escaped.is_err(),
        "wakeup id must not escape the wakeups dir"
    );

    // Only the due (not the future) wakeup is claimed.
    let due: Vec<String> = claim_due_wakeups(now, 0)
        .into_iter()
        .map(|w| w.id)
        .collect();
    assert!(due.contains(&id_due), "due wakeup must be claimable");
    assert!(
        !due.contains(&id_future),
        "future wakeup must not be claimable yet"
    );

    // Consuming makes it disappear from the due set (exactly-once).
    consume_wakeup(&id_due).unwrap();
    let due2: Vec<String> = claim_due_wakeups(now, 0)
        .into_iter()
        .map(|w| w.id)
        .collect();
    assert!(
        !due2.contains(&id_due),
        "consumed wakeup must not be re-claimed"
    );
    // Idempotent: consuming again is a no-op, not an error.
    consume_wakeup(&id_due).unwrap();
}

#[test]
fn wakeup_survives_process_restart() {
    // The registry is file-backed with no in-memory cache, so a brand-new
    // reader (simulating a process restart) must observe wakeups persisted by a
    // previous "process". This is the P3 cross-restart durability guarantee.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let root = std::env::temp_dir().join(format!("rc_wakeup_rt_{}", mint_wakeup_id(now, 7)));
    let _ = std::fs::create_dir_all(&root);

    let due_id = mint_wakeup_id(now, 8);
    let future_id = mint_wakeup_id(now, 9);
    register_wakeup_in(
        &root,
        &ScheduledWakeup {
            id: due_id.clone(),
            task_id: None,
            due_at: now - 5,
            prompt: "resume".into(),
            reason: "r".into(),
            created_at: now,
            generation: None,
            consumed: false,
        },
    )
    .unwrap();
    register_wakeup_in(
        &root,
        &ScheduledWakeup {
            id: future_id.clone(),
            task_id: None,
            due_at: now + 10_000,
            prompt: "later".into(),
            reason: "r".into(),
            created_at: now,
            generation: None,
            consumed: false,
        },
    )
    .unwrap();

    // Fresh reader (a new "process") sees both entries on disk.
    let all: Vec<String> = list_wakeups_in(&root).into_iter().map(|w| w.id).collect();
    assert!(all.contains(&due_id), "due wakeup must survive restart");
    assert!(
        all.contains(&future_id),
        "future wakeup must survive restart"
    );

    // And the due one is claimable by the fresh reader (restart-safe tick).
    let claimable: Vec<String> = list_wakeups_in(&root)
        .into_iter()
        .filter(|w| !w.consumed && w.due_at <= now)
        .map(|w| w.id)
        .collect();
    assert!(claimable.contains(&due_id));
    assert!(!claimable.contains(&future_id));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn wakeup_consumed_exactly_once() {
    // Consuming a wakeup must remove it from the due set so a later tick (or a
    // concurrent tick) cannot fire it twice. Idempotent re-consume is a no-op.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let root = std::env::temp_dir().join(format!("rc_wakeup_once_{}", mint_wakeup_id(now, 11)));
    let _ = std::fs::create_dir_all(&root);

    let id = mint_wakeup_id(now, 12);
    register_wakeup_in(
        &root,
        &ScheduledWakeup {
            id: id.clone(),
            task_id: None,
            due_at: now - 1,
            prompt: "do".into(),
            reason: "r".into(),
            created_at: now,
            generation: None,
            consumed: false,
        },
    )
    .unwrap();

    let claimable: Vec<String> = list_wakeups_in(&root)
        .into_iter()
        .filter(|w| !w.consumed && w.due_at <= now)
        .map(|w| w.id)
        .collect();
    assert!(claimable.contains(&id), "due wakeup must be claimable");

    consume_wakeup_in(&root, &id).unwrap();

    let claimable2: Vec<String> = list_wakeups_in(&root)
        .into_iter()
        .filter(|w| !w.consumed && w.due_at <= now)
        .map(|w| w.id)
        .collect();
    assert!(
        !claimable2.contains(&id),
        "consumed wakeup must not be re-claimable"
    );

    // Idempotent: consuming again is a no-op, not an error.
    consume_wakeup_in(&root, &id).unwrap();

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn stale_generation_wakeup_is_dropped_without_side_effect() {
    // A wakeup stamped with a generation that no longer matches the current
    // runtime generation (its creating runtime was replaced) must be dropped by
    // claim_due_wakeups: not returned for firing, and left on disk untouched
    // (never consumed, never fired) so it can never cause a run.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let root = std::env::temp_dir().join(format!("rc_wakeup_gen_{}", mint_wakeup_id(now, 21)));
    let _ = std::fs::create_dir_all(&root);

    // Stale wakeup: created by generation 5, but current generation is 0.
    let stale_id = mint_wakeup_id(now, 22);
    register_wakeup_in(
        &root,
        &ScheduledWakeup {
            id: stale_id.clone(),
            task_id: None,
            due_at: now - 1,
            prompt: "stale".into(),
            reason: "r".into(),
            created_at: now,
            generation: Some(5),
            consumed: false,
        },
    )
    .unwrap();

    // Current-generation wakeup: must still be claimable.
    let live_id = mint_wakeup_id(now, 23);
    register_wakeup_in(
        &root,
        &ScheduledWakeup {
            id: live_id.clone(),
            task_id: None,
            due_at: now - 1,
            prompt: "live".into(),
            reason: "r".into(),
            created_at: now,
            generation: Some(0),
            consumed: false,
        },
    )
    .unwrap();

    // Claim with current generation 0: stale dropped, live claimed.
    let claimed: Vec<String> = claim_due_wakeups_in(&root, now, 0)
        .into_iter()
        .map(|w| w.id)
        .collect();
    assert!(
        !claimed.contains(&stale_id),
        "stale wakeup must be dropped from the claim set"
    );
    assert!(
        claimed.contains(&live_id),
        "current-generation wakeup must be claimed"
    );

    // Dropping must be side-effect-free: the stale entry stays on disk and
    // is not consumed, so a later tick cannot fire or re-evaluate it.
    let stale_path = root.join(format!("{stale_id}.json"));
    assert!(
        stale_path.exists(),
        "stale wakeup file must remain on disk (no side effect on drop)"
    );
    let stale: ScheduledWakeup =
        serde_json::from_slice(&std::fs::read(&stale_path).unwrap()).unwrap();
    assert!(!stale.consumed, "stale wakeup must not be consumed on drop");

    let _ = std::fs::remove_dir_all(&root);
}
