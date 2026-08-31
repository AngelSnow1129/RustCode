# PHASE-1 系统重构与二次开发详细设计方案 (v2)

```text
[STATUS]   DRAFT v2 - 待 GATEWAY 审核；审核通过前禁止编写任何代码
[BASELINE] branch=dev  HEAD=ea8a0b9b  worktree=dirty（34 modified / 10 untracked）
[VERIFIED] 2026-08-30：cargo check --workspace --all-targets  PASS
[SUPERSEDES] docs/REFACTOR_DESIGN_PHASE1.md（基线 main/287bff70，重命名之前）
[SUPERSEDED] 本文件 v1（基线 6dbf57bb，同日 09:22 起草）
[SCOPE]    O1 品牌重命名 / O2 零遥测 / O3 LLM 解耦 / O4 合规
[STYLE]    全文 ASCII 标记；严禁 Unicode Emoji
```

### v2 相对 v1 的三处实质变化

| # | v1 的判断 | v2 的实测结论 |
|---|---|---|
| 1 | O3 缺 model mapping / 重试配置 / 统一错误类型 | `ModelMapping` 已落地并接入工厂；`retry_max_attempts` 已可配；`LlmError` 已落地。真正缺口收窄为**超时不可配**与**适配器构造点未走 `LlmError`** |
| 2 | O4 只有一套许可档案 | 新增 `docs/licenses/`（第二套），与 `docs/*.md` 四件套**内容重复**，且其中 `LICENSE-MIT-ORIGINAL.md` 含模板 MIT 正文（臆造风险） |
| 3 | 测试基线 4 个既有失败 | 现状 **6 个失败**：4 个既有 + **2 个重命名回归**（`rustcode-tuix` skills 菜单测试，工作区在途改动引入） |

---

## 0. 勘察结论摘要（先读这一节）

四个目标里，三个"看起来要做的大工程"在当前代码库中**已经完成或接近完成**。按 v1 的方案盲目重写会破坏已收口的架构。

| 目标 | 现状（2026-08-30 实测） | 真实剩余工作量 |
|---|---|---|
| O1 重命名 | `crates/` 内 `rustcode` 命中 **0**。包/二进制/配置目录/env 前缀全部落地 | 小：仅 `extensions/` 类名与 `site/` 品牌字符串收尾 |
| O2 零遥测 | 上报运行时已删；第三方埋点 SDK 依赖 **0 命中**；代码内仅剩注释 | 小：文档口径（`docs/telemetry.md`、`README.zh-CN.md`）与注释 |
| O3 LLM 解耦 | kernel 已有中立 `LlmProvider`（含**非流式 `chat()` 默认实现**）；capabilities 已有三套适配器 + SSE 解码器 + `LlmError` + `egress` 出站工厂 | 中：超时配置下沉、适配器错误构造迁移、装配层收口 |
| O4 合规 | 存在**两套**许可档案 | 小但必须先做：去重 + 删除模板 MIT 正文 |

### [WARN] 四条必须先确认的事实

1. **[WARN] 不存在第三方埋点 SDK。** `Cargo.toml` / `Cargo.lock` 中 `sentry` / `posthog` / `segment` / `mixpanel` / `amplitude` / `opentelemetry` / `prometheus` 全部 **0 命中**（唯一命中 `unicode-segmentation` 是分词库，属误报）。历史遥测是**自建**管线且运行时已删除。OBJECTIVE-2 的性质是"清死代码 + 修文档口径"，不是"拆 SDK"。
2. **[WARN] LLM 抽象层不是空白。** 在 kernel 之上新建第二套 `LlmClient` trait 会与 `AGENTS.md` 的"单一状态所有权 / 不得新增重叠抽象"直接冲突，并让三套已验证适配器变成死代码。解耦落点是**配置 + 装配 + 错误映射**。
3. **[WARN] 重命名不是一次 sed。** 会话磁盘目录、daemon 端口、User-Agent、发布产物前缀、升级 manifest 都是**外部契约**；`rustcode-tuix` 的两个测试已经因为"改了 fixture 名、没改过滤片段"而红掉——这是机械替换风险的实证。
4. **[WARN] 工作区有并发写入方。** 本轮勘察期间存在非本会话的落盘活动（`AGENTS.md` mtime 22:17、`docs/licenses/` 12:53）。**任何批量改动前必须重新读取目标文件**，且每个 Agent 交付后由主 Agent 立即 `cargo check --workspace --all-targets` 验收。

### [ERROR] 顺带发现的缺陷（不计入四目标，但必须在 PHASE-2 修掉）

| # | 位置 | 问题 |
|---|---|---|
| X1 | `crates/rustcode-capabilities/src/provider/error.rs:13` | 注释中英混排：`"... ) is承重 and stays exactly as-is."` |
| X2 | `AGENTS.md` | 写 `bind_session`，代码里是 `bind_session_id`（`kernel/src/provider.rs:186`）——文档漂移 |
| X3 | `docs/licenses/LICENSE-MIT-ORIGINAL.md` | 声明是占位文件，却在 `---` 之后附了一份 `Copyright (c) [YEAR] [UPSTREAM_AUTHOR]` 的**模板 MIT 正文**；违反"绝不臆造许可文本" |
| X4 | `crates/rustcode-tuix/src/event_loop/mod.rs:6850,7010` | 重命名回归：`/skills atom smoke` 过滤片段未随 fixture 改名，2 个测试失败 |

---

## 1. 系统勘察：结构、依赖与网络交互入口

### 1.1 crate 地图与依赖方向

```text
L3  drivers   rustcode-cli(pkg `rustcode`)  rustcode-tuix  rustcode-daemon
              rustcode-clix(bin `rustcodex`)  ACP(crates/rustcode-cli/src/acp/)
                    |             |                |
L2  specialize      |             +----> rustcode-coding (CodingRuntime)
                    |                          |     `-- rustcode-review
L1  capabilities    rustcode-capabilities <----+
L0  neutral         rustcode-kernel <----------+
leaf                rustcode-config / rustcode-auth / rustcode-updater
                    rustcode-codingplan / rustcode-codingplan-crypto
