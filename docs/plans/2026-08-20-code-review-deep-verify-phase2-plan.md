# code-review deep+verify（Phase 2）实施计划

> **面向 agentic worker：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实施本计划。步骤使用复选框（`- [ ]`）语法跟踪进度。

**目标：** 新增可选的 `depth:"deep+verify"`，从 deep 模式合并后的发现中剔除误报 —— 每条存活的发现配一个 verify agent，单票判定且偏向保留 —— 同时 `single` 与 `deep` 保持不变。

**架构：** 把 Phase 1 位于 `rustcode-review/src/fanout.rs` 的 `finalize_deep_review` 拆成可复用的几块（`merge_deep_findings`、`dimension_coverage`、带可选 `verify_dropped` 计数的 `render_deep_result`）；新增 verifier 人设视角、一个有并发上限的 keep-mask runner `run_verify`，以及辅助函数 `render_verify_task`。`review_tool.rs` 新增 `wants_verify()`、schema 枚举值，以及一个在 merge 与 render 之间执行 verify 的 `deep+verify` 分支。verify 复用 `build_review_agent_with` + `report_finding` —— verify agent 重新报告该发现即保留；未报告任何内容即丢弃；出错/取消则保留（fail-open）。

**技术栈：** Rust、tokio（`rt-multi-thread`、`sync`、`macros` —— 均已启用）、`rustcode-kernel` 的 Agent、`rustcode-capabilities` 的 `Finding`/`ReportFindingTool`。不引入新依赖。

**规范：** `docs/plans/2026-08-20-code-review-deep-mode-fanout-design.md`（§ "Phase 2 — adversarial verify pass" 一节）。

## 全局约束

- 不新增 crate 依赖。并发使用 `tokio::task::JoinSet`（而非 `futures`）。
- `single` 与 `deep` 路径保持行为一致。`rustcode-review` 与 `rustcode-tuix` 的所有既有测试保持通过且不改动；特别是重构之后，无 verify 路径下 `finalize_deep_review` 的输出必须逐字节相同（它委托时传入 `verify_dropped = None`）。
- verify 仅按需开启（只有 `depth:"deep+verify"`）。单票判定偏向 KEEP：只有当某条发现的 verify agent 干净完成**且**未重新报告任何内容时，该发现才会被丢弃。出错/取消/panic 一律保留该发现（fail-open）。
- 范围预检仍保持在 fan-out 之前（不变）。
- 发现以英文渲染。
- `Finding` 字段（来自 `rustcode-capabilities`，禁止修改）：`title: String, body: String, priority: String ("P0".."P3"), confidence: f32, file_path: String, line_start: u32, line_end: u32, suggestion: String, suggested_code: String`。
- 当前相关地标（Phase 1，已合入）：`fanout.rs` 中有 `finalize_deep_review`（约 151 行）、`render_deep`（约 192 行）、`MergedFinding`/`DimensionOutcome`、`merge_findings`、`run_deep_review`；`review_tool.rs` 中有 `Args.depth` + `is_deep()`（约 307 行）、schema 的 `depth` 条目（约 439 行），以及调用 `finalize_deep_review` 的 deep 分支（约 567 行）。`annotated`、`files`、`rules`、`task` 在 deep 分支处已在作用域内。

---

### 任务 1：拆分 finalize，并新增 verifier 视角、run_verify 与 verify 任务辅助函数（fanout.rs）

**文件：**
- 修改：`crates/rustcode-review/src/fanout.rs`
- 修改：`crates/rustcode-review/src/lib.rs`（导出新增项）
- 测试：位于 `fanout.rs` 的 `#[cfg(test)]`

**接口：**
- 消费：`MergedFinding`、`DimensionOutcome`、`merge_findings`、`REVIEW_DIMENSIONS`、`crate::Finding`、`crate::review_tool::{cmp_finding, paths_match}`（本文件已在使用）。
- 产出：
  - `pub fn merge_deep_findings(outcomes: &[DimensionOutcome], changed_paths: &[String]) -> (Vec<MergedFinding>, usize)`
  - `pub fn dimension_coverage(outcomes: &[DimensionOutcome]) -> (Vec<&'static str>, Vec<&'static str>)`
  - `pub fn render_deep_result(merged: &[MergedFinding], changed_files: usize, completed: &[&str], failed: &[&str], deduped: usize, verify_dropped: Option<usize>) -> (bool, String)`
  - `pub const VERIFY_LENS: &str`, `pub const VERIFY_CONCURRENCY: usize`
  - `pub fn render_verify_task(f: &Finding, rules: &str, annotated: &str) -> String`
  - `pub async fn run_verify<F, Fut>(n: usize, cap: usize, verify_one: F) -> Vec<bool>`，其中 `F: Fn(usize) -> Fut`、`Fut: Future<Output = (usize, bool)> + Send + 'static`

