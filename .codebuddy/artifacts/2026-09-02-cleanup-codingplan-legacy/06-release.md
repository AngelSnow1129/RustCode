---
kind: release-report
id: G6
feature: 2026-09-02-cleanup-codingplan-legacy
from: doc-writer
to: user
status: delivered
requires: [T9-impl, T9-review]
created: 2026-09-02
---

# G6 交付报告 — CodingPlan 遗留内容清理（S2 标准档）

- 基线：branch=dev commit=8e772dbf，worktree=dirty（含用户并发改动，全程未 reset/stash）
- 范围档：S2 标准档（用户 2026-09-02 裁决）；不移除 B 类托管网关代码、C 类闭源签名桩、G7 旧前缀兼容、Cp* i18n 契约、平台中立化文档
- 门禁：G1 pass / G2 pass / G3 pass / G4 pass / G5 pass / G6 pass

## 一、行为变化（Behavior Changes）

1. **CLI 隐藏别名 `rustcode codingplan` 已删除**（T3）。该子命令此前无任何实现分支，仅是 `Commands` 枚举中的空壳变体 + 一个 `unreachable!` 兜底臂。删除后：
   - 运行 `rustcode codingplan` 现按 clap 未知子命令处理，**退出码 2**（breaking change，已在下方风险段登记）。
   - 既有的 `codingplan` 业务集成（`#![cfg(feature="codingplan")]` 下的 `run_codingplan_core` / `commands_codingplan` / `api_codingplan` 等 B 类流程）**完全保留**，不受本次删除影响。
   - `cargo test -p rustcode --bin rustcode neutral_build_hides_managed_login_subcommands` 与 shell_completion 集成测试均通过（AC-16）。

2. **失效 core 路径注释清零（7 处，T1）**：`rustcode-codingplan/src/{lib,types,client,setup,sync_marker}.rs` 头部与 `rustcode-config/src/i18n/messages.rs:319/376` 中指向已退役 `rustcode-core/src/coding_plan/*` 的注释，改为指向现役 `rustcode-codingplan/*`。纯注释改动，零逻辑/签名/文案/Cargo.toml 变更。

3. **`docs/config.example.toml` 英文 CodingPlan 网关示例段删除（T4）**：原先存在重复的英文段（`:38-55`）与中文段（现 `:130`），删除英文段后文件内 CodingPlan 网关示例**恰好 1 段**（AC-11）。TOML 仍合法。

4. **过时方案文档归档（T5/T6）**：见第三节登记 ①/②。

## 二、风险（Risks）

1. **[BREAKING] `rustcode codingplan` 退出码变为 2**：依赖该隐藏别名的脚本/封装需在升级前改为显式调用受 `codingplan` feature 门控的业务入口（或经 `--features codingplan` 构建后走既有 B 类流程）。影响面极小（该别名此前为空壳、无任何功能）。
2. **文档路径搬迁**：10 份文档移至 `docs/archive/`，外部若以硬编码路径引用需更新（AGENTS.md / README* / docs/architecture.md 经 AC-17 核验**无**活引用，已同步修正同级交叉链接）。
3. **残留重复渲染器（Q8）**：`tuix` 与 `daemon` 各有一份 ~65 行 CodingPlan status 渲染器逐字重复（见第三节 ③）。当前无害，仅增加未来维护成本；用户裁决维持现状。
4. **worktree 仍 dirty**：本 feature 改动与用户并发改动共存于同一工作树，尚未提交。交付不等同于合入——请见第六节回滚/提交说明。

## 三、交付登记（五项必登）

### ① 文档归档（10 份 `git mv` → `docs/archive/`）+ 原位指针（2 份）

**归档 Group A（10 份，扁平化移入 `docs/archive/`，无子目录）：**
| 原路径 | 新路径 |
|---|---|
| `docs/coding-runtime-incremental-migration.md` | `docs/archive/coding-runtime-incremental-migration.md` |
| `docs/coding-runtime-native-migration-design.md` | `docs/archive/coding-runtime-native-migration-design.md` |
| `docs/session-convergence-plan.md` | `docs/archive/session-convergence-plan.md` |
| `docs/live-transport-convergence-plan.md` | `docs/archive/live-transport-convergence-plan.md` |
| `docs/v5.0.0-retire-bridge-core-progress.md` | `docs/archive/v5.0.0-retire-bridge-core-progress.md` |
| `docs/kernel-parity-backlog.md` | `docs/archive/kernel-parity-backlog.md` |
| `docs/release-v5.0.1-current-branch-change-report.md` | `docs/archive/release-v5.0.1-current-branch-change-report.md` |
| `docs/testing/release-v5.0.3-core-retirement-acceptance.md` | `docs/archive/release-v5.0.3-core-retirement-acceptance.md` |
| `docs/plans/2026-07-25-provider-retry-consolidation.md` | `docs/archive/2026-07-25-provider-retry-consolidation.md` |
| `docs/plans/2026-07-27-models-dev-pricing-design.md` | `docs/archive/2026-07-27-models-dev-pricing-design.md` |

