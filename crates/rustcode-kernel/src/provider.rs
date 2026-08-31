use crate::message::Message;
use crate::stream::{ProviderError, StreamEvent, TokenUsage};
use crate::tool::{ToolCall, ToolDef};
use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use serde::{Deserialize, Serialize};

/// NEUTRAL per-call request knobs handed to the provider on each `chat_stream`.
///
/// SLOT, not POLICY. This is the kernel-owned *mechanism* by which the turn loop
/// can carry per-call request options (a reasoning/thinking effort, a
/// `tool_choice`, a `max_tokens`, a `temperature`) down to the provider. The
/// *values* are set by a specialization (via `AgentBuilder::chat_options`); the
/// *meaning on the wire* is the L1 provider ADAPTER's job — it MAPS each neutral
/// knob onto whatever its backend speaks (e.g. `reasoning_effort` → OpenAI's
/// `reasoning_effort` string vs Anthropic's thinking `budget_tokens`), and a given
/// adapter MAY IGNORE any option it does not support. The kernel never interprets
/// these model knobs — it only forwards them. The runtime-only retry owner below
/// is a lifecycle sideband and is deliberately excluded from serialization.
///
/// `ChatOptions::default()` is a NEUTRAL request: every tunable is `None` and
/// `tool_choice` is `ToolChoice::Auto` — i.e. "no opinion", the model decides. An
/// adapter receiving the default uses provider-owned retry behavior, which also
/// preserves direct consumers that do not have a kernel turn lifecycle.
///
/// PREFIX-CACHE: these are a SIDEBAND request parameter, NOT part of the messages
/// or tool block. They do NOT enter the conversation history bytes, so they never
/// perturb the append-only wire prefix the provider's prefix cache keys on.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ChatOptions {
    /// Desired reasoning/thinking effort. `None` = no opinion (adapter default).
    /// The adapter maps the neutral level onto its wire format (OpenAI's
    /// `reasoning_effort` string, Anthropic's thinking `budget_tokens`, …) or
    /// ignores it if unsupported.
    pub reasoning_effort: Option<ReasoningEffort>,
    /// Cap on output tokens for this call. `None` = no opinion (adapter default).
    pub max_tokens: Option<u32>,
    /// Sampling temperature for this call. `None` = no opinion (adapter default).
    pub temperature: Option<f32>,
    /// Whether/how the model must use tools this call. `Auto` (default) = no
    /// opinion — the model decides.
    pub tool_choice: ToolChoice,
    /// Which layer owns HTTP 429 retries for this call. Direct provider
    /// consumers keep the provider default; the kernel turn loop overrides this
    /// so waits remain cancellable and visible.
    #[serde(skip)]
    pub rate_limit_retry_owner: RateLimitRetryOwner,
}

/// Per-call ownership of HTTP 429 retry policy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RateLimitRetryOwner {
    /// The provider adapter may retry a 429 within its bounded OPEN retry loop.
    #[default]
    Provider,
    /// Surface the first 429 so the kernel can apply lifecycle-aware policy.
    Kernel,
}

/// NEUTRAL reasoning/thinking effort level. The provider adapter maps it onto the
/// backend's wire format (e.g. OpenAI's `reasoning_effort` string, or an
/// Anthropic thinking `budget_tokens`); an adapter MAY ignore it if unsupported.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReasoningEffort {
    Low,
    Medium,
    High,
    /// Extra-high effort — some endpoints (e.g. AtomGit Qwen) accept
    /// `reasoning_effort: "xhigh"` between the standard `high` and the ceiling `max`.
    XHigh,
    /// Maximum effort — DeepSeek V4 accepts `reasoning_effort: "max"` beyond the
    /// OpenAI low/medium/high ladder.
    Max,
}

impl ReasoningEffort {
    pub const fn as_str(self) -> &'static str {
        match self {
            ReasoningEffort::Low => "low",
            ReasoningEffort::Medium => "medium",
            ReasoningEffort::High => "high",
            ReasoningEffort::XHigh => "xhigh",
            ReasoningEffort::Max => "max",
        }
    }

    /// Parse a config string (`"low"|"medium"|"high"|"xhigh"|"max"`, case-insensitive) into an
    /// effort level. `None`/empty/`"off"` ⇒ `None` (no opinion); an UNKNOWN value also ⇒
    /// `None` (effort is a non-critical optimization — unlike `reasoning_history`, a typo
    /// degrades to the adapter default rather than failing the turn). Lets a driver plumb
    /// a per-provider `reasoning_effort` config knob into [`ChatOptions::reasoning_effort`].
    pub fn from_config(s: Option<&str>) -> Option<ReasoningEffort> {
        match s.unwrap_or("").trim().to_ascii_lowercase().as_str() {
            "low" => Some(ReasoningEffort::Low),
            "medium" => Some(ReasoningEffort::Medium),
            "high" => Some(ReasoningEffort::High),
            "xhigh" => Some(ReasoningEffort::XHigh),
            "max" => Some(ReasoningEffort::Max),
            _ => None,
        }
    }
}

