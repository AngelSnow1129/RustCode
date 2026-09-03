---
kind: task
id: TASKS-001
from: solution-architect
to: [project-manager, code-implementer]
feature: 2026-09-02-cleanup-codingplan-legacy
status: ready
decision: proceed
requires: [DESIGN-001]
created: 2026-09-02
---

# 02 · 任务图（S2 标准档 · CodingPlan 遗留痕迹清理）

> **上游**：`REQ-001`（`00-requirement.md`）、`DESIGN-001`（`01-design.md`）。契约已冻结，**实现者不得擅自改契约**，争议回退 `solution-architect`。
> **基线**：branch=`dev` commit=`8e772dbf`，worktree **dirty**。
> **全局第 0 步（每个任务必做，fail-closed）**：`git status --porcelain -- <本任务 files_owned>` → **非空即中止该文件的改动并上报 PM**，禁止 `git checkout` / `reset` / `stash`（N10）。
> **包名**：CLI 包名是 `rustcode`，`-p rustcode-cli` 会失败。**并发**：`cargo test --workspace` 必须 `-j 1`（`AGENTS.md:541`）。
> **唯一允许的既有失败**：`mcp::registry::tests::trust_key_golden_matches_core_algorithm`（`AGENTS.md:226`、`:350`），禁止改测试凑绿。

---

## 1. 任务列表

### T1 · D 类失效 core 路径注释清零

| 字段 | 内容 |
|---|---|
| **id** | `T1` |
| **标题** | D 类失效 core 路径注释清零（7 处） |
| **目标** | 使 AC-10 两条 grep 判据在 `crates/` 下均为 **0 命中** |
| **批次** | **Batch 1（独占批次，串行）** —— 本任务跨 `rustcode-codingplan` + `rustcode-config` 两个 crate，按 `.codebuddy/rules/multi-agent-workflow.md` §3「跨 crate 改动默认串行」独占批次 |
| **依赖** | `DESIGN-001`（无前序任务依赖） |
| **crate** | `rustcode-codingplan`、`rustcode-config` |
| **files_owned** | `crates/rustcode-codingplan/src/lib.rs`<br>`crates/rustcode-codingplan/src/types.rs`<br>`crates/rustcode-codingplan/src/client.rs`<br>`crates/rustcode-codingplan/src/setup.rs`<br>`crates/rustcode-codingplan/src/sync_marker.rs`<br>`crates/rustcode-config/src/i18n/messages.rs` |
| **改动内容** | 仅改第 1 行 / 指定行**注释**，**禁止改任何代码、`Msg` 变体、en/zh 文案、签名**：<br>① `lib.rs:1` → `// crates/rustcode-codingplan/src/lib.rs`<br>② `types.rs:1` → `// crates/rustcode-codingplan/src/types.rs`<br>③ `client.rs:1` → `// crates/rustcode-codingplan/src/client.rs`<br>④ `setup.rs:1` → `// crates/rustcode-codingplan/src/setup.rs`<br>⑤ `sync_marker.rs:1` → `// crates/rustcode-codingplan/src/sync_marker.rs`<br>⑥ `messages.rs:319` → `// SetupReport renderer (rustcode-codingplan::setup)`<br>⑦ `messages.rs:376` → 注释内 `coding_plan::setup::strikethrough` 改 `rustcode_codingplan::setup::strikethrough` |
| **验收标准** | ① `rg -n "rustcode-core/src/coding_plan" crates/` → **0 命中**<br>② `rg -n "core/coding_plan/setup.rs" crates/` → **0 命中**<br>③ `rg -n "mod codingplan_crypto_tests" crates/rustcode-config/src/i18n/` → 仍命中（AC-13）<br>④ 无新增编译 warning |
| **验收命令** | `rg -n "rustcode-core/src/coding_plan" crates/ ; rg -n "core/coding_plan/setup.rs" crates/ ; rg -n "mod codingplan_crypto_tests" crates/rustcode-config/src/i18n/`<br>`cargo build`<br>`cargo check --workspace --all-targets`<br>`cargo check -p rustcode-codingplan --features client --all-targets`<br>`cargo check -p rustcode-tuix --features codingplan --all-targets`<br>`cargo check -p rustcode-daemon --features codingplan --all-targets`<br>`cargo check -p rustcode --features codingplan --all-targets`<br>`cargo check -p rustcode --features codingplan-crypto --all-targets`<br>`cargo check -p rustcode-daemon --features codingplan-crypto --all-targets`<br>`cargo test -p rustcode-codingplan --lib`<br>`cargo test -p rustcode-codingplan --lib sync_marker`<br>`cargo test -p rustcode-config --lib`<br>`cargo test -p rustcode-tuix --lib`<br>`rg -n "read_last_sync\(\)" crates/rustcode-tuix/src/lib.rs crates/rustcode-tuix/src/event_loop/mod.rs`<br>`rg -n "^use rustcode_codingplan" crates/rustcode-tuix/src/modals/usage.rs`<br>`rg -n "codingplan_sync.json" crates/`<br>`rg -n "LEGACY_CODINGPLAN_PREFIX\|is_codingplan_provider_name" crates/`<br>`rg -rni "atomcode" crates/ scripts/ .github/`（须 0 命中）<br>`cargo fmt --check`（差异集不得扩大，改动文件不在存量清单内 → 应完全干净） |
| **回滚方式** | 上述 6 个文件经编排者核验为**非 dirty**。回滚前先 `git diff -- <6 files> > .codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/rollback/T1.patch`，再 `git checkout -- <6 files>`。**若第 0 步复核发现任一文件 dirty → 禁止 checkout，改人工 diff 还原并上报。** |
| **预估复杂度** | S（7 行注释；验证命令多但均为机械执行） |
| **状态** | `ready` |

