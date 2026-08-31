//! The ONE outbound HTTP client factory for this crate.
//!
//! Every egress call site (LLM providers, `web_fetch`, `web_search`, AtomGit, MCP)
//! used to hand-roll its own `reqwest::Client::builder()`, so proxy handling, TLS
//! trust roots, timeouts and pool policy silently diverged between them. This module
//! is the single home for that policy; callers only describe WHAT they need via
//! [`HttpClientSpec`].
//!
//! # Why the trust-root dance (issue #514)
//!
//! `reqwest` is compiled with the infallible **webpki base** roots
//! (`rustls-tls`, NOT `rustls-tls-native-roots`), because the latter reads the OS
//! store / `SSL_CERT_FILE` STRICTLY and fails `.build()` outright ("zero valid
//! certificates") on a poisoned file. OS roots (corporate MITM CAs) are then layered
//! ON TOP, best-effort, by [`add_trusted_roots`]. Because `reqwest` defers cert
//! validation to `rustls::RootCertStore::add` inside `.build()`, ONE bad root can
//! still abort the whole client -- so [`add_trusted_roots`] pre-probes every cert and
//! [`build_http_client`] keeps a backstop: if the OS-rooted build fails, it retries
//! ONCE on the webpki base and warns. A request-time TLS error beats a client that
//! never builds.

use std::error::Error as _;
use std::time::Duration;

use crate::egress::error::EgressError;

/// How long an idle keep-alive connection may sit in the pool before we drop it.
///
/// reqwest's default is 90s; gateway load balancers commonly close idle connections
/// sooner, and 30s proved too generous against a real gateway -- half-open reuse there
/// surfaced as hyper `IncompleteMessage`, a class the old classifier hard-failed.
/// 15s stays well under observed LB windows while keeping reuse for the back-to-back
/// requests of a tool loop (their gaps are far below 15s). Only affects *idle*
/// connections -- an active stream is never reaped.
///
/// SINGLE SOURCE OF TRUTH: `provider::retry` re-exports this constant rather than
/// keeping a second copy that could drift.
pub const POOL_IDLE_TIMEOUT: Duration = Duration::from_secs(15);

/// Default TCP+TLS connect budget. Deliberately short: a hung connect blocks the
/// whole tool loop, and 10s is ample for a reachable endpoint (the pre-egress
/// `web_fetch` used 5s, the providers 10s+).
pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Default whole-request budget (connect + send + headers + body).
/// Streaming responses are NOT covered by reqwest's `.timeout()` -- it bounds the
/// whole exchange, which is exactly what a tool loop needs as a hard ceiling.
pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(120);

/// Bare fallback User-Agent. No version on purpose: this crate is versioned
/// independently of the product, so a local `CARGO_PKG_VERSION` would be MISLEADING.
/// The host adapter injects the real `rustcode/<version>` through
/// [`HttpClientSpec::user_agent`].
pub const DEFAULT_USER_AGENT: &str = "rustcode";

/// The browser UA shared by `web_fetch` / `web_search` (and `web_fetch`'s `curl`
/// fallback, so both present the same identity). Many sites -- docs hosts, forges --
/// 403 a generic/bot UA. Ported verbatim from the pre-egress `web_fetch`.
pub const BROWSER_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36";

