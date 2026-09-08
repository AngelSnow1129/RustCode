# Kernel/Capabilities/Coding → cli/tuix 对等性 Backlog

> 状态：历史 backlog，已由
> [`coding-runtime-native-migration-design.md`](coding-runtime-native-migration-design.md)
> 的实施结果取代。下列未勾选项记录的是接线前的设计假设，不代表当前待办；当前
> CLI、TUI、daemon、background、ACP 和 clix 已切到 `CodingRuntime`，core driver 协议与
> `rustcode-bridge` 已退役。仍未完成的是北极星架构中的非引擎 core/foundation/protocol
> 拆分，不应从本清单推导迁移状态。

**历史目标：** 在把新的倒置栈（L0 `rustcode-kernel` + L1 `rustcode-capabilities` + L2 `rustcode-coding`）接入 cli/tuix 之前，先让它达到生产 `cli/tuix`所需要的**完整功能对等**，从而最终集成时是对等的，而不是降级的。

**硬性规则（每一条目均适用）：**
- **零 `rustcode-core` 改动** —— `cargo tree -p <crate>` 显示 0 个 rustcode-core；每次提交后 `git status --short -- crates/rustcode-core` 为空。
- 所有 kernel 新增都是**追加式的**（`#[serde(default)]`，不对无法批量修复的构造点做破坏性的变体/字段改动）。
- **缓存红线**：纯文本会话序列化后必须逐字节一致。Memory / context 注入发生在 `session_start`/`turn_start`（永久、位于缓存之前）或以**尾部** `pre_request` 追加的形式 —— 绝不允许前缀改写（`pre_request` 守卫现在会对此 Warn）。
- 新增的 provider / tool / middleware / hook 都用**一致性套件**验证（`rustcode_kernel::conformance::{provider,tool,middleware,hooks}::check`）。

图例：工作量 `[T]`rivial `[S]`mall `[M]`edium `[L]`arge · 状态 [ ] 待办 / ◐ 进行中 / [x] 已完成。

---

## 阶段 A —— Kernel L0 追加式原语（打通全局；改动小、零 core）

这几个原语可以在无需为每个特性改动 kernel 的前提下，解锁 Tier-3 driver 特性的绝大部分。

- [ ] **A1 [M] `AgentCommand::SetConversation(SessionSnapshot)`** —— 运行时历史替换（重新播种 convo、重新校验 tool-call/result 配对、递增 `cache_epoch`）。**解锁**：`ClearConversation`（替换为空）、`SetMessages`/`/resume` 的飞行中替换（替换为已加载内容）、`/undo`/`UndoToPrompt`（由 driver 计算截断后的快照并发送）、带连续性的模型切换（snapshot → 新 agent → SetConversation）。验收标准：可在会话中途替换；下一回合发送新历史；配对有效；cache_epoch 递增。
- [ ] **A2 [S] `AgentCommand::ChangeDir(PathBuf)`** —— 修改 `ToolContext` 在后续工具调用中上报的工作目录（目前仅在构建期确定）。**关闭**：`/cd`、`WorkingDirChanged`。验收标准：命令执行后 `WorkingDirProbeTool` 能看到新目录；snapshot/UI 同步反映。
- [ ] **A3 [S] 回合前输入注入** —— `AgentCommand::AppendInput(String)` 将一条用户消息排队，合并进下一回合（在 LLM 调用之前）；此外为 `LocalShell` 提供一条“注入消息但不触发 LLM 回合”的路径（类似 `SendMessage` 但带 `respond:false` 标志，或作为 driver 侧职责 —— 设计阶段再定）。验收标准：排队的文本在下次 LLM 调用之前落地；无 LLM 路径只追加一条消息并结束，不发起 provider 调用。
- [ ] **A4 [S] `SessionSnapshot.turn_stats: Vec<TurnStat>`**（追加式）—— 每回合记录 `{after_message, turn_count, tool_call_count, duration_ms, total_tokens, errored}`，使 `/resume` 能重新渲染 `✓ … tokens` 分隔线。验收标准：snapshot 能往返保存 turn_stats；旧 snapshot 可加载（`serde(default)` → 空）。
- [ ] **A5 [M] 上下文预算内省** —— 发出 `AgentEvent::ContextStats{system_tokens, sent_tokens, dropped_tokens, working_set_tokens, total_messages, tool_defs_tokens, ctx_window, ...}`（通过消息组装之后 / 在 `on_request` 中触发的 builder-opt hook）。**关闭**：`/context`、`RefreshContextStats`。验收标准：driver 无需猜测即可渲染预算明细。
- [ ] **A6 [S]（待定）终态事件持久化** —— 要么为 `TurnComplete`/`Error`/`Cancelled` 增加 `#[serde(default)] messages`，要么明确约定“driver 在终态时发送 `Snapshot`”。倾向后者（不重复）；应作为 driver-adapter 任务（D 档），而非 kernel 改动。验收标准：driver 能在每条终态路径上持久化。

