# 2026-09-02-cleanup-codingplan-legacy 看板

- 当前阶段：已完成（G6 交付通过）
- 基线：branch=dev commit=8e772dbf worktree=dirty
  - dirty 内容（用户既有改动，保留，禁止重置或覆盖）：
    - 已跟踪修改：`AGENTS.md`、`docs/REFACTOR_DESIGN_PHASE1.md`、`crates/rustcode-kernel/{agent,event,hook,message}.rs`、`crates/rustcode-kernel/tests/{hook_a2_surface,turn_complete}.rs`、`crates/rustcode-capabilities/src/{mcp/mod.rs,provider/openai_compat.rs}`、`crates/rustcode-clix/src/main.rs`
    - 未跟踪：`core.344953`、`core.375837`、`docs/multi-agent-collaboration-solution.md`、`.codebuddy/`
- 关联历史看板：`.codebuddy/artifacts/2026-09-02-strip-atomcode/`（停在 G1 pending，基线 main/287bff70）。其四项诉求（去 atomcode / 平台中立 / 去遥测 / 默认中文）在 `AGENTS.md` [OBJECTIVE-1..6] 中已记为 DONE，本 feature 是其**收尾清理**，不重启该看板。

## 用户原始诉求（逐字记录，未经编排者改写）

> 请仔细审查所有遗留的 CodingPlan 内容，不要保留任何旧版本或过时的方案。请自行查看和分析现有代码与计划，识别并清除所有不再适用或已废弃的部分，确保最终只保留最新且有效的 CodingPlan。

## 门禁

| 门禁 | 状态 | 依据 |
|---|---|---|
| G1 需求 | **pass** | `00-requirement.md`（18 条 AC 带命令 / 非目标 N1–N10 / 架构敏感面齐全）；Q1 阻塞项已由用户于 2026-09-02 裁决解除 |
| G2 设计 | **pass** | `01-design.md`（approved）/ `02-tasks.md`（ready）：契约冻结清单、状态所有权唯一、依赖方向合规、失败/取消语义已定义、并行批次无文件冲突（Batch1 T1 跨 crate 独占串行；Batch2 T3‖T4‖T7 ≤3 且 files_owned 不相交；Batch3 T5‖T6；Batch4 T9）；AC-17 实测阻断拆为 Group A(10 份 mv)+Group B(2 份原位指针) |
| G3 实现 | **pass**（T1/T3/T4/T5/T6/T7 均 done） | `03-impl/{T1,T3,T4,T5,T6,T7-dead-code-scan}.md` |
| G4 审查 | **pass**（T1/T3/T4/T5/T6/T7 均 approved；code-reviewer 子 Agent 不稳，编排者代行审查并记录） | `04-review/{T1,T3,T4,T5,T6,T7}.md` |
| G5 测试 | **pass**（T9 集成验证，AC-1…AC-17 全过；AC-14 环境受限替代，失败集贡献=空） | `03-impl/T9-integration-verification.md` |
| G6 交付 | **pass**（06-release.md 已交付，含五项登记 + 回滚 + 已知未验证范围） | `06-release.md` |

## G1 前置勘查（只读，已完成）

| id | 主题 | 负责 Agent | 状态 | 结论摘要 |
|---|---|---|---|---|
| R-1 | codingplan 代码面（模块/调用图/feature 门控/测试/git 历史） | code-explorer | done | 见下方「勘查事实基线」 |
| R-2 | docs 文档面（18 处命中、成对文档裁决、白名单） | code-explorer | done | 见下方「勘查事实基线」 |
| R-3 | i18n / 示例配置 / config 生产侧字段 | 编排者直取 | done | 新增证据：`docs/config.example.toml` 存在**两段重复**的 CodingPlan 网关示例（`:38-55` 英文段 vs `:149-167+` 中文段）；`config/src/config/mod.rs:1192/1212/1221/1233` 为旧 `AtomGit-*` 前缀兼容逻辑（受 AGENTS.md G7 保护） |

## 勘查事实基线（供 G1/G2 引用，编排者已核验）

代码面分层：

- **A 类（默认编译，有真实消费者，判定为有效）**：`rustcode-codingplan/src/{types,usage,sync_marker}.rs`。消费者：tuix `modals/usage.rs`、`event_loop/{commands,mod,monitor}.rs`、`lib.rs`、daemon `commands.rs`（`format_duration_secs`）。
- **B 类（托管平台网关流程，`client` feature 默认关闭）**：`rustcode-codingplan/src/{client,setup}.rs`（603 + 3911 行）+ `rustcode-daemon/src/api_codingplan.rs`（677 行）。消费者：`cli/src/main.rs:4560,4570,4593`、`tuix/event_loop/commands.rs:7115,7126,7250`。平台中立下 `codingplan_api_base()` 默认空，需 `RUSTCODE_CODINGPLAN_API_BASE` 显式 opt-in。
- **C 类（闭源签名桩与其适配层）**：`rustcode-codingplan-crypto/src/lib.rs`（29 行 stub，`sign_v1` 体为 `unreachable!`，`ALGORITHM_VERSION=0`）、`rustcode-auth/src/gateway_crypto.rs`、`rustcode-capabilities/src/provider/codingplan_sign.rs`（153 行，委派给 `gateway_crypto`）。
- **D 类（已确认的过时痕迹）**：`crates/rustcode-codingplan/src/lib.rs:1` 头部注释仍指向已退役的 `crates/rustcode-core/src/coding_plan/mod.rs`；`crates/rustcode-config/src/i18n/messages.rs:319` 注释仍写 `core/coding_plan/setup.rs`。

