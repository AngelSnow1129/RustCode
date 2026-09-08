# 2026-09-09-omo-skills-import 看板

- 当前阶段：交付（G1–G5 已过，G6 完成）
- 基线：branch=dev commit=ee96e1e4 worktree=dirty（仅 README.md 改动 + 未跟踪 artifacts 目录，均为本特性产出）

## 门禁

| 门禁 | 状态 | 依据 |
| G1 需求 | pass | 用户裁决：不复制 SUL-1.0 原文，按 RustCode 方式重写等价技能；纯运行时导入（零代码） |
| G2 设计 | pass | 工具映射表冻结（call_omo_agent→task、lsp_diagnostics→bash+cargo check/clippy、background_output→task 直返、session_*→原生会话命令） |
| G3 开发 | pass | 03-impl/T-01.md |
| G4 审查 | pass | 静态校验 6/6 + 工具名真实性 13/13 |
| G5 测试 | pass | 05-test-report.md（72 用例 0 失败） |
| G6 交付 | pass | 06-release.md + 01-import-plan.md |

## 任务

| id | 标题 | 批次 | 负责 Agent | 状态 | 返工轮次 | 交接件 |
| T-01 | 重写 6 个技能 + 17 个技能定级方案 | B1 | doc-writer | done | 0 | 03-impl/T-01.md |
| T-02 | 运行侧加载验证 + 回归 + 门禁复核 | B2 | test-engineer | done | 0 | 05-test-report.md |
| T-03 | 修 debugging description 转义引号残留 | B3 | 编排者 | done | 0 | 见下方验证 |

## 合规声明

oh-my-openagent 采用 SUL-1.0（非 OSI 开源、不可再许可/不可转让）。本特性**未复制任何其文本**，
仅借鉴流程骨架；6 个技能正文为 RustCode 原创。仓库侧零 SUL 内容，材料只落到 `~/.rustcode/skills/`（用户目录）。

## T-03 验证

- `grep -rn '\\"' */SKILL.md` → 无命中（exit 1）
- 复刻 `fm_value` 解析 6 个 description → 6/6 无反斜杠且非空

## 遗留

- README.md 的账号命令表述修正（README:100、493-501）已过门禁但**未提交**，等待用户确认。
- `lsp-setup` 定级需按实测修正为「需改造后可行」（rustcode-coding 默认带 `lsp` feature）。
- `frontend`（2.9MB / 24 个 CSV）、`programming`（848KB）重资产未引入，第三方许可链未做法律澄清。
