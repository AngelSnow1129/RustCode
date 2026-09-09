---
kind: requirement
id: REQ-001
from: requirements-analyst
to: [solution-architect]
feature: 2026-09-09-codingplan-removal-eval
status: blocked
decision: block
requires: []
files_owned:
  - 00-requirement.md
architecture_constraints:
  touches_runtime_lifecycle: true
  touches_persistence: true
  touches_cross_crate_deps: true
created: 2026-09-09
---

# REQ-001 移除 codingplan 相关代码 —— 范围评估与边界澄清

> **只读分析产物。** 本轮仅落盘本文件，未修改任何生产代码 / `Cargo.toml` / `docs/**` / `AGENTS.md`。
> **工具限制（如实标注）**：本会话无 `Bash` 与 `AskUserQuestion` 权限。
> - 未执行 `git rev-parse` / `git log` / `git status`，故 `AGENTS.md:190` 要求的「branch / commit SHA / worktree 状态」与「近期 Git 历史」**证据不足**，G2 开工前必须由编排者补齐。
> - 无法向用户提问，故方向性歧义按规程升级为开放问题（Q1），本件 `status: blocked` / `decision: block`。
> - 行数类数据凡引自既有交接件者，均标注来源与本轮是否复核。

---

## 1. 背景与问题

### 1.1 用户原始诉求（逐字引用，未改写）

> 移除 /workspace/RustCode 仓库中的 codingplan 相关代码

### 1.2 现状：codingplan 在当前仓库的四种存在形态

本 fork 已完成平台中立化（`AGENTS.md:98`、`:113`、`:289`）。codingplan 相关代码经本轮逐文件复核，实际分为四层，**它们的可删除性完全不同**：

| 层 | 内容 | 门控 | 默认构建是否链接 |
|---|---|---|---|
| **L1 识别/兼容层（config）** | `is_codingplan_provider_name` / `LEGACY_CODINGPLAN_PREFIX` / `codingplan_provider_prefix` / `codingplan_group_account_id` / `codingplan_builtin_effort_levels` / `is_codingplan_llm_gateway` | 无 | **是** |
| **L2 安全/签名层（auth + capabilities + coding）** | `rustcode_auth::gateway_crypto`、`capabilities/provider/codingplan_sign.rs`、`coding::CodingPlanProviderAuthenticator` | 无 | **是**（但默认恒返回「无签名器」） |
| **L3 网络侧（codingplan crate 的 client/setup + daemon 路由）** | `client.rs` / `setup.rs` / `daemon/api_codingplan.rs` | `#[cfg(feature="codingplan")]` | **否** |
| **L4 闭源签名桩** | `crates/rustcode-codingplan-crypto/`（29 行，`sign_v1()` 体为 `unreachable!()`） | `codingplan-crypto` feature（官方构建才开） | **否** |

**关键事实：L3 与 L4 在默认构建下已经是死代码。** 三个 `HOSTED_*` 端点常量全为空串（`endpoints.rs:71/75/79`），因此 `is_codingplan_llm_gateway()` 恒 false（`endpoints.rs:204-218`），所有 provider 走纯 `bearer_auth(api_key)`，L4 桩永不被调用。

### 1.3 与既有裁决/铁律的冲突（必须先说清）

本诉求**与三处已记载结论直接冲突**，这不是可以自行推进的细节问题：

1. **上一轮用户裁决 S2 标准档**（`.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/STATUS.md:67-76`，已落 commit `783d48e4`）：
   > 用户裁决：**S2 标准档**。不移除 B 类（client/setup/api_codingplan）与 C 类闭源签名桩，保留官方闭源 overlay 接入点与 `Cp*` i18n 契约。
2. **`AGENTS.md` 四次记载「codingplan gate 不删」**：
   - `:289` ——「`atomgit` cargo feature、`#[cfg(feature="codingplan")]` 块、`LEGACY_CODINGPLAN_PREFIX` 识别器按 fork 铁律**不动**」
   - `:498` ——「codingplan gate、Cp* 签名 i18n 族…全部**刻意保留**（gate，不删除）」
   - `:509` 与 `:537` —— 同上，与 `atomgit` feature 同列
3. **`AGENTS.md:220-221` G7 门禁自身**：「旧 `AtomGit-*` provider 前缀兼容（`is_codingplan_provider_name` / `LEGACY_CODINGPLAN_PREFIX`，旧配置键仍需识别，**勿删**）」。删除这些字样会让 G7 门禁自身失效（`:226` 已就同类问题给过教训）。

**结论**：任何触及 L3/L4 的方案（即 B、D）都要求**用户显式推翻上述裁决**，并同步修订 `AGENTS.md` 相关条目。按上一轮的先例（`STATUS.md:85`：G1 判 blocked 并升级用户，而非自行选档推进），本件不得自行选档。

### 1.4 对任务书中两处前提的更正（本轮实测，非臆测）

| 任务书写法 | 实测更正 | 证据 |
|---|---|---|
| 「CLI 子命令 `rustcode codingplan` 与 TUI `/codingplan` slash 命令共用」 | **两者均已不存在**。CLI 隐藏别名已由上一轮 feature 删除；TUI `/codingplan` 早已被折叠进 `/login` | `crates/rustcode-cli/src/main.rs:4898`（`!script.contains("codingplan")`）、`:4961`（`!about.contains("CodingPlan")`）；`crates/rustcode-tuix/src/event_loop/commands.rs:7139-7142`「`/codingplan` used to be a separate slash command; it has been folded into `/login`」。另：`crates/rustcode-tuix/src/commands.rs` 的 `BUILTIN_COMMANDS` 中**没有** `codingplan` 条目，仅 `login`（`:191`）与 `usage`（`:216`）的描述文案提到 CodingPlan |
| 「crypto 桩真实依赖方是 `rustcode-auth`」 | 属实，但**签名能力的消费者远不止 codingplan**：`capabilities`/`coding`/`tuix`/`daemon`/`clix` 五个 crate 均在**非门控**代码里调用 `is_codingplan_gateway` / `signer_available` | `capabilities/provider/codingplan_sign.rs:97`、`coding/provider_factory.rs:52-60`、`daemon/runtime_host.rs:56`、`daemon/api_config.rs:85,155`、`daemon/api_provider.rs:43,55`、`daemon/main.rs:151`、`tuix/modals/provider_panel.rs:936`、`tuix/event_loop/mod.rs:10725,28529-28530`、`clix/main.rs:886` |

---

## 2. 目标 / 非目标

### 2.1 目标

- **G-1**：把「移除 codingplan」这一模糊诉求拆解为四个**可对照**的候选范围（A/B/C/D），每条给出删除清单、影响面与冲突项，使用户能一次性裁决。
- **G-2**：给出**可自动化判定**的验收标准与具体命令，覆盖编译、测试、feature 矩阵、残留 grep、持久化兼容、卸载清单六个面。
- **G-3**：明确标注本次改动是否触碰持久化格式 / 公共协议 / 安全边界 / 运行时生命周期，并给出每个风险面的**实测**（而非推断）。
- **G-4**：把与既有裁决/铁律的冲突显式升级，不在需求阶段偷偷选档。

