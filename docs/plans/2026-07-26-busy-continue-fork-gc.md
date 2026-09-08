# Busy Continue Fork GC 实施计划

> **致 Claude：** 必需子技能：使用 superpowers:executing-plans 按任务逐条实施本计划。

**目标：** 持久化自动 busy-continue fork 的谱系，清理被遗弃的零轮次 fork，并把保留下来的 fork 分支折叠为一条逻辑会话行。

**架构：** `SessionMeta` 新增一个可追加、可选的 `ForkInfo` 字段，独立于旧有的 `ImportInfo`。在发布新的 busy-continue fork 之前，`SessionManager` 只有在能够租借（lease）某个更旧的自动 fork、并能证明其创建后未新增任何轮次或转写时，才删除它。带内容的 fork 依旧持久，因为当前的 fork 操作不会克隆父级的 `.jsonl`；目录呈现按根谱系折叠这些保留下来的聚合，同时精确定位 ID 的加载能力保持可用。逻辑删除会保留 `.lease` 与 `.meta.lock`。

**技术栈：** Rust、兼容 serde 的原生会话元数据、`SessionManager`、fs2 lease、原生会话目录。

---

### 任务 1：新增持久化的 fork 谱系

- 新增 `ForkInfo { root_id, parent_id, forked_at_ms, base_message_count, base_turn_count }` 结构。
- 追加式新增 `#[serde(default)] SessionMeta::fork_info`。
- 校验 ID 与时间戳，且不改动 `META_VERSION`。
- 保持与在该字段出现之前写入的元数据的兼容性。

### 任务 2：只回收已被证明遭遗弃的 fork

- 只扫描当前的 project bucket。
- 获取并重新校验候选者的 lease 与元数据。
- 仅当消息/轮次计数与时间戳仍等于 fork 基线、且 JSONL 缺失或为空时才删除。
- 保留活跃的、已有进展的、损坏的、无谱系的以及持有转写的会话。

### 任务 3：在目录呈现中折叠保留下来的 fork

- 在原生 `CatalogEntry` 中携带可选的根谱系。
- 每个 `(project_bucket, root_id)` 只保留最近更新的条目。
- 折叠只作用于 list/latest 呈现面。
- 原始目录与按精确 ID 加载的行为保持不变。

### 任务 4：验证

- 运行聚焦的会话谱系、GC 与目录测试。
- 运行 `cargo test -p rustcode-capabilities --features session`。
- 运行下游 CLI/daemon 的编译与测试。
- 审计确认没有任何清理动作会移除 `.lease` 或 `.meta.lock`。