> A1–A5 是本 backlog 中仅有的 kernel 源码改动。每一项都附带一致性风格测试与零 core 验证。

---

## 阶段 B —— L1 `rustcode-capabilities`（主体部分；新增 capability 模块）

- [ ] **B1 [L] MCP capability**（`rustcode-capabilities::mcp`，预留的 L1）—— McpRegistry + server 生命周期（spawn/reload/login/logout）+ 动态工具发现，并以 kernel `Tool` 的形式暴露。动态挂载的张力：kernel 在构建期挂载 → 要么提供一个 agent 可重复读取的**共享可变工具注册表**，要么在 MCP 变化时**重建** agent（先采用简洁的重建方案；后续再评估）。每个被发现的工具都用 `conformance::tool::check` 验证。验收标准：某个 MCP server 的工具能在回合中被调用；`/mcp` 生命周期可用。
- [ ] **B2 [M] Provider 工厂 + 运行时切换** —— 在 L1 provider（OpenAiCompat / Anthropic / Ollama，现已具备）之上提供 `create_provider(config)`，外加**重建**模式：在 `/model`·`/provider` 时构建新 provider → 新 `Agent` → `SetConversation(snapshot)` 以保持连续性。无需改动 kernel（工厂模式，类似 core 的 `AgentRuntimeFactory`）。每个 provider 都用 `conformance::provider::check` 验证。验收标准：会话中途切换模型，对话得以保留，且携带的历史不会破坏 prefix-cache。
- [ ] **B3 [L] 会话持久层**（`rustcode-capabilities::session` 或某个 driver crate）—— 在 `SessionSnapshot` 与元数据（`name`、`working_dir`、`created_at`/`updated_at`、`user_renamed`、`turn_stats`）之上构建 `SessionManager`，并提供 `save/load/list/delete/rename`，存放于 `$RUSTCODE_HOME/sessions/<project_hash>/`。Resume 通过 A1 的 `SetConversation`（飞行中）或 `AgentBuilder::resume`（构建期）重新播种。验收标准：list/load/save/rename/delete 可用；`/resume` 选择器有内容；分隔线能重新渲染（依赖 A4）。
- [ ] **B4 [M] Memory 存储 + 注入 hook**（`rustcode-capabilities::memory`）—— 读写 `memory.md`（全局 + 项目级），并提供一个 `LifecycleHooks`，在 **`session_start`/`turn_start`** 把 memory 注入 system/persona（永久、位于缓存之前 —— 不是 `pre_request`，以遵守缓存守卫）。**关闭**：`/remember`·`/forget`·`/memory`、`Remember/Forget/ShowMemory`。验收标准：`/remember` 能持久化；下一会话的 system 携带该内容；各回合之间 cache prefix 逐字节稳定。
- [ ] **B5 [M] Plan 模式 middleware** —— 一个 `ToolMiddleware`，当共享的 `plan_mode` 标志置位时（由命令或 `Respond` 切换）阻断写入/高风险工具。**关闭**：`SetPlanMode`、`/plan`。用 `conformance::middleware::check` 验证。验收标准：在 plan 模式下，`write_file`/`edit_file`/高风险 `bash` 被阻断并返回清晰的 ToolResult；切换开关可翻转该状态。
- [ ] **B6 [M] 文件历史 / 编辑撤销 + git checkpoint**（capabilities 工具层）—— 记录文件编辑，使文件级 `/undo` 可以回滚，并提供可选的每回合 git checkpoint。区别于会话级 `/undo`（A1）。验收标准：一次工具编辑可以回滚到其先前的字节内容。
- [ ] **B7 [S] 工具输出实时流式化** —— bash/长时间运行的工具通过 kernel 的 `ProgressSink`（→ `AgentEvent::ToolProgress`）实时发出真实的 stdout 分块。Adapter 将其渲染为 `ToolOutputChunk`。验收标准：`bash` 输出在执行过程中实时流出，而不只在结果处出现。
- [ ] **B8 [S] Skills 接线** —— 把 `use_skill`/`list_skills`（已在 `rustcode-capabilities::skills` 中）挂载进 coding 装配流程；向斜杠命令面板暴露该注册表。验收标准：`/use_skill` 可用；面板列出 `user_invocable()` 的 skills。

---

