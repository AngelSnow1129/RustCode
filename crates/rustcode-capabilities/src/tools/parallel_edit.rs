//! `parallel_edit_files` -- edit several INDEPENDENT files concurrently, each via its own
//! child agent (subagent-by-composition). The model supplies, per file, a `path` + a
//! natural-language `instruction`, plus a cross-file `contract` (shared invariants)
//! forwarded verbatim to every child. Each child is a fresh kernel [`Agent`] (its own
//! provider + mounted tools) that edits ONLY its assigned file, then stops; the children
//! run in parallel and their per-file statuses are collected into one result.
//!
//! L1 placement: a tool may hold an [`LlmProvider`](rustcode_kernel::provider::LlmProvider)
//! and spawn child agents -- same construction-time-injection pattern as the stateful
//! `change_dir`/`todo` tools. The kernel ([`Agent`] + `run_to_completion`) is L0, so this
//! needs nothing above the kernel. Because it carries a provider + a tool factory it is
//! OPT-IN (constructed by the embedder, not part of `register_coding_tools`).

use super::{err, ok};
use async_trait::async_trait;
use rustcode_kernel::agent::{Agent, AutoRespond, ToolLoopPolicy};
use rustcode_kernel::event::StopReason;
use rustcode_kernel::provider::LlmProvider;
use rustcode_kernel::tool::{MountedTools, Tool, ToolContext, ToolResult};
use serde::Deserialize;
use serde_json::json;
use std::path::Path;
use std::sync::Arc;

/// Default per-child system prompt: a focused single-file editor.
const DEFAULT_PERSONA: &str = "You are a focused file editor working on ONE file as part \
of a parallel batch. Read the file if needed, make exactly the change described in the \
instruction, and honor the cross-file contract. Do not edit any other file. When the \
edit is complete, stop with a one-line summary of what you changed.";

const DEFAULT_MAX_FILES: usize = 12;

/// Fallback candidates for a child, in the order they should be tried (FR-6.2).
/// Named so the field below stays readable (clippy::type_complexity).
type ChainProviderFn = dyn Fn() -> Vec<Arc<dyn LlmProvider>> + Send + Sync;

/// Edit multiple files in parallel via child agents. Construct with a provider factory
/// (a fresh provider per child -- a session consumes its provider) and a tools factory
/// (a fresh `MountedTools` per child -- it is not `Clone`); typically mount the L1
/// `read_file`/`edit_file`/`write_file` tools for the children.
pub struct ParallelEditTool {
    // `Arc` rather than `Box`: each spawned per-file task needs its own handle, and
    // a fallback hop consults the factories again inside that task.
    make_provider: Arc<dyn Fn() -> Arc<dyn LlmProvider> + Send + Sync>,
    make_tools: Arc<dyn Fn() -> MountedTools + Send + Sync>,
    /// Explicit fallback chain (FR-6.2), resolved by the layer that owns
    /// `Config`. `None` = no chain = one attempt per file, exactly as before.
    make_chain_providers: Option<Arc<ChainProviderFn>>,
    persona: String,
    max_files: usize,
    max_rounds: Option<u32>,
    tool_loop_policy: Option<ToolLoopPolicy>,
}

impl ParallelEditTool {
    pub fn new(
        make_provider: impl Fn() -> Arc<dyn LlmProvider> + Send + Sync + 'static,
        make_tools: impl Fn() -> MountedTools + Send + Sync + 'static,
    ) -> Self {
        Self {
            make_provider: Arc::new(make_provider),
            make_tools: Arc::new(make_tools),
            make_chain_providers: None,
            persona: DEFAULT_PERSONA.to_string(),
            max_files: DEFAULT_MAX_FILES,
            max_rounds: Some(super::DEFAULT_CHILD_MAX_ROUNDS),
            tool_loop_policy: Some(ToolLoopPolicy::default()),
        }
    }

