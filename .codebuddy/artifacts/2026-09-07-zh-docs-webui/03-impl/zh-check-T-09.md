# gate 报告

- 基线 (--base): `3ee655e3`

## PASS AC-1 清单自洽

- `SKIP_A=4 SKIP_B=47 SKIP_C=1 SKIP_D=2 TODO=29 ZH=202 total=285`

## PASS AC-2/3/4/6/7b/32 全量 check

- `全量 check 受检 231，FAIL 0`
- `AC-4 授权放行合计 12 条（D1 11 / D2 1）`
- `  .codebuddy/agents/doc-writer.md | removed=README.zh-CN.md | added=- | cause=D1`
- `  AGENTS.md | removed=README.zh-CN.md | added=- | cause=D1`
- `  AGENTS.md | removed=README.zh-CN.md | added=- | cause=D1`
- `  AGENTS.md | removed=README.zh-CN.md:94 | added=- | cause=D1`
- `  README.md | removed=- | added=./scripts/build-webui.sh | cause=D2`
- `  docs/codex-claude-config-analysis.md | removed=README.zh-CN.md | added=- | cause=D1`
- `  docs/phase1-refactor-design.md | removed=README.zh-CN.md | added=- | cause=D1`
- `  docs/phase1-refactor-design.md | removed=README.zh-CN.md:151 | added=- | cause=D1`
- `  docs/phase1-refactor-design.md | removed=[2] README.md / README.zh-CN.md   零遥测口径统一；清除 Emoji；补"自定义网关"配置章节 | added=[2] README.md   零遥测口径统一；清除 Emoji；补"自定义网关"配置章节 | cause=D1`
- `  docs/platform-neutralization.md | removed=README.zh-CN.md | added=- | cause=D1`
- `  docs/superpowers/plans/2026-05-29-webui.md | removed=README.zh-CN.md | added=- | cause=D1`
- `  docs/superpowers/plans/2026-05-29-webui.md | removed=git add webui/ crates/ README.md README.zh-CN.md | added=git add webui/ crates/ README.md | cause=D1`

## PASS AC-5 运行时载荷

- `AC-5a rules 目录无改动: OK`
- `AC-5b setup-seeds 有改动 (6 个文件): OK`
- `AC-5b 无 ^[+-]name: 变更: OK`

## PASS AC-7a 无改名/删除

- `AC-7a 无 md 删除/重命名（README.zh-CN.md 与 A/C 类除外）: OK`

## PASS AC-8 外链残留

- `段 1 正式域（候选 515，已排除 .codebuddy/artifacts/）：0 命中 -> OK`
- `段 2 历史域（候选 37）：命中 39 处，全部位于 .codebuddy/artifacts/ -> OK（历史痕迹仍在）`
- `段 3 兜底域 site/.github/docs/extensions（全扩展名）：0 命中 -> OK`

## 附录 · 被 R5 跳过的行（供抽检）

共 56 行。

