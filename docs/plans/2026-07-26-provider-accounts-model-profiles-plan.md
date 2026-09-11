# Provider Accounts 与 Model Profiles 实施计划

> **给 Claude：** 必需子技能：使用 superpowers:executing-plans，逐任务执行本计划。

**目标：** 在不破坏既有配置的前提下，新增精选 provider 选择、可复用的 provider 账号、每个账号下多个模型档案、遗留 provider 兼容，以及重新设计的 `/provider` 流程。

**架构：** `rustcode-config` 拥有预设、账号、模型档案、遗留投影，以及到单一扁平运行时值的解析。所有 driver 消费该解析值；`CodingRuntime` 保留 provider reload 的所有权。上线过程保留遗留 provider API 与配置，同时引入带版本的账号/模型接口。

**技术栈：** Rust、Serde/TOML、`ConfigStore` CAS、Ratatui/crossterm TUI、Axum daemon API，以及既有的 RustCode 编码运行时与 provider factory。

---

## 基线与约束

- 实施前，记录分支、SHA、worktree 状态，以及每个被修改的运行时/配置文件的最近历史。
- 在隔离的 worktree 中工作，因为当前工作区可能包含无关改动。
- 不修改发布版本号。
- 不向 `rustcode-kernel` 添加 provider/模型生命周期的所有权。
- 本计划中不移除 `ProviderConfig`、`default_provider` 或遗留的 `/providers` API。
- 每个逻辑单元完成后，对受影响的 crate 运行 `cargo test`；在等效的测试编译之后，不要重复运行 `cargo check`。
- 遵守设计文档 §14 的集成约束：唯一的 `default_model` 选择，且不直接读取 `default_provider`（§14.1）；`capable_model` 按模型档案在模型目录上做分层解析（§14.2）；`evaluator_provider`/`vision_preprocessor_provider` 取值改为模型选择 ID（§14.3）；v1 中 WebUI 继续使用遗留的扁平化 API（§14.4）。

### 任务 1：新增 provider preset 领域模型

**文件：**

- 新增：`crates/rustcode-config/src/config/provider_preset.rs`
- 修改：`crates/rustcode-config/src/config/mod.rs`
- 测试：`crates/rustcode-config/src/config/provider_preset.rs`

**步骤：**

1. 为预设 ID 唯一性、必需的稳定字段、按 ID 查找，以及自定义兼容回退编写失败测试。
2. 新增 `ProviderPreset`、`ProviderType`、`AuthKind` 与 `ModelSource`。
3. 新增首批预设：AtomGit、Alibaba、Volcengine、Xiaomi MiMo、DeepSeek、Zhipu、Moonshot、MiniMax、SiliconFlow、OpenRouter、OpenAI、Anthropic、Ollama、OpenAI-compatible 与 Anthropic-compatible。
4. 把模型推荐数据留在本模块之外。
5. 运行：

   ```bash
   cargo test -p rustcode-config provider_preset --offline
   ```

6. 提交：

   ```bash
   git commit -m "feat(config): add provider preset registry"
   ```

### 任务 2：新增账号与模型档案 schema

**文件：**

- 修改：`crates/rustcode-config/src/config/provider.rs`
- 修改：`crates/rustcode-config/src/config/mod.rs`
- 测试：`crates/rustcode-config/src/config/mod.rs`

**步骤：**

1. 为 `provider_accounts`、`models`、`default_model` 编写「仅新格式」的 TOML 往返失败测试。
2. 新增 `ProviderAccountConfig` 与 `ModelProfileConfig`，并以 `#[serde(default)]` 集成进 `Config`。
3. 校验账号/模型 ID、引用关系、上下文窗口、token 限制，以及预设/自定义端点的必备条件。
4. 确保序列化绝不输出仅存在于运行时或临时性的凭据。
5. 运行：

   ```bash
   cargo test -p rustcode-config config::tests --offline
   ```

6. 提交：

   ```bash
   git commit -m "feat(config): add provider accounts and model profiles"
   ```

