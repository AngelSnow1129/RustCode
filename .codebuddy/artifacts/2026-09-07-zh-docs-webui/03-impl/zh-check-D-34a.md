# D-34a 汉化自检报告（doc-writer）

- 任务：D-34a / feature `2026-09-07-zh-docs-webui`
- 基线：`3ee655e3`，分支 `dev`
- 工具：`python3 scripts/check-zh-docs.py`（`--base 3ee655e3`，默认阈值 `en_ratio <= 0.05`）
- 结论：**2 个文件全部 PASS**，`status: done`
- 与 `03-impl/zh-check-D-34b.md` 的关系：D-34b 已做 `references/` 下 4 件，本批补 `SKILL.md` 与 `references/hooks-patterns.md` 2 件；两批合计覆盖 `rustcode-automation-recommender/` 下全部 6 个 md（见 §6）

## 1. 本次覆盖的文件（2 个）

路径前缀统一记为 `.../ = crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/`

| 文件 | 改动前 en/total | 改动后 en/total | `--numstat` | 总行数 old → new |
|---|---|---|---|---|
| `.../SKILL.md` | 144/153 = 0.9412 | **1/152 = 0.0066** | 144 +/145 - | 399 → 398 |
| `.../references/hooks-patterns.md` | 100/111 = 0.9009 | **1/111 = 0.0090** | 105 +/105 - | 209 → 209 |

合计 `249 insertions(+), 250 deletions(-)`（与 `git diff 3ee655e3 --stat -- <两文件>` 的汇总行一致）。

`en/total` 的分母 `total` 由脚本 `ac2_en_lines()` 定义：剔除 fence 行、fence 体内整行、空行、缩进代码块、表格分隔行、剥离 inline code / URL / HTML / 图片后为空的行。**不是**文件的物理行数，故 `total` 与 `wc -l` 不相等属正常现象。

改动前数字非肉眼估算，由脚本自身口径复算：

```
$ python3 -c "
import importlib.util, subprocess
spec = importlib.util.spec_from_file_location('zh','scripts/check-zh-docs.py')
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
for rel in [...]:
    old = subprocess.run(['git','show','3ee655e3:'+rel],capture_output=True,text=True).stdout
    new = open(rel, encoding='utf-8').read()
    ot,oe,_,_ = m.ac2_en_lines(old); nt,ne,_,_ = m.ac2_en_lines(new)
    print(...)
"
.../SKILL.md
  OLD en=144/total=153 ratio=0.9412
  NEW en=1/total=152 ratio=0.0066
.../references/hooks-patterns.md
  OLD en=100/total=111 ratio=0.9009
  NEW en=1/total=111 ratio=0.0090
```

未触碰（本批未写入任何 md 正文）：本节为**自检报告**，D-34a 的汉化正文改动由上一批（12 批）完成并留在未提交工作区；本次只做核实与记录，未再改动这两个文件的一个字节。

## 2. check 命令的真实输出与退出码

两文件一次性合并执行（`EXIT=` 为 `python3` 的真实退出码，`0` = PASS）：

```
$ python3 scripts/check-zh-docs.py check --base 3ee655e3 --files \
    crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/SKILL.md \
    crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/hooks-patterns.md
PASS crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/SKILL.md en=1/152=0.0066
  [AC-6-desc-changed] description: 'Analyze a codebase and recommend RustCode automations (hooks, subagents, skills, plugins, MCP servers). Use when user asks for automation recommendations, wants to optimize their RustCode setup, mentions improving their RustCode workflow, asks how to first set up RustCode for a project, or wants to know what RustCode features they should use.' => '分析代码库并推荐 RustCode 自动化配置（hooks、subagents、skills、plugins、MCP servers）。当用户询问自动化推荐、希望优化 RustCode 配置、提到改进 RustCode 工作流、询问如何为一个项目首次配置 RustCode，或想知道该使用哪些 RustCode 功能时使用。'
PASS crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/hooks-patterns.md en=1/111=0.0090

check: 受检 2，PASS 2，FAIL 0
EXIT=0
```

逐个文件单独执行（确认结论不依赖合并口径）：