- `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/05-test-report.md:245`: `     | 文件 | 差异性质 | 语义影响 |`
- `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/05-test-report.md:246`: `     |---|---|---|`
- `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/05-test-report.md:247`: `     | 'tuix/event_loop/commands.rs' | 删除单参调用冗余尾随逗号（'t(..),)' → 't(..))'） | 无 |`
- `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/05-test-report.md:248`: `     | 'tuix/event_loop/mod.rs' | 结构体字面量尾随逗号增删 | 无 |`
- `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/05-test-report.md:249`: `     | 'tuix/modals/dir_picker.rs' | 多行调用末参补尾随逗号 | 无 |`
- `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/05-test-report.md:250`: `     | 'tuix/test_term.rs' | rustfmt 'merge_derives' 合并相邻 '#[derive]' | 无 |`
- `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/HANDOFF-codingplan-legacy.md:103`: `     + 'event_loop::tool_format_tests::summarise_*' ×4`
- `.codebuddy/artifacts/2026-09-03-git-wrapup/STATUS.md:8`: `    - '?? .codebuddy/artifacts/2026-09-03-git-wrapup/'（本轮交接件）`
- `.goals/rustcode-migration-finalize/goal.md:32`: `    'mcp::registry::tests::trust_key_golden_matches_core_algorithm' 除外)`
- `AGENTS.md:559`: `    'result_non_task_output_falls_back' 与 cli 的 'acp::translate' 共 **6 个用例**同为`
- `AGENTS.md:565`: `    是**刻意与 locale 无关**的测试——用 'i18n::t(Msg::TuixFoldLinesSuffix{count})' 现算期望后缀再比对,`
- `AGENTS.md:567`: `    反而**新增 2 个红**:(1) 该用例因我增多的 En 设置翻转了全局 locale,使两次调用拿到不同 locale 而失配;`
- `AGENTS.md:571`: `    断言 '!label.contains("987")'(987654 为其「仅记账」哨兵值),但标签渲染了会话 ID`
- `AGENTS.md:576`: `    **115/1 → 116/0**;'rustcode-review --lib' **100/0**(第三十轮);'coding' 430/0;'updater' 41/0;`
- `AGENTS.md:584`: `    **唯一失败即 'trust_key_golden_matches_core_algorithm'**('AGENTS.md:226' 文档化已知红,`
- `AGENTS.md:586`: `    执行要点:后台 'setsid' + 唯一日志路径,启动前确认 'pgrep -c cargo = 0',`
- `AGENTS.md:601`: `    教训:'search_content' 工具的 'count' 模式**会低估**,统计残留面**必须用 shell 精确计数**,`
- `AGENTS.md:607`: `    'docs/{UPSTREAM_RUSTCODE_LICENSE,UPSTREAM_CREDITS,THIRD_PARTY_NOTICES,ORIGINAL_LICENSE}.md';`
- `AGENTS.md:608`: `    ② **门禁定义必留** —— G7/G8 那几行**本身就是 grep 模式**,删掉 'atomcode' 字样会让门禁失效`
- `AGENTS.md:612`: `    'docs/{features,platform-neutralization}.md' 正文中的上游 slug 'atomgit_atomcode/atomcode'`
- `AGENTS.md:613`: `    改为「上游项目」,**并保留指向 'ORIGINAL_LICENSE.md' / 'UPSTREAM_CREDITS.md' 的归属链接**;`
- `AGENTS.md:633`: `    ('cargo test -p atomcode-{tuix,capabilities,daemon}' → 'rustcode-*',及任务书未点名的`
- `AGENTS.md:644`: `    'cargo fmt --all -- --check' / 'cargo clippy --workspace --all-targets' / 'cargo test --workspace',`
- `AGENTS.md:646`: `    格式违规,**使 CI 的 'fmt' job 由红转绿**;② CI 'test' job 跑裸 'cargo test --workspace',`
- `AGENTS.md:647`: `    而 'trust_key' 确定性失败,**该 job 仍将为红**(既有状态,非本轮引入);③ 'clippy' job 未加`
- `AGENTS.md:651`: `    ('cli/src/main.rs:3363')存活**,与对方删除在同一文件共存;G1 **exit=0 / 0 差异**;`
- `README.md:549`: `>`
- `README.md:551`: `>`
- `crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/SKILL.md:219`: `    "[name]": {`
- `crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/SKILL.md:220`: `      "command": "npx",`
- `crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/SKILL.md:221`: `      "args": ["-y", "@package/name"]`
- `crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/SKILL.md:222`: `    }`
- `crates/rustcode-kernel/SPIKE.md:74`: `    cargo test -p rustcode-kernel`
- `crates/rustcode-kernel/SPIKE.md:75`: `    cargo run -p rustcode-kernel --example minimal_specialization`
- `docker/README.md:188`: `>`
- `docs/platform-neutralization.md:38`: `    ('crates/rustcode-cli/src/main.rs:5081')`
- `docs/platform-neutralization.md:49`: `    ('crates/rustcode-config/src/config/provider_preset.rs:303')`
- `docs/platform-neutralization.md:75`: `    'uninstall.sh' / 'uninstall.ps1' 头部 'curl|sh' / 'irm' 厂商 URL 改本地 / 发行`
- `docs/platform-neutralization.md:157`: `    status-glyph*.md' 等 3 个文件)里的 '🟢/🟡/🔴' 是该彩色状态点特性的规格主语义`
- `docs/platform-neutralization.md:168`: `    ('crates/rustcode-capabilities/src/cc_hooks.rs:1144'):测试自身缺陷——同一 '&&'`
- `docs/platform-neutralization.md:169`: `    链中两个 'grep' 共享 stdin,首个 grep 抽干管道致第二个必失败;改为先`
- `docs/platform-neutralization.md:172`: `    ('crates/rustcode-capabilities/src/tools/read.rs:1276'):断言停留在旧 300 行分页,`
- `docs/platform-neutralization.md:173`: `    实现已为 1500 行页('DEFAULT_READ_LIMIT','crates/rustcode-capabilities/src/`
- `docs/platform-neutralization.md:174`: `    tools/read.rs:25');夹具改为 1600 行并对齐`
- `docs/platform-neutralization.md:177`: `    'SpawnFailed("Text file busy (os error 26)")'(overlayfs / 容器内新写脚本`
- `docs/platform-neutralization.md:178`: `    execve 的 ETXTBSY 竞态;Go fork/exec 内置重试而 Rust std 没有):新增`
- `docs/platform-neutralization.md:179`: `    'process_utils::spawn_retrying_etxtbsy'(8 次退避重试,`
- `docs/platform-neutralization.md:180`: `    'crates/rustcode-capabilities/src/process_utils.rs:204'),接入`
- `docs/platform-neutralization.md:181`: `    'subagent::proc::ManagedChild::spawn'('.../subagent/proc.rs:146')与`
- `docs/superpowers/specs/2026-05-29-provider-add-simplify-design.md:180`: `    model = body 里 '"model"' 值`
- `docs/testing/windows-path-normalization.md:25`: `      → 正常运行(过去会 'python C:sers...' 找不到)`
- `docs/testing/windows-path-normalization.md:45`: `      (过去 '\\?\C:\..' 传给 'cmd start' 打不开)`
- `docs/testing/windows-path-normalization.md:48`: `      '/context' / footer 的 cwd 正常`
- `docs/testing/windows-path-normalization.md:50`: `      'fs_mkdir')→ 选它 → 该目录的会话在 **webui 和 TUI 两边是同一个**`
- `docs/testing/windows-path-normalization.md:56`: `      读 / grep → **不再被拒**(过去每个绝对路径都报 "outside the review repository")`
- `docs/testing/windows-path-normalization.md:74`: `      也接受,但实测 'type' / 'cd' / python 等是否 OK(若有问题需单独处理)`

# zh-check 报告

- 基线 (--base): `3ee655e3`
- 阈值 (--en-ratio): `0.05`
- 判定: AC-2 / AC-4 / AC-6 / AC-7b / AC-32，任一失败即 FAIL（fail-closed）

## 汇总

- 受检文件: 231
- PASS: 231
- FAIL: 0

## PASS `.claude/plans/atomgit-decouple.md`

- en=0/40 ratio=0.0000

## PASS `.codebuddy/agents/code-implementer.md`

- en=0/44 ratio=0.0000

## PASS `.codebuddy/agents/code-reviewer.md`

- en=0/53 ratio=0.0000

## PASS `.codebuddy/agents/doc-writer.md`

- en=0/41 ratio=0.0000
- AC-4-authorized: `.codebuddy/agents/doc-writer.md | removed=README.zh-CN.md | added=- | cause=D1`
- AC-6-desc-changed: `description: '文档与交付说明专家。在集成测试通过（G5 达成）后调用：同步架构与设计文档、更新受影响 README 与 docs、编写 CHANGELOG 与发布说明、整理行为变化/风险/验证结果/已知未验证范围四段式交付清单。触发示例：05-test-report.md 通过需要出交付说明；公共协议或配置项发生变更；需要补写或修订 docs 下的设计文档；发布前需要 CHANGELOG 与回滚说明。禁止修改源码与测试，禁止执行构建或测试命令，禁止编造未验证的结论。' => '文档与交付说明专家。在集成测试通过（G5 达成）后调用：同步架构与设计文档、更新受影响 README 与 docs、编写 CHANGELOG 与发布说明、整理行为变化/风险/验证结果/已知未验证范围四段式交付清单。触发示例：05-test-report.md 通过需要出交付说明；公共协议或配置项发生变更；需要补写或修订 docs 下的设计文档；发布前需要 CHANGELOG 与回滚说明。禁止修改源码与测试，禁止执行 cargo/npm 构建与测试命令（但**允许且应当**运行 python3 scripts/check-zh-docs.py 做文档自检），禁止编造未验证的结论。'`

## PASS `.codebuddy/agents/project-manager.md`

- en=0/63 ratio=0.0000

## PASS `.codebuddy/agents/requirements-analyst.md`

- en=0/36 ratio=0.0000

## PASS `.codebuddy/agents/solution-architect.md`

- en=0/47 ratio=0.0000

## PASS `.codebuddy/agents/test-engineer.md`

- en=0/44 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/00-requirement.md`

- en=0/284 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/01-design.md`

