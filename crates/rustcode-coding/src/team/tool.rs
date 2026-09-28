use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use rustcode_capabilities::team::{role_by_id, TeamRoleId, TeamRunId, TeamTaskSpec};
use rustcode_kernel::tool::{RiskLevel, Tool, ToolContext, ToolResult};
use serde::Deserialize;
use serde_json::{json, Value};

use super::{TeamJobFactory, TeamModelFactory, TeamRunManager, TeamRunSnapshot};

const DEFAULT_WAIT_SECS: u64 = 30;
const MAX_WAIT_SECS: u64 = 300;

#[derive(Clone)]
pub struct TeamTool {
    manager: TeamRunManager,
    jobs: TeamJobFactory,
    models: TeamModelFactory,
}

impl TeamTool {
    pub fn new(manager: TeamRunManager, jobs: TeamJobFactory, models: TeamModelFactory) -> Self {
        Self {
            manager,
            jobs,
            models,
        }
    }

    fn result(content: impl Into<String>, is_error: bool) -> ToolResult {
        ToolResult {
            call_id: String::new(),
            content: content.into(),
            is_error,
            images: Vec::new(),
        }
    }

    fn json_result(value: Value) -> ToolResult {
        Self::result(
            serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_string()),
            false,
        )
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
enum TeamArgs {
    Delegate {
        #[serde(deserialize_with = "de_tasks_lenient")]
        tasks: Vec<DelegateTask>,
    },
    Status {
        #[serde(default)]
        run_id: Option<String>,
    },
    Wait {
        run_id: String,
        #[serde(default = "default_wait_secs")]
        timeout_secs: u64,
    },
    Result {
        run_id: String,
    },
    Stop {
        run_id: String,
    },
}

#[derive(Debug, Deserialize)]
struct DelegateTask {
    description: String,
    prompt: String,
    role: TeamRoleId,
    /// Accepted as an alias for the lane the `role` belongs to, because the `task`
    /// tool spells the same concept `subagent_type` and models carry that name
    /// over. It is validated against `role`, never a second source of truth: a
    /// disagreement is an explicit batch error rather than a silent downgrade
    /// (the old behavior ignored the field entirely) or a silent escalation.
    #[serde(default)]
    subagent_type: Option<String>,
    #[serde(default, deserialize_with = "de_scope_lenient")]
    scope: Vec<String>,
}

fn default_wait_secs() -> u64 {
    DEFAULT_WAIT_SECS
}

fn json_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "a boolean",
        Value::Number(_) => "a number",
        Value::String(_) => "a string",
        Value::Array(_) => "an array",
        Value::Object(_) => "an object",
    }
}

fn de_tasks_lenient<'de, D>(deserializer: D) -> Result<Vec<DelegateTask>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    de_vec_lenient(deserializer, "tasks")
}

fn de_scope_lenient<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    de_vec_lenient(deserializer, "scope")
}

fn de_vec_lenient<'de, D, T>(deserializer: D, field: &str) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::de::DeserializeOwned,
{
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Array(_) => {
            serde_json::from_value::<Vec<T>>(value).map_err(serde::de::Error::custom)
        }
        Value::String(text) => serde_json::from_str::<Vec<T>>(text.trim()).map_err(|error| {
            let snippet: String = text.chars().take(80).collect();
            serde::de::Error::custom(format!(
                "expected a JSON array or a JSON-encoded array string in `{field}`, got \"{snippet}\": {error}"
            ))
        }),
        other => Err(serde::de::Error::custom(format!(
            "expected a JSON array or a JSON-encoded array string in `{field}`, got {}",
            json_type_name(&other)
        ))),
    }
}

