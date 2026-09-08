# PHASE-1：系统分析与重构设计

> [SUPERSEDED BY docs/phase1-refactor-design.md] 本文档已被取代

> [INFO] 编排者设计文档，属于 GATEWAY 交付件。
> [INFO] 本设计经评审确认之前，不改动任何生产源码。
> [INFO] 源码根目录：`/workspace/gitCode/SecLab/RustCode`
> [INFO] 取证基线：分支 `main`，HEAD `287bff70f9400c24f4afd8fcf4762fbab63d8efa`，
>        工作区版本 `5.0.9`，edition `2021`，`crates/` 下共 14 个 crate。

---

## 0. 工程约束（来自编排规约）

| ID | 约束 | 执行方式 |
|----|-----------|-------------|
| C1 | 禁止 Unicode emoji。仅使用 ASCII 标签：`[INFO]`、`[WARN]`、`[ERROR]`、`[SUCCESS]`、`[AGENT-A..D]`、`[+]`、`[-]`、`[*]`、`[CHECK]` | AGENT-D 的 grep 门禁 |
| C2 | `tokio` async/await 一致性；异步路径中不得出现阻塞 I/O | `cargo clippy -- -W clippy::block_in_async` 建议项 |
| C3 | 代码、测试与默认配置中不得硬编码 API key / token / 内部端点 | AGENT-D 的密钥扫描门禁 |
| C4 | LLM 客户端必须同时支持 SSE 流式**与**非流式 | OBJECTIVE-3 缺口 G1 |
| C5 | 模块内错误用 `thiserror`，顶层上下文用 `anyhow`；生产路径上不得出现 `.unwrap()`/`.expect()` | AGENT-C + AGENT-D |
| C6 | 移除遥测后压缩共享依赖的 feature 开关；复核 `Cargo.lock` | AGENT-B + AGENT-D |

---

## 1. 系统分析：仓库实际是什么

### 1.1 工作区结构

```text
rustcode/                              (workspace root, resolver = "2")
  crates/  (members = ["crates/*"], default-members = cli, daemon, telemetry, tuix)
    rustcode-kernel/        L0  neutral agent loop, LlmProvider trait, StreamEvent
    rustcode-capabilities/  L1  provider adapters, tools, MCP, skills, sessions, compaction
    rustcode-coding/        L2  CodingRuntime lifecycle, provider factory, assembly
    rustcode-review/        L2  review specialization
    rustcode-tuix/          L3  terminal UI
    rustcode-cli/           L3  TUI + headless entry point (bin `rustcode`)
    rustcode-daemon/        L3  HTTP/SSE/WebSocket transport + legacy session importer
    rustcode-config/        leaf  disk/TOML config + endpoints + distribution names
    rustcode-auth/          leaf  OAuth login + gateway request signer
    rustcode-codingplan/    leaf  CodingPlan REST business layer
    rustcode-codingplan-crypto/  closed-source overlay stub (optional feature)
    rustcode-clix/          standalone bin `rustcodex`
    rustcode-updater/       self-update mechanics
    rustcode-telemetry/     [DELETED by OBJECTIVE-2]
  scripts/ docker/ .github/workflows/  install, release, CI
  docs/ site/ extensions/   documentation, docs site, vscode/jetbrains extensions
```

### 1.2 实测规模（在 HEAD 上 grep 核实）

| 指标 | 数值 |
|--------|-------|
| `rustcode` 字符串在 `crates/**/*.rs` 中的出现次数 | **6735** |
| `rustcode_telemetry` 引用 | **108** 处，分布于 **9 个 crate** 的 **24 个文件** |
| 不同的 `RUSTCODE_*` 环境变量 | **125** |
| `LlmProvider` 实现（生产） | 3 个适配器 + 1 个遥测装饰器；约 40 个测试替身 |
| 现有的 provider mock 集成测试 | `capabilities/tests/{anthropic_mock,ollama_mock,http_mock,e2e}.rs` |

### 1.3 运行时所有权（不得回退）

`CLI/TUI/daemon/headless/background/ACP/clix -> CodingRuntime -> kernel Agent`。
`CodingRuntime` 是活跃 agent、provider、会话绑定、generation、待处理请求、
快照 broker 与控制器的唯一所有者。按 `AGENTS.md`，驱动层不得再造第二个活跃
agent 生命周期所有者，`rustcode-core`/`rustcode-bridge` 也不得重现。重命名与
遥测剥离都是机械改动；而 LLM 重构触及一处生命周期接缝（`CodingRuntime` 组装
内部的 provider 构造），因此 AGENT-C 必须遵守 `AGENTS.md` 中的单一所有者与
fail-closed 规则。

---

## 2. OBJECTIVE-1：全局重命名映射

### 2.0 已锁定的标识决策（D1）

产品标识：**`rustcode`**。这是一个*记为假设的提案*；若被推翻，则只
需改动一处映射表即可（见 §2.7）。

| 维度 | 取值 |
|-------|-------|
| Cargo 包名前缀 | `rustcode-*` |
| 主 CLI 二进制 | `rustcode` |
| 守护进程二进制 / 库 | `rustcode-daemon` / `rustcode_daemon` |
| 独立 CLI 二进制 | `rustcodex` |
| 配置 / 数据目录 | `~/.rustcode` |
| 环境变量前缀 | `RUSTCODE_*` |
| Windows 安装目录 | `RustCode` |
| 发布产物前缀 | `rustcode` |
| 更新暂存前缀 | `.rustcode` |
| Repository 字段 | fork 的新 URL（上游记录于 `docs/UPSTREAM_CREDITS.md`） |

### 2.1 Crate 包

| 旧名 | 新名 | 备注 |
|-----|-----|------|
| `rustcode-auth` | `rustcode-auth` | |
| `rustcode-capabilities` | `rustcode-capabilities` | |
| `rustcode-cli` | `rustcode-cli` | `[package] name` + `[[bin]] name` + `[lib] name` |
| `rustcode-clix` | `rustcode-clix` | |
| `rustcode-coding` | `rustcode-coding` | |
| `rustcode-codingplan` | `rustcode-codingplan` | |
| `rustcode-codingplan-crypto` | `rustcode-codingplan-crypto` | 不在 `default-members` 中；feature `rustcode/codingplan-crypto` |
| `rustcode-config` | `rustcode-config` | |
| `rustcode-daemon` | `rustcode-daemon` | |
| `rustcode-kernel` | `rustcode-kernel` | |
| `rustcode-review` | `rustcode-review` | |
| `rustcode-telemetry` | **[DELETED]** | OBJECTIVE-2（整个 crate 删除） |
| `rustcode-tuix` | `rustcode-tuix` | |
| `rustcode-updater` | `rustcode-updater` | |

