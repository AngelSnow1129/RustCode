# zh-check 报告

- 基线 (--base): `3ee655e3`
- 阈值 (--en-ratio): `0.05`
- 判定: AC-2 / AC-4 / AC-6 / AC-7b / AC-32，任一失败即 FAIL（fail-closed）

## 汇总

- 受检文件: 9
- PASS: 7
- FAIL: 2

## PASS `evals/deepseek-v4-flash/README.md`

- en=0/17 ratio=0.0000

## FAIL `evals/deepseek-v4-flash/prompts/codex-judge.md`

- en=6/6 ratio=1.0000
- **AC-2 FAIL**
  - `AC-2 en_ratio 1.0000 > 0.0500 (en=6/total=6)`
  - `  evals/deepseek-v4-flash/prompts/codex-judge.md:1: You are a strict blind evaluator. Candidate identities are intentionally hidden.`
  - `  evals/deepseek-v4-flash/prompts/codex-judge.md:2: Use machine verification as authoritative. Assess only the supplied evidence.`
  - `  evals/deepseek-v4-flash/prompts/codex-judge.md:3: Return one JSON object, without a Markdown fence, with this schema:`
  - `  evals/deepseek-v4-flash/prompts/codex-judge.md:5: {"winner":"A|B|tie","scores":{"A":{"correctness":0,"quality":0,"instruction_following":0,"agent_execution":0},"B":{"correctness":0,"quality":0,"instruction_following":0,"agent_execution":0}},"evidence":["specific evidence"],"critical_failures":[],"confidence":0.0}`
  - `  evals/deepseek-v4-flash/prompts/codex-judge.md:7: Every score is an integer from 0 through 100. Confidence is from 0 through 1.`
  - `  evals/deepseek-v4-flash/prompts/codex-judge.md:8: Do not guess missing facts and do not attempt to identify the providers.`
- AC-3 残留行清单 (en 行，共 6 行):
  - `evals/deepseek-v4-flash/prompts/codex-judge.md:1`: `You are a strict blind evaluator. Candidate identities are intentionally hidden.`
  - `evals/deepseek-v4-flash/prompts/codex-judge.md:2`: `Use machine verification as authoritative. Assess only the supplied evidence.`
  - `evals/deepseek-v4-flash/prompts/codex-judge.md:3`: `Return one JSON object, without a Markdown fence, with this schema:`
  - `evals/deepseek-v4-flash/prompts/codex-judge.md:5`: `{"winner":"A|B|tie","scores":{"A":{"correctness":0,"quality":0,"instruction_following":0,"agent_execution":0},"B":{"correctness":0,"quality":0,"instruction_following":0,"agent_execution":0}},"evidence":["specific evidence"],"critical_failures":[],"confidence":0.0}`
  - `evals/deepseek-v4-flash/prompts/codex-judge.md:7`: `Every score is an integer from 0 through 100. Confidence is from 0 through 1.`
  - `evals/deepseek-v4-flash/prompts/codex-judge.md:8`: `Do not guess missing facts and do not attempt to identify the providers.`

## FAIL `evals/deepseek-v4-flash/prompts/codex-report.md`

- en=12/12 ratio=1.0000
- **AC-2 FAIL**
  - `AC-2 en_ratio 1.0000 > 0.0500 (en=12/total=12)`
  - `  evals/deepseek-v4-flash/prompts/codex-report.md:1: You are writing the final engineering evaluation report for two DeepSeek V4 Flash`
  - `  evals/deepseek-v4-flash/prompts/codex-report.md:2: service candidates. Use only the supplied aggregate data and evidence. Machine`
  - `  evals/deepseek-v4-flash/prompts/codex-report.md:3: measurements are authoritative. Label explanations about gateway behavior as`
  - `  evals/deepseek-v4-flash/prompts/codex-report.md:4: inferences. Include: Executive conclusion, Capability, Stability, latency and cache hit rate,`
  - `  evals/deepseek-v4-flash/prompts/codex-report.md:5: Task-level strengths, Anomalies and limitations, and Deployment recommendation.`
  - `  evals/deepseek-v4-flash/prompts/codex-report.md:6: Apply the supplied decision rules. If the formal suite is incomplete, explicitly`
  - `  evals/deepseek-v4-flash/prompts/codex-report.md:7: call the report a harness smoke result and do not recommend a production default,`
  - `  evals/deepseek-v4-flash/prompts/codex-report.md:8: even when a single latency observation differs substantially.`
  - `  evals/deepseek-v4-flash/prompts/codex-report.md:9: Treat explicit 'suite_completion' metadata as authoritative when deciding whether`
  - `  evals/deepseek-v4-flash/prompts/codex-report.md:10: the formal quick suite is complete. A model case need not have an executable`
  - `  evals/deepseek-v4-flash/prompts/codex-report.md:11: verifier when it has a rubric-based blind judgment. Diagnosis-only agent cases may`
  - `  evals/deepseek-v4-flash/prompts/codex-report.md:12: intentionally have no executable verifier.`
- AC-3 残留行清单 (en 行，共 12 行):
  - `evals/deepseek-v4-flash/prompts/codex-report.md:1`: `You are writing the final engineering evaluation report for two DeepSeek V4 Flash`
  - `evals/deepseek-v4-flash/prompts/codex-report.md:2`: `service candidates. Use only the supplied aggregate data and evidence. Machine`
  - `evals/deepseek-v4-flash/prompts/codex-report.md:3`: `measurements are authoritative. Label explanations about gateway behavior as`
  - `evals/deepseek-v4-flash/prompts/codex-report.md:4`: `inferences. Include: Executive conclusion, Capability, Stability, latency and cache hit rate,`
  - `evals/deepseek-v4-flash/prompts/codex-report.md:5`: `Task-level strengths, Anomalies and limitations, and Deployment recommendation.`
  - `evals/deepseek-v4-flash/prompts/codex-report.md:6`: `Apply the supplied decision rules. If the formal suite is incomplete, explicitly`
  - `evals/deepseek-v4-flash/prompts/codex-report.md:7`: `call the report a harness smoke result and do not recommend a production default,`
  - `evals/deepseek-v4-flash/prompts/codex-report.md:8`: `even when a single latency observation differs substantially.`
  - `evals/deepseek-v4-flash/prompts/codex-report.md:9`: `Treat explicit 'suite_completion' metadata as authoritative when deciding whether`
  - `evals/deepseek-v4-flash/prompts/codex-report.md:10`: `the formal quick suite is complete. A model case need not have an executable`
  - `evals/deepseek-v4-flash/prompts/codex-report.md:11`: `verifier when it has a rubric-based blind judgment. Diagnosis-only agent cases may`
  - `evals/deepseek-v4-flash/prompts/codex-report.md:12`: `intentionally have no executable verifier.`

