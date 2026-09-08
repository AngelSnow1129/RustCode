---
kind: review
id: CODE-REVIEW-R1
from: code-reviewer
to: [code-implementer, project-manager]
feature: 2026-09-07-zh-docs-webui
status: changes_requested
decision: rework
requires: [T-01, T-02, T-03, T-04, T-05, T-06, T-08, T-09]
files_owned: []
created: 2026-09-09
---

# CODE-REVIEW · 代码/脚本侧审查（`2026-09-07-zh-docs-webui`，第 1 轮）

> 审查对象：**未提交的工作区改动**，基线 `ZH_BASE=3ee655e3`，分支 `dev`（`HEAD` 仍等于基线）。
> md 汉化正文不在本轮审查范围（由 `scripts/check-zh-docs.py` 的 AC-2/4/6/7b/32 机检把关）。
> 本轮为**只读**审查：未修改任何源码、脚本、文档；唯一新增文件是本报告。

---

## 1. 审查范围

### 1.1 取差方式

- 已跟踪文件：`git diff 3ee655e3 -- <path>`
- 未跟踪新文件（无 diff，直接读全文）：`scripts/check-zh-docs.py`、`scripts/build-webui.sh`、`crates/rustcode-daemon/tests/default_host_lock.rs`

### 1.2 在审文件清单（与编排者给定范围逐条比对）

| 文件 | 任务 | 性质 | 是否在 `files_owned` | 结论 |
|---|---|---|---|---|
| `crates/rustcode-cli/src/main.rs` | T-03 | `:1045-1047` doc comment + clap `default_value`；`:1780` `ServerOpts.host`；`:5467-5517` 新增 `#[cfg(test)] mod default_host_tests` | `files_owned: [crates/rustcode-cli/src/main.rs]` | 一致 |
| `crates/rustcode-daemon/src/lib.rs` | T-04 | `:6333-6341` 注释改写 + 删除 `DaemonWarnNonLoopback` 打印块（单 hunk，+9/-14） | `files_owned: [crates/rustcode-daemon/src/lib.rs]` | 一致 |
| `crates/rustcode-config/src/i18n/en.rs` | T-05 | `:1191-1192` `CliWebuiNotBuilt`、`:2521` `CliHelpHost` | `files_owned` 内 | 一致 |
| `crates/rustcode-config/src/i18n/zh_cn.rs` | T-05 | `:1135-1136` `CliWebuiNotBuilt`、`:2429` `CliHelpHost` | `files_owned` 内 | 一致 |
| `crates/rustcode-daemon/tests/default_host_lock.rs` | T-03 | 新增 46 行源码文本断言 | T-03 `files_owned` 之外，**报告 §2.3 已显式声明偏差** | 已声明，接受 |
| `scripts/check-zh-docs.py` | T-01/T-08/T-09 | 新增 1422 行 | 新增脚本 | 一致 |
| `scripts/build-webui.sh` | T-02 | 新增 244 行 | 新增脚本 | 一致 |

**越界改动：无。** 未发现任何落在上述清单之外的源码/脚本改动被夹带。
（`git status --porcelain scripts/` 仅两个 `??` 新文件；`__pycache__/` 已被 `.gitignore:23` 覆盖。）

---

## 2. 裁决

**`request-changes` / `decision: rework`**，路由回 `code-implementer`。

一句话理由：安全主线成立（两条 `0.0.0.0` 入口的 token 鉴权仍生效、审批不翻转、独立二进制默认地址仍为回环且被测试锁定），AC-4/AC-8 判据经对抗性验证确为 fail-closed，但残留 3 处「注释/文档与行为不一致 + 阈值可被静默放宽」的缺陷，其中 2 处是 T-04 本应消灭的同类失实注释，必须修。

| 级别 | 条数 |
|---|---|
| blocker | 0 |
| critical | 0 |
| major | 3 |
| minor | 5 |
| nit | 2 |

**必须修（MUST）**：MAJOR-1、MAJOR-2、MAJOR-3。
**可选改进（SHOULD/MAY）**：MINOR-1 ~ MINOR-5、NIT-1 ~ NIT-2（不阻断合入，建议同批或登记 TODO）。

---

## 3. 问题清单

### 3.1 MAJOR-1 · `ServerOpts.host` 的文档注释仍声称「非回环会打印安全告警」，与删除告警后的行为直接矛盾

| 项 | 内容 |
|---|---|
| **级别** | major |
| **证据** | `crates/rustcode-daemon/src/lib.rs:6034`：<br>`/// Bind host (e.g. \`127.0.0.1\`). Non-loopback hosts emit a security warning.` |
| **影响** | T-04 已按 Q3 裁决 A 删除 `DaemonWarnNonLoopback` 打印块（`lib.rs:6345-6347` 旧址），`run_server` 现在**不再**对非回环 host 打印任何告警（`grep -n "DaemonWarnNonLoopback" crates/rustcode-daemon/src/lib.rs` 命中 0）。但 `ServerOpts` 是 `pub` 结构体、该字段是构造 server 的必填入参，其文档仍在承诺一个已不存在的行为。下游调用方（CLI/TUI/独立二进制/未来新增 driver）会据此误判「绑定 0.0.0.0 时框架会替我告警」，从而在新增入口时省掉自己的提示逻辑——这正是本次把 `rustcode daemon` 改成 `0.0.0.0` 后 MAJOR-3 暴露出来的缺口。 |
| **判据** | `00-decisions.md` Q3「连带必修」：`lib.rs:6333-6341` 的失实叙述「必须改写」，不得保留与新默认矛盾的因果叙述；`02-tasks.md` T-04 K5。本条是**同一缺陷类在相邻行未被覆盖**。 |
| **最小修复** | 把 `lib.rs:6034` 改为与 `:6333-6341` 一致的中性表述，例如：<br>`/// Bind host (e.g. \`0.0.0.0\` / \`127.0.0.1\`). Decided by the driver; this function does not warn on non-loopback binds.`（一行，零行为变更） |
| **验证方式** | `grep -rn "emit a security warning" crates/` 命中 0；`cargo doc -p rustcode-daemon` 可构建。 |

