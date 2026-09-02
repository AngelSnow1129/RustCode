# Goal: RustCode 迁移收尾 — 命名/遥测/中文默认/多Agent

## Goal

完成 RustCode fork 的迁移收尾工作,覆盖四个维度:
1. **命名清理** — 产品标识统一为 `rustcode-*`,package-lock.json 与历史文本口径已收尾
2. **平台中立** — 确认不与任何模型/平台关联,只保留第三方 provider 配置
3. **零遥测** — 确认无遥测 SDK 残留,`install_panic_hook` 是本地逻辑不删
4. **中文默认** — 使 `Locale::default()` trait 与运行时产品默认(zh_CN)一致
5. **多 Agent 并行** — 在 `config.example.toml` 提供 3 角色并行模板

## Acceptance Criteria

1. **G7 命名**: `grep -rn "atomcode" crates/ scripts/ .github/` = 0 hits
   (已是 0,保持)
2. **G8 命名**: `grep -rn "atomcode" docs/architecture.md` = 0 hits
   (已是 0,保持)
3. **package-lock.json**: `webui/package-lock.json` 和
   `extensions/vscode/package-lock.json` 的顶层 `"name"` 字段为
   `rustcode-*`(package.json 已是 rustcode-*,只需重生成 lock 或手动改 name)
4. **G6 遥测**: `grep -ri "sentry|posthog|segment|analytics" --include=*.rs --include=*.toml`
   = 0 hits(已是 0,保持)。`install_panic_hook` 保留(stderr 本地逻辑)
5. **Locale::default()**: `crates/rustcode-config/src/locale.rs` 的
   `impl Default for Locale` 返回 `Locale::ZhCn`,与运行时产品默认一致
6. **Locale 测试**: `config_default_has_no_language` 等相关测试仍通过
   (`Config.language` 字段仍 `None`,auto-detect;只改 `Locale::default()`)
7. **config.example.toml**: 添加 `# language = "zh_CN"` 注释行(默认中文)
8. **config.example.toml**: 添加 `[[subagent.external]]` 3 角色并行模板
   (explorer/builder/reviewer),`max_concurrent = 4`
9. **G1 fmt**: `cargo fmt --check` 通过
10. **G3 test**: `cargo test --workspace` 通过(已知红测试
    `mcp::registry::tests::trust_key_golden_matches_core_algorithm` 除外)

## Scope Boundaries

### In scope

- `webui/package-lock.json`, `extensions/vscode/package-lock.json` 的 name 字段
- `crates/rustcode-config/src/locale.rs` 的 `Default` impl
- `docs/config.example.toml` 添加注释和模板
- 相关测试更新(如果 Locale::default() 改动导致)

### Out of scope

- 不改 `Config.language` 字段默认值(保持 `None` = auto-detect)
- 不改 `LOCALE` 静态默认值(已是 ZhCn)
- 不改 `resolve_initial_locale` 逻辑(已正确)
- 不删 `install_panic_hook`(本地逻辑,非遥测)
- 不改 `site/` 和 `extensions/` 的二进制名(非 Rust 工作区,不阻塞编译)
- 不改 README/CLAUDE/AGENTS 中的 fork 声明历史引用(合法历史名引用)
- 不改 `default_subagent_level()`(便捷开关保持 off,用 external 模板)

## Project Conventions

- **No Unicode emoji** — logs/comments/UI use ASCII tags
  (`[INFO]`/`[WARN]`/`[ERROR]`/`[SUCCESS]`/`[CHECK]`/`[+]`/`[-]`/`[*]`)
- **Errors**: thiserror for module-level, anyhow at top; no bare
  `unwrap()`/`expect()` outside tests
- **Test isolation**: `#[ctor]` redirects `RUSTCODE_HOME` to temp dir
- **Package-name gotcha**: `crates/rustcode-cli/` has package name `rustcode`
  (use `-p rustcode`, NOT `-p rustcode-cli`)
- **Commit convention**: conventional commits, `type(scope): description`
- **Product identity locked**: `rustcode` (decision D1, do not rename again)

## Quality Gates

- G1: `cargo fmt --check`
- G3: `cargo test --workspace`
- G6: `grep -ri "sentry|posthog|segment|analytics" --include=*.rs --include=*.toml` = 0
- G7: `grep -rn "atomcode" crates/ scripts/ .github/` = 0
- G8: `grep -rn "atomcode" docs/architecture.md` = 0

## Initial SHA

`af7e8715723aa33343f049b15d3032e71200567c` (dev branch)
