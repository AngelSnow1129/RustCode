# 2026-09-09-codingplan-removal-eval 看板

- 当前阶段：实施（G3，按 `07-orchestration-optimization.md` 的 9 轮调度）
- 基线（**已按 HEAD 重测更正**）：branch=dev commit=**e80fb7ae** worktree=clean
  - 沿革：`d5b4f2e1`（原记）→ `d4695013`（omo-skills 第二批入库）→ **`e80fb7ae`**（omo-skills 收尾）。
  - T-10 在 `e80fb7ae` 上**重跑刷新**（此前一轮是在 `d4695013` 上跑的），数值与 OPT-001 §1.1 记录**完全一致**：
    5488 passed / 1 failed（`trust_key_golden_matches_core_algorithm`）/ 11 ignored；
    `N_RS=1100` / 66 文件、`N_EXT=65`、`N_DOC=117`、`Cargo.lock` 6 行；`cargo fmt --check` exit 0。
  - 下游常引基线：daemon lib **307/0**、tuix lib **2064/0**、codingplan lib **27/0**。

## 目标

彻底移除 codingplan 相关代码（方案 B），并同步 `AGENTS.md` / `docs/` 口径。

## 用户裁决（推翻既有结论，已获明确授权）

| 项 | 内容 |
| Q1 范围 | **B 彻底移除**（约 6000–7500 行，跨 8 crate + 2 扩展 + webui + 18 份文档） |
| Q2 交付 | **评估 + 直接实施到底**（G2→G5），提交前单独征求同意 |
| Q2' config 识别层 | **一并删除**（`is_codingplan_provider_name` / `LEGACY_CODINGPLAN_PREFIX` / `codingplan_group_account_id` / `codingplan_builtin_effort_levels` / `is_codingplan_llm_gateway`）→ **持久化语义变更已授权** |
| Q5 扩展与前端 | **一并清理**（VS Code + JetBrains + webui） |
| Q3 i18n | 随 B 一并删（`Cp*` 全族 + `CodingPlanSetupFailed` + `WelcomeOptionCodingPlan*` + `LoginManagedUnavailable` 待定），三处同删 |
| Q6 卸载清理项 | **编排者定夺：保留**（`cli/src/uninstall/paths.rs:36`、`:150`、`scripts/uninstall.sh:21`、`scripts/uninstall.ps1:24`）。理由：`AGENTS.md:87` 保留旧 `telemetry/` 目录的同一先例——`codingplan_sync.json` 是真实存在的历史文件，删清理项会让老用户残留孤儿文件；这也是 `00-requirement.md` AC-6 对 B 方案预留的唯一白名单例外。若用户否决，单列 T-07 处理。 |
| Q8 重复实现 | 随 B 自然消除（`endpoints.rs` 与 `codingplan_sign.rs` 两条 `is_codingplan_gateway` 可达路径一并删除） |

**被本次裁决推翻的既有结论**（实施时必须同步修订，不得遗漏）：

1. `2026-09-02-cleanup-codingplan-legacy/STATUS.md:67-76` 用户裁决 S2 标准档（commit `783d48e4`）：原定不移除 B 类（client/setup/api_codingplan）与 C 类闭源签名桩。**本次推翻。**
2. `AGENTS.md:289` / `:498` / `:509` / `:537`：四次记载「`#[cfg(feature="codingplan")]` 块按 fork 铁律不动」。**本次推翻。**
3. `AGENTS.md:345`：`Cp*` 签名族 i18n 契约「按门控不删除铁律不动」。**本次推翻。**
4. `AGENTS.md:329`：扩展侧 `/codingplan/setup` 客户端方法/类型为保留开关。**待 Q5 裁决。**
5. `AGENTS.md:220-221` G7 门禁：把 `is_codingplan_provider_name` / `LEGACY_CODINGPLAN_PREFIX` 写进豁免清单。**待 Q2 裁决；若删则 G7 条文需同步重写。**

## 门禁

| 门禁 | 状态 | 依据 |
| G1 需求 | pass | `00-requirement.md` + 用户裁决 Q1=B / Q2'=删识别层 / Q5=清前端 |
| G2 设计 | pass | `01-design.md` / `02-tasks.md`；契约已冻结，11 批 22 任务 |
| G3/G4 开发+审查 | **blocked** | 9 轮调度（`07-orchestration-optimization.md`）；P1 done、**P2（T-11'）实现完成且验证绿（未提交）**；F 组 T-26/T-27/T-30 同步开工。**因并发写入冲突暂停提交与 P3，见「阻塞项」** |
| G5 集成测试 | pending | 待 P9 |
| G6 交付 | pending | 待 P9 |