### 3.2 MAJOR-2 · `--en-ratio 0` 被静默改写为默认 0.05，阈值收紧请求被无声放宽

| 项 | 内容 |
|---|---|
| **级别** | major |
| **证据** | `scripts/check-zh-docs.py:1404-1405`：<br>`if not getattr(args, "en_ratio", None):`<br>`    args.en_ratio = EN_RATIO_DEFAULT`（`EN_RATIO_DEFAULT = 0.05`，`:25`） |
| **影响** | argparse 把 `--en-ratio 0` 存成 `0.0`，`not 0.0` 为真，于是**用户显式要求的最严阈值被静默替换为 0.05**。实测复现：`en_ratio=0.0 -> 实际生效阈值 0.05`。后果是把闸门往松的方向调且**无任何提示**：一个 en_ratio=0.03 的文件在 `--en-ratio 0` 下应 FAIL，实际 PASS。方向与 fail-closed 相反。 |
| **判据** | 审查维度 4「退出码与阈值语义正确」；`AGENTS.md:154`「构建/准备/组装/恢复任一步失败时是否显式失败，而非静默 fresh / 假成功」。 |
| **最小修复** | 改为显式判 `None`：<br>`if getattr(args, "en_ratio", None) is None:`<br>`    args.en_ratio = EN_RATIO_DEFAULT`<br>并补一条断言：`en_ratio < 0` 时以退出码 2 报用法错误。 |
| **验证方式** | `python3 -c` 载入模块后以 `args.en_ratio=0.0` 走同一分支，断言结果仍为 `0.0`；`scripts/check-zh-docs.py check --files README.md --en-ratio 0` 的判定不得比 `--en-ratio 0.05` 更松。 |

### 3.3 MAJOR-3 · `rustcode daemon`（本次改为 0.0.0.0）在删除横幅后 0 风险提示，而新注释声称提示由 `WebuiLanWarning` 承担

| 项 | 内容 |
|---|---|
| **级别** | major |
| **证据** | 新注释 `crates/rustcode-daemon/src/lib.rs:6337-6341`：<br>`// ... This function no longer prints a host-dependent warning; the`<br>`// non-loopback risk notice is owned by \`Msg::WebuiLanWarning\` /`<br>`// \`Msg::WebuiNonLoopbackWarning\`, which are emitted together with the`<br>`// reachable URLs.`<br>但这两个 `Msg` 的**唯一发射点**是 `ensure_server_and_open`（`lib.rs:5398-5407`），而 `rustcode daemon` 走 `cli/src/main.rs:1779` 直连 `run_server`（`quiet: false`），只打印 `Msg::DaemonListening { addr }`（`lib.rs:6354`，内容为 `0.0.0.0:13456`）。<br>已验证：`Msg::WebuiLanWarning` / `Msg::WebuiNonLoopbackWarning` 在 `lib.rs` 中仅出现于 `:5402` / `:5405`（发射）与 `:6338-6339`（注释），无第二发射点。 |
| **影响** | `rustcode daemon` 本次从 `127.0.0.1` 改为 `0.0.0.0`，是两条入口中**唯一没有任何 CLI 逃生开关**的一条（`Commands::Daemon` 无 `--host` 形参）。删除横幅后，该路径的用户可见信息只剩一行监听地址，不再有「无 TLS / token 即权限 / 仅限可信网络」的可操作提示。同时新注释对这条路径的表述是假的：它把风险提示的责任指派给了一个该路径不会执行的函数。Q3 理由 3 的立论正是「风险信息由 `WebuiLanWarning` 承载、随访问地址一起输出」，该立论对 `rustcode daemon` 不成立。 |
| **判据** | `00-decisions.md` Q3 理由 1-3（删除横幅的前提是风险信息另有承载）；审查维度 1（暴露面变更的可用性/可感知性）与维度 6（注释与代码行为一致）。 |
| **最小修复** | 二选一，**优先 A**：<br>**A（推荐，2-3 行）**：在 `run_server` 非 quiet 分支（`lib.rs:6353` 之后）对非回环 addr 追加 `Msg::WebuiLanWarning`（wildcard `0.0.0.0`/`::`）或 `Msg::WebuiNonLoopbackWarning`（具体 IP），复用既有 `is_loopback_authority`。<br>**B（若 PM 判定该路径按 Q3 无需提示）**：把 `:6337-6341` 的注释改为只在 `ensure_server_and_open` 路径成立的表述，并写明「`rustcode daemon` 无风险提示」这一已知事实。 |
| **验证方式** | `RUSTCODE_HOME=/tmp/x timeout 10 ./target/debug/rustcode daemon --port <p>` 的 stderr/stdout 含 LAN/非回环提示；`cargo test -p rustcode-daemon --lib` 仍 0 failed。 |