/// The description of an outbound HTTP client: everything a call site may want to
/// vary, and nothing else. Every field has a safe default, so `Default::default()`
/// yields the shared policy (honor the process proxy, trust OS roots on top of the
/// webpki base, follow redirects, 10s connect / 120s request / 15s idle pool).
#[derive(Debug, Clone, PartialEq)]
pub struct HttpClientSpec {
    /// TCP + TLS handshake budget.
    pub connect_timeout: Duration,
    /// Whole-request budget. `None` ⇒ no client-level timeout (callers that wrap
    /// every call in their own `tokio::time::timeout`, e.g. MCP, opt out here).
    pub request_timeout: Option<Duration>,
    /// How long a keep-alive connection may idle in the pool.
    pub pool_idle_timeout: Duration,
    /// `User-Agent`. `None` ⇒ [`DEFAULT_USER_AGENT`].
    pub user_agent: Option<String>,
    /// Explicit proxy URL. Empty/`None` ⇒ the process-wide policy
    /// (`crate::proxy::apply_async_proxy_policy`, which honors `RUSTCODE_PROXY_MODE`
    /// and `no_proxy`).
    pub proxy: Option<String>,
    /// Disable certificate verification (self-signed / internal gateways).
    /// Diagnostic escape hatch ONLY -- it makes TLS MITM-able.
    pub skip_tls_verify: bool,
    /// `false` ⇒ `redirect(Policy::none())`, i.e. the caller walks redirects itself.
    /// REQUIRED by `web_fetch`: it re-runs the SSRF host/IP checks on every hop, so
    /// letting reqwest follow automatically would let a 302 rebind to `127.0.0.1`
    /// after the start URL passed.
    pub follow_redirects: bool,
    /// Layer the OS native store + `SSL_CERT_FILE` on top of the webpki base.
    /// `false` is the #514 backstop path (base roots only).
    pub trust_os_roots: bool,
    /// Optional TLS ceiling (e.g. `TLS_1_2` for endpoints behind a TLS-1.3-hostile
    /// gateway). `None` ⇒ leave reqwest's default, as further constrained by the
    /// process-wide `RUSTCODE_TLS_MAX` policy.
    pub max_tls_version: Option<reqwest::tls::Version>,
}

impl Default for HttpClientSpec {
    fn default() -> Self {
        Self {
            connect_timeout: DEFAULT_CONNECT_TIMEOUT,
            request_timeout: Some(DEFAULT_REQUEST_TIMEOUT),
            pool_idle_timeout: POOL_IDLE_TIMEOUT,
            user_agent: None,
            proxy: None,
            skip_tls_verify: false,
            follow_redirects: true,
            trust_os_roots: true,
            max_tls_version: None,
        }
    }
}

impl HttpClientSpec {
    /// The default spec with a browser User-Agent -- what the web-surfacing tools
    /// (`web_fetch` / `web_search`) want, since many sites 403 a bot UA.
    #[must_use]
    pub fn browser() -> Self {
        Self {
            user_agent: Some(BROWSER_UA.to_string()),
            ..Self::default()
        }
    }

    /// Set the User-Agent (chainable).
    #[must_use]
    pub fn with_user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    /// Disable automatic redirect following (chainable). See
    /// [`HttpClientSpec::follow_redirects`] for why `web_fetch` needs this.
    #[must_use]
    pub fn with_no_redirects(mut self) -> Self {
        self.follow_redirects = false;
        self
    }

    /// Override the whole-request timeout (chainable). `None` drops the
    /// client-level timeout.
    #[must_use]
    pub fn with_request_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// Override the connect timeout (chainable).
    #[must_use]
    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Override the proxy (chainable). Empty string is treated as "no explicit
    /// proxy" (fall back to the process policy).
    #[must_use]
    pub fn with_proxy(mut self, proxy: Option<String>) -> Self {
        self.proxy = proxy.filter(|p| !p.trim().is_empty());
        self
    }

    /// Disable certificate verification (chainable). Diagnostic escape hatch.
    #[must_use]
    pub fn with_skip_tls_verify(mut self, skip: bool) -> Self {
        self.skip_tls_verify = skip;
        self
    }

    /// Cap the TLS version (chainable) -- the endpoint-aware equivalent of the
    /// process-wide `RUSTCODE_TLS_MAX`.
    #[must_use]
    pub fn with_max_tls_version(mut self, version: Option<reqwest::tls::Version>) -> Self {
        self.max_tls_version = version;
        self
    }
}

/// The default spec with a browser UA. Convenience free function so call sites read
/// `build_http_client(&browser_spec())` instead of naming the type twice.
#[must_use]
pub fn browser_spec() -> HttpClientSpec {
    HttpClientSpec::browser()
}

/// Build a `reqwest::Client` from `spec`.
///
/// Applies, in order: proxy policy -> timeouts -> UA -> TLS ceiling -> trust roots ->
/// redirect policy -> `skip_tls_verify`, then builds. If the OS-rooted build fails
/// (a poisoned `SSL_CERT_FILE`, a legacy root rustls rejects), it retries ONCE on the
/// infallible webpki base and logs a warning -- see the module docs (issue #514).
pub fn build_http_client(spec: &HttpClientSpec) -> Result<reqwest::Client, EgressError> {
    build_with(spec, |b| b)
}

