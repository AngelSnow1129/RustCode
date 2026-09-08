# 多问题 `request_user_input` 实施计划

> **致 agentic worker：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实施本计划。步骤使用复选框（`- [ ]`）语法进行跟踪。

**目标：** 让一次 `request_user_input` 调用在一次交互中提出最多 4 个问题 —— TUI 里是可按 Tab 导航的表单，webui 里是顺序分步器 —— 并作为一次批量响应提交。

**架构：** 内核不改（请求/响应载荷是不透明的 `serde_json::Value`）。工具发出 `{questions:[...]}`，各 driver 收集答案并回 `{responses:[...]}`，工具按每个问题格式化一行输出。TUI 复用 `UserInputPanel` 作为新问题封装 `UserInputBatch` 里每道题的状态；webui 则逐题推进，复用它现有的单问题卡片，并在最后提交一次批量结果。

**技术栈：** Rust（`rustcode-capabilities`、`rustcode-tuix`、`rustcode-daemon`）、React/TS（`webui`）、`cargo test`。

## 全局约束

- **向后兼容。** 采用旧扁平结构（`header/question/mode/options`，没有 `questions`）的调用，其单问题的线路、UI 与结果完全保持现状。只有非空的 `questions` 数组才会激活批量路径。
- **最多 4 个问题。** 截断到前 4 个（`MAX_QUESTIONS = 4`）。
- **部分提交（B）。** 用户从未作答的问题以 `declined` 返回。`Esc` 拒绝整批。
- **范围 B。** TUI 获得完整的 Tab 表单；webui 一次一张卡片地逐题推进，并在最后提交一次批量响应。本轮不做 webui 的并行表单。
- **N==1 不得退化。** 单问题交互（无论旧的扁平结构，还是只有 1 个元素的 `questions`）在渲染与行为上都与今天完全一致。
- 在分支 `release/v5.0.1` 上工作。提交 trailer：`Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`。

---

## 文件结构

- `crates/rustcode-capabilities/src/tools/request_user_input.rs` —— 工具层：批量解析、批量格式化、schema、execute。（任务 1）
- `crates/rustcode-tuix/src/state.rs` —— 包裹 `UserInputPanel` 的 `UserInputBatch`。（任务 2）
- `crates/rustcode-tuix/src/render/mod.rs` + `render/retained.rs` —— 批量导航器 + 复用每道题的行渲染。（任务 3）
- `crates/rustcode-tuix/src/event_loop/mod.rs` —— Tab/Shift+Tab、请求解析、批量投递。（任务 4）
- `crates/rustcode-daemon/src/live_api.rs` —— 批量响应体 + `questions` 投影。（任务 5）
- `webui/src/components/UserInputCard.tsx` + `webui/src/api.ts` —— 顺序分步器 + 批量 POST。（任务 6）

---

### 任务 1：工具层 —— 批量解析、格式化、执行

**文件：**
- 修改：`crates/rustcode-capabilities/src/tools/request_user_input.rs`
- 测试：同一文件（`#[cfg(test)] mod tests`）

**接口：**
- 消费：既有的 `UserInputRequest`、`UserInputResponse`、`UserInputMode`、`parse_args`、`format_result`、`ok_result`/`err_result`/`null_result`。
- 产出：`pub const MAX_QUESTIONS: usize`；`pub fn parse_batch(args: &str) -> Result<(Vec<UserInputRequest>, bool), String>`（bool 表示是否为批量）；`pub fn format_batch_result(reqs: &[UserInputRequest], resps: &[UserInputResponse]) -> ToolResult`。任务 4/5 依赖这些线路结构：请求 `{ "questions": [UserInputRequest,...] }`，响应 `{ "responses": [UserInputResponse,...] }`。

- [ ] **步骤 1：编写失败测试**

添加到 `mod tests`：