## PASS `evals/deepseek-v4-flash/cases/agent-fixture/ARCHITECTURE.md`

- en=0/3 ratio=0.0000

## PASS `.goals/rustcode-migration-finalize/goal.md`

- en=0/59 ratio=0.0000
- R5 跳过的行 (共 1 行):
  - `.goals/rustcode-migration-finalize/goal.md:32`: `    'mcp::registry::tests::trust_key_golden_matches_core_algorithm' 除外)`

---

# D-36 手写小结（在最后一次自检之后追加）

## 1. 本批范围、前提与自检命令

- 任务 ID：**D-36**（批次 13）。基线 `ZH_BASE=3ee655e3`，自检工具 `scripts/check-zh-docs.py`。
- files_owned 共 9 个文件，**全部无 YAML frontmatter**（首行均不是 `---`）：
  铁律 §2.7 的「YAML 键名一律保留英文」本批无适用对象，AC-6 键名集合在基线与工作区
  **皆为空集、相等**，无 `description` 需按 AC-6 逐条列示。
- 最后一次自检命令（本报告上面的自动生成段即由该命令产出）：

```bash
python3 scripts/check-zh-docs.py check --base 3ee655e3 \
  --report .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-36.md \
  --files evals/deepseek-v4-flash/README.md \
  evals/deepseek-v4-flash/prompts/codex-judge.md \
  evals/deepseek-v4-flash/prompts/codex-report.md \
  evals/deepseek-v4-flash/cases/agent-fixture/ARCHITECTURE.md \
  .goals/rustcode-migration-finalize/goal.md \
  .goals/rustcode-migration-finalize/inspector-feedback-1.md \
  .goals/rustcode-migration-finalize/summary.md CONTEXT.md DEVENV.md
```

- 实测输出：`check: 受检 9，PASS 7，FAIL 2`（2 个 FAIL 为本批**判定为 no-op 的评测 prompt 模板**，见 §3）。
- 开工前已先跑一次自检拿 offender 清单（本批特殊说明第 5 条），据此区分了
  「纯英文需全文译」（4 个）与「已中文只需清 offender」（3 个）+「运行时载荷 no-op」（2 个）。

## 2. 每个文件的改动量（含 no-op 判定）

| 文件 | 基线 en/total | 现 en/total | 改动性质 | 说明 |
|---|---|---|---|---|
| `evals/deepseek-v4-flash/README.md` | 19/19 | **0/17** | **纯英文 → 全文译** | 命令/flag/路径/产物名全保留；段落内重排使物理行数 19→17（§5.7 允许） |
| `evals/deepseek-v4-flash/prompts/codex-judge.md` | 6/6 | 6/6 | **no-op** | 评测运行时载荷，见 §3 |
| `evals/deepseek-v4-flash/prompts/codex-report.md` | 12/12 | 12/12 | **no-op** | 评测运行时载荷 + 硬断言，见 §3 |
| `evals/deepseek-v4-flash/cases/agent-fixture/ARCHITECTURE.md` | 4/4 | **0/3** | **纯英文 → 全文译** | `Runtime`/`Driver`/`legacy.py` 原样保留；属 fixture 数据，风险见 §10 |
| `.goals/rustcode-migration-finalize/goal.md` | 17/59 | **0/59** | 已中文 → 只清 offender（17 行） | 6 个英文标题 + 11 行约定条目；commit hash 与全部 inline code 未动 |
| `.goals/rustcode-migration-finalize/inspector-feedback-1.md` | 31/31 | **0/31** | **纯英文 → 全文译** | 表格结构、`PASS`、全部 `grep`/`cargo` 命令 inline code 原样保留 |
| `.goals/rustcode-migration-finalize/summary.md` | 37/50 | **0/50** | 混合 → 只清 offender（37 行） | 已中文的 6 条设计原则条目未改写（铁律 §2.8） |
| `CONTEXT.md` | 30/45 | **0/45** | 已中文 → 术语行补中文释义（30 行） | **英文术语一字未改**，见 §4 |
| `DEVENV.md` | 3/23 | **0/23** | 已中文 → 只清 offender（3 行） | 表格两行工具名 + 一行镜像说明补中文 |

汇总：`git diff --stat` = **7 files changed, 142 insertions(+), 145 deletions(-)**（2 个 prompt 文件 no-op，零 diff）。
净行数 -3 全部来自段内重排（`README.md` 19→17、`ARCHITECTURE.md` 4→3 非空白行）；
已用逐行「空/非空」序列对拍确认：**空行位置一字未变**（仅 `README.md` 两段各少 1 个物理行），
符合铁律 §5.4 与 §5.7。

## 3. `prompts/*.md` 的处理决定：判定为运行时载荷，记 no-op（单独说明理由）

**本批对 `evals/deepseek-v4-flash/prompts/codex-judge.md` 与 `codex-report.md` 未做任何改动。**
判定依据（逐条实测，非臆测）：

1. **两者都不是给人看的说明，而是被 `eval.py` 在运行时读取并拼进模型请求的载荷**：
   - `eval.py:547`：`template = (ROOT/"prompts/codex-judge.md").read_text()`，随后
     `prompt = template + "\n\nEvaluation packet:\n" + packet_path.read_text()` 送 `codex_exec`。
   - `eval.py:567` 与 `eval.py:587`：`(ROOT/"prompts/codex-report.md").read_text()+"\n\nData:\n"+json.dumps(...)`。
   即：**改动词面 = 改变评测行为**，正属本批特殊说明第 3 条所说的「纯 prompt 指令」。
2. **`codex-report.md` 存在硬断言，译文会直接破坏门禁（决定性证据）**：
   `eval.py:569` 与 `eval.py:589` 用
   `required = ("Executive conclusion", "Capability", "Stability", "Deployment recommendation")` /
   `("Executive conclusion", "Capability", "Stability", "cache", "Deployment recommendation")`
   对生成报告做 `x.lower() in answer.lower()` 断言，缺失即
   `raise ValueError("Codex report is missing required sections")`。
   而 `codex-report.md:4-5` 的 `Executive conclusion`、`Capability`、`Stability`、
   `latency and cache hit rate`、`Deployment recommendation` 正是这些被代码消费的**章节标识符**。
   翻译它们会让 `eval.py report` / `combined-report` 直接抛错。
