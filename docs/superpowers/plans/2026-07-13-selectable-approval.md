# 可选中审批面板实施计划

> **致 agentic worker：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实施本计划。步骤使用复选框（`- [ ]`）语法进行跟踪。

**目标：** 用固定在页脚、可用方向键选择的纵向选项列表（Allow once / Always allow / Deny）替换键入 `y/a/n` 的工具审批方式。

**架构：** 一个新的页脚区面板（类似待办面板）在 `UiPhase::Approval` 期间显示，由 `UiState.approval_panel` 驱动。`↑/↓` 移动选中项，`Enter` 确认，`Esc` 拒绝，`y/a/n` 作为快捷键。决策仍映射到既有的 `AgentCommand` → `deliver_approval` 路径（tuix 以下没有任何改动）。把审批移出正文区之后，就可以删掉脆弱的 `pop_approval_prompt` 正文擦除逻辑。

**技术栈：** Rust、`rustcode-tuix`（state/render/input）、`rustcode-core` i18n（选项标签）。

## 全局约束

- 纯键盘操作（v1，不支持鼠标）。`↑/↓` 选择（循环回绕），`Enter` 确认，`Esc` = Deny，`y/a/n` 为快捷键。默认选中项 = 索引 0（Allow once）。
- 选项恰好是这三项：Allow once / Always allow / Deny —— 分别映射到 `AgentCommand::ApproveTool` / `ApproveToolAlways` / `DenyTool`。没有“拒绝并附带反馈”这一项。
- “Always allow” 标签使用工具名（`Always allow bash`）—— `AgentEvent::ApprovalNeeded` 事件不携带作用域，因此**不**展示精确的作用域模式。
- 只使用前景色 + `reverse`（cell 模型没有背景色）。选中行 = `▸ ` 前缀 + 反显（与 `build_menu_row` 的做法一致）。不硬编码颜色 —— 使用 `style_for(Role)`。
- 所有字形（`▌` 左侧竖条、`▸`、`⚠`）都需要 ASCII 回退，并以 `self.caps.unicode_symbols` 作为开关。
- 绝不硬编码自然语言字符串 —— 选项标签 + 标题 + 提示语一律经由 `rustcode-core` 的 i18n `Msg`。
- 面板属于页脚区；决策后只是停止渲染（不擦除正文）。保留常驻的 `▸ Tool(detail)` 正文行。
- 提交纪律：只用 `git add <exact path>`；绝不用 `-A`/`.`/`-u`。忽略无关的 `crates/rustcode-codingplan-crypto/*` 文件 —— 绝不暂存它们。
- 已知问题：约 4 个既有的 tuix “byte budget” retained 测试会失败 —— 与本改动无关；确认数量不再增加。修改 core i18n 之后，先 `touch crates/rustcode-core/src/lib.rs` 再运行 tuix 测试。

## 文件结构

| 文件 | 职责 | 变更 |
|---|---|---|
| `crates/rustcode-tuix/src/state.rs` | UI 状态 | `ApprovalKind`/`ApprovalOption`/`ApprovalPanel` 类型；`UiState.approval_panel`；在 `on_approval_resolved` + turn 结束/重置路径中清理 |
| `crates/rustcode-core/src/i18n/{messages,en,zh_cn}.rs` | i18n | 4 个 `Msg` 变体（allow-once / always-allow / deny / hint） |
| `crates/rustcode-tuix/src/event_loop/mod.rs` | 接线 + 输入 | `build_approval_options`；`ApprovalNeeded` 处理器设置面板；`handle_approval_key` 处理方向键/enter/esc/y-a-n + 去掉 pop 调用 |
| `crates/rustcode-tuix/src/event_loop/commands.rs` | `/bg` 恢复 | 设置面板，而不再发送 `ApprovalPrompt` |
| `crates/rustcode-tuix/src/render/retained.rs` | 页脚渲染 | `build_approval_rows` + `approval_panel_row_count` + `paint_footer` 插槽 + 高度；**移除** `ApprovalPrompt` arm + `pop_approval_prompt` + `approval_block_rows` |
| `crates/rustcode-tuix/src/render/plain.rs` | 管道渲染 | 非交互式审批文本（保留）；清理时移除 `ApprovalPrompt` arm |
| `crates/rustcode-tuix/src/render/mod.rs` + `worker.rs` | 清理 | 移除 `UiLine::ApprovalPrompt` 变体 + `PopApprovalPrompt` cmd + trait 方法 |

---

## 任务 1：状态类型 + i18n 标签 + 选项构造器

**文件：**
- 修改：`crates/rustcode-tuix/src/state.rs`（新增类型 + `approval_panel` 字段 + 在 `on_approval_resolved` 中清理）
- 修改：`crates/rustcode-core/src/i18n/messages.rs` + `en.rs` + `zh_cn.rs`
- 修改：`crates/rustcode-tuix/src/event_loop/mod.rs`（`build_approval_options` + 测试）

