# DeepSeek Skill-First 提醒 —— 实施计划

> **面向 agentic worker：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 按任务逐步实施本计划。步骤使用复选框（`- [ ]`）语法进行跟踪。

**目标：** 通过在开场 turn 注入一段强硬的 skill-first `<system-reminder>`，让 DeepSeek 在探索或给出方案之前先加载匹配的流程类 skill（例如 `brainstorming`）。

**架构：** 新增一个仅限 DeepSeek 的 `LifecycleHooks` 实现（`SkillFirstHook`），只在 `turn_id==1 && round==1` 时触发一次，在请求尾部追加一个 `<system-reminder>` —— 与 `StatusReminderHook`/`TodoHook` 所用的是同一种每轮临时注入机制；对弱模型而言，它的时效性强于静态 persona 行。仅在 DeepSeek 且 skill 目录非空时启用。在 `prepare()` 中注册，因此同时覆盖 TUI/CLI 与 daemon/webui（二者共用同一条 `CodingRuntime` 流水线）。

**技术栈：** Rust、`rustcode-coding` crate、`rustcode-kernel` hook trait、`rustcode-capabilities::reminder`、`cargo test`。

## 全局约束

- **仅限 DeepSeek。** 通过既有的 `crate::persona::model_needs_firm_execution(model)` 谓词做门控。GLM / frontier 模型永远拿不到该 hook。
- **绝不对未挂载的工具做提示。** 同时以 skill 目录非空为门控 —— 未安装任何 skill 时该 hook 是 no-op（沿用 `TodoHook` / `request_user_input` 的门控纪律）。
- **仅开场 turn、一次性**（`ctx.turn_id == 1 && ctx.round == 1`）。后续 round/turn 不再注入 —— 避免在进行中的编码过程里产生每轮噪声。
- **刻意在 round 1 触发（与 `StatusReminderHook` 不同）。** 该提醒必须先于模型的第一个动作。由此产生的 user-after-user 尾部是安全的，*正因为该 hook 仅限 DeepSeek*（OpenAI 兼容 API 容忍连续的 user 消息；导致 `StatusReminderHook` 跳过 round 1 的那条 Anthropic 严格拒绝规则在此根本不适用）。
- **临时注入**，经 `rustcode_capabilities::reminder::system_reminder` 包成 `<system-reminder>`，以 `Message::user(...)` 追加 —— 与 `TodoHook`/`StatusReminderHook` 同一约定。绝不改动被持久化的 user 消息。
- 同时覆盖 TUI/CLI 与 daemon/webui（同一条 `CodingRuntime` → `prepare()` 流水线）。
- 提交 trailer：`Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`。在当前分支 `release/v5.0.1` 上工作。

---

## 文件结构

- `crates/rustcode-coding/src/skill_first.rs` —— **新建。** `SkillFirstHook` 单元：构造期门控、纯提醒正文，以及 `pre_request` 触发逻辑。自包含 + 带单测。
- `crates/rustcode-coding/src/lib.rs` —— 添加 `mod skill_first;`（照 `mod todo;` 的写法，约第 56 行）。
- `crates/rustcode-coding/src/persona.rs:181` —— 把 `fn model_needs_firm_execution` 放宽为 `pub(crate) fn`，以便该 hook 复用 DeepSeek 谓词。
- `crates/rustcode-coding/src/parts.rs` —— 在 `skill_catalog` 被 move 之前（约第 509 行）捕获 `has_skills` 标志，并在 `TodoHook` 之后（约第 541 行）注册该 hook。

---

### Task 1: `SkillFirstHook` —— hook 单元

**文件：**
- 修改：`crates/rustcode-coding/src/persona.rs:181`（可见性）
- 修改：`crates/rustcode-coding/src/lib.rs`（模块声明，约第 56 行）
- 新建：`crates/rustcode-coding/src/skill_first.rs`
- 测试：`crates/rustcode-coding/src/skill_first.rs`（`#[cfg(test)] mod tests`）

**接口：**
- 消费：`crate::persona::model_needs_firm_execution(&str) -> bool`（在此改为 `pub(crate)`）；`rustcode_capabilities::reminder::system_reminder(&str) -> String`；`rustcode_kernel::hook::{LifecycleHooks, TurnCtx}`；`rustcode_kernel::message::Message`。
- 产出：`pub struct SkillFirstHook`，带 `pub fn new(model: &str, has_skills: bool) -> Self`，实现 `LifecycleHooks`。Task 2 以 `crate::skill_first::SkillFirstHook::new(&cfg.model, has_skills)` 构造它。

- [ ] **步骤 1：放宽 DeepSeek 谓词的可见性**

在 `crates/rustcode-coding/src/persona.rs` 第 181 行，将：