```

- 工作区 `members = ["crates/*"]`（14 个目录），`default-members` 为 cli / daemon / tuix。
- `rustcode-codingplan-crypto` 是闭源签名占位桩，默认成员外，官方构建用 `--features rustcode/codingplan-crypto`。
- 依赖只向下；`rustcode-capabilities` 禁止反向依赖 coding / driver / 前端（编译期强制）。

### 1.2 出站网络交互入口清单（实测）

| 调用点 | 客户端类型 | 是否走 `egress` 唯一工厂 | 备注 |
|---|---|---|---|
| `capabilities/src/egress/client.rs` | async | **[是] 工厂本体** | `build_http_client` / `build_pinned_http_client`，承载 #514 信任根分层 + 代理 + 超时 + UA + pool-idle |
| `capabilities/src/tools/web_fetch.rs` | async | [是] | `build_pinned_http_client`，保留 DNS pin 与手动重定向（SSRF 语义不变） |
| `capabilities/src/tools/web_search.rs` | async | [是] | 新增 connect 超时 |
| `capabilities/src/atomgit/client.rs` | async | [是] | UA 与 30s 预算不变 |
| `capabilities/src/mcp/transport_http.rs` | async | [是] | `try_new() -> Result`，已消除 `unwrap_or_else(\|_\| Client::new())` 静默降级 |
| `capabilities/src/provider/openai_compat.rs:342` | async | [否] | 自建；`egress` 的信任根逻辑正是从此处泛化而来 |
| `capabilities/src/provider/anthropic.rs:150` | async | [否] | 自建 UA/代理 |
| `capabilities/src/provider/ollama.rs:110` | async | [否] | 自建 |
| `rustcode-auth/src/oauth.rs:116,120` | **blocking** | [否] | 有自己的 `apply_blocking_proxy_policy`；**已确认**在独立 `std::thread` 上调用（`tuix/src/event_loop/oauth_poll.rs:67`），未污染 tokio worker |
| `rustcode-codingplan/src/client.rs:103` | **blocking** | [否] | 含 TLS 1.2 强制降级逻辑，同步登录流程 |
| `rustcode-tuix/src/version_check.rs:77` | async | [否] | 自建 builder + `apply_async_proxy_policy`；是**升级检查**不是埋点 |
| `rustcode-updater/src/lib.rs` | async | [否] | 30s / 600s 超时是刻意设计 |

**[CHECK] 结论**：异步出站面的 7 个入口中 4 个已收敛；剩余 3 个 LLM 适配器是**已决策不动**的已验证热路径。同步面（auth / codingplan）与 `version_check` / `updater` 保持现状。全仓 `reqwest::Client::new()` 仅剩测试代码与注释。

### 1.3 依赖侧核验（TIP-2）

```text
grep -rniE "sentry|posthog|segment|amplitude|mixpanel|opentelemetry|prometheus|statsd" Cargo.toml Cargo.lock
  -> 0 命中（唯一命中 crates/rustcode-tuix/Cargo.toml:41 unicode-segmentation = "1"，误报）

reqwest = { version = "0.12", default-features = false,
            features = ["stream", "json", "rustls-tls"] }      # 已最小化
