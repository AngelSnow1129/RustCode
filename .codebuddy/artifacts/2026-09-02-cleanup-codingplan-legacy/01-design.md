---
kind: design
id: DESIGN-001
from: solution-architect
to: [project-manager]
feature: 2026-09-02-cleanup-codingplan-legacy
status: approved
decision: proceed
requires: [REQ-001]
files_owned: []
architecture_constraints:
  touches_runtime_lifecycle: false
  touches_persistence: false
  touches_cross_crate_deps: false
created: 2026-09-02
---

# 01 · 设计说明书（S2 标准档 · CodingPlan 遗留痕迹清理）

> **只读设计产物。** 本轮仅写本文件与 `02-tasks.md`，未改动任何生产代码 / 文档 / `Cargo.toml` / `AGENTS.md`。
> **基线**：branch=`dev` commit=`8e772dbf`，worktree **dirty**。所有判定均带 `文件:行` 证据；无 shell 权限、未能复核处一律写「证据不足」。
> **档位**：Q1=S2 标准档（用户已裁决）。B/C 类保留、CLI 隐藏别名删除、Q3/Q4 铁律保护面保留、Q8 重复渲染器仅登记。

---

## 1. 目标与非目标

### 1.1 目标

| # | 目标 | 覆盖 AC |
|---|---|---|
| G1 | 清除 `crates/` 下全部指向已退役 `rustcode-core/src/coding_plan/*` 的失效路径注释（D1–D4，共 7 处） | AC-10 |
| G2 | 删除 `docs/config.example.toml` 重复的英文网关示例段，保留中文段，文件仍是合法可拷贝 TOML | AC-11 |
| G3 | 归档已完结的过时方案文档；对被 supersede 者加指针；归档后零断链 | AC-17 |
| G4 | 删除 CLI 隐藏别名 `Commands::Codingplan`，消除与 `AGENTS.md:278` 的事实冲突 | AC-16 |
| G5 | 扫描并**登记**死代码/跨 crate 重复实现（只出清单，不改代码） | —（交付物） |
| G6 | 全程不回归默认构建、不回归 A 类消费者、不回归 `codingplan_sync.json` 读写语义 | AC-1/2/6/7/8/9/13/14 |

### 1.2 非目标（沿用 `00-requirement.md` §2.2 的 N1–N10，本档位逐条重申）

- **N1 不动 A 类对外行为与文件布局**：`types.rs` / `usage.rs` / `sync_marker.rs` 只改第 1 行注释，**不迁、不重命名、不改 JSON schema**。
- **N2** 不重启 [OBJECTIVE-1..6] 的既有结论。
- **N3** 不触碰 `rustcode-kernel` / `rustcode-capabilities` / `rustcode-coding` 的 core-free 生产依赖（`AGENTS.md:138`）——**本轮这三个 crate 零改动**。
- **N4** 不恢复 bridge / v1-v2 开关 / core session 磁盘模型 / 任何 runtime fallback（`AGENTS.md:143`、`:168`）。
- **N5** 不新增第二运行时生命周期所有者。
- **N6 不自作主张删铁律保护面**（`AGENTS.md:530`"gate 不删"）：`atomgit` feature、`#[cfg(feature="codingplan")]` 块、`LEGACY_CODINGPLAN_PREFIX` / `is_codingplan_provider_name`（`AGENTS.md:213-214` G7）、`Cp*` i18n 族与 `codingplan_crypto_tests`（`AGENTS.md:338`）、托管 QR 屏（`AGENTS.md:354`）——**全部保留，即便零引用也不得删**。
- **N7 不改 `AGENTS.md` / `README.md` / `README.zh-CN.md` / `docs/architecture.md` / `.codebuddy/rules/**` 的规范性引用**。`AGENTS.md:278` 的表述同步降级为 **optional T8，需用户确认**，不得作为 must-have。
- **N8** 不做 i18n 文案改写（不改任何 `Msg` arm 的 en/zh 正文），**仅允许改注释**。
- **N9** 不动 `Cargo.toml` 的 `default-members`。
- **N10** 禁止 `git checkout` / `git reset` / `git stash` 掉用户改动；dirty 文件一律原位处理、人工 diff 还原。

---

## 2. 现状

### 2.1 A/B/C/D 四类事实分层（复核确认）

| 类 | 内容 | 门控 | 本轮动作 |
|---|---|---|---|
| **A** | `rustcode-codingplan/src/{types.rs,usage.rs,sync_marker.rs}` | 无，默认编译 | **只改 3 个文件第 1 行注释** |
| **B** | `client.rs`(602)、`setup.rs`(3911)、`daemon/src/api_codingplan.rs`(677) | `#[cfg(feature="client")]` / `codingplan`（`Cargo.toml:9` `default=[]`、`:15`；`lib.rs:21-24`） | **只改 2 个文件第 1 行注释** |
| **C** | `rustcode-codingplan-crypto`、`auth/gateway_crypto.rs`、`capabilities/provider/codingplan_sign.rs` | 默认编译（非死代码） | **零改动**（Q1=S2） |
| **D** | 失效 core 路径注释 7 处 | — | **全部清零** |

### 2.2 A 类真实消费者（默认编译、无 cfg，逐条已复核）

- `crates/rustcode-tuix/src/lib.rs:872` — `monitor_last_sync_seen: rustcode_codingplan::read_last_sync()`
- `crates/rustcode-tuix/src/event_loop/mod.rs:12304-12314` — `refresh_after_cross_process_codingplan_sync()` 调 `read_last_sync()`
- `crates/rustcode-tuix/src/modals/usage.rs:6-7` — `use rustcode_codingplan::types::{PlanInfo, RateLimitWindow};` / `use rustcode_codingplan::usage::{...};`
- `crates/rustcode-tuix/src/event_loop/usage_monitor.rs:21` — `use rustcode_codingplan::types::UsageInfo;`

> 上述 4 处**只读**，本轮零改动；AC-7 用 grep 钉死。

