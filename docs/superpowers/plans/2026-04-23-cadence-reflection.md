# Cadence Reflection 实施计划

> **面向 agentic worker：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 按任务逐步实施本计划。步骤使用复选框（`- [ ]`）语法进行跟踪。

**目标：** 给 rustcode 的 agent loop 加入周期性反思 checkpoint —— 每 N 次 tool call 后，在下一个 turn 开始前注入一段语言中立的 "restate goal / what ruled out / next concrete output" 提示，防止长尾任务方向漂移。

**架构：** 复用现有 `apply_post_turn_discipline` 钩子。新增两个纯函数：`should_inject_reflection(current, last, cadence)` 决定是否注入，`reflection_prompt(delta)` 渲染提示文本。触发条件 = `tool_call_count - last_reflection_at_tool_count >= cadence`。注入通过 `conversation.add_user_message` 完成，并更新标记。cadence 可配置（`Config.reflection_cadence: usize`，默认 10，0 禁用）。AgentLoop 层的集成只是 glue，靠类型系统保证，测试集中在两个纯函数。

**技术栈：** Rust, 现有 `AgentLoop` / `DisciplineState` / `Conversation` / `Config` / CLI — 无新依赖。

---

## 文件结构

- 修改：`crates/rustcode-core/src/config/mod.rs` — `Config` 加 `reflection_cadence: usize` 字段（serde 默认 10）。
- 修改：`crates/rustcode-core/src/agent/mod.rs` — `DisciplineState` 加 `last_reflection_at_tool_count: usize` 字段；新 task 开始时与 `tool_call_count` 一同重置。
- 修改：`crates/rustcode-core/src/agent/discipline.rs` — 加 `should_inject_reflection` 和 `reflection_prompt` 两个自由函数 + 单测；在 `apply_post_turn_discipline` 顶部 wire。
- 修改：`crates/rustcode-cli/src/main.rs` — 加 `--reflection-cadence <N>` flag，覆盖 config。

---

### Task 1: Config 字段 + serde 默认

**文件：**
- 修改：`crates/rustcode-core/src/config/mod.rs:47-69` (Config struct)
- 修改：`crates/rustcode-core/src/config/mod.rs:86` (附近加 default helper)

- [ ] **步骤 1：编写失败测试**

在 `crates/rustcode-core/src/config/mod.rs` 末尾追加：

```rust
#[cfg(test)]
mod reflection_config_tests {
    use super::*;

    #[test]
    fn reflection_cadence_defaults_to_ten_when_missing_from_toml() {
        let toml_text = r#"
default_provider = "claude"
[providers]
"#;
        let cfg: Config = toml::from_str(toml_text).expect("parses minimal config");
        assert_eq!(cfg.reflection_cadence, 10);
    }

    #[test]
    fn reflection_cadence_zero_means_disabled() {
        let toml_text = r#"
default_provider = "claude"
reflection_cadence = 0
[providers]
"#;
        let cfg: Config = toml::from_str(toml_text).expect("parses config with 0");
        assert_eq!(cfg.reflection_cadence, 0);
    }

    #[test]
    fn reflection_cadence_custom_value_is_preserved() {
        let toml_text = r#"
default_provider = "claude"
reflection_cadence = 7
[providers]
"#;
        let cfg: Config = toml::from_str(toml_text).expect("parses");
        assert_eq!(cfg.reflection_cadence, 7);
    }
}
```

- [ ] **步骤 2：运行以确认失败**

```bash
cargo test -p rustcode-core --lib config::reflection_config_tests
```

预期：编译错误 `no field 'reflection_cadence' on type 'Config'`。

- [ ] **步骤 3：添加字段**

在 `Config` 结构体中（`auto_update: bool` 字段附近，约 L67-68）添加：

```rust
    /// Every N tool calls, inject a "restate goal / what ruled out / next
    /// output" reflection prompt before the next turn. 0 disables the
    /// checkpoint entirely. Default 10 matches typical multi-file analysis
    /// step counts — below which the reflection is overhead, above which
    /// the agent can drift unnoticed.
    #[serde(default = "default_reflection_cadence")]
    pub reflection_cadence: usize,
```

在现有 `fn default_true() -> bool { true }`（约 L86）下方添加：

```rust
fn default_reflection_cadence() -> usize { 10 }
```

- [ ] **步骤 4：运行测试以确认通过**

```bash
cargo test -p rustcode-core --lib config::reflection_config_tests
```

