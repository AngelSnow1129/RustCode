# 交接文件：P3 唤醒持久化（Wakeup Persistence）

> 生成日期：2026-10-05 · 推进方式：RustCode 多 agent（主会话编排 + explorer 只读勘察）
> 当前 HEAD：`a3e24012` · 状态：**代码完成、全 workspace 编译绿、P3 源文件已提交 `a3e24012`（dev，未推送）**

## 0. 一句话目标

让“唤醒意图”落盘（`ScheduledWakeup` JSON），跨进程重启不丢，由 daemon tick（`rustcode schedule tick --once` 子进程）到期兑现为一次 `RunTrigger::Wakeup` task run。用户裁定 = **完整 session 恢复**（不恢复 `/loop` 内存轮次）。

## 1. 设计来源与裁定

- 设计文档：`docs/plans/2026-09-23-continuous-agent-design.md` 第 5.5 节（五期 P0–P3 严格串行）。
- 用户裁定（2026-10-05）：P3 = “完整 session 恢复”；方案“唤醒意图持久化 + 既有 daemon tick 到期兑现”，不恢复 `/loop` 内存轮次。
- 分层合规：WakeupRegistry 在 `config` leaf，coding/cli 上层依赖；**无 capability -> config 反向依赖**。

## 2. 当前状态

- 已完成 `cargo check --workspace` 全绿、零警告（本次实测，项目 `-D warnings` 门禁满足）。
- 已完成 `cargo check -p rustcode` 绿（daemon 接线）。
- config 层 2 项 wakeup 测试**实跑 passed**（`-j 1 --config 'profile.dev.package.rustcode.debug=0'` 链接绕 OOM）。
- P3 源文件已提交 `a3e24012`（dev，未推送）；**非 P3 改动与文档仍在工作树**（见第 7 节）。
- coding/daemon/clix 测试二进制因 4 GiB cgroup 链接 OOM，**仅 check 绿、未实跑**；端到端“注册 -> 重启 -> tick -> run”未实跑。

## 3. 架构决策 / 硬约束（接手必读，避免踩坑）

1. **单一落点** `crates/rustcode-config/src/schedule.rs`：`WakeupRegistry{register/claim_due/consume/list/prune}` 存 `<schedules_root>/wakeups/<id>.json`，复用 `valid_id` + `task_path_in` + `schedules_root`。
2. **文件后端、无内存缓存** -> “新进程”重读磁盘即见全部条目，跨重启持久化天然成立（无需任何 cache 失效逻辑）。这是 P3 能“跨重启”的核心原因。
3. **测试隔离**：coding/tuix/daemon/clix 用 `#[ctor]` 把 `RUSTCODE_HOME` 重定向到临时目录；config 测试用私有 `*_in(root)` 助手 + 临时目录。**config 既有测试 `wakeup_register_claim_and_consume` 用全局 `wakeups_root()`，整跑会写真实 `~/.rustcode`**——历史遗留，隔离测试已避开。
4. **`ScheduleRunner` 不能放 capabilities(L1)**；daemon tick 以子进程复用 `schedule tick` 入口（单飞锁 `fs2`）。
5. **`native_protocol == false` 时 wakeup 不生效**（硬约束）。
6. **`generation` 比对防旧 runtime 污染**：当前 `generation` 恒 `None`，比对钩子是 no-op；待 generation 持久化才真正生效。
7. **`list_wakeups` 是自由函数** `rustcode_config::schedule::list_wakeups()`，不是 `WakeupRegistry::*`（设计文档名与现实落地不一致，别按文档找方法）。

## 4. 已完成批次（文件:符号 + 验证）

