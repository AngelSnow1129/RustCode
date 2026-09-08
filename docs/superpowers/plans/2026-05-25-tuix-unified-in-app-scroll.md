# TUI 统一应用内滚动实施方案

> **致 agentic worker：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 来逐任务实施本计划。步骤使用复选框（`- [ ]`）语法跟踪进度。

**Goal:** 把 retained 和 alt-screen 两个 renderer 的 body 滚动统一到 in-app 缓冲，消除 retained 模式下偶现的"无法滚动"问题，并加可视滚动条 + 跳消息键 + `/keys` 文档。

**Architecture:** retained 加 `view_mode` 状态机，sticky 跟底时走原 DECSTBM 流式，view_mode 时切到 alt-screen 风格的 CUP+EL 重绘；retained 同步接管鼠标（`?1002h ?1006h`）。selection 逻辑抽到 `render/selection.rs` 共享模块，alt-screen 和 retained 共用。两个 renderer 共享 `MessageMark` + 滚动条 + 跳转算法。

**Tech Stack:** Rust, crossterm, alt_screen.rs 现有 alt-screen 渲染器，retained.rs 现有 DECSTBM 渲染器。详细设计见 `docs/superpowers/specs/2026-05-25-tuix-unified-in-app-scroll-design.md`。

---

## 文件结构

**新增文件：**
- `crates/rustcode-tuix/src/render/selection.rs` — 共享选择模块（trait + 状态 + 高亮 + OSC 52 / arboard 复制）
- `crates/rustcode-tuix/src/render/scrollbar.rs` — 滚动条绘制 helper
- `crates/rustcode-tuix/src/render/ui_state.rs` — `$RUSTCODE_HOME/ui-state.toml` 读写

**修改的文件：**
- `crates/rustcode-tuix/src/render/mod.rs` — `Renderer` trait 加方法
- `crates/rustcode-tuix/src/render/worker.rs` — 新方法通过 worker 转发
- `crates/rustcode-tuix/src/render/alt_screen.rs` — 切到 shared selection 模块，加 scrollbar 接入，加 MessageMark + 跳转
- `crates/rustcode-tuix/src/render/retained.rs` — 大改：view_mode、body 缓冲扩容、scroll_body、mouse 接管、selection 接入、MessageMark + 跳转、scrollbar 接入
- `crates/rustcode-tuix/src/event_loop/mod.rs` — `handle_scroll_key` 加 Alt+↑↓ / Ctrl+↑↓
- `crates/rustcode-tuix/src/event_loop/commands.rs` — `/scrollbar` 命令处理
- `crates/rustcode-tuix/src/commands.rs` — 注册 `scrollbar` 命令
- `crates/rustcode-core/src/i18n/messages.rs` — 新 `Msg` 变体
- `crates/rustcode-core/src/i18n/zh_cn.rs` — i18n 文案 + `KeybindingsHelp` 更新
- `crates/rustcode-core/src/i18n/en.rs` — 同上

---

## 阶段 0：i18n 消息变体

### 任务 0.1：添加新的 Msg 变体

**文件：**
- 修改：`crates/rustcode-core/src/i18n/messages.rs`
- 修改：`crates/rustcode-core/src/i18n/zh_cn.rs`
- 修改：`crates/rustcode-core/src/i18n/en.rs`

后续 phase 引用 `Msg::ScrollbarOn` / `Msg::ScrollbarOff` / `CmdDescScrollbar`。先添加，避免后面分散加。

- [ ] **步骤 1：添加 Msg 枚举变体**

编辑 `crates/rustcode-core/src/i18n/messages.rs`，在 `Msg` 枚举中添加：

```rust
ScrollbarOn,
ScrollbarOff,
CmdDescScrollbar,
```

- [ ] **步骤 2：添加 zh_cn 翻译**

编辑 `crates/rustcode-core/src/i18n/zh_cn.rs`，在 `t()` 的 match 中添加新的分支：

```rust
Msg::ScrollbarOn => "Scrollbar: ON".into(),
Msg::ScrollbarOff => "Scrollbar: OFF".into(),
Msg::CmdDescScrollbar => "切换右侧滚动条显示".into(),
```

- [ ] **步骤 3：添加 en 翻译**

编辑 `crates/rustcode-core/src/i18n/en.rs`：

```rust
Msg::ScrollbarOn => "Scrollbar: ON".into(),
Msg::ScrollbarOff => "Scrollbar: OFF".into(),
Msg::CmdDescScrollbar => "Toggle the right-side scrollbar".into(),
```

- [ ] **步骤 4：构建验证**

运行：`cargo check -p rustcode-core`
预期：构建干净，没有 match 非穷尽的告警。

- [ ] **步骤 5：提交**

```bash
git add crates/rustcode-core/src/i18n/{messages.rs,zh_cn.rs,en.rs}
git commit -m "i18n: add ScrollbarOn/Off + CmdDescScrollbar messages"
```

---

## Phase 1: Selection 共享模块

### 任务 1.1：创建 selection.rs 骨架

**文件：**
- 新建：`crates/rustcode-tuix/src/render/selection.rs`
- 修改：`crates/rustcode-tuix/src/render/mod.rs`（声明模块）

抽 alt-screen 的 selection 代码到独立模块，先建骨架与类型。

- [ ] **步骤 1：创建带类型的 selection.rs**

创建 `crates/rustcode-tuix/src/render/selection.rs`：

```rust
//! Shared text-selection module used by both AltScreenRenderer and
//! RetainedRenderer. Owns: anchor/head pos, drag tracking, range
//! computation, line rendering with reverse-video highlight, OSC 52
//! emission and arboard fallback for Ctrl+C copy.
//!
//! Each renderer holds a `SelectionState` and implements `BodyLineView`
//! over its native body buffer type (`Vec<String>` for alt-screen,
//! `Vec<Vec<Cell>>` for retained).

use std::borrow::Cow;

/// A single (row, col) cursor position in body_lines coordinates.
/// `row` is the index into body_lines; `col` is display-column.
pub type BodyPos = (usize, u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    pub anchor: BodyPos,
    pub head: BodyPos,
}

#[derive(Debug, Default)]
pub struct SelectionState {
    pub selection: Option<Selection>,
    pub active: bool,  // true while mouse button held down
}

/// Trait adapter so the selection module can read body content without
/// caring whether the renderer stores `Vec<String>` or `Vec<Vec<Cell>>`.
pub trait BodyLineView {
    fn line_count(&self) -> usize;
    fn line_text(&self, idx: usize) -> Cow<'_, str>;
}

// Impl for the alt-screen body_lines type.
impl BodyLineView for Vec<String> {
    fn line_count(&self) -> usize { self.len() }
    fn line_text(&self, idx: usize) -> Cow<'_, str> {
        Cow::Borrowed(self.get(idx).map(|s| s.as_str()).unwrap_or(""))
    }
}
```

- [ ] **步骤 2：把模块接入 render/mod.rs**

编辑 `crates/rustcode-tuix/src/render/mod.rs`。找到现有的 `pub mod alt_screen;` 代码块，并在其附近添加：

```rust
pub mod selection;
```

- [ ] **步骤 3：构建验证接线**

运行：`cargo check -p rustcode-tuix`
预期：构建干净。

- [ ] **步骤 4：提交**

```bash
git add crates/rustcode-tuix/src/render/{mod.rs,selection.rs}
git commit -m "tuix(render): add selection module skeleton + BodyLineView trait"
```

### 任务 1.2：迁移 SGR 感知的文本辅助函数 + OSC 52 发送器

**文件：**
- 修改：`crates/rustcode-tuix/src/render/selection.rs`
- 修改：`crates/rustcode-tuix/src/render/alt_screen.rs`（删除已迁移的函数）

把 alt_screen.rs 现有的 `line_display_width_sgr_aware`、`extract_line_selection_text`、`render_line_with_selection`、`selection_col_range_for_line` 搬到 selection.rs。**同时移走** `base64_encode`（alt_screen.rs:269）和 `write_osc52_clipboard`（line 953），后者改名为 `pub fn emit_osc52(out: &mut dyn Write, text: &str)` 以便两个 renderer 共用。

- [ ] **步骤 1：定位现有函数**

运行：
```bash
grep -nE "^fn line_display_width_sgr_aware|^fn extract_line_selection_text|^fn render_line_with_selection|^fn selection_col_range_for_line|^fn base64_encode|fn write_osc52_clipboard" crates/rustcode-tuix/src/render/alt_screen.rs
```
预期：6 个行号（4 个文本辅助函数 + base64_encode + write_osc52_clipboard）。

- [ ] **步骤 2：把函数原样复制到 selection.rs**

打开 `crates/rustcode-tuix/src/render/alt_screen.rs`，复制全部 6 个函数的函数体。粘贴到 `crates/rustcode-tuix/src/render/selection.rs` 的 trait impl 之后。把 `fn` 改成 `pub fn`，并把内部的 `use` 路径调整为指向 `crate::width::display_width` 等。对于 `write_osc52_clipboard`，重命名为 `emit_osc52`，并把签名改为 `out` 是 `&mut dyn std::io::Write`：

```rust
pub fn emit_osc52(out: &mut dyn std::io::Write, text: &str) {
    if text.is_empty() { return; }
    let encoded = base64_encode(text.as_bytes());
    let _ = write!(out, "\x1b]52;c;{}\x07", encoded);
    let _ = out.flush();
}
```

确认 import 能编译通过：

运行：`cargo check -p rustcode-tuix`
预期：很可能报重复符号错误 —— 这是对的，下一步会修掉。

- [ ] **步骤 3：从 alt_screen.rs 删除原函数**

从 alt_screen.rs 中删除原来的 6 个函数（4 个文本辅助函数、`base64_encode` 和 `write_osc52_clipboard`）。

- [ ] **步骤 4：更新 alt_screen.rs 中的调用点**

在 alt_screen.rs 顶部添加 `use crate::render::selection::{self, selection_col_range_for_line, render_line_with_selection, extract_line_selection_text, line_display_width_sgr_aware, emit_osc52};`。把裸调用换成导入进来的名字。原先的 `self.write_osc52_clipboard(text)` 调用点（在 `end_selection` 中）改成 `selection::emit_osc52(&mut self.out, text);`。

- [ ] **步骤 5：运行 alt_screen 的选择相关测试**

运行：`cargo test -p rustcode-tuix --lib render::alt_screen::tests:: -- selection 2>&1 | tail -30`
预期：所有与选择相关的测试通过（`line_display_width_skips_sgr`、`extract_line_selection_strips_sgr_and_clips_to_range`、`render_line_with_selection_emits_reverse_video`、`render_line_with_selection_drops_inline_csi_inside_range`、`render_line_with_empty_selection_is_plain_truncate`、`selection_range_clamps_to_line_width`、`selection_range_multi_line_shape`）。

- [ ] **步骤 6：把测试体迁移到 selection.rs**

把测试从 alt_screen.rs 的 `tests` 模块迁移到 selection.rs 底部新建的 `#[cfg(test)] mod tests` 代码块中。调整 import 路径。

- [ ] **步骤 7：运行共享模块测试**

运行：`cargo test -p rustcode-tuix --lib render::selection::tests 2>&1 | tail -30`
预期：7 个测试全部通过。

- [ ] **步骤 8：提交**

```bash
git add crates/rustcode-tuix/src/render/{alt_screen.rs,selection.rs}
git commit -m "tuix(selection): move SGR-aware text helpers + tests to shared module"
```

### 任务 1.3：迁移 SelectionState 鼠标处理逻辑

**文件：**
- 修改：`crates/rustcode-tuix/src/render/selection.rs`
- 修改：`crates/rustcode-tuix/src/render/alt_screen.rs`

