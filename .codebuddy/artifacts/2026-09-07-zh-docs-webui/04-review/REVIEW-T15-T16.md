---
kind: review
id: T-15-T-16-R2
from: code-reviewer
to: [code-implementer, project-manager]
feature: 2026-09-07-zh-docs-webui
status: changes_requested
decision: rework
requires: [T-15, T-16]
files_owned: []
created: 2026-09-09
---

# T-15 + T-16 复审报告（G4，第 2 轮）

> 复审对象：`03-impl/T-15.md`（`rustcode daemon` 补 `--host`）与 `03-impl/T-16-artifacts-exempt.md`（`.codebuddy/artifacts/` 提升为门禁豁免域）。
> 本报告所有数字均为本轮**实测**（命令与输出见 §7 验证复述），未采信报告中的二手数字而不复核。

---

## 1. 元信息

| 项 | 实测值 |
|---|---|
| 分支 | `dev` |
| `git rev-parse HEAD` | `3ee655e381d28052e428196daedd456cc6079520`（与基线一致，**未提交**） |
| `git rev-parse --short HEAD` | `3ee655e3` |
| worktree 状态 | `git status --porcelain` 共 232 项；`git diff --cached --name-only` = **230**（全部已 stage）；`git diff --name-only`（工作区 vs 索引）= **0**；未跟踪 **6**（全部是 `.codebuddy/memory/` 与 `.codebuddy/teams/` 两个 agent 运行时目录） |
| `.codebuddy/memory` / `.codebuddy/teams` 是否被 stage | **0 个**（T-16 已用 `git restore --staged` 移出索引，复核通过） |
| 被审文件 1 | `crates/rustcode-cli/src/main.rs` |
| 被审文件 2 | `scripts/check-zh-docs.py` |

### 1.1 被审 diff 实测（`git diff --stat 3ee655e3 -- <file>`）

```console
$ git diff --stat 3ee655e3 -- crates/rustcode-cli/src/main.rs scripts/check-zh-docs.py
 crates/rustcode-cli/src/main.rs |  65 ++++++++++++++++++++++++++++++++++++++---
 scripts/check-zh-docs.py        | 1529 +++++++++++++++++++++++++++++++++++++++++
 2 files changed, 1590 insertions(+), 4 deletions(-)
```

口径说明（已复核，与 T-15 报告 §5 一致）：

- `main.rs` 的 61/4 中，**只有 +7/−1 属 T-15**（枚举字段 4 行 / `.mut_arg` 1 行 / 解构 1 行 / `ServerOpts{host}` +1−1），其余 +51/−3 属 T-03（`webui` 默认值 `127.0.0.1 → 0.0.0.0` 与 `mod default_host_tests`）的既有工作区改动。
- `scripts/check-zh-docs.py` 在基线 `3ee655e3` **不存在**（`git cat-file -e 3ee655e3:scripts/check-zh-docs.py` → `fatal: ... not in '3ee655e3'`，exit 128），故 1529 行全部显示为新增；T-16 的 6 处语义改动**无法用 `git diff` 隔离**，只能以「读代码 + 与冻结常量对照 + 运行时行为」核验（见 §4 复审点 8 与 §7）。**这是本轮证据边界，必须明示。**

### 1.2 T-15 四 hunk 落点（改后行号，实测 `grep -n`）

| # | 文件:行 | 内容 |
|---|---|---|
| (a) | `crates/rustcode-cli/src/main.rs:1031-1034` | `Commands::Daemon` 新增 `host: String`（2 行 doc comment + `#[arg(long, default_value = "0.0.0.0")]` + 字段） |
| (b) | `crates/rustcode-cli/src/main.rs:470` | `.mut_subcommand("daemon", …)` 内 `.mut_arg("host", \|a\| a.help(t(Msg::CliHelpHost).into_owned()))`（`webui` 的同名行为 `:478`，未改） |
| (c1) | `crates/rustcode-cli/src/main.rs:1745` | `Commands::Daemon { port, host, client, idle_timeout }` 解构新增 `host` |
| (c2) | `crates/rustcode-cli/src/main.rs:1786` | `ServerOpts { host, port, … }`，替换原 `host: "0.0.0.0".to_string()` 字面量 |

### 1.3 T-16 六处落点（改后行号）

| # | 文件:行 | 内容 |
|---|---|---|
| 1 | `scripts/check-zh-docs.py:43-64` | 新增 `ARTIFACTS_EXEMPT_PREFIX = ".codebuddy/artifacts/"`（:60）与 `is_artifacts_exempt()`（:63-64） |
| 2 | `scripts/check-zh-docs.py:110` | `AC8_EXEMPT_PREFIX = ARTIFACTS_EXEMPT_PREFIX` |
| 3 | `scripts/check-zh-docs.py:152-171` | `_git()` 增加 `-c core.quotePath=false` |
| 4 | `scripts/check-zh-docs.py:191-199` | `list_md_files()` 过滤豁免域 |
| 5 | `scripts/check-zh-docs.py:985-991` | `resolve_files()` 批量分支过滤豁免域（`args.files` 早返回 `:974-975` 未动） |
| 6 | `scripts/check-zh-docs.py:1332-1350` + `:865-867` | `cmd_gate()` 提示 + `render_inventory()` 分母注记 |

