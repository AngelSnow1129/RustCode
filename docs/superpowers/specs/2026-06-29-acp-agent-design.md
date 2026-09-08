# `rustcode acp` —— ACP Agent 模式（v1）

日期：2026-06-29
分支：`feat/acp-agent`（基于 `main` 的 worktree）
状态：设计已批准，待 spec 评审

## 动机

多 agent 协作越来越常见。rustcode 目前还无法接入使用 **Agent Client Protocol（ACP）** 的编辑器 / 编排器 —— 那是来自 Zed 的、基于 stdio 的 JSON-RPC 协议，它让 *client*（编辑器或多 agent 编排器）启动一个 *agent* 子进程并驱动它。我们希望提供 `rustcode acp`，让 rustcode 能作为这类团队中的一个 agent —— 就像 Claude Code 可以被直接放进 Zed 一样。

## 范围（v1）

**角色：仅 Agent 侧。** rustcode 作为 ACP agent 子进程运行，经 stdio 被驱动。（Client 侧 —— 由 rustcode 编排 *其它* ACP agent —— 明确不在范围内，未来单独出 spec。）

**功能集：核心 + 权限。**

范围内：
- `initialize`、`session/new`、`session/prompt`、`session/cancel`
- 流式 `session/update` 通知（文本、推理、工具调用）
- `session/request_permission` 接到 rustcode 现有的审批流程
- 工具调用更新携带 `raw_input` + 纯文本结果内容，以及一个
  `ToolKind`（read/edit/execute/…），让客户端渲染出合理的操作入口

> **实现说明（已推迟）：** 为编辑类工具提供结构化 `ToolCallContent::Diff { path,
> old_text, new_text }` —— 让 Zed 渲染富 diff 视图 —— 原本在本 spec 范围内，但**已推迟到 Phase 2**：生成它需要把每个编辑工具的参数解析成 old/new 文本，这更接近文件系统委派的工作，而不是 v1 的转换层。v1 先交付上面的纯文本基线；diff 内容随 Phase 2 落地。

不在范围内（后续阶段，各自独立 spec）：
- Phase 2：文件系统委派（`fs/read_text_file`、`fs/write_text_file`），以尊重编辑器未保存的缓冲区，**并**提供结构化编辑工具 `Diff` 内容，供客户端渲染富 diff
- 每会话拆除：v1 只在连接结束时释放所有会话（ACP 没有 `session/close`）；在 kernel task 完成时丢弃会话已推迟
- Phase 3：终端委派（`terminal/*`）、`plan` 更新、`available_commands`（斜杠命令）、`authenticate`
- `session/load`（会话恢复）—— v1 中声明为不支持

## 关键决策

| 决策 | 选择 | 理由 |
|---|---|---|
| 角色 | Agent（被驱动） | "把 rustcode 接入多 agent 团队"的直接含义 |
| v1 范围 | 核心 + 权限 | 能在 ACP 客户端里真正跑起来的最小完整集；几乎就是现有 kernel channel 之上的纯转换层 |
| 引擎 | kernel 原生 `AgentHandle`，经 coding `assemble` | ACP 权限需要 JSON-RPC request-id 关联；kernel 原生 `RequestId` 是 1:1 映射，而 legacy bridge 会合并并发审批 |
| 协议类型 | 官方 `agent-client-protocol` crate | 保证 wire format 与版本协商的正确性，Zed 互操作性最好；隔离在薄适配层之后，可替换 |
| 隔离方式 | 基于 `main` 的独立 worktree，新分支 `feat/acp-agent` | 新特性，保持发布分支干净，避免 per-turn 自动提交钩子把 WIP 打包进去 |

## 架构

新 crate **`rustcode-acp`**，依赖 `rustcode-kernel`、`rustcode-coding`、`rustcode-capabilities`、官方 `agent-client-protocol` crate，以及 `serde_json` / `tokio`。

唯一公开入口：

```rust
pub async fn serve_stdio(opts: AcpServeOptions) -> anyhow::Result<()>
```

