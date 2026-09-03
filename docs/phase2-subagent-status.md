# PHASE-2 子 Agent 状态与分工

```text
[BASELINE] branch=dev  HEAD=6dbf57bb  worktree=dirty
[DISPATCH] 并行 Task 调用在本运行时不可用（返回 "No result found"）——必须串行派发
[ORDER]    O4 合规 -> BATCH-1  -> BATCH-2(A) -> BATCH-3(B ‖ C) -> BATCH-4 -> BATCH-5(D)
```

## 1. Agent 状态总表

| Agent | 范围 | 状态 | 已完成 | 剩余 |
|---|---|---|---|---|
| **AGENT-A** | 重命名收尾 + 合规 | [DONE] | `docs/architecture.md` / `docs/mcp.md` / `docs/target-architecture.md` / `docs/testing/windows-path-normalization.md` 的 `rustcode-*` 修正 + O4 License 合规（见 3.7） | `extensions/`（vscode/jetbrains）与 `site/` 旧前缀（已评估规模，单独立项，见 3.7） |
| **AGENT-B** | 零遥测死脚手架 | [DONE]（主 Agent 收尾） | B1-B5、B7-B12 | B6 部分、B10 文档、B9 改名 |
| **AGENT-C** | 外部调用收敛（egress） | [PARTIAL] | 见 3.4 | MCP 传输层的静默降级（见 3.5） |
| **AGENT-D** | 门禁 / 测试 / 收口 | [PENDING] | — | G1-G8；补 `ci.yml`；Mock/wire 一致性测试 |

## 2. AGENT-B 完成记录（2026-08-30）

已删除/迁移：

- `crates/rustcode-config/src/telemetry_legacy.rs` **删除**；`SessionMode` /
  `RepoOrigin` / `detect_repo_origin` 迁至 `crates/rustcode-config/src/session_mode.rs`。
  `CliOverride` **删除**。
- `crates/rustcode-clix/src/tel.rs` **删除**（no-op 桩、`meter_provider`、
  `.clix_telemetry_notice` marker、`RUSTCODE_TELEMETRY` 文案）。
  `build_review_provider` 迁至 `crates/rustcode-clix/src/code.rs`（单一实现）。
- `crates/rustcode-cli/src/telemetry_cmd.rs` **删除**（源码与集成测试两个文件）。
- `runtime_config_from()` 去掉 `CliOverride` 参数（含 6 处测试调用点同步）。
- daemon `--no-telemetry` 按决策降级为**被忽略的 no-op + stderr 告警**（不报错退出，
  老 IDE 扩展仍会传该参数）。

主 Agent 收尾（子 Agent 被中断，遗留修复）：

1. `build_review_provider` 在 `main.rs` 与 `code.rs` 重复定义 -> 收敛到 `code.rs`。
2. `runtime_config_from` 的 6 处测试调用点参数个数同步。
3. 删除 `tests/telemetry_cmd.rs`（测的是已移除的 `telemetry status/clear` 子命令）。
4. 清理随之产生的 `unused import: std::sync::Arc`。

验证：

```text
cargo check -p rustcode-config -p rustcode-clix -p rustcode -p rustcode-daemon --all-targets   [PASS]
cargo check -p rustcode-clix --all-targets                                                      [PASS, 0 warning]
cargo test  -p rustcode-config     318 passed / 0 failed
cargo test  -p rustcode-clix       7 + 3 + 0 + 44 passed / 0 failed
```

已知非回归失败（环境导致，非本次改动）：`rustcode-daemon` 的
`webui::tests::serves_embedded_index` 与 `unknown_path_falls_back_to_index` ——
本机没有 `webui/dist/`（gitignored，需先 `cd webui && npm ci && npm run build`）。

## 3. AGENT-C：外部直接调用收敛（待派发）

### 3.1 现状盘点（已核实）

生产侧 `reqwest` 客户端构造点，**没有任何一处共享**：

| 调用点 | 文件 | 现状 |
|---|---|---|
| LLM（OpenAI 兼容） | `capabilities/src/provider/openai_compat.rs:297` | 已有 `build_http_client()`（含 #514 信任根回退、proxy、UA、pool idle），**质量最高** |
| LLM（Anthropic） | `capabilities/src/provider/anthropic.rs:133` | 自己拼 builder，未复用上面那份 |
| LLM（Ollama） | `capabilities/src/provider/ollama.rs:99` | 同上 |
| `web_fetch` | `capabilities/src/tools/web_fetch.rs:530` | 自建，手动重定向 + IP pin（SSRF 防护），浏览器 UA |
| `web_search` | `capabilities/src/tools/web_search.rs:164` | 自建，浏览器 UA，无 connect_timeout |
| AtomGit REST | `capabilities/src/atomgit/client.rs:26` | 自建 |
| MCP HTTP 传输 | `capabilities/src/mcp/transport_http.rs:71` | 自建，失败时 `unwrap_or_else(\|_\| reqwest::Client::new())` —— **静默丢掉 proxy 与 TLS 配置** |
| 自更新 | `updater/src/lib.rs:264,314` | 自建，30s / 600s 硬编码超时 |

