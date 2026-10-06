use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use rustcode_capabilities::team::{
    role_by_id, TeamDifficulty, TeamPermission, TeamRoleProfile, TeamTaskSpec,
};
#[cfg(test)]
use rustcode_capabilities::tools::team_child_middlewares;
use rustcode_capabilities::tools::team_child_middlewares_with_bypass;
use rustcode_kernel::agent::{Agent, AutoRespond, Outcome, ToolLoopPolicy};
use rustcode_kernel::event::StopReason;
use rustcode_kernel::hook::{LifecycleHooks, TurnCtx};
use rustcode_kernel::message::Message;
use rustcode_kernel::middleware::{BeforeOutcome, ToolMiddleware};
use rustcode_kernel::provider::LlmProvider;
use rustcode_kernel::request::RequestCtx;
use rustcode_kernel::tool::{MountedTools, Tool, ToolCall};
use tokio_util::sync::CancellationToken;

use rustcode_config::i18n::{t, Msg};

use super::{TeamActivitySink, TeamJobFactory, TeamMemberOutcome, TeamModelFactory};

pub type TeamProviderFactory = Arc<dyn Fn(TeamDifficulty) -> Arc<dyn LlmProvider> + Send + Sync>;
pub type TeamToolsFactory = Arc<dyn Fn(TeamPermission) -> MountedTools + Send + Sync>;
/// Fallback candidates for a tier, in the order they should be tried (FR-6.2).
/// Empty means "no chain", which keeps the single-shot behaviour intact.
pub type TeamChainProviderFactory =
    Arc<dyn Fn(TeamDifficulty) -> Vec<Arc<dyn LlmProvider>> + Send + Sync>;

#[derive(Clone)]
pub struct TeamRunnerFactory {
    providers: TeamProviderFactory,
    tools: TeamToolsFactory,
    /// Explicit per-tier fallback chain, resolved by the layer that owns `Config`
    /// (see `parts.rs`); `None` = no chain configured = one attempt per member.
    chain_providers: Option<TeamChainProviderFactory>,
    working_dir: std::path::PathBuf,
    max_rounds: Option<u32>,
    tool_loop_policy: Option<ToolLoopPolicy>,
    stream_timeout: Option<Duration>,
    request_timeout: Option<Duration>,
    first_token_timeout: Option<Duration>,
    inherited_worker_middlewares: Vec<Arc<dyn ToolMiddleware>>,
    credential_shell_policy: rustcode_capabilities::tools::CredentialShellPolicy,
    /// Parent's live Auto-mode flag, cloned into every member's credential gate.
    credential_shell_bypass: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    /// Dispatch depth of runners produced by this factory (0 = root).
    depth: u8,
    /// Maximum delegation depth (0 = flat, 2 = three-tier).
    max_depth: u8,
}

impl TeamRunnerFactory {
    pub fn new(
        providers: TeamProviderFactory,
        tools: TeamToolsFactory,
        working_dir: std::path::PathBuf,
    ) -> Self {
        Self {
            providers,
            tools,
            chain_providers: None,
            working_dir,
            max_rounds: None,
            tool_loop_policy: None,
            stream_timeout: None,
            request_timeout: None,
            first_token_timeout: None,
            inherited_worker_middlewares: Vec::new(),
            credential_shell_policy: Default::default(),
            credential_shell_bypass: None,
            depth: 0,
            max_depth: 2,
        }
    }

    pub fn with_depth(mut self, depth: u8, max_depth: u8) -> Self {
        self.depth = depth;
        self.max_depth = max_depth;
        self
    }

    pub fn depth(&self) -> u8 {
        self.depth
    }

    pub fn max_depth(&self) -> u8 {
        self.max_depth
    }