- en=0/246 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/02-tasks.md`

- en=7/173 ratio=0.0405
- AC-3 残留行清单 (en 行，共 7 行):
  - `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/02-tasks.md:34`: `| **crate** | 'rustcode-codingplan'、'rustcode-config' |`
  - `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/02-tasks.md:35`: `| **files_owned** | 'crates/rustcode-codingplan/src/lib.rs'<br>'crates/rustcode-codingplan/src/types.rs'<br>'crates/rustcode-codingplan/src/client.rs'<br>'crates/rustcode-codingplan/src/setup.rs'<br>'crates/rustcode-codingplan/src/sync_marker.rs'<br>'crates/rustcode-config/src/i18n/messages.rs' |`
  - `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/02-tasks.md:54`: `| **crate** | 'rustcode-cli' |`
  - `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/02-tasks.md:55`: `| **files_owned** | 'crates/rustcode-cli/src/main.rs' |`
  - `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/02-tasks.md:76`: `| **files_owned** | 'docs/config.example.toml' |`
  - `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/02-tasks.md:156`: `| **files_owned** | '.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/03-impl/T9-integration-verification.md' |`
  - `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/02-tasks.md:176`: `| **files_owned** | 'AGENTS.md'（**dirty**） |`

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/03-impl/T1.md`

- en=0/32 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/03-impl/T3.md`

- en=0/24 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/03-impl/T4.md`

- en=0/15 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/03-impl/T5.md`

- en=0/19 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/03-impl/T6.md`

- en=0/12 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/03-impl/T7-dead-code-scan.md`

- en=0/33 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/03-impl/T9-integration-verification.md`

- en=0/44 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/04-review/T1.md`

- en=0/33 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/04-review/T3.md`

- en=0/10 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/04-review/T4.md`

- en=0/8 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/04-review/T5.md`

- en=0/6 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/04-review/T6.md`

- en=0/5 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/04-review/T7.md`

- en=0/8 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/04-review/T9.md`

- en=0/12 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/06-release.md`

- en=0/62 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/STATUS.md`

- en=0/81 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/03-impl/T6-docs-atomcode.md`

- en=2/194 ratio=0.0103
- AC-3 残留行清单 (en 行，共 2 行):
  - `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/03-impl/T6-docs-atomcode.md:284`: `| docs/features.md:113-114 | 'G7  crates/scripts/.github 无 atomcode 残留' / 'G8  docs/architecture.md 无 atomcode 残留' |`
  - `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/03-impl/T6-docs-atomcode.md:285`: `| docs/platform-neutralization.md:204-206 | 'G7  crates/ scripts/ .github/ 无 atomcode 残留(atomgit feature / 旧前缀兼容 /' / '    fork 发行主页三类除外)' / 'G8  docs/architecture.md 无 atomcode 残留' |`

## PASS `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/03-impl/T7-goals-superpowers.md`

- en=2/264 ratio=0.0076
- AC-3 残留行清单 (en 行，共 2 行):
  - `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/03-impl/T7-goals-superpowers.md:370`: `| 'inspector-feedback-1.md:11' | ''| 1 | G7: atomcode in crates/scripts/.github = 0 | PASS | 'grep -rn "atomcode" crates/ scripts/ .github/' = 0 hits |'' |`
  - `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/03-impl/T7-goals-superpowers.md:371`: `| 'inspector-feedback-1.md:12' | ''| 2 | G8: atomcode in docs/architecture.md = 0 | PASS | 'grep -rn "atomcode" docs/architecture.md' = 0 hits |'' |`

## PASS `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/03-impl/T8-artifacts.md`

- en=0/79 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/05-test-report.md`

- en=0/181 ratio=0.0000
- R5 跳过的行 (共 6 行):
  - `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/05-test-report.md:245`: `     | 文件 | 差异性质 | 语义影响 |`
  - `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/05-test-report.md:246`: `     |---|---|---|`
  - `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/05-test-report.md:247`: `     | 'tuix/event_loop/commands.rs' | 删除单参调用冗余尾随逗号（'t(..),)' → 't(..))'） | 无 |`
  - `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/05-test-report.md:248`: `     | 'tuix/event_loop/mod.rs' | 结构体字面量尾随逗号增删 | 无 |`
  - `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/05-test-report.md:249`: `     | 'tuix/modals/dir_picker.rs' | 多行调用末参补尾随逗号 | 无 |`
  - `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/05-test-report.md:250`: `     | 'tuix/test_term.rs' | rustfmt 'merge_derives' 合并相邻 '#[derive]' | 无 |`

## PASS `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/06-release.md`

- en=0/200 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/HANDOFF-codingplan-legacy.md`

- en=1/77 ratio=0.0130
- AC-3 残留行清单 (en 行，共 1 行):
  - `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/HANDOFF-codingplan-legacy.md:80`: `| coding --lib / updater --lib | 430/0、41/0 |`
- R5 跳过的行 (共 1 行):
  - `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/HANDOFF-codingplan-legacy.md:103`: `     + 'event_loop::tool_format_tests::summarise_*' ×4`

## PASS `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/STATUS.md`

- en=0/247 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-strip-atomcode/STATUS.md`

- en=0/46 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-03-banner-release/03-impl/T1.md`

- en=0/58 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-03-banner-release/03-impl/T2.md`

- en=0/87 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-03-banner-release/STATUS.md`

- en=0/235 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-03-doc-consistency/STATUS.md`

- en=0/50 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-03-git-wrapup/01-plan.md`

- en=0/194 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-03-git-wrapup/02-tasks.md`

- en=0/122 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-03-git-wrapup/03-impl/GW-02.md`

- en=0/66 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-03-git-wrapup/03-impl/GW-03.md`

- en=0/80 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-03-git-wrapup/STATUS.md`

- en=0/139 ratio=0.0000
- R5 跳过的行 (共 1 行):
  - `.codebuddy/artifacts/2026-09-03-git-wrapup/STATUS.md:8`: `    - '?? .codebuddy/artifacts/2026-09-03-git-wrapup/'（本轮交接件）`

## PASS `.codebuddy/artifacts/2026-09-03-residual-two-items/STATUS.md`

- en=0/64 ratio=0.0000

## PASS `.codebuddy/rules/multi-agent-workflow.md`

- en=0/43 ratio=0.0000

## PASS `.goals/rustcode-migration-finalize/goal.md`

- en=0/59 ratio=0.0000
- R5 跳过的行 (共 1 行):
  - `.goals/rustcode-migration-finalize/goal.md:32`: `    'mcp::registry::tests::trust_key_golden_matches_core_algorithm' 除外)`

