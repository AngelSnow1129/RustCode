# zh-check 报告

- 基线 (--base): `3ee655e3`
- 阈值 (--en-ratio): `0.05`
- 判定: AC-2 / AC-4 / AC-6 / AC-7b / AC-32，任一失败即 FAIL（fail-closed）

## 汇总

- 受检文件: 15
- PASS: 15
- FAIL: 0

## PASS `docs/testing/release-v5.0.0-acceptance.md`

- en=0/190 ratio=0.0000

## PASS `docs/archive/release-v5.0.3-core-retirement-acceptance.md`

- en=0/256 ratio=0.0000

## PASS `docs/archive/release-v5.0.1-current-branch-change-report.md`

- en=0/167 ratio=0.0000

## PASS `docs/archive/kernel-parity-backlog.md`

- en=1/58 ratio=0.0172
- AC-3 残留行清单 (en 行，共 1 行):
  - `docs/archive/kernel-parity-backlog.md:4`: `> ['coding-runtime-native-migration-design.md'](coding-runtime-native-migration-design.md)`

## PASS `docs/archive/live-transport-convergence-plan.md`

- en=0/130 ratio=0.0000

## PASS `docs/archive/v5.0.0-retire-bridge-core-progress.md`

- en=0/81 ratio=0.0000

## PASS `docs/security/permission-model.md`

- en=0/253 ratio=0.0000

## PASS `docs/pr-descriptions/2026-04-27-vscode-extension-ui-mcp.md`

- en=0/59 ratio=0.0000

## PASS `docs/mcp/github.md`

- en=0/70 ratio=0.0000

## PASS `docs/testing/windows-path-normalization.md`

- en=0/63 ratio=0.0000
- R5 跳过的行 (共 6 行):
  - `docs/testing/windows-path-normalization.md:25`: `      → 正常运行(过去会 'python C:sers...' 找不到)`
  - `docs/testing/windows-path-normalization.md:45`: `      (过去 '\\?\C:\..' 传给 'cmd start' 打不开)`
  - `docs/testing/windows-path-normalization.md:48`: `      '/context' / footer 的 cwd 正常`
  - `docs/testing/windows-path-normalization.md:50`: `      'fs_mkdir')→ 选它 → 该目录的会话在 **webui 和 TUI 两边是同一个**`
  - `docs/testing/windows-path-normalization.md:56`: `      读 / grep → **不再被拒**(过去每个绝对路径都报 "outside the review repository")`
  - `docs/testing/windows-path-normalization.md:74`: `      也接受,但实测 'type' / 'cd' / python 等是否 OK(若有问题需单独处理)`

## PASS `docs/archive/2026-07-25-provider-retry-consolidation.md`

- en=0/50 ratio=0.0000

## PASS `docs/adr/0002-runtime-owned-dynamic-mcp-tool-catalog.md`

- en=0/17 ratio=0.0000

## PASS `docs/archive/2026-07-27-models-dev-pricing-design.md`

- en=0/37 ratio=0.0000

## PASS `docs/adr/0003-runtime-owned-turn-execution-policy.md`

- en=0/24 ratio=0.0000

## PASS `docs/adr/0001-runtime-owned-session-transitions.md`

- en=0/3 ratio=0.0000

## 附录 · 被 R5 跳过的行（供抽检）

共 6 行。

- `docs/testing/windows-path-normalization.md:25`: `      → 正常运行(过去会 'python C:sers...' 找不到)`
- `docs/testing/windows-path-normalization.md:45`: `      (过去 '\\?\C:\..' 传给 'cmd start' 打不开)`
- `docs/testing/windows-path-normalization.md:48`: `      '/context' / footer 的 cwd 正常`
- `docs/testing/windows-path-normalization.md:50`: `      'fs_mkdir')→ 选它 → 该目录的会话在 **webui 和 TUI 两边是同一个**`
- `docs/testing/windows-path-normalization.md:56`: `      读 / grep → **不再被拒**(过去每个绝对路径都报 "outside the review repository")`
- `docs/testing/windows-path-normalization.md:74`: `      也接受,但实测 'type' / 'cd' / python 等是否 OK(若有问题需单独处理)`

---

# D-26 手写小结（doc-writer）

## 1. 自检命令（基线与终检同一条，报告为最后一次运行的结果）