---

### T3 · 删除 CLI 隐藏别名 `Commands::Codingplan`

| 字段 | 内容 |
|---|---|
| **id** | `T3` |
| **标题** | 删除 CLI 隐藏别名 `Commands::Codingplan`（用户裁决 Q2=B） |
| **目标** | 消除与 `AGENTS.md:278`「该别名已移除」的事实冲突；AC-16 保持绿 |
| **批次** | **Batch 2** |
| **依赖** | `DESIGN-001` |
| **crate** | `rustcode-cli` |
| **files_owned** | `crates/rustcode-cli/src/main.rs` |
| **改动内容** | ① **删** `:1022-1026`（doc comment + `#[command(hide = true)]` + `Codingplan,`）<br>② **改** `:1696` `Commands::Login \| Commands::Codingplan =>` → `Commands::Login =>`（**注意**：`:1697-1715` 的 login 流程体完整保留，仅改 match 臂头）<br>③ **删** `:3574-3578` 整条 `unreachable!` 臂<br>④ **同步修注释** `:1669-1670`、`:3519`（提及 "hidden alias `Codingplan`" 的表述）<br>**不改动**：`:1387`、`:1704-1705`、`:1715-1718`、`:2897`、`:4515-4625`（`#[cfg(feature="codingplan")]` 面）、`:4903`、`:4966`（既有断言）<br>**实施前须先 grep 验证补全/help 文案是否另有依赖**：`rg -n "Codingplan" crates/rustcode-cli/src/ crates/rustcode-tuix/src/` |
| **验收标准** | ① `rg -n "Codingplan" crates/rustcode-cli/src/main.rs` → **0 命中**<br>② 默认构建与 `codingplan` feature 构建均 exit 0<br>③ 补全脚本断言与中立构建断言仍绿<br>④ `Commands` 枚举不再含 `Codingplan`，且**新增 0 个变体** |
| **验收命令** | `rg -n "Codingplan" crates/rustcode-cli/src/main.rs`（0 命中）<br>`cargo build`<br>`cargo check --workspace --all-targets`<br>`cargo check -p rustcode --features codingplan --all-targets`<br>`cargo check -p rustcode --features codingplan-crypto --all-targets`<br>`cargo test -p rustcode shell_completion`<br>`cargo test -p rustcode --lib neutral_build_hides_managed_login_subcommands`<br>`cargo fmt --check`（差异集不得扩大；注意 `main.rs` 在 `AGENTS.md:542` 的 19 处**存量**违规内） |
| **回滚方式** | `main.rs` 经编排者核验为**非 dirty**，但**含 19 处非本轮引入的存量 fmt 违规**（`AGENTS.md:542`）→ **必须按 hunk 人工还原，禁止整体 `git checkout`**。回滚前先 `git diff -- crates/rustcode-cli/src/main.rs > .codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/rollback/T3.patch`。若第 0 步复核发现 dirty → 禁止 checkout，人工 diff 还原并上报。 |
| **预估复杂度** | M（3 处删除 + 2 处注释；需穷尽式 grep 复核补全/help/派生代码） |
| **状态** | `ready` |
| **⚠ 契约变更登记** | `rustcode codingplan` 由「等价于 `rustcode login`」变为 **clap 未知子命令 → exit 2**。这是对外 CLI 契约的 breaking change，已由用户 Q2=B 授权；**必须写入交付报告 / release note**。**禁止**为兼容而保留别名或新增转发（与用户"不保留旧版本入口"诉求冲突）。 |