tokio   = { version = "1", features = ["rt", "macros", "sync", "time"] }
```

无需再动。`rustls-tls`（webpki base）是**刻意**选择：OS 根 + `SSL_CERT_FILE` 在其上分层，构建永不硬失败（issue #514）。

---

## 2. OBJECTIVE-1：全局重命名

### 2.1 命名现状：D1 已锁定 `rustcode`，且已落地

**[CORRECTION] 本方案不提议新品牌名。** 仓内 `docs/REFACTOR_DESIGN_PHASE1.md` §2.0 的决策 D1 把产品身份锁定为 `rustcode`，并已在 `6dbf57bb` / `9974c6b9` / `ea8a0b9b` 三次提交中落地。OBJECTIVE-1 在主线上是 **[DONE]**。

本节剩余价值只有两种：**收尾**（2.3 残留清单）与**备用**（2.2 是纯声明式映射表，日后若真要换名，替换"目标"列即可，执行步骤不变）。

### 2.2 重命名映射表（当前 -> 备用示例；示例列未作为提案提交）

| 维度 | 当前（D1 已锁定） | 备用示例（未提案） |
|---|---|---|
| crate 前缀 | `rustcode-*` (14 个) | 例 `codeforge-*` |
| CLI 包名 / 二进制 | package `rustcode` / bin `rustcode` | 例 `codeforge` |
| daemon 二进制 | `rustcode-daemon` | 例 `codeforge-daemon` |
| TUI crate | `rustcode-tuix` | 例 `codeforgex` |
| clix 包 + 二进制 | `rustcodex` | 例 `codeforgex` |
| 配置目录 | `~/.rustcode` | 例 `~/.codeforge` |
| home env | `RUSTCODE_HOME` | 例 `CODEFORGE_HOME` |
| env 前缀 | `RUSTCODE_*` | 例 `CODEFORGE_*` |
| 项目指令文件 | `RUSTCODE.md` / `.rustcode.md` | 例 `CODEFORGE.md` / `.codeforge.md` |

事实源集中在两个文件，改这两处即带动大部分：

- `crates/rustcode-config/src/distribution.rs`：`HOME_ENV` / `HOME_DIR_NAME` / 端口 `13456,13457,13458` / `PROCESS_NAMES` / `WINDOWS_INSTALL_DIR` / `RELEASE_ASSET_PREFIX` / `UPDATE_TEMP_PREFIX` / `DEFAULT_USER_AGENT`。
- `crates/rustcode-config/src/endpoints.rs`：11 个 `RUSTCODE_*` 环境变量名 + 4 个托管端点。

### 2.3 残留清单（2026-08-30 实测计数）

| 区域 | 文件数 / 命中数 | 性质 | 处置建议 |
|---|---|---|---|
| `crates/` | 0 / 0 | 已干净 | 无需处理 |
| `extensions/jetbrains` | — / 645 | Kotlin **类名** `RustCode*` + 品牌字符串；包路径已改 `com.rustcode.jetbrains` | 类名级 rename（需 IDE 符号重构，本机无法编译验证） |
| `extensions/vscode` | — / 200 | 同上（`RustCode-Client` UA 头、`RustCode` 进程名）；command ID 已改 `rustcode.*` | 同上 |
| `site/` | 51 / 1493 | 纯品牌字符串 | sed + 人工抽查首页 |
| `docs/` | 150 / 3111 | 绝大多数在 `docs/plans/**` 等**历史记录**中 | **不动**；只改仍在描述当前架构的条目 |
| `packages/` | 5 / 80 | npm 包 `rustcode` / bin `rustcode.js` / homebrew 脚本 | 发布通道改名，需与 release 矩阵同步 |
| `evals/` | 4 / 28 | 评测脚本中的品牌词 | 顺带 |
| `scripts/` | 1 / 2 | `dev-env-quickstart.sh` | 顺带 |
| `examples/` | 1 / 1 | `hooks.toml` 注释 | 顺带 |

### 2.4 机械替换 vs 高风险项

**[SUCCESS] 可全局 sed（低风险）**：crate 名与目录、`use rustcode_*` 导入路径、环境变量字符串常量、UA、进程名、安装目录名、文档与脚本品牌词。

**[ERROR] 高风险，必须显式决策**：

| 项 | 风险 | 建议 |
|---|---|---|
| 会话磁盘目录 `~/.rustcode/sessions/<hash>/` | 用户历史会话不可丢 | 首次启动做一次性目录迁移（旧目录存在且新目录不存在时 rename），失败则保留旧目录并告警，**不静默 fresh** |
| daemon / webui / app 端口 `13456/13457/13458` | IDE 扩展与手机端按端口连 | **保持端口不变** |
| 发布产物前缀 + `latest.json` + upgrade manifest | 自更新器按名下载 | 同步改 `endpoints.rs` 与 `scripts/release*.sh`；老版本升级需"桥接版本"或明示手动重装 |
| `SessionMode` 的 serde wire 值 | 落盘值，老 `extensions/` 客户端仍在传 | 保持稳定；daemon 对未知参数静默忽略 |
| npm / homebrew 包名 | 已发布的分发通道 | 单独批次，与 release 矩阵联动 |

### 2.5 执行顺序（AGENT-A）

**[CHECK] 在 `rustcode` 身份下，A3-A7 已执行完毕。** 只有 GATEWAY 决定"再做一次品牌切换"才需重跑；否则 AGENT-A 只做 A1/A2（合规，见第 5 节）与 A8/A9（收尾）。

```text
STEP A1  冻结基线：记录 branch/commit，确认不覆盖用户改动（工作区 dirty，禁 git 写操作）
STEP A2  合规先行：LICENSE / 许可档案去重 / THIRD_PARTY_NOTICES / UPSTREAM_CREDITS
STEP A3  Rust 重命名（已完成）
STEP A4  常量层：distribution.rs + endpoints.rs（已完成）
STEP A5  二进制名 / 进程名 / UA（已完成）
STEP A6  脚本与 CI（已完成）
STEP A7  文档（已完成；docs/plans/** 刻意保留历史）
STEP A8  extensions/ 类名与 site/ 品牌字符串收尾（本轮剩余）
STEP A9  packages/ 发布通道改名（独立批次）
```

---

## 3. OBJECTIVE-2：零遥测

### 3.1 已完成（实测证据）

| 项 | 状态 |
|---|---|
| `rustcode-telemetry` crate | 已删除，工作区无目录无依赖 |
| `config/src/telemetry_legacy.rs` | 已删除；`SessionMode` / `RepoOrigin` / `detect_repo_origin` 迁至 `config/src/session_mode.rs`；`CliOverride` 已删 |
| `clix/src/tel.rs` | 已删除（空桩、`meter_provider`、`.clix_telemetry_notice`）；`build_review_provider` 迁至 `clix/src/code.rs` |
| `cli/src/telemetry_cmd.rs` + `tests/telemetry_cmd.rs` | 已删除 |
| `telemetry` 子命令与 `[telemetry]` 配置段 | 已从 schema 移除；遗留段**静默忽略**（回归测试 `config/mod.rs:4310 legacy_telemetry_section_tests`） |
| daemon `--no-telemetry` | 降级为被忽略的 no-op + stderr 告警（老 IDE 扩展仍会传） |
| 崩溃处理 | 仅 stderr，无离箱上报 |
| 第三方埋点 SDK 依赖 | 0 命中 |

### 3.2 剩余清单（AGENT-B）

| # | 文件 | 内容 | 动作 |
|---|---|---|---|
| B10 | `docs/telemetry.md` | 整篇仍在描述 "RustCode ships anonymous usage telemetry by default"、队列目录、`rustcode telemetry disable` 子命令 | **删除**，或改写为"本项目零遥测"的单页声明 |
| B10b | `README.zh-CN.md:151` | "匿名遥测（默认开启，可关闭）" —— 与零遥测事实**直接矛盾** | 改写为零遥测声明（注意该行含 Emoji，一并清除） |
| B10c | `site/docs/en/headless-daemon.html`、`site/docs/zh/headless-daemon.html` | 旧遥测口径 + `--no-telemetry` 说明 | 改写 |
| B11 | `kernel/src/{hook.rs:10,227,295,320; message.rs:385; event.rs:365; agent.rs:1739,2140; conformance/provider.rs:29}`、`kernel/tests/{turn_complete.rs:4, hook_a2_surface.rs:14}`、`tuix/src/event_loop/{commands.rs:4056,8887,9017; mod.rs:9469}`、`cli/src/main.rs:{835,1136,1361,1363,2020,2118,4009}` | 仅**注释**里出现 telemetry 字样（多为"telemetry/datalog 的 home seam"这类词义歧义） | 只改注释为中性描述，**不动语义** |
| B12 | `capabilities/Cargo.toml` | 注释 "config→telemetry (and its reqwest/rustls/flate2 stack)" | 改注释 |
| X1 | `capabilities/src/provider/error.rs:13` | 中英混排 `is承重 and` | 改注释 |

### 3.3 必须保留（勿误删）

- **[ERROR] `SessionMode` 是活的功能**：daemon 用它区分 IDE / webui / jetbrains / vscode 客户端（`daemon/src/client_mode.rs` + `--client` + `x-rustcode-client` header），是本地分支逻辑，不是上报。
- `RepoOrigin` / `detect_repo_origin`：纯字符串解析，无网络。
- `capabilities/src/datalog.rs`：turn datalog 是**本地文件**结构化日志，非上报。
- `tuix/src/version_check.rs`：升级检查（网络出站），改名 `update_check` 并给显式开关，不删除。

### 3.4 门禁（AGENT-D 复核）

```text
grep -rniE "sentry|posthog|segment|amplitude|mixpanel|google-analytics|opentelemetry" \
     --include=*.rs --include=*.toml crates/   必须 0 命中
grep -rn "track(\|track_event\|identify(" --include=*.rs crates/   必须 0 命中
```

---

## 4. OBJECTIVE-3：LLM Provider 解耦

### 4.1 现状盘点（2026-08-30 实测）

**L0 中立契约（`crates/rustcode-kernel`）**：

```text
provider.rs :168  pub trait LlmProvider: Send + Sync
                     fn model_name(&self) -> &str
                     fn context_window(&self) -> u32            { 0 }
                     fn bind_session_id(&self, _: &str)         { }   // 会话亲和，OneLock
                     async fn chat_stream(&self, messages, tools, options)
                         -> Result<BoxStream<'static, StreamEvent>, ProviderError>
provider.rs :225  async fn chat(&self, ...) -> Result<ChatResponse, ProviderError>
                     // 默认实现：折叠本适配器的 chat_stream，非流式开箱即用
stream.rs   :36   pub struct ProviderError { retryable, message, http_status, code, retry_after_secs }
                     + is_context_overflow()
stream.rs   :93   pub enum StreamEvent { TextDelta, Reasoning, ReasoningSignature,
                     ToolCall, ToolCallDelta, Done, Error, Malformed, Usage, ... }
provider.rs :150  pub struct ChatResponse { text, reasoning, tool_calls, usage, finish_reason }
```

**[SUCCESS] 非流式已被 trait 原生覆盖**（GLOBAL CONSTRAINT #4 的 SSE + 非流式双支持）：`chat()` 有默认实现，三套适配器与所有测试替身**零改动**即获得正确的非流式语义；OpenAI 兼容适配器可用 `stream:false` 覆写以省掉 SSE 握手。

**L1 适配器（`crates/rustcode-capabilities/src/provider/`）**：

| 文件 | 协议 | SSE 处理 |
|---|---|---|
| `openai_compat.rs` | `/v1/chat/completions` | `delta.content` / `delta.tool_calls` / `[DONE]` |
| `anthropic.rs` | `/v1/messages` | `message_start` / `content_block_start` / `content_block_delta`（`text_delta` / `input_json_delta` / `thinking_delta` / `signature_delta`）/ `message_delta` / `message_stop` / `error` / `ping` |
| `ollama.rs` | `/api/chat` | NDJSON 行流 |

**已完成增量（v1 之后落地）**：

| 能力 | 位置 | 状态 |
|---|---|---|
| 模型别名 | `config/provider.rs:483 ModelMapping` + `provider_factory.rs:112 cfg.model_mapping.resolve(&cfg.model)` | [DONE] 三协议统一在工厂解析，适配器看不到别名 |
| 重试次数可配 | `ProviderConfig.retry_max_attempts` -> `retry_policy_for()` -> `ac.retry` | [DONE] |
| 强类型错误 | `provider/error.rs::LlmError`（thiserror，`retryable()` 单点判定，`from_provider()` 分类器，可无损转回 `ProviderError`） | [DONE] 分类层；适配器构造点迁移未做 |
| 出站 HTTP 工厂 | `egress/client.rs` + `egress/config.rs::SecretString`（zeroize，`Debug` 打印 `***`，不实现 `Serialize`） | [DONE] 非 LLM 面 |
| 协议族判定 | `config/provider.rs::ProviderKind::from_type()`（anthropic / ollama / openai-compatible / 未知回落） | [DONE] |

### 4.2 设计原则：不新建第二套 trait

**[ERROR] 禁止在 kernel 之上再叠一个 `LlmClient` trait。** 理由：
1. 与 `AGENTS.md`"单一状态所有权 / 不得新增重叠抽象"冲突；
2. `LlmProvider` 已是 driver 无关的中立 seam，叠加会让三套适配器（尤其 Anthropic 思考块签名回传）变成死代码或双写；
3. 装配入口 `CodingProviderFactory`（trait）+ `DefaultCodingProviderFactory` 已经存在，ACP / daemon / clix 都通过它注入。

**本次解耦落点 = 配置下沉 + 装配收口 + 错误映射迁移。**

### 4.3 缺口清单（本次真正要做的）

| ID | 缺口 | 证据 | 影响 |
|---|---|---|---|
| G1 | **`[providers.*]` 段无法配置任何超时** | `config/src/config/provider.rs` 全文 `grep timeout` = 0 命中 | 用户遇到慢网关只能改代码；企业长思考模型易被默认 idle 超时掐断 |
| G2 | 现有超时来自 **coding 层**而非配置 | `coding/src/config.rs:48 stream_timeout`（`default_stream_timeout()`）-> `provider_factory.rs:118/136/161` 三处赋给 `idle_timeout` | 值与 provider 段脱钩，无法按 provider 差异化 |
| G3 | `connect_timeout` 仅 Ollama 有 | `ollama.rs:57 pub connect_timeout: Duration` | OpenAI/Anthropic 侧无首字节超时配置 |
| G4 | `request_timeout` 是 **kernel round-trip**（含审批），不是 HTTP 超时 | `coding/src/config.rs:56` doc；`assemble.rs:186`、`parts.rs:1760` 传给 `AgentBuilder` | 命名易误读；应在文档与配置名上区分 |
| G5 | 适配器构造点仍原地裸建 `ProviderError`，未走 `LlmError` | `anthropic.rs` / `ollama.rs` / `openai_compat.rs` 内联构造 | 分类逻辑分散；`error.rs` 模块头已登记为渐进迁移项 |
| G6 | 无显式 `protocol` 选择键 | 只能靠 `provider_type` 字符串推断 | 自建网关 `type = "openai"` 但走 Anthropic 协议时无法表达 |

**[CHECK] 不做的事**：不新建 trait；不改 `friendly_http_error` 的承重文案（401/402 标题、CodingPlan 403 提示、kernel 限流路径要剥的 `HTTP 429: ` 前缀）；不合并三套适配器的 HTTP 客户端（已决策：它们是已验证热路径，为统一而改风险高于收益）。

### 4.4 配置模型：把超时下沉到 `[providers.*]`

复用既有资产，不另造一套：`ModelMapping` 用 config crate 的，`SecretString` 用 `egress::config` 的，协议族用 `ProviderKind`。

```toml
# ~/.rustcode/config.toml
[providers."my-gateway"]
type         = "openai-compatible"      # openai | anthropic | ollama | 任意兼容串
base_url     = "https://llm.internal/v1"
api_key      = "$MY_GATEWAY_KEY"        # 支持 $VAR / ${VAR} / ${VAR:-default}
model        = "fast"                   # 逻辑名（用户可见 / `/model` 切换用）

[providers."my-gateway".model_mapping]  # 逻辑名 -> 线上名；未命中原样透传
fast         = "gpt-4o-mini"
smart        = "gpt-4o"

[providers."my-gateway".timeout]        # 新增段；全部可选，缺省回落到适配器默认
connect      = 10      # 秒，首字节
request      = 600     # 秒，整轮（HTTP 层，区别于 kernel 的 round-trip）
idle         = 120     # 秒，流内字节空闲（卡流保护）

[providers."my-gateway".retry]          # 新增段；可选
max_attempts        = 3
initial_backoff_ms  = 500
max_backoff_ms      = 30000
respect_retry_after = true              # 遵循服务端 Retry-After (429/503)

[providers."my-gateway".extra_headers]  # 自建网关租户 / 鉴权头
X-Tenant-Id = "team-a"
```

```rust
// crates/rustcode-config/src/config/provider.rs —— 新增两个可选段
use serde::{Deserialize, Serialize};

/// HTTP 层超时。三档语义互斥，缺一档即回落适配器默认，不硬编码。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimeoutConfig {
    /// 首字节超时（connect + TLS + 首个响应字节之前的等待）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connect: Option<u64>,
    /// 整轮请求超时（HTTP 层）。与 kernel 的 request/respond round-trip 超时
    /// 是两回事 —— 后者管审批等人，前者管对端不回。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request: Option<u64>,
    /// 流内字节空闲超时（卡流保护）。对应各适配器的 `idle_timeout`。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idle: Option<u64>,
}

impl TimeoutConfig {
    pub fn is_empty(&self) -> bool {
        self.connect.is_none() && self.request.is_none() && self.idle.is_none()
    }
}

/// 重试策略。`respect_retry_after` 只影响 429/503 的等待时长，不把 4xx 变成可重试。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
    pub respect_retry_after: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff_ms: 500,
            max_backoff_ms: 30_000,
            respect_retry_after: true,
        }
    }
}
```

`ProviderConfig` 追加两个字段，均 `#[serde(default, skip_serializing_if = ...)]`，**向后兼容**（旧 config.toml 无需改动即可加载）：

```rust
    #[serde(default, skip_serializing_if = "TimeoutConfig::is_empty")]
    pub timeout: TimeoutConfig,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry: Option<RetryConfig>,
```

### 4.5 装配层：三处构造点收敛为一次映射

现状是三个调用点各写一遍字段搬运：`coding/src/provider_factory.rs`（主链路）、`coding/src/subagent_tiers.rs`（子 agent 分层）、`review/src/assemble.rs`（review agent）。它们重复了同一套 `cfg -> XxxConfig` 搬运逻辑，是 G1-G3 会散落三处的根因。

```rust
// crates/rustcode-coding/src/provider_factory.rs —— 收敛后的形状（草案）
fn apply_common_timeouts(kind: ProviderKind, cfg: &CodingAgentConfig, t: &TimeoutConfig) {
    // 单一映射点：TimeoutConfig -> 各适配器字段。缺省回落，不硬编码。
    //   idle     -> AnthropicConfig::idle_timeout / OllamaConfig::idle_timeout
    //               / OpenAiCompatConfig::idle_timeout
    //   connect  -> OllamaConfig::connect_timeout；其余协议 push 到 egress spec
    //   request  -> 交给 egress HttpClientSpec::request_timeout（HTTP 层）
}
```

**[CHECK] 落地顺序必须是"先加字段、后接工厂、最后扩配置"**：`TimeoutConfig` 是纯新增的 `Default` 结构，先合入不会改变任何现有行为（GATEWAY 更倾向于零风险批次时，可只做 4.4 的配置下沉）。

### 4.6 错误映射迁移（G5）

`LlmError` 已就位，缺的是把适配器里裸建的 `ProviderError` 迁移过去。迁移**必须**遵守：

- 保留 `http_status` / `code` / `retry_after_secs` 三个承重字段，转回 `ProviderError` 时无损；
- **不改动 `friendly_http_error` 的既有文案**（401/402 标题、CodingPlan 403 提示、`HTTP 429: ` 前缀是 kernel 限流路径的解析依赖）；
- 按适配器逐个迁移，每迁移一个跑一次 `cargo test -p rustcode-capabilities`，不允许三个一起改；
- 上下文溢出必须走 `LlmError::InvalidRequest` + `ProviderError::is_context_overflow()` 的既有判定，**不得**改成"可重试"。

```rust
// 迁移后的构造点形状（草案）—— 每个失败点都是"分类 + 转换"两行
let classified = LlmError::from_provider(&raw, Some(status));
if classified.retryable() { /* 交给 RetryPolicy */ }
return Err(classified.into_provider_error());   // 字段无损
```

### 4.7 两套协议差异对照（TIP-3，现有实现的回归防线）

| 维度 | OpenAI `/v1/chat/completions` | Anthropic `/v1/messages` |
|---|---|---|
| 鉴权头 | `Authorization: Bearer <key>` | `x-api-key: <key>` + `anthropic-version: 2023-06-01` |
| system 位置 | 消息数组中 `role:"system"`（多段需合并） | 顶层 `system` 字段 |
| 文本增量 | `choices[0].delta.content` | `content_block_delta.delta.text_delta.text` |
| 工具调用 | `delta.tool_calls[].function.arguments` 累积 | `content_block_start`（`tool_use`）+ `content_block_delta.input_json_delta.partial_json` 累积 |
| 思考 | `reasoning_content`（部分网关） | `thinking_delta` + `signature_delta`（签名块**原样回传**） |
| 结束 | `[DONE]` 哨兵 + `finish_reason` | `message_stop` + `message_delta.delta.stop_reason` |
| 心跳 | 无 | `ping` 事件，需忽略 |
| 错误 | `{"error":{"message","code"}}` + HTTP 状态 | SSE `event: error` + HTTP 状态 |
| 非流式 | `stream:false`（适配器可覆写 `chat()`） | 同 `chat()` 默认折叠 |

统一到 `StreamEvent`：`text_delta` -> `TextDelta`，`thinking_delta` -> `Reasoning`，`signature_delta` -> `ReasoningSignature`，`tool_use` 组装完成 -> `ToolCall`，`message_delta/message_stop` -> `Done { finish_reason, usage }`。

### 4.8 落地顺序（AGENT-C）

```text
STEP C1  修 X1（error.rs 中英混排注释）                       —— 零风险，先清场
STEP C2  config: 新增 TimeoutConfig / RetryConfig + 反序列化单测（旧配置必须仍能加载）
STEP C3  config: ProviderConfig 追加两字段 + 构造点补默认值（10 处字面量，脚本批量）
STEP C4  coding: provider_factory.rs 抽 apply_common_timeouts，三协议统一映射
STEP C5  capabilities: 适配器错误构造逐适配器迁移到 LlmError（一个一测）
STEP C6  review/assemble.rs 与 subagent_tiers.rs 复用 C4 的映射，消除三处重复搬运
STEP C7  文档：docs/config.example.toml 补 timeout / retry / model_mapping 示例
```

---

## 5. OBJECTIVE-4：License 与合规

### 5.1 现状（实测）

存在**两套**许可档案，且根 `LICENSE` 指向后者：

| 位置 | 文件 | 判定 |
|---|---|---|
| `LICENSE` | 本项目 MIT，双版权行 `Copyright (c) 2026 Yubang Xu` + `Copyright (c) 2026 The rustcode authors (fork of rustcode)`；正文指向 `docs/licenses/` | [OK] 但需随去重结果定稿 |
| `docs/ORIGINAL_LICENSE.md` | 上游 rustcode 的**诚实占位**（无许可正文） | [OK] 语义正确 |
| `docs/UPSTREAM_RUSTCODE_LICENSE.md` | 前身 `SecLab/RustCode`（Yubang Xu）MIT 全文 | [OK] |
| `docs/THIRD_PARTY_NOTICES.md` / `docs/UPSTREAM_CREDITS.md` | 存在 | 需与 `docs/licenses/` 内同名文件去重 |
| `docs/licenses/README.md` | 许可链图：rustcode -> RustCode -> rustcode | [OK] 内容正确 |
| `docs/licenses/LICENSE-MIT-FORK.md` | 本 fork MIT | 与根 `LICENSE` 重复 |
| `docs/licenses/LICENSE-MIT-PREDECESSOR.md` | 前身 MIT | 与 `docs/UPSTREAM_RUSTCODE_LICENSE.md` 重复 |
| `docs/licenses/LICENSE-MIT-ORIGINAL.md` | 声明占位，**但附了模板 MIT 正文** | **[ERROR] 见 5.2** |
| `docs/licenses/THIRD-PARTY-NOTICES.md` / `UPSTREAM-CREDITS.md` | 与 docs 顶层同名文件重复 | 需去重 |

### 5.2 [ERROR] 必须先修的合规缺陷

`docs/licenses/LICENSE-MIT-ORIGINAL.md` 第 26-50 行在 `---` 之后附了一份以
`Copyright (c) [YEAR] [UPSTREAM_AUTHOR]` 为版权行的 MIT 模板正文，文件标题却是
`# LICENSE - MIT (Original Upstream: rustcode)`。

