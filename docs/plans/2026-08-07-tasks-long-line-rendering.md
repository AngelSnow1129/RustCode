# Tasks 长行渲染实施计划

> **致 Claude：** 必需子技能：使用 superpowers:executing-plans 按任务逐条实施本计划。

**目标：** 在让当前任务最多跨三行折行终端行保持可读的同时，使固定的 Tasks 面板仍有界。

**架构：** 保留 `TodoProgress` 与现存的 frontier 窗口选择。先计算当前任务的视觉行数，再选择逻辑行；按额外折行出来的行数扣减逻辑行预算；续行采用悬挂缩进渲染。`/todo` 仍是完整列表视图；runtime、持久化、协议与键盘焦点状态均不变。

**技术栈：** Rust、rustcode-tuix retained terminal renderer、现存的显示宽度折行辅助函数。

---

### 任务 1：补充视觉行回归覆盖

**文件：**
- 修改：`crates/rustcode-tuix/src/render/retained.rs`

1. 新增一个测试：使用一条很长的待处理 frontier 任务与固定宽度的终端。
2. 断言该任务使用续行、不超过 `MAX_TODO_PANEL_ROWS`，并保留折叠的 `+N more` 指示。
3. 新增一个短任务的一致性断言，证明既有的单行布局未变。

### 任务 2：按视觉行分配 Tasks

**文件：**
- 修改：`crates/rustcode-tuix/src/render/retained.rs`

1. 新增一个辅助函数，把一条当前任务折成至多三行对显示宽度安全的文本行。
2. 把当前 frontier 判定为进行中的任务，否则取第一个待处理任务。
3. 在调用 `todo_panel_rows` 之前，从逻辑任务窗口预算中扣除续行。
4. 续行以与任务文本对齐的悬挂缩进渲染。
5. 让底栏行高测量复用同一个宽度感知的构建器，使测量与绘制不会分叉。

### 任务 3：验证 retained 底栏

**文件：**
- 测试：`crates/rustcode-tuix/src/render/retained.rs`

1. 运行聚焦的 todo 渲染器测试。
2. 运行 `cargo test -p rustcode-tuix --lib`。
3. 运行 `git diff --check` 并检查最终 diff，保留既有的 text-caret 改动。