**默认策略（前次提问未获答复，按推荐项执行）**：每轮 1 commit、不 push；任一轮编译红且一轮返工未修复则停下汇报；回滚按 §4.3 三组整批 revert。

## 阻塞项 B-1（2026-09-09，编排者 OPT 轮发现）

**并发 Agent 正在改写同一工作区，且出现契约外删除。** 事实（实测，非推断）：

1. P2 执行期间，`crates/rustcode-daemon/src/api_auth.rs` 被**另一个 worker** 改写（mtime 12:42:15）。
   对方除删 codingplan 调用外，**额外删除了 `LoginPollStep::Authorized.newly_authorized` 字段**并连带改了 5 处
   （含 `GearMenuLabelsTest.kt` 同级测试夹具）。`grep -rn newly_authorized crates/` → **0 命中**，字段已全仓消失。
2. 该删除**不在 `01-design.md` 冻结契约的删除表内**，属越界改动。
3. vscode / jetbrains / docs / README 在 P2 期间同步被改；`extensions/jetbrains/.gradle/9.3.0/**` 有写入
   → **gradle 实际可用**（与 `AGENTS.md:318`「本机无 JDK/gradle」的既有结论不符，需复核该条文）。
4. `cargo metadata` exit 0，manifest 仍可解析；`cargo check --workspace --all-targets` 在含上述改动的全量下 exit 0。

**处置**：P2 不提交、P3 不启动。等用户裁决：
- (a) 接受越界删除并继续（需把它补记进冻结契约的删除表）；
- (b) 回退 `newly_authorized` 相关删除（仅该字段，不回滚其余）；
- (c) 串行化：等 F 组全部落地后再进 P3。

## 任务

| id | 标题 | 批次 | 负责 Agent | 状态 | 返工轮次 | 交接件 |
| T-01 | 需求澄清与边界定档 | 1 | requirements-analyst | done | 0 | `00-requirement.md` |
| T-02 | 架构方案与任务拆分 | 2 | solution-architect | done | 0 | `01-design.md` / `02-tasks.md` |
| T-03 | 实施（方案 B） | 3 | code-implementer | in_progress | 0 | `03-impl/` |
| T-04 | 审查 | 4 | code-reviewer | pending | - | `04-review/` |
| T-05 | 集成测试 | 5 | test-engineer | pending | - | `05-test-report.md` |
| T-06 | 交付说明与文档同步 | 6 | doc-writer | pending | - | `06-release.md` |
| T-10 | B1 基线留档（AC-0） | B1 | code-implementer | **done** | 0 | `03-impl/T-10.md`（产物名以此为准，订正 `02-tasks.md:52` 的 `baseline.md`） |
| T-32 | B1 补采 clippy 基线（C-2） | P1 | 编排者 | **done** | 0 | `03-impl/T-10-clippy-baseline.txt`（exit 0，53 条存量告警） |
| T-11 | B2 daemon：删 api_codingplan / 三条路由 / feature | B2 | code-implementer | pending | 0 | `03-impl/T-11.md` |
| T-12 | B2 cli：删 run_codingplan_core / login 链 / feature | B2 | code-implementer | pending | 0 | `03-impl/T-12.md` |
| T-13 | B3 tuix 消费点整体剥离（T-13+14+15 合并） | B3 | code-implementer | **in_progress（受阻）** | 3 | `03-impl/T-13.md` |
| T-14 | B3 tuix lib/state/render：去 read_last_sync 与 footer_usage | B3 | code-implementer | pending | 0 | `03-impl/T-14.md` |
| T-15 | B3 tuix LoopCtx：删 usage/monitor 字段与调用点 | B3 | code-implementer | pending | 0 | `03-impl/T-15.md` |
| T-16 | B4 删 tuix 四模块（usage/usage_render/usage_monitor/monitor） | B4 | code-implementer | pending | 0 | `03-impl/T-16.md` |
| T-17 | B4 摘 tuix 依赖边 + 删 rustcode-codingplan crate（原子） | B4 | code-implementer | pending | 0 | `03-impl/T-17.md` |
| T-18 | B5 daemon：去 is_codingplan_gateway/signer_available + managed/requires_login 字段 | B5 | code-implementer | pending | 0 | `03-impl/T-18.md` |
| T-19 | B5 tuix：provider_panel + event_loop/mod.rs 去 L2/L1 消费点 | B5 | code-implementer | pending | 0 | `03-impl/T-19.md` |
| T-20 | B5 clix：去 is_codingplan_gateway 分支 | B5 | code-implementer | pending | 0 | `03-impl/T-20.md` |
| T-21 | B6 删 auth::gateway_crypto + capabilities::codingplan_sign + 闭源桩 | B6 | code-implementer | pending | 0 | `03-impl/T-21.md` |
| T-22 | B6 coding：去 ProviderAuthenticator / CodingPlanProviderAuthenticator | B6 | code-implementer | pending | 0 | `03-impl/T-22.md` |
| T-23 | B7 config：删识别层与折叠 + 新测试（持久化语义变更落点） | B7 | code-implementer | pending | 0 | `03-impl/T-23.md` |
| T-24 | B7 kernel/coding/capabilities 注释中立化 | B7 | code-implementer | pending | 0 | `03-impl/T-24.md` |
| T-25 | B8 i18n 三件套收尾（Cp*/StatusCp*/Usage* + /usage 变体改名） | B8 | code-implementer | pending | 0 | `03-impl/T-25.md` |
| T-26 | B9 VS Code 扩展清理 | B9 | code-implementer | pending | 0 | `03-impl/T-26.md` |
| T-27 | B9 JetBrains 插件清理（无 JDK，仅源码级验证） | B9 | code-implementer | pending | 0 | `03-impl/T-27.md` |
| T-28 | B9 webui 清理 + npm 重建 + cargo clean -p rustcode-daemon | B9 | code-implementer | pending | 0 | `03-impl/T-28.md` |
| T-29 | B10 AGENTS.md 重写被推翻条目（含 G7 :220-221） | B10 | doc-writer | pending | 0 | `03-impl/T-29.md` |
| T-30 | B10 docs/ 12 份 + README + crate README 口径同步 | B10 | doc-writer | pending | 0 | `03-impl/T-30.md` |
| T-31 | B11 全量验证（AC-1~AC-15）+ 残留清单 + 交付说明 | B11 | test-engineer | pending | 0 | `05-test-report.md` / `06-release.md` |

