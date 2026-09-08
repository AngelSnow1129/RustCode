# zh-check 报告

- 基线 (--base): `3ee655e3`
- 阈值 (--en-ratio): `0.05`
- 判定: AC-2 / AC-4 / AC-6 / AC-7b / AC-32，任一失败即 FAIL（fail-closed）

## 汇总

- 受检文件: 3
- PASS: 3
- FAIL: 0

## PASS `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/03-impl/T6-docs-atomcode.md`

- en=2/194 ratio=0.0103
- AC-3 残留行清单 (en 行，共 2 行):
  - `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/03-impl/T6-docs-atomcode.md:284`: `| docs/features.md:113-114 | 'G7  crates/scripts/.github 无 atomcode 残留' / 'G8  docs/architecture.md 无 atomcode 残留' |`
  - `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/03-impl/T6-docs-atomcode.md:285`: `| docs/platform-neutralization.md:204-206 | 'G7  crates/ scripts/ .github/ 无 atomcode 残留(atomgit feature / 旧前缀兼容 /' / '    fork 发行主页三类除外)' / 'G8  docs/architecture.md 无 atomcode 残留' |`

## PASS `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/03-impl/T8-artifacts.md`

- en=0/79 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/HANDOFF-codingplan-legacy.md`

- en=1/77 ratio=0.0130
- AC-3 残留行清单 (en 行，共 1 行):
  - `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/HANDOFF-codingplan-legacy.md:80`: `| coding --lib / updater --lib | 430/0、41/0 |`
- R5 跳过的行 (共 1 行):
  - `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/HANDOFF-codingplan-legacy.md:103`: `     + 'event_loop::tool_format_tests::summarise_*' ×4`

## 附录 · 被 R5 跳过的行（供抽检）

共 1 行。

- `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/HANDOFF-codingplan-legacy.md:103`: `     + 'event_loop::tool_format_tests::summarise_*' ×4`

---

# 手写小结（D-30 · 批次 10）

## 1. 结论摘要

本批 3 个文件在基线 `3ee655e3` 上**正文已基本为中文**。按本批特殊说明 4「先跑一次自检拿 offender 清单再动手」
与铁律 §3「只处理 `check` 报出的英文 offender 行」，首轮自检共报出 **4 行 offender**，逐行分类后：

- **可清 1 行**：`HANDOFF-codingplan-legacy.md:78` 的 `0 error` → `0 错误`（已改）。
- **不可清 3 行**：`T6-docs-atomcode.md:284` / `:285` 与 `HANDOFF-codingplan-legacy.md:80`，
  其英文全部落在 inline code、文件路径、命令 flag、crate 名与数字内，受铁律 §2 / §3 保护，
  **按白名单保留**（理由见 §3）。

`T6-docs-atomcode.md` 与 `T8-artifacts.md` 判定为 **no-op（零写入）**；
`HANDOFF-codingplan-legacy.md` 仅清 1 行 offender，**未改写任何已中文段落**（铁律 §8）。

前置事实核对：改动前 `git --no-pager diff --stat 3ee655e3 -- <3 个文件>` 输出为空，
即 3 个文件**与基线逐字节相同、基线本身即中文**，不存在「早前批次遗留改动需原样保留」的情况。
本批总写入量：**1 文件 / 1 行**，行数零变化（108 → 108），无增删空行，
不存在铁律 §5.4 / §5.7 所述问题。

## 2. 逐文件改动量

| 文件 | 行数（基线 → 现） | en/total（改前 → 改后） | 本批改动 | 判定 |
|---|---|---|---|---|
| `03-impl/T6-docs-atomcode.md` | 361 → 361 | 2/194 = 0.0103 → 0.0103 | 0 行 | **no-op**（2 行 offender 均不可清，白名单） |
| `03-impl/T8-artifacts.md` | 122 → 122 | 0/79 = 0.0000 → 0.0000 | 0 行 | **no-op**（无 offender） |
| `HANDOFF-codingplan-legacy.md` | 108 → 108 | 2/77 = 0.0260 → **1/77 = 0.0130** | 1 行（:78） | **清 1 行 offender**，余 1 行白名单 |

## 3. 残留行清单与白名单判定（逐行）

### 3.1 `HANDOFF-codingplan-legacy.md:78` —— 已清（本批唯一改动）

- 改动前：``| G2 `cargo check -j 1 --workspace --all-targets` | exit=0 / 0 error |``
- 改动后：``| G2 `cargo check -j 1 --workspace --all-targets` | exit=0 / 0 错误 |``
- 改动范围：仅反引号**外**的英文词 `error` → `错误`。
  反引号内的命令与 CLI flag `cargo check -j 1 --workspace --all-targets`（铁律 §3）、
  退出码字面量 `exit=0`、以及数字 `0` **全部原样保留**。
- 未新增反引号、未改链接、未增删空行、未新增 emoji，故 AC-4 / AC-7b / AC-32 实测仍 PASS。
- 改动的**一致性依据**：同一 feature 的 `STATUS.md:265` 记录同一 G2 门禁结果，
  并行批次 D-29 已将其 `exit=0 / 0 error` 改为 `exit=0 / 0 错误`
  （`git diff 3ee655e3 -- STATUS.md` 可见该 hunk）。本批跟进后，
  两份交接件对同一门禁状态的表述恢复一致。

### 3.2 `T6-docs-atomcode.md:284` / `:285` —— 白名单（不可清）

- 两行是「规则 B 保留项（未改动，列出以证完整性）」表体，行内 ASCII 只来自两类内容：
  1. inline code 内**引用的门禁定义原文**（`G7  crates/scripts/.github 无 atomcode 残留` 等）；
  2. 首列的**文件路径** `docs/features.md:113-114`、`docs/platform-neutralization.md:204-206`。
- 被判为 en 行的**成因**：脚本 R4 会先剥离 inline code 再判定，而汉字「无 / 残留」位于反引号内，
  被一并剥离；剥离后仅剩 ASCII 文件路径，故 `ASCII4` 命中且无 CJK。
- **不可修**：
  - 反引号内是被引用的**原文**，铁律 §2 禁止翻译 inline code；且按本批特殊说明 2，
    `atomcode` 是被改掉的历史包名，**必须保留原文**，改动会使门禁定义记录失真。
  - 反引号外的 ASCII 只有文件路径，铁律 §3 禁止改动。
  - 若要在行内补入汉字，只能靠**新增**叙述性文字；这既违反铁律 §3 / §8（不改写已中文段落），
    又属越界新增内容（铁律 §7 边界：只做汉化，不新增信息）。

### 3.3 `HANDOFF-codingplan-legacy.md:80` —— 白名单（不可清）

`coding` / `updater` 为 crate 名、`--lib` 为 cargo flag（铁律 §3 保留），
`430/0、41/0` 为数字（铁律 §3 保留）。**全行无可译散文**。

### 3.4 `HANDOFF-codingplan-legacy.md:103`（R5 跳过，不进 total）

`+ 'event_loop::tool_format_tests::summarise_*' ×4` —— 缩进续行，
ASCII 全部位于 inline code 内的**测试路径**（铁律 §2 / §3 保留）。

### 3.5 frontmatter（T6:2-15、T8:2-15，共 28 行）

键名 `kind` / `id` / `from` / `to` / `feature` / `status` / `decision` / `requires` /
`files_owned` / `created` **一律保留英文**（铁律 §7）；
其值为 feature slug、任务 ID、日期与文件路径（铁律 §3 保留）。
脚本 R1 已整块剥离 frontmatter，不计入 `en`。

### 3.6 历史名 `atomcode` 与现役名 `rustcode-*` 的处理声明

按本批特殊说明 2，**历史名 `atomcode` 保留原文，现役 crate 名 `rustcode-*` 亦保留原文，两者均未改动**。
实测核对：`T6-docs-atomcode.md` 中 `atomcode` 共出现 **29 处**，分布为 —— 标题（`:18`）、
规则 B 门禁定义引用（`:85`、`:86`、`:91`、`:93`、`:284`、`:285`）、
diff 代码块内的前后对照（`:231`、`:243`、`:254`、`:261`、`:269`）、
术语与结论段（`:32`、`:33`、`:41`、`:68`、`:70`、`:160`、`:173`、`:178`、`:214`、`:278`、`:341`、`:342`、`:345`）、
以及自身路径与补丁文件名（`:303`、`:322`、`:328`）。
**全部属记录性引用或代码块内容，无一处需要清理或翻译。**

## 4. AC-6：`description` 逐条对照

| # | 文件 | 是否有 frontmatter | `description` 键 | 本批是否改动 |
|---|---|---|---|---|
| 1 | `03-impl/T6-docs-atomcode.md` | 是（第 1–16 行，`---` 包裹） | **无** | 否 |
| 2 | `03-impl/T8-artifacts.md` | 是（第 1–15 行，`---` 包裹） | **无** | 否 |
| 3 | `HANDOFF-codingplan-legacy.md` | **无**（首行即 `# 交接件：g1-fmt-gate → cleanup-codingplan-legacy`） | 无 | 否 |

