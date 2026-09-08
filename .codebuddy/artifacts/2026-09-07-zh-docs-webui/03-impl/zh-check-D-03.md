# zh-check 报告

- 基线 (--base): `3ee655e3`
- 阈值 (--en-ratio): `0.05`
- 判定: AC-2 / AC-4 / AC-6 / AC-7b / AC-32，任一失败即 FAIL（fail-closed）

## 汇总

- 受检文件: 2
- PASS: 2
- FAIL: 0

## PASS `docs/superpowers/plans/2026-05-08-vision-preprocessor.md`

- en=0/159 ratio=0.0000

## PASS `docs/superpowers/plans/2026-05-08-vision-preprocessor-auto-config.md`

- en=0/109 ratio=0.0000

---

## D-03 汉化小结（doc-writer 手写）

### 1. 改动范围与改动量

本轮只改 `files_owned` 内的两个文件，未触碰任何源码、其他文档或脚本。改动方式全部是「整行替换散文，代码与符号原样保留」，两文件均无 YAML frontmatter，文件名与标题层级未变。

| 指标 | `2026-05-08-vision-preprocessor.md` | `2026-05-08-vision-preprocessor-auto-config.md` |
|---|---|---|
| 规模 | 1030 行 / 40.27 KB → 40.70 KB | 809 行 / 32.62 KB → 32.88 KB |
| 代码块 | 19 个（38 行围栏 + 697 行正文） | 25 个（50 行围栏 + 540 行正文） |
| inline code | 236 处（142 个去重值） | 167 处（74 个去重值） |
| `](...)` 链接 | 0 | 0 |
| Emoji | 0 | 4（均在代码块内：`✔` × 3、`⚠` × 1） |
| **实际改动行数** | **149** | **101** |
| 英文残留率（AC-2） | 基线 `134/159 = 0.8428` → `0/159 = 0.0000` | 基线 `99/109 = 0.9083` → `0/109 = 0.0000` |

改动行数构成：

| 来源 | 文件 1 | 文件 2 | 说明 |
|---|---|---|---|
| AC-2 判定的英文残留行 | 134 | 99 | 即 `check` 基线报告的 offenders 全量，逐条译完，无遗漏 |
| `Run:` / `Run:` 命令引导行 | 14 | 1 | AC-2 不计数（`Run` 仅 3 个字母，不触发 `[A-Za-z]{4,}`），但属英文散文，按要求译为 `运行：` |
| 中英混排段落 | 1（第 999 行） | 1（第 15 行） | 见 §3 说明 |

改动内容按类别（以基线原文行首特征归类）：

| 类别 | 文件 1 | 文件 2 | 处理方式 |
|---|---|---|---|
| `- [ ] **Step N: …**` 步骤标题 | 30 | 21 | 译为 `- [ ] **步骤 N：中文标题**` |
| 其他说明性长句（导语、注意事项、术语说明等） | 29 | 25 | 逐条精确替换 |
| 自查清单条目（`- … [x]` / `[+]`） | 14 | 11 | 译文 + 保留尾部 `[x]` / `[+]` 标记 |
| `运行：` 命令引导行 | 14 | 1 | `Run: …` → `运行：…` |
| `预期：` 引导行 | 11 | 8 | `Expected: …` → `预期：…` |
| 手动验证编号步骤（`1.` ~ `10.`） | 10 | 6 | 整句译出，命令与字符串保留 |
| 表格行（表头 + 数据行） | 5 | 9 | 仅译表头与单元格散文，`Some("AtomGit-*")` 等字面量保留 |
| `**文件：**` 小标题 | 6 | 2 | `**Files:**` → `**文件：**` |
| 文件清单行 | 8 | 2 | `- Modify:` → `- 修改：`、`- Create:` → `- 新建：`、`- Test:` → `- 测试：`、`- (None — …)` → `-（无 —— 仅验证。）` |
| `## Task N: …` 任务标题 | 6 | 3 | `## 任务 N：中文标题` |
| `**粗体小标题**`（`Goal:` / `Architecture:` / `Tech Stack:` / `Spec coverage:` / `Placeholder scan:` / `Type consistency:`） | 6 | 6 | `**目标：**` / `**架构：**` / `**技术栈：**` / `**1. Spec 覆盖情况：**` / `**2. 占位符扫描：**` / `**3. 类型一致性：**` |
| 其他 `##` 章节标题 | 4 | 5 | `## 参考`、`## 文件结构`、`## 手动集成验证`、`## 自查清单（交接前执行）` 等 |
| 括号补充说明段 | 4 | 0 | 如 `（不写超时测试 —— …）`、`（若无需修补，跳过该提交。）` |
| 一级标题 | 1 | 1 | 保留 `Vision Preprocessor` 与 `/codingplan` 原样，仅译剩余部分 |
| 开头引用块（agentic worker 提示） | 1 | 1 | 保留 `superpowers:subagent-driven-development`、`superpowers:executing-plans` 与 `` `- [ ]` `` |