---

### T4 · `docs/config.example.toml` 英文网关示例段去重

| 字段 | 内容 |
|---|---|
| **id** | `T4` |
| **标题** | `docs/config.example.toml` 删除英文 CodingPlan 网关示例段（用户裁决 Q6=A） |
| **目标** | 文件内 CodingPlan 网关示例**恰好 1 段**，且仍是合法、可直接拷贝的 TOML 范例 |
| **批次** | **Batch 2** |
| **依赖** | `DESIGN-001` |
| **crate** | 无（文档） |
| **files_owned** | `docs/config.example.toml` |
| **改动内容** | **删** `:38-56`（`:38` 段首标题、`:39-54` 两个 provider 注释块、`:55` `───` 分隔线、`:56` 空行）。**保留**中文段 `:149-169`（含 `max_tokens`，信息更全，与文件主体中文注释风格一致）。**只删完整注释块，不删任何结构性行、不删任何非注释行。** |
| **验收标准** | ① `rg -n "CodingPlan" docs/config.example.toml` → **恰 1 命中**（中文段标题，行号位移后）<br>② 文件仍是合法 TOML（可用任意 TOML 解析器校验；未注释的 `[providers.*]` 节结构完整未变）<br>③ 该文件被 `README.md:408`、`README.zh-CN.md:365` 引用 → 两处引用路径不变 |
| **验收命令** | `rg -n "CodingPlan" docs/config.example.toml`（1 命中）<br>`rg -c "" docs/config.example.toml`（行数 = 原行数 − 19）<br>人工/脚本校验：文件可被 TOML 解析且 `[providers.deepseek]` 等未注释节仍在（**证据不足**：无 shell 权限未能给出解析命令，实施者自行选用 `python3 -c "import tomllib;tomllib.load(open('docs/config.example.toml','rb'))"` 或等价手段） |
| **回滚方式** | 该文件**不在**编排者核验的 dirty 清单内 → 回滚前先 `git diff -- docs/config.example.toml > .codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/rollback/T4.patch`，再 `git checkout -- docs/config.example.toml`。若第 0 步复核发现 dirty → 禁止 checkout，人工 diff 还原并上报。 |
| **预估复杂度** | S（19 行注释删除） |
| **状态** | `ready` |

---

### T7 · 死代码 / 跨 crate 重复实现扫描与登记（只登记，不改代码）

| 字段 | 内容 |
|---|---|
| **id** | `T7` |
| **标题** | 死代码 / 重复实现扫描与登记（**只产出清单，零代码改动**） |
| **目标** | 产出可交付的清单，为未来评审提供依据（G4 的"扫描"部分） |
| **批次** | **Batch 2** |
| **依赖** | `DESIGN-001` |
| **crate** | 无（只读扫描） |
| **files_owned** | `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/03-impl/T7-dead-code-scan.md`（**唯一可写文件**） |
| **改动内容** | 扫描并登记，**不改任何代码**：<br>① `Cp*` / `StatusCp*` i18n 变体的引用计数（`crates/rustcode-config/src/i18n/messages.rs:320-395+`）<br>② `codingplan*` 相关符号的零引用者<br>③ 跨 crate 重复实现：tuix `event_loop/commands.rs:4988-5060` ≈ daemon `commands.rs:641-705`（~65 行逐字重复）<br>④ `crates/` 下**非本 feature** 的失效 `rustcode-core/src/*` 路径注释 7 处（`tuix/src/custom_commands.rs:1`、`config/src/config/instructions.rs:1`、`kernel/src/agent.rs:169`、`kernel/tests/fallible_stream.rs:154`、`coding/src/persona.rs:2`、`capabilities/src/tools/bash.rs:1241`、`capabilities/src/skills/render.rs:4`）<br>**铁律**：`Cp*` 族、`codingplan_crypto_tests`、`LEGACY_CODINGPLAN_PREFIX` / `prefixes_for` / `codingplan_prefixes()` / `is_codingplan_provider_name()`、`atomgit` feature、`#[cfg(feature="codingplan")]` 块 —— **即便扫描结果为零引用，也不得删除**（`AGENTS.md:213-214`、`:338`、`:530`；N6） |
| **验收标准** | ① 清单文件存在且含上述 4 类条目，每条目带 `文件:行` 与引用计数<br>② 清单对每个零引用项明确标注「受 XX 铁律保护 / 可删（需另开 feature）」<br>③ **工作区零代码改动**：`git status --porcelain crates/ docs/config.example.toml` 的输出与本批次开始前一致 |
| **验收命令** | `rg -n "Cp[A-Z]" crates/ --stats -g '!target'`<br>`rg -n "StatusCp" crates/`<br>`rg -n "rustcode-core/src" crates/`<br>`rg -n "codingplan" crates/ --stats -g '!target'`<br>`test -f .codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/03-impl/T7-dead-code-scan.md` |
| **回滚方式** | 删除清单文件即可（`rm .codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/03-impl/T7-dead-code-scan.md`）。**本任务不产生任何代码改动，无代码回滚面。** |
| **预估复杂度** | M（扫描面广；但零改动、零风险） |
| **状态** | `ready` |

