# UPSTREAM CREDITS

This project (`rustcode`) is a fork / secondary development of:

- **Predecessor project (direct parent):** RustCode — MIT,
  Copyright (c) 2026 Yubang Xu, preserved verbatim in
  [`docs/UPSTREAM_RUSTCODE_LICENSE.md`](./UPSTREAM_RUSTCODE_LICENSE.md).
- **Upstream origin (ultimate):** MIT. **The verbatim text is NOT yet archived;**
  see [`docs/ORIGINAL_LICENSE.md`](./ORIGINAL_LICENSE.md), which is a placeholder
  recording the gap. Do not reconstruct it from a downstream copy.
- **Third-party notices:**
  [`docs/THIRD_PARTY_NOTICES.md`](./THIRD_PARTY_NOTICES.md).

## Compliance statement

In accordance with the MIT license of the upstream project, the original
copyright notice and permission notice are retained in all inherited source
files. New files authored by this fork add a separate copyright header and do
not remove or alter the upstream notice on inherited code.

## What changed relative to upstream

1. **Rebranding (OBJECTIVE-1):** all `atomcode-*` crates, binaries
   (`atomcode`, `rustcode-daemon`, `atomcodex`), the `~/.rustcode` config
   directory, and `ATOMCODE_*` environment variables were renamed to the
   `rustcode` identity. On-disk wire-contract keys (`atomcode-v1:`,
   `atomcode-rewind-v1`, `atomcode.*`) were migrated to `rustcode.*` (no
   compatibility read of old sessions).
2. **Zero telemetry (OBJECTIVE-2):** the `atomcode-telemetry` crate and every
   telemetry/analytics/panic-reporting call site, CLI flag (`--no-telemetry`),
   `telemetry` subcommand, and `telemetry = {}` config key were removed.
3. **LLM provider decoupling (OBJECTIVE-3):** the `LlmProvider` adapter layer
   was decoupled from the upstream AtomGit request-signing gateway (signing is
   now applied only to `atomgit`/`relay` hosts), and self-hosted configuration
   was extended with `extra_headers`, `proxy`, `model_mapping`, per-provider
   `timeout`, and explicit `openai-compatible` / `anthropic-compatible` provider
   types.
4. **License & compliance (OBJECTIVE-4):** the predecessor's MIT license is
   archived in `docs/UPSTREAM_RUSTCODE_LICENSE.md`, third-party notices in
   `docs/THIRD_PARTY_NOTICES.md`, and the fork lineage in this file.
   `docs/ORIGINAL_LICENSE.md` is a **placeholder** for the ultimate upstream's
   license text, which has not been obtained.

The `atomgit` brand (gateway / OAuth provider) is a separate service and is not
renamed; only the product identity was updated to the `rustcode` name.
