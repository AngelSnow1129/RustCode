# 多 Agent 协作开发与项目推进方案

本文档定义一套可直接在 CodeBuddy（IDE / CLI）落地的多 Agent 协作流水线：
**需求收集 → 架构设计与任务拆分 → 并行开发 → 集成测试 → 交付**。

配套产物：

| 文件 | 作用 |
|---|---|
| `.codebuddy/agents/*.md` | 7 个 Agent 定义（CodeBuddy 可直接识别并加载） |
| `.codebuddy/rules/multi-agent-workflow.md` | 协作协议，随会话自动加载，约束主 Agent 与各子 Agent |
| `.codebuddy/artifacts/<feature-slug>/` | 运行期交接件（handoff）与任务看板 |

---

## 1. 设计前提与硬约束

1. **单一编排者（Orchestrator）**
   CodeBuddy 中 `agentMode: agentic` 的子 Agent 由主 Agent（Craft）自动调度，**子 Agent 之间不能直接互调**。
   因此所有跨 Agent 通信一律经过「落盘交接件 + 编排者路由」。编排者由 `project-manager`（manual 模式，用户手动选中）或主 Craft Agent 承担。
   > 若使用 CodeBuddy CLI 的 Agent Teams，可用 `SendMessage` 建立第二条消息通道，但**交接件仍是唯一权威事实源**，消息只做通知与同步。

2. **文件即消息（File-as-Message）**
   每个 Agent 的产出必须是 `.codebuddy/artifacts/<feature-slug>/` 下带 YAML 信封的 Markdown 文件。
   不在对话里口头传递结论；对话中只传递「文件路径 + 状态 + 决策」。

3. **最小权限（Least Privilege）**
   每个 Agent 只授予完成本职工作必需的工具。审查/分析类 Agent 不授予 `Write`/`Edit`；文档类 Agent 不授予 `Bash`；执行写操作与命令的 Agent 默认关闭 `enabledAutoRun`，需人工确认。

4. **契约先行（Contract-First）**
   并行开发只允许发生在「已冻结的接口契约之下」。契约未定的模块不允许并行，必须先回到 `solution-architect` 冻结签名、trait、事件与错误类型。

5. **门禁不可跳过（Gate Cannot Be Skipped）**
   每个阶段有明确出口门禁（G1–G6）。任一门禁未通过，编排者不得推进到下一阶段；返工次数超限则升级给用户。

6. **本项目架构约束**
   所有 Agent 必须遵守仓库根目录 `AGENTS.md`：
   `CodingRuntime` 是 coding agent 唯一运行时所有者；kernel 只负责中立 agent 循环；`rustcode-kernel`/`rustcode-capabilities`/`rustcode-coding` 生产依赖保持 core-free，不得反向依赖 core、L2 或前端；不得恢复 bridge、v1/v2 开关、core session 磁盘模型或任何 fallback。
   交接件中的 `architecture_constraints` 字段必须显式声明本次改动是否触碰上述边界。

---

## 2. 角色总览