```rust
    #[test]
    fn parse_batch_reads_questions_array_and_clamps_to_four() {
        let args = r#"{"questions":[
            {"header":"A","question":"Q1?","mode":"single","options":[{"label":"x"}]},
            {"header":"B","question":"Q2?","mode":"text"},
            {"header":"C","question":"Q3?","mode":"text"},
            {"header":"D","question":"Q4?","mode":"text"},
            {"header":"E","question":"Q5?","mode":"text"}
        ]}"#;
        let (reqs, is_batch) = parse_batch(args).unwrap();
        assert!(is_batch);
        assert_eq!(reqs.len(), 4, "clamped to MAX_QUESTIONS");
        assert_eq!(reqs[0].header, "A");
    }

    #[test]
    fn parse_batch_falls_back_to_single_legacy_shape() {
        let (reqs, is_batch) =
            parse_batch(r#"{"header":"H","question":"Q?","mode":"text"}"#).unwrap();
        assert!(!is_batch);
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].mode, UserInputMode::Text);
    }

    #[test]
    fn parse_batch_validates_each_question_options() {
        let args = r#"{"questions":[{"header":"A","question":"Q?","mode":"single","options":[]}]}"#;
        assert!(parse_batch(args).is_err(), "choice question needs options");
    }

    #[test]
    fn format_batch_keys_each_line_by_header_and_declines_untouched() {
        let reqs = vec![
            UserInputRequest { header: "Auth".into(), question: "?".into(), mode: UserInputMode::Single, options: vec![UserInputOption{label:"OAuth".into(),description:None}] },
            UserInputRequest { header: "Note".into(), question: "?".into(), mode: UserInputMode::Text, options: vec![] },
        ];
        let resps = vec![
            UserInputResponse { declined: false, selected: vec!["OAuth".into()], text: None },
            UserInputResponse::declined(),
        ];
        let out = format_batch_result(&reqs, &resps).content;
        assert_eq!(out, "Q1 (Auth): User selected: \"OAuth\"\nQ2 (Note): No answer (declined).");
    }

    #[test]
    fn format_batch_all_declined_is_the_single_no_answer_guidance() {
        let reqs = vec![UserInputRequest { header: "A".into(), question: "?".into(), mode: UserInputMode::Text, options: vec![] }];
        let out = format_batch_result(&reqs, &[UserInputResponse::declined()]);
        assert!(!out.is_error);
        assert!(out.content.starts_with("No answer was provided."));
    }
```

- [ ] **步骤 2：运行测试以确认它们失败**

运行：`cargo test -p rustcode-capabilities --lib request_user_input`
预期：FAIL，无法编译（`parse_batch` / `format_batch_result` / `MAX_QUESTIONS` 未定义）。

- [ ] **步骤 3：实现 `MAX_QUESTIONS`、`validate_question`、`parse_batch`、`format_batch_result`**

重构 `parse_args` 以共享校验逻辑，并添加批量函数。把既有的 `parse_args`（55-67 行）替换为：

```rust
/// Max questions a single batch may pose.
pub const MAX_QUESTIONS: usize = 4;

fn validate_question(req: &UserInputRequest) -> Result<(), String> {
    if matches!(req.mode, UserInputMode::Single | UserInputMode::Multiple) && req.options.is_empty()
    {
        return Err(
            "request_user_input: single/multiple mode requires a non-empty `options` array".into(),
        );
    }
    Ok(())
}

/// Parse raw tool args into a `UserInputRequest`. Rejects choice modes with no options.
/// Returns a human message on failure (never panics).
pub fn parse_args(args: &str) -> Result<UserInputRequest, String> {
    let req: UserInputRequest = serde_json::from_str(args)
        .map_err(|e| format!("invalid request_user_input arguments: {e}"))?;
    validate_question(&req)?;
    Ok(req)
}

/// Parse args into 1..=`MAX_QUESTIONS` questions. Accepts a `{ "questions": [...] }`
/// array (batch) or the flat single-question shape (legacy). The bool is `is_batch`
/// — the caller uses it to pick the wire shape. Clamps a batch to `MAX_QUESTIONS`.
pub fn parse_batch(args: &str) -> Result<(Vec<UserInputRequest>, bool), String> {
    let val: serde_json::Value = serde_json::from_str(args)
        .map_err(|e| format!("invalid request_user_input arguments: {e}"))?;
    if let Some(qs) = val.get("questions").and_then(serde_json::Value::as_array) {
        if qs.is_empty() {
            return Err("request_user_input: `questions` must be a non-empty array".into());
        }
        let mut out = Vec::new();
        for q in qs.iter().take(MAX_QUESTIONS) {
            let req: UserInputRequest = serde_json::from_value(q.clone())
                .map_err(|e| format!("invalid question in `questions`: {e}"))?;
            validate_question(&req)?;
            out.push(req);
        }
        Ok((out, true))
    } else {
        Ok((vec![parse_args(args)?], false))
    }
}

/// Map one question's response to its answer clause (shared by single + batch).
fn answer_clause(resp: &UserInputResponse) -> String {
    if resp.declined {
        return "No answer (declined).".to_string();
    }
    if let Some(t) = &resp.text {
        return format!("User answered: {t:?}");
    }
    if resp.selected.is_empty() {
        return "User selected nothing.".to_string();
    }
    let joined = resp
        .selected
        .iter()
        .map(|s| format!("{s:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!("User selected: {joined}")
}

/// Format a batch of answers, one line per question keyed by its `header`. When every
/// question was declined, degrade to the same "no answer" guidance a single decline gives.
pub fn format_batch_result(reqs: &[UserInputRequest], resps: &[UserInputResponse]) -> ToolResult {
    if resps.iter().all(|r| r.declined) && resps.len() >= reqs.len() {
        return ok_result(
            "No answer was provided. Proceed with your own best judgment; only ask again if you \
             are truly blocked.",
        );
    }
    let lines: Vec<String> = reqs
        .iter()
        .enumerate()
        .map(|(i, req)| {
            let clause = resps
                .get(i)
                .map(answer_clause)
                .unwrap_or_else(|| "No answer (declined).".to_string());
            format!("Q{} ({}): {}", i + 1, req.header, clause)
        })
        .collect();
    ok_result(lines.join("\n"))
}
```

