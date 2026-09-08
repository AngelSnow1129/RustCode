# 模型成本归因实施计划

> **给 Claude：** 必需子技能：使用 superpowers:executing-plans 逐任务实施本计划。

**目标：** 让 `/cost` 把 token 用量与估算成本归因到产出它的 provider/model 上，
并且不在 `/model` 之后重新标注历史用量。

**架构：** `CodingRuntime` 已经在 provider/model 变更时重建其装配好的 agent。
每一代都会用该代稳定的 provider/model 身份与可选的定价快照来构造原生
`SnapshotHook`。该 hook 会累加本轮中的每一次模型响应，并把可累加的模型用量
记录持久化到原生会话元数据中。TUI、daemon 与远端 `/cost` 复用同一套会话
聚合模型；TUI 的实时计数器仍然只作展示之用。

**技术栈：** Rust、serde 兼容的原生会话元数据、rustcode-kernel 生命周期 hook、rustcode-coding 运行时装配、TUI/daemon 命令投影。

---

### Task 1：原生用量 schema 与聚合

**文件：**
- 修改：`crates/rustcode-capabilities/src/session/manager.rs`
- 修改：`crates/rustcode-capabilities/src/session/mod.rs`
- 测试：`crates/rustcode-capabilities/src/session/manager.rs`

**步骤：**
1. 增加失败测试，覆盖 provider/model 分组、同名模型分属不同 provider、
   定价未知，以及遗留的未归因总量。
2. 增加带 serde 默认值的 token、定价快照、每模型用量与报表类型。
3. 按 `(provider_id, model_id)` 实现聚合，同时仅在没有任何明细记录时，
   才把遗留的 `TurnStat.total_tokens` 保留为未归因数据。
4. 运行聚焦的会话测试。

### Task 2：运行时持有的归因

**文件：**
- 修改：`crates/rustcode-capabilities/src/session/snapshot.rs`
- 修改：`crates/rustcode-coding/src/parts.rs`
- 测试：`crates/rustcode-capabilities/src/session/snapshot.rs`

**步骤：**
1. 增加一个失败 hook 测试，在一轮内包含两次模型响应。
2. 给 `SnapshotHook` 一份由 `rustcode-coding` 配置的、不可变的世代归属信息。
3. 累加每次响应的 prompt、completion 与 cached token。
4. 在 `turn_complete` 持久化明细用量；为兼容测试保留未归因的旧构造函数。
5. 验证模型重载会自然地使用新的运行时配置重建该 hook。

### Task 3：provider 定价配置

**文件：**
- 修改：`crates/rustcode-config/src/config/provider.rs`
- 修改：`crates/rustcode-coding/src/config.rs`
- 仅在编译需要之处修改 provider API 投影。
- 测试：`crates/rustcode-config/src/config/provider.rs`

**步骤：**
1. 增加测试，覆盖省略、显式免费与已配置的每百万价格。
2. 增加一个可选的 provider 定价对象；省略表示未知，而非零。
3. 把不可变的定价快照解析进 `CodingRuntimeConfig`。
4. 本次改动不引入远端模型价格服务。

### Task 4：统一的 `/cost` 投影

**文件：**
- 修改：`crates/rustcode-daemon/src/commands.rs`
- 修改：`crates/rustcode-tuix/src/event_loop/commands.rs`
- 修改：`crates/rustcode-tuix/src/session.rs`
- 按需修改 `crates/rustcode-tuix/src/i18n` 下的文件。
- 测试：所涉 crate 中的命令与渲染单元测试。

**步骤：**
1. 增加失败测试：先使用 A，随后切换到未被使用的 B。
2. 把按当前模型重新计价的逻辑替换为原生会话聚合。
3. 渲染按 provider/model 分组的 token 行；仅对已知/免费定价给出估算成本；
   并保留一个遗留的“未归因”分区。
4. 让 TUI、daemon 与远端命令投影使用同一套报表语义。
5. 从 `/cost` 路径中移除未知模型的 `$1/$3` 兜底。

### Task 5：兼容性与验证

**文件：**
- 复查所有被修改的文件与相关 fixture。

**步骤：**
1. 每个逻辑单元完成后运行受影响 crate 的测试。
2. 运行 config、capabilities、coding、daemon 与 tuix 的跨 crate 测试/检查。
3. 验证旧元数据可反序列化且仍为未归因状态。
4. 审计 provider 重载、恢复、undo、压缩、取消与世代边界。
5. 检查最终 diff 是否夹杂无关改动，并记录任何未经真机验证的行为。
