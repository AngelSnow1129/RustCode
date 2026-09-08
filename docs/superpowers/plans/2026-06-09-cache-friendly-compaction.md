# 缓存友好的历史压缩 —— 实施计划

> **面向 agentic worker：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实施本计划。步骤使用复选框（`- [ ]`）语法进行跟踪。

**目标：** 用一次已提交、幂等、单调的旧 tool result 折叠，替换掉每次渲染时临时执行的 `microcompact`，从而消除接近 1M token 的 deepseek-v4-flash 会话中 prompt 前缀缓存被打断的问题。

**架构：** 目前 `microcompact`（render.rs）在每次渲染时都针对一个临时 Vec 重新推导要把哪些旧 `ToolResult` 打桩，且从不持久化 —— 于是渲染出的前缀在各轮之间逐字节漂移，provider 的缓存随之失效。我们把打桩搬到一个已提交的步骤（`collapse_committed`）中：它在 `turn/runner.rs` 真正发送前的渲染之前，一次性、幂等地修改 `conv.messages`。旧 tool result 一旦被打桩就永久保持逐字节一致（单调），因此前缀是只追加的。活动轮（最后一个 `Role::User` 之后的所有内容）保持完整；在正常路径上 `read_file` 被豁免。独立的 80% `FINAL BYTE CEILING` 与紧急 Tier-3 截断（真正的溢出防线）不受影响，因此不会引入 litellm 上下文溢出回归。

**技术栈：** Rust、`cargo` workspace、crate `rustcode-core`。测试是同文件内的 `#[test]` 函数（`mod tests`）。

**规格：** `docs/superpowers/specs/2026-06-09-cache-friendly-compaction-design.md`

**工作树 / 分支：** `/Users/lichao/project/gitcode/ai/rustcode-v4.25.1`，分支 `fix/cache-friendly-compaction`。

---

## 文件结构

| 文件 | 职责 | 改动 |
|---|---|---|
| `crates/rustcode-core/src/ctx/render.rs` | 渲染 + 压缩策略 | 给 `compact_old_tool_results_in_place` 增加 `exempt_read_file` 参数；新增 `collapse_committed` 自由函数；删除 `microcompact` 函数及其在 `build_messages` 内的调用；迁移测试 |
| `crates/rustcode-core/src/turn/runner.rs` | 每轮的 provider 渲染/发送 | 在真正发送前的 `build_messages` 之前立即调用 `collapse_committed(&mut conversation, context_window)` |
| `crates/rustcode-core/src/agent/mod.rs` | Agent 循环、紧急压缩 | 更新紧急 Tier-1 调用点，传入 `exempt_read_file = false`（行为不变）|
| `crates/rustcode-core/src/agent/compression.rs` | 压缩辅助函数 | 更新 `compact_old_tool_results_in_place` 调用点，传入 `false` |

所有工作都在上述工作树中进行。所有命令都在 `/Users/lichao/project/gitcode/ai/rustcode-v4.25.1` 下执行。

---

## Task 1: 为 `compact_old_tool_results_in_place` 增加 `exempt_read_file`

增加一个开关，使正常路径可以跳过 `read_file`（避免“伪自信”式的重复编辑循环），而紧急路径在真实的预算压力下仍然继续折叠它。

**文件：**
- 修改：`crates/rustcode-core/src/ctx/render.rs`（函数位于约 869–900 行，签名在第 869 行）
- 修改调用点：`crates/rustcode-core/src/agent/mod.rs:2936`、`crates/rustcode-core/src/agent/compression.rs:282`，以及编译器报出的其他所有调用方
- 测试：`crates/rustcode-core/src/ctx/render.rs`（`mod tests`）

- [ ] **Step 1: 编写失败测试**

在 `render.rs` 的 `mod tests` 块中添加（靠近其他 `collapse_*` 测试）：

