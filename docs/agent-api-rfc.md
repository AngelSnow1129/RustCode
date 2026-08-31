# RFC: RustCode Agent API — 可供外部程序调用的独立 Agent 服务

状态: DRAFT (评审中)
作者: fork 维护
日期: 2026-08-30
参考: <https://github.com/m-sec-org/kimi-cli-for-xbow> (轻量并发 agent / `clew` 调度 / `ClewAPI` 编程式 API / `agent-worktree` 隔离 / 原生 retry + 限流)

---

## 1. 背景与目标

### 1.1 现状

RustCode 当前的能力以**交互式前端**为中心：

- `rustcode-cli` / `rustcode-tuix` / WebUI 通过 `rustcode-daemon` 的 `/live` SSE
  协议驱动 `CodingRuntime`。
- `rustcode-daemon` 已暴露 `run_server`（库函数）+ `CodingRuntime` 装配逻辑
  （`runtime_host.rs` 的 `coding_provider_factory` / `start_native_runtime_with_session`），
  具备被程序调用的**基础**，但 `/live` 协议是为 WebUI 设计的**交互协议**，
  不是"程序调用 agent 完成任务"的**编程式 Task API**。

### 1.2 参考项目

`kimi-cli-for-xbow` 的关键设计（待对齐）：

| 维度 | kimi 做法 | 本项目对应 |
|------|-----------|------------|
| 编程式 API | `ClewAPI` 类,可被外部调度器直接调用 | 新增 `rustcode-agent-api` crate |
| 调度层 | `clew` 负责并发 + 任务编排 | 复用 `rustcode-coding/src/rate_limit.rs` |
| 工作树隔离 | `agent-worktree`,每任务独立 git worktree | 复用 capabilities git tool + `working_dir` seam |
| 重试/限流 | 原生 retry + rate_limit | 复用 provider `retry_max_attempts` + task 级退避 |
| headless 审批 | 默认非交互, auto-approve / fail-closed | 复用 kernel `ApprovalMode` fail-closed 语义 |
| 管道组合 | `--apply` / 单命令调用 | 复用 `SkillRegistry` + `skill` 字段 |

### 1.3 目标

1. 暴露一套**任务导向 (task-oriented)** 的编程式 API,使外部编排程序
   （类 xbow 的 scheduler / orchestrator）能以 "提交任务 → 轮询/流式 → 取结果"
   的方式驱动 RustCode agent,而非依赖交互式 UI。
2. 任务在**隔离的工作树**中运行,互不污染,结果可独立评审/合并。
3. 无人值守场景用**明确、非交互的审批策略**,失败语义 fail-closed。
4. 复用现有 `CodingRuntime` 单一状态所有权,**不新建第二生命周期 owner**。

### 1.4 非目标

- 不改动 `rustcode-kernel` 的 agent 循环。
- 不引入遥测 / Emoji / 网络上报（遵循 fork 硬性约束）。
- 不替换 `/live` WebUI 协议,二者并存。

---

## 2. 架构

```text
外部程序 (orchestrator / scheduler)
        │  POST /agent/tasks          {prompt, cwd, provider?, skill?, approval_mode, timeout}
        │  GET  /agent/tasks/{id}     (轮询)
        │  GET  /agent/tasks/{id}/stream (SSE, 可选)
        │  POST /agent/tasks/{id}/cancel
        ▼
  rustcode-agent-api  (新增 crate)
        │  ├─ TaskManager: 任务队列 + 状态机 + 限流准入
        │  ├─ TaskRunner:  每任务 spawn 一个 CodingRuntime (单 owner)
        │  └─ WorktreeScope: per-task 隔离目录 / git worktree
        ▼
  CodingRuntime (已有, 单 owner) → rustcode-kernel Agent
        │  provider (anthropic/openai/ollama) + tools + skills
        ▼
  LLM Provider / 本地工具
```

设计要点:
- `rustcode-agent-api` **仅依赖** `rustcode-coding` / `rustcode-capabilities` /
  `rustcode-daemon`(库导出) / `rustcode-kernel`。**不反向依赖** core/L2/前端。
- 每个 task 拥有自己独立的 `CodingRuntime` 实例 (由 `CodingRuntimeHandle`
  `start_native_runtime_with_session` 装配),满足 "单一状态所有权" 不变量。

---

## 3. 目录结构 (新增 crate)

