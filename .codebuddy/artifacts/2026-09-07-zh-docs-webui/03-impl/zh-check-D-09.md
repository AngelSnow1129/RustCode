# zh-check 报告

- 基线 (--base): `3ee655e3`
- 阈值 (--en-ratio): `0.05`
- 判定: AC-2 / AC-4 / AC-6 / AC-7b / AC-32，任一失败即 FAIL（fail-closed）

## 汇总

- 受检文件: 7
- PASS: 7
- FAIL: 0

## PASS `docs/superpowers/plans/2026-07-25-retire-core-provider-B-vision.md`

- en=0/110 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-06-double-esc-undo-cooldown.md`

- en=0/62 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-22-brainstorming-request-user-input.md`

- en=0/78 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-25-retire-core-tool-ball-D.md`

- en=0/67 ratio=0.0000

## PASS `docs/superpowers/2026-07-27-release-v5.0.3-test-checklist.md`

- en=0/93 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-22-batch-user-questions-persona-nudge.md`

- en=0/52 ratio=0.0000

## PASS `docs/superpowers/plans/2026-04-19-tuix-ink-cell-diff.md`

- en=0/93 ratio=0.0000

---

# D-09 汉化小结（人工撰写）

任务：把 `docs/superpowers/` 下 6 个实施计划 + 1 个测试清单里的**英文正文**译为简体中文，纯中文替换、不做中英并列、不保留英文原文段落。基线 `3ee655e3`，受检文件即本次 `files_owned` 的 7 篇。

## 1. 每个文件改动量

| 文件（`docs/superpowers/` 下） | 总行数 | 改动行（+/-，vs `3ee655e3`） | 汉化前 en/total | 汉化后 en/total | 判定 |
|---|---|---|---|---|---|
| `plans/2026-07-25-retire-core-provider-B-vision.md` | 517 | +83 / -83 | 16/110 = 0.1455 | 0/110 = 0.0000 | 已汉化 |
| `plans/2026-07-06-double-esc-undo-cooldown.md` | 244 | +30 / -30 | 9/62 = 0.1452 | 0/62 = 0.0000 | 已汉化 |
| `plans/2026-07-22-brainstorming-request-user-input.md` | 220 | +73 / -73 | 65/78 = 0.8333 | 0/78 = 0.0000 | 已汉化（全文翻译） |
| `plans/2026-07-25-retire-core-tool-ball-D.md` | 128 | +44 / -44 | 7/67 = 0.1045 | 0/67 = 0.0000 | 已汉化 |
| `2026-07-27-release-v5.0.3-test-checklist.md` | 113 | **0 / 0** | 0/93 = 0.0000 | 0/93 = 0.0000 | **no-op** |
| `plans/2026-07-22-batch-user-questions-persona-nudge.md` | 135 | +48 / -48 | 44/52 = 0.8462 | 0/52 = 0.0000 | 已汉化（全文翻译） |
| `plans/2026-04-19-tuix-ink-cell-diff.md` | 205 | +29 / -29 | 10/93 = 0.1075 | 0/93 = 0.0000 | 已汉化 |

改动量取自 `git diff --numstat 3ee655e3 -- <file>`。

**no-op 判定依据（checklist）**：该文件在基线 `3ee655e3` 上首轮自检即为 `PASS ... en=0/93=0.0000`，`en/total = 0.0000 <= 0.05`，`check` 未报出任何 en offender 行。按任务约定「已是中文，只处理 `check` 报出的英文 offender 行；若 `en/total <= 0.05` 则记 no-op」，本次**一个字符都未改动**（`git diff --numstat` 为空），未改写任何已中文段落。

六篇已改文件均为**纯行内替换**：行数、标题数与层级、复选框数、fence 行数与基线逐项相等（见下表），文件名与计划日期未改动。

| 文件 | 标题数 base→work | 复选框数 base→work | fence 行数 base→work |
|---|---|---|---|
| B-vision | 7 → 7 | 24 → 24 | 22 → 22 |
| double-esc | 4 → 4 | 6 → 6 | 30 → 30 |
| brainstorming | 7 → 7 | 12 → 12 | 18 → 18 |
| tool-ball-D | 6 → 6 | 15 → 15 | 14 → 14 |
| checklist（no-op） | 17 → 17 | 60 → 60 | 0 → 0 |
| batch | 6 → 6 | 6 → 6 | 8 → 8 |
| ink-cell-diff | 17 → 17 | 0 → 0 | 10 → 10 |

本批主要翻译对象（与已完成的 D-05 等批次保持同一套术语）：

