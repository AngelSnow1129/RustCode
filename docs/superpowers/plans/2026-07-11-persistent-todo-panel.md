# 持久化待办面板实施计划

> **致 agentic worker：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实施本计划。步骤使用复选框（`- [ ]`）语法进行跟踪。

**目标：** 用固定在输入框上方页脚中的多行待办面板替换反复重印的内联 todowrite 块；该面板持久存在并原地更新。

**架构：** 把现有页脚的“待办行”（单行 `TodoProgress`）扩展为高度可变的面板。面板由持久化的内存态 `UiState.active_todos` 缓存驱动（由 `todowrite` 调用实时写入，在恢复/切换会话时通过 `derive_current_todos` 从 transcript 播种，在 `/clear`/`/new` 时重置）。一个纯折叠函数负责限制面板高度；retained-mode 的 cell/diff 渲染器负责原地更新。内联 todowrite 块从实时与回放两条路径中同时移除。

**技术栈：** Rust、`rustcode-tuix`（retained-mode TUI）、`rustcode-capabilities::tools::todo`（待办数据类型，不变）、`rustcode-core` i18n。

## 全局约束

- 绝不在 TUI 中硬编码自然语言字符串 —— 一律使用 `rustcode-core` 的 i18n `Msg`（在 `messages.rs` + `en.rs` + `zh_cn.rs` 中新增变体）。逐字取自规范 §样式/§边界。
- 绝不硬编码颜色 —— 在 `self.style_for(Role)`（它会解析出与主题相关的前景色）之上叠加；只允许追加 `bold`/`faint` 两种 cell 属性。`CellStyle` 仅支持 `fg`/`bold`/`reverse`/`faint`（没有删除线）—— 已完成项使用 `faint`。
- 所有字形都必须具备 ASCII 回退，并以 `self.caps.unicode_symbols` 作为开关（复用 `todo_glyph` / `todo_marker`）。
- 面板绝不溢出屏幕：它被折进输入框的高度预留里（`max_input_rows(..., status_rows + goal_rows + todo_rows)`）；`todo_rows` 即面板行数。
- `active_todos` 纯内存 —— 绝不落盘。恢复会话时的再水化由 transcript 推导得出。
- 特性保持在现有 `RUSTCODE_TODO` 环境变量开关之后（无需改动 —— 该工具仅在开关打开时才注册；面板只由 `todowrite` 调用供数）。
- 修改 `rustcode-core`（i18n）中的任何内容后，运行 `rustcode-tuix` 测试前先 `touch crates/rustcode-core/src/lib.rs`，以避免陈旧构建产物（依据仓库 lore）。

**面板视觉（unicode）：**
```
☑ Todos · 2/5          ← header: ☑ marker (Brand), "Todos", " · N/M" (Muted)
  [✓] 2 completed      ← completed fold (faint), one line
  [•] wire openai_compat   ← in-progress (bold + Brand)
  [ ] update docs      ← pending (Muted)
  [ ] +1 more…         ← overflow (Muted)
```

---

## 文件结构

| 文件 | 职责 | 变更 |
|---|---|---|
| `crates/rustcode-tuix/src/render/mod.rs` | `TodoProgress` 类型 | 新增 `items` 字段 |
| `crates/rustcode-core/src/i18n/messages.rs` + `en.rs` + `zh_cn.rs` | i18n | 3 个新的 `Msg` 变体 |
| `crates/rustcode-tuix/src/render/retained.rs` | 页脚渲染 | 纯折叠函数 + cell 构造器 + 页脚接线 + 高度 |
| `crates/rustcode-tuix/src/state.rs` | UI 状态 | 重命名 `live_turn_todo`→`active_todos`，去掉 turn 结束时的清理 |
| `crates/rustcode-tuix/src/event_loop/mod.rs` | 实时捕获 / 辅助函数 | 仅捕获（无内联块）、全部完成时的隐藏过滤、`todo_progress_from_messages`、删除失效的块函数 |
| `crates/rustcode-tuix/src/modals/session_picker.rs` | 回放 | 移除内联块，播种 `active_todos` |
| `crates/rustcode-tuix/src/event_loop/commands.rs` | `/clear`/`/new` 重置 | 重置 `active_todos` |

---

## 任务 1：为 `TodoProgress` 扩展完整条目列表

**文件：**
- 修改：`crates/rustcode-tuix/src/render/mod.rs:516-525`
- 修改：`crates/rustcode-tuix/src/event_loop/mod.rs:11821-11835`（`todo_progress_from_items`）
- 测试：`crates/rustcode-tuix/src/event_loop/mod.rs`（现有 `todo_block_tests` mod，约在 11848 行附近）

**接口：**
- 产出：`TodoProgress.items: Vec<(rustcode_capabilities::tools::todo::TodoStatus, String)>` —— 完整有序列表，由 `todo_progress_from_items` 填充。

- [ ] **步骤 1：编写失败测试** —— 追加到 `event_loop/mod.rs` 的 `todo_block_tests` 模块：

```rust
    #[test]
    fn todo_progress_carries_full_items_in_order() {
        let p = todo_progress_from_args(
            r#"{"todos":[
                {"content":"a","status":"completed"},
                {"content":"b","status":"in_progress"},
                {"content":"c","status":"pending"}
            ]}"#,
        )
        .unwrap();
        use rustcode_capabilities::tools::todo::TodoStatus;
        assert_eq!(p.items.len(), 3);
        assert_eq!(p.items[0], (TodoStatus::Completed, "a".to_string()));
        assert_eq!(p.items[1], (TodoStatus::InProgress, "b".to_string()));
        assert_eq!(p.items[2], (TodoStatus::Pending, "c".to_string()));
        assert_eq!((p.completed, p.total), (1, 3));
    }
```

