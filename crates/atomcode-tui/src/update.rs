//! Whether a newer build is out, for the status row.
//!
//! Finding out is a network question and the answer's wording depends on how
//! this binary was installed (a package manager upgrades it, not `/upgrade`),
//! and neither is this screen's business: it cannot reach the network or the
//! filesystem (`gates/tui-layers.sh`). So the launcher answers, in the words the
//! row should show, and the screen only draws them. No row, no hint.

/// The launcher's answer to "is there a newer build".
#[async_trait::async_trait]
pub trait UpdateCheck: Send + Sync + 'static {
    /// What the status row should say about a newer build, or `None` when
    /// there is none (or it could not be found out — this is a nudge, not a
    /// report, and silence is the right failure).
    async fn available(&self) -> Option<String>;
}
