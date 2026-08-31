//! Configuration for NON-LLM outbound services (search, forges, gateways).
//!
//! Deliberately shaped like an LLM provider config -- `base_url` + credential +
//! `custom_headers` + timeout -- so every outbound endpoint in the product is
//! configured, validated and redacted the same way instead of each tool inventing
//! its own env-var convention.

use std::collections::HashMap;
use std::time::Duration;

use zeroize::Zeroize;

/// A credential that must never reach a log, a `Debug` dump, a serialized config
/// file, or the model.
///
/// Backed by `zeroize` so the plaintext is wiped on drop rather than lingering in
/// freed heap memory. `Debug` prints `***` and `Serialize` is intentionally NOT
/// implemented: the one way to get a config file written without a key is to make
/// it unrepresentable.
pub struct SecretString(String);

impl SecretString {
    /// Wrap a credential. Prefer sourcing it from config or the environment --
    /// never a literal in code.
    pub fn new(raw: impl Into<String>) -> Self {
        Self(raw.into())
    }

    /// Read the plaintext. Every call site is a deliberate boundary: the value is
    /// about to leave the process on the wire.
    pub fn reveal(&self) -> &str {
        &self.0
    }

    /// True when the credential is absent or all whitespace.
    pub fn is_empty(&self) -> bool {
        self.0.trim().is_empty()
    }
}

impl Drop for SecretString {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl Clone for SecretString {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl std::fmt::Debug for SecretString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(if self.is_empty() { "<empty>" } else { "***" })
    }
}

impl From<String> for SecretString {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for SecretString {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

/// How to reach one external HTTP service.
///
/// The non-LLM counterpart of a provider config. Kept separate from
/// [`crate::egress::HttpClientSpec`] on purpose: this is *user configuration*
/// (where do I point the search tool), that is *transport policy* (how do we
/// build the client once we know where).
#[derive(Debug, Clone, Default)]
pub struct ExternalServiceConfig {
    /// Endpoint base URL, e.g. `https://api.moonshot.cn/v1`. Trailing slashes are
    /// trimmed by [`ExternalServiceConfig::normalized_base_url`].
    pub base_url: String,
    /// Credential for this service. `None` means the service needs no auth (some
    /// self-hosted endpoints don't) and is distinct from "misconfigured".
    pub api_key: Option<SecretString>,
    /// Arbitrary extra headers (tenant ids, gateway auth). Sourced from config
    /// only -- never hardcoded.
    pub custom_headers: HashMap<String, String>,
    /// Whole-request budget override. `None` falls back to
    /// [`DEFAULT_REQUEST_TIMEOUT`](super::client::DEFAULT_REQUEST_TIMEOUT).
    pub timeout_secs: Option<u64>,
}

impl ExternalServiceConfig {
    /// Whether this service can actually be called.
    ///
    /// Callers use it to decide between "don't register the tool at all" and
    /// "register it and fail at request time" -- the former is what we want, so a
    /// missing credential produces a clean absence instead of a runtime error the
    /// model then has to work around.
    pub fn is_configured(&self) -> bool {
        !self.base_url.trim().is_empty()
    }

    /// Whether the service is configured AND its (optional) credential is usable.
    /// A service that needs no key is `ready` as soon as `base_url` is set.
    pub fn is_ready(&self) -> bool {
        self.is_configured() && self.api_key.as_ref().is_none_or(|k| !k.is_empty())
    }

    /// `base_url` without trailing slashes, so callers can `format!("{base}/path")`
    /// without producing `//path`.
    pub fn normalized_base_url(&self) -> &str {
        self.base_url.trim_end_matches('/')
    }

    /// The request budget for this service.
    pub fn request_timeout(&self) -> Option<Duration> {
        self.timeout_secs.map(Duration::from_secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_debug_never_leaks() {
        let s = SecretString::new("sk-super-secret-value");
        let dbg = format!("{s:?}");
        assert_eq!(dbg, "***");
        assert!(!dbg.contains("super-secret"), "debug must not leak: {dbg}");
        // ...but the value is still usable at the deliberate boundary.
        assert_eq!(s.reveal(), "sk-super-secret-value");
    }

    #[test]
    fn empty_secret_is_distinct_from_present() {
        assert!(SecretString::new("").is_empty());
        assert!(
            SecretString::new("   ").is_empty(),
            "whitespace-only is empty"
        );
        assert!(!SecretString::new("k").is_empty());
        // An empty credential still redacts rather than printing nothing at all,
        // so a missing key is visible in a dump.
        assert_eq!(format!("{:?}", SecretString::new("")), "<empty>");
    }

    #[test]
    fn clone_keeps_the_value_and_drops_do_not_alias() {
        let a = SecretString::new("abc");
        let b = a.clone();
        assert_eq!(b.reveal(), "abc");
        drop(a);
        // `b` is an independent allocation; zeroizing `a` must not blank it.
        assert_eq!(b.reveal(), "abc");
    }

    #[test]
    fn is_configured_only_requires_a_base_url() {
        let mut c = ExternalServiceConfig::default();
        assert!(!c.is_configured());
        assert!(!c.is_ready());

        c.base_url = "https://example.test/v1".into();
        assert!(c.is_configured());
        // No key configured at all ⇒ the service simply needs no auth.
        assert!(c.is_ready());

        // A key that is present but blank is a misconfiguration, not "no auth".
        c.api_key = Some(SecretString::new("  "));
        assert!(!c.is_ready());

        c.api_key = Some(SecretString::new("sk-1"));
        assert!(c.is_ready());
    }

    #[test]
    fn trailing_slashes_are_trimmed() {
        let c = ExternalServiceConfig {
            base_url: "https://example.test/v1///".into(),
            ..ExternalServiceConfig::default()
        };
        assert_eq!(c.normalized_base_url(), "https://example.test/v1");
        assert_eq!(
            format!("{}/search", c.normalized_base_url()),
            "https://example.test/v1/search"
        );
    }

    #[test]
    fn timeout_override_is_optional() {
        let c = ExternalServiceConfig::default();
        assert_eq!(c.request_timeout(), None);
        let c = ExternalServiceConfig {
            timeout_secs: Some(30),
            ..ExternalServiceConfig::default()
        };
        assert_eq!(c.request_timeout(), Some(Duration::from_secs(30)));
    }

    #[test]
    fn config_debug_redacts_the_key() {
        let c = ExternalServiceConfig {
            base_url: "https://example.test".into(),
            api_key: Some(SecretString::new("sk-must-not-appear")),
            ..ExternalServiceConfig::default()
        };
        let dbg = format!("{c:?}");
        assert!(
            !dbg.contains("sk-must-not-appear"),
            "derived Debug must redact: {dbg}"
        );
    }
}
