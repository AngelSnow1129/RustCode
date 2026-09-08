---
kind: test-report
id: TEST-001
from: test-engineer
to: [project-manager, code-implementer]
feature: 2026-09-07-zh-docs-webui
status: done
decision: proceed
requires: [DESIGN-001, T-01, T-02, T-03, T-04, T-05, T-06, T-07, T-13]
files_owned:
  - .codebuddy/artifacts/2026-09-07-zh-docs-webui/05-test-report.md
  - .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-gate.md
architecture_constraints:
  touches_runtime_lifecycle: true
  touches_persistence: false
  touches_cross_crate_deps: false
created: 2026-09-09
---

# T-07 集成验收报告（G5）· 全仓 md 汉化 + WebUI 开箱可访问 + 默认绑定 0.0.0.0

> 本轮只写 `files_owned` 两个文件，未修改任何源码 / 脚本 / md 正文 / 文档；未 `git add`、未 `git commit`。
> 所有数字均为本轮实测。未验证项一律标为「未验证」，不写成通过。

---

## 1. 元信息

| 项 | 实测值 |
|---|---|
| 分支 | `dev` |
| 基线 `ZH_BASE` | `3ee655e381d28052e428196daedd456cc6079520` |
| `git rev-parse HEAD` | `3ee655e381d28052e428196daedd456cc6079520`（**HEAD == 基线**，全部改动在未提交工作区） |
| `git status --porcelain \| wc -l` | 开工 **169** → 收工 **169**（本轮未新增/删除任何文件） |
| 执行时间窗（UTC） | `2026-09-08T18:05:39Z` → `2026-09-08T18:38:58Z`（本地 CST 02:05 → 02:39） |
| 执行人 | `test-engineer`（G5） |

### 1.1 并发扰动记录

| 时点 | 观察 | 结论 |
|---|---|---|
| 18:12 开工前 | `pgrep -c cargo` = **0** | 无遗留 cargo 占用 |
| 18:12–18:17 | `cargo test --workspace --no-fail-fast` 后台运行 | 全程日志**无** `Blocking waiting for file lock` |
| 18:13–18:16 | 同时执行 D 段 `scripts/build-webui.sh`（`npm ci` + `vite build`） | npm 与 cargo 不共享锁，二者**无冲突**；实测无失败重试 |
| 18:17–18:39 | cargo 类命令（fmt / config / daemon / clippy / clean+build / check）**全部串行** | 无并发 cargo 争用 |
| 全程 | 所有退出码均为**首次执行**结果 | 无重试掩盖失败 |

### 1.2 硬约束自检

| 约束 | 结果 |
|---|---|
| 未修改任何源码 / 脚本 / md 正文 | 通过。本轮仅写 `05-test-report.md` 与 `03-impl/zh-check-gate.md`（后者由 `gate --report` 覆写） |
| 未 `git add` / `git commit` | 通过。仅使用 `rev-parse` / `status` / `diff` / `ls-files` / `show` / `grep` / `check-ignore` 只读命令 |
| 未删除 / `#[ignore]` 任何既有测试 | 通过。本轮未新增测试文件（T-07 为验收任务，按派单不需要新增测试代码） |
| 未放宽断言以让测试变绿 | 通过。所有断言均为 AC 原文判据 |
| `webui/dist` 破坏性用例已恢复 | 通过。AC-13/14 前把 `webui/dist` 移出至 `/tmp/t07-dist-backup`，AC-15 已重建，`webui/dist/index.html` 实测存在（1031 B，02:15 生成） |

---

## 2. 测试策略

| 层级 | 划分与理由 |
|---|---|
| **A 段 · 脚本门禁** | AC-1/2/4/5/6/7/8/32 全部设计为「可脚本判定」，交由 `scripts/check-zh-docs.py gate` 一次性裁决；再用 `inventory` 复跑恒等式，防止 gate 内部计数与清单口径不一致。 |
| **B 段 · 人工机械判定** | AC-3（残留行白名单）、AC-8（三域 grep）、AC-9/10/11（README 三项）在需求中被标为「人工可明确判定（二值）」。做法：用脚本产出**全集**残留行清单，再用**确定性等距抽样**取 20 行（避免人为挑选），逐行对照 `00-requirement.md` §3.2 白名单。 |
| **C 段 · Rust 门禁** | AC-31 的 fmt / clippy / test 三层。`cargo test --workspace --no-fail-fast` 全量跑（跨 crate 与 i18n 公共面均有改动），再用 `-p rustcode-config --lib`、`-p rustcode-daemon --lib channel_mode_tests`、`-p rustcode-daemon` 三个定点套件交叉验证 AC-29/30。 |
| **D 段 · 构建与端到端** | AC-12…AC-18 中每一条都用**隔离手段**验证：伪造 PATH（无 node）、伪造低版本 node、`--if-missing` 前后对比、`git check-ignore`、`ss -ltn` 监听面。AC-24 用 `RUSTCODE_HOME=$(mktemp -d)` 保证零 provider 前置。 |
| **E 段 · 审批回归闸门** | 最高优先级。本轮**成功构造出真实需审批的工具调用**（mock OpenAI 兼容 provider + 破坏性 `bash` 命令），把 AC-27 从「替代证据」升级为端到端实证；并以非破坏性命令做对照组，证明「执行/审批」的差别来自审批门而非一律阻断。 |
| **F 段 · 前端回归** | AC-33 三项（`tsc --noEmit` / `npm test` / `npm run build`）按 `AGENTS.md:332` 口径执行。 |

---

## 3. 用例清单

| 用例 id | 类型 | 覆盖场景（AC 映射） | 文件路径 / 命令 | 结果 |
|---|---|---|---|---|
| T07-A-01 | 脚本门禁 | AC-1/2/4/5/6/7/8/32 | `python3 scripts/check-zh-docs.py gate --base 3ee655e3 --report …/zh-check-gate.md` | PASS（exit 0） |
| T07-A-02 | 脚本门禁 | AC-1 恒等式 | `python3 scripts/check-zh-docs.py inventory --base 3ee655e3` | PASS（`SKIP_A=4 SKIP_B=47 SKIP_C=1 SKIP_D=2 TODO=29 ZH=202 total=285`） |
| T07-A-03 | 脚本门禁（辅助） | AC-3 残留行全集 | `check --base 3ee655e3 --files <231 文件> --report /tmp/ac3-all.md` | PASS（受检 231，FAIL 0；en 残留 43 行） |
| T07-B-01 | 人工抽检 | AC-3 | 43 行等距抽样 20 行 | PASS（20/20 落白名单） |
| T07-B-02 | 机械判定 | AC-8 | `git grep -n -F "README.zh-CN"` 三域计数 | PASS（正式域 0 / 历史域 39 / 兜底域 0） |
| T07-B-03 | 机械判定 | AC-9a/9b/9c | `git ls-files README.zh-CN.md`、`test ! -e`、`git diff --numstat` | PASS |
| T07-B-04 | 脚本门禁 | AC-10 | `check --files README.md` | PASS（`en=0/374=0.0000`） |
| T07-B-05 | 机械判定 | AC-11 | README.md 全量 href + markdown 链接目标 `test -e` | PASS（7/7 存在） |
| T07-C-01 | Rust 门禁 | AC-31 G1 | `cargo fmt --check` | PASS（exit 0，0 行输出） |
| T07-C-02 | Rust 门禁 | AC-31 G3 | `cargo test --workspace --no-fail-fast` | PASS（5487 passed / **1** failed，唯一红测为已文档化项） |
| T07-C-03 | Rust 门禁 | AC-31 / i18n | `cargo test -p rustcode-config --lib` | PASS（327 passed / 0 failed） |
| T07-C-04 | Rust 门禁 | AC-29 | `cargo test -p rustcode-daemon --lib channel_mode_tests` | PASS（4 passed，含 `known_clients_interactive_on_loopback_or_token`） |
| T07-C-05 | Rust 门禁 | AC-29（断言无净减少） | `git diff … lib.rs \| grep -E "^[+-]\s*assert"` | PASS（0 命中） |
| T07-C-06 | Rust 门禁 | AC-30 | `cargo test -p rustcode-daemon` | PASS（316 passed / 0 failed，含 `default_host_lock.rs`） |
| T07-C-07 | Rust 门禁 | AC-31 G2 | `cargo clippy -p rustcode-daemon -p rustcode -p rustcode-config --all-targets` | PASS（exit 0，`^error` 0 行） |
| T07-C-08 | 跨 crate 编译 | —（补充） | `cargo check --workspace --all-targets` | PASS（exit 0，`^error` 0 行） |
| T07-D-01 | 脚本静态 | AC-12 | `bash -n scripts/build-webui.sh`；`test -x`；`webui --help`；`webui --bogus` | PASS（0 / 0 / 0 / 2） |
| T07-D-02 | fail-closed | AC-13 | 无 node 的伪造 PATH 运行 | PASS（exit 2，含 `Node.js >= 22.6`，`webui/dist` 未生成） |
| T07-D-03 | fail-closed | AC-14 | 伪造 `node v20.11.0` | PASS（exit 2，含当前版本与要求版本） |
| T07-D-04 | 构建行为 | AC-15c | `--if-missing` 且 dist 缺失 | PASS（exit 0，构建产出 index.html） |
| T07-D-05 | 构建行为 | AC-15a | 默认重跑（dist 已存在） | PASS（exit 0，覆盖重建） |
| T07-D-06 | 构建行为 | AC-15b | `--if-missing` 且 dist 存在 | PASS（exit 0，打印跳过说明） |
| T07-D-07 | 机械判定 | AC-16a | `git check-ignore -v webui/dist/index.html` + `git status --porcelain webui/dist` | PASS（`.gitignore:88:dist/`；状态 0 行） |
| T07-D-08 | 机械判定 | AC-16b | `grep -rn "npm\|npx" crates/*/build.rs` | PASS（exit 1，0 命中） |
| T07-D-09 | 机械判定 | AC-16c | `git diff 3ee655e3 -- .gitignore` | PASS（0 行） |
| T07-D-10 | 人工比对 | AC-17 | `Msg::CliWebuiNotBuilt` zh/en 双语文案命令行逐条比对 | PASS（4 条命令完全一致） |
| T07-D-11 | 端到端 | AC-18 / AC-24 | `RUSTCODE_HOME=$(mktemp -d) rustcode webui --port 13497` + curl | PASS（进程存活；`/health` 200；`/` 200 含 `<html`） |
| T07-D-12 | 端到端 | AC-19 | `sed -n` 三处常量 + `ss -ltn` | PASS（CLI 两处 `0.0.0.0`；daemon `127.0.0.1` 未动；运行时 `0.0.0.0:13497`） |
| T07-D-13 | 端到端 | AC-20 | `rustcode webui --host 127.0.0.1 --port 13458` + `ss -ltn` | PASS（`127.0.0.1:13458`，`0.0.0.0:13458` 计数 0） |
| T07-D-14 | 端到端 | AC-21 | `rustcode daemon` / `rustcode-daemon` 默认启动输出精确 grep | PASS（0 命中） |
| T07-D-15 | 端到端（本机） | AC-22（部分） | 以 LAN IP `172.24.0.2:13465` curl | 部分验证（`/health` 200、`/` 200；跨设备未验证） |
| T07-E-01 | 端到端（构造） | AC-27 | 独立二进制 `--port 13456 --client vscode` + `X-RustCode-Client: vscode` + Build 模式 `POST /chat` 触发破坏性 `bash` | **PASS**（`permission_request` 事件；工具未执行） |
| T07-E-02 | 对照组 | AC-27（反证） | 同上，非破坏性 `bash echo` | PASS（无需审批的工具正常执行，证明非一律阻断） |
| T07-E-03 | 静态 + 运行时 | AC-28 | Q2-A 补偿三要素核对 | PASS |
| T07-E-04 | 替代证据 | AC-28/29 | `daemon_token_auth.rs::chat_requires_token_health_is_public` | PASS（`/models` 无 token → 401） |
| T07-F-01 | 前端 | AC-33 | `cd webui && npx tsc --noEmit` | PASS（exit 0，0 行输出） |
| T07-F-02 | 前端 | AC-33 | `cd webui && npm test` | PASS（227 pass / 0 fail） |
| T07-F-03 | 前端 | AC-33 | `cd webui && npm run build`（由 AC-15a 覆盖） | PASS（vite build 成功） |
| T07-X-01 | 遗留项复核 | T-13 遗留风险 B | `rustc` 复刻 `is_loopback_authority` + 独立二进制 `--host ::1` 实跑 | 复现确认（见 §5 缺陷 D-1） |

