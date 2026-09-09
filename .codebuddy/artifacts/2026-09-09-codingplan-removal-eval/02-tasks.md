---
kind: task
id: TASKS-001
from: solution-architect
to: [project-manager, code-implementer]
feature: 2026-09-09-codingplan-removal-eval
status: ready
decision: proceed
requires: [DESIGN-001]
created: 2026-09-09
---

# TASKS-001 彻底移除 codingplan（方案 B）—— 任务图

> **通用前置（每个任务都适用）**
> - 工作目录 `/workspace/RustCode`；`cargo test` 一律加 `-j 1`（`AGENTS.md:549`）；日志路径必须唯一（`AGENTS.md:561`）；禁止 `sudo`（`AGENTS.md:19`）。
> - 每批开工前 `git status --short` 实测，**禁止** `checkout`/`reset` 任何他人改动（含 `2026-09-09-omo-skills-import` 的两个未跟踪文件）。
> - cli 包名是 `rustcode`，`-p rustcode-cli` 会失败（`AGENTS.md:13`）。
> - 每批结束必须：`cargo fmt --check` exit 0 且 `cargo check --workspace --all-targets`（默认 feature）exit 0。
> - 失败语义：编译红立即停手，禁止加 `#[allow]` / cfg 兜底 / 改断言凑绿（DESIGN §7）。
> - **i18n 三件套**（`messages.rs` + `en.rs` + `zh_cn.rs`）在任意批次只能由**一个**任务独占（穷尽 match，DESIGN §3.3 A-6）。

---

## 1. 批次总览

| 批次 | 主题 | 任务 | 并行性 | 结束后校验 |
|---|---|---|---|---|
| **1** | 基线留档 | T-10 | 单任务 | AC-0 全部命令成功 |
| **2** | L3 网络侧：daemon + cli | T-11 / T-12 | **并行** | V-0 / V-1 / V-2 / V-4 |
| **3** | tuix 消费点剥离（命令面 / lib+state / LoopCtx） | T-13 / T-14 / T-15 | **串行**：T-13 → T-14 → T-15 | V-0 / V-1 / V-2 |
| **4** | tuix 模块删除 + crate 摘除（原子） | T-16 / T-17 | **串行**：T-16 → T-17 | V-0 / V-1 / V-2 / AC-13 部分 |
| **5** | L2 消费点先改（daemon / tuix / clix） | T-18 / T-19 / T-20 | **并行** | V-0 / V-1 / V-2 |
| **6** | L2 生产方删除 + L4 闭源桩 | T-21 / T-22 | **串行**：T-21 → T-22 | V-0 / V-1 / V-2 / V-5(部分) |
| **7** | L1 生产方删除（config）+ 注释中立化 | T-23 / T-24 | **并行** | V-0 / V-1 / V-2 / V-6 |
| **8** | i18n 收尾 | T-25 | 单任务 | V-0 / V-1 / V-6 / AC-15 |
| **9** | 扩展与前端 | T-26 / T-27 / T-28 | **并行** | V-7 / AC-11 |
| **10** | 文档与 `AGENTS.md` 口径同步 | T-29 / T-30 | **并行** | V-8 / AC-12 |
| **11** | 全量验证与交付 | T-31 | 单任务 | AC-1 … AC-15 |

**可并行的批次**：批次 9（扩展/前端）与批次 10（文档）之间无文件交集，可与批次 8 之后任意时点并行；其余批次默认串行（跨 crate 或存在符号因果）。
**必须串行**：2 → 3 → 4 → 5 → 6 → 7 → 8（符号生产方/消费方因果链；反向会编译失败）。

---

## 2. 任务清单

### 批次 1 —— 基线留档

#### T-10 基线留档（AC-0）
- 批次：1 ｜ 依赖：无 ｜ 负责：code-implementer（或编排者）
- `files_owned`：`.codebuddy/artifacts/2026-09-09-codingplan-removal-eval/03-impl/baseline.md`（新建）
- 改动摘要：执行并记录 AC-0 的 7 条命令输出（branch / commit SHA / worktree 状态、`cargo build`、`cargo build --workspace`、`cargo fmt --check`、`cargo test -j 1 --workspace --no-fail-fast` 失败集全名、`N_RS` / 文件数 / `N_EXT` / `N_DOC` / `Cargo.lock` 中 `rustcode-codingplan` 行号）。**不改动任何生产文件**。
- 验证命令：
```bash
git rev-parse --abbrev-ref HEAD && git rev-parse HEAD && git status --short
cargo build 2>&1 | tail -5
cargo build --workspace 2>&1 | tail -5
cargo fmt --check
cargo test -j 1 --workspace --no-fail-fast 2>&1 | tail -40
grep -rniI "codingplan" crates/ --include=*.rs --include=*.toml | wc -l
grep -rniI "codingplan" extensions/ webui/src | wc -l
grep -rniI "codingplan" docs/ | wc -l
grep -n "rustcode-codingplan" Cargo.lock
```
- 验收标准：**AC-0**（全部命令执行成功并留档；失败集应只含 `mcp::registry::tests::trust_key_golden_matches_core_algorithm`，其余红必须先记录、不得归因于本 feature）。
- 预计风险：低。注意 8GB 内存与磁盘（必要时先清 `target/debug/incremental`）。

---

### 批次 2 —— L3 网络侧（daemon + cli）

#### T-11 daemon：删 `api_codingplan.rs`、三条路由、门控块与 feature（原子 A-2）
- 批次：2 ｜ 依赖：T-10 ｜ 可与 T-12 并行
- `files_owned`：
  - `crates/rustcode-daemon/src/api_codingplan.rs`（删除）
  - `crates/rustcode-daemon/src/lib.rs`
  - `crates/rustcode-daemon/src/commands.rs`
  - `crates/rustcode-daemon/src/runtime_host.rs`
  - `crates/rustcode-daemon/Cargo.toml`
