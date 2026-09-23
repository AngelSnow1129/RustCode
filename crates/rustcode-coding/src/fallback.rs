//! Turn-scoped model fallback bookkeeping (FR-1/FR-3/FR-4 of
//! `docs/model-fallback-requirements.md`).
//!
//! This module owns the *decision* half of fallback: which model to try next
//! and what to say when nothing is left. It is deliberately pure -- no config
//! access, no provider construction, no event emission -- so the owner loop can
//! drive it while keeping its own lifecycle invariants (generation, held turn,
//! snapshot) under sole ownership.
//!
//! Whether a failure *justifies* a failover at all is a separate question with a
//! separate owner: [`rustcode_capabilities::fallback::fallback_eligible`].

use crate::controllers::retry_reason_label;
use rustcode_kernel::agent::Outcome;
use rustcode_kernel::event::StopReason;

/// One failed attempt on the way down a fallback chain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FallbackAttempt {
    /// The model selection id that failed.
    pub model: String,
    /// Human-readable cause, already localized by the caller's edge (the runtime
    /// passes `retry_reason_label` output). Kept as an opaque string on purpose:
    /// this type composes text, it does not choose a language.
    pub reason: String,
}

/// A turn's ordered walk over `[primary, fb1, fb2, ...]`.
///
/// Movement is monotonic (FR-3.4): once a model has been attempted and failed it
/// is never revisited, so a misconfigured chain cannot ping-pong between two
/// models. Exhaustion is explicit rather than silent (FR-3.6).
#[derive(Clone, Debug)]
pub struct FallbackWalk {
    /// `[primary, fb1, fb2, ...]` -- always at least two entries; a single-model
    /// plan is not a walk, and [`Self::new`] returns `None` for it.
    plan: Vec<String>,
    /// Index into `plan` of the model currently being attempted.
    current: usize,
    failures: Vec<FallbackAttempt>,
}

impl FallbackWalk {
    /// Build a walk for `primary` with `chain` as its fallback targets (already
    /// sanitized by [`rustcode_config::config::Config::model_fallback_chain`]).
    ///
    /// Returns `None` when there is nothing to fall back to, so the caller keeps
    /// its existing single-shot behaviour untouched -- the default-off contract
    /// of FR-1.2. `primary` is never added twice: a chain that lists the primary
    /// (already stripped upstream, but defended here) contributes no walk.
    pub fn new(primary: &str, chain: &[String]) -> Option<Self> {
        let primary = primary.trim();
        if primary.is_empty() {
            return None;
        }
        let mut plan = vec![primary.to_string()];
        for target in chain {
            let target = target.trim();
            if target.is_empty() || target == primary || plan.iter().any(|seen| seen == target) {
                continue;
            }
            plan.push(target.to_string());
        }
        if plan.len() < 2 {
            return None;
        }
        Some(Self {
            plan,
            current: 0,
            failures: Vec::new(),
        })
    }

    /// The model being attempted right now.
    pub fn current_model(&self) -> &str {
        &self.plan[self.current]
    }

    /// How many models remain after the current one.
    pub fn remaining(&self) -> usize {
        self.plan.len() - 1 - self.current
    }

    /// Record that the CURRENT model failed, and step to the next target.
    ///
    /// Returns the next model to try, or `None` when the chain is exhausted --
    /// at which point the caller must terminate explicitly with
    /// [`Self::failures_diagnostic`] rather than silently reporting an empty
    /// success (FR-3.6).
    pub fn record_failure(&mut self, reason: impl Into<String>) -> Option<&str> {
        self.failures.push(FallbackAttempt {
            model: self.plan[self.current].clone(),
            reason: reason.into(),
        });
        if self.current + 1 < self.plan.len() {
            self.current += 1;
            Some(&self.plan[self.current])
        } else {
            None
        }
    }

    /// Every failed attempt so far, in order (primary first).
    pub fn failures(&self) -> &[FallbackAttempt] {
        &self.failures
    }

    /// Locale-free `model: reason` list for the terminal error (FR-4.4). The
    /// surrounding prose is supplied by an i18n message, so this stays pure and
    /// every driver localizes it through the normal path.
    pub fn failures_diagnostic(&self) -> String {
        self.failures
            .iter()
            .map(|attempt| format!("{}: {}", attempt.model, attempt.reason))
            .collect::<Vec<_>>()
            .join("; ")
    }
}

/// Machine-readable tokens for the fallback events, for `--json` / daemon / webui
/// consumers. The human-facing prose travels separately through i18n; these never
/// get localized (FR-4.5).
pub mod tokens {
    /// A failover is starting: the current model failed and a target is chosen.
    pub const STARTED: &str = "model_fallback_started";
    /// Every model in the chain failed; the turn ends.
    pub const EXHAUSTED: &str = "model_fallback_exhausted";
}