`AcpServeOptions` 承载由 CLI 全局 flag 与已解析的 rustcode config 得到的 provider/model 覆盖。工作目录不在这里固定 —— 客户端通过 `session/new` 按会话提供。

CLI：在 `crates/rustcode-cli/src/main.rs` 的 `Commands` 枚举中增加一个 `Acp` 变体。处理函数解析 config（复用 headless 路径现有的 provider/model 解析逻辑）并调用 `rustcode_acp::serve_stdio`。该子命令复用现有全局 `--provider` / `--model` flag；cwd 来自客户端。

### stdout 纪律（硬性不变量）

ACP 模式下 **stdout 专用于 ACP JSON-RPC 流**。任何游离的 `println!` 都会破坏协议。rustcode 的 headless 模式已经把 stderr 指向真实终端并保持 stdout 干净（没有全局 stdout sink —— 已在 cli 启动处确认）。`rustcode-acp` 中的所有诊断信息都走 stderr 或文件 sink。这由代码评审与传输层 single-writer 共同保障。

## Crate / API 代次（已确定）

我们使用 `agent-client-protocol = "1.0.1"` —— 最新发布版本。它的 API 是 **builder + handler 闭包** 模型（不是更老的 `trait Agent` / `AgentSideConnection` 形态，后者只存在于约一年前的 0.4.x 版本 —— 0.15.x 与 1.0.x 都已转向 builder API）。wire 数据类型位于 `agent-client-protocol-schema` v1.1.0，以 `agent_client_protocol::schema::v1::*` 重新导出。

塑造代码实现的关键事实：
- 该 crate 是 **edition 2024**，并使用 **原生异步闭包**（`AsyncFnMut`），因此需要 Rust 工具链 ≥ 1.85。我们的 `rustcode-acp` crate 保持 edition 2021，只是依赖它。
- agent 的构造方式为
  `Agent.builder().name("rustcode").on_receive_request::<InitializeRequest>(handler, on_receive_request!())… .connect_to(Stdio::new()).await`。
  每个请求处理闭包接收 `(req, responder, cx: ConnectionTo<Client>)`；
  它调用 `responder.respond(resp)` 作答，并用 `cx.send_notification(...)`
  / `cx.send_request(...)` 来流式推送更新与请求权限。
- `on_receive_request!()` / `on_receive_notification!()` 宏会提供一个必需的 `to_future_hack` boxing 参数。
- 几乎所有 schema 类型都是 `#[non_exhaustive]`；应通过 `::new(...)` builder 构造，不能用结构体字面量。
- wire 协议与更老的系列完全相同（声明 `ProtocolVersion::V1`），因此编辑器/编排器互操作（例如 Zed）不受 crate 代次影响。

**待首个任务（spike）确认的开放风险：** `connect_to` 的分发循环是否并发运行各个处理 future —— 即 `session/cancel` 通知能否在 `session/prompt` 处理函数仍在 await 时被投递。我们的取消语义依赖于此；spike 的冒烟测试会把这个行为钉死。

## 组件

三个内部模块，各自可独立测试。（crate 的 builder 已经涵盖了原本需要手写的 JSON-RPC 传输层，因此没有独立的 `transport` 模块 —— 但 **single-writer / stdout 纪律** 这一不变量依然适用，由 crate 的 `Stdio` 传输层负责。）

### `protocol`
`agent_client_protocol`（含 `schema::v1::*`）之上的薄适配层：重新导出 crate 其余部分用到的类型，并集中构造 capability/version，这样 dispatch/translate 依赖的是我们的适配层接口，而不是散落各处的 crate 路径。同时负责构造接好所有 handler 的 agent `Builder`。

