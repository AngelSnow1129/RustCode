# D-34b 汉化自检报告（doc-writer）

- 任务：D-34b / feature `2026-09-07-zh-docs-webui`
- 基线：`3ee655e3`，分支 `dev`
- 工具：`python3 scripts/check-zh-docs.py`（默认 `--base 3ee655e3`，阈值 `en_ratio <= 0.05`）
- 结论：**4 个文件全部 PASS**，`status: done`

## 1. 本次实际改动的文件（files_owned，4 个）

路径前缀统一记为 `.../ = crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/`

| 文件 | 改动前 en/total | 改动后 en/total | 行数变化 | 说明 |
|---|---|---|---|---|
| `.../mcp-servers.md` | 103/172 = 0.5988 | **0/172 = 0.0000** | 163 +/163 - | 上一批只译到第 84 行；本次补齐第 85 行至文末 |
| `.../plugins-reference.md` | 56/61 = 0.9180 | **0/61 = 0.0000** | 58 +/58 - | 全文英译中，此前基本未译 |
| `.../skills-reference.md` | 82/94 = 0.8723 | **0/94 = 0.0000** | 84 +/84 - | 只译散文与表格；全部 fenced block 原样保留 |
| `.../subagent-templates.md` | 103/114 = 0.9035 | **0/114 = 0.0000** | 103 +/103 - | 全文英译中，含唯一一个 fenced block 原样保留 |

合计 `407 insertions(+), 407 deletions(-)`：新增行与删除行数相等，未增删空行，段落结构与标题层级未变。

未触碰（符合 files_owned 约束）：同目录 `SKILL.md`、`references/hooks-patterns.md`。这两个文件在本次开始前就已处于已改动状态（上一批产物），本次未再写入，工作区内容与接手时一致。

## 2. 四条 check 命令的真实输出与退出码

逐个文件执行（每条命令后 `EXIT=` 为 `python3` 的真实退出码，`0` = PASS）：

```
$ python3 scripts/check-zh-docs.py check --files crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/mcp-servers.md
PASS crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/mcp-servers.md en=0/172=0.0000

check: 受检 1，PASS 1，FAIL 0
EXIT=0
```

```
$ python3 scripts/check-zh-docs.py check --files crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/plugins-reference.md
PASS crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/plugins-reference.md en=0/61=0.0000

check: 受检 1，PASS 1，FAIL 0
EXIT=0
```

```
$ python3 scripts/check-zh-docs.py check --files crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/skills-reference.md
PASS crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/skills-reference.md en=0/94=0.0000

check: 受检 1，PASS 1，FAIL 0
EXIT=0
```

```
$ python3 scripts/check-zh-docs.py check --files crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/subagent-templates.md
PASS crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/subagent-templates.md en=0/114=0.0000

check: 受检 1，PASS 1，FAIL 0
EXIT=0
```

补充：4 个文件一次性合并执行（最终快照）

```
$ python3 scripts/check-zh-docs.py check --files <上述 4 个路径>
PASS .../mcp-servers.md en=0/172=0.0000
PASS .../plugins-reference.md en=0/61=0.0000
PASS .../skills-reference.md en=0/94=0.0000
PASS .../subagent-templates.md en=0/114=0.0000

check: 受检 4，PASS 4，FAIL 0
EXIT=0
```

说明：`check` 子命令为 fail-closed，任一 AC 失败都会打印 `[AC-x]` 明细并返回 1。上述 5 次执行均为空明细 + 退出码 0，即 AC-2 / AC-4 / AC-6 / AC-7b / AC-32 五项全部通过。4 个文件均无 YAML frontmatter（首行不是 `---`），因此 AC-6 键名集合为空集恒等，`description` 无变更可列。

## 3. gate FAIL 列表快照

```
$ python3 scripts/check-zh-docs.py gate --base 3ee655e3 2>&1 | grep -E '^    FAIL '
    FAIL README.md [AC-2] en=388/409=0.9487
GREP_EXIT=0
```

判读：FAIL 列表中**只剩 `README.md`**（他人任务文件）。本次负责的 4 个文件 `mcp-servers.md`、`plugins-reference.md`、`skills-reference.md`、`subagent-templates.md` 均已不在 FAIL 列表中。

## 4. AC-5 校验命令与输出

```
$ git diff 3ee655e3 -- crates/rustcode-review/rules/
（无输出）
AC5a_lines=0
```

- AC-5a：`crates/rustcode-review/rules/` 相对基线零改动（输出 0 行），红线守住。

```
$ git diff -U0 3ee655e3 -- crates/rustcode-capabilities/assets/setup-seeds/ | grep -E '^[+-]name:'
（无输出）
GREP_EXIT=1 (1=无命中，符合预期)
```

- AC-5b：`setup-seeds/` 的 `-U0` diff 中没有任何 `^[+-]name:` 行，运行时 prompt 载荷的 `name:` 键未被改动。

```
$ git diff --name-only 3ee655e3 -- crates/rustcode-capabilities/assets/setup-seeds/
crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/SKILL.md
crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/hooks-patterns.md
crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/mcp-servers.md
crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/plugins-reference.md
crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/skills-reference.md
crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/subagent-templates.md
```

- 前 2 个为上一批产物（本次未写入）；后 4 个为本次 files_owned。

## 5. 遇到的取舍与决策

### 5.1 mcp-servers.md 为什么之前没达标

