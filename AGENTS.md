# 项目全局开发约束 (RustCode fork)

## 适用范围与使用方式

本文件记录 RustCode fork 的重构目标、当前架构边界和长期开发约束。历史方案、
旧基线和上游 core 退役过程留在 `docs/`,不得继续作为当前实现前提。

- 二次开发的总体设计与分阶段计划见 **`docs/phase1-refactor-design.md`**(PHASE-1
  方案,GATEWAY 待审核;审核通过前不得进入 PHASE-2 编码)。本文件只放长期约束。

涉及以下范围时,设计或修改前必须先核对当前代码、调用方和近期 Git 历史:

- `crates/rustcode-kernel/`;
- `crates/rustcode-capabilities/`;
- `crates/rustcode-coding/` 的 runtime、provider、session、controller;
- CLI、TUI、daemon、ACP、clix 的 runtime、session、command/event 接入;
- daemon 中保留的历史 core JSON 单向 importer 与兼容 DTO;
- 公共协议、持久化格式、审批、安全边界或跨 crate 依赖方向。

本文件描述的是约束,不是永远正确的现状快照。若约束中的事实与当前代码冲突,
以当前代码为准;先说明差异,再修正文档或实现,不得按旧路径盲目补代码。
`docs/architecture.md` 与 `AGENTS.md` 都曾长期停留在 `atomcode-*` 旧命名上,
引用 crate 前先确认目录真实存在。

## 常用命令

构建:

- `cargo build` — 构建 `default-members`(`rustcode-cli` / `rustcode-daemon` /
  `rustcode-tuix`),产物 `target/debug/rustcode`。
- `cargo build --release -p rustcode` — 只构建发布版 CLI。`rustcode-cli` 的**包名
  是 `rustcode`**(`-p rustcode-cli` 会失败),产物 `target/release/rustcode`。
- `cargo build --workspace` — 构建全部 `crates/*`(含默认成员外的 `clix`、
  `review`、`codingplan`)。
- `cargo install --path crates/rustcode-cli --locked` — 安装到 `~/.cargo/bin`。
- WebUI 需先构建前端:`cd webui && npm ci && npm run build` 产出 `webui/dist/`
  (gitignored)。重建前端后必须 `cargo clean -p rustcode-daemon`,cargo 不追踪
  `webui/dist/` 变化;缺失时所有 webui 页面返回 `webui not built`。
- 发布打包:`scripts/release*.sh`、`scripts/macOS-release-*.sh`、
  `scripts/linux-release-*.sh`、`scripts/sign-macos.sh`;矩阵见
  `.github/workflows/build.yml`。
- `RUSTCODE_HOME` 覆盖配置目录(默认 `~/.rustcode`)。**禁止 `sudo` 运行**——
  `~/.rustcode` 一旦出现 root 属主文件,后续非 root 启动在 runtime 初始化即失败。

测试:

- `cargo test` — 默认成员;`cargo test --workspace` — 全量(跨 crate 改动必跑)。
- `cargo test -p rustcode-capabilities` — 单 crate;
  `cargo test -p rustcode-coding <filter>` — 按测试名过滤;`--` 后接
  `--nocapture` / `--test-threads=1`。
- `cargo test -p rustcode-kernel --test <name>` — 只跑某个 `tests/*.rs`;
  `--lib` 只跑内联单测,`--bins` 只跑 bin 内测试。
- `./scripts/test-all.sh` — 全量测试并产出 `test-report.md`,能区分"编译失败 /
  测试失败 / 全部通过"。
- `./scripts/test-headless.sh` — headless 冒烟(需先 `cargo build`);未设
  `RUSTCODE_TEST_PROVIDER` 时跳过联网用例。
- `./scripts/smoke-test-all.sh`(校验 test-all.sh 自身)、
  `python3 scripts/acp_smoke.py`(ACP stdio 冒烟)、
  `python3 scripts/analyze_datalogs.py`(turn datalog 分析)。
