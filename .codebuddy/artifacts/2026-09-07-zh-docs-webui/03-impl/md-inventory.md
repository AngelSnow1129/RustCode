# md 清单（AC-1）

- 生成者: `scripts/check-zh-docs.py inventory`
- 基线 (--base): `3ee655e3`
- 分母: `git ls-files -- '*.md'`（仓库当前跟踪的 md）

## 恒等式

```
SKIP_A(4) + SKIP_B(47) + SKIP_C(1) + TODO(96) + ZH(138) = 286 ；全仓 md 总数 = 286 ；恒等式成立
```

| 桶 | 数量 | 期望 | 说明 |
|---|---|---|---|
| SKIP_A | 4 | 4 | 路径/文件名小写含 license/licence/notices/credits/copying |
| SKIP_B | 47 | 47 | `crates/rustcode-review/rules/*.md`（运行时载荷，Q1 不动） |
| SKIP_C | 1 | 1 | `extensions/jetbrains/CHANGELOG.md`（逐字历史） |
| TODO=EN | 52 | - | ratio == 0，待汉化 |
| TODO=MIXED | 44 | - | 0 < ratio < 0.5，待汉化 |
| ZH | 138 | - | ratio >= 0.5，已中文（含 OWNED_ELSEWHERE） |

> ratio = 含 CJK 行数 / max(含 ASCII 字母行数, 1)

## A 类 · 法律/声明文本（跳过）（4）

| 路径 | Han 行 | 字母行 | ratio | 备注 |
|---|---|---|---|---|
| `docs/ORIGINAL_LICENSE.md` | 0 | 38 | 0.000 |  |
| `docs/THIRD_PARTY_NOTICES.md` | 0 | 28 | 0.000 |  |
| `docs/UPSTREAM_CREDITS.md` | 0 | 42 | 0.000 |  |
| `docs/UPSTREAM_RUSTCODE_LICENSE.md` | 0 | 28 | 0.000 |  |

## B 类 · review rules（跳过）（47）