### 3.4 MINOR-1 · `ensure_server_and_open` 的文档注释仍写「默认 `127.0.0.1`」

| 项 | 内容 |
|---|---|
| **级别** | minor |
| **证据** | `crates/rustcode-daemon/src/lib.rs:5275`：`/// \`host\` 为绑定地址（默认 \`127.0.0.1\`；\`0.0.0.0\` 暴露到局域网/外网）。` |
| **影响** | 该函数本身无默认值（默认由调用方决定），而 `rustcode webui` 调用方现已是 `0.0.0.0`（`cli/src/main.rs:1047`）。读注释者会以为默认仍是回环。 |
| **判据** | 一致性（维度 6）。 |
| **修复** | 改为「（CLI 默认 `0.0.0.0`；TUI `/webui` 默认 `127.0.0.1`，见 O-1）」。 |

### 3.5 MINOR-2 · 新增测试的立论注释与代码现状矛盾：独立二进制并非「无 token」

| 项 | 内容 |
|---|---|
| **级别** | minor |
| **证据** | `crates/rustcode-daemon/tests/default_host_lock.rs:3-6`：<br>`//! 独立二进制会被 VS Code / JetBrains 插件在**不传 \`--host\`** 且**无 token** 的情况下拉起。一旦默认地址被改成 \`0.0.0.0\`，会造成「无鉴权暴露 + Build 模式审批静默放行」的双重回归。`<br>但 `crates/rustcode-daemon/src/main.rs:173` 传入 `webui_tokens: Some(token_store)`，配合 `lib.rs:6152` `enforce_token: webui_tokens.is_some()` ⇒ **独立二进制 `enforce_token = true`**，且 `:180` `daemon_token_file: Some(daemon_token)` 会写出 `~/.rustcode/daemon-<port>.json`。既有测试 `tests/daemon_token_auth.rs:39-49` 已断言「无 token 访问受保护路由必须 401」。 |
| **影响** | 安全结论方向没错（不该改 `DEFAULT_HOST`），但**理由写错了**，且写进的是新代码。后续维护者一旦发现「其实有 token」，可能连带推翻这条锁定测试本身（测试断言正确、立论错误，是最容易被误删的组合）。`00-decisions.md` Q2 理由 3 的同一前提（`webui_tokens=None ⇒ enforce_token=false`）也是错的。 |
| **判据** | 一致性（维度 6）；注释不得陈述可证伪的事实。 |
| **修复** | 把注释改为正确立论：独立二进制虽 `enforce_token=true`，但由 IDE 无人值守拉起、token 落盘于 `~/.rustcode/daemon-<port>.json`、无审批交互方，故默认仍须回环；并同步知会 PM 修正 `00-decisions.md` Q2 理由 3。测试体本身**不要动**（断言正确）。 |

### 3.6 MINOR-3 · 同文件另两处「VSCode daemon `enforce_token=false`」的注释亦与代码矛盾（既存，不在 `files_owned`）

| 项 | 内容 |
|---|---|
| **级别** | minor |
| **证据** | `crates/rustcode-daemon/src/lib.rs:5264-5265`：`/// VSCode 扩展自带的守护进程以 \`enforce_token=false\`（不带 token）在 13456 上工作。`<br>`lib.rs:5442-5443`：`/// - **daemon 模式**（\`webui_tokens=None\` -> \`enforce_token=false\`）：...`<br>对照 `daemon/src/main.rs:173` `webui_tokens: Some(token_store)` ⇒ `enforce_token=true`。 |
| **影响** | 与 MINOR-2 同源；会持续误导安全判定。 |
| **判据** | 一致性（维度 6）。 |
| **修复** | 不在本 feature `files_owned`，**不建议本轮改**。建议 PM 单开一条 TODO 登记（可并入 `06-release.md` 已知项），避免本轮越权扩面。 |

### 3.7 MINOR-4 · `Msg::DaemonWarnNonLoopback` 已成为无发射点的死变体，缺「为何保留」的标记

| 项 | 内容 |
|---|---|
| **级别** | minor |
| **证据** | 全仓仅剩 3 处引用，均在 `rustcode-config`：`messages.rs:4899`（变体定义）、`en.rs:3099`、`zh_cn.rs:2949`（两语种 `match` 臂）；`rustcode-daemon` 内命中 0。 |
| **影响** | 契约（T-04 K4）要求保留变体与两语种文案，这一点已遵守。但没有任何注释说明「此变体已无发射点、保留是契约要求」，后续极易被当作死码删除，或被误认为「非回环告警还在」——后者与本次删除告警的意图直接冲突。 |
| **判据** | 可维护性；`00-decisions.md` Q3 裁决 A。 |
| **修复** | 在 `messages.rs:4899` 上方加一行注释：`// 已无发射点：Q3 裁决删除启动横幅；变体与两语种文案按契约保留，勿当死码清理。` |

### 3.8 MINOR-5 · `gate` 与 `check --diff` 的文件分母口径不一致，未跟踪的新 md 会逃过 gate

