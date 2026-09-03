---
kind: requirement
id: REQ-001
from: requirements-analyst
to: [solution-architect]
feature: 2026-09-02-cleanup-codingplan-legacy
status: blocked
decision: escalate
requires: []
files_owned:
  - 00-requirement.md
architecture_constraints:
  touches_runtime_lifecycle: false
  touches_persistence: true
  touches_cross_crate_deps: true
created: 2026-09-02
---

# REQ-001 CodingPlan 遗留内容清理

> **只读分析产物。** 本轮仅落盘本文件，未修改任何生产代码 / 文档 / `Cargo.toml` / `AGENTS.md`。
> **基线**：branch=`dev` commit=`8e772dbf`，worktree dirty（用户未提交改动）。本轮**未**执行 `git status` / `git log`（无 shell 权限），涉及"某文件是否 dirty / 近期提交"的判定一律标注**证据不足**。

---

## 1. 背景与问题

### 1.1 用户原始诉求（逐字引用，未改写）

> 请仔细审查所有遗留的 CodingPlan 内容，不要保留任何旧版本或过时的方案。请自行查看和分析现有代码与计划，识别并清除所有不再适用或已废弃的部分，确保最终只保留最新且有效的 CodingPlan。

### 1.2 现状

本 fork 已完成 [OBJECTIVE-1..6]（重命名 / 零遥测 / Provider 解耦与平台中立 / 默认中文 / 子代理双车道），见 `AGENTS.md:61-117`。CodingPlan 相关内容经逐条源码复核，事实分层如下。

#### A 类 — 默认编译、有真实消费者、**有效**（不得删）

| 文件 | 行数 | 门控 |
|---|---|---|
| `crates/rustcode-codingplan/src/types.rs` | 706 | 无（`lib.rs:26` `pub mod types;`） |
| `crates/rustcode-codingplan/src/usage.rs` | 262 | 无（`lib.rs:27`） |
| `crates/rustcode-codingplan/src/sync_marker.rs` | 151 | 无（`lib.rs:25`） |

**默认编译的真实消费者（逐条 `#[cfg]` 复核后确认，非 cfg 块内）**：

- `crates/rustcode-tuix/src/lib.rs:872` — `monitor_last_sync_seen: rustcode_codingplan::read_last_sync()`
- `crates/rustcode-tuix/src/event_loop/mod.rs:3963` — `LoopCtx.usage_slot` 字段类型含 `rustcode_codingplan::types::UsageInfo`（结构体字段，非 cfg）
- `crates/rustcode-tuix/src/event_loop/mod.rs:12304-12314` — `refresh_after_cross_process_codingplan_sync()` 调 `read_last_sync()`，无 cfg
- `crates/rustcode-tuix/src/modals/usage.rs:6-7` — `use rustcode_codingplan::types::{PlanInfo, RateLimitWindow}`、`usage::{compute_overview, humanize_tokens, OverviewStats, UsageResponse}`；`crates/rustcode-tuix/src/modals/mod.rs:91` `pub mod usage;` 无 cfg
- `crates/rustcode-tuix/src/event_loop/usage_monitor.rs:21` — `use rustcode_codingplan::types::UsageInfo;`（模块顶，无 cfg）

**持久化**：`crates/rustcode-codingplan/src/sync_marker.rs:22` `FILE_NAME = "codingplan_sync.json"`；`:33-35` `Config::config_dir().join(FILE_NAME)`（即 `$RUSTCODE_HOME/codingplan_sync.json`）；`:40-45` `read_last_sync()`；`:52+` `write_last_sync_now()`。该文件由 `sync_marker.rs` 独占读写。

#### B 类 — 托管平台网关流程，`client` feature 默认关闭

- `crates/rustcode-codingplan/src/client.rs`（602 行 blocking reqwest REST）、`setup.rs`（**3911 行**，末行已复核至 `:3911`；含内联单测）
- `crates/rustcode-daemon/src/api_codingplan.rs`（677 行 axum 路由，末行 `:677` 已复核）
- 门控：`crates/rustcode-codingplan/Cargo.toml:9` `default = []`；`:15` `client = ["dep:reqwest"]`；`src/lib.rs:21-24` `#[cfg(feature="client")] pub mod client/setup`
- 驱动侧 feature 链：`crates/rustcode-cli/Cargo.toml:29-38`（`codingplan` → `rustcode-codingplan/client` + `rustcode-daemon/codingplan` + `rustcode-tuix/codingplan`；`codingplan-crypto` → `rustcode-auth/codingplan-crypto` + `codingplan`）；`crates/rustcode-daemon/Cargo.toml:21-22`；`crates/rustcode-tuix/Cargo.toml:14`
- 消费者**全部在 cfg 块内**：`crates/rustcode-cli/src/main.rs:4544-4625`（cfg 于 `:4544`）、`crates/rustcode-tuix/src/event_loop/commands.rs:7110-7290`（cfg 于 `:7110`）、`crates/rustcode-daemon/src/runtime_host.rs:48-83`、`crates/rustcode-daemon/src/lib.rs:29/54`

#### C 类 — 闭源签名桩 + 适配层，**默认编译**（非"死代码"）

- `crates/rustcode-codingplan-crypto/src/lib.rs`（29 行桩：`:11` `ALGORITHM_VERSION: u8 = 0`；`:28` `unreachable!`）
- `crates/rustcode-auth/src/gateway_crypto.rs` — `:49-55` 桩 `signer()` / `:57-88` 真实 `RealSigner`；`:91/96` `signer_available()`；`:100` `is_codingplan_gateway()`
- `crates/rustcode-auth/Cargo.toml:21/25` — `codingplan-crypto = ["dep:rustcode-codingplan-crypto"]`
- `crates/rustcode-capabilities/src/provider/codingplan_sign.rs`（153 行）— `provider/mod.rs:19` `mod codingplan_sign;`（**无 cfg**）、`:28` 重导出
- **默认编译的真实消费者（重要，编排者清单未列）**：`crates/rustcode-coding/src/provider_factory.rs:5,52,55,60`（`is_codingplan_gateway` / `signer_available` / `codingplan_request_signer`，无 cfg）、`crates/rustcode-daemon/src/main.rs:151`、`crates/rustcode-tuix/src/event_loop/mod.rs:28520-28521`、`crates/rustcode-tuix/src/modals/provider_panel.rs:885`、`crates/rustcode-clix/src/main.rs:886`、`crates/rustcode-daemon/src/api_provider.rs:43/55`、`crates/rustcode-daemon/src/api_config.rs:85/155`
- `Cargo.toml:8-17` — `default-members` 刻意不含 `rustcode-codingplan-crypto`；官方构建走 `--features rustcode/codingplan-crypto`

#### D 类 — 已确认过时痕迹