- 测试隔离:`coding` / `tuix` / `daemon` / `capabilities` / `cli` 的入口文件顶部
  有 `#[ctor]` 把 `RUSTCODE_HOME` 重定向到临时目录。新增测试不得依赖真实
  `~/.rustcode`;**重命名这批目录/变量名时同步修改,否则测试隔离失效**。

Lint / 格式:

- `cargo fmt` / `cargo fmt --check`;提交前 `cargo clippy --workspace --all-targets`。
- `cargo check --workspace --all-targets` — 快速编译校验;`cargo test` 已覆盖相同
  编译验证时不再重复 `cargo check`。
- 仓库当前无 `rustfmt.toml` / `clippy.toml` / `[lints]` 段,无需新增。

## 架构总览(分层与 crate 地图)

```text
L3  drivers   rustcode-cli(pkg `rustcode`)  rustcode-tuix  rustcode-daemon
              rustcode-clix(bin `rustcodex`)  ACP(src/acp/)
                    |             |                |
L2  specialize      |             +----> rustcode-coding (CodingRuntime)
                    |                          |     `-- rustcode-review
L1  capabilities    rustcode-capabilities <----+
L0  neutral         rustcode-kernel <----------+
leaf                rustcode-config / rustcode-auth / rustcode-updater
                    rustcode-codingplan / rustcode-codingplan-crypto
```

- 依赖只向下:`kernel`(无内部依赖) <- `capabilities`(禁止反向依赖 coding /
  driver / 已退役 core) <- `coding`(另依赖 kernel、config、review) <-
  `tuix`(另依赖 daemon、updater、codingplan、auth) <- `cli`(唯一同时依赖
  tuix + daemon)。`clix -> review, coding, capabilities, kernel, config`。
- 工作区 `members = ["crates/*"]`,`default-members` 为 cli / daemon / tuix;
  `rustcode-codingplan-crypto` 是闭源签名占位桩,默认成员故意不含它,官方构建用
  `--features rustcode/codingplan-crypto` 开启。
- **`CodingRuntime`**(`coding/src/runtime.rs:814`)是 coding agent 的唯一运行时
  所有者,对外暴露 `CodingRuntimeHandle`(同文件 `:1096`)。Driver 通过有序
  fire-and-forget 的 `DriverCommand`(同文件 `:525`)驱动,读 `CodingRuntimeEvent`;
  不得自建第二套 live agent 生命周期。
- 启动路径:CLI `spawn_native_cli_runtime`(`cli/src/main.rs:2688`)、ACP
  `rustcode_daemon::spawn_native_runtime_for_session_deferred_with_preprocessor`、
  daemon `kernel_runtime::start_native_runtime*`,三者最终都落到
  `CodingRuntime::start_with_session_lease` / `start_with_bootstrap`。
  **TUI 不自己启动 runtime**:外部把已启动的 `SpawnedRuntime` 传进 `tuix::run`。
- kernel `Agent`(`kernel/src/agent.rs:794`)+ `AgentBuilder` / `AgentHandle` /
  `AgentCommand` / `AgentEvent` 是中立循环,不得承载 coding 产品语义。
  `rustcode-review` 是独立业务 agent,以 `code_review` 子 agent 工具挂进 coding。
- 测试布局:`tests/` 集成测试在 kernel(25)、capabilities(11)、coding(11)、
  cli(8)、config(2)、daemon(2)、tuix(1);review / auth / clix / updater /
  codingplan 只有内联 `#[cfg(test)]`。可运行示例见
  `kernel/examples/minimal_specialization.rs`、`coding/examples/run_task.rs`、
  `cli/examples/acp_stdio_agent.rs`。
- 运行时领域术语(Live View、Runtime Generation、Session Transition、Tool
  Catalog Revision、Committed Snapshot、Replay Window)**以 `CONTEXT.md` 为准**,
  其中 `_Avoid_` 条目是硬性命名约束。