| 项 | 内容 |
|---|---|
| **级别** | minor |
| **证据** | `scripts/check-zh-docs.py:161-163` `list_md_files()` = `git ls-files -- *.md`（仅已跟踪），`cmd_gate` 用它做分母（`:1258`）；而 `resolve_files()`（`:924`）额外并入 `git ls-files --others --exclude-standard -- *.md`。 |
| **影响** | 一个**新建且未 `git add`** 的 md 不会进入 AC-1 恒等式分母，也不受 AC-2/4/6/7b/32 与 AC-8 段 1 约束，直到被 add。对本 feature 现状无影响（`scripts/` 新增的是 `.py`/`.sh`，artifacts md 均已跟踪），但对后续使用是个静默口子。 |
| **判据** | fail-closed（维度 2）；AC-1「清单恒等式」的分母定义应与全量判定一致。 |
| **修复** | 或在 `list_md_files()` 并入 `--others --exclude-standard`，或在 `cmd_gate` 显式打印「未跟踪 md 已排除：N 个」并在 N>0 时 FAIL（后者不改动 AC-1 分母定义，风险更低）。 |

### 3.9 NIT-1 · `cmd_hostscan` 恒返回 0

`scripts/check-zh-docs.py:1031`：`return 0`（无论命中多少文件）。作为 T-06 的清单产出器语义可接受，但若被 CI 当门禁调用会假绿。建议在 `--help`/docstring 写明「本子命令非门禁，恒 0」。（仅文档性改，不阻断。）

### 3.10 NIT-2 · 非法 `--base` 时 `gate` 返回 1 而非 2，可诊断性欠佳

实测：`python3 scripts/check-zh-docs.py gate --base deadbeefdeadbeef` → `EXIT=1`。原因是 `read_base_file()`（`:177-182`）把 `git show` 失败静默当成「基线无此文件」（退化为与空文档比较），于是全量文件被判为新增、AC-4 大量 FAIL。**结果是红的（fail-closed 成立）**，但根因被淹没在几百条 FAIL 里。建议在 `gate` 入口先 `git rev-parse --verify <base>`，失败即 `EnvError` → 退出码 2。（不阻断。）

---

## 4. 已核对无问题的维度

### 4.1 安全性（维度 1）—— 结论：**无回归，Q2 的安全前提成立**

| 入口 | 绑定地址 | `enforce_token` | 判定依据 | 结论 |
|---|---|---|---|---|
| `rustcode webui` | `0.0.0.0`（`cli/src/main.rs:1047`，新增测试锁定） | **true** | `ensure_server_and_open`（`lib.rs:5321`）传 `webui_tokens: Some(tokens.clone())` → `lib.rs:6152` `enforce_token: webui_tokens.is_some()` | 受保护路由 401（既有 `tests/daemon_token_auth.rs:39-49` 已断言） |
| `rustcode daemon`（CLI 子命令） | `0.0.0.0`（`cli/src/main.rs:1780`） | **true** | `cli/src/main.rs:1784` `webui_tokens: Some(token_store)`，`daemon_token_file: Some(daemon_token)` | 同上 |
| TUI `/webui` | `127.0.0.1`（`tuix/.../commands.rs:2207`，未改，O-1） | true | 未改 | 无变化 |
| 独立 `rustcode-daemon` 二进制 | `127.0.0.1`（`daemon/src/main.rs:21`，被 `tests/default_host_lock.rs` 锁定） | **true** | `daemon/src/main.rs:173` | 未改，回归哨兵已就位 |

- **`client_interactive_permission` 未被翻转**：`lib.rs:1284-1294`，`enforce_token=true` 时短路返回 `true`，与 `is_loopback_authority(bind_host)` 的取值无关。既有测试 `known_clients_interactive_on_loopback_or_token`（`lib.rs:8850-8886`）已包含 `ClientMode::Ide + true + "0.0.0.0"` 的正向断言，实测通过 ⇒ Build 模式审批不会静默放行。
- **无「无鉴权 + 工具执行」路径**：`require_webui_token`（`auth_token.rs:115-143`）在 `enforce_token=true` 且 token 缺失/无效时返回 `UNAUTHORIZED`；两条新入口都传了 `Some(store)`。
- **`is_loopback_authority` 的其它两处消费**（`lib.rs:5369` 选浏览器地址、`:5673` 可达性判定）语义在新默认值下正确：`0.0.0.0` 走「探测局域网 IP」分支（`:5356-5377`），并触发 `Msg::WebuiLanWarning`（`:5402`）。
- `dangerously_skip_permissions`（`lib.rs:4671-4674`，`00-decisions.md` N4 已登记 fail-open 隐患）**未被本次改动触及**，不在本 feature 范围。

### 4.2 fail-closed（维度 2）—— 结论：**AC-4 / AC-8 判据经对抗性验证均不可绕过**

对 `ac4_verdict`（`check-zh-docs.py:495-566`）做了 10 组对抗性输入（直接 import 模块调用，未改动任何文件）：

| 用例 | 期望 | 实测 |
|---|---|---|
| 无 span 变化 | PASS | PASS |
| 新增任意未登记 span | FAIL | FAIL：`越界新增（未登记...）` |
| 新增已登记 `./scripts/build-webui.sh`（README.md，1 次） | PASS(D2) | PASS(D2) |
| 该登记 span 出现 2 次 | FAIL | FAIL（超 `AC4_MAX_PER_ENTRY`，且两条一起判红，比「首条放行」更严） |
| 移除无关 span | FAIL | FAIL：`越界移除（非 D1）` |
| D1 整串消失 | PASS(D1) | PASS(D1) |
| D1 最小剔除（S2 `" / README.zh-CN.md"`） | PASS(D1) | PASS(D1) |
| D1 剔除后 added 串不符（`x` vs `y`） | FAIL | FAIL（同时把 `y` 判为越界新增） |
| D1 放行但 `new_text` 残留 `README.zh-CN` | FAIL（半清理护栏） | FAIL |
| **借 D1 洗白额外新 span**（D1 + `evil`） | FAIL | FAIL：`evil` 判越界新增 |