```
$ python3 scripts/check-zh-docs.py check --base 3ee655e3 --files <...>/SKILL.md
PASS .../SKILL.md en=1/152=0.0066
  [AC-6-desc-changed] description: 'Analyze a codebase ... they should use.' => '分析代码库 ... 该使用哪些 RustCode 功能时使用。'

check: 受检 1，PASS 1，FAIL 0
EXIT_A=0

$ python3 scripts/check-zh-docs.py check --base 3ee655e3 --files <...>/references/hooks-patterns.md
PASS .../references/hooks-patterns.md en=1/111=0.0090

check: 受检 1，PASS 1，FAIL 0
EXIT_B=0
```

判读：`check` 为 fail-closed，任一 AC 失败都会打印 `[AC-x]` 明细并返回 1。上述 3 次执行均无 FAIL 明细、退出码均为 0，即 **AC-2 / AC-4 / AC-6 / AC-7b / AC-32 五项全部通过**。

`[AC-6-desc-changed]` 是 AC-6 的**信息条目**而非失败项（AC-6 判的是 frontmatter **键名集合相等**，`description` 值变更只是被逐条列示供人工确认）；它与 `05-test-report.md` §5 AC-6 行登记的「`SKILL.md` 的 `description` 变更已逐条记录」一致。

## 3. AC-5b：frontmatter `name:` 未被改动

```
$ git diff 3ee655e3 -U0 -- \
    crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/SKILL.md \
    crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/hooks-patterns.md \
    | grep -E '^[+-]name:'
（无输出）
GREP_EXIT=1        # 1 = 无命中，符合预期
```

两个文件的 `-U0` diff 中**没有任何** `^[+-]name:` 行，运行时 prompt 载荷的 `name:` 键与未改动。`gate` 侧同一判据也已通过（`AC-5b 无 ^[+-]name: 变更: OK`）。

skill 的 `name:` 值仍为 `setup`（`SKILL.md:2`），与基线一致；`references/hooks-patterns.md` 首行不是 `---`，无 frontmatter。

## 4. frontmatter 键名集合未变（AC-6）

```
$ python3 -c "... m.frontmatter_keys(old) / m.frontmatter_keys(new) ..."
.../SKILL.md                keys_old=['allowed_tools','argument_hint','description','name','user_invocable']
.../SKILL.md                keys_new=['allowed_tools','argument_hint','description','name','user_invocable']
.../references/hooks-patterns.md  keys_old=[]
.../references/hooks-patterns.md  keys_new=[]
```

- `SKILL.md`：键名集合 `{allowed_tools, argument_hint, description, name, user_invocable}` 前后**完全相等**，无增删、无重命名。
- `hooks-patterns.md`：无 frontmatter，空集恒等。

`--stat` 与 AC-5b 全仓口径：

```
$ git diff 3ee655e3 --stat -- <两文件>
 .../rustcode-automation-recommender/SKILL.md       | 289 ++++++++++-----------
 .../references/hooks-patterns.md                   | 210 +++++++--------
 2 files changed, 249 insertions(+), 250 deletions(-)
EXIT=0
```

diff 非空，满足 AC-5b「setup-seeds 必须有改动」的**正向**要求。

## 5. 两处残留 en 行的归因（均未超阈值，不修）

两个文件各剩 1 行 en，均**不是**漏译，逐条给出实测归因：

| 文件:行 | 内容 | 归因 | 是否需修 |
|---|---|---|---|
| `.../SKILL.md:218` | `  "mcpServers": {` | JSON 模板载荷，位于 ```` ```json ```` 块内，只可逐字保留。它被计为 en 是脚本 fence 状态机错位的**误计**（见 §5.1） | 否 |
| `.../references/hooks-patterns.md:17` | `### ESLint（JavaScript/TypeScript lint）` | 全角括号 `（` `）` 是 U+FF08/FF09，**不在**脚本 `CJK_RE` 的四个区间内，故整行无 CJK 被判 en。同 `zh-check-D-34b.md` §5.2 已记录的全角标点坑 | 否（改标题会破坏专有名词原样） |

### 5.1 `SKILL.md:218` 的错位证据（与 G5 缺陷 D-3 同源）

按脚本 `_is_fence()` 走状态机实测，`SKILL.md` 正文的围栏开合如下（工作区行号）：