| # | 位置 | 内容 | 判定 |
|---|---|---|---|
| D1 | `crates/rustcode-codingplan/src/lib.rs:1` | `// crates/rustcode-core/src/coding_plan/mod.rs` | 失效路径（core 已退役）→ **可修** |
| D2 | `.../types.rs:1`、`.../client.rs:1`、`.../setup.rs:1`、`.../sync_marker.rs:1` | 同为 `crates/rustcode-core/src/coding_plan/*.rs` | 失效路径，**编排者清单只列了 `lib.rs:1`，本轮另发现 4 处** → **可修** |
| D3 | `crates/rustcode-config/src/i18n/messages.rs:319` | `// SetupReport renderer (core/coding_plan/setup.rs)` | 失效路径 → **可修** |
| D4 | `crates/rustcode-config/src/i18n/messages.rs:376` | 注释引用 `coding_plan::setup::strikethrough` | 模块已迁至 `rustcode-codingplan::setup`；但 `Cp*` 变体本体受 AGENTS.md:338 保护 → **仅修注释，不删变体** |
| D5 | `docs/config.example.toml:38-55` vs `:149-167+` | 两段 CodingPlan 网关示例重复（英文段 `providers.RustCode-deepseek-v4-flash` / `RustCode-Qwen-Qwen3-VL-8B-Instruct`；中文段 `providers."RustCode-GLM-5.2"` / `"RustCode-deepseek-v4-flash"` / `"RustCode-Qwen-Qwen3-VL-8B-Instruct"`，两段均 `gateway.example.com`） | **可去重** |
| D6 | `crates/rustcode-config/src/config/mod.rs:1192` `LEGACY_CODINGPLAN_PREFIX = "AtomGit"`、`:1212` `prefixes_for`、`:1221` `codingplan_prefixes()`、`:1233` `is_codingplan_provider_name()` | 旧平台前缀兼容逻辑 | **受 AGENTS.md:213-214 G7 第(2)类明示"旧配置键仍需识别，勿删"保护** → **不得删**，见 Q3 |
| D7（附带发现，非 codingplan） | `crates/rustcode-tuix/tests/plugin_integration.rs:1`、`crates/rustcode-tuix/src/custom_commands.rs:1`、`crates/rustcode-config/src/config/instructions.rs:1` | 同为失效 core 路径头注释 | 不在本 feature 范围内，仅登记 |

#### 文档面（复核确认 18 个文件命中 `CodingPlan|codingplan`）

`docs/REFACTOR_DESIGN_PHASE1.md`、`docs/phase1-refactor-design.md`、`docs/platform-neutralization.md`、`docs/release-v5.0.1-current-branch-change-report.md`、`docs/coding-runtime-incremental-migration.md`、`docs/coding-runtime-native-migration-design.md`、`docs/mcp-rmcp-feasibility.md`、`docs/config.example.toml`、`docs/testing/release-v5.0.3-core-retirement-acceptance.md`、`docs/plans/2026-07-25-provider-retry-consolidation.md`、`docs/plans/2026-07-26-provider-accounts-model-profiles-design.md`、`docs/plans/2026-07-26-provider-accounts-model-profiles-plan.md`、`docs/plans/2026-07-27-models-dev-pricing-design.md`，以及白名单 5 个：`docs/i18n-style.md`、`docs/i18n-field-mapping.md`、`docs/architecture.md`、`docs/agent-api-rfc.md`、`docs/UPSTREAM_CREDITS.md`。

**白名单依据（不得建议删除）**：`docs/architecture.md` 被 `AGENTS.md:7/216/533/548` 引用；`docs/i18n-style.md`、`docs/i18n-field-mapping.md` 为 i18n 规范；`docs/agent-api-rfc.md` 为 DRAFT；`docs/UPSTREAM_CREDITS.md` 被 `README.md:60/169` 与 `README.zh-CN.md:46` 引用（MIT 归属，AGENTS.md:539 明示删改有合规风险）；`docs/config.example.toml` 被 `README.md:408`、`README.zh-CN.md:365` 引用。

**三处对编排者勘查结论的事实校正（重要）**：

1. **不是"成对同源"的三组文档实际是相邻阶段、互相交叉引用的序列文档，不能"保留较新一份、删旧一份"**：
   - `docs/coding-runtime-incremental-migration.md:8-9` 交叉引用 `compact-native-migration-retrospective.md`；`docs/coding-runtime-native-migration-design.md:13-14` 反向交叉引用 `coding-runtime-incremental-migration.md`。二者是"渐进迁移"与"全量原生迁移"两个阶段，非重复。
   - `docs/session-convergence-plan.md:9` 明确"后续 live transport 的实施结果见 `live-transport-convergence-plan.md`"，二者是 S0-S5 与 LT0-LT5 两个不同收敛面。
   - `docs/plans/` 下 `*-design.md` + `*-plan.md` 成对是仓库既有惯例（另有 `2026-07-28-rewind-design.md` / `rewind-implementation-plan.md`、`2026-08-20-code-review-deep-mode-fanout-design.md` / `-plan.md`），**不是重复**。
2. **唯一真正的 supersede 关系**：`docs/phase1-refactor-design.md:7` 自述 `[SUPERSEDES] docs/REFACTOR_DESIGN_PHASE1.md`（基线 main/287bff70，重命名之前）。但 `docs/REFACTOR_DESIGN_PHASE1.md` 仍被 `AGENTS.md:68`（决策 D1 §2.0）、`:92`（§4.3 G6）、`:534`（决策 D1 + commit 6dbf57bb）、`:549`（高信号文档索引）**四处规范性引用** → 按白名单规则不可删，见 Q5。
3. **CLI `codingplan` 隐藏别名仍然存在**（与 `AGENTS.md:278` 的"该别名已移除"表述不符）：`crates/rustcode-cli/src/main.rs:1022-1026` 定义 `#[command(hide = true)] Codingplan`，`:1696` `Commands::Login | Commands::Codingplan =>` **无 cfg 门控**，默认构建下落至 `:4517-4520` 的中性 stub（`Msg::CliManagedLoginNotBuilt`）。另有 `:3574` 的 `unreachable!` 兜底臂。补全脚本断言 `!script.contains("codingplan")` 在 `:4903`。**TUI `/codingplan` 斜杠命令已不存在**（`crates/rustcode-tuix/src/commands.rs` 中仅 `:191`（`/login` desc）与 `:216`（`/usage` desc）命中 "CodingPlan" 字样，无 codingplan 命令条目）——这缩小了 S3 的实际爆破半径。

### 1.3 问题陈述

CodingPlan 相关内容中，**真正"不再适用或已废弃"的是 D 类过时痕迹与一批已完结的方案文档**；而 B/C 类是**刻意保留的 opt-in 门控开关**（`AGENTS.md:50`、`:338`、`:530` 三处明示"gate 不删"铁律）。用户诉求"不要保留任何旧版本或过时的方案"若被直接读作"删除 B/C 类"，将与上述铁律及官方闭源 overlay 的构建入口（`Cargo.toml:11-12`）冲突。因此本需求以**三档范围提案**形式交由用户裁决，而非由 agent 自选。

---

## 2. 目标 / 非目标

### 2.1 目标

- G1 清除所有指向已退役 `rustcode-core` 的失效路径注释（D1-D4）。
- G2 消除 `docs/config.example.toml` 中重复的 CodingPlan 网关示例段（D5），保留且仅保留一段。
- G3 归档已完结/过时的 CodingPlan 相关方案文档，并在被 supersede 的文档头部加指针，使读者不会误引历史结论。
- G4 在选定档位内清除无消费者的死代码与跨 crate 重复实现。
- G5 全程不回归默认构建、不回归 A 类消费者行为、不回归 `$RUSTCODE_HOME/codingplan_sync.json` 读写语义。

### 2.2 非目标（防范围蔓延，**本需求的硬边界**）

