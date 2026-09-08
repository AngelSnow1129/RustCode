# zh-check 报告

- 基线 (--base): `3ee655e3`
- 阈值 (--en-ratio): `0.05`
- 判定: AC-2 / AC-4 / AC-6 / AC-7b / AC-32，任一失败即 FAIL（fail-closed）

## 汇总

- 受检文件: 4
- PASS: 4
- FAIL: 0

## PASS `docs/superpowers/plans/2026-04-19-tuix-retained-mode-rewrite.md`

- en=0/169 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-12-github-style-diff.md`

- en=0/129 ratio=0.0000

## PASS `docs/superpowers/plans/2026-06-09-cache-friendly-compaction.md`

- en=0/135 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-31-local-scheduled-tasks-phase1.md`

- en=0/131 ratio=0.0000

---

# D-06 汉化小结（人工撰写）

任务：把 `docs/superpowers/plans/` 下四个实施计划的**英文正文**译为简体中文，纯中文替换、不做中英并列、不保留英文原文段落。基线 `3ee655e3`，受检文件即本次 `files_owned` 的四篇。

## 1. 每个文件改动量

| 文件 | 总行数（基线 = 改后）| 改动行（+/-，vs `3ee655e3`）| 汉化前 en/total | 汉化后 en/total |
|---|---|---|---|---|
| `docs/superpowers/plans/2026-04-19-tuix-retained-mode-rewrite.md` | 688 | +28 / -28 | 27/169 = 0.1598 | 0/169 = 0.0000 |
| `docs/superpowers/plans/2026-07-12-github-style-diff.md` | 568 | +119 / -119 | 107/129 = 0.8295 | 0/129 = 0.0000 |
| `docs/superpowers/plans/2026-06-09-cache-friendly-compaction.md` | 601 | +120 / -120 | 105/135 = 0.7778 | 0/135 = 0.0000 |
| `docs/superpowers/plans/2026-07-31-local-scheduled-tasks-phase1.md` | 463 | +41 / -41 | 27/131 = 0.2061 | 0/131 = 0.0000 |

改动量取自 `git diff --numstat 3ee655e3 -- <file>`。四个文件均为**纯行内替换**：`+N/-N` 严格相等、总行数不变、标题层级不变；各级标题数 / 复选框 `- [ ]` 数 / fence 行数与基线逐项相等 —— 依次为 (22, 36, 18)、(13, 23, 46)、(10, 27, 38)、(19, 22, 26)；文件名与计划日期未改动。

四篇的翻译密度差异很大，原因如下：`2026-04-19` 与 `2026-07-31` 原本就是中英混排（中文骨架 + 英文小标题/标签），只需补齐 `**Files:**` / `**Interfaces:**` / `Expected:` / 章节名等残留英文；`2026-07-12` 与 `2026-06-09` 则几乎全篇英文（Han=0 / Han=2），需要整篇翻译正文。

## 2. 残留行清单及白名单理由

**AC-2（AC-3）残留 en 行：四个文件均为 0 行。** 上方各文件节中未打印 "AC-3 残留行清单"，正是因为 offenders 为空；被 R5 规则（缩进代码块 / 含 fence 引用块内的裸日志行）跳过的行也均为 0 行，所以不存在"被豁免掉的英文正文"。

仍以英文出现的部分全部属于白名单，理由如下：

