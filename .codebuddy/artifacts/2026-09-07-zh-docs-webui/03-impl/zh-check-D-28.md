# zh-check 报告

- 基线 (--base): `3ee655e3`
- 阈值 (--en-ratio): `0.05`
- 判定: AC-2 / AC-4 / AC-6 / AC-7b / AC-32，任一失败即 FAIL（fail-closed）

## 汇总

- 受检文件: 19
- PASS: 19
- FAIL: 0

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/00-requirement.md`

- en=0/284 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/01-design.md`

- en=0/246 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/02-tasks.md`

- en=7/173 ratio=0.0405
- AC-3 残留行清单 (en 行，共 7 行):
  - `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/02-tasks.md:34`: `| **crate** | 'rustcode-codingplan'、'rustcode-config' |`
  - `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/02-tasks.md:35`: `| **files_owned** | 'crates/rustcode-codingplan/src/lib.rs'<br>'crates/rustcode-codingplan/src/types.rs'<br>'crates/rustcode-codingplan/src/client.rs'<br>'crates/rustcode-codingplan/src/setup.rs'<br>'crates/rustcode-codingplan/src/sync_marker.rs'<br>'crates/rustcode-config/src/i18n/messages.rs' |`
  - `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/02-tasks.md:54`: `| **crate** | 'rustcode-cli' |`
  - `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/02-tasks.md:55`: `| **files_owned** | 'crates/rustcode-cli/src/main.rs' |`
  - `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/02-tasks.md:76`: `| **files_owned** | 'docs/config.example.toml' |`
  - `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/02-tasks.md:156`: `| **files_owned** | '.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/03-impl/T9-integration-verification.md' |`
  - `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/02-tasks.md:176`: `| **files_owned** | 'AGENTS.md'（**dirty**） |`

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/06-release.md`

- en=0/62 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/STATUS.md`

- en=0/81 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/03-impl/T1.md`

- en=0/32 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/03-impl/T3.md`

- en=0/24 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/03-impl/T4.md`

- en=0/15 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/03-impl/T5.md`

- en=0/19 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/03-impl/T6.md`

- en=0/12 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/03-impl/T7-dead-code-scan.md`

- en=0/33 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/03-impl/T9-integration-verification.md`

- en=0/44 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/04-review/T1.md`

- en=0/33 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/04-review/T3.md`

- en=0/10 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/04-review/T4.md`

- en=0/8 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/04-review/T5.md`

- en=0/6 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/04-review/T6.md`

- en=0/5 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/04-review/T7.md`

- en=0/8 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/04-review/T9.md`

- en=0/12 ratio=0.0000

---

# 手写小结（D-28，任务 ID `D-28`）

> 本节在**最后一次**自检之后追加（铁律 §5 第 5 条：`--report` 会覆盖报告，手写内容必须后置）。

## 1. 批次概况

- 任务：汉化批次 10 / **D-28**，目标目录 `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/`。
- `files_owned` **19 个文件**（根级 5 + `03-impl/` 7 + `04-review/` 7），合计约 180 KB。
- 自检命令（与铁律 §4 一致，基线 `3ee655e3`）：
  ```bash
  python3 scripts/check-zh-docs.py check --base 3ee655e3 \
    --report .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-28.md \
    --files <本批 19 个文件相对路径，空格分隔>
  ```
- 总判定：`受检 19，PASS 19，FAIL 0`。**改动前**为 `受检 19，PASS 16，FAIL 3`（`03-impl/T1.md`、`04-review/T5.md`、`04-review/T6.md` 因 AC-2 超限 FAIL），本批把 3 个 FAIL 全部转为 PASS。

## 2. 每个文件的 no-op / 改动判定

| 文件 | 改动前 en/total | 改动后 en/total | 判定 | 说明 |
|---|---|---|---|---|
| `00-requirement.md` | 0/284 | 0/284 | **no-op** | 实测已全中文，无 offender |
| `01-design.md` | 0/246 | 0/246 | **no-op** | 无 AC-2 offender；另有 2 处英文本属白名单，见 §4 |
| `02-tasks.md` | 7/173 = 0.0405 | 7/173 = 0.0405 | **no-op（白名单残留）** | 7 行残留全为表格字段名 + inline code，见 §4 |
| `06-release.md` | 1/62 = 0.0161 | **0/62** = 0.0000 | **改动 1 行** | 标题 `breaking change` → `破坏性变更` |
| `STATUS.md` | 0/81 | 0/81 | **no-op** | 实测已全中文，无 offender |
| `03-impl/T1.md` | 2/32 = 0.0625 | **0/32** = 0.0000 | **改动 2 行**（FAIL→PASS） | 见 §3 |
| `03-impl/T3.md` | 0/24 | 0/24 | **no-op** | 实测已全中文，无 offender |
| `03-impl/T4.md` | 0/15 | 0/15 | **no-op** | 实测已全中文，无 offender |
| `03-impl/T5.md` | 0/19 | 0/19 | **no-op** | 实测已全中文，无 offender |
| `03-impl/T6.md` | 0/12 | 0/12 | **no-op** | 实测已全中文，无 offender |
| `03-impl/T7-dead-code-scan.md` | 0/33 | 0/33 | **no-op** | 实测已全中文，无 offender |
| `03-impl/T9-integration-verification.md` | 1/44 = 0.0227 | **0/44** = 0.0000 | **改动 1 行** | 见 §3 |
| `04-review/T1.md` | 0/33 | 0/33 | **no-op** | 实测已全中文，无 offender |
| `04-review/T3.md` | 0/10 | 0/10 | **no-op** | 实测已全中文，无 offender |
| `04-review/T4.md` | 0/8 | 0/8 | **no-op** | 实测已全中文，无 offender |
| `04-review/T5.md` | 1/6 = 0.1667 | **0/6** = 0.0000 | **改动 1 行**（FAIL→PASS） | 见 §3 |
| `04-review/T6.md` | 1/5 = 0.2000 | **0/5** = 0.0000 | **改动 1 行**（FAIL→PASS） | 见 §3 |
| `04-review/T7.md` | 0/8 | 0/8 | **no-op** | 实测已全中文，无 offender |
| `04-review/T9.md` | 0/12 | 0/12 | **no-op** | 实测已全中文，无 offender |

合计：**14 个 no-op，5 个文件共改动 6 行**（`git diff --stat` = `5 files changed, 6 insertions(+), 6 deletions(-)`，行数中性，纯行内替换，未增删空行）。

## 3. 本批实际改动（逐条）

改动全部落在「补入真实汉字」这一条上，**未触碰**任何 inline code、fenced code、链接、Emoji、frontmatter：

1. `03-impl/T1.md:53`
   `- \`cargo check -p rustcode-config --all-targets -j 1\` → Finished ✅`
   → 末尾补 `（rustcode-config 全部编译）`，与相邻的 `:52` 行（`→ Finished（…全部编译）`）措辞一致。
