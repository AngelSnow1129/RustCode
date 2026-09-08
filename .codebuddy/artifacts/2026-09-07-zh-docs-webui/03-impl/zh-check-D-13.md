# zh-check 报告

- 基线 (--base): `3ee655e3`
- 阈值 (--en-ratio): `0.05`
- 判定: AC-2 / AC-4 / AC-6 / AC-7b / AC-32，任一失败即 FAIL（fail-closed）

## 汇总

- 受检文件: 2
- PASS: 2
- FAIL: 0

## PASS `docs/plans/2026-08-20-code-review-deep-mode-fanout-plan.md`

- en=0/171 ratio=0.0000

## PASS `docs/plans/2026-08-20-code-review-deep-verify-phase2-plan.md`

- en=0/112 ratio=0.0000

---

# D-13 汉化小结（手写）

- 任务 ID：D-13（批次 5）
- 基线：`3ee655e3`；自检命令：`python3 scripts/check-zh-docs.py check --base 3ee655e3 --report .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-13.md --files docs/plans/2026-08-20-code-review-deep-mode-fanout-plan.md docs/plans/2026-08-20-code-review-deep-verify-phase2-plan.md`
- 本小结写于**最后一次**自检之后（该次自检 2 受检 / 2 PASS / 0 FAIL）。

## 一、改动量（两文件均为全量汉化，非 no-op）

| 文件 | 改动行 | 判定 | 说明 |
|---|---|---|---|
| `docs/plans/2026-08-20-code-review-deep-mode-fanout-plan.md` | 159 | 全量汉化 | 原文 Han=0 的纯英文大件。已译：标题与 agentic worker 提示、`Goal`/`Architecture`/`Tech Stack`/`Spec`、`Global Constraints`、任务 1–7 的全部 `Files`/`Interfaces`/`Step 1–5` 说明性文字、以及 `Self-Review` 全部条目 |
| `docs/plans/2026-08-20-code-review-deep-verify-phase2-plan.md` | 102 | 全量汉化 | 同上，覆盖任务 1–4 与 `Self-Review`；正文后半段（任务 2 的 `(d)` 实现说明、任务 3 的解析说明）同样完整翻译，不存在只处理前半部分的情况 |

- 行数守恒：`git diff --numstat` 两文件分别为 `159/159`、`102/102`，即 `+N/-N` 相等的纯行内替换。
- 代码块零改动：另用 fence 状态机比对，两文件 fenced 区域（file1 687 行、file2 418 行，含 fence 行本身）与基线**逐字节相同**。
- frontmatter：两文件首行均非 `---`，无 YAML frontmatter；AC-6 键名集合在基线与工作区皆为空集，也不存在 `description` 值需要汉化，故 AC-6 无逐条列示项。

## 二、残留行清单与白名单理由

自检输出 **en=0/171** 与 **en=0/112**，两文件 AC-2 offender 行**均为 0**，报告中亦无 R5 跳过行；因此**没有任何需要豁免的残留英文行**。

文中仍在出现的英文，全部属于铁律 §2.1–§2.3 要求原样保留、且 AC-2 不计入分母的类别：

1. fenced code block 内的一切内容（Rust 源码、注释、字符串、`cargo`/`git` 命令、commit message）—— 一个字符未动。
2. inline code 内的符号：路径 `crates/rustcode-review/src/fanout.rs`、`crates/rustcode-tuix/src/event_loop/commands.rs`；函数/类型 `merge_findings`、`run_deep_review`、`finalize_deep_review`、`DimensionOutcome`、`MergedFinding`、`render_verify_task`、`run_verify`、`ReportFindingTool`、`ReviewAgentConfig::with_persona_append`、`tokio::task::JoinSet` 等；命令 `cargo test -p rustcode-review ...`。
3. 第三方与项目专有名词：Rust、tokio、`superpowers:subagent-driven-development`、`superpowers:executing-plans`。
4. 外部规范章节名（不可译的引用）：`§ "Phase 2 — adversarial verify pass"`。
5. 本批要求保留原文的 `code-review` 领域术语：`/review deep`、`deep+verify`、`fan-out`、`keep-mask`、`fail-open`、`persona_append`、`verify`、`single`/`deep` 深度名。

## 三、四项实测结果

| 项 | 文件 1（deep-mode-fanout-plan） | 文件 2（deep-verify-phase2-plan） | 结论 |
|---|---|---|---|
| AC-2 `en/total <= 0.05` | 0/171 = 0.0000 | 0/112 = 0.0000 | PASS |
| AC-4 inline code + fenced code 多重集 | 与基线相等（各 952 项） | 与基线相等 | PASS |
| AC-7b `](...)` 链接目标多重集 | 与基线相等（全文件 0 处 markdown 链接） | 与基线相等（0 处） | PASS |
| AC-32 Emoji 数不增加 | 0 → 0 | 0 → 0 | PASS |
| AC-6 frontmatter 键名集合 | 空集 = 空集 | 空集 = 空集 | PASS |

## 四、过程记录（踩坑与修正）

1. **首轮 AC-4 FAIL（新增 1 个 code span）**：原文 `Run: ...args_parse_depth_field` 与 `then the full crate: ...` 是相邻两行，我误合成一行，导致 `cargo test -p rustcode-review` 这个 span 比基线多出现一次。已拆回两行并删掉残留的英文原行，AC-4 恢复 PASS。
2. **首轮 AC-2 残留 3 行**（`fanout-plan.md:34`、`fanout-plan.md:838`、`phase2-plan.md:40`）：都是「整行 inline code + 少量连接词」的签名条目，正是铁律 §5.2 记的全角标点不算 CJK 的坑。已补入**真实汉字**（分别为「4 项」、「然后跑整个 crate」、「，其中」），未只加全角标点。
3. **全程未使用批量替换**：改用「按行号 1:1 替换 + fence 状态机硬校验」的脚本执行，任何落在 fenced 代码块内的目标行都会被直接拒绝，从机制上排除铁律 §5.3 的批量替换事故。
4. 未新增 Emoji，未新增/删除空行，未改动任何链接与锚点（本批两文件无 markdown 链接）。