/// Rebuild `TeamArgs`-shaped JSON from shapes serde cannot read directly.
///
/// `TeamArgs` is an internally tagged enum (`#[serde(tag = "action")]`), so it only
/// deserializes from an object that carries `action`. A bare task array or a
/// double-encoded string is a hard serde error whose message the model cannot learn
/// from, so it keeps guessing. Returns `None` when genuinely unsalvageable -- the
/// caller then replays the original parse error rather than faking success.
fn salvage_team_args(args: &str) -> Option<Value> {
    let mut value = serde_json::from_str::<Value>(args).ok()?;
    // Rung 1: the whole payload arrived as a JSON-encoded string.
    if let Value::String(inner) = &value {
        value = serde_json::from_str::<Value>(inner.trim()).ok()?;
    }
    match value {
        // Rung 2: a bare task array -> `delegate.tasks`.
        Value::Array(items) => Some(json!({"action": "delegate", "tasks": items})),
        Value::Object(mut map) => {
            let action = map.get("action").and_then(Value::as_str).unwrap_or("");
            if matches!(action, "delegate" | "status" | "wait" | "result" | "stop") {
                return Some(Value::Object(map));
            }
            // Rung 3: task list present but `action` missing/misspelled -> delegate.
            if map.contains_key("tasks") {
                map.insert("action".into(), json!("delegate"));
                return Some(Value::Object(map));
            }
            // Rung 4: a single task object -> wrap it in `tasks`.
            if map.contains_key("description") || map.contains_key("prompt") {
                return Some(json!({"action": "delegate", "tasks": [Value::Object(map)]}));
            }
            None
        }
        _ => None,
    }
}

/// Parse the team tool args, repairing malformed weak-model output on failure.
/// Repairs ONLY on failure, so well-formed arguments are never altered.
///
/// Ladder: direct parse -> `repair_json` (control chars / trailing commas) ->
/// `salvage_team_args` (shape rescue). Shared by `risk` and `execute` so both
/// agree on whether a dispatch contains a write-authority role: if they disagreed,
/// a scoped-write worker could be rated `Safe` at the approval gate and then run
/// as a Worker (or be denied after being rated Risky).
fn parse_team_args(args: &str) -> Result<TeamArgs, String> {
    if let Ok(parsed) = serde_json::from_str::<TeamArgs>(args) {
        return Ok(parsed);
    }
    if let Ok(parsed) =
        serde_json::from_str::<TeamArgs>(&rustcode_capabilities::tools::repair::repair_json(args))
    {
        return Ok(parsed);
    }
    if let Some(salvaged) = salvage_team_args(args) {
        if let Ok(parsed) = serde_json::from_value::<TeamArgs>(salvaged) {
            return Ok(parsed);
        }
    }
    // Reproduce the original parse error so the caller can surface it to the model.
    serde_json::from_str::<TeamArgs>(args).map_err(|error| format!("invalid team args: {error}"))
}

fn task_spec(task: DelegateTask) -> Result<TeamTaskSpec, String> {
    if task.description.trim().is_empty() || task.prompt.trim().is_empty() {
        return Err("team task description and prompt must not be empty".into());
    }
    let profile = role_by_id(task.role.as_str())
        .ok_or_else(|| format!("unknown team role: {}", task.role))?;
    // `subagent_type` is an accepted alias, not a second authority: here it only
    // has to AGREE with the role's lane. Rejecting the disagreement beats the
    // silent alternatives -- ignoring it (the old behavior, which let a model
    // believe it had escalated) or letting it override `role` (silent escalation).
    if let Some(requested) = task.subagent_type.as_deref() {
        let expected = match profile.permission {
            rustcode_capabilities::team::TeamPermission::Worker => "worker",
            rustcode_capabilities::team::TeamPermission::Explore => "explore",
        };
        if requested != expected {
            return Err(format!(
                "team task `{}`: subagent_type \"{requested}\" does not match role \"{}\" ({expected}); \
                 drop subagent_type or use \"{expected}\"",
                task.description, task.role
            ));
        }
    }
    Ok(TeamTaskSpec {
        description: task.description,
        prompt: task.prompt,
        role: task.role,
        permission: profile.permission,
        difficulty: profile.difficulty,
        scope: task.scope,
    })
}

