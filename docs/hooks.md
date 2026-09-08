# RustCode Hooks(钩子)

Hooks 系统允许你在 RustCode 的关键执行点插入自定义逻辑,从而获得灵活的扩展能力。

## 快速开始

### 三步配置:目录 → 脚本 → TOML

**第 1 步**:创建 hooks 目录

```bash
# Global hooks (apply to all projects)
mkdir -p ~/.rustcode/hooks

# Project-level hooks (only apply to current project, override same-name global hook)
mkdir -p .rustcode/hooks
```

**第 2 步**:编写 hook 脚本

创建 `~/.rustcode/hooks/my_hook.sh`:

```bash
#!/bin/bash
# Receive context JSON via stdin
INPUT=$(cat)

# Parse key info (install jq recommended: brew install jq / apt-get install jq)
if command -v jq &> /dev/null; then
    TOOL=$(echo "$INPUT" | jq -r '.tool_name // empty')
    echo "Hook saw tool: $TOOL" >&2
else
    # Without jq, use python instead:
    # TOOL=$(echo "$INPUT" | python3 -c "import sys,json; print(json.load(sys.stdin).get('tool_name',''))" 2>/dev/null)
    echo "Hook: raw input received" >&2
fi

# Return execution result
echo "ok"
```

给它加可执行权限:

```bash
chmod +x ~/.rustcode/hooks/my_hook.sh
```

**第 3 步**:配置 `hooks.toml`

创建 `~/.rustcode/hooks/hooks.toml`:

```toml
[[hooks]]
name = "my-hook"
description = "My custom hook"
trigger = "post_tool"      # Trigger timing
script = "my_hook.sh"
script_type = "shell"       # shell | python
enabled = true
timeout_secs = 2
```

完成!RustCode 启动时会自动加载 hooks。

---

## 配置总览

RustCode 支持**三种** hook 实现,由两个配置文件管理:

| 方式 | 配置文件 | 实现 | 适用场景 |
|------|---------|------|---------|
| **TOML ScriptHook** | `hooks.toml` → `[[hooks]]` | 本地脚本(shell/python) | 本地定制、快速原型 |
| **TOML Webhook** | `hooks.toml` → `[[webhooks]]` / `[[async_webhooks]]` | HTTP 远程调用 | 云服务、外部集成 |
| **JSON CC 兼容** | `.hooks.json` / `hooks.json` | Shell 命令(旧协议) | CC 插件兼容 |

> 三种方式可以共存。它们统一通过 `HookEngine::load_all()` 加载:
> 1. JSON hooks(来自 `hooks.json`)
> 2. TOML hooks(ScriptHook + WebhookHook,来自 `hooks.toml`)
> 3. 内置 Hooks(Rust 原生,自动注册)
>
> 全局 hooks 先加载,项目 hooks 后加载。同名项目 hooks **覆盖**全局 hooks(后加载者优先)。

---

## TOML ScriptHook(推荐)

### 支持的 trigger 取值

| trigger 取值 | 别名 | 触发时机 | 能否影响流程 |
|-----------|------|---------|:--:|
| `pre_tool` | `pre_tool_execution` | 工具执行前 | [+] 可阻止/修改参数 |
| `post_tool` | `post_tool_execution` | 工具执行后 | [-] 发出即忘 |
| `post_turn` | — | 轮次结束后 | [-] 发出即忘 |
| `system_prompt` | — | 构建 system prompt 时 | [+] 可追加指令 |

### 脚本输入(stdin JSON)

```json
{
  "tool_name": "edit_file",
  "tool_args": "{\"file_path\": \"...\", ...}",
  "working_dir": "/path/to/project",
  "session_id": "session-123",
  "turn_number": 5
}
```

`post_tool` 的 stdin 是嵌套结构,额外包含 `result_context`(与 `hook_context` 平级):

```json
{
  "hook_context": {
    "tool_name": "edit_file",
    "tool_args": "{...}",
    "working_dir": "/path/to/project",
    "session_id": "session-123",
    "turn_number": 5
  },
  "result_context": {
    "tool_name": "edit_file",
    "tool_args": "{...}",
    "result": "File updated",
    "success": true,
    "duration_ms": 150
  }
}
```

`system_prompt` 的 stdin 输入与 `post_turn` 相同(包含基础上下文,没有 `tool_args`/`result_context`)。脚本应把要追加的 system prompt 内容输出到 stdout(纯文本,或 JSON 的 `message` 字段)。

### 脚本输出格式

```
ok                    # Continue (default)
deny: <reason>        # Block (only effective for pre_tool)
modify: <new_args>    # Replace args (only effective for pre_tool)
warning: <message>    # Continue but print warning
```

也支持 JSON 输出(推荐):

