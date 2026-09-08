# 忙碌会话继续时分叉的实施计划

> **给 Claude：** 必需子技能：使用 superpowers:executing-plans 逐任务实施本计划。

**目标：** 让第二个交互式 `rustcode -c` 从最近一次已提交的上下文启动，
同时不共享也不损坏已被另一个运行时持有的会话。

**架构：** 保持 `SessionLease` 独占且 fail-closed。在 `SessionManager` 上增加一个
原生聚合分叉操作；仅当交互式 `-c` 收到 `SessionInUse` 时 CLI 才调用它，
以分叉后的新 ID 启动运行时，回放分叉得到的展示状态，并把一条可见的启动
提示传入 TUI。无头继续与一切非争用类错误保持原样。

**技术栈：** Rust、原生 `SessionManager` 聚合持久化、`CodingRuntime`、Clap CLI、RustCode TUI/i18n。

---

### Task 1：增加原生会话聚合分叉

**文件：**
- 修改：`crates/rustcode-capabilities/src/session/manager.rs`

1. 增加一个失败测试：持有源租约，把其完整的原生聚合分叉到调用方提供的
   UUID，并验证源与目标拥有彼此独立的 ID 与产物。
2. 增加测试：源缺失/损坏时失败且不发布目标；已存在的目标仍受保护。
3. 实现 `SessionManager::fork_native_session`：加载一份严格的原生聚合，
   创建全新的目标元数据，获取目标租约，并原子提交快照、展示状态与元数据。
4. 保留消息、展示状态、轮次统计、工作目录与用户可见标题，同时重置目标的
   身份、时间戳与遗留导入来源标记。
5. 运行聚焦的 session-manager 测试。

### Task 2：从交互式继续回退到分叉

**文件：**
- 修改：`crates/rustcode-cli/src/main.rs`

1. 增加一个聚焦的辅助测试，证明只有 `SessionStoreError::SessionInUse` 加上
   交互模式才会选中分叉路径。
2. 修改 CLI 运行时准备逻辑，返回实际继续的会话 ID 与可选的源 ID。
3. 正常获取租约时，完全照旧恢复所请求的 ID。
4. 仅在交互式争用时生成新的 UUID，分叉源聚合，并把目标租约移交给
   `CodingRuntime`。
5. 无头 `-c`、数据损坏、产物缺失、权限失败与目标提交失败都保持显式报错。
6. 从实际的目标 ID 加载回放/遥测状态。

### Task 3：在 TUI 中展示分叉决策

**文件：**
- 修改：`crates/rustcode-config/src/i18n/messages.rs`
- 修改：`crates/rustcode-config/src/i18n/en.rs`
- 修改：`crates/rustcode-config/src/i18n/zh_cn.rs`
- 修改：`crates/rustcode-tuix/src/lib.rs`
- 修改：`crates/rustcode-tuix/src/event_loop/mod.rs`

1. 增加英文与中文文案，说明最近的会话正在别处活跃，并且已从它最后一次
   已提交的状态创建了一个独立分叉。
2. 把一条可选的启动提示传入 TUI 上下文。
3. 在回放分叉会话之前渲染一次该提示。
4. 若既有事件循环测试台能暴露启动输出，则增加一个聚焦的渲染/状态测试；
   否则在编译期覆盖消息格式化与参数透传。

### Task 4：验证与审计

**文件：**
- 只验证上述文件，并保留所有既有的脏 persona/site 文件。

1. 运行聚焦的 capabilities、CLI 与 TUI 测试。
2. 运行 `cargo test -p rustcode-capabilities --features session`。
3. 运行 `cargo test -p rustcode`。
4. 运行 `cargo test -p rustcode-tuix`。
5. 运行 `git diff --check`。
6. 审计会话所有者、租约转移、源/目标 ID、回放绑定、遥测绑定、
   错误传播、无头行为与脏工作区隔离。
