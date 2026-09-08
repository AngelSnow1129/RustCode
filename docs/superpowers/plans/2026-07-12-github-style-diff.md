# GitHub 风格 Diff（行号 + 颜色）实施计划

> **面向 agentic worker：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实施本计划。步骤使用复选框（`- [ ]`）语法进行跟踪。

**目标：** 以 GitHub 风格渲染 edit/write diff —— 真正的行级 diff，带右对齐的行号 gutter、`+`/`-`/context 符号，以及绿/红/灰的前景着色。

**架构：** 把 `rustcode-capabilities` 中简陋的 `build_compact_diff`（只取旧的头几行 / 新的头几行，没有真正做匹配）替换为由 `similar` crate 在**整个**旧文件与新文件上计算出的真正 unified diff（上下文半径 3，带上限）—— 这样可以直接得到正确的文件行号 + hunk，正如 codex 的做法。diff 仍然作为 tool-result 字符串传给 TUI（不新增事件管线）；TUI 再把该 unified diff 重新解析成带行号、按颜色编码的行。仅使用前景色（cell 模型没有背景），不做语法高亮。

**技术栈：** Rust、`similar` crate（行 diff，与 codex 使用的是同一个 crate）、`rustcode-capabilities`（diff 计算）、`rustcode-tuix`（解析 + 渲染）。

## 全局约束

- 仅使用前景色。`CellStyle` 没有背景（`fg`/`bold`/`reverse`/`faint`）；不要添加背景底纹。着色沿用现有的 `Role::DiffAdd`（绿）/ `Role::DiffRemove`（红）/ `Role::Muted`（context），它们都跟随主题。
- 不做语法高亮。本仓库有意从 TUI 中移除了 syntect（macOS Terminal 选区叠加层 bug）；着色仅到行级别。
- `similar` 作为**可选**依赖加入，由 `rustcode-capabilities` 现有的 `tools` feature 控制（与 `edit_file` 所用的 feature 相同）。
- 范围限定在 `rustcode-capabilities` 的 diff（喂给默认的 v2 引擎 → TUI）。与之平行的 `rustcode-core/src/tool/edit.rs::build_compact_diff`（v1/legacy，本分支正在退役）不在范围内。
- 提交纪律：只用 `git add <path>` 暂存每个任务所改动的文件；绝不使用 `-A`/`.`/`-u`。
- 已知问题：rustcode-tuix 中约有 4 个既有的 "byte budget" retained 测试失败 —— 与本任务无关；确认失败数量不增加。改动下层 crate 后，若遇到产物过期，跑 tuix 测试前先 `touch crates/rustcode-core/src/lib.rs`。

## 文件结构

| 文件 | 职责 | 改动 |
|---|---|---|
| `Cargo.toml`（workspace）| 依赖版本 | 在 `[workspace.dependencies]` 中添加 `similar = "2"` |
| `crates/rustcode-capabilities/Cargo.toml` | crate 依赖 | 添加可选的 `similar`，放进 `tools` feature |
| `crates/rustcode-capabilities/src/tools/edit.rs` | diff 计算 | `build_compact_diff` → 对整个文件做 unified diff；2 处调用点；测试 |
| `crates/rustcode-tuix/src/render/mod.rs` | diff 数据类型 | `DiffEntry` → `{ kind, old_lineno, new_lineno, text }` + `DiffKind` 枚举 |
| `crates/rustcode-tuix/src/render/diff.rs`（新增）| 纯 diff 逻辑 | `parse_unified_diff`、`diff_gutter_width`、`diff_row_text`（+ 测试）|
| `crates/rustcode-tuix/src/render/mod.rs` | 模块接线 | `pub(crate) mod diff;` |
| `crates/rustcode-tuix/src/event_loop/mod.rs` | 接入解析器 | 把 `strip_prefix` 解析器替换为 `parse_unified_diff` |
| `crates/rustcode-tuix/src/render/retained.rs` | 交互式渲染 | 绘制 gutter + 符号，按 kind 着色 |
| `crates/rustcode-tuix/src/render/plain.rs` | 管道渲染 | 同上，使用 SGR |

---

## Task 1: 在 capabilities 中实现真正的 unified diff