| # | Agent | 模式 | 核心职责 | 主要输入 | 主要输出 | 模型 | 工具 |
|---|---|---|---|---|---|---|---|
| 1 | `requirements-analyst` | agentic | 澄清需求、界定范围、产出可测验收标准 | 用户原始诉求、`00-requirement.md` 草稿 | `00-requirement.md` | sonnet | Read, Grep, Glob, WebFetch, WebSearch, AskUserQuestion, Write |
| 2 | `solution-architect` | agentic | 架构设计、接口契约、状态所有权、任务拆分 | `00-requirement.md` | `01-design.md`、`02-tasks.md` | opus | Read, Grep, Glob, WebFetch, Write, LSP, AskUserQuestion |
| 3 | `code-implementer` | agentic | 按契约做最小改动实现并自测 | `02-tasks.md` 中的单个任务 + `01-design.md` | 源码改动 + `03-impl/<task-id>.md` | sonnet | Read, Write, Edit, MultiEdit, Grep, Glob, Bash, LSP, TaskList, TaskGet, TaskUpdate |
| 4 | `code-reviewer` | agentic | 只读审查，产出分级问题清单与裁决 | `03-impl/<task-id>.md` + `git diff` | `04-review/<task-id>.md`、`ReportFindings` | sonnet | Read, Grep, Glob, Bash(只读), LSP, ReportFindings |
| 5 | `test-engineer` | agentic | 补测、执行、回归验证与覆盖率判断 | `01-design.md`、`03-impl/*` | 测试代码 + `05-test-report.md` | sonnet | Read, Write, Edit, MultiEdit, Grep, Glob, Bash, LSP, TaskOutput |
| 6 | `doc-writer` | agentic | 同步文档、CHANGELOG、发布说明 | 已通过门禁的全部交接件 | `06-release.md` + `docs/**` 更新 | sonnet | Read, Grep, Glob, Write, Edit, MultiEdit, WebFetch |
| 7 | `project-manager` | **manual** | 编排、路由、看板、门禁判定、交付 | 全部交接件 | `STATUS.md`、路由决策、交付结论 | opus | Read, Grep, Glob, Write, Edit, MultiEdit, Bash, Agent, TaskCreate/TaskGet/TaskList/TaskUpdate/TaskOutput/TaskStop, SendMessage, AskUserQuestion, EnterWorktree, LeaveWorktree |

> `project-manager` 采用 `manual` 模式：用户在 Agent 选择框中手动选中它，由它接管主会话并调度其余 6 个 agentic Agent，适合端到端推进一个特性。
> 若用户停留在 Craft 模式，主 Agent 会自动依据 `description` 调度 6 个 agentic Agent，编排职责回落到主 Agent。

---

## 3. 产物与目录布局

```text
.codebuddy/
  agents/                      # 7 个 Agent 定义（project 级）
    requirements-analyst.md
    solution-architect.md
    code-implementer.md
    code-reviewer.md
    test-engineer.md
    doc-writer.md
    project-manager.md
  rules/
    multi-agent-workflow.md    # 协作协议（自动加载）
  artifacts/
    <feature-slug>/            # 例：2026-09-02-session-resume
      STATUS.md                # 看板：任务状态、路由、门禁结论（PM 唯一可写）
      00-requirement.md        # requirements-analyst
      01-design.md             # solution-architect
      02-tasks.md              # solution-architect（任务图 + 依赖 + 并行批次）
      03-impl/<task-id>.md     # code-implementer，每任务一份
      04-review/<task-id>.md   # code-reviewer，每任务一份
      05-test-report.md        # test-engineer
      06-release.md            # doc-writer
```

`<feature-slug>` 由编排者在创建时分配，格式 `YYYY-MM-DD-<短名>`；同一次推进内所有 Agent 复用同一 slug。

---

## 4. 消息传递机制

### 4.1 通信通道

| 通道 | 载体 | 适用 | 权威级别 |
|---|---|---|---|
| **文件总线（主）** | `.codebuddy/artifacts/<slug>/*.md` 的 YAML 信封 | 全部跨 Agent 传递 | 权威事实源 |
| **任务总线** | `TaskCreate/TaskUpdate/TaskList`（编排者持有） | 任务状态与依赖 | 派生态，需与 `STATUS.md` 一致 |
| **消息总线（CLI Agent Teams）** | `SendMessage` | 通知、催办、阻塞上报 | 非权威，仅同步 |
| **人工通道** | `AskUserQuestion` | 歧义澄清、门禁裁决 | 最高优先级，可阻塞全流程 |

### 4.2 信封（Envelope）Schema

每个交接件文件头必须是如下 YAML：

```yaml
---
kind: requirement | design | task | implementation | review | test-report | release | clarification
id: T-003                      # 任务/交接件唯一 id
from: code-implementer         # 产出方 Agent 名
to: [code-reviewer]            # 下游 Agent 名（可多个）
feature: 2026-09-02-session-resume
status: draft | ready | in_progress | changes_requested | approved | blocked | done
decision: proceed | rework | escalate | block   # 对下游的建议动作
requires: [T-001, T-002]       # 依赖的上游交接件 id
files_owned:                   # 本任务独占写入的文件（冲突判定依据）
  - crates/rustcode-coding/src/runtime.rs
architecture_constraints:      # 是否触碰 AGENTS.md 架构边界
  touches_runtime_lifecycle: false
  touches_persistence: true
  touches_cross_crate_deps: false
created: 2026-09-02
---
```