### 2.2 非目标（明确排除，防范围蔓延）

| id | 非目标 |
|---|---|
| **N-1** | **不**删除或改名 `crates/rustcode-config` 中的 provider 前缀识别逻辑：`is_codingplan_provider_name`（`config/mod.rs:1233`）、`LEGACY_CODINGPLAN_PREFIX = "AtomGit"`（`:1192`）、`prefixes_for`（`:1212`）、`codingplan_group_account_id`（`:1245`）、`codingplan_provider_prefix`（`endpoints.rs:265`）。受 `AGENTS.md:220-221` G7 门禁保护，且删除会让**已写入用户磁盘**的 `RustCode*` / `AtomGit-*` 配置键失去识别。 |
| **N-2** | **不**修改 `~/.rustcode/config.toml` 的 `[providers.*]` / `[provider_accounts.*]` / `[models.*]` 结构，不引入任何迁移步骤、重命名步骤或写入器。 |
| **N-3** | **不**改动 `CodingRuntime` 的生命周期所有权、启动路径或事件协议。`CodingRuntime` 与 codingplan **零符号耦合**（`coding/src/runtime.rs` 全文件仅 `:7894-7897` 一处注释提及，无代码引用）。 |
| **N-4** | **不**恢复或新建任何 core 兼容层、bridge、v1/v2 开关、fallback 或 facade（`AGENTS.md:150`、`.codebuddy/rules/multi-agent-workflow.md:57` 硬约束）。 |
| **N-5** | **不**改动 `rustcode-auth::gateway_crypto` 的公共 API 形状（`is_codingplan_gateway` / `canonical_chat_completions_path` / `signer_available` / `RequestSigner`），因为五个 crate 在非门控路径调用它们。 |
| **N-6** | **不**进行产品改名、不做 i18n 文案大规模重写、不动 `atomgit` cargo feature、不动 `[telemetry]` 兼容段、不动 managed-QR 登录流（均为 `AGENTS.md:509/537` 记载的保留面）。 |
| **N-7** | **不**在需求阶段修改 `AGENTS.md` 的铁律条目或 G7 门禁定义本身。若 Q1 裁决为 B/D，则「同步修订 `AGENTS.md`」**单列为一个需用户二次确认的后续动作**，不混在删除任务里。 |
| **N-8** | **不**重建 WebUI 前端（`npm ci && npm run build`），除非 Q5 裁决要动 `webui/src`。 |
| **N-9** | **不**改动 `evals/`、`docker/`、`packages/` 下的 codingplan 字样（本轮仅 `evals/deepseek-v4-flash/eval.py:285` 一处，属评估脚本白名单）。 |
| **N-10** | **不**删除 `$RUSTCODE_HOME/codingplan_sync.json` 的**卸载清理项**（`cli/src/uninstall/paths.rs:36`、`scripts/uninstall.sh:21`、`scripts/uninstall.ps1:24`）—— 与 `AGENTS.md:87` 保留旧 `telemetry/` 目录同理：删了会漏清理真实存在的历史文件。是否清理见 Q6。 |

---

## 3. 用户故事

| id | 用户故事 |
|---|---|
| **US-1** | 作为 **fork 维护者**，我希望把四种「移除 codingplan」的解读摆成一张可对照的范围表，以便一次性裁决而不用反复追问。 |
| **US-2** | 作为 **fork 维护者**，我希望删除后默认构建的行为**零变化**（或变化被逐条列出），以便这个改动是纯删死码而不是行为变更。 |
| **US-3** | 作为 **fork 维护者**，我希望看到每条验收标准都带可直接复制执行的命令，以便 CI 与人工都能判定。 |
| **US-4** | 作为 **已配置过 `RustCode*` / `AtomGit*` provider 的老用户**，我希望升级后我的 `config.toml` 条目仍能正常加载、选择与使用，以便升级不打断我的工作。 |
| **US-5** | 作为 **安全审计者**，我希望明确知道闭源签名 overlay 的接入点是否被移除、以及移除后 provider 请求是否仍走纯 bearer auth，以便判断供应链与出网行为。 |
| **US-6** | 作为 **IDE 扩展使用者**，我希望知道 `POST /codingplan/setup` 这一 daemon 契约的处置结论（保留 / 删除 / 降级为 404），以便扩展不会突然报无法理解的错误。 |
| **US-7** | 作为 **执行实现的 Agent**，我希望提前知道 feature 传递链的断裂风险与 `cargo test` 的内存/磁盘约束，以便不重复踩坑。 |

---

## 4. 验收标准（AC）

> **通用前置**：所有命令在仓库根目录 `/workspace/RustCode` 执行；`cargo test --workspace` 必须加 `-j 1`（`AGENTS.md:549`：cgroup 内存上限 8GB，默认并发会 SIGBUS）。
> **基线要求（AC-0）**：任何删除动作开始前必须先留档基线，否则 AC-2/AC-3/AC-6/AC-11 的「与基线一致」无从判定。

### AC-0 基线留档（改动前执行，输出存入 `03-impl/`，必须成功）

```bash
git rev-parse --abbrev-ref HEAD && git rev-parse HEAD && git status --short
cargo build 2>&1 | tail -5                       # 期望 exit 0
cargo build --workspace 2>&1 | tail -5           # 期望 exit 0
cargo fmt --check                                # 期望 exit 0（AGENTS.md:557 已归零）
cargo test -j 1 --workspace --no-fail-fast 2>&1 | tail -40   # 记录 passed/failed 与失败用例全名
grep -rniI "codingplan" crates/ --include=*.rs --include=*.toml | wc -l   # 记为 N_RS
grep -rniI "codingplan" crates/ --include=*.rs --include=*.toml -l | wc -l
grep -rniI "codingplan" extensions/ webui/src | wc -l                     # 记为 N_EXT
grep -rniI "codingplan" docs/ | wc -l                                     # 记为 N_DOC
grep -n "rustcode-codingplan" Cargo.lock
```

**判定**：命令全部执行成功并留档；`cargo test` 的失败集应只含文档化已知红 `mcp::registry::tests::trust_key_golden_matches_core_algorithm`（`AGENTS.md:233`，铁律禁改）。若基线本身还有其他红，先记录、不得归因于本 feature。

### AC-1 默认构建通过

```bash
cargo build            # default-members = cli/daemon/tuix
```
**判定**：exit 0，0 error。

### AC-2 全工作区构建通过

```bash
cargo build --workspace
```
**判定**：exit 0，0 error。方案 B/D 下 `rustcode-codingplan` 可能已不在 workspace（B）或仍在（D），两种都必须是 exit 0。

### AC-3 全量测试无新增失败

```bash
cargo test -j 1 --workspace --no-fail-fast
```
**判定**：失败用例全名集合 **⊆** AC-0 基线失败集合。允许的唯一失败为 `trust_key_golden_matches_core_algorithm`。

### AC-4 feature 矩阵编译（防 cfg 传递链断裂 —— 最高风险项）