**文件：**
- 修改：`Cargo.toml` (workspace `[workspace.dependencies]`, ~line 38)
- 修改：`crates/rustcode-capabilities/Cargo.toml` (`[dependencies]` + `tools` feature)
- 修改：`crates/rustcode-capabilities/src/tools/edit.rs:177-204` (`build_compact_diff`), call sites `:123` and `:167`, tests `:356-357` and `:362-371`

**接口：**
- 产出： `fn build_compact_diff(old_file: &str, new_file: &str) -> String` —— 一个 git unified diff（`@@ -a,b +c,d @@` hunk，3 行上下文，上限 60 行）。调用方传入**完整**的编辑前与编辑后文件内容。

- [ ] **Step 1: 添加 `similar` 依赖**

在 workspace 的 `Cargo.toml` 中，在 `[workspace.dependencies]` 下（按字母序，放在 `anyhow = "1"` 之后）添加：
```toml
similar = "2"
```
在 `crates/rustcode-capabilities/Cargo.toml` 的 `[dependencies]` 中添加（靠近 `regex` 等其他可选 tools 依赖）：
```toml
# Real line diff for edit_file's compact diff (git-style unified hunks + line numbers).
similar = { workspace = true, optional = true }
```
并把 `"dep:similar"` 加入 `tools` feature 列表：
```toml
tools = ["dep:ignore", "dep:regex", "dep:grep", "dep:globset", "dep:encoding_rs", "dep:tokio-util", "dep:similar", "tokio/fs", "tokio/process", "tokio/io-util"]
```

- [ ] **Step 2: 编写失败测试**

在 `edit.rs` 中，把既有的 `compact_diff_truncates_each_side` 测试（第 362-371 行）替换为：
```rust
    #[test]
    fn compact_diff_is_unified_with_line_numbers() {
        // Whole-file old vs new; a real diff must produce a `@@` hunk header whose
        // new-side start reflects the changed line's position in the file.
        let old = "fn main() {\n    let x = 1;\n}\n";
        let new = "fn main() {\n    let x = 2;\n}\n";
        let diff = build_compact_diff(old, new);
        assert!(diff.contains("@@"), "must be a unified diff with a hunk header: {diff}");
        assert!(diff.contains("-    let x = 1;"), "removed line present: {diff}");
        assert!(diff.contains("+    let x = 2;"), "added line present: {diff}");
        // The change is on file line 2, so the hunk header covers line 2 on both sides.
        assert!(diff.contains("-2") && diff.contains("+2"), "hunk covers line 2: {diff}");
    }

    #[test]
    fn compact_diff_caps_huge_diffs() {
        let old = String::new();
        let new: String = (0..200).map(|i| format!("line {i}\n")).collect();
        let diff = build_compact_diff(&old, &new);
        assert!(diff.lines().count() <= 61, "capped: {} lines", diff.lines().count());
        assert!(diff.contains("more diff lines"), "shows a truncation note: {diff}");
    }
```
同时把既有 `unique_replace_succeeds` 测试（第 356-357 行）中的断言从：
```rust
        assert!(r.content.contains("- let x = 1;"), "{}", r.content);
        assert!(r.content.contains("+ let x = 2;"), "{}", r.content);
```
改为（新格式在符号后没有空格，且周围的 `fn main` 行是未改动的文件上下文）：
```rust
        assert!(r.content.contains("-    let x = 1;"), "{}", r.content);
        assert!(r.content.contains("+    let x = 2;"), "{}", r.content);
```
（注意：`unique_replace_succeeds` 写入文件 `"fn main() {\n    let x = 1;\n}\n"`，并把 `let x = 1;` 改为 `let x = 2;`，因此 diff 覆盖整个文件，被改动的行保留其 4 空格缩进。）

- [ ] **Step 3: 运行测试确认失败**

运行：`cargo test -p rustcode-capabilities compact_diff_is_unified_with_line_numbers`
预期：FAIL —— 旧的 `build_compact_diff` 产出的是 `- 1` 风格的输出，没有 `@@`。

- [ ] **Step 4: 重写 `build_compact_diff`**

