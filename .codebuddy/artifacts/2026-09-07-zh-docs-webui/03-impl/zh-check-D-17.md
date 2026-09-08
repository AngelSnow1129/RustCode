# zh-check 报告

- 基线 (--base): `3ee655e3`
- 阈值 (--en-ratio): `0.05`
- 判定: AC-2 / AC-4 / AC-6 / AC-7b / AC-32，任一失败即 FAIL（fail-closed）

## 汇总

- 受检文件: 12
- PASS: 12
- FAIL: 0

## PASS `docs/plans/2026-07-25-subtasks-footer-panel.md`

- en=0/37 ratio=0.0000

## PASS `docs/plans/2026-08-15-webui-blocking-interaction-dock.md`

- en=0/38 ratio=0.0000

## PASS `docs/plans/2026-07-23-windows-qr-rendering.md`

- en=0/33 ratio=0.0000

## PASS `docs/plans/2026-08-07-request-user-input-review-design.md`

- en=0/31 ratio=0.0000

## PASS `docs/plans/2026-07-26-atomgit-production-tools.md`

- en=0/30 ratio=0.0000

## PASS `docs/plans/2026-07-26-busy-continue-fork-gc.md`

- en=0/26 ratio=0.0000

## PASS `docs/plans/2026-08-06-internal-continuation-compaction-design.md`

- en=0/8 ratio=0.0000

## PASS `docs/plans/2026-08-07-tasks-long-line-rendering.md`

- en=0/26 ratio=0.0000

## PASS `docs/plans/2026-08-23-headless-eval-controls.md`

- en=1/28 ratio=0.0357
- AC-3 残留行清单 (en 行，共 1 行):
  - `docs/plans/2026-08-23-headless-eval-controls.md:28`: `  cache hit rate；`

## PASS `docs/plans/2026-08-02-session-recovery-implementation-plan.md`

- en=0/28 ratio=0.0000

## PASS `docs/plans/2026-08-11-todo-agent-body-projection-design.md`

- en=0/24 ratio=0.0000

## PASS `docs/plans/2026-08-11-provider-form-horizontal-editing-design.md`

- en=0/21 ratio=0.0000

---

# D-17 手写小结（批次 6 · docs/plans 小文件）

- 任务 ID：`D-17`；基线 `3ee655e3`；自检命令见铁律 §4（`--files` 为本批 12 个文件相对路径）。
- 本批 12 个文件**均无 YAML frontmatter**，因此 AC-6 无 `description` 可汉化，无需逐条列示。
- 本批**无 fenced code block**（代码块密度为「中」指 inline code 较多，fence 为零），翻译面全部是散文 + inline code。

## 一、每文件改动量

| 文件 | 改动行数 | en/total（改前 → 改后） | 处理结论 |
|---|---|---|---|
| `docs/plans/2026-07-25-subtasks-footer-panel.md` | +36/-36 | 35/37=0.9459 → 0/37=0.0000 | 全量汉化 |
| `docs/plans/2026-08-15-webui-blocking-interaction-dock.md` | +37/-37 | 37/38=0.9737 → 0/38=0.0000 | 全量汉化 |
| `docs/plans/2026-07-23-windows-qr-rendering.md` | +32/-32 | 30/33=0.9091 → 0/33=0.0000 | 全量汉化 |
| `docs/plans/2026-08-07-request-user-input-review-design.md` | +30/-30 | 30/31=0.9677 → 0/31=0.0000 | 全量汉化 |
| `docs/plans/2026-07-26-atomgit-production-tools.md` | +29/-29 | 28/30=0.9333 → 0/30=0.0000 | 全量汉化 |
| `docs/plans/2026-07-26-busy-continue-fork-gc.md` | +25/-25 | 23/26=0.8846 → 0/26=0.0000 | 全量汉化（含补回文件末尾既有的 1 个空行） |
| `docs/plans/2026-08-06-internal-continuation-compaction-design.md` | +8/-8 | 8/8=1.0000 → 0/8=0.0000 | 全量汉化；该文件 `total=8`，阈值下**只允许 en=0**，已逐行清零 |
| `docs/plans/2026-08-07-tasks-long-line-rendering.md` | +25/-25 | 24/26=0.9231 → 0/26=0.0000 | 全量汉化 |
| `docs/plans/2026-08-02-session-recovery-implementation-plan.md` | +36/-36 | 26/28=0.9286 → 0/28=0.0000 | 全量汉化（保留 4 处硬折行） |
| `docs/plans/2026-08-23-headless-eval-controls.md` | 0 | 1/28=0.0357 → 1/28=0.0357 | **no-op**（已中文且已 ≤ 0.05） |
| `docs/plans/2026-08-11-todo-agent-body-projection-design.md` | 0 | 0/24=0.0000 → 0/24=0.0000 | **no-op**（已中文，无 offender） |
| `docs/plans/2026-08-11-provider-form-horizontal-editing-design.md` | 0 | 0/21=0.0000 → 0/21=0.0000 | **no-op**（已中文，无 offender） |