把 alt-screen 的 `begin_selection`/`update_selection`/`end_selection`/`copy_selection` 逻辑搬到 `SelectionState` methods，参数化 `BodyLineView`。

- [ ] **步骤 1：定位现有实现**

运行：
```bash
grep -nE "fn begin_selection|fn update_selection|fn end_selection|fn copy_selection|fn screen_to_body" crates/rustcode-tuix/src/render/alt_screen.rs
```
预期：5 个行号（4 个 trait impl + 1 个辅助函数 `screen_to_body`）。

- [ ] **步骤 2：向 selection.rs 添加 SelectionState 方法**

在 `selection.rs` 中添加（假定你已经抽出了原有逻辑；保留 OSC 52 + arboard 行为）：

```rust
impl SelectionState {
    /// Start a new selection at body coordinates `pos`.
    pub fn begin(&mut self, pos: BodyPos) {
        self.selection = Some(Selection { anchor: pos, head: pos });
        self.active = true;
    }

    /// Extend selection head to `pos` while button held.
    pub fn update(&mut self, pos: BodyPos) {
        if !self.active { return; }
        if let Some(sel) = self.selection.as_mut() {
            sel.head = pos;
        }
    }

    /// Finalise selection. Returns the selected text if non-empty, so the
    /// caller can emit OSC 52 to the host terminal. Selection state is
    /// preserved so the highlight stays drawn until the next click.
    pub fn end<B: BodyLineView>(&mut self, body: &B) -> Option<String> {
        self.active = false;
        let sel = self.selection.as_ref()?;
        let text = extract_text(body, sel);
        if text.is_empty() { None } else { Some(text) }
    }

    /// Copy current selection to system clipboard via arboard. Returns
    /// true iff a non-empty selection was copied. Clears highlight.
    pub fn copy<B: BodyLineView>(&mut self, body: &B) -> bool {
        let Some(sel) = self.selection else { return false };
        let text = extract_text(body, &sel);
        if text.is_empty() { return false; }
        let copied = match arboard::Clipboard::new() {
            Ok(mut cb) => cb.set_text(text).is_ok(),
            Err(_) => false,
        };
        if copied {
            self.selection = None;
            self.active = false;
        }
        copied
    }

    pub fn clear(&mut self) {
        self.selection = None;
        self.active = false;
    }
}

/// Concatenate the selected text across (possibly multiple) body lines,
/// using the existing per-line range helpers.
fn extract_text<B: BodyLineView>(body: &B, sel: &Selection) -> String {
    let (lo, hi) = ord(sel.anchor, sel.head);
    let mut out = String::new();
    for row in lo.0..=hi.0 {
        let line = body.line_text(row);
        let Some((start, end)) = selection_col_range_for_line(row, lo, hi, &line) else {
            continue;
        };
        if row > lo.0 { out.push('\n'); }
        out.push_str(&extract_line_selection_text(&line, start, end));
    }
    out
}

fn ord(a: BodyPos, b: BodyPos) -> (BodyPos, BodyPos) {
    if a < b { (a, b) } else { (b, a) }
}
```

- [ ] **步骤 3：添加单元测试**

在 selection.rs 的 `#[cfg(test)] mod tests` 中：

```rust
#[test]
fn selection_state_begin_sets_anchor_and_active() {
    let mut s = SelectionState::default();
    s.begin((2, 5));
    assert_eq!(s.selection, Some(Selection { anchor: (2, 5), head: (2, 5) }));
    assert!(s.active);
}

#[test]
fn selection_state_update_only_while_active() {
    let mut s = SelectionState::default();
    s.begin((0, 0));
    s.update((1, 4));
    assert_eq!(s.selection.unwrap().head, (1, 4));
    s.active = false;
    s.update((2, 9));
    // head shouldn't change after active = false
    assert_eq!(s.selection.unwrap().head, (1, 4));
}

#[test]
fn selection_state_end_returns_concatenated_text() {
    let body: Vec<String> = vec!["first".into(), "second".into(), "third".into()];
    let mut s = SelectionState::default();
    s.begin((0, 3));
    s.update((2, 2));
    let text = s.end(&body).expect("non-empty");
    // Selection spans (0,3) → (2,2)
    assert_eq!(text, "st\nsecond\nthi");
}
```

- [ ] **步骤 4：运行测试**

运行：`cargo test -p rustcode-tuix --lib render::selection::tests -- 2>&1 | tail -20`
预期：全部通过。

- [ ] **步骤 5：提交**

```bash
git add crates/rustcode-tuix/src/render/selection.rs
git commit -m "tuix(selection): add SelectionState begin/update/end/copy with tests"
```

### 任务 1.4：alt-screen 使用共享的 SelectionState

**文件：**
- 修改：`crates/rustcode-tuix/src/render/alt_screen.rs`

替换 alt-screen 的 `selection: Option<Selection>` + `selection_active: bool` 字段为 `SelectionState`，trait 方法委托。

- [ ] **步骤 1：替换字段**

在 `AltScreenRenderer` 结构体定义中删除：
```rust
selection: Option<Selection>,
selection_active: bool,
```

添加：
```rust
selection: crate::render::selection::SelectionState,
```

相应更新 `Self { ... }` 构造函数（初始化为 `SelectionState::default()`）。删除 alt_screen.rs 中旧的 `Selection` 结构体定义（现在定义在 selection.rs 中）。

- [ ] **步骤 2：更新 trait 方法体**

替换 `begin_selection` 的方法体：
```rust
fn begin_selection(&mut self, col: u16, row: u16) {
    if let Some(pos) = self.screen_to_body(col, row) {
        self.selection.begin(pos);
    } else {
        self.selection.clear();
    }
    self.body_dirty = true;
    self.paint_frame();
}
```

`update_selection` 方法体：
```rust
fn update_selection(&mut self, col: u16, row: u16) {
    if let Some(pos) = self.screen_to_body(col, row) {
        self.selection.update(pos);
        self.body_dirty = true;
        self.paint_frame();
    }
}
```

`end_selection` 方法体：
```rust
fn end_selection(&mut self) {
    if let Some(text) = self.selection.end(&self.body_lines) {
        crate::render::selection::emit_osc52(&mut self.out, &text);
    }
}
```

`copy_selection` 方法体：
```rust
fn copy_selection(&mut self) -> bool {
    let copied = self.selection.copy(&self.body_lines);
    if copied {
        self.body_dirty = true;
        self.paint_frame();
    }
    copied
}
```

注意：保留原实现中 OSC 52 的确切报文格式。如果原实现用的是另一个 base64 库路径，照搬它的写法。

- [ ] **步骤 3：更新 paint_body 以使用共享范围 helper**

在 `paint_body` 中，现有的选择高亮代码大概会调用 `selection_col_range_for_line` / `render_line_with_selection`。把 import 指向 `crate::render::selection::*`。把 `self.selection` 的读取改成 `self.selection.selection`。

- [ ] **步骤 4：运行 alt-screen 全量测试**

运行：`cargo test -p rustcode-tuix --lib render::alt_screen::tests 2>&1 | tail -30`
预期：所有测试通过（包括 `multi_line_drag_extracts_across_rows` 等）。

- [ ] **步骤 5：提交**

```bash
git add crates/rustcode-tuix/src/render/alt_screen.rs
git commit -m "tuix(alt-screen): delegate selection to shared SelectionState"
```

---

## 阶段 2：retained body 缓冲 + MessageMark

### 任务 2.1：把 body_lines 上限扩展到 5000

**文件：**
- 修改：`crates/rustcode-tuix/src/render/retained.rs`

把 `height * 4` 的 cap 改为 `MAX_SCROLLBACK_ROWS = 5000` 常量，与 alt-screen 对齐。

- [ ] **步骤 1：编写失败测试**

在 retained.rs 的 `#[cfg(test)] mod tests` 中添加：

```rust
#[test]
fn retained_body_lines_cap_is_5000_not_height_times_4() {
    let (mut r, _buf) = new_capturing(80, 24);
    // Push 5050 user lines (use a method that goes through push_body_row).
    for i in 0..5050 {
        r.render(UiLine::User(format!("line {}", i)));
    }
    assert_eq!(r.body_lines.len(), 5000, "body_lines should cap at 5000, got {}", r.body_lines.len());
}
```

- [ ] **步骤 2：运行测试确认其失败**

运行：`cargo test -p rustcode-tuix --lib retained_body_lines_cap_is_5000 2>&1 | tail -10`
预期：FAIL —— 当前上限是 `height * 4 = 96`。

- [ ] **步骤 3：实现常量并替换内联表达式**

在 `retained.rs` 顶部附近（import 之后）添加：

```rust
/// Max body_lines kept in the in-app scrollback buffer (matches alt-screen).
/// Bounded so memory doesn't grow without limit on long sessions.
pub const MAX_SCROLLBACK_ROWS: usize = 5000;
```

把每一处 `(self.screen.height() as usize).saturating_mul(4).max(128)` 替换为 `MAX_SCROLLBACK_ROWS`。用 grep 确认：

运行：`grep -nE "saturating_mul\(4\)" crates/rustcode-tuix/src/render/retained.rs`
预期：没有结果。

- [ ] **步骤 4：运行测试**

运行：`cargo test -p rustcode-tuix --lib retained_body_lines_cap_is_5000 2>&1 | tail -10`
预期：PASS。

- [ ] **步骤 5：运行 retained 全量测试套件**

运行：`cargo test -p rustcode-tuix --lib render::retained::tests 2>&1 | tail -10`
预期：全部通过。

- [ ] **步骤 6：提交**

```bash
git add crates/rustcode-tuix/src/render/retained.rs
git commit -m "tuix(retained): cap body_lines at MAX_SCROLLBACK_ROWS=5000"
```

### 任务 2.2：添加 MessageMark 结构体与字段

**文件：**
- 修改：`crates/rustcode-tuix/src/render/retained.rs`
- 修改：`crates/rustcode-tuix/src/render/alt_screen.rs`

两个 renderer 都加 `message_marks: Vec<MessageMark>` 字段。共享 `MessageMark` 类型放 `render/mod.rs`。

- [ ] **步骤 1：在 render/mod.rs 中添加类型**

在 `crates/rustcode-tuix/src/render/mod.rs` 顶部附近（模块声明之后）添加：

```rust
/// Boundary marker for an originated message in the body buffer. Drives
/// "jump to prev/next message" navigation keys. Marked at push time;
/// kept in sync when body_lines drains from the front.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkKind {
    User,
    Assistant,
    ToolCall,
    ToolResult,
}

#[derive(Debug, Clone, Copy)]
pub struct MessageMark {
    pub line_idx: usize,
    pub kind: MarkKind,
}
```

- [ ] **步骤 2：为 RetainedRenderer 添加字段**

在 `RetainedRenderer<W>` 结构体定义中添加（放在 `body_lines` 附近）：

```rust
message_marks: Vec<crate::render::MessageMark>,
```

在构造函数中添加：
```rust
message_marks: Vec::new(),
```

- [ ] **步骤 3：为 AltScreenRenderer 添加字段**

在 `alt_screen.rs` 中做同样的改动。

- [ ] **步骤 4：构建验证**

运行：`cargo check -p rustcode-tuix`
预期：构建干净（目前还没有使用方，只是加了字段）。

- [ ] **步骤 5：提交**

```bash
git add crates/rustcode-tuix/src/render/{mod.rs,retained.rs,alt_screen.rs}
git commit -m "tuix(render): add MessageMark type + message_marks field on both renderers"
```

### 任务 2.3：push 时打消息标记 + drain 同步（retained）

**文件：**
- 修改：`crates/rustcode-tuix/src/render/retained.rs`