## PASS `.goals/rustcode-migration-finalize/inspector-feedback-1.md`

- en=0/31 ratio=0.0000

## PASS `.goals/rustcode-migration-finalize/summary.md`

- en=0/50 ratio=0.0000

## PASS `.superpowers/pr/feat-rust-tui-selection-session-preview.md`

- en=0/81 ratio=0.0000

## PASS `AGENTS.md`

- en=0/540 ratio=0.0000
- AC-4-authorized: `AGENTS.md | removed=README.zh-CN.md | added=- | cause=D1`
- AC-4-authorized: `AGENTS.md | removed=README.zh-CN.md | added=- | cause=D1`
- AC-4-authorized: `AGENTS.md | removed=README.zh-CN.md:94 | added=- | cause=D1`
- R5 跳过的行 (共 17 行):
  - `AGENTS.md:559`: `    'result_non_task_output_falls_back' 与 cli 的 'acp::translate' 共 **6 个用例**同为`
  - `AGENTS.md:565`: `    是**刻意与 locale 无关**的测试——用 'i18n::t(Msg::TuixFoldLinesSuffix{count})' 现算期望后缀再比对,`
  - `AGENTS.md:567`: `    反而**新增 2 个红**:(1) 该用例因我增多的 En 设置翻转了全局 locale,使两次调用拿到不同 locale 而失配;`
  - `AGENTS.md:571`: `    断言 '!label.contains("987")'(987654 为其「仅记账」哨兵值),但标签渲染了会话 ID`
  - `AGENTS.md:576`: `    **115/1 → 116/0**;'rustcode-review --lib' **100/0**(第三十轮);'coding' 430/0;'updater' 41/0;`
  - `AGENTS.md:584`: `    **唯一失败即 'trust_key_golden_matches_core_algorithm'**('AGENTS.md:226' 文档化已知红,`
  - `AGENTS.md:586`: `    执行要点:后台 'setsid' + 唯一日志路径,启动前确认 'pgrep -c cargo = 0',`
  - `AGENTS.md:601`: `    教训:'search_content' 工具的 'count' 模式**会低估**,统计残留面**必须用 shell 精确计数**,`
  - `AGENTS.md:607`: `    'docs/{UPSTREAM_RUSTCODE_LICENSE,UPSTREAM_CREDITS,THIRD_PARTY_NOTICES,ORIGINAL_LICENSE}.md';`
  - `AGENTS.md:608`: `    ② **门禁定义必留** —— G7/G8 那几行**本身就是 grep 模式**,删掉 'atomcode' 字样会让门禁失效`
  - `AGENTS.md:612`: `    'docs/{features,platform-neutralization}.md' 正文中的上游 slug 'atomgit_atomcode/atomcode'`
  - `AGENTS.md:613`: `    改为「上游项目」,**并保留指向 'ORIGINAL_LICENSE.md' / 'UPSTREAM_CREDITS.md' 的归属链接**;`
  - `AGENTS.md:633`: `    ('cargo test -p atomcode-{tuix,capabilities,daemon}' → 'rustcode-*',及任务书未点名的`
  - `AGENTS.md:644`: `    'cargo fmt --all -- --check' / 'cargo clippy --workspace --all-targets' / 'cargo test --workspace',`
  - `AGENTS.md:646`: `    格式违规,**使 CI 的 'fmt' job 由红转绿**;② CI 'test' job 跑裸 'cargo test --workspace',`
  - `AGENTS.md:647`: `    而 'trust_key' 确定性失败,**该 job 仍将为红**(既有状态,非本轮引入);③ 'clippy' job 未加`
  - `AGENTS.md:651`: `    ('cli/src/main.rs:3363')存活**,与对方删除在同一文件共存;G1 **exit=0 / 0 差异**;`

## PASS `CONTEXT.md`

- en=0/45 ratio=0.0000

## PASS `DEVENV.md`

- en=0/23 ratio=0.0000

## PASS `README.md`

- en=0/374 ratio=0.0000
- AC-4-authorized: `README.md | removed=- | added=./scripts/build-webui.sh | cause=D2`
- R5 跳过的行 (共 2 行):
  - `README.md:549`: `>`
  - `README.md:551`: `>`

## PASS `crates/rustcode-capabilities/README.md`

- en=1/46 ratio=0.0217
- AC-3 残留行清单 (en 行，共 1 行):
  - `crates/rustcode-capabilities/README.md:1`: `# rustcode-capabilities (L1)`

## PASS `crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/SKILL.md`

- en=1/152 ratio=0.0066
- AC-6-desc-changed: `description: 'Analyze a codebase and recommend RustCode automations (hooks, subagents, skills, plugins, MCP servers). Use when user asks for automation recommendations, wants to optimize their RustCode setup, mentions improving RustCode workflows, asks how to first set up RustCode for a project, or wants to know what RustCode features they should use.' => '分析代码库并推荐 RustCode 自动化配置（hooks、subagents、skills、plugins、MCP servers）。当用户询问自动化推荐、希望优化 RustCode 配置、提到改进 RustCode 工作流、询问如何为一个项目首次配置 RustCode，或想知道该使用哪些 RustCode 功能时使用。'`
- AC-3 残留行清单 (en 行，共 1 行):
  - `crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/SKILL.md:218`: `  "mcpServers": {`
- R5 跳过的行 (共 4 行):
  - `crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/SKILL.md:219`: `    "[name]": {`
  - `crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/SKILL.md:220`: `      "command": "npx",`
  - `crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/SKILL.md:221`: `      "args": ["-y", "@package/name"]`
  - `crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/SKILL.md:222`: `    }`

## PASS `crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/hooks-patterns.md`

- en=1/111 ratio=0.0090
- AC-3 残留行清单 (en 行，共 1 行):
  - `crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/hooks-patterns.md:17`: `### ESLint（JavaScript/TypeScript lint）`

## PASS `crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/mcp-servers.md`

- en=0/172 ratio=0.0000

## PASS `crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/plugins-reference.md`

- en=0/61 ratio=0.0000

## PASS `crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/skills-reference.md`

- en=0/94 ratio=0.0000

## PASS `crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/subagent-templates.md`

- en=0/114 ratio=0.0000

## PASS `crates/rustcode-clix/README.md`

- en=3/82 ratio=0.0366
- AC-3 残留行清单 (en 行，共 3 行):
  - `crates/rustcode-clix/README.md:83`: `**GitHub:**`
  - `crates/rustcode-clix/README.md:171`: `C# / Swift / Objective-C / Dart / Scala / Ruby / PHP / Groovy / Lua / Perl / R / Elixir /`
  - `crates/rustcode-clix/README.md:173`: `HTML / CSS / XML / YAML / JSON / TOML / Protobuf / GraphQL / Makefile / CMake / properties /`

