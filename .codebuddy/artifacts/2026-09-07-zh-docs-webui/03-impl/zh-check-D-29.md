# zh-check 报告

- 基线 (--base): `3ee655e3`
- 阈值 (--en-ratio): `0.05`
- 判定: AC-2 / AC-4 / AC-6 / AC-7b / AC-32，任一失败即 FAIL（fail-closed）

## 汇总

- 受检文件: 4
- PASS: 4
- FAIL: 0

## PASS `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/03-impl/T7-goals-superpowers.md`

- en=2/264 ratio=0.0076
- AC-3 残留行清单 (en 行，共 2 行):
  - `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/03-impl/T7-goals-superpowers.md:370`: `| 'inspector-feedback-1.md:11' | ''| 1 | G7: atomcode in crates/scripts/.github = 0 | PASS | 'grep -rn "atomcode" crates/ scripts/ .github/' = 0 hits |'' |`
  - `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/03-impl/T7-goals-superpowers.md:371`: `| 'inspector-feedback-1.md:12' | ''| 2 | G8: atomcode in docs/architecture.md = 0 | PASS | 'grep -rn "atomcode" docs/architecture.md' = 0 hits |'' |`

## PASS `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/06-release.md`

- en=0/200 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/STATUS.md`

- en=0/247 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/05-test-report.md`

- en=0/181 ratio=0.0000
- R5 跳过的行 (共 6 行):
  - `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/05-test-report.md:245`: `     | 文件 | 差异性质 | 语义影响 |`
  - `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/05-test-report.md:246`: `     |---|---|---|`
  - `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/05-test-report.md:247`: `     | 'tuix/event_loop/commands.rs' | 删除单参调用冗余尾随逗号（'t(..),)' → 't(..))'） | 无 |`
  - `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/05-test-report.md:248`: `     | 'tuix/event_loop/mod.rs' | 结构体字面量尾随逗号增删 | 无 |`
  - `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/05-test-report.md:249`: `     | 'tuix/modals/dir_picker.rs' | 多行调用末参补尾随逗号 | 无 |`
  - `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/05-test-report.md:250`: `     | 'tuix/test_term.rs' | rustfmt 'merge_derives' 合并相邻 '#[derive]' | 无 |`

## 附录 · 被 R5 跳过的行（供抽检）

共 6 行。

- `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/05-test-report.md:245`: `     | 文件 | 差异性质 | 语义影响 |`
- `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/05-test-report.md:246`: `     |---|---|---|`
- `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/05-test-report.md:247`: `     | 'tuix/event_loop/commands.rs' | 删除单参调用冗余尾随逗号（'t(..),)' → 't(..))'） | 无 |`
- `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/05-test-report.md:248`: `     | 'tuix/event_loop/mod.rs' | 结构体字面量尾随逗号增删 | 无 |`
- `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/05-test-report.md:249`: `     | 'tuix/modals/dir_picker.rs' | 多行调用末参补尾随逗号 | 无 |`
- `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/05-test-report.md:250`: `     | 'tuix/test_term.rs' | rustfmt 'merge_derives' 合并相邻 '#[derive]' | 无 |`

---

## 手写小结（D-29 · 批次 10，追加于最后一次自检之后）

### 任务范围

本批 4 个文件全部位于 `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/`：

- `03-impl/T7-goals-superpowers.md`
- `06-release.md`
- `STATUS.md`
- `05-test-report.md`

自检命令（先后执行两次：改动前取 offender 清单、改动后复检）：

```bash
python3 scripts/check-zh-docs.py check --base 3ee655e3 \
  --report .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-29.md \
  --files .codebuddy/artifacts/2026-09-02-g1-fmt-gate/03-impl/T7-goals-superpowers.md \
          .codebuddy/artifacts/2026-09-02-g1-fmt-gate/06-release.md \
          .codebuddy/artifacts/2026-09-02-g1-fmt-gate/STATUS.md \
          .codebuddy/artifacts/2026-09-02-g1-fmt-gate/05-test-report.md
```

### 改动前基线（首跑实测）

