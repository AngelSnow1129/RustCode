# RustCode 持续工作能力设计与推进方案

状态: DRAFT (评审中)
日期: 2026-09-23
范围: `rustcode-config` / `rustcode-cli` / `rustcode-daemon` / `rustcode-coding` / `rustcode-tuix`
目标: 让 Agent 能**持续工作** —— 任务持久化、后台执行、定时唤醒

---

## 0. 一句话结论

仓库里已经有三条"持续执行"路径，但**没有任何一条把 `durable task` + `durable wakeup` +
后台进程三者串起来**：

| 现有路径 | 持久性 | 连续性 | 后台执行 | 核心问题 |
|---------|-------|-------|---------|---------|
| OS 调度器 → `rustcode schedule run <id>` | 有（plist/timer/schtasks） | **无** | 有（脱离终端） | 每次触发都**冷启动新进程 + Fresh 新会话**，前后两次运行毫无关联，且只有一个 `last_run_at` 槽位 |
| `/loop` self-paced（`schedule_wakeup`） | **无** | 有 | 无（前台 turn 绑定） | 唤醒定时器是 `tokio::spawn` 持有的进程内计时器，进程退出即丢；且**只能续命、不能启动** |
| `/goal`（自主目标循环） | **无** | 有（最强） | 无 | 完全前台绑定，session fresh / `/cd` / 关闭即终止 |

因此本方案的落点**不是新造一个生命周期 owner**（违反 AGENTS.md「单一状态所有权」），
而是补齐两处**缺失的持久化**，并把调度 tick 挂到**已存在**的 daemon 上：

1. **任务级运行台账（run ledger）** —— 让"任务"成为跨进程、跨重启的持久对象；
2. **可持久化的唤醒登记表（wakeup registry）** —— 让"定时唤醒"从"进程内续命"升级为
   "跨重启可恢复的下一次执行意图"。

---

## 1. 现状盘点（带证据）

### 1.1 已有资产清单

| 资产 | 落点 | 现状 |
|------|------|------|
| 调度任务存储 | `crates/rustcode-config/src/schedule.rs` | `ScheduleTask` + `schedules_root()`（`config_dir/schedules`），单文件 `<id>.json`，`last_run_at` / `last_status` 单槽位 |
| 调度抽象 `Schedule` | 同上 L6-12 | `Daily` / `Weekly` / `Hourly` / `Interval` / `Cron` |
| `next_run()` 纯函数 | 同上 L134-171 | 仅用于**列表展示**；`Cron` 返回 `None`，`Weekly` **忽略 weekday** |
| id 安全守卫 | 同上 L54-63 `valid_id` | 拒绝 `..` / 路径分隔符，防目录穿越（已有测试覆盖） |
| CLI 子命令 | `crates/rustcode-cli/src/schedule_cmd.rs` | `add` / `list` / `remove` / `enable` / `disable` / `run` / `sync` |
| 任务执行器 | 同上 `run_task()` L546-689 | 复用 headless 引导链，见 §1.2 |
| OS 调度器适配 | `crates/rustcode-cli/src/schedule_os.rs` | `OsScheduler` trait（`install`/`uninstall`/`status`）+ `Launchd`(macOS) / `SystemdTimer`(Linux) / Windows schtasks 三个实现 + `CommandRunner` 便于测试 |
| 进程内唤醒工具 | `crates/rustcode-coding/src/controllers.rs` L898-961 `ScheduleWakeupTool` | 工具名 `schedule_wakeup`，`delay_seconds` 夹取到 `[60, 3600]` |
| 唤醒请求载荷 | 同上 L100-105 `WakeupRequest` | `{ delay_seconds, prompt, reason }`；derive 仅 `Clone, Debug, PartialEq, Eq`，**无 `Serialize`/`Deserialize`** |
| 唤醒定时器 | `crates/rustcode-coding/src/runtime.rs` L6372-6382 | turn 结束后 `tokio::spawn` + `select!{ sleep(delay) => tx.send(fire), cancel.cancelled() => {} }` |
| 唤醒事件通道 | 同上 L2745 `loop_fire_tx/rx` | `mpsc::unbounded_channel::<(u64 /*generation*/, u64 /*controller_id*/, WakeupRequest)>`，在 owner `select!` 中以 `if native_protocol` 门控 |
| 自主控制器 | `crates/rustcode-coding/src/controllers.rs` | `LoopState`（前台绑定 + `CancellationToken`）、`GoalState`（含 `max_rounds` / deadline / `no_progress` 熔断） |
| driver 命令面 | `crates/rustcode-coding/src/runtime.rs` L559-562 | `StartGoal` / `StopGoal` / `StartLoop` / `StopLoop` |
| 会话来源标记 | `crates/rustcode-capabilities/src/session/manager.rs` L357-369 | `SessionOrigin::{Manual(default), Scheduled}`；`list_visible()` 把 `Scheduled` 排除出 `/resume` 与 WebUI 侧栏 |
| 编排型 API 设计 | `docs/agent-api-rfc.md` | DRAFT：`POST /agent/tasks` 等，规划新增 `rustcode-agent-api` crate（**尚未实现**） |

### 1.2 `run_task()` 的真实行为（关键事实）

`crates/rustcode-cli/src/schedule_cmd.rs:546-689`，按序：

1. 载入任务记录，`enabled == false` 直接返回 0；
2. 从 `Config::default_path()` 读用户配置（缺失则 `Config::default()`）；
3. `cwd` 不存在 → 写 `last_status = "error"`，返回 1（`tests/schedule_run_exit_code.rs` 覆盖此路径）；
4. `spawn_native_cli_runtime(..., resume_session_id = None, ...)` —— **注释明写 "Fresh session -- no resume"**，即每次触发都是全新会话；
5. 成功后把会话 meta 标为 `SessionOrigin::Scheduled`；
6. `permission_mode == "auto"` 降级为 `AcceptEdits`；
7. 走 `run_native_headless(..., strict_unattended = true)` —— 任何被升级到审批的工具调用一律**拒绝**（`main.rs:3717-3718`：`strict_unattended` 直接 `return false`）；
8. 回写 `last_run_at` / `last_status`。

结论：这是一条**一次性、无状态、无记忆**的批处理路径，不具备"持续工作"所需的连续性。

### 1.3 消费面盘点（谁读调度存储）

```text
crates/rustcode-config/src/schedule.rs          <- 定义与存储
crates/rustcode-cli/src/schedule_cmd.rs         <- CLI 读写
crates/rustcode-cli/src/schedule_os.rs          <- OS 注册
crates/rustcode-tuix/src/event_loop/commands.rs <- /schedule 仅列表（L1660）
crates/rustcode-daemon/src/                      <- 零引用（已实测 grep 为空）
```

**daemon 完全不认识调度存储。** 这意味着：没有常驻进程负责"到期即触发"，
唯一触发者是 OS 调度器冷启动 CLI。这是架构上最关键的一处缺口。

### 1.4 已有周期性任务的先例

`crates/rustcode-daemon/src/lib.rs:5292` 的 `spawn_idle_timeout_task` 已经在用
`tokio::time::interval(Duration::from_secs(60))` 做 60 秒轮询。这证明
**daemon 内挂一个周期性 tick 是本仓既有的、被接受的模式**，可以直接复用同一手法，
无需引入新的运行时机制（如 cron crate 或独立驻留进程）。

### 1.5 各交互面的真实可达性（实测）

`/schedule` 命令**已经存在**，但它只是一个只读列表空壳。逐面实测：

