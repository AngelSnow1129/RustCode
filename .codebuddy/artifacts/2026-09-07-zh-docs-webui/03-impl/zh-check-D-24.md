# zh-check 报告

- 基线 (--base): `3ee655e3`
- 阈值 (--en-ratio): `0.05`
- 判定: AC-2 / AC-4 / AC-6 / AC-7b / AC-32，任一失败即 FAIL（fail-closed）

## 汇总

- 受检文件: 2
- PASS: 2
- FAIL: 0

## PASS `docs/archive/coding-runtime-incremental-migration.md`

- en=0/776 ratio=0.0000

## PASS `docs/archive/session-convergence-plan.md`

- en=0/585 ratio=0.0000

---

# D-24 人工小结（批次 8 · 归档目录两个超大件）

## 1. 本批范围与自检命令

```bash
python3 scripts/check-zh-docs.py check --base 3ee655e3 \
  --report .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-24.md \
  --files docs/archive/coding-runtime-incremental-migration.md docs/archive/session-convergence-plan.md
```

两个文件合计 112 KB，**已中文为主**（Han 760 / Han 596），因此按铁律 §3 只处理 `check`
报出的英文 offender 行，**未润色任何已中文段落**。两文件均无 YAML frontmatter。

改动形态：`git diff --stat` 为 **58  insertions(+) / 58 deletions(-)**，纯行内替换，
`git diff -U0` 中 `^-`/`^+` 空行计数均为 **0**，未增删空行、未改动段落结构。

## 2. 文件一 `docs/archive/coding-runtime-incremental-migration.md`

- 基线自检：**FAIL**，en=49/776 = 0.0631 > 0.05。
- 改后自检：**PASS**，en=**0**/776 = **0.0000**。
- 实际改动 **50 行**（49 个 offender 行 + 1 个为修折行而必须同改的伴随行 388）。

### 2.1 逐条改动清单（行号 = 基线行号 = 改后行号，行数未变）