- 段落标签：`**Files:**`→`**文件：**`、`**Interfaces:**`→`**接口：**`、`- Modify:`→`- 修改：`、`- Create:`→`- 新建：`、`- Test:`→`- 测试：`、`- Consumes:`/`- Consumes：`→`- 消费：`、`- Produces:`/`- Produces：`→`- 产出：`、`- Possibly delete:`→`- 可能删除：`、`Run:`→`运行：`、`Expected:`→`预期：`、`**Goal:**`→`**目标：**`、`**Architecture:**`→`**架构：**`、`**Tech Stack:**`→`**技术栈：**`、`**Spec coverage:**`→`**Spec 覆盖：**`、`**Placeholder scan:**`→`**占位符扫描：**`、`**Type consistency:**`→`**类型一致：**`。
- 结构与步骤标题：`## Global Constraints`→`## 全局约束`、`## File Structure`→`## 文件结构`、`### Task N: `→`### 任务 N：`、`**Step N: `→`**步骤 N：`、`## Self-Review`→`## 自审`、`## Execution Notes`→`## 执行说明`。
- 正文段落（brainstorming / batch 两篇为英文原文，逐段全译；其余四篇只译其中的英文句与英文标签）。

## 2. 残留行清单及白名单理由

**AC-3 残留行清单（en 行）：7 个文件全部 0 行。** 上方各文件节未打印「AC-3 残留行清单」正是因为 offenders 为空。

**「附录 · 被 R5 跳过的行」同为 0 行** —— 即不存在「被缩进代码块 / 引用块裸日志规则豁免掉的英文正文」，因此没有藏在豁免规则后面的漏网段落。

仍以英文出现的部分全部属于白名单，逐条理由如下：

1. **fenced code block（``` / ~~~）内全部内容** —— 铁律 1。含注释与字符串，一个字符未动；fence 行数与基线逐项相等（22/30/18/14/0/8/10）。例如 B-vision 的 `vision.rs` 测试替身与实现、`git commit -m "..."` 提交模板、brainstorming/batch 里被替换的 persona 常量字符串（`environment or a secrets store, not a question. \` 等）原样保留。
2. **inline code（反引号）内容** —— 铁律 2，未改动；也未把任何原本不在反引号里的词新包成反引号（AC-4 多重集相等可证）。
3. **保留原文的技术符号与专名** —— 铁律 3：文件路径与目录名（`crates/rustcode-coding/src/persona.rs`、`crates/rustcode-tuix/src/event_loop/mod.rs`）、crate 名（`rustcode-coding`、`rustcode-cli`、`rustcode-daemon`、`rustcode-capabilities`、`rustcode-core`、`rustcode-kernel`）、命令与 CLI flag（`cargo test -p rustcode-coding`、`--no-run`、`-p rustcode-tuix`、`CARGO_INCREMENTAL=0`）、环境变量（`RUSTCODE_REQUEST_USER_INPUT=0`、`RUSTCODE_TUIX_LOG`）、配置键、`Msg::Xxx` 变体名、枚举变体（`EmptyEscIntercept::TriggerUndo`、`PermissionDecision::AllowOnce`、`PreprocessOutcome::{Skipped, Replaced, Failed}`）、函数名/类型名/工具名/hook 名（`coding_persona`、`run_vl_caption`、`should_skip`、`apply_outcome`、`intercept_empty_bare_esc`、`derive_tier_config`）、技能名（`superpowers:subagent-driven-development`、`superpowers:executing-plans`）、第三方专名（Rust、crossterm、tokio、async-trait、deepseek、GLM、OpenAI 系邮箱）、版本号/分支/日期/数字（`release/v5.0.3`、`release/v5.0.1`、`v5.0.3`、`:3497`、约 303 行）。
4. **已中文段落中的既有技术术语** —— 铁律 8，未润色、未改写：`bare Esc`、`init`、`redo`、`min-gap`、`re-export` 之外的既有写法（如 `parity`、`wiring`、`orphan`、`emit`、`cell-diff`、`row-level`、`dumb terminal`、`alt-screen` 等在同一段落中与中文混排的既有写法），仅在它们作为**英文标签/标题**出现时才替换（如 `**Files**:`→`**文件**：`、`### Task 18:`→`### 任务 18：`），其余保持原样。
5. **链接** —— 铁律 4 在本次无实际对象：7 个文件基线与工作区的 `](...)` 链接目标均为 0 条，改后仍为 0 条（AC-7b 恒等）。
6. **Emoji** —— 铁律 6：既有的 1 处 Emoji（B-vision）原样保留，未新增、未清理；其余 6 文件为 0。
7. **frontmatter** —— 7 个文件均无 YAML frontmatter（首行不是 `---`），AC-6 键名集合为空集且相等。

## 3. 四项实测结果

