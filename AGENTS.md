# RustCode 开发约束 (面向 Agent)

本文件是二次开发 fork 的长期约束与高信号上下文。历史方案、旧基线和上游 core 退役过程留在 `docs/`。

**维护规则:文件内容变更时同步更新,不得滞后。** 若与代码冲突,以代码为准,先修正文档。

`rustcode` 在本文件与 `docs/architecture.md` 中均为**历史名引用**,不代表现役 crate。现役 crate 一律 `rustcode-*`。

## 常用命令

构建:
- `cargo build` — 构建 `default-members`(`rustcode-cli` / `rustcode-daemon` / `rustcode-tuix`),产物 `target/debug/rustcode`。
- `cargo build --release -p rustcode` — 只构建发布版 CLI。`rustcode-cli` 的**包名是 `rustcode`**(`-p rustcode-cli` 会失败),产物 `target/release/rustcode`。
- `cargo build --workspace` — 构建全部 `crates/*`(含默认成员外的 `clix`、`review`、`codingplan`)。
- `cargo install --path crates/rustcode-cli --locked` — 安装到 `~/.cargo/bin`。
- WebUI 需先构建前端:`cd webui && npm ci && npm run build` 产出 `webui/dist/`(gitignored)。重建前端后必须 `cargo clean -p rustcode-daemon`,cargo 不追踪 `webui/dist/` 变化;缺失时所有 webui 页面返回 `webui not built`。
- 发布打包:`scripts/release*.sh`、`scripts/macOS-release-*.sh`、`scripts/linux-release-*.sh`、`scripts/sign-macos.sh`;矩阵见 `.github/workflows/build.yml`。
- `RUSTCODE_HOME` 覆盖配置目录(默认 `~/.rustcode`)。**禁止 `sudo` 运行**——`~/.rustcode` 一旦出现 root 属主文件,后续非 root 启动在 runtime 初始化即失败。

测试:
- `cargo test` — 默认成员;`cargo test --workspace` — 全量(跨 crate 改动必跑)。
- `cargo test -p rustcode-capabilities` — 单 crate;`cargo test -p rustcode-coding <filter>` — 按测试名过滤;`--` 后接 `--nocapture` / `--test-threads=1`。
- `cargo test -p rustcode-kernel --test <name>` — 只跑某个 `tests/*.rs`;`--lib` 只跑内联单测,`--bins` 只跑 bin 内测试。
- `./scripts/test-all.sh` — 全量测试并产出 `test-report.md`,能区分"编译失败 / 测试失败 / 全部通过"。
- `./scripts/test-headless.sh` — headless 冒烟(需先 `cargo build`);未设 `RUSTCODE_TEST_PROVIDER` 时跳过联网用例。
- `./scripts/smoke-test-all.sh`(校验 test-all.sh 自身)、`python3 scripts/acp_smoke.py`(ACP stdio 冒烟)、`python3 scripts/analyze_datalogs.py`(turn datalog 分析)。
- 测试隔离:`coding` / `tuix` / `daemon` / `capabilities` / `cli` 的入口文件顶部有 `#[ctor]` 把 `RUSTCODE_HOME` 重定向到临时目录。新增测试不得依赖真实 `~/.rustcode`;**重命名这批目录/变量名时同步修改,否则测试隔离失效**。

Lint / 格式:
- `cargo fmt` / `cargo fmt --check`;提交前 `cargo clippy --workspace --all-targets`。
- `cargo check --workspace --all-targets` — 快速编译校验;`cargo test` 已覆盖相同编译验证时不再重复 `cargo check`。
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

- **出站 HTTP 只有一个入口**:`capabilities/src/egress/`(`egress` feature,由 `provider` / `web` / `atomgit` / `mcp` 拉起)。`egress::client::build_http_client` 是唯一工厂,统一承载信任根分层、代理策略、超时、UA、pool-idle。**新增任何出站调用都必须走它,禁止再写 `reqwest::Client::new()`。**
- 依赖只向下:`kernel`(无内部依赖) <- `capabilities`(禁止反向依赖 coding / driver / 已退役 core) <- `coding`(另依赖 kernel、config、review) <- `tuix`(另依赖 daemon、updater、codingplan、auth) <- `cli`(唯一同时依赖 tuix + daemon)。`clix -> review, coding, capabilities, kernel, config`。
- 工作区 `members = ["crates/*"]`,`default-members` 为 cli / daemon / tuix;`rustcode-codingplan-crypto` 是闭源签名占位桩,默认成员故意不含它,官方构建用 `--features rustcode/codingplan-crypto` 开启。
- **`rustcode-codingplan` 的网关 HTTP client 默认不链接**:`client` / `setup` 模块(以及 reqwest 依赖)由 `client` Cargo feature 门控、默认 `default = []`;纯数据/usage 模块(`types` / `usage` / `sync_marker`)始终编译,供 TUI 用量面板在无网关 client 时也能渲染形状。驱动 crate 的 `codingplan` feature **必须**向上传递 `rustcode-codingplan/client`(daemon: `codingplan = ["dep:rustcode-codingplan", "rustcode-codingplan/client"]`;cli 另加 `rustcode-tuix/codingplan`),否则 `#[cfg(feature="codingplan")]` 代码会引用被 cfg 掉的 `Client`/`run`/`is_auth_expired` 而编译失败。注意:默认 feature 下 `cargo test -p rustcode-codingplan` **不编译** `client.rs`/`setup.rs`,改动这两个文件后要显式 `cargo check -p rustcode-codingplan --features client --all-targets`。跨 crate 复用 `format_duration_secs` 走 crate 根重导出(`rustcode_codingplan::format_duration_secs`),`setup::format_duration_secs` 是私有 `use`,外部不可达。
- **`CodingRuntime`**(`coding/src/runtime.rs`)是 coding agent 的唯一运行时所有者,对外暴露 `CodingRuntimeHandle`。Driver 通过 `DriverCommand` 驱动,读 `CodingRuntimeEvent`;不得自建第二套 live agent 生命周期。
- 启动路径:CLI `spawn_native_cli_runtime`、ACP `spawn_native_runtime_for_session_deferred_with_preprocessor`、daemon `kernel_runtime::start_native_runtime*`,三者最终都落到 `CodingRuntime::start_with_session_lease` / `start_with_bootstrap`。**TUI 不自己启动 runtime**:外部把已启动的 `SpawnedRuntime` 传进 `tuix::run`。
- kernel `Agent` + `AgentBuilder` / `AgentHandle` / `AgentCommand` / `AgentEvent` 是中立循环,不得承载 coding 产品语义。`rustcode-review` 是独立业务 agent,以 `code_review` 子 agent 工具挂进 coding。
- 运行时领域术语(Live View、Runtime Generation、Session Transition、Tool Catalog Revision、Committed Snapshot、Replay Window)**以 `CONTEXT.md` 为准**(注意:该文件使用旧 `RustCode` 命名,但术语定义仍有效),其中 `_Avoid_` 条目是硬性命名约束。

## Fork 重构目标与完成状态

本仓库是 `SecLab/RustCode` 的二次开发 fork,核心约束:禁止使用任何 Unicode Emoji,日志/注释一律用 ASCII 标签 (`[INFO]`/`[WARN]`/`[ERROR]`/`[SUCCESS]`/`[CHECK]`/`[+]`/`[-]`/`[*]`)。

### [OBJECTIVE-1] 产品重命名 — [DONE]

14 个 crate 全部 `atomcode-*` -> `rustcode-*`。配置目录 `~/.rustcode`;环境变量前缀 `RUSTCODE_*`。

重命名的事实源集中在两个文件(改这两处即可带动大部分):
`rustcode-config/src/distribution.rs`(`HOME_ENV` / `HOME_DIR_NAME` / 端口 `13456,13457,13458` / `PROCESS_NAMES` / `RELEASE_ASSET_PREFIX` 等)与 `rustcode-config/src/endpoints.rs`(11 个 `RUSTCODE_*` 环境变量名 + 4 个托管端点)。

**[CHECK] `rustcode` 是已锁定的产品身份**(决策 D1,见 `docs/REFACTOR_DESIGN_PHASE1.md` §2.0),不是中间态;不要再次改名。O1 剩余工作只有 `extensions/` / `site/` / `docs/architecture.md` 的旧 `rustcode` 前缀收尾。

### [OBJECTIVE-2] 零遥测 — [DONE, 仅文档口径残留]

`rustcode-telemetry` crate 已删除;`Telemetry`/`Event`/`track`/`install_panic_hook` 等上报调用已移除。**全仓不存在 Sentry/PostHog/Segment/GA 等第三方埋点 SDK 或依赖**。崩溃处理仅保留 stderr 输出。

文档口径已对齐:`docs/telemetry.md` 已改为"Telemetry — removed"说明页;`README.zh-CN.md` 零遥测声明与事实一致;`site/docs/{en,zh}/headless-daemon.html` 中 `--no-telemetry` 标注为"accepted and ignored"。kernel 与 capabilities 注释里仍有 `telemetry` 字样(纯注释,无上报,属词义歧义)。

**[CHECK] 客户端身份现状(不得误删)**:daemon 的 `ClientMode`(`daemon/src/client_mode.rs`)是本地分支逻辑,不是遥测;`RepoOrigin` / `detect_repo_origin` 已迁至 `rustcode-config/src/session_mode.rs`(纯字符串解析,无网络)。

### [OBJECTIVE-3] LLM Provider 解耦 — [DONE, 平台中立]

- 统一 trait `LlmProvider`(`kernel/src/provider.rs`),`async_trait`,唯一方法是 `chat_stream` -> `BoxStream<'static, StreamEvent>`。
- 三个适配器在 `capabilities/src/provider/`:`anthropic.rs`、`openai_compat.rs`、`ollama.rs`。
- 工厂是 **trait** `CodingProviderFactory`(`coding/src/provider_factory.rs:74`),默认实现按 `provider_type` 分发。ACP / daemon / clix 都通过它注入。
- 配置(`config/src/config/provider.rs`)已支持:`base_url`、`api_key`(支持 `$VAR` / `${VAR}` / `${VAR:-default}` 展开)、`extra_headers`、`proxy`、`skip_tls_verify`、`retry_max_attempts`、`thinking_*` / `reasoning_*` 系列等。
- **平台中立**:默认不绑定任何平台。`is_codingplan_llm_gateway` 不再硬编码 host,仅当操作者显式配置 `RUSTCODE_CODINGPLAN_LLM_BASE_URL` 时才识别为签名网关;否则所有 provider 走纯 `bearer_auth(api_key)`。`/login` 在无 `RUSTCODE_PLATFORM_SERVER` 时提示用户直接配置 provider。AtomGit REST 工具(`atomgit_repo/pr/issue`)由 `atomgit` Cargo feature 门控,默认成员不启用。
- 强类型错误分类器已就位:`capabilities/src/provider/error.rs` 的 `LlmError`(thiserror,`retryable()` 单点判定);**新代码必须经 `LlmError` 转换,存量按 `docs/phase1-refactor-design.md` 第 3 节渐进迁移**。

**[ERROR] 禁止在 kernel 之上再叠第二套 `LlmClient` trait。** 解耦落点是**配置 + 装配 + 错误映射**,即复用既有 `CodingProviderFactory` 与 `reassemble_provider` 热切换命令。

### [OBJECTIVE-4] License 与合规 — [DONE]

根 `LICENSE` 为 MIT,双版权行 `Copyright (c) 2026 Yubang Xu` + `Copyright (c) 2026 The rustcode authors (fork of rustcode)`。新模块头部只追加本 fork 声明,**不得覆盖或删除任何既有版权行**。

### [OBJECTIVE-5] 默认中文与平台残留收尾 — [DONE]