```
37  ```bash     -> inside=True
50  ```         -> inside=False
...
192 ```markdown -> inside=True
216 ```json     -> inside=False      ← 应为开，实为关
225 ```         -> inside=True       ← 应为关，实为开
...
```

即 **第 192 行的 ```` ```markdown ```` 在文件中没有闭合围栏**（其模板内容一路延伸到第 216 行的 ```` ```json ````，后者被状态机当成它的闭合行）。结果是 217–224 行的 JSON 模板被误判为**围栏外**散文，其中 2 空格缩进的 `  "mcpServers": {` 既不满足 R3-a 的缩进代码块跳过条件，又含 `mcpServers`（≥4 个连续 ASCII 字母）且无 CJK，于是被计为 en；4 空格缩进的 219–222 行走 R3-a 进入 R5 跳过列表。

**该错位是基线既有**：基线同样在 193 行开 ```` ```markdown ````、217 行遇 ```` ```json ```` 时置 `inside=False`，32 条围栏行、最终状态 `False`，与工作区逐条一致（两侧围栏条数均为 32）。本批未加重、也未修复——修复需在文件里补一条闭合围栏，属**改 md 正文**，不在 D-34a 的 `files_owned` 授权内，且与 G5 已登记的 D-3（脚本围栏嵌套失效）为同一根因，应由 T-01 统一处理。

方向性判断：**over-count（多算）是 fail-safe 方向**，不会掩盖真实英文残留；且 `1/152 = 0.0066` 远低于 0.05 阈值，不影响 PASS。

### 5.2 用 `--en-ratio 0` 收紧后的行为（如实记录）

```
$ python3 scripts/check-zh-docs.py check --base 3ee655e3 --en-ratio 0 --files <两文件>
FAIL .../SKILL.md en=1/152=0.0066
  [AC-2] AC-2 en_ratio 0.0066 > 0.0000 (en=1/total=152)
  [AC-2]   .../SKILL.md:218:   "mcpServers": {
FAIL .../references/hooks-patterns.md en=1/111=0.0090
  [AC-2] AC-2 en_ratio 0.0090 > 0.0000 (en=1/total=111)
  [AC-2]   .../references/hooks-patterns.md:17: ### ESLint（JavaScript/TypeScript lint）

check: 受检 2，PASS 0，FAIL 2
EXIT=1
```

即：**按默认阈值 0.05 判 PASS；按 `--en-ratio 0` 的极严口径判 FAIL**，失败原因恰为上表两行。本 feature 的门禁口径以默认 0.05 为准（`01-design.md` / `gate` 实现），`--en-ratio 0` 仅作定位工具使用。此结论与 `05-test-report.md` §5 AC-2 行一致。

## 6. D-34a + D-34b 合计覆盖的 6 个文件

`rustcode-automation-recommender/` 下**全部** md 即 6 个，两批刚好覆盖完整：

| # | 文件 | 归属批次 | en/total（现工作区） |
|---|---|---|---|
| 1 | `.../SKILL.md` | **D-34a**（本报告） | 1/152 = 0.0066 |
| 2 | `.../references/hooks-patterns.md` | **D-34a**（本报告） | 1/111 = 0.0090 |
| 3 | `.../references/mcp-servers.md` | D-34b | 0/172 = 0.0000 |
| 4 | `.../references/plugins-reference.md` | D-34b | 0/61 = 0.0000 |
| 5 | `.../references/skills-reference.md` | D-34b | 0/94 = 0.0000 |
| 6 | `.../references/subagent-templates.md` | D-34b | 0/114 = 0.0000 |

覆盖完整性实测（目录下无遗漏 md、无未跟踪新增）：

```
$ find crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender -name '*.md' | sort
（恰好上述 6 个路径）

$ git diff --name-only 3ee655e3 -- crates/rustcode-capabilities/assets/setup-seeds/
（恰好上述 6 个路径，6 条，全部为 ` M` 已跟踪修改，无新增/删除）

$ git status --porcelain -- crates/rustcode-capabilities/assets/setup-seeds/
 M <6 个路径>
```

6 个文件全为**已跟踪文件的修改**（无 `??`、无 `D`），因此全部进入 AC-1 分母与 AC-4 比较，无「新文件无基线」问题（对比 R3 提交口径下 `.codebuddy/artifacts/**` 的处境）。

## 7. 术语与命名一致性检查结论

- 6 个文件的 `description` / `name` 等 frontmatter 键名保持英文原样，仅 `SKILL.md` 的 `description` **值**由英译中（AC-6 已逐条列示，属 feature 明确要求的范围：`00-requirement.md` Q5-A）。
- 专有名词（MCP、Prettier、ESLint、Biome、gofmt、rustfmt、Clippy、GitHub、GitLab、Linear、Neon、Sentry、Slack、Notion、Docker、Kubernetes、LSP、subagent、plugin、hook 等）保持英文原样。
- D-34a 与 D-34b 的译法一致：`**Best for** / **Value** / **Model** / **Tools**` 统一译 `**适用场景** / **价值** / **模型** / **工具**`；检测依据表头统一译 `**检测依据** / **是否存在**`。
- 未新增 Emoji（AC-32 在 `gate` 的 231 文件全量 check 中 0 FAIL）。
- 未改文件名、未改目录结构、未改标题层级（`SKILL.md` 的一级标题为 `# RustCode 自动化推荐器`，`hooks-patterns.md` 为 `# Hooks 推荐`）。

## 8. 风险与未验证范围

### 8.1 风险

1. **围栏未闭合（基线既有，中低风险）**：§5.1 实测 `SKILL.md` 的 ```` ```markdown ```` 块（工作区第 192 行）无闭合围栏，会导致该文件在**渲染**时后半部分被当作代码块。这不是本批引入，但会随汉化产物一起进入运行时 prompt 载荷。修复需改 md 正文并补一个围栏行，与 D-3 同根因，**建议由 T-01 / architect 统一裁决后另开小任务**，不在本批 `files_owned` 内自行处理。
2. **en=1 的口径敏感性（低风险）**：两文件在 `--en-ratio 0` 下会 FAIL。若后续把门禁阈值统一收紧到 0，这两行会成为阻塞项，届时必须先解决 §5.1 的围栏错位并给 `hooks-patterns.md:17` 一个带汉字的标题（例如 `### ESLint（JavaScript/TypeScript 代码 lint）`）。
3. **语义可读性代价（低风险）**：与 D-34b §6.1 同源——为同时满足「保留专有名词」与「行内必须有 CJK」，个别标题带中文括注。不影响功能。
4. **运行时未验证（低风险）**：本批未构建、未运行 RustCode，未验证 setup-seeds 汉化后被实际加载的推荐效果。依赖的匹配关键字均在 inline code / JSON 键内，本批一字未改，理论无影响。

### 8.2 未验证范围

- 未用 `--report` 之外的方式交叉核对 AC-3 残留行白名单（本次只 `check` 了本批 2 个文件，全量白名单判定见 `05-test-report.md` §4.B）。
- 未验证这 2 个文件在 Windows / macOS 下的换行与编码（工作区为 Linux，文件均为 UTF-8、LF，与基线一致）。
- 未对 `references/` 下另外 4 个文件（D-34b 的 `files_owned`）做二次复核，本批不越界。

## 9. 回滚方案

改动全部在未提交工作区（HEAD 仍为 `3ee655e3`），回滚代价为一次文件级还原：

```bash
git checkout 3ee655e3 -- \
  crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/SKILL.md \
  crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/hooks-patterns.md
```

- 回滚后：`setup-seeds` 的改动文件数由 6 降为 4，AC-5b 的「有改动」判据**仍满足**（另 4 个文件仍在），但这两个文件的 en 会回到 0.9412 / 0.9009，`gate` 的 AC-2 将立刻 FAIL。
- 判定时机：只要 `gate` 重新出现这两个文件的 `[AC-2]` 明细，即可确认回滚已生效；若需保留汉化、仅撤回本报告，删除 `03-impl/zh-check-D-34a.md` 即可（该文件为未跟踪新文件，不影响任何门禁口径）。

## 10. 纪律确认

- 本轮 `files_owned` 仅本报告文件；**未修改任何 md 汉化正文、源码、测试、脚本**。
- 未执行 `cargo` / `npm`；未执行 `git add` / `git commit`；HEAD 仍为 `3ee655e3`。
- 所有数字均为本机实测，可由上文贴出的命令原样复现；本文件无 TODO / 占位符。