把 `build_compact_diff`（第 177-204 行）替换为：
```rust
/// A compact GIT UNIFIED DIFF (`@@` hunks, 3 lines of context) between the OLD
/// and NEW whole-file contents, capped so a large edit can't flood the model
/// context / transcript. The TUI re-parses this into a line-numbered, color-
/// coded diff block; the model reads it as a normal unified diff.
fn build_compact_diff(old_file: &str, new_file: &str) -> String {
    const MAX_DIFF_LINES: usize = 60;
    let full = similar::TextDiff::from_lines(old_file, new_file)
        .unified_diff()
        .context_radius(3)
        .to_string();
    let full = full.trim_end();
    let lines: Vec<&str> = full.lines().collect();
    if lines.len() <= MAX_DIFF_LINES {
        return full.to_string();
    }
    let mut out = lines[..MAX_DIFF_LINES].join("\n");
    out.push_str(&format!(
        "\n… ({} more diff lines)",
        lines.len() - MAX_DIFF_LINES
    ));
    out
}
```

- [ ] **Step 5: 更新两处调用点以传入完整文件**

在 `edit.rs:123`（fuzzy 路径 —— `content` 是旧文件，`fuzzy_result` 是新文件），把：
```rust
                let diff = build_compact_diff(&a.old_string, &a.new_string);
```
改为：
```rust
                let diff = build_compact_diff(&content, &fuzzy_result);
```
在 `edit.rs:167`（普通路径 —— `content` 旧，`updated` 新），把：
```rust
        let diff = build_compact_diff(&a.old_string, &a.new_string);
```
改为：
```rust
        let diff = build_compact_diff(&content, &updated);
```

- [ ] **Step 6: 运行测试 + 构建**

运行：`cargo test -p rustcode-capabilities compact_diff unique_replace_succeeds`
预期：PASS（两个新的 diff 测试 + 更新后的 edit 测试）。
运行：`cargo build -p rustcode-capabilities`
预期：干净无错误。

- [ ] **Step 7: 提交**
```bash
git add Cargo.toml crates/rustcode-capabilities/Cargo.toml crates/rustcode-capabilities/src/tools/edit.rs Cargo.lock
git commit -m "feat(capabilities): compute edit diffs as real unified diffs (similar)"
```

---

## Task 2: TUI diff 类型 + 纯解析/格式化逻辑

**文件：**
- 修改：`crates/rustcode-tuix/src/render/mod.rs:560-565` (`DiffEntry`) + add `pub(crate) mod diff;`
- 新建：`crates/rustcode-tuix/src/render/diff.rs`

**接口：**
- 产出：
  - `pub enum DiffKind { Add, Del, Context }`
  - `pub struct DiffEntry { pub kind: DiffKind, pub old_lineno: Option<usize>, pub new_lineno: Option<usize>, pub text: String }`
  - `pub(crate) fn parse_unified_diff(diff: &str, max_lines: usize) -> Vec<DiffEntry>`
  - `pub(crate) fn diff_gutter_width(entries: &[DiffEntry]) -> usize`
  - `pub(crate) fn diff_row_text(entry: &DiffEntry, gutter: usize) -> String` — `"  {num:>gutter} {sign} {text}"`

- [ ] **Step 1: 替换 `DiffEntry` 类型**

在 `render/mod.rs` 中，替换（第 560-565 行）：
```rust
/// One line in a diff batch. `added = true` renders as `+`, false as `-`.
#[derive(Debug, Clone)]
pub struct DiffEntry {
    pub added: bool,
    pub text: String,
}
```
为：
```rust
/// The role of a diff line: an addition (`+`), a deletion (`-`), or unchanged
/// context (` `). Drives the sign + color in the renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffKind {
    Add,
    Del,
    Context,
}

/// One line of a rendered diff, with the file line number for its side.
/// `old_lineno` is set for Del + Context, `new_lineno` for Add + Context.
#[derive(Debug, Clone)]
pub struct DiffEntry {
    pub kind: DiffKind,
    pub old_lineno: Option<usize>,
    pub new_lineno: Option<usize>,
    pub text: String,
}
```
然后在 `render/mod.rs` 顶部其他 `mod` 行附近添加模块声明（搜索 `mod retained;` / `mod plain;`，与它们并列添加）：
```rust
pub(crate) mod diff;
```