- [ ] **步骤 2：运行测试以确认它失败**

运行：`cargo test -p rustcode-tuix todo_progress_carries_full_items_in_order`
预期：FAIL —— `no field 'items' on type TodoProgress`。

- [ ] **步骤 3：添加该字段。** 在 `render/mod.rs` 中，把 `TodoProgress` 结构体（516-525 行）替换为：

```rust
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TodoProgress {
    /// The description of the task currently `in_progress` (todowrite enforces
    /// at most one). `None` when no task is in progress (all pending / all done).
    pub current: Option<String>,
    /// Number of tasks marked `completed`.
    pub completed: usize,
    /// Total number of tasks in the list.
    pub total: usize,
    /// The full ordered list (status + content) — drives the multi-line footer
    /// todo panel. `current`/`completed`/`total` are retained as pre-computed
    /// conveniences for the header + hide-when-all-done filter.
    pub items: Vec<(rustcode_capabilities::tools::todo::TodoStatus, String)>,
}
```

- [ ] **步骤 4：填充该字段。** 在 `event_loop/mod.rs` 中，把 `todo_progress_from_items` 的函数体（11821-11835 行）替换为：

```rust
pub(crate) fn todo_progress_from_items(
    todos: &[rustcode_capabilities::tools::todo::TodoItem],
) -> crate::render::TodoProgress {
    use rustcode_capabilities::tools::todo::{todo_counts, TodoStatus};
    let (completed, total) = todo_counts(todos);
    let current = todos
        .iter()
        .find(|t| t.status == TodoStatus::InProgress)
        .map(|t| t.content.clone());
    let items = todos
        .iter()
        .map(|t| (t.status, t.content.clone()))
        .collect();
    crate::render::TodoProgress {
        current,
        completed,
        total,
        items,
    }
}
```

- [ ] **步骤 5：运行测试以确认它通过**

运行：`cargo test -p rustcode-tuix todo_progress_carries_full_items_in_order`
预期：PASS。同时运行 `cargo build -p rustcode-tuix` —— 构造 `TodoProgress { current, completed, total, .. }` 或 `TodoProgress::default()` 的 `retained.rs` 测试夹具仍需能编译（新字段通过 `..Default` / 字面量默认为空 vec）。若任何字面量 `TodoProgress { current, completed, total }` 编译失败，给它补上 `items: vec![],`。

- [ ] **步骤 6：提交**

```bash
git add crates/rustcode-tuix/src/render/mod.rs crates/rustcode-tuix/src/event_loop/mod.rs
git commit -m "feat(tuix): TodoProgress carries the full ordered item list"
```

---

## 任务 2：面板标签的 i18n `Msg` 变体

**文件：**
- 修改：`crates/rustcode-core/src/i18n/messages.rs`（enum，约 228 行）
- 修改：`crates/rustcode-core/src/i18n/en.rs`（arm，约 310 行）
- 修改：`crates/rustcode-core/src/i18n/zh_cn.rs`（arm，约 300 行）
- 测试：`crates/rustcode-core/src/i18n/mod.rs` 或最近的现有 i18n 测试（新增一小段渲染断言）

**接口：**
- 产出：`Msg::TodoPanelTitle`、`Msg::TodoPanelCompleted { n: usize }`、`Msg::TodoPanelMore { n: usize }` —— 通过 `crate::i18n::t(...)` 渲染，返回 `Cow<'static, str>`。

- [ ] **步骤 1：编写失败测试** —— 添加到 `rustcode-core/src/i18n/mod.rs` 的测试模块（若不存在则新建 `#[cfg(test)] mod tests` 块；若已存在则追加）：

```rust
#[cfg(test)]
mod todo_panel_i18n_tests {
    use super::*;
    #[test]
    fn todo_panel_labels_render() {
        // Non-empty in the default locale; exact copy is locale-dependent.
        assert!(!t(Msg::TodoPanelTitle).is_empty());
        assert!(t(Msg::TodoPanelCompleted { n: 3 }).contains('3'));
        assert!(t(Msg::TodoPanelMore { n: 2 }).contains('2'));
    }
}
```

- [ ] **步骤 2：运行测试以确认它失败**

运行：`cargo test -p rustcode-core todo_panel_labels_render`
预期：FAIL —— `no variant named TodoPanelTitle`。

- [ ] **步骤 3：添加 enum 变体。** 在 `messages.rs` 中，于 `SessionResumedLabel { name: &'a str },`（228 行）之后添加：

```rust
    // ── Todo panel ──
    TodoPanelTitle,
    TodoPanelCompleted { n: usize },
    TodoPanelMore { n: usize },
```

- [ ] **步骤 4：添加英文 arm。** 在 `en.rs` 中，于 `Msg::SessionResumedLabel` arm（309-310 行）之后添加：

```rust
        // ── Todo panel ──
        Msg::TodoPanelTitle => "Todos".into(),
        Msg::TodoPanelCompleted { n } => format!("{n} completed").into(),
        Msg::TodoPanelMore { n } => format!("+{n} more…").into(),
```