```rust
    #[test]
    fn compact_old_exempts_read_file_when_flagged() {
        use crate::tool::{ToolCall, ToolResult};
        let mut conv = Conversation::new();
        conv.add_user_message("t0");
        conv.add_assistant_tool_calls(
            None,
            vec![ToolCall { id: "r".into(), name: "read_file".into(), arguments: "{}".into() }],
            None,
        );
        conv.add_tool_result(ToolResult {
            call_id: "r".into(),
            output: format!("L1\n{}", "x".repeat(2_000)),
            success: true,
        });
        conv.add_assistant_tool_calls(
            None,
            vec![ToolCall { id: "b".into(), name: "bash".into(), arguments: "{}".into() }],
            None,
        );
        conv.add_tool_result(ToolResult {
            call_id: "b".into(),
            output: format!("[elapsed: 0.0s, exit: 0]\n{}", "x".repeat(2_000)),
            success: true,
        });
        conv.add_user_message("t1"); // active turn → kept full

        compact_old_tool_results_in_place(&mut conv, 1, true);

        let get = |cid: &str, conv: &Conversation| {
            conv.messages.iter().find_map(|m| match &m.content {
                MessageContent::ToolResult(r) if r.call_id == cid => Some(r.output.clone()),
                _ => None,
            }).unwrap()
        };
        assert!(!get("r", &conv).starts_with('['), "read_file must stay full when exempt");
        assert!(get("b", &conv).starts_with("[bash "), "bash must be stubbed");
    }

    #[test]
    fn compact_old_stubs_read_file_when_not_exempt() {
        use crate::tool::{ToolCall, ToolResult};
        let mut conv = Conversation::new();
        conv.add_user_message("t0");
        conv.add_assistant_tool_calls(
            None,
            vec![ToolCall { id: "r".into(), name: "read_file".into(), arguments: "{}".into() }],
            None,
        );
        conv.add_tool_result(ToolResult {
            call_id: "r".into(),
            output: format!("L1\n{}", "x".repeat(2_000)),
            success: true,
        });
        conv.add_user_message("t1");

        compact_old_tool_results_in_place(&mut conv, 1, false);

        let out = conv.messages.iter().find_map(|m| match &m.content {
            MessageContent::ToolResult(r) if r.call_id == "r" => Some(r.output.clone()),
            _ => None,
        }).unwrap();
        assert!(out.starts_with("[read_file "), "emergency path must still stub read_file");
    }
```

- [ ] **Step 2: 运行测试确认无法编译**

运行：`cargo test -p rustcode-core compact_old_exempts_read_file_when_flagged 2>&1 | tail -20`
预期：编译错误 —— `compact_old_tool_results_in_place` 只接受 2 个参数，却传入了 3 个。

- [ ] **Step 3: 添加参数与跳过逻辑**

在 `render.rs` 中，把签名（约第 869 行）从：

```rust
pub(crate) fn compact_old_tool_results_in_place(
    conv: &mut crate::conversation::Conversation,
    keep_recent_turns: usize,
) {
```

改为：

```rust
pub(crate) fn compact_old_tool_results_in_place(
    conv: &mut crate::conversation::Conversation,
    keep_recent_turns: usize,
    exempt_read_file: bool,
) {
```

在循环内，紧接 `let tool_name = call_id_to_tool.get(&tr.call_id).map(|s| s.as_str()).unwrap_or("tool");` 之后、且在 `let summary = build_compact_stub(...)` 之前，插入：

```rust
        if exempt_read_file && tool_name == "read_file" {
            continue;
        }
```

- [ ] **Step 4: 修复所有既有调用点（由编译器指引）**

运行：`cargo build -p rustcode-core 2>&1 | grep -E 'compact_old_tool_results_in_place|error\[' | head`
每处调用点都会因参数个数报错。给**每一处**既有调用加上第三个参数 `false`（它们都必须保持今天的行为 —— 只有 Task 2 中新增的 `collapse_committed` 传入 `true`）：
- `agent/mod.rs:2936` → `compact_old_tool_results_in_place(&mut self.conversation, 3, false)`
- `agent/compression.rs:282` → `..., 3, false)`
- `render.rs`/`agent/mod.rs` 测试模块内的任何调用（例如 `compact_old_tool_results_in_place(&mut conv, 3, false)`、`..., 2, false)`、`..., 1, false)`）→ 追加 `, false`。