| 路径 | Han 行 | 字母行 | ratio | 备注 |
|---|---|---|---|---|
| `crates/rustcode-review/rules/arkts.md` | 0 | 42 | 0.000 |  |
| `crates/rustcode-review/rules/build_gradle.md` | 0 | 3 | 0.000 |  |
| `crates/rustcode-review/rules/c.md` | 0 | 25 | 0.000 |  |
| `crates/rustcode-review/rules/cangjie.md` | 0 | 42 | 0.000 |  |
| `crates/rustcode-review/rules/clojure.md` | 0 | 22 | 0.000 |  |
| `crates/rustcode-review/rules/cmake.md` | 0 | 7 | 0.000 |  |
| `crates/rustcode-review/rules/cpp.md` | 0 | 30 | 0.000 |  |
| `crates/rustcode-review/rules/csharp.md` | 0 | 27 | 0.000 |  |
| `crates/rustcode-review/rules/css.md` | 0 | 7 | 0.000 |  |
| `crates/rustcode-review/rules/dart.md` | 0 | 22 | 0.000 |  |
| `crates/rustcode-review/rules/dockerfile.md` | 0 | 10 | 0.000 |  |
| `crates/rustcode-review/rules/elixir.md` | 0 | 24 | 0.000 |  |
| `crates/rustcode-review/rules/erlang.md` | 0 | 22 | 0.000 |  |
| `crates/rustcode-review/rules/go.md` | 0 | 39 | 0.000 |  |
| `crates/rustcode-review/rules/graphql.md` | 0 | 6 | 0.000 |  |
| `crates/rustcode-review/rules/groovy.md` | 0 | 10 | 0.000 |  |
| `crates/rustcode-review/rules/haskell.md` | 0 | 23 | 0.000 |  |
| `crates/rustcode-review/rules/html.md` | 0 | 7 | 0.000 |  |
| `crates/rustcode-review/rules/java.md` | 0 | 35 | 0.000 |  |
| `crates/rustcode-review/rules/json.md` | 0 | 2 | 0.000 |  |
| `crates/rustcode-review/rules/kotlin.md` | 0 | 32 | 0.000 |  |
| `crates/rustcode-review/rules/lua.md` | 0 | 23 | 0.000 |  |
| `crates/rustcode-review/rules/makefile.md` | 0 | 7 | 0.000 |  |
| `crates/rustcode-review/rules/mapper_dao_xml.md` | 1 | 28 | 0.036 |  |
| `crates/rustcode-review/rules/markdown.md` | 0 | 11 | 0.000 |  |
| `crates/rustcode-review/rules/objc.md` | 0 | 23 | 0.000 |  |
| `crates/rustcode-review/rules/package_json.md` | 0 | 5 | 0.000 |  |
| `crates/rustcode-review/rules/perl.md` | 0 | 9 | 0.000 |  |
| `crates/rustcode-review/rules/php.md` | 0 | 15 | 0.000 |  |
| `crates/rustcode-review/rules/pom_xml.md` | 0 | 3 | 0.000 |  |
| `crates/rustcode-review/rules/properties.md` | 0 | 9 | 0.000 |  |
| `crates/rustcode-review/rules/protobuf.md` | 0 | 7 | 0.000 |  |
| `crates/rustcode-review/rules/python.md` | 0 | 34 | 0.000 |  |
| `crates/rustcode-review/rules/python_deps.md` | 1 | 10 | 0.100 |  |
| `crates/rustcode-review/rules/r.md` | 0 | 23 | 0.000 |  |
| `crates/rustcode-review/rules/ruby.md` | 0 | 15 | 0.000 |  |
| `crates/rustcode-review/rules/rust.md` | 0 | 16 | 0.000 |  |
| `crates/rustcode-review/rules/scala.md` | 0 | 22 | 0.000 |  |
| `crates/rustcode-review/rules/shell.md` | 1 | 13 | 0.077 |  |
| `crates/rustcode-review/rules/solidity.md` | 0 | 12 | 0.000 |  |
| `crates/rustcode-review/rules/sql.md` | 0 | 16 | 0.000 |  |
| `crates/rustcode-review/rules/swift.md` | 0 | 24 | 0.000 |  |
| `crates/rustcode-review/rules/terraform.md` | 0 | 9 | 0.000 |  |
| `crates/rustcode-review/rules/toml.md` | 0 | 6 | 0.000 |  |
| `crates/rustcode-review/rules/ts.md` | 0 | 38 | 0.000 |  |
| `crates/rustcode-review/rules/xml.md` | 0 | 7 | 0.000 |  |
| `crates/rustcode-review/rules/yaml.md` | 0 | 2 | 0.000 |  |

## C 类 · 逐字历史（跳过）（1）

| 路径 | Han 行 | 字母行 | ratio | 备注 |
|---|---|---|---|---|
| `extensions/jetbrains/CHANGELOG.md` | 0 | 8 | 0.000 |  |

## 待汉化 · EN（ratio == 0）（52）

