# 原生运行时数据日志实施计划

> **给 Claude：** 必需子技能：使用 superpowers:executing-plans 逐任务实施本计划。

**目标：** 在原生 `CodingRuntime` 路径上恢复已配置的每轮 Markdown 数据日志
与每轮 LLM 请求的 JSONL 日志。

**架构：** 在 `rustcode-capabilities` 中增加一个 provider 中立的观察者，同时实现
`LifecycleHooks` 与 `ToolMiddleware`，并由 `rustcode-coding::assemble`（CLI、TUI、
daemon、ACP 与 clix 共用的生产装配边界）挂载同一个实例。保留项目分桶与
Markdown/JSONL 成对的布局，同时不恢复 `rustcode-core`，也不引入第二个运行时所有者。

**技术栈：** Rust、`rustcode-kernel::LifecycleHooks`、`rustcode-config::DatalogConfig`、serde JSON、SHA-256。

---

## 设计

`DatalogHook` 只持有当前轮次的观测状态。`user_prompt_submit` 缓冲最终重写后的
提示。首次 `on_request` 会创建一个防冲突的成对文件，其名称包含会话/轮次/进程/
实例身份，把最终的中立 kernel 请求（`messages`、`tools`、`ChatOptions`、`TurnCtx`）
作为一条紧凑的 JSONL 记录追加，并向 Markdown 文件追加一段请求摘要。
`on_model_response` 记录助手文本、推理、工具调用与用量。工具中间件会记录工具
的成功、失败与被拒绝结果，即使之后没有下一轮 LLM 请求。`on_error` 记录
provider/运行时错误。`turn_complete` 记录终态原因与耗时，并冲刷排队的写入。

配置根目录沿用历史规则：省略目录时使用 `$RUSTCODE_HOME/datalog`；`~/...`
按真实用户主目录展开；绝对路径保持固定；相对路径从运行时工作目录解析。
始终追加一个经过净化的项目基名加八位字符的稳定 SHA-256 后缀。所有建目录
与写入都是尽力而为：日志绝不能拒绝提示、改动请求、panic 或改变轮次终态。
文件工作跑在专用的单写入者线程上，Unix 下项目目录与文件权限为 `0700`/`0600`，
新成对文件用 `create_new` 并带防撞后缀。该 hook 不持有任何 provider、会话、
控制器或持久化的所有权。

`DatalogConfig` 通过 `CodingRuntimeConfig` 与 `CodingAgentConfig` 逐层传递。
该 hook 在 `assemble` 阶段而非 `prepare` 阶段构建，因此 provider/model 重建装配
会复用同一套运行时/会话生命周期，同时记录当前生效的模型。日志关闭时不挂载
任何 hook，也不创建任何文件。

## Task 1：增加中立的数据日志 hook

**文件：**
- 新建：`crates/rustcode-capabilities/src/datalog.rs`
- 修改：`crates/rustcode-capabilities/src/lib.rs`

1. 增加失败测试，覆盖日志关闭、路径解析、项目名冲突规避、
   多轮 JSONL、Markdown 中的响应/错误内容，以及终态冲刷。
2. 实现最小可用的 `LifecycleHooks` 写入器。
3. 运行 `cargo test -p rustcode-capabilities datalog --lib`。

## Task 2：在公共编码边界挂载

**文件：**
- 修改：`crates/rustcode-coding/src/config.rs`
- 修改：`crates/rustcode-coding/src/parts.rs`
- 更新：编译器报出的显式 `CodingAgentConfig` / `CodingRuntimeConfig` 构造函数。

1. 把 `DatalogConfig` 从产品配置一路穿到 agent 装配。
2. 启用时挂载一个世代局部的 hook。
3. 增加一个装配测试，证明一次真实的 mock provider 轮次会同时创建两个文件。
4. 运行 `cargo test -p rustcode-coding datalog`。

## Task 3：跨入口验证

1. 运行 `cargo test -p rustcode-capabilities -p rustcode-coding --lib`。
2. 若构造函数变更波及 CLI、daemon、clix 与 TUI 这些消费方，则对它们跑编译检查。
3. 运行 `git diff --check`。
4. 检查最终 diff，确认没有改动与用户既有的 TUI/Cargo 工作区修改重叠。
