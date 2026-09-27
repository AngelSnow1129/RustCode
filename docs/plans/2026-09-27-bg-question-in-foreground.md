# 后台的问询在前台直接回答

- 日期：2026-09-27
- 状态：待复核（未实施）
- 基线：`release/v5.2.0`。本文档写于 `03f995f6a`、提交进 `a87780ea8`；这段工作区一直有人在改
  （不是本文档的东西），实施前以当时的工作区为准，别照抄这里的 SHA。
- 相关：`docs/plans/2026-09-25-bg-design.md`（后台会话本体）、`docs/plans/2026-09-25-ctrl-b-background-tool.md`

## 1 问题

后台会话撞上「需要人回答」的请求时，它停在 `RuntimePhase::WaitingApproval` →
`BackgroundState::Waiting`（`crates/atomcode-cli/src/background.rs:327`），前台只收到
一句**被动提示**：`BgView::waiting_caption()`（`crates/atomcode-tui/src/bg.rs:142-153`）
→ `Msg::BgWaitingTip` =「后台 [N] <title> 在等你回答 · /bg N 打开」。

要回答，人必须把那个会话换到前台（`/bg N`）；切过去时 `Kind::Subscribe` 把挂着的
那个 `Request` 接在重放后面重发一次（`background.rs:662-704`，判据是
`crates/atomcode-cli/tests/tui_bg.rs::a_question_asked_in_the_background_is_asked_again_on_return`）。

一句话：**信息给到了，答案在那块屏幕上无处可送。**

这个「只提示不弹」是当初特意做的（`10d754b3d feat(tui): …后台等你回答时前台提示一行`，
`a73c6c2b4` 又专门让在等的会话不收就地回复，见 `bg.rs:268-274`），所以这一轮是**改一个
决定**，不是补一个疏漏。

### 1.1 为什么送不到（三处证据）

1. 屏幕回答用的是**裸** `AgentCommand::Respond`（`crates/atomcode-tui/src/plugin.rs:413-415`），
   不像别的命令经 `addressed()`（`plugin.rs:220-229`）带上 `To { session }`。
2. 后台路由把 `Kind::Respond` **一律**送给前台 runtime（`background.rs:644`、`706-716`），
   `To` 的目标根本不看。
3. 到了前台泵就是 `handle.respond(id, value)`（`crates/atomcode-cli/src/host.rs:778`），
   撞上不认识的 id 是 fail-closed 空转（`crates/atomcode-kernel/src/agent/engine.rs:1509`、`1733`）。

## 2 被否掉的方向

- **自动把那个会话换到前台**（`/bg N` 自动化）：`may_move` 明确拒绝在前台正问着的时候换
  （`background.rs:805-808`），于是还得在这里再加一层仲裁；而且会把人当前的会话换走。
- **把 `Respond` 目标化，走现成的 `To { session }`**：`To` 今天只被三处剥离
  （`crates/atomcode-harness/src/plugins/handle.rs:1427-1445`、`crates/atomcode-cli/src/host.rs:355-378`、
  `handle.rs:1636-1645`），后两处都进 `command_member`，而它只认三件事——message、cancel
  它的回合、回合之间的 compaction（`handle.rs:1044-1046`），且只对**被委派出去的成员**生效
  （`handle.rs:1061`）。`Respond` 不在其中。所以那是**扩团队的契约**，不是复用信封。
- **daemon 那条路一起做**：`crates/atomcode-daemon/src/live_hub.rs:817` 的 `respond(id)` 认的是
  当前绑定 live 的 `pending_requests`，另一套机制、另一层 generation 归属。本次不做；
  不支持新命令的宿主按各自兜底拒掉即可（见 §5.3）。

## 3 本次边界（明确不碰）

只碰三处：

- `crates/atomcode-host-api`：两个命令变体 + 一个回复 + 一个事实 DTO；
- `crates/atomcode-cli/src/background.rs`：后台槽位表自己的路由；
- `crates/atomcode-tui`：把问询提上来。

不碰：`command_member` 的三件事、`Respond` 现有的「永远送前台」语义、
`crates/atomcode-daemon`、`BgView` / `BackgroundSession` 的形状、`crates/atomcode-coding`。

## 4 状态 owner 与目标边界

- 后台问询的**唯一 owner 仍是那个 `Live` 自己的 runtime**：请求挂在它的命令通道上
  （`background.rs:56` 的 `commands`），答案由 `Background` 找到那个槽位后送过去。
  屏幕不持有它，也不替它下结论。
- **屏幕新增的状态只有一条**：`Moment` 上 `bg_asked: Option<(String /*session*/, u64 /*ask id*/)>`
  ——「哪一条问询是从后台提上来的」。形状照 `resume_preview`
  （`crates/atomcode-tui/src/moment.rs:798`）。
- 目标边界：人答完**留在自己那段对话里**，那个后台会话继续在后台跑。