| 路径 | Han 行 | 字母行 | ratio | 备注 |
|---|---|---|---|---|
| `.goals/rustcode-migration-finalize/inspector-feedback-1.md` | 0 | 31 | 0.000 |  |
| `crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/hooks-patterns.md` | 0 | 118 | 0.000 |  |
| `crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/mcp-servers.md` | 0 | 162 | 0.000 |  |
| `crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/plugins-reference.md` | 0 | 58 | 0.000 |  |
| `crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/skills-reference.md` | 0 | 242 | 0.000 |  |
| `crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/subagent-templates.md` | 0 | 108 | 0.000 |  |
| `crates/rustcode-kernel/SPIKE.md` | 0 | 59 | 0.000 |  |
| `docs/REFACTOR_SUMMARY.md` | 0 | 56 | 0.000 |  |
| `docs/acp-sdk-handler-notes.md` | 0 | 25 | 0.000 |  |
| `docs/adr/0003-runtime-owned-turn-execution-policy.md` | 0 | 27 | 0.000 |  |
| `docs/archive/2026-07-25-provider-retry-consolidation.md` | 0 | 49 | 0.000 |  |
| `docs/plans/2026-07-23-windows-qr-rendering.md` | 0 | 32 | 0.000 |  |
| `docs/plans/2026-07-24-busy-continue-fork.md` | 0 | 42 | 0.000 |  |
| `docs/plans/2026-07-25-subtasks-footer-panel.md` | 0 | 36 | 0.000 |  |
| `docs/plans/2026-07-26-atomgit-production-tools.md` | 0 | 29 | 0.000 |  |
| `docs/plans/2026-07-26-busy-continue-fork-gc.md` | 0 | 25 | 0.000 |  |
| `docs/plans/2026-07-26-model-cost-attribution.md` | 0 | 59 | 0.000 |  |
| `docs/plans/2026-07-26-provider-accounts-model-profiles-design.md` | 0 | 312 | 0.000 |  |
| `docs/plans/2026-07-26-provider-accounts-model-profiles-plan.md` | 0 | 214 | 0.000 |  |
| `docs/plans/2026-07-28-rewind-design.md` | 0 | 76 | 0.000 |  |
| `docs/plans/2026-07-28-rewind-implementation-plan.md` | 0 | 92 | 0.000 |  |
| `docs/plans/2026-07-31-native-runtime-datalog.md` | 0 | 57 | 0.000 |  |
| `docs/plans/2026-08-02-session-recovery-design.md` | 0 | 56 | 0.000 |  |
| `docs/plans/2026-08-02-session-recovery-implementation-plan.md` | 0 | 28 | 0.000 |  |
| `docs/plans/2026-08-06-internal-continuation-compaction-design.md` | 0 | 8 | 0.000 |  |
| `docs/plans/2026-08-07-config-panel-implementation-plan.md` | 0 | 49 | 0.000 |  |
| `docs/plans/2026-08-07-lightweight-lsp-phase-one.md` | 0 | 59 | 0.000 |  |
| `docs/plans/2026-08-07-tasks-long-line-rendering.md` | 0 | 25 | 0.000 |  |
| `docs/plans/2026-08-15-webui-blocking-interaction-dock.md` | 0 | 37 | 0.000 |  |
| `docs/plans/2026-08-17-project-input-history-design.md` | 0 | 49 | 0.000 |  |
| `docs/plans/2026-08-20-code-review-deep-mode-fanout-design.md` | 0 | 205 | 0.000 |  |
| `docs/plans/2026-08-20-code-review-deep-mode-fanout-plan.md` | 0 | 688 | 0.000 |  |
| `docs/plans/2026-08-20-code-review-deep-verify-phase2-plan.md` | 0 | 443 | 0.000 |  |
| `docs/plans/2026-08-23-deepseek-v4-flash-evaluation-design.md` | 0 | 76 | 0.000 |  |
| `docs/plans/2026-08-23-deepseek-v4-flash-evaluation.md` | 0 | 91 | 0.000 |  |
| `docs/security/permission-model.md` | 0 | 269 | 0.000 |  |
| `docs/superpowers/plans/2026-06-29-acp-agent.md` | 0 | 715 | 0.000 |  |
| `docs/superpowers/plans/2026-07-12-github-style-diff.md` | 0 | 410 | 0.000 |  |
| `docs/superpowers/specs/2026-04-23-p2-doctor-review-notebook-todo-design.md` | 0 | 56 | 0.000 |  |
| `docs/superpowers/specs/2026-06-29-acp-agent-design.md` | 0 | 215 | 0.000 |  |
| `docs/superpowers/specs/2026-07-22-batch-user-questions-persona-nudge-design.md` | 0 | 69 | 0.000 |  |
| `docs/superpowers/specs/2026-07-22-deepseek-skill-first-reminder-design.md` | 0 | 93 | 0.000 |  |
| `docs/superpowers/specs/2026-07-22-multi-question-request-user-input-design.md` | 0 | 124 | 0.000 |  |
| `docs/telemetry.md` | 0 | 36 | 0.000 |  |
| `docs/vscode-i18n-implementation-plan-2026-06-30.md` | 0 | 71 | 0.000 |  |
| `evals/deepseek-v4-flash/README.md` | 0 | 25 | 0.000 |  |
| `evals/deepseek-v4-flash/cases/agent-fixture/ARCHITECTURE.md` | 0 | 4 | 0.000 |  |
| `evals/deepseek-v4-flash/prompts/codex-judge.md` | 0 | 6 | 0.000 |  |
| `evals/deepseek-v4-flash/prompts/codex-report.md` | 0 | 12 | 0.000 |  |
| `extensions/jetbrains/PRIVACY.md` | 0 | 24 | 0.000 |  |
| `extensions/jetbrains/docs/jetbrains.md` | 0 | 84 | 0.000 |  |
| `webui/README.md` | 0 | 40 | 0.000 |  |