### 任务 3：实现遗留投影与混合 schema 加载

**文件：**

- 修改：`crates/rustcode-config/src/config/mod.rs`
- 修改：`crates/rustcode-config/src/store.rs`
- 测试：`crates/rustcode-config/src/config/mod.rs`
- 测试：`crates/rustcode-config/tests/config_store.rs`

**步骤：**

1. 为仅旧格式、混合格式、格式错误的遗留条目、格式错误的新账号、格式错误的模型、ID 冲突，以及「加载时不重写」等场景编写失败测试。
2. 引入一个只读的逻辑目录，把每个遗留 `ProviderConfig` 投影为一个合成账号/模型。
3. 保存时保留原始遗留表与被隔离的表。
4. 定义精确的冲突优先级，并输出诊断信息，而不是静默丢弃条目。
5. 新增一个显式的、由 CAS 支撑的 `upgrade_legacy_provider` 变更操作；加载期间不得调用它。
6. 测试 `ConfigStore` 的并发更新与 revision 冲突。
7. 运行：

   ```bash
   cargo test -p rustcode-config --offline
   ```

8. 提交：

   ```bash
   git commit -m "feat(config): project legacy providers into model catalog"
   ```

### 任务 4：建立单一的模型解析边界

**文件：**

- 修改：`crates/rustcode-config/src/config/mod.rs`
- 修改：`crates/rustcode-config/src/config/provider.rs`
- 测试：`crates/rustcode-config/src/config/mod.rs`

**步骤：**

1. 为解析预设默认值、账号覆盖、环境变量中的 API key、遗留条目、缺失引用，以及密钥安全的错误信息编写失败测试。
2. 新增 `ResolvedModelConfig` 与 `Config::resolve_model`。解析值中需包含每个模型的动态 `base_url` 与 `system_prompt`（§14.5）。
3. 尽可能把 `active_provider` 保留为由同一套解析逻辑支撑的兼容包装。
4. 确保解析值包含 provider 构造所需的一切，但不包含任何运行时状态所有者。
5. 让 `default_model` 成为唯一的权威选择：把 `default_context_window()` 重新实现为 `resolve_model(None).context_window`；把 `evaluator_provider`/`vision_preprocessor_provider` 的校验与查找迁移到 `resolve_model(id)`（仍接受遗留名称）。增加守卫测试，拒绝在配置解析之外新增对 `default_provider` 的直接读取（§14.1、§14.3）。
5. 运行：

   ```bash
   cargo test -p rustcode-config resolve_model --offline
   ```

6. 提交：

   ```bash
   git commit -m "feat(config): resolve provider accounts and models"
   ```

### 任务 5：把 provider 构造的消费方迁移到解析后的模型

**文件：**

- 修改：`crates/rustcode-coding/src/config.rs`
- 修改：`crates/rustcode-coding/src/assemble.rs`
- 修改：`crates/rustcode-coding/src/parts.rs`
- 修改：`crates/rustcode-coding/src/provider_factory.rs`
- 修改：`crates/rustcode-coding/src/runtime.rs`
- 按需修改：`crates/rustcode-cli/src/main.rs`
- 按需修改：`crates/rustcode-cli/src/acp/engine.rs`
- 按需修改：`crates/rustcode-daemon/src/live_api.rs`
- 按需修改：`crates/rustcode-daemon/src/native_live.rs`
- 按需修改：`crates/rustcode-daemon/src/commands.rs`

**步骤：**