- **默认语言为简体中文**。事实源:`rustcode-config/src/locale.rs` 的 `Default for Locale` 与 `i18n/mod.rs`(static `LOCALE` 初值、`current_locale` 毒锁回退、`resolve_initial_locale_with_env` 终值)三处默认均为 `Locale::ZhCn`。优先级仍是 CLI `--lang` > config `language` > `LC_ALL/LC_MESSAGES/LANG` > 默认。`LANG=C`/`POSIX`/空值视为"无偏好"→ 中文;显式但不支持的 locale(如 `fr_FR`)→ 英文回退。WebUI(`webui/src/settings.tsx` `readLang`)默认本就是 `zh`。
- Agent 对话默认语言:persona 装配时按 `preferred_language` 注入 `## LANGUAGE` 段(`coding/src/persona.rs` `conversation_language_guidance`),用户语言不明时中文回复;显式英文 locale 则英文。
- **扩展旧名清零**:`extensions/vscode`(`_atomCode*Watcher` 字段、测试名、package-lock 根 name)与 `extensions/jetbrains`(`commonAtomcodePaths`/`guardAtomcodeHome`、临时目录前缀)已全部改为 rustcode;`packages/*` 无旧名。
- **平台残留注释泛化**:oauth/tls/proxy/codingplan/persona/kernel 等处硬编码 `*.atomgit.com` 的注释改为"managed endpoint"中性表述;`friendly_http_error` 的 403 提示去掉 `/login` 引导,改为"检查 API key 权限与账户状态";测试夹具 URL 改 `example.com`。OAuth loopback 回调的**错误分支不再 302 跳转 `atomgit.com`**,改为与成功分支同构的本地中性 HTML 错误页(本构建无附属平台站点);`strip_force_login` 等测试夹具 URL 改 `example.com`。`atomgit` Cargo feature(REST 工具、`api.atomgit.com/api/v5` 装配、push-label 中间件)整体 `#[cfg(feature = "atomgit")]` 门控、默认成员不启用——这是刻意保留的上游开关,不是残留;生产代码中其余 `AtomGit*` 字样仅出现在旧前缀兼容逻辑(`is_codingplan_provider_name`)及其测试夹具中。
- **遥测注释收尾**:失实的"telemetry-tracked/telemetry sink"注释已改为实际行为;纯 hook seam 注释(datalog/cache-RCA 可挂载点)保留,无上报逻辑。`openai_compat.rs` UA 注释中的 "analytics" 措辞已改为中性的路由/缓存说明。**G6 复核为 0 命中**(sentry/posthog/segment/analytics,含 extensions)。
- **tuix 测试编译修复**:`event_loop/mod.rs` 测试模块中历史机械重命名残留 `atomgit_configcodingplan_config("model-b")`(E0425,函数不存在)已改为 `codingplan_config("model-b")`。此修复让 tuix lib-test 重新能编译,也因此暴露了一批存量红测试(见已知剩余项)。

### [OBJECTIVE-6] 子代理并发双车道(worker / explore 独立信号量)— [DONE]

- 写车道与只读车道分离:`task` 工具(`tools/task.rs`,`TaskTool::max_concurrent` vs 新增 `max_concurrent_explore`,builder `with_max_concurrent_explore`)与 team 运行时(`team/manager.rs` `TeamRuntimeConfig` 同名字段,`delegate` 按 `TeamPermission::Worker` 分配 worker semaphore、其余分配 explore semaphore)各自持有独立信号量。
- 默认值:worker 车道保守(3,scope-confined 写活树);explore 车道更宽(8,只读不冲突,排查扇出不被 worker 预算节流)。
- 配置面:`[subagent]` 表 `max_concurrent`(写,默认 3,env `RUSTCODE_SUBAGENT_MAX_CONCURRENT`)与新增 `max_concurrent_explore`(只读,默认 8,env `RUSTCODE_SUBAGENT_EXPLORE_MAX_CONCURRENT`),serde 默认函数 `default_subagent_worker_concurrent`/`default_subagent_explore_concurrent`;装配与 env 解析在 `coding/src/parts.rs`(`subagent_runtime_knobs` 返回三元组)。
- 测试:`team/manager.rs` 双屏障/峰值并发测试证明两车道独立(worker=1 串行、explore=4 同发);`tests/team_runtime.rs` 与 `config/mod.rs` 断言默认值。
- **内置并行模板(默认开启)**:`SubAgentConfig.parallel_template`(默认 `true`,`/config` 可切,`ApplyPolicy::CapabilityReprepare`,镜像 `/think` 实时开关模式)。开启时 `resolve_external_subagents`(`coding/src/parts.rs`)自动挂载三个外部代理角色——`explorer`(codex / read-only)、`builder`(claude-code / accept-edits)、`reviewer`(codex / read-only)——并把 worker 车道并发下限提到 4;显式 `[[subagent.external]]` 同名条目优先(覆盖内置角色);daemon/headless 传 `allow_dangerous_context=false`(bypass 降级 fail-closed)。接入点:CLI/TUI(`cli/main.rs` 传 true)、clix(`code.rs`)、daemon(`kernel_runtime.rs` 传 false);ACP 通道(`acp/engine.rs`)用 `PrepareOptions::default()` 不挂外部角色。

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

- `CodingRuntime` 是 coding agent 运行时所有者;driver 不应重建第二套 live agent 生命周期。
- kernel `AgentCommand/AgentEvent` 是运行时执行边界。coding 产品 driver 用 `CodingRuntime`;其他业务 driver 可驱动其 L2 已装配的 kernel agent,但不得另建第二生命周期 owner。
- 上游 core legacy `AgentClient`/v1 engine/`rustcode-bridge` 已退役;生产代码不得重建同名兼容层。历史 core JSON 只由 daemon 私有 DTO 单向导入。
- `rustcode-kernel`/`rustcode-capabilities`/`rustcode-coding` 生产依赖必须保持 core-free。
- native `SessionManager/SessionMeta/SessionSnapshot/PresentationFile` 是唯一 session 持久化模型。

## 架构方向

- 单一状态所有权、清晰依赖方向、可验证兼容性;已删除的 core 不得以 facade、兼容 crate 或复制状态所有者方式回流。
- driver 负责输入/展示/传输/本地操作;coding runtime 负责业务生命周期;kernel 只负责中立 agent 循环;capabilities 提供可复用能力实现。
- 问题修复必须检查同一状态所有权、协议边界和受影响 driver,优先修共同根因。

## Runtime 生命周期不变量

涉及 submit/steer/cancel/approval/request/compact/provider reload/session resume/fresh/undo/cd/goal/loop/shutdown 时,必须检查:

- live `AgentHandle`、config、parts、provider、session binding、generation、pending request、snapshot broker、controller 是否仍由单一 runtime owner 管理。
- session id、working目录、snapshot、provider 选择、审批 grant、gateway affinity、持久化目标在重建前后是否保持。
- build/prepare/assemble/restore 任一步失败时是否显式失败/回滚,而非静默 fresh、空 snapshot、noop handle 或假成功。
- pending approval/request 在 cancel/reload/session switch/shutdown 时是否 fail-closed。
- 旧 generation 迟到事件是否污染 replacement runtime。
- 每个 accepted operation 是否都有 success/error/cancel/replace/shutdown 终态。

涉及 turn completion 或 compaction 时,先复核 `LifecycleHooks::turn_complete`、kernel 终止路径与 `rustcode-capabilities` compaction 实现;无证据证明 seam 缺失前不得新增重叠 hook 或第二压缩状态机。

## 历史兼容面维护

`rustcode-core` 已退役;后续兼容只允许围绕仍保留的单向 importer 与明确 wire DTO:

1. 明确当前数据/状态的唯一 owner;
2. 找全持久化格式与兼容入口;
3. 历史格式只能作边界清晰、可测试的单向 importer;
4. importer 消费者归零后删除对应 DTO/转换/测试;
5. 禁止恢复 legacy writer、双向转换、运行时 fallback 或新 core facade。

## 编码约束 (fork 硬性)

- [STYLE] 严禁 Unicode Emoji;状态/日志/注释/输出一律 ASCII 标签。
- [ASYNC] 保持 tokio async/await 一致性;async 链路禁止阻塞 I/O。
- [SECURITY] 严禁硬编码真实 API Key/Token/敏感内部端点;默认配置与测试用 `env:VAR` 或占位符。
- [STREAMING] LLM client 必须原生支持 SSE 流式与非流式;Anthropic 与 OpenAI chunk 差异统一映射为内部 `StreamEvent`。
- [ERROR] 用 `thiserror` 定义模块级强类型错误,顶层用 `anyhow` 包装;禁止裸 `.unwrap()`/`.expect()` 导致 panic(测试与明确不变量处除外)。
- [DEP] 清理遥测后同步检查 `Cargo.toml`/`Cargo.lock`,最小化 `reqwest`/`tokio` 等共用依赖 feature。

## 修改前检查

涉及 runtime 生命周期、公共协议、持久化、安全边界、跨 crate 依赖或兼容面退役时,开始修改前必须:

1. 记录当前 branch、commit SHA 和 worktree 状态;
2. 搜索目标符号的生产方、消费者、持久化点和转换边界;
3. 查看相关文件近期 Git 历史,确认任务未已实现或改变方向;
4. 写明状态 owner、目标边界和失败语义;
5. 若为退役任务,写明预计删除的旧 surface,而非仅新增内容。

## 验证与交付

- 修改过程中运行最小相关测试;一个逻辑单元完成后跑受影响 crate 测试。
- 纯文档/注释/格式修改可不跑测试,但必须检查 diff 与文档内部一致性。
- runtime/协议迁移按实际影响覆盖 CLI/TUI/daemon/headless/background/session resume/approval/cancel/provider reload/持久化兼容;不受影响入口无需机械重复验证。
- 退役任务最终说明必须含:验证基线、四态、实际删除项、仍保留的 importer/legacy surface、测试结果、唯一下一步。

## 门禁 (AGENT-D)

```text
G1  cargo fmt --check
G2  cargo clippy --workspace --all-targets -- -D warnings
G3  cargo test --workspace
G4  ./scripts/test-headless.sh                    (需先 cargo build)
G5  python3 scripts/acp_smoke.py
G6  grep -riE "sentry|posthog|segment|google-analytics|googletagmanager|mixpanel|amplitude"
    --include=*.rs --include=*.toml   必须 0 命中(rustcode-telemetry 已删除;
    代码里仅剩的 "telemetry" 字样是卸载时清理遗留 telemetry/ 目录名、以及"旧
    [telemetry] 配置段被忽略而非报错"的兼容测试,均非遥测采集)
G7  grep -rni "atomcode" crates/ scripts/ .github/   0 命中(描述已退役 core 的
    历史注释除外,须逐条人工确认确属历史名)。`atomgit` 字样仅允许三类:
    (1) opt-in `atomgit` Cargo feature(capabilities/src/atomgit/、tools/atomgit*、
        以及 persona/parts 里 `#[cfg(feature = "atomgit")]` 的工具装配/人设注入);
    (2) 旧 `AtomGit-*` provider 前缀兼容(is_codingplan_provider_name /
        LEGACY_CODINGPLAN_PREFIX,旧配置键仍需识别,勿删);
    (3) fork 自己的发行主页 gitcode.com/SecLab/RustCode。
G8  grep -rni "atomcode" docs/architecture.md   0 命中(描述已退役 core 的历史
    小节除外;AGENTS.md 本身保留重命名映射表与历史名引用,不参与此门禁)