## PASS `crates/rustcode-coding/README.md`

- en=1/49 ratio=0.0204
- AC-3 残留行清单 (en 行，共 1 行):
  - `crates/rustcode-coding/README.md:1`: `# rustcode-coding (L2)`

## PASS `crates/rustcode-daemon/README.md`

- en=2/210 ratio=0.0095
- AC-3 残留行清单 (en 行，共 2 行):
  - `crates/rustcode-daemon/README.md:1`: `# rustcode-daemon`
  - `crates/rustcode-daemon/README.md:600`: `### MCP（Model Context Protocol）`

## PASS `crates/rustcode-kernel/README.md`

- en=1/23 ratio=0.0435
- AC-3 残留行清单 (en 行，共 1 行):
  - `crates/rustcode-kernel/README.md:1`: `# rustcode-kernel (L0)`

## PASS `crates/rustcode-kernel/SPIKE.md`

- en=0/61 ratio=0.0000
- R5 跳过的行 (共 2 行):
  - `crates/rustcode-kernel/SPIKE.md:74`: `    cargo test -p rustcode-kernel`
  - `crates/rustcode-kernel/SPIKE.md:75`: `    cargo run -p rustcode-kernel --example minimal_specialization`

## PASS `crates/rustcode-review/LANGUAGES.md`

- en=0/74 ratio=0.0000

## PASS `crates/rustcode-review/README.md`

- en=1/41 ratio=0.0244
- AC-3 残留行清单 (en 行，共 1 行):
  - `crates/rustcode-review/README.md:1`: `# rustcode-review(L2)`

## PASS `crates/rustcode-tuix/tests/smoke.md`

- en=0/55 ratio=0.0000

## PASS `docker/README.md`

- en=0/54 ratio=0.0000
- R5 跳过的行 (共 1 行):
  - `docker/README.md:188`: `>`

## PASS `docs/HOOK_DOC_UPDATE_SPEC.md`

- en=0/212 ratio=0.0000

## PASS `docs/REFACTOR_DESIGN_PHASE1.md`

- en=0/297 ratio=0.0000

## PASS `docs/REFACTOR_SUMMARY.md`

- en=0/44 ratio=0.0000

## PASS `docs/acp-sdk-handler-notes.md`

- en=0/23 ratio=0.0000

## PASS `docs/adr/0001-runtime-owned-session-transitions.md`

- en=0/3 ratio=0.0000

## PASS `docs/adr/0002-runtime-owned-dynamic-mcp-tool-catalog.md`

- en=0/17 ratio=0.0000

## PASS `docs/adr/0003-runtime-owned-turn-execution-policy.md`

- en=0/24 ratio=0.0000

## PASS `docs/agent-api-rfc.md`

- en=0/141 ratio=0.0000

## PASS `docs/architecture.md`

- en=0/122 ratio=0.0000

## PASS `docs/archive/2026-07-25-provider-retry-consolidation.md`

- en=0/50 ratio=0.0000

## PASS `docs/archive/2026-07-27-models-dev-pricing-design.md`

- en=0/37 ratio=0.0000

## PASS `docs/archive/coding-runtime-incremental-migration.md`

- en=0/776 ratio=0.0000

## PASS `docs/archive/coding-runtime-native-migration-design.md`

- en=0/578 ratio=0.0000

## PASS `docs/archive/kernel-parity-backlog.md`

- en=1/58 ratio=0.0172
- AC-3 残留行清单 (en 行，共 1 行):
  - `docs/archive/kernel-parity-backlog.md:4`: `> ['coding-runtime-native-migration-design.md'](coding-runtime-native-migration-design.md)`

## PASS `docs/archive/live-transport-convergence-plan.md`

- en=0/130 ratio=0.0000

## PASS `docs/archive/release-v5.0.1-current-branch-change-report.md`

- en=0/167 ratio=0.0000

## PASS `docs/archive/release-v5.0.3-core-retirement-acceptance.md`

- en=0/256 ratio=0.0000

## PASS `docs/archive/session-convergence-plan.md`

- en=0/585 ratio=0.0000

## PASS `docs/archive/v5.0.0-retire-bridge-core-progress.md`

- en=0/81 ratio=0.0000

## PASS `docs/async-webhook-guide.md`

- en=0/108 ratio=0.0000

## PASS `docs/async-webhook-summary.md`

- en=0/125 ratio=0.0000

## PASS `docs/codex-claude-config-analysis.md`

- en=0/33 ratio=0.0000
- AC-4-authorized: `docs/codex-claude-config-analysis.md | removed=README.zh-CN.md | added=- | cause=D1`

## PASS `docs/compact-durable-checkpoint-design.md`

- en=0/102 ratio=0.0000

## PASS `docs/compact-native-migration-retrospective.md`

- en=0/282 ratio=0.0000

## PASS `docs/custom-endpoint-guide.md`

- en=0/51 ratio=0.0000

## PASS `docs/dev-env-setup.md`

- en=3/61 ratio=0.0492
- AC-3 残留行清单 (en 行，共 3 行):
  - `docs/dev-env-setup.md:26`: `| Node.js | v24.20.0 (LTS) | nvm | '~/.nvm/' |`
  - `docs/dev-env-setup.md:97`: `### 3.2 Node.js（nvm）`
  - `docs/dev-env-setup.md:135`: `> [!WARNING]`

## PASS `docs/features.md`

- en=0/55 ratio=0.0000

## PASS `docs/hook-architecture.md`

- en=0/21 ratio=0.0000

## PASS `docs/hook-cli-guide.md`

- en=0/111 ratio=0.0000

## PASS `docs/hook-expansion-summary.md`

- en=0/101 ratio=0.0000

## PASS `docs/hook-implementation-summary.md`

- en=0/102 ratio=0.0000

## PASS `docs/hook-timing-complete.md`

- en=0/127 ratio=0.0000

## PASS `docs/hooks.md`

- en=0/112 ratio=0.0000

## PASS `docs/i18n-field-mapping.md`

- en=0/58 ratio=0.0000

## PASS `docs/i18n-style.md`

- en=0/69 ratio=0.0000

## PASS `docs/mcp-rmcp-feasibility.md`

- en=0/121 ratio=0.0000

## PASS `docs/mcp.md`

- en=0/115 ratio=0.0000

## PASS `docs/mcp/github.md`

- en=0/70 ratio=0.0000

## PASS `docs/multi-agent-collaboration-solution.md`

- en=0/179 ratio=0.0000

## PASS `docs/phase1-refactor-design.md`

