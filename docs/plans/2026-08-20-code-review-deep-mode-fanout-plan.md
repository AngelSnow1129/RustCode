# code-review deep mode（维度 fan-out）实施计划

> **面向 agentic worker：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实施本计划。步骤使用复选框（`- [ ]`）语法跟踪进度。

**目标：** 新增可选的 `/review deep` 模式：按关注维度（correctness / security / performance / tests&contracts）各派发一个只读 reviewer 并发执行，再合并并去重它们的发现；而默认的 `/review` 保持原样，仍只运行单个 agent。

**架构：** 新增 `rustcode-review/src/fanout.rs`，承载一张固定的维度表（每个维度是一段 `persona_append` 视角）、一个纯函数去重器 `merge_findings`、一个纯 deep-review 渲染器，以及一个泛型编排器 `run_deep_review`（并发用 `tokio::task::JoinSet`，每维度的 runner 可注入以便测试）。`ReviewTool::execute` 新增 `depth` 参数并分派：`single`（完全沿用现有路径，不做改动）与 `deep`（通过 `build_review_agent_with` + `ReviewAgentConfig::with_persona_append` 构建 N 个维度 agent，每个 agent 自带 `ReportFindingTool` 汇聚点，然后 merge → 范围过滤 → 渲染）。

**技术栈：** Rust、tokio（`rt-multi-thread`、`macros`、`time`、`sync` —— 均已启用）、`rustcode-kernel` 的 Agent、`rustcode-capabilities` 的 `Finding`/`ReportFindingTool`。不引入新依赖。

**规范：** `docs/plans/2026-08-20-code-review-deep-mode-fanout-design.md`

## 全局约束

- 不新增 crate 依赖。`futures` 在 `rustcode-review` 中仅为 dev 依赖，因此生产环境的编排必须使用 `tokio::task::JoinSet`（而非 `futures::future::join_all`）。
- 单 agent 路径（`depth` 缺省或 `"single"`）必须与当前行为完全一致。`rustcode-review` 与 `rustcode-tuix` 的所有既有测试保持通过且不改动。
- deep 模式仅按需开启（`/review deep` / 工具参数 `depth:"deep"`）。默认仍为 single。
- 范围预检/确认必须在任何 fan-out 之前执行（复用 `execute()` 中现有的 `ScopeManifest`/`ScopeLimits` 代码块）。
- review 发现以英文渲染（与现有 `render_findings` 的输出保持一致）。
- `Finding` 字段（来自 `rustcode-capabilities`，禁止修改）：`title: String`、`body: String`、`priority: String`（`"P0"`..`"P3"`，0 最严重）、`confidence: f32`（0.0..=1.0）、`file_path: String`、`line_start: u32`、`line_end: u32`、`suggestion: String`、`suggested_code: String`。

---

### 任务 1：维度表与模块接线

**文件：**
- 新增：`crates/rustcode-review/src/fanout.rs`
- 修改：`crates/rustcode-review/src/lib.rs`（声明并导出该模块）
- 测试：位于 `fanout.rs` 的 `#[cfg(test)]`

**接口：**
- 产出：
  - `pub struct ReviewDimension { pub id: &'static str, pub display: &'static str, pub lens: &'static str }`
  - `pub const REVIEW_DIMENSIONS: &[ReviewDimension]`（4 项：`correctness`、`security`、`performance`、`tests_contracts`）

- [ ] **步骤 1：编写失败测试**

在新建的 `crates/rustcode-review/src/fanout.rs` 中，放入这张维度表与下面的测试：

```rust
//! Deep-mode dimension fan-out for `code_review`: many read-only reviewers, one
//! per concern lens, merged into a single deduped finding set. See
//! docs/plans/2026-08-20-code-review-deep-mode-fanout-design.md.

/// One review lens. `lens` is appended to the base reviewer persona via
/// `ReviewAgentConfig::with_persona_append`, biasing focus without replacing the
/// shared reviewer instructions.
pub struct ReviewDimension {
    pub id: &'static str,
    pub display: &'static str,
    pub lens: &'static str,
}

/// The concern dimensions a deep review fans out across, in display order. Each
/// reviewer sees the FULL diff through its lens; overlap is resolved by
/// `merge_findings` (favor recall).
pub const REVIEW_DIMENSIONS: &[ReviewDimension] = &[
    ReviewDimension {
        id: "correctness",
        display: "Correctness",
        lens: "\n\n## This review's lens: CORRECTNESS\nConcentrate on logic errors, wrong \
               conditions, off-by-one, unhandled edge cases, error handling, concurrency/races, \
               and regressions introduced by this diff. Still report anything clearly severe you \
               notice outside this lens.",
    },
    ReviewDimension {
        id: "security",
        display: "Security",
        lens: "\n\n## This review's lens: SECURITY\nConcentrate on injection, missing authz/authn, \
               secret handling, unsafe deserialization, path/SSRF issues, and supply-chain surface \
               (dependency, CI, and config changes). Still report anything clearly severe you \
               notice outside this lens.",
    },
    ReviewDimension {
        id: "performance",
        display: "Performance",
        lens: "\n\n## This review's lens: PERFORMANCE\nConcentrate on hot-path cost, needless \
               allocations/clones, blocking calls on async paths, N+1 / repeated I/O, and \
               accidental quadratic behavior introduced by this diff. Still report anything \
               clearly severe you notice outside this lens.",
    },
    ReviewDimension {
        id: "tests_contracts",
        display: "Tests & contracts",
        lens: "\n\n## This review's lens: TESTS & CONTRACTS\nConcentrate on whether the change is \
               covered by tests, whether public APIs/contracts stay consistent, and whether the \
               diff changes a convention on its lines while leaving sibling/parallel code on the \
               old form (a one-sided divergence). Still report anything clearly severe you notice \
               outside this lens.",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dimension_table_is_the_four_expected_lenses() {
        let ids: Vec<_> = REVIEW_DIMENSIONS.iter().map(|d| d.id).collect();
        assert_eq!(
            ids,
            ["correctness", "security", "performance", "tests_contracts"]
        );
        for d in REVIEW_DIMENSIONS {
            assert!(!d.display.is_empty(), "{} display", d.id);
            assert!(
                d.lens.contains("This review's lens"),
                "{} lens must be an appendable section",
                d.id
            );
        }
    }
}
```