```

**[CHECK] CI 已补齐**:`.github/workflows/ci.yml` 已创建,在 push/PR 到 `main`/`dev` 时触发 G1(`cargo fmt --check`)、G2(`cargo clippy --workspace --all-targets`)、G3(`cargo test --workspace`)三个 job。G2 暂不 `-D warnings`(约 420 条存量 warning,见下方已知剩余项),待收敛后收紧。`build.yml` 仍只管 release 构建。

## 已知剩余项 (follow-up, 非重命名必须)

- ~~`extensions/` 旧名~~ — **已清零**(vscode/jetbrains 均已改为 rustcode,见 OBJECTIVE-5)。`site/` 静态文档站此前残留大量 `gitcode.com/SecLab` install/clone/npm-scope/marketplace 链接与 `gitcode.host`/`gitcode.com` CDN 截图、`referral.html` 邀请奖励 SPA(硬编码厂商 ACS 后端)——**已于 2026-08-31 全部去平台化**(见下方发行链路条目):install 改"发行渠道获取 + `RUSTCODE_RELEASE_BASE=...`"、clone 改 `example.com`、npm scope 统一 `@rustcode/rustcode`、marketplace 改自有渠道、CDN 图改内联 SVG/data-URI、`referral.html` 替为中性占位页、`search-index.{zh,en}.json` 经 `node build-search-index.mjs` 重生成;站内颜色 emoji(✅/❌/📋/📎/💬)一并 ASCII 化。站内剩余 `*.rustcode.dev` 导航链接是本项目**自有重命名域名**(品牌),非厂商主机,保留。
- `cargo clippy` 仍有 per-crate warnings(未用变量、命名),非 errors;可择机 `cargo clippy --fix` 收敛。
- **[WARN] `mcp::registry::tests::trust_key_golden_matches_core_algorithm` 当前是红的**:`project_trust_key` 用 `std::collections::hash_map::DefaultHasher`,输出不保证跨工具链稳定。不要随手改测试去凑绿。
- **[DONE] `rustcode-capabilities` lib 存量红测试已收敛**(2026-08-31):
  - `cc_hooks::tests::turn_complete_payload_alignment`:测试自身缺陷——同一 `&&` 链中两个 `grep` 共享 stdin,首个 grep 抽干管道致第二个必失败;改为先 `cat > payload.json` 再对文件断言。
  - `tools::read::tests::large_{non_code,symbolless_code}_..._bounded_page`:断言停留在旧 300 行分页,实现已为 1500 行页(`DEFAULT_READ_LIMIT`);夹具改为 1600 行并对齐 `Continue with read_file({"limit":1500,"offset":1501})` 格式。
  - `subagent/claude_code`、`subagent/codex` 在全 workspace 高并发下偶发 `SpawnFailed("Text file busy (os error 26)")`(overlayfs/容器内新写脚本 execve 的 ETXTBSY 竞态,Go fork/exec 内置重试而 Rust std 没有):新增 `process_utils::spawn_retrying_etxtbsy`(8 次退避重试),接入 `subagent::proc::ManagedChild::spawn` 与 `cc_hooks::run_command_hook`。
- **[DONE] `rustcode-tuix` lib 存量红测试已全部修复**(2026-08-31,多 agent 协作):从 2008 passed / 65 failed 收敛到 **2053 passed / 0 failed**(`cargo test -p rustcode-tuix`;`rustcode-review` 100/0)。根因是平台中立化重构时 TUI 渲染串被**机械 ASCII 化**:Unicode 排版字形(分隔符 `·` U+00B7、省略号 `…`、输入尖括号 `❯`、菜单 `▸`、工具圆点 `●`、spinner `◐/⠋`、状态 `⚠/ⓘ/⏸`、树形 `└`、箭头 `→/↑/↳`、勾叉 `✓/✗`)被改成 ASCII 占位,而上游测试断言的是 Unicode。修复原则(遵循 `glyph.rs` 政策):**渲染器在 `unicode_symbols` 分支恢复 Unicode**,dumb terminal / legacy conhost / `LANG=C` 仍由 `glyph::downgrade_glyphs` / `ascii_for` 降级为 ASCII;测试夹具是 `en_US.UTF-8 xterm-256color`(`caps_with_color()` 推得 `unicode_symbols=true`),故断言对齐到 Unicode,**测试未被削弱**。注意 `·` U+00B7 刻意**不在** downgrade 表里(几乎所有终端都能渲染,无条件透传)。刻意保留 ASCII 的只有:替换表情符号的方括号标签(`[+]/[-]/[*]/[!]/[INFO]/[Y]es`)与 config 层 tagged 模板(`TurnSummary`/`LoopRound`/`[x]` 停止/拒绝等)。`rustcode-review/src/review_tool.rs` 同款 `·` 分隔符一并恢复。此批红测**不是默认中文改动导致**——断言英文串的测试均显式 `i18n::test_lock()` + `set_locale(Locale::En)`。
- **[DONE] 平台中立化收尾 + headless provider 预检**(2026-08-31,多 agent 协作):
  - **真实 bug 修复(G4 T5b)**:headless `-p` 在无 provider 或 `--provider <不存在>` 时,`ProviderBootstrap::Required` 会让 `provider_factory.build()` 以空 `base_url` 的 OpenAI 适配器"真空成功",随后在 reqwest 处报含义不清的 `relative URL without a base`。现于 CLI headless 路径加 `headless_missing_provider_message()` 预检:解析后 `runtime_cfg.model.is_empty()` 即 bail,给出可操作报错(指名请求的 provider、指向 config 路径、提示无参启动做交互配置),判定与 TUI onboarding 一致。新增单测 `headless_missing_provider_message_fires_only_without_resolved_model`;G4 T5b 由"reqwest 报错"变为"provider lookup 失败 exit=1"。
  - **docker 中立化**:`Dockerfile-Daemon-Tosslib` 基础镜像从内网 `swr.cn-north-4.myhuaweicloud.com/gitcode-be/...` 改为公共 `debian:bookworm-slim`;三个 Dockerfile 硬编码的 aliyun APT 源改为**默认空、按需 `--build-arg APT_MIRROR=` 注入**(默认走 Debian 官方源);`docker-compose.yml`/`docker/README.md` 内网镜像仓库地址改 `your-registry.example.com/...` 占位、"GitCode App" 改中性"移动端 agent 应用"、`⚠️` 改 `[WARN]`;`docker/config-example.toml` 默认 provider 从 openrouter 改为 `my-provider` + `api.example.com` 模板。
  - **扩展/前端/夹具**:`extensions/vscode` `/config` 帮助片段、`webui` 模型输入框 placeholder、`extensions/jetbrains` SSE 测试夹具 URL 从 deepseek 改为 `your-model-id` / `api.example.com`;`crates/rustcode-coding/examples/run_task.rs` 去掉 DeepSeek 默认端点,`RUSTCODE_BASE_URL`/`RUSTCODE_MODEL` 改为必填(未设则退出并提示);i18n 的 Base-URL 引导提示改 `api.example.com`。
  - **凭据/平台文档**:`docs/dev-env-setup.md`、`DEVENV.md` 原为 Huawei Cloud EulerOS + CodeArts 专用笔记,且**硬编码了疑似真实的 `CODEARTS_CLI_AK/SK`**(已全仓清除,grep 0 命中)。重写为平台中立:适用范围改通用 Linux/macOS,删 CodeArts/arkcli 等厂商云 CLI 步骤,tokenhub/hwdevspace/`openpangu` 网关示例改 `gateway.example.com` + `${MY_PROVIDER_API_KEY}` 环境变量注入,修正 `cargo run -p rustcode-cli`(包名实为 `rustcode`)为 `cargo run` / `-p rustcode`;`scripts/dev-env-quickstart.sh` 的 `✅` 改 `[SUCCESS]` 并同步修正 cargo 命令;`docs/codex-claude-config-analysis.md` 的 tokenhub 端点一并中立化。
  - **预设中立默认**:`provider_preset.rs` 的厂商预设(deepseek/openai/anthropic/qwen/ollama 等)与 `docs/config.example.toml` 的多 provider 示例都是**第三方 BYO 自带密钥**配置,正是"只保留第三方配置"目标要求**保留**的,不算平台绑定;平台绑定特指签名网关/托管端点(已统一为 example.com 占位或 `RUSTCODE_*_BASE_URL` 显式 env)。但预设**显示顺序**原先把 `taotoken`(某商业 token 转售商)放在索引 0,TUI 的 `preset_idx_by_id` 防御性 fallback(`unwrap_or(0)`)也会落到它身上。已把两个通用 `*-compatible` 自带端点预设移到 `PRESETS` 最前(索引 0 = openai-compatible),厂商预设随后;新增测试 `generic_endpoints_lead_the_registry` 锁定"索引 0 必须是中立自定义端点",TUI fallback 注释同步。`preset_or_compatible` 对未知 id 本就回落到 `OPENAI_COMPATIBLE`。
  - **运行时无厂商默认**:全仓排查确认生产代码路径**没有**硬编码默认 model/provider/base_url 会把用户导向某厂商——无配置时 headless 预检直接 bail(见上),预设全为 opt-in 手选;`retained.rs` 等测试夹具里的 `glm-5`/`deepseek-v4-flash` 仅作渲染显示数据,`vision.rs`/`kernel_runtime.rs`/`provider_factory.rs`/clix `SAMPLE` 里的厂商模型名均在 `#[test]` 内且端点为 `example.test`/`127.0.0.1` 桩,非默认值、不影响行为。
  - `.gitignore` 里 `.codeartsdoer/` / `codearts.zip` 是**防凭据入库的保护性忽略规则**,保留。
- **[DONE] 全仓 emoji ASCII 化扫尾**(2026-08-31,多 agent 并行):
  - **文档(子 agent)**:脚本化把 `docs/` 下 ~49 个 md 的装饰性/状态 emoji 转 ASCII——表格 `✅/❌→[+]/[-]`、编号完成清单 `✅→[x]`、banner `⚠️→[!]/[WARN]`、装饰 `✨/🎉→[*]`,并剥掉残留 U+FE0F;fenced 代码块与 TUI 字形设计稿受保护,表格管道数逐一核对不变。`hooks.md`/`hook-architecture.md`/`async-webhook-{summary,guide}.md`/`hook-expansion-summary.md` 等通用文档已 0 装饰 emoji。
  - **扩展/前端(子 agent)**:vscode/jetbrains 扩展与 webui 共 12 文件——`📋⚙🗑✎📄📁🔧⬆⚡☕📝🌐🐛🧪💬🧠🔍🔌🔟` 等图形 emoji 全部删除或改 ASCII 标签(`[#]/[X]/[EDIT]/[F]/[D]/[=]/[*]/[KEY]/[~]/[!]`),U+FE0F 残基清零;JetBrains 齿轮菜单生产标签在 `RustCodeBundle*.properties`(9/10 此前已迁移,本轮补齐 `gear.settings`),测试 `GearMenuLabelsTest.kt` 同步锁定到实际标签。
  - **顶层/CI(主 agent)**:`README.md`/`README.zh-CN.md` 贡献 bullets 与捐赠行、`extensions/vscode/README.md` 能力 bullets、`.github/workflows/build.yml` 步骤标记 `🔟→[10]` 均 ASCII 化;捐赠行顺带去掉"Coding Plan 免费"这类托管服务口径。
  - **刻意保留(有界例外)**:① TUI 终端状态点**设计稿**(`docs/superpowers/{plans,specs}/2026-07-03-terminal-status-glyph*.md` 与 `...-round-cap-checkpoint.md`)里的 `🟢/🟡/🔴` 是该彩色状态点特性的**规格主语义**(实际渲染为 `●` + ANSI 色),ASCII `[+]/[-]` 无法表达颜色,故保留——仅此 3 个视觉规格文件、仅这 3 个彩色圆点;② 排版类标记 `✓/✗/⚠(无 FE0F)/→/·/●/❯/☐/☑(线框)` 在浏览器/JCEF/终端等 unicode 能力端与 TUI 字形政策一致,保留;③ `tab-title-truncation.test.ts` 的 `🎉` 是被测输入;④ AGENTS.md 里 backtick 包裹的字形名是字形政策文档引用。新增 UI/文档一律用 ASCII 标签,不要再引入图形 emoji。