```bash
python3 scripts/check-zh-docs.py check --base 3ee655e3 \
  --report .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-26.md \
  --files docs/testing/release-v5.0.0-acceptance.md \
    docs/archive/release-v5.0.3-core-retirement-acceptance.md \
    docs/archive/release-v5.0.1-current-branch-change-report.md \
    docs/archive/kernel-parity-backlog.md \
    docs/archive/live-transport-convergence-plan.md \
    docs/archive/v5.0.0-retire-bridge-core-progress.md \
    docs/security/permission-model.md \
    docs/pr-descriptions/2026-04-27-vscode-extension-ui-mcp.md \
    docs/mcp/github.md docs/testing/windows-path-normalization.md \
    docs/archive/2026-07-25-provider-retry-consolidation.md \
    docs/adr/0002-runtime-owned-dynamic-mcp-tool-catalog.md \
    docs/archive/2026-07-27-models-dev-pricing-design.md \
    docs/adr/0003-runtime-owned-turn-execution-policy.md \
    docs/adr/0001-runtime-owned-session-transitions.md
```

终检结果：**受检 15，PASS 15，FAIL 0**。本小结在最后一次自检之后追加（铁律 §5 第 5 条）。

## 2. 逐文件改动量与判定

| 文件 | 基线 en/total | 现 en/total | 判定 | 增删行 |
|---|---|---|---|---|
| `docs/security/permission-model.md` | 159/253 = 0.6285 | **0/253 = 0.0000** | 纯英文，全文译 | +161/-161 |
| `docs/archive/2026-07-25-provider-retry-consolidation.md` | 45/50 = 0.9000 | **0/50 = 0.0000** | 纯英文，全文译 | +49/-49 |
| `docs/adr/0003-runtime-owned-turn-execution-policy.md` | 27/27 = 1.0000 | **0/24 = 0.0000** | 纯英文，全文译 | +24/-27 |
| `docs/archive/kernel-parity-backlog.md` | 46/58 = 0.7931 | **1/58 = 0.0172** | 混排，译英文段 | +46/-46 |
| `docs/adr/0002-runtime-owned-dynamic-mcp-tool-catalog.md` | 4/17 = 0.2353 | **0/17 = 0.0000** | 已中文，清 offender（H1 + 三个结构性标题） | +4/-4 |
| `docs/adr/0001-runtime-owned-session-transitions.md` | 1/3 = 0.3333 | **0/3 = 0.0000** | 已中文，清 offender（仅 H1） | +1/-1 |
| `docs/archive/release-v5.0.3-core-retirement-acceptance.md` | 7/256 = 0.0273 | **0/256 = 0.0000** | 已中文，清 offender | +7/-7 |
| `docs/archive/release-v5.0.1-current-branch-change-report.md` | 5/167 = 0.0299 | **0/167 = 0.0000** | 已中文，清 offender | +5/-5 |
| `docs/archive/live-transport-convergence-plan.md` | 6/130 = 0.0462 | **0/130 = 0.0000** | 已中文，清 offender | +6/-6 |
| `docs/mcp/github.md` | 3/70 = 0.0429 | **0/70 = 0.0000** | 已中文，清 offender（3 个排障小标题） | +3/-3 |
| `docs/testing/windows-path-normalization.md` | 2/63 = 0.0317 | **0/63 = 0.0000** | 已中文，清 offender | +2/-2 |
| `docs/pr-descriptions/2026-04-27-vscode-extension-ui-mcp.md` | 1/59 = 0.0169 | **0/59 = 0.0000** | 已中文，清 offender（`- [x] DeepSeek`） | +1/-1 |
| `docs/testing/release-v5.0.0-acceptance.md` | 0/190 = 0.0000 | 0/190 = 0.0000 | **no-op**（基线已全中文，无 offender） | 0 |
| `docs/archive/v5.0.0-retire-bridge-core-progress.md` | 0/81 = 0.0000 | 0/81 = 0.0000 | **no-op**（基线已全中文，无 offender） | 0 |
| `docs/archive/2026-07-27-models-dev-pricing-design.md` | 0/37 = 0.0000 | 0/37 = 0.0000 | **no-op**（基线已全中文，无 offender） | 0 |

说明：`docs/adr/0003` 为 +24/-27，总行数减少 3 行。原因是中文表达更紧凑、按语义重新折行（铁律 §5 第 7 条允许）；该文件空行数改动前后均为 8，段落结构未被增删空行破坏，且 AC-2/4/7b/32 全 PASS。

## 3. 残留行清单与白名单理由

终检后仅剩 **1 行**英文行：

- `docs/archive/kernel-parity-backlog.md:4`：
  `> [`coding-runtime-native-migration-design.md`](coding-runtime-native-migration-design.md)`

**属于白名单的理由**：该行是中文引用块中间独立成行的一条相对链接。行内唯一文本是被反引号包裹的文件名（不可译，铁律 §2 第 2 条），链接目标本身也不可改（AC-7b）。在不改动链接目标、不新增反引号、不破坏该 blockquote 三行折行结构的前提下，无法在这一行补入真实汉字；若强行把第 3、5 行的汉字挪上来，属于改写已中文段落（铁律 §8）。该行占比 1/58 = 0.0172，低于 0.05 阈值，故按白名单保留。