## Fork 重构目标与完成状态

本仓库是 `SecLab/RustCode` 的二次开发 fork,目标由 `prompt.txt` 定义,核心约束:
禁止使用任何 Unicode Emoji,日志/注释一律用 ASCII 标签 (`[INFO]`/`[WARN]`/
`[ERROR]`/`[SUCCESS]`/`[CHECK]`/`[+]`/`[-]`/`[*]`)。

### [OBJECTIVE-1] 产品重命名 — [DONE]

- 14 个 crate 全部 `atomcode-*` -> `rustcode-*`。
- 二进制:`rustcode` (cli)、`rustcode-daemon` (daemon);TUI crate 为 `rustcode-tuix`。
- 配置目录 `~/.rustcode`;环境变量前缀 `RUSTCODE_*`(集中在
  `rustcode-config/src/distribution.rs` 与 `endpoints.rs`)。
- Wire-contract keys 迁移到 `rustcode.*` 命名空间;不兼容读取旧 `~/.atomcode`
  session,按"fresh start"决策处理。
- 全局重命名映射(关键项):

  | 维度        | 旧 (atomcode)            | 新 (rustcode)                       |
  |-------------|--------------------------|-------------------------------------|
  | crate 前缀  | `atomcode-*`             | `rustcode-*`                        |
  | CLI 二进制  | `atomcode`               | `rustcode`                          |
  | daemon 二进制 | `atomcode-daemon`      | `rustcode-daemon`                   |
  | TUI 二进制  | `atomcodex`              | `rustcode-tuix` (crate)             |
  | 配置目录    | `~/.atomcode`            | `~/.rustcode`                       |
  | home env    | `ATOMCODE_HOME`          | `RUSTCODE_HOME`                     |
  | env 前缀    | `ATOMCODE_*`             | `RUSTCODE_*` (api key/ proxy/ 等)   |
  | config 段   | `[telemetry]` 等        | 已删除 telemetry 段;provider 段保留 |

重命名的事实源集中在两个文件(改这两处即可带动大部分):
`rustcode-config/src/distribution.rs`(`HOME_ENV` / `HOME_DIR_NAME` / 端口
`13456,13457,13458` / `PROCESS_NAMES` / `WINDOWS_INSTALL_DIR` /
`RELEASE_ASSET_PREFIX` / `UPDATE_TEMP_PREFIX`)与
`rustcode-config/src/endpoints.rs`(11 个 `RUSTCODE_*` 环境变量名 + 4 个托管端点)。

**[CHECK] `rustcode` 是已锁定的产品身份**(决策 D1,见
`docs/REFACTOR_DESIGN_PHASE1.md` §2.0),不是中间态;不要再次改名。O1 的剩余
工作只有 `extensions/` / `site/` / `docs/architecture.md` 的旧 `atomcode` 前缀
收尾,清单见 `docs/phase1-refactor-design.md` 第 1 节。

### [OBJECTIVE-2] 零遥测 — [PARTIAL]

已完成:

- `rustcode-telemetry` crate 已删除;`Telemetry`/`Event`/`CurrentContext`/
  `track`/`install_panic_hook` 等上报调用已从 cli/daemon/coding/auth/clix/
  tuix/config 移除。**全仓不存在 Sentry/PostHog/Segment/GA 等第三方埋点 SDK 或
  依赖**——历史遥测是自建管线,拆的是自己的代码。
- `telemetry` 子命令与 `[telemetry]` 配置段已从 schema 移除;`rustcode-config`
  保留回归测试确认遗留段被**静默忽略**(见 `config/mod.rs` 的
  `legacy_telemetry_section_tests`)。
- 崩溃处理仅保留 stderr 输出,无离箱上报。
- 依赖侧已满足最小化:`reqwest = { default-features = false, features =
  ["stream","json","rustls-tls"] }`,`tokio` 只开 `rt/macros/sync/time`。