每次重命名都会触及：该 crate 自身的 `[package] name` / `[lib] name` /
`[[bin]] name`、其他所有 crate 的 path 依赖、`Cargo.lock`、根 `Cargo.toml`
（`default-members` 与 feature 接线），以及安装 / 发布脚本。

### 2.2 二进制

| 旧名 | 新名 | 定义位置 |
|-----|-----|-----------|
| `rustcode` | `rustcode` | `crates/rustcode-cli/Cargo.toml` |
| `rustcode-daemon` | `rustcode-daemon` | `crates/rustcode-daemon/Cargo.toml` |
| `rustcodex` | `rustcodex` | `crates/rustcode-clix/Cargo.toml` |
| `mcp-test-server` | 保持不变 | `crates/rustcode-capabilities/Cargo.toml` |

### 2.3 目录、文件与进程名

杠杆率最高的单个文件：**`crates/rustcode-config/src/distribution.rs`**。

| 常量 | 旧值 | 新值 |
|----------|-----|-----|
| `HOME_ENV` | `RUSTCODE_HOME` | `RUSTCODE_HOME` |
| `HOME_DIR_NAME` | `.rustcode` | `.rustcode` |
| `PROCESS_NAMES` | `["rustcode","rustcode-daemon"]` | `["rustcode","rustcode-daemon"]` |
| `WINDOWS_INSTALL_DIR` | `RustCode` | `RustCode` |
| `RELEASE_ASSET_PREFIX` | `rustcode` | `rustcode` |
| `UPDATE_TEMP_PREFIX` | `.rustcode` | `.rustcode` |

派生项必须与上表同步改动（卸载器会扫描更新器创建出来的内容），
即 `update_download_name()`、`update_rolling_name()`、`update_probe_name()` 三个函数。

同样在范围内：`docker/Dockerfile-*`（用户 `rustcode`、`/usr/local/bin/rustcode-daemon`、
`ENTRYPOINT`）、`docker/build-multiarch.sh` 的产物 glob、`docker/README.md`、
`.github/workflows/build.yml`（`dist/rustcode-*`）、`scripts/install.sh|ps1`、
`scripts/uninstall.sh|ps1`、`scripts/release*.sh` 与 `latest.json`。

### 2.4 环境变量

规则：全部 125 个名字一律 `RUSTCODE_X` -> `RUSTCODE_X`。例外如下：

| 变量 | 处理方式 |
|-----|--------|
| `RUSTCODE_TELEMETRY` | **DELETE**（按 OBJECTIVE-2 删除） |
| `RUSTCODE_TELEMETRY_ENDPOINT` | **DELETE**（按 OBJECTIVE-2 删除） |
| `RUSTCODE_TEST_EP_{WHOLE,ABSENT,URL,LIST,BOOL}` | 机械改名（仅测试内使用） |

[DECISION D2] **不做旧名回退。** 一次干净的 fork 在改名后不再读取 `RUSTCODE_*`；
`RUSTCODE_HOME` 未设置时只回退到 `~/.rustcode`。这对既有安装是不兼容变更，必须
在发布说明中明确写出。（备选方案：双读并在一个小版本内给出弃用告警 —— 对 fork
而言否决，理由是保持更小的表面，并避免出现第二条配置解析路径。）

与产品相关的子集（LLM / 网络），用于配置模板：

```text
RUSTCODE_API_KEY  RUSTCODE_BASE_URL  RUSTCODE_MODEL
RUSTCODE_ANTHROPIC_{BASE_URL,KEY,MODEL}
RUSTCODE_OLLAMA_{BASE_URL,MODEL}
RUSTCODE_LIVE_{BASE_URL,KEY,MODEL}
RUSTCODE_PROXY_MODE  RUSTCODE_TLS_MAX  RUSTCODE_USER_AGENT
RUSTCODE_STREAM_TIMEOUT_SECS  RUSTCODE_OFFLINE
RUSTCODE_WIRE_DUMP  RUSTCODE_WIRE_LOG_FILE
RUSTCODE_HOME  RUSTCODE_BRAND_NAME  RUSTCODE_OAUTH_PROVIDER_NAME
```

### 2.5 线上契约键（落盘 / 传输层使用，不只是展示字符串）

| 旧键 | 新键 | 位置 |
|---------|---------|----------|
| `rustcode.legacy_cold_summary` | `rustcode.legacy_cold_summary` | `kernel/src/message.rs` |
| `rustcode.user_interruption` | `rustcode.user_interruption` | `kernel/src/message.rs` |
| `rustcode-rewind-v1` | `rustcode-rewind-v1` | `capabilities/src/session/rewind.rs` |
| `rustcode-v1:`（ACP 游标） | `rustcode-v1:` | `cli/src/acp/discovery.rs` |
| `x-rustcode-session-id`（请求头） | `x-rustcode-session-id` | `openai_compat.rs`, `anthropic.rs`, `ollama.rs` |

[DECISION D3] 与 D2 一致：**改名且不保留兼容读取。** 既有的 `~/.rustcode`
会话将变为不可读。按不兼容变更写入文档；不新增导入器（新增会与 `AGENTS.md`
中"减少导入器"的规则相抵触）。

### 2.6 代码内的产品字符串

| 类别 | 示例 | 处理方式 |
|----------|----------|--------|
| 默认品牌 | `default_brand_name()`, `ui.brand_name`, `RUSTCODE_BRAND_NAME` | 重命名默认值；保留覆盖接缝 |
| User-Agent 回退值 | `DEFAULT_USER_AGENT = "rustcode"` (`provider/mod.rs`), `rustcode/<version>` in updater + config | 改名 |
| 仓库 / 发布 URL | `raw.gitcode.com/SecLab/RustCode/...`, `relay-rustcode.atomgit.com`, `rustcode-skills.git` (`config/src/endpoints.rs`) | 重新指向 fork 仓库；**保留** `atomgit.com` / `gitcode.com` 服务域名（独立品牌） |
| OpenRouter 归因 | `OPENROUTER_ATTRIBUTION_HEADERS` (`openai_compat.rs`) | 见 §4.3 G6 |
| 日志前缀 `[RustCode]` | `extensions/vscode`, `extensions/jetbrains` | 后续补丁；不在核心改名范围内 |

### 2.7 单点覆盖

整个改名由一张表数据驱动。若产品名变更，只需改动本文档 §2 的表与
机械化的 `s/rustcode/rustcode/` 替换；没有任何架构决策依赖字面字符串。

---

## 3. OBJECTIVE-2：零遥测

### 3.1 现状（已核实）