3. **`codex-judge.md` 含机器解析的 JSON 契约字面量**：第 5 行整行是
   `{"winner":"A|B|tie","scores":{...},"confidence":0.0}`，由 `eval.py:552` 的
   `extract_json(answer)` + `validate_judgment(value)` 消费。该行按铁律 §2.3 本就不得翻译；
   而它是裁判指令的核心，只译散文、留 JSON 会造成「中英混排的裁判指令」，
   裁判口径一变，成对对比的分值就不再可比。
4. **仓库既有先例支持「运行时载荷不译」**：`scripts/check-zh-docs.py:28` 把
   `crates/rustcode-review/rules/` 列为 `SKIP_B`，注释即「运行时载荷，Q1 不动」。
   评测 prompt 模板属同类资产。

**因此本批按「不要贸然全译」记 no-op，并在此上报编排者（升级项，见 §10）。**
若编排者确认仍需汉化，前置条件（均超出本批「只改 9 个 md」的边界）：
(a) 同步修改 `eval.py:569`/`eval.py:589` 的 `required` 元组（源码改动）；
(b) 保留 `codex-judge.md` 的 JSON 字面量与 `codex-report.md` 的章节标识符原词；
(c) 重新基线化评测结果。请另派任务，本批不擅自处理。

## 4. `CONTEXT.md` 的处理决定：术语零改动，只补中文释义（单独说明理由）

`AGENTS.md:55` 明确 `CONTEXT.md` 是运行时领域术语的**事实源**，且 `_Avoid_` 条目是**硬性命名约束**。
本批对该文件的处理口径如下：

- **14 个领域术语（Coding Runtime、Live View、Live View Hub、Runtime Binding、
  Session Transition、MCP Scope、Tool Catalog、Tool Catalog Revision、
  Turn Tool Snapshot、MCP Readiness、View Projection、Committed Snapshot、
  Replay Window、Pending Interaction）英文原词一字未改**，包括大小写与空格。
- **14 条 `_Avoid_` 的禁用词列表（如 `LiveSession`、`Fire-and-forget session command`、
  `Runtime generation`、`Global approval slot`）英文原词一字未改**；
  `_Avoid_` 标签本身也保留（它是该事实源的结构性键名，便于后续工具识别）。
- 为同时满足 AC-2（`en/total <= 0.05`）与上述命名约束，采用
  「**`**英文原词**（中文释义）**」与「**`_Avoid_（避免使用）：<禁用词原样>`**」两种形式：
  即**只在原英文之后追加中文释义，不替换、不删减、不加反引号**。
- 该文件**没有任何 backtick**（基线 inline code 集合为空集），本批也未新增任何反引号，
  故 AC-4 在该文件上恒等成立；**后续任何人给这些术语加反引号都会立刻 AC-4 FAIL，请勿「顺手整理」。**
- 中文释义与仓库既有译法对齐：`会话切换`（见 `docs/plans/2026-08-17-project-input-history-design.md:53`）、
  `工具目录`（见 `docs/plans/2026-07-26-atomgit-production-tools.md:7`）、
  `已提交快照`/`回放窗口`/`实时视图` 与该术语在 `docs/archive/live-transport-convergence-plan.md:35,72`
  中出现的中文语境一致。

## 5. 残留行清单与白名单判定

| 文件 | 残留 en 行 | 为何属于白名单 |
|---|---|---|
| `evals/deepseek-v4-flash/prompts/codex-judge.md` | 6/6 | 评测运行时载荷 + 机器解析的 JSON 契约，见 §3。**按判定记 no-op，非遗漏** |
| `evals/deepseek-v4-flash/prompts/codex-report.md` | 12/12 | 评测运行时载荷 + `eval.py:569/589` 硬断言的章节标识符，见 §3。**按判定记 no-op，非遗漏** |

其余 7 个文件 **en = 0**，无残留。

另有三类「刻意保留、不算残留」的内容：

- **fenced code block 内一字未动**：`evals/deepseek-v4-flash/README.md` 的 eval.py 命令块、
  `DEVENV.md` 的 quickstart / cargo / 环境变量块。
- **inline code 原样保留**（摘选）：`--ephemeral --output-format jsonl`、`--no-tools`、
  `rustcode`、`codex`、`benchmark.json`、`--case smoke-model --repetitions 1`、
  `--dangerously-skip-permissions`、`allow_edits = true`、`rustcode_bin`、
  `../../target/debug/rustcode`、`case.json`、`prompt.md`、`fixture`、`verify`、
  `summary.json`、`report.md`、`events.jsonl`、`results/`；
  `Runtime`、`Driver`、`legacy.py`；
  `grep -rn "atomcode" crates/ scripts/ .github/`、`grep -rn "atomcode" docs/architecture.md`、
  `grep -ri "sentry\|posthog\|segment\|analytics"`、`install_panic_hook`、
  `impl Default for Locale`、`Locale::ZhCn`、`cargo test -p rustcode-config --lib`、
  `# language = "zh_CN"`、`cli_flag_unparseable_falls_through`、`LOCALE`、
  `resolve_initial_locale_with_env`、`Config.language`、`Option<Locale>`、`#[serde(default)]`、
  `None`、`Locale::default()`、`is_codingplan_llm_gateway`、
  `RUSTCODE_CODINGPLAN_LLM_BASE_URL`、`bearer_auth(api_key)`、`atomgit`、
  `rustcode-telemetry`、`current_locale()`、`LANG=C`/`POSIX`、
  `mcp::registry::tests::trust_key_golden_matches_core_algorithm`、
  `crates/rustcode-config/src/locale.rs`、`crates/rustcode-config/src/i18n/mod.rs`、
  `docs/config.example.toml`、`webui/package-lock.json`、`crates/rustcode-cli/`、
  `type(scope): description`、`RUSTCODE_HOME`、`af7e8715723aa33343f049b15d3032e71200567c`。
- **专有名词/版本号/日期保留**：RustCode、DeepSeek V4 Flash、Volcano Engine、Codex、
  Python 3.7+、Node、Rust、GitHub、npm、cargo、ASCII、Unicode、AtomGit、PASS、SHA、dev branch。

## 6. 四项实测结果

| 项 | 阈值/口径 | 实测 | 结论 |
|---|---|---|---|
| **AC-2** | `en/total <= 0.05` | 7 个文件：`0/17`、`0/3`、`0/59`、`0/31`、`0/50`、`0/45`、`0/23` 全部 0.0000；2 个 prompt 文件 `6/6`、`12/12`（no-op） | **PASS 7/9；FAIL 2/9 为 §3 的判定性 no-op** |
| **AC-4** | inline code + fenced code 多重集与基线完全相等 | 9 个文件**无一报差异**（含 2 个零 diff 的 no-op 文件） | **PASS 9/9** |
| **AC-7b** | `](...)` 链接目标多重集相等 | 9 个文件无一报差异（本批 9 个文件中仅 `DEVENV.md` 含链接，未改动其目标） | **PASS 9/9** |
| **AC-32** | Emoji 数不增加 | 9 个文件无一报 `emoji 命中数增加`；本批未新增、也未清理既有 Emoji | **PASS 9/9** |

