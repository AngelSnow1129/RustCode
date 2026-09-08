# rustcode-kernel — Phase A0 验证性试验

面向中立内核平台化策略（`docs/superpowers/specs/2026-06-05-rustcode-kernel-platform-strategy.md`）的
设计验证性试验。内部实现是最小化的、可丢弃的 —— Phase A1 承接的是它**公开 API 的 *形态***，
并把已验证的生产热路径搬进去。

## 它证明了什么

| 论断 | 验证位置 |
|---|---|
| 1. 中立内核 —— 一次回合运行时没有 persona、也没有中间件 | `tests/spike_claims.rs::neutral_turn_runs_without_persona_or_middleware` |
| 2. 审批是叠加在 id 关联往返之上的外部中间件 | `tests/spike_claims.rs::approval_middleware_gates_risky_tool_via_id_roundtrip` |
| 3. 工具的选择性挂载 —— 未挂载的工具不可见、不生效 | `src/tool.rs::tests::only_mounted_tools_are_exposed_or_resolvable` |
| 4. 同一个原语同时服务 one-shot 与交互式 driver | `tests/...::one_shot_adapter_auto_answers_and_aggregates` + `examples/minimal_specialization.rs` |
| 5. 线上兼容（serde 往返）—— web/daemon 可以复用同一条 seam | `tests/...::events_and_commands_are_wire_serializable` |
| 6. LifecycleHooks —— 回合级注入（turn_end 可让循环继续）+ TurnStarted 观测 | `tests/spike_claims.rs::lifecycle_hook_injects_and_continues_loop` |
| 7. 完整的 LifecycleHooks 面 —— 8 个回合级点位全部接线并触发 | `tests/spike_claims.rs::lifecycle_hooks_complete_surface_all_fire` |
| 8. 执行状态被记录（Message.meta）+ 以尾部提醒投影给 LLM + 前缀缓存安全 | `tests/spike_claims.rs::execution_state_recorded_projected_to_llm_and_cache_safe` |
| 9. 回合预算投影给 LLM（"round X/Y"）+ 硬上限，记录在 meta.round 中，前缀缓存安全 | `tests/spike_claims.rs::round_budget_projected_to_llm_and_hard_capped` |
| 10. on_model_response 拿到 `&mut Message` —— 可以改写响应；改写结果会落入存储（经 Snapshot） | `tests/spike_claims.rs::on_model_response_can_transform_response_into_storage` |
| 11. on_model_response 对 tool_calls 的编辑会被采纳（被丢弃的调用不会执行） | `tests/spike_claims.rs::dropping_tool_calls_in_on_model_response_prevents_execution` |
| 12. ToolMiddleware 的 `before` 可改写/拦截（不产生幽灵 ToolStarted）+ `after` 可变换结果 | `tests/spike_claims.rs::tool_middleware_rewrites_blocks_and_transforms` |
| 13. 命令级审批 —— 参数感知的 `Tool::risk(args)`；ApprovalMiddleware 拦截危险命令、放行安全命令，并缓存会话级授权 | `tests/spike_claims.rs::dangerous_command_requires_approval_safe_does_not_and_grant_is_cached` |
| 14. user_prompt_submit 可以拦下一条 prompt（返回 Err → prompt 被拒绝、不产生回合、不落存储） | `tests/spike_claims.rs::user_prompt_submit_can_block_a_prompt` |

## driver 模型

只有一个原语：一个长生命周期的 session，消费 `AgentCommand`、产出 `AgentEvent`
（`AgentHandle`）。往返 seam 就是那条 id 关联的
`AgentEvent::Request{id,kind,payload}` ↔ `AgentCommand::Respond{id,value}`。用来兑现中间件
await 的那个 `oneshot` 只存在于 `RequestCtx`（内核侧），从不进入事件 —— 因此事件与命令
都可序列化，进程内与跨网络都能跑。`run_to_completion(input, policy)` 是面向批处理/CI 的
one-shot 适配器。四种 driver 形态（one-shot/CI、TUI、web、server）全部坐落在这一个
原语之上：

| driver | 命令来源 | 事件去向 | 请求由谁应答 |
|---|---|---|---|
| one-shot / CI / CodeReview | 一条 SendMessage | 聚合后的 Outcome | AutoRespond 策略 |
| TUI | 按键 | 渲染循环 | 模态框 → Respond |
| Web | WS/HTTP → AgentCommand | AgentEvent → SSE/WS | 用户 → Respond 帧 |
| server / daemon | 每会话 RPC | 每会话 SSE | 策略或远端用户 |

## Hook 面（perceive 与 inject）

两套截然不同的机制：**perceive** = 只读的 `AgentEvent` 流（观察者无法改变循环）；
**inject** = `LifecycleHooks` trait（在循环内部运行，可以修改/延续循环）。该 trait 声明了
8 个回合级点位 —— `session_start`、`user_prompt_submit`、`turn_start`、`pre_request`、
`on_model_response`、`turn_end`、`on_error`、`session_end` —— 每一个都接进了循环（论断 7
断言每一个都会触发）。TOOL 级的关注点（改写/拦截/变换一次工具调用）位于可组合的
`ToolMiddleware`（`before` + `after`）中，而**不在** LifecycleHooks 里。进程外注入复用
那条 id 关联的 `Request`/`Respond` 往返（hook 去问远端 driver 并等待）。

执行状态反馈遵循一条规则：在 `on_model_response` **记录**（内核原生的 `Message.meta` 旁路），
在 `pre_request` 以尾部提醒的方式**投影**给 LLM —— 绝不改动历史字节（前缀缓存安全）。需要
知道循环位置的 hook 会拿到 `TurnCtx { round, max_rounds }`；`pre_request` 用它把回合预算
投影给 LLM，内核同时在 `max_rounds` 处硬性截断循环，作为一道保险丝。

`on_model_response` 收到的是 `&mut Message`（已完整构建、且 `meta` 由内核填好的 assistant
消息）。hook 可以**观测**也可以**改写**这个响应（例如给文本脱敏、做截断）。`MessageMeta`
只存放内核实测到的事实（`tokens`、`elapsed_ms`、`ctx_window`、`used_tokens`、`utilization`、
`round`）；成本及其它特化关注点都留在内核之外。

## 关键边界事实

- 内核核心（`agent.rs`、`event.rs`、`tool.rs`）从不出现 "approval" 这个词。工具自带一个
  参数感知的 `risk(&str) -> RiskLevel` 方法（工具自己知道哪些命令危险）；审批完全位于
  `testkit::ApprovalMiddleware`（特化侧）之上，依托 `RequestCtx::request`。
  `ApprovalMiddleware` 还持有会话级授权缓存，因此相同的危险命令每个会话只需批准一次。
- `ToolContext` 不携带任何语义/图/lsp 服务 —— 内核一个都不需要。
- 本 crate 被排除在 workspace 的 `default-members` 之外，因此产品构建不受影响。

## 运行

    cargo test -p rustcode-kernel
    cargo run -p rustcode-kernel --example minimal_specialization

## 下一步（Phase A1）

把生产热路径搬进这些槽位，且**不重写**：`TurnRunner` 循环 → `agent.rs`；`ctx/render` →
位于 persona 注入点之后的一个 `CtxBuilder` impl；`conversation` → `message.rs`；中立 provider
实现 → `provider.rs`。保留前缀缓存不变量与既有的边界修复。
