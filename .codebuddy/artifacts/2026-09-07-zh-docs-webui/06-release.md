---
kind: release
id: RELEASE-001
from: doc-writer
to: [project-manager]
feature: 2026-09-07-zh-docs-webui
status: done
decision: proceed
requires: [TEST-001]
files_owned:
  - .codebuddy/artifacts/2026-09-07-zh-docs-webui/06-release.md
  - .codebuddy/artifacts/2026-09-07-zh-docs-webui/00-decisions.md
  - .codebuddy/artifacts/2026-09-07-zh-docs-webui/STATUS.md
  - AGENTS.md
created: 2026-09-09
updated: 2026-09-09（契约登记轮：补 T-15 / T-16 与 Q6）
---

# G6 交付说明 · 全仓 md 汉化 + WebUI 开箱可访问 + 默认绑定 0.0.0.0

- 基线 `ZH_BASE`：`3ee655e381d28052e428196daedd456cc6079520`，分支 `dev`
- `git rev-parse HEAD` = `3ee655e381d28052e428196daedd456cc6079520`（**HEAD 仍等于基线**，全部改动在未提交工作区）
- 依赖验收件：`05-test-report.md`（G5，`id: TEST-001`，结论 `pass`）
- 本说明的验证结论**全部引用** `05-test-report.md` 与本轮实测，**不重写、不推断**；未验证项一律保留为未验证

---

## 1. 行为变化

### 1.1 文档：全仓 markdown 汉化 + `README.zh-CN.md` 删除

| 项 | 变化 |
|---|---|
| 汉化范围 | **受检 md 248 个**（`inventory --base 3ee655e3`：`SKIP_A=4 SKIP_B=47 SKIP_C=1 SKIP_D=2 TODO=27 ZH=167`，恒等式 OK；**已按 Q6 排除 artifacts 豁免域**，旧口径为全仓 285 = `TODO=29 + ZH=202`）。其中 `TODO` 桶 27 个全为 MIXED（0 < ratio < 0.5），EN 桶为 **0**。注：`git ls-files -- '*.md'` 的真实全仓数仍是 **349**，差 101 即豁免域 |
| 实际改动 | **157 个 md**，`+8684 / -9840`（`git diff --shortstat 3ee655e3 -- '*.md'`） |
| 根 `README.md` | 英文 → 中文，`en/total` 由基线 `388/409 = 0.9487` 变为 `0/374 = 0.0000`（实测命令见 `03-impl/zh-check-D-35.md` §3） |
| `README.zh-CN.md` | **删除**（`-830` 行）。`git ls-files README.zh-CN.md` 输出为空、`test ! -e README.zh-CN.md` 为真 |
| 语言导航 | `README.md` 顶部的「English · 简体中文」切换导航**已移除**；`README.md` 内 `README.zh-CN` 字面零命中（基线导航位于 `README.md:16`） |
| 不汉化的例外 | 4 类跳过桶（license/credits 类 4、`crates/rustcode-review/rules/*.md` 47、JetBrains `CHANGELOG.md` 1、评测 prompt 载荷 2），理由见 `inventory` 输出 |

**迁移步骤（调用方 / 使用者）**：

1. 指向 `README.zh-CN.md` 的链接改为指向 `README.md`（本仓内已无残留引用；仓库外引用需各自更新）。
2. `crates/rustcode-review/rules/*.md` 与 `evals/deepseek-v4-flash/prompts/codex-{judge,report}.md` **仍是英文**，这是刻意的运行时载荷豁免，不要补译。
3. setup-seeds（`crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/` 下 6 个 md）已汉化：`description` 值由英译中，**`name:` 键未动**（`git diff -U0 3ee655e3 -- <setup-seeds> | grep -E '^[+-]name:'` 无输出）。

### 1.2 新增 `scripts/build-webui.sh` 与 `cargo clean -p rustcode-daemon` 约定

| 项 | 变化 |
|---|---|
| 新增文件 | `scripts/build-webui.sh`（未跟踪新文件，可执行）。依次执行 `npm ci` + `vite build`，产物输出 `webui/dist/` |
| 失败语义 | **fail-closed**：无 Node.js → exit 2 并提示 `Node.js >= 22.6`；Node 版本不足 → exit 2 并显示「当前 = … / 要求 >= 22.6」，**不降级为警告** |
| 幂等选项 | `--if-missing`：`webui/dist` 已存在则跳过构建 |
| 前端重编后的强制约定 | `cargo build` 不触发 npm、也不跟踪 `webui/dist/` 变化；**重新构建前端后必须执行 `cargo clean -p rustcode-daemon`** 才能把新 bundle 嵌入二进制。该约定已写入 `README.md:204`、`README.md:217-218` 与 `AGENTS.md:16` |
| 不构建时的行为 | 缺少 `webui/dist` 也能编译通过，但所有 webui 页面返回 `webui not built` |

**迁移步骤**：本地/CI 若在 `cargo build` 之前手工改过前端，需在构建流程中插入 `./scripts/build-webui.sh`；发布脚本已在 `cargo build` 前自动调用该脚本，无需手工改。

### 1.3 默认绑定由 `127.0.0.1` 改为 `0.0.0.0`（**暴露面扩大，最重要的行为变化**）

| 入口 | 变化前 | 变化后 | 源码位置 |
|---|---|---|---|
| `rustcode webui`（CLI 子命令） | `127.0.0.1` | **`0.0.0.0`** | `crates/rustcode-cli/src/main.rs:1047` `#[arg(long, default_value = "0.0.0.0")]` |
| `rustcode daemon`（CLI 子命令） | `127.0.0.1` | **`0.0.0.0`** | `crates/rustcode-cli/src/main.rs:1031-1034` `#[arg(long, default_value = "0.0.0.0")] host: String`（T-15 起该入口可用 `--host` 覆盖；`:1786` 以变量 `host` 传入 `ServerOpts`） |
| 独立二进制 `rustcode-daemon` | `127.0.0.1` | **不变，仍 `127.0.0.1`** | `crates/rustcode-daemon/src/main.rs:21` `const DEFAULT_HOST: &str = "127.0.0.1"` |
| TUI 内 `/webui` | `127.0.0.1` | **不变，仍 `127.0.0.1`** | 见 `README.md:121`；跨设备需显式 `--host 0.0.0.0`（等价写法 `lan`） |

- 鉴权前提（**未变**）：两条 `0.0.0.0` 入口均为 token 保护。`crates/rustcode-daemon/src/main.rs:177` `webui_tokens: Some(token_store)` ⇒ `enforce_token = true`；无 token 的 `POST /chat` 实测返回 401（AC-28）。
- **迁移步骤**：不需要跨设备访问的用户，显式加 `--host 127.0.0.1` 即恢复旧行为。通过 CLI 拉起 daemon 的 IDE / 脚本若只想本机访问，也应显式传 `--host 127.0.0.1`。

### 1.4 启动横幅：`DaemonWarnNonLoopback` 删除，改由 `WebuiLanWarning` / `WebuiNonLoopbackWarning` 承担

