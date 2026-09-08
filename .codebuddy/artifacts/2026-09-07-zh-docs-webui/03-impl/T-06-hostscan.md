# T-06 · 清除 md 中与新默认 host 矛盾的 `127.0.0.1` 叙述

- feature：`2026-09-07-zh-docs-webui`
- 基线：`ZH_BASE=3ee655e3`（HEAD 仍等于基线，改动仅在未提交工作区）
- 分支：`dev`
- 工具：`python3 scripts/check-zh-docs.py`（仅此脚本，未执行任何 `cargo` / `npm`）
- files_owned（hostscan 实测命中，6 文件）：`docker/README.md`、`docs/superpowers/plans/2026-05-29-webui.md`、`docs/superpowers/specs/2026-05-29-webui-design.md`、`extensions/jetbrains/PRIVACY.md`、`extensions/jetbrains/README.md`、`extensions/jetbrains/docs/jetbrains.md`

## 0. 结论速览

**本任务为 no-op（0 行改动、0 个文件改动）。** `hostscan` 命中的 9 行经逐行取证后：

- 3 行属 R1（带日期的历史计划/规格，陈述写作当时现状）→ 保留原文，不补充、不改写；
- 4 行属 R3（IDE 插件拉起/连接后端）→ 与裁决 Q2 一致，保留；
- 2 行属 R4（`docker/README.md`）→ 实证 `docker/docker-compose.yml` **显式写死**宿主侧绑定默认值，按裁决归入 R3 保留；
- **R2 命中 0 行**：没有任何一行属于「陈述当前产品默认行为且与新默认 `0.0.0.0` 矛盾」。

因此未发生任何 `Edit`，`check` 自检按任务书规定不执行（下文 §3 写明 no-op 及依据）。命中数变化：**改前 6 文件 / 9 行 → 改后 6 文件 / 9 行（无变化，全部为白名单保留项）**。

## 1. 逐行分类表

| # | 行号 | 原文 | 判定 | 处置 | 理由 |
|---|------|------|------|------|------|
| 1 | `docker/README.md:186` | `> compose 端口默认仅绑定 `127.0.0.1`（本机访问）。如需从局域网/NAS 访问，用` | R4 → 实证后按 R3 | 不动 | 实证依据 `docker/docker-compose.yml:44`：`- "${BIND_ADDR:-127.0.0.1}:13456:13456"`。宿主侧端口发布地址由 compose **显式写死**默认值，不由产品默认值决定，故不随 CLI 默认改 `0.0.0.0` 而失效。同文件 `:42-43` 注释亦复述该 `BIND_ADDR` 机制 |
| 2 | `docker/README.md:210` | `默认端口只绑定宿主机 `127.0.0.1`。仅在可信局域网中需要手机直连时，使用：` | R4 → 实证后按 R3 | 不动 | 同一事实（compose `:44` 写死 `${BIND_ADDR:-127.0.0.1}`）；`:24` 亦给出 `BIND_ADDR=0.0.0.0` 的显式放开方式。容器侧另有 `docker-compose.yml:39` 的 `command: ["--host", "0.0.0.0", ...]` 显式传参，与产品默认无关 |
| 3 | `docs/superpowers/plans/2026-05-29-webui.md:44` | `- daemon 默认 `127.0.0.1:13456`；CLI `Commands::Daemon` 通过 re-exec `rustcode-daemon` 二进制启动（`rustcode-cli/src/main.rs:930+`）` | R1（并叠加 R3） | 不动 | 文件名带日期 `2026-05-29`，属写作当时的现状快照；改之即篡改历史。且该句描述对象是**独立 `rustcode-daemon` 二进制**的默认值（裁决 Q2 明确不改，`crates/rustcode-daemon/src/main.rs:21` 仍为 `127.0.0.1`），双重命中保留规则 |
| 4 | `docs/superpowers/specs/2026-05-29-webui-design.md:34` | `- `crates/rustcode-daemon`（axum）默认绑 `127.0.0.1:13456`，已提供：`/chat`(SSE 流式)、` | R1（并叠加 R3） | 不动 | 带日期的规格文档，位于「现状事实（实现时依赖）」小节，是 2026-05-29 定稿时的快照（同小节还写有 `main.rs:2051`、：`1782` 等当时行号，性质一致）；描述对象同样是 daemon crate / 独立二进制默认 |
| 5 | `docs/superpowers/specs/2026-05-29-webui-design.md:142` | `- server 默认只绑 `127.0.0.1`（现状已是）。` | R1 | 不动 | 同属带日期规格的「安全模型（本地）」小节定稿快照；「（现状已是）」是写作当时的判定，不是对当前产品的承诺 |
| 6 | `extensions/jetbrains/PRIVACY.md:19` | `默认情况下，插件会连接到位于 `127.0.0.1:13456` 的 RustCode 后端。插件也可以在你的机器上启动打包的或已配置的后端进程。……` | R3 | 不动 | 描述 IDE 插件连接/拉起后端。实证：`extensions/jetbrains/src/main/kotlin/com/rustcode/jetbrains/settings/RustCodeSettingsState.kt:25` 为 `var host: String = "127.0.0.1"`（`:62` 空值时回填同值），与裁决 Q2 一致；且插件未向后端显式传 `--host`（`grep '"--host"' extensions/jetbrains/src/main/kotlin/` 无命中），走的是二进制自身默认 |
| 7 | `extensions/jetbrains/README.md:241` | `插件默认将后端主机设置为 `127.0.0.1`，使用后端的 HTTP API，并且不收集插件遥测数据。……` | R3 | 不动 | 同上，陈述插件侧 host 设置默认值（`RustCodeSettingsState.kt:25`），与 Q2 一致 |
| 8 | `extensions/jetbrains/docs/jetbrains.md:44` | `- 主机与端口，默认为 `127.0.0.1:13456`` | R3 | 不动 | 位于「配置后端」小节，描述插件设置项默认值，同上 |
| 9 | `extensions/jetbrains/docs/jetbrains.md:53` | `默认情况下，插件与位于 `127.0.0.1` 的本地后端通信。如果你配置了其他主机，请在发送项目上下文之前先评估隐私与安全影响。` | R3 | 不动 | 同上；该句还承担隐私提示作用，改写会削弱安全语义 |