/// NEUTRAL tool-use directive for one call. The provider adapter maps it onto the
/// backend's tool-choice field (e.g. OpenAI's `tool_choice: "auto"|"required"|
/// "none"`, Anthropic's `tool_choice` object); an adapter MAY ignore it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolChoice {
    /// No opinion — the model decides whether to call a tool. The neutral default.
    #[default]
    Auto,
    /// The model MUST call at least one tool this call.
    Required,
    /// The model MUST call the named tool this call.
    Specific(String),
    /// The model must NOT call a tool this call.
    None,
}

/// Why the model stopped generating. The NEUTRAL, kernel-side spelling of the
/// provider-specific terminal reason (`finish_reason` / `stop_reason`), so a
/// consumer branches on a variant instead of string-matching wire text.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FinishReason {
    /// Normal end of turn (`finish_reason: "stop"` / `stop_reason: "end_turn"`).
    /// The DEFAULT: a stream that simply ends without a terminal signal is a
    /// natural completion, never an error.
    #[default]
    Stop,
    /// Output cap hit — the response is TRUNCATED, not complete
    /// (`finish_reason: "length"` / `stop_reason: "max_tokens"`, or
    /// `StreamEvent::Done { truncated: true }`).
    Length,
    /// The model produced tool calls to execute before it can continue.
    /// Reported even when a backend also said `finish_reason: "stop"` — the tool
    /// calls are what the turn loop acts on, so they win.
    ToolCalls,
    /// Content policy / safety filter cut the response.
    ContentFilter,
    /// An unrecognized terminal reason from the backend. Never invented by the
    /// kernel: it is the honest fallback for a new provider spelling.
    Other,
}

/// One COMPLETE, non-streaming assistant response.
///
/// This is the `[STREAMING]`-spec counterpart to the event stream: callers that
/// want the whole answer at once (sub-agents, one-shot completions, tests) use
/// [`LlmProvider::chat`] instead of hand-folding [`StreamEvent`]s. The fields
/// mirror exactly what the default fold collects, so the two paths cannot drift.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChatResponse {
    /// Accumulated assistant text.
    pub text: String,
    /// Accumulated thinking/reasoning channel text, or `None` when the model
    /// emitted no reasoning at all (distinct from an EMPTY reasoning string).
    pub reasoning: Option<String>,
    /// Whole tool calls requested by the model, in emission order.
    pub tool_calls: Vec<ToolCall>,
    /// Reported token usage, or `None` when the provider reported none.
    pub usage: Option<TokenUsage>,
    /// Why generation stopped.
    pub finish_reason: FinishReason,
}

