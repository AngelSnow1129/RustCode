//! ONE home for outbound HTTP — client construction, external-service config, and
//! error mapping.
//!
//! # Why this module exists
//!
//! Before it, every egress call site in this crate hand-rolled its own
//! `reqwest::Client::builder()`. Proxy handling, TLS trust roots, timeouts, pool
//! policy and User-Agent silently diverged between them, and one of them
//! (`mcp::transport_http`) swallowed a build failure with
//! `unwrap_or_else(|_| reqwest::Client::new())` — silently dropping the proxy and
//! the whole trust-root policy. This module is the single place that policy lives;
//! call sites only describe WHAT they need.
//!
//! # What lives here
//!
//! - [`client`] — [`build_http_client`](client::build_http_client) plus
//!   [`HttpClientSpec`]: TLS trust-root layering (issue #514), proxy policy,
//!   timeouts, UA, redirect policy. `provider` adapters, `web_fetch`, `web_search`,
//!   `atomgit` and `mcp` all go through it.
//! - [`config`] — [`ExternalServiceConfig`]: the non-LLM flavour of an outbound
//!   endpoint (base url + credential + custom headers), deliberately shaped like a
//!   provider config so the two are configured and validated the same way.
//! - [`error`] — [`EgressError`] with the `brief()` / `detail()` pair: a ONE-LINE
//!   summary for the UI and a bounded diagnostic text for logs and the model.
//!   Response bodies are never replayed whole (see
//!   [`BODY_EXCERPT_BYTES`](error::BODY_EXCERPT_BYTES)).
//!
//! # Feature gating
//!
//! `egress` is never enabled directly. It is pulled in by each feature that
//! performs outbound HTTP (`provider`, `web`, `atomgit`, `mcp`), so a lean build
//! with none of them compiles no egress code and no HTTP stack.

pub mod client;
pub mod config;
pub mod error;

pub use client::{
    browser_spec, build_http_client, HttpClientSpec, BROWSER_UA, DEFAULT_CONNECT_TIMEOUT,
    DEFAULT_REQUEST_TIMEOUT, DEFAULT_USER_AGENT, POOL_IDLE_TIMEOUT,
};
pub use config::{ExternalServiceConfig, SecretString};
pub use error::{EgressError, BODY_EXCERPT_BYTES};