### 2.3 D 类过时痕迹清单（本轮清零，AC-10）

| # | 文件:行 | 当前内容 |
|---|---|---|
| D1 | `crates/rustcode-codingplan/src/lib.rs:1` | `// crates/rustcode-core/src/coding_plan/mod.rs` |
| D2 | `.../types.rs:1` | `// crates/rustcode-core/src/coding_plan/types.rs` |
| D3 | `.../client.rs:1` | `// crates/rustcode-core/src/coding_plan/client.rs` |
| D4 | `.../setup.rs:1` | `// crates/rustcode-core/src/coding_plan/setup.rs` |
| D5 | `.../sync_marker.rs:1` | `// crates/rustcode-core/src/coding_plan/sync_marker.rs` |
| D6 | `crates/rustcode-config/src/i18n/messages.rs:319` | `// SetupReport renderer (core/coding_plan/setup.rs)` |
| D7 | `crates/rustcode-config/src/i18n/messages.rs:376` | 注释引用 `coding_plan::setup::strikethrough` |

**本轮 grep 实测基线**（`rg -n "rustcode-core/src/coding_plan\|core/coding_plan/setup\.rs\|rustcode-core/src" crates/`，14 命中）：

- 属于本 feature 的 7 命中 = 上表 D1–D7。
- **不属于本 feature 的 7 命中（不动，仅登记）**：`crates/rustcode-tuix/src/custom_commands.rs:1`、`crates/rustcode-config/src/config/instructions.rs:1`、`crates/rustcode-tuix/src/...`（见下表）、`crates/rustcode-review/src/impact_plan.rs:370-372`（测试夹具内 diff 文本）、`crates/rustcode-kernel/tests/fallible_stream.rs:154`、`crates/rustcode-kernel/src/agent.rs:169`、`crates/rustcode-coding/src/persona.rs:2`、`crates/rustcode-capabilities/src/tools/bash.rs:1241`、`crates/rustcode-capabilities/src/skills/render.rs:4`。
- **结论**：AC-10 的两条判据 `rg -n "rustcode-core/src/coding_plan" crates/` 与 `rg -n "core/coding_plan/setup.rs" crates/` 当前各为 **5 命中 / 2 命中**，清零后须为 **0 / 0**。这两条判据**不受**上列非本 feature 命中影响（它们不含 `coding_plan` 子串），故 AC-10 可达。

### 2.4 CLI 隐藏别名现状（`crates/rustcode-cli/src/main.rs`）

`rg -n "Codingplan|codingplan" crates/rustcode-cli/src/main.rs` 共 29 命中，其中与别名直接相关者：

| 行 | 内容 | 处置 |
|---|---|---|
| `:1022-1025` | 变体 doc comment（`Hidden alias for rustcode login ...`） | 删 |
| `:1025` | `#[command(hide = true)]` | 删 |
| `:1026` | `Codingplan,` | 删 |
| `:1696` | `Commands::Login \| Commands::Codingplan =>` | 改回 `Commands::Login =>` |
| `:3574-3578` | `Commands::Codingplan => { unreachable!(...) }` 整臂 | 删 |
| `:1669-1670` | 注释"（and its hidden alias `Codingplan`）" | 同步修（注释，与 D 类同性质） |
| `:3519` | 注释"（and its Codingplan alias）" | 同步修 |

**不在处置范围（保留）**：`:1387`（`spawn_blocking` 线程用途注释）、`:1704-1705` / `:1715-1718`（`run_codingplan_core` 流程，属 B 类门控面）、`:2897` / `:4515-4625`（`#[cfg(feature="codingplan")]` 双实现）、`:4903` / `:4966`（既有断言，不动）。

**关键事实**：`:1696` 是**无 cfg 门控**的合并臂；默认构建下落至 `:4517-4520` 的中性 stub（`Msg::CliManagedLoginNotBuilt`）。补全断言 `:4903` `!script.contains("codingplan")` 因 `hide = true` **当前已通过**，删除别名后仍通过 → AC-16 无回归风险。

### 2.5 `docs/config.example.toml` 现状

`rg -n "codingplan|CodingPlan" docs/config.example.toml` 共 4 命中：

| 行 | 段 |
|---|---|
| `:38` / `:44` / `:51` | **英文段**（`Example: CodingPlan gateway (recommended)`，`providers.RustCode-deepseek-v4-flash` / `RustCode-Qwen-Qwen3-VL-8B-Instruct`，`:38-55`） |
| `:149` | **中文段**（`CodingPlan 网关（OAuth 自动配置）`，`providers."RustCode-GLM-5.2"` 等，`:149-169`，含 `max_tokens`，信息更全，与文件主体中文注释风格一致） |

两段**全为注释行**（每行以 `#` 开头），删除 `:38-56`（含 `:55` 分隔线与 `:56` 空行）后文件仍为合法 TOML；中文段 `:149-169` 保留 → `rg -n "CodingPlan"` 恰 1 命中，AC-11 可达。

### 2.6 文档归档面（AC-17 硬约束的实测结果 —— **本设计最重要的发现**）

对 00 §8 S2 表 12 份候选归档文档，本轮执行了
`rg -n "<被移动文件名>" AGENTS.md README.md README.zh-CN.md docs/ .codebuddy/`（实际以仓库根全量 grep 执行），结果：

**（a）无任何 `AGENTS.md` / `README.md` / `README.zh-CN.md` / `.codebuddy/` 命中** —— 12 份文档均未被规范性文件引用，可安全归档。

**（b）`docs/` 内部存在 2 处「来自非移动文档的引用」→ 按 AC-17「只要命中就不得移动」硬规则，这两份不得移动**：

| 被引用文档 | 引用方（**不在**归档清单内） | 判定 |
|---|---|---|
| `docs/mcp-rmcp-feasibility.md` | `docs/mcp.md:179` `[mcp-rmcp-feasibility.md](./mcp-rmcp-feasibility.md)` | **禁止 `git mv`** → 原位加指针 |
| `docs/compact-native-migration-retrospective.md` | `docs/compact-durable-checkpoint-design.md:9` | **禁止 `git mv`** → 原位加指针 |