- **N1 不改变 A 类模块（`types` / `usage` / `sync_marker`）的对外行为与文件布局**，不迁移、不重命名、不改 JSON schema。
- **N2 不重启 [OBJECTIVE-1..6] 的迁移**，不重新审视重命名 / 零遥测 / provider 解耦 / 默认中文 / 子代理双车道的既有结论。
- **N3 不改动 `rustcode-kernel`**；不触碰 `rustcode-capabilities` / `rustcode-coding` 的 core-free 生产依赖约束（`AGENTS.md:138`）。
- **N4 不恢复 core 兼容层、bridge、v1/v2 开关或任何 runtime fallback**（`AGENTS.md:143`、`:168`）。
- **N5 不新增第二运行时生命周期所有者**；`CodingRuntime` 仍是唯一 owner（`AGENTS.md:52`）。
- **N6 不自作主张删除受显式铁律保护的面**：`atomgit` Cargo feature、`#[cfg(feature="codingplan")]` 块、`LEGACY_CODINGPLAN_PREFIX` / `is_codingplan_provider_name`（`AGENTS.md:213-214` G7）、`Cp*` i18n 族与 `codingplan_crypto_tests`（`AGENTS.md:338`）、托管专属 QR 屏（`AGENTS.md:354`）——`AGENTS.md:530` 汇总为"gate 不删"。若用户裁决要求删除，须**书面升级**并在 Q3/Q4 中记录授权。
- **N7 不修改 `AGENTS.md`、`README.md`、`README.zh-CN.md`、`docs/architecture.md`、`.codebuddy/rules/**` 中的规范性引用**（这些文件是本需求的白名单依据；如需改动，另开 feature）。
- **N8 不做 i18n 文案改写**（不改任何 `Msg` arm 的 en/zh 正文），仅允许修注释。
- **N9 不改动 `Cargo.toml` 的 `default-members`**（除非用户明确选择 S3）。
- **N10 不执行任何 `git checkout` / `git reset` / `git stash`**；worktree 为 dirty，用户改动必须原样保留。

---

## 3. 用户故事

- **US-1** 作为**维护者**，我希望源码中不再出现指向已退役 `rustcode-core/src/coding_plan/*` 的路径注释，以便新读者不会被导向不存在的模块。
- **US-2** 作为**新用户**，我希望 `docs/config.example.toml` 里只有一段 CodingPlan 网关示例，以便不会因两份不一致的示例而配错。
- **US-3** 作为**维护者**，我希望已完结的迁移方案文档被归档、被 supersede 的文档头部有指针，以便我能一眼分辨"当前有效"与"历史记录"。
- **US-4** 作为**维护者**，我希望无消费者的死代码与逐字重复的跨 crate 实现被清除，以便改动时只需改一处。
- **US-5** 作为**开源构建用户**，我希望清理后默认构建与 TUI 用量面板行为完全不变，以便升级无感。
- **US-6** 作为**发行版打包者**，我希望（若选择 S3）`--features rustcode/codingplan-crypto` 的存废有明确结论，以便我决定闭源 overlay 的接入方式。

---

## 4. 验收标准（AC）

> **验证命令约定**：CLI 二进制的包名是 `rustcode`（`crates/rustcode-cli/Cargo.toml:2`），`-p rustcode-cli` 会失败——一律用 `-p rustcode`。
> **环境约束**（`AGENTS.md:541`）：cgroup 内存上限 8GB，`cargo test --workspace` **必须加 `-j 1`**，否则 rustc SIGBUS。
> **已知红**（`AGENTS.md:226`、`:350`）：`mcp::registry::tests::trust_key_golden_matches_core_algorithm`（`DefaultHasher` 跨工具链不稳定，铁律禁改）是**唯一允许的既有失败**，任何 AC 不得要求它变绿，也不得要求本次改动使其变红。

| # | 前置条件 | 操作 | 期望结果 | 验证命令 |
|---|---|---|---|---|
| **AC-1** | worktree 保持 dirty，不重置 | 编译默认成员 | `cargo build` exit 0，无 error | `cargo build` |
| **AC-2** | 同上 | 全工作区编译校验（含 tests/benches/examples） | exit 0，无 error；warning 数不增加 | `cargo check --workspace --all-targets` |
| **AC-3** | 同上 | 校验 `codingplan` feature 组合可编译 | exit 0 | `cargo check -p rustcode-codingplan --features client --all-targets` |
| **AC-4** | 同上 | 校验驱动 crate 的 `codingplan` feature 传递链可编译（tuix / daemon / cli 三条） | 三条 exit 0 | `cargo check -p rustcode-tuix --features codingplan --all-targets`<br>`cargo check -p rustcode-daemon --features codingplan --all-targets`<br>`cargo check -p rustcode --features codingplan --all-targets` |
| **AC-5** | 同上 | 校验闭源签名 overlay 的两种接入路径仍可编译（**这是 S3 的生死线**） | 两条 exit 0 | `cargo check -p rustcode --features codingplan-crypto --all-targets`<br>`cargo check -p rustcode-daemon --features codingplan-crypto --all-targets` |
| **AC-6** | 同上 | A 类模块单测 | `rustcode-codingplan` lib 测试全绿（编排者基线 27/0，本轮未复跑） | `cargo test -p rustcode-codingplan --lib` |
| **AC-7** | 同上 | A 类默认消费者仍在且行为不变：grep 断言 | 三条 grep 均命中且行号不变（± 允许整体位移，但表达式必须存在）：<br>① `crates/rustcode-tuix/src/lib.rs` 含 `rustcode_codingplan::read_last_sync()`<br>② `crates/rustcode-tuix/src/event_loop/mod.rs` 含 `fn refresh_after_cross_process_codingplan_sync` 且体内含 `rustcode_codingplan::read_last_sync()`<br>③ `crates/rustcode-tuix/src/modals/usage.rs` 含 `use rustcode_codingplan::types::{PlanInfo, RateLimitWindow};` 与 `use rustcode_codingplan::usage::{compute_overview, humanize_tokens, OverviewStats, UsageResponse};` | `rg -n "read_last_sync\(\)" crates/rustcode-tuix/src/lib.rs crates/rustcode-tuix/src/event_loop/mod.rs`<br>`rg -n "^use rustcode_codingplan" crates/rustcode-tuix/src/modals/usage.rs` |
| **AC-8** | 同上 | TUI 用量面板渲染不回归 | `cargo test -p rustcode-tuix --lib` 中与 usage / modals 相关的用例全绿（基线 tuix lib 2063/0，见 `AGENTS.md:368`） | `cargo test -p rustcode-tuix --lib usage` 及 `cargo test -p rustcode-tuix --lib` |
| **AC-9** | 同上 | `codingplan_sync.json` 持久化契约不变 | ① `crates/rustcode-codingplan/src/sync_marker.rs` 中 `const FILE_NAME: &str = "codingplan_sync.json";` 字面量不变；② `read_last_sync()` 对"文件缺失 / JSON 损坏 / 时间戳不可解析"三种输入仍返回 `None`（不返回 `Err`、不 panic）；③ 生产者/消费者不因清理而失配 | `cargo test -p rustcode-codingplan --lib sync_marker`<br>`rg -n "codingplan_sync.json" crates/`（必须仅 1 处定义 + 注释） |
| **AC-10** | 同上 | 失效 core 路径注释清零 | grep 在 `crates/` 下 0 命中 | `rg -n "rustcode-core/src/coding_plan" crates/` → 0 命中<br>`rg -n "core/coding_plan/setup.rs" crates/` → 0 命中 |
| **AC-11** | 同上 | `docs/config.example.toml` 去重 | 文件中 CodingPlan 网关示例段**恰好 1 段**：`rg -c -i "codingplan" docs/config.example.toml` 与 `rg -n "^# -+ .*CodingPlan" docs/config.example.toml` 各返回 1 行 | `rg -n "CodingPlan" docs/config.example.toml` |
| **AC-12** | 同上 | G7 门禁不回退 | ① `LEGACY_CODINGPLAN_PREFIX` 与 `is_codingplan_provider_name` 仍存在（除非用户按 Q3 选 B 并同步改门禁）；② `atomcode` 在 `crates/ scripts/ .github/` 0 命中（`AGENTS.md:209-217`） | `rg -n "LEGACY_CODINGPLAN_PREFIX\|is_codingplan_provider_name" crates/`<br>`rg -rni "atomcode" crates/ scripts/ .github/` |
| **AC-13** | 同上 | i18n 不劣化 | `rustcode-config` lib 测试全绿；`codingplan_crypto_tests` 模块仍存在且双语 arm 齐（编译期保证） | `cargo test -p rustcode-config --lib`（基线 327/0，见 `AGENTS.md:368`）<br>`rg -n "mod codingplan_crypto_tests" crates/rustcode-config/src/i18n/` |
| **AC-14** | 同上 | 全工作区测试不新增失败 | 失败集与基线一致，唯一允许失败为 `trust_key_golden_matches_core_algorithm` | `cargo test -j 1 --workspace --no-fail-fast` |
| **AC-15** | 同上 | 格式门禁 | `cargo fmt --check` 不新增差异（`AGENTS.md:542` 记录 19 处**存量**违规，位于 `crates/rustcode-cli/src/{main.rs,schedule_cmd.rs}`、`crates/rustcode-coding/src/runtime.rs`、`crates/rustcode-tuix/src/event_loop/*`，非本 feature 引入，不要求清零） | `cargo fmt --check` |
| **AC-16** | 同上 | 命令发现面不回归（S1/S2 档必过） | ① CLI 补全脚本不含 codingplan；② login about 不含 CodingPlan（中立构建） | `cargo test -p rustcode shell_completion`<br>`cargo test -p rustcode --lib neutral_build_hides_managed_login_subcommands`（`AGENTS.md:305`） |
| **AC-17** | 同上 | 文档归档后无断链 | 归档文档的内部相对链接在新路径下仍可解析；`AGENTS.md` / `README*.md` / `docs/architecture.md` 引用的文档路径全部存在 | 对每份被 `git mv` 的文档：`rg -n "<被移动文件名>" AGENTS.md README.md README.zh-CN.md docs/ .codebuddy/` 后逐条确认；若命中则**不得移动**，改为在原位加 `[SUPERSEDED]` 指针 |
| **AC-18** | 仅 S3 档 | 若移除 `/codingplan/*` 路由，对外协议变更已同步 | ① `crates/rustcode-daemon/README.md` 的 CodingPlan 节（`AGENTS.md:342`）已更新；② `site/docs/{en,zh}/headless-daemon.html` 的 `/codingplan/*` 行已删（`AGENTS.md:278`、`:310`） | `rg -n "codingplan" crates/rustcode-daemon/README.md site/docs/en/headless-daemon.html site/docs/zh/headless-daemon.html` |