结论：**不存在「新增 span 也能过」的口子**——`ac4_excise` 的输出是被冻结基线 span 集合决定的确定性函数，攻击者无法用它凭空造出任意新串；`remaining` 逐条按文件白名单 + 计数上限兜底。

AC-8（`gate_check_ac8`，`:1169-1253`）护栏：
- **A1**（正式域非空）、**A2**（`README.md`/`AGENTS.md` 锚点在正式域）：实现额外加了「非退化前缀」前置条件（`:1193-1200`），把 `AC8_EXEMPT_PREFIX` 取 `.` / `/` / 缺尾斜杠的情况直接判 FAIL —— 即已知偏差 DEV-1，属**加严**，确认未削弱强度。
- **A3**：实测全仓 `README.zh-CN` 命中**全部**位于 `.codebuddy/artifacts/`（`git grep -l -F "README.zh-CN" -- '*.md' ...` 过滤后非 `.codebuddy/` 命中为 0）。因此把豁免前缀改成任何其它目录都会因 A3「历史域必须仍有命中」而 FAIL ⇒ **「改豁免前缀也能过」的口子不存在**。
- **A4**：段 2 命中行越界检查（`:1227`）在位。

### 4.3 AGENTS.md 硬约束（维度 3）—— 结论：**全部合规**

| 约束 | 核对结果 |
|---|---|
| 不新增第二运行时生命周期所有者 | 合规。改动仅为 clap 默认值字面量、`ServerOpts` 字段字面量、注释、i18n 串、两个测试、两个独立脚本。无新 runtime / 无新生命周期 owner。 |
| `kernel / capabilities / coding` 生产依赖 core-free | 本次**未触碰**这三个 crate；`git diff 3ee655e3 --stat` 中三者只有 md 变更。 |
| 禁止 Unicode Emoji（`AGENTS.md:59,172`） | 合规。`git diff 3ee655e3 -U0` 的新增行中 emoji 命中 0；`scripts/build-webui.sh` emoji 命中 0；脚本输出一律 `[INFO]`/`[WARN]`/`[ERROR]`/`[SUCCESS]` ASCII 标签。 |
| 禁止硬编码真实密钥/内部端点 | 合规。新增代码只含 `0.0.0.0` / `127.0.0.1` / `./scripts/build-webui.sh`；`build-webui.sh` 只引用 `nodejs.org`、`github.com/nodesource` 等公开安装指引（`AGENTS.md:55-58` 在 `install.sh` 中已有同类先例）。 |
| 禁止裸 `.unwrap()`/`.expect()`（`AGENTS.md:176`，测试除外） | 合规。新代码中的 `panic!` / `.unwrap_or_else(|e| panic!(...))` / `.expect("...")` 全部位于 `#[cfg(test)]` 与 `tests/`，且均带显式消息（属豁免项）。生产路径零新增 unwrap。 |
| 异步链路无阻塞 I/O | 合规。本次无新增异步代码；`run_server` 调用链未变。 |
| 禁止 `sudo`（`AGENTS.md:18`） | 合规。`build-webui.sh` 中 `sudo` 仅出现 5 次且**全在提示文案里**（`:12,53,58,186,243`），无一处执行。 |
| 未恢复 bridge / v1-v2 开关 / core driver fallback / core session 磁盘模型 | 合规，零相关改动。 |
| turn completion / compaction 重叠 hook | 不涉及。 |

### 4.4 脚本正确性（维度 4）—— 结论：**`build-webui.sh` 全项符合契约；`check-zh-docs.py` 仅 MAJOR-2 / MINOR-5 两项**

`scripts/build-webui.sh`：

| 契约项（01-design.md §4.2） | 实测 |
|---|---|
| `set -euo pipefail` | `:16` 在位 |
| 退出码 0/1/2 | `--help` → 0；参数过多 → 2；未知参数 → 2（实测）；`npm ci` / `npm run build` 失败 → `:212` / `:221` 显式 `exit 1` 并透传原始 stderr；缺 lockfile / 缺 node / 版本不足 → `exit 2` |
| node 版本从 `engines.node` 动态解析，失败回退 22.6 | 实测真实 `webui/package.json`（`engines.node = ">=22.6"`）解析出 `22.6`；`PKG_JSON` 缺失时回退 `22.6` 并打 `[WARN]` |
| 版本比较边界 | 隔离测试：`v22.13.1/22.6` OK、`22.6/22.6` OK、`22.5/22.6` 判过旧、`23.0.0/22.6` OK、`21.9/22.6` 判过旧、`22/22.6` 判过旧、`22.6.0/22.6` OK —— 全部符合语义 |
| 成功结尾打印 `cargo clean -p rustcode-daemon` | `:240-241` 在位 |
| 半产出 dist 判失败（E6） | `:224-230` 后置校验 `dist/index.html` 不存在即 `exit 1` |
| 不破坏 `crates/*/build.rs` 不调用 npm 的约束 | `crates/rustcode-capabilities/build.rs`、`crates/rustcode-cli/build.rs` 均无 npm/node 引用；脚本本身不触碰 `build.rs` |
| 语法 | `bash -n scripts/build-webui.sh` → SYNTAX_OK |

`scripts/check-zh-docs.py`：

