# zh-check 报告

- 基线 (--base): `3ee655e3`
- 阈值 (--en-ratio): `0.05`
- 判定: AC-2 / AC-4 / AC-6 / AC-7b / AC-32，任一失败即 FAIL（fail-closed）

## 汇总

- 受检文件: 1
- PASS: 1
- FAIL: 0

## PASS `docs/superpowers/plans/2026-05-29-webui.md`

- en=0/502 ratio=0.0000

---

# 手写小结 · D-02 `docs/superpowers/plans/2026-05-29-webui.md`

作者：doc-writer ｜ 基线：`3ee655e3` ｜ 受检文件：`docs/superpowers/plans/2026-05-29-webui.md`（1958 行 / 73.01 KB）

## 1. 改动量

共改动 **77 行**，全部为行内字符串替换：**无新增行、无删除行**（改动前后均为 1958 行），
`git diff --stat` 显示 `77 insertions(+), 77 deletions(-)`。

| 类别 | 处数 | 改前 → 改后 |
|---|---|---|
| 文件清单标题 | 27 | `**Files:**` → `**文件：**` |
| 文件项标签 | 12 | `- Create: ` → `- 新建：` |
| 文件项标签 | 30 | `- Modify: ` → `- 修改：` |
| 文件项标签 | 1 | `- Create/Modify: ` → `- 新建/修改：` |
| 标题/整行翻译 | 7 | 见下表 |

7 处整行/标题翻译（行号为改动后文件内的行号）：

| 位置 | 改前 → 改后 |
|---|---|
| `2026-05-29-webui.md:1` | `# rustcode webui (Phase 1) Implementation Plan` → `# rustcode webui（Phase 1）实施计划` |
| `2026-05-29-webui.md:3` | agent 执行者提示整段英译中（`superpowers:subagent-driven-development`、`superpowers:executing-plans` 技能名与 `` `- [ ]` `` 原样保留） |
| `2026-05-29-webui.md:13` | `**Spec:**` → `**Spec（设计文档）：**`（链接/路径原文不变） |
| `2026-05-29-webui.md:17` | `## File Structure` → `## 文件结构`（层级不变） |
| `2026-05-29-webui.md:780` | ``Expected: `OK`。`` → ``Expected: 输出 `OK`。`` |
| `2026-05-29-webui.md:1529` | `…重定向到 vite dev server）` → `…重定向到 vite 开发服务器）` |
| `2026-05-29-webui.md:1532` | ``**Step 1: `/webui stop`**`` → ``**Step 1: `/webui stop` 停止进程内 server**`` |
| `2026-05-29-webui.md:1769` | ``### Task 20: server `/live` SSE + `POST /live/message` `` → 末尾补 `端点`（`` `/live` `` 与 `` `POST /live/message` `` 原文不变） |

（1529 行同时属于 `- Modify:` → `- 修改：` 的 30 处之一，故 70 + 7 = 77。）

**未触碰的内容**：所有 fenced code block（rust / toml / tsx / ts / bash / html）与所有 inline code
一字未改；已用脚本逐行核对「改动行是否落在代码块内」——**77 行改动，0 行在代码块内**。
链接目标、锚点、Emoji 均未改动（本文件原本就没有任何 `](...)` 链接，也没有 Emoji）。

## 2. 残留行清单（及「为何属于白名单」）

AC-2 统计意义上的英文残留行为 **0 行**（改动前 53 行 → 改动后 0 行）。
以下英文片段仍出现在**含中文的行**里，按铁律属于保留项，故不计入 AC-2 也不做改写：

1. **计划模板固化标记**：`Step N:`（127 处）、`Run: `（31 处）、`Expected: `（33 处）。
   这些是 superpowers 计划模板的步骤标记，且本仓库已汉化的同类计划
   （`docs/superpowers/plans/2026-04-23-merge-current-task-into-cadence.md`、
   `docs/superpowers/plans/2026-04-19-tuix-retained-mode-rewrite.md`）同样保留原文；
   铁律 9「已中文的段落不润色、不改写」，故不动。
2. **`2026-05-29-webui.md:938` 的英文 i18n 文案** `"Launch the browser webui"`：这是要写进
   `crates/rustcode-core/src/i18n/en.rs` 的实际英文字符串，翻译即破坏实现语义；按铁律 3 保留原文。
   特意**未**给它加反引号——新增 inline code 会改变 code span 多重集并导致 AC-4 FAIL。
3. **第三方/技术专有名词**：`Rust`、`axum`、`tokio`、`Preact`、`Vite`、`Tailwind`、`rust-embed`、
   `OpenAI`/`Anthropic`/`Ollama`/`VS Code`、`SPA`、`fallback`、`loopback`、`token`、`Bearer`、`SSE`、
   `mockup` 等，按铁律 3 保留原文。
4. **路径 / crate 名 / 命令 / 环境变量 / 接口签名**：`rustcode-daemon`、`rustcode-cli`、
   `rustcode_core::turn::permission::InteractivePermissionDecider`、`RUSTCODE_WEBUI_DEV`、
   `--host`、`/webui`、`/chat/permission`、`Msg::Xxx` 等，全部原样。
5. **无 YAML frontmatter**：本文件第 1 行不是 `---`，故 AC-6 的 frontmatter 键名集合为空集对空集，
   判定不适用但恒等（脚本未报 AC-6 FAIL）。

## 3. 四项实测结果（命令：`python3 scripts/check-zh-docs.py check --base 3ee655e3 --files docs/superpowers/plans/2026-05-29-webui.md`）

| 项 | 阈值 | 实测 | 结论 |
|---|---|---|---|
| AC-2 英文行占比 | `en/total <= 0.05` | **0 / 502 = 0.0000**（改动前 53 / 502 = 0.1056） | PASS |
| AC-4 inline code + fenced code 集合 | 与基线完全相等 | 基线 1654 项 / 工作区 1654 项，`==` 为 True | PASS |
| AC-7b `](...)` 链接目标多重集 | 与基线相等 | 基线 0 项 / 工作区 0 项（本文件无 markdown 链接），`==` 为 True | PASS |
| AC-32 Emoji 数 | 不增加 | 基线 0 → 工作区 0 | PASS |

脚本整体输出：`PASS docs/superpowers/plans/2026-05-29-webui.md en=0/502=0.0000`、
`check: 受检 1，PASS 1，FAIL 0`，退出码 `0`。

## 4. 结论

D-02 完成：英文正文已纯中文替换（无中英并列、无英文原文段落），四项门禁全 PASS，
代码块 / inline code / 链接 / Emoji 零改动，无 TODO 与占位符残留。