/// Build a client whose DNS resolution for `host` is PINNED to `addrs`.
///
/// `web_fetch` needs this: it validates the resolved addresses itself (SSRF), then
/// pins them so reqwest performs no second lookup -- that is what closes the
/// DNS-rebinding TOCTOU window. Pinning is the ONE reason to touch the builder
/// after [`spec_builder`], and it must still go through the #514 backstop below,
/// so it lives here rather than at the call site.
///
/// An empty `addrs` (a literal-IP host, where there is nothing to pin) is
/// equivalent to [`build_http_client`].
pub fn build_pinned_http_client(
    spec: &HttpClientSpec,
    host: &str,
    addrs: &[std::net::SocketAddr],
) -> Result<reqwest::Client, EgressError> {
    if addrs.is_empty() {
        return build_with(spec, |b| b);
    }
    build_with(spec, |b| b.resolve_to_addrs(host, addrs))
}

/// Shared build loop: assemble the builder for `spec`, let `decorate` apply the
/// caller's extras, then build -- retrying ONCE on the infallible webpki base if the
/// OS-rooted build fails.
fn build_with<F>(spec: &HttpClientSpec, decorate: F) -> Result<reqwest::Client, EgressError>
where
    F: Fn(reqwest::ClientBuilder) -> reqwest::ClientBuilder,
{
    match spec_builder(spec, spec.trust_os_roots)
        .map(&decorate)
        .and_then(build)
    {
        Ok(client) => Ok(client),
        Err(first) if spec.trust_os_roots => {
            // BACKSTOP (issue #514): rather than a total outage, retry ONCE with the
            // INFALLIBLE webpki base only. Public-CA endpoints still work; only a
            // corporate MITM root is lost, and a request-time TLS error is far better
            // than a client that never builds.
            tracing::warn!(
                "http client build failed with the OS/SSL_CERT_FILE trust roots ({}); \
                 retrying with the webpki base only -- a custom/corporate root may be ignored (issue #514)",
                first
            );
            spec_builder(spec, false).map(&decorate).and_then(build)
        }
        Err(e) => Err(e),
    }
}

/// Build the [`reqwest::ClientBuilder`] for `spec`. Exposed so the rare call site that
/// must pin DNS (SSRF `resolve_to_addrs`) can still do so on top of the SHARED policy
/// instead of re-deriving it -- see `tools::web_fetch::build_client`.
pub fn spec_builder(
    spec: &HttpClientSpec,
    trust_os_roots: bool,
) -> Result<reqwest::ClientBuilder, EgressError> {
    let mut builder = match spec.proxy.as_deref().filter(|p| !p.is_empty()) {
        // An explicit proxy overrides the process-wide env policy.
        Some(proxy_url) => {
            let p = reqwest::Proxy::all(proxy_url).map_err(|e| {
                EgressError::Config(format!("invalid proxy url `{proxy_url}`: {e}"))
            })?;
            reqwest::Client::builder().proxy(p)
        }
        None => crate::proxy::apply_async_proxy_policy(reqwest::Client::builder()),
    }
    .connect_timeout(spec.connect_timeout)
    // Drop idle keep-alive connections before the server/LB does, so we don't reuse
    // a half-closed socket (the "error sending request" / ConnectionReset class).
    .pool_idle_timeout(spec.pool_idle_timeout)
    .user_agent(spec.user_agent.as_deref().unwrap_or(DEFAULT_USER_AGENT));

    if let Some(timeout) = spec.request_timeout {
        builder = builder.timeout(timeout);
    }
    if let Some(version) = spec.max_tls_version {
        builder = builder.max_tls_version(version);
    }
    // TLS trust (issue #514): webpki base roots are always present so `.build()`
    // never hard-fails on certs; the OS native store (corporate MITM CAs) and
    // SSL_CERT_FILE are layered on top, additively and best-effort.
    // Skip the rustls root-layering on Windows: the native-tls (SChannel) default
    // backend trusts the Windows system store natively, and re-feeding certs through
    // native-tls's parser risks rejecting one rustls accepted. A runtime `cfg!`
    // (not `#[cfg]`) keeps the fn referenced -- and therefore dead-code-free -- on
    // every platform while compiling the call out on Windows.
    if trust_os_roots && !cfg!(target_os = "windows") {
        builder = add_trusted_roots(builder);
    }
    if spec.skip_tls_verify {
        builder = builder.danger_accept_invalid_certs(true);
    }
    if !spec.follow_redirects {
        builder = builder.redirect(reqwest::redirect::Policy::none());
    }
    Ok(builder)
}