在 retained 的 `render(UiLine)` 的 User/Assistant/ToolCall/ToolResult 分支入口处打标记；body_lines drain front 时同步更新 marks。

- [ ] **步骤 1：编写失败测试**

```rust
#[test]
fn retained_message_marks_tracked_on_user_push() {
    let (mut r, _buf) = new_capturing(80, 24);
    r.render(UiLine::User("hi".into()));
    assert_eq!(r.message_marks.len(), 1);
    assert_eq!(r.message_marks[0].kind, crate::render::MarkKind::User);
}

#[test]
fn retained_message_marks_decremented_on_drain() {
    let (mut r, _buf) = new_capturing(80, 24);
    // Push 5005 user lines so body_lines drains 5 from front.
    for i in 0..5005 {
        r.render(UiLine::User(format!("line {}", i)));
    }
    // First mark's line_idx should reflect the drain: original idx=0 dropped,
    // remaining marks shifted by 5.
    assert_eq!(r.message_marks.len(), 5000);
    assert_eq!(r.message_marks[0].line_idx, 0, "first surviving mark should point at body_lines[0] after drain");
}
```

- [ ] **步骤 2：运行以确认失败**

运行：`cargo test -p rustcode-tuix --lib retained_message_marks 2>&1 | tail -15`
预期：FAIL —— 还没有 push 任何标记。

- [ ] **步骤 3：添加 mark_message 辅助函数**

在 retained.rs 的 impl 代码块中：

```rust
fn mark_message(&mut self, kind: crate::render::MarkKind) {
    self.message_marks.push(crate::render::MessageMark {
        line_idx: self.body_lines.len(),
        kind,
    });
}
```

- [ ] **步骤 4：把 mark_message 接入 render(UiLine) 各分支**

在 retained.rs 的 `render(line: UiLine)` 中（找到那个大的 `match line { UiLine::User(...) => ..., UiLine::AssistantText(...) => ..., UiLine::ToolCall(...) => ..., UiLine::ToolResult(...) => ..., ... }`）。

对每个开启新逻辑消息的分支（现有代码大概已经有 `push_user_row` / `push_tool_row` 之类的辅助函数），在该消息第一次 push body_lines **之前**插入 `self.mark_message(MarkKind::...)` 调用。

具体来说：
- `UiLine::User(...)` 分支 → 在 push 之前调用 `self.mark_message(MarkKind::User);`。
- `UiLine::AssistantText(...)` 分支：只在一轮的第一个 chunk 上打标记。最简单的判断方式：如果 `message_marks.last()` 不是 `MarkKind::Assistant`，或者其间触发过 `UiLine::TurnSeparator` / 新一轮边界，就 push 一个新标记。具体做法是加一个 `last_mark_was_assistant: bool`（在 `UiLine::User` / `UiLine::ToolCall` / `UiLine::TurnSeparator` 时清零），并用它控制是否插入标记。
- `UiLine::ToolCall(...)` 分支 → `MarkKind::ToolCall`。
- `UiLine::ToolResult(...)` 分支 → `MarkKind::ToolResult`。

在结构体和构造函数中同步新增 `last_mark_was_assistant` 字段。

- [ ] **步骤 5：在 push_body_row 中同步 drain**

在 `push_body_row` 中找到现有的 `body_lines.drain(0..drain)`（约在第 1426 行）。替换为：

```rust
let drain = self.body_lines.len() - MAX_SCROLLBACK_ROWS;
self.body_lines.drain(0..drain);
self.message_marks.retain(|m| m.line_idx >= drain);
for m in self.message_marks.iter_mut() {
    m.line_idx -= drain;
}
```

在其他任何会 drain body_lines 的地方做同样处理（搜索：`grep -nE "body_lines\.drain|body_lines\.remove" retained.rs`）。

- [ ] **步骤 6：运行测试**

运行：`cargo test -p rustcode-tuix --lib retained_message_marks 2>&1 | tail -15`
预期：PASS。

- [ ] **步骤 7：运行 retained 全量测试**

运行：`cargo test -p rustcode-tuix --lib render::retained::tests 2>&1 | tail -10`
预期：全部通过。

- [ ] **步骤 8：提交**

```bash
git add crates/rustcode-tuix/src/render/retained.rs
git commit -m "tuix(retained): mark message boundaries + sync marks on drain"
```

### 任务 2.4：push 时打消息标记（alt-screen）

**文件：**
- 修改：`crates/rustcode-tuix/src/render/alt_screen.rs`

镜像 retained 的逻辑到 alt-screen。

- [ ] **步骤 1：编写失败测试**

```rust
#[test]
fn alt_message_marks_tracked_on_user_push() {
    let mut buf = Vec::new();
    let mut r = AltScreenRenderer::with_writer(&mut buf, caps_default(), 80, 24);
    r.render(UiLine::User("hi".into()));
    assert_eq!(r.message_marks.len(), 1);
    assert_eq!(r.message_marks[0].kind, crate::render::MarkKind::User);
}

#[test]
fn alt_message_marks_decremented_on_drain() {
    let mut buf = Vec::new();
    let mut r = AltScreenRenderer::with_writer(&mut buf, caps_default(), 80, 24);
    for i in 0..5005 {
        r.render(UiLine::User(format!("line {}", i)));
    }
    assert_eq!(r.message_marks.len(), 5000);
    assert_eq!(r.message_marks[0].line_idx, 0);
}
```

- [ ] **步骤 2：确认失败**

运行：`cargo test -p rustcode-tuix --lib alt_message_marks 2>&1 | tail -15`
预期：FAIL。

- [ ] **步骤 3：添加 mark_message 及接线**

在 alt_screen.rs 中镜像阶段 2.3 的做法。同样的 `mark_message` 辅助函数，同样的分支插入。alt_screen.rs 中 drain 的位置在 `push_body_row`（约第 740 行 `body_lines.remove(0)`）；改成：

```rust
while self.body_lines.len() > self.max_scrollback_rows {
    self.body_lines.remove(0);
    self.message_marks.retain(|m| m.line_idx > 0);
    for m in self.message_marks.iter_mut() {
        m.line_idx -= 1;
    }
}
```

（每行一次 `remove(0)` 保留了现有逻辑；drain 语义完全一致。）

同时用相同的 retain+shift 调整该函数底部 `reflow_body_lines` 的 drain 代码块。

- [ ] **步骤 4：确认测试通过**

运行：`cargo test -p rustcode-tuix --lib message_marks 2>&1 | tail -15`
预期：retained 与 alt 两处测试都通过。

- [ ] **步骤 5：提交**

```bash
git add crates/rustcode-tuix/src/render/alt_screen.rs
git commit -m "tuix(alt-screen): mirror MessageMark push + drain sync from retained"
```

---

## 阶段 3：retained view_mode 状态机 + 滚动

### 任务 3.1：添加 view_mode + viewport_top + sticky_bottom 字段

**文件：**
- 修改：`crates/rustcode-tuix/src/render/retained.rs`

- [ ] **步骤 1：添加字段**

在 `RetainedRenderer<W>` 中：

```rust
/// True iff user has scrolled away from the tail. While true, body
/// emit suppresses terminal writes and paint_body redraws from
/// body_lines[viewport_top..] via CUP+EL instead of DECSTBM \n.
view_mode: bool,
/// Top body_lines index visible at body region top, when view_mode = true.
viewport_top: usize,
/// True iff viewport_top >= max_top (auto-tail). Drives view_mode entry/exit.
sticky_bottom: bool,
```

构造函数：`view_mode: false, viewport_top: 0, sticky_bottom: true,`。

- [ ] **步骤 2：构建验证**

运行：`cargo check -p rustcode-tuix`
预期：干净。

- [ ] **步骤 3：提交**

```bash
git add crates/rustcode-tuix/src/render/retained.rs
git commit -m "tuix(retained): add view_mode/viewport_top/sticky_bottom state fields"
```

### 任务 3.2：实现 scroll_body 及相关变体

**文件：**
- 修改：`crates/rustcode-tuix/src/render/retained.rs`

- [ ] **步骤 1：编写失败测试**

```rust
#[test]
fn retained_scroll_up_enters_view_mode() {
    let (mut r, _buf) = new_capturing(80, 24);
    for i in 0..30 {
        r.render(UiLine::User(format!("L{}", i)));
    }
    assert!(r.sticky_bottom);
    assert!(!r.view_mode);
    r.scroll_body(-3);
    assert!(r.view_mode, "scroll up must enter view_mode");
    assert!(!r.sticky_bottom);
}

#[test]
fn retained_scroll_to_bottom_exits_view_mode() {
    let (mut r, _buf) = new_capturing(80, 24);
    for i in 0..30 {
        r.render(UiLine::User(format!("L{}", i)));
    }
    r.scroll_body(-5);
    assert!(r.view_mode);
    r.scroll_body_to_bottom();
    assert!(!r.view_mode);
    assert!(r.sticky_bottom);
}

#[test]
fn retained_scroll_up_then_to_top_lands_at_zero() {
    let (mut r, _buf) = new_capturing(80, 24);
    for i in 0..30 {
        r.render(UiLine::User(format!("L{}", i)));
    }
    r.scroll_body_to_top();
    assert_eq!(r.viewport_top, 0);
    assert!(r.view_mode);
}
```

- [ ] **步骤 2：确认失败**

运行：`cargo test -p rustcode-tuix --lib retained_scroll 2>&1 | tail -15`
预期：FAIL。

- [ ] **步骤 3：实现 scroll_body**

在 `impl<W> Renderer for RetainedRenderer<W>` 内部（找到现有的 trait impl 代码块）添加：

```rust
fn scroll_body(&mut self, delta: i32) {
    let body_height = self.body_bottom_row() as usize;
    let total = self.body_lines.len();
    let max_top = total.saturating_sub(body_height);
    if max_top == 0 {
        // nothing to scroll; stay sticky
        self.sticky_bottom = true;
        self.view_mode = false;
        return;
    }
    let current_top = if self.sticky_bottom { max_top } else { self.viewport_top };
    let new_top: usize = if delta < 0 {
        current_top.saturating_sub(delta.unsigned_abs() as usize)
    } else {
        (current_top + delta as usize).min(max_top)
    };
    self.viewport_top = new_top;
    self.sticky_bottom = new_top >= max_top;
    let was_view = self.view_mode;
    self.view_mode = !self.sticky_bottom;
    // Trigger paint. When transitioning out of view_mode (was_view=true,
    // view_mode=false), the next paint_body must repaint the body tail
    // without a `\n` scroll (handled in Task 3.5).
    if was_view != self.view_mode || self.view_mode {
        // mark body dirty; concrete paint happens via existing render path
        // Use the renderer's standard "redraw body region" mechanism. If
        // retained currently invalidates via a screen-level dirty flag,
        // call that. If it lacks one, force a body repaint here.
        self.repaint_body_region();
    }
}

fn scroll_body_to_top(&mut self) {
    let body_height = self.body_bottom_row() as usize;
    let total = self.body_lines.len();
    if total <= body_height {
        return;
    }
    self.viewport_top = 0;
    self.sticky_bottom = false;
    self.view_mode = true;
    self.repaint_body_region();
}

fn scroll_body_to_bottom(&mut self) {
    let was_view = self.view_mode;
    self.viewport_top = self.body_lines.len().saturating_sub(self.body_bottom_row() as usize);
    self.sticky_bottom = true;
    self.view_mode = false;
    if was_view {
        // Exiting view: repaint body tail without LF (Task 3.5).
        self.repaint_body_region();
    }
}
```

另外添加该辅助函数：