**合计 34 条用例：33 PASS / 0 FAIL / 1 遗留复现（不计入门禁）。**

---

## 4. A/B/C/D/E/F 六段逐段结论与真实输出

### 4.A 段 · 脚本门禁 —— **PASS**

**A-1 `gate`**
```bash
$ cd /workspace/RustCode
$ date -u +"%Y-%m-%dT%H:%M:%SZ"
2026-09-08T18:05:39Z
$ python3 scripts/check-zh-docs.py gate --base 3ee655e3 \
    --report .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-gate.md
gate: 未跟踪 md 已排除：56 个（不进 AC-1 分母，也不受 AC-2/4/6/7b/32 与 AC-8 段 1 约束）
  - .codebuddy/artifacts/2026-09-07-zh-docs-webui/00-decisions.md
  - ... 其余 51 个未列出

gate: base=3ee655e3

PASS AC-1 清单自洽
  SKIP_A=4 SKIP_B=47 SKIP_C=1 SKIP_D=2 TODO=29 ZH=202 total=285
PASS AC-2/3/4/6/7b/32 全量 check
  全量 check 受检 231，FAIL 0
  AC-4 授权放行合计 12 条（D1 11 / D2 1）
PASS AC-5 运行时载荷
  AC-5a rules 目录无改动: OK
  AC-5b setup-seeds 有改动 (6 个文件): OK
  AC-5b 无 ^[+-]name: 变更: OK
PASS AC-7a 无改名/删除
  AC-7a 无 md 删除/重命名（README.zh-CN.md 与 A/C 类除外）: OK
PASS AC-8 外链残留
  段 1 正式域（候选 515，已排除 .codebuddy/artifacts/）：0 命中 -> OK
  段 2 历史域（候选 37）：命中 39 处，全部位于 .codebuddy/artifacts/ -> OK（历史痕迹仍在）
  段 3 兜底域 site/.github/docs/extensions（全扩展名）：0 命中 -> OK

gate: PASS
gate: 报告已写入 .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-gate.md
GATE_EXIT=0
```
报告落盘确认：`1185` 行，`SKIP_A=4 SKIP_B=47 SKIP_C=1 SKIP_D=2 TODO=29 ZH=202 total=285` 在正文首屏。覆盖 **AC-1/2/4/5/6/7/8/32**。

**A-2 `inventory` 复跑恒等式**
```bash
$ python3 scripts/check-zh-docs.py inventory --base 3ee655e3
...
SKIP_A(4) + SKIP_B(47) + SKIP_C(1) + SKIP_D(2) + TODO(29) + ZH(202) = 285 ；全仓 md 总数 = 285 ；恒等式成立
...
inventory: total=285 SKIP_A=4 SKIP_B=47 SKIP_C=1 SKIP_D=2 TODO(EN=0,MIXED=29) ZH=202 恒等式=OK
INV_EXIT=0
```
> **与编排者实测完全一致**（`SKIP_A=4 SKIP_B=47 SKIP_C=1 SKIP_D=2 TODO=29 ZH=202 total=285`）。
> 说明：`TODO=29` 全部是 **MIXED** 桶（`0 < ratio < 0.5`，中英混排且已中文为主），**EN 桶为 0**；这 29 个文件同样受 AC-2 约束（`全量 check 受检 231 = TODO 29 + ZH 202`，FAIL 0）。

### 4.B 段 · 人工机械判定 —— **PASS**

**B-1 AC-3 残留行抽检**
```bash
# 产出 231 个受检文件的残留行全集
$ python3 scripts/check-zh-docs.py check --base 3ee655e3 --report /tmp/ac3-all.md --files <231 文件…>
check: 受检 231，PASS 231，FAIL 0
EXIT=0
# 解析：EN 残留行数 = 43；R5 跳过行 = 56（分别统计，避免混淆）
```
43 > 20，按 `(file, lineno)` 排序后**等距抽样 20 行**（索引 `round(i*42/19)`，确定性、不可人为挑选）：

| # | 文件:行 | 原文 | 判定 | 理由（§3.2 白名单类别） |
|---|---|---|---|---|
| 01 | `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/02-tasks.md:34` | `\| **crate** \| `rustcode-codingplan`、`rustcode-config` \|` | 白名单 | 表格键名 `crate`（frontmatter/元数据键名）+ crate 名内联码 |
| 02 | 同上 `:54` | `\| **crate** \| `rustcode-cli` \|` | 白名单 | 同上 |
| 03 | 同上 `:76` | `\| **files_owned** \| `docs/config.example.toml` \|` | 白名单 | 键名 `files_owned` + 文件路径 |
| 04 | `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/03-impl/T6-docs-atomcode.md:284` | `\| docs/features.md:113-114 \| `G7 …` / `G8 …` \|` | 白名单 | 剥离 inline code 后残留 `docs/features.md:113-114`（**文件路径:行号**） |
| 05 | `…/T7-goals-superpowers.md:370` | `\| `inspector-feedback-1.md:11` \| … \|` | 白名单 | 剥离后残留 `inspector-feedback-1.md:11`（路径:行号） |
| 06 | `…/HANDOFF-codingplan-legacy.md:80` | `\| coding --lib / updater --lib \| 430/0、41/0 \|` | 白名单 | crate 名 `coding`/`updater` + CLI flag `--lib` + 数字/单位 |
| 07 | `crates/rustcode-capabilities/assets/setup-seeds/…/SKILL.md:218` | `  "mcpServers": {` | 白名单 | JSON 键，位于 ```json 围栏内；脚本因**嵌套围栏**未剥离（见缺陷 D-3），内容为代码 |
| 08 | `crates/rustcode-clix/README.md:83` | `**GitHub:**` | 白名单 | 第三方专有名词 GitHub（§3.2 明列） |
| 09 | `crates/rustcode-coding/README.md:1` | `# rustcode-coding (L2)` | 白名单 | crate 名 + 层级编号 |
| 10 | `crates/rustcode-daemon/README.md:600` | `### MCP（Model Context Protocol）` | 白名单 | 协议专有名词 / 类型名 |
| 11 | `crates/rustcode-review/README.md:1` | `# rustcode-review(L2)` | 白名单 | crate 名 |
| 12 | `docs/dev-env-setup.md:26` | `\| Node.js \| v24.20.0 (LTS) \| nvm \| `~/.nvm/` \|` | 白名单 | 产品名 Node.js + 版本号 `v24.20.0` + 命令 `nvm` + 路径 |
| 13 | `docs/plans/2026-08-14-webui-browser-notification-design.md:108` | `\| stop_reason \| title \| body \|` | 白名单 | 表格表头 / 字段名标识符 |
| 14 | `docs/plans/2026-08-17-acp-sdk-2.0-protocol-roadmap.md:37` | `` `session/update`(agent_message_chunk / … ) `` | 白名单 | 剥离后残留事件名/字段标识符（`agent_message_chunk` 等） |
| 15 | 同上 `:100` | `initialize / session/new / session/prompt / session/cancel / session/request_permission` | 白名单 | API 方法 / 路径 |
| 16 | `docs/plans/2026-08-21-external-agent-subagent-drivers-spec.md:96` | ``### 4.1 `SubagentBackend` trait`` | 白名单 | 类型名 + 数字 |
| 17 | 同上 `:200` | `- spawn（`tokio::process::Command`，`kill_on_drop`）。` | 白名单 | 剥离后残留 `spawn`（**函数名**） |
| 18 | `docs/superpowers/specs/2026-07-25-retire-core-provider-B-vision-design.md:16` | `- **daemon** `live_api.rs:1119 …` → `maybe_preprocess`（core）。` | 白名单 | 剥离后残留 `daemon`（子命令 `rustcode daemon`）与 `core`（模块/包名） |
| 19 | `docs/superpowers/specs/2026-07-31-local-scheduled-tasks-design.md:98` | `**DEFER**：` | 白名单 | 状态枚举字面量（与同系列 `DONE`/`ACTIVE` 状态机一致，属「枚举变体 / 配置值字面量」） |
| 20 | `extensions/jetbrains/docs/jetbrains.md:63` | `- Ollama` | 白名单 | 第三方专有名词 Ollama（§3.2 明列） |