风险：自动化合规扫描与人工阅读都可能把它当成上游许可原文；`[YEAR] [UPSTREAM_AUTHOR]`
是从模板**重构**出来的，而上游文本从未独立取得。这违反 `AGENTS.md` 的
"绝不臆造许可文本"。

**处置**：删除第 26-50 行的模板正文，只保留占位说明与"如何补齐"的步骤（与
`docs/ORIGINAL_LICENSE.md` 的处理保持一致 —— 该文件正确地**不含**任何许可正文）。

### 5.3 目标布局（去重后，单一事实源）

```text
LICENSE                              本 fork 的 MIT（双版权行 + fork 声明）—— 唯一生效许可
docs/licenses/README.md              许可链图 + 索引（唯一索引页）
docs/licenses/LICENSE-MIT-PREDECESSOR.md   前身 SecLab/RustCode 的 MIT 全文（逐字）
docs/licenses/LICENSE-MIT-ORIGINAL.md      上游 rustcode 的 MIT —— 占位，无正文，待取得后逐字补入
docs/licenses/THIRD-PARTY-NOTICES.md       依赖声明（cargo license 从 Cargo.lock 生成）+ 上游溯源
docs/licenses/UPSTREAM-CREDITS.md          上游致谢
```

`docs/ORIGINAL_LICENSE.md`、`docs/UPSTREAM_RUSTCODE_LICENSE.md`、
`docs/THIRD_PARTY_NOTICES.md`、`docs/UPSTREAM_CREDITS.md` 四个顶层文件改为
**指向 `docs/licenses/` 的索引**（或删除，由 GATEWAY 定）。