- **stdlib-only、无网络**：`import` 仅 `argparse / os / posixpath / re / subprocess / sys`（`:17-22`）+ 函数内 `collections`；无 `urllib`/`socket`/`requests`。
- **退出码语义**：实测 `check` 无参 → 2；`check --files NOPE.md` → 2；`check --files README.md` → 0；无子命令 → 2；`gate` → 0；`gate --base <非法>` → 1（红，fail-closed，见 NIT-2）。`0/1/2` 语义成立。
- 基线固定：`resolve_base()`（`:1381-1388`）顺序为 显式 `--base` > `ZH_BASE` > `DEFAULT_BASE("3ee655e3")`，**禁止回落 HEAD**，符合契约。

### 4.5 测试与隔离（维度 5）—— 结论：**合规**

- `tests/default_host_lock.rs`：只读 `CARGO_MANIFEST_DIR/src/main.rs` 文本，**不触碰 `~/.rustcode`**、不起端口、不设 `RUSTCODE_HOME`。fail-closed 验证：`find(...)` 找不到 `const DEFAULT_HOST` 时 `panic!` 并提示必须同步更新（`:33-38`）；常量改名/移动/改值都会红。实测 `cargo test -p rustcode-daemon --test default_host_lock` → `1 passed`。
- `cli/src/main.rs` 新增 `mod default_host_tests`（2 例）：仅做 clap 解析断言，无 `HOME` 依赖、无 I/O。实测 `cargo test -p rustcode --bin rustcode default_host_tests` → `2 passed`。
- 未引入对真实 `~/.rustcode` 的依赖（既有 `tests/daemon_token_auth.rs` 已用 `RUSTCODE_HOME` 重定向作先例，本次新增测试不需要）。

### 4.6 一致性（维度 6）—— 结论：**i18n 成对；注释一致性见 MAJOR-1 / MAJOR-3 / MINOR-1~4**

- 双语种同构：`CliWebuiNotBuilt`（`en.rs:1191-1192` / `zh_cn.rs:1135-1136`）、`CliHelpHost`（`en.rs:2521` / `zh_cn.rs:2429`）**两语同批更新**，无单语种漂移；`Msg` 的 `match` 在两语种均穷尽（非穷尽即为编译错误），编译通过 ⇒ 无遗漏臂。
- `CliHelpHost` 的实际生效点已核对：`cli/src/main.rs:477` `.mut_arg("host", |a| a.help(t(Msg::CliHelpHost)...))`，与 `:1045-1046` 的 clap doc comment 内容一致（都声明默认 `0.0.0.0`）。
- `CliWebuiNotBuilt` 生效点为 `daemon/src/webui.rs:32`；文案推荐的 `./scripts/build-webui.sh` 真实存在且可执行（`-rwxr-xr-x`），README.md:200 亦已同步引用。
- O-1（CLI `webui` 默认 `0.0.0.0` vs TUI `/webui` 默认 `127.0.0.1` 不一致）已按设计在 `README.md:120-121` 文档化解，且文中「等价写法 `lan`」经核对确有实现（`tuix/.../commands.rs:2191`）⇒ 文档准确。
- 注释与行为一致性：**存在 4 处不一致**，已列为 MAJOR-1 / MAJOR-3 / MINOR-1 / MINOR-2（另 MINOR-3 为既存同类项）。

---

## 5. 契约符合性

| 契约来源 | 要求 | 实现 | 结论 |
|---|---|---|---|
| `00-decisions.md` Q2 裁决 A | 只改 `cli/src/main.rs:1047` 与 `:1780`；`daemon/src/main.rs:21` `DEFAULT_HOST` 保持 `127.0.0.1` | `:1047` / `:1780` 已改；`daemon/src/main.rs:21` 未改（git diff 确认），并被新增测试锁定 | 一致（注：Q2 理由 3 的事实前提有误，见 MINOR-2，但**裁决方向不变、实现正确**） |
| `00-decisions.md` Q3 裁决 A | 删除 `DaemonWarnNonLoopback` 横幅；保留 `WebuiLanWarning`；连带改写 `:6333-6341` 失实注释 | 打印块整块删除（daemon crate 内命中 0）；变体与两语种文案保留；注释已改写且 `PR #82`/`loopback-only for security` 表述全消失 | 一致（注释改写**未覆盖** `lib.rs:6034` 同类失实表述，见 MAJOR-1） |
| `01-design.md` §4.3 K1-K3 / §4.5 | 逐处字面量替换；不新增 `enum`/`trait`/`struct`/事件/协议字段 | 两处字面量 + doc comment；零新增公共类型 | 一致 |
| `01-design.md` §11 O-1 | TUI `/webui` 保持 `127.0.0.1`，靠 README 说明化解 | 未改；`README.md:120-121` 已写明 | 一致 |
| `01-design-addendum.md` §2 AC-4 v2 | 冻结算子 `ac4_excise` S1-S6、判定顺序 1-5、一致性护栏、可见性 | 逐条实现，顺序与补遗一致；10 组对抗性输入验证不可绕过 | 一致 |
| `01-design-addendum.md` §3 AC-8 v2 | 三段式 + A1-A4 护栏；排除集恒 1 成员 | 实现且额外加严退化前缀断言（DEV-1） | 一致（加严） |
| `01-design.md` §4.2 T-02 脚本契约 | 见 4.4 表格 | 全项实测符合 | 一致 |

