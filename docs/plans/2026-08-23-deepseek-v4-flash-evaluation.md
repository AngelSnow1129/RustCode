# DeepSeek V4 Flash 评测台实施计划

> **给 Claude：** 必需子技能：使用 superpowers:executing-plans 逐任务实施本计划。

**目标：** 构建可复现的成对并发评测台，通过 RustCode 对比两个已配置的 DeepSeek V4 Flash 选型，并用 Codex 做盲评与最终报告。

**架构：** 在 `evals/deepseek-v4-flash/` 下新增一个仓库内、仅用标准库的 Python 评测台。它在隔离的 home 与 fixture 中启动既有的 RustCode 无头进程，持久化可审计的 JSON 产物，运行用例校验器，对候选方做匿名化，调用 `codex exec` 得到结构化评判，并在不改动运行时所有权的前提下汇总结果。

**技术栈：** Python 3.7 标准库、JSON、RustCode 无头 CLI、Codex CLI、JSONL、Markdown。

---

### Task 1：定义用例集与产物契约

**文件：**
- 新建：`evals/deepseek-v4-flash/benchmark.json`
- 新建：`evals/deepseek-v4-flash/cases/example-model/case.json`
- 新建：`evals/deepseek-v4-flash/cases/example-model/prompt.md`
- 新建：`evals/deepseek-v4-flash/README.md`

1. 定义两个不可变的选型 ID 与期望的线上模型名。
2. 定义超时、成对并发度、重复次数、随机种子、RustCode/Codex 路径，
   以及产物保留策略。
3. 定义用例 schema，包含 tier、prompt、fixture、校验器、超时与评分细则。
4. 记录凭据处理方式、隔离要求，以及四条命令。
5. 用 `python3 -m unittest discover -s evals/deepseek-v4-flash/tests -v` 验证 JSON 可加载。

### Task 2：实现配置校验与准备

**文件：**
- 新建：`evals/deepseek-v4-flash/eval.py`
- 新建：`evals/deepseek-v4-flash/tests/test_eval.py`

1. 先写失败测试，覆盖用例集解析、用例发现、不安全路径，以及稳定的
   用例/配置指纹。
2. 实现带类型的数据类与严格校验，仅使用 Python 标准库。
3. 实现 `prepare`，生成运行清单、随机化成对调度表、
   候选方别名、隔离目录与不可变的用例副本。
4. 确保清单数据中不含任何凭据值。
5. 运行单元测试，确认所有准备类测试通过。

### Task 3：实现成对并发执行

**文件：**
- 修改：`evals/deepseek-v4-flash/eval.py`
- 修改：`evals/deepseek-v4-flash/tests/test_eval.py`

1. 编写一个假的 RustCode 可执行文件，记录参数/环境变量，并返回
   可配置的成功、延迟、stderr 与失败结果。
2. 先写失败测试，证明两个候选方并发启动、拿到各自独立的
   `RUSTCODE_HOME`/工作目录、能在对端失败时存活，并能干净地超时。
3. 用 asyncio 成对屏障与成对信号量实现 `run`。
4. 以 `--provider`、`--config`、`--prompt-file`、`-C`、`--verbose`、
   `--dev`、`--no-telemetry` 调用 RustCode；仅对显式可信的 agent fixture 追加 `-y`。
5. 按运行原子化地持久化 stdout、脱敏后的 stderr、退出状态、单调耗时、
   启动偏移、哈希与归类后的终态结果。
6. 用假可执行文件运行单元测试。

### Task 4：校验并匿名化产物

**文件：**
- 修改：`evals/deepseek-v4-flash/eval.py`
- 修改：`evals/deepseek-v4-flash/tests/test_eval.py`
- 新建：`evals/deepseek-v4-flash/prompts/codex-judge.md`

1. 先写失败测试，覆盖校验器的通过/失败/超时、密钥脱敏、随机
   A/B 映射，以及评判包中不出现 provider 标识。
2. 在各自 fixture 中以有界超时运行每个校验器，并保存 stdout/stderr。
3. 生成评判包，内含用例、评分细则、候选方输出/diff，以及随机 A/B 名称下的
   机器校验证据。
4. 拒绝含有已配置选型、账号、端点或线上模型标识的评判包。
5. 运行全部单元测试。

### Task 5：调用 Codex 并校验评判结果

**文件：**
- 修改：`evals/deepseek-v4-flash/eval.py`
- 修改：`evals/deepseek-v4-flash/tests/test_eval.py`
- 修改：`evals/deepseek-v4-flash/prompts/codex-judge.md`

1. 补充假 Codex 测试：合法 JSON、带围栏的 JSON、非法输出、非零退出、
   超时，以及重试/复核归类。
2. 用 stdin 传 prompt 的方式实现 `judge`，调用 `codex exec --json --cd <run> -o
   <file>`，Codex 模型固定且可配置。
3. 校验胜者、各维度取值区间、证据、严重失败项与置信度；
   绝不静默地强行修正畸形评分。
4. 原始 Codex 事件与校验后的评判 JSON 分开持久化。
5. 运行单元测试。

### Task 6：汇总并生成最终报告

**文件：**
- 修改：`evals/deepseek-v4-flash/eval.py`
- 新建：`evals/deepseek-v4-flash/prompts/codex-report.md`
- 修改：`evals/deepseek-v4-flash/tests/test_eval.py`

1. 编写测试，覆盖成对差异、百分位计算、成功/错误
   分布、分数方差，以及确定性的 bootstrap 置信区间。
2. 实现 `summarize`，仅在盲评定稿之后生成带 provider 视角的 `summary.json`。
3. 实现 `report`，把汇总数据与有界证据交给 Codex，校验必需章节存在，
   并写出 `report.md`。
4. 明确区分观测到的事实与推断出的网关解释。
5. 运行单元测试与一次完整的假二进制冒烟测试。

### Task 7：铺底快速用例集并记录真实执行

**文件：**
- 新建/修改：`evals/deepseek-v4-flash/cases/**`
- 修改：`evals/deepseek-v4-flash/README.md`

1. 增加 20 个模型用例与 8 个 agent 用例，均带确定性校验器。
2. 把每个仓库 fixture 固定到显式 commit 或内容哈希。
3. 记录 provider 解析的预检步骤，以及如何核查真实的模型身份。
4. 先运行 `prepare`，再做一次不含网络凭据的单用例空跑。
5. 检查 `git diff --check` 与完整 diff；不要把结果产物包含进来。