```json
{"result": "ok", "message": "checked"}
{"result": "deny", "message": "unsafe path"}
{"result": "modify", "modified_content": "{\"file_path\": \"/safe\"}"}
{"result": "warning", "message": "file is large, review carefully"}
```

### 完整配置示例

```toml
[[hooks]]
name = "pre-check"
description = "Block dangerous write operations"
trigger = "pre_tool"
script = "check_write.sh"
script_type = "shell"
enabled = true
timeout_secs = 3
```

---

## TOML Webhook 配置

### 支持的 trigger 取值(多个用逗号分隔)

| trigger 取值(规范名) | 别名 | 触发时机 |
|-----------|------|---------|
| `turn_start` | — | 轮次开始前 |
| `tool_call_start` | — | 工具调用开始时 |
| `pre_tool` | `before_tool` | 工具执行前 |
| `post_tool` | `after_tool` | 工具执行后 |
| `turn_complete` | `after_turn` | 轮次结束后(含详细统计) |
| `post_turn` | — | 轮次结束后(旧版兼容) |
| `session_start` | — | 会话开始时 |
| `session_end` | — | 会话结束时 |
| `error` | — | 发生错误时 |
| `model_response` | — | 模型响应后 |
| `system_prompt` | — | 构建 system prompt 时 |
| `message`² | `message_received` | 收到用户消息时 |

> 采用**包含匹配**(trigger 以逗号分隔)。例如 `trigger = "pre_tool,post_tool"` 会在两个时机都触发。
>
> ² `message`:WebhookHook 已实现对应 trait,但引擎尚未注册 trigger 槽位,当前不可用。

### 同步 Webhook

```toml
[[webhooks]]
name = "slack-notify"
description = "Send tool call notifications to Slack"
trigger = "pre_tool,post_tool"
url = "https://hooks.slack.com/services/XXX"
method = "POST"
timeout_secs = 10
retries = 2
enabled = true

[webhooks.headers]
Authorization = "Bearer YOUR_TOKEN"
```

### 异步批量 Webhook(高频场景推荐)

```toml
[[async_webhooks]]
name = "audit-log"
trigger = "post_tool"
url = "https://log.example.com/batch"
timeout_secs = 10
batch_size = 20            # default 10, send when reached
flush_interval_ms = 1000   # default 1000ms, periodic flush
retries = 2
enabled = true

[async_webhooks.headers]
Authorization = "Bearer AUDIT_TOKEN"
```

> 异步 webhook 不阻塞主流程。详见 [Webhook 指南](./webhook-guide.md) 与 [异步 Webhook 指南](./async-webhook-guide.md)。

---

## JSON CC 兼容配置

兼容 Claude Code 插件的 `.hooks.json`。加载路径:

- `~/.rustcode/hooks.json` —— 全局
- `<project>/.hooks.json` —— 项目(覆盖同名全局 hook)

```json
{
  "hooks": {
    "my-hook": {
      "event": "pre_tool_use",
      "matcher": "write*",
      "command": "echo '{\"action\": \"allow\"}'",
      "timeout_ms": 10000,
      "disabled": false
    }
  }
}
```

支持的 `event` 取值:`pre_tool_use`、`post_tool_use`、`post_tool_use_failure`、`session_start`、`session_end`、`user_prompt_submit`。

> **大小写/风格不敏感:** 加载器同时接受 CC 的 PascalCase(`PreToolUse`、`PostToolUse`、`PostToolUseFailure`、`SessionStart`、`UserPromptSubmit`)和 snake_case(`pre_tool_use`、`post_tool_use_failure`、`session_start` 等)—— 两种写法等价。

Hook 通过环境变量接收上下文(`RUSTCODE_HOOK_EVENT`、`RUSTCODE_HOOK_CONTEXT`、`RUSTCODE_TOOL_NAME` 等)。stdout 协议按 event 而不同:

- **`pre_tool_use`** —— 输出 `{"action":"allow"}` / `{"action":"block","reason":"..."}` / `{"action":"modify","args":{...}}`(`args` 会替换工具调用参数)
- **`user_prompt_submit`** —— 输出 `{"decision":"block","reason":"..."}` 阻止提交,或输出 `{"hookSpecificOutput":{"additionalContext":"..."}}` 注入额外上下文;纯文本 stdout 也被视为 additionalContext 注入
- **`post_tool_use`** —— 发出即忘;stdout 不影响流程
- **`post_tool_use_failure`** —— 与 `post_tool_use` 类似,但仅在工具调用失败时触发(`tool_response` 携带错误输出),因此插件可区分成功与失败

---

## 内置 Hooks(无需配置,自动启用)

