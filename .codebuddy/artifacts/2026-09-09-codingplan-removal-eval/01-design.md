---
kind: design
id: DESIGN-001
from: solution-architect
to: [project-manager]
feature: 2026-09-09-codingplan-removal-eval
status: approved
decision: proceed
requires: [REQ-001]
files_owned: []
architecture_constraints:
  touches_runtime_lifecycle: true
  touches_persistence: true
  touches_cross_crate_deps: true
created: 2026-09-09
---

# DESIGN-001 彻底移除 codingplan —— 技术方案（方案 B）

> **前置说明（如实标注）**
> 1. `00-requirement.md` 的 `status` 仍为 `blocked` / `decision: block`（Q1 未由需求方自行选档）。本件依据 `STATUS.md` 记录的**用户裁决**（G1 pass：Q1=B、Q2'=删除识别层、Q5=清理前端）推进；该裁决与 `00-requirement.md` 的 header 状态不一致，**由编排者负责在 G1 关闭前把 `00-requirement.md` 的 header 改为 approved 或补一条决策记录**，本设计不再重复裁决。
> 2. 本会话**无 `Bash` 工具**：`git rev-parse` / `git log` / `cargo` 全部未由我执行。所有 `file:line` 证据来自 `Grep` / `Read` 实读；所有命令型验证在 `02-tasks.md` 中下放给实现者，基线留档（AC-0）为批次 1 的强制首步。
> 3. 本件未修改任何生产代码 / `Cargo.toml` / `docs/**` / `AGENTS.md`。

---

## 1. 目标 / 非目标（继承并细化 `00-requirement.md` §2）

### 1.1 目标

| id | 目标 |
|---|---|
| **G-1** | 把 `rustcode-codingplan` 与 `rustcode-codingplan-crypto` 两个 crate、其 REST 路由、签名层、config 识别层、i18n 族、扩展/前端客户端**整体删除**，使 `crates/` 下 `codingplan` 字样归零（仅剩本设计 §5.4 定义的 4 行白名单）。 |
| **G-2** | 全过程保持**中间态可编译**：每个批次结束时 `cargo check --workspace --all-targets` exit 0；每批次一个 commit，可反向 `git revert`。 |
| **G-3** | 持久化语义变更（账号折叠 / 只读保护 / 内置 effort 回退）**显式化**：不迁移、不改写用户磁盘，变更面逐条写入交付说明。 |
| **G-4** | 公共协议（三条 `/codingplan/*` 路由）退役与扩展侧清理在**同一次交付内**完成，且中间态不比现状更糟（默认构建现状即 404）。 |
| **G-5** | 同步修订被本次裁决推翻的既有结论：`AGENTS.md:220-221/289/329/345/498/509/537`、`docs/` 12 份非归档文件、`2026-09-02-cleanup-codingplan-legacy/STATUS.md:67-76`。 |

### 1.2 非目标

| id | 非目标 |
|---|---|
| **N-1** | **不**引入任何 config 迁移器 / 重命名器 / 双向转换 / legacy writer（`AGENTS.md:173-175` 硬约束）。`[providers.*]` 键与值一律不动。 |
| **N-2** | **不**改动 `CodingRuntime` 生命周期与事件协议（`coding/src/runtime.rs` 与 codingplan 零符号耦合，仅 `:7894-7897` 注释）。 |
| **N-3** | **不**改动 native session 持久化模型、`LlmProvider` trait、`CodingProviderFactory` trait 形状（:75-81）。 |
| **N-4** | **不**删除 `rustcode_auth::managed_login_available()` 驱动的托管登录门控面：`/login` / `/logout` / `/whoami` / `managed_available`（daemon `/auth/status`、webui、两个扩展的谓词门控）——它们属于 **OAuth 托管登录**，不是 codingplan crate，**本次保留**。 |
| **N-5** | **不**删除 `capabilities::provider::sign::{RequestSigner, RequestSigningError, SignedAuth}` 与 `OpenAiCompatConfig.request_signer` 接缝（中性命名、非 codingplan 产物；删它会连带删掉 401 恢复与重试的既有测试，属范围蔓延）。 |
| **N-6** | **不**删除 `coding::rate_limit::{RateLimitHook, RateLimitWindowSource, RateLimitWindow}`（`kernel` 的 `retry_max_attempts` 边界语义仍由它承载，见 §4.4）。 |
| **N-7** | **不**清理 `docs/archive/*`（22 处，按既往用户裁决「归档即历史」）与 `docs/UPSTREAM_CREDITS.md:1`（MIT 合规归属）。`extensions/jetbrains/CHANGELOG.md:7` 为历史记录，保留。 |
| **N-8** | **不**删除卸载清理项：`cli/src/uninstall/paths.rs:36`、`:150`、`scripts/uninstall.sh:21`、`scripts/uninstall.ps1:24`（编排者已定夺，理由见 `STATUS.md`）。 |
| **N-9** | **不**顺手修复 `tuix/src/modals/provider_panel.rs:1121` 的 `reqwest::blocking::Client::builder()` 直连（与 codingplan 无关的**存量** egress 绕行），仅登记进 follow-up（见 §8.3）。 |
| **N-10** | **不**在产品内进行改名（产品身份 `rustcode` 锁定，`AGENTS.md:541`）。 |

---

## 2. 现状（四层模型 + 实测证据）

### 2.1 四层

| 层 | 内容 | 门控 | 默认构建链接 |
|---|---|---|---|
| **L1 识别/兼容层** | `config::{endpoints, config::mod}` 的前缀识别、账号折叠、managed 判定、内置 effort 回退 | 无 | 是 |
| **L2 签名层** | `auth::gateway_crypto`、`capabilities::provider::codingplan_sign`、`coding::CodingPlanProviderAuthenticator` | 无 | 是（恒返回「无签名器」） |
| **L3 网络侧** | `codingplan/src/{client,setup}.rs`、`daemon/api_codingplan.rs`、三条路由 | `#[cfg(feature="codingplan")]` | 否 |
| **L4 闭源桩** | `crates/rustcode-codingplan-crypto/` | `codingplan-crypto` feature | 否 |

### 2.2 生产方 / 消费方（实读）

**L1**（`rustcode-config`）
- 生产：`endpoints.rs:35/36/54`（三个 env 名）、`:75/79/116`（三个 `HOSTED_CODINGPLAN_*`）、`:185/192/204/265/278`、`config/mod.rs:1192/1198/1212/1221/1233/1245/1275/1286`。
- 消费（**非门控**）：`config/mod.rs:800-806`（折叠账号）、`:820-838`、`:842-853`、`:864-883`（折叠模型 + effort 回退）、`:1036-1040`；`tuix/src/modals/provider_panel.rs:438/807/916/930/939/1306/1384/1570`；`tuix/src/event_loop/monitor.rs:67`；`tuix/src/lib.rs:23-29`（测试 ctor 设置 `RUSTCODE_CODINGPLAN_LLM_BASE_URL`）。
- 测试：`config/mod.rs:1366-1401`（`codingplan_prefix_tests`，7 个）、`:4236-4237`、`endpoints.rs:386/387/395-396`。

