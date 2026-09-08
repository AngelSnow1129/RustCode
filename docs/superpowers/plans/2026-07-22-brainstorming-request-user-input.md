# Brainstorming → request_user_input Persona 桥接 —— 实施计划

> **致 agentic worker：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实施本计划。步骤使用复选框（`- [ ]`）语法进行跟踪。

**目标：** 让模型把技能驱动的 brainstorming/interview 里的选择题，通过既有的 `request_user_input` 工具（可在 TUI/webui 界面中作答）提出，而不是把它们写成散文式的问题。

**架构：** 纯 prompt 改动。全部机制（工具、TUI 面板、webui 弹窗、kernel 往返、环境变量开关、`coding_persona` 参数 + 调用点）都已存在并已接好。唯一的缺口是：在 brainstorming 过程中，persona 里的 `## SKILLS:` 与 `## ASKING THE USER:` 两段没有衔接——「尽量少问」的措辞会被读成「探索性问题不要用这个工具」的理由。我们在已被开关控制的 `REQUEST_USER_INPUT_USAGE` 段内加一句桥接条款。

**技术栈：** Rust、`rustcode-coding` crate、`cargo test`。

## 全局约束

- 改动位于 `REQUEST_USER_INPUT_USAGE` **内部**（`crates/rustcode-coding/src/persona.rs:303`），因此自动受既有的 `request_user_input_enabled` 开关管辖——当工具关闭（`RUSTCODE_REQUEST_USER_INPUT=0`）时，该条款必须随整段一起消失。绝不引导模型去使用未挂载的工具。
- 不新增函数参数、不新增调用点、不改动外部 superpowers 技能文件。
- webui 的 `/chat` 路径（`build_api_system_prompt`，不使用 `coding_persona`）本轮明确不在范围内。
- 不得削弱针对模型**自身**临时提问的通用稀缺性规则；新条款的适用范围限定为「由某个技能在驱动这轮问答」。
- 提交信息尾部：`Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`。当前分支是 `release/v5.0.1`；就提交在该分支上（它已是工作分支）。

---

## 文件结构

- `crates/rustcode-coding/src/persona.rs` —— 唯一改动的生产文件。修改 `REQUEST_USER_INPUT_USAGE` 字符串常量（约 303 行）。在既有的 `mod tests` 里加一个测试（约 406 行）。

不新增文件。

---

### 任务 1：给 `## ASKING THE USER` 加上 brainstorming 桥接条款

**文件：**
- 修改：`crates/rustcode-coding/src/persona.rs` —— `REQUEST_USER_INPUT_USAGE` 常量（约 303–311 行）
- 测试：`crates/rustcode-coding/src/persona.rs` —— `mod tests`（约 406 行）

**接口：**
- 消费：既有的 `coding_persona(model: &str, todo_enabled: bool, request_user_input_enabled: bool) -> String`。签名不变。
- 产出：无新符号。行为上：当 `request_user_input_enabled == true` 时，persona 字符串额外包含子串 `structured interview`；为 `false` 时则不包含（这已由开关保证）。

- [ ] **步骤 1：写失败测试**

在 `crates/rustcode-coding/src/persona.rs` 的 `mod tests` 内加下面这个测试（例如放在既有的 `request_user_input_guidance_gated` 测试之后，约 425 行处）：

```rust
    #[test]
    fn brainstorming_bridge_present_only_when_enabled() {
        let on = coding_persona("deepseek-v4-flash", false, true);
        assert!(
            on.contains("structured interview"),
            "enabled → brainstorming bridge clause present"
        );
        assert!(
            on.contains("brainstorming"),
            "enabled → clause names the brainstorming case"
        );
        let off = coding_persona("deepseek-v4-flash", false, false);
        assert!(
            !off.contains("structured interview"),
            "disabled → bridge clause gone with the whole block"
        );
    }
```

- [ ] **步骤 2：跑测试确认失败**

运行：`cargo test -p rustcode-coding brainstorming_bridge_present_only_when_enabled`
预期：FAIL —— `on.contains("structured interview")` 这条断言会 panic（`enabled → brainstorming bridge clause present`），因为该条款还没写进常量。

- [ ] **步骤 3：把桥接条款追加到常量末尾**

在 `crates/rustcode-coding/src/persona.rs` 里，`REQUEST_USER_INPUT_USAGE` 常量目前的结尾是这样的：

```rust
for what you genuinely cannot decide, look up, or verify yourself — never for something the \
code, the task, or a quick check already answers. One focused question at a time. Never ask the \
user to type a secret (password, API key, token) into the prompt — those come from the \
environment or a secrets store, not a question.";
```

把最后一行改成让字符串继续（而不是收尾），然后追加该条款。将：

```rust
environment or a secrets store, not a question.";
```

替换为：

```rust
environment or a secrets store, not a question. \
When a skill (for example brainstorming) is driving a round of clarifying, interview-style \
questions to refine a design, surface ITS questions through this tool too: use `single` or \
`multiple` with concrete `options` for choice questions and `text` for an open answer, so the \
user answers in the UI instead of reading a prose question. The 'ask sparingly, only for what \
you cannot decide yourself' guidance above governs YOUR OWN ad-hoc questions; it does not \
constrain a skill's structured interview.";
```