## 5 数据流

### 5.1 取回（按需，不挂在列表事件上）

```text
HostCommand::BackgroundQuestion { target: String }
  → HostReply::BackgroundQuestion {
        session: String,
        id: u64,                 // 那个 Request 的 id
        kind: String,            // "approval" / "request_user_input" / …
        payload: serde_json::Value,
        facts: Vec<LoggedFact>,  // 那个会话日志的尾 64 条
    }

struct LoggedFact { seq: SeqNo, at: u64, event: SessionEvent }   // host-api 自己的形状
// 必须 Serialize + Deserialize：HostReply 是跨进程的（enum 自身就要求）
```

**为什么带事实**：`question_for`/`batch_for` 只反向找最近一条 `SessionEvent::Asked`
（`ask.rs:377-410`、`545-555`），那条记录里有面板要的选项、问者和被问的调用。少了它，
审批会退化成「没有 allow-all、原始 header」的问法——`ask.rs:430-442` 的注释自己说了这件事。
`SessionPreview` 那条给的是**渲染好的行**（`host-api/src/lib.rs:414-418`），因为「读一段对话」
就是读行；这里是**要回答**，行不够。

**为什么不是 `Vec<LoggedEvent>`**：`LoggedEvent` 只有 `Clone, Debug, PartialEq`，**没有 serde**
（`crates/atomcode-kernel/src/session.rs:641-650`），进不了跨进程的 `HostReply`。同一组三元组
在磁盘上是手拼的 JSON（`crates/atomcode-capabilities/src/session/events.rs:883-887`），
`Committed` 是它的带 session 版本（`session.rs:659`）。所以由 host-api 出这个 DTO——它本来
就是放跨边界形状的地方；kernel 一个字节不改，TUI 侧把它映射成 `LoggedEvent`（字段全公开，
一行的事）。`event` 与其中的 `Question` 本来就是 serde（`session.rs:111-114`、`1279`）。

**取多少**：尾 64 条。`question_for` 取的是**最新的那条匹配**，所以尾巴够；64 是预算，
实测可调。来源是现成的 `log_of`（`background.rs:344-357`），`describe()` 每轮已经在调它。

**分层**：cli 只搬不解释；把 payload 变成问题是 TUI 自己的 `question_for`。

### 5.2 画

TUI 用 `question_for`/`batch_for` 造出 `Asked`，塞进现有的 `host.asks`，把 ask id 与那个
session 记进 `bg_asked`。

- 绘制、按键、`sync_asking`、面板与流尾两种画法**全部复用**，不新画一套——「直接出来问询」
  就是这一句的实现。
- **asker 那行要标出是哪个后台会话在问**（前缀式，不覆盖原来的问者）：否则人在自己那段对话
  底下看到「允许写文件吗」会以为是自己触发的。slot 与标题从屏幕已有的 `BgView` 取
  （`bg.rs:124-127` 的 `slot_of`），**不往协议里加**。

### 5.3 回答

push 时 spawn 的任务等答案；回来后经 `AgentClient` 上的一条新方法发出去（它已经握着
`control`，`plugin.rs:175-178` 的 `Link { commands, control }`）：

```text
control.call(HostCommand::AnswerBackground { target, id, value })
  → Background::call 找到槽位 → slot.commands.send(AgentCommand::Respond { id, value })
```

并把 `track.pending` 清掉（照今天 `Kind::Respond` 的做法，`background.rs:706-716`）。

**一处必须写对的坑**：`Background::call` 今天的兜底是**把不认识的命令转给前台**
（`background.rs:1021-1024`），所以两个新变体必须各自显式成臂——漏了就会被转给前台去答。
而完全不认识它的宿主（daemon、测试里的假宿主）走各自兜底拒掉
（如 `crates/atomcode-cli/src/host.rs:2652-2654`），屏幕就退回今天的提示行：不崩、不假装。

## 6 仲裁（前台优先，后台排队）

一条规则：**只在 `asks` 空着时才提后台的问询**；前台一有问询就停止提，已经提上来的不抢。

- 两个触发点调同一个函数：`HostEvent::BackgroundChanged` 那个臂里（`plugin.rs:1483-1489`）、
  以及一次问询答完之后。
- 取的时候按 slot 顺序拿第一个 `Waiting` 的；同一个会话不重复取。
- **提出来的必须能收回去**：`Asks` 多一个 `withdraw(id)`——**移除且不投递**。`ask.rs` 里
  `Pending` 没有 `Drop` 实现，丢掉 oneshot sender 就等于取消，所以谁也没答。这与
  `finish(Vec::new())`（明确拒绝、接收端拿到 `None`，`ask.rs:186-204`）是**两件不同的事**，
  测试要分别钉死。

## 7 作废与失败语义

