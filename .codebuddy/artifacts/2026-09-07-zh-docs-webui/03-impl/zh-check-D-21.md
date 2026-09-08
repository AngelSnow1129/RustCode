# zh-check 报告

- 基线 (--base): `3ee655e3`
- 阈值 (--en-ratio): `0.05`
- 判定: AC-2 / AC-4 / AC-6 / AC-7b / AC-32，任一失败即 FAIL（fail-closed）

## 汇总

- 受检文件: 6
- PASS: 6
- FAIL: 0

## PASS `docs/hook-architecture.md`

- en=0/21 ratio=0.0000

## PASS `docs/phase2-subagent-status.md`

- en=0/154 ratio=0.0000

## PASS `docs/mcp.md`

- en=0/115 ratio=0.0000

## PASS `docs/mcp-rmcp-feasibility.md`

- en=0/121 ratio=0.0000

## PASS `docs/webhook-implementation-summary.md`

- en=0/106 ratio=0.0000

## PASS `docs/compact-durable-checkpoint-design.md`

- en=0/102 ratio=0.0000

---

# D-21 手写小结（追加于最后一次自检之后）

- 任务 ID：`D-21`（批次 7）
- 自检命令（最后一次执行，本报告即其产物）：

```bash
python3 scripts/check-zh-docs.py check --base 3ee655e3 \
  --report .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-21.md \
  --files docs/hook-architecture.md docs/phase2-subagent-status.md docs/mcp.md \
          docs/mcp-rmcp-feasibility.md docs/webhook-implementation-summary.md \
          docs/compact-durable-checkpoint-design.md
```

- 结果：受检 6，PASS 6，FAIL 0；6 个文件 en 均归零。

## 1. 动手前的自检结论（区分两类文件）

开工前先跑了一次自检，基线 offender 清单为：

| 文件 | 自检前 en/total | 判定 |
|---|---|---|
| `docs/hook-architecture.md` | 0/21 = 0.0000 | **已中文且无 offender → no-op** |
| `docs/mcp-rmcp-feasibility.md` | 0/121 = 0.0000 | **已中文且无 offender → no-op** |
| `docs/phase2-subagent-status.md` | 4/154 = 0.0260 | 已中文，按本批说明「已中文只需清 offender」清残留 |
| `docs/mcp.md` | 2/115 = 0.0174 | 已中文，同上 |
| `docs/webhook-implementation-summary.md` | 12/106 = 0.1132 | 已中文但 **AC-2 FAIL**，须清 offender 才能过阈 |
| `docs/compact-durable-checkpoint-design.md` | 7/102 = 0.0686 | 已中文但 **AC-2 FAIL**，须清 offender 才能过阈 |

本批**没有**「纯英文需全文译」的文件：6 个文件在基线 `3ee655e3` 上正文均已为中文，
只是残留少量英文 offender 行（多为英文标题、事件名清单、字段名续行）。

## 2. 逐文件改动量

| 文件 | 改动行数 | 说明 |
|---|---|---|
| `docs/hook-architecture.md` | 0 | **no-op**。全文 241 行，除 ASCII 架构图（均在 fenced block 内）外正文已全中文；en=0，无任何 offender 可清，按铁律 §3 一字未动 |
| `docs/mcp-rmcp-feasibility.md` | 0 | **no-op**。200 行，en=0，无 offender，未改动 |
| `docs/phase2-subagent-status.md` | 4 行（+4/-4） | 清 4 处 offender：`:102` 加「字段：」、`:103` 把 `+` 改为「与」、`:107` 加「派生」、`:111` 加「含」 |
| `docs/mcp.md` | 2 行（+2/-2） | 清 2 处 offender：`:185` 加「开启时」、`:192` 加「与 … 定义」 |
| `docs/webhook-implementation-summary.md` | 12 行（+12/-12） | 清 12 处 offender：`:62`–`:73` 的 12 个 Hook 时机名后各加中文括注 |
| `docs/compact-durable-checkpoint-design.md` | 7 行（+7/-7） | 清 7 处 offender：`:74` 加「（运行时租约与栅栏机制）」；`:83`/`:95`/`:102` 三个英文小标题加「（内核层）」「（能力装配层）」「（驱动层）」；`:122` 加「（compact 处理器）」；`:123` 加「（v1 回退路径）」；`:154` 加「（规范化快照提交）」 |