1. 枚举当前 `ProviderConfig` 的生产消费方，并记录哪些需要连接字段、哪些需要模型字段、哪些需要展示字段。
2. 新增失败测试，证明模型切换会保持会话绑定、工作目录、审批、generation 隔离，以及构造失败时保留先前的运行时。
3. 通过既有的编码运行时 build/prepare/assemble 接缝传递 `ResolvedModelConfig`。把约 12 个文件中约 230 处直接的 provider 字段读取，按消费方集群（runtime/factory、daemon API、TUI、CLI、codingplan）逐集群、逐提交迁移 —— **注：codingplan（rustcode-codingplan）已于 2026-09-09 移除，不再是迁移目标；此列表为历史遗留描述** —— 每个集群各自跑测试 —— 而不是一次性大爆炸式改动（§14.5）。
4. 移除 driver 中重复的 provider/模型查找；不要创建第二个运行时所有者。
5. 改造子 agent 分层：`resolve_tier_keys` 在解析后的模型目录（含遗留投影）上按每个模型档案的 `capable_model` 排序，`provider_factory` 通过 `resolve_model(<selection-id>)` 而不是 `config.providers.get` 构建每一层（§14.2）。增加一个测试，证明遗留配置保持相同的 fast/capable 路由。
6. 在受影响处验证 CLI、TUI、daemon、ACP、headless/后台与子 agent 的路由行为。
6. 运行：

   ```bash
   cargo test -p rustcode-coding --offline
   cargo test -p rustcode --offline
   cargo test -p rustcode-daemon --offline
   ```

7. 提交：

   ```bash
   git commit -m "refactor(runtime): build providers from resolved models"
   ```

### 任务 6：新增带版本的 daemon 账号/模型 API

**文件：**

- 新增：`crates/rustcode-daemon/src/api_provider_account.rs`
- 新增：`crates/rustcode-daemon/src/api_model.rs`
- 修改：`crates/rustcode-daemon/src/lib.rs`
- 修改：`crates/rustcode-daemon/src/api_config.rs`
- 保留：`crates/rustcode-daemon/src/api_provider.rs`
- 更新：`crates/rustcode-daemon/README.md`

**步骤：**

1. 为列表/创建/更新/删除、引用完整性、凭据脱敏、设为默认、连接测试、发现错误与 CAS 冲突编写 handler 测试。
2. 实现 `/provider-accounts` 与 `/models` 资源。
3. 保持遗留 `/providers` 的响应与变更操作可用。
4. 确保删除账号不会静默留下孤立的模型档案。
5. 确保连接测试不会持久化草稿配置。
6. 运行：

   ```bash
   cargo test -p rustcode-daemon api_provider --offline
   cargo test -p rustcode-daemon api_model --offline
   ```

7. 提交：

   ```bash
   git commit -m "feat(daemon): expose provider account and model APIs"
   ```

### 任务 7：围绕账号重新设计 `/provider`

**文件：**

- 修改：`crates/rustcode-tuix/src/modals/provider_wizard.rs`
- 修改：`crates/rustcode-tuix/src/event_loop/commands.rs`
- 修改：`crates/rustcode-tuix/src/event_loop/mod.rs`
- 修改：`crates/rustcode-tuix/src/render/mod.rs`
- 修改：`crates/rustcode-tuix/src/i18n/mod.rs` 或当前的 i18n 消息归属文件

**步骤：**

1. 为主账号列表、预设选择、凭据输入、高级设置、保存前测试、模型选择、遗留标记/升级、删除确认与取消编写状态机测试。
2. 把扁平化的新增/编辑序列替换为「预设 → 账号 → 模型」流程。
3. 保留自定义模板导入，作为高级/自定义兼容路径。
4. 模态框内绝不绑定 Tab；验证模态框外的模式切换行为保持不变。
5. 遮蔽密钥，并让编辑时的空白输入保留既有凭据。
6. 运行：

   ```bash
   cargo test -p rustcode-tuix provider_wizard --offline
   ```

7. 提交：

   ```bash
   git commit -m "feat(tui): manage provider accounts in provider wizard"
   ```

### 任务 8：把 `/model` 改为模型档案

**文件：**

- 修改：`crates/rustcode-tuix/src/modals/model_picker.rs`
- 修改：`crates/rustcode-tuix/src/event_loop/mod.rs`
- 修改：`crates/rustcode-tuix/src/event_loop/commands.rs`
- 测试：`crates/rustcode-tuix/src/modals/model_picker.rs`

