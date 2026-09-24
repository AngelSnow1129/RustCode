//! Approval round-trip for IM turns (P-IM2).
//!
//! An IM turn has a human behind it -- just an asynchronous one -- so escalated
//! tool calls must be surfaced to the chat and answered before the agent
//! proceeds. The headless event loop answers approvals in-process (auto-approve
//! bash, deny the rest); this module gives it one optional escape hatch:
//!
//! ```text
//! kernel: AgentEvent::Request(kind="approval")
//!   -> headless Request arm (main.rs): not auto-approved?
//!      -> ApprovalPort::request_decision(card)      <- this module's trait
//!         -> adapter.send_text(card text)            <- chat sees the ask
//!         -> adapter.next_message() ... parse reply  <- chat answers
//!            (unparseable -> hint, keep waiting; timeout -> None)
//!      -> None => fail-closed Deny (same as today)
//! ```
//!
//! Fail-closed invariants kept from the rest of the approval stack:
//!
//! - `strict_unattended` never consults the port (scheduled runs are genuinely
//!   unattended -- there is nobody to ask);
//! - `skip_permissions` / bash auto-approval happens *before* the port is
//!   consulted, so `-p` and the bash-liveness behaviour are unchanged;
//! - a `None` return (timeout, transport trouble) maps to
//!   [`crate::im::ImError`]-free `ApprovalResponse::deny()` upstream.
//!
//! The decision tokens themselves (`y/yes/allow`, `n/no/deny`, `always`) are
//! the repo's canonical untranslated judgment tokens; only the surrounding
//! card/hint/timeout prose is localized.

use std::time::{Duration, Instant};

use async_trait::async_trait;
use rustcode_config::i18n::{t, Msg};

use super::ImAdapter;

/// How long one approval may stay unanswered before the port gives up and the
/// caller fail-closes to Deny.
///
/// Deliberately shorter than the kernel's default 300 s request budget: the IM
/// relay should time out (and tell the user) before the runtime does, so the
/// chat always learns the outcome from us rather than from a silent turn end.
pub const APPROVAL_WAIT: std::time::Duration = std::time::Duration::from_secs(120);

/// One escalated tool call, rendered for a human.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalCard {
    pub tool: String,
    /// Exact argument bytes that would execute (approve-what-runs contract).
    /// Truncated for chat display -- the full bytes live in the runtime, not
    /// in the chat bubble.
    pub args: String,
    /// Why the gate escalated (destructive / out-of-workspace / ...), if given.
    pub reason: Option<String>,
}

impl ApprovalCard {
    /// Build from the capabilities `ApprovalRequest`, truncating args for chat
    /// display. The full bytes stay authoritative in the runtime; the card is
    /// a readable summary, not the executable record.
    pub fn from_request(tool: String, args: &str, reason: Option<&str>) -> Self {
        Self {
            tool,
            args: truncate_chars(args, ARGS_DISPLAY_LIMIT),
            reason: reason.filter(|r| !r.trim().is_empty()).map(str::to_string),
        }
    }
}

/// Args preview cap, in characters. Long enough for a typical command, short
/// enough to stay well under every platform's message limit after the
/// surrounding prose.
const ARGS_DISPLAY_LIMIT: usize = 500;

fn truncate_chars(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_string();
    }
    let head: String = text.chars().take(limit).collect();
    format!("{head}…")
}

/// The human's decision, parsed from a chat reply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalReply {
    /// Allow this call once.
    Allow,
    /// Allow this tool for the rest of the turn set (remember=true upstream).
    AllowAlways,
    Deny,
}

impl ApprovalReply {
    /// Canonical lowercase token, for logs.
    pub fn as_token(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::AllowAlways => "allow_always",
            Self::Deny => "deny",
        }
    }
}

/// Parse one chat reply into a decision.
///
/// Accepts the repo's canonical judgment tokens (`y/yes/always/n`) and their
/// common Chinese equivalents; leading whitespace and a leading `/` are
/// tolerated. Returns `None` for anything else -- the caller then re-prompts
/// rather than guessing. Deliberately strict: a prose answer like "允许一下吧
/// 不过小心点" must NOT read as allow.
pub fn parse_approval_reply(raw: &str) -> Option<ApprovalReply> {
    let text = raw.trim().trim_start_matches('/').trim();
    const ALLOW: &[&str] = &["y", "yes", "allow", "ok", "允许", "同意"];
    const ALWAYS: &[&str] = &["always", "always_allow", "总是", "总是允许"];
    const DENY: &[&str] = &["n", "no", "deny", "拒绝", "不允许"];
    if ALLOW.contains(&text) {
        Some(ApprovalReply::Allow)
    } else if ALWAYS.contains(&text) {
        Some(ApprovalReply::AllowAlways)
    } else if DENY.contains(&text) {
        Some(ApprovalReply::Deny)
    } else {
        None
    }
}

/// The port the headless loop uses to ask a human.
///
/// `&mut self`: the IM implementation drives the adapter's `next_message`
/// while awaiting the reply, and the serve loop is blocked inside this turn
/// anyway -- sequential polling, no shared-borrow gymnastics.
#[async_trait]
pub trait ApprovalPort: Send {
    /// Surface one approval and wait for the answer.
    ///
    /// `None` = timed out (or the channel broke) -- the caller fail-closes to
    /// Deny. Implementations should have already told the user what happened.
    async fn request_decision(&mut self, card: ApprovalCard) -> Option<ApprovalReply>;
}

