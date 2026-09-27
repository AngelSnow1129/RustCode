# 后台的问询在前台直接回答 —— 实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** 后台会话撞上需人回答的请求时，那个问询在前台**直接出来**并可就地回答，人不必先把
会话换到前台。

**Architecture:** 新增两条宿主命令——`BackgroundQuestion`（按需取回那次问答：id、kind、
payload，以及画它要用的日志尾巴）与 `AnswerBackground`（把答案送进那个会话自己的命令通道）。
屏幕把问题塞进现有的 `host.asks`，绘制、按键、仲裁全部复用；只在队列空着时才提后台的问询，
提上来的可以**静默收回**（不是拒绝）。

**Tech Stack:** Rust，tokio；`atomcode-host-api`（跨进程的宿主契约）、`atomcode-cli` 的
`background`（后台槽位表与路由）、`atomcode-tui`（`ask`/`host`/`moment`/`plugin`）、
`atomcode-i18n`。

**Spec:** `docs/plans/2026-09-27-bg-question-in-foreground.md` —— 本计划从它出发，执行时两份都读。

## Global Constraints

- 基线 `release/v5.2.0`。**工作区一直有别人未提交的改动**（`crates/atomcode-tui/src/{commands.rs,plugin.rs}`、
  `crates/atomcode-i18n/src/screen/{messages.rs,en.rs,zh_cn.rs}` 等）：只提交本计划改的文件，
  永远不要 `git add -A`、不要 `cargo fmt --all` 之外的批量改写别人的文件。
- **不碰**：`command_member` 的三件事（`harness/src/plugins/handle.rs`）、`Respond` 现有的
  「永远送前台」语义（`cli/src/background.rs:706-716` 那条路照旧）、`crates/atomcode-daemon`
  的 `live_hub.rs`、`crates/atomcode-coding`、`BgView` / `BackgroundSession` 的形状。
- 关掉新命令的宿主（daemon、测试假宿主）必须**按各自兜底拒掉**，屏幕退回今天的提示行：
  不崩、不假装。
- **屏幕绝不替后台下结论**：要么把人给的答案原样送到，要么什么都不发（收回 = oneshot 取消，
  ≠ `finish(Vec::new())` 那个「拒绝」）。
- 跑测试用 `cargo nextest run -p <crate>`；本机没装 nextest 时用 `cargo test -p <crate>`。
  合之前 `bash gates/compile.sh`（`--workspace --all-targets` 的量）与 `cargo fmt --all -- --check`。
- 新增 async 测试只要路径上有超时/退避，写 `#[tokio::test(flavor = "current_thread", start_paused = true)]`；
  本计划的判据都不等墙钟，除集成测试外都按各 crate 既有装置写。

---

### Task 1: host-api —— 两条命令、一条回复、一个事实 DTO

**Files:**
- Modify: `crates/atomcode-host-api/src/lib.rs`
  - `HostCommand` 枚举：`DropBackground`（`:319` 附近）之后加两个变体
  - `HostCommand::addressed()`：`:379-382` 那个 `None` 臂
  - `HostReply` 枚举：`BackgroundSessions`（`:593-596`）之后加一个变体
  - 新 `struct LoggedFact`：`BackgroundSession`（`:1243-1263`）之后
  - 夹具 `commands()`（`:1558` 附近）、`replies()`（`:1859` 附近）
  - 测试 `a_command_on_the_live_session_names_it`（`:2028-2035`）的 match
- Test: 同文件 `mod tests`

**Interfaces:**
- Produces（后面每个任务都靠这些确切名字与类型）：
  - `HostCommand::BackgroundQuestion { target: String }`
  - `HostCommand::AnswerBackground { target: String, id: atomcode_kernel::event::RequestId, value: serde_json::Value }`
  - `HostReply::BackgroundQuestion { session: String, id: atomcode_kernel::event::RequestId, kind: String, payload: serde_json::Value, facts: Vec<LoggedFact> }`
  - `pub struct LoggedFact { pub seq: SeqNo, pub at: u64, pub event: atomcode_kernel::session::SessionEvent }`
- Consumes: `crosses()` 测试助手（同文件 `:2015-2020` 在用）

- [ ] **Step 1: 写失败的测试**

在 `crates/atomcode-host-api/src/lib.rs` 的 `mod tests` 里，`background_session()` 那个助手旁边加：

```rust
    /// 那个问询要原样过得去:id、载荷、以及画它要用的那几条事实。
    #[test]
    fn a_background_question_crosses_the_wire_whole() {
        let reply = HostReply::BackgroundQuestion {
            session: "b".into(),
            id: 7,
            kind: "approval".into(),
            payload: serde_json::json!({ "tool": "write_file", "args": "{}" }),
            facts: vec![LoggedFact {
                seq: 1,
                at: 1_758_000_000_000,
                event: atomcode_kernel::session::SessionEvent::TurnStart { turn: 1 },
            }],
        };
        crosses(&reply);
    }
```

- [ ] **Step 2: 跑它，确认它失败**

Run: `cargo test -p atomcode-host-api`
Expected: 编译错 `error[E0599]: no variant or associated item named 'BackgroundQuestion' found for enum 'HostReply'`

- [ ] **Step 3: 加两个命令变体**

`HostCommand` 里 `DropBackground { target: String },` 之后：

```rust
    /// 一个后台会话此刻挂着的那个问询，原样取回来。
    ///
    /// 它不是列表的一部分：`BackgroundChanged` 说的是「有哪些会话、各自什么状态」，
    /// 而这里要的是那次问答本身 —— 载荷，以及画它要用的那几条刚被记下的 `Asked`。
    /// 会话不在后台、或没在等人，都是 [`HostError::NotFound`]：没有那个问询可说，
    /// 不猜一个。
    BackgroundQuestion { target: String },
    /// 把一个后台会话挂着的那个请求答了，用那个请求自己的 id 与词汇。
    ///
    /// 与 [`HostCommand::TellBackground`] 的区别：那一个送一句话开一个新回合，这一个
    /// 只回答已经挂着的那个。挂着的已经不是它了就明说（拒绝），绝不把答案安在别的问题上。
    AnswerBackground {
        target: String,
        id: atomcode_kernel::event::RequestId,
        value: serde_json::Value,
    },
```

- [ ] **Step 4: 把两条命令归到「不针对当前 live 会话」**

`addressed()` 里 `Self::BackgroundSessions | Self::StartBackground { .. } | Self::TellBackground { .. }
| Self::DropBackground { .. } => None,` 那个臂加上两条：

```rust
            Self::BackgroundSessions
            | Self::StartBackground { .. }
            | Self::TellBackground { .. }
            | Self::DropBackground { .. }
            | Self::BackgroundQuestion { .. }
            | Self::AnswerBackground { .. } => None,
```

同时把测试 `a_command_on_the_live_session_names_it` 里那份同名列表（`:2029-2033`）照抄加上这两条 ——
**两处要一致**，这个测试就是钉这件事的。

- [ ] **Step 5: 加回复变体与事实 DTO**

`HostReply` 里 `BackgroundSessions { sessions: Vec<BackgroundSession> },` 之后：

```rust
    /// 一个后台会话挂着的那个问询，完整到能画。
    BackgroundQuestion {
        session: String,
        /// 它挂着的那个请求的 id —— 要回答的就是它。
        id: atomcode_kernel::event::RequestId,
        /// 请求自己的 kind，屏幕照它把载荷变成问题（与它自己那个会话同一套代码）。
        kind: String,
        payload: serde_json::Value,
        /// 那个会话日志的尾巴：屏幕要靠它找到 agent 在发问前记下的那条 `Asked`，
        /// 少了它，审批会退化成没有 allow-all 的原始问法。
        facts: Vec<LoggedFact>,
    },
```

`BackgroundSession` 那个 struct 之后：

```rust
/// 一条已提交的事实，过线时的形状。
///
/// `atomcode_kernel::session::LoggedEvent` 是同一个三元组，但**故意不可序列化**
/// （只有 `Clone, Debug, PartialEq`）：它是读者的类型，落盘是一条条手写的。
/// 凡是前端要问的都得过进程边界，所以形状在这里说一遍 —— 里面的 `event` 与它
/// 带的 `Question` 本来就是可序列化的。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LoggedFact {
    pub seq: SeqNo,
    pub at: u64,
    pub event: atomcode_kernel::session::SessionEvent,
}
```