    /// Wire the explicit fallback chain for the per-file child agents (FR-6.2).
    ///
    /// A file whose child failed BEFORE producing anything is retried on the next
    /// candidate instead of being reported as a hard failure -- the same
    /// content-free rule the main turn and `task` follow (FR-6.1). Leaving this
    /// unset keeps the historic single-attempt behaviour.
    pub fn with_chain_providers(
        mut self,
        make_chain: impl Fn() -> Vec<Arc<dyn LlmProvider>> + Send + Sync + 'static,
    ) -> Self {
        self.make_chain_providers = Some(Arc::new(make_chain));
        self
    }
    /// Override the per-child system prompt.
    pub fn with_persona(mut self, persona: impl Into<String>) -> Self {
        self.persona = persona.into();
        self
    }
    /// Override the max number of files per call (default 12).
    pub fn with_max_files(mut self, max: usize) -> Self {
        self.max_files = max.max(2);
        self
    }
    /// Override the per-child model-round high-water mark. `0` disables this cap;
    /// the exact no-progress policy is configured independently.
    pub fn with_max_rounds(mut self, n: u32) -> Self {
        self.max_rounds = (n != 0).then_some(n);
        self
    }

    /// Use the embedding product's exact no-progress policy. `None` disables it
    /// for intentional repeated operations; the independent round cap still applies.
    pub fn with_tool_loop_policy(mut self, policy: Option<ToolLoopPolicy>) -> Self {
        self.tool_loop_policy = policy;
        self
    }
}

#[derive(Deserialize)]
struct FileEdit {
    path: String,
    instruction: String,
}

#[derive(Deserialize)]
struct Args {
    files: Vec<FileEdit>,
    #[serde(default)]
    contract: String,
}

