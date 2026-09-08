# zh-check 报告

- 基线 (--base): `3ee655e3`
- 阈值 (--en-ratio): `0.05`
- 判定: AC-2 / AC-4 / AC-6 / AC-7b / AC-32，任一失败即 FAIL（fail-closed）

## 汇总

- 受检文件: 9
- PASS: 9
- FAIL: 0

## PASS `extensions/jetbrains/README.md`

- en=0/117 ratio=0.0000

## PASS `extensions/vscode/README.md`

- en=0/92 ratio=0.0000

## PASS `extensions/jetbrains/docs/jetbrains.md`

- en=2/80 ratio=0.0250
- AC-3 残留行清单 (en 行，共 2 行):
  - `extensions/jetbrains/docs/jetbrains.md:62`: `- Claude`
  - `extensions/jetbrains/docs/jetbrains.md:63`: `- Ollama`

## PASS `extensions/jetbrains/PRIVACY.md`

- en=0/24 ratio=0.0000

## PASS `.claude/plans/atomgit-decouple.md`

- en=0/40 ratio=0.0000

## PASS `.superpowers/pr/feat-rust-tui-selection-session-preview.md`

- en=0/81 ratio=0.0000

## PASS `docker/README.md`

- en=0/54 ratio=0.0000
- R5 跳过的行 (共 1 行):
  - `docker/README.md:188`: `>`

## PASS `webui/README.md`

- en=0/21 ratio=0.0000

## PASS `packages/npm/README.md`

- en=0/16 ratio=0.0000

## 附录 · 被 R5 跳过的行（供抽检）

共 1 行。

- `docker/README.md:188`: `>`

---

# D-33 手写小结（在最后一次自检之后追加）

## 1. 本批范围、前提与自检命令

- 任务 ID：**D-33**（批次 11）。基线 `ZH_BASE=3ee655e3`，自检工具 `scripts/check-zh-docs.py`。
- files_owned 共 9 个文件，**全部无 YAML frontmatter**（首行均不是 `---`）：
  因此铁律 §2.7 的「键名一律保留英文」本批无适用对象，AC-6 键名集合在基线与工作区**皆为空集、相等**，
  `description` 无变更，无需按 AC-6 逐条列示。
- 最后一次自检命令（本报告的自动生成段即由该命令产出）：

```bash
python3 scripts/check-zh-docs.py check --base 3ee655e3 \
  --report .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-33.md \
  --files extensions/jetbrains/README.md extensions/vscode/README.md \
  extensions/jetbrains/docs/jetbrains.md extensions/jetbrains/PRIVACY.md \
  .claude/plans/atomgit-decouple.md \
  .superpowers/pr/feat-rust-tui-selection-session-preview.md \
  docker/README.md webui/README.md packages/npm/README.md
```

- 实测输出：`check: 受检 9，PASS 9，FAIL 0`。

## 2. 红线声明：`extensions/jetbrains/CHANGELOG.md` 未被改动

**`extensions/jetbrains/CHANGELOG.md` 本批未被改动（零 diff）。**
它是脚本 `SKIP_C_VERBATIM` 中的 C 类「逐字历史记录」，已在跳过清单内，本批未读取之外的任何写入，
未翻译、未重排、未删除任何一条历史条目。校验方式：

```bash
git diff --stat -- extensions/jetbrains/CHANGELOG.md   # 输出为空
```

## 3. 每个文件的改动量（含 no-op 判定）

| 文件 | 基线 en/total | 现 en/total | 改动性质 | 说明 |
|---|---|---|---|---|
| `extensions/jetbrains/README.md` | 2/117 | **0/117** | 已中文 → 只清 offender（2 行） | 第 1 行标题补中文；第 11 行 `Kotlin Gradle Plugin 2.2.21` 补「版本」二字（**未加反引号**，避免 AC-4） |
| `extensions/vscode/README.md` | 1/92 | **0/92** | 已中文 → 只清 offender（1 行） | 仅第 1 行标题补中文 |
| `extensions/jetbrains/docs/jetbrains.md` | 81/83 | **2/80** | **纯英文 → 全文译** | 结构体、标题层级、全部 inline code 与 `../PRIVACY.md` 引用原样保留 |
| `extensions/jetbrains/PRIVACY.md` | 24/24 | **0/24** | **纯英文 → 全文译** | 隐私声明，否定式断言逐条对等翻译（见 §6） |
| `.claude/plans/atomgit-decouple.md` | 0/40 | 0/40 | **no-op** | 全文已中文、零 offender，按铁律 §3 未改写任何段落 |
| `.superpowers/pr/feat-rust-tui-selection-session-preview.md` | 6/81 | **0/81** | 已中文 → 只清 offender（6 行） | 2 个英文小标题 + 1 个三级标题补中文；3 行测试结论 `1965 passed` 等改为「N 项测试通过」 |
| `docker/README.md` | 0/54 | 0/54 | **no-op** | 全文已中文、零 offender；所有 docker/compose 命令、镜像名、端口均在代码块内，本批一字未动 |
| `webui/README.md` | 27/28 | **0/21** | **纯英文 → 全文译** | npm 命令、`node` 相关表述、脚本路径全保留；跨行反引号按铁律 §5.6 处理（见 §4） |
| `packages/npm/README.md` | 4/16 | **0/16** | 已中文 → 只清 offender（4 行） | 标题补中文；两行徽章仅改**显示文字**（`npm version`→`npm 版本号`、`license`→`许可证`），链接目标与 URL 未动；末行标语译为中文 |