- 改动摘要：删 `lib.rs:29-30/54`（cfg mod + use）、`:6096-6116`（`codingplan_routes()` 两变体）、`:6318`（`.merge`）、`:6463-6476`（横幅三条端点）；删 `commands.rs:622-705` 两个 cfg 变体并让 `assemble_status` 的 codingplan 段消失；删 `runtime_host.rs:3-8/48-85`（`CodingPlanRateLimitSource`）——**保留** `coding_provider_factory()`（`:87-89`，本批不改函数体）；`Cargo.toml:17-25` 删 `codingplan` 与 `codingplan-crypto` feature 与 optional dep。
- 验证命令：
```bash
cargo fmt --check
cargo check --workspace --all-targets
cargo test -j 1 -p rustcode-daemon --lib
cargo check -p rustcode-daemon --features codingplan --all-targets   # 期望：does not have the feature `codingplan`
grep -rn "codingplan" crates/rustcode-daemon/src | wc -l              # 期望 0
```
- 验收标准：**AC-2 / AC-4（第 4 条开始报 feature 不存在）/ AC-5**；`daemon --lib` 与基线一致（基线 307/0）。
- 预计风险：中。`api_codingplan.rs` 677 行一次性删除需确认无 `#[cfg]` 之外的引用；`runtime_host.rs` 删 cfg import 后注意 `async_trait` use 是否变成未用。

#### T-12 cli：删 `run_codingplan_core` 两变体、login→codingplan 链与 feature（原子 A-2）
- 批次：2 ｜ 依赖：T-10 ｜ 可与 T-11 并行
- `files_owned`：
  - `crates/rustcode-cli/src/main.rs`
  - `crates/rustcode-cli/Cargo.toml`
- 改动摘要：删 `:4510-4625` 两个 `run_codingplan_core` 变体及其 cfg 常量（`:2898`）、`:1670-1718` 的 login→codingplan 链（保留 headless 登录的其余语义）；**保留** `:4898` / `:4961` 的 completion 断言（AC-6 白名单）；`Cargo.toml:22-42` 删 `codingplan` / `codingplan-crypto` feature 与 optional dep（含 `rustcode-daemon/codingplan`、`rustcode-tuix/codingplan` 传递项）。
- 验证命令：
```bash
cargo fmt --check
cargo check --workspace --all-targets
cargo test -j 1 -p rustcode --lib -- completion
cargo check -p rustcode --features codingplan --all-targets           # 期望：does not have the feature
cargo build -p rustcode && ./target/debug/rustcode --help | grep -ci codingplan   # 期望 0
```
- 验收标准：**AC-4（第 5 条）/ AC-7 / AC-5**。
- 预计风险：低-中。注意 `main.rs:1670-1718` 的 `spawn_blocking(run_codingplan_core)` 删除后 `pending_run_login_setup` 等相关状态是否残留（本批只删 codingplan 分支，不删 login 状态机）。

---

### 批次 3 —— tuix 消费点剥离（串行：T-13 → T-14 → T-15）

> 本批必须按 T-13 → T-14 → T-15 顺序合并（每一步单独可编译）：先删使用点，再删字段。

#### T-13 tuix 命令面：门控块、`/usage` 降级、`/login` 单一降级实现
- 批次：3 ｜ 依赖：T-11、T-12
- `files_owned`：
  - `crates/rustcode-tuix/src/event_loop/commands.rs`
  - `crates/rustcode-tuix/src/commands.rs`
- 改动摘要：删 17 处 `#[cfg(feature="codingplan")]` 块与其 neutral 孪生；删 `fetch_usage_data()` 两变体（`:5341-5377`）；`open_usage()`（`:5381-5391`）退化为**恒渲染** `t(Msg::UsageCodingPlanOnly)` 后 flush（后续 T-25 改名为 `Msg::UsageUnavailableNeutral`）；`run_login_flow` 只保留 `#[cfg(not(feature="codingplan"))]` 版本（`:7311-7318`）并去掉 cfg 属性；删 `:7282`（`write_last_sync_now`）与 `:7286`（`read_last_sync`）调用；`BUILTIN_COMMANDS` 的 `usage`（`commands.rs:216`）与 `login`（`:191`）描述改用既有中性变体 `CmdDescUsageNeutral` / `CmdDescLoginNeutral`。**不动** `Msg` 定义（T-25 统一处理）。
- 验证命令：
```bash
cargo fmt --check
cargo check -p rustcode-tuix --all-targets
cargo test -j 1 -p rustcode-tuix --lib 2>&1 | tail -20
```
- 验收标准：**AC-2 / AC-5**；tuix lib 测试无新增失败。
- 预计风险：中。`rate_limited_tests`（9 个，断言 `format_rate_limited_line`）与 `/usage`、`/login` 相关测试可能需改夹具，改夹具不改断言。

#### T-14 tuix 入口与渲染：`lib.rs` / `state.rs` / `render/*` / `sanitize.rs`
- 批次：3 ｜ 依赖：T-13
- `files_owned`：
  - `crates/rustcode-tuix/src/lib.rs`
  - `crates/rustcode-tuix/src/state.rs`
  - `crates/rustcode-tuix/src/render/mod.rs`
  - `crates/rustcode-tuix/src/render/retained.rs`
  - `crates/rustcode-tuix/src/render/qr.rs`
  - `crates/rustcode-tuix/src/render/cell.rs`
  - `crates/rustcode-tuix/src/sanitize.rs`
- 改动摘要：删 `lib.rs:23-29` 的 `RUSTCODE_CODINGPLAN_LLM_BASE_URL` env 设置、`:872` `read_last_sync()`（改 `monitor_last_sync_seen: None`）、`:864/866` 初始化、`:929-935` 启动漂移检查；删 `state.rs:1209` `footer_usage` 与 `:2807` 夹具；`render/*` 与 `sanitize.rs` 的 codingplan 注释/测试串中立化（`retained.rs:7485/9555/21020`、`qr.rs:2`、`mod.rs:466/641-642`、`cell.rs:482`）。
- 验证命令：
```bash
cargo fmt --check
cargo check -p rustcode-tuix --all-targets
cargo test -j 1 -p rustcode-tuix --lib 2>&1 | tail -20
grep -rniI "codingplan" crates/rustcode-tuix/src/lib.rs crates/rustcode-tuix/src/state.rs crates/rustcode-tuix/src/render crates/rustcode-tuix/src/sanitize.rs | wc -l  # 期望 0
```
- 验收标准：**AC-2 / AC-5**；无新增失败。
- 预计风险：中。删除 env 后，依赖 `is_codingplan_gateway("https://gateway.test.example/v1")==true` 的测试（若有）会翻转 → 由 T-19（批次 5）统一收口；本批若已见红，先记录并在 T-19 处理，不得改断言。

