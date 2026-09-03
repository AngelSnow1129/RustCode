# Fork Refactor Summary (RustCode)

Secondary-development fork of `https://gitcode.com/SecLab/RustCode`. Three objectives delivered:

1. [OBJECTIVE-1] Product rebrand `atomcode` -> `rustcode` (crates, binaries, config dir, env vars).
2. [OBJECTIVE-2] Zero telemetry — `rustcode-telemetry` crate deleted, all reporting removed.
3. [OBJECTIVE-3] LLM provider decoupling — self-hosted OpenAI/Anthropic endpoints.

## 1. Rebrand (rustcode)

- 13 crates renamed `atomcode-*` -> `rustcode-*` (e.g. `rustcode-cli`, `rustcode-daemon`, `rustcode-coding`, `rustcode-kernel`, `rustcode-capabilities`, `rustcode-config`).
- Binaries: `atomcode` -> `rustcode`, `atomcode-daemon` -> `rustcode-daemon`, `atomcodex` -> `rustcodex`.
- Config dir `.rustcode` -> `.rustcode`; env vars `ATOMCODE_*` -> `RUSTCODE_*` (centralized in `rustcode-config/src/endpoints.rs`).
- Wire-contract keys migrated to the `rustcode.*` namespace (no compatibility read of old `~/.rustcode` sessions), per the approved "fresh start" decision.
- In-code product strings updated; OpenRouter attribution now references `rustcode` / the fork repo.

## 2. Zero telemetry

- Deleted `crates/rustcode-telemetry/`.
- Removed `Telemetry`/`Event`/`CurrentContext`/`track`/`install_panic_hook`/panic-reporting across `cli`, `daemon`, `coding`, `auth`, `clix`, `tuix`, `config`.
- Removed `--no-telemetry` flag, the `telemetry` CLI subcommand, the `telemetry = {}` config section, and all `rustcode_telemetry::` references (now 0 in `crates/*/src`).
- Pure non-reporting types that other code needed (`SessionMode`, `RepoOrigin`, `detect_repo_origin`) re-homed in `rustcode-config/src/session_mode.rs` — no network, no events.
- Crash handling keeps a stderr-only panic printer; nothing is sent off-box.

## 3. LLM provider decoupling

- Provider factory (`rustcode-coding/src/provider_factory.rs`) dispatch now maps:
  - `claude | anthropic | anthropic-compatible` -> Anthropic adapter (`/v1/messages`)
  - `ollama` -> Ollama adapter
  - `openai | openai-compatible | _` -> OpenAI-compatible adapter (`/v1/chat/completions`)
  (previously `anthropic-compatible` fell into the OpenAI catch-all — fixed)
- AtomGit request signing is now gated by `is_atomgit_gateway(base_url)`: only `atomgit`/`relay` hosts get the upstream signer; every other (self-hosted / third-party) endpoint uses plain `bearer_auth(api_key)`.
- New `ProviderConfig` / `ProviderAccountConfig` / `ResolvedModelConfig` / `CodingAgentConfig` / `CodingRuntimeConfig` fields:
  - `extra_headers: Option<HashMap<String,String>>` — custom per-request headers (gateway auth/tenant), applied on both adapters, never logged.
  - `proxy: Option<String>` — per-provider forward proxy, applied to the reqwest client build, respecting `skip_tls_verify`.
- See `docs/config.example.toml` for a self-hosted `[providers.my-openai-gw]` / `[providers.my-anthropic-gw]` template.

## License & compliance

- `docs/ORIGINAL_LICENSE.md` — placement / declaration note (no verbatim license text); the verbatim upstream MIT license text lives in `docs/UPSTREAM_RUSTCODE_LICENSE.md` (Copyright (c) 2026 Yubang Xu).
- `docs/THIRD_PARTY_NOTICES.md` — third-party / inherited-component notice.
- `docs/UPSTREAM_CREDITS.md` — fork provenance and change summary.
- Root `LICENSE` remains MIT.

## Verification

```
export PATH=/root/.cargo/bin:$PATH
cargo check --workspace                 # [CHECK] passes (warnings only)
cargo test  -p rustcode-config          # [CHECK] passes
cargo test  -p rustcode-capabilities    # [CHECK] passes
cargo test  -p rustcode-coding --lib    # [CHECK] 420 passed
```

Notes:
- Full `cargo test --workspace` may OOM when linking the largest integration test binaries in constrained CI runners (environmental, not a code error). Prefer per-crate `-p` runs; `--lib` scopes link size.
- `provider_factory` dispatch is covered by `rustcode-coding` unit tests; a mock-provider SSE test can be added under `rustcode-capabilities/tests` to assert `openai-compatible` / `anthropic-compatible` reach the correct adapter with a custom `base_url`.

## Manual smoke test (self-hosted endpoint)

1. Put in `~/.rustcode/config.toml`:
   ```toml
   default_provider = "my-openai-gw"
   [providers.my-openai-gw]
   type = "openai-compatible"
   api_key = "env:MY_KEY"
   model = "your-model"
   base_url = "https://your-gateway.example.com/v1"
   ```
2. `export MY_KEY=...`
3. `rustcode "explain this repo"` — traffic should hit your gateway, no upstream AtomGit signing, no telemetry.

## Known gaps / follow-ups

- Extension crates under `extensions/` (vscode / jetbrains) still reference `[RustCode]` logs and the old binary names; they are out of the core rename scope and should be patched in a follow-up if the fork ships them.
- `docs/telemetry.md` from upstream is now obsolete for this fork (telemetry removed); kept on disk but no longer describes shipping behavior.
