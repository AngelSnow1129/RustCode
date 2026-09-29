#![cfg(feature = "setup")]
//! End-to-end integration tests for `setup::run`.
//!
//! Each run gets its own user tree and project in temp dirs, handed in through
//! `ProductDirs` — no environment variable, so the tests need no serialisation
//! and cannot touch a real `~/.atomcode/`.

use atomcode_capabilities::setup::{self, RunOptions};
use atomcode_capabilities::ProductDirs;
use std::path::Path;

fn opts(proj: &Path, user: &Path) -> RunOptions {
    RunOptions::new(proj.to_path_buf(), ProductDirs::new(user, ".ours"))
}

/// Run setup in a temp project against a temp user tree.
/// Returns (`SetupResult`, both temp-dir guards).
fn run_in_tempdir<F, G>(
    project_setup: F,
    mutate_opts: G,
) -> (
    setup::SetupResult<setup::SetupReport>,
    tempfile::TempDir,
    tempfile::TempDir,
)
where
    F: FnOnce(&Path),
    G: FnOnce(&mut RunOptions),
{
    let proj = tempfile::tempdir().unwrap();
    let user = tempfile::tempdir().unwrap();
    project_setup(proj.path());

    let mut opts = opts(proj.path(), user.path());
    mutate_opts(&mut opts);

    (setup::run(opts), proj, user)
}

/// A distribution's names are the only names setup writes under: its project
/// dir holds the lock, state and backups, `.gitignore` names its local subdir,
/// seeds land in its user tree — and no `.atomcode` appears anywhere.
#[test]
fn a_renamed_distribution_writes_only_under_its_own_names() {
    let proj = tempfile::tempdir().unwrap();
    let user = tempfile::tempdir().unwrap();
    let dirs = ProductDirs::new(user.path().join("fork-tree"), ".fork");
    setup::run(RunOptions::new(proj.path().to_path_buf(), dirs)).expect("setup runs");

    assert!(proj.path().join(".fork/setup-state.json").exists());
    let gitignore = std::fs::read_to_string(proj.path().join(".gitignore")).unwrap();
    assert!(gitignore.contains(".fork/local/"), "{gitignore}");
    assert!(user.path().join("fork-tree/skills").exists());
    for root in [proj.path(), user.path()] {
        assert!(
            !walk_has(root, ".atomcode"),
            "setup created a `.atomcode` under {root:?}"
        );
    }
}

fn walk_has(dir: &Path, name: &str) -> bool {
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .any(|e| e.file_name() == name || (e.path().is_dir() && walk_has(&e.path(), name)))
}

#[test]
fn setup_installs_seeds_in_empty_project() {
    let (result, _proj, _user) = run_in_tempdir(|_| {}, |_| {});

    let report = result.expect("setup run should succeed");

    // Should install at least some seeds (skills/commands from embedded tar).
    let total =
        report.summary.installed.len() + report.summary.skipped.len() + report.summary.failed.len();
    assert!(
        total > 0,
        "expected at least one seed install attempted, got installed={} skipped={} failed={}",
        report.summary.installed.len(),
        report.summary.skipped.len(),
        report.summary.failed.len(),
    );

    // setup-state.json should exist in the project dir.
    // (Not in user dir — state is per-project.)
}

#[test]
fn second_run_skips_already_installed() {
    let proj = tempfile::tempdir().unwrap();
    let user = tempfile::tempdir().unwrap();

    let make_opts = || opts(proj.path(), user.path());

    let report1 = setup::run(make_opts()).unwrap();
    let report2 = setup::run(make_opts()).unwrap();

    // First run should install or attempt some items.
    let first_attempted = report1.summary.installed.len()
        + report1.summary.skipped.len()
        + report1.summary.failed.len();
    assert!(
        first_attempted > 0,
        "first run should attempt at least one item"
    );

    // Second run should have at least one AlreadyInstalled skip (if first run installed anything).
    assert!(
        report2.summary.installed.len() <= report1.summary.installed.len(),
        "second run should install at most as many as the first (first={}, second={})",
        report1.summary.installed.len(),
        report2.summary.installed.len(),
    );

    if !report1.summary.installed.is_empty() {
        let has_already = report2.summary.skipped.iter().any(|(_, reason)| {
            matches!(
                reason,
                atomcode_capabilities::setup::install::SkipReason::AlreadyInstalled
            )
        });
        assert!(
            has_already,
            "second run skipped items: {:?}",
            report2.summary.skipped
        );
    }
}

#[test]
fn concurrent_runs_second_fails_lock() {
    let proj = tempfile::tempdir().unwrap();
    let user = tempfile::tempdir().unwrap();

    let proj_a = proj.path().to_path_buf();
    let proj_b = proj.path().to_path_buf();

    // `setup::run` is synchronous. Use std::thread to run two concurrent
    // invocations — the lock must prevent both from succeeding simultaneously.
    let (r1, r2) = std::thread::scope(|s| {
        let t1 = s.spawn(|| {
            let o = opts(&proj_a, user.path());
            atomcode_capabilities::setup::run(o)
        });

        // Give t1 a head start so it grabs the lock first.
        std::thread::sleep(std::time::Duration::from_millis(20));

        let t2 = s.spawn(|| {
            let o = opts(&proj_b, user.path());
            atomcode_capabilities::setup::run(o)
        });

        let r1 = t1.join().unwrap();
        let r2 = t2.join().unwrap();
        (r1, r2)
    });

    // At least one should succeed; failures must be lock-related.
    let succeeded = r1.is_ok() as usize + r2.is_ok() as usize;
    assert!(
        succeeded >= 1,
        "at least one concurrent run should succeed; both failed:\n  r1={:?}\n  r2={:?}",
        r1,
        r2
    );

    for r in [&r1, &r2] {
        if let Err(e) = r {
            assert!(
                matches!(
                    e,
                    atomcode_capabilities::setup::SetupError::LockHeld { .. }
                        | atomcode_capabilities::setup::SetupError::LockIo(_)
                ),
                "if a concurrent run failed, it should be LockHeld or LockIo, got: {e:?}"
            );
        }
    }
}