| 交互面 | 是否存在 | 能力 | 证据 |
|--------|---------|------|------|
| TUI `/schedule` | **存在** | **仅只读列表** | `tuix/src/commands.rs:266`（`hidden: false`，在 `/` 菜单与 Tab 补全中）；派发臂 `event_loop/commands.rs:1660` |
| TUI 写操作（add/remove/enable/run） | **无** | 零 | `grep "schedule::save\|schedule::remove" crates/rustcode-tuix/src/` 为空 |
| WebUI | **无** | 零 | `grep -i schedule webui/src/` 仅命中 `Chat.tsx` 的 `scheduleReconnect`（SSE 重连，与调度无关） |
| ACP | **不暴露** | 零 | `commands.rs:266` 的 `acp: false` |
| **Agent 工具** | **无** | 零 | capabilities 的 21 个工具（`bash`/`read_file`/`task`/`todowrite`…）**无任何调度类工具** |

TUI 侧之所以只是空壳，有三个叠加原因：

1. `needs_args: false`（`commands.rs:266`）—— 补全后停在 `/schedule`，不提示任何子命令；
2. handler **完全不读参数**（`event_loop/commands.rs:1660` 起）—— 直接
   `schedule::list()` + `build_schedule_list_text()`，传入的文本被忽略；
3. 命令表里没有 `schedule add/remove/enable/run` 任何形态。

**最关键的一行是 Agent 工具为零**：Agent 在运行中**无法给自己派活**。
"让 Agent 持续工作"目前只能靠人事先在 CLI 排好任务，Agent 没有任何手段表达
"明早 9 点做这个"。

### 1.6 既有设计文档与代码的漂移

| `docs/superpowers/` 下的既有材料 | 与实际代码的关系 |
|-----------------------------------|-----------------|
| `plans/2026-07-31-local-scheduled-tasks-phase1.md` | 已落地（store + CLI + `run_task` + session origin） |
| `plans/2026-07-31-local-scheduled-tasks-phase2.md` | 已落地（`schedule_os.rs` 三平台实现） |
| **两文档均未覆盖** | **任务间依赖、agent 自主派生、事件触发、WebUI 面** —— 本方案 §5.3.5 补齐 |

注：`crates/rustcode-cli/src/schedule_cmd.rs:1-4` 的模块注释仍写着
"Task 4 will fill in the `Run` arm; for now it returns a non-zero exit code … **stub**"，
但 `run_task()`（L546-689）已完整实现且 `ScheduleCli::Run`（L506）已接线。
该注释**已过期**，P0 应顺手修正，避免后来者误判能力边界。


---

## 2. 缺口分析（逐条可验证）

| 编号 | 缺口 | 证据 | 影响 |
|------|------|------|------|
| G1 | **无调度器**：没有任何常驻进程在到期时触发任务 | §1.3 daemon 零引用 | 只能依赖 OS 调度器冷启动；容器/CI/无 systemd 环境完全不可用 |
| G2 | **任务级持久化为空**：只有 `last_run_at` / `last_status` 两个单槽位，无运行历史 | `schedule.rs:29-32` | 无法回答"上次跑了什么、改了什么、为什么失败"；无法做断点续跑 |
| G3 | **唤醒只能续命、不能启动**：`ScheduleWakeupTool` 在 `active == false` 时直接报错 | `controllers.rs:921-928` | 没有活跃 `/loop` 就唤不醒任何东西；无法用唤醒表达"明早 9 点继续做 X" |
| G4 | **唤醒不可跨重启**：定时器是 `tokio::spawn` 持有，且 `WakeupRequest` 无 `Serialize` | `runtime.rs:6372-6382`、`controllers.rs:100-105` | 进程退出/机器重启 → 唤醒意图静默丢失 |
| G5 | **错过补跑语义跨平台不一致且未建模** | `schedule_os.rs:229`（systemd `Persistent=true`）vs 同文件 L67-71 注释（launchd **不会**补跑） | 同一条任务在 Linux 会补跑、macOS 不会，且 `last_run_at` 无法区分"没跑"与"跑了但没记" |
| G6 | **`/loop` 与 `/goal` 是前台绑定** | `LoopState.cancel: CancellationToken`，随 session fresh / `/cd` / drop 取消 | 无法"关掉终端让它继续跑" |
| G7 | **调度显示会漂移**：`next_run()` 对 `Cron` 返回 `None`、对 `Weekly` 忽略 weekday | `schedule.rs:157-169` | `schedule list` 与 TUI `/schedule` 对 cron 任务显示 `-`，但 OS 实际会触发 → **可见的事实错误** |
| G8 | **两套 `Schedule` 语义**：存储里的 `Schedule` 是"模板值"，真实调度权已移交 OS | `schedule_os.rs` 与 `schedule.rs` 各持一份解释逻辑 | 用户手改 systemd timer 后存储与事实不一致，无对账机制 |
| G9 | **执行结果对用户不可见** | `list_visible()` 排除 `Scheduled`，且无运行日志 | 任务"跑了成功了"与"卡住了"对用户同样不可区分 |
| G10 | **任务间无任何关系**：`ScheduleTask` 无依赖/链式/触发源字段 | `grep "depends_on\|after\|chain\|on_success\|next_task" schedule.rs` 为空 | 无法表达"A 成功后再跑 B"；批量任务只能各自独立 |
| G11 | **Agent 无法自主派生任务**：无调度类 agent 工具 | §1.5 工具清单实为零 | Agent 运行中不能给自己派活；"持续工作"必须由人预先排定 |
| G12 | **除时间外无任何触发源**：`Schedule` 枚举纯时间语义 | `schedule.rs:6-12` 五个变体全是时间 | 无法由事件驱动（turn 完成 / 文件变更 / 任务终态） |
| G13 | **`/schedule` 是只读空壳**：TUI 不能增删启停，WebUI 与 ACP 完全没有调度面 | §1.5 全表 | 用户只能回到命令行管理；WebUI 用户无任何入口 |

### 2.1 核心架构判断

把上表收敛成一句：

> **"持久"与"连续"目前分居两端。** OS 调度器给了持久性但丢掉连续性（每次全新会话）；
> `/loop` 给了连续性但丢掉持久性（进程内计时器）。二者之间缺少的是一座桥：
> **一个能把"未完成的工作"和"下一次执行意图"写进磁盘、并在重启后可恢复的持久层。**

这座桥就是本方案两个新组件的职责，且两者都**不需要**新的生命周期 owner。

### 2.2 "相互调度"为何在当前架构下不可能（根因，非疏漏）

G10–G12 三条缺口有同一个根因，写在本仓自己的既有设计文档里
（`docs/superpowers/specs/2026-07-31-local-scheduled-tasks-phase2-design.md`）：

> 阶段 2 = OS 调度器自动注册（"自动到点"）……把触发**外包给**
> launchd / Task Scheduler / systemd-timer

**OS 调度器的模型是「时间 → 拉起一个进程」，它是无状态的。** 它不知道 A 是否跑成功，
也无法在 A 成功后**立即**拉起 B。因此在该架构下：

- 任务之间**无法有边** —— 没有任何组件持有能承载"边"的状态机；
- "相互调度"**无处安放** —— 这是把调度权交出去之后的必然结果，不是遗漏。

这恰好印证 §2.1：要恢复任务间的关系，**必须先有常驻调度器**（P2）**和运行台账**（P1），
否则后继任务无从得知前驱的终态。故相互调度在方案中排在 P2 之后（见 §5.3.5）。


---

## 3. 目标架构

### 3.1 组件图