**判定：20/20 全部落入 §3.2 白名单 ⇒ AC-3 PASS。**
补充事实：20 行中 **14 行位于本 feature 未改动文件**（存量中文文档，非本轮汉化产物），6 行位于本轮已改动文件（`T7-goals-superpowers.md`、`HANDOFF-codingplan-legacy.md`、`SKILL.md`、`rustcode-coding/README.md`、`rustcode-review/README.md`、`jetbrains.md`）。两种来源均按同一判据通过。

**B-2 AC-8 三域计数**
```bash
$ git grep -n -F "README.zh-CN" | wc -l
39
$ git grep -n -F "README.zh-CN" -- . ':(exclude).codebuddy/artifacts' | wc -l
0                                          # 正式域 —— 必须 0 ✔
$ git grep -n -F "README.zh-CN" -- .codebuddy/artifacts | wc -l
39                                         # 历史域（本 feature 过程文档，记录删除决策）
$ git grep -n -F "README.zh-CN" -- site .github docs extensions | wc -l
0                                          # 兜底域（全扩展名）—— 必须 0 ✔
```
**判定：正式域 0 + 兜底域 0 ⇒ AC-8 PASS（按编排者三域口径）。** 与 gate 内部计数一致（段 2：39 处）。
> 口径偏差见 §5 缺陷 D-2：AC-8 在 `00-requirement.md` 的**字面**表述（全仓 `grep` 0 命中）**不成立**（39 处命中全在 `.codebuddy/artifacts/**`）。本轮按编排者下发的三域口径判定。

**B-3 AC-9 / AC-10 / AC-11**
```bash
=== AC-9a: git ls-files README.zh-CN.md ===
ls_files_exit=0 / 输出行数=0                                  ✔
=== AC-9b: test ! -e README.zh-CN.md ===
TRUE(文件不存在) exit=0                                        ✔
=== AC-9c: README.md 相对基线非空变更 ===
416	458	README.md
-rw-r--r-- 1 root root 39779 Sep  8 22:44 README.md
$ git status --porcelain -- README.zh-CN.md README.md
 M README.md
D  README.zh-CN.md                                             ✔（索引内已删除）

=== AC-10 ===
$ python3 scripts/check-zh-docs.py check --base 3ee655e3 --files README.md
PASS README.md en=0/374=0.0000
  [AC-4-authorized] README.md | removed=- | added=./scripts/build-webui.sh | cause=D2
check: 受检 1，PASS 1，FAIL 0                                   ✔（0.0000 ≤ 0.05）

=== AC-11 ===
总链接: 18
外链/页内锚点/邮箱: 9
需 test -e 的相对目标（去重）:
  OK   [mdlink] docs/ORIGINAL_LICENSE.md
  OK   [mdlink] docs/UPSTREAM_CREDITS.md
  OK   [mdlink] ./docs/security/permission-model.md
  OK   [mdlink] docs/telemetry.md
  OK   [mdlink] docs/config.example.toml
  OK   [mdlink] site/docs/en/index.html
  OK   [mdlink] LICENSE
去重相对目标 7：存在 7，缺失 0                                   ✔
$ grep -n -E "README\.zh-CN|English|简体中文" README.md
（仅命中第 42 行正文「**默认简体中文**」的产品说明，非链接、非中英切换导航）
```
**判定：AC-9 / AC-10 / AC-11 全部 PASS。**

### 4.C 段 · Rust 门禁（AC-31 / AC-29 / AC-30）—— **PASS**

**C-1 `cargo fmt --check`**
```bash
$ cargo fmt --check > /tmp/fmt.log 2>&1; echo "FMT_EXIT=$?"
FMT_EXIT=0
$ wc -l < /tmp/fmt.log
0
```

**C-2 `cargo test --workspace --no-fail-fast`**（后台 18:12:24Z 启动，日志末次写入 18:17:11Z，约 4 分 47 秒）
```bash
$ grep -oP "test result: (ok|FAILED)\. \K[0-9]+ passed; [0-9]+ failed" /tmp/cargo-test-ws.log \
    | awk -F'[ ;]' '{p+=$1; f+=$4} END{print "TOTAL passed="p" failed="f}'
TOTAL passed=5487 failed=1
$ grep -n "FAILED$" /tmp/cargo-test-ws.log
706:test mcp::registry::tests::trust_key_golden_matches_core_algorithm ... FAILED
$ sed -n '1910,1926p' /tmp/cargo-test-ws.log
failures:
---- mcp::registry::tests::trust_key_golden_matches_core_algorithm stdout ----
thread 'mcp::registry::tests::trust_key_golden_matches_core_algorithm' (288549) panicked at crates/rustcode-capabilities/src/mcp/registry.rs:1507:9:
assertion `left == right` failed
  left: "e07a86b0ce8a1c59"
 right: "8b6a67e0b2c06dae"
test result: FAILED. 1475 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 25.67s
...
error: 1 target failed:
    `-p rustcode-capabilities --lib`
$ grep -c "test result:" /tmp/cargo-test-ws.log
92
```
**判定：`5487 passed / 1 failed`，唯一红测 = `AGENTS.md:226` 已文档化的 `mcp::registry::tests::trust_key_golden_matches_core_algorithm`。**
归属判定：**存量，非本 feature 引入** —— 该文件 `crates/rustcode-capabilities/src/mcp/registry.rs` 不在本 feature 的 169 项改动内（`git status --porcelain` 未列出），且本 feature 未触碰 `rustcode-capabilities` 的 Rust 代码（仅改动其 `assets/setup-seeds/**` 的 6 个 md）。

**C-3 `cargo test -p rustcode-config --lib`**
```bash
$ cargo test -p rustcode-config --lib; echo "CFG_EXIT=$?"
CFG_EXIT=0
test result: ok. 327 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
$ grep -c "^test i18n" /tmp/t-cfg.log
52
test i18n::tests::authoritative_set_brand_overrides_pre_scan ... ok
test i18n::tests::cli_flag_wins_over_everything ... ok
test i18n::tests::current_locale_fallback_is_zh_cn ... ok
test i18n::tests::env_explicit_english_resolves_to_en ... ok
```
i18n 内容测试全覆盖且通过（含 T-04/T-05 的 `en.rs` / `zh_cn.rs` 文案与 `messages.rs` 新增注释）。

**C-4 `cargo test -p rustcode-daemon --lib channel_mode_tests`（AC-29）**
```bash
$ cargo test -p rustcode-daemon --lib channel_mode_tests; echo "CM_EXIT=$?"
CM_EXIT=0
test channel_mode_tests::interactive_responder_is_required_for_prompt_required_modes ... ok
test channel_mode_tests::known_clients_interactive_on_loopback_or_token ... ok
test channel_mode_tests::resolve_channel_header ... ok
test channel_mode_tests::request_approval_mode_overrides_global_mode ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 303 filtered out; finished in 0.00s
```

**C-5 AC-29 断言无净减少**
```bash
$ git diff 3ee655e3 -- crates/rustcode-daemon/src/lib.rs > /tmp/lib.diff
$ wc -l < /tmp/lib.diff
76
$ grep -c -E "^[+-][[:space:]]*assert" /tmp/lib.diff
0                                            # 无任何 assert 行的增删 ⇒ 无净减少 ✔
# 交叉验证：按 hunk 头定位，测试函数区间（工作区行 8866..8905）内 +/- 行数 = +0 / -0
```
补充：`lib.rs` 相对基线的全部 `+` 行为 27 行，内容是 3 处文档注释 + 1 处新增非回环提示分支，**无任何生产控制流/判定函数改动**。