**注意：不要给 `LoggedFact` 加 `#[non_exhaustive]`** —— 别的 crate 要构造它（`cli` 侧就是），
加了就编不出来。也不要加 `Eq`：`SessionEvent` 只有 `PartialEq`。

- [ ] **Step 6: 把两条命令与那条回复加进夹具**

`commands()` 里 `HostCommand::DropBackground { target: "b".into() },` 之后：

```rust
            HostCommand::BackgroundQuestion { target: "b".into() },
            HostCommand::AnswerBackground {
                target: "b".into(),
                id: 7,
                value: serde_json::json!({ "decision": "allow" }),
            },
```

`replies()` 里 `HostReply::BackgroundSessions { sessions: vec![background_session()] }` 那一项之后：

```rust
                HostReply::BackgroundQuestion {
                    session: "b".into(),
                    id: 7,
                    kind: "approval".into(),
                    payload: serde_json::json!({ "tool": "write_file", "args": "{}" }),
                    facts: vec![LoggedFact {
                        seq: 1,
                        at: 1_758_000_000_000,
                        event: atomcode_kernel::session::SessionEvent::TurnStart { turn: 1 },
                    }],
                },
```

- [ ] **Step 7: 跑测试，确认全过**

Run: `cargo test -p atomcode-host-api`
Expected: PASS（含既有的 `crosses` 系列：命令、回复、事件、错误各走一遍 JSON 往返）

- [ ] **Step 8: 提交**

```bash
git add crates/atomcode-host-api/src/lib.rs
git commit -m "feat(host-api): 后台问询的两条命令与一条回复

BackgroundQuestion 取回那次问答(载荷 + 画它的日志尾巴),
AnswerBackground 把答案送进那个会话自己的命令通道。LoggedFact 是
过线的事实形状:kernel 的 LoggedEvent 故意不可序列化。"
```

---

### Task 2: cli —— `BackgroundQuestion` 取回问询

**Files:**
- Modify: `crates/atomcode-cli/src/background.rs`
  - `TellBackground` / `DropBackground` 两个方法（`:959-992`）附近加 `fn question`
  - `impl HostControl for Background`（`:1007-1025`）加一条显式臂
  - 文件里加 `const ASKED_TAIL` 与 `fn tail_of`
- Test: `crates/atomcode-cli/tests/tui_bg.rs`

**Interfaces:**
- Consumes: Task 1 的 `HostCommand::BackgroundQuestion`、`HostReply::BackgroundQuestion`、`LoggedFact`
- Produces: `Background::question(target) -> Result<HostReply, HostError>`（私有）、
  `fn tail_of(log: &[LoggedEvent]) -> Vec<LoggedFact>`（私有）
- 用到的现成件：`log_of(&Live)`（`:344-357`，`describe()` 每轮已经在调）、
  `Live.track.pending`（`Track::saw` 在 `:127-130` 存的是整个 `AgentEvent::Request`）

- [ ] **Step 1: 写失败的测试**

在 `crates/atomcode-cli/tests/tui_bg.rs` 末尾加（该文件已有 `Rig::new()`、`rig.term.type_line`、
`rig.until_background`、`rig.background()`、`rig.control()` 这些装置，照既有测试的写法用）：

```rust
/// 后台会话挂着的那个问询要能整个取回来:id、载荷、以及画它要用的日志尾巴。
#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial(atomcode_home)]
async fn a_background_question_comes_back_whole() {
    let rig = Rig::new().await;
    rig.term.type_line("/background ask me");
    rig.until_background("the background session is waiting", |list| {
        list.first()
            .is_some_and(|s| s.state == BackgroundState::Waiting)
    })
    .await;
    let target = rig.background().await[0].session.clone();

    let reply = rig
        .control()
        .call(HostCommand::BackgroundQuestion {
            target: target.clone(),
        })
        .await
        .expect("在等的会话有问询可取");
    let HostReply::BackgroundQuestion {
        session,
        kind,
        payload,
        facts,
        ..
    } = reply
    else {
        panic!("不是一条问询: {reply:?}");
    };
    assert_eq!(session, target);
    assert!(!kind.is_empty(), "kind 要说得出是什么问法");
    assert!(payload.is_object(), "载荷要原样带回来: {payload:?}");
    assert!(!facts.is_empty(), "尾巴不能是空的");
    assert!(
        facts
            .iter()
            .any(|fact| matches!(&fact.event, SessionEvent::Asked { .. })),
        "画问询要的那条 Asked 在尾巴里"
    );
}
```

`SessionEvent` 从 `atomcode_kernel::session::SessionEvent` 引进来（文件顶部的 `use` 里加）。

> 若那条 `/background ask me` 脚本走的是**不记 `Asked`** 的问法（`background.rs:448-461`
> 的注释写着「不是每种问法都先在日志里记一条 `Asked`」），把最后那条断言换成
> `assert!(!facts.is_empty())` 并在测试注释里写明原因 —— 别把断言删掉了事。

- [ ] **Step 2: 跑它，确认它失败**

Run: `cargo test -p atomcode-cli --test tui_bg a_background_question_comes_back_whole`
Expected: 编译错 —— `BackgroundQuestion` 这条命令在 `Background::call` 里落到兜底，测试里则是
`HostReply::BackgroundQuestion` 无法解构（或 `expect` 拿到 `HostError::Failed { message: "this host
does not do that yet" }` 之类）。两种都算「还没实现」。

- [ ] **Step 3: 加尾巴长度常量与搬运函数**

`crates/atomcode-cli/src/background.rs` 里 `fn log_of` 附近：

```rust
/// 画一个问询要看的日志尾巴有多长。
///
/// `question_for` 取的是**最新的那条**匹配（`crates/atomcode-tui/src/ask.rs`），
/// 所以尾巴够。64 是预算，不是契约：真要更长的问法，先量再改。
const ASKED_TAIL: usize = 64;

/// 日志的最后 [`ASKED_TAIL`] 条，照过线的形状。
///
/// 搬的是事实本身，不解释：把载荷变成问题是屏幕自己的事（`crate::ask`）——
/// 这一层不认识「问询」这个词。
fn tail_of(log: &[LoggedEvent]) -> Vec<LoggedFact> {
    log[log.len().saturating_sub(ASKED_TAIL)..]
        .iter()
        .map(|logged| LoggedFact {
            seq: logged.seq,
            at: logged.at,
            event: logged.event.clone(),
        })
        .collect()
}
```

顶部 `use atomcode_host_api::{...}` 里加上 `LoggedFact`（与 `HostReply` 同一个列表）。

- [ ] **Step 4: 实现取回**

`fn drop_one` 之后加：

```rust
    /// 那个会话此刻挂着的问询。没在等人就没有可说的。
    ///
    /// 不猜、也不编：列表那一行已经说了它在干什么，这里再说一遍是噪音；而屏幕
    /// 只在它自己认为那个会话在等人时才会来要，所以这两条 `NotFound` 都是竞态的护栏。
    fn question(&self, target: String) -> Result<HostReply, HostError> {
        let state = self.state.lock().expect("background poisoned");
        let live = state
            .slots
            .iter()
            .find(|slot| slot.control.session_id() == target)
            .ok_or(HostError::NotFound)?;
        let (id, kind, payload) = {
            let track = live.track.lock().expect("track poisoned");
            match track.pending.clone() {
                Some(AgentEvent::Request { id, kind, payload }) => (id, kind, payload),
                _ => return Err(HostError::NotFound),
            }
        };
        Ok(HostReply::BackgroundQuestion {
            session: live.control.session_id(),
            id,
            kind,
            payload,
            facts: tail_of(&log_of(live)),
        })
    }
```

- [ ] **Step 5: 加显式的命令臂**

`impl HostControl for Background` 里 `DropBackground` 那条之后：

```rust
            HostCommand::BackgroundQuestion { target } => self.question(target),
```

**必须显式成臂**：今天的兜底（`:1021-1024`）是「不认识就转给前台」，漏了这条会把
「问后台会话」转给前台去答，而前台既不认识那个会话也不认识那个问题。

- [ ] **Step 6: 跑测试，确认它过**

Run: `cargo test -p atomcode-cli --test tui_bg a_background_question_comes_back_whole`
Expected: PASS