```text
触发源（四类，最终都落到同一个执行入口）
  1. OS 调度器            2. daemon tick         3. 进程内唤醒        4. 事件
     launchd / systemd /     rustcode-daemon        run 的 turn 结束     任务终态 / turn
     schtasks 冷启动 CLI     周期轮询（复用既有     后 self-paced 续跑   完成 / 文件变更
     `schedule run`          spawn_idle_timeout_                        (P2.5 新增)
             |                task 同款 interval)         |                   |
             |                    |                       |                   |
             +----------+---------+                       |                   |
                        v                                 |                   |
        +-----------------------------------+            |                   |
        |  ScheduleRunner（新，capabilities）|<-----------+-------------------+
        |  - 触发判定 trigger_fired()        |   同一 run 生命周期
        |  - DAG 解析（前驱终态 -> 后继 due） |   <- P2.5
        |  - 单飞锁（防重入 / 防双触发）      |
        |  - 环检测（加载期拒绝非法图）       |
        |  - 装配一次 CodingRuntime          |
        |  - 记录 RunRecord                  |
        +-----------------+-----------------+
                          | 复用既有链路，不新建 owner
                          v
              spawn_native_cli_runtime / CodingRuntime
                          |
                          v
              rustcode-kernel Agent（零改动）
                          ^
                          | agent 调用（与 schedule_wakeup 对称）
        +-----------------------------------+
        |  schedule_task 工具（P2.5 新增）    |  <- capabilities；无活跃 loop 也可用
        |  - Agent 自主派生任务               |     补 G3「唤醒只能续命不能启动」
        +-----------------------------------+
        +-----------------------------------+
        |  WakeupRegistry（新，config 层）   |  <- 与 ScheduleTask 同存储根
        |  - 持久化下一次执行意图             |
        |  - 进程启动时可恢复                 |
        +-----------------------------------+
```

**四类触发源的分工（P2.5 后）**：

| 触发源 | 模型 | 延迟 | 仅用 OS 调度器时可用 |
|--------|------|------|--------------------------|
| 时间（OS 调度器） | 无状态，进程被拉起 | 分钟级 | 是 |
| 时间（daemon tick） | 有状态，轮询到期 | `tick_interval_secs` | **否**（需 daemon 常驻） |
| 唤醒（self-paced） | 有状态，进程内计时器 + registry | 秒级 | 否 |
| **事件（P2.5）** | 有状态，仅在 daemon/tick 内 | **近实时** | **否**（OS 调度器无法表达事件） |

这张表是 §2.2 结论的直接推论：**除第一行外，其余三类都要求常驻进程。**
这是为什么 P2（daemon tick）是 P2.5 的硬前置，而非可选优化。

### 3.2 分层归属（严格按 AGENTS.md 依赖方向）

| 新组件 | 落点 crate | 层 | 理由 |
|--------|-----------|----|------|
| `RunRecord` / `RunLedger` / `WakeupRegistry` | `rustcode-config` | leaf | 纯数据 + 文件存储，与既有 `schedule.rs` 同性质同目录；leaf 可被所有上层依赖 |
| `depends_on` / `triggers` 字段 + `Trigger` 枚举 + `validate_graph` | `rustcode-config` | leaf | 纯数据 + 校验，无运行时依赖 |
| `ScheduleRunner`（due 判定 + 单飞 + 生命周期） | `rustcode-capabilities` | L1 | 与 `session/manager.rs`（唯一会话持久化模型）同层；`cli` 与 `daemon` 都能依赖 |
| `schedule/graph.rs`（DAG 求值 + 环检测） | `rustcode-capabilities` | L1 | 纯算法，与 `ScheduleRunner` 同 crate 便于共享内部表示 |
| `schedule_task` agent 工具 | `rustcode-capabilities` | L1 | 与既有工具族同层（`tools/*.rs`）；只依赖 config 的 store API |
| 冷启动驱动（`schedule run` / `schedule tick`） | `rustcode-cli` | L3 | driver 职责：输入/流程编排 |
| 常驻 tick + 事件源订阅 | `rustcode-daemon` | L3 | 已依赖 capabilities，且已有 60s interval 先例 |
| `/schedule` 子命令交互（读写） | `rustcode-tuix` | L3 | driver 职责；**不得横向依赖 cli**（见 §5.4.4） |
| WebUI 调度只读路由 | `rustcode-daemon` | L3 | WebUI 后端本就属 daemon |
| 唤醒恢复接线 | `rustcode-coding` | L2 | `WakeupRequest` 的序列化与「重启后恢复」语义属运行时领域 |


**硬约束（不得违反）**：

- `rustcode-kernel` **零改动** —— 它不得承载任何调度语义（L0 零文案、零产品语义的不变量）。
- 不新增第二套 `CodingRuntime` 生命周期 owner：`ScheduleRunner` 只**调用**既有引导链，
  自己不持有 agent / 事件通道的所有权（对齐 `docs/agent-api-rfc.md` §1.3 第 4 条）。
- 不引入新 crate 作为过渡；`docs/agent-api-rfc.md` 的 `rustcode-agent-api` 是**后续**的
  远程编程式 API 面，本方案是其**前置地基**（它同样需要任务级持久化），不与之冲突。

---

## 4. 数据模型（新增，全部 additive）

### 4.1 运行台账 `RunRecord`