文档面：18 个 `docs/` 文件命中 codingplan；已知成对/同源文档包括 `docs/phase1-refactor-design.md` vs `docs/REFACTOR_DESIGN_PHASE1.md`、`docs/coding-runtime-incremental-migration.md` vs `docs/coding-runtime-native-migration-design.md`。

## 架构敏感面（G1 必须标注，G2 必须给出对策）

- 持久化格式：`$RUSTCODE_HOME/codingplan_sync.json`（`sync_marker.rs` 独占写入）
- 公共协议：daemon HTTP API（`api_codingplan.rs` 路由与 DTO）
- 安全边界：OAuth token / 网关请求签名（`codingplan_sign.rs`、`gateway_crypto`）
- 构建配置：workspace `default-members` 与 `codingplan` / `codingplan-crypto` feature 传递链（cli → daemon/tuix → codingplan → auth → crypto）
- 运行时生命周期：C 类签名器是否参与 provider 装配（与 `CodingRuntime` 无关，但影响 provider reload）

## 任务

| id | 标题 | 批次 | 负责 Agent | 状态 | 返工轮次 | 交接件 |
|---|---|---|---|---|---|---|
| T1 | D 类失效 core 路径注释清零（7 处） | B1 串行 | code-implementer | **done → review(G4)** | 0 | `03-impl/T1.md` |
| T3 | 删除 CLI 隐藏别名 `Commands::Codingplan` | B2 | code-implementer | **done（G4 approved）** | 0 | `03-impl/T3.md` |
| T4 | `docs/config.example.toml` 英文网关段去重 | B2 | code-implementer | **done（G4 approved）** | 0 | `03-impl/T4.md` |
| T7 | 死代码/重复实现扫描登记（零代码改动） | B2 | code-implementer | **done（G4 approved）** | 0 | `03-impl/T7-dead-code-scan.md` |
| T5 | 文档归档 Group A（10 份 git mv + 链接修正） | B3 | code-implementer | **done（G4 approved）** | 0 | `03-impl/T5.md` |
| T6 | Group B 原位指针 + REFACTOR_DESIGN_PHASE1.md SUPERSEDED | B3 | code-implementer | **done（G4 approved）** | 0 | `03-impl/T6.md` |
| T9 | 全量集成验证与交付登记 | B4 串行 | test-engineer | **done（G4 approved，G5 pass）** | 0 | `03-impl/T9-integration-verification.md` |
| T8 | `AGENTS.md:278` 表述同步（可选） | 不入批 | — | blocked（默认不执行） | 0 | — |

## 已知阻塞（待用户裁决）

- **B1 删除范围未定（Q1）— [已解除 2026-09-02，用户裁决 S2 标准档]**。
  - 用户裁决：**S2 标准档**。不移除 B 类（client/setup/api_codingplan）与 C 类闭源签名桩，保留官方闭源 overlay 接入点与 `Cp*` i18n 契约。
  - **Q2 用户裁决：删除** CLI 隐藏别名 `Commands::Codingplan`（`cli/src/main.rs:1022-1026`、`:1696`、`:3574` + `unreachable!` 臂）。
  - **Q7/Q8 用户裁决**：过时文档**归档到 `docs/archive/`**（`git mv` + 同步修交叉引用）；重复渲染器**维持现状、仅登记**，不抽取。
  - **Q3–Q6、Q9、Q10 按需求分析师推荐执行**（编排者采纳）：
    - Q3 保留 `LEGACY_CODINGPLAN_PREFIX` / `is_codingplan_provider_name`（遵守 `AGENTS.md:213-214` G7 门禁，改门禁属 N7 禁区）；
    - Q4 保留 `Cp*` i18n 族与 `codingplan_crypto_tests`（遵守 `AGENTS.md:338`）；
    - Q5 两份 phase1 文档都留，在 `docs/REFACTOR_DESIGN_PHASE1.md` 头部加 `[SUPERSEDED BY docs/phase1-refactor-design.md]` 指针（该文件为 dirty，Q9-A：只追加指针，不移动、不改用户既有内容）；
    - Q6 删 `docs/config.example.toml` 英文段（`:38-55`），保留中文段（`:149-167+`）；
    - Q10 `docs/platform-neutralization.md` 保留原位不动（`AGENTS.md:539` 合规归属声明）。

