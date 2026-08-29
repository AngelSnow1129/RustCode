# PHASE-1: System Analysis & Refactor Design

> [INFO] Lead Orchestrator design document. GATEWAY deliverable.
> [INFO] No production source edit happens until this design is reviewed and confirmed.
> [INFO] Source root: `/workspace/gitCode/SecLab/RustCode`
> [INFO] Evidence baseline: branch `main`, HEAD `287bff70f9400c24f4afd8fcf4762fbab63d8efa`,
>        workspace version `5.0.9`, edition `2021`, 14 crates under `crates/`.

---

## 0. Engineering constraints (from the orchestration spec)

| ID | Constraint | Enforcement |
|----|-----------|-------------|
| C1 | No Unicode emoji. ASCII tags only: `[INFO]`, `[WARN]`, `[ERROR]`, `[SUCCESS]`, `[AGENT-A..D]`, `[+]`, `[-]`, `[*]`, `[CHECK]` | grep gate in AGENT-D |
| C2 | `tokio` async/await consistency; no blocking I/O in async paths | `cargo clippy -- -W clippy::block_in_async` advisory |
| C3 | No hardcoded API keys / tokens / internal endpoints in code, tests, or default config | secret-scan gate in AGENT-D |
| C4 | LLM client must support SSE streaming **and** non-streaming | OBJECTIVE-3 gap G1 |
| C5 | `thiserror` for module errors, `anyhow` for top-level context; no `.unwrap()`/`.expect()` on production paths | AGENT-C + AGENT-D |
| C6 | After telemetry removal, minimize shared-dep feature flags; verify `Cargo.lock` | AGENT-B + AGENT-D |

---

## 1. System analysis: what the repository actually is

### 1.1 Workspace shape

```text
rustcode/                              (workspace root, resolver = "2")
  crates/  (members = ["crates/*"], default-members = cli, daemon, telemetry, tuix)
    rustcode-kernel/        L0  neutral agent loop, LlmProvider trait, StreamEvent
    rustcode-capabilities/  L1  provider adapters, tools, MCP, skills, sessions, compaction
    rustcode-coding/        L2  CodingRuntime lifecycle, provider factory, assembly
    rustcode-review/        L2  review specialization
    rustcode-tuix/          L3  terminal UI
    rustcode-cli/           L3  TUI + headless entry point (bin `rustcode`)
    rustcode-daemon/        L3  HTTP/SSE/WebSocket transport + legacy session importer
    rustcode-config/        leaf  disk/TOML config + endpoints + distribution names
    rustcode-auth/          leaf  OAuth login + gateway request signer
    rustcode-codingplan/    leaf  CodingPlan REST business layer
    rustcode-codingplan-crypto/  closed-source overlay stub (optional feature)
    rustcode-clix/          standalone bin `rustcodex`
    rustcode-updater/       self-update mechanics
    rustcode-telemetry/     [DELETED by OBJECTIVE-2]
  scripts/ docker/ .github/workflows/  install, release, CI
  docs/ site/ extensions/   documentation, docs site, vscode/jetbrains extensions
```

### 1.2 Measured scope (grep-verified on HEAD)

| Metric | Value |
|--------|-------|
| `atomcode` string occurrences in `crates/**/*.rs` | **6735** |
| `atomcode_telemetry` references | **108** across **24 files** in **9 crates** |
| Distinct `ATOMCODE_*` env vars | **125** |
| `LlmProvider` impls (production) | 3 adapters + 1 telemetry decorator; ~40 test doubles |
| Existing provider mock integration tests | `capabilities/tests/{anthropic_mock,ollama_mock,http_mock,e2e}.rs` |

### 1.3 Runtime ownership (must not regress)

`CLI/TUI/daemon/headless/background/ACP/clix -> CodingRuntime -> kernel Agent`.
`CodingRuntime` is the single owner of live agent, provider, session binding, generation,
pending request, snapshot broker, controllers. Per `AGENTS.md`, drivers must not build a
second live-agent lifecycle owner, and `atomcode-core`/`atomcode-bridge` must not reappear.
The rename and the telemetry strip are mechanical; the LLM refactor touches a lifecycle seam
(provider construction inside `CodingRuntime` assembly), so AGENT-C must respect the
single-owner and fail-closed rules in `AGENTS.md`.

---

## 2. OBJECTIVE-1: Global rename mapping

### 2.0 Locked identity decision (D1)

Product identity: **`rustcode`**. This is a *proposal recorded as assumed*; it is a single
mapping-table change if overruled (see §2.7).

| Facet | Value |
|-------|-------|
| Cargo package prefix | `rustcode-*` |
| Main CLI binary | `rustcode` |
| Daemon binary / lib | `rustcode-daemon` / `rustcode_daemon` |
| Standalone CLI binary | `rustcodex` |
| Config/data directory | `~/.rustcode` |
| Env var prefix | `RUSTCODE_*` |
| Windows install dir | `RustCode` |
| Release asset prefix | `rustcode` |
| Update scratch prefix | `.rustcode` |
| Repository field | new fork URL (upstream noted in `docs/UPSTREAM_CREDITS.md`) |

### 2.1 Crate packages

| Old | New | Note |
|-----|-----|------|
| `atomcode-auth` | `rustcode-auth` | |
| `atomcode-capabilities` | `rustcode-capabilities` | |
| `atomcode-cli` | `rustcode-cli` | `[package] name` + `[[bin]] name` + `[lib] name` |
| `atomcode-clix` | `rustcode-clix` | |
| `atomcode-coding` | `rustcode-coding` | |
| `atomcode-codingplan` | `rustcode-codingplan` | |
| `atomcode-codingplan-crypto` | `rustcode-codingplan-crypto` | not in `default-members`; feature `rustcode/codingplan-crypto` |
| `atomcode-config` | `rustcode-config` | |
| `atomcode-daemon` | `rustcode-daemon` | |
| `atomcode-kernel` | `rustcode-kernel` | |
| `atomcode-review` | `rustcode-review` | |
| `atomcode-telemetry` | **[DELETED]** | OBJECTIVE-2 |
| `atomcode-tuix` | `rustcode-tuix` | |
| `atomcode-updater` | `rustcode-updater` | |