---

## 2. 裁决

| 任务 | 裁决 | 一句话理由 |
|---|---|---|
| **T-15** | **approve**（代码面） | 4 处改动与 `01-design-addendum-d5.md` §4.1–§4.4 逐条一致、三处成对无 panic 缺口、默认值 `0.0.0.0` 保住、fail-closed 由我独立实测复现；无 blocker/critical。 |
| **T-16** | **request-changes** | 豁免实现本身正确且 stage 前后自洽，但**支撑「整前缀豁免」的关键论断被实测证伪**（§4 复审点 11），且 AC-1 分母 285→248 属契约层范围变更，尚未在 `00-decisions.md` 登记。 |

**合并裁决：`request-changes` / `decision: rework`，返工轮次 R2。**
**可推进提交的判据（全部满足即可 commit）**：

1. PM 在 `.codebuddy/artifacts/2026-09-07-zh-docs-webui/00-decisions.md` 登记一条裁决：AC-1 分母 = 「全仓 md 减显式豁免域 `.codebuddy/artifacts/`」（方案 A，我推荐），或改走方案 B（解耦收窄，见 §4 复审点 11）；
2. T-16 修正 `03-impl/T-16-artifacts-exempt.md` §4.1 / §7-A 的「唯一实现路径」论断（已被证伪，不得留错误的排他性结论）；
3. `scripts/check-zh-docs.py:885` 的「全仓 md 总数 = %d」改为「受检 md 总数（已排除 artifacts 豁免域）= %d」（当前实测 349 vs 打印 248，输出自相矛盾）；
4. PM 更新 `AGENTS.md:37` 与 `STATUS.md:115`（设计 I-2 / I-3）——非 T-15 的 `files_owned`，故不扣 T-15 分，但为**提交前置条件**。

问题计数：**blocker 0 / critical 0 / major 3 / minor 3 / nit 1**。

---

## 3. 分级问题清单

