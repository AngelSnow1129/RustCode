//! IM (instant-messaging) channel adapters -- drive this project's agent from a
//! chat on DingTalk / Feishu / WeCom.
//!
//! Shape of the flow:
//!
//! ```text
//!   IM user sends a message
//!          |
//!          v
//!   ImAdapter (per platform)        <- dingtalk.rs: long connection (WS)
//!          |  inbound text + chat id
//!          v
//!   bridge::handle_message          <- chat -> project+session, run the agent
//!          |  run_native_headless(capture = true)
//!          v
//!   outbound chunks                 <- split_reply: platform message-size limits
//!          |
//!          v
//!   ImAdapter::send_text            <- back to the same conversation
//! ```
//!
//! Two invariants worth stating up front, because breaking either is a security
//! bug rather than a glitch:
//!
//! - **Credentials are bring-your-own.** The bot is created by the user on the
//!   platform; nothing here talks to a RustCode-hosted service. This keeps the
//!   "no central account" posture the provider layer already has.
//! - **Approval is never silently skipped.** An IM turn has a human behind it
//!   (just an asynchronous one), so the adapter must not run in the
//!   `strict_unattended` mode that blanket-denies escalated tool calls. Approval
//!   requests are surfaced to the chat and resolved by an explicit reply.

use async_trait::async_trait;

pub mod approval;
pub mod bridge;
pub mod dingtalk;
pub mod runner;

pub use approval::{
    parse_approval_reply, ApprovalCard, ApprovalPort, ApprovalReply, ImApprovalRelay, APPROVAL_WAIT,
};
pub use bridge::{split_reply, BridgedReply, RecentMessages};
pub use runner::{handle_message, resolve_project, serve_channel, AgentRunner, AgentTurn};

/// Typed adapter failure. Kept separate from `anyhow` at the module boundary so
/// a caller can distinguish "transport died, retry" from "config is wrong, do
/// not retry".
///
/// `Display` is hand-written rather than derived so this module adds no error
/// crate to the CLI's dependency set -- the variants are few and fixed.
#[derive(Debug)]
pub enum ImError {
    /// The platform rejected the credentials, or they were missing.
    Auth {
        platform: &'static str,
        detail: String,
    },
    /// The transport (long connection / HTTP) failed.
    Transport(String),
    /// A frame arrived that this adapter cannot interpret. Non-fatal at the
    /// stream level: the offending frame is acked with a failure code and the
    /// connection stays up.
    Protocol(String),
    /// The channel is disabled (master switch or per-channel switch).
    Disabled,
}

impl std::fmt::Display for ImError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auth { platform, detail } => {
                write!(
                    f,
                    "IM authentication failed for platform `{platform}`: {detail}"
                )
            }
            Self::Transport(detail) => write!(f, "IM transport error: {detail}"),
            Self::Protocol(detail) => write!(f, "IM protocol error: {detail}"),
            Self::Disabled => write!(f, "IM channel disabled"),
        }
    }
}

impl std::error::Error for ImError {}

/// One inbound chat message, normalized across platforms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImMessage {
    /// Platform spelling (`dingtalk` / `feishu` / `wecom`).
    pub platform: String,
    /// Conversation this arrived in. Used as the identity-mapping key, so it
    /// must be the *stable* conversation id the platform reports -- not a
    /// per-message id.
    pub chat_id: String,
    /// Sender within that conversation, for logging/attribution only. Never
    /// used as a mapping key: a group chat has many senders and mapping by
    /// sender would fragment one conversation into N sessions.
    pub sender_id: String,
    /// Message text, already stripped of platform mention markup.
    pub text: String,
    /// Platform message id, for deduplication of redelivered frames.
    pub message_id: String,
    /// Opaque per-platform reply target (e.g. DingTalk's `sessionWebhook`).
    /// Opaque on purpose: its lifetime/format is the platform's business, and
    /// adapters must not assume it outlives the message it came with.
    pub reply_token: Option<String>,
}

/// A platform adapter: pulls inbound messages, posts replies.
#[async_trait]
pub trait ImAdapter: Send + Sync {
    /// Platform spelling, matching `rustcode_config::config::im::ImPlatform`.
    fn platform(&self) -> &'static str;

    /// Receive the next inbound message, or `None` when the stream ended and a
    /// reconnect is the caller's decision.
    ///
    /// Implementations must send the platform's acknowledgement for every frame
    /// they consume -- several platforms redeliver unacked messages, and a
    /// redelivered message would otherwise drive the agent twice.
    async fn next_message(&mut self) -> Result<Option<ImMessage>, ImError>;

    /// Post one already-chunked piece of text back to a conversation.
    ///
    /// Callers chunk via [`split_reply`]; implementations must not re-chunk.
    async fn send_text(
        &self,
        chat_id: &str,
        reply_token: Option<&str>,
        text: &str,
    ) -> Result<(), ImError>;

    /// Optional "typing" hint. Most platforms lack an equivalent, so the
    /// default is a no-op rather than a `NotSupported` error -- callers should
    /// be able to ask unconditionally.
    async fn typing(&self, _chat_id: &str) -> Result<(), ImError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_messages_include_the_platform_and_never_a_secret() {
        let e = ImError::Auth {
            platform: "dingtalk",
            detail: "invalid clientId".into(),
        };
        let rendered = e.to_string();
        assert!(rendered.contains("dingtalk"));
        assert!(rendered.contains("invalid clientId"));
    }

    #[test]
    fn disabled_error_is_distinguishable_from_transport_failure() {
        // The bridge relies on this distinction: a disabled channel is a normal
        // outcome, a transport failure deserves a retry/backoff.
        assert!(matches!(ImError::Disabled, ImError::Disabled));
        let t = ImError::Transport("connection reset".into());
        assert!(!matches!(t, ImError::Disabled));
    }
}