- [ ] **Step 2: 编写失败测试** —— 先创建 `crates/rustcode-tuix/src/render/diff.rs`，其中只放测试模块（这样会因缺少相应 fn 而编译失败）：
```rust
//! Pure diff-parsing and row-formatting logic for `UiLine::DiffBlock`.
//! Rendering (cells/SGR) lives in retained.rs / plain.rs; this module only
//! turns a unified-diff string into line-numbered entries and formats a row.

use crate::render::{DiffEntry, DiffKind};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hunk_line_numbers_and_kinds() {
        let diff = "\
@@ -1,3 +1,3 @@
 fn main() {
-    let x = 1;
+    let x = 2;
 }";
        let e = parse_unified_diff(diff, 100);
        assert_eq!(e.len(), 4);
        assert_eq!(e[0].kind, DiffKind::Context);
        assert_eq!((e[0].old_lineno, e[0].new_lineno), (Some(1), Some(1)));
        assert_eq!(e[1].kind, DiffKind::Del);
        assert_eq!((e[1].old_lineno, e[1].new_lineno), (Some(2), None));
        assert_eq!(e[1].text, "    let x = 1;");
        assert_eq!(e[2].kind, DiffKind::Add);
        assert_eq!((e[2].old_lineno, e[2].new_lineno), (None, Some(2)));
        assert_eq!(e[3].kind, DiffKind::Context);
        assert_eq!((e[3].old_lineno, e[3].new_lineno), (Some(3), Some(3)));
    }

    #[test]
    fn ignores_preamble_and_file_headers() {
        let diff = "\
Edited a.rs (1 replacement)
--- a/a.rs
+++ b/a.rs
@@ -2,1 +2,1 @@
-old
+new";
        let e = parse_unified_diff(diff, 100);
        // The `Edited …`, `--- a/a.rs`, `+++ b/a.rs` lines must NOT become entries.
        assert_eq!(e.len(), 2);
        assert_eq!(e[0].kind, DiffKind::Del);
        assert_eq!(e[0].text, "old");
        assert_eq!(e[1].kind, DiffKind::Add);
        assert_eq!(e[1].text, "new");
    }

    #[test]
    fn respects_max_lines() {
        let mut diff = String::from("@@ -1,0 +1,50 @@\n");
        for i in 0..50 {
            diff.push_str(&format!("+line {i}\n"));
        }
        let e = parse_unified_diff(&diff, 10);
        assert_eq!(e.len(), 10);
    }

    #[test]
    fn gutter_width_and_row_format() {
        let entries = vec![
            DiffEntry { kind: DiffKind::Context, old_lineno: Some(9), new_lineno: Some(9), text: "ctx".into() },
            DiffEntry { kind: DiffKind::Add, old_lineno: None, new_lineno: Some(10), text: "added".into() },
            DiffEntry { kind: DiffKind::Del, old_lineno: Some(10), new_lineno: None, text: "removed".into() },
        ];
        let w = diff_gutter_width(&entries);
        assert_eq!(w, 2); // largest line number is 10 → width 2
        assert_eq!(diff_row_text(&entries[0], w), "   9   ctx");
        assert_eq!(diff_row_text(&entries[1], w), "  10 + added");
        assert_eq!(diff_row_text(&entries[2], w), "  10 - removed");
    }
}
```

- [ ] **Step 3: 运行测试确认失败**

运行：`cargo test -p rustcode-tuix --lib render::diff`
预期：编译 FAIL —— 找不到 `parse_unified_diff` / `diff_gutter_width` / `diff_row_text`。

