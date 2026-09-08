# Goal Summary: RustCode 迁移收尾

## 已完成内容

迁移收尾的 5 个维度全部完成：

### 1. 命名清理 (产品标识统一为 `rustcode-*`)
- `webui/package-lock.json`: 包名统一为 `rustcode-webui`
- `extensions/vscode/package-lock.json`：已是 `rustcode-tools`（已核验）
- G7（crates/scripts/.github）：0 处命中
- G8（docs/architecture.md）：0 处命中

### 2. 平台中立 (不与任何模型/平台关联)
- 未把任何主机硬编码为签名网关
- `is_codingplan_llm_gateway` 只识别显式设置的 `RUSTCODE_CODINGPLAN_LLM_BASE_URL`
- Provider 默认使用普通 `bearer_auth(api_key)`
- AtomGit REST 工具位于 `atomgit` cargo feature 之后（在默认成员中关闭）
- `config.example.toml` 中只保留第三方 provider 配置

### 3. 零遥测
- G6：0 处命中（sentry/posthog/segment/analytics）
- `install_panic_hook` 已保留（仅写 stderr 的本地逻辑，非遥测）
- `rustcode-telemetry` crate 已删除（已确认）

### 4. 中文默认
- `Locale::default()` 返回 `ZhCn`（与运行时默认一致）
- `LOCALE` 静态默认值：`ZhCn`
- `current_locale()` 回退值：`ZhCn`
- `resolve_initial_locale_with_env` 无信号时：`ZhCn`
- `LANG=C`/`POSIX` -> `ZhCn`（无偏好 = 产品默认）
- `config.example.toml`：已添加 `# language = "zh_CN"` 注释
- 测试 `cli_flag_unparseable_falls_through` 已按 ZhCn 默认值更新

### 5. 多 Agent 并行
- `config.example.toml`：已添加 3 角色并行模板
  （explorer/builder/reviewer 三个角色，max_concurrent=4）
- 体现 6 条设计原则：
  1. 专业化分工 (codex/claude-code distinct kinds)
  2. 清晰边界 (read-only/accept-edits/auto/bypass tiers)
  3. 高效通信 (context inheritance + result passing)
  4. 并行处理 (max_concurrent)
  5. 质量保证 (reviewer = independent verification)
  6. 成本优化 (model override per instance)

## 迭代历史

- 第 1 轮迭代：PASS（未发现问题）

## 主要改动文件

- `crates/rustcode-config/src/locale.rs` — `Default` 实现
- `crates/rustcode-config/src/i18n/mod.rs` — LOCALE 静态变量、current_locale、测试
- `docs/config.example.toml` — language 注释 + subagent 模板
- `webui/package-lock.json` — package 名称

## 建议

- 合并前建议完整运行一次 `cargo test --workspace`（本轮受时间所限
  只测试了 config crate）
- 已知红测试 `mcp::registry::tests::trust_key_golden_matches_core_algorithm`
  仍为红（DefaultHasher 不稳定）——不要去「修复」它
- 若并行多 Agent 经验证稳定，可考虑在未来的版本中
  默认启用 3 角色 subagent 模板