- [ ] **步骤 1：编写失败测试**

在 `fanout.rs` 的 `mod tests` 中补充（`f(...)` 辅助函数已存在于此）：

```rust
    #[test]
    fn finalize_output_is_unchanged_after_the_split() {
        // The no-verify path must be byte-identical to Phase 1's behavior.
        let outcomes = vec![
            DimensionOutcome { dimension: "correctness", findings: vec![f("P1", 0.8, "a.rs", 3, 4, "bad unwrap")], completed: true, error: None },
            DimensionOutcome { dimension: "security", findings: vec![], completed: true, error: None },
            DimensionOutcome { dimension: "performance", findings: vec![], completed: false, error: Some("boom".into()) },
            DimensionOutcome { dimension: "tests_contracts", findings: vec![], completed: true, error: None },
        ];
        let (err_a, out_a) = finalize_deep_review(&outcomes, 1, &["a.rs".to_string()]);
        let (merged, deduped) = merge_deep_findings(&outcomes, &["a.rs".to_string()]);
        let (completed, failed) = dimension_coverage(&outcomes);
        let (err_b, out_b) = render_deep_result(&merged, 1, &completed, &failed, deduped, None);
        assert_eq!((err_a, out_a), (err_b, out_b), "finalize must equal its decomposed form with verify_dropped=None");
    }

    #[test]
    fn render_deep_result_notes_verify_dropped() {
        let merged = vec![]; // all survivors culled
        let (_e, out) = render_deep_result(&merged, 1, &["correctness"], &[], 0, Some(2));
        assert!(out.contains("verify"), "verify note present: {out}");
        assert!(out.contains('2'), "dropped count present: {out}");
    }

    #[tokio::test]
    async fn run_verify_applies_keep_mask_in_order_with_a_small_cap() {
        // Keep evens, drop odds; cap < n exercises the refill path.
        let keep = run_verify(5, 2, |i| async move { (i, i % 2 == 0) }).await;
        assert_eq!(keep, vec![true, false, true, false, true]);
    }

    #[test]
    fn verify_task_embeds_the_candidate_and_diff() {
        let finding = f("P1", 0.9, "a.rs", 10, 12, "unchecked unwrap");
        let task = render_verify_task(&finding, "RULES-HERE", "DIFF-HERE");
        assert!(task.contains("unchecked unwrap") && task.contains("a.rs:10-12"));
        assert!(task.contains("RULES-HERE") && task.contains("DIFF-HERE"));
        assert!(task.to_lowercase().contains("verify"));
    }
```

- [ ] **步骤 2：运行测试确认它们失败**

运行：`cargo test -p rustcode-review --lib fanout::tests::finalize_output_is_unchanged_after_the_split fanout::tests::render_deep_result_notes_verify_dropped fanout::tests::run_verify_applies fanout::tests::verify_task_embeds`
预期：编译失败 —— 新函数/常量尚不存在。

- [ ] **步骤 3：编写实现**

在 `fanout.rs` 中，把 `finalize_deep_review` 的函数体（当前约 151–190 行）改为委托实现，并新增各项。同时修改 `render_deep` 的签名，使其接受 `verify_dropped: Option<usize>`。

把 `finalize_deep_review` 替换为：

