# zh-check 报告

- 基线 (--base): `3ee655e3`
- 阈值 (--en-ratio): `0.05`
- 判定: AC-2 / AC-4 / AC-6 / AC-7b / AC-32，任一失败即 FAIL（fail-closed）

## 汇总

- 受检文件: 7
- PASS: 7
- FAIL: 0

## PASS `docs/async-webhook-guide.md`

- en=0/108 ratio=0.0000

## PASS `docs/agent-api-rfc.md`

- en=0/141 ratio=0.0000

## PASS `docs/webhook-guide.md`

- en=0/108 ratio=0.0000

## PASS `docs/hook-cli-guide.md`

- en=0/111 ratio=0.0000

## PASS `docs/architecture.md`

- en=0/122 ratio=0.0000

## PASS `docs/async-webhook-summary.md`

- en=0/125 ratio=0.0000

## PASS `docs/hook-expansion-summary.md`

- en=0/101 ratio=0.0000

---

# D-22 手写小结（追加于最后一次自检之后）

- 任务 ID：`D-22`（批次 8）
- 自检命令（最后一次执行，本报告的上半段即其产物）：

```bash
python3 scripts/check-zh-docs.py check --base 3ee655e3 \
  --report .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-22.md \
  --files docs/async-webhook-guide.md docs/agent-api-rfc.md docs/webhook-guide.md \
          docs/hook-cli-guide.md docs/architecture.md docs/async-webhook-summary.md \
          docs/hook-expansion-summary.md
```

- 结果：受检 7，PASS 7，FAIL 0；7 个文件改动后 en 全部归零（ratio 均为 0.0000）。

## 1. 动手前的自检结论（区分两类文件）

开工前先跑了一次自检，基线 `3ee655e3` 上的 offender 清单为：

| 文件 | 改动前 en/total | 判定 |
|---|---|---|
| `docs/async-webhook-guide.md` | 0/108 = 0.0000 | **已中文且无 offender → no-op** |
| `docs/agent-api-rfc.md` | 0/141 = 0.0000 | **已中文且无 offender → no-op** |
| `docs/webhook-guide.md` | 1/108 = 0.0093 | 已中文，按本批说明「已中文只需清 offender」清残留 |
| `docs/hook-cli-guide.md` | 0/111 = 0.0000 | **已中文且无 offender → no-op** |
| `docs/architecture.md` | 3/122 = 0.0246 | 已中文，同上 |
| `docs/async-webhook-summary.md` | 0/125 = 0.0000 | **已中文且无 offender → no-op** |
| `docs/hook-expansion-summary.md` | 0/101 = 0.0000 | **已中文且无 offender → no-op** |

本批 **7 个文件全部属于「已中文」一类，没有「纯英文需全文译」的文件**：7 个文件在基线
`3ee655e3` 上的正文均已为中文（代码块密度为中-高，散文部分早已汉化完毕），
`check` 共报出 **4 行**英文 offender，全部是「纯术语构成的标题行」。

## 2. 逐文件改动量

| 文件 | 改动行数 | 说明 |
|---|---|---|
| `docs/async-webhook-guide.md` | 0 | **no-op**。en=0，无 offender 可清，按铁律 §3 一字未动 |
| `docs/agent-api-rfc.md` | 0 | **no-op**。en=0，无 offender 可清，一字未动 |
| `docs/hook-cli-guide.md` | 0 | **no-op**。en=0，无 offender 可清，一字未动 |
| `docs/async-webhook-summary.md` | 0 | **no-op**。en=0，无 offender 可清，一字未动 |
| `docs/hook-expansion-summary.md` | 0 | **no-op**。en=0，无 offender 可清，一字未动 |
| `docs/webhook-guide.md` | 1 行（+1/-1） | 清 1 处 offender：`:407` `### hooks.toml` → `### hooks.toml（完整配置文件）` |
| `docs/architecture.md` | 4 行（+4/-4） | 清 3 处 offender + 1 处同级一致性补齐（见 §3）：<br>`:1` `# RustCode Architecture` → `# RustCode 架构`；<br>`:122` `### CLI / TUI` → `### CLI / TUI（命令行入口与终端界面）`；<br>`:128` `### daemon / WebUI` → `### daemon / WebUI（守护进程与 Web 界面）`；<br>`:133` `### ACP / clix / background` → `### ACP / clix / background（协议入口、独立命令行驱动与后台任务）` |

合计 **2 个文件、5 行纯行内替换**（`git diff --numstat` 为 `4/4` 与 `1/1`，`+N` 与 `-N`
严格相等），**未增删空行、未改段落结构、未改标题层级**；7 个文件总行数保持
503 / 308 / 455 / 424 / 205 / 395 / 306，与基线逐行一致。

## 3. 关于 `docs/architecture.md:122` 这行「非 offender」改动

`### CLI / TUI` 未被 `check` 报出，原因是脚本的 `ASCII4_RE` 要求 `[A-Za-z]{4,}`，
而 `CLI` / `TUI` 各只有 3 个字母，不构成 offender。但它与同一节 `## Driver 与服务边界`
下的 `:128`、`:133` 是**同一类英文标题**：若只改后两者，会出现「两个中文括注标题夹一个
纯英文标题」的不一致外观。因此按铁律 §1（英文正文译为中文）而非 §3（不得改写已中文段落）
处理，补上中文括注。**这是本批唯一一处超出 `check` offender 清单的改动**，特此显式声明。

## 4. 保留原文的处置方式（本批特殊说明 §2 / §3）