---

### T5 · 文档归档 Group A（10 份 `git mv` 到 `docs/archive/` + 链接同步修正）

| 字段 | 内容 |
|---|---|
| **id** | `T5` |
| **标题** | 过时方案文档归档 Group A（10 份）+ 交叉引用同步修正（用户裁决 Q7=A） |
| **目标** | 归档已完结文档，**零断链**（AC-17） |
| **批次** | **Batch 3** |
| **依赖** | `DESIGN-001` |
| **crate** | 无（文档） |
| **files_owned** | `docs/coding-runtime-incremental-migration.md` → `docs/archive/coding-runtime-incremental-migration.md`<br>`docs/coding-runtime-native-migration-design.md` → `docs/archive/coding-runtime-native-migration-design.md`<br>`docs/session-convergence-plan.md` → `docs/archive/session-convergence-plan.md`<br>`docs/live-transport-convergence-plan.md` → `docs/archive/live-transport-convergence-plan.md`<br>`docs/v5.0.0-retire-bridge-core-progress.md` → `docs/archive/v5.0.0-retire-bridge-core-progress.md`<br>`docs/kernel-parity-backlog.md` → `docs/archive/kernel-parity-backlog.md`<br>`docs/release-v5.0.1-current-branch-change-report.md` → `docs/archive/release-v5.0.1-current-branch-change-report.md`<br>`docs/testing/release-v5.0.3-core-retirement-acceptance.md` → `docs/archive/release-v5.0.3-core-retirement-acceptance.md`<br>`docs/plans/2026-07-25-provider-retry-consolidation.md` → `docs/archive/2026-07-25-provider-retry-consolidation.md`<br>`docs/plans/2026-07-27-models-dev-pricing-design.md` → `docs/archive/2026-07-27-models-dev-pricing-design.md`<br>`docs/archive/`（新建目录）<br>**扁平化**：不建 `docs/archive/testing/`、`docs/archive/plans/` 子目录（理由见 DESIGN-001 §5.2，可使组内 7 处同目录链接天然成立） |
| **改动内容** | ① `git mv` 上述 10 份<br>② **6 处外链加 `../` 前缀**（目标留在 `docs/`）：`archive/coding-runtime-incremental-migration.md` 的 `:9`、`:69`、`:71`；`archive/coding-runtime-native-migration-design.md` 的 `:13`、`:15`；`archive/v5.0.0-retire-bridge-core-progress.md` 的 `:119`<br>③ **无需修改**的同目录链接：`:70`、`:14`、`:61`、`:62`、`:4`（kernel-parity-backlog）、`:9`/`:799`（session-convergence-plan）<br>**注意**：行号在移动后不变（`git mv` 不改内容；仅上述 6 行内容被改） |
| **验收标准** | ① 10 份均在 `docs/archive/` 且原路径不存在<br>② 对每份被移动文档跑 `rg -n "<被移动文件名>" AGENTS.md README.md README.zh-CN.md docs/ .codebuddy/` → **全部命中均为有意保留的相对链接且可解析**（`docs/mcp.md` 与 `docs/compact-durable-checkpoint-design.md` **不在**本任务 files_owned，其引用的是 Group B 两份文档，由 T6 处理）<br>③ 6 处 `../` 修正已生效<br>④ `docs/plans/`、`docs/testing/` 目录仍存在且非空 |
| **验收命令** | `ls docs/archive/`（10 个文件）<br>`test ! -f docs/coding-runtime-incremental-migration.md && echo OK`<br>`rg -n "\]\((?!\.\./)[a-zA-Z0-9_.-]+\.md\)" docs/archive/ -P`（应 **0 命中**；`-P` 不可用时改用逐文件 `rg -n "](" docs/archive/` 人工核对 6 处）<br>`for f in docs/archive/*.md; do rg -n "$(basename $f)" AGENTS.md README.md README.zh-CN.md docs/ .codebuddy/; done` |
| **回滚方式** | 逐个 `git mv docs/archive/<X> docs/<原路径>` 反向移动；对 6 处链接修正执行 `git checkout --`（前提：第 0 步复核非 dirty）。回滚前先 `git diff -- docs/ > .codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/rollback/T5.patch`。**若任一文件第 0 步复核为 dirty → 禁止 `git mv`、禁止 checkout，改走 Group B「原位加指针」流程并上报。** |
| **预估复杂度** | M（10 次移动 + 6 处链接 + AC-17 逐条核验） |
| **状态** | `ready` |