附带核对：

- **AC-6**：9 个文件均无 frontmatter，键名集合基线与工作区皆为空集，相等；无 `description` 变更。
- **未踩坑复核**：本批复用了前批教训——未把普通词包成反引号（§5.1）；
  未只加全角标点（`summary.md:36` 首轮写成「（explorer/builder/reviewer，max_concurrent=4）」
  仍被判 en，因 CJK 正则只认汉字不认全角标点，已按 §5.2 补真实汉字「三个角色」后转 PASS，
  见 §7）；未做批量替换（§5.3）；未增删空行（§5.4，已用空/非空序列对拍确认）；
  手写小结在最后一次自检之后追加（§5.5）；本批 9 个文件**无跨行反引号对**，不适用 §5.6。
- **范围**：只改 files_owned 的 9 个文件；未改源码；未执行 `cargo`/`npm` 构建与测试（铁律 §7）。

## 7. 本批唯一踩坑：`summary.md:36` 的全角标点不算 CJK

首轮把该行译为 `  （explorer/builder/reviewer，max_concurrent=4）`，自检仍报 AC-2 残留 1 行。
成因与铁律 §5.2 完全一致：`scripts/check-zh-docs.py:40` 的
`CJK_RE = [\u3400-\u4dbf\u4e00-\u9fff\uf900-\ufaff\u3040-\u30ff]` **只匹配汉字与假名，不含全角标点**，
而 `ASCII4_RE = [A-Za-z]{4,}` 命中 `explorer`，于是该行仍被判英文行。
**处置**：补入真实汉字，改为 `  （explorer/builder/reviewer 三个角色，max_concurrent=4）`，随后转 PASS。
`goal.md:29` 的同类行因首轮即写为「（explorer/builder/reviewer 三个角色）」，未触发该坑。

## 8. 术语与命名一致性检查结论

| 术语 | 本批译法 | 一致性依据 |
|---|---|---|
| paired evaluation / harness / case / candidate / repetition | 配对评测 / 评测框架 / 用例 / 候选 / 重复 | 与 `evals/deepseek-v4-flash/cases/agent-cases.json`、`eval.py` 的字段名语义一致 |
| fixture | fixture（不译） | `eval.py:126-129`、`case.json` 的 `fixture` 键名与目录名，铁律 §2.3 |
| preflight | 预检 | 与 `README.md` 中 `--case smoke-model` 的用途一致 |
| session / working directory | session / working directory（不译） | `CONTEXT.md` 事实源内已有此写法，保持跨文档一致 |
| provider | provider（不译） | 与 `DEVENV.md`、「第三方 OpenAI/Anthropic 兼容端点」表述一致 |
| Acceptance Criteria / Quality Gates | 验收标准 / 质量门禁 | 与 `AGENTS.md` 的 G1/G3/G6/G7/G8 门禁编号口径一致 |
| Initial SHA | 初始 SHA | `SHA` 为标识符，保留 |
| Inspector Feedback | 检查员反馈 | 与 `.goals/` 目录下该文件作为审查记录的角色一致 |
| 运行时领域术语 | **保留英文原词 + 中文释义** | `AGENTS.md:55`「以 `CONTEXT.md` 为准」，见 §4 |

**发现一处源文本自身的术语不一致，本批按原文直译、未擅自统一，在此上报**：
`inspector-feedback-1.md` 的 6 条设计原则英文为
`specialization / clear boundaries / context inheritance / parallel processing / independent verification / cost optimization`，
本批译为「专业化分工、清晰边界、上下文继承、并行处理、独立验证、成本优化」；
而同 feature 的 `summary.md` 已中文段落把同样两条写作
「高效通信 (context inheritance + result passing)」与「质量保证 (reviewer = independent verification)」。
其中 4 条（专业化分工 / 清晰边界 / 并行处理 / 成本优化）两文件一致，**2 条不一致**。
本批遵守铁律 §2.8「已中文的段落不改写」，未改 `summary.md` 的既有中文；是否统一由编排者裁定。

## 9. 文档更新清单

| 文件路径 | 变更类型 | 摘要 |
|---|---|---|
| `evals/deepseek-v4-flash/README.md` | 修订（全文英译中） | 评测框架说明汉化；命令、flag、路径、产物名保留 |
| `evals/deepseek-v4-flash/prompts/codex-judge.md` | **无变更（no-op）** | 评测运行时载荷，见 §3 |
| `evals/deepseek-v4-flash/prompts/codex-report.md` | **无变更（no-op）** | 评测运行时载荷 + `eval.py:569/589` 硬断言，见 §3 |
| `evals/deepseek-v4-flash/cases/agent-fixture/ARCHITECTURE.md` | 修订（全文英译中） | fixture 架构说明汉化；`Runtime`/`Driver`/`legacy.py` 保留 |
| `.goals/rustcode-migration-finalize/goal.md` | 修订 | 6 个英文标题 + 11 行项目约定条目补中文 |
| `.goals/rustcode-migration-finalize/inspector-feedback-1.md` | 修订（全文英译中） | 检查员反馈汉化；表格结构、`PASS`、命令证据保留 |
| `.goals/rustcode-migration-finalize/summary.md` | 修订 | 5 个英文标题 + 32 行条目补中文；已中文段落未改写 |
| `CONTEXT.md` | 修订 | 标题、章节名、14 个术语行、14 条 `_Avoid_` 行补中文释义；英文术语零改动 |
| `DEVENV.md` | 修订 | 工具表 2 行 + cargo 镜像 1 行补中文 |

## 10. 回滚方案

本批为纯文档汉化，改动均未提交（工作区改动）。按优先级：

1. **整体回滚本批（推荐）**：
   `git checkout -- evals/deepseek-v4-flash/README.md evals/deepseek-v4-flash/cases/agent-fixture/ARCHITECTURE.md .goals/rustcode-migration-finalize/goal.md .goals/rustcode-migration-finalize/inspector-feedback-1.md .goals/rustcode-migration-finalize/summary.md CONTEXT.md DEVENV.md`
   判定时机：任一文件出现 AC-4/AC-7b/AC-32 FAIL 且无法就地修复，或编排者否决本批译文。
2. **单文件回滚**：对上表中任一路径单独执行 `git checkout -- <path>`。
   判定时机：仅个别文件译文被否决（例如 `CONTEXT.md` 的术语释义口径需要架构负责人复核）。