`crates/rustcode-telemetry/` —— 15 个文件、单个 crate，默认端点
`https://gateway.example.com/api/v1/events`（`telemetry/src/config.rs:6`），磁盘队列位于
`$RUSTCODE_HOME` 下，另含 gzip 上传器、UUID 标识、仓库来源探测、panic hook、首次运行提示。
`Cargo.toml` 与 `Cargo.lock` 中**不存在**任何 Sentry / PostHog / Segment /
OpenTelemetry / Mixpanel / Amplitude 依赖（[CHECK] 对所有 `*.toml` 与
`Cargo.lock` 的 grep 命中为零）。唯一的遥测就是这一个第一方 crate。

### 3.2 删除

```text
[-] crates/rustcode-telemetry/                     (entire crate, 15 files)
[-] root Cargo.toml: remove "crates/rustcode-telemetry" from default-members
[-] docs/telemetry.md                              (145 lines, documents a removed feature)
```

### 3.3 调用点中和 —— 精确清单

共 108 处 `rustcode_telemetry` 引用，分布于 9 个 crate 的 24 个文件。

| Crate | 文件 | 引用数 | 处理方式 |
|-------|------|-----:|--------|
| config | `config/mod.rs` | 1 处 import + 1 个字段 + render 函数 | 删除 `telemetry: TelemetryConfig`、`render_telemetry_section` 与对应的 `use` |
| config | `settings.rs` | 3 | 删除 `telemetry.enabled` 设置行及其 match 分支 |
| config | `store.rs` | 2 | 从写出的配置中移除 `[telemetry]` 块，并移除禁用写入逻辑 |
| config | `i18n/{en,zh_cn,messages}.rs` | ~12 | 删除 `CliAboutTelemetry*`、`CliHelpNoTelemetry`、`CliHelpClient`；调整离线文案行 |
| config | `Cargo.toml` | 1 | **移除该依赖**（这是依赖图上的收益，见 §3.5） |
| auth | `oauth.rs` | 2 | 从 `login()` / `LoginSession::finish()` 移除 `tel: Option<&Arc<Telemetry>>`；去掉 `pending_invite` 的使用 |
| codingplan | `setup.rs` | 5 | 移除 `tel` 参数与 3 处 `TakeCodingplan` 埋点 |
| coding | `telemetry.rs` | 整个模块 | 删除 `TelemetryHook`、`ToolTelemetryMiddleware`、`MeteredProvider` |
| coding | `config.rs` | 4 | 去掉 `telemetry: Option<Arc<Telemetry>>` 与 `SubagentTelemetryProviderFactory` |
| coding | `parts.rs` | 4 处代码块 | 移除 `MeteredProvider` 包装（host/review/subagent）、中间件与 hook 注册，以及 `set_telemetry_provider_factory` |
| coding | `lib.rs` | 2 | 移除 `pub mod telemetry;` 及 re-export |
| cli | `main.rs` | 17+ | 移除 import、`--no-telemetry`、`telemetry` 子命令与 `TelemetryAction`、初始化块、`install_panic_hook`、`SessionMode` 映射，以及全部 `track(...)` / flush 调用 |
| cli | `telemetry_cmd.rs` | 整个文件 | 删除 |
| cli | `tests/telemetry_cmd.rs` | 整个文件 | 删除 |
| clix | `tel.rs` | 整个文件 | 删除（sink、notice、`meter_provider`）；`build_review_provider` 迁至 `code.rs` |
| clix | `main.rs` | 10 | 移除 sink 接线与 `--no-telemetry` |
| daemon | `lib.rs` | 13 | 移除 `AppState.telemetry`、初始化与 flush |
| daemon | `live_api.rs` | 11+ | 移除 4 个 config builder 上的 `telemetry` 参数；替换 `Extension<SessionMode>`（见 §3.4） |
| daemon | `api_codingplan.rs`, `api_auth.rs` | 4+ | 移除全部 `track(Event::TakeCodingplan{..})` / `login_success` 持久化入队 |
| daemon | `telemetry_scope.rs` | 整个文件 | 删除 `daemon_scope()` 包装 |
| daemon | `commands.rs`, `native_live.rs`, `kernel_runtime.rs`, `main.rs` | 4 | 移除 `telemetry` 字段 / 参数 / CLI flag / 测试初始化 |
| tuix | `lib.rs`, `event_loop/mod.rs`, `event_loop/commands.rs`, `event_loop/oauth_poll.rs`, `modals/onboarding_wizard.rs` | 4 个文件 | 移除 `LoopCtx.telemetry`、`bind_telemetry_to_session`、`UseCommand` 埋点与 `Event` import |

### 3.4 非显而易见的移除项（漏掉就编译不过）

1. **`rustcode_telemetry::SessionMode` 在 daemon 的 HTTP API 中是一个*类型*。**
   `daemon/src/api_codingplan.rs:325` 与 `daemon/src/api_auth.rs:250` 声明了
   `Extension<rustcode_telemetry::SessionMode>`；`live_api.rs:1698` 消费它。删除该
   crate 会连带移除这个类型，而不只是一次调用。
   [+] 替代方案：本地 `enum ClientMode { Headless, Tui, Ide, Vscode, Jetbrains, Webui, Desktop, Channel }`
   定义于 `daemon/src/client_mode.rs`，仍从同一个请求头解析。线上词表须保持逐字节
   一致（`"webui"`、`"rustcode_desktop"` -> 仅当在同一次改动中同步修补了扩展客户端时才改为
   `"rustcode_desktop"`；否则保留旧标签，改名留到后续跟进）。**建议：保留标签，
   只改 Rust 类型名** —— 避免与 `extensions/` 之间出现静默的协议破坏。

2. **Panic hook。** `install_panic_hook(telemetry)` 会上传 `Event::Panic`。改为一个
   不依赖遥测的 hook：打印到 stderr 并写本地崩溃日志，不做任何网络发送。

3. **`rustcode_coding::SessionMode` 是*另一个*类型**（`live_api.rs:441` 使用的是
   `rustcode_coding::SessionMode::ExternalSnapshot`）。不要把两者混为一谈 —— 全仓
   233 处 `SessionMode` grep 命中同时覆盖这两个类型以及测试中的用法。

4. **`rustcode-config` 是叶子 crate，当前依赖 `rustcode-telemetry`。**
   移除该依赖同时也是依赖方向的改善，而不仅仅是一次删除（§3.5）。

5. **离线 / 提示耦合。** `RUSTCODE_OFFLINE` 与"首次运行提示"逻辑和遥测初始化纠缠在
   一起（`main.rs:1450-1494`）。离线模式必须保留，只删除其中的遥测分支。

### 3.5 依赖清理（TIP-2）

删除之后，下列依赖将不再被引用，必须从那些仅为遥测而持有它们的 crate 中剪除
（用 `cargo tree -i` 确认，不要凭猜测）：