- en=0/296 ratio=0.0000
- AC-4-authorized: `docs/phase1-refactor-design.md | removed=README.zh-CN.md | added=- | cause=D1`
- AC-4-authorized: `docs/phase1-refactor-design.md | removed=README.zh-CN.md:151 | added=- | cause=D1`
- AC-4-authorized: `docs/phase1-refactor-design.md | removed=[2] README.md / README.zh-CN.md   零遥测口径统一；清除 Emoji；补"自定义网关"配置章节 | added=[2] README.md   零遥测口径统一；清除 Emoji；补"自定义网关"配置章节 | cause=D1`

## PASS `docs/phase2-subagent-status.md`

- en=0/154 ratio=0.0000

## PASS `docs/plans/2026-07-23-windows-qr-rendering.md`

- en=0/33 ratio=0.0000

## PASS `docs/plans/2026-07-24-busy-continue-fork.md`

- en=0/55 ratio=0.0000

## PASS `docs/plans/2026-07-25-subtasks-footer-panel.md`

- en=0/37 ratio=0.0000

## PASS `docs/plans/2026-07-26-atomgit-production-tools.md`

- en=0/30 ratio=0.0000

## PASS `docs/plans/2026-07-26-busy-continue-fork-gc.md`

- en=0/26 ratio=0.0000

## PASS `docs/plans/2026-07-26-model-cost-attribution.md`

- en=0/68 ratio=0.0000

## PASS `docs/plans/2026-07-26-provider-accounts-model-profiles-design.md`

- en=0/164 ratio=0.0000

## PASS `docs/plans/2026-07-26-provider-accounts-model-profiles-plan.md`

- en=0/173 ratio=0.0000

## PASS `docs/plans/2026-07-28-provider-panel-ui-design.md`

- en=0/98 ratio=0.0000

## PASS `docs/plans/2026-07-28-rewind-design.md`

- en=0/59 ratio=0.0000

## PASS `docs/plans/2026-07-28-rewind-implementation-plan.md`

- en=0/84 ratio=0.0000

## PASS `docs/plans/2026-07-31-native-runtime-datalog.md`

- en=0/51 ratio=0.0000

## PASS `docs/plans/2026-08-02-session-recovery-design.md`

- en=0/45 ratio=0.0000

## PASS `docs/plans/2026-08-02-session-recovery-implementation-plan.md`

- en=0/28 ratio=0.0000

## PASS `docs/plans/2026-08-06-internal-continuation-compaction-design.md`

- en=0/8 ratio=0.0000

## PASS `docs/plans/2026-08-07-config-panel-implementation-plan.md`

- en=0/54 ratio=0.0000

## PASS `docs/plans/2026-08-07-lightweight-lsp-phase-one.md`

- en=0/71 ratio=0.0000

## PASS `docs/plans/2026-08-07-request-user-input-review-design.md`

- en=0/31 ratio=0.0000

## PASS `docs/plans/2026-08-07-tasks-long-line-rendering.md`

- en=0/26 ratio=0.0000

## PASS `docs/plans/2026-08-10-first-stage-planning-quality-design.md`

- en=0/40 ratio=0.0000

## PASS `docs/plans/2026-08-11-provider-form-horizontal-editing-design.md`

- en=0/21 ratio=0.0000

## PASS `docs/plans/2026-08-11-todo-agent-body-projection-design.md`

- en=0/24 ratio=0.0000

## PASS `docs/plans/2026-08-14-webui-browser-notification-design.md`

- en=2/138 ratio=0.0145
- AC-3 残留行清单 (en 行，共 2 行):
  - `docs/plans/2026-08-14-webui-browser-notification-design.md:108`: `| stop_reason | title | body |`
  - `docs/plans/2026-08-14-webui-browser-notification-design.md:114`: `| undefined | RustCode finished | Finished |`

## PASS `docs/plans/2026-08-15-webui-blocking-interaction-dock.md`

- en=0/38 ratio=0.0000

## PASS `docs/plans/2026-08-17-acp-sdk-2.0-protocol-roadmap.md`

- en=4/268 ratio=0.0149
- AC-3 残留行清单 (en 行，共 4 行):
  - `docs/plans/2026-08-17-acp-sdk-2.0-protocol-roadmap.md:37`: `  'session/update'(agent_message_chunk / agent_thought_chunk / tool_call / tool_call_update),`
  - `docs/plans/2026-08-17-acp-sdk-2.0-protocol-roadmap.md:86`: `- matcher:'if_message' → 'if_dispatch';'with_responder' → 'with_runner';`
  - `docs/plans/2026-08-17-acp-sdk-2.0-protocol-roadmap.md:100`: `initialize / session/new / session/prompt / session/cancel / session/request_permission`
  - `docs/plans/2026-08-17-acp-sdk-2.0-protocol-roadmap.md:209`: `   'state_update';capabilities+info;tool_call_update-only + content_chunk;plan_update;`

## PASS `docs/plans/2026-08-17-project-input-history-design.md`

- en=0/51 ratio=0.0000

## PASS `docs/plans/2026-08-20-code-review-deep-mode-fanout-design.md`

- en=0/95 ratio=0.0000

## PASS `docs/plans/2026-08-20-code-review-deep-mode-fanout-plan.md`

- en=0/171 ratio=0.0000

## PASS `docs/plans/2026-08-20-code-review-deep-verify-phase2-plan.md`

- en=0/112 ratio=0.0000

## PASS `docs/plans/2026-08-21-external-agent-subagent-drivers-spec.md`

- en=3/167 ratio=0.0180
- AC-3 残留行清单 (en 行，共 3 行):
  - `docs/plans/2026-08-21-external-agent-subagent-drivers-spec.md:96`: `### 4.1 'SubagentBackend' trait`
  - `docs/plans/2026-08-21-external-agent-subagent-drivers-spec.md:147`: `| rustcode | Claude Code ('claude -p --permission-mode') | Codex ('codex exec') |`
  - `docs/plans/2026-08-21-external-agent-subagent-drivers-spec.md:200`: `- spawn（'tokio::process::Command'，'kill_on_drop'）。`

## PASS `docs/plans/2026-08-23-deepseek-v4-flash-evaluation-design.md`

- en=0/55 ratio=0.0000

## PASS `docs/plans/2026-08-23-deepseek-v4-flash-evaluation.md`

- en=0/89 ratio=0.0000

## PASS `docs/plans/2026-08-23-headless-eval-controls.md`

- en=1/28 ratio=0.0357
- AC-3 残留行清单 (en 行，共 1 行):
  - `docs/plans/2026-08-23-headless-eval-controls.md:28`: `  cache hit rate；`