---

### T6 · Group B 原位加指针 + `docs/REFACTOR_DESIGN_PHASE1.md` SUPERSEDED 指针

| 字段 | 内容 |
|---|---|
| **id** | `T6` |
| **标题** | AC-17 阻断的 2 份文档原位加指针 + dirty 文件头部追加 SUPERSEDED 指针（Q5=A、Q9=A） |
| **目标** | 对不可移动的文档给出明确"历史记录"标识，避免读者误引 |
| **批次** | **Batch 3** |
| **依赖** | `DESIGN-001` |
| **crate** | 无（文档） |
| **files_owned** | `docs/mcp-rmcp-feasibility.md`（AC-17 阻断：`docs/mcp.md:179` 引用）<br>`docs/compact-native-migration-retrospective.md`（AC-17 阻断：`docs/compact-durable-checkpoint-design.md:9` 引用）<br>`docs/REFACTOR_DESIGN_PHASE1.md`（**dirty**，Q9=A：不移动、不删改既有内容） |
| **改动内容** | ① 前两份在**标题行之后**各**插入 1 行**指针：<br>`> [ARCHIVED-IN-PLACE] 本文档为历史记录；因被 docs/mcp.md:179 / docs/compact-durable-checkpoint-design.md:9 引用，按 AC-17 不移动。`<br>（措辞可微调，但必须含 `ARCHIVED-IN-PLACE` 标记与阻断原因）<br>② `docs/REFACTOR_DESIGN_PHASE1.md`：**仅在第 1 行 `# PHASE-1: System Analysis & Refactor Design` 之后、第 3 行 `> [INFO] Lead Orchestrator...` 之前插入 1 行**：<br>`> [SUPERSEDED BY docs/phase1-refactor-design.md]`<br>**硬约束**：该文件是 **dirty 未提交** —— **只追加 1 行，禁止移动、禁止改写 `:1-8` 任何既有内容**（Q5=A、Q9=A、N10）<br>③ Group A 中指向本任务两份文档的外链，由 **T5** 在其 own 的文件内改为 `../<filename>`（跨任务协作点，T6 不越界改 T5 的文件） |
| **验收标准** | ① `rg -n "ARCHIVED-IN-PLACE" docs/mcp-rmcp-feasibility.md docs/compact-native-migration-retrospective.md` → 各 1 命中<br>② `rg -n "SUPERSEDED BY docs/phase1-refactor-design.md" docs/REFACTOR_DESIGN_PHASE1.md` → 1 命中<br>③ `git diff -- docs/REFACTOR_DESIGN_PHASE1.md` 的 diff **只有 1 行 `+`、0 行 `-`（除插入上下文外无删除）**<br>④ `docs/mcp.md:179`、`docs/compact-durable-checkpoint-design.md:9` 的相对链接仍可解析（因目标未移动） |
| **验收命令** | `rg -n "ARCHIVED-IN-PLACE" docs/mcp-rmcp-feasibility.md docs/compact-native-migration-retrospective.md`<br>`rg -n "SUPERSEDED BY docs/phase1-refactor-design.md" docs/REFACTOR_DESIGN_PHASE1.md`<br>`git diff --numstat -- docs/REFACTOR_DESIGN_PHASE1.md`（须为 `1  0`）<br>`git diff -- docs/REFACTOR_DESIGN_PHASE1.md \| rg -c "^-"`（须 0 行删除）<br>`test -f docs/mcp-rmcp-feasibility.md && test -f docs/compact-native-migration-retrospective.md && echo OK` |
| **回滚方式** | ⚠️ `docs/REFACTOR_DESIGN_PHASE1.md` 是 **dirty 文件** → **人工 diff 还原，禁止 `git checkout` / `git reset` / `git stash`**（N10）。回滚前先 `git diff -- docs/REFACTOR_DESIGN_PHASE1.md > .codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/rollback/T6.patch`，随后由人工逐个 hunk 反向编辑。前两份文件若复核为非 dirty，可 `git checkout --` 回滚。 |
| **预估复杂度** | S（3 行追加；但 dirty 文件需人工操作，谨慎度高） |
| **状态** | `ready` |