| 级别 | 文件:行 | 问题 | 失败场景 | 期望行为 | 建议修复 | 归属 |
|---|---|---|---|---|---|---|
| **major** | `.codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/T-16-artifacts-exempt.md:250-264`、`:326` | 报告断言「收窄豁免前缀到本 feature 目录 ⇒ 双重破坏冻结的 AC-8；要保持分母 285 的唯一路径是先改 AC-8 契约并回退 SA」。**该论断被本轮实测证伪**：只测了「把 `AC8_EXEMPT_PREFIX` 一并改窄」的耦合变体，未测「AC-1 豁免域收窄、**AC-8 前缀保持不变**」的解耦变体 | PM 依据该排他性论断，在「牺牲 37 个历史交接件的门禁监管」与「回退 architect 改 AC-8」之间做非必要取舍；错误结论固化在交付件里会误导下一位 agent | 结论必须由覆盖全部候选方案的实测支撑；两种可行方案都要摆到 PM 面前 | ① 在 §4.1 补测解耦变体并更正 §7-A；② PM 在 `00-decisions.md` 登记最终选择。实测反证见 §7.4 | T-16 + PM |
| **major** | `scripts/check-zh-docs.py:196-199`（过滤点）、`:885`（`render_inventory` 打印「全仓 md 总数 = %d」）；契约面 `00-requirement.md:259` | 豁免整前缀后 AC-1 分母 **285 → 248**、全量受检 **231 → 194**，差值 37 全是基线已入库的**历史**交接件（`.codebuddy/artifacts/2026-09-02-*` 等），永久脱离 AC-2/4/6/7b/32 监管；同时脚本仍打印「全仓 md 总数 = 248」，而 `git ls-files -- '*.md'` 实测 **349** | 后续任何一次对历史交接件的汉化回退不再被门禁拦截；审计方读到「全仓 md 总数 248」会误判覆盖率（真实 349，其中 101 在 artifacts 域） | 契约层显式登记新分母定义；脚本输出口径与之一致 | 方案 A（推荐）：维持整前缀 + `00-decisions.md` 登记 + 改 `:885` 措辞 1 行。方案 B：解耦收窄（见 §4 复审点 11）。**不升 critical 的理由**：实测受检 231 / FAIL 0（报告 §4.1 与本轮 §7.4 均一致），豁免**未掩盖任何既有失败**；AC-8 段 2 与 AC-7a 仍覆盖该域 | PM 裁决 + T-16 改 1 行 |
| **major** | `AGENTS.md:37`；`.codebuddy/artifacts/2026-09-07-zh-docs-webui/STATUS.md:115`、`:139` | T-15 落地后，这两处仍写「**rustcode daemon 子命令没有 --host 参数**，该入口无法显式改回 127.0.0.1」，与代码事实相反（`main.rs:1034` 已有该参数） | 提交后仓库主约束文件与代码矛盾；下一位 agent 按 `AGENTS.md:37` 派单会重复造轮子或误判安全边界 | 按设计 I-2 / I-3 同步两处文字（行内引用的 `main.rs:1780` 亦需改为引用 `host` 字段） | PM 在 commit 前更新；`main.rs:1780` 的行号引用同步 | project-manager（**非 T-15 缺陷**，提交前置条件） |
| minor | `scripts/check-zh-docs.py:40` | `OWNED_ELSEWHERE` 第 3 个成员 `.codebuddy/artifacts/2026-09-07-zh-docs-webui/` 在 T-16 后成为死配置：`is_owned_elsewhere()` 仅被 `classify():258`（输入来自 `list_md_files()`）与 `cmd_hostscan():1061`（同源）调用，两者均已排除 artifacts 域 | 后续维护者以该成员为依据，认为「本 feature 交接件被计入 ZH 桶」，与 T-16 语义相反，可能反向"修复" | 死配置要么删除，要么显式标注失效 | 删除该成员，或改为注释「T-16 起对 artifacts 域无效（域内文件不进分母），保留仅为可读性」 | T-16 |
| minor | `.codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/T-16-artifacts-exempt.md:32`、`:275` | (a) 称改动 6 hunk，§1 表格却列 7 行（gate 提示与 inventory 注记被并成一项）；(b) 称不加 `-c core.quotePath=false` 会让非 ASCII 交接件「**静默**漏出豁免域」——实测并非静默 | (a) 交付件自相矛盾，审计时无法核对改动面；(b) 低估/误述影响面：`list_md_files():196-199` **没有** `isfile` 过滤，带引号路径会进入分母并在 `check_file():646-647` 抛 `EnvError` → **exit 2（响亮失败）**；只有 `resolve_files():988-991`（有 isfile 过滤）才是真静默漏检 | hunk 计数与表格一致；后果描述分路径写清 | 修订报告两处文字（不改代码） | T-16 |
| minor | `crates/rustcode-cli/src/main.rs:5474-5523`（`mod default_host_tests`） | 解析层哨兵只覆盖 `webui`（`webui_host()`），**没有 `daemon_host()`**；`crates/rustcode-daemon/tests/default_host_lock.rs` 锁的是**独立二进制**（`DEFAULT_HOST=127.0.0.1`），`crates/rustcode-config/tests/cli_webui_i18n_lock.rs:90` 只锁 `Msg::CliHelpHost` 文案 | 后续把 `Commands::Daemon::host` 的 `default_value` 改回 `127.0.0.1`，或把 `ServerOpts { host, … }`（`main.rs:1786`）改回字面量，CI 不红——而 `0.0.0.0` 是 Q2-A 冻结的安全暴露面默认值 | 与 `webui` 对称的解析层锁定 | 在 `default_host_tests` 内补 `fn daemon_host(args)` 两条断言（默认 `0.0.0.0`；`--host 127.0.0.1` 覆盖），约 12 行，在 T-15 的 `files_owned` 内 | T-15（或转 test-engineer） |
| nit | `crates/rustcode-daemon/src/lib.rs:6379-6394`（横幅/提示） vs `:6498-6509`（bind 失败 exit 1） | 非法 host 时先打印「RustCode API 服务已启动，监听地址 http://999.999.999.999:13483」与「已绑定非回环地址…」，**之后**才 bind 失败 exit 1。报告把它列为纯观察项（O-3） | 输出陈述了尚未成立的事实（"已启动"/"已绑定"），运维/脚本据此认为服务起来了 | 横幅应在 bind 成功之后打印，或改措辞为「正在启动，将监听 …」 | **判定：应修的缺陷（低优先），不阻塞本任务**：退出码 1 与 stderr `致命错误：无法绑定到 …`（实测 `/tmp/cr_bad.log:42`）均正确，无静默失败、无假成功；冻结面在 `rustcode-daemon`（契约 §4.4），须由 architect 另派单 | 另派单（architect） |

---

## 4. 复审点逐项结论（1–12）

### T-15

**1. 三处是否与契约一致且成对？——一致，且成对（无 panic 缺口）。**

- 类型 `String`、无 `value_parser`、默认值 `"0.0.0.0"`、位置在 `port` 之后 `client` 之前：`main.rs:1031-1034` 与契约 §4.1 逐字一致；`--help` 实测顺序为 `--port` → `--host` → `--client`（`daemon --help` 输出第 6–8 行）。
- `.mut_arg("host", …)` 复用 `Msg::CliHelpHost`，未新增 i18n 变体（`crates/rustcode-config/src/i18n/messages.rs:3916` 已存在）。
- 四处（含解构与 `ServerOpts`）在同一文件一次落地，`mut_arg` panic 耦合无缺口。防 panic 三条路径实测：`rustcode --help` exit 0、`rustcode completion bash` exit 0、`rustcode daemon --help` exit 0，`grep -ci panic` = 0。
- 另一匹配点 `main.rs:3574` 为 `Commands::Daemon { .. } => unreachable!(...)`，用 `..` 通配，加字段不受影响（已 `grep -n "Commands::Daemon"` 确认仅 `:1743` 与 `:3574` 两处）。