**L2**
- 生产：`auth/gateway_crypto.rs` 全文（171 行）；`auth/lib.rs:17`；`auth/Cargo.toml:21/25`；`capabilities/src/provider/codingplan_sign.rs` 全文（153 行）；`capabilities/src/provider/mod.rs:19/28`；`coding/src/provider_factory.rs:5-7/38-43/45-64/66-73/86/97/214-216`；`coding/src/lib.rs:84-87`。
- 消费（**非门控**）：`daemon/runtime_host.rs:56/87-89`、`daemon/main.rs:151`、`daemon/api_config.rs:85/155`、`daemon/api_provider.rs:38-56`、`tuix/src/event_loop/mod.rs:10725/28529-28530`、`tuix/src/modals/provider_panel.rs:936`、`clix/src/main.rs:886`（+ `:1855-1857` 测试）。

**L3**
- 生产：`codingplan/src/{lib,client,setup,types,usage,sync_marker}.rs`；`daemon/src/api_codingplan.rs`（677 行）；`daemon/src/lib.rs:29-30/54/6096-6116/6318/6463-6476`。
- 门控消费：`cli/src/main.rs:2898/4510-4625`（含 neutral stub）、`cli/src/main.rs:1670-1718`（login→codingplan 链）、`tuix/src/event_loop/commands.rs:4971/4987/5063/5341/5374/5976/5990/6036/6053/6652/6673/6876/6935/7016/7108/7149/7311`、`tuix/src/event_loop/mod.rs:18347`、`tuix/src/event_loop/monitor.rs:140-204`、`tuix/src/event_loop/usage_monitor.rs:46-69`、`daemon/src/runtime_host.rs:48-85`、`daemon/src/commands.rs:622-705`。
- **非门控消费（B 的主要工作量，已逐点实读）**：
  - `tuix/Cargo.toml:27`（`rustcode-codingplan` **非可选**依赖）、`:14`（`codingplan = ["rustcode-codingplan/client"]`）
  - `tuix/src/lib.rs:872`（`read_last_sync()`）、`:929-935`（启动漂移检查）、`:23-29`（env）
  - `tuix/src/event_loop/mod.rs:3964-3966`（`usage_slot` 字段类型含 `rustcode_codingplan::types::UsageInfo`）、`:3943-3979`（`monitor_warning` / `monitor_last_check_at` / `monitor_last_sync_seen`）、`:9723-9741`、`:12319-12345`、`:23001-23013`、`:26851-26858`、`:28568-28581`
  - `tuix/src/event_loop/usage_monitor.rs:21`（模块顶 `use`）、`:73-74`（`#[cfg(not)]` 桩也用该类型）
  - `tuix/src/modals/usage.rs:6-7`（`types::{PlanInfo, RateLimitWindow}`、`usage::{compute_overview, humanize_tokens, OverviewStats, UsageResponse}`）、`:932`、`:1329`；`tuix/src/state.rs:1209`、`:2807`
  - `cli/src/uninstall/paths.rs:36/150`（`codingplan_sync.json` 清理项，**保留**）

**L4**：`crates/rustcode-codingplan-crypto/`（29 行，`:28` `unreachable!()`，`Cargo.toml:5` `license = "Proprietary"`）；根 `Cargo.toml:3-12` 注释；`Cargo.lock:2859`。

**i18n**（`rustcode-config`）：`messages.rs:6-7/14/23/27/319-453/4971-4975/503-523/1733/2280/2301`；`en.rs` / `zh_cn.rs` 各约 39/40 处命中，含末尾 `codingplan_crypto_tests` 两模块。

**扩展 / 前端**：VS Code（`src/daemon/types.ts:226`、`client.ts:13/350/374-375`、`chat/provider.ts:11/679-680/2036/2117-2146/2657`、`webview-ui/src/state/types.ts:327`、`ChatProvider.tsx:274`、`components/WelcomeScreen.tsx:45-46/136`、`i18n.tsx` 多键、`l10n/bundle.l10n.zh-cn.json:45`）；JetBrains（`RustCodeDaemonTypes.kt:118`、`RustCodeDaemonClient.kt:201-203`、`RustCodeProjectService.kt:546-556`、`RustCodeChatPanel.kt:552/1890-1894`、`ui/GearMenuLabels.kt:14/33`、`ui/GearMenuLabelsTest.kt:19/43`、`resources/messages/RustCodeBundle{,_zh}.properties:8`）；webui（`src/i18n.ts:224/287/653/715`、`api.ts:419 requires_login`、`components/SettingsDialogs.tsx:44-45/310/340`、`components/Chat.tsx:2384-2409`）。

**文档**：`docs/` 18 份（`archive/` 6 份 22 处不动）、`README.md` 6、`crates/rustcode-daemon/README.md` 6、`crates/rustcode-coding/README.md` 2、`crates/rustcode-clix/README.md` 1、`extensions/jetbrains/README.md:1`、`AGENTS.md` 68。

---

## 3. 整体策略（D-1：删除顺序与「中间态可编译」）

### 3.1 候选方案与取舍

| 候选 | 描述 | 取舍 |
|---|---|---|
| **S-A 纯自顶向下**（先删所有消费者，最后删 crate） | 每步默认构建可编译 | **放弃**：`tuix` 的消费者与 `config` L1 / `capabilities` L2 交织（`provider_panel.rs` 同时含 L1 与 L2 调用点），「先消费者」无法定义稳定的批次边界；且最后一批删 crate 时会一次性暴露全部遗漏，`cargo check` 失败定位成本最高。 |
| **S-B 单批一次性提交**（一次 commit 删完 6000–7500 行） | 无中间态 | **放弃**：① 违反 `AGENTS.md:190`「退役任务逐步可验证」；② 8GB cgroup 下全量测试耗时长，一次失败需全量重试；③ 回滚粒度=整个 feature，无法只回退扩展面或文档面；④ git bisect 无法区分是哪一层引入回归。 |
| **S-C 垂直切片 + 依赖边最后摘除（选定）** | 按「能力面」切 9 批；每批内部先删消费点、批内最后一个任务才摘依赖边/删生产者；每批一个 commit | **选定**。理由：① 每批结束 `cargo check --workspace --all-targets` 可绿，中间态可提交、可回滚、可 bisect；② 「依赖边最后摘除」消除了 `AGENTS.md:58` 警告的 cfg 传递链断裂（删 crate 与删 `tuix` 非可选依赖同提交）；③ 跨 crate 面（daemon+cli / auth+capabilities / drivers）按因果串行，同批内文件不相交的部分并行。 |

### 3.2 批次间的编译校验点

每个批次结束时必须全绿（实现者执行，写入 `03-impl/<task>.md`）：

