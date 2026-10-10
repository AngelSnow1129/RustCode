// crates/rustcode-tuix/src/modals/schedule_editor.rs
//
// A full-field form modal for `/schedule add`. Collects the fields of a
// `ScheduleTask`, parses the schedule expression, validates the dependency
// graph, and only persists (via `rustcode_config::schedule::save`) when the
// graph is legal. Cancelling (Esc) is side-effect free; a failed validation
// or save keeps the modal open with the error shown (fail-closed).
//
// All persistence goes through the `rustcode_config::schedule` public API --
// this modal never execs the CLI binary and never writes the schedule file
// itself.

use anyhow::{anyhow, bail, Result};
use crossterm::event::{KeyCode, KeyModifiers};

use super::{
    backspace_at_cursor, delete_at_cursor, insert_at_cursor, next_grapheme_boundary,
    previous_grapheme_boundary, step_down, step_up, Modal, ModalAction,
};
use crate::event_loop::{build_status, Buffer, LoopCtx};
use crate::render::{MenuKind, MenuPayload, Renderer, UiLine};
use crate::state::UiState;
use rustcode_config::schedule::{Schedule, ScheduleTask};

/// Stable field indices into `ScheduleEditor::fields`. The order matches the
/// `ScheduleTask` construction in `submit`.
const IDX_ID: usize = 0;
const IDX_TITLE: usize = 1;
const IDX_PROMPT: usize = 2;
const IDX_CWD: usize = 3;
const IDX_SCHEDULE: usize = 4;
const IDX_DEPENDS_ON: usize = 5;
const IDX_TRIGGERS: usize = 6;

/// One editable line of the form. The cursor is a byte offset into `value`
/// (grapheme-aware helpers keep it on a char boundary).
struct Field {
    label: &'static str,
    value: String,
    cursor: usize,
}

pub struct ScheduleEditor {
    fields: Vec<Field>,
    focus: usize,
    error: Option<String>,
}

impl ScheduleEditor {
    /// Build an empty form. `cwd` is pre-filled from the current working
    /// directory so the user rarely has to retype it.
    pub fn open(cwd_default: std::path::PathBuf) -> Self {
        let fields = vec![
            Field {
                label: "id",
                value: String::new(),
                cursor: 0,
            },
            Field {
                label: "title",
                value: String::new(),
                cursor: 0,
            },
            Field {
                label: "prompt",
                value: String::new(),
                cursor: 0,
            },
            Field {
                label: "cwd",
                value: cwd_default.to_string_lossy().into_owned(),
                cursor: 0,
            },
            Field {
                label: "schedule",
                value: String::new(),
                cursor: 0,
            },
            Field {
                label: "depends_on",
                value: String::new(),
                cursor: 0,
            },
            Field {
                label: "triggers",
                value: String::new(),
                cursor: 0,
            },
        ];
        Self {
            fields,
            focus: 0,
            error: None,
        }
    }