---

### T9 · 集成验证与交付登记

| 字段 | 内容 |
|---|---|
| **id** | `T9` |
| **标题** | 全量集成验证（AC-1…AC-17）与交付报告登记 |
| **目标** | 收敛全部批次，产出可交付的验证证据与风险登记 |
| **批次** | **Batch 4** |
| **依赖** | `T1`、`T3`、`T4`、`T5`、`T6`、`T7` 全部 `done` |
| **crate** | 全工作区（只读验证） |
| **files_owned** | `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/03-impl/T9-integration-verification.md` |
| **改动内容** | 执行 §3 的**全量验证序列**，逐条记录命令输出；登记：① 实际归档份数（10）与原位加指针份数（2）及阻断原因 ② `rustcode codingplan` → exit 2 的 breaking change ③ Q8 重复渲染器已知重复 ④ `crates/` 下非本 feature 的 7 处失效路径注释（follow-up） ⑤ `AGENTS.md:278` 表述状态（取决于 T8 是否获批） |
| **验收标准** | AC-1…AC-17 全部通过（AC-18 不适用）；AC-14 失败集与基线一致，唯一允许失败为 `trust_key_golden_matches_core_algorithm` |
| **验收命令** | 见 §3 全量序列 |
| **回滚方式** | 删除报告文件即可；本任务零代码/零文档改动，无回滚面（若验证失败，回滚的是对应前序任务，而非本任务） |
| **预估复杂度** | L（全量 `cargo test -j 1 --workspace` 耗时长，8GB cgroup 限制） |
| **状态** | `pending`（依赖前序任务） |

---

### T8（OPTIONAL / BLOCKED）· `AGENTS.md:278` 表述同步

| 字段 | 内容 |
|---|---|
| **id** | `T8` |
| **标题** | （**可选，需用户确认**）同步 `AGENTS.md:278` 关于 CLI `codingplan` 别名的表述 |
| **目标** | 使规范性文档与代码一致 |
| **批次** | **不入批次**（默认不执行） |
| **依赖** | `T3` done **且用户书面确认** |
| **crate** | 无（维护规则文件） |
| **files_owned** | `AGENTS.md`（**dirty**） |
| **改动内容** | 若获批：将 `AGENTS.md:278` 中"原表称 `rustcode codingplan` 作为隐藏别名保留……该别名已移除"的措辞与行号引用（`main.rs:4339`）校准为实际行号（删除前为 `:4903`）。**不得改动 `:68`/`:92`/`:534`/`:549` 对 `docs/REFACTOR_DESIGN_PHASE1.md` 的规范性引用（N7）。** |
| **验收标准** | 若执行：`git diff -- AGENTS.md` 只含本条表述的修改，无其他 hunk；`rg -n "REFACTOR_DESIGN_PHASE1" AGENTS.md` 命中数不变（4 处） |
| **验收命令** | `git diff -- AGENTS.md`（人工逐 hunk 审阅）<br>`rg -n "REFACTOR_DESIGN_PHASE1" AGENTS.md`（须仍为 4 处）<br>`rg -rni "atomcode" crates/ scripts/ .github/`（须 0 命中） |
| **回滚方式** | ⚠️ `AGENTS.md` 是 **dirty 文件** → **人工 diff 还原，禁止 `git checkout` / `git reset` / `git stash`**（N10）。回滚前先 `git diff -- AGENTS.md > .codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/rollback/T8.patch`。 |
| **预估复杂度** | S |
| **状态** | `blocked` — **`decision: 需用户确认`**，下一跳：`project-manager` → 用户 |
| **⚠ 说明** | 删除别名（T3）后，`AGENTS.md:278` 的"该别名已移除"表述**由失真变为准确**，故**不执行 T8 也不会产生新的文档/代码矛盾**；`site/docs/{en,zh}/headless-daemon.html` 已按"CLI 没有单独的 `rustcode codingplan` 命令"书写。因此本任务默认不执行，仅在交付报告登记。 |

---

## 2. 并行批次

### 并行安全性校验