### 5.4 执行要点

1. **[ERROR] 不得删除或改写任何既有版权行**，包括 `Copyright (c) 2026 Yubang Xu`。
2. 上游 rustcode 真实许可文本**未取得**之前，只能保留占位，**绝不臆造**。
3. 新编写模块（如本轮新增的配置结构）**不逐文件加版权头**，避免与既有风格割裂；只在 `THIRD-PARTY-NOTICES.md` 统一声明。
4. 去重时保留 `docs/licenses/README.md` 的许可链图 —— 它是唯一把三段关系讲清的地方。

---

## 6. PHASE-2 子 Agent 并行执行计划

### 6.1 任务切分

| Agent | 范围 | 交付物 |
|---|---|---|
| **AGENT-A** | O4 合规去重 + O1 收尾 | 5.3 的布局；`extensions/` 类名 rename 规格；`site/` 品牌字符串替换 |
| **AGENT-B** | O2 文档与注释 | B10 / B10b / B10c / B11 / B12 / X1 |
| **AGENT-C** | O3 配置下沉 + 错误迁移 | C1-C7 |
| **AGENT-D** | 门禁 / 测试 / 收口 | 基线复核、冲突修复、Mock 一致性测试、Changelog |
| **AGENT-E**（可选） | egress 收敛收尾 | `version_check.rs` 迁到 `egress`；`updater` 评估（**不做**） |