```rust
fn model_needs_firm_execution(model: &str) -> bool {
```

改为：

```rust
pub(crate) fn model_needs_firm_execution(model: &str) -> bool {
```

- [ ] **步骤 2：声明模块**

在 `crates/rustcode-coding/src/lib.rs` 中，紧邻 `mod todo;`（约第 56 行）添加：

```rust
mod skill_first;
```

- [ ] **步骤 3：创建 hook 文件，带 NO-OP `pre_request` 和完整测试（red 步骤）**

创建 `crates/rustcode-coding/src/skill_first.rs`，包含结构体、真实的 `body()`、故意留空的 `pre_request`（以便触发类测试先失败），以及测试：

```rust
//! `SkillFirstHook` — a DeepSeek-only opening-turn `<system-reminder>` that forces a
//! skill-first check before the model explores or proposes a solution.
//!
//! A weak model (DeepSeek) under-weights the soft `## SKILLS:` guidance and the static
//! `SKILL/PROCESS FIRST` persona line (both proved insufficient on real hardware): it
//! opens by exploring the codebase and pre-solutioning instead of loading a matching
//! process skill like `brainstorming`. This injects the skill-first directive with high
//! recency — at the request TAIL, on the opening turn — the same ephemeral mechanism
//! `TodoHook`/`StatusReminderHook` use.
//!
//! Gated to DeepSeek (via `model_needs_firm_execution`) AND a non-empty skill catalog
//! (never nudge `use_skill` when no skills are installed). One-shot: opening turn only.
//!
//! Unlike `StatusReminderHook` we DO fire on round 1 — the reminder must preempt the
//! model's very first action. The resulting user-after-user tail is safe here because the
//! hook is DeepSeek-only (OpenAI-compatible; consecutive user messages are accepted,
//! unlike the Anthropic-strict rejection that makes `StatusReminderHook` skip round 1).

use async_trait::async_trait;
use rustcode_capabilities::reminder::system_reminder;
use rustcode_kernel::hook::{LifecycleHooks, TurnCtx};
use rustcode_kernel::message::Message;

/// Injects a one-shot skill-first `<system-reminder>` on the opening turn, for DeepSeek only.
pub struct SkillFirstHook {
    /// Precomputed at construction: DeepSeek AND at least one skill installed.
    enabled: bool,
}

impl SkillFirstHook {
    /// Enabled only for a weak model needing firm steering (DeepSeek) AND when the skill
    /// catalog is non-empty (`has_skills`). Anything else yields a no-op hook.
    pub fn new(model: &str, has_skills: bool) -> Self {
        Self {
            enabled: has_skills && crate::persona::model_needs_firm_execution(model),
        }
    }

    /// The forceful skill-first reminder body (pure, testable). Wrapped by
    /// `system_reminder` before injection.
    fn body() -> &'static str {
        "Before you explore the codebase, plan, or propose a solution: check the \
\"=== AVAILABLE SKILLS ===\" catalog above. If this request matches a skill's description \
— a design / build / \"help me figure out / plan this\" request matches `brainstorming` — \
you MUST call `use_skill` with that skill NOW and let it drive: ask the user ONE question \
at a time and do NOT pre-decide the solution or start exploring first. If nothing in the \
catalog matches, proceed normally."
    }
}

#[async_trait]
impl LifecycleHooks for SkillFirstHook {
    async fn pre_request(&self, _messages: &mut Vec<Message>, _ctx: &TurnCtx) {
        // (implemented in Step 5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(turn_id: u64, round: u32) -> TurnCtx {
        TurnCtx {
            turn_id,
            round,
            ..Default::default()
        }
    }

    #[test]
    fn body_names_use_skill_brainstorming_and_one_at_a_time() {
        let b = SkillFirstHook::body();
        assert!(b.contains("use_skill"), "{b}");
        assert!(b.contains("brainstorming"), "{b}");
        assert!(b.contains("ONE question at a time"), "{b}");
    }

    #[tokio::test]
    async fn deepseek_opening_turn_injects_one_wrapped_reminder() {
        let hook = SkillFirstHook::new("deepseek-v4-flash", true);
        let mut msgs = vec![Message::system("s"), Message::user("hi")];
        hook.pre_request(&mut msgs, &ctx(1, 1)).await;
        assert_eq!(msgs.len(), 3, "opening turn appends exactly one reminder");
        assert!(
            msgs[2].text.starts_with("<system-reminder>") && msgs[2].text.contains("use_skill"),
            "wrapped skill-first reminder: {:?}",
            msgs[2].text
        );
    }

