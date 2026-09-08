# zh-check 报告

- 基线 (--base): `3ee655e3`
- 阈值 (--en-ratio): `0.05`
- 判定: AC-2 / AC-4 / AC-6 / AC-7b / AC-32，任一失败即 FAIL（fail-closed）

## 汇总

- 受检文件: 3
- PASS: 3
- FAIL: 0

## PASS `docs/superpowers/plans/2026-06-29-acp-agent.md`

- en=0/264 ratio=0.0000

## PASS `docs/superpowers/plans/2026-06-27-v2-rate-limit-pause-resume.md`

- en=0/234 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-25-round-cap-checkpoint.md`

- en=0/188 ratio=0.0000

---

# 人工小结（D-04 汉化）

本小结由 doc-writer 手写，覆盖任务要求的三项内容：每文件改动量、残留行清单及白名单理由、四项实测结果。数据与上文脚本输出一致，未做二次加工。

## 1. 各文件改动量

| 文件 | 行数（改前→改后） | 改动行数 | en（改前） | en（改后） |
|---|---|---|---|---|
| `docs/superpowers/plans/2026-06-29-acp-agent.md` | 1080 → 1080 | **245** | 222 / 264 = 0.8409 | **0 / 264 = 0.0000** |
| `docs/superpowers/plans/2026-06-27-v2-rate-limit-pause-resume.md` | 958 → 958 | **91** | 75 / 234 = 0.3205 | **0 / 234 = 0.0000** |
| `docs/superpowers/plans/2026-07-25-round-cap-checkpoint.md` | 706 → 706 | **46** | 34 / 188 = 0.1809 | **0 / 188 = 0.0000** |
| 合计 | 2744 → 2744 | **382** | 331 | **0** |

三个文件均为纯中文替换，未做中英并列、未保留英文原段落；行数、标题层级、文件名、YAML frontmatter（三文件均无）均未变动。

### 1.1 改动分类（按基线行形态归类）

`2026-06-29-acp-agent.md`（245 行）：

| 类别 | 行数 | 处理方式 |
|---|---|---|
| `- [ ] **Step N: …**` 步骤标题 | 59 | `- [ ] **步骤 N：中文标题**` |
| 说明性长句与正文段落（目标/架构/约束/注释说明等） | 60 | 逐句译出 |
| `Run: …` / `Then: …` 命令引导 | 24 | `运行：…` / `然后：…` |
| `Expected: …` 预期引导 | 22 | `预期：…` |
| `**Files:**` 清单标题 | 11 | `**文件：**` |
| `**Interfaces:**` 清单标题 | 11 | `**接口：**` |
| `### Task N: …` 任务标题 | 11 | `### 任务 N：中文标题` |
| `- Modify: …` | 12 | `- 修改：…` |
| `- Test: …` | 10 | `- 测试：…` |
| `- Produces: …` / `- Consumes: …` | 9 / 5 | `- 产生：…` / `- 消费：…` |
| `- Create: …` | 7 | `- 新建：…` |
| 其它标题（`# …`、`## 全局约束`、`### 试点…`、`## 自查`） | 4 | 同级中文标题 |

`2026-06-27-v2-rate-limit-pause-resume.md`（91 行）：步骤标题 37、`Run: …` 16、`**Files:**` 9、`**Interfaces:**` 8、`Expected: …` 4、`- Consumes: …` 5、`- Produces: …` 3、`- Modify: …` 2、`- Create: …` 1、`- Test: …` 1、说明性长句 3（`**Tech Stack:**`→`**技术栈：**`、`**Spec coverage：**`→`**Spec 覆盖：**` 等）、其它标题 2（`## 全局约束`、`## 自查`）。

`2026-07-25-round-cap-checkpoint.md`（46 行）：`Run: …` 12、步骤标题 10、`**Files:**` 5、`**Interfaces:**` 5、`- Test: …` 4、`- Consumes: …` 4、`- Modify: …` 1、`- Produces: …` 1、说明性长句 1、其它标题 3（`## 全局约束`、`## 文件结构`、`## 延后项`）。