- [ ] **Step 7: 跑这个测试文件，确认没踩到别人**

Run: `cargo test -p atomcode-cli --test tui_bg`
Expected: PASS（全绿）

- [ ] **Step 8: 提交**

```bash
git add crates/atomcode-cli/src/background.rs crates/atomcode-cli/tests/tui_bg.rs
git commit -m "feat(cli): 后台会话挂着的问询可以按需取回

BackgroundQuestion 回答那次问答本身:请求的 id、kind、载荷,以及那个
会话日志的尾 64 条 —— 屏幕要靠它找到发问前记下的那条 Asked。"
```

---

### Task 3: cli —— `AnswerBackground` 把答案送进那个会话

**Files:**
- Modify: `crates/atomcode-cli/src/background.rs`（`fn question` 之后加 `fn answer`；`impl HostControl`
  里再加一条显式臂）
- Modify: `crates/atomcode-i18n/src/screen/messages.rs`（`Msg::BgAnswerStale`）、`zh_cn.rs`、`en.rs`
- Test: `crates/atomcode-cli/tests/tui_bg.rs`

**Interfaces:**
- Consumes: Task 1 的 `HostCommand::AnswerBackground`；现成的 `Self::busy`（`:795-797`）、
  `Live.commands`（`:56`）、`Live.track.pending`
- Produces: `Background::answer(target, id, value) -> Result<HostReply, HostError>`

- [ ] **Step 1: 写失败的测试**

`crates/atomcode-cli/tests/tui_bg.rs` 末尾加两条：

```rust
/// 答了,那个后台会话就不再等它了 —— 而且挂着的那个确实被这一次答掉了。
#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial(atomcode_home)]
async fn answering_a_background_question_clears_what_it_was_waiting_on() {
    let rig = Rig::new().await;
    rig.term.type_line("/background ask me");
    rig.until_background("the background session is waiting", |list| {
        list.first()
            .is_some_and(|s| s.state == BackgroundState::Waiting)
    })
    .await;
    let target = rig.background().await[0].session.clone();
    let HostReply::BackgroundQuestion { id, .. } = rig
        .control()
        .call(HostCommand::BackgroundQuestion {
            target: target.clone(),
        })
        .await
        .expect("在等的会话有问询可取")
    else {
        panic!("不是一条问询");
    };

    // `Value::Null` 是任何 kind 都收得下的「没有答案」,与屏幕画不出来时的答复一致
    // (`plugin.rs:4020-4024`)。这一条测的是**送达与记账**;答的语义由端到端那条测。
    rig.control()
        .call(HostCommand::AnswerBackground {
            target: target.clone(),
            id,
            value: serde_json::Value::Null,
        })
        .await
        .expect("挂着的就是它,答得进去");

    rig.until_background("it moved on", |list| {
        list.first()
            .is_some_and(|s| s.state != BackgroundState::Waiting)
    })
    .await;

    // 同一个 id 再答一次:挂着的已经不是它了,明说,而不是塞给一个等着别的东西的 runtime。
    assert!(matches!(
        rig.control()
            .call(HostCommand::AnswerBackground {
                target: target.clone(),
                id,
                value: serde_json::Value::Null,
            })
            .await,
        Err(HostError::Busy { .. })
    ));
}

/// 不在后台的会话没得答:如实说找不到。
#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial(atomcode_home)]
async fn answering_a_session_that_is_not_in_the_background_is_not_found() {
    let rig = Rig::new().await;
    assert!(matches!(
        rig.control()
            .call(HostCommand::AnswerBackground {
                target: "nobody".into(),
                id: 1,
                value: serde_json::Value::Null,
            })
            .await,
        Err(HostError::NotFound)
    ));
}
```

- [ ] **Step 2: 跑它们，确认它们失败**

Run: `cargo test -p atomcode-cli --test tui_bg answering_a_background_question_clears_what_it_was_waiting_on answering_a_session_that_is_not_in_the_background_is_not_found`
Expected: 第一条在 `expect("挂着的就是它,答得进去")` 上失败（命令落到「转给前台」的兜底，被前台
拒掉）；第二条同样拿不到 `NotFound`。

- [ ] **Step 3: 加那条 i18n 串**

`crates/atomcode-i18n/src/screen/messages.rs`：`BgWaitingTip { slot, title },`（`:635-638`）旁边：

```rust
    /// 那个后台会话挂着的已经不是这个请求了:答案不能安在别的问题上。
    BgAnswerStale,
```

`zh_cn.rs`：`Msg::BgWaitingTip { .. }` 那一行附近：

```rust
        Msg::BgAnswerStale => "那个后台会话已经不在等这个问题了".into(),
```

`en.rs`：同一位置：

```rust
        Msg::BgAnswerStale => "that background session is no longer waiting on this question".into(),
```

> 这两个 i18n 文件**别人也有在飞的改动**。提交前 `git diff crates/atomcode-i18n/src/screen/zh_cn.rs`
> 看一眼：只属于本次的 hunk 才 `git add`（能交互时用 `git add -p`）；分不开就把 i18n 那一份留在
> 工作区、本任务先提交别的，并在最后统一说明。**永远不要 `git add -A`。**

- [ ] **Step 4: 实现**

`fn question` 之后：

```rust
    /// 把那个后台会话挂着的那个请求答了。答不进去就明说。
    ///
    /// 答案走的是**那个 runtime 自己的命令通道**（`Live.commands`），与屏幕答自己那个
    /// 会话时走的 `AgentCommand::Respond` 是同一条路 —— 变的只是它去的 runtime。
    fn answer(
        &self,
        target: String,
        id: atomcode_kernel::event::RequestId,
        value: serde_json::Value,
    ) -> Result<HostReply, HostError> {
        use atomcode_i18n::screen::{t as tr, Msg as SMsg};
        let state = self.state.lock().expect("background poisoned");
        let live = state
            .slots
            .iter()
            .find(|slot| slot.control.session_id() == target)
            .ok_or(HostError::NotFound)?;
        {
            let mut track = live.track.lock().expect("track poisoned");
            match track.pending.as_ref() {
                Some(AgentEvent::Request { id: waiting, .. }) if *waiting == id => {
                    track.pending = None;
                }
                // 挂着的已经不是它了（作废、被别处答过、回合结束）：拒绝，而不是把
                // 人的答案安在别的问题上。
                _ => return Err(Self::busy(tr(SMsg::BgAnswerStale).into_owned())),
            }
        }
        let _ = live.commands.send(AgentCommand::Respond { id, value });
        Ok(HostReply::Done)
    }
```

- [ ] **Step 5: 加显式的命令臂**

`impl HostControl for Background` 里 `BackgroundQuestion` 那条之后：

```rust
            HostCommand::AnswerBackground { target, id, value } => {
                self.answer(target, id, value)
            }
```

- [ ] **Step 6: 跑它们，确认它们过**

Run: `cargo test -p atomcode-cli --test tui_bg`
Expected: PASS（含 Task 2 那条与既有的 `/bg` 全系列）

- [ ] **Step 7: 提交**

```bash
git add crates/atomcode-cli/src/background.rs crates/atomcode-cli/tests/tui_bg.rs
git commit -m "feat(cli): 后台会话挂着的问询可以就地答掉

AnswerBackground 把答案送进那个会话自己的命令通道,并核对着的确实是
那一个:不是就拒绝,绝不把答案安在别的问题上。"
```

（i18n 那两个文件按 Step 3 的说明处理。）

---

### Task 4: tui —— `Asks` 的「收回」与带 id 的 push

**Files:**
- Modify: `crates/atomcode-tui/src/ask.rs`
  - `Asks::take_id`（`:305-314`）之后加 `withdraw`
  - `push`/`push_batch`（`:330-346`）之后加两条带 id 的
  - `enqueue`（`:348-364`）改成返回那个 id
- Test: 同文件 `mod tests`

**Interfaces:**
- Produces（Task 6/7/8 靠它们）：
  - `Asks::withdraw(&self, id: u64) -> bool`
  - `Asks::push_with_id(&self, asked: impl Into<Asked>) -> (u64, oneshot::Receiver<Option<Reply>>)`
  - `Asks::push_batch_with_id(&self, asked: Vec<Asked>) -> (u64, oneshot::Receiver<Vec<Option<Reply>>>)`