- **[DONE] 发行/安装/打包/CI 功能面去平台化**(2026-08-31,多 agent 协作):此前的中立化聚焦 LLM/relay/marketplace 运行时;本轮清掉**发行链路上仍硬编码的厂商主机/账号**,原则统一为"发行主机一律由 env / 运营方注入,构建里不带厂商默认":
  - **安装器** `scripts/install.sh`、`scripts/install.ps1`:删掉硬编码 `gitcode.com/SecLab/RustCode` 的 `REPO_BASE`/`REPO_LATEST_API` 与 `v5.0.2` 兜底 tag;改 `RUSTCODE_RELEASE_BASE`(必填,未设则 fail-closed 打印可操作指引)、`RUSTCODE_RELEASE_LATEST_API`(可选,用于 latest 自动探测)、`RUSTCODE_VERSION`。**删除 referral/invite 代码块**(写 `~/.rustcode/pending_invite` + install_uuid):遥测上报路径随 telemetry crate 删除后已无任何 Rust 读取方(grep 0),纯平台增长残留。`install.ps1` 失实注释 `crates/rustcode-core/...` 改 `rustcode-cli`;uninstall.sh/uninstall.ps1 头部 `curl|sh`/`irm` 厂商 URL 改本地/发行渠道口径。
  - **release 上传** `.github/workflows/create_tag_release.py`:删硬编码 `API_HOST=https://api.gitcode.com`、owner `bangxu`;改 `--api-host`/`--owner` 或 `RUSTCODE_RELEASE_API_HOST/OWNER/REPO/ACCESS_TOKEN`(GitLab-v5 兼容 releases/upload_url 方言),缺失即列出缺什么并退出。
  - **CI** `.github/workflows/build.yml`:删 4 处 "Add hosts" 步骤(macOS/linux/windows/distro-pm-check)——它们把 `api.gitcode.com`/`file.gitcode.com` 用 sudo 钉到厂商 IP `159.138.147.37`,是厂商 DNS 绕行;发行主机改为运营方注入后这些无意义。上传步骤改传 `secrets.RELEASE_API_HOST/RELEASE_OWNER/RELEASE_ACCESS_TOKEN`。
  - **打包脚本** `packages/npm/scripts/build_npm_package.sh`、`packages/homebrew/scripts/package-tar-gz.sh`:GitCode v5 API/下载根与 `SecLab/RustCode`、`GITCODE_TOKEN/OWNER/REF/VERSION/JQ_URL` 全部改 `RUSTCODE_RELEASE_*`(API_HOST/OWNER/REPO/REF/ACCESS_TOKEN/DOWNLOAD_BASE/VERSION),缺关键项 fail-closed;去掉无关的 `ghfast.top` 代理镜像;日志 `↓/⚠` 改 ASCII `[*]/[!]`。
  - **元数据** 根 `Cargo.toml` 的 `repository`、`packages/npm/package.json` homepage/repository、`packages/npm/README.md` 链接 → example.com / 发行渠道口径(npm scope 统一为真实的 `@rustcode/rustcode`)。
  - **维护者脚本** `scripts/scheduled-dev-sync.sh`:PR 的 host/slug 严格从 `origin` remote 推断,**不再回退** `SecLab/RustCode`/`gitcode.com`;推断不出则打印"在你的远程仓库 Web 界面手工建 PR";token 改 `RUSTCODE_PR_TOKEN`。
  - **Rust 夹具/注释**:`rustcode-config/src/config/mod.rs` issue-353 注释去 URL;`rustcode-tuix/src/modals/qr.rs` 两个**非门控** QR 渲染冒烟夹具 `acs.atomgit.com` → `example.com`。(`event_loop/commands.rs` 的 `acs.atomgit.com/login` 与 `monitor.rs` 的 AtomGit provider 夹具在 `#[cfg(feature="codingplan")]` 测试里,`endpoints.rs:407` 是"api-ai.gitcode.com 视为外部"的**负向中立回归断言**,均刻意保留。)
  - **文档/元数据(子 agent)**:README(中英)、extensions vscode `package.json`+README、jetbrains `docs/jetbrains.md`、`rustcode-clix/README.md`(签名网关段改"可选托管网关 + 默认 BYO")、seed skill `SKILL.md`/`skills-reference.md` 市场 URL、`rustcode-codingplan/Cargo.toml` 注释——grep 0 残留。
  - **文档站(子 agent)**:`site/` 46 个 html(install/clone/npm-scope/marketplace/CDN 图片/login 口径)中立化,`site/referral.html`(硬编码厂商 ACS 后端的邀请奖励 SPA)替为中性占位页并摘除入链;`search-index.{zh,en}.json` 由 `cd site && node build-search-index.mjs` 重新生成(不手改 JSON)。
  - **G6 复核**:产品内 **0 个 telemetry SDK 依赖/调用**(无 sentry/posthog/... crate 或依赖)。唯一 `sentry` 字符串命中是 seed skill `references/mcp-servers.md` 的**第三方 MCP 目录**——`@sentry/*` 作为"用户项目用了 Sentry 就推荐 Sentry MCP"的栈探测启发式(与同表 `@aws-sdk/*`/`@supabase/*`/GitHub MCP 同类),是外部工具文档,非本产品遥测;判为**已复核的假阳性**,保留。
  - **验证**:G1 `cargo fmt --check` 通过;`cargo check --workspace --all-targets` Finished(仅存量 warnings:capabilities dead_code、tuix `usage_render` unused_parens、cli 未用 `SessionId` 导入,均非本轮引入);tuix qr 7/7、config fallback 3/3 通过;install/uninstall/packaging/sync 脚本 `sh -n`/`bash -n` 通过;`create_tag_release.py` py_compile 通过;`scripts/`、`packages/`、`.github/` 厂商主机 grep 清零。
- **[DONE] "不与任何模型/平台绑定" 复核(2026-09-01,有证据,非断言)**:针对目标里"不与任何模型有关联、只保留第三方 BYO 配置、零遥测",逐面排查确认产品**不默认、不背书、不私连任何厂商**:
  - **运行时无厂商默认**:无配置时 headless 走 `headless_missing_provider_message()` 直接 bail(见上轮);TUI/daemon 新建 provider 面板 `protocol_preset_idx(OpenAi) = "openai-compatible"`,`default_base_url: None`、表单起点为中立兼容端点,不预填任何厂商 URL(`provider_panel.rs`)。厂商 preset(deepseek/openai/anthropic/qwen/zhipu/moonshot/… )全是**用户自带 key 手选的第三方 BYO 配置**,中立 `openai-compatible`/`anthropic-compatible` 居 PRESETS 最前(测试 `generic_endpoints_lead_the_registry` 锁定索引 0 中立)。`taotoken`(token 转售商,曾是 invite/referral 漏斗终点)现为非默认、`ModelSource::Manual` 的可选项,且代码里已无任何 affiliate/referral/invite 端点(pending_invite 写入侧随安装器一并删除)。
  - **模型相关代码是"兼容性适配"非"背书"**:`persona.rs model_needs_firm_execution`(deepseek/qwen 弱模型加严 reviewer 指令)、`reasoning.rs` 按模型派生 thinking 策略、`openai_compat.rs` 仅在 `is_openrouter_url` 时发 OpenRouter 归属头——都只在**用户已选用**某模型时调整行为,不引导用户去用某厂商;保留。
  - **webui / 扩展无绑定**:webui、VS Code(`daemon/client.ts`)、JetBrains(`RustCodeDaemonClient.kt`)只连本地 daemon(`http://host:port`),默认模型取用户配置里 `is_default`;无硬编码厂商端点、无分析 SDK;文档链接指向自有品牌域 `docs.rustcode.dev`(保留)。webui `SettingsDialogs` 账号名占位 `my-deepseek` 改中立 `my-provider`。
  - **零遥测(再证)**:全部 `package.json`(webui/extensions/site)无 sentry/posthog/mixpanel/amplitude/segment/gtag 依赖;webui/site/extension HTML 无 beacon/gtag/sendBeacon 追踪脚本;生产 .rs 无 `/metrics|collect|track|events|ingest|analytics|report|beacon` 上报端点;`rustcode-telemetry` crate 不存在。唯一 `sentry` 字符串是 seed skill `references/mcp-servers.md` 的第三方 MCP 推荐目录(与 GitHub/AWS MCP 同类),假阳性。
  - **示例配置是第三方模板**:`docs/config.example.toml`/`docker/config-example.toml` 演示第三方 BYO(`api_key="sk-..."` 用户自带、deepseek/glm 为可选第三方、codingplan 例用 `gateway.example.com`),属"只保留第三方配置"目标要求保留,不视为绑定。
- **[FIXED] i18n 文案测试对齐中立口径**(2026-09-01):上轮把 `Msg::CpOfficialBuildRequired` 改写为"第三方 BYO 或用带 CodingPlan 支持的发行版本"后,两个 i18n 内容测试仍断言旧厂商口径关键词(en 要求 `official`+`releases`、zh 要求 `官方`+`releases/发布`)——`cargo test -p rustcode-config --lib` 因此 2 failed。开放构建无"官方 releases"主机,旧断言绑定了厂商框架;改为断言**中立解决路径**:en 含 `bring your own key` + `distribution`,zh 含 `第三方提供商` + `发行`(`en_/zh_official_build_required_guides_byo_or_distribution`)。这是按去厂商化后的正确文案重新瞄准,非削弱断言。修正后 `rustcode-config` 328 passed / 0 failed。**教训:改 i18n 文案后跑整个 crate 的 `--lib`(内容测试散落在 en.rs/zh_cn.rs 末尾的 `codingplan_crypto_tests` 等模块),不要只跑受影响功能测试。**
- **[DONE] 多 agent 复核校准:首启引导 / 文案 / egress / 文档站中立化**(2026-09-01,多 agent 协助方案逐条 triage 并落地):
  - **首启引导按 `platform_server()` 门控**:`onboarding_wizard.rs` 新增 `SetupChoice{Login,Manual,Skip}` + `managed_login_available()`/`setup_choices()`——中立构建(空 `platform_server`)setup 菜单只出 Manual+Skip 两行、**不领死路 `/login`**;托管发行版保留 Login。引导第 3 条 bullet 分 `OnboardingIntroBullet3`(托管)/`OnboardingIntroBullet3Neutral`(自带 key 无需注册账号)。新增 `OnboardingSetupNavHint`("数字键选择",菜单条数 2/3 可变),语言步仍用 `OnboardingNavHint`("1-3 选择",语言恒为 3 项)。`welcome_tips.rs` pinned tip:空 `platform_server` 置顶 `/provider` 而非 `/login`(`LOGIN_TIP` 仅托管构建)。测试重瞄中立口径,断言未被削弱:9 个 onboarding + welcome_tip 单元测试断言 2 行 BYO 菜单;`render/retained.rs` 另有 **9 处** 把 pinned tip 当 `/login` 标记的欢迎屏 vterm 断言(`found_hint`/窄终端 reflow/首启 scrollback 计数/mascot 开关/resize 不重摇/colors-off 堆叠)改判 `/provider`——`/provider` 在中立构建是 pinned 且被随机池排除,故仍满足"恰好出现一次";`welcome_wide_*_pinned_login` 改名 `..._pinned_provider_tip` 并加"中立构建不出现 /login"负断言。顺带修一处**既有 locale 竞态**:`event_loop/commands.rs::todo_command_text_with_and_without_list` 两次全局 `t()` 未持 `i18n::test_lock()`,与并发 `set_locale` 测试竞争时两次读到不同语种 → 全量并行跑偶发红(单跑绿);补 `let _g = crate::i18n::test_lock();`(符合 AGENTS.md 既定模式)。
  - **"official build / 官方版本" 口径中立化**(en+zh):`CpUpgradeRequired`、`ProviderInitSourceBuild`、`GatewayAuthUnavailable`、`CmdProviderUnsupportedBuild`、`StatusOfficialBuildRequired`、`ProviderPanelManagedAccountHint` 去掉 "official releases / official binary / 官方版本 / 免费网关 / 官方 CodingPlan 账号",统一为"用带网关(托管签名)支持的**发行版构建**,或用 `/provider` 配第三方 BYO provider";`ProviderInitSourceBuild` 不再谎称"source build / 免费网关"。`i18n/mod.rs` 的 `GatewayAuthUnavailable` 测试只断言 `gateway/网关` + 回显 url,改后保持绿。
  - **clix 签名网关报错去厂商举例**:`rustcode-clix/src/main.rs` 把 "a closed-source overlay in the official binary" 改为"只在某些发行版构建中提供的闭源覆盖层",举例 `--provider openrouter` 改为通用 `--provider <name>`(选中具名 `[providers.<name>]` 表项,或 `RUSTCODE_API_KEY/BASE_URL/MODEL`)。
  - **egress 单一工厂补齐**:`rustcode-daemon/src/api_provider.rs` 模型发现(fetch_discovery_body)不再 `reqwest::Client::builder()` 自建,改 `rustcode_capabilities::egress::{build_http_client,HttpClientSpec}`(timeout/skip_tls_verify/user_agent 映射到 spec),与 provider/web/mcp/atomgit 共用同一 TLS 信任根 / proxy / UA / redirect 策略;egress 由 daemon 已启用的 `mcp` feature 拉入。
  - **seed skill 去硬编码 registry + 修 sudo 误导**:`rustcode-automation-recommender/SKILL.md` 把 `npx skills` / `skills.sh` 从"默认在线路径"降级为"发行版**可选**的社区 registry CLI 示例",删除全部硬编码 `https://skills.sh/` 链接;安装失败提示里"权限不足尝试 sudo"改为**明确禁止 sudo 写 `~/.rustcode`**(root 属主文件会破坏后续非 root 运行初始化,与 fork 铁律一致)。
  - **README(中英)**:PowerShell 安装示例补 `$env:RUSTCODE_RELEASE_BASE`(`install.ps1` 未设即 fail-closed,与 bash 示例对齐);uninstall 示例删掉 `uninstall.sh` 根本不读的 `RUSTCODE_RELEASE_BASE`;en README 中文小标 `**平台 Issues.**` → `**Platform issues.**` 并与站点一致加"附带托管平台支持的发行版"限定(中英都补,`platform_issue` 是 opt-in 平台工具的文档通用名);赞赏/社区二维码 `<img src="https://example.com/assets/...">`(example.com 永不托管,必裂图)换文字占位。
  - **site**:`index.html` 下载按钮从可点的 `example.com` 死链 `<a target=_blank>` 改为 `aria-disabled="true"` 非导航 `<span role=button>` + tooltip + `.dl-note` 双语说明(`RELEASE_BASE` 仍用于 binary tab 的 curl 文档示例,非死变量);`docs/en/login.html` stale meta("Three ways…CodingPlan / plain OAuth / API key")改两路 BYO 默认口径;`docs/{en,zh}/slash-commands.html` 的 `rustcode-channel.git` → 通用 `<marketplace>.git`;`build-search-index.mjs` 的 `GROUPS` 从"中文名硬编码进两种语言"改为 `{zh,en}` 双语标、`groupOf(slug,lang)` 按语言取(en 索引不再漏中文组名;顺带把 stale `运维` 对齐侧栏 `side.g.ops` 实际值 `问题/Help`),`cd site && node build-search-index.mjs` 重新生成 `search-index.{en,zh}.json`(不手改 JSON,0 个 heading id 注入)。
  - **验证**:G1 `cargo fmt --check` 通过(顺带 fmt 了 `parts.rs` 的 `parallel_template` 块);`cargo check --workspace --all-targets` Finished(仅存量 warning:cli `acp/discovery.rs` 未用 `SessionId` 导入,非本轮引入);默认成员全量 `cargo test` **2593 passed / 0 failed**(此前全量跑曾暴露上述 8 个欢迎屏 vterm 测试 + 1 个 todo locale 竞态红,均已改/修);`rustcode-tuix --lib` 全量 **2055/0**、`rustcode-config --lib` 328/0、`rustcode-clix` 44/0、`rustcode-daemon api_provider` 18/0;site 内联脚本 `new Function()` 语法检查 0 错误;搜索索引 en/zh 组名分别为 `Overview / Get Started / Usage / Advanced / Help` 与 `概览 / 开始 / 使用 / 进阶 / 问题`。