- [ ] **步骤 4：跑测试确认通过**

运行：`cargo test -p rustcode-coding brainstorming_bridge_present_only_when_enabled`
预期：PASS。

- [ ] **步骤 5：跑完整 persona 测试套件（无回归）**

运行：`cargo test -p rustcode-coding persona`
预期：全部 persona 测试 PASS（包括既有的 `request_user_input_guidance_gated`；因为该条款位于同一个受控段内，它依然成立）。

- [ ] **步骤 6：提交**

```bash
git add crates/rustcode-coding/src/persona.rs
git commit -m "feat(persona): bridge brainstorming questions to request_user_input

The SKILLS block tells the model to let a skill drive the questions; the
ASKING THE USER block frames request_user_input as a scarce gate. During
brainstorming those two don't connect, so the model writes prose questions
instead of surfacing them in the UI. Add one bridging clause inside the
already-gated REQUEST_USER_INPUT_USAGE block: when a skill is driving a
clarifying/interview flow, route its choice questions through the tool
(single/multiple with options; text for open), while leaving the scarcity
rule for the model's own ad-hoc questions intact.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### 任务 2（可选）：从 `## SKILLS` 反向引用该桥接

价值低、风险低的润色。`SKILLS_USAGE` 段已经说了 brainstorming 应该「让它来主导提问」；这里加一个指引，使两段互相引用。如果你更想要最小的 diff，可以跳过——任务 1 本身已能独立成立。

**文件：**
- 修改：`crates/rustcode-coding/src/persona.rs` —— `SKILLS_USAGE` 常量（约 288–296 行）

**接口：**
- 消费：无新增。
- 产出：无新符号。行为上：给始终存在的 `## SKILLS` 段增加子串 `answer in the UI`。

- [ ] **步骤 1：写失败测试**

在 `mod tests` 内加：

```rust
    #[test]
    fn skills_block_points_at_ui_answering() {
        // Always-present block, independent of the request_user_input gate.
        let p = coding_persona("m", true, false);
        assert!(
            p.contains("answer in the UI"),
            "SKILLS block cross-references answering skill questions in the UI"
        );
    }
```

- [ ] **步骤 2：跑测试确认失败**

运行：`cargo test -p rustcode-coding skills_block_points_at_ui_answering`
预期：在 `p.contains("answer in the UI")` 处 FAIL。

- [ ] **步骤 3：把指引追加到 `SKILLS_USAGE`**

`SKILLS_USAGE` 常量目前的结尾是：

```rust
use the minimal set; if none match, proceed normally.";
```

替换为：

```rust
use the minimal set; if none match, proceed normally. When the loaded skill runs an interview \
(for example brainstorming asking questions to refine a design), let the user answer in the UI: \
prefer `request_user_input` for its choice questions when that tool is available.";
```

- [ ] **步骤 4：跑测试确认通过**

运行：`cargo test -p rustcode-coding skills_block_points_at_ui_answering`
预期：PASS。

- [ ] **步骤 5：跑完整 persona 套件**

运行：`cargo test -p rustcode-coding persona`
预期：全部 PASS。

- [ ] **步骤 6：提交**

```bash
git add crates/rustcode-coding/src/persona.rs
git commit -m "feat(persona): cross-reference UI answering from the SKILLS block

Point the SKILLS guidance at request_user_input for skill-driven interviews
so the SKILLS and ASKING THE USER blocks reference each other.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## 自审

**Spec 覆盖：**
- 「在 `REQUEST_USER_INPUT_USAGE` 内加桥接条款，受既有开关管辖」→ 任务 1。 [x]
- 「不新增参数/调用点/外部技能改动」→ 已遵守（只改了常量字符串 + 一个测试）。 [x]
- 「`SKILLS_USAGE` 里的可选指引」→ 任务 2，已标注为可选。 [x]
- 「Persona 单测：开启时存在、关闭时消失」→ 任务 1 步骤 1。 [x]
- 「跑既有 persona 测试，保持全绿」→ 任务 1 步骤 5。 [x]
- 「webui `/chat` 延后」→ 未实现，与 spec 中「不在范围内」一致。 [x]
- 「真实验证靠人工 / 未真机」→ 没有自动化的真机步骤；正确，由用户验证。 [x]

**占位符扫描：** 无 TBD/TODO；每个代码步骤都给出了确切字符串。 [x]

**类型一致：** 无签名变更。测试断言的是字面子串 `structured interview`，它逐字出现在步骤 3 追加的文本里。任务 2 断言 `answer in the UI`，它同样逐字出现在其步骤 3 的文本里。 [x]

---

## 执行说明

- 两个任务都只触碰 `crates/rustcode-coding/src/persona.rs`。该 crate 无需特殊 feature flag 即可构建和测试（`persona.rs` 在默认构建里）；因为 `core` 未被改动，所以不需要 `touch core/lib.rs` 那套防陈旧的操作。
- 合并之后，本改动是未经真机验证（「未真机」）就发出的——其行为效果（在真实的 brainstorming 会话中出现面板）只有用户在真机终端上才能观察到。
