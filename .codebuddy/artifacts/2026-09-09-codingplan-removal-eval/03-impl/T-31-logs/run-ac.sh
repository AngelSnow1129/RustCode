#!/bin/bash
cd /workspace/RustCode
LOG=/workspace/RustCode/.codebuddy/artifacts/2026-09-09-codingplan-removal-eval/03-impl/T-31-logs
export CARGO_TERM_COLOR=never
run() {
  local name="$1"; shift
  echo "########## $name ##########"
  echo "\$ $*"
  "$@" > "$LOG/ac-$name.out" 2>&1
  local rc=$?
  echo "exit=$rc"
  echo "--- output (tail 25) ---"
  tail -25 "$LOG/ac-$name.out"
  echo ""
}
{
echo "=== T-31 AC-BATCH START $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="
run ac4-1 cargo check -p rustcode-codingplan --all-targets
run ac4-2 cargo check -p rustcode-codingplan --features client --all-targets
run ac4-3 cargo check -p rustcode-tuix --features codingplan --all-targets
run ac4-4 cargo check -p rustcode-daemon --features codingplan --all-targets
run ac4-5 cargo check -p rustcode --features codingplan --all-targets
run ac4-6 cargo check -p rustcode --features codingplan-crypto --all-targets
run ac4-7 cargo check --workspace --all-targets
run ac13-metadata cargo metadata --format-version 1

echo "########## ac7-help ##########"
cargo build -p rustcode > "$LOG/ac-build-cli.out" 2>&1; echo "cargo build -p rustcode exit=$?"
for sh in bash zsh fish; do
  n=$(./target/debug/rustcode completion $sh 2>/dev/null | grep -ci codingplan || true)
  echo "completion $sh -> codingplan count=$n"
done
n=$(./target/debug/rustcode --help 2>/dev/null | grep -ci codingplan || true)
echo "--help -> codingplan count=$n"

echo "########## ac8-route ##########"
n=$(grep -rn "codingplan/setup" crates/rustcode-daemon/src | wc -l)
echo "grep -rn codingplan/setup crates/rustcode-daemon/src -> $n (expect 0)"

run ac7-completion-test cargo test -j 1 -p rustcode --lib -- completion
run ac10-uninstall-test cargo test -j 1 -p rustcode --lib -- uninstall
run ac15-config-lib cargo test -j 1 -p rustcode-config --lib
run ac9-newtests cargo test -j 1 -p rustcode-config --lib -- legacy_providers_project_one_account_per_provider declared_effort_levels_are_authoritative_without_builtin_fallback empty_declared_levels_mean_unrestricted legacy_provider_names_are_editable --exact --nocapture
run flaky-recheck cargo test -j 1 -p rustcode-capabilities --lib -- plugin::marketplace::tests::git_runs_rejects_present_but_failing_stub --exact --nocapture
run known-red cargo test -j 1 -p rustcode-capabilities --lib -- mcp::registry::tests::trust_key_golden_matches_core_algorithm --exact --nocapture
echo "=== T-31 AC-BATCH END $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="
} > "$LOG/summary-ac.log" 2>&1
echo "ACDONE" >> "$LOG/summary-ac.log"