### `dispatch`
方法路由 + 会话表（`HashMap<SessionId, SessionState>`；支持多个并发会话）。
- `initialize` → 返回 capabilities（见下）
- `session/new` → 在客户端提供的 cwd 上运行 `prepare → assemble → spawn`；保存 `AgentHandle`；返回一个新的 `sessionId`
- `session/prompt` → 把 prompt content block 翻译成
  `AgentCommand::SendMessage { text, images }`，把 kernel 事件泵成
  `session/update` 通知直到 `TurnComplete`，然后返回
  `{ stopReason }`
- `session/cancel`（通知）→ `AgentCommand::Cancel`

### `translate`
纯函数：kernel `AgentEvent` → ACP `session/update`（以及 `StopReason` → ACP `stopReason`）。单元测试的主要目标（表驱动）。

## 引擎集成（路径 (a)）

每个会话：
1. `prepare(&cfg, PrepareOptions { cwd, ... }).await` → `CodingParts`
   （`rustcode-coding`；负责 MCP 连接、skill 加载、会话绑定）
2. `assemble(&mut parts, &cfg, provider).await` → kernel 原生 `Agent`
   （`crates/rustcode-coding/src/parts.rs:396`）
3. `agent.spawn()` → `AgentHandle { commands, events, task }`
   （`crates/rustcode-kernel/src/agent.rs:366`）
4. 泵循环：把 `events` 抽干 → `session/update`；把入站 prompt/cancel 路由到
   `commands`
5. 会话结束 / 关闭时：`AgentCommand::Shutdown`，await `task`

这与 cli 为其 headless 路径构造 provider 和 `CodingAgentConfig` 的方式一致，但保留 **原生** handle（cli headless v2 路径走 `spawn_bridged_runtime`；ACP 刻意不走，以保留原生 `RequestId`）。

## 事件映射

| kernel `AgentEvent` | ACP 事件 |
|---|---|
| `TextDelta(s)` | `session/update` → `agent_message_chunk` |
| `Reasoning(s)` | `session/update` → `agent_thought_chunk` |
| `ToolStarted { call }` | `session/update` → `tool_call`（id、title、kind、status；编辑类工具携带结构化 diff 内容） |
| `ToolResult { result }` | `session/update` → `tool_call_update`（status 为 completed/failed + content） |
| `Request { id, kind:"approval", payload }` | `session/request_permission` 请求；客户端选择 → `AgentCommand::Respond { id, value }` |
| `TurnComplete { reason }` | prompt 响应中的 `stopReason` |
| `Error` / provider 失败 | prompt 返回一个 JSON-RPC 错误 |
| `Cancelled` | `stopReason: cancelled` |
| `Usage` / `RateLimited` / `Warning` | v1 仅记日志（ACP 没有对应的标准 update 字段） |

kernel `StopReason` → ACP `schema::v1::StopReason` 的映射：`Stopped → EndTurn`；
`MaxRounds`/`MaxContinuations → MaxTurnRequests`；`Cancelled → Cancelled`；
`PromptRejected → Refusal`；`ProviderError`/`Timeout`/`RateLimited` 则由 prompt 处理函数返回 JSON-RPC
错误（不作为 stop reason）。

工具 **kind** 映射：rustcode 工具名 → ACP `ToolKind`（`Read` / `Edit` /
`Execute` / `Search` / `Fetch` / … / `Other`），让客户端展示合适的操作入口与图标。编辑/写入类工具附带一个 `ToolCallContent::Diff { path,
old_text, new_text }`，让客户端渲染 diff。

## 权限流程

kernel 的审批请求携带 `ApprovalRequest { call_id, tool, args }`
（`rustcode-capabilities/src/tools/approval.rs`）；它期望的响应是
`ApprovalResponse { decision: "allow"|"allow_always"|"deny", remember: bool }`
（失败时保守落到 `deny`）。

