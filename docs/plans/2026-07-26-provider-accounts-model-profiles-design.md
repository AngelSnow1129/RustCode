# Provider Accounts 与 Model Profiles 设计

**状态：** 已提案  
**日期：** 2026-07-26  
**范围：** RustCode 配置、TUI `/provider` 与 `/model`、daemon provider API、运行时 provider 解析  
**参考实现：** OpenCode 的 provider 注册表与 Models.dev 集成；Codex 的 `model_providers` 注册表与独立的模型选择

## 1. 问题

RustCode 目前为每个条目保存一个扁平化的 `ProviderConfig`：

```toml
default_provider = "MyDeepSeek"

[providers.MyDeepSeek]
type = "openai"
base_url = "https://api.deepseek.com/v1"
api_key = "sk-..."
model = "deepseek-chat"
context_window = 128000
```

这使得同一个 provider 条目同时承担了三种职责：

1. 服务/供应商与线协议；
2. 账号与凭据；
3. 单个模型及其限制。

因此，同一个账号下要使用两个模型时，就必须重复填写 `type`、`base_url`，通常还要重复 `api_key`。当前的 `/provider` 向导也要求用户即便是面对知名服务，也要理解这些底层字段。

期望的体验是：

```text
select vendor → configure/login account → select or enter models → choose default model
```

## 2. 目标与非目标

### 目标

- 为常见供应商提供精选预设，首批覆盖 AtomGit、阿里云百炼、火山方舟、Xiaomi MiMo、DeepSeek、Zhipu、Moonshot、MiniMax、SiliconFlow、OpenRouter、OpenAI、Anthropic 与 Ollama。
- 把稳定的连接默认值，与用户凭据、模型专属限制分离开。
- 允许每个供应商配置多个账号、每个账号配置多个模型。
- 保持自定义 OpenAI 兼容与 Anthropic 兼容端点的一等公民地位。
- 保留全部既有的 `[providers.*]` 配置，无需人工迁移。
- provider/模型解析继续由 `rustcode-config` 独占，并向编码运行时与 driver 传入一份完全解析后的运行时配置。
- 确保 API key 与 OAuth 凭据不出现在脱敏 API 与日志中。

### 非目标

- 首个版本不维护穷尽的全球模型目录。
- 不把 Models.dev 变成必需的运行时依赖。
- 启动时不会自动重写 `config.toml`。
- 不向 `rustcode-kernel` 添加 provider 专属的业务行为。
- 不做基于 OAuth 的 provider adapter，包含 GitHub Copilot。首个版本只覆盖基于 API key 与端点的 provider；Copilot 推迟为独立的后续任务（见 §9）。

## 3. 推荐的领域模型

持久化概念与运行时概念必须区分开。

### 3.1 Provider preset（供应商预设）

一份编译进二进制、只读的稳定供应商行为定义：

```rust
pub struct ProviderPreset {
    pub id: &'static str,
    pub display_name: &'static str,
    pub provider_type: ProviderType,
    pub default_base_url: Option<&'static str>,
    pub auth_kind: AuthKind,
    pub api_key_env: Option<&'static str>,
    pub model_source: ModelSource,
}
```

预设用于改善体验，但不拥有持久化数据。小而精的注册表胜过大而全的清单。预设中任何可能变化的字段，都保持可被账号覆盖。

### 3.2 Provider account（供应商账号）

由用户掌控的连接与凭据身份：

```rust
pub struct ProviderAccountConfig {
    pub provider: String,
    pub display_name: Option<String>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub user_agent: Option<String>,
    pub skip_tls_verify: bool,
    pub enterprise_url: Option<String>,
}
```

`provider` 字段引用一个 preset ID 或自定义协议预设。账号持有默认凭据。多个账号可以引用同一个预设。后续增强可以允许 model profile 覆盖账号凭据，但在首个 UI 中不应暴露，除非确有真实供应商需要。

### 3.3 Model profile（模型档案）

一个可选模型及其模型专属行为：

