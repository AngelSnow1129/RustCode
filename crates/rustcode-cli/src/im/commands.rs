//! Chat commands: control an IM conversation from the chat itself.
//!
//! Before this module the only way to steer an IM-driven session was to stop
//! the process and edit `config.toml` -- there was no way to ask "which project
//! am I driving?" or "start over" from the conversation. That is the difference
//! between a demo and something a team can actually live in.
//!
//! # Why only a small set
//!
//! A chat is a poor terminal: no completion, no key bindings, one line at a
//! time, and every command must be explainable in a single bubble. So this set
//! is deliberately tiny and split by *who owns the state*:
//!
//! - **Reads are answered immediately** by a worker task (it owns this chat's
//!   current session id at that moment): `/help`, `/status`, `/project`.
//! - **Writes are handed to the agent loop**, not applied directly. `/new`
//!   invalidates the binding, but the binding is also what the running loop
//!   reads and rewrites -- clearing it out from under an in-flight turn would
//!   race. So `/new` is marked [`ChatCommand::resets_session`] and the loop
//!   applies it at the next message boundary, in order.
//!
//! # Deliberately absent
//!
//! - `/stop` (cancel the in-flight turn): the only cancel path today is the
//!   runtime's own cancel command, reachable from a driver, and this adapter
//!   has no handle to the live runtime -- each turn builds and tears down its
//!   own runtime (`CliAgentRunner`). Wiring it would mean promoting the turn
//!   runtime into the dispatcher and rethinking shutdown; that is its own
//!   change, not a side effect of this one.
//! - `/cd` (rebind the chat to another project): moving a conversation between
//!   working directories is a permission-relevant act (it changes which files
//!   the agent can touch), and a chat message carries no proof of who sent it
//!   beyond a platform-supplied id. Left to `config.toml` for now.
//!
//! Both refusals are honest gaps, not oversights; see the IM section of
//! `AGENTS.md`.

use rustcode_config::i18n::{t, Msg};

use super::ImMessage;

/// A command recognized in an inbound message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatCommand {
    /// `/help` -- what can I type here.
    Help,
    /// `/status` -- which project/session this conversation is driving.
    Status,
    /// `/project` -- the working directory bound to this channel.
    Project,
    /// `/new` -- start a fresh session for the next message.
    New,
}

impl ChatCommand {
    /// Stable token for logs and tests (never localized).
    pub fn machine_token(&self) -> &'static str {
        match self {
            Self::Help => "help",
            Self::Status => "status",
            Self::Project => "project",
            Self::New => "new",
        }
    }

    /// Whether applying this command must happen inside the agent loop.
    ///
    /// `true` for commands that change conversation state: the loop owns the
    /// session binding, and applying such a change from the worker would race
    /// with an in-flight turn's own read/write of the same binding.
    pub fn resets_session(&self) -> bool {
        matches!(self, Self::New)
    }
}

/// Parse one message as a command.
///
/// Strict by design, mirroring `parse_approval_reply`: a leading `/`, then
/// exactly one known word, then nothing. `None` -- including for unknown
/// commands -- means "this is an ordinary prompt", so a message that merely
/// starts with a slash is still sent to the agent rather than being swallowed.
/// (The agent's own slash-command surface is not reachable from IM: those
/// commands are a TUI/CLI feature, and half-supporting them here would be a
/// trap.)
pub fn parse_chat_command(raw: &str) -> Option<ChatCommand> {
    let text = raw.trim();
    let rest = text.strip_prefix('/')?;
    let word = rest.trim();
    // Reject multi-word input: `/help me please` is a prompt, not a command.
    if word.is_empty() || word.split_whitespace().count() != 1 {
        return None;
    }
    match word.to_ascii_lowercase().as_str() {
        "help" => Some(ChatCommand::Help),
        "status" => Some(ChatCommand::Status),
        "project" => Some(ChatCommand::Project),
        "new" => Some(ChatCommand::New),
        _ => None,
    }
}

/// Everything needed to render a command answer.
pub struct CommandContext<'a> {
    /// Channel's bound working directory (this *is* the agent identity).
    pub project: &'a str,
    /// Session the chat currently continues, if any.
    pub session_id: Option<&'a str>,
}