### 4.3 消息类型

| kind | 发出方 → 接收方 | 关键载荷 |
|---|---|---|
| `requirement` | analyst → architect | 目标/非目标、用户故事、验收标准（可测）、异常场景、开放问题 |
| `clarification` | 任意 → PM/用户 | 阻塞性歧义 + 候选方案 + 推荐项；阻塞下游 |
| `design` | architect → PM | 方案对比、模块划分、接口契约（签名/trait/事件/错误）、状态所有权、迁移与回退方案 |
| `task` | architect → PM → implementer | 任务图、依赖、并行批次、`files_owned`、每任务验收标准 |
| `implementation` | implementer → reviewer | 改动清单、自测命令与结果、契约偏差说明、遗留风险 |
| `review` | reviewer → implementer/PM | 分级问题（blocker/critical/major/minor/nit）、裁决（approve / request-changes / reject） |
| `test-report` | test-engineer → PM | 新增用例、执行结果、覆盖率变化、回归结论 |
| `release` | doc-writer → PM | 文档更新清单、CHANGELOG、发布说明、交付清单 |

### 4.4 路由规则

| 上游产物 | status / decision | 路由到 | 附加条件 |
|---|---|---|---|
| `requirement` | `approved` | `solution-architect` | 全部开放问题已关闭 |
| `requirement` | `blocked` | 用户（AskUserQuestion） | 存在无法自行消解的歧义 |
| `design` | `approved` | `project-manager`（拆任务/排期） | 契约完整、架构约束已核对 |
| `design` | `blocked` | `requirements-analyst` | 方案导致需求不可行或需缩范围 |
| `task` | `ready` | `code-implementer`（按批次并行） | 依赖任务已 `done`、契约已冻结 |
| `implementation` | `done` | `code-reviewer` | 自测命令已执行且输出已附 |
| `review` | `request-changes` | `code-implementer` | 返工轮次 < 2 |
| `review` | `request-changes`（第 3 次） | 用户 + `project-manager` | 强制升级，禁止无限返工 |
| `review` | `reject` | `solution-architect` | 问题根因在契约/设计 |
| 全部任务 `approved` | — | `test-engineer` | 进入集成测试阶段 |
| `test-report` | 通过 | `doc-writer` | 门禁 G5 达成 |
| `test-report` | 失败 | `code-implementer`（失败用例归属任务） | 带失败用例与最小复现 |
| `release` | `done` | `project-manager` → 用户 | 门禁 G6 达成 |

---

## 5. 任务状态机

```text
                 ┌──────────── escalate（返工 ≥2 或门禁阻塞）──────────┐
                 ▼                                                      │
  pending ──> ready ──> in_progress ──> review ──> testing ──> done     │
     │           │            │             │          │                │
     │           │            │             │          └── 失败 ──┐      │
     │           │            │             │                     ▼      │
     │           │            │             └── changes_requested ──> in_progress
     │           │            ▼                                          │
     └───────────┴──────> blocked（依赖未就绪 / 契约未冻结 / 待澄清）──────┘
```

规则：

1. 只有 `ready` 任务可被 `code-implementer` 领取；一次领取一个任务。
2. `files_owned` 重叠的任务**不得同批次并行**；跨 crate 改动默认串行，除非契约已冻结且文件无交集。
3. 进入 `in_progress` 必须在 `03-impl/<task-id>.md` 先落盘（含计划），再改代码，避免「先写后报」导致状态不可追溯。
4. 任何非 `done` 终态都必须写明 `decision` 与下一跳接收方，禁止无归属的悬挂任务。
5. 每个任务至多 2 次返工；第 3 次必须 `escalate`。

---

## 6. 五阶段工作流

### 阶段一：需求收集（Requirements）