---

## 5. 异常与非功能场景

| 类别 | 场景 | 期望行为 / 约束 |
|---|---|---|
| **失败** | 归档 `git mv` 时目标文件处于 dirty 未提交状态（据编排者，`docs/REFACTOR_DESIGN_PHASE1.md` 疑似 dirty；**本轮未执行 `git status`，证据不足**） | 中止该文件的移动，**禁止** `git checkout` / `git restore` / `git stash`；改为在原位加 `[SUPERSEDED BY ...]` 指针，并把该文件列入交付报告 |
| **失败** | 删除段落后 `docs/config.example.toml` 不再是合法 TOML 范例 | 该文件是被 `README.md:408` 引用的用户拷贝源，去重后必须仍可被用户直接拷改；若注释块边界含空行/注释符号歧义，只删**完整注释块**，不删结构性行 |
| **取消** | 用户在实施中途撤销 | 所有改动仅在 worktree 且未 commit；回滚只允许 `git checkout -- <未被用户改动的文件>`，dirty 文件一律人工 diff 后还原 |
| **超时** | `cargo test --workspace` 全量耗时过长 | 必须用 `-j 1`（`AGENTS.md:541`）；允许分 crate 先跑受影响面（config / codingplan / tuix / daemon / cli）再跑全量 |
| **并发** | tuix / config 测试存在全局 locale 竞态历史（`AGENTS.md:349`、`:366`、`:360`） | 若本次改动触及任何本地化输出路径，相关测试必须 `i18n::test_lock()` + `set_locale(Locale::En)` 钉死；本 feature 原则上不改 i18n 文案（N8），故理想情况下不触发 |
| **降级** | `/workspace` 磁盘 100% 满致链接器失败（历史多次，见 `AGENTS.md:343/350/368/386/394`） | 清 `target/debug/incremental`（可弃缓存）后重试；**禁止**删 `target/debug/deps` 以外的用户数据 |
| **权限** | 需要 root 才能写某路径 | **禁止 sudo**（`AGENTS.md:18`：`~/.rustcode` 一旦出现 root 属主文件，后续非 root 启动即失败） |
| **数据缺失** | `$RUSTCODE_HOME/codingplan_sync.json` 不存在 / JSON 损坏 / 时间戳溢出 | `sync_marker.rs:40-45` 的既有语义必须保持：全部视为"从未同步"，返回 `None`，**不得**改为返回 `Err` 或 panic；AC-9 覆盖 |
| **数据残留**（仅 S3） | 移除 B 类后 `codingplan_sync.json` 不再有生产者 | 必须同时移除 `sync_marker.rs` 与 `refresh_after_cross_process_codingplan_sync`（mod.rs:12304）及 `LoopCtx` 相关字段，否则留下无生产者的孤儿读取与残留文件；**这与 N1（不动 A 类）直接冲突，是 S3 必须解决的矛盾** |
| **feature 矩阵** | 只验默认 feature 而漏验 `client` / `codingplan` / `codingplan-crypto` | AC-3/4/5 强制覆盖；`AGENTS.md:51` 明确"默认 feature 下 `cargo test -p rustcode-codingplan` **不编译** `client.rs`/`setup.rs`" |
| **非功能** | 新增编译 warning | 不得增加；`cargo clippy --workspace --all-targets` 的存量 ~420 warning 不作为门禁（`AGENTS.md:220`），但本次改动文件不得新增 |
| **非功能** | 依赖方向 | 不得引入 `capabilities → driver`、`kernel → 上层` 等反向依赖（`AGENTS.md:49`）；G4 的"跨 crate 重复渲染器抽取"若需新增依赖，必须先过架构评审（Q8） |

---

## 6. 受影响范围（初步）

### 6.1 受影响的 crate 与模块路径

