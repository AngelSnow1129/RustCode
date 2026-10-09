//! `schedule_task` —— agent 自主派生定时任务（P2.5 静态 DAG 的写入侧）。
//!
//! 设计文档 §5.4.2：工具**只写 schedule store、不立即运行、不注册 OS 调度器**，由 daemon
//! tick / `schedule tick --once` 子进程到期触发（与 P2 已落地的「执行经 CLI 子进程 IPC」哲学一致）。
//! `cwd` 必须落在当前 runtime `working_dir` 之内（对齐 `BashWorkspaceGate` 越界语义）。
//!
//! 分层：本工具是 `rustcode-capabilities` 中唯一需要 `rustcode-config` 的 coding 工具，故置于
//! 按需 `schedule` feature 之后，lean tools-only embedder 不编译它，避免拖入 config 传递依赖。

use super::{err, ok};
use async_trait::async_trait;
use rustcode_config::schedule::{list, save, validate_graph, Schedule, ScheduleTask};
use rustcode_kernel::tool::{Tool, ToolContext, ToolResult};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const SCHEDULE_TASK_DESCRIPTION: &str = "Agent 自主派生定时任务（P2.5）。直写 schedule store，由 daemon tick / `schedule tick --once` 到期触发；工具本身不注册 OS 调度器、不立即运行。\
参数：title 任务标题（兼作 id 的 slug）、prompt 触发时交给 agent 的提示词、cwd 运行目录（必须落在 runtime working_dir 之内）、\
when{kind,time,weekday,every_minutes,expr} 调度时间（kind 为 daily|weekly|hourly|interval|cron）、\
depends_on 任务 id 列表（构成 DAG，依赖全部成功后才触发本任务）、triggers 事件名列表（除自身 schedule 外的事件触发源）、permission_mode 默认 plan。\
保存前做依赖图环检测与依赖存在性校验，失败则返回错误且不写入。";

