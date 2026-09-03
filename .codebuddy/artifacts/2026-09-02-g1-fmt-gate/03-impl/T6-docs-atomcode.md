---
kind: release
id: RELEASE-001
from: doc-writer
to: [project-manager]
feature: 2026-09-02-g1-fmt-gate
status: done
decision: proceed
requires: []
files_owned:
  - README.md
  - README.zh-CN.md
  - docs/features.md
  - docs/platform-neutralization.md
created: 2026-09-02
---

# T6 用户可见文档 `atomcode` 残留清理 — 交付说明

> `requires: []` 说明：本任务为纯文档改动，零 `.rs` / `.toml` / 配置变更，
> 不触发 G1-G3；且 `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/` 下无
> `05-test-report.md`，故不声明对测试件的前置依赖。

---

## 1. 行为变化

### 1.1 用户与调用方可感知的变化

| 变化 | 影响面 | 前后对比 |
|------|--------|----------|
| 4 个用户可见文档不再出现上游仓库 slug `atomgit_atomcode/atomcode` | README.md、README.zh-CN.md、docs/features.md、docs/platform-neutralization.md | 前：fork 声明直接点名上游 slug；后：表述为「上游项目 / 上游项目的二次开发 fork」 |
| docs/features.md 不再把 `atomcode-*` 作为活名称 | docs/features.md:12 | 前：「全量 `atomcode-*` → `rustcode-*`」；后：「产品标识统一为 `rustcode-*`」，无旧名 |
| 归属信息落点不变 | README.md:58-60、README.zh-CN.md:46 | MIT 原文与版权（(c) 2026 Yubang Xu）仍链 `docs/ORIGINAL_LICENSE.md`，完整归属仍链 `docs/UPSTREAM_CREDITS.md` |
| 质量门禁 G7 / G8 定义文本 | docs/features.md:113-114、docs/platform-neutralization.md:204-206 | 原样未改，grep 模式继续有效 |

### 1.2 无变化项

- 无代码、无配置、无构建/发布行为变化。
- 无对外协议、无版本号、无发布配置改动。
- 无已退役接口被重新引入或删除，`atomcode-*` 未被写成仍可使用的能力。

### 1.3 迁移步骤

无。纯文档措辞调整，读者与调用方无需任何动作。

---

## 2. 风险

| 风险类别 | 评估 | 依据 |
|----------|------|------|
| 不兼容 | 无 | 仅 Markdown 正文措辞，无 API / CLI / 配置项变化 |
| 性能 | 无 | 不涉及代码路径 |
| 数据影响 | 无 | 不触碰 `~/.rustcode` 配置、会话或凭据 |
| G1 / G2 / G3 | 不受影响 | 零 `.rs` 改动；`cargo fmt` 不扫描 `.md` |
| G6 遥测门禁 | 不受影响 | G6 为 `--include=*.rs --include=*.toml` |
| G7 / G8 | **不受影响（结构性免疫）** | G7 扫描 `crates/ scripts/ .github/`，G8 扫描 `docs/architecture.md`；本次改动的 4 个文件**均不在 G7/G8 的 grep 范围内**，故结果不可能被改变 |
| 合规（MIT 归属） | 已缓解，但需编排者确认（见第 5 节 U1） | `LICENSE`、`docs/UPSTREAM_*.md`、`docs/THIRD_PARTY_NOTICES.md` 未触碰；两个 README 的归属链接完整保留 |
| 回滚代价 | 极低 | 纯文本；回滚不牵动构建产物、配置或数据 |

**补充说明**：`.github/workflows/ci.yml` 仅实现 G1/G2/G3（fmt / clippy / test）三个 job，G4-G8 未纳入 CI。因此本次改动不会被任何 CI job 拒绝，但也不会被 CI 自动验证。

---

## 3. 验证结果

### 验证 1：`grep -in atomcode <file>` 剩余项逐条标注

实测命令：对 4 个文件分别执行 `grep -in atomcode <file>`（大小写不敏感）。

**README.md** — 0 命中（无剩余）
```
(empty)
```

**README.zh-CN.md** — 0 命中（无剩余）
```
(empty)
```

**docs/features.md** — 2 处
| 行号 | 内容 | 分类 |
|------|------|------|
| 113 | `G7  crates/scripts/.github 无 atomcode 残留` | 规则 B 门禁定义（必留） |
| 114 | `G8  docs/architecture.md 无 atomcode 残留` | 规则 B 门禁定义（必留） |