**2. 默认值 `0.0.0.0` 是否保住？是否复活 `Msg::DaemonWarnNonLoopback`？判据是否仍是 `is_loopback_bind_host`？——全部是。**

- `daemon --help` 实测：`--host <HOST>  Bind address (default: 0.0.0.0; use 127.0.0.1 for local-only) [default: 0.0.0.0]`；`--lang zh` 实测：`绑定地址（默认：0.0.0.0；改 127.0.0.1 则仅本机可访问） [default: 0.0.0.0]`。
- `grep -n "DaemonWarnNonLoopback" crates/rustcode-cli/src/main.rs` → **0 命中**，未复活。
- 判据未改：`crates/rustcode-daemon/src/lib.rs:6388` 仍为 `if !is_loopback_bind_host(&host)`（谓词定义 `:1300`），`crates/rustcode-daemon/**` 零改动。
- AC-19 残余 `127.0.0.1` 复核：`main.rs` 中仅剩 `:1031`/`:1050` 的 doc comment（契约 §4.2(a) 要求与 `webui` 逐字相同的措辞）、`:5476-5477`/`:5512-5514` 的测试与注释；默认值字面量与硬编码点均已为 `0.0.0.0` / 变量，**不违反 AC-19**。

**3. 是否引入第二生命周期所有者 / 动 `ServerOpts` / 越界改 daemon·i18n·extensions·docs？——否。**

- 仍调用同一 `rustcode_daemon::run_server`（`main.rs:1785`），无新 `tokio::spawn`、无第二 idle watchdog → 无第二运行时生命周期所有者。
- `ServerOpts` 字段与类型未动，仅把字面量换成变量（`main.rs:1786`）。
- T-15 自身 diff 仅 `main.rs`（4 hunk）。工作区相对基线确有 `crates/rustcode-config/src/i18n/{en,messages,zh_cn}.rs`、`crates/rustcode-daemon/src/lib.rs` 的改动，但属 T-05 / T-14 等既有任务（`Msg::CliHelpHost` 两语种文案已含 `0.0.0.0`，与本轮 `--help` 实测一致），**非本任务产物**，可采信其「i18n 零改动」声明。

**4. 失败语义（缺值 exit 2 / 非法值 exit 1 且无 token 残留 / 无静默回退）——证据充分，我已独立复现。**

- 缺值：`rustcode daemon --host` → **exit 2**，stderr `error: a value is required for '--host <HOST>' but none was supplied`。
- 非法值（我实测 `--host 999.999.999.999 --port 13483`，`RUSTCODE_HOME=$(mktemp -d)`）：**exit 1**，末尾 `致命错误：无法绑定到 999.999.999.999:13483：failed to lookup address information: Name or service not known`；`ls -A $RUSTCODE_HOME` 仅 `config.toml.lock` 与 `logs`，`test ! -e daemon-13483.json` → **exit 0（无 token 文件）**，fail-closed 成立。
- 静默回退路径：代码中不存在"非法则用默认"分支（新增 4 行仅为字段与 `mut_arg`，`ServerOpts.host` 原样传递），与契约 §6「禁止静默 fallback」一致。

**5. 测试覆盖缺口——确有缺口，严重度 minor（建议本任务内补）。**

`mod default_host_tests`（`main.rs:5474-5523`）只有 `webui_host()`；`crates/rustcode-daemon/tests/default_host_lock.rs` 锁的是独立二进制（`127.0.0.1`），`crates/rustcode-config/tests/cli_webui_i18n_lock.rs:90` 只锁 help 文案。即 `rustcode daemon --host` 的**默认值**与**显式覆盖**两条不变式在解析层无哨兵。考虑到 `0.0.0.0` 是安全暴露面默认值且 Q2-A 已冻结，建议按 minor 项补齐（约 12 行，在 `files_owned` 内）。不升 major 的理由：AC-D5-1…AC-D5-5 已由真实二进制冒烟覆盖，且 help 文案有 i18n 层锁定。

**6. 观察项 O-3（非法 host 时的前置横幅）——判定为「应修的缺陷（低优先）」，非纯信息噪声，但不阻塞。**

理由是它陈述了尚未成立的事实（"API 服务已启动"/"已绑定"），会误导运维与脚本；但退出码 1 与 stderr 的致命错误都在，**不构成静默失败或假成功**，且契约 §6 只规定了退出码与 stderr、§4.4 冻结 `rustcode-daemon`，故归 nit，须由 architect 另派单。已给出可定位证据：`lib.rs:6379-6394` 打印早于 `:6498-6509` 的 `TcpListener::bind`。

### T-16

**7. 豁免是否只作用于批量候选？显式 `--files` 是否仍可检？`.codebuddy/agents/*.md` 是否未被误豁免？——是 / 是 / 是。**