仍残留(实测,详见 `docs/phase1-refactor-design.md` 第 2 节 B1-B12):

- `rustcode-config/src/telemetry_legacy.rs` 整个模块仍在,`CliOverride` 仍被
  `cli/src/main.rs:41` 与 `daemon` 使用 —— 该删。
- `daemon/src/main.rs:59` 仍解析 `--no-telemetry` 并构造 `CliOverride`;
  `ServerOpts.cli_override` 仍在。**删前必须先处理 IDE 扩展会传该参数的问题。**
- `clix/src/tel.rs` 整个 no-op 桩仍在(空 `Telemetry`、`meter_provider` 直通、
  `.clix_telemetry_notice` marker、`RUSTCODE_TELEMETRY` 文案)。
- `docs/telemetry.md`、README / README.zh-CN 遥测章节、
  `site/docs/{en,zh}/headless-daemon.html` 仍是遥测口径。
- kernel 的 `event.rs`/`message.rs`/`hook.rs`/`agent.rs` 与 capabilities 的
  `Cargo.toml` 注释里仍有 telemetry 字样(纯注释,无上报)。

**不得误删**:`SessionMode`(daemon 用它区分 vscode / jetbrains / webui 客户端,
是本地分支逻辑)与 `RepoOrigin` / `detect_repo_origin`(纯字符串解析,无网络,
被 daemon 用于会话分桶)。删除 `telemetry_legacy` 模块时把它们迁到中性模块名。
注意:注释中出现的 `telemetry`/`segment`/`datalog` 多为单词歧义或 path segment,
不是上报调用点;改动前用 grep 确认是否存在真实网络上报。

### [OBJECTIVE-3] LLM Provider 解耦 — [DONE, 有缺口]

- 统一 trait `LlmProvider`(`kernel/src/provider.rs:123`),`async_trait`,
  `Send + Sync`,唯一方法是 `chat_stream` -> `BoxStream<'static, StreamEvent>`。
  **[CORRECTION] 不存在 `message_stream`;非流式走同一入口的汇聚路径。**
- 三个适配器在 `capabilities/src/provider/`:`anthropic.rs`(原生
  `/v1/messages`,`AnthropicSseDecoder` 处理 `message_start` /
  `content_block_start` / `content_block_delta` / `message_delta` /
  `message_stop` / `error` / `ping`)、`openai_compat.rs`(`/v1/chat/completions`,
  `delta.content` + `[DONE]`)、`ollama.rs`(`/api/chat`,NDJSON)。
- 工厂是 **trait** `CodingProviderFactory`(`coding/src/provider_factory.rs:74`),
  默认实现 `DefaultCodingProviderFactory` 按 `provider_type` 分发
  (`claude|anthropic|anthropic-compatible` -> Anthropic,`ollama` -> Ollama,
  `openai|openai-compatible|_` -> OpenAI 兼容);另有 `ProviderAuthenticator`
  seam 与 `atomgit_provider_factory()`。ACP / daemon / clix 都通过它注入。
- 配置(`config/src/config/provider.rs`)已支持:`base_url`、`api_key`(支持
  `$VAR` / `${VAR}` / `${VAR:-default}` 展开,并按 type 回落
  `OPENAI_API_KEY` / `ANTHROPIC_API_KEY` / `OLLAMA_API_KEY` / `RUSTCODE_API_KEY`)、
  `extra_headers`、`proxy`、`skip_tls_verify`、`user_agent`、
  `retry_max_attempts`、`thinking_*` / `reasoning_*` 系列、`capable_model`、
  `supports_vision`、`context_window`、`max_tokens`。
- 模型别名由 **account / profile 双层 schema** 承担:`ProviderAccountConfig`
  持有连接与凭据,`ModelProfileConfig.model` 是线上名,选择 id 形如
  `<account>/<model-or-alias>`;解析结果统一收敛为 `ResolvedModelConfig`。
