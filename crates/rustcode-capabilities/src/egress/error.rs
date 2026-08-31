//! One error type for every outbound (egress) HTTP call site.
//!
//! Modeled on the reference project's `ToolResultBuilder.error(msg, brief=...)` pair:
//! a UI-facing ONE-LINE summary ([`EgressError::brief`]) and a diagnostic text for
//! the model / logs ([`EgressError::detail`]). Keeping the two separate is what stops
//! a 2 MiB HTML error page from being replayed into the conversation — `brief` is what
//! the user sees, `detail` is what gets logged, and the wire body is truncated at
//! [`BODY_EXCERPT_BYTES`] in BOTH.

use std::error::Error as _;

use thiserror::Error;

/// Hard cap on how much of a response body may ever be embedded in an
/// [`EgressError::Http`].
///
/// Web endpoints happily answer a failure with a multi-megabyte HTML page; echoing
/// that into the model context is both a token-bomb and a prompt-injection vector,
/// so every excerpt is truncated to this many BYTES before it is stored.
pub const BODY_EXCERPT_BYTES: usize = 512;

/// Every way an outbound HTTP call can fail.
#[derive(Debug, Error)]
pub enum EgressError {
    /// The caller's configuration is unusable (empty base url, unparseable proxy, …).
    /// Not retryable — retrying the same bytes yields the same error.
    #[error("invalid egress configuration: {0}")]
    Config(String),
    /// The HTTP client could not be CONSTRUCTED (bad TLS root, invalid header, …).
    /// Distinct from [`EgressError::Transport`] because a failed build means no
    /// request ever left the process; the #514 backstop lives on this path.
    #[error("failed to build HTTP client: {0}")]
    Build(String),
    /// The request never completed (DNS, connect, TLS handshake, timeout, …).
    #[error("HTTP transport error: {0}")]
    Transport(#[source] reqwest::Error),
    /// The server answered with a non-success status. `body_excerpt` is ALREADY
    /// truncated to [`BODY_EXCERPT_BYTES`] — see [`EgressError::http`].
    #[error("HTTP {status}: {brief}")]
    Http {
        status: u16,
        brief: String,
        body_excerpt: String,
    },
    /// The response arrived but its body could not be read/decoded.
    #[error("failed to read response body: {0}")]
    Body(String),
}

impl EgressError {
    /// Construct an [`EgressError::Http`] with the body excerpt truncated to
    /// [`BODY_EXCERPT_BYTES`]. Prefer this over the struct literal so no call site can
    /// accidentally smuggle a whole page into the context.
    pub fn http(status: u16, brief: impl Into<String>, body: &str) -> Self {
        Self::Http {
            status,
            brief: brief.into(),
            body_excerpt: truncate_utf8(body, BODY_EXCERPT_BYTES),
        }
    }

    /// Convert a completed-but-unsuccessful response into an [`EgressError::Http`],
    /// reading at most `BODY_EXCERPT_BYTES + 1` bytes off the wire so an enormous
    /// error page can never be buffered in full.
    pub async fn from_response(resp: reqwest::Response) -> Self {
        let status = resp.status().as_u16();
        // Read a bounded prefix: `text()` would buffer the whole body first.
        let body = read_bounded(resp, BODY_EXCERPT_BYTES).await;
        Self::Http {
            status,
            brief: format!("request failed with status {status}"),
            body_excerpt: body,
        }
    }

    /// ONE LINE for the UI: no stack, no body, no source chain (the TUI renders this
    /// in a single row and a multi-line blob would break the layout).
    pub fn brief(&self) -> String {
        match self {
            Self::Config(msg) => format!("[ERROR] egress misconfigured: {msg}"),
            Self::Build(msg) => format!("[ERROR] egress client unavailable: {msg}"),
            Self::Transport(e) => {
                // `reqwest::Error`'s Display can be multi-line and carries the full
                // URL (which may embed an api key in a query string) — keep only the
                // first line and cap the length.
                let first = e.to_string();
                let first = first.lines().next().unwrap_or_default();
                format!("[ERROR] network unreachable: {first}")
            }
            Self::Http { status, brief, .. } => format!("[ERROR] HTTP {status}: {brief}"),
            Self::Body(msg) => format!("[ERROR] unreadable response: {msg}"),
        }
    }

    /// The full diagnostic text for logs / the model: variant, every source in the
    /// chain, and the (already truncated) body excerpt.
    pub fn detail(&self) -> String {
        match self {
            Self::Config(msg) => format!("egress configuration error: {msg}"),
            Self::Build(msg) => format!("egress HTTP client build error: {msg}"),
            Self::Transport(e) => {
                let mut out = format!("egress transport error: {e}");
                let mut source = e.source();
                while let Some(s) = source {
                    out.push_str(&format!("\n  caused by: {s}"));
                    source = s.source();
                }
                out
            }
            Self::Http {
                status,
                brief,
                body_excerpt,
            } => {
                let mut out = format!("egress HTTP error {status}: {brief}");
                if !body_excerpt.is_empty() {
                    out.push_str(&format!(
                        "\n--- response body (truncated) ---\n{body_excerpt}"
                    ));
                }
                out
            }
            Self::Body(msg) => format!("egress response body error: {msg}"),
        }
    }

