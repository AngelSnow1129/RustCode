# zh-check 报告

- 基线 (--base): `3ee655e3`
- 阈值 (--en-ratio): `0.05`
- 判定: AC-2 / AC-4 / AC-6 / AC-7b / AC-32，任一失败即 FAIL（fail-closed）

## 汇总

- 受检文件: 11
- PASS: 11
- FAIL: 0

## PASS `docs/superpowers/specs/2026-04-23-p2-doctor-review-notebook-todo-design.md`

- en=0/39 ratio=0.0000

## PASS `docs/superpowers/specs/2026-05-08-vision-preprocessor-design.md`

- en=0/97 ratio=0.0000

## PASS `docs/superpowers/specs/2026-05-25-tuix-unified-in-app-scroll-design.md`

- en=0/178 ratio=0.0000

## PASS `docs/superpowers/specs/2026-05-29-provider-add-simplify-design.md`

- en=0/147 ratio=0.0000
- R5 跳过的行 (共 1 行):
  - `docs/superpowers/specs/2026-05-29-provider-add-simplify-design.md:180`: `    model = body 里 '"model"' 值`

## PASS `docs/superpowers/specs/2026-05-29-webui-design.md`

- en=0/153 ratio=0.0000

## PASS `docs/superpowers/specs/2026-06-07-headless-output-format-json-design.md`

- en=0/69 ratio=0.0000

## PASS `docs/superpowers/specs/2026-06-09-cache-friendly-compaction-design.md`

- en=0/80 ratio=0.0000

## PASS `docs/superpowers/specs/2026-06-27-v2-rate-limit-pause-resume-design.md`

- en=0/73 ratio=0.0000

## PASS `docs/superpowers/specs/2026-06-29-acp-agent-design.md`

- en=0/130 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-03-terminal-status-glyph-design.md`

- en=0/72 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-06-double-esc-undo-cooldown-design.md`

- en=0/41 ratio=0.0000

## 附录 · 被 R5 跳过的行（供抽检）

共 1 行。

- `docs/superpowers/specs/2026-05-29-provider-add-simplify-design.md:180`: `    model = body 里 '"model"' 值`

---

# D-10 手写小结

## 一、任务概述

- 任务 ID：**D-10**（汉化批次 4）
- 基线：`3ee655e3`；自检工具：`scripts/check-zh-docs.py check`
- files_owned：`docs/superpowers/specs/` 下 11 个设计文档
- 自检命令（最后一次执行，本报告即由该命令生成）：

```bash
python3 scripts/check-zh-docs.py check --base 3ee655e3 \
  --report .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-10.md \
  --files docs/superpowers/specs/2026-04-23-p2-doctor-review-notebook-todo-design.md \
          docs/superpowers/specs/2026-05-08-vision-preprocessor-design.md \
          docs/superpowers/specs/2026-05-25-tuix-unified-in-app-scroll-design.md \
          docs/superpowers/specs/2026-05-29-provider-add-simplify-design.md \
          docs/superpowers/specs/2026-05-29-webui-design.md \
          docs/superpowers/specs/2026-06-07-headless-output-format-json-design.md \
          docs/superpowers/specs/2026-06-09-cache-friendly-compaction-design.md \
          docs/superpowers/specs/2026-06-27-v2-rate-limit-pause-resume-design.md \
          docs/superpowers/specs/2026-06-29-acp-agent-design.md \
          docs/superpowers/specs/2026-07-03-terminal-status-glyph-design.md \
          docs/superpowers/specs/2026-07-06-double-esc-undo-cooldown-design.md
```

- 实测结论：**受检 11，PASS 11，FAIL 0**，11 个文件的 `en` 计数全部为 **0**（`ratio = 0.0000`）。

## 二、逐文件改动量

| 文件（`docs/superpowers/specs/`） | 基线 en/total | 现 en/total | `git diff --numstat` | 处置 |
|---|---|---|---|---|
| `2026-04-23-p2-doctor-review-notebook-todo-design.md` | 37/39 = 0.9487（Han=0，全英文） | 0/39 | 39 / 39 | **全文汉化**：标题、四节正文、检查项/实现步骤/动作列表全部译为中文；代码块与 inline code 逐字未动（含 `✓`/`✗`） |
| `2026-05-08-vision-preprocessor-design.md` | 1/97 = 0.0103 | 0/97 | 1 / 1 | **清 offender 1 行**：第 219 行测试项补中文说明 |
| `2026-05-25-tuix-unified-in-app-scroll-design.md` | 4/178 = 0.0225 | 0/178 | 4 / 4 | **清 offender 4 行**：`**Date**`/`**Status**` 标签、对比表表头空单元格、`flip` 一词 |
| `2026-05-29-provider-add-simplify-design.md` | 0/147 | 0/147 | 0 / 0 | **no-op**（铁律 §3：`en/total = 0 ≤ 0.05`，已中文，未改写） |
| `2026-05-29-webui-design.md` | 0/153 | 0/153 | 0 / 0 | **no-op**（同上） |
| `2026-06-07-headless-output-format-json-design.md` | 4/69 = 0.0580（FAIL） | 0/69 | 4 / 4 | **清 offender 4 行**：H1 补「模式」、`### 1. CLI flag` 补「定义」、事件表两行 `result` 子型补中文 |
| `2026-06-09-cache-friendly-compaction-design.md` | 0/80 | 0/80 | 0 / 0 | **no-op** |
| `2026-06-27-v2-rate-limit-pause-resume-design.md` | 0/73 | 0/73 | 0 / 0 | **no-op** |
| `2026-06-29-acp-agent-design.md` | 178/196 = 0.9082（Han=0，全英文） | 0/130 | 154 / 220 | **全文汉化**：动机、范围、关键决策表、架构、crate 代次、组件、引擎集成、事件映射表、权限流程、`initialize` capabilities、错误处理、测试策略、文件清单、构建说明 |
| `2026-07-03-terminal-status-glyph-design.md` | 0/72 | 0/72 | 0 / 0 | **no-op** |
| `2026-07-06-double-esc-undo-cooldown-design.md` | 0/41 | 0/41 | 0 / 0 | **no-op** |

