//! One STRUCTURED error taxonomy for every LLM (provider) call.
//!
//! The adapters currently build a flat [`ProviderError`] inline at each failure
//! site, which is correct but forces every consumer to string-match `message`
//! (or re-derive a class from `http_status`). This module adds the typed layer
//! on top: [`LlmError`] names the class, decides retryability in ONE place, and
//! converts back to a [`ProviderError`] without losing the load-bearing fields
//! (`http_status`, `retry_after_secs`, `code`).
//!
//! Scope note: this is the TAXONOMY, not a rewrite. The existing adapter error
//! construction (including the shared [`super::friendly_http_error`] wording --
//! the 401/402 headlines, the managed-plan 403 hint, and the literal `HTTP 429: `
//! prefix the kernel rate-limit path strips) is load-bearing and stays exactly
//! as-is.
//! New code converts THROUGH [`LlmError`]; the existing sites are migrated
//! incrementally.

use rustcode_kernel::stream::ProviderError;
use std::time::Duration;

/// A classified LLM failure. Variants are chosen so a caller can branch on the
/// CLASS (auth vs. rate-limit vs. bad request vs. upstream vs. transport) and on
/// [`LlmError::retryable`] instead of parsing prose.
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    /// The credential was rejected or lacks entitlement (HTTP 401/403).
    #[error("authentication rejected (HTTP {status}): {detail}")]
    Auth { status: u16, detail: String },
    /// The endpoint is rate limiting us (HTTP 429). `retry_after_secs` comes from
    /// the response's real `Retry-After` header when the server sent one.
    #[error("rate limited (HTTP 429); retry after {retry_after_secs:?}s")]
    RateLimited { retry_after_secs: Option<u64> },
    /// We sent something the endpoint refuses -- malformed arguments, unknown
    /// model, context overflow (HTTP 400/404/422).
    #[error("invalid request (HTTP {status}): {detail}")]
    InvalidRequest { status: u16, detail: String },
    /// The endpoint itself failed (5xx, or any other non-2xx status).
    #[error("upstream error (HTTP {status}): {detail}")]
    Upstream { status: u16, detail: String },
    /// No HTTP status at all: connect/DNS/TLS/reset/timeout.
    #[error("transport failure: {0}")]
    Transport(String),
    /// An established response went quiet for longer than the idle budget.
    #[error("stream idle for {0:?}")]
    IdleTimeout(Duration),
    /// A 2xx body we could not interpret (bad JSON, missing `choices`, ...).
    #[error("response decode failure: {0}")]
    Decode(String),
    /// The provider was built wrong (invalid proxy URL, missing base URL, ...).
    #[error("provider misconfiguration: {0}")]
    Config(String),
}

impl LlmError {
    /// Whether retrying the SAME request could plausibly succeed.
    ///
    /// Deliberately conservative for `Decode`: a body this request produced as
    /// unparseable will be unparseable again, and re-sending risks duplicating
    /// an already-executed side effect. `Auth` / `InvalidRequest` / `Config` are
    /// terminal for the same reason -- nothing about a retry changes them.
    pub fn retryable(&self) -> bool {
        match self {
            // A rejected credential never becomes valid by asking again; the user
            // must fix it (or re-`/login`).
            LlmError::Auth { .. } | LlmError::InvalidRequest { .. } | LlmError::Config(_) => false,
            LlmError::RateLimited { .. } => true,
            LlmError::Upstream { status, .. } => *status >= 500,
            LlmError::Transport(_) | LlmError::IdleTimeout(_) => true,
            LlmError::Decode(_) => false,
        }
    }

    /// Classify an existing [`ProviderError`]. `status` overrides the error's own
    /// `http_status` when the call site knows better (e.g. a status captured
    /// before the body was consumed); pass `None` to use the error's own.
    ///
    /// `None` + `retryable` ⇒ [`LlmError::Transport`] (a transient network class);
    /// `None` + terminal ⇒ [`LlmError::Decode`] (a response we could not make
    /// sense of -- the usual shape of a mid-stream failure, which carries no HTTP
    /// status because the status arrived with the headers long before).
    pub fn from_provider(e: &ProviderError, status: Option<u16>) -> Self {
        let detail = e.message.clone();
        match status.or(e.http_status) {
            Some(401) | Some(403) => LlmError::Auth {
                status: status.or(e.http_status).unwrap_or(401),
                detail,
            },
            Some(429) => LlmError::RateLimited {
                retry_after_secs: e.retry_after_secs,
            },
            // 404/422 are request-shaped (unknown model / unprocessable args);
            // 400 covers context overflow and bad arguments.
            Some(400) | Some(404) | Some(422) => LlmError::InvalidRequest {
                status: status.or(e.http_status).unwrap_or(400),
                detail,
            },
            Some(other) => LlmError::Upstream {
                status: other,
                detail,
            },
            None if e.retryable => LlmError::Transport(detail),
            None => LlmError::Decode(detail),
        }
    }
}

