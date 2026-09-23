//! Single source of truth for "is this failure worth retrying on a *different*
//! model?".
//!
//! Why this lives here and not at the call sites: two consumers need the exact
//! same judgement -- the `task`/`team` subagent driver and the main-agent coding
//! runtime -- and a second hand-written copy is how the two drift until one of
//! them replays a terminal failure (a 401 that will never succeed) or refuses to
//! fail over on a rate limit. Requirement FR-6.1 of
//! `docs/model-fallback-requirements.md` freezes exactly one predicate.
//!
//! Deliberately NOT in the kernel: the kernel owns mechanism (neutral
//! `Outcome`/`StopReason`), not the policy of when to change models. It is
//! feature-gate-free so a tools-only build and the provider runtime both get it.

use std::sync::Arc;

use rustcode_kernel::agent::Outcome;
use rustcode_kernel::event::StopReason;
use rustcode_kernel::provider::LlmProvider;

/// The predicate in its typing-friendly form: the runtime knows a terminal
/// [`StopReason`] plus the structured classification it observed, not a whole
/// [`Outcome`]. [`fallback_eligible`] delegates here so both shapes share ONE
/// decision (FR-6.1).
pub fn fallback_eligible_for_stop(
    stop: StopReason,
    provider_retryable: Option<bool>,
    http_status: Option<u16>,
    produced_output: bool,
    cancelled: bool,
) -> bool {
    if cancelled {
        return false;
    }
    // Content-free only: once the model has said or done anything, a retry on
    // another model would duplicate that work rather than recover from it.
    if produced_output {
        return false;
    }
    match stop {
        // A stream idle timeout is intrinsically transient, and a rate limit is
        // by definition a capacity problem another model need not share.
        StopReason::Timeout | StopReason::RateLimited => true,
        StopReason::ProviderError => match provider_retryable {
            Some(retryable) => retryable,
            // No structured verdict: accept only the transient HTTP classes.
            // 408 request timeout, 425 too early, 429 rate limited, and 5xx.
            None => matches!(http_status, Some(408 | 425 | 429) | Some(500..=599)),
        },
        _ => false,
    }
}

/// Whether `outcome` represents a failure that a DIFFERENT model could plausibly
/// survive, making it a candidate for model fallback.
///
/// Returns `false` (never fall back) when:
/// - the run was cancelled -- the user asked to stop, so the failure is not a
///   provider-availability question (FR-2.1.6 / FR-5.1);
/// - the run produced any text or tool result -- replaying would duplicate
///   side effects and repeat already-visible output (FR-2.3 / FR-5.4).
///
/// Otherwise the provider's own structured classification is authoritative:
/// `Some(true)` fails over, `Some(false)` does NOT (a terminal failure must not
/// be replayed merely because a compatible endpoint attached a nominally
/// transient status). Only when the provider gave no classification do we fall
/// back to the HTTP status, restricted to the transient classes.
pub fn fallback_eligible(outcome: &Outcome, cancelled: bool) -> bool {
    let produced_output = !outcome.text.is_empty() || !outcome.tool_results.is_empty();
    fallback_eligible_for_stop(
        outcome.stop,
        outcome.provider_retryable,
        outcome.http_status,
        produced_output,
        cancelled,
    )
}