重复执行 `cargo build -p rustcode-core`，直到编译通过。

- [ ] **Step 5: 运行测试确认通过**

运行：`cargo test -p rustcode-core compact_old 2>&1 | tail -20`
预期：`compact_old_exempts_read_file_when_flagged` 与 `compact_old_stubs_read_file_when_not_exempt` PASS，且既有的 `collapse_*` 测试仍然 PASS。

- [ ] **Step 6: 提交**

```bash
git add crates/rustcode-core/src/ctx/render.rs crates/rustcode-core/src/agent/mod.rs crates/rustcode-core/src/agent/compression.rs
git commit -m "feat(ctx): add exempt_read_file flag to compact_old_tool_results_in_place"
```

---

## Task 2: 增加已提交的 `collapse_committed` 折叠

正常路径的入口：受阈值门控，保持活动轮完整，豁免 read_file，提交到 `conv.messages`。

**文件：**
- 修改：`crates/rustcode-core/src/ctx/render.rs`（在 `compact_old_tool_results_in_place` 附近新增自由函数）
- 测试：`crates/rustcode-core/src/ctx/render.rs` (`mod tests`)

- [ ] **Step 1: 编写失败测试**

```rust
    #[test]
    fn collapse_committed_noop_below_threshold() {
        use crate::tool::{ToolCall, ToolResult};
        let mut conv = Conversation::new();
        conv.add_user_message("t0");
        conv.add_assistant_tool_calls(
            None,
            vec![ToolCall { id: "b".into(), name: "bash".into(), arguments: "{}".into() }],
            None,
        );
        conv.add_tool_result(ToolResult {
            call_id: "b".into(),
            output: format!("[elapsed: 0.0s, exit: 0]\n{}", "x".repeat(1_000)),
            success: true,
        });
        conv.add_user_message("t1");

        // budget 1_000_000 → threshold 2.8M chars; ~1K payload → no-op.
        collapse_committed(&mut conv, 1_000_000);

        let out = conv.messages.iter().find_map(|m| match &m.content {
            MessageContent::ToolResult(r) if r.call_id == "b" => Some(r.output.clone()),
            _ => None,
        }).unwrap();
        assert!(!out.starts_with('['), "below threshold must stay full (append-only)");
    }

    #[test]
    fn collapse_committed_stubs_old_keeps_active_and_exempts_read_file() {
        use crate::tool::{ToolCall, ToolResult};
        let mut conv = Conversation::new();
        // 3 old turns: each a big read_file + big bash; then an active turn.
        for n in 0..3 {
            conv.add_user_message(&format!("t{n}"));
            let r = format!("r{n}");
            conv.add_assistant_tool_calls(
                None,
                vec![ToolCall { id: r.clone(), name: "read_file".into(), arguments: "{}".into() }],
                None,
            );
            conv.add_tool_result(ToolResult {
                call_id: r,
                output: format!("L1\n{}", "x".repeat(6_000)),
                success: true,
            });
            let b = format!("b{n}");
            conv.add_assistant_tool_calls(
                None,
                vec![ToolCall { id: b.clone(), name: "bash".into(), arguments: "{}".into() }],
                None,
            );
            conv.add_tool_result(ToolResult {
                call_id: b,
                output: format!("[elapsed: 0.0s, exit: 0]\n{}", "x".repeat(6_000)),
                success: true,
            });
        }
        // active turn (kept full): a bash that must NOT be stubbed.
        conv.add_user_message("active");
        conv.add_assistant_tool_calls(
            None,
            vec![ToolCall { id: "ba".into(), name: "bash".into(), arguments: "{}".into() }],
            None,
        );
        conv.add_tool_result(ToolResult {
            call_id: "ba".into(),
            output: format!("[elapsed: 0.0s, exit: 0]\n{}", "x".repeat(6_000)),
            success: true,
        });

        // budget 8_000 → threshold 22_400 chars; payload ~42K → fires.
        collapse_committed(&mut conv, 8_000);

        let get = |cid: &str, conv: &Conversation| {
            conv.messages.iter().find_map(|m| match &m.content {
                MessageContent::ToolResult(r) if r.call_id == cid => Some(r.output.clone()),
                _ => None,
            }).unwrap()
        };
        // old bash → stubbed; old read_file → exempt (full); active bash → full.
        assert!(get("b0", &conv).starts_with("[bash "), "old bash must be stubbed");
        assert!(!get("r0", &conv).starts_with('['), "old read_file must stay full (exempt)");
        assert!(!get("ba", &conv).starts_with('['), "active-turn bash must stay full");
    }

    #[test]
    fn collapse_committed_is_idempotent() {
        use crate::tool::{ToolCall, ToolResult};
        let mut conv = Conversation::new();
        conv.add_user_message("t0");
        conv.add_assistant_tool_calls(
            None,
            vec![ToolCall { id: "b".into(), name: "bash".into(), arguments: "{}".into() }],
            None,
        );
        conv.add_tool_result(ToolResult {
            call_id: "b".into(),
            output: format!("[elapsed: 0.0s, exit: 0]\n{}", "x".repeat(30_000)),
            success: true,
        });
        conv.add_user_message("t1");

        collapse_committed(&mut conv, 8_000);
        let after_first = conv.messages.iter().find_map(|m| match &m.content {
            MessageContent::ToolResult(r) if r.call_id == "b" => Some(r.output.clone()),
            _ => None,
        }).unwrap();
        collapse_committed(&mut conv, 8_000);
        let after_second = conv.messages.iter().find_map(|m| match &m.content {
            MessageContent::ToolResult(r) if r.call_id == "b" => Some(r.output.clone()),
            _ => None,
        }).unwrap();
        assert_eq!(after_first, after_second, "re-running must not re-stub (idempotent)");
    }
```