```rust
pub fn finalize_deep_review(
    outcomes: &[DimensionOutcome],
    changed_files: usize,
    changed_paths: &[String],
) -> (bool, String) {
    let (merged, deduped) = merge_deep_findings(outcomes, changed_paths);
    let (completed, failed) = dimension_coverage(outcomes);
    render_deep_result(&merged, changed_files, &completed, &failed, deduped, None)
}

/// Merge → scope-filter → sort the fan-out outcomes into the deduped survivor
/// set, plus the count collapsed by dedup. Shared by the deep and deep+verify
/// paths.
pub fn merge_deep_findings(
    outcomes: &[DimensionOutcome],
    changed_paths: &[String],
) -> (Vec<MergedFinding>, usize) {
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
    (merged, deduped)
}

/// Completed vs failed dimension ids, in table order. A dimension absent from
/// `outcomes` counts as neither — but never enters `completed`, so `is_error`
/// stays correct.
pub fn dimension_coverage(
    outcomes: &[DimensionOutcome],
) -> (Vec<&'static str>, Vec<&'static str>) {
    let completed = REVIEW_DIMENSIONS
        .iter()
        .filter(|d| outcomes.iter().any(|o| o.dimension == d.id && o.completed))
        .map(|d| d.id)
        .collect();
    let failed = REVIEW_DIMENSIONS
        .iter()
        .filter(|d| outcomes.iter().any(|o| o.dimension == d.id && !o.completed))
        .map(|d| d.id)
        .collect();
    (completed, failed)
}

/// Render the merged (post-verify, if any) survivor set. `verify_dropped` adds a
/// "verify dropped K" note when `Some`. Returns `(is_error, rendered)`;
/// `is_error` is true only when no dimension completed cleanly.
pub fn render_deep_result(
    merged: &[MergedFinding],
    changed_files: usize,
    completed: &[&str],
    failed: &[&str],
    deduped: usize,
    verify_dropped: Option<usize>,
) -> (bool, String) {
    let is_error = completed.is_empty();
    let rendered = render_deep(
        merged,
        changed_files,
        completed,
        failed,
        deduped,
        is_error,
        verify_dropped,
    );
    (is_error, rendered)
}
```

修改既有的 `fn render_deep(...)` 签名，加上末尾参数并输出该提示。它当前用于构建头部信息的代码块（约 200–224 行的 `if is_error {..} else if merged.is_empty() {..} else {..}`）变为：

```rust
fn render_deep(
    merged: &[MergedFinding],
    changed_files: usize,
    completed: &[&str],
    failed: &[&str],
    deduped: usize,
    is_error: bool,
    verify_dropped: Option<usize>,
) -> String {
    let total_dims = REVIEW_DIMENSIONS.len();
    let verify_note = match verify_dropped {
        Some(k) => format!(" · verify dropped {k}"),
        None => String::new(),
    };
    let mut out = String::new();
    if is_error {
        out.push_str(&format!(
            "Deep review incomplete — every dimension failed (0/{total_dims}). \
             Coverage is not reliable.{verify_note}\n"
        ));
    } else if merged.is_empty() {
        out.push_str(&format!(
            "Deep review complete — no issues found across {changed_files} changed file(s) \
             ({}/{total_dims} dimensions completed){verify_note}.\n",
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
        out.push_str(&verify_note);
        out.push('\n');
    }
    // ... the rest (failed line + the per-finding loop) is UNCHANGED ...
```

`render_deep` 的其余部分（`if !failed.is_empty()` 那一行与 `for (i, m) in merged.iter()...` 循环）保持原样。

注意：`verify_dropped = None` 会生成空的 `verify_note`，因此 `finalize_deep_review` 的输出与 Phase 1 逐字节相同 —— `finalize_output_is_unchanged_after_the_split` 测试即断言这一点。

然后在模块体末尾（`#[cfg(test)]` 之前）补充 verify 机制：