#### T-15 tuix `LoopCtx`：删 usage/monitor 字段与全部调用点
- 批次：3 ｜ 依赖：T-14
- `files_owned`：`crates/rustcode-tuix/src/event_loop/mod.rs`
- 改动摘要：删 `usage_slot`（`:3964-3966`）、`usage_last_check_at`（`:3970`）、`monitor_warning`（`:3948`）、`monitor_last_check_at`（`:3958`）、`monitor_last_sync_seen`（`:3979`）字段与其全部读写点（`:9723-9741`、`:12319-12345`、`:23001-23013`、`:26851-26858`、`:28568-28581`、`:18347`、`:5492-5580` 的 usage 面板测试）；删 `:28517-28541` 的 `needs_official_build` 分支（L2 分支，随 `signer_available()` 一并消失；`Msg::StatusOfficialBuildRequired` 使用点清零，变体由 T-25 决定是否删）；注释中立化。
- 验证命令：
```bash
cargo fmt --check
cargo check -p rustcode-tuix --all-targets
cargo test -j 1 -p rustcode-tuix --lib 2>&1 | tail -20
```
- 验收标准：**AC-2 / AC-5**；`git diff --stat` 中本文件无 `rustcode_codingplan` 残留引用。
- 预计风险：**高**（本文件是 tuix 最大的 codingplan 面，83 处命中；`build_status` 的 hint 优先级链改动易漏臂）。

---

### 批次 4 —— 模块删除 + crate 摘除（原子 A-1）

#### T-16 删除 tuix 的四个 codingplan 模块
- 批次：4 ｜ 依赖：T-15
- `files_owned`：
  - `crates/rustcode-tuix/src/event_loop/usage_monitor.rs`（删除）
  - `crates/rustcode-tuix/src/event_loop/monitor.rs`（删除）
  - `crates/rustcode-tuix/src/modals/usage.rs`（删除）
  - `crates/rustcode-tuix/src/modals/usage_render.rs`（删除）
  - `crates/rustcode-tuix/src/modals/mod.rs`
- 改动摘要：删四个文件；`modals/mod.rs` 去掉 `mod usage;` / `mod usage_render;` 与相关 `pub use`；`event_loop/mod.rs` 的 `mod monitor;` / `mod usage_monitor;` 声明**由 T-15 已删**（若仍有残留本任务一并清）。
- 验证命令：
```bash
cargo fmt --check
cargo check -p rustcode-tuix --all-targets
cargo test -j 1 -p rustcode-tuix --lib 2>&1 | tail -20
```
- 验收标准：**AC-2 / AC-5**。
- 预计风险：中（`usage_render.rs` 的 `braille_line_plot` / `calendar_layout` / `sparkline` 等是否被其它模态复用，需先 grep 确认零引用）。

#### T-17 摘除 `rustcode-codingplan` 依赖边并删除 crate（原子 A-1）
- 批次：4 ｜ 依赖：T-16
- `files_owned`：
  - `crates/rustcode-tuix/Cargo.toml`
  - `crates/rustcode-codingplan/**`（整目录删除）
  - `Cargo.lock`
- 改动摘要：`tuix/Cargo.toml:9-13/26-27` 删 `rustcode-codingplan` **非可选**依赖与 `codingplan` feature 及其注释；删除 `crates/rustcode-codingplan/` 全目录；`Cargo.lock` 移除 `rustcode-codingplan` 包条目。`uninstall/paths.rs` 的 `codingplan_sync.json` **不动**（N-8）。
- 验证命令：
```bash
cargo fmt --check
cargo metadata --format-version 1 > /dev/null
cargo check --workspace --all-targets
grep -n "rustcode-codingplan" Cargo.lock | wc -l        # 期望 0（codingplan-crypto 条目由 T-21 处理）
cargo check -p rustcode-codingplan --all-targets        # 期望：did not match any packages
```
- 验收标准：**AC-13**（部分）/ **AC-4 第 1 条** / **AC-2**。
- 预计风险：**高**（唯一「删生产者」任务；必须与依赖边同提交，否则 tuix 立即不可编译）。

---

### 批次 5 —— L2 消费点先改（并行）

> 本批只**停止使用** L2 符号，不删符号本体（本体在批次 6 删）。这样每个任务都可独立编译。

#### T-18 daemon：去 `is_codingplan_gateway` / `signer_available` 与 managed DTO 字段
- 批次：5 ｜ 依赖：T-17 ｜ 与 T-19、T-20 并行
- `files_owned`：
  - `crates/rustcode-daemon/src/api_provider.rs`
  - `crates/rustcode-daemon/src/api_config.rs`
  - `crates/rustcode-daemon/src/main.rs`
  - `crates/rustcode-daemon/src/runtime_host.rs`
- 改动摘要：
  - `api_provider.rs`：删本地 `selection_is_managed()`（`:38-44`）与 `account_is_managed()`（`:46-56`）及 5 处 403 守卫（`:790/923/1039/1200/1300`）、删 `managed_provider_locked_message()`（`:62-80`）与其测试（`:1425` 起）和相关 import（`:1378-1381`）；**保留** `selection_name_is_reserved()` 与 `Msg::DaemonProvManagedReserved`。
  - `api_config.rs`：删 `ProviderAccountInfo.managed`（`:83-85/94`）与 `ProviderInfo.requires_login`（`:152-155`）**字段本身**（wire 契约变更），并更新构造点。
  - `main.rs:151`：删 `signer_available()` 分支。
  - `runtime_host.rs:87-89`：`coding_provider_factory()` 改为 `Arc::new(rustcode_coding::DefaultCodingProviderFactory::new(rustcode_auth::RUSTCODE_USER_AGENT))`（函数名保留）。