- **[DONE] 第二轮文档站/文档中立化校准(2026-09-01,多 agent 复核方案逐条落地,纯文档站/README,零 Rust 改动)**:
  - **落地页向导不再预选托管免费额度**(`site/index.html`):模拟首启向导第 3 步从三行(CodingPlan `class="opt sel"` "免费额度 · 推荐" + 手动 + 跳过)改为**两行 BYO**——手动配置预选(`wiz.s3.manual`/`.manual.hint`="自带 API Key · 推荐")+ 暂时跳过(`wiz.s3.skip`/`.skip.hint`);删除 dict 里 `wiz.s3.o1/o2/o3` 旧键。欢迎 bullet `wiz.s1.b3` 从"通过 CodingPlan 获取免费额度"改"自带 API Key,无需注册账号/Bring your own API key -- no account or signup"。
  - **删除托管计划导航/页脚入口**:header nav 与移动端抽屉里指向 `https://docs.rustcode.dev/models` 的 "Coding Plan" `<a class="nav-cp">/class="cp">` 两条链接、footer `ftr.links` 里 CodingPlan 锚点(静态 HTML + dict zh/en 三处)全部移除;随之删干净失效 CSS——`.hdr-nav a.nav-cp*`(8 行)、`.mb-drawer a.cp`/`a .free-badge`(浅色变体共 4 行)、dict 键 `nav.codingplan`;`@keyframes dotPulse` 仍被 hero badge `.dot` 使用,保留。`feat.platform.d`/`plug.platform.d` 里 OAuth 字样已是"发行渠道**可选**提供…开源默认构建 BYO"的正确门控口径,保留。
  - **M1 快速开始文档改为 BYO 默认**(`site/docs/{en,zh}/getting-started.html`):banner 第三条 "Free tokens via CodingPlan/通过 CodingPlan 获取免费额度" → "Bring your own API key — no account or signup/自带 API Key,无需注册账号";Step 3/3 配置转录从 3 项(1. CodingPlan 推荐领免费额度)改为 **2 项**(1. 手动配置 provider 推荐自带 Key,2. 跳过),`<ul>` 首条改为手动 `/provider` 推荐路径并列出 Claude/OpenAI/DeepSeek/GLM/Qwen/Ollama,另加一段说明"内置托管服务的发行版可能额外显示一键登录行(等价 `/login`),开源默认构建无此行、默认自带 Key"——与真实二进制 `setup_choices()` 的 2/3 行门控一致。
  - **S1 斜杠命令文档门控**(`site/docs/{en,zh}/slash-commands.html`):`/login` 表格行从"推荐·一条命令 OAuth + 领免费额度"改为"**仅发行版本/Distribution builds only**……开源默认构建无此命令,用 `/provider` 自带 Key";`/provider` 行改标 **推荐** 并写明第三方预设 + 自定义 OpenAI 风格接口 + 无需注册;`/cost` 行的 `/usage` 说明改"发行版查询托管账号额度,开源构建不可用、`/cost` 即全部用量";`/status` 去 "CodingPlan 用量" 改"provider 与鉴权状态(发行版另显托管账号用量)";`/whoami` 加"仅发行版本";`/logout` 从"清除 OAuth token"改"清除托管账号凭据(BYO 的 API Key 仍存配置)";`/reload` 括号里"另一个终端的 /login"改"provider/凭据变更";`/app` 中继行加"内置托管中继的发行版…开源默认构建不支持";典型工作流标题 "First launch — one-click onboarding/第一次启动-一键接入" 改 "bring-your-own-key/自带 Key",转录从 `/login`+CodingPlan claimed 改 `/provider` 三步(加账号→粘 Key→加模型)+ `/model`,末尾补"发行版也可 `/login` 一键 OAuth"。
  - **S2 daemon HTTP 文档与代码对齐**(`site/docs/{en,zh}/headless-daemon.html`):"Auth / CodingPlan" 小节改名 "认证与托管账号(仅发行版本)" 并加一段:开源默认构建无托管端点,BYO 走上文 `POST /providers` 或 `config.toml`;该构建里 `/auth/*` 路由虽注册但 OAuth 无法完成(无服务可登,`auth::start_login()` 对空 `platform_server` 报错返回 500),`/codingplan/*` 路由由 `#[cfg(feature="codingplan")]` 门控、**不参与编译返回 404**(`lib.rs::codingplan_routes()` 空 router 证实)。修正一处**事实错误**:原表称 `rustcode codingplan` 作为隐藏别名保留——实际 CLI 补全测试 `main.rs:4339` 断言 `!script.contains("codingplan")`,该别名已移除,文档改为"CLI 没有单独的 `rustcode codingplan` 命令";并补登 `/codingplan/usage/summary`、`/codingplan/usage/daily` 两条仅发行版路由(此前漏列)。
  - **S3 README**:`README.zh-CN.md:94` 智谱 GLM 行去掉虚构的"（RustCode Pro 套餐专属模型）"(不存在任何付费套餐;en README 本就平铺 GLM-4/5/5.2),与 en 对齐。
  - **NIT 批**:(N1)两份 README "会话与登录/Sessions & Login" 的 OAuth/SSO bullet 加"仅发行版本/distribution builds only"并新增首条"第三方供应商(BYO)——配置自己的 base_url+api_key,无需注册,开源默认方式";README 斜杠命令表 `/login` 行从"claim CodingPlan free models"改"登录托管服务(仅发行版;BYO 用 `/provider`)"。(N4)`docs/zh/login.html` 单薄 meta("登录方式 — RustCode 文档。")补成与 en 同义的丰富描述;该页正文已是 BYO 默认 + 托管登录门控,无需改。(N6)安装占位符统一中立域:`site/index.html` unix/win 安装命令 `https://your-host/...` → `https://example.com/your-host/...`,README(en/zh)PowerShell 与 bash 的 `RUSTCODE_RELEASE_BASE=https://<host>/...` → `https://example.com/your-host/...`(与同仓 `git clone https://example.com/<your-org>/...` 及站点 `RELEASE_BASE` 口径一致;`api.deepseek.com/v1` 等是**真实第三方 BYO provider 端点示例**,属正确用户文档,保留)。(N7)README 隐私/遥测节链接补齐:en Privacy bullet 原只链 ORIGINAL_LICENSE/UPSTREAM_CREDITS,补上 `docs/telemetry.md`(zh `零遥测` bullet 本就链该文件),中英对齐。(N5)`site/index.html` 无 JS 静态兜底与 i18n dict 对齐:静态 `step2.t` "配置 API Key" → "配置 provider"、`step2.d` 换成 dict 的富文本"Claude/OpenAI/DeepSeek/GLM/Qwen/Ollama 等第三方 API 任一 · 自带 Key";dict `wiz.s3.skip.hint` en 残片 "explore first" → "Set up later"(配 zh 稍后再说)。(N3)`build-search-index.mjs` 的 GROUPS 进阶组补 `headless-daemon` slug——该页被 webui/webui-remote-access 链接引用、此前却不在任何组里,索引阶段被 `skip ungrouped` 丢弃且拿不到 heading id 注入。
  - **搜索索引重生成**:`cd site && node build-search-index.mjs`(不手改 JSON)——en/zh 各 **21 pages**(headless-daemon 现入索引),各向 1 页注入 heading id(即 headless-daemon:h1/h2/h3 获 `id` 保证搜索 `#anchor` 可滚动);`search-index.{en,zh}.json` 随内容更新。
  - **未改/刻意保留**:README 与 docs 里 `atomgit_atomcode/atomcode` 仅出现在 fork 归属声明与 ORIGINAL_LICENSE/UPSTREAM_CREDITS(合法历史出处,G7/G8 逐案豁免);"关于可选的 CodingPlan 网关(闭源签名)" 小节是对 opt-in 发行版能力的正确门控说明,保留;`atomgit` cargo feature、`#[cfg(feature="codingplan")]` 块、`LEGACY_CODINGPLAN_PREFIX` 识别器按 fork 铁律**不动**。本轮零 `.rs` 改动,G1–G3 不受影响。
