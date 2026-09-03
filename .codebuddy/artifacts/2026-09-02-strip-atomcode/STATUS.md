# 2026-09-02-strip-atomcode 看板

- 当前阶段：需求（G1 前置只读勘查）
- 基线：branch=main commit=287bff70 worktree=dirty
  - dirty 内容：未跟踪 `.codebuddy/`（7 个 Agent 定义 + 协作规则）与 `docs/multi-agent-collaboration-solution.md`
  - 处理：属用户既有改动，保留，禁止重置或覆盖

## 用户原始诉求（逐字记录，未经编排者改写）

> 参考 git 的最新的推进等内容：分析下面的内容。并根据系统的情况进行完善继续迁移，并实现相应的去处 atomcode 的内容，并未不与任何其他的模型有关联，只保留第三方的配置，并去除遥测等内容，并配置默认为中文

编排者拆分出的四条诉求线：

1. 继续并完善既有迁移（依据最新 git 推进方向）
2. 去除 `atomcode` 相关内容（重命名/去品牌面）
3. 不与任何模型厂商绑定，只保留第三方（自定义）配置
4. 去除遥测；默认语言改为中文

## 门禁

| 门禁 | 状态 | 依据 |
|---|---|---|
| G1 需求 | pending | 待 `00-requirement.md` |
| G2 设计 | pending | 待 `01-design.md` / `02-tasks.md` |
| G3 实现 | pending | - |
| G4 审查 | pending | - |
| G5 测试 | pending | - |
| G6 交付 | pending | - |

## 勘查任务（G1 前置，全部只读）

| id | 主题 | 负责 Agent | 状态 | 交接件 |
|---|---|---|---|---|
| R-0a | `atomcode` 命名面清单与迁移方向 | code-explorer | in_progress | 勘查纪要（会话内） |
| R-0b | 遥测/上报/远程端点面清单 | code-explorer | in_progress | 勘查纪要（会话内） |
| R-0c | 模型/提供方耦合与默认语言面清单 | code-explorer | in_progress | 勘查纪要（会话内） |

## 任务

| id | 标题 | 批次 | 负责 Agent | 状态 | 返工轮次 | 交接件 |
|---|---|---|---|---|---|---|
| — | 待 G1 通过后由 02-tasks.md 填充 | — | — | — | — | — |

## 已知阻塞（待用户裁决）

- **B1 命名目标缺失**：去除 `atomcode` 需要一个替换名（产品名 / 二进制名 / crate 前缀 / 配置目录名），此决策只能由用户给出，勘查完成后升级。

## 决策日志

| 时间 | 决策 | 依据 |
|---|---|---|
| 2026-09-02 | q-0 选「续推既有 feature」但无交接件，按新需求启动，slug 由编排者确定 | 无 `.codebuddy/artifacts/<slug>` 目录；q-1 为新诉求文本 |

## [CLOSED] 闭板记录（2026-09-03，只追加，不改写既有行）

- **结论：废弃（superseded），不再推进。**
- 理由：本看板四条诉求线（继续并完善既有迁移 / 去除 `atomcode` / 不与模型厂商绑定只留第三方配置 /
  去遥测且默认中文）已在 `dev` 分支落地，并在 `AGENTS.md` [OBJECTIVE-1]~[OBJECTIVE-6] 逐条记为 [DONE]。
  G1 长期停在 pending 的原因是**诉求已从「待实现」变为「已完成」**，继续推进无对象。
- 勘查任务 R-0a / R-0b / R-0c 三条 `in_progress` 的终态与 decision：

| id | 终态 | decision | 承接依据 |
|---|---|---|---|
| R-0a | 终止 | escalate → 由 OBJECTIVE-1 承接 | 命名面已迁移至 `rustcode-*`，产品身份锁定（决策 D1） |
| R-0b | 终止 | escalate → 由 OBJECTIVE-2 承接 | 遥测 crate 已删除，残留仅守卫性声明（禁「顺手清理」） |
| R-0c | 终止 | escalate → 由 OBJECTIVE-3 / OBJECTIVE-5 承接 | 平台中立与默认中文均已落地 |

- 已知阻塞 B1（命名目标缺失）**已解除**：产品身份为 `rustcode`（决策 D1，`docs/REFACTOR_DESIGN_PHASE1.md` §2.0），
  AGENTS.md 明载「`rustcode` 是已锁定的产品身份，不是中间态，不要再次改名」。
- **承接看板**：`2026-09-03-git-wrapup`。本看板的 G1 遗留不再单独推进。