| 验收项 | 口径 | B-vision | double-esc | brainstorming | tool-ball-D | checklist | batch | ink-cell-diff | 结论 |
|---|---|---|---|---|---|---|---|---|---|
| **AC-2** | `en/total <= 0.05` | 0/110=0.0000 | 0/62=0.0000 | 0/78=0.0000 | 0/67=0.0000 | 0/93=0.0000 | 0/52=0.0000 | 0/93=0.0000 | **7/7 PASS** |
| **AC-4** | inline code + fenced code 多重集与基线相等 | 538→538 | 234→234 | 147→147 | 144→144 | 42→42 | 97→97 | 96→96 | **7/7 PASS** |
| **AC-7b** | `](...)` 链接目标多重集相等 | 0→0 | 0→0 | 0→0 | 0→0 | 0→0 | 0→0 | 0→0 | **7/7 PASS** |
| **AC-32** | Emoji 命中数不增加 | 1→1 | 0→0 | 0→0 | 0→0 | 0→0 | 0→0 | 0→0 | **7/7 PASS** |

附带项：**AC-6** frontmatter 键名集合 —— 7 文件均无 frontmatter，键名集合为空集且相等，PASS。

## 4. 复现命令与实测输出

```bash
python3 scripts/check-zh-docs.py check --base 3ee655e3 \
  --report .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-09.md \
  --files docs/superpowers/plans/2026-07-25-retire-core-provider-B-vision.md \
          docs/superpowers/plans/2026-07-06-double-esc-undo-cooldown.md \
          docs/superpowers/plans/2026-07-22-brainstorming-request-user-input.md \
          docs/superpowers/plans/2026-07-25-retire-core-tool-ball-D.md \
          docs/superpowers/2026-07-27-release-v5.0.3-test-checklist.md \
          docs/superpowers/plans/2026-07-22-batch-user-questions-persona-nudge.md \
          docs/superpowers/plans/2026-04-19-tuix-ink-cell-diff.md
```

实测输出（7 个文件一起跑，退出码 0）：

```text
PASS docs/superpowers/plans/2026-07-25-retire-core-provider-B-vision.md en=0/110=0.0000
PASS docs/superpowers/plans/2026-07-06-double-esc-undo-cooldown.md en=0/62=0.0000
PASS docs/superpowers/plans/2026-07-22-brainstorming-request-user-input.md en=0/78=0.0000
PASS docs/superpowers/plans/2026-07-25-retire-core-tool-ball-D.md en=0/67=0.0000
PASS docs/superpowers/2026-07-27-release-v5.0.3-test-checklist.md en=0/93=0.0000
PASS docs/superpowers/plans/2026-07-22-batch-user-questions-persona-nudge.md en=0/52=0.0000
PASS docs/superpowers/plans/2026-04-19-tuix-ink-cell-diff.md en=0/93=0.0000

check: 受检 7，PASS 7，FAIL 0
check: 报告已写入 .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-09.md
```

## 5. 过程记录与踩坑

- **首轮自检前的基线摸底**：7 文件首轮 `check` 结果为 6 FAIL / 1 PASS；唯一 PASS 的正是 `2026-07-27-release-v5.0.3-test-checklist.md`（`en=0/93`），据此直接判定为 no-op 并保持零改动。
- **踩坑 1（替换顺序冲突）**：brainstorming 文件里 `with:` 出现 2 次，其中一次位于 `Replace with:` 内部。若先用裸 `with:` 做全局替换，`Replace with:` 会被二次替换成 `替换替换为：`。已改为**先**整体替换 `Replace with:`、**再**用行锚定正则 `^with:$` 处理独立成行的那一处；batch 文件同样处理。
- **踩坑 2（计数断言）**：脚本对每处替换都断言命中次数，首轮报出 3 处计数不符（B-vision 的 `- Consumes：` 实为 2 处而非 3 处、`Run:` 分为独立成行 2 处 + 行内 11 处、tool-ball-D 的 `Expected:` 为 4 处而非 5 处）。均已按实际命中数校正后重跑，未出现"替换落空"或"误替换"。
- **踩坑 3（遗留英文标题）**：ink-cell-diff 的 `### Task 18:` / `### Task 21:` 不在首轮处理的三个文件里，被漏掉；`| Task | 规模 | 风险 |` 表头、`Low`/`Medium` 风险值、`## 不做的（out of scope）` 同样是英文标签。已在第三轮补译（任务 18/21、任务、低/中、范围外）。
- **踩坑 4（术语回补）**：B-vision 与 tool-ball-D 里的 `Task 4` / `Task1 Step3` / `对应 Task` 等引用未随标题一起转换，`re-export`、`repoint`、`Commit message` 三处英文词同样残留，已在第三轮回补为 `任务 4` / `任务 1 步骤 3` / `对应任务` / `重新导出` / `重新指向` / `提交信息`。
- **AC-2 的全角括号陷阱**（前批实测的坑）：仅由全角括号包裹的英文词（如「（re-export）」）不含 CJK 表意文字，会被判为 en 行。本次凡出现此类括注都补入了中文（如「（\`mod vision;\` + 重新导出）」「（应提示 No such file）」），故 7 文件 en 行均为 0。
- **范围**：本次只改 `files_owned` 的 6 个文件，未触碰 `docs/` 下任何其他文件，未改动任何源码或测试；`docs/superpowers/` 目录里其余文件的 diff 属于其它批次（D-01…D-08），与本次无关。