- **[DONE] 第三轮多 agent 校准:斜杠命令发现面按构建门控(2026-09-01,代码复核 MED 落地,含 Rust 改动)**:
  - **问题(代码复核 agent MED)**:上一轮文档把门控口径写进了文档,但 `rustcode-tuix/src/commands.rs` 的命令发现面只按静态 `hidden` 标志过滤、**从不查 `platform_server()`**——中立构建里 `/help`、`/` 斜杠菜单、Tab 补全、ACP `available_commands` 仍展示 `/login`("Sign in with OAuth and claim CodingPlan models")与 `/usage`("Show CodingPlan usage",该行 `acp: true`,此前**确实**被广告给 ACP 客户端),即首启引导 `setup_choices()`/欢迎 pinned tip 已消除的"死路托管推销"在命令发现面残留。
  - **修复**:`commands.rs` 新增 `const MANAGED_ONLY_COMMANDS: &[&str] = &["login", "logout", "whoami", "usage"];` 与 `fn command_visible(cmd) -> bool`——静态 `hidden` 恒隐藏;名字 ∈ MANAGED_ONLY_COMMANDS **且** `!crate::modals::onboarding_wizard::managed_login_available()`(即空 `platform_server` 的中立构建)时隐藏。五个发现面全部改走该谓词:`matching_prefix`(斜杠菜单)、`help_text` 两处(最大列宽计算 + 输出循环)、`acp_commands`(ACP v1/v2 广告与 ACP `/help` 同源)、`complete_commands`(Tab 补全)。**派发表 `find()` 不查可见性**:四个命令在中立构建仍可手动输入执行(走既有 BYO `LoginManagedUnavailable` / 本地用量提示),只是从所有发现面消失——与 onboarding 2/3 行门控同一谓词,dispatch 与 discovery 分离。
  - **测试**:`help_text_lists_all_commands` 的分支条件 `if c.hidden` → `if !command_visible(c)`(注释说明隐藏别名与中立构建的托管命令都不进 `/help`);新增 `neutral_build_hides_managed_account_commands`——断言中立 `/help` 不含四个托管命令、`matching_prefix` 不浮现它们、ACP 不广告 `/usage`、Tab 补全不浮现 `/login`,且 `/help` 必含 `/provider`(BYO 领路)。
  - **验证(全绿)**:G1 `cargo fmt --check` 通过(rustfmt 重排了两处长行);`cargo test -p rustcode-tuix --lib` **2060 passed / 0 failed**;`cargo test -p rustcode acp::commands` 3/0(既有 `acp_catalog_is_sorted_and_filtered` 本就禁 `login`/`logout`/`whoami`——它们 `acp: false`,`parse_slash_command("/login")` 恒 None;门控后 `usage` 在中立构建也退出 ACP 广告,托管发行版仍保留);`cargo clippy -p rustcode-tuix --all-targets` 在 `src/commands.rs` **零 warning**(`event_loop/commands.rs` 的存量 `io_other_error`/大枚举变体告警非本轮引入)。
  - **代码复核 agent 干净判定(留档)**:硬编码托管主机全清(所有 `HOSTED_*` 常量为空;`is_codingplan_llm_gateway()` 仅在显式 `RUSTCODE_CODINGPLAN_LLM_BASE_URL` 时为真,否则一律 `bearer_auth(api_key)`);HTTP egress 单一工厂基本干净(leaf `rustcode-updater`/`rustcode-auth` 无法依赖 capabilities 属结构豁免,LLM 适配器打用户自供 base_url 豁免);遥测 SDK grep 为零;默认中文在全部回退点(`locale.rs` Default、`i18n/mod.rs` 静态/resolve/poison 三处、persona 注入、webui `i18n.ts` DEFAULT_LANG='zh')成立;无明显裸 unwrap/expect 风险。
  - **LOW 不改(留档)**:webui `SettingsDialogs.tsx` 硬编码 `https://pgy.oray.com`(蒲公英 Oray 虚拟局域网远程访问说明链接)——客户端从不请求该地址,性质同 Ollama 下载页这类第三方工具文档链接,复核 agent 自标 "awareness only";与 `webui-remote-access` 文档的蒲公英方案一致,保留。
  - **刻意保留**:`BUILTIN_COMMANDS` 里 login/usage 的静态 `desc` 与 i18n `CmdDescLogin`/`CmdDescUsage`(zh_cn.rs/en.rs)文案仍提 CodingPlan——它们只在托管发行版渲染(中立构建命令已从发现面隐藏,`cmd_desc_i18n` 无 Neutral 变体),是对 opt-in 托管能力的准确描述,与 `#[cfg(feature="codingplan")]` 块同属保留开关,不动。
- **[DONE] 第四轮多 agent 校准:同一谓词贯通 daemon/WebUI/TUI 的托管账号门控(2026-09-01,承接第三轮 TUI 命令门控,补齐剩余两个用户面)**:
  - **背景**:第三轮把 TUI 斜杠命令发现面门控后,代码面排查发现 **WebUI 与守护进程 API 仍无条件推销托管登录**——`useAuth`(webui `LoginButton.tsx`)每 2s 轮询 `/auth/status`,侧栏底部恒显"登录/Sign in"按钮;中立构建点击后 `POST /auth/login/start` 对空 `platform_server` 必返 500(死路,且无任何用户反馈);WebUI 自己的斜杠命令表(`lib/slashCommands.ts`)还把 `/whoami` 列进菜单/`/help`;TUI 的 `/whoami` 与 provider `AuthenticationRequired` 状态提示一律输出"使用 /login 进行认证"(中立构建无 login 可用)。
  - **单一谓词落地**:`rustcode-auth/src/oauth.rs` 新增 `pub fn managed_login_available() -> bool`(对**原始** `rustcode_config::endpoints::platform_server()` 判空——不能用 `platform_base_url()`,sanitize 空串会产出 `"http:"` 非空;`pub use oauth::*` 自动导出)。TUI `onboarding_wizard::managed_login_available()` 改为**委托**该谓词(此前直接查 config),`commands::command_visible`/onboarding/daemon/WebUI 从此同源,杜绝漂移。
  - **daemon**:`api_auth.rs` 的 `AuthStatusResponse` 新增 `managed_available: bool`(3 个构造点:status 有凭据/无凭据、logout 后),值取 `auth::managed_login_available()`;新增 `neutral_build_reports_managed_unavailable` 测试(起 `auth_status()` handler 断言 JSON `logged_in=false`、`managed_available=false`、谓词一致)。
  - **TUI 文案中立变体**:`rustcode-config` i18n 新增 `Msg::CmdWhoamiNotSignedInNeutral`(en:"This build has no managed account -- use /provider to add a bring-your-own-key provider";zh:"此构建无托管账号 -- 用 /provider 配置第三方供应商(自带 API Key)");`onboarding_wizard` 新增 `pub(crate) fn not_signed_in_msg()` 按谓词二选一;3 个旧 `CmdWhoamiNotSignedIn` 调用点全部改走它——`build_whoami_text()`(`/whoami` 输出,TUI arm 与远程执行共用)、`provider_unavailable_announcement()`、状态栏 provider-waiting 提示(event_loop/mod.rs 两处)。新增 tuix 测试 `neutral_build_whoami_points_at_provider_not_login`(断言中立文本含 `/provider` 且不含 `/login`)。
  - **WebUI**(React/Preact):(1)`LoginButton.tsx` 的 `useAuth` 读取 `managed_available` 并暴露 `managedAvailable`;中立构建首次轮询拿到 false 后**停止 2s 定时轮询**(状态不可能翻转为登录),`startLogin()` 加 `!managedRef` fail-closed 守卫。(2)`Sidebar.tsx` 侧栏底部账号区(头像 chip / 登录按钮整块)与设置菜单"退出登录"行均按 `auth.managedAvailable` 门控,中立构建完全不渲染。(3)`lib/slashCommands.ts`:`SlashCommandDef` 加 `managedOnly?: boolean`(`/whoami` 标记),新增 `visibleCommands(managedAvailable)` 过滤;`buildHelpText(t, managedAvailable)` 与斜杠菜单改用可见列表;`SlashHandlers` 加 `managedAvailable` 字段;**dispatch map 仍建自全量 `FRONTEND_COMMANDS`**——手敲 `/whoami` 依旧执行(服务端返回中立文案),与 TUI"隐藏但可派发"一致。(4)`Chat.tsx` 挂载时一次性 fetch `/auth/status` 取 `managed_available`(fail-closed false),`advertisedCommands` memo 接入两处菜单构建与 handlers。(5)新增 4 个 webui 单测(中立列表隐藏 whoami/`/help` 双语门控/菜单过滤/显式键入仍可派发),既有 fakeHandlers 全部补 `managedAvailable` 字段。
  - **文档**:`site/docs/{en,zh}/headless-daemon.html` 的 `GET /auth/status` 行补述 `managed_available` 字段语义(false 时 WebUI 隐藏账号入口);搜索索引重新生成(en/zh 各 21 页)。
  - **验证(全绿)**:`cargo fmt --check` 通过;`cargo check --workspace --all-targets` Finished;`cargo test -p rustcode-tuix --lib` **2061/0**(新增 1)、`-p rustcode-daemon --lib` **304/0**(api_auth 8/0,含新增)、`-p rustcode-config --lib` **328/0**(i18n 双语 arm 齐全);clippy 在本轮改动区域**零新增告警**(新测试的 `assert_eq! bool` 已按 clippy 改 `assert!`;onboarding_wizard/oauth 既有存量告警位置远离改动);webui `tsc --noEmit` 0 错误、`npm test` **221/0**(新增 4)、`npm run build` 成功后按铁律 `cargo clean -p rustcode-daemon` 再 check(dist 嵌入刷新)。
  - **刻意保留**:WebUI `cmd.whoami.none`("未登录/Not signed in")无 login 推销,仅显式键入时出现,保留;`SettingsDialogs.tsx` 的托管 provider 徽标 `settings.officialCodingPlan` 由 daemon 数据门控(`provider.requires_login`/`account.managed`,中立构建无此数据),保留;上一轮记录的 pgy.oray.com LOW 仍维持 awareness-only。
- **[DONE] 第五轮:CLI 子命令发现面门控 + 补全脚本口径(2026-09-01,承接第三/四轮,补齐最后一个 Rust 驱动面)**:
  - **问题**:第三/四轮把门控谓词(`rustcode_auth::managed_login_available()`)贯通了 TUI 命令发现面、daemon `/auth/status`、WebUI,但 **CLI 二进制自身的 `--help`/clap 子命令与 shell 补全脚本**仍无条件列出 `rustcode login`/`logout`(about 文案推销 OAuth 托管登录),中立构建用户照做即死路;`completion_command()` 直接用未门控的 command 树生成 5 种 shell 补全。
  - **修复**(`crates/rustcode-cli/src/main.rs`):`build_i18n_command()` 计算 `let managed = auth::managed_login_available();`,中立构建对 `login`/`logout` 两个子命令 `.hide(true)` 并把 `login` 的 about 换成 `Msg::CliAboutLoginNeutral`(en:"Managed sign-in (distribution builds only) -- this open build uses bring-your-own-key providers; configure config.toml";zh 同义),`logout` about 改"安全 no-op"口径;`status` 不隐藏、about 用新增 `Msg::CliAboutStatus`("Show current provider and sign-in status/查看当前供应商与登录状态")。`completion_command()` 改为从 `build_i18n_command()`(门控后的同一棵树)生成补全。**dispatch 不门控**:手敲 `rustcode login` 仍可执行并打印中立 about(hidden != removed,与 TUI/WebUI "隐藏但可派发"一致)。顺带修一处既有文案 bug:`{oauth}` 占位符原样泄漏到 login about。
  - **测试**:(1)新增 `neutral_build_hides_managed_login_subcommands`——`find_subcommand("login")/("logout")` 存在但 `is_hide_set()`、login about 含 `config.toml` 不含 `CodingPlan`、`status` 未隐藏(注意 clap 4.6 `get_about()` 返回 `Option<&StyledStr>`,断言前 `.map(|s| s.to_string())`)。(2)`completion_scripts_cover_all_supported_shells` 原来的扁平 `!script.contains("login")` 断言误伤合法的 `mcp login`/`mcp logout`(MCP server OAuth,与托管登录无关,五种 shell 都应出现)——改为逐行上下文跟踪器:bash 函数头(`() {`)、zsh `curcontext=` 行(如 `:rustcode-mcp-command-$line[1]:`)、elvish/powershell map 路径(`&'rustcode;...` / `'rustcode;...{`)作为上下文标记,任何含 `login/logout` 的行必须处于 mcp 上下文(行内或上下文标记含 `mcp`/`github-oauth`)。
  - **i18n**:新增 `Msg::CliAboutLoginNeutral` 与 `Msg::CliAboutStatus`(en.rs/zh_cn.rs 双语 arm 齐全);`rustcode-config --lib` **328/0**。
  - **运行时验证**:`rustcode --help` 不再出现 login/logout/codingplan;`rustcode login --help` 仍可派发并打印中立 about。CLI 测试套件全绿(含两个新/改测试)。