| 校验点 | 命令 | 批次 |
|---|---|---|
| **V-0** | `cargo check --workspace --all-targets`（默认 feature） | **每一批**（B1–B9） |
| **V-1** | `cargo fmt --check` | 每一批 |
| **V-2** | `cargo test -j 1 -p <本批受影响 crate> --lib` | 每一批（受影响 crate） |
| **V-3** | `cargo check -p rustcode-tuix --features codingplan --all-targets` | B2 结束前必须仍 exit 0（tuix 门控块未删；**B3 起该 feature 已不存在**） |
| **V-4** | `cargo check -p rustcode-daemon --features codingplan --all-targets` / `-p rustcode --features codingplan` | B2 结束起必须报「does not have the feature」 |
| **V-5** | AC-4 七条 feature 矩阵全量 | **B9**（终态）+ B4 结束各跑一次 |
| **V-6** | `cargo test -j 1 --workspace --no-fail-fast` | B5 结束、B6 结束、B9（三次） |
| **V-7** | VS Code `npx tsc --noEmit`（host+webview）、`npm run test:webview`；webui `npm run typecheck` + `npm test` | B7 |
| **V-8** | `python3 scripts/check-zh-docs.py gate` | B8 |

### 3.3 必须原子合并的组合（同一次 commit / 同一次 squash）

| # | 原子组合 | 理由 |
|---|---|---|
| **A-1** | 删除 `crates/rustcode-codingplan/` 全目录 **+** `crates/rustcode-tuix/Cargo.toml:27` 去依赖 **+** `Cargo.lock` | `tuix` 是**非可选**依赖方，先删 crate 会导致 `tuix` 编译失败（`AGENTS.md:58` 同族风险）。归属任务 T-13，且为批次 3 的最后一个合并。 |
| **A-2** | `daemon/Cargo.toml:17-25` 与 `cli/Cargo.toml:22-42` 的 `codingplan` / `codingplan-crypto` feature + optional dep 删除 **+** 本 crate 内所有 `#[cfg(feature="codingplan")]` 块删除 | 只删其一会在 `--features codingplan` 下引用已删模块（`api_codingplan.rs`）→ 断裂；默认构建看不见，必须靠 AC-4 兜底。归属 T-11 / T-12。 |
| **A-3** | `auth::gateway_crypto` 模块删除 **+** `capabilities::provider::codingplan_sign.rs` 删除 | 后者 `use rustcode_auth::gateway_crypto::{self, SignInput}`（`codingplan_sign.rs:3`）。归属同一任务 T-16。 |
| **A-4** | `capabilities` 的 `is_codingplan_gateway` / `signer_available` 重导出删除（`provider/mod.rs:28`）**+** 全部 drivers 消费点改造（`daemon/runtime_host.rs`、`daemon/main.rs`、`daemon/api_config.rs`、`daemon/api_provider.rs`、`tuix/event_loop/mod.rs`、`tuix/modals/provider_panel.rs:936`、`clix/main.rs`） | 同批次、串行合并（T-16 → T-17 → T-18）。 |
| **A-5** | `config` L1 符号删除 **+** `tuix/src/modals/provider_panel.rs` 消费点改造 | 同批次、串行合并（T-19 → T-20）。 |
| **A-6** | i18n 三件套 `messages.rs` + `en.rs` + `zh_cn.rs` | `Msg` 的 match 是穷尽的（AC-15），漏一处即编译失败；三件套在任意批次只能由**一个**任务独占。 |

### 3.4 是否单批一次性提交：**否**

**结论：按批次提交，每批 1 个 commit（跨 crate 串行批次内部按合并顺序 squash）**。理由：① bisect 粒度=批次足以定位回归层；② 回滚可只回退扩展面/文档面而不动 Rust 主体；③ 每批都有可执行的编译与测试校验点；④ 磁盘 8GB/40G 满盘历史故障（`AGENTS.md:350/374/499`）要求验证可分次重试。**回滚顺序必须严格反向**（B9→B1），因为后批次依赖前批次的符号删除。

---

## 4. 逐项设计决策

### 4.1 D-1（已答，见 §3）

### 4.2 D-2 持久化语义变更

#### 4.2.1 既有 `config.toml` 条目变成什么

`[providers.RustCode]` / `[providers.RustCode-anthropic]` / `[providers."AtomGit-GLM-5.2"]` 本来就是合法的 `[providers.*]` 条目：

| 行为 | 现状（`config/mod.rs:797-887`） | 删除 L1 后 |
|---|---|---|
| 加载 / 保存 | 正常 | **不变**（磁盘格式零改动） |
| `logical_accounts()` | 命中等前缀的条目按 `codingplan_group_account_id(&provider_type)` 折叠成 1 个合成账号（`:800-806`） | **每个条目一个账号，key 为 provider 原名**（`:807-809` 分支成为唯一路径） |
| `logical_models()` | 模型 key 仍是 provider 名，但 `account` 指向合成账号（`:864-868`） | **account 恒为 provider 原名** |
| 只读保护 | `account_is_codingplan_managed()` 命中前缀 → 面板/daemon 拒绝改名改 base_url | **消失**：所有 provider 一律可编辑（由 `daemon/api_provider.rs` 的保留名规则 `selection_name_is_reserved`（`:58-60`）继续兜住「新 schema 名不可占用」这条，与前缀无关） |
| `reasoning_effort_levels` | `effective_reasoning_effort_levels(codingplan_managed, model, declared)`：声明为空且 managed 时回落到 `codingplan_builtin_effort_levels`（`deepseek-v4-flash` → `["high","max"]`，`:1275-1279`） | **声明值即最终值**；声明为空 ⇒ `None` ⇒ `allowed_effort_levels(None)` 返回全量 5 级（`:1325-1343`，空表=不限制的既有语义不变） |

**真实变化面（必须写进交付说明）**：默认构建下 `is_codingplan_llm_gateway()` 恒 false（三个 `HOSTED_*` 为空串，`endpoints.rs:75/79`），所以「base_url 命中网关」这条判定从来不会真；**唯一真实生效的是 provider 名前缀判定**（默认前缀 `HOSTED_CODINGPLAN_PROVIDER_PREFIX = "RustCode"`，`endpoints.rs:116`，外加历史前缀 `LEGACY_CODINGPLAN_PREFIX = "AtomGit"`，`config/mod.rs:1192`）。因此升级后唯一可见变化是：**用户手写的 `RustCode*` / `AtomGit*` provider 名，从「折叠成一个只读账号」变成「各自独立、可自由编辑」**，且名为 `deepseek-v4-flash` 且未声明 levels 的模型不再被客户端限制为 high/max。这两条即用户 Q2 已授权的语义变更。

#### 4.2.2 内置 effort levels 的处置

`codingplan_builtin_effort_levels` 与 `effective_reasoning_effort_levels` **整体删除**，不保留任何等价物。理由：它是一个硬编码模型名（`deepseek-v4-flash`）的客户端产物，`AGENTS.md:1309` 已明确「no hardcoded model name needed anywhere」——保留它等于为已删的托管网关留下一条隐形行为。`REASONING_EFFORT_LEVELS`（`:1269`）、`allowed_effort_levels`（`:1325`）、`clamp_effort_to_levels`（`:1351`）、`endpoint_supports_reasoning_effort`（`:1310`）**全部不变**。

#### 4.2.3 是否需要一次性迁移：**确认不需要**（支持 `00-requirement.md` Q7 推荐）