**（c）其余 10 份的全部引用方都在移动集内**，同批移动后相对链接仍可解析（详见 §5.2 链接修正表）。

**（d）10 份移动文档对「留在原位」的文档存在 6 处外链，移动后必须加 `../` 前缀**：

| 移动后文件 | 行 | 目标（留在 `docs/`） | 修正为 |
|---|---|---|---|
| `docs/archive/coding-runtime-incremental-migration.md` | `:9` | `compact-native-migration-retrospective.md` | `../compact-native-migration-retrospective.md` |
| 同上 | `:69` | `target-architecture.md` | `../target-architecture.md` |
| 同上 | `:71` | `compact-native-migration-retrospective.md` | `../compact-native-migration-retrospective.md` |
| `docs/archive/coding-runtime-native-migration-design.md` | `:13` | `compact-native-migration-retrospective.md` | `../compact-native-migration-retrospective.md` |
| 同上 | `:15` | `target-architecture.md` | `../target-architecture.md` |
| `docs/archive/v5.0.0-retire-bridge-core-progress.md` | `:119` | `compact-native-migration-retrospective.md` | `../compact-native-migration-retrospective.md` |

**无需修正者（目标同在 `docs/archive/`）**：`coding-runtime-incremental-migration.md:70` → `v5.0.0-retire-bridge-core-progress.md`；`coding-runtime-native-migration-design.md:14` → `coding-runtime-incremental-migration.md`、`:61` → `session-convergence-plan.md`、`:62` → `live-transport-convergence-plan.md`；`kernel-parity-backlog.md:4` → `coding-runtime-native-migration-design.md`；`session-convergence-plan.md:9`、`:799` → `live-transport-convergence-plan.md`。
**无外链者（4 份）**：`release-v5.0.1-current-branch-change-report.md`、`testing/release-v5.0.3-core-retirement-acceptance.md`、`plans/2026-07-25-provider-retry-consolidation.md`、`plans/2026-07-27-models-dev-pricing-design.md`。

**（e）保留原位不动者（3 份）**：`docs/platform-neutralization.md`（`AGENTS.md:539` 合规归属，Q10=A）、`docs/REFACTOR_DESIGN_PHASE1.md`（dirty + `AGENTS.md:68/92/534/549` 规范性引用，Q5=A、Q9=A）、`docs/plans/2026-07-26-provider-accounts-model-profiles-{design,plan}.md`（仓库 design/plan 成对惯例，非重复）。

---

## 3. 候选方案与取舍

| 方案 | 描述 | 结论 |
|---|---|---|
| **A（采纳）** | **零契约变更的痕迹清理**：只改 7 行注释、删 1 个 CLI 枚举变体、删 1 段 TOML 注释、归档 10 份文档 + 2 份原位加指针、出 1 份扫描清单。全部改动可被编译 + grep 验证。 | **采纳**。不改任何 trait/enum 变体（除删 1 个）/错误类型/持久化/路由/feature，G2 门禁可过。 |
| **B（放弃）** | 抽取 tuix `event_loop/commands.rs:4988-5060` 与 daemon `commands.rs:641-705` 的 ~65 行重复渲染器 + 删除零引用死代码 | **放弃**。① Q8=A（用户裁决：维持现状、仅登记）；② 抽取会改变 `rustcode-daemon` 对 `rustcode-codingplan` 的 **optional dep 语义**、触碰 feature 传递链（`AGENTS.md:49` 依赖只向下）；③ `Cp*` 族即便零引用也受 `AGENTS.md:338`/`:530` 保护不得删 → 收益 ≈ 0、风险 > 0。00 §9 亦不推荐。 |
| **C（放弃）** | 归档整个交叉引用簇，并同步修正 `docs/mcp.md:179`、`docs/compact-durable-checkpoint-design.md:9` 两处引用后一并移动 | **放弃**。AC-17 明文「只要命中就不得移动」；且会扩大 `files_owned` 到两份**非归档清单内**的文档，超出本 feature 授权。改为方案 A 的「Group B 原位加指针」。 |
| **D（放弃）** | 删 CLI 别名的同时同步 `AGENTS.md:278` 表述 | **放弃为 must-have**。`AGENTS.md` 是 dirty 且属维护规则文件、N7 禁区 → 降级为 **optional T8（需用户确认）**。补充事实：删除别名后 `AGENTS.md:278` 的"该别名已移除"表述反而**由失真变为准确**，故不改不产生新矛盾（`site/docs/{en,zh}/headless-daemon.html` 已按"CLI 没有单独的 `rustcode codingplan` 命令"书写）。 |

---

## 4. 目标架构（变更边界图）

### 4.1 边界图

| 区域 | 文件 | 本轮动作 | 所有者 |
|---|---|---|---|
| **改（代码）** | `crates/rustcode-codingplan/src/{lib,types,client,setup,sync_marker}.rs` | 仅第 1 行注释 | T1 |
| **改（代码）** | `crates/rustcode-config/src/i18n/messages.rs` | 仅 `:319`、`:376` 两行注释 | T1 |
| **改（代码）** | `crates/rustcode-cli/src/main.rs` | 删 `Commands::Codingplan`（`:1022-1026`）+ `:1696` + `:3574-3578` + `:1669-1670`/`:3519` 注释 | T3 |
| **改（文档）** | `docs/config.example.toml` | 删 `:38-56` | T4 |
| **移动（文档）** | 10 份 → `docs/archive/`（扁平化，不建子目录） | `git mv` + 6 处链接加 `../` | T5 |
| **改（文档，原位）** | `docs/mcp-rmcp-feasibility.md`、`docs/compact-native-migration-retrospective.md` | 头部加指针（AC-17 阻断，不移动） | T6 |
| **改（文档，dirty）** | `docs/REFACTOR_DESIGN_PHASE1.md` | **仅头部追加** `[SUPERSEDED BY docs/phase1-refactor-design.md]` 指针，正文一字不动 | T6 |
| **只读** | `crates/rustcode-tuix/**`（A 类消费者）、`crates/rustcode-daemon/**`、`crates/rustcode-auth/**`、`crates/rustcode-capabilities/**`、`crates/rustcode-coding/**`、`crates/rustcode-kernel/**`、`crates/rustcode-codingplan-crypto/**` | 零改动 | — |
| **明确不动** | 全部 `Cargo.toml`、`AGENTS.md`（除非 T8 获批）、`README*.md`、`docs/architecture.md`、`docs/platform-neutralization.md`、`.codebuddy/rules/**`、`site/**` | 零改动 | — |