## 待汉化 · MIXED（0 < ratio < 0.5）（44）

| 路径 | Han 行 | 字母行 | ratio | 备注 |
|---|---|---|---|---|
| `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/04-review/T5.md` | 5 | 15 | 0.333 |  |
| `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/04-review/T6.md` | 4 | 14 | 0.286 |  |
| `.goals/rustcode-migration-finalize/summary.md` | 13 | 47 | 0.277 |  |
| `CONTEXT.md` | 15 | 45 | 0.333 |  |
| `README.md` | 1 | 594 | 0.002 |  |
| `crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/SKILL.md` | 46 | 240 | 0.192 |  |
| `crates/rustcode-review/LANGUAGES.md` | 1 | 79 | 0.013 |  |
| `crates/rustcode-tuix/tests/smoke.md` | 1 | 56 | 0.018 |  |
| `docs/REFACTOR_DESIGN_PHASE1.md` | 1 | 633 | 0.002 |  |
| `docs/archive/kernel-parity-backlog.md` | 6 | 50 | 0.120 |  |
| `docs/hooks.md` | 22 | 230 | 0.096 |  |
| `docs/plans/2026-07-28-provider-panel-ui-design.md` | 21 | 114 | 0.184 |  |
| `docs/plans/2026-08-07-request-user-input-review-design.md` | 1 | 30 | 0.033 |  |
| `docs/plans/2026-08-10-first-stage-planning-quality-design.md` | 1 | 47 | 0.021 |  |
| `docs/superpowers/plans/2026-04-19-tuix-retained-mode-rewrite.md` | 137 | 479 | 0.286 |  |
| `docs/superpowers/plans/2026-04-23-cadence-reflection.md` | 11 | 348 | 0.032 |  |
| `docs/superpowers/plans/2026-04-23-merge-current-task-into-cadence.md` | 90 | 296 | 0.304 |  |
| `docs/superpowers/plans/2026-05-08-vision-preprocessor-auto-config.md` | 1 | 554 | 0.002 |  |
| `docs/superpowers/plans/2026-05-08-vision-preprocessor.md` | 22 | 695 | 0.032 |  |
| `docs/superpowers/plans/2026-05-25-tuix-unified-in-app-scroll.md` | 62 | 1614 | 0.038 |  |
| `docs/superpowers/plans/2026-05-29-webui.md` | 521 | 1254 | 0.415 |  |
| `docs/superpowers/plans/2026-06-09-cache-friendly-compaction.md` | 2 | 397 | 0.005 |  |
| `docs/superpowers/plans/2026-06-27-v2-rate-limit-pause-resume.md` | 149 | 621 | 0.240 |  |
| `docs/superpowers/plans/2026-07-03-terminal-status-glyph.md` | 63 | 262 | 0.240 |  |
| `docs/superpowers/plans/2026-07-06-double-esc-undo-cooldown.md` | 59 | 169 | 0.349 |  |
| `docs/superpowers/plans/2026-07-11-persistent-todo-panel.md` | 6 | 593 | 0.010 |  |
| `docs/superpowers/plans/2026-07-13-selectable-approval.md` | 5 | 466 | 0.011 |  |
| `docs/superpowers/plans/2026-07-22-batch-user-questions-persona-nudge.md` | 1 | 82 | 0.012 |  |
| `docs/superpowers/plans/2026-07-22-brainstorming-request-user-input.md` | 2 | 134 | 0.015 |  |
| `docs/superpowers/plans/2026-07-22-deepseek-skill-first-reminder.md` | 1 | 231 | 0.004 |  |
| `docs/superpowers/plans/2026-07-22-multi-question-request-user-input.md` | 2 | 446 | 0.004 |  |
| `docs/superpowers/plans/2026-07-22-request-user-input-custom-and-submit-spacing.md` | 1 | 290 | 0.003 |  |
| `docs/superpowers/plans/2026-07-24-skills-multi-compose.md` | 83 | 194 | 0.428 |  |
| `docs/superpowers/plans/2026-07-24-windows-native-tls-schannel-fallback.md` | 58 | 121 | 0.479 |  |
| `docs/superpowers/plans/2026-07-25-retire-core-provider-A-compact.md` | 47 | 128 | 0.367 |  |
| `docs/superpowers/plans/2026-07-25-retire-core-provider-B-vision.md` | 93 | 352 | 0.264 |  |
| `docs/superpowers/plans/2026-07-25-round-cap-checkpoint.md` | 171 | 492 | 0.348 |  |
| `docs/superpowers/plans/2026-07-29-webui-sync-compact.md` | 83 | 169 | 0.491 |  |
| `docs/superpowers/plans/2026-07-31-local-scheduled-tasks-phase1.md` | 83 | 329 | 0.252 |  |
| `docs/superpowers/plans/2026-07-31-local-scheduled-tasks-phase2.md` | 76 | 229 | 0.332 |  |
| `docs/superpowers/plans/2026-07-31-project-memory-dir-override.md` | 55 | 133 | 0.414 |  |
| `docs/superpowers/specs/2026-07-22-brainstorming-request-user-input-design.md` | 1 | 94 | 0.011 |  |
| `docs/superpowers/specs/2026-07-22-request-user-input-custom-and-submit-spacing-design.md` | 3 | 94 | 0.032 |  |
| `docs/webhook-guide.md` | 135 | 278 | 0.486 |  |