预期：3 个测试通过。

- [ ] **步骤 5：检查是否有被破坏的 Config 构造函数**

```bash
cargo build 2>&1 | grep -E "missing field|E0063"
```

预期：输出为空。若任何字面量 `Config { ... }` 构造函数被破坏（通常出现在测试或 CLI 中），为其补上 `reflection_cadence: 10`。

- [ ] **步骤 6：全工作区构建**

```bash
cargo build
```

预期：干净通过。

- [ ] **步骤 7：提交**

```bash
git add crates/rustcode-core/src/config/mod.rs
git commit -m "feat(config): add reflection_cadence (default 10, 0 disables)"
```

---

### Task 2: DisciplineState 字段 + reset

**文件：**
- 修改：`crates/rustcode-core/src/agent/mod.rs:197-221` (DisciplineState struct)
- 修改：`crates/rustcode-core/src/agent/mod.rs:787` (reset block after `self.tool_call_count = 0;`)

- [ ] **步骤 1：添加字段**

在 `DisciplineState` 结构体中，于 `file_read_counts` 附近（约 L210）添加该字段：

```rust
    /// Snapshot of `AgentLoop.tool_call_count` at the last cadence
    /// reflection injection. The delta
    /// `tool_call_count - last_reflection_at_tool_count` feeds
    /// `should_inject_reflection`. Resets together with `tool_call_count`
    /// when a new user task starts.
    pub last_reflection_at_tool_count: usize,
```

`DisciplineState` 已经 derive 了 `Default`，因此 `usize` 会自动归零 —— 无需改动 `DisciplineState::default()` 路径。

- [ ] **步骤 2：与 tool_call_count 一起添加重置**

在 L787 找到 `self.tool_call_count = 0;`。紧随其后添加：

```rust
self.discipline_state.last_reflection_at_tool_count = 0;
```

理由：若 `tool_call_count` 回绕而标记未回绕，下一次 `tool_call_count - last_reflection_at_tool_count` 相减会得到一个无意义的 delta（`usize::saturating_sub` 虽然不会下溢，但语义依然是错的 —— 新任务的第一次 tool call 看起来会像“距上次 checkpoint 已 0 次调用”，而不是“N 次中的第 1 次”）。

- [ ] **步骤 3：验证构建**

```bash
cargo build -p rustcode-core
```

预期：构建干净（暂不添加测试 —— 该字段是纯数据；其用途在 Task 3/5 中测试）。

- [ ] **步骤 4：提交**

```bash
git add crates/rustcode-core/src/agent/mod.rs
git commit -m "feat(agent): track last_reflection_at_tool_count in DisciplineState"
```

---

### Task 3：纯函数 `should_inject_reflection` + 测试

**文件：**
- 修改：`crates/rustcode-core/src/agent/discipline.rs` — add free fn at bottom, add `#[cfg(test)] mod reflection_tests` block

- [ ] **步骤 1：编写失败测试**

追加到 `crates/rustcode-core/src/agent/discipline.rs`：

```rust
#[cfg(test)]
mod reflection_tests {
    use super::should_inject_reflection;

    #[test]
    fn no_injection_when_cadence_is_zero() {
        // cadence = 0 is the "disabled" sentinel — must never fire.
        assert_eq!(should_inject_reflection(50, 0, 0), None);
        assert_eq!(should_inject_reflection(1, 0, 0), None);
    }

    #[test]
    fn no_injection_when_delta_below_cadence() {
        // 9 tool calls since last checkpoint, cadence = 10 → not yet.
        assert_eq!(should_inject_reflection(9, 0, 10), None);
    }

    #[test]
    fn injection_when_delta_meets_cadence() {
        // Exactly at the threshold → fire.
        assert_eq!(should_inject_reflection(10, 0, 10), Some(10));
    }

    #[test]
    fn injection_when_delta_exceeds_cadence_after_batched_turn() {
        // A single turn can burn multiple tool calls, so the delta may
        // jump past the cadence in one go (13 - 0 = 13 ≥ 10).
        assert_eq!(should_inject_reflection(13, 0, 10), Some(13));
    }

    #[test]
    fn honors_prior_reflection_marker() {
        // After a checkpoint at count=10, the next one fires at count=20
        // (delta = 10 since last marker), not at count=10 trivially.
        assert_eq!(should_inject_reflection(19, 10, 10), None);
        assert_eq!(should_inject_reflection(20, 10, 10), Some(10));
    }

    #[test]
    fn marker_ahead_of_count_is_safe() {
        // Defensive: if the marker somehow exceeds current (should never
        // happen, but usize subtraction would underflow), saturate to 0
        // and do not fire.
        assert_eq!(should_inject_reflection(5, 10, 10), None);
    }
}
```

