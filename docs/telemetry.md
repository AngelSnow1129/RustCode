# Telemetry — removed

```text
[STATUS] This fork ships ZERO telemetry.
[NOTE]   This page is kept only so existing links resolve. It describes a
         pipeline that no longer exists, and none of it is shipping behavior.
```

The upstream project shipped an anonymous usage-telemetry pipeline. This fork
(`gitcode.com/SecLab/RustCode`) removed it entirely.

## What was removed

- The `rustcode-telemetry` crate — deleted; no directory, no dependency.
- The `telemetry` CLI subcommand and the `[telemetry]` config section. A legacy
  `[telemetry]` section in `config.toml` is now **silently ignored**, so existing
  configuration keeps loading.
- The `CliOverride` type and the effect of `--no-telemetry`. The flag is still
  **accepted and ignored** (with a warning on stderr) because older IDE
  extensions pass it.
- Crash reporting. A panic writes to stderr only; nothing leaves the machine.

## The historical pipeline (for auditing only)

Upstream collected launch, LLM-turn, command, login, and crash events into a
local NDJSON queue and posted them to a self-hosted endpoint. None of that code
is present here: there is no event queue, no sender, and no endpoint. The
description is retained only so a reader who remembers the old behavior can
confirm it is gone.

## What still exists and is NOT telemetry

Do not remove these — they are frequently misidentified:

| Item | What it actually is |
|---|---|
| `SessionMode` / `ClientMode` | Local branching on which client is connected (IDE, webui, TUI). Drives token permissions and webui paths. Wire tag strings are deliberately stable so older extensions keep working. |
| `RepoOrigin` / `detect_repo_origin` | Pure string parsing of the git remote host. No network. |
| turn datalog | Structured log written to a **local** file for debugging a turn. |
| update check | A version check against the release manifest. It is a network request and is therefore configurable, but it sends no usage events. |

## Verifying

```sh
grep -rniE 'sentry|posthog|amplitude|mixpanel|opentelemetry|prometheus|statsd' \
  --include='*.rs' --include='*.toml' crates/
# [CHECK] must print nothing
```

No third-party analytics SDK is a dependency of this project. The historical
pipeline was self-built, so removing it was dead-code removal, not SDK surgery.