合计：`+258/-258`，即纯行内替换，未增删空行、未改变段落结构（对齐铁律 §5 第 4、7 条）。

## 二、残留行清单与白名单理由

只有 1 行残留，全部位于本批标注「已中文」的文件中：

- `docs/plans/2026-08-23-headless-eval-controls.md:28` → `  cache hit rate；`
  - 白名单理由：这是第 27 行「最终事件聚合 TTFT、duration、round/tool 数、prompt/completion/cached token 与」的**硬折行续行**，整行只有英文术语 + 全角分号。按铁律 §3，该文件 `en/total = 0.0357 ≤ 0.05`，判为 **no-op**，**不得改写已中文段落**，故保留原样。
  - 若强行清理，只能把两行重排成一行；该行不存在跨行反引号对（第 27 行无反引号），重排本身不会触发 AC-4，但会改写已中文段落并违反 §3，故不执行。

其余 11 个文件改后 `en = 0`，**无残留行**。

## 三、四项实测结果（最后一次自检，即本报告脚本自动段）

| 判定项 | 结果 | 说明 |
|---|---|---|
| **AC-2** `en/total <= 0.05` | **PASS**（12/12） | 9 个汉化文件全部 `0.0000`；3 个 no-op 文件分别为 `0.0357 / 0.0000 / 0.0000` |
| **AC-4** inline code + fenced code 多重集相等 | **PASS**（12/12） | 脚本未输出任何 AC-4 FAIL；全部 inline code 原样保留，未新增/删除反引号，未把普通词包成反引号（规避 §5 第 1、3 条） |
| **AC-7b** `](...)` 链接目标多重集相等 | **PASS**（12/12） | 本批 12 个文件正文无 markdown 链接，链接目标集合为空且前后一致 |
| **AC-32** Emoji 数不增加 | **PASS**（12/12） | 未引入任何 Emoji，也未清理既有 Emoji（本批文件原本即无 Emoji） |

汇总行：`check: 受检 12，PASS 12，FAIL 0`。

## 四、额外核对（超出脚本范围，人工确认）

1. **硬折行处理**：`2026-08-02-session-recovery-implementation-plan.md` 有 4 处硬折行（原 5-6、23-24、32-34、35-36 行）。已**保留原折行位置**，只在每个物理行内做中文替换，未把跨行内容并成一行；`rustcode-capabilities::session`、`ControllerWarning` 等反引号对仍完整落在同一物理行（规避 §5 第 6 条）。
2. **全角标点陷阱**：所有中文行都补入了真实汉字，没有出现「只加全角括号/全角句号包裹英文词」的写法（规避 §5 第 2 条）。例如 `**目标：**` 后接的是中文散文，而不是裸英文。
3. **保留原文**：crate 名（`rustcode-tuix`、`rustcode-capabilities`、`rustcode-coding`）、路径、命令（`cargo test -p ...`、`git diff --check`）、flag（`--features session`、`--lib`）、配置与协议符号（`AgentEvent::ToolOutputChunk`、`SessionMeta::fork_info`、`META_VERSION`、`POST /projects/:hash/sessions/:id/repair`、`compact_threshold`）、第三方名（Rust、Preact、TypeScript、CSS、Node test runner、crossterm、qrcode 0.14、OpenAI-compatible、DeepSeek、Claude、Codex）与版本号（`v5.0.5`、`80×24`）全部原样保留。
4. **入站锚点引用**：已用 `Grep` 在全仓检索这 12 个文件名（含 `docs/**`），**0 命中**，即没有任何文档以 `文件#锚点` 形式引用本批文件的标题，标题汉化不会造成断链。
5. **边界**：只改 `files_owned` 的 9 个文件；3 个 no-op 文件零改动（`git status` 中未出现）。同目录其它 `docs/plans/*.md` 的改动来自并行批次，与本批无关。未执行任何 `cargo`/`npm` 构建或测试，未改源码。