**产出的接口：**
- `pub enum ApprovalKind { AllowOnce, AlwaysAllow, Deny }`
- `pub struct ApprovalOption { pub label: String, pub kind: ApprovalKind, pub accel: char }`
- `pub struct ApprovalPanel { pub tool: String, pub detail: String, pub options: Vec<ApprovalOption>, pub selected: usize }`，带有方法 `move_up(&mut self)`、`move_down(&mut self)`、`accel_index(&self, c: char) -> Option<usize>`
- `UiState.approval_panel: Option<ApprovalPanel>`
- `pub(crate) fn build_approval_options(tool: &str) -> Vec<ApprovalOption>`（位于 event_loop 内）
- `Msg::ApprovalAllowOnce`、`Msg::ApprovalAlwaysAllow { tool: &'a str }`、`Msg::ApprovalDeny`、`Msg::ApprovalHint`

- [ ] **步骤 1：添加 i18n 变体。** 在 `crates/rustcode-core/src/i18n/messages.rs` 中，于待办面板变体（`TodoPanelMore { n: usize }`）之后添加：
```rust
    // ── Approval panel ──
    ApprovalAllowOnce,
    ApprovalAlwaysAllow { tool: &'a str },
    ApprovalDeny,
    ApprovalHint,
```
在 `en.rs` 中，于待办面板 arm 之后添加：
```rust
        // ── Approval panel ──
        Msg::ApprovalAllowOnce => "Allow once".into(),
        Msg::ApprovalAlwaysAllow { tool } => format!("Always allow {tool} (this session)").into(),
        Msg::ApprovalDeny => "Deny".into(),
        Msg::ApprovalHint => "↑↓ select · enter confirm · esc deny".into(),
```
在 `zh_cn.rs` 中，于待办面板 arm 之后添加：
```rust
        // ── 审批面板 ──
        Msg::ApprovalAllowOnce => "允许一次".into(),
        Msg::ApprovalAlwaysAllow { tool } => format!("本会话总是允许 {tool}").into(),
        Msg::ApprovalDeny => "拒绝".into(),
        Msg::ApprovalHint => "↑↓ 选 · enter 确认 · esc 拒绝".into(),
```

- [ ] **步骤 2：编写失败测试** —— 追加到 state.rs 的测试模块（`mod tests`，使用 `UiState::new()`）：
```rust
    #[test]
    fn approval_panel_selection_wraps_and_accel_maps() {
        use crate::state::{ApprovalKind, ApprovalOption, ApprovalPanel};
        let mut p = ApprovalPanel {
            tool: "bash".into(),
            detail: "rm -rf build/".into(),
            options: vec![
                ApprovalOption { label: "Allow once".into(), kind: ApprovalKind::AllowOnce, accel: 'y' },
                ApprovalOption { label: "Always allow bash".into(), kind: ApprovalKind::AlwaysAllow, accel: 'a' },
                ApprovalOption { label: "Deny".into(), kind: ApprovalKind::Deny, accel: 'n' },
            ],
            selected: 0,
        };
        p.move_up();
        assert_eq!(p.selected, 2, "up from 0 wraps to last");
        p.move_down();
        assert_eq!(p.selected, 0, "down from last wraps to 0");
        p.move_down();
        assert_eq!(p.selected, 1);
        assert_eq!(p.accel_index('A'), Some(1), "accel is case-insensitive");
        assert_eq!(p.accel_index('n'), Some(2));
        assert_eq!(p.accel_index('z'), None);
    }
```

- [ ] **步骤 3：运行测试以确认它失败**

运行：`cargo test -p rustcode-tuix approval_panel_selection_wraps_and_accel_maps`
预期：FAIL —— 找不到 `ApprovalPanel`。

- [ ] **步骤 4：添加这些类型** —— 在 `state.rs` 中，靠近其他 UI 状态结构体（模块顶层）添加：
```rust
/// A tool-approval decision offered in the footer approval panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalKind {
    AllowOnce,
    AlwaysAllow,
    Deny,
}

/// One selectable row of the approval panel.
#[derive(Debug, Clone)]
pub struct ApprovalOption {
    pub label: String,
    pub kind: ApprovalKind,
    /// Single-key accelerator (lower-case): 'y' / 'a' / 'n'.
    pub accel: char,
}

/// The active tool-approval prompt, shown as a footer panel while
/// `UiPhase::Approval`. `None` when no approval is pending.
#[derive(Debug, Clone)]
pub struct ApprovalPanel {
    pub tool: String,
    pub detail: String,
    pub options: Vec<ApprovalOption>,
    pub selected: usize,
}

impl ApprovalPanel {
    pub fn move_up(&mut self) {
        if self.options.is_empty() {
            return;
        }
        self.selected = if self.selected == 0 {
            self.options.len() - 1
        } else {
            self.selected - 1
        };
    }
    pub fn move_down(&mut self) {
        if self.options.is_empty() {
            return;
        }
        self.selected = (self.selected + 1) % self.options.len();
    }
    /// Index of the option whose accelerator matches `c` (case-insensitive).
    pub fn accel_index(&self, c: char) -> Option<usize> {
        let c = c.to_ascii_lowercase();
        self.options.iter().position(|o| o.accel == c)
    }
}
```