- [ ] **步骤 5：添加中文 arm。** 在 `zh_cn.rs` 中，于 `Msg::SessionResumedLabel` arm（299-300 行）之后添加：

```rust
        // ── 待办面板 ──
        Msg::TodoPanelTitle => "待办".into(),
        Msg::TodoPanelCompleted { n } => format!("{n} 已完成").into(),
        Msg::TodoPanelMore { n } => format!("+{n} 更多…").into(),
```

- [ ] **步骤 6：运行测试以确认它通过**

运行：`cargo test -p rustcode-core todo_panel_labels_render`
预期：PASS。同时 `cargo build -p rustcode-core` —— `t()` 的 match 必须对所有 locale 穷尽；缺一个 arm 就是编译错误（这正是预期中的安全网）。

- [ ] **步骤 7：提交**

```bash
git add crates/rustcode-core/src/i18n/messages.rs crates/rustcode-core/src/i18n/en.rs crates/rustcode-core/src/i18n/zh_cn.rs crates/rustcode-core/src/i18n/mod.rs
git commit -m "i18n: add todo panel labels (title, completed fold, more)"
```

---

## 任务 3：纯折叠函数 `todo_panel_rows`

**文件：**
- 修改：`crates/rustcode-tuix/src/render/retained.rs` —— 在其他页脚行辅助函数附近新增 const + enum + fn（`todo_row_parts` 之后，约 252 行）
- 测试：同一文件（其中已有带页脚夹具的 `#[cfg(test)] mod` —— 新增一个嵌套测试模块）

**接口：**
- 产出：
  - `const MAX_TODO_PANEL_ROWS: usize = 6;`
  - `enum TodoPanelRow { Header { completed, total }, CompletedFold { count }, Item { status, content }, More { hidden } }`
  - `fn todo_panel_rows(items: &[(TodoStatus, String)], completed: usize, total: usize, max_rows: usize) -> Vec<TodoPanelRow>` —— 总行数 ≤ `max_rows`；存在 in-progress 时必定显示；显示顺序为 Header、CompletedFold?、InProgress?、Pending…、More?。

- [ ] **步骤 1：编写失败测试** —— 添加到 retained-mode 测试夹具附近：

```rust
#[cfg(test)]
mod todo_panel_rows_tests {
    use super::*;
    use rustcode_capabilities::tools::todo::TodoStatus;

    fn items(spec: &[(TodoStatus, &str)]) -> Vec<(TodoStatus, String)> {
        spec.iter().map(|(s, c)| (*s, c.to_string())).collect()
    }

    #[test]
    fn header_plus_all_when_fits() {
        let it = items(&[
            (TodoStatus::Completed, "a"),
            (TodoStatus::InProgress, "b"),
            (TodoStatus::Pending, "c"),
        ]);
        let rows = todo_panel_rows(&it, 1, 3, MAX_TODO_PANEL_ROWS);
        // Header, CompletedFold, InProgress, Pending
        assert_eq!(rows.len(), 4);
        assert!(matches!(rows[0], TodoPanelRow::Header { completed: 1, total: 3 }));
        assert!(matches!(rows[1], TodoPanelRow::CompletedFold { count: 1 }));
        assert!(matches!(&rows[2], TodoPanelRow::Item { status: TodoStatus::InProgress, content } if content == "b"));
        assert!(matches!(&rows[3], TodoPanelRow::Item { status: TodoStatus::Pending, content } if content == "c"));
    }

    #[test]
    fn no_fold_when_none_completed() {
        let it = items(&[(TodoStatus::InProgress, "b"), (TodoStatus::Pending, "c")]);
        let rows = todo_panel_rows(&it, 0, 2, MAX_TODO_PANEL_ROWS);
        assert!(!rows.iter().any(|r| matches!(r, TodoPanelRow::CompletedFold { .. })));
    }

    #[test]
    fn pending_overflow_becomes_more() {
        let it = items(&[
            (TodoStatus::InProgress, "ip"),
            (TodoStatus::Pending, "p1"),
            (TodoStatus::Pending, "p2"),
            (TodoStatus::Pending, "p3"),
            (TodoStatus::Pending, "p4"),
        ]);
        // max_rows=4 → header + ip + (2 pending, but reserve 1 for More) → 1 pending + More{3}
        let rows = todo_panel_rows(&it, 0, 5, 4);
        assert_eq!(rows.len(), 4);
        assert!(matches!(rows.last().unwrap(), TodoPanelRow::More { hidden: 3 }));
    }

    #[test]
    fn in_progress_survives_tight_budget() {
        // body budget = 1: in-progress wins the single slot over the completed fold.
        let it = items(&[(TodoStatus::Completed, "done"), (TodoStatus::InProgress, "ip")]);
        let rows = todo_panel_rows(&it, 1, 2, 2);
        assert_eq!(rows.len(), 2);
        assert!(matches!(&rows[1], TodoPanelRow::Item { status: TodoStatus::InProgress, .. }));
    }

    #[test]
    fn never_exceeds_max_rows() {
        let mut it = items(&[(TodoStatus::InProgress, "ip"), (TodoStatus::Completed, "c")]);
        for i in 0..20 { it.push((TodoStatus::Pending, format!("p{i}"))); }
        let rows = todo_panel_rows(&it, 1, 22, MAX_TODO_PANEL_ROWS);
        assert!(rows.len() <= MAX_TODO_PANEL_ROWS);
    }
}
```