## PASS `docs/platform-neutralization.md`

- en=0/192 ratio=0.0000
- AC-4-authorized: `docs/platform-neutralization.md | removed=README.zh-CN.md | added=- | cause=D1`
- R5 跳过的行 (共 14 行):
  - `docs/platform-neutralization.md:38`: `    ('crates/rustcode-cli/src/main.rs:5081')`
  - `docs/platform-neutralization.md:49`: `    ('crates/rustcode-config/src/config/provider_preset.rs:303')`
  - `docs/platform-neutralization.md:75`: `    'uninstall.sh' / 'uninstall.ps1' 头部 'curl|sh' / 'irm' 厂商 URL 改本地 / 发行`
  - `docs/platform-neutralization.md:157`: `    status-glyph*.md' 等 3 个文件)里的 '🟢/🟡/🔴' 是该彩色状态点特性的规格主语义`
  - `docs/platform-neutralization.md:168`: `    ('crates/rustcode-capabilities/src/cc_hooks.rs:1144'):测试自身缺陷——同一 '&&'`
  - `docs/platform-neutralization.md:169`: `    链中两个 'grep' 共享 stdin,首个 grep 抽干管道致第二个必失败;改为先`
  - `docs/platform-neutralization.md:172`: `    ('crates/rustcode-capabilities/src/tools/read.rs:1276'):断言停留在旧 300 行分页,`
  - `docs/platform-neutralization.md:173`: `    实现已为 1500 行页('DEFAULT_READ_LIMIT','crates/rustcode-capabilities/src/`
  - `docs/platform-neutralization.md:174`: `    tools/read.rs:25');夹具改为 1600 行并对齐`
  - `docs/platform-neutralization.md:177`: `    'SpawnFailed("Text file busy (os error 26)")'(overlayfs / 容器内新写脚本`
  - `docs/platform-neutralization.md:178`: `    execve 的 ETXTBSY 竞态;Go fork/exec 内置重试而 Rust std 没有):新增`
  - `docs/platform-neutralization.md:179`: `    'process_utils::spawn_retrying_etxtbsy'(8 次退避重试,`
  - `docs/platform-neutralization.md:180`: `    'crates/rustcode-capabilities/src/process_utils.rs:204'),接入`
  - `docs/platform-neutralization.md:181`: `    'subagent::proc::ManagedChild::spawn'('.../subagent/proc.rs:146')与`

## PASS `docs/pr-descriptions/2026-04-27-vscode-extension-ui-mcp.md`

- en=0/59 ratio=0.0000

## PASS `docs/pr-hook-test-command.md`

- en=0/37 ratio=0.0000

## PASS `docs/security/permission-model.md`

- en=0/253 ratio=0.0000

## PASS `docs/superpowers/2026-07-27-release-v5.0.3-test-checklist.md`

- en=0/93 ratio=0.0000

## PASS `docs/superpowers/plans/2026-04-19-tuix-ink-cell-diff.md`

- en=0/93 ratio=0.0000

## PASS `docs/superpowers/plans/2026-04-19-tuix-retained-mode-rewrite.md`

- en=0/169 ratio=0.0000

## PASS `docs/superpowers/plans/2026-04-23-agent-harness-principles.md`

- en=0/120 ratio=0.0000

## PASS `docs/superpowers/plans/2026-04-23-cadence-reflection.md`

- en=0/131 ratio=0.0000

## PASS `docs/superpowers/plans/2026-04-23-merge-current-task-into-cadence.md`

- en=0/109 ratio=0.0000

## PASS `docs/superpowers/plans/2026-05-08-vision-preprocessor-auto-config.md`

- en=0/109 ratio=0.0000

## PASS `docs/superpowers/plans/2026-05-08-vision-preprocessor.md`

- en=0/159 ratio=0.0000

## PASS `docs/superpowers/plans/2026-05-25-tuix-unified-in-app-scroll.md`

- en=0/562 ratio=0.0000

## PASS `docs/superpowers/plans/2026-05-29-webui.md`

- en=0/502 ratio=0.0000
- AC-4-authorized: `docs/superpowers/plans/2026-05-29-webui.md | removed=README.zh-CN.md | added=- | cause=D1`
- AC-4-authorized: `docs/superpowers/plans/2026-05-29-webui.md | removed=git add webui/ crates/ README.md README.zh-CN.md | added=git add webui/ crates/ README.md | cause=D1`

## PASS `docs/superpowers/plans/2026-06-09-cache-friendly-compaction.md`

- en=0/135 ratio=0.0000

## PASS `docs/superpowers/plans/2026-06-27-v2-rate-limit-pause-resume.md`

- en=0/234 ratio=0.0000

## PASS `docs/superpowers/plans/2026-06-29-acp-agent.md`

- en=0/264 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-03-terminal-status-glyph.md`

- en=0/97 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-06-double-esc-undo-cooldown.md`

- en=0/62 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-11-persistent-todo-panel.md`

- en=0/189 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-12-github-style-diff.md`

- en=0/129 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-13-selectable-approval.md`

- en=0/158 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-22-batch-user-questions-persona-nudge.md`

- en=0/52 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-22-brainstorming-request-user-input.md`

- en=0/78 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-22-deepseek-skill-first-reminder.md`

- en=0/85 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-22-multi-question-request-user-input.md`

- en=0/157 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-22-request-user-input-custom-and-submit-spacing.md`

- en=0/138 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-24-retire-core-conversation-tui-port.md`

- en=0/146 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-24-skills-multi-compose.md`

- en=0/77 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-24-windows-native-tls-schannel-fallback.md`

- en=0/65 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-25-retire-core-provider-A-compact.md`

- en=0/66 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-25-retire-core-provider-B-vision.md`

- en=0/110 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-25-retire-core-provider-C2-conversation-transport.md`

- en=0/67 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-25-retire-core-tool-ball-D.md`

- en=0/67 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-25-round-cap-checkpoint.md`

- en=0/188 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-29-webui-sync-compact.md`

- en=0/98 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-31-local-scheduled-tasks-phase1.md`

- en=0/131 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-31-local-scheduled-tasks-phase2.md`

- en=0/95 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-31-project-memory-dir-override.md`

- en=0/74 ratio=0.0000

## PASS `docs/superpowers/specs/2026-04-23-p2-doctor-review-notebook-todo-design.md`

- en=0/39 ratio=0.0000

## PASS `docs/superpowers/specs/2026-05-08-vision-preprocessor-design.md`

- en=0/97 ratio=0.0000

## PASS `docs/superpowers/specs/2026-05-25-tuix-unified-in-app-scroll-design.md`

- en=0/178 ratio=0.0000