- AtomGit 网关签名由 `is_atomgit_gateway(base_url)` 门控:仅 `atomgit`/`relay`
  host 走上游签名器,其余自托管/第三方端点用 `bearer_auth(api_key)`。
- SSE chunk 差异统一映射为内部 `StreamEvent`(见 `provider/reasoning.rs`、
  `provider/mod.rs`);Anthropic 的思考块签名经 `StreamEvent::ReasoningSignature`
  原样回传。

已知缺口(补在 `docs/phase1-refactor-design.md` 第 3 节,不在 trait 层动刀):

- **`ProviderConfig` 没有 `timeout` 字段**;超时仅存在于适配器内部默认
  (如 `OpenAiCompatConfig::idle_timeout`)。
- 无显式 `model_mapping` 表;无按 provider 的 `connect_timeout` /
  `request_timeout` / 退避策略配置。
- 无统一的强类型错误映射器:现状是 `kernel::stream::ProviderError`
  (`retryable` / `http_status` / `code` / `retry_after_secs`)裸传,缺少
  `thiserror` 模块错误与"是否可重试"的集中判定。

**[ERROR] 禁止在 kernel 之上再叠第二套 `LlmClient` trait。** 那会与"单一状态
所有权 / 不得新增重叠抽象"冲突,并让三个已验证适配器变成死代码。解耦落点是
**配置 + 装配 + 错误映射**,即在 `capabilities` 层加 `llm/` 装配模块产出
`Arc<dyn LlmProvider>`,复用既有 `CodingProviderFactory` 与 `reassemble_provider`
热切换命令,不新增命令。

### [OBJECTIVE-4] License 与合规 — [DONE, 有缺陷]

- 根 `LICENSE` 为 MIT,双版权行 `Copyright (c) 2026 Yubang Xu` +
  `Copyright (c) 2026 The rustcode authors (fork of atomcode)`,正文声明本仓库是
  `gitcode.com/SecLab/RustCode` 的 fork。
- `docs/ORIGINAL_LICENSE.md`、`docs/THIRD_PARTY_NOTICES.md`、
  `docs/UPSTREAM_CREDITS.md` 三件套已就位。
- 新模块头部只追加本 fork 声明,**不得覆盖或删除任何既有版权行**。

**[ERROR] 待修缺陷**:`docs/ORIGINAL_LICENSE.md` 归档的是 **Yubang Xu /
SecLab/RustCode** 的 MIT 全文,而它正是本仓库自身;真正上游
`atomgit_atomcode/atomcode` 的许可全文没有独立归档,只在括号里提了一句。
应改为:根 `LICENSE` = 本项目 MIT(保留 Yubang Xu 版权行)+ fork 声明;
`ORIGINAL_LICENSE.md` = 上游 atomcode 的 MIT 全文。**拿不到上游原文就改名为
`docs/UPSTREAM_RUSTCODE_LICENSE.md` 并新建占位说明,绝不臆造许可文本。**

## 当前架构事实

coding agent 目标调用链:

```text
CLI / TUI / daemon / background / ACP / clix code
                    |
                    v
       CodingRuntimeHandle / DriverCommand
                    |
                    v
               CodingRuntime
                    |
                    v
          rustcode-kernel Agent
```

- `CodingRuntime` 是 coding agent 运行时所有者;driver 不应重建第二套 live
  agent 生命周期。
- kernel `AgentCommand/AgentEvent` 是运行时执行边界。coding 产品 driver 用
  `CodingRuntime`;其他业务 driver 可驱动其 L2 已装配的 kernel agent,但不得
  另建第二生命周期 owner,也不得把 provider/session/cd/goal/loop 等 coding
  生命周期塞回 kernel 命令。
- 上游 core legacy `AgentClient`/v1 engine/`atomcode-bridge` 已退役;`atomcode-core`
  crate 已从 workspace 删除。生产代码不得重建同名兼容层。历史 core JSON 只由
  daemon 私有 DTO 单向导入,禁止恢复 legacy writer、core 磁盘投影或双向转换。