然后在 `crates/rustcode-review/src/lib.rs` 中接线：在其他 `pub mod` 行旁边加上 `pub mod fanout;`，并在再导出列表中补充：

```rust
pub use fanout::{ReviewDimension, REVIEW_DIMENSIONS};
```

- [ ] **步骤 2：运行测试确认其失败**

运行：`cargo test -p rustcode-review --lib fanout::tests::dimension_table_is_the_four_expected_lenses`
预期：先编译失败（模块未声明）→ 声明后转为 PASS。若编译通过却测试失败，说明维度表写错了；修正到它只因真实原因而失败为止。（本任务以数据为主，文件与模块接线就绪后即通过。）

- [ ] **步骤 3：编写最小实现**

已在步骤 1 写好（这张表本身就是实现）。确保 `lib.rs` 声明了 `pub mod fanout;`，且再导出能编译通过。

- [ ] **步骤 4：运行测试确认其通过**

运行：`cargo test -p rustcode-review --lib fanout`
预期：PASS。

- [ ] **步骤 5：提交**

```bash
git add crates/rustcode-review/src/fanout.rs crates/rustcode-review/src/lib.rs
git commit -m "feat(review): deep-mode dimension table (fanout scaffolding)"
```

---

### 任务 2：`merge_findings` 去重器（纯函数）

**文件：**
- 修改：`crates/rustcode-review/src/fanout.rs`
- 测试：位于 `fanout.rs` 的 `#[cfg(test)]`

**接口：**
- 消费：`Finding`（`crate::Finding`）。
- 产出：
  - `pub struct MergedFinding { pub finding: Finding, pub dimensions: Vec<&'static str> }`
  - `pub fn merge_findings(per_dim: Vec<(&'static str, Vec<Finding>)>) -> Vec<MergedFinding>`

- [ ] **步骤 1：编写失败测试**

在 `fanout.rs` 中补充（文件顶部的 `use`）：

```rust
use std::cmp::Ordering;

use crate::Finding;
```

在既有的 `mod tests` 内补充以下测试用例：

```rust
    fn f(priority: &str, conf: f32, file: &str, ls: u32, le: u32, title: &str) -> Finding {
        Finding {
            title: title.into(),
            body: String::new(),
            priority: priority.into(),
            confidence: conf,
            file_path: file.into(),
            line_start: ls,
            line_end: le,
            suggestion: String::new(),
            suggested_code: String::new(),
        }
    }

    #[test]
    fn merge_dedups_same_file_overlapping_range_and_similar_title() {
        let merged = merge_findings(vec![
            ("correctness", vec![f("P1", 0.8, "a.rs", 10, 12, "unchecked unwrap on None")]),
            ("security", vec![f("P2", 0.6, "a.rs", 11, 15, "unwrap on None value")]),
        ]);
        assert_eq!(merged.len(), 1, "overlapping near-duplicate collapses");
        // Higher-priority (P1) content wins; both dimensions are credited.
        assert_eq!(merged[0].finding.priority, "P1");
        assert_eq!(merged[0].dimensions, vec!["correctness", "security"]);
    }

    #[test]
    fn merge_keeps_distinct_findings() {
        let merged = merge_findings(vec![
            ("correctness", vec![f("P1", 0.8, "a.rs", 10, 12, "unchecked unwrap")]),
            ("performance", vec![f("P2", 0.7, "a.rs", 90, 92, "needless clone in loop")]),
            ("security", vec![f("P1", 0.9, "b.rs", 10, 12, "unchecked unwrap")]),
        ]);
        assert_eq!(merged.len(), 3, "different range or file are not duplicates");
    }

    #[test]
    fn merge_prefers_higher_confidence_when_priority_ties() {
        let merged = merge_findings(vec![
            ("correctness", vec![f("P2", 0.5, "a.rs", 1, 1, "same bug title")]),
            ("security", vec![f("P2", 0.9, "a.rs", 1, 1, "same bug title")]),
        ]);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].finding.confidence, 0.9);
    }
```

- [ ] **步骤 2：运行测试确认其失败**

运行：`cargo test -p rustcode-review --lib fanout::tests::merge_`
预期：编译失败 —— `merge_findings` / `MergedFinding` 尚未定义。

- [ ] **步骤 3：编写最小实现**

