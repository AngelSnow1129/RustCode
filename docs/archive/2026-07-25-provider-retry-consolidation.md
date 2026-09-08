# Provider 重试收敛实施计划

> **说明：** 必需子技能：使用 superpowers:executing-plans 逐任务实现本计划。

**目标：** 把 HTTP 429 重试的所有权收敛到 kernel，使瞬时 RPM/TPM 或网关限流以可取消的有界退避重试，同时不会放大 provider 重试或重放已部分流式输出的内容。

**架构：** 重试所有权按每次 provider 调用选择。直接调用方（compaction、vision、title、evaluation）保留有界的 provider 自有 429 重试，而 kernel turn loop 把自己的调用标记为 kernel 所有，使第一个 429 带上结构化的 status、code、body 与 `Retry-After` 上报。kernel 持有该调用的事故预算，并在以下策略间选择：按服务端指示等待、带抖动的回退指数退避、需确认的 CodingPlan 长时间配额暂停、终态计费/余额错误，以及五次等待熔断。mid-stream 429 只在任何用户可见的模型内容已发出之前重试；可见的部分输出会在干净的限流终态之前先落盘持久化。

**技术栈：** Rust、Tokio、reqwest、rustcode-kernel lifecycle hooks、rustcode-capabilities provider adapters、rustcode-coding CodingPlan hook。

---

### 任务 1：把 provider 的可重试判定与 provider 自有的重试执行分离

**文件：**
- 修改：`crates/rustcode-capabilities/src/provider/retry.rs`
- 修改：`crates/rustcode-capabilities/src/provider/openai_compat.rs`
- 修改：`crates/rustcode-capabilities/src/provider/anthropic.rs`
- 修改：`crates/rustcode-capabilities/src/provider/ollama.rs`

**步骤：**

1. 补充测试，证明 429 仍可重试，且其 OPEN 循环行为遵循每次调用的归属方。
2. 在 `ChatOptions` 上增加一个仅运行时的重试归属方；直接调用默认为 provider 所有，而 kernel turn loop 会覆盖为 kernel 所有。
3. 将三个 provider 的 OPEN 循环全部切换到感知归属方的辅助函数。
4. 保持最终的 429 错误仍标记为 `retryable`，并为 kernel 保留完整的 `Retry-After` 与 provider 错误数据。
5. 运行 `cargo test -p rustcode-capabilities provider::retry --lib`。

### 任务 2：为未知瞬时 429 提供有界的 kernel 退避

**文件：**
- 修改：`crates/rustcode-kernel/src/hook.rs`
- 修改：`crates/rustcode-kernel/src/agent.rs`
- 修改：`crates/rustcode-kernel/src/testkit.rs`
- 修改：`crates/rustcode-kernel/tests/rate_limit.rs`
- 修改：`crates/rustcode-coding/src/rate_limit.rs`

**步骤：**

1. 补充缺少 `Retry-After` 时的策略测试：按事故尝试次数分别取 3、6、12、24、48 秒基准，并带 ±25% 抖动。
2. 为 `RateLimitHint` 扩展一个从 1 起算的事故尝试次数。
3. 让 `RateLimitDecision::from_hint` 遵循 `Retry-After <= 120s`，对更长的服务端指定等待执行暂停，对已知的计费/余额错误立即停止，并在缺少该响应头时使用有界抖动的回退退避。
4. 在 OPEN 与 mid-stream 两条路径上，都由按回合持有的限流计数器填充该尝试次数。
5. 更新 CodingPlan 与 testkit 的构造函数。
6. 运行 kernel 与 coding 的限流测试。

### 任务 3：防止部分流重放

**文件：**
- 修改：`crates/rustcode-kernel/src/agent.rs`
- 修改：`crates/rustcode-kernel/tests/rate_limit.rs`

**步骤：**

1. 新增一个失败测试：text delta 之后出现 429，并验证不会再发起第二次 provider 请求。
2. 仅当尚未发出任何 text、reasoning、reasoning signature 或 tool call 时，才允许 mid-stream 429 自动重试。
3. 先持久化已可见的部分输出，再发出一个干净的 `RateLimited` 终态并带上服务端原因。
4. 保持可取消的等待与单一终态不变式。
5. 运行 `cargo test -p rustcode-kernel --test rate_limit`。

### 任务 4：跨层验证

**文件：**
- 复查：所有被修改的文件

**步骤：**

1. 运行聚焦的 provider、kernel 与 coding 测试。
2. 运行 `cargo test -p rustcode-capabilities -p rustcode-kernel -p rustcode-coding --lib`。
3. 运行 `cargo check --workspace --all-targets`。
4. 运行 `git diff --check`，并检查最终 diff 是否存在重试放大、取消与终态回归。