Each rename touches: that crate's `[package] name` / `[lib] name` / `[[bin]] name`, every
other crate's path dependencies, `Cargo.lock`, root `Cargo.toml` (`default-members`,
feature wiring), and install/release scripts.

### 2.2 Binaries

| Old | New | Defined at |
|-----|-----|-----------|
| `atomcode` | `rustcode` | `crates/rustcode-cli/Cargo.toml` |
| `atomcode-daemon` | `rustcode-daemon` | `crates/rustcode-daemon/Cargo.toml` |
| `atomcodex` | `rustcodex` | `crates/rustcode-clix/Cargo.toml` |
| `mcp-test-server` | unchanged | `crates/rustcode-capabilities/Cargo.toml` |

### 2.3 Directories, files, process names

Single highest-leverage file: **`crates/rustcode-config/src/distribution.rs`**.

| Constant | Old | New |
|----------|-----|-----|
| `HOME_ENV` | `ATOMCODE_HOME` | `RUSTCODE_HOME` |
| `HOME_DIR_NAME` | `.atomcode` | `.rustcode` |
| `PROCESS_NAMES` | `["atomcode","atomcode-daemon"]` | `["rustcode","rustcode-daemon"]` |
| `WINDOWS_INSTALL_DIR` | `AtomCode` | `RustCode` |
| `RELEASE_ASSET_PREFIX` | `atomcode` | `rustcode` |
| `UPDATE_TEMP_PREFIX` | `.atomcode` | `.rustcode` |

Derived, must move together (uninstaller scans what the updater creates):
`update_download_name()`, `update_rolling_name()`, `update_probe_name()`.

Also in scope: `docker/Dockerfile-*` (user `atomcode`, `/usr/local/bin/atomcode-daemon`,
`ENTRYPOINT`), `docker/build-multiarch.sh` artifact globs, `docker/README.md`,
`.github/workflows/build.yml` (`dist/atomcode-*`), `scripts/install.sh|ps1`,
`scripts/uninstall.sh|ps1`, `scripts/release*.sh`, `latest.json`.

### 2.4 Environment variables

Rule: `ATOMCODE_X` -> `RUSTCODE_X` for all 125 names. Exceptions:

| Var | Action |
|-----|--------|
| `ATOMCODE_TELEMETRY` | **DELETE** (OBJECTIVE-2) |
| `ATOMCODE_TELEMETRY_ENDPOINT` | **DELETE** (OBJECTIVE-2) |
| `ATOMCODE_TEST_EP_{WHOLE,ABSENT,URL,LIST,BOOL}` | rename mechanically (test-local) |

[DECISION D2] **No legacy fallback.** A clean fork does not read `ATOMCODE_*` after the
rename. `RUSTCODE_HOME` unset falls back to `~/.rustcode` only. This is a breaking change for
existing installs and must be called out in the release notes. (Alternative: dual-read with a
deprecation warning for one minor version — rejected for a fork, keeps the surface small and
avoids a second config-resolution path.)

Product-relevant subset (LLM / networking), for the config template:

```text
RUSTCODE_API_KEY  RUSTCODE_BASE_URL  RUSTCODE_MODEL
RUSTCODE_ANTHROPIC_{BASE_URL,KEY,MODEL}
RUSTCODE_OLLAMA_{BASE_URL,MODEL}
RUSTCODE_LIVE_{BASE_URL,KEY,MODEL}
RUSTCODE_PROXY_MODE  RUSTCODE_TLS_MAX  RUSTCODE_USER_AGENT
RUSTCODE_STREAM_TIMEOUT_SECS  RUSTCODE_OFFLINE
RUSTCODE_WIRE_DUMP  RUSTCODE_WIRE_LOG_FILE
RUSTCODE_HOME  RUSTCODE_BRAND_NAME  RUSTCODE_OAUTH_PROVIDER_NAME
```

### 2.5 Wire-contract keys (on-disk / on-wire, not just display strings)

| Old key | New key | Location |
|---------|---------|----------|
| `atomcode.legacy_cold_summary` | `rustcode.legacy_cold_summary` | `kernel/src/message.rs` |
| `atomcode.user_interruption` | `rustcode.user_interruption` | `kernel/src/message.rs` |
| `atomcode-rewind-v1` | `rustcode-rewind-v1` | `capabilities/src/session/rewind.rs` |
| `atomcode-v1:` (ACP cursor) | `rustcode-v1:` | `cli/src/acp/discovery.rs` |
| `x-atomcode-session-id` (header) | `x-rustcode-session-id` | `openai_compat.rs`, `anthropic.rs`, `ollama.rs` |

[DECISION D3] Same as D2: **rename with no compatibility read.** Existing `~/.atomcode`
sessions become unreadable. Documented as breaking; no importer is added (adding one would
contradict the "reduce importers" rule in `AGENTS.md`).

### 2.6 In-code product strings

| Category | Examples | Action |
|----------|----------|--------|
| Default brand | `default_brand_name()`, `ui.brand_name`, `RUSTCODE_BRAND_NAME` | rename default value; keep the override seam |
| User-Agent fallback | `DEFAULT_USER_AGENT = "atomcode"` (`provider/mod.rs`), `rustcode/<version>` in updater + config | rename |
| Repository / release URLs | `raw.atomgit.com/atomgit_atomcode/atomcode/...`, `relay-atomcode.atomgit.com`, `atomcode-skills.git` (`config/src/endpoints.rs`) | repoint to the fork repo; **keep** the `atomgit.com` / `gitcode.com` service hosts (separate brand) |
| OpenRouter attribution | `OPENROUTER_ATTRIBUTION_HEADERS` (`openai_compat.rs`) | see §4.3 G6 |
| Log prefix `[AtomCode]` | `extensions/vscode`, `extensions/jetbrains` | follow-up patch; out of core rename scope |

### 2.7 Single-point override

The whole rename is data-driven from one table. If the product name changes, only this
document's §2 table and the mechanical `s/atomcode/rustcode/` pass change; no architecture
decision depends on the literal string.

---

## 3. OBJECTIVE-2: Zero telemetry

### 3.1 What exists (verified)