```text
crates/rustcode-agent-api/
├── Cargo.toml
├── src/
│   ├── lib.rs              # 库导出: AgentServer, serve()
│   ├── server.rs           # axum Router 装配 + 路由
│   ├── task/
│   │   ├── manager.rs      # TaskManager: 注册/状态机/限流准入
│   │   ├── model.rs        # TaskId, TaskState, TaskRequest, TaskResult
│   │   ├── runner.rs       # TaskRunner: spawn CodingRuntime + 驱动 submit/cancel
│   │   └── worktree.rs     # WorktreeScope: per-task 隔离目录 + git worktree 生命周期
│   ├── approval.rs         # AgentApprovalMode (Bypass / AutoOnly) 适配器
│   ├── limit.rs            # 复用 rate_limit 的并发配额 + task 级退避
│   └── errors.rs           # 强类型错误 (thiserror)
└── bin/
    └── rustcode-agent.rs   # 独立二进制: `rustcode-agent serve --port 8080`
```

> `bin` 放在 crate 内 (与 `rustcode-cli` 同构), 由 workspace 统一管理。

---

## 4. 接口契约 (REST, JSON)

### 4.1 提交任务

`POST /agent/tasks`

```json
{
  "prompt": "为 src/foo.rs 添加单元测试",
  "cwd": "/workspace/repo",
  "provider": "claude",          // 可选, 缺省用配置默认
  "skill": "write-tests",        // 可选, 复用 SkillRegistry
  "approval_mode": "auto-only",  // "bypass" | "auto-only", 缺省 auto-only
  "timeout_secs": 600,           // 可选, task 级上限
  "worktree": "isolate"          // "isolate" | "shared", 缺省 isolate
}
```

响应 `202 Accepted`:

```json
{ "task_id": "t_01HXYZ..." }
```

### 4.2 轮询状态/结果

`GET /agent/tasks/{id}`

```json
{
  "task_id": "t_01HXYZ...",
  "state": "running",            // pending|running|succeeded|failed|cancelled
  "created_at": 1234567890,
  "updated_at": 1234567950,
  "result": null,                // running 时为 null
  "error": null
}
```

终态 (`succeeded` / `failed` / `cancelled`) 时 `result`:

```json
{
  "state": "succeeded",
  "result": {
    "summary": "已添加 12 个测试用例",
    "diff": "diff --git ...",     // 可选, 来自 worktree 与 base 的 diff
    "artifacts": ["/tmp/worktree-xxx/tests/foo_test.rs"]
  }
}
```

### 4.3 流式事件 (可选)

`GET /agent/tasks/{id}/stream` (SSE, `text/event-stream`)

事件与 kernel `AgentEvent` 对齐: `token` / `tool_call` / `tool_result` /
`turn_complete` / `error`。调用方不需要实时反馈时可只轮询 4.2。

### 4.4 取消

`POST /agent/tasks/{id}/cancel` → `200 OK` `{ "state": "cancelled" }`

底层调用 `CodingRuntimeHandle::cancel()` (已有 `rustcode-daemon/src/live_hub.rs:773`)。

---

## 5. 状态机

```text
        submit
  pending ──────────────► running
     │                      │   │
     │ cancel               │ cancel
     ▼                      ▼   ▼
  cancelled ◄───────── succeeded / failed
```

- `pending`: 已入队, 等待限流准入 (并发配额空闲后转 `running`)。
- `running`: `CodingRuntime` 已 spawn, 正在执行。
- 终态 (`succeeded`/`failed`/`cancelled`) **不可逆**; 迟到事件不污染已终态任务。
- 每个 accepted task 必有终态 (成功/错误/取消), 遵循 runtime 生命周期不变量
  "每个 accepted operation 都有终态"。

### 失败语义 (fail-closed)

- 审批超时 / provider 429 / tool 权限不足 → 任务转 `failed`, 不静默重试为成功。
- `auto-only` 模式下遇到非预批准工具 → fail-closed (不阻塞、不交互、不默认放行)。

---

## 6. 审批策略

复用 kernel `ApprovalMode` 的 fail-closed 语义 (`CodingAgentConfig` 注释明确
`HEADLESS` 场景用 `Some(d)` 自动降级为 deny)。

新增 `AgentApprovalMode` (agent-api 层枚举) 映射到 kernel:

| AgentApprovalMode | 语义 | 映射 |
|-------------------|------|------|
| `bypass` | 完全自动, 危险操作显式声明 | kernel `ApprovalMode::Bypass` (需安全边界校验) |
| `auto-only` (默认) | 仅允许预批准安全工具, 其余 fail-closed | `ApprovalMode::{Auto, fail_closed_timeout}` |

`auto-only` 的预批准工具集复用 `permission_bridge.rs` 的 safe-tool 分类。

---

## 7. 隔离设计 (参考 agent-worktree)

每个 `worktree: "isolate"` 任务:

