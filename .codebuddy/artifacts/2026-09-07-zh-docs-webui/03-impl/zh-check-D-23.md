# zh-check 报告

- 基线 (--base): `3ee655e3`
- 阈值 (--en-ratio): `0.05`
- 判定: AC-2 / AC-4 / AC-6 / AC-7b / AC-32，任一失败即 FAIL（fail-closed）

## 汇总

- 受检文件: 15
- PASS: 15
- FAIL: 0

## PASS `docs/hooks.md`

- en=0/112 ratio=0.0000

## PASS `docs/dev-env-setup.md`

- en=3/61 ratio=0.0492
- AC-3 残留行清单 (en 行，共 3 行):
  - `docs/dev-env-setup.md:26`: `| Node.js | v24.20.0 (LTS) | nvm | '~/.nvm/' |`
  - `docs/dev-env-setup.md:97`: `### 3.2 Node.js（nvm）`
  - `docs/dev-env-setup.md:135`: `> [!WARNING]`

## PASS `docs/hook-implementation-summary.md`

- en=0/102 ratio=0.0000

## PASS `docs/hook-timing-complete.md`

- en=0/127 ratio=0.0000

## PASS `docs/target-architecture.md`

- en=0/70 ratio=0.0000

## PASS `docs/i18n-field-mapping.md`

- en=0/58 ratio=0.0000

## PASS `docs/REFACTOR_SUMMARY.md`

- en=0/44 ratio=0.0000

## PASS `docs/i18n-style.md`

- en=0/69 ratio=0.0000

## PASS `docs/vscode-i18n-implementation-plan-2026-06-30.md`

- en=0/71 ratio=0.0000

## PASS `docs/features.md`

- en=0/55 ratio=0.0000

## PASS `docs/custom-endpoint-guide.md`

- en=0/51 ratio=0.0000

## PASS `docs/pr-hook-test-command.md`

- en=0/37 ratio=0.0000

## PASS `docs/codex-claude-config-analysis.md`

- en=0/33 ratio=0.0000

## PASS `docs/telemetry.md`

- en=0/28 ratio=0.0000

## PASS `docs/acp-sdk-handler-notes.md`

- en=0/23 ratio=0.0000

---

# D-23 手写小结（doc-writer）

## 一、执行顺序

先跑自检拿 offender 清单（首轮：PASS 7 / FAIL 8），把 15 个文件分成两类处理：

- **纯英文需全文译**（`en/total` 接近 1）：`telemetry.md`、`vscode-i18n-implementation-plan-2026-06-30.md`、`REFACTOR_SUMMARY.md`、`acp-sdk-handler-notes.md`；
- **已中文只需清 offender**（含混排）：`hooks.md`、`hook-implementation-summary.md`、`target-architecture.md`、`i18n-field-mapping.md`；
- **已达标 no-op**（`en/total <= 0.05`）：`dev-env-setup.md`、`hook-timing-complete.md`、`i18n-style.md`、`features.md`、`custom-endpoint-guide.md`、`pr-hook-test-command.md`、`codex-claude-config-analysis.md`。

## 二、逐文件改动量

| 文件 | 判定 | 增删行 | 说明 |
|---|---|---|---|
| `docs/hooks.md` | 混排清 offender | +91 / -91 | 全文英文散文、表头/表体、blockquote、安全清单、相关文档列表译为中文；代码块与 CLI 示例零改动 |
| `docs/telemetry.md` | 全文译 | +28 / -28 | 见 §五.1，否定式声明保留原意 |
| `docs/vscode-i18n-implementation-plan-2026-06-30.md` | 全文译 | +71 / -71 | 目标/现状/架构/文件/Phase 1-5/范围外/风险全部译为中文 |
| `docs/REFACTOR_SUMMARY.md` | 全文译 | +43 / -43 | 三个 OBJECTIVE、许可证、验证、冒烟步骤、已知缺口译为中文 |
| `docs/acp-sdk-handler-notes.md` | 全文译 | +20 / -20 | 见 §五.3 |
| `docs/hook-implementation-summary.md` | 清 offender | +8 / -8 | 覆盖 trait 的 7 个表格行补中文语义标注；加载顺序第 2 条补“配置/两类实现” |
| `docs/target-architecture.md` | 清 offender | +4 / -4 | 4 条英文术语清单行补“等运行时状态/等运行时命令/…”汉字收尾 |
| `docs/i18n-field-mapping.md` | 清 offender | +3 / -3 | 基线表 3 行 `PASS` → `PASS(键集合一致)`，沿用同表第 128 行既有写法 |
| `docs/dev-env-setup.md` | no-op | 0 | en=3/61=0.0492 已达标，按铁律 §3 不改写已中文段落 |
| `docs/hook-timing-complete.md` | no-op | 0 | en=0/127 |
| `docs/i18n-style.md` | no-op | 0 | en=0/69 |
| `docs/features.md` | no-op | 0 | en=0/55 |
| `docs/custom-endpoint-guide.md` | no-op | 0 | en=0/51 |
| `docs/pr-hook-test-command.md` | no-op | 0 | en=0/37 |
| `docs/codex-claude-config-analysis.md` | no-op | 0 | en=0/33 |