```rust
pub struct ModelProfileConfig {
    pub account: String,
    pub model: String,
    pub display_name: Option<String>,
    pub context_window: usize,
    pub max_tokens: Option<usize>,
    pub capable_model: Option<i64>,
    pub thinking_type: Option<String>,
    pub thinking_keep: Option<String>,
    pub reasoning_history: Option<String>,
    pub reasoning_effort: Option<String>,
    pub thinking_enabled: Option<bool>,
    pub thinking_budget: Option<u32>,
}
```

模型 ID 是稳定键，推荐形式为 `<account>/<model-or-alias>`。线上的模型名仍是独立取值，以便支持别名与部署名。

### 3.4 解析后的运行时配置

所有消费方都拿到同一份扁平化、不可变的值：

```rust
pub struct ResolvedModelConfig {
    pub selection_id: String,
    pub account_id: String,
    pub provider_id: String,
    pub provider_type: String,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub model: String,
    pub context_window: usize,
    pub max_tokens: Option<usize>,
    // existing thinking, TLS, routing, and prompt fields
}
```

只有 `rustcode-config` 负责解析预设、账号、模型、环境变量与遗留条目。编码运行时、CLI、TUI、daemon、ACP 与 clix 都不得自行实现兼容分支。

## 4. 建议的配置格式

```toml
default_model = "aliyun-default/qwen3-coder-plus"

[provider_accounts.aliyun-default]
provider = "aliyun"
api_key = "$DASHSCOPE_API_KEY"

[models."aliyun-default/qwen3-coder-plus"]
account = "aliyun-default"
model = "qwen3-coder-plus"
context_window = 131072

[models."aliyun-default/qwen3-max"]
account = "aliyun-default"
model = "qwen3-max"
context_window = 131072
```

自定义端点采用同样的结构：

```toml
[provider_accounts.corp]
provider = "openai-compatible"
base_url = "https://llm.example.com/v1"
api_key = "$CORP_LLM_KEY"

[models."corp/code"]
account = "corp"
model = "company-code-model"
context_window = 200000
```

密钥可以继续以环境变量引用的形式存在。未来的凭据存储可以替换字面量持久化，而无需改变账号与模型之间的关系。

## 5. 向后兼容

既有的 `[providers.<name>]` 条目继续有效，并在内存中投影为一个合成账号加一个模型档案。`default_provider` 映射到对应的合成模型选择。

兼容规则：

1. 两种 schema 各自独立解析，只隔离格式错误的条目，与当前宽松的 provider 加载行为一致。
2. 启动或普通读取期间不重写遗留配置。
3. 仅在 ID 完全冲突时新格式 ID 优先，并对冲突输出可见的诊断信息。
4. 遗留条目在 `/provider`、`/model`、CLI 覆盖、daemon API、ACP 与运行时 reload 中保持可选。
5. 编辑遗留条目时提供显式的「升级为多模型配置」操作。
6. 升级通过 `ConfigStore` 的 revision/CAS 原子地写入账号与模型。
7. 至少保留一个完整大版本周期的遗留读取能力；本特性不包含任何移除动作。
8. 保存时保留被隔离的原始 provider 表与未知兼容字段。

首个版本应继续以原始 schema 序列化未被改动的遗留条目，新条目则使用新 schema。这样既避免了高风险的全文件迁移，也让回滚成为可能。

## 6. 预设目录策略

RustCode 应采用混合策略：

- 像 Codex 一样，把一小批稳定的 provider 定义编译进二进制。
- 像 OpenCode 一样，把 provider 元数据与模型元数据分离，并允许自定义扩展。
- 不像 OpenCode，首个版本不要求依赖 Models.dev。

预设条目只包含稳定的连接/认证默认值。模型列表可以来自：

- provider 的模型发现 API；
- 一份内嵌的小型推荐清单；
- 手动录入模型。

未知 provider 一律使用自定义兼容预设。预设可覆盖，确保供应商 URL 变更不会让用户卡在等待 RustCode 发版上。

## 7. `/provider` 交互

`/provider` 成为账号管理中心。它不得占用 `Tab`，因为 TUI 用 Tab 切换 agent 模式。