```rust
/// Force a fresh paint of body region rows from body_lines.
/// In view_mode: paint body_lines[viewport_top..viewport_top+body_height].
/// Out of view_mode (just exited): paint body_lines tail.
/// Always uses CUP+EL+content per row; never emits LF.
fn repaint_body_region(&mut self) {
    let bottom = self.body_bottom_row();
    if bottom == 0 || self.body_lines.is_empty() { return; }
    let body_height = bottom as usize;
    let total = self.body_lines.len();
    let start = if self.view_mode {
        self.viewport_top.min(total.saturating_sub(1))
    } else {
        total.saturating_sub(body_height)
    };
    let end = (start + body_height).min(total);
    for (i, row) in self.body_lines[start..end].iter().enumerate() {
        let target_row = 1 + i as u16;
        let seq = format!("\x1b[{};1H\x1b[K", target_row);
        let _ = self.out.write_all(seq.as_bytes());
        let bytes = serialize_row(row);
        let _ = self.out.write_all(&bytes);
    }
    // Clear any rows below content (when body_lines is short).
    for i in (end - start)..body_height {
        let target_row = 1 + i as u16;
        let seq = format!("\x1b[{};1H\x1b[K", target_row);
        let _ = self.out.write_all(seq.as_bytes());
    }
    let _ = self.out.flush();
    // Cursor must return to footer's input row — caller / existing paint
    // chain will re-anchor on next footer paint.
    self.screen.invalidate();
}
```

如果 retained 现有的 `screen` 单元格 diff 缓存让这一步变得复杂，另一种做法是只设置 `self.body_dirty = true`（若存在该字段）并调用现有绘制路径。两者皆可 —— 选一个最贴合 retained.rs 现有写法的。

- [ ] **步骤 4：运行测试**

运行：`cargo test -p rustcode-tuix --lib retained_scroll 2>&1 | tail -15`
预期：PASS。

- [ ] **步骤 5：运行 retained 全量测试**

运行：`cargo test -p rustcode-tuix --lib render::retained::tests 2>&1 | tail -10`
预期：全部通过（无回归）。

- [ ] **步骤 6：提交**

```bash
git add crates/rustcode-tuix/src/render/retained.rs
git commit -m "tuix(retained): implement scroll_body + scroll_body_to_top/bottom"
```

### 任务 3.3：emit_body_line_inner 在 view_mode 下抑制写入

**文件：**
- 修改：`crates/rustcode-tuix/src/render/retained.rs`

- [ ] **步骤 1：编写失败测试**

```rust
#[test]
fn retained_view_mode_suppresses_terminal_writes() {
    let (mut r, buf) = new_capturing(80, 24);
    // Get into view_mode
    for i in 0..30 {
        r.render(UiLine::User(format!("L{}", i)));
    }
    r.scroll_body(-5);
    assert!(r.view_mode);
    let bytes_before = buf.lock().unwrap().len();
    // Push more content; terminal write count should not grow (view paint
    // is idempotent and we already painted in scroll_body).
    r.render(UiLine::User("after view".into()));
    // Snapshot to drop the lock before further mutation
    let new_bytes = buf.lock().unwrap()[bytes_before..].to_vec();
    let s = String::from_utf8_lossy(&new_bytes);
    assert!(!s.contains('\n'), "view_mode must NOT emit \\n scroll: {:?}", s);
    // body_lines should still grow.
    let non_empty = r.body_lines.iter().filter(|row| !row.is_empty()).count();
    assert!(non_empty >= 31, "expected body_lines to keep growing in view_mode, got {}", non_empty);
}
```

- [ ] **步骤 2：确认失败**

运行：`cargo test -p rustcode-tuix --lib retained_view_mode_suppresses 2>&1 | tail -10`
预期：FAIL —— 当前 emit 总会写入。

- [ ] **步骤 3：改造 emit_body_line_inner**

找到 `emit_body_line_inner`（约第 1324 行）。在开头加一个提前返回：

```rust
fn emit_body_line_inner(&mut self, row: &[Cell], bottom: u16) {
    if self.view_mode {
        // In view_mode the body_lines buffer is the source of truth and
        // paint_body repaints from buffer. Don't write to terminal here —
        // we'd overwrite scrolled-away content.
        return;
    }
    // ... existing implementation unchanged
}
```

注意：把该行 push 进 `body_lines` 发生在*调用方*（`push_body_row`），所以该行仍会被缓冲。我们跳过的是写终端。

- [ ] **步骤 4：运行测试**

运行：`cargo test -p rustcode-tuix --lib retained_view_mode_suppresses 2>&1 | tail -10`
预期：PASS。

- [ ] **步骤 5：运行全量测试**

运行：`cargo test -p rustcode-tuix --lib render::retained::tests 2>&1 | tail -10`
预期：全部通过。

- [ ] **步骤 6：提交**

```bash
git add crates/rustcode-tuix/src/render/retained.rs
git commit -m "tuix(retained): suppress terminal writes in emit_body_line_inner while view_mode"
```

### 任务 3.4：在 reset / clear / resize / 审批时强制退出 view_mode

**文件：**
- 修改：`crates/rustcode-tuix/src/render/retained.rs`

- [ ] **步骤 1：编写测试**

```rust
#[test]
fn retained_reset_clears_view_mode() {
    let (mut r, _buf) = new_capturing(80, 24);
    for i in 0..30 { r.render(UiLine::User(format!("L{}", i))); }
    r.scroll_body(-5);
    assert!(r.view_mode);
    r.reset();
    assert!(!r.view_mode);
    assert!(r.sticky_bottom);
}

#[test]
fn retained_resize_clears_view_mode() {
    let (mut r, _buf) = new_capturing(80, 24);
    for i in 0..30 { r.render(UiLine::User(format!("L{}", i))); }
    r.scroll_body(-5);
    assert!(r.view_mode);
    r.on_resize(100, 30);
    assert!(!r.view_mode);
}
```

- [ ] **步骤 2：确认失败**

运行：`cargo test -p rustcode-tuix --lib "retained_reset_clears_view|retained_resize_clears_view" 2>&1 | tail -10`
预期：FAIL。

- [ ] **步骤 3：添加强制退出辅助函数并在 reset/clear/resize/审批处调用**

添加辅助函数：
```rust
fn exit_view_mode(&mut self) {
    if self.view_mode {
        self.view_mode = false;
        self.sticky_bottom = true;
        self.viewport_top = 0;
    }
}
```

在下列函数的开头调用 `self.exit_view_mode();`：
- `fn reset(&mut self)` 
- `fn clear_screen(&mut self)`（如果与 reset 是分开的）
- `fn on_resize(&mut self, ...)`
- `render(UiLine)` 中处理 `UiLine::ApprovalPrompt` 的审批提示分支（或者不管现有变体叫什么 —— 在 retained.rs 里 grep `ApprovalPrompt` 并找到 push 的那个分支）

- [ ] **步骤 4：添加审批提示测试**

```rust
#[test]
fn retained_approval_prompt_forces_view_exit() {
    let (mut r, _buf) = new_capturing(80, 24);
    for i in 0..30 { r.render(UiLine::User(format!("L{}", i))); }
    r.scroll_body(-5);
    assert!(r.view_mode);
    // Push an approval prompt — uses whatever UiLine variant exists.
    r.render(UiLine::ApprovalPrompt {
        // Fill in fields per the actual UiLine::ApprovalPrompt definition;
        // grep `grep -nE "ApprovalPrompt" crates/rustcode-tuix/src/render/mod.rs`
        // to find the exact shape.
        tool: "Bash".into(),
        detail: "ls".into(),
    });
    assert!(!r.view_mode, "approval prompt must force exit from view_mode");
}
```

如果实际的 `UiLine::ApprovalPrompt` 结构不同，按真实字段调整测试。

- [ ] **步骤 5：运行测试**

运行：`cargo test -p rustcode-tuix --lib "retained_reset_clears_view|retained_resize_clears_view|retained_approval_prompt_forces_view" 2>&1 | tail -15`
预期：PASS。

- [ ] **步骤 6：提交**

```bash
git add crates/rustcode-tuix/src/render/retained.rs
git commit -m "tuix(retained): force exit view_mode on reset/clear/resize/approval"
```

---

## 阶段 4：retained 鼠标捕获

### 任务 4.1：启动时发送 ?1002h ?1006h，关闭时发送 ?1002l ?1006l

**文件：**
- 修改：`crates/rustcode-tuix/src/render/retained.rs`

- [ ] **步骤 1：编写失败测试**

```rust
#[test]
fn retained_with_writer_enables_mouse_capture() {
    let mut buf = Vec::new();
    let _r = RetainedRenderer::with_writer(&mut buf, caps_with_color(), 80, 24);
    let s = String::from_utf8_lossy(&buf);
    assert!(s.contains("\x1b[?1002h"), "must enable button-event tracking: {:?}", s);
    assert!(s.contains("\x1b[?1006h"), "must enable SGR coordinates: {:?}", s);
}

#[test]
fn retained_shutdown_disables_mouse_capture() {
    let buf = Arc::new(Mutex::new(Vec::new()));
    let sink = CapturingSink::new(buf.clone());
    let mut r = RetainedRenderer::with_writer(sink, caps_with_color(), 80, 24);
    // clear startup bytes
    buf.lock().unwrap().clear();
    r.shutdown();
    let bytes = buf.lock().unwrap().clone();
    let s = String::from_utf8_lossy(&bytes);
    assert!(s.contains("\x1b[?1002l"), "shutdown must disable button-event: {:?}", s);
    assert!(s.contains("\x1b[?1006l"), "shutdown must disable SGR coords: {:?}", s);
}
```

- [ ] **步骤 2：确认失败**

运行：`cargo test -p rustcode-tuix --lib "retained_with_writer_enables_mouse|retained_shutdown_disables_mouse" 2>&1 | tail -15`
预期：FAIL。

- [ ] **步骤 3：更新 with_writer**

定位 `with_writer` 构造函数（约第 385 行）。找到现有的 `out.write_all(b"\x1b[3J")` 行。改成：

```rust
let _ = out.write_all(b"\x1b[3J\x1b[?1002h\x1b[?1006h");
let _ = out.flush();
```

- [ ] **步骤 4：更新 shutdown**

定位 `fn shutdown(&mut self)`（搜索：`grep -nE "fn shutdown" crates/rustcode-tuix/src/render/retained.rs`）。在开头（或现有清理逻辑所在处）前置：

```rust
let _ = self.out.write_all(b"\x1b[?1006l\x1b[?1002l");
let _ = self.out.flush();
```

- [ ] **步骤 5：更新 Drop 实现**

找到 `impl<W> Drop for RetainedRenderer<W>`（若存在；否则只在 shutdown 里加）。在 Drop 中镜像同样的关闭序列，作为 panic 路径下的双保险。

- [ ] **步骤 6：运行测试**

运行：`cargo test -p rustcode-tuix --lib "retained_with_writer_enables_mouse|retained_shutdown_disables_mouse" 2>&1 | tail -10`
预期：PASS。

- [ ] **步骤 7：运行 retained 全量测试**

运行：`cargo test -p rustcode-tuix --lib render::retained::tests 2>&1 | tail -10`
预期：全部通过。

- [ ] **步骤 8：提交**

```bash
git add crates/rustcode-tuix/src/render/retained.rs
git commit -m "tuix(retained): enable button-event + SGR mouse capture at startup"
```

### 任务 4.2：为外部子进程挂起/恢复鼠标捕获

**文件：**
- 修改：`crates/rustcode-tuix/src/render/retained.rs`

- [ ] **步骤 1：编写失败测试**