- [ ] **Step 4: 实现纯函数** —— 添加到 `diff.rs` 中 `#[cfg(test)] mod tests` 的上方：
```rust
/// Parse a git unified diff (`@@ -a,b +c,d @@` hunks + ` `/`+`/`-` lines) into
/// line-numbered entries. Lines before the first `@@`, and `---`/`+++` file
/// headers, are ignored. Stops after `max_lines` entries.
pub(crate) fn parse_unified_diff(diff: &str, max_lines: usize) -> Vec<DiffEntry> {
    let mut out: Vec<DiffEntry> = Vec::new();
    let mut old_ln = 0usize;
    let mut new_ln = 0usize;
    for line in diff.lines() {
        if out.len() >= max_lines {
            break;
        }
        if let Some(rest) = line.strip_prefix("@@") {
            if let Some((o, n)) = parse_hunk_header(rest) {
                old_ln = o;
                new_ln = n;
            }
            continue;
        }
        if line.starts_with("---") || line.starts_with("+++") {
            continue; // unified-diff file headers
        }
        if old_ln == 0 && new_ln == 0 {
            continue; // preamble before the first hunk
        }
        match line.as_bytes().first() {
            Some(b'+') => {
                out.push(DiffEntry {
                    kind: DiffKind::Add,
                    old_lineno: None,
                    new_lineno: Some(new_ln),
                    text: line[1..].to_string(),
                });
                new_ln += 1;
            }
            Some(b'-') => {
                out.push(DiffEntry {
                    kind: DiffKind::Del,
                    old_lineno: Some(old_ln),
                    new_lineno: None,
                    text: line[1..].to_string(),
                });
                old_ln += 1;
            }
            Some(b' ') => {
                out.push(DiffEntry {
                    kind: DiffKind::Context,
                    old_lineno: Some(old_ln),
                    new_lineno: Some(new_ln),
                    text: line[1..].to_string(),
                });
                old_ln += 1;
                new_ln += 1;
            }
            _ => {} // `\ No newline at end of file`, blank lines, etc.
        }
    }
    out
}

/// Parse the two 1-based start line numbers from a hunk header body
/// (`rest` = the text after `@@`, e.g. ` -12,3 +14,4 @@ …`). Returns
/// `(old_start, new_start)`.
fn parse_hunk_header(rest: &str) -> Option<(usize, usize)> {
    let mut old_start = None;
    let mut new_start = None;
    for tok in rest.split_whitespace() {
        if let Some(o) = tok.strip_prefix('-') {
            old_start = o.split(',').next().and_then(|s| s.parse::<usize>().ok());
        } else if let Some(n) = tok.strip_prefix('+') {
            new_start = n.split(',').next().and_then(|s| s.parse::<usize>().ok());
        }
    }
    Some((old_start?, new_start?))
}

/// Width of the line-number gutter: the digit count of the largest line number
/// shown across `entries` (Del shows old, others show new), minimum 1.
pub(crate) fn diff_gutter_width(entries: &[DiffEntry]) -> usize {
    entries
        .iter()
        .filter_map(|e| match e.kind {
            DiffKind::Del => e.old_lineno,
            _ => e.new_lineno,
        })
        .max()
        .unwrap_or(1)
        .to_string()
        .len()
        .max(1)
}

/// Format one diff row as `"  {num:>gutter} {sign} {text}"` — the display line
/// (WITHOUT color; the caller applies the theme role). `text` is control-scrubbed.
pub(crate) fn diff_row_text(entry: &DiffEntry, gutter: usize) -> String {
    let num = match entry.kind {
        DiffKind::Del => entry.old_lineno,
        _ => entry.new_lineno,
    };
    let numstr = num.map(|n| n.to_string()).unwrap_or_default();
    let sign = match entry.kind {
        DiffKind::Add => '+',
        DiffKind::Del => '-',
        DiffKind::Context => ' ',
    };
    format!(
        "  {numstr:>gutter$} {sign} {}",
        crate::render::scrub_controls_pub(&entry.text)
    )
}
```
关于 `scrub_controls` 的说明：如果 `scrub_controls` 是 `render/diff.rs` 无法访问的私有辅助函数，那么要么 (a) 在 `scrub_controls` 所在模块中做一个小的 `pub(crate) fn scrub_controls_pub` 再导出，要么 (b) 把既有的 `scrub_controls` 调用内联到两处 RENDER 站点（retained/plain），而不是放在 `diff_row_text` 内部，并去掉这里的 scrub。推荐 (b)：去掉这里的 `crate::render::scrub_controls_pub(&entry.text)` 包裹（直接使用原始的 `&entry.text`），并保留 retained.rs/plain.rs 中既有的 `scrub_controls(&…)`。据此更新 Step-2 测试的期望字符串（它们用的是纯 ASCII，因此无需改动）。实现前先 grep `fn scrub_controls` 确认采用哪一种。

- [ ] **Step 5: 运行测试确认通过**

运行：`cargo test -p rustcode-tuix --lib render::diff`
预期：PASS（全部 4 个）。

- [ ] **Step 6: 提交**
```bash
git add crates/rustcode-tuix/src/render/mod.rs crates/rustcode-tuix/src/render/diff.rs
git commit -m "feat(tuix): line-numbered DiffEntry + pure unified-diff parser/formatter"
```

---

## Task 3: 接入解析器 + 渲染 gutter

**文件：**
- 修改：`crates/rustcode-tuix/src/event_loop/mod.rs:9017-9040` (the diff parser call)
- 修改：`crates/rustcode-tuix/src/render/retained.rs:4448-4459` (`UiLine::DiffBlock` arm)
- 修改：`crates/rustcode-tuix/src/render/plain.rs:287-306` (`UiLine::DiffBlock` arm)

