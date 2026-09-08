# 内置 code-review 深度模式（维度扇出）—— 设计

日期：2026-08-20
状态：设计已批准，待产出实施计划
涉及 crate：`rustcode-review`（L2），以及 `rustcode-tuix` 中少量命令接线改动。

## 问题

内置的 `code_review` 工具（`rustcode-review/src/review_tool.rs`）只运行**单个**只读的 reviewer 子 agent：它计算限定范围的 diff，构造一个任务（带注解的 diff + 按语言的规则 + 确定性影响计划），通过 `build_review_agent_with` 启动唯一一个 agent 并跑到结束，从单个 `report_finding` sink 收集发现，然后过滤/排序/渲染。

它已经相当精巧（确定性影响计划、轮次预算压力 hook、范围预检/确认、把工具锁定到变更文件、仔细打磨的 persona），但它只有**一个视角、一轮通过**。单一 reviewer 是以成本换召回：不同的关注点（正确性 vs 安全性 vs 性能 vs 测试/契约）争夺同一份轮次预算，而且没有对抗性的二次审视。

rustcode 已经具备扇出所需的编排原语（`rustcode-capabilities` 的 task/team：`JoinSet` + 信号量、`reviewer`/`security`/`performance` 角色、explore/worker 权限），而 `rustcode-review` 自身也已经暴露了扇出所需的一切：`build_review_agent_with` 返回 `(agent, report_sink)`，`ReviewAgentConfig::with_persona_append` 让每个 agent 都能带上专门化的视角，而无需触碰基础 persona。

## 目标

- 新增一个**可选加入**的深度评审模式：按关注点维度扇出多个 reviewer，并发运行，并合并/去重它们的发现 —— 相比单 agent 默认路径提升召回。
- 把当前的单 agent 路径保持为**默认**（成本敏感：无头工程服务会在廉价模型上大规模驱动评审）。
- 每个维度复用既有的评审机制（影响计划、轮次预算、范围预检、规则、渲染）—— 不做平行的重复实现。

## 非目标（v1）

- **v1 不做对抗式复核轮次。** 它留给第 2 阶段（见下文）；编排层为它预留好了挂钩。
- 不改变单 agent 的默认行为及其输出。
- 不新增对 `task`/team 工具的依赖：`rustcode-review` 通过 `build_review_agent_with` 自行扇出，这更简单，也让评审 crate 保持自包含。
- v1 不提供配置文件级别的默认开关（仅限调用作用域）。以后可以再加，无需重新设计。

## 用户可见接口

- 斜杠命令：`/review deep [scope]`。前导 `deep` 关键字在 `/review` 的参数映射（`rustcode-tuix/src/event_loop/commands.rs`）中解析，并成为工具的 `depth` 参数。不带 `deep` 的 `/review [scope]` 行为不变。
- 工具参数：`code_review` 的 `Args` 新增 `depth: Option<String>`，取值为 `"single"`（缺省时的默认值）与 `"deep"`。未知值或空值回退为 `"single"`。
- `code_review` 的工具描述与 reviewer persona 会说明 `depth` 选项，让模型（以及可以直接传该参数的无头 `gitcode-assist-service`）能够请求它。

## 架构

`ReviewTool::execute()` 的前半部分保持不变：

1. 解析参数（现在包含 `depth`）。
2. 在实时工作目录中计算限定范围的 diff。
3. diff 为空 → 提前返回「无可评审内容」。
4. **范围预检/确认在任何扇出之前执行。** 大 diff 仍然需要确认令牌；扇出绝不会放大一个未确认的大范围。
5. 构建共享的任务输入：带注解的 diff、变更文件列表、按语言的规则、确定性影响计划。

然后按 `depth` 分派：

- `single`（默认）：与今天完全一致的路径 —— 一次 `build_review_agent_with`、一次 `run_to_completion`、一个 sink。**行为零变化。**
- `deep`：把共享的任务输入交给新的 `fanout` 编排器。

### 扇出编排器（`rustcode-review/src/fanout.rs`）

新增模块，承担三项职责，各自可独立测试：