/// [`ApprovalPort`] implementation that relays to one IM chat.
///
/// v1 semantics, stated plainly: the serve loop is blocked inside this turn,
/// so the relay polls the adapter's inbound stream itself. Messages that
/// arrive during the wait are consumed: a parseable decision token answers the
/// ask, anything else gets the unparsed hint and is otherwise dropped (it is
/// acked to the platform, so no redelivery). Multi-channel queuing is future
/// work; the single-channel case -- the only one `rustcode im serve` runs --
/// is fully served.
pub struct ImApprovalRelay<'a> {
    adapter: &'a mut dyn ImAdapter,
    chat_id: &'a str,
    reply_token: Option<&'a str>,
    wait: Duration,
}

impl<'a> ImApprovalRelay<'a> {
    pub fn new(
        adapter: &'a mut dyn ImAdapter,
        chat_id: &'a str,
        reply_token: Option<&'a str>,
        wait: Duration,
    ) -> Self {
        Self {
            adapter,
            chat_id,
            reply_token,
            wait,
        }
    }

    /// Best-effort send: a failed prompt delivery must not escalate into turn
    /// failure; the deadline below still fail-closes the decision.
    async fn send(&mut self, text: &str) {
        let _ = self
            .adapter
            .send_text(self.chat_id, self.reply_token, text)
            .await;
    }
}

#[async_trait]
impl ApprovalPort for ImApprovalRelay<'_> {
    async fn request_decision(&mut self, card: ApprovalCard) -> Option<ApprovalReply> {
        self.send(&format_card(&card)).await;
        let deadline = Instant::now() + self.wait;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                self.send(&t(Msg::ImApprovalTimeout {
                    secs: self.wait.as_secs(),
                }))
                .await;
                return None;
            }
            match tokio::time::timeout(remaining, self.adapter.next_message()).await {
                Err(_elapsed) => {
                    self.send(&t(Msg::ImApprovalTimeout {
                        secs: self.wait.as_secs(),
                    }))
                    .await;
                    return None;
                }
                // Transport broke mid-wait: no further replies can arrive, so
                // fail closed (the caller maps None to Deny).
                Ok(Err(_error)) => return None,
                // Stream ended: same -- nobody can answer anymore.
                Ok(Ok(None)) => return None,
                Ok(Ok(Some(message))) => match parse_approval_reply(&message.text) {
                    Some(reply) => return Some(reply),
                    None => {
                        let hint = t(Msg::ImApprovalUnparsed).into_owned();
                        self.send(&hint).await;
                    }
                },
            }
        }
    }
}

/// Render one approval card as chat text.
///
/// The tool name and args are model-face values and pass through verbatim
/// (repo rule); only the surrounding prose is localized.
fn format_card(card: &ApprovalCard) -> String {
    let mut text = format!(
        "{}\n{}",
        t(Msg::ImApprovalAsk { tool: &card.tool }),
        card.args
    );
    if let Some(reason) = &card.reason {
        text.push('\n');
        text.push_str(&t(Msg::ImApprovalReason { reason }));
    }
    text.push('\n');
    text.push_str(&t(Msg::ImApprovalHint));
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_truncates_long_args_and_keeps_short_ones_verbatim() {
        let short = ApprovalCard::from_request("bash".into(), "ls -la", None);
        assert_eq!(short.args, "ls -la");
        assert_eq!(short.reason, None);

        let long = "x".repeat(ARGS_DISPLAY_LIMIT + 100);
        let card = ApprovalCard::from_request("bash".into(), &long, Some("destructive"));
        assert_eq!(card.args.chars().count(), ARGS_DISPLAY_LIMIT + 1); // + ellipsis
        assert!(card.args.ends_with('…'));
        assert_eq!(card.reason.as_deref(), Some("destructive"));
    }

    #[test]
    fn empty_reason_is_dropped_not_rendered() {
        let card = ApprovalCard::from_request("bash".into(), "ls", Some("   "));
        assert_eq!(card.reason, None);
    }

    #[test]
    fn canonical_judgment_tokens_parse() {
        assert_eq!(parse_approval_reply("y"), Some(ApprovalReply::Allow));
        assert_eq!(parse_approval_reply(" yes "), Some(ApprovalReply::Allow));
        assert_eq!(parse_approval_reply("允许"), Some(ApprovalReply::Allow));
        assert_eq!(
            parse_approval_reply("always"),
            Some(ApprovalReply::AllowAlways)
        );
        assert_eq!(
            parse_approval_reply("总是允许"),
            Some(ApprovalReply::AllowAlways)
        );
        assert_eq!(parse_approval_reply("n"), Some(ApprovalReply::Deny));
        assert_eq!(parse_approval_reply("拒绝"), Some(ApprovalReply::Deny));
        // A leading slash (habit from slash commands) is tolerated.
        assert_eq!(parse_approval_reply("/y"), Some(ApprovalReply::Allow));
    }

    #[test]
    fn prose_answers_do_not_parse_as_decisions() {
        // Strictness is the point: a sentence that merely CONTAINS a token must
        // not be read as a decision.
        assert_eq!(parse_approval_reply("允许一下吧 不过小心点"), None);
        assert_eq!(parse_approval_reply("why?"), None);
        assert_eq!(parse_approval_reply(""), None);
        assert_eq!(parse_approval_reply("   "), None);
        // Mixed-language lookalikes that are not exact tokens stay unparsed.
        assert_eq!(parse_approval_reply("yes!"), None);
    }

    #[test]
    fn reply_tokens_are_stable_for_logs() {
        assert_eq!(ApprovalReply::Allow.as_token(), "allow");
        assert_eq!(ApprovalReply::AllowAlways.as_token(), "allow_always");
        assert_eq!(ApprovalReply::Deny.as_token(), "deny");
    }
}