3. **回滚后全量校验**：`git stash` 后重跑 §1 的自检命令，用于排除本批之外的干扰。
4. **无需回滚数据/配置/代码**：本批未触碰源码、未改任何配置文件与版本号、无数据迁移，回滚代价为零；
   两个 prompt 文件本就是零 diff，不存在回滚问题。

## 11. 已知未验证范围、风险与升级项

- **未验证**：未执行 `cargo` / `npm` 构建与测试（铁律 §7 明确禁止；本批为纯文档任务）。
  因此 `README.md` 中的 `python3 eval.py ...` 命令序列、`DEVENV.md` 中的 cargo/npm 命令
  **仅按原文保留，未实际执行验证**。
- **未验证**：`inspector-feedback-1.md` 与 `summary.md` 记录的验证结论
  （`cargo test -p rustcode-config --lib` = 324 项通过、`grep` = 0 处命中等）
  是**历史记录，本批未重跑、也未核对当前实现**；本批只做语言转换，未改任何数字与结论。
- **风险（须编排者知悉）**：`cases/agent-fixture/ARCHITECTURE.md` 是被
  `evals/deepseek-v4-flash/cases/agent-cases.json` 中 `agent-context-contract` 用例
  「Read ARCHITECTURE.md ...」显式读取的 **fixture 数据**。
  本批汉化了它，会改变被测模型读到的输入。已确认的缓解因素：
  (a) 语义逐句对等，未改任何架构约束（单一状态所有权、依赖方向、`legacy.py` 单向导入器）；
  (b) 同一配对的**两个候选拿到完全相同的 fixture**，成对对比仍然公平；
  (c) 判定只看行为（`Driver` 是否把变更委托给 `Runtime`），不看文档语言；
  (d) `eval.py:328` 的 `diff -ruN` 是拿 fixture 与工作副本比对，本批改动落在 fixture 基线上，不产生额外 diff。
  若编排者认为 eval fixture 应一律保持英文，请单独回滚该文件（`git checkout -- <path>`），成本为零。
- **遗留后续项 / 升级项（建议负责人）**：
  1. **`prompts/codex-judge.md` 与 `codex-report.md` 是否汉化** —— 本批判定 no-op，
     前置条件见 §3 末尾（需改 `eval.py:569/589` 源码 + 重基线），**请编排者裁定并另派任务**。
  2. **§8 报告的 6 条设计原则两文件中译名不一致** —— 建议由编排者裁定统一口径后另派任务，
     本批未擅自改写已中文段落。
  3. `goal.md:61` 的 `  (use \`-p rustcode\`, NOT \`-p rustcode-cli\`)` 与
     `DEVENV.md:49` 的 `- npm: \`https://registry.npmmirror.com\`` 在基线上**未被自检判为 en 行**
     （`ASCII4_RE` 要求连续 4 个以上 ASCII 字母，这两行剥离 inline code 后不足 4 连字母），
     按铁律 §3「只处理 check 报出的 offender 行」本批**未改动**。若编排者希望彻底清零英文，
     需另派任务并同步更新基线。
  4. `CONTEXT.md` 的中文释义（如「MCP 就绪状态」「视图投影」）若与团队既有术语表冲突，
     建议由架构负责人复核后统一；本批释义已与 `docs/archive/`、`docs/plans/` 既有中文对齐（见 §4）。


## PASS `.goals/rustcode-migration-finalize/inspector-feedback-1.md`

- en=0/31 ratio=0.0000

## PASS `.goals/rustcode-migration-finalize/summary.md`

- en=0/50 ratio=0.0000

## PASS `CONTEXT.md`

- en=0/45 ratio=0.0000

## PASS `DEVENV.md`

- en=0/23 ratio=0.0000

## 附录 · 被 R5 跳过的行（供抽检）

共 1 行。

- `.goals/rustcode-migration-finalize/goal.md:32`: `    'mcp::registry::tests::trust_key_golden_matches_core_algorithm' 除外)`

---

# D-36 手写小结（在最后一次自检之后追加）

## 1. 本批范围、前提与自检命令

- 任务 ID：**D-36**（批次 13）。基线 `ZH_BASE=3ee655e3`，自检工具 `scripts/check-zh-docs.py`。
- files_owned 共 9 个文件，**全部无 YAML frontmatter**（首行均不是 `---`）：
  铁律 §2.7 的「YAML 键名一律保留英文」本批无适用对象，AC-6 键名集合在基线与工作区
  **皆为空集、相等**，无 `description` 需按 AC-6 逐条列示。
- 最后一次自检命令（本报告上面的自动生成段即由该命令产出）：

```bash
python3 scripts/check-zh-docs.py check --base 3ee655e3 \
  --report .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-36.md \
  --files evals/deepseek-v4-flash/README.md \
  evals/deepseek-v4-flash/prompts/codex-judge.md \
  evals/deepseek-v4-flash/prompts/codex-report.md \
  evals/deepseek-v4-flash/cases/agent-fixture/ARCHITECTURE.md \
  .goals/rustcode-migration-finalize/goal.md \
  .goals/rustcode-migration-finalize/inspector-feedback-1.md \
  .goals/rustcode-migration-finalize/summary.md CONTEXT.md DEVENV.md
```

- 实测输出：`check: 受检 9，PASS 7，FAIL 2`（2 个 FAIL 为本批**判定为 no-op 的评测 prompt 模板**，见 §3）。
- 开工前已先跑一次自检拿 offender 清单（本批特殊说明第 5 条），据此区分了
  「纯英文需全文译」（4 个）与「已中文只需清 offender」（3 个）+「运行时载荷 no-op」（2 个）。

## 2. 每个文件的改动量（含 no-op 判定）

| 文件 | 基线 en/total | 现 en/total | 改动性质 | 说明 |
|---|---|---|---|---|
| `evals/deepseek-v4-flash/README.md` | 19/19 | **0/17** | **纯英文 → 全文译** | 命令/flag/路径/产物名全保留；段落内重排使物理行数 19→17（§5.7 允许） |
| `evals/deepseek-v4-flash/prompts/codex-judge.md` | 6/6 | 6/6 | **no-op** | 评测运行时载荷，见 §3 |
| `evals/deepseek-v4-flash/prompts/codex-report.md` | 12/12 | 12/12 | **no-op** | 评测运行时载荷 + 硬断言，见 §3 |
| `evals/deepseek-v4-flash/cases/agent-fixture/ARCHITECTURE.md` | 4/4 | **0/3** | **纯英文 → 全文译** | `Runtime`/`Driver`/`legacy.py` 原样保留；属 fixture 数据，风险见 §10 |
| `.goals/rustcode-migration-finalize/goal.md` | 17/59 | **0/59** | 已中文 → 只清 offender（17 行） | 6 个英文标题 + 11 行约定条目；commit hash 与全部 inline code 未动 |
| `.goals/rustcode-migration-finalize/inspector-feedback-1.md` | 31/31 | **0/31** | **纯英文 → 全文译** | 表格结构、`PASS`、全部 `grep`/`cargo` 命令 inline code 原样保留 |
| `.goals/rustcode-migration-finalize/summary.md` | 37/50 | **0/50** | 混合 → 只清 offender（37 行） | 已中文的 6 条设计原则条目未改写（铁律 §2.8） |
| `CONTEXT.md` | 30/45 | **0/45** | 已中文 → 术语行补中文释义（30 行） | **英文术语一字未改**，见 §4 |
| `DEVENV.md` | 3/23 | **0/23** | 已中文 → 只清 offender（3 行） | 表格两行工具名 + 一行镜像说明补中文 |