- `list_md_files():196-199` 与 `resolve_files():985-991`（批量分支）各过滤一次；`resolve_files():974-975` 的 `if args.files: return list(args.files)` 早返回**未改动**。
- 实测 `check --base 3ee655e3 --files .codebuddy/artifacts/2026-09-07-zh-docs-webui/STATUS.md` → `check: 受检 1，PASS 0，FAIL 1`，**未被静默跳过**（FAIL 原因是该文件对基线新增、AC-4 记全文 code span 为 added，属预期）。
- 实测 `inventory` 中 `.codebuddy/agents/` 行数 = **7**（7 个 agents md 全在清单内）；`gate` 输出中 `.codebuddy/agents/doc-writer.md | removed=README.zh-CN.md | added=- | cause=D1` 仍参与 AC-4 判分。
- 前缀判定边界：`startswith(".codebuddy/artifacts/")` 含尾斜杠，`.codebuddy/artifacts-x/` 之类不会误伤；`.codebuddy/agents/`、`memory/`、`teams/`、`rules/` 前缀不同，一律不豁免。

**8. 是否改了判据或阈值？——未改（在可核验范围内）。**

证据边界：`scripts/check-zh-docs.py` 在基线不存在，**无法用 `git diff` 隔离 T-16 的增量**，故该项以「冻结常量对照 + 代码结构」核验：

- `EN_RATIO_DEFAULT: float = 0.05`（`:25`）、`DEFAULT_BASE = "3ee655e3"`（`:24`）、`AC4_ALLOWED_ADDED_BY_FILE`（`:95`）、`AC4_MAX_PER_ENTRY = 1`（`:96`）、`AC8_NEEDLE`（`:108`）、`AC8_GLOBS`（`:109`）、`AC8_ANCHOR_FILES`（`:111`）、`AC8_FALLBACK_SCOPES`（`:112`）、`SKIP_B_RULES`（`:29`）——全部与 `01-design.md:189` / `00-requirement.md:260` / `STATUS.md:130` 记载的冻结值一致。
- 新增代码只有「过滤」（`if not is_artifacts_exempt(...)`）与「提示打印」，未引入任何新的 PASS/FAIL 分支、未新增命令行开关、未新增返回码（`0/1/2` 语义不变）。
- **残留不可核验项**：无法从版本控制证明 AC-2/4/6/7b/32 的**算法体**在 T-16 中未被顺手改动；只能由「冻结常量未变 + 受检数 231 与报告 §4.1 实测一致 + 全量 FAIL 0」间接佐证。若 PM 要求强证据，建议在下一轮把该脚本提交一次（使其成为受版本控制的文件）后再改。

**9. `AC8_EXEMPT_PREFIX` 改为引用新常量后，AC-8 三段是否逐字不变？——是（构造性成立，并已运行时复核）。**

`:110` `AC8_EXEMPT_PREFIX = ARTIFACTS_EXEMPT_PREFIX`，而 `ARTIFACTS_EXEMPT_PREFIX = ".codebuddy/artifacts/"`（`:60`）与改动前的字面量**逐字相同**；AC-8 的候选划分（`ac8_candidates():1201-1211`）与三段判定（`gate_check_ac8():1236-1321`）只读 `AC8_EXEMPT_PREFIX`，不读 `is_artifacts_exempt()`。实测当前三段输出：段 1 候选 515 / 0 命中 → OK；段 2 候选 101 / 命中 264 全在域内 → OK；段 3 兜底域 0 命中 → OK。

**10. `_git()` 增加 `-c core.quotePath=false`：必要且安全。**

- 必要性：实测 `git ls-files -- '*.md' | grep -c '^"'` = **1**（即 `.codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/HANDOFF-汉化铁律.md` 的八进制转义带引号形式）。该串既不匹配任何路径前缀、`os.path.isfile()` 也判否 → 不加该选项会在 `git add` 后出问题。加后 `git -c core.quotePath=false ls-files | grep -c '^\.codebuddy/artifacts/'` = **101**（= 100 ASCII + 1 非 ASCII），与报告一致。
- 安全性：该选项只影响 `_git()` 包装的 `ls-files` / `grep` / `diff --name-only` / `rev-parse`；对这四者，输出**去引号**只让路径更接近真实磁盘路径，对前缀匹配与 `os.path.isfile()` 判优、无副作用。`rev-parse --show-toplevel` 为 ASCII，不受影响。
- 唯一理论副作用：含换行/控制字符的路径不再被转义，行切分可能出错。实测 `git -c core.quotePath=false ls-files | grep -c -P '[\x00-\x1f]'` = **0**，仓库无非 ASCII 之外的异常路径（非 ASCII 路径仅 1 个）。结论：安全。

**11.（重点裁决项）分母 285 → 248：是否构成真实风险？与 AC 定义是否冲突？报告论断是否成立？**

**(a) 是否构成「门禁范围收窄、历史文件脱离监管」的真实风险？——是真实风险，但当前影响为零。**