/// `title` → 文件系统/文件名安全的 slug（小写字母数字 + `-`）。
fn slugify(title: &str) -> String {
    let mut out = String::new();
    for c in title.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

fn now_epoch() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// `<slug>-<6位纳秒尾数>`，写入前再与 `list()` 去重。
fn gen_id(title: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{}-{}", slugify(title), nanos % 1_000_000)
}

/// 把 `cwd` 解析为绝对路径并确认其落在 `base`（runtime working_dir）之内。
/// 相对 `cwd` 相对 `base` 解析；任一 canonicalize 失败时回退到原始路径做前缀比较。
fn cwd_in_bounds(cwd: &str, base: &Path) -> Result<PathBuf, String> {
    let base = base.canonicalize().unwrap_or_else(|_| base.to_path_buf());
    let target = if Path::new(cwd).is_absolute() {
        PathBuf::from(cwd)
    } else {
        base.join(cwd)
    };
    let target = target.canonicalize().unwrap_or(target);
    if target.starts_with(&base) {
        Ok(target)
    } else {
        Err(format!(
            "schedule_task: `cwd` `{}` 必须落在 runtime working_dir `{}` 之内",
            cwd,
            base.display()
        ))
    }
}

fn build_schedule(v: &serde_json::Value) -> Result<Schedule, String> {
    let when = v
        .get("when")
        .and_then(|w| w.as_object())
        .ok_or_else(|| "schedule_task: 缺少 `when` 对象".to_string())?;
    let kind = when.get("kind").and_then(|k| k.as_str()).ok_or_else(|| {
        "schedule_task: `when.kind` 必填 (daily|weekly|hourly|interval|cron)".to_string()
    })?;
    match kind {
        "hourly" => Ok(Schedule::Hourly),
        "daily" => {
            let time = when
                .get("time")
                .and_then(|t| t.as_str())
                .ok_or_else(|| "schedule_task: daily 需要 `when.time` (HH:MM)".to_string())?;
            Ok(Schedule::Daily {
                time: time.to_string(),
            })
        }
        "weekly" => {
            let weekday = when
                .get("weekday")
                .and_then(|w| w.as_u64())
                .ok_or_else(|| "schedule_task: weekly 需要 `when.weekday` (1..=7)".to_string())?
                as u8;
            let time = when
                .get("time")
                .and_then(|t| t.as_str())
                .ok_or_else(|| "schedule_task: weekly 需要 `when.time` (HH:MM)".to_string())?;
            Ok(Schedule::Weekly {
                weekday,
                time: time.to_string(),
            })
        }
        "interval" => {
            let every_minutes = when
                .get("every_minutes")
                .and_then(|e| e.as_u64())
                .ok_or_else(|| "schedule_task: interval 需要 `when.every_minutes`".to_string())?
                as u32;
            Ok(Schedule::Interval { every_minutes })
        }
        "cron" => {
            let expr = when
                .get("expr")
                .and_then(|e| e.as_str())
                .ok_or_else(|| "schedule_task: cron 需要 `when.expr` (五段)".to_string())?;
            Ok(Schedule::Cron {
                expr: expr.to_string(),
            })
        }
        other => Err(format!("schedule_task: 不支持的 `when.kind` `{other}`")),
    }
}

fn str_array(v: &serde_json::Value, key: &str) -> Vec<String> {
    v.get(key)
        .and_then(|a| a.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

#[derive(Clone, Default)]
pub struct ScheduleTaskTool;

impl ScheduleTaskTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for ScheduleTaskTool {
    fn name(&self) -> &str {
        "schedule_task"
    }
    fn description(&self) -> &str {
        SCHEDULE_TASK_DESCRIPTION
    }
    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "title": { "type": "string", "description": "任务标题（同时用于生成任务 id 的 slug）" },
                "prompt": { "type": "string", "description": "任务触发时交给 agent 的提示词" },
                "cwd": { "type": "string", "description": "任务运行目录，必须落在当前 runtime working_dir 之内（相对则相对 working_dir）" },
                "when": {
                    "type": "object",
                    "description": "调度时间",
                    "properties": {
                        "kind": { "type": "string", "enum": ["daily", "weekly", "hourly", "interval", "cron"] },
                        "time": { "type": "string", "description": "daily / weekly 用 HH:MM" },
                        "weekday": { "type": "integer", "description": "weekly 用 1..=7 (1=Mon)" },
                        "every_minutes": { "type": "integer", "description": "interval 用" },
                        "expr": { "type": "string", "description": "cron 用五段表达式" }
                    },
                    "required": ["kind"]
                },
                "depends_on": { "type": "array", "items": { "type": "string" }, "description": "本任务触发前必须成功的任务 id（构成 DAG，可空）" },
                "triggers": { "type": "array", "items": { "type": "string" }, "description": "除自身 schedule 外，可被这些事件名触发的事件源（可空）" },
                "permission_mode": { "type": "string", "enum": ["plan", "accept_edits", "auto"], "description": "运行权限模式，默认 plan" }
            },
            "required": ["title", "prompt", "cwd", "when"]
        })
    }
    // 只写 schedule store（~/.rustcode/schedules/<id>.json），不触碰 OS / 不立即运行 -> 默认 Safe。
    fn always_grant_scope(&self, _args: &str) -> String {
        String::new()
    }
    async fn execute(&self, args: &str, ctx: &ToolContext) -> ToolResult {
        let v: serde_json::Value = match serde_json::from_str(args) {
            Ok(v) => v,
            Err(e) => return err(format!("schedule_task: 无效 JSON 参数: {e}")),
        };
        let title = v.get("title").and_then(|t| t.as_str()).unwrap_or("").trim();
        if title.is_empty() {
            return err("schedule_task: `title` 必填".to_string());
        }
        let prompt = v
            .get("prompt")
            .and_then(|p| p.as_str())
            .unwrap_or("")
            .trim();
        if prompt.is_empty() {
            return err("schedule_task: `prompt` 必填".to_string());
        }
        let cwd = v.get("cwd").and_then(|c| c.as_str()).unwrap_or("").trim();
        if cwd.is_empty() {
            return err("schedule_task: `cwd` 必填".to_string());
        }
        let cwd_abs = match cwd_in_bounds(cwd, &ctx.working_dir) {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(e) => return err(e),
        };
        let schedule = match build_schedule(&v) {
            Ok(s) => s,
            Err(e) => return err(e),
        };
        let depends_on = str_array(&v, "depends_on");
        let triggers = str_array(&v, "triggers");
        let permission_mode = v
            .get("permission_mode")
            .and_then(|m| m.as_str())
            .unwrap_or("plan")
            .to_string();

        let id = gen_id(title);
        let task = ScheduleTask {
            id: id.clone(),
            title: title.to_string(),
            prompt: prompt.to_string(),
            cwd: cwd_abs,
            schedule,
            permission_mode,
            notify: "important".into(),
            enabled: true,
            created_at: now_epoch(),
            last_run_at: None,
            last_status: None,
            last_run_id: None,
            depends_on,
            triggers,
        };

        // 校验：依赖环（validate_graph 三色 DFS）+ 依赖存在性。
        let mut all = list();
        all.push(task.clone());
        let errors = validate_graph(&all);
        if !errors.is_empty() {
            return err(format!("schedule_task: 依赖图校验失败: {:?}", errors));
        }
        for dep in &task.depends_on {
            if !all.iter().any(|t| &t.id == dep) {
                return err(format!(
                    "schedule_task: depends_on 引用不存在的任务 `{dep}`"
                ));
            }
        }

        match save(&task) {
            Ok(()) => ok(json!({
                "id": id,
                "status": "scheduled",
                "title": title,
                "note": "任务已写入 schedule store；由 daemon tick 或 `schedule tick --once` 到期触发，工具本身不注册 OS 调度器"
            })
            .to_string()),
            Err(e) => err(format!("schedule_task: 保存失败: {e}")),
        }
    }
}
