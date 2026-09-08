# TUI 配置面板实施计划
> **给 Claude：** 必需子技能：使用 superpowers:executing-plans 逐任务实施本计划。

**目标：** 增加一个可搜索的半屏 `/config` 面板，用于安全地编辑非 provider 设置，
并在无需重启 RustCode 的情况下应用受支持的改动。

**架构：** `rustcode-config` 持有一份与 UI 无关的设置目录，以及保留注释的
TOML 补丁事务。TUI 负责弹窗交互与渲染。影响运行时的提交依旧走既有的配置
修订/协调与 `CodingRuntime` 生命周期；该面板绝不制造第二个运行时所有者。

**技术栈：** Rust、`toml_edit`、crossterm、既有的 `ConfigStore`、弹窗渲染器与 `CodingRuntime`。

---

### Task 1：保留注释的配置事务

**文件：**
- 修改：`crates/rustcode-config/Cargo.toml`
- 修改：`crates/rustcode-config/src/store.rs`

1. 增加失败测试，证明一次标量补丁会保留注释、未知键与无关的 provider 文本。
2. 增加一个带锁的文档补丁 API，支持可选的修订匹配。
3. 在原子替换之前先解析并校验打过补丁的文档。
4. 运行 `cargo test -p rustcode-config store`。

### Task 2：与 UI 无关的设置目录

**文件：**
- 新建：`crates/rustcode-config/src/settings.rs`
- 修改：`crates/rustcode-config/src/lib.rs`

1. 增加目录测试，确认 model/provider/凭据类字段被排除在外。
2. 定义稳定的设置 ID、TOML 路径、值类型、默认值、搜索别名与应用策略。
3. 为受支持的标量设置增加带类型的读取/解析/补丁/重置辅助函数。
4. 运行 `cargo test -p rustcode-config settings`。

### Task 3：可搜索的半屏 `/config` 弹窗

**文件：**
- 新建：`crates/rustcode-tuix/src/modals/config_panel.rs`
- 修改：`crates/rustcode-tuix/src/modals/mod.rs`
- 修改：`crates/rustcode-tuix/src/event_loop/commands.rs`
- 仅在既有菜单框架需要配置专用变体时修改 `crates/rustcode-tuix/src/render/*`。

1. 增加状态测试，覆盖搜索、导航、布尔开关、枚举循环、编辑校验与重置确认。
2. 以 `/resume` 风格的半屏框架渲染各项设置，并标出当前值/默认值/已修改标记。
3. 把旧的 `/config` 帮助输出替换为安装弹窗。
4. 运行聚焦的 TUI 弹窗测试。

### Task 4：保存、应用、回滚与并发编辑

**文件：**
- 修改：`crates/rustcode-tuix/src/modals/config_panel.rs`
- 修改：`crates/rustcode-tuix/src/event_loop/mod.rs`
- 修改：`crates/rustcode-tuix/src/event_loop/commands.rs`

1. 每次编辑都基于最新文档经 `ConfigStore` 提交。
2. 复用既有的持久化配置协调机制处理 UI/下一轮/运行时重建准备类改动。
3. 应用失败时，仅当已提交的修订仍是最新时才回滚；否则协调更新的磁盘状态。
4. 增加测试，覆盖并发的无关编辑与应用失败后的回滚。

### Task 5：本地化、文档与验证

**文件：**
- 修改：按需修改 TUI/config 的 i18n 资源
- 修改：`site/docs/en/keybindings.html`
- 修改：`site/docs/zh/keybindings.html`
- 修改：配置参考文档

1. 增加本地化的标签、帮助、校验与应用状态文案。
2. 文档化 `/config`、被排除的 provider/model 字段、重置语义与需重启标记。
3. 运行受影响 crate 的测试，检查 `git diff --check`，并报告任何未经验证的
   交互式终端行为。