#[async_trait]
impl Tool for ParallelEditTool {
    fn name(&self) -> &str {
        "parallel_edit_files"
    }
    fn description(&self) -> &str {
        "Edit multiple INDEPENDENT files in parallel via fork sub-agents.\n\n\
        Use ONLY when:\n\
        - You have 2+ concrete files to edit, each with a clear instruction\n\
        - Edits in different files don't depend on each other\n\
        - You can express any cross-file invariants (shared trait/type/interface) in `contract`\n\n\
        Do NOT use when:\n\
        - You're still exploring or the edit isn't fully decided\n\
        - Files have impl/decl splits that need coordinated edits (use sequential edit_file)\n\
        - You want to read more files first (use read_file)\n\n\
        Each sub-agent sees only its assigned file content + the contract you provide. \
        Cross-file changes that aren't expressed in `contract` will be missed by the merge -- \
        the sub-agents cannot see each other's edits. After all sub-agents settle, the \
        framework runs a build probe (cargo/npm/mvn/go) and surfaces compile errors so you \
        can repair cross-file gaps."
    }
    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "files": {
                    "type": "array",
                    "minItems": 2,
                    "maxItems": 12,
                    "items": {
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "File path. Absolute, or relative to the working directory."
                            },
                            "instruction": {
                                "type": "string",
                                "description": "Concrete edit description for THIS file. Be specific: what to add/modify/remove and why. The sub-agent sees only this instruction + the file content + the contract -- no other context."
                            }
                        },
                        "required": ["path", "instruction"]
                    }
                },
                "contract": {
                    "type": "string",
                    "description": "Cross-file invariants every sub-agent must honour: shared traits, type signatures, interface contracts, naming conventions. Empty if files are fully independent."
                }
            },
            "required": ["files"]
        })
    }
    // children call edit_file (itself Risky + gateable); the dispatch itself reads as
    // Risky since it mutates many files. Approval middleware can gate on the name.
    fn risk(&self, _args: &str) -> rustcode_kernel::tool::RiskLevel {
        rustcode_kernel::tool::RiskLevel::Risky
    }
    fn always_grant_scope(&self, _args: &str) -> String {
        // Tool-wide: "总是 / Always" approves every batch edit this session (v1 parity).
        String::new()
    }
    async fn execute(&self, args: &str, ctx: &ToolContext) -> ToolResult {
        let a: Args = match serde_json::from_str(args) {
            Ok(a) => a,
            Err(e) => {
                return err(format!(
                    "{} (parallel_edit_files arguments must be {{\"files\": [{{\"path\": \"...\", \"instruction\": \"...\"}}, ...], \"contract\": \"...\"?}})",
                    e
                ))
            }
        };
        if a.files.len() < 2 {
            return err("parallel_edit_files requires at least 2 files. For a single file, call edit_file directly.");
        }
        if a.files.len() > self.max_files {
            return err(format!(
                "parallel_edit_files capped at {} files; you sent {}. Split into smaller batches or run sequentially.",
                self.max_files, a.files.len()
            ));
        }
        for (i, f) in a.files.iter().enumerate() {
            if f.path.trim().is_empty() {
                return err(format!("files[{}].path is empty", i));
            }
            if f.instruction.trim().is_empty() {
                return err(format!(
                    "files[{}].instruction is empty. Each file needs a concrete edit description; \
                     a sub-agent with no instruction will either fake an edit or burn its budget.",
                    i
                ));
            }
        }

        let contract_block = if a.contract.trim().is_empty() {
            String::new()
        } else {
            format!(
                "\n\nCross-file contract (honor exactly):\n{}",
                a.contract.trim()
            )
        };

        // Spawn one detached child agent per file, concurrently. Detaching via
        // tokio::spawn (then awaiting the JoinHandle) keeps the child's cancel wired to
        // the parent token: if this tool future is dropped on cancel, the still-running
        // child is stopped only by `ctx.cancel.child_token()` cascading in.
        // Dispatch header so the user sees the fan-out begin (v1 SubAgentDispatchStart
        // parity). Per-file ↻/[+]/[x] lines follow via `ctx.progress` -> ToolProgress.
        ctx.progress.emit({
            #[cfg(feature = "provider")]
            {
                rustcode_config::i18n::t(rustcode_config::i18n::Msg::ToolProgressParallelEdit {
                    count: a.files.len(),
                })
                .into_owned()
            }
            #[cfg(not(feature = "provider"))]
            {
                format!("Parallel editing of {} files (subagents)", a.files.len())
            }
        });
        let mut handles = Vec::with_capacity(a.files.len());
        // Shared handles for the spawned per-file tasks: the tools factory and the
        // chain resolver are consulted again inside the task, once per attempt.
        let make_tools = Arc::clone(&self.make_tools);
        let chain_providers = self.make_chain_providers.clone();
        let persona = self.persona.clone();
        for f in &a.files {
            let task = format!(
                "File to edit: {}\n\nInstruction:\n{}{}\n\nEdit ONLY this file using your tools, then stop.",
                f.path, f.instruction, contract_block
            );
            // Candidates in priority order: this file's own provider first, then the
            // explicit chain (FR-6.2). The SHARED helper de-dupes by provider
            // IDENTITY -- two providers may legitimately serve the same raw model
            // name, so display text cannot decide this.
            let primary = (self.make_provider)();
            let candidates = crate::fallback::chain_candidates(
                primary,
                chain_providers
                    .as_deref()
                    .map(|chain| chain())
                    .unwrap_or_default(),
            );
            let path = f.path.clone();
            let persona = persona.clone();
            let make_tools = Arc::clone(&make_tools);
            let tool_loop_policy = self.tool_loop_policy;
            let max_rounds = self.max_rounds;
            // Cheap clone (Arc inside); moved into the child task so it can report the
            // moment THIS child settles -- concurrent, so lines interleave by real
            // completion order, giving live per-file progress instead of a black box.
            let progress = ctx.progress.clone();
            let cancel = ctx.cancel.child_token();
            let working_dir = ctx.working_dir.clone();
            handles.push(tokio::spawn(async move {
                progress.emit(format!("↻ {path}"));
                let total = candidates.len();
                let mut outcome = rustcode_kernel::agent::Outcome::default();
                for (index, provider) in candidates.iter().enumerate() {
                    let mut builder = Agent::builder()
                        .provider(Arc::clone(provider))
                        .tools((make_tools)())
                        .persona(persona.clone())
                        .working_dir(working_dir.clone())
                        .cancel_token(cancel.clone());
                    if let Some(policy) = tool_loop_policy {
                        builder = builder.tool_loop_policy(policy);
                    }
                    if let Some(max_rounds) = max_rounds {
                        builder = builder.max_rounds(max_rounds);
                    }
                    outcome = builder
                        .build()
                        .run_to_completion(task.clone(), AutoRespond::AllowAll)
                        .await;
                    if outcome.stop == StopReason::Stopped {
                        break;
                    }
                    // Same content-free rule as the main turn and `task` (FR-6.1): a
                    // child that already edited the file must NOT be replayed on the
                    // next model, or the edit lands twice. A terminal failure (401)
                    // is not a model problem either, so the walk stops there too.
                    if !crate::fallback::fallback_eligible(&outcome, cancel.is_cancelled()) {
                        break;
                    }
                    if index + 1 == total {
                        break;
                    }
                    let from = candidates[index].model_name().to_string();
                    let to = candidates[index + 1].model_name().to_string();
                    let reason = outcome
                        .error
                        .clone()
                        .unwrap_or_else(|| format!("{:?}", outcome.stop));
                    progress.emit({
                        #[cfg(feature = "provider")]
                        {
                            rustcode_config::i18n::t(
                                rustcode_config::i18n::Msg::ModelFallbackStarted {
                                    from: &from,
                                    to: &to,
                                    reason: &reason,
                                },
                            )
                            .into_owned()
                        }
                        #[cfg(not(feature = "provider"))]
                        {
                            format!("`{from}` failed ({reason}); retrying on `{to}`")
                        }
                    });
                }
                let icon = if outcome.stop == StopReason::Stopped {
                    "[+]"
                } else {
                    "[x]"
                };
                progress.emit(format!("{icon} {path}"));
                (path, outcome)
            }));
        }

        struct FileResult {
            path: String,
            success: bool,
            turns_used: usize,
            summary: String,
            failures: Vec<String>,
        }

        let mut results = Vec::with_capacity(handles.len());
        for h in handles {
            let (path, outcome) = match h.await {
                Ok(pair) => pair,
                Err(_) => {
                    results.push(FileResult {
                        path: "<unknown>".to_string(),
                        success: false,
                        turns_used: 0,
                        summary: String::new(),
                        failures: vec!["child task panicked/aborted".to_string()],
                    });
                    continue;
                }
            };
            if outcome.stop == StopReason::Stopped {
                let summary = outcome.text.lines().next().unwrap_or("").trim();
                let summary = if summary.is_empty() {
                    "(edited)"
                } else {
                    summary
                };
                results.push(FileResult {
                    path,
                    success: true,
                    turns_used: outcome.tool_results.len(),
                    summary: summary.to_string(),
                    failures: vec![],
                });
            } else {
                let reason = outcome
                    .error
                    .unwrap_or_else(|| format!("{:?}", outcome.stop));
                results.push(FileResult {
                    path: path.clone(),
                    success: false,
                    turns_used: outcome.tool_results.len(),
                    summary: String::new(),
                    failures: vec![reason],
                });
            }
        }

        let ok_count = results.iter().filter(|r| r.success).count();
        let fail_count = results.len() - ok_count;
        let mut summary = format!(
            "Sub-agents: {} ok, {} fail (of {})\n",
            ok_count,
            fail_count,
            results.len(),
        );
        let mut all_success = fail_count == 0;
        for r in &results {
            let icon = if r.success { "[+]" } else { "[x]" };
            let summary_line = r.summary.lines().next().unwrap_or("").trim();
            summary.push_str(&format!(
                "  {} {} ({}T) -- {}\n",
                icon, r.path, r.turns_used, summary_line,
            ));
            if !r.success {
                for failure in &r.failures {
                    summary.push_str(&format!("      reason: {:?}\n", failure));
                }
            }
        }

        // Build verification -- best-effort, structural detector (probes
        // for build-system markers, not model intent). On miss the table
        // is the final answer. The marker probe does blocking `read_dir`,
        // so run it on the blocking pool to keep cancellation responsive.
        let working_dir = ctx.working_dir.clone();
        let build_detect = tokio::task::spawn_blocking(move || find_build_command(&working_dir))
            .await
            .ok()
            .flatten();
        if let Some((cmd, build_dir)) = build_detect {
            // Platform-appropriate shell: cmd.exe on Windows, sh on Unix (mirrors the
            // bash tool + v1 parallel_edit). Without this the probe spawned `sh`, which
            // is absent on Windows -> the probe silently never ran.
            #[cfg(windows)]
            let (shell, flag) = ("cmd.exe", "/C");
            #[cfg(not(windows))]
            let (shell, flag) = ("sh", "-c");

            let mut build_cmd = tokio::process::Command::new(shell);
            build_cmd.args([flag, &cmd]).current_dir(&build_dir);
            // Suppress the Windows console-window flash for the probe; no-op off Windows.
            crate::process_utils::suppress_console_window(&mut build_cmd);
            let output = build_cmd.output().await;
            if let Ok(out) = output {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                let combined = format!("{}{}", stdout, stderr);
                if !out.status.success() || combined.to_lowercase().contains("error") {
                    let err_lines: String =
                        combined.lines().take(15).collect::<Vec<_>>().join("\n");
                    summary.push_str(&format!(
                        "\n[!] BUILD ERRORS after merge:\n{}\nFix these before proceeding.\n",
                        err_lines
                    ));
                    all_success = false;
                } else {
                    summary.push_str("\n[+] Build verification passed.\n");
                }
            }
        }

        if all_success {
            ok(summary)
        } else {
            err(summary)
        }
    }
}