```text
[-] crates/rustcode-telemetry/Cargo.toml deps:
    flate2, fs2, filetime, url, regex, dirs, chrono, uuid, reqwest, tracing, thiserror
[-] rustcode-config: drops rustcode-telemetry -> drops reqwest/rustls/flate2/uuid from
    every config-only build (this is what capabilities/Cargo.toml:26 warns about)
[-] dev-deps that existed only for telemetry tests: wiremock (coding/clix/daemon dev slots),
    rustcode-telemetry's `test-util` feature
[-] `test-util` feature of rustcode-telemetry: gone with the crate
[CHECK] reqwest feature flags re-audit across crates: keep
    ["stream","json","rustls-tls"] for the provider path; do NOT re-enable
    default features. rustls-tls (webpki base) must stay — it is the #514 backstop.
[CHECK] tokio features: telemetry required ["rt","sync","time","fs","macros"];
    fs was pulled for the queue. Re-audit per crate after removal.
[CHECK] Cargo.lock: `cargo update --workspace` then grep for any orphaned
    flate2/fs2/filetime entries.
```

### 3.6 行为差异（必须写进文档，不得静默丢弃）

| 被移除的表面 | 替代方案 |
|-----------------|-------------|
| `--no-telemetry` flag (CLI + daemon) | 已移除；没有可禁用的东西了 |
| `rustcode telemetry {status,enable,disable,dump,clear,recover}` subcommand | 已移除 |
| `[telemetry]` config section | 已移除；旧 config.toml 中残留的 `[telemetry]` 键**不得**导致加载失败 —— 按忽略未知键处理，或在加载器中剥离 |
| `telemetry.enabled` setting row | 已移除 |
| login/use-command/LLM-chat/panic events | 已移除 |
| offline-mode notice text mentioning telemetry | 改写文案 |

[DECISION D4] 含有 `[telemetry]` 的 config.toml 仍必须能**加载**（该结构体并未启用
serde 的 `deny_unknown_fields`；由 AGENT-B 复核）。删除字段后该键会被静默忽略，
这是可接受的，但必须有回归测试覆盖。

---

## 4. OBJECTIVE-3：LLM provider 解耦

### 4.1 现状审计（对照 HEAD 核实）

**已经做好、不要重造**的部分：

| 资产 | 位置 | 结论 |
|-------|----------|---------|
| 中性 provider trait | `kernel/src/provider.rs` `LlmProvider` (`model_name`, `context_window`, `bind_session_id`, `chat_stream`) | [SUCCESS] 保留为接缝 |
| OpenAI 兼容适配器 | `capabilities/src/provider/openai_compat.rs` | [SUCCESS] `/chat/completions`、SSE、tool-call 缓冲、重试、TLS 兜底、SwappableClient |
| Anthropic adapter | `capabilities/src/provider/anthropic.rs` | [SUCCESS] `/v1/messages`、按事件类型分发的 SSE 解码器、签名 thinking 往返、确定性请求体 |
| Ollama 适配器 | `capabilities/src/provider/ollama.rs` | [SUCCESS] |
| 工厂 / 分发 | `coding/src/provider_factory.rs` `DefaultCodingProviderFactory::build` | [SUCCESS] 结构可用；缺口见下 |
| 重试策略 | `capabilities/src/provider/retry.rs` | [SUCCESS] backoff、Retry-After、重放敏感门禁 |
| 环境变量代理 | `capabilities/src/proxy.rs` + reqwest 环境探测 | [SUCCESS] `RUSTCODE_PROXY_MODE` + `HTTP(S)_PROXY` |
| Mock 集成测试 | `capabilities/tests/{anthropic_mock,ollama_mock,http_mock,e2e}.rs` | [SUCCESS] **扩展，不要重复建设** |
| 超时旋钮 | `connect_timeout` / `open_timeout` (TTFB) / `idle_timeout` | [SUCCESS] 已按适配器分别配置 |
| TLS | `skip_tls_verify`, `RUSTCODE_TLS_MAX`, webpki 兜底 | [SUCCESS] |

**对前一版草稿的更正：**"AtomGit 请求签名器施加于每一个 OpenAI 兼容构建"的说法
是**错误的**。`AtomGitProviderAuthenticator::request_signer` 只有在
`is_atomgit_gateway(base_url)` 成立时才返回非空，否则返回 `Ok(None)`；
`is_atomgit_gateway == rustcode_config::endpoints::is_codingplan_llm_gateway`，
该判定基于域名且仅限 HTTPS（`gateway.example.com`、`pre-llm-api-cce.atomgit.com`、
`api-ai.gitcode.com`，或 `RUSTCODE_CODINGPLAN_LLM_BASE_URL` 的来源）。自定义
`base_url` 本来就已走朴素的 `bearer_auth(api_key)`。AGENT-C **不得**去"修正"这一点。

### 4.2 缺口清单（真正要做的事）

| ID | 缺口 | 严重度 | 修复方式 |
|----|-----|----------|-----|
| **G1** | **缺少非流式 API。** `LlmProvider` 只暴露 `chat_stream`。规范（[STREAMING]）要求原生支持非流式。 | **高** | 新增 `chat()`，默认实现折叠流；适配器可用真实的非流式动词覆盖（§4.4） |
| **G2** | 没有按 provider 自定义请求头（`extra_headers`）。 | 高 | 加入 transport 配置，并在三个适配器中注入 |
| **G3** | 没有按 provider 配置代理，只有环境 / 进程级全局代理。 | 中 | 增加 `proxy: Option<String>`；用 `reqwest::Proxy::all()`，该客户端绕过环境变量代理 |
| **G4** | 没有模型映射表。 | 中 | 增加 `model_mapping: HashMap<String,String>`，在工厂中一次性解析 |
| **G5** | `provider_type: String` 分发：`"anthropic-compatible"` 当前落入 OpenAI 兜底分支（`provider_factory.rs:112-169`）。 | 中 | 在 config crate 中定义显式 `ProviderKind` 枚举；兜底分支保持 OpenAI 行为并输出 `tracing::warn!` |
| **G6** | `OPENROUTER_ATTRIBUTION_HEADERS` 硬编码了上游身份（`HTTP-Referer: https://gitcode.com/atomgit_atomcode/atomcode`、`X-OpenRouter-Title: RustCode`）。 | 中 | 默认删除；保留 `is_openrouter_url` 门禁；取值改为配置驱动且需 opt-in —— **[DONE]** 归因头默认全部不发,由 `RUSTCODE_OPENROUTER_ATTRIBUTION=1\|true\|on\|yes` opt-in;`HTTP-Referer` 永不硬编码 host,另由 `RUSTCODE_OPENROUTER_REFERER` 单独 opt-in;host 门禁与 userinfo 冒用防护保留并有测试覆盖 |
| **G7** | 没有统一的错误映射。每个适配器各自临时构造 `ProviderError { retryable, message, http_status, code, retry_after_secs }`；`friendly_http_error()` 虽已共享但不完整。 | 中 | 引入 `LlmError`（`thiserror`），并在 kernel 边界提供双向 `From` 转换（§4.5） |
| **G8** | Anthropic 配置缺少与 OpenAI 配置对等的 `extra_headers` / `proxy`。 | 低 | 统一到同一个 transport 结构体 |