### 2. 术语与命名一致性

- 两文件统一采用「中文正文 + 英文原样符号」：`Step` → `步骤`、`Task` → `任务`、`Files` → `文件`、`Modify/Create/Test` → `修改/新建/测试`、`Expected` → `预期`、`Run` → `运行`；`[x]` / `[+]` 勾选标记、`---` 分隔线、`| … |` 表格语法全部保持原样。
- 保留原文（未译、未改写）：文件路径与目录名（`crates/rustcode-core/src/…`、`~/.rustcode/config.toml`、`docs/superpowers/specs/…`）、crate 名（`rustcode-core`、`rustcode-cli`）、命令与 CLI flag（`cargo test -p rustcode-core --lib`、`--workspace --all-targets`、`--nocapture`、`-D warnings`、`grep -n`）、TOML 表名与配置键（`vision_preprocessor_provider`、`[providers."RustCode-Qwen-Qwen3-VL-32B-Instruct"]`、`[dev-dependencies]`）、函数名/类型名/枚举变体/trait 名（`maybe_preprocess`、`model_name_suggests_vision`、`is_codingplan_provider_name`、`provider_names_for`、`create_provider`、`PreprocessOutcome::{Skipped,Replaced,Failed}`、`VisionPreprocessorOutcome::{UnchangedNone,UserSupplied,AutoSet,Cleared}`、`AgentEvent::Warning`、`StreamEvent::Delta`、`LlmProvider`、`Config`、`ModelsInfo`、`SetupReport::render`、`StepResult::Ok`）、第三方专名（Rust、Tokio/`tokio`、`async-trait`、`wiremock`、`OpenAiProvider`、`PaddleOCR`、`GOT-OCR`、`MonkeyOCR`、`MinerU-OCR`、`AtomGit`、`SiliconFlow`、`Claude`、`DeepSeek`、`TUIX`、`Claude Opus 4.7`）、提交号与行号（`1379510..4ce8bc0`、`82-125`、`~575`）、数字与时长（30s →「30 秒」仅改量词，数值 30 未动）。
- 领域词按上下文取通用译法：`short-circuit` → 短路、`heuristic` → 启发式、`placeholder` → 占位符、`blast radius` → 影响范围、`trip-wire` → 绊线、`happy path` 保留英文（Rust 测试惯用词）、`provider` 在指代配置实体时保留英文、在指代角色时译作「provider」（与仓库既有中文文档一致）。

### 3. 残留清单与白名单说明

自检口径（AC-2）下两文件英文残留均为 **0 行**。以下为改后仍可见的非中文内容，全部属于铁律白名单，**不属于遗漏**：

| 残留位置 | 内容 | 白名单理由 |
|---|---|---|
| 文件 1：19 个代码块；文件 2：25 个代码块（围栏内全部 1237 行） | Rust / bash / TOML 源码、注释、字符串（含 `// VL HTTP call lands in Task 3`、`Co-Authored-By: Claude Opus 4.7 (1M context)`、`OCR-on-VLM endpoints (PaddleOCR, GOT-OCR…)` 等英文注释与提交信息） | 铁律 1：不翻译 fenced code block 内任何内容，含注释与字符串。已用逐字节比对确认围栏行与围栏内正文改动前后完全一致 |
| 文件 1：236 处；文件 2：167 处 inline code | 路径、crate 名、命令与 flag、环境变量、配置键、`Msg`/枚举变体、函数名等 | 铁律 2：不翻译 inline code |
| 上表 §2 列出的全部路径、命令、flag、配置键、类型名、专名、版本号、日期、数字 | 同左 | 铁律 3：保留原文 |
| 文件 1：第 11、23、36、138、407、637、814、950、988、1005 行；文件 2：第 11、19、33、42、175、726、779、790 行 | `---` 水平分隔线（共 18 行） | 无语言属性的 Markdown 排版符号，译为任何文字都会破坏文档结构 |
| 文件 1 第 908、959、965、971 行等内联 `` `cargo …` `` | 命令行本身 | 铁律 2/3：命令属 inline code，不译 |