| 项 | 变化 |
|---|---|
| 删除 | 旧的 `if host != "127.0.0.1" && ... { eprintln!(Msg::DaemonWarnNonLoopback) }` 启动横幅（原 `lib.rs:6332-6350` 区域） |
| 新增 | `run_server` 内补发：`crates/rustcode-daemon/src/lib.rs:6388` 起 `if !is_loopback_bind_host(&host) { host == "0.0.0.0" || host == "::" → Msg::WebuiLanWarning; else → Msg::WebuiNonLoopbackWarning }` |
| 变体保留 | `Msg::DaemonWarnNonLoopback` 变体与两语种文案**按契约 K4 保留**（`messages.rs:4900-4903` 已加注释说明「勿当死码清理」），但已无发射点 |
| 静默性 | 回环绑定保持**静默**；非回环绑定在任何 driver 下都有用户可见提示 |
| 对用户的可感变化 | `rustcode daemon` 默认启动（`0.0.0.0`）现在会打印**局域网风险提示**，而不是旧的安全告警横幅；`rustcode daemon --host 127.0.0.1` 与独立二进制保持静默 |

**删除/退役接口**：`Msg::DaemonWarnNonLoopback` 已从「发射点」退役——**替代方案**为 `Msg::WebuiLanWarning`（`0.0.0.0` / `::`）与 `Msg::WebuiNonLoopbackWarning`（其它非回环地址）。变体本身保留但不得新增发射点，也不得当作死码删除。

### 1.5 T-14：IPv6 回环判定在提示路径收敛

| 项 | 变化 |
|---|---|
| 新增 | 私有谓词 `is_loopback_bind_host`（`crates/rustcode-daemon/src/lib.rs:1281-1303`），仅供 `run_server` 的提示分支使用，**不参与任何鉴权决策** |
| 行为 | `--host ::1`、`--host ::ffff:127.0.0.1` **不再误报**非回环提示；`0.0.0.0` / `::` / `192.168.1.7` 仍按预期告警 |
| 零改动 | `is_loopback_authority` 与 `client_interactive_permission` **零改动**（T-14 §5 有逐函数抽取 + md5 证明），权限面未变 |
| 未收敛 | `is_loopback_authority` 本体缺陷（`is_loopback_authority("::1") == false`）**未修**，交 architect 裁决（见 §4） |

### 1.6 `scripts/check-zh-docs.py`：`--en-ratio 0` 与 `--base` 校验收紧

| 项 | 变化 |
|---|---|
| 新增文件 | `scripts/check-zh-docs.py`（未跟踪新文件）。子命令 `inventory` / `check` / `hostscan` / `gate` |
| `--en-ratio` | 默认 `0.05`；显式传 `0` **不再被静默放宽**为默认值（返工项 MAJOR-2）——`0` 表示不允许任何纯英文行，会按字面生效 |
| `--base` | 非法 base 时 `gate` 返回 **exit 2**（环境错误），而不是大量假 FAIL + exit 1（返工项 NIT-2） |
| 未跟踪 md | `gate` 明确排除未跟踪 md 并在输出首行打印数量（实测 58 个，均为 `.codebuddy/artifacts/**` 等交接件），口径与 `check --diff` 的差已被显式登记（返工项 MINOR-5） |
| `hostscan` | 恒返回 0，是**清单产出器、非门禁**（返工项 NIT-1） |

**迁移步骤**：CI 若原先依赖 `check-zh-docs.py gate` 的退出码，需注意非法 `--base` 现在是 2 而非 1；依赖阈值被静默放宽的调用点需显式传参。

### 1.7 T-15：`rustcode daemon` 新增 `--host` 参数（缺陷 D-5 修复）

| 项 | 变化 |
|---|---|
| 新增 | `rustcode daemon --host <HOST>`（`crates/rustcode-cli/src/main.rs:1031-1034`）：`#[arg(long, default_value = "0.0.0.0")] host: String`，**无** `value_parser`；位置在 `--port` 之后、`--client` 之前 |
| 默认值 | **仍 `0.0.0.0`**（Q2-A 冻结值未回退）；`main.rs:1786` 由原硬编码字面量改为变量 `host` 传入 `ServerOpts` |
| help 文案 | 复用既有 `Msg::CliHelpHost`，**不新增 i18n 变体**：en `Bind address (default: 0.0.0.0; use 127.0.0.1 for local-only)` / zh `绑定地址（默认：0.0.0.0；改 127.0.0.1 则仅本机可访问）`；与 `rustcode webui --host` 两语种**逐字一致**（diff 零差异） |
| 失败语义 | 缺值 exit **2**（`a value is required for '--host <HOST>' but none was supplied`）；非法值（空串 / `1.2.3.4:80` / `999.999.999.999`）exit **1** 且**无** `daemon-<port>.json` token 文件残留；代码中不存在「非法则用默认」的静默 fallback |
| 未变 | `crates/rustcode-daemon/**`、`crates/rustcode-config/src/i18n/**` 零改动；风险提示判据仍是 `is_loopback_bind_host`（`lib.rs:1300`）；未复活 `Msg::DaemonWarnNonLoopback`；独立二进制 `DEFAULT_HOST` 仍 `127.0.0.1` |

**迁移步骤**：不需要跨设备访问的调用方（含以 CLI 拉起 daemon 的 IDE / 脚本）现在可在 `rustcode daemon` 上直接显式传 `--host 127.0.0.1` 退回仅本机。T-15 之前该入口**没有**该参数，只能改用 `rustcode webui --host 127.0.0.1` 或独立二进制；`06-release.md` 早前版本 §1.3 的「迁移步骤」因此对本入口不可行，现已被本条取代。

### 1.8 T-16：`.codebuddy/artifacts/` 成为门禁豁免域（支撑「全部提交，含 artifacts」）

| 项 | 变化 |
|---|---|
| 新增常量 | `ARTIFACTS_EXEMPT_PREFIX = ".codebuddy/artifacts/"` 与 `is_artifacts_exempt()`（`scripts/check-zh-docs.py:60`、`:63-64`）；`AC8_EXEMPT_PREFIX` 改为引用它，取值**逐字不变** |
| 域内约束 | 域内 md（当前 **101** 个）不进 AC-1 分母，不受 AC-2 / AC-4 / AC-6 / AC-7b / AC-32 与 AC-8 段 1 约束；**仍受 AC-7a 约束**（改名/删除判定未加豁免），**仍是 AC-8 段 2 的历史域**（段 2 反向断言依赖它） |
| 分母口径 | AC-1 分母由「全仓已跟踪 md 总数」改为「**受检 md 总数（已排除 artifacts 豁免域）**」：**285 → 248**、全量受检 **231 → 194**；差值 37 = 基线已入库的历史交接件（`2026-09-02-*` / `2026-09-03-*` 共 7 个历史 feature 目录）。契约层已由 `00-decisions.md` **Q6** 追认 |
| 对用户/调用方可感变化 | **无运行时行为变化**，仅门禁脚本的受检范围与输出数字变化；`check --files <path>` 对域内文件仍可单点检（`resolve_files()` 的 `args.files` 早返回未动） |
| 提交语义 | 使「本 feature 改动全部提交，含 artifacts 交接件」可行：`git add -A` 后 `gate --base 3ee655e3` 仍 **exit 0 / PASS**；若撤销豁免再提交，实测 `受检 295 / FAIL 64 / exit 1`（见 §6.5） |

**迁移步骤**：CI / 审计脚本若硬编码了 `inventory` 的 `total=285` 或 gate 的「受检 231 / 未跟踪 58」等数字，需改为以脚本产出为准（或只断言恒等式成立）。注意 `git ls-files -- '*.md'` 的真实全仓数仍是 **349**，受检 **248**，二者差 101 即豁免域。

---

## 2. 风险

### 2.1 默认绑定面扩大（**最高风险**）