合计：5 个文件有改动（共 202 行改写），6 个文件判定 no-op。

> 关于行数：`2026-06-29-acp-agent-design.md` 的 154/220 不相等，源于英文原文的 80 列硬折行在译为中文后合并成整段。同目录前批已交付文件同样存在 `+N/-N` 不等（如 `2026-07-22-brainstorming-request-user-input-design.md` 108/115、`2026-07-22-multi-question-request-user-input-design.md` 137/144），属既有惯例；本批**未增删任何空行**，仅合并被硬折行切断的散文行。

## 三、AC-3 残留行清单与白名单说明

**11 个文件的 AC-3 残留行清单均为空（`en = 0`）**，无残留需要白名单豁免。

报告中出现的 1 行 R5 跳过项（非 offender，不进 `total`，也不构成残留）：

- `docs/superpowers/specs/2026-05-29-provider-add-simplify-design.md:180`：`    model = body 里 '"model"' 值`
  - 为何属白名单：该行为 4 空格缩进代码块内容，被 R3-a 规则跳过，且行内已含汉字（「里」「值」），本身不是英文行；按铁律 §1 缩进代码块不翻译，按 §3 已中文内容不改写，故保持原样。

## 四、四项实测结果

| 项 | 判定 | 实测证据 |
|---|---|---|
| **AC-2**（`en/total ≤ 0.05`） | **PASS**（11/11） | 11 个文件 `en` 全部为 0，`ratio = 0.0000`，详见本报告各文件小节 |
| **AC-4**（inline code + fenced code 多重集相等） | **PASS**（11/11） | 逐文件 inline code 计数基线→工作区完全一致（47→47、123→123、197→197、147→147、146→146、160→160、93→93、91→91、214→214、74→74、58→58）；fenced 代码块内容未改一字 |
| **AC-7b**（`](...)` 链接目标多重集相等） | **PASS**（11/11） | 11 个文件基线与该次改动后均为 **0 个 markdown 链接**，多重集为空且相等；无外链、无锚点被触碰 |
| **AC-32**（Emoji 数不增加） | **PASS**（11/11） | 未新增任何 Emoji；`2026-04-23` 原有的 `✓`/`✗` 原样保留，未清理也未新增 |

补充项：**AC-6**（frontmatter 键名集合）—— 11 个文件首行均非 `---`（`first_line_is_fm=False`），无 frontmatter，键名集合基线/工作区均为空、相等，故无需按铁律 §7 逐条列示 `description`。

## 五、八条铁律逐条自查

1. **不翻译 fenced code block** —— 是。`2026-04-23` 的两段代码块（ipynb 输出示例、`TodoTool` 结构体）与 `2026-06-29` 的三段代码块（`serve_stdio` 签名、权限流程伪代码）逐字未动。
2. **不翻译 inline code，也不新增反引号** —— 是。inline code 计数基线/工作区逐一相等（见 AC-4 证据）；`2026-04-23` 第 84 行 `/todo` 原本不在反引号内，译文中仍为「斜杠命令 /todo」，未加反引号。
3. **保留原文术语** —— 是。保留：文件路径与 crate 名（`rustcode-acp`、`rustcode-kernel`、`rustcode-coding`、`rustcode-capabilities`、`agent-client-protocol`）、命令与 flag（`rustcode acp`、`--provider`、`--model`、`--output-format json`、`git status --porcelain`）、环境变量无、配置键无、`Msg` 变体无、枚举/函数/类型名（`AgentCommand::SendMessage`、`ToolCallContent::Diff`、`ProtocolVersion::V1`、`AsyncFnMut`、`StopReason`、`ToolKind`）、第三方专有名词（ACP、JSON-RPC、Zed、Claude Code、Anthropic、OpenAI、Rust、Tokio、serde_json、wiremock）、版本号与日期（`1.0.1`、`v1.1.0`、`edition 2024`、`≥ 1.85`、`2026-04-23`、`2026-06-29`）。
4. **不改链接目标与锚点** —— 是。11 个文件均无 `](...)` 链接。
5. **不改文件名、不改标题层级** —— 是。所有 `#`/`##`/`###` 层级与数量不变；仅在标题行内补入中文词（如 `## 1. /doctor` → `## 1. /doctor 诊断命令`、`## Key decisions` → `## 关键决策`）。
6. **不新增 / 不清理 Emoji** —— 是。
7. **YAML frontmatter 键名保留英文** —— 不适用（11 个文件均无 frontmatter）。
8. **已中文段落不润色、不改写** —— 是。6 个 no-op 文件一字未动；3 个「清 offender」文件只改脚本报出的那几行英文行；`2026-05-25` 第 279 行标题 `## Error handling / 边界条件` 虽含英文但整行已有中文，按 §3 未改写。