- [ ] **步骤 2：运行以确认失败**

```bash
cargo test -p rustcode-core --lib agent::discipline::reflection_tests
```

预期：编译错误 `cannot find function 'should_inject_reflection'`。

- [ ] **步骤 3：实现该纯函数**

在 `crates/rustcode-core/src/agent/discipline.rs` 底部、任意 `impl` 块**之外**添加：

```rust
/// Decide whether to inject a cadence-reflection prompt.
///
/// Returns `Some(delta)` when the number of tool calls since the last
/// reflection meets or exceeds `cadence`. Returns `None` otherwise,
/// including the `cadence == 0` "disabled" case.
///
/// The returned `delta` tells the caller how many tool calls have elapsed
/// since the last checkpoint, for use in the rendered prompt.
pub(crate) fn should_inject_reflection(
    current_tool_count: usize,
    last_reflection_at: usize,
    cadence: usize,
) -> Option<usize> {
    if cadence == 0 {
        return None;
    }
    let delta = current_tool_count.saturating_sub(last_reflection_at);
    if delta >= cadence {
        Some(delta)
    } else {
        None
    }
}
```

- [ ] **步骤 4：运行以确认通过**

```bash
cargo test -p rustcode-core --lib agent::discipline::reflection_tests
```

预期：6 个测试通过。

- [ ] **步骤 5：提交**

```bash
git add crates/rustcode-core/src/agent/discipline.rs
git commit -m "feat(discipline): add should_inject_reflection pure fn + tests"
```

---

### Task 4：纯函数 `reflection_prompt` + 测试

**文件：**
- 修改：`crates/rustcode-core/src/agent/discipline.rs` — add another free fn + test

- [ ] **步骤 1：编写失败测试**

追加到同一个 `reflection_tests` 模块内（最后一个 `}` 之上）：

```rust
    use super::reflection_prompt;

    #[test]
    fn reflection_prompt_is_language_neutral_and_mentions_delta() {
        let msg = reflection_prompt(12);

        // The delta must appear so the model sees the scale of the gap.
        assert!(msg.contains("12"), "prompt must include delta count, got: {}", msg);

        // Must NOT pretend this is an error — it's a scheduled recalibration.
        assert!(
            !msg.to_lowercase().contains("error"),
            "prompt must not frame as error, got: {}", msg
        );
        assert!(
            !msg.to_lowercase().contains("blocked"),
            "prompt must not look like a BLOCKED guard, got: {}", msg
        );

        // Must ask the three canonical language-neutral questions.
        assert!(
            msg.contains("original task") || msg.contains("restate"),
            "prompt must ask to restate the task, got: {}", msg
        );
        assert!(
            msg.contains("ruled out") || msg.contains("learned") || msg.contains("proven"),
            "prompt must ask what was learned/ruled out, got: {}", msg
        );
        assert!(
            msg.contains("next") && (msg.contains("concrete") || msg.contains("output")),
            "prompt must ask for the next concrete output, got: {}", msg
        );

        // Must NOT embed language-/tool-specific hints (this is the whole
        // point of the reflection being generic).
        assert!(!msg.to_lowercase().contains("cargo"));
        assert!(!msg.to_lowercase().contains("grep"));
        assert!(!msg.to_lowercase().contains("npm"));
    }
```

- [ ] **步骤 2：运行以确认失败**

```bash
cargo test -p rustcode-core --lib agent::discipline::reflection_tests::reflection_prompt_is_language_neutral_and_mentions_delta
```

预期：编译错误 `cannot find function 'reflection_prompt'`。

- [ ] **步骤 3：实现**

在 `crates/rustcode-core/src/agent/discipline.rs` 底部（`should_inject_reflection` 旁）添加：