1. **维度表** —— 一份固定的视角清单，每项为 `{ id, display, lens }`，其中 `lens` 是 `persona_append` 字符串，用于让一个原本标准的 reviewer 带上偏向：
   - `correctness` —— 逻辑、边界情况、并发、错误处理、回归
   - `security` —— 注入、授权、密钥、供应链（依赖、CI、配置）
   - `performance` —— 热路径、内存分配、阻塞调用、N+1
   - `tests_contracts` —— 变更的测试覆盖、API/契约一致性、跨文件的单边分歧
   每个 reviewer 都通过自己的视角审视**完整** diff；维度之间的重叠是预期内的，由去重环节消解（偏向召回）。

2. **编排** —— 为每个维度构建一个评审 agent（以基础配置加上该维度的 `with_persona_append` 调用 `build_review_agent_with`，并配一个独立的 report sink），在并发上限（= 维度数量，即很小）之下并发运行，并遵循宿主轮次的取消信号。「把单个维度跑到结束」这一步被抽到一个可注入的函数/trait 背后，因此测试无需真实模型即可喂入预置的发现。

3. **合并/去重** —— 一个纯函数 `merge_findings(Vec<Vec<Finding>>) -> Vec<Finding>`，按键 **（规范化后的文件路径、重叠的行区间、相似的标题）** 折叠近似重复项。发生碰撞时保留优先级/置信度更高的发现，并累积贡献该发现的维度 ID 集合作为标签。

### 数据流

```
args(depth=deep)
  → git_diff (scoped)                     [unchanged]
  → scope preflight / confirmation        [unchanged, BEFORE fan-out]
  → shared task: annotated diff + rules + impact plan   [unchanged builders]
  → fanout:
       for d in DIMENSIONS (concurrent, capped, cancellable):
         agent_d = build_review_agent_with(cfg.with_persona_append(d.lens), provider)
         run_to_completion(agent_d, task)  → sink_d.findings()
       merge_findings([sink_d.findings() ...])  (dedup + dimension tags)
  → retain changed-file paths              [unchanged sort/filter]
  → sort_findings                          [unchanged]
  → render (findings + per-dimension summary line)
```

## 错误处理

- 某个维度 agent 出错或被取消**不会让整个评审失败**：它已上报的内容仍会参与合并。渲染会说明 N 个维度中完成了多少个（沿用现有 `render_incomplete_review` 的风格），以及哪些失败了。
- 只有当**所有**维度都失败、且合并结果为空时，评审才算硬失败。
- 用户取消（`ctx.cancel`）通过丢弃在途 future 来传播，从而取消子 agent（与单 agent 路径所依赖的机制相同）。

## 结果呈现

复用 `sort_findings` 与 `render_findings`。新增一行紧凑的按维度汇总（哪些维度已完成、每个维度贡献了多少发现、去重数量），让用户看到覆盖情况。合并后的发现会带上贡献它的维度标签。

## 成本控制

- 深度模式严格可选（`/review deep` / `depth:"deep"`）。
- 并发上限为（很小的）维度数量。
- 每个维度复用同一个评审模型，以及既有的轮次预算与影响计划机制。
- 范围预检在扇出之前拦截大 diff，因此深度模式不会静默放大一个未确认的巨大范围。

## 代码组织 / 单元

- `rustcode-review/src/fanout.rs`（新增）：维度表、`merge_findings`（纯函数）、带可注入单维度执行步骤的编排。自包含且可单元测试。
- `rustcode-review/src/review_tool.rs`：为 `Args` 增加 `depth`，在 `deep` 时分派给 `fanout`；前半部分的 diff/预检/任务构建保持不变，并被两条路径共用。
- `rustcode-tuix/src/event_loop/commands.rs`：在 `/review` 的参数映射中解析前导 `deep` 关键字 → 在合成出来的工具请求/提示上设置 `depth`。
- `rustcode-review/src/lib.rs`：按需导出新的扇出入口。

## 测试（TDD）

纯逻辑 / 确定性优先：

- `merge_findings`：按文件 + 重叠行区间 + 相似标题去重；碰撞时保留更高的优先级/置信度；维度标签累积；互不相同的发现被保留。
- 维度表：id/lens 存在且稳定；每个 lens 都是非空追加，且不会替换基础 persona。
- `/review` 参数解析：`deep [scope]` → `depth=deep` + 正确的 scope；裸 scope → `depth=single`；未知 depth 值 → single。
- 深度渲染：按维度的汇总行；部分失败的渲染（x/N 个维度已完成）。