| 文件 | en/total（改前） | 判定 |
|---|---|---|
| `03-impl/T7-goals-superpowers.md` | 4/264 = 0.0152 | 已 PASS，属「大段中文」 |
| `06-release.md` | 0/200 = 0.0000 | 已 PASS，no-op |
| `STATUS.md` | 6/247 = 0.0243 | 已 PASS，属「大段中文」 |
| `05-test-report.md` | 4/181 = 0.0221 | 已 PASS，属「大段中文」 |

按铁律 §3：4 个文件改前 `en/total` 均 `<= 0.05`，因此**未做任何整段改写**，
只清 `check` 报出的 14 行英文 offender；已中文段落一字未动（铁律 §8）。

### 逐文件改动量

**1. `03-impl/T7-goals-superpowers.md` —— 小改（2 行，1:1 行内替换）**

| 行 | 改前 | 改后 |
|---|---|---|
| 228 | `.goals/rustcode-migration-finalize/goal.md`（1 insertion / 1 deletion） | 同路径（**1 增 / 1 删**） |
| 229 | `.goals/rustcode-migration-finalize/summary.md`（2 insertions / 2 deletions） | 同路径（**2 增 / 2 删**） |

diffstat：`2 insertions / 2 deletions`，纯行内替换，行数闭合。
译文采用本仓库既有计数口径（参见 `STATUS.md:263`「4 增 15 删」、
`05-test-report.md:254`「283 增 / 104 删」），未引入新说法。

**2. `06-release.md` —— no-op（0 改动）**

首跑即 `en=0/200`，全文已中文，且含 YAML frontmatter（键名 `kind`/`id`/`from`/`to`/
`feature`/`status`/`decision`/`requires`/`files_owned`/`created` 与枚举值 `done`/
`proceed` 一律保留英文，见下节 AC-6）。无任何可清 offender，未落笔。

**3. `STATUS.md` —— 小改（6 行，1:1 行内替换）**

| 行 | 改前 | 改后 | 说明 |
|---|---|---|---|
| 63 | 路径清单续行，无中文 | 行尾补（**共 6 个文件**） | 与第 62 行「6 个文件」自洽 |
| 160 | `...`（tuix）、 | `...`（**tuix 侧**）、 | 标注来源组件 |
| 193 | `2059 / **5 failed**` | `2059 / **5 个失败**` | 门禁结果计数 |
| 194 | `115 / **1 failed**` | `115 / **1 个失败**` | 同上 |
| 195 | `99 / **1 failed**` | `99 / **1 个失败**` | 同上 |
| 265 | `**exit=0 / 0 error**` | `**exit=0 / 0 错误**` | 与第 264 行「0 差异」同构 |

diffstat：`6 insertions / 6 deletions`，纯行内替换，行数闭合。

**4. `05-test-report.md` —— 小改（4 行，1:1 行内替换）**

| 行 | 改前 | 改后 |
|---|---|---|
| 38 | `\| branch \|` | `\| 分支 \|` |
| 39 | `\| commit \|` | `\| 提交 \|` |
| 40 | `\| worktree \|` | `\| 工作树 \|` |
| 242 | 路径清单续行，无中文 | 行尾补（**共 6 个文件**） |

diffstat：`4 insertions / 4 deletions`，纯行内替换，行数闭合。
第 38-40 行三处为**表格标签散文**（非代码、非 CLI flag、非配置键），
值列 `dev` / `8e772dbf` / `**dirty**` 保持原样不动。

### 残留行清单与白名单理由

改动后仅 `03-impl/T7-goals-superpowers.md` 残留 2 行 en 行（`en=2/264=0.0076`）：

| 行 | 内容 | 白名单理由 |
|---|---|---|
| 370 | `\| `inspector-feedback-1.md:11` \| ``\| 1 \| G7: atomcode in crates/scripts/.github = 0 \| PASS \| `grep -rn "atomcode" crates/ scripts/ .github/` = 0 hits \|`` \|` | 该单元格整体是**双反引号 code span**，逐字引用 `inspector-feedback-1.md:11` 的原文。铁律 §1/§2.2 禁止翻译 inline code 且禁止改动反引号；且该表标题为「规则 A 保留项（未改动，列出以证完整性）」，改写即破坏「未改动」的举证效力。检测器把 code span 内的 `atomcode`（≥4 个 ASCII 字母）计入 en，属已知计数口径，非英文散文。 |
| 371 | 同上，`inspector-feedback-1.md:12` 的 G8 行 | 同上 |