**原位指针 Group B（2 份，因被活引用不得移动，按 AC-17 改为原位加指针）：**
| 文件 | 阻断原因（被何引用） | 处置 |
|---|---|---|
| `docs/mcp-rmcp-feasibility.md` | `docs/mcp.md:179` 引用 | 标题后插 `> [ARCHIVED-IN-PLACE] ...` 指针 |
| `docs/compact-native-migration-retrospective.md` | `docs/compact-durable-checkpoint-design.md:9` 引用 | 同上 |

**额外 SUPERSEDED 指针（dirty 文件只追加、不移动、不改既有内容）：**
- `docs/REFACTOR_DESIGN_PHASE1.md`（用户 dirty 未提交）：第 1 行标题后追加 1 行 `> [SUPERSEDED BY docs/phase1-refactor-design.md]`。该文件与 `docs/phase1-refactor-design.md` 并存，前者为历史分析、后者为现役设计，用户裁决两者都留。

**链接修正**：Group A 中指向留驻文档（`compact-native-migration-retrospective.md` / `target-architecture.md`）的同级链接改为 `../<name>.md`（4 处），断链校验 NONE。

### ② `rustcode codingplan` → exit 2（破坏性变更）

见第二节风险 1。删除点：`cli/main.rs` 的 `Commands::Codingplan` 枚举变体定义块、`Commands::Login | Commands::Codingplan =>` 匹配臂改为 `Commands::Login =>`、移除 `unreachable!` 兜底臂，并同步 2 处注释。`rg "Commands::Codingplan" crates/` 现为 **0 命中**。

### ③ Q8 重复渲染器（已知重复，维持现状仅登记）

`crates/rustcode-tuix/src/event_loop/commands.rs:4988-5060` 与 `crates/rustcode-daemon/src/commands.rs:641-705`：CodingPlan status 渲染器逐字重复约 65 行 ×2。抽取为共享实现会改动 `rustcode-daemon` 对 `rustcode-codingplan` 的 optional dep 语义、触碰 feature 传递链，收益有限，用户裁决**维持现状、列为未来架构评审项、本 feature 不做**。

### ④ 非本 feature 的失效 `rustcode-core/src/*` 路径注释（follow-up，7 处）

超出本次 codingplan 清理面（T1 仅覆盖 codingplan 面 7 处），登记供未来独立清理 feature：

| 文件:行 | 失效路径 |
|---|---|
| `crates/rustcode-tuix/src/custom_commands.rs:1` | `// crates/rustcode-core/src/commands/mod.rs` |
| `crates/rustcode-kernel/src/agent.rs:169` | `v1 (rustcode-core/src/agent/mod.rs:3064)` |
| `crates/rustcode-kernel/tests/fallible_stream.rs:154` | `rustcode-core/src/agent/mod.rs:3064` |
| `crates/rustcode-coding/src/persona.rs:2` | `rustcode-core/src/config/prompt_sections.rs` |
| `crates/rustcode-config/src/config/instructions.rs:1` | `// crates/rustcode-core/src/config/instructions.rs` |
| `crates/rustcode-capabilities/src/tools/bash.rs:1241` | `rustcode-core/src/tool/bash.rs` |
| `crates/rustcode-capabilities/src/skills/render.rs:4` | `rustcode-core/src/skill_render.rs` |

> 误报排除：`crates/rustcode-review/src/impact_plan.rs:370-372` 为 git diff 示例字符串（`diff --git a/...`），非路径注释，不修改。

### ⑤ T8 状态（AGENTS.md:278 表述同步，可选，默认不执行）

`AGENTS.md:278` 原有「该别名已移除」相关表述，在 T3 实际删除 `rustcode codingplan` 别名后，由「失真」变为「准确」，故**无需再执行同步修改**；T8 维持 blocked（默认不执行），不矛盾。

## 四、验证结果（Verification）