`crates/rustcode-telemetry/` — 15 files, one crate, default-endpoint
`https://acs.atomgit.com/api/v1/events` (`telemetry/src/config.rs:6`), on-disk queue under
`$RUSTCODE_HOME`, gzip uploader, UUID identity, repo-origin detection, panic hook, first-run
notice. There is **no** Sentry / PostHog / Segment / OpenTelemetry / Mixpanel / Amplitude
dependency anywhere in `Cargo.toml` or `Cargo.lock` ([CHECK] grep over all `*.toml` and
`Cargo.lock` returned zero). The only telemetry is this first-party crate.

### 3.2 Delete

```text
[-] crates/rustcode-telemetry/                     (entire crate, 15 files)
[-] root Cargo.toml: remove "crates/rustcode-telemetry" from default-members
[-] docs/telemetry.md                              (145 lines, documents a removed feature)
```

### 3.3 Neutralize call sites — exact inventory

108 `rustcode_telemetry` references across 24 files in 9 crates.

| Crate | File | Refs | Action |
|-------|------|-----:|--------|
| config | `config/mod.rs` | 1 import + 1 field + render fn | drop `telemetry: TelemetryConfig`, `render_telemetry_section`, the `use` |
| config | `settings.rs` | 3 | remove the `telemetry.enabled` setting row + its match arm |
| config | `store.rs` | 2 | remove the `[telemetry]` block from the written config + the disable-write |
| config | `i18n/{en,zh_cn,messages}.rs` | ~12 | remove `CliAboutTelemetry*`, `CliHelpNoTelemetry`, `CliHelpClient`; adjust the offline copy line |
| config | `Cargo.toml` | 1 | **remove the dependency** (this is the dep-graph win, see §3.5) |
| auth | `oauth.rs` | 2 | remove `tel: Option<&Arc<Telemetry>>` from `login()` / `LoginSession::finish()`; drop `pending_invite` usage |
| codingplan | `setup.rs` | 5 | remove `tel` param + 3 `TakeCodingplan` tracks |
| coding | `telemetry.rs` | whole module | delete `TelemetryHook`, `ToolTelemetryMiddleware`, `MeteredProvider` |
| coding | `config.rs` | 4 | drop `telemetry: Option<Arc<Telemetry>>` + `SubagentTelemetryProviderFactory` |
| coding | `parts.rs` | 4 blocks | remove `MeteredProvider` wraps (host/review/subagent), middleware + hook registration, `set_telemetry_provider_factory` |
| coding | `lib.rs` | 2 | remove `pub mod telemetry;` + re-exports |
| cli | `main.rs` | 17+ | remove import, `--no-telemetry`, `telemetry` subcommand + `TelemetryAction`, init block, `install_panic_hook`, `SessionMode` mapping, all `track(...)` / flush calls |
| cli | `telemetry_cmd.rs` | whole file | delete |
| cli | `tests/telemetry_cmd.rs` | whole file | delete |
| clix | `tel.rs` | whole file | delete (sink, notice, `meter_provider`); `build_review_provider` moves to `code.rs` |
| clix | `main.rs` | 10 | remove sink wiring + `--no-telemetry` |
| daemon | `lib.rs` | 13 | remove `AppState.telemetry`, init, flush |
| daemon | `live_api.rs` | 11+ | remove `telemetry` params on 4 config builders; replace `Extension<SessionMode>` (§3.4) |
| daemon | `api_codingplan.rs`, `api_auth.rs` | 4+ | remove all `track(Event::TakeCodingplan{..})` / `login_success` durable enqueue |
| daemon | `telemetry_scope.rs` | whole file | delete `daemon_scope()` wrapper |
| daemon | `commands.rs`, `native_live.rs`, `kernel_runtime.rs`, `main.rs` | 4 | remove `telemetry` fields/params/CLI flag/test init |
| tuix | `lib.rs`, `event_loop/mod.rs`, `event_loop/commands.rs`, `event_loop/oauth_poll.rs`, `modals/onboarding_wizard.rs` | 4 files | remove `LoopCtx.telemetry`, `bind_telemetry_to_session`, `UseCommand` tracks, `Event` import |

### 3.4 Non-obvious removals (the ones that break compilation if missed)

1. **`rustcode_telemetry::SessionMode` is a *type* in the daemon HTTP API.**
   `daemon/src/api_codingplan.rs:325` and `daemon/src/api_auth.rs:250` declare
   `Extension<rustcode_telemetry::SessionMode>`; `live_api.rs:1698` consumes it. Deleting the
   crate removes the type, not just a call.
   [+] Replacement: a local `enum ClientMode { Headless, Tui, Ide, Vscode, Jetbrains, Webui, Desktop, Channel }`
   in `daemon/src/client_mode.rs`, parsed from the same request header. Keep the wire
   vocabulary byte-identical (`"webui"`, `"atomcode_desktop"` -> rename to `"rustcode_desktop"`
   only if the extension clients are patched in the same change; otherwise keep the old
   tag and rename in a follow-up). **Recommendation: keep the tags, rename only the Rust
   type** — avoids a silent protocol break with `extensions/`.

2. **Panic hook.** `install_panic_hook(telemetry)` uploads `Event::Panic`. Replace with a
   telemetry-free hook that prints to stderr and writes a local crash log; no network send.

3. **`atomcode_coding::SessionMode` is a *different* type** (`live_api.rs:441` uses
   `atomcode_coding::SessionMode::ExternalSnapshot`). Do not conflate the two — 233 total
   `SessionMode` grep hits cover both plus test usages.

4. **`rustcode-config` is a leaf that currently depends on `rustcode-telemetry`.**
   Removing it is a dependency-direction improvement, not just a deletion (§3.5).

5. **Offline/notice coupling.** `ATOMCODE_OFFLINE` and the "first-run notice" logic are
   entangled with telemetry init (`main.rs:1450-1494`). Offline mode must survive; only its
   telemetry branch goes.

### 3.5 Dependency cleanup (TIP-2)

After deletion, these become unreferenced and must be pruned from crates that held them only
for telemetry (`cargo tree -i` to confirm, not by guess):