- 验证命令：
```bash
cargo fmt --check
cargo check --workspace --all-targets
cargo test -j 1 -p rustcode-daemon --lib 2>&1 | tail -20
grep -rn "is_codingplan_gateway\|signer_available" crates/rustcode-daemon/src | wc -l   # 期望 0
```
- 验收标准：**AC-2 / AC-5**；daemon lib 与基线一致。
- 预计风险：中-高。`api_provider.rs` 的 403 测试（`:1454/1465/1477`）需整体删除；`managed_provider_locked_message` 删除后 `Msg::DaemonProvManagedLocked` / `DaemonProvAction*` 可能变死消息 → 由 T-25 复核后删。

#### T-19 tuix：去 L2 与 L1 消费点（provider_panel + event_loop/mod.rs）
- 批次：5 ｜ 依赖：T-17 ｜ 与 T-18、T-20 并行
- `files_owned`：
  - `crates/rustcode-tuix/src/modals/provider_panel.rs`
  - `crates/rustcode-tuix/src/event_loop/mod.rs`
- 改动摘要：`provider_panel.rs` 删 L2 行 `:936`（`is_codingplan_gateway`）与 L1 行 `:438/807/916/930/939/1306/1384/1570`（`account_is_codingplan_managed` / `is_codingplan_provider_name`）及其只读保护分支与测试；`mod.rs` 删 `:10725` 的 `is_codingplan_gateway` 用法（判定改为「不需要签名网关」的直白语义，无托管概念）；顺带处理 T-14 因删除 env 而翻转的测试（**只改夹具/删除不可达断言，不得削弱其它断言**）。
- 验证命令：
```bash
cargo fmt --check
cargo check -p rustcode-tuix --all-targets
cargo test -j 1 -p rustcode-tuix --lib 2>&1 | tail -20
grep -rn "is_codingplan" crates/rustcode-tuix/src | wc -l    # 期望 0
```
- 验收标准：**AC-2 / AC-5**；tuix lib 无新增失败。
- 预计风险：**高**（`provider_panel.rs` 的 vendor_locked / 只读保护是产品行为，删除后「可编辑」是新语义；需确认没有测试断言「不可编辑」）。

#### T-20 clix：去 `is_codingplan_gateway` 分支
- 批次：5 ｜ 依赖：T-17 ｜ 与 T-18、T-19 并行
- `files_owned`：`crates/rustcode-clix/src/main.rs`
- 改动摘要：删 `:886` 的 `is_codingplan_gateway(base_url)` 判定与其「签名网关不可用」报错分支（含 `:1855-1857` 的既有测试一并删除或改为「不再有该分支」的等价断言）；确认删除后不会对任何 `base_url` 静默降级为「无签名请求」（本来就没有签名器，语义不变）。
- 验证命令：
```bash
cargo fmt --check
cargo check --workspace --all-targets
cargo test -j 1 -p rustcode-clix 2>&1 | tail -10
grep -rn "codingplan" crates/rustcode-clix/src | wc -l      # 期望 0
```
- 验收标准：**AC-2 / AC-5**；clix 与基线一致（基线 44/0）。
- 预计风险：低。

---

### 批次 6 —— L2 生产方删除 + L4 闭源桩（串行：T-21 → T-22）

#### T-21 删 `auth::gateway_crypto` + `capabilities::codingplan_sign` + 闭源桩（原子 A-3）
- 批次：6 ｜ 依赖：T-18、T-19、T-20
- `files_owned`：
  - `crates/rustcode-auth/src/gateway_crypto.rs`（删除）
  - `crates/rustcode-auth/src/lib.rs`
  - `crates/rustcode-auth/Cargo.toml`
  - `crates/rustcode-capabilities/src/provider/codingplan_sign.rs`（删除）
  - `crates/rustcode-capabilities/src/provider/mod.rs`
  - `crates/rustcode-codingplan-crypto/**`（整目录删除）
  - `Cargo.toml`（根，删 `:3-12` 的 crypto overlay 注释）
  - `Cargo.lock`
- 改动摘要：删 `gateway_crypto` 模块与 `lib.rs:17` 声明、删 `auth/Cargo.toml:21/25` optional dep 与 `codingplan-crypto` feature；删 `codingplan_sign.rs` 与 `provider/mod.rs:19/28`（**保留** `pub use sign::{RequestSigner, RequestSigningError, SignedAuth};`，`mod.rs:36`）；删 `rustcode-codingplan-crypto/`；根 `Cargo.toml` 注释改写（保留 `members = ["crates/*"]`）。
- 验证命令：
```bash
cargo fmt --check
cargo check --workspace --all-targets
cargo test -j 1 -p rustcode-auth --lib && cargo test -j 1 -p rustcode-capabilities --lib 2>&1 | tail -20
cargo check -p rustcode --features codingplan-crypto --all-targets   # 期望：does not have the feature
grep -rn "rustcode-codingplan-crypto" Cargo.lock | wc -l             # 期望 0
```
- 验收标准：**AC-4 第 6 条 / AC-13 / AC-2 / AC-5**；capabilities 唯一失败仍为已知红 `trust_key_golden_matches_core_algorithm`。
- 预计风险：中（`capabilities` 的 `signer_available()` 测试（若有）随删；`auth --lib` 基线 42/0）。

#### T-22 coding：去 `ProviderAuthenticator` / `CodingPlanProviderAuthenticator` / `codingplan_provider_factory`
- 批次：6 ｜ 依赖：T-21
- `files_owned`：
  - `crates/rustcode-coding/src/provider_factory.rs`
  - `crates/rustcode-coding/src/lib.rs`