理由：① **磁盘格式未变**——`[providers.*]` 的键、值、schema 全部不动，无任何需要「转换」的数据；② 旧键仍合法可加载可选用，不存在孤儿；③ `AGENTS.md:173-175` 禁止恢复 legacy writer 与双向转换，任何「把 `RustCode*` 重命名成用户自定义名」的迁移器正是被禁止的写入器，且会引入迁移失败语义（违反 fail-closed）；④ 只读保护消失后，用户若想改名可自愿通过 `/provider` 完成，属增量行为。**本设计明确：零迁移、零 importer、零 writer。**

#### 4.2.4 测试如何改写

| 现有测试 | 处置 |
|---|---|
| `config/mod.rs:1366-1401` `codingplan_prefix_tests`（7 个：前缀识别、账号分组、`prefixes_for`、双前缀） | **整体删除**（断言对象已不存在） |
| `config/mod.rs:4236-4237`（前缀相关断言） | **删除** |
| `endpoints.rs:379-396` `nothing_configured_keeps_the_hosted_addresses` | **改写**：删掉 `:386`/`:387`/`:395-396` 三条 codingplan 断言，其余保留 |
| `tuix/src/event_loop/monitor.rs:230-235` `is_codingplan_provider_matches_prefix_and_exact` | 随 `monitor.rs` 整体删除 |
| 新增（T-19） | `legacy_providers_project_one_account_per_provider`（3 个 `RustCode*` / `AtomGit*` 条目 → 3 账号 3 模型、key 为原名）、`declared_effort_levels_are_authoritative_without_builtin_fallback`、`empty_declared_levels_mean_unrestricted`、`legacy_provider_names_are_editable`（无 managed 概念；以「不存在任何 API 返回 true」的编译期事实表达） |

### 4.3 D-3 tuix 非门控消费点：`/usage` 与用量监控

**决策：整体删除，不保留、不在 tuix 内重定义等价类型。**

| 项 | 处置 |
|---|---|
| `tuix/src/modals/usage.rs`（约 1400 行）、`modals/usage_render.rs` | 删除 |
| `tuix/src/event_loop/usage_monitor.rs` | 删除 |
| `tuix/src/event_loop/monitor.rs`（353 行，100% codingplan：漂移监控 + `is_codingplan_provider`） | **整体删除**（不是只删门控块） |
| `LoopCtx.usage_slot`（`mod.rs:3964-3966`）、`usage_last_check_at`（`:3970`）、`monitor_warning`（`:3948`）、`monitor_last_check_at`（`:3958`）、`monitor_last_sync_seen`（`:3979`） | 删除字段与全部读写点（`:9723-9741`、`:12319-12345`、`:23001-23013`、`:26851-26858`、`:28568-28581`、`lib.rs:864/866/872/929-935`） |
| `state.rs:1209` `footer_usage`、`:2807` 夹具 | 删除 |
| `lib.rs:872` `read_last_sync()`、`:23-29` env ctor | 删除 |
| `/usage` 命令 | **保留注册与派发**（`BUILTIN_COMMANDS` 与 `MANAGED_ONLY_COMMANDS` 不变，`AGENTS.md:291`），`open_usage` 退化为一律渲染降级文案 |
| `Msg::UsageCodingPlanOnly`（`messages.rs:1733`） | **重命名为 `Msg::UsageUnavailableNeutral`** 并把 en/zh 文案改为中性（不含 CodingPlan 字样），`commands.rs:2011/5386` 调用点同步改名 |
| `/login` | **保留** `commands.rs:7311-7318` 的 `#[cfg(not(feature="codingplan"))]` 降级实现（删掉 `#cfg(feature="codingplan")]` 孪生实现后成为唯一实现），输出 `Msg::LoginManagedUnavailable`，**不得**变成静默空实现（fail-closed） |

**理由**：① 数据源（`rustcode_codingplan::client::Client`）随 L3 删除，弹窗永远为空，保留即为死 UI；② 「保留并本地定义等价类型」=把 `types.rs` 的 `PlanInfo/RateLimitWindow/UsageResponse` 与 `usage.rs` 的 `compute_overview/humanize_tokens` 复制进 tuix，制造**第二份格式化与状态形状**，直接违反单一所有权与「禁止以 facade / 复制状态所有者方式回流」（`AGENTS.md:150`）；③ 代价/收益倒挂：1400+ 行 UI + 约 70 个 i18n arm 只为渲染永远拿不到的数据。

### 4.4 D-4 公共协议退役

三条路由 `POST /codingplan/setup`、`GET /codingplan/usage/summary`、`GET /codingplan/usage/daily`（`daemon/src/lib.rs:6099-6112`）删除。

**不需要与扩展侧同批次**。关键论据：这三条路由在**默认构建的现状下已经返回 404**——`codingplan_routes()` 在 `#[cfg(not(feature="codingplan"))]` 下返回空 router（`lib.rs:6114-6116`），模块本身不编译（`:29-30`）。因此 B 方案**不改变默认构建的路由行为**，Rust 侧（批次 2）与扩展侧（批次 7）之间的中间态只存在「旧扩展调一个本来就 404 的路径」，无新增破坏。

约束：两侧必须在**同一次交付内**完成（批次 2 与批次 7 之间不得发布）；交付说明必须写明「三条路由永久消失；旧版扩展连新 daemon 收 404（与现状一致）」。

### 4.5 D-5 扩展与前端清理可行性

| 面 | 验证手段 | 决定 |
|---|---|---|
| **VS Code** | host + webview 双 `npx tsc --noEmit`、`npm run test:webview`（基线 11/11 组通过，`AGENTS.md:305`）、`npm run build:webview` | 纳入批次 7，任务 T-23。删除 `CodingPlanSetupResponse`（`types.ts:226`）、`setupCodingPlan()`（`client.ts:13/350/374-375`）、`provider.ts` 调用点（`:11/679-680/2036/2117-2146/2657`）、webview 类型/动作（`state/types.ts:327`、`ChatProvider.tsx:274`、`WelcomeScreen.tsx:45-46/136`）、对应 i18n 键与 `bundle.l10n.zh-cn.json:45`。**保留** `managed_available` 门控全部逻辑（N-4）。 |
| **JetBrains** | **本机无 JDK/gradle（`AGENTS.md:318`）→ 只能源码级验证**。验证手段：① `RustCodeBundle*.properties` 与 IntelliJ `%key` 引用的 parity 校验（0 missing / 0 unused，`AGENTS.md:399` 有先例脚本）；② `grep -rniI "codingplan" extensions/jetbrains` 归零；③ 括号/引用人工核对 | 纳入批次 7，任务 T-24，**交付报告必须标注「未编译验证」**。**最小化改动**：只删 `CodingPlanSetupResponse` DTO、`client.setupCodingPlan()`、`service.setupCodingPlan()`（`RustCodeProjectService.kt:546-556`）、`RustCodeChatPanel.kt:552` 调用与 `:1890-1894` 齿轮项、`GearMenuLabels.kt:14/33` 的 `codingPlanSetup` 字段与其构造点赋值、`GearMenuLabelsTest.kt:19/43` 对应断言行、两个 `.properties:8` 键。**不重构** `GearMenuLabels` 的其它结构，不动 `managedLogin` 门控。 |
| **webui** | `npm run typecheck`、`npm test`（基线 227/0）、`npm run build` + `cargo clean -p rustcode-daemon`（`AGENTS.md:16`） | **纳入批次 7 而非单列**（任务 T-25）。理由：改动仅 4 文件，单列批次会让 `webui/dist` 与源码长期不一致；合并可在同批完成重建与 `cargo clean`。删除 `i18n.ts:224/287/653/715`（`settings.modelsIntroManaged`、`settings.officialCodingPlan`）、`api.ts:419 requires_login` 字段、`SettingsDialogs.tsx:44-45` 判定与 `:310`/`:340` 徽标、`Chat.tsx:2384-2409` 的 CodingPlan 429 判定分支（改为只走通用 429 文案）。 |