- `rustcode-kernel`/`rustcode-capabilities`/`rustcode-coding` 生产依赖必须保持
  core-free;尤其禁止 capabilities 反向依赖 core、L2 或前端。
- native `SessionManager/SessionMeta/SessionSnapshot/PresentationFile` 是唯一
  session 持久化模型;daemon/TUI 投影必须直接用 kernel/coding 中立类型。

## 架构方向

- 单一状态所有权、清晰依赖方向、可验证兼容性;已删除的 core 不得以 facade、
  兼容 crate 或复制状态所有者方式回流。
- driver 负责输入/展示/传输/本地操作;coding runtime 负责业务生命周期;
  kernel 只负责中立 agent 循环;capabilities 提供可复用能力实现。
- 新能力优先放入职责正确的现有层;不得为去 core 新建无边界"杂物 crate"。
- 不预设创建 `rustcode-protocol` 或大而全 `rustcode-foundation`;只有出现稳定
  跨进程 schema 或独立版本契约时才拆叶子。
- 问题修复必须检查同一状态所有权、协议边界和受影响 driver,优先修共同根因。

## Runtime 生命周期不变量

涉及 submit/steer/cancel/approval/request/compact/provider reload/session
resume/fresh/undo/cd/goal/loop/shutdown 时,必须检查:

- live `AgentHandle`、config、parts、provider、session binding、generation、
  pending request、snapshot broker、controller 是否仍由单一 runtime owner 管理;
- session id、working目录、snapshot、provider 选择、审批 grant、gateway affinity、
  持久化目标在重建前后是否保持;
- build/prepare/assemble/restore 任一步失败时是否显式失败/回滚,而非静默 fresh、
  空 snapshot、noop handle 或假成功;
- pending approval/request 在 cancel/reload/session switch/shutdown 时是否 fail-closed;
- 旧 generation 迟到事件是否污染 replacement runtime;
- 每个 accepted operation 是否都有 success/error/cancel/replace/shutdown 终态;
- goal/loop 互斥、evaluation、held turn、delay/wakeup、cancel、turn terminal 是否
  属同一生命周期;
- snapshot 运行时权威来源是否明确,历史 core 数据仅作 importer/兼容输入。

涉及 turn completion 或 compaction 时,先复核 `LifecycleHooks::turn_complete`、
kernel 终止路径与 `rustcode-capabilities` compaction 实现;无证据证明 seam 缺失前
不得新增重叠 hook 或第二压缩状态机。

## 历史兼容面维护

`atomcode-core` 已退役;后续兼容只允许围绕仍保留的单向 importer 与明确 wire DTO:

1. 明确当前数据/状态的唯一 owner;
2. 找全持久化格式与兼容入口;
3. 历史格式只能作边界清晰、可测试的单向 importer;
4. importer 消费者归零后删除对应 DTO/转换/测试;
5. 禁止恢复 legacy writer、双向转换、运行时 fallback 或新 core facade。

兼容收口以"减少一个 importer/数据模型/转换链/fallback"为度量;不得以移动文件、
加 facade、建新 crate 或净删行数冒充架构进度。

## 兼容面迁移与退役判定 (四态)

只适用于正在删除的旧协议/旧格式/旧 API/fallback,不要求普通功能开发套用:

1. **逻辑已实现**:新 owner 已有能力;
2. **消费者已切换**:目标 driver/服务已用新路径;
3. **legacy fallback 仍保留**:旧入口/旧格式写入/旧 handler/回退仍可达;
4. **legacy 接口面已退役**:旧调用点/类型/handler/依赖/fallback 已删除并通过验证。

仅第 4 态可称"已退役"。兼容格式可读取但已独立单向 importer 时,必须报告
importer 仍保留,不得称格式已删除。