- **风险**：`rustcode webui` / `rustcode daemon` 两个 CLI 入口默认从「仅本机」变为「所有网卡」。该服务**无 TLS**，仅凭一次性 token 保护；同一局域网内任何能路由到该主机的设备都可尝试访问，token 泄露即等于会话与工具执行权限泄露。
- **影响面**：所有通过 CLI 启动 daemon 的用户；以及 IDE 侧——JetBrains 插件在**非 Windows 且无打包 daemon** 时回退到 CLI `rustcode daemon`（`RustCodeDaemonProcess.kt:61-63`），该入口默认已改为 `0.0.0.0`，故 IDE 用户暴露面同步扩大。
- **未变的部分（缓解事实）**：独立二进制 `rustcode-daemon` 与 TUI `/webui` **仍默认 `127.0.0.1`**；两条 `0.0.0.0` 入口的 token 鉴权与交互审批均实测生效（AC-28、AC-27）。
- **缓解建议（用户侧）**：
  1. 不需要跨设备访问时显式 `--host 127.0.0.1`；
  2. 需要跨设备访问时置于反向代理 / 隧道之后并启用 TLS，不要直接把 `0.0.0.0` 暴露到不可信网络；
  3. 只使用独立二进制 `rustcode-daemon`（默认回环）作为常驻后端；
  4. IDE 用户建议由插件显式传 `--host 127.0.0.1`（详见 STATUS.md「待用户裁决」段，本 feature 未改插件）。

### 2.2 未验证项带来的残余不确定性

AC-22（跨设备）、AC-23（同源 CORS）、AC-25（无阻断遮罩）、AC-26 的 UI 交互层**未验证或仅部分验证**，详见 §4。这意味着「默认绑 `0.0.0.0` 后跨设备真的可用、且页面真的无 CORS 报错」尚未被实证，只验证了本机以 LAN IP 自测的部分（AC-22）。

### 2.3 G5 登记的缺陷（均不阻塞，随本说明交付）

| 编号 | 内容 | 处置 |
|---|---|---|
| **D-1** | `is_loopback_authority("::1") == false`，裸 `::1` 与 `::ffff:127.0.0.1` 判为非回环 | T-14 已在**提示路径**局部收敛（`is_loopback_bind_host`）；本体交 architect（涉及 `client_interactive_permission` 权限面，属安全决策） |
| **D-2** | AC-8 字面「全仓 `README.zh-CN` 0 命中」不成立：实为 39 处命中，**全部**位于 `.codebuddy/artifacts/`（历史交接件），正式域 0 命中 | 按三域口径判 PASS；本说明 §3 已注明口径 |
| **D-3** | `check-zh-docs.py` 的 AC-2 围栏剥离对**未闭合/嵌套围栏**失效（over-count） | 基线存量、fail-safe 方向（多算不会掩盖真实残留）；归 T-01。实例见 `03-impl/zh-check-D-34a.md` §5.1 |
| **D-4** | `Msg::WebuiLanWarning` 文案在 `run_server` 路径下称「主地址为局域网 IP」，但实际只打印 `0.0.0.0` | 文案范围断言与指代不精确，P1/P2/P3 待 architect 裁决 |
| **D-5** | `rustcode daemon` 子命令**无 `--host` 参数**（G5 登记） —— **已由 T-15 修复** | 已闭环：现支持 `--host`（默认仍 `0.0.0.0`，可显式传 `127.0.0.1`），见 §1.7 与 §3；残留风险转 T-15 §8 O-4（扩展侧若要传 `--host` 需版本门控） |

### 2.4 T-14 登记的遗留项

| 编号 | 内容 | 方向 |
|---|---|---|
| **RI-1** | `ensure_server_and_open`（`lib.rs:5393` 的 `open_host` 选择、`lib.rs:5422` 的非回环提示）仍用旧谓词 | 基线既有。后果：`rustcode webui --host ::1` 仍会（a）误报非回环提示，（b）生成 `http://::1:PORT/?token=...` 这种**缺少方括号的非法 URL**。本轮未加重也未修复，与 D-1 合并评估 |
| **RI-3** | 新谓词是**白名单式**而非解析式 | 只覆盖 `::1` 与 `::ffff:127.0.0.1`；`0:0:0:0:0:0:0:1`、`127.0.0.2`（同属 127/8）仍判为非回环 → **少告警**。这是 **fail-safe 方向**（漏提示不漏保护），且符合「不引入 `IpAddr` 解析或更宽匹配」的硬约束 |
| **RI-4** | `is_loopback_bind_host` **无新增单元用例** | 建议补 `::1` / `::FFFF:127.0.0.1` / `0.0.0.0` / `192.168.1.7` 四条表驱动用例 |

### 2.5 提交后 artifacts 交接件脱离部分门禁监管（T-16 的既定代价）

- **风险**：`.codebuddy/artifacts/` 成为豁免域后，域内 **101** 个 md（含基线已入库的 **37** 个历史交接件）**不再受** AC-2 / AC-3 / AC-4 / AC-6 / AC-7b / AC-32 与 AC-8 段 1 约束。后续对历史交接件的汉化回退、或新增交接件的英文散文残留，**不会被门禁拦截**。
- **为什么接受**（依据 `00-decisions.md` Q6、PM 方案 A）：① 与 `SKIP_A`…`SKIP_D` 同源（按路径前缀判定、与基线无关），对未来所有 feature 一致；② 实测域内文件在当前判据下**全部 PASS**（解耦变体实测 `受检 231 / FAIL 0`），豁免**未掩盖任何既有失败**；③ 域内并未彻底脱管 —— AC-7a（改名/删除）仍生效，AC-8 段 2 仍以它们为「历史域」做反向断言。
- **残余覆盖缺口**：域内 md 的英文残留率与 code span 完整性今后**无自动化拦截**；如需恢复监管，必须走裁决（收窄豁免域或登记放行清单），不得由实现方自行决定。
- **数据影响**：无（纯文档域，不进编译产物、不影响运行时）。

### 2.6 其它

- **setup-seeds hash 变更**：种子内容汉化会改变其 hash，已安装用户将触发**一次种子重装**，可能覆盖用户对种子文件的手工改动（`05-test-report.md` §8 登记项）。
- **回滚代价**：文档类改动回滚代价低（文件级还原）；**默认绑定变更与 `lib.rs` 的 hunk 强耦合**，T-15 的三处改动亦**必须成对回退**（否则 clap `mut_arg` panic），另有 `webui/dist` 需要 `cargo clean -p rustcode-daemon`（详见 §6）。

---

## 3. 验证结果

全部引用 `05-test-report.md` 的实测数字，未重新执行 `cargo` / `npm`。