**docs/platform-neutralization.md** — 2 处（3 行，含 G7 续行）
| 行号 | 内容 | 分类 |
|------|------|------|
| 204 | `G7  crates/ scripts/ .github/ 无 atomcode 残留(atomgit feature / 旧前缀兼容 /` | 规则 B 门禁定义（必留） |
| 205 | `    fork 发行主页三类除外)` | 规则 B 门禁定义续行（必留） |
| 206 | `G8  docs/architecture.md 无 atomcode 残留` | 规则 B 门禁定义（必留） |

**汇总**：4 个文件共剩余 **4 处 grep 条目 / 5 行**，**全部为规则 B 门禁定义（必留）**；
**「遗漏未清」= 0**。改动前基线为 8 处（README.md 1、README.zh-CN.md 1、features.md 3、platform-neutralization.md 3），清理掉 4 处规则 C/D 目标。

### 验证 2：规则 B 的 5 行门禁定义原样未改

| 位置 | 结论 |
|------|------|
| docs/features.md:113 | 原样 |
| docs/features.md:114 | 原样 |
| docs/platform-neutralization.md:204 | 原样 |
| docs/platform-neutralization.md:205 | 原样 |
| docs/platform-neutralization.md:206 | 原样 |

判定依据：本次共 5 次 Edit，落点分别为 `README.md:48-49`、`README.zh-CN.md:46`、
`docs/features.md:10`、`docs/features.md:12-13`、`docs/platform-neutralization.md:13`，
均不落在上述 5 行。改动后逐行 Read 复核，与任务书给出的原文逐字符一致。

### 验证 3：`git diff --stat` 只应含这 4 个文件

**未能实测。** 本会话未提供任何 shell / Bash 工具（`Bash` 工具未注册），
无法执行 `git diff --stat`。此项为**验证缺口，非已完成**。

可提供的等效证据（来自本次会话的工具调用记录）：

- Write 调用：**0 次**对仓库既有文件；仅新建本交付说明。
- Edit 调用：**5 次**，目标文件全部落在授权 4 文件内：
  - `README.md`（1 次）
  - `README.zh-CN.md`（1 次）
  - `docs/features.md`（2 次）
  - `docs/platform-neutralization.md`（1 次）
- 明确**未**触碰：`AGENTS.md`、`LICENSE`、`docs/ORIGINAL_LICENSE.md`、
  `docs/UPSTREAM_CREDITS.md`、`docs/UPSTREAM_RUSTCODE_LICENSE.md`、
  `docs/THIRD_PARTY_NOTICES.md`、`docs/REFACTOR_DESIGN_PHASE1.md`、
  `docs/architecture.md`、`.goals/`、`.codebuddy/` 既有文件、`crates/` 下任何文件。

请编排者补跑：
```bash
git diff --stat
```
预期输出恰好 4 个文件（3 insertions / 3 deletions 量级，features.md 为 2 改）。

### 验证 4：Markdown 结构未被破坏

| 检查项 | 结论 |
|--------|------|
| `docs/features.md` 代码围栏 ```text（106 行开 / 115 行闭） | 成对完整，G1-G8 块未受影响 |
| `docs/platform-neutralization.md` 代码围栏 ```text（197 / 207） | 成对完整 |
| `docs/features.md` 有序列表 1.-4. | 编号连续未变；第 1 项续行缩进 3 空格，与第 2/3/4 项续行风格一致 |
| `README.md` blockquote（48-60 行） | 全部行以 `>` 前缀，块未被截断 |
| `README.zh-CN.md:46` blockquote | 单行完整 |
| 链接语法 | `[docs/ORIGINAL_LICENSE.md](docs/ORIGINAL_LICENSE.md)` 与 `[docs/UPSTREAM_CREDITS.md](docs/UPSTREAM_CREDITS.md)` 在 README.md:59-60 与 README.zh-CN.md:46 均完整保留（已 Read 复核） |
| 新增围栏 / 列表 / 标题 | 0；5 处改动全部为行内措辞替换 |
| 新增 Unicode emoji | 0；替换文本仅含 ASCII 与 CJK 文字，状态标签维持 ASCII |

**未做**：Markdown 渲染验证（本会话无渲染工具），仅逐处 Read 人工核对。

