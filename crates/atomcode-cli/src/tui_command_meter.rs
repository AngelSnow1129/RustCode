//! Counting the commands a person runs on the new screen.
//!
//! Here and not in `atomcode-tui` for the reason `tui_login` and
//! `tui_onboarding` are here: the screen may know neither the host nor what it
//! reports to (`gates/layers.sh`, "UI 不认 Host,也不认 Product"). The screen
//! states that a command ran; this decides that the fact is worth counting and
//! owns the sink it is counted into.
//!
//! # What it is replacing
//!
//! `atomcode-tuix` has reported one `use_command` per dispatch since the event
//! existed (`src/event_loop/commands.rs:1615`, and `:4127` for a name nothing
//! answers to). The row-assembled screen reported none — it has no telemetry
//! dependency at all — so every command run on it was invisible. The moment
//! that screen becomes the default, a dashboard that has counted commands for
//! two years would simply stop.
//!
//! # Parity, deliberately
//!
//! `success` is `true` for anything that reached a command set, even if the
//! command then refused: tuix reports before it dispatches, precisely so a
//! command that errors still counts as run. A name nothing answers to is the
//! one failure shape, `NotFound`, with the same `error_data` keys tuix sends.
//! The richer thing — reporting whether the command itself succeeded — is
//! available here (`Outcome` says), and is deliberately not taken: it would
//! change what `success = false` has meant in every stored row.
//!
//! # Which session a command is counted against
//!
//! tuix rebinds `Telemetry.session_id` every time a session is opened, resumed
//! or switched (`event_loop/commands.rs::bind_telemetry_to_session`, five call
//! sites), so its `use_command` rows group per conversation. Emitting here
//! with no scope falls to the process launch id instead
//! (`runtime.rs::build_envelope`: scope → field → `launch_id`), which put every
//! command a person ran in one TUI process into a single bucket — so a panel
//! grouping by session could not tell two conversations apart after `/new` or
//! `/resume`.
//!
//! The screen is not asked to change: it states the fact, and this row reads the
//! session off `AgentClientSvc` — already registered, and already the screen's
//! own answer to "which conversation is on screen" (`plugin.rs::session`). The
//! same seam `LlmChat`/`ToolCall` take their attribution from, so all three now
//! group alike. A session that is not a uuid, or no screen at all, keeps the
//! old behaviour rather than losing the row.

use std::sync::Arc;

use async_trait::async_trait;
use atomcode_plexus::{Context, Plugin};
use serde_json::Value;

/// The row's name.
pub const ROW: &str = "tui-command-meter";

pub fn row_layer() -> String {
    format!("[[insert]]\nname = \"{ROW}\"\n")
}

/// Installs the observer. Mounted only when the launcher has a sink.
pub struct CommandMeterRow {
    pub telemetry: Arc<atomcode_telemetry::Telemetry>,
}

#[async_trait]
impl Plugin for CommandMeterRow {
    fn name(&self) -> &'static str {
        ROW
    }
    fn inject(&self) -> &'static [&'static str] {
        &["tui-commands"]
    }
    fn description(&self) -> &'static str {
        "counts each command a person runs into the host's telemetry"
    }
    async fn apply(&self, ctx: &Context, _config: &Value) -> Result<(), String> {
        let commands = ctx
            .require::<atomcode_tui::plugin::CommandsSvc>()
            .map_err(|e| e.to_string())?;
        // Optional, and deliberately so: both seams are provided by the same row
        // in the same `apply`, so in a mounted screen this is always `Some` — but
        // a tree without the screen should still count its commands under the
        // launch id rather than lose the row.
        let screen = ctx.service::<atomcode_tui::plugin::AgentClientSvc>();
        commands.observe(Arc::new(UseCommandMeter {
            telemetry: self.telemetry.clone(),
            screen,
        }));
        Ok(())
    }
}

struct UseCommandMeter {
    telemetry: Arc<atomcode_telemetry::Telemetry>,
    /// The screen, for the conversation a command was run in. `None` when there
    /// is no screen: the event is then emitted unscoped, as it was before.
    screen: Option<Arc<atomcode_tui::plugin::AgentClient>>,
}