退役任务必须基于当前代码检查并报告:生产发送点/处理方/事件消费者/持久化读写方;
CLI/TUI/daemon/headless/background/ACP/clix 实际受影响入口;旧类型/handler/
feature flag/fallback/依赖是否仍可达;被删除/迁移/仍保留的测试;新旧失败/取消/
恢复/降级语义。

## 编码约束 (fork 硬性)

- [STYLE] 严禁 Unicode Emoji;状态/日志/注释/输出一律 ASCII 标签。
- [ASYNC] 保持 tokio async/await 一致性;async 链路禁止阻塞 I/O。
- [SECURITY] 严禁硬编码真实 API Key/Token/敏感内部端点;默认配置与测试用
  `env:VAR` 或占位符。
- [STREAMING] LLM client 必须原生支持 SSE 流式与非流式;Anthropic 与 OpenAI
  chunk 差异统一映射为内部 `StreamEvent`。
- [ERROR] 用 `thiserror` 定义模块级强类型错误,顶层用 `anyhow` 包装;禁止裸
  `.unwrap()`/`.expect()` 导致 panic(测试与明确不变量处除外)。
- [DEP] 清理遥测后同步检查 `Cargo.toml`/`Cargo.lock`,最小化 `reqwest`/`tokio`
  等共用依赖 feature(禁用无用 telemetry/logging feature)。

## 修改前检查

普通局部修改按风险执行最小检查。涉及 runtime 生命周期、公共协议、持久化、安全
边界、跨 crate 依赖或兼容面退役时,开始修改前必须:

1. 记录当前 branch、commit SHA 和 worktree 状态;
2. 搜索目标符号的生产方、消费者、持久化点和转换边界;
3. 查看相关文件近期 Git 历史,确认任务未已实现或改变方向;
4. 写明状态 owner、目标边界和失败语义;
5. 若为退役任务,写明预计删除的旧 surface,而非仅新增内容。

发现 dirty worktree 时保留用户改动;不擅自重置/覆盖/删除或借架构任务重构无关代码。

## 验证与交付

- 修改过程中运行最小相关测试;一个逻辑单元完成后跑受影响 crate 测试。
- 仅当变更跨 crate/公共协议/持久化/workspace 依赖或构建配置时,跑相关 workspace 检查。
- `cargo test` 已覆盖相同编译验证时,不紧接着重复 `cargo check`;代码未变化不重跑。
- 纯文档/注释/格式修改可不跑测试,但必须检查 diff 与文档内部一致性。
- runtime/协议迁移按实际影响覆盖 CLI/TUI/daemon/headless/background/session resume/
  approval/cancel/provider reload/持久化兼容;不受影响入口无需机械重复验证。
- 退役任务最终说明必须含:验证基线、四态、实际删除项、仍保留的 importer/legacy
  surface、测试结果、唯一下一步。
- 普通功能/修复只报行为变化、风险、验证结果、已知未验证范围,不强制套迁移模板。
- 未删除旧类型/handler/依赖/fallback 时,必须明确"尚未退役",不得以功能可用替代完成。

## PHASE-2 执行计划与子 Agent 分工

完整方案在 `docs/phase1-refactor-design.md`。要点:

- **批内并行、批间串行**。AGENT-A(重命名)、AGENT-B(零遥测)、AGENT-C(LLM
  装配)都会触碰 `cli/src/main.rs`、`clix/`、`coding/src/provider_factory.rs`,
  同时并发必然冲突。顺序:BATCH-1 合规与基线冻结 -> BATCH-2 重命名(A 独占)
  -> BATCH-3(B 与 C 并行,无重叠文件)-> BATCH-4 `clix/tel.rs` 删除与 review
  provider 迁移(与 B 同批,单独做)-> BATCH-5 AGENT-D 收口。