不是"遇到不可译内容"，而是**上一批只做了一半**：文件第 1–84 行已译完，第 85 行（`### Neon MCP（serverless Postgres）`）到文末第 263 行仍是英文，共 103 行 offender，占 172 行的 59.88%。剩余部分全部是可自由翻译的散文、表格单元格与标题（GitHub / GitLab / Linear / AWS / Cloudflare / Sentry / Slack / Notion / Docker / Kubernetes 等专有名词保留原文即可），不存在"不可译的英文代码块"——该文件通篇没有 fenced code block。因此本次按 HANDOFF 铁律 §1 直接补齐，**没有删除任何代码块或内容去凑阈值**。

### 5.2 踩到并已修掉的坑：全角顿号不算 CJK（HANDOFF §5 第 2 条）

第一版 `### Cloudflare MCP（Workers、Pages、R2、D1）` 被判为 en 行。原因：`、` 是 U+3001，不在脚本 `CJK_RE` 的判定区间（`\u3400-\u4dbf`、`\u4e00-\u9fff`、`\uf900-\ufaff`、`\u3040-\u30ff`）内，同理全角逗号/句号/冒号/括号也不算。修正为 `### Cloudflare MCP（Workers、Pages、R2、D1 服务）`，补入真实汉字后 en 归零。此坑在其余三个文件中已预先规避：凡需要"补中文"的位置都补的是汉字，而非只加全角标点。

### 5.3 为了让专有名词与"必须有汉字"并存而做的措辞取舍

- **`plugins-reference.md` 的 LSP 表格**：第二列只有语言名（`TypeScript/JavaScript`、`Python`、`Go`、`Rust`、`C/C++`、`Java`、`Kotlin`、`Swift`、`C#`、`PHP`、`Lua`），属铁律 §2.3 明确保留原文的专有名词，但整行会因为没有 CJK 而被判 en。取舍：在列值后追加中文类别词"语言"，写成 `| **typescript-lsp** | TypeScript/JavaScript 语言 |`。语言名本身一字未改。
- **`skills-reference.md` 的 `**SKILL.md:**` 标签**（共 10 处）：`SKILL.md` 会命中 `[A-Za-z]{4,}`，但该串在基线中不是 inline code，若改写成 `` `SKILL.md` `` 会**新增** code span 直接触发 AC-4 FAIL。取舍：改为 `**SKILL.md 内容：**`，补汉字、不新增反引号。同理 `**openapi-template.yaml:**`、`**scripts/validate-migration.sh:**`、`**checklist.md:**` 一律改为 `…… 内容：`。
- **`subagent-templates.md` 的 `**Best for** / **Value** / **Model** / **Tools**`**：沿用同目录 `mcp-servers.md` 已通过样本的译法，统一译为 `**适用场景** / **价值** / **模型** / **工具**`，保证 6 个参考文件术语一致。
- **跨行反引号对（HANDOFF §5 第 6 条）**：`skills-reference.md` 第 55–57 行是跨 3 行的引用块，其中 `` `/guide` `` 与 `` `/guide <question>` `` 各自成对且未跨行。本次保持原有三行折行、保持 `>` 前缀与反引号位置不变，只替换自由散文，逐行补入汉字（含最后一行 `` `/guide <question>` 即可，无需安装插件。``，其中 inline code 内容 `<question>` 未译）。AC-4 已验证 PASS。

### 5.4 明确保留、未翻译的内容

fenced code block 全部逐字保留（含其中英文注释）；inline code 内容一字未改（`.mcp.json`、`~/.rustcode/mcp.json`、`rustcode --mcp-debug`、`@supabase/supabase-js`、`.git`、`ABC-123`、`@aws-sdk/*`、`@sentry/*`、`.linear`、`pg`、`postgres`、`docker-compose.yml`、`@anthropic-ai/sdk`、`/skills ask`、`/plugin install <name>`、`/plugin marketplace add <your-channel-marketplace-url>`、`/guide <question>`、`$ARGUMENTS`、`ARGUMENTS: <value>`、`` !`command` ``、各 skill/command/agent/plugin/crate 名、链接目标等）。链接目标未变、未新增 Emoji、未改文件名与标题层级、未改 frontmatter 键名。

## 6. 已知风险

1. **语义可读性代价（低风险）**：为同时满足"保留专有名词"与"每行必须有 CJK"，LSP 表格的"语言"后缀与 mcp-servers 若干标题的中文括注（如"serverless Postgres 数据库"）在纯中文语境下略显冗余。若后续验收方认为影响观感，可在不触碰专有名词的前提下微调括注措辞，但调整后必须重跑 check 确认 en 仍为 0。
2. **AC-2 判定对全角标点的敏感性（中风险，全局性）**：脚本只认四个 CJK 区间的汉字/假名，全角标点不算。同批次其他文件的负责人若不知此坑，容易残留"看似已汉化"的 en 行。建议把 §5.2 补进 `HANDOFF-汉化铁律.md` §5 的踩坑清单。
3. **与其他批次的分工边界（低风险）**：`gate` 的 FAIL 列表目前只剩 `README.md`（另有 `doc-writer.md` 可能出现在他人的报告中），均非本批 files_owned，不应计入 D-34b。
4. **未执行构建/测试**：按项目约束本次未运行 `cargo`/`npm`。本次为纯文档改动，不涉及源码行为；但运行时会读取这些 prompt 载荷，建议在整体门禁通过后由测试方做一次 setup-seeds 冒烟，确认汉化未影响推荐逻辑的关键字匹配（依赖的是 inline code 内的包名/路径，本次均未改动，理论无影响）。

## 7. 未验证范围

- 未验证 RustCode 运行时实际加载这 4 个参考文件后的推荐效果（无构建/运行权限，且本任务不要求）。
- 未验证除 `check-zh-docs.py` 之外的其他门禁脚本（如可能存在的 markdown lint）对上述改动的态度。
- 未对 `SKILL.md`、`hooks-patterns.md` 做二次核对（非本批 files_owned，本次未写入）。