---

## 4. 已知未验证范围

| # | 未覆盖项 | 建议负责人 |
|---|----------|------------|
| N1 | `git diff --stat` 未实测（会话无 shell 工具） | 编排者（一条只读命令即可补齐） |
| N2 | 未执行任何构建 / 测试 / 门禁命令（按硬性约束 2 禁止；本会话亦无 shell） | 编排者在合并前补跑 G1-G8 |
| N3 | Markdown 未做渲染验证 | 文档维护者，随站点构建观察 |
| N4 | 全仓其余 87 处 `atomcode`（92 - 本次清理 4 - 本次保留 1 处重复计数口径）未做任何改动，也未逐一核实分类是否合理 | 编排者另开任务；本次严格限于授权 4 文件 |
| N5 | 未验证 `site/` 文档站或其他文档是否存在指向本 4 文件已删字符串的交叉引用（如站内搜索索引 `search-index.{en,zh}.json` 缓存了旧正文） | 站点维护者；若搜索索引由 `node build-search-index.mjs` 从 md 重生成则需重跑 |
| N6 | 未验证 G7/G8 的实际执行脚本是否真以 `docs/features.md:113-114` / `docs/platform-neutralization.md:204-206` 的文本为准绳（`.github/workflows/ci.yml` 只实现 G1-G3，G7/G8 无 CI job） | 编排者 / CI 维护者 |

---

## 5. 不确定项清单（上报，未自行决定）

### U1 [高] AGENTS.md 两处过程记录已过时 —— 需编排者处置

`AGENTS.md` 明确记录过「刻意不改这些文件里的 slug」这一**相反决策**：

- `AGENTS.md:282`
  > 「**未改/刻意保留**：README 与 docs 里 `atomgit_atomcode/atomcode` 仅出现在
  > fork 归属声明与 ORIGINAL_LICENSE/UPSTREAM_CREDITS（合法历史出处，G7/G8 逐案豁免）」
- `AGENTS.md:539`
  > 「刻意**未改** README 与 `docs/{UPSTREAM_RUSTCODE_LICENSE,UPSTREAM_CREDITS,
  > THIRD_PARTY_NOTICES,platform-neutralization,features}.md` 中的
  > `atomgit_atomcode/atomcode`——那是 MIT 归属声明,删改有合规风险。」

本次任务书要求我清理其中 4 个文件（README.md、README.zh-CN.md、
docs/features.md、docs/platform-neutralization.md），与上述记录**直接冲突**。

我的处置：**按任务书执行清理，但不动 `AGENTS.md`**（该文件在 `files_owned` 之外的
禁止改动清单中）。因此上述两行现在是**与实现不符的过时表述**，属于
「文档与实现冲突」情形，按职责边界**上报，不自行改写他人的过程记录**。

缓解说明（不构成合规损失）：`LICENSE` 与 `docs/UPSTREAM_*.md`、
`docs/THIRD_PARTY_NOTICES.md` 中的 MIT 归属原文**未触碰**；两个 README 的 fork 声明
**仍链接** `docs/ORIGINAL_LICENSE.md`（含 MIT 原文与 (c) 2026 Yubang Xu）与
`docs/UPSTREAM_CREDITS.md`（含上游 slug 完整归属）。归属信息未丢失，只是不再出现在
README 正文的 inline code 里。

**建议**：由编排者决定是否在后续任务中同步修订 `AGENTS.md:282` / `:539`，
或由决策者确认「内联 slug 移除 + 链接保留」这一折中是否为最终口径。

### U2 [中] docs/features.md 与 docs/platform-neutralization.md 删除 slug 后无归属链接

规则 C 总述要求「保留指向归属文档的链接」，但这两处**原本就没有**此类链接
（其文档头引用区只指向 `AGENTS.md` / `CONTEXT.md` / `docs/architecture.md` /
`docs/platform-neutralization.md`），即无链接可「保留」。

我按分项说明（第 3、4 项只给出替换文本、未提链接）做了**最小替换**，
未擅自新增链接，避免越界添加内容。

**建议**：若编排者希望这两个内部文档也带归属落点，可在其后补一句
「归属见 `docs/UPSTREAM_CREDITS.md`」。这属于新增内容，需明确指示后我再执行。