| 情形 | 行为 |
| --- | --- |
| 目标会话不再 `Waiting`（回合结束 / 被取消 / 失败） | withdraw，一个字节都不发 |
| 被 `/bg drop` | withdraw；屏幕上那行说它已经不在后台了 |
| 被换到前台（`/bg N`） | withdraw——那条问询会由现有的重订阅路径在前台重问一次，留着就出两份 |
| 屏幕正在关 | withdraw；`asks.refuse_all()`（`ask.rs:316-328`）已兜住其余 |
| 这个屏幕画不出来（`question_for` 返回 `None`） | 照前台现行规矩 fail-closed 回 `Null`（`plugin.rs:4020-4024`），否则那个后台会话永远挂着；**并在屏幕上留一行说明**（新 i18n 串），否则人只会看到后台莫名其妙不动了 |
| `target` 找不到 | `HostError::NotFound`，屏幕明说「它已经不在后台了」（比今天 `Kind::Respond` 的静默空转好） |
| `id` 对不上当时挂着的那个 | 明确拒绝并说「那个问题已经不在了」 |
| 宿主不认识这条命令 | 屏幕退回今天的提示行（不崩、不假装） |

不变式：**屏幕绝不会替后台下结论**——要么把人给的答案原样送到，要么什么都不发。

## 8 新增面（实施清单）

`crates/atomcode-host-api/src/lib.rs`：

- `HostCommand::{BackgroundQuestion, AnswerBackground}`、`HostReply::BackgroundQuestion`、
  `struct LoggedFact`（enum 已是 `#[non_exhaustive]`，加变体是既定做法）。
- **两处跟着枚举走的既有代码必须补臂**，否则是编译错而不是运行错：
  - 命令 → 会话的映射（`lib.rs:376-382` 那段 `BackgroundSessions | StartBackground { .. } |
    TellBackground { .. } | DropBackground { .. } => None`）；
  - 测试里逐条列举的 `commands()` / `replies()` 夹具（`lib.rs:1542-1548`、`2027-2033`）。

`crates/atomcode-cli/src/background.rs`：

- 两个**显式**成臂（不能落到 `:1021-1024` 那个「转给前台」的兜底）；
- `BackgroundQuestion`：找槽位，`NotFound` 时如实拒；带上尾 64 条事实（复用 `log_of`）；
- `AnswerBackground`：找槽位、核对 `track.pending` 的 id、`slot.commands.send(Respond)`、
  清掉 `pending`。

`crates/atomcode-tui`：

- `ask.rs`：`Asks::withdraw(id)`；一个能拿到 ask id 的 push（今天 `push` 只回 receiver，
  `ask.rs:334-338`；`peek()` 只报队首，`:254`）。
- `moment.rs`：`bg_asked`。
- `host.rs` / `plugin.rs`：`*_wanted()` 式的「要不要提」、`RawFacts → LoggedEvent` 的映射、
  回答走 `AnswerBackground`、asker 前缀、作废时机。
- `crates/atomcode-i18n/src/screen/{messages.rs,zh_cn.rs,en.rs}`：asker 前缀、画不出来那一行、
  `NotFound` 与 id 对不上两句。

## 9 判据

单元（`crates/atomcode-tui/src/ask.rs`）：

- `withdraw` 之后接收端是**取消**，`finish(vec![])` 是 `None`——两者不能混。
- 同一个 ask id 被 withdraw 两次、withdraw 一个已经答掉的 id，都不出错。

单元（TUI）：

- 前台有问询时不提后台的；前台答完才提；同一会话不重复提。
- 「画提示行」与「已提上来画问询」互斥。

集成（`crates/atomcode-cli/tests/tui_bg.rs`，已有 `Rig` + `until_background` 装置）：

- `/background ask me` → 前台**直接出现问询**（不再是只提示）→ 答 → 那个后台会话收到答案继续跑。
- `/bg drop` 时屏幕收回问询，且那个 runtime **没收到任何答案**。
- 反向：不支持这条命令的宿主上，屏幕仍是今天的提示行。

## 10 未决 / 后续（不在这次范围）

- **团队成员会话的问询归谁答：今天这条不存在**（证据见 §2 的 `command_member` 一段：它只认
  message / cancel / compaction，且只对委派出去的成员生效；屏幕的回答是裸 `Respond`，只能
  落到根 runtime）。候选是给 `command_member` 加 `Respond`，与后台无关，另开一份。
- daemon 那条路（`live_hub.rs`）若要一起支持，按 generation 归属另做。

## 11 验证方式

- 合之前：`bash gates/compile.sh`（`--workspace --all-targets` 的量，按 crate 跑看不见
  `lib.rs` 那两处漏臂）。
- 相关 crate：`cargo nextest run -p atomcode-host-api -p atomcode-cli -p atomcode-tui`
  （本机没装 nextest 时退回 `cargo test -p …`；`ask.rs` 的单元测试落在 tui 的同一个 binary 里）。
- `cargo fmt --all -- --check` 必须退出 0。