### 4.2 数据流 / 控制流

- **数据流零变更**：`sync_marker.rs` 的 `read_last_sync()` / `write_last_sync_now()` 调用图与 JSON 形状完全不变（§6 冻结 F5）。
- **控制流唯一变更点**：`rustcode codingplan` 不再进入 `main.rs:1696` 的 login 合并臂。变更前：`Commands::Codingplan` 与 `Commands::Login` 等价（默认构建下打印 `CliManagedLoginNotBuilt` 后落穿 TUI）；变更后：clap 将其视为**未知子命令 → 报错 exit 2**。这是**显式失败**，不是静默降级（用户 Q2=B 已授权）。`rustcode login` 路径完全不变。
- **无新增模块、无新增 hook、无新增第二压缩状态机**（本 feature 不涉及 turn completion / compaction）。

---

## 5. 接口契约（冻结清单）

> **总原则：本 feature 不新增、不修改任何公共 trait / enum 变体 / 错误类型 / 事件 / 持久化格式 / HTTP 路由 / feature 名。唯一对外契约变化是 `Commands` 枚举收缩 1 个变体（已由 Q2=B 授权）。**

| # | 冻结项 | 冻结理由 |
|---|---|---|
| **F1** | 不新增/不修改任何 `pub trait` | N3/N4；kernel、capabilities、coding 零触碰 |
| **F2** | 不新增/不修改任何 `pub enum` 变体；`Msg::Cp*` 全族（`messages.rs:320-395+`）与 `mod codingplan_crypto_tests` **零改动**（仅 `:319`、`:376` 两行**注释**可变） | `AGENTS.md:338`（闭源发行构建 i18n 契约 + 门控不删除铁律）、`:530` |
| **F3** | 不新增/不修改任何错误类型（`SignError`、`AuthExpired` 等） | C 类保留，安全边界零改动 |
| **F4** | 不新增/不修改任何事件枚举、ACP/daemon 命令枚举 | 本 feature 不触碰协议面 |
| **F5** | **持久化冻结**：`$RUSTCODE_HOME/codingplan_sync.json` 文件名常量与三态语义零改动 | AC-9；N1 |
| **F6** | **HTTP 路由冻结**：`/codingplan/*` 路由族（`api_codingplan.rs`）零改动 | S2 不动 B 类；AC-18 不适用 |
| **F7** | **feature 链冻结**：`client` / `codingplan` / `codingplan-crypto` 三个 feature 名与全部 `Cargo.toml` 声明零改动；`default-members` 不动 | Q1=S2；N9；保官方闭源 overlay 接入点（`Cargo.toml:11-12`） |
| **F8** | **i18n 冻结**：不改任何 `Msg` arm 的 en/zh 正文 | N8 |
| **F9** | **G7 门禁冻结**：`config/src/config/mod.rs` 4 项零改动（仅文件内**无**本轮改动） | `AGENTS.md:213-214` G7 第(2)类"旧配置键仍需识别，勿删"；Q3=A |
| **F10** | **唯一可变面**：D1–D7 共 7 行注释；`docs/config.example.toml:38-56` 注释段；`docs/` 文档移动与指针；`crates/rustcode-cli/src/main.rs` 内 `Commands::Codingplan` 相关的 5 处（3 删 + 2 注释） | 用户裁决 Q2/Q5/Q6/Q7 授权 |
| **F11** | 禁止恢复任何 legacy writer、core 磁盘投影、双向转换 | `AGENTS.md:143`、`:168`；N4 |

### 5.1 冻结签名（逐字，不得擅改）

```rust
// crates/rustcode-codingplan/src/sync_marker.rs —— 冻结，零改动
const FILE_NAME: &str = "codingplan_sync.json";          // :22
struct SyncMarker { last_sync_unix_secs: u64 }           // :24-31（私有，非 pub）
fn marker_path() -> PathBuf                              // :33-35  = Config::config_dir().join(FILE_NAME)
pub fn read_last_sync() -> Option<SystemTime>            // :40-45  三态 None 语义
pub fn write_last_sync_now() -> std::io::Result<()>      // :52-67
```

```rust
// crates/rustcode-config/src/config/mod.rs —— 冻结，零改动（Q3=A，AGENTS.md:213-214）
const LEGACY_CODINGPLAN_PREFIX: &str = "AtomGit";                       // :1192
fn name_matches_prefix(name: &str, prefix: &str) -> bool                // :1198
fn prefixes_for(configured: &str) -> Vec<String>                        // :1212
fn codingplan_prefixes() -> &'static [String]                           // :1221
pub fn is_codingplan_provider_name(name: &str) -> bool                  // :1233
pub fn codingplan_group_account_id(provider_type: &str) -> &'static str // :1245
```