| 验证项 | 实测结论 | 出处 |
|---|---|---|
| 门禁 `gate --base 3ee655e3`（**G5 当时口径**：`total=285`、`受检 231 / FAIL 0`、历史域 39 处；AC-4 授权放行 12 条 D1 11 / D2 1；AC-8 正式域 0 命中、兜底域 0 命中） | **exit 0 / `gate: PASS`**。T-16 后分母口径按 Q6 变更，**当前实测**为 `total=248`、`受检 194 / FAIL 0`、历史域 270 处（见本节末代码块与 §1.8） | `05-test-report.md` §4.A / §5 + `03-impl/T-16-artifacts-exempt.md` §3.3 |
| `cargo fmt --check` | **exit 0**，0 行输出 | `05-test-report.md` §5 AC-31 |
| clippy | 3 个 crate（`-p rustcode-daemon -p rustcode -p rustcode-config --all-targets`）**exit 0**，`^error` 0；并用 `cargo check --workspace --all-targets`（exit 0）作跨 crate 补充 | `05-test-report.md` §5 AC-31、§7 |
| `cargo test --workspace` | **5487 passed / 1 failed**；唯一红测 `mcp::registry::tests::trust_key_golden_matches_core_algorithm`（**存量**，`AGENTS.md:226` 已文档化，文件不在本 feature 改动集内） | `05-test-report.md` §8 |
| `cargo test -p rustcode-daemon` | **316 passed / 0 failed** | `05-test-report.md` §4.C |
| T-13 返工验证 | `cargo test` 合计 **645 passed / 0 failed**；`gate` exit 0；运行时 `rustcode daemon` 绑 `0.0.0.0` **含 LAN 提示**，独立二进制绑 `127.0.0.1` **无提示** | `03-impl/T-13-r1-rework.md` §7 |
| 前端回归 | `npx tsc --noEmit` exit 0 / 0 错；`npm test` **227 pass / 0 fail**；`npm run build` 成功 | `05-test-report.md` §5 AC-33 |
| AC 总计 | **PASS 29 · 部分验证 2（AC-22 / AC-26）· 未验证 2（AC-23 / AC-25）· FAIL 0** | `05-test-report.md` §5 |
| **AC-27（审批闸门）** | **端到端实证**：用 mock provider 真实构造出 `permission_request` 事件（tool=bash，reason=Requires approval），破坏性工具**未**被自动执行，mock 未收到第二轮 | `05-test-report.md` §4.E / §5 |
| AC-28 | Q2-A 三要素在位（独立二进制 `127.0.0.1`、`webui_tokens: Some(..)` ⇒ `enforce_token=true`、`lib.rs:1284-1294` 覆盖 Channel/Webui/Vscode/Jetbrains）；无 token `POST /chat` → **401** | `05-test-report.md` §5 |
| AC-19 / AC-20 | `rustcode daemon` 运行时实测绑 `0.0.0.0:13497`；显式 `--host 127.0.0.1` 时 `ss` 只见 `127.0.0.1:13458`、`0.0.0.0:13458` 计数 0 | `05-test-report.md` §5 |
| AC-15 / AC-18 | 构建脚本三种模式（默认重跑 / `--if-missing` + dist 存在 / `--if-missing` + dist 缺失）exit 0；端到端 `/health` 200、`/` 200 且 `grep -c '<html'` = 1 | `05-test-report.md` §5 |
| **T-15 `default_host_tests`** | `cargo test -p rustcode --bin rustcode default_host_tests` → **3 passed / 0 failed**（R2 新增 `daemon_defaults_to_all_interfaces`，由原 2 passed 增至 3；只锁默认值 `0.0.0.0`，按执行书**不**锁非默认值） | `03-impl/T-15.md` §10.3 |
| **T-15 `shell_completion`** | **1 passed / 0 failed**（`cargo test -p rustcode --test shell_completion`） | `03-impl/T-15.md` §4.9 / §10.3 |
| **T-15 `rustcode-daemon`** | **316 passed / 0 failed**（与 `05-test-report.md` §4.C 的 316/0 一致，无回归） | `03-impl/T-15.md` §4.9 |
| **T-15 门禁** | `gate --base 3ee655e3` **exit 0 / PASS**（T-15 收尾与 R2 追加后各复跑一次） | `03-impl/T-15.md` §4.9、§10.3 |
| **T-16 提交（stage）前后一致** | `git add -A` **前**与**后** `gate --base 3ee655e3` **均 exit 0**；`inventory` 恒等式 `SKIP_A=4 SKIP_B=47 SKIP_C=1 SKIP_D=2 TODO=27 ZH=167 total=248 OK`（分母口径按 Q6） | `03-impl/T-16-artifacts-exempt.md` §3.3、§9.4 |
| **T-16 域内实测 PASS** | 解耦变体（只收窄 AC-1 豁免域）实测 `受检 231 / FAIL 0`，即 37 个历史交接件在当前判据下**无一失败** ⇒ 豁免未掩盖失败 | `03-impl/T-16-artifacts-exempt.md` §9.2 |
| **本轮（doc-writer 契约登记轮）复跑** | `gate --base 3ee655e3` **exit 0 / PASS**（`total=248`、`受检 194 / FAIL 0`）；`check --files AGENTS.md` **PASS / exit 0**（`AGENTS.md:37` 修订后 0 新增 code span） | 本节末代码块 |

**G6 当时的补充实测快照**（doc-writer 执行，只读命令；**为 T-16 之前**的口径：未跟踪排除 58、`total=285`、`受检 231**。T-16 后分母与受检数已按 Q6 变更，当前实测见紧随其后的代码块）：

```
$ python3 scripts/check-zh-docs.py gate --base 3ee655e3
gate: 未跟踪 md 已排除：58 个（不进 AC-1 分母，也不受 AC-2/4/6/7b/32 与 AC-8 段 1 约束）
PASS AC-1 清单自洽
  SKIP_A=4 SKIP_B=47 SKIP_C=1 SKIP_D=2 TODO=29 ZH=202 total=285
PASS AC-2/3/4/6/7b/32 全量 check
  全量 check 受检 231，FAIL 0
PASS AC-5 运行时载荷
  AC-5a rules 目录无改动: OK
  AC-5b setup-seeds 有改动 (6 个文件): OK
  AC-5b 无 ^[+-]name: 变更: OK
PASS AC-7a 无改名/删除
PASS AC-8 外链残留
  段 1 正式域（候选 515，已排除 .codebuddy/artifacts/）：0 命中 -> OK
  段 2 历史域（候选 37）：命中 39 处，全部位于 .codebuddy/artifacts/ -> OK（历史痕迹仍在）
  段 3 兜底域 site/.github/docs/extensions（全扩展名）：0 命中 -> OK
gate: PASS
GATE_EXIT=0

$ python3 scripts/check-zh-docs.py inventory --base 3ee655e3
inventory: total=285 SKIP_A=4 SKIP_B=47 SKIP_C=1 SKIP_D=2 TODO(EN=0,MIXED=29) ZH=202 恒等式=OK

$ python3 scripts/check-zh-docs.py check --base 3ee655e3 --files README.md
PASS README.md en=0/374=0.0000
  [AC-4-authorized] README.md | removed=- | added=./scripts/build-webui.sh | cause=D2
check: 受检 1，PASS 1，FAIL 0
EXIT=0

$ python3 scripts/check-zh-docs.py check --base 3ee655e3 --files \
    <SKILL.md> <references/hooks-patterns.md>
PASS .../SKILL.md en=1/152=0.0066
PASS .../references/hooks-patterns.md en=1/111=0.0090
check: 受检 2，PASS 2，FAIL 0
EXIT=0
```

**本轮（T-15 / T-16 之后 · 契约登记轮）实测**（doc-writer 执行，只读命令；口径已按 Q6）：

```console
$ python3 scripts/check-zh-docs.py gate --base 3ee655e3
gate: 未跟踪 md 已排除：1 个（不进 AC-1 分母，也不受 AC-2/4/6/7b/32 与 AC-8 段 1 约束）
  - .codebuddy/memory/2026-09-09.md

gate: artifacts 豁免域 `.codebuddy/artifacts/` 已排除：101 个（与 SKIP_A..SKIP_D 同级，按路径前缀判定、与基线无关；不进 AC-1 分母，也不受 AC-2/4/6/7b/32 与 AC-8 段 1 约束；域内文件仍是 AC-8 段 2 的历史域）

gate: base=3ee655e3

PASS AC-1 清单自洽
  SKIP_A=4 SKIP_B=47 SKIP_C=1 SKIP_D=2 TODO=27 ZH=167 total=248