    pub fn with_runtime_policy(
        mut self,
        max_rounds: Option<u32>,
        tool_loop_policy: Option<ToolLoopPolicy>,
        stream_timeout: Option<Duration>,
        request_timeout: Option<Duration>,
        first_token_timeout: Option<Duration>,
    ) -> Self {
        self.max_rounds = max_rounds.filter(|rounds| *rounds > 0);
        self.tool_loop_policy = tool_loop_policy;
        self.stream_timeout = stream_timeout;
        self.request_timeout = request_timeout;
        self.first_token_timeout = first_token_timeout;
        self
    }

    pub fn with_worker_middleware(mut self, middleware: Arc<dyn ToolMiddleware>) -> Self {
        self.inherited_worker_middlewares.push(middleware);
        self
    }

    pub fn with_credential_shell_policy(
        mut self,
        policy: rustcode_capabilities::tools::CredentialShellPolicy,
    ) -> Self {
        self.credential_shell_policy = policy;
        self
    }

    /// Wire the parent's live Auto-mode flag: while it is set, a member's credential
    /// gate allows instead of failing closed / terminating the member's turn.
    pub fn with_credential_shell_bypass(
        mut self,
        bypass: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        self.credential_shell_bypass = Some(bypass);
        self
    }

    /// Wire members' explicit per-tier fallback chain (FR-6.2 / FR-6.3).
    ///
    /// The chain is resolved by the layer that owns `Config` and handed over as
    /// ready-to-run providers, so this crate stays the only one that knows about
    /// config shape. Leaving it unset is the default and restores the historic
    /// single-attempt behaviour exactly.
    pub fn with_chain_providers(
        mut self,
        chain: impl Fn(TeamDifficulty) -> Vec<Arc<dyn LlmProvider>> + Send + Sync + 'static,
    ) -> Self {
        self.chain_providers = Some(Arc::new(chain));
        self
    }

    pub fn job_factory(&self) -> TeamJobFactory {
        let runner = self.clone();
        Arc::new(move |task, cancel, activity| {
            let runner = runner.clone();
            Box::pin(async move { runner.run(task, cancel, activity).await })
        })
    }

    pub fn model_factory(&self) -> TeamModelFactory {
        let providers = Arc::clone(&self.providers);
        Arc::new(move |task| (providers)(task.difficulty).model_name().to_string())
    }

    /// Assemble one attempt's agent.
    ///
    /// Extracted so a fallback hop builds a *fresh* agent on the next provider
    /// rather than reusing a builder that already failed -- the previous attempt's
    /// provider must leave no residue behind (FR-3.5).
    fn build_member(
        &self,
        provider: Arc<dyn LlmProvider>,
        tools: MountedTools,
        task: &TeamTaskSpec,
        profile: &TeamRoleProfile,
        cancel: CancellationToken,
        progress: Arc<TeamProgressHook>,
    ) -> Agent {
        let mut builder = Agent::builder()
            .provider(provider)
            .tools(tools)
            .persona(team_member_persona(profile, &task.scope))
            .working_dir(self.working_dir.clone())
            .cancel_token(cancel)
            .hook(progress)
            .middleware(Arc::new(DenyTeamBash));
        for middleware in team_child_middlewares_with_bypass(
            task.permission == TeamPermission::Worker,
            &task.scope,
            &self.working_dir,
            &self.inherited_worker_middlewares,
            self.credential_shell_policy,
            self.credential_shell_bypass.clone(),
        ) {
            builder = builder.middleware(middleware);
        }
        if let Some(rounds) = self.max_rounds {
            builder = builder.max_rounds(rounds);
        }
        if let Some(policy) = self.tool_loop_policy {
            builder = builder.tool_loop_policy(policy);
        }
        if let Some(timeout) = self.stream_timeout {
            builder = builder.stream_timeout(timeout);
        }
        if let Some(timeout) = self.request_timeout {
            builder = builder.request_timeout(timeout);
        }
        if let Some(timeout) = self.first_token_timeout {
            builder = builder.first_token_timeout(timeout);
        }
        builder.build()
    }