### 4.6 D-6 冻结契约（删 / 改 / 不变三表）

见 §5。

### 4.7 D-7 硬约束校验（逐条）

| 约束 | 结论 | 依据 |
|---|---|---|
| **不恢复 core / bridge / v1-v2 开关 / fallback / facade**（`AGENTS.md:150`） | **合规** | 本方案是纯删除，新增代码仅限：config 投影的简化表达式、4 个新测试、1 个 i18n 变体改名。未引入任何兼容层、开关或转换。`RateLimitWindowSource` 与 `sign::RequestSigner` 是**既有中性接缝的保留**，不是新增 facade。 |
| **依赖方向只向下**（`AGENTS.md:56`） | **合规，且依赖图变小** | 删除的边：`tuix → rustcode-codingplan`（非可选，唯一非法嫌疑边）、`daemon → rustcode-codingplan`（optional）、`cli → rustcode-codingplan`（optional）、`auth → rustcode-codingplan-crypto`（optional）、`coding → capabilities::codingplan_sign`（模块内边）。**无新增边、无反向边**：`coding` 仍依赖 `capabilities`，`capabilities/auth/config` 仍为 leaf。删除后 leaf 仅 `config / auth / updater`，`AGENTS.md:52` 的 crate 地图需同步（T-26）。 |
| **kernel / capabilities / coding 保持 core-free** | **合规** | 三者均无新增依赖；`coding` 删掉对 `capabilities::codingplan_sign` 的引用后依赖只减不增。`kernel` 仅注释中立化（T-21）。 |
| **不新增第二运行时生命周期所有者** | **合规** | `CodingRuntime` 零改动；provider 装配仍在 `daemon/runtime_host.rs:87-89` 的 `coding_provider_factory()` 单点，仅把内部从 `codingplan_provider_factory(ua)` 改为 `DefaultCodingProviderFactory::new(ua)`，工厂仍由 `rustcode-coding` 提供，**daemon 不得自行构造 provider**。 |
| **出站 HTTP 唯一入口 `capabilities/src/egress/`**（`AGENTS.md:44`） | **正向收益，但表述必须精确** | 随 L3 删除的 `codingplan/src/client.rs:104-110` 是**自建 `reqwest::blocking::Client`**（`apply_blocking_proxy_policy(reqwest::blocking::Client::builder(), force_tls12)`），删除后**不再存在任何「自建客户端访问托管网关」的出站路径**。**但仓库内仍有与 codingplan 无关的既有 reqwest 直连**：`rustcode-auth/src/oauth.rs:150/1993/2023`（leaf 结构豁免，`AGENTS.md:295`）、`rustcode-updater/src/lib.rs:279/331`（leaf）、`tuix/src/modals/provider_panel.rs:1121`（**存量违规，登记表 follow-up，本 feature 不修**，见 §8.3）、`capabilities` 自身即 egress 工厂与其 adapter 豁免。**交付说明只能写「codingplan 的自建出站路径已归零 + 发现一处存量绕行已登记」，不得写「全仓再无 reqwest 直连」。** |

---

## 5. 冻结契约清单

> 冻结后不可擅改；任何变更需回到 `solution-architect` 重新冻结并通知编排者。

### 5.1 删除（Deletes）

**L1 —— `rustcode-config`**

| 符号 | 位置 |
|---|---|
| `CODINGPLAN_API_BASE_ENV` / `CODINGPLAN_LLM_BASE_URL_ENV` / `CODINGPLAN_PROVIDER_PREFIX_ENV` | `endpoints.rs:35` / `:36` / `:54` |
| `HOSTED_CODINGPLAN_API_BASE` / `HOSTED_CODINGPLAN_LLM_BASE_URL` / `HOSTED_CODINGPLAN_PROVIDER_PREFIX` | `endpoints.rs:75` / `:79` / `:116` |
| `codingplan_api_base()` / `codingplan_llm_base_url()` / `is_codingplan_llm_gateway()` / `codingplan_provider_prefix()` / `normalize_codingplan_prefix()` | `endpoints.rs:185` / `:192` / `:204` / `:265` / `:278` |
| `LEGACY_CODINGPLAN_PREFIX` / `name_matches_prefix()` / `prefixes_for()` / `codingplan_prefixes()` / `is_codingplan_provider_name()` / `codingplan_group_account_id()` | `config/mod.rs:1192` / `:1198` / `:1212` / `:1221` / `:1233` / `:1245` |
| `codingplan_builtin_effort_levels()` / `effective_reasoning_effort_levels()` | `config/mod.rs:1275` / `:1286` |
| `account_is_codingplan_managed()` / `selection_is_codingplan_managed()` | `config/mod.rs:820` / `:842` |
| 账号折叠分支 / 模型 account 重映射 / effort 回退调用 | `config/mod.rs:800-806` / `:864-874` / `:879-883` / `:1036-1040` |
| `mod codingplan_prefix_tests`（7 个用例）+ `:4236-4237` | `config/mod.rs:1366-1401` / `:4236-4237` |
| env 名（对外契约，同步进交付说明） | `RUSTCODE_CODINGPLAN_API_BASE` / `RUSTCODE_CODINGPLAN_LLM_BASE_URL` / `RUSTCODE_CODINGPLAN_PROVIDER_PREFIX` |

**L2 —— `rustcode-auth` / `rustcode-capabilities` / `rustcode-coding`**

| 符号 | 位置 |
|---|---|
| `gateway_crypto` 模块整体（`RequestSigner` / `SignInput` / `SignOutput` / `SignError` / `UnavailableSigner` / `RealSigner` / `signer()` / `signer_available()` / `is_codingplan_gateway()` / `canonical_chat_completions_path()` + 4 个测试） | `auth/src/gateway_crypto.rs:1-171`；`auth/src/lib.rs:17` |
| optional dep + `codingplan-crypto` feature | `auth/Cargo.toml:21` / `:25` |
| `codingplan_sign.rs` 整体（`CodingPlanRequestSigner` / `codingplan_request_signer()` / `is_codingplan_gateway` 与 `signer_available` 重导出 + 2 个测试） | `capabilities/src/provider/codingplan_sign.rs:1-153`；`provider/mod.rs:19` / `:28` |
| `ProviderAuthenticator` trait / `CodingPlanProviderAuthenticator` / `codingplan_provider_factory()` / `with_authenticator()` / `authenticator` 字段与分支 | `coding/src/provider_factory.rs:38-43` / `:45-64` / `:66-73` / `:97` / `:86` / `:214-216`；`coding/src/lib.rs:84-87` |