- 真实风险成立：`git ls-files -- '*.md'` 实测 **349**，其中 artifacts 域 **101**；豁免后受检 **194**（= 248 − SKIP 54）。差值 37 个**基线已入库**的历史交接件（`.codebuddy/artifacts/2026-09-02-*` 等）永久脱离 AC-2 / AC-4 / AC-6 / AC-7b / AC-32。
- 当前影响为零：报告 §4.1 与本轮复算均显示「收窄变体」下 `全量 check 受检 231，FAIL 0` —— 37 个历史件**全部通过**现有判据；且其中 TODO(EN/MIXED) 由 27 变 29，说明域内有 2 个非纯中文件，但它们同样 FAIL 0。即**豁免未掩盖任何既有失败**。
- 残留覆盖：AC-8 段 2 仍以它们为「历史域」做反向断言（必须仍有 `README.zh-CN` 命中）、AC-7a 仍对它们判改名/删除（`gate_check_ac7a()` 未加豁免）。

**(b) 与 `00-requirement.md` 的 AC-1 / AC-2 定义是否冲突？——与字面定义冲突，需契约层追认。**

`00-requirement.md:259` 定义 AC-1 恒等式为「跳过数 + 待汉化数 + 已中文数 = **全仓 md 总数**」。脚本 `render_inventory():885` 至今仍打印「全仓 md 总数 = 248」，而真实全仓 tracked md = 349（域内 101）。`01-design.md:60` 确有「AC-1 一律以脚本产出计数为准，不硬编码」的表述，但该句解决的是**数字来源**（不要手写 290/287），**并未授权改变分母的集合定义**。因此：这是契约层的范围变更，必须由 PM 显式登记，不能只由实现报告的 §7-A 悬空。

**(c) 报告「收窄到本 feature 目录会双重破坏冻结的 AC-8」是否成立？——对耦合变体成立；作为「唯一实现路径」的证明不成立（已被实测证伪）。**

报告只测了把 `AC8_EXEMPT_PREFIX` 一并改窄的**耦合变体**。我实测了**解耦变体**（`ARTIFACTS_EXEMPT_PREFIX` 收窄到 `.codebuddy/artifacts/2026-09-07-zh-docs-webui/`，`AC8_EXEMPT_PREFIX` 保持 `.codebuddy/artifacts/`），结果（见 §7.4）：

```
NARROW total= 285 受检= 231
AC8 ok= True
段 1 正式域（候选 515，已排除 .codebuddy/artifacts/）：0 命中 -> OK
段 2 历史域（候选 101）：命中 264 处，全部位于 .codebuddy/artifacts/ -> OK
段 3 兜底域 site/.github/docs/extensions（全扩展名）：0 命中 -> OK
```

即：**分母回到 285、受检回到 231、AC-8 三段逐字不变且判定仍 OK**。PM 关心的问题有解，且不需要回退 architect 改 AC-8。

**(d) 我的建议（明确结论）：**

- **推荐方案 A：维持整前缀豁免 + 补契约登记。** 理由：① 与 `SKIP_A..SKIP_D` 同源（按路径判定、与基线无关），对未来所有 feature 一致，不必每 feature 改常量；② 域内文件当前全部 PASS，不存在掩盖；③ 收窄到单 feature 会让「下一个 feature 的交接件」再次踩同一个假 FAIL 坑。
- **但方案 A 必须补两条**：`00-decisions.md` 写明「AC-1 分母 = 全仓 md 减显式豁免域 `.codebuddy/artifacts/`；域内文件不进分母、不受 AC-2/4/6/7b/32 与 AC-8 段 1 约束，仍为 AC-8 段 2 历史域、仍受 AC-7a 约束」；并把 `render_inventory():885` 的「全仓 md 总数」改为「受检 md 总数（已排除 artifacts 豁免域）」。
- **若 PM 坚持分母必须是 285**，正确路径是**方案 B（解耦收窄）**：`ARTIFACTS_EXEMPT_PREFIX` 改为本 feature 目录，`AC8_EXEMPT_PREFIX` 恢复为独立字面量 `.codebuddy/artifacts/`（两常量脱钩）——而非报告所称的「必须先改 AC-8 契约」。代价：两个常量作用域不同需注释说明，且未来 feature 需追加豁免前缀。

**12. 提交（stage）前后一致性证据是否可信？——可信（后态我独立复现，前态机制上成立）。**

- 后态复现：`gate --base 3ee655e3` → **exit 0 / PASS**，`SKIP_A=4 SKIP_B=47 SKIP_C=1 SKIP_D=2 TODO=27 ZH=167 total=248`，`全量 check 受检 194，FAIL 0`，`AC-4 授权放行合计 12 条（D1 11 / D2 1）`，与报告 §3.2 逐项一致。
- 前态一致性成立的两条硬证据：① 豁免为**路径前缀**判定，与是否已跟踪、与基线无关；② `git diff --cached --diff-filter=A --name-only -- '*.md' | grep -vc "codebuddy/artifacts"` = **0**，即本次新增的 md **全部**落在豁免域内，stage 不会让任何新文件进入分母；删除的 md 只有 `README.zh-CN.md`（1 个，且其删除在 T-16 之前已 stage，由 248 = 285 − 37 与 248 = 349 − 101 双向自洽可证）。
- 唯一不逐字相同的输出：`gate` 提示行的「artifacts 豁免域 … 已排除：**37** 个」在 stage 前 vs「**101** 个」在 stage 后——报告已如实披露，且退出码、恒等式、受检/FAIL 数确实完全相同。**判据 12 通过。**