### 4.3 解耦策略

[DECISION D5] **签名保留，但仍限于域名门禁。** `ProviderAuthenticator` 继续作为
`DefaultCodingProviderFactory` 上的可选注入项。不接触 AtomGit 的 fork 只需从不
调用 `.with_authenticator(...)`。源码构建下 `signer_available() == false` 已经会
产生显式的 `SourceBuildGatewayUnsupported` 错误，而不是静默降级 —— 这种
fail-closed 行为是正确的，必须保留。

[DECISION D6] **预置默认值。** 不内置任何指向托管网关的 `base_url` 默认值。缺少
`base_url` 的 provider 条目在构建时以显式的 "base_url is required" 错误失败
（按 `AGENTS.md` 的 fail-closed 要求），唯一例外是 `ollama`，它可以保留
`http://localhost:11434`。

### 4.4 Trait 设计（Rust 草稿）

文件：`crates/rustcode-kernel/src/provider.rs`（扩展，不要分叉）。

```rust
// ---- Non-streaming: added to the existing neutral seam -------------------

/// A complete, non-streamed turn. The adapter fills what its backend returned;
/// fields are never synthesized.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChatResponse {
    pub text: String,
    pub reasoning: Option<String>,
    pub tool_calls: Vec<ToolCall>,
    pub usage: Option<TokenUsage>,
    pub finish_reason: FinishReason,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FinishReason {
    #[default]
    Stop,
    Length,        // finish_reason == "length" / stop_reason == "max_tokens"
    ToolCalls,
    ContentFilter,
    Other,
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn model_name(&self) -> &str;
    fn context_window(&self) -> u32 { 0 }
    fn bind_session_id(&self, _session_id: &str) {}

    /// Streaming turn. Unchanged.
    async fn chat_stream(
        &self,
        messages: &[Message],
        tools: &[ToolDef],
        options: &ChatOptions,
    ) -> Result<BoxStream<'static, StreamEvent>, ProviderError>;

    /// Non-streaming turn.
    ///
    /// DEFAULT IMPLEMENTATION folds [`Self::chat_stream`], so no existing adapter
    /// or test double has to change and the kernel turn loop is unaffected. An
    /// adapter overrides this only when its backend has a cheaper non-stream verb
    /// (`stream: false` for OpenAI-compatible; Anthropic has no non-stream SSE
    /// discount, so it keeps the default path).
    ///
    /// Contract: the first `StreamEvent::Error` terminates the fold and is returned
    /// as `Err`. `StreamEvent::Malformed` is skipped, never fatal.
    async fn chat(
        &self,
        messages: &[Message],
        tools: &[ToolDef],
        options: &ChatOptions,
    ) -> Result<ChatResponse, ProviderError> {
        let mut stream = self.chat_stream(messages, tools, options).await?;
        let mut out = ChatResponse::default();
        while let Some(ev) = stream.next().await {
            match ev {
                StreamEvent::TextDelta(t) => out.text.push_str(&t),
                StreamEvent::Reasoning(r) => out.reasoning.get_or_insert_with(String::new).push_str(&r),
                StreamEvent::ToolCall(tc) => out.tool_calls.push(tc),
                StreamEvent::Usage(u) => out.usage = Some(u),
                StreamEvent::Done { truncated } => {
                    out.finish_reason =
                        if truncated { FinishReason::Length } else { FinishReason::Stop };
                    break;
                }
                StreamEvent::Error(e) => return Err(e),
                StreamEvent::Malformed
                | StreamEvent::ResponseId(_)
                | StreamEvent::ResponseModel(_)
                | StreamEvent::ReasoningSignature { .. }
                | StreamEvent::ToolCallDelta { .. } => {}
            }
        }
        if !out.tool_calls.is_empty() {
            out.finish_reason = FinishReason::ToolCalls;
        }
        Ok(out)
    }
}
```

[CHECK] `chat()` 可用后应当改用它的非流式调用点（已识别为一次性 LLM 调用）：
`coding/src/session_title.rs`、`coding/src/next_prompt_suggestion.rs`、
`capabilities/src/compaction.rs`（摘要）与 `review`。**这属于 PHASE-2 步骤 2.3
的范围之外**，除非评审者另行要求；AGENT-C 只交付接缝，调用点迁移是后续任务。

文件：`crates/rustcode-capabilities/src/provider/transport.rs`（新建，所有适配器共用）。

```rust
/// Everything the HTTP adapters need, independent of the wire protocol.
/// Secrets are only ever read from config/env; nothing is hard-coded.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderTransport {
    pub base_url: String,
    pub api_key: String,
    /// Extra request headers (gateway tenancy, routing tags, vendor-specific auth).
    /// Injected on EVERY request of this provider, after auth headers, so a
    /// caller CANNOT overwrite `authorization` / `x-api-key` / `anthropic-version`.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub extra_headers: HashMap<String, String>,
    /// Per-provider proxy. `None` => follow the process proxy policy
    /// (`RUSTCODE_PROXY_MODE` + `HTTP(S)_PROXY`). `Some(url)` => this client uses
    /// `reqwest::Proxy::all(url)` and IGNORES env proxies for this provider only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxy: Option<String>,
    #[serde(default)]
    pub skip_tls_verify: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    #[serde(default = "defaults::connect_timeout")] pub connect_timeout: Duration,
    #[serde(default = "defaults::open_timeout")]    pub open_timeout: Duration,
    #[serde(default = "defaults::idle_timeout")]    pub idle_timeout: Duration,
    #[serde(default)] pub retry: RetryPolicy,
}

/// Header names a caller may not override via `extra_headers` (auth integrity).
const RESERVED_HEADERS: &[&str] = &[
    "authorization", "x-api-key", "anthropic-version", "content-type",
    "content-length", "host", "accept-encoding",
];

impl ProviderTransport {
    /// Reject a mapping that tries to clobber auth/transport headers.
    /// Fail-closed at construction, per AGENTS.md.
    pub fn validate(&self) -> Result<(), LlmError> { /* ... */ }

    pub fn http_client(&self) -> Result<reqwest::Client, LlmError> {
        let mut b = crate::proxy::apply_async_proxy_policy(reqwest::Client::builder())
            .connect_timeout(self.connect_timeout)
            .pool_idle_timeout(retry::POOL_IDLE_TIMEOUT)
            .user_agent(self.user_agent.as_deref().unwrap_or(DEFAULT_USER_AGENT));
        if let Some(p) = &self.proxy {
            b = b.proxy(reqwest::Proxy::all(p).map_err(|e| LlmError::Config(e.to_string()))?);
        }
        if self.skip_tls_verify {
            b = b.danger_accept_invalid_certs(true);
        }
        b.build().map_err(|e| LlmError::Config(e.to_string()))
    }
}
```

