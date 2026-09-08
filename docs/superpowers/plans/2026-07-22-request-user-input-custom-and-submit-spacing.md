# `request_user_input` custom 答案标志 + Submit 行间距 —— 实施计划

> **面向 agentic worker：** 必备子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实施本计划。步骤使用复选框（`- [ ]`）语法进行跟踪。

**目标：** 新增按题目生效的 `custom` 标志（默认 true），当其为 false 时隐藏自动追加的 "type your own answer" 自由文本行；并在 multiple 模式的 Submit 行上方增加一个空行间隔。

**架构：** `custom` 是 `UserInputRequest` 上的新字段（serde 默认 true），一路传递到 `UserInputPanel` 与渲染视图；Other 行是否存在由它控制，涉及状态索引计算、渲染器与事件循环的数字键处理器三处。Submit 间距调整仅影响渲染。内核无需改动。

**技术栈：** Rust（`rustcode-capabilities`、`rustcode-tuix`、`rustcode-daemon`）、React/TS（`webui`）、`cargo test`。

## 全局约束

- `custom` 默认为 `true`（`#[serde(default = ...)]`）—— 缺省即当前行为，对既有调用方零回归。
- Other 行存在当且仅当 `custom == true`。所有曾把它计入的索引（`other_index`、`submit_index`、`last_row`、`checked` 向量长度、渲染行、行数统计、数字键处理器）都必须按 `custom` 做门控。
- 行数不变量：`user_input_panel_row_count(view) == build_user_input_rows(view).len()` 必须在 `custom` 两种取值下、以及新增 multiple 模式空行的情况下都成立。
- Submit 空行仅限 multiple 模式（single 模式没有 Submit 行）。
- 措辞保持中立。在 `release/v5.0.1` 上工作。提交 trailer：`Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`。
- **WIP 提醒：** 实施时 `crates/rustcode-tuix/src/{state.rs, event_loop/mod.rs}` 可能带有与本次无关的未提交改动。只暂存本次改动的 hunk（`git add -p <file>`）；绝不提交无关的 WIP。

---

## 文件结构

- `crates/rustcode-capabilities/src/tools/request_user_input.rs` —— `custom` 字段、serde 默认值、schema、描述。（任务 1）
- `crates/rustcode-tuix/src/state.rs` —— `UserInputPanel.custom` 与索引计算。（任务 2）
- `crates/rustcode-tuix/src/render/mod.rs` 与 `render/retained.rs` —— 门控 Other 行、Submit 前空行、行数统计、视图字段。（任务 3）
- `crates/rustcode-tuix/src/event_loop/mod.rs` —— `custom == false` 时数字键不得跳到 Other 行；构造视图时传入 `custom`。（任务 4）
- `crates/rustcode-daemon/src/live_api.rs`、`webui/src/api.ts`、`webui/src/components/UserInputCard.tsx` —— 转发并遵循 `custom`。（任务 5）

---

### 任务 1：工具层 —— `custom` 字段

**文件：** 修改并测试 `crates/rustcode-capabilities/src/tools/request_user_input.rs`

**接口：**
- 产出：`UserInputRequest` 新增 `pub custom: bool`（serde 默认 true）。任务 2/3/5 读取 `req.custom`。

- [ ] **步骤 1：编写失败测试**

加入 `mod tests`：

```rust
    #[test]
    fn parse_custom_defaults_true_and_reads_false() {
        // Absent → true (backward compatible).
        let r = parse_args(
            r#"{"header":"H","question":"Q?","mode":"single","options":[{"label":"A"}]}"#,
        )
        .unwrap();
        assert!(r.custom, "custom absent → defaults true");
        // Explicit false.
        let r2 = parse_args(
            r#"{"header":"H","question":"Q?","mode":"single","options":[{"label":"A"}],"custom":false}"#,
        )
        .unwrap();
        assert!(!r2.custom, "custom:false parsed");
    }
```

- [ ] **步骤 2：运行以确认它失败**

运行：`cargo test -p rustcode-capabilities --lib parse_custom_defaults_true_and_reads_false`
预期：编译失败（`UserInputRequest` 没有 `custom` 字段）。

- [ ] **步骤 3：加入带 serde 默认值的字段**

在 `crates/rustcode-capabilities/src/tools/request_user_input.rs` 中，加入默认值辅助函数与该字段。在 `UserInputMode` 枚举之后（或结构体靠前的位置）添加：

```rust
fn default_true() -> bool {
    true
}
```

然后修改 `UserInputRequest` 结构体：