    /// Build the `ScheduleTask` candidate, validate it against the existing
    /// graph, and persist only when the graph is legal. Any failure keeps the
    /// modal open (fail-closed): `self.error` carries the reason and the caller
    /// redraws. Success renders a confirmation line to scrollback and closes.
    fn submit(&mut self, renderer: &mut dyn Renderer) -> ModalAction {
        let id = self.fields[IDX_ID].value.trim().to_string();
        let title = self.fields[IDX_TITLE].value.trim().to_string();
        let prompt = self.fields[IDX_PROMPT].value.trim().to_string();
        let cwd = self.fields[IDX_CWD].value.trim().to_string();
        let schedule_str = self.fields[IDX_SCHEDULE].value.trim().to_string();
        let depends_on = split_csv(&self.fields[IDX_DEPENDS_ON].value);
        let triggers = split_csv(&self.fields[IDX_TRIGGERS].value);

        let schedule = match parse_schedule(&schedule_str) {
            Ok(s) => s,
            Err(e) => {
                self.error = Some(format!("schedule 解析失败: {e}"));
                return ModalAction::Continue;
            }
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let candidate = ScheduleTask {
            id: id.clone(),
            title,
            prompt,
            cwd,
            schedule,
            permission_mode: "plan".to_string(),
            notify: "important".to_string(),
            enabled: true,
            created_at: now,
            last_run_at: None,
            last_status: None,
            last_run_id: None,
            depends_on,
            triggers,
        };

        let mut tasks = rustcode_config::schedule::list();
        tasks.push(candidate.clone());
        let errs = rustcode_config::schedule::validate_graph(&tasks);
        if !errs.is_empty() {
            self.error = Some(
                errs.iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join("\n"),
            );
            return ModalAction::Continue;
        }

        match rustcode_config::schedule::save(&candidate) {
            Ok(()) => {
                renderer.render(UiLine::CommandOutput(format!("schedule: 已保存任务 {id}")));
                renderer.flush();
                ModalAction::Close
            }
            Err(e) => {
                self.error = Some(format!("保存失败: {e}"));
                ModalAction::Continue
            }
        }
    }
}

/// Split a comma-separated list into `Vec<String>`, dropping empty entries.
/// An empty/whitespace-only string yields an empty `Vec`, so the on-disk
/// `serde(default)` round-trips unchanged.
fn split_csv(s: &str) -> Vec<String> {
    s.split(',')
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

/// Parse a TUI schedule expression into a `Schedule`.
///
/// Recognised (case-insensitive keyword):
///   `daily HH:MM` · `weekly N@HH:MM` (N in 1..=7) · `interval Nm` / `every Nm`
///   (N > 0) · `hourly` · `cron <expr>`
/// Anything else (including an empty string) is rejected.
fn parse_schedule(s: &str) -> Result<Schedule> {
    let trimmed = s.trim();
    let mut parts = trimmed.split_whitespace();
    let Some(kind) = parts.next() else {
        bail!("无法识别的 schedule 表达式: {trimmed}");
    };
    match kind.to_ascii_lowercase().as_str() {
        "daily" => {
            let time = parts
                .next()
                .ok_or_else(|| anyhow!("daily 需要 HH:MM 时间"))?;
            if !time.contains(':') {
                bail!("daily 时间格式应为 HH:MM");
            }
            Ok(Schedule::Daily {
                time: time.to_string(),
            })
        }
        "weekly" => {
            let spec = parts.next().ok_or_else(|| anyhow!("weekly 需要 N@HH:MM"))?;
            let (n, time) = spec
                .split_once('@')
                .ok_or_else(|| anyhow!("weekly 格式应为 N@HH:MM"))?;
            let weekday: u8 = n
                .parse()
                .map_err(|_| anyhow!("weekly 星期 N 应为 1..=7 的整数"))?;
            if !(1..=7).contains(&weekday) {
                bail!("weekly 星期 N 应为 1..=7");
            }
            if !time.contains(':') {
                bail!("weekly 时间格式应为 HH:MM");
            }
            Ok(Schedule::Weekly {
                weekday,
                time: time.to_string(),
            })
        }
        "interval" | "every" => {
            let spec = parts
                .next()
                .ok_or_else(|| anyhow!("interval/every 需要 Nm（分钟数）"))?;
            let minutes = spec
                .strip_suffix('m')
                .ok_or_else(|| anyhow!("interval/every 格式应为 Nm"))?;
            let every_minutes: u32 = minutes
                .parse()
                .map_err(|_| anyhow!("interval/every 分钟数应为正整数"))?;
            if every_minutes == 0 {
                bail!("interval/every 分钟数必须大于 0");
            }
            Ok(Schedule::Interval { every_minutes })
        }
        "hourly" => Ok(Schedule::Hourly),
        "cron" => {
            let expr: Vec<&str> = parts.collect();
            if expr.is_empty() {
                bail!("cron 需要表达式");
            }
            Ok(Schedule::Cron {
                expr: expr.join(" "),
            })
        }
        _ => bail!("无法识别的 schedule 表达式: {trimmed}"),
    }
}

impl Modal for ScheduleEditor {
    fn handle_key(
        &mut self,
        code: KeyCode,
        mods: KeyModifiers,
        buf: &mut Buffer,
        state: &mut UiState,
        ctx: &mut LoopCtx,
        renderer: &mut dyn Renderer,
    ) -> Result<ModalAction> {
        match code {
            KeyCode::Esc => return Ok(ModalAction::Close),
            KeyCode::Up => {
                self.focus = step_up(self.focus, self.fields.len());
            }
            KeyCode::Down => {
                self.focus = step_down(self.focus, self.fields.len());
            }
            KeyCode::Enter => {
                if self.focus + 1 < self.fields.len() {
                    // Not the last field: advance focus (like Tab-lite).
                    self.focus = step_down(self.focus, self.fields.len());
                } else {
                    // Last field (or explicit submit key): attempt to persist.
                    let action = self.submit(renderer);
                    if matches!(action, ModalAction::Continue) {
                        // Validation/save failed: keep the modal open, redraw
                        // so the error line (in the menu) is visible.
                        self.draw(buf, state, ctx, renderer);
                    }
                    return Ok(action);
                }
            }
            KeyCode::Char('s') if mods.contains(KeyModifiers::CONTROL) => {
                let action = self.submit(renderer);
                if matches!(action, ModalAction::Continue) {
                    self.draw(buf, state, ctx, renderer);
                }
                return Ok(action);
            }
            KeyCode::Char(c) if !mods.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) => {
                let f = &mut self.fields[self.focus];
                insert_at_cursor(&mut f.value, &mut f.cursor, c.encode_utf8(&mut [0; 4]));
            }
            KeyCode::Backspace => {
                let f = &mut self.fields[self.focus];
                backspace_at_cursor(&mut f.value, &mut f.cursor);
            }
            KeyCode::Delete => {
                let f = &mut self.fields[self.focus];
                delete_at_cursor(&mut f.value, &mut f.cursor);
            }
            KeyCode::Left => {
                let f = &mut self.fields[self.focus];
                f.cursor = previous_grapheme_boundary(&f.value, f.cursor);
            }
            KeyCode::Right => {
                let f = &mut self.fields[self.focus];
                f.cursor = next_grapheme_boundary(&f.value, f.cursor);
            }
            KeyCode::Home => {
                let f = &mut self.fields[self.focus];
                f.cursor = 0;
            }
            KeyCode::End => {
                let f = &mut self.fields[self.focus];
                f.cursor = f.value.len();
            }
            _ => {}
        }
        self.draw(buf, state, ctx, renderer);
        Ok(ModalAction::Continue)
    }

    fn handle_paste(
        &mut self,
        text: &str,
        buf: &mut Buffer,
        state: &mut UiState,
        ctx: &mut LoopCtx,
        renderer: &mut dyn Renderer,
    ) -> Result<ModalAction> {
        // Drop control chars, then append into the focused field at its cursor.
        let clean: String = text.chars().filter(|c| !c.is_control()).collect();
        let f = &mut self.fields[self.focus];
        insert_at_cursor(&mut f.value, &mut f.cursor, &clean);
        self.draw(buf, state, ctx, renderer);
        Ok(ModalAction::Continue)
    }

    fn draw(&self, _buf: &Buffer, state: &UiState, ctx: &LoopCtx, renderer: &mut dyn Renderer) {
        // Header hint row + one row per field (focus marked by the renderer)
        // + an optional error row, all transient (the menu, not scrollback).
        let mut items: Vec<(String, String)> = vec![(
            "↑/↓ 切换字段 · Enter 下移/提交末字段 · Ctrl-S 保存 · Esc 取消".to_string(),
            String::new(),
        )];
        for f in &self.fields {
            items.push((format!("{}: {}", f.label, f.value), String::new()));
        }
        if let Some(error) = &self.error {
            items.push((format!("错误: {error}"), String::new()));
        }
        let selected = self.focus + 1;
        let focused = &self.fields[self.focus];
        renderer.render(UiLine::InputPrompt {
            buf: focused.value.clone(),
            cursor_byte: focused.cursor,
            menu: Some(MenuPayload {
                items,
                selected,
                kind: MenuKind::Action,
            }),
            status: build_status(state, ctx),
            attachments: Vec::new(),
        });
        renderer.flush();
    }
}

#[cfg(test)]
mod tests {
    // `ScheduleEditor`'s `submit` is the only place the TUI persists a schedule:
    // it parses the schedule string, validates the candidate against the
    // existing dependency graph, and ONLY calls `save` when the graph is legal
    // (fail-closed). `submit` takes only `&mut dyn Renderer` (it does not read
    // runtime context) and does not call `draw`, so we call it directly rather
    // than routing through `handle_key` (which would require a fully valid
    // `Buffer`/`UiState` and `draw`/`build_status`).
    use super::*;
    use rustcode_config::schedule::{load, save, schedules_root};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    // ── Renderer stub ────────────────────────────────────────────────────────
    // Implements only the `Renderer` methods that have no default body, recording
    // every emitted `UiLine` so the success path can be asserted.
    #[derive(Default)]
    struct StubRenderer {
        lines: Mutex<Vec<UiLine>>,
    }