### 6.2 批次与冲突消解

**[WARN] 并行 Task 在本运行时不可用**（返回 "No result found"）——必须串行派发。
**[WARN] 存在并发写入方**：每个 Agent 交付后主 Agent 立即跑 `cargo check --workspace --all-targets`；改动共享文件前重读。

```text
BATCH-0（串行）  修 X4（tuix skills 菜单两处过滤片段）—— 先把红的测试修绿，再谈新增
BATCH-1（串行）  O4：许可档案去重 + 删模板 MIT 正文 + 根 LICENSE 定稿   -> AGENT-A
BATCH-2（并行）  AGENT-B: B10/B10b/B10c/B11/B12/X1  ‖  AGENT-C: C1-C3  -> 无重叠文件
BATCH-3（串行）  AGENT-C: C4-C7（provider_factory / 适配器 / review / 配置模板）
BATCH-4（并行）  AGENT-A: extensions+site 规格（仅文档产出，不落盘代码）
BATCH-5（串行）  AGENT-D: 门禁、冲突修复、Mock 测试、Changelog、README/配置模板
```

重叠文件矩阵：

| 文件 | A | B | C | D |
|---|---|---|---|---|
| `config/src/config/provider.rs` | | | [X] | |
| `coding/src/provider_factory.rs` | | | [X] | |
| `capabilities/src/provider/{anthropic,ollama,openai_compat}.rs` | | [X] 注释 | [X] 错误迁移 | |
| `crates/rustcode-cli/src/main.rs` | | [X] 注释 | | |
| `docs/**`、`README*.md`、`site/**` | [X] | [X] | [X] C7 | |
| `crates/rustcode-tuix/src/event_loop/mod.rs` | [X] X4 | [X] 注释 | | |