**L3 —— 网络侧**

| 符号 / 文件 | 位置 |
|---|---|
| `crates/rustcode-codingplan/` 全目录（`lib.rs` / `client.rs` / `setup.rs` / `types.rs` / `usage.rs` / `sync_marker.rs` / `Cargo.toml`） | — |
| `daemon/src/api_codingplan.rs` 全文件（677 行） | — |
| `codingplan_routes()` 两个 cfg 变体 + `.merge(codingplan_routes())` + 横幅三条端点描述 | `daemon/src/lib.rs:6096-6116` / `:6318` / `:6463-6476`；`daemon/src/lib.rs:29-30` / `:54` |
| `render_cp_auth_error()` / `render_codingplan_status_for_status_cmd()` 两个 cfg 变体 | `daemon/src/commands.rs:622-705` |
| `CodingPlanRateLimitSource` / `coding_plan_rate_limit_source()` | `daemon/src/runtime_host.rs:48-85`（含 `:3-8` 的 cfg import） |
| `run_codingplan_core()` 两个 cfg 变体 + `#[cfg(feature="codingplan")]` 常量 + login→codingplan 链 | `cli/src/main.rs:4510-4625` / `:2898` / `:1670-1718` |
| `tuix` 全部 `#[cfg(feature="codingplan")]` 块与 neutral 孪生 | `tuix/src/event_loop/commands.rs` 17 处（含 `:4971/4987/5063/5341/5374/5976/5990/6036/6053/6652/6673/6876/6935/7016/7108/7149/7311`）、`fetch_usage_data()`（`:5341-5377`）、`tuix/src/event_loop/mod.rs:18347` |
| `tuix` 非门控消费点 | `tuix/src/lib.rs:23-29` / `:872` / `:929-935`；`tuix/src/event_loop/mod.rs:3943-3979` / `:3964-3966` / `:9723-9741` / `:12319-12345` / `:23001-23013` / `:26851-26858` / `:28568-28581`；`tuix/src/event_loop/usage_monitor.rs` 全文件；`tuix/src/event_loop/monitor.rs` 全文件；`tuix/src/modals/usage.rs` + `usage_render.rs` 全文件；`tuix/src/state.rs:1209` / `:2807` |
| **Cargo feature**：`codingplan`（tuix `Cargo.toml:14`、daemon `:21`、cli `:29-34`）、`codingplan-crypto`（cli `:38`、daemon `:22`、auth `:25`）、`client`（`codingplan/Cargo.toml:15`） | — |
| **REST 路由**：`POST /codingplan/setup`、`GET /codingplan/usage/summary`、`GET /codingplan/usage/daily` | `daemon/src/lib.rs:6103/6105-6107/6109-6111` |

**L4 —— 闭源桩**：`crates/rustcode-codingplan-crypto/` 全目录；根 `Cargo.toml:3-12` 注释；`Cargo.lock:2859`。

**i18n**（三件套同提交删除）

| 变体 | 位置 |
|---|---|
| `CpSetupHeader`…`CpUpgradeRequired`（约 30 个） | `messages.rs:320-453` |
| `CpReauthAfter401`、`CodingPlanSetupFailed` | `messages.rs:23` / `:14` |
| `WelcomeOptionCodingPlan{,Hint}` | `messages.rs:6-7`（调用点 `tuix/src/modals/onboarding_wizard.rs:874-875`） |
| `StatusCpNotSignedIn` / `StatusCpFetchFailed` / `StatusCpAuthExpired` / `StatusCpNoActive` / `StatusCpLine` / `StatusCpUsage` / `StatusCpWindowExhausted` / `StatusCpWindowHint` | `messages.rs:503-523` |
| `DaemonEpCpSetup` / `DaemonEpCpUsageSummary` / `DaemonEpCpUsageDaily` | `messages.rs:4971` / `:4973` / `:4975` |
| `mod codingplan_crypto_tests` ×2 | `en.rs:3325` / `zh_cn.rs:3137` |
| `Usage*` 用量弹窗族（随弹窗删除，约 35 arm ×2 语） | `en.rs` / `zh_cn.rs`（`messages.rs` 对应段） |

**扩展 / webui**：VS Code / JetBrains / webui 的 `CodingPlanSetupResponse` DTO、`setupCodingPlan()` 客户端方法与调用点、齿轮/欢迎页入口、`settings.officialCodingPlan` 与 `settings.modelsIntroManaged` 双语键、`api.ts requires_login` 字段（清单见 §2.2）。

**文档**：`AGENTS.md:220-221/289/329/345/498/509/537` 等条目重写；`docs/` 12 份非归档 + `README.md` + 3 份 crate README + `extensions/jetbrains/README.md:108`。

### 5.2 新增 / 改写（Changes）

| 项 | 新契约 |
|---|---|
| `Config::logical_accounts()` | `crates/rustcode-config/src/config/mod.rs:797`。新语义：**每个 `[providers.*]` 条目投影为一个同名账号**；`provider_accounts` 仍在精确 id 冲突时优先。签名与返回类型不变。 |
| `Config::logical_models()` | `:859`。新语义：模型 key 为 provider 原名，`account` 恒为原名；`reasoning_effort_levels` 直接取投影值（不再回退）。签名与返回类型不变。 |
| `Config::provider_config_for_selection()` | `:1025`。legacy 分支不再调用 `effective_reasoning_effort_levels`，只保留 `clamp_effort_to_levels`。 |
| `Msg::UsageCodingPlanOnly` → **`Msg::UsageUnavailableNeutral`** | `messages.rs:1733`；en/zh 文案改为中性（不得出现 CodingPlan 字样）：en 指向「本构建无托管用量查询，用 `/cost` 看本地统计」，zh 同义。调用点 `tuix/src/event_loop/commands.rs:2011` / `:5386` 同步改名。 |
| `coding_provider_factory()` | `daemon/src/runtime_host.rs:87-89`。**保留函数名**，函数体改为 `Arc::new(rustcode_coding::DefaultCodingProviderFactory::new(rustcode_auth::RUSTCODE_USER_AGENT))`。 |
| daemon wire DTO | 删除 `ProviderAccountInfo.managed`（`api_config.rs:83-85/94`）与 `ProviderInfo.requires_login`（`api_config.rs:152-155`）。**这是 wire 契约变更**，webui 与两个扩展同批清理（T-23/24/25）。 |
| tuix `/usage` 派发 | `open_usage()`（`commands.rs:5381`）退化为：渲染 `t(Msg::UsageUnavailableNeutral)` 后 flush。**命令仍注册、仍可派发**，不产生空输出。 |
| tuix `/login` | `commands.rs:7311-7318` 的 neutral 实现成为唯一实现（删掉 cfg 孪生）；输出 `Msg::LoginManagedUnavailable`。 |
| 新测试 | `crates/rustcode-config/src/config/mod.rs` 新增 4 个（§4.2.4）；`endpoints.rs:379-396` 改写。 |
| `Cargo.lock` | 移除 `rustcode-codingplan`、`rustcode-codingplan-crypto` 两条 package（AC-13）。 |