其余 14 个文件 **en = 0**，无残留。

## 4. 四项实测结果

| 项 | 结果 | 说明 |
|---|---|---|
| **AC-2** `en/total <= 0.05` | **PASS（15/15）** | 最高为 `kernel-parity-backlog.md` 的 0.0172，其余 14 个文件为 0.0000 |
| **AC-4** inline code + fenced code 多重集相等 | **PASS（15/15）** | 除独立复核外，另用脚本逐个比对基线与现状的 code span 多重集，12 个改动文件全部相等；未新增/删除任何反引号，未把非代码词包成反引号 |
| **AC-7b** `](...)` 链接目标多重集相等 | **PASS（15/15）** | 同上脚本复核，链接目标多重集全部相等；只译了 `docs/mcp/github.md` 等处的标题显示文字，未动任何链接目标与锚点 |
| **AC-32** Emoji 数不增加 | **PASS（15/15）** | `kernel-parity-backlog.md` 的 `◐` 为 1→1，其余文件 0→0；未新增任何 Emoji |

补充 **AC-6（YAML frontmatter）**：15 个文件首行均为 `#` 开头的 Markdown 标题，**本批无任何 YAML frontmatter**（与本批特殊说明第 1 条一致），因此没有 `description` 值被汉化，无需逐条列举。

## 5. 本批特殊说明的遵守情况

1. **`docs/security/permission-model.md`（安全边界文档，Han=0）**：完整汉化。权限模式与策略键名一律保留英文原样 —— `AutoApprove`、`RequireApproval`、`RequireApprovalAlways`、`Enumerate`、`Read`、`Write`、受保护前缀（`/System`、`/etc` 等）、例外前缀、home/文件名/扩展名清单、工具名映射表（`read_file`→`Read` 等）、shell 命令分类清单（`cat`/`ls`/`cp` 等），全部未改动，只译表头与说明文字。
2. **`docs/adr/*` 三条**：ADR 编号（`0001`/`0002`/`0003`）体现在文件名中，未改；文件名里的 `runtime-owned-*` slug 未改；H1 标题译为 `# 会话状态流转由 Coding Runtime 提交`、`# MCP 就绪判定与会话状态流转解耦`、`# 每回合执行策略由 Runtime 持有`，其中的 `Coding Runtime`、`Runtime`、`MCP` 等专有名词保留英文；结构性标题沿用仓库既有中文惯例 `## 背景` / `## 决策` / `## 结论`（与 `docs/superpowers/specs/*` 现有写法一致）。已确认全仓无指向这三个文件锚点的链接，译标题不会断链。
3. **`docs/archive/release-v5.0.*` 历史验收报告**：`v5.0.x` 版本号、commit hash（`9fcc92d9`、`cfe4209c`）、crate 名、测试数（`52`/`202`/`1633`/`1094` 等）、`cargo` 命令全部原样保留，只把 `tests` 等英文量词改为 `个测试` / `项通过`。
4. **先跑自检再动手**：已按特殊说明第 5 条先跑基线自检，据此把 15 个文件分成「纯英文需全文译」（3 个）与「已中文只需清 offender」（12 个，其中 3 个最终为 no-op）两类。

## 6. 踩坑规避说明

- **坑 1（误加反引号）**：`bash`、`native runtime`、`change directory`、`DeepSeek` 等原文未加反引号的词，译文中一律保持不加；独立复核 code span 多重集已确认相等。
- **坑 2（全角标点不算汉字）**：没有只用全角标点糊弄的行，所有原本被判 en 的行都补入了真实汉字（如 `tests`→`个测试`、`passed`→`项通过`、`unit tests`→`单元测试`、`,`→`与`）。
- **坑 3（批量替换波及代码块）**：`permission-model.md` 中两处 fenced `bash` 代码块（`cat /etc/hosts`、`python -c ...`）逐字未动；所有替换均为成对的显式字符串替换，改后立即复检。
- **坑 4（多插/少删空行）**：4 个改动量最大的文件空行数逐一核对，`permission-model.md` 110→110、`provider-retry-consolidation.md` 21→21、`kernel-parity-backlog.md` 25→25、`adr/0003` 8→8。
- **坑 5（报告被覆盖）**：本小结写在最后一次自检之后追加。
- **坑 6（跨行反引号对）**：动手前用脚本扫描了 4 个待全文译文件的基线，确认除 fence 行外没有奇数反引号行（即不存在跨行 inline code 对），因此可以安全重排折行；`adr/0003` 的重排正是基于这一确认。
- **坑 7（行数判定）**：`adr/0003` 总行数 36→36 但计入 total 的行 27→24（折行更紧凑），已按「看 AC 而非看行数」判定；段落结构与空行数未变。