```rust
#[test]
fn retained_suspend_disables_mouse_capture() {
    let buf = Arc::new(Mutex::new(Vec::new()));
    let sink = CapturingSink::new(buf.clone());
    let mut r = RetainedRenderer::with_writer(sink, caps_with_color(), 80, 24);
    buf.lock().unwrap().clear();
    r.suspend_for_external();
    let s = String::from_utf8_lossy(&buf.lock().unwrap());
    assert!(s.contains("\x1b[?1006l"), "suspend must disable SGR: {:?}", s);
    assert!(s.contains("\x1b[?1002l"), "suspend must disable button-event: {:?}", s);
}

#[test]
fn retained_resume_reenables_mouse_capture() {
    let buf = Arc::new(Mutex::new(Vec::new()));
    let sink = CapturingSink::new(buf.clone());
    let mut r = RetainedRenderer::with_writer(sink, caps_with_color(), 80, 24);
    r.suspend_for_external();
    buf.lock().unwrap().clear();
    r.resume_from_external();
    let s = String::from_utf8_lossy(&buf.lock().unwrap());
    assert!(s.contains("\x1b[?1002h"), "resume must re-enable button-event: {:?}", s);
    assert!(s.contains("\x1b[?1006h"), "resume must re-enable SGR: {:?}", s);
}
```

- [ ] **步骤 2：确认失败**

运行：`cargo test -p rustcode-tuix --lib "retained_suspend_disables_mouse|retained_resume_reenables_mouse" 2>&1 | tail -10`
预期：FAIL。

- [ ] **步骤 3：实现**

在 `suspend_for_external`（约第 2985 行）中找到现有的清理代码块（raw_mode、bracketed paste、Kitty 增强）。在其前面加一次关闭鼠标的写入：

```rust
let _ = self.out.write_all(b"\x1b[?1006l\x1b[?1002l");
// ... existing code follows
```

在 `resume_from_external` 中，在现有的重新启用代码块之后追加：

```rust
let _ = self.out.write_all(b"\x1b[?1002h\x1b[?1006h");
let _ = self.out.flush();
```

- [ ] **步骤 4：运行测试**

运行：`cargo test -p rustcode-tuix --lib "retained_suspend_disables_mouse|retained_resume_reenables_mouse" 2>&1 | tail -10`
预期：PASS。

- [ ] **步骤 5：提交**

```bash
git add crates/rustcode-tuix/src/render/retained.rs
git commit -m "tuix(retained): pop/repush mouse capture in suspend/resume_for_external"
```

### 任务 4.3：Windows conhost 鼠标捕获对齐

**文件：**
- 修改：`crates/rustcode-tuix/src/render/retained.rs`

复用 alt-screen 的 `enable_conhost_mouse_capture()` 和 `restore_conhost_console_in_mode()`。

- [ ] **步骤 1：定位 alt-screen 的 Windows 辅助函数**

运行：
```bash
grep -nE "fn enable_conhost_mouse_capture|fn restore_conhost_console_in_mode|prior_console_in_mode" crates/rustcode-tuix/src/render/alt_screen.rs | head -5
```
预期：定义在 alt_screen.rs 中。

- [ ] **步骤 2：把辅助函数上提到仅 Windows 的模块**

创建 `crates/rustcode-tuix/src/render/conhost.rs`（仅 Windows）：

```rust
//! Windows conhost mouse capture helpers, used by both AltScreenRenderer
//! and RetainedRenderer to set/clear `ENABLE_MOUSE_INPUT` while
//! preserving the pre-enter console mode.

#![cfg(windows)]

// Move the existing enable_conhost_mouse_capture + restore_conhost_console_in_mode
// + any associated constants from alt_screen.rs verbatim. Mark them `pub`.

pub fn enable_conhost_mouse_capture() -> Option<u32> {
    // ... existing alt_screen.rs body ...
}

pub fn restore_conhost_console_in_mode(prior: u32) {
    // ... existing alt_screen.rs body ...
}
```

在 `crates/rustcode-tuix/src/render/mod.rs` 中声明模块：

```rust
#[cfg(windows)]
pub mod conhost;
```

- [ ] **步骤 3：更新 alt_screen.rs 以使用共享模块**

把本地调用替换为 `crate::render::conhost::enable_conhost_mouse_capture()` 等。删除本地定义。

- [ ] **步骤 4：在 retained.rs 中添加 Windows 字段与调用**

在 `RetainedRenderer<W>` 中：

```rust
#[cfg(windows)]
prior_console_in_mode: Option<u32>,
```

构造函数：`#[cfg(windows)] prior_console_in_mode: None,`。

在 `with_writer` 中，在写入 `\x1b[3J\x1b[?1002h\x1b[?1006h` 之后：

```rust
#[cfg(windows)]
let prior_console_in_mode = crate::render::conhost::enable_conhost_mouse_capture();
```

在 `Self { ... #[cfg(windows)] prior_console_in_mode, ... }` 中设置该字段。

在 `suspend_for_external` 中：

```rust
#[cfg(windows)]
if let Some(prior) = self.prior_console_in_mode.take() {
    crate::render::conhost::restore_conhost_console_in_mode(prior);
}
```

在 `resume_from_external` 中：

```rust
#[cfg(windows)] {
    self.prior_console_in_mode = crate::render::conhost::enable_conhost_mouse_capture();
}
```

在 `shutdown` 与 Drop 中：

```rust
#[cfg(windows)]
if let Some(prior) = self.prior_console_in_mode.take() {
    crate::render::conhost::restore_conhost_console_in_mode(prior);
}
```

- [ ] **步骤 5：跨平台构建检查**

运行：`cargo check -p rustcode-tuix`
预期：在 macOS/Linux 上干净（`#[cfg(windows)]` 代码块会被编译掉）。

如果你有 Windows 环境，也运行 `cargo check --target x86_64-pc-windows-msvc -p rustcode-tuix`（或等价命令）。

- [ ] **步骤 6：运行 alt-screen 与 retained 全量测试**

运行：`cargo test -p rustcode-tuix --lib 2>&1 | tail -10`
预期：全部通过。

- [ ] **步骤 7：提交**

```bash
git add crates/rustcode-tuix/src/render/{mod.rs,conhost.rs,alt_screen.rs,retained.rs}
git commit -m "tuix: hoist conhost mouse-capture helpers into shared module; retained uses them"
```

---

## 阶段 5：retained 选择功能接线

### 任务 5.1：为 Vec<Vec<Cell>> 实现 BodyLineView

**文件：**
- 修改：`crates/rustcode-tuix/src/render/selection.rs`

- [ ] **步骤 1：添加该 impl**

在 `selection.rs` 中：

```rust
use crate::render::cell::Cell;

impl BodyLineView for Vec<Vec<Cell>> {
    fn line_count(&self) -> usize { self.len() }
    fn line_text(&self, idx: usize) -> Cow<'_, str> {
        let Some(row) = self.get(idx) else { return Cow::Borrowed(""); };
        // Build a visible-text string from cells; skip continuation cells
        // (width == 0) which are placeholders for the 2nd column of a wide glyph.
        let s: String = row.iter().filter(|c| c.width > 0).map(|c| c.ch).collect();
        Cow::Owned(s)
    }
}
```

- [ ] **步骤 2：添加测试**

```rust
#[cfg(test)]
mod cell_view_tests {
    use super::*;
    use crate::render::cell::{Cell, CellStyle};

    #[test]
    fn vec_vec_cell_line_text_extracts_visible_chars() {
        let row = vec![
            Cell { ch: 'h', style: CellStyle::default(), width: 1 },
            Cell { ch: 'i', style: CellStyle::default(), width: 1 },
            Cell { ch: '中', style: CellStyle::default(), width: 2 },
            Cell { ch: ' ', style: CellStyle::default(), width: 0 }, // continuation
        ];
        let body: Vec<Vec<Cell>> = vec![row];
        assert_eq!(body.line_text(0), "hi中");
    }
}
```

- [ ] **步骤 3：运行测试**

运行：`cargo test -p rustcode-tuix --lib render::selection 2>&1 | tail -10`
预期：PASS。

- [ ] **步骤 4：提交**

```bash
git add crates/rustcode-tuix/src/render/selection.rs
git commit -m "tuix(selection): impl BodyLineView for Vec<Vec<Cell>> (retained body type)"
```

### 任务 5.2：为 retained 添加 SelectionState 字段与 trait 方法

**文件：**
- 修改：`crates/rustcode-tuix/src/render/retained.rs`

- [ ] **步骤 1：编写失败测试**

```rust
#[test]
fn retained_begin_selection_records_anchor() {
    let (mut r, _buf) = new_capturing(80, 24);
    for i in 0..5 { r.render(UiLine::User(format!("L{}", i))); }
    r.begin_selection(3, 1);
    assert!(r.selection.selection.is_some());
}

#[test]
fn retained_copy_selection_writes_clipboard() {
    let (mut r, _buf) = new_capturing(80, 24);
    r.render(UiLine::User("hello world".into()));
    // Anchor at body row 0 col 0, head at col 5.
    r.selection.begin((0, 0));
    r.selection.update((0, 5));
    assert!(r.copy_selection(), "expected non-empty selection to copy");
}
```

- [ ] **步骤 2：确认失败**

运行：`cargo test -p rustcode-tuix --lib "retained_begin_selection_records|retained_copy_selection_writes" 2>&1 | tail -10`
预期：FAIL。

- [ ] **步骤 3：添加字段**

在 `RetainedRenderer<W>` 中：

```rust
selection: crate::render::selection::SelectionState,
```

构造函数：`selection: Default::default(),`。

- [ ] **步骤 4：实现 trait 方法**

在 `impl<W> Renderer for RetainedRenderer<W>` 中：

```rust
fn begin_selection(&mut self, col: u16, row: u16) {
    if let Some(pos) = self.screen_to_body(col, row) {
        self.selection.begin(pos);
        self.repaint_body_region();
    } else {
        self.selection.clear();
    }
}

fn update_selection(&mut self, col: u16, row: u16) {
    if let Some(pos) = self.screen_to_body(col, row) {
        self.selection.update(pos);
        self.repaint_body_region();
    }
}

fn end_selection(&mut self) {
    if let Some(text) = self.selection.end(&self.body_lines) {
        crate::render::selection::emit_osc52(&mut self.out, &text);
    }
}

fn copy_selection(&mut self) -> bool {
    let copied = self.selection.copy(&self.body_lines);
    if copied { self.repaint_body_region(); }
    copied
}
```

- [ ] **步骤 5：添加 screen_to_body 辅助函数**

为 retained 镜像 alt-screen 的 `fn screen_to_body(&self, col: u16, row: u16) -> Option<(usize, u16)>`。在 retained 的场景下，body 区域是 `1..=body_bottom_row()` 这些行。该函数把屏幕坐标转换成 body_lines 下标。若处于 `view_mode`，下标是 `viewport_top + (row - 1)`；否则是相对尾部的下标。

```rust
fn screen_to_body(&self, col: u16, row: u16) -> Option<(usize, u16)> {
    let bottom = self.body_bottom_row();
    if row == 0 || row > bottom { return None; }
    let body_height = bottom as usize;
    let total = self.body_lines.len();
    let viewport_start = if self.view_mode {
        self.viewport_top
    } else {
        total.saturating_sub(body_height)
    };
    let body_row = viewport_start + (row - 1) as usize;
    if body_row >= total { return None; }
    Some((body_row, col))
}
```

- [ ] **步骤 6：运行测试**

运行：`cargo test -p rustcode-tuix --lib "retained_begin_selection_records|retained_copy_selection_writes" 2>&1 | tail -10`
预期：PASS。

- [ ] **步骤 7：提交**