    #[tokio::test]
    async fn does_not_fire_after_the_opening_turn() {
        let hook = SkillFirstHook::new("deepseek-v4-flash", true);
        // Round 2 of turn 1 — too late, and would double-inject.
        let mut a = vec![Message::user("hi"), Message::assistant("a", vec![])];
        let before_a = a.clone();
        hook.pre_request(&mut a, &ctx(1, 2)).await;
        assert_eq!(a, before_a, "must not fire on later rounds");
        // Turn 2 — a fresh user message later in the session.
        let mut b = vec![Message::user("hi")];
        let before_b = b.clone();
        hook.pre_request(&mut b, &ctx(2, 1)).await;
        assert_eq!(b, before_b, "must not fire on later turns");
    }

    #[tokio::test]
    async fn disabled_for_glm_frontier_and_empty_catalog() {
        for (model, has_skills) in [
            ("glm-5.2", true),
            ("m", true),
            ("deepseek-v4-flash", false),
        ] {
            let hook = SkillFirstHook::new(model, has_skills);
            let mut msgs = vec![Message::user("hi")];
            let before = msgs.clone();
            hook.pre_request(&mut msgs, &ctx(1, 1)).await;
            assert_eq!(
                msgs, before,
                "must be a no-op for (model={model}, has_skills={has_skills})"
            );
        }
    }
}
```

- [ ] **步骤 4：运行测试，确认触发类测试 FAIL**

运行：`cargo test -p rustcode-coding --lib skill_first`

预期：`body_names_...` 与 `disabled_...` PASS（no-op hook 不注入任何内容，正好符合“已禁用”的预期），但 `deepseek_opening_turn_injects_one_wrapped_reminder` 与 `does_not_fire_after_the_opening_turn` —— 特别是*开场 turn* 那一个 —— FAIL（断言 `msgs.len() == 3`，而 no-op 让它停在 2）。

- [ ] **步骤 5：实现 `pre_request`**

把 no-op 的 `pre_request` 函数体替换为：

```rust
    async fn pre_request(&self, messages: &mut Vec<Message>, ctx: &TurnCtx) {
        if !self.enabled {
            return;
        }
        // Opening turn only (one-shot). We DO fire on round 1 (see module doc): the
        // reminder must land before the model's first action, and the user-after-user
        // tail is safe because this hook is DeepSeek-only (OpenAI-compatible).
        if ctx.turn_id != 1 || ctx.round != 1 {
            return;
        }
        messages.push(Message::user(system_reminder(Self::body())));
    }
```

- [ ] **步骤 6：运行测试确认通过**

运行：`cargo test -p rustcode-coding --lib skill_first`

预期：四个测试全部 PASS。

- [ ] **步骤 7：确认该 crate 无回归**

运行：`cargo test -p rustcode-coding`

预期：全量套件 PASS（包括 persona.rs 中既有的 `model_needs_firm_execution_is_deepseek_only` —— 可见性改动不影响行为）。

- [ ] **步骤 8：提交**

```bash
git add crates/rustcode-coding/src/skill_first.rs crates/rustcode-coding/src/lib.rs crates/rustcode-coding/src/persona.rs
git commit -m "feat(coding): SkillFirstHook — deepseek opening-turn skill-first reminder

A weak model (deepseek) skips use_skill and dives into exploring/solutioning;
the static SKILL/PROCESS FIRST persona line did not hold on real hardware. This
adds a deepseek-only lifecycle hook that injects a forceful skill-first
<system-reminder> at the request tail on the opening turn (high recency, same
mechanism as StatusReminderHook/TodoHook). Gated to deepseek + a non-empty skill
catalog. Fires on round 1 by design (must preempt the first action; safe because
deepseek is OpenAI-compatible). Not yet wired into prepare() (next commit).

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task 2: 把 `SkillFirstHook` 接入 `prepare()`

**文件：**
- 修改：`crates/rustcode-coding/src/parts.rs`（在第 ~509 行之前捕获 `has_skills`；在第 ~541 行之后注册 hook）

**接口：**
- 消费：Task 1 的 `crate::skill_first::SkillFirstHook::new(&cfg.model, has_skills)`；`prepare()` 中既有的 `skill_catalog: Option<String>` 局部变量与 `cfg.model`。
- 产出：无新增 —— 只是往既有的 `hooks` vec 追加一个 hook。

- [ ] **步骤 1：在 `skill_catalog` 被 move 之前捕获 `has_skills` 标志**

在 `crates/rustcode-coding/src/parts.rs` 中，目录在约第 509 行被 move 进 `SkillCatalogHook::new(skill_catalog)`。在该行**之前**立即加入捕获。将：

```rust
    // Skill catalog — leading system message (persona → context → memory → skills), so
    // the model sees which skills are installed and can trigger one on a description
    // match. `None` (no skills) makes the hook a no-op. Reconciles in place on resume.
    hooks.push(Arc::new(SkillCatalogHook::new(skill_catalog)));
```

改为：