- [ ] **步骤 5：添加字段 + 决策时清理。** 在 `UiState` 结构体中添加（`phase`/`prior_spinner_label` 附近）：
```rust
    /// The active footer approval panel (arrow-key selectable). `None` when no
    /// tool approval is pending. Set in the `ApprovalNeeded` handler, cleared on
    /// resolve / turn-end / session reset.
    pub approval_panel: Option<ApprovalPanel>,
```
在 `UiState::new()` 中添加初始化项 `approval_panel: None,`。
在 `on_approval_resolved`（即设置 `self.phase = UiPhase::Streaming` 的那个函数）中，把下面这行作为函数体的**第一行**添加：
```rust
        self.approval_panel = None;
```

- [ ] **步骤 6：添加 `build_approval_options`** —— 在 `event_loop/mod.rs` 中（`approval_command_to_decision` 附近）添加：
```rust
/// The three approval options for `tool`, in display order (Allow once is the
/// default selection). The "Always allow" label carries the tool name because
/// `AgentEvent::ApprovalNeeded` does not carry the grant scope.
pub(crate) fn build_approval_options(tool: &str) -> Vec<crate::state::ApprovalOption> {
    use crate::state::{ApprovalKind, ApprovalOption};
    vec![
        ApprovalOption {
            label: crate::i18n::t(crate::i18n::Msg::ApprovalAllowOnce).into_owned(),
            kind: ApprovalKind::AllowOnce,
            accel: 'y',
        },
        ApprovalOption {
            label: crate::i18n::t(crate::i18n::Msg::ApprovalAlwaysAllow { tool }).into_owned(),
            kind: ApprovalKind::AlwaysAllow,
            accel: 'a',
        },
        ApprovalOption {
            label: crate::i18n::t(crate::i18n::Msg::ApprovalDeny).into_owned(),
            kind: ApprovalKind::Deny,
            accel: 'n',
        },
    ]
}
```
并把它的测试追加到 `event_loop/mod.rs` 的 `bypass_approval_tests` 模块（或新建一个模块）：
```rust
    #[test]
    fn build_approval_options_shape() {
        use crate::state::ApprovalKind;
        let opts = super::build_approval_options("bash");
        assert_eq!(opts.len(), 3);
        assert_eq!((opts[0].kind, opts[0].accel), (ApprovalKind::AllowOnce, 'y'));
        assert_eq!((opts[1].kind, opts[1].accel), (ApprovalKind::AlwaysAllow, 'a'));
        assert!(opts[1].label.contains("bash"), "always-allow label names the tool: {}", opts[1].label);
        assert_eq!((opts[2].kind, opts[2].accel), (ApprovalKind::Deny, 'n'));
    }
```

- [ ] **步骤 7：构建 + 测试**

运行：`cargo build -p rustcode-core && cargo build -p rustcode-tuix`
预期：干净（en/zh_cn 之间 i18n match 的穷尽性由编译器兜底）。
运行：`cargo test -p rustcode-tuix approval_panel_selection_wraps_and_accel_maps build_approval_options_shape`
预期：PASS。

- [ ] **步骤 8：提交**
```bash
git add crates/rustcode-core/src/i18n/messages.rs crates/rustcode-core/src/i18n/en.rs crates/rustcode-core/src/i18n/zh_cn.rs crates/rustcode-tuix/src/state.rs crates/rustcode-tuix/src/event_loop/mod.rs
git commit -m "feat(tuix): approval-panel state types + options builder + i18n labels"
```

---

## 任务 2：审批面板的页脚渲染

**文件：**
- 修改：`crates/rustcode-tuix/src/render/retained.rs` —— 新增 `approval_panel_row_count` + `build_approval_rows`，并接入 `paint_footer` 与高度计算（照搬现有待办面板区域）
- 测试：`crates/rustcode-tuix/src/render/retained.rs`（vterm 测试）

**接口：**
- 消费：`UiState.approval_panel`（任务 1）。渲染器通过 `self.status` 读取它 —— 见步骤 1：面板必须能传到渲染器。`StatusLine`（位于 `render/mod.rs`）已经承载页脚数据（`todo`、`goal` 等）；给它添加 `pub approval: Option<crate::render::ApprovalPanelView>`，或者把 `state.approval_panel` 通过与待办面板相同的通道传下去。严格照待办面板的接线方式来做：`build_status`（event_loop）把 `state.approval_panel` 复制进 `StatusLine`，`paint_footer` 再从 `self.status.approval` 渲染。
- 产出：`fn build_approval_rows(&self, panel, rule_width) -> Vec<Vec<Cell>>`、`fn approval_panel_row_count(&self, panel) -> usize`。