注意：`format_result`（单题）可以选择性地简化为复用 `answer_clause`，但为避免无谓改动就保持原样 —— 它的确切字符串被既有测试断言着。

- [ ] **步骤 4：重写 `execute`，按批量/单题分流**

把 `execute`（156-173 行）替换为：

```rust
    async fn execute(&self, args: &str, ctx: &ToolContext) -> ToolResult {
        let (reqs, is_batch) = match parse_batch(args) {
            Ok(x) => x,
            Err(e) => return err_result(e),
        };
        if !is_batch {
            // Legacy single-question path — wire + result unchanged.
            let payload = match serde_json::to_value(&reqs[0]) {
                Ok(v) => v,
                Err(e) => return err_result(format!("request_user_input: serialize failed: {e}")),
            };
            let resp_val = ctx.request(REQUEST_USER_INPUT_KIND, payload).await;
            if resp_val.is_null() {
                return null_result();
            }
            return match serde_json::from_value::<UserInputResponse>(resp_val) {
                Ok(resp) => format_result(&resp),
                Err(_) => format_result(&UserInputResponse::declined()),
            };
        }
        // Batch path.
        let payload = serde_json::json!({ "questions": reqs });
        let resp_val = ctx.request(REQUEST_USER_INPUT_KIND, payload).await;
        if resp_val.is_null() {
            return null_result();
        }
        let resps: Vec<UserInputResponse> = resp_val
            .get("responses")
            .and_then(|r| serde_json::from_value::<Vec<UserInputResponse>>(r.clone()).ok())
            .unwrap_or_default();
        format_batch_result(&reqs, &resps)
    }
```

- [ ] **步骤 5：扩展 schema 与描述以支持 `questions`**

替换 `parameters_schema`（132-154 行），使其增加一个可选的 `questions` 数组，并去掉顶层 `required`（批量调用没有顶层 `header`）；更新 `description` 以提及批量能力。新的 `description`：

```rust
    fn description(&self) -> &str {
        "Ask the user structured question(s) and wait for their answer before continuing. \
         Use ONLY for decisions that are genuinely the user's to make — a preference, a \
         confirmation, a choice between approaches — NOT for anything you can decide, look \
         up, or verify yourself. For ONE question, set `header`, `question`, `mode` \
         (\"single\"=pick one, \"multiple\"=pick any, \"text\"=free-form) and `options` \
         (non-empty for single/multiple). To ask up to 4 related questions answered in ONE \
         interaction, pass a `questions` array of those same objects instead. Keep each \
         `header` short (a few words)."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        let question = serde_json::json!({
            "type": "object",
            "required": ["header", "question", "mode"],
            "properties": {
                "header": {"type": "string", "description": "Very short label (a few words)."},
                "question": {"type": "string", "description": "One clear sentence, ideally ending in '?'."},
                "mode": {"type": "string", "enum": ["single", "multiple", "text"]},
                "options": {
                    "type": "array",
                    "description": "Choices for single/multiple; omit for text.",
                    "items": {
                        "type": "object",
                        "required": ["label"],
                        "properties": {
                            "label": {"type": "string"},
                            "description": {"type": "string"}
                        }
                    }
                }
            }
        });
        serde_json::json!({
            "type": "object",
            "properties": {
                "header": question["properties"]["header"],
                "question": question["properties"]["question"],
                "mode": question["properties"]["mode"],
                "options": question["properties"]["options"],
                "questions": {
                    "type": "array",
                    "description": "Up to 4 questions answered in one interaction. Provide EITHER top-level header/question/mode/options for a single question, OR this array.",
                    "maxItems": 4,
                    "items": question
                }
            }
        })
    }
```