impl atomcode_tui::command::CommandObserver for UseCommandMeter {
    fn ran(&self, run: &atomcode_tui::command::CommandRun<'_>) {
        // Read at dispatch, not at mount: `/new` and `/resume` both put another
        // conversation on screen without remounting anything, so a session read
        // once at startup would be stale for the rest of the process.
        let session = self
            .screen
            .as_ref()
            .and_then(|s| session_uuid(&s.session()));
        self.emit(use_command(run.name, run.found), session);
    }
}

impl UseCommandMeter {
    /// Emit one command's record, against `session` when there is one.
    ///
    /// Split from `ran` so the scoping is reachable without a staged screen:
    /// `AgentClient`'s own ways of putting a session on screen are crate-private
    /// to `atomcode-tui`, and a row must not grow a public setter to satisfy a
    /// test. The scope itself is the whole of what this row adds, so that is
    /// what the criterion drives.
    fn emit(&self, event: atomcode_telemetry::Event, session: Option<uuid::Uuid>) {
        match session {
            // No screen, or a session id that is not a uuid. The old behaviour:
            // no scope, so the envelope falls to the launch id — a row is not
            // worth losing to get a better id on it.
            None => self.telemetry.track(event),
            Some(session_id) => atomcode_telemetry::CurrentContext::scope_blocking(
                atomcode_telemetry::CurrentContext {
                    session_id: Some(session_id),
                    // The rest of the scope, not a fresh one: `CTX` is a
                    // task-local and `sync_scope` REPLACES it, so
                    // `..Default::default()` would drop everything the launch
                    // scope established. `mode` survives that by falling back to
                    // the launch default (`runtime.rs::build_envelope`), but
                    // `repo_origin` has no such fallback — it is read straight
                    // off the context, so a `use_command` row would carry
                    // `repo_origin: null` while every `LlmChat` around it carried
                    // the repository. Same shape as `main.rs`'s own scopes.
                    ..atomcode_telemetry::CurrentContext::current()
                },
                || self.telemetry.track(event),
            ),
        }
    }
}

/// A screen's session id, when it can be named as one.
///
/// A free function because `AgentClient`'s own ways of putting a session on
/// screen are crate-private, so a test outside `atomcode-tui` cannot stage one —
/// and what is worth pinning here is the mapping, not the plumbing that reaches
/// it. Empty (a client nothing has been put on screen yet) and non-uuid ids both
/// answer `None`, which is the old unscoped behaviour: a row is not worth losing
/// to get a better id on it.
fn session_uuid(on_screen: &str) -> Option<uuid::Uuid> {
    if on_screen.is_empty() {
        return None;
    }
    uuid::Uuid::parse_str(on_screen).ok()
}