文件：`crates/rustcode-config/src/config/provider.rs`（新增字段 + kind 枚举）。

```rust
/// Wire protocol family. Replaces the free-form `provider_type: String` at the
/// dispatch boundary; the string form is still accepted on read via aliases.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderKind {
    #[serde(alias = "openai", alias = "openai-compatible", alias = "openai_compatible")]
    OpenAiCompatible,
    #[serde(alias = "anthropic", alias = "claude", alias = "anthropic-compatible")]
    Anthropic,
    #[serde(alias = "ollama")]
    Ollama,
}

/// Requested model -> wire model. Resolved once in the provider factory, so no
/// adapter or driver sees the alias. Empty => identity.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ModelMapping(pub HashMap<String, String>);

impl ModelMapping {
    pub fn resolve<'a>(&'a self, requested: &'a str) -> &'a str {
        self.0.get(requested).map(String::as_str).unwrap_or(requested)
    }
}

// --- additive fields on ProviderConfig / ModelProfile / ResolvedModelConfig ---
//   pub extra_headers: Option<HashMap<String, String>>
//   pub proxy: Option<String>
//   pub model_mapping: ModelMapping
```

工厂分发（`coding/src/provider_factory.rs`）修复后的形态：

```rust
let kind = ProviderKind::from_str(&cfg.provider_type).unwrap_or_else(|| {
    tracing::warn!("unknown provider_type {:?}; treating as openai-compatible", cfg.provider_type);
    ProviderKind::OpenAiCompatible
});
let model = cfg.model_mapping.resolve(&cfg.model).to_string();  // one place, all adapters
match kind {
    ProviderKind::Anthropic            => { /* AnthropicConfig  + transport */ }
    ProviderKind::Ollama               => { /* OllamaConfig     + transport */ }
    ProviderKind::OpenAiCompatible     => { /* OpenAiCompatConfig + transport */ }
}
```

### 4.5 错误映射（TIP-1、TIP-3）

文件：`crates/rustcode-capabilities/src/provider/error.rs`（新建）。

```rust
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("authentication rejected (HTTP {status}): {detail}")]
    Auth { status: u16, detail: String },
    #[error("rate limited (HTTP 429); retry after {retry_after_secs:?}s")]
    RateLimited { retry_after_secs: Option<u64> },
    #[error("invalid request (HTTP {status}): {detail}")]
    InvalidRequest { status: u16, detail: String },
    #[error("upstream error (HTTP {status}): {detail}")]
    Upstream { status: u16, detail: String },
    #[error("transport failure: {0}")]      Transport(String),
    #[error("stream idle for {0:?}")]       IdleTimeout(Duration),
    #[error("response decode failure: {0}")] Decode(String),
    #[error("provider misconfiguration: {0}")] Config(String),
}

impl LlmError {
    pub fn retryable(&self) -> bool {
        matches!(self, Self::RateLimited { .. } | Self::Upstream { .. } | Self::Transport(_))
    }
    /// Wire a kernel `ProviderError` into the typed set. `code` carries the
    /// vendor's own error type (e.g. Anthropic `overloaded_error`).
    pub fn from_provider(e: &ProviderError, status: Option<u16>) -> Self { /* ... */ }
}

impl From<LlmError> for ProviderError {
    fn from(e: LlmError) -> Self {
        ProviderError {
            retryable: e.retryable(),
            message: e.to_string(),
            http_status: /* from variant */,
            code: None,
            retry_after_secs: /* RateLimited only */,
        }
    }
}
```

顶层（`cli`、`daemon`、`clix`）用 `anyhow::Context` 包装，且绝不 `.unwrap()`。
现有的 `friendly_http_error()`（401/402 的标题文案、CodingPlan 403 提示，以及
kernel 限流路径会剥离的 `HTTP 429: ` 字面前缀）**逐字保留** —— 它对 TUI 与
`rate_limit_server_message` 都是承重逻辑。

### 4.6 SSE 归一化（TIP-3 对照表）

| 内部 `StreamEvent` | OpenAI-compatible | Anthropic |
|---|---|---|
| `TextDelta` | `choices[].delta.content` | `content_block_delta` / `text_delta` |
| `Reasoning` | `choices[].delta.reasoning_content`（按策略门禁） | `content_block_delta` / `thinking_delta` |
| `ReasoningSignature` | (n/a) | `content_block_stop` on `thinking` / `redacted_thinking` |
| `ToolCall`（整体） | 在 `finish_reason == "tool_calls"` 时发出 | 在 `tool_use` 的 `content_block_stop` 时发出 |
| `ToolCallDelta`（实时） | `delta.tool_calls[].function.arguments` | `content_block_delta` / `input_json_delta` |
| `Usage` | 取最后一个非空 `usage` | `message_start` 的 input + `message_delta` 的 output；`message_stop` 时发出 |
| `Done{truncated}` | `finish_reason` | `message_stop`; `max_tokens` -> `truncated: true` |
| `Error` | 流中途的 chunk 错误 | `event: error` |
| `ResponseId` | `id` | `message_start.message.id` |

[CHECK] 两个适配器中的一致性已实现并有单元测试覆盖（`anthropic.rs` 的 `tests`、
`openai_compat.rs` 的 `tests`）。AGENT-C 需新增一个共用的一致性测试，断言*同一份*
录制对话在两个适配器中产生*相同的* `StreamEvent` 序列 —— 这才是跨协议保证，
而它目前没有测试覆盖。

### 4.7 配置结构（用于 `config.example.toml`）