其余 3 个文件 `en=0`，无残留。

**因已含中文而未被判为 offender、按铁律 §8 未改写的英文词**（如实登记，不做「顺手统一」，
以免触碰已中文段落）：`STATUS.md:198` 的 `1475 / **1 failed**`、
`STATUS.md:266` 的 `**5481 passed / 1 failed**`、`06-release.md:19` 的
`branch=dev commit=8e772dbf，worktree=**dirty**`。

### 四项实测结果

脚本判定（fail-closed，任一 AC 失败即 FAIL）：**受检 4，PASS 4，FAIL 0**。

| 文件 | AC-2 `en/total` | 阈值 0.05 |
|---|---|---|
| `03-impl/T7-goals-superpowers.md` | 2/264 = **0.0076** | PASS |
| `06-release.md` | 0/200 = **0.0000** | PASS |
| `STATUS.md` | 0/247 = **0.0000** | PASS |
| `05-test-report.md` | 0/181 = **0.0000** | PASS |

AC-4 / AC-7b / AC-32 另用独立脚本对照基线 `3ee655e3` 复算（inline code span 多重集、
fenced fence 行多重集、`](...)` 链接目标多重集、emoji 计数），四文件结果完全一致：

| 文件 | inline code 集合 | fenced 集合 | 链接目标集合 | emoji（基线/现状） |
|---|---|---|---|---|
| `03-impl/T7-goals-superpowers.md` | 相等 | 相等 | 相等 | 0 / 0 |
| `06-release.md` | 相等 | 相等 | 相等 | 0 / 0 |
| `STATUS.md` | 相等 | 相等 | 相等 | 0 / 0 |
| `05-test-report.md` | 相等 | 相等 | 相等 | 0 / 0 |

即 AC-4 PASS、AC-7b PASS、AC-32 PASS（emoji 未增加）。
行数闭合：`git diff --numstat` 为 2/2、4/4、6/6（`06-release.md` 无 diff），
全为 1:1 行内替换，未增删空行、未改动段落结构（铁律 §5.4/§5.7）。

### AC-6 · YAML frontmatter 结论

4 个文件中 `T7-goals-superpowers.md`、`06-release.md`、`05-test-report.md` 含 frontmatter，
`STATUS.md` 无 frontmatter（首行即为 `#` 标题）。

- 键名 `kind` / `id` / `from` / `to` / `feature` / `status` / `decision` / `requires` /
  `files_owned` / `created` **全部保留英文**，零改动。
- 枚举值 `status: done`、`decision: proceed` **保留英文**，零改动。
- 三份 frontmatter **均无 `description` 键**，因此不存在需汉化的 `description` 值，
  本次无需逐条列出（铁律 §7 的报告项为空集）。
- `feature: 2026-09-02-g1-fmt-gate`、`files_owned` 路径、`created: 2026-09-02` 均为路径/日期，保留原文。

### 铁律遵守与边界

- 未翻译任何 fenced code block 与 inline code；未把任何原本不在反引号内的词改成反引号（规避 §5.1）。
- 未改链接目标与锚点、未改文件名与标题层级、未新增或清理 emoji。
- 保留了全部 commit hash（`8e772dbf`）、`cargo` 子命令与 flag
  （`cargo check -j 1 --workspace --all-targets`、`cargo fmt --check` 等）、文件路径与 crate 名。
- 只改 `files_owned` 内 4 个文件；**未**触碰 `.codebuddy/artifacts/2026-09-07-zh-docs-webui/`
  下除本报告 `03-impl/zh-check-D-29.md` 之外的任何文件；`03-impl/T6-docs-atomcode.md`、
  `03-impl/T8-artifacts.md`、`HANDOFF-codingplan-legacy.md` 留给 D-30，未动。
- 未执行 `cargo` / `npm` 构建与测试，未改源码，未提交、未打标签。