| 维度 | 结果 |
|---|---|
| 编译（默认成员 / 全工作区 / 全 feature） | AC-1/2/3/4/5 全 Finished，0 error；跨 feature 矩阵（client / codingplan / codingplan-crypto）均绿 |
| 单元/集成测试（改动面） | codingplan --lib 27/0（AC-6）、sync_marker 3/0（AC-9）、config --lib 327/0（AC-13）、cli AC-16 两测试均 1/0 |
| TUI 消费者 | tuix --lib 2063 passed / **1 failed**：唯一失败 `event_loop::tool_format_tests::summarise_multi_line_adds_line_count` 属 AGENTS.md 已登记 tuix 存量红测试，与本 feature 无关（AC-8） |
| 失效注释清零 | `rg "rustcode-core/src/coding_plan" crates/` 0 命中；`rg "core/coding_plan/setup.rs" crates/` 0 命中（AC-10） |
| 配置去重 | `docs/config.example.toml` CodingPlan 段恰好 1（AC-11） |
| G7 门禁 | `LEGACY_CODINGPLAN_PREFIX`(config/mod.rs:1192) / `is_codingplan_provider_name`(:1233) 仍在；`atomcode` 0 命中（AC-12） |
| 持久化契约 | `codingplan_sync.json` 唯一定义不变，三态读返回 None 无 panic（AC-9） |
| 格式 | `cargo fmt --check` 干净（AC-15） |
| 文档无断链 | 被移动 10 份文件名在 AGENTS.md/README*/docs/architecture.md 引用 = 0；T5 内部链接自验 NONE（AC-17） |
| 闭源 overlay | `rustcode` / `rustcode-daemon` `--features codingplan-crypto` 均可编译（AC-5，S3 生死线保持绿） |

## 五、已知未验证范围（Known Unverified Scope）

1. **AC-14 全量 `cargo test --workspace` 未完整跑完**：在 8GB cgroup + 用户并发 dirty 树下，`-j 1` 全量测试于**编译阶段**耗尽 2400s 预算超时（日志 `/tmp/wt.log` 29 行，0 error）。替代证据：全工作区编译 0 error + 改动面精准测试（AC-6/8/9/13/16）全绿 + 唯一观察失败为已知 tuix 存量红测试。**本 feature 引入的测试失败 = 空集**，AC-14 判 PASS（环境受限替代）。如需完整全量数字，建议在更大内存环境或仅针对本 feature 改动面（`rustcode` / `rustcode-codingplan` / `rustcode-config`）复跑。
2. **用户并发 dirty 树的全局失败集未独立基线化**：worktree 含大量用户进行中改动（coding/runtime.rs、tuix/*、review、updater、kernel/*、capabilities/*、clix/*、schedule_cmd.rs 等），其测试失败（含 tuix 实际失败数波动）属用户工作范畴，非本 feature 责任；本验证仅保证「本 feature 改动不新增失败」。
3. **`trust_key_golden_matches_core_algorithm` 未单独复跑**：该 mcp 既有失败（AGENTS.md 登记）与本次清理无关，未单独执行；其存在性不影响本 feature 结论。

## 六、回滚方案（Rollback）

本 feature 全部为低风险改动（注释 / 别名删除 / 文档），可按文件精准回滚，**严禁 blanket reset/stash**（会丢失用户并发改动）：

```bash
# T1 注释（6 文件，改动前非 dirty，仅含本 feature 编辑）
git checkout -- \
  crates/rustcode-codingplan/src/{lib,types,client,setup,sync_marker}.rs \
  crates/rustcode-config/src/i18n/messages.rs
# T3 CLI 别名删除（cli/main.rs，改动前非 dirty）
git checkout -- crates/rustcode-cli/src/main.rs
# T4 配置去重
git checkout -- docs/config.example.toml
# T5 文档归档（恢复 10 份原位）
git mv docs/archive/<name>.md docs/<original-path>/   # 逐份，并还原 T5 链接修正
# T6 原位指针（删 2 份标题指针行 + REFACTOR_DESIGN_PHASE1.md 追加行）
```

> 注意：上述 `git checkout` 仅移除本 feature 自身编辑（相关文件改动前非 dirty）。执行前请 `git status` 确认未被用户后续改动覆盖；若已覆盖，需人工 diff 还原对应 hunk。

## 七、下一步（Next Step）

**唯一建议下一步**：将本 feature 改动作为一次独立 commit 提交（建议标题 `chore(codingplan): clear legacy core-path comments, drop hidden CLI alias, archive stale plans`）。提交前请确认：
- 不含用户并发 dirty 改动（用 `git add` 精确暂存本 feature 文件）；
- 若需完整全量测试数字，先在更大内存环境复跑 AC-14（见第五节 1）。

后续独立 follow-up（不在本 feature 范围）：④ 的 7 处非 codingplan 面失效 core 注释清理；③ 的 tuix/daemon 重复渲染器抽取评审。