```rust
/// Concurrency cap for the verify pass (one agent per surviving finding).
pub const VERIFY_CONCURRENCY: usize = 6;

/// Persona lens appended to the base reviewer persona for a verify agent.
pub const VERIFY_LENS: &str = "\n\n## This review's task: VERIFY ONE CANDIDATE FINDING\n\
You are checking a single candidate finding from a prior review pass. Using the DIFF as the \
authoritative source (plus read-only tools for context), decide whether it is a REAL defect \
INTRODUCED by these changes. If it is real — OR if you are unsure — call `report_finding` to \
re-report it (you may refine its wording). Report NOTHING only when you are confident it is a \
false positive, is not introduced by this diff, or is already handled, and briefly say why. Do \
not hunt for new, unrelated issues.";

/// Build the single-finding task text handed to a verify agent: the candidate,
/// the per-language rules, and the authoritative DIFF.
pub fn render_verify_task(f: &Finding, rules: &str, annotated: &str) -> String {
    format!(
        "Verify the following single candidate finding from a prior review.\n\n\
         CANDIDATE FINDING:\n\
         - [{} · conf {:.2}] {}:{}-{}\n  {}\n  {}\n\n{rules}\n\n=== DIFF ===\n{annotated}",
        f.priority,
        f.confidence,
        f.file_path,
        f.line_start,
        f.line_end,
        f.title.trim(),
        f.body.trim(),
    )
}

/// Run up to `cap` verify checks concurrently over `n` items; return a keep-mask
/// in index order. `verify_one(i)` yields `(i, keep)`. The default is `true`
/// (fail-open): a panicked/aborted verify task leaves its finding kept.
pub async fn run_verify<F, Fut>(n: usize, cap: usize, verify_one: F) -> Vec<bool>
where
    F: Fn(usize) -> Fut,
    Fut: std::future::Future<Output = (usize, bool)> + Send + 'static,
{
    let cap = cap.max(1);
    let mut keep = vec![true; n];
    let mut set = tokio::task::JoinSet::new();
    let mut next = 0usize;
    while next < n && set.len() < cap {
        set.spawn(verify_one(next));
        next += 1;
    }
    while let Some(joined) = set.join_next().await {
        if let Ok((i, kept)) = joined {
            if i < n {
                keep[i] = kept;
            }
        }
        if next < n {
            set.spawn(verify_one(next));
            next += 1;
        }
    }
    keep
}
```

在 `lib.rs` 中扩展 fanout 的再导出，纳入新增的公开项：

```rust
pub use fanout::{
    dimension_coverage, merge_deep_findings, render_deep_result, render_verify_task, run_verify,
    ReviewDimension, DimensionOutcome, MergedFinding, REVIEW_DIMENSIONS, VERIFY_CONCURRENCY,
    VERIFY_LENS,
};
```
（保留已再导出的 fanout 项，只做追加。若 `finalize_deep_review`/`run_deep_review` 已导出，保持不动。）

- [ ] **步骤 4：运行测试确认它们通过**

运行：`cargo test -p rustcode-review --lib fanout`，再运行 `cargo test -p rustcode-review`。
预期：PASS，包括既有的 `finalize_*` / `run_deep_review_*` 测试（输出不变）。

- [ ] **步骤 5：提交**

```bash
git add crates/rustcode-review/src/fanout.rs crates/rustcode-review/src/lib.rs
git commit -m "feat(review): split deep finalize + add verify lens/runner (phase 2 scaffolding)"
```

---

### 任务 2：把 `deep+verify` 接入 `code_review` 的 execute（review_tool.rs）

**文件：**
- 修改：`crates/rustcode-review/src/review_tool.rs`
- 测试：位于 `review_tool.rs` 的 `#[cfg(test)]`

**接口：**
- 消费：`fanout::{merge_deep_findings, dimension_coverage, render_deep_result, render_verify_task, run_verify, VERIFY_LENS, VERIFY_CONCURRENCY}`，以及已导入的 `run_deep_review`、`DimensionOutcome`、`REVIEW_DIMENSIONS`。
- 产出：`code_review` 接受 `{"depth":"deep+verify"}`；新增 `Args::wants_verify()`；schema 枚举扩展。

- [ ] **步骤 1：编写失败测试**

在 `review_tool.rs` 的 `mod tests` 中补充：

```rust
    #[test]
    fn args_parse_deep_verify_depth() {
        let v: Args = serde_json::from_str(r#"{"depth":"deep+verify"}"#).unwrap();
        assert!(v.is_deep(), "deep+verify still counts as deep (fans out)");
        assert!(v.wants_verify());
        let d: Args = serde_json::from_str(r#"{"depth":"deep"}"#).unwrap();
        assert!(d.is_deep() && !d.wants_verify());
        let s: Args = serde_json::from_str("{}").unwrap();
        assert!(!s.is_deep() && !s.wants_verify());
    }

    #[tokio::test]
    async fn deep_verify_keeps_a_confirmed_finding() {
        if Command::new("git").arg("--version").output().is_err() {
            return;
        }
        let dir = repo_with_working_tree_change();
        let provider: SharedReviewProvider =
            Arc::new(RwLock::new(Some(Arc::new(ScriptedReviewProvider))));
        let tool = ReviewTool::new(
            provider,
            ReviewToolConfig { model: "mock-model".into(), ..Default::default() },
        );
        let ctx = ToolContext {
            working_dir: dir.path().to_path_buf(),
            cancel: Default::default(),
            progress: ProgressSink::noop(),
            requester: None,
        };

        let res = tool.execute(r#"{"depth":"deep+verify"}"#, &ctx).await;

        // 4 dimensions report the same finding → merged to 1; each finding's
        // verify agent (ScriptedReviewProvider) re-reports it → kept, dropped 0.
        assert!(!res.is_error, "deep+verify should succeed: {}", res.content);
        assert!(res.content.contains("Deep review"), "deep header: {}", res.content);
        assert!(res.content.contains("verify dropped 0"), "verify note, nothing culled: {}", res.content);
        assert!(res.content.contains("1 finding"), "the confirmed finding survives: {}", res.content);
    }
```