| 批次 | 任务 | `files_owned` 并集 | 两两不相交 | 跨 crate | 任务数 |
|---|---|---|---|---|---|
| **Batch 1** | `T1` | 6 个 `.rs`（codingplan ×5 + config ×1） | —（单任务） | ⚠️ **是**（codingplan + config）→ 按规则**独占批次串行** | 1 |
| **Batch 2** | `T3`、`T4`、`T7` | `crates/rustcode-cli/src/main.rs` ／ `docs/config.example.toml` ／ `.codebuddy/.../03-impl/T7-dead-code-scan.md` | ✅ 三者**两两不相交** | ❌ 否（T3 单 crate；T4/T7 非 crate） | 3（= 上限） |
| **Batch 3** | `T5`、`T6` | T5：10 个 `docs/*.md`（新旧路径）+ `docs/archive/` ／ T6：`docs/mcp-rmcp-feasibility.md`、`docs/compact-native-migration-retrospective.md`、`docs/REFACTOR_DESIGN_PHASE1.md` | ✅ **两两不相交**（T5 的 10 份与 T6 的 3 份无重叠） | ❌ 否（全文档，无 crate） | 2 |
| **Batch 4** | `T9` | `.codebuddy/.../03-impl/T9-integration-verification.md` | —（单任务） | ❌ 否 | 1 |
| **不入批次** | `T8`（blocked，需用户确认） | `AGENTS.md`（dirty） | — | — | 0 |

**校验结论**：同批次 ≤ 3 ✅；`files_owned` 两两不相交 ✅；跨 crate 改动（`T1`）已独占批次串行 ✅；单批并行 ≤ 3 ✅。

### Batch 执行顺序与批次后验证

```
Batch 1（串行）  T1  ──►  批次后验证 V1
Batch 2（并行）  T3 ‖ T4 ‖ T7  ──►  批次后验证 V2
Batch 3（并行）  T5 ‖ T6  ──►  批次后验证 V3
Batch 4（串行）  T9  ──►  全量验证 V4
[T8 不入批次；需用户确认后才可插入 Batch 4 之前]
```

- **Batch 1 与 Batch 2 无文件交集**，理论上可合并并行；但 `T1` 跨 crate 需串行，且 `T1`/`T3` 都要跑 `cargo check --workspace`，同批次并行会争抢 `target/` 锁与 8GB cgroup 内存（`AGENTS.md:541`）→ **刻意保持批次串行以降低 OOM/SIGBUS 风险**。
- **Batch 3 与 Batch 1/2 无文件交集**（纯 `docs/`），但 `T5`/`T6` 完成后才能跑 AC-17 全量断链终验 → 排在 Batch 3 之后由 V4 统一执行。

---

## 3. 批次后验证命令

### V1（Batch 1 完成后）

```bash
rg -n "rustcode-core/src/coding_plan" crates/          # 期望 0 命中
rg -n "core/coding_plan/setup.rs" crates/              # 期望 0 命中
rg -n "mod codingplan_crypto_tests" crates/rustcode-config/src/i18n/
rg -n "LEGACY_CODINGPLAN_PREFIX|is_codingplan_provider_name" crates/
rg -rni "atomcode" crates/ scripts/ .github/           # 期望 0 命中
rg -n "codingplan_sync.json" crates/                   # 期望仍为 sync_marker.rs:22 + tuix 注释
rg -n "read_last_sync\(\)" crates/rustcode-tuix/src/lib.rs crates/rustcode-tuix/src/event_loop/mod.rs
rg -n "^use rustcode_codingplan" crates/rustcode-tuix/src/modals/usage.rs
cargo build
cargo check --workspace --all-targets
cargo check -p rustcode-codingplan --features client --all-targets
cargo check -p rustcode-tuix --features codingplan --all-targets
cargo check -p rustcode-daemon --features codingplan --all-targets
cargo check -p rustcode --features codingplan --all-targets
cargo check -p rustcode --features codingplan-crypto --all-targets
cargo check -p rustcode-daemon --features codingplan-crypto --all-targets
cargo test -p rustcode-codingplan --lib
cargo test -p rustcode-codingplan --lib sync_marker
cargo test -p rustcode-config --lib
cargo test -p rustcode-tuix --lib
cargo fmt --check
```

### V2（Batch 2 完成后）

```bash
rg -n "Codingplan" crates/rustcode-cli/src/main.rs     # 期望 0 命中
rg -n "CodingPlan" docs/config.example.toml            # 期望恰 1 命中
cargo build
cargo check --workspace --all-targets
cargo check -p rustcode --features codingplan --all-targets
cargo test -p rustcode shell_completion
cargo test -p rustcode --lib neutral_build_hides_managed_login_subcommands
cargo fmt --check                                       # 差异集不得扩大（main.rs 有 19 处存量违规）
test -f .codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/03-impl/T7-dead-code-scan.md
```