| crate | 路径 | 档位 |
|---|---|---|
| `rustcode-codingplan` | `src/lib.rs:1`、`src/types.rs:1`、`src/client.rs:1`、`src/setup.rs:1`、`src/sync_marker.rs:1` | S1（注释） |
| `rustcode-codingplan` | `src/client.rs`（602）、`src/setup.rs`（3911）、`Cargo.toml:9-28` | S3 |
| `rustcode-codingplan-crypto` | 整 crate（`src/lib.rs` 29、`Cargo.toml`） | S3 |
| `rustcode-config` | `src/i18n/messages.rs:319`、`:376`；`src/config/mod.rs:1192/1212/1221/1233` | S1（注释）/ S3（前缀逻辑，受限） |
| `rustcode-auth` | `src/gateway_crypto.rs`、`Cargo.toml:21/25` | S3 |
| `rustcode-capabilities` | `src/provider/codingplan_sign.rs`（153）、`src/provider/mod.rs:19/28` | S3 |
| `rustcode-coding` | `src/provider_factory.rs:5/52/55/60` | S3 |
| `rustcode-cli` | `src/main.rs:1022-1026`、`:1696`、`:3574`、`:4514-4520`、`:4544-4625`、`:4903`；`Cargo.toml:29-38` | S2/S3 |
| `rustcode-daemon` | `src/api_codingplan.rs`（677）、`src/lib.rs:29/54/6074/6431`、`src/commands.rs:622-705`、`src/runtime_host.rs:3-83`、`src/api_auth.rs:289`、`Cargo.toml:17-22`、`README.md` | S2/S3 |
| `rustcode-tuix` | `src/event_loop/commands.rs:31/4971-5066/5341/7018/7110-7290`、`src/event_loop/monitor.rs`、`src/event_loop/usage_monitor.rs`、`src/event_loop/mod.rs:18328/12304`、`src/modals/usage.rs`、`src/commands.rs:191/216`、`Cargo.toml:14` | S2/S3 |
| `rustcode-clix` | `src/main.rs:886` | S3 |
| 根 | `Cargo.toml:1-17`（`members` / `default-members` 与 codingplan-crypto 注释） | S3 |
| `docs/` | 见 §8 各档文件清单 | S1/S2 |

### 6.2 影响的入口（本次实际相关者）

- **CLI**：`rustcode codingplan`（隐藏别名，`main.rs:1026/1696`）、`rustcode login`（`:1696`）、`rustcode status` 的 auth hint（`:4517`/`:4532`）、shell 补全生成（`:4903`）。
- **TUI**：`/login`、`/usage`（`commands.rs:191/216`，受 `MANAGED_ONLY_COMMANDS` 门控，见 `AGENTS.md:285`）、用量 modal（`modals/usage.rs`）、状态栏用量提示（`event_loop/mod.rs:28520`）、**`/codingplan` 斜杠命令已不存在**（§1.2 校正 3）。
- **daemon**：`/codingplan/*` 路由族（`api_codingplan.rs`，`#[cfg(feature="codingplan")]`）、`/status` 的 CodingPlan 段（`commands.rs:622-705`）。
- **background**：`monitor.rs` / `usage_monitor.rs` 的用量轮询（cfg 门控）。
- **ACP**：`commands.rs` 的 `/usage` 广告经 `command_visible` 过滤（`AGENTS.md:285`）。
- **headless / clix**：clix 签名网关报错（`clix/src/main.rs:886`）。

---

## 7. 架构敏感面标记

| 敏感面 | 是否触碰 | 证据与说明 |
|---|---|---|
| **持久化格式** | **是** | `$RUSTCODE_HOME/codingplan_sync.json`，`sync_marker.rs:22/33-35/40-45/52`。S1/S2 不触碰；S3 会移除生产者 → 触碰持久化格式与残留文件治理 |
| **公共协议** | **是**（仅 S3） | daemon `/codingplan/*` axum 路由（`api_codingplan.rs`，677 行），已被 `site/docs/{en,zh}/headless-daemon.html` 与 `crates/rustcode-daemon/README.md` 文档化为"仅发行版本"（`AGENTS.md:278`、`:342`）。S1/S2 不触碰 |
| **安全边界** | **是**（仅 S3） | OAuth token 与请求签名：`gateway_crypto.rs:57-88` `RealSigner` 使用 `rustcode_codingplan_crypto::sign_v1`；`codingplan_sign.rs:28` 委派 `gateway_crypto::signer()`；`coding/src/provider_factory.rs:52-60` 的 `is_codingplan_gateway` + `signer_available` 双条件决定签名头是否附加。S1/S2 不触碰 |
| **运行时生命周期** | **否** | 本次清理不涉及 `CodingRuntime` 的 start/stop/cancel/approval/provider reload/session resume；无第二生命周期所有者引入（N5） |
| **跨 crate 依赖方向** | **是** | feature 传递链：cli（`Cargo.toml:29-38`）→ daemon（`:21-22`）→ codingplan（`:15`）；tuix（`:14`）→ codingplan/client；auth（`Cargo.toml:21/25`）→ codingplan-crypto。S3 会重写四层 feature 链 |
| **workspace 构建配置 / default-members** | **是**（仅 S3） | `Cargo.toml:7` `members = ["crates/*"]`、`:13-17` `default-members`、`:8-12` 关于"官方构建用 `--features rustcode/codingplan-crypto`"的注释。S1/S2 不触碰（N9） |
| **i18n 契约** | **是**（仅 S3） | `Cp*` 族（`messages.rs:320-395+`）与 `codingplan_crypto_tests` 双语守护是闭源发行构建的 i18n 契约（`AGENTS.md:338`） |

### 7.1 敏感面对策

1. **`codingplan_sync.json`（持久化）**
   - S1/S2：`sync_marker.rs` **零改动**（AC-9 锁定 `FILE_NAME` 字面量与 `read_last_sync` 的 `None` 语义）。
   - S3：必须同时删除 `sync_marker.rs`、`refresh_after_cross_process_codingplan_sync`（`mod.rs:12304`）、`LoopCtx.usage_slot` / `monitor_last_sync_seen` / `usage_last_check_at`（`mod.rs:3963/3977/3968`）与 `lib.rs:872` 的种子读取；并明确是否清理用户磁盘上的既有文件（**建议不清**，属用户数据，留待自然淘汰；若清则必须写进 release note）。
2. **daemon HTTP 公共协议**（仅 S3）：先在 `crates/rustcode-daemon/README.md` 与 `site/docs/{en,zh}/headless-daemon.html` 标记 deprecated，再下一版本删路由；禁止同版本内静默删除。AC-18 覆盖。
3. **OAuth token / 请求签名安全边界**（仅 S3）：删除后必须保证 `coding/src/provider_factory.rs` 的 provider 装配路径不出现"网关 URL 有配但 signer 缺失"的静默降级——要么整个分支消失（走纯 `bearer_auth`），要么保留显式 `SignError::Unavailable` 失败路径。**禁止**静默降级为无签名请求。
4. **workspace 构建配置与 feature 传递链**（仅 S3）：`codingplan-crypto` 是官方闭源 overlay 的唯一入口（`Cargo.toml:11-12`）；删除前必须取得用户对"放弃该入口"的书面确认（Q1= S3 即视为确认），否则不得删。
5. **default-members**：S1/S2 保持 `Cargo.toml:13-17` 原样。S3 若删 `rustcode-codingplan-crypto` crate，需同步更新 `:8-12` 的注释，但 `default-members` 三项不变。
6. **跨 crate 依赖方向**：G4 的重复渲染器抽取（tuix `commands.rs:4988-5060` 与 daemon `commands.rs:641-705` 逐字重复约 65 行 ×2）不得在 `rustcode-codingplan` 之外新建 driver 间依赖；放置位置需架构评审（Q8）。

---

## 8. 范围三档提案

> 三档互斥，用户择一。每档列出：涉及文件清单 / 预计删除·修改行数 / 破坏性 / 残留风险 / 回滚方式。
> **行数口径**：注释与文档为精确行号；`setup.rs` 3911、`api_codingplan.rs` 677 已逐行复核至末行；`client.rs` 602、C 类 153 为 grep 行数统计（**±1 行误差，未逐行复核**）。