- [ ] **步骤 2：运行测试以确认它失败**

运行：`cargo test -p rustcode-tuix todo_panel_rows_tests`
预期：FAIL —— `cannot find function todo_panel_rows`。

- [ ] **步骤 3：实现 const、enum 与函数**（放在 `todo_row_parts` 之后，约 252 行）：

```rust
/// Max rows the footer todo panel may occupy, INCLUDING the header. The panel
/// is additionally clamped against screen height by the caller.
const MAX_TODO_PANEL_ROWS: usize = 6;

/// One logical row of the collapsed todo panel. Pure structure — glyphs,
/// i18n words, styling and width-fitting are applied in `build_todo_rows`.
#[derive(Debug, Clone, PartialEq)]
enum TodoPanelRow {
    Header { completed: usize, total: usize },
    CompletedFold { count: usize },
    Item {
        status: rustcode_capabilities::tools::todo::TodoStatus,
        content: String,
    },
    More { hidden: usize },
}

/// Collapse a todo list into at most `max_rows` panel rows (incl. header).
///
/// Selection priority under a tight budget: the in-progress task always shows
/// when present, then the completed fold, then pending items. Pending overflow
/// collapses into a single `More` row (which itself costs a row). Display order
/// is: Header, CompletedFold?, InProgress?, Pending…, More?.
fn todo_panel_rows(
    items: &[(rustcode_capabilities::tools::todo::TodoStatus, String)],
    completed: usize,
    total: usize,
    max_rows: usize,
) -> Vec<TodoPanelRow> {
    use rustcode_capabilities::tools::todo::TodoStatus;
    let mut rows = vec![TodoPanelRow::Header { completed, total }];
    let body_budget = max_rows.saturating_sub(1);
    if body_budget == 0 {
        return rows;
    }

    let in_progress: Option<&String> = items
        .iter()
        .find(|(s, _)| *s == TodoStatus::InProgress)
        .map(|(_, c)| c);
    let pendings: Vec<&String> = items
        .iter()
        .filter(|(s, _)| *s == TodoStatus::Pending)
        .map(|(_, c)| c)
        .collect();

    // Reserve high-priority slots first (in-progress, then completed fold).
    let mut used = 0usize;
    let show_ip = in_progress.is_some() && used < body_budget;
    if show_ip {
        used += 1;
    }
    let show_fold = completed > 0 && used < body_budget;
    if show_fold {
        used += 1;
    }

    // Remaining budget for pending rows (+ possible More row).
    let pend_budget = body_budget.saturating_sub(used);
    let (shown_pending, hidden) = if pend_budget == 0 {
        (0usize, 0usize) // header N/M still reflects them
    } else if pendings.len() <= pend_budget {
        (pendings.len(), 0)
    } else {
        let shown = pend_budget - 1; // reserve 1 row for the More marker
        (shown, pendings.len() - shown)
    };

    // Emit in display order.
    if show_fold {
        rows.push(TodoPanelRow::CompletedFold { count: completed });
    }
    if show_ip {
        if let Some(c) = in_progress {
            rows.push(TodoPanelRow::Item {
                status: TodoStatus::InProgress,
                content: c.clone(),
            });
        }
    }
    for c in pendings.iter().take(shown_pending) {
        rows.push(TodoPanelRow::Item {
            status: TodoStatus::Pending,
            content: (*c).clone(),
        });
    }
    if hidden > 0 {
        rows.push(TodoPanelRow::More { hidden });
    }
    rows
}
```

- [ ] **步骤 4：运行测试以确认它们通过**

运行：`cargo test -p rustcode-tuix todo_panel_rows_tests`
预期：PASS（全部 5 个）。

- [ ] **步骤 5：提交**

```bash
git add crates/rustcode-tuix/src/render/retained.rs
git commit -m "feat(tuix): pure todo-panel collapse (todo_panel_rows)"
```

---

## 任务 4：把面板渲染为 cell 并接入页脚高度

**文件：**
- 修改：`crates/rustcode-tuix/src/render/retained.rs`
  - 新增 `build_todo_rows` + `todo_panel_row_count`（在 `build_todo_row` 附近，约 1707 行）
  - `paint_footer`：`todo_rows` 计算（1816）、`todo_cells` 构造（1878-1881）、绘制循环（1964-1974）
  - `current_footer_rows`：`todo_rows` 计算（约 2034）
  - 删除现已不再使用的单行 `build_todo_row`（1707-1717）
- 测试：同一文件

**接口：**
- 消费：`todo_panel_rows`、`MAX_TODO_PANEL_ROWS`、`TodoPanelRow`（任务 3）；`TodoProgress.items`（任务 1）；`Msg::TodoPanel*`（任务 2）；`todo_marker`、`todo_glyph`、`build_marker_row`、`push_str_cells`、`style_for`、`CellStyle`、`scrub_controls`、`crate::width`。
- 产出：`fn build_todo_rows(&self, todo: &TodoProgress, rule_width: usize) -> Vec<Vec<Cell>>`；`fn todo_panel_row_count(&self, todo: &TodoProgress) -> usize`。

- [ ] **步骤 1：编写失败测试** —— 添加到 retained 测试模块：