```rust
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UserInputRequest {
    pub header: String,
    pub question: String,
    pub mode: UserInputMode,
    #[serde(default)]
    pub options: Vec<UserInputOption>,
    /// Whether the auto "type your own answer" free-text row is offered
    /// (single/multiple). Default true (absent ⇒ true) — backward compatible.
    /// Set false when `options` are exhaustive.
    #[serde(default = "default_true")]
    pub custom: bool,
}
```

本文件测试中任何 `UserInputRequest` 的结构体字面量都必须补上 `custom: true` —— 更新既有的 `roundtrip_serde` 测试字面量以及 `format_batch_*` 测试字面量，使其包含 `custom: true`。

- [ ] **步骤 4：更新 schema 与描述**

在 `parameters_schema` 中，把 `custom` 加入共享的 `question` 对象的 `properties`（这样平铺形式与 `questions[]` 条目同时生效）：

```rust
                "options": { /* unchanged */ },
                "custom": {"type": "boolean", "description": "Offer a free-text 'type your own answer' row (default true). Set false when your options are exhaustive."}
```

更新 `description`：在既有文本后追加：

```
 A free-text \"type your own answer\" row is added automatically for single/multiple unless you set `custom` to false — so do NOT add your own \"Other\"/catch-all option; set `custom:false` when your options already cover every case.
```

- [ ] **步骤 5：运行测试以确认通过**

运行：`cargo test -p rustcode-capabilities --lib request_user_input`
预期：PASS（新测试与既有测试，且字面量已更新）。

- [ ] **步骤 6：提交**（仅暂存本文件）

```bash
git add crates/rustcode-capabilities/src/tools/request_user_input.rs
git commit -m "feat(request_user_input): per-question custom flag (default true)

Add UserInputRequest.custom (serde default true) so the auto 'type your own
answer' row can be suppressed; schema + description tell the model to set
custom:false for exhaustive options and not add its own Other option.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### 任务 2：TUI 状态 —— 用 `custom` 门控 Other 行

**文件：** 修改并测试 `crates/rustcode-tuix/src/state.rs`

**接口：**
- 消费：`UserInputRequest.custom`（任务 1）。
- 产出：`UserInputPanel` 新增 `pub custom: bool`。`submit_index`/`last_row`/`is_other_row`/`checked` 长度均需考虑它。任务 3/4 读取 `panel.custom`。

- [ ] **步骤 1：编写失败测试**

在 `UserInputPanel` 附近添加（在该文件的测试模块内，或新建 `#[cfg(test)] mod`）：

```rust
#[cfg(test)]
mod user_input_custom_tests {
    use super::*;
    use rustcode_capabilities::tools::request_user_input::{
        UserInputMode, UserInputOption, UserInputRequest,
    };

    fn req(mode: UserInputMode, custom: bool) -> UserInputRequest {
        UserInputRequest {
            header: "H".into(),
            question: "Q?".into(),
            mode,
            options: vec![
                UserInputOption { label: "A".into(), description: None },
                UserInputOption { label: "B".into(), description: None },
            ],
            custom,
        }
    }

    #[test]
    fn single_no_custom_row_when_disabled() {
        let p = UserInputPanel::new(1, &req(UserInputMode::Single, false));
        // Only the 2 concrete options are navigable; no Other row.
        p_move_to_bottom(&mut { p.clone() });
        assert!(!UserInputPanel::new(1, &req(UserInputMode::Single, false)).custom);
        let p2 = UserInputPanel::new(1, &req(UserInputMode::Single, false));
        assert_eq!(p2.last_row_for_test(), 1, "single, no custom → last row = last option (idx 1)");
        let p3 = UserInputPanel::new(1, &req(UserInputMode::Single, true));
        assert_eq!(p3.last_row_for_test(), 2, "single, custom → Other row is last (idx 2)");
    }

    #[test]
    fn multiple_submit_index_shifts_without_custom() {
        let with = UserInputPanel::new(1, &req(UserInputMode::Multiple, true));
        assert_eq!(with.submit_index(), Some(3), "custom → other@2, submit@3");
        let without = UserInputPanel::new(1, &req(UserInputMode::Multiple, false));
        assert_eq!(without.submit_index(), Some(2), "no custom → submit right after options@2");
        assert_eq!(without.checked.len(), 2, "no custom → no Other checkbox slot");
        assert_eq!(with.checked.len(), 3, "custom → Other checkbox slot present");
    }
}
```

注意：`last_row` 目前是私有的。为便于测试，可在 `impl UserInputPanel` 中添加 `#[cfg(test)] pub fn last_row_for_test(&self) -> usize { self.last_row() }`，或者把 `last_row` 改为 `pub(crate)`。采用 `pub(crate)` 方案（更简单）：把 `fn last_row` 改为 `pub(crate) fn last_row`，并去掉 `last_row_for_test` 垫片与 `p_move_to_bottom` 那行（删掉那行多余代码）。最终测试直接断言 `p2.last_row()` / `p3.last_row()`。