### U3 [低] 两个 README 的 fork 声明第 (1) 项仍保留「将产品重命名为 `rustcode`」

`README.md:49`（"renames the product to `rustcode`"）与 `README.zh-CN.md:46`
（「将产品重命名为 `rustcode`」）继续描述「重命名」这一相对上游的差异。

判定为**保留**：这是 fork notice 中与上游的 diff 描述，且不点名旧名，
不构成「把 `atomcode-*` 当作活名称」；规则 D 只点名了 `docs/features.md:12`。

**建议**：若编排者认为 README 该项也应改为纯现状表述，请另行指示。

### U4 [低] docs/features.md:12 加粗标题仍为「**产品重命名**」

规则 D 只要求改条目正文。该标题与第 2/3/4 项（**零遥测** / **平台中立** /
**默认中文**）同属「核心诉求」目标名，改动会破坏并列结构，故保留。

---

## 6. 逐处改动前后对照

### C1. README.md:48-49（规则 C-1）

```diff
  > **Fork notice.** This repository is a secondary-development
- > fork of `atomgit_atomcode/atomcode`. Relative to upstream it (1) renames the
+ > upstream project. Relative to upstream it (1) renames the
  > product to `rustcode` (crates, binaries, config dir `~/.rustcode`, `RUSTCODE_*`
```

后续行 50-60 未改，其中 58-60 行的
`[docs/ORIGINAL_LICENSE.md](docs/ORIGINAL_LICENSE.md)` 与
`[docs/UPSTREAM_CREDITS.md](docs/UPSTREAM_CREDITS.md)` 链接**保留**。

### C2. README.zh-CN.md:46（规则 C-2）

```diff
- > **Fork 声明。** 本仓库是 `atomgit_atomcode/atomcode` 的二次开发 fork。相对上游：(1) ...
+ > **Fork 声明。** 本仓库是上游项目的二次开发 fork。相对上游：(1) ...
```

同行尾部的
`[docs/ORIGINAL_LICENSE.md](docs/ORIGINAL_LICENSE.md)` 与
`[docs/UPSTREAM_CREDITS.md](docs/UPSTREAM_CREDITS.md)` 链接**保留**。

### C3. docs/features.md:10（规则 C-3）

```diff
- RustCode 是 `atomgit_atomcode/atomcode` 的二次开发 fork,核心诉求:
+ RustCode 是上游项目的二次开发 fork,核心诉求:
```

### C4. docs/platform-neutralization.md:13（规则 C-4）

```diff
- RustCode 是上游 `atomgit_atomcode/atomcode` 的二次开发 fork。中立化的目标是:
+ RustCode 是上游项目的二次开发 fork。中立化的目标是:
```

### D1. docs/features.md:12-13（规则 D）

```diff
  1. **产品重命名** —
-    全量 `atomcode-*` → `rustcode-*`,配置目录 `~/.rustcode`,
-    环境变量前缀 `RUSTCODE_*`。
+    产品标识统一为 `rustcode-*`(crate、二进制、命令名),配置
+    目录 `~/.rustcode`,环境变量前缀 `RUSTCODE_*`。
```

（上式为便于阅读的展开排版；实际落盘保持原缩进与换行风格，
续行缩进 3 空格，与第 2/3/4 项一致。）

未采用「（历史名 `atomcode-*` 已全量迁移）」旁注：任务书「默认倾向于不出现该词」。

### 规则 B 保留项（未改动，列出以证完整性）

| 位置 | 文本 |
|------|------|
| docs/features.md:113-114 | `G7  crates/scripts/.github 无 atomcode 残留` / `G8  docs/architecture.md 无 atomcode 残留` |
| docs/platform-neutralization.md:204-206 | `G7  crates/ scripts/ .github/ 无 atomcode 残留(atomgit feature / 旧前缀兼容 /` / `    fork 发行主页三类除外)` / `G8  docs/architecture.md 无 atomcode 残留` |

### 规则 A 保留项（未触碰，列出以证未越界）

`LICENSE`、`docs/ORIGINAL_LICENSE.md`、`docs/UPSTREAM_CREDITS.md`、
`docs/UPSTREAM_RUSTCODE_LICENSE.md`、`docs/THIRD_PARTY_NOTICES.md`
—— 零改动。

---

## 7. 文档更新清单