- [ ] **步骤 2：运行测试确认它们失败**

运行：`cargo test -p rustcode-review --lib args_parse_deep_verify_depth deep_verify_keeps_a_confirmed_finding`
预期：编译失败 —— 缺少 `wants_verify` / 未处理 `deep+verify`。

- [ ] **步骤 3：编写实现**

(a) 在 `impl Args` 中更新 `is_deep` 并新增 `wants_verify`（替换既有的 `is_deep`）：

```rust
    fn is_deep(&self) -> bool {
        self.depth
            .as_deref()
            .map(|d| d.eq_ignore_ascii_case("deep") || d.eq_ignore_ascii_case("deep+verify"))
            .unwrap_or(false)
    }

    fn wants_verify(&self) -> bool {
        self.depth
            .as_deref()
            .map(|d| d.eq_ignore_ascii_case("deep+verify"))
            .unwrap_or(false)
    }
```

(b) 更新 import（约 36 行的 `use crate::fanout::...`）以纳入新增项：

```rust
use crate::fanout::{
    dimension_coverage, merge_deep_findings, render_deep_result, render_verify_task, run_deep_review,
    run_verify, DimensionOutcome, REVIEW_DIMENSIONS, VERIFY_CONCURRENCY, VERIFY_LENS,
};
```
（若 `finalize_deep_review` 已不再被引用，就从 import 中去掉 —— deep 分支现在使用 `merge_deep_findings`/`render_deep_result`。只有在别处仍被使用时才保留其导入。）

(c) 更新 schema 的 `depth` 条目（约 439 行）为新的枚举与描述：

```rust
                "depth": { "type": "string", "enum": ["single", "deep", "deep+verify"], "description": "Review depth. `deep` fans out one reviewer per concern dimension (correctness/security/performance/tests) and merges findings; `deep+verify` additionally runs one verify pass per finding to cull false positives; omit for the default single reviewer." }
```

(d) 把 deep 路径的尾部（当前约 567–568 行的两行 `let (is_error, content) = finalize_deep_review(&outcomes, files.len(), &files); if is_error { err(content) } else { ok(content) }`）替换为 merge → 可选 verify → render 的序列：

```rust
        // Merge the fan-out outcomes; optionally cull false positives with a
        // single verify pass per surviving finding.
        let (mut merged, deduped) = merge_deep_findings(&outcomes, &files);
        let (completed, failed) = dimension_coverage(&outcomes);
        let mut verify_dropped = None;
        if a.wants_verify() && !merged.is_empty() {
            // One verify agent per finding, capped. Keep a finding when its
            // verify agent re-reports it (or fails open on error/cancel).
            let inputs: Vec<String> = merged
                .iter()
                .map(|m| render_verify_task(&m.finding, &rules, &annotated))
                .collect();
            let keep = run_verify(merged.len(), VERIFY_CONCURRENCY, |i| {
                let provider = provider.clone();
                let vtask = inputs[i].clone();
                let mut cfg = make_cfg();
                let cancel = ctx.cancel.clone();
                async move {
                    cfg = cfg.with_persona_append(VERIFY_LENS);
                    let (agent, report) = build_review_agent_with(&cfg, provider);
                    let (stop, run_error) = tokio::select! {
                        _ = cancel.cancelled() => (StopReason::Cancelled, Some("cancelled by user".to_string())),
                        outcome = agent.run_to_completion(vtask, AutoRespond::AllowAll) => {
                            (outcome.stop, outcome.error)
                        }
                    };
                    // Fail-open: keep on error/cancel; else keep iff the verifier re-reported.
                    let clean = stop == StopReason::Stopped && run_error.is_none();
                    let kept = if clean { !report.findings().is_empty() } else { true };
                    (i, kept)
                }
            })
            .await;
            let before = merged.len();
            let mut mask = keep.into_iter();
            merged.retain(|_| mask.next().unwrap_or(true));
            verify_dropped = Some(before - merged.len());
        }
        let (is_error, content) =
            render_deep_result(&merged, files.len(), &completed, &failed, deduped, verify_dropped);
        if is_error { err(content) } else { ok(content) }
```