- **[DONE] 第六轮:daemon 托管登录中立失败契约从 500 改可操作 501(2026-09-01)**:
  - **问题**:第四轮给 `GET /auth/status` 加了 `managed_available` 字段,但中立构建里 `POST /auth/login/start` 仍走到 `auth::start_login()` 对空 `platform_server` 报错,返回**通用 HTTP 500 `login_start_failed`**——客户端无法区分"服务暂时挂了(可重试)"与"此构建根本没有托管登录(永远别重试)",扩展端只能显示死路错误。
  - **修复**(`crates/rustcode-daemon/src/api_auth.rs`):新增 `pub(crate) fn managed_login_unavailable_response()`——HTTP **501** + JSON code **`managed_login_unavailable`**、`retryable: false`、消息指向第三方 BYO provider 配置;`auth_login_start` handler 在 spawn_blocking 之前 `if !auth::managed_login_available() { return managed_login_unavailable_response(); }`(fail-closed,与谓词同源)。`/codingplan/*` 路由仍由 `#[cfg(feature="codingplan")]` 门控、中立构建 404,不动。
  - **测试**:`neutral_build_login_start_returns_actionable_501`(起 handler 断言 501、`json["code"]=="managed_login_unavailable"`、`retryable==false`、消息含 "third-party provider")。`rustcode-daemon --lib` **305/0**。
  - **文档**:`site/docs/{en,zh}/headless-daemon.html` 的 `POST /auth/login/start` 行补述中立 501 契约(zh:"开源构建中会快速失败,返回 501 与错误码 managed_login_unavailable(不可重试),提示客户端引导用户改用自带 API Key 的第三方供应商配置");搜索索引已重新生成(en/zh 各 21 页,索引含 `managed_login_unavailable`)。
- **[DONE] 第七轮:两个 IDE 扩展按 `managed_available` 门控托管登录 UI(2026-09-01,多 agent 扩展审计方案逐条 triage;VS Code 全验证,JetBrains 仅源码级——本机无 JDK/gradle)**:
  - **VS Code**(`extensions/vscode/`,host tsc 0 错、webview tsc 0 错、`npm run test:webview` 全组 0 失败、`npm run build:webview` 成功):
    - **wire/host**:`src/daemon/types.ts` 的 `AuthStatusResponse` 加 `managed_available?: boolean`(旧 daemon 缺字段 → fail-closed);`src/auth/status.ts` 加 `managedLoginAvailable(auth)`(`=== true` 判定);`src/chat/provider.ts` 持有 `_managedLoginAvailable`(初始 false fail-closed,`_sendSetupState` 轮询 `/auth/status` 时赋值),`_startLogin()`/`_setupCodingPlan()` 在不可用时早返回并广播 setupError(新中立文案 `_managedLoginUnavailableMessage()`,`vscode.l10n.t()`),`/login` 手敲回中立提示、`/logout` 回"此构建无托管账号"、`/whoami` 未登录分支按谓词二选一;webview HTML locale 注入从硬编码 `'en'` 改为 `vscode.env.language || 'zh-cn'`(与 Rust ZhCn 默认对齐)。
    - **webview(Preact)**:`state/types.ts` 的 `AuthStatus` 加 `managed_available?`(snake_case 直出);`WelcomeScreen.tsx` 加 `managed` 谓词——账号步骤/loginUrl 块/同步按钮/"手动添加"折叠开关全部 `{managed && ...)}`,中立构建手动 provider 表单默认展开;`baseUrl` 初值从预填 `https://api.openai.com/v1` 改空串(不预选厂商);models 步副标题用新 `setup.addProviderHint`。`SlashPicker.tsx` 的 `SlashCommand` 加 `managedOnly?`,`/login`/`/logout`/`/whoami` 标记后按 `state.auth.managed_available` 过滤(派发路径不变,手敲仍达 host)。`i18n.tsx` 加 `setup.addProviderHint` zh/en;`l10n/bundle.l10n.zh-cn.json` 加 3 条中立文案。
    - **测试**:`webview-ui/test/auth-status.test.ts` 加 4 个 `managedLoginAvailable` 断言(true/false/undefined/缺字段全 fail-closed)。顺带修一处**既有** tsc 错误:host 会 post `{type:'runtimeInfo',provider,model}` 但 webview `ExtensionMessage` 联合缺该变体(已用 git stash 证实为先存),补上。
    - **npm 环境注记**:npmmirror 镜像的 `y18n@5.0.9` tarball 是幻影(404,真版本止于 5.0.8),`npm ci` 必败;解法是把坏 lock 移走后 `npm_config_registry=https://registry.npmjs.org npm install --no-audit --no-fund` 重生成 `package-lock.json`(已落地)。
  - **JetBrains**(`extensions/jetbrains/`,**源码完成,未编译验证**——本机 EulerOS aarch64 无 JDK、无缓存 gradle 发行包,不声称绿灯):
    - `RustCodeDaemonTypes.kt` 的 `AuthStatusResponse` 加 `managedAvailable: Boolean = false`(缺字段 fail-closed);`RustCodeDaemonClient.kt` 的 `authStatus()` 解析 `raw.jsonBoolean("managed_available") ?: false`。
    - `RustCodeChatPanel.kt` 持有 `private var managedLogin = false`,`renderSetupSnapshot` 里 `snapshot.auth?.managedAvailable == true` 赋值;齿轮菜单的 login + CodingPlan 设置项(含分隔线)`if (managedLogin)` 包裹;`showCommandMenu` 的 `/login` 行同理;welcome-action `"login"` 与手敲 `/login` 在不可用时改发 `managedLoginUnavailableMessage()`(新 bundle key `login.unavailable`,en + `RustCodeBundle_zh.properties` 的 `\uXXXX` 转义中文)。
    - `JBCefMessageView.kt`:`showWelcomePage(language, loggedIn, managedAvailable = false)` 贯穿到 `welcomeContent`,zh/en 文档文案按谓词二选一(托管:平台登录说明;中立:齿轮菜单 Provider -> Create Provider BYO 指引),`showLogin = !loggedIn && managedAvailable`;welcome HTML 品牌标记 AtomCode 时代残留 `<span class="home-mark">A</span>` 改 `"R"`。
    - `EditorAtomCodeActions.kt` 重命名为 `EditorRustCodeActions.kt`(内部 object 本就叫 `EditorRustCodeActions`,无残留引用);README 斜杠命令清单修正为实际接线的 `/review` + 门控的 `/login`,删掉虚构的 `/codingplan /explain /fix /test /refactor /docs /optimize`。
  - **两个审计 agent 的刻意保留/LEAVE 项(留档,勿再报)**:扩展侧 OAuth 传输与 `/codingplan/setup` 客户端方法/类型是托管发行版 API 的对应保留开关(门控不删除,同 `atomgit` cargo feature 铁律);`requires_login` 驱动的 ModelSelector 徽标、legacy-null fail-closed 分支、example.com 夹具、127.0.0.1:13456 默认、零遥测(两扩展依赖仅 gson/kotlinx 与 dompurify/marked/react,无分析 SDK)均判干净。**延后 SHOULD-FIX 候选**:JetBrains 硬编码 zh/en 字符串迁入 bundle(含 `SlashCommand` 描述硬编码中文、`managedLoginUnavailableMessage()` 用无 locale 重载——与齿轮标签一致跟随 JVM locale 而非 `welcomeLanguage`,属既有模式)、PRIVACY.md 中立措辞、ASCII 符号巡查。
  - **JetBrains 独立编译审计(general-purpose agent,逐条核)**:**无 BLOCKER,判定 "COMPILE-VERIFIED BY INSPECTION"**——`showWelcomePage/welcomeContent` 新旧调用点参数顺序全对(外部唯一调用 `RustCodeChatPanel.kt:373` 传 `(welcomeLanguage, loggedIn, managedLogin)`);`jsonBoolean` 定义在 `SseParser.kt:104`(同包 internal 扩展)可解析;`RustCodeBundle.message(key)` 单参重载存在(locale 内部按 JVM 解析);`login.unavailable` 双语 properties 键齐(16/16 parity)、zh 文件纯 ASCII 转义;改动区括号平衡;`managedLogin` 字段声明即 false(fail-closed),首启 welcome 在 snapshot 到达前渲染=隐藏托管 UI;门控在 Kotlin 侧烘焙进 Gson payload(`showLogin`/`docsText`),JS 不引用未填充字段;jetbrains 目录 `atomcode/atomgit` grep 零命中;`EditorRustCodeActions.kt` 重命名无悬挂引用(plugin.xml 只注册 Action 类、无 per-file sourceSet、git 纯 rename);唯一既有 pictograph U+2713 为允许的单色符号。