- Consumes: 现成的 `Pending`（无 `Drop` 实现）、`Replier::One` / `Replier::Many`（`:186-204`）、
  `Asks::peek`（`:254`）、`Asks::take_id`（`:308`）

- [ ] **Step 1: 写失败的测试**

`crates/atomcode-tui/src/ask.rs` 的 `mod tests` 里，`a_batch_comes_back_whole_through_the_queue`
（`:1811` 附近）旁边：

```rust
    /// 收回去**不是**拒绝:等着的那个人看到的是取消,不是「答了 no」。
    ///
    /// 这两件事在屏幕上看起来一样（问询没了),在那个后台会话那里差得远:一个什么
    /// 都没收到,一个收到了一次拒绝。屏幕收回时走的必须是前一条。
    #[tokio::test]
    async fn withdrawing_is_a_cancel_not_a_refusal() {
        let asks = Asks::new();
        let up = asks.push(Question::plain("Allow?", &["yes", "no"]));
        let (id, _) = asks.peek().expect("它就在屏幕上");
        assert!(asks.withdraw(id), "它在队列里");
        assert!(up.await.is_err(), "取消:通道没了,而不是 Some(None)");

        let refused = asks.push(Question::plain("Allow?", &["yes", "no"]));
        let (id, _) = asks.peek().expect("它就在屏幕上");
        asks.take_id(id).unwrap().finish(Vec::new());
        assert_eq!(refused.await.unwrap(), None, "拒绝是一次答复,拿得到 None");

        assert!(!asks.withdraw(999), "不在队列里的 id 什么都不做");
    }

    /// 提上来的时候要记住是哪一条,才收得回去。
    #[tokio::test]
    async fn a_pushed_question_reports_its_own_id() {
        let asks = Asks::new();
        let (id, answer) = asks.push_with_id(Question::plain("Allow?", &["yes", "no"]));
        assert_eq!(asks.peek().map(|(up, _)| up), Some(id), "报的就是队首那条");
        asks.take_id(id).unwrap().answer(Some(Reply::from("yes")));
        assert_eq!(answer.await.unwrap(), Some(Reply::from("yes")));

        let (id, answer) = asks.push_batch_with_id(vec![single(), text()]);
        assert_eq!(asks.peek().map(|(up, _)| up), Some(id));
        asks.take_id(id)
            .unwrap()
            .finish(vec![Some(Reply::from("vanilla"))]);
        assert_eq!(
            answer.await.unwrap(),
            vec![Some(Reply::from("vanilla")), None],
            "没答到的那条是否决"
        );
    }
```

- [ ] **Step 2: 跑它们，确认它们失败**

Run: `cargo test -p atomcode-tui withdraw`
Expected: 编译错 —— `no method named 'withdraw' found for struct 'Asks'`

（`cargo test -p atomcode-tui` 这个 crate 只有一个 test binary，跑全量也不贵。）

- [ ] **Step 3: 实现 `withdraw`**

`take_id` 之后：

```rust
    /// 一条问询不必再问了：从队列里拿掉，**不回答**。
    ///
    /// 丢掉 `Pending` 就丢掉了应答通道，等着的那个人看到的是一次取消 —— 既不是
    /// `finish(Vec::new())` 那个「拒绝」，更不是同意。给一条已经作废的问询用（它来自
    /// 的那个会话不在等了），见 `crate::host::Host::withdraw_bg_question`。
    pub fn withdraw(&self, id: u64) -> bool {
        let mut q = self.queue.lock().expect("asks poisoned");
        let before = q.len();
        q.retain(|pending| pending.id != id);
        q.len() != before
    }
```

（不叫醒循环：调用它的就是那个循环，是它自己发现该收的。`take` / `take_id` 同样不叫。）

- [ ] **Step 4: 让 `enqueue` 报出它的 id，并加上带 id 的两条**

`enqueue` 的签名与结尾：

```rust
    fn enqueue(&self, mut asked: Vec<Asked>, reply: Replier) -> u64 {
        // （函数体照旧，只改最后一步）
        let id = self.next.fetch_add(1, Ordering::SeqCst) + 1;
        self.queue.lock().expect("asks poisoned").push(Pending {
            id,
            asked,
            got: Vec::new(),
            reply,
        });
        if let Some(tx) = self.wake.lock().expect("asks poisoned").as_ref() {
            let _ = tx.send(());
        }
        id
    }
```

`push_batch` 之后：

```rust
    /// 一次问询，外加它的 id —— 屏幕要记住自己提上来的是哪一条，好把它收回去。
    pub(crate) fn push_with_id(
        &self,
        asked: impl Into<Asked>,
    ) -> (u64, oneshot::Receiver<Option<Reply>>) {
        let (reply, rx) = oneshot::channel();
        (self.enqueue(vec![asked.into()], Replier::One(reply)), rx)
    }

    /// 同上，一次问几条、一起答。
    pub(crate) fn push_batch_with_id(
        &self,
        asked: Vec<Asked>,
    ) -> (u64, oneshot::Receiver<Vec<Option<Reply>>>) {
        let (reply, rx) = oneshot::channel();
        (self.enqueue(asked, Replier::Many(reply)), rx)
    }
```

`push` / `push_batch` 里的 `self.enqueue(...)` 现在是「有返回值但不接」——不用改，忽略一个非
`#[must_use]` 的返回值不是错。

- [ ] **Step 5: 跑它们，确认它们过**

Run: `cargo test -p atomcode-tui`
Expected: PASS

- [ ] **Step 6: 提交**

```bash
git add crates/atomcode-tui/src/ask.rs
git commit -m "feat(tui): ask 队列能收回一条问询,并报出它的 id

收回去是取消(丢掉应答通道),不是拒绝(finish 空 vec)。屏幕要把
提上来的后台问询收回去时,差的就是这一条。"
```

---

### Task 5: tui —— 屏幕记住「提上来的是哪一条」

**Files:**
- Modify: `crates/atomcode-tui/src/moment.rs`（`resume_preview` 那些字段旁边，`:793-798`）
- Modify: `crates/atomcode-tui/src/host.rs`（后台那一节，`show_bg`（`:3900-3911`）附近）
- Modify: `crates/atomcode-tui/src/plugin.rs`（`AgentClient`，`:248-266` 那些访问子旁边）
- Test: `crates/atomcode-tui/src/host.rs` 的 `mod tests`

**Interfaces:**
- Consumes: Task 4 的 `Asks::push_with_id` / `Asks::withdraw`；现成的 `BgView::sessions()`
  （`bg.rs:116`）、`BgView::slot_of`（`bg.rs:124`）、`bg::Session { id, title, waiting, .. }`
  （`bg.rs:39-49`）、`Host::asks`（`host.rs:1240`）
- Produces（Task 6/7/8 靠它们）：
  - `Moment::bg_asked: Option<(String, Option<u64>)>`
  - `Host::bg_question_wanted(&self) -> Option<String>`
  - `Host::bg_question_shown(&self, session: &str, id: u64) -> bool`
  - `Host::withdraw_bg_question(&self, session: &str) -> bool`
  - `Host::bg_name(&self, session: &str) -> Option<(usize, String)>`
  - `AgentClient::control(&self) -> Option<Arc<dyn HostControl>>`

- [ ] **Step 1: 写失败的测试**

`crates/atomcode-tui/src/host.rs` 的 `mod tests` 里加：

