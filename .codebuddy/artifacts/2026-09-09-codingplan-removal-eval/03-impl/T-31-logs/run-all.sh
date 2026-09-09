#!/bin/bash
# T-31 全量验证脚本（只读，不改生产代码）
cd /workspace/RustCode
LOG=/workspace/RustCode/.codebuddy/artifacts/2026-09-09-codingplan-removal-eval/03-impl/T-31-logs
export CARGO_TERM_COLOR=never
{
echo "=== T-31 START $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="
echo "=== HEAD: $(git rev-parse HEAD) branch: $(git rev-parse --abbrev-ref HEAD) ==="

echo ""
echo "########## CMD-1: cargo fmt --check ##########"
cargo fmt --check > "$LOG/01-fmt.log" 2>&1
echo "exit=$?"

echo ""
echo "########## CMD-2: cargo build ##########"
cargo build > "$LOG/02-build.log" 2>&1
echo "exit=$?"

echo ""
echo "########## CMD-3: cargo build --workspace ##########"
cargo build --workspace > "$LOG/03-build-workspace.log" 2>&1
echo "exit=$?"

echo ""
echo "########## CMD-4: cargo clippy --workspace --all-targets ##########"
cargo clippy --workspace --all-targets > "$LOG/04-clippy.log" 2>&1
echo "exit=$?"

echo ""
echo "########## CMD-5: cargo test -j 1 --workspace --no-fail-fast ##########"
cargo test -j 1 --workspace --no-fail-fast > "$LOG/05-test.log" 2>&1
echo "exit=$?"

echo ""
echo "########## CMD-6: python3 scripts/check-zh-docs.py gate ##########"
python3 scripts/check-zh-docs.py gate > "$LOG/06-zhdocs.log" 2>&1
echo "exit=$?"

echo ""
echo "########## CMD-7..10: residual greps ##########"
grep -rniI "codingplan" crates/ --include=*.rs --include=*.toml > "$LOG/07-crates-residual.txt" 2>&1
echo "CMD7 exit_count=$(wc -l < "$LOG/07-crates-residual.txt")"
grep -rniI "codingplan" extensions/ webui/src > "$LOG/08-ext-webui-residual.txt" 2>&1
echo "CMD8 exit_count=$(wc -l < "$LOG/08-ext-webui-residual.txt")"
grep -rniI "codingplan" docs/ > "$LOG/09-docs-residual.txt" 2>&1
echo "CMD9 exit_count=$(wc -l < "$LOG/09-docs-residual.txt")"
grep -n "rustcode-codingplan" Cargo.lock > "$LOG/10-cargo-lock.txt" 2>&1
echo "CMD10 exit=$? count=$(wc -l < "$LOG/10-cargo-lock.txt")"

echo ""
echo "=== T-31 END $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="
} > "$LOG/summary.log" 2>&1
echo "ALLDONE" >> "$LOG/summary.log"