### V3（Batch 3 完成后）

```bash
ls docs/archive/                                        # 期望 10 个文件
test ! -f docs/coding-runtime-incremental-migration.md && echo OK
rg -n "ARCHIVED-IN-PLACE" docs/mcp-rmcp-feasibility.md docs/compact-native-migration-retrospective.md
rg -n "SUPERSEDED BY docs/phase1-refactor-design.md" docs/REFACTOR_DESIGN_PHASE1.md
git diff --numstat -- docs/REFACTOR_DESIGN_PHASE1.md    # 期望 "1  0"
rg -n "](" docs/archive/ | rg -v "http"                 # 人工核对 6 处 ../ 前缀
for f in docs/archive/*.md; do rg -n "$(basename $f)" AGENTS.md README.md README.zh-CN.md docs/ .codebuddy/; done
```

### V4（Batch 4 · 全量）

```bash
cargo build                                             # AC-1
cargo check --workspace --all-targets                   # AC-2
cargo check -p rustcode-codingplan --features client --all-targets   # AC-3
cargo check -p rustcode-tuix --features codingplan --all-targets     # AC-4
cargo check -p rustcode-daemon --features codingplan --all-targets   # AC-4
cargo check -p rustcode --features codingplan --all-targets          # AC-4
cargo check -p rustcode --features codingplan-crypto --all-targets   # AC-5
cargo check -p rustcode-daemon --features codingplan-crypto --all-targets  # AC-5
cargo test -p rustcode-codingplan --lib                 # AC-6
cargo test -p rustcode-codingplan --lib sync_marker     # AC-9
cargo test -p rustcode-config --lib                     # AC-13
cargo test -p rustcode-tuix --lib                       # AC-8
cargo test -p rustcode shell_completion                 # AC-16
cargo test -p rustcode --lib neutral_build_hides_managed_login_subcommands  # AC-16
cargo fmt --check                                       # AC-15（不新增）
cargo test -j 1 --workspace --no-fail-fast              # AC-14（必须 -j 1）
```

**AC-14 判定**：失败集须与无改动基线一致；**唯一允许失败为 `mcp::registry::tests::trust_key_golden_matches_core_algorithm`**（`AGENTS.md:226`、`:350`、`:540`）。若出现任何其他失败 → `T9` 置 `changes_requested`，`decision` 写明归零路径，禁止"先交付后修"。
**AC-18**：S2 档**不适用**（无需执行）。

---

## 4. 集成顺序说明

1. **合并顺序**：`T1 → (T3, T4, T7) → (T5, T6) → T9`。每批次合并前，该批次所有任务须自验通过（G3：`cargo check -p <crate> --all-targets` + 本任务验收命令全绿 + 改动不超 `files_owned`）。
2. **批次间门禁**：
   - Batch 1 → 2：V1 全绿方可开始 Batch 2（Batch 2 的 `T3` 会在 `main.rs` 上改动，需以 V1 的干净基线为起点，便于定位回归）。
   - Batch 2 → 3：V2 全绿方可开始 Batch 3。Batch 3 是纯文档面，与代码面无耦合，若进度紧张可在 V2 绿后与 Batch 4 的准备工作重叠。
   - Batch 3 → 4：V3 全绿方可开始 T9。
3. **AC-17 终验归属 T9 而非 T5**：`T5`、`T6` 同在 Batch 3 并行，二者都完成前无法判定全仓断链情况；故 T5/T6 各自只做**本任务 `files_owned` 内**的链接自验，AC-17 全量终验由 `T9` 统一执行。
4. **`T8` 插入点**：若用户确认执行 `T8`，应在 **Batch 3 之后、Batch 4 之前**插入（需重跑 V4 中的 `rg -rni "atomcode" crates/ scripts/ .github/` 与 `rg -n "REFACTOR_DESIGN_PHASE1" AGENTS.md`）。默认不执行。
5. **返工上限**：单任务返工至多 2 轮，第 3 轮必须升级给用户（`.codebuddy/rules` §3）。
6. **失败即停**：任一任务验收命令红 → 该任务置 `changes_requested` 并写明 `decision` 与下一跳；**禁止**为推进批次而放宽判据、跳过 AC 或改契约。契约争议一律回退 `solution-architect`。

---

## 5. 修订记录

| 日期 | 版本 | 变更 |
|---|---|---|
| 2026-09-02 | v1 | 首版。8 个任务（7 个入批次 + 1 个 blocked optional），4 个批次。 |