/// `.build()` with a USEFUL message: reqwest's builder-error `Display` is a bare
/// "builder error" -- the real reason (bad cert, invalid header, ...) lives in its
/// `source()` chain, so walk it or the message is useless (issue #514).
fn build(builder: reqwest::ClientBuilder) -> Result<reqwest::Client, EgressError> {
    builder.build().map_err(|e| {
        let mut msg = e.to_string();
        let mut source = e.source();
        while let Some(s) = source {
            msg.push_str(&format!(": {s}"));
            source = s.source();
        }
        EgressError::Build(msg)
    })
}

/// Add the OS native root store and `SSL_CERT_FILE` (if set) to the builder's
/// trusted roots, ON TOP of the built-in webpki roots. Best-effort: unparseable
/// certs, an unreadable/malformed `SSL_CERT_FILE`, or native-store load errors are
/// warned and skipped -- NEVER fatal (the webpki base guarantees a working client).
/// Codex-style graceful `load_native_certs`. See issue #514.
fn add_trusted_roots(mut builder: reqwest::ClientBuilder) -> reqwest::ClientBuilder {
    // 1) OS native roots (corporate MITM CAs live here).
    let native = rustls_native_certs::load_native_certs();
    if !native.errors.is_empty() {
        tracing::warn!(
            "loaded OS native roots with {} error(s); using the {} that parsed (issue #514)",
            native.errors.len(),
            native.certs.len()
        );
    }
    // `reqwest::Certificate::from_der` does NOT validate under rustls -- it just stores
    // the bytes and defers validation to `rustls::RootCertStore::add` INSIDE `.build()`,
    // which aborts the WHOLE client on the first cert rustls rejects (a legacy OS root
    // without X509v3 extensions is enough). Pre-filter each cert through the same rustls
    // parser so one bad OS root can't take every client down. See issue #514.
    let mut rejected = 0usize;
    for der in native.certs {
        if rustls::RootCertStore::empty().add(der.clone()).is_err() {
            rejected += 1;
            continue;
        }
        if let Ok(cert) = reqwest::Certificate::from_der(der.as_ref()) {
            builder = builder.add_root_certificate(cert);
        }
    }
    if rejected > 0 {
        tracing::warn!(
            "skipped {rejected} OS root cert(s) rustls rejected; they would have aborted the whole client build (issue #514)"
        );
    }

    // 2) SSL_CERT_FILE override/extra (empty string = unset). Loaded explicitly for
    //    cross-platform certainty. reqwest's `Certificate` is validated only at
    //    `.build()`; unlike the native loop above we do NOT pre-probe these (no DER in
    //    hand from `from_pem_bundle`), so a MALFORMED SSL_CERT_FILE still poisons
    //    `.build()` -- but the [`build_http_client`] BACKSTOP catches that and rebuilds
    //    on the webpki base (never a panic; the file is then ignored with a warning
    //    rather than killing the client). See #514.
    let Some(path) = std::env::var_os("SSL_CERT_FILE").filter(|p| !p.is_empty()) else {
        return builder;
    };
    let Ok(pem) = std::fs::read(&path) else {
        tracing::warn!("SSL_CERT_FILE={path:?} could not be read; ignoring (issue #514)");
        return builder;
    };
    match reqwest::Certificate::from_pem_bundle(&pem) {
        Ok(certs) => {
            let count = certs.len();
            for c in certs {
                builder = builder.add_root_certificate(c);
            }
            tracing::info!("Loaded {count} TLS root(s) from SSL_CERT_FILE={path:?} (issue #514)");
        }
        Err(e) => {
            tracing::warn!(
                "SSL_CERT_FILE={path:?} is not a valid PEM bundle: {e}; ignoring (issue #514)"
            );
        }
    }
    builder
}