PASS AC-2/3/4/6/7b/32 全量 check
  全量 check 受检 194，FAIL 0
  AC-4 授权放行合计 12 条（D1 11 / D2 1）
PASS AC-5 运行时载荷
PASS AC-7a 无改名/删除
PASS AC-8 外链残留
  段 1 正式域（候选 515，已排除 .codebuddy/artifacts/）：0 命中 -> OK
  段 2 历史域（候选 101）：命中 270 处，全部位于 .codebuddy/artifacts/ -> OK（历史痕迹仍在）
  段 3 兜底域 site/.github/docs/extensions（全扩展名）：0 命中 -> OK

gate: PASS
GATE_EXIT=0

$ python3 scripts/check-zh-docs.py inventory --base 3ee655e3 | tail -n 1
inventory: total=248 SKIP_A=4 SKIP_B=47 SKIP_C=1 SKIP_D=2 TODO(EN=0,MIXED=27) ZH=167 恒等式=OK

$ python3 scripts/check-zh-docs.py check --base 3ee655e3 --files AGENTS.md
PASS AGENTS.md en=0/546=0.0000
  [AC-4-authorized] AGENTS.md | removed=README.zh-CN.md | added=- | cause=D1
  [AC-4-authorized] AGENTS.md | removed=README.zh-CN.md | added=- | cause=D1
  [AC-4-authorized] AGENTS.md | removed=README.zh-CN.md:94 | added=- | cause=D1

check: 受检 1，PASS 1，FAIL 0
CHECK_EXIT=0

$ git -c core.quotePath=false ls-files -- '*.md' | wc -l
349
$ git -c core.quotePath=false ls-files -- '*.md' | grep -c '^\.codebuddy/artifacts/'
101          # 349 - 101 = 248 = 脚本分母
```

> 段 2 历史域的命中数会随域内交接件正文增长（37/39 → 101/270），这是「历史域」既有设计；**判定（命中全部位于域内）与退出码 0 不变**。

**测试覆盖到的入口**（据 `05-test-report.md` 逐条登记）：

- CLI：`rustcode webui`、`rustcode daemon`（默认 / `--host 127.0.0.1` 覆盖 / `--host` 缺值 / `--host` 非法值 / `--host=<IP>` 等号形式）、`rustcode webui --help` / `--bogus`、`rustcode daemon --help`（en 与 `--lang zh` 两语种）、独立二进制 `rustcode-daemon`
- 脚本：`scripts/build-webui.sh`（默认 / `--if-missing` / 无 node / node 版本不足）
- HTTP：`GET /health`、`GET /`、`POST /chat`（无 token → 401）、`POST /providers`、`POST /providers/<id>/default`、`GET /providers`
- 审批：`permission_request` 事件（mock provider + 破坏性 bash 工具）
- 前端：`tsc --noEmit`、`npm test`、`npm run build`
- 文档门禁：`inventory` / `check` / `hostscan` / `gate` 四个子命令（含合法与非法 `--base`），并覆盖 `git add -A` **前 / 后**两种状态（T-16 §3.3）

---

## 4. 已知未验证范围

| 项 | 未验证内容 | 所需环境 | 建议补验方式 | 负责人建议 |
|---|---|---|---|---|
| **AC-22（跨设备部分）** | 局域网**另一台主机**对 `<LAN-IP>:<port>` 的 `/health` 与 `/`。本机以 LAN IP `172.24.0.2:13465` 自测已通过（200/200），但**跨主机未验证** | 同一二层网络的第二台主机（或另一容器 / network namespace + veth） | 在第二台主机执行 `curl -sSf http://<LAN-IP>:<port>/health` 与 `curl -sSf http://<LAN-IP>:<port>/ \| grep -c '<html'` | test-engineer（需环境） |
| **AC-23** | 浏览器同源 fetch **无 CORS 错误**。目前只有静态证据：`lib.rs:1222-1227` `allow_origin(predicate(is_loopback_origin))`，页面由 daemon 同源提供 | 真实浏览器（DevTools Network 面板）+ 已完成 `?token=` → Cookie 交接的页面 | 打开 `http://<LAN-IP>:<port>/?token=…`，确认 `/health`、`/project`、`/providers` 全 2xx 且 Console 无 CORS 错误；或 headless Chrome 抓 console 日志 | test-engineer（需浏览器环境） |
| **AC-25** | 无 provider 时首页**无全屏阻断遮罩**、可打开设置对话框。替代证据仅为 `webui/src/app.tsx` grep `provider` 0 命中 | 真实浏览器渲染 | Playwright/Puppeteer 打开首页，断言无「必须先配置 provider」类遮罩；点击设置 → provider 列表可见 | test-engineer（需浏览器环境） |
| **AC-26（UI 层）** | 在**设置对话框 UI** 中新建 provider 并设为默认。**注**：后端链路已实证（`POST /providers` 201 含 `is_default: true` → `POST /providers/t07-api/default` 200 → `config.toml` 落 `default_provider` 与 `[providers.t07-api]` → `GET /providers` 回读一致，无需重启），仅 UI 交互未验证 | 真实浏览器 | 在 UI 中填 name/base_url/api_key/model → 保存 → 设默认 → 校验 `config.toml` 与 `GET /providers` | test-engineer（需浏览器环境） |
| **AC-31 clippy 全 workspace** | `cargo clippy --workspace --all-targets` | 无（仅耗时） | 本轮按派单许可只跑 3 个 crate（exit 0），已用 `cargo check --workspace --all-targets`（exit 0，`^error` 0）作跨 crate 补充；如需完整 clippy 单独排期 | test-engineer |
| **AC-27 的真实 IDE 拉起路径** | 由 VS Code 扩展（`extensions/vscode/src/daemon/process.ts:369`）真实拉起 daemon（不带 `--host`） | VS Code + 已构建扩展 | 本轮用命令行 `--port 13456 --client vscode` + `X-RustCode-Client: vscode` 头**等价模拟**（`startup_mode` 是死字段 N1，`--client` 对请求处理无影响，客户端身份 100% 来自请求头，等价性成立） | 可豁免；若需，由 IDE 扩展 owner 补验 |
| **AC-13/14 在 macOS/Windows 的行为** | 脚本在非 Linux 下的 PATH/版本解析 | macOS / Windows 主机 | 本轮仅在 Linux（zsh/bash）验证 | 可豁免（脚本为 bash，验证环境为 Linux） |

**遗留后续项与负责人建议**：