### S1 保守档 — 只清 D 类过时痕迹 + 归档明确过时文档，不动生产逻辑

**涉及文件**

| 文件 | 动作 | 行 |
|---|---|---|
| `crates/rustcode-codingplan/src/lib.rs:1` | 改注释为 `crates/rustcode-codingplan/src/lib.rs` | 1 |
| `crates/rustcode-codingplan/src/types.rs:1` | 同上 | 1 |
| `crates/rustcode-codingplan/src/client.rs:1` | 同上 | 1 |
| `crates/rustcode-codingplan/src/setup.rs:1` | 同上 | 1 |
| `crates/rustcode-codingplan/src/sync_marker.rs:1` | 同上 | 1 |
| `crates/rustcode-config/src/i18n/messages.rs:319` | 改注释为 `// SetupReport renderer (rustcode-codingplan::setup)` | 1 |
| `crates/rustcode-config/src/i18n/messages.rs:376` | 改注释引用为 `rustcode_codingplan::setup::strikethrough` | 1 |
| `docs/config.example.toml:38-55` 或 `:149-167+` | 删其中一段（择一，见 Q6） | 净删 ~19 |
| `docs/release-v5.0.1-current-branch-change-report.md` | `git mv` 至归档目录 | 0（移动） |
| `docs/testing/release-v5.0.3-core-retirement-acceptance.md` | `git mv` 至归档目录 | 0（移动） |

**预计**：修改 7 行注释；净删 ~19 行配置示例；移动 2 份文档。生产逻辑 0 改动。

**破坏性**
- CLI `codingplan` 子命令：**无影响**（隐藏别名保留，`main.rs:1026`）。
- TUI `/codingplan` 斜杠命令：**无影响**（本就无此条目）。
- daemon HTTP 路由：**无影响**。
- feature 开关：**无影响**。
- i18n key：**无影响**（只动注释，不动变体与 arm）。

**残留风险**：低。唯一风险是 `docs/config.example.toml` 若删错段落导致示例信息缺失（AC-11 覆盖）。

**回滚方式**：`git checkout -- <4 个 .rs 文件>`（需先确认这些文件非 dirty；若 dirty 则人工 diff 还原）+ `git mv` 反向移动。

---

### S2 标准档 — S1 + 清无消费者死代码与重复实现 + 归档过时方案文档 + 标注 supersede

**涉及文件（在 S1 基础上追加）**

| 文件 / 目录 | 动作 | 说明 |
|---|---|---|
| `docs/coding-runtime-incremental-migration.md`（57KB） | `git mv` 归档 | 渐进迁移阶段记录，已完结 |
| `docs/coding-runtime-native-migration-design.md`（46KB） | `git mv` 归档 | 全量原生迁移阶段记录，`:3` 自述"已实施" |
| `docs/session-convergence-plan.md`（56KB） | `git mv` 归档 | `:3` 自述"S0～S5 已完成" |
| `docs/live-transport-convergence-plan.md`（11KB） | `git mv` 归档 | `:3` 自述"LT0～LT5 已实施" |
| `docs/v5.0.0-retire-bridge-core-progress.md`（11KB） | `git mv` 归档 | core/bridge 退役过程记录 |
| `docs/kernel-parity-backlog.md`（11KB） | `git mv` 归档 | kernel parity 收尾 backlog |
| `docs/compact-native-migration-retrospective.md`（21KB） | `git mv` 归档 | 被上面两份交叉引用（`:8-9` / `:13-14`），归档后需同步修链接 |
| `docs/platform-neutralization.md`（16KB） | **保留原位** | `AGENTS.md:539` 明示其 `atomgit_atomcode/atomcode` 字样是合规归属声明，不参与清理 |
| `docs/mcp-rmcp-feasibility.md`（13KB） | `git mv` 归档 | 可行性调研；含 codingplan 字样 |
| `docs/plans/2026-07-25-provider-retry-consolidation.md`、`docs/plans/2026-07-27-models-dev-pricing-design.md` | `git mv` 归档 | 已完成阶段计划 |
| `docs/plans/2026-07-26-provider-accounts-model-profiles-{design,plan}.md` | **保留原位** | design/plan 成对是仓库惯例，非重复 |
| `docs/REFACTOR_DESIGN_PHASE1.md` | **保留原位 + 头部加 `[SUPERSEDED BY] docs/phase1-refactor-design.md` 指针** | 被 `AGENTS.md:68/92/534/549` 规范性引用，不可删不可移 |
| `crates/rustcode-tuix/src/event_loop/commands.rs:4988-5060` 与 `crates/rustcode-daemon/src/commands.rs:641-705` | 评估抽取为共享渲染器（**需 Q8 架构评审**） | 逐字重复约 65 行 ×2；若评审不通过则维持现状，仅登记 |
| 死代码扫描 | 全仓扫 `Cp*` / `StatusCp*` / `codingplan*` 符号的零引用者 | **注意**：`Cp*` 族受 `AGENTS.md:338` 与 `:530` 保护，即便当前零引用也不得删（Q4） |

**预计**：在 S1 基础上，移动 ~10 份文档（0 内容删除），修改 `REFACTOR_DESIGN_PHASE1.md` 头部 ~3 行，同步修交叉链接 ~6 处；若 Q8 通过，抽取共享渲染器净删 ~65 行。**生产逻辑删除量 ≈ 0（除非 Q8 通过）**。

**破坏性**
- CLI / TUI / daemon / feature 开关 / i18n key：**全部无影响**（与 S1 相同）。
- 文档站：`site/` 搜索索引与 `docs/` 无直接关联（site 索引由 `site/build-search-index.mjs` 从 `site/` 内页面生成），`docs/` 归档**不影响** `site/search-index.{en,zh}.json`。
- **风险点**：`docs/compact-native-migration-retrospective.md` 被两份已归档文档交叉引用（`:8-9`、`:13-14`），移动后必须同步改相对链接，否则产生断链（AC-17 覆盖）。

**残留风险**：中低。主要风险是归档目录名与 site/AGENTS.md 引用冲突（AC-17 覆盖）以及 dirty 文件的移动中止（§5 失败场景）。

**回滚方式**：`git mv` 反向 + `git checkout --` 受影响文件（dirty 文件人工还原）。

---

### S3 激进档 — S2 + 移除 B 类与 C 类，使 fork 彻底平台中立

**涉及文件（在 S2 基础上追加）**