汇总：`git diff --stat` = **7 files changed, 138 insertions(+), 148 deletions(-)**（其余 2 个文件 no-op，零 diff）。
净行数变化 -10（`jetbrains.md` 126→123、`webui/README.md` 55→48、`PRIVACY.md` 44→44），
全部来自英文硬折行段落重排与中文更紧凑；**未增删任何空行**——
`PRIVACY.md` 基线第 32–33 行的连续两个空行原样保留。

## 4. 本批唯一踩坑：`webui/README.md` 的跨行反引号（铁律 §5.6）

首轮译文把基线第 52–53 行

```
rebuild `webui/dist/` via `npm ci && npm run build` before invoking `cargo
build`, so release binaries always embed the latest frontend. If `npm` is not
```

重排成单行后，自检立刻报 **AC-4 FAIL**（old=36 new=36，但集合不等）。
成因与铁律 §5.6 完全一致：`code_spans()` 用 `` `[^`\n]*` `` 匹配，**不能跨行**，
于是基线上「行尾未闭合的反引号（`` `cargo ``）」被跳过，其后第 53 行上
`` build` `` 的收尾反引号与 `` `npm` `` 的起始反引号被错误配成一对，
把英文散文 `, so release binaries always embed the latest frontend. If `
算成了一个 inline code span。

**处置**：恢复原有折行位置与反引号位置（行尾的 `` `cargo `` 留在行尾、行首的 `` build` `` 留在下一行行首），
并**逐字保留**被吞掉的英文片段 `, so release binaries always embed the latest frontend. If `，
只把其余自由散文译为中文。该行随后补入汉字（「在构建环境中不可用，脚本会回退到已提交的」），
因此不计入 en 行。同时在前一句补了中文释义「（使发行二进制总是内嵌最新前端）」，
避免读者只看到英文片段。

**代价**：`, so release binaries always embed the latest frontend. If ` 作为英文片段残留。
这是满足 AC-4「code span 多重集与基线完全相等」所**必须**的；
**后续任何人把这两行重排为单行、或删改该英文片段，AC-4 会立刻 FAIL，请勿「顺手整理」。**

## 5. 残留行清单与白名单判定

自检报告中仍列出 en 行的只有 `extensions/jetbrains/docs/jetbrains.md`，共 **2 行**：

| 行 | 内容 | 白名单理由 |
|---|---|---|
| `extensions/jetbrains/docs/jetbrains.md:62` | `- Claude` | 第三方模型供应商专有名词，铁律 §2.3 明确保留原文；该行除专有名词外无可译散文，补任何汉字都会引入原文没有的信息（如厂商归属） |
| `extensions/jetbrains/docs/jetbrains.md:63` | `- Ollama` | 同上；本地模型运行时的专有名词 |

该 2 行占比 `2/80 = 0.0250 <= 0.05`，AC-2 判定 **PASS**，无需进一步处理。

另有一类「刻意保留、不算残留」的内容（均已含 CJK，本就不进 offender 清单）：

- **fenced code block 内一字未动**：`extensions/jetbrains/README.md` 的 gradle 命令块、
  `docker/README.md` 的全部 docker/compose 块、`webui/README.md` 的 npm/cargo 块、
  `packages/npm/README.md` 的安装/卸载/使用块。
- **inline code 原样保留**：`RUSTCODE_WEBUI_DEV`、`127.0.0.1:13456`、`127.0.0.1`、
  `Settings | Plugins`、`Install Plugin from Disk...`、`rustcode-jetbrains-<version>-signed.zip`、
  `Tools | RustCode: Open Chat`、`RustCode: Explain Selection`、`RustCode: Fix Selection`、
  `RustCode: Optimize Selection`、`RustCode: Add Selection/File as Context`、`RustCode: Open Changes`、
  `RustCode: Open Settings`、`../PRIVACY.md`、`--no-telemetry`、`[telemetry]`、`config.toml`、
  `.env`、`webui/dist/`、`dist/`、`npm ci && npm run build`、`rust-embed`、
  `crates/rustcode-daemon/src/webui.rs`、6 个 `scripts/*.sh` 路径、`<your-org>`、
  两个 `https://example.com/<your-org>/...` 占位 URL、`rustcode@rustcode.dev`、
  `cargo test -p ... --locked` 三行命令、Kotlin Gradle Plugin 版本号 `2.2.21`。
- **专有名词与技术词保留**：RustCode、JetBrains、IntelliJ、Marketplace、Search Everywhere、
  Alt+Enter、Preact、Vite、Tailwind、Node.js、npm、OpenAI、Claude、Ollama、GnuPG、AWS、SSH、
  Terraform、HMR、vite、cargo、ratatui、base URL、API key、Session Preview、InteractionPublisher。
- **日期保留**：`Last updated:` 译为「最后更新：」，日期字面量 `June 23, 2026` 原样未改。

## 6. 隐私/合规文档专项核对（`extensions/jetbrains/PRIVACY.md`）

本文件是隐私声明，本批按「否定式断言必须保留原意」逐条对等翻译，未做任何弱化、模糊化或承诺升级：

| 原文断言 | 译文 | 语义 |
|---|---|---|
| `does not intentionally send your code or project data directly to any RustCode service` | 不会有意把你的代码或项目数据直接发送到任何 RustCode 服务 | 否定保留 |
| `Open builds (the default source build) contain no managed service and connect only to the providers you configure` | 开源构建（默认的源码构建）不包含任何托管服务，只会连接你自己配置的供应商 | 否定 + 限定保留 |
| `This fork ships ZERO telemetry. No event queue, no sender, no endpoint.` | 本 fork 的遥测为零：没有事件队列，没有发送器，没有上报端点 | 三重否定保留（未弱化为「极少」「基本不」） |
| `Crash reporting writes to stderr only; nothing leaves the machine` | 崩溃报告只写入 stderr，不会有任何数据离开本机 | 否定保留 |
| `The --no-telemetry flag is accepted and ignored` / `A legacy [telemetry] section ... is silently ignored` | `--no-telemetry` 参数会被接受并忽略 / 遗留的 `[telemetry]` 配置段会被静默忽略 | 「被忽略」语义保留 |
| `Do not enter API keys unless you trust ...` | 除非你信任该本地后端与所配置的供应商，否则不要输入 API key | 警示语气保留 |
| `This classification is best-effort and does not replace your own review` | 该分类是尽力而为的，不能替代你在把上下文发送给模型供应商之前自行进行的检查 | 免责限定保留 |

同时全文**未新增**任何收集范围、未删除任何数据项，章节结构（`##` 层级与 7 个小节）与基线一致。

## 7. 四项实测结果

| 项 | 阈值/口径 | 实测 | 结论 |
|---|---|---|---|
| **AC-2** | `en/total <= 0.05` | 9 个文件：`0/117`、`0/92`、`2/80`、`0/24`、`0/40`、`0/81`、`0/54`、`0/21`、`0/16` | **PASS 9/9**（最差的 `jetbrains.md` 0.0250；改动前 `PRIVACY.md` 1.0000、`webui/README.md` 0.9643、`jetbrains.md` 0.9759） |
| **AC-4** | inline code + fenced code 多重集与基线完全相等 | 无一文件报差异（首轮 `webui/README.md` 曾 FAIL，已按 §4 修复后转 PASS） | **PASS 9/9** |
| **AC-7b** | `](...)` 链接目标多重集相等 | 无一文件报差异；徽章行仅改显示文字，两个 npmjs/shields URL 原样未动 | **PASS 9/9** |
| **AC-32** | Emoji 数不增加 | 无一文件报 `emoji 命中数增加`；本批未新增任何 Emoji，也未清理既有 Emoji | **PASS 9/9** |

附带核对：

- **AC-6**：9 个文件均无 frontmatter，键名集合基线与工作区皆为空集，相等；`description` 无变更。
- **空行/结构**：未增删空行（`PRIVACY.md` 第 32–33 行双空行原样保留），标题层级未变，文件名未变。
- **范围**：只改 files_owned 的 9 个文件；未改源码；未执行 `cargo`/`npm` 构建与测试（铁律 §7）。

## 8. 术语与命名一致性检查结论

| 术语 | 本批译法 | 一致性依据 |
|---|---|---|
| daemon | **后端** | 与早前批次已汉化的 `extensions/jetbrains/README.md`「后端 REST/SSE 客户端」「后端二进制路径」一致 |
| provider | **供应商** | 同上，README 已用「第三方供应商」「提供商创建/编辑/删除」两种写法；本批在新增译文中统一为「供应商」 |
| context | **上下文** | 与 README「上下文级别」「选中文本上下文」一致 |
| settings / preferences | **设置** | 与 README「RustCode 设置页面」一致 |
| issue tracker | **Issue 跟踪器** | 保留 `Issue` 原词，与仓库既有中文文档一致 |
| Session Preview / InteractionPublisher | 保留英文 + 补中文（「Session Preview 会话预览」「InteractionPublisher 交互发布器」） | 与该文件既有「Composer 文本选择」「Transcript 文本选择与复制」的「英文 + 中文」并列风格一致 |
| tool window | **工具窗口** | 与 README 一致 |
| daemon 端口/主机 | 保留 `127.0.0.1:13456` | 铁律 §2.3 |

未发现与既有中文文档冲突的译法；未发现已退役组件被写成可用能力（本批 9 个文件不涉及退役组件描述）。

## 9. 文档更新清单

| 文件路径 | 变更类型 | 摘要 |
|---|---|---|
| `extensions/jetbrains/README.md` | 修订 | 标题补中文；`Kotlin Gradle Plugin` 行补「版本」 |
| `extensions/vscode/README.md` | 修订 | 标题补中文 |
| `extensions/jetbrains/docs/jetbrains.md` | 修订（全文英译中） | 插件使用说明全文汉化，结构与 inline code 不变 |
| `extensions/jetbrains/PRIVACY.md` | 修订（全文英译中） | 隐私政策全文汉化，否定式断言逐条对等 |
| `.claude/plans/atomgit-decouple.md` | 无变更（no-op） | 已中文且零 offender |
| `.superpowers/pr/feat-rust-tui-selection-session-preview.md` | 修订 | 3 个英文标题补中文；3 行测试结论补中文 |
| `docker/README.md` | 无变更（no-op） | 已中文且零 offender |
| `webui/README.md` | 修订（全文英译中） | 前端构建说明全文汉化，命令/路径/端口保留 |
| `packages/npm/README.md` | 修订 | 标题、徽章显示文字、末行标语汉化 |
| `extensions/jetbrains/CHANGELOG.md` | **无变更（红线）** | C 类逐字历史记录，本批零 diff |

## 10. 回滚方案

本批为纯文档汉化，且全部改动尚未提交（工作区改动）。回滚方式按优先级：

1. **整体回滚本批（推荐）**：`git checkout -- extensions/jetbrains/README.md extensions/vscode/README.md extensions/jetbrains/docs/jetbrains.md extensions/jetbrains/PRIVACY.md .superpowers/pr/feat-rust-tui-selection-session-preview.md webui/README.md packages/npm/README.md`
   判定时机：任一文件出现 AC-2/AC-4/AC-7b/AC-32 FAIL 且无法就地修复，或编排者否决本批译文。
2. **单文件回滚**：对上表中任一路径单独执行 `git checkout -- <path>`。
   判定时机：仅个别文件译文被否决（例如 `PRIVACY.md` 的合规措辞需要法务复核）。
3. **回滚到基线全量校验**：`git stash` 后重跑 §1 的自检命令，确认 `PASS 9` 仍成立（用于排除本批之外的干扰）。
4. **无需回滚数据/配置/代码**：本批未触碰源码、未修改任何配置文件与版本号，无数据迁移代价，回滚代价为零。

## 11. 已知未验证范围与后续项

- **未验证**：未执行 `cargo` / `npm` 构建与测试（铁律 §7 明确禁止；本批为纯文档任务）。
  因此 `webui/README.md` 中的 npm 命令、`docker/README.md` 中的 docker/compose 命令、
  `extensions/jetbrains/README.md` 中的 gradle 命令**仅按原文保留，未实际执行验证**。
- **未验证**：`extensions/jetbrains/docs/jetbrains.md` 与 `PRIVACY.md` 中描述的插件行为
  （工具窗口入口、设置项、供应商配置、敏感路径分类）未与当前插件实现逐条对拍；本批只做语言转换，未改语义。
- **未验证**：`packages/npm/README.md` 徽章改用中文 alt 文字后，各渲染端（npmjs、GitHub）的无障碍显示效果未实测。
- **遗留后续项（建议负责人）**：
  1. `webui/README.md` 末段的英文片段 `, so release binaries always embed the latest frontend. If `（§4）——
     若编排者认为可接受中文破损，可另派任务在**同步更新基线**的前提下重排该段，属基线变更决策，**本批不擅自处理**。
  2. 本 feature 已新增 `scripts/build-webui.sh`，但按本批指示**未**在 `webui/README.md` 中提及该脚本，
     是否补写由编排者另派任务决定。
  3. `extensions/jetbrains/docs/jetbrains.md` 第 62–63 行的 `Claude` / `Ollama` 若需清零 en 行，
     需先确定是否允许在专有名词后追加中文限定语（涉及引入原文没有的信息），建议由编排者裁定。