> T-10 ~ T-31 为 T-03（实施）的展开子任务，批次列为 `01-design.md` / `02-tasks.md` 中的 B1~B11。
> **调度已被 `07-orchestration-optimization.md` 覆盖**：关键路径 14 轮 → **9 轮**（P1~P9）；`02-tasks.md` 的批次编号仅保留为任务内容索引，不再作为执行顺序。

## 编排优化（OPT-001，2026-09-09）

- 合并：T-11+T-12 → **T-11'**（`cli/Cargo.toml` 引用 `rustcode-daemon/codingplan`，拆开则后者无法自验，C-8）｜T-13+T-14+T-15 → **T-13'**（同字段 `footer_usage` 三处耦合）｜T-16+T-17 → **T-16'**（原子）。
- 重排：T-23 并入 P7 与 T-22 并行（实测 L2→L1 依赖：`gateway_crypto.rs:100-101` 调 `endpoints::is_codingplan_llm_gateway`，故 T-23 依赖 T-21，不可早于 P6）。
- 离链：T-26 / T-27 / T-30（零依赖，立即开工）｜T-28（P5 后）｜T-29（与 T-25 并行）。
- 验证右配：`cargo check --workspace --all-targets` 仅 P2/P4/P7/P8/P9；轮内改跑受影响 crate。
- 审查左移：R1（P4 后）、R2（P8 后）只读、与下一轮并行；返工半径 9 批 → 1 批。

### `files_owned` 补归属（C-1，原任务图遗漏，会潜伏到 T-31）

| 文件 | 命中 | 补入任务 |
|---|---|---|
| `crates/rustcode-daemon/src/kernel_runtime.rs:177-179` | 2（真实代码，cfg 块） | **T-11** |
| `crates/rustcode-daemon/src/api_auth.rs:289-296` | 3（真实代码，cfg 块） | **T-11** |
| `crates/rustcode-daemon/src/live_api.rs:291/2125` | 2（注释） | T-11 |
| `crates/rustcode-tuix/src/event_loop/oauth_poll.rs:27/43/49/90` | 4（注释） | **T-13'** |
| `crates/rustcode-auth/src/oauth.rs:604/679` | 2（注释） | T-21 |
| `crates/rustcode-kernel/tests/rate_limit.rs:313/370` | 2（注释） | T-24 |
| `crates/rustcode-capabilities/src/provider/openai_compat.rs:3615/3619` | 2（403 测试夹具串） | **AC-6 白名单保留** |

## 关键事实（实施依据）