```rust
// crates/rustcode-cli/src/main.rs —— 本 feature 唯一的对外契约变更（只减不增）
// 删除 :1022-1026：
//     /// Hidden alias for `rustcode login` -- kept so existing scripts /
//     /// muscle memory don't break after `/codingplan` and `rustcode
//     /// codingplan` were folded into the unified `/login` flow.
//     #[command(hide = true)]
//     Codingplan,
// 修改 :1696：Commands::Login | Commands::Codingplan =>   ==>   Commands::Login =>
// 删除 :3574-3578：
//     Commands::Codingplan => {
//         // Hidden alias for Login -- `run()` intercepts both before
//         // handle_command is called, so this arm is unreachable.
//         unreachable!("Codingplan is handled inline in run() before handle_command")
//     }
// 同步修注释：:1669-1670、:3519
// 契约后果：enum Commands 不再含 Codingplan；`rustcode codingplan` 由「等价于 login」变为
//           clap 未知子命令 → 报错 exit 2（显式失败，非静默降级）。须写入交付报告/release note。
```

```rust
// docs/config.example.toml —— 删除 :38-56（含 :55 分隔线、:56 空行），保留 :149-169
// 判据：rg -n "CodingPlan" docs/config.example.toml  →  恰 1 命中（:149 位移后）
```

### 5.2 文档归档契约（AC-17）

- **Group A（可 `git mv`，10 份）** → `docs/archive/`（**扁平化**：`docs/testing/release-v5.0.3-core-retirement-acceptance.md` → `docs/archive/release-v5.0.3-core-retirement-acceptance.md`；`docs/plans/2026-07-2x-*.md` → `docs/archive/2026-07-2x-*.md`）。
  1. `coding-runtime-incremental-migration.md`
  2. `coding-runtime-native-migration-design.md`
  3. `session-convergence-plan.md`
  4. `live-transport-convergence-plan.md`
  5. `v5.0.0-retire-bridge-core-progress.md`
  6. `kernel-parity-backlog.md`
  7. `release-v5.0.1-current-branch-change-report.md`
  8. `release-v5.0.3-core-retirement-acceptance.md`（自 `docs/testing/`）
  9. `2026-07-25-provider-retry-consolidation.md`（自 `docs/plans/`）
  10. `2026-07-27-models-dev-pricing-design.md`（自 `docs/plans/`）
- **Group B（AC-17 阻断，禁止 `git mv`，原位加指针，2 份）**：`docs/mcp-rmcp-feasibility.md`、`docs/compact-native-migration-retrospective.md`。
- **链接修正 6 处**：见 §2.6(d) 表，统一改为 `../<filename>`。
- **扁平化理由**：`docs/archive/testing/`、`docs/archive/plans/` 只有 1–2 个文件，建子目录会让 Group A 之间的同目录相对链接（`:70`、`:14`、`:61`、`:62`、`:4`、`:9`、`:799`）**全部断裂**并需要 7 处额外修正；扁平化后这些链接天然成立，净修正量从 13 处降到 6 处。
- **前置硬门禁**：每份文档 `git mv` 前必须先跑 `git status --porcelain <file>`；**非空即中止**（dirty）→ 改走 Group B 流程。

---

## 6. 状态所有权

| 状态 | 唯一所有者 | 只读者 | 创建/销毁 | 单一所有者保证 |
|---|---|---|---|---|
| `$RUSTCODE_HOME/codingplan_sync.json` | `crates/rustcode-codingplan/src/sync_marker.rs`（`FILE_NAME:22` 定义、`marker_path():33-35` 解析、`write_last_sync_now():52` 唯一写入者） | `read_last_sync():40`（同文件）；跨 crate 只读者：`rustcode-tuix/src/lib.rs:872`、`event_loop/mod.rs:12304-12314` | 进程内无缓存（`OnceLock` 未用于该文件）；磁盘文件由 `write_last_sync_now()` 创建，由卸载清单 `crates/rustcode-cli/src/uninstall/paths.rs:36/150` 与 `scripts/uninstall.{sh:21,ps1:24}` 列出（只登记路径字符串，非读写者） | 全仓 `rg -n "codingplan_sync.json"` 7 命中中，仅 `sync_marker.rs:22` 是定义、`:3970`(tuix) 是注释、其余 4 处是卸载清单字符串 → **无第二写入者、无第二解析者** |
| `is_codingplan_provider_name()` 前缀判定 | `crates/rustcode-config/src/config/mod.rs:1233`（`:1231` 自述 "Single source of truth"） | `rustcode-codingplan`、`rustcode-tuix` 等委派方 | 无状态（纯函数 + `OnceLock` 进程级缓存 `:1222`） | 本轮**零改动** → 不引入第二真源 |
| `LoopCtx.usage_slot` / `monitor_last_sync_seen` | `crates/rustcode-tuix/src/event_loop/mod.rs`（`:3963`、`:872`） | — | 随 TUI session | 本轮零改动 |
| `docs/archive/` 目录 | 无（纯静态文档目录） | — | 由 T5 首次创建 | 不引入代码状态 |

**结论**：本 feature 的全部动作（删注释、删 CLI 变体、删 TOML 注释段、归档文档）**均不新增任何状态持有者**，也不改变既有所有者。

---

## 7. 失败与取消语义

> 本 feature 无异步 pending 请求、无运行时/网络改动，故 cancel/reload/shutdown 语义不适用。**所有失败一律 fail-closed：任一 AC 红 → 任务置 `changes_requested` + 写明 `decision`，禁止"先合后修"、禁止静默 fallback、禁止假成功。**

| 失败场景 | 错误/信号 | 恢复动作 | 责任任务 | 禁止动作 |
|---|---|---|---|---|
| **归档目标文件 dirty**（`git status --porcelain` 非空） | 非空输出 | **中止该文件的 `git mv`**；改为原位加 `[ARCHIVED-IN-PLACE]` / `[SUPERSEDED BY]` 指针；登记进交付报告；任务不置 `done`（置 `blocked`，`decision` 写明） | T5/T6 | 禁止 `git checkout` / `restore` / `reset` / `stash`（N10，硬约束） |
| **归档产生断链**（AC-17 grep 命中来自非移动文档的引用） | `rg` 命中 | **回退该文件的 `git mv`**（`git mv docs/archive/X docs/X`），改走 Group B 原位加指针流程 | T5 | 禁止"先移动、链接以后再说" |
| **链接修正遗漏**（移动后 `../` 未加） | AC-17 终验 `rg` 命中失效相对路径 | 逐条补 `../`；重跑 AC-17 | T5 | 禁止放宽 AC-17 判据 |
| **删除段落后 `docs/config.example.toml` 非法 TOML / 结构破损** | TOML 解析失败或 `rg -n "CodingPlan"` ≠ 1 | 回退删除（该文件非 dirty，`git checkout -- docs/config.example.toml`）；只删**完整注释块**，不删结构性行 | T4 | 禁止留下半截注释块 |
| **删 CLI 别名后编译失败 / 测试红**（AC-1/2/14/16） | 编译 error 或测试失败 | 该任务置 `changes_requested`；人工 diff 排查遗漏的 match 臂 | T3 | 禁止为编译通过而保留别名或加 `#[allow]` 兜底 |
| **注释修正后 AC-10 仍非 0** | `rg` 命中 | 补改遗漏处；重跑 AC-10 两条判据 | T1 | 禁止改用更宽松的正则 |
| **`cargo fmt --check` 新增差异** | 差异集扩大 | 对本次改动行执行 `cargo fmt`；重跑对比 | T1/T3 | 注意 `crates/rustcode-cli/src/main.rs` 在 `AGENTS.md:542` 的 **19 处存量违规**内；判据是「**不新增**」，不是「清零」 |
| **CLI 别名删除后 `AGENTS.md:278` 表述失准** | — | **optional T8**：由 PM 征询用户；未获批则**在交付报告中登记为已知不一致**，不动 `AGENTS.md` | T8（blocked） | 禁止未经确认改动 `AGENTS.md`（dirty + N7） |
| **磁盘 100% 满致链接器失败**（`AGENTS.md:343/350/368/386/394`） | `No space left on device` | 清 `target/debug/incremental`（可弃缓存）后重试 | 全局 | 禁止删 `target/debug/deps` 以外的数据；禁止 sudo（`AGENTS.md:18`） |
| **rustc SIGBUS** | cgroup 8GB 上限 | `cargo test --workspace` **必须 `-j 1`**（`AGENTS.md:541`） | 全局 | 禁止提高并发绕过 |
| **唯一允许的既有失败** | `mcp::registry::tests::trust_key_golden_matches_core_algorithm` | 保持红；**不得改测试凑绿**（`AGENTS.md:226`、`:350`） | 全局 | 禁止修改该测试或其依赖的 `DefaultHasher` |