`AGENTS.md:58` 明确警告：驱动 crate 的 `codingplan` feature **必须**向上传递 `rustcode-codingplan/client`，否则 `#[cfg(feature="codingplan")]` 代码会引用被 cfg 掉的 `Client`/`run`/`is_auth_expired` 而编译失败。按所选方案逐条执行并按下表判定：

```bash
cargo check -p rustcode-codingplan --all-targets
cargo check -p rustcode-codingplan --features client --all-targets
cargo check -p rustcode-tuix    --features codingplan --all-targets
cargo check -p rustcode-daemon  --features codingplan --all-targets
cargo check -p rustcode         --features codingplan --all-targets
cargo check -p rustcode         --features codingplan-crypto --all-targets
cargo check --workspace --all-targets
```

| 方案 | 期望结果 |
|---|---|
| **A** | 前 6 条全部 exit 0；第 6 条改为 `cargo check -p rustcode --features codingplan --all-targets`（`codingplan-crypto` feature 已删，再传应报 `Package does not have feature`） |
| **C** | 与 AC-0 基线完全一致（零改动方案） |
| **D** | 第 2 条应报「`codingplan` does not have feature `client`」；第 3–6 条应报「does not have the feature `codingplan`」；第 1、7 条 exit 0 |
| **B** | 第 1–6 条均应报「package ID specification `rustcode-codingplan` did not match any packages」或 feature 不存在；第 7 条 exit 0 |

> 注意：`cli` 的**包名是 `rustcode`**（`crates/rustcode-cli/Cargo.toml:2`），`-p rustcode-cli` 会失败。

### AC-5 格式与 lint 无新增

```bash
cargo fmt --check                                   # 期望 exit 0 / 0 差异
cargo clippy --workspace --all-targets 2>&1 | tee /tmp/clippy-after.txt
```
**判定**：`cargo fmt --check` exit 0；`clippy` 告警**逐条比对** AC-0 基线，新增告警数为 0（存量 ~296 条基线不动，`AGENTS.md:393`）。

### AC-6 残留 grep 落在白名单内

```bash
grep -rniI "codingplan" crates/ --include=*.rs --include=*.toml \
  | grep -vE "is_codingplan_provider_name|LEGACY_CODINGPLAN_PREFIX|codingplan_provider_prefix|codingplan_group_account_id|codingplan_builtin_effort_levels|is_codingplan_llm_gateway|codingplan_llm_base_url|codingplan_api_base|HOSTED_CODINGPLAN_|RUSTCODE_CODINGPLAN_|is_codingplan_gateway|codingplan_sync\.json" \
  | wc -l
```

| 方案 | 期望值 | 说明 |
|---|---|---|
| **A** | `≤ N_RS - Δ_A`（Δ_A ≈ 60–80，含 `crates/rustcode-codingplan-crypto/` 全目录、`codingplan-crypto` feature 定义×3、`Cargo.lock` 1 条目、`gateway_crypto` RealSigner 分支、`codingplan_crypto_tests` 2 模块） | 允许残留：L1 识别层 + `is_codingplan_gateway` + 全部 `#[cfg(feature="codingplan")]` 块 |
| **B** | **0** | 上表中除 `codingplan_sync.json`（卸载清单，N-10）外**全部**清零 |
| **C** | `= N_RS` | 零改动 |
| **D** | `≤ N_RS - Δ_D`（Δ_D 主要来自 `client.rs` / `setup.rs` / `api_codingplan.rs` 全删 + 所有 `#[cfg(feature="codingplan")]` 块 + feature 定义） | 同 A 的白名单继续保留 |

**判定**：实测值 ≤ 上表期望值，且**每一条残留都能在上表白名单或本文件 §6 登记表中找到出处**。残留清单必须逐条写入 `03-impl/`，不得「大致相符」。

### AC-7 CLI 面零 codingplan 痕迹（既有契约回归）

```bash
cargo build -p rustcode
./target/debug/rustcode --help            | grep -ci codingplan   # 期望 0
./target/debug/rustcode completion bash   | grep -ci codingplan   # 期望 0
./target/debug/rustcode completion zsh    | grep -ci codingplan   # 期望 0
cargo test -p rustcode --lib -- completion
```
**判定**：前三行输出 `0`；`completion` 测试全绿（`main.rs:4898` 与 `:4961` 既有断言）。

### AC-8 daemon `/codingplan/*` 路由契约

三条路由：`POST /codingplan/setup`、`GET /codingplan/usage/summary`、`GET /codingplan/usage/daily`（`daemon/src/lib.rs:6099-6112`）。

- **方案 A / C**：行为必须与改动前一致 —— 默认构建返回 404。
  ```bash
  cargo build -p rustcode-daemon
  ./target/debug/rustcode-daemon --host 127.0.0.1 --port 13456 &
  curl -s -o /dev/null -w "%{http_code}\n" -X POST http://127.0.0.1:13456/codingplan/setup   # 期望 404
  ```
  > 若该端口/token 组合在 CI 不可用，**等价替代**：断言 `daemon/src/lib.rs:6114-6116` 的 `#[cfg(not(feature="codingplan"))] fn codingplan_routes()` 仍返回 `axum::Router::new()`，且 `cargo test -p rustcode-daemon --lib` 全绿（基线 307/0，`AGENTS.md:590`）。
- **方案 B / D**：路由整体消失，三条路径仍返回 404（与默认构建现状一致），且
  ```bash
  grep -rn "codingplan/setup" crates/rustcode-daemon/src | wc -l   # 期望 0
  ```

### AC-9 既有 `config.toml` 的 `RustCode*` / `AtomGit*` 条目仍可加载（持久化兼容，核心风险）

```bash
cargo test -p rustcode-config --lib -- codingplan_prefix_tests --nocapture   # 期望 7/7（config/mod.rs:1367-1401）
cargo test -p rustcode-config --lib                                          # 期望与基线一致（基线 327/0，AGENTS.md:589）
```

**手工补充判定**（必做，回答任务书风险面 1）：在临时 `RUSTCODE_HOME` 下写入
```toml
[providers.RustCode]
provider_type = "openai"
base_url = "https://api.example.com/v1"
api_key = "sk-test"
[providers."AtomGit-GLM-5.2"]
provider_type = "openai"
base_url = "https://api.example.com/v1"
api_key = "sk-test"
[providers.deepseek]
provider_type = "openai"
base_url = "https://api.example.com/v1"
api_key = "sk-test"
```
启动 TUI/daemon 后断言：`logical_accounts()` 仍产出 2 个账号（`RustCode` 与 `RustCode-…` 折叠进同一合成账号，`deepseek` 独立）、`logical_models()` 仍产出 3 个模型、选择 id 仍为 provider 原名。
**依据**：`config/mod.rs:797-816`（`logical_accounts`）、`:859-887`（`logical_models`）、`:1198-1203`（分隔符规则：`AtomGitx` / `AtomGit_GLM` 不匹配）。
**结论预告**：只要 N-1 成立（保留 config 识别层），**删除 codingplan crate 不会让既有条目变成孤儿**——识别与「移除」必须解耦。

