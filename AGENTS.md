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

- ~~`extensions/` 旧名~~ — **已清零**(vscode/jetbrains 均已改为 rustcode,见 OBJECTIVE-5)。`site/` 无 atomcode/atomgit 残留;其中 `rustcode.dev`/`gitcode.com` CDN 链接属现品牌/静态站点内容,按需维护。
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
- **[WARN] 历史 docs 的表格 emoji 待清理(纯 cosmetic,非代码/UI/日志路径)**:`docs/hooks.md`、`docs/async-webhook-{summary,guide}.md`、`docs/hook-architecture.md`、`docs/hook-expansion-summary.md` 等历史文档仍用 `✅/❌/⚠️/✨` 作表格标记。G 门禁不 grep docs emoji,不影响运行时,且批量改表格标记有语义风险,留待专门文档扫尾(拟改 `[x]/[-]/[!]`)。本轮新写/改动的文件(dev-env、docker、quickstart、analysis、config-example)已全部 ASCII 标签化。
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