- [ ] **步骤 6：运行测试以确认它们通过**

运行：`cargo test -p rustcode-capabilities --lib request_user_input`
预期：PASS —— 新的批量测试 + 所有既有的单问题测试（字符串未变）。

- [ ] **步骤 7：提交**

```bash
git add crates/rustcode-capabilities/src/tools/request_user_input.rs
git commit -m "feat(request_user_input): batch questions in the tool layer

Accept an optional questions[] array (max 4) alongside the legacy single-question
shape; send {questions:[...]} and read back {responses:[...]}; format one result
line per question keyed by header. Single-question wire + result unchanged.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### 任务 2：TUI 批量状态（`UserInputBatch`）

**文件：**
- 修改：`crates/rustcode-tuix/src/state.rs`（在 `UserInputPanel` 之后添加 `UserInputBatch`，约 310 行）
- 测试：`crates/rustcode-tuix/src/state.rs`（若已有 `#[cfg(test)] mod` 就用它，否则新增一个）

**接口：**
- 消费：既有的 `UserInputPanel`（每道题的状态，逐字复用）以及 `UserInputRequest`/`UserInputResponse`。
- 产出：`pub struct UserInputBatch { pub request_id: u64, pub questions: Vec<UserInputPanel>, pub current: usize }`，带有 `new`、`is_multi`、`submit_stop`、`on_submit_stop`、`next_question`、`prev_question`、`is_answered`、`build_batch_response`。任务 3 读取 `questions`/`current`/`is_answered`；任务 4 调用导航方法与 `build_batch_response`。

- [ ] **步骤 1：编写失败测试**

在 `state.rs` 靠底部的位置添加（按该文件的惯例调整 `mod tests`/导入）：

```rust
#[cfg(test)]
mod user_input_batch_tests {
    use super::*;
    use rustcode_capabilities::tools::request_user_input::{
        UserInputMode, UserInputOption, UserInputRequest,
    };

    fn text_q(h: &str) -> UserInputRequest {
        UserInputRequest { header: h.into(), question: "?".into(), mode: UserInputMode::Text, options: vec![] }
    }
    fn single_q(h: &str) -> UserInputRequest {
        UserInputRequest { header: h.into(), question: "?".into(), mode: UserInputMode::Single,
            options: vec![UserInputOption { label: "x".into(), description: None }] }
    }

    #[test]
    fn tab_wraps_through_submit_stop() {
        let mut b = UserInputBatch::new(7, &[text_q("a"), text_q("b")]);
        assert_eq!(b.current, 0);
        assert_eq!(b.submit_stop(), 2);
        b.next_question(); assert_eq!(b.current, 1);
        b.next_question(); assert_eq!(b.current, 2); // submit stop
        assert!(b.on_submit_stop());
        b.next_question(); assert_eq!(b.current, 0); // wrap
        b.prev_question(); assert_eq!(b.current, 2); // wrap back to submit stop
    }

    #[test]
    fn build_batch_response_declines_untouched_questions() {
        let mut b = UserInputBatch::new(1, &[single_q("a"), text_q("b")]);
        // Answer q0 by moving its cursor onto the concrete option (cursor 0 already is it).
        b.questions[0].select_current_option();
        let resps = b.build_batch_response();
        assert_eq!(resps.len(), 2);
        assert!(!resps[0].declined, "answered question 0");
        assert_eq!(resps[0].selected, vec!["x".to_string()]);
        assert!(resps[1].declined, "untouched text question 1 → declined");
    }

    #[test]
    fn is_answered_tracks_content() {
        let mut b = UserInputBatch::new(1, &[text_q("a")]);
        assert!(!b.is_answered(0), "empty text → not answered");
        b.questions[0].text.push_str("hi");
        assert!(b.is_answered(0));
    }

    #[test]
    fn single_question_batch_is_not_multi() {
        let b = UserInputBatch::new(1, &[text_q("only")]);
        assert!(!b.is_multi());
        assert_eq!(b.submit_stop(), 1);
    }
}
```