- **crate 名**（`rustcode-kernel` / `rustcode-capabilities` / `rustcode-coding` /
  `rustcode-cli` / `rustcode-tuix` / `rustcode-daemon` / `rustcode-clix` 等）、
  **分层名**（`L0` / `L1` / `L2` / `L3`、`drivers` / `specialize` / `capabilities` /
  `neutral` / `leaf`）、**依赖方向箭头**一律保留原文：这些符号本就位于 fenced 代码块内
  （`:32`–`:47` 的分层图）或表格内，本批**零触碰**。
- **hook 事件名**（`PreToolUse` / `PostToolUse` / `turn_complete` / `session_end` /
  `tool_call_start` / `post_tool` 等）、**webhook 字段名与 JSON 键名**（`hook_name` /
  `trigger` / `event` / `context` / `modified_content` / `batch_size` /
  `flush_interval_ms` / `timeout_secs` / `retries` / `enabled` 等）、
  **HTTP 方法与 API 路径**（`GET /agent/tasks/{id}/stream`、`POST /agent/tasks/{id}/cancel`）、
  **配置文件名**（`hooks.toml`）一律保留原文，未给它们套反引号（铁律 §2.2）。
- 三个英文术语标题采用「**英文原名 + 全角括号中文括注**」的形式补齐中文，括注内写的是
  真实汉字（「命令行入口与终端界面」「守护进程与 Web 界面」「协议入口、独立命令行驱动与
  后台任务」「完整配置文件」），**不是**只加全角标点（铁律 §5.2）。
- 标题层级（`#` / `###`）与文件名均未改动；未新增、删除或改写任何 Markdown 链接与锚点
  （铁律 §4、§5）。改动前已用 `git grep` 确认全仓**没有**以
  `#rustcode-architecture` / `#daemon--webui` / `#acp--clix--background` / `#hooks-toml`
  形式引用这些标题的锚点链接，故改标题文字不会造成断链。
- 7 个文件**均无 YAML frontmatter**（首行非 `---`），故 AC-6 键名集合前后皆为空集（相等），
  且**无** `description` 需要汉化 —— 铁律 §7 的「逐条列示」清单为**空**。

## 5. 残留行清单与白名单判定

- **AC-2 残留 en 行：0 行。** 改动后 7 个文件 en 计数全部为 0，故无需申请 AC-2 白名单。
- 另用一次性只读脚本（存放在 `/tmp`，**未写入仓库**）做了两轮兜底扫描，结果如下：
  1. **中英混排扫描**（英文词 ≥5 且汉字占比 <0.8）：仅 `docs/architecture.md` 命中 9 行
     （如 `:27`、`:64`、`:100`、`:110`、`:112`、`:181`、`:185`、`:186`、`:198`）。
     这些行**已是中文散文**，英文部分全是 `provider` / `session` / `goal` / `loop` /
     `submit` / `steer` / `cancel` / `compact` 等按铁律 §2.3 必须保留的术语与操作名，
     且均含真实汉字，按铁律 §8 **不润色、不改写**。
  2. **严格扫描**（剥离 inline code / URL 后仍不含 CJK 且含 ASCII 字母，不限 4 字母）：
     改动前命中 2 行 —— `docs/architecture.md:122`（已修，见 §3）与
     `docs/agent-api-rfc.md:159`。
- **白名单条目（1 条）**：`docs/agent-api-rfc.md:159`
  `` `GET /agent/tasks/{id}/stream` (SSE, `text/event-stream`) ``。
  剥离 inline code 后只剩 `(SSE, )`，`SSE` 仅 3 个字母，不触发 `ASCII4_RE`，**本来就不是
  AC-2 offender**；且 `SSE` 与 `HTTP` 同属协议缩略语，`text/event-stream` 是 MIME
  类型，均按铁律 §2.3 保留原文。该行主体是 API 路径与媒体类型，改写反而会破坏代码 span，
  故**维持原样并列入白名单**。
- 本批**未出现**跨行反引号对（铁律 §5.6）：5 处改动全部是标题行末尾追加中文括注，
  反引号位置与折行位置一字未动。

## 6. 四项实测结果

| 项 | 结果 | 实测依据 |
|---|---|---|
| **AC-2** `en/total <= 0.05` | **PASS**（7/7） | 改动后 en/total 为 0/108、0/141、0/108、0/111、0/122、0/125、0/101，ratio 全为 0.0000 |
| **AC-4** inline code + fenced code 多重集相等 | **PASS**（7/7） | 最后一次 `check` 无 AC-4 FAIL 输出；5 行改动**未新增/删除任何反引号**，`git diff -U0` 逐行核对确认仅替换标题文字，fenced 代码块零改动 |
| **AC-7b** `](...)` 链接目标多重集相等 | **PASS**（7/7） | 最后一次 `check` 无 AC-7b FAIL 输出；本批未新增、删除或改写任何 Markdown 链接 |
| **AC-32** Emoji 数不增加 | **PASS**（7/7） | 最后一次 `check` 无 AC-32 FAIL 输出；本批新增字符仅为汉字与全角括号「（）」，未引入任何 Emoji |

补充实测：`git diff --numstat` 显示 2 个改动文件的 `+N` 与 `-N` 完全相等（`4/4`、`1/1`），
无空行增删（铁律 §5.4）；`wc -l` 显示 7 个文件行数与基线**完全一致**
（503 / 308 / 455 / 424 / 205 / 395 / 306）。

## 7. 边界声明

- 只改 `files_owned` 内 7 个文件中的 2 个；另外 5 个判定 no-op 后**一字未动**（铁律 §3、§8）。
- 未碰 `docs/` 根级其它文件（工作区中 `docs/` 下的其它改动属其它批次，非本批产物）。
- 未改源码、未执行 `cargo`/`npm` 构建与测试、未做 git 写操作（commit/stash/tag）。
- 已中文段落未做润色或改写；改动严格限于英文标题行（含 §3 显式声明的 1 处一致性补齐）。
