---
kind: design-addendum
id: DESIGN-001-A1
from: solution-architect
to: [project-manager, code-implementer, doc-writer]
feature: 2026-09-07-zh-docs-webui
status: approved
decision: proceed
requires: [DESIGN-001, DECISIONS-Q1-Q5, TASKS-001]
supersedes: 01-design.md 中且仅中「AC-4 白名单常量 AC4_ALLOWED_REMOVED」与「AC-8 作用域」两处
files_owned: []
created: 2026-09-08
---

# 01-design-addendum · AC-4 / AC-8 判据精确化补遗

本补遗是 `01-design.md` 的追加契约，**只**取代其中两点：AC-4 的 span 白名单判据、AC-8 的 grep 作用域。
`01-design.md` 与 `02-tasks.md` 正文不回改（冻结件保持稳定）；两者与本补遗冲突处以本补遗为准，
偏差由 T-07 / T-10 在报告中显式登记。

`code-implementer` 依本补遗改 `scripts/check-zh-docs.py`（任务 T-09）。
本补遗不修改任何源码、任何脚本、任何业务文档正文。

---

## 0. 三条裁决（一句话结论）

1. **AC-4**：判据从「added 必须为空 + removed ⊆ {`README.zh-CN.md`}」升级为「**removed 必须是 D1 类（span 字面含已删文件名 `README.zh-CN.md`）且 added 必须是其最小剔除式等价串（冻结算子输出）或冻结登记的 D2 项**」；据此 `AGENTS.md` 与 `docs/superpowers/plans/2026-05-29-webui.md` **直接放行**，`docs/phase1-refactor-design.md` 与 `README.md` **必须文档侧回退**（前者改回最小剔除式，后者恢复被整段删掉的手工构建步骤代码块）。
2. **AC-8**：作用域改为「正式域（`git ls-files` 候选集 **减去**唯一豁免前缀 `.codebuddy/artifacts/`，要求 0 命中）+ 历史域（豁免前缀内，反向断言**必须仍有命中**，即历史不可篡改）+ 既有全扩展名兜底域（`site/ .github/ docs/ extensions/`，0 命中，不变）」。
3. **编号**：`T-06` 归 `02-tasks.md`（hostscan），实现期新增任务从 `T-08` 顺序分配 —— `T-08` = 门禁脚本修正（原 `03-impl/T-06-gate-script.md`）、`T-09` = 本补遗的脚本实施、`T-10` = 补报告 + `06-release.md`、`T-11` = 文档侧 AC-4 回退；历史文件名一律不重命名，引用时首现给别名。

---

## 1. 现状（实测，逐条可复核）

| 对象 | 位置 | 现状 |
|---|---|---|
| AC-4 白名单常量 | `scripts/check-zh-docs.py:41-51` | `AC4_ALLOWED_REMOVED = frozenset({"README.zh-CN.md"})`；注释写明「绝不允许新增」 |
| AC-4 判定 | `scripts/check-zh-docs.py:480-507` | `if added or illegal_removed: FAIL`；`else` 记 note |
| span 提取 | `scripts/check-zh-docs.py:357-369` | `code_spans` = inline code 去反引号 **+ fenced code 块内整行**（整行作为 span，含块内注释与英文） |
| 完整差集 | `scripts/check-zh-docs.py:412-424` | `ac4_span_diff`（不截断）；`split_removed` 在 `427-431` |
| AC-8 | `scripts/check-zh-docs.py:1002-1046` | 第一次 `git grep -n -F README.zh-CN -- *.md *.html *.json *.ts *.kt *.yml *.toml` 全仓；第二次扫 `site/ .github/ docs/ extensions/`（全扩展名） |
| md 分母 | `scripts/check-zh-docs.py:124-126` | `list_md_files` = `git ls-files -- '*.md'`，**只收已跟踪文件**；本 feature 的 `.codebuddy/artifacts/2026-09-07-zh-docs-webui/**` 当前 untracked，故不参与 AC-4 |
| 汉化铁律 | `03-impl/HANDOFF-汉化铁律.md:13-15` | 铁律 1「不翻译 fenced code block，含注释与字符串，一个字符都不动」；铁律 3「保留原文：文件路径…命令」 |
| 裁决 a | `00-decisions.md:58-65`（Q4）、`STATUS.md:12` | 汉化 `README.md`、删除 `README.zh-CN.md`；中文单语，英文原文由 git 历史提供 |
| 裁决 b | `01-design.md §4.4`（K7/K8，`:295`/`:297`）、`03-impl/T-05.md:61-62,100-101` | README / i18n 构建指引指向 `./scripts/build-webui.sh`；**冻结文案同时保留手工等价步骤** `cd webui && npm ci && npm run build` + `cargo clean -p rustcode-daemon` |