---

## 5. 架构边界核对（对照 `AGENTS.md`）

| 约束 | 结论 | 证据 |
|---|---|---|
| 依赖只向下，不新增 crate 依赖 | 通过 | T-15 新增 4 行无新 `use`；T-16 为 stdlib-only 脚本 |
| `kernel` / `capabilities` / `coding` 生产依赖 core-free；capabilities 不反向依赖 | 不涉及 | 两任务均未触碰这些 crate |
| 目标调用链 `CLI → CodingRuntime → kernel Agent`；无第二生命周期 owner | 通过 | `main.rs:1785` 仍调用同一 `rustcode_daemon::run_server`，无新 spawn / watchdog |
| 禁 bridge / v1-v2 开关 / core driver fallback / core session 磁盘模型 / 双向持久化 | 通过 | 无开关；非法 host 一律 exit 1，无静默回退；无持久化读写 |
| `AGENTS.md:37`（Q2-A 两个入口默认 `0.0.0.0`） | 语义保住，但**文字已失实** | `main.rs:1033` `default_value = "0.0.0.0"`；`AGENTS.md:37` 仍称"daemon 子命令没有 --host 参数" → 见 major 第 3 条 |
| `AGENTS.md:38`（独立二进制与 TUI `/webui` 保持 `127.0.0.1`） | 通过 | `crates/rustcode-daemon/src/main.rs:21`、`commands.rs:2207` 未动 |
| `AGENTS.md:39`（非回环提示判据 = `is_loopback_bind_host`；`Msg::DaemonWarnNonLoopback` 保留但不复活） | 通过 | `lib.rs:6388` 未动；`main.rs` 中 `DaemonWarnNonLoopback` 0 命中 |
| turn completion / compaction 不新增重叠 hook 或第二压缩状态机 | 不涉及 | — |
| 单一状态所有者 | 通过 | bind 地址所有者仍是 driver（`ServerOpts.host` 由调用方填），所有者数量不变 |
| 命令注入 / 路径穿越 / 敏感信息 | 通过 | T-16 的 `_git()` 用 `subprocess.run(list)` 无 shell；`-c core.quotePath=false` 为固定字面量；豁免为常量前缀非用户输入 |

---

## 6. 契约符合性

- **T-15**：`01-design-addendum-d5.md` §4.1–§4.4 逐条符合（形式 / arg id / 类型 / clap 属性 / 默认值 / 字段位置 / 与 `webui` 一致 / doc comment 逐字 / 复用 `Msg::CliHelpHost` / 解构与传参 / 两语种文案 / 冻结不变项），**零未声明偏差**；§6 失败语义全部实测通过。**结论：符合。**
- **T-16**：`01-design.md` / `01-design-addendum.md` 的 AC-2 v2、AC-4 v2、AC-6、AC-7a/b、AC-8 v2、AC-32 判据与常量一字未改（§4 复审点 8）；`AC8_EXEMPT_PREFIX` 取值逐字不变（复审点 9）。**唯一契约偏差是 AC-1 的集合定义被收窄**（285 → 248），已由报告 §2 声明并报 PM 裁决，但**尚未落到 `00-decisions.md`**。**结论：实现符合，契约登记缺位。**

---

## 7. 测试覆盖评估

| 路径 | 覆盖 | 评价 |
|---|---|---|
| `rustcode daemon --host` 解析（默认 / 显式覆盖） | **无解析层测试** | minor 缺口，见问题表 m6；AC-D5-1…D5-5 由真实二进制冒烟覆盖 |
| `rustcode webui --host` 解析 | 有（`main.rs:5474-5523`，实测 2 passed） | 充分 |
| help 文案两语种 | 有（`crates/rustcode-config/tests/cli_webui_i18n_lock.rs:90`） | 覆盖 `Msg::CliHelpHost`，daemon 复用同一变体故间接受益 |
| 独立二进制 `DEFAULT_HOST` | 有（`crates/rustcode-daemon/tests/default_host_lock.rs`） | 与本任务正交 |
| 缺值 exit 2 / 非法值 exit 1 / 无 token 残留 | 报告有冒烟；**我独立复现**（exit 2 / exit 1 / 无 `daemon-13483.json`） | 充分；但无自动化回归测试（契约未要求） |
| T-16 豁免行为 | **无自动化测试**（脚本无测试套件，既有约定） | 依赖运行时复核；本轮已复核「显式 `--files` 仍可检」「agents 未误豁免」「前缀边界」三项 |
| 失败与取消路径 | `gate --base deadbeefdeadbeef` → exit 2（NIT-2 未回退）已由报告实测 | 通过 |

---

## 8. 验证复述（本轮实际执行的只读命令与输出）