| 文件 | 动作 | 行数 |
|---|---|---|
| `crates/rustcode-codingplan/src/client.rs` | 删除 | ~602 |
| `crates/rustcode-codingplan/src/setup.rs` | 删除（含内联单测） | ~3911 |
| `crates/rustcode-codingplan/src/sync_marker.rs` | 删除（连带 A 类） | ~151 |
| `crates/rustcode-codingplan/Cargo.toml` | 删 `client` feature 与 `reqwest`（含 Windows native-tls 段） | ~14 |
| `crates/rustcode-daemon/src/api_codingplan.rs` | 删除 | ~677 |
| `crates/rustcode-codingplan-crypto/`（整 crate） | 删除 | ~29 + Cargo.toml |
| `crates/rustcode-auth/src/gateway_crypto.rs` | 删 `codingplan-crypto` 分支（`:57-93`）与 `canonical_chat_completions_path` 的调用方判断 | ~40 |
| `crates/rustcode-auth/Cargo.toml:21/25` | 删 optional dep 与 feature | 2 |
| `crates/rustcode-capabilities/src/provider/codingplan_sign.rs` | 删除 | ~153 |
| `crates/rustcode-capabilities/src/provider/mod.rs:19/28` | 删模块与重导出 | 2 |
| `crates/rustcode-coding/src/provider_factory.rs:5/52-60` | 删网关分支 | ~10 |
| `crates/rustcode-cli/src/main.rs` | 删 `Commands::Codingplan`（`:1022-1026`）、`:1696` 合并臂、`:3574` 兜底臂、`:4514-4625` 双实现、`Cargo.toml:29-38` feature | ~120 |
| `crates/rustcode-daemon/` | `lib.rs:29/54/6074/6431`、`runtime_host.rs:3-83`、`commands.rs:622-705`、`api_auth.rs:289`、`Cargo.toml:17-22` | ~120 |
| `crates/rustcode-tuix/` | `event_loop/commands.rs:31/4971-5066/5341/7018/7110-7290`、`monitor.rs`、`usage_monitor.rs`、`mod.rs:18328`、`Cargo.toml:14` | ~250 |
| `crates/rustcode-clix/src/main.rs:886` | 删签名网关报错分支 | ~5 |
| `crates/rustcode-config` | 删 `Cp*` / `StatusCp*` i18n 变体与 `codingplan_crypto_tests`（**与 `AGENTS.md:338` 冲突**）、`config/mod.rs:1192/1212/1221/1233`（**与 G7 冲突**） | ~100+ |
| 根 `Cargo.toml:8-12` | 删闭源 overlay 注释与构建入口 | ~5 |
| `crates/rustcode-daemon/README.md`、`site/docs/{en,zh}/headless-daemon.html` | 删/改 `/codingplan/*` 段落 | ~20 |

**预计删除**：约 **5300-5600 行**（含 setup.rs 内联单测），跨 **10 个 crate**。

**破坏性（逐项）**

| 面 | 影响 |
|---|---|
| CLI `codingplan` 子命令 | 隐藏别名 `rustcode codingplan`（`main.rs:1026`）消失。默认构建下它本就只打印中性提示（`:4517-4520`），**实际用户影响低**；补全脚本断言（`:4903`）本就不含 codingplan，不受影响。需用户确认（Q2） |
| TUI `/codingplan` 斜杠命令 | **无此命令**（§1.2 校正 3），不受影响。但 `/login`、`/usage` 的 desc 文案（`commands.rs:191/216`）与 `MANAGED_ONLY_COMMANDS` 门控（`AGENTS.md:285`）需重做 |
| daemon HTTP 路由 | `/codingplan/*` 全部消失 → **公共协议破坏性变更**，需 deprecate-then-remove 两阶段（AC-18） |
| feature 开关 | `codingplan` / `codingplan-crypto` 从 cli / daemon / tuix / auth 全部移除；**官方发行构建入口 `--features rustcode/codingplan-crypto` 失效**（`Cargo.toml:11-12`） |
| i18n key | `Cp*` 族与 `codingplan_crypto_tests` 双语守护消失（受 `AGENTS.md:338` 保护） |
| 安全边界 | `coding/src/provider_factory.rs:52-60` 的网关签名分支消失；已配 `RUSTCODE_CODINGPLAN_LLM_BASE_URL` 的用户将由"签名网关"变为普通 bearer 请求 → **静默行为变更，必须显式失败而非降级**（§7.1 第 3 条） |
| 持久化 | `codingplan_sync.json` 无生产者；连带删 `sync_marker.rs` 违反 N1（不动 A 类） |

**残留风险**：**高**，且存在**与仓库铁律的直接冲突**：
1. 违反 `AGENTS.md:338`（`Cp*` 族"按门控不删除铁律不动"）、`:530`（"codingplan cfg、Cp* 族…全部保留（gate 不删）"）、`:213-214`（G7 第 2 类 `is_codingplan_provider_name` / `LEGACY_CODINGPLAN_PREFIX`"勿删"）；
2. 拆除根 `Cargo.toml:8-12` 的闭源 overlay 接入点后，本 fork 无法再构建官方签名发行版；
3. 删除 A 类 `sync_marker.rs` 与 N1 冲突，需用户显式放宽；
4. `site/docs`、README、两 IDE 扩展（VS Code `types.ts` / JetBrains `RustCodeDaemonTypes.kt` 的托管字段）的"仅发行版本"口径全部需同步改写，工作量外溢至非 Rust 面。

**回滚方式**：**必须整体 revert**（跨 10 crate 的 feature 链与 i18n 契约互相咬合，单文件 revert 不可行）。建议以独立 commit 承载，且不在 release 分支合入。

---

## 9. 推荐档位与理由

**推荐：S2（标准档）。**

理由（按权重排序）：

1. **[OBJECTIVE-3] 平台中立已由"配置 + 门控"达成，而非由"删除"达成。** `AGENTS.md:91` 明示：默认不绑定任何平台，`is_codingplan_llm_gateway` 仅在显式配置 `RUSTCODE_CODINGPLAN_LLM_BASE_URL` 时为真，否则一律 `bearer_auth(api_key)`。B/C 类是 **opt-in 开关**，不是默认绑定；删掉它们不增加中立性，只减少可选能力。
2. **`client` feature 默认关闭，B 类在中立构建零链接、零可达。** `crates/rustcode-codingplan/Cargo.toml:9` `default = []`，`:15` `client = ["dep:reqwest"]`；所有 B 类消费者（`cli/src/main.rs:4544`、`tuix/.../commands.rs:7110`、`daemon/src/lib.rs:29/54`）都在 `#[cfg(feature="codingplan")]` 内。C 类的 `UnavailableSigner`（`gateway_crypto.rs:39-47`）是**显式 fail-closed**（`Err(SignError::Unavailable)`）而非静默降级。
3. **A 类仍被 tuix 用量面板真实消费**，S3 会连带伤及默认构建。`modals/usage.rs:6-7`、`lib.rs:872`、`mod.rs:12305` 三条消费者均无 cfg 门控（§1.2 A 类）。S3 要删 `sync_marker.rs` 就必须同时改 `LoopCtx` 与 `refresh_after_cross_process_codingplan_sync`，直接违反 N1。
4. **删除 B/C 类属"突破架构禁止方向"级别的删改，须用户授权而非 agent 自行决定。** 根 `Cargo.toml:8-12` 与 `AGENTS.md:50`、`:338`、`:530` 三处明示 codingplan-crypto 是官方闭源 overlay 的**刻意接入点**（"gate 不删"）。我的职责边界要求：涉及此类方向性删除，写入开放问题并升级。
5. **S1 太少，S3 太多。** S1 只清注释与一段重复示例，未满足用户"识别并清除所有不再适用或已废弃的部分"；S3 违反三条铁律并外溢到 site/扩展面。

**补充建议**：S3 中**零风险子集**可单独裁决并前置——例如 `Commands::Codingplan` 隐藏别名的存废（Q2）、`docs/config.example.toml` 删哪一段（Q6）。这两项与档位选择正交，可在用户裁决档位的同时一并决定。

**不推荐在 S2 内默认执行 Q8（跨 crate 重复渲染器抽取）**：需要确定共享代码的落点（`rustcode-codingplan` 是 leaf，但 tuix 与 daemon 都依赖它，方向合法；不过 `AGENTS.md:49` 要求依赖只向下，且 daemon 对 codingplan 是 optional dep，把渲染器放进去会改变 feature 语义）。**先过架构评审再动。**