**C-6 `cargo test -p rustcode-daemon`（AC-30）**
```bash
$ cargo test -p rustcode-daemon; echo "DAEMON_EXIT=$?"
DAEMON_EXIT=0
     Running unittests src/lib.rs (target/debug/deps/rustcode_daemon-c00f6f792f2e5f83)
test result: ok. 307 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.59s
     Running unittests src/main.rs
test result: ok. 0 passed; 0 failed; ...
     Running tests/daemon_token_auth.rs (target/debug/deps/daemon_token_auth-...)
test chat_requires_token_health_is_public ... ok
test result: ok. 1 passed; 0 failed; ... finished in 8.62s
     Running tests/default_host_lock.rs
test standalone_daemon_default_host_stays_loopback ... ok
test result: ok. 1 passed; 0 failed; ...
     Running tests/legacy_turn_boundary_repair.rs
test result: ok. 7 passed; 0 failed; ...
   Doc-tests rustcode_daemon
test result: ok. 0 passed; 0 failed; ...
# 合计 316 passed / 0 failed
```
**AC-30 PASS**：`tests/default_host_lock.rs` 的 `standalone_daemon_default_host_stays_loopback` 在场且通过（**未重复新增**）。

**C-7 `cargo clippy`（AC-31 G2）**
```bash
$ cargo clippy -p rustcode-daemon -p rustcode -p rustcode-config --all-targets; echo "CLIPPY_EXIT=$?"
CLIPPY_EXIT=0
$ grep -c "^error" /tmp/clippy2.log
0
$ grep "^warning: .rustcode" /tmp/clippy.log
warning: `rustcode-config` (lib) generated 3 warnings
warning: `rustcode-capabilities` (lib) generated 11 warnings
warning: `rustcode-config` (lib test) generated 4 warnings (2 duplicates)
warning: `rustcode-daemon` (lib) generated 6 warnings
warning: `rustcode-daemon` (lib test) generated 14 warnings (6 duplicates)
warning: `rustcode-tuix` (lib) generated 8 warnings
warning: `rustcode` (lib) generated 1 warning
warning: `rustcode` (lib test) generated 1 warning (1 duplicate)
$ grep -n "rustcode-daemon/src/lib.rs\|i18n/messages.rs\|default_host_lock.rs" /tmp/clippy.log
210:    --> crates/rustcode-daemon/src/lib.rs:4785:1
333:    --> crates/rustcode-daemon/src/lib.rs:4521:1
```
`rustcode-daemon/src/lib.rs` 仅被点名 2 处（`:4785` empty line after doc comment、`:4521` too many arguments），**均远离**本轮改动区（5275 / 6035 / 6332-6374）；`messages.rs`、`default_host_lock.rs` 零告警。⇒ **无新增告警**。
> 范围声明：按派单许可只跑 3 个 crate（T-13 已跑通并本轮复跑确认 exit 0）；未跑 `clippy --workspace --all-targets`（耗时过长），改用 C-8 的 workspace 级 `check` 作为跨 crate 补充。

**C-8 `cargo check --workspace --all-targets`**
```bash
$ cargo check --workspace --all-targets; echo "CHECK_WS_EXIT=$?"
CHECK_WS_EXIT=0
$ grep -c "^error" /tmp/check-ws.log
0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 13.05s
```

### 4.D 段 · WebUI 构建与端到端（AC-12…AC-18、AC-24）—— **PASS（AC-22/23 见 §5/§6）**

**D-1 AC-12**
```bash
$ bash -n scripts/build-webui.sh; echo bash_n_exit=$?
bash_n_exit=0
$ test -x scripts/build-webui.sh; echo test_x_exit=$?
test_x_exit=0
-rwxr-xr-x 1 root root 9140 Sep  7 16:50 scripts/build-webui.sh

$ ./target/debug/rustcode webui --help
Start the local browser webui

Usage: rustcode webui [OPTIONS]

Options:
      --port <PORT>  Port (default: 13457) [default: 13457]
      --host <HOST>  Bind address (default: 0.0.0.0; use 127.0.0.1 for local-only) [default: 0.0.0.0]
  -h, --help         Print help
HELP_EXIT=0

$ ./target/debug/rustcode webui --bogus >/dev/null 2>&1; echo BADARG_EXIT=$?
BADARG_EXIT=2
```

**D-2 AC-13（无 node，fail-closed）**
```bash
$ mv webui/dist /tmp/t07-dist-backup                  # 先移走 dist，使"不产生半成品"可判定
$ mkdir -p /tmp/t07-nonode/bin                        # 仅含 awk/basename/env/bash… 等，无 node/npm
$ PATH=/tmp/t07-nonode/bin bash scripts/build-webui.sh; echo AC13_EXIT=$?
AC13_EXIT=2
--- stderr ---
[ERROR] 前置检查失败：PATH 中未找到可执行的 node。

[INFO] 修复指引：安装 Node.js >= 22.6 后重试（要求来自 /workspace/RustCode/webui/package.json 的 engines.node）。
  - nvm（推荐，不需要 sudo）：
      nvm install 22.6 && nvm use 22.6
  - 官方安装包： https://nodejs.org/en/download
  ...
$ test ! -e webui/dist/index.html; echo test_not_e_exit=$?
test_not_e_exit=0                                     # webui/dist 目录根本未创建 ✔
$ grep -A3 '"engines"' webui/package.json
  "engines": {
    "node": ">=22.6"
  },
```
exit=2 ≠ 0；指引含 `Node.js >= 22.6`，与 `engines.node` 一致；无半成品。**PASS**。

**D-3 AC-14（node 版本不足，fail-closed）**
```bash
$ printf '#!/bin/sh\nif [ "$1" = "--version" ]; then echo "v20.11.0"; exit 0; fi\nexec /usr/bin/node "$@"\n' > /tmp/t07-oldnode/bin/node
$ PATH=/tmp/t07-oldnode/bin bash scripts/build-webui.sh; echo AC14_EXIT=$?
AC14_EXIT=2
--- stderr ---
[ERROR] 前置检查失败：node 版本不足。当前 = v20.11.0，要求 >= 22.6（/workspace/RustCode/webui/package.json engines.node）。
[ERROR] 版本不足不降级为警告，构建终止。
...
$ test ! -e webui/dist/index.html; echo $?
0
```
exit=2；同时打印当前版本 `v20.11.0` 与要求 `>= 22.6`；**未降级为警告后继续**。**PASS**。

**D-4/D-5/D-6 AC-15**
```bash
# (c) dist 缺失 + --if-missing -> 正常构建
$ ./scripts/build-webui.sh --if-missing
[INFO] 模式       : if-missing
[INFO] node       : /root/.nvm/versions/node/v22.13.1/bin/node (v22.13.1)
[INFO] 执行 npm ci（依赖以 package-lock.json 为准，会重建 node_modules）
added 158 packages, and audited 160 packages in 6s
[INFO] 执行 npm run build（vite build）
vite v8.0.16 building client environment for production...
✓ 55 modules transformed.
dist/index.html                                                      1.03 kB │ gzip:  0.61 kB
[SUCCESS] 前端构建完成：/workspace/RustCode/webui/dist/index.html
$ ls -la webui/dist/index.html
-rw-r--r-- 1 root root 1031 Sep  9 02:14 webui/dist/index.html        ✔ AC-15c

# (b) dist 存在 + --if-missing -> 跳过
$ ./scripts/build-webui.sh --if-missing; echo AC15B_EXIT=$?
[INFO] --if-missing：/workspace/RustCode/webui/dist/index.html 已存在，跳过构建。
[INFO] 如需强制重跑，去掉 --if-missing 直接运行本脚本。
[INFO] 提示：继续 cargo 构建前请执行 cargo clean -p rustcode-daemon（cargo 不追踪 webui/dist/）。
AC15B_EXIT=0                                                          ✔ AC-15b

# (a) 默认重跑（dist 已存在 -> 覆盖重建）
$ ./scripts/build-webui.sh
[INFO] 执行 npm ci（…） / [INFO] 执行 npm run build（vite build） / [SUCCESS] 前端构建完成
$ ls -la webui/dist/index.html
-rw-r--r-- 1 root root 1031 Sep  9 02:15 webui/dist/index.html        ✔ AC-15a（时间戳刷新，幂等覆盖）
```

**D-7/D-8/D-9 AC-16**
```bash
$ git check-ignore -v webui/dist/index.html; echo check_ignore_exit=$?
.gitignore:88:dist/	webui/dist/index.html
check_ignore_exit=0                                   ✔ AC-16a（被忽略）
$ git status --porcelain webui/dist; echo status_lines=$(git status --porcelain webui/dist | wc -l)
status_lines=0                                        ✔ AC-16a（不入库）

$ ls crates/*/build.rs
crates/rustcode-capabilities/build.rs
crates/rustcode-cli/build.rs
$ grep -rn "npm\|npx" crates/*/build.rs; echo grep_exit=$?
grep_exit=1                                           ✔ AC-16b（0 命中）

$ git diff 3ee655e3 -- .gitignore; echo diff_lines=$(git diff 3ee655e3 -- .gitignore | wc -l)
diff_lines=0                                          ✔ AC-16c
```