合计 4 个文件、25 行纯行内替换（`git diff --numstat` 为 7/7、2/2、4/4、12/12，
`+N` 与 `-N` 严格相等），**未增删空行、未改段落结构、未改标题层级**；
6 个文件总行数保持 241 / 232 / 260 / 200 / 311 / 156，与改动前完全一致。

## 3. 保留原文的处置方式（本批特殊说明 §2）

- Hook 事件名（`PreToolUse` / `PostToolUse` / `OnTurnStart` / `OnSessionStart` / `SystemPrompt` 等）
  、MCP 方法名（`initialize` / `tools/list` / `tools/call`）、webhook 字段名（`hook_name` /
  `trigger` / `event` / `context` / `modified_content` 等）、JSON 键名**一律保留原文**，
  采用「英文名 + 全角括号中文括注」的形式补齐中文，例如
  `   - [+] OnTurnStart（Turn 开始）`；**未**给这些名字套反引号（铁律 §2）。
- 文件路径 / crate 名（`rustcode-core`、`rustcode-bridge`、`rustcode-capabilities`）、
  配置键（`skip_tls_verify`、`trust_os_roots`）、函数名、trait 名原样保留。
- 三个英文小标题 `### kernel` / `### capabilities / coding assembly` / `### drivers`
  只**追加**中文括注，未改 `#` 层级，也未改任何链接目标与锚点（铁律 §4、§5）。
- 6 个文件**均无 YAML frontmatter**（首行非 `---`），故 AC-6 键名集合前后皆为空集（相等），
  且**无** `description` 需要汉化，铁律 §7 的「逐条列示」清单为空。

## 4. 残留行清单与白名单判定

- **残留 en 行：0 行。** 6 个文件 AC-2 的 en 计数全部为 0（见上表 ratio 列），
  因此**没有**需要申请白名单的残留行。
- 说明一个本批易踩的点：全角标点（`，。：；、（）`）不在脚本 `CJK_RE` 判定区间内，
  只加全角括号**不能**消除 offender（铁律 §5.2）。本批所有括注内均写入了真实汉字
  （如「字段」「派生」「内核层」「Turn 开始」），故 6 处/25 行全部真正归零。
- 本批未出现跨行反引号对（铁律 §5.6）需要保留原折行的情形：所有改动均为单行内追加，
  反引号位置与折行位置一字未动。

## 5. 四项实测结果

| 项 | 结果 | 实测依据 |
|---|---|---|
| **AC-2** `en/total <= 0.05` | **PASS**（6/6） | 改动后 en/total 全部为 0/21、0/154、0/115、0/121、0/106、0/102，ratio 均为 0.0000 |
| **AC-4** inline code + fenced code 多重集相等 | **PASS**（6/6） | 最后一次 `check` 无 AC-4 FAIL 输出；本批 25 行改动**未新增/删除任何反引号**，fenced block 与代码块内容零改动（已用 `git diff -U0` 逐行核对） |
| **AC-7b** `](...)` 链接目标多重集相等 | **PASS**（6/6） | 最后一次 `check` 无 AC-7b FAIL 输出；本批未新增、删除或改写任何 Markdown 链接（仅 `webhook-implementation-summary.md` 文末的 4 条相对链接保持原样，未触碰） |
| **AC-32** Emoji 数不增加 | **PASS**（6/6） | 最后一次 `check` 无 AC-32 FAIL 输出；本批新增字符仅为汉字与全角括号，未引入任何 Emoji |

补充实测：`git diff --numstat` 显示 4 个改动文件的 `+N` 与 `-N` 完全相等
（7/7、2/2、4/4、12/12），无空行增删（铁律 §5.4）。

## 6. 边界声明

- 只改 `files_owned` 内 6 个文件，未碰 `docs/` 根级其它文件（属 D-18/D-19/D-20/D-22/D-23），
  未改源码、未执行 `cargo`/`npm` 构建与测试、未做 git 写操作（commit/stash）。
- 已中文段落未做润色或改写，改动严格限于 `check` 报出的英文 offender 行（铁律 §8）。