---

## 10. 待用户裁决项清单（开放问题）

> 每条给出候选与推荐项。**Q1 是阻塞项**，其余按档位相关性决定是否阻塞。

| # | 问题 | 证据 | 候选 | 推荐 |
|---|---|---|---|---|
| **Q1**（阻塞） | **清理范围档位** | §8 | A) S1 保守<br>B) S2 标准<br>C) S3 激进 | **B（S2）**，理由见 §9 |
| **Q2** | CLI 隐藏别名 `Commands::Codingplan`（`crates/rustcode-cli/src/main.rs:1022-1026`、`:1696`、`:3574`）存废——这是"旧版本入口"的典型残留 | §1.2 校正 3；`AGENTS.md:278` 称其已移除，与代码不符 | A) 保留（默认构建打 `CliManagedLoginNotBuilt` 中性提示，零成本）<br>B) **删除**（真正落实"不保留旧版本"）<br>C) 保留但改中性名字 | **B**。它是"被折叠进 `/login` 后遗留的肌肉记忆别名"，正是用户诉求所指；且删除只需动 3 处 + 1 处 `unreachable!` 臂，风险极低。若选 A，请在交付报告中说明保留理由 |
| **Q3** | `crates/rustcode-config/src/config/mod.rs:1192` `LEGACY_CODINGPLAN_PREFIX = "AtomGit"`、`:1212` `prefixes_for`、`:1221` `codingplan_prefixes()`、`:1233` `is_codingplan_provider_name()` 是否删除——用户"不保留任何旧版本"与 G7 门禁冲突 | `AGENTS.md:213-214` G7 第(2)类明示"旧 `AtomGit-*` provider 前缀兼容（is_codingplan_provider_name / LEGACY_CODINGPLAN_PREFIX，旧配置键仍需识别，勿删）" | A) **保留**（遵守 G7）<br>B) 删除并同步改 `AGENTS.md:209-217` 的 G7 门禁与 `:530` 汇总 | **A**。删除会让"配置里已写 `AtomGit-*` 键的老用户"静默降级为普通自定义 provider（丢失 plan 信息与 `/login` 接管能力），属数据兼容破坏；且改门禁属 N7 禁区 |
| **Q4** | `Cp*` i18n 族（`crates/rustcode-config/src/i18n/messages.rs:320-395+`，约 30+ 变体）与 `codingplan_crypto_tests` 双语守护是否删除 | `AGENTS.md:338` 明示"按门控不删除铁律不动"，是闭源发行构建的 i18n 契约 | A) **保留**（遵守铁律）<br>B) 随 S3 一并删 | **A**（除非 Q1=C，此时 Q4 自动选 B，但仍需在交付报告记录"已突破 AGENTS.md:338"） |
| **Q5** | `docs/phase1-refactor-design.md` 与 `docs/REFACTOR_DESIGN_PHASE1.md` 的处置——前者 `:7` 自述 supersedes 后者，但后者被 AGENTS.md 规范性引用 | `phase1-refactor-design.md:7`；`AGENTS.md:68`（决策 D1 §2.0）、`:92`（§4.3 G6）、`:534`（决策 D1 + commit 6dbf57bb）、`:549`（文档索引） | A) **两份都留**：在旧文件头部加 `[SUPERSEDED BY docs/phase1-refactor-design.md]` 指针，正文不动<br>B) 把 D1/G6 决策段落迁进新文件、改 AGENTS.md 4 处引用后删旧文件 | **A**。B 需改 `AGENTS.md`，属 N7 禁区，且决策原文迁移有失真风险 |
| **Q6** | `docs/config.example.toml` 两段重复的 CodingPlan 网关示例删哪一段 | `:38-55`（英文段，`providers.RustCode-deepseek-v4-flash` / `RustCode-Qwen-Qwen3-VL-8B-Instruct`）<br>`:149-167+`（中文段，`providers."RustCode-GLM-5.2"` / `"RustCode-deepseek-v4-flash"` / `"RustCode-Qwen-Qwen3-VL-8B-Instruct"`） | A) **删英文段 `:38-55`**，保留中文段<br>B) 删中文段，保留英文段 | **A**。文件其余注释主体为中文（`:30-32`、`:57`、`:67-69`、`:140` 等），中文段与上下文风格一致且信息更全（含 `max_tokens`） |
| **Q7** | 归档目录路径 | — | A) `docs/archive/`<br>B) `docs/_archive/`<br>C) `docs/history/` | **A**。需确认不撞 `site/build-search-index.mjs`（该脚本只索引 `site/` 内页面，预期不受影响，但实施前须 AC-17 验证） |
| **Q8** | 是否允许把 tuix（`event_loop/commands.rs:4988-5060`）与 daemon（`commands.rs:641-705`）逐字重复的 CodingPlan status 渲染器（~65 行 ×2）抽取为共享实现 | §1.2、§7.1 第 6 条 | A) 维持现状，仅登记（本 feature 不做）<br>B) 抽取，落点待架构评审 | **A**。收益有限（只省 ~65 行），但会改变 `rustcode-daemon` 对 `rustcode-codingplan` 的 optional dep 语义、触碰 feature 链。**先做架构评审再决定**，不在本需求内强行推进 |
| **Q9** | worktree dirty：S1/S2 的文档 `git mv` 是否会撞上用户未提交的改动文件 | 编排者称 `docs/REFACTOR_DESIGN_PHASE1.md` 为 dirty（2026-09-02 19:40）；**本轮无 shell 权限，未执行 `git status`，证据不足** | A) **对 dirty 文件一律不移动**，改为原位加 `[SUPERSEDED BY]` 指针<br>B) 允许 `git mv`（会连带移动用户未提交改动） | **A**。N10 硬约束：禁止 `git checkout/reset/stash`，也不得在用户未提交改动上做文件级移动 |
| **Q10** | `docs/platform-neutralization.md` 是否纳入清理 | `AGENTS.md:539` 明示其 `atomgit_atomcode/atomcode` 是 MIT 归属声明，"删改有合规风险" | A) **保留原位，不移动不改**<br>B) 归档 | **A**。合规面零风险优先 |

---

## 附：本轮未复核 / 证据不足项

1. **`git` 历史与 worktree 状态**：本轮无 shell 权限，未执行 `git log --oneline -10 -- <path>` 与 `git status`。`AGENTS.md:185` 要求的"查看相关文件近期 Git 历史，确认任务未已实现或改变方向"**未完成**，需实施前由 implementer 补做（尤其是 `docs/REFACTOR_DESIGN_PHASE1.md` 的 dirty 状态，直接影响 Q9）。
2. **精确行数**：`client.rs`（602）、`codingplan_sign.rs`（153）为 grep 行数统计，未逐行复核；`setup.rs`（3911）、`api_codingplan.rs`（677）已复核至末行。
3. **`docs/compact-native-migration-retrospective.md` 等 5 份"阶段性报告"未被 `codingplan` grep 命中**：它们是编排者按主题（而非字面）归入本 feature 的，本轮按 §8 S2 表格处理；若用户认为超出 CodingPlan 范围，可从清单中剔除。
4. **测试基线数字**：`rustcode-codingplan` 27/0、`rustcode-tuix --lib` 2063/0、`rustcode-config --lib` 327/0 均引自 `AGENTS.md:368` 的历史记录，本轮**未复跑**（硬约束：只读分析）。