- **参与者**：`project-manager`（建 slug 与 `STATUS.md`）→ `requirements-analyst`
- **活动**：还原用户目标；区分目标/非目标；识别受影响 crate 与现有实现；补齐异常场景、边界条件、非功能需求；把模糊表述转成可测 AC。
- **产物**：`00-requirement.md`
- **出口门禁 G1**：
  - [ ] 每条 AC 可自动化或人工明确判定
  - [ ] 范围边界（非目标）已写明
  - [ ] 开放问题列表为空，或已转为 `clarification` 并获用户答复
  - [ ] 已标注是否触碰持久化格式、公共协议、安全边界、运行时生命周期
- **失败处理**：存在无法消解的歧义 → `blocked` + `AskUserQuestion`，不进入设计阶段。

### 阶段二：架构设计与任务拆分（Design & Breakdown）

- **参与者**：`solution-architect` → `project-manager`
- **活动**：现状勘察（调用方、持久化点、转换边界、近期 Git 历史）；方案对比；确定模块与接口契约；声明状态所有权；评估迁移/回退；拆任务图（依赖 + 并行批次 + `files_owned`）。
- **产物**：`01-design.md`、`02-tasks.md`
- **出口门禁 G2**：
  - [ ] 接口契约（函数签名 / trait / 事件 / 错误类型）已冻结并写入文档
  - [ ] 状态所有权唯一且明确；依赖方向符合 `AGENTS.md`
  - [ ] 失败、取消、重试、降级语义已定义（无静默 fallback、无假成功）
  - [ ] 每个任务粒度 ≤ 1 个 crate 主体 + 明确 `files_owned`，可独立编译验证
  - [ ] 并行批次无文件所有权冲突，且都位于已冻结契约之下
- **失败处理**：方案导致需求不可行 → 回退 `requirements-analyst` 缩范围。

### 阶段三：并行开发（Parallel Implementation）

- **参与者**：`project-manager`（派发）→ N × `code-implementer` → `code-reviewer`
- **活动**：PM 按批次派发 `ready` 任务；实现者做最小改动、遵循契约、改动后跑受影响 crate 的 `cargo test`；产出实现报告；审查者只读审查并裁决。
- **产物**：源码改动、`03-impl/<task-id>.md`、`04-review/<task-id>.md`
- **出口门禁 G3（实现）**：
  - [ ] `cargo check -p <crate> --all-targets` 通过
  - [ ] `cargo test -p <crate>` 通过且输出已附在报告中
  - [ ] 无未经批准的契约变更（若有，必须回退到 `solution-architect`）
  - [ ] 改动范围不超过 `files_owned`
- **出口门禁 G4（审查）**：
  - [ ] 无 `blocker` / `critical`
  - [ ] `major` 已修复或有明确后续项并获 PM 接受
  - [ ] 返工轮次 ≤ 2
- **失败处理**：`request-changes` → 回实现者；`reject` → 回架构；超限 → 升级用户。

### 阶段四：集成测试（Integration & Verification）

- **参与者**：`project-manager`（合批/解冲突）→ `test-engineer`
- **活动**：合并并行分支；补集成测试与回归测试；跑 `cargo test --workspace`（至少受影响 crate 全集）；跨 crate / 公共协议 / 持久化变更时补 `cargo check --workspace --all-targets` 与 `cargo clippy`。
- **产物**：`05-test-report.md`
- **出口门禁 G5**：
  - [ ] 新增/修改路径均有测试覆盖，含失败与取消路径
  - [ ] 受影响 crate 测试全绿；跨 crate 变更时 workspace 检查通过
  - [ ] 已覆盖 `AGENTS.md` 要求的入口矩阵（CLI / TUI / daemon / headless / background / ACP / clix 中实际受影响者）
  - [ ] 无跳过的用例（或有记录的原因与负责人）
- **失败处理**：失败用例按 `files_owned` 归属回退到对应 `code-implementer`，并携带最小复现。

### 阶段五：交付（Delivery）

- **参与者**：`doc-writer` → `project-manager` → 用户
- **活动**：同步 `docs/**`、README、CHANGELOG、公开协议/配置说明；清理临时交接件中的过程噪声，保留结论；整理交付清单与回滚方案。
- **产物**：`06-release.md` 与文档改动
- **出口门禁 G6**：
  - [ ] 行为变化、风险、验证结果、已知未验证范围已写明
  - [ ] 所有受影响文档已同步，无残留 TODO / 占位符
  - [ ] 回滚方案可执行
  - [ ] `STATUS.md` 全部任务终态为 `done`，无悬挂任务