术语遵循项目约束：中文正文 + 英文原样符号。`rustcode-acp` / `agent-client-protocol` / `rustcode-kernel` / `rustcode-coding` / `rustcode-capabilities` 等 crate 名与路径、`AgentEvent` / `SessionUpdate` / `ToolKind` / `StopReason` / `PromptRequest` / `ApprovalResponse` / `AgentCommand::Cancel` / `ROUND_CAP_CHECKPOINT_KIND` / `TurnEvent::RateLimited` / `LiveWireEvent` / `UiLine` / `UiPhase::RoundCap` 等类型与变体、`session/update` / `session/request_permission` / `session/new` / `session/cancel` / `session/prompt` 等 API 路径、`CARGO_INCREMENTAL=0 cargo test -p …`、`cargo build --workspace`、`--provider` / `--model` 等命令与 flag、`RUSTCODE_TURN_MAX_ROUNDS` 与 `[coding] max_rounds` 等环境变量与配置键、OpenAI / Anthropic / Ollama / Rust / tokio / ACP / JSON-RPC / stdio / Zed 等第三方专名、版本号与日期数字一律原样保留。

## 2. 残留行清单与白名单理由

自检口径（AC-2）下三文件英文残留均为 **0 行**。以下是肉眼可见的「非中文内容」，全部命中铁律白名单，**不属于残留缺陷**：

| 残留位置 | 内容 | 属于白名单的理由 |
|---|---|---|
| 三文件共 102 个代码块（围栏内全部行） | Rust / bash / TOML 源码、注释、字符串 | 铁律 1：不翻译 fenced code block 内任何内容，含注释与字符串 |
| 三文件共 1132 处 inline code（378 + 371 + 383） | 路径、crate 名、函数名、命令、flag、类型与变体名等 | 铁律 2：不翻译 inline code |
| 译文中保留的英文术语（crate 名、类型/变体名、API 路径、命令、环境变量、配置键、Zed / JSON-RPC / ACP / stdio 等专名、版本号与日期） | 见第 1 节术语说明 | 铁律 3：保留原文 |
| 水平分隔线 `---` 与表格分隔行 | 排版符号 | 无语言属性，改写会破坏 Markdown 结构 |
| 文件 2、3 中原本已含中文的行（见 2.1） | `Expected: …中文`、`### Task N: …中文`、`- Modify: …（中文）` 等引导词 | 铁律 8：已中文的段落不润色、不改写，只处理英文段落 |

### 2.1 铁律 8 导致的引导词混用（需编排者确认）

文件 2、3 原本是中英混排文档：同一类标签在「纯英文行」与「已含中文的行」上同时存在。本次只译了纯英文行，已含中文的行原样保留，因此产生了 `预期：` / `Expected:`、`运行：` / `Run:`、`- 修改：` / `- Modify:` 的混用。逐条清单如下（均为改动前的既有中文行，本次一字未动）：

- `2026-06-27-v2-rate-limit-pause-resume.md`：共 58 行 —— `Expected: ` 12 行（82、184、225、320、405、491、579、622…）、`- Modify: ` 13 行（27、28、200、274、595、596、668、741…）、`### Task N: ` 9 行（24、197、271、419、592、665、738、836…）、`- [ ] **Step N:**` 8 行（821、848、863、871、888、914、919、923）、`- Test: ` 7 行（29、201、275、597、669、743、842）、`- Produces: ` 5 行（205、279、429、747、846）、`- Consumes: ` 2 行（278、845）、`**Goal:**` / `**Architecture:**` 2 行（5、7）。
- `2026-07-25-round-cap-checkpoint.md`：共 60 行 —— `- [ ] **Step N:**` 23 行（47、147、162、185、241、266、306、323…）、`Expected: ` 12 行（145、239、244、304、353、416、475、560…）、`- Modify: ` 10 行（39、40、257、258、368、369、488、489…）、`## Task N: ` 6 行（36、254、365、485、581、670）、`- Produces: ` 3 行（44、374、495）、`**Goal:**` / `**Architecture:**` / `**Tech Stack:**` 3 行（5、7、9）、`- Test: ` 1 行（260）、`- Consumes: ` 1 行（45）、`Run: ` 1 行（679，该行后半为中文）。