```rust
    #[test]
    fn build_todo_rows_header_and_inprogress() {
        use rustcode_capabilities::tools::todo::TodoStatus;
        let r = renderer_80x24_unicode(); // existing test helper that builds a Renderer
        let todo = crate::render::TodoProgress {
            current: Some("wire it".into()),
            completed: 1,
            total: 3,
            items: vec![
                (TodoStatus::Completed, "done a".into()),
                (TodoStatus::InProgress, "wire it".into()),
                (TodoStatus::Pending, "later".into()),
            ],
        };
        let rows = r.build_todo_rows(&todo, 40);
        let text = |cells: &Vec<Cell>| cells.iter().map(|c| c.ch).collect::<String>();
        assert!(text(&rows[0]).contains("Todos") && text(&rows[0]).contains("1/3"));
        // in-progress row is bold
        let ip = rows.iter().find(|row| text(row).contains("wire it")).unwrap();
        assert!(ip.iter().any(|c| c.style.bold));
    }
```

注意：若 `renderer_80x24_unicode()` / 等价构造器在测试模块中并非确切的辅助函数名，请使用周围测试已经在用的、构造 `Renderer` 的辅助函数（在测试模块中 grep `fn renderer` / `Renderer::new`）。断言逻辑不变。

- [ ] **步骤 2：运行测试以确认它失败**

运行：`cargo test -p rustcode-tuix build_todo_rows_header_and_inprogress`
预期：FAIL —— `no method named build_todo_rows`。

- [ ] **步骤 3：把 `build_todo_row`（1707-1717）替换为多行构造器 + 计数辅助函数：**

```rust
    /// Effective panel height cap: `MAX_TODO_PANEL_ROWS`, clamped so the panel
    /// never claims more than the screen can spare (rules + one input + status).
    fn todo_panel_cap(&self) -> usize {
        let h = self.screen.height() as usize;
        MAX_TODO_PANEL_ROWS.min(h.saturating_sub(4)).max(1)
    }

    /// Number of rows the todo panel will occupy — mirrors `build_todo_rows`'
    /// row count without building cells (used by the footer height math).
    fn todo_panel_row_count(&self, todo: &crate::render::TodoProgress) -> usize {
        todo_panel_rows(&todo.items, todo.completed, todo.total, self.todo_panel_cap()).len()
    }

    /// Build the multi-line todo panel: a header marker row (`☑ Todos · N/M`)
    /// followed by collapsed item rows. Sits directly above the status line (and
    /// above the goal/loop row when present). Theme-safe: Brand marker, bold
    /// in-progress, faint completed/fold, Muted pending/more. ASCII fallback via
    /// `todo_marker`/`todo_glyph`.
    fn build_todo_rows(
        &self,
        todo: &crate::render::TodoProgress,
        rule_width: usize,
    ) -> Vec<Vec<Cell>> {
        use rustcode_capabilities::tools::todo::{todo_glyph, TodoStatus};
        let unicode = self.caps.unicode_symbols;
        let rows = todo_panel_rows(&todo.items, todo.completed, todo.total, self.todo_panel_cap());

        // width budget for an indented item line: `  <glyph> <content>`
        let item_line = |glyph: &str, body: &str, style: &CellStyle| -> Vec<Cell> {
            let mut row = Vec::new();
            let gw = crate::width::display_width(glyph);
            let budget = rule_width.saturating_sub(2 + gw + 1); // indent + glyph + space
            let fitted = crate::width::truncate_with_ellipsis(&scrub_controls(body), budget);
            push_str_cells(&mut row, &format!("  {glyph} {fitted}"), style);
            row
        };

        rows.into_iter()
            .map(|r| match r {
                TodoPanelRow::Header { completed, total } => self.build_marker_row(
                    todo_marker(unicode),
                    &crate::i18n::t(crate::i18n::Msg::TodoPanelTitle).into_owned(),
                    &format!(" \u{b7} {completed}/{total}"),
                ),
                TodoPanelRow::CompletedFold { count } => {
                    let style = CellStyle { faint: true, ..self.style_for(Role::Muted) };
                    let label = crate::i18n::t(crate::i18n::Msg::TodoPanelCompleted { n: count })
                        .into_owned();
                    item_line(todo_glyph(TodoStatus::Completed, unicode), &label, &style)
                }
                TodoPanelRow::Item { status, content } => {
                    let style = match status {
                        TodoStatus::InProgress => {
                            CellStyle { bold: true, ..self.style_for(Role::Brand) }
                        }
                        TodoStatus::Completed => {
                            CellStyle { faint: true, ..self.style_for(Role::Muted) }
                        }
                        TodoStatus::Pending => self.style_for(Role::Muted),
                    };
                    item_line(todo_glyph(status, unicode), &content, &style)
                }
                TodoPanelRow::More { hidden } => {
                    let style = self.style_for(Role::Muted);
                    let label =
                        crate::i18n::t(crate::i18n::Msg::TodoPanelMore { n: hidden }).into_owned();
                    item_line(todo_glyph(TodoStatus::Pending, unicode), &label, &style)
                }
            })
            .collect()
    }
```

- [ ] **步骤 4：更新 `paint_footer` 的高度与绘制。**

（4a）替换 `todo_rows` 那一行（1816）：

```rust
        let todo_rows = self
            .status
            .todo
            .as_ref()
            .map(|t| self.todo_panel_row_count(t))
            .unwrap_or(0);
```

（4b）替换 `todo_cells` 的预构造（1878-1881）：

```rust
        let todo_cells: Vec<Vec<Cell>> = status_clone
            .todo
            .as_ref()
            .map(|t| self.build_todo_rows(t, rule_width))
            .unwrap_or_default();
```

