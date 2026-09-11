# THIRD PARTY NOTICES

This fork (product `rustcode`) is derived from a predecessor project, which is
itself derived from an earlier ultimate upstream project.

## Upstream attribution

- Predecessor (`atomgit_atomcode/atomcode`, product "AtomCode") — MIT,
  Copyright (c) 2026 Yubang Xu.
  The verbatim text is archived in `docs/UPSTREAM_RUSTCODE_LICENSE.md`.
- Ultimate upstream — MIT.
  **The verbatim text is NOT yet archived**; `docs/ORIGINAL_LICENSE.md` is a
  placeholder recording the gap. It must not be reconstructed from a downstream
  copy.

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
- Removal of the `rustcode-telemetry` crate and all telemetry/analytics call sites.
- Full decoupling of the LLM provider layer from any platform-specific signing gateway;
  added `extra_headers` / `proxy` provider config and explicit `openai-compatible`
  / `anthropic-compatible` provider types for self-hosted endpoints.

New code authored in this fork carries its own copyright header and is offered
under the same MIT terms unless stated otherwise in the file.