## PASS `docs/superpowers/specs/2026-05-29-provider-add-simplify-design.md`

- en=0/147 ratio=0.0000
- R5 跳过的行 (共 1 行):
  - `docs/superpowers/specs/2026-05-29-provider-add-simplify-design.md:180`: `    model = body 里 '"model"' 值`

## PASS `docs/superpowers/specs/2026-05-29-webui-design.md`

- en=0/153 ratio=0.0000

## PASS `docs/superpowers/specs/2026-06-07-headless-output-format-json-design.md`

- en=0/69 ratio=0.0000

## PASS `docs/superpowers/specs/2026-06-09-cache-friendly-compaction-design.md`

- en=0/80 ratio=0.0000

## PASS `docs/superpowers/specs/2026-06-27-v2-rate-limit-pause-resume-design.md`

- en=0/73 ratio=0.0000

## PASS `docs/superpowers/specs/2026-06-29-acp-agent-design.md`

- en=0/130 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-03-terminal-status-glyph-design.md`

- en=0/72 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-06-double-esc-undo-cooldown-design.md`

- en=0/41 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-11-persistent-todo-panel-design.md`

- en=0/113 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-13-selectable-approval-design.md`

- en=1/62 ratio=0.0161
- AC-3 残留行清单 (en 行，共 1 行):
  - `docs/superpowers/specs/2026-07-13-selectable-approval-design.md:36`: `- 'struct ApprovalOption { label: String, decision: PermissionDecision, accel: char }'(accel:'y'/'a'/'n')。`

## PASS `docs/superpowers/specs/2026-07-22-batch-user-questions-persona-nudge-design.md`

- en=0/67 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-22-brainstorming-request-user-input-design.md`

- en=0/87 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-22-deepseek-skill-first-reminder-design.md`

- en=0/86 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-22-multi-question-request-user-input-design.md`

- en=0/114 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-22-request-user-input-custom-and-submit-spacing-design.md`

- en=0/90 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-24-retire-core-conversation-tui-port-design.md`

- en=0/59 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-24-skills-multi-compose-design.md`

- en=0/57 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-24-windows-native-tls-schannel-fallback-design.md`

- en=0/59 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-25-retire-core-provider-A-compact-design.md`

- en=0/38 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-25-retire-core-provider-B-vision-design.md`

- en=1/47 ratio=0.0213
- AC-3 残留行清单 (en 行，共 1 行):
  - `docs/superpowers/specs/2026-07-25-retire-core-provider-B-vision-design.md:16`: `- **daemon** 'live_api.rs:1119 preprocess_image_caption(config, active: &dyn core::LlmProvider, …)' → 'maybe_preprocess'（core）。`

## PASS `docs/superpowers/specs/2026-07-25-retire-core-provider-C-conversation-design.md`

- en=1/47 ratio=0.0213
- AC-3 残留行清单 (en 行，共 1 行):
  - `docs/superpowers/specs/2026-07-25-retire-core-provider-C-conversation-design.md:35`: `- 'AuthoritativeTerminal.snapshot: ConversationSnapshot' → 'SessionSnapshot'（live_api.rs:72）。`

## PASS `docs/superpowers/specs/2026-07-25-retire-core-tool-ball-D-design.md`

- en=0/44 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-25-round-cap-checkpoint-design.md`

- en=0/80 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-29-user-input-background-block-design.md`

- en=0/57 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-29-webui-sync-compact-design.md`

- en=0/62 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-30-progress-signposts-preamble-design.md`

- en=0/45 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-30-workflow-intent-understanding-design.md`

- en=0/54 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-31-local-scheduled-tasks-design.md`

- en=1/52 ratio=0.0192
- AC-3 残留行清单 (en 行，共 1 行):
  - `docs/superpowers/specs/2026-07-31-local-scheduled-tasks-design.md:98`: `**DEFER**：`

## PASS `docs/superpowers/specs/2026-07-31-local-scheduled-tasks-phase2-design.md`

- en=0/59 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-31-project-memory-dir-override-design.md`

- en=0/50 ratio=0.0000

## PASS `docs/target-architecture.md`

- en=0/70 ratio=0.0000

## PASS `docs/telemetry.md`

- en=0/28 ratio=0.0000

## PASS `docs/testing/release-v5.0.0-acceptance.md`

- en=0/190 ratio=0.0000

## PASS `docs/testing/windows-path-normalization.md`

- en=0/63 ratio=0.0000
- R5 跳过的行 (共 6 行):
  - `docs/testing/windows-path-normalization.md:25`: `      → 正常运行(过去会 'python C:sers...' 找不到)`
  - `docs/testing/windows-path-normalization.md:45`: `      (过去 '\\?\C:\..' 传给 'cmd start' 打不开)`
  - `docs/testing/windows-path-normalization.md:48`: `      '/context' / footer 的 cwd 正常`
  - `docs/testing/windows-path-normalization.md:50`: `      'fs_mkdir')→ 选它 → 该目录的会话在 **webui 和 TUI 两边是同一个**`
  - `docs/testing/windows-path-normalization.md:56`: `      读 / grep → **不再被拒**(过去每个绝对路径都报 "outside the review repository")`
  - `docs/testing/windows-path-normalization.md:74`: `      也接受,但实测 'type' / 'cd' / python 等是否 OK(若有问题需单独处理)`

## PASS `docs/vscode-i18n-implementation-plan-2026-06-30.md`

- en=0/71 ratio=0.0000

## PASS `docs/webhook-guide.md`

- en=0/108 ratio=0.0000

## PASS `docs/webhook-implementation-summary.md`

- en=0/106 ratio=0.0000

## PASS `evals/deepseek-v4-flash/README.md`

- en=0/17 ratio=0.0000

## PASS `evals/deepseek-v4-flash/cases/agent-fixture/ARCHITECTURE.md`

- en=0/3 ratio=0.0000

## PASS `extensions/jetbrains/PRIVACY.md`

- en=0/24 ratio=0.0000

## PASS `extensions/jetbrains/README.md`

- en=0/117 ratio=0.0000

## PASS `extensions/jetbrains/docs/jetbrains.md`

- en=2/80 ratio=0.0250
- AC-3 残留行清单 (en 行，共 2 行):
  - `extensions/jetbrains/docs/jetbrains.md:62`: `- Claude`
  - `extensions/jetbrains/docs/jetbrains.md:63`: `- Ollama`

## PASS `extensions/vscode/README.md`

- en=0/92 ratio=0.0000

## PASS `packages/npm/README.md`

- en=0/16 ratio=0.0000

## PASS `webui/README.md`

- en=0/21 ratio=0.0000
