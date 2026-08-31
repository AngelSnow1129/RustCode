// crates/rustcode-core/src/coding_plan/mod.rs
//
// CodingPlan integration: claim the free plan, fetch the eligible model
// list, set up matching provider entries, and pull plan status. Drives
// both `rustcode codingplan` (CLI subcommand) and `/codingplan` (TUI
// slash command); they share the orchestrator in `setup` and differ only
// in how the returned `SetupReport` is rendered.
//
// All three REST endpoints are addressed through the platform base URL
// (`codingplan_api_base()` -- empty by default in this platform-neutral
// build; an operator opts in via `RUSTCODE_CODINGPLAN_API_BASE`). The OAuth
// flow runs against the separately configured platform server; the same
// token authenticates both. Every request honours the shared UA
// (`RUSTCODE_USER_AGENT`) so the gateway sees a consistent
// `rustcode/<ver>` identifier.

// Network gateway client and claim/login/sync orchestration. These pull in
// reqwest and encode the managed-platform REST flow; they are gated behind the
// off-by-default `client` feature so the platform-neutral build links only the
// pure data/usage/marker modules below.
#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "client")]
pub mod setup;
pub mod sync_marker;
pub mod types;
pub mod usage;

// Gateway request-signing (is_codingplan_gateway / RequestSigner / SignInput / ...) lives in
// `rustcode_auth::gateway_crypto` now -- this crate is the CodingPlan REST/usage/setup business
// layer only. Consumers that need signing import from `rustcode_auth::gateway_crypto` directly.

#[cfg(feature = "client")]
pub use client::{api_base_url, is_auth_expired, AuthExpired, Client};
#[cfg(feature = "client")]
pub use setup::{merge_successful_config, run, DefaultModelPolicy, SetupReport, StepResult};
pub use sync_marker::{read_last_sync, write_last_sync_now};
pub use types::{ClaimResponse, ModelEntry, PlanInfo, PlanType, StatusResponse, UsageInfo};
pub use usage::format_duration_secs;