关于闭包的说明：`render_verify_task` 与 `make_cfg()` 在 `async move` **之前**同步调用（生成所有权的 `String`/`cfg`），且 `provider`/`cancel` 都被克隆 —— 因此每个 verify future 都是 `Send + 'static`，与维度闭包完全一样。`inputs[i].clone()` 读取的是局部变量 `inputs`（它跨 `.await` 保持存活）。**不要**削弱 `run_verify` 的约束；若借用检查器与该闭包冲突，就照搬维度闭包的结构。若经过认真尝试仍无法满足 `Send + 'static`，请停下并报告 BLOCKED，附上确切的报错信息。

- [ ] **步骤 4：运行测试确认它们通过**

运行：`cargo test -p rustcode-review --lib args_parse_deep_verify_depth deep_verify_keeps_a_confirmed_finding`，再运行 `cargo test -p rustcode-review`。
预期：PASS；所有 Phase 1 测试（含 `deep_review_fans_out_and_dedups_across_dimensions` 与单路径测试）保持通过。

- [ ] **步骤 5：提交**

```bash
git add crates/rustcode-review/src/review_tool.rs
git commit -m "feat(review): code_review deep+verify runs one verify pass per finding"
```

---

### 任务 3：`/review deep+verify` 命令映射（commands.rs）

**文件：**
- 修改：`crates/rustcode-tuix/src/event_loop/commands.rs`（`review_prompt`）
- 测试：位于 `commands.rs` 的 `#[cfg(test)]`

**接口：**
- 产出：`/review deep+verify [scope]` 合成携带 `"depth":"deep+verify"` 的工具调用；`deep` 与普通 scope 不变。

- [ ] **步骤 1：编写失败测试**

在 `commands.rs` 的 `mod tests` 中补充：

```rust
    #[test]
    fn review_prompt_deep_verify_sets_depth_and_keeps_scope() {
        let wt = review_prompt("deep+verify");
        assert!(wt.contains(r#""scope":{"kind":"working_tree"}"#), "{wt}");
        assert!(wt.contains(r#""depth":"deep+verify""#), "{wt}");

        let st = review_prompt("deep+verify staged");
        assert!(st.contains(r#""scope":{"kind":"staged"}"#), "{st}");
        assert!(st.contains(r#""depth":"deep+verify""#), "{st}");

        let rng = review_prompt("deep+verify main");
        assert!(rng.contains(r#""scope":{"kind":"range","base":"main","head":"HEAD"}"#), "{rng}");
        assert!(rng.contains(r#""depth":"deep+verify""#), "{rng}");

        // Plain `deep` still maps to depth "deep" (not deep+verify).
        let d = review_prompt("deep");
        assert!(d.contains(r#""depth":"deep""#) && !d.contains("deep+verify"), "{d}");
    }
```

- [ ] **步骤 2：运行测试确认其失败**

运行：`cargo test -p rustcode-tuix --lib review_prompt_deep_verify_sets_depth`
预期：FAIL —— 当前 `deep+verify` 会被当成 git ref 解析，输出一个不带 depth 的 range scope。

- [ ] **步骤 3：编写实现**

把 `review_prompt` 内部的前导关键字解析与参数对象构建（当前的 `let (deep, scope) = match arg.strip_prefix("deep") {...};` 块与 `let args = if deep {...} else {...};` 块）替换为可感知 depth 的版本，先判 `deep+verify` 再判 `deep`：