```rust
    // Skill catalog — leading system message (persona → context → memory → skills), so
    // the model sees which skills are installed and can trigger one on a description
    // match. `None` (no skills) makes the hook a no-op. Reconciles in place on resume.
    // Capture whether any skill is installed BEFORE the catalog is moved — SkillFirstHook
    // (registered below) uses it to stay a no-op when there's nothing to trigger.
    let has_skills = skill_catalog.as_ref().is_some_and(|c| !c.trim().is_empty());
    hooks.push(Arc::new(SkillCatalogHook::new(skill_catalog)));
```

- [ ] **步骤 2：在 `TodoHook` 之后注册该 hook**

在 `crates/rustcode-coding/src/parts.rs` 中，`TodoHook` 块（约第 539-541 行）之后：

```rust
    if crate::persona::todo_switch_enabled() {
        hooks.push(Arc::new(crate::todo::TodoHook));
    }
```

添加：

```rust
    // DeepSeek-only opening-turn skill-first reminder. A weak model (deepseek) skips
    // use_skill and dives straight into exploring/solutioning; a static persona line did
    // not hold. This injects a forceful <system-reminder> on the opening turn only, where
    // recency is high. Gated to deepseek (model_needs_firm_execution) + a non-empty skill
    // catalog (never nudge use_skill when no skills are installed). No-op otherwise.
    hooks.push(Arc::new(crate::skill_first::SkillFirstHook::new(
        &cfg.model,
        has_skills,
    )));
```

- [ ] **步骤 3：构建并运行该 crate 的测试套件**

运行：`cargo test -p rustcode-coding`

预期：编译通过且全量套件 PASS（既有 hook 行为不变；新 hook 只是被追加）。

- [ ] **步骤 4：确认提醒正文已被编译进 rustcode 二进制**

运行：`cargo build --bin rustcode && strings target/debug/rustcode | grep -c "ONE question at a time"`

预期：打印 `1`（提醒正文已被烘焙进二进制）。

- [ ] **步骤 5：提交**

```bash
git add crates/rustcode-coding/src/parts.rs
git commit -m "feat(coding): register SkillFirstHook in prepare()

Wire the deepseek-only opening-turn skill-first reminder into the canonical hook
chain (after TodoHook), gated on deepseek + a non-empty skill catalog. Reaches
both TUI/CLI and daemon/webui via the shared CodingRuntime pipeline.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## 自查

**规格覆盖：**
- “仅限 DeepSeek 的 lifecycle hook” → Task 1（`SkillFirstHook`，经 `model_needs_firm_execution` 门控）。[x]
- “目录非空门控” → Task 1 的 `new(model, has_skills)` + Task 2 的 `has_skills` 捕获。[x]
- “仅开场 turn、一次性；在 round 1 触发” → Task 1 步骤 5（`turn_id==1 && round==1`）+ 模块文档中的理由。[x]
- “经 `system_reminder` 追加尾部 `<system-reminder>`，用 `Message::user`” → Task 1 步骤 5。[x]
- “在 `prepare()` 中注册，覆盖 TUI + webui” → Task 2。[x]
- “纯提醒文本构造被测试；门控被测试；触发被测试” → Task 1 步骤 3/6。[x]
- “跑既有测试” → Task 1 步骤 7、Task 2 步骤 3。[x]
- 已否决/已推迟项（意图分类、强制加载、全模型、会话中途）→ 未实现，与 spec 的范围外一致。[x]

**占位符扫描：** 无 TBD/TODO。每个代码步骤都给出了确切内容。步骤 3 的 no-op `pre_request` 是刻意的 red 步骤桩，在步骤 5 中被逐字替换。[x]

**类型一致性：** `SkillFirstHook::new(model: &str, has_skills: bool)` 在 Task 1 定义，在 Task 2 以完全相同的方式调用。`body()` 返回 `&'static str`，经 `system_reminder(&str) -> String` 包装，以 `Message::user(String)` 推入。`TurnCtx { turn_id: u64, round: u32, ..Default::default() }` 与 kernel 定义一致。[x]

---

## 执行备注

- 只触及 `rustcode-coding`；没有 `core` 改动，因此无需 `touch core/lib.rs` 那套应对陈旧产物的动作。`#[tokio::test]` 与 `async-trait` 在该 crate 中已可用。
- 合并之后，本次改动在行为效果上是**未真机**发货的 —— deepseek 是否真的会在开场 turn 调用 `use_skill(brainstorming)`，只有用户在真实终端上才能观察到（重新构建 `target/debug/rustcode`，跑 deepseek-v4-flash，发出设计类请求）。按 spec 中那条诚实的局限性说明，该 hook 保证的是送达，而不是模型此后是否遵守 skill 的“一次一个问题”纪律。