```text
[-] crates/rustcode-telemetry/Cargo.toml deps:
    flate2, fs2, filetime, url, regex, dirs, chrono, uuid, reqwest, tracing, thiserror
[-] rustcode-config: drops rustcode-telemetry -> drops reqwest/rustls/flate2/uuid from
    every config-only build (this is what capabilities/Cargo.toml:26 warns about)
[-] dev-deps that existed only for telemetry tests: wiremock (coding/clix/daemon dev slots),
    rustcode-telemetry's `test-util` feature
[-] `test-util` feature of rustcode-telemetry: gone with the crate
[CHECK] reqwest feature flags re-audit across crates: keep
    ["stream","json","rustls-tls"] for the provider path; do NOT re-enable
    default features. rustls-tls (webpki base) must stay — it is the #514 backstop.
[CHECK] tokio features: telemetry required ["rt","sync","time","fs","macros"];
    fs was pulled for the queue. Re-audit per crate after removal.
[CHECK] Cargo.lock: `cargo update --workspace` then grep for any orphaned
    flate2/fs2/filetime entries.
```

### 3.6 Behavior deltas (must be documented, not silently dropped)

| Removed surface | Replacement |
|-----------------|-------------|
| `--no-telemetry` flag (CLI + daemon) | gone; nothing to disable |
| `rustcode telemetry {status,enable,disable,dump,clear,recover}` subcommand | gone |
| `[telemetry]` config section | gone; a stale `[telemetry]` key in an old config.toml must **not** fail the load — keep it as ignored-unknown or strip it in the loader |
| `telemetry.enabled` setting row | gone |
| login/use-command/LLM-chat/panic events | gone |
| offline-mode notice text mentioning telemetry | reworded |

[DECISION D4] A config.toml containing `[telemetry]` must still **load** (serde
`deny_unknown_fields` is not in play for this struct; verify in AGENT-B). Deleting the field
silently ignores it; that is acceptable and must be covered by a regression test.

---

## 4. OBJECTIVE-3: LLM provider decoupling

### 4.1 Current state audit (verified against HEAD)

What is **already good** — do not rebuild it:

| Asset | Location | Verdict |
|-------|----------|---------|
| Neutral provider trait | `kernel/src/provider.rs` `LlmProvider` (`model_name`, `context_window`, `bind_session_id`, `chat_stream`) | [SUCCESS] keep as the seam |
| OpenAI-compatible adapter | `capabilities/src/provider/openai_compat.rs` | [SUCCESS] `/chat/completions`, SSE, tool-call buffering, retry, TLS backstop, SwappableClient |
| Anthropic adapter | `capabilities/src/provider/anthropic.rs` | [SUCCESS] `/v1/messages`, event-typed SSE decoder, signed thinking round-trip, deterministic body |
| Ollama adapter | `capabilities/src/provider/ollama.rs` | [SUCCESS] |
| Factory / dispatch | `coding/src/provider_factory.rs` `DefaultCodingProviderFactory::build` | [SUCCESS] structure; gaps below |
| Retry policy | `capabilities/src/provider/retry.rs` | [SUCCESS] backoff, Retry-After, replay-sensitive gating |
| Env proxy | `capabilities/src/proxy.rs` + reqwest env detection | [SUCCESS] `RUSTCODE_PROXY_MODE` + `HTTP(S)_PROXY` |
| Mock integration tests | `capabilities/tests/{anthropic_mock,ollama_mock,http_mock,e2e}.rs` | [SUCCESS] **extend, do not duplicate** |
| Timeout knobs | `connect_timeout` / `open_timeout` (TTFB) / `idle_timeout` | [SUCCESS] already per-adapter |
| TLS | `skip_tls_verify`, `RUSTCODE_TLS_MAX`, webpki backstop | [SUCCESS] |

**Correction to the earlier draft:** the claim that the AtomGit request signer is applied to
every OpenAI-compatible build is **false**. `AtomGitProviderAuthenticator::request_signer`
returns `Ok(None)` unless `is_atomgit_gateway(base_url)` holds, and
`is_atomgit_gateway == atomcode_config::endpoints::is_codingplan_llm_gateway`, a host-based
HTTPS-only check (`llm-api.atomgit.com`, `pre-llm-api-cce.atomgit.com`, `api-ai.gitcode.com`,
or the `RUSTCODE_CODINGPLAN_LLM_BASE_URL` origin). A custom `base_url` already gets plain
`bearer_auth(api_key)`. AGENT-C must **not** "fix" this.

### 4.2 Gap list (the real work)

| ID | Gap | Severity | Fix |
|----|-----|----------|-----|
| **G1** | **No non-streaming API.** `LlmProvider` exposes only `chat_stream`. The spec ([STREAMING]) requires native non-stream support. | **High** | add `chat()` with a default stream-folding impl; adapters override with a real non-stream verb (§4.4) |
| **G2** | No per-provider custom headers (`extra_headers`). | High | add to transport config + inject in all three adapters |
| **G3** | No per-provider proxy. Only env/process-global proxy exists. | Medium | add `proxy: Option<String>`; `reqwest::Proxy::all()`, bypass env for that client |
| **G4** | No model mapping table. | Medium | `model_mapping: HashMap<String,String>` resolved once in the factory |
| **G5** | `provider_type: String` dispatch: `"anthropic-compatible"` currently falls into the OpenAI catch-all (`provider_factory.rs:112-169`). | Medium | explicit `ProviderKind` enum in the config crate; catch-all keeps OpenAI behavior + `tracing::warn!` |
| **G6** | `OPENROUTER_ATTRIBUTION_HEADERS` hardcodes the upstream identity (`HTTP-Referer: https://gitcode.com/atomgit_atomcode/atomcode`, `X-OpenRouter-Title: AtomCode`). | Medium | drop by default; keep `is_openrouter_url` gating; make the values config-driven and opt-in |
| **G7** | No unified error mapper. Each adapter builds `ProviderError { retryable, message, http_status, code, retry_after_secs }` ad hoc; `friendly_http_error()` is shared but partial. | Medium | `LlmError` (`thiserror`) + bidirectional `From` conversions at the kernel boundary (§4.5) |
| **G8** | Anthropic config lacks `extra_headers`/`proxy` parity with the OpenAI config. | Low | unify on one transport struct |

### 4.3 Decoupling policy

[DECISION D5] **Signing stays, but stays host-gated.** `ProviderAuthenticator` remains an
optional injection on `DefaultCodingProviderFactory`. Forks that never touch AtomGit simply
never call `.with_authenticator(...)`. `signer_available() == false` in a source build
already produces the explicit `SourceBuildGatewayUnsupported` error rather than a silent
downgrade — that fail-closed behavior is correct and must be preserved.