/// Render the immediate answer for a read-only command.
///
/// Returns `None` for commands that mutate state -- those are applied by the
/// loop, which also owns the reply wording. Localized: the command tokens stay
/// ASCII, the prose around them does not.
pub fn answer_for(command: &ChatCommand, ctx: &CommandContext<'_>) -> Option<String> {
    match command {
        ChatCommand::Help => Some(t(Msg::ImCmdHelp).into_owned()),
        ChatCommand::Status => Some(
            t(Msg::ImCmdStatus {
                project: ctx.project,
                session: ctx.session_id.unwrap_or(""),
                new_session: ctx.session_id.is_none(),
            })
            .into_owned(),
        ),
        ChatCommand::Project => Some(
            t(Msg::ImCmdProject {
                project: ctx.project,
            })
            .into_owned(),
        ),
        ChatCommand::New => None,
    }
}

/// Message the loop sends back after it applied `/new`.
pub fn new_session_ack() -> String {
    t(Msg::ImCmdNewSession).into_owned()
}

/// Whether a message is a `/new` for this chat, i.e. the one command the loop
/// has to apply itself.
pub fn is_new_session(message: &ImMessage) -> bool {
    matches!(parse_chat_command(&message.text), Some(ChatCommand::New))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustcode_config::i18n::{set_locale, test_lock, Locale};

    #[test]
    fn only_exact_known_commands_parse() {
        assert_eq!(parse_chat_command("/help"), Some(ChatCommand::Help));
        assert_eq!(parse_chat_command("  /HELP  "), Some(ChatCommand::Help));
        assert_eq!(parse_chat_command("/status"), Some(ChatCommand::Status));
        assert_eq!(parse_chat_command("/project"), Some(ChatCommand::Project));
        assert_eq!(parse_chat_command("/new"), Some(ChatCommand::New));

        // Unknown slash commands are ordinary prompts, not swallowed.
        assert_eq!(parse_chat_command("/model gpt"), None);
        assert_eq!(parse_chat_command("/unknown"), None);
        // Multi-word input is a prompt even when the first word matches.
        assert_eq!(parse_chat_command("/help me now"), None);
        // No slash at all.
        assert_eq!(parse_chat_command("help"), None);
        assert_eq!(parse_chat_command(""), None);
        assert_eq!(parse_chat_command("/"), None);
    }

    #[test]
    fn only_new_resets_the_session() {
        assert!(ChatCommand::New.resets_session());
        for other in [ChatCommand::Help, ChatCommand::Status, ChatCommand::Project] {
            assert!(!other.resets_session(), "{other:?} must not reset");
        }
    }

    #[test]
    fn machine_tokens_are_stable_ascii() {
        assert_eq!(ChatCommand::New.machine_token(), "new");
        assert_eq!(ChatCommand::Help.machine_token(), "help");
    }

    #[test]
    fn read_only_answers_are_localized_and_carry_the_session() {
        let _guard = test_lock();
        set_locale(Locale::ZhCn);
        let ctx = CommandContext {
            project: "/work/demo",
            session_id: Some("sess-1"),
        };
        let status = answer_for(&ChatCommand::Status, &ctx).unwrap();
        assert!(status.contains("/work/demo"));
        assert!(status.contains("sess-1"));

        // No session yet: the answer must say so rather than rendering an
        // empty placeholder.
        let fresh = answer_for(
            &ChatCommand::Status,
            &CommandContext {
                project: "/work/demo",
                session_id: None,
            },
        )
        .unwrap();
        assert!(!fresh.contains("sess-1"));

        // `/new` is applied by the loop, so there is no immediate answer.
        assert!(answer_for(&ChatCommand::New, &ctx).is_none());

        set_locale(Locale::En);
        let en = answer_for(&ChatCommand::Project, &ctx).unwrap();
        assert!(en.contains("/work/demo"));
    }

    #[test]
    fn new_session_detection_matches_the_parser() {
        let message = |text: &str| ImMessage {
            platform: "dingtalk".into(),
            chat_id: "c".into(),
            sender_id: "u".into(),
            text: text.into(),
            message_id: "m".into(),
            reply_token: None,
        };
        assert!(is_new_session(&message("/new")));
        assert!(!is_new_session(&message("/status")));
    }
}