3 个文件**均不含 `description` 键**，故不存在 description 汉化项，AC-6 无任何差异需要列出。

**前提偏差（上报，不擅自处理）**：本批特殊说明 3 提到「`HANDOFF-codingplan-legacy.md` 是交接件，
其 frontmatter 键名保留英文」，但该文件**实际不含 YAML frontmatter**。
本批按其现状处理（无 frontmatter 即无键名可改），**未凭空补写**；
是否需要补 frontmatter、采用何种键集，属改 schema，超出汉化范围，**交由编排者裁定**。

## 5. 四项实测结果

| 项 | 判定 | 实测 |
|---|---|---|
| **AC-2** 英文占比 `<= 0.05` | **PASS（3/3）** | `T6` 2/194 = 0.0103、`T8` 0/79 = 0.0000、`HANDOFF` **1/77 = 0.0130**（改前 0.0260），全部远低于阈值 `0.05` |
| **AC-4** inline code + fenced code 多重集相等 | **PASS（3/3）** | `T6` / `T8` 与基线逐字节相同（恒等）；`HANDOFF` 唯一改动未触及任何反引号，code span 集合仍与基线相等 |
| **AC-7b** `](...)` 链接目标多重集相等 | **PASS（3/3）** | 3 文件均不含或不含改动 `](...)` 链接，集合与基线相等 |
| **AC-32** Emoji 数不增加 | **PASS（3/3）** | 本批唯一改动为 `error` → `错误`，**未新增任何 emoji** |