/// Probe for build markers in the working directory (Cargo.toml, package.json, pom.xml,
/// go.mod) and return the appropriate build command + the directory where the marker was
/// found. Searches the working directory then immediate subdirectories so nested project
/// layouts (a Cargo workspace under a monorepo) still resolve.
fn find_build_command(wd: &Path) -> Option<(String, std::path::PathBuf)> {
    // No Unix-only pipes (head/tail): the probe runs under cmd.exe on Windows where
    // those coreutils don't exist. Full output is captured; the error display is
    // truncated Rust-side (`combined.lines().take(15)`) below.
    let markers: &[(&str, &str)] = &[
        ("package.json", "npm run build 2>&1"),
        ("Cargo.toml", "cargo check 2>&1"),
        ("pom.xml", "mvn compile -q 2>&1"),
        ("go.mod", "go build ./... 2>&1"),
    ];

    for &(marker, cmd) in markers {
        if wd.join(marker).exists() {
            return Some((cmd.to_string(), wd.to_path_buf()));
        }
    }

    if let Ok(entries) = std::fs::read_dir(wd) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let sub = entry.path();
                let name = sub.file_name().unwrap_or_default().to_string_lossy();
                if name.starts_with('.') || name == "node_modules" || name == "target" {
                    continue;
                }
                for &(marker, cmd) in markers {
                    if sub.join(marker).exists() {
                        return Some((cmd.to_string(), sub));
                    }
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream::{self, BoxStream};
    use futures::StreamExt;
    use rustcode_kernel::message::Message;
    use rustcode_kernel::provider::ChatOptions;
    use rustcode_kernel::stream::{ProviderError, StreamEvent};
    use rustcode_kernel::tool::{ProgressSink, ToolDef, ToolRegistry};
    use tokio_util::sync::CancellationToken;

    /// The build-verification probe runs under `cmd.exe /C` on Windows, where the
    /// Unix coreutils `head`/`tail` don't exist -- so the detected commands must not
    /// pipe through them (output is already truncated Rust-side for display).
    #[test]
    fn build_commands_are_cross_platform_no_unix_pipes() {
        let d = tempfile::tempdir().unwrap();
        for (marker, body) in [
            ("Cargo.toml", "[package]\nname=\"x\"\nversion=\"0.0.0\"\n"),
            ("package.json", "{}"),
            ("pom.xml", "<project/>"),
            ("go.mod", "module x\n"),
        ] {
            let dir = d.path().join(marker.replace('.', "_"));
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join(marker), body).unwrap();
            let (cmd, _) = find_build_command(&dir).expect("marker should be detected");
            assert!(
                !cmd.contains("head") && !cmd.contains("tail"),
                "build command must not depend on Unix-only head/tail (breaks cmd.exe on Windows): {cmd}"
            );
        }
    }

    /// Stateless scripted provider: `Some(reply)` -> one text turn then stop;
    /// `None` -> a terminal open error (simulates a failed child).
    struct MockProvider {
        reply: Option<String>,
    }
    #[async_trait]
    impl LlmProvider for MockProvider {
        fn model_name(&self) -> &str {
            "mock"
        }
        async fn chat_stream(
            &self,
            _m: &[Message],
            _t: &[ToolDef],
            _o: &ChatOptions,
        ) -> Result<BoxStream<'static, StreamEvent>, ProviderError> {
            match &self.reply {
                Some(text) => {
                    let evs = vec![
                        StreamEvent::TextDelta(text.clone()),
                        StreamEvent::Done { truncated: false },
                    ];
                    Ok(stream::iter(evs).boxed())
                }
                None => Err(ProviderError {
                    retryable: false,
                    message: "mock open failure".into(),
                    ..Default::default()
                }),
            }
        }
    }

    fn ctx() -> ToolContext {
        // Isolated EMPTY working dir. Pointing at the *shared* std::env::temp_dir()
        // let the post-edit build-verification probe (find_build_command scans
        // working_dir + its immediate subdirs) pick up a stray sibling
        // package.json/Cargo.toml left by another test/tool and run a real, failing
        // build -- spuriously flipping is_error. A dedicated empty dir keeps it inert.
        let dir = tempfile::tempdir().expect("tempdir").keep();
        ToolContext {
            working_dir: dir,
            cancel: CancellationToken::new(),
            progress: ProgressSink::noop(),
            requester: None,
        }
    }

    fn tool(reply: Option<&'static str>) -> ParallelEditTool {
        let reply = reply.map(|s| s.to_string());
        ParallelEditTool::new(
            move || {
                Arc::new(MockProvider {
                    reply: reply.clone(),
                }) as Arc<dyn LlmProvider>
            },
            || ToolRegistry::new().mount(&[]), // children need no tools for these tests
        )
    }

    #[test]
    fn child_round_limit_is_configurable_and_zero_means_unbounded() {
        assert_eq!(tool(None).max_rounds, Some(200));
        assert_eq!(tool(None).with_max_rounds(500).max_rounds, Some(500));
        assert_eq!(tool(None).with_max_rounds(0).max_rounds, None);
    }

    #[test]
    fn child_exact_loop_policy_can_be_inherited_or_disabled() {
        assert_eq!(tool(None).tool_loop_policy, Some(ToolLoopPolicy::default()));
        assert_eq!(
            tool(None).with_tool_loop_policy(None).tool_loop_policy,
            None
        );
    }

    #[tokio::test]
    async fn emits_per_file_dispatch_progress() {
        // Real-time per-file progress (v1 SubAgentDispatch* parity): each child's
        // start (↻) and settle ([+]/[x]) is surfaced via ctx.progress so the driver
        // can stream it (AgentEvent::ToolProgress) instead of a black-box result.
        let captured = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let sink = {
            let c = captured.clone();
            ProgressSink::new(Arc::new(move |m| c.lock().unwrap().push(m)))
        };
        let ctx = ToolContext {
            // Isolated empty dir -- see `ctx()` for why the shared temp dir is unsafe.
            working_dir: tempfile::tempdir().expect("tempdir").keep(),
            cancel: CancellationToken::new(),
            progress: sink,
            requester: None,
        };
        let _ = tool(Some("done"))
            .execute(
                r#"{"files":[{"path":"a.rs","instruction":"x"},{"path":"b.rs","instruction":"y"}]}"#,
                &ctx,
            )
            .await;
        let msgs = captured.lock().unwrap();
        // Start line per file.
        assert!(
            msgs.iter().any(|m| m.contains("↻") && m.contains("a.rs")),
            "start a.rs: {msgs:?}"
        );
        assert!(
            msgs.iter().any(|m| m.contains("↻") && m.contains("b.rs")),
            "start b.rs: {msgs:?}"
        );
        // Settle line per file (mock provider stops cleanly -> [+]).
        assert!(
            msgs.iter().any(|m| m.contains("[+]") && m.contains("a.rs")),
            "done a.rs: {msgs:?}"
        );
        assert!(
            msgs.iter().any(|m| m.contains("[+]") && m.contains("b.rs")),
            "done b.rs: {msgs:?}"
        );
    }

    #[tokio::test]
    async fn dispatches_one_child_per_file_and_collects_statuses() {
        let t = tool(Some("renamed the symbol"));
        let r = t
            .execute(
                r#"{"files":[{"path":"a.rs","instruction":"do x"},{"path":"b.rs","instruction":"do y"}],"contract":"keep trait T stable"}"#,
                &ctx(),
            )
            .await;
        assert!(!r.is_error, "{}", r.content);
        assert!(r.content.contains("2 ok, 0 fail (of 2)"), "{}", r.content);
        assert!(r.content.contains("[+] a.rs"), "{}", r.content);
        assert!(r.content.contains("[+] b.rs"), "{}", r.content);
    }

    #[tokio::test]
    async fn fewer_than_two_files_errors() {
        let r = tool(Some("x"))
            .execute(r#"{"files":[{"path":"a.rs","instruction":"y"}]}"#, &ctx())
            .await;
        assert!(r.is_error);
        assert!(r.content.contains("at least 2 files"), "{}", r.content);
    }

    #[tokio::test]
    async fn too_many_files_errors() {
        let files: Vec<String> = (0..13)
            .map(|i| format!("{{\"path\":\"f{i}.rs\",\"instruction\":\"x\"}}"))
            .collect();
        let args = format!("{{\"files\":[{}]}}", files.join(","));
        let r = tool(Some("x")).execute(&args, &ctx()).await;
        assert!(r.is_error);
        assert!(r.content.contains("capped at 12 files"), "{}", r.content);
    }

    #[tokio::test]
    async fn empty_path_or_instruction_errors() {
        let r = tool(Some("x"))
            .execute(
                r#"{"files":[{"path":"a.rs","instruction":""},{"path":"b.rs","instruction":"y"}]}"#,
                &ctx(),
            )
            .await;
        assert!(r.is_error);
        assert!(
            r.content.contains("files[0].instruction is empty"),
            "{}",
            r.content
        );
    }

    #[tokio::test]
    async fn child_failure_is_surfaced_and_marks_error() {
        // provider returns None -> every child fails its open; the row shows [x] and the
        // overall result is_error.
        let r = tool(None)
            .execute(r#"{"files":[{"path":"a.rs","instruction":"x"},{"path":"b.rs","instruction":"y"}]}"#, &ctx())
            .await;
        assert!(r.is_error, "{}", r.content);
        assert!(r.content.contains("0 ok, 2 fail (of 2)"), "{}", r.content);
        assert!(r.content.contains("[x] a.rs"), "{}", r.content);
    }

    #[test]
    fn risk_is_risky() {
        assert_eq!(
            tool(Some("x")).risk("{}"),
            rustcode_kernel::tool::RiskLevel::Risky
        );
    }

    /// Scripted provider that records which model was asked and fails transiently
    /// (retryable 503) when it has no reply -- the shape a fallback hop can survive.
    struct ChainProvider {
        model: &'static str,
        reply: Option<&'static str>,
        attempted: Arc<std::sync::Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl LlmProvider for ChainProvider {
        fn model_name(&self) -> &str {
            self.model
        }

        async fn chat_stream(
            &self,
            _m: &[Message],
            _t: &[ToolDef],
            _o: &ChatOptions,
        ) -> Result<BoxStream<'static, StreamEvent>, ProviderError> {
            self.attempted.lock().unwrap().push(self.model.to_string());
            if let Some(text) = self.reply {
                return Ok(stream::iter(vec![
                    StreamEvent::TextDelta(text.to_string()),
                    StreamEvent::Done { truncated: false },
                ])
                .boxed());
            }
            Err(ProviderError {
                retryable: true,
                http_status: Some(503),
                message: "down".into(),
                ..Default::default()
            })
        }
    }

    /// Models attempted, in FIRST-APPEARANCE order.
    ///
    /// Two reasons the raw log repeats a model, and only first-appearance order is
    /// stable against both: (1) the kernel retries a retryable failure INSIDE one
    /// child, so the log repeats a model before the walk moves on; (2) the per-file
    /// children run CONCURRENTLY and share this log, so their retries INTERLEAVE --
    /// a consecutive-run collapse splits under interleaving (observed as
    /// `primary, hop-1, primary, ...` under load) and flakes. First appearance is
    /// the sound contract: every child walks the chain in order, so each model's
    /// first appearance IS the chain order, and no model outside the chain can
    /// appear at all.
    fn models_tried(attempted: &Arc<std::sync::Mutex<Vec<String>>>) -> Vec<String> {
        let mut seen: Vec<String> = Vec::new();
        for model in attempted.lock().unwrap().iter() {
            if !seen.contains(model) {
                seen.push(model.clone());
            }
        }
        seen
    }

    fn chain_tool(
        attempted: Arc<std::sync::Mutex<Vec<String>>>,
        chain: Vec<(&'static str, Option<&'static str>)>,
    ) -> ParallelEditTool {
        let primary = Arc::clone(&attempted);
        ParallelEditTool::new(
            move || {
                Arc::new(ChainProvider {
                    model: "primary",
                    reply: None,
                    attempted: Arc::clone(&primary),
                }) as Arc<dyn LlmProvider>
            },
            || ToolRegistry::new().mount(&[]),
        )
        .with_chain_providers(move || {
            chain
                .iter()
                .map(|(model, reply)| {
                    Arc::new(ChainProvider {
                        model,
                        reply: *reply,
                        attempted: Arc::clone(&attempted),
                    }) as Arc<dyn LlmProvider>
                })
                .collect()
        })
    }

    /// FR-6.2: a file whose child failed BEFORE editing is retried on the next chain
    /// candidate, so one flaky model no longer fails the whole batch. The ORDER of
    /// attempted models is the contract -- a final success could come from anywhere.
    #[tokio::test]
    async fn a_failed_child_walks_the_chain_in_order() {
        let attempted = Arc::new(std::sync::Mutex::new(Vec::new()));
        let tool = chain_tool(
            Arc::clone(&attempted),
            vec![("hop-1", None), ("hop-2", Some("recovered"))],
        );
        let r = tool
            .execute(
                r#"{"files":[{"path":"a.rs","instruction":"x"},{"path":"b.rs","instruction":"y"}]}"#,
                &ctx(),
            )
            .await;
        assert_eq!(
            models_tried(&attempted),
            vec!["primary", "hop-1", "hop-2"],
            "the chain must be walked in order"
        );
        assert!(!r.is_error, "{}", r.content);
        assert!(r.content.contains("2 ok, 0 fail (of 2)"), "{}", r.content);
    }

    /// A-10: with no chain configured the child gets exactly one model, so an
    /// unconfigured call behaves as it did before fallback existed.
    #[tokio::test]
    async fn an_absent_chain_attempts_only_the_primary() {
        let attempted = Arc::new(std::sync::Mutex::new(Vec::new()));
        let primary = Arc::clone(&attempted);
        let tool = ParallelEditTool::new(
            move || {
                Arc::new(ChainProvider {
                    model: "primary",
                    reply: None,
                    attempted: Arc::clone(&primary),
                }) as Arc<dyn LlmProvider>
            },
            || ToolRegistry::new().mount(&[]),
        );
        let r = tool
            .execute(
                r#"{"files":[{"path":"a.rs","instruction":"x"},{"path":"b.rs","instruction":"y"}]}"#,
                &ctx(),
            )
            .await;
        assert_eq!(models_tried(&attempted), vec!["primary"]);
        assert!(r.is_error, "{}", r.content);
    }

    /// FR-6.1 / A-5: a child that already produced output is NOT replayed on the
    /// next model -- its edit would land twice. Asserted on the shared predicate the
    /// walk consults, since a mid-failure text delta cannot be scripted here.
    #[test]
    fn a_child_that_produced_output_is_never_replayed() {
        let mut with_text = rustcode_kernel::agent::Outcome {
            stop: StopReason::ProviderError,
            text: "edited the file".into(),
            http_status: Some(503),
            ..Default::default()
        };
        with_text.tool_results.push(Default::default());
        assert!(!crate::fallback::fallback_eligible(&with_text, false));

        let content_free = rustcode_kernel::agent::Outcome {
            stop: StopReason::ProviderError,
            http_status: Some(503),
            ..Default::default()
        };
        assert!(crate::fallback::fallback_eligible(&content_free, false));
    }
}
