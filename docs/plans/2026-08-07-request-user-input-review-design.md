# RequestUserInput 复核页实施计划

> **致 Claude：** 必需子技能：使用 `executing-plans` 按任务逐条实施本计划。

**目标：** 用可读的框式输入框取代通栏反显文本字段，并让最后一个批次停靠点在提交前复核每一个答案。

**架构：** `UserInputBatch` 仍是进行中答案的唯一所有者。把不可变的答案摘要投影进面向渲染器的批次元数据；渲染器负责复核页的换行与滚动，不改变工具的线上响应。既有的部分提交语义保持不变：未回答的问题显式展示，并序列化为 declined 响应。

**技术栈：** Rust、RustCode TUI retained renderer、crossterm 按键事件、既有的虚拟终端测试。

---

### 任务 1：建模复核摘要与滚动

**文件：**
- 修改：`crates/rustcode-tuix/src/state.rs`
- 修改：`crates/rustcode-tuix/src/render/mod.rs`
- 修改：`crates/rustcode-tuix/src/event_loop/mod.rs`

1. 新增与渲染器无关的摘要记录，包含表头、问题与可选的格式化答案。
2. 新增由批次自有的提交页滚动状态。
3. 通过 `UserInputBatchMeta`/`UserInputPanelView` 投影摘要与提交滚动。
4. 当批次光标停在提交停靠点时处理 PageUp/PageDown；保留 Tab/Shift+Tab 与 Enter 行为。
5. 为 selected、custom、text 与未回答摘要补充状态/按键测试。

### 任务 2：渲染真正的文本输入框

**文件：**
- 修改：`crates/rustcode-tuix/src/render/retained.rs`

1. 新增一个失败的渲染器测试，拒绝文本输入模式下的反显单元格。
2. 渲染一个随主题自适应的三行方框，带提示符标记、答案文本与插入光标。
3. 在字段宽度内换行或截断，且不输出控制字符。
4. 更新行数断言，并验证浅色/深色兼容样式。

### 任务 3：渲染最终复核页

**文件：**
- 修改：`crates/rustcode-tuix/src/render/retained.rs`

1. 新增一个失败测试，要求提交停靠点上列出全部问题与答案。
2. 渲染已回答的值与显式的 `未回答` 标记。
3. 让确认动作保持为活动行，使小尺寸终端仍保留可操作的视口。
4. 对高于终端的摘要支持 PageUp/PageDown 指示。
5. 运行聚焦测试，然后执行 `cargo test -p rustcode-tuix --lib` 与 `git diff --check`。