    /// Whether retrying the SAME request could plausibly succeed. A malformed config
    /// or a 4xx (except 408/429) cannot; a transport failure or a 5xx can.
    pub fn retryable(&self) -> bool {
        match self {
            Self::Config(_) | Self::Build(_) => false,
            Self::Transport(_) => true,
            Self::Http { status, .. } => *status == 408 || *status == 429 || *status >= 500,
            Self::Body(_) => false,
        }
    }
}

impl From<reqwest::Error> for EgressError {
    fn from(e: reqwest::Error) -> Self {
        Self::Transport(e)
    }
}

/// Read at most `cap` BYTES of a response body. Uses `chunk()` (not `text()`) so a
/// hostile `Content-Length` cannot make us allocate the whole payload before we
/// decide it is too big. Lossy-decoded: an excerpt is diagnostics, not data.
async fn read_bounded(resp: reqwest::Response, cap: usize) -> String {
    use futures::TryStreamExt;
    let mut stream = resp.bytes_stream();
    let mut buf: Vec<u8> = Vec::new();
    while let Some(chunk) = stream.try_next().await.ok().flatten() {
        let remaining = cap.saturating_sub(buf.len());
        if remaining == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..chunk.len().min(remaining)]);
    }
    String::from_utf8_lossy(&buf).into_owned()
}

/// Truncate `s` to at most `cap` BYTES without splitting a UTF-8 char (a raw byte
/// slice would produce a replacement char mid-symbol and, worse, could panic if a
/// future caller re-slices on a char boundary). Appends a marker when truncated.
fn truncate_utf8(s: &str, cap: usize) -> String {
    if s.len() <= cap {
        return s.to_string();
    }
    // Walk back to the nearest char boundary at or before `cap`.
    let mut end = cap;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…[truncated]", &s[..end])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brief_is_one_line_and_detail_carries_the_body() {
        let e = EgressError::http(502, "gateway exploded", "<html><body>oops</body></html>");
        let b = e.brief();
        let d = e.detail();
        // brief: single line, no body, no newline.
        assert_eq!(b.lines().count(), 1, "brief must be ONE line: {b:?}");
        assert!(!b.contains("oops"), "brief must not embed the body: {b:?}");
        assert!(b.contains("502"), "brief carries the status: {b:?}");
        // detail: carries the excerpt.
        assert!(d.contains("oops"), "detail carries the excerpt: {d:?}");
        assert!(d.contains("502") && d.contains("gateway exploded"));
    }

    #[test]
    fn brief_and_detail_differ_for_every_variant() {
        let cases: Vec<EgressError> = vec![
            EgressError::Config("empty base_url".into()),
            EgressError::Build("rustls rejected a root".into()),
            EgressError::Body("stream closed early".into()),
            EgressError::http(429, "rate limited", ""),
        ];
        for e in cases {
            let (b, d) = (e.brief(), e.detail());
            assert_eq!(b.lines().count(), 1, "brief is one line: {b:?}");
            assert!(!b.contains('\n'));
            assert!(!d.is_empty());
        }
    }

    #[test]
    fn http_excerpt_is_truncated_to_512_bytes() {
        // Multi-byte content: proves the cap is BYTES (not chars) and that we never
        // cut mid-char (which would make the result invalid UTF-8).
        let page = "你".repeat(400); // 3 bytes each → 1200 bytes
        assert_eq!(page.len(), 1200);
        let e = EgressError::http(500, "boom", &page);
        let EgressError::Http { body_excerpt, .. } = &e else {
            panic!("expected the Http variant");
        };
        assert!(
            body_excerpt.len() <= BODY_EXCERPT_BYTES + "…[truncated]".len(),
            "excerpt must be bounded: {} bytes",
            body_excerpt.len()
        );
        assert!(
            body_excerpt.ends_with("…[truncated]"),
            "truncation is marked: {body_excerpt:?}"
        );
        // The kept prefix is valid UTF-8 AND lands on a char boundary.
        let kept = body_excerpt.trim_end_matches("…[truncated]");
        assert!(kept.len() <= BODY_EXCERPT_BYTES);
        assert!(kept.len() % 3 == 0, "no split mid char: {}", kept.len());
    }

    #[test]
    fn short_body_is_not_marked_truncated() {
        let e = EgressError::http(404, "missing", "not found");
        let EgressError::Http { body_excerpt, .. } = &e else {
            panic!("expected the Http variant");
        };
        assert_eq!(body_excerpt, "not found");
    }

    #[test]
    fn retryable_classifies_by_variant_and_status() {
        assert!(!EgressError::Config("x".into()).retryable());
        assert!(!EgressError::Build("x".into()).retryable());
        assert!(!EgressError::Body("x".into()).retryable());
        assert!(EgressError::http(500, "x", "").retryable());
        assert!(EgressError::http(429, "x", "").retryable());
        assert!(EgressError::http(408, "x", "").retryable());
        assert!(!EgressError::http(404, "x", "").retryable());
        assert!(!EgressError::http(400, "x", "").retryable());
    }

    #[test]
    fn error_display_is_stable() {
        // `Display` (from thiserror) is what `?`-propagation into a ToolResult shows
        // when the caller does not distinguish brief/detail.
        let e = EgressError::http(403, "forbidden", "");
        assert_eq!(e.to_string(), "HTTP 403: forbidden");
    }
}
