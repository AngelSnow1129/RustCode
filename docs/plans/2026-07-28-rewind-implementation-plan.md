# Rewind 实施计划

> **v5.0.5 状态覆盖：** 下列任务描述的是最初的实现。在发现每会话 Git 对象库
> 可能耗尽系统磁盘之后，v5.0.5 中已禁用工作区/代码 Rewind。当前线上行为是
> 只记录会话快照点，工作区树可选或缺失；会话 Rewind 仍可用，而纯代码或合并
> 范围的回退会被明确拒绝并给出原因。要重新启用代码恢复，需要另立计划，覆盖
> 项目级共享对象库、源对象复用、配额、空闲空间检查、可中断超时、GC
> 与孤儿清理。

> **给 Claude：** 必需子技能：使用 superpowers:executing-plans 逐任务实施本计划。

**原始目标（在 v5.0.5 已被取代）：** 增加一个安全的、Claude 风格的 Rewind 选择器，
支持仅会话、仅代码与合并三种恢复范围。

**原始架构（工作区部分在 v5.0.5 已禁用）：** 扩展既有的原生会话 `SnapshotHook`，
增加一个使用独立 Git 目录的工作区检查点服务，持久化紧凑的每轮回退元数据，
并暴露一个由运行时持有的 rewind 操作。TUI 只是纯粹的选择器/投影，
默认选中 `(current)`。

**技术栈：** Rust、Tokio、serde/serde_json、Git plumbing 命令、crossterm、RustCode 原生会话/运行时 API。

---

### Task 1：工作区检查点服务

**文件：**
- 新建：`crates/rustcode-capabilities/src/session/rewind.rs`
- 修改：`crates/rustcode-capabilities/src/session/mod.rs`
- 测试：`rewind.rs` 内联测试

**步骤：**

1. 先写失败测试，覆盖 Git worktree 检测、捕获、变更文件摘要、
   未跟踪文件捕获、忽略文件排除、恢复与冲突检测。
2. 运行 `cargo test -p rustcode-capabilities --features session session::rewind`。
3. 用显式的 `--git-dir` 与 `--work-tree` 实现独立 Git 目录存储。
4. 让恢复以文件为作用域，并在改动前先捕获一份恢复树。
5. 重新运行这些聚焦测试。

### Task 2：持久化每轮回退元数据

**文件：**
- 修改：`crates/rustcode-capabilities/src/session/manager.rs`
- 修改：`crates/rustcode-capabilities/src/session/snapshot.rs`
- 修改：`crates/rustcode-capabilities/src/session/mod.rs`
- 测试：manager 与 snapshot 内联测试

**步骤：**

1. 先写失败测试，覆盖 `<id>.rewind.json`、有界反序列化、删除，
   以及轮次开始/轮次完成快照点的创建。
2. 增加带版本号的 `RewindLedger` 与 `RewindPoint` 类型。
3. 在 `turn_start` 捕获前态树，在 `turn_complete` 捕获后态树/diff。
4. 普通轮次的检查点失败保持尽力而为，但为 UI 记录明确的不可用原因。
5. 运行会话相关测试。

### Task 3：由运行时持有的 rewind 操作

**文件：**
- 修改：`crates/rustcode-coding/src/runtime.rs`
- 修改：`crates/rustcode-coding/src/parts.rs`
- 测试：runtime 内联测试

**步骤：**

1. 先写失败测试，覆盖目标列举与三种回退范围。
2. 增加中立的 `RewindScope`、`RewindTarget`、`RewindResult` 与带类型的错误。
3. 增加 `CodingRuntimeHandle::rewind_points()` 与 `rewind(...)`。
4. 复用既有的 undo 计算与原生聚合提交。
5. 当合并的会话持久化失败时，补充工作区补偿逻辑。
6. 拒绝忙碌、世代过期、修订过期、文件冲突与检查点不可用的请求。
7. 运行 `cargo test -p rustcode-coding --lib`。

### Task 4：Rewind 弹窗与 Esc 路由

**文件：**
- 新建：`crates/rustcode-tuix/src/modals/rewind.rs`
- 修改：`crates/rustcode-tuix/src/modals/mod.rs`
- 修改：`crates/rustcode-tuix/src/event_loop/mod.rs`
- 修改：`crates/rustcode-tuix/src/state.rs`
- 修改：`crates/rustcode-config/src/i18n/messages.rs`
- 修改：`crates/rustcode-config/src/i18n/en.rs`
- 修改：`crates/rustcode-config/src/i18n/zh_cn.rs`
- 测试：弹窗与事件循环内联测试

**步骤：**

1. 先写失败测试，证明初始选中 `(current)` 且 Enter 为空操作。
2. 增加提示列表与范围选择的弹窗状态。
3. 把直接双击 Esc 触发的 `dispatch_undo` 替换为异步加载目标并安装弹窗。
4. 在流式取消时清除/抑制 rewind 的待触发状态。
5. 把提交的 rewind 交给运行时处理，并在其终态之后才重绘。
6. 运行 `cargo test -p rustcode-tuix --lib`。

### Task 5：跨层审计与验证

**文件：**
- 更新：若实现与设计不符，更新 `docs/plans/2026-07-28-rewind-design.md`

**步骤：**

1. 运行 `cargo fmt --all -- --check`。
2. 运行 `cargo test -p rustcode-capabilities --features session`。
3. 运行 `cargo test -p rustcode-coding --lib`。
4. 运行 `cargo test -p rustcode-tuix --lib`。
5. 仅当依赖/特性变更超出前述测试已编译的范围时，才运行 `cargo check --workspace`。
6. 审计 CLI、daemon、后台任务、ACP 与 clix 的影响；确认未改动的驱动
   保持既有的显式 `/undo` 行为。
7. 产出一份人工检查清单，覆盖 Git/非 Git 项目、被取消的轮次、
   冲突、恢复与合并恢复。