    async fn run(
        &self,
        task: TeamTaskSpec,
        cancel: CancellationToken,
        activity: TeamActivitySink,
    ) -> TeamMemberOutcome {
        let Some(profile) = role_by_id(task.role.as_str()) else {
            return TeamMemberOutcome::failed(format!("unknown team role: {}", task.role));
        };
        // The activity sink is also the member's only rendering channel, so the
        // failover notice goes out through it (FR-6.3). Keep a handle before the
        // hook takes ownership.
        let activity_sink = Arc::clone(&activity);
        let progress = Arc::new(TeamProgressHook::new(activity));
        // Candidates, in priority order: this tier's own provider first, then its
        // explicit chain (FR-6.2). The SHARED helper de-dupes by provider IDENTITY
        // -- two configured providers may legitimately expose the same raw model
        // name, so display text cannot decide this.
        let primary = (self.providers)(task.difficulty);
        let chain = self
            .chain_providers
            .as_ref()
            .map(|resolve| resolve(task.difficulty))
            .unwrap_or_default();
        let candidates = rustcode_capabilities::fallback::chain_candidates(primary, chain);
        let total = candidates.len();
        let mut outcome = Outcome::default();
        let mut failures: Vec<String> = Vec::new();
        let mut exhausted = false;
        for (index, provider) in candidates.iter().enumerate() {
            if index > 0 {
                let from = candidates[index - 1].model_name().to_string();
                let to = provider.model_name().to_string();
                let reason = crate::fallback::attempt_reason(&outcome);
                (activity_sink)(
                    t(Msg::ModelFallbackStarted {
                        from: &from,
                        to: &to,
                        reason: &reason,
                    })
                    .into_owned(),
                    progress.live_tokens(),
                );
            }
            let agent = self.build_member(
                Arc::clone(provider),
                (self.tools)(task.permission),
                &task,
                profile,
                cancel.clone(),
                Arc::clone(&progress),
            );
            outcome = agent
                .run_to_completion(task.prompt.clone(), AutoRespond::AllowAll)
                .await;
            if outcome.stop == StopReason::Stopped {
                break;
            }
            failures.push(format!(
                "{}: {}",
                provider.model_name(),
                crate::fallback::attempt_reason(&outcome)
            ));
            // The shared predicate decides whether a DIFFERENT model could survive
            // this failure (FR-6.1). A terminal 401 or an already-produced answer
            // stops the walk here rather than replaying it on the next candidate.
            if !rustcode_capabilities::fallback::fallback_eligible(&outcome, cancel.is_cancelled())
            {
                break;
            }
            if index + 1 == total {
                // Eligible failure but nothing left to walk to. Only call this
                // "exhausted" when there WAS a chain: with no chain configured the
                // member simply failed once, and the notice would be new behaviour
                // on a config that never asked for fallback (A-10).
                exhausted = total > 1;
                break;
            }
        }
        if exhausted {
            // Say the chain is spent instead of letting the last provider error
            // stand in for the whole walk (FR-3.6 / FR-4.4).
            let attempts = failures.join("; ");
            (activity_sink)(
                t(Msg::ModelFallbackExhausted {
                    attempts: &attempts,
                })
                .into_owned(),
                progress.live_tokens(),
            );
        }
        let output = if !outcome.text.is_empty() {
            outcome.text
        } else if let Some(error) = outcome.error {
            error
        } else {
            outcome
                .tool_results
                .into_iter()
                .map(|result| result.content)
                .collect::<Vec<_>>()
                .join("\n")
        };
        // Carry the final accumulated token total out via the outcome: the closing
        // round has no tool call, so on_model_response surfaced no activity for it.
        let output_tokens = progress.total_tokens();
        let member = match outcome.stop {
            StopReason::Stopped => TeamMemberOutcome::completed(output),
            StopReason::Cancelled => TeamMemberOutcome {
                success: false,
                stop: "stopped".into(),
                output,
                output_tokens: 0,
            },
            stop => TeamMemberOutcome {
                success: false,
                stop: format!("{stop:?}"),
                output,
                output_tokens: 0,
            },
        };
        TeamMemberOutcome {
            output_tokens,
            ..member
        }
    }
}