- [ ] **步骤 2：运行以确认它失败**

运行：`cargo test -p rustcode-tuix --lib user_input_custom`
预期：编译失败（构造 `UserInputRequest` 缺 `custom` 字段没关系 —— 任务 1 已补上；真正的失败是 `UserInputPanel` 没有 `custom` 字段 / `last_row` 为私有）。

- [ ] **步骤 3：把 `custom` 加入结构体与 `new`**

在 `crates/rustcode-tuix/src/state.rs` 中，给 `UserInputPanel` 添加字段（放在 `custom_text` 之后）：

```rust
    /// Whether the always-appended "Other" free-text row is offered. Mirrors
    /// `UserInputRequest.custom`. When false, the Other row does not exist.
    pub custom: bool,
```

在 `UserInputPanel::new` 中读取它，并据此设置 `checked` 的长度：

```rust
        // One checkbox slot per concrete option PLUS the trailing "Other" row —
        // but only when custom answers are offered.
        let checked = vec![false; options.len() + r.custom as usize];
        Self {
            request_id,
            header: r.header.clone(),
            question: r.question.clone(),
            mode: r.mode.clone(),
            options,
            cursor: 0,
            checked,
            text: String::new(),
            custom_text: String::new(),
            custom: r.custom,
        }
```

- [ ] **步骤 4：让索引辅助函数按 `custom` 门控**

修改 `impl UserInputPanel` 中的 `submit_index`、`last_row`、`is_other_row`：

```rust
    /// Index of the Submit row (multiple mode only). After the concrete options,
    /// plus the "Other" row when `custom` is on.
    pub fn submit_index(&self) -> Option<usize> {
        use rustcode_capabilities::tools::request_user_input::UserInputMode;
        if matches!(self.mode, UserInputMode::Multiple) {
            Some(self.options.len() + self.custom as usize)
        } else {
            None
        }
    }

    /// Last navigable cursor index.
    pub(crate) fn last_row(&self) -> usize {
        use rustcode_capabilities::tools::request_user_input::UserInputMode;
        match self.mode {
            UserInputMode::Multiple => self.submit_index().unwrap(),
            // single/text: the "Other" row is last when custom, else the last option.
            _ => {
                if self.custom {
                    self.other_index()
                } else {
                    self.options.len().saturating_sub(1)
                }
            }
        }
    }

    /// Whether `cursor` is on the always-appended "Other" free-text row.
    pub fn is_other_row(&self) -> bool {
        self.custom && self.cursor == self.other_index()
    }
```

`other_index` 保持 `self.options.len()`（仅在 `custom` 为真时有意义）。`build_response` 无需改动：`custom == false` 时 single 模式的光标永远到不了 `options.len()`，multiple 模式下 `custom_text` 保持为空，因此它既有的分支天然正确。

- [ ] **步骤 5：运行测试以确认通过**

运行：`cargo test -p rustcode-tuix --lib user_input_custom`
预期：PASS。

- [ ] **步骤 6：提交**（仅暂存你自己的 hunk）

```bash
git add -p crates/rustcode-tuix/src/state.rs
git commit -m "feat(tuix): gate the Other free-text row on UserInputPanel.custom

When custom is false the Other row does not exist: submit_index, last_row,
is_other_row and the checked-vec length all drop it. custom=true is unchanged.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### 任务 3：TUI 渲染 —— 门控 Other 行、Submit 前空行、行数统计

**文件：** 修改 `crates/rustcode-tuix/src/render/mod.rs`（视图结构体）、`crates/rustcode-tuix/src/render/retained.rs`（`build_user_input_rows`、`user_input_panel_row_count`）。

**接口：**
- 消费：`panel.custom`（任务 2）。
- 产出：`UserInputPanelView` 新增 `pub custom: bool`。仅在 `custom` 为真时渲染 Other 行；在 multiple 模式的 Submit 行之前增加一个空行。

- [ ] **步骤 1：把 `custom` 加入视图结构体**

在 `render/mod.rs` 中，给 `UserInputPanelView` 添加字段（`custom_text` 之后、`batch` 之前）：

```rust
    /// Whether to render the "Other" free-text row (mirrors UserInputPanel.custom).
    pub custom: bool,