失败面（2026-09-08 实测 `gate --base 3ee655e3`）：AC-4 FAIL 4 文件、AC-8 FAIL 39 处。
AC-8 的 39 处**全部**位于 `.codebuddy/artifacts/` 前缀下的历史交接件（13 个已跟踪文件，来自 `2026-09-02-*` / `2026-09-03-*` 两个旧 feature）；`docs/`、`extensions/`、`site/`、`.github/` 第二次 grep 已 0 命中。

---

## 2. AC-4 判据（v2，冻结）

### 2.1 判据文本（实现者逐字照此判定，不得增删条件）

对单个受检文件 `f`，令 `old_spans / new_spans` 为基线/工作区的 span 多重集（`code_spans` 语义不变），
`removed, added = ac4_span_diff(old_spans, new_spans)`（完整、不截断）：

1. **D1 类移除**（唯一可放行的移除类别）：`removed` 中每一条 `r` 必须**字面含** `AC4_DELETED_DOC`（即 `README.zh-CN.md`）。不含者 → **越界移除 → FAIL**。
2. **最小剔除式等价**：对每条 D1 移除 `r`，令 `e = ac4_excise(r)`：
   - `e is AC4_NOT_D1`（不可能，因已过第 1 步）；
   - `e is None` → 该 span 允许**整体消失**，不产生对应 added；
   - `e` 为 `str` → `added` 中必须**恰有一条**等于 `e`（按多重集扣减 1）。缺该条 → **FAIL**（「D1 移除未产生最小剔除式等价 span」）。
3. **D2 类新增**（唯一可放行的非衍生新增）：扣减后剩余 `added` 必须逐条满足
   `path in AC4_ALLOWED_ADDED_BY_FILE` 且 `span in AC4_ALLOWED_ADDED_BY_FILE[path]` 且
   `Counter(added)[span] <= AC4_MAX_PER_ENTRY`（计数上限 1）。否则 → **FAIL**。
4. **一致性护栏**：若本文件产生了任何 D1 放行，则 `new_text` 全文不得再出现子串 `README.zh-CN`（含非 span 位置）；否则 → **FAIL**（「半清理」）。
5. **可见性硬要求**：存在放行时，必须输出授权明细（格式见 §2.5），`gate` 汇总段与 `check` 单文件输出都要打印；**不打印即视为静默放行 → FAIL**。
6. `removed` 与 `added` 同时为空（最常见）→ PASS，无明细。

### 2.2 冻结算子 `ac4_excise`

```python
def ac4_excise(span: str):
    """D1 最小剔除式重写算子（冻结，顺序固定，每步至多替换一次，禁止任何其它字符改动）。

    返回:
      None        -> 允许该 span 整体消失
      str         -> 允许出现且只允许出现该等价串
      AC4_NOT_D1  -> 该 span 与 D1 无关（调用方判越界）

    步骤:
      S1 re.fullmatch(r"README\\.zh-CN\\.md(:\\d+)?", span)            -> None
      S2 " / README.zh-CN.md" in span -> replace(" / README.zh-CN.md", "", 1)
      S3 elif " README.zh-CN.md" in span -> replace(" README.zh-CN.md", "", 1)
      S4 elif "README.zh-CN.md" in span  -> replace("README.zh-CN.md", "", 1)
      S5 结果 .strip() 为空、或 fullmatch(r":\\d+") -> None
      S6 未含 AC4_DELETED_DOC -> AC4_NOT_D1
    """
```

该算子是**唯一的**重写许可：被删文件名连同至多一个紧邻分隔符被剔除，其余字符逐字不变。
任何措辞润色、空格调整、顺序变化都会使 added ≠ e 而 FAIL —— 这正是「不许顺手改写」的执行点。

### 2.3 冻结常量（写入脚本，不得通配、不得运行时自动发现）

```python
AC4_DELETED_DOC = "README.zh-CN.md"                 # 裁决 D1 删除的交付文档
AC4_DELETED_DOC_FULL_RE = re.compile(r"README\.zh-CN\.md(:\d+)?")   # 仅用于 S1 的 fullmatch
AC4_ALLOWED_ADDED_BY_FILE = {"README.md": ("./scripts/build-webui.sh",)}   # 裁决 D2，绑定文件
AC4_MAX_PER_ENTRY = 1                                # 同一 (file, span) 放行次数上限
AC4_NOT_D1 = object()                                # 哨兵
```