/// Human-readable cause for a failed attempt, reusing the SAME localized labels
/// as the in-turn retry notices so a user never sees two names for one problem
/// (FR-4.3).
///
/// Classification order mirrors [`fallback_eligible`]: the structured HTTP status
/// is the most specific fact we hold, then the terminal [`StopReason`], and only
/// then the free-form provider message.
///
/// [`fallback_eligible`]: rustcode_capabilities::fallback::fallback_eligible
pub fn attempt_reason(outcome: &Outcome) -> String {
    if let Some(status) = outcome.http_status {
        return format!("HTTP {status}");
    }
    let reason = match outcome.stop {
        StopReason::RateLimited => Some(rustcode_kernel::event::RetryReason::RateLimited),
        StopReason::Timeout => Some(rustcode_kernel::event::RetryReason::Timeout),
        StopReason::ProviderError => Some(rustcode_kernel::event::RetryReason::UpstreamUnavailable),
        _ => None,
    };
    if let Some(reason) = reason {
        return retry_reason_label(reason).into_owned();
    }
    outcome
        .error
        .clone()
        .unwrap_or_else(|| "provider error".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustcode_kernel::event::StopReason;

    fn chain(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn no_targets_means_no_walk() {
        assert!(FallbackWalk::new("primary", &[]).is_none());
        // A chain that only repeats the primary has nothing to walk to.
        assert!(FallbackWalk::new("primary", &chain(&["primary"])).is_none());
        assert!(FallbackWalk::new("", &chain(&["fb"])).is_none());
    }

    #[test]
    fn walks_forward_in_configured_order() {
        let mut walk = FallbackWalk::new("primary", &chain(&["fb1", "fb2"])).unwrap();
        assert_eq!(walk.current_model(), "primary");
        assert_eq!(walk.remaining(), 2);

        let next = walk.record_failure("rate limited").unwrap().to_string();
        assert_eq!(next, "fb1");
        assert_eq!(walk.current_model(), "fb1");
        assert_eq!(walk.remaining(), 1);

        let next = walk.record_failure("upstream 503").unwrap().to_string();
        assert_eq!(next, "fb2");
        assert_eq!(walk.remaining(), 0);
    }

    #[test]
    fn exhaustion_is_explicit_and_lists_every_attempt() {
        let mut walk = FallbackWalk::new("primary", &chain(&["fb1"])).unwrap();
        assert!(walk.record_failure("HTTP 429").is_some());
        // Last target failed: no next target, and the caller learns why.
        assert!(walk.record_failure("HTTP 503").is_none());
        assert_eq!(
            walk.failures_diagnostic(),
            "primary: HTTP 429; fb1: HTTP 503"
        );
    }

    /// FR-3.4: movement is monotonic -- a walked-over model is never revisited.
    #[test]
    fn a_failed_target_is_never_revisited() {
        let mut walk = FallbackWalk::new("primary", &chain(&["fb1", "fb2"])).unwrap();
        let mut seen = vec![walk.current_model().to_string()];
        while let Some(next) = walk.record_failure("boom") {
            seen.push(next.to_string());
        }
        assert_eq!(seen, vec!["primary", "fb1", "fb2"]);
        // Once exhausted it stays exhausted; no wrap-around.
        assert!(walk.record_failure("boom again").is_none());
        assert_eq!(walk.current_model(), "fb2");
    }

    /// A chain may not smuggle a duplicate that would re-run the same model.
    #[test]
    fn duplicate_targets_collapse() {
        let walk = FallbackWalk::new("primary", &chain(&["fb1", "fb1", "fb2"])).unwrap();
        let mut seen = vec![walk.current_model().to_string()];
        let mut walk = walk;
        while let Some(next) = walk.record_failure("boom") {
            seen.push(next.to_string());
        }
        assert_eq!(seen, vec!["primary", "fb1", "fb2"]);
    }

    #[test]
    fn attempt_reason_prefers_the_structured_status_then_stop_reason() {
        let mut with_status = Outcome {
            stop: StopReason::ProviderError,
            ..Default::default()
        };
        with_status.http_status = Some(503);
        // A concrete status beats the coarser stop-reason label.
        assert_eq!(attempt_reason(&with_status), "HTTP 503");

        let rate_limited = Outcome {
            stop: StopReason::RateLimited,
            ..Default::default()
        };
        assert_eq!(
            attempt_reason(&rate_limited),
            retry_reason_label(rustcode_kernel::event::RetryReason::RateLimited).into_owned()
        );

        let timed_out = Outcome {
            stop: StopReason::Timeout,
            ..Default::default()
        };
        assert_eq!(
            attempt_reason(&timed_out),
            retry_reason_label(rustcode_kernel::event::RetryReason::Timeout).into_owned()
        );

        // No status and no meaningful stop reason: fall back to the raw message
        // rather than inventing a classification we do not have.
        let mut only_text = Outcome {
            stop: StopReason::Stopped,
            ..Default::default()
        };
        only_text.error = Some("connection reset".into());
        assert_eq!(attempt_reason(&only_text), "connection reset");

        // Nothing at all: still a non-empty, stable placeholder.
        assert_eq!(
            attempt_reason(&Outcome::default()),
            "provider error".to_string()
        );
    }
}
