# 2026-09-03-residual-two-items 看板

- 当前阶段：开发（T1 实现中 / T2 待派发）
- 基线：branch=dev commit=ba863a1a worktree=dirty（仅 1 项：未跟踪 `docs/multi-agent-collaboration-solution.md`）
- 上轮看板：`2026-09-02-g1-fmt-gate`（G1/G2/G3 pass，冻结解除）；`2026-09-02-cleanup-codingplan-legacy`（G6 pass，已提交 783d48e4 并推送）

## 来源

上一轮（commit `ba863a1a`，已推送 origin/dev）收尾时遗留两项未处理，均已升级用户裁决。
用户于 2026-09-03 裁决：**两项都推进** + session_picker 采用**钉定会话名**修法。

## 阶段裁剪决策（依据 `docs/multi-agent-collaboration-solution.md` §9.3）

两项均为「单文件、无接口契约变化」，故**跳过阶段二（架构设计与任务拆分）**，
流程裁剪为 `code-implementer → code-reviewer`（T1）与 `doc-writer`（T2）。
G1 需求直接以**用户裁决 + 根因实证**作为依据，不另派 `requirements-analyst`（范围无歧义）。

## 门禁

| 门禁 | 状态 | 依据 |
|---|---|---|
| G1 需求 | **pass** | 用户裁决（范围=两项；修法=钉定会话名）+ 根因实证见下「T1 根因链」 |
| G2 设计 | **裁剪** | §9.3：单文件、无契约变化，跳过阶段二 |
| G3 实现 | pending | T1：`.codebuddy/artifacts/2026-09-03-residual-two-items/03-impl/T1.md` |
| G4 审查 | pending | T1：`04-review/T1.md` |
| G5 测试 | pending | tuix --lib 全绿 + fmt 门禁复检 |
| G6 交付 | pending | `06-release.md`；**提交动作须用户显式确认**（编排者不得自行 commit/push） |

## 任务

| id | 标题 | 批次 | 负责 Agent | 状态 | 返工轮次 | 交接件 | files_owned |
|---|---|---|---|---|---|---|---|
| T1 | session_picker 用例钉定会话名（消除毫秒时间戳偶发红） | B1 | code-implementer | in_progress | 0 | `03-impl/T1.md` | `crates/rustcode-tuix/src/modals/session_picker.rs` |
| T2 | 多 Agent 协作方案文档校验与入库准备 | B1 | doc-writer | ready | 0 | `06-release.md` | `docs/multi-agent-collaboration-solution.md` |

同批次并行依据：`files_owned` **两两不相交**（T1 仅 tuix 源码，T2 仅 docs 文档），并行任务数 2 ≤ 3。

## T1 根因链（编排者只读勘查，已实证）

1. `crates/rustcode-tuix/src/session.rs:63` — `pub type Session = TuiSession;`
2. `crates/rustcode-tuix/src/session.rs:119-123` — `TuiSession::new` 生成
   `name: format!("session-{now}")`，`now = rustcode_capabilities::session::now_ms()`（毫秒时间戳）
3. `crates/rustcode-tuix/src/modals/session_picker.rs:912-915` — `replay_session` 渲染
   `UiLine::TurnSeparator { label: SessionResumedLabel { name: &session.name } }`
4. `crates/rustcode-tuix/src/modals/session_picker.rs:2166-2169` — 用例断言
   `labels.iter().all(|label| !label.contains("987"))`，其中 `987654` 是「仅记账」哨兵
   `total_tokens`

**结论**：断言本意是「验证 `position_valid: false` 的记账统计未泄漏进 replay 分隔线」，
但判据误用「标签不含子串 987」，而 resumed 标签内含毫秒时间戳数字串。
当该数字串**恰好含 "987"** 时误判失败。属**与时间相关的固有偶发缺陷**，非 locale 问题、
与上一轮改动无关（`AGENTS.md:570` 已登记为待裁决缺陷）。

**修法（用户裁决：钉定会话名）**：在用例内构造 `Session::new` 后把 `session.name` 覆盖为
固定值，消除时间依赖。**不改任何断言、不改生产代码**。
既有同构写法先例：`TuiSession::default_session`（`session.rs:137-141`）构造后覆盖
`session.name = "default".into()`。

## 环境约束（承 `2026-09-02-g1-fmt-gate` 看板）

- cgroup 内存上限 8GB，`cargo test` 必须 `-j 1`，否则 rustc SIGBUS。
- 工具会话超时约 60–90s；长任务须 `setsid nohup ... &` 脱离会话后轮询，**日志路径全局唯一**
  （此前因两进程共用日志污染过一次结果）。
- daemon 测试争用固定端口 13456-13458；启动前确认 `pgrep -c cargo = 0`。
- 子代理通道不稳：此前累计 3 次派发失败（1 次 "No result found" + 2 次 idle timeout），
  **根因是编译期无增量输出触发空闲超时**。本轮已在派发前用
  `cargo test -j 1 -p rustcode-tuix --lib --no-run` 预热（1.96s 缓存命中），
  测试二进制 `target/debug/deps/rustcode_tuix-873dba58ced4d3b8` 就绪。
- `rustcode-cli` 包名为 `rustcode`，`-p rustcode-cli` 会失败。

## 已知红基线（G5 比对基准，不得当作回归）

- `crates/rustcode-capabilities` — `mcp::registry::tests::trust_key_golden_matches_core_algorithm`
  （`AGENTS.md:226` 文档化已知红，`DefaultHasher` 跨工具链不稳定，**铁律禁改**）
- 其余全绿基线：`tuix --lib` 2064/0、`cli(rustcode) --lib` 116/0、`review --lib` 100/0、
  `coding --lib` 430/0、`config --lib` 327/0、`daemon --lib` 307/0、`updater --lib` 41/0

## 边界声明

- T1 **不触碰**运行时生命周期、持久化格式、公共协议、跨 crate 依赖——仅 `#[cfg(test)]`
  用例内的夹具数据。构造函数 `TuiSession::new` 保持原样（生产行为不变）。
- T2 为纯文档入库，**不提交**；提交动作待用户显式确认后由编排者执行。

## 决策日志

| 时间 | 决策 | 依据 |
|---|---|---|
| 2026-09-03 | 新起 slug `2026-09-03-residual-two-items`，承接上轮两项遗留 | 上轮已推送，遗留项需独立跟踪 |
| 2026-09-03 | 跳过阶段二，裁剪为 implementer→reviewer / doc-writer | §9.3：单文件、无契约变化 |
| 2026-09-03 | 用户裁决：两项都推进 + 钉定会话名 | 用户显式选择（推荐项） |
| 2026-09-03 | 派发前预热 tuix 测试二进制 | 消除子代理编译期 idle timeout（历史 3 连败根因） |