- **高风险项必须显式决策,不得默认**。`--no-telemetry` 删除涉及老 IDE 扩展的
  启动参数;`RUSTCODE_*` 旧环境变量是否保留一期"读旧名 + 告警"的兼容。若日后
  真要再做品牌切换,还会牵连:会话目录迁移(丢数据)、daemon 端口
  `13456/13457/13458`(IDE 扩展与手机端契约)、发布产物前缀与升级 manifest
  (自更新路径)。
- `extensions/`(vscode / jetbrains)仍用 `atomcode` 前缀且**不属于 Rust 工作区**,
  单列子任务,不阻塞主线,但交付说明必须写明"两侧命名尚未统一"。
- 工作区当前 dirty,批量改动前先 `git stash push -u` 并在独立分支进行。

## 门禁 (AGENT-D)

```text
G1  cargo fmt --check
G2  cargo clippy --workspace --all-targets -- -D warnings
G3  cargo test --workspace
G4  ./scripts/test-headless.sh                    (需先 cargo build)
G5  python3 scripts/acp_smoke.py
G6  grep -ri "sentry|posthog|segment|analytics" --include=*.rs --include=*.toml   必须 0 命中
G7  grep -rn "rustcode|RustCode|RUSTCODE" crates/ scripts/ .github/               重命名完成后 0 命中
G8  grep -rn "atomcode" AGENTS.md docs/architecture.md                            必须 0 命中
```

**[ERROR] CI 现状缺口**:`.github/workflows/build.yml` 只有 release 构建 job
(macOS / Linux-musl / Windows × x64 / arm64 + distro-pm 检查),**没有 fmt /
clippy / test job**。G1-G3 目前只能靠本地执行,建议 BATCH-5 补一个 `ci.yml`。

Mock Provider 测试基建(已具备,无需新建):`wiremock = "0.6"` 是 capabilities
的 dev-dependency;已有 `tests/{http_mock,anthropic_mock,ollama_mock}.rs` 与
`tests/fixtures/*.jsonl` 语料;`kernel` 的 `test-support` feature + `testkit.rs`
可装配测试用 agent。新增落点:`tests/llm_factory.rs`(model mapping / 协议分发 /
错误分类表)、`tests/llm_wire_parity.rs`(同一段脚本化响应分别喂 OpenAI 与
Anthropic mock,断言产出**同一** `StreamEvent` 序列)。

## 已知剩余项 (follow-up, 非重命名必须)

- `site/`(含 `site/docs/zh/*`)、`extensions/`(vscode/jetbrains)、`docs/zh`
  站点文档仍用旧二进制名/旧目录;不阻塞编译与运行,按需清理。
- `examples/hooks/*`、`docs/config.example.toml` 的 `.atomcode`/`atomcode` 引用
  已随本轮修复为 `.rustcode`/`rustcode`。
- `cargo clippy` 仍有 per-crate warnings(未用变量、命名),非 errors;可择机
  `cargo clippy --fix` 收敛,但不得借清理引入行为变更。
- `docs/architecture.md` 全篇仍是 `atomcode-*` 命名,与磁盘上的 `rustcode-*`
  不一致;引用前以目录真实内容为准,并纳入 BATCH-2 一并修正。
- **[CHECK] 产品身份已锁定为 `rustcode`**(见 `docs/REFACTOR_DESIGN_PHASE1.md`
  §2.0 决策 D1),并已在 commit `6dbf57bb` 落地。不要再提议或先行改名;`rustcode`
  就是当前品牌,`atomcode` 是历史名。O1 剩余项只有 `extensions/`(vscode /
  jetbrains)、`site/`、`docs/architecture.md` 的旧前缀清理。
- 仓内有两份 PHASE-1 设计文档:`docs/REFACTOR_DESIGN_PHASE1.md`(基线
  `main`/`287bff70`,重命名之前)与 `docs/phase1-refactor-design.md`(基线
  `dev`/`6dbf57bb`,重命名之后的复核版)。**冲突时以后者为准**;前者保留作历史
  决策记录(D1 等)。