```
kernel  AgentEvent::Request{ id: RequestId(u64), kind:"approval",
                              payload: ApprovalRequest }
  → cx.send_request(RequestPermissionRequest::new(session_id, tool_call_update, options))
        options = [AllowOnce, AllowAlways, RejectOnce, RejectAlways]
                  (PermissionOption with stable option_id strings)
  → client returns RequestPermissionResponse {
        outcome: Selected { option_id } | Cancelled }
  → map option_id → decision JSON:
        allow_once    → {"decision":"allow"}
        allow_always  → {"decision":"allow","remember":true}
        reject_*      → {"decision":"deny"}
        Cancelled     → {"decision":"deny"}   (fail closed)
  → AgentCommand::Respond{ id, value }
```

kernel 原生 `RequestId` 让多个并发审批能正确关联 —— 这正是选择 kernel 原生路径而非 legacy bridge 的原因。

## `initialize` 的 capabilities（v1）

`InitializeResponse::new(req.protocol_version).agent_capabilities(...)`：
- `prompt_capabilities`：`image(true)`（kernel `SendMessage` 已携带
  `images: Vec<ImageContent>`）；v1 中 `embedded_context` 保持 false
- `load_session`：false（会话恢复推迟到后续阶段）
- `auth_methods`：`[]` —— rustcode 通过自己的 `/login` / config 完成认证。
  未认证时，`session/new` 返回清晰的错误，引导用户去
  执行 `rustcode login`。

把客户端的 `protocol_version` 原样回传（并收敛到我们支持的版本；`ProtocolVersion::V1`）。

## 错误处理

- 未知/未处理的方法 → 由 crate 的 catch-all 分发以 JSON-RPC 错误应答，分发循环继续存活（1.0.1 也默认忽略未处理的通知）
- kernel 致命错误（`ProviderError` / `Timeout` / `Error`）→ prompt 处理函数
  返回 `Err(agent_client_protocol::Error)` → JSON-RPC 错误响应
- 通过把全部诊断信息路由到 stderr/文件来防止 stdout 被污染（crate 的 `Stdio` 持有 stdout）；由评审 + crate 内禁止 `println!` 的规则保障

## 测试策略（TDD）

- **单元 —— `translate`**：表驱动；每个 kernel `AgentEvent` 断言出对应的 ACP
  `SessionUpdate`（比较序列化后的 JSON）。纯函数，性价比最高的
  覆盖。另外还包括 `StopReason` 映射与工具名 → `ToolKind`。
- **单元 —— `dispatch` 会话表 + 权限映射**：纯辅助逻辑 —— 会话
  插入/查找、`option_id → ApprovalResponse` 决策映射 —— 不经过传输层直接测试。
- **集成**：一个假 ACP 客户端（自身用 `Client.builder()` 建立在进程内双工之上，或用 stdio 管道拉起 `rustcode acp` 子进程）驱动
  `initialize → session/new → session/prompt`，断言它收到了
  `agent_message_chunk`、一次 `request_permission` 往返，以及一个终态
  `stop_reason`。使用 stub provider，因此不需要网络。

## 涉及文件清单（预计）

- `crates/rustcode-acp/` —— 新 crate（`Cargo.toml`、`src/lib.rs`、
  `src/protocol.rs`、`src/dispatch.rs`、`src/translate.rs`、`src/engine.rs`、
  测试）
- `crates/rustcode-cli/src/main.rs` —— `Acp` 命令变体 + 处理函数
- `Cargo.toml`（workspace）—— 在 `[workspace.dependencies]` 中加入 `agent-client-protocol` + `agent-client-protocol-schema`
  （锁定 `=1.0.1` / 匹配的 schema）
- `crates/rustcode-cli/Cargo.toml` —— 依赖 `rustcode-acp`

## 构建说明

按仓库约束：用 `CARGO_INCREMENTAL=0` 按包构建，而不是整个 workspace。`agent-client-protocol` 1.0.1 依赖是 **edition 2024 + 原生异步闭包** → 需要工具链 ≥ 1.85（对 2026 年没问题）；`rustcode-acp` 自身保持 edition 2021。注意在尺寸优化的 release profile（`opt-level=z`、`lto`、`panic=abort`）下新增依赖带来的体积；锁定 `=1.0.1` 以避免这个年轻 crate 的意外 API 变动。