**[ERROR] 消解规则**：`capabilities/src/provider/*.rs` 同时被 B（注释）与 C（错误迁移）触碰 —— B 先做，C 重读后再改。

### 6.3 变更影响分析

| 改动 | 受影响入口 | 验证方式 |
|---|---|---|
| 配置新增 `timeout` / `retry` | CLI / TUI / daemon / headless / ACP / clix | 旧 config.toml 回归加载测试 + `cargo test -p rustcode-config` |
| 工厂统一超时映射 | coding runtime / review agent / 子 agent 分层 | `cargo test -p rustcode-coding`、`-p rustcode-review` |
| 适配器错误迁移 | 三个适配器的全部失败路径 | `capabilities/tests/{http_mock,anthropic_mock,ollama_mock}.rs` 全绿 |
| 许可档案去重 | 构建产物 NOTICE、合规扫描 | 人工复核 + `grep` 交叉引用无悬空链接 |
| `extensions/` 类名 rename | IDE 扩展（本机无法编译验证） | 只出规格文档，不落盘 |

### 6.4 门禁（AGENT-D）

```text
G1  cargo fmt --check
G2  cargo clippy --workspace --all-targets -- -D warnings
G3  cargo test --workspace --lib -j 1 --no-fail-fast   （本机只能 --lib，见 6.6）
G4  cargo check --workspace --all-targets
G5  ./scripts/test-headless.sh                          （需先 cargo build）
G6  python3 scripts/acp_smoke.py
G7  grep -rniE "sentry|posthog|segment|amplitude|mixpanel|opentelemetry" \
        --include=*.rs --include=*.toml crates/          必须 0 命中
G8  许可档案无悬空交叉引用（docs/ 与 docs/licenses/ 之间）
G9  全仓新增代码 0 Emoji（grep -P "[\x{1F300}-\x{1FAFF}\x{2600}-\x{27BF}]"）
```

**[WARN] CI 现状缺口**：`.github/workflows/` **只有 `build.yml`**（release 构建矩阵），**没有 fmt / clippy / test job**。建议 BATCH-5 补一份 `ci.yml`。

### 6.5 Mock Provider 测试基建

现有可用资产（无需新建）：

- `wiremock = "0.6"`（capabilities dev-dependency）；`tests/{http_mock,anthropic_mock,ollama_mock}.rs` + `tests/fixtures/*.jsonl` 录制语料；
- kernel 的 `test-support` feature + `src/testkit.rs`；
- `#[ctor]` 把 `RUSTCODE_HOME` 重定向到临时目录（各 crate `lib.rs` 顶部 + 每个 `tests/*.rs`）。**[ERROR] 新增测试必须遵守，否则污染真实 home。**

新增测试落点：

```text
capabilities/tests/llm_wire_parity.rs   同一段脚本化响应分别喂 OpenAI / Anthropic mock，
                                        断言产出同一个 StreamEvent 序列（TIP-3 回归防线）
capabilities/tests/llm_error_map.rs     LlmError 分类表全分支 + 转回 ProviderError 字段无损
config: TimeoutConfig/RetryConfig       反序列化、旧配置兼容、缺省回落
```

### 6.6 当前验证基线（2026-08-30，GATEWAY 判定"绿"的参照）

```text
cargo check --workspace --all-targets                    [PASS]
cargo test --workspace --lib -j 1 --no-fail-fast         [6 FAILED]
```

| 测试 | 位置 | 判定 |
|---|---|---|
| `mcp::registry::tests::trust_key_golden_matches_core_algorithm` | `capabilities/src/mcp/registry.rs` | 既有。`DefaultHasher` 定值过期（期望 `8b6a67e0b2c06dae`，实际 `e07a86b0ce8a1c59`）。重定 golden 属用户决策 |
| `cc_hooks::tests::turn_complete_payload_alignment` | capabilities | 既有。payload 缺 `transcript_path` / `stop_hook_active:false` |
| `tools::read::tests::large_non_code_file_uses_a_bounded_page` | capabilities | 既有。分页读取断言失败 |
| `tools::read::tests::large_symbolless_code_file_falls_back_to_a_bounded_page` | capabilities | 同上 |
| `event_loop::menu_tests::build_skill_menu_items_multi_fragment_and_filter` | `tuix/src/event_loop/mod.rs:7011` | **[NEW] 重命名回归（X4）**：fixture 改名 `rustcode-*`，过滤片段 `"atom smoke"` 未同步 -> 0 匹配 |
| `event_loop::menu_tests::skills_sub_mode_multi_fragment_filters_then_closes_on_complete_skill` | `tuix/src/event_loop/mod.rs:6850` | **[NEW] 同源（X4）** |

**[WARN] 环境限制**：`cargo test --workspace`（含集成测试）链接 `acp_end_to_end` 等大型测试二进制时 OOM（`ld terminated with signal 9`）。本机只能 `--lib` 且需 `-j 1`。完整集成测试需在内存更充裕的机器上执行。

---

## 7. PHASE-3 交付物清单（BATCH-5）

```text
[1] Changelog            变更汇总 + 关键架构改动说明
[2] README.md / README.zh-CN.md   零遥测口径统一；清除 Emoji；补"自定义网关"配置章节
[3] config.example.toml  升级 docs/config.example.toml（当前 331 行，缺 timeout / retry /
                         model_mapping 示例），并在仓库根提供可发现的入口
[4] .env.example         新增：RUSTCODE_HOME / RUSTCODE_API_KEY / RUSTCODE_BASE_URL /
                         RUSTCODE_MODEL / 代理与 TLS 相关变量（全部占位，无真实凭据）
[5] 验证指南             本地编译 / 单元测试 / 模型连通性测试命令 + 门禁脚本
```

模型连通性测试建议（不写入仓库，只写进验证指南）：用 `wiremock` 起本地假网关跑
`capabilities/tests/*_mock.rs`；真实连通性用一次性 `RUSTCODE_BASE_URL` 指向自建网关，
**禁止**在测试或默认配置中硬编码任何真实 API Key。