| 批次 | 内容 | 关键落点 | 验证 |
|---|---|---|---|
| 1 config 地基 | `ScheduledWakeup` + `WakeupRegistry` 全量；守门测试 | `schedule.rs` | check 绿 + 测试 passed |
| 2 coding 层 | `RunTrigger::Wakeup`；`WakeupRequest` 补 (De)Serialize + `id`；`ScheduleWakeupTool::execute` 落盘；`runtime.rs` 快路径 consume；25 处测试构造点补齐 | `controllers.rs`、`schedule_wakeup_tool.rs`、`runtime.rs` | check --tests 绿 |
| 3a cli 兑现骨架 | `ScheduledWakeup.task_id`；`ScheduleWakeupTool` 加 task_id；`tick_once()` 到期兑现分支（claim -> 窗口外 consume+skipped / 窗口内 `run_task_with(task_id, Wakeup, false)` / 兑现后 consume 去重） | `schedule_cmd.rs` | check 绿 |
| 3b task_id 注入链 | `CodingRuntimeStart` + `RuntimeResources` 加 `task_id`；解构 + `ScheduleWakeupTool::new` 传 task_id；`RuntimeResources` 构造传 task_id（覆盖 reconfigure） | coding | check --tests 绿 |
| 3c tuix 展示 | `/loop status` 追加持久唤醒列表（`list_wakeups` 过滤 `!consumed` + 新 `Msg::LoopWakeups`） | `tuix/commands.rs`、`i18n/*` | check --workspace 绿 |
| daemon 接线 | `spawn_native_cli_runtime` 加 `task_id` 形参；`run_task_with` 传 `Some(task.id.clone())`；交互三处保持 `None` | `main.rs`、`im_runner.rs` | check -p rustcode 绿 |
| 4 测试 | 新增 `wakeup_survives_process_restart` + `wakeup_consumed_exactly_once`（实测 passed）；修复既有测试缺 3 处 `task_id` | `schedule.rs` | 实测 passed |

**功能闭环已接通**：`CodingRuntimeStart.task_id -> RuntimeResources.task_id -> ScheduleWakeupTool.task_id -> ScheduledWakeup.task_id` 全链贯通；`tick_once` 赎回分支 `run_task_with(task_id, Wakeup)` 真正启动 task run。

## 5. 关键文件 / 符号索引（接手快速定位）

- `crates/rustcode-config/src/schedule.rs`：`ScheduledWakeup`、`WakeupRegistry`、`list_wakeups`（free fn）、`mint_wakeup_id`、`valid_id`、`schedules_root`/`wakeups_root`、私有 `register_wakeup_in`/`list_wakeups_in`/`consume_wakeup_in`（测试用）、测试 `wakeup_register_claim_and_consume`、`wakeup_survives_process_restart`、`wakeup_consumed_exactly_once`。
- `crates/rustcode-config/src/i18n/{messages,zh_cn,en}.rs`：`Msg::LoopWakeups { count: usize }`（三处同步）。
- `crates/rustcode-coding/src/controllers/mod.rs`：`WakeupRequest`（补 `id: Option<String>`、补 `Serialize`/`Deserialize`）。
- `crates/rustcode-coding/src/controllers.rs`：`ScheduleWakeupTool`（struct 约 905 行、impl/execute 约 914/929 行；`task_id` 字段；`execute` 落盘，`active == false` 仅落盘返回成功）。注意：与 `WakeupRequest`（约 101 行）同在 `controllers.rs`，**无独立 `schedule_wakeup_tool.rs` 文件**。
- `crates/rustcode-coding/src/runtime.rs`：快路径 `consume_wakeup(id)` 与 daemon tick 去重（约 6372 行附近）。
- `crates/rustcode-cli/src/schedule_cmd.rs`：`tick_once`（兑现分支，约 953–978 行；`run_task_with` 约 967 行）、`run_task_with`（约 641 行加载 task，约 745 行传 `Some(task.id.clone())`）。
- `crates/rustcode-cli/src/main.rs`：`spawn_native_cli_runtime`（签名约 3920 行、构造约 4028 行、交互调用约 3328 行）。
- `crates/rustcode-cli/src/im_runner.rs`：`None` 调用（约 57 行）。
- `crates/rustcode-tuix/src/event_loop/commands.rs`：`LoopArg::Status` 分支（约 3252 行）；`render/mod.rs` `LoopStatus`（约 998 行）；`retained.rs` `loop_row_parts`（约 586 行）、`build_loop_row`（约 3396 行）。
- 交互路径 `CodingRuntimeStart` 构造（均 `task_id: None`）：`daemon/kernel_runtime.rs`、`clix/code.rs`、`cli/acp/engine.rs`。

## 6. 验证矩阵（如实）

| 项 | 状态 |
|---|---|
| `cargo check --workspace`（lib + 全 crate 类型检查） | 已完成，绿，零警告 |
| `cargo check -p rustcode`（cli/daemon 接线） | 已完成，绿 |
| config 测试 `wakeup_survives_process_restart` / `wakeup_consumed_exactly_once` | 已完成，实测 passed（`debug=0` 链接绕 OOM） |
| coding/daemon/clix 测试二进制运行 | 未做，4 GiB OOM 无法链接（仅 check 绿） |
| 端到端“注册 -> 重启 -> tick -> run” | 未做，未实跑 |
| 既有 `wakeup_register_claim_and_consume` 整跑 | 警告，用全局 root，会写真实 home（已避开） |