汇总：`git diff --stat` = **7 files changed, 142 insertions(+), 145 deletions(-)**（2 个 prompt 文件 no-op，零 diff）。
净行数 -3 全部来自段内重排（`README.md` 19→17、`ARCHITECTURE.md` 4→3 非空白行）；
已用逐行「空/非空」序列对拍确认：**空行位置一字未变**（仅 `README.md` 两段各少 1 个物理行），
符合铁律 §5.4 与 §5.7。

## 3. `prompts/*.md` 的处理决定：判定为运行时载荷，记 no-op（单独说明理由）

**本批对 `evals/deepseek-v4-flash/prompts/codex-judge.md` 与 `codex-report.md` 未做任何改动。**
判定依据（逐条实测，非臆测）：

1. **两者都不是给人看的说明，而是被 `eval.py` 在运行时读取并拼进模型请求的载荷**：
   - `eval.py:547`：`template = (ROOT/"prompts/codex-judge.md").read_text()`，随后
     `prompt = template + "\n\nEvaluation packet:\n" + packet_path.read_text()` 送 `codex_exec`。
   - `eval.py:567` 与 `eval.py:587`：`(ROOT/"prompts/codex-report.md").read_text()+"\n\nData:\n"+json.dumps(...)`。
   即：**改动词面 = 改变评测行为**，正属本批特殊说明第 3 条所说的「纯 prompt 指令」。
2. **`codex-report.md` 存在硬断言，译文会直接破坏门禁（决定性证据）**：
   `eval.py:569` 与 `eval.py:589` 用
   `required = ("Executive conclusion", "Capability", "Stability", "Deployment recommendation")` /
   `("Executive conclusion", "Capability", "Stability", "cache", "Deployment recommendation")`
   对生成报告做 `x.lower() in answer.lower()` 断言，缺失即
   `raise ValueError("Codex report is missing required sections")`。
   而 `codex-report.md:4-5` 的 `Executive conclusion`、`Capability`、`Stability`、
   `latency and cache hit rate`、`Deployment recommendation` 正是这些被代码消费的**章节标识符**。
   翻译它们会让 `eval.py report` / `combined-report` 直接抛错。
3. **`codex-judge.md` 含机器解析的 JSON 契约字面量**：第 5 行整行是
   `{"winner":"A|B|tie","scores":{...},"confidence":0.0}`，由 `eval.py:552` 的
   `extract_json(answer)` + `validate_judgment(value)` 消费。该行按铁律 §2.3 本就不得翻译；
   而它是裁判指令的核心，只译散文、留 JSON 会造成「中英混排的裁判指令」，
   裁判口径一变，成对对比的分值就不再可比。
4. **仓库既有先例支持「运行时载荷不译」**：`scripts/check-zh-docs.py:28` 把
   `crates/rustcode-review/rules/` 列为 `SKIP_B`，注释即「运行时载荷，Q1 不动」。
   评测 prompt 模板属同类资产。

**因此本批按「不要贸然全译」记 no-op，并在此上报编排者（升级项，见 §10）。**
若编排者确认仍需汉化，前置条件（均超出本批「只改 9 个 md」的边界）：
(a) 同步修改 `eval.py:569`/`eval.py:589` 的 `required` 元组（源码改动）；
(b) 保留 `codex-judge.md` 的 JSON 字面量与 `codex-report.md` 的章节标识符原词；
(c) 重新基线化评测结果。请另派任务，本批不擅自处理。

## 4. `CONTEXT.md` 的处理决定：术语零改动，只补中文释义（单独说明理由）

`AGENTS.md:55` 明确 `CONTEXT.md` 是运行时领域术语的**事实源**，且 `_Avoid_` 条目是**硬性命名约束**。
本批对该文件的处理口径如下：

- **14 个领域术语（Coding Runtime、Live View、Live View Hub、Runtime Binding、
  Session Transition、MCP Scope、Tool Catalog、Tool Catalog Revision、
  Turn Tool Snapshot、MCP Readiness、View Projection、Committed Snapshot、
  Replay Window、Pending Interaction）英文原词一字未改**，包括大小写与空格。
- **14 条 `_Avoid_` 的禁用词列表（如 `LiveSession`、`Fire-and-forget session command`、
  `Runtime generation`、`Global approval slot`）英文原词一字未改**；
  `_Avoid_` 标签本身也保留（它是该事实源的结构性键名，便于后续工具识别）。