`AC4_ALLOWED_REMOVED`（旧常量）删除；`split_removed`（`scripts/check-zh-docs.py:427-431`）若不再被调用则删除，避免留下第二判据。
`ac4_span_diff`、`code_spans`、**不得**改动。

### 2.4 逐文件裁决（4 个 FAIL 文件）

| 文件 | 现状 diff | 裁定 | 因果链与理由 |
|---|---|---|---|
| `AGENTS.md` | removed 1 条 `README.zh-CN.md:94`；added 无 | **放行**（D1，无需文档侧改动，现状即通过） | D1 删除 `README.zh-CN.md` ⇒ 该文件内任何指向它的字面引用（含 `path:line` 锚点）成为悬空引用；`AC-8` 又要求正式域 0 命中该字面量 ⇒ 该 span **不可能以任何等价 span 保留**（保留即同时违反 AC-8 与「不得留死链」`AGENTS.md:270`）。`ac4_excise` S1 fullmatch `README.zh-CN.md:94` → `None` ⇒ 允许整体消失。doc-writer 已改写为无 span 的散文「原中文 README 第 94 行」（`AGENTS.md:279`），语义未丢。 |
| `docs/superpowers/plans/2026-05-29-webui.md` | removed `git add webui/ crates/ README.md README.zh-CN.md`；added `git add webui/ crates/ README.md` | **放行**（D1，无需文档侧改动，现状即通过） | 同上：span 含已删文件名且位于 `docs/`（AC-8 严格域）⇒ 必须剔除；`ac4_excise` S3 输出恰为 `git add webui/ crates/ README.md`，与实测 added **逐字相等** ⇒ 属最小剔除式，无任何附带改写。 |
| `docs/phase1-refactor-design.md` | removed `README.zh-CN.md:151`（→None）；removed `[2] README.md / README.zh-CN.md   零遥测口径统一；清除 Emoji；补"自定义网关"配置章节`；added `[2] README.md 与原中文 README     零遥测口径统一；清除 Emoji；补"自定义网关"配置章节` | **必须文档侧回退**（doc-writer 改第 601 行） | 第一条同 AGENTS.md，放行。第二条：D1 只授权「剔除已删文件名」，`ac4_excise` S2 的合法输出是 `[2] README.md   零遥测口径统一；清除 Emoji；补"自定义网关"配置章节`（保留原三空格与全部原文，仅删 ` / README.zh-CN.md`）。实测 added 在剔除之外**还**把 `/` 改写成「与原中文 README」并重排空格 —— 属编辑性改写，不是删除的必然后果（不删文件名同样能写出这句）⇒ 不得放行。 |
| `README.md` | removed 9 条（实测清单见下）；added 1 条 `./scripts/build-webui.sh` | **必须文档侧回退**（doc-writer 恢复被删代码块）+ **放行 1 条 D2 新增** | removed 的 9 条**无一含** `README.zh-CN.md`，即全部非 D1；它们也不属于 D2 的必然后果：裁决 b / K7-K8 的冻结文案是「先指向新脚本，**并保留手工等价步骤**」（`01-design.md:295`：`./scripts/build-webui.sh` + `cargo clean …` + `或手工执行等价步骤：cd webui && npm ci && npm run build …`）。手工步骤在引入脚本后**依然真实有效**，删除它们是编辑取舍而非逻辑必然；且整段改写 fenced code block 直接违反汉化铁律 1（`:13`）。⇒ 判「无因果关系的 span 丢失」，必须回退。新增的 `./scripts/build-webui.sh` 是 D2 的**直接产物**（新交付物必须有调用指引），放行且仅此 1 条、仅在此文件。 |

`README.md` 实测 removed 9 条（登记值，以 `check` 输出为权威，逐字复制）：
`cd webui`、`npm ci`、`npm run build    # outputs webui/dist/, embedded by the next Rust build`、`cd ..`、
`# for Windows MSYS / Git Bash users, run`、`# to use the system-installed Node.js.`、
`` # `PATH="/c/Program Files/nodejs:$PATH" npm run build` ``、`PATH="/c/Program Files/nodejs:$PATH" npm run build`、
`) — cargo does not track changes under `（末条为跨行反引号对产生的片段 span，见 `HANDOFF-汉化铁律.md:51` 第 6 条）。

**回退后的 `README.md` 目标态（可机检）**：`removed == []`，`added == ["./scripts/build-webui.sh"]`（计数 1）。
该目标态与条数无关，实现者不必依赖上表逐字转录。