- [ ] **步骤 2：运行测试以确认它们失败**

运行：`cargo test -p rustcode-tuix --lib user_input_batch`
预期：FAIL，无法编译（`UserInputBatch` 未定义）。

- [ ] **步骤 3：实现 `UserInputBatch`**

在 `state.rs` 中 `impl UserInputPanel { ... }` 之后（310 行之后）添加：

```rust
/// A batch of 1..=4 questions answered in one interaction. Wraps per-question
/// `UserInputPanel`s; `request_id` lives here (the panels' own `request_id` is unused
/// in a batch). `current` ranges `0..questions.len()` (question panels) plus
/// `questions.len()` (the Submit stop that `Tab` cycles to).
pub struct UserInputBatch {
    pub request_id: u64,
    pub questions: Vec<UserInputPanel>,
    pub current: usize,
}

impl UserInputBatch {
    pub fn new(
        request_id: u64,
        reqs: &[rustcode_capabilities::tools::request_user_input::UserInputRequest],
    ) -> Self {
        let questions = reqs.iter().map(|r| UserInputPanel::new(request_id, r)).collect();
        Self { request_id, questions, current: 0 }
    }

    /// More than one question → render the navigator + Tab/Submit chrome.
    pub fn is_multi(&self) -> bool {
        self.questions.len() > 1
    }

    /// The Submit stop index (one past the last question).
    pub fn submit_stop(&self) -> usize {
        self.questions.len()
    }

    pub fn on_submit_stop(&self) -> bool {
        self.current == self.submit_stop()
    }

    /// `Tab`: next question, wrapping through the Submit stop back to the first.
    pub fn next_question(&mut self) {
        self.current = if self.current >= self.submit_stop() { 0 } else { self.current + 1 };
    }

    /// `Shift+Tab`: previous question, wrapping to the Submit stop.
    pub fn prev_question(&mut self) {
        self.current = if self.current == 0 { self.submit_stop() } else { self.current - 1 };
    }

    /// Whether question `i` has real content (used for the ✓/○ navigator marker).
    pub fn is_answered(&self, i: usize) -> bool {
        self.questions.get(i).is_some_and(Self::panel_answered)
    }

    /// One response per question, in order. A question with no real content becomes
    /// `declined` (partial-submit semantics).
    pub fn build_batch_response(
        &self,
    ) -> Vec<rustcode_capabilities::tools::request_user_input::UserInputResponse> {
        use rustcode_capabilities::tools::request_user_input::UserInputResponse;
        self.questions
            .iter()
            .map(|p| {
                if Self::panel_answered(p) {
                    p.build_response().unwrap_or_else(UserInputResponse::declined)
                } else {
                    UserInputResponse::declined()
                }
            })
            .collect()
    }

    /// A panel counts as answered when it builds a response with a non-empty selection
    /// or non-blank text. (Text mode's `build_response` is always `Some`, possibly empty.)
    fn panel_answered(p: &UserInputPanel) -> bool {
        match p.build_response() {
            Some(r) => {
                !r.selected.is_empty()
                    || r.text.as_deref().map(|t| !t.trim().is_empty()).unwrap_or(false)
            }
            None => false,
        }
    }
}
```

- [ ] **步骤 4：运行测试以确认它们通过**

运行：`cargo test -p rustcode-tuix --lib user_input_batch`
预期：PASS。

- [ ] **步骤 5：提交**