1. **D-1（含 RI-1）** → **architect**：`is_loopback_authority` 本体是否改用 `normalize_authority` + `IpAddr` 解析，连同 `client_interactive_permission` 的权限面影响一并裁决；`ensure_server_and_open` 的两处调用（`lib.rs:5393`、`lib.rs:5422`）是否一并收敛。
2. **D-3** → **T-01**：`check-zh-docs.py` 的 AC-2 围栏状态机对未闭合/嵌套围栏失效（`03-impl/zh-check-D-34a.md` §5.1 有可复现实例）。
3. **D-4** → **architect**：`Msg::WebuiLanWarning` 的 P1/P2/P3 三种文案裁决。
4. ~~**D-5** → PM / code-implementer：是否为 `rustcode daemon` 子命令补 `--host` 参数~~ **已由 T-15 完成并闭环**（§1.7）。接替项：**T-15 §8 O-4 扩展侧前向兼容** → **PM / IDE owner**：JetBrains `RustCodeDaemonProcess.kt:113-126` 与 VSCode `process.ts:369` 目前不传 `--host`，若要让扩展传 `--host 127.0.0.1`，须先做**版本门控**（旧 CLI 收到 `--host` 会 exit 2），建议与本看板「待用户裁决」第 3 条合并评估。
5. **RI-4** → **test-engineer**：为 `is_loopback_bind_host` 补 4 条表驱动用例。
6. **AC-27 的 mock 夹具固化** → **PM 派单给 code-implementer**：建议把本轮 `/tmp` 下的 mock OpenAI 兼容 provider 夹具固化为 `crates/rustcode-daemon/tests/` 下的集成测试（本轮 `files_owned` 不含测试代码，未落地）。
7. **JetBrains 插件显式传 `--host 127.0.0.1`** → **PM / IDE owner**：建议单开小任务（见 STATUS.md「待用户裁决」段）。
8. **`lib.rs:5264-5265` 与 `lib.rs:5442-5443` 的「VSCode daemon `enforce_token=false`」注释**与 `daemon/src/main.rs:173` 矛盾（**既存**、不在本 feature `files_owned`）→ 已裁决**本轮不改**，登记 TODO 待后续处理。
9. **非法 host 时先打印「监听地址 …」再 exit 1（T-15 §8 O-3，复审 nit）** → **`rustcode-daemon` / architect 另派单（低优先）**：`crates/rustcode-daemon/src/lib.rs:6379-6394` 的横幅先于 `:6498-6509` 的 `TcpListener::bind` 打印，输出陈述了尚未成立的事实（「已启动」「已绑定」）；退出码 1 与 stderr `致命错误：无法绑定到 …` 均正确，**无静默失败、无假成功**，故不阻塞。修复面在 `crates/rustcode-daemon`（T-15 契约 §4.4 冻结），不在本 feature 改动集内。
10. **`OWNED_ELSEWHERE` 死配置与 T-16 报告文字勘误（复审 R2 minor）** → **PM（需授权后由 T-16 或下轮处理）**：`scripts/check-zh-docs.py:40` 的 `OWNED_ELSEWHERE` 第 3 个成员 `.codebuddy/artifacts/2026-09-07-zh-docs-webui/` 在 T-16 后已失效（`is_owned_elsewhere()` 的输入已排除 artifacts 域）；`03-impl/T-16-artifacts-exempt.md` §0 称改动 6 hunk 而 §1 表格列 7 行、§4.2 把 `quotePath` 后果写成「静默」（实测 `list_md_files()` 路径是 exit 2 响亮失败，只有 `resolve_files()` 路径为真静默）。二者均**不影响任何判据**，本轮按裁决未改（不在 `files_owned`）。

---

## 5. 文档更新清单

| 文件路径 | 变更类型 | 摘要 |
|---|---|---|
| `README.md` | 修订 | 全文英译中（`en` 388/409 → 0/374）；顶部中英切换导航移除；补 WebUI 四处理事实：构建步骤指向 `./scripts/build-webui.sh`（`README.md:200`）、默认绑定 `0.0.0.0`（`:120`）、TUI `/webui` 默认仍 `127.0.0.1`（`:121`）、WebUI 无需预配 provider 且可在网页端配置（`:122`）；`cargo clean -p rustcode-daemon` 约定（`:204`、`:217-218`） |
| `README.zh-CN.md` | **删除** | 与汉化后的 `README.md` 重复，删除 `-830` 行；`git ls-files` 已为空 |
| `AGENTS.md` | 修订（他人批次） | 移除对 `README.zh-CN.md` 的引用（AC-4 授权放行 `cause=D1` × 3）；`AGENTS.md:16` 已含 `webui/dist` + `cargo clean -p rustcode-daemon` 约定 |
| `AGENTS.md` | 修订（**T-10b，本轮**） | 补记本 feature 的两项行为变化（此前 `0.0.0.0` / `build-webui` 关键字均 **0 命中**），**纯新增 7 行、0 删除**：`AGENTS.md:17` 新增 `scripts/build-webui.sh` 一键构建条目（幂等、`--if-missing`、无 node / node 版本不足 fail-closed 退出 2）；`:28` 新增 `scripts/check-zh-docs.py` 门禁用法；`:36-39` 新增「WebUI 默认绑定地址」段，区分 CLI 两入口（`rustcode webui` / `rustcode daemon`）默认 `0.0.0.0` 与独立 `rustcode-daemon` 二进制 / TUI `/webui` 仍 `127.0.0.1`（安全红线），并记载非回环提示已由 `Msg::DaemonWarnNonLoopback` 启动横幅改为 `Msg::WebuiLanWarning` / `Msg::WebuiNonLoopbackWarning`（`run_server` 非 quiet 分支补发，IPv6 回环经 `is_loopback_bind_host` 不告警）。`AGENTS.md:322` 的「`127.0.0.1:13456 默认`」属刻意保留的历史留档，**未改动** |
| `crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/SKILL.md` | 修订 | 汉化（144/153 → 1/152）；`description` 值英译中；`name:` 等 frontmatter 键名集合未变 |
| `crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/hooks-patterns.md` | 修订 | 汉化（100/111 → 1/111） |
| `crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/{mcp-servers,plugins-reference,skills-reference,subagent-templates}.md` | 修订 | 汉化，4 个文件 `en` 均降至 0（见 `03-impl/zh-check-D-34b.md`） |
| 其余 **149** 个 md | 修订 | 分批汉化（D-01…D-33、D-36 等）。构成核对：`157 = 1(README.md) + 1(README.zh-CN.md，删除) + 6(setup-seeds) + 149` |
| `.codebuddy/artifacts/2026-09-07-zh-docs-webui/STATUS.md` | 修订（**重写**） | 旧版停留在「开发收尾 / 门禁 FAIL / T-09、T-11 in_progress」，已按实测重写为「交付 / G6 in_progress / gate PASS」 |
| `.codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-34a.md` | **新增** | 补 D-34a 缺失报告（`SKILL.md` + `references/hooks-patterns.md`） |
| `.codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-35.md` | **新增** | 补 D-35 缺失报告（根 `README.md` 汉化 + 删 `README.zh-CN.md`，AC-8/9/10/11） |
| `.codebuddy/artifacts/2026-09-07-zh-docs-webui/06-release.md` | **新增** | 本文件（G6 交付说明） |
| `AGENTS.md:37` | 修订（**本轮，T-15 后**） | 由「`rustcode daemon` 子命令没有 `--host` 参数」改为如实记载「同样支持 `--host`、默认 `0.0.0.0`、可显式传 `127.0.0.1` 退回仅本机」，并把行内源码引用由 `main.rs:1780` 改为 `main.rs:1031-1034`（`default_value`）+ `:1786`（`ServerOpts`）。**0 新增 code span**（`AGENTS.md` 不在 `AC4_ALLOWED_ADDED_BY_FILE`），`check --files AGENTS.md` PASS / exit 0 |
| `.codebuddy/artifacts/2026-09-07-zh-docs-webui/00-decisions.md` | 修订（**本轮，追加**） | 追加 **Q6**：AC-1 分母 = 全仓 md 减显式豁免域 `.codebuddy/artifacts/`；域内不进分母、不受 AC-2/4/6/7b/32 与 AC-8 段 1 约束、仍受 AC-7a 与 AC-8 段 2 约束；记录 285→248 / 231→194 与差值 37 的构成、方案 A 优于方案 B 的理由、对 `00-requirement.md:259` 与 `01-design.md:60` 的口径取代关系。Q1–Q5 **未改动** |
| `.codebuddy/artifacts/2026-09-07-zh-docs-webui/STATUS.md` | 修订（**本轮**） | 实测现状表改为当前数字（`total=248`、`TODO=27`、`ZH=167`、受检 194 / FAIL 0）并注明口径按 Q6；任务表补 T-15 / T-16；D-5 行改为「已由 T-15 修复」；R3 提交口径改为「已裁决：全部提交（含 artifacts），待 PM commit」；补 R2 复审遗留（OWNED_ELSEWHERE 死配置、T-16 §1 hunk 计数与 §4.2「静默」措辞勘误、非法 host 前置横幅 nit） |
| `.codebuddy/artifacts/2026-09-07-zh-docs-webui/06-release.md` | 修订（**本轮**） | 补 §1.7（T-15 `--host`）、§1.8（T-16 豁免域与分母口径）、§2.5（提交后 artifacts 脱离部分监管的风险）、§3 的 T-15/T-16 验证行与当前复跑输出、§6.4 / §6.5（T-15 成对回退、T-16 撤销豁免后果）、§5 本文档行与 §7 编号范围 |
| `CHANGELOG` | 无 | 仓库根目录无 `CHANGELOG` 文件，本轮不新增 |