- [ ] **步骤 1：把面板接到渲染器。** 渲染器从 `self.status: StatusLine` 读取页脚数据。照待办面板来做：
  - 在 `render/mod.rs` 中定义渲染器消费的视图类型（避免在 render 里依赖 `state`）：
    ```rust
    /// Renderer-facing snapshot of the approval panel (mirrors how `TodoProgress`
    /// feeds the todo panel). Header + option rows + selected index.
    #[derive(Debug, Clone)]
    pub struct ApprovalPanelView {
        pub tool: String,
        pub detail: String,
        /// (label, is_selected) per option, in display order.
        pub options: Vec<String>,
        pub selected: usize,
    }
    ```
    并把 `pub approval: Option<ApprovalPanelView>` 加到 `StatusLine` 上。把测试中所有既有的 `StatusLine { … }` 字面量都补上 `approval: None`（在 retained.rs 测试中 grep `todo: None` —— 在每个旁边补上 `approval: None`）。
  - 在 `event_loop/mod.rs` 的 `build_status`（即组装 `StatusLine` 的函数，其中有 `let todo = state.active_todos…`）中添加：
    ```rust
    let approval = state.approval_panel.as_ref().map(|p| crate::render::ApprovalPanelView {
        tool: p.tool.clone(),
        detail: p.detail.clone(),
        options: p.options.iter().map(|o| o.label.clone()).collect(),
        selected: p.selected,
    });
    ```
    并把 `approval,` 加到它返回的 `StatusLine { … }` 中。

- [ ] **步骤 2：编写失败的 vterm 测试** —— 添加到 retained 测试模块（使用 `new_capturing`/`drain_into_vterm`/`status_basic`）：
```rust
    #[test]
    fn approval_panel_renders_selectable_options() {
        let (mut r, buf) = new_capturing(80, 24);
        r.caps.colors = true;
        let mut vterm = crate::test_term::VirtualTerminal::new(80, 24);
        let mut status = status_basic();
        status.approval = Some(crate::render::ApprovalPanelView {
            tool: "Bash".into(),
            detail: "rm -rf build/".into(),
            options: vec!["Allow once".into(), "Always allow Bash".into(), "Deny".into()],
            selected: 0,
        });
        r.render(UiLine::InputPrompt {
            buf: String::new(), cursor_byte: 0, menu: None, status, attachments: Vec::new(),
        });
        r.flush_deferred();
        drain_into_vterm(&buf, &mut vterm);
        let dump = vterm.dump();
        // Header names the tool + shows the detail.
        assert!(vterm.any_row(|row| row.contains("Bash") && row.contains("rm -rf build/")) 
            || (vterm.any_row(|r| r.contains("Bash")) && vterm.any_row(|r| r.contains("rm -rf build/"))),
            "header + detail present\n{dump}");
        // All three options render.
        assert!(vterm.any_row(|r| r.contains("Allow once")), "allow once row\n{dump}");
        assert!(vterm.any_row(|r| r.contains("Always allow Bash")), "always row\n{dump}");
        assert!(vterm.any_row(|r| r.contains("Deny")), "deny row\n{dump}");
        // Selected row (index 0 = Allow once) carries the ▸ marker.
        assert!(vterm.any_row(|r| r.contains("▸") && r.contains("Allow once")), "selected marker on option 0\n{dump}");
        // Panel renders ABOVE the input box.
        let h = vterm.height() as usize;
        let row_of = |n: &str| (0..h).find(|&i| vterm.row_text(i).contains(n));
        assert!(row_of("Allow once") < row_of("❯").or(Some(h)), "panel above input\n{dump}");
    }
```

- [ ] **步骤 3：运行测试以确认它失败**

运行：`cargo test -p rustcode-tuix approval_panel_renders_selectable_options`
预期：FAIL —— 目前还没有审批渲染（并且步骤 1 接线的 `status.approval` 字段 / 视图类型必须能编译；若不能，先完成步骤 1）。