在 `fanout.rs` 中补充（模块体内，`#[cfg(test)]` 之上）：

```rust
/// A finding that survived dedup, tagged with every dimension that reported it.
pub struct MergedFinding {
    pub finding: Finding,
    pub dimensions: Vec<&'static str>,
}

/// Collapse per-dimension findings into a deduped set. Two findings are the same
/// issue when they touch the same file, their line ranges overlap, and their
/// titles are similar. On a collision the higher-priority (then higher-confidence)
/// finding's content is kept and every contributing dimension is credited.
pub fn merge_findings(per_dim: Vec<(&'static str, Vec<Finding>)>) -> Vec<MergedFinding> {
    let mut merged: Vec<MergedFinding> = Vec::new();
    for (dim, findings) in per_dim {
        for finding in findings {
            match merged.iter_mut().find(|m| is_duplicate(&m.finding, &finding)) {
                Some(existing) => {
                    if !existing.dimensions.contains(&dim) {
                        existing.dimensions.push(dim);
                    }
                    if outranks(&finding, &existing.finding) {
                        existing.finding = finding;
                    }
                }
                None => merged.push(MergedFinding {
                    finding,
                    dimensions: vec![dim],
                }),
            }
        }
    }
    merged
}

fn is_duplicate(a: &Finding, b: &Finding) -> bool {
    same_file(&a.file_path, &b.file_path)
        && ranges_overlap(a.line_start, a.line_end, b.line_start, b.line_end)
        && titles_similar(&a.title, &b.title)
}

fn same_file(a: &str, b: &str) -> bool {
    let na = a.trim_start_matches("./");
    let nb = b.trim_start_matches("./");
    na == nb || na.ends_with(nb) || nb.ends_with(na)
}

fn ranges_overlap(a0: u32, a1: u32, b0: u32, b1: u32) -> bool {
    a0 <= b1 && b0 <= a1
}

/// Title similarity by token-set Jaccard (≥ 0.5), case/punctuation-insensitive.
/// Empty token sets fall back to trimmed case-insensitive equality.
fn titles_similar(a: &str, b: &str) -> bool {
    let ta = title_tokens(a);
    let tb = title_tokens(b);
    if ta.is_empty() || tb.is_empty() {
        return a.trim().eq_ignore_ascii_case(b.trim());
    }
    let inter = ta.iter().filter(|t| tb.contains(*t)).count();
    let union = ta.len() + tb.len() - inter;
    union > 0 && (inter as f32 / union as f32) >= 0.5
}

fn title_tokens(s: &str) -> std::collections::BTreeSet<String> {
    s.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .collect()
}

/// Priority ascending (`P0` most severe), then confidence descending.
fn outranks(candidate: &Finding, current: &Finding) -> bool {
    match candidate.priority.cmp(&current.priority) {
        Ordering::Less => true,
        Ordering::Greater => false,
        Ordering::Equal => candidate.confidence > current.confidence,
    }
}
```

- [ ] **步骤 4：运行测试确认其通过**

运行：`cargo test -p rustcode-review --lib fanout::tests::merge_`
预期：PASS（3 个测试）。

- [ ] **步骤 5：提交**

```bash
git add crates/rustcode-review/src/fanout.rs
git commit -m "feat(review): merge_findings deduplicator for deep mode"
```

---

### 任务 3：共享的 finding 比较器与 deep 渲染器（纯函数）

**文件：**
- 修改：`crates/rustcode-review/src/review_tool.rs`（抽出 `cmp_finding`，并让 `paths_match` 可复用）
- 修改：`crates/rustcode-review/src/fanout.rs`（新增 `DimensionOutcome`、`finalize_deep_review`、`render_deep`）
- 测试：位于 `fanout.rs` 的 `#[cfg(test)]`

**接口：**
- 消费：`merge_findings`、`MergedFinding`、`REVIEW_DIMENSIONS`、`crate::review_tool::{cmp_finding, paths_match}`。
- 产出：
  - `pub struct DimensionOutcome { pub dimension: &'static str, pub findings: Vec<Finding>, pub completed: bool, pub error: Option<String> }`
  - `pub fn finalize_deep_review(outcomes: &[DimensionOutcome], changed_files: usize, changed_paths: &[String]) -> (bool, String)` —— 返回 `(is_error, rendered)`；`is_error` 仅在**没有任何**维度干净完成时为真。

- [ ] **步骤 1：编写失败测试**

首先，在 `crates/rustcode-review/src/review_tool.rs` 中抽出比较器并放宽可见性，让 `fanout` 可以复用它们。替换 `sort_findings` 的函数体，并对外暴露 `cmp_finding` 与 `paths_match`：

```rust
/// Priority ascending (`P0` most severe) then confidence descending. Shared with
/// deep-mode merge ordering.
pub(crate) fn cmp_finding(a: &Finding, b: &Finding) -> std::cmp::Ordering {
    a.priority.cmp(&b.priority).then(
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal),
    )
}

fn sort_findings(findings: &mut [Finding]) {
    findings.sort_by(|a, b| cmp_finding(a, b));
}
```

并把 `fn paths_match(` 改为 `pub(crate) fn paths_match(`（约 668 行）。

然后在 `fanout.rs` 的 `mod tests` 中补充以下测试：