- **[DONE] 第八轮多 agent 校准:残余"死路 /login"文案清零(2026-09-01,跨面 sweep agent 报 4 项,逐条 triage 全落地;含 Rust+webui 改动)**:
  - **背景**:第七轮后 very-thorough sweep 专查"中立构建仍会显示、且未被谓词门控的托管登录/注册/免费额度文案",报 HIGH 1 + MED 2 + LOW 2;另确认一大批 LEAVE(CLI/TUI/onboarding/WebUI 侧栏/`/auth/status` 已门控;`ChatAuthExpired`/api_provider "managed by /login" 错误只在 signer/`selection_is_managed()` 成立时触发;MCP `/mcp login` 是第三方 server OAuth 合法 BYO 功能;`UsageCodingPlanOnly` 本身已是中立文案)。
  - **(HIGH)daemon `/status` 的 Login 行**:`crates/rustcode-daemon/src/commands.rs::render_login_line_from_stored_auth()` 此前无条件渲染 `StatusLoginNotSignedIn`("not signed in (run /login)"),webui `/status` 与 CLI status 透传该文本——TUI 等价处(`event_loop/commands.rs` status 渲染)早已按谓词返回空串,daemon 漏网。修复:函数开头 `if !rustcode_auth::managed_login_available() { return String::new(); }`(`assemble_status` 直接拼接空串,输出干净)。新测试 `neutral_status_omits_managed_login_line`:谓词为 false 时函数返回空,且 `exec_status()` 组装文本不含 `/login`(注意 `CommandResult` 只 derive `Serialize` 无 `Debug`,测试用 `if let ... else { panic! }` 不能 `{other:?}`)。
  - **(MED)TUI `CmdProviderUnavailable` 七处调用点**:`"Provider is unavailable. Use /login to sign in or /provider..."` 在提交拒绝/steer 失败/队列排空失败/goal/loop 启动失败/status bar 提示里无条件出现。新增 `Msg::CmdProviderUnavailableNeutral`(en:"Use /provider to configure a third-party provider with your own API key";zh:"请用 /provider 配置第三方 Provider(自带 API Key)")与 `onboarding_wizard::provider_unavailable_msg()`(按 `managed_login_available()` 二选一,与既有 `not_signed_in_msg()` 同构);mod.rs 5 处 + commands.rs 2 处(goal/loop `StartGoal/StartLoop` dispatch 失败)全部改走该 helper;其中提交拒绝 match 的 `AuthenticationRequired` 臂改走 `not_signed_in_msg()`(与 status bar 臂 28388 行既有行为对齐,托管构建显示未登录、中立显示 BYO),`None` 臂走 `provider_unavailable_msg()`。
  - **(MED)`SubmitHeldUntilLogin` 排队提示**:`event_loop/mod.rs` 首条提交在 `AwaitingProvider && !AuthObservation::is_available()` 时提示"run /login 后自动发送";中立构建无登录系统,该条件不可能因"未登录"成立——加 `managed_login_available()` 合取,中立构建回落已有的 `SubmitHeldUntilProviderReady`("provider 尚未就绪,消息已排队,就绪后自动发送",文案本就中立),无需新消息。
  - **(LOW-MED)WebUI 模型设置弹窗 intro**:`settings.modelsIntro` 原文"Official CodingPlan models are managed by your login/官方 CodingPlan 模型由登录状态同步管理"在 `ModelConfigDialog` 无条件渲染。拆为两条:base `modelsIntro` 改中立 BYO 文案(双语),新增 `modelsIntroManaged`(托管同步说明),组件 `useAuth()`(复用 `LoginButton.tsx` 导出的轮询 hook,无循环依赖——LoginButton 仅依赖 preact/settings)取 `managedAvailable`,仅 true 时追加渲染托管句。顺带删除两个**死键** `cmd.status.body`/`cmd.status.notLoggedIn`(全仓 grep 无引用,/status 走 daemon 文本;死键里含 "login {login}" 段,防止将来接线泄漏)。
  - **(LOW)TUI 欢迎提示池 `/usage` 死路**:`render/welcome_tips.rs` 的 `POOL` 常量无条件含 `/usage`("view token usage & quota")提示——第三轮已把 `/usage` 从中立构建所有发现面隐藏(手敲只回 `UsageCodingPlanOnly` 不可用通知),提示池却仍随机广告它。新增 `const MANAGED_ONLY_TIP_CMDS: &["/usage"]` 与 `tip_allowed()`(谓词同源 `onboarding_wizard::managed_login_available()`),`choose_pool_indices` 过滤候选、`tips_from_indices` 对缓存索引防御性过滤;新测试 `managed_only_tips_never_surface_in_neutral_build`(64 个种子,新鲜选择与缓存解析两路都不得出现 `/usage`)。
  - **VS Code 死代码清理(审计 SHOULD-FIX 落地)**:删除 webview 死组件 `webview-ui/src/components/ProviderSettings.tsx`(零引用的旧设置浮层,唯一引用是过期构建产物 `webview/webview.js`——esbuild 输出、git 未跟踪,重建即刷新;删后 `settings-overlay` class 在新 bundle 中 0 命中)。随之移除 `ChatProvider.tsx` 中**无消费者**的 5 个 context action 包装(`startLogin`/`cancelLogin`/`setupCodingPlan`/`refreshSetupState`/`setDefaultProvider`——唯一消费者就是该死组件;WelcomeScreen 用本地 `postMessage` 直发,host 消息处理不受影响):接口成员、`useCallback` 定义、value 条目三处同删,留注释说明 setup/login action 刻意不做 context 方法。
  - **验证**:`cargo fmt --check` 通过;`rustcode-config --lib` 328/0、`rustcode-tuix --lib` **2062/0**(新增 1)、`rustcode-daemon --lib` **306/0**(新增 1);`cargo clippy -p rustcode-tuix -p rustcode-daemon -p rustcode-config --all-targets` 0 新增告警;webui `tsc --noEmit` 0 错、`npm test` **221/0**、`npm run build` 成功后按铁律 `cargo clean -p rustcode-daemon`(dist 嵌入刷新);VS Code 扩展 host + webview `tsc` 双 0 错、`test:webview` 11/11 组通过、`build:webview` 成功;全工作区 `cargo test --workspace --no-fail-fast` 复跑 **5472 passed / 1 failed**,唯一失败为文档化已知红 `mcp::registry::tests::trust_key_golden_matches_core_algorithm`(DefaultHasher 不稳定,铁律禁止改)。
  - **刻意保留(sweep LEAVE 留档)**:`ProviderPanelManagedAccountHint`(仅选中托管账号时显示)、`CpOfficialBuildRequired`/`StatusOfficialBuildRequired`(仅 `is_codingplan_gateway(base_url)` 且 signer 缺失的误配边缘)、api_provider.rs 五处 "managed by /login"(仅 `selection_is_managed()/account_is_managed()` 成立)、`ProviderInitNeedsLogin` 与 webui `settings.contextWindowLocked` 死消息/死键(不可达,留待清理)、MCP OAuth `/mcp login`(第三方 server 登录,合法 BYO)、clix 签名网关报错(上轮已中立)。**(第八轮 LEAVE 中的 api_provider 五处、`ProviderInitNeedsLogin` 族、provider_panel 两条 hint 已在第九轮收尾。)**
- **[DONE] 第九轮多 agent 校准:门控谓词一致性审计 + 纵深防御/死代码/边缘文案收尾(2026-09-01,general-purpose 高努力审计 agent 报 0 BLOCKER + 6 NIT,全部 triage 落地)**:
  - **审计结论 "GATE CONSISTENCY CLEAN"**:第八轮全部改动逐条核验 PASS——daemon login 行空串拼接干净、`provider_unavailable_msg()` 七处调用点无遗漏(零直接 `Msg::CmdProviderUnavailable` 引用)、`SubmitHeldUntilLogin` 合取正确、welcome tips `tip_allowed()` 覆盖新鲜选择与缓存索引两路、i18n 双语 arm 穷尽匹配编译期保证、webui zh/en 字典 380/380 键 parity(死键确认删除)、VS Code ChatProvider/ProviderSettings 删除面干净、WelcomeScreen 所有 postMessage 调用点均在 `managed_available === true` JSX 内、Kotlin `?: false` fail-closed 一致;漂移审计:`platform_server()`/`platform_base_url()` 直接读取点除 oauth.rs/endpoints/welcome_tips 三处特许外**为零**;`ChatAuthExpired` 仅 signer 存在时触发、`codingplan_sign` /login 错误需网关+signer、`StatusCp*`/`CodingPlanSetupFailed` 在 `#[cfg(feature="codingplan")]` 后、ACP 命令广告经 `command_visible` 过滤。
  - **NIT1(纵深防御真 bug)**:`rustcode-auth/src/oauth.rs::start_login()` 守卫误用 `platform_base_url().is_empty()`——该函数把 `""` sanitize 成 `"http:"`,条件**恒 false**,中立错误永不返回(调用方上游已门控故零用户可见影响,但防线失效)。改为 `if !managed_login_available()`(同文件谓词,RAW 值+trim);错误文案改中立 BYO 指引(不再提 `/login`);顺带修正谓词文档注释里 `platform_server().is_empty()` 的陈旧描述为 `.trim().is_empty()`。
  - **NIT2(单一事实源)**:`welcome_tips.rs::pinned()` 直接读 `platform_server().is_empty()`(无 trim,与谓词漂移;不可达但违反对齐),改委托 `onboarding_wizard::managed_login_available()`(与 `tip_allowed` 同源)。
  - **NIT3(死消息清理)**:`ProviderInitFailed{detail}`/`ProviderInitNeedsLogin`/`ProviderInitSourceBuild`/`GatewayAuthUnavailable{base_url}` 四个 Msg 变体**零生产引用**(渲染点在 commit 59d4c284 "retire legacy engine bridge" 中移除,仅剩两个 i18n 测试引用)——messages.rs 声明、en.rs/zh_cn.rs arm、mod.rs 两个测试及随之失活的 `has_cjk` test helper 全删(config 328→326)。**Cp* 签名族保留**(`CpOfficialBuildRequired`/`CpAuthRequired`/`CpSignStaleClockSkew`/`CpSignReplayPersisted`/`CpSignVersionTooOld`/`CpUpgradeRequired`):是闭源发行构建(`codingplan-crypto` overlay)的 i18n 契约,有专门 `codingplan_crypto_tests` 双语守护且文案已中立化(断言含 BYO/distribution 指引)——按"门控不删除"铁律不动。
  - **NIT4(边缘可达死路文案)**:daemon `api_provider.rs` 五处 403 `"CodingPlan providers are managed by /login"` 按名/网关检测触发,中立构建仅当用户手写 `RustCode*` 保留名 provider 时可达——新增 `managed_provider_locked_message(action)` 按谓词二选一(托管:原文案;中立:保留名说明 + 改名/自带 api_key 指引),modify/replace/edit×2/delete 五处全切,陈旧注释同步;tuix `provider_panel.rs` 两条提示栏(`ProviderPanelManagedModelsHint` 含 "managed by /login"、`ProviderPanelManagedAccountHint` 提托管账号)新增 `*Neutral` arm(双语,键位提示尾部不变),两个渲染点按谓词选择,删除保护注释改中立。新测试 daemon `managed_locked_message_is_neutral_without_service`(4 个 action 全断言无 `/login` 且含 `api_key`)。
  - **NIT5(marketplace 插件 git 认证提示)**:`plugin/marketplace.rs::auth_required_message()` 可信主机+未登录臂提示 `/login`——可信域名默认空(`HOSTED_TRUSTED_DOMAINS: &[]`),仅 `RUSTCODE_TRUSTED_HOSTS` 显式 opt-in 可达;修复为 `!host_is_trusted(url) || !managed_login_available()` 均回落 SSH/git 凭证指引(与不可信主机同文案),文档注释补中立分支;新增 `relogin_hint()`,clone/pull 注入凭证重试仍失败的两处 bail 改用它——覆盖"中立构建残留旧发行版 `auth.toml` 导致 `auth_retry_args` 返回 Some、重试失败后提示 /login"这条**此前可达**的死路。新测试 `auth_required_message_neutral_build_never_pitches_login`(谓词 false 时两类 URL 均无 `/login` 且给 SSH 路径)。
  - **NIT6**:`rustcode-auth/src/lib.rs` Windows-only save-auth 错误上下文 `"please use /login again"` → `"please sign in again"`(该路径仅门控登录流可达;措辞改构建中立)。
  - **文档同步(第八轮文档审计 finding 5 收尾)**:`crates/rustcode-daemon/README.md` CodingPlan 节改标"仅发行版本"——`/codingplan/*` 位于 `#[cfg(feature = "codingplan")]` 门控后,默认构建不编译(404),引导走 `POST /providers`/`config.toml` BYO;源码树 `api_codingplan.rs` 注释补 cfg 说明。与 `site/docs/{zh,en}/headless-daemon.html` 既有口径一致。
  - **验证**:`cargo fmt --check` 通过;`cargo clippy --workspace --all-targets` exit 0;config 326/0、auth 42/0、capabilities 默认 823/0 且 `--features plugin` 978/0(新增 1)、tuix 2062/0、daemon 307/0(新增 1);全工作区 `cargo test --workspace --no-fail-fast` **5472 passed / 1 failed**,唯一失败仍为文档化已知红 `trust_key_golden_matches_core_algorithm`(DefaultHasher 不稳定,铁律禁止改)。期间 `/workspace` 磁盘 100% 满,删除可弃的 `target/debug/incremental`(15G)恢复,不影响产物正确性。
  - **LEAVE/deferred**:JetBrains 硬编码 zh/en 字符串迁移 bundle(SlashCommand labels 等;本机无 JDK/gradle,仅源码级审计);webui `settings.contextWindowLocked` 死键(零引用,留待顺手清理)。
- `ProviderConfig` / `ModelProfileConfig` 新增了**非 Option** 的 `model_mapping: ModelMapping`(空表即恒等映射),是**必填字段**。新增结构体字面量时必须显式给 `ModelMapping::default()`。
- `docs/architecture.md` 正文已改为 `rustcode-*` 命名;残留的 `rustcode` 字样仅在描述已退役 `rustcode-core` 的历史小节中(合法的历史名引用)。
- **[CHECK] 产品身份已锁定为 `rustcode`**(见 `docs/REFACTOR_DESIGN_PHASE1.md` §2.0 决策 D1),并已在 commit `6dbf57bb` 落地。不要再提议或先行改名。

## 高信号文档索引

- **`docs/phase1-refactor-design.md`** — PHASE-1 方案与 PHASE-2 缺口(设计决策、已知失败、缺陷清单)
- **`docs/CONTEXT.md`** — 运行时领域术语定义(注意:旧 `RustCode` 命名,但术语有效)
- **`docs/architecture.md`** — 架构描述(已更新为 `rustcode-*` 命名)
- **`docs/REFACTOR_DESIGN_PHASE1.md`** — 重命名前的基线设计(历史决策记录)
- **`rustcode-config/src/distribution.rs`** — 重命名事实源(HOME_ENV / 端口 / 进程名)
- **`rustcode-config/src/endpoints.rs`** — 环境变量名与托管端点