struct TeamProgressHook {
    activity: TeamActivitySink,
    /// Output+reasoning tokens finalized from completed rounds -- the provider's
    /// reported `completion` count when available, else a chars/4 estimate. Mirrors
    /// the legacy `task` subagent panel so both show a real live token count.
    total_tokens: std::sync::atomic::AtomicU64,
    /// Chars streamed in the CURRENT (unfinished) round, reset at each round end.
    round_chars: std::sync::atomic::AtomicU64,
}

impl TeamProgressHook {
    fn new(activity: TeamActivitySink) -> Self {
        Self {
            activity,
            total_tokens: std::sync::atomic::AtomicU64::new(0),
            round_chars: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Finalized tokens plus a running estimate for the in-progress round.
    fn live_tokens(&self) -> u64 {
        use std::sync::atomic::Ordering::Relaxed;
        self.total_tokens.load(Relaxed) + self.round_chars.load(Relaxed) / 4
    }

    fn add_chars(&self, delta: &str) {
        self.round_chars.fetch_add(
            delta.chars().count() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
    }
}

#[async_trait]
impl LifecycleHooks for TeamProgressHook {
    async fn pre_request(&self, _messages: &mut Vec<Message>, _ctx: &TurnCtx) {
        (self.activity)("thinking".to_string(), self.live_tokens());
    }

    async fn on_text_delta(&self, delta: &mut String) {
        self.add_chars(delta);
    }

    async fn on_reasoning_delta(&self, delta: &mut String) {
        self.add_chars(delta);
    }

    async fn on_model_response(&self, response: &mut Message) {
        use std::sync::atomic::Ordering::Relaxed;
        // Finalize this round: prefer the provider's reported completion count,
        // falling back to the chars/4 estimate when usage is unavailable.
        let estimated = self.round_chars.swap(0, Relaxed) / 4;
        let reported = response
            .meta
            .as_ref()
            .map(|meta| meta.tokens.completion as u64)
            .unwrap_or(0);
        self.total_tokens
            .fetch_add(reported.max(estimated), Relaxed);
        // Only surface an activity when the model is about to use a tool. A
        // response WITHOUT a tool call ends the turn -- emitting "thinking" here
        // would just overwrite the last real activity and double the event rate;
        // the final token total is carried out via the member outcome instead.
        if let Some(call) = response.tool_calls.first() {
            (self.activity)(
                format!("using {}", call.name),
                self.total_tokens.load(Relaxed),
            );
        }
    }
}

impl TeamProgressHook {
    fn total_tokens(&self) -> u64 {
        self.total_tokens.load(std::sync::atomic::Ordering::Relaxed)
    }
}

struct DenyTeamBash;

#[async_trait]
impl ToolMiddleware for DenyTeamBash {
    async fn before(
        &self,
        call: &mut ToolCall,
        _tool: &Arc<dyn Tool>,
        _rt: &RequestCtx,
    ) -> BeforeOutcome {
        if call.name == "bash" {
            BeforeOutcome::deny_turn(
                "team child may not run bash; verification remains owned by the parent agent",
            )
        } else {
            BeforeOutcome::Proceed
        }
    }
}

fn team_member_persona(profile: &TeamRoleProfile, scope: &[String]) -> String {
    let authority = match profile.permission {
        TeamPermission::Explore => "You are read-only. Investigate with the mounted read tools and report concise findings.".to_string(),
        TeamPermission::Worker => format!(
            "You may edit only within the assigned scope [{}]. Do not run shell commands. Make the focused change and report what remains for the parent to verify.",
            scope.join(", ")
        ),
    };
    format!(
        "You are the {} Team Agent role.\n{}\n{}\n{}",
        profile.display_name, authority, profile.persona, profile.when_to_use
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream::BoxStream;
    use rustcode_capabilities::team::TeamRoleId;
    use rustcode_kernel::hook::TurnCtx;
    use rustcode_kernel::message::Message;
    use rustcode_kernel::provider::ChatOptions;
    use rustcode_kernel::stream::{ProviderError, StreamEvent};
    use rustcode_kernel::tool::{ToolCall, ToolContext, ToolDef, ToolRegistry, ToolResult};

    struct DummyTool;
    #[async_trait]
    impl Tool for DummyTool {
        fn name(&self) -> &str {
            "dummy"
        }
        fn description(&self) -> &str {
            "dummy"
        }
        fn parameters_schema(&self) -> serde_json::Value {
            serde_json::json!({"type":"object"})
        }
        async fn execute(&self, _args: &str, _ctx: &ToolContext) -> ToolResult {
            ToolResult::default()
        }
    }

    fn request_ctx() -> RequestCtx {
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        RequestCtx::new(tx, None)
    }

    async fn apply(
        middlewares: &[Arc<dyn ToolMiddleware>],
        name: &str,
        args: &str,
    ) -> BeforeOutcome {
        let mut call = ToolCall {
            id: "call".into(),
            name: name.into(),
            arguments: args.into(),
        };
        for middleware in middlewares {
            let outcome = middleware
                .before(
                    &mut call,
                    &(Arc::new(DummyTool) as Arc<dyn Tool>),
                    &request_ctx(),
                )
                .await;
            if outcome != BeforeOutcome::Proceed {
                return outcome;
            }
        }
        BeforeOutcome::Proceed
    }

    #[tokio::test]
    async fn worker_denies_bash_and_out_of_scope_writes() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        let mut middleware = vec![Arc::new(DenyTeamBash) as Arc<dyn ToolMiddleware>];
        middleware.extend(team_child_middlewares(
            true,
            &["src/**".into()],
            dir.path(),
            &[],
        ));
        assert!(apply(&middleware, "bash", r#"{"command":"cargo test"}"#)
            .await
            .is_deny());
        assert!(
            apply(&middleware, "write_file", r#"{"file_path":"../escape.rs"}"#)
                .await
                .is_deny()
        );
        assert_eq!(
            apply(&middleware, "write_file", r#"{"file_path":"src/ok.rs"}"#).await,
            BeforeOutcome::Proceed
        );
        assert!(apply(
            &middleware,
            "read_file",
            r#"{"file_path":"tests/outside.rs"}"#
        )
        .await
        .is_deny());
        assert_eq!(
            apply(&middleware, "read_file", r#"{"file_path":"src/ok.rs"}"#).await,
            BeforeOutcome::Proceed
        );
    }

    #[tokio::test]
    async fn worker_denies_sensitive_paths() {
        let dir = tempfile::tempdir().unwrap();
        let middleware = team_child_middlewares(true, &["src/**".into()], dir.path(), &[]);
        assert!(apply(&middleware, "read_file", r#"{"file_path":".env"}"#)
            .await
            .is_deny());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn worker_denies_write_through_symlink_outside_workspace() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::os::unix::fs::symlink(outside.path(), dir.path().join("src/link")).unwrap();
        let middleware = team_child_middlewares(true, &["src/**".into()], dir.path(), &[]);
        assert!(apply(
            &middleware,
            "write_file",
            r#"{"file_path":"src/link/escape.rs"}"#
        )
        .await
        .is_deny());
    }

    #[tokio::test]
    async fn progress_hook_estimates_tokens_chars_over_four() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let activity: TeamActivitySink = Arc::new(move |text, tokens| {
            let _ = tx.send((text, tokens));
        });
        let hook = TeamProgressHook::new(activity);
        // 8 chars -> 2 tokens（chars/4 估算）。
        let mut delta = "abcdefgh".to_string();
        hook.on_text_delta(&mut delta).await;
        assert_eq!(hook.live_tokens(), 2);
        // pre_request 发布 "thinking" 并携带当前 token 估算。
        hook.pre_request(&mut vec![], &TurnCtx::default()).await;
        let (text, tokens) = rx.try_recv().unwrap();
        assert_eq!(text, "thinking");
        assert_eq!(tokens, 2);
        // on_model_response 发布 "using <tool>"。
        let mut response = Message::assistant(
            "",
            vec![ToolCall {
                id: "1".into(),
                name: "read_file".into(),
                arguments: "{}".into(),
            }],
        );
        hook.on_model_response(&mut response).await;
        let (text, _) = rx.try_recv().unwrap();
        assert_eq!(text, "using read_file");
    }

    struct NamedProvider(&'static str);
    #[async_trait]
    impl LlmProvider for NamedProvider {
        fn model_name(&self) -> &str {
            self.0
        }
        async fn chat_stream(
            &self,
            _messages: &[Message],
            _tools: &[ToolDef],
            _options: &ChatOptions,
        ) -> Result<BoxStream<'static, StreamEvent>, ProviderError> {
            Ok(Box::pin(futures::stream::empty()))
        }
    }

    #[test]
    fn model_factory_maps_difficulty() {
        let providers: TeamProviderFactory = Arc::new(|difficulty| {
            let name = match difficulty {
                TeamDifficulty::Simple => "fast-model",
                TeamDifficulty::Hard => "capable-model",
            };
            Arc::new(NamedProvider(name)) as Arc<dyn LlmProvider>
        });
        let runner = TeamRunnerFactory::new(
            providers,
            Arc::new(|_| ToolRegistry::new().mount(&[])),
            std::env::temp_dir(),
        );
        let models = runner.model_factory();
        let simple = TeamTaskSpec {
            description: "d".into(),
            prompt: "p".into(),
            role: TeamRoleId::Explorer,
            permission: TeamPermission::Explore,
            difficulty: TeamDifficulty::Simple,
            scope: vec![],
        };
        let hard = TeamTaskSpec {
            difficulty: TeamDifficulty::Hard,
            ..simple.clone()
        };
        assert_eq!(models(&simple), "fast-model");
        assert_eq!(models(&hard), "capable-model");
    }

    #[test]
    fn persona_embeds_scope_and_authority() {
        let worker = role_by_id("implementer").unwrap();
        let persona = team_member_persona(worker, &["src/**".into()]);
        assert!(persona.contains("src/**"), "{persona}");
        assert!(persona.contains("Do not run shell commands"), "{persona}");
        let explorer = role_by_id("explorer").unwrap();
        let persona = team_member_persona(explorer, &[]);
        assert!(persona.contains("read-only"), "{persona}");
        assert!(!persona.contains("Do not run shell commands"), "{persona}");
    }

    /// Scripted member provider: `Some(text)` answers in one turn; `None` fails.
    /// `transient` picks a retryable 503 (another model could survive it) versus a
    /// terminal 401 (no model would), which is what the walk branches on.
    struct ScriptedProvider {
        model: &'static str,
        reply: Option<&'static str>,
        transient: bool,
        attempts: Arc<std::sync::Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl LlmProvider for ScriptedProvider {
        fn model_name(&self) -> &str {
            self.model
        }

        async fn chat_stream(
            &self,
            _messages: &[Message],
            _tools: &[ToolDef],
            _options: &ChatOptions,
        ) -> Result<BoxStream<'static, StreamEvent>, ProviderError> {
            self.attempts.lock().unwrap().push(self.model.to_string());
            if let Some(text) = self.reply {
                return Ok(Box::pin(futures::stream::iter(vec![
                    StreamEvent::TextDelta(text.to_string()),
                    StreamEvent::Done { truncated: false },
                ])));
            }
            Err(ProviderError {
                retryable: self.transient,
                http_status: Some(if self.transient { 503 } else { 401 }),
                message: "down".into(),
                ..Default::default()
            })
        }
    }

    /// Build a member task + a recording activity sink.
    fn spec() -> TeamTaskSpec {
        TeamTaskSpec {
            description: "d".into(),
            prompt: "p".into(),
            role: TeamRoleId::Explorer,
            permission: TeamPermission::Explore,
            difficulty: TeamDifficulty::Simple,
            scope: vec![],
        }
    }

    fn make(
        model: &'static str,
        reply: Option<&'static str>,
        transient: bool,
        attempts: &Arc<std::sync::Mutex<Vec<String>>>,
    ) -> Arc<dyn LlmProvider> {
        Arc::new(ScriptedProvider {
            model,
            reply,
            transient,
            attempts: Arc::clone(attempts),
        })
    }

    /// The models attempted, in first-appearance order.
    ///
    /// The kernel retries a retryable failure INSIDE one `run_to_completion`, so the
    /// raw call log repeats a model several times before the walk moves on. That
    /// layering is deliberate (fallback fires only after the retry budget is spent),
    /// so the contract to assert is the ORDER of models, not the call count.
    fn models_tried(attempts: &Arc<std::sync::Mutex<Vec<String>>>) -> Vec<String> {
        let mut seen: Vec<String> = Vec::new();
        for model in attempts.lock().unwrap().iter() {
            if seen.last() != Some(model) {
                seen.push(model.clone());
            }
        }
        seen
    }

    /// FR-6.2: a member walks its explicit chain in order and stops at the first
    /// model that answers. The ORDER is the contract -- asserting only the final
    /// text would pass even if the chain were skipped entirely.
    #[tokio::test]
    async fn member_walks_its_chain_in_order_and_stops_at_the_first_success() {
        let attempts = Arc::new(std::sync::Mutex::new(Vec::new()));
        let a = Arc::clone(&attempts);
        let for_chain = Arc::clone(&attempts);
        let runner = TeamRunnerFactory::new(
            Arc::new(move |_| make("primary", None, true, &a)),
            Arc::new(|_| ToolRegistry::new().mount(&[])),
            std::env::temp_dir(),
        )
        .with_chain_providers(move |_| {
            vec![
                make("hop-1", None, true, &for_chain),
                make("hop-2", Some("RECOVERED"), true, &for_chain),
            ]
        });
        let activities = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let sink = {
            let captured = Arc::clone(&activities);
            Arc::new(move |text: String, _tokens: u64| {
                captured.lock().unwrap().push(text);
            }) as TeamActivitySink
        };

        let outcome = runner.run(spec(), CancellationToken::new(), sink).await;

        assert_eq!(
            models_tried(&attempts),
            vec!["primary", "hop-1", "hop-2"],
            "the chain must be walked in order"
        );
        assert!(outcome.success, "{outcome:?}");
        assert_eq!(outcome.output, "RECOVERED");
        // The failover is visible to the member's rendering channel (FR-6.3).
        let logs = activities.lock().unwrap().join("\n");
        assert!(logs.contains("primary") && logs.contains("hop-1"), "{logs}");
    }

    /// FR-3.6 / FR-4.4: when every model in the chain fails, say so -- the last
    /// provider error must not silently stand in for the whole walk.
    #[tokio::test]
    async fn exhausted_chain_reports_every_attempt() {
        let attempts = Arc::new(std::sync::Mutex::new(Vec::new()));
        let a = Arc::clone(&attempts);
        let for_chain = Arc::clone(&attempts);
        let runner = TeamRunnerFactory::new(
            Arc::new(move |_| make("primary", None, true, &a)),
            Arc::new(|_| ToolRegistry::new().mount(&[])),
            std::env::temp_dir(),
        )
        .with_chain_providers(move |_| vec![make("hop-1", None, true, &for_chain)]);
        let activities = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let sink = {
            let captured = Arc::clone(&activities);
            Arc::new(move |text: String, _tokens: u64| {
                captured.lock().unwrap().push(text);
            }) as TeamActivitySink
        };

        let outcome = runner.run(spec(), CancellationToken::new(), sink).await;

        assert!(!outcome.success, "{outcome:?}");
        // Locale-free assertion: the diagnostic list itself is `model: reason`, so
        // this holds without pinning a language.
        let logs = activities.lock().unwrap().join("\n");
        assert!(
            logs.contains("primary: HTTP 503") && logs.contains("hop-1: HTTP 503"),
            "every attempt must be listed: {logs}"
        );
    }

    /// A-10: with NO chain configured the member keeps the historic single-attempt
    /// behaviour -- one try, and no fallback notice on a plain failure.
    #[tokio::test]
    async fn an_absent_chain_keeps_the_single_attempt_behaviour() {
        let attempts = Arc::new(std::sync::Mutex::new(Vec::new()));
        let a = Arc::clone(&attempts);
        let runner = TeamRunnerFactory::new(
            Arc::new(move |_| make("primary", None, true, &a)),
            Arc::new(|_| ToolRegistry::new().mount(&[])),
            std::env::temp_dir(),
        );
        let activities = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let sink = {
            let captured = Arc::clone(&activities);
            Arc::new(move |text: String, _tokens: u64| {
                captured.lock().unwrap().push(text);
            }) as TeamActivitySink
        };

        let outcome = runner.run(spec(), CancellationToken::new(), sink).await;

        assert_eq!(models_tried(&attempts), vec!["primary"]);
        assert!(!outcome.success, "{outcome:?}");
        // The exhaustion diagnostic carries the `model: reason` list; on an
        // unconfigured member it must NOT appear -- that would be new behaviour.
        let logs = activities.lock().unwrap().join("\n");
        assert!(
            !logs.contains("primary: HTTP 503"),
            "an unconfigured member must not report a spent chain: {logs}"
        );
    }

    /// FR-5.3: a terminal failure (401) is not a model-availability problem, so the
    /// chain must NOT be walked even though candidates remain.
    #[tokio::test]
    async fn a_terminal_failure_does_not_walk_the_chain() {
        let attempts = Arc::new(std::sync::Mutex::new(Vec::new()));
        let a = Arc::clone(&attempts);
        let for_chain = Arc::clone(&attempts);
        let runner = TeamRunnerFactory::new(
            Arc::new(move |_| make("primary", None, false, &a)),
            Arc::new(|_| ToolRegistry::new().mount(&[])),
            std::env::temp_dir(),
        )
        .with_chain_providers(move |_| {
            vec![make("hop-1", Some("SHOULD NOT RUN"), true, &for_chain)]
        });

        let outcome = runner
            .run(
                spec(),
                CancellationToken::new(),
                Arc::new(|_: String, _: u64| {}),
            )
            .await;

        assert_eq!(
            *attempts.lock().unwrap(),
            vec!["primary"],
            "a 401 must not be replayed on another model"
        );
        assert!(!outcome.success, "{outcome:?}");
    }

    /// FR-6.1 / A-5: a member that already produced output must not be replayed --
    /// the second attempt would duplicate whatever the first one did.
    #[tokio::test]
    async fn a_member_that_produced_output_is_never_replayed() {
        // `produced_output` comes from the kernel Outcome, which the scripted
        // provider cannot fake mid-failure, so assert the shared predicate that
        // the walk consults directly.
        let mut with_text = Outcome {
            stop: StopReason::ProviderError,
            text: "half a change".into(),
            ..Default::default()
        };
        with_text.http_status = Some(503);
        assert!(
            !rustcode_capabilities::fallback::fallback_eligible(&with_text, false),
            "producing output must suppress the walk"
        );
        let content_free = Outcome {
            stop: StopReason::ProviderError,
            http_status: Some(503),
            ..Default::default()
        };
        assert!(rustcode_capabilities::fallback::fallback_eligible(
            &content_free,
            false
        ));
    }
}