```

更新 `retained.rs` 中 3 处 `UserInputPanelView` 的测试构造（在测试模块中 grep `custom_text: `）以补上 `custom: true,`，批量渲染测试的 `base` 闭包同理。

- [ ] **步骤 2：阅读当前渲染器**

运行：`sed -n '2515,2560p' crates/rustcode-tuix/src/render/retained.rs`，并阅读 `build_user_input_rows`（选项循环、Other 行代码块、multiple 模式的 Submit 代码块）。

- [ ] **步骤 3：在 `build_user_input_rows` 中门控 Other 行并加入 Submit 空行**

在 `build_user_input_rows` 的 single/multiple 分支中：
- 把整个 "Always-appended custom-answer row" 代码块（`{ let idx = other_index; ... out.push(row); }`）包进 `if panel.custom { ... }`。
- 在 multiple 模式的 Submit 代码块之前，压入一个空行占位行：`if multiple { blank_row(&mut out); /* then the existing Submit row */ }`。Submit 行的 `on_cursor` 索引必须使用 `submit_index = panel.options.len() + panel.custom as usize`（原来是 `other_index + 1`）。
- 提示行中的 `n`（可导航行数）改为 `panel.options.len() + panel.custom as usize`（原来是 `+ 1`）。

- [ ] **步骤 4：同步修改 `user_input_panel_row_count`**

在 `user_input_panel_row_count` 中，对 single/multiple：
- 把 custom 行那句无条件的 `n += 1;` 改为 `if panel.custom { n += 1; }`。
- 对 multiple，Submit 部分的贡献从 `+1` 变为 `+2`（空行 + Submit）。

具体来说，single/multiple 分支变为：

```rust
            UserInputMode::Single | UserInputMode::Multiple => {
                let mut n = 4; // header chip + blank + question + blank
                for (_, desc) in &panel.options {
                    n += 1;
                    if desc.as_deref().map(|d| !d.trim().is_empty()).unwrap_or(false) {
                        n += 1;
                    }
                }
                if panel.custom {
                    n += 1; // the Other row
                }
                if matches!(panel.mode, UserInputMode::Multiple) {
                    n += 2; // blank spacer + Submit row
                }
                n += 2; // blank + hint
                n
            }
```

- [ ] **步骤 5：把 `custom` 接入视图构造**

（推迟到任务 4 的事件循环改动，那里构造 `UserInputPanelView` —— 但如果 `render/` 的非测试代码中存在 `UserInputPanelView { .. }` 字面量，则补上 `custom: panel.custom`。）本任务只做结构体字段 + 测试 + 渲染器改动；生产代码的构造位于 event_loop（任务 4）。

- [ ] **步骤 6：更新/扩充测试并运行**

更新既有的 `user_input_panel_renders_all_three_modes` 中 multiple 模式的行数期望值（因新增空行而 +1）。添加一个 `custom: false` 用例，断言 Other 行不存在且不变量成立：

```rust
        // custom == false: no Other row; row_count still matches build.
        let no_custom = crate::render::UserInputPanelView { custom: false, ..view_multiple.clone() };
        assert_eq!(
            r.build_user_input_rows(&no_custom, 78, 80).len(),
            r.user_input_panel_row_count(&no_custom),
            "row_count invariant holds with custom=false"
        );
```

运行：`cargo test -p rustcode-tuix --lib user_input`
预期：PASS（`custom` 两种取值与新增空行下行数不变量均成立）。

- [ ] **步骤 7：提交**（仅暂存 render/ 的 hunk）

```bash
git add crates/rustcode-tuix/src/render/mod.rs crates/rustcode-tuix/src/render/retained.rs
git commit -m "feat(tuix): render Other row only when custom + blank before Submit

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### 任务 4：TUI 事件 —— 数字键处理器与视图构造

**文件：** 修改 `crates/rustcode-tuix/src/event_loop/mod.rs`。

**接口：**
- 消费：`panel.custom`（任务 2）。
- 产出：`custom == false` 时数字键无法跳到不存在的 Other 行；`UserInputPanelView` 构造时传入 `custom`。

- [ ] **步骤 1：门控数字键处理器（single 与 batch）**

在 `handle_user_input_key`（single）与 `handle_user_input_batch_key`（batch）中，数字键分支里是 `if idx == p.other_index() { p.cursor = p.other_index(); }`。两处都用 `custom` 加保护：

```rust
                if idx < p.options.len() {
                    match p.mode { /* Multiple → toggle_index(idx); _ => cursor = idx */ }
                } else if idx == p.other_index() && p.custom {
                    p.cursor = p.other_index();
                }
```

（即仅当 `p.custom` 为真时，才把第 N+1 个数字当作 Other 行；否则忽略它。）把既有的外层 `if idx <= p.other_index()` 判定改为 `if idx < p.options.len() || (idx == p.other_index() && p.custom)`。

