//! Test-only isolation of `RUSTCODE_HOME`.
//!
//! rustcode persists sessions / config / memory under `RUSTCODE_HOME` (default
//! `~/.rustcode`). Tests that construct a `SessionManager`, run the agent, or
//! otherwise persist without setting `RUSTCODE_HOME` write into the developer's
//! REAL home -- a full `cargo test` run leaves dozens of junk `sessions/<hash>/`
//! buckets (working dirs that are throwaway `tempfile` paths).
//!
//! [`isolate_home`] redirects `RUSTCODE_HOME` to a throwaway temp dir the FIRST
//! time it runs, replacing any value inherited from the developer's shell. It's
//! idempotent (guarded by a `Once`), so calling it from a `#[ctor]` in each test
//! binary sets one stable value before libtest spawns any thread -- no `set_var`
//! race (unlike per-test `set_var`, which races under the parallel harness).
//! Tests that need a dedicated home may still replace the isolated value inside
//! their test, using the crate's process-global environment lock where required.
//!
//! Gated behind the `test-support` cargo feature so the env-mutating helper never
//! enters a normal (non-test) build. Consuming crates enable it via a
//! dev-dependency and call it from a `#[ctor]` in their own `#[cfg(test)]` module
//! (and every `tests/*.rs` integration binary):
//!
//! ```ignore
//! // Cargo.toml
//! [dev-dependencies]
//! rustcode-kernel = { path = "../rustcode-kernel", features = ["test-support"] }
//! ctor = "0.2"
//! ```
//! ```ignore
//! #[cfg(test)]
//! #[ctor::ctor]
//! fn _isolate_rustcode_home() {
//!     rustcode_kernel::test_support::isolate_home();
//! }
//! ```
//!
//! Putting the `#[ctor]` in the CONSUMING crate (and referencing this fn) is what
//! forces the linker to keep it -- a bare `use ... as _` on a ctor-only crate gets
//! dropped and never fires.

use std::path::PathBuf;
use std::sync::Once;

static INIT: Once = Once::new();
static SUBDIR_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Redirect `RUSTCODE_HOME` to a per-process temp dir. Any inherited value is
/// deliberately replaced: a test process must never interpret a developer's
/// real configured data directory as disposable fixture state.
///
/// Idempotent and race-free (runs once). Call from a `#[ctor]` so it lands before
/// any test.
pub fn isolate_home() {
    INIT.call_once(|| {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock must be after Unix epoch")
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("rustcode-test-home-{}-{nonce}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap_or_else(|error| {
            panic!(
                "failed to create isolated RUSTCODE_HOME at {}: {error}",
                dir.display()
            )
        });
        std::env::set_var("RUSTCODE_HOME", &dir);
    });
}

/// A DEDICATED test home that stays valid for the WHOLE process.
///
/// `std::env::set_var` is process-global: a test that points `RUSTCODE_HOME` at
/// its own `tempfile::TempDir` hands every OTHER concurrently running test a
/// path that is deleted the instant that `TempDir` drops. The next session
/// write from an unrelated test then fails with a spurious
/// `NotFound("<deleted>/sessions/<bucket>/<id>.snapshot")` -- a failure that only
/// reproduces under the parallel harness, so it presents as a flake in whichever
/// test happens to lose the race.
///
/// Directories handed out here live under the process-wide isolated home and are
/// NEVER removed, so a value that leaks past its own test still points at a
/// usable directory. `label` aids debugging only; uniqueness comes from a counter.
pub fn isolate_home_subdir(label: &str) -> PathBuf {
    isolate_home();
    let base = std::env::var_os("RUSTCODE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let seq = SUBDIR_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = base.join(format!("{label}-{}-{seq}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap_or_else(|error| {
        panic!(
            "failed to create isolated test home subdir {}: {error}",
            dir.display()
        )
    });
    dir
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inherited_rustcode_home_is_never_reused_as_test_storage() {
        let inherited = std::env::temp_dir().join(format!(
            "rustcode-real-home-sentinel-{}",
            std::process::id()
        ));
        std::env::set_var("RUSTCODE_HOME", &inherited);

        isolate_home();

        let isolated = std::env::var_os("RUSTCODE_HOME").unwrap();
        assert_ne!(std::path::PathBuf::from(&isolated), inherited);
        assert!(std::path::Path::new(&isolated).is_dir());
    }
}