**D-10 AC-17（`Msg::CliWebuiNotBuilt` 双语文案比对）**
```rust
// crates/rustcode-config/src/i18n/zh_cn.rs:1135-1136
Msg::CliWebuiNotBuilt =>
    "本二进制未内嵌 webui 资源。\n请先构建前端，再重新构建：\n\n   ./scripts/build-webui.sh\n   cargo clean -p rustcode-daemon\n   cargo build -p rustcode\n\n或手工执行等价步骤：\n   cd webui && npm ci && npm run build\n   cargo clean -p rustcode-daemon\n   cargo build -p rustcode\n".into(),

// crates/rustcode-config/src/i18n/en.rs:1191-1192
Msg::CliWebuiNotBuilt =>
    "webui assets are not embedded in this binary.\nBuild the frontend first, then rebuild:\n\n   ./scripts/build-webui.sh\n   cargo clean -p rustcode-daemon\n   cargo build -p rustcode\n\nOr run the equivalent steps manually:\n   cd webui && npm ci && npm run build\n   cargo clean -p rustcode-daemon\n   cargo build -p rustcode\n".into(),
```
| 判据 | 结果 |
|---|---|
| (a) 指向新脚本 `scripts/build-webui.sh` | ✔ 两语种均含 |
| (b) 含 `cargo clean -p rustcode-daemon`（修正 N2） | ✔ 两语种均含（各出现 2 次） |
| (c) 与 `AGENTS.md:16` 口径一致（`npm ci`） | ✔ 两语种均写 `npm ci`，非 `npm install` |
| (d) 中英命令行逐条相等 | ✔ `./scripts/build-webui.sh` / `cargo clean -p rustcode-daemon` / `cargo build -p rustcode` / `cd webui && npm ci && npm run build` 四条**逐字一致** |

**D-11 AC-18 / AC-24（无 provider 端到端）**
```bash
$ RUSTCODE_HOME=$(mktemp -d) ./target/debug/rustcode webui --port 13497
Opened webui in your browser: http://172.24.0.2:13497/?token=f479b72705c34543b8b03d7ac2b66b44
[!] Primary address is a LAN IP; only devices on the same network can reach it. For public access use a tunnel (e.g. cloudflared / Tailscale). There is no TLS, so anyone who can reach it can get in with the token.

$ ss -ltn | grep 13497
LISTEN 0      4096         0.0.0.0:13497      0.0.0.0:*

$ curl -sS -w "http_code=%{http_code}\n" http://127.0.0.1:13497/health
http_code=200
{"status":"ok","version":"5.0.9","service":"rustcode-daemon", …}

$ curl -sS -o /tmp/index.html -w "http_code=%{http_code}\n" http://127.0.0.1:13497/
http_code=200
$ grep -c '<html' /tmp/index.html
1
$ head -c 120 /tmp/index.html
<!doctype html>
<html lang="zh">
  <head>
    <meta charset="utf-8" />

$ kill -0 <pid> && echo ALIVE
ALIVE (未退出，AC-24 满足)
```
**AC-18 PASS**（`/health` 200、`/` 200 且含 `<html`）；**AC-24 PASS**（`RUSTCODE_HOME` 指向空目录、零 `[[providers]]`，进程不因 provider 缺失退出）。
附带证据：首页 `<html lang="zh">`，与 fork 默认中文一致。

**D-12 AC-19（三处默认 host）**
```bash
$ sed -n '18,24p' crates/rustcode-daemon/src/main.rs
fn parse_daemon_args() -> (String, u16, u64, ClientMode) {
    const DEFAULT_HOST: &str = "127.0.0.1";       # ← Q2-A 红线：必须保持 127.0.0.1 ✔

$ sed -n '1045,1048p' crates/rustcode-cli/src/main.rs
        #[arg(long, default_value = "0.0.0.0")]   # ← webui --host 默认 ✔
        host: String,

$ sed -n '1779,1781p' crates/rustcode-cli/src/main.rs
                let res = rustcode_daemon::run_server(rustcode_daemon::ServerOpts {
                    host: "0.0.0.0".to_string(),  # ← daemon 子命令 ✔
```
运行时旁证：`ss -ltn` → `0.0.0.0:13497`；`--help` → `default: 0.0.0.0`。**PASS**。

**D-13 AC-20（显式 `--host` 覆盖）**
```bash
$ RUSTCODE_HOME=$(mktemp -d) ./target/debug/rustcode webui --host 127.0.0.1 --port 13458
Opened webui in your browser: http://127.0.0.1:13458/?token=4d0c4ea5c393430394c3c358f28a21ac
$ ss -ltn | grep 13458
LISTEN 0      4096       127.0.0.1:13458      0.0.0.0:*
$ ss -ltn | grep -c "0.0.0.0:13458"
0                                             # 无通配监听 ✔
```
（回环绑定下未打印 LAN/非回环提示，与 Q3-A「回环静默」一致。）**PASS**。

**D-14 AC-21（非回环警告不再打印）**
```bash
# 生产调用点：仅剩注释引用，无 emit
$ grep -rn "DaemonWarnNonLoopback" crates/ | grep -v "i18n/"
crates/rustcode-daemon/src/lib.rs:6339:    // no TLS. The old loopback-only startup banner (`Msg::DaemonWarnNonLoopback`)

# 运行时（rustcode daemon，默认参数，绑 0.0.0.0）
$ RUSTCODE_HOME=$(mktemp -d) ./target/debug/rustcode daemon --port 13461
RustCode API server listening on http://0.0.0.0:13461

[!] Primary address is a LAN IP; only devices on the same network can reach it. …
$ grep -c -E "正在绑定到非回环地址|守护进程暴露了敏感端点|binding to non-loopback address|sensitive endpoints" /tmp/daemon-cli.log
0                                             ✔ 精确判据 0 命中
$ grep -n -E "非回环|non-loopback" /tmp/daemon-cli.log
（无输出）                                     ✔ 宽松判据亦 0 命中

# 运行时（独立二进制 rustcode-daemon，默认参数）
$ RUSTCODE_HOME=$(mktemp -d) ./target/debug/rustcode-daemon --port 13462
RustCode API server listening on http://127.0.0.1:13462
（无任何告警行）
```
**判定依据（重要）**：AC-21 只约束「不再打印 `Msg::DaemonWarnNonLoopback` 的文案」。该变体文案为
zh `警告：正在绑定到非回环地址 '{host}'。守护进程暴露了敏感端点（聊天、文件编辑、工具执行）。` /
en `Warning: binding to non-loopback address '{host}'. The daemon exposes sensitive endpoints …`。
R1 返工新增的 `Msg::WebuiLanWarning`（`[!] Primary address is a LAN IP; …`）**不是该变体**，因此：
- 上面出现的 LAN 提示**不判为回归**；
- 本环境 `LANG=en_US.UTF-8`，`WebuiLanWarning` 英文文案不含 `non-loopback`，故宽松 grep 也是 0。
若在 zh-CN 语区复跑，`Msg::WebuiNonLoopbackWarning` 的中文文案含「非回环」字样，宽松 grep 会命中 —— **届时必须用精确判据**，否则会误判为 AC-21 回归。
**PASS**。

**D-15 AC-22（本机 LAN IP 侧，部分验证）**
```bash
$ RUSTCODE_HOME=/tmp/t07-whome ./target/debug/rustcode webui --port 13465
Opened webui in your browser: http://172.24.0.2:13465/?token=6b9a845b873240d8881402867b400b0f
$ ss -ltn | grep 13465
LISTEN 0      4096         0.0.0.0:13465      0.0.0.0:*
$ curl -sS -o /dev/null -w "LAN /health http=%{http_code}\n" http://172.24.0.2:13465/health
LAN /health http=200
$ curl -sS -o /tmp/lan-index.html -w "LAN / http=%{http_code}\n" http://172.24.0.2:13465/
LAN / http=200
$ grep -c '<html' /tmp/lan-index.html
1
```
已验证「绑定面含 LAN IP 且非回环地址可达」这一服务端必要条件；**跨设备（另一台主机）未验证**，见 §6。

### 4.E 段 · 审批回归闸门（AC-27/28/29/30）—— **PASS**

**E-0 环境构造（关键：本轮成功构造出真实需审批的工具调用）**
```
1) 启动独立二进制（模拟 VS Code 拉起，不传 --host）：
   RUSTCODE_HOME=/tmp/t07-ehome ./target/debug/rustcode-daemon --port 13456 --client vscode
   -> RustCode API server listening on http://127.0.0.1:13456
   -> ss: LISTEN 127.0.0.1:13456
2) 读一次性 token：/tmp/t07-ehome/daemon-13456.json
   {"pid":353331,"port":13456,"token":"671390bd37a14e1ab72f57c9c8bee9c4"}
3) 无 token 请求：POST /chat -H 'X-RustCode-Client: vscode'  -> 401（AC-28 替代证据）
4) 配置 mock OpenAI 兼容 provider（127.0.0.1:18903/18904），其 /v1/chat/completions
   返回 OpenAI SSE 流，第一个 delta 携带 tool_calls[0].function.name = "bash"。
5) 带 token + X-RustCode-Client: vscode 在 Build 模式发 POST /chat。
```

**E-1 AC-27 对照组（非破坏性命令 → 正常执行，证明非一律阻断）**
```bash
$ curl -N -X POST http://127.0.0.1:13456/chat -H 'X-RustCode-Client: vscode' \
    -H 'Authorization: Bearer …' -d '{"message":"run: echo T07_APPROVAL_PROBE","approval_mode":"build","provider":"mock"}'
data: {"type":"runtime_info","provider":"mock","model":"mock-model"}
data: {"type":"session_assigned","session_id":"2be42f29-…"}
data: {"type":"tool_start","id":"call_t07","name":"bash","arguments":"{\"command\":\"echo T07_APPROVAL_PROBE\"}"}
data: {"type":"tool_result","id":"call_t07","name":"bash","output":"T07_APPROVAL_PROBE\n","success":true,"duration_ms":25}
data: {"type":"text","content":"done"}
data: {"type":"done", … }
```