共有基础件：`capabilities/src/proxy.rs::apply_async_proxy_policy()`（代理策略）、
`provider/openai_compat.rs::add_trusted_roots()`（OS/SSL_CERT_FILE 信任根）。

### 3.2 参考项目 kimi-cli-for-xbow 的可迁移模式

该仓库是 **Python**（kimi-cli 的 CTF/安全分支），代码不可复用，但下列模式可迁移：

| 模式 | 参考实现 | 迁移到本仓库 |
|---|---|---|
| 单一 HTTP 客户端工厂 | `utils/aiohttp.py::new_client_session()`（固定 `certifi` 信任根） | 抽 `capabilities/src/egress/client.rs`，把 #514 信任根回退 + proxy + UA + 超时合并为唯一构造入口，禁止各处 `reqwest::Client::new()` |
| 服务级外部调用与 LLM 同构配置 | `MoonshotSearchConfig{base_url, api_key: SecretStr, custom_headers}` | 抽 `ExternalServiceConfig`，供 web_fetch / web_search / atomgit / mcp 复用 |
| 连接与模型分离 | `LLMProvider` + `LLMModel` | 已有 `ProviderAccountConfig` + `ModelProfileConfig`，无需改动 |
| 未配置服务优雅降级 | `SearchWeb.__init__` 里 `raise SkipThisTool()` | web_search/atomgit 未配置时不注册工具，而不是运行时报错 |
| 错误返回 brief/detail 两件套 | `ToolResultBuilder.error(msg, brief=...)` | `ToolResult{is_error}` 增加一行摘要，UI 展示 brief、模型看到 detail |
| 混沌替身 | `provider.type == "_chaos"` 注入 429/500/503 | kernel `testkit` 增加 chaos provider，替代部分 wiremock 场景 |
| 环境变量覆盖可回显 | `augment_provider_with_env_vars()` 返回 `applied` 字典 | `resolved_api_key()` 增加"哪些 env 生效了"的诊断回显 |

### 3.3 AGENT-C 任务边界

**做**：新增 `capabilities/src/egress/`（`client.rs` / `config.rs` / `error.rs`），
迁移 `web_fetch`、`web_search`、`atomgit/client`、`mcp/transport_http` 四处到统一工厂；
补单元测试；`MCP transport_http` 的 `unwrap_or_else(|_| Client::new())` 必须消除
（静默丢失 proxy/TLS 是真 bug）。

**不做**（高风险，单独批次）：`provider/` 三件套的 HTTP 客户端不动；
`updater` 不动（跨 crate，且超时策略是刻意的 30s/600s）。

### 3.4 AGENT-C 完成记录（2026-08-30）

新增（子 Agent 产出 `client.rs` / `error.rs`，主 Agent 补 `mod.rs` / `config.rs`：

- `capabilities/src/egress/client.rs` —— 唯一 HTTP 客户端工厂。
  `HttpClientSpec`（connect / request / pool-idle / UA / proxy / skip_tls_verify /
  follow_redirects / trust_os_roots / max_tls_version）+ `build_http_client()` +
  `build_pinned_http_client()`（给 `web_fetch` 的 DNS pin）+ `spec_builder()`。
  完整移植并泛化了 issue #514 的信任根逻辑：OS 根 + `SSL_CERT_FILE` 逐证书预校验后
  叠加，构建失败则回退 webpki base 一次并 `tracing::warn!`。
- `capabilities/src/egress/error.rs` —— `EgressError`（thiserror）+ `brief()` /
  `detail()` 两件套（对标参考项目 `ToolResultBuilder.error(msg, brief=...)`），
  响应体一律截断到 `BODY_EXCERPT_BYTES = 512` 字节，避免整页 HTML 灌进上下文；
  含 `retryable()` 判定（408/429/5xx 可重试）。
- `capabilities/src/egress/config.rs` —— `ExternalServiceConfig`（base_url /
  api_key / custom_headers / timeout）+ `SecretString`（`zeroize` 落盘即擦，
  `Debug` 打印 `***`，不实现 `Serialize` 以免凭据被写进配置文件）。