判定分布（按 9 行计）：**R1 = 3，R2 = 0，R3 = 4，R4 = 2**；两行 R4 实证为「compose 显式写死」后按裁决归入 R3 处置，故**实际按保留处理的共 9 行（3+4+2）**。

### R4 实证记录（`docker/docker-compose.yml`）

```yaml
37:    # 容器内默认绑定 127.0.0.1，必须显式指定 --host 0.0.0.0 才能在容器外访问；
38:    # 宿主侧的对外暴露则由 ports 的 BIND_ADDR 控制（默认仅本机，见上方安全警告）
39:    command: ["--host", "0.0.0.0", "--port", "13456"]
...
44:      - "${BIND_ADDR:-127.0.0.1}:13456:13456"
```

判定：`docker/README.md` 两行讲的是**宿主侧 `ports` 发布地址**，该默认值由 compose 第 44 行以 `${BIND_ADDR:-127.0.0.1}` **显式写死**，不依赖产品默认；容器内绑定另有第 39 行显式 `--host 0.0.0.0` 传参。故按 R4 分支「写死了 → 按 R3 保留」，两行均不改。另注：`docker/Dockerfile-Daemon:43` 的 `ENTRYPOINT ["rustcode-daemon"]` 指向**独立二进制**（默认仍 `127.0.0.1`），因此第 37 行注释与 README 两行在新默认下**依然成立**，不存在与新默认的矛盾。

## 2. 实际改动清单（含 no-op 判定）

| 文件 | 改动前 | 改动后 | 判定 | diff 行数 |
|------|--------|--------|------|-----------|
| `docker/README.md` | 2 行命中 | 2 行命中 | **no-op**（R4→R3 保留） | +0 / -0 |
| `docs/superpowers/plans/2026-05-29-webui.md` | 1 行命中 | 1 行命中 | **no-op**（R1 保留） | +0 / -0 |
| `docs/superpowers/specs/2026-05-29-webui-design.md` | 2 行命中 | 2 行命中 | **no-op**（R1 保留） | +0 / -0 |
| `extensions/jetbrains/PRIVACY.md` | 1 行命中 | 1 行命中 | **no-op**（R3 保留） | +0 / -0 |
| `extensions/jetbrains/README.md` | 1 行命中 | 1 行命中 | **no-op**（R3 保留） | +0 / -0 |
| `extensions/jetbrains/docs/jetbrains.md` | 2 行命中 | 2 行命中 | **no-op**（R3 保留） | +0 / -0 |

合计：**改动文件 0 个，diff +0 / -0**。`git status --short` 中上述 6 个文件的状态与本任务执行前一致（其中 `plans/2026-05-29-webui.md`、`jetbrains/PRIVACY.md`、`jetbrains/README.md`、`jetbrains/docs/jetbrains.md` 的 `M` 状态来自本 feature 前序汉化批次的改动，非本任务引入）。