**E-2 AC-27 正式用例（破坏性命令 → 必须走审批）**
```bash
$ mkdir -p /tmp/t07_approval_probe_dir && echo probe > /tmp/t07_approval_probe_dir/f.txt
$ echo 'rm -rf /tmp/t07_approval_probe_dir' > /tmp/mk_cmd.txt      # 危险命令（递归强制删除）
$ curl -N -m 60 -X POST http://127.0.0.1:13456/chat -H 'X-RustCode-Client: vscode' \
    -H 'Authorization: Bearer …' -d '{"message":"delete the probe dir","approval_mode":"build","provider":"mock"}'
data: {"type":"runtime_info","provider":"mock","model":"mock-model"}
data: {"type":"session_assigned","session_id":"08bcd074-d129-4cd4-9620-ba7bdfd7ca92"}
data: {"type":"tokens","prompt":10,"completion":5,"total":15}
data: {"type":"permission_request","session_id":"08bcd074-…","tool_name":"bash","reason":"Requires approval","call_id":"call_t07","arguments":"{\"command\": \"rm -rf /tmp/t07_approval_probe_dir\"}"}
: ping
: ping
: ping
（curl 60s 超时退出：流保持打开等待审批 —— 正是"工具未自动执行"的表现）

$ ls -la /tmp/t07_approval_probe_dir/
-rw-r--r-- 1 root root 6 Sep  9 02:33 f.txt          # 目录与文件仍在 ⇒ 工具未被执行 ✔
$ cat /tmp/t07_approval_probe_dir/f.txt
probe
$ cat /tmp/mk4.log
REQ#1 has_tools=True roles=['system', 'user']        # 无第二轮（无 role=tool 请求）⇒ 未走到工具执行
```
**判定：AC-27 PASS。**
- 事件流中出现 `permission_request`（`tool_name=bash`、`reason="Requires approval"`）⇒ 走交互审批路径（AC-27 选项 a）；
- 目标目录/文件**未被删除**、mock **未收到**第二轮（role=tool）请求 ⇒ **工具未在未获批准的情况下被自动执行**；
- 红线（「`dangerously_skip_permissions=true` 且无审批事件」）**未触发**；
- 对照组证明：无需审批的工具仍可正常执行，故上述阻断来自审批门，而非把 /chat 一律拒绝。

**E-3 AC-28（补偿存在且覆盖 4 个 ClientMode）**
Q2-A 的补偿形态 = **「该入口不跟随默认 host」+「该路径 token 保护」**，实测三要素均在位：

| 要素 | 证据 | 结果 |
|---|---|---|
| 独立二进制 `DEFAULT_HOST` 保持 `127.0.0.1` | `crates/rustcode-daemon/src/main.rs:21` `const DEFAULT_HOST: &str = "127.0.0.1";`；运行时 `ss` → `127.0.0.1:13456` | ✔ |
| 该路径 `enforce_token = true` | `crates/rustcode-daemon/src/main.rs:177` `webui_tokens: Some(token_store),` → `lib.rs:6154` `enforce_token: webui_tokens.is_some(),`；实测无 token `POST /chat` → **401** | ✔ |
| `client_interactive_permission` 覆盖 4 个 ClientMode | `lib.rs:1284-1294`：`enforce_token \|\| (matches!(client_mode, Channel \| Webui \| Vscode \| Jetbrains) && is_loopback_authority(bind_host))`；`enforce_token=true` 时**对 4 个 Mode 一律短路为 true** | ✔ |
| 行为闭环（Build 模式不放行） | `lib.rs:4671-4674` `dangerously_skip_permissions = Auto \|\| (Build && !interactive_permission)`；E-2 实测 `permission_request` 出现 | ✔ |

**E-4 替代证据（即使 E-2 不可复现也成立）**
- `tests/daemon_token_auth.rs::chat_requires_token_health_is_public` → `ok`（C-6）：断言 `/health` 200、`/models` 无 token **401**、带 token 非 401；
- `channel_mode_tests::known_clients_interactive_on_loopback_or_token` → `ok`（C-4）：断言 `client_interactive_permission(Vscode, false, "127.0.0.1") == true` 与 `(Channel, false, "0.0.0.0") == false`；
- `git diff 3ee655e3 -- crates/rustcode-daemon/src/lib.rs` 中 `client_interactive_permission`（`lib.rs:1284-1294`）与其唯一消费点 `lib.rs:4327-4328`、`4671-4674` **零改动**（diff 仅 3 处注释 + 1 处提示分支）。

**E-5 AC-29 / AC-30** —— 见 C-4 / C-5 / C-6，均 **PASS**。

### 4.F 段 · 前端回归（AC-33）—— **PASS**

```bash
$ cd webui && npx tsc --noEmit; echo TSC_EXIT=$?
TSC_EXIT=0
$ wc -l < /tmp/tsc.log
0                                             # 0 错 ✔

$ cd webui && npm test; echo NPM_TEST_EXIT=$?
NPM_TEST_EXIT=0
...
1..227
# tests 227
# suites 0
# pass 227
# fail 0
# cancelled 0
# skipped 0
# todo 0
# duration_ms 2141.622275                     ✔ 227 pass / 0 fail

$ npm run build                               # 由 AC-15a 覆盖（vite build 成功）
vite v8.0.16 building client environment for production...
✓ 55 modules transformed.
[SUCCESS] 前端构建完成：/workspace/RustCode/webui/dist/index.html   ✔
```
> `webui/package.json` 的 `test` 脚本**存在**（`node --experimental-strip-types --test "src/**/*.test.ts"`），源码下有 30 个 `*.test.ts`，故 `npm test` **适用**，不属「不适用/未验证」。

---

## 5. AC-1 … AC-33 逐条对照表