```bash
git add crates/rustcode-tuix/src/render/retained.rs
git commit -m "tuix(retained): wire SelectionState begin/update/end/copy via trait"
```

### 任务 5.3：在 retained 的 paint_body 中应用选择高亮

**文件：**
- 修改：`crates/rustcode-tuix/src/render/retained.rs`

退出 view_mode 后 `repaint_body_region` 当前没有应用 selection 高亮。要让选中范围用反色显示。

- [ ] **步骤 1：更新 repaint_body_region**

修改 `repaint_body_region` 的函数体，逐行应用选择高亮：

```rust
fn repaint_body_region(&mut self) {
    let bottom = self.body_bottom_row();
    if bottom == 0 || self.body_lines.is_empty() { return; }
    let body_height = bottom as usize;
    let total = self.body_lines.len();
    let start = if self.view_mode {
        self.viewport_top.min(total.saturating_sub(1))
    } else {
        total.saturating_sub(body_height)
    };
    let end = (start + body_height).min(total);
    use crate::render::selection::{selection_col_range_for_line};
    let sel = self.selection.selection.map(|s| (
        if s.anchor < s.head { (s.anchor, s.head) } else { (s.head, s.anchor) }
    ));
    for (i, row) in self.body_lines[start..end].iter().enumerate() {
        let target_row = 1 + i as u16;
        let seq = format!("\x1b[{};1H\x1b[K", target_row);
        let _ = self.out.write_all(seq.as_bytes());
        let body_idx = start + i;
        let row_text: String = row.iter().filter(|c| c.width > 0).map(|c| c.ch).collect();
        // If selection covers this row, emit with reverse-video using
        // the shared helper. Otherwise emit normally via serialize_row.
        let sel_range = sel.and_then(|(lo, hi)| selection_col_range_for_line(body_idx, lo, hi, &row_text));
        if let Some((sel_start, sel_end)) = sel_range {
            let highlighted = crate::render::selection::render_line_with_selection(&row_text, self.screen.width(), sel_start, sel_end);
            let _ = self.out.write_all(highlighted.as_bytes());
        } else {
            let bytes = serialize_row(row);
            let _ = self.out.write_all(&bytes);
        }
    }
    for i in (end - start)..body_height {
        let target_row = 1 + i as u16;
        let seq = format!("\x1b[{};1H\x1b[K", target_row);
        let _ = self.out.write_all(seq.as_bytes());
    }
    let _ = self.out.flush();
    self.screen.invalidate();
}
```

- [ ] **步骤 2：添加测试**

```rust
#[test]
fn retained_selection_highlight_emits_reverse_video() {
    let (mut r, buf) = new_capturing(80, 24);
    r.render(UiLine::User("hello world".into()));
    buf.lock().unwrap().clear();
    // Force view_mode so repaint_body_region path executes
    r.scroll_body(-1);
    r.selection.begin((0, 0));
    r.selection.update((0, 5));
    r.repaint_body_region();
    let s = String::from_utf8_lossy(&buf.lock().unwrap());
    assert!(s.contains("\x1b[7m"), "selection paint must include reverse-video SGR: {:?}", s);
}
```

- [ ] **步骤 3：运行测试**

运行：`cargo test -p rustcode-tuix --lib retained_selection_highlight 2>&1 | tail -10`
预期：PASS。

- [ ] **步骤 4：提交**

```bash
git add crates/rustcode-tuix/src/render/retained.rs
git commit -m "tuix(retained): apply selection highlight via shared render_line_with_selection"
```

---

## 阶段 6：滚动条 + /scrollbar 命令 + ui-state.toml

### 任务 6.1：创建 ui_state.rs 持久化

**文件：**
- 新建：`crates/rustcode-tuix/src/render/ui_state.rs`
- 修改：`crates/rustcode-tuix/src/render/mod.rs`

- [ ] **步骤 1：编写失败测试**

创建 `crates/rustcode-tuix/src/render/ui_state.rs`：

```rust
//! UI state persisted between sessions. Currently: scrollbar visibility.
//! Stored at `$RUSTCODE_HOME/ui-state.toml`. Load/save are best-effort —
//! missing file or parse error returns default (everything false).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiState {
    #[serde(default)]
    pub ui: UiSection,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiSection {
    #[serde(default)]
    pub show_scrollbar: bool,
}

fn ui_state_path() -> Option<PathBuf> {
    let home = std::env::var_os("RUSTCODE_HOME")
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|h| h.join(".rustcode")))?;
    Some(home.join("ui-state.toml"))
}

pub fn load() -> UiState {
    let Some(path) = ui_state_path() else { return UiState::default(); };
    let Ok(text) = std::fs::read_to_string(&path) else { return UiState::default(); };
    toml::from_str(&text).unwrap_or_default()
}

pub fn save(state: &UiState) {
    let Some(path) = ui_state_path() else { return; };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let Ok(text) = toml::to_string(state) else {
        crate::tuix_trace!("UI", "ui-state serialize failed");
        return;
    };
    if let Err(e) = std::fs::write(&path, text) {
        crate::tuix_trace!("UI", "ui-state write failed: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    #[test]
    fn ui_state_round_trip_via_rustcode_home() {
        let td = TempDir::new().unwrap();
        env::set_var("RUSTCODE_HOME", td.path());
        let mut s = UiState::default();
        s.ui.show_scrollbar = true;
        save(&s);
        let loaded = load();
        assert!(loaded.ui.show_scrollbar);
    }

    #[test]
    fn ui_state_missing_file_returns_default() {
        let td = TempDir::new().unwrap();
        env::set_var("RUSTCODE_HOME", td.path());
        let loaded = load();
        assert!(!loaded.ui.show_scrollbar);
    }
}
```

在 `render/mod.rs` 中声明：
```rust
pub mod ui_state;
```

检查 `Cargo.toml` 里是否有 `tempfile` 这个 dev-dep —— 多半已经有了；如果没有，把 `tempfile = "3"` 加到 `[dev-dependencies]`。

- [ ] **步骤 2：确认依赖**

运行：`grep -nE "^(toml|dirs|serde)" crates/rustcode-tuix/Cargo.toml`
预期：全部存在（serde + toml 用得很普遍；dirs 多半也有）。缺哪个就补哪个。

- [ ] **步骤 3：运行测试**

运行：`cargo test -p rustcode-tuix --lib render::ui_state 2>&1 | tail -10`
预期：PASS。

- [ ] **步骤 4：提交**

```bash
git add crates/rustcode-tuix/src/render/{mod.rs,ui_state.rs} crates/rustcode-tuix/Cargo.toml
git commit -m "tuix(ui_state): persist UI prefs to \$RUSTCODE_HOME/ui-state.toml"
```

### 任务 6.2：创建 scrollbar.rs 辅助模块

**文件：**
- 新建：`crates/rustcode-tuix/src/render/scrollbar.rs`
- 修改：`crates/rustcode-tuix/src/render/mod.rs`

- [ ] **步骤 1：编写失败测试与模块骨架**

创建 `crates/rustcode-tuix/src/render/scrollbar.rs`：

```rust
//! Pure compute for the right-edge scrollbar. Both renderers call into
//! `compute()` to decide thumb shape, then call `paint_row(...)` to emit
//! a single column's worth of cells per body row.

/// Vertical thumb position + height in body-region coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrollbarShape {
    pub thumb_top: usize,     // 0-indexed body row
    pub thumb_height: usize,
}

/// Returns None when no thumb should be drawn (no overflow or disabled).
pub fn compute(
    total: usize,
    visible: usize,
    viewport_top: usize,
    sticky_bottom: bool,
    show: bool,
) -> Option<ScrollbarShape> {
    if !show || total <= visible || visible == 0 {
        return None;
    }
    let max_top = total - visible;
    let effective_top = if sticky_bottom { max_top } else { viewport_top };
    let thumb_h = ((visible * visible) / total).max(1);
    let track_avail = visible.saturating_sub(thumb_h);
    let thumb_top = if max_top == 0 {
        0
    } else {
        effective_top * track_avail / max_top
    };
    Some(ScrollbarShape { thumb_top, thumb_height: thumb_h })
}

/// Whether the given body row index (0..visible) should paint a thumb char.
pub fn is_thumb_row(shape: &ScrollbarShape, body_row: usize) -> bool {
    body_row >= shape.thumb_top && body_row < shape.thumb_top + shape.thumb_height
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_returns_none_when_no_overflow() {
        assert!(compute(10, 20, 0, true, true).is_none());
        assert!(compute(20, 20, 0, true, true).is_none());
    }

    #[test]
    fn compute_returns_none_when_disabled() {
        assert!(compute(50, 10, 5, false, false).is_none());
    }

    #[test]
    fn compute_thumb_height_proportional_to_visible_over_total() {
        let s = compute(30, 10, 0, true, true).unwrap();
        // 10 * 10 / 30 = 3
        assert_eq!(s.thumb_height, 3);
    }

    #[test]
    fn compute_thumb_at_bottom_when_sticky() {
        let s = compute(30, 10, 0, true, true).unwrap();
        // sticky_bottom => effective_top = max_top = 20
        // thumb_top = 20 * (10 - 3) / 20 = 7
        assert_eq!(s.thumb_top, 7);
    }

    #[test]
    fn compute_thumb_at_top_when_viewport_top_zero() {
        let s = compute(30, 10, 0, false, true).unwrap();
        assert_eq!(s.thumb_top, 0);
    }

    #[test]
    fn is_thumb_row_covers_thumb_range() {
        let shape = ScrollbarShape { thumb_top: 3, thumb_height: 4 };
        assert!(!is_thumb_row(&shape, 2));
        assert!(is_thumb_row(&shape, 3));
        assert!(is_thumb_row(&shape, 6));
        assert!(!is_thumb_row(&shape, 7));
    }
}
```

在 `render/mod.rs` 中声明：
```rust
pub mod scrollbar;
```

- [ ] **步骤 2：运行测试**

运行：`cargo test -p rustcode-tuix --lib render::scrollbar 2>&1 | tail -15`
预期：PASS（6 个测试）。

- [ ] **步骤 3：提交**

```bash
git add crates/rustcode-tuix/src/render/{mod.rs,scrollbar.rs}
git commit -m "tuix(scrollbar): add pure compute module for thumb shape + placement"
```

### 任务 6.3：添加 show_scrollbar 字段与 toggle_scrollbar trait 方法

**文件：**
- 修改：`crates/rustcode-tuix/src/render/mod.rs`
- 修改：`crates/rustcode-tuix/src/render/retained.rs`
- 修改：`crates/rustcode-tuix/src/render/alt_screen.rs`
- 修改：`crates/rustcode-tuix/src/render/plain.rs`
- 修改：`crates/rustcode-tuix/src/render/worker.rs`

- [ ] **步骤 1：添加 trait 方法**

在 `render/mod.rs` 的 `Renderer` trait 中：

```rust
/// Toggle the right-side visible scrollbar. Default: no-op for renderers
/// that don't have a body region (Plain).
fn toggle_scrollbar(&mut self) -> bool { false }
```

（返回新状态 —— true 表示现在显示。）

- [ ] **步骤 2：在 alt-screen 中添加字段与实现**

在 `AltScreenRenderer` 中：

```rust
show_scrollbar: bool,
```

构造函数：从 `ui_state::load().ui.show_scrollbar` 读取。覆盖该 trait 方法：

```rust
fn toggle_scrollbar(&mut self) -> bool {
    self.show_scrollbar = !self.show_scrollbar;
    let mut state = crate::render::ui_state::load();
    state.ui.show_scrollbar = self.show_scrollbar;
    crate::render::ui_state::save(&state);
    // Body width changes — force reflow + repaint
    self.reflow_body_lines();
    self.body_dirty = true;
    self.paint_frame();
    self.show_scrollbar
}
```

