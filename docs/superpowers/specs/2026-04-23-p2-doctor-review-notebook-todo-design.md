# P2: /doctor + /review + Notebook + TodoWrite 设计

日期：2026-04-23

## 1. /doctor 诊断命令

斜杠命令。不涉及 AgentLoop —— 纯 TUI 侧诊断。

检查项：
- Provider：发一个最小 API 调用（列举模型或 1-token 补全），报告延迟或错误
- MCP：从 `mcp::get_statuses()` 读取
- Settings：加载并报告 allow/deny/hook 计数
- Git：执行 `git status --porcelain` + `git branch --show-current`
- Project：检查 `.rustcode.md` 是否存在
- Tools：从 ToolRegistry 统计已注册工具数（经 LoopCtx 或静态传入）

输出格式：每个组件用 `✓`/`✗` 标记，并附详情。

涉及文件：`commands.rs`（斜杠处理器）、`commands.rs`（注册到帮助）

## 2. /review 代码评审

读取 `git diff` 并把它作为评审 prompt 发给 agent 的斜杠命令。

实现：
1. 执行 `git diff`（参数为 `--staged` 时执行 `git diff --staged`）
2. diff 为空则显示 "No changes to review"
3. 构造 prompt：`"Review the following code changes for bugs, security issues, and improvements:\n\n```diff\n{diff}\n```"`
4. 以 `AgentCommand::SendMessage(prompt)` 发出
5. Agent 以普通文本响应流式输出评审内容

涉及文件：`commands.rs`（斜杠处理器 + 注册）

## 3. Notebook（.ipynb）支持

为 `read_file` 工具增加 `.ipynb` 解析。不新增文件 —— 扩展现有 `read.rs`。

当 `read_file` 遇到 `.ipynb` 文件时：
1. 按 JSON 解析（`serde_json::Value`）
2. 提取 `cells` 数组
3. 每个 cell：渲染 `cell_type`（code/markdown）、`source` 行，以及 `outputs`（仅 text/plain）
4. 返回如下格式的文本：
```
[Cell 1 - code]
import pandas as pd
df = pd.read_csv("data.csv")

[Output]
     name  age
0   Alice   30
1     Bob   25

[Cell 2 - markdown]
# Analysis
This notebook analyzes...
```

涉及文件：`crates/rustcode-core/src/tool/read.rs`（在二进制回退之前加 ipynb handler）

## 4. TodoWrite 工具

新工具 `todo` 注册进 ToolRegistry。每个会话一份内存任务列表。

```rust
pub struct TodoTool {
    items: Arc<Mutex<Vec<TodoItem>>>,
}

struct TodoItem {
    id: usize,
    content: String,
    status: TodoStatus, // Pending | InProgress | Completed
}
```

动作：`add`、`update`、`complete`、`list`
- `add`：压入新项，返回 id
- `update`：把状态改为 in_progress
- `complete`：标记为已完成
- `list`：返回所有项及其状态

LLM 用它跟踪多步任务。`/todo` 斜杠命令展示当前列表。

涉及文件：`crates/rustcode-core/src/tool/todo.rs`（新增）、`tool/mod.rs`、`cli/main.rs`（注册）、`commands.rs`（斜杠命令 /todo）