---

## 8. 风险登记册

| ID | 风险 | 等级 | 缓解 |
|---|---|---|---|
| R1 | 工作区 dirty 且有并发写入方，批量改动放大冲突 | 高 | 独立分支；每 Agent 交付即 `cargo check`；改前重读；禁 git 写操作（commit/stash/reset/clean） |
| R2 | 机械重命名漏改语义依赖（X4 已实证） | 高 | 改完必跑全量 `--lib`；对 fixture / 过滤串 / golden 常量单独人工核对 |
| R3 | `docs/licenses/` 与顶层四件套内容漂移 | 中 | BATCH-1 去重到单一事实源，交叉引用一次收敛 |
| R4 | 上游 rustcode 许可文本缺失 | 中 | 保持占位，绝不臆造；取得后逐字补入 `LICENSE-MIT-ORIGINAL.md` |
| R5 | 在 kernel 上叠第二套 trait 破坏架构约束 | 中 | 已在 4.2 明令禁止；评审重点检查 |
| R6 | 超时语义混淆（HTTP 层 vs kernel round-trip） | 中 | 配置键命名区分；`config.rs:56` 的 doc 注释补充对照说明 |
| R7 | `extensions/` 类名 rename 本机无法编译验证 | 中 | 只出规格文档，不落盘；交付说明标注"两侧命名尚未统一" |
| R8 | 无 CI 质量门禁 | 中 | BATCH-5 补 `ci.yml`（fmt -> clippy -> test） |
| R9 | 会话目录迁移丢数据 | 中 | rename 而非 copy；迁移前写备份标记；失败保留旧目录并告警 |

---

## 9. GATEWAY 待确认清单

请逐条确认（或给出替代答案），确认后方可进入 PHASE-2 编码。

### 必须项

1. **[必须] 是否再做一次品牌切换**：D1 已锁定 `rustcode` 且已落地，`crates/` 命中 0。
   维持 `rustcode` -> O1 只剩 2.3 的 `extensions/` / `site/` / `packages/` 收尾；
   若要换名，请给出新名，2.2 整张映射表替换"目标"列即可，执行步骤不变。
2. **[必须] O4 去重方向**：保留 `docs/licenses/` 作为唯一档案（顶层四个文件改索引或删除），
   还是保留顶层四个文件（删除 `docs/licenses/`）？根 `LICENSE` 目前指向 `docs/licenses/`。
3. **[必须] `docs/licenses/LICENSE-MIT-ORIGINAL.md` 的模板 MIT 正文**：确认删除（本方案建议），
   还是保留？—— 它与"绝不臆造许可文本"直接冲突。
4. **[必须] 6 个测试失败的处理**：4 个既有失败是否本轮修（其中 `trust_key_golden` 需重新定值
   golden 常量，属决策）；2 个 X4 重命名回归必须修（本方案建议 BATCH-0 先修）。
5. **[必须] 超时是否下沉到 `[providers.*]`**：本方案 4.4/4.5 建议下沉（配置段 + 工厂统一映射）；
   替代方案是只改文档、保持现状（coding 层默认值）。
6. **[必须] `--no-telemetry` 参数现状**：维持"被忽略的 no-op + stderr 告警"（当前实现，老扩展可用），
   还是改为显式报错？

### 建议项

7. **[建议] `extensions/` 的 `RustCode*` 类名与 `site/` 品牌字符串是否纳入本次范围**（845 + 1493 处，
   本机无法编译验证；本方案建议只出规格文档，不落盘）。
8. **[建议] 是否补 CI 门禁**（`ci.yml`：fmt -> clippy -> test）。
9. **[建议] 会话数据策略**：`~/.rustcode` 保持不变（推荐，无迁移风险），还是做一次性目录迁移？
10. **[建议] daemon 端口** `13456 / 13457 / 13458`：确认保持不变（改了会断 IDE 扩展与手机端）。
11. **[建议] 遥测定义边界**：版本更新检查（`tuix/version_check.rs`）与本地 turn datalog
    是否也算"遥测"而需移除？本方案按"不算，但需显式开关 + 改名 `update_check`"处理。
12. **[建议] 是否把 `version_check.rs` 迁到 `egress`**（AGENT-E 可选批次）。

---

## 10. 附录：关键代码位置（2026-08-30 实测）

```text
crates/rustcode-kernel/src/provider.rs                  :150 ChatResponse  :168 LlmProvider
                                                        :186 bind_session_id  :225 chat()
crates/rustcode-kernel/src/stream.rs                    :36  ProviderError  :93  StreamEvent
crates/rustcode-kernel/src/testkit.rs                        测试装配（feature = test-support）
crates/rustcode-capabilities/src/egress/client.rs            唯一出站 HTTP 工厂
crates/rustcode-capabilities/src/egress/config.rs      :20   SecretString（zeroize，不实现 Serialize）
crates/rustcode-capabilities/src/egress/error.rs             EgressError + brief/detail
crates/rustcode-capabilities/src/provider/error.rs     :24   LlmError  :60  retryable()  :80  from_provider()
crates/rustcode-capabilities/src/provider/mod.rs             适配器导出
crates/rustcode-capabilities/src/provider/{openai_compat,anthropic,ollama}.rs  三套适配器
crates/rustcode-capabilities/src/provider/retry.rs           RetryPolicy（Retry-After HTTP-date）
crates/rustcode-config/src/config/provider.rs           :110  model_mapping  :483  ModelMapping
crates/rustcode-config/src/config/mod.rs                :4310 legacy_telemetry_section_tests
crates/rustcode-config/src/session_mode.rs                   SessionMode / RepoOrigin（非遥测）
crates/rustcode-config/src/distribution.rs                   HOME_ENV / 端口 / 进程名 / 发布前缀
crates/rustcode-config/src/endpoints.rs                      11 个 RUSTCODE_* 环境变量
crates/rustcode-coding/src/provider_factory.rs          :112  model_mapping 应用  :114  协议族分派
crates/rustcode-coding/src/config.rs                    :48   stream_timeout  :56  request_timeout
crates/rustcode-coding/src/runtime.rs                   :814  CodingRuntime  :1096 CodingRuntimeHandle
crates/rustcode-tuix/src/event_loop/mod.rs              :6850 / :7010  X4 两处待修过滤片段
crates/rustcode-cli/src/main.rs                         :2688 spawn_native_cli_runtime
```