- feature gating：新增 `egress` feature，由 `provider` / `web` / `atomgit` / `mcp`
  各自拉起，不单独启用；零出站能力的 lean 构建仍不编译 HTTP 栈。

已迁移的调用点：

| 调用点 | 迁移后 | 行为变化 |
|---|---|---|
| `tools/web_fetch.rs::build_client` | `build_pinned_http_client(&browser_spec().with_no_redirects(), host, pinned)` | **保留** IP pin 与手动重定向（SSRF 语义不变）；新增 #514 信任根回退 |
| `tools/web_search.rs::client` | `build_http_client(&browser_spec())` | **新增** connect 超时（此前缺失）；UA 统一到 `BROWSER_UA`；新增 OS 信任根分层 |
| `atomgit/client.rs::new` | `build_http_client(&spec.with_user_agent(cfg.user_agent))` | UA 与 30s 预算不变；新增 #514 回退与 pool-idle 策略 |

### 3.5 AGENT-C 未完成项（明确记录，不以"功能可用"替代完成）

- **[TODO] `mcp/transport_http.rs:71` 的 `unwrap_or_else(|_| reqwest::Client::new())`
  仍然存在** —— 构造失败时静默退回一个忽略代理与 TLS 策略的裸客户端。
  消除它需要把 `HttpClient::new` 改为返回 `Result`，而两处生产调用点在
  `tokio::spawn` 的 `async move` 闭包内（`registry.rs:532` 与 `:676`），无法直接
  用 `?`，必须重新设计"MCP 服务器连接失败"的上报路径。本轮不做半改，单列任务。
- **[TODO] C3（provider 三件套收敛到 egress）未执行。** 三个 LLM 主链路适配器保持
  原状 —— 它们是已验证的热路径，为统一而改动风险高于收益。
- **[TODO] `updater` 未动**（跨 crate，30s / 600s 超时是刻意的）。

### 3.6 验证结果

```text
cargo check -p rustcode-capabilities --all-targets --features web,atomgit,mcp   [PASS]
cargo test  -p rustcode-capabilities --features web,atomgit,mcp --lib egress    [PASS] 20 passed / 0 failed
cargo test  -p rustcode-capabilities --features web,atomgit,mcp --lib           [1009 passed / 1 FAILED]
cargo check -p rustcode --all-targets                                           [PASS]
```

**[WARN] 那个 1 FAILED 是既有失败，与本次改动无关**：
`mcp::registry::tests::trust_key_golden_matches_core_algorithm`
（期望 `8b6a67e0b2c06dae`，实际 `e07a86b0ce8a1c59`）。`project_trust_key` 用
`std::collections::hash_map::DefaultHasher`，其输出不保证跨版本稳定；该文件在本轮
之前就是 staged 修改状态，本次未触碰。修它需要重新定值 golden 常量——属于用户决策，
按 AGENTS.md "不擅自改动无关代码" 留给用户。

## 3.7 AGENT-A 完成记录（2026-08-30）：文档命名 + O4 合规

AGENT-A 派发到 `code-explorer` 子代理，**该实例是只读的**（返回了精确规格但未落地），
由主 Agent 执行：

- `docs/architecture.md`：21 处 `rustcode-*` crate 名 -> `rustcode-*`；删除
  `rustcode-telemetry` 整行（该 crate 已退役）；标题 `RustCode Architecture` ->
  `RustCode Architecture`。**刻意保留** `rustcode-core` / `rustcode-bridge`
  （退役件，从未以 `rustcode-*` 形态存在，改名会伪造事实）。
- 同批处理 `docs/mcp.md`、`docs/target-architecture.md`、
  `docs/testing/windows-path-normalization.md`（只改仍在描述当前架构的 crate 名）。
  **`docs/plans/**` 与历史回溯文档一律不动**——它们是历史记录。
- O4：新建 `docs/UPSTREAM_RUSTCODE_LICENSE.md` 归档前身 `SecLab/RustCode` 的 MIT 正文；
  `docs/ORIGINAL_LICENSE.md` 改为**诚实占位**（声明应归档上游 rustcode 的 MIT 但尚未
  取得，并给出补齐指引，**绝不臆造许可文本**）；同步修正 `UPSTREAM_CREDITS.md`、
  `THIRD_PARTY_NOTICES.md` 与根 `LICENSE` 的自指表述。
- 未处理（已评估规模，单独立项）：`site/` 779 处、`extensions/` 约 751 处
  `rustcode` 残留，且 `extensions/` 含 `RustCode*` 类名与文件名，需符号级 rename，
  本机无法编译验证。