- 改动摘要：删 `ProviderAuthenticator` trait（`:38-43`）、`CodingPlanProviderAuthenticator`（`:45-64`）、`codingplan_provider_factory()`（`:66-73`）、`with_authenticator()`（`:97-100`）、`authenticator` 字段（`:86`）与 `:214-216` 分支、相关 import（`:5-7`）；删 `ProviderBuildError::SourceBuildGatewayUnsupported` 变体与其 Display/构造（唯一生产方即被删实现）；`lib.rs:84-87` 重导出同步。**不变**：`CodingProviderFactory` trait（`:75-81`）与 `DefaultCodingProviderFactory::build` 其余逻辑。
- 验证命令：
```bash
cargo fmt --check
cargo check --workspace --all-targets
cargo test -j 1 -p rustcode-coding --lib 2>&1 | tail -20
grep -rn "codingplan\|CodingPlan" crates/rustcode-coding/src/provider_factory.rs crates/rustcode-coding/src/lib.rs | wc -l  # 期望 0
```
- 验收标准：**AC-2 / AC-5**；coding lib 与基线一致（基线 430/0，8 ignored）。
- 预计风险：中（`ProviderBuildError` 是 pub enum，删除变体需确认无外部 match：仅 `runtime_host`/`provider_factory` 使用）。

---

### 批次 7 —— L1 生产方删除 + 注释中立化（并行）

#### T-23 config：删识别层与折叠，改写测试（A-5 的生产方侧）
- 批次：7 ｜ 依赖：T-19（tuix 消费点已清空）
- `files_owned`：
  - `crates/rustcode-config/src/endpoints.rs`
  - `crates/rustcode-config/src/config/mod.rs`
  - `crates/rustcode-config/src/config/provider.rs`（注释）
  - `crates/rustcode-config/src/tls.rs`（注释）
- 改动摘要：
  - `endpoints.rs`：删 `:35/36/54` 三个 env 常量、`:75/79/116` 三个 `HOSTED_CODINGPLAN_*`、`:185/192/204/265/278` 五个函数、`:25/73-79/113/259-263` 文档注释、`:386/387/395-396` 测试断言。
  - `config/mod.rs`：删 `LEGACY_CODINGPLAN_PREFIX`(`:1192`)、`name_matches_prefix`(`:1198`)、`prefixes_for`(`:1212`)、`codingplan_prefixes`(`:1221`)、`is_codingplan_provider_name`(`:1233`)、`codingplan_group_account_id`(`:1245`)、`codingplan_builtin_effort_levels`(`:1275`)、`effective_reasoning_effort_levels`(`:1286`)、`account_is_codingplan_managed`(`:820`)、`selection_is_codingplan_managed`(`:842`)；`logical_accounts()` 改为「每条目一个同名账号」（`:797-816`）；`logical_models()`（`:859-887`）account 恒为原名、不再调 `effective_reasoning_effort_levels`；`provider_config_for_selection()`（`:1036-1040`）同步；删 `mod codingplan_prefix_tests`（`:1366-1401`）与 `:4236-4237`。
  - **新增 4 个测试**（DESIGN §4.2.4）：`legacy_providers_project_one_account_per_provider`、`declared_effort_levels_are_authoritative_without_builtin_fallback`、`empty_declared_levels_mean_unrestricted`、`legacy_provider_names_are_editable`。
- 验证命令：
```bash
cargo fmt --check
cargo check --workspace --all-targets
cargo test -j 1 -p rustcode-config --lib 2>&1 | tail -20
grep -rniI "codingplan" crates/rustcode-config/src | grep -v "i18n/" | wc -l   # 期望 0（i18n 由 T-25 收尾）
```
- 验收标准：**AC-9**（新测试全绿 + 手工 `RUSTCODE_HOME` 复核 3 条目 → 3 账号 3 模型）/ **AC-2 / AC-5**。
- 预计风险：**高**（持久化语义变更的落点；`config` 是 327 测试的 crate，改动面 `:797-887` 会影响模型选择器/账号面板的一批断言，需逐名比对失败集）。

#### T-24 kernel / coding / capabilities 注释中立化
- 批次：7 ｜ 依赖：T-22 ｜ 与 T-23 并行
- `files_owned`：
  - `crates/rustcode-kernel/src/hook.rs`（`:92/124`）
  - `crates/rustcode-kernel/src/event.rs`（`:493/495`）
  - `crates/rustcode-kernel/src/agent.rs`（`:152`）
  - `crates/rustcode-coding/src/rate_limit.rs`（`:3/6/7/9/19/37/52/119/120/128/129/132/185/190/213/215/230/269/275/380` 注释）
  - `crates/rustcode-coding/src/config.rs`（`:625/631/632/643/661/857/915` 注释）
  - `crates/rustcode-coding/src/parts.rs`（`:1048/1052/1184/1697/1700` 注释）
  - `crates/rustcode-coding/src/persona.rs`（`:114`）
  - `crates/rustcode-coding/src/skill_first.rs`（`:18`）
  - `crates/rustcode-coding/src/runtime.rs`（`:7897`）
- 改动摘要：**仅注释**（无行为改动）：把「CodingPlan」字样改为中性的「managed plan quota / 托管额度窗口」等表述；`rate_limit.rs` 的 `RateLimitHook` / `RateLimitWindowSource` / `RateLimitWindow` / `decide_from_windows` **代码全部保留**（N-6）；`runtime.rs` 的 `CodingRuntime` 零改动（N-2）。
- 验证命令：
```bash
cargo fmt --check
cargo check --workspace --all-targets
cargo test -j 1 -p rustcode-coding --lib && cargo test -j 1 -p rustcode-kernel 2>&1 | tail -10
grep -rniI "codingplan" crates/rustcode-coding/src crates/rustcode-kernel/src | wc -l   # 期望 0
```
- 验收标准：**AC-2 / AC-5 / AC-6（kernel/coding 面归零）**。
- 预计风险：低（纯注释）。

---

### 批次 8 —— i18n 收尾（独占三件套）

#### T-25 i18n 三件套：删 `Cp*` / `StatusCp*` / `CodingPlanSetupFailed` / `WelcomeOptionCodingPlan*` / `DaemonEpCp*` / `Usage*` 族 + `/usage` 变体改名
- 批次：8 ｜ 依赖：T-23（且 T-13/15/18 的使用点已清零）
- `files_owned`：
  - `crates/rustcode-config/src/i18n/messages.rs`
  - `crates/rustcode-config/src/i18n/en.rs`
  - `crates/rustcode-config/src/i18n/zh_cn.rs`
  - `crates/rustcode-tuix/src/modals/onboarding_wizard.rs`
  - `crates/rustcode-tuix/src/event_loop/commands.rs`