铁律遵守情况：本任务零编辑，故不存在译代码块（铁律 1）、误加反引号（铁律 2）、改路径/标识符/命令（铁律 3）、改链接目标（铁律 4）、改文件名与标题层级（铁律 5）、新增 Emoji（铁律 6）、frontmatter 键名（铁律 7）、润色已中文段落（铁律 8）的违规可能。另按任务书要求，**未**在 R1 小节做任何补充句。

## 3. 自检真实输出与退出码

### 3.1 `hostscan`（改前 / 改后一致，因零改动；实际执行两次，输出相同）

命令：

```bash
python3 scripts/check-zh-docs.py hostscan --base 3ee655e3
```

输出：

```
hostscan: base=3ee655e3 命中 6 个文件（内容来源：工作区优先，缺失时回退基线）

## 命中文件（供 T-06 的 files_owned；T-06 需自行排除 README.md）

docker/README.md
docs/superpowers/plans/2026-05-29-webui.md
docs/superpowers/specs/2026-05-29-webui-design.md
extensions/jetbrains/PRIVACY.md
extensions/jetbrains/README.md
extensions/jetbrains/docs/jetbrains.md

## 明细

docker/README.md:186: > compose 端口默认仅绑定 `127.0.0.1`（本机访问）。如需从局域网/NAS 访问，用
docker/README.md:210: 默认端口只绑定宿主机 `127.0.0.1`。仅在可信局域网中需要手机直连时，使用：
docs/superpowers/plans/2026-05-29-webui.md:44: - daemon 默认 `127.0.0.1:13456`；CLI `Commands::Daemon` 通过 re-exec `rustcode-daemon` 二进制启动（`rustcode-cli/src/main.rs:930+`）
docs/superpowers/specs/2026-05-29-webui-design.md:34: - `crates/rustcode-daemon`（axum）默认绑 `127.0.0.1:13456`，已提供：`/chat`(SSE 流式)、
docs/superpowers/specs/2026-05-29-webui-design.md:142: - server 默认只绑 `127.0.0.1`（现状已是）。
extensions/jetbrains/PRIVACY.md:19: 默认情况下，插件会连接到位于 `127.0.0.1:13456` 的 RustCode 后端。插件也可以在你的机器上启动打包的或已配置的后端进程。插件向该本地后端发送请求，以便后端运行编码智能体工作流、管理会话、与模型供应商通信，并执行经用户批准的操作。
extensions/jetbrains/README.md:241: 插件默认将后端主机设置为 `127.0.0.1`，使用后端的 HTTP API，并且不收集插件遥测数据。在发送编辑器选中内容或文件作为聊天上下文之前，会应用敏感路径分类。
extensions/jetbrains/docs/jetbrains.md:44: - 主机与端口，默认为 `127.0.0.1:13456`
extensions/jetbrains/docs/jetbrains.md:53: 默认情况下，插件与位于 `127.0.0.1` 的本地后端通信。如果你配置了其他主机，请在发送项目上下文之前先评估隐私与安全影响。

hostscan: 命中文件 6，命中行 9
EXIT=0
```

- 改前：6 文件 / 9 行，退出码 `0`
- 改后：6 文件 / 9 行，退出码 `0`
- **命中数变化：无变化**。这不是漏改：9 行全部落在 R1/R3/R4(→保留) 白名单内，`hostscan` 是**扫描器**（命中即列出，不区分是否需改），其退出码与命中数不构成「必须清零」的门禁；本任务的门禁是逐行判定正确 + 未误改。

### 3.2 `check`：no-op，未执行

按任务书规定（「无改动则不执行本条并在报告写明 no-op」），本任务**未执行** `check`，也未生成 `03-impl/T-06-hostscan-check.md`。

- 原因：改动文件集合为空，`--files` 无参数可传；四项判据（AC-2 `en/total <= 0.05`、AC-4 code span 多重集相等、AC-7b 链接目标多重集相等、AC-32 Emoji 不增加）均为「改动前 vs 改动后」的比较型判据，零改动时差集为空，四项**空集成立（vacuously PASS）**。
- 反向确认：未对 6 个文件做任何写入，因此不可能引入 AC-4 / AC-7b / AC-32 回归；`git status --short` 中 6 个文件的状态与执行前一致。

## 4. 残留项与白名单理由

hostscan 仍命中的 9 行**全部**是白名单保留项，逐项理由见 §1。按类别归纳：