---

## 8. 迁移与回退

### 8.1 数据格式兼容

- **唯一持久化面 `codingplan_sync.json` 零改动**（F5）→ 不存在格式迁移、不存在双向转换、不存在 legacy writer 恢复需求。
- **文档归档不是数据迁移**：`docs/` 不参与构建、不参与运行时读取、不影响 `site/search-index.{en,zh}.json`（`site/build-search-index.mjs` 只索引 `site/` 内页面）。
- **CLI 契约变更**是唯一的对外不兼容点（`rustcode codingplan` → exit 2），已由 Q2=B 授权，须写入交付报告；**不提供兼容别名**（提供别名 = 保留旧版本入口，与用户诉求冲突）。

### 8.2 回退步骤（按任务，见 `02-tasks.md` 逐条展开）

1. **通用前置**：任何 `git checkout` 回滚前，先 `git diff -- <files> > <artifacts>/rollback-backup.patch` 留档。
2. **代码面（T1/T3）**：`crates/rustcode-codingplan/src/*`、`crates/rustcode-config/src/i18n/messages.rs`、`crates/rustcode-cli/src/main.rs` 经编排者核验为**非 dirty** → 允许 `git checkout --`；**但实施者必须先跑 `git status --porcelain` 复核**（编排者核验与实施时刻之间可能有新改动）。
   - ⚠️ `main.rs` 含 19 处**存量** fmt 违规（`AGENTS.md:542`），整体 `git checkout` 会一并抹掉这些非本轮引入的差异 → **必须人工按 hunk 还原，不得整体 checkout**。
3. **dirty 文件（T6 的 `docs/REFACTOR_DESIGN_PHASE1.md`、T8 的 `AGENTS.md`）**：**一律人工 diff 还原，禁止 `git checkout` / `reset` / `stash`**（N10）。
4. **文档移动（T5）**：`git mv docs/archive/X docs/X` 反向移动 + 对 6 处链接修正执行 `git checkout --`（前提：复核非 dirty）。
5. **TOML（T4）**：`git checkout -- docs/config.example.toml`（复核非 dirty）。
6. **整体回退**：本 feature 各任务 `files_owned` 两两不相交、无跨任务契约咬合 → **支持逐任务独立回退**，不需要整体 revert（与 S3 档不同）。

---

## 9. 架构边界核对（逐条对照 `AGENTS.md`）

