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
| T-03 | 修 debugging description 转义引号残留 | B3 | 编排者 | done | 0 | 05-test-report-batch2.md（OBS-B2-1 复验） |
| T-04 | 重写 rustcode-lsp-setup（含 config 键名阻塞项） | B4 | doc-writer | done | 0 | 03-impl/T-04.md |
| T-05 | 重写 rustcode-rust-standards（AGENTS.md 冲突比对） | B4 | doc-writer | done | 0 | 03-impl/T-05.md |
| T-06 | 第二批加载验证 + LSP 键名独立核实 | B5 | test-engineer | done | 0 | 05-test-report-batch2.md |

## 事实更正（T-04 推翻前序假设）

前序方案称「config.toml 的 LSP schema 是 `coding/src/config.rs:140` 的 `LspSettings`」——**不成立**。
`LspSettings` 是运行时结构；真实 config schema 是 `crates/rustcode-config/src/config/mod.rs:1538` 的
`LspConfig`，经 `coding/src/config.rs:232` 的 `lsp_settings_from_config()` 映射。
四项声明已由 test-engineer 与编排者**各自独立回源**，4/4 属实：
`[lsp]` 段 `enabled`/`auto_detect`/`servers`/`diagnostics_settle_delay_ms`；
`[lsp.servers.<ext>]` 按**文件扩展名**键控（内置表用 `rs`）；`settle_delay_ms` ↔ `diagnostics_settle_delay_ms`；
`migrate_legacy_lsp_default`（`mod.rs:1594`，调用点 `:1955`）在「enabled && auto_detect && delay==150 && servers 空」
四条同时成立时把整段重置为全关 —— 故最小配置组合会静默失效，技能正文推荐 `300` 规避。

## 合规声明

oh-my-openagent 采用 SUL-1.0（非 OSI 开源、不可再许可/不可转让）。本特性**未复制任何其文本**，
仅借鉴流程骨架；6 个技能正文为 RustCode 原创。仓库侧零 SUL 内容，材料只落到 `~/.rustcode/skills/`（用户目录）。

## T-03 验证

- `grep -rn '\\"' */SKILL.md` → 无命中（exit 1）
- 复刻 `fm_value` 解析 6 个 description → 6/6 无反斜杠且非空
- T-06 复验：8/8 加载正常（OBS-B2-1）

## 第二批（B4/B5）结论

- 新增 `rustcode-lsp-setup`（190 行）、`rustcode-rust-standards`（176 行），累计 **8 个技能**。
- V1 真实加载 8/8（`reload()` 路径同样 8 个）；V2 回归 49/49；V3 工具名 16/16 命中；
  V4 门禁 PASS；V5 LSP 键名 4/4 属实。
- 已定级 17 个中：采纳并重写 8、需改造后可行 1（data-scientist，本仓无使用场景暂不立项）、放弃 8。

## 遗留

- `lsp` 工具本身未实机验证（需写用户 config + 重启 runtime），本批次不动用户环境。
- `allowed-tools` 是否被 L2 审批策略消费未确认（`skill.rs:17` 注明为元数据，L1 不强制）。
- `skills` 是 opt-in Cargo feature（不在 default），验证必须带 `--features skills`。
- 技能正文对 `AGENTS.md` 的行号引用会随其修订漂移，结构变动时需同步复核。
- `frontend`（2.9MB / 24 个 CSV）、`programming` 非 rust 子集、`visual-qa` / `ultimate-browsing`
  均未引入；第三方 vendored 许可链未做法律澄清。