| AC | 结论 | 证据（命令 / 文件:行） | 备注 |
|---|---|---|---|
| AC-1 跳过清单准确 | **PASS** | `gate`：`SKIP_A=4 SKIP_B=47 SKIP_C=1 SKIP_D=2 TODO=29 ZH=202 total=285`；`inventory` 恒等式 OK | 与编排者实测完全一致 |
| AC-2 英文残留率 ≤5% | **PASS** | `gate`：`全量 check 受检 231，FAIL 0`；`README.md en=0/374=0.0000` | TODO=29 全为 MIXED，EN 桶为 0 |
| AC-3 残留白名单 | **PASS** | 43 行残留 → 等距抽样 20 行，20/20 落白名单（§4.B B-1 表） | 14 行位于未改动文件（存量），6 行位于本轮改动文件 |
| AC-4 标识符零损坏 | **PASS** | `gate`：`AC-4 授权放行合计 12 条（D1 11 / D2 1）`，无未授权差异 | D1=README.zh-CN 引用移除；D2=新增 build-webui.sh |
| AC-5(a) rules 无改动 | **PASS** | `gate`：`AC-5a rules 目录无改动: OK` | Q1-A |
| AC-5(b) setup-seeds 非空且不动 name | **PASS** | `gate`：`AC-5b setup-seeds 有改动 (6 个文件): OK` + `AC-5b 无 ^[+-]name: 变更: OK` | Q5-A 已汉化 `description`（见 `SKILL.md` check 报告 `AC-6-desc-changed`） |
| AC-6 frontmatter 完整性 | **PASS** | `gate` 覆盖（231 文件 AC-6 0 FAIL）；`SKILL.md` 的 `description` 变更已逐条记录 | |
| AC-7(a) 无改名/删除 | **PASS** | `gate`：`AC-7a 无 md 删除/重命名（README.zh-CN.md 与 A/C 类除外）: OK` | |
| AC-7(b) 链接目标零变更 | **PASS** | `gate` 内 AC-7b 0 FAIL（231 文件） | |
| AC-8 外部引用同步 | **PASS**（三域口径） | 正式域 0 / 历史域 39（全在 `.codebuddy/artifacts/`）/ 兜底域 0 | 字面口径偏差见 **D-2** |
| AC-9(a) `git ls-files README.zh-CN.md` 空 | **PASS** | 输出行数 = 0 | |
| AC-9(b) `test ! -e README.zh-CN.md` | **PASS** | `TRUE(文件不存在) exit=0` | |
| AC-9(c) README.md 非空变更 | **PASS** | `git diff --numstat` → `416 458 README.md` | |
| AC-10 README 已汉化 | **PASS** | `PASS README.md en=0/374=0.0000` | |
| AC-11 导航链接清理 | **PASS** | 7 个相对链接目标全部 `test -e` 通过；无 `README.zh-CN` 链接；无中英切换导航 | 仅 42 行正文出现「默认简体中文」字样（产品说明，非链接） |
| AC-12 脚本存在且可执行 | **PASS** | `bash -n`=0；`test -x`=0；`webui --help`=0；`webui --bogus`=2 | |
| AC-13 无 node fail-closed | **PASS** | exit **2**；stderr 含 `Node.js >= 22.6`；`test ! -e webui/dist/index.html` = 0 | 与 `engines.node`（`>=22.6`）一致 |
| AC-14 node 版本不足 fail-closed | **PASS** | exit **2**；含 `当前 = v20.11.0` 与 `要求 >= 22.6`；明确「不降级为警告」 | |
| AC-15(a) 默认重跑 | **PASS** | exit 0；`webui/dist/index.html` 1031 B @ 02:15（覆盖重建） | |
| AC-15(b) `--if-missing` + dist 存在 | **PASS** | exit 0；打印「已存在，跳过构建」 | |
| AC-15(c) `--if-missing` + dist 缺失 | **PASS** | exit 0；正常构建并产出 index.html | |
| AC-16(a) dist 被忽略/不入库 | **PASS** | `.gitignore:88:dist/`；`git status --porcelain webui/dist` = 0 行 | |
| AC-16(b) build.rs 无 npm | **PASS** | `grep -rn "npm\|npx" crates/*/build.rs` exit=1（0 命中） | 仅 2 个 build.rs |
| AC-16(c) .gitignore 未改 | **PASS** | `git diff 3ee655e3 -- .gitignore` = 0 行 | |
| AC-17 提示文案可执行 | **PASS** | zh_cn.rs:1135-1136 vs en.rs:1191-1192，4 条命令逐字一致，含 `cargo clean -p rustcode-daemon` 与 `npm ci` | (a)(b)(c)(d) 全满足 |
| AC-18 端到端 | **PASS** | `/health` 200；`/` 200 且 `grep -c '<html'` = 1 | 端口 13497（派单指定） |
| AC-19 默认值已改 | **PASS** | `cli/src/main.rs:1047` `default_value = "0.0.0.0"`；`:1780` `host: "0.0.0.0"`；`daemon/src/main.rs:21` 保持 `127.0.0.1`（Q2-A） | 运行时 `0.0.0.0:13497` |
| AC-20 显式 `--host` 覆盖 | **PASS** | `ss` → `127.0.0.1:13458`，`0.0.0.0:13458` 计数 0 | |
| AC-21 非回环警告不再打印 | **PASS** | 精确 grep（4 个特征串）0 命中；生产调用点仅 `lib.rs:6339` 注释 | 判定依据见 §4.D D-14；R1 新增 LAN 提示**不属该变体** |
| AC-22 跨设备可达 | **部分验证** | 本机以 LAN IP `172.24.0.2:13465` → `/health` 200、`/` 200 | **跨另一台主机未验证**（见 §6） |
| AC-23 同源无 CORS | **未验证** | 静态：`lib.rs:1222-1227` `allow_origin(predicate(is_loopback_origin))`；页面由 daemon 同源提供 | 需真实浏览器（见 §6） |
| AC-24 无 provider 可启动 | **PASS** | `RUSTCODE_HOME=$(mktemp -d)`；进程 ALIVE；`/health` 200；`/` 200 | |
| AC-25 无阻断遮罩 | **未验证** | 替代：`webui/src/app.tsx` grep `provider` 0 命中（`00-requirement.md` F6） | 需浏览器渲染（见 §6） |
| AC-26 网页端完成配置 | **部分验证** | `POST /providers` 201（含 `is_default: true`）→ `POST /providers/t07-api/default` 200 → `config.toml` 落 `default_provider = "t07-api"` 与 `[providers.t07-api]` → `GET /providers` 返回该条目；无需重启 | **UI 交互层未验证**（见 §6） |
| AC-27 审批不静默 bypass | **PASS** | `permission_request` 事件（tool=bash，reason=Requires approval）；`/tmp/t07_approval_probe_dir/f.txt` 仍在；mock 未收到第二轮 | 端到端构造成功（见 §4.E） |
| AC-28 非回环无鉴权判定被显式覆盖 | **PASS** | Q2-A 三要素：`daemon/src/main.rs:21` 保持 `127.0.0.1`；`:177` `webui_tokens: Some(..)` → `enforce_token=true`；`lib.rs:1284-1294` 覆盖 Channel/Webui/Vscode/Jetbrains | 无 token `POST /chat` → 401 |
| AC-29 现有测试不被削弱 | **PASS** | `channel_mode_tests` 4 passed（含 `known_clients_interactive_on_loopback_or_token`）；`diff` 中 `^[+-]\s*assert` 0 命中 | |
| AC-30 默认 host 有测试锁定 | **PASS** | `tests/default_host_lock.rs::standalone_daemon_default_host_stays_loopback ... ok` | 已存在，未重复新增 |
| AC-31 fmt / clippy / test | **PASS** | `cargo fmt --check` 0；clippy（3 crate）exit 0 / `^error` 0；workspace test **5487 passed / 1 failed**（唯一红测为 `AGENTS.md:226` 已文档化项） | clippy 范围为 3 crate（派单许可）+ `cargo check --workspace --all-targets` exit 0 |
| AC-32 无新增 Emoji | **PASS** | `gate`：`AC-2/3/4/6/7b/32 全量 check` 0 FAIL | |
| AC-33 前端回归 | **PASS** | `npx tsc --noEmit` exit 0 / 0 错；`npm test` 227 pass / 0 fail；`npm run build` 成功 | |

**统计：PASS 29 · 部分验证 2（AC-22 / AC-26）· 未验证 2（AC-23 / AC-25）· FAIL 0。**

---

## 6. 缺陷与归属

### D-1【低 · 遗留登记，不阻塞】`is_loopback_authority("::1") == false` → `--host ::1` 误报非回环提示

- **现象**：`crates/rustcode-daemon/src/lib.rs:1272-1279` 的 `is_loopback_authority` 对裸 `::1` 返回 `false`，导致 R1 新增的 `run_server` 提示分支在 `--host ::1` 下打印「已绑定非回环地址 / Bound to a non-loopback address」。
- **实测复现（本轮）**：
  ```bash
  # (1) 复刻谓词（/tmp/loopback_probe.rs，rustc 编译）
  ::1                  -> false      ← 期望 true
  [::1]                -> true
  [::1]:13456          -> true
  localhost            -> true
  127.0.0.1            -> true
  0.0.0.0              -> false
  192.168.1.5          -> false
  ::ffff:127.0.0.1     -> false      ← 期望 true（IPv4-mapped 回环）

  # (2) 运行时
  $ RUSTCODE_HOME=$(mktemp -d) ./target/debug/rustcode-daemon --host ::1 --port 13464
  RustCode API server listening on http://::1:13464
  [!] Bound to a non-loopback address: anyone who can reach it can get in with this token. Use only on a trusted network (no TLS).
  ```
- **根因判断**：`strip_prefix("[::1]")` 只处理带括号形式；裸 `::1` 走 `authority.split(':').next()` 得到空串 `""`，`matches!("", "localhost" \| "127.0.0.1" \| "::1")` 为 false。Q3 之前的旧代码用字面量 `host != "::1"` 显式排除过 `::1`，故这是 R1 新路径**继承**（不是新写）的谓词缺陷；同一谓词早被 `lib.rs:5399`（`ensure_server_and_open`）与 `lib.rs:1288` 使用，缺陷在这些路径上**已存在**。
- **方向性**：**过度告警**（fail-safe 方向）——不会「本该告警却静默」，故不阻塞 G5。
- **归属**：既有谓词缺陷；本轮新路径（R1/T-13）继承。**PM 已裁决需 architect 评估，本轮仅登记，不修。**
- **建议修法（供 architect 参考，本轮不执行）**：
  1. 新增纯函数 `normalize_authority(authority) -> &str`：剥离 `[...]` 括号、`strip_port`（IPv6 仅按最后一个 `:` 且其后为数字才视为端口）、`trim`；
  2. `is_loopback_authority` 改为 `matches!(host, "localhost" | "::1" | "::ffff:127.0.0.1")` 或按 `IpAddr` 解析后判 `is_loopback()`（含 `127.0.0.0/8`）；
  3. **必须同步评估**：该谓词是 `client_interactive_permission`（`lib.rs:1284-1294`）的组成部分。把 `::1` 从 `false` 改为 `true` 会**放宽**权限面（在 `enforce_token=false` 时使 `Vscode/JetBrains/Channel/Webui` 于 `--host ::1` 下获得交互审批 ⇒ `dangerously_skip_permissions=false`，方向是更严格；但若反向用于「是否允许」类判定则需重审全部消费点）。建议先补 `#[cfg(test)]` 覆盖 `::1` / `[::1]` / `::ffff:127.0.0.1` / `127.0.0.1` / `localhost` / `0.0.0.0`，再改实现，最后跑 `cargo test -p rustcode-daemon` 全量。
- **严重度**：**低**。

### D-2【低 · 口径偏差】AC-8 字面表述（全仓 `README.zh-CN` 0 命中）不成立

- **现象**：`git grep -n -F "README.zh-CN"` 全仓 **39 处命中**，全部位于 `.codebuddy/artifacts/**`（本 feature 自己的过程文档，用于记录「删除 README.zh-CN.md」这一决策与其依据）。`00-requirement.md` §4.1 AC-8 的字面要求是"全仓 grep 0 命中"。
- **根因判断**：AC 撰写时未考虑「本 feature 的过程文档自身会引用被删文件名」这一自指情况；`gate` 实现已把 `.codebuddy/artifacts/` 划为「历史域」排除。
- **归属**：编排者口径裁决（已下发「正式域 / 历史域 / 兜底域」三域判定）→ 本轮按该口径判 PASS。**若 PM 要求恢复字面 0 命中**，则需改写 artifacts 内 39 处表述，归属建议 **T-11 / doc-writer**。
- **建议**：在 `06-release.md` 的 AC-8 条目上注明「按三域口径判定；历史域 39 处为本 feature 过程文档自指，不计入」。
- **严重度**：**低**（不阻塞）。

### D-3【低 · 脚本健壮性】`check-zh-docs.py` 的 AC-2 围栏剥离对嵌套围栏失效（over-count）