落点 `rustcode-config/src/schedule.rs`（或拆出 `schedule/run.rs`，见 §5.1）：

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunRecord {
    pub run_id: String,          // uuid simple
    pub task_id: String,
    pub started_at: i64,         // epoch secs
    pub finished_at: Option<i64>,// None = 仍在跑
    pub exit_code: Option<i32>,
    pub status: RunStatus,       // Running | Ok | Error | Cancelled | Skipped
    pub session_id: Option<String>, // 复用，用于「继续上次的工作」
    pub trigger: RunTrigger,     // Os | DaemonTick | Manual | Wakeup | Event
    pub summary: Option<String>, // 终态摘要（有界长度，见 §6 风险）
    /// 因前驱失败/跳过而 Skipped 时，记录被阻断的直接前驱（可审计、可定位断链）。
    #[serde(default)]
    pub skipped_because_of: Option<String>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RunStatus { Running, Ok, Error, Cancelled, Skipped }

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RunTrigger { Os, DaemonTick, Manual, Wakeup, Event }
```

### 4.1.1 DAG 与触发源字段（P2.5，全部 `#[serde(default)]`）

```rust
// 加在既有 ScheduleTask 上（不新增结构体，避免第二套定义）
pub struct ScheduleTask {
    // ... 既有 11 个字段保持不变 ...
    /// 前驱任务 id。全部为 Ok 时才允许本任务运行（fail-fast，见 §5.4.1）。
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// 除时间外的额外触发源。空 = 纯时间触发（等价当前行为）。
    #[serde(default)]
    pub triggers: Vec<Trigger>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Trigger {
    /// 显式时间触发（默认语义，与既有 Schedule 等价）。
    Time,
    /// 某个任务终态后触发。与 depends_on 归一到同一内部表示（§5.4.3）。
    AfterTask { id: String },
    /// 本进程内一次 turn 完成后触发。
    OnTurnComplete,
    /// 文件变更触发（P2.5 可选子项；未实现时加载期报错，不静默不触发）。
    OnFileChange { glob: String },
}
```

**两个必须守住的约束**：

1. `triggers` 为空 `Vec` 时语义**必须**等价于 `[Trigger::Time]` —— 否则既有任务
   （文件里没有该字段）在 P2.5 后行为会变，违反「additive」承诺。
2. `Trigger::AfterTask { id }` 与 `depends_on` 表达同一件事，**加载时归一**：
   `AfterTask` 被合并进内部依赖图，`depends_on` 保持为面向用户的声明式字段。
   禁止让两条路径各维护一份邻接表（会产生第二套真相）。


存储布局（与既有 `<id>.json` 平级，不破坏现有读取）：

```text
$RUSTCODE_HOME/schedules/
|-- <task-id>.json                     # 既有：任务定义（保持不变）
`-- <task-id>/                          # 新增：普通目录，被既有 list_in() 跳过
    |-- runs/
    |   |-- <run-id>.json               # 每次运行的记录
    |   `-- ...
    `-- last_run.json                   # 指向最近一次（避免 list 时全量扫）
```

**兼容性保证**：既有读取路径 `list_in()`（`schedule.rs:89-107`）遍历目录时只接受
`extension == "json"` 的**文件**，新建的 `<task-id>/` 是**目录**，会被 `p.extension()` 判定跳过。
因此新增目录布局**不会**让既有 `list()` 误读，无需迁移既有数据。

### 4.2 唤醒登记表 `WakeupRegistry`

落点 `rustcode-config/src/schedule.rs`：

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduledWakeup {
    pub id: String,              // 目录安全 id，复用 valid_id 守卫
    pub due_at: i64,             // epoch secs；绝对时间，而非 delay
    pub prompt: String,
    pub reason: String,
    pub created_at: i64,
    pub generation: Option<u64>, // 绑定创建它的 runtime generation（见 §6 风险）
    pub consumed: bool,          // 单次消费；消费后保留用于审计
}
```

**关键设计决策：存绝对时间 `due_at`，而非相对 `delay_seconds`。**

理由是 `WakeupRequest` 现有语义（相对 delay）在"进程外"没有意义：进程在
sleep 期间退出时，`delay_seconds` 无法回答"还剩多久"。落盘时立刻把
`now + delay_seconds` 折算为绝对时间戳，恢复时用 `due_at` 与当前时间比较即可。

`WakeupRequest` 需补 `Serialize, Deserialize`（`controllers.rs:100-105`，
当前 derive 列表里没有），这是**唯一**对 L2 现有类型的改动，且是纯 additive derive。

### 4.3 配置面

`rustcode-config` 新增 `[schedule]` 表（对齐仓库既有 `[subagent]` 表的做法）：

| 键 | 默认 | 环境变量 | 语义 |
|----|------|---------|------|
| `enabled` | `false` | `RUSTCODE_SCHEDULE_ENABLED` | 是否启用**任何**调度触发（含 daemon tick）。默认关闭 = 保持当前行为，零回归风险 |
| `daemon_tick` | `false` | `RUSTCODE_SCHEDULE_DAEMON_TICK` | daemon 内是否跑 tick（与 OS 调度器**互斥**，见 §6） |
| `tick_interval_secs` | `60` | `RUSTCODE_SCHEDULE_TICK_SECS` | 轮询周期，夹取到 `[10, 3600]` |
| `catch_up_window_secs` | `3600` | `RUSTCODE_SCHEDULE_CATCHUP_SECS` | 补跑容忍窗口；超过则记 `Skipped` 而非补跑（对齐 G5） |
| `max_run_history` | `50` | `RUSTCODE_SCHEDULE_HISTORY` | 每任务保留的 `RunRecord` 上限，超出按时间淘汰 |

环境变量名必须落在 `rustcode-config/src/endpoints.rs`（事实源），
**不得**在 `config` 里另写字面量——这是仓库既有约定（对齐 `WEBUI_NO_AUTH_ENV` 的做法）。

---

## 5. 分期实施

分五期（P0 / P1 / P2 / P2.5 / P3），每期**可独立合并、可独立回滚**，
且每期结束时门禁全绿（G1–G5）。期与期之间不存在必须同时上线的耦合，
唯一的硬前置是 **P2.5 依赖 P1 + P2**（DAG 需要前驱终态，事件触发需要常驻进程）。

### 5.1 P0 —— 修可见的事实错误（最小改动，独立价值）

**目标**：消除 §2 的 G7。这是纯粹的正确性修复，不引入新概念。

| 文件 | 改动 |
|------|------|
| `crates/rustcode-config/src/schedule.rs` | `next_run()` 增加 `Schedule::Cron` 的真实求值；`Weekly` 按 `weekday` 对齐（当前 L157-169 忽略它） |
| 同上 | 单位测试：`next_run_cron_is_none_in_phase1` **必须改写**——它是"承认缺陷"的测试，P0 后应断言真实值 |

**决策（已定，D1）**：cron 求值**手写五段最小子集，不引入 `cron` crate**。
需正确覆盖 `*`、`a-b`、`a,b`、`*/n` 四类，限 `分 时 日 月 周` 五段，
不支持 `@daily` 等宏。

**验收判据**：

- `next_run(Cron{"0 9 * * 1-5"}, now)` 返回下一个工作日 09:00，且对同一 `now` 幂等；
- `next_run(Weekly{weekday:1, time:"09:00"}, now)` 落在**下周一**而非"下一个 09:00"；
- `cargo test -p rustcode-config --lib` 全绿，且 `schedule list` 对 cron 任务不再显示 `-`。

**回滚**：纯函数级改动，`git revert` 单 commit 即可。

---

### 5.2 P1 —— 任务运行台账（任务持久化）（已落地 2026-09-25，见本节末尾）

**目标**：消除 G2、G8、G9 的存储侧。**本阶段不新增任何触发路径**，只让既有
OS 调度器路径产出可审计的记录。

| 文件 | 改动 |
|------|------|
| `crates/rustcode-config/src/schedule.rs` | 新增 `RunRecord` / `RunStatus` / `RunTrigger`；新增 `<task-id>/runs/` 读写（`save_run` / `list_runs` / `latest_run` / `prune_runs`）；`ScheduleTask` 增加 `#[serde(default)]` 的 `last_run_id: Option<String>` |
| 同上 | 测试：目录布局不被既有 `list_in()` 误读（**关键回归测试**，见 §4.1 兼容性说明） |
| `crates/rustcode-cli/src/schedule_cmd.rs` | `run_task()` 在 L605 启动前写入 `RunRecord{status: Running}`，在 L679-686 回写处补 `finished_at` / `exit_code` / `session_id` / `summary` |
| 同上 | 新增 `ScheduleCli::History { id, limit }` + `handle_history_with`（`schedule history <id>`） |
| `crates/rustcode-cli/src/main.rs` | L556 起的 `mut_subcommand("schedule", ...)` 增加 `history` 的 i18n about |
| `crates/rustcode-config/src/i18n/{messages,en,zh_cn}.rs` | 新增 `CliSched*` 变体（`CliAboutScheduleHistory`、`CliSchedHistoryRow`、`CliSchedHistoryEmpty` 等），en/zh **双语 arm 必须齐备**（编译器强制 parity） |
| `crates/rustcode-tuix/src/event_loop/commands.rs` | L1660 的 `/schedule` 输出追加每任务最近一次运行的状态与耗时 |

**关键不变量**：

- 写 `RunRecord` **不得**让 `run_task` 的退出码语义变化 ——
  `tests/schedule_run_exit_code.rs` 必须仍绿（OS 调度器靠退出码判失败）。
- 运行记录写入是 best-effort（对齐 L590 / L686 既有 `let _ = schedule::save(&task);` 风格）：
  记录失败不得让任务本身失败。

**验收判据**：

- 连续跑同一任务两次 → `list_runs` 长度 2，`started_at` 递增，两者 `run_id` 不同；
- 任务失败（cwd 缺失）→ 该 run 的 `status == Error`、`exit_code == 1`，
  且 `schedule run` 进程退出码仍为 1；
- `max_run_history = 3` 时跑 5 次 → 磁盘只剩 3 个 `run-id.json`（淘汰最旧）；
- `cargo test -p rustcode-config --lib` + `cargo test -p rustcode --lib` 全绿。

**回滚**：新增字段均为 `#[serde(default)]`，旧任务 JSON 无字段仍可解析；
删除新目录即回到 P0 状态。

**落地记录（2026-09-25，与设计的偏差如实登记）**：

- 全部验收判据达成：roundtrip/两次运行记录递增、失败 run `status=Error`+`exit_code=1`
  且进程退出码仍 1（`schedule_run_exit_code.rs` 绿）、prune 保留最新 N、
  config `--lib` **371/0** + cli `--bins` **130/0** + tuix `--lib` **2029/0**。
- **偏差 1**：`max_run_history` **未做成 config 键**——实测 `[schedule]` 配置段不存在，
  P1 按最小面用常量 `DEFAULT_MAX_RUN_HISTORY = 20`；`prune_runs(keep)` 签名已参数化，
  P2 引入 `[schedule]` 段时直接接线即可。
- **偏差 2**：`run_id` 用 `<secs>-<nanos九位>`（config crate 无 uuid 依赖，不为其新增）；
  字典序即时间序，比 uuid 更利于排序，连续运行必不同。
- **偏差 3**：OS 调度器与手动 `schedule run` 共用同一命令行，`trigger` 一律记
  `Manual`（`OsScheduler`/`Daemon` 变体留给 P2）；已在 AGENTS.md 登记为残留。
- **偏差 4**：TUI `/schedule` 的摘要行放 `ScheduleLastRun`（状态+耗时）而非完整行，
  避免与既有 `ScheduleRow` 列宽耦合。

---

### 5.3 P2 —— daemon 常驻调度（后台执行）

**目标**：消除 G1、G5。让"到期即触发"不再依赖 OS 调度器。

| 文件 | 改动 |
|------|------|
| `crates/rustcode-capabilities/src/schedule/mod.rs`（新模块） | `ScheduleRunner`：`due(now) -> Vec<ScheduleTask>`（纯函数，可测）、`try_claim(task_id) -> Option<RunGuard>`（单飞锁）、`RunGuard` drop 时释放 |
| 同上 | 单飞锁用 `<task-id>/.lock` 文件 + `flock`（对齐 `session/manager.rs` 已有的 lock 文件先例：`locks/` 目录下持 inode 锁） |
| `crates/rustcode-daemon/src/lib.rs` | 新增 `spawn_schedule_tick(...)`，**完全照抄** L5286-5305 `spawn_idle_timeout_task` 的 `interval` 骨架；受 `[schedule].daemon_tick` 门控，默认 `false` |
| `crates/rustcode-daemon/src/main.rs` | 启动时读 config，`daemon_tick == true` 才 spawn tick；否则打印一行 `[INFO] schedule tick disabled` |
| `crates/rustcode-cli/src/schedule_cmd.rs` | 新增 `ScheduleCli::Tick { once }`（`--once` 供外部 cron 调用，无 `--once` 则前台常驻），复用同一 `ScheduleRunner` |

**互斥规则（必须实现，否则会双触发）**：

> `[schedule].daemon_tick == true` 时，`schedule add` 打印显式警告：
> 该任务同时已注册到 OS 调度器，两侧都会触发。
> 提供 `schedule sync --unregister-os` 一键撤销 OS 注册，切换到 daemon tick 模式。

判定依据落在 storage 层（读注册状态），不是配置层的猜测 —— 对齐仓库
「单一事实源」倾向（参见 `webui_no_auth` 的 `webui_no_auth_masks_access_key` 先例：
用一个显式谓词统一判定，禁止各调用点自行推断）。

**验收判据**：

- 单飞锁：并发两次 `try_claim(同一 id)` → 只有一次拿到 guard，另一次返回 `None`；
- 补跑语义：把任务 `due_at` 设为 2 小时前、`catch_up_window_secs = 3600`
  → `Tick` **不**触发该任务，反而写入 `RunRecord{status: Skipped}`；
  设为 30 分钟前则正常触发（对齐 G5，且**跨平台行为一致**）；
- `daemon_tick = false`（默认）时，daemon 进程的 CPU 占用与当前**无差异**（可用
  `pgrep` + 前后 `getrusage` 对比，或在测试中断言 tick 任务未被 spawn）；
- `cargo test -p rustcode-capabilities --lib` + `-p rustcode-daemon --lib` 全绿。

**回滚**：`daemon_tick` 默认 `false`，未显式开启时**零行为变化**；
删除 `spawn_schedule_tick` 调用点即回滚。

**落地记录（2026-09-27，与设计的偏差如实登记）**：

- 全部验收判据达成：单飞锁互斥（`try_claim_is_exclusive_and_released_on_drop`）、
  补跑超窗记 `Skipped` 且前移锚点（`catch_up_outside_window_records_skipped_and_advances_the_anchor`）、
  `daemon_tick` 关闭时零行为（`schedule_config_defaults_keep_every_trigger_off`）、
  陈旧 `Running` 回收（`stale_running_run_is_reaped_only_when_the_lock_is_free`）。
  门禁：config `--lib` **383/0**、cli `--bins` **131/0**、daemon `--lib` **320/0**、
  `cargo fmt --check` 三 crate 干净、clippy 对改动文件 0 新增命中。
- **偏差 1（架构，重要）**：设计 §3.1/§3.2 把 `ScheduleRunner` 放在
  `rustcode-capabilities`（L1）并"装配一次 CodingRuntime"。**这在本仓依赖方向下不可能**：
  `CodingRuntime` 属 `rustcode-coding`（L2），L1 不得依赖 L2。实际落法改为两段：
  (1) **due / 单飞锁 / 补跑窗口 / 陈旧回收**全部下沉到 `rustcode-config/src/schedule.rs`
  （leaf，cli 与 daemon 都能用，且与 `RunRecord` 存储同址，避免第二份真相）；
  (2) **执行**留在 CLI（`spawn_native_cli_runtime` / `run_native_headless` 本就在 bin 侧），
  daemon tick 通过 `schedule tick --once` 子进程复用同一入口。这同时满足
  "不新建第二套 CodingRuntime owner" 与"不制造 L3↔L3 依赖"。
- **偏差 2**：设计 §4.1 的 `skipped_because_of` 字段未新增，超窗原因写进既有的
  `summary`（`CATCH_UP_MISSED_SUMMARY`），使 `RunRecord` 的加性面为零。
- **偏差 3**：设计 §5.3 把 `[schedule]` 段写成 `enabled` 门控 + `daemon_tick` 开关；
  实现额外用 `skip_serializing_if` 让**未触碰**的 `[schedule]` 不写回用户 config.toml
  （否则每次保存都给所有用户的文件加一段无意义内容）。配置表已在
  `docs/config.example.toml` 与 `AGENTS.md` 登记。
- **偏差 4**：设计要求的 `schedule add` 显式互斥警告与
  `schedule sync --unregister-os` 均已实现；后者刻意与 `handle_sync_with` 分离，
  因为 reconcile 会把刚卸下的任务立即重装。
- **未做（属 P2.5/P3 范围）**：DAG / `schedule_task` 工具 / 事件触发（P2.5）、
  可持久化唤醒（P3）。`OnFileChange` 的 O1 裁决（轮询 stat）仍待 P2.5 落地。

---

### 5.4 P2.5 —— 相互调度（静态 DAG + Agent 自主派生 + 事件触发）

**目标**：消除 G10、G11、G12、G13。**硬前置：P1（需终态）+ P2（需常驻触发）。**
排在此处而非并入 P2 的理由：P2 只做"时间 → 触发"，本期的 DAG 依赖与事件触发
都要求 P2 已提供"近实时触发"能力（§3.1 表结论）；合并会让 P2 无法独立回滚。

已裁决语义（用户裁决，2026-09-23）：

- **DAG 由人事先定义**（静态图），不做运行期动态拓扑；
- **失败传播 = fail-fast**：前驱 `Error`/`Cancelled`/`Skipped` → 后继**不跑**，记 `Skipped`；
- 事件触发除时间外，由 **turn 完成 / 任务终态 / 文件变更** 驱动。

#### 5.4.1 静态 DAG 依赖

| 文件 | 改动 |
|------|------|
| `crates/rustcode-config/src/schedule.rs` | `ScheduleTask` 增加 `#[serde(default)] pub depends_on: Vec<String>`；新增 `pub fn validate_graph(tasks: &[ScheduleTask]) -> Result<(), GraphError>` |
| 同上 | 新增 `Trigger` 枚举（见 5.4.3），`ScheduleTask` 增加 `#[serde(default)] pub triggers: Vec<Trigger>` |
| `crates/rustcode-capabilities/src/schedule/graph.rs`（新） | DAG 求值：`ready_successors(finished_task, run_status) -> Vec<String>`；**拓扑排序 + 环检测**（DFS 三色标记，检出环即返回环上节点用于报错） |

**环检测与非法引用必须在加载期拒绝，不得静默降级**：

- 环 → `load` 返回错误，`schedule list` 标红该任务，**拒绝触发**；
- `depends_on` 指向不存在的任务 id → 同上（悬空边，fail-closed）；
- 自我依赖（`A depends_on A`）→ 视同环。

这三点是对齐仓库「显式失败、禁止静默 fresh」的既有不变量
（AGENTS.md「Runtime 生命周期不变量」第 3 条）。

**fail-fast 传播规则（实现为单一谓词，禁止各调用点各写一份）**：

```text
successor_should_run(pred_status) =
    match pred_status {
        Ok                      => Run,
        Error | Cancelled       => Skip("predecessor failed"),
        Skipped                 => Skip("predecessor skipped"),   // 传染：跳过会沿链传下去
        Running                 => Wait,                          // 未终态，本轮不评估
    }
```

`Skipped` **必须沿链传染**，否则 A→B→C 中 A 失败时 B 记 `Skipped` 而 C 会照跑，
产生一个"孤立的成功"，掩盖真实断链。

#### 5.4.2 Agent 自主派生（`schedule_task` 工具）

| 文件 | 改动 |
|------|------|
| `crates/rustcode-capabilities/src/tools/schedule_task.rs`（新） | `ScheduleTaskTool`，与既有 `schedule_wakeup`（`controllers.rs:898-961`）**对称命名与形状** |
| `crates/rustcode-coding/src/parts.rs` | 装配该工具（对齐 `ScheduleWakeupTool` 在 `runtime.rs:1985` / `:7351` 的注册方式） |

工具签名（与 `schedule_wakeup` 的 `{delay_seconds, reason, prompt}` 对齐，但为**绝对时间 + 可重复**）：

```json
{
  "title": "nightly-dep-audit",
  "prompt": "审计依赖并生成报告",
  "cwd": "/abs/path",
  "when": { "kind": "daily", "time": "09:00" },   // 复用既有 Schedule 枚举
  "depends_on": ["prepare-deps"],                  // 可选
  "permission_mode": "plan"                        // 默认 plan，与 CLI 一致
}
```

**关键设计差异（必须说清，否则会被误实现）**：

| | `schedule_wakeup`（既有） | `schedule_task`（新增） |
|---|---|---|
| 语义 | **续命**当前 loop | **派活**给未来 |
| 参数 | 相对 `delay_seconds` | 绝对时间（复用 `Schedule` 枚举） |
| 前置 | 仅活跃 `/loop` 内合法（`active` 守卫 `controllers.rs:921-928`） | **无前置**，任何 turn 可用 |
| 落点 | 进程内 `mpsc` → `loop_fire_tx` | **直写** `~/.rustcode/schedules/` |

**安全边界（不得放宽）**：

- 该工具**必须**受审批门控：写盘任务会在未来无人值守执行，等价于"给自己排了个后门"。
  实现为：默认 `permission_mode: "plan"`，且**不允许** agent 传 `"auto"`
  （尝试传入 → 降级为 `accept_edits` 并记警告），对齐 `run_task()` L634-638 既有的
  「scheduled 的 auto 降级为 AcceptEdits」策略；
- 该工具**不注册 OS 调度器**（只写 store）：由 tick 负责到期触发。
  这是刻意的——避免 agent 在无人监督下写 launchd/systemd 条目。
- `cwd` 必须落在当前 runtime 的 `working_dir` 之内，越界即拒绝
  （对齐既有 `BashWorkspaceGate` 的越界语义）。

#### 5.4.3 事件触发

| 文件 | 改动 |
|------|------|
| `crates/rustcode-config/src/schedule.rs` | `Trigger` 枚举：`Time`（默认，等价现状）/ `AfterTask { id }` / `OnTurnComplete` / `OnFileChange { glob }` |
| `crates/rustcode-capabilities/src/schedule/mod.rs` | `trigger_fired(trigger, ctx) -> bool` 单一判定谓词 |
| `crates/rustcode-daemon/src/lib.rs` | tick 内除时间判定外，追加事件源订阅（见下） |

**三类事件源与成本（诚实标注，不夸大）**：

| 事件 | 可用信号 | 成本 | 建议 |
|------|---------|------|------|
| 任务终态 | `RunRecord.status` 落盘（P1） | 零 —— tick 已在轮询 | **本期实现** |
| turn 完成 | daemon `live_hub` 已有 `TurnFinished`（`live_api.rs:1049`） | 低 —— 复用既有事件流 | **本期实现** |
| 文件变更 | **无现成 watcher** | 高 —— 需引入 `notify` 类依赖或轮询 stat | **降级为轮询 stat**（对齐 fork 最小化依赖倾向）；`OnFileChange` 标记为 P2.5 内的**可选子项**，未实现时该 trigger 加载期报错而非静默不触发 |

`AfterTask { id }` 本质上就是 5.4.1 的一条边，实现上可复用同一求值路径
（`Trigger::AfterTask` 与 `depends_on` 归一到同一个内部表示，避免第二套真相）。

#### 5.4.4 交互面补齐（G13）

| 文件 | 改动 |
|------|------|
| `crates/rustcode-tuix/src/commands.rs` | L266 `needs_args: false` → `true`，desc 补子命令（与 `/webui` 的 L174 同款写法） |
| `crates/rustcode-tuix/src/event_loop/commands.rs` | L1660 起 handler 开始**读参数**：`add` / `remove` / `enable` / `disable` / `run` / `history`，复用 CLI 的 `handle_*_with` 层（不可复制粘贴逻辑） |
| `crates/rustcode-daemon/src/webui.rs` | 新增 `/schedule` 只读路由（列表 + 最近运行），WebUI 侧最小可用面 |

**分层约束**：TUI 不得直接调用 `crates/rustcode-cli/src/schedule_cmd.rs`（L3 之间不能横向依赖）。
需要把 `handle_add_with` / `handle_remove_with` 等**与 `OsScheduler` 解耦的纯逻辑**
下沉到 `rustcode-capabilities/src/schedule/mod.rs`，CLI 与 TUI 各自薄封装。
这是本期最容易做错的地方 —— 直接把 CLI 函数 `pub` 出来会制造 L3 ↔ L3 依赖，违反 §3.2。

**验收判据**：

- A→B（`B.depends_on = [A]`）：A 成功后 tick 立即将 B 转 due 并跑；A 失败 → B 记
  `Skipped{reason: "predecessor failed"}`，且 **B 的 `RunRecord` 存在**（可审计）；
- A→B→C 且 A 失败 → B、C 均 `Skipped`（传染），无一执行；
- 环图 `A→B→A` 加载即报错，`schedule list` 标红，且**不触发任何任务**；
- `schedule_task` 工具：在**无** `/loop` 的普通 turn 中调用成功落盘；
  尝试传 `"auto"` → 实际写入 `accept_edits` 并有警告；
  `cwd` 越界 → 拒绝且**不写盘**；
- 事件触发：同一 daemon 进程内完成一次 turn → `OnTurnComplete` 任务在下一个 tick 内转 due；
- TUI `/schedule add <title> ...` 能建任务，`/schedule history <id>` 能看台账；
- `cargo test -p rustcode-config --lib` + `-p rustcode-capabilities --lib` + `-p rustcode-tuix --lib` 全绿。

**回滚**：`depends_on` / `triggers` 均为 `#[serde(default)]` → 旧任务文件照常解析，
回滚后行为等价于"无依赖、纯时间"；`schedule_task` 工具从 `parts.rs` 装配点摘除即可；
TUI 交互面回退为只读列表。

---

### 5.5 P3 —— 可持久化唤醒（定时唤醒）

**目标**：消除 G3、G4、G6。这是三项能力中**最难、风险最高**的一期，
必须在 P1/P2 落地且有真实使用数据后再开始。

| 文件 | 改动 |
|------|------|
| `crates/rustcode-config/src/schedule.rs` | 新增 `ScheduledWakeup` + `WakeupRegistry`（`register` / `claim_due` / `consume` / `prune`），复用 `valid_id` 守卫 |
| `crates/rustcode-coding/src/controllers.rs` | L100-105 `WakeupRequest` 补 `Serialize, Deserialize`（**唯一** L2 类型改动，纯 additive）；`ScheduleWakeupTool::execute` 在发送成功后**同时**落盘一条 `ScheduledWakeup`（`due_at = now + delay_seconds`） |
| `crates/rustcode-coding/src/runtime.rs` | L6372-6382 的 `tokio::spawn` 定时器保留（快路径不变），但 wakeup 兑现后把对应 registry 条目标 `consumed`；进程启动路径增加"恢复未兑现且已到期/未到期的 wakeup" |
| `crates/rustcode-cli/src/schedule_cmd.rs` | `Tick` 增加 wakeup 兑现分支：到期的 `ScheduledWakeup` 转成一次任务运行（`RunTrigger::Wakeup`） |
| `crates/rustcode-tuix/src/event_loop/commands.rs` | `/loop status` 追加"已登记的持久唤醒"一行（含 `due_at` 本地时间） |

**语义决策（已定，D3）**：跨重启恢复时，到期的 wakeup **立即跑**，
但受 `catch_up_window_secs` 约束：窗口内的立即跑，窗口外记 `Skipped`。
与 P2 的补跑语义**同一个谓词**，不产生第二套真相。

**关键不变量（必须守住）**：

- 恢复的 wakeup **不得**复活已终止的 controller：`WakeupRequest` 里的 `generation`
  必须与当前 runtime generation 比对，不匹配则丢弃（对齐 AGENTS.md
  「旧 generation 迟到事件不得污染 replacement runtime」不变量）。
- wakeup 在 `native_protocol == false` 时**不生效**（对齐 L3299 `loop_fire_rx` 既有的
  `if native_protocol` 门控），否则会破坏只读/测试路径。
- 无活跃 `/loop` 时的独立唤醒：`ScheduleWakeupTool` 当前 `active == false` 直接报错
  （L921-928）。P3 若要让"无 loop 也能唤起"，**必须**改成落盘 + 由 tick 兑现，
  而**不是**放宽这个守卫 —— 放宽会破坏"工具只在 loop 内合法"的模型契约。

**验收判据**：

- 在一个 `/loop` 里调用 `schedule_wakeup`，**杀掉进程**再启动 →
  registry 中该条目仍在，且在 `due_at` 到达后由 tick 兑现为一次运行；
- 兑现有且仅有一次（`consumed` 标记 + 单飞锁双重保证，并发 tick 不重复兑现）；
- generation 不匹配的 wakeup 被丢弃且**不产生**任何运行记录（无副作用）；
- `cargo test -p rustcode-coding --lib` + `-p rustcode-tuix --lib` 全绿，
  且既有 `native_protocol` 门控测试不受影响。

**回滚**：新增 `WakeupRegistry` 目录可整体删除；`WakeupRequest` 的 derive
是纯 additive，回滚时保留也无害（不影响任何既有行为）。

---

## 6. 风险与失败语义

| 风险 | 严重度 | 缓解 | 失败时的语义 |
|------|-------|------|-------------|
| **双触发**：OS 调度器与 daemon tick 同时触发同一任务 | 高 | 单飞锁（flock）+ `daemon_tick` 默认 `false` + `schedule add` 显式互斥警告 | 第二个触发者拿不到 guard → 记 `Skipped` 而**不是**并发跑两次 |
| **长时间运行的任务被重复触发** | 高 | 锁在 run 期间持有；tick 只跳过，不排队 | 不重入；下一次到期正常触发 |
| **`summary` 无限增长撑爆磁盘** | 中 | `RunRecord.summary` 截断到固定长度（如 2 KiB）；`max_run_history` 淘汰 | 截断而非丢弃整条记录，保留可诊断性 |
| **daemon tick 与 live chat 争抢 provider 配额** | 中 | tick 触发的 run 复用既有 `strict_unattended` 审批；**不**新增第二套限流 | 401/429 → 走既有 provider 重试与失败路径，run 记 `Error` |
| **旧 generation 唤醒污染新 runtime** | 高 | `ScheduledWakeup.generation` 与当前 generation 比对，不匹配即丢弃 | 静默丢弃，**不产生任何副作用**（不写 RunRecord） |
| **P3 恢复的 wakeup 无限复活** | 高 | `consumed` 单次消费 + 单飞锁 + `catch_up_window_secs` 上限 | 超窗记 `Skipped`，不再重试 |
| **执行结果对用户不可见**（G9） | 中 | P1 的 `schedule history` + TUI `/schedule` 摘要 | 记录写入失败仅 stderr 提示，不影响任务退出码 |
| **DAG 成环导致死锁**：A→B→A 永远互等 | 高 | 加载期 `validate_graph` 三色标记检测，检出即拒绝加载并列出环上节点 | **拒绝触发**整个图，而非部分执行；`schedule list` 标红 |
| **悬空边**：`depends_on` 指向已删除的任务 | 高 | 加载期校验引用存在性，fail-closed | 同上：拒绝触发，不静默忽略该边（忽略会让后继误以为"无前驱"而照跑） |
| **`Skipped` 不传染产生"孤立成功"**：A→B→C 中 A 失败、B 跳过、C 照跑 | 高 | `skipped_because_of` 显式记录 + 传染规则（§5.4.1） | C 必须也 Skipped；专项测试 `skipped_propagates_along_chain` 守住 |
| **Agent 自我派活形成无人监督的执行链** | 高 | `schedule_task` 工具受审批门控；默认 `plan`，拒绝 `auto`；`cwd` 限工作区内；**不注册 OS 调度器** | 越界/非法参数 → 拒绝且**不写盘**（无副作用），而非静默降级写入 |
| **`triggers` 空值语义漂移**：既有任务在新版本行为改变 | 中 | 空 `Vec` 必须等价于 `[Trigger::Time]`；专项测试 `empty_triggers_equals_time_only` | 违反即视为回归，非"新特性差异" |
| **`AfterTask` 与 `depends_on` 双份邻接表** | 中 | 加载时归一为同一内部表示（§4.1.1 约束 2） | 同一张图两条路径求值不一致时，以归一后的单一表示为唯一真相 |
| **TUI 横向依赖 CLI（L3 ↔ L3）** | 中 | 共享逻辑下沉 `rustcode-capabilities`，两侧薄封装（§5.4.4） | 若直接 `pub` CLI 函数即违反 §3.2，属架构违规须在评审拦截 |

**统一的失败原则**（对齐 AGENTS.md「runtime 生命周期不变量」）：

1. 每个被接受的运行必须有终态：`Ok` / `Error` / `Cancelled` / `Skipped` 四者之一，
   **不得**停留在 `Running`（进程被杀导致的悬挂 `Running` 由下次 tick 的
   "stale run 回收"修正：超过阈值仍为 `Running` 且锁已释放 → 判定为 `Error`）。
2. 任何一步失败都**显式失败**，不得静默 fresh、空 snapshot 或假成功。
3. 调度层**不得**改变既有退出码契约（`tests/schedule_run_exit_code.rs`）。

---

## 7. 验证与交付

每期合并前必须通过：

| 门禁 | 命令 | 说明 |
|------|------|------|
| G1 | `cargo fmt --check` | 注意 cli 包名是 `rustcode`，格式化用 `cargo fmt -p <包名>` |
| G2 | `cargo clippy --workspace --all-targets -- -D warnings` | 本仓已收紧为 `-D warnings` |
| G3 | `cargo test -j 1 --workspace --no-fail-fast` | **必须 `-j 1`**（cgroup 8 GiB 上限，默认并发会 SIGBUS） |
| G4 | `./scripts/test-headless.sh` | 需先 `cargo build` |
| G5 | `python3 scripts/acp_smoke.py` | |
| 补充 | `python3 scripts/check-zh-docs.py gate` | 本期会改 `docs/plans/`，需过中文文档门禁（AC-4 有已知存量红，见 AGENTS.md） |

**分批测试**（避免满盘导致链接失败，参见 AGENTS.md「环境约束」）：

```bash
cargo test -p rustcode-config --lib
cargo test -p rustcode-capabilities --lib
cargo test -p rustcode-coding --lib
cargo test -p rustcode --lib                      # cli 包名为 rustcode
cargo test -p rustcode --test schedule_run_exit_code
cargo test -p rustcode-tuix --lib
cargo test -p rustcode-daemon --lib
```

**新增测试清单（按期）**：

| 期 | 必增测试 |
|----|---------|
| P0 | `next_run_cron_weekday_alignment`、`next_run_weekly_respects_weekday`、`next_run_cron_is_idempotent` |
| P1 | `run_ledger_roundtrip`、`run_ledger_not_read_as_task_definition`（**关键**）、`prune_runs_keeps_newest_n`、`run_task_still_exits_nonzero_on_failure`（既有测试保持绿） |
| P2 | `try_claim_is_exclusive`、`catch_up_outside_window_records_skipped`、`daemon_tick_disabled_spawns_nothing`、`stale_running_run_is_reaped` |
| P2.5 | `graph_rejects_cycle_and_dangling_edge`、`skipped_propagates_along_chain`（**关键**：A 失败 → B、C 均 Skipped）、`successor_runs_only_when_all_preds_ok`、`schedule_task_tool_denies_auto_mode`、`schedule_task_tool_rejects_out_of_workspace_cwd`、`after_task_trigger_unifies_with_depends_on`、`empty_triggers_equals_time_only`（**关键**：additive 兼容） |
| P3 | `wakeup_survives_process_restart`、`wakeup_consumed_exactly_once`、`stale_generation_wakeup_is_dropped_without_side_effect` |

**locale 锁约定（易踩，务必遵守）**：任何断言本地化文案的测试必须持
`i18n::test_lock()` + `set_locale(...)`。注意本仓**两种锁语义不同**：
`summarise_*` 类测试是**刻意与 locale 无关**的（用 `t()` 现算期望值再比对），
只持锁、**绝不能 `set_locale`**——两次调用间 locale 被翻转会制造新红。

**交付物**：

1. 五期的独立 commit（每期一个，可单独 revert）；
2. 本文件的「实施记录」小节（每期落地后追加实测结果，按 AGENTS.md 惯例）；
3. 更新 `docs/features.md` 的持续工作能力条目；
4. 若 P2 引入 `[schedule]` 配置表，同步更新 `docs/config.example.toml`。

---

## 8. 与既有规划的关系

| 既有材料 | 关系 |
|---------|------|
| `docs/agent-api-rfc.md`（`rustcode-agent-api`，DRAFT） | **互补**。该 RFC 定义"外部编排程序调用 agent"的**远程 API 面**；本方案定义"任务与唤醒如何**持久**"。其 `TaskManager` 同样需要任务级持久化，可直接复用 P1 的 `RunRecord` / `RunLedger`，无需各写一套 |
| `docs/plans/2026-08-21-external-agent-subagent-drivers-spec.md` | 其「scheduled / headless / 非交互上下文下 `Bypass` 一律拒绝」约束已被本方案 §5.3 引为不变量，P2/P3 的调度触发**必须**保持该 posture |
| `crates/rustcode-tunnel` | 与调度无直接耦合；但若后续要做"远程触发"，`/tunnel` 刻意不受 `no_auth` 影响的既有安全边界必须保留（远程接入始终要 token） |
| `docs/plans/2026-08-24-continuous-agent-*.md`（若存在） | 本文件为该项的正式设计，落实后应替换占位 |

---

## 9. 决策记录

### 9.1 已裁决（2026-09-23，用户）

| # | 决策 | 结论 | 生效期 |
|---|------|------|--------|
| D1 | cron 求值实现方式 | **手写五段最小子集**，不引入 `cron` crate（符合 fork 最小化依赖倾向；只需覆盖展示与 tick 判定，无需完整 cron 语义） | P0 |
| D2 | 运行台账目录布局 | **`<task-id>/runs/` 子目录** —— 已实测证明不会被既有 `list_in()` 误读（`p.extension() != "json"` 过滤目录），且便于整任务清理 | P1 |
| D3 | 跨重启到期的 wakeup | **立即跑，但受 `catch_up_window_secs` 约束** —— 与 P2 补跑用**同一个谓词**，避免第二套真相 | P3 |
| D4 | 推进顺序 | **严格串行**，每期验证后再进下一期 —— P1 与 P0 改动 `schedule.rs` 同文件，并行会互相踩 | 全程 |
| **D5** | **"相互调度"的语义范围** | **三类同时要**：① 时序依赖（静态 DAG，人事先定义）② Agent 自主派生（`schedule_task` 工具）③ 事件触发（turn 完成 / 任务终态 / 文件变更） | **P2.5** |
| **D6** | **依赖链失败传播策略** | **fail-fast：前驱失败则不跑，记 `Skipped`**（最保守，不消耗 provider 配额；且 `Skipped` **必须沿链传染**） | **P2.5** |
| **D7** | **补齐方式** | **新增 P2.5 一期，并同步修订本设计文档** —— 保持排期与文档一致，且 P2 保持可独立回滚 | **P2.5** |

> D5–D7 是用户对"为什么没有 `/schedule` 以及相互调度"这一质疑的裁决回应，
> 补齐了本方案初版遗漏的 G10–G13（§2）。

### 9.2 仍开放（待 P1/P2 有实测数据后再定）

| # | 决策 | 选项 | 当前倾向 |
|---|------|------|---------|
| O1 | `OnFileChange` 的文件变更检测方式 | (a) 引入 `notify` 类 watcher (b) 轮询 stat | **(b)** —— 对齐最小化依赖；P2.5 内标记为可选子项，未实现时加载期报错而非静默不触发 |
| O2 | 依赖链的最大深度/宽度限制 | (a) 不限制 (b) 设上限（如深度 32、总节点 256）防病态图 | **(b)** —— 对齐仓库既有的有界化倾向（如 `max_run_history`、`tick_interval_secs` 夹取） |
| O3 | WebUI 调度面做到什么程度 | (a) 只读列表（本期） (b) 完整增删启停 | **(a)** —— 先建立可见性；写操作仍需评估非回环绑定的安全问题（`webui` 默认 `0.0.0.0`） |

---

## 10. 唯一下一步

**P0**：在 `crates/rustcode-config/src/schedule.rs` 的 `next_run()` 中实现
`Schedule::Cron` 求值与 `Weekly` 的 weekday 对齐（**手写五段解析，不引新依赖**，D1），
改写 `next_run_cron_is_none_in_phase1` 为真实值断言，并顺手修正
`crates/rustcode-cli/src/schedule_cmd.rs:1-4` 那条已过期的 "stub" 模块注释（§1.6）；
跑 `cargo test -p rustcode-config --lib` 与 `cargo fmt --check`。

该步自包含、无跨 crate 依赖、可独立合并，且立刻修复一个用户可见的事实错误
（`schedule list` 对 cron 任务显示 `-`）。

**后续路线（严格串行，D4）**：

```text
P0  cron/weekly 求值修复        <- 现在做这一步
P1  运行台账 RunRecord          <- 任务持久化的存储地基
P2  daemon tick + 统一补跑语义  <- 常驻触发（后台执行）
P2.5 相互调度（DAG + agent 派生 + 事件触发）  <- 依赖 P1 的终态 + P2 的常驻触发
P3  可持久化唤醒（跨重启）      <- 最难，需前四期有真实使用数据
```

P2.5 是"相互调度"的落点，其三项子能力（静态 DAG / `schedule_task` 工具 / 事件触发）
**可再拆为独立 commit**，但它们的验收共同依赖 P1 的 `RunRecord`（需前驱终态）
与 P2 的 tick（需近实时触发），故不早于 P2。