### 2.5 最小回退方案（doc-writer 执行，任务 T-11）

**`README.md`（`#### WebUI 构建` 小节，现行 `:193-206`）**

1. 用 `git show 3ee655e3:README.md` 取回基线该小节的 fenced code block 与注释行，**逐字恢复**（英文注释原样保留 —— 铁律 1 要求代码块一个字符都不动，且 fence 内行不参与 AC-2 统计）。
2. 保留 T-05 已写入的中文散文与新的 ```` ```bash\n./scripts/build-webui.sh\n``` ```` 块（这是 D2 的唯一授权新增，计数保持 1）。
3. **span 计数对齐**（易漏，必做）：恢复代码块后，散文中与代码块内重复的字面量**不得再用反引号包裹**（例如第 203 行的 `` `npm ci` `` 若与恢复后的代码块内的 `npm ci` 重复，须改写成不带反引号的中文叙述，或删除该复述句）。判据：最终每个 span 的出现次数必须与基线**逐条相等**，唯一例外是 D2 的 `./scripts/build-webui.sh`（+1）。
4. 不得改动链接目标（AC-7b）与 Emoji 数（AC-32）；fence 外的散文保持现状中文。

**`docs/phase1-refactor-design.md`（第 601 行，位于 ```` ```text ```` 块内）**

把当行改为最小剔除式输出，即：删除 ` / README.zh-CN.md`，其余字符（含 `[2] README.md` 之后的三空格与后半句全部原文）一字不动。
该行属 fenced block，整行即 span，改动后不得增删其它行、不得改 `[1]/[3]` 的对齐列以外的任何字符。

### 2.6 强度不降的论证（必须写入脚本注释）

- **等效域**：对**不含** `README.zh-CN.md` 且**不等于** `./scripts/build-webui.sh` 的任意 span，v2 与 v1 **判定完全相同** —— added 一侧在两版中都必然越界 FAIL（v1：`added` 非空；v2：不在按文件冻结的放行表内，该表仅 `README.md` 一个成员），removed 一侧在两版中都越界 FAIL。v2 的放宽**只**发生在「与已冻结裁决 D1/D2 有字面因果关系」的 span 上。
- **v2 新增的硬约束（v1 没有）**：(a) D1 移除必须逐条可解释（span 字面含已删文件名）；(b) 其对应 added 必须是冻结算子的机械输出，而非任意串；(c) 一致性护栏（放行后全文不得残留该字面量）；(d) 放行必须可见（不打印即 FAIL）。
- **不变量**：`AC-2` 阈值 `0.05`、`AC-6`、`AC-7a`、`AC-7b`、`AC-32` 的判据与常量一律不变；`code_spans` / `ac4_span_diff` 的提取与完整性不变。
- **不降级的放宽方式一律禁止**：见 §6。

### 2.7 放行明细输出格式

`check` 单文件（note 标签 `AC-4-authorized`）：

```
[AC-4-authorized] <path> | removed=<span 或 -> | added=<span 或 -> | cause=D1|D2
```

`gate` 汇总（在 `AC-2/3/4/6/7b/32 全量 check` 段 detail 末尾追加）：

```
AC-4 授权放行合计 <N> 条（D1 <n1> / D2 <n2>）
  <path> | removed=... | added=... | cause=...