[DECISION D6] **Preset defaults.** No bundled `base_url` default pointing at any hosted
gateway. A provider entry without `base_url` fails at build time with an explicit
"base_url is required" error (fail-closed, per `AGENTS.md`), except `ollama`, which may keep
`http://localhost:11434`.

### 4.4 Trait design (Rust draft)

File: `crates/rustcode-kernel/src/provider.rs` (extend, do not fork).

```rust
// ---- Non-streaming: added to the existing neutral seam -------------------

/// A complete, non-streamed turn. The adapter fills what its backend returned;
/// fields are never synthesized.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChatResponse {
    pub text: String,
    pub reasoning: Option<String>,
    pub tool_calls: Vec<ToolCall>,
    pub usage: Option<TokenUsage>,
    pub finish_reason: FinishReason,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FinishReason {
    #[default]
    Stop,
    Length,        // finish_reason == "length" / stop_reason == "max_tokens"
    ToolCalls,
    ContentFilter,
    Other,
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn model_name(&self) -> &str;
    fn context_window(&self) -> u32 { 0 }
    fn bind_session_id(&self, _session_id: &str) {}

    /// Streaming turn. Unchanged.
    async fn chat_stream(
        &self,
        messages: &[Message],
        tools: &[ToolDef],
        options: &ChatOptions,
    ) -> Result<BoxStream<'static, StreamEvent>, ProviderError>;

    /// Non-streaming turn.
    ///
    /// DEFAULT IMPLEMENTATION folds [`Self::chat_stream`], so no existing adapter
    /// or test double has to change and the kernel turn loop is unaffected. An
    /// adapter overrides this only when its backend has a cheaper non-stream verb
    /// (`stream: false` for OpenAI-compatible; Anthropic has no non-stream SSE
    /// discount, so it keeps the default path).
    ///
    /// Contract: the first `StreamEvent::Error` terminates the fold and is returned
    /// as `Err`. `StreamEvent::Malformed` is skipped, never fatal.
    async fn chat(
        &self,
        messages: &[Message],
        tools: &[ToolDef],
        options: &ChatOptions,
    ) -> Result<ChatResponse, ProviderError> {
        let mut stream = self.chat_stream(messages, tools, options).await?;
        let mut out = ChatResponse::default();
        while let Some(ev) = stream.next().await {
            match ev {
                StreamEvent::TextDelta(t) => out.text.push_str(&t),
                StreamEvent::Reasoning(r) => out.reasoning.get_or_insert_with(String::new).push_str(&r),
                StreamEvent::ToolCall(tc) => out.tool_calls.push(tc),
                StreamEvent::Usage(u) => out.usage = Some(u),
                StreamEvent::Done { truncated } => {
                    out.finish_reason =
                        if truncated { FinishReason::Length } else { FinishReason::Stop };
                    break;
                }
                StreamEvent::Error(e) => return Err(e),
                StreamEvent::Malformed
                | StreamEvent::ResponseId(_)
                | StreamEvent::ResponseModel(_)
                | StreamEvent::ReasoningSignature { .. }
                | StreamEvent::ToolCallDelta { .. } => {}
            }
        }
        if !out.tool_calls.is_empty() {
            out.finish_reason = FinishReason::ToolCalls;
        }
        Ok(out)
    }
}
```

[CHECK] Non-stream call sites that should adopt `chat()` once available (already identified
as one-shot LLM calls): `coding/src/session_title.rs`,
`coding/src/next_prompt_suggestion.rs`, `capabilities/src/compaction.rs` (summary),
`review`. **Out of scope for PHASE-2 step 2.3** unless the reviewer asks; AGENT-C delivers
the seam, migration of call sites is a follow-up task.

File: `crates/rustcode-capabilities/src/provider/transport.rs` (new, shared by all adapters).

```rust
/// Everything the HTTP adapters need, independent of the wire protocol.
/// Secrets are only ever read from config/env; nothing is hard-coded.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderTransport {
    pub base_url: String,
    pub api_key: String,
    /// Extra request headers (gateway tenancy, routing tags, vendor-specific auth).
    /// Injected on EVERY request of this provider, after auth headers, so a
    /// caller CANNOT overwrite `authorization` / `x-api-key` / `anthropic-version`.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub extra_headers: HashMap<String, String>,
    /// Per-provider proxy. `None` => follow the process proxy policy
    /// (`RUSTCODE_PROXY_MODE` + `HTTP(S)_PROXY`). `Some(url)` => this client uses
    /// `reqwest::Proxy::all(url)` and IGNORES env proxies for this provider only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxy: Option<String>,
    #[serde(default)]
    pub skip_tls_verify: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    #[serde(default = "defaults::connect_timeout")] pub connect_timeout: Duration,
    #[serde(default = "defaults::open_timeout")]    pub open_timeout: Duration,
    #[serde(default = "defaults::idle_timeout")]    pub idle_timeout: Duration,
    #[serde(default)] pub retry: RetryPolicy,
}

/// Header names a caller may not override via `extra_headers` (auth integrity).
const RESERVED_HEADERS: &[&str] = &[
    "authorization", "x-api-key", "anthropic-version", "content-type",
    "content-length", "host", "accept-encoding",
];

impl ProviderTransport {
    /// Reject a mapping that tries to clobber auth/transport headers.
    /// Fail-closed at construction, per AGENTS.md.
    pub fn validate(&self) -> Result<(), LlmError> { /* ... */ }

    pub fn http_client(&self) -> Result<reqwest::Client, LlmError> {
        let mut b = crate::proxy::apply_async_proxy_policy(reqwest::Client::builder())
            .connect_timeout(self.connect_timeout)
            .pool_idle_timeout(retry::POOL_IDLE_TIMEOUT)
            .user_agent(self.user_agent.as_deref().unwrap_or(DEFAULT_USER_AGENT));
        if let Some(p) = &self.proxy {
            b = b.proxy(reqwest::Proxy::all(p).map_err(|e| LlmError::Config(e.to_string()))?);
        }
        if self.skip_tls_verify {
            b = b.danger_accept_invalid_certs(true);
        }
        b.build().map_err(|e| LlmError::Config(e.to_string()))
    }
}
```

