# 批量用户提问的 Persona 提示 —— 实施计划

> **致 agentic worker：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实施本计划。步骤使用复选框（`- [ ]`）语法进行跟踪。

**目标：** 在 persona 里加一条规则，让模型把多个用户问题放进**一次** `request_user_input` 调用的 `questions[]` 数组（即已发布的、用 Tab 导航的批量表单），而不是发出 N 次独立的单问题调用。

**架构：** 在 coding persona 中已被开关控制的 `REQUEST_USER_INPUT_USAGE` 段里加一条条款。无代码/机制改动——复用已发布的批量 UI。依托既有的 `request_user_input_enabled` 开关，因此在工具关闭时该条款会随之消失。

**技术栈：** Rust、`rustcode-coding` crate、`cargo test`。

## 全局约束

- 该条款位于 `REQUEST_USER_INPUT_USAGE` **内部**（`crates/rustcode-coding/src/persona.rs:336`），因此只在 `request_user_input_enabled == true` 时出现（绝不引导模型去使用未挂载的工具）。
- 与模型无关（本轮不做按模型的开关）。
- 必须与既有的 "One focused question at a time" 措辞调和——每个问题仍然保持聚焦，但多个聚焦的问题要放进**一次**调用。
- 措辞中立——代码与提交里不得出现 opencode/codex 的名字。
- 在分支 `release/v5.0.1` 上工作。提交尾部：`Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`。

---

## 文件结构

- `crates/rustcode-coding/src/persona.rs` —— 唯一改动的文件。修改 `REQUEST_USER_INPUT_USAGE` 字符串常量（约 336 行），并在 `mod tests` 里加一个验证开关行为的单元测试。

---

### 任务 1：给 `## ASKING THE USER` 加上批量提问规则

**文件：**
- 修改：`crates/rustcode-coding/src/persona.rs` —— `REQUEST_USER_INPUT_USAGE` 常量（约 336 行）与 `mod tests`。

**接口：**
- 消费：既有的 `coding_persona(model: &str, todo_enabled: bool, request_user_input_enabled: bool) -> String`。签名不变。
- 产出：行为上——当 `request_user_input_enabled == true` 时，persona 额外包含子串 `answers them together in one form`；为 `false` 时则没有。

- [ ] **步骤 1：写失败测试**

加到 `crates/rustcode-coding/src/persona.rs` 的 `mod tests` 里（放在其它 `request_user_input` persona 测试附近）：

```rust
    #[test]
    fn batch_questions_rule_present_only_when_enabled() {
        let on = coding_persona("deepseek-v4-flash", false, true);
        assert!(
            on.contains("answers them together in one form"),
            "enabled → batching rule present"
        );
        assert!(
            on.contains("`questions` array"),
            "enabled → names the questions array"
        );
        let off = coding_persona("deepseek-v4-flash", false, false);
        assert!(
            !off.contains("answers them together in one form"),
            "disabled → batching rule gone with the whole block"
        );
    }
```

- [ ] **步骤 2：跑测试确认失败**

运行：`cargo test -p rustcode-coding --lib batch_questions_rule_present_only_when_enabled`
预期：FAIL —— `on.contains("answers them together in one form")` 会 panic（该条款还没写进常量）。

- [ ] **步骤 3：插入批量提问条款**

在 `crates/rustcode-coding/src/persona.rs` 里，`REQUEST_USER_INPUT_USAGE` 常量中有 `One focused question at a time.` 这一句，其后是 `Never ask the user to type a secret`。把批量条款插在两者之间。将：

```rust
code, the task, or a quick check already answers. One focused question at a time. Never ask the \
user to type a secret (password, API key, token) into the prompt — those come from the \
```

替换为：

```rust
code, the task, or a quick check already answers. Keep each question focused. If you have MORE \
THAN ONE question for the user at this point, put them ALL into ONE `request_user_input` call's \
`questions` array — do NOT make several `request_user_input` calls in the same turn, and never \
write a multiple-choice question as prose; the user answers them together in one form. Never ask \
the user to type a secret (password, API key, token) into the prompt — those come from the \
```

（这删掉了独立的 "One focused question at a time." 一句，并把 "Keep each question focused" 折进批量规则，使两者不再被读成「要分多次调用」。）

- [ ] **步骤 4：跑测试确认通过 + 无 persona 回归**

运行：`cargo test -p rustcode-coding --lib persona`
预期：PASS —— 新增的 `batch_questions_rule_present_only_when_enabled` 以及全部既有 persona 测试（`## ASKING THE USER` 段仍包含 `## ASKING THE USER`、`request_user_input`、`structured interview` 等）。

- [ ] **步骤 5：确认该字符串已编译进二进制（可选的健康检查）**

运行：`cargo build --bin rustcode && strings target/debug/rustcode | grep -c "answers them together in one form"`
预期：输出 `1`。

- [ ] **步骤 6：提交**

```bash
git add crates/rustcode-coding/src/persona.rs
git commit -m "feat(persona): batch multiple user questions into one request_user_input call

A weak model (deepseek) emitted 3 separate request_user_input calls instead of one
questions[] batch, so each was a standalone single-question panel with no
back-navigation. Add a rule to the gated ASKING THE USER block: put multiple
questions into ONE request_user_input call's questions array, never several calls
in a turn, never a multiple-choice question as prose. Reuses the shipped batch UI;
mirrors how comparable agents guide the model (tool takes an array + prompt rule,
no runtime coalescing).

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## 自审

**Spec 覆盖：**
- 「在受控的 `REQUEST_USER_INPUT_USAGE` 内加批量规则」→ 任务 1 步骤 3。 [x]
- 「与 'One focused question at a time' 调和」→ 步骤 3 把它折进 "Keep each question focused"。 [x]
- 「与模型无关、无机制改动」→ 只改了常量字符串 + 一个测试。 [x]
- 「Persona 测试：开启时存在 / 关闭时消失」→ 步骤 1。 [x]
- 「跑既有 persona 测试」→ 步骤 4。 [x]
- 运行时合并已延后 → 未实现，与 spec 一致。 [x]

**占位符扫描：** 无 TBD/TODO。每个步骤都给出确切字符串。 [x]

**类型一致：** 无签名变更。测试断言 `answers them together in one form` 与 `` `questions` array ``，两者都逐字出现在步骤 3 插入的文本里。 [x]

---

## 执行说明

- 只触碰 `rustcode-coding/persona.rs`；没有 `core` 改动，也不需要防陈旧操作。
- 行为效果属于**未真机**就发出的——验证方式：向 deepseek/GLM 提一个会引出多个选项的问题，确认只出现**一次**带 `questions[]` 的 `request_user_input`（Tab 表单），而不是 N 次调用。若 deepseek 仍不批量提问，则升级到已延后的运行时合并兜底方案。