- **现象**：`crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/SKILL.md:218`（`  "mcpServers": {`）被计入 `en` 残留行，尽管它位于 ```json 围栏（file 216 开）内。
- **根因判断**：`ac2_en_lines` 的 R2 用**单一布尔 `inside` 做奇偶翻转**（`scripts/check-zh-docs.py:353-358`）。该文件在 file 192 处有外层 ```markdown，内层 ```json（file 216）会被当成外层的**关闭**围栏，于是 217-224 行被当作正文；其中 4 空格缩进的 219-222 被 R3-a「缩进代码块」兜住，而 2 空格缩进的 218 漏出 → 计入 `en`。基线 `3ee655e3` 的同文件**结构完全相同**（```markdown @193 / ```json @217 / ``` @226 / ``` @251），故**非本轮汉化引入**。
- **影响**：本文件 `en=1/152 = 0.0066`，远低于 0.05，**不影响任何 PASS 结论**；方向是**高估** `en`（fail-safe，不会漏报英文散文）。但若其它文件出现反向奇偶错位，理论上可能把真实英文散文误当代码跳过（**低估**），属于健壮性隐患。
- **归属**：**T-01**（`scripts/check-zh-docs.py`）。
- **建议修复**：R2 改为跟踪「当前围栏标记与其长度/缩进」的栈，只有同长度（或更长）的围栏才能关闭；或对 CommonMark 语义下的嵌套（外层 4 反引号）显式支持。修复后需复跑 `gate` 确认 `en` 计数只减不增。
- **严重度**：**低**。

### D-4【观察 · 低】`Msg::WebuiLanWarning` 在 `run_server` 路径下的范围断言与指代缺失

- **现象**：`rustcode daemon --port 13461` 输出顺序为 `RustCode API server listening on http://0.0.0.0:13461` 紧接 `[!] Primary address is a LAN IP; only devices on the same network can reach it...`。该路径**未打印任何局域网 IP**（"主地址"指代对象未出现），且绑定 `0.0.0.0` 时「仅同一网络内的设备可访问」这一范围断言可以为假（可能含公网网卡 / 端口转发）。
- **根因判断**：`Msg::WebuiLanWarning` 文案是为 `ensure_server_and_open` 路径写的（那里会探测并打印 `lan_ip`），`run_server` 路径复用时缺少该上下文。与 T-13 §5「遗留判断 A」为同一问题，本轮实测复现。
- **归属**：**T-04（i18n owner）+ solution-architect**（P1 新增 `Msg::DaemonBindWildcard` / P2 改发 `WebuiNonLoopbackWarning` / P3 现状，待裁决）。
- **建议**：维持 T-13 的建议；若选 P2，`run_server` 的 `0.0.0.0`/`::` 分支改发 `Msg::WebuiNonLoopbackWarning`（文案对通配绑定完全准确且不宣称范围），代价是与 `ensure_server_and_open` 不一致且丢「隧道」提示。
- **严重度**：**低**（认知层，风险信息「无 TLS，凭 token 即可进入」仍在）。

### D-5【观察 · 信息】`rustcode daemon` 子命令无 `--host` 参数

- **现象**：`./target/debug/rustcode daemon --host ::1 --port 13463` → `error: unexpected argument '--host' found` (exit 2)。`rustcode daemon` 只接受 `--port`；其 host 在 `cli/src/main.rs:1780` 硬编码为 `"0.0.0.0"`。
- **影响**：用户无法对 `rustcode daemon` 子命令限制绑定地址（只能靠防火墙/反向代理）。这是**既有设计**（Q2-A 范围内只要求改默认值），不是本次改动引入的回归。
- **归属**：既有行为，非本 feature 缺陷；如需 `--host` 支持建议独立开单。
- **严重度**：**信息**。

---

## 7. 未验证范围

| 项 | 未验证内容 | 所需环境 | 建议补验方式 | 负责人建议 |
|---|---|---|---|---|
| **AC-22（跨设备部分）** | 局域网**另一台主机**对 `<LAN-IP>:<port>` 的 `/health` 与 `/` | 同一二层网络的第二台主机（或另一容器 / network namespace + veth） | 在第二台主机执行 `curl -sSf http://<LAN-IP>:<port>/health` 与 `curl -sSf http://<LAN-IP>:<port>/ \| grep -c '<html'` | test-engineer（需环境） |
| **AC-23** | 浏览器同源 fetch 无 CORS 错误 | 真实浏览器（DevTools Network 面板）+ 已完成 `?token=` → Cookie 交接的页面 | 打开 `http://<LAN-IP>:<port>/?token=…`，在 DevTools 中确认 `/health`、`/project`、`/providers` 全 2xx 且 Console 无 CORS 错误；或 headless Chrome + `--disable-web-security=false` 抓取 console 日志 | test-engineer（需浏览器环境） |
| **AC-25** | 无 provider 时首页无全屏阻断遮罩、可打开设置对话框 | 真实浏览器渲染 | Playwright/Puppeteer 打开首页，断言无「必须先配置 provider」类遮罩；点击设置 → provider 列表可见（空态文案允许） | test-engineer（需浏览器环境） |
| **AC-26（UI 层）** | 在**设置对话框 UI** 中新建 provider 并设默认 | 真实浏览器 | 在 UI 中填 name/base_url/api_key/model → 保存 → 设默认 → 校验 `config.toml` 与 `GET /providers`。**注：本轮已用 HTTP API 完成同一后端链路（201/200/落盘/回读全通过），仅 UI 交互未验证** | test-engineer（需浏览器环境） |
| **AC-31 clippy 全 workspace** | `cargo clippy --workspace --all-targets` | 无（仅耗时） | 本轮按派单许可只跑 `-p rustcode-daemon -p rustcode -p rustcode-config --all-targets`（exit 0）；已用 `cargo check --workspace --all-targets`（exit 0，`^error` 0）作跨 crate 补充。如需完整 clippy，单独排期跑一次 | test-engineer |
| **AC-27 的真实 IDE 拉起路径** | 由 VS Code 扩展（`extensions/vscode/src/daemon/process.ts:369`）真实拉起 daemon（不带 `--host`） | VS Code + 已构建扩展 | 本轮用命令行 `--port 13456 --client vscode` + `X-RustCode-Client: vscode` 头**等价模拟**（`startup_mode` 是死字段 N1，`--client` 对请求处理无影响，客户端身份 100% 来自请求头，故等价性成立） | 可豁免；若需，由 IDE 扩展 owner 补验 |
| **AC-13/14 在 macOS/Windows 的行为** | 脚本在非 Linux 下的 PATH/版本解析 | macOS / Windows 主机 | 本轮仅在 Linux（zsh/bash）验证 | 可豁免（脚本为 bash，AGENTS 环境为 Linux） |

---

## 8. G5 结论

### 结论：**pass**

**判据逐条对照 `02-tasks.md`「完成判据」：**

| 完成判据 | 本轮结果 |
|---|---|
| `gate` exit 0 | ✔ `GATE_EXIT=0`，报告已落盘 `03-impl/zh-check-gate.md`（1185 行） |
| AC-3 抽检全白名单 | ✔ 43 行 → 等距抽样 20 行，**20/20** 落 §3.2 白名单 |
| `cargo fmt --check` 通过 | ✔ `FMT_EXIT=0`，0 行输出 |
| `cargo test --workspace` 仅剩 `AGENTS.md:226` 已文档化的那一条红测 | ✔ **5487 passed / 1 failed**，唯一红测 = `mcp::registry::tests::trust_key_golden_matches_core_algorithm`（存量，文件不在本 feature 改动集内） |
| AC-27/28/29/30 全部通过 | ✔ AC-27 **端到端实证**（`permission_request` 出现 + 破坏性命令未执行）；AC-28 三要素在位；AC-29 4 passed 且断言无净减少；AC-30 `default_host_lock` 在场且通过 |
| AC-33 前端三项通过 | ✔ `tsc --noEmit` 0 错；`npm test` 227/0；`npm run build` 成功 |

### 能否推进 G6：**可以推进**，但须随 G6 交付以下两项

1. **必随 release 说明记录**（非阻塞，但不可遗漏）：
   - 缺陷 **D-1**（`is_loopback_authority("::1")`）与 **D-3**（脚本嵌套围栏）—— 已交 architect / T-01 排期；
   - 缺陷 **D-2**（AC-8 三域口径）—— 需在 `06-release.md` 的 AC-8 条目注明判定口径；
   - 缺陷 **D-4**（LAN 文案范围断言）—— 待 architect P1/P2/P3 裁决；
   - `03-impl/T-13-r1-rework.md` §5 已登记的：setup-seeds hash 变更触发已安装用户一次种子重装（可能覆盖用户手工改动）、`Msg::DaemonWarnNonLoopback` 已无发射点（勿当死码清理）。
2. **未验证项必须原样带到 G6 的「已知未验证」清单**：AC-23、AC-25、AC-22 与 AC-26 的跨设备/UI 部分（§7）。**不得**在 release 中表述为已验证。

### 附加说明（不阻塞，供 PM 决策）

- **AC-27 是本轮最大增量**：原计划按「缺 provider ⇒ 未验证」处理，实测通过 mock OpenAI 兼容 provider 成功构造出需审批的 `bash` 调用，把该最高优先级闸门从「替代证据」升级为**端到端实证**，并配非破坏性命令对照组排除「一律阻断」的假阳性。建议后续把这套 mock 夹具固化为 `crates/rustcode-daemon/tests/` 下的集成测试（**需 PM 派单给 code-implementer**，本轮 files_owned 不含测试代码，故未落地）。
- **AC-26 的后端链路已实证**（API 层 201/200/落盘/回读），仅浏览器 UI 交互未验证。
- 本轮**未新增任何测试文件**：T-07 的 `files_owned` 只含两个报告件，且既有测试已覆盖 AC-29/30；新增集成测试会越界，故登记为建议而非执行。