| 文件路径 | 变更类型 | 摘要 |
|----------|----------|------|
| README.md | 修订 | Fork notice 去掉上游仓库 slug，改为「fork of an upstream project」；归属链接保留 |
| README.zh-CN.md | 修订 | Fork 声明去掉上游仓库 slug，改为「上游项目的二次开发 fork」；归属链接保留；与英文版语义对齐 |
| docs/features.md | 修订 | 产品定位段去掉上游 slug；「产品重命名」条目由历史重命名叙述改为现状表述，不再出现 `atomcode-*` |
| docs/platform-neutralization.md | 修订 | 背景段去掉上游 slug，改为「上游项目的二次开发 fork」 |
| .codebuddy/artifacts/2026-09-02-g1-fmt-gate/03-impl/T6-docs-atomcode.md | 新增 | 本交付说明 |

无删除项，`CHANGELOG` 在本仓库中不存在。

---

## 8. 回滚方案

**判定时机**（任一出现即回滚）：

1. 合规审计要求用户可见文档必须内联保留上游 slug 出处（对应不确定项 U1）。
2. 编排者认定 `AGENTS.md:539` 记录的原决策（「删改有合规风险，刻意未改」）优先级更高。
3. 发现 `site/` 搜索索引或其他文档站产物依赖被删字符串且无法重生成（对应 N5）。

**回滚步骤**（需编排者执行；本人无 git 写权限，按硬性约束 3 不参与）：

```bash
# 1) 先存补丁，保留可重放能力
git diff -- README.md README.zh-CN.md docs/features.md docs/platform-neutralization.md \
  > /tmp/t6-docs-atomcode.patch

# 2) 回滚
git checkout -- README.md README.zh-CN.md docs/features.md docs/platform-neutralization.md

# 3) 需要恢复时
git apply /tmp/t6-docs-atomcode.patch
```

**回滚代价**：极低。纯文本改动，无代码 / 配置 / 数据 / 构建产物依赖；
`crates/`、`scripts/`、`.github/`、`docs/architecture.md` 均未改动，
回滚不改变 G1-G8 任何一项的结果。

---

## 9. 术语与命名一致性检查结论

| 检查项 | 结论 |
|--------|------|
| 上游 slug `atomgit_atomcode/atomcode` | 授权 4 文件中**已清零**。仓库内仍存在于：`docs/UPSTREAM_RUSTCODE_LICENSE.md:4`、`docs/UPSTREAM_CREDITS.md:6`、`docs/THIRD_PARTY_NOTICES.md:8`（规则 A，未触碰）；`docs/REFACTOR_DESIGN_PHASE1.md:357`（授权范围外，未触碰）；`AGENTS.md:282` / `:539`（授权范围外，且已过时 —— 见 U1） |
| `atomgit`（不含 `atomcode`） | G7 三类豁免项，本次未触碰。`docs/platform-neutralization.md:51-54` 的 `atomgit` Cargo feature 描述与 `:219-220` 的刻意保留说明**正确保留**，与 G7 豁免口径一致 |
| 产品标识命名 | 统一为 `rustcode-*` / `~/.rustcode` / `RUSTCODE_*`；`docs/features.md:12` 已按此校准，未引入任何新命名 |
| 中英 README 语义一致 | 一致。en「fork of an upstream project」=== zh「上游项目的二次开发 fork」；两版 fork 声明的 4 条差异项与归属链接均对齐 |
| 退役组件表述 | 未将 `atomcode-*` 写成仍可使用的能力；`docs/features.md:12` 现为现状表述 |
| ASCII 化 | 替换文本零 emoji；状态标签（`[DONE]` / `[CHECK]` / `[WARN]`）维持 ASCII |
| 架构边界表述 | 本次改动不涉及 `AGENTS.md` 的单一状态所有权 / 依赖方向 / core 退役结论，无需校准 |

---

## 10. 结论

`status: done`，`decision: proceed`。

授权范围内的清理**已全部完成**，无遗漏未清项；规则 B 的 5 行门禁定义原样保留；
规则 A 的合规归属文件零改动。

**唯一未达成的验证项是 `git diff --stat`（N1）**，原因是本会话无 shell 工具，
已如实标注为验证缺口而非完成项；请编排者用一条只读命令补齐。

需编排者决策的上报项：**U1（AGENTS.md 过时记录，高优先级）**，其余 U2-U4 为低优先级口径确认。