- [ ] **步骤 4：添加渲染辅助函数** —— 在 `retained.rs` 中 `build_todo_rows`/`todo_panel_row_count` 附近添加：
```rust
    /// Rows the approval panel occupies (header + detail + one per option + hint),
    /// for the footer height math.
    fn approval_panel_row_count(&self, panel: &crate::render::ApprovalPanelView) -> usize {
        // header + detail + N options + hint
        2 + panel.options.len() + 1
    }

    /// Build the footer approval panel: a warning `⚠ <tool> …` header, the
    /// command detail, the selectable options (selected row = `▸ ` + reverse),
    /// and a hint line. Left `▌` accent bar per row (ASCII `|`). Colorless
    /// besides the warning header + reverse selection.
    fn build_approval_rows(
        &self,
        panel: &crate::render::ApprovalPanelView,
        rule_width: usize,
    ) -> Vec<Vec<Cell>> {
        let unicode = self.caps.unicode_symbols;
        let bar = if unicode { "\u{258c} " } else { "| " }; // ▌
        let bar_style = self.style_for(Role::Warning);
        let warn = if unicode { "\u{26a0} " } else { "! " }; // ⚠
        let mut out: Vec<Vec<Cell>> = Vec::new();

        // header row: `▌ ⚠ <tool> …`
        {
            let mut row = Vec::new();
            push_str_cells(&mut row, bar, &bar_style);
            push_str_cells(&mut row, warn, &self.style_for(Role::Warning));
            let head = crate::width::truncate_with_ellipsis(
                &scrub_controls(&panel.tool),
                rule_width.saturating_sub(6),
            );
            push_str_cells(&mut row, &head, &self.style_bold(Role::ToolName));
            out.push(row);
        }
        // detail row: `▌   <detail>`
        {
            let mut row = Vec::new();
            push_str_cells(&mut row, bar, &bar_style);
            let budget = rule_width.saturating_sub(4);
            let det = crate::width::truncate_with_ellipsis(&scrub_controls(&panel.detail), budget);
            push_str_cells(&mut row, &format!("  {det}"), &self.style_for(Role::Secondary));
            out.push(row);
        }
        // option rows: `▌  ▸ <label>` (selected: ▸ + reverse) / `▌    <label>`
        for (i, label) in panel.options.iter().enumerate() {
            let mut row = Vec::new();
            push_str_cells(&mut row, bar, &bar_style);
            let selected = i == panel.selected;
            let marker = if selected {
                if unicode { "  \u{25b8} " } else { "  > " } // ▸
            } else {
                "    "
            };
            let style = if selected {
                CellStyle { reverse: true, ..CellStyle::default() }
            } else {
                self.style_for(Role::Secondary)
            };
            let budget = rule_width.saturating_sub(6);
            let lbl = crate::width::truncate_with_ellipsis(&scrub_controls(label), budget);
            push_str_cells(&mut row, marker, &style);
            push_str_cells(&mut row, &lbl, &style);
            out.push(row);
        }
        // hint row: `▌  ↑↓ select · enter confirm · esc deny`
        {
            let mut row = Vec::new();
            push_str_cells(&mut row, bar, &bar_style);
            let hint = crate::i18n::t(crate::i18n::Msg::ApprovalHint).into_owned();
            let budget = rule_width.saturating_sub(4);
            let fitted = crate::width::truncate_with_ellipsis(&hint, budget);
            push_str_cells(&mut row, &format!("  {fitted}"), &self.style_for(Role::Muted));
            out.push(row);
        }
        out
    }
```

- [ ] **步骤 5：接入 `paint_footer`。** 在 `paint_footer` 中，严格照待办面板区域来做（待办面板绘制在页脚顶部，并预留了 `todo_rows`）。新增 `approval_rows` 预留，并把审批面板画在待办面板的正下方、上分隔线的上方：
  - 在计算 `todo_rows` 的位置添加：
    ```rust
    let approval_rows = self
        .status
        .approval
        .as_ref()
        .map(|p| self.approval_panel_row_count(p))
        .unwrap_or(0);
    ```
  - 把 `approval_rows` 计入 `total_rows`，并计入 `max_input_rows(... status_rows + goal_rows + todo_rows + approval_rows)` 预留（补上 `+ approval_rows`）。
  - 在绘制部分：待办面板绘制在 `todo_top`（`= footer_top`）之后，审批面板从 `footer_top + todo_rows` 开始绘制，同时把 `rules_top` 也向下移 `approval_rows`：
    ```rust
    let approval_top = footer_top + todo_rows;
    if let Some(p) = self.status.approval.clone() {
        for (i, ar) in self.build_approval_rows(&p, rule_width).into_iter().enumerate() {
            let mut padded = ar;
            Self::pad_row_to_width(&mut padded, w);
            self.screen.draw_row(approval_top + i, 0, &padded);
        }
    }
    let rules_top = footer_top + todo_rows + approval_rows;
    ```
    （把既有的 `let rules_top = footer_top + todo_rows;` 替换为带 `+ approval_rows` 的版本，并用 `approval_top` 来绘制面板。光标计算本来就基于 `rules_top`。）

- [ ] **步骤 6：运行测试 + 构建**

运行：`cargo test -p rustcode-tuix approval_panel_renders_selectable_options`
预期：PASS。
运行：`cargo test -p rustcode-tuix --lib`
预期：PASS，除约 4 个既有的 byte-budget 红测（数量不变）。任何编译不过的 `StatusLine { … }` 字面量，通过补上 `approval: None` 修复。

- [ ] **步骤 7：提交**
```bash
git add crates/rustcode-tuix/src/render/mod.rs crates/rustcode-tuix/src/render/retained.rs crates/rustcode-tuix/src/event_loop/mod.rs
git commit -m "feat(tuix): render the footer approval panel (selectable options)"
```

---

## 任务 3：接线请求与输入，去掉 pop 调用

**文件：**
- 修改：`crates/rustcode-tuix/src/event_loop/mod.rs`（`ApprovalNeeded` 处理器 + `handle_approval_key`）
- 修改：`crates/rustcode-tuix/src/event_loop/commands.rs`（`/bg` 恢复路径 ~2228）
- 修改：`crates/rustcode-tuix/src/state.rs`（在 turn 结束 + 重置路径中清理 `approval_panel`）

**接口：**
- 消费：`build_approval_options`（任务 1）、`ApprovalPanel`（任务 1）、`deliver_approval`/`AgentCommand`（既有）。