### AC-10 卸载清单不退化

```bash
cargo test -p rustcode --lib -- uninstall
grep -c "codingplan_sync.json" scripts/uninstall.sh scripts/uninstall.ps1 crates/rustcode-cli/src/uninstall/paths.rs
```
**判定**：测试全绿；若 Q6 裁决「保留清理项」，三处计数合计仍为 3（`:21` / `:24` / `:36`，另 `:150` 为测试断言）。

### AC-11 扩展与前端不受影响（Q5 = 保留时的判定）

```bash
grep -rniI "codingplan" extensions/ webui/src | wc -l     # 期望 == AC-0 的 N_EXT（不减）
```
**判定**：数值与基线完全相同。若 Q5 裁决要清理，则改为逐条登记删除清单并复跑两个扩展的 `tsc --noEmit` 与 `npm test`（基线：VS Code `test:webview` 11/11 组通过；webui `npm test` 227/0，`AGENTS.md:305/530`）。

### AC-12 文档口径同步（Q4）

```bash
grep -rniI "codingplan" docs/ | wc -l                     # 期望 == N_DOC 或按 Q4 清单递减
python3 scripts/check-zh-docs.py gate --base <BASE_SHA>   # 期望 exit 0
```
**判定**：中文文档门禁 exit 0（`AGENTS.md:28`）；`docs/` 命中数变化必须能逐文件对上 Q4 裁决清单。
**注意**：`.codebuddy/artifacts/` 已被提升为门禁豁免域（`2026-09-09` memory 记录 Q6 裁决），本文件的 codingplan 字样不进分母。

### AC-13 Cargo.lock 与依赖图无孤儿

```bash
cargo metadata --format-version 1 > /dev/null                 # 期望 exit 0
grep -n "rustcode-codingplan" Cargo.lock
```
**判定**：A → 仅剩 `rustcode-codingplan` 一条（`:2846`）与依赖它的 crate；D → 同上；B → 0 条；`rustcode-codingplan-crypto`（`:2859`）在 A/B 下为 0、C/D 下保留。

### AC-14 磁盘状态无孤儿、启动不报错

前置：在临时 `RUSTCODE_HOME` 下放一个**损坏的** `codingplan_sync.json`（内容 `not json at all`）与一个**正常的**。
**判定**：两种情况下进程均正常启动，不 panic、不打印错误（依据 `sync_marker.rs:40-45` 对缺失/损坏一律返回 `None`，其 3 个既有测试 `write_then_read_round_trips_timestamp` / `read_last_sync_returns_none_when_file_absent` / `read_last_sync_returns_none_on_corrupt_json`）。
```bash
cargo test -p rustcode-codingplan --lib     # A/C/D 下期望 3/3（另有基线 27/0，AGENTS.md:406）；B 下该包不存在
```

### AC-15 i18n 双语 arm 完整（编译期 parity）

```bash
cargo check -p rustcode-config --all-targets
cargo test  -p rustcode-config --lib
```
**判定**：exit 0 且测试全绿。任何 `Msg` 变体的删除都必须**同时**删 `messages.rs` 声明 + `en.rs` 与 `zh_cn.rs` 的 arm —— 漏删 arm 立即编译失败，这是编译期保证；反之漏删声明会留下死 arm。

---

## 5. 异常与非功能场景

| id | 场景 | 现状与证据 | 要求的处置 |
|---|---|---|---|
| **E-1** | 用户磁盘已有 `RustCode*` / `AtomGit*` provider 条目 | 前缀由 `endpoints.rs:116` `HOSTED_CODINGPLAN_PROVIDER_PREFIX = "RustCode"` 决定，识别侧 `config/mod.rs:1233`，G7 保护（`AGENTS.md:220-221`） | **不迁移、不改写**。保留识别层即保留行为（AC-9）。方案 B 若连带删识别层 → 条目降级为普通自定义 provider、可被手工编辑、`RustCode*` 与 `RustCode-anthropic` 不再折叠为同一账号 → **属行为变化，必须先由用户裁决 Q2** |
| **E-2** | 用户磁盘已有 `codingplan_sync.json` | `sync_marker.rs:22`；读取端 `tuix/lib.rs:872`（非门控）、`tuix/event_loop/mod.rs:12320`（非门控）；写入端全部在 cfg 块内（`commands.rs:7282`、`cli/main.rs:4615`、`daemon/api_codingplan.rs:450,581`） | 删除 crate 后该文件成为**孤儿文件**：无害（无人读），但卸载必须仍能清掉 → N-10 + AC-10 |
| **E-3** | 用户正在执行 `/login`（托管流）或 `/usage` | 中立构建下两者已从所有发现面隐藏（`AGENTS.md:291-292` `MANAGED_ONLY_COMMANDS`），但**仍可手敲派发**：`/login` 走 `LoginManagedUnavailable`（`commands.rs:7311-7318`），`/usage` 走 `UsageCodingPlanOnly` | 方案 A/C/D 下行为不变；方案 B 需确认这两个命令的降级文案仍在（`Msg::LoginManagedUnavailable` `messages.rs:27`、`Msg::UsageCodingPlanOnly` `:1733`），不得出现「命令存在但无输出」的空实现 |
| **E-4** | **feature 传递链断裂导致编译失败** | `AGENTS.md:58` 明确警告。链：`cli:29-34` → `daemon:21` / `tuix:14` → `codingplan` `client` → `auth` `codingplan-crypto` | 这是**本任务最高概率的失败模式**。任何删除都必须按 AC-4 跑完整矩阵；只跑默认构建的 `cargo build` **不能**发现此类断裂 |
| **E-5** | `cargo test --workspace` OOM / 链接失败 | `AGENTS.md:549`（8GB cgroup，须 `-j 1`）；`:350/374/407/499`（磁盘 100% 致 cc 链接失败，需清 `target/debug/{incremental,examples}`） | 按 AC-0/AC-3 使用 `-j 1`；日志路径必须唯一（`AGENTS.md:561`：两进程共用日志路径会污染结果） |
| **E-6** | 并发会话 / worktree 冲突 | `AGENTS.md:560-562`、`:599-603` 记录过同一 worktree 上两个编排会话并发写入，且曾因「非 dirty 可安全 checkout」的误判差点回滚掉对方的改动 | 开工前必须 `git status --short` 实测，不得凭 `.gitignore` 或旧记录推断（`AGENTS.md:646` 教训：已跟踪文件不受 gitignore 约束）。dirty 的用户改动**禁止**重置或覆盖 |
| **E-7** | 用户残留旧发行版 `auth.toml`（含托管凭据） | `AGENTS.md:347` 记录过这条可达死路 | 不属本任务；但若方案 B 删掉 `gateway_crypto`，需确认 `provider_factory.rs:55-59` 的 `SourceBuildGatewayUnsupported` 分支不存在后不会退化为「静默无签名请求」。推荐保留该错误分支的语义 |
| **E-8** | 旧版扩展 / 前端 与新 daemon 混用 | 扩展在 TypeScript/Kotlin 侧各自定义了 `CodingPlanSetupResponse`（`extensions/vscode/src/daemon/types.ts:226`、`extensions/jetbrains/.../RustCodeDaemonTypes.kt:118`），不 import Rust 类型；调用 `POST /codingplan/setup`（`vscode/src/daemon/client.ts:374-375`、`jetbrains/.../RustCodeDaemonClient.kt:201-203`） | 方案 B/D 删路由 → 旧扩展拿到 404 并弹错。需 Q5 裁决：是保留路由（仅 cfg 门控）还是同步清理扩展 UI（`WelcomeScreen.tsx:136` 同步按钮、`RustCodeChatPanel.kt:1890-1894` 齿轮菜单项） |
| **E-9** | i18n arm 漏删 / 死消息 | `AGENTS.md:270` 教训：改 i18n 文案后必须跑**整个 crate 的 `--lib`**，内容测试散落在 `en.rs:3325` / `zh_cn.rs:3137` 的 `codingplan_crypto_tests` 模块里 | AC-15；方案 D 后 `messages.rs:320-423` 中仅供 `setup.rs` 渲染的 `Cp*` 变体会变死消息 → Q3 |
| **E-10** | locale 竞态导致假红 | `AGENTS.md:494`、`:571-576`：断言英文串的测试必须 `test_lock()` + `set_locale(En)`；但**locale 无关型**兄弟测试只持锁、绝不 set_locale，否则制造新红 | 删除 i18n 变体后若出现新红，先按此二分定位，不得直接改断言 |
| **E-11** | 权限 / 只读磁盘 | `AGENTS.md:19` 禁止 sudo；`~/.rustcode` 出现 root 属主文件会导致后续非 root 启动即失败 | 所有验证在临时 `RUSTCODE_HOME` 下跑（`AGENTS.md:29` 的 `#[ctor]` 已为 coding/tuix/daemon/capabilities/cli 做到） |
| **E-12** | 回滚 | 删除类任务必须 `git` 可回退 | 回滚方案 = `git revert` / `git checkout` 到 AC-0 记录的 commit；**前提是 E-6 的 dirty 面已被隔离**。方案 B 额外需要一次 `AGENTS.md` 回改 |

