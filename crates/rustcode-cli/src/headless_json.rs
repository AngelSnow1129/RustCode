use serde::Serialize;

/// Versioned, driver-owned JSONL protocol for `rustcode -p`.
///
/// This deliberately remains in the CLI boundary: it projects neutral runtime
/// events without creating a second runtime protocol or state owner.
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub(crate) enum HeadlessEvent {
    #[serde(rename = "run.started")]
    RunStarted {
        schema_version: u32,
        provider: String,
        model: String,
    },
    #[serde(rename = "message.delta")]
    MessageDelta { text: String },
    #[serde(rename = "reasoning.delta")]
    ReasoningDelta { text: String },
    #[serde(rename = "tool.started")]
    ToolStarted {
        id: String,
        name: String,
        arguments: String,
    },
    #[serde(rename = "tool.completed")]
    ToolCompleted {
        id: String,
        content: String,
        is_error: bool,
    },
    #[serde(rename = "usage")]
    Usage {
        round: u32,
        turn_id: u64,
        request_id: u64,
        prompt_tokens: u32,
        completion_tokens: u32,
        cached_tokens: u32,
        elapsed_ms: u64,
        reasoning_elapsed_ms: u64,
    },
    #[serde(rename = "error")]
    Error {
        message: String,
        http_status: Option<u16>,
        code: Option<String>,
        retryable: Option<bool>,
    },
    #[serde(rename = "retry")]
    Retry {
        kind: String,
        attempt: u32,
        max_attempts: u32,
        recovered: Option<bool>,
        backoff_secs: Option<u64>,
        reason: Option<String>,
    },
    #[serde(rename = "warning")]
    Warning { message: String },
    #[serde(rename = "rate_limit")]
    RateLimit {
        reset_at: String,
        reset_label: String,
        seconds_until_reset: Option<u64>,
        auto_resuming: bool,
        server_message: Option<String>,
    },
    #[serde(rename = "turn.completed")]
    TurnCompleted {
        stop_reason: rustcode_kernel::event::StopReason,
        exit_code: i32,
        duration_ms: u64,
        rounds: usize,
        tool_calls: usize,
        total_tokens: usize,
        prompt_tokens: usize,
        completion_tokens: usize,
        cached_tokens: usize,
        cache_hit_rate: Option<f64>,
        ttft_ms: Option<u64>,
        snapshot_error: Option<String>,
    },
    #[serde(rename = "run.failed")]
    RunFailed { exit_code: i32, message: String },
}

pub(crate) fn line(event: &HeadlessEvent) -> std::io::Result<Vec<u8>> {
    let mut line = serde_json::to_vec(event)?;
    line.push(b'\n');
    Ok(line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_is_one_valid_line_with_escaped_content() {
        let line = line(&HeadlessEvent::MessageDelta {
            text: "a\nb".into(),
        })
        .unwrap();
        assert_eq!(line.last(), Some(&b'\n'));
        assert_eq!(line.iter().filter(|byte| **byte == b'\n').count(), 1);
        let decoded: serde_json::Value = serde_json::from_slice(&line).unwrap();
        assert_eq!(decoded["type"], "message.delta");
        assert_eq!(decoded["text"], "a\nb");
    }

    /// A-11: the model-fallback machine token must reach the machine channel
    /// VERBATIM, while the human-facing prose stays in `message`.
    ///
    /// This is the machine half of FR-4.5: a `--json` consumer keys off `code`,
    /// so the token is a wire contract and must never be localized, translated,
    /// or rewritten -- the stability of the constant is the contract.
    #[test]
    fn model_fallback_token_is_carried_verbatim_in_the_code_field() {
        for token in [
            rustcode_coding::fallback::tokens::STARTED,
            rustcode_coding::fallback::tokens::EXHAUSTED,
        ] {
            let line = line(&HeadlessEvent::Error {
                // The rendered prose a user would see; the token beside it must
                // not be affected by it.
                message: "provider exploded".into(),
                http_status: None,
                code: Some(token.to_string()),
                retryable: Some(false),
            })
            .unwrap();
            let decoded: serde_json::Value = serde_json::from_slice(&line).unwrap();
            assert_eq!(decoded["type"], "error");
            assert_eq!(
                decoded["code"], token,
                "the fallback token must be emitted byte-for-byte"
            );
            assert_eq!(decoded["retryable"], false);
        }
    }

    /// The tokens themselves are stable ASCII identifiers with the documented
    /// `model_fallback_` namespace -- a consumer's switch statement depends on it,
    /// so this locks the literal values rather than merely their existence.
    #[test]
    fn model_fallback_tokens_are_stable_ascii_identifiers() {
        assert_eq!(
            rustcode_coding::fallback::tokens::STARTED,
            "model_fallback_started"
        );
        assert_eq!(
            rustcode_coding::fallback::tokens::EXHAUSTED,
            "model_fallback_exhausted"
        );
        for token in [
            rustcode_coding::fallback::tokens::STARTED,
            rustcode_coding::fallback::tokens::EXHAUSTED,
        ] {
            assert!(
                token.is_ascii(),
                "a wire token must be ASCII, got {token:?}"
            );
            assert!(
                token.starts_with("model_fallback_"),
                "tokens live in one namespace, got {token:?}"
            );
        }
    }
}