File: `crates/rustcode-config/src/config/provider.rs` (additive fields + kind enum).

```rust
/// Wire protocol family. Replaces the free-form `provider_type: String` at the
/// dispatch boundary; the string form is still accepted on read via aliases.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderKind {
    #[serde(alias = "openai", alias = "openai-compatible", alias = "openai_compatible")]
    OpenAiCompatible,
    #[serde(alias = "anthropic", alias = "claude", alias = "anthropic-compatible")]
    Anthropic,
    #[serde(alias = "ollama")]
    Ollama,
}

/// Requested model -> wire model. Resolved once in the provider factory, so no
/// adapter or driver sees the alias. Empty => identity.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ModelMapping(pub HashMap<String, String>);

impl ModelMapping {
    pub fn resolve<'a>(&'a self, requested: &'a str) -> &'a str {
        self.0.get(requested).map(String::as_str).unwrap_or(requested)
    }
}

// --- additive fields on ProviderConfig / ModelProfile / ResolvedModelConfig ---
//   pub extra_headers: Option<HashMap<String, String>>
//   pub proxy: Option<String>
//   pub model_mapping: ModelMapping
```

Factory dispatch (`coding/src/provider_factory.rs`), after the fix:

```rust
let kind = ProviderKind::from_str(&cfg.provider_type).unwrap_or_else(|| {
    tracing::warn!("unknown provider_type {:?}; treating as openai-compatible", cfg.provider_type);
    ProviderKind::OpenAiCompatible
});
let model = cfg.model_mapping.resolve(&cfg.model).to_string();  // one place, all adapters
match kind {
    ProviderKind::Anthropic            => { /* AnthropicConfig  + transport */ }
    ProviderKind::Ollama               => { /* OllamaConfig     + transport */ }
    ProviderKind::OpenAiCompatible     => { /* OpenAiCompatConfig + transport */ }
}
```

### 4.5 Error mapper (TIP-1, TIP-3)

File: `crates/rustcode-capabilities/src/provider/error.rs` (new).

```rust
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("authentication rejected (HTTP {status}): {detail}")]
    Auth { status: u16, detail: String },
    #[error("rate limited (HTTP 429); retry after {retry_after_secs:?}s")]
    RateLimited { retry_after_secs: Option<u64> },
    #[error("invalid request (HTTP {status}): {detail}")]
    InvalidRequest { status: u16, detail: String },
    #[error("upstream error (HTTP {status}): {detail}")]
    Upstream { status: u16, detail: String },
    #[error("transport failure: {0}")]      Transport(String),
    #[error("stream idle for {0:?}")]       IdleTimeout(Duration),
    #[error("response decode failure: {0}")] Decode(String),
    #[error("provider misconfiguration: {0}")] Config(String),
}

impl LlmError {
    pub fn retryable(&self) -> bool {
        matches!(self, Self::RateLimited { .. } | Self::Upstream { .. } | Self::Transport(_))
    }
    /// Wire a kernel `ProviderError` into the typed set. `code` carries the
    /// vendor's own error type (e.g. Anthropic `overloaded_error`).
    pub fn from_provider(e: &ProviderError, status: Option<u16>) -> Self { /* ... */ }
}

impl From<LlmError> for ProviderError {
    fn from(e: LlmError) -> Self {
        ProviderError {
            retryable: e.retryable(),
            message: e.to_string(),
            http_status: /* from variant */,
            code: None,
            retry_after_secs: /* RateLimited only */,
        }
    }
}
```

Top-level layers (`cli`, `daemon`, `clix`) wrap with `anyhow::Context` and never
`.unwrap()`. The existing `friendly_http_error()` (401/402 headlines, the CodingPlan 403
hint, the literal `HTTP 429: ` prefix the kernel rate-limit path strips) is **preserved
verbatim** — it is load-bearing for the TUI and for `rate_limit_server_message`.

### 4.6 SSE normalization (TIP-3 parity table)

| Internal `StreamEvent` | OpenAI-compatible | Anthropic |
|---|---|---|
| `TextDelta` | `choices[].delta.content` | `content_block_delta` / `text_delta` |
| `Reasoning` | `choices[].delta.reasoning_content` (policy-gated) | `content_block_delta` / `thinking_delta` |
| `ReasoningSignature` | (n/a) | `content_block_stop` on `thinking` / `redacted_thinking` |
| `ToolCall` (whole) | emitted at `finish_reason == "tool_calls"` | emitted at `content_block_stop` on `tool_use` |
| `ToolCallDelta` (live) | `delta.tool_calls[].function.arguments` | `content_block_delta` / `input_json_delta` |
| `Usage` | last non-null `usage` | `message_start` input + `message_delta` output; `message_stop` emits |
| `Done{truncated}` | `finish_reason` | `message_stop`; `max_tokens` -> `truncated: true` |
| `Error` | mid-stream chunk error | `event: error` |
| `ResponseId` | `id` | `message_start.message.id` |

[CHECK] Parity is already implemented and unit-tested in both adapters
(`anthropic.rs` `tests`, `openai_compat.rs` `tests`). AGENT-C adds a shared conformance test
asserting the *same* recorded conversation produces the *same* `StreamEvent` sequence from
both adapters — that is the cross-protocol guarantee, and it is currently untested.

### 4.7 Configuration schema (for `config.example.toml`)

```toml
# ---- OpenAI-compatible (any gateway / relay / self-hosted) ---------------
[providers.my-openai]
type           = "openai-compatible"        # or: openai
base_url       = "https://my-gateway.example.com/v1"
api_key        = "env:MY_OPENAI_KEY"        # env: indirection; never a literal key
model          = "gpt-4o"
context_window = 128000
extra_headers  = { "X-Tenant" = "acme", "X-Experiment" = "sse-v2" }
proxy          = "http://corp-proxy.example.com:3128"
skip_tls_verify = false
timeout_connect_secs = 30
timeout_open_secs    = 90
timeout_idle_secs    = 120
retry_max_attempts   = 3

[providers.my-openai.model_mapping]
"fast"   = "gpt-4o-mini"
"smart"  = "gpt-4o"

# ---- Anthropic native /v1/messages ---------------------------------------
[providers.my-anthropic]
type     = "anthropic-compatible"           # or: anthropic | claude
base_url = "https://my-anthropic-gw.example.com"
api_key  = "env:MY_ANTHROPIC_KEY"
model    = "claude-opus-4"
thinking_enabled = false
```