编排接缝：

- 注入预置的按维度发现（无需真实模型），断言完整的「合并 → 过滤 → 排序 → 渲染」流水线，包含部分失败与全部失败路径。

既有的单 agent 测试必须原样保持通过（默认路径未被触碰）。

## 第 2 阶段 —— 对抗式复核轮次（2026-08-20 批准；单独实现）

第 1 阶段（上文）通过维度扇出提升召回。第 2 阶段增加一个精确率层：一个可选加入的复核轮次，用于从合并后的发现中剔除误报。

### 触发方式

`depth` 增加第三个取值 `"deep+verify"`。`is_deep()` 对 `deep`
与 `deep+verify` 均为真；新增的 `wants_verify()` 仅对 `deep+verify` 为真。
schema 枚举扩展为 `["single", "deep", "deep+verify"]`。`/review deep+verify
[scope]` maps in `commands.rs` (the `deep+verify` 关键字先于
`deep` 匹配）。`single` 与 `deep` 保持不变。

### 执行位置

位于深度合并（合并 → 范围过滤 → 排序）与渲染之间。每条幸存的 `MergedFinding` 分配**一个**复核 agent；判定保留/丢弃之后，再渲染幸存者。

### 复核 agent —— 复用，零新工具

复核 agent 是另一个 `build_review_agent_with` reviewer，带有 verifier 的 `persona_append` 视角和单条发现的任务：“以下是一条来自先前评审的候选发现：<finding>。请基于 DIFF（权威来源）与只读工具，判断它是否是本次改动引入的真实缺陷。如果确实是 —— 或者你不确定 —— 调用 `report_finding` 重新上报（可以对其做 refinement）。只有在你确信它是误报 / 并非本 diff 引入 / 已被处理时，才什么都不上报，并说明原因。”

- **保留信号 = 该复核 agent 的 `report_finding` sink 非空。** 这复用了既有的 sink 与装配逻辑；不需要新工具，也不需要新的判定类型。
- **单票且偏向保留：** 不确定 ⇒ 重新上报 ⇒ 保留。只有确信的反驳才会丢弃发现，因此一个迟疑的怀疑者永远不会抹掉真实发现。
- 被保留的条目是**原始** `MergedFinding`（保留累积的维度标签）；重新上报只被用作是/否信号。
- **失败开放（fail-open）：** 出错或被取消的复核 agent 保留其发现 —— 复核失败绝不能丢弃一条可能为真的发现。

### 并发 / 取消

复核 agent（每条幸存的发现一个，通常是个位数）在带上限的 `tokio::task::JoinSet` 中运行（并发上限例如 6）。每个都在自己的 `tokio::select!` 中携带 `ctx.cancel`，与维度 agent 相同。

### 结果呈现

幸存者按今天的方式渲染；汇总新增一行内容：`verify: dropped K of M
candidate finding(s)`，用于说明剔除了多少候选发现。

### 成本

`deep+verify` 的 agent 数量 = 4 个维度 + K 条幸存的发现。可选加入且并发受限。`deep`（不做复核）不受影响。

### 代码组织（第 2 阶段）

- `fanout.rs`：把 `finalize_deep_review` 拆为可复用的几块 —— `merge_deep_findings(outcomes, changed_paths) -> (Vec<MergedFinding>, deduped)`、`dimension_coverage(outcomes) -> (completed, failed)`，以及 `render_deep_result(..., verify_dropped: Option<usize>)`；`finalize_deep_review` 委托给它们（当 `verify_dropped = None` 时输出逐字节一致，因此第 1 阶段的测试保持通过）。新增一个 verifier lens 常量，以及一个带上限的 `run_verify(n, cap, verify_one) -> Vec<bool>` 保留掩码运行器。
- `review_tool.rs`：`Args::wants_verify()`、schema 枚举取值，以及 `deep+verify` 分支 —— 执行 `merge_deep_findings` → `run_verify`（每条发现构建一个复核 agent）→ `render_deep_result(..., Some(dropped))`。
- `commands.rs`：映射 `deep+verify` 关键字。

第 2 阶段不做：多票评审团、按簇复核、只复核高优先级 —— 全部推迟；单票设计是已达成共识的起点。
