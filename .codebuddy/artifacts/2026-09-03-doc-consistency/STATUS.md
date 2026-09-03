# 2026-09-03-doc-consistency 看板

- 当前阶段：分析（Phase 1 并行扫描中）
- 基线：branch=dev commit=4c9059dc worktree=dirty（仅含本任务产物目录 + 前序 banner-release 的 03-impl/T2.md 未跟踪文件）
- 前序：`2026-09-03-banner-release`（已推送 C1–C4，dev=3d8466c0→ 现 HEAD 4c9059dc）

## 用户诉求（逐字 + 解读）

> 多 agent 分析和优化相应的文档等内容

经澄清确认：
1. 范围 = **全仓文档一致性排查**（不限于旧名，含事实准确性 / 术语一致 / 与代码对得上 / 链接有效 / ASCII 标签合规）
2. 动作 = **分析 + 直接优化修改**（受保护文件 `AGENTS.md` 仍需单独授权；`LICENSE` / `UPSTREAM_*` / `THIRD_PARTY_NOTICES` / `ORIGINAL_LICENSE` 合规归属原文须保留）

## 已知背景（来自前序轮次）

- 用户可见 `atomcode` 残留（README、`docs/{features,platform-neutralization}.md`）已由第三十二轮清掉。
- `AGENTS.md` 中仍记载「cgroup 内存上限 8GB」「rustup 恢复前构建须走工具链绝对路径 + PATH 注入」——上轮多平台构建实测 252GB 可用、rustup 已恢复，这两条已证伪，属**事实过期**（待授权后修）。
- `.md` 现存 `atomcode` 多为：合规文件保留项、AGENTS.md 映射表/门禁/历史日志、历史问题描述 → 默认 LEAVE，不误删。

## 门禁

| 门禁 | 状态 | 依据 |
|---|---|---|
| G1 需求 | pass | 范围与动作已澄清（双选问答记录） |
| G2 设计 | **pass** | Phase 1 四域扫描完成，已收敛任务图（T1–T8 + 3 个 flagged 项） |
| G3 实现 | **pass** | T1–T8 已落地（doc-writer-A/B + 编排者），含 REFACTOR_SUMMARY.md:12 补修 |
| G4 审查 | **pass** | 独立 grep 复核：`cargo ... -p rustcode-cli` 仅余正确语境 + F1 规划文档；`site/index.html` `.atom` 0 命中 |
| G5 测试 | **pass** | 静态核验：`bash -n scripts/linux-release-linux.sh`=SYNTAX_OK；grep 自验通过；无源码改动 |
| G6 交付 | **pass** | 用户授权「提交并推送 dev」+「扩大修 F1」；F2(AGENTS.md) 未授权维持原样，F3(rustcodex CI) 维持现状 |

## Phase 1 并行扫描域（code-explorer × 4）

| 域 | 范围 | 负责 | 状态 |
|---|---|---|---|
| D1 | 根文档：AGENTS.md / CONTEXT.md / README.md / README.zh-CN.md | code-explorer | in_progress |
| D2 | docs/ 设计与架构文档 | code-explorer | in_progress |
| D3 | site/ 网页文档 + extensions/ 文档 + packages/ | code-explorer | in_progress |
| D4 | .codebuddy/agents/*.md + scripts/*.sh + .github/workflows/* | code-explorer | in_progress |

## 任务（Phase 2 修复）

| id | 标题 | 域 | 负责 | 状态 | 严重度 |
|---|---|---|---|---|---|
| T1 | README/README.zh-CN `-p rustcode-cli`→`-p rustcode`（8 处，blocker） | D1 | doc-writer-A | ready | blocker |
| T2 | `site/index.html` `.atom` CSS 类→`.rustcode`（3 处） | D3 | doc-writer-A | ready | low |
| T3 | `docs/features.md:4` 链接 `CONTEXT.md`→`../CONTEXT.md` | D2 | doc-writer-A | ready | low |
| T4 | `docs/REFACTOR_SUMMARY.md` 5 处事实过期（重命名方向/数量/env/telemetry_legacy/ORIGINAL_LICENSE） | D2 | doc-writer-B | ready | medium |
| T5 | `docs/phase1-refactor-design.md` `docs/licenses/` 虚假现状断言 + X2 + L29 + L677 | D2 | doc-writer-B | ready | medium |
| T6 | `docs/REFACTOR_DESIGN_PHASE1.md:797-828` `-p rustcode-cli`×4 | D2 | doc-writer-B | ready | low |
| T7 | `docs/phase2-subagent-status.md:13` AGENT-A [PENDING]→对齐 [DONE] | D2 | doc-writer-B | ready | low |
| T8 | `scripts/linux-release-linux.sh:9` 版本来源统一为 Cargo.toml | D4 | 编排者 | done | medium |

## 验证汇总（G4/G5 依据）

- `grep -rn "cargo (run|build|test|check) -p rustcode-cli"` 全仓仅余：
  - `crates/rustcode-clix/README.md:7`、`phase2-subagent-status.md:44` → `-p rustcode-clix`（**正确**，clix 包名即 rustcode-clix）
  - `AGENTS.md:236`、`platform-neutralization.md:119` → 解释性语境（描述「已修正为 -p rustcode」），**保留**
  - `docs/plans/*.md`（×4）→ **F1 历史规划文档，待裁决**
- `grep -n "\.atom" site/index.html` → 0 命中（已统一 `.rustcode`）
- `bash -n scripts/linux-release-linux.sh` → SYNTAX_OK
- 未触碰：AGENTS.md / LICENSE / docs/UPSTREAM_* / THIRD_PARTY_NOTICES / ORIGINAL_LICENSE / .codebuddy/agents/ / 合规归属声明中的 atomcode

## Flagged（待裁决 / 待授权，本轮不擅自改）

| 项 | 说明 | 处置 |
|---|---|---|
| F1 | `docs/plans/*.md`(×4) 仍含 `-p rustcode-cli` 错误命令（archive 那处实为 `-p rustcode-clix`，正确） | **done**（用户授权扩大范围，已修 4 处） |
| F2 | `AGENTS.md` 事实过期：cgroup 8GB 上限、`rustup` 恢复前「工具链绝对路径+PATH 注入」变通已不需；`:52` 路径简写 `coding/src/runtime.rs` | 受保护文件，需你单独授权后修 |
| F3 | `rustcodex`(clix) 未纳入 `scripts/*.sh` 与 `.github/workflows` 构建步骤 | 疑似有意（独立流水线），待你确认是否漏建 |
