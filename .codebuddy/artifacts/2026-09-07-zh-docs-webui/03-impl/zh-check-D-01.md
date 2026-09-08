# zh-check 报告

- 基线 (--base): `3ee655e3`
- 阈值 (--en-ratio): `0.05`
- 判定: AC-2 / AC-4 / AC-6 / AC-7b / AC-32，任一失败即 FAIL（fail-closed）

## 汇总

- 受检文件: 1
- PASS: 1
- FAIL: 0

## PASS `docs/superpowers/plans/2026-05-25-tuix-unified-in-app-scroll.md`

- en=0/562 ratio=0.0000

---

## D-01 汉化小结（doc-writer 手写）

### 1. 改动范围

- 受改文件（files_owned，唯一）：`docs/superpowers/plans/2026-05-25-tuix-unified-in-app-scroll.md`
- 文件规模：2515 行 / 78.66 KB，含 **118 个代码块**、**595 处 inline code**、0 个 `](...)` 链接、0 个 Emoji。
- 实际改动：**515 行**（全部为非围栏正文行），`git diff --stat` 为 `1 file changed, 515 insertions(+), 515 deletions(-)`，未触碰其他任何文件。
- 英文残留率：改动前 `en=437/562 = 0.7776` → 改动后 `en=0/562 = 0.0000`。

改动内容按类别：

| 类别 | 改动行数 | 处理方式 |
|---|---|---|
| `- [ ] **Step N: …**` 步骤标题 | 156 | `- [ ] **步骤 N：中文标题**`；88 个去重标题逐个译出 |
| 其他说明性长句与标题（步骤正文、注意事项、文件清单导语等） | 125 | 逐条精确替换 |
| `Run: …` / `Run:` 命令引导 | 59 | `运行：…` / `运行：` |
| `Expected: …` 预期引导 | 59 | `预期：…`；27 种去重句式逐个译出 |
| `- Modify: …` 文件清单行 | 42 | `- 修改：…` |
| `### Task X.Y: …` 任务标题 | 31 | `### 任务 X.Y：中文标题`；全部 31 个逐个译出 |
| `**Files:**` / `**Files:** none` | 31 | `**文件：**` / `**文件：** 无` |
| `## Phase N: …` 阶段标题 | 9 | `## 阶段 N：中文标题`（Phase 1 见下） |
| `- Create: …` 文件清单行 | 3 | `- 新建：…` |
| **合计** | **515** | |

术语遵循项目约束：中文正文 + 英文原样符号。`retained` / `alt-screen` / `view_mode` / `sticky` / `OSC 52` / `arboard` / `scrollbar` / `MessageMark` / `Msg::Xxx` / `UiLine::Xxx` / `MarkKind::Xxx` / `crossterm` / `conhost` / Rust / Windows / macOS 等一律保留原文；`cargo check -p rustcode-tuix`、`grep -nE …`、`\x1b[3J`、`?1002h ?1006h` 等命令与转义序列一字未改。

### 2. 残留清单与白名单说明

自检口径（AC-2）下的英文残留为 **0 行**。以下为「肉眼可见的非中文内容」，均属铁律白名单，**不属于残留缺陷**：

| 残留位置 | 内容 | 属于白名单的理由 |
|---|---|---|
| 118 个代码块（围栏内全部行） | Rust / bash / TOML 源码、注释、字符串 | 铁律 1：不翻译 fenced code block 内任何内容，含注释与字符串 |
| 595 处 inline code | 路径、crate 名、函数名、命令、flag、`Msg` 变体等 | 铁律 2：不翻译 inline code |
| 文件路径、crate 名（`rustcode-cli` 等）、命令与 CLI flag、环境变量 `RUSTCODE_*`、配置键、`Msg::Xxx` 变体、枚举变体、函数名/类型名/hook 名、第三方专名（Rust / Tokio / Kitty / Windows / macOS / arboard 等）、版本号与日期数字 | 同左 | 铁律 3：保留原文 |
| 12 行水平分隔线 | `---`（第 11、32、87、456、742、1109、1370、1625、2175、2366、2456、2499 行） | 无语言属性的排版符号，译为任何文字都会破坏 Markdown 结构 |
| 第 1069、1071 行 | `- \`fn reset(&mut self)\``、`- \`fn on_resize(&mut self, ...)\`` | 整行仅由 inline code 构成，无散文可译；改任何字符都会破坏 AC-4 |
| 33 行（第 5、7、9、16-18、21-30、43、89、97、173、236、374、465、526、585、681、1261、1546、2503、2506、2509、2510、2514 行） | 文件中原本已含中文的段落与标题 | 铁律 8：已中文的段落不润色、不改写。含 `**Goal:**` / `**Architecture:**` / `**Tech Stack:**` 摘要、`## Phase 1: Selection 共享模块` 等半中半英标题 —— 按要求原样保留 |

注：`## Phase 1: Selection 共享模块` 与自查清单中 4 条 `Phase 1/4/5/9` 项属于「已含中文」行，按铁律 8 未改写，因此标题风格与相邻条目存在 `阶段 N` / `Phase N` 混用。这是规则约束下的刻意结果，不是遗漏；若编排者希望统一，需明确授权后再改。

### 3. 自检四项实际结果

执行命令：

```bash
python3 scripts/check-zh-docs.py check --base 3ee655e3 \
  --report .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-01.md \
  --files docs/superpowers/plans/2026-05-25-tuix-unified-in-app-scroll.md
```

脚本输出：`PASS docs/superpowers/plans/2026-05-25-tuix-unified-in-app-scroll.md en=0/562=0.0000` / `check: 受检 1，PASS 1，FAIL 0`。

| 验收项 | 判据 | 实测 | 结论 |
|---|---|---|---|
| AC-2 | `en/total <= 0.05` | `0 / 562 = 0.0000`（基线 `437 / 562 = 0.7776`） | PASS |
| AC-4 | inline code + fenced code 集合与改动前完全相等 | code span 多重集 `1780 = 1780`，逐项相等；inline code 出现次数 `595 = 595`；118 个代码块的围栏行全部逐字节未变 | PASS |
| AC-7b | `](...)` 链接目标多重集相等 | `0 = 0`（本文件无任何 Markdown 链接，故不存在链接目标被改写） | PASS |
| AC-32 | Emoji 数不增加 | `0 → 0`，未新增任何 Emoji，也未清理原有内容 | PASS |

附加核对（非本任务硬性门禁，一并记录）：AC-6 frontmatter 键名集合 `[] = []`（本文件无 YAML frontmatter，与铁律 7 一致）；改动前后总行数均为 2515 行，文件结构层级与标题级别未变，文件名未变。

### 4. 未验证范围

- 未执行 `cargo` / 构建 / 测试：本任务为纯文档汉化，不涉及源码与测试。
- 代码块内的英文（含注释与字符串）按铁律一律未译，其中若存在面向读者的说明性英文注释，需另行确认是否要翻译。
- 与本文件同源的 spec `docs/superpowers/specs/2026-05-25-tuix-unified-in-app-scroll-design.md` 不在本次 files_owned 内，未检查、未改动；若后续汉化，阶段/任务标题措辞应与本文件保持一致。