| 行 | 改动前 | 改动后 | 说明 |
|---|---|---|---|
| 90 | `### 2.2 core-free` | `### 2.2 core-free 架构属性` | 补中文名词，层级不变 |
| 169 | `- session binding；` | `- 会话绑定；` | 纯译 |
| 171 | `- review/subagent provider slot；` | `- review/subagent provider 槽位；` | 保留 provider 术语 |
| 237 | `- send/respond/compact/snapshot/cancel/shutdown；` | 末尾追加 ` 等运行态操作` | kernel 命令名按铁律 3 保留 |
| 314 | `### 6.1 reassemble` | `### 6.1 reassemble（重新组装）` | 层级不变 |
| 331 | `- session ID；` | `- 会话 ID；` | 纯译 |
| 332 | `- conversation snapshot；` | `- conversation 快照；` | 半译 |
| 333 | `- approval grants；` | `- 审批授权；` | 纯译 |
| 336 | `- gateway affinity。` | `- gateway 亲和性。` | 保留 gateway |
| 338 | `### 6.2 reprepare` | `### 6.2 reprepare（重新准备）` | 层级不变 |
| 358 | `### 6.3 fresh` | `### 6.3 fresh（全新会话）` | 层级不变 |
| 374 | `- pending approval；` | `- 未决审批；` | 纯译 |
| 375 | `- goal/loop controller；` | `- goal/loop 控制器；` | 保留 goal/loop |
| 388–389 | `…Agent、session 或 event` / `receiver。` | `…Agent、session 或事件` / `接收端（receiver）。` | **跨行硬折行**，见 §2.2 |
| 454 | `- pending request ID；` | `- 待处理请求 ID；` | 纯译 |
| 455 | `- approval grant；` | `- 审批授权；` | 纯译 |
| 456 | `- context usage/report。` | `- context 用量/报告。` | 半译 |
| 620 | `- session affinity；` | `- session 亲和性；` | 半译 |
| 621 | `- proxy/TLS/user-agent；` | 末尾追加 ` 配置` | 协议名按铁律 3 保留 |
| 622 | `- reasoning/chat options；` | `- reasoning/chat 选项；` | 半译 |
| 623 | `- review/subagent tier；` | `- review/subagent 档位；` | 半译 |
| 652 | `- start/send/terminal event；` | `- start/send/terminal 事件；` | 半译 |
| 653 | `- approval request/respond/cancel；` | `- approval 请求/响应/取消；` | 半译 |
| 654 | `- snapshot/compact/shutdown；` | 末尾追加 ` 行为` | 命令名保留 |
| 657 | `- resume/undo；` | 末尾追加 ` 流程` | 命令名保留 |
| 661 | `### 10.2 Driver parity` | `### 10.2 Driver parity（驱动一致性）` | 层级不变，首字母大小写未改 |
| 682 | `- bridge \`on_command\` handler；` | `- bridge 的 \`on_command\` handler；` | 反引号内容未动 |
| 685 | `- v1/bridge fallback；` | 末尾追加 ` 路径` | 术语保留 |
| 828 | `- core \`AgentEvent::CompactionUi\`；` | `- core 的 \`AgentEvent::CompactionUi\`；` | 反引号内容未动 |
| 875 | `- core \`AgentCommand::Compact\` variant；` | `- core 的 \`AgentCommand::Compact\` variant；` | 反引号内容未动 |
| 877 | `- bridge \`CoreCmd::Compact\` handler；` | `- bridge 的 \`CoreCmd::Compact\` handler；` | 反引号内容未动 |
| 905–910 | `：2 passed；` 等 6 行 | `：2 项通过；`、`：1 项通过；`、`：1555 项通过，1 项忽略；`、`：13 项通过；` | **仅改反引号外的散文**，命令串与数字原样 |
| 911 | `- \`cargo check … -p rustcode-tuix \` | 行首加 `依赖检查 ` | **跨行反引号对**，见 §2.2 |
| 1213 | `\| Finished + manual no-op \| …` | `\| Finished + 手动 no-op \| …` | 表内显示文字，反引号未动 |
| 1218 | `- persistent \`/live\` \`KernelTurnExecutor\`；` | `- 常驻 \`/live\` \`KernelTurnExecutor\`；` | 反引号内容未动 |
| 1220 | `- daemon bridge fallback；` | 末尾追加 ` 路径` | 术语保留 |
| 1221 | `- daemon opt-in kernel driver；` | `- daemon 的 opt-in kernel driver；` | 半译 |
| 1311 | `runtime/domain：` | `runtime 与领域：` | 小标题式标签 |
| 1316 | `- committed drain、committed stub、manual no-op、auto no-op、overflow no-op；` | `manual/auto/overflow` → `手动/自动/溢出` | 5 类 outcome 名保留 committed |
| 1319 | `bridge/order：` | `bridge 与顺序：` | 小标题式标签 |
| 1336 | `CLI/daemon：` | `CLI 与 daemon：` | CLI、daemon 均按铁律 3 保留 |
| 1338 | `- headless committed marker；` | `- headless 下的 committed marker；` | 半译 |
| 1355 | `- CLI headless、TUI foreground/background、…` | `foreground/background` → `前台/后台` | 跨行硬折行，只改本行 |
| 1385 | `workspace check。` | `workspace 检查。` | 该词在第 1307 行亦出现，已用「最广 + 换行」限定唯一命中 |

### 2.2 两处硬折行 / 跨行反引号对的处理（铁律 §5 第 6 条）

- **行 388–389**：原文把 `event receiver` 硬折成两行，389 行只有 `receiver。`。
  首轮只改 388 行后复检，389 行仍判 en（该行自身无汉字）。二轮改为 388 行收尾
  `或事件`、389 行改写为 `接收端（receiver）。`，折行位置与反引号位置均保持原样。