```rust
    fn outcome(dim: &'static str, completed: bool, findings: Vec<Finding>) -> DimensionOutcome {
        DimensionOutcome {
            dimension: dim,
            findings,
            completed,
            error: (!completed).then(|| "boom".to_string()),
        }
    }

    #[test]
    fn finalize_merges_filters_to_changed_files_and_sorts() {
        let outcomes = vec![
            outcome("correctness", true, vec![f("P2", 0.7, "a.rs", 5, 6, "clone in loop")]),
            outcome("security", true, vec![
                f("P0", 0.9, "a.rs", 1, 1, "hardcoded secret"),
                f("P1", 0.8, "not_changed.rs", 1, 1, "ignored"),
            ]),
        ];
        let (is_error, out) = finalize_deep_review(&outcomes, 1, &["a.rs".to_string()]);
        assert!(!is_error);
        // P0 sorts before P2; the non-changed-file finding is dropped.
        let p0 = out.find("hardcoded secret").unwrap();
        let p2 = out.find("clone in loop").unwrap();
        assert!(p0 < p2, "P0 must render before P2:\n{out}");
        assert!(!out.contains("ignored"), "off-scope finding filtered:\n{out}");
        assert!(out.contains("2/4"), "dimension completion summary:\n{out}");
    }

    #[test]
    fn finalize_flags_error_only_when_no_dimension_completed() {
        let all_failed = vec![
            outcome("correctness", false, vec![]),
            outcome("security", false, vec![]),
            outcome("performance", false, vec![]),
            outcome("tests_contracts", false, vec![]),
        ];
        let (is_error, out) = finalize_deep_review(&all_failed, 1, &["a.rs".to_string()]);
        assert!(is_error, "every dimension failed → hard error");
        assert!(out.contains("incomplete") || out.contains("0/4"), "{out}");

        let one_ok = vec![
            outcome("correctness", true, vec![]),
            outcome("security", false, vec![]),
            outcome("performance", false, vec![]),
            outcome("tests_contracts", false, vec![]),
        ];
        let (is_error, _) = finalize_deep_review(&one_ok, 1, &["a.rs".to_string()]);
        assert!(!is_error, "one clean dimension → partial but not a hard error");
    }

    #[test]
    fn finalize_tags_findings_with_contributing_dimensions() {
        let outcomes = vec![
            outcome("correctness", true, vec![f("P1", 0.8, "a.rs", 3, 4, "bad unwrap")]),
            outcome("security", true, vec![f("P1", 0.8, "a.rs", 3, 4, "bad unwrap")]),
        ];
        let (_, out) = finalize_deep_review(&outcomes, 1, &["a.rs".to_string()]);
        assert!(out.contains("correctness") && out.contains("security"), "dims tagged:\n{out}");
    }
```

- [ ] **步骤 2：运行测试确认其失败**

运行：`cargo test -p rustcode-review --lib fanout::tests::finalize_`
预期：编译失败 —— `DimensionOutcome` / `finalize_deep_review` 尚未定义。

- [ ] **步骤 3：编写最小实现**

在 `fanout.rs` 中补充（模块体），并在文件顶部补上所需的 import：

```rust
use crate::review_tool::{cmp_finding, paths_match};
```

```rust
/// Result of running one dimension reviewer. `completed` is true only on a clean
/// finish (agent stopped, no error); a cancelled/errored dimension still
/// contributes whatever findings it already reported.
pub struct DimensionOutcome {
    pub dimension: &'static str,
    pub findings: Vec<Finding>,
    pub completed: bool,
    pub error: Option<String>,
}

/// Merge → scope-filter → sort → render the deep-review outcomes. Returns
/// `(is_error, rendered)`. `is_error` is true only when NO dimension completed
/// cleanly (a fully failed fan-out); a partial run renders its findings and notes
/// coverage.
pub fn finalize_deep_review(
    outcomes: &[DimensionOutcome],
    changed_files: usize,
    changed_paths: &[String],
) -> (bool, String) {
    // Feed merge in stable dimension order regardless of completion order.
    let per_dim: Vec<(&'static str, Vec<Finding>)> = REVIEW_DIMENSIONS
        .iter()
        .filter_map(|d| {
            outcomes
                .iter()
                .find(|o| o.dimension == d.id)
                .map(|o| (d.id, o.findings.clone()))
        })
        .collect();
    let raw_total: usize = per_dim.iter().map(|(_, v)| v.len()).sum();
    let mut merged = merge_findings(per_dim);
    merged.retain(|m| {
        changed_paths
            .iter()
            .any(|cf| paths_match(cf, &m.finding.file_path))
    });
    merged.sort_by(|a, b| cmp_finding(&a.finding, &b.finding));
    let deduped = raw_total.saturating_sub(merged.len());

    let completed: Vec<&'static str> = REVIEW_DIMENSIONS
        .iter()
        .filter(|d| outcomes.iter().any(|o| o.dimension == d.id && o.completed))
        .map(|d| d.id)
        .collect();
    let failed: Vec<&'static str> = REVIEW_DIMENSIONS
        .iter()
        .filter(|d| outcomes.iter().any(|o| o.dimension == d.id && !o.completed))
        .map(|d| d.id)
        .collect();

    let is_error = completed.is_empty();
    let rendered = render_deep(&merged, changed_files, &completed, &failed, deduped, is_error);
    (is_error, rendered)
}

fn render_deep(
    merged: &[MergedFinding],
    changed_files: usize,
    completed: &[&str],
    failed: &[&str],
    deduped: usize,
    is_error: bool,
) -> String {
    let total_dims = REVIEW_DIMENSIONS.len();
    let mut out = String::new();
    if is_error {
        out.push_str(&format!(
            "Deep review incomplete — every dimension failed (0/{total_dims}). \
             Coverage is not reliable.\n"
        ));
    } else if merged.is_empty() {
        out.push_str(&format!(
            "Deep review complete — no issues found across {changed_files} changed file(s) \
             ({}/{total_dims} dimensions completed).\n",
            completed.len()
        ));
    } else {
        out.push_str(&format!(
            "Deep review: {} finding(s) across {changed_files} changed file(s) · \
             {}/{total_dims} dimensions completed",
            merged.len(),
            completed.len()
        ));
        if deduped > 0 {
            out.push_str(&format!(" · deduped {deduped}"));
        }
        out.push('\n');
    }
    if !failed.is_empty() {
        out.push_str(&format!("Failed dimensions: {}\n", failed.join(", ")));
    }
    for (i, m) in merged.iter().enumerate() {
        let f = &m.finding;
        out.push_str(&format!(
            "\n{}. [{} · conf {:.2}] {}:{}-{} · dims: {}\n   {}\n",
            i + 1,
            f.priority,
            f.confidence,
            f.file_path,
            f.line_start,
            f.line_end,
            m.dimensions.join(","),
            f.title.trim()
        ));
        if !f.body.trim().is_empty() {
            out.push_str(&format!("   {}\n", f.body.trim().replace('\n', "\n   ")));
        }
        if !f.suggestion.trim().is_empty() {
            out.push_str(&format!("   ↳ fix: {}\n", f.suggestion.trim().replace('\n', "\n   ")));
        }
    }
    out
}
```