（4c）替换待办绘制 + 状态绘制代码块（1964-1974）：

```rust
        let todo_top = goal_top + goal_rows;
        for (i, tr) in todo_cells.into_iter().enumerate() {
            let mut padded = tr;
            Self::pad_row_to_width(&mut padded, w);
            self.screen.draw_row(todo_top + i, 0, &padded);
        }
        if let Some(st) = status_cells {
            let mut padded = st;
            Self::pad_row_to_width(&mut padded, w);
            self.screen.draw_row(todo_top + todo_rows, 0, &padded);
        }
```

- [ ] **步骤 5：更新 `current_footer_rows`** —— 替换 `todo_rows` 那一行（约 2034）：

```rust
        let todo_rows = self
            .status
            .todo
            .as_ref()
            .map(|t| self.todo_panel_row_count(t))
            .unwrap_or(0);
```

- [ ] **步骤 6：运行测试**

运行：`cargo test -p rustcode-tuix build_todo_rows_header_and_inprogress`，然后 `cargo test -p rustcode-tuix --lib`
预期：PASS。4 个既有的 retained byte-budget 红测属于已知无关项（依据仓库 lore）—— 确认没有**新增**失败。

- [ ] **步骤 7：提交**

```bash
git add crates/rustcode-tuix/src/render/retained.rs
git commit -m "feat(tuix): render multi-line todo panel in footer"
```

---

## 任务 5：持久化状态 + 实时捕获（无内联块）+ 全部完成时隐藏 + 重置

**文件：**
- 修改：`crates/rustcode-tuix/src/state.rs`（341 字段，475 初始化，724/742/753 清理）
- 修改：`crates/rustcode-tuix/src/event_loop/mod.rs`（8754-8774 实时分支，10767 读取过滤）
- 修改：`crates/rustcode-tuix/src/event_loop/commands.rs`（reset_to_new_session，约 4295）
- 测试：`crates/rustcode-tuix/src/state.rs`

**接口：**
- 产出：`UiState.active_todos: Option<TodoProgress>` —— 持久化（turn 结束时**不**清理）；页脚以 `total > 0 && completed < total` 过滤后读取。

- [ ] **步骤 1：编写失败测试** —— 添加到 state.rs 的测试模块（在 state.rs 中 grep `mod tests`；若不存在就新增一个）：

```rust
    #[test]
    fn active_todos_persists_across_turn_end() {
        let mut s = UiState::default(); // or the existing test constructor
        s.active_todos = Some(crate::render::TodoProgress {
            current: Some("x".into()),
            completed: 0,
            total: 2,
            items: vec![],
        });
        s.on_turn_complete();
        assert!(s.active_todos.is_some(), "panel must survive turn end");
    }
```

- [ ] **步骤 2：运行测试以确认它失败**

运行：`cargo test -p rustcode-tuix active_todos_persists_across_turn_end`
预期：FAIL —— `no field active_todos`（字段仍名为 `live_turn_todo`）。

- [ ] **步骤 3：在 `state.rs` 中重命名并修改语义。**

（3a）替换字段（335-341）的文档注释与声明：

```rust
    /// Active todo list for the persistent footer todo PANEL. Written from the
    /// turn's `todowrite` calls, seeded from the transcript on resume/switch
    /// (`replay_session`), reset on `/clear`/`/new` (`reset_to_new_session`).
    /// Unlike the old live-only row, this PERSISTS across turn boundaries — the
    /// panel is a standing view, hidden only when the list is empty or all done.
    pub active_todos: Option<crate::render::TodoProgress>,
```

（3b）初始化（475）：`live_turn_todo: None,` → `active_todos: None,`

（3c）移除三处 turn 结束清理 —— 删除 724、742、753 行的 `self.live_turn_todo = None;`。（保留其周围的 `subagent_activity = None;` 等不动。同时删除/调整 720-723 行那段解释 live-only 交接、现已过时的注释块。）

- [ ] **步骤 4：更新实时捕获分支** —— 在 `event_loop/mod.rs`（8754-8774）中，把整个 `if name == "todowrite" { … }` 块替换为仅捕获（没有内联块，仍然抑制工具结果）：

```rust
            // todowrite: the persistent footer PANEL is the sole view. Capture the
            // full list into `active_todos` (the transcript won't carry it until
            // turn end), and suppress the tool CALL + RESULT rows. On a parse
            // failure fall through to the normal tool row so the error surfaces.
            if name == "todowrite" {
                if let Some(progress) = todo_progress_from_args(&arguments) {
                    state.active_todos = Some(progress);
                    // call_rendered=true ⇒ ToolCallResult suppresses the result row.
                    pending_tools.insert(id, (display.clone(), detail, true));
                    state.on_tool_call_started(&display);
                    return;
                }
            }
```

（注意：`todo_block_styled_lines`、用于该块的 `UiLine::AssistantLineBreak` / `CommandOutput`，以及旧块中的 `renderer.flush()` 均已消失 —— 由面板取而代之。）

- [ ] **步骤 5：更新页脚读取过滤**（10761-10767）—— 把注释 + `let todo = …` 替换为：

```rust
    // Todo panel source: the persistent `active_todos` cache. Hidden when the
    // list is empty (`total == 0`) or fully done (`completed == total`) so a
    // finished panel disappears — otherwise it stands across turns and resume.
    let todo = state
        .active_todos
        .clone()
        .filter(|p| p.total > 0 && p.completed < p.total);
```