- [ ] **Step 2: 运行测试确认失败**

运行：`cargo test -p rustcode-core collapse_committed 2>&1 | tail -20`
预期：编译错误 —— 找不到 `collapse_committed`。

- [ ] **Step 3: 实现 `collapse_committed`**

在 `render.rs` 中，紧接 `compact_old_tool_results_in_place` 函数之后（其结束的 `}` 约在第 900 行），添加：

```rust
/// Normal-path committed collapse (cache-friendly replacement for the
/// removed ephemeral `microcompact`). Threshold-gated by the same
/// `70% × budget × 4` char trigger; below it this is a no-op so short
/// sessions stay full-fidelity and byte-stable (append-only). Above it,
/// permanently stubs old (non-active-turn, non-`read_file`) ToolResults in
/// `conv.messages`. Idempotent + monotonic via `compact_old_tool_results_in_place`
/// (`keep_recent_turns = 1` → keeps everything after the last `Role::User`,
/// i.e. the active turn). Because it commits, the stubbed prefix never
/// changes again across turns — the property `microcompact` violated.
pub(crate) fn collapse_committed(conv: &mut crate::conversation::Conversation, token_budget: usize) {
    let threshold_chars = (token_budget as u64 * 4 * 70 / 100) as usize;
    let total_chars: usize = conv
        .messages
        .iter()
        .map(|m| match &m.content {
            MessageContent::ToolResult(r) => r.output.len(),
            MessageContent::Text(t) => t.len(),
            _ => 100,
        })
        .sum();
    if total_chars < threshold_chars {
        return;
    }
    compact_old_tool_results_in_place(conv, /* keep_recent_turns */ 1, /* exempt_read_file */ true);
}
```

- [ ] **Step 4: 运行测试确认通过**

运行：`cargo test -p rustcode-core collapse_committed 2>&1 | tail -20`
预期：三个 `collapse_committed_*` 测试全部 PASS。

- [ ] **Step 5: 提交**

```bash
git add crates/rustcode-core/src/ctx/render.rs
git commit -m "feat(ctx): add committed, idempotent collapse_committed (replaces microcompact)"
```