```rust
    /// 一个在等人的后台会话，照面板要的样子。
    fn waiting_bg(session: &str) -> crate::bg::BgView {
        crate::bg::BgView::from_host(vec![atomcode_host_api::BackgroundSession {
            session: session.into(),
            title: Some("review".into()),
            state: atomcode_host_api::BackgroundState::Waiting,
            created_at: 1,
            last: Some("Allow?".into()),
            stats: None,
        }])
    }

    /// 屏幕上有别的问题在等，后台的就先不提 —— 前台优先。
    #[test]
    fn a_background_question_stays_out_while_the_screen_has_one() {
        let h = host();
        assert!(h.show_bg(waiting_bg("b")));
        drop(h.asks.push(atomcode_harness::seams::Question::plain(
            "Allow?",
            &["yes", "no"],
        )));
        assert_eq!(h.bg_question_wanted(), None, "前台有问询，后台的排队");

        // 前台那个答掉、队列空了，才轮到它。
        h.asks.refuse_all();
        assert_eq!(h.bg_question_wanted().as_deref(), Some("b"));
        assert_eq!(h.bg_question_wanted(), None, "已经在取，不再要第二次");
    }

    /// 提上来的是哪一条要记得住，才收得回去；收回去是取消，不是回答。
    #[tokio::test]
    async fn the_question_that_came_up_can_be_taken_back() {
        let h = host();
        h.show_bg(waiting_bg("b"));
        assert_eq!(h.bg_question_wanted().as_deref(), Some("b"));
        let (id, answer) = h.asks.push_with_id(atomcode_harness::seams::Question::plain(
            "Allow?",
            &["yes", "no"],
        ));
        assert!(h.bg_question_shown("b", id));
        assert_eq!(h.bg_name("b").map(|(slot, _)| slot), Some(1));

        assert!(h.withdraw_bg_question("b"), "收得回去");
        assert!(answer.await.is_err(), "收回是取消，谁也不当它答过了");
        assert_eq!(h.bg_question_wanted().as_deref(), Some("b"), "收回去可以再提");
        assert!(!h.withdraw_bg_question("z"), "不是它就没有可收的");
    }
```

- [ ] **Step 2: 跑它们，确认它们失败**

Run: `cargo test -p atomcode-tui --lib a_background_question_stays_out the_question_that_came_up`
Expected: 编译错 —— `no method named 'bg_question_wanted' found for struct 'Host'`

- [ ] **Step 3: 加 `Moment` 的字段**

`crates/atomcode-tui/src/moment.rs` 里 `pub resume_preview: ...`（`:798`）之后：

```rust
    /// 从后台提上来的那条问询：哪个会话问的，以及它在 `Asks` 里的 id。
    ///
    /// `Some((session, None))` 是**正在取**（事实还在路上），`Some((session, Some(id)))`
    /// 是已经在屏幕上。两件事共用一条状态，就没有「在屏幕上但不知道是哪一条」的中间态
    /// —— 形状与 `resume_preview` 一样。
    pub bg_asked: Option<(String, Option<u64>)>,
```

- [ ] **Step 4: 加 Host 的三条访问子与一个称呼**

`crates/atomcode-tui/src/host.rs`，`show_bg`（`:3900`）之后：

```rust
    /// 要不要把一个后台会话的问询提上来 —— 提谁的。
    ///
    /// 只在屏幕上**没有**别的问询时要：前台优先（设计 §6）。同一时刻只提一条，
    /// 已经在取、或已经提上来的不再要第二次。
    pub fn bg_question_wanted(&self) -> Option<String> {
        if self.asks.is_waiting() {
            return None;
        }
        let mut m = self.moment.write().expect("moment poisoned");
        if m.bg_asked.is_some() {
            return None;
        }
        let session = m.bg.sessions().iter().find(|s| s.waiting)?.id.clone();
        m.bg_asked = Some((session.clone(), None));
        Some(session)
    }

    /// 那条问询现在在屏幕上了，带着它的 ask id。
    pub fn bg_question_shown(&self, session: &str, id: u64) -> bool {
        let mut m = self.moment.write().expect("moment poisoned");
        match m.bg_asked.as_mut() {
            Some((shown, slot)) if shown == session => {
                *slot = Some(id);
                true
            }
            _ => false,
        }
    }

    /// 那条问询不必再问了：从屏幕上收回去。
    ///
    /// 收回**不是**回答：`Asks::withdraw` 丢掉应答通道，等着的那个人看到的是取消
    /// （`crate::ask`）。屏幕只是不再问它 —— 谁也没替那个后台会话下结论。
    pub fn withdraw_bg_question(&self, session: &str) -> bool {
        let ask = {
            let mut m = self.moment.write().expect("moment poisoned");
            match m.bg_asked.take() {
                Some((shown, ask)) if shown == session => ask,
                Some(other) => {
                    m.bg_asked = Some(other);
                    return false;
                }
                None => return false,
            }
        };
        match ask {
            Some(id) => self.asks.withdraw(id),
            // 还在取：没有可收的，把「在取」这件事忘掉就是全部。
            None => true,
        }
    }

    /// 那个后台会话在列表里怎么称呼：第几个、叫什么。提它的问询时要说得出是谁在问。
    pub fn bg_name(&self, session: &str) -> Option<(usize, String)> {
        let m = self.moment.read().expect("moment poisoned");
        let slot = m.bg.slot_of(session)?;
        let title = m
            .bg
            .sessions()
            .iter()
            .find(|s| s.id == session)?
            .title
            .clone();
        Some((slot, title))
    }
```

- [ ] **Step 5: 让客户端交出宿主**

`crates/atomcode-tui/src/plugin.rs`，`AgentClient::root()`（`:253`）附近：

```rust
    /// The host as this screen holds it — for the commands that are about a session
    /// other than the one on screen (`/bg`).
    pub fn control(&self) -> Option<Arc<dyn HostControl>> {
        self.link
            .lock()
            .expect("client poisoned")
            .as_ref()
            .map(|link| link.control.clone())
    }
```

（`HostControl` 已在该文件的作用域里 —— 它就在 `Link`（`:175-178`）的字段类型上。）

- [ ] **Step 6: 跑它们，确认它们过**

Run: `cargo test -p atomcode-tui`
Expected: PASS

- [ ] **Step 7: 提交**

```bash
git add crates/atomcode-tui/src/moment.rs crates/atomcode-tui/src/host.rs crates/atomcode-tui/src/plugin.rs
git commit -m "feat(tui): 屏幕记住从后台提上来的是哪一条问询

bg_asked 与 resume_preview 同形状(在取 / 已在屏幕上),外加三条访问子:
要不要提、提上来了、收回去。收回去走 Asks::withdraw —— 是取消。"
```

---

### Task 6: tui —— 把问询提上来、照样画、答完送回

**Files:**
- Modify: `crates/atomcode-tui/src/plugin.rs`
  - `fn pour_bg_question`（放在 `fetch_resume_preview`（`:2815-2839`）之后，与它是同一类东西）
  - 自由函数 `fn as_background_question`（放在 `images_reach_the_model`（`:6661`）这类自由函数旁边）
  - `HostEvent::BackgroundChanged` 那个臂（`:1483-1489`）
  - 「一个问题在屏幕上」那个键臂（`:2117-2120`）
- Modify: `crates/atomcode-i18n/src/screen/{messages.rs,zh_cn.rs,en.rs}`（`Msg::BgAsker`）
- Test: `crates/atomcode-cli/tests/tui_bg.rs`

**Interfaces:**
- Consumes: Task 1 的两条命令；Task 5 的 `Host::bg_question_wanted` / `bg_question_shown` /
  `bg_name` / `withdraw_bg_question`、`AgentClient::control()`；Task 4 的 `push_with_id` /
  `push_batch_with_id`；现成的 `crate::ask::{question_for, batch_for, response_for, declinable}`
  （`ask.rs:377`、`:545`、plugin.rs `:4031`）、`Host::say`（`plugin.rs:3072` 在用）、`Wake::Fact`

- [ ] **Step 1: 写失败的测试**

`crates/atomcode-cli/tests/tui_bg.rs` 末尾加：

```rust
/// 后台会话的问询**直接出现在前台**,人不用先 `/bg 1` 切过去 —— 而且那个会话还在后台。
#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial(atomcode_home)]
async fn a_background_question_comes_out_on_the_foreground() {
    let rig = Rig::new().await;
    rig.term.type_line("/background ask me");
    rig.until_background("the background session is waiting", |list| {
        list.first()
            .is_some_and(|s| s.state == BackgroundState::Waiting)
    })
    .await;

    let list = rig.background().await;
    let title = list[0].title.clone().unwrap_or_default();
    // 屏幕上那句话得说得出是谁在问 —— 没有这一句,人会以为是自己那段对话在问。
    let who = t(Msg::BgAsker {
        slot: 1,
        title: &title,
    });
    rig.until_screen(&who).await;

    // 而且它没有被换到前台:人还留在自己那段对话里(设计 §4 的目标边界)。
    assert_eq!(
        rig.background().await.first().map(|s| s.session.clone()),
        Some(list[0].session.clone()),
        "它还在后台"
    );
}
```

