# zh-check 报告

- 基线 (--base): `3ee655e3`
- 阈值 (--en-ratio): `0.05`
- 判定: AC-2 / AC-4 / AC-6 / AC-7b / AC-32，任一失败即 FAIL（fail-closed）

## 汇总

- 受检文件: 11
- PASS: 11
- FAIL: 0

## PASS `.codebuddy/artifacts/2026-09-03-git-wrapup/01-plan.md`

- en=0/194 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-03-git-wrapup/02-tasks.md`

- en=0/122 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-03-banner-release/STATUS.md`

- en=0/235 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-03-git-wrapup/STATUS.md`

- en=0/139 ratio=0.0000
- R5 跳过的行 (共 1 行):
  - `.codebuddy/artifacts/2026-09-03-git-wrapup/STATUS.md:8`: `    - '?? .codebuddy/artifacts/2026-09-03-git-wrapup/'（本轮交接件）`

## PASS `.codebuddy/artifacts/2026-09-03-git-wrapup/03-impl/GW-02.md`

- en=0/66 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-03-git-wrapup/03-impl/GW-03.md`

- en=0/80 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-03-banner-release/03-impl/T2.md`

- en=0/87 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-03-banner-release/03-impl/T1.md`

- en=0/58 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-03-residual-two-items/STATUS.md`

- en=0/64 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-03-doc-consistency/STATUS.md`

- en=0/50 ratio=0.0000

## PASS `.codebuddy/artifacts/2026-09-02-strip-atomcode/STATUS.md`

- en=0/46 ratio=0.0000

## 附录 · 被 R5 跳过的行（供抽检）

共 1 行。

- `.codebuddy/artifacts/2026-09-03-git-wrapup/STATUS.md:8`: `    - '?? .codebuddy/artifacts/2026-09-03-git-wrapup/'（本轮交接件）`
## 手写小结（D-31，追加于最后一次自检之后）

自检命令（与本报告同源，基线 `3ee655e3`）：

```bash
python3 scripts/check-zh-docs.py check --base 3ee655e3 \
  --report .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-31.md \
  --files <本批 11 个文件>
```

### 1. 逐文件改动量（含 no-op 判定）

本批 11 个文件基测即已大段中文，按铁律 §3 采取「只清 offender、不改写已中文段落」的口径：
全部改动均为**行内替换**，`git diff --numstat` 为 `32 insertions(+) / 32 deletions(-)`，
**总行数不变**（未增删空行、未改变段落结构）。

| 文件 | 基测 en/total | 判定 | 实改行数 | 改动位置与内容 |
|---|---|---|---|---|
| `.codebuddy/artifacts/2026-09-03-git-wrapup/01-plan.md` | 1/194 = 0.0052 | no-op 候选（已 ≤0.05），仍清 1 处 offender | 1 | `:175` `exit=0。` → `退出码=0。` |
| `.codebuddy/artifacts/2026-09-03-git-wrapup/02-tasks.md` | 2/122 = 0.0164 | no-op 候选，仍清 2 处 offender | 2 | `:71` files_owned 单元格补「（协作方案文档）」；`:199` `C5(merge)` → `C5（合并提交）` |
| `.codebuddy/artifacts/2026-09-03-banner-release/STATUS.md` | 13/235 = 0.0553 | **FAIL，必须改** | 13 | `:80` `ELF 64-bit LSB pie, x86-64, dynamically linked` → `ELF 64 位 LSB pie，x86-64，动态链接`；`:83`（default-members）→（构建默认成员）、`exit=0` → `退出码=0`；`:142-144` 三项工具名后补「（已安装）」；`:161` `dynamically linked` → `动态链接`；`:219` `exit=0`×2 → `退出码=0`；`:287` 脚本报错文案整句汉化为「现在构建出的 release 会对 /webui 返回 404」；`:301-304` 平台标签补「产物」（原生 gnu / musl / aarch64 / windows）；`:312` `native gnu` → `原生 gnu` |
| `.codebuddy/artifacts/2026-09-03-git-wrapup/STATUS.md` | 7/139 = 0.0504 | **FAIL，必须改** | 7 | `:86-90` 子代理通道结果列 `No result found` → `未返回结果`、`idle timeout` → `空闲超时`；`:146` 补「合并提交」；`:158` `(8 ignored)` → `（8 项忽略）` |
| `.codebuddy/artifacts/2026-09-03-git-wrapup/03-impl/GW-02.md` | 0/66 = 0.0000 | **no-op，未改动** | 0 | — |
| `.codebuddy/artifacts/2026-09-03-git-wrapup/03-impl/GW-03.md` | 1/80 = 0.0125 | no-op 候选，仍清 1 处 offender | 2 | `:33-34`（跨行一句）`"No result found"` → `「未返回结果」`、`idle timeout` → `空闲超时`。保留原折行位置，未合并为一行 |
| `.codebuddy/artifacts/2026-09-03-banner-release/03-impl/T2.md` | 4/87 = 0.0460 | no-op 候选（贴近阈值），仍清 4 处 offender | 4 | `:36`、`:40` `file` 输出描述汉化（`dynamically linked, stripped` → `动态链接，已剥离符号`）；`:43` `stripped to external PDB` → `已剥离至外部 PDB`；`:50` 说明列 `—` → `已启用` |
| `.codebuddy/artifacts/2026-09-03-banner-release/03-impl/T1.md` | 0/58 = 0.0000 | **no-op，未改动** | 0 | — |
| `.codebuddy/artifacts/2026-09-03-residual-two-items/STATUS.md` | 0/64 = 0.0000 | **no-op，未改动** | 0 | — |
| `.codebuddy/artifacts/2026-09-03-doc-consistency/STATUS.md` | 3/50 = 0.0600 | **FAIL，必须改** | 3 | `:39` 扫描域 D4 补「（Agent 定义 + 脚本 + 工作流）」；`:50` 补结构助词「内」；`:61` `SYNTAX_OK` 前补中文「语法检查通过（…）」 |
| `.codebuddy/artifacts/2026-09-02-strip-atomcode/STATUS.md` | 0/46 = 0.0000 | **no-op，未改动** | 0 | — |