1. 在 `cwd` 对应 git 仓库中创建 worktree (`git worktree add`), 或退化为临时
   目录副本 (无 git 时)。
2. 将 worktree 路径作为该 task 的 `CodingAgentConfig.working_dir`
   (已存在 `PINNED working_dir` seam, 并发 agent 不 race)。
3. task 终态后: `succeeded` 保留 worktree 供调用方评审/合并; `failed`/`cancelled`
   按策略清理 (默认保留, 由 TTL 回收, 避免误删用户产物)。
4. `shared` 模式复用调用方传入的 `cwd`, 不隔离 (用于只读分析类任务)。

> 复用 `rustcode-capabilities` 的 git tool, `WorktreeScope` 只管理生命周期,
> 不重复实现 git 逻辑。

---

## 8. 限流与重试 (参考 kimi retry/rate_limit)

### 8.1 并发准入

复用 `rustcode-coding/src/rate_limit.rs` 的并发模型, 给 agent-api 独立配额
(默认 `max_concurrent_tasks = 4`, 可配置)。超过配额的任务停留在 `pending`
队列, 不拒绝。

### 8.2 provider 限流

- provider 层已有 `retry_max_attempts` (OpenAI/Anthropic 适配器)。
- task 层额外: 收到 429 时退避重排回 `pending` (借用 `RateLimitWindowSource`
  的 `seconds_until_reset` 信息, 来自 `runtime_host.rs` 的 `CodingPlanRateLimitSource`),
  避免雪崩。

### 8.3 task 超时

`timeout_secs` 映射到 kernel `stream_timeout` + 一个 task watchdog
(`tokio` `timeout` + `CancellationToken`), 超时 → `failed` (fail-closed),
不无限挂起。

---

## 9. skill 管道组合 (参考 kimi --apply)

- 复用 `rustcode-capabilities/src/skills/registry.rs` 的 `SkillRegistry::load`。
- 提交任务的 `skill` 字段指定用哪个 skill 完成; 缺省为空 (通用 agent)。
- 同时提供 CLI 入口 (对应 kimi 编程式调用):

```bash
rustcode agent run --skill write-tests --prompt "..." --cwd /workspace/repo
```

内部等价于一次 `POST /agent/tasks` + 等待终态。

---

## 10. 与现有约束的一致性

| 约束 | 落实 |
|------|------|
| 状态单一所有权 | 每 task 一个 `CodingRuntime`, 无第二 owner |
| 零遥测 / 无 Emoji | 全程 ASCII 标签, 无上报 |
| 跨 crate 依赖方向 | `rustcode-agent-api` → coding/capabilities/daemon/kernel, 不反向 |
| provider 解耦 | 复用 `CodingProviderFactory`, 不碰 provider 层 |
| 失败语义 | fail-closed, 终态不可逆 |
| SECURITY | api_key 用 `env:VAR` / 占位符, 不硬编码 |

---

## 11. 落地步骤 (建议顺序)

1. **P0 MVP** — `rustcode-agent-api` crate 骨架 + `POST /agent/tasks` 同步/轮询接口,
   接 `start_native_runtime_with_session`, 跑通最小可调用示例 (单 task, shared worktree)。
2. **P1** — 状态机 + `cancel` + `GET /agent/tasks/{id}`; 接入 `rate_limit` 并发配额。
3. **P1** — `WorktreeScope` 隔离 (`isolate` 模式) + 终态保留/清理。
4. **P2** — `auto-only` 审批策略 (`fail-closed`) + `bypass` 安全边界校验。
5. **P2** — SSE 流式 (`/stream`) + `rustcode agent` CLI + `skill` 字段。
6. **P3** — task 级 429 退避重排 + `rustcode-agent` 独立二进制。

---

## 12. 验收基线

- 外部程序可 `POST /agent/tasks` 提交, 轮询得到 `succeeded` 结果 (含 diff)。
- 并发提交 N>N 配额任务时, 超额任务 `pending` 排队, 不报错。
- `auto-only` 遇非安全工具 → `failed`, 不挂起、不默认放行。
- `isolate` 任务的产物不污染 `cwd` 原 worktree。
- `cargo test -p rustcode-agent-api` 通过; 与 `/live` WebUI 协议不冲突。

---

## 13. 待评审问题

- [ ] 独立 crate vs 在 `rustcode-daemon` 内新增模块? (倾向独立, 依赖更清晰)
- [ ] `auto-only` 的预批准工具清单具体范围? (需查 `permission_bridge`)
- [ ] worktree 清理策略默认保留还是删除? (倾向保留 + TTL)
- [ ] 是否暴露 gRPC / 仅 REST? (倾向 REST + SSE, 与现有 daemon 一致)
</content>
</invoke>