---

## 6. 受影响范围（初步）

### 6.1 按层列出（file:line 证据）

**L1 识别/兼容层 —— `rustcode-config`（始终编译，G7 保护，N-1 默认不动）**
- `endpoints.rs:54` `RUSTCODE_CODINGPLAN_PROVIDER_PREFIX` env；`:71/75/79` 三个空 `HOSTED_*`；`:116` `HOSTED_CODINGPLAN_PROVIDER_PREFIX = "RustCode"`；`:185` `codingplan_api_base()`；`:192` `codingplan_llm_base_url()`；`:204-218` `is_codingplan_llm_gateway()`；`:265-273` `codingplan_provider_prefix()`；`:395-396` 测试
- `config/mod.rs:797-816` `logical_accounts()`；`:818-838` `account_is_codingplan_managed()`；`:840-853` `selection_is_codingplan_managed()`；`:855-887` `logical_models()`；`:1018`、`:1037`；`:1192` `LEGACY_CODINGPLAN_PREFIX`；`:1198-1203` `name_matches_prefix`；`:1212-1218` `prefixes_for`；`:1221-1224` `codingplan_prefixes`；`:1233-1237` `is_codingplan_provider_name`；`:1245+` `codingplan_group_account_id`；`:1275` `codingplan_builtin_effort_levels`；`:1287-1299` `effective_reasoning_effort_levels`；`:1367-1401` + `:4236-4237` 测试
- `config/provider.rs` 1 处

**L2 安全/签名层 —— `rustcode-auth` / `rustcode-capabilities` / `rustcode-coding`（始终编译）**
- `auth/gateway_crypto.rs:49-55`（UnavailableSigner 回退）、`:57-93`（RealSigner，仅 `codingplan-crypto`）、`:90-98` `signer_available`、`:100-102` `is_codingplan_gateway`、`:104-114` `canonical_chat_completions_path`、`:116+` 测试
- `auth/Cargo.toml:21` optional dep、`:25` feature
- `capabilities/provider/codingplan_sign.rs`（全文 153 行，`:86-95` `codingplan_request_signer`、`:97` 重导出、`:99-153` 2 个测试）
- `capabilities/provider/mod.rs:19` `mod codingplan_sign;`、`:28` 重导出
- `coding/provider_factory.rs:38-43` `ProviderAuthenticator` trait、`:45-64` `CodingPlanProviderAuthenticator`、`:66-73` `codingplan_provider_factory`；`coding/lib.rs:84-87` 重导出
- 消费者：`daemon/runtime_host.rs:87-89`（**非门控**，唯一 chokepoint）、`daemon/main.rs:151`、`daemon/api_config.rs:85,155`、`daemon/api_provider.rs:43,55`、`daemon/runtime_host.rs:56`、`tuix/modals/provider_panel.rs:936`、`tuix/event_loop/mod.rs:10725,28529-28530`、`clix/main.rs:886` + `:1855-1857` 测试

**L3 网络侧 —— 门控，默认不链接**
- `codingplan/src/client.rs`、`setup.rs`（行数引自 `2026-09-02-cleanup-codingplan-legacy/STATUS.md:38`：602 / 3911，本轮未复核）
- `codingplan/src/lib.rs:21-24` cfg 门；`:33-36` gated 重导出；`Cargo.toml:15` `client = ["dep:reqwest"]`；`:20`、`:28` optional reqwest
- `daemon/src/api_codingplan.rs`（全文，677 行，引自同上 `:38`）；`daemon/src/lib.rs:6099-6116` `codingplan_routes()`、`:6318` `.merge(...)`、`:6463-6464` 横幅注释；`daemon/Cargo.toml:21-22`、`:25`
- 门控消费者：`cli/main.rs:4510-4625`（`:4512` / `:4527` / `:4539` / `:4540` 四处 cfg）；`tuix/event_loop/commands.rs:4971/4987/5063/5341/5374/5976/5990/6036/6053/6652/6673/6876/6935/7016/7108/7149/7311`；`tuix/event_loop/mod.rs:18347`；`tuix/event_loop/monitor.rs:21/78/105/140/197/208/224-331`；`tuix/event_loop/usage_monitor.rs:46/73`；`daemon/runtime_host.rs:48-85`；`daemon/commands.rs:622/635/640/645-646`
- **门控之外的真实消费者（删 crate 必须一起改，这是 B/D 的主要工作量来源）**：
  - `tuix/Cargo.toml:27` —— `rustcode-codingplan` 是**非可选**依赖
  - `tuix/src/lib.rs:872` `read_last_sync()`
  - `tuix/src/event_loop/mod.rs:3965`（`LoopCtx.usage_slot` 字段类型含 `rustcode_codingplan::types::UsageInfo`）、`:12319-12320`
  - `tuix/src/event_loop/usage_monitor.rs:21`（模块顶 `use`，`:73-74` 的 `#[cfg(not)]` 桩也用到该类型）
  - `tuix/src/modals/usage.rs:6-7`（`types::{PlanInfo, RateLimitWindow}`、`usage::{compute_overview, humanize_tokens, OverviewStats, UsageResponse}`）、`:932`、`:1329`
  - `cli/src/uninstall/paths.rs:36` + `:150`