```rust
/// Render the cadence-reflection prompt injected every `cadence` tool
/// calls. Language- and ecosystem-neutral by design — the three
/// questions apply to any task, any tool. Phrased as a scheduled
/// checkpoint (not a corrective intervention) because this fires
/// every N steps whether or not the agent appears stuck.
pub(crate) fn reflection_prompt(delta: usize) -> String {
    format!(
        "[Checkpoint — not an interruption, just a scheduled recalibration.]\n\
         You have made {} tool calls since the last checkpoint. \
         Before your next tool call, answer in plain text:\n\
         1. Restate the original task in one sentence.\n\
         2. What have the last {} steps proven or ruled out?\n\
         3. What is the next concrete output (an edit, an answer, a summary), \
         and roughly how many steps away?\n",
        delta, delta
    )
}
```

- [ ] **步骤 4：运行以确认**

```bash
cargo test -p rustcode-core --lib agent::discipline::reflection_tests
```

预期：7 个测试通过（Task 3 的 6 个 + 新增 1 个）。

- [ ] **步骤 5：提交**

```bash
git add crates/rustcode-core/src/agent/discipline.rs
git commit -m "feat(discipline): add reflection_prompt pure fn + tests"
```

---

### Task 5：接入 apply_post_turn_discipline

**文件：**
- 修改：`crates/rustcode-core/src/agent/discipline.rs:9-50` (apply_post_turn_discipline body)

- [ ] **步骤 1：在函数体顶部插入 cadence 检查**

编辑 `apply_post_turn_discipline`。该函数（在其文档注释之后）当前开头如下：

```rust
    pub(crate) fn apply_post_turn_discipline(&mut self) {
        // Re-read guard: when the same *region* of a file is read 2+ times,
        ...
```

在 re-read guard **之前**插入新块：

```rust
    pub(crate) fn apply_post_turn_discipline(&mut self) {
        // Cadence reflection: every N tool calls, inject a scheduled
        // "restate goal / what ruled out / next output" prompt regardless
        // of whether the agent appears stuck. Language-neutral, domain-
        // neutral. Fires one turn after the Nth tool call, so the agent
        // answers the three questions at the start of the following turn.
        if let Some(delta) = should_inject_reflection(
            self.tool_call_count,
            self.discipline_state.last_reflection_at_tool_count,
            self.config.reflection_cadence,
        ) {
            let msg = reflection_prompt(delta);
            self.conversation.add_user_message(&msg);
            self.discipline_state.last_reflection_at_tool_count = self.tool_call_count;
        }

        // Re-read guard: when the same *region* of a file is read 2+ times,
        // ... (existing code continues unchanged)
```

- [ ] **步骤 2：验证构建**

```bash
cargo build -p rustcode-core
```

预期：构建干净。若从 `impl AgentLoop { ... }` 块内部看不到 `should_inject_reflection` 或 `reflection_prompt` 在作用域内，则加 `self::` 前缀，或把它们移进该 `impl`（不过带 `pub(crate)` 的自由函数在同一模块内应当是直接可见的）。

- [ ] **步骤 3：跑全 crate 测试检查回归**

```bash
cargo test -p rustcode-core --lib 2>&1 | tail -6
```

预期：原有通过数 + Task 3/4 新增的 7 个测试。既有的 `self_update::tests::is_newer_semver` 可能仍会失败 —— 与本改动无关。

- [ ] **步骤 4：手工冒烟验证**

（没有自动化端到端测试 —— 在单元测试里构造 `AgentLoop` 的代价远超这段 glue 本身的价值；两个纯函数已被完整覆盖，这一步只是一次性的健全性检查。）

编辑本地的 `~/.config/rustcode/config.toml`，设置 `reflection_cadence = 2`。对任意仓库运行 rustcode，下发一个需要 ≥ 3 次 tool call 的任务。打开 turn datalog，确认第 2 次 tool call 之后出现了 `[Checkpoint — ...]` 这条 user message。

撤销该 config 覆盖。

- [ ] **步骤 5：提交**

```bash
git add crates/rustcode-core/src/agent/discipline.rs
git commit -m "feat(discipline): wire cadence reflection into apply_post_turn_discipline"
```

---

### Task 6：CLI flag 覆盖

**文件：**
- 修改：`crates/rustcode-cli/src/main.rs` around L371 (Cli struct) and L663 (config wiring)

- [ ] **步骤 1：添加 CLI 字段**

在 `crates/rustcode-cli/src/main.rs` 约 L371 处找到 `max_turns: Option<usize>` 声明。紧接其下添加：

```rust
    /// Inject a scheduled reflection prompt every N tool calls.
    /// 0 disables. Overrides the value in config.toml for this run.
    #[arg(long, value_name = "N")]
    reflection_cadence: Option<usize>,
```