## 已中文 · ZH（ratio >= 0.5）（138）

| 路径 | Han 行 | 字母行 | ratio | 备注 |
|---|---|---|---|---|
| `.claude/plans/atomgit-decouple.md` | 40 | 33 | 1.212 |  |
| `.codebuddy/agents/code-implementer.md` | 45 | 51 | 0.882 |  |
| `.codebuddy/agents/code-reviewer.md` | 55 | 50 | 1.100 |  |
| `.codebuddy/agents/doc-writer.md` | 41 | 38 | 1.079 |  |
| `.codebuddy/agents/project-manager.md` | 72 | 58 | 1.241 |  |
| `.codebuddy/agents/requirements-analyst.md` | 37 | 43 | 0.860 |  |
| `.codebuddy/agents/solution-architect.md` | 45 | 57 | 0.789 |  |
| `.codebuddy/agents/test-engineer.md` | 45 | 50 | 0.900 |  |
| `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/00-requirement.md` | 259 | 249 | 1.040 |  |
| `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/01-design.md` | 231 | 252 | 0.917 |  |
| `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/02-tasks.md` | 163 | 209 | 0.780 |  |
| `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/03-impl/T1.md` | 26 | 47 | 0.553 |  |
| `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/03-impl/T3.md` | 24 | 33 | 0.727 |  |
| `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/03-impl/T4.md` | 17 | 24 | 0.708 |  |
| `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/03-impl/T5.md` | 22 | 39 | 0.564 |  |
| `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/03-impl/T6.md` | 14 | 25 | 0.560 |  |
| `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/03-impl/T7-dead-code-scan.md` | 35 | 40 | 0.875 |  |
| `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/03-impl/T9-integration-verification.md` | 49 | 57 | 0.860 |  |
| `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/04-review/T1.md` | 33 | 33 | 1.000 |  |
| `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/04-review/T3.md` | 12 | 20 | 0.600 |  |
| `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/04-review/T4.md` | 8 | 15 | 0.533 |  |
| `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/04-review/T7.md` | 8 | 13 | 0.615 |  |
| `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/04-review/T9.md` | 12 | 17 | 0.706 |  |
| `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/06-release.md` | 67 | 94 | 0.713 |  |
| `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/STATUS.md` | 81 | 72 | 1.125 |  |
| `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/03-impl/T6-docs-atomcode.md` | 190 | 181 | 1.050 |  |
| `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/03-impl/T7-goals-superpowers.md` | 253 | 250 | 1.012 |  |
| `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/03-impl/T8-artifacts.md` | 72 | 74 | 0.973 |  |
| `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/05-test-report.md` | 182 | 169 | 1.077 |  |
| `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/06-release.md` | 196 | 209 | 0.938 |  |
| `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/HANDOFF-codingplan-legacy.md` | 64 | 57 | 1.123 |  |
| `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/STATUS.md` | 225 | 207 | 1.087 |  |
| `.codebuddy/artifacts/2026-09-02-strip-atomcode/STATUS.md` | 46 | 35 | 1.314 |  |
| `.codebuddy/artifacts/2026-09-03-banner-release/03-impl/T1.md` | 54 | 60 | 0.900 |  |
| `.codebuddy/artifacts/2026-09-03-banner-release/03-impl/T2.md` | 78 | 100 | 0.780 |  |
| `.codebuddy/artifacts/2026-09-03-banner-release/STATUS.md` | 220 | 196 | 1.122 |  |
| `.codebuddy/artifacts/2026-09-03-doc-consistency/STATUS.md` | 47 | 43 | 1.093 |  |
| `.codebuddy/artifacts/2026-09-03-git-wrapup/01-plan.md` | 201 | 191 | 1.052 |  |
| `.codebuddy/artifacts/2026-09-03-git-wrapup/02-tasks.md` | 122 | 126 | 0.968 |  |
| `.codebuddy/artifacts/2026-09-03-git-wrapup/03-impl/GW-02.md` | 66 | 65 | 1.015 |  |
| `.codebuddy/artifacts/2026-09-03-git-wrapup/03-impl/GW-03.md` | 77 | 98 | 0.786 |  |
| `.codebuddy/artifacts/2026-09-03-git-wrapup/STATUS.md` | 129 | 116 | 1.112 |  |
| `.codebuddy/artifacts/2026-09-03-residual-two-items/STATUS.md` | 60 | 54 | 1.111 |  |
| `.codebuddy/rules/multi-agent-workflow.md` | 44 | 38 | 1.158 |  |
| `.goals/rustcode-migration-finalize/goal.md` | 36 | 58 | 0.621 |  |
| `.superpowers/pr/feat-rust-tui-selection-session-preview.md` | 72 | 68 | 1.059 |  |
| `AGENTS.md` | 565 | 576 | 0.981 | OWNED_ELSEWHERE（owner 非 doc-writer） |
| `DEVENV.md` | 23 | 25 | 0.920 |  |
| `README.zh-CN.md` | 415 | 443 | 0.937 | OWNED_ELSEWHERE（owner 非 doc-writer） |
| `crates/rustcode-capabilities/README.md` | 42 | 40 | 1.050 |  |
| `crates/rustcode-clix/README.md` | 109 | 147 | 0.741 |  |
| `crates/rustcode-coding/README.md` | 42 | 54 | 0.778 |  |
| `crates/rustcode-daemon/README.md` | 174 | 348 | 0.500 |  |
| `crates/rustcode-kernel/README.md` | 20 | 23 | 0.870 |  |
| `crates/rustcode-review/README.md` | 35 | 62 | 0.565 |  |
| `docker/README.md` | 78 | 118 | 0.661 |  |
| `docs/HOOK_DOC_UPDATE_SPEC.md` | 208 | 206 | 1.010 |  |
| `docs/adr/0001-runtime-owned-session-transitions.md` | 2 | 3 | 0.667 |  |
| `docs/adr/0002-runtime-owned-dynamic-mcp-tool-catalog.md` | 13 | 17 | 0.765 |  |
| `docs/agent-api-rfc.md` | 153 | 167 | 0.916 |  |
| `docs/architecture.md` | 117 | 129 | 0.907 |  |
| `docs/archive/2026-07-27-models-dev-pricing-design.md` | 37 | 25 | 1.480 |  |
| `docs/archive/coding-runtime-incremental-migration.md` | 760 | 975 | 0.779 |  |
| `docs/archive/coding-runtime-native-migration-design.md` | 480 | 745 | 0.644 |  |
| `docs/archive/live-transport-convergence-plan.md` | 124 | 118 | 1.051 |  |
| `docs/archive/release-v5.0.1-current-branch-change-report.md` | 161 | 154 | 1.045 |  |
| `docs/archive/release-v5.0.3-core-retirement-acceptance.md` | 235 | 249 | 0.944 |  |
| `docs/archive/session-convergence-plan.md` | 596 | 564 | 1.057 |  |
| `docs/archive/v5.0.0-retire-bridge-core-progress.md` | 78 | 71 | 1.099 |  |
| `docs/async-webhook-guide.md` | 164 | 248 | 0.661 |  |
| `docs/async-webhook-summary.md` | 159 | 178 | 0.893 |  |
| `docs/codex-claude-config-analysis.md` | 31 | 52 | 0.596 |  |
| `docs/compact-durable-checkpoint-design.md` | 96 | 91 | 1.055 |  |
| `docs/compact-native-migration-retrospective.md` | 288 | 296 | 0.973 |  |
| `docs/custom-endpoint-guide.md` | 50 | 29 | 1.724 |  |
| `docs/dev-env-setup.md` | 77 | 143 | 0.538 |  |
| `docs/features.md` | 61 | 81 | 0.753 |  |
| `docs/hook-architecture.md` | 75 | 139 | 0.540 |  |
| `docs/hook-cli-guide.md` | 136 | 252 | 0.540 |  |
| `docs/hook-expansion-summary.md` | 144 | 142 | 1.014 |  |
| `docs/hook-implementation-summary.md` | 89 | 92 | 0.967 |  |
| `docs/hook-timing-complete.md` | 119 | 101 | 1.178 |  |
| `docs/i18n-field-mapping.md` | 72 | 83 | 0.867 |  |
| `docs/i18n-style.md` | 79 | 59 | 1.339 |  |
| `docs/mcp-rmcp-feasibility.md` | 116 | 107 | 1.084 |  |
| `docs/mcp.md` | 110 | 132 | 0.833 |  |
| `docs/mcp/github.md` | 67 | 112 | 0.598 |  |
| `docs/multi-agent-collaboration-solution.md` | 186 | 180 | 1.033 |  |
| `docs/phase1-refactor-design.md` | 367 | 436 | 0.842 |  |
| `docs/phase2-subagent-status.md` | 152 | 161 | 0.944 |  |
| `docs/plans/2026-08-11-provider-form-horizontal-editing-design.md` | 21 | 11 | 1.909 |  |
| `docs/plans/2026-08-11-todo-agent-body-projection-design.md` | 24 | 20 | 1.200 |  |
| `docs/plans/2026-08-14-webui-browser-notification-design.md` | 153 | 151 | 1.013 |  |
| `docs/plans/2026-08-17-acp-sdk-2.0-protocol-roadmap.md` | 260 | 243 | 1.070 |  |
| `docs/plans/2026-08-21-external-agent-subagent-drivers-spec.md` | 171 | 179 | 0.955 |  |
| `docs/plans/2026-08-23-headless-eval-controls.md` | 27 | 22 | 1.227 |  |
| `docs/platform-neutralization.md` | 200 | 195 | 1.026 |  |
| `docs/pr-descriptions/2026-04-27-vscode-extension-ui-mcp.md` | 102 | 124 | 0.823 |  |
| `docs/pr-hook-test-command.md` | 40 | 59 | 0.678 |  |
| `docs/superpowers/2026-07-27-release-v5.0.3-test-checklist.md` | 91 | 84 | 1.083 |  |
| `docs/superpowers/plans/2026-04-19-tuix-ink-cell-diff.md` | 73 | 118 | 0.619 |  |
| `docs/superpowers/plans/2026-04-23-agent-harness-principles.md` | 107 | 121 | 0.884 |  |
| `docs/superpowers/plans/2026-07-24-retire-core-conversation-tui-port.md` | 120 | 181 | 0.663 |  |
| `docs/superpowers/plans/2026-07-25-retire-core-provider-C2-conversation-transport.md` | 62 | 64 | 0.969 |  |
| `docs/superpowers/plans/2026-07-25-retire-core-tool-ball-D.md` | 54 | 93 | 0.581 |  |
| `docs/superpowers/specs/2026-05-08-vision-preprocessor-design.md` | 115 | 157 | 0.732 |  |
| `docs/superpowers/specs/2026-05-25-tuix-unified-in-app-scroll-design.md` | 183 | 225 | 0.813 |  |
| `docs/superpowers/specs/2026-05-29-provider-add-simplify-design.md` | 138 | 134 | 1.030 |  |
| `docs/superpowers/specs/2026-05-29-webui-design.md` | 162 | 144 | 1.125 |  |
| `docs/superpowers/specs/2026-06-07-headless-output-format-json-design.md` | 67 | 95 | 0.705 |  |
| `docs/superpowers/specs/2026-06-09-cache-friendly-compaction-design.md` | 79 | 58 | 1.362 |  |
| `docs/superpowers/specs/2026-06-27-v2-rate-limit-pause-resume-design.md` | 80 | 96 | 0.833 |  |
| `docs/superpowers/specs/2026-07-03-terminal-status-glyph-design.md` | 75 | 77 | 0.974 |  |
| `docs/superpowers/specs/2026-07-06-double-esc-undo-cooldown-design.md` | 44 | 42 | 1.048 |  |
| `docs/superpowers/specs/2026-07-11-persistent-todo-panel-design.md` | 113 | 91 | 1.242 |  |
| `docs/superpowers/specs/2026-07-13-selectable-approval-design.md` | 74 | 64 | 1.156 |  |
| `docs/superpowers/specs/2026-07-24-retire-core-conversation-tui-port-design.md` | 59 | 58 | 1.017 |  |
| `docs/superpowers/specs/2026-07-24-skills-multi-compose-design.md` | 65 | 60 | 1.083 |  |
| `docs/superpowers/specs/2026-07-24-windows-native-tls-schannel-fallback-design.md` | 56 | 53 | 1.057 |  |
| `docs/superpowers/specs/2026-07-25-retire-core-provider-A-compact-design.md` | 38 | 32 | 1.188 |  |
| `docs/superpowers/specs/2026-07-25-retire-core-provider-B-vision-design.md` | 46 | 42 | 1.095 |  |
| `docs/superpowers/specs/2026-07-25-retire-core-provider-C-conversation-design.md` | 49 | 45 | 1.089 |  |
| `docs/superpowers/specs/2026-07-25-retire-core-tool-ball-D-design.md` | 44 | 39 | 1.128 |  |
| `docs/superpowers/specs/2026-07-25-round-cap-checkpoint-design.md` | 92 | 72 | 1.278 |  |
| `docs/superpowers/specs/2026-07-29-user-input-background-block-design.md` | 61 | 49 | 1.245 |  |
| `docs/superpowers/specs/2026-07-29-webui-sync-compact-design.md` | 66 | 66 | 1.000 |  |
| `docs/superpowers/specs/2026-07-30-progress-signposts-preamble-design.md` | 45 | 40 | 1.125 |  |
| `docs/superpowers/specs/2026-07-30-workflow-intent-understanding-design.md` | 53 | 56 | 0.946 |  |
| `docs/superpowers/specs/2026-07-31-local-scheduled-tasks-design.md` | 64 | 62 | 1.032 |  |
| `docs/superpowers/specs/2026-07-31-local-scheduled-tasks-phase2-design.md` | 60 | 63 | 0.952 |  |
| `docs/superpowers/specs/2026-07-31-project-memory-dir-override-design.md` | 47 | 50 | 0.940 |  |
| `docs/target-architecture.md` | 68 | 68 | 1.000 |  |
| `docs/testing/release-v5.0.0-acceptance.md` | 172 | 145 | 1.186 |  |
| `docs/testing/windows-path-normalization.md` | 62 | 59 | 1.051 |  |
| `docs/webhook-implementation-summary.md` | 118 | 183 | 0.645 |  |
| `extensions/jetbrains/README.md` | 113 | 139 | 0.813 |  |
| `extensions/vscode/README.md` | 78 | 42 | 1.857 |  |
| `packages/npm/README.md` | 18 | 23 | 0.783 |  |