**接口：**
- 消费： `parse_unified_diff`, `diff_gutter_width`, `diff_row_text`, `DiffKind` (Task 2).

- [ ] **Step 1: 编写失败测试** —— 添加到 retained 测试模块中（使用 `new_capturing`/`drain_into_vterm`，与其他 vterm 测试位于同一模块）：
```rust
    #[test]
    fn diff_block_renders_line_number_gutter() {
        use crate::render::{DiffEntry, DiffKind};
        let (mut r, buf) = new_capturing(80, 24);
        r.caps.colors = true;
        let mut vterm = crate::test_term::VirtualTerminal::new(80, 24);
        r.render(UiLine::DiffBlock(vec![
            DiffEntry { kind: DiffKind::Context, old_lineno: Some(9), new_lineno: Some(9), text: "keep".into() },
            DiffEntry { kind: DiffKind::Del, old_lineno: Some(10), new_lineno: None, text: "old line".into() },
            DiffEntry { kind: DiffKind::Add, old_lineno: None, new_lineno: Some(10), text: "new line".into() },
        ]));
        r.render(UiLine::InputPrompt {
            buf: String::new(), cursor_byte: 0, menu: None,
            status: status_basic(), attachments: Vec::new(),
        });
        r.flush_deferred();
        drain_into_vterm(&buf, &mut vterm);
        // Gutter shows the line number, then the sign, then content.
        assert!(vterm.any_row(|row| row.contains("10 - old line")), "removed row w/ gutter\n{}", vterm.dump());
        assert!(vterm.any_row(|row| row.contains("10 + new line")), "added row w/ gutter\n{}", vterm.dump());
        assert!(vterm.any_row(|row| row.contains("9   keep")), "context row w/ gutter\n{}", vterm.dump());
    }
```

- [ ] **Step 2: 运行测试确认失败**

运行：`cargo test -p rustcode-tuix diff_block_renders_line_number_gutter`
预期：FAIL —— 当前渲染器输出的是 `       - old line`（7 个空格，没有 gutter），而且本来就无法针对新的 `DiffEntry` 字段编译通过。

- [ ] **Step 3: 重写 event_loop 解析器**

把 `event_loop/mod.rs:9017-9040`（通过 `strip_prefix` 构建 `diff_entries` 的 `if emits_diff { … }` 块）替换为：
```rust
            if emits_diff {
                let diff_entries = crate::render::diff::parse_unified_diff(&output, 120);
                if !diff_entries.is_empty() {
                    renderer.render(UiLine::DiffBlock(diff_entries));
                }
            }
```

- [ ] **Step 4: 重写 retained 渲染器**

把 `retained.rs:4448-4459`（`UiLine::DiffBlock(entries) => { … }`）替换为：
```rust
            UiLine::DiffBlock(entries) => {
                let gutter = crate::render::diff::diff_gutter_width(&entries);
                for entry in &entries {
                    let role = match entry.kind {
                        crate::render::DiffKind::Add => Role::DiffAdd,
                        crate::render::DiffKind::Del => Role::DiffRemove,
                        crate::render::DiffKind::Context => Role::Muted,
                    };
                    let style = self.style_for(role);
                    let body = crate::render::diff::diff_row_text(entry, gutter);
                    self.push_body_text(&scrub_controls(&body), &style);
                }
            }
```
（按 Task 2 的说明 (b)，`diff_row_text` 返回原始文本；`scrub_controls` 在此处的渲染站点上应用，与旧行为一致。）

- [ ] **Step 5: 重写 plain 渲染器**

把 `plain.rs:287-306`（`UiLine::DiffBlock(entries) => { … }`）替换为：
```rust
            UiLine::DiffBlock(entries) => {
                self.drop_transient();
                let gutter = crate::render::diff::diff_gutter_width(&entries);
                for entry in &entries {
                    let color = if self.caps.colors {
                        match entry.kind {
                            crate::render::DiffKind::Add => SGR_GREEN,
                            crate::render::DiffKind::Del => SGR_RED,
                            crate::render::DiffKind::Context => "",
                        }
                    } else {
                        ""
                    };
                    let reset = if self.caps.colors && !color.is_empty() { SGR_RESET } else { "" };
                    let body = crate::render::diff::diff_row_text(entry, gutter);
                    let _ = writeln!(self.out, "{}{}{}", color, scrub_controls(&body), reset);
                }
            }
```