**未声明的契约偏差：无。**
**已知偏差复核**：
- **DEV-1**（A2 退化前缀加严断言）：复核为**纯加严**——只在 `AC8_EXEMPT_PREFIX` 取非法值时新增一条 FAIL，对冻结值 `.codebuddy/artifacts/` 恒成立，不减少任何正式文档判定。**不削弱强度，接受**。
- **DEV-3**（补遗登记 11 条 vs 实测 12 条 AC-4 放行）：实测 `gate` 输出 `AC-4 授权放行合计 12 条（D1 11 / D2 1）`，与实测一致；判据本身未放宽，按补遗 §6「以 check 输出为权威」处理。**接受**。

---

## 6. 测试覆盖评估

| 路径 | 覆盖情况 | 评价 |
|---|---|---|
| `rustcode webui` 默认 `0.0.0.0` | 新增 `webui_defaults_to_all_interfaces` | 覆盖了 AC-19 正向 |
| 显式 `--host` 覆盖默认 | 新增 `webui_explicit_host_overrides_default`（3 组值，含与默认同值） | 覆盖了「显式赢过默认」不变式 |
| 独立二进制 `DEFAULT_HOST` 保持回环 | 新增 `standalone_daemon_default_host_stays_loopback`（源码文本断言，含「找不到常量即 panic」） | fail-closed 成立 |
| 审批交互性在 token + 非回环下不翻转 | **既有** `known_clients_interactive_on_loopback_or_token`（含 `Ide + true + "0.0.0.0"`） | 已覆盖，无需新增 |
| 受保护路由无 token 401 | **既有** `tests/daemon_token_auth.rs` | 已覆盖 |
| 两条 `0.0.0.0` 入口的**集成**鉴权验证（真实起服务 + 跨网卡访问） | **无** | 建议后续补：但既有 `daemon_token_auth.rs` 已覆盖同一 `run_server` + `Some(store)` 组合，风险已部分对冲；本轮不作为必须项 |
| `check-zh-docs.py` 判据 | **无自测**（脚本本身没有被测） | 本轮以对抗性手工调用替代（见 4.2）。建议后续为 `ac4_verdict` / `ac4_excise` / `version_ge` 各补几组表驱动用例；**本轮不作为必须项** |
| 失败/取消路径 | 本次改动不引入新失败/取消语义（无新异步操作、无新状态机） | 不适用 |

无被跳过的用例（`cargo test -p rustcode-daemon`：`0 ignored`）。

---

## 7. 验证复述（本轮实际执行的命令与结果）

| # | 命令 | 结果 |
|---|---|---|
| 1 | `git status --porcelain` / `git rev-parse HEAD` / `git branch --show-current` | `HEAD=3ee655e381d28052e428196daedd456cc6079520`，分支 `dev`，符合「改动全在未提交工作区」 |
| 2 | `git --no-pager diff 3ee655e3 --stat` | 161 files changed；代码侧仅 4 个 `.rs` 在范围内 |
| 3 | `git --no-pager diff 3ee655e3 -- crates/rustcode-cli/src/main.rs crates/rustcode-daemon/src/lib.rs crates/rustcode-config/src/i18n/{en,zh_cn}.rs` | 与实现报告声明的 hunk 逐条吻合；无夹带 |
| 4 | `grep -rn "DaemonWarnNonLoopback" --include=*.rs .` | 3 处，全在 `rustcode-config`；`rustcode-daemon` 内 0 |
| 5 | `grep -rn "WebuiLanWarning\|WebuiNonLoopbackWarning" --include=*.rs .` | 发射点仅 `lib.rs:5402` / `:5405` |
| 6 | `grep -rn "enforce_token" --include=*.rs crates/` | `lib.rs:6152` `enforce_token: webui_tokens.is_some()`；三个调用点均传 `Some`（`/app` 传 `None`，未改） |
| 7 | `ZH_BASE=3ee655e3 python3 scripts/check-zh-docs.py gate` | **`GATE_EXIT=0`**；AC-1 PASS（SKIP_A=4 SKIP_B=47 SKIP_C=1 SKIP_D=2 TODO=29 ZH=202 total=285）；全量 check 受检 231 / FAIL 0；AC-4 授权放行 12 条（D1 11 / D2 1）；AC-5/AC-7a/AC-8 均 PASS |
| 8 | 对抗性单测 `ac4_verdict`（import 模块，10 组） | 见 4.2 表；无一组可绕过 |
| 9 | `--en-ratio 0` 分支复现 | `en_ratio=0.0 -> 实际生效阈值 0.05`（MAJOR-2） |
| 10 | `git grep -l -F "README.zh-CN" -- '*.md' '*.html' '*.json' '*.ts' '*.kt' '*.yml' '*.toml'` | 命中全部位于 `.codebuddy/artifacts/`；非 `.codebuddy/` 命中 0 |
| 11 | `cargo fmt --check` | `FMT_EXIT=0` |
| 12 | `cargo test -p rustcode-daemon`（全目标） | `EXIT=0`：lib 307 passed / main 0 / `daemon_token_auth` 1 passed / `default_host_lock` **1 passed** / `legacy_turn_boundary_repair` 7 passed；`0 ignored` |
| 13 | `cargo test -p rustcode --bin rustcode default_host_tests` | `EXIT=0`，`2 passed`（`webui_defaults_to_all_interfaces`、`webui_explicit_host_overrides_default`） |
| 14 | `cargo clippy -p rustcode-daemon -p rustcode -p rustcode-config --all-targets` | `CLIPPY_EXIT=0`；改动区间（lib.rs:63xx、main.rs:104x/54xx、i18n:1189/1133）零告警 |
| 15 | `bash -n scripts/build-webui.sh` / `--help` / `a b` / `--bogus` | SYNTAX_OK；`HELP_EXIT=0`；`ARG_EXIT=2`；`UNK_EXIT=2` |
| 16 | 隔离测试 `version_ge`（7 组）与 `parse_node_min`（真实 + 缺失 package.json） | 全部符合语义；解析 `22.6`，缺失回退 `22.6` |
| 17 | `python3 scripts/check-zh-docs.py check`（无参 / `--files NOPE.md` / `--files README.md` / 无子命令）、`gate --base deadbeefdeadbeef` | 2 / 2 / 0 / 2 / 1 |
| 18 | `grep -rn "sudo" scripts/build-webui.sh`、`git check-ignore -v scripts/__pycache__`、网络 import 扫描、emoji 扫描 | 5 处全在提示文案；`__pycache__/` 已忽略；无网络 import；改动行 emoji 0 |