- [ ] **步骤 3：在 retained 中添加字段与实现**

在 `RetainedRenderer<W>` 中：

```rust
show_scrollbar: bool,
```

构造函数同样读取。trait 实现：

```rust
fn toggle_scrollbar(&mut self) -> bool {
    self.show_scrollbar = !self.show_scrollbar;
    let mut state = crate::render::ui_state::load();
    state.ui.show_scrollbar = self.show_scrollbar;
    crate::render::ui_state::save(&state);
    // Body width changes — force repaint of body region tail
    self.repaint_body_region();
    self.show_scrollbar
}
```

- [ ] **步骤 4：经 worker 转发**

在 `crates/rustcode-tuix/src/render/worker.rs` 中添加一个 `RenderCmd` 变体 + AckOp（需要返回值，所以用 AckOp 模式）：

```rust
pub enum AckOp {
    // ... existing variants ...
    ToggleScrollbar,
}
```

在 `run_worker` 中处理：

```rust
AckOp::ToggleScrollbar => {
    let _ = inner.toggle_scrollbar();
    // No ack value needed; the toggle outcome is rendered visually + persisted
}
```

在 `TaskRenderer` 中：

```rust
fn toggle_scrollbar(&mut self) -> bool {
    self.ack(AckOp::ToggleScrollbar);
    // Return is best-effort; if the caller needs the new state, it can
    // read from ui_state::load() after this returns.
    crate::render::ui_state::load().ui.show_scrollbar
}
```

（如果现有的 `ack()` 模式是用 channel 通知完成的，确保新变体以同样的方式接通。）

- [ ] **步骤 5：构建**

运行：`cargo check -p rustcode-tuix`
预期：干净。

- [ ] **步骤 6：提交**

```bash
git add crates/rustcode-tuix/src/render/{mod.rs,retained.rs,alt_screen.rs,plain.rs,worker.rs}
git commit -m "tuix(scrollbar): add show_scrollbar field + toggle_scrollbar trait method"
```

### 任务 6.4：在 alt-screen 的 paint_body 中应用滚动条

**文件：**
- 修改：`crates/rustcode-tuix/src/render/alt_screen.rs`

- [ ] **步骤 1：编写失败测试**

```rust
#[test]
fn alt_scrollbar_paints_thumb_when_enabled_and_overflow() {
    let mut buf = Vec::new();
    let mut r = AltScreenRenderer::with_writer(&mut buf, caps_default(), 80, 10);
    r.show_scrollbar = true;
    for i in 0..30 { r.push_body_row(format!("R{:02}", i)); }
    r.paint_body();
    drop(r);
    let s = String::from_utf8_lossy(&buf);
    assert!(s.contains("█"), "thumb char missing: {:?}", s);
}

#[test]
fn alt_scrollbar_not_painted_when_disabled() {
    let mut buf = Vec::new();
    let mut r = AltScreenRenderer::with_writer(&mut buf, caps_default(), 80, 10);
    r.show_scrollbar = false;
    for i in 0..30 { r.push_body_row(format!("R{:02}", i)); }
    r.paint_body();
    drop(r);
    let s = String::from_utf8_lossy(&buf);
    assert!(!s.contains("█"), "thumb should not appear when disabled: {:?}", s);
}
```

- [ ] **步骤 2：确认失败**

运行：`cargo test -p rustcode-tuix --lib "alt_scrollbar_paints_thumb|alt_scrollbar_not_painted" 2>&1 | tail -10`
预期：FAIL。

- [ ] **步骤 3：更新 paint_body**

在 `paint_body` 中（alt_screen.rs 约第 800 行），在现有的逐行绘制之后添加滚动条列：

```rust
let scrollbar_shape = crate::render::scrollbar::compute(
    self.body_lines.len(),
    body_height,
    viewport_start,
    self.sticky_bottom,
    self.show_scrollbar,
);
if let Some(shape) = &scrollbar_shape {
    let scrollbar_col = self.width;  // 1-indexed rightmost
    for row_idx in 0..body_height {
        let target_row = 1 + row_idx as u16;
        let glyph = if crate::render::scrollbar::is_thumb_row(shape, row_idx) { "█" } else { "│" };
        let seq = format!("\x1b[{};{}H{}", target_row, scrollbar_col, glyph);
        let _ = self.out.write_all(seq.as_bytes());
    }
}
```

如果 body 内容目前会写到 `width` 列，那么当 `scrollbar_shape.is_some()` 时还需要把 body 行绘制限制到 `width - 1`，避免覆盖。最省事的做法：当 `show_scrollbar = true` 且存在溢出时，逐行内容 emit 在 `width - 1` 处截断（若有 `truncate_to_width` 辅助函数就用它，否则直接切片）。在现有的行绘制循环里加上这个条件限制。

- [ ] **步骤 4：运行测试**

运行：`cargo test -p rustcode-tuix --lib "alt_scrollbar_paints_thumb|alt_scrollbar_not_painted" 2>&1 | tail -10`
预期：PASS。

- [ ] **步骤 5：运行 alt-screen 全量测试**

运行：`cargo test -p rustcode-tuix --lib render::alt_screen::tests 2>&1 | tail -10`
预期：全部通过。

- [ ] **步骤 6：提交**

```bash
git add crates/rustcode-tuix/src/render/alt_screen.rs
git commit -m "tuix(alt-screen): paint right-side scrollbar when overflow + show_scrollbar"
```

### 任务 6.5：在 retained 的绘制中应用滚动条

**文件：**
- 修改：`crates/rustcode-tuix/src/render/retained.rs`

- [ ] **步骤 1：编写失败测试**

```rust
#[test]
fn retained_scrollbar_paints_when_enabled_in_view_mode() {
    let (mut r, buf) = new_capturing(80, 10);
    r.show_scrollbar = true;
    for i in 0..30 { r.render(UiLine::User(format!("R{:02}", i))); }
    r.scroll_body(-3);  // enter view_mode + repaint_body_region
    let s = String::from_utf8_lossy(&buf.lock().unwrap());
    assert!(s.contains("█"), "thumb missing in view paint: {:?}", s);
}
```

- [ ] **步骤 2：确认失败**

运行：`cargo test -p rustcode-tuix --lib retained_scrollbar_paints 2>&1 | tail -10`
预期：FAIL。

- [ ] **步骤 3：更新 repaint_body_region**

在 `repaint_body_region` 中，在逐行绘制循环之后镜像 alt-screen 的滚动条绘制：

```rust
let scrollbar_shape = crate::render::scrollbar::compute(
    total,
    body_height,
    if self.view_mode { self.viewport_top } else { total.saturating_sub(body_height) },
    self.sticky_bottom,
    self.show_scrollbar,
);
if let Some(shape) = &scrollbar_shape {
    let scrollbar_col = self.screen.width();
    for row_idx in 0..body_height {
        let target_row = 1 + row_idx as u16;
        let glyph = if crate::render::scrollbar::is_thumb_row(shape, row_idx) { "█" } else { "│" };
        let seq = format!("\x1b[{};{}H{}", target_row, scrollbar_col, glyph);
        let _ = self.out.write_all(seq.as_bytes());
    }
}
```

另外：当滚动条可见时，在 `emit_body_line_inner` 路径（sticky 模式）中把行内容截断到 `width - 1`，避免与滚动条列重叠。最简单的做法：在 `serialize_row` 时做限制；或者在序列化之前先截断该行。

- [ ] **步骤 4：运行测试**

运行：`cargo test -p rustcode-tuix --lib retained_scrollbar_paints 2>&1 | tail -10`
预期：PASS。

- [ ] **步骤 5：运行 retained 全量测试**

运行：`cargo test -p rustcode-tuix --lib render::retained::tests 2>&1 | tail -10`
预期：全部通过。

- [ ] **步骤 6：提交**

```bash
git add crates/rustcode-tuix/src/render/retained.rs
git commit -m "tuix(retained): paint right-side scrollbar in repaint_body_region"
```

### 任务 6.6：注册 /scrollbar 斜杠命令

**文件：**
- 修改：`crates/rustcode-tuix/src/commands.rs`
- 修改：`crates/rustcode-tuix/src/event_loop/commands.rs`

- [ ] **步骤 1：注册命令**

在 `commands.rs` 的 BUILT_INS 数组中添加（放在 `keys` / `help` 附近）：

```rust
Command { name: "scrollbar", desc: "Toggle the right-side scrollbar", needs_args: false },
```

在 `cmd_desc_i18n` 分支中：

```rust
"scrollbar" => Msg::CmdDescScrollbar,
```

- [ ] **步骤 2：处理命令**

在 `event_loop/commands.rs` 中，在斜杠命令分发处添加一个分支（放在 `keys` 附近）：

```rust
"scrollbar" => {
    let now_on = renderer.toggle_scrollbar();
    renderer.render(UiLine::CommandOutput(
        t(if now_on { Msg::ScrollbarOn } else { Msg::ScrollbarOff }).into_owned(),
    ));
    renderer.flush();
}
```

- [ ] **步骤 3：构建**

运行：`cargo build -p rustcode-tuix 2>&1 | tail -10`
预期：干净。

- [ ] **步骤 4：添加命令注册测试**

在 `commands.rs` 的 `#[cfg(test)] mod tests` 中：

```rust
#[test]
fn scrollbar_command_registered_with_i18n_description_in_both_locales() {
    use crate::i18n::Locale;
    assert!(BUILT_INS.iter().any(|c| c.name == "scrollbar"));
    for locale in [Locale::EnUs, Locale::ZhCn] {
        crate::i18n::set_locale(locale);
        let desc = cmd_desc_i18n("scrollbar").expect("CmdDescScrollbar translation");
        assert!(!desc.is_empty(), "CmdDescScrollbar ({locale:?}) must not be empty");
    }
}
```

运行：`cargo test -p rustcode-tuix --lib scrollbar_command_registered 2>&1 | tail -10`
预期：PASS。

- [ ] **步骤 5：提交**

```bash
git add crates/rustcode-tuix/src/{commands.rs,event_loop/commands.rs}
git commit -m "tuix(commands): register /scrollbar + i18n description"
```

---

## 阶段 7：附加滚动按键（Alt+↑/↓、Ctrl+↑/↓）

### 任务 7.1：添加 scroll_to_prev_message / scroll_to_next_message

**文件：**
- 修改：`crates/rustcode-tuix/src/render/mod.rs`
- 修改：`crates/rustcode-tuix/src/render/alt_screen.rs`
- 修改：`crates/rustcode-tuix/src/render/retained.rs`
- 修改：`crates/rustcode-tuix/src/render/worker.rs`

- [ ] **步骤 1：添加 trait 方法**

在 `Renderer` trait 中：

```rust
/// Jump body viewport to the prev/next message boundary. No-op when no
/// such boundary exists in the configured direction.
fn scroll_to_prev_message(&mut self) {}
fn scroll_to_next_message(&mut self) {}
fn scroll_to_prev_user_message(&mut self) {}
fn scroll_to_next_user_message(&mut self) {}
```

- [ ] **步骤 2：编写测试（alt-screen）**

```rust
#[test]
fn alt_scroll_to_prev_message_finds_nearest_above() {
    let mut buf = Vec::new();
    let mut r = AltScreenRenderer::with_writer(&mut buf, caps_default(), 80, 10);
    // Populate body with: 5 user lines, 5 assistant lines, 5 tool lines.
    for i in 0..5 { r.render(UiLine::User(format!("u{}", i))); }
    for i in 0..5 { r.render(UiLine::AssistantText(format!("a{}", i))); }
    for i in 0..5 { r.render(UiLine::ToolCall { /* fill required fields */ }); }
    // Scroll to viewport_top = 10 (toolcall area).
    r.scroll_body(-100);  // top
    r.scroll_body(10);     // back down 10
    let before = r.viewport_top;
    r.scroll_to_prev_message();
    assert!(r.viewport_top < before, "viewport should jump up to prev message");
}
```