**L4 闭源签名桩**
- `crates/rustcode-codingplan-crypto/`（`src/lib.rs` 29 行，`:28` `unreachable!()`；`Cargo.toml:5` `license = "Proprietary"`、`:3` `version = "0.1.0"`、`:7` 描述仍写 "AtomGit request-signing crate"、`:13` 引用的 `scripts/build-official.sh` **实际不存在**（本轮 Glob 未找到））
- 根 `Cargo.toml:7` `members = ["crates/*"]`、`:8-12` 注释
- `Cargo.lock:2859`

**i18n（`rustcode-config`）**
- `messages.rs:6-7` `WelcomeOptionCodingPlan{,Hint}`、`:14` `CodingPlanSetupFailed`、`:23-27` `CpReauthAfter401` / `LoginManagedUnavailable`、`:319-453` 约 30 个 `Cp*` 变体、`:4970-5011` 三条 `/codingplan/*` 端点描述、`:5146-5166`、`:1733` `UsageCodingPlanOnly`
- `en.rs` / `zh_cn.rs` 对应 arm（各 ~39/40 处命中）+ `codingplan_crypto_tests`（`en.rs:3325`、`zh_cn.rs:3137`）

**扩展与前端（公共协议消费方）**
- VS Code：`src/daemon/types.ts:226`、`src/daemon/client.ts:13,350,374-375`、`src/chat/provider.ts:11,679-680,2036,2117-2146,2657`、`webview-ui/src/state/types.ts:327`、`webview-ui/src/state/ChatProvider.tsx:274`、`webview-ui/src/components/WelcomeScreen.tsx:45-46,136`、`webview-ui/src/i18n.tsx:21,28,30,39,106,204,211,213,222,289`、`l10n/bundle.l10n.zh-cn.json:45`
- JetBrains：`daemon/RustCodeDaemonTypes.kt:118`、`daemon/RustCodeDaemonClient.kt:201-203`、`services/RustCodeProjectService.kt:546-556`、`ui/RustCodeChatPanel.kt:552,1890-1894`、`ui/GearMenuLabels.kt:14,33`、`ui/GearMenuLabelsTest.kt:19,43`、`resources/messages/RustCodeBundle{,_zh}.properties:8`
- WebUI：`src/i18n.ts:224,287,653,715`、`src/components/SettingsDialogs.tsx:340`、`src/components/Chat.tsx:2384-2409`

**脚本 / CI / 文档**
- `scripts/uninstall.sh:21`、`scripts/uninstall.ps1:24`（仅此两条；`.github/` 本轮 grep **0 命中**，故无 CI job 依赖 codingplan feature）
- `docs/` 18 个文件命中：`platform-neutralization.md`(7)、`REFACTOR_DESIGN_PHASE1.md`(13)、`phase1-refactor-design.md`(6)、`i18n-style.md`(5)、`i18n-field-mapping.md`(2)、`architecture.md`(2)、`config.example.toml`(1)、`mcp-rmcp-feasibility.md`(1)、`agent-api-rfc.md`(1)、`UPSTREAM_CREDITS.md`(1，合规归属，N-7 不动)、`docs/plans/*`(3)、`docs/archive/*`(22)
- `crates/rustcode-daemon/README.md`(6)、`crates/rustcode-clix/README.md`(1)、`crates/rustcode-coding/README.md`(2)

### 6.2 受影响的入口

| 入口 | 是否受影响 | 说明 |
|---|---|---|
| CLI（`rustcode`） | 是（A/B/D） | 子命令别名已删；剩余为 cfg 门控的 `run_codingplan_core` 与 completion 断言 |
| TUI（`rustcode-tuix`） | **是，且是非门控面** | `lib.rs:872`、`mod.rs:3965/12320`、`usage_monitor.rs:21`、`modals/usage.rs:6-7` —— 删 crate 必改 |
| daemon（`rustcode-daemon`） | 是 | `api_codingplan.rs` 三条路由、`runtime_host.rs`、`commands.rs`、`lib.rs` 路由表 |
| headless / background | 是（间接） | `cli/main.rs:1704-1718` 的 login→codingplan 链；`daemon/runtime_host.rs:87-89` provider 装配 |
| ACP | 弱 | `tuix/commands.rs:216` 的 `usage` 曾 `acp: true`，现已被 `command_visible` 在 neutral 构建过滤（`AGENTS.md:292-294`） |
| clix（`rustcodex`） | 弱 | `clix/main.rs:886` 仅调用 `is_codingplan_gateway`（L2，N-5 保留） |
| WebUI / 两个 IDE 扩展 | 是（仅 B/D 且 Q5 裁决清理时） | L3 路由与 DTO 消费方 |

---

## 7. 架构敏感面标记

| 敏感面 | 结论 | 依据与说明 |
|---|---|---|
| **持久化格式** | **true** | ① `$RUSTCODE_HOME/codingplan_sync.json`（`sync_marker.rs:22`，本 crate 独占读写）；② `config.toml` 的 `[providers.*]` 键前缀识别（`endpoints.rs:116` + `config/mod.rs:1233`）。**A/D 不改任何磁盘格式**；**B 会改变 `logical_accounts()` / `logical_models()` 的投影语义**（是否折叠为合成账号），属持久化语义变更，须经 Q2 裁决并补 AC-9 手工判定 |
| **公共协议** | **true** | daemon REST 三条路由（`/codingplan/setup` POST、`/codingplan/usage/summary` GET、`/codingplan/usage/daily` GET，`daemon/src/lib.rs:6099-6112`），且**已有外部消费方**：VS Code 与 JetBrains 扩展各自定义 DTO 并调用（`client.ts:374-375`、`RustCodeDaemonClient.kt:201-203`）。`AGENTS.md:329` 将 `/codingplan/setup` 客户端方法列为「托管发行版 API 对应保留开关，门控不删除」。B/D 删除路由 = 契约退役，须 Q5 裁决 |
| **安全边界** | **true** | 网关请求签名的**唯一**入口链：`coding::CodingPlanProviderAuthenticator`（`provider_factory.rs:45-64`）→ `capabilities::codingplan_sign::codingplan_request_signer`（`:86-95`）→ `auth::gateway_crypto::signer()`（`:53/86`）。默认构建走 `UnavailableSigner`（`:39-47`），**签名能力恒不可用**；`is_codingplan_gateway` 默认恒 false（`endpoints.rs:204-218`）。方案 A 删除的是闭源 overlay 接入点（`RealSigner`，`:57-93`），**不改变默认构建的出网行为**（仍为纯 bearer auth） |
| **运行时生命周期** | **true（按项目约束从严）** | `CodingRuntime` 本身**零符号耦合**（`coding/src/runtime.rs` 仅 `:7894-7897` 注释）。但 provider 装配链穿过它：`daemon/runtime_host.rs:87-89` `coding_provider_factory()`（**非门控**）→ `codingplan_provider_factory`（`provider_factory.rs:66-73`）→ `CodingPlanProviderAuthenticator`。项目约束要求「涉及 provider reload」即置 true，且 B/D 会改动该 trait 实现，故置 **true**；回归面为 provider reload / 热切换测试 |
| **跨 crate 依赖方向** | **true** | `tuix → rustcode-codingplan`（`tuix/Cargo.toml:27`，**非可选**）、`cli/daemon → rustcode-codingplan`（optional）、`capabilities → auth`、`coding → capabilities`、`daemon → coding`。删除会改变依赖图；依赖方向本身不反转（符合 `AGENTS.md:56` 只向下） |
| **G7/G8 门禁自身** | **受威胁** | `AGENTS.md:220-221` 的 G7 条文把 `is_codingplan_provider_name` / `LEGACY_CODINGPLAN_PREFIX` 写进豁免清单；删除这些字样会让门禁失效。`:226` 已就同类问题（G6 遥测）给过教训。**N-1 与 N-7 是为守住这条** |