- [ ] **步骤 1：在 `ApprovalNeeded` 处理器中设置面板状态。** 在 `event_loop/mod.rs` 中，把 `renderer.render(UiLine::ApprovalPrompt { tool: display.clone(), detail: detail.clone() });` 这一行（`AgentEvent::ApprovalNeeded` arm 中，紧挨在 `renderer.flush();` 之前）替换为：
```rust
            state.approval_panel = Some(crate::state::ApprovalPanel {
                tool: display.clone(),
                detail: detail.clone(),
                options: build_approval_options(&display),
                selected: 0,
            });
```
（保留其上方的 `▸ Tool(detail)` 正文行输出、`renderer.flush()`、`notify`、`state.on_approval_needed(&display)` 以及 `redraw_idle_plain(...)` —— 重绘现在会绘制页脚审批面板。）

- [ ] **步骤 2：在 `/bg` 恢复时设置面板状态。** 在 `commands.rs` 中，把 `renderer.render(UiLine::ApprovalPrompt { tool: tool_name, detail });` 代码块（`find_pending_approval` 分支 ~2228）替换为：
```rust
                    if let Some((tool_name, detail)) = pending_approval {
                        state.approval_panel = Some(crate::state::ApprovalPanel {
                            options: crate::event_loop::build_approval_options(&tool_name),
                            selected: 0,
                            tool: tool_name,
                            detail,
                        });
                        state.on_approval_needed("");
                    }
```
（若 `build_approval_options` 从 `commands.rs` 访问不到 `pub(crate)` 可见性，就把它改成 `pub(crate)` —— 按任务 1，它本来就是。）

- [ ] **步骤 3：编写失败的输入测试** —— 添加到 `event_loop/mod.rs` 的 `bypass_approval_tests`（或新建 `approval_key_tests`）模块。由于 `handle_approval_key` 需要完整的 `App`/`LoopCtx`，改为通过一个小辅助函数来测试**纯**“选中项→命令”的映射。把这个辅助函数加到 `build_approval_options` 旁边：
```rust
/// The `AgentCommand` for a chosen approval option kind. Pure seam so the
/// key handler's decision mapping is unit-testable.
pub(crate) fn approval_kind_to_command(kind: crate::state::ApprovalKind) -> AgentCommand {
    use crate::state::ApprovalKind;
    match kind {
        ApprovalKind::AllowOnce => AgentCommand::ApproveTool,
        ApprovalKind::AlwaysAllow => AgentCommand::ApproveToolAlways,
        ApprovalKind::Deny => AgentCommand::DenyTool,
    }
}
```
测试：
```rust
    #[test]
    fn approval_kind_command_mapping() {
        use crate::state::ApprovalKind;
        assert!(matches!(super::approval_kind_to_command(ApprovalKind::AllowOnce), AgentCommand::ApproveTool));
        assert!(matches!(super::approval_kind_to_command(ApprovalKind::AlwaysAllow), AgentCommand::ApproveToolAlways));
        assert!(matches!(super::approval_kind_to_command(ApprovalKind::Deny), AgentCommand::DenyTool));
    }
```

- [ ] **步骤 4：运行测试以确认它失败**

运行：`cargo test -p rustcode-tuix approval_kind_command_mapping`
预期：FAIL —— 找不到 `approval_kind_to_command`。

- [ ] **步骤 5：改造 `handle_approval_key`。** 把 `handle_approval_key` 中 Ctrl+C 块**之后**的函数体（从 `// Any other key resets…` 一直到最后的 `Ok(())`）替换为：
```rust
    // Any other key resets the exit confirmation
    app.exit_pending = None;

    // Navigation: move the selection and repaint the footer, no decision yet.
    match code {
        KeyCode::Up => {
            if let Some(p) = app.state.approval_panel.as_mut() {
                p.move_up();
            }
            redraw_idle_plain(&app.buf, &mut app.state, ctx, renderer);
            return Ok(());
        }
        KeyCode::Down => {
            if let Some(p) = app.state.approval_panel.as_mut() {
                p.move_down();
            }
            redraw_idle_plain(&app.buf, &mut app.state, ctx, renderer);
            return Ok(());
        }
        _ => {}
    }

    // Resolve to a decision: Enter = the selected option; y/a/n = accelerators;
    // Esc = Deny (safe default). Any other key is ignored.
    let kind = match code {
        KeyCode::Enter => app
            .state
            .approval_panel
            .as_ref()
            .and_then(|p| p.options.get(p.selected).map(|o| o.kind)),
        KeyCode::Esc => Some(crate::state::ApprovalKind::Deny),
        KeyCode::Char(c) => app
            .state
            .approval_panel
            .as_ref()
            .and_then(|p| p.accel_index(c).and_then(|i| p.options.get(i).map(|o| o.kind))),
        _ => None,
    };
    let Some(kind) = kind else {
        return Ok(());
    };
    let cmd = approval_kind_to_command(kind);
    deliver_approval(ctx, cmd);
    app.state.on_approval_resolved(); // clears approval_panel + phase → Streaming
    Ok(())
```
同时在同一函数的 Ctrl+C 块中**删除** `renderer.pop_approval_prompt();` 这一行（面板现在由 `on_approval_resolved()` 清理，而 Ctrl+C 分支已经调用了它）。在非 Ctrl+C 路径下 `renderer` 参数可能变成未使用 —— 但 Up/Down 重绘和 Ctrl+C 的 `CommandOutput` 仍然要用它，所以保留。