- 改动摘要：
  - 三件套**同提交**删除：`Cp*` 约 30 个（`messages.rs:320-453`）、`CpReauthAfter401`(`:23`)、`CodingPlanSetupFailed`(`:14`)、`WelcomeOptionCodingPlan{,Hint}`(`:6-7`)、`StatusCp*`(`:503-523`)、`DaemonEpCp*`(`:4971/4973/4975`)、`Usage*` 用量弹窗族、两个 `mod codingplan_crypto_tests`（`en.rs:3325` / `zh_cn.rs:3137`）。
  - `Msg::UsageCodingPlanOnly`(`:1733`) **改名**为 `Msg::UsageUnavailableNeutral`，en/zh 文案改为中性（不得含 CodingPlan 字样；指向 `/cost` 本地统计）；`commands.rs:2011/5386` 调用点同步改名。
  - `onboarding_wizard.rs:874-875` 删 `WelcomeOptionCodingPlan{,Hint}` 的两行菜单项（菜单条数由 3 → 2，与 `setup_choices()` 的中立 2 行保持一致；若测试断言条数需同步）。
  - **复核后删**（先 grep 零引用再删）：`StatusOfficialBuildRequired`（T-15 已清零使用点）、`DaemonProvManagedLocked` / `DaemonProvAction*`（T-18 已清零）、`ProviderPanelManagedModelsHint` / `ProviderPanelManagedAccountHint` 及其 `*Neutral` arm（T-19 已清零）。**保留** `Msg::LoginManagedUnavailable`、`CmdDescLogin{,Neutral}`、`CmdDescUsage{,Neutral}`、`DaemonWarnNonLoopback`（`AGENTS.md:39` 明确保留）。
- 验证命令：
```bash
cargo fmt --check
cargo check -p rustcode-config --all-targets
cargo test -j 1 -p rustcode-config --lib 2>&1 | tail -20
cargo check --workspace --all-targets
grep -rniI "codingplan" crates/rustcode-config/src | wc -l   # 期望 0
```
- 验收标准：**AC-15**（编译期双语 arm parity + config lib 全绿）/ **AC-3**（config 失败集不扩大）/ **AC-5**。
- 预计风险：中-高（约 70+ arm ×2 语；漏删 arm 立即编译失败，漏删声明留死 arm —— 逐变体三处同删；`usage` 相关 vterm 测试可能断言弹窗标题，需同步删）。

---

### 批次 9 —— 扩展与前端（并行）

#### T-26 VS Code 扩展清理
- 批次：9 ｜ 依赖：T-18（daemon 已删 `managed` / `requires_login` 字段）
- `files_owned`：`extensions/vscode/src/**`、`extensions/vscode/webview-ui/src/**`、`extensions/vscode/l10n/bundle.l10n.zh-cn.json`、`extensions/vscode/package.nls*.json`（如需）
- 改动摘要：删 `CodingPlanSetupResponse`（`src/daemon/types.ts:226`）、`setupCodingPlan()`（`src/daemon/client.ts:13/350/374-375`）、`src/chat/provider.ts` 的调用与状态（`:11/679-680/2036/2117-2146/2657`）、webview `state/types.ts:327`、`ChatProvider.tsx:274`、`WelcomeScreen.tsx:45-46/136`、`i18n.tsx` 相关键、`bundle.l10n.zh-cn.json:45`。**保留** `managed_available` 门控全部逻辑（N-4）。
- 验证命令：
```bash
cd extensions/vscode && npx tsc --noEmit -p . && npx tsc --noEmit -p webview-ui
cd extensions/vscode && npm run test:webview    # 基线 11/11 组通过
cd extensions/vscode && npm run build:webview
grep -rniI "codingplan" extensions/vscode/src extensions/vscode/webview-ui/src | wc -l   # 期望 0
```
- 验收标准：**AC-11**。
- 预计风险：中（若 `npm ci` 遇 npmmirror 幻影包，按 `AGENTS.md:323` 用 `npm_config_registry=https://registry.npmjs.org npm install` 重生成 lock）。

#### T-27 JetBrains 插件清理（**源码级验证，无 JDK/gradle**）
- 批次：9 ｜ 依赖：T-18
- `files_owned`：`extensions/jetbrains/src/main/kotlin/**`、`extensions/jetbrains/src/main/resources/messages/RustCodeBundle.properties`、`RustCodeBundle_zh.properties`、`extensions/jetbrains/README.md`
- 改动摘要：删 `CodingPlanSetupResponse`（`RustCodeDaemonTypes.kt:118`）、`setupCodingPlan()`（`RustCodeDaemonClient.kt:201-203`）、`RustCodeProjectService.kt:546-556`、`RustCodeChatPanel.kt:552` 与 `:1890-1894` 齿轮项、`GearMenuLabels.kt:14/33` 的 `codingPlanSetup` 字段与其构造点赋值、`GearMenuLabelsTest.kt:19/43` 断言行、两个 `.properties:8` 键；`README.md:108` 现状描述改写。**不动** `managedLogin` 门控与 `GearMenuLabels` 其余结构；`CHANGELOG.md:7` 保留（历史记录，登记为 AC-11 白名单例外）。
- 验证命令（**无编译，仅源码级**）：
```bash
# 1) 键 parity：%key 引用计数 == en 键数 == zh 键数，0 missing / 0 unused
python3 - <<'PY'
import re,pathlib
base=pathlib.Path('extensions/jetbrains/src/main')
src="\n".join(p.read_text() for p in base.rglob('*.kt'))+"\n".join(p.read_text() for p in (base/'resources').rglob('*.xml'))
used=set(re.findall(r'(?<![\w%])%([A-Za-z0-9_.]+)', src))
for f in ['RustCodeBundle.properties','RustCodeBundle_zh.properties']:
    keys={l.split('=')[0].strip() for l in (base/'resources/messages'/f).read_text().splitlines() if l.strip() and not l.startswith('#')}
    print(f, 'missing', sorted(used-keys), 'unused', sorted(k for k in keys-used if not k.startswith(('action.','gear.','slash.'))==False))
PY
grep -rniI "codingplan\|CodingPlan" extensions/jetbrains/src extensions/jetbrains/README.md | wc -l   # 期望 0
```
- 验收标准：**AC-11**（`extensions/jetbrains/src` 与 `README.md` 归零；`CHANGELOG.md:7` 为登记例外）。
- 预计风险：**中-高（已知未验证范围）** —— 本机无 JDK/gradle（`AGENTS.md:318`），**不声称编译通过**；交付说明必须标注。