| Hook | 触发时机 | 作用 |
|------|---------|------|
| `ToolAuditLogHook` | 工具调用时 | 将调用记录到审计日志(tracing) |
| `TurnStatsHook` | 轮次开始 + 结束 | 统计轮次耗时与操作数 |
| `AutoCommitHook` | 轮次结束 | 每 N 个轮次自动 `git commit` |
| `SessionSummaryHook` | 会话开始 + 结束 | 打印会话摘要 |
| `ErrorReportHook` | 发生错误时 | 记录错误详情 |
| `ResponseValidationHook` | 模型响应后 | 检测敏感信息 |

内置 hooks 自动注册,目前尚不能通过配置禁用(未来的 CLI 会提供启用/禁用开关)。同名项目级 hooks 不能覆盖内置 hooks(内置 hooks 是 Rust 原生实现,位于 TOML 配置体系之外)。

---

## CLI 命令

```bash
# List loaded hooks
rustcode hooks list

# View config paths
rustcode hooks paths

# Test a single hook
rustcode hooks test my-hook
```
---

## 调试技巧

### 手动测试 hook 脚本

TOML ScriptHook 通过 stdin 接收上下文 JSON（字段对应 `HookCtx`：`tool_name` / `tool_args` / `working_dir` / `session_id` / `turn_number`，无 `event` 字段）：

```bash
# pre_tool 测试上下文（扁平结构）
echo '{"tool_name":"read_file","tool_args":"{}","working_dir":"/tmp","session_id":"s1","turn_number":1}' | bash path/to/hook.sh

# post_tool 测试上下文（嵌套结构，含 result_context）
echo '{"hook_context":{"tool_name":"read_file","tool_args":"{}","working_dir":"/tmp","session_id":"s1","turn_number":1},"result_context":{"tool_name":"read_file","tool_args":"{}","result":"File content here","success":true,"duration_ms":12}}' | bash path/to/hook.sh
```

JSON CC 兼容 Hook 通过环境变量接收（TOML ScriptHook 不适用，TOML 用 stdin）：

```bash
# 导出环境变量模拟运行环境（仅 JSON CC 格式）
export RUSTCODE_HOOK_EVENT="post_tool_use"
export RUSTCODE_TOOL_NAME="read_file"
export RUSTCODE_HOOK_CONTEXT='{"tool_name":"read_file"}'
python path/to/hook.py
```

### 配置文件语法校验

```bash
# TOML 格式校验（需要 Python ≥ 3.11；旧版请 pip install tomli 并将 tomllib 替换为 tomli）
python -c "from pathlib import Path; import tomllib; tomllib.load(Path('path/to/hooks.toml').open('rb'))"

# JSON 格式校验
python -c "from pathlib import Path; import json; json.load(Path('path/to/hooks.json').open('rb'))"
```

### CLI 排查命令

```bash
# 查看当前加载的所有 hook
rustcode hooks list

# 查看 hook 配置路径
rustcode hooks paths

# 测试单个 hook 是否正常触发
rustcode hooks test <hook-name>
```

### Hook 不触发的 6 步排查清单

| 步骤 | 检查项 | 常见问题 |
|------|--------|---------|
| 1 | 文件路径是否存在 | `~` 不会自动展开，需用绝对路径（如 `C:\Users\you\...` 或 `/home/you/...`） |
| 2 | `enabled = true` | 默认 `true`，检查是否意外设为 `false` |
| 3 | `trigger` / `event` 拼写正确 | 参考上方事件表，大小写敏感 |
| 4 | 脚本有执行权限 | Linux/macOS 需 `chmod +x` |
| 5 | 脚本没有超时 | TOML 默认 2s，JSON 默认 10s |
| 6 | 项目级 hook 覆盖了全局 hook | 项目 hook 优先级更高 |

---

## 安全注意事项

1. **项目 hooks 覆盖同名全局 hooks**(项目 hooks 后加载)
2. **Hooks 不能绕过权限系统** —— `pre_tool` 的 deny 不会覆盖用户自己的 `always_allow` 设置
3. **脚本执行有超时** —— TOML ScriptHook 默认 2s,JSON 默认 10s,Webhook 默认 10s
4. **脚本以用户权限运行** —— 需自行注意脚本本身的安全性
5. **超时/崩溃为 fail-open** —— 脚本超时或崩溃被视为 `ok`,不会阻塞流程
6. **Windows 兼容性** —— `~` 不会自动展开;请使用绝对路径(如 `C:\Users\you\...` 或 `/home/you/...`);路径分隔符用 `\\` 或 `/`;Python 脚本请显式指定解释器路径

---

## 相关文档

- [CLI 指南](./hook-cli-guide.md) —— `rustcode hooks` 命令参考
- [完整时机清单](./hook-timing-complete.md) —— 所有 hook 时机与可用配置
- [Webhook 指南](./webhook-guide.md) —— HTTP 远程调用
- [异步 Webhook 指南](./async-webhook-guide.md) —— 批量异步投递
- [架构说明](./hook-architecture.md) —— 面向开发者的架构参考
