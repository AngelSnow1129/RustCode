# 多 Agent 协作工作流协议

本规则随会话自动加载。涉及跨角色协作、任务拆分、并行开发或交付推进时，主 Agent 与所有子 Agent 必须遵守。

完整方案见 `docs/multi-agent-collaboration-solution.md`，Agent 定义见 `.codebuddy/agents/`。

## 1. 角色与调用

| Agent | 调用时机 |
|---|---|
| `requirements-analyst` | 需求模糊、缺少验收标准、需要界定范围时 |
| `solution-architect` | 需要方案设计、接口契约、状态所有权或任务拆分时 |
| `code-implementer` | 任务已 `ready` 且契约已冻结，需要编码实现时 |
| `code-reviewer` | 代码改动完成后、提交前，做只读审查 |
| `test-engineer` | 需要补测、执行测试、回归验证时 |
| `doc-writer` | 需要同步文档、CHANGELOG、发布说明时 |
| `project-manager` | 需要端到端推进一个特性（manual 模式，用户手动选中） |

无编排者时，主 Craft Agent 承担编排职责。

## 2. 交接件（唯一权威事实源）

- 目录：`.codebuddy/artifacts/<feature-slug>/`，slug 格式 `YYYY-MM-DD-<短名>`。
- 文件：`STATUS.md`、`00-requirement.md`、`01-design.md`、`02-tasks.md`、`03-impl/<task-id>.md`、`04-review/<task-id>.md`、`05-test-report.md`、`06-release.md`。
- 每个文件头必须带 YAML 信封：`kind / id / from / to / feature / status / decision / requires / files_owned / architecture_constraints / created`。
- 对话中只传递「路径 + 状态 + 决策」；结论一律落盘。
- 交接件只追加不改写，修订写入「修订记录」小节。

## 3. 任务状态机

`pending → ready → in_progress → review → testing → done`，旁路状态 `blocked`、`changes_requested`。

- 只有 `ready` 任务可被领取，一次一个。
- `files_owned` 重叠的任务不得同批次并行；跨 crate 改动默认串行。
- 非 `done` 终态必须写明 `decision` 与下一跳接收方，禁止悬挂任务。
- 单任务返工至多 2 轮，第 3 轮必须升级给用户。

## 4. 门禁

| 门禁 | 阶段 | 关键条件 |
|---|---|---|
| G1 | 需求 | AC 可测、非目标明确、无未决歧义、已标注是否触碰持久化/协议/安全/生命周期 |
| G2 | 设计 | 契约已冻结、状态所有权唯一、依赖方向合规、失败与取消语义已定义、并行批次无文件冲突 |
| G3 | 实现 | `cargo check -p <crate> --all-targets` 与 `cargo test -p <crate>` 通过、改动不超 `files_owned`、无未批契约变更 |
| G4 | 审查 | 无 blocker/critical，major 已修复或获 PM 接受，返工 ≤ 2 轮 |
| G5 | 测试 | 新增/修改路径有覆盖，受影响 crate 全绿，跨 crate 变更补 workspace 检查，无静默跳过 |
| G6 | 交付 | 行为变化/风险/验证/未验证范围齐全，文档同步，回滚方案可执行，`STATUS.md` 无悬挂任务 |

未通过门禁不得推进到下一阶段。

## 5. 硬约束

1. 子 Agent 之间不直接互调，全部经编排者路由。
2. 最小权限：审查/分析类不授予写工具，文档类不授予 `Bash`。
3. 契约先行：契约未冻结不得并行；实现者不得擅自改契约，必须回退 `solution-architect`。
4. 显式失败：禁止 noop、静默降级、假成功；失败必须记录状态 + 决策 + 命令输出证据。
5. 遵守 `AGENTS.md`：`CodingRuntime` 为唯一运行时所有者；kernel/capabilities/coding 生产依赖保持 core-free，依赖方向不得反转；不得恢复 bridge、v1/v2 开关、core session 磁盘模型或任何 fallback。
6. 修改前先记录 branch 与 commit SHA、搜索目标符号的生产方与消费者、查看相关文件近期 Git 历史；dirty worktree 中的用户改动不得重置或覆盖。

## 6. 验证命令基线

```bash
cargo check -p <crate> --all-targets
cargo test  -p <crate>
cargo check --workspace --all-targets   # 跨 crate / 公共协议 / 持久化变更时
cargo clippy -p <crate> --all-targets
cargo fmt --check
```

`cargo test` 已完成同等编译验证时，不重复运行 `cargo check`。