## 阶段 C —— L2 `rustcode-coding`（装配已完成的 capabilities）

- [ ] **C1 [M] “完整” coding-agent 装配** —— 扩展 `build_coding_agent`（或新增 `build_full_coding_agent`）以接线：memory hook（B4）、plan 模式 middleware（B5）、MCP 工具（B1）、文件历史（B6）、skills（B8）、会话持久化句柄（B3）、provider 工厂（B2）。顺序是有承载关系的（批准 middleware 优先）。验收标准：一个装配完成的 agent 暴露全部对等能力；装配测试断言每一项都已挂载/接线。
- [ ] **C2 [L] 后台 + 并行子 agent 组合** —— `/bg`（隔离子会话）与 `parallel_edit` 作为基于 kernel `Agent` 的 **L2 组合**（kernel 已证明“以组合实现子 agent”以及用 `ToolProgress` 传递嵌套进度）。一个小型 pool + 每任务回合预算。已废弃的 core `Background/BackgroundComplete` 协议已退役；剩余的后台工作是把 `/bg` 的运行时 spawner 从 bridge `AgentClient` 切换为 kernel 原生运行时 facade。**关闭**：`SubAgentDispatch*`、`parallel_edit_files`，以及 `/bg` 对 bridge 运行时的依赖。验收标准：后台任务隔离运行并上报完成；并行编辑能分派并按任务上报。
- [ ] **C3 [S] 视觉/图片预处理 middleware** —— 在 `SendMessage` 到达 kernel 之前预处理粘贴的图片（可选的 VL 描述）（kernel 只转发 `ImageContent`）。**关闭**：`VisionPreprocessSuccess`/`RestorePendingImages`（作为 L2/driver 职责）。验收标准：一次图片发送可往返；预处理失败会暴露重新附带的路径。

---

## 阶段 D —— Driver adapter 前置项（tuix 侧胶水；A–C 落地后再构建）

- [ ] **D1 [M] 双向事件/命令转换器** —— kernel↔tuix 的 `AgentEvent`/`AgentCommand` 映射，包括 **PhaseChange 合成**（基于 TurnStarted/TextDelta/ToolStarted/Request 的状态机）、**ToolBatch 分组**（同一条 assistant 消息 ⇒ 一个批次）、**批准 id 关联**（跟踪待处理的 `Request.id` → `Respond{id}`）、**耗时跟踪**（ToolStarted→ToolResult 的间隔）、TurnComplete 元数据聚合。
- [ ] **D2 [L] 斜杠命令分发器** —— 把 tuix 的 `/cmd` 路由到 kernel 命令 / `Respond` / capability 调用（30 多个命令）。复用已有的 modal（DirPicker/ModelPicker/SessionPicker），但指向新栈。
- [ ] **D3 [M] 消息形状桥接** —— core 的 `Message::MessageContent` 枚举 ↔ kernel 扁平的 `Message`（含 `ToolResultRef` 磁盘背衬摘要：桥接或有意丢弃）。仅在与 OLD core 会话互操作时才需要；新的 kernel 会话自身是自洽的。

---

## 阶段 E —— 集成（在 A–D 之后；真正的接线与绞杀）

- [ ] **E1 [M] 标志位后的并行 driver 路径** —— 一个 tuix 入口，通过 adapter（D1）启动完整的 coding agent（C1）并渲染一次真实回合。A–C 的全部内容都已接线，没有任何降级。
- [ ] **E2 [M] 对等性验证** —— 在新路径上跑通现有 tuix 流程（回合、批准、工具、压缩、会话、/undo、/model、/cd、plan、skills、MCP、bg）；补齐差距。
- [ ] **E3 [L] 绞杀** —— 让新路径成为默认；待每一项达到对等后，逐特性移除 core agent 路径。

---

## 建议顺序与关键路径

1. **A1 → A2 → A4 → A3 → A5**（kernel 原语；这几项可以由我来做 —— 改动小、零 core、有一致性测试）。
2. **B2（provider 工厂）+ B3（会话持久化）** —— 任何真实的多回合对等都依赖它们。
3. **B4（memory）+ B5（plan）+ B7（工具输出）+ B8（skills）+ B6（文件历史）** —— 可并行推进。
4. **B1（MCP）** —— 最大的 L1；可以滞后。
5. **C1（完整装配）**，随后是 **C2（后台/并行）**、**C3（视觉）**。
6. **D1 → D2 → D3**（adapter），然后是 **E**。

**E 之前的对等门禁：** A1–A5 [x]、B2/B3/B4/B5/B7/B8 [x]、C1 [x]（若明确接受，B1/B6/C2/C3 可以作为已知降级项滞后）。
