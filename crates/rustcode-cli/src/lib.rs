//! Library surface for the `rustcode` binary.
//!
//! Exists so integration tests (e.g. `tests/script_parity.rs`) and the binary
//! share testable modules. The bulk of the CLI still lives in `main.rs`; only
//! modules that need to be reachable from `tests/` belong here.

// Redirect RUSTCODE_HOME to a temp dir before this lib crate's tests run, so they
// don't pollute the real ~/.rustcode (mirrors the bin's ctor in main.rs).
#[cfg(test)]
#[ctor::ctor]
fn _isolate_rustcode_home() {
    rustcode_kernel::test_support::isolate_home();
}

#[cfg(unix)]
pub mod askpass;
pub mod uninstall;

/// ACP (Agent Client Protocol) stdio server — lets rustcode be driven by Zed /
/// multi-agent orchestrators over stdin/stdout. Wired up by the `rustcode acp`
/// subcommand in `main.rs`; the engine/dispatch/translate/permission internals
/// live here. Does not depend on `rustcode-core` (v2 stack only).
pub mod acp;