### 主视图

```text
Provider accounts

● Alibaba Cloud · aliyun-default       3 models
○ DeepSeek · personal                  2 models

  Add provider…
  Custom compatible endpoint…
```

按键：上/下方向键移动，Enter 打开详情，`a` 新增，`e` 编辑，`d` 断开/删除，Esc 关闭。

### 新增流程

1. 选择精选 provider 或自定义兼容端点。
2. 配置账号名与认证方式。
3. 在不落盘部分配置的前提下测试认证/连接。
4. 支持时发现模型；否则展示推荐清单与手动录入。
5. 配置模型限制，高级字段默认折叠。
6. 保存，并可选地把该模型设为默认。

官方预设的 URL 与协议类型默认隐藏，但可在高级设置中覆盖。编辑时 API key 留空表示保留当前密钥。

### 账号详情

详情视图展示脱敏后的连接状态、凭据是否存在、端点来源、已配置模型，以及新增模型、更新凭据、测试、删除等操作。删除仍被模型引用的账号需要显式确认，并说明哪些 profile 会被一并移除。

遗留条目标注为「遗留配置」且保持可用。其详情视图提供原地编辑与显式升级。

## 8. `/model` 交互

`/model` 仍是快速选择入口。它列出的是模型档案，而非扁平化的 provider：

```text
Models

● Alibaba Cloud / Qwen3 Coder Plus
  Alibaba Cloud / Qwen3 Max
  DeepSeek / DeepSeek V4
```

搜索匹配供应商、账号、显示名与线上模型名。Enter 通过既有的运行时 reload 边界修改 `default_model`。重建失败时必须保留先前的运行时与已持久化的默认项，遵循当前 fail-closed 的 reload 语义。

## 9. GitHub Copilot（已推迟，不在本特性范围内）

GitHub Copilot 有意被**排除**在本特性之外。它是一个基于 OAuth 的专用 adapter（GitHub 设备码流程、Copilot token 交换/刷新、动态发现、Enterprise URL、Copilot 专属请求头，以及与既有 Copilot MCP OAuth 的凭据隔离），其新颖度与认证生命周期风险，与本次重构的其余部分不成比例。

这里的领域模型对它前向兼容：`AuthKind` 预留了 OAuth 变体，账号也可以携带凭据引用而非字面量密钥。当 Copilot 作为独立后续任务启动时，它只需作为一个新的预设加 adapter 插入，而无需改变账号/模型 schema。在此之前，首个版本只交付基于 API key 与端点的 provider。

## 10. 解析、生命周期与失败语义

`Config::resolve_model(selection)` 是唯一的解析边界。解析会校验：

- 所选模型存在；
- 所引用的账号存在；
- 预设/自定义协议受支持；
- base URL 与凭据要求已满足；
- 上下文与输出 token 限制合法。

provider/模型 reload 仍由 `CodingRuntime` 拥有。driver 提交解析后的选择；运行时准备一个替代 provider，且仅在构造成功后才切换。认证、发现或客户端构造失败时，不得留下 noop 句柄、空会话或部分持久化的默认项。

配置持久化使用 `ConfigStore` 的 CAS。测试必须覆盖 TUI/WebUI 的并发写入。API key、OAuth token 与授权头必须从诊断信息、遥测、daemon 响应和调试表示中脱敏。

## 11. API 演进

在保留既有 provider 端点的同时，新增带版本的账号/模型资源：

- `GET/POST/PATCH/DELETE /provider-accounts`
- `GET/POST/PATCH/DELETE /models`
- `POST /provider-accounts/:id/test`
- `POST /provider-accounts/:id/discover-models`
- `POST /models/:id/default`

既有 `/providers` 端点继续提供遗留兼容的扁平视图。新客户端使用账号/模型 API。脱敏响应只暴露 `has_api_key` 或认证状态，绝不暴露密钥值。

## 12. 测试与上线

必须覆盖：