```toml
# ---- OpenAI-compatible (any gateway / relay / self-hosted) ---------------
[providers.my-openai]
type           = "openai-compatible"        # or: openai
base_url       = "https://my-gateway.example.com/v1"
api_key        = "env:MY_OPENAI_KEY"        # env: indirection; never a literal key
model          = "gpt-4o"
context_window = 128000
extra_headers  = { "X-Tenant" = "acme", "X-Experiment" = "sse-v2" }
proxy          = "http://corp-proxy.example.com:3128"
skip_tls_verify = false
timeout_connect_secs = 30
timeout_open_secs    = 90
timeout_idle_secs    = 120
retry_max_attempts   = 3

[providers.my-openai.model_mapping]
"fast"   = "gpt-4o-mini"
"smart"  = "gpt-4o"

# ---- Anthropic native /v1/messages ---------------------------------------
[providers.my-anthropic]
type     = "anthropic-compatible"           # or: anthropic | claude
base_url = "https://my-anthropic-gw.example.com"
api_key  = "env:MY_ANTHROPIC_KEY"
model    = "claude-opus-4"
thinking_enabled = false
```

[SECURITY] `api_key` 支持 `env:` 间接引用，因此凭据不会落进配置文件或仓库。
默认模板只带占位值；AGENT-D 的密钥扫描会强制校验这一点。

---

## 5. OBJECTIVE-4：许可证与合规

### 5.1 现状

根 `LICENSE` 为 MIT，`Copyright (c) 2026 Yubang Xu`。上游来源
`https://gitcode.com/SecLab/RustCode`；fork 来源
`https://gitcode.com/SecLab/RustCode`。

工作区中已存在三份文档（未跟踪，由早前一轮产出）：`docs/ORIGINAL_LICENSE.md`
（33 行）、`docs/THIRD_PARTY_NOTICES.md`（36 行）、`docs/UPSTREAM_CREDITS.md`
（40 行）。已对照 TIP-4 核实内容正确：上游 MIT 文本逐字保留，版权声明完整，
fork 自身的改动有描述且未声称对任何内容进行重新授权。

### 5.2 动作项

```text
[+] Root LICENSE: keep MIT, keep "Copyright (c) 2026 Yubang Xu" verbatim, APPEND a second
    copyright line for the fork. Do not delete or edit the upstream notice.
[+] docs/ORIGINAL_LICENSE.md      — verbatim upstream MIT. KEEP AS IS.
[+] docs/THIRD_PARTY_NOTICES.md   — upstream attribution + pointer to `cargo license`. KEEP.
[+] docs/UPSTREAM_CREDITS.md      — fork statement + change summary. KEEP (update after PHASE-2).
[+] New modules authored by the fork carry a short header:
        // Copyright (c) 2026 The rustcode authors. MIT. Derived from rustcode (MIT,
        // Copyright (c) 2026 Yubang Xu) — see docs/UPSTREAM_CREDITS.md.
[-] Do NOT strip notices from inherited files.
[-] Do NOT add a license-incompatible dependency.
[!] `crates/rustcode-codingplan-crypto/` is an open-source placeholder for a CLOSED-SOURCE
    signer overlaid only by the upstream release pipeline. A self-built binary therefore
    cannot sign AtomGit gateway requests (`signer_available() == false`). This is
    pre-existing upstream behavior, it fails closed, and it is out of scope — but it must
    be stated in the fork README so users are not surprised.
```

[CHECK] `THIRD_PARTY_NOTICES.md` 目前把传递依赖的 crate 清单委托给 `cargo license`。
这是可接受的，且能自动保持准确，不存在会腐化的静态清单。

---

## 6. 子代理执行计划

### 6.1 文件归属（互不重叠）

| 代理 | 归属文件 | 步骤 |
|-------|------|------|
| **[AGENT-A]** 架构与合规 | `docs/ORIGINAL_LICENSE.md`, `docs/THIRD_PARTY_NOTICES.md`, `docs/UPSTREAM_CREDITS.md`, root `LICENSE`; root `Cargo.toml`; all `crates/*/Cargo.toml` (package/lib/bin names, path deps, `default-members`); `config/src/distribution.rs`; `config/src/endpoints.rs`; `docker/*`; `.github/workflows/build.yml`; `scripts/install.*`, `scripts/uninstall.*`, `scripts/release*.sh`; `latest.json` | 2.1 |
| **[AGENT-B]** 遥测剥离 | delete `crates/rustcode-telemetry/`, `docs/telemetry.md`; `coding/src/{lib.rs,telemetry.rs,config.rs,parts.rs}`; `cli/src/main.rs`, `cli/src/telemetry_cmd.rs`, `cli/tests/telemetry_cmd.rs`; `auth/src/oauth.rs`; `codingplan/src/setup.rs`; `config/src/{config/mod.rs,settings.rs,store.rs,i18n/*,Cargo.toml}`; `daemon/src/*` (+ new `daemon/src/client_mode.rs`); `clix/src/{main.rs,code.rs,tel.rs}`; `tuix/src/*`; i18n message enums | 2.2 |
| **[AGENT-C]** LLM 引擎构建 | `kernel/src/provider.rs` (`chat()`, `ChatResponse`, `FinishReason`); new `capabilities/src/provider/{transport.rs,error.rs}`; `capabilities/src/provider/{openai_compat.rs,anthropic.rs,ollama.rs,mod.rs}`; `config/src/config/provider.rs`; `coding/src/provider_factory.rs`; `docs/config.example.toml` | 2.3 |
| **[AGENT-D]** 质量与验证 | `cargo fmt`, `cargo check --workspace --all-targets`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`; extend `capabilities/tests/{anthropic_mock,ollama_mock,http_mock}.rs`; new `capabilities/tests/provider_conformance.rs`; new `capabilities/tests/nonstream.rs`; C1/C3 grep gates | 2.4 |

### 6.2 顺序与合并纪律

```text
STEP 2.1  [AGENT-A]  rename + license
          |
          +-- (must land first: every later agent edits files whose crate names changed)
          |
STEP 2.2  [AGENT-B]  telemetry strip
          |
          +-- overlaps AGENT-C ONLY in `coding/src/provider_factory.rs` and
          |   `config/src/config/provider.rs`, and in DISJOINT functions.
          |   Rule: B deletes the `telemetry` FIELD, C adds `extra_headers` /
          |   `proxy` / `model_mapping` FIELDS. If both run in parallel, C rebases
          |   and re-applies only its own field additions.
          |
STEP 2.3  [AGENT-C]  LLM engine  (recommended: run AFTER 2.2 to avoid the field collision)
          |