### 5.3 明确不变（Frozen-unchanged）

| 项 | 说明 |
|---|---|
| `CodingRuntime` 生命周期与事件协议 | `coding/src/runtime.rs` 零符号耦合（仅 `:7894-7897` 注释中立化）。 |
| `CodingProviderFactory` trait 形状 | `coding/src/provider_factory.rs:75-81` 的 `build(&self, &CodingAgentConfig, Option<&str>) -> Result<Arc<dyn LlmProvider>, ProviderBuildError>` 不变；`DefaultCodingProviderFactory::build` 除删除 `:214-216` 的 authenticator 分支外逻辑不变。**改变**：新增结构体字面量不再需要 `.with_authenticator(...)`。 |
| `LlmProvider` trait | `kernel/src/provider.rs` 不变。 |
| native session 持久化模型 | `SessionManager/SessionMeta/SessionSnapshot/PresentationFile` 不变。 |
| `capabilities::provider::sign::{RequestSigner, RequestSigningError, SignedAuth}` + `OpenAiCompatConfig.request_signer` | 保留为中性接缝（恒为 `None`）；不删、不新增实现。 |
| `coding::rate_limit::{RateLimitHook, RateLimitWindowSource, RateLimitWindow, decide_from_windows}` | 保留。删除 `CodingPlanRateLimitSource` 后无源注入，`on_rate_limit` 仍承载 `max_attempts` 边界（`:203-212`）与 generic hint 回落。 |
| `rustcode_auth::managed_login_available()` 与 `/login`、`managed_available`（daemon `/auth/status`、webui、两扩展） | 保留（N-4）。 |
| `MANAGED_ONLY_COMMANDS`（`login/logout/whoami/usage`）与 `command_visible()` | 保留（`AGENTS.md:291-294`）。 |
| `cli/src/main.rs:4898` / `:4961` 的 completion 断言 | **保留**（`!script.contains("codingplan")` / `!about.contains("CodingPlan")`），作为永久回归守卫；列入 AC-6 白名单。 |
| `cli/src/uninstall/paths.rs:36/150`、`scripts/uninstall.sh:21`、`scripts/uninstall.ps1:24` | 保留（`codingplan_sync.json` 清理项，N-8）。 |
| `docs/archive/*`（22 处）、`docs/UPSTREAM_CREDITS.md:1`、`extensions/jetbrains/CHANGELOG.md:7` | 保留。 |

### 5.4 AC-6 残留白名单（新定义，替换 `00-requirement.md` 的旧白名单）

方案 B 终态，`grep -rniI "codingplan" crates/ --include=*.rs --include=*.toml` 的**全部**残留只能是这 4 行：

1. `crates/rustcode-cli/src/uninstall/paths.rs:36` / `:150` —— `"codingplan_sync.json"`
2. `crates/rustcode-cli/src/main.rs:4898` / `:4961` —— completion 断言字面量

**期望值：4**。任何第 5 条残留都必须逐条登记并说明，不得「大致相符」。

---

## 6. 状态所有权

| 状态 | 现状所有者 | 删除后 |
|---|---|---|
| `LoopCtx.usage_slot` / `monitor_warning` / `monitor_last_sync_seen` / `monitor_last_check_at` / `usage_last_check_at` | tuix 事件循环单一持有（`event_loop/mod.rs:3943-3979`），写入方为 `usage_monitor::spawn_check` / `monitor::spawn_check`，读取方为 `build_status` | **整体删除，无替代所有者**。单一所有者原则不受影响（减字段而非换手）。 |
| `$RUSTCODE_HOME/codingplan_sync.json` | 读写均由 `rustcode-codingplan::sync_marker` 独占（`sync_marker.rs:22`；`AGENTS.md` §5 E-2） | **无人读写**，磁盘文件成为孤儿（无害，AC-14 已覆盖缺失/损坏/存在三种情形）；卸载清单仍清理该文件（N-8），所有权归卸载器。 |
| `codingplan_prefixes()` / `codingplan_group_account_id()` 的 `OnceLock` 缓存 | `config/mod.rs:1222` / `:1248` 进程级缓存 | 随删除消失，**无替代缓存**（新投影是纯函数，不需要缓存）。 |
| `logical_accounts()` / `logical_models()` | `Config` 的只读投影（不写回，`config/mod.rs:795-796` 注释） | 保持只读投影，无新状态，无第二投影实现。 |
| `DefaultCodingProviderFactory.authenticator` | `coding/src/provider_factory.rs:86` | 字段删除；provider 构造的唯一所有者仍为 `DefaultCodingProviderFactory::build`。 |

---

## 7. 失败与取消语义

| 场景 | 处置（禁止静默降级 / 假成功） |
|---|---|
| **编译失败（E-4 cfg 断裂同族）** | 立即停止本批后续任务；**禁止**加 `#[allow(...)]`、加 cfg 兜底、删测试凑绿；定位到具体 `file:line` 后回到本批起点 commit 重做。 |
| **新增测试红** | 先按 `AGENTS.md:571-576` 二分：① 是否 locale 竞态（补 `test_lock()`，locale 无关型只持锁不 `set_locale`）；② 是否夹具（如 `RustCode*` / `AtomGit*` 前缀夹具需改写）；③ 才是真回归。**禁止改断言**（`AGENTS.md:233`）。 |
| **`/usage` 无数据** | 必须输出 `Msg::UsageUnavailableNeutral`，**不得**输出空行或静默成功。 |
| **`/login` 无托管服务** | 必须输出 `Msg::LoginManagedUnavailable`（`commands.rs:7311-7318`），**不得**静默返回 `Ok(())`。 |
| **provider 装配（签名缺失）** | L2 删除后不存在「网关可达但无签名器」的状态；`ProviderBuildError::SourceBuildGatewayUnsupported` 变体随 `CodingPlanProviderAuthenticator` 一并删除（其唯一生产方即该实现），**不保留死错误分支**。`ProviderBuildError::{Adapter, Authentication}` 保留。 |
| **磁盘写满（`AGENTS.md:350/374`）** | 仅清 `target/debug/{incremental,examples}`（可弃缓存），**禁止**删 `.rlib/.rmeta`；`cargo test` 恒用 `-j 1`。 |
| **并发会话（E-6）** | 每批开工前 `git status --short` 实测；**禁止** `git checkout` / `reset` 任何他人改动（含 `2026-09-09-omo-skills-import` 的两个未跟踪文件）。 |
| **回滚** | 每批 1 个 commit，反向顺序 revert（B9→B1）。单批回滚必须整批回滚：A-1（crate 删除 + tuix 依赖边 + Cargo.lock）、A-6（i18n 三件套）拆开必然编译失败。B7（扩展）与 B8（文档）可独立回退。 |

---

## 8. 架构边界核对、风险与开放问题

### 8.1 边界核对（逐条见 §4.7）：**全部合规**，无 `blocked` 项。