两处**中英混排段落的处置**（需编排者知悉，非疏漏）：

| 位置 | 基线原文 | 处置 | 理由 |
|---|---|---|---|
| 文件 1 第 999 行 | `6. Ctrl+V paste a code-screenshot, append caption "解释这段代码", press Enter.` | **已译**为 `6. Ctrl+V 粘贴一张代码截图，追加 caption "解释这段代码"，按 Enter。` | 该行虽因含中文字符串被 AC-2 判为中文行（不计入残留），但其散文语言是英文；若保留会与已译成中文的相邻步骤 1–5、7–10 形成中英并列，违反「不保留英文原文段落」。其中的中文 caption 字面量 `"解释这段代码"` 一字未动 |
| 文件 2 第 15 行 | `Spec at … (the original feature). This plan addresses the §风险与权衡 item 4 follow-up …` | **已译**；`§风险与权衡` 一字未动 | 同上：散文语言是英文，`§风险与权衡` 是对 spec 章节的引用（属需保留的原文，不是可译的中文散文） |

除这两行外，两文件中不存在其他中英混排散文行（已用「剥离 inline code 后同时含 ASCII 字母与 CJK」的规则全量扫描确认，命中数仅为上述 2 行）。

### 4. 自检四项实际结果

执行命令：

```bash
python3 scripts/check-zh-docs.py check --base 3ee655e3 \
  --report .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-03.md \
  --files docs/superpowers/plans/2026-05-08-vision-preprocessor.md \
          docs/superpowers/plans/2026-05-08-vision-preprocessor-auto-config.md
```

脚本输出：`PASS …vision-preprocessor.md en=0/159=0.0000`、`PASS …vision-preprocessor-auto-config.md en=0/109=0.0000`、`check: 受检 2，PASS 2，FAIL 0`（退出码 0）。

| 验收项 | 判据 | 实测（文件 1 / 文件 2） | 结论 |
|---|---|---|---|
| AC-2 | `en/total <= 0.05` | `0/159 = 0.0000`（基线 `134/159 = 0.8428`）；`0/109 = 0.0000`（基线 `99/109 = 0.9083`） | PASS |
| AC-4 | inline code + fenced code 集合与改动前完全相等 | inline code 出现次数 `236 = 236` / `167 = 167`，多重集逐项相等；去重值 `142 = 142` / `74 = 74`；围栏行 + 围栏内正文 `735 = 735` / `590 = 590` 行，逐字节相等 | PASS |
| AC-7b | `](...)` 链接目标多重集相等 | `0 = 0` / `0 = 0`（两文件均无 Markdown 链接，故不存在链接目标被改写） | PASS |
| AC-32 | Emoji 数不增加 | `0 → 0` / `4 → 4`，未新增任何 Emoji，也未清理代码块内既有的 `✔`、`⚠` | PASS |

附加核对（非本任务硬性门禁，一并记录）：改动前后总行数不变（1031 / 810 行）；两文件首行均非 `---`，无 YAML frontmatter，AC-6 键名集合 `[] = []`，与铁律 7 一致；标题级别、文件名、代码块语言标注（```rust / ```bash）全部未变。

### 5. 未验证范围与遗留

- 未执行 `cargo` / 构建 / 测试：本任务为纯文档汉化，不涉及源码与测试。
- 代码块内的英文（含注释与字符串）按铁律一律未译；其中存在面向读者的说明性英文注释与 `git commit -m` 英文提交信息，若后续要求一并汉化需另行授权。
- 同源 spec `docs/superpowers/specs/2026-05-08-vision-preprocessor-design.md` 不在 `files_owned` 内，未检查、未改动；文件 2 第 15 行引用的 `§风险与权衡` 章节名取自该 spec，若该 spec 后续被汉化，此处引用需同步复核。
- 文件 2 代码块中的绝对路径 `cd /Users/theo/Documents/workspace/rustcode/` 属原文内容，按铁律 1 保留未改。
- `git diff --numstat` 对文件 1 报 `153/153`（实际内容改动 149 行）：差额 4 组是 diff 算法在空白行上产生的空配对（`-` 与 `+` 内容均为空字符串），无内容差异；用 `--diff-algorithm=patience` 可复现该差额来源，文件内容以逐行比对结果为准。
- 遗留建议：两文件任务编号不连续（`vision-preprocessor.md` 为任务 1–6，`-auto-config.md` 为任务 7–9，后者是前者的追加计划）。若后续要合并或重排，需要在 `docs/superpowers/` 内统一，本轮未做。