/// The record a run becomes.
///
/// A free function so the mapping is testable without a mounted screen: what
/// is worth pinning is the shape, and the shape is all of this.
pub(crate) fn use_command(name: &str, found: bool) -> atomcode_telemetry::Event {
    match found {
        true => atomcode_telemetry::Event::UseCommand {
            type_: name.to_string(),
            success: Some(true),
            error_kind: None,
            error_data: None,
        },
        false => atomcode_telemetry::Event::UseCommand {
            type_: name.to_string(),
            success: Some(false),
            error_kind: Some(atomcode_telemetry::UseCommandErrorKind::NotFound),
            error_data: Some(
                serde_json::json!({
                    "command": name,
                    "duration_ms": 0,
                    "message": format!("Unknown command: {name}"),
                })
                .to_string(),
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The name a command is counted under, and the shape of the record.
    ///
    /// Pinned against `crates/atomcode-telemetry/tests/golden/wire/` — the
    /// same two shapes, `use_command` and `use_command_not_found`.
    #[test]
    fn a_command_that_ran_is_counted_as_a_success() {
        let atomcode_telemetry::Event::UseCommand {
            type_,
            success,
            error_kind,
            error_data,
        } = use_command("session", true)
        else {
            panic!("not a use_command");
        };
        assert_eq!(type_, "session");
        assert_eq!(success, Some(true));
        assert!(error_kind.is_none());
        assert!(error_data.is_none());
    }

    /// The whole path: a person types a line, and a record lands in the sink.
    ///
    /// The two halves are tested apart above and in
    /// `atomcode-tui/src/command.rs`; this is the one that fails if they are
    /// never joined — the mistake that left the row-assembled screen counting
    /// nothing while every piece of the machinery existed.
    #[tokio::test]
    async fn a_command_typed_on_the_screen_lands_in_the_sink() {
        use atomcode_tui::command::{Command, CommandSet, Commands, Outcome};

        struct Anything;
        #[async_trait]
        impl CommandSet for Anything {
            fn id(&self) -> &'static str {
                "test"
            }
            fn commands(&self) -> Vec<Command> {
                vec![Command::new("session", "fresh start").with_aliases(&["new"])]
            }
            async fn run(&self, _n: &str, _a: &str, _c: &Context) -> Outcome {
                Outcome::Quiet
            }
        }

        let (telemetry, captured) = atomcode_telemetry::Telemetry::in_memory("test".into());
        let commands = Commands::new();
        commands.add(Arc::new(Anything)).unwrap();
        commands.observe(Arc::new(UseCommandMeter {
            telemetry,
            screen: None,
        }));
        let app = atomcode_plexus::App::new(
            atomcode_plexus::PluginRegistry::new(),
            atomcode_plexus::ConfigTree::default(),
        );
        let ctx = app.context();

        commands.dispatch("/new", &ctx).await;
        commands.dispatch("/nope", &ctx).await;

        let mut counted = Vec::new();
        for _ in 0..200 {
            counted = captured
                .lock()
                .await
                .iter()
                .filter_map(|record| match &record.event {
                    atomcode_telemetry::Event::UseCommand { type_, success, .. } => {
                        Some((type_.clone(), *success))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            if counted.len() == 2 {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        assert_eq!(
            counted,
            vec![
                // The alias counted under the command it names.
                ("session".to_string(), Some(true)),
                ("nope".to_string(), Some(false)),
            ]
        );
    }

    /// A name nothing answers to is the one failure shape, with the detail
    /// blob tuix sends — a dashboard reads `command` and `message` out of it.
    #[test]
    fn a_name_nothing_answers_to_is_counted_as_not_found() {
        let atomcode_telemetry::Event::UseCommand {
            type_,
            success,
            error_kind,
            error_data,
        } = use_command("nope", false)
        else {
            panic!("not a use_command");
        };
        assert_eq!(type_, "nope");
        assert_eq!(success, Some(false));
        assert!(matches!(
            error_kind,
            Some(atomcode_telemetry::UseCommandErrorKind::NotFound)
        ));
        let detail: serde_json::Value =
            serde_json::from_str(&error_data.expect("a miss says what was typed")).unwrap();
        assert_eq!(detail["command"], "nope");
        assert_eq!(detail["duration_ms"], 0);
        assert_eq!(detail["message"], "Unknown command: nope");
    }

    /// A command is counted against the conversation it was run in.
    ///
    /// The defect this pins: emitted unscoped, every `use_command` in a TUI
    /// process fell to the process launch id, so a panel grouping by session
    /// could not tell two conversations apart after `/new` or `/resume` — while
    /// `LlmChat` and `ToolCall`, scoped off the same session, could. The grouping
    /// is the whole point of the event, so it is asserted on the envelope.
    ///
    /// **What this does not cover:** that a mounted screen's session reaches
    /// `emit` at all. `AgentClient::follow` — the only way to put a session on a
    /// client — is crate-private to `atomcode-tui`, so a test here cannot stage
    /// one, and a row must not grow a public setter to satisfy a test. What is
    /// covered is the half that was broken: given a session, the event carries
    /// it; given none, the event is still emitted. The read that supplies it is
    /// `AgentClient::session`, already public and already on the seam `apply`
    /// hands this row.
    #[tokio::test]
    async fn a_command_is_counted_against_the_session_it_ran_in() {
        let session = uuid::Uuid::new_v4();
        let other = uuid::Uuid::new_v4();
        let (telemetry, captured) = atomcode_telemetry::Telemetry::in_memory("test".into());
        let meter = UseCommandMeter {
            telemetry,
            screen: None,
        };

        // Two conversations in one process — the `/new` case. Before the fix both
        // rows carried the launch id and neither could be told from the other.
        meter.emit(use_command("session", true), Some(session));
        meter.emit(use_command("compact", true), Some(other));

        let mut ids = Vec::new();
        for _ in 0..200 {
            ids = captured
                .lock()
                .await
                .iter()
                .filter(|r| matches!(r.event, atomcode_telemetry::Event::UseCommand { .. }))
                .map(|r| r.envelope.session_id)
                .collect();
            if ids.len() == 2 {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        assert_eq!(
            ids,
            vec![session, other],
            "each command is counted against the conversation it ran in"
        );
    }

    /// With no session to name, the event is still counted — unscoped, exactly
    /// as it was before this row learned about sessions.
    ///
    /// The regression this guards is the tempting other shape: dropping the
    /// event when there is no session to name. That would silently lose every
    /// command run before a screen is up, and `in_memory` reports the launch id
    /// as the nil uuid, so "unscoped" is a value a criterion can read.
    #[tokio::test]
    async fn a_command_with_no_session_to_name_is_still_counted() {
        let (telemetry, captured) = atomcode_telemetry::Telemetry::in_memory("test".into());
        let meter = UseCommandMeter {
            telemetry,
            screen: None,
        };
        meter.emit(use_command("session", true), None);

        let mut ids = Vec::new();
        for _ in 0..200 {
            ids = captured
                .lock()
                .await
                .iter()
                .filter(|r| matches!(r.event, atomcode_telemetry::Event::UseCommand { .. }))
                .map(|r| r.envelope.session_id)
                .collect();
            if !ids.is_empty() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        assert_eq!(
            ids,
            vec![uuid::Uuid::nil()],
            "no session to name ⇒ the launch id, and the row is not lost"
        );
    }

    /// Adding a session to the scope must not cost the scope the launch built.
    ///
    /// `CTX` is a task-local and `sync_scope` replaces it wholesale, so a
    /// `..Default::default()` here would not "leave the rest alone" — it would
    /// blank everything the launch scope established. `mode` hides that, because
    /// `build_envelope` falls back to the launch default for it. `repo_origin`
    /// does not: it is read straight off the context
    /// (`runtime.rs::build_envelope`), so the row would carry
    /// `repo_origin: null` while every `LlmChat` around it named the repository.
    ///
    /// Compared as JSON because `RepoOrigin`/`RepoHost` are `Serialize` but not
    /// `PartialEq` — the wire shape is what a dashboard reads anyway.
    #[tokio::test]
    async fn a_command_keeps_the_scope_the_launch_established() {
        let session = uuid::Uuid::new_v4();
        let repo = atomcode_telemetry::event::RepoOrigin {
            host: atomcode_telemetry::event::RepoHost::Gitcode,
            has_git: true,
        };
        let (telemetry, captured) = atomcode_telemetry::Telemetry::in_memory("test".into());
        let meter = UseCommandMeter {
            telemetry,
            screen: None,
        };

        // The scope `main.rs` wraps a TUI launch in.
        atomcode_telemetry::CurrentContext::scope_blocking(
            atomcode_telemetry::CurrentContext {
                repo_origin: Some(repo.clone()),
                mode: Some(atomcode_telemetry::event::SessionMode::Tui),
                ..Default::default()
            },
            || meter.emit(use_command("session", true), Some(session)),
        );

        let want = serde_json::to_value(&repo).expect("a repo origin is serializable");
        let mut seen = Vec::new();
        for _ in 0..200 {
            seen = captured
                .lock()
                .await
                .iter()
                .filter(|r| matches!(r.event, atomcode_telemetry::Event::UseCommand { .. }))
                .map(|r| {
                    (
                        serde_json::to_value(&r.envelope.repo_origin).expect("serializable"),
                        r.envelope.session_id,
                    )
                })
                .collect();
            if !seen.is_empty() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        assert_eq!(
            seen,
            vec![(want, session)],
            "the session is added to the launch scope, not substituted for it"
        );
    }

    /// The mapping from what a screen says is on screen to what may be put in
    /// an envelope.
    ///
    /// Both refusals matter, and for the same reason: a row is not worth losing
    /// to get a better id on it. A client nothing has been put on screen yet
    /// reports an empty string, and a session id that is not a uuid cannot be
    /// correlated with anything — either way the event is still emitted, just
    /// unscoped, exactly as it was before.
    #[test]
    fn only_a_session_id_that_can_be_correlated_is_named() {
        let id = uuid::Uuid::new_v4();
        assert_eq!(session_uuid(&id.to_string()), Some(id));
        assert_eq!(session_uuid(""), None, "no session on screen yet");
        assert_eq!(session_uuid("not-a-uuid"), None);
        // The shape a team member's storage id has (`<lead>/<name>` with `/`
        // swapped for `~`) is not a uuid either, and must not be forced into one.
        assert_eq!(session_uuid("lead~scout"), None);
    }
}