    impl Renderer for StubRenderer {
        fn render(&mut self, line: UiLine) {
            self.lines.lock().unwrap().push(line);
        }
        fn flush(&mut self) {}
        fn shutdown(&mut self) {}
        fn reset(&mut self) {}
        fn clear_screen(&mut self) {}
        fn suspend_for_external(&mut self) {}
        fn resume_from_external(&mut self) {}
        fn flush_deferred(&mut self) {}
    }

    // ── FS isolation ─────────────────────────────────────────────────────────
    // All tests that touch `RUSTCODE_HOME` (and therefore the real schedules
    // directory) must serialise on this lock, because the env var is process
    // global. Each test saves the previous value, points `RUSTCODE_HOME` at a
    // unique temp dir, runs, restores/removes, and deletes the temp dir.
    static TEST_LOCK: Mutex<()> = Mutex::new(());
    static TEST_DIR_SEQ: AtomicUsize = AtomicUsize::new(0);

    fn with_temp_home<F: FnOnce()>(f: F) {
        let _guard = TEST_LOCK.lock().unwrap();
        let original = std::env::var("RUSTCODE_HOME").ok();
        let dir = std::env::temp_dir().join(format!(
            "rustcode-tui-sched-test-{}-{}",
            std::process::id(),
            TEST_DIR_SEQ.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::env::set_var("RUSTCODE_HOME", &dir);
        f();
        std::env::remove_var("RUSTCODE_HOME");
        if let Some(v) = original {
            std::env::set_var("RUSTCODE_HOME", v);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn fixture_task(id: &str) -> ScheduleTask {
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
            depends_on: Vec::new(),
            triggers: Vec::new(),
        }
    }

    // ── parse_schedule: pure-function unit tests ─────────────────────────────
    #[test]
    fn parse_schedule_daily_variants() {
        assert!(
            matches!(parse_schedule("daily 09:00"), Ok(Schedule::Daily { time }) if time == "09:00"),
            "daily 09:00"
        );
        assert!(
            matches!(parse_schedule("daily 9:00"), Ok(Schedule::Daily { time }) if time == "9:00"),
            "daily 9:00"
        );
        assert!(
            parse_schedule("daily").is_err(),
            "daily (no time) must fail"
        );
        assert!(
            parse_schedule("daily 0900").is_err(),
            "daily 0900 (no colon) must fail"
        );
    }

    #[test]
    fn parse_schedule_weekly_variants() {
        assert!(
            matches!(parse_schedule("weekly 1@09:00"), Ok(Schedule::Weekly { weekday, time }) if weekday == 1 && time == "09:00"),
            "weekly 1@09:00"
        );
        assert!(
            matches!(parse_schedule("weekly 7@23:59"), Ok(Schedule::Weekly { weekday, time }) if weekday == 7 && time == "23:59"),
            "weekly 7@23:59"
        );
        assert!(
            parse_schedule("weekly 0@09:00").is_err(),
            "weekday 0 out of range"
        );
        assert!(
            parse_schedule("weekly 8@09:00").is_err(),
            "weekday 8 out of range"
        );
        assert!(
            parse_schedule("weekly 1@0900").is_err(),
            "weekly missing colon"
        );
        assert!(
            parse_schedule("weekly x@09:00").is_err(),
            "weekday not integer"
        );
        assert!(parse_schedule("weekly 1").is_err(), "weekly missing @");
    }

    #[test]
    fn parse_schedule_interval_variants() {
        assert!(
            matches!(parse_schedule("interval 30m"), Ok(Schedule::Interval { every_minutes }) if every_minutes == 30),
            "interval 30m"
        );
        assert!(
            matches!(parse_schedule("every 15m"), Ok(Schedule::Interval { every_minutes }) if every_minutes == 15),
            "every 15m"
        );
        assert!(
            parse_schedule("interval 0m").is_err(),
            "interval 0m must fail"
        );
        assert!(
            parse_schedule("interval 30").is_err(),
            "interval 30 (no m) must fail"
        );
        assert!(
            parse_schedule("interval xm").is_err(),
            "interval xm must fail"
        );
    }

    #[test]
    fn parse_schedule_hourly_and_cron() {
        assert!(
            matches!(parse_schedule("hourly"), Ok(Schedule::Hourly)),
            "hourly"
        );
        assert!(
            matches!(parse_schedule("cron 0 9 * * 1-5"), Ok(Schedule::Cron { expr }) if expr == "0 9 * * 1-5"),
            "cron 0 9 * * 1-5"
        );
        assert!(
            matches!(parse_schedule("cron 0 9 * * *"), Ok(Schedule::Cron { expr }) if expr == "0 9 * * *"),
            "cron 0 9 * * *"
        );
        assert!(parse_schedule("cron").is_err(), "cron (no expr) must fail");
    }

    #[test]
    fn parse_schedule_empty_and_unknown() {
        assert!(parse_schedule("").is_err(), "empty string must fail");
        assert!(parse_schedule("   ").is_err(), "whitespace-only must fail");
        assert!(parse_schedule("foo").is_err(), "unknown keyword must fail");
    }

    #[test]
    fn parse_schedule_case_insensitive() {
        assert!(
            matches!(parse_schedule("DAILY 09:00"), Ok(Schedule::Daily { time }) if time == "09:00"),
            "DAILY 09:00"
        );
        assert!(
            matches!(parse_schedule("Hourly"), Ok(Schedule::Hourly)),
            "Hourly"
        );
        assert!(
            matches!(parse_schedule("CRON 0 9 * * 1-5"), Ok(Schedule::Cron { expr }) if expr == "0 9 * * 1-5"),
            "CRON 0 9 * * 1-5"
        );
    }

    // ── split_csv: pure-function unit tests ──────────────────────────────────
    #[test]
    fn split_csv_behavior() {
        assert_eq!(split_csv(""), Vec::<String>::new());
        assert_eq!(split_csv("a,b,c"), vec!["a", "b", "c"]);
        assert_eq!(split_csv(" a , b "), vec!["a", "b"]);
        assert_eq!(split_csv(",, "), Vec::<String>::new());
    }

    // ── submit: fail-closed (no persistence on graph error) ──────────────────
    #[test]
    fn submit_self_dependency_is_fail_closed() {
        with_temp_home(|| {
            let mut editor = ScheduleEditor::open(PathBuf::from("/tmp"));
            editor.fields[IDX_ID].value = "t1".into();
            editor.fields[IDX_SCHEDULE].value = "daily 09:00".into();
            editor.fields[IDX_DEPENDS_ON].value = "t1".into();
            let mut renderer = StubRenderer::default();
            let action = editor.submit(&mut renderer);
            assert_eq!(
                action,
                ModalAction::Continue,
                "self-dep must keep modal open"
            );
            let err = editor.error.expect("self-dep must record an error");
            assert!(
                err.contains("depends on itself"),
                "error should mention self-dependency, got: {err}"
            );
            assert!(
                !schedules_root().join("t1.json").exists(),
                "must NOT persist when the graph is illegal"
            );
        });
    }

    #[test]
    fn submit_unknown_dependency_is_fail_closed() {
        with_temp_home(|| {
            let mut editor = ScheduleEditor::open(PathBuf::from("/tmp"));
            editor.fields[IDX_ID].value = "t1".into();
            editor.fields[IDX_SCHEDULE].value = "daily 09:00".into();
            editor.fields[IDX_DEPENDS_ON].value = "ghost".into();
            let mut renderer = StubRenderer::default();
            let action = editor.submit(&mut renderer);
            assert_eq!(
                action,
                ModalAction::Continue,
                "unknown-dep must keep modal open"
            );
            let err = editor.error.expect("unknown-dep must record an error");
            assert!(
                err.contains("does not exist"),
                "error should mention missing dependency, got: {err}"
            );
            assert!(
                !schedules_root().join("t1.json").exists(),
                "must NOT persist when the graph is illegal"
            );
        });
    }

    // ── submit: success path (persist + report) ──────────────────────────────
    #[test]
    fn submit_success_persists_and_reports() {
        with_temp_home(|| {
            // Pre-create the dependency tasks so the candidate's
            // `depends_on = ["a", "b"]` resolves and the graph is legal.
            save(&fixture_task("a")).unwrap();
            save(&fixture_task("b")).unwrap();

            let mut editor = ScheduleEditor::open(PathBuf::from("/tmp"));
            editor.fields[IDX_ID].value = "ok1".into();
            editor.fields[IDX_TITLE].value = "title1".into();
            editor.fields[IDX_PROMPT].value = "prompt1".into();
            editor.fields[IDX_SCHEDULE].value = "daily 08:00".into();
            editor.fields[IDX_DEPENDS_ON].value = "a,b".into();
            editor.fields[IDX_TRIGGERS].value = "ev1".into();

            let mut renderer = StubRenderer::default();
            let action = editor.submit(&mut renderer);
            assert_eq!(
                action,
                ModalAction::Close,
                "legal graph must close the modal"
            );

            // The success line is rendered into scrollback and names the task.
            let lines = renderer.lines.lock().unwrap().clone();
            let rendered_ok = lines
                .iter()
                .any(|l| matches!(l, UiLine::CommandOutput(s) if s.contains("ok1")));
            assert!(
                rendered_ok,
                "expected a CommandOutput naming ok1, got: {lines:?}"
            );

            // Persisted to disk and round-trips through the public `load` API.
            let path = schedules_root().join("ok1.json");
            assert!(path.exists(), "task file must be written");
            let saved = load("ok1").expect("saved task must load");
            assert_eq!(saved.depends_on, vec!["a".to_string(), "b".to_string()]);
            assert_eq!(saved.triggers, vec!["ev1".to_string()]);
            assert!(matches!(saved.schedule, Schedule::Daily { .. }));
        });
    }
}