/// LLM backend abstraction. The turn loop never names Claude/OpenAI/Ollama — it
/// only calls `chat_stream` once per turn and consumes the event stream.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn model_name(&self) -> &str;
    /// Effective context window in tokens. 0 = unknown.
    fn context_window(&self) -> u32 {
        0
    }
    /// Bind this provider to its owning Agent's session id, ONCE. The kernel calls
    /// this at spawn — the single point where the session id (allocated by the coding
    /// layer's `prepare`, threaded in via `AgentBuilder::session_id`) meets the
    /// provider — so no driver re-threads it. An adapter forwards it as the
    /// `x-rustcode-session-id` header, letting a forwarding gateway (LiteLLM) pin the
    /// whole conversation to one upstream for prefix-cache affinity.
    ///
    /// This is a one-shot binding, NOT a mutable setter: the session id is constant
    /// for an Agent's life (a `/session` switch rebuilds the Agent + provider, not the
    /// id in place), so adapters back it with a `OnceLock`. Default no-op: adapters
    /// that don't forward an affinity id, and test doubles, ignore it. Never called ⇒
    /// no affinity (header omitted) — the neutral default for session-less sub-agents.
    fn bind_session_id(&self, _session_id: &str) {}
    /// Open the stream for one turn. `Err` = a failed OPEN (auth/connect/etc.);
    /// the stream itself may then still fail mid-flight via `StreamEvent::Error`.
    ///
    /// `options` carries the NEUTRAL per-call request knobs (reasoning effort,
    /// tool_choice, max_tokens, temperature). The kernel forwards them verbatim;
    /// it is the adapter's job to MAP each onto its wire format, and an adapter MAY
    /// IGNORE any option it does not support. `ChatOptions::default()` is a neutral
    /// request (all `None` + `ToolChoice::Auto`), so most test doubles just ignore
    /// `_options`.
    async fn chat_stream(
        &self,
        messages: &[Message],
        tools: &[ToolDef],
        options: &ChatOptions,
    ) -> Result<BoxStream<'static, StreamEvent>, ProviderError>;

    /// Run one NON-STREAMING turn: send the request and return the whole
    /// [`ChatResponse`] at once. `Err` = the call failed (OPEN failure, or the
    /// first mid-flight `StreamEvent::Error`); a partial answer is never
    /// surfaced as success.
    ///
    /// DEFAULT IMPLEMENTATION folds this adapter's own [`Self::chat_stream`], so
    /// every existing adapter (and every test double) gets a correct `chat()`
    /// for free. An adapter SHOULD override it only when the backend has a
    /// genuinely cheaper non-streaming verb (OpenAI-compatible `stream:false`
    /// skips the SSE handshake and the per-chunk decode); an override MUST
    /// reproduce this contract:
    ///
    /// * the FIRST [`StreamEvent::Error`] terminates the fold and is returned as
    ///   `Err` — content emitted before it is discarded;
    /// * [`StreamEvent::Malformed`] is SKIPPED. It is a content-free diagnostic
    ///   signal (an unparseable chunk), never fatal;
    /// * [`StreamEvent::Done { truncated }`] ends the fold: `truncated` ⇒
    ///   [`FinishReason::Length`], otherwise [`FinishReason::Stop`]. A stream
    ///   that ends with NO `Done` keeps the default [`FinishReason::Stop`];
    /// * a non-empty `tool_calls` set overrides the terminal reason with
    ///   [`FinishReason::ToolCalls`] — the tool calls are what a turn loop acts
    ///   on, so they win over a backend that also said `stop`;
    /// * [`StreamEvent::TextDelta`] appends to `text`,
    ///   [`StreamEvent::Reasoning`] appends to `reasoning`,
    ///   [`StreamEvent::ToolCall`] is collected in order, and
    ///   [`StreamEvent::Usage`] OVERWRITES `usage` (last report wins; adapters
    ///   emit a single cumulative figure).
    ///
    /// Observational events (`ReasoningSignature`, `ToolCallDelta`,
    /// `ResponseId`, `ResponseModel`) carry no folded content and are ignored.
    async fn chat(
        &self,
        messages: &[Message],
        tools: &[ToolDef],
        options: &ChatOptions,
    ) -> Result<ChatResponse, ProviderError> {
        let mut stream = self.chat_stream(messages, tools, options).await?;
        let mut text = String::new();
        let mut reasoning: Option<String> = None;
        let mut out = ChatResponse::default();
        while let Some(event) = stream.next().await {
            match event {
                StreamEvent::TextDelta(delta) => text.push_str(&delta),
                StreamEvent::Reasoning(delta) => {
                    reasoning.get_or_insert_with(String::new).push_str(&delta);
                }
                StreamEvent::ToolCall(tc) => out.tool_calls.push(tc),
                StreamEvent::Usage(u) => out.usage = Some(u),
                StreamEvent::Error(e) => return Err(e),
                // Diagnostic only: an unparseable chunk is not a failure.
                StreamEvent::Malformed => {}
                StreamEvent::Done { truncated } => {
                    out.finish_reason = if truncated {
                        FinishReason::Length
                    } else {
                        FinishReason::Stop
                    };
                    break;
                }
                StreamEvent::ReasoningSignature { .. }
                | StreamEvent::ToolCallDelta { .. }
                | StreamEvent::ResponseId(_)
                | StreamEvent::ResponseModel(_) => {}
            }
        }
        out.text = text;
        out.reasoning = reasoning;
        if !out.tool_calls.is_empty() {
            out.finish_reason = FinishReason::ToolCalls;
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reasoning_effort_from_config_maps_levels() {
        assert_eq!(
            ReasoningEffort::from_config(Some("low")),
            Some(ReasoningEffort::Low)
        );
        assert_eq!(
            ReasoningEffort::from_config(Some("medium")),
            Some(ReasoningEffort::Medium)
        );
        assert_eq!(
            ReasoningEffort::from_config(Some("high")),
            Some(ReasoningEffort::High)
        );
        assert_eq!(
            ReasoningEffort::from_config(Some("xhigh")),
            Some(ReasoningEffort::XHigh)
        );
        assert_eq!(ReasoningEffort::XHigh.as_str(), "xhigh");
        assert_eq!(
            ReasoningEffort::from_config(Some("MAX")),
            Some(ReasoningEffort::Max),
            "case-insensitive"
        );
        // off / empty / unset / unknown → no opinion (None), never a panic.
        assert_eq!(ReasoningEffort::from_config(Some("off")), None);
        assert_eq!(ReasoningEffort::from_config(Some("")), None);
        assert_eq!(ReasoningEffort::from_config(None), None);
        assert_eq!(ReasoningEffort::from_config(Some("bogus")), None);
    }

    #[test]
    fn chat_options_default_is_neutral() {
        let o = ChatOptions::default();
        assert_eq!(
            o.reasoning_effort, None,
            "default reasoning_effort must be None"
        );
        assert_eq!(o.max_tokens, None, "default max_tokens must be None");
        assert_eq!(o.temperature, None, "default temperature must be None");
        assert_eq!(
            o.tool_choice,
            ToolChoice::Auto,
            "default tool_choice must be Auto"
        );

        // serde round-trips losslessly.
        let json = serde_json::to_string(&o).unwrap();
        let back: ChatOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(o, back, "default ChatOptions must serde round-trip");

        // A fully-populated value round-trips too.
        let full = ChatOptions {
            reasoning_effort: Some(ReasoningEffort::High),
            max_tokens: Some(1000),
            temperature: Some(0.2),
            tool_choice: ToolChoice::Required,
            rate_limit_retry_owner: RateLimitRetryOwner::Provider,
        };
        let back: ChatOptions =
            serde_json::from_str(&serde_json::to_string(&full).unwrap()).unwrap();
        assert_eq!(full, back, "populated ChatOptions must serde round-trip");

        let mut runtime = full;
        runtime.rate_limit_retry_owner = RateLimitRetryOwner::Kernel;
        let json = serde_json::to_string(&runtime).unwrap();
        assert!(!json.contains("rate_limit_retry_owner"));
        let back: ChatOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(
            back.rate_limit_retry_owner,
            RateLimitRetryOwner::Provider,
            "runtime retry ownership must not leak into persisted/wire options"
        );
    }

    #[test]
    fn reasoning_effort_and_tool_choice_serialize_to_stable_tags() {
        assert_eq!(
            serde_json::to_string(&ReasoningEffort::Low).unwrap(),
            "\"Low\""
        );
        assert_eq!(
            serde_json::to_string(&ReasoningEffort::Medium).unwrap(),
            "\"Medium\""
        );
        assert_eq!(
            serde_json::to_string(&ReasoningEffort::High).unwrap(),
            "\"High\""
        );
        assert_eq!(
            serde_json::to_string(&ToolChoice::Auto).unwrap(),
            "\"Auto\""
        );
        assert_eq!(
            serde_json::to_string(&ToolChoice::Required).unwrap(),
            "\"Required\""
        );
        assert_eq!(
            serde_json::to_string(&ToolChoice::None).unwrap(),
            "\"None\""
        );
        assert_eq!(
            serde_json::to_string(&ToolChoice::Specific("todowrite".into())).unwrap(),
            r#"{"Specific":"todowrite"}"#
        );
    }

    // -----------------------------------------------------------------------
    // `chat()` default fold (spec [STREAMING])
    // -----------------------------------------------------------------------

    /// A test double that replays a canned event list. It implements ONLY
    /// `chat_stream` — proving an adapter (or a test double) gets `chat()` for
    /// free from the default fold, with no per-backend boilerplate.
    struct CannedProvider(Vec<StreamEvent>);

    #[async_trait]
    impl LlmProvider for CannedProvider {
        fn model_name(&self) -> &str {
            "canned"
        }

        async fn chat_stream(
            &self,
            _messages: &[Message],
            _tools: &[ToolDef],
            _options: &ChatOptions,
        ) -> Result<BoxStream<'static, StreamEvent>, ProviderError> {
            let events = self.0.clone();
            Ok(futures::stream::iter(events).boxed())
        }
    }

    fn tc(id: &str, name: &str) -> ToolCall {
        ToolCall {
            id: id.into(),
            name: name.into(),
            arguments: "{}".into(),
        }
    }

    #[tokio::test]
    async fn chat_folds_text_reasoning_usage_and_tool_calls() {
        let p = CannedProvider(vec![
            StreamEvent::Reasoning("think".into()),
            StreamEvent::Reasoning(" more".into()),
            StreamEvent::TextDelta("he".into()),
            StreamEvent::TextDelta("llo".into()),
            StreamEvent::Usage(TokenUsage {
                prompt: 10,
                completion: 4,
                cached: 1,
            }),
            StreamEvent::ToolCall(tc("c1", "read_file")),
            StreamEvent::Done { truncated: false },
        ]);
        let r = p.chat(&[], &[], &ChatOptions::default()).await.unwrap();
        assert_eq!(r.text, "hello");
        assert_eq!(r.reasoning.as_deref(), Some("think more"));
        assert_eq!(
            r.usage,
            Some(TokenUsage {
                prompt: 10,
                completion: 4,
                cached: 1
            })
        );
        assert_eq!(r.tool_calls.len(), 1);
        // Tool calls outrank the backend's own `stop`: the turn loop acts on them.
        assert_eq!(r.finish_reason, FinishReason::ToolCalls);
    }

    #[tokio::test]
    async fn chat_maps_truncated_done_to_length_and_stop_otherwise() {
        let truncated = CannedProvider(vec![
            StreamEvent::TextDelta("cut".into()),
            StreamEvent::Done { truncated: true },
        ]);
        assert_eq!(
            truncated
                .chat(&[], &[], &ChatOptions::default())
                .await
                .unwrap()
                .finish_reason,
            FinishReason::Length
        );

        let stopped = CannedProvider(vec![
            StreamEvent::TextDelta("ok".into()),
            StreamEvent::Done { truncated: false },
        ]);
        assert_eq!(
            stopped
                .chat(&[], &[], &ChatOptions::default())
                .await
                .unwrap()
                .finish_reason,
            FinishReason::Stop
        );

        // A stream that ends with NO `Done` at all is still a natural stop.
        let silent = CannedProvider(vec![StreamEvent::TextDelta("ok".into())]);
        assert_eq!(
            silent
                .chat(&[], &[], &ChatOptions::default())
                .await
                .unwrap()
                .finish_reason,
            FinishReason::Stop
        );
    }

    #[tokio::test]
    async fn chat_skips_malformed_and_keeps_no_reasoning_when_absent() {
        let p = CannedProvider(vec![
            StreamEvent::Malformed,
            StreamEvent::TextDelta("a".into()),
            StreamEvent::Malformed,
        ]);
        let r = p.chat(&[], &[], &ChatOptions::default()).await.unwrap();
        assert_eq!(r.text, "a", "Malformed is a signal, not content");
        assert_eq!(
            r.reasoning, None,
            "no reasoning emitted ⇒ None, not Some(\"\")"
        );
        assert!(r.tool_calls.is_empty());
    }

    #[tokio::test]
    async fn chat_returns_the_first_error_and_discards_partial_content() {
        let p = CannedProvider(vec![
            StreamEvent::TextDelta("partial".into()),
            StreamEvent::Error(ProviderError {
                retryable: false,
                message: "boom".into(),
                http_status: Some(500),
                code: None,
                retry_after_secs: None,
            }),
            StreamEvent::TextDelta("after".into()),
        ]);
        let e = p.chat(&[], &[], &ChatOptions::default()).await.unwrap_err();
        assert_eq!(e.message, "boom");
        assert_eq!(e.http_status, Some(500));
    }

    /// A failing OPEN surfaces from `chat()` exactly as from `chat_stream()`.
    struct FailingOpen;

    #[async_trait]
    impl LlmProvider for FailingOpen {
        fn model_name(&self) -> &str {
            "failing"
        }
        async fn chat_stream(
            &self,
            _messages: &[Message],
            _tools: &[ToolDef],
            _options: &ChatOptions,
        ) -> Result<BoxStream<'static, StreamEvent>, ProviderError> {
            Err(ProviderError {
                retryable: false,
                message: "HTTP 429: slow down".into(),
                http_status: Some(429),
                code: None,
                retry_after_secs: Some(3),
            })
        }
    }

    #[tokio::test]
    async fn chat_propagates_an_open_failure() {
        let e = FailingOpen
            .chat(&[], &[], &ChatOptions::default())
            .await
            .unwrap_err();
        assert_eq!(e.http_status, Some(429));
        assert_eq!(e.retry_after_secs, Some(3));
    }

    #[test]
    fn finish_reason_defaults_to_stop() {
        assert_eq!(FinishReason::default(), FinishReason::Stop);
        let r = ChatResponse::default();
        assert_eq!(r.finish_reason, FinishReason::Stop);
        assert_eq!(r.reasoning, None);
        assert!(r.text.is_empty() && r.tool_calls.is_empty() && r.usage.is_none());
    }
}