- [ ] **步骤 6：新会话时重置** —— 在 `commands.rs` 的 `reset_to_new_session` 中，于 `state.on_turn_complete();`（4295 行）之后添加：

```rust
    state.active_todos = None;
```

- [ ] **步骤 7：运行测试 + 构建**

运行：`cargo build -p rustcode-tuix && cargo test -p rustcode-tuix active_todos_persists_across_turn_end`
预期：构建干净（所有 `live_turn_todo` 引用均已更新 —— 由编译器强制保证），测试 PASS。

- [ ] **步骤 8：提交**

```bash
git add crates/rustcode-tuix/src/state.rs crates/rustcode-tuix/src/event_loop/mod.rs crates/rustcode-tuix/src/event_loop/commands.rs
git commit -m "feat(tuix): persistent active_todos, capture-only todowrite, hide when done"
```

---

## 任务 6：回放 —— 移除内联块，从 transcript 播种面板

**文件：**
- 修改：`crates/rustcode-tuix/src/modals/session_picker.rs`（576-593 回放分支；`replay_session` 的末尾，约在消息循环之后）
- 修改：`crates/rustcode-tuix/src/event_loop/mod.rs` —— 新增 `todo_progress_from_messages`
- 测试：`crates/rustcode-tuix/src/modals/session_picker.rs`（现有回放测试位于约 998/1083 行）

**接口：**
- 消费：`derive_current_todos`（capabilities）、`todo_progress_from_items`（任务 1）。
- 产出：`fn todo_progress_from_messages(messages: &[Message]) -> Option<TodoProgress>`。

- [ ] **步骤 1：编写失败测试** —— 添加到 session_picker 测试模块：

```rust
    #[test]
    fn replay_seeds_active_todos_from_transcript() {
        use rustcode_core::conversation::message::Message;
        use rustcode_kernel::tool::ToolCall;
        let mut rec = /* the existing recording-renderer used by neighbouring tests */;
        let mut state = /* the existing UiState test constructor used nearby */;
        let mut session = rustcode_core::session::Session::default_session(".".into());
        session.messages = vec![Message::assistant(
            "",
            vec![ToolCall {
                id: "1".into(),
                name: "todowrite".into(),
                arguments: r#"{"todos":[{"content":"a","status":"in_progress"},{"content":"b","status":"pending"}]}"#.into(),
            }],
        )];
        replay_session(&mut rec, &mut state, &session, false);
        let p = state.active_todos.expect("panel seeded from transcript");
        assert_eq!(p.total, 2);
        assert_eq!(p.current.as_deref(), Some("a"));
    }
```

（把 `rec`/`state` 的构造与 998/1083 行附近两处现有 `replay_session(&mut rec, &mut state, &session, false)` 测试对齐 —— 逐字复制它们的 setup。）

- [ ] **步骤 2：运行测试以确认它失败**

运行：`cargo test -p rustcode-tuix replay_seeds_active_todos_from_transcript`
预期：FAIL —— 回放后 `active_todos` 仍是 `None`。

- [ ] **步骤 3：新增 `todo_progress_from_messages`** —— 在 `event_loop/mod.rs` 中（`todo_progress_from_args` 旁，约 11842 行）：

```rust
/// Todo panel state derived from a full transcript — the last VALID `todowrite`
/// call wins (see `derive_current_todos`). `None` when the session never used
/// todowrite. Used to seed the panel on `/resume` / session switch with zero
/// extra storage.
pub(crate) fn todo_progress_from_messages(
    messages: &[rustcode_kernel::message::Message],
) -> Option<crate::render::TodoProgress> {
    let todos = rustcode_capabilities::tools::todo::derive_current_todos(messages);
    if todos.is_empty() {
        None
    } else {
        Some(todo_progress_from_items(&todos))
    }
}
```

（若 `rustcode_kernel::message::Message` 并非 `Session.messages` 所持有的类型，请使用回放循环实际迭代的那个类型 —— grep `session.messages` 的元素类型；`derive_current_todos` 接收 `&[rustcode_kernel::message::Message]`，据此转换/借用。`session_picker.rs` 已经导入了它所需的消息类型。）

- [ ] **步骤 4：从回放中剥离内联块** —— 把 todowrite 分支（576-593）替换为仅抑制：

```rust
                for tc in tool_calls {
                    // todowrite → no inline block; the persistent panel is the
                    // sole view. Suppress the (successful) tool RESULT below by
                    // remembering the call id. Mirror the live path: only a
                    // PARSEABLE call is suppressed — a bad one falls through to a
                    // normal tool row so its error still shows.
                    if tc.name == "todowrite"
                        && rustcode_capabilities::tools::todo::parse_todos(&tc.arguments).is_ok()
                    {
                        if !tc.id.is_empty() {
                            todowrite_call_ids.insert(tc.id.clone());
                        }
                        continue;
                    }
                    renderer.render(UiLine::ToolCall {
                        name: crate::event_loop::display_tool_name(&tc.name),
                        detail: format_tool_detail(&tc.name, &tc.arguments),
                    });
                }
```

- [ ] **步骤 5：播种面板** —— 在 `replay_session` 末尾，`for (i, m) in session.messages.iter().enumerate()` 循环闭合之后、最后的 `renderer.end_sync()` / return 之前，添加：