```bash
git add crates/rustcode-tuix/src/state.rs
git commit -m "feat(tuix): UserInputBatch — per-question state + Tab/submit navigation

Wraps UserInputPanel as per-question state with a current index that cycles
through the questions and a Submit stop. build_batch_response yields one response
per question, declining untouched ones (partial submit).

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### 任务 3：TUI 渲染 —— 批量导航器 + 复用每道题的行

**文件：**
- 修改：`crates/rustcode-tuix/src/render/mod.rs`（view struct 约 571-591）、`crates/rustcode-tuix/src/render/retained.rs`（`user_input_panel_row_count` 约 2519、`build_user_input_rows` 约 2560）
- 修改：从 state 生成 panel view 的地方（grep `UserInputPanelView` 的构造处）

**接口：**
- 消费：来自 `state` 的 `UserInputBatch`（任务 2）。
- 产出：批量渲染 —— 当 `is_multi()` 时，前置一行导航器 `Question {current+1}/{N}` 并带每道题的 `✓`/`○` 标记，接着是当前问题的既有行，当 `on_submit_stop()` 时一个 Submit 行，以及一条 Tab 提示。当 N==1 时，输出与今天逐字节相同。

- [ ] **步骤 1：阅读当前的单问题渲染器**

运行：`sed -n '2519,2620p' crates/rustcode-tuix/src/render/retained.rs`，并阅读 `render/mod.rs:571-591` 处的 `UserInputPanelView`。留意 `build_user_input_rows` 如何输出 header/question/option/Other/Submit/hint 这些行，以及调用方如何从 `state.user_input_panel` 构造 view。

- [ ] **步骤 2：添加批量 view + 导航器（N==1 时行为不变）**

在 `render/mod.rs` 中添加一个 `UserInputBatchView`，承载 `current: usize`、`total: usize`、`answered: Vec<bool>` 以及当前问题的 `UserInputPanelView`。在 `retained.rs` 中添加 `build_user_input_batch_rows(&self, view: &UserInputBatchView) -> Vec<Vec<Cell>>`，它：
  - 当 `total > 1`：先推入一行导航器 `Question {current+1}/{total}`，后跟每道题的 `✓`（已答）/`○`（未答）标记（复用既有的字形降级路径，使非 unicode 终端得到 `x`/`o`），然后委托既有的单题行构造器处理当前的 `UserInputPanelView`，再（当光标位于 Submit 停靠位时）输出一个 `提交 / Submit` 行，最后输出一条包含 `Tab 切换问题` 的提示行；
  - 当 `total == 1`：直接调用既有的 `build_user_input_rows`，不做改动（输出逐字节一致）。
  如有必要，把 `build_user_input_rows` 当前的单题主体抽成共享辅助函数，使两条路径共用（DRY）—— **不要**复制 option 行的逻辑。

- [ ] **步骤 3：从 `UserInputBatch` 接线 view 的构造**

在从 `state.user_input_panel` 构造 `UserInputPanelView` 的位置，并行地添加从 `state.user_input_batch`（在任务 4 中添加）的构造：当 `current < questions.len()` 时，从 `batch.questions[batch.current]` 构造当前问题的 `UserInputPanelView`，设置 `answered[i] = batch.is_answered(i)`，并标记 `on_submit_stop`。

- [ ] **步骤 4：行数测试 + 手工渲染核对**

添加一个单元测试，断言 1 题批量的 `build_user_input_batch_rows` 产生的行与该题 `build_user_input_rows` 的行相同（N==1 等价性），并且 2 题批量会包含一行含 `Question 1/2` 的内容。运行：`cargo test -p rustcode-tuix --lib user_input`。预期：PASS。

- [ ] **步骤 5：提交**

```bash
git add crates/rustcode-tuix/src/render/
git commit -m "feat(tuix): render multi-question batch navigator (N==1 unchanged)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### 任务 4：TUI 事件 —— Tab 导航、请求解析、批量投递

**文件：**
- 修改：`crates/rustcode-tuix/src/event_loop/mod.rs` —— `handle_user_input_key`（约 10932）、请求解析（约 11920）、`deliver_user_input`（约 10106）
- 修改：`crates/rustcode-tuix/src/state.rs` —— 在 app state 结构体中（`user_input_panel` 旁边）添加 `pub user_input_batch: Option<UserInputBatch>`

**接口：**
- 消费：`UserInputBatch`（任务 2）。
- 产出：批量按键处理 + 一个 `deliver_user_input_batch`，它通过 `deliver_user_input` 所用的同一条 `DriverCommand::Respond`/`native_live::respond` 路径响应 `serde_json::json!({ "responses": batch.build_batch_response() })`。

- [ ] **步骤 1：阅读当前的处理器**

阅读 `handle_user_input_key`（约 10932-11078）、`deliver_user_input`（约 10106-10127）以及请求解析 arm（约 11920-11930）。

- [ ] **步骤 2：解析批量请求**

在 `REQUEST_USER_INPUT_KIND` arm 中，在单问题的 `from_value::<UserInputRequest>` 之前先尝试批量：如果 `request.payload.get("questions")` 是非空数组，则解析 `Vec<UserInputRequest>`，设置 `state.user_input_batch = Some(UserInputBatch::new(request.id, &reqs))`、`state.phase = UiPhase::UserInput`，重绘，然后返回。否则回落到今天的单问题 `UserInputPanel` 路径，不做改动。