impl From<LlmError> for ProviderError {
    /// Back to the kernel's flat error, preserving the fields that are actually
    /// load-bearing downstream: `http_status` (branching + display),
    /// `retry_after_secs` (the authoritative rate-limit countdown), and `code`
    /// (structured branching, e.g. `context_length_exceeded`).
    ///
    /// The 429 message keeps the literal `HTTP 429: ` prefix -- the kernel's
    /// rate-limit recovery strips exactly that prefix to recover the server's
    /// own wording, so it must not be reworded here.
    fn from(e: LlmError) -> Self {
        let retryable = e.retryable();
        let (http_status, retry_after_secs, code, message) = match &e {
            LlmError::Auth { status, detail } => (
                Some(*status),
                None,
                Some("authentication_failed".to_string()),
                detail.clone(),
            ),
            LlmError::RateLimited { retry_after_secs } => {
                let mut message = "HTTP 429: rate limited".to_string();
                if let Some(secs) = retry_after_secs {
                    message = format!("{message} (retry after {secs}s)");
                }
                (
                    Some(429),
                    *retry_after_secs,
                    Some("rate_limited".to_string()),
                    message,
                )
            }
            LlmError::InvalidRequest { status, detail } => (
                Some(*status),
                None,
                Some("invalid_request".to_string()),
                detail.clone(),
            ),
            LlmError::Upstream { status, detail } => (
                Some(*status),
                None,
                None,
                format!("HTTP {status}: {detail}"),
            ),
            LlmError::Transport(message) => (
                None,
                None,
                Some("transport_failure".to_string()),
                message.clone(),
            ),
            LlmError::IdleTimeout(_) => {
                (None, None, Some("idle_timeout".to_string()), e.to_string())
            }
            LlmError::Decode(message) => (
                None,
                None,
                Some("response_decode_failure".to_string()),
                message.clone(),
            ),
            LlmError::Config(message) => (
                None,
                None,
                Some("provider_misconfiguration".to_string()),
                message.clone(),
            ),
        };
        ProviderError {
            retryable,
            message,
            http_status,
            code,
            retry_after_secs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(retryable: bool, status: Option<u16>, msg: &str) -> ProviderError {
        ProviderError {
            retryable,
            message: msg.into(),
            http_status: status,
            code: None,
            retry_after_secs: None,
        }
    }

    #[test]
    fn classifies_http_statuses() {
        assert!(matches!(
            LlmError::from_provider(&flat(false, Some(401), "nope"), None),
            LlmError::Auth { status: 401, .. }
        ));
        assert!(matches!(
            LlmError::from_provider(&flat(false, Some(403), "nope"), None),
            LlmError::Auth { status: 403, .. }
        ));
        assert!(matches!(
            LlmError::from_provider(&flat(true, Some(429), "slow down"), None),
            LlmError::RateLimited { .. }
        ));
        for code in [400u16, 404, 422] {
            assert!(
                matches!(
                    LlmError::from_provider(&flat(false, Some(code), "bad"), None),
                    LlmError::InvalidRequest { .. }
                ),
                "{code} is request-shaped"
            );
        }
        assert!(matches!(
            LlmError::from_provider(&flat(true, Some(503), "down"), None),
            LlmError::Upstream { status: 503, .. }
        ));
    }

    #[test]
    fn classifies_statusless_errors_by_retryability() {
        assert!(matches!(
            LlmError::from_provider(&flat(true, None, "connection reset"), None),
            LlmError::Transport(_)
        ));
        assert!(matches!(
            LlmError::from_provider(&flat(false, None, "bad chunk"), None),
            LlmError::Decode(_)
        ));
    }

    #[test]
    fn explicit_status_overrides_the_error_status() {
        // The call site saw a 429 before the body was consumed; the flat error
        // carries none. The explicit status must win.
        assert!(matches!(
            LlmError::from_provider(&flat(true, Some(500), "x"), Some(429)),
            LlmError::RateLimited { .. }
        ));
    }

    #[test]
    fn retryability_matches_the_class() {
        assert!(!LlmError::Auth {
            status: 401,
            detail: "x".into()
        }
        .retryable());
        assert!(!LlmError::InvalidRequest {
            status: 400,
            detail: "x".into()
        }
        .retryable());
        assert!(!LlmError::Decode("x".into()).retryable());
        assert!(!LlmError::Config("x".into()).retryable());
        assert!(!LlmError::Upstream {
            status: 429,
            detail: "x".into()
        }
        .retryable());
        assert!(LlmError::Upstream {
            status: 500,
            detail: "x".into()
        }
        .retryable());
        assert!(LlmError::RateLimited {
            retry_after_secs: Some(2)
        }
        .retryable());
        assert!(LlmError::Transport("x".into()).retryable());
        assert!(LlmError::IdleTimeout(Duration::from_secs(1)).retryable());
    }

    #[test]
    fn conversion_preserves_status_retry_after_and_429_prefix() {
        let e: ProviderError = LlmError::RateLimited {
            retry_after_secs: Some(7),
        }
        .into();
        assert!(e.retryable);
        assert_eq!(e.http_status, Some(429));
        assert_eq!(e.retry_after_secs, Some(7));
        assert!(
            e.message.starts_with("HTTP 429: "),
            "kernel rate-limit recovery strips this literal prefix: {}",
            e.message
        );

        let auth: ProviderError = LlmError::Auth {
            status: 401,
            detail: "bad key".into(),
        }
        .into();
        assert!(!auth.retryable);
        assert_eq!(auth.http_status, Some(401));
        assert_eq!(auth.message, "bad key");

        let up: ProviderError = LlmError::Upstream {
            status: 502,
            detail: "bad gateway".into(),
        }
        .into();
        assert!(up.retryable);
        assert_eq!(up.message, "HTTP 502: bad gateway");
    }
}