#[cfg(test)]
mod tests {
    use super::*;

    // Env-mutating (SSL_CERT_FILE) -> must not race the sibling tests in this binary.
    use serial_test::serial;

    // Ported from `provider::openai_compat`'s test of the same name: the shared
    // factory must build on the webpki base roots with no SSL_CERT_FILE in the way.
    #[test]
    #[serial(ssl_cert_file_env)]
    fn build_http_client_builds_with_webpki_base_no_ssl_cert_file() {
        std::env::remove_var("SSL_CERT_FILE");
        let spec = HttpClientSpec::default();
        assert!(
            build_http_client(&spec).is_ok(),
            "the default spec must build on the webpki base roots"
        );
    }

    // Ported from `provider::openai_compat`'s test of the same name: root loading
    // happens BEFORE the danger_accept path, so it must not break that build.
    #[test]
    #[serial(ssl_cert_file_env)]
    fn build_http_client_skip_tls_verify_still_builds() {
        std::env::remove_var("SSL_CERT_FILE");
        let spec = HttpClientSpec {
            skip_tls_verify: true,
            ..HttpClientSpec::default()
        };
        assert!(
            build_http_client(&spec).is_ok(),
            "skip_tls_verify must still build"
        );
    }

    // The #514 backstop, now asserted against the SHARED factory: a malformed
    // SSL_CERT_FILE poisons the OS-rooted build, and we must still come back on the
    // webpki base instead of leaving every caller dead on startup.
    #[test]
    #[serial(ssl_cert_file_env)]
    fn build_http_client_recovers_from_malformed_ssl_cert_file_via_backstop() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cert_path = tmp.path().join("roots.pem");
        std::fs::write(
            &cert_path,
            "-----BEGIN CERTIFICATE-----\nZm9v\n-----END CERTIFICATE-----\n",
        )
        .expect("write poisoned bundle");
        std::env::set_var("SSL_CERT_FILE", &cert_path);
        let built = build_http_client(&HttpClientSpec::default());
        std::env::remove_var("SSL_CERT_FILE");
        assert!(
            built.is_ok(),
            "a malformed SSL_CERT_FILE must fall back to the webpki base, not abort the client"
        );
    }

    #[test]
    fn default_spec_carries_the_shared_policy() {
        let spec = HttpClientSpec::default();
        assert_eq!(spec.connect_timeout, Duration::from_secs(10));
        assert_eq!(spec.request_timeout, Some(Duration::from_secs(120)));
        // Same value `provider::retry` re-exports -- the two must not drift.
        assert_eq!(spec.pool_idle_timeout, POOL_IDLE_TIMEOUT);
        assert!(
            spec.trust_os_roots,
            "OS roots are layered by default (#514)"
        );
        assert!(spec.follow_redirects, "redirects are followed by default");
        assert!(!spec.skip_tls_verify, "verification stays ON by default");
        assert_eq!(spec.proxy, None, "unset ⇒ the process proxy policy applies");
    }

    #[test]
    fn browser_spec_keeps_the_existing_browser_ua() {
        let spec = browser_spec();
        assert_eq!(spec.user_agent.as_deref(), Some(BROWSER_UA));
        // Only the UA differs from the default policy -- the rest is shared.
        let mut same = spec.clone();
        same.user_agent = None;
        assert_eq!(same, HttpClientSpec::default());
    }

    #[test]
    fn no_redirects_and_empty_proxy_are_normalised() {
        let spec = browser_spec()
            .with_no_redirects()
            .with_proxy(Some("   ".to_string()));
        assert!(!spec.follow_redirects);
        // An all-whitespace proxy is "unset", so the process policy applies.
        assert_eq!(spec.proxy, None);
    }

    #[test]
    fn an_invalid_proxy_url_is_a_config_error_not_a_panic() {
        let spec = HttpClientSpec::default().with_proxy(Some("not a url".to_string()));
        let err = build_http_client(&spec).expect_err("must not silently build");
        assert!(
            matches!(err, EgressError::Config(_)),
            "expected a Config error, got {err:?}"
        );
    }
}