```rust
    // A leading `deep+verify` or `deep` keyword (alone or before a scope) sets depth.
    let (depth, scope): (Option<&str>, &str) = if let Some(rest) = arg
        .strip_prefix("deep+verify")
        .filter(|r| r.is_empty() || r.starts_with(char::is_whitespace))
    {
        (Some("deep+verify"), rest.trim())
    } else if let Some(rest) = arg
        .strip_prefix("deep")
        .filter(|r| r.is_empty() || r.starts_with(char::is_whitespace))
    {
        (Some("deep"), rest.trim())
    } else {
        (None, arg)
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
    let args = match depth {
        Some(d) => format!(r#"{{"scope":{scope_json},"depth":"{d}"}}"#),
        None => format!(r#"{{"scope":{scope_json}}}"#),
    };
```

保持结尾那行 `format!("Review the requested changes: call the `code_review` tool with {args}, ...")` 不变。这样可以保留既有测试断言的每一个子串（普通 scope → 无 `depth`；`deep` → `"depth":"deep"`；range 的 JSON 转义仍由 `serde_json::to_string` 负责）。

- [ ] **步骤 4：运行测试确认其通过**

运行：`cargo test -p rustcode-tuix --lib review_prompt`
预期：PASS —— 新增的 deep+verify 测试，以及既有的 `review_prompt_uses_explicit_tool_scopes`、`review_prompt_json_escapes_the_base_ref`、`review_prompt_deep_adds_depth_and_keeps_scope` 全部通过。

- [ ] **步骤 5：提交**

```bash
git add crates/rustcode-tuix/src/event_loop/commands.rs
git commit -m "feat(tuix): /review deep+verify maps to depth deep+verify"
```

---

### 任务 4：全量回归 + 文档备注

**文件：**
- 修改：`crates/rustcode-review/src/review_tool.rs`（扩展模块头部备注）

- [ ] **步骤 1：扩展文档备注**

在 `review_tool.rs` 模块头部，更新 deep 模式备注以提及 verify：

```rust
//! Deep mode: `{"depth":"deep"}` fans out one read-only reviewer per concern
//! dimension (see `crate::fanout`) and merges/dedups their findings;
//! `{"depth":"deep+verify"}` additionally runs one verify pass per finding to
//! cull false positives (single vote, biased toward keep). The default
//! single-reviewer path is unchanged.
```

- [ ] **步骤 2：运行测试套件**

```bash
cargo test -p rustcode-review
cargo test -p rustcode-tuix --lib
cargo build -p rustcode-review -p rustcode-tuix
```
预期：全绿，零新增告警。

- [ ] **步骤 3：提交**

```bash
git add crates/rustcode-review/src/review_tool.rs
git commit -m "docs(review): note deep+verify pass in code_review header"
```

---

## 自检

**规范覆盖：**
- `depth:"deep+verify"` 触发，且两种深度下 `is_deep` 均为真，另有 `wants_verify` → 任务 2（Args）、任务 3（命令）。
- verify 位于 merge 与 render 之间 → 任务 2（deep 分支：merge_deep_findings → run_verify → render_deep_result）。
- verify agent 复用 `build_review_agent_with` + `report_finding`；keep = 被重新报告；drop = 未报告任何内容；出错/取消时 fail-open → 任务 2 的闭包（`clean` 判定；`unwrap_or(true)`）、任务 1 中默认为 true 的 `run_verify`。
- 单票判定偏向 keep → `VERIFY_LENS` 的措辞（任务 1）+ fail-open 逻辑（任务 2）。
- 通过 JoinSet 实现有界并发、不引入 `futures` → 任务 1 的 `run_verify`（`VERIFY_CONCURRENCY`）。
- 报告 “verify dropped K” → 任务 1 中 `render_deep`/`render_deep_result` 的 verify_note。
- 无 verify 路径逐字节一致 → 任务 1 中 `finalize_deep_review` 以 `None` 委托；由 `finalize_output_is_unchanged_after_the_split` 断言。
- single/deep 不变 → 任务 2 保持 `!is_deep()` 块与 deep fan-out 不动，只改其渲染尾部。

**占位符扫描：** 无 —— 所有步骤都带有具体代码。

**类型一致性：** `merge_deep_findings`/`dimension_coverage`/`render_deep_result`/`render_verify_task`/`run_verify`/`VERIFY_LENS`/`VERIFY_CONCURRENCY` 在任务 1 定义，并在任务 2 以完全相同的签名被消费。`run_verify` 返回 `Vec<bool>`，通过 `retain` 当作 keep-mask 使用。`wants_verify`/`is_deep` 在任务 2 定义并使用。verify 闭包沿用了 Phase 1 维度闭包的 `Send + 'static` 结构。