- [ ] **Step 2: 跑它，确认它失败**

Run: `cargo test -p atomcode-cli --test tui_bg a_background_question_comes_out_on_the_foreground`
Expected: `until_screen` 超时（屏幕上只有那句 `/bg 1 打开` 的提示行）。

- [ ] **Step 3: 加 i18n 那句「谁在问」**

`messages.rs`：`BgWaitingTip { slot, title },` 旁边：

```rust
    /// 提上来的那个问询是谁在问：第几个后台会话、它叫什么。
    BgAsker { slot: usize, title: &'a str },
```

`zh_cn.rs`：

```rust
        Msg::BgAsker { slot, title } => format!("后台 [{slot}] {title} 在问").into(),
```

`en.rs`：

```rust
        Msg::BgAsker { slot, title } => format!("background [{slot}] {title} is asking").into(),
```

（i18n 文件里别人有在飞改动的处理方式见 Task 3 Step 3 的说明。）

- [ ] **Step 4: 加前缀那个自由函数**

`crates/atomcode-tui/src/plugin.rs` 里，`fn images_reach_the_model`（`:6661`）旁边：

```rust
/// 把一个后台会话的问询放上屏幕时说清是谁在问。
///
/// 没有这一句，人在自己那段对话底下看到一个「允许写文件吗」会以为是自己触发的。
/// 前缀式，不覆盖原来的问者：审批卡片上那个名字（是哪一个闸门在问）也有用。
fn as_background_question(mut asked: crate::ask::Asked, who: String) -> crate::ask::Asked {
    asked.question.asker = Some(match asked.question.asker.take() {
        Some(asker) if !asker.trim().is_empty() => format!("{who} · {asker}"),
        _ => who,
    });
    asked
}
```

- [ ] **Step 5: 写提上来那一趟**

`crates/atomcode-tui/src/plugin.rs` 里 `fn fetch_resume_preview`（`:2819`）之后：

```rust
    /// 把一个后台会话问询提到前台来：按需取回它的载荷与那个会话的日志尾巴，照屏幕上
    /// 一样的面板画出来，答完原样送回那个 runtime（`HostCommand::AnswerBackground`）。
    ///
    /// 只在屏幕上没有别的问询时才提（`Host::bg_question_wanted`，前台优先）。这一趟
    /// 无论成不成，「正在取」这件事都要有个了结，否则下一次没人再提它。
    fn pour_bg_question(&self) {
        let Some(session) = self.host.bg_question_wanted() else {
            return;
        };
        let Some(control) = self.client.control() else {
            return;
        };
        let host = self.host.clone();
        let keys = self.wake.lock().expect("wake poisoned").clone();
        tokio::spawn(async move {
            let reply = control
                .call(atomcode_host_api::HostCommand::BackgroundQuestion {
                    target: session.clone(),
                })
                .await;
            let Ok(atomcode_host_api::HostReply::BackgroundQuestion {
                id,
                kind,
                payload,
                facts,
                ..
            }) = reply
            else {
                // 不在等了，或这个宿主不认识这条命令：当作没要过。屏幕上照旧只有那
                // 一行提示（设计 §5.3）—— 不崩，也不假装提上来了。
                host.withdraw_bg_question(&session);
                return;
            };
            let events: Vec<LoggedEvent> = facts
                .iter()
                .map(|fact| LoggedEvent {
                    seq: fact.seq,
                    at: fact.at,
                    event: fact.event.clone(),
                })
                .collect();
            let who = match host.bg_name(&session) {
                Some((slot, title)) => t(Msg::BgAsker {
                    slot,
                    title: &title,
                })
                .into_owned(),
                None => session.clone(),
            };
            // 一次问几条：与屏幕自己那个会话的批问询走同一套（`crate::ask::batch_for`）。
            if let Some(questions) = crate::ask::batch_for(&kind, &payload, &events) {
                let n = questions.len();
                let questions = questions
                    .into_iter()
                    .map(|one| as_background_question(one, who.clone()))
                    .collect();
                let (ask, answer) = host.asks.push_batch_with_id(questions);
                host.bg_question_shown(&session, ask);
                if let Some(keys) = &keys {
                    let _ = keys.send(Wake::Fact);
                }
                let target = session.clone();
                tokio::spawn(async move {
                    // 收回去的那些：接收端是取消，什么都不发。
                    let Ok(mut replies) = answer.await else {
                        return;
                    };
                    replies.resize(n, None);
                    let answers: Vec<Value> =
                        replies.into_iter().map(crate::ask::declinable).collect();
                    let _ = control
                        .call(atomcode_host_api::HostCommand::AnswerBackground {
                            target,
                            id,
                            value: serde_json::json!({ "responses": answers }),
                        })
                        .await;
                });
                return;
            }
            let Some(asked) = crate::ask::question_for(&kind, &payload, &events) else {
                // 这个屏幕画不出来：照前台的规矩 fail-closed 回 Null（不回，那个后台
                // 会话就永远挂着），并说一声 —— 不说，人只会看到它莫名其妙不动了。
                let _ = control
                    .call(atomcode_host_api::HostCommand::AnswerBackground {
                        target: session.clone(),
                        id,
                        value: Value::Null,
                    })
                    .await;
                host.withdraw_bg_question(&session);
                host.say(t(Msg::BgQuestionUnanswerable).into_owned(), false);
                if let Some(keys) = &keys {
                    let _ = keys.send(Wake::Fact);
                }
                return;
            };
            let (ask, answer) = host
                .asks
                .push_with_id(as_background_question(asked, who));
            host.bg_question_shown(&session, ask);
            if let Some(keys) = &keys {
                let _ = keys.send(Wake::Fact);
            }
            let target = session.clone();
            tokio::spawn(async move {
                let Ok(chosen) = answer.await else {
                    return;
                };
                let value = crate::ask::response_for(&kind, chosen);
                let _ = control
                    .call(atomcode_host_api::HostCommand::AnswerBackground {
                        target,
                        id,
                        value,
                    })
                    .await;
            });
        });
    }
```

> `Msg::BgQuestionUnanswerable` 那条 i18n 串在 Task 7 加；本任务先把它**留成编译不过**是不行的
> —— 所以本任务先加它：`messages.rs` 里 `BgAsker { .. }` 旁边加 `BgQuestionUnanswerable,`，
> `zh_cn.rs` 加 `Msg::BgQuestionUnanswerable => "后台那个问询这个屏幕画不出来,已按拒绝答复".into(),`，
> `en.rs` 加 `Msg::BgQuestionUnanswerable => "this screen cannot draw that background question — it was answered as a refusal".into(),`。
> Task 7 只是给它补一条判据，不再加串。

- [ ] **Step 6: 在事件与按键两处叫它**

`HostEvent::BackgroundChanged` 那个臂（`:1483-1489`）里，`show_bg` 之后：

```rust
                    self.pour_bg_question();
```

「一个问题在屏幕上」那个键臂（`:2117-2120`）里，答完之后：

```rust
                Wake::Input(Input::Key(press)) if self.host.asks.is_waiting() => {
                    quit = self.answer_question(press);
                    // 答完了可能腾出位置来：前台优先，前台的空位一出现就轮到后台的。
                    self.pour_bg_question();
                    stale = true;
                }
```

- [ ] **Step 7: 跑它，确认它过**

Run: `cargo test -p atomcode-cli --test tui_bg a_background_question_comes_out_on_the_foreground`
Expected: PASS

- [ ] **Step 8: 跑这两处涉及的全量，确认没踩到别人**

Run: `cargo test -p atomcode-cli --test tui_bg && cargo test -p atomcode-tui`
Expected: PASS（既有的「后台在等你回答时前台提示一行」那条也要照旧过 —— 提示行与问询
不能同时出现，Task 8 会把它钉死）

- [ ] **Step 9: 提交**

```bash
git add crates/atomcode-tui/src/plugin.rs crates/atomcode-i18n/src/screen
git commit -m "feat(tui): 后台的问询在前台直接出来,就地答完送回那个会话

前台队列空着时才提(前台优先);画的是同一套面板,只是说清是哪个后台
会话在问;答完走 AnswerBackground 送进那个 runtime,人留在原处。"
```

---

### Task 7: 批问询的判据 —— 一次问几条，提上来的就是那几条

