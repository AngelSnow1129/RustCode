# 子任务底栏面板实施计划

> **致 Claude：** 必需子技能：使用 executing-plans 按任务逐条实施本计划。

**目标：** 把并发的 `task` 进度渲染为一个固定、原地更新的底栏面板，而不再输出常驻的转写内容或快速变化的加载指示器标签。

**架构：** TUI 仍然是呈现投影的唯一所有者。它从 `ToolCallStarted.arguments` 播种结构化的子任务列表，按稳定的 `explore#N` / `worker#N` 标签把已有的任务进度消息折叠进该列表，通过 `StatusLine` 暴露该投影，并在匹配的工具终态清除它。内核与 coding-runtime 协议保持不变。

**技术栈：** Rust、rustcode-tuix retained renderer、现存的 `AgentEvent::ToolOutputChunk` 与底栏布局。

---

### 任务 1：建模并解析子任务进度

**文件：**
- 修改：`crates/rustcode-tuix/src/render/mod.rs`
- 修改：`crates/rustcode-tuix/src/state.rs`
- 修改：`crates/rustcode-tuix/src/event_loop/mod.rs`

1. 为播种任务描述以及折叠 start/activity/completion 更新补充失败测试。
2. 新增 TUI 自有的 `SubtaskProgress` 视图，以 call id 与子标签为键。
3. 通用工具进度保持不变；只消费归属 `task`/`code_review` 的进度。
4. 在成功、失败、取消、会话重置或 runtime 替换时清除对应的投影。

### 任务 2：渲染固定底栏面板

**文件：**
- 修改：`crates/rustcode-tuix/src/render/retained.rs`

1. 为表头、running/completed/failed 条目、截断与行数上限补充失败的行布局测试。
2. 复用现存的 top-panel/footer-height 机制，把面板放到输入框上方。
3. 每个子任务只占一行；公共 model 在表头显示一次，仅当混合使用时才逐行显示 model。
4. 面板让位于审批/用户输入/轮次上限面板，并在子任务活跃期间折叠 TodoWrite。

### 任务 3：移除瞬时转写噪声

**文件：**
- 修改：`crates/rustcode-tuix/src/event_loop/mod.rs`
- 修改：`crates/rustcode-tuix/src/render/retained.rs`

1. 停止把 `dispatching`、子任务启动、子任务活动与子任务完成消息渲染为 `CommandOutput`。
2. 阻止子代理活动顶替常规的加载指示器标签。
3. 在匹配的任务终态移除底栏投影，仅保留现存的紧凑已提交 `Task(...)` 工具行。

### 任务 4：验证生命周期与渲染不变量

**文件：**
- 测试：`crates/rustcode-tuix/src/event_loop/mod.rs`
- 测试：`crates/rustcode-tuix/src/render/retained.rs`
- 测试：`crates/rustcode-tuix/src/state.rs`

1. 运行聚焦的 event-loop 与底栏测试。
2. 运行 `cargo test -p rustcode-tuix`。
3. 运行 `git diff --check` 并检查最终 diff，不改动无关的工作区改动。