全部为只读命令（含 `cargo test` / `cargo clippy`，仅写 `target/` 构建缓存）；未执行任何写入型命令，未修改任何源码、脚本或既有文档。

---

## 8. 结构化发现（供 `ReportFindings` 通道提交）

| # | file | summary | failure scenario | category |
|---|---|---|---|---|
| MAJOR-1 | `crates/rustcode-daemon/src/lib.rs:6034` | `ServerOpts.host` 文档仍称「非回环会打印安全告警」，该告警已被 T-04 删除 | 新增调用 `run_server` 的 driver 以为框架会替自己告警，从而在绑定 `0.0.0.0` 时不自行提示，风险提示静默缺失 | architecture / correctness(documentation-vs-behavior) |
| MAJOR-2 | `scripts/check-zh-docs.py:1404-1405` | `--en-ratio 0` 被 `not 0.0` 分支静默改写为 `0.05` | CI 或人工以最严阈值运行闸门时，阈值被无声放宽，en_ratio 介于 0 与 0.05 的文件由 FAIL 变 PASS | correctness / fail-closed |
| MAJOR-3 | `crates/rustcode-daemon/src/lib.rs:6337-6341` + `crates/rustcode-cli/src/main.rs:1780` | 新注释称非回环风险提示由 `WebuiLanWarning`/`WebuiNonLoopbackWarning` 承担，但 `rustcode daemon` 直连 `run_server`、不经 `ensure_server_and_open` | `rustcode daemon` 绑定 `0.0.0.0` 后用户只看到一行监听地址，零「无 TLS / token 即权限」提示；且该子命令无 `--host` 逃生开关 | security-notice / documentation-vs-behavior |
| MINOR-1 | `crates/rustcode-daemon/src/lib.rs:5275` | `ensure_server_and_open` 文档写「默认 `127.0.0.1`」，调用方已是 `0.0.0.0` | 维护者误判默认暴露面 | maintainability |
| MINOR-2 | `crates/rustcode-daemon/tests/default_host_lock.rs:3-6` | 注释称独立二进制「无 token 被拉起」，实际 `main.rs:173` 传 `webui_tokens: Some(...)` ⇒ `enforce_token=true` | 后人发现立论错误后连带删掉这条断言正确、立论错误的锁定测试 | maintainability / documentation-vs-behavior |
| MINOR-3 | `crates/rustcode-daemon/src/lib.rs:5264-5265,5442-5443` | 两处注释称 VSCode daemon `enforce_token=false`，与 `main.rs:173` 矛盾 | 安全判定被持续误导（既存问题，不在 `files_owned`） | maintainability / documentation-vs-behavior |
| MINOR-4 | `crates/rustcode-config/src/i18n/messages.rs:4899` | `Msg::DaemonWarnNonLoopback` 已无发射点却无保留说明 | 被当死码删除，或被误认为非回环告警仍在 | maintainability |
| MINOR-5 | `scripts/check-zh-docs.py:161-163,1258` | `gate` 分母仅已跟踪 md，`check --diff` 含未跟踪 | 新建未 `git add` 的 md 逃过 AC-1/2/4/6/7b/32 与 AC-8 段 1 | fail-closed / test-coverage |
| NIT-1 | `scripts/check-zh-docs.py:1031` | `cmd_hostscan` 恒 `return 0` | 被当门禁调用时假绿 | usability |
| NIT-2 | `scripts/check-zh-docs.py:177-182,1256` | 非法 `--base` 被当成「基线无此文件」而非环境错误 | 根因淹没在几百条假 FAIL 中（结果仍为红，fail-closed 未被破坏） | diagnosability |

---

## 9. 返工与下一步

- **返工轮次**：第 1 轮（本 feature 代码/脚本侧首次审查）。
- **下一跳**：`code-implementer`（修 MAJOR-1/2/3；MINOR 项可同批处理）。
- **升级条件未触发**：无 blocker/critical，且非第 3 次返工，故不 escalate；根因在「注释/阈值处理的落地遗漏」而非契约或设计本身，故不 reject 回 `solution-architect`。
- **需 PM 知会（不阻断）**：`00-decisions.md` Q2 理由 3 的「`webui_tokens=None` ⇒ `enforce_token=false`」前提与代码现状不符（MINOR-2/3 同源），建议在下一次文档维护时校正，避免后续裁决建立在错误前提上。
