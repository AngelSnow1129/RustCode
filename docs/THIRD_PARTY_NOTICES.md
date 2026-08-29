# THIRD PARTY NOTICES

This fork (product `rustcode`) is derived from the upstream project
`https://gitcode.com/SecLab/RustCode` (origin `atomgit_atomcode/atomcode`),
distributed under the MIT license archived in `docs/ORIGINAL_LICENSE.md`
(Copyright (c) 2026 Yubang Xu).

## Upstream attribution

- Original author / copyright holder: Yubang Xu (2026).
- Upstream license: MIT (see `docs/ORIGINAL_LICENSE.md`).
- Upstream repository path string referenced in code: `atomgit_atomcode/atomcode`.

## Inherited third-party components

The full and authoritative list of third-party Rust crate licenses is produced
by the build tooling from `Cargo.lock`:

    cargo license --workspace   # or: cargo tree + license metadata

Each transitive dependency retains its own license (MIT / Apache-2.0 / BSD /
ISC / etc.) as declared in its own `Cargo.toml`. This document does not
relicense any dependency; it only records the upstream project's origin and the
MIT terms under which the upstream *code* was received.

## Modifications

Modifications made in this fork:
- Product rename `atomcode` -> `rustcode` (crates, binaries, config dir, env vars).
- Removal of the `atomcode-telemetry` crate and all telemetry/analytics call sites.
- Decoupling of the LLM provider layer from the upstream AtomGit signing gateway;
  added `extra_headers` / `proxy` provider config and explicit `openai-compatible`
  / `anthropic-compatible` provider types for self-hosted endpoints.

New code authored in this fork carries its own copyright header and is offered
under the same MIT terms unless stated otherwise in the file.