- [ ] **步骤 3：处理批量按键**

在 `handle_user_input_key` 中，当 `state.user_input_batch.is_some()` 时分流到批量处理：
  - `Esc` / `Ctrl+C` → 以全部 declined 调用 `deliver_user_input_batch`（构造一个长度与问题数相同的 `UserInputResponse::declined()` 的 `Vec`）并清理；与单次 Esc 的语义一致。
  - `Tab` → `batch.next_question()`；`BackTab`/`Shift+Tab` → `batch.prev_question()`。
  - 在 Submit 停靠位（`batch.on_submit_stop()`）：`Enter` → `deliver_user_input_batch(batch.build_batch_response())` 并清理。
  - 在某个问题上（`current < questions.len()`）：用与单面板**相同**的逻辑把 `↑↓`、`Space`、数字键、字符键、`Backspace` 路由到 `batch.questions[current]`；单模式问题上的 `Enter` → 前进（`next_question()`）；多模式选项上的 `Enter` → 切换（不变）；文本模式的 Enter → 前进。
  - 保持 `state.user_input_panel.is_some()` 的既有单面板路径不变（N==1 的旧路径仍走 `user_input_panel`，因此其行为字面意义上未变）。

- [ ] **步骤 4：批量投递**

新增 `deliver_user_input_batch(ctx, request_id, resps: Vec<UserInputResponse>)`，照 `deliver_user_input` 的形式，响应 `serde_json::json!({ "responses": resps })`。

- [ ] **步骤 5：构建 + 在可行的地方加状态机测试**

运行：`cargo build -p rustcode-tuix` 与 `cargo test -p rustcode-tuix`。预期：能编译，测试套件全绿。如果该处理器暴露了纯辅助函数（参照 `user_input_response_for`），就为纯“按键→导航”的转换加一个测试；否则依赖任务 2 的状态测试 + 构建。

- [ ] **步骤 6：提交**

```bash
git add crates/rustcode-tuix/src/event_loop/ crates/rustcode-tuix/src/state.rs
git commit -m "feat(tuix): Tab-navigated batch input — parse, keys, batched deliver

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### 任务 5：Daemon —— 批量响应体 + `questions` 投影

**文件：**
- 修改：`crates/rustcode-daemon/src/live_api.rs` —— `UserInputAnswerReq`（约 1752）、`live_user_input`（约 1767）、`LiveWireEvent::UserInputRequest` 投影（约 954）

**接口：**
- 消费：与任务 1 相同的线路结构（请求 `{questions:[...]}`，响应 `{responses:[...]}`）。
- 产出：daemon 接受 `{ request_id, responses: [...] }`，并把 `questions` 数组转发给 webui。

- [ ] **步骤 1：接受批量响应**

给 `UserInputAnswerReq` 添加 `#[serde(default)] pub responses: Option<serde_json::Value>`。在 `live_user_input` 中，当 `responses` 为 `Some` 时响应 `serde_json::json!({ "responses": responses })`；否则保持当前的扁平结构 `{ declined, selected, text }`。

- [ ] **步骤 2：把 `questions` 转发给 webui**

在 `REQUEST_USER_INPUT_KIND` 投影中，当 `payload.get("questions")` 存在时，把它带到 `LiveWireEvent::UserInputRequest` 上（给该事件加一个 `questions: Option<Value>` 字段，单题时为 `None`）。webui 用它来判断是否为批量。

- [ ] **步骤 3：构建 + 测试**

运行：`cargo build -p rustcode-daemon` 与 `cargo test -p rustcode-daemon`。预期：能编译，测试全绿。若该端点有可测试的辅助函数，就加一个小测试，验证带 `responses` 的 `UserInputAnswerReq` 能反序列化并产出 `{responses:...}` 这个值；否则只做构建验证。

- [ ] **步骤 4：提交**