- [ ] **步骤 2：接到 Config 上**

在约 L663 处找到 `agent_loop.set_max_turns(cli.max_turns);` 被调用的位置。此处 `config` 变量仍在作用域内（也可通过 `agent_loop.config` 访问）。在 `AgentLoop::new(...)` 之前（或 `config` 定型的任何位置 —— 遵循局部既有写法）立即添加：

```rust
if let Some(n) = cli.reflection_cadence {
    config.reflection_cadence = n;
}
```

若接线点在构造之后使用 `agent_loop.config`，则改为：

```rust
if let Some(n) = cli.reflection_cadence {
    agent_loop.config.reflection_cadence = n;
}
```

（选用与同一函数中 `max_turns` 模式相符的那种。）

- [ ] **步骤 3：验证构建**

```bash
cargo build
```

预期：构建干净。

- [ ] **步骤 4：手工冒烟测试**

```bash
cargo run -- --reflection-cadence 3 --help 2>&1 | grep reflection-cadence
```

预期：help 输出中出现该 flag 的说明。

```bash
cargo run -- --reflection-cadence 0
```

启动 rustcode，确认在多次 tool call 之后 checkpoint 消息不出现（0 表示禁用）。退出。

```bash
cargo run -- --reflection-cadence 3
```

启动 rustcode，跑一个 4 步任务，确认 checkpoint 在第 3 步之后出现。

- [ ] **步骤 5：提交**

```bash
git add crates/rustcode-cli/src/main.rs
git commit -m "feat(cli): --reflection-cadence flag overrides config.toml"
```

---

## 自查

**1. 规格覆盖：**
- “每 N 次 tool call 注入一次反思” → Task 5 [x]
- “语言中立的提示词（不出现 cargo/grep/npm）” → Task 4 测试断言其缺席 [x]
- “可配置的默认值” → Task 1（toml + serde default）[x]
- “CLI 覆盖” → Task 6 [x]
- “0 表示禁用” → Task 3 测试 `no_injection_when_cadence_is_zero` [x]
- “新任务链开始时重置” → Task 2 步骤 2 与 `tool_call_count` 一并重置 [x]
- “为可测试性而设的纯函数” → Task 3、4 都是自由函数 [x]

**2. 占位符扫描：**
- 无 “TBD”、无 “implement later”、无 “similar to task N” 之类的占位表述。
- 每个代码步骤都给出了完整代码。
- 每条命令都有预期输出或预期失败。
- Task 5 步骤 4 与 Task 6 步骤 4 是手工冒烟测试 —— 此处明确说明它们是一次性验证而非自动化覆盖，因为纯函数测试已覆盖逻辑，而 AgentLoop 构造的代价超过收益。

**3. 类型一致性：**
- `reflection_cadence: usize` —— 在 `Config`（Task 1）、CLI（Task 6）、函数签名（Task 3）以及 `apply_post_turn_discipline` 中的调用点（Task 5）保持一致。
- `last_reflection_at_tool_count: usize` —— 在 Task 2 声明，在 Task 5 使用。
- `should_inject_reflection(usize, usize, usize) -> Option<usize>` —— 在 Task 3 声明，在 Task 5 以一致的参数顺序（current、last、cadence）调用。
- `reflection_prompt(usize) -> String` —— 在 Task 4 声明，在 Task 5 调用。
- CLI 侧的 `Option<usize>`（Task 6）解包后赋给 `Config` 字段的 `usize` —— 与代码库中既有的 `max_turns` 模式一致。

---

## 范围外（推迟到后续计划）

- **强制反思**：阻塞下一次 `tool_call`，直到 agent 产出最低限量的文本。若 dogfooding 期间证明软注入不够，后续计划可在此之上加一层 `discipline`。
- **动态 cadence**：BLOCKED 事件之后收紧 N，成功编辑之后放宽 N。先做静态 cadence；仅当 dogfooding 表明这个固定值不合适时才调整。
- **按任务类型的 cadence**：为 “diagnosis” 与 “implementation” 阶段设置不同的 N。先用单一 N；若 dogfooding 表明一个数字不够用再重审。
- **Subagent 集成**：每个 subagent 都有自己的循环，因此同一机制可以移植过来，但不在本计划范围内。
- **反思质量打分**：衡量 agent 的反思文本是否真正回应了那三个问题。若软注入被证明太容易被忽略，则作为后续工作。