---

## Task 3: 核心回归 —— 前缀在各轮之间保持字节冻结

这是最核心的验收测试：正是缺少这条性质才导致缓存被打断。一个 tool result 一旦被打桩，就必须在之后每一轮都保持逐字节一致。

**文件：**
- 测试：`crates/rustcode-core/src/ctx/render.rs` (`mod tests`)

- [ ] **Step 1: 编写失败测试**

```rust
    #[test]
    fn collapse_committed_freezes_stubbed_prefix_across_turns() {
        use crate::tool::{ToolCall, ToolResult};
        // Low budget (threshold = 1_000*4*70/100 = 2_800 chars) on purpose:
        // after the first collapse shrinks the conv, the SECOND collapse must
        // still be above threshold so it actually re-fires and re-examines the
        // already-stubbed prefix — otherwise the freeze assertion is vacuous.
        let budget = 1_000;

        let add_turn = |conv: &mut Conversation, n: usize| {
            conv.add_user_message(&format!("task {n}"));
            let id = format!("c{n}");
            conv.add_assistant_tool_calls(
                None,
                vec![ToolCall { id: id.clone(), name: "bash".into(), arguments: "{}".into() }],
                None,
            );
            conv.add_tool_result(ToolResult {
                call_id: id,
                output: format!("[elapsed: 0.0s, exit: 0]\n{}", "x".repeat(6_000)),
                success: true,
            });
        };

        let mut conv = Conversation::new();
        for n in 0..5 {
            add_turn(&mut conv, n);
        }
        collapse_committed(&mut conv, budget);

        // Snapshot every ALREADY-stubbed tool result (output starts with '[').
        let stubbed_before: Vec<(usize, String)> = conv
            .messages
            .iter()
            .enumerate()
            .filter_map(|(i, m)| match &m.content {
                MessageContent::ToolResult(r) if r.output.starts_with('[')
                    && r.output.contains("lines, first:") =>
                {
                    Some((i, r.output.clone()))
                }
                _ => None,
            })
            .collect();
        assert!(!stubbed_before.is_empty(), "expected stubs after first collapse");

        // A new turn arrives; collapse again (the just-aged turn now stubs).
        add_turn(&mut conv, 5);
        collapse_committed(&mut conv, budget);

        // Every previously-stubbed result must be byte-identical (monotonic, frozen).
        for (i, before) in &stubbed_before {
            match &conv.messages[*i].content {
                MessageContent::ToolResult(r) => assert_eq!(
                    &r.output, before,
                    "stub at message #{i} mutated across turns — prefix cache would break"
                ),
                other => panic!("message #{i} changed content variant: {:?}", other),
            }
        }
    }
```

- [ ] **Step 2: 运行测试确认立即通过**

运行：`cargo test -p rustcode-core collapse_committed_freezes_stubbed_prefix_across_turns 2>&1 | tail -20`
预期：PASS（Task 2 的实现已经保证了这一点；本测试用于把这条不变量钉死，防止回归）。

> 如果 FAIL，就不要继续 —— 单调/幂等保证已被破坏。重新检查 Task 2 中 `compact_old_tool_results_in_place` 的幂等性（stub `< MIN_COLLAPSE_SIZE` 时跳过）。

- [ ] **Step 3: 提交**

```bash
git add crates/rustcode-core/src/ctx/render.rs
git commit -m "test(ctx): pin byte-frozen prefix invariant across turns"
```

---

## Task 4: 把 `collapse_committed` 接入发送路径；停止在 `build_messages` 中调用 `microcompact`

**文件：**
- 修改：`crates/rustcode-core/src/turn/runner.rs:124-128`
- 修改：`crates/rustcode-core/src/ctx/render.rs:312-313` (remove the microcompact call + threshold local)

- [ ] **Step 1: 在真正发送前的渲染之前插入已提交的折叠**

在 `turn/runner.rs` 中，真正发送前的渲染位于第 124–128 行：

