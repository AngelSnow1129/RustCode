# UPSTREAM CREDITS

This project (`rustcode`) is a fork / secondary development of:

- **Upstream repository:** https://gitcode.com/SecLab/RustCode
- **Upstream origin:** https://atomgit.com/atomgit_atomcode/atomcode
- **Original license:** MIT — Copyright (c) 2026 Yubang Xu (preserved verbatim in
  `docs/ORIGINAL_LICENSE.md`).
- **Third-party notices:** `docs/THIRD_PARTY_NOTICES.md`.

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
   was extended with `extra_headers`, `proxy`, and explicit
   `openai-compatible` / `anthropic-compatible` provider types.
4. **License & compliance (OBJECTIVE-4):** upstream MIT license and credits are
   archived in `docs/ORIGINAL_LICENSE.md`, `docs/THIRD_PARTY_NOTICES.md`, and
   this file.

The `atomgit` brand (gateway / OAuth provider, `api.gitcode.com`,
`llm-api.atomgit.com`) is a separate service and is not renamed; only the
product-named repo path segment was updated to the fork's repository.