#### T-28 webui 清理 + 前端重建
- 批次：9 ｜ 依赖：T-18
- `files_owned`：`webui/src/i18n.ts`、`webui/src/api.ts`、`webui/src/components/SettingsDialogs.tsx`、`webui/src/components/Chat.tsx`、`webui/package-lock.json`（仅在需要重生成时）
- 改动摘要：删 `i18n.ts:224/287`（zh）与 `:653/715`（en）的 `settings.modelsIntroManaged`、`settings.officialCodingPlan`；删 `api.ts:419` `requires_login` 字段；删 `SettingsDialogs.tsx:44-45` 判定、`:310` 托管句、`:340` 徽标；`Chat.tsx:2384-2409` 的 CodingPlan 429 判定改为只走通用 429 文案。**保留** `managedAvailable` 门控（`LoginButton.tsx` / `Sidebar.tsx` / `slashCommands.ts`）与 `cmd.*` 键。
- 验证命令：
```bash
cd webui && npm ci && npm run typecheck && npm test && npm run build     # 基线 227/0
cd /workspace/RustCode && cargo clean -p rustcode-daemon && cargo check -p rustcode-daemon
grep -rniI "codingplan" webui/src | wc -l     # 期望 0
grep -rn "officialCodingPlan\|modelsIntroManaged" webui/dist | wc -l   # 期望 0
```
- 验收标准：**AC-11**；`cargo check -p rustcode-daemon` exit 0（新 bundle 已嵌入）。
- 预计风险：中（`npm ci` 网络/镜像风险；`cargo clean -p rustcode-daemon` 后需重链，注意磁盘）。

---

### 批次 10 —— 文档与 `AGENTS.md` 口径同步（并行）

#### T-29 `AGENTS.md` 重写被推翻的条目
- 批次：10 ｜ 依赖：T-25
- `files_owned`：`AGENTS.md`
- 改动摘要（68 处命中，逐条处理）：
  - **必须重写**：`:220-221` G7 门禁条文中的 `is_codingplan_provider_name` / `LEGACY_CODINGPLAN_PREFIX` 豁免（符号已删，条文失效）→ 改为「旧 `AtomGit-*` / `RustCode*` provider 名称不再享有折叠与只读保护；`atomgit` feature 与 fork 发行主页两类豁免保留」；`:289`、`:498`、`:509`、`:537` 的「`#[cfg(feature="codingplan")]` 块 / `LEGACY_CODINGPLAN_PREFIX` 不动」→ 改为「已于 2026-09-09 随 codingplan 一并移除」；`:329` 扩展侧 `/codingplan/setup` 保留开关 → 改为「路由与客户端已同步移除」；`:345` `Cp*` i18n 契约保留 → 改为「已随闭源签名 overlay 一并删除」。
  - **必须更新**：`:52` crate 地图（删 `rustcode-codingplan / rustcode-codingplan-crypto` 两行）、`:57`（`codingplan-crypto` feature 与官方构建说明）、`:58`（整条 feature 传递链说明 → 删除）、`:113`（`is_codingplan_provider_name` 引用）、`:98`（`is_codingplan_llm_gateway` 引用）、`:295`（网关/egress 复核结论）、`:393`/`:406`/`:417`/`:496`/`:539` 等含 `codingplan_crypto_tests` 的存量说明。
  - **保持**：门禁 G1–G8 的**结构**与 `atomcode` / `atomgit` 相关条文（除引用了 codingplan 符号的部分）。
- 验证命令：
```bash
python3 scripts/check-zh-docs.py gate
grep -n "codingplan\|CodingPlan" AGENTS.md | wc -l   # 期望 0（或仅剩「已移除」历史说明，需逐条登记）
```
- 验收标准：**AC-12**（中文文档门禁 exit 0）；门禁条文与代码一致（无失效 grep 模式）。
- 预计风险：中（G7 条文是**门禁自身**，重写后须确认 `grep -rni "atomcode"` 仍 0 命中且 `atomgit` 三类豁免仍可判定）。

#### T-30 `docs/` / README / 旧裁决标注
- 批次：10 ｜ 依赖：T-29（并行亦可，文件不相交）
- `files_owned`：`docs/platform-neutralization.md`、`docs/phase1-refactor-design.md`、`docs/i18n-style.md`、`docs/i18n-field-mapping.md`、`docs/architecture.md`、`docs/agent-api-rfc.md`、`docs/mcp-rmcp-feasibility.md`、`docs/config.example.toml`、`docs/plans/2026-07-26-provider-accounts-model-profiles-{plan,design}.md`、`docs/REFACTOR_DESIGN_PHASE1.md`、`README.md`、`crates/rustcode-daemon/README.md`、`crates/rustcode-clix/README.md`、`crates/rustcode-coding/README.md`
- 改动摘要：把「codingplan 作为当前能力/门控开关」的现状描述改为「已于 2026-09-09 移除」；`docs/config.example.toml:130` 的 CodingPlan 网关示例段删除；`daemon/README.md` 的 `/codingplan/*` 节删除。**不动**：`docs/archive/*`（22 处）、`docs/UPSTREAM_CREDITS.md:1`（MIT 合规）、`extensions/jetbrains/CHANGELOG.md:7`（历史）。
- 验证命令：
```bash
python3 scripts/check-zh-docs.py gate
grep -rniI "codingplan" docs/ | wc -l        # 期望 == 22（仅 archive/）+ 1（UPSTREAM_CREDITS）
grep -rniI "codingplan" README.md crates/*/README.md | wc -l   # 期望 0
```
- 验收标准：**AC-12**。
- 预计风险：低-中（若需为 `2026-09-02-cleanup-codingplan-legacy/STATUS.md:67-76` 的 S2 裁决加 `[SUPERSEDED]` 标注，**先取得编排者同意**再改（该文件属另一 feature 的交接件）。

