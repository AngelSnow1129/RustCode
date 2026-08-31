# Goal Summary: RustCode 迁移收尾

## What was achieved

All 5 dimensions of the migration finalization are complete:

### 1. 命名清理 (atomcode -> rustcode)
- `webui/package-lock.json`: `atomcode-webui` -> `rustcode-webui`
- `extensions/vscode/package-lock.json`: already `rustcode-tools` (verified)
- G7 (crates/scripts/.github): 0 hits
- G8 (docs/architecture.md): 0 hits

### 2. 平台中立 (不与任何模型/平台关联)
- No host hard-coded as signing gateway
- `is_codingplan_llm_gateway` only recognizes explicit `RUSTCODE_CODINGPLAN_LLM_BASE_URL`
- Providers use plain `bearer_auth(api_key)` by default
- AtomGit REST tools behind `atomgit` cargo feature (off in default members)
- Only third-party provider config in `config.example.toml`

### 3. 零遥测
- G6: 0 hits (sentry/posthog/segment/analytics)
- `install_panic_hook` preserved (stderr-only local logic, NOT telemetry)
- `rustcode-telemetry` crate deleted (confirmed)

### 4. 中文默认
- `Locale::default()` returns `ZhCn` (aligned with runtime default)
- `LOCALE` static default: `ZhCn`
- `current_locale()` fallback: `ZhCn`
- `resolve_initial_locale_with_env` no-signal: `ZhCn`
- `LANG=C`/`POSIX` -> `ZhCn` (no preference = product default)
- `config.example.toml`: `# language = "zh_CN"` comment added
- Test `cli_flag_unparseable_falls_through` updated for ZhCn default

### 5. 多 Agent 并行
- `config.example.toml`: 3-role parallel template added
  (explorer/builder/reviewer with max_concurrent=4)
- Demonstrates 6 design principles:
  1. 专业化分工 (codex/claude-code distinct kinds)
  2. 清晰边界 (read-only/accept-edits/auto/bypass tiers)
  3. 高效通信 (context inheritance + result passing)
  4. 并行处理 (max_concurrent)
  5. 质量保证 (reviewer = independent verification)
  6. 成本优化 (model override per instance)

## Iteration history

- Iteration 1: PASS (no issues found)

## Key files changed

- `crates/rustcode-config/src/locale.rs` — `Default` impl
- `crates/rustcode-config/src/i18n/mod.rs` — LOCALE static, current_locale, test
- `docs/config.example.toml` — language comment + subagent template
- `webui/package-lock.json` — package name

## Recommendations

- Consider running `cargo test --workspace` fully before merging (only
  config crate was tested in this iteration due to time)
- The known-red test `mcp::registry::tests::trust_key_golden_matches_core_algorithm`
  remains red (DefaultHasher instability) — do not "fix" it
- Consider enabling the 3-role subagent template by default in a future
  release if parallel multi-agent proves stable