| # | `AGENTS.md` 约束 | 本设计 | 结论 |
|---|---|---|---|
| 1 | `:49` 依赖只向下，不得 `capabilities → driver` / `kernel → 上层` | 本轮只在 `rustcode-codingplan`/`rustcode-config`/`rustcode-cli` 三个 crate 内改注释与删 1 个枚举变体；**不新增任何 `use`、不新增任何 Cargo 依赖** | ✅ 合规 |
| 2 | `:138` kernel / capabilities / coding 生产依赖 core-free | 这三个 crate **零改动**（N3） | ✅ 合规 |
| 3 | `:143`、`:168` 禁止恢复 bridge / v1-v2 开关 / core session 磁盘模型 / fallback | 不恢复任何兼容层；删 CLI 别名是**显式失败**（clap exit 2），非 fallback | ✅ 合规 |
| 4 | `:52` `CodingRuntime` 为唯一运行时生命周期所有者 | 本轮不新增生命周期所有者；删别名不影响 `run()` 的 TUI 落穿（`main.rs:1696` 后仍 fall through 到 TUI 启动） | ✅ 合规（`touches_runtime_lifecycle: false`） |
| 5 | native `SessionManager/SessionMeta/SessionSnapshot` 为唯一 session 持久化模型 | 未触碰；唯一持久化面 `codingplan_sync.json` 零改动 | ✅ 合规（`touches_persistence: false`） |
| 6 | `:209-217` G7 门禁（`atomcode` 0 命中；`atomgit` 仅三类） | 保留 `LEGACY_CODINGPLAN_PREFIX` 等 4 项 → `rg -rni "atomcode" crates/ scripts/ .github/` 仍 0 命中；本轮注释修正**不引入** `atomcode` 字样 | ✅ 合规（AC-12） |
| 7 | `:338`、`:530` 门控不删铁律（`Cp*` 族、codingplan cfg、atomgit feature、managed-QR） | **全部保留**；只改 `messages.rs` 的 2 行**注释**，不改任何变体与 arm | ✅ 合规（AC-13） |
| 8 | `:539` `docs/platform-neutralization.md` 的 `atomgit_atomcode/atomcode` 属 MIT 合规归属 | 原位不动、不移动、不改（Q10=A） | ✅ 合规 |
| 9 | `:68`、`:534` 产品身份锁定 `rustcode`，不得再改名 | 不改名 | ✅ 合规 |
| 10 | `:285` / `:305` 命令发现面与中立构建断言 | 删别名后补全脚本仍不含 `codingplan`（`:4903` 断言本就通过）；login about 仍不含 `CodingPlan`（`:4966`）| ✅ 合规（AC-16） |
| 11 | `:541` `cargo test --workspace` 须 `-j 1` | 验证命令统一加 `-j 1` | ✅ 合规 |
| 12 | `:542` 19 处存量 fmt 违规 | 判据为「不新增」；`main.rs` 在存量清单内，要求按 hunk 处理 | ✅ 合规（AC-15） |
| 13 | `:18` 禁止 sudo（`~/.rustcode` root 属主会致后续启动失败） | 任务不涉 `~/.rustcode`；磁盘清理只删 `target/debug/incremental` | ✅ 合规 |
| 14 | N7 不改 `AGENTS.md` / README / `docs/architecture.md` 规范性引用 | `AGENTS.md:278` 同步降级为 **optional T8，需用户确认**；README/architecture 零改动 | ✅ 合规 |
| 15 | 不得新增第二压缩状态机 / 重叠 `LifecycleHooks::turn_complete` | 本 feature 不涉及 turn completion / compaction，未新增任何 hook | ✅ 合规 |

**架构边界总结论：全部 15 条合规，无 `blocked` 项。** 本设计 `touches_runtime_lifecycle / touches_persistence / touches_cross_crate_deps` 三项均为 `false`。

---

## 10. 验证策略

> **包名为 `rustcode`，不是 `rustcode-cli`**（`crates/rustcode-cli/Cargo.toml:2`），`-p rustcode-cli` 会失败。
> **`cargo test --workspace` 必须 `-j 1`**（`AGENTS.md:541`）。
> **唯一允许的既有失败**：`mcp::registry::tests::trust_key_golden_matches_core_algorithm`（`AGENTS.md:226`、`:350`、`:540`）。

| AC | 验证命令 | S2 档 | 主责任务 |
|---|---|---|---|
| AC-1 | `cargo build` | ✅ 必过 | T1/T3 |
| AC-2 | `cargo check --workspace --all-targets` | ✅ 必过 | T1/T3 |
| AC-3 | `cargo check -p rustcode-codingplan --features client --all-targets` | ✅ 必过（**关键**：`setup.rs`/`client.rs` 第 1 行被改，默认 feature 下不编译，必须显式验） | T1 |
| AC-4 | `cargo check -p rustcode-tuix --features codingplan --all-targets`<br>`cargo check -p rustcode-daemon --features codingplan --all-targets`<br>`cargo check -p rustcode --features codingplan --all-targets` | ✅ 必过（保 feature 传递链） | T1/T3 |
| AC-5 | `cargo check -p rustcode --features codingplan-crypto --all-targets`<br>`cargo check -p rustcode-daemon --features codingplan-crypto --all-targets` | ✅ 必过（保闭源 overlay 接入点） | T1/T3 |
| AC-6 | `cargo test -p rustcode-codingplan --lib` | ✅ 必过（基线 27/0，`AGENTS.md:368`） | T1 |
| AC-7 | `rg -n "read_last_sync\(\)" crates/rustcode-tuix/src/lib.rs crates/rustcode-tuix/src/event_loop/mod.rs`<br>`rg -n "^use rustcode_codingplan" crates/rustcode-tuix/src/modals/usage.rs` | ✅ 必过（A 类消费者仍在） | T1 |
| AC-8 | `cargo test -p rustcode-tuix --lib usage` 与 `cargo test -p rustcode-tuix --lib` | ✅ 必过 | T1 |
| AC-9 | `cargo test -p rustcode-codingplan --lib sync_marker`<br>`rg -n "codingplan_sync.json" crates/`（须仍为 1 处定义 + 注释） | ✅ 必过 | T1 |
| **AC-10** | `rg -n "rustcode-core/src/coding_plan" crates/` → **0 命中**<br>`rg -n "core/coding_plan/setup.rs" crates/` → **0 命中** | ✅ **本档核心判据** | T1 |
| AC-11 | `rg -n "CodingPlan" docs/config.example.toml` → **恰 1 命中** | ✅ 必过 | T4 |
| AC-12 | `rg -n "LEGACY_CODINGPLAN_PREFIX\|is_codingplan_provider_name" crates/` → 仍命中<br>`rg -rni "atomcode" crates/ scripts/ .github/` → **0 命中** | ✅ 必过 | T1 |
| AC-13 | `cargo test -p rustcode-config --lib`（基线 327/0）<br>`rg -n "mod codingplan_crypto_tests" crates/rustcode-config/src/i18n/` | ✅ 必过 | T1 |
| AC-14 | `cargo test -j 1 --workspace --no-fail-fast` | ✅ 必过（失败集须与基线一致，唯一允许 `trust_key_golden_matches_core_algorithm`） | T9 |
| AC-15 | `cargo fmt --check` | ✅ **不新增**差异（19 处存量违规非本轮引入，`AGENTS.md:542`） | T1/T3 |
| AC-16 | `cargo test -p rustcode shell_completion`<br>`cargo test -p rustcode --lib neutral_build_hides_managed_login_subcommands` | ✅ **本档核心判据** | T3 |
| AC-17 | 对每份被移动文档：`rg -n "<被移动文件名>" AGENTS.md README.md README.zh-CN.md docs/ .codebuddy/`，逐条确认无失效相对链接 | ✅ 必过 | T5/T6/T9 |
| AC-18 | `rg -n "codingplan" crates/rustcode-daemon/README.md site/docs/en/headless-daemon.html site/docs/zh/headless-daemon.html` | ⛔ **S2 不适用**（S3 only） | — |

