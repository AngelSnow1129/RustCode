# AtomGit 生产工具实施计划

> **致 Codex：** 在当前工作区按任务逐条执行本计划，同时保留无关的用户改动。

**目标：** 把内置的 AtomGit 仓库、拉取请求与议题工具暴露给生产环境的 coding runtime，并引导模型远离携带明文凭据的原始 `curl` 调用。

**架构：** AtomGit REST 的所有权保留在 `rustcode-capabilities`。新增一个 coding 层注册辅助函数，由最小 agent builder 与生产 `prepare → assemble` 路径共用，使两者发布同一份工具目录。仅在编译 `atomgit` feature 时追加提示词引导，并校验生产 `CodingParts` 目录，而不只校验底层注册表。

**技术栈：** Rust、rustcode-kernel 工具注册表、rustcode-coding 两阶段 runtime 组装、Cargo 测试。

---

### 任务 1：复现生产目录缺口

**文件：**
- 修改：`crates/rustcode-coding/src/parts.rs`

1. 新增一个 feature 门控的单元测试，准备生产 `CodingParts`。
2. 断言 `atomgit_repo`、`atomgit_pr` 与 `atomgit_issue` 已被选中。
3. 运行该聚焦测试，确认它因名称缺失而失败。

### 任务 2：共用 AtomGit 工具注册

**文件：**
- 修改：`crates/rustcode-coding/src/assemble.rs`
- 修改：`crates/rustcode-coding/src/parts.rs`

1. 抽出一个 feature 门控的辅助函数，创建 AtomGit 客户端、注册全部三个工具并追加其名称。
2. 若客户端构造失败则显式报错；不得静默发布不完整的目录。
3. 最小 builder 与生产准备流程都复用该辅助函数。
4. 运行聚焦的生产目录测试。

### 任务 3：优先使用专用 AtomGit 工具

**文件：**
- 修改：`crates/rustcode-coding/src/persona.rs`

1. 新增 feature 门控的提示词引导，要求使用 `atomgit_repo`、`atomgit_pr` 与 `atomgit_issue`，而不是读取鉴权文件或拼装原始 API `curl` 命令。
2. 新增覆盖该引导的 persona 测试。
3. 运行聚焦的 persona 测试。

### 任务 4：验证受影响的 crate

1. 对改动的 Rust 文件运行格式检查。
2. 运行 `cargo test -p rustcode-coding --features atomgit`。
3. 检查最终 diff，确认无关的脏文件未被触碰。