```rust
        let context_window = self.ctx.ctx_window();

        let (messages, ctx_stats) =
            self.ctx
                .build_messages(conversation, system_prompt, turn_reminder);
```

在 `context_window` 与 `build_messages` 调用之间插入一行（按 runner.rs:62-64 中 `run` 的签名，`conversation` 是 `&mut Conversation`）：

```rust
        let context_window = self.ctx.ctx_window();

        // Commit-collapse old tool results (idempotent, monotonic) so the
        // sent prefix stays byte-stable across turns and the provider
        // prompt-cache holds. Replaces the removed ephemeral microcompact.
        crate::ctx::render::collapse_committed(conversation, context_window);

        let (messages, ctx_stats) =
            self.ctx
                .build_messages(conversation, system_prompt, turn_reminder);
```

- [ ] **Step 2: 从 `build_messages` 中移除 microcompact 调用**

在 `render.rs` 中，删除第 312-313 行（阈值局部变量与调用）：

```rust
    let microcompact_threshold = (token_budget as u64 * 4 * 70 / 100) as usize;
    microcompact(&mut result, conv.messages.len(), microcompact_threshold);
```

保留周围的注释块（291-329）—— 但要更新其中已经过时的说法。把第 291-311 行的注释段落替换为一行指引：

```rust
    // Prior-turn ToolResult stubbing now happens via the COMMITTED
    // `collapse_committed` (called on `conv.messages` before render, in
    // turn/runner.rs) — NOT here. build_messages stays a pure renderer so
    // the rendered prefix is byte-stable across turns. The 80% FINAL BYTE
    // CEILING below remains the render-time overflow backstop.
```

（第 315-329 行关于 `replace_stale_reads` 的 `NOTE (prompt-cache)` 块保持原样。）

- [ ] **Step 3: 构建 —— 预期只有 microcompact 测试会挂**

运行：`cargo build -p rustcode-core 2>&1 | tail -20`
预期：库可以编译（`microcompact` 函数仍然存在，只是未使用 → 会产生一个 dead-code 警告）。测试的编译在 Task 5 中处理。

- [ ] **Step 4: 提交**

```bash
git add crates/rustcode-core/src/turn/runner.rs crates/rustcode-core/src/ctx/render.rs
git commit -m "feat(ctx): collapse old tool results committed on the send path; drop microcompact from build_messages"
```

---

## Task 5: 删除 `microcompact` 并迁移其测试

**文件：**
- 修改：`crates/rustcode-core/src/ctx/render.rs` —— 删除 `microcompact` 函数（约 925-1001 行）并处理其测试

- [ ] **Step 1: 删除 `microcompact` 函数**

移除整个 `fn microcompact(...)`（从约第 902 行的文档注释到约第 1001 行结束的 `}`）。辅助函数 `build_call_id_to_tool_map`（被 `compact_old_tool_results_in_place` 使用）和 `build_compact_stub` 必须保留。

- [ ] **Step 2: 构建测试目标以列出破坏点**

运行：`cargo test -p rustcode-core --no-run 2>&1 | grep -E "cannot find function .microcompact|error" | head`
预期：只有直接调用 `microcompact(...)` 的测试会报错。

- [ ] **Step 3: 逐个处理被破坏 / 现已冗余的测试（确定性规则）**

对每个失败的测试套用下面这条确切规则：

1. **直接调用 `microcompact(&mut msgs, ...)` 的测试** —— 它们测的是那个已不存在的临时函数。其行为现在已由 Tasks 2–3 中新增的 `collapse_committed_*` 测试覆盖。**删除**它们：
   - `microcompact_uses_generic_format_with_tool_label_from_call_id`
   - `microcompact_preserves_current_turn_in_full`
   - `microcompact_is_idempotent_no_double_stub`
   - `microcompact_respects_threshold_parameter`
   - `microcompact_scales_with_window`