**步骤：**

1. 为供应商/账号/模型搜索、遗留投影、当前默认项排序，以及 reload 失败回滚编写失败测试。
2. 列出逻辑上的模型档案，而不是 provider map 条目。
3. 只有在运行时替换成功后才通过 `ConfigStore` 持久化 `default_model`，否则按既有的 reload 契约原子回滚。
4. 更新状态展示，显示供应商/模型而不暴露账号密钥。
5. 运行：

   ```bash
   cargo test -p rustcode-tuix model_picker --offline
   ```

6. 提交：

   ```bash
   git commit -m "feat(tui): select model profiles in model picker"
   ```

### 任务 9：新增模型推荐与发现抽象

**文件：**

- 新增：`crates/rustcode-config/src/config/model_catalog.rs`
- 修改：`crates/rustcode-config/src/config/provider_preset.rs`
- 修改：`crates/rustcode-daemon/src/api_provider_account.rs`
- 测试：相关的 config 与 daemon 模块

**步骤：**

1. 定义 `ModelCatalogSource`，包含内嵌、远程发现与手动三种实现。
2. 新增一份带来源/版本元数据的小型内嵌推荐集。
3. 把发现失败视为可恢复，并始终保留手动录入模型。
4. 缓存远程发现结果，但不让启动依赖网络访问。
5. 保持 Models.dev 集成为可选，且在单独批准前保持禁用。
6. 运行受影响的 config 与 daemon 测试。
7. 提交：

   ```bash
   git commit -m "feat(provider): add optional model discovery"
   ```

> **已推迟：** GitHub Copilot（基于 OAuth 的专用 adapter）有意被排除在本计划之外，参见设计 §9。账号/模型 schema 对它前向兼容，因此将来可以作为一个新的预设加 adapter 加入，而无需返工本特性。

### 任务 10：跨端验收与文档

**文件：**

- 更新：`crates/rustcode-daemon/README.md`
- 更新或新建：`docs/testing/provider-accounts-model-profiles-acceptance.md`
- 更新实施过程中发现的用户可见的配置文档

**步骤：**

1. 为仅遗留、仅新格式、混合、格式错误、自定义兼容与多账号配置补充 fixture。
2. 在实际受影响的范围内验证 CLI、TUI、daemon、headless/后台、ACP、clix、会话恢复、provider reload、取消审批与子 agent 路由。
3. 验证日志、诊断信息、API 负载、快照与遥测中均不出现密钥。
4. 验证回滚到先前的 RustCode 版本后，未被改动的遗留配置仍然可用。
5. 运行受影响的 crate 测试套件与相关的工作区验收命令。
6. 记录已知不受支持的 provider 与手动自定义端点的回退方案。
7. 提交：

   ```bash
   git commit -m "docs(provider): document account and model workflows"
   ```

## 交付门禁

只有满足以下条件，该特性才算就绪：

- 既有遗留配置启动后选择的模型与之前一致；
- 一个账号下至少可以选择两个模型，且无需重复连接配置；
- 格式错误的账号/模型条目不会禁用无关的 provider；
- 模型 reload 失败时保留先前的活跃运行时与会话绑定；
- TUI 与 daemon 的写入是 CAS 安全的；
- 没有任何 API 或 UI 暴露凭据；
- `/provider` 不截获 Tab；
- 遗留接口被明确报告为保留（而非退役）；
- 切换 `default_model` 会通过单一解析路径更新页脚上下文窗口，且没有任何展示/运行时路径直接读取 `default_provider`（§14.1）；
- 分层改造后，遗留配置保持完全相同的 fast/capable 子 agent 路由（§14.2）；
- `evaluator_provider` 与 `vision_preprocessor_provider` 对遗留名称与新的模型选择 ID 都能解析（§14.3）；
- WebUI 在遗留的扁平化 `/providers` + `/models` API 上继续照常工作（§14.4）。