```rust
    // Seed the persistent todo panel from the transcript (zero extra storage).
    // This both RESETS the previous session's panel and rehydrates the loaded
    // one, so session switch / resume land on the correct list.
    state.active_todos = crate::event_loop::todo_progress_from_messages(&session.messages);
```

（通过阅读 `replay_session` 的尾部定位确切插入点；放在该函数最后一次 renderer flush/`end_sync` 之前。）

- [ ] **步骤 6：运行测试**

运行：`touch crates/rustcode-core/src/lib.rs && cargo test -p rustcode-tuix replay_seeds_active_todos_from_transcript`
预期：PASS。

- [ ] **步骤 7：提交**

```bash
git add crates/rustcode-tuix/src/modals/session_picker.rs crates/rustcode-tuix/src/event_loop/mod.rs
git commit -m "feat(tuix): seed todo panel on resume, drop inline replay block"
```

---

## 任务 7：删除现已失效的内联块辅助函数

**文件：**
- 修改：`crates/rustcode-tuix/src/event_loop/mod.rs` —— 移除 `todo_block_lines`（11781-11798）、`todo_block_styled_lines`（11800-11817），以及 `todo_block_tests` 中引用它们的用例（`..._weights_by_status` 测试与原始 SGR 断言）。

**接口：** 无（纯删除）。删除前先确认没有剩余调用者。

- [ ] **步骤 1：确认没有剩余调用者**

运行：`grep -rn "todo_block_styled_lines\|todo_block_lines" crates/`
预期：仅剩定义及其自身测试（两处调用点已在任务 5 和任务 6 中移除）。若仍有任何非测试调用者，先停下修复。

- [ ] **步骤 2：删除这两个函数**（`todo_block_lines`、`todo_block_styled_lines`）以及调用它们的测试。保留 `todo_progress_*` 测试（它们仍然有效）。若 `todo_block_tests` 因此变成空模块，则删除该空模块。

- [ ] **步骤 3：运行构建 + 测试**

运行：`cargo build -p rustcode-tuix && cargo test -p rustcode-tuix --lib`
预期：构建干净（被删函数不再产生 `unused function` 警告），测试除 4 个已知无关的 retained byte-budget 红测外全绿。

- [ ] **步骤 4：提交**

```bash
git add crates/rustcode-tuix/src/event_loop/mod.rs
git commit -m "refactor(tuix): remove dead inline todo-block renderers"
```

---

## 任务 8：整体验证

- [ ] **步骤 1：全工作区构建 + 测试**

运行：`touch crates/rustcode-core/src/lib.rs && cargo build && cargo test -p rustcode-tuix -p rustcode-capabilities -p rustcode-core`
预期：构建干净；tuix 除 4 个既有的 retained byte-budget 红测外全绿（确认它们与干净检出下是**同样的 4 个** —— 依据仓库 lore 禁止 `git stash`；若不确定，改为在独立 worktree 中对父提交跑一次全新的 `cargo test` 来对比）。

- [ ] **步骤 2：手工冒烟（记录在案，非自动化）** —— 在 commit/PR 描述中记录以下需要真实终端的项（无法单元测试）：
  1. 触发一次多步 `todowrite`（开启 `RUSTCODE_TODO`）；确认面板出现在输入框上方，并跨 turn **原地更新**（滚动区中不再出现重复的内联块）。
  2. 长列表（>5 项）→ 已完成项折叠为一行，显示 in-progress，pending 以 `+K more…` 截断。
  3. 全部标记完成 → 面板消失。
  4. `/resume` 一个用过 todowrite 的会话 → 面板再水化。
  5. `/clear` → 面板消失。
  6. 非 unicode 终端（`TERM` 不带 unicode / caps 关闭）→ 回退到 `+`/`[~]`/`[x]`/`[ ]` 这些 ASCII 字形。
  7. 窄且矮的终端 → 输入框仍然可用（面板让位），无溢出。

- [ ] **步骤 3：申请代码评审**

合并前对分支 diff 使用 `superpowers:requesting-code-review` 技能（或 `/code-review`）。

---

## 自审（撰写期间完成）

- **规范覆盖：** §数据模型→T1；§生命周期(跨turn/隐藏/清空)→T5；§Resume→T6；§渲染→T4；§折叠算法→T3；§样式/主题→T4；§字形/降级→T4（ASCII 经由 todo_glyph/todo_marker）；§内联块移除→T5(live)+T6(replay)+T7(delete)；§开关→不变（全局约束）；§测试→各任务自测 + T8。i18n 纪律 → T2。
- **占位符扫描：** 无 —— 每个代码步骤都给出了完整代码。两处被标记的查找（T4-S1 的 `renderer_80x24_unicode` 辅助函数名，T6-S1 的回放测试 `rec`/`state` setup）明确是“复制邻近测试的 setup”，不是 TODO。
- **类型一致性：** `active_todos: Option<TodoProgress>`（T5）在读取过滤（T5）、回放播种（T6）、重置（T5）中的用法完全一致。`TodoProgress.items: Vec<(TodoStatus, String)>`（T1）被 `todo_panel_rows`（T3）和 `build_todo_rows`（T4）消费。`todo_panel_rows`/`TodoPanelRow`/`MAX_TODO_PANEL_ROWS` 的命名在 T3/T4 中一致。`Msg::TodoPanelTitle|Completed{n}|More{n}` 在 T2/T4 中一致。