fn snapshot_json(run: &TeamRunSnapshot, include_results: bool) -> Value {
    json!({
        "run_id": run.run_id,
        "generation": run.generation,
        "total": run.total,
        "completed": run.completed,
        "failed": run.failed,
        "stopped": run.stopped,
        "terminal": run.completed + run.failed + run.stopped == run.total,
        "members": run.members.iter().map(|member| json!({
            "id": member.id,
            "role": member.role,
            "status": format!("{:?}", member.status).to_ascii_lowercase(),
            "result": include_results.then_some(member.result.as_str()),
        })).collect::<Vec<_>>(),
    })
}

#[async_trait]
impl Tool for TeamTool {
    fn name(&self) -> &str {
        "team"
    }

    fn description(&self) -> &str {
        "Run and manage a persistent team of specialized child agents. Use `delegate` with one or more independent tasks, then `status`, `wait`, or `result` with the returned run_id; use `stop` to cancel a run. SHAPE NOTE: unlike the `task` tool, every call is wrapped in an action envelope -- {\"action\":\"delegate\",\"tasks\":[...]} -- and each task's lane is chosen by its `role` alone (no `subagent_type`). Roles determine read-only vs scoped-write authority and fast vs capable model routing. Worker roles require a non-empty working-directory-relative scope and cannot run Bash. Supports hierarchical dispatch: a child agent may itself delegate to a deeper tier up to the configured max_depth, enabling three-tier blackboard tree fan-out."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "oneOf": [
                {
                    "properties": {
                        "action": {"const": "delegate"},
                        "tasks": {
                            "type": "array", "minItems": 1,
                            "items": {
                                "type": "object",
                                "properties": {
                                    "description": {"type": "string"},
                                    "prompt": {"type": "string"},
                                    "role": {"type": "string", "enum": ["planner", "architect", "explorer", "implementer", "rust", "tui_ux", "reviewer", "tester", "debugger", "security", "performance", "docs_writer", "release_manager", "migration_compat"]},
                                    "subagent_type": {"type": "string", "enum": ["explore", "worker"], "description": "Optional alias: must agree with `role`'s lane (read-only roles -> \"explore\", write roles -> \"worker\"). `role` alone is enough."},
                                    "scope": {"type": "array", "items": {"type": "string"}, "description": "Required for worker roles; ignored for read-only roles."}
                                },
                                "required": ["description", "prompt", "role"]
                            }
                        }
                    },
                    "required": ["action", "tasks"]
                },
                {"properties": {"action": {"const": "status"}, "run_id": {"type": "string"}}, "required": ["action"]},
                {"properties": {"action": {"const": "wait"}, "run_id": {"type": "string"}, "timeout_secs": {"type": "integer", "minimum": 0, "maximum": MAX_WAIT_SECS}}, "required": ["action", "run_id"]},
                {"properties": {"action": {"const": "result"}, "run_id": {"type": "string"}}, "required": ["action", "run_id"]},
                {"properties": {"action": {"const": "stop"}, "run_id": {"type": "string"}}, "required": ["action", "run_id"]}
            ]
        })
    }

    fn risk(&self, args: &str) -> RiskLevel {
        match parse_team_args(args) {
            Ok(TeamArgs::Delegate { tasks })
                if tasks.iter().any(|task| {
                    role_by_id(task.role.as_str()).is_some_and(|profile| {
                        profile.permission == rustcode_capabilities::team::TeamPermission::Worker
                    })
                }) =>
            {
                RiskLevel::Risky
            }
            _ => RiskLevel::Safe,
        }
    }

    async fn execute(&self, args: &str, ctx: &ToolContext) -> ToolResult {
        let parsed = match parse_team_args(args) {
            Ok(parsed) => parsed,
            Err(error) => return Self::result(error, true),
        };
        let outcome: Result<Value, String> = match parsed {
            TeamArgs::Delegate { tasks } => {
                let tasks = tasks
                    .into_iter()
                    .map(task_spec)
                    .collect::<Result<Vec<_>, _>>();
                match tasks {
                    Ok(tasks) => self
                        .manager
                        .delegate(tasks, Arc::clone(&self.jobs), Arc::clone(&self.models))
                        .await
                        .map(|run_id| json!({"run_id": run_id, "status": "running"})),
                    Err(error) => Err(error),
                }
            }
            TeamArgs::Status { run_id } => {
                let selected = run_id.as_deref().map(TeamRunId::new);
                let depth = self.manager.depth();
                self.manager.snapshot(selected.as_ref()).map(|snapshot| {
                    json!({
                        "depth": depth,
                        "runs": snapshot.runs.iter().map(|run| snapshot_json(run, false)).collect::<Vec<_>>()
                    })
                }).or_else(|| run_id.is_none().then(|| json!({"depth": depth, "runs": []})))
                  .ok_or_else(|| format!("unknown team run: {}", run_id.unwrap_or_default()))
            }
            TeamArgs::Wait {
                run_id,
                timeout_secs,
            } => {
                let run_id = TeamRunId::new(run_id);
                let timeout = Duration::from_secs(timeout_secs.min(MAX_WAIT_SECS));
                tokio::select! {
                    wait = self.manager.wait(&run_id, timeout) => wait.and_then(|wait| {
                        self.manager.snapshot(Some(&run_id))
                            .and_then(|snapshot| snapshot.runs.first().cloned())
                            .map(|run| snapshot_json(&run, wait.terminal))
                            .ok_or_else(|| format!("unknown team run: {run_id}"))
                    }),
                    _ = ctx.cancel.cancelled() => Err("team wait cancelled".into()),
                }
            }
            TeamArgs::Result { run_id } => {
                let run_id = TeamRunId::new(run_id);
                self.manager
                    .snapshot(Some(&run_id))
                    .and_then(|snapshot| snapshot.runs.first().cloned())
                    .map(|run| snapshot_json(&run, true))
                    .ok_or_else(|| format!("unknown team run: {run_id}"))
            }
            TeamArgs::Stop { run_id } => {
                let run_id = TeamRunId::new(run_id);
                self.manager
                    .stop(&run_id)
                    .await
                    .map(|()| json!({"run_id": run_id, "status": "stopped"}))
            }
        };
        match outcome {
            Ok(value) => Self::json_result(value),
            Err(error) => Self::result(error, true),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustcode_capabilities::team::TeamPermission;
    use rustcode_kernel::tool::{ProgressSink, ToolContext};
    use tokio_util::sync::CancellationToken;

    fn ctx() -> ToolContext {
        ToolContext {
            working_dir: std::env::temp_dir(),
            cancel: CancellationToken::new(),
            progress: ProgressSink::noop(),
            requester: None,
        }
    }

    fn tool(max_result_chars: usize) -> TeamTool {
        let manager = TeamRunManager::new(super::super::TeamRuntimeConfig {
            max_result_chars,
            cancel_grace: Duration::from_millis(1),
            ..Default::default()
        });
        manager.begin_generation(4);
        let jobs: TeamJobFactory = Arc::new(|task, cancel, _activity| {
            Box::pin(async move {
                if task.description == "block" {
                    cancel.cancelled().await;
                    return super::super::TeamMemberOutcome::failed("cancelled");
                }
                super::super::TeamMemberOutcome::completed("abcdefghij")
            })
        });
        TeamTool::new(manager, jobs, Arc::new(|_| "test-model".to_string()))
    }

    #[tokio::test]
    async fn delegate_status_wait_result_and_stop() {
        let tool = tool(5);
        let empty = tool.execute(r#"{"action":"status"}"#, &ctx()).await;
        assert!(!empty.is_error, "{}", empty.content);
        assert_eq!(
            serde_json::from_str::<Value>(&empty.content).unwrap()["runs"],
            json!([])
        );
        let delegated = tool.execute(r#"{"action":"delegate","tasks":[{"description":"read","prompt":"inspect","role":"explorer"}]}"#, &ctx()).await;
        assert!(!delegated.is_error, "{}", delegated.content);
        let run_id = serde_json::from_str::<Value>(&delegated.content).unwrap()["run_id"]
            .as_str()
            .unwrap()
            .to_string();
        let waited = tool
            .execute(
                &format!(r#"{{"action":"wait","run_id":"{run_id}","timeout_secs":1}}"#),
                &ctx(),
            )
            .await;
        assert_eq!(
            serde_json::from_str::<Value>(&waited.content).unwrap()["terminal"],
            true
        );
        assert_eq!(
            serde_json::from_str::<Value>(&waited.content).unwrap()["members"][0]["result"],
            "abcde..."
        );
        let result = tool
            .execute(
                &format!(r#"{{"action":"result","run_id":"{run_id}"}}"#),
                &ctx(),
            )
            .await;
        assert!(result.content.contains("abcde..."));

        let blocked = tool.execute(r#"{"action":"delegate","tasks":[{"description":"block","prompt":"wait","role":"explorer"}]}"#, &ctx()).await;
        let blocked_value = serde_json::from_str::<Value>(&blocked.content).unwrap();
        let blocked_id = blocked_value["run_id"].as_str().unwrap().to_string();
        let stopped = tool
            .execute(
                &format!(r#"{{"action":"stop","run_id":"{blocked_id}"}}"#),
                &ctx(),
            )
            .await;
        assert!(!stopped.is_error, "{}", stopped.content);
    }

    #[tokio::test]
    async fn malformed_unknown_and_worker_without_scope_fail_closed() {
        let tool = tool(100);
        assert!(tool.execute("{", &ctx()).await.is_error);
        assert!(
            tool.execute(r#"{"action":"status","run_id":"missing"}"#, &ctx())
                .await
                .is_error
        );
        let worker = tool.execute(r#"{"action":"delegate","tasks":[{"description":"edit","prompt":"change","role":"rust"}]}"#, &ctx()).await;
        assert!(worker.is_error);
        assert!(worker.content.contains("non-empty scope"));
        assert_eq!(tool.risk(r#"{"action":"delegate","tasks":[{"description":"edit","prompt":"change","role":"rust","scope":["src/**"]}]}"#), RiskLevel::Risky);
        assert_eq!(
            role_by_id("rust").unwrap().permission,
            TeamPermission::Worker
        );
    }

    #[test]
    fn risk_is_safe_for_read_only_roles_and_risky_for_workers() {
        let tool = tool(100);
        // 只读角色（explorer）delegate -> Safe。
        assert_eq!(
            tool.risk(r#"{"action":"delegate","tasks":[{"description":"read","prompt":"inspect","role":"explorer"}]}"#),
            RiskLevel::Safe
        );
        // worker 角色（rust，带 scope）delegate -> Risky。
        assert_eq!(
            tool.risk(r#"{"action":"delegate","tasks":[{"description":"edit","prompt":"change","role":"rust","scope":["src/**"]}]}"#),
            RiskLevel::Risky
        );
        // 非 delegate 动作（status/wait）-> Safe。
        assert_eq!(tool.risk(r#"{"action":"status"}"#), RiskLevel::Safe);
        assert_eq!(
            tool.risk(r#"{"action":"wait","run_id":"team-1-1","timeout_secs":1}"#),
            RiskLevel::Safe
        );
    }

    #[tokio::test]
    async fn wait_timeout_is_capped_and_unknown_run_fails_closed() {
        let tool = tool(100);
        // 未知 run：wait/result/stop 都应报错。
        let waited = tool
            .execute(
                r#"{"action":"wait","run_id":"missing","timeout_secs":1}"#,
                &ctx(),
            )
            .await;
        assert!(waited.is_error);
        assert!(waited.content.contains("unknown team run"));
        let resulted = tool
            .execute(r#"{"action":"result","run_id":"missing"}"#, &ctx())
            .await;
        assert!(resulted.is_error);
        assert!(resulted.content.contains("unknown team run"));
        let stopped = tool
            .execute(r#"{"action":"stop","run_id":"missing"}"#, &ctx())
            .await;
        assert!(stopped.is_error);
        assert!(stopped.content.contains("unknown team run"));
    }

    #[tokio::test]
    async fn delegate_rejects_empty_description_or_prompt() {
        let tool = tool(100);
        // description/prompt 为空 -> task_spec 拒绝。
        let empty = tool
            .execute(
                r#"{"action":"delegate","tasks":[{"description":"","prompt":"","role":"explorer"}]}"#,
                &ctx(),
            )
            .await;
        assert!(empty.is_error);
        assert!(empty.content.contains("must not be empty"));
        // 只缺 prompt -> 同样拒绝。
        let no_prompt = tool
            .execute(
                r#"{"action":"delegate","tasks":[{"description":"read","prompt":"","role":"explorer"}]}"#,
                &ctx(),
            )
            .await;
        assert!(no_prompt.is_error);
        assert!(no_prompt.content.contains("must not be empty"));
        // 空任务数组 -> delegate 拒绝。
        let no_tasks = tool
            .execute(r#"{"action":"delegate","tasks":[]}"#, &ctx())
            .await;
        assert!(no_tasks.is_error);
    }

    #[test]
    fn delegate_tasks_accepts_real_array() {
        let args: TeamArgs = serde_json::from_str(
            r#"{"action":"delegate","tasks":[{"description":"read","prompt":"inspect","role":"explorer"}]}"#,
        )
        .unwrap();
        match args {
            TeamArgs::Delegate { tasks } => {
                assert_eq!(tasks.len(), 1);
                assert_eq!(tasks[0].description, "read");
                assert_eq!(tasks[0].prompt, "inspect");
                assert_eq!(tasks[0].role.as_str(), "explorer");
                assert!(tasks[0].scope.is_empty());
            }
            _ => panic!("expected delegate args"),
        }
    }

    #[test]
    fn delegate_tasks_accepts_json_encoded_string() {
        let args: TeamArgs = serde_json::from_str(
            r#"{"action":"delegate","tasks":"[{\"description\":\"read\",\"prompt\":\"inspect\",\"role\":\"explorer\",\"scope\":[\"src/**\"]}]"}"#,
        )
        .unwrap();
        match args {
            TeamArgs::Delegate { tasks } => {
                assert_eq!(tasks.len(), 1);
                assert_eq!(tasks[0].description, "read");
                assert_eq!(tasks[0].prompt, "inspect");
                assert_eq!(tasks[0].role.as_str(), "explorer");
                assert_eq!(tasks[0].scope, vec!["src/**".to_string()]);
            }
            _ => panic!("expected delegate args"),
        }
    }

    #[test]
    fn delegate_tasks_reject_malformed_string_with_field_name() {
        let error = serde_json::from_str::<TeamArgs>(r#"{"action":"delegate","tasks":"not json"}"#)
            .unwrap_err();
        assert!(error.to_string().contains("tasks"), "{}", error);
    }

    // --- parse ladder: shapes a weak model actually emits (see salvage_team_args) ---

    fn delegated(args: &str) -> Vec<rustcode_capabilities::team::TeamTaskSpec> {
        match parse_team_args(args).expect("should parse") {
            TeamArgs::Delegate { tasks } => tasks
                .into_iter()
                .map(task_spec)
                .collect::<Result<Vec<_>, _>>()
                .expect("tasks should validate"),
            other => panic!("expected delegate, got {other:?}"),
        }
    }

    #[test]
    fn ladder_accepts_a_bare_task_array() {
        let tasks = delegated(r#"[{"description":"read","prompt":"inspect","role":"explorer"}]"#);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].description, "read");
    }

    #[test]
    fn ladder_unwraps_a_double_encoded_payload() {
        let inner = r#"{"action":"delegate","tasks":[{"description":"read","prompt":"inspect","role":"explorer"}]}"#;
        let tasks = delegated(&serde_json::to_string(inner).unwrap());
        assert_eq!(tasks.len(), 1);
    }

    #[test]
    fn ladder_supplies_delegate_when_action_is_missing() {
        let tasks =
            delegated(r#"{"tasks":[{"description":"read","prompt":"inspect","role":"explorer"}]}"#);
        assert_eq!(tasks.len(), 1);
    }

    #[test]
    fn ladder_wraps_a_single_task_object() {
        let tasks = delegated(r#"{"description":"read","prompt":"inspect","role":"explorer"}"#);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].description, "read");
    }

    #[test]
    fn ladder_still_fails_closed_on_unsalvageable_input() {
        for hopeless in ["{", r#"{"action":"nonsense"}"#, r#"{"unrelated":1}"#] {
            let error = parse_team_args(hopeless).unwrap_err();
            assert!(
                error.starts_with("invalid team args:"),
                "{hopeless}: {error}"
            );
        }
    }

    #[test]
    fn risk_and_execute_share_one_parser_for_worker_dispatches() {
        // A `rust` (write-authority) task reached through the ladder must still be
        // rated Risky, or the approval gate would let a scoped write through.
        let tool = tool(100);
        let args = r#"[{"description":"edit","prompt":"change","role":"rust","scope":["src/**"]}]"#;
        assert_eq!(tool.risk(args), RiskLevel::Risky);
        assert!(matches!(
            parse_team_args(args),
            Ok(TeamArgs::Delegate { .. })
        ));
    }

    // --- RC-5: `subagent_type` is accepted as an alias on `team` (the `task` tool
    //     spells the lane that way), but `role` stays the single authority ---

    #[test]
    fn subagent_type_alias_is_accepted_when_it_agrees_with_the_role() {
        let tasks = delegated(
            r#"{"action":"delegate","tasks":[{"description":"edit","prompt":"change","role":"rust","subagent_type":"worker","scope":["src/**"]}]}"#,
        );
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].role.as_str(), "rust");
        assert_eq!(
            tasks[0].permission,
            rustcode_capabilities::team::TeamPermission::Worker
        );
    }

    #[test]
    fn subagent_type_alias_conflict_is_rejected_not_silently_ignored() {
        // `reviewer` is read-only; claiming the write lane must fail loudly rather
        // than run as Explore (silent downgrade) or as Worker (silent escalation).
        let err = task_spec(single_task(
            r#"{"description":"audit","prompt":"check","role":"reviewer","subagent_type":"worker"}"#,
        ))
        .unwrap_err();
        assert!(
            err.contains("does not match role"),
            "expected an explicit lane mismatch, got: {err}"
        );
    }

    #[test]
    fn omitting_subagent_type_keeps_working_as_before() {
        let tasks = delegated(
            r#"{"action":"delegate","tasks":[{"description":"read","prompt":"inspect","role":"explorer"}]}"#,
        );
        assert_eq!(tasks.len(), 1);
        assert_eq!(
            tasks[0].permission,
            rustcode_capabilities::team::TeamPermission::Explore
        );
    }

    fn single_task(json: &str) -> DelegateTask {
        serde_json::from_str::<DelegateTask>(json).expect("fixture must deserialize")
    }

    #[test]
    fn team_schema_advertises_the_subagent_type_alias() {
        let schema = tool(100).parameters_schema();
        let items = &schema["oneOf"][0]["properties"]["tasks"]["items"];
        let alias = &items["properties"]["subagent_type"];
        assert_eq!(alias["type"], "string");
        let enum_values: Vec<&str> = alias["enum"]
            .as_array()
            .expect("subagent_type enum")
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(enum_values, vec!["explore", "worker"]);
    }
}