脚本总判定：`受检 3，PASS 3，FAIL 0`。

## 6. 未验证范围与遗留项

- 未执行 `cargo` / `npm` 构建与测试（铁律 §7：纯文档任务，禁止）。
- 未做 Markdown 渲染验证（改动仅 1 处行内文字，无结构变化；本会话无渲染工具）。
- 未清理的 3 行白名单 offender（§3.2 / §3.3 / §3.4）均为铁律保护内容，**本批不再处理**；
  若编排者要求强行清零，需先豁免铁律 §2 / §3 或调整 `check-zh-docs.py` 的 R4 剥离口径（上报决策）。
- `HANDOFF-codingplan-legacy.md` 是否补写 frontmatter —— **上报编排者**裁定，本批未补。
- 同目录的 `03-impl/T7-goals-superpowers.md`、`06-release.md`、`STATUS.md`、`05-test-report.md`
  由并行批次 **D-29** 处理，本批**未读取、未改动**。
  唯一交叉引用为 §3.1 中 `STATUS.md:265` 的 `0 error` → `0 错误` 改动，
  系通过 `git diff 3ee655e3 -- STATUS.md` 只读核实，**未与 D-29 做全文一致性核对**（如需，请编排者另派任务）。
- 未触碰 `.codebuddy/artifacts/2026-09-07-zh-docs-webui/` 下除本报告外的任何文件。