合计：8 个文件改动，+268 / -268。

## 三、残留行清单与白名单理由

全批仅 `docs/dev-env-setup.md` 有 3 行残留，该文件 en=3/61=0.0492 ≤ 0.05 本就 PASS，按铁律 §3 记 no-op、不改写已中文段落：

| 行 | 内容 | 白名单理由 |
|---|---|---|
| `docs/dev-env-setup.md:26` | `\| Node.js \| v24.20.0 (LTS) \| nvm \| '~/.nvm/' \|` | `Node.js`、`LTS`、`nvm` 为第三方专有名词与版本标记，路径在 inline code 内；均属铁律 3 保留项，改译会与表内其余行不一致 |
| `docs/dev-env-setup.md:97` | `### 3.2 Node.js（nvm）` | 同上，标题内仅剩专有名词；全角括号不计入 CJK，补汉字会破坏“工具名原样”的既有体例 |
| `docs/dev-env-setup.md:135` | `> [!WARNING]` | GitHub alert 语法标记，任何改写都会使告警块失效 |

其余 14 个文件 en 残行为 0。

## 四、四项实测结果

脚本为 fail-closed，以下四项在 15 个文件上逐文件执行，最终 15 PASS / 0 FAIL：

| AC | 结果 | 依据 |
|---|---|---|
| AC-2 | 15/15 PASS | 14 个文件 `en=0`；`dev-env-setup.md` en=3/61=0.0492 ≤ 0.05 |
| AC-4 | 15/15 PASS | 8 个改动文件的 inline code + fenced code 多重集与基线 `3ee655e3` 完全相等（未新增/删除任何反引号，未重排跨行反引号对） |
| AC-7b | 15/15 PASS | `](...)` 链接目标多重集相等；`hooks.md` 的 5 个 `./hook-*.md` 相对链接只译了显示文字，锚点与目标未动 |
| AC-32 | 15/15 PASS | Emoji 命中数未增加（全批未新增任何 Emoji，也未清理既有 Emoji） |
| AC-6 | 不适用 | 15 个文件均无 YAML frontmatter（首行不是 `---`），无 `description` 值需逐条列示 |

行数核对：`wc -l` 逐文件比对基线，15 个文件行数全部与基线一致（无增删空行、无段落结构变化）；`git diff --numstat` 显示 8 个改动文件均为 `+N / -N` 相等。

## 五、本批特殊说明的落实

1. **`docs/telemetry.md`（已删除遥测的说明页）**：否定式与断言英文一律保留原意——
   - 标题 `Telemetry — removed` → `Telemetry(遥测)—— 已移除`（未写成“遥测说明”这类误导措辞）；
   - `## What still exists and is NOT telemetry` → `## 仍然存在、且**并非**遥测的部分`（保留“并非遥测”的否定）；
   - `Do not remove these` → `不要删除下面这些`；
   - `**silently ignored**` → `**静默忽略**`，`**accepted and ignored**` → `**被接受但被忽略**`；
   - `no event queue, no sender, and no endpoint` → `没有 event queue,没有 sender,也没有 endpoint`；
   - `nothing leaves the machine` / `nothing is sent off-box` 分别译为“不会有任何数据离开本机”；
   - 首屏 ```` ```text ```` 块内 `[STATUS] This fork ships ZERO telemetry.` 与 `grep` 验证脚本一字未改。
2. **i18n 两件**：`Msg::Xxx` 变体名、`MsgKey`、`Lang`、`normalizeLocale`、`createTranslator`、`messages`、`package.nls*.json`、i18n key（如 `header.menu`）、locale 标记 `zh`/`zh-CN`/`zh-TW` 全部保留英文。`docs/i18n-style.md` 已为全中文，记 no-op。
3. **`docs/acp-sdk-handler-notes.md`**：crate 源码注释引文 `"The connection processes messages on a single async task..."` 不属于 fenced code block（铁律 1 只保护围栏块），为同时满足 AC-2 与可核对性，译为中文本意并以“crate 源码注释的大意是”限定，句内 `cx.spawn()`、`Channel::duplex()`、`ConnectTo<R>` 等符号原样保留。
4. **折行与反引号**：所有改动均为“行内替换”，未重排任何硬折行位置，因此不存在跨行反引号对被打散/合并导致的 AC-4 风险；`telemetry.md` 曾因中文更紧凑少 2 行，已按基线折行位置复原为 51 行。

## 六、复现命令

```bash
python3 scripts/check-zh-docs.py check --base 3ee655e3 \
  --report .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-23.md \
  --files docs/hooks.md docs/dev-env-setup.md docs/hook-implementation-summary.md \
  docs/hook-timing-complete.md docs/target-architecture.md docs/i18n-field-mapping.md \
  docs/REFACTOR_SUMMARY.md docs/i18n-style.md \
  docs/vscode-i18n-implementation-plan-2026-06-30.md docs/features.md \
  docs/custom-endpoint-guide.md docs/pr-hook-test-command.md \
  docs/codex-claude-config-analysis.md docs/telemetry.md docs/acp-sdk-handler-notes.md
```

纯文档任务：未执行 `cargo` / `npm` 构建与测试，未改动源码，未改动 `files_owned` 之外的任何文件。