- [ ] **步骤 6：在 turn 结束 + 重置时清理 `approval_panel`。** 在 `state.rs` 中，把 `self.approval_panel = None;` 加到 `on_turn_cancelled` 和 `on_error`（紧挨着既有的 `self.active_todos`… 不对 —— approval_panel 是独立的；加在这些函数清理其他临时状态的地方附近）。**不要**在 `on_turn_complete` 中清理（审批进行中本不应出现已完成的 turn，但在那里清理也无害 —— 为保险起见也加上）。在 `event_loop/commands.rs::reset_to_new_session` 中，把 `state.approval_panel = None;` 加到 `state.active_todos = None;` 旁边。在 `SessionSwitched` 处理器（`event_loop/mod.rs`，之前加 `state.active_todos = None;` 的地方）里，在其旁边加上 `state.approval_panel = None;`。

- [ ] **步骤 7：运行测试 + 构建**

运行：`cargo test -p rustcode-tuix approval_kind_command_mapping`
预期：PASS。
运行：`cargo build -p rustcode-tuix`
预期：干净。（`pop_approval_prompt` 的调用者在 `handle_approval_key` 中已消失，可能会出现 unused 警告 —— 这将在任务 4 中处理。若构建把警告视为错误，记录下来并继续；任务 4 会删掉该函数。）
运行：`cargo test -p rustcode-tuix --lib`
预期：PASS，除约 4 个既有的 byte-budget 红测。

- [ ] **步骤 8：提交**
```bash
git add crates/rustcode-tuix/src/event_loop/mod.rs crates/rustcode-tuix/src/event_loop/commands.rs crates/rustcode-tuix/src/state.rs
git commit -m "feat(tuix): drive approval from the footer panel (arrows/enter/esc/yan)"
```

---

## 任务 4：移除已失效的 ApprovalPrompt / pop 机制

**文件：**
- 修改：`crates/rustcode-tuix/src/render/retained.rs`（移除 `UiLine::ApprovalPrompt` arm、`pop_approval_prompt`、`approval_block_rows` 字段及其 resize 逻辑）
- 修改：`crates/rustcode-tuix/src/render/plain.rs`（移除 `UiLine::ApprovalPrompt` arm）
- 修改：`crates/rustcode-tuix/src/render/mod.rs`（移除 `UiLine::ApprovalPrompt` 变体 + `pop_approval_prompt` trait 方法）
- 修改：`crates/rustcode-tuix/src/render/worker.rs`（移除 `RenderCmd::PopApprovalPrompt` + 其处理器 + `pop_approval_prompt` impl + name-map arm）

**接口：** 无产出（纯删除）。任务 3 之后应当没有任何地方再**发出** `UiLine::ApprovalPrompt`，也没有任何地方**调用** `pop_approval_prompt` —— 删除前先确认。

- [ ] **步骤 1：确认没有残留的活跃发送者 / 调用者**

运行：`grep -rn "UiLine::ApprovalPrompt\|pop_approval_prompt\|PopApprovalPrompt\|approval_block_rows" crates/rustcode-tuix/src`
预期：命中项**仅**是定义/render arm/worker 管道（match arm 之外没有 `renderer.render(UiLine::ApprovalPrompt` 的发送，也没有 `renderer.pop_approval_prompt()` 调用）。若仍有活跃的发送/调用，立即停止 —— 任务 3 尚未完成。

- [ ] **步骤 2：移除变体 + trait 方法**（`render/mod.rs`）：从 `UiLine` enum 中删除 `ApprovalPrompt { tool, detail }` 变体，并从 `Renderer` trait 中删除 `fn pop_approval_prompt(&mut self)`（连同其文档注释）。

- [ ] **步骤 3：移除 retained 实现**（`retained.rs`）：删除 `UiLine::ApprovalPrompt { tool, detail } => { … }` 渲染 arm；删除 `fn pop_approval_prompt`；删除 `approval_block_rows` 字段、它的初始化以及消费它的 resize 逻辑（`reflow_body_to_current_width` 中的 `Some(0)` 消费空操作 + 任何 `approval_block_rows` 的写入/读取）。从 transient 变体 match（约 3661 行）中移除 `UiLine::ApprovalPrompt`，并移除 `impl Renderer` 的 `pop_approval_prompt` 方法。

- [ ] **步骤 4：移除 plain + worker 管道**（`plain.rs` + `worker.rs`）：删除 `plain.rs` 中的 `UiLine::ApprovalPrompt` arm；在 `worker.rs` 中删除 `RenderCmd::PopApprovalPrompt` 及其 send/handler、`pop_approval_prompt` 转发 impl，以及 `UiLine::ApprovalPrompt { .. } => "ApprovalPrompt"` 的 name-map arm。