2. `03-impl/T1.md:56`
   `- \`cargo test -p rustcode-codingplan --lib sync_marker -j 1\` → 3 passed ✅`
   → 末尾补 `（3 项测试通过）`。
3. `03-impl/T9-integration-verification.md:34`（AC-3 表格行「证据」列）
   `| … | Finished |` → `| … | Finished（编译完成） |`；与同表 `:35`（`均 Finished`）、`:32` 等既有写法对齐。
4. `04-review/T5.md:20` → 行首补 `结论：`，成为 `结论：**approve**。0 blocker / 0 critical / 0 major。`
5. `04-review/T6.md:19` → 同上，行首补 `结论：`。
6. `06-release.md:67` 标题
   `### ② \`rustcode codingplan\` → exit 2（breaking change）` → `（破坏性变更）`。

**为何必须「补真实汉字」而不是加全角标点**：铁律 §5 第 2 条指出全角括号/句号不在脚本 `CJK_RE`（`\u3400-\u4dbf`、`\u4e00-\u9fff`、`\uf900-\ufaff`、`\u3040-\u30ff`）判定区间内。本批 3 个 FAIL 行正是典型：它们**原本就带** `。` / `（）`（如 `**approve**。0 blocker…`），加全角标点无效；本批补入的 `结论`、`全部编译`、`编译完成`、`项测试通过` 均落在 `U+4E00–U+9FFF`，故 AC-2 转 0。

**`approve` 一词刻意保留英文**：`04-review/T5.md` / `T6.md` 的 frontmatter 有 `decision: approve`，正文 `**approve**` 与之一致；按铁律 §7 与本任务书第 4 条，枚举值保留英文以免破坏下游解析，本批只在其**前面**加中文引导词，未改词本身。

**`06-release.md:67` 仅改标题、未改正文**：`breaking change` 在本 feature 内共出现 9 次，其中 8 次位于**已是中文的句子内部**（如 `01-design.md:391`、`02-tasks.md:62/157`、`03-impl/T3.md:47`、`04-review/T3.md:24`、`06-release.md:21`、`03-impl/T9:77`），按铁律 §8「已中文的段落不润色、不改写」一律**原样保留**，故未做全批替换。改动前已确认本 feature 目录下**不存在任何 `)#` 锚点链接**，改标题不会触发 AC-7b。

## 4. 残留行清单与白名单判定