/// Candidate models for a fallback walk, in priority order: `primary` first, then
/// `extras` (the configured explicit chain, FR-6.2), de-duplicated by provider
/// IDENTITY (`Arc::ptr_eq`).
///
/// Why identity and not the display name: two configured providers may legitimately
/// expose the same raw model name (two gateways serving the same model), and a
/// dedupe on `model_name()` would silently drop a WORKING second route. Why here
/// and not at the call sites: the drivers (main turn, `task`, `team`,
/// `parallel_edit`, `code_review`) used to carry identical hand-rolled dedupe
/// loops -- the same drift this module exists to prevent for the eligibility
/// predicate (FR-6.1).
pub fn chain_candidates(
    primary: Arc<dyn LlmProvider>,
    extras: impl IntoIterator<Item = Arc<dyn LlmProvider>>,
) -> Vec<Arc<dyn LlmProvider>> {
    let mut candidates = vec![primary];
    for candidate in extras {
        if !candidates.iter().any(|seen| Arc::ptr_eq(seen, &candidate)) {
            candidates.push(candidate);
        }
    }
    candidates
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustcode_kernel::event::StopReason;

    /// A stand-in provider whose only meaningful property is its IDENTITY: two
    /// instances sharing a display name are still different configured routes.
    struct NamedProvider(&'static str);

    #[async_trait::async_trait]
    impl LlmProvider for NamedProvider {
        fn model_name(&self) -> &str {
            self.0
        }

        async fn chat_stream(
            &self,
            _messages: &[rustcode_kernel::message::Message],
            _tools: &[rustcode_kernel::tool::ToolDef],
            _options: &rustcode_kernel::provider::ChatOptions,
        ) -> Result<
            futures::stream::BoxStream<'static, rustcode_kernel::stream::StreamEvent>,
            rustcode_kernel::stream::ProviderError,
        > {
            // Never called: the helper under test only compares identities.
            Err(rustcode_kernel::stream::ProviderError::default())
        }
    }

    #[test]
    fn chain_candidates_dedupe_by_identity_in_priority_order() {
        let primary: Arc<dyn LlmProvider> = Arc::new(NamedProvider("same-model"));
        let twin: Arc<dyn LlmProvider> = Arc::new(NamedProvider("same-model"));
        let backup: Arc<dyn LlmProvider> = Arc::new(NamedProvider("backup"));

        let candidates = chain_candidates(
            Arc::clone(&primary),
            [
                Arc::clone(&primary), // the primary re-listed as an extra: dropped
                Arc::clone(&twin),    // SAME display name, DIFFERENT route: kept
                Arc::clone(&backup),  // first appearance: kept
                Arc::clone(&backup),  // identity repeat: dropped
            ],
        );

        assert_eq!(candidates.len(), 3);
        assert!(Arc::ptr_eq(&candidates[0], &primary), "primary stays first");
        assert!(
            Arc::ptr_eq(&candidates[1], &twin),
            "a second route sharing the display name must SURVIVE: dedupe is by identity, not model_name()"
        );
        assert!(Arc::ptr_eq(&candidates[2], &backup));
        // No extras: the list is exactly the primary.
        assert_eq!(chain_candidates(Arc::clone(&primary), []).len(), 1);
    }

    fn outcome(stop: StopReason) -> Outcome {
        Outcome {
            stop,
            ..Default::default()
        }
    }

    #[test]
    fn transient_stops_are_eligible() {
        assert!(fallback_eligible(&outcome(StopReason::Timeout), false));
        assert!(fallback_eligible(&outcome(StopReason::RateLimited), false));
    }

    #[test]
    fn a_clean_stop_is_not_a_failure() {
        assert!(!fallback_eligible(&outcome(StopReason::Stopped), false));
        assert!(!fallback_eligible(&outcome(StopReason::Cancelled), false));
        assert!(!fallback_eligible(
            &outcome(StopReason::PolicyDenied),
            false
        ));
        assert!(!fallback_eligible(&outcome(StopReason::MaxRounds), false));
    }

    #[test]
    fn cancel_supersedes_an_otherwise_eligible_failure() {
        assert!(!fallback_eligible(&outcome(StopReason::Timeout), true));
        assert!(!fallback_eligible(
            &outcome(StopReason::ProviderError),
            true
        ));
    }

    #[test]
    fn produced_output_blocks_fallback() {
        let mut with_text = outcome(StopReason::ProviderError);
        with_text.text = "half an answer".into();
        assert!(!fallback_eligible(&with_text, false));

        let mut with_tools = outcome(StopReason::Timeout);
        with_tools.tool_results.push(Default::default());
        assert!(!fallback_eligible(&with_tools, false));
    }

    #[test]
    fn structured_classification_is_authoritative() {
        // A provider that says "retryable" wins over the absence of an HTTP status.
        let mut retryable = outcome(StopReason::ProviderError);
        retryable.provider_retryable = Some(true);
        assert!(fallback_eligible(&retryable, false));

        // A provider that says "terminal" wins OVER a nominally transient status:
        // a 5xx that the adapter classified as permanent must not be replayed.
        let mut terminal = outcome(StopReason::ProviderError);
        terminal.provider_retryable = Some(false);
        terminal.http_status = Some(503);
        assert!(!fallback_eligible(&terminal, false));
    }

    #[test]
    fn unclassified_provider_errors_use_the_transient_http_classes() {
        for status in [408u16, 425, 429, 500, 503, 599] {
            let mut o = outcome(StopReason::ProviderError);
            o.http_status = Some(status);
            assert!(fallback_eligible(&o, false), "{status} should be eligible");
        }
        // Auth / bad-request / not-found: another model would not help.
        for status in [400u16, 401, 403, 404, 422] {
            let mut o = outcome(StopReason::ProviderError);
            o.http_status = Some(status);
            assert!(
                !fallback_eligible(&o, false),
                "{status} must not be eligible"
            );
        }
        // No status, no classification -> cannot justify a failover.
        assert!(!fallback_eligible(
            &outcome(StopReason::ProviderError),
            false
        ));
    }
}