合计：改动 7 个文件 32 行；4 个文件未改动（GW-02 / T1 / residual-two-items / strip-atomcode，基测 en 均为 0）。

### 2. 残留行清单与白名单理由

**残留 en 行：0 行。** 11 个文件的 AC-3 残留清单均为空（见本报告自动段，无「AC-3 残留行清单」条目）。

仍保留的英文文本（按铁律 §2/§3 属保留原文，**不计入 en**，故不构成残留）：

- **inline code 内的英文**：`No result found`（`GW-03.md:25/26`，位于反引号内，按铁律 §2 一个字符未动）、`statically linked`、`SMOKE OK`、`SYNTAX_OK`（括注形态）、`--no-ff` 等在反引号内的命令与工具返回字面量。
- **fenced code block 内全部内容**：`01-plan.md` 的 bash 命令块与分支拓扑块、`GW-03.md` 的 rust 改动前后对比块与 `cargo test` 输出块、`T2.md` 的 ACP 冒烟输出块与 `/health` JSON 块 —— 逐字符未动。
- **commit hash 与历史包名**：`a81fb69f`、`2e5baa33`、`f3489055`、`73efc258`、`7cdc1294`、`783d48e4` 等，以及历史包名 `atomcode` / `atomcode-daemon*` / `-p rustcode-cli` 在引用语境中的原样保留。
- **文件路径、命令、CLI flag、环境变量、配置键**：`target/release/rustcode`、`/usr/bin/musl-gcc`、`cargo build --release -j 2`、`RUSTCODE_HOME`、`--version` / `--help` 等。
- **ASCII 状态标签**：`[WARN]` / `[ERROR]` / `[DONE]` / `[IN PROGRESS]` / `done` / `ready` / `in_progress` / `pending` / `blocked` 等结构性标签与状态枚举值（含 frontmatter 的 `status` / `decision` 枚举值），按铁律 §7 保留英文。
- **工具返回字面量括注**：`doc-consistency/STATUS.md:61` 写作「语法检查通过（SYNTAX_OK）」—— 中文在前、原样字面量作括注以便溯源，这是唯一的括注形态，其余均为纯中文替换（无中英并列段落）。
- **R5 跳过的行 1 条**：`.codebuddy/artifacts/2026-09-03-git-wrapup/STATUS.md:8`（4 空格缩进的列表续行），脚本按 R3 规则跳过、不进 total；该行本身已是中文，未改动。

### 3. 四项实测结果（本次自检实际输出）

| 判定项 | 结果 | 实测依据 |
|---|---|---|
| **AC-2**（`en/total <= 0.05`） | **PASS 11/11** | 11 个文件 `en=0`，ratio 全为 `0.0000`（改动前 3 个 FAIL：0.0553 / 0.0504 / 0.0600，现全部归零） |
| **AC-4**（inline code + fenced code 多重集与基线相等） | **PASS 11/11** | 自动段无任何 `AC-4 FAIL` 条目，即 11 个文件 `old_spans == new_spans`；未新增/删除任何反引号，未触碰任何代码块 |
| **AC-7b**（`](...)` 链接目标多重集相等） | **PASS 11/11** | 自动段无任何 `AC-7b FAIL` 条目；本批改动未触碰任何链接目标 |
| **AC-32**（Emoji 命中数不增加） | **PASS 11/11** | 自动段无任何 `AC-32 FAIL` 条目；`T2.md:50` 的 `✓` 数量保持 3 个未变，未新增任何 emoji |

附加核对：

- **AC-6**（frontmatter 键名集合相等）PASS：本批 6 个含 frontmatter 的文件（`01-plan.md`、`02-tasks.md`、`03-impl/GW-02.md`、`03-impl/GW-03.md`、`03-impl/T2.md`、`03-impl/T1.md`）键名集合与基线一致，键名一律保留英文，`status` / `decision` 枚举值保留英文。
- **AC-6 description 变更：无。** 经 `grep -n "^description:"` 核对，上述 6 个文件的 frontmatter **均不含** `description` 键，故本批无 description 汉化项需要逐条列出（铁律 §7 要求列出项为空）。
- 行数口径：7 个改动文件均为 `+N/-N` 相等（合计 +32/-32），未增删空行，未改变标题层级与文件名。

### 4. 本批边界确认

- 只改 `files_owned` 内 11 个文件；未触碰 `.codebuddy/artifacts/2026-09-07-zh-docs-webui/` 下除本报告外的任何文件，未触碰 `.codebuddy/artifacts/2026-09-02-*` 下由 D-28/D-29/D-30 并行处理的其它文件。
- 未执行 `cargo` / `npm` 构建与测试，未改源码，未提交、未打标签。