**Files:**
- Modify: `crates/atomcode-cli/tests/tui_bg.rs`（`Replay` 那个 provider 的脚本分支（`:68-82`），再加一条判据）

**Interfaces:**
- Consumes: Task 6 的 `pour_bg_question`（它里面的 `crate::ask::batch_for` 分支）
- 用到的现成件：批的载荷形状 `{"questions":[{header,question,mode,options}]}`（判据在
  `crates/atomcode-tui/src/ask.rs:1276-1334`，形状的另一个例子在
  `crates/atomcode-capabilities/src/tools/request_user_input.rs:778-789`）；答复形状
  `{"responses":[...]}`（`declinable`，`ask.rs:1322-1327` 已判）

> Task 6 已经把批那一支写进 `pour_bg_question` 了。这个任务补的是它的判据：**先把判据跑红
> 一次**，再恢复。红的手法写在 Step 3。

- [ ] **Step 1: 给脚本加一个「一次问两条」的分支**

`crates/atomcode-cli/tests/tui_bg.rs` 里，`"ask me"` 那个分支（`:70-82`）之后：

```rust
        // `ask two`: 一次问两条 —— 批问询走的是 `{"questions": [...]}`
        // (`crate::ask::batch_for`,形状见 `crates/atomcode-tui/src/ask.rs:1276-1334`)。
        if !tools.is_empty() && last.is_some_and(|m| m.role == Role::User && m.text == "ask two") {
            return Ok(Box::pin(futures::stream::iter(vec![
                StreamEvent::ToolCall(ToolCall {
                    id: "call-ask-two".into(),
                    name: "request_user_input".into(),
                    arguments: serde_json::json!({
                        "questions": [
                            {
                                "header": "Flavour",
                                "question": "Which one?",
                                "mode": "single",
                                "options": [{ "label": "vanilla" }, { "label": "pistachio" }],
                            },
                            {
                                "header": "Count",
                                "question": "How many?",
                                "mode": "text",
                            },
                        ],
                    }),
                }),
            ])));
        }
```

- [ ] **Step 2: 写判据**

同一个文件末尾：

```rust
/// 后台一次问两条，提到前台的**是那两条**（第一页先出），而不是只把第一条当成一条单问询。
#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial(atomcode_home)]
async fn a_background_batch_comes_out_as_a_batch() {
    let rig = Rig::new().await;
    rig.term.type_line("/background ask two");
    rig.until_background("the background session is waiting", |list| {
        list.first()
            .is_some_and(|s| s.state == BackgroundState::Waiting)
    })
    .await;

    let title = rig.background().await[0]
        .title
        .clone()
        .unwrap_or_default();
    rig.until_screen(&t(Msg::BgAsker {
        slot: 1,
        title: &title,
    }))
    .await;
    // 批是一块面板、一页一条：第一条在屏幕上就说明它整批都提上来了
    // （只提第一条的话，`batch_for` 根本不会被走到）。
    rig.until_screen("Which one?").await;
}
```

- [ ] **Step 3: 先跑红一次，确认这条判据抓得住**

Run: `cargo test -p atomcode-cli --test tui_bg a_background_batch_comes_out_as_a_batch`
Expected: PASS

然后故意弄红：把 `pour_bg_question` 里 `if let Some(questions) = crate::ask::batch_for(...) {`
那一支的整个 body 换成 `return;`，再跑一次 ——
Expected: FAIL（`until_screen("Which one?")` 等不到：批问询被丢掉了）。恢复代码，再跑回 PASS。

> 这一条不是「先红后绿」的新代码（代码在 Task 6），是**回归判据**：它红了说明那一支真的没在
> 干活。故意弄红那一步就是它的证明。

- [ ] **Step 4: 已知未判到的一处（写在这里，别假装判过）**

「这个屏幕画不出来 → fail-closed 回 `Null` + 留一行」那一条**本计划没有判据**：脚本里的
provider 只能发出这个屏幕认识的问法（`ask me` / `ask two` / 审批 / 两个 kernel checkpoint），
拿不出一个「kind 摆在那儿但画不出来」的请求。要判它得在
`crates/atomcode-tui/tests/e2e.rs` 里照 `ModeHost`（`:8260`）那个假宿主的样子做一个：
`BackgroundQuestion` 回一个 `kind` 谁都不认识、载荷里没有词的答复，然后断言屏幕上出现
`Msg::BgQuestionUnanswerable` 那一行、并且 `AnswerBackground` 被用 `Value::Null` 调过。
**留给下一个人**：这条路径的代码在 Task 6 里，行为按设计 §7 那张表；没判就是没判。

- [ ] **Step 5: 提交**

```bash
git add crates/atomcode-cli/tests/tui_bg.rs
git commit -m "test(cli): 后台一次问两条,提到前台的就是那两条"
```

---

### Task 8: tui —— 它不再等了，就把问询收回去；并且让提示行让位

**Files:**
- Modify: `crates/atomcode-tui/src/plugin.rs`（`HostEvent::BackgroundChanged` 那个臂）
- Modify: `crates/atomcode-tui/src/host.rs`（`withdraw_bg_question` 之后加 `drop_stale_bg_question`）
- Modify: `crates/atomcode-tui/src/modules/tip.rs`（`:106` 那个 `None` 臂之前加一条带条件的臂）
- Test: `crates/atomcode-tui/src/host.rs` 与 `crates/atomcode-tui/src/modules/tip.rs` 的 `mod tests`

**Interfaces:**
- Consumes: Task 5 的 `withdraw_bg_question`、`Moment::bg_asked`；现成的 `HostEvent::BackgroundChanged`
  （`plugin.rs:1483`）、`bg::BgView`、`Tip::render` 的判据装置 `draw_at(&Moment, w, h)`
  （`modules/tip.rs:146-149`）
- Produces: `Host::drop_stale_bg_question(&self) -> bool`

- [ ] **Step 1: 写失败的测试（回收）**

`crates/atomcode-tui/src/host.rs` 的 `mod tests` 里：

```rust
    /// 它不再等了（被丢了、被换到前台、回合结束、被取消、失败），屏幕就把它收回去。
    #[tokio::test]
    async fn a_question_whose_session_stopped_waiting_is_taken_back() {
        let h = host();
        h.show_bg(waiting_bg("b"));
        assert_eq!(h.bg_question_wanted().as_deref(), Some("b"));
        let (id, answer) = h.asks.push_with_id(atomcode_harness::seams::Question::plain(
            "Allow?",
            &["yes", "no"],
        ));
        assert!(h.bg_question_shown("b", id));

        // 列表变了：它不在后台了。
        assert!(h.show_bg(crate::bg::BgView::new(Vec::new())));
        assert!(h.drop_stale_bg_question(), "收回去");
        assert!(answer.await.is_err(), "收回是取消，不是拒绝");
        assert!(!h.drop_stale_bg_question(), "已经没有了，第二次无事");

        // 换成另一个还在等的会话：它在等，凭什么收。
        h.show_bg(waiting_bg("c"));
        assert_eq!(h.bg_question_wanted().as_deref(), Some("c"));
        assert!(!h.drop_stale_bg_question());
    }
```

- [ ] **Step 2: 写失败的测试（提示行让位）**

`crates/atomcode-tui/src/modules/tip.rs` 的 `mod tests` 里（用现成的 `draw_at`）：

```rust
    /// 提上来的问询就在屏幕上时，那一行「/bg N 打开」不再来指路 —— 屏幕已经替他做了。
    #[test]
    fn the_waiting_tip_stands_down_while_its_question_is_on_screen() {
        let mut moment = Moment {
            bg: crate::bg::BgView::new(vec![crate::bg::Session {
                id: "b".into(),
                title: "review".into(),
                group: crate::bg::Group::NeedsInput,
                last: Some("Allow?".into()),
                waiting: true,
                stats: None,
            }]),
            ..Moment::default()
        };
        let tip = crate::i18n::t(crate::i18n::Msg::BgWaitingTip {
            slot: 1,
            title: "review",
        })
        .into_owned();
        assert!(
            draw_at(&moment, 80, 3).iter().any(|l| l.plain().contains(&tip)),
            "没提上来时它在那儿"
        );

        moment.bg_asked = Some(("b".into(), Some(1)));
        assert!(
            !draw_at(&moment, 80, 3)
                .iter()
                .any(|l| l.plain().contains(&tip)),
            "问询已经在屏幕上，这一行不该还在"
        );
    }
```