---

## 8. 开放问题

> 全部需用户裁决。Q1 是方向性的，未获答复前不得进入 G2（先例见 `2026-09-02-cleanup-codingplan-legacy/STATUS.md:85`）。

### Q1 —— 选哪个方案？（阻塞项）

| 方案 | 删除内容 | 改动量 | 用户可见行为变化 | 需迁移/回滚 | 与 `AGENTS.md` 冲突 |
|---|---|---|---|---|---|
| **A 只删 crypto 桩** | `crates/rustcode-codingplan-crypto/` 全目录；`auth/Cargo.toml:21,25`；`cli/Cargo.toml:38`；`daemon/Cargo.toml:22`；`gateway_crypto.rs:57-93` RealSigner 与 `signer_available()==true` 分支；根 `Cargo.toml:8-12` 注释；`Cargo.lock:2859`；（Q3）`CpSign*` 等 6 个签名变体 + 2 个 `codingplan_crypto_tests` 模块 | 小（~29 行代码 + ~6 feature/依赖条目 + i18n 若干） | **无**（桩在默认构建下永不调用） | 纯 revert | **是**：`AGENTS.md:345` 明载 `Cp*` 签名族「按门控不删除铁律不动」 |
| **B 彻底移除** | A 全部 + `crates/rustcode-codingplan/` 全目录 + 所有 `codingplan`/`client` feature 与 optional dep + 所有 `#[cfg(feature="codingplan")]` 块 + `daemon/api_codingplan.rs` 与三条路由 + `capabilities/provider/codingplan_sign.rs` + `coding::CodingPlanProviderAuthenticator` + `auth::gateway_crypto`（或中性化）+ tuix 非门控消费点（`lib.rs:872`、`mod.rs:3965/12320`、`usage_monitor.rs:21`、`modals/usage.rs:6-7`）+ 全部 i18n `Cp*`/`StatusCp*` + （Q2）config 识别层 + （Q5）扩展/前端 | 大（~6000–7500 行删除，跨 8 个 crate + 2 个扩展 + webui + 脚本 + 18 份文档） | **有**：`RustCode*`/`AtomGit*` 条目不再折叠为合成账号、变为可编辑；`/usage` 与用量弹窗消失；`/codingplan/*` 永久 404 | 需 revert + `AGENTS.md` 回改 | **严重**：`AGENTS.md:289/498/509/537` 四次「codingplan gate 不删」+ `:220-221` G7 + `:329` 扩展保留 + `:345` i18n 保留 |
| **C 保留但中立化 / 确认现状即终态** | 零代码改动（仅登记与文档校准） | 0 | 无 | 无 | 无 |
| **D 仅移除网络侧** | `codingplan/src/{client,setup}.rs` + `client` feature + optional reqwest + `daemon/api_codingplan.rs` 与三条路由 + 三个驱动的 `codingplan` feature + 所有 `#[cfg(feature="codingplan")]` 块（`cli`、`tuix`、`daemon/runtime_host.rs:48-85`、`daemon/commands.rs`）；**保留** `types`/`usage`/`sync_marker`、`gateway_crypto`、`codingplan_sign.rs`、`CodingPlanProviderAuthenticator`、config 识别层、扩展 DTO | 中-大（~5200 行网络侧 + cfg 块与 feature；引自 `STATUS.md:38` 的 603+3911+677） | **无**（默认构建下这些代码本就不编译；删后路由仍 404） | 纯 revert | **是**：`AGENTS.md:289/498/509/537`「`#[cfg(feature="codingplan")]` 块…按 fork 铁律不动」 |

### Q2 —— 若选 B：是否连带删除 config 侧的 provider 前缀识别层？
- 候选 1（**推荐**）：**保留** `is_codingplan_provider_name` / `LEGACY_CODINGPLAN_PREFIX` / `codingplan_provider_prefix` / `codingplan_group_account_id` / `codingplan_builtin_effort_levels` / `is_codingplan_llm_gateway`。理由：它们识别的是**已写入用户磁盘的键**，与 codingplan crate 是否存在无关；`AGENTS.md:220-221` G7 明令勿删；删除会让 G7 门禁条文自身失效。代价：`crates/` 里会保留一批 `codingplan` 字样，AC-6 的白名单必须把它们列为**豁免**。
- 候选 2：删除并把前缀概念中性化为「legacy managed provider prefix」。代价：行为变化（E-1）、需补迁移与回归、需改 G7 门禁定义（N-7）。
- 候选 3：删 crate 但把识别函数**下沉到 `rustcode-config` 并改名**（去掉 codingplan 字样）。代价：改名即等于换前缀语义，仍需 G7 修订，且 `endpoints.rs:54` 的 env 名 `RUSTCODE_CODINGPLAN_PROVIDER_PREFIX` 是**对外 env 契约**，改名会破坏已配置的部署。

### Q3 —— 若选 A/B/D：i18n 的 `Cp*` 族与 `codingplan_crypto_tests` 是否一并删？
- 候选 1（**A 场景下推荐**）：随 A 删**签名相关** 6 个（`CpOfficialBuildRequired` `:430`、`CpAuthRequired` `:437`、`CpSignStaleClockSkew` `:441`、`CpSignReplayPersisted` `:445`、`CpSignVersionTooOld` `:449`、`CpUpgradeRequired` `:453`）与 `codingplan_crypto_tests` 两个模块，其余 `Cp*` 保留。理由：桩已不存在，这 6 个变体在中立构建不可达。
- 候选 2（**保守**）：全部保留。`AGENTS.md:345` 的理由是「闭源发行构建的 i18n 契约 + 有双语守护测试」；A 已经删了闭源接入点，该理由随之失效。
- 候选 3（**D 场景下推荐**）：删掉仅供 `setup.rs` 渲染器的那批（`messages.rs:320-423` 大部分），保留被非门控路径引用的 `LoginManagedUnavailable` / `UsageCodingPlanOnly` / `StatusCp*` / `CodingPlanSetupFailed`。
> 无论哪种，每个变体都必须三处同删（`messages.rs` 声明 + `en.rs` + `zh_cn.rs` arm），并由 AC-15 兜底。