- 为同时满足 AC-2（`en/total <= 0.05`）与上述命名约束，采用
  「**`**英文原词**（中文释义）**」与「**`_Avoid_（避免使用）：<禁用词原样>`**」两种形式：
  即**只在原英文之后追加中文释义，不替换、不删减、不加反引号**。
- 该文件**没有任何 backtick**（基线 inline code 集合为空集），本批也未新增任何反引号，
  故 AC-4 在该文件上恒等成立；**后续任何人给这些术语加反引号都会立刻 AC-4 FAIL，请勿「顺手整理」。**
- 中文释义与仓库既有译法对齐：`会话切换`（见 `docs/plans/2026-08-17-project-input-history-design.md:53`）、
  `工具目录`（见 `docs/plans/2026-07-26-atomgit-production-tools.md:7`）、
  `已提交快照`/`回放窗口`/`实时视图` 与该术语在 `docs/archive/live-transport-convergence-plan.md:35,72`
  中出现的中文语境一致。

## 5. 残留行清单与白名单判定

| 文件 | 残留 en 行 | 为何属于白名单 |
|---|---|---|
| `evals/deepseek-v4-flash/prompts/codex-judge.md` | 6/6 | 评测运行时载荷 + 机器解析的 JSON 契约，见 §3。**按判定记 no-op，非遗漏** |
| `evals/deepseek-v4-flash/prompts/codex-report.md` | 12/12 | 评测运行时载荷 + `eval.py:569/589` 硬断言的章节标识符，见 §3。**按判定记 no-op，非遗漏** |

其余 7 个文件 **en = 0**，无残留。

另有三类「刻意保留、不算残留」的内容：

- **fenced code block 内一字未动**：`evals/deepseek-v4-flash/README.md` 的 eval.py 命令块、
  `DEVENV.md` 的 quickstart / cargo / 环境变量块。
- **inline code 原样保留**（摘选）：`--ephemeral --output-format jsonl`、`--no-tools`、
  `rustcode`、`codex`、`benchmark.json`、`--case smoke-model --repetitions 1`、
  `--dangerously-skip-permissions`、`allow_edits = true`、`rustcode_bin`、
  `../../target/debug/rustcode`、`case.json`、`prompt.md`、`fixture`、`verify`、
  `summary.json`、`report.md`、`events.jsonl`、`results/`；
  `Runtime`、`Driver`、`legacy.py`；
  `grep -rn "atomcode" crates/ scripts/ .github/`、`grep -rn "atomcode" docs/architecture.md`、
  `grep -ri "sentry\|posthog\|segment\|analytics"`、`install_panic_hook`、
  `impl Default for Locale`、`Locale::ZhCn`、`cargo test -p rustcode-config --lib`、
  `# language = "zh_CN"`、`cli_flag_unparseable_falls_through`、`LOCALE`、
  `resolve_initial_locale_with_env`、`Config.language`、`Option<Locale>`、`#[serde(default)]`、
  `None`、`Locale::default()`、`is_codingplan_llm_gateway`、
  `RUSTCODE_CODINGPLAN_LLM_BASE_URL`、`bearer_auth(api_key)`、`atomgit`、
  `rustcode-telemetry`、`current_locale()`、`LANG=C`/`POSIX`、
  `mcp::registry::tests::trust_key_golden_matches_core_algorithm`、
  `crates/rustcode-config/src/locale.rs`、`crates/rustcode-config/src/i18n/mod.rs`、
  `docs/config.example.toml`、`webui/package-lock.json`、`crates/rustcode-cli/`、
  `type(scope): description`、`RUSTCODE_HOME`、`af7e8715723aa33343f049b15d3032e71200567c`。
- **专有名词/版本号/日期保留**：RustCode、DeepSeek V4 Flash、Volcano Engine、Codex、
  Python 3.7+、Node、Rust、GitHub、npm、cargo、ASCII、Unicode、AtomGit、PASS、SHA、dev branch。

## 6. 四项实测结果

| 项 | 阈值/口径 | 实测 | 结论 |
|---|---|---|---|
| **AC-2** | `en/total <= 0.05` | 7 个文件：`0/17`、`0/3`、`0/59`、`0/31`、`0/50`、`0/45`、`0/23` 全部 0.0000；2 个 prompt 文件 `6/6`、`12/12`（no-op） | **PASS 7/9；FAIL 2/9 为 §3 的判定性 no-op** |
| **AC-4** | inline code + fenced code 多重集与基线完全相等 | 9 个文件**无一报差异**（含 2 个零 diff 的 no-op 文件） | **PASS 9/9** |
| **AC-7b** | `](...)` 链接目标多重集相等 | 9 个文件无一报差异（本批 9 个文件中仅 `DEVENV.md` 含链接，未改动其目标） | **PASS 9/9** |
| **AC-32** | Emoji 数不增加 | 9 个文件无一报 `emoji 命中数增加`；本批未新增、也未清理既有 Emoji | **PASS 9/9** |

附带核对：

- **AC-6**：9 个文件均无 frontmatter，键名集合基线与工作区皆为空集，相等；无 `description` 变更。
- **未踩坑复核**：本批复用了前批教训——未把普通词包成反引号（§5.1）；
  未只加全角标点（`summary.md:36` 首轮写成「（explorer/builder/reviewer，max_concurrent=4）」
  仍被判 en，因 CJK 正则只认汉字不认全角标点，已按 §5.2 补真实汉字「三个角色」后转 PASS，
  见 §7）；未做批量替换（§5.3）；未增删空行（§5.4，已用空/非空序列对拍确认）；
  手写小结在最后一次自检之后追加（§5.5）；本批 9 个文件**无跨行反引号对**，不适用 §5.6。
- **范围**：只改 files_owned 的 9 个文件；未改源码；未执行 `cargo`/`npm` 构建与测试（铁律 §7）。

## 7. 本批唯一踩坑：`summary.md:36` 的全角标点不算 CJK

首轮把该行译为 `  （explorer/builder/reviewer，max_concurrent=4）`，自检仍报 AC-2 残留 1 行。
成因与铁律 §5.2 完全一致：`scripts/check-zh-docs.py:40` 的
`CJK_RE = [\u3400-\u4dbf\u4e00-\u9fff\uf900-\ufaff\u3040-\u30ff]` **只匹配汉字与假名，不含全角标点**，
而 `ASCII4_RE = [A-Za-z]{4,}` 命中 `explorer`，于是该行仍被判英文行。
**处置**：补入真实汉字，改为 `  （explorer/builder/reviewer 三个角色，max_concurrent=4）`，随后转 PASS。
`goal.md:29` 的同类行因首轮即写为「（explorer/builder/reviewer 三个角色）」，未触发该坑。

## 8. 术语与命名一致性检查结论

| 术语 | 本批译法 | 一致性依据 |
|---|---|---|
| paired evaluation / harness / case / candidate / repetition | 配对评测 / 评测框架 / 用例 / 候选 / 重复 | 与 `evals/deepseek-v4-flash/cases/agent-cases.json`、`eval.py` 的字段名语义一致 |
| fixture | fixture（不译） | `eval.py:126-129`、`case.json` 的 `fixture` 键名与目录名，铁律 §2.3 |
| preflight | 预检 | 与 `README.md` 中 `--case smoke-model` 的用途一致 |
| session / working directory | session / working directory（不译） | `CONTEXT.md` 事实源内已有此写法，保持跨文档一致 |
| provider | provider（不译） | 与 `DEVENV.md`、「第三方 OpenAI/Anthropic 兼容端点」表述一致 |
| Acceptance Criteria / Quality Gates | 验收标准 / 质量门禁 | 与 `AGENTS.md` 的 G1/G3/G6/G7/G8 门禁编号口径一致 |
| Initial SHA | 初始 SHA | `SHA` 为标识符，保留 |
| Inspector Feedback | 检查员反馈 | 与 `.goals/` 目录下该文件作为审查记录的角色一致 |
| 运行时领域术语 | **保留英文原词 + 中文释义** | `AGENTS.md:55`「以 `CONTEXT.md` 为准」，见 §4 |