- **`02-tasks.md` 残留 7 行（en=7/173 = 0.0405 ≤ 0.05，PASS）**，逐行说明：

  | 行 | 内容（去 inline code 后） | 为何属白名单 |
  |---|---|---|
  | `:34` | `\| **crate** \|` | `crate` 是 Rust 术语兼本表字段名；同行其余为 inline code 包名，铁律 §3 第 3 条保留 |
  | `:35` | `\| **files_owned** \|` | `files_owned` 是 frontmatter 键名，铁律 §7 + 本任务书第 1 条明确保留英文 |
  | `:54` | `\| **crate** \|` | 同 `:34` |
  | `:55` | `\| **files_owned** \|` | 同 `:35` |
  | `:76` | `\| **files_owned** \|` | 同 `:35` |
  | `:156` | `\| **files_owned** \|` | 同 `:35` |
  | `:176` | `\| **files_owned** \|` | 同 `:35` |

  这 7 行的共同结构：`| **<字段名>** | <inline code 路径/包名> |`。剥离 inline code 后**只剩字段名**这一个英文词，命中的是结构化标签而非散文；若要消除，只能把 `crate` / `files_owned` 这两个**必须保留英文**的字段名改掉，或插入无意义的中文填充，二者都违反铁律 §3/§7。故判为白名单残留，文件记 **no-op**（0.0405 ≤ 0.05）。

- **`01-design.md:103-104` 两处英文短语**（`and its hidden alias` / `and its Codingplan alias`）：
  - 这两行**已含中文**（`同步修（注释，与 D 类同性质）`），AC-2 计 `en=0`，非 offender；
  - 英文部分是**对源码注释原文的逐字引用**（`cli/main.rs:1669-1670`、`:3519` 的注释内容），作用是让实现方能按原文定位到待改注释；
  - 翻译会使其与代码实际文本失配、丧失定位用途，且该行已是中文（铁律 §8）。故**保留**，在此登记备查。

- **无标题级英文残留**：另以 `grep` 扫描 19 个文件的全部 `^#{1,6} ` 标题行，筛选「不含 `\u4e00-\u9fff`」者，**命中 0 条**——所有标题均已含汉字。
- **无隐藏英文散文**：另用一次性脚本扫描代码块/缩进块之外「≥4 个连续英文词」的短语，全批仅命中上列 `01-design.md:103-104` 两处，已在上面说明。

## 5. 四项实测结果

| 项 | 判定 | 实测 |
|---|---|---|
| **AC-2** `en/total <= 0.05` | **PASS（19/19）** | 改动后 18 个文件 `en=0`；唯一非零 `02-tasks.md` 为 `7/173 = 0.0405`，属 §4 白名单。改动前 3 个 FAIL（`T1.md` 0.0625、`04-review/T5.md` 0.1667、`04-review/T6.md` 0.2000）已全部转 PASS |
| **AC-4** inline code + fenced code 多重集相等 | **PASS（19/19）** | 无一文件报差异。本批 6 处改动全部位于 inline code **之外**、且未新增任何反引号（规避铁律 §5 第 1、3 条） |
| **AC-7b** `](...)` 链接目标多重集相等 | **PASS（19/19）** | 无一文件报差异；改标题前已确认无 `)#` 锚点引用 |
| **AC-32** Emoji 数不增加 | **PASS（19/19）** | 无一文件报 `emoji 命中数增加`；`T1.md` 两行的 `✅` 原样保留（铁律 §5 第 6 条：既不加也不删） |

脚本总判定（末次运行）：`受检 19，PASS 19，FAIL 0`。

## 6. frontmatter 与 AC-6 核对

- **无 `description` 键**：对 19 个文件 `grep '^description:'` → **0 命中**，故铁律 §7 的「`description` 值可汉化需逐条列示」在本批**无适用项**，AC-6 未产生 `AC-6-desc-changed` 记录。
- 有 frontmatter 的文件（`00-requirement.md`、`01-design.md`、`02-tasks.md`、`06-release.md` 及 `03-impl/*`、`04-review/*`）：**键名一律保持英文**，`kind` / `id` / `from` / `to` / `feature` / `status` / `decision` / `requires` / `files_owned` / `architecture_constraints` / `created` 与基线逐字一致，AC-6 键名集合相等。
- **枚举值保留英文**：`status: done`、`decision: review` / `approve` / `proceed`、`requires: [TASKS-001]` 等一律未动（下游会解析）。
- `STATUS.md` 与部分文件无 frontmatter，本批未为其补写（超出汉化范围）。

## 7. 未验证范围

- 未执行 `cargo` / `npm` 构建与测试（铁律 §7：纯文档任务，禁止）。
- 本批只对 `files_owned` 的 19 个文件自检；`.codebuddy/artifacts/2026-09-07-zh-docs-webui/` 下除本报告 `zh-check-D-28.md` 外**未触碰任何文件**（该目录其余文件由其它任务负责）。
- `02-tasks.md` 的 7 行白名单残留未消除，理由见 §4；若编排者认为应连字段名一并中文化，需先确认 `files_owned` / `crate` 是否允许改（会影响其它批次的一致性），本批不擅自处理。
- `01-design.md:103-104` 的源码注释原文引用未翻译，理由见 §4；如需改为「中文译名 + 英文原文」并列形式，需编排者确认可接受，本批未动。
- 未对 19 个文件做 YAML 解析级校验（无 `description` 键且本批未改任何 frontmatter 行，改动面不含 frontmatter，故未执行）。