```

---

## 3. AC-8 作用域（v2，冻结）

### 3.1 三段结构（三段都必须执行；任一段不满足即 FAIL）

| 段 | 名称 | 作用域 | 期望 |
|---|---|---|---|
| 1 | 正式域 | `git ls-files -- '*.md' '*.html' '*.json' '*.ts' '*.kt' '*.yml' '*.toml'` 的候选集，**减去**以 `AC8_EXEMPT_PREFIX` 开头的路径 | **0 命中** |
| 2 | 历史域自证 | 候选集中以 `AC8_EXEMPT_PREFIX` 开头的路径 | **≥1 命中，且全部命中行的路径前缀等于 `AC8_EXEMPT_PREFIX`** |
| 3 | 全扩展名兜底 | `site/ .github/ docs/ extensions/`（目录存在才扫；不限扩展名） | **0 命中**（语义与既有的第二次 grep 完全一致，不变） |

### 3.2 排除集与理由

```python
AC8_NEEDLE = "README.zh-CN"
AC8_GLOBS = ("*.md", "*.html", "*.json", "*.ts", "*.kt", "*.yml", "*.toml")
AC8_EXEMPT_PREFIX = ".codebuddy/artifacts/"   # 唯一豁免成员；前缀精确匹配，非 glob 通配
AC8_ANCHOR_FILES = ("README.md", "AGENTS.md")  # 反向护栏锚点
```

排除 `.codebuddy/artifacts/` 的理由：

1. **历史记录不可篡改**：该目录是各 feature 的交接件归档，内容是**已发生事实的历史记录**（任务派发、实测输出、审查结论、用户裁决原文）。其中出现的 `README.zh-CN.md` 是对「当时存在该文件」的记录，改写即篡改历史，与汉化铁律 3「保留原文：文件路径」同源同理。
2. **不是交付文档**：不进 `site/`、不进发行包、不进 IDE 扩展，也不属于 `docs/`；AC-8 的立法意图是「**正式交付文档**不得残留指向已删除文件的引用」，本排除集不触碰该意图。
3. **精确而非通配**：排除集**只有这一个成员**，且按路径前缀精确匹配；`docs/`、`extensions/`、`site/`、`.github/`、根 `README.md`、`AGENTS.md` 以及 `.codebuddy` 下的其它路径（如 `.codebuddy/agents/`）**仍严格 0 命中**。

实现要求（可移植性）：**默认并推荐**用 `git ls-files` 取候选 → Python 按 `AC8_EXEMPT_PREFIX` 过滤 → 对剩余路径调 `git grep -n -F AC8_NEEDLE -- <paths...>`（路径数 ≤1000 时单次调用，超过则分片每片 ≤500）。
仅当实现者实测 `git grep -- ':(exclude).codebuddy/artifacts'` 与上述过滤结果**逐行相同**并把实测输出写进 `03-impl/T-09.md` 时，才允许改用 pathspec magic 形式。两段命中行的输出格式保持 `<path>:<lineno>:<line>`。

### 3.3 反向护栏（防止豁免被滥用或空转）

- **A1 非空**：正式域候选集非空，否则 FAIL（防排除集被写成 `.` 让检查空转）。
- **A2 锚点存在**：`AC8_ANCHOR_FILES` 中的每个路径必须出现在正式域候选集内，否则 FAIL（防误排除；`README.md`、`AGENTS.md` 是本补遗点名要求 0 命中的文件，必须真的被扫到）。
- **A3 历史域存在性**：段 2 命中数必须 ≥1，否则 FAIL，失败文案固定为「历史交接件中的 `README.zh-CN` 记录消失：违反历史不可篡改」。这是**探测器**：把豁免区从「不检查」变成「反向断言其历史痕迹仍在」，可捕获「悄悄清理历史件来修绿 AC-8」这一作弊路径。
- **A4 排除集不外溢**：`AC8_EXEMPT_PREFIX` 在脚本中只能有一处定义、一处使用；段 1/段 2 之外的任何目录级排除一律禁止；段 2 若出现前缀不等于 `AC8_EXEMPT_PREFIX` 的命中行 → FAIL。
- **A5 未跟踪文件**：`git ls-files` / `git grep` 只覆盖已跟踪文件（既有属性，保持不变）。本 feature 的 `.codebuddy/artifacts/2026-09-07-zh-docs-webui/**` 当前 untracked，故不计入段 2；**一旦提交**，段 1 因前缀豁免仍为 0 命中、段 2 命中数上升（仍 ≥1），判定不变 —— 提交不会破坏 AC-8。未跟踪文件的残留由 T-07 人工复核 `git status --porcelain` 兜底。

### 3.4 将来出现新的正式文档残留时怎么办（不降强度）

禁止扩 `AC8_EXEMPT_PREFIX`、禁止新增 `--allow/--skip` 开关。按序取以下文档侧修正：

1. 引用意图是「当前中文 README」→ 改写指向 `README.md`（或删掉行号锚点后指向 `README.md`）；
2. 引用只是列举文件 → 删除该引用句（死链本来就该删，符合 `AGENTS.md:270` 的处理原则）；
3. 该处是**历史事实陈述**且必须保留原文件名 → 只有把该文档移入 `.codebuddy/artifacts/`（历史交接件域）才豁免；留在正式域内一律 FAIL。

任何「改字面量躲避匹配」（`README.zh_CN`、`README.zh-CN `、`README．zh-CN`）同时违反铁律 3 与 AC-8 立法意图，视为作弊。

### 3.5 T-07 人工复核命令（替代 `02-tasks.md:643` 中的旧命令，本补遗优先）

```bash
# 段 1 正式域：应 0 命中
git ls-files -- '*.md' '*.html' '*.json' '*.ts' '*.kt' '*.yml' '*.toml' \
  | grep -v '^\.codebuddy/artifacts/' \
  | xargs -r grep -n -F 'README.zh-CN' --

# 段 2 历史域：应 >=1 命中，且全部位于 .codebuddy/artifacts/
git ls-files -- '*.md' '*.html' '*.json' '*.ts' '*.kt' '*.yml' '*.toml' \
  | grep '^\.codebuddy/artifacts/' \
  | xargs -r grep -n -F 'README.zh-CN' --

# 段 3 全扩展名兜底：应 0 命中
grep -rn 'README.zh-CN' site/ .github/ docs/ extensions/
```

`02-tasks.md:643` 的旧命令（全仓 `grep -rn`）不回改；T-07 报告中须登记「已按 `01-design-addendum.md §3.5` 执行」。

---

## 4. 任务编号方案（T-06 重号消解）

**规则**

1. `T-01 … T-07` 的编号以 `02-tasks.md`（TASKS-001 冻结件）为**唯一权威**，任何实现期文件都不得反噬其编号；故 `T-06 = hostscan 文档改写`（批次 14，doc-writer）保持不变。
2. 实现期新增任务一律取「当前已分配最大号 + 1」顺序分配；**禁止**复用 `T-06`，**禁止**造 `T-06S` 这类后缀号（`STATUS.md:49` 现记的 `T-06S` 作废）。
3. 历史文件名与历史交接件正文**不重命名、不改写**（与 §3.2 的「历史不可篡改」同一原则）。
4. 引用规则：首次出现必写别名，后续可用短号。
   `T-08（= 03-impl/T-06-gate-script.md）`、`T-09（AC-4/AC-8 脚本实施）`、`T-11（文档侧 AC-4 回退）`。
5. 新产出文件命名 `03-impl/T-09*.md`、`04-review/T-09.md`、`03-impl/T-11.md`；自检报告沿用 `03-impl/zh-check-<任务ID>.md`（铁律 §6）。

**映射表（编排者据此更新 `STATUS.md`，本补遗不改该文件）**

| 终态编号 | 任务 | 旧记 | owner | 状态 |
|---|---|---|---|---|
| T-06 | hostscan：清除与新默认矛盾的 `127.0.0.1` 叙述 | T-06（`02-tasks.md:616`） | doc-writer | in_progress |
| T-07 | 全量门禁 + 集成/冒烟验收 | T-07 | test-engineer | pending |
| T-08 | 门禁脚本修正（SKIP_D 等） | `T-06S`、`03-impl/T-06-gate-script.md` | code-implementer | done（仅改编号） |
| T-09 | 本补遗实施：AC-4 v2 + AC-8 v2（脚本） | `STATUS.md:55` 的 T-08 | code-implementer | ready（依赖本补遗） |
| T-10 | 补 D-34a / D-35 报告 + `06-release.md` + `AGENTS.md` 同步 | `STATUS.md:57` 的 T-09 | doc-writer | pending |
| T-11 | 文档侧 AC-4 回退（`README.md`、`docs/phase1-refactor-design.md`） | 新增 | doc-writer | ready（依赖本补遗） |

---

## 5. 任务与改动点

### T-09 · AC-4 v2 + AC-8 v2 脚本实施（owner = `code-implementer`）

- **files_owned**（独占写入）：`scripts/check-zh-docs.py`、`03-impl/T-09.md`、`03-impl/zh-check-T-09.md`
  （注：`02-tasks.md:634` 把 `03-impl/**` 归 T-07，此处按「谁产出谁写」处理，PM 知悉即可，不回改 `02-tasks.md`。）
- **依赖**：`DESIGN-001-A1`（本补遗）。可与 T-11 并行（文件不相交）。
- **复杂度**：M

**改动点（逐项，照做）**

1. `scripts/check-zh-docs.py:41-51`：删除 `AC4_ALLOWED_REMOVED` 及其注释块，替换为 §2.3 的常量集（含 §2.6 的强度注释）。
2. 在 `ac4_span_diff`（`:412-424`）之后新增 `ac4_excise`（§2.2）与 `ac4_verdict`（§2.1 的判定顺序，签名 `ac4_verdict(path, old_text, new_text, old_spans, new_spans) -> (ok, failure_detail_lines, authorized_lines)`）。
3. `:480-507` 的 AC-4 段改为调用 `ac4_verdict`；PASS 分支按 §2.7 打印 `AC-4-authorized` 明细。删除不再被调用的 `split_removed`（`:427-431`）。
4. `:1002-1046` `gate_check_ac8` 重写为 §3.1 三段 + §3.3 的 A1–A4 断言；新增 `ac8_candidates(root)` 与 `git_grep_fixed(root, needle, paths)`（后者 `rc not in (0,1)` → `EnvError`，映射到退出码 2）。
5. `cmd_gate`（`:1049` 起）：在 `AC-2/3/4/6/7b/32 全量 check` 段 detail 末尾追加 §2.7 的放行汇总。
6. **不得改动**：`code_spans`、`ac4_span_diff`、`ac2_en_lines`、AC-2 阈值、AC-5/AC-5b、AC-6、AC-7a、AC-7b、AC-32、`hostscan` 相关、`SKIP_*` / `OWNED_ELSEWHERE` 常量。

**验收标准（可测）**

1. `python3 scripts/check-zh-docs.py check --base 3ee655e3 --files AGENTS.md docs/superpowers/plans/2026-05-29-webui.md` → 两文件 PASS，且授权明细恰为 2 条 `cause=D1`。
2. T-11 完成后：`check --base 3ee655e3 --files README.md docs/phase1-refactor-design.md` → PASS；`README.md` 授权明细恰为 1 条 `cause=D2 / added=./scripts/build-webui.sh`；`docs/phase1-refactor-design.md` 恰为 2 条 `cause=D1`（其中 1 条带 added）。
3. `python3 scripts/check-zh-docs.py gate --base 3ee655e3 --report .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-T-09.md` → **exit 0**；报告中 AC-8 三段分别显示「正式域 0 命中」「历史域 N 命中（N≥1，全部位于 `.codebuddy/artifacts/`）」「兜底域 0 命中」。
4. **反例自检（必须 FAIL，否则判据无效）** —— 每项做完必须还原，并用 `git diff --stat 3ee655e3 -- <file>` 确认与改前一致（禁止用 `git checkout --` 还原，会丢失未提交的汉化）：
   - (a) 备份 `README.md` → 删除回退后代码块中的任意一行（如 `cd webui`）→ AC-4 必须 FAIL（证明「无因果丢失」仍被拦截）→ 还原；
   - (b) 备份 `docs/` 下任一已跟踪 md → 追加一行含 `README.zh-CN.md` 的文字 → AC-8 段 1 必须 FAIL → 还原；
   - (c) 临时把 `AC8_EXEMPT_PREFIX` 改为 `.` → A2 锚点断言必须 FAIL（退出码 1 或 2，不得为 0）→ 还原；
   - (d) 临时把 `AC8_EXEMPT_PREFIX` 改为 `.codebuddy/artifacts-nonexistent/` → 段 2 命中 0 → 必须 FAIL → 还原。
5. `03-impl/T-09.md` 报告：新增/修改函数的签名、4 项反例的**实测输出**、gate 全量输出、以及「若采用 pathspec magic 形式」的逐行比对证据。

### T-11 · 文档侧 AC-4 回退（owner = `doc-writer`）

- **files_owned**：`README.md`、`docs/phase1-refactor-design.md`、`03-impl/T-11.md`、`03-impl/zh-check-T-11.md`
- **依赖**：本补遗（可与 T-09 并行）
- **复杂度**：S
- **内容**：按 §2.5 执行两处回退。
- **验收标准**：
  1. `check --base 3ee655e3 --files README.md docs/phase1-refactor-design.md` → PASS（AC-2 / AC-4 / AC-7b / AC-32 全绿），授权明细如 T-09 验收第 2 条；
  2. `README.md` 的 `removed == []`、`added == ["./scripts/build-webui.sh"]`（计数 1）；
  3. `docs/phase1-refactor-design.md` 第 601 行等于 `ac4_excise` 的输出（逐字，含三空格），且该 `text` 块内其它行未变；
  4. AC-8 段 3（含 `docs/`）仍 0 命中；回退不得引入 `README.zh-CN` 字面量，也不得新增 Emoji。

### 集成顺序

1. T-09 与 T-11 并行（`scripts/check-zh-docs.py` vs 两个 md，files_owned 不相交）；
2. T-09 单独先跑验收第 1 条（不依赖 T-11）；
3. T-11 完成后跑 T-09 验收第 2、3 条；
4. 再进入 T-06（hostscan，doc-writer）与 T-07（全量验收）；T-07 依赖 T-09 + T-11 + T-06。

---

## 6. 不得做的事（边界）

**绝对禁止**

1. 改 `AC-2` 阈值 `0.05`、`AC-6`、`AC-7a`、`AC-7b`、`AC-32` 的任何判据或常量；改 `code_spans` / `ac4_span_diff` 的提取与完整性（含截断）。
2. 按目录 / 通配 / 前缀 / 正则**匹配** span 白名单（白名单只能是与冻结串的**完整相等**，或 `ac4_excise` 的输出）；按目录豁免 AC-4。
3. 让脚本**运行时自动发现**放行项（如「扫到含 `README.zh-CN` 的 span 就自动放行」「新文件自动免检」）。
4. 给 `gate` / `check` 增加任何降级开关（`--allow-ac4-diff`、`--skip-ac8`、`--en-ratio` 上调等）。
5. 为转绿而修改 `.codebuddy/artifacts/**` 历史件正文（A3 断言会捕获），或把历史件移出该目录来「减少命中」。
6. 修改被引用字面量的拼写以躲避匹配（`README.zh_CN`、增删空格、替换连字符）。
7. AC-4 放行时不打印明细（静默 PASS 视同 FAIL）；把「越界移除」降级成 note/warning。
8. 扩大 `AC8_EXEMPT_PREFIX`（成员数必须恒为 1）；在段 1/段 2 之外新增任何排除。
9. 用 `git checkout -- <file>` / `git stash` 做反例自检的还原（会丢失未提交汉化）；只能先备份再还原。
10. 实现者自行修改 `01-design.md`、`02-tasks.md`、`STATUS.md`、`00-*` 与本补遗的正文（需要变更走编排者）。

**需重新走裁决的触发条件**（出现任一即停止、置 `blocked` 并升级给编排者与用户）

- 出现 §2.3/§2.4 清单之外的 removed 或 added span（含「同义改写」「顺手润色」导致的串不等）；
- 需要在 `.codebuddy/artifacts/` 之外新增 AC-8 豁免路径，或在 AC-4 中引入第二个放行类别（D3）；
- 需要放宽 AC-2 / AC-6 / AC-7a / AC-7b / AC-32 中的任意一项；
- 又要删除其它交付文档（如 `docs/x.md`）并由此产生成批 span 消失 —— 必须作为新的 D1' 条目单独裁决并登记到本补遗 v2；
- 反向护栏 A3（历史域命中 ≥1）在历史件未被清理的情况下失败 —— 先查实现，再走裁决。

---

## 7. 风险与开放问题

- **R1 · 转录风险**：§2.4 的字符串取自 2026-09-08 的 `check` 实测输出；若全角/半角标点或空格与文件实际内容有偏差，以 `check` 输出为权威逐字复制，**不得**为消除偏差而放宽判据。发现清单外条目 → 按 §6 升级。
- **R2 · README 回退后的副作用**：恢复 fenced block 不影响 AC-2（fence 内行被 R2 跳过），但会改变行数与 span 计数；必须同时做 §2.5 第 3 步的计数对齐，否则「恢复过度」会变成新的 added。
- **R3 · 提交 artifacts 后的 AC-4（重要，需编排者裁决）**：`list_md_files`（`:124-126`）只收已跟踪 md。当前本 feature 的 `.codebuddy/artifacts/2026-09-07-zh-docs-webui/**` 是 untracked，故不参与 AC-4。**一旦提交**，这些文件在基线不存在（`old_text = ""`，`is_new=True`），其全部 span 都会记为 added → AC-4 大面积 FAIL。本补遗**不授权**为「无基线的新文件」开豁免（那属降强度）。可选处置（需裁决）：(a) 提交时不纳入该目录（维持与 AC-1 分母「当前跟踪的 md」一致的现状）；(b) 新增 AC-4 免检域并绑定同一前缀 `.codebuddy/artifacts/`（与 AC-8 同前缀同理由，但属目录豁免，必须用户裁决后写入本补遗 v2）。此项**不阻塞** T-09 / T-11 / T-07（现状下 gate 不受影响），但阻塞「把 artifacts 一并提交」这一交付动作。
- **R4 · A3 的脆弱性**：历史域存在性断言使 gate 依赖历史件内容；若将来确有理由清理旧交接件，需按 §6 走裁决更新本补遗，不得自行删除断言。
- **R5 · T-06 与 T-11 的文件所有权**：T-06 的 `files_owned` 是 `hostscan` 输出的文件列表（排除 `README.md`）；若其输出包含 `docs/phase1-refactor-design.md`，则 T-06 与 T-11 存在写冲突 —— 由 PM 决定串行顺序（建议 T-11 先，T-06 后，且 T-06 不得改动 fenced block）。
- **R6 · 人工与脚本口径**：`02-tasks.md:643` 的 AC-8 人工命令为全仓 grep，与新作用域不一致；已按 §3.5 处理，不回改冻结件，由 T-07 报告登记偏差。