---

## 6. 回滚方案

### 6.1 方案 A：仅回退提交（改动尚未提交，最简）

当前 `HEAD == 基线 3ee655e3`，**全部改动都在未提交工作区**，因此「回滚」= 还原工作区，无需 revert commit：

```bash
# 1) 还原全部已跟踪文件的改动（含 157 个 md、5 个 rust 文件、README.zh-CN.md 的删除）
git checkout 3ee655e3 -- .

# 2) 移除本 feature 新增的未跟踪文件
rm -f scripts/check-zh-docs.py scripts/build-webui.sh
rm -f crates/rustcode-daemon/tests/default_host_lock.rs
rm -f crates/rustcode-config/tests/cli_webui_i18n_lock.rs

# 3) 确认回到基线
git status --porcelain   # 只剩 .codebuddy/** 交接件等未跟踪项
python3 scripts/check-zh-docs.py gate --base 3ee655e3   # 脚本已删，此步应不可用 → 说明已回到无脚本状态
```

- **判定时机**：`git status --porcelain` 中不再出现任何 ` M` / `D ` 条目，且 `git diff 3ee655e3 --stat` 为空。
- **代价**：丢失全部汉化成果与默认绑定变更；`README.zh-CN.md` 恢复。
- **注意**：不要只删 `scripts/check-zh-docs.py` —— 它是本 feature 的门禁工具，删除后 AC-1…AC-8 无法复验。

### 6.2 方案 B：只回退「默认绑定」这一项行为变更（保留汉化）

默认绑定变更涉及 5 个 hunk 分布在 3 个文件，**必须按耦合关系成对回退**，不可拆分：

| 文件 | 回滚内容 | 是否可单独回退 |
|---|---|---|
| `crates/rustcode-cli/src/main.rs:1047` | `default_value = "0.0.0.0"` → `"127.0.0.1"`（`rustcode webui`） | 是 |
| `crates/rustcode-cli/src/main.rs:1034`（T-15 后） | `--host` 的 `default_value = "0.0.0.0"` → `"127.0.0.1"`（`rustcode daemon`；`:1786` 现为变量 `host`，改默认值即可，无需动 `:1786`） | 是 |
| `crates/rustcode-daemon/src/lib.rs:6332-6350`（改前编号） | 删除的 `eprintln!(Msg::DaemonWarnNonLoopback)` 横幅 | **否** —— 与下一行强耦合 |
| `crates/rustcode-daemon/src/lib.rs:6357-6374`（改前编号） | 新增的 `if !is_loopback_bind_host(&host) { ... }` 补发块 | **否** —— 与上一行强耦合 |
| `crates/rustcode-daemon/src/lib.rs:1281-1303` | T-14 新增的 `is_loopback_bind_host`（含 `lib.rs:6388` 的调用切换） | 是（回退后 `::1` 误报回归，即 D-1 复现） |

**关键耦合警告**（引自 `03-impl/T-13-r1-rework.md` §6）：`lib.rs` 的 MAJOR-3 两个 hunk 是同一逻辑改动的两半——

- **只回退后半（删除补发块）** ⇒ `rustcode daemon --host 0.0.0.0` **完全静默、无任何非回环提示**，属**最危险**情况（静默暴露，违反「失败路径必须显式错误 / 禁止静默」约束）；
- **只回退前半（恢复旧横幅）** ⇒ 旧横幅与新增提示并存，重复告警。

因此 MAJOR-3 只能**整体回退**（`git checkout 3ee655e3 -- crates/rustcode-daemon/src/lib.rs` 后重放其余 hunk）或**整体保留**。

```
# B-1 回退两个 CLI 默认值（安全，可直接单独执行）
#     编辑 crates/rustcode-cli/src/main.rs:1047（webui）与 :1034（daemon 的 --host default_value），
#     把 "0.0.0.0" 改回 "127.0.0.1"；:1786 的 ServerOpts { host, .. } 无需改动（T-15 后是变量传参）
#     注：若按 §6.4 整体回退 T-15，则三处必须成对，且要一并删除 R2 新增的 daemon_defaults_to_all_interfaces

# B-2 回退 lib.rs 的提示逻辑（必须整体）
git checkout 3ee655e3 -- crates/rustcode-daemon/src/lib.rs

# B-3 回退独立二进制默认 host —— 无需操作：main.rs:21 本就保持 127.0.0.1

# B-4 复验
cargo test -p rustcode-daemon            # 期望 default_host_lock 仍通过
cargo build -p rustcode-daemon -p rustcode
```

- **判定时机**：`rustcode daemon` 与 `rustcode webui` 默认启动后不再打印 LAN 提示（`ss` 只见 `127.0.0.1:<port>`、`0.0.0.0` 计数 0）；`crates/rustcode-daemon/tests/default_host_lock.rs::standalone_daemon_default_host_stays_loopback` 仍通过。
- **代价**：失去跨设备访问能力；T-14 的 IPv6 收敛随之失效（若走了 B-2）。

### 6.3 `webui/dist` 相关的强制步骤（两种方案都适用）

`cargo build` **不跟踪** `webui/dist/` 的变化。任何回滚或重编之后，若要让二进制嵌入正确的前端产物，**必须**执行：

```bash
cargo clean -p rustcode-daemon
cargo build --release -p rustcode-daemon      # 或整体 cargo build
```

否则会出现「代码已回滚、但二进制里仍嵌着旧 bundle」的不一致状态。若同时要重建前端，先 `./scripts/build-webui.sh` 再 `cargo clean -p rustcode-daemon`。

### 6.4 只回退 T-15（`rustcode daemon --host`）——三处必须成对

T-15 的三处改动在同一文件、同一 clap 参数上，**必须成对回退**（`03-impl/T-15.md` §6.2）：

| # | 回退动作 | 位置（改后行号） |
|---|---|---|
| 1 | 删除 `host: String` 字段及其 2 行 doc comment 与 `#[arg(long, default_value = "0.0.0.0")]` | `crates/rustcode-cli/src/main.rs:1031-1034` |
| 2 | 删除 `.mut_arg("host", \|a\| a.help(t(Msg::CliHelpHost).into_owned()))` 一行 | `crates/rustcode-cli/src/main.rs:470` |
| 3 | `Commands::Daemon` 解构删 `host,`；`ServerOpts { host, … }` 改回 `host: "0.0.0.0".to_string(),` | `crates/rustcode-cli/src/main.rs:1745`、`:1786` |
| 3b | （R2 新增哨兵）删除 `daemon_host()` 与 `daemon_defaults_to_all_interfaces()`，否则回退后编译不通过 | `crates/rustcode-cli/src/main.rs` `mod default_host_tests` 内 |

