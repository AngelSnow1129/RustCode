# 轻量 LSP 一期实施计划

本计划记录第一个只读 LSP 阶段的实现边界与验证顺序。

**目标：** 增加一个可选启用的只读 `lsp` 模型工具，提供 definition、references、
hover 与 diagnostics 四类操作；它只在首次使用时拉起本地已安装的语言服务器，
并且在不可用时降级而不让本轮失败。

**架构：** `rustcode-capabilities` 持有中立的 LSP 协议客户端、工作区作用域的
管理器与工具。`rustcode-coding` 把 `[lsp]` 配置映射为中立的设置 DTO，并向
运行时持有的工具注册表注入一个由管理器支撑的工具。kernel 与各驱动对 LSP
进程保持无感知；丢弃运行时持有的工具图即可终止已派生的子进程。

**技术栈：** Rust、Tokio stdio 进程、LSP JSON-RPC、serde/serde_json、既有的 RustCode kernel 工具 API。

---

### Task 1：定义协议查询并配套确定性测试

**文件：**
- 修改：`crates/rustcode-capabilities/src/codeintel/lsp/client.rs`
- 修改：`crates/rustcode-capabilities/src/codeintel/lsp/types.rs`

**步骤：**
1. 扩展注入的 mock LSP 传输层，使其能应答 hover、definition 与 references 请求。
2. 增加失败测试，断言线上从零开始的位置与归一化后的结果。
3. 增加带请求超时上限的只读客户端查询方法。
4. 运行 `cargo test -p rustcode-capabilities --features lsp codeintel::lsp::client`。

### Task 2：让管理器具备工作区安全性与容错能力

**文件：**
- 修改：`crates/rustcode-capabilities/src/codeintel/lsp/manager.rs`
- 修改：`crates/rustcode-capabilities/src/codeintel/lsp/registry.rs`

**步骤：**
1. 增加测试，证明客户端/失败是按归一化后的工作区根加语言来区分的，
   而不是仅按扩展名。
2. 增加启动超时，并缓存不可用/损坏的服务器结果，以避免重试循环。
3. 暴露用于文档同步与三类语义查询的管理器方法。
4. 在服务器未配置、缺失、启动失败或超时时保持优雅降级。
5. 运行管理器测试模块。

### Task 3：新增统一的只读 `lsp` 工具

**文件：**
- 新建：`crates/rustcode-capabilities/src/codeintel/lsp_tool.rs`
- 修改：`crates/rustcode-capabilities/src/codeintel/mod.rs`
- 修改：`crates/rustcode-capabilities/src/codeintel/diagnostics.rs`

**步骤：**
1. 为 `definition`、`references`、`hover` 与 `diagnostics` 增加失败测试，
   覆盖 schema、校验与降级。
2. 实现一个需要 `file_path` 的 `lsp` 工具；语义类操作还要求从 1 开始计数的
   `line` 与 `character`。
3. 把位置归一化为项目相对路径的 `file:line:column`，限制输出体积，
   并保留诊断严重级别的过滤。
4. 仅把 `DiagnosticsTool` 保留为既有嵌入方的兼容门面；不要在新建的编码
   运行时中自动注册或挂载它。
5. 保持常规 Tree-sitter/文本代码智能的注册独立于 LSP 注册。
6. 用 `--features lsp` 运行全部 codeintel 测试。

### Task 4：把配置接入运行时所有者

**文件：**
- 修改：`crates/rustcode-capabilities/Cargo.toml`
- 修改：`crates/rustcode-coding/Cargo.toml`
- 修改：`crates/rustcode-coding/src/config.rs`
- 修改：`crates/rustcode-coding/src/assemble.rs`
- 修改：`crates/rustcode-coding/src/parts.rs`

**步骤：**
1. 增加测试，把 `Config.lsp` 映射为中立的 LSP 设置 DTO。
2. 只通过编码装配层启用 capabilities 的 `lsp` 构建特性。
3. 仅当 `enabled=true` 时才注册并挂载 `lsp`；仅当 `auto_detect=true` 时才合并
   内建服务器，且显式配置的服务器覆盖内建项。
4. 确保 provider 重载/会话重建路径使用同一份 `CodingAgentConfig` 策略。
5. 运行编码配置与装配相关测试。

### Task 5：验证与审计

**文件：**
- 复查上述所有文件；不要把无关的脏工作区文件包含进来。

**步骤：**
1. 运行 `cargo fmt --all -- --check`，必要时只格式化本次改动。
2. 运行 `cargo test -p rustcode-capabilities --features lsp codeintel`。
3. 运行 `cargo test -p rustcode-coding`。
4. 若共享运行时配置改变了 daemon/CLI 配置测试的编译面，则运行针对性的测试。
5. 检查 `git diff`，关注生命周期所有权、缺失的终态、意外的默认开启行为
   与无关改动。