## 7. 工作树混合警告（提交须知）

`git status` 中 P3 改动已提交（`a3e24012`）；以下**非 P3 改动仍在工作树**，提交时勿夹带：

- **P3 文件（提交目标）**：`schedule.rs`(+314)、`controllers.rs`(+70)、`runtime.rs`(+37)、`schedule_cmd.rs`(+37)、`i18n/{messages,zh_cn,en}.rs`(+9)、`main.rs`(+6)、`acp/engine.rs`/`im_runner.rs`/`clix/code.rs`/`daemon/kernel_runtime.rs`/`tuix/commands.rs`（各 +1）。
- **非 P3（勿夹带）**：`.codebuddy/agents/project-manager.md`(+34)、`capabilities/src/tools/{sensitive_path.rs(+37),task.rs(+17)}`、`scripts/install-freebsd-cross.sh`(+56)，以及未跟踪的 `.codebuddy/agents/*.md`（6 个）、`docs/plans/2026-10-05-rustcode-dev-team-config.md`、`.codebuddy/memory/MEMORY.md`。

## 8. 剩余 / 暂缓项（接手下一步）

1. 已完成 **提交 P3 改动**：`a3e24012`（dev 分支，未推送），13 文件 / 493 增 5 删；非 P3 改动与文档仍在工作树（见第 7 节）。
2. 待办 **批次4 第三项测试** `stale_generation_wakeup_is_dropped_without_side_effect`：需先做 **generation 持久化**（独立增强），否则 `generation` 恒 `None`、钩子 no-op，写了必失败。
3. 待办 **端到端赎回集成测试**：`cargo test -p rustcode ... --config 'profile.dev.package.rustcode.debug=0'` 尝试跑 coding 集成测试验证 `tick_once` 兑现链路；仍 OOM 则需更高内存或 mold。
4. 可选增强 **generation 持久化**：让 `generation` 比对真正生效，防旧 runtime 污染 session。
5. 已知遗留 config 测试用全局 root 污染真实 home——长期应给 config 测试也做 `RUSTCODE_HOME` 重定向或改用 `*_in` 助手。

## 9. 门禁 / 坑（来自实测）

- 项目 `-D warnings`：未使用变量、未覆盖 match 都是错误。**新增 `Msg` 变体须 `messages.rs` + `zh_cn.rs` + `en.rs` 三处同步**（否则穷举 match 编译失败）。
- CLI 包名 `-p rustcode`（非 `rustcode-cli`）。
- **OOM 绕法**：`-j 1 --config 'profile.dev.package.rustcode.debug=0'`；**别用 `RUSTFLAGS`**（会破坏链接）。
- **`cargo check --tests` 须覆盖所有受影响 crate**——曾漏 `rustcode-config` 导致既有测试长期未编译、3 处 `task_id` 缺失未被发现（教训）。
- 子代理只适**只读勘察 / 纯文档**；编译型实现派子代理会 `cargo` 阶段 idle timeout，留“看似完整实则有洞”改动，主进程须复核接线 / `warnings` / 测试编译。

## 10. 如何继续（接手步骤）

1. 读 `docs/plans/2026-09-23-continuous-agent-design.md` 第 5.5 节 + 本文件 + `.codebuddy/memory/MEMORY.md` 的“持续任务 / 调度（schedule）” P3 段。
2. 决定下一步：提交 / generation 增强 / 端到端测试（见第 8 节）。
3. 若提交：按第 7 节精挑 hunk；`git log` 确认 authorship；提交须用户明确同意。
4. 若做 generation：先定 generation 计数器来源（runtime 还是 task），再改比对钩子 + 补 `stale_generation_wakeup_*` 测试。
5. 验证始终用 `cargo check --workspace` 兜底；可运行测试用 `debug=0` 绕 OOM。

## 11. 持久记忆指针

`.codebuddy/memory/MEMORY.md` -> “持续任务 / 调度（schedule）” -> “P3（唤醒持久化…）” 行与 “P3 批次进度” 列表（含 OOM 约束、批次4 教训）。