- 四层划分：L1 config 识别/兼容层（始终编译）｜L2 签名层 `auth::gateway_crypto` + `capabilities::codingplan_sign` + `coding::CodingPlanProviderAuthenticator`（始终编译，五个 crate 非门控调用）｜L3 网络侧（cfg 门控）｜L4 闭源桩（29 行 `unreachable!()`）。
- L3/L4 默认构建下已是死代码：`endpoints.rs:71/75/79` 三个 `HOSTED_*` 为空串 → `is_codingplan_llm_gateway()` 恒 false。
- CLI `rustcode codingplan` 与 TUI `/codingplan` **均已不存在**，不是待删项。
- 最高风险：① 持久化语义变更（`config/mod.rs:797-816` 账号折叠）② feature 传递链断裂（`AGENTS.md:58`，须跑 AC-4 七种 `cargo check` 组合）③ 公共协议退役（三条 `/codingplan/*` 路由有两个 IDE 扩展真实客户端）。

## 决策记录

| 时间 | 决策 |
| 2026-09-09 | 建立基线，派发 requirements-analyst，产出 `00-requirement.md`（blocked）。 |
| 2026-09-09 | G1 判 blocked，Q1 升级用户。 |
| 2026-09-09 | **用户裁决 Q1=B（彻底移除）+ 实施到底**，推翻 S2 裁决与四条 AGENTS.md 铁律。 |
| 2026-09-09 | Q2（config 识别层）/ Q5（扩展前端）升级用户，暂停 G2。 |
| 2026-09-09 | **OPT-001 编排优化**：关键路径 14 → 9 轮；补 7 个未归属文件（含 2 处 daemon 真实代码）；补 clippy 基线；审查左移 R1/R2。见 `07-orchestration-optimization.md`。 |
| 2026-09-09 | **编排者接手（并发检测后）**：发现 tuix/docs 有非本会话派发的改动 → 判定存在并发编排者；按推荐默认「我方独占接手 + 暂不提交」执行。 |
| 2026-09-09 | **P2 / T-11' 完成并验证**：daemon+cli 删除，fmt 0 / workspace check 0 / daemon lib 307/0 / 两条 feature 断言 101（期望）。T-26（vscode）完成：tsc 双 project 0、webview 11/11 与基线逐行一致、grep 0。 |
| 2026-09-09 | T-27（jetbrains）完成：源码级核验 8/8 落实、5 个被删符号零残留引用、**实测纠正前任事实错误（本机有 JDK 20 + gradle 9.3.0，但仍未编译：需联网下载 GB 级 IntelliJ SDK，且 gradle 读工作区外 `/root/.gradle`）**。 |
| 2026-09-09 | **T-13' 受阻（系统性，非任务本身）**：连派 6 次有 5 次中途 stall；最近两轮进展为 0（grep 127→127、failed 5→5）。根因：`event_loop/mod.rs` 与 `commands.rs` 均约 2.5 万行，agent 在其上反复耗尽。当前 tuix **编译绿 + fmt 绿，但剥离未完成（127 残留）、5 测试红**。按 OPT §4.4 中断策略停下汇报。 |

## 阻塞项 B-2（2026-09-09，P3 两轮未达成，升级）

**P3（T-13' / T-13-finish）连续 2 轮派发均未产生实质改动**，看板状态置为 `blocked`，待用户裁决后继续。

事实（实测）：
1. 第一轮 T-13'：改了 `lib.rs` / `state.rs` / `event_loop/commands.rs` / `event_loop/mod.rs` 四文件（编译绿，
   `cargo check --workspace --all-targets` exit 0），但**报告 §1~§5 全部为「（实现填充）」占位**，
   且 `event_loop/commands.rs` 仍 70 处、`event_loop/mod.rs` 仍 48 处 codingplan 命中（含
   `rustcode_codingplan::run()` / `merge_successful_config()` / `read_last_sync()` 等真实调用）。
2. 第二轮 T-13-finish：11 次工具调用 / 57.83s 后中断，**残留计数与派发前逐字节相同**，报告未重写。
3. 中断模式：子 Agent 在读取 `crates/rustcode-config/src/i18n/en.rs` 时触发**需审批的 Bash 调用**而停摆
   —— 子 Agent 无法自行通过审批，是本轮的直接死因。

**根因（编排侧，非代码侧）**：
- A. 单批次过大：`commands.rs` 70 处 + `mod.rs` 48 处 + 需删 15 个测试模块，单 Agent 一轮做不完。
- B. 报告骨架化：要求「先落骨架后填内容」→ 一旦中断就只剩占位，无有效交接件。
- C. 子 Agent 触发审批即停摆 → 派工必须限定在已授权命令集内。
- D. 并发 worker 与本批同工作区（B-1 同源）。

**候选处置（待用户选）**：
- (a) 拆分 P3 为 P3a（`event_loop/commands.rs` 70）/ P3b（`event_loop/mod.rs` 48）+ P3c（注释类 12），
      各自单文件单 Agent，并**禁用需审批命令**；
- (b) 用 git worktree 隔离后续批次，彻底消除并发污染；
- (c) 停手，交人工/更小的步子推进。