- **行 911–912**：一对反引号跨 911/912 两行，脚本 `INLINE_RE` 不生成 code span。
  仅在 911 行行首加 `依赖检查 `，**未重排成一行**，反引号与续行符 `\` 原位保留，
  故 AC-4 不受影响。

### 2.3 白名单（判为未改）说明

本文件 **无残留 offender，无白名单未改项**。以下英文属铁律 3 明确保留、且位于
fenced code block 内或已有汉字的行中，脚本本就不计入 en，未做任何改动：crate 名
（`rustcode-core`/`rustcode-bridge`/`rustcode-coding`/`rustcode-tuix`/`rustcode-daemon`）、
`AgentCommand`/`AgentEvent`/`CoreCmd`/`CompactionUi`/`CodingRuntime` 等类型与变体名、
`cargo test/check` 命令串、版本号与 commit hash、`/compact` 等 slash 命令。

按本批特殊说明第 4 条与 `AGENTS.md:7`，`rustcode` 作为**已退役 `rustcode-core` 的
历史名引用**全部保留原文，未改写为 `rustcode-*`。

## 3. 文件二 `docs/archive/session-convergence-plan.md`

- 基线自检：**PASS**，en=8/585 = 0.0137（已 ≤ 0.05）。
- 按铁律 §3，该文件整体可记 **no-op**（不得改写已中文段落）；但 8 行 offender 是
  **纯英文行**（标题与列表项），不属于「已中文段落」，故逐行处理，**未润色任何
  已中文段落**。
- 改后自检：**PASS**，en=**0**/585 = **0.0000**。
- 实际改动 **8 行**，无白名单未改项。

| 行 | 改动前 | 改动后 |
|---|---|---|
| 32 | `- native \`SessionSnapshot/…/TranscriptHook\`；` | `- 原生 \`SessionSnapshot/…/TranscriptHook\`；` |
| 105 | `- schema version；` | `- schema 版本；` |
| 120 | `### Runtime store` | `### Runtime 存储` |
| 129 | `### Catalog store` | `### Catalog 存储` |
| 133 | `- AI naming、turn stats、context restore、working directory；` | `- AI 命名、回合统计、上下文恢复、工作目录；` |
| 136 | `### Compatibility importer` | `### 兼容性 importer` |
| 277 | `### S1：Native schema parity` | `### S1：Native schema 对齐` |
| 568 | `#### S3c：Delete` | `#### S3c：删除` |

标题层级与 `S1`/`S3c` 编号均未改动；改后标题与同文件既有标题
（`### Session 存储所有权`、`#### S3a：只读 catalog`、`#### S3b：Rename 与 AI naming`）
风格一致。已确认两文件内**无内部锚点链接**，全仓对这两个文件也只有文件级引用、
无 `#anchor` 引用，故改标题文字不会断链。

## 4. 四项实测结果

| 验收项 | 文件一 | 文件二 | 实测手段与结论 |
|---|---|---|---|
| **AC-2**（en/total ≤ 0.05） | PASS，0.0000（0/776） | PASS，0.0000（0/585） | 脚本实测；均由 0.0631 / 0.0137 降至 0 |
| **AC-4**（inline code + fenced code 多重集相等） | PASS | PASS | 改动全部落在反引号外与 fence 外；`git diff -U0` 中无任何 `\`\`\`` 或 fence 内行变更 |
| **AC-7b**（链接目标多重集相等） | PASS | PASS | 全文链接仅 4 处相对文件链接（`../target-architecture.md`、`v5.0.0-retire-bridge-core-progress.md`、`../compact-native-migration-retrospective.md` ×2、`live-transport-convergence-plan.md` ×2），只译显示文字，未动目标 |
| **AC-32**（Emoji 数不增加） | PASS | PASS | 未新增也未删除任何 Emoji |

补充实测：

- **AC-6**：两个文件均**无 YAML frontmatter**（首行非 `---`），键名集合与
  `description` 均不适用，脚本未报差异。
- **行数**：文件一 1580 行 → 1580 行，文件二 800 行 → 800 行；`git diff` 为空行
  增删计数 0/0，符合铁律 §5 第 4、7 条。
- **铁律 2**（不新包反引号）：全批改后未新增任何反引号对，AC-4 保持 PASS。
- **未越界**：本批全部 Edit 操作只落在 `files_owned` 的上述 2 个文件上，未触碰
  `docs/archive/` 下其它文件（D-25 / D-26 的 files_owned）。说明：本批开工时
  `git status --porcelain -- docs/archive/` 为空；收尾时该目录下另出现
  `2026-07-25-provider-retry-consolidation.md`、`coding-runtime-native-migration-design.md`、
  `kernel-parity-backlog.md`、`release-v5.0.1-current-branch-change-report.md`、
  `release-v5.0.3-core-retirement-acceptance.md` 的改动，均来自并行批次，非本批所为。
- 未执行 `cargo`/`npm` 构建与测试，未改源码，未提交、未打标签。