**漏回退的后果（必须写清）**：clap 的 `mut_arg` 对不存在的 arg id 是**无条件 panic**（`clap_builder-4.6.0/src/builder/command.rs:244-254`，非 debug-only）。

- 只回退 1（删字段）而保留 2（保留 `mut_arg`） ⇒ `rustcode --help`、`rustcode <任一子命令> --help`、`rustcode completion <shell>` **全部 panic**；
- 只回退 2 而保留 1 ⇒ 不 panic，但 `--host` 退化成无 help 的裸参数，属**静默降级**，同样不可接受。

回退后验证：`cargo build -p rustcode`；`./target/debug/rustcode daemon --help \| grep -c -- '--host'` 期望 **0**；`rustcode --help` 与 `rustcode completion bash` 均 exit 0（不 panic）；`cargo test -p rustcode --bin rustcode default_host_tests` 回到 **2 passed**。

### 6.5 只回退 T-16（撤销 artifacts 豁免域）——**提交后会打红**

最小动作：去掉 `scripts/check-zh-docs.py:196-199` 的 `if not is_artifacts_exempt(path)` 与 `:985-991` 的 `and not is_artifacts_exempt(path)`（常量与提示可保留）。

**后果（实测，`03-impl/T-16-artifacts-exempt.md` §6）**：在已 stage 状态下用去掉两处过滤的脚本实测：

```console
$ python3 <no-exempt 脚本> gate --base 3ee655e3
  SKIP_A=4 SKIP_B=47 SKIP_C=1 SKIP_D=2 TODO=29 ZH=266 total=349
  全量 check 受检 295，FAIL 64
FAIL AC-2/3/4/6/7b/32 全量 check
gate: FAIL
NOEXEMPT_GATE_EXIT=1
```

即：**撤销豁免后再提交，门禁由 PASS 变为 FAIL（exit 1）**，受检 295 / FAIL 64，全部源于「新增文件无基线 → AC-4 把全文 code span 记为 added」的假 FAIL（含非 ASCII 文件名的 `HANDOFF-汉化铁律.md`）。AC-8 三段不受影响（段 1 仍 0 命中），因为 AC-8 的排除前缀是独立常量。

**因此：若必须回退 T-16，须同时回退 R3 提交口径**（artifacts 交接件不入库），否则提交即红。判定时机：`python3 scripts/check-zh-docs.py gate --base 3ee655e3` 由 exit 0 变为 exit 1 且 `FAIL 64`。

### 6.6 回滚判定时机汇总

| 场景 | 判定命令 | 期望 |
|---|---|---|
| 确认已回到基线 | `git diff 3ee655e3 --stat` | 空 |
| 确认默认绑定已回退 | 启动 daemon 后 `ss -ltnp \| grep <port>` | 只见 `127.0.0.1:<port>`，`0.0.0.0` 计数 0 |
| 确认汉化未被破坏 | `python3 scripts/check-zh-docs.py gate --base 3ee655e3` | exit 0 / `gate: PASS` |
| 确认独立二进制红线 | `cargo test -p rustcode-daemon -- default_host_lock` | 通过 |
| 确认 T-15 已回退 | `./target/debug/rustcode daemon --help \| grep -c -- '--host'`；`cargo test -p rustcode --bin rustcode default_host_tests` | `0`；`2 passed` |
| 确认 T-16 未被误撤 | `python3 scripts/check-zh-docs.py inventory --base 3ee655e3 \| tail -n 1` | `total=248 … 恒等式=OK`（若为 `349` 说明豁免被撤销，提交会打红） |

---

## 7. 术语与命名一致性检查结论

- **产品与 crate 命名**：`rustcode` / `RustCode`、crate 名（`rustcode-cli`、`rustcode-daemon`、`rustcode-config`、`rustcode-capabilities`）、二进制名（`rustcode`、`rustcode-daemon`）、配置目录 `~/.rustcode`、环境变量 `RUSTCODE_*` 在本说明与全仓文档中均为英文原样，未本土化。
- **技术术语**：`provider`、`token`、`hook`、`subagent`、`skill`、`plugin`、`MCP`、`LSP`、`BYO`、`TUI`、`WebUI`、`daemon`、`bundle`、`CORS`、`TLS`、`IPv6`、`loopback` 保持英文原样；中文表述统一用「回环」（非「环回」）、「非回环」、「局域网」（非「内网/LAN」混用，引用代码符号时保留 `LAN`）。
- **符号一致性**：本说明引用的符号与实现逐条核对一致 —— `Msg::WebuiLanWarning` / `Msg::WebuiNonLoopbackWarning` / `Msg::DaemonWarnNonLoopback`（`messages.rs:2957/2959/4903`）、`is_loopback_bind_host`（`lib.rs:1281-1303`）、`is_loopback_authority`、`client_interactive_permission`、`DEFAULT_HOST`（`daemon/src/main.rs:21`）、`webui_tokens`（`daemon/src/main.rs:177`）。
- **路径与命令**：`scripts/build-webui.sh`、`cargo clean -p rustcode-daemon`、`webui/dist`、`--host`、`--if-missing`、`--en-ratio`、`--base` 与实现/脚本帮助文本逐字一致。
- **文档口径**：AC 编号（AC-1…AC-33）、缺陷编号（D-1…D-5）、遗留项编号（RI-1…RI-5）、门禁编号（G1…G6）、任务编号（T-01…**T-16**、D-01…D-36）沿用 `00-requirement.md` / `02-tasks.md` / `01-design-addendum.md` / `01-design-addendum-d5.md` / `05-test-report.md` 的原编号，本说明未新造编号（T-15 = `rustcode daemon` 补 `--host`，T-16 = artifacts 门禁豁免域）。
- **AC-1 分母口径（本轮新增，务必与其它文档一致）**：一律写作「**受检 md 总数（已排除 artifacts 豁免域）**」，当前 **248**；「全仓 md 总数」若出现，指 `git ls-files -- '*.md'` 的 **349**（含豁免域 101）。取代关系由 `00-decisions.md` **Q6** 登记，`00-requirement.md:259` 的旧字样按 Q6 理解（未改原文）。
- **Antora / 文档域命名**：本轮未引入新的文档域；「artifacts 豁免域」「历史域」「正式域」「兜底域」均为 `scripts/check-zh-docs.py` 与 `01-design-addendum.md` §3 既有称谓，未新造同义混写。
- **新旧称谓对照（迁移期）**：旧的「非回环安全警告 / security warning 横幅」= `Msg::DaemonWarnNonLoopback`（已退役发射点）；新的提示 = `Msg::WebuiLanWarning`（`0.0.0.0` / `::`）+ `Msg::WebuiNonLoopbackWarning`（其它非回环地址）。文档与注释中**不得**再把 `DaemonWarnNonLoopback` 描述为「仍在打印」。
- **无残留歧义**：本说明无占位符、无「待补」；文中出现的 `TODO=27`（以及历史快照中的 `TODO=29`）是 `inventory` 输出的**桶名**（MIXED 待汉化桶），不是未完成标记；「登记 TODO」两处均指已裁决、已指定归属的后续项。文中保留的旧数字（`total=285` / `受检 231` / 未跟踪 58 / `169`）均**显式标注为 T-16 之前的快照**，与当前值（248 / 194 / 1 / 233）不冲突。未把未验证项写成已验证（§4 与 `05-test-report.md` §7 逐条一致）。