- [ ] **步骤 4：运行测试确认其通过**

运行：`cargo test -p rustcode-review --lib fanout::tests::finalize_`，再运行 `cargo test -p rustcode-review --lib`（确认 `sort_findings`/`paths_match` 的重构没让既有测试变红）。
预期：PASS，无回归。

- [ ] **步骤 5：提交**

```bash
git add crates/rustcode-review/src/fanout.rs crates/rustcode-review/src/review_tool.rs
git commit -m "feat(review): deep-review finalize/merge/render + shared cmp_finding"
```

---

### 任务 4：`run_deep_review` 编排器（并发、可注入）

**文件：**
- 修改：`crates/rustcode-review/src/fanout.rs`
- 测试：位于 `fanout.rs` 的 `#[cfg(test)]`

**接口：**
- 消费：`ReviewDimension`、`REVIEW_DIMENSIONS`、`DimensionOutcome`。
- 产出：
  - `pub async fn run_deep_review<F, Fut>(dims: &'static [ReviewDimension], run_one: F) -> Vec<DimensionOutcome>` where `F: Fn(&'static ReviewDimension) -> Fut`，`Fut: std::future::Future<Output = DimensionOutcome> + Send + 'static`。结果按 `dims` 顺序返回，与完成顺序无关。

- [ ] **步骤 1：编写失败测试**

在 `fanout.rs` 的 `mod tests` 中补充：

```rust
    #[tokio::test]
    async fn run_deep_review_runs_all_dimensions_and_preserves_order() {
        let outcomes = run_deep_review(REVIEW_DIMENSIONS, |dim| {
            let id = dim.id;
            async move {
                DimensionOutcome {
                    dimension: id,
                    findings: vec![f("P2", 0.7, "a.rs", 1, 1, id)],
                    completed: true,
                    error: None,
                }
            }
        })
        .await;
        let ids: Vec<_> = outcomes.iter().map(|o| o.dimension).collect();
        assert_eq!(ids, ["correctness", "security", "performance", "tests_contracts"]);
        assert!(outcomes.iter().all(|o| o.completed && o.findings.len() == 1));
    }
```

- [ ] **步骤 2：运行测试确认其失败**

运行：`cargo test -p rustcode-review --lib fanout::tests::run_deep_review_runs_all`
预期：编译失败 —— `run_deep_review` 尚未定义。

- [ ] **步骤 3：编写最小实现**

在 `fanout.rs` 中补充。使用 `tokio::task::JoinSet`（不引入 `futures` 依赖）。由于 `JoinSet` 按完成顺序产出结果，返回前需按维度索引重新排序：