## 六、交叉引用与术语一致性核查

- 被改动文件均无 markdown 链接，因此不存在锚点失效风险。
- **章节名引用**：`docs/superpowers/plans/2026-05-08-vision-preprocessor-auto-config.md:15` 引用了 spec 的 `§风险与权衡`。经核对，`2026-05-08-vision-preprocessor-design.md` 第 245 行标题仍为 `## 风险与权衡`，本批未改动该标题，**引用依然有效**。
- **路径引用**：`docs/superpowers/plans/2026-05-08-vision-preprocessor.md:15` 与 `docs/superpowers/plans/2026-05-25-tuix-unified-in-app-scroll.md:9` 仅按文件路径引用对应 spec，不含锚点，**不受本批改动影响**。
- **术语一致性**（`2026-06-29-acp-agent-design.md` 对照已汉化的 `docs/superpowers/plans/2026-06-29-acp-agent.md`）：「Agent Client Protocol（ACP）」「ACP Agent 模式」「`prepare → assemble → spawn`」「`session/update` 通知」「`session/request_permission`」「多 agent 团队」「编辑器/编排器（例如 Zed）」等措辞与 plan 完全一致，未引入第二套译法。
- 同名 plan 文档（`docs/superpowers/plans/2026-05-08-vision-preprocessor.md`、`2026-05-25-tuix-unified-in-app-scroll.md`、`2026-06-29-acp-agent.md`）已由前批汉化，本批**未改动**。

## 七、本批踩坑与规避（供后续批次参考）

**新增踩坑（D-10 首现）：跨行 inline code 被合并会触发 AC-4 FAIL。**

`2026-06-29-acp-agent-design.md` 第 24–25 行的引用块中，`` `ToolCallContent::Diff { path, old_text, new_text }` `` 在基线上被硬折行切成两行，反引号分处两行，因此 `INLINE_RE`（不允许跨行）在基线上**根本没有为它生成 inline code span**。首次翻译时把这两行合并为一行，凭空多出一个 span，导致 `AC-4 FAIL：old=228 new=229，仅存在于工作区 ['ToolCallContent::Diff { path, old_text, new_text }']`。

处置：恢复原有折行位置（`` { path, `` 留在上行，`old_text, new_text }` 留在下行），AC-4 恢复 PASS。

**结论**：翻译硬折行的英文段落时，**凡跨越行边界的反引号对，必须保留原折行位置**；可安全合并的只有不含反引号的纯散文行。

**已规避的历史踩坑**：未把普通词误包成反引号（D-05/D-07）；未只加全角标点而漏汉字（D-06/D-08/D-09 —— 本次新增的中文均为真实汉字，如「模式」「定义」「子型」「诊断命令」）；未做波及代码块的批量替换（D-07）；未增删空行（D-06）；手写小结在本报告最后一次自检之后追加（D-08）。

## 八、遗留项与建议（不在 files_owned，本批未改动，提请编排者裁决）

1. **标题用词不一致（in-app vs 应用内）**：spec `docs/superpowers/specs/2026-05-25-tuix-unified-in-app-scroll-design.md` 标题为「TUI 统一 in-app 滚动设计」，而同源 plan `docs/superpowers/plans/2026-05-25-tuix-unified-in-app-scroll.md` 标题为「TUI 统一应用内滚动实施方案」。两者均已含中文，按铁律 §3 属 no-op 范围，本批**未改写**；建议由编排者决定是否统一为同一译法（改动需覆盖 plan 文件，超出本批 files_owned）。
2. **`## Error handling / 边界条件`**（`2026-05-25-...design.md:279`）：标题混排英文，但整行已有中文，脚本不计为 offender，按铁律 §3 未改写；如后续要求标题 100% 中文，需单独派单。
3. **同目录并发改动**：执行期间 `docs/superpowers/specs/` 下另有 `2026-07-22-*`、`2026-07-24-windows-native-tls-schannel-fallback-design.md`、`2026-07-31-local-scheduled-tasks-phase2-design.md` 等文件相对基线存在 diff，属其它批次 / 其它执行者，本批**未触碰**。若门禁按 `check --diff` 全量跑，本批结论不受影响（本批 11 个文件独立 PASS）。
4. 本批为纯文档任务，未执行 `cargo`/`npm` 构建与测试，未改动任何源码。