## 3.8 AGENT-E 完成记录（主 Agent 执行）：MCP 静默降级

- `mcp/transport_http.rs`：`HttpClient::new` -> **`try_new` 返回 `Result<Self, String>`**，
  改用 `egress::client::build_http_client`。消除了
  `unwrap_or_else(|_| reqwest::Client::new())` —— 那个 fallback 会在构造失败时静默
  退回一个忽略代理与 TLS 信任根策略的裸客户端。
- 调用点：`add_server`（`registry.rs:676` 区）用 `match` + `anyhow!` 走既有失败
  上报路径；`tokio::spawn` 内那处（不能用 `?`）把客户端构造改为
  `Result<Box<dyn McpClient>, String>`，失败时走**与 `initialize()` 失败完全相同**
  的上报路径（写 `failed_servers` + 发 `McpConnectEvent::Failed`）。
- 2 处测试构造点改为 `try_new(...).expect(...)`（测试内 expect 符合仓库约定）。

## 3.9 [WARN] 并发写入事故：有后台 Agent 仍在异步落盘

本轮发现**除主 Agent 派发的子代理外，还有 Agent 在异步写文件**，证据：

- `provider/mod.rs:20` 在会话中途新增 `mod error;`，`provider/error.rs` 是未跟踪新文件；
- 同一文件在我 `cargo check` 与 `grep` 之间行号发生漂移；
- AGENT-E 派发后仅 10 次工具调用即终止。

该后台 Agent 正在实现 **OBJECTIVE-3 的 model mapping**（正是第 3.4 节登记的缺口）：
新增 `ModelMapping(pub HashMap<String,String>)` + `ProviderKind` 枚举，并在
`provider_factory.rs` 里应用 `cfg.model_mapping.resolve(&cfg.model)`。但它**中途停止**，
留下 11 个编译错误。主 Agent 接手收尾：

1. `provider/error.rs` 两处 `#[test]` 缺 `fn` 关键字；
2. `provider/openai_compat.rs` 的 `ChatResponse` / `effort_known_unsupported` 未定义
   （该 Agent 后续自行收敛）；
3. `ProviderKind::OpenAiCompatible` 分支缺失 -> 与 `None` 合并为同一回退路径；
4. `model_mapping` 新增为**非 Option** 字段，10 处 `ProviderConfig` /
   `ModelProfileConfig` 结构体字面量需补 `ModelMapping::default()`（用脚本一次性插入）。

## 3.10 当前验证基线

```text
cargo check --workspace --all-targets     [PASS]
cargo test  --workspace --lib -j 1        [1522 passed / 4 FAILED]
cargo test  -p rustcode-capabilities --lib egress   [PASS] 20 passed
cargo test  -p rustcode-capabilities --features mcp --lib mcp  [99 passed / 1 FAILED]
```

**[WARN] 4 个失败全部在本次改动的影响半径之外，均为既有/在途失败**：

| 测试 | 位置 | 判定 |
|---|---|---|
| `mcp::registry::tests::trust_key_golden_matches_core_algorithm` | `registry.rs:1502` | 既有。`DefaultHasher` 定值过期（期望 `8b6a67e0b2c06dae`，实际 `e07a86b0ce8a1c59`）。重定 golden 属用户决策 |
| `cc_hooks::tests::turn_complete_payload_alignment` | `cc_hooks.rs:1177` | 既有。payload 缺 `transcript_path` / `stop_hook_active:false`。`cc_hooks.rs` 未在任何批次内被改动 |
| `tools::read::tests::large_non_code_file_uses_a_bounded_page` | `read.rs:1254` | 既有。分页读取断言失败。`tools/read.rs` 未被本轮改动 |
| `tools::read::tests::large_symbolless_code_file_falls_back_to_a_bounded_page` | `read.rs:1231` | 同上 |

**[WARN] 环境限制**：`cargo test --workspace`（含集成测试）在链接 `acp_end_to_end`
等大型测试二进制时 OOM（`ld terminated with signal 9 [Killed]`）。本机只能跑
`--lib`，且需 `-j 1` 降低峰值内存。完整集成测试需在内存更充裕的机器上执行。

## 4. 派发纪律

- 串行派发（并行 Task 会失败）。
- 每个 Agent 交付后由主 Agent 跑一次 `cargo check --all-targets` 验收再加下一个。
- 三个 Agent 都会碰 `cli/src/main.rs`：A 与 B 已先后改过，C 若需要改应放在最后并
  重新读取文件（防止 `replace_in_file` 命中过期上下文）。
- 工作区 dirty，禁止任何 git 写操作（commit/stash/reset/clean）。