以下全部为只读命令；除本报告文件外**未修改任何文件**（`git status` 前后一致：230 staged / 0 unstaged / 6 untracked）。

1. `git rev-parse HEAD` → `3ee655e381d28052e428196daedd456cc6079520`；`git status --porcelain | wc -l` → 232；`git diff --cached --name-only | wc -l` → 230；`git diff --name-only | wc -l` → 0；`git ls-files --others --exclude-standard | wc -l` → 6（全为 `.codebuddy/memory/`、` .codebuddy/teams/`）。
2. `git cat-file -e 3ee655e3:scripts/check-zh-docs.py` → exit 128（`not in '3ee655e3'`），确认脚本为基线新增、T-16 增量无法用 diff 隔离。
3. `git diff --stat 3ee655e3 -- crates/rustcode-cli/src/main.rs scripts/check-zh-docs.py` → `65 ++++` / `1529 ++++`，`2 files changed, 1590 insertions(+), 4 deletions(-)`。
4. `./target/debug/rustcode daemon --help`（二进制 05:33 晚于 `main.rs` 05:28，为改后构建）→ 含 `--host <HOST> … [default: 0.0.0.0]`，exit 0；`--lang zh` → 含 `绑定地址（默认：0.0.0.0；改 127.0.0.1 则仅本机可访问）`。
5. `./target/debug/rustcode daemon --host` → **exit 2** + `a value is required for '--host <HOST>' but none was supplied`。
6. `./target/debug/rustcode --help` / `completion bash` → exit 0 / exit 0，`grep -ci panic` = 0，`grep -c -- "--host"` = 4（防 panic 三路径）。
7. fail-closed：`RUSTCODE_HOME=$(mktemp -d) timeout 8 ./target/debug/rustcode --lang zh daemon --host 999.999.999.999 --port 13483` → **exit 1**；日志含 `监听地址 http://999.999.999.999:13483`（第 4 行）、`已绑定非回环地址`（第 6 行）、`致命错误：无法绑定到 …`（第 42 行）；`ls -A $RUSTCODE_HOME` 仅 `config.toml.lock`、`logs`；`test ! -e daemon-13483.json` → exit 0。
8. `cargo fmt --check` → **exit 0**（无写入）。
9. `cargo test -p rustcode --bin rustcode default_host_tests` → `2 passed; 0 failed`。
10. `python3 scripts/check-zh-docs.py gate --base 3ee655e3` → **exit 0 / PASS**，`total=248`、`受检 194，FAIL 0`、AC-4 放行 12 条、AC-8 三段全 OK。
11. `git ls-files -- '*.md' | wc -l` → 349；`grep -c '^\.codebuddy/artifacts/'` → 100；`git -c core.quotePath=false … | grep -c` → **101**；`git ls-files -- '*.md' | grep -c '^"'` → **1**（quotePath 必要性）。
12. 解耦收窄变体（**内存中** monkeypatch，未写任何文件）：`NARROW total= 285 受检= 231`、`AC8 ok= True`、三段输出逐字不变（见 §4 复审点 11c）。
13. `python3 scripts/check-zh-docs.py check --base 3ee655e3 --files .codebuddy/artifacts/.../STATUS.md` → `受检 1，PASS 0，FAIL 1`（显式 `--files` 未被豁免）。
14. `git diff --cached --diff-filter=A --name-only -- '*.md' | grep -vc "codebuddy/artifacts"` → **0**；`--diff-filter=D` → 仅 `README.zh-CN.md`（stage 前后一致性硬证据）。
15. `grep -n` 复核冻结常量：`EN_RATIO_DEFAULT=0.05`（:25）、`AC4_MAX_PER_ENTRY=1`（:96）、`AC8_*`（:108-112）等均未变。
16. 副作用披露：我曾执行 `python3 -m py_compile scripts/check-zh-docs.py`（exit 0），它在 `scripts/__pycache__/` 生成了字节码缓存；该目录被 `.gitignore:23` 忽略（`git check-ignore -v` 命中），已随后删除该缓存文件与空目录，`ls scripts/ | grep -c pycache` → 0，git 状态未受污染。

---

## 9. 回报编排者

- **T-15：approve**（代码面无 blocker/critical；3 项待办归 PM / test-engineer / architect，其中 `AGENTS.md:37` 更新为**提交前置条件**）。
- **T-16：request-changes**（major 2 项：报告排他性论断被证伪需更正 + AC-1 分母收窄缺契约登记；minor 2 项）。
- **合并裁决：`request-changes` / `decision: rework`**；问题计数 **blocker 0 / critical 0 / major 3 / minor 3 / nit 1**。
- **对第 11 项（分母收窄）的明确建议**：维持整前缀豁免（方案 A），但必须由 PM 在 `00-decisions.md` 登记新分母定义，并把 `render_inventory():885` 的「全仓 md 总数」改为「受检 md 总数（已排除 artifacts 豁免域）」；若 PM 坚持 285，则走**解耦收窄**（方案 B，我已实测可行：285 / 231 / AC-8 逐字不变），而不是改 AC-8 契约。