（按真实结构填写 `UiLine::ToolCall { ... }` —— 用 `grep -nE "enum UiLine" crates/rustcode-tuix/src/render/mod.rs` 找到它。）

- [ ] **步骤 3：在 alt-screen 上实现**

```rust
fn scroll_to_prev_message(&mut self) {
    let target = self.message_marks.iter().rev().find(|m| m.line_idx < self.viewport_top);
    if let Some(target) = target {
        self.scroll_body_to(target.line_idx);
    }
}

fn scroll_to_next_message(&mut self) {
    let target = self.message_marks.iter().find(|m| m.line_idx > self.viewport_top);
    if let Some(target) = target {
        self.scroll_body_to(target.line_idx);
    }
}

fn scroll_to_prev_user_message(&mut self) {
    let target = self.message_marks.iter().rev().find(|m| {
        m.line_idx < self.viewport_top && m.kind == crate::render::MarkKind::User
    });
    if let Some(target) = target {
        self.scroll_body_to(target.line_idx);
    }
}

fn scroll_to_next_user_message(&mut self) {
    let target = self.message_marks.iter().find(|m| {
        m.line_idx > self.viewport_top && m.kind == crate::render::MarkKind::User
    });
    if let Some(target) = target {
        self.scroll_body_to(target.line_idx);
    }
}
```

添加辅助函数：
```rust
fn scroll_body_to(&mut self, target: usize) {
    let body_height = self.body_height() as usize;
    let max_top = self.body_lines.len().saturating_sub(body_height);
    self.viewport_top = target.min(max_top);
    self.sticky_bottom = self.viewport_top >= max_top;
    self.body_dirty = true;
    self.footer_dirty = true;
    self.paint_frame();
}
```

- [ ] **步骤 4：在 retained 上实现**

镜像实现，但用 `repaint_body_region()` 代替 `paint_frame`：

```rust
fn scroll_to_prev_message(&mut self) {
    let target = self.message_marks.iter().rev().find(|m| m.line_idx < self.viewport_top);
    if let Some(target) = target {
        self.scroll_body_to(target.line_idx);
    }
}
// ... (same shape for other 3)

fn scroll_body_to(&mut self, target: usize) {
    let body_height = self.body_bottom_row() as usize;
    let max_top = self.body_lines.len().saturating_sub(body_height);
    self.viewport_top = target.min(max_top);
    self.sticky_bottom = self.viewport_top >= max_top;
    self.view_mode = !self.sticky_bottom;
    self.repaint_body_region();
}
```

- [ ] **步骤 5：经 worker 接线**

在 `worker.rs` 中添加 4 个新的 `RenderCmd` 变体及处理逻辑：

```rust
RenderCmd::ScrollToPrevMessage,
RenderCmd::ScrollToNextMessage,
RenderCmd::ScrollToPrevUserMessage,
RenderCmd::ScrollToNextUserMessage,
```

并在 `TaskRenderer` 上添加 4 个转发方法。

- [ ] **步骤 6：运行测试**

运行：`cargo test -p rustcode-tuix --lib "scroll_to_prev_message|scroll_to_next_message|scroll_to_prev_user|scroll_to_next_user" 2>&1 | tail -15`
预期：PASS。

- [ ] **步骤 7：提交**

```bash
git add crates/rustcode-tuix/src/render/{mod.rs,alt_screen.rs,retained.rs,worker.rs}
git commit -m "tuix(scroll): add scroll_to_prev/next_message and _user_message variants"
```

### 任务 7.2：在 handle_scroll_key 中绑定 Alt+↑/↓ 与 Ctrl+↑/↓

**文件：**
- 修改：`crates/rustcode-tuix/src/event_loop/mod.rs`

- [ ] **步骤 1：定位 handle_scroll_key**

运行：`grep -nE "fn handle_scroll_key" crates/rustcode-tuix/src/event_loop/mod.rs`
预期：约第 4160 行。

- [ ] **步骤 2：添加按键分支**

在 `handle_scroll_key` 中添加（放进现有的 `match code { ... }`）：

```rust
KeyCode::Up if modifiers.contains(KeyModifiers::ALT) && !modifiers.contains(KeyModifiers::SHIFT) => {
    renderer.scroll_to_prev_message();
    Some(true)
}
KeyCode::Down if modifiers.contains(KeyModifiers::ALT) && !modifiers.contains(KeyModifiers::SHIFT) => {
    renderer.scroll_to_next_message();
    Some(true)
}
KeyCode::Up if modifiers.contains(KeyModifiers::CONTROL) && !modifiers.contains(KeyModifiers::SHIFT) => {
    renderer.scroll_to_prev_user_message();
    Some(true)
}
KeyCode::Down if modifiers.contains(KeyModifiers::CONTROL) && !modifiers.contains(KeyModifiers::SHIFT) => {
    renderer.scroll_to_next_user_message();
    Some(true)
}
```

（把这些放在现有的 `KeyCode::Up if has_shift =>` 分支**之前**，使修饰键的判断顺序没有歧义。）

- [ ] **步骤 3：构建**

运行：`cargo build -p rustcode-tuix 2>&1 | tail -10`
预期：干净。

- [ ] **步骤 4：提交**

```bash
git add crates/rustcode-tuix/src/event_loop/mod.rs
git commit -m "tuix(event_loop): bind Alt+↑/↓ + Ctrl+↑/↓ to message-jump scrolls"
```

---

## 阶段 8：/keys 文档更新

### 任务 8.1：更新中文 KeybindingsHelp

**文件：**
- 修改：`crates/rustcode-core/src/i18n/zh_cn.rs`

- [ ] **步骤 1：编辑 KeybindingsHelp 文案**

找到 `Msg::KeybindingsHelp => r#"..."#.into(),`（约第 161 行）。在现有的 `── 历史 ──` 代码块之后（`── 会话 ──` 之前）插入：

```
  ── 翻看输出 ──
    PageUp / PageDown                上下翻一页（10 行）
    Shift+↑ / Shift+↓                上下翻一行
    Alt+↑ / Alt+↓                    跳到上/下一条消息 ***
    Ctrl+↑ / Ctrl+↓                  跳到上/下一条自己发的消息
    Home / End                       跳到最顶 / 跳回最新
    鼠标滚轮                          上下滚（rustcode 接管）
    Shift+拖鼠标                      用宿主终端选择文本（绕过 rustcode）

  ── 显示 ──
    /scrollbar                       切换右侧滚动条显示
```

并在现有的脚注代码块后追加：

```
  *** Alt+↑/↓ macOS Apple Terminal 需在
      Settings → Profiles → Keyboard 启用 "Use Option as Meta key"
      才会发送修饰键。其他终端默认即可。
```

- [ ] **步骤 2：运行 i18n 一致性测试**

运行：`cargo test -p rustcode-tuix --lib keys_command_is_registered_with_i18n_description_in_both_locales 2>&1 | tail -10`
预期：PASS（现在通过或保持不变）。

- [ ] **步骤 3：提交**

```bash
git add crates/rustcode-core/src/i18n/zh_cn.rs
git commit -m "i18n(zh-cn): add scroll keys + /scrollbar to /keys help"
```

### 任务 8.2：更新英文 KeybindingsHelp

**文件：**
- 修改：`crates/rustcode-core/src/i18n/en.rs`

- [ ] **步骤 1：镜像该改动**

在 en.rs 的 `Msg::KeybindingsHelp` 中添加：

```
  ── Scrollback ──
    PageUp / PageDown                Page up / down (10 lines)
    Shift+↑ / Shift+↓                Line up / down
    Alt+↑ / Alt+↓                    Jump to prev / next message ***
    Ctrl+↑ / Ctrl+↓                  Jump to prev / next user message
    Home / End                       Jump to top / back to latest
    Mouse wheel                      Scroll body (rustcode captures)
    Shift+drag mouse                 Use host terminal selection (bypass)

  ── Display ──
    /scrollbar                       Toggle right-side scrollbar
```

添加脚注：

```
  *** Alt+↑/↓ on macOS Apple Terminal requires enabling "Use Option as
      Meta key" under Settings → Profiles → Keyboard. Other terminals
      send the modifier by default.
```

- [ ] **步骤 2：构建**

运行：`cargo build -p rustcode-core 2>&1 | tail -10`
预期：干净。

- [ ] **步骤 3：提交**

```bash
git add crates/rustcode-core/src/i18n/en.rs
git commit -m "i18n(en): add scroll keys + /scrollbar to /keys help"
```

---

## 阶段 9：最终集成 QA

### 任务 9.1：运行全量测试套件

**文件：** 无

- [ ] **步骤 1：全工作区测试**

运行：`cargo test --workspace 2>&1 | tail -30`
预期：全绿。若有失败，修复后重跑；不要继续往下做。

- [ ] **步骤 2：Lint 检查**

运行：`cargo clippy --workspace --all-targets 2>&1 | tail -30`
预期：本分支不引入新的告警（与 `main` 对比）。

### 任务 9.2：手工集成检查清单

**文件：** 无 —— 本任务产出的是给用户的检查清单，不是代码。

输出给用户（不要自动执行）：

```
请手动验证以下场景（在你日常使用的终端上）：

1. macOS Terminal.app retained + 滚轮上滚 → 进入翻看，新内容静默累积，按 End 跳回
2. iTerm2 alt-screen + Alt+↑/↓ → 在 user/assistant/tool 消息间跳转
3. retained 上 Shift+拖鼠标 → 终端原生选择高亮（rustcode 让出鼠标）
4. retained 上普通拖鼠标 → rustcode 反色高亮，松手 OSC 52 写剪贴板
5. /scrollbar 切换可视滚动条；重启 rustcode 后状态保留
6. streaming 进行时 PageUp → viewport 不动，spinner 继续转
7. 翻看中 /clear → 立即回 sticky 跟底
8. 翻看中 approval 弹出 → 强制回 sticky，approval 在底部正常审批
9. retained 启动 → 滚轮事件确实由 rustcode 处理（验证：宿主终端滚轮不再滚启动前的历史）
10. retained + /bash ls 等长命令走 suspend_for_external → child 期间宿主终端鼠标恢复；resume 后 rustcode 重新接管

复测后告诉我结果。如果有失败的，反馈是哪条 + 终端/OS 信息。
```

不要自动执行；这些必须由用户手动驱动。只有在用户确认手工检查清单完成后，才把任务标记为完成。

---

## 自查

跑完所有 Phase 后，按照 spec 的每段对照检查任务覆盖：

- [x] 阶段 0：Msg 变体 —— 任务 0.1
- [x] Phase 1: Selection 共享模块 — Tasks 1.1-1.4
- [x] 阶段 2：body 缓冲 + MessageMark —— 任务 2.1-2.4
- [x] 阶段 3：retained view_mode + 滚动 —— 任务 3.1-3.4
- [x] Phase 4: retained 鼠标接管 — Tasks 4.1-4.3
- [x] Phase 5: retained selection 接入 — Tasks 5.1-5.3
- [x] 阶段 6：滚动条 + /scrollbar + ui-state.toml —— 任务 6.1-6.6
- [x] 阶段 7：附加滚动按键 —— 任务 7.1-7.2
- [x] 阶段 8：/keys 文档 —— 任务 8.1-8.2
- [x] Phase 9: 集成 QA — Tasks 9.1-9.2