- 仅旧格式、仅新格式、混合格式的 TOML 解析；
- 格式错误条目的隔离性；
- 确定性的优先级与诊断信息；
- 启动时不重写；
- 显式的原子化遗留升级；
- 账号删除的引用完整性；
- 模型选择与运行时失败回滚；
- API 脱敏与并发 revision 冲突；
- TUI 导航不截获 Tab；
- provider 发现不可用/离线时的回退。

上线应分阶段推进：先领域模型/解析器，其次兼容性投影，第三 daemon API，第四 TUI 账号/模型 UX，最后是 provider 预设。

## 13. 决策

- Provider 凭据默认属于账号作用域。
- 模型级凭据覆盖推迟到出现被证实的需求时再做。
- 预设是精选且可覆盖的，不追求穷尽。
- Models.dev 是未来可选的目录增强，不是首个版本的依赖。
- 遗留 provider 格式继续受支持，且加载时绝不自动重写。
- `/provider` 管理账号；`/model` 负责快速的模型选择。
- `default_model` 是唯一权威选择；没有任何展示或运行时路径直接读取 `default_provider`（§14.1）。
- `capable_model` 是每个模型档案的排序值；子 agent 分层在模型目录上求解，而非 provider map（§14.2）。
- `evaluator_provider` 与 `vision_preprocessor_provider` 引用模型选择 ID（仍接受遗留 provider 名）（§14.3）。
- v1 中 WebUI 继续停留在遗留的扁平化 `/providers` + `/models` API；新的账号/模型 API 是增量追加（§14.4）。

## 14. 基于代码库的集成约束

以下约束已针对当前实现核验。它们补齐了上述领域模型自身无法解决的若干具体耦合。每一条都必须被遵守，该特性才可靠；编号按风险排序。

### 14.1 单一选择字段、单一解析路径（取代 `default_provider` 双来源）

**依据。** 如今「用哪个模型」有两个分叉的读取方：运行时通过 `active_provider()` 解析，而展示/生命周期路径直接读取 `default_provider` —— 例如 `Config::default_context_window()`（`config/mod.rs`）会查 `self.providers.get(&self.default_provider)`，而页脚、WebUI 的 `is_default` 与 TUI respawn 读取的也是这个原始键。这种分裂已经造成过已发布缺陷（页脚上下文窗口没有跟随模型切换；参见 `rustcode-cli/src/main.rs` 中 `apply_cli_runtime_overrides` 的注释）。

**处理方案。**

1. `default_model` 是**唯一**的权威选择。引入它，同时**不**允许任何展示/运行时路径直接读取 `default_provider`。
2. `Config::resolve_model(None)` 解析当前生效的选择（新的 `default_model`，或按 §5 投影出的遗留 `default_provider`）。每一个需要模型、窗口或 provider 的消费方 —— 页脚上下文窗口、运行时构建、CLI 的 `--provider`/`--model` 覆盖、WebUI 的 `is_default`、会话 respawn —— 都从**同一份** `ResolvedModelConfig` 读取。
3. 把 `default_context_window()` 重新实现为 `resolve_model(None).context_window`（或直接删掉并迁移调用点）。`default_provider` **仅**作为投影的遗留输入保留（§5），绝不被展示或运行时路径直接读取。
4. 增加一个守卫测试：在 `rustcode-config` 的投影/解析代码之外 grep 全仓对 `default_provider` 的直接读取，出现新的即失败；再增加一个行为测试，验证切换 `default_model` 会通过这条单一路径更新页脚窗口。

### 14.2 基于账号 + 模型档案的子 agent 分层解析

**依据。** `resolve_tier_keys()`（`rustcode-coding/src/subagent_tiers.rs`）按 `capable_model` 给扁平 `config.providers` map 中的条目排序；`provider_factory.rs` 的 `resolve_subagent_tier_thunks()` 用这些键构建 fast/capable 分层 provider；daemon 的 create handler 硬编码 `capable_model: None`。把 provider 拆成账号 + 档案会破坏这次扫描。

**处理方案。**