2. **`microcompact_skips_read_file_to_preserve_long_session_context`**（调用 `build_messages`，断言 read_file 保持完整）—— read_file 现在由 `collapse_committed` 豁免，而不是由 `build_messages` 豁免。**改写**它那唯一一次渲染调用：把

   ```rust
        let (msgs, _) = build_messages(&conv, "sys", 40_000, "");
   ```

   改为

   ```rust
        collapse_committed(&mut conv, 40_000);
        let (msgs, _) = build_messages(&conv, "sys", 40_000, "");
   ```

   （该测试的其余部分不变 —— 它仍然断言 `c_read` 是完整的，且至少有一个 bash 是 `[bash ok: ...]`。注意 `conv` 必须是 `let mut conv` —— 它本来就是。）

3. **任何其他断言“上一轮的 ToolResult 已被打桩”的 `build_messages` 测试**（排查方式：先 `cargo test -p rustcode-core --no-run`，再跑 render 测试并查看失败项）—— 套用与 (2) 相同的机械修复：在 `build_messages(&conv, …, <budget>, …)` 调用的上一行插入 `collapse_committed(&mut conv, <same budget>);`。低于阈值 / 当前轮 / 溢出上限的测试无需改动。

- [ ] **Step 4: 运行完整的 render 测试模块**

运行：`cargo test -p rustcode-core ctx::render 2>&1 | tail -30`
预期：全部 PASS。若某个基于 `build_messages` 的测试仍因缺少 stub 而失败，就对它套用规则 (2)/(3)；若它因为断言“没有 stub”而现在出现了 stub 而失败，说明预算低于阈值 —— 保持原样并重新检查该断言。

- [ ] **Step 5: 提交**

```bash
git add crates/rustcode-core/src/ctx/render.rs
git commit -m "refactor(ctx): delete microcompact; migrate tests to collapse_committed"
```

---

## Task 6: 完整验证

**文件：** 无（仅验证）

- [ ] **Step 1: workspace 构建 + lint**

运行：`cargo build --workspace 2>&1 | tail -15`
预期：成功，无错误。对现已未使用的条目，通过删除来消除 dead-code 警告。

运行：`cargo clippy -p rustcode-core 2>&1 | tail -25`
预期：这些文件不引入新的警告。

- [ ] **Step 2: 运行受影响 crate 的测试**

运行：`cargo test -p rustcode-core 2>&1 | tail -30`
预期：全部 PASS，包括 `collapse_committed_*`、`compact_old_*`、紧急压缩测试（`proactive_tier1_*`、`collapse_keeps_last_n_turns_full` 等），以及迁移后的 read_file 测试。

- [ ] **Step 3: 定向重跑不变量测试**

运行：`cargo test -p rustcode-core collapse_committed_freezes_stubbed_prefix_across_turns -- --nocapture 2>&1 | tail -10`
预期：PASS —— 字节冻结前缀的保证成立。

- [ ] **Step 4: 最终提交（若有清理）**

```bash
git add -A
git commit -m "chore(ctx): cleanup after cache-friendly compaction" --allow-empty
```

---

## 完成标准

- `microcompact` 已消失；`build_messages` 不再修改 stub 状态。
- `collapse_committed` 在 `turn/runner.rs` 中真正发送前的渲染之前运行一次：已提交 + 幂等 + 单调，按 `70% × budget × 4` 字符做阈值门控，保持活动轮完整，豁免 `read_file`。
- 紧急路径不变（`compact_old_tool_results_in_place(..., 3, false)`）。
- 80% `FINAL BYTE CEILING` 与紧急 Tier-3 未受影响 → 无溢出回归。
- `collapse_committed_freezes_stubbed_prefix_across_turns` 通过（缓存不变量）。
- `cargo test -p rustcode-core` 全绿。

## 合入后度量（非代码任务）

发布后，在 4.25.x 上重跑 `cache-hit-rca` skill：`find_pairs --version <new> --day today` → `classify_pairs` → 确认 `tool_*改写/截断` 的占比以及 `bad_hit<10%` 的前缀打断量下降。若首次跨阈值那一次性的前缀打断仍然显著，则评估规格第 ② 项（把最旧的轮次折叠进已冻结的 `cold_summaries` 前缀）。