- [ ] **Step 6: 运行测试 + 构建**

运行：`cargo build -p rustcode-tuix` —— 干净通过（`DiffEntry.added` 字段已移除；编译器会确认所有使用方都已更新）。若 `UiLine::DiffLine { added, text }`（位于 `render/mod.rs:116` 的另一个单行变体）仍能编译 —— 它用的是自己的 `added`/`text`，而非 `DiffEntry`，因此不受影响；保持原样。
运行：`cargo test -p rustcode-tuix diff_block_renders_line_number_gutter`
预期：PASS。
运行：`cargo test -p rustcode-tuix --lib`
预期：PASS，除了约 4 个既有的 byte-budget 红灯（数量不变）。

- [ ] **Step 7: 提交**
```bash
git add crates/rustcode-tuix/src/event_loop/mod.rs crates/rustcode-tuix/src/render/retained.rs crates/rustcode-tuix/src/render/plain.rs
git commit -m "feat(tuix): render diffs with a line-number gutter + kind coloring"
```

---

## Task 4: 验证

- [ ] **Step 1: 全 workspace 构建 + 受影响 crate 的测试**

运行：`touch crates/rustcode-core/src/lib.rs && cargo build`
预期：干净无错误。
运行：`cargo test -p rustcode-capabilities -p rustcode-tuix`
预期：全绿，除了约 4 个既有的 tuix byte-budget 红灯（与干净 checkout 的数量相同）。

- [ ] **Step 2: 手动冒烟（记录在案，仅限真实终端）**

在 PR/commit 中记录以下各项需要真实终端：
1. 对一个真实源文件执行 `edit_file` → diff 显示右对齐的行号 gutter、`+`/`-`/context 符号，以及绿/红/灰的行，且文件行号正确。
2. 多 hunk 编辑（`replace_all`）→ 多个 hunk，每个都有自己的行号。
3. 一次超大改写 → 截断后的 diff，带 `… (N more diff lines)`。
4. 非彩色终端（`caps.colors=false`）→ gutter + 符号仍然存在，无颜色。
5. `/resume` 一个包含编辑操作的会话 → diff 从存储的 tool result 重新渲染（走同一个解析器路径）。

- [ ] **Step 3: 申请评审** —— 合并前对分支 diff 执行 `/code-review`。

---

## 自审（编写期间完成）

- **覆盖度：** 真正的 diff + 行号 → Task 1（similar unified diff）+ Task 2（解析器从 hunk header 中取文件行号）。颜色区分 → Task 3（DiffAdd/DiffRemove/Muted，仅前景色）。gutter 渲染 → Task 3。截断上限 → Task 1 + 解析器的 `max_lines`。遵守了仅前景色 / 不使用 syntect 的约束（无背景、无高亮器）。`similar` 的 feature 门控 → Task 1 Step 1。
- **占位符：** 无 —— 每个代码步骤都是完整的。唯一被标记的待查项（`scrub_controls` 的可见性，Task 2 Step 4）采用方案 (b)：在渲染站点（retained/plain）做 scrub，而不是在 `diff_row_text` 内部 —— 渲染步骤（Task 3）已经用 `scrub_controls` 包裹，且 `diff_row_text` 返回原始文本。实现时确保 `diff_row_text` 使用 `&entry.text`（而非 scrub 包装）。
- **类型一致性：** `DiffEntry { kind, old_lineno, new_lineno, text }` + `DiffKind { Add, Del, Context }` 在 Task 2（定义/解析器/格式化）与 Task 3（event_loop/retained/plain）中用法完全一致。`parse_unified_diff(&str, usize)`、`diff_gutter_width(&[DiffEntry])`、`diff_row_text(&DiffEntry, usize)` 的签名在定义处（Task 2）与调用点（Task 3）保持一致。
- **留给实现者的开放项：** 在 Task 2 Step 4 实现之前，先 `grep -n "fn scrub_controls" crates/rustcode-tuix/src`，并采用方案 (b)：`diff_row_text` 返回 `format!("  {numstr:>gutter$} {sign} {}", entry.text)`（原始文本），scrub 仍留在两处渲染站点。