---

### 批次 11 —— 全量验证与交付

#### T-31 全量验证、残留清单与交付说明
- 批次：11 ｜ 依赖：T-25、T-26、T-27、T-28、T-29、T-30
- `files_owned`：`03-impl/residual.md`、`05-test-report.md`、`06-release.md`（本 feature 目录下）
- 改动摘要：跑完 AC-1 … AC-15 全部命令；产出残留清单（逐行可溯源）；产出四段式交付说明（验证基线 / 四态 / 实际删除项 / 仍保留的 legacy surface / 测试结果 / 唯一下一步），**必须写明**：① 三条 `/codingplan/*` 路由永久消失（现状即 404，无行为回退）；② 持久化语义变更三条（账号不折叠、只读保护消失、`deepseek-v4-flash` 内置 effort 回退消失）；③ 三个 `RUSTCODE_CODINGPLAN_*` env 退役；④ 出站收益：codingplan 自建 `reqwest::blocking::Client` 已归零，并**登记** `tuix/src/modals/provider_panel.rs:1121` 的存量 reqwest 直连为 follow-up（不得声称全仓再无直连）。
- 验证命令：
```bash
cargo build && cargo build --workspace
cargo fmt --check
cargo clippy --workspace --all-targets 2>&1 | tee /tmp/clippy-after.txt     # 与基线逐条比对，新增 0
cargo test -j 1 --workspace --no-fail-fast 2>&1 | tail -40
# AC-4 七条
cargo check -p rustcode-codingplan --all-targets                       # 期望 package 不存在
cargo check -p rustcode-codingplan --features client --all-targets     # 期望 package 不存在
cargo check -p rustcode-tuix   --features codingplan --all-targets     # 期望 feature 不存在
cargo check -p rustcode-daemon --features codingplan --all-targets     # 期望 feature 不存在
cargo check -p rustcode        --features codingplan --all-targets     # 期望 feature 不存在
cargo check -p rustcode        --features codingplan-crypto --all-targets  # 期望 feature 不存在
cargo check --workspace --all-targets                                  # 期望 exit 0
# AC-6 残留（期望 4）
grep -rniI "codingplan" crates/ --include=*.rs --include=*.toml
# AC-7 / AC-8 / AC-10 / AC-13 / AC-14
cargo build -p rustcode && ./target/debug/rustcode --help | grep -ci codingplan
./target/debug/rustcode completion bash | grep -ci codingplan
./target/debug/rustcode completion zsh  | grep -ci codingplan
cargo test -p rustcode --lib -- completion && cargo test -p rustcode --lib -- uninstall
grep -rn "codingplan/setup" crates/rustcode-daemon/src | wc -l         # 期望 0
cargo metadata --format-version 1 > /dev/null && grep -n "rustcode-codingplan" Cargo.lock | wc -l   # 期望 0
grep -rniI "codingplan" extensions/ webui/src | wc -l                  # 期望 1（jetbrains CHANGELOG 登记例外）
python3 scripts/check-zh-docs.py gate
```
- 验收标准：**AC-1 … AC-15 全部**；残留清单逐条可溯源；失败集 ⊆ AC-0 基线（唯一允许 `trust_key_golden_matches_core_algorithm`）。
- 预计风险：中（全量测试耗时与磁盘；`clippy` 基线比对需逐条）。

---

## 3. 并行性矩阵

| 批次 | 任务 | 可并行 | 说明 |
|---|---|---|---|
| 1 | T-10 | — | 阻塞全部后续 |
| 2 | T-11 ∥ T-12 | 是 | daemon 与 cli 文件不相交；`cli/Cargo.toml` 删除对 `rustcode-daemon/codingplan` 的传递引用，与 T-11 不冲突（feature 已不存在即无引用） |
| 3 | T-13 → T-14 → T-15 | **否（串行合并）** | 先删使用点、再删字段；TU=文件相交于 `LoopCtx` |
| 4 | T-16 → T-17 | **否（串行）** | T-17 依赖 T-16 清空引用 |
| 5 | T-18 ∥ T-19 ∥ T-20 | 是 | 三个 crate 文件不相交 |
| 6 | T-21 → T-22 | **否（串行）** | coding 依赖 capabilities 符号 |
| 7 | T-23 ∥ T-24 | 是 | config 与 kernel/coding 不相交 |
| 8 | T-25 | — | 独占 i18n 三件套 |
| 9 | T-26 ∥ T-27 ∥ T-28 | 是 | 三个前端目录不相交 |
| 10 | T-29 ∥ T-30 | 是 | `AGENTS.md` 与 `docs/` 不相交 |
| 11 | T-31 | — | 收口 |

跨批次可并行：**批次 9 与批次 10**（与批次 8 之后任意时点）；批次 1 之后其余默认串行。

---

## 4. 集成顺序与回滚

1. **合并顺序**：严格 1 → 2 → … → 11；批次内按上表标注的串行顺序合并。每批 1 个 commit（批次 3/4 的串行任务按批 squash）。
2. **回滚**：反向 `git revert`（11 → 1）。以下组合**必须整批回滚**，拆开必然编译失败：
   - 批次 4（T-16 + T-17：模块删除 + 依赖边 + crate 删除 + `Cargo.lock`）
   - 批次 6（T-21 + T-22：`gateway_crypto` / `codingplan_sign` / `coding` 工厂）
   - 批次 8（T-25：i18n 三件套）
3. **可独立回滚**：批次 9（扩展/前端）、批次 10（文档）不影响 Rust 主体编译。