- [ ] **步骤 2：在视图构造中传入 `custom`**

在 `event_loop/mod.rs` 的 `UserInputPanelView { .. }` 构造处（单面板的 `.map(|p| ...)` 与批量分支），分别添加 `custom: p.custom,`（single）以及 `custom: p.custom,`（batch，其中 `p = &b.questions[idx]`）。

- [ ] **步骤 3：构建并运行测试套件**

运行：`cargo build -p rustcode-tuix && cargo test -p rustcode-tuix`
预期：编译通过；整套测试为绿（单问题默认 `custom=true` 行为不变；multiple 模式空行已在任务 3 的测试中更新）。

- [ ] **步骤 4：提交**（仅暂存你自己的 hunk）

```bash
git add -p crates/rustcode-tuix/src/event_loop/mod.rs
git commit -m "feat(tuix): don't jump to the Other row via number keys when custom=false; pass custom to the view

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### 任务 5：daemon 与 webui —— 转发并遵循 `custom`

**文件：** 修改 `crates/rustcode-daemon/src/live_api.rs`、`webui/src/api.ts`、`webui/src/components/UserInputCard.tsx`。

**接口：**
- 消费：请求载荷上的 `custom` 字段（任务 1）。
- 产出：`custom === false` 时 webui 隐藏 Other 单选/复选与自由文本输入。

- [ ] **步骤 1：daemon —— 在单问题事件上转发 `custom`**

在 `live_api.rs` 的 `LiveWireEvent::UserInputRequest` 投影中：给事件添加 `custom: bool` 字段（默认 true），取值来自 `request.payload.get("custom").and_then(Value::as_bool).unwrap_or(true)`。（批量路径已经在每个 `questions[]` 条目内携带 `custom`。）

- [ ] **步骤 2：webui 类型**

在 `api.ts` 中：给 `UserInputQuestion` 与 `UserInputRequestEvent` 添加 `custom?: boolean`。

- [ ] **步骤 3：webui —— 在 `QuestionBody`/`SingleCard` 中遵循 `custom`**

在 `UserInputCard.tsx` 中计算 `const showOther = q.custom !== false;`，仅在 `showOther` 为真时渲染 "Other" 单选框（single）/复选框（multiple）与自由文本输入。对单卡片，`q.custom` 来自 `req.custom`；对批量步进器，来自 `req.questions[step].custom`。

- [ ] **步骤 4：类型检查**

运行：`cd webui && npx tsc --noEmit`
预期：无错误。

- [ ] **步骤 5：提交**

```bash
git add crates/rustcode-daemon/src/live_api.rs webui/src/
git commit -m "feat(daemon,webui): forward + honor request_user_input custom flag

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## 自审

**规格覆盖：**
- `custom` 字段 + serde 默认值 + schema + 描述 → 任务 1。[x]
- TUI 状态按 Other 行做门控（索引计算）→ 任务 2。[x]
- 渲染门控 Other 行 + Submit 前空行 + 行数统计 → 任务 3。[x]
- 事件循环数字键处理器 + 视图 `custom` → 任务 4。[x]
- daemon 转发 + webui 遵循 → 任务 5。[x]
- `custom` 两种取值 + 新增空行下的行数不变量 → 任务 3 步骤 4/6。[x]
- 既有 multiple 模式行数测试已按 +1 空行更新 → 任务 3 步骤 6。[x]
- WIP 暂存提醒 → 全局约束及任务 2/4 中的 `git add -p`。[x]

**占位符扫描：** 任务 1-2 给出了完整代码；3-5 给出了精确的锚点与具体的门控/改动（它们修改的是既有的大函数，因此展示的是增量而非 300 行渲染器的全貌）。没有 "TBD"/"handle edge cases" 这类表述。

**类型一致性：** `custom: bool` 在 `UserInputRequest`（任务 1）、`UserInputPanel`（任务 2）、`UserInputPanelView`（任务 3）、webui 类型（任务 5）上用法一致。`submit_index = options.len() + custom as usize` 在状态层（任务 2）与渲染层（任务 3）中写法完全相同。

---

## 执行说明

- 任务 2 与 4 会触及 `state.rs` / `event_loop/mod.rs`，其中可能含有无关的 WIP —— 使用 `git add -p`，只暂存本计划描述的 hunk。
- 视觉效果（Other 行在 `custom:false` 时隐藏；Submit 上方空行）标注为 **未真机**。请在真实终端 / webui 中提出一个 `custom:false`（选项已穷举）的问题以及一个多选题来验证。