[SECURITY] `api_key` supports an `env:` indirection so no credential lands in the config
file or in a repo. Default templates ship with placeholder values only; AGENT-D's secret
scan enforces this.

---

## 5. OBJECTIVE-4: License & compliance

### 5.1 State

Root `LICENSE` is MIT, `Copyright (c) 2026 Yubang Xu`. Upstream origin
`https://atomgit.com/atomgit_atomcode/atomcode`; fork source
`https://gitcode.com/SecLab/RustCode`.

Three documents already exist in the working tree (untracked, produced by an earlier pass):
`docs/ORIGINAL_LICENSE.md` (33 lines), `docs/THIRD_PARTY_NOTICES.md` (36 lines),
`docs/UPSTREAM_CREDITS.md` (40 lines). Content verified correct against TIP-4:
upstream MIT text is verbatim, the copyright notice is intact, and the fork's own changes
are described without claiming to relicense anything.

### 5.2 Actions

```text
[+] Root LICENSE: keep MIT, keep "Copyright (c) 2026 Yubang Xu" verbatim, APPEND a second
    copyright line for the fork. Do not delete or edit the upstream notice.
[+] docs/ORIGINAL_LICENSE.md      — verbatim upstream MIT. KEEP AS IS.
[+] docs/THIRD_PARTY_NOTICES.md   — upstream attribution + pointer to `cargo license`. KEEP.
[+] docs/UPSTREAM_CREDITS.md      — fork statement + change summary. KEEP (update after PHASE-2).
[+] New modules authored by the fork carry a short header:
        // Copyright (c) 2026 The rustcode authors. MIT. Derived from atomcode (MIT,
        // Copyright (c) 2026 Yubang Xu) — see docs/UPSTREAM_CREDITS.md.
[-] Do NOT strip notices from inherited files.
[-] Do NOT add a license-incompatible dependency.
[!] `crates/rustcode-codingplan-crypto/` is an open-source placeholder for a CLOSED-SOURCE
    signer overlaid only by the upstream release pipeline. A self-built binary therefore
    cannot sign AtomGit gateway requests (`signer_available() == false`). This is
    pre-existing upstream behavior, it fails closed, and it is out of scope — but it must
    be stated in the fork README so users are not surprised.
```

[CHECK] `THIRD_PARTY_NOTICES.md` currently delegates the transitive crate list to
`cargo license`. That is acceptable and stays accurate automatically; no static list to rot.

---

## 6. Sub-agent execution plan

### 6.1 File ownership (non-overlapping)

| Agent | Owns | Step |
|-------|------|------|
| **[AGENT-A]** Architect & Compliance | `docs/ORIGINAL_LICENSE.md`, `docs/THIRD_PARTY_NOTICES.md`, `docs/UPSTREAM_CREDITS.md`, root `LICENSE`; root `Cargo.toml`; all `crates/*/Cargo.toml` (package/lib/bin names, path deps, `default-members`); `config/src/distribution.rs`; `config/src/endpoints.rs`; `docker/*`; `.github/workflows/build.yml`; `scripts/install.*`, `scripts/uninstall.*`, `scripts/release*.sh`; `latest.json` | 2.1 |
| **[AGENT-B]** Telemetry Stripper | delete `crates/rustcode-telemetry/`, `docs/telemetry.md`; `coding/src/{lib.rs,telemetry.rs,config.rs,parts.rs}`; `cli/src/main.rs`, `cli/src/telemetry_cmd.rs`, `cli/tests/telemetry_cmd.rs`; `auth/src/oauth.rs`; `codingplan/src/setup.rs`; `config/src/{config/mod.rs,settings.rs,store.rs,i18n/*,Cargo.toml}`; `daemon/src/*` (+ new `daemon/src/client_mode.rs`); `clix/src/{main.rs,code.rs,tel.rs}`; `tuix/src/*`; i18n message enums | 2.2 |
| **[AGENT-C]** LLM Engine Builder | `kernel/src/provider.rs` (`chat()`, `ChatResponse`, `FinishReason`); new `capabilities/src/provider/{transport.rs,error.rs}`; `capabilities/src/provider/{openai_compat.rs,anthropic.rs,ollama.rs,mod.rs}`; `config/src/config/provider.rs`; `coding/src/provider_factory.rs`; `docs/config.example.toml` | 2.3 |
| **[AGENT-D]** QA & Validation | `cargo fmt`, `cargo check --workspace --all-targets`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`; extend `capabilities/tests/{anthropic_mock,ollama_mock,http_mock}.rs`; new `capabilities/tests/provider_conformance.rs`; new `capabilities/tests/nonstream.rs`; C1/C3 grep gates | 2.4 |

### 6.2 Ordering and merge discipline

```text
STEP 2.1  [AGENT-A]  rename + license
          |
          +-- (must land first: every later agent edits files whose crate names changed)
          |
STEP 2.2  [AGENT-B]  telemetry strip
          |
          +-- overlaps AGENT-C ONLY in `coding/src/provider_factory.rs` and
          |   `config/src/config/provider.rs`, and in DISJOINT functions.
          |   Rule: B deletes the `telemetry` FIELD, C adds `extra_headers` /
          |   `proxy` / `model_mapping` FIELDS. If both run in parallel, C rebases
          |   and re-applies only its own field additions.
          |
STEP 2.3  [AGENT-C]  LLM engine  (recommended: run AFTER 2.2 to avoid the field collision)
          |