1. `capable_model` 是**每个模型档案**的排序值（已如 §3.3 放在 `ModelProfileConfig` 中），而不是每个账号的。
2. `resolve_tier_keys` 在**解析后的模型目录**上工作（遗留 + 新增，按 §5 统一投影），按 `capable_model` 给模型选择 ID 排序。宿主模型 = 当前的 `default_model`；fast = 排序最低的 capable 档案，capable = 最高。遗留 provider 投影为一个携带其既有 `capable_model` 的档案，因此当前分层行为不变。
3. `provider_factory` 通过 `resolve_model(<selection-id>)` 构建每一层，而不是 `config.providers.get(key)`。
4. 新的模型档案 create/patch API 按档案接受 `capable_model`；遗留的 `/providers` create handler 保持其当前默认行为。

### 14.3 Provider 名引用字段（`evaluator_provider`、`vision_preprocessor_provider`）

**依据。** 两者都是顶层 `Option<String>` 字段，**按名字**引用 provider，并用 `config.providers.contains_key(...)` 校验（`config/mod.rs` 中的 vision 校验；`runtime.rs` 中的目标评估器查找；`rustcode-cli/src/vision.rs`）。§3–§5 的领域模型没有提到它们；一旦 ID 空间变化，它们会静默失效。

**处理方案。**

1. 两个字段的取值改为**模型选择 ID**（与 `default_model` 同一 ID 空间），通过 `resolve_model(id)` 解析。
2. 向后兼容：取值若匹配某个遗留 provider 名，则通过遗留投影解析（§5），因此既有 `config.toml` 无需修改即可继续工作。
3. 校验从 `providers.contains_key` 改为「`resolve_model(id)` 成功」；两处查找点读取解析后的模型。字段名保持不变以避免无谓改动；需在文档中说明取值现在是模型选择 ID（仍接受遗留名称）。

### 14.4 WebUI 与遗留 API 面（v1 范围）

**依据。** daemon 的 `/providers` 与 `/models` handler，以及 WebUI（`webui/src/api.ts` 中的 `ModelInfo` / `ProviderInfo` / `ConfigInfo`），都假定存在一个以 provider 名作为身份标识的扁平 `providers` map。WebUI 不在本计划的文件清单中。

**决策。** v1 **不**迁移 WebUI。daemon 继续提供遗留的扁平化 `/providers` 与 `/models` 视图（由同一份解析后的目录支撑），WebUI 保持其「provider 名即身份」的契约不变。新增的 `/provider-accounts` 与模型档案资源（§11）是**增量**能力，只被 TUI 与未来的客户端消费。后续阶段再把 WebUI 迁移到账号/模型资源。这样既限制了影响半径，也让 WebUI 无需配合前端改动即可发布。

### 14.5 解析值的完备性与增量迁移

1. `ResolvedModelConfig`（§3.4）除已列字段外，还必须携带**每个模型动态的 `base_url`**（CodingPlan 会从服务端选择模型专属端点，而非账号的静态 base URL）与 `system_prompt`。
2. 任务 5 的影响半径是真实的：约 230 处直接的 provider 字段读取分布在约 12 个文件中
   （尤以 `rustcode-tuix/src/event_loop/mod.rs`、`rustcode-coding/src/{runtime,
   config,provider_factory,parts}.rs`, `rustcode-daemon/src/api_provider.rs` 等文件为主，
   `rustcode-codingplan/src/setup.rs` 亦在其列）。必须**增量**迁移，藏在
   `active_provider()` 兼容包装之后（计划任务 4 第 3 步），一次转换一个消费方集群
   并各自跑测试 —— 而不是一次性大爆炸式提交。

### 14.6 修正后的假设：`/provider` 与 `Tab`

关于 `/provider`「不得占用 `Tab`」的担忧（§7 与交付门禁），其实没有文中说的那么强：当模态框处于活动状态时，事件循环会先把**所有**按键路由给 `modal.handle_key()`，之后才轮到全局的 agent 模式开关；而当前的 `provider_wizard` 与 `model_picker` 都没有绑定 `Tab`。遵守「不绑定 `Tab`」仍然没问题，但那只是风格偏好，不是正确性风险 —— 无论如何模态框都不可能把 `Tab` 泄漏给模式切换。