**发现一处源文本自身的术语不一致，本批按原文直译、未擅自统一，在此上报**：
`inspector-feedback-1.md` 的 6 条设计原则英文为
`specialization / clear boundaries / context inheritance / parallel processing / independent verification / cost optimization`，
本批译为「专业化分工、清晰边界、上下文继承、并行处理、独立验证、成本优化」；
而同 feature 的 `summary.md` 已中文段落把同样两条写作
「高效通信 (context inheritance + result passing)」与「质量保证 (reviewer = independent verification)」。
其中 4 条（专业化分工 / 清晰边界 / 并行处理 / 成本优化）两文件一致，**2 条不一致**。
本批遵守铁律 §2.8「已中文的段落不改写」，未改 `summary.md` 的既有中文；是否统一由编排者裁定。

## 9. 文档更新清单

| 文件路径 | 变更类型 | 摘要 |
|---|---|---|
| `evals/deepseek-v4-flash/README.md` | 修订（全文英译中） | 评测框架说明汉化；命令、flag、路径、产物名保留 |
| `evals/deepseek-v4-flash/prompts/codex-judge.md` | **无变更（no-op）** | 评测运行时载荷，见 §3 |
| `evals/deepseek-v4-flash/prompts/codex-report.md` | **无变更（no-op）** | 评测运行时载荷 + `eval.py:569/589` 硬断言，见 §3 |
| `evals/deepseek-v4-flash/cases/agent-fixture/ARCHITECTURE.md` | 修订（全文英译中） | fixture 架构说明汉化；`Runtime`/`Driver`/`legacy.py` 保留 |
| `.goals/rustcode-migration-finalize/goal.md` | 修订 | 6 个英文标题 + 11 行项目约定条目补中文 |
| `.goals/rustcode-migration-finalize/inspector-feedback-1.md` | 修订（全文英译中） | 检查员反馈汉化；表格结构、`PASS`、命令证据保留 |
| `.goals/rustcode-migration-finalize/summary.md` | 修订 | 5 个英文标题 + 32 行条目补中文；已中文段落未改写 |
| `CONTEXT.md` | 修订 | 标题、章节名、14 个术语行、14 条 `_Avoid_` 行补中文释义；英文术语零改动 |
| `DEVENV.md` | 修订 | 工具表 2 行 + cargo 镜像 1 行补中文 |

## 10. 回滚方案

本批为纯文档汉化，改动均未提交（工作区改动）。按优先级：

1. **整体回滚本批（推荐）**：
   `git checkout -- evals/deepseek-v4-flash/README.md evals/deepseek-v4-flash/cases/agent-fixture/ARCHITECTURE.md .goals/rustcode-migration-finalize/goal.md .goals/rustcode-migration-finalize/inspector-feedback-1.md .goals/rustcode-migration-finalize/summary.md CONTEXT.md DEVENV.md`
   判定时机：任一文件出现 AC-4/AC-7b/AC-32 FAIL 且无法就地修复，或编排者否决本批译文。
2. **单文件回滚**：对上表中任一路径单独执行 `git checkout -- <path>`。
   判定时机：仅个别文件译文被否决（例如 `CONTEXT.md` 的术语释义口径需要架构负责人复核）。
3. **回滚后全量校验**：`git stash` 后重跑 §1 的自检命令，用于排除本批之外的干扰。
4. **无需回滚数据/配置/代码**：本批未触碰源码、未改任何配置文件与版本号、无数据迁移，回滚代价为零；
   两个 prompt 文件本就是零 diff，不存在回滚问题。

## 11. 已知未验证范围、风险与升级项

- **未验证**：未执行 `cargo` / `npm` 构建与测试（铁律 §7 明确禁止；本批为纯文档任务）。
  因此 `README.md` 中的 `python3 eval.py ...` 命令序列、`DEVENV.md` 中的 cargo/npm 命令
  **仅按原文保留，未实际执行验证**。
- **未验证**：`inspector-feedback-1.md` 与 `summary.md` 记录的验证结论
  （`cargo test -p rustcode-config --lib` = 324 项通过、`grep` = 0 处命中等）
  是**历史记录，本批未重跑、也未核对当前实现**；本批只做语言转换，未改任何数字与结论。
- **风险（须编排者知悉）**：`cases/agent-fixture/ARCHITECTURE.md` 是被
  `evals/deepseek-v4-flash/cases/agent-cases.json` 中 `agent-context-contract` 用例
  「Read ARCHITECTURE.md ...」显式读取的 **fixture 数据**。
  本批汉化了它，会改变被测模型读到的输入。已确认的缓解因素：
  (a) 语义逐句对等，未改任何架构约束（单一状态所有权、依赖方向、`legacy.py` 单向导入器）；
  (b) 同一配对的**两个候选拿到完全相同的 fixture**，成对对比仍然公平；
  (c) 判定只看行为（`Driver` 是否把变更委托给 `Runtime`），不看文档语言；
  (d) `eval.py:328` 的 `diff -ruN` 是拿 fixture 与工作副本比对，本批改动落在 fixture 基线上，不产生额外 diff。
  若编排者认为 eval fixture 应一律保持英文，请单独回滚该文件（`git checkout -- <path>`），成本为零。
- **遗留后续项 / 升级项（建议负责人）**：
  1. **`prompts/codex-judge.md` 与 `codex-report.md` 是否汉化** —— 本批判定 no-op，
     前置条件见 §3 末尾（需改 `eval.py:569/589` 源码 + 重基线），**请编排者裁定并另派任务**。
  2. **§8 报告的 6 条设计原则两文件中译名不一致** —— 建议由编排者裁定统一口径后另派任务，
     本批未擅自改写已中文段落。
  3. `goal.md:61` 的 `  (use \`-p rustcode\`, NOT \`-p rustcode-cli\`)` 与
     `DEVENV.md:49` 的 `- npm: \`https://registry.npmmirror.com\`` 在基线上**未被自检判为 en 行**
     （`ASCII4_RE` 要求连续 4 个以上 ASCII 字母，这两行剥离 inline code 后不足 4 连字母），
     按铁律 §3「只处理 check 报出的 offender 行」本批**未改动**。若编排者希望彻底清零英文，
     需另派任务并同步更新基线。
  4. `CONTEXT.md` 的中文释义（如「MCP 就绪状态」「视图投影」）若与团队既有术语表冲突，
     建议由架构负责人复核后统一；本批释义已与 `docs/archive/`、`docs/plans/` 既有中文对齐（见 §4）。