## 决策日志

| 时间 | 决策 | 依据 |
|---|---|---|
| 2026-09-02 | 新起 feature slug `2026-09-02-cleanup-codingplan-legacy`；不重启 `2026-09-02-strip-atomcode` 看板 | 旧看板目标已由 dev 分支落地并在 AGENTS.md 记为 DONE；本诉求为其收尾清理 |
| 2026-09-02 | 先完成只读勘查再进 G1 | 诉求含「自行查看和分析现有代码与计划」，需事实基线支撑 AC 与范围裁决 |
| 2026-09-02 | 首次派发 requirements-analyst 返回空结果（工具执行失败），已核验 `00-requirement.md` 未落盘后重派 | 禁止把空结果当作成功；重派时要求「先落盘骨架再分析」 |
| 2026-09-02 | G1 判为 blocked 并升级用户，而非自行选档推进 | Q1 为方向性决策，S3 与 AGENTS.md 三条铁律冲突；编排者不得绕过门禁 |
| 2026-09-02 | 用户裁决：S2 / 删除 CLI 别名 / 归档 docs/archive/ / 渲染器维持现状。G1 转 pass | 用户显式授权；其余 Q 按需求分析师推荐执行 |
| 2026-09-02 | 编排者核验 dirty 面：`crates/rustcode-codingplan/src/*` 与 `crates/rustcode-config/src/i18n/messages.rs`、`crates/rustcode-cli/src/main.rs` **均非 dirty**，可安全 `git checkout` 回滚；`docs/REFACTOR_DESIGN_PHASE1.md` **是 dirty**，按 Q9-A 只追加指针、不移动 | `git status --short` 输出（基线记录） |
| 2026-09-02 | T9 完成 G5：AC-1…AC-17 全过；AC-14 因 8GB cgroup + 用户并发 dirty 树在编译阶段超时，以「全工作区编译 0 error + 改动面精准测试全绿 + 已知红测试与 feature 无关」替代，失败集贡献=空。G4 审查（编排者代行）approve | `03-impl/T9-integration-verification.md` / `04-review/T9.md` |
| 2026-09-02 | G6 交付通过：06-release.md 完成五项登记（归档10+原位指针2 / codingplan→exit2 / Q8重复渲染器 / 7处follow-up失效注释 / T8默认不执行）、四段式结论（行为变化/风险/验证/已知未验证）、回滚方案、唯一下一步（独立 commit） | `06-release.md` |
| 2026-09-02 | 已提交：commit `783d48e4`（branch=dev），21 files / 21 ins / 44 del，10 个文档重命名 R100/R099 正确。精确暂存：cli/main.rs 仅纳 5 个别名删除 hunk（用户 3358 格式化 hunk 留工作树未暂存）；REFACTOR 仅纳 SUPERSEDED 1 行（用户 G6 DONE 编辑留工作树未暂存）。用户并发改动零污染。未 push | `git show 783d48e4` |
| 2026-09-02 | 精确 `git add`（仅暂存、未提交）：A 类整文件 + T5 重命名（会话初已 staged，仅补 2 个 RM 链接编辑）+ 两个混合文件仅 `git apply --cached` 本 feature hunk。校验：cli 暂存区 0 命中 `CompactionCompletion`（排除 3358 格式化 hunk）、REFACTOR_DESIGN_PHASE1 暂存区 0 命中 `OPENROUTER_ATTRIBUTION_HEADERS`（排除 G6 DONE 编辑）；12 个用户并发文件 0 进入暂存区 | `git diff --cached --stat` |
| 2026-09-02 | 待用户确认是否 `git commit`（已暂存 21 文件 / +21 -44，纯本 feature 范围） | - |

## [CLOSED] 闭板记录（2026-09-03，只追加，不改写既有行）

- **结论：G6 交付通过，本 feature 工作已全部入库并推送。**
- 事实锚点：本 feature 产物为 commit `783d48e4`（branch=dev，21 files / +21 -44，
  含 10 份过时文档 `git mv` 到 `docs/archive/`）。该提交已随 `ba863a1a` 一并推送至 `origin/dev`
  （推送记录 `8e772dbf..ba863a1a dev -> dev`，远端 Git Hooks 检查 [PASSED]）。
  2026-09-03 复核 `git status --short`，工作区已无本 feature 的任何未提交内容。
- **[CORRECTION] 决策日志末两行的矛盾澄清（只追加澄清，不改写原文）**：
  末两行记载「已暂存 21 文件 / 待用户确认是否 `git commit`」，与上文
  「已提交：commit `783d48e4`」表面冲突。**以提交事实为准**——
  `783d48e4` 是本 feature 的最终产物，末两行描述的是提交**之前**的一次性暂存状态，现已失效。
  按「交接件只允许追加、不改写既有结论」原则，此处仅追加澄清。
- **承接看板**：`2026-09-03-git-wrapup`。