**验证顺序**：单任务自验（G3）→ 批次内合并后跑该批次的 AC 子集 → 全部批次完成后由 T9 跑全量（AC-1…AC-17）。

---

## 11. 风险与开放问题

### 11.1 风险

| # | 风险 | 等级 | 缓解 |
|---|---|---|---|
| **R1** | **AC-17 硬规则导致 12 份归档清单中 2 份无法移动**（`mcp-rmcp-feasibility.md` ← `docs/mcp.md:179`；`compact-native-migration-retrospective.md` ← `docs/compact-durable-checkpoint-design.md:9`）。用户裁决的 S2 清单是 12 份，本设计实归档 **10 份 + 2 份原位加指针**。 | **中** | 已按 AC-17「命中即不移动」严格执行；需编排者确认接受，或授权把 `docs/mcp.md`、`docs/compact-durable-checkpoint-design.md` 纳入 `files_owned`（那时可 12 份全归档，但需多改 2 处链接） |
| **R2** | `docs/REFACTOR_DESIGN_PHASE1.md` 是 **dirty**，只允许"仅追加指针"。追加位置在 `:1` 标题行之后、`:3` 首个 `> [INFO]` 之前，不得触碰 `:1-8` 既有内容 | **中** | T6 明确写"仅插入 1 行，禁止改写既有行"；回滚方式 = 人工 diff 还原，禁止 git checkout |
| **R3** | `crates/rustcode-cli/src/main.rs` 含 19 处**存量** fmt 违规（`AGENTS.md:542`），整体 `git checkout` 回滚会一并抹掉非本轮差异 | **中** | 回滚前 `git diff > backup.patch`；按 hunk 还原 |
| **R4** | `Commands::Codingplan` 删除是**对外 CLI 契约的 breaking change**（`rustcode codingplan` → exit 2），可能影响既有脚本 | **中** | 已由用户 Q2=B 授权；必须写入交付报告/release note。**不提供兼容别名**（与用户"不保留旧版本入口"诉求冲突） |
| **R5** | 编排者核验的 dirty 面与实施时刻可能不一致（`crates/rustcode-codingplan/src/*`、`messages.rs`、`main.rs` 被判为非 dirty） | **中** | 每个任务的第 0 步强制 `git status --porcelain <files_owned>`；非空即中止并上报，不得 checkout |
| **R6** | Q8 重复渲染器（tuix `commands.rs:4988-5060` ≈ daemon `commands.rs:641-705`，~65 行逐字重复）维持现状 | **低** | 仅登记为已知重复与未来评审项；抽取会改变 daemon 对 codingplan 的 optional dep 语义 |
| **R7** | 磁盘 100% 满（历史多次，`AGENTS.md:343` 等） | **低** | 清 `target/debug/incremental`；禁止 sudo、禁止删 `target/debug/deps` 以外数据 |
| **R8** | 唯一允许红 `trust_key_golden_matches_core_algorithm`（`DefaultHasher` 跨工具链不稳定） | **低** | 铁律禁改；AC-14 失败集对比基线时须排除它 |

### 11.2 开放问题

| # | 问题 | 建议 | 需谁裁决 |
|---|---|---|---|
| **O1** | Group B 2 份文档：接受「原位加指针」，还是授权改 `docs/mcp.md:179` / `docs/compact-durable-checkpoint-design.md:9` 后归档？ | **默认接受原位加指针**（严格遵守 AC-17） | 编排者 / 用户 |
| **O2** | optional T8：是否在删别名后同步 `AGENTS.md:278` 的表述？ | **默认不执行**（AGENTS.md 是 dirty + N7 禁区）；且删别名后 `:278` 表述由失真变为准确，不改不产生新矛盾 | 用户 |
| **O3** | `docs/archive/` 为首次创建的新目录，仓库无既有先例 | 采用 `docs/archive/`（Q7=A）；扁平化不建子目录（§5.2） | 已裁决 |
| **O4** | `crates/` 下另有 7 处**非本 feature** 的 `rustcode-core/src/*` 失效路径注释（`custom_commands.rs:1`、`instructions.rs:1`、`kernel/src/agent.rs:169`、`kernel/tests/fallible_stream.rs:154`、`coding/src/persona.rs:2`、`capabilities/src/tools/bash.rs:1241`、`capabilities/src/skills/render.rs:4`，另 `review/src/impact_plan.rs:370-372` 为测试夹具内 diff 文本） | **不在本 feature 范围**，仅在交付报告登记为 follow-up | 编排者 |

---

## 12. 修订记录

| 日期 | 版本 | 变更 |
|---|---|---|
| 2026-09-02 | v1 | 首版。基于 REQ-001 与用户裁决 Q1=S2 / Q2=B / Q3=A / Q4=A / Q5=A / Q6=A / Q7=A / Q8=A / Q9=A / Q10=A 冻结契约。 |