```rust
/// Run every dimension concurrently and collect their outcomes in `dims` order.
/// `run_one` builds and drives one dimension's reviewer; it must return a
/// `Send + 'static` future (the production runner clones everything it needs).
pub async fn run_deep_review<F, Fut>(
    dims: &'static [ReviewDimension],
    run_one: F,
) -> Vec<DimensionOutcome>
where
    F: Fn(&'static ReviewDimension) -> Fut,
    Fut: std::future::Future<Output = DimensionOutcome> + Send + 'static,
{
    let mut set = tokio::task::JoinSet::new();
    for dim in dims {
        set.spawn(run_one(dim));
    }
    let mut collected: Vec<DimensionOutcome> = Vec::with_capacity(dims.len());
    while let Some(joined) = set.join_next().await {
        if let Ok(outcome) = joined {
            collected.push(outcome);
        }
        // A panicked/aborted task is simply absent; finalize treats a missing
        // dimension as not-completed (it never appears in `completed`).
    }
    // Return in stable dimension order regardless of completion order.
    dims.iter()
        .filter_map(|d| {
            collected
                .iter()
                .position(|o| o.dimension == d.id)
                .map(|i| collected.remove(i))
        })
        .collect()
}
```

- [ ] **步骤 4：运行测试确认其通过**

运行：`cargo test -p rustcode-review --lib fanout::tests::run_deep_review_runs_all`
预期：PASS。

- [ ] **步骤 5：提交**

```bash
git add crates/rustcode-review/src/fanout.rs
git commit -m "feat(review): run_deep_review concurrent orchestrator (JoinSet)"
```

---

### 任务 5：把 `depth` 接入 `code_review` 的 execute 与工具 schema

**文件：**
- 修改：`crates/rustcode-review/src/review_tool.rs`（`Args.depth`、`is_deep`、schema、`execute` 分派、真实的每维度 runner）
- 测试：位于 `review_tool.rs` 的 `#[cfg(test)]`（复用 `ScriptedReviewProvider`）

**接口：**
- 消费：`fanout::{run_deep_review, finalize_deep_review, DimensionOutcome, REVIEW_DIMENSIONS}`、`build_review_agent_with`、`ReviewAgentConfig::with_persona_append`。
- 产出：`code_review` 接受 `{"depth":"deep"}` 并执行 fan-out；默认值与 `"single"` 行为不变。

- [ ] **步骤 1：编写失败测试**

在 `review_tool.rs` 的 `mod tests` 中补充（`ScriptedReviewProvider` 每个 agent 会在 `a.rs:1` 报告一条 `unchecked unwrap` 发现；deep 会跑 4 个 agent → 4 条相同发现 → 去重为 1 条）：

```rust
    #[test]
    fn args_parse_depth_field() {
        let d: Args = serde_json::from_str(r#"{"depth":"deep"}"#).unwrap();
        assert!(d.is_deep());
        let s: Args = serde_json::from_str("{}").unwrap();
        assert!(!s.is_deep());
        let explicit: Args = serde_json::from_str(r#"{"depth":"single"}"#).unwrap();
        assert!(!explicit.is_deep());
    }

    #[tokio::test]
    async fn deep_review_fans_out_and_dedups_across_dimensions() {
        if Command::new("git").arg("--version").output().is_err() {
            return;
        }
        let dir = repo_with_working_tree_change();
        let provider: SharedReviewProvider =
            Arc::new(RwLock::new(Some(Arc::new(ScriptedReviewProvider))));
        let tool = ReviewTool::new(
            provider,
            ReviewToolConfig {
                model: "mock-model".into(),
                ..Default::default()
            },
        );
        let ctx = ToolContext {
            working_dir: dir.path().to_path_buf(),
            cancel: Default::default(),
            progress: ProgressSink::noop(),
            requester: None,
        };

        let res = tool.execute(r#"{"depth":"deep"}"#, &ctx).await;

        assert!(!res.is_error, "deep review should succeed: {}", res.content);
        assert!(
            res.content.contains("Deep review"),
            "deep header present: {}",
            res.content
        );
        // All four dimensions report the same finding → merged to ONE.
        assert!(
            res.content.contains("1 finding(s)") || res.content.contains("1 finding"),
            "identical findings across dimensions must dedup to one: {}",
            res.content
        );
        assert!(
            res.content.contains("dims:"),
            "merged finding is tagged with its dimensions: {}",
            res.content
        );
    }
```

- [ ] **步骤 2：运行测试确认其失败**

运行：`cargo test -p rustcode-review --lib deep_review_fans_out`
预期：编译失败 —— `Args::is_deep` 尚未定义 / 缺少 `depth` 字段。

- [ ] **步骤 3：编写最小实现**

(a) 在 `struct Args` 中新增字段（放在 `confirm_scope` 之后）：

```rust
    /// Review depth. `"deep"` fans out one read-only reviewer per concern
    /// dimension and merges their findings; absent / `"single"` runs the default
    /// single reviewer. Unknown values fall back to single.
    #[serde(default)]
    depth: Option<String>,
```

并在 `impl Args` 中新增辅助方法：

```rust
    fn is_deep(&self) -> bool {
        self.depth
            .as_deref()
            .map(|d| d.eq_ignore_ascii_case("deep"))
            .unwrap_or(false)
    }
```

(b) 把 `depth` 加进工具 schema 的 `parameters()` `properties`（紧邻 `confirm_scope`，约 421 行）：

```rust
                "depth": { "type": "string", "enum": ["single", "deep"], "description": "Review depth. `deep` fans out one reviewer per concern dimension (correctness/security/performance/tests) and merges findings; omit for the default single reviewer." }
```

(c) 在 `review_tool.rs` 顶部导入 fanout 的入口：

```rust
use crate::fanout::{finalize_deep_review, run_deep_review, DimensionOutcome, REVIEW_DIMENSIONS};
```

(d) 在 `execute()` 中，把单 agent 代码块（步骤 3–5，即当前构建单个 `cfg`、调用 `build_review_agent_with`、`tokio::select!` 并渲染的那几行）替换为分派逻辑。共享的准备部分（`annotated`、`files`、`rules`、`impact_plan`、`task`、`provider`）保持原样，然后：

```rust
        // Shared per-agent config seed (both paths).
        let make_cfg = || {
            let mut cfg = ReviewAgentConfig::new("", "", &self.cfg.model, &ctx.working_dir);
            cfg.context_window = self.cfg.context_window;
            cfg.stream_timeout = self.cfg.stream_timeout;
            cfg.request_timeout = self.cfg.request_timeout;
            cfg.max_rounds = self.max_rounds;
            cfg.max_turn_duration = self.max_turn_duration;
            cfg.tool_loop_policy = self.tool_loop_policy;
            cfg.progress = Some(ctx.progress.clone());
            cfg.review_paths = files.clone();
            cfg
        };

        if !a.is_deep() {
            // --- single-agent path (unchanged behavior) ---
            let (agent, report) = build_review_agent_with(&make_cfg(), provider);
            let (stop, run_error) = tokio::select! {
                _ = ctx.cancel.cancelled() => (StopReason::Cancelled, Some("cancelled by user".to_string())),
                outcome = agent.run_to_completion(task, AutoRespond::AllowAll) => {
                    (outcome.stop, outcome.error)
                }
            };
            let mut findings = report.findings();
            findings.retain(|f| files.iter().any(|cf| paths_match(cf, &f.file_path)));
            sort_findings(&mut findings);
            return if stop == StopReason::Stopped && run_error.is_none() {
                ok(render_findings(&findings, files.len()))
            } else {
                err(render_incomplete_review(&findings, files.len(), stop, run_error.as_deref()))
            };
        }

        // --- deep fan-out path ---
        let outcomes = run_deep_review(REVIEW_DIMENSIONS, |dim| {
            // Clone everything so each dimension future is Send + 'static.
            let provider = provider.clone();
            let task = task.clone();
            let mut cfg = make_cfg();
            let cancel = ctx.cancel.clone();
            async move {
                cfg = cfg.with_persona_append(dim.lens);
                let (agent, report) = build_review_agent_with(&cfg, provider);
                let (stop, run_error) = tokio::select! {
                    _ = cancel.cancelled() => (StopReason::Cancelled, Some("cancelled by user".to_string())),
                    outcome = agent.run_to_completion(task, AutoRespond::AllowAll) => {
                        (outcome.stop, outcome.error)
                    }
                };
                DimensionOutcome {
                    dimension: dim.id,
                    findings: report.findings(),
                    completed: stop == StopReason::Stopped && run_error.is_none(),
                    error: run_error,
                }
            }
        })
        .await;

        let (is_error, content) = finalize_deep_review(&outcomes, files.len(), &files);
        return if is_error { err(content) } else { ok(content) };
```

注意：`make_cfg` 借用了 `self`、`ctx`、`files`。deep 闭包在 `async move` **之前**同步调用 `make_cfg()`，因此移入 future 的是生成的所有权 `cfg` —— `self`/`ctx` 的借用不会跨越 await。请确认能编译；若借用检查器对闭包中的 `make_cfg` 报错，就把 cfg 构造内联到闭包体内、放在 `async move` 之前（先构建出所有权 `cfg`，再以 `let cfg = cfg;` 移入）。

- [ ] **步骤 4：运行测试确认其通过**

运行：`cargo test -p rustcode-review --lib deep_review_fans_out args_parse_depth_field`
然后跑整个 crate：`cargo test -p rustcode-review`。
预期：PASS；既有的单路径测试（`review_tool_reviews_a_real_diff`、轮次/时长测试）仍然通过。

- [ ] **步骤 5：提交**

```bash
git add crates/rustcode-review/src/review_tool.rs
git commit -m "feat(review): code_review deep depth arg → dimension fan-out"
```

---

### 任务 6：`/review deep` 命令映射

**文件：**
- 修改：`crates/rustcode-tuix/src/event_loop/commands.rs`（`review_prompt`）
- 测试：位于 `commands.rs` 的 `#[cfg(test)]`

**接口：**
- 消费：无新增。
- 产出：`/review deep [scope]` 合成携带 `"depth":"deep"` 的工具调用；`/review [scope]` 不变。

- [ ] **步骤 1：编写失败测试**

在 `commands.rs` 的 `mod tests` 中补充（靠近 `review_prompt_uses_explicit_tool_scopes`）：

```rust
    #[test]
    fn review_prompt_deep_adds_depth_and_keeps_scope() {
        // `deep` alone → working-tree + depth.
        let wt = review_prompt("deep");
        assert!(wt.contains(r#""scope":{"kind":"working_tree"}"#), "{wt}");
        assert!(wt.contains(r#""depth":"deep""#), "{wt}");

        // `deep staged` → staged + depth.
        let st = review_prompt("deep staged");
        assert!(st.contains(r#""scope":{"kind":"staged"}"#), "{st}");
        assert!(st.contains(r#""depth":"deep""#), "{st}");

        // `deep <ref>` → range + depth.
        let rng = review_prompt("deep main");
        assert!(rng.contains(r#""scope":{"kind":"range","base":"main","head":"HEAD"}"#), "{rng}");
        assert!(rng.contains(r#""depth":"deep""#), "{rng}");

        // Plain scope carries NO depth (default single).
        assert!(!review_prompt("").contains("depth"));
        assert!(!review_prompt("staged").contains("depth"));
    }
```

- [ ] **步骤 2：运行测试确认其失败**

运行：`cargo test -p rustcode-tuix --lib review_prompt_deep_adds_depth`
预期：FAIL（当前 `review_prompt` 不输出 `depth`，且把 `deep` 当作 git ref 处理）。

- [ ] **步骤 3：编写最小实现**

把 `review_prompt`（commands.rs:90）替换为能解析前导 `deep` 关键字并组装工具参数对象的版本：

```rust
fn review_prompt(arg: &str) -> String {
    let arg = arg.trim();
    // A leading `deep` keyword (alone or before a scope) opts into deep mode.
    let (deep, scope) = match arg.strip_prefix("deep") {
        Some(rest) if rest.is_empty() || rest.starts_with(char::is_whitespace) => (true, rest.trim()),
        _ => (false, arg),
    };
    let scope_json = if scope.is_empty() {
        r#"{"kind":"working_tree"}"#.to_string()
    } else if scope.eq_ignore_ascii_case("staged") {
        r#"{"kind":"staged"}"#.to_string()
    } else {
        format!(
            r#"{{"kind":"range","base":{base},"head":"HEAD"}}"#,
            base = serde_json::to_string(scope).expect("serializing a string cannot fail")
        )
    };
    let args = if deep {
        format!(r#"{{"scope":{scope_json},"depth":"deep"}}"#)
    } else {
        format!(r#"{{"scope":{scope_json}}}"#)
    };
    format!(
        "Review the requested changes: call the `code_review` tool with {args}, then give me a \
         concise summary of its findings."
    )
}
```

注意：该实现保留了当前测试断言的那些既有子串（`{"scope":{"kind":"working_tree"}}`、`{"scope":{"kind":"staged"}}`、`{"scope":{"kind":"range","base":"release/v5.0.9","head":"HEAD"}}`，以及经 JSON 转义的奇特 ref），因此 `review_prompt_uses_explicit_tool_scopes` 与 `review_prompt_json_escapes_the_base_ref` 仍会通过。

- [ ] **步骤 4：运行测试确认其通过**

运行：`cargo test -p rustcode-tuix --lib review_prompt`
预期：PASS（新增的 deep 测试 + 既有的两个 review_prompt 测试）。

- [ ] **步骤 5：提交**

```bash
git add crates/rustcode-tuix/src/event_loop/commands.rs
git commit -m "feat(tuix): /review deep opts code_review into dimension fan-out"
```

---

### 任务 7：全量回归 + 文档备注

**文件：**
- 修改：`crates/rustcode-review/src/review_tool.rs`（模块级 deep 模式文档备注 —— 2 行）
- 不新增测试（本任务为验证任务）。

- [ ] **步骤 1：补充文档备注**

在 `review_tool.rs` 模块头部末尾（约 12 行之后）补充：

```rust
//! Deep mode: passing `{"depth":"deep"}` fans out one read-only reviewer per
//! concern dimension (see `crate::fanout`) and merges/dedups their findings; the
//! default single-reviewer path is unchanged.
```

- [ ] **步骤 2：运行全部相关测试套件**

运行：
```bash
cargo test -p rustcode-review
cargo test -p rustcode-tuix --lib
cargo build -p rustcode-review -p rustcode-tuix
```
预期：全绿，零告警。

- [ ] **步骤 3：提交**

```bash
git add crates/rustcode-review/src/review_tool.rs
git commit -m "docs(review): note deep-mode fan-out in code_review header"
```

---

## 自检

**规范覆盖：**
- 接口 `/review deep` + 工具 `depth` 参数 → 任务 5（参数/schema/分派）、任务 6（命令）。
- 维度 fan-out（correctness/security/performance/tests_contracts，通过 `persona_append` 施加全量 diff 视角）→ 任务 1（维度表）、任务 5（runner 使用 `with_persona_append`）。
- 并发执行、可取消、使用 `JoinSet`（生产环境不引入 `futures` 依赖）→ 任务 4、任务 5。
- 按（文件 + 重叠行范围 + 相似标题）合并/去重，保留更高优先级/置信度的一方，并累积维度标签 → 任务 2。
- 范围预检在 fan-out 之前 → 任务 5 把既有的 `ScopeManifest` 代码块保留在分派之前（共享准备部分未动）。
- 错误处理：部分维度仍然贡献结果；仅当全部未完成时才是硬错误 → 任务 3（`finalize_deep_review` 的 is_error 规则）+ 任务 5（`completed` 标志）。
- 报告：复用排序 + 每维度汇总 + 维度标签 → 任务 3（`render_deep`）。
- 默认 single 路径不变 → 任务 5 的分派原样返回原有代码块。
- Phase-2 verify 预留（未实现）→ 按设计不在范围内；`depth` 枚举可后续扩展。

**占位符扫描：** 无 —— 每个步骤都有具体代码。

**类型一致性：** `Finding` 字段与 capabilities 结构体逐字一致；`cmp_finding`/`paths_match` 在任务 3 定义并在任务 3 的 `finalize_deep_review` 中消费；`DimensionOutcome`/`MergedFinding`/`run_deep_review`/`finalize_deep_review` 在产出方（任务 2–4）与消费方（任务 5）的签名完全一致。`is_deep` 在任务 5 定义并使用。
