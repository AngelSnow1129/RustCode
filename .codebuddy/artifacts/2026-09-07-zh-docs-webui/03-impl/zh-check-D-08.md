# zh-check 报告

- 基线 (--base): `3ee655e3`
- 阈值 (--en-ratio): `0.05`
- 判定: AC-2 / AC-4 / AC-6 / AC-7b / AC-32，任一失败即 FAIL（fail-closed）

## 汇总

- 受检文件: 8
- PASS: 8
- FAIL: 0

## PASS `docs/superpowers/plans/2026-07-22-request-user-input-custom-and-submit-spacing.md`

- en=0/138 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-29-webui-sync-compact.md`

- en=0/98 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-24-skills-multi-compose.md`

- en=0/77 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-25-retire-core-provider-C2-conversation-transport.md`

- en=0/67 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-25-retire-core-provider-A-compact.md`

- en=0/66 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-24-windows-native-tls-schannel-fallback.md`

- en=0/65 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-31-project-memory-dir-override.md`

- en=0/74 ratio=0.0000

## PASS `docs/superpowers/plans/2026-04-23-agent-harness-principles.md`

- en=0/120 ratio=0.0000

---

# D-08 汉化小结（人工撰写）

任务：把 `docs/superpowers/plans/` 下 8 个计划文档的英文正文译为简体中文，纯中文替换、不做中英并列、不保留英文原文段落。基线 `3ee655e3`，受检文件即本次 `files_owned` 的 8 篇。

## 1. 每个文件改动量

| 文件 | 总行数 | 改动行（+/-，vs `3ee655e3`）| 汉化前 en/total | 汉化后 en/total | fence 行数 |
|---|---|---|---|---|---|
| `docs/superpowers/plans/2026-07-22-request-user-input-custom-and-submit-spacing.md` | 473 | +130 / -130 | 120/138 = 0.8696 | 0/138 = 0.0000 | 36 |
| `docs/superpowers/plans/2026-07-29-webui-sync-compact.md` | 267 | +55 / -55 | 13/98 = 0.1327 | 0/98 = 0.0000 | 20 |
| `docs/superpowers/plans/2026-07-24-skills-multi-compose.md` | 290 | +47 / -47 | 11/77 = 0.1429 | 0/77 = 0.0000 | 16 |
| `docs/superpowers/plans/2026-07-25-retire-core-provider-C2-conversation-transport.md` | 105 | +35 / -35 | 7/67 = 0.1045 | 0/67 = 0.0000 | 0 |
| `docs/superpowers/plans/2026-07-25-retire-core-provider-A-compact.md` | 191 | +38 / -38 | 9/66 = 0.1364 | 0/66 = 0.0000 | 14 |
| `docs/superpowers/plans/2026-07-24-windows-native-tls-schannel-fallback.md` | 191 | +40 / -40 | 8/65 = 0.1231 | 0/65 = 0.0000 | 26 |
| `docs/superpowers/plans/2026-07-31-project-memory-dir-override.md` | 211 | +51 / -51 | 10/74 = 0.1351 | 0/74 = 0.0000 | 14 |
| `docs/superpowers/plans/2026-04-23-agent-harness-principles.md` | 246 | +25 / -25 | 8/120 = 0.0667 | 0/120 = 0.0000 | 8 |

改动量取自 `git diff --numstat 3ee655e3 -- <file>`。8 个文件的 `+` 与 `-` 行数**逐项相等**，即全部为**纯行内替换**：总行数不变、标题层级不变、文件名与计划日期未改动。合计改动 429 行。

其中 7 篇是「局部英文残留」型（基线已为中文，en_ratio 0.0667 ~ 0.1429），只有 `2026-07-22-request-user-input-custom-and-submit-spacing.md` 是整体英文正文（0.8696），该篇做了整篇逐段翻译。

## 2. 残留行清单及「为何属于白名单」

**AC-3 残留行清单（en 行）：8 个文件全部为 0 行。** 上方各文件节中未打印「AC-3 残留行清单」，正是因为 offenders 为空；「R5 跳过的行」同样全部为 0 行，因此不存在被豁免掉的英文正文，也不存在靠缩进/引用块规则蒙混过关的散文行。

仍以英文出现的部分全部属于白名单，理由如下：