1. **fenced code block（``` / ~~~）内的全部内容** —— 铁律 1：含注释与字符串，一个字符都未改动。四文件 fence 行数与基线一致（18/46/38/26 条 fence 行）。
2. **inline code（反引号）内容** —— 铁律 2。AC-2 的 R4 会先剥离 inline code 再判定；AC-4 则强制其与基线多重集相等。
3. **文件路径、目录名、crate 名（`rustcode-capabilities` / `rustcode-tuix` / `rustcode-core` / `rustcode-config` / `rustcode-cli`）、命令与 CLI flag（`cargo test -p …`、`--nocapture`、`git add <path>`）、环境变量（`RUSTCODE_HOME`）、配置键/TOML 表名（`[workspace.dependencies]`、`[dependencies]`、`tools` feature）、HTTP/API 名、`Msg::`/`UiLine::` 变体名、枚举变体（`DiffKind::Add`、`SessionOrigin::Scheduled`）、函数名/类型名/工具名/hook 名（`build_compact_diff`、`collapse_committed`、`parse_unified_diff`、`next_run`、`list_visible`）、第三方专有名词（Rust、crossterm、unicode-width、tokio、similar、syntect、codex、deepseek、litellm、macOS、GitHub、Cron、SGR、OS、DST、UTC）、版本号/日期/数字** —— 铁律 3，逐字保留。
4. **本就为中文的段落** —— 铁律 8，未润色、未改写。例如 `2026-04-19` 的核心设计决策、工期 & 风险表；`2026-07-31` 的目标/架构/技术栈/全局约束与 Task 1–4 的全部中文步骤说明。仅对夹在中文行里的英文小标题（`**2. Placeholder scan:**`、`**3. Type consistency:**`）做了标签本地化，以与已翻译的 `**1. 规格覆盖度：**` 保持一致，正文未动。
5. **链接** —— 铁律 4 在本次无实际对象：四文件基线中 `](...)` 链接目标均为 0 条，改后仍为 0 条。
6. **Emoji** —— 铁律 6：既有 Emoji 原样保留，未新增也未清理（详见第 3 节 AC-32 行）。

## 3. 四项实测结果

| 验收项 | 口径 | 04-19 retained-mode | 07-12 github-diff | 06-09 compaction | 07-31 scheduled-tasks | 结论 |
|---|---|---|---|---|---|---|
| **AC-2** | `en/total <= 0.05` | 0/169 = 0.0000 | 0/129 = 0.0000 | 0/135 = 0.0000 | 0/131 = 0.0000 | **PASS** |
| **AC-4** | inline code + fenced code 多重集与基线相等 | 534 → 534 | 593 → 593 | 520 → 520 | 451 → 451 | **PASS** |
| **AC-7b** | `](...)` 链接目标多重集相等 | 0 → 0 | 0 → 0 | 0 → 0 | 0 → 0 | **PASS** |
| **AC-32** | Emoji 命中数不增加 | 1 → 1 | 0 → 0 | 0 → 0 | 0 → 0 | **PASS** |

附带项：**AC-6** frontmatter 键名集合 —— 四文件均无 YAML frontmatter（与铁律 7 一致），键名集合为空集且相等，PASS。

`2026-04-19` 那 1 个 Emoji 位于 rust 代码围栏内（`format!("  ❯ {}", scrub_controls(&self.input_buf))` 中的 `❯`，U+276F），未改动。

## 4. 复现命令与实测输出

```bash
python3 scripts/check-zh-docs.py check --base 3ee655e3 \
  --report .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-06.md \
  --files docs/superpowers/plans/2026-04-19-tuix-retained-mode-rewrite.md \
          docs/superpowers/plans/2026-07-12-github-style-diff.md \
          docs/superpowers/plans/2026-06-09-cache-friendly-compaction.md \
          docs/superpowers/plans/2026-07-31-local-scheduled-tasks-phase1.md
```

实测输出（四文件一起跑，退出码 0）：

```text
PASS docs/superpowers/plans/2026-04-19-tuix-retained-mode-rewrite.md en=0/169=0.0000
PASS docs/superpowers/plans/2026-07-12-github-style-diff.md en=0/129=0.0000
PASS docs/superpowers/plans/2026-06-09-cache-friendly-compaction.md en=0/135=0.0000
PASS docs/superpowers/plans/2026-07-31-local-scheduled-tasks-phase1.md en=0/131=0.0000

check: 受检 4，PASS 4，FAIL 0
check: 报告已写入 .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-06.md
```

## 5. 过程记录

- 首轮自检中 `2026-07-31-local-scheduled-tasks-phase1.md` 残留 3 行 en 行（原第 34、375、380 行）：这三行的形式为 `\`code\`（英文串）`，剥离 inline code 后只剩全角括号包裹的英文（`（serde tag = "kind"）`、`（main.rs L2213）`、`（manager.rs L1412）+ \`SessionOrigin\`（Task 2）`），而全角括号不属于 `CJK_RE` 的判定范围。已补入中文（`serde 标签`、`main.rs 第 2213 行`、`manager.rs 第 1412 行`），复检 en=0。
- 同一文件另有若干处半角括号包裹的英文（如 `\`…\`(Task 1)。`、`(None → Fresh session)`），虽然所在行已有中文而不算 en 行，但为与全篇统一，已一并改为全角括号并本地化其中可译部分。
- 中途一次编辑在 `2026-07-12-github-style-diff.md` 的 Step 5 前多插了一个空行，导致行数 568 → 569。这不是硬性违规，但与"纯行内替换"的交付口径不符，已删除该空行，恢复为 568 行、`+119/-119`。
- 全程未触碰任何 fenced code block 与 inline code 内容，未改动源码、测试与 `docs/` 下其他文件。