STEP 2.4  [AGENT-D]  validation over the whole tree
```

[WARN] 建议的串行顺序：**2.1 -> 2.2 -> 2.3 -> 2.4**。A 与 B 改动的文件互不重叠，
原则上可以并行；但 B 会改 `config/src/config/mod.rs`，而 A 会改 `config/Cargo.toml`
与 `distribution.rs`，风险虽低却不为零（Cargo.toml 与模块改动并存）。
把 C 排在 B 之后串行执行，即可消除唯一的真实冲突。

### 6.3 变更影响分析

| 变更 | 影响半径 | 风险 | 缓解措施 |
|--------|--------------|------|------------|
| Crate 重命名 | 14 个 `Cargo.toml`、`Cargo.lock`，以及按 import 路径计约 439 个 `.rs` 文件 | 低（机械改动） | 单次 `sed` 替换，每批 crate 之后跑 `cargo check` |
| `RUSTCODE_HOME` / `.rustcode` | 每个用户的既有安装 | **高（数据）** | D2/D3：按不兼容变更写文档；不得静默读取 `~/.rustcode` |
| 遥测 crate 删除 | 24 个文件、9 个 crate | 中 | §3.4 的四项非显而易见条目即失败模式；每一项都有具名检查 |
| 从 daemon 移除 `SessionMode` 类型 | daemon HTTP API | 中 | 本地 `ClientMode` 枚举，**线上标签不变** |
| 旧配置中残留 `[telemetry]` 键 | 配置加载器 | 低 | D4：仍须能加载；补回归测试 |
| `LlmProvider` 新增 `chat()` | 约 40 个测试替身 + 3 个适配器 | **低** —— 有默认实现，故不需要任何改动 | 断言改动前后 `cargo test` 均为绿 |
| `ProviderKind` 分发 | 每一个已配置的 provider | 中 | 别名保证旧字符串继续可用；未知值 -> 按 OpenAI 处理并告警（行为不变） |
| `extra_headers` 保留名校验 | 仅新增表面 | 低 | 构造时校验，fail-closed |
| Anthropic `thinking` / 签名块回显 | 对话重放 | **一旦触碰即为高** | AGENT-C **不得**触碰 `format_assistant_message` / `ReasoningSignature`；它与 provider 绑定且测试密集 |

---

## 7. 验证计划（AGENT-D）

```bash
# [CHECK] 1 — compile, every target
cargo check --workspace --all-targets

# [CHECK] 2 — lint (C2: no blocking I/O in async paths)
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --workspace --all-targets -- -W clippy::await_holding_lock

# [CHECK] 3 — tests
cargo test --workspace

# [CHECK] 4 — rename residue (expect: only intentional upstream-URL/atomgit-service hits)
grep -rn "rustcode" --include=*.rs --include=*.toml crates/ | grep -v "atomgit\|gitcode" || true

# [CHECK] 5 — telemetry residue (expect: EMPTY)
grep -rni "telemetry\|analytics\|posthog\|sentry\|segment" --include=*.rs --include=*.toml crates/
grep -n "rustcode-telemetry\|rustcode-telemetry" Cargo.lock || true

# [CHECK] 6 — C1: no emoji in rust sources
grep -rnP "[\x{1F300}-\x{1FAFF}\x{2600}-\x{27BF}]" --include=*.rs crates/ || true

# [CHECK] 7 — C3: no hardcoded secrets in the tracked tree
grep -rnE "(sk-[A-Za-z0-9]{20,}|sk-ant-[A-Za-z0-9_-]{20,}|ghp_[A-Za-z0-9]{20,})" \
     --include=*.rs --include=*.toml --include=*.md crates/ docs/ || true

# [CHECK] 8 — C5: no unwrap/expect on production paths (tests excluded)
grep -rn "\.unwrap()\|\.expect(" --include=*.rs crates/*/src/ | grep -v "#\[cfg(test)\]" || true

# [CHECK] 9 — runtime smoke
cargo run -p rustcode -- --help          # no `telemetry` subcommand, no --no-telemetry
cargo run -p rustcode -- -p "say ok"     # headless turn against a configured provider
RUSTCODE_WIRE_DUMP=1 cargo run -p rustcode -- -p "say ok"
ls ~/.rustcode/wire-dump                     # proves the new name is live
```

新增 / 扩展的测试：

```text
[+] capabilities/tests/provider_conformance.rs
    - one recorded conversation, two adapters, identical StreamEvent sequence
    - extra_headers injected on both; assert auth headers were NOT clobbered
    - reserved header name -> construction error (fail-closed)
[+] capabilities/tests/nonstream.rs
    - OpenAI-compatible `chat()` with `"stream": false` (real non-stream verb)
    - Anthropic `chat()` via the default fold (no non-stream verb exists)
    - both: text / tool_calls / usage / finish_reason parity with the stream path
[~] capabilities/tests/{anthropic_mock,ollama_mock,http_mock}.rs
    - extend existing mocks with a custom base_url + extra_headers + proxy; do NOT
      duplicate what is already there
[+] config: model_mapping resolution (alias hit, alias miss, empty mapping)
[+] config: a config.toml containing a stale `[telemetry]` section still loads (D4)
[+] coding: provider_factory dispatch for every ProviderKind alias
[+] daemon: ClientMode parse parity with the previous SessionMode tags
```

手工模型连通性检查（需要用户自己的凭据；绝不提交）：

```bash
export RUSTCODE_BASE_URL="https://<your-gateway>/v1"
export RUSTCODE_API_KEY="<your-key>"          # or api_key = "env:YOUR_KEY" in config.toml
cargo run -p rustcode -- --model <model> -p "reply with the single word: ok"
```

---

## 8. 风险、未决问题与 GATEWAY

| ID | 条目 | 类型 |
|----|------|------|
| D1 | 产品名 `rustcode` —— 确认或推翻 | **决策** |
| D2 | 不保留旧 `RUSTCODE_*` / `~/.rustcode` 回退（不兼容） | **决策** |
| D3 | 线上契约键改名且不保留兼容读取（既有会话不可读） | **决策** |
| D4 | 残留的 `[telemetry]` 配置段仍必须能加载 | **决策** |
| D5 | AtomGit 签名器保留但受域名门禁；不移除 | 决策 |
| D6 | 不内置托管 `base_url`；缺少 `base_url` 时 fail-closed | 决策 |
| D7 | `extensions/`（vscode、jetbrains）引用了 `[RustCode]` 日志与二进制名 —— 超出核心范围，后续补丁处理 | 未决 |
| D8 | `crates/rustcode-codingplan-crypto/` 闭源叠加层：自构建二进制无法签名网关请求（上游既有行为） | 未决 |
| D9 | 非流式 `chat()` 在 PHASE-2 只作为接缝交付；迁移 `session_title` / `next_prompt_suggestion` / `compaction` 的调用点是后续任务 | 未决 |
| D10 | 在任何一次发版之前，`.github/workflows/build.yml`、`scripts/release*.sh`、`latest.json` 都需要换成新的产物名 | 未决 |

[GATEWAY] 需确认 **D1**（产品名）与 **D2/D3**（接受不兼容变更）。
本文档其余部分已可执行。确认之后，从 STEP 2.1（[AGENT-A]）开始，
并按 §6.2 确定的顺序推进。