- **失败处理**：任一未满足 → 回退对应责任 Agent，PM 记录阻塞原因。

---

## 7. 并发与冲突控制

| 风险 | 控制手段 |
|---|---|
| 写文件冲突 | 任务级 `files_owned` 独占；同批次任务文件集合必须两两不相交 |
| 契约漂移 | 契约在阶段二冻结；实现者发现必须改契约时禁止自改，回退 `solution-architect` |
| 审查返工死循环 | 至多 2 轮，第 3 轮强制升级 |
| 上下文污染 | agentic 子 Agent 独立上下文窗口；只通过交接件传递结论 |
| 并行度失控 | 单批次并行任务 ≤ 3；跨 crate 改动默认串行 |
| 环境隔离需求 | 需要长时并行/互不干扰时使用 `EnterWorktree` / `isolation: worktree`，结束前 `LeaveWorktree` |
| 迟到结果污染 | 交接件带 `feature` 与 `id`；PM 只接受与当前 slug 匹配且状态最新的产物 |

---

## 8. 失败、返工与升级语义

1. **显式失败优于静默降级**：任何一步失败必须在交接件中记录 `status: blocked` + `decision` + 证据（命令与输出），禁止 noop、假成功、静默 fresh。
2. **返工闭环**：`changes_requested` 必须携带「问题 → 文件:行 → 期望行为 → 验证方式」四元组。
3. **升级路径**：实现/审查/测试任一环节 2 轮未通过 → `project-manager` → 用户决策（缩范围 / 换方案 / 暂停）。
4. **取消与中断**：用户中断或 PM 终止任务时，必须更新 `STATUS.md` 与对应交接件为 `blocked`/终止态，并注明未完成的改动是否需要回滚。
5. **不可变历史**：交接件只允许追加与状态推进，不改写既有结论；修订通过新增小节「修订记录」完成。

---

## 9. 落地与裁剪

1. **安装**：本仓库已自带 `.codebuddy/agents/`，project 级生效，随仓库共享给团队；放到 `~/.codebuddy/agents/` 则全局生效。
2. **字段说明**（CodeBuddy IDE Subagent 规范）：

   | 字段 | 必填 | 说明 |
   |---|---|---|
   | `name` | 是 | 唯一标识，kebab-case |
   | `description` | 是 | 专长 + 范围 + 明确触发条件，主 Agent 依此自动调度 |
   | `model` | 否 | 需为账号可用模型；若 `sonnet`/`opus` 别名不可用，删除该行或在设置页改选（如 `glm-4.6`） |
   | `tools` | 否 | 内置工具白名单，逗号分隔；留空表示继承全部 |
   | `agentMode` | 否 | `agentic`（自动调度）或 `manual`（手动选中，接管主会话） |
   | `enabled` | 否 | 是否启用 |
   | `enabledAutoRun` | 否 | 工具调用是否免确认。写操作/命令执行类 Agent 建议 `false` |

3. **裁剪建议**：小改动（单文件、无契约变化）跳过阶段二，直接 `code-implementer → code-reviewer`；纯文档任务只走 `doc-writer`。
4. **度量**：以「返工轮次、门禁一次通过率、升级次数、并行任务冲突数」衡量流水线健康度，持续收紧 `description` 与门禁。

---

## 10. 已知限制

- IDE 的 agentic 子 Agent **不可中途干预**：触发后只能等待完成或中断整个会话，因此任务粒度必须小、边界必须清晰。
- 子 Agent 之间**不能互发消息**；本方案用文件总线规避，代价是磁盘上会留下交接件（需在交付时归档或清理）。
- `tools` 白名单只能限制工具类别，无法限制写入路径；路径约束由各 Agent 的 system prompt 与 `files_owned` 共同保证，属于软约束，最终由 `code-reviewer` 与 G6 门禁兜底。
- 不同 CodeBuddy 版本对 frontmatter 字段（`effort`/`maxTurns`/`disallowedTools`/`skills`/`isolation` 等，plugin 形态支持）的识别程度不同；本方案只用 IDE 已文档化的字段，保证最大兼容性。