### Q4 —— 是否同步修订 `AGENTS.md` 与 `docs/`？
- 候选 1（**推荐**）：若 Q1 = C，则**不动** `AGENTS.md`（现状与铁律一致）。若 Q1 = A/B/D，则**单列一个需二次确认的后续任务**修订 `AGENTS.md:289/498/509/537`（及 `:345`、`:329`，视 Q3/Q5），并同步 `docs/` 18 份命中文件里的**现状描述**（归档文件 `docs/archive/*` 22 处按前例不动，见 `STATUS.md:70` 用户裁决「过时文档归档到 `docs/archive/`」）。
- 候选 2：完全不动文档（会留下失实口径，违反 `AGENTS.md:5`「文件内容变更时同步更新，不得滞后」）。

### Q5 —— `extensions/` 与 `webui/` 的 codingplan 客户端是否清理？
- 候选 1（**推荐**）：**不动**。理由：`AGENTS.md:329` 明确「扩展侧 OAuth 传输与 `/codingplan/setup` 客户端方法/类型是托管发行版 API 的对应保留开关（门控不删除，同 `atomgit` cargo feature 铁律）」；且扩展已按 `managed_available` 门控 UI，中立构建本就不展示。AC-11 断言数值不减。
- 候选 2：随 B/D 一并清理（`WelcomeScreen.tsx:136` 同步按钮、`RustCodeChatPanel.kt:1890-1894` 齿轮项、`GearMenuLabels{,Test}`、`webui/i18n.ts` 4 键、`SettingsDialogs.tsx:340` 徽标）。代价：需重建两端前端 + `cargo clean -p rustcode-daemon`（`AGENTS.md:16`），且本机**无 JDK/gradle**，JetBrains 只能源码级验证（`AGENTS.md:318`）。

### Q6 —— `codingplan_sync.json` 的卸载清理项是否保留？
- 候选 1（**推荐**）：**保留** `cli/src/uninstall/paths.rs:36`、`:150`、`scripts/uninstall.sh:21`、`scripts/uninstall.ps1:24`。理由同 `AGENTS.md:87` 保留旧 `telemetry/` 目录——那是真实存在的历史文件，删了会漏清理。
- 候选 2：随 crate 一并删。风险：老用户升级后 `$RUSTCODE_HOME` 残留孤儿文件。

### Q7 —— 是否需要一次性的 config 迁移？
- 候选 1（**推荐**）：**不需要**。N-1 保留识别层后，`RustCode*` 条目本来就是合法的 `[providers.*]` 条目，能正常加载与选择（AC-9）。
- 候选 2：提供一次性重命名（`RustCode*` → 用户自定义名）。**不推荐**：会改写用户配置、引入迁移失败语义，违反 N-2 与 `AGENTS.md:173-175`（历史格式只作单向 importer，禁止恢复 legacy writer 或双向转换）。

### Q8 —— 「同源但命名不同的重复实现」是否顺带处理？
- 事实：`endpoints.rs:100-102`（`auth::gateway_crypto::is_codingplan_gateway`）与 `capabilities::provider::is_codingplan_gateway`（`codingplan_sign.rs:97` 重导出）是**同一个函数的两条可达路径**；`provider/mod.rs` 未对 `codingplan_sign` 加 cfg。
- 候选 1（**推荐**）：不在本 feature 处理，登记进 `03-impl/` 的 follow-up（前例：`STATUS.md:70` 用户裁决「重复渲染器维持现状、仅登记」）。
- 候选 2：收敛为单一路径。属架构改动，应交 `solution-architect` 单独评估。

---

## 9. 推荐与理由（供编排者转达，不替代用户裁决）

**推荐顺序：C（默认）> A（若确实要删点东西）>> D > B。**

理由：

1. **C 是零风险且与现状一致**。当前 L3/L4 在默认构建下**已经是死代码**，用户可见面（`/codingplan` 命令、`rustcode codingplan` 别名、托管登录入口）在 2026-09-01~09-02 的六轮中立化里**已经全部移除或门控完毕**。也就是说「codingplan 残留」在**用户可见行为上已接近归零**，剩下的都是 `#[cfg]` 之后的开关体——这正是 `AGENTS.md` 用四条铁律刻意保留的形态（与 `atomgit` feature 同类）。
2. **若用户确实要求删掉一些代码，A 是性价比最高、冲突最小的**：它删掉的是**唯一一处 `license = "Proprietary"` 的闭源痕迹**（`crates/rustcode-codingplan-crypto/Cargo.toml:5`）、一个 `unreachable!()` 占位（`:28`）与一条官方 overlay 接入链，零行为变化、回滚只需 revert。它只与 `AGENTS.md:345` 一条铁律冲突，且该条铁律的前提（闭源发行构建）在删除后自然失效。
3. **D 能删掉最多的真实代码（~5200 行网络侧），但需用户显式推翻四条「gate 不删」铁律**，且删完后 `rustcode-codingplan` 仍是 tuix 的非可选依赖、`gateway_crypto` 与 `CodingPlanProviderAuthenticator` 仍在——即「codingplan」这个名字并不会消失，只是变小。投入产出比不如 A。
4. **B 不推荐**：它是唯一会**改变持久化语义**的方案（`RustCode*` 条目不再折叠为合成账号、变为可编辑），同时与四条铁律 + G7 门禁 + 扩展保留裁决 + 上一轮用户 S2 裁决**全面冲突**，并需要改动 `CodingRuntime` 的 provider 装配路径（`daemon/runtime_host.rs:87-89`）。

**最大风险点（三项，按严重度）**：

1. **持久化语义变更（仅 B）**——`config/mod.rs:797-816` 的 `logical_accounts()` 折叠行为被移除，已写入 `config.toml` 的 `RustCode*` / `RustCode-anthropic` / `RustCode-ollama` 会从「一个合成账号」散成「多个独立 provider」，模型选择器的左列分组、右列账号标签、只读保护全部变化。这不是删除代码，是**改用户的数据视图**。
2. **feature 传递链断裂（B/D 高概率）**——`AGENTS.md:58` 已明确警告。只跑默认构建 `cargo build` **发现不了**；必须按 AC-4 跑七个 `cargo check` 组合。
3. **公共协议退役的对外影响（B/D）**——`POST /codingplan/setup` 有两个 IDE 扩展的真实客户端（VS Code `client.ts:374-375`、JetBrains `RustCodeDaemonClient.kt:201-203`），删路由后旧扩展会收到 404 并弹错；`AGENTS.md:329` 明确要求保留。

---

## 10. 修订记录

| 时间 | 修订 | 说明 |
|---|---|---|
| 2026-09-09 | 初版 | 只读勘查产出。无 shell 权限 → 未执行 `git`；无提问工具 → Q1 升级为阻塞项，`status: blocked` / `decision: block`。任务书中「CLI `rustcode codingplan` 子命令 / TUI `/codingplan` slash 命令仍存在」两处前提已按实测更正（§1.4）。 |