- [ ] **Step 3: 跑它们，确认它们失败**

Run: `cargo test -p atomcode-tui --lib a_question_whose_session_stopped_waiting the_waiting_tip_stands_down`
Expected: 编译错 —— `no method named 'drop_stale_bg_question' found for struct 'Host'`

- [ ] **Step 4: 实现回收**

`crates/atomcode-tui/src/host.rs`，`withdraw_bg_question` 之后：

```rust
    /// 提上来的那条问询还站得住吗：它来自的那个会话还在等人吗。
    ///
    /// 列表每次一变就问一遍。丢了、被换到前台、回合结束、被取消、失败 —— 这五种都推
    /// `BackgroundChanged`，所以这一个挂点就把设计 §7 那张表全罩住了。
    pub fn drop_stale_bg_question(&self) -> bool {
        let session = {
            let m = self.moment.read().expect("moment poisoned");
            match m.bg_asked.as_ref() {
                Some((session, _)) => session.clone(),
                None => return false,
            }
        };
        let still_waiting = self
            .moment
            .read()
            .expect("moment poisoned")
            .bg
            .sessions()
            .iter()
            .any(|s| s.id == session && s.waiting);
        match still_waiting {
            true => false,
            false => self.withdraw_bg_question(&session),
        }
    }
```

- [ ] **Step 5: 在列表变化那个臂里叫它**

`crates/atomcode-tui/src/plugin.rs`，`HostEvent::BackgroundChanged` 那个臂：

```rust
                    stale |= self.host.show_bg(crate::bg::BgView::from_host(sessions));
                    // 提上来的那条还站得住吗 —— 列表变了就是问它的时机。
                    stale |= self.host.drop_stale_bg_question();
                    self.pour_bg_question();
```

（`pour_bg_question` 是 Task 6 加的那一行，保持它在最后。）

- [ ] **Step 6: 让提示行让位**

`crates/atomcode-tui/src/modules/tip.rs`，`:106` 那个 `None => match vp.moment.bg.waiting_caption()` 之前：

```rust
            // 提示行与提上来的问询互斥：问询已经在屏幕上，这一行再说「/bg N 打开」
            // 是让人去做一件屏幕已经替他做了的事。退回剪贴板那一句。
            None if vp.moment.bg_asked.is_some() => (
                vp.moment.clipboard_caption().unwrap_or_default(),
                Role::Muted,
            ),
```

- [ ] **Step 7: 跑它们，确认它们过**

Run: `cargo test -p atomcode-tui`
Expected: PASS（含 `tip_conformance` 那条形态判据）

- [ ] **Step 8: 提交**

```bash
git add crates/atomcode-tui/src/plugin.rs crates/atomcode-tui/src/host.rs crates/atomcode-tui/src/modules/tip.rs
git commit -m "feat(tui): 后台那个问询不再等时就收回,提示行同时让位

列表一变就问一遍「它还在等人吗」——丢了、换到前台、回合结束、被取消、
失败都推 BackgroundChanged,一个挂点罩住五种。收回是取消:谁也不当
它答过了。"
```

---

### Task 9: 端到端 —— 在前台答的那一下，真的到了那个后台会话

**Files:**
- Test: `crates/atomcode-cli/tests/tui_bg.rs`

**Interfaces:**
- Consumes: 前面全部；现成的 `Rig`、`rig.term.press`、`rig.until_screen`、`rig.until_background`

- [ ] **Step 1: 写判据**

```rust
/// **在前台回答后台的问询，答案真的到了那个会话**：它接着跑，人没有离开自己那段对话。
///
/// 这是设计 §5.3 那条路由的判据 —— Task 3 只判到「挂着的被答掉了」，判不到答案是
/// 不是进了**那个 runtime**；这一条靠那个会话自己的下一个回合来证明。
#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial(atomcode_home)]
async fn answering_it_from_the_foreground_sends_the_answer_to_that_session() {
    let rig = Rig::new().await;
    rig.term.type_line("/background ask me");
    rig.until_background("the background session is waiting", |list| {
        list.first()
            .is_some_and(|s| s.state == BackgroundState::Waiting)
    })
    .await;
    let title = rig.background().await[0]
        .title
        .clone()
        .unwrap_or_default();

    // 它在当前这块屏上等着 —— 不用 /bg 切过去。
    rig.until_screen("Which one?").await;
    rig.until_screen(&t(Msg::BgAsker {
        slot: 1,
        title: &title,
    }))
    .await;

    // 就地答掉：面板亮着第一行，enter 取它（`answer_question` 的规矩）。
    rig.term.press(KeyPress::plain(Key::Enter));

    // 它接着跑：不在等人了。答案真进了那个 runtime，才会这样。
    rig.until_background("the background session moved on", |list| {
        list.first()
            .is_some_and(|s| s.state != BackgroundState::Waiting)
    })
    .await;

    // 而人还留在自己那段对话里：后面还能照常打字。
    rig.term.type_line("still here");
    rig.until_screen("still here").await;
}
```

若 `Key` 没在文件顶部的 `use` 里，加上（`use atomcode_tui::surface::{Key, KeyPress};`）。

- [ ] **Step 2: 跑它，确认它过**

Run: `cargo test -p atomcode-cli --test tui_bg answering_it_from_the_foreground_sends_the_answer_to_that_session`
Expected: PASS

若 `until_screen("Which one?")` 这一步等不到，先看 Task 6 的 `pour_bg_question` 是不是真的在
`BackgroundChanged` 那个臂里被叫到了（`host.say` 与 `Wake::Fact` 那两处也在里面）。

- [ ] **Step 3: 反向判据：宿主不认识这条命令时，屏幕照旧**

**这一条本计划没有判据**，与 Task 7 Step 4 同一个原因：要一个假宿主。做法是照
`crates/atomcode-tui/tests/e2e.rs:8260` 的 `ModeHost` 做一个：`BackgroundQuestion` 回
`Err(HostError::Failed { message: "this host does not do that yet".into() })`，断言屏幕上
**只有** `Msg::BgWaitingTip` 那一行、没有 `Msg::BgAsker`，而且没有任何 `AnswerBackground`
被发出去。**留给下一个人**，与那一条一起做。

- [ ] **Step 4: 跑这个文件全量**

Run: `cargo test -p atomcode-cli --test tui_bg`
Expected: PASS

- [ ] **Step 5: 提交**

```bash
git add crates/atomcode-cli/tests/tui_bg.rs
git commit -m "test(cli): 前台答的那一下,真的到了那个后台会话"
```

---

## 收尾：全部做完之前要跑的三件事

```bash
bash gates/compile.sh                              # 按 crate 跑看不见那两处枚举漏臂
cargo test -p atomcode-host-api -p atomcode-cli -p atomcode-tui
cargo fmt --all -- --check                          # 必须退出 0
```

## 本计划的自审（写完这份计划后做的检查）

- **spec 覆盖**：设计 §1-§2 是问题与被否方向（无需任务）；§4 owner 由 Task 5 的 `bg_asked`
  与 Task 1 的命令面落实；§5.1/§5.2/§5.3 是 Task 1-3、5-6；§6 仲裁是 Task 5 的
  `bg_question_wanted` + Task 6 的两处调用点；§7 那张表逐行落在 Task 3（NotFound / 对不上）、
  Task 6（画不出来回 `Null` + 一行）、Task 8（五种作废）；§8 清单逐项都有任务；§9 的判据一一
  对应（**除两处已具名未判**：画不出来的那条路径、宿主不认识命令时的退回，两条都要假宿主）。
- **占位扫查**：没有 TBD/TODO；两处「未判到」是具名缺口，写明了做法与位置，不是待办占位。
- **类型一致**：`LoggedFact { seq, at, event }`、`BackgroundQuestion { session, id, kind, payload, facts }`、
  `AnswerBackground { target, id, value }`、`Asks::push_with_id / push_batch_with_id / withdraw`、
  `Host::bg_question_wanted / bg_question_shown / withdraw_bg_question / drop_stale_bg_question / bg_name`
  —— 后文用到的名字与前文定义的逐个一致。`id` 用 kernel 的 `RequestId`（= `u64` 别名，
  `crates/atomcode-kernel/src/event.rs:18`）。