1. **历史快照类（R1，3 行）**：`plans/2026-05-29-webui.md:44`、`specs/2026-05-29-webui-design.md:34`、`:142`。文件名带日期、小节为「现状事实（实现时依赖）」/「安全模型（本地）」的定稿快照，改写等于篡改历史；且 §1 已说明其中两行的描述对象本就是 Q2 保留不变的独立二进制默认。
2. **IDE 插件类（R3，4 行）**：`jetbrains/PRIVACY.md:19`、`jetbrains/README.md:241`、`jetbrains/docs/jetbrains.md:44`、`:53`。陈述插件侧 host 设置默认值，实证与 `RustCodeSettingsState.kt:25`（`host = "127.0.0.1"`）一致，与裁决 Q2 一致。
3. **compose 显式写死类（R4→R3，2 行）**：`docker/README.md:186`、`:210`。宿主侧绑定由 `docker-compose.yml:44` 的 `${BIND_ADDR:-127.0.0.1}` 显式决定，与产品默认无关。

非命中行中的 `127.0.0.1` / `localhost`（**均未改动**，按「不得自行扩大 scope」处理）：

- `docker/README.md:146`、`:182`：验证用 `curl http://localhost:13456/...`，是访问示例而非默认绑定叙述。
- `extensions/jetbrains/README.md:227-234`：代码块内的显式 `--host 127.0.0.1` 传参与 curl 冒烟命令；铁律 1 禁止改动代码块，且显式传参语境本就在脚本护栏（`HOST_EXCLUDE_RE`）排除之列。
- `plans/2026-05-29-webui.md:138, 435, 503, 775, 862, 880, 894, 944, 1081, 1440, 1550` 与 `specs/2026-05-29-webui-design.md:58, 92, 154`：历史文档中的验证步骤、命令行、代码块与 dev 模式 URL，属 R1 + 代码块豁免，不动。
- 无一处出现 `0.0.0.0` 之外的矛盾默认叙述；本次也未在任何文件中补充 O-1（TUI 内 `/webui` 默认仍 `127.0.0.1`）说明，因为**无 R2 行**——O-1 补句仅在 R2 改写处才需要，擅自补入 R1/R3 段落会违反「R1 不做任何补充 / 未判定为 R2 的行一律不动」。

## 5. 上报建议（不在本任务 scope，需编排者裁决）

以下三项为取证过程中发现的**潜在**风险，均**未**改动，仅上报：

1. **JetBrains 插件在非 Windows 上可能走 CLI 路径**：`extensions/jetbrains/src/main/kotlin/.../daemon/RustCodeDaemonProcess.kt:61-63` 显示解析顺序为「已配置二进制 → 打包 daemon →（非 Windows 时）`rustcode daemon` CLI」。因此当打包 daemon 缺失时，插件会以 CLI `rustcode daemon`（新默认 `0.0.0.0`）拉起后端，暴露面由 loopback 扩大到全部网卡。前置事实假设 IDE 路径恒为独立二进制，二者存在偏差。建议编排者确认是否需要（a）让插件显式传 `--host`，或（b）在 `extensions/jetbrains/*` 安全说明中补充说明（本任务按 R3 未改）。
2. **`docker/README.md` 的 `docker run -p 13456:13456` 示例（`:107-140` + `:142-150` 验证）**：`Dockerfile-Daemon:43` 的 `ENTRYPOINT` 是独立 `rustcode-daemon`，其默认绑定 `127.0.0.1`（`crates/rustcode-daemon/src/main.rs:21`，且无容器内特判），容器内仅绑 loopback 时宿主侧端口发布不可达。这是**基线既有**问题（该文件 `git diff 3ee655e3` 为空），不属于 hostscan 命中行，本任务按「不得扩大 scope」未改；建议单开任务核实并修正示例（补 `--host 0.0.0.0` 或改述）。
3. **`docker/docker-compose.yml:37` 注释**「容器内默认绑定 127.0.0.1」对新默认（CLI 入口 `0.0.0.0`）而言仅对独立二进制成立。该行是 compose 文件（非 md），不在 files_owned；因镜像实际 ENTRYPOINT 为独立二进制，当前仍准确，暂无需改，供后续统一口径时参考。

## 6. 完成声明

- 只读取与只运行了 `python3 scripts/check-zh-docs.py`；未执行 `cargo` / `npm`；未改源码；未改 files_owned 之外的 md；未执行 `git commit` / `git add`。
- 报告无「（待填）」等占位符；所有结论均来自上文的实证行与真实命令输出。
