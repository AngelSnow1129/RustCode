# DeepSeek V4 Flash 成对评测设计

## 目标

在裸模型层与 RustCode 编码 agent 层同时对比 `RustCode-deepseek-v4-flash`
（线上模型 `deepseek-v4-flash`）与 `volcengine/deepseek-v4-flash`
（线上模型 `ep-20260822184526-dbhzk`）。运行成对且并发；由 Codex 对定性结果做盲评并撰写最终报告。

## 架构边界

评测器是架在 RustCode 既有无头 CLI 之上的外部评测台。每个候选方运行都独占
独立的进程、`RUSTCODE_HOME`、会话与可写 fixture。它通过 `--provider` 选择已配置的
模型；不新增第二个在线 agent 所有者，不在线上运行时内重载 provider，也不改动
kernel、编码运行时、provider、会话或持久化契约。

```text
case + immutable fixture
    |-- paired launch --> official DS --> isolated RustCode runtime --> artifacts
    `-- paired launch --> Volcano DS  --> isolated RustCode runtime --> artifacts
                                                              |
                                              verify + anonymize
                                                              |
                                              Codex blind judge
                                                              |
                                              statistics + report
```

## 配对与并发

- 一对始终在几乎相同的时刻把同一个用例提交给两个候选方。
- 快速用例集使用两对并发（最多四个 RustCode 进程）。
- 压力阶段依次使用 1、4、8 对（即 2、8、16 个请求）。
- 每对的候选方启动顺序随机化。评测台记录单调的起止时间戳；
  启动偏移超过 500 ms 则把该对标记为非严格。
- 每次重复都使用全新的会话与可写 fixture。agent 用例使用两份以同一 commit
  为根的独立副本或 worktree。
- 一个候选方失败绝不会取消它的对端。每次被接受的运行都记录一个终态结果：
  success、timeout、cancelled、provider error、rate limited，或
  empty output、launch error、verification failure 等失败类别。

## 快速用例集

裸模型层有 20 个用例：4 个代码理解/调试、4 个逻辑或算法、4 个代码生成、
3 个指令遵循、3 个长上下文，以及 2 个工具 schema 用例。RustCode 层有 8 个
基于仓库的用例：2 个本地缺陷修复、2 个跨文件特性、1 个仅诊断任务、1 个
保行为重构、1 个长上下文任务，以及 1 个误导性遗留路径任务。每个用例
在每个候选方上运行三次。

每个用例都包含不可变的指令与显式校验器。在编译、测试、必需/禁止文件、
输出 schema 与精确答案上，机器检查具有权威性。类人的质量维度只在候选方
身份被替换为随机 A/B 标签之后才评判。

## 度量项

能力按正确性（45%）、代码质量（20%）、指令遵循（15%）、agent 执行质量（10%）
与 Codex 盲评（10%）打分。稳定性作为独立结果保留：成功率与首次成功率、
P50/P90/P95 延迟、重试、429/5xx/传输/流式/空响应失败、截断、非法或重复的
工具调用、token 用量，以及分数方差。缓存效率依据 RustCode 的 provider 用量上报，
即缓存 prompt token 除以 prompt token，含均值/P50/P95 与冷启动对重复执行两个队列。

RustCode 既有的重试属于端到端结果的一部分，但评测台会区分首次请求成功与
最终成功，并记录重试代价。能力领先需至少 5 分，且成对 bootstrap 的 95%
置信区间不含零。稳定性差异在成功率相差 3 个百分点或 P95 延迟相差 20% 时
视为实质性差异。否则报告判定两个候选方在被测负载下相当。

## Codex 评判

Codex 收到用例、评分细则、校验器输出与匿名化后的 A/B 产物。它不会收到
端点或 provider 身份。A/B 顺序随机化。每条评判以校验通过的 JSON 输出，
包含胜者、各维度评分、证据、严重失败项与置信度。重要或存在冲突的用例
在全新上下文中重评；出现分歧时记为 `needs_review`，而不是强行判出胜者。

另一次全新的 Codex 调用读取聚合统计与可追溯的样本证据，随后写出 Markdown
报告。报告包含能力、延迟、稳定性、各任务类型的优势、局限与部署建议。
机器结果不得被定性判断覆盖。任何对网关行为的解释都标注为推断。

## 可复现性与安全

运行清单记录 RustCode commit、二进制版本、配置指纹、选型 ID、期望的
线上模型标识、Codex 版本/模型、用例哈希、并发度、超时、重试策略、
时间戳与随机种子。启动阶段必须确认每个选型都解析到期望的账号/模型；
回退到默认模型即视为基础设施故障。

API key、OAuth token、授权头与完整的私有配置绝不复制进结果产物。原始输出
在聚合前先脱敏。报告保留指向本地证据的链接，但不内嵌密钥。