### 8.2 风险（按严重度）

| # | 风险 | 等级 | 缓解 |
|---|---|---|---|
| R-1 | **持久化语义变更**（账号不折叠、只读保护消失、`deepseek-v4-flash` 内置 levels 消失） | 高 | 用户已授权（Q2'）；不迁移；新增 4 个测试锁定新语义；交付说明逐条列出；回滚 = revert B5。 |
| R-2 | **批次 3（tuix）体量最大**：删 crate + 依赖边 + 3 个模块 + `LoopCtx` 5 个字段 + 17 处门控块，约 3000+ 行 | 高 | 拆 3 个任务、按 T-14 → T-15 → T-13 顺序合并；每步 `cargo check -p rustcode-tuix`；A-1 原子合并。 |
| R-3 | **i18n 三件套串行**（A-6）：`Msg` 的穷尽 match 使 i18n 无法与消费点并行删 | 中 | 采用「先删使用点、后删变体」的两阶段（B3/B4/B5 只停用，B6 统一删）；B6 独占三件套。 |
| R-4 | **JetBrains 无编译验证** | 中 | 源码级 parity + grep 归零；最小化改动；交付报告如实标注未验证。 |
| R-5 | **webui 重建**（`npm ci && npm run build` + `cargo clean -p rustcode-daemon`） | 中 | 归入 T-25 并在 B9 复验 daemon 嵌入 bundle；`npm ci` 若遇 npmmirror 幻影包，按 `AGENTS.md:323` 用官方 registry 重生成 lock。 |
| R-6 | **wire DTO 变更**（`managed` / `requires_login` 删除）与前端同批 | 中 | B4 删字段、B7 清前端；中间态前端读到 `undefined`（`=== true` 判定 fail-closed），不假阳性。 |
| R-7 | 磁盘 / 内存（8GB cgroup、历史 100% 满盘） | 中 | `-j 1`、唯一日志路径、必要时清 incremental/examples（`AGENTS.md:549/561`）。 |
| R-8 | doc 门禁（`check-zh-docs.py gate`）在 B8 后失败 | 低 | B8 结束必跑门禁，且 `--base <BASE_SHA>` 用 B1 记录的 commit。 |

### 8.3 开放问题 / 发现的 backlog（**不在本 feature 处理**）

| # | 问题 | 处置 |
|---|---|---|
| O-1 | `tuix/src/modals/provider_panel.rs:1121` `reqwest::blocking::Client::builder()` 自建（无 proxy policy、不经 egress） | 登记 follow-up，**本 feature 不修**（N-9）。 |
| O-2 | `capabilities::provider::sign::RequestSigner` 接缝在 L2 删除后无生产方（`request_signer` 恒 `None`） | 保留为中性接缝（N-5）；登记 follow-up 评估是否随 401 恢复路径一并收敛。 |
| O-3 | `coding::rate_limit::RateLimitWindowSource` 无生产方（仅 `max_attempts` 语义在用） | 保留（N-6）；登记 follow-up。 |
| O-4 | `config` 的 `[telemetry]` 兼容段、`atomgit` feature、managed-QR 登录流 | 明确不在范围，保持不变。 |
| O-5 | `00-requirement.md` header 仍 `blocked` | 由编排者在 G1 关闭前处理（见 §0）。 |

---

## 9. 验证策略（AC 映射）

| AC | 由谁/何时验证 | 关键点 |
|---|---|---|
| AC-0 基线留档 | **B1** | 7 条命令全执行并落盘 `03-impl/baseline.md`；记录 commit SHA、测试失败集、`N_RS` / `N_EXT` / `N_DOC` |
| AC-1 默认构建 | B9（每批 V-0 预检） | `cargo build` exit 0 |
| AC-2 全工作区构建 | B9 | `cargo build --workspace` exit 0 |
| AC-3 全量测试无新增失败 | B5 / B6 / B9 三次 | 失败集 ⊆ 基线；唯一允许 `mcp::registry::tests::trust_key_golden_matches_core_algorithm` |
| AC-4 feature 矩阵 | B4 结束 + B9 | 前 6 条均报 package/feature 不存在；第 7 条 `cargo check --workspace --all-targets` exit 0 |
| AC-5 格式与 lint | 每批 V-1；B9 跑 clippy 比对 | `cargo fmt --check` exit 0；clippy 新增告警 0 |
| AC-6 残留 grep | B9 | 期望 **4**（§5.4 白名单） |
| AC-7 CLI 面零痕迹 | B9 | `--help` / completion 三 shell grep 为 0；`cargo test -p rustcode --lib -- completion` 全绿 |
| AC-8 路由契约 | B9 | `grep -rn "codingplan/setup" crates/rustcode-daemon/src` = 0；三条路径仍 404 |
| AC-9 既有条目仍可加载 | B5（新测试）+ B9（手工 `RUSTCODE_HOME` 复核） | 旧测试删除、新测试 4 个绿；手工确认 3 个条目 → 3 账号 3 模型 |
| AC-10 卸载清单 | B9 | `cargo test -p rustcode --lib -- uninstall` 绿；三处计数合计 3（paths.rs:36/150 计入则 3 处脚本 + 2 处 rs） |
| AC-11 扩展与前端 | B7 + B9 | `grep -rniI "codingplan" extensions/ webui/src` = 0（CHANGELOG 例外按 N-7 保留 → 若保留则为 1，需在 B7 明确登记） |
| AC-12 文档口径 | B8 | `python3 scripts/check-zh-docs.py gate` exit 0；`docs/` 命中数按清单递减（`archive/` 22 处不动） |
| AC-13 Cargo.lock 与依赖图 | B3（crate 删）+ B9 | `cargo metadata` exit 0；`grep -n "rustcode-codingplan" Cargo.lock` = 0 |
| AC-14 磁盘状态无孤儿 | B9 | 临时 `RUSTCODE_HOME` 下放损坏/正常 `codingplan_sync.json`，启动不 panic（`sync_marker` 已删，行为上恒等同「文件不存在」） |
| AC-15 i18n 双语 arm | B6 + B9 | `cargo check -p rustcode-config --all-targets` + `cargo test -p rustcode-config --lib`（穷尽 match 编译期保证） |

---

## 10. 已知未验证范围（如实标注）

1. **本会话无 `Bash`**：`git` 基线与全部 `cargo` / `npm` 命令均未由我执行；`file:line` 证据均来自 `Grep` / `Read` 实读，但**未执行编译器验证**。
2. **JetBrains**：本机无 JDK/gradle（`AGENTS.md:318`），只能源码级验证（properties parity + grep 归零 + 人工核对），**不声称编译通过**。
3. **webui 视觉**：只做 `typecheck` / `npm test` / `build`，不做视觉校验。
4. `docs/archive/*`（22 处）与 `docs/UPSTREAM_CREDITS.md:1` 未清理（既定裁决）。
5. `extensions/jetbrains/CHANGELOG.md:7` 保留（历史记录），因此 AC-11 的 `extensions/` 命中数终态为 **1**，需由 T-24 明确登记为白名单例外。