STEP 2.4  [AGENT-D]  validation over the whole tree
```

[WARN] Recommended serialization: **2.1 -> 2.2 -> 2.3 -> 2.4**. A and B touch disjoint files
and could parallelize in principle, but B edits `config/src/config/mod.rs` while A edits
`config/Cargo.toml` and `distribution.rs`; the risk is low but non-zero (Cargo.toml vs
module edits). C serialized after B removes the only real conflict.

### 6.3 Change-impact analysis

| Change | Blast radius | Risk | Mitigation |
|--------|--------------|------|------------|
| Crate rename | 14 `Cargo.toml`, `Cargo.lock`, ~439 `.rs` files by import path | Low (mechanical) | single `sed` pass + `cargo check` after each crate batch |
| `RUSTCODE_HOME` / `.rustcode` | every user's existing install | **High (data)** | D2/D3: document as breaking; no silent read of `~/.atomcode` |
| Telemetry crate deletion | 24 files, 9 crates | Medium | the four non-obvious items in §3.4 are the failure modes; each has a named check |
| `SessionMode` type removal from daemon | daemon HTTP API | Medium | local `ClientMode` enum, **wire tags unchanged** |
| `[telemetry]` key left in old configs | config loader | Low | D4: must still load; regression test |
| `chat()` added to `LlmProvider` | ~40 test doubles + 3 adapters | **Low** — default impl means zero required changes | assert `cargo test` green before/after |
| `ProviderKind` dispatch | every configured provider | Medium | aliases keep old strings working; unknown -> OpenAI + warn (no behavior change) |
| `extra_headers` reserved-name guard | new surface only | Low | validate at construction, fail-closed |
| Anthropic `thinking` / signed-block echo | conversation replay | **High if touched** | AGENT-C must NOT touch `format_assistant_message` / `ReasoningSignature`; it is provider-bound and heavily tested |

---

## 7. Verification plan (AGENT-D)

```bash
# [CHECK] 1 — compile, every target
cargo check --workspace --all-targets

# [CHECK] 2 — lint (C2: no blocking I/O in async paths)
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --workspace --all-targets -- -W clippy::await_holding_lock

# [CHECK] 3 — tests
cargo test --workspace

# [CHECK] 4 — rename residue (expect: only intentional upstream-URL/atomgit-service hits)
grep -rn "atomcode" --include=*.rs --include=*.toml crates/ | grep -v "atomgit\|gitcode" || true

# [CHECK] 5 — telemetry residue (expect: EMPTY)
grep -rni "telemetry\|analytics\|posthog\|sentry\|segment" --include=*.rs --include=*.toml crates/
grep -n "atomcode-telemetry\|rustcode-telemetry" Cargo.lock || true

# [CHECK] 6 — C1: no emoji in rust sources
grep -rnP "[\x{1F300}-\x{1FAFF}\x{2600}-\x{27BF}]" --include=*.rs crates/ || true

# [CHECK] 7 — C3: no hardcoded secrets in the tracked tree
grep -rnE "(sk-[A-Za-z0-9]{20,}|sk-ant-[A-Za-z0-9_-]{20,}|ghp_[A-Za-z0-9]{20,})" \
     --include=*.rs --include=*.toml --include=*.md crates/ docs/ || true

# [CHECK] 8 — C5: no unwrap/expect on production paths (tests excluded)
grep -rn "\.unwrap()\|\.expect(" --include=*.rs crates/*/src/ | grep -v "#\[cfg(test)\]" || true

# [CHECK] 9 — runtime smoke
cargo run -p rustcode-cli -- --help          # no `telemetry` subcommand, no --no-telemetry
cargo run -p rustcode-cli -- -p "say ok"     # headless turn against a configured provider
RUSTCODE_WIRE_DUMP=1 cargo run -p rustcode-cli -- -p "say ok"
ls ~/.rustcode/wire-dump                     # proves the new name is live
```

New / extended tests:

```text
[+] capabilities/tests/provider_conformance.rs
    - one recorded conversation, two adapters, identical StreamEvent sequence
    - extra_headers injected on both; assert auth headers were NOT clobbered
    - reserved header name -> construction error (fail-closed)
[+] capabilities/tests/nonstream.rs
    - OpenAI-compatible `chat()` with `"stream": false` (real non-stream verb)
    - Anthropic `chat()` via the default fold (no non-stream verb exists)
    - both: text / tool_calls / usage / finish_reason parity with the stream path
[~] capabilities/tests/{anthropic_mock,ollama_mock,http_mock}.rs
    - extend existing mocks with a custom base_url + extra_headers + proxy; do NOT
      duplicate what is already there
[+] config: model_mapping resolution (alias hit, alias miss, empty mapping)
[+] config: a config.toml containing a stale `[telemetry]` section still loads (D4)
[+] coding: provider_factory dispatch for every ProviderKind alias
[+] daemon: ClientMode parse parity with the previous SessionMode tags
```

Manual model-connectivity check (requires the user's own credentials; never committed):

```bash
export RUSTCODE_BASE_URL="https://<your-gateway>/v1"
export RUSTCODE_API_KEY="<your-key>"          # or api_key = "env:YOUR_KEY" in config.toml
cargo run -p rustcode-cli -- --model <model> -p "reply with the single word: ok"
```

---

## 8. Risks, open questions, and the GATEWAY

| ID | Item | Kind |
|----|------|------|
| D1 | Product name `rustcode` — confirm or override | **Decision** |
| D2 | No legacy `ATOMCODE_*` / `~/.atomcode` fallback (breaking) | **Decision** |
| D3 | Wire-key rename with no compatibility read (existing sessions unreadable) | **Decision** |
| D4 | A stale `[telemetry]` config section must still load | **Decision** |
| D5 | AtomGit signer retained but host-gated; not removed | Decision |
| D6 | No bundled hosted `base_url`; missing `base_url` fails closed | Decision |
| D7 | `extensions/` (vscode, jetbrains) reference `[AtomCode]` logs and binary names — out of core scope, follow-up patch | Open |
| D8 | `crates/rustcode-codingplan-crypto/` closed-source overlay: self-built binaries cannot sign gateway requests (pre-existing upstream behavior) | Open |
| D9 | Non-stream `chat()` is delivered as a seam in PHASE-2; migrating `session_title` / `next_prompt_suggestion` / `compaction` call sites is a follow-up | Open |
| D10 | `.github/workflows/build.yml`, `scripts/release*.sh`, `latest.json` need the new asset names before any release is cut | Open |

[GATEWAY] Confirm **D1** (product name) and **D2/D3** (breaking-change acceptance).
Everything else in this document is ready to execute. On confirmation, execution starts at
STEP 2.1 ([AGENT-A]) and proceeds in the order fixed in §6.2.