- [ ] **步骤 5：删除现已失效的测试** —— 任何断言旧的正文 `ApprovalPrompt` 渲染或 `pop_approval_prompt` 行为的测试（步骤 1 的 grep 会暴露它们；例如 approval-pop / blank-gap / resize-count 测试）。删除或重写它们；新行为已由任务 2/3 的测试覆盖。如果 Ctrl+C 拒绝行为的测试不依赖 `pop_approval_prompt`，则保留。

- [ ] **步骤 6：构建 + 测试**

运行：`cargo build -p rustcode-tuix`
预期：干净，被删项不再产生 `unused` 警告。
运行：`grep -rn "ApprovalPrompt\|pop_approval_prompt\|PopApprovalPrompt\|approval_block_rows" crates/rustcode-tuix/src`
预期：只允许残留 `highlight/theme.rs:51` 的注释（顺带提及）—— 可更新也可保留；不应再有代码引用。
运行：`cargo test -p rustcode-tuix --lib`
预期：PASS，除约 4 个既有的 byte-budget 红测。

- [ ] **步骤 7：提交**
```bash
git add crates/rustcode-tuix/src/render/mod.rs crates/rustcode-tuix/src/render/retained.rs crates/rustcode-tuix/src/render/plain.rs crates/rustcode-tuix/src/render/worker.rs
git commit -m "refactor(tuix): remove the dead body-ApprovalPrompt + pop machinery"
```

---

## 任务 5：验证

- [ ] **步骤 1：全工作区构建 + 受影响 crate 的测试**

运行：`touch crates/rustcode-core/src/lib.rs && cargo build`
预期：干净。
运行：`cargo test -p rustcode-core -p rustcode-tuix`
预期：全绿，除约 4 个既有的 tuix byte-budget 红测（与干净检出数量相同）。

- [ ] **步骤 2：手工冒烟（记录在案，仅真实终端）** —— 记录以下需要真实终端的项：
  1. 触发一个需要审批的工具（例如某条 bash 命令）→ 页脚面板出现在输入框上方：`⚠ Bash` / 命令 / `▸ Allow once` / `Always allow bash` / `Deny` / 提示语。
  2. `↑/↓` 移动 `▸` + 反显高亮；在 “Allow once” 上按 `Enter` 会执行该工具；在 “Deny” 上按 `Enter` 会拒绝。
  3. `y` / `a` / `n` 仍然可以直接作为快捷键使用。
  4. `Esc` 拒绝。`Ctrl+C` 拒绝并进入退出预备状态。
  5. 决策后面板消失；`▸ Bash(…)` 行留在滚动区中；输入框可用。
  6. 把一个正在等待审批的会话 `/bg` 出去，再恢复它 → 面板重新出现。
  7. 非 unicode 终端 → 显示 `|` 竖条、`>` 标记、`!` 警告；不出现 `▸`/`▌`/`⚠`。

- [ ] **步骤 3：申请评审** —— 合并前对分支 diff 执行 `/code-review`。

---

## 自审（撰写期间完成）

- **规范覆盖：** 纵向可选中列表 → 任务 2 渲染 + 任务 3 输入。页脚固定 → 任务 2 的 paint_footer 插槽。纯键盘（↑↓/enter/esc/y-a-n，默认 0）→ 任务 3。3 个选项 + 带工具名的 Always 标签 → 任务 1 的 `build_approval_options`。反显选中项 + `▌`/`▸`/`⚠` + ASCII 回退 → 任务 2 的 `build_approval_rows`。i18n 标签 → 任务 1。无背景色 / 不硬编码颜色 → 任务 2（Role + reverse）。决策→AgentCommand 的既有管道 → 任务 3。去掉 pop_approval_prompt → 任务 4。决策/turn 结束/重置时清理 → 任务 1 + 任务 3。plain 回退 → 保留（任务 4 只移除正文里的 ApprovalPrompt arm；面板是 retained 独有的，plain 仍像以前那样显示 `▸ Tool` 行 + 结果 —— 注意：v1 的 plain 没有交互式审批；daemon/管道路径使用同步审批，不受影响）。
- **占位符扫描：** 无 —— 每个**新增**部分都给出了完整代码。paint_footer 插槽（任务 2 步骤 5）引用仓库内既有的待办面板区域作为照搬模式（一个真实可读的模式，不是占位符），并给出了 `approval_rows`/`approval_top`/`rules_top` 的确切改法。
- **类型一致性：** `ApprovalKind`/`ApprovalOption`/`ApprovalPanel`（任务 1）在任务 2（经由 `ApprovalPanelView`）和任务 3（`approval_kind_to_command`、`accel_index`、`move_up/down`）中使用。`ApprovalPanelView`（任务 2，render/mod.rs）只被渲染器消费。`build_approval_options`/`approval_kind_to_command` 的签名在任务 1/3 的定义处与调用点之间一致。
- **计划阶段问题已解决：** V1（事件中没有作用域）→ Always 标签使用工具名。V2（ApprovalPrompt 纠缠）→ 用专门的任务 4 并带前置检查来移除。V3（页脚插槽）→ 任务 2 步骤 5（待办面板下方、上分隔线上方）。