1. **fenced code block（``` / ~~~）内全部内容** —— 铁律 1：含注释与字符串一个字符都未改动。上表 fence 行数与基线逐项一致（36/20/16/0/14/26/14/8）。例如 `2026-07-24-windows-native-tls-schannel-fallback.md` 中 toml/rust 代码块里的英文注释、`2026-04-23-agent-harness-principles.md` 中两条 ASCII 架构图（原则 1-4 的层次图）均原样保留。
2. **inline code（反引号）内容** —— 铁律 2。AC-2 的 R4 会先剥离 inline code 再判定，AC-4 则强制其与基线多重集相等（见第 3 节，8 个文件全部逐项相等）。
3. **文件路径、目录名、crate 名、命令与 CLI flag、环境变量 `RUSTCODE_*`、配置键/TOML 表名、HTTP 方法与 API 路径、`Msg::Xxx` 变体名、枚举变体、函数名/类型名/工具名/hook 名、第三方专有名词（Rust、Tokio、Cargo、reqwest、Schannel、Windows、OpenAI、axum、Preact、React、SSE、dogfooding 等）、版本号/日期/数字** —— 铁律 3，逐字保留。例如 `RUSTCODE_PROJECT_MEMORY_DIR`、`POST /live/compact`、`DriverCommand::Compact(None)`、`Msg::SkillsLoaded { names: &'a str }`、`release/v5.0.3`、`Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`。
4. **代码标识符类的英文单词** —— 即使在中文句中也保留原样，如 `custom`、`expand_skill`、`split_skill_names`、`last_row`、`user_invocable`、`build_user_input_rows`，均未译义。
5. **本就为中文的段落** —— 铁律 8，未润色、未改写。仅对其中的结构性英文小标题/标签做了术语统一（`Files`→`文件`、`Interfaces`→`接口`、`Consumes`→`消费`、`Produces`→`产出`、`Tech Stack`→`技术栈`、`Global Constraints`→`全局约束`、`Self-Review`→`自审`、`Spec coverage`→`规格覆盖`、`Placeholder scan`→`占位符扫描`、`Type consistency`→`类型一致性`、`Task N`→`任务 N`、`Step N`→`步骤 N`、`Run`→`运行`、`Expected`→`预期`、`What/Why/How/Status/Related plans/Revision policy`→`是什么/为什么/怎么做/状态/相关计划/修订策略`），与 D-01~D-07 已交付文档的口吻保持一致。
6. **链接** —— 铁律 4 在本次无实际对象：8 个文件基线中 `](...)` 链接目标均为 0 条，改后仍为 0 条。
7. **Emoji** —— 铁律 6：8 个文件基线 Emoji 命中数均为 0，改后仍为 0，未新增也未清理。

### 本批实际踩到并已修掉的两个坑

- **全角标点不算 CJK**：脚本的 `CJK_RE` 只认汉字/假名区间，`（）`、`、`、`。` 均不计数。因此 `**Tech Stack:** Rust（axum daemon：...）` 与 `Expected: PASS。` 这类「只有全角标点包裹英文」的行仍会被判为英文行。已通过补入真实汉字修正（如 `**技术栈：** Rust（axum 守护进程：...）`、`预期：测试通过（PASS）。`）。
- **不要给普通词加反引号**：本批全程未新增反引号，故 AC-4 一次通过（见第 3 节）。

## 3. 四项实测结果

| 验收项 | 口径 | 文件1 | 文件2 | 文件3 | 文件4 | 文件5 | 文件6 | 文件7 | 文件8 | 结论 |
|---|---|---|---|---|---|---|---|---|---|---|
| **AC-2** | `en/total <= 0.05` | 0/138 | 0/98 | 0/77 | 0/67 | 0/66 | 0/65 | 0/74 | 0/120 | **PASS** |
| **AC-4** | inline code + fenced code 多重集与基线相等 | 446→446 | 225→225 | 259→259 | 124→124 | 175→175 | 139→139 | 180→180 | 80→80 | **PASS** |
| **AC-7b** | `](...)` 链接目标多重集相等 | 0→0 | 0→0 | 0→0 | 0→0 | 0→0 | 0→0 | 0→0 | 0→0 | **PASS** |
| **AC-32** | Emoji 命中数不增加 | 0→0 | 0→0 | 0→0 | 0→0 | 0→0 | 0→0 | 0→0 | 0→0 | **PASS** |

文件列序同第 1 节表格（文件1 = `2026-07-22-request-user-input-custom-and-submit-spacing.md`，依次至文件8 = `2026-04-23-agent-harness-principles.md`）。

附带项：**AC-6** frontmatter 键名集合 —— 8 个文件均无 YAML frontmatter（与铁律 7 一致），键名集合为空集且前后相等，PASS。

## 4. 复现命令与实测输出

```bash
python3 scripts/check-zh-docs.py check --base 3ee655e3 \
  --report .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-08.md \
  --files docs/superpowers/plans/2026-07-22-request-user-input-custom-and-submit-spacing.md \
          docs/superpowers/plans/2026-07-29-webui-sync-compact.md \
          docs/superpowers/plans/2026-07-24-skills-multi-compose.md \
          docs/superpowers/plans/2026-07-25-retire-core-provider-C2-conversation-transport.md \
          docs/superpowers/plans/2026-07-25-retire-core-provider-A-compact.md \
          docs/superpowers/plans/2026-07-24-windows-native-tls-schannel-fallback.md \
          docs/superpowers/plans/2026-07-31-project-memory-dir-override.md \
          docs/superpowers/plans/2026-04-23-agent-harness-principles.md
```

实测输出（8 个文件一起跑，退出码 0）：

```text
PASS docs/superpowers/plans/2026-07-22-request-user-input-custom-and-submit-spacing.md en=0/138=0.0000
PASS docs/superpowers/plans/2026-07-29-webui-sync-compact.md en=0/98=0.0000
PASS docs/superpowers/plans/2026-07-24-skills-multi-compose.md en=0/77=0.0000
PASS docs/superpowers/plans/2026-07-25-retire-core-provider-C2-conversation-transport.md en=0/67=0.0000
PASS docs/superpowers/plans/2026-07-25-retire-core-provider-A-compact.md en=0/66=0.0000
PASS docs/superpowers/plans/2026-07-24-windows-native-tls-schannel-fallback.md en=0/65=0.0000
PASS docs/superpowers/plans/2026-07-31-project-memory-dir-override.md en=0/74=0.0000
PASS docs/superpowers/plans/2026-04-23-agent-harness-principles.md en=0/120=0.0000

check: 受检 8，PASS 8，FAIL 0
```

> 注意：`--report` 会**覆盖**已存在的报告文件。上面的小结是在最后一次自检**之后**追加的；若需复跑自检，请复跑后重新追加以免覆盖本節。

## 5. 结论

D-08 批次 8 个文件全部通过 AC-2 / AC-4 / AC-6 / AC-7b / AC-32，无残留英文正文行，无残留 TODO 或占位符，未新增 Emoji，未改动代码与链接，未改动文件名与标题层级。可以提交验收。