这与仓库既有汉化惯例一致：`docs/superpowers/plans/2026-05-29-webui.md:354`、`2026-04-23-cadence-reflection.md:18`、`2026-07-11-persistent-todo-panel.md:864` 等已汉化文件同样保留 `- Create: ` / `- Modify: ` / `**Spec coverage:**` 引导词；D-01（`2026-05-25-tuix-unified-in-app-scroll.md`）也按铁律 8 原样保留了 `**Goal:**` / `**Architecture:**` / `**Tech Stack:**`。**若编排者希望统一为全中文引导词，需明确授权后我再改**——那会触及 118 行已含中文的既有内容。

## 3. 自检四项实测结果

执行命令（与任务给定命令一致）：

```bash
python3 scripts/check-zh-docs.py check --base 3ee655e3 \
  --report .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-04.md \
  --files docs/superpowers/plans/2026-06-29-acp-agent.md \
          docs/superpowers/plans/2026-06-27-v2-rate-limit-pause-resume.md \
          docs/superpowers/plans/2026-07-25-round-cap-checkpoint.md
```

脚本输出：`PASS … en=0/264=0.0000` / `PASS … en=0/234=0.0000` / `PASS … en=0/188=0.0000` / `check: 受检 3，PASS 3，FAIL 0`，退出码 0。

| 验收项 | 判据 | 实测（acp-agent / rate-limit / round-cap） | 结论 |
|---|---|---|---|
| AC-2 | `en/total <= 0.05` | `0/264=0.0000`（基线 0.8409）、`0/234=0.0000`（基线 0.3205）、`0/188=0.0000`（基线 0.1809） | PASS |
| AC-4 | inline code + fenced code 集合与改动前完全相等 | code span 多重集 `914=914` / `834=834` / `728=728` 逐项相等；inline code 出现次数 `378=378` / `371=371` / `383=383`；代码块数 `37/36/29` 与围栏行 `74/72/58` 不变，围栏内所有行逐字节未变 | PASS |
| AC-7b | `](...)` 链接目标多重集相等 | `0=0` / `0=0` / `0=0`（三文件均无任何 Markdown 链接，不存在被改写的链接目标） | PASS |
| AC-32 | Emoji 数不增加 | `0→0` / `0→0` / `3→3`（文件 3 原有 3 处图标未清理、未新增） | PASS |

附加核对（非本任务硬性门禁，一并记录）：AC-6 frontmatter 键名集合三文件均为 `[] = []`（无 YAML frontmatter，与铁律 7 一致）；三文件总行数 `1080/958/706` 改前改后一致；`git diff` 中不含任何围栏行（`grep` 计数 0）；文件名与标题层级未变。

## 4. 未验证范围

- 未执行 `cargo` / 构建 / 测试：本任务为纯文档汉化，不涉及源码与测试。
- 代码块内的英文（含注释与字符串）按铁律一律未译；其中若有面向读者的说明性英文注释，需另行确认是否翻译。
- 文件 2、3 共 118 行「已含中文 + 英文引导词」的行按铁律 8 未改动，混用情况见 2.1，需编排者授权后才统一。
- 与这三个 plan 同源的 spec（`docs/superpowers/specs/`）不在本次 `files_owned` 内，未检查、未改动；若后续汉化，任务/步骤标题措辞应与本批保持一致。
- 同目录下另有 7 个 plan 文件在工作区中处于已修改状态（`2026-05-08-vision-preprocessor*.md`、`2026-05-25-tuix-unified-in-app-scroll.md`、`2026-05-29-webui.md`、`2026-07-11-persistent-todo-panel.md`、`2026-07-13-selectable-approval.md`、`2026-07-22-multi-question-request-user-input.md`），属 D-01~D-03 等并行任务的产物，本次未读写。