```bash
git add crates/rustcode-daemon/src/live_api.rs
git commit -m "feat(daemon): batched user-input response + questions projection

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### 任务 6：WebUI —— 顺序分步器 + 批量 POST

**文件：**
- 修改：`webui/src/api.ts` —— `UserInputRequestEvent`（增加 `questions?`）、`postLiveUserInput`（接受 `responses`）
- 修改：`webui/src/components/UserInputCard.tsx` —— 逐题推进、累积答案、最后一次性提交

**接口：**
- 消费：SSE 的 `user_input_request` 事件（现在携带 `questions?`）。
- 产出：批量时一次 POST `{ request_id, responses: [...] }`；单题时仍是扁平的请求体。

- [ ] **步骤 1：类型**

在 `api.ts` 中：给 `UserInputRequestEvent` 添加 `questions?: { header: string; question: string; mode: 'single'|'multiple'|'text'; options: {label:string;description?:string}[] }[]`；把 `postLiveUserInput` 的请求体放宽为 `{ request_id: number } & ({ declined: boolean; selected: string[]; text: string|null } | { responses: {declined:boolean;selected:string[];text:string|null}[] })`。

- [ ] **步骤 2：`UserInputCard` 中的分步器**

当 `event.questions` 存在且长度 > 1：维护局部状态 `stepIndex` 与 `answers: Response[]`。渲染当前问题（复用既有的单问题渲染，只是改为由 `questions[stepIndex]` 而不是顶层字段驱动）。“Next”（或在最后一步为提交）把当前答案压入 `answers` 并前进；在最后一个问题上，POST `{ request_id, responses: answers }`。显示 `Question {stepIndex+1}/{n}` 和一个 Back 控件。Skip → 压入一个 declined 响应并前进（部分提交）；首个可操作处的 Skip-all 提交全部 declined。单题（没有 `questions`）保持今天的扁平 POST。

- [ ] **步骤 3：对 webui 做类型检查**

运行：`cd webui && npx tsc --noEmit`（dist 已被 gitignore；只有 `src` 会提交，且 tsc 必须通过）。
预期：没有类型错误。

- [ ] **步骤 4：提交**

```bash
git add webui/src/
git commit -m "feat(webui): sequential multi-question stepper -> one batched answer

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## 自审

**规范覆盖：**
- 单元 1 工具层（schema/parse/format/response）→ 任务 1。[x]
- 单元 2 TUI 批量状态 → 任务 2。[x]
- 单元 3 TUI 渲染导航器（N>1）/ N==1 不变 → 任务 3。[x]
- 单元 4 TUI 事件（Tab/Shift+Tab、解析、批量投递、Esc 拒绝全部）→ 任务 4。[x]
- 单元 5 daemon（批量响应 + questions 投影）→ 任务 5。[x]
- 单元 6 webui 顺序分步器 + 一次批量 POST → 任务 6。[x]
- 向后兼容 / N==1 无回归 → 任务 1 的旧路径、任务 3 步骤 2（N==1 逐字节一致）、任务 4（单题路径经由 `user_input_panel`，未改动）。[x]
- 部分提交（未作答 → declined）→ 任务 2 的 `build_batch_response` + 任务 6 的 skip。[x]
- 最多 4 条截断 → 任务 1 的 `parse_batch`。[x]

**占位符扫描：** 任务 1-2 给出了完整代码。任务 3-6 是对大型既有函数（300 行的渲染器、事件处理器、一个 React 组件）的集成；每一步都给出了确切的锚点、具体的新代码/结构，以及一条具体的验证命令。没有 “TBD”/“handle edge cases” —— 每一项都点名了要添加的具体行/键/字段。先读后改的步骤（3.1、4.1）是有意为之：执行者先读当前这个大函数再动手改，而不是让计划复述数百行未改动的代码。

**类型一致性：** 各任务之间的线路结构一致 —— 请求 `{questions:[UserInputRequest]}`，响应 `{responses:[UserInputResponse]}`。`UserInputBatch { request_id, questions, current }` 在任务 2 中定义，任务 3/4 以这些字段名消费它。`format_batch_result(reqs, resps)` / `parse_batch → (Vec, bool)` 的用法一致。

---

## 执行说明

- 任务 1-2 是纯 Rust 单元，采用完整 TDD。任务 3-6 是对既有大文件的集成；每个任务都从阅读当前函数开始。
- 为避免破坏各 driver，落地顺序很重要：在用真实批量跑通之前，先把任务 1-5（工具层 + 两端 driver）一起落地。由于只有新 schema 部署后模型才会发出 `questions`，且所有 driver 端一起发布，因此不存在中间的破损状态。
- 合并后，批量 UX 属于 **未真机** 状态 —— 验证方式是向 deepseek/GLM 提一个会引出多个选项的设计问题，并确认出现一个可用 Tab 导航的面板（TUI）/ 顺序分步器（webui），且只产生一次合并后的答案。
- `webui/dist` 已被 gitignore —— 只提交 `webui/src`，并确保 `tsc --noEmit` 通过。
