use super::messages::Msg;
use std::borrow::Cow;

pub(super) fn en(msg: Msg<'_>) -> Cow<'static, str> {
    match msg {
        Msg::WelcomeBannerLine1 =>
            "Welcome to {brand}. Pick an option to get started:".into(),
        Msg::WelcomeBannerLine2 =>
            "(↑↓ to navigate, Enter to confirm, Esc to skip)".into(),
        Msg::WelcomeOptionCodingPlan => "Set up CodingPlan".into(),
        Msg::WelcomeOptionCodingPlanHint => "Free tokens . recommended".into(),
        Msg::WelcomeOptionConfigureManually => "Configure manually".into(),
        Msg::WelcomeOptionConfigureManuallyHint => "API key".into(),
        Msg::WelcomeOptionSkip => "Skip for now".into(),
        Msg::WelcomeOptionSkipHint => "explore first".into(),

        // ── /login (full setup flow) ──
        Msg::CodingPlanSetupFailed { error } =>
            format!("/login setup failed: {error}").into(),
        Msg::CpReauthAfter401 =>
            "  [!] Stored login expired -- re-authenticating...\n".into(),
        Msg::LoginManagedUnavailable =>
            "  [*] Managed login is not built into this build.\n  \
             Configure a third-party provider directly in ~/.rustcode/config.toml \
             (base_url + api_key), or add a model with its own key via /provider; set \
             RUSTCODE_PLATFORM_SERVER to point at a self-hosted gateway.\n"
                .into(),
        Msg::ChatAuthExpired =>
            "Authentication expired -- please run /login to sign in again".into(),
        Msg::ProviderErrEntitlement403 =>
            "This model plan is not enabled for the account or the authorization has expired (HTTP 403); check your API key permissions and account status.".into(),
        Msg::ProviderErrUnauthorized { code } =>
            format!("API key unauthorized or invalid (HTTP {code})").into(),
        Msg::ProviderErrInsufficientBalance { code } =>
            format!("Insufficient account balance (HTTP {code})").into(),
        Msg::ProviderErrConnResetRetried { attempts } => format!(
            "Network connection interrupted: the remote end closed or reset the connection; {attempts} automatic retries still failed. You can retry."
        )
        .into(),
        Msg::ProviderErrConnResetPartial =>
            "Response interrupted: to avoid duplicate output or tool execution, it was not automatically replayed; the safely-saved partial reply is kept and you can continue.".into(),
        Msg::ProviderErrDetailLabel => "Details".into(),
        Msg::ProviderErrCorpProxyHint =>
            "This error is common on corporate networks or proxy setups; check your proxy/VPN/firewall settings and retry.".into(),
        Msg::ProviderErrProxyNamed { proxy } => format!("proxy {proxy}").into(),
        Msg::ProviderErrProxyConfigured => "the configured proxy".into(),
        Msg::ProviderErrProxyUnreachable { who } => format!(
            "Could not connect to {who} (the proxy may not be running or the address is unreachable). If you don't need a proxy, run /proxy and choose no_proxy (no proxy), then retry."
        )
        .into(),
        Msg::ProviderErrTtfbTimeout { secs } =>
            format!("timed out waiting for the first byte after {secs}s (gateway not responding)").into(),
        Msg::ProviderErrEffortUnsupported =>
            "Provider rejected reasoning_effort; auto-disabled for this session -- resend to continue.".into(),
        Msg::ToolProgressParallelEdit { count } =>
            format!("Parallel editing {count} files (subagents)").into(),
        Msg::RoundCapHeader => "Round limit".into(),
        Msg::RoundCapQuestion { cap } => format!("Ran {cap} rounds. Continue?").into(),
        Msg::RoundCapQuestionStats { cap, stats } =>
            format!("Ran {cap} rounds ({stats}). Continue?").into(),
        Msg::RoundCapContinue => "Continue".into(),
        Msg::RoundCapContinueDesc { base } =>
            format!("Run {base} more rounds, then ask again").into(),
        Msg::RoundCapStop => "Stop".into(),
        Msg::RoundCapStopDesc => "End this turn".into(),
        Msg::GitRepoRootEmpty => "git did not return a repository root".into(),
        Msg::GitCmdFailed { cmd, detail } => format!("git {cmd} failed: {detail}").into(),
        Msg::GitOutputTooLarge { cmd, kib } =>
            format!("git {cmd} output exceeded {kib} KiB; cannot display reliably").into(),
        Msg::GitSpawnFailed { error } => format!("failed to launch git: {error}").into(),
        Msg::GitStdoutUnavailable => "could not read git output".into(),
        Msg::GitStderrUnavailable => "could not read git error output".into(),
        Msg::GitWaitFailed { error } =>
            format!("failed while waiting for git to exit: {error}").into(),
        Msg::GitTimeout { secs } => format!("git command timed out after {secs}s").into(),
        Msg::GitStatusPollFailed { error } => format!("failed to poll git status: {error}").into(),
        Msg::GitStdoutThreadPanicked =>
            "the thread reading git output exited unexpectedly".into(),
        Msg::GitStdoutReadFailed { error } => format!("failed to read git output: {error}").into(),
        Msg::GitStderrThreadPanicked =>
            "the thread reading git error output exited unexpectedly".into(),
        Msg::GitStderrReadFailed { error } =>
            format!("failed to read git error output: {error}").into(),
        Msg::GitNumstatMissingPath => "git numstat record missing file path".into(),
        Msg::GitNumstatMissingOldPath => "git numstat record missing pre-rename path".into(),
        Msg::GitNumstatMissingNewPath => "git numstat record missing post-rename path".into(),
        Msg::GitNumstatMissingCount => "git numstat record missing change counts".into(),
        Msg::GitNumstatCountNotUtf8 => "git numstat count is not valid UTF-8".into(),
        Msg::GitNumstatCountInvalid { text } =>
            format!("git numstat count is invalid: {text}").into(),
        Msg::GoalCapRound { max } =>
            format!("Round budget reached ({max} rounds). Continue the conversation to advance.").into(),
        Msg::GoalCapRoundNoMax =>
            "Round budget reached. Continue the conversation to advance.".into(),
        Msg::GoalCapTime => "Time limit reached. Continue the conversation to advance.".into(),
        Msg::GoalCapStopped { other } =>
            format!("Stopped ({other}). Continue the conversation to advance.").into(),
        Msg::RetryReasonRateLimited => "rate-limited or out of credit".into(),
        Msg::RetryReasonUpstream => "upstream service temporarily unavailable".into(),
        Msg::RetryReasonTimeout => "model response timed out".into(),
        Msg::RetryReasonNetwork => "network connection failed".into(),
        Msg::TuixProviderRetry {
            reason,
            backoff_secs,
            attempt,
            max_attempts,
        } => format!(
            "API error {reason} -- retrying in {backoff_secs}s ({attempt}/{max_attempts})..."
        )
        .into(),
        Msg::TuixLoopUsage =>
            "usage: /loop <interval> <prompt or /command>, e.g. /loop 5m /diff".into(),
        Msg::TuixLoopSelfRef => "cannot loop /loop on itself".into(),
        Msg::TuixLoopIntervalRange => "interval must be between 10s and 24h".into(),
        Msg::TuixRateLimitAutoResume { secs } =>
            format!("⏳ rate limited, resuming automatically in {secs}s...").into(),
        Msg::TuixRateLimit429 { reason, tail } => format!(
            "⏸ rate limited (HTTP 429){reason}{tail} · completed work kept · retry later or switch model"
        )
        .into(),
        Msg::TuixRateLimitRetryAfter { dur } =>
            format!(" (retry available in about {dur})").into(),
        Msg::TuixRateLimitWindowNoTime { tail } => format!(
            "⏸ 5-hour quota window exhausted, resuming later{tail} · completed work kept · switch model or retry later"
        )
        .into(),
        Msg::TuixRateLimitWindowWithTime { reset_at, tail } => format!(
            "⏸ 5-hour quota window exhausted, resuming around {reset_at}{tail} · completed work kept · switch model or retry later"
        )
        .into(),
        Msg::TuixRateLimitWindowRemaining { dur } => format!(" ({dur} remaining)").into(),
        Msg::TuixToolBatchSameParallel { count, tool } =>
            format!("Running {count} {tool} calls in parallel").into(),
        Msg::TuixToolBatchSame { count, tool } =>
            format!("Running {count} {tool} calls").into(),
        Msg::TuixToolBatchParallel { count } => format!("Running {count} tools in parallel").into(),
        Msg::TuixToolBatch { count } => format!("Running {count} tools").into(),
        Msg::TuixRuntimeDeliveryFailed { operation } =>
            format!("coding runtime {operation} delivery failed").into(),
        Msg::TuixShellContextQueueFailed =>
            "[failed to add shell output to runtime context]".into(),
        Msg::TuixProviderReloadStartFailed { error } =>
            format!("provider reload could not be started: {error}").into(),
        Msg::TuixConfigRollbackFailed { error } =>
            format!("; config rollback failed: {error}").into(),
        Msg::TuixProviderNoLongerAvailable { name } =>
            format!("provider {name:?} is no longer available").into(),
        Msg::TuixFoldNoOutput => "(no output)".into(),
        Msg::TuixFoldLinesSuffix { count } => format!(" ({count} lines)").into(),
        Msg::KernelNoticeEmptyRetryMalformed {
            wait_secs,
            attempt,
            max,
        } => format!("Malformed response; retrying in {wait_secs}s ({attempt}/{max})...").into(),
        Msg::KernelNoticeEmptyRetryEmpty {
            wait_secs,
            attempt,
            max,
        } => format!(
            "Model returned an empty response; retrying in {wait_secs}s ({attempt}/{max})..."
        )
        .into(),
        Msg::KernelNoticeReplyTruncated =>
            "The model's reply reached the length limit and may be incomplete; ask it to continue and it will finish the rest.".into(),
        Msg::KernelNoticeOverWindow { est_k, window_k } => format!(
            "The request is about {est_k}K tokens, near the model's usable limit (window about {window_k}K; room must be reserved for the reply); trim the input or switch to a larger-window model."
        )
        .into(),
        Msg::KernelNoticeEmptyExhMalformed { max_retries } => format!(
            "The model returned {max_retries} consecutive unparseable responses (transient upstream fault). You can retry now or later."
        )
        .into(),
        Msg::KernelNoticeEmptyExhOverWindowBrief { max_retries } => format!(
            "The model returned {max_retries} consecutive empty responses. As noted earlier, this request exceeds the model's context window -- trim the input or /compact and retry."
        )
        .into(),
        Msg::KernelNoticeEmptyExhOverWindowFull {
            max_retries,
            est_k,
            window_k,
        } => format!(
            "The model returned {max_retries} consecutive empty responses. The request is about {est_k}K tokens, at or over the context window (about {window_k}K), so it is likely too large. Try /compact or trim the input and retry."
        )
        .into(),
        Msg::KernelNoticeEmptyExhTransient { max_retries } => format!(
            "The model returned {max_retries} consecutive empty responses (transient upstream fault, unrelated to context length). You can retry now or later."
        )
        .into(),
        Msg::NetworkConnectHint =>
            "Network connect failed. If this works in a browser you may be behind a proxy/firewall: configure a proxy with /proxy or set HTTPS_PROXY, or open the login URL above in a browser to finish. Press Esc to skip and /login later.".into(),
        Msg::CpSetupHeader =>
            "  {brand} CodingPlan setup:\n\n".into(),
        Msg::CpLoggedIn { who, username, email } =>
            format!("  [+] Logged in as {} ({}, {})\n", who, username, email).into(),
        Msg::CpStepSkipped { reason } =>
            format!("  [+] {}\n", reason).into(),
        Msg::CpLoginFailed { error } =>
            format!("  [x] Login failed -- {}\n", error).into(),
        Msg::CpClaimed { message, plan_type } =>
            format!("  [+] CodingPlan claimed -- {} (CodingPlan {})\n", message, plan_type).into(),
        Msg::CpClaimSuccessFallback => "success".into(),
        Msg::CpAlreadyClaimed { reason } =>
            format!("  [+] CodingPlan already claimed -- {}\n", reason).into(),
        Msg::CpClaimFailed { error } =>
            format!("  [x] CodingPlan tier setup failed -- {}\n", error).into(),
        Msg::CpClaimFailedBare =>
            "  [x] CodingPlan tier setup failed\n".into(),
        Msg::CpClaimTierSucceeded { plan } =>
            format!("  [+] {} active\n", plan).into(),
        Msg::CpClaimTierAlreadyHeld { plan } =>
            format!("  [+] {} active\n", plan).into(),
        Msg::CpClaimTierFailed { tier, reason } =>
            format!("  [x] CodingPlan {} tier setup failed -- {}\n", tier, reason).into(),
        Msg::CpAddedProviders { accounts, models } =>
            format!(
                "  [+] Added {} account{} . {} model{}:\n",
                accounts,
                if accounts == 1 { "" } else { "s" },
                models,
                if models == 1 { "" } else { "s" }
            )
            .into(),
        Msg::CpLocked { name } =>
            // SGR 31 = standard red foreground, SGR 39 = reset to
            // default fg. Standard (not bright) so the terminal's
            // theme palette decides the exact shade -- Solarized,
            // Dracula, light-mode, etc. all map this onto their
            // own "red" rather than a hard-coded RGB the user can't
            // tune. The `[x] ... (requires Pro plan or higher)` text inside is
            // a redundant signal so retained-mode terminals (which
            // strip SGR via the strict sanitizer path) still get
            // the meaning, just without the colour.
            format!("      \x1b[31m[x] {}  (requires Pro plan or higher)\x1b[39m\n", name).into(),
        Msg::CpProviderRow { provider, model, default_suffix } =>
            format!("      * {}  .  {}{}\n", provider, model, default_suffix).into(),
        Msg::CpDefaultSuffix => "  (default)".into(),
        Msg::CpVisionAuto { kind } =>
            format!("  [+] Vision preprocessor -> {}  (auto-detected)\n", kind).into(),
        Msg::CpVisionUserSupplied { kind } =>
            format!("  [+] Vision preprocessor -> {}  (user setting kept)\n", kind).into(),
        Msg::CpVisionCleared =>
            "  [!] Vision preprocessor cleared -- no VL/OCR model in current list\n".into(),
        Msg::CpModelsSkipped { reason } =>
            format!("  [+] Models step skipped -- {}\n", reason).into(),
        Msg::CpModelsFailed { error } =>
            format!("  [x] Models step failed -- {}\n", error).into(),
        Msg::CpStatusHeader =>
            "  [+] CodingPlan status:\n".into(),
        Msg::CpPlanPending { plan } =>
            format!("      Plan: {}  .  pending activation\n", plan).into(),
        Msg::CpPlanActive { plan, expires_at, remaining_days, total_days } =>
            format!(
                "      Plan: {}  .  expires {} ({}d / {}d remaining)\n",
                plan, expires_at, remaining_days, total_days,
            ).into(),
        Msg::CpUsageLine { usage, reset_at, duration } =>
            format!("      Usage: {}  .  resets {} (in {})\n", usage, reset_at, duration).into(),
        Msg::CpWindowQuotaExhausted =>
            "      [!] Current window quota exhausted\n".into(),
        Msg::CpWindowQuotaHint { hint } =>
            format!("      [!] {}\n", hint).into(),
        Msg::CpStatusFetchSkipped { reason } =>
            format!("  [!] Status fetch skipped -- {}\n", reason).into(),
        Msg::CpStatusFetchFailed { error } =>
            format!("  [!] Status fetch failed (non-fatal) -- {}\n", error).into(),
        Msg::CpOfficialBuildRequired => Cow::Borrowed(
            "The configured CodingPlan gateway needs a build with managed-signing \
             support, which this {brand} build does not include. Configure a third-party \
             provider (bring your own key) instead, or use a distribution that ships \
             CodingPlan support.",
        ),
        Msg::CpAuthRequired => Cow::Borrowed(
            "Not signed in to {brand} CodingPlan. Run /login to sign in \
             before sending a request.",
        ),
        Msg::CpSignStaleClockSkew => Cow::Borrowed(
            "Request rejected: signed timestamp outside the accepted window. \
             Please check your system clock (NTP sync) and retry.",
        ),
        Msg::CpSignReplayPersisted => Cow::Borrowed(
            "Request was repeatedly flagged as a replay. Please try the command again.",
        ),
        Msg::CpSignVersionTooOld => Cow::Borrowed(
            "{brand} is out of date and no longer compatible with CodingPlan. \
             Please upgrade {brand} to continue.",
        ),
        Msg::CpUpgradeRequired => Cow::Borrowed(
            "An upgrade is required to continue using CodingPlan. \
             Please update to a newer {brand} build from your distribution, or configure \
             a third-party provider (bring your own key) with /provider.",
        ),

        Msg::ErrUnsupportedLocale { input } =>
            format!("unsupported locale: {input}").into(),

        // ── Status bar ──
        Msg::StatusNoProvider =>
            "no provider . /provider to configure".into(),
        Msg::StatusRuntimeUnavailable =>
            "runtime unavailable . restart or inspect the error above".into(),
        Msg::StatusOfficialBuildRequired =>
            "CodingPlan unsupported in this build".into(),
        Msg::StatusUpgradeHint { version } =>
            format!("↑ {version} available . /upgrade").into(),
        Msg::StatusUpgradeHintPm { version } =>
            format!("↑ {version} available . brew upgrade rustcode").into(),
        Msg::StatusModelNotConfigured =>
            "(not configured)".into(),
        Msg::StatusClipboardImageHint =>
            "Image in clipboard . ctrl+v / ctrl+alt+v to paste".into(),
        Msg::StatusClipboardImageHintSlash =>
            "Image in clipboard . /paste".into(),
        Msg::StatusWebuiHint =>
            "Tips: Use /webui to open {brand} in your browser".into(),

        // ── /status command body ──
        Msg::StatusBody { model, dir, config } =>
            format!(
                "  Model:  {}\n  Dir:    {}\n  Config: {}\n",
                model, dir, config,
            ).into(),
        Msg::StatusLoginLoggedIn { user } =>
            format!("  Login:  {}\n", user).into(),
        Msg::StatusLoginNotSignedIn =>
            "  Login:  not signed in (run /login)\n".into(),
        Msg::StatusCpNotSignedIn =>
            "  CodingPlan: (not signed in -- run /login to set up)\n".into(),
        Msg::StatusCpFetchFailed { error } =>
            format!("  CodingPlan: (status fetch failed -- {})\n", error).into(),
        Msg::StatusCpAuthExpired =>
            "  CodingPlan: (login expired -- run /login to sign in again)\n".into(),
        Msg::StatusCpNoActive =>
            "  CodingPlan: (no active plan -- run /login)\n".into(),
        Msg::StatusCpLine { plan, expires_at, remaining_days, total_days } =>
            format!(
                "  CodingPlan: {}  .  expires {} ({}d/{}d)\n",
                plan, expires_at, remaining_days, total_days,
            ).into(),
        Msg::StatusCpUsage { usage, reset_at, duration } =>
            format!("  Usage: {}  .  resets {} (in {})\n", usage, reset_at, duration).into(),
        Msg::StatusCpWindowExhausted =>
            "  [!] Current window quota exhausted\n".into(),
        Msg::StatusCpWindowHint { hint } =>
            format!("  [!] {}\n", hint).into(),
        Msg::StatusInstructionFilesHeader =>
            "  Instruction files:\n".into(),
        Msg::StatusInstructionScopeGlobal => "User global".into(),
        Msg::StatusInstructionScopeProject => "Project shared".into(),
        Msg::StatusInstructionScopeUser => "User project override".into(),
        Msg::StatusInstructionPresent { path, label, scope } =>
            format!("    [+] {scope} ({label}): {path}\n").into(),
        Msg::StatusInstructionMissing { path, label, scope } =>
            format!("    [x] {scope} ({label}): {path} -- not found\n").into(),
        Msg::StatusMemoryFilesHeader => "  Memory files:\n".into(),
        Msg::StatusMemoryScopeGlobal => "User global".into(),
        Msg::StatusMemoryScopeProject => "Project memory".into(),
        Msg::StatusMemoryPresent { path, scope } =>
            format!("    [+] {scope}: {path}\n").into(),
        Msg::StatusMemoryMissing { path, scope } =>
            format!("    [x] {scope}: {path} -- not found\n").into(),

        // ── Help ──
        Msg::HelpAvailableCommands =>
            "  Available commands:\n".into(),
        Msg::KeybindingsHelp => r#"  Keyboard shortcuts

  ── Input ──
    Enter                            Send message
    \ then Enter                     Insert newline (works in every terminal)
    Shift / Alt / Ctrl+Enter         Insert newline *
    Ctrl+J                           Insert newline *
    /                                Open slash command menu
    Tab                              Accept slash-command or file completion
    Backspace / Ctrl+H               Delete previous char
    Delete / Ctrl+?                  Delete next char
    Ctrl+W                           Delete word backward
    Ctrl+U                           Clear current line
    Ctrl+K                           Delete to end of line
    Ctrl+A / Home                    Jump to line start
    Ctrl+E / End                     Jump to line end
    Left / Right                     Move cursor

  ── History ──
    Up / Down                        Previous / next input
    Ctrl+R                           Reverse-search; press again for older match
    Right                            Accept next-prompt suggestion (does not send)

  ── Mode and model ──
    Shift+Tab                        Without a menu, cycle to the next mode
    F2 / Shift+F2                    Next / previous model (Mac: Fn+F2 / Fn+Shift+F2)
    Ctrl+T                           Cycle reasoning_effort

  ── Browse output ──
    Shift+Up / Shift+Down            Scroll up / down one line
    PageUp / PageDown                Scroll up / down 10 lines
    Alt+Up / Alt+Down                Jump to previous / next message
    Ctrl+Up / Ctrl+Down              Jump to previous / next user message
    Home / End                       With empty input, jump to top / bottom
    Mouse wheel                      Scroll the chat area
    Mouse drag                       Select text
    Shift+mouse drag                 Use the terminal's native selection
    Ctrl+Shift+C                     Copy the {brand} selection

  ── Control flow ──
    Esc                              Clear input / close modal / cancel action
    Esc Esc                          Undo previous turn when idle with empty input
    Ctrl+C                           Cancel action; press again while idle to exit
    Ctrl+O                           Toggle tool real-time output
    Ctrl+V / Ctrl+Alt+V              Paste text or image **

  ── Slash menu / modal navigation ──
    Up / Down                        Move selection
    Enter                            Confirm
    Esc                              Cancel / close modal
    Tab                              Insert highlighted slash command
    1..9                             Pick an approval / question option
    y / a / n                        Approval: allow once / always / deny

  * Modified Enter requires a terminal that disambiguates modifiers.
     Known-supported: Kitty / WezTerm / iTerm2 (with Report Modifiers
     enabled) / Windows Terminal / Ghostty / Warp. Other terminals
     (macOS Apple Terminal, default xterm, GNOME Terminal, VS Code's
     integrated terminal) collapse Shift+Enter into plain Enter --
     use \ + Enter instead.
  ** Ctrl+Alt+V is the fallback when the terminal intercepts Ctrl+V;
     Ctrl+Shift+V remains the terminal's plain-text paste shortcut.

  Tip: run /help for the full slash command list.
"#.into(),

        // ── Provider wizard ──
        Msg::ProviderWizardHeader =>
            "  Manage providers: add, edit, delete, or set the global default. Press Esc to cancel.\n".into(),
        Msg::ProviderWizardCancelled =>
            "(cancelled)".into(),
        Msg::ProviderMenuAdd => "Add".into(),
        Msg::ProviderMenuAddDesc => "Create a provider configuration".into(),
        Msg::ProviderMenuEdit => "Edit".into(),
        Msg::ProviderMenuEditDesc => "Modify an existing provider configuration".into(),
        Msg::ProviderMenuDelete => "Delete".into(),
        Msg::ProviderMenuDeleteDesc => "Delete an existing provider configuration".into(),
        Msg::ProviderMenuSetDefault => "Set global default".into(),
        Msg::ProviderMenuSetDefaultDesc =>
            "Set the default provider and switch this session".into(),
        Msg::ProviderImportPrompt =>
            "Paste a template to auto-detect (curl / JSON / TOML), or Enter to fill manually:".into(),
        Msg::ProviderImportParsed { base_url, type_name, model } =>
            format!("Detected: {base_url} . {type_name} . {model}").into(),
        Msg::ProviderImportFailed =>
            "Not recognized as a template. Paste curl / JSON / TOML, or Enter to fill manually.".into(),
        Msg::ProviderNoProviders =>
            "No providers configured yet.".into(),
        Msg::ProviderDeleteConfirm { name } =>
            format!("Delete \"{name}\"? [y/N]").into(),
        Msg::ProviderDeleted { name } =>
            format!("Deleted \"{name}\".").into(),
        Msg::ProviderDeleteKept => "(kept)".into(),
        Msg::ProviderDefaultSet { name } =>
            format!("Default set to {name}.").into(),
        Msg::ProviderAdded { name } =>
            format!("Added account \"{name}\". Opened its model list; press Ctrl+A to add a model.").into(),
        Msg::ProviderUpdated { name } =>
            format!("Updated \"{name}\".").into(),
        Msg::ProviderStepName => "Provider name?".into(),
        Msg::ProviderStepType => "Type? (openai / claude / ollama)".into(),
        Msg::ProviderStepTypeWithHint { current } =>
            format!("Type? [{current}] (openai / claude / ollama, blank to keep)").into(),
        Msg::ProviderStepBaseUrl =>
            "Base URL? (e.g. https://api.example.com/v1 -- your third-party provider's endpoint)".into(),
        Msg::ProviderStepBaseUrlWithHint { current } =>
            format!("Base URL? [{current}] (blank to keep)").into(),
        Msg::ProviderDefaultHint => "provider default".into(),
        Msg::ProviderStepApiKey =>
            "API key? (blank to leave unset)".into(),
        Msg::ProviderStepApiKeyWithHint { hint } =>
            format!("API key? [{hint}]").into(),
        Msg::ProviderStepApiKeySet => "set -- blank to keep".into(),
        Msg::ProviderStepApiKeyUnset => "unset".into(),
        Msg::ProviderStepModel => "Model?".into(),
        Msg::ProviderStepModelWithHint { current } =>
            format!("Model? [{current}] (blank to keep)").into(),
        Msg::ProviderStepContextWindow { default } =>
            format!("Context window? [{default}] tokens (blank to use default; e.g. 128000 / 256000 / 512000 / 1000000, or 128k / 1m)").into(),
        Msg::ProviderStepContextWindowWithHint { current } =>
            format!("Context window? [{current}] tokens (blank to keep; e.g. 128000 / 256000 / 512000 / 1000000, or 128k / 1m)").into(),
        Msg::ProviderContextWindowInvalid =>
            "Context window must be a positive number of tokens, e.g. 128000 or 128k.".into(),
        Msg::ProviderNameEmpty => "Name cannot be empty.".into(),
        Msg::ProviderBaseUrlEmpty => "Base URL cannot be empty.".into(),
        Msg::ProviderUnknownType =>
            "Unknown type. Choose openai / claude / ollama.".into(),
        Msg::ProviderUnknownTypeEdit =>
            "Unknown type. Choose openai / claude / ollama or leave blank.".into(),
        Msg::ProviderModelEmpty => "Model cannot be empty.".into(),
        Msg::ProviderEditKeep => "(keep)".into(),
        Msg::ProviderTypeInferred { type_name } =>
            format!("Detected type: {type_name}").into(),
        Msg::ProviderStepNameDefault { default } =>
            format!("Provider name? [{default}] (blank to use this)").into(),
        Msg::ProviderStepProgress { current, total } =>
            format!("({current}/{total})").into(),

        // ── Provider panel ──
        Msg::ProviderPanelTabAccounts => "Accounts".into(),
        Msg::ProviderPanelTabModels => "Models".into(),
        Msg::ProviderPanelEmptyAccounts =>
            "(No provider accounts yet -- press Ctrl+A to add one)".into(),
        Msg::ProviderPanelNoMatchingAccounts => "(No matching provider accounts)".into(),
        Msg::ProviderPanelEmptyModels =>
            "(No models yet -- press Ctrl+A to add one)".into(),
        Msg::ProviderPanelNoMatchingModels => "(No matching models)".into(),
        Msg::ProviderPanelLegacyBadge => "legacy".into(),
        Msg::ProviderPanelDefaultBadge => "default".into(),
        Msg::ProviderPanelModelCount { count } =>
            format!("{count} model{}", if count == 1 { "" } else { "s" }).into(),
        Msg::ProviderPanelAddModelRow => "+ Add model".into(),
        Msg::ProviderPanelAccountsHint =>
            "Filter . ↑↓ select . ↵ models . Ctrl+A add . Ctrl+E edit . Ctrl+Dx2 delete . Tab switch . Esc close".into(),
        Msg::ProviderPanelManagedAccountHint =>
            "Managed CodingPlan account . view only . ↵ models . Tab switch . Esc close".into(),
        Msg::ProviderPanelManagedAccountHintNeutral =>
            "Reserved managed-account name; this build has no managed service . view only . ↵ models . Tab switch . Esc close".into(),
        Msg::ProviderPanelModelsHint =>
            "Filter . ↑↓ select . ↵ default/add . Ctrl+A add . Ctrl+E edit . Ctrl+Dx2 delete . Tab switch . Esc close".into(),
        Msg::ProviderPanelManagedModelsHint =>
            "CodingPlan models are managed by /login . ↑↓ select . ↵ default . Tab all . Esc close".into(),
        Msg::ProviderPanelManagedModelsHintNeutral =>
            "Reserved managed name; this build has no managed service . ↑↓ select . ↵ default . Tab all . Esc close".into(),
        Msg::ProviderPanelFilteredModelsHint { account } =>
            format!("[{account}] . ↑↓ select . ↵ default/add . Ctrl+A add model . Ctrl+E edit . Ctrl+Dx2 delete . Tab all . Esc close").into(),
        Msg::ProviderPanelModelSaved { model } => format!("Saved model \"{model}\".").into(),
        Msg::ProviderPanelAddTitle => "[Add provider account]".into(),
        Msg::ProviderPanelEditAccountTitle { account } =>
            format!("[Edit account {account}]").into(),
        Msg::ProviderPanelAddModelTitle => "[Add model]".into(),
        Msg::ProviderPanelEditModelTitle => "[Edit model]".into(),
        Msg::ProviderPanelFieldVendor => "Provider".into(),
        Msg::ProviderPanelFieldAccount => "Account".into(),
        Msg::ProviderPanelFieldBaseUrl => "Base URL".into(),
        Msg::ProviderPanelFieldApiKey => "API key".into(),
        Msg::ProviderPanelFieldModel => "Model".into(),
        Msg::ProviderPanelFieldVision => "Image input".into(),
        Msg::ProviderPanelVisionAuto => "Auto".into(),
        Msg::ProviderPanelVisionEnabled => "Enabled".into(),
        Msg::ProviderPanelVisionDisabled => "Disabled".into(),
        Msg::ProviderPanelFieldEffort => "Default reasoning effort".into(),
        Msg::ProviderPanelFieldEffortLevels => "Supported levels".into(),
        Msg::ProviderPanelFieldWindow => "Context window".into(),
        Msg::ProviderPanelFieldMakeDefault => "Set as default".into(),
        Msg::ProviderPanelSwitchHint => "←-> to switch".into(),
        Msg::ProviderPanelEnvHint { env } => format!("blank uses ${env}").into(),
        Msg::ProviderPanelDefaultValue => "default".into(),
        Msg::ProviderPanelKeepOriginal => "blank keeps current value".into(),
        Msg::ProviderPanelProviderFormHint =>
            "Tab Next  ←-> Switch provider  Space Toggle  ↵ Save  Esc Back".into(),
        Msg::ProviderPanelAccountFormHint => "Tab Switch  ↵ Save  Esc Back".into(),
        Msg::ProviderPanelModelFormHint =>
            "Tab Next  ←-> Switch option  Space Toggle  ↵ Save  Esc Back".into(),
        // ── Model picker ──
        Msg::ModelSwitched { provider, model } =>
            format!("  Switched to {provider} . {model} for this session\n").into(),
        Msg::ModelSwitchedAndDefault { provider, model } =>
            format!("  Switched to {provider} . {model}; set as default for new sessions\n").into(),

        // ── Session picker ──
        Msg::SessionLoadFailed { error } =>
            format!("load session failed: {error}").into(),
        Msg::SessionResumeInProgress =>
            "another session resume is still in progress".into(),
        Msg::SessionPrepJoinFailed { error } =>
            format!("session preparation task failed: {error}").into(),
        Msg::SessionNotFoundById { session_id } =>
            format!("session {session_id} not found").into(),
        Msg::SessionPrepTimeout =>
            "session preparation is taking unusually long (large session or slow disk); please try again".into(),
        Msg::ProjectFallbackWord => "project".into(),
        Msg::SessionResumedLabel { name } =>
            format!("resumed: {name}").into(),
        Msg::SessionBusyForked { source_id, fork_id } =>
            format!(
                "The latest session ({source_id}) is active in another window. \
                 Created an independent fork ({fork_id}) from its last committed state."
            ).into(),

        // ── Todo panel ──
        Msg::TodoPanelTitle => "Todos".into(),
        Msg::TodoPanelCompleted { n } => format!("{n} completed").into(),
        Msg::TodoPanelMore { n } => format!("+{n} more...").into(),

        // ── Approval panel ──
        Msg::ApprovalAllowOnce => "Allow once".into(),
        Msg::ApprovalAlwaysAllow { tool } => format!("Always allow {tool} (this session)").into(),
        Msg::ApprovalAlwaysAllowFolder => {
            "Always allow writes to this folder (this session)".into()
        }
        Msg::ApprovalAlwaysAllowCommand => "Always allow this command (this session)".into(),
        Msg::ApprovalDeny => "Deny".into(),
        Msg::ApprovalHint => "↑↓ select . Enter confirm . Esc cancel".into(),
        Msg::ApprovalHeader { tool, detail } => {
            if detail.is_empty() {
                format!("Allow {tool}?").into()
            } else {
                format!("Allow {tool}({detail})?").into()
            }
        }
        Msg::CredentialApprovalNote => {
            "[!] May send credentials or sensitive content to the model provider".into()
        }
        Msg::ToolDenied => "denied".into(),
        Msg::ToolBlockedBySecurityPolicy =>
            "Tool call blocked by security policy: credentials cannot be passed through generic shell arguments, temporary files, or environment variables".into(),
        Msg::PolicyRecoveryHeader => "Security decision required".into(),
        Msg::PolicyRecoveryQuestion => "The credential operation was blocked. What should {brand} do next?".into(),
        Msg::PolicyRecoveryComplete => "I completed it externally".into(),
        Msg::PolicyRecoveryCompleteDesc => "Acknowledge external completion without calling the model".into(),
        Msg::PolicyRecoverySkip => "Skip this step".into(),
        Msg::PolicyRecoverySkipDesc => "Skip the authenticated step and end recovery without calling the model".into(),
        Msg::PolicyRecoveryInstructions => "View safe instructions".into(),
        Msg::PolicyRecoveryInstructionsDesc => "Show fixed local guidance with placeholders only".into(),
        Msg::PolicyRecoveryEnd => "End task".into(),
        Msg::PolicyRecoveryEndDesc => "Close this intervention without calling the model".into(),
        Msg::PolicyRecoverySafeInstructions => "Safe manual path:\n  1. Open a separate terminal you control.\n  2. Use the service's documented login flow or a credential-aware typed tool.\n  3. Complete the authenticated operation there; do not paste the secret into {brand}.\n  4. Return here and choose ‘I completed it externally’.\n{brand} intentionally does not reconstruct or display the rejected command.".into(),
        Msg::PolicyRecoveryCompletedLocally => "External completion acknowledged. To avoid regenerating sensitive commands, recovery did not call the model; you may now submit a task that does not involve credentials.".into(),
        Msg::PolicyRecoverySkippedLocally => "The credential-dependent step was skipped. To avoid regenerating sensitive commands, recovery did not call the model; you may now submit another safe task.".into(),
        Msg::PolicyRecoverySubmitError => "Could not submit the security recovery decision. Please retry.".into(),

        Msg::CmdSwitchedAutoMode => "  Switched to auto mode (all tools auto-approved).\n".into(),
        Msg::CmdSwitchedAcceptEditsMode => {
            "  Switched to accept-edits mode (file edits auto-approved; bash still prompts).\n".into()
        }

        Msg::SessionTimeJustNow => "just now".into(),
        Msg::SessionTimeMinAgo { n } => format!("{n}m ago").into(),
        Msg::SessionTimeHourAgo { n } => format!("{n}h ago").into(),
        Msg::SessionTimeDayAgo { n } => format!("{n}d ago").into(),
        Msg::SessionMsgCount { count } =>
            format!("{count} msgs").into(),
        Msg::SessionNameEmpty =>
            "Session name cannot be empty".into(),
        Msg::SessionNameTooLong { max } =>
            format!("Session name too long (max {max} characters)").into(),
        Msg::SessionNameControlChars =>
            "Session name cannot contain control characters".into(),
        Msg::SessionListFailed { error } =>
            format!("list sessions failed: {error}").into(),
        Msg::SessionRenamed { old, new } =>
            format!("  Renamed: '{old}' -> '{new}'").into(),
        Msg::SessionSaveFailed { error } =>
            format!("Failed to save session: {error}. The name was not persisted.").into(),
        Msg::SessionNoneSelected =>
            "No session selected".into(),
        Msg::SessionPickerHint =>
            "↑↓ move . Enter open . Ctrl+D[x]2 delete . Type to search . Esc cancel".into(),
        Msg::SessionPickerTitle { n, total, project } =>
            format!("Resume session ({n}/{total} . {project})").into(),
        Msg::SessionPickerTitleBare =>
            "Resume session".into(),
        Msg::SessionPickerEmptyProject =>
            "(no sessions in this project yet)".into(),
        Msg::SessionPickerEmptyFilter =>
            "(no sessions match)".into(),
        Msg::SessionPickerEmptyFilterQuery { query } =>
            format!("(no sessions match \"{query}\" -- Backspace to clear)").into(),
        Msg::SessionDeleted { name } =>
            format!("\"{name}\" deleted").into(),
        Msg::SessionDeleteConfirm { name } =>
            format!("Press Ctrl+D again to delete \"{name}\"").into(),
        Msg::SessionDeleteFailed { error } =>
            format!("Failed to delete session: {error}").into(),
        Msg::SessionRenameEditing { buffer } =>
            format!("> {buffer}_  [Enter: confirm, Esc: cancel]").into(),

        // ── Dir picker ──
        Msg::DirPickerTitle { n, total } =>
            format!("Change working directory ({n}/{total})").into(),
        Msg::DirPickerHint =>
            "↑↓ move . Tab complete . Enter open . Type to search/path . Esc cancel".into(),
        Msg::DirPickerEmptyPath { query } =>
            format!("No saved project matches \"{query}\" . Enter to open it as a path").into(),
        Msg::DirCurrent => "current".into(),
        Msg::DirNotExists { path } =>
            format!("directory no longer exists: {path}").into(),
        Msg::DirChanged { path } =>
            format!("  Changed to: {path}\n").into(),
        Msg::DirNotADirectory { path } =>
            format!("Not a directory: {path}").into(),
        Msg::CdHomeUnknown => "home directory not known".into(),
        Msg::CdNoPrevious => "No previous directory".into(),

        // ── Language ──
        Msg::LanguageSwitched { label, locale } =>
            format!("  [+] Language switched to {label} ({locale}).\n").into(),

        // ── Idle / onboarding hints ──
        Msg::IdleHintPrefix =>
            "type something, or press ".into(),
        Msg::IdleHintSlash => "/".into(),
        Msg::IdleHintSuffix =>
            " to browse commands".into(),
        Msg::IdleHintFull =>
            "type something, or press / to browse commands".into(),
        Msg::IdleHintProvider => "/provider".into(),
        Msg::IdleHintProviderSuffix =>
            "to add a custom model".into(),
        Msg::IdleHintProviderFull =>
            "/provider  to add a custom model".into(),
        Msg::IdleHintWebui => "/webui".into(),
        Msg::IdleHintWebuiSuffix =>
            "open a synced session in the browser".into(),
        Msg::IdleHintWebuiFull =>
            "/webui  open a synced session in the browser".into(),

        // ── Welcome screen tips ──
        Msg::WelcomeTipsHeading => "Tips for getting started".into(),
        Msg::WelcomeTipLogin => "claim a free token quota".into(),
        Msg::WelcomeTipProvider => "add a custom model".into(),
        Msg::WelcomeTipModel => "set the default model".into(),
        Msg::WelcomeTipResume => "resume your last session".into(),
        Msg::WelcomeTipSetup => "one-shot recommended setup".into(),
        Msg::WelcomeTipSkills => "browse available skills".into(),
        Msg::WelcomeTipPlugin => "install skill/command plugins".into(),
        Msg::WelcomeTipWebui => "open a synced session in the browser".into(),
        Msg::WelcomeTipMcp => "connect MCP tools".into(),
        Msg::WelcomeTipPlan => "read-only planning mode".into(),
        Msg::WelcomeTipSession => "manage & switch sessions".into(),
        Msg::WelcomeTipLoop => "run a prompt on a recurring loop".into(),
        Msg::WelcomeTipGoal => "set a goal for the session".into(),
        Msg::WelcomeTipInit => "scan the codebase into AGENTS.md".into(),
        Msg::WelcomeTipLanguage => "switch the UI language".into(),
        Msg::WelcomeTipUsage => "view token usage & quota".into(),

        // ── Slash commands ──
        Msg::CmdSwitchedPlanMode =>
            "  Switched to Plan mode (read-only exploration).\n".into(),
        Msg::CmdSwitchedBuildMode =>
            "  Switched to Build mode (full execution).\n".into(),
        Msg::CmdNewSession =>
            "  New session started.\n".into(),
        Msg::CmdSessionTransitionPending =>
            "  Runtime is reconfiguring; your input is preserved until it is ready.\n".into(),
        Msg::CmdSessionTransitionFailed { error } =>
            format!("Session switch failed; the previous session is still active: {error}").into(),
        Msg::CmdCapabilityReloadFailed { error } =>
            format!("Runtime capability reload failed; the previous runtime is still active: {error}").into(),
        Msg::CmdNoProviders =>
            "  No providers configured.\n".into(),
        Msg::CmdSessionListLoading =>
            "  Loading sessions...\n".into(),
        Msg::CmdNoSessions =>
            "  No previous sessions found. Start a conversation first.\n".into(),
        Msg::CmdUnknownCommand { name } =>
            format!("Unknown command: /{name}").into(),
        Msg::CmdCustomArgRequired { name } =>
            format!("/{name} requires an argument. Usage: /{name} <your-input>").into(),
        Msg::CmdLoginFailed { error } =>
            format!("login failed: {error}").into(),
        Msg::CmdLogoutDone =>
            "  Signed out. Permissions refreshed.\n".into(),
        Msg::CmdLogoutFailed { error } =>
            format!("logout failed: {error}").into(),
        Msg::CmdWhoamiNotSignedIn =>
            "  Not signed in. Use /login to authenticate.\n".into(),
        Msg::CmdWhoamiNotSignedInNeutral =>
            "  Not signed in. This build has no managed account -- use /provider to \
             add a bring-your-own-key provider.\n"
                .into(),
        Msg::CmdReloadDone { provider, model } =>
            format!("  Config reloaded. Active: {provider} . {model}\n").into(),
        Msg::CmdReloadFailed { error } =>
            format!("reload failed: {error} (kept previous config)").into(),
        Msg::CmdUndoNotSupported =>
            "  Undo is not yet supported.\n".into(),
        Msg::CmdUndoDone { target, last } =>
            format!("  ↩ Rolled back to before turn {target} (removed turns {target}-{last}). Your prompt is back in the input box.\n").into(),
        Msg::CmdUndoDiskWarning =>
            "  [!] Only conversation memory was rolled back -- files on disk were NOT restored. Use /diff to review.\n".into(),
        Msg::CmdUndoNoTurns =>
            "  Nothing to undo (no prompts yet).\n".into(),
        Msg::CmdUndoOutOfRange { requested, available } =>
            format!("  Invalid turn {requested} (conversation has {available} turn(s)).\n").into(),
        Msg::CmdUndoBusy =>
            "  Can't undo while the agent is working -- press Esc to cancel first.\n".into(),
        Msg::CmdRewindBusy =>
            "  Can't rewind while the agent is working -- press Esc to cancel first.\n".into(),
        Msg::CmdRewindUnavailable => "Rewind is unavailable".into(),
        Msg::CmdUndoBadArg =>
            "  Usage: /undo  or  /undo N  (N = turn number).\n".into(),
        Msg::CmdNoChanges =>
            "  (no changes)\n".into(),
        Msg::CmdDiffTruncated =>
            "  ... diff output truncated\n".into(),
        Msg::CmdDiffUntracked => "untracked".into(),
        Msg::CmdDiffBinary => "binary".into(),
        Msg::CmdDiffSummary { files, additions, deletions } =>
            format!("{files} files changed, +{additions} -{deletions}").into(),
        Msg::CmdCheckingUpdate =>
            "  Checking for updates...\n".into(),
        Msg::CmdNoActiveProvider =>
            "No active provider configured. Use /provider to add one.".into(),
        Msg::CmdNoModelConfigured =>
            "no model is configured; run /provider to add a third-party API key first".into(),
        Msg::CmdProviderUnavailable =>
            "Provider is unavailable. Use /login to sign in or /provider to configure one.".into(),
        Msg::CmdProviderUnavailableNeutral =>
            "Provider is unavailable. Use /provider to configure a third-party provider with your own API key.".into(),
        Msg::CmdProviderUnsupportedBuild =>
            "This build cannot access the gateway. Use a distribution build that ships gateway support, or use /provider to configure a third-party provider (bring your own key).".into(),
        Msg::CmdProviderReloading =>
            "Provider/model is switching. Send after the switch completes.".into(),
        Msg::SubmitHeldUntilProviderReady =>
            "  ↳ provider not ready yet -- message queued, will send automatically once ready\n".into(),
        Msg::SubmitHeldUntilLogin =>
            "  ↳ not signed in -- message queued; run /login and it will send automatically\n".into(),

        // ── Approval prompt ──
        Msg::ApprovalPromptAlt { tool, detail } =>
            format!("Allow {}({})? [Y]es=Enter / [N]o / [A]lways", tool, detail).into(),
        Msg::ApprovalWaitingLabel =>
            "> Waiting for approval: ".into(),
        Msg::ApprovalAllow => " Allow  ".into(),
        Msg::ApprovalAlways => " Always  ".into(),

        // ── Cancelled / Error prefix ──
        Msg::Cancelled => "(cancelled)".into(),
        Msg::ErrorPrefix { msg } =>
            format!("[Error: {msg}]").into(),

        // ── Upgrade ──
        Msg::UpgradeSuccess { from, to } =>
            format!("  [+] Upgraded {} -> {}\n", from, to).into(),
        Msg::UpgradeManifestFetched { version } =>
            format!("  Latest version: {}\n", version).into(),
        Msg::UpgradeDownloading { pct, bytes, total } =>
            format!("  Downloading {}% ({} / {} bytes)\n", pct, bytes, total).into(),
        Msg::UpgradeVerifying =>
            "  Verifying SHA256\n".into(),
        Msg::UpgradeReplacing =>
            "  Replacing binary\n".into(),
        Msg::UpgradeDone { version, backup } =>
            format!("\n[+] Upgraded to {} (previous version kept at {})\n  Restarting new version...\n", version, backup).into(),
        Msg::UpgradeAlreadyLatest { current, latest } =>
            format!(
                "  [+] Already on the latest version. already on {} (latest is {}). Pass --force to reinstall.\n",
                current, latest
            ).into(),
        Msg::UpgradeFailed { error } =>
            format!("Upgrade failed: {}", error).into(),
        Msg::UpgradeRolledBack { exe, backup } =>
            format!("\n[+] Rolled back. Current binary: {}; other version saved at {}\n  Restarting rolled-back version...\n", exe, backup).into(),
        Msg::UpgradeReplaceRestored { error } =>
            format!("moving new binary into place failed ({error}). Previous version restored.").into(),
        Msg::UpgradeBackupPreserveFailed { error } =>
            format!("Note: could not preserve previous version as backup ({error}). Rollback unavailable until next upgrade.").into(),
        Msg::UpgradeBackupRemoveFailed { backup, rolling } =>
            format!("Note: could not remove old backup {backup}. Rollback may point to an older version.\n  The .rolling file at {rolling} will be cleaned up on the next upgrade.").into(),
        Msg::UpgradeNoRelease { os, arch } =>
            format!("this platform has no published rustcode release ({os}/{arch})").into(),
        Msg::UpgradeNoTarget { target } =>
            format!("manifest has no entry for target {target} -- this platform may not be in this release").into(),
        Msg::UpgradeManifestHttp { status } =>
            format!("fetching latest.json returned HTTP {status}").into(),
        Msg::UpgradeDownloadHttp { url, status } =>
            format!("downloading {url} returned HTTP {status} -- release may not exist for this platform").into(),
        Msg::UpgradeShortDownload { got, expected } =>
            format!("short download: got {got} bytes, expected {expected}").into(),
        Msg::UpgradeChecksumMismatch { expected, got } =>
            format!("checksum mismatch -- possible corruption or tampering.\n  expected: {expected}\n  got:      {got}").into(),
        Msg::UpgradeExeNoParent { exe } =>
            format!("executable has no parent directory: {exe}").into(),
        Msg::UpgradeDirNotWritable { dir, error } =>
            format!(
                "{dir} is not writable by the current user ({error}).\n\
                 Re-run with elevated privileges:  sudo rustcode upgrade\n\
                 Or reinstall into a user-writable location (e.g. ~/.local/bin)."
            )
            .into(),
        Msg::CliUpgradeAvailable { version } =>
            format!("[*] New version available: {version}").into(),
        Msg::CliUpgradeDownloading { pct, mb, total_mb } =>
            format!("\r   Downloading {pct}% ({mb} / {total_mb} MB)      ").into(),
        Msg::CliUpgradeVerifying => "\n[+] Verifying sha256".into(),
        Msg::CliUpgradeApplying { version } =>
            format!("[+] Upgrading to {version}...").into(),
        Msg::CliUpgradeReexecFailed { error } =>
            format!("Upgrade applied but re-exec failed ({error}). The new version will be used on the next launch.").into(),
        Msg::CliUpgradeCheckFailed =>
            "Note: could not check for updates at startup (will retry in background).".into(),
        Msg::CliUpgradeApplyFailed { error } =>
            format!("Note: pending upgrade could not be applied ({error}). Continuing with current version.").into(),
        Msg::CliUpgradeDevDisabled => "[dev] auto-update disabled".into(),
        Msg::CliFatalError { error } => format!("\nRustCode error: {error}").into(),
        Msg::CliStartingAfterLogin => "\n  Starting RustCode...\n".into(),
        Msg::CliDaemonStarting { port } =>
            format!("Starting RustCode daemon on port {port}...").into(),
        Msg::CliDaemonStopHint => "Press Ctrl+C to stop.".into(),
        Msg::CliDaemonFatal { error } => format!("Fatal: daemon server error: {error}").into(),
        Msg::CliLoginSetupFailed { error } => format!("login setup failed: {error}").into(),
        Msg::CliLoggedOut => "  You have been logged out.".into(),
        Msg::CliStatusLoggedIn { username, id } =>
            format!("\n  Logged in as: {username} ({id})").into(),
        Msg::CliStatusName { name } => format!("  Name: {name}").into(),
        Msg::CliStatusEmail { email } => format!("  Email: {email}").into(),
        Msg::CliStatusAuthFile { path } => format!("  Auth file: {path}\n").into(),
        Msg::CliStatusNotLoggedInManaged => "\n  Not logged in.".into(),
        Msg::CliStatusLoginHint => "  Run 'rustcode login' to authenticate.\n".into(),
        Msg::CliStatusHintNeutral =>
            "\n  [*] No managed login in this build -- it uses bring-your-own-key providers.\n\
Configure a third-party provider in ~/.rustcode/config.toml with your own\n\
base_url and api_key, or run rustcode with --provider <name>.\n"
                .into(),
        Msg::CliManagedLoginNotBuilt =>
            "\n  [*] Managed login is not built into this build.\n\
Skip `/login` and configure a third-party provider directly in\n\
~/.rustcode/config.toml with your own base_url and api_key\n\
(or set RUSTCODE_PLATFORM_SERVER for a managed gateway).\n"
                .into(),
        Msg::CliReauthFailed { error } => format!("re-authentication failed: {error}").into(),
        Msg::CliConfigSaveFailed { path, error } =>
            format!("  [!] Failed to save config to {path}: {error}").into(),
        Msg::CliSyncMarkerWriteFailed { error } =>
            format!("  [!] Failed to write codingplan sync marker: {error}").into(),
        Msg::CliUpgradeLatest { version } => format!("==> Latest: {version}").into(),
        Msg::CliUpgradeDownloadProgress { pct, bytes, total } =>
            format!("\r    downloading {pct}% ({bytes} / {total} bytes)   ").into(),
        Msg::CliUpgradeVerifyingSha => "\n==> Verifying SHA256".into(),
        Msg::CliUpgradeReplacingBinary => "==> Replacing binary".into(),
        Msg::CliUpgradeCmdDone { version, backup } =>
            format!("\n[+] Upgraded to {version} (previous version kept at {backup})").into(),
        Msg::CliUpgradeStartNewHint => "  Run `rustcode` to start the new version.".into(),
        Msg::CliUpgradeCmdFailed { error } => format!("\nupgrade failed: {error}").into(),
        Msg::CliUpgradePanicked { error } => format!("upgrade task panicked: {error}").into(),
        Msg::CliRollbackCmdDone { exe, backup } =>
            format!("\n[+] Rolled back. exe={exe}, backup={backup}").into(),
        Msg::CliRollbackCmdDoneTwo { current, saved } =>
            format!("[+] Rolled back. Previous binary is now at {current}, other version saved at {saved}").into(),
        Msg::CliRollbackStartHint =>
            "  Run `rustcode` to start the rolled-back version.".into(),
        Msg::CliPluginSpecEmpty =>
            "expected <plugin> or <plugin>@<marketplace>, got empty string".into(),
        Msg::CliPluginSpecPartEmpty { spec } =>
            format!("plugin/marketplace name must not be empty in `{spec}`").into(),

        // ── Headless (`-p`/`--print`) stderr lines ──
        Msg::CliHeadlessProviderRetry { reason, backoff_secs, attempt, max_attempts } =>
            format!("API error {reason} -- retrying in {backoff_secs}s ({attempt}/{max_attempts})...").into(),
        Msg::CliHeadlessRateAutoResume { secs } =>
            format!("auto-continuing in {secs}s...").into(),
        Msg::CliHeadlessRateRetry { reason, secs } =>
            format!("HTTP 429{reason} -- retry later (in {secs}s)").into(),
        Msg::CliHeadlessRatePaused { reason } =>
            format!("HTTP 429{reason} -- paused, retry later").into(),
        Msg::CliHeadlessRateWindowResetAt { reset_at } =>
            format!("rate-limit window exhausted -- resets around {reset_at}").into(),
        Msg::CliHeadlessRateWindowSecs { secs } =>
            format!("rate-limit window exhausted -- resets in {secs}s, retry later").into(),
        Msg::CliHeadlessRateWindowPaused =>
            "rate-limit window exhausted -- paused, retry later".into(),
        Msg::CliHeadlessAutoApproved { tool } =>
            format!("auto-approved {tool}").into(),
        Msg::CliHeadlessDenied { tool } =>
            format!("{tool} requires interactive approval").into(),
        Msg::CliSetupCwdError { error } =>
            format!("setup error: cannot read current directory: {error}").into(),
        Msg::CliSetupFailed { error } => format!("setup error: {error}").into(),
        Msg::CliSeedInitialized { path, source } =>
            format!("initialized {path} from {source}").into(),
        Msg::CliSeedInvalid { error } =>
            format!("Warning: --seed-config ignored (not a valid config): {error}").into(),
        Msg::CliSeedIoError { error } =>
            format!("Warning: --seed-config could not be applied: {error}").into(),
        Msg::CliConfigLoadWarnings { path, warnings } =>
            format!("Warning: Some provider sections in {path} could not be loaded:\n{warnings}").into(),
        Msg::CliConfigLoadFailed { path, error } =>
            format!("Warning: Failed to load {path} ({error}); using default configuration.").into(),
        Msg::CliResumeHint { cmd } => format!("To resume this session, run: {cmd}").into(),
        // NOTE: no `\` line-continuations here -- Rust strips the newline AND
        // the following line's leading whitespace, which would defeat the
        // 23-space hint indentation (the original code had exactly that bug).
        Msg::CliWindowsCodePage { input, output } =>
            format!(
                "\n[!]  Console code pages -- input: {input} (expected 65001/UTF-8), output: {output}.\n                       Chinese/Japanese/Korean IME input/output may show garbled text.\n                       -> Use Windows Terminal for native UTF-8 support.\n                       -> Or enable Beta: Use Unicode UTF-8 in Region settings.\n"
            ).into(),
        Msg::CliPromptFileReadFailed { path, error } =>
            format!("error: failed to read --prompt-file {path}: {error}").into(),
        Msg::CliResumeNoMatch { selector } =>
            format!(
                "no session matches id or name {selector} in this project -- run `rustcode resume` to list, or check the working directory (-C)"
            ).into(),
        Msg::CliHeadlessNoProviderNamed { name, path } =>
            format!(
                "Provider not found: '{name}' matches no configured provider and no default provider is set. \
                 Configure a third-party provider (base_url, api_key, model) in {path}, or run `rustcode` with \
                 no arguments for interactive setup."
            ).into(),
        Msg::CliHeadlessNoProvider { path } =>
            format!(
                "No provider configured. Add a third-party provider (base_url, api_key, model) in {path}, \
                 or run `rustcode` with no arguments for interactive setup."
            ).into(),

        // ── `rustcode mcp` ──
        Msg::CliMcpAdded { name, path, program, args } =>
            format!("  Added MCP server {name} -> {path} (stdio: {program} + {args} arg(s))").into(),
        Msg::CliMcpAddedOauth { name, path, url } =>
            format!("  Added OAuth MCP server {name} -> {path} ({url})").into(),
        Msg::CliMcpAddedGithub { name, path } =>
            format!("  Added GitHub OAuth MCP server {name} -> {path}").into(),
        Msg::CliMcpLoginSaved { provider, name, scopes } =>
            format!("  Saved {provider} OAuth token for MCP server {name} with {scopes} scope(s)").into(),
        Msg::CliMcpLogoutRemoved { name } =>
            format!("  Removed saved OAuth token for MCP server {name}").into(),
        Msg::CliMcpLogoutNotFound { name } =>
            format!("  No saved OAuth token found for MCP server {name}").into(),
        Msg::CliMcpServerNotFound { name } =>
            format!("MCP server {name} not found in config").into(),

        // ── `rustcode hooks` ──
        Msg::CliHooksLoadedHeader => "\nLoaded Hooks:".into(),
        Msg::CliHooksNone => "  (No hooks loaded)".into(),
        Msg::CliHooksTableEvent => "Event".into(),
        Msg::CliHooksTableCount => "Count".into(),
        Msg::CliHooksTableTotal => "Total".into(),
        Msg::CliHooksConfigFiles => "\nHook Config Files:".into(),
        Msg::CliHooksPathGlobal { path } => format!("Global:   {path}").into(),
        Msg::CliHooksPathProject { path } => format!("Project:  {path}").into(),
        Msg::CliHooksPathNoHome => "Global:   (no home directory)".into(),
        Msg::CliHooksUntrustedHeader => "Untrusted plugin hooks (not loaded):".into(),
        Msg::CliHooksUntrustedRow { plugin, count, events } =>
            format!("  {plugin} -- {count} hook(s) [{events}] . run: rustcode plugin trust {plugin}").into(),
        Msg::CliHooksTestNotFound { name } =>
            format!("[x] No hook matching '{name}' found.").into(),
        Msg::CliHooksTestNoneLoaded =>
            "\n  (No hooks loaded. Check hooks.json / .hooks.json.)".into(),
        Msg::CliHooksTestAvailable =>
            "\nAvailable hooks (test by event name or a command substring):".into(),
        Msg::CliHooksTesting { event } => format!("\n[*] Testing Hook ({event})").into(),
        Msg::CliHooksFieldCommand { command } => format!("  Command:   {command}").into(),
        Msg::CliHooksFieldTimeout { ms } => format!("  Timeout:   {ms} ms").into(),
        Msg::CliHooksFieldMatcher { matcher } => format!("  Matcher:   {matcher}").into(),
        Msg::CliHooksResultHeader => "[+] Result:".into(),
        Msg::CliHooksDuration { duration } => format!("  Duration:  {duration}").into(),
        Msg::CliHooksFieldStatus { label, detail } =>
            format!("  Status:    {label} ({detail})").into(),
        Msg::CliHooksStatusSuccess => "exit code 0".into(),
        Msg::CliHooksStatusBlock =>
            "exit code 2 -- hook requested a block (CC contract)".into(),
        Msg::CliHooksStatusExitCode { code } => format!("exit code {code}").into(),
        Msg::CliHooksStatusSignal => "terminated by signal".into(),
        Msg::CliHooksDidNotComplete { ms } =>
            format!("  [x] Hook did not complete: it timed out (>{ms} ms) or failed to spawn.").into(),
        Msg::CliHooksPathsHeader => "\nHook Configuration Files:".into(),
        Msg::CliHooksDocsHeader => "\nDocumentation:".into(),
        Msg::CliHooksDocsEntry => "  docs/hooks.md - Hook usage guide".into(),

        // ── `rustcode plugin` / `marketplace` ──
        Msg::CliPluginMpAdded { name, commit, plugins } =>
            format!("  marketplace `{name}` added at {commit} ({plugins} plugins)").into(),
        Msg::CliPluginMpRemoved { name } =>
            format!("  marketplace `{name}` removed").into(),
        Msg::CliPluginMpUpdated { name, commit } =>
            format!("  marketplace `{name}` updated to {commit}").into(),
        Msg::CliPluginMpNone => "  no marketplaces registered".into(),
        Msg::CliPluginMpRow { name, source, commit, plugins } =>
            format!("  {name}  {source}  {commit}  ({plugins} plugins)").into(),
        Msg::CliPluginInstalled { plugin, marketplace } =>
            format!("  installed `{plugin}@{marketplace}`").into(),
        Msg::CliPluginInstallAmbiguous { plugin, list } =>
            format!("plugin `{plugin}` found in multiple marketplaces, please specify:\n{list}").into(),
        Msg::CliPluginNotFound { plugin } =>
            format!("plugin `{plugin}` not found in any marketplace").into(),
        Msg::CliPluginUntrustedNotice { plugin, count, events } =>
            format!(
                "Plugin `{plugin}` ships {count} hook(s) on [{events}]. They will NOT run until trusted:\n  rustcode plugin trust {plugin}"
            ).into(),
        Msg::CliPluginUninstalled { plugin, marketplace } =>
            format!("  uninstalled `{plugin}@{marketplace}`").into(),
        Msg::CliPluginNotInstalled { plugin } =>
            format!("plugin `{plugin}` is not installed").into(),
        Msg::CliPluginUninstallAmbiguous { plugin, list } =>
            format!(
                "plugin `{plugin}` installed from multiple marketplaces, please specify:\n{list}Use /plugin to select the installation scope to remove."
            ).into(),
        Msg::CliPluginNoHooks { name } =>
            format!("plugin `{name}` has no hooks (or is not installed)").into(),
        Msg::CliPluginTrusted { count, name, events } =>
            format!("Trusted {count} hook(s) from `{name}` [{events}].").into(),
        Msg::CliPluginTrustAmbiguous { name, list } =>
            format!(
                "plugin `{name}` has hooks in multiple installations:\n{list}Use /plugin to inspect the installation scopes."
            ).into(),
        Msg::CliPluginUntrusted { name } =>
            format!("Untrusted hooks from `{name}`.").into(),
        Msg::CliPluginNone => "  no installed plugins".into(),
        Msg::CliPluginErrAddMp => "add marketplace".into(),
        Msg::CliPluginErrRemoveMp => "remove marketplace".into(),
        Msg::CliPluginErrUpdateMp => "update marketplace".into(),
        Msg::CliPluginErrInstall => "install".into(),
        Msg::CliPluginErrResolve => "resolve".into(),
        Msg::CliPluginErrUninstall => "uninstall".into(),

        // ── `rustcode schedule` ──
        Msg::CliSchedDailyBad { value } =>
            format!("--daily expects HH:MM format, got {value}").into(),
        Msg::CliSchedWeeklyBad { value } =>
            format!("--weekly expects N@HH:MM format, got {value}").into(),
        Msg::CliSchedWeekdayBad { value } =>
            format!("--weekly weekday must be 1..7, got {value}").into(),
        Msg::CliSchedWeeklyTimeBad { value } =>
            format!("--weekly time must be HH:MM, got {value}").into(),
        Msg::CliSchedEveryBad { value } =>
            format!("--every expects format like '30m', got {value}").into(),
        Msg::CliSchedEveryIntBad { value } =>
            format!("--every minutes value must be a positive integer, got {value}").into(),
        Msg::CliSchedEveryZero => "--every minutes must be > 0".into(),
        Msg::CliSchedFrequencyRequired =>
            "one frequency flag is required: --daily HH:MM | --weekly N@HH:MM | --every Nm | --hourly | --cron EXPR".into(),
        Msg::CliSchedRemoveFailed { id } => format!("failed to remove task {id}").into(),
        Msg::CliSchedTaskNotFound { id } => format!("task {id} not found").into(),
        Msg::CliSchedSaveFailed { id } => format!("failed to save task {id}").into(),
        Msg::CliSchedRegFailed { id, error } =>
            format!("[schedule] warning: OS scheduler registration failed for {id}: {error}\nRun `rustcode schedule sync` to retry.").into(),
        Msg::CliSchedSyncInstalled { id } => format!("  sync: installed {id}").into(),
        Msg::CliSchedSyncInstallFailed { id, error } =>
            format!("  sync: failed to install {id}: {error}").into(),
        Msg::CliSchedSyncUninstalled { id } => format!("  sync: uninstalled {id}").into(),
        Msg::CliSchedSyncUninstallFailed { id, error } =>
            format!("  sync: failed to uninstall {id}: {error}").into(),
        Msg::CliSchedSyncDone { installed, uninstalled, errors } =>
            format!("  sync done: {installed} installed, {uninstalled} uninstalled, {errors} errors").into(),
        Msg::CliSchedAdded { id, title } => format!("  Added task {id} ({title})").into(),
        Msg::CliSchedNone =>
            "  No scheduled tasks. Use `rustcode schedule add` to create one.".into(),
        Msg::CliSchedRemoved { id } => format!("  Removed task {id}").into(),
        Msg::CliSchedEnabled { id } => format!("  Enabled task {id}").into(),
        Msg::CliSchedDisabled { id } => format!("  Disabled task {id}").into(),
        Msg::CliSchedRunSkipped { id } =>
            format!("  schedule run: task {id} is disabled, skipping").into(),
        Msg::CliSchedBadCwd { cwd, id } =>
            format!("[schedule] working directory {cwd} does not exist for task {id}").into(),
        Msg::CliSchedListRow { id, title, next, last, state, reg } =>
            format!("  {id} | {title} | next:{next} | last:{last} | {state} | {reg}").into(),
        Msg::CliSchedStateOn => "on".into(),
        Msg::CliSchedStateOff => "off".into(),
        Msg::CliSchedRegRegistered => "registered".into(),
        Msg::CliSchedRegMissing => "missing".into(),
        Msg::CliSchedRegUnknown => "unknown".into(),

        // ── `rustcode uninstall` ──
        Msg::CliUninstallPurgeConflict =>
            "rustcode uninstall: --purge conflicts with --keep-data".into(),
        Msg::CliUninstallNoTty =>
            "rustcode uninstall: refusing to run interactively without a TTY.\nPass one of: --yes (use defaults), --purge (delete everything), --keep-data (binary only), --dry-run.".into(),
        Msg::CliUninstallBinaryRequired =>
            "rustcode uninstall: cannot uninstall without removing binary; aborted.".into(),
        Msg::CliUninstallProcsAborted =>
            "aborted: running processes were not terminated.".into(),
        Msg::CliUninstallProcsFound { count } =>
            format!("\nFound {count} running rustcode process(es):").into(),
        Msg::CliUninstallKillPrompt => "Kill them and continue? [y/N]: ".into(),
        Msg::CliUninstallKillFailed { pid, error } =>
            format!("could not kill pid {pid}: {error}").into(),
        Msg::CliUninstallKillWarn { pid, error } =>
            format!("warn: could not kill pid {pid}: {error} (continuing -- Unix unlink doesn't need it)").into(),
        Msg::CliUninstallDryRun => "DRY RUN -- no changes will be made.\n".into(),
        Msg::CliUninstallGroup1Plan => "[Group 1] Binary + PATH edit".into(),
        Msg::CliUninstallGroup2Plan => "[Group 2] Credentials and global config".into(),
        Msg::CliUninstallGroup3Plan => "[Group 3] Local state and extensions".into(),
        Msg::CliUninstallGroup1Prompt => "[Group 1] Remove binary and PATH edit?".into(),
        Msg::CliUninstallGroup2Prompt =>
            "[Group 2] Remove credentials and global config?".into(),
        Msg::CliUninstallGroup3Prompt => "[Group 3] Remove local state and extensions?".into(),
        Msg::CliUninstallTagWillRemove => "WILL REMOVE".into(),
        Msg::CliUninstallTagKeep => "KEEP".into(),
        Msg::CliUninstallIntro => "This will uninstall RustCode from your system.\n".into(),
        Msg::CliUninstallGroup1Declined =>
            "Group 1 declined; aborting (cannot keep binary while removing data).".into(),
        Msg::CliUninstallSummaryHeader => "\nSummary:".into(),
        Msg::CliUninstallContinuePrompt => "\nContinue? [y/N]: ".into(),
        Msg::CliUninstallProceedPrompt { suffix } => format!("Proceed? {suffix}: ").into(),
        Msg::CliUninstallActionRemove => "Remove".into(),
        Msg::CliUninstallActionKeep => "Keep".into(),
        Msg::CliUninstallLabelBinary => "binary + PATH".into(),
        Msg::CliUninstallLabelCredentials => "credentials".into(),
        Msg::CliUninstallLabelState => "local state".into(),
        Msg::CliUninstallSummaryRow { action, count, label } =>
            format!("  {action}: {count} items ({label})").into(),
        Msg::CliUninstallResultRemoved => "Removed:".into(),
        Msg::CliUninstallResultKept => "Kept (use --purge to remove later):".into(),
        Msg::CliUninstallResultFailed => "Failed:".into(),
        Msg::CliUninstallResultBackups => "Backups:".into(),
        Msg::CliWebuiNotBuilt =>
            "webui assets are not embedded in this binary.\nBuild the frontend first, then rebuild:\n\n   cd webui && npm install && npm run build\n   cargo build -p rustcode\n".into(),

        // ── /config ──
        Msg::ConfigProviderLabel { provider, path } =>
            format!("  Provider: {}\n  Config: {}\n\n", provider, path).into(),

        // ── /cost ──
        Msg::CostTokenReport { prompt, completion, cached, cache_rate, total } =>
            format!(
                "  Prompt tokens:     {}\n  Completion tokens: {}\n  Cached tokens:     {} ({}% hit rate)\n  Total tokens:      {}\n",
                prompt, completion, cached, cache_rate, total
            ).into(),
        Msg::CostUnattributed { tokens } =>
            format!("Unattributed legacy usage\n  Total tokens:      {}", tokens).into(),

        // ── /think ──
        Msg::ThinkStatus { enabled, budget, provider } =>
            format!(
                "  Extended thinking: {}\n  Budget: {} tokens\n  Provider: {}\n\n  Usage: /think on | off | budget <N>\n",
                if enabled { "enabled" } else { "disabled" },
                budget, provider
            ).into(),
        Msg::ThinkEnabled { budget } =>
            format!("  Extended thinking enabled (budget: {} tokens).\n", budget).into(),
        Msg::ThinkDisabled =>
            "  Extended thinking disabled.\n".into(),
        Msg::ThinkBudgetSet { n } =>
            format!("  Thinking budget set to {} tokens.\n", n).into(),
        Msg::ThinkBudgetTooSmall { n } =>
            format!("Budget must be >= 1024 (got {})", n).into(),
        Msg::ThinkBudgetUsage =>
            "Usage: /think budget <number>".into(),
        Msg::ThinkUsage =>
            "  Usage: /think [on | off | budget <N>]\n".into(),

        // ── /remember, /forget ──
        Msg::RememberUsage =>
            "Usage: /remember <fact to remember>  (--global for global scope)".into(),
        Msg::ForgetUsage =>
            "Usage: /forget <keyword>".into(),
        Msg::MemoryScopeGlobal => "global".into(),
        Msg::MemoryScopeProject => "project".into(),
        Msg::Remembered { scope, content } =>
            format!("Remembered ({scope}): {content}").into(),
        Msg::AlreadyRemembered { scope, content } =>
            format!("Already remembered ({scope}): {content}").into(),
        Msg::RememberFailed { error } =>
            format!("Failed to remember: {error}").into(),
        Msg::ForgetNoMatch { keyword } =>
            format!("No memory entries matched '{keyword}'.").into(),
        Msg::ForgotOne => "Forgot 1 entry.".into(),
        Msg::ForgotMany { count } => format!("Forgot {count} entries.").into(),
        Msg::MemoryEmpty => "(memory is empty)".into(),

        // ── /team ──
        Msg::TeamNoRuns => "No Team runs.".into(),
        Msg::TeamSummary { runs, completed, running, failed, stopped } => format!(
            "Team: {runs} run(s) · {completed} completed · {running} running · {failed} failed · {stopped} stopped"
        ).into(),
        Msg::TeamNoticeDispatched { run_id } =>
            format!("  ○ Team dispatched · {run_id}\n").into(),
        Msg::TeamNoticeStopped { run_id } =>
            format!("  ○ Team stopped · {run_id}\n").into(),
        Msg::TeamNoticeResultsHeader { run_id } =>
            format!("  Team results · {run_id}").into(),
        Msg::TeamMemberFallbackId => "agent".into(),
        Msg::TeamStatusUnknown => "unknown".into(),
        Msg::TeamResultNone => "no report".into(),
        Msg::TeamSuffixDispatched { run_id } =>
            format!("dispatched · {run_id}").into(),
        Msg::TeamSuffixStopped { run_id } =>
            format!("stopped · {run_id}").into(),
        Msg::TeamSuffixUpdated => "updated".into(),

        // ── generic on/off ──
        Msg::WordOn => "on".into(),
        Msg::WordOff => "off".into(),

        // ── /schedule list ──
        Msg::ScheduleListEmpty =>
            "  No scheduled tasks. Use `rustcode schedule add` to create one.\n".into(),
        Msg::ScheduleListHeader => "  Scheduled tasks:\n\n".into(),
        Msg::ScheduleRow { id, title, next, last, state } => format!(
            "  {id} | {title} | next:{next} | last:{last} | {state}\n"
        ).into(),

        // ── /background ──
        Msg::BackgroundUsage =>
            "  Usage: /background <task description>\n".into(),

        // ── /init ──
        Msg::InitKickoff =>
            "  Analyzing the project and generating AGENTS.md...\n".into(),

        // ── /cd ──
        Msg::CdWorkingDir { cwd } =>
            format!("  Working directory: {}\n  No recent projects. Use `/cd <path>` to switch.\n", cwd).into(),

        // ── /diff ──
        Msg::DiffFailed { error } =>
            format!("git diff failed: {}", error).into(),

        // ── /upgrade ──
        Msg::UpgradePackageManaged =>
            "This build is managed by HarmonyBrew. Run `brew upgrade rustcode` to upgrade.".into(),
        Msg::UpgradeUnknownArg { arg } =>
            format!("unknown /upgrade argument: {}\n  usage: /upgrade [rollback|--force]", arg).into(),
        Msg::UpgradeNoEndpoint =>
            "  Self-update is not available in this build: no update manifest endpoint is configured.\n  This open build installs via your package manager or a release tarball.\n".into(),

        // ── /skills ──
        Msg::SkillsNone =>
            "  No user-invocable skills loaded.\n".into(),
        Msg::SkillsAvailable =>
            "  Available skills:\n".into(),
        Msg::CmdSkillsEmptyHint =>
            "No user-invocable skills installed yet.\n    \u{2022} Drop SKILL.md into ~/.rustcode/skills/<name>/ \n      (Windows: %USERPROFILE%\\.rustcode\\skills\\<name>\\)\n    \u{2022} Or install a plugin that ships skills via /plugin install <git-url>\n\n".into(),
        Msg::SkillUnknown { name } =>
            format!("Unknown skill: {} (try /skills to list)", name).into(),
        Msg::SkillsLoaded { names } =>
            format!("  Loaded skills: {}\n", names).into(),

        // ── /mcp ──
        Msg::McpReloading { count } =>
            format!("  Reloading MCP servers... ({} configured)\n", count).into(),
        Msg::McpConnecting =>
            "  Connecting:\n".into(),
        Msg::McpConnectingServer { name } =>
            format!("    - {}  connecting...\n", name).into(),
        Msg::McpNoServersConfigured =>
            "  No MCP servers configured.\n".into(),
        Msg::McpClearedReconnecting =>
            "  MCP reload requested. Old MCP tools are withdrawn before reconnecting in the background.\n".into(),
        Msg::McpClearedNoServers =>
            "  MCP reload requested. Old MCP tools are withdrawn; no servers are configured.\n".into(),
        Msg::McpToolsUsage =>
            "  Usage: /mcp tools <server>\n  Example: /mcp tools filesystem\n".into(),
        Msg::McpServersHeader =>
            "  MCP Servers:\n".into(),
        Msg::McpBlockedTrustHint { count } =>
            format!(
                "  {count} server(s) blocked because this project is untrusted.\n  Run /mcp trust to load this project's MCP servers.\n"
            ).into(),
        Msg::McpReloadFailed { error } =>
            format!("mcp reload failed: failed to load .mcp.json / $RUSTCODE_HOME/mcp.json: {:#}", error).into(),
        // /mcp login / logout
        Msg::McpOAuthLoginUsage =>
            "  Usage: /mcp login <server>\n  Example: /mcp login github\n".into(),
        Msg::McpOAuthLogoutUsage =>
            "  Usage: /mcp logout <server>\n  Example: /mcp logout github\n".into(),
        Msg::McpOAuthLoadConfigFailed { error } =>
            format!("  MCP OAuth login failed to load config: {error}\n").into(),
        Msg::McpOAuthServerNotFound { server } =>
            format!("  MCP OAuth login failed: server '{server}' not found in config.\n").into(),
        Msg::McpOAuthStarting { server } =>
            format!("  Starting MCP OAuth for '{server}' in your browser...\n").into(),
        Msg::McpOAuthSaved { provider, server } =>
            format!("  Saved {provider} OAuth token for MCP server '{server}'. Reloading MCP capabilities.\n").into(),
        Msg::McpOAuthFailed { error } =>
            format!("  MCP OAuth failed: {error}\n").into(),
        Msg::McpOAuthTokenRemoved { server } =>
            format!("  Removed saved OAuth token for MCP server '{server}'.\n").into(),
        Msg::McpOAuthNoToken { server } =>
            format!("  No saved OAuth token found for MCP server '{server}'.\n").into(),
        Msg::McpOAuthLogoutFailed { error } =>
            format!("  MCP OAuth logout failed: {error}\n").into(),
        Msg::McpProjectTrusted =>
            "  Project trusted -- reloading MCP servers.\n".into(),
        Msg::McpProjectUntrusted =>
            "  Project trust revoked.\n".into(),
        Msg::McpProjectNotTrusted =>
            "  This project was not trusted.\n".into(),
        Msg::LspServerStarted { name, ext } =>
            format!("[+] LSP server '{name}' started for .{ext}").into(),
        Msg::LspServerFailed { name, ext, error } =>
            format!("[x] LSP server '{name}' for .{ext} failed: {error}").into(),

        // ── /worktree ──
        Msg::WorktreeUsage =>
            "  Usage:\n    /worktree create <branch> [base]  Create worktree and switch\n    /worktree list                     List all worktrees\n    /worktree done                     Switch back to original directory\n    /worktree cleanup <branch>         Clean up worktree\n".into(),
        Msg::WorktreeCreateUsage =>
            "  Usage: /worktree create <branch> [base]\n  Example: /worktree create fix-bug main\n".into(),
        Msg::WorktreeCreated { branch, base, path } =>
            format!("  [+] Worktree created\n    Branch: {} (based on {})\n    Path: {}\n    Working directory switched\n", branch, base, path).into(),
        Msg::WorktreeCreateFailed { error } =>
            format!("worktree create failed: {}", error).into(),
        Msg::WorktreeNoActive =>
            "  No active worktrees.\n".into(),
        Msg::WorktreeListFailed { error } =>
            format!("worktree list failed: {}", error).into(),
        Msg::WorktreeActiveHeader =>
            "  Active worktrees:\n".into(),
        Msg::WorktreeHasChanges => "(has changes)".into(),
        Msg::WorktreeClean => "(clean)".into(),
        Msg::WorktreeCurrent => " ← current".into(),
        Msg::WorktreeDoneBack { path } =>
            format!("  [+] Switched back to: {}\n", path).into(),
        Msg::WorktreeDoneMergeHint { branch } =>
            format!("  Hint: use 'git merge {}' or create a PR to merge into main branch\n", branch).into(),
        Msg::WorktreeNoSession =>
            "  No active worktree session. Use /worktree create first.\n".into(),
        Msg::WorktreeCleanupUsage =>
            "  Usage: /worktree cleanup <branch> [--force]\n".into(),
        Msg::WorktreeCleaned { branch } =>
            format!("  [+] Worktree '{}' cleaned up\n", branch).into(),
        Msg::WorktreeCleanedSwitched { path } =>
            format!("  Switched back to: {}\n", path).into(),
        Msg::WorktreeCleanupUncommitted { branch } =>
            format!("  [!] Worktree '{}' has uncommitted changes.\n  Use /worktree cleanup {} --force to force cleanup\n", branch, branch).into(),
        Msg::WorktreeCleanupFailed { error } =>
            format!("worktree cleanup failed: {}", error).into(),

        // ── /help commands (custom) ──
        Msg::HelpCustomCommandsHeader =>
            "  Custom commands:\n".into(),
        Msg::HelpCustomNone =>
            "    (none)\n\n".into(),
        Msg::HelpCustomCreateHint =>
            "  Create: ~/.rustcode/commands/<name>.md or .rustcode/commands/<name>.md\n".into(),
        Msg::HelpSourceGlobal => "global".into(),
        Msg::HelpSourceProject => "project".into(),

        // ── /setup ──
        Msg::SetupHeader { installed, skipped, failed, duration_ms } =>
            format!("\n[+] Setup complete -- {} installed, {} skipped, {} failed  . {}ms\n\n", installed, skipped, failed, duration_ms).into(),
        Msg::SetupInstalledLabel =>
            "Installed:\n".into(),
        Msg::SetupSkippedLabel =>
            "\nSkipped:\n".into(),
        Msg::SetupFailedLabel =>
            "\nFailed:\n".into(),
        Msg::SetupInstalledRow { kind, slug, path } =>
            format!("  [+] {}:{} -> {}\n", kind, slug, path).into(),
        Msg::SetupSkippedRow { kind, slug, reason } =>
            format!("  - {}:{} ({:?})\n", kind, slug, reason).into(),
        Msg::SetupFailedRow { kind, slug, error } =>
            format!("  [x] {}:{} -- {}\n", kind, slug, error).into(),
        Msg::CmdSetupTip =>
            // No leading emoji: U+1F4A1 has terminal/font-dependent display
            // width (1 vs 2 cells), which desynced this line's cell layout
            // on some terminals (garbled "TTip:RRun..." over SSH). ASCII-only
            // prefix keeps the width unambiguous.
            "Tip: Run \x1b[1;96m/setup\x1b[0m to auto-configure hooks, skills, and MCP for this project.".into(),
        Msg::CmdSetupRunning =>
            "Running rustcode setup...".into(),
        Msg::CmdSetupSkillsReloaded { count } =>
            format!("  [*] Skills reloaded -- {} available", count).into(),
        Msg::CmdSetupError { error } =>
            format!("setup error: {error}").into(),
        Msg::CmdSetupRunningSkill =>
            "  [*] Running setup skill -- analyzing project and generating recommendations...".into(),
        Msg::CmdSetupSkillMissing =>
            "setup skill not found -- try running /setup again to reinstall".into(),

        // ── /plugin ──
        Msg::PluginUsage =>
            "usage: /plugin [marketplace add|remove|update|list | install <p>@<m> | uninstall <p>@<m> | reload | list]".into(),
        Msg::PluginMarketplaceUsage =>
            "usage: /plugin marketplace [add|remove|update|list] <args>".into(),
        Msg::PluginInstallUsage =>
            "usage: /plugin install <plugin> or <plugin>@<marketplace>".into(),
        Msg::PluginInstallNotFound { plugin } =>
            format!("plugin `{plugin}` not found in any marketplace. Use /plugin marketplace list to see registered marketplaces.").into(),
        Msg::PluginInstallAmbiguous { plugin } =>
            format!("plugin `{plugin}` exists in multiple marketplaces, please specify one:").into(),
        Msg::PluginUninstallUsage =>
            "usage: /plugin uninstall <plugin> or <plugin>@<marketplace>".into(),
        Msg::PluginUninstallNotFound { plugin } =>
            format!("plugin `{plugin}` is not installed. Use /plugin list to see installed plugins.").into(),
        Msg::PluginUninstallAmbiguous { plugin } =>
            format!("plugin `{plugin}` is installed from multiple marketplaces, please specify:\n").into(),
        Msg::PluginNoMarketplaces =>
            "no marketplaces registered".into(),
        Msg::PluginMarketplacesHeader =>
            "registered marketplaces:".into(),
        Msg::PluginNoInstalled =>
            "no installed plugins".into(),
        Msg::PluginInstalledHeader =>
            "installed plugins:".into(),
        Msg::PluginMarketplaceCloning { url } =>
            format!("cloning marketplace from {url}...").into(),
        Msg::PluginMarketplaceRemoved { name } =>
            format!("marketplace `{name}` removed").into(),
        Msg::PluginMarketplaceRemoveFailed { error } =>
            format!("remove marketplace: {error}").into(),
        Msg::PluginMarketplaceUpdating { name } =>
            format!("updating marketplace `{name}`...").into(),
        Msg::PluginMarketplaceListFailed { error } =>
            format!("list marketplaces: {error}").into(),
        Msg::PluginAutoUpdateSkipped { detail } =>
            format!("Marketplace sync skipped (chat unaffected): {detail}").into(),
        Msg::OfflineModeActive =>
            "Offline mode: web tools and auto-update are disabled.".into(),
        Msg::PluginHooksUntrusted { count, names } => format!(
            "{count} plugin(s) ship untrusted hooks ({names}) -- they won't run. Trust: rustcode plugin trust <name>"
        ).into(),
        Msg::PluginInstalling { plugin, marketplace } =>
            format!("installing `{plugin}@{marketplace}`...").into(),
        Msg::PluginInstallingByName { plugin } =>
            format!("installing `{plugin}`...").into(),
        Msg::PluginAlreadyInstalled { id } =>
            format!("  plugin `{id}` is already installed.\n  PS: To reinstall, first run `/plugin uninstall {id}` then `/plugin install {id}`\n").into(),
        Msg::PluginMgrBrowse => "Browse & install".into(),
        Msg::PluginMgrAdd => "Add marketplace...".into(),
        Msg::PluginMgrRemove => "Remove marketplace...".into(),
        Msg::PluginMgrInstalled { count } => format!("Installed ({count})").into(),
        Msg::PluginMgrInstalledMark => "[+] installed".into(),
        Msg::PluginMgrInstalledStatus => "installed".into(),
        Msg::PluginMgrInstallableStatus => "can be installed".into(),
        Msg::PluginMgrInstallingStatus => "installing".into(),
        Msg::PluginMgrUpdatingStatus => "updating".into(),
        Msg::PluginMgrHintNav => "↑/↓ select . ⏎ open . esc back".into(),
        Msg::PluginMgrHintToggle => "⏎ install/uninstall . esc back".into(),
        Msg::PluginMgrHintRemove => "⏎ remove . esc back".into(),
        Msg::PluginMgrHintUninstall => "⏎ uninstall . esc back".into(),
        Msg::PluginMgrHintUrl => "Enter to add . Esc to cancel".into(),
Msg::PluginMgrHintPending => "Installing, please wait... . esc back".into(),
Msg::PluginMgrHintUpdating => "Updating, please wait... . esc back".into(),
Msg::PluginMgrInstallingLabel => "Installing...".into(),
        Msg::PluginMgrEmptyMarketplaces => "No marketplaces. Pick “Add marketplace...” . esc back".into(),
        Msg::PluginMgrEmptyPlugins => "No plugins in this marketplace . esc back".into(),
        Msg::PluginMgrEmptyInstalled => "No plugins installed . esc back".into(),
        Msg::PluginMgrCloning => "Cloning marketplace...".into(),
        Msg::PluginMgrInstalling { plugin } => format!("Installing {plugin}...").into(),
        Msg::PluginMgrUpdating { plugin } => format!("Updating {plugin}...").into(),
        Msg::PluginMgrEscToCancel => "Esc to cancel".into(),
        Msg::PluginMgrRemoveMarketplaceTitle => "  * Remove Marketplace".into(),
        Msg::PluginMgrRemoveMarketplacePrompt { name } => format!("  \x1b[33mAre you sure you want to remove marketplace '{name}'?\x1b[39m").into(),
        Msg::PluginMgrRemoveMarketplaceYes => "Yes, remove".into(),
        Msg::PluginMgrRemoveMarketplaceNo => "No, keep".into(),
        Msg::PluginMgrRemoveMarketplaceHint => "↑/↓ select . ⏎ confirm . esc cancel".into(),
        Msg::PluginScopeUser => "Install for you (user scope)".into(),
        Msg::PluginScopeUserDesc => "~/.rustcode/plugins -- all projects".into(),
        Msg::PluginScopeProject => "Install for all collaborators (project scope)".into(),
        Msg::PluginScopeProjectDesc => ".rustcode/plugins -- shared via git".into(),
        Msg::PluginScopeLocal => "Install for you, in this repo only (local scope)".into(),
        Msg::PluginScopeLocalDesc => ".rustcode/plugins/local -- not committed".into(),
        Msg::PluginScopeHint => "↑↓ Select scope . Enter confirm . Esc back".into(),
        Msg::PluginScopeUserShort => "user".into(),
        Msg::PluginScopeProjectShort => "project".into(),
        Msg::PluginScopeLocalShort => "local".into(),
        Msg::PluginActionUninstall => "Uninstall".into(),
        Msg::PluginActionUninstallDesc => "Uninstall all components and settings".into(),
        Msg::PluginActionUpdate => "Update".into(),
        Msg::PluginActionUpdateDesc => "Reinstall / Upgrade to latest version".into(),
        Msg::PluginActionDisable => "Disable".into(),
        Msg::PluginActionDisableDesc => "Temporarily disable this plugin".into(),
        Msg::PluginActionBack => "Back to parent".into(),
        Msg::PluginActionBackDesc => "Return to the installed list".into(),
        Msg::PluginUninstalled { plugin, marketplace } =>
            format!("uninstalled `{plugin}@{marketplace}`").into(),
        Msg::PluginUninstallFailed { error } =>
            format!("uninstall: {error}").into(),
        Msg::PluginListFailed { error } =>
            format!("list plugins: {error}").into(),
        Msg::PluginReloadDone { skills, warnings } =>
            format!("Plugins reloaded: {skills} skill(s), {warnings} warning(s)").into(),
        Msg::PluginGitNotFound =>
            "[!] git is not installed or not on PATH. Plugin marketplace auto-install and auto-update are disabled. Install git (e.g. `xcode-select --install` on macOS, `sudo apt install git` on Ubuntu) and restart {brand}.".into(),
        Msg::PluginMarketplaceAdded { name, commit, count, plugins } =>
            format!(
                "[+] marketplace `{name}` added at {commit} ({count} plugins)\n  \
                 Plugins: {plugins} -- run /plugin install <plugin>@{name} to install before using its commands"
            ).into(),
        Msg::PluginMarketplaceUpdated { name, commit } =>
            format!("[+] marketplace `{name}` updated to {commit}").into(),
        Msg::PluginInstallDone { plugin, marketplace: _, loaded, skipped, show_details_hint } => {
            format!("  `  [+] Installed {plugin} -- {}", plugin_reload_summary(loaded, skipped, show_details_hint)).into()
        }
        Msg::PluginUpdateDone { plugin, marketplace: _, loaded, skipped, show_details_hint } => {
            format!("  `  [+] Updated {plugin} -- {}", plugin_reload_summary(loaded, skipped, show_details_hint)).into()
        }
        Msg::SetupAutoReloaded { skills, warnings } =>
            format!("[+] Setup complete, auto-reloaded: {skills} skill(s), {warnings} warning(s)").into(),

        // ── Plugin manager modal ──
        Msg::PluginTabAll => "All Plugins".into(),
        Msg::PluginTabInstalled { count } => format!("Installed ({count})").into(),
        Msg::PluginTabMarketplaces => "Marketplaces".into(),
        Msg::PluginAutoUninstallFailed { name, error } =>
            format!("Failed to auto-uninstall plugin '{name}': {error}").into(),
        Msg::PluginNoPluginsMatch { query } =>
            format!("No plugins match '{query}'").into(),
        Msg::PluginNoInstalledMatch { query } =>
            format!("No installed plugins match '{query}'").into(),
        Msg::PluginAddMarketplaceRow => "Add Marketplace".into(),
        Msg::PluginAddMarketplacePlus => "+ Add Marketplace".into(),
        Msg::PluginEnterSourceRow => "Enter marketplace source:".into(),
        Msg::PluginExamplesRow => "Examples:".into(),
        Msg::PluginBrowseRow { count } => format!("Browse plugins ({count})").into(),
        Msg::PluginUpdateRow { date } =>
            format!("Update marketplace (last updated {date})").into(),
        Msg::PluginRemoveMarketplaceRow => "Remove marketplace".into(),
        Msg::PluginInfoHeader => "  * Plugin Info".into(),
        Msg::PluginNameLabel => "  Name:        ".into(),
        Msg::PluginMarketplaceLabel => "  Marketplace: ".into(),
        Msg::PluginVersionLabel => "  Version:     ".into(),
        Msg::PluginScopeLabel => "  Scope:       ".into(),
        Msg::PluginDescriptionLabel => "  Description: ".into(),
        Msg::PluginSelectScopeHeader => "  Select Install Scope:".into(),
        Msg::PluginManageHeader => "  Manage Plugin:".into(),
        Msg::PluginStatusInstalling { label } =>
            format!("  Status:      {label}...").into(),
        Msg::PluginAvailableCount { count } =>
            format!("  {count} available plugins").into(),
        Msg::PluginModalInstalledHeader { count } =>
            format!("  \x1b[1mInstalled plugins ({count}):\x1b[22m").into(),
        Msg::PluginNoInstalledFromMarketplace =>
            "No plugins installed from this marketplace.".into(),
        Msg::PluginVersionUnknown => "unknown".into(),
        Msg::PluginCategoryGit => "Git".into(),
        Msg::PluginCategoryLinter => "Linter".into(),
        Msg::PluginCategoryFormatter => "Formatter".into(),
        Msg::PluginCategoryLanguage => "Language".into(),
        Msg::PluginCategorySecurity => "Security".into(),
        Msg::PluginCategoryAi => "AI".into(),
        Msg::PluginCategoryUtility => "Utility".into(),
        Msg::PluginCategoryTool => "Tool".into(),
        Msg::PluginCategoryCompletion => "Completion".into(),

        // ── Command descriptions ──
        Msg::CmdDescWebui => "Launch the browser webui (subcommands: stop, lan, --host <addr>)".into(),
Msg::CmdDescSetup =>
"Scan project, install seeds, and run setup skill [hooks|mcp|skills|all]".into(),
        Msg::CmdDescResume => "Resume a previous session".into(),
        Msg::CmdDescRename => "Rename current session".into(),
        Msg::CmdDescLogin => "Sign in with {oauth} and claim CodingPlan models".into(),
        Msg::CmdDescLoginNeutral =>
            "Managed sign-in (distribution builds only) -- this open build uses bring-your-own-key providers; configure config.toml".into(),
        Msg::CmdDescLogout => "Sign out".into(),
        Msg::CmdDescWhoami => "Show current logged-in user".into(),
        Msg::CmdDescModel =>
            "Set the default provider / model and switch this session".into(),
        Msg::CmdDescProvider =>
            "Manage providers (add / edit / delete / set global default)".into(),
        Msg::CmdDescStatus => "Show session status".into(),
        Msg::CmdDescConfig => "Show config path".into(),
        Msg::CmdDescReload => "Reload $RUSTCODE_HOME/config.toml from disk".into(),
        Msg::CmdDescCd => "Change working directory and start a new session".into(),
Msg::CmdDescInit => "Analyze the project and generate AGENTS.md".into(),
Msg::CmdDescBg => "Background sessions: /bg, /bg list, /bg <N>, /bg drop <N>".into(),
Msg::CmdDescBackground => "Run a one-shot task in an isolated background context (read-only-ish tool subset)".into(),
        Msg::CmdDescDiff => "Show git diff".into(),
        Msg::CmdDescClear => "Clear screen".into(),
        Msg::CmdDescSession => "Start a new session (clears conversation)".into(),
        Msg::CmdDescCost => "Show session token usage".into(),
        Msg::CmdDescUsage => "Show CodingPlan usage (tabs: current / overview / models)".into(),
        Msg::CmdDescUsageNeutral =>
            "Show token usage (tabs: current / overview / models)".into(),
        Msg::CmdDescContext => "Show context budget breakdown".into(),
        Msg::CmdDescCompact => "Compact conversation history".into(),
        Msg::CmdDescRemember => "Save a fact to memory (/remember --global for global)".into(),
        Msg::CmdDescForget => "Remove matching memories".into(),
        Msg::CmdDescMemory => "Show all saved memories".into(),
        Msg::CmdDescMcp => "Show MCP server status (subcommand: reload)".into(),
        Msg::CmdDescUndo => "Undo: roll conversation memory back a turn (/undo or /undo N)".into(),
        Msg::CmdDescRewind => "Rewind: restore the conversation to an earlier checkpoint".into(),
        Msg::CmdDescWorktree => "Git worktree isolation (create/list/done/cleanup)".into(),
        Msg::CmdDescUpgrade => "Upgrade rustcode to latest (subcommand: rollback)".into(),
        Msg::CmdDescPlan => "Switch to Plan mode (read-only exploration)".into(),
        Msg::CmdDescBuild => "Switch to Build mode (full execution)".into(),
        Msg::CmdDescAuto => "Switch to Auto mode (auto-approve all tools)".into(),
        Msg::CmdDescThink => "Extended thinking control (on/off/budget N)".into(),
        Msg::CmdDescEffort => "Model reasoning effort control (low / medium / high / xhigh / max / auto)".into(),
        Msg::CmdDescHelp => "Show this help".into(),
        Msg::CmdDescKeys => "Show keyboard shortcuts".into(),
        Msg::CmdDescLanguage => "Switch display language".into(),
        Msg::CmdDescQuit => "Exit {brand}".into(),
        Msg::CmdDescSkills => "Browse loaded skills".into(),
        Msg::CmdDescPlugin => "Plugin marketplace (subcommands: marketplace, install, uninstall, reload, list)".into(),
        Msg::CmdDescPaste => "Attach an image from the clipboard (Windows fallback for Ctrl+V)".into(),
        Msg::CmdDescCopy => "Copy a code block, or the full reply with /copy msg (/copy, /copy N, /copy all, /copy msg)".into(),
        Msg::CopyOk { lines, chars } => format!("Copied code block to clipboard ({lines} lines, {chars} chars)").into(),
        Msg::CopyOkMsg { lines, chars } => format!("Copied reply to clipboard ({lines} lines, {chars} chars)").into(),
        Msg::CopyNoCodeBlock => "No code block in the last reply to copy".into(),
        Msg::CopyMsgEmpty => "The last reply is empty -- nothing to copy".into(),
        Msg::CopyBadIndex { count } => format!("No such code block -- the last reply has {count} (use /copy N, 1..={count})").into(),
        Msg::CopyFailed => "Clipboard unavailable -- could not copy".into(),
        Msg::CmdDescSave => "Save the current conversation to a markdown file (/save, /save [filename])".into(),
        Msg::SaveOk { path } => format!("Conversation saved to {path}").into(),
        Msg::SaveEmpty => "No conversation to export yet".into(),
        Msg::SaveIoError { error } => format!("Failed to save conversation: {error}").into(),
        Msg::SaveInvalidPath { path } => format!("Invalid path -- directory does not exist: {path}").into(),
        Msg::SaveRefuseOverwrite { path } => format!("Target exists and isn't a markdown file -- refused to overwrite it (avoids clobbering source/config): {path}. Use a .md filename or a new path.").into(),
        Msg::CodeBlockCopied => "[+] Copied code block to clipboard".into(),
        Msg::CmdDescGuide => "Ask rustcode-guide how to use".into(),
        Msg::CmdDescView => "View file content in an overlay modal".into(),
        Msg::CmdDescApp => "Expose this session to the mobile App via relay (QR pairing; /app stop to detach)".into(),
        Msg::CmdDescSync => "Attach to live webui session (/sync off to detach)".into(),
        Msg::CmdDescReview => "Code review the current changes (/review . /review staged . /review <base>)".into(),
        Msg::CmdDescGoal => "Set a completion goal (autonomous loop until met)".into(),
        Msg::CmdDescProxy => "Switch outbound proxy mode".into(),
        Msg::CmdDescTodo => "Show the current todo list; `/todo add <task>` appends one, `/todo clear` wipes it".into(),
        Msg::CmdDescTeam => "Show or control the Team Agent progress panel".into(),
        Msg::CmdDescSchedule => "List scheduled tasks and next run times".into(),
        Msg::CmdDescDesktop =>
            "Open the {brand} desktop app (launch it if installed, else show the download link)".into(),
        // ── /proxy picker ──
        Msg::ProxyTitleFollowSystem => "follow_system".into(),
        Msg::ProxyTitleDefaultProxy => "default_proxy".into(),
        Msg::ProxyTitleNoProxy => "no_proxy".into(),
        Msg::ProxyDescFollowSystem =>
            "Follow current launch environment / system proxy state".into(),
        Msg::ProxyDescDefaultProxy =>
            "Pin the current proxy env and reuse it on later launches".into(),
        Msg::ProxyDescNoProxy =>
            "Disable proxy resolution for outbound HTTP clients".into(),
        Msg::ProxyModeLine { mode } => format!("  Proxy mode: {mode}\n").into(),
        Msg::ProxyDefaultPinned { count } =>
            format!("default_proxy ({count} pinned vars)").into(),
        Msg::ProxyDefaultEmpty => "default_proxy (no pinned env captured)".into(),
        Msg::DesktopOpening { name, path } =>
            format!("Opening {}...\n  {}\n", name, path).into(),
        Msg::DesktopNotInstalled { url } =>
            format!("{{brand}} desktop app not found. Download & install:\n  {}\n", url).into(),
        Msg::DesktopNotInstalledNoUrl =>
            "{brand} desktop app not found, and this build ships no desktop download link.\n  Use the terminal UI or configure your own provider in config.toml.\n".into(),
        Msg::DesktopLaunchFailed { path, err } =>
            format!("Found the app but couldn't launch it: {}\n  {}\n", err, path).into(),
        Msg::TodoNoList => "No task list yet (the model hasn't created todos).".into(),
        Msg::TodoListHeader => "Current tasks:".into(),
        Msg::TodoAddUsage => "Usage: /todo add <task description>".into(),
        Msg::GuideMenuHeader => "[*] {brand} Guide -- type /guide <question>".into(),
        Msg::GuideMenuTopics => "Common topics:".into(),
        Msg::GuideMenuGettingStarted => "Getting started          First install, login, config".into(),
        Msg::GuideMenuSwitchModel => "Set default model        /model /provider usage".into(),
        Msg::GuideMenuMcp => "Using MCP                MCP server config & management".into(),
        Msg::GuideMenuSkills => "Skills and plugins       /skills /plugin usage".into(),
        Msg::GuideMenuMemory => "Memory feature           /remember /forget /memory".into(),
        Msg::GuideMenuBackground => "Background tasks         /bg background execution".into(),
        Msg::GuideMenuContext => "Context management       /compact /context /cost".into(),
        Msg::GuideMenuKeybindings => "Keyboard shortcuts       Keyboard shortcut reference".into(),
        Msg::GuideMenuConfig => "Configuration            config.toml reference".into(),
        Msg::GuideMenuTip => "
  Tip: type /guide <your question> for a specific answer.
  Example: /guide How to set the default model
".into(),
        Msg::GuideMenuDocUrl => "  Full docs: https://docs.rustcode.dev/en/".into(),
        Msg::CmdGuideInstalling => "Installing ask skill, please wait...".into(),
        Msg::CmdGuideAutoInstall => "ask skill not installed -- auto-installing rustcode@rustcode-skills...".into(),
        Msg::CmdGuideAutoInvoke { topic } =>
            format!("ask skill installed, now answering: {}", topic).into(),
        Msg::CmdGuideSkillNotFound =>
            "Installation complete but ask skill not found -- run /plugin reload and try again".into(),
        Msg::CmdGuideInstallFailed { error } =>
            format!("ask skill install failed: {}. Run /plugin install rustcode@rustcode-skills manually", error).into(),
        Msg::CmdPasteNoImage => "No image in clipboard.".into(),
        Msg::CmdPasteNoImageOhos => {
            "HarmonyOS can't read images from the system clipboard yet. Save the image to a file, then paste/type its absolute path (e.g. /storage/.../pic.png) to attach it.".into()
        }

        // ── reasoning effort ──
        Msg::ReasoningEffortNoEffect => "The current model is not configured to support reasoning_effort; enable it in /provider".into(),
        Msg::EffortCleared => "  reasoning_effort cleared (API default)\n".into(),

        // ── config save failed ──
        Msg::ConfigSaveFailed { error } =>
            format!("config save failed: {}", error).into(),
        Msg::CfgLegacyProviderNotFound { name } =>
            format!("legacy provider `{name}` not found").into(),
        Msg::CfgLegacyProviderExists { name } =>
            format!("cannot upgrade `{name}`: a new-schema account or model already uses that id").into(),
        Msg::CfgResolveNoModel =>
            "no model selected (set `default_model` or `default_provider`)".into(),
        Msg::CfgResolveModelNotFound { id } => format!("model `{id}` not found").into(),
        Msg::CfgResolveModelUnknownAccount { id, account } =>
            format!("model `{id}` references unknown account `{account}`").into(),
        Msg::CfgDiagAccountMissingProvider { id } =>
            format!("provider account `{id}` is missing `provider`").into(),
        Msg::CfgDiagAccountNoEndpoint { id, provider } =>
            format!(
                "provider account `{id}` uses `{provider}`, which has no default endpoint; set `base_url`"
            )
            .into(),
        Msg::CfgDiagModelMissingModel { id } =>
            format!("model `{id}` is missing `model`").into(),
        Msg::CfgDiagModelMissingAccount { id } =>
            format!("model `{id}` is missing `account`").into(),
        Msg::CfgDiagModelUnknownAccount { id, account } =>
            format!("model `{id}` references unknown account `{account}`").into(),
        Msg::CfgDiagModelContextWindow { id } =>
            format!("model `{id}` has context_window = 0").into(),
        Msg::CfgDiagModelMaxTokens { id } => format!("model `{id}` has max_tokens = 0").into(),
        Msg::CfgDiagDefaultModelMismatch { sel } =>
            format!("default_model `{sel}` does not match any model profile").into(),
        Msg::CfgDiagAccountCollision { id } =>
            format!(
                "provider account `{id}` collides with a legacy provider of the same name; the new-schema account wins"
            )
            .into(),
        Msg::CfgDiagModelCollision { id } =>
            format!(
                "model `{id}` collides with a legacy provider of the same name; the new-schema model wins"
            )
            .into(),

        // ── OnboardingWizard ──
        Msg::OnboardingStepHeaderWelcome => "Step 1/3 . Welcome".into(),
        Msg::OnboardingStepHeaderLanguage => "Step 2/3 . Language".into(),
        Msg::OnboardingStepHeaderSetup => "Step 3/3 . Setup".into(),
        Msg::OnboardingPanelTitle => "{brand}".into(),
        Msg::OnboardingStepIndicator { current, total } =>
            format!("Step {current}/{total}").into(),
        Msg::OnboardingIntroVersionLine { v } =>
            format!("Version {v}  .  AI coding agent in your terminal").into(),
        Msg::OnboardingIntroBullet1 =>
            "* Multi-step agent loop . built-in code-graph tools".into(),
        Msg::OnboardingIntroBullet2 =>
            "* Connects to any OpenAI-compatible API".into(),
        Msg::OnboardingIntroBullet3 =>
            "* Free tokens via CodingPlan".into(),
        Msg::OnboardingIntroBullet3Neutral =>
            "* Bring your own API key -- no account or signup".into(),
        Msg::OnboardingIntroPressEnter => "Press Enter to continue.".into(),
        Msg::OnboardingIntroCtrlC => "Ctrl+C exits at any point.".into(),
        Msg::OnboardingIntroCompactTagline =>
            "AI coding agent that lives in your terminal.".into(),
        Msg::OnboardingLanguageTitleBilingual =>
            "Choose your language / 选择语言".into(),
        Msg::OnboardingLanguagePrompt =>
            "Pick the UI language. You can change it any time with `/language`.".into(),
        Msg::OnboardingLanguageOptionAuto =>
            "Auto-detect (LC_ALL / LANG)".into(),
        Msg::OnboardingLanguageOptionEn => "English".into(),
        Msg::OnboardingLanguageOptionZhCn => "简体中文 (Simplified Chinese)".into(),
        Msg::OnboardingSetupTitle => "How would you like to set up?".into(),
        Msg::OnboardingNavHint =>
            "1-3 select . Enter confirm . ← back . Esc skip".into(),
        Msg::OnboardingSetupNavHint =>
            "number to select . Enter confirm . ← back . Esc skip".into(),
        Msg::OnboardingConfirmClear =>
            "/welcome will clear the screen. Continue? [y/N]".into(),
        Msg::CmdWelcomeDescription => "Re-run the onboarding wizard".into(),
        Msg::VisionPreprocessSuccess { char_count } =>
            format!("[+] VL recognised image, returned {char_count} chars").into(),
        Msg::VisionPreprocessFailed { reason } =>
            format!("VL preprocessing failed: {reason} . continuing text-only this turn; images restored, retry to re-run recognition").into(),
        Msg::TurnSummary { done, turn_count, tool_call_count, duration, total_tokens, cached_pct } =>
            format!(
                "[+] {done} . {turn_count} rounds . {tool_call_count} tools . {duration} . {} tokens{}",
                super::fmt_tokens(total_tokens),
                cached_pct.map(|p| format!(" . {p}% cached")).unwrap_or_default(),
            ).into(),
        Msg::TurnStatsFragment { tool_call_count, duration, total_tokens, cached_pct } =>
            format!(
                "{tool_call_count} tools . {duration} . {} tokens{}",
                super::fmt_tokens(total_tokens),
                cached_pct.map(|p| format!(" . {p}% cached")).unwrap_or_default(),
            ).into(),
        Msg::GoalRound { round, stats } =>
            format!("\u{21bb} goal round {round} . {stats}").into(),
        Msg::TurnSummaryError { turn_count, tool_call_count, duration, total_tokens, reason } => {
            let cause = reason.map(|r| format!(": {r}")).unwrap_or_default();
            format!("[x] Stopped{cause} . {turn_count} rounds . {tool_call_count} tools . {duration} . {} tokens", super::fmt_tokens(total_tokens)).into()
        }
        Msg::TurnSummaryPolicyDenied { turn_count, tool_call_count, duration, total_tokens, reason } => {
            let cause = reason.map(|r| format!(": {r}")).unwrap_or_default();
            format!("[x] Turn stopped by security policy{cause} . {turn_count} rounds . {tool_call_count} tools . {duration} . {} tokens", super::fmt_tokens(total_tokens)).into()
        }
        Msg::SpinnerEffortSuffix { effort } => format!(" \u{b7} thinking with {effort} effort").into(),
        Msg::SpinnerQueuedSuffix { count } => format!(" \u{b7} {count} queued").into(),
        Msg::SpinnerElapsedTokens { elapsed, tokens } =>
            format!(" ({elapsed} \u{b7} \u{2191} {tokens} tokens)").into(),
        Msg::SpinnerElapsedOnly { elapsed } => format!(" ({elapsed})").into(),
        Msg::SpinnerSubAgents { done, total } => format!("SubAgents {done}/{total}").into(),
        Msg::SpinnerWaitingApproval => "Waiting approval".into(),
        Msg::SpinnerRunningLabel => "Running".into(),

        // ── Live hub / phone remote synchronization errors ──
        Msg::LiveSyncEventFailed { error } =>
            format!("Live event synchronization failed: {error}").into(),
        Msg::LiveSyncProviderFailed { error } =>
            format!("Live provider synchronization failed: {error}").into(),
        Msg::LiveSyncGoalFailed { error } =>
            format!("Live Goal synchronization failed: {error}").into(),
        Msg::LiveSyncRemoteOutputFailed { error } =>
            format!("Remote command output synchronization failed: {error}").into(),
        Msg::LiveSyncRemoteRejectFailed { error } =>
            format!("Remote command rejection synchronization failed: {error}").into(),
        Msg::LiveRemoteCommandEcho { display } =>
            format!("(executed from phone: {display})").into(),
        Msg::LiveRemoteCommandRejected =>
            "  This command must run on the desktop (phone supports only /status /cost /whoami /diff)".into(),
        Msg::LiveProjectionNoSessionIdentity =>
            "session switch completed without a session identity".into(),
        Msg::LiveProjectionUnexpectedIdentity =>
            "capability reload returned an unexpected session identity".into(),
        Msg::LiveCapabilitySnapshotFailed { error } =>
            format!("Failed to update live capability snapshot: {error}").into(),
        Msg::LiveSessionDecodeFailed { session_id, error } =>
            format!("Failed to decode session {session_id}: {error}").into(),
        Msg::LiveSessionDisappeared { session_id } =>
            format!("Session {session_id} disappeared after runtime switch").into(),
        Msg::LiveSessionResolveFailed { session_id, error } =>
            format!("Failed to resolve session {session_id}: {error}").into(),
        Msg::LiveSessionSnapshotFailed { error } =>
            format!("Failed to update live session snapshot: {error}").into(),

        // ── Renderer: status badges / agent-group headers ──
        Msg::BadgeSearchPrefix => " Search '".into(),
        Msg::BadgeSearchCount { current, total } =>
            format!(" {current}/{total} ").into(),
        Msg::BadgeHistory { current, total } =>
            format!(" History {current}/{total} ").into(),
        Msg::AgentGroupTeamKind => "Team agents".into(),
        Msg::AgentGroupSubKind => "SubAgents".into(),
        Msg::AgentGroupFinished { marker, kind, terminal, total, failed } => format!(
            "{marker} {kind} · {terminal}/{total} finished · {failed} failed"
        ).into(),
        Msg::AgentGroupRunning { marker, kind, running, total } =>
            format!("{marker} Running {running}/{total} {kind}…").into(),
        Msg::SubtaskCounts { finished, total, running, pending } => format!(
            " · {finished}/{total} finished · {running} running · {pending} pending"
        ).into(),
        Msg::SubtaskPanelTeamTitle => " Team".into(),
        Msg::SubtaskPanelSubTitle => " SubTasks".into(),
        Msg::SubtaskActivityAnalyzing => "analyzing task".into(),
        Msg::SubtaskSummaryRunning { count } => format!("{count} running").into(),
        Msg::SubtaskSummaryPending { count } => format!("{count} pending").into(),
        Msg::SubtaskSummaryFailed { count } => format!("{count} failed").into(),
        Msg::SubtaskSummaryStopped { count } => format!("{count} stopped").into(),
        Msg::SubtaskPendingSuffix => " · pending".into(),
        Msg::SubtaskStatePending => "pending".into(),
        Msg::SubtaskStateRunning => "running".into(),
        Msg::SubtaskStateQueued => "queued".into(),
        Msg::SubtaskStateDone => "done".into(),
        Msg::SubtaskStateStopped => "stopped".into(),
        Msg::SubtaskStateFailed => "failed".into(),
        Msg::TodoHeaderTitle => "Tasks ".into(),
        Msg::TodoHeaderCounts { completed, in_progress, open } =>
            format!("({completed} done, {in_progress} in progress, {open} open)").into(),
        Msg::TodoMoreFold { hidden, ellipsis } =>
            format!("  +{hidden} more{ellipsis}").into(),
        Msg::MoreLinesHint { count } => format!(" +{count} more lines ").into(),
        Msg::ScrollHiddenLines { count } =>
            format!("{count} hidden lines · PgUp/PgDn").into(),
        Msg::BodyMoreLines { ellipsis, count } =>
            format!("  {ellipsis} +{count} more lines").into(),
        Msg::RoundMeta { round, elapsed } => format!(" · round {round} · {elapsed}").into(),
        Msg::RoundBare { round, elapsed } => format!("round {round} · {elapsed}").into(),
        Msg::GoalRowPausedBody => "goal paused".into(),
        Msg::GoalRowPausedMeta => " · resume by chatting · /goal stop to end".into(),
        Msg::GoalRowPausedAtCapBody => "goal paused at cap".into(),
        Msg::GoalRowPausedAtCapMeta { round } =>
            format!(" · round {round} reached · resume by chatting").into(),
        Msg::GoalRowSatisfiedBody => "goal met".into(),
        Msg::GoalRowSatisfiedMeta => " · /goal clear to dismiss".into(),
        Msg::SlashOutputSyncFailed { error } =>
            format!("slash command output sync failed: {error}").into(),
        Msg::RefreshContextStartFailed { error } =>
            format!("refresh context stats could not be started: {error}").into(),
        Msg::RefreshContextFailed { error } =>
            format!("refresh context stats failed: {error}").into(),
        Msg::SyncStoppedSharing => "Stopped sharing the current session.".into(),
        Msg::SyncNotActive => "Not currently in sync mode.".into(),
        Msg::AppRemoteStopped => "Stopped App remote access.".into(),
        Msg::AppRemoteNotRunning => "App remote access is not running.".into(),
        Msg::AppRemoteDetachSuffix { error } =>
            format!("\n{error}; the TUI stays in sync for now").into(),
        Msg::WebuiOpenedBrowser { url } => format!("Opened webui in your browser: {url}").into(),
        Msg::WebuiOpenManually { url } =>
            format!("Open this URL manually in your browser: {url}").into(),
        Msg::WebuiBindFailed { host, port, error } =>
            format!("webui failed to start: port bind for {host}:{port} failed ({error})").into(),
        Msg::WebuiRebindHint { bound_host, host } => format!(
            "\n(webui is already running, bound to {bound_host}; to rebind to {host}, run /webui stop first)"
        )
        .into(),
        Msg::WebuiLanWarning => "\n[!] Primary address is a LAN IP; only devices on the same network can reach it. For public access use a tunnel (e.g. cloudflared / Tailscale). There is no TLS, so anyone who can reach it can get in with the token.".into(),
        Msg::WebuiNonLoopbackWarning => "\n[!] Bound to a non-loopback address: anyone who can reach it can get in with this token. Use only on a trusted network (no TLS).".into(),
        Msg::WebuiStopped => "webui server stopped".into(),
        Msg::WebuiNotRunning => "webui server is not running".into(),
        Msg::AppServerBindFailed { host, port, error } =>
            format!("bind {host}:{port} failed ({error})").into(),
        Msg::AppRemoteUsage =>
            "Usage: /app <relay-address>; or set RUSTCODE_APP_RELAY to a default relay and run /app"
                .into(),
        Msg::AppRemoteLoginRequired =>
            "Remote access requires sign-in first. Run /login to sign in, then run /app.".into(),
        Msg::AppServerStartFailed { error } =>
            format!("App server failed to start: {error}").into(),
        Msg::AppRelayClientStartFailed { error } =>
            format!("Failed to start relay-client: {error}").into(),
        Msg::AppRelayClientSpawnFailed { error, bin, cache } => format!(
            "Failed to start relay-client ({error}). Tried path `{bin}`. Make sure relay-client is under {cache}, or delete that directory and retry /app to auto-download it."
        )
        .into(),
        Msg::AppPairQrBlock { qr, encoded } => format!(
            "[*] Mobile pairing\n\nThe companion mobile app is provided by your relay operator:\n1. Open the scan feature in the mobile app\n2. Point it at the QR code below to pair\n\n{qr}\n\nOr copy this token to connect in the app:\n{encoded}\n\n(/app stop to disconnect)"
        )
        .into(),
        Msg::AppPairLinkFallback { pair_uri } =>
            format!("Pairing link (QR generation failed, paste manually): {pair_uri}").into(),
        Msg::BgSessionLoadFailed { error } =>
            format!("background session could not be loaded: {error}").into(),
        Msg::McpToolsHeader => "tools:\n".into(),
        Msg::McpToolsEmpty { status } => format!("  (none -- {status})\n").into(),
        Msg::McpToolsNoServer => "  (none -- server not configured)\n".into(),
        Msg::TeamPanelShown => "Team panel shown.".into(),
        Msg::TeamPanelHidden => "Team panel hidden.".into(),
        Msg::TeamPanelCleared => "Team panel cleared.".into(),
        Msg::TeamPanelUsage => "Usage: /team [show|hide|status|clear]".into(),
        Msg::InternalError { error } => format!("internal error: {error}").into(),
        Msg::LoginFailedHint { reason } =>
            format!("Login failed: {reason}. Run /login to retry.").into(),
        Msg::SteerQueuedLine { prompt } => format!("  ↳ queued: {prompt}\n").into(),
        Msg::EmptyCompletionReasoningOnly =>
            "The model produced only reasoning this turn, no answer. Press Ctrl+O to view the reasoning; retry or rephrase to continue.".into(),
        Msg::EmptyCompletionNoOutput =>
            "The model produced no answer this turn. Retry or rephrase to continue.".into(),
        Msg::TaskWordFinished => "finished".into(),
        Msg::TaskWordCompleted => "completed".into(),
        Msg::UndoFailed { error } => format!("undo failed: {error}").into(),
        Msg::CompactFailed { error } => format!("compact failed: {error}").into(),
        Msg::WebSourcesPrefix { sources } => format!("sources: {sources}").into(),
        Msg::GoalExecFailed { error } => format!("Goal execution failed: {error}").into(),
        Msg::TurnDoneDispatched => "Dispatched".into(),
        Msg::BgProjectionSessionless =>
            "background runtime changed to a sessionless state".into(),
        Msg::BgProjectionLoadFailed { bucket, session_id, error } =>
            format!("failed to load background session {bucket}/{session_id}: {error}").into(),
        Msg::BgProjectionDecodeFailed { session_id, error } =>
            format!("failed to decode background session {session_id}: {error}").into(),
        Msg::BgProjectionIdentityMismatchRuntime { session_id, catalog } =>
            format!("background session identity mismatch: runtime={session_id:?}, catalog={catalog:?}").into(),
        Msg::BgProjectionIdentityMismatchLoaded { expected, loaded } =>
            format!("background session identity mismatch: runtime={expected:?}, loaded={loaded:?}").into(),
        Msg::ReviewCompleteClean { changed_files } =>
            format!("Code review complete — no issues found across {changed_files} changed file(s).").into(),
        Msg::ReviewHeader { findings, changed_files } =>
            format!("Code review: {findings} finding(s) across {changed_files} changed file(s).\n").into(),
        Msg::ReviewFindingEntry { index, priority, confidence, location, title } =>
            format!("\n{index}. [{priority} · conf {confidence}] {location}\n   {title}\n").into(),
        Msg::ReviewFixSuggestion { suggestion } =>
            format!("   ↳ fix: {suggestion}\n").into(),
        Msg::ReviewMoreFindings { hidden, shown } =>
            format!("\n… and {hidden} more (showing the top {shown} by priority).\n").into(),
        Msg::ReviewIncompleteHeader { stop, findings, changed_files } =>
            format!("Code review incomplete ({stop}) — coverage is partial, not a clean review. {findings} confirmed finding(s) across {changed_files} changed file(s).").into(),
        Msg::ReviewIncompleteReason { reason } => format!("\nReason: {reason}").into(),
        Msg::ReviewDeepIncomplete { total, note } =>
            format!("Deep review incomplete -- every dimension failed (0/{total}). Coverage is not reliable.{note}\n").into(),
        Msg::ReviewDeepClean { changed_files, completed, total, note } =>
            format!("Deep review complete -- no issues found across {changed_files} changed file(s) ({completed}/{total} dimensions completed){note}.\n").into(),
        Msg::ReviewDeepHeader { findings, changed_files, completed, total } =>
            format!("Deep review: {findings} finding(s) across {changed_files} changed file(s) . {completed}/{total} dimensions completed").into(),
        Msg::ReviewDeepDeduped { count } => format!(" . deduped {count}").into(),
        Msg::ReviewVerifyDropped { count } => format!(" . verify dropped {count}").into(),
        Msg::ReviewFailedDimensions { list } => format!("Failed dimensions: {list}\n").into(),
        Msg::ReviewDeepFindingEntry { index, priority, confidence, location, dims, title } =>
            format!("\n{index}. [{priority} . conf {confidence}] {location} . dims: {dims}\n   {title}\n").into(),
        Msg::ReviewActivityHead => "review".into(),
        Msg::ReviewActivityHeadLabeled { label } => format!("review [{label}]").into(),
        Msg::ReviewActivityFindingOne { count } => format!("{count} finding").into(),
        Msg::ReviewActivityFindingMany { count } => format!("{count} findings").into(),
        Msg::ReviewActivityThinking => "thinking".into(),
        Msg::ReviewActivityReporting => "reporting finding".into(),
        Msg::ReviewActivityPreparing => "review · preparing diff".into(),
        Msg::ReviewActivityAnalyzing { files } =>
            format!("review · analyzing {files} file(s)").into(),
        Msg::ReviewStageVerify => "verify".into(),
        Msg::ModeWordPlan => "plan".into(),
        Msg::ModeWordAcceptEdits => "accept edits".into(),
        Msg::ModeWordBuild => "build".into(),
        Msg::ModeWordAuto => "auto".into(),
        Msg::ModeSwitchedLine { mode } => format!("  Switched to {mode} mode.\n").into(),
        Msg::GoalMetBanner { reason } => format!("  [+] Goal met: {reason}\n").into(),
        Msg::GoalPausedBanner { reason } => format!("  \u{23f8} Goal paused: {reason}\n").into(),
        Msg::GoalPausedByUserBanner =>
            "  \u{23f8} Goal paused; continue the conversation to resume, or use /goal stop to end.\n"
                .into(),
        Msg::GoalStoppedBanner { reason } => format!("  [!] Goal stopped: {reason}\n").into(),
        Msg::ParallelDispatchStart { count } =>
            format!("Dispatching {count} sub-agents in parallel...").into(),
        Msg::WordFailed => "failed".into(),
        Msg::ParallelSummaryOk { ok, total, elapsed } =>
            format!("\u{25cf} ParallelEditFiles . {ok}/{total} ok . {elapsed} wall").into(),
        Msg::ParallelSummaryFail { ok, failed, elapsed } =>
            format!("\u{25cf} ParallelEditFiles . {ok} ok . {failed} fail . {elapsed} wall").into(),
        Msg::BashInflightCtrlOHint =>
            "Press Ctrl+o to show real-time output while running".into(),
        Msg::VerboseOnLine { mute, reset } =>
            format!("{mute}  o Verbose mode enabled (tool output + reasoning visible) (Ctrl+o to hide){reset}\n").into(),
        Msg::VerboseOffLine { mute, reset } =>
            format!("{mute}  o Verbose mode disabled (Ctrl+o to show tool output + reasoning){reset}\n").into(),
        Msg::EffortLevelLow => "Minimal reasoning effort".into(),
        Msg::EffortLevelMedium => "Moderate reasoning effort".into(),
        Msg::EffortLevelHigh => "Deeper reasoning".into(),
        Msg::EffortLevelXhigh => "Extra-high reasoning effort".into(),
        Msg::EffortLevelMax => "Maximum reasoning depth".into(),
        Msg::EffortLevelDefault => "Return to the API default (keeps capability)".into(),
        Msg::EffortUsage { levels } =>
            format!("  Usage: /effort {levels} | default\n  Shortcut: Ctrl+T\n").into(),
        Msg::EffortCurrent { current, usage } =>
            format!("  Current reasoning effort: {current}\n{usage}").into(),
        Msg::EffortStatusUnsupported => "unsupported".into(),
        Msg::EffortStatusDefault => "default (API default)".into(),
        Msg::EffortSet { level } => format!("  o Reasoning effort set to: {level}\n").into(),
        Msg::EffortSetDefault =>
            "  o Reasoning effort: default (API-selected; capability kept)\n".into(),
        Msg::LoginQrHeader =>
            "  Sign in -- scan the QR code with your WeChat:\n\n".into(),
        Msg::LoginUrlAfterQr =>
            "\n\n  OR open the URL below in a browser:\n  ".into(),
        Msg::LoginNoQrNoUrl =>
            "  Cannot render a QR code in this terminal,\n  \
             and URL-based login is unavailable on this platform.\n  \
             Try a Unicode-capable terminal to display the QR.".into(),
        Msg::LoginUrlOnly =>
            "  Open this URL in any browser to sign in:\n  ".into(),
        Msg::LoginCancelHint => "\n\n  Press ESC to cancel\n".into(),
        Msg::AuthLoginBrowserHint =>
            "  Browser didn't open? Open the URL below in any browser to sign in:".into(),
        Msg::AuthLoginEscHint => "  Press ESC to cancel".into(),
        Msg::AuthLoginPollerStopped => "login poller stopped unexpectedly".into(),
        Msg::AuthLoginCancelled => "login cancelled by user".into(),
        Msg::AuthInvalidFilePath => "Invalid auth file path -- please sign in again".into(),
        Msg::AuthNotLoggedIn => "Not logged in -- please use /login first".into(),
        Msg::AuthInvalidAuthToml => "Invalid auth.toml -- please use /login first".into(),
        Msg::AuthAccountChanged => "Login account changed -- please retry the request".into(),
        Msg::AuthTokenRefreshFailed { error } => {
            format!("Token expired and refresh failed: {error}").into()
        }
        Msg::McpCfgCommentsWouldDelete { path } => format!(
            "{path} contains comments, and rewriting it would delete them. \
             Edit the file by hand, or remove the comments and retry."
        )
        .into(),
        Msg::McpServerNeedsCommandOrUrl { name } => format!(
            "MCP server '{name}' must have either 'command' (stdio) or 'url' (http)"
        )
        .into(),
        Msg::McpAuthTypeUnsupported { name, ty } => {
            format!("MCP server '{name}' has unsupported auth.type '{ty}'").into()
        }
        Msg::McpCfgNameEmpty => "MCP server name must not be empty".into(),
        Msg::McpCfgCommandEmpty => "command must not be empty".into(),
        Msg::McpCfgUrlEmpty => "url must not be empty".into(),
        Msg::McpCfgProviderEmpty => "provider must not be empty".into(),
        Msg::McpCfgRootNotObject => "MCP config root must be a JSON object".into(),
        Msg::McpOAuthBrowserHintServer { name } => format!(
            "  Browser didn't open? Open the URL below to authorize MCP server '{name}':"
        )
        .into(),
        Msg::McpOAuthBrowserHintGithub =>
            "  Browser didn't open? Open the URL below to authorize GitHub MCP:".into(),
        Msg::McpOAuthStateMismatch => "OAuth state mismatch".into(),
        Msg::McpOAuthRefreshNoRefreshToken { server } => format!(
            "MCP server {server} OAuth token is expired and has no refresh token"
        )
        .into(),
        Msg::McpOAuthRefreshNoTokenEndpoint { server } => format!(
            "MCP server {server} OAuth token is expired and has no saved token endpoint"
        )
        .into(),
        Msg::McpOAuthRefreshNoClientId { server } => format!(
            "MCP server {server} OAuth token is expired and has no saved client id"
        )
        .into(),
        Msg::McpOAuthRefreshFailed { status } => {
            format!("MCP OAuth refresh failed: HTTP {status}").into()
        }
        Msg::McpOAuthHttpNotOAuth { name } => format!(
            "MCP server '{name}' is HTTP but does not use OAuth auth"
        )
        .into(),
        Msg::McpOAuthStdioUnsupported { name } => format!(
            "MCP server '{name}' uses stdio; OAuth login only applies to HTTP MCP servers"
        )
        .into(),
        Msg::McpOAuthExchangeFailed { status } => {
            format!("MCP OAuth token exchange failed: HTTP {status}").into()
        }
        Msg::McpGithubClientIdRequired => "GitHub OAuth client id is required".into(),
        Msg::McpGithubSecretEnvRequired =>
            "GitHub MCP OAuth requires --client-secret-env or auth.client_secret_env in mcp.json"
                .into(),
        Msg::McpGithubExchangeFailed { status } => {
            format!("GitHub OAuth token exchange failed: HTTP {status}").into()
        }
        Msg::McpOAuthRegistrationRequired =>
            "MCP OAuth requires a pre-registered client_id because the authorization server \
             does not support dynamic client registration (RFC 7591). \
             Add a pre-registered client_id to auth.client_id in your .mcp.json and try again."
                .into(),
        Msg::McpOAuthRegisterRejected { status, body } => format!(
            "MCP OAuth dynamic client registration failed: HTTP {status} -- \
             the authorization server rejected the request. \
             Add a pre-registered client_id to auth.client_id \
             in your .mcp.json and try again.\n\
             Response: {body}"
        )
        .into(),
        Msg::McpOAuthRegisterFailed { status, body } => {
            format!("MCP OAuth dynamic client registration failed: HTTP {status}\nResponse: {body}")
                .into()
        }
        Msg::McpOAuthRequiredHint { name } => format!(
            "MCP server {name} requires OAuth; run `rustcode mcp login {name}` or `/mcp login {name}`"
        )
        .into(),
        Msg::PluginGitRequired =>
            "git is not installed or not on PATH. \
             RustCode requires git to manage plugin marketplaces. \
             Please install git (e.g. `xcode-select --install` on macOS, \
             `sudo apt install git` on Ubuntu) and restart RustCode."
                .into(),
        Msg::PluginVerbClone => "clone".into(),
        Msg::PluginVerbUpdate => "update".into(),
        Msg::PluginMpNameEmpty { name } => {
            format!("marketplace name `{name}` sanitized to empty string").into()
        }
        Msg::PluginMpExists { name } => {
            format!("marketplace `{name}` already exists; remove first").into()
        }
        Msg::PluginMpDirExists { path } => format!(
            "directory {path} already exists but is not registered; remove it manually"
        )
        .into(),
        Msg::PluginMpNotFound { name } => format!("marketplace `{name}` not found").into(),
        Msg::PluginMpHasPlugins { name } => format!(
            "marketplace `{name}` has installed plugins; uninstall them first"
        )
        .into(),
        Msg::PluginGitCloneFailed { stderr } => format!("git clone failed: {stderr}").into(),
        Msg::PluginGitPullFailed { stderr } => format!("git pull failed: {stderr}").into(),
        Msg::PluginGitRevParseFailed { stderr } => {
            format!("git rev-parse failed: {stderr}").into()
        }
        Msg::PluginReloginHintManaged => "re-run /login to sign in again".into(),
        Msg::PluginReloginHintNeutral =>
            "this build has no managed sign-in service; use an SSH URL or update \
             your local git credentials and retry"
                .into(),
        Msg::PluginGitAuthUntrusted { verb, stderr } => format!(
            "{verb} failed: this repository requires authentication (private repository). \
             Use an SSH URL (git@...) or configure git credentials first, then retry.\n\
             Original error: {stderr}"
        )
        .into(),
        Msg::PluginGitAuthExpired { verb, stderr } => format!(
            "{verb} failed: login expired or credentials are invalid -- run /login to sign \
             in again, then retry.\n\
             Original error: {stderr}"
        )
        .into(),
        Msg::PluginGitAuthLoginRequired { verb, stderr } => format!(
            "{verb} failed: this private repository requires authentication. Sign in with \
             /login first (credentials are then used automatically), or use an SSH URL \
             (git@...).\n\
             Original error: {stderr}"
        )
        .into(),
        Msg::PluginGitAuthRetryFailed { verb, hint, stderr } => format!(
            "{verb} failed: the repository is still inaccessible with the signed-in \
             credentials ({hint}).\n\
             Original error: {stderr}"
        )
        .into(),
        Msg::PluginUrlMalformed { url } => format!("malformed git url: {url}").into(),
        Msg::PluginUrlUnsupported { url } => {
            format!("unsupported or malformed git url: {url}").into()
        }
        Msg::PluginUrlMissingHost { url } => format!("git url missing host: {url}").into(),
        Msg::PluginUrlMissingPath { url } => {
            format!("git url missing repository path: {url}").into()
        }
        Msg::PluginUrlBadScheme { scheme } => {
            format!("unsupported git url scheme: {scheme}").into()
        }
        Msg::PluginInstallDirRegistered { path } => format!(
            "plugin install dir already exists and is registered: {path}"
        )
        .into(),
        Msg::PluginAlreadyInstalledError { id } => format!(
            "plugin `{id}` is already installed.\n\
             PS: To reinstall, first run `/plugin uninstall {id}` then `/plugin install {id}`"
        )
        .into(),
        Msg::PluginAlreadyInProject { path } => {
            format!("plugin already installed in project at {path}").into()
        }
        Msg::PluginAlreadyInProjectScope { id, scope } => {
            format!("plugin `{id}` already installed in project scope {scope}").into()
        }
        Msg::PluginSubdirEmpty => "git-subdir source has empty path".into(),
        Msg::PluginSparseCheckoutFailed { stderr } => {
            format!("git sparse-checkout failed: {stderr}").into()
        }
        Msg::PluginCheckoutFailed { stderr } => format!("git checkout failed: {stderr}").into(),
        Msg::PluginSubdirNotFound { sub, url } => {
            format!("git-subdir path `{sub}` not found in repo {url}").into()
        }
        Msg::PluginGithubForm { repo } => {
            format!("github repo must be in `owner/name` form, got `{repo}`").into()
        }
        Msg::PluginGithubChars { repo } => {
            format!("github repo `{repo}` contains disallowed characters").into()
        }
        Msg::PluginGithubDash { repo } => {
            format!("github repo `{repo}` segment must not start with '-'").into()
        }
        Msg::PluginLocalMissing { path } => {
            format!("local plugin source does not exist: {path}").into()
        }
        Msg::PluginPinCheckoutFailed { rev, stderr } => {
            format!("git checkout {rev} failed: {stderr}").into()
        }
        Msg::PluginSourceBadComponents { source } => format!(
            "plugin source path '{source}' contains disallowed components"
        )
        .into(),
        Msg::PluginMpNotRegistered { name } => {
            format!("marketplace `{name}` not registered").into()
        }
        Msg::PluginNotInMarketplace { plugin, marketplace } => format!(
            "plugin `{plugin}` not found in marketplace `{marketplace}`"
        )
        .into(),
        Msg::CtxUsageHeader => "Context Usage".into(),
        Msg::CtxUsageNoTurns => "(run at least one turn first -- stats are captured per turn)".into(),
        Msg::CtxUsageWaiting => "(waiting for first complete turn -- partial stats only)".into(),
        Msg::CtxProvider => "Provider".into(),
        Msg::CtxCtxName => "ctx".into(),
        Msg::CtxLabelSystemPrompt => "System prompt".into(),
        Msg::CtxLabelToolDefs => "Tool defs".into(),
        Msg::CtxLabelColdZone => "Cold zone".into(),
        Msg::CtxLabelMessages => "Messages".into(),
        Msg::CtxLabelFree => "Free".into(),
        Msg::CtxMessagesInWindow { n } => format!("Messages in window: {n}").into(),
        Msg::CtxSystemPromptHeader => "=== SYSTEM PROMPT ===".into(),
        Msg::CtxSystemPromptEmpty => "(empty -- wait for one complete turn to capture)".into(),
        Msg::CtxTokensSuffix => "tokens".into(),
        Msg::CompactNothingShort => "(nothing to compact -- conversation is short)\n".into(),
        Msg::CompactStarting => "(compacting with LLM summary...)\n".into(),
        Msg::CompactInterrupted =>
            "(compaction interrupted -- the coding runtime changed or stopped)\n".into(),
        Msg::CompactUnavailableDuringSync =>
            "Cannot compact while live sync is active; run /sync off first".into(),
        Msg::CompactUnavailableDuringResync =>
            "Cannot compact until the local runtime has restored the latest synced conversation".into(),
        Msg::LocalRuntimeRestorePending =>
            "The local runtime is restoring the synced conversation; please wait".into(),
        Msg::LocalRuntimeRestoreTimedOut =>
            "The local runtime restore timed out; Live sync has been restored".into(),
        Msg::CompactNothingNoSavings { before, after } =>
            format!("(nothing to compact -- would not save tokens: {} -> {})\n", before, after).into(),
        Msg::CompactDropped { messages, before, after } => {
            let plural = if messages == 1 { "" } else { "s" };
            format!("(compacted -- dropped {} message{}, {} -> {} tokens)\n", messages, plural, before, after).into()
        }
        Msg::Compacting => "Compacting...".into(),
        Msg::CompactingSlow => "Compacting... (slow)".into(),
        Msg::CompactMarkDrain { messages, before, after } => {
            let plural = if messages == 1 { "" } else { "s" };
            format!("Compacted . {} message{} summarized . ~{}->~{} tok", messages, plural, before, after).into()
        }
        Msg::CompactMarkStub { saved } =>
            format!("Tool output folded . saved ~{} tok", saved).into(),
        Msg::CompactNegligibleSavings => "(this conversation doesn't need compacting)\n".into(),
        Msg::GoalHelp =>
            "  /goal -- autonomous multi-round work toward a stated condition.\n  \
             Usage:\n  \
             \u{20}\u{20}/goal <condition>     set a new goal; agent loops until the evaluator says met\n  \
             \u{20}\u{20}/goal                 show current goal status\n  \
             \u{20}\u{20}/goal status          same as above\n  \
             \u{20}\u{20}/goal clear           stop the active goal (aliases: stop, off, reset, none, cancel)\n  \
             \u{20}\u{20}/goal help            this help\n  \
             Notes:\n  \
             \u{20}\u{20}- A fast model evaluates each round; configure via [providers] +\n  \
             \u{20}\u{20}\u{20}\u{20}evaluator_provider in ~/.rustcode/config.toml.\n  \
             \u{20}\u{20}- No built-in round / time cap -- express budgets in the condition\n  \
             \u{20}\u{20}\u{20}\u{20}text itself (e.g. \"or stop after 20 turns\"). CC's /goal works the same way.\n  \
             \u{20}\u{20}- Esc / Ctrl+C stops the goal at any time.\n".into(),
        Msg::GoalStatus { condition, round, mins, secs } =>
            format!("  * Goal: {}\n  Round: {}\n  Elapsed: {}m {}s\n", condition, round, mins, secs).into(),
        Msg::GoalNoActive =>
            "  No active goal.\n  Usage: /goal <condition>   |   /goal help\n".into(),
        Msg::GoalCleared => "  Goal cleared.\n".into(),

        // ── /loop ──
        Msg::LoopStatus { label, round, mins, secs } =>
            format!("  ↻ loop: {} . round {} . {}m {}s\n", label, round, mins, secs).into(),
        Msg::LoopNoActive =>
            "  No active /loop.\n  Usage: /loop <interval> <cmd>  or  /loop <prompt>\n".into(),
        Msg::LoopCleared => "  /loop stopped.\n".into(),
        Msg::LoopRound { round, stats } =>
            format!("[*] loop round {} . {}", round, stats).into(),
        Msg::LoopStopped => "[!] loop stopped (limit reached)\n".into(),
        Msg::LoopEnded { reason } =>
            format!("  ↻ Loop ended: {reason}\n").into(),
        Msg::LoopNoPersistHint =>
            "  (note: the loop won't survive a restart / resume)".into(),
        Msg::CmdDescLoop =>
            "Repeat a prompt/command on an interval, or let the model self-pace".into(),
        Msg::ModelNoImageSupport { model } => format!(
            "Current model \"{}\" does not support image input and no \
             vision_preprocessor_provider is configured. Use /model to \
             switch to a vision-capable model, or set \
             vision_preprocessor_provider in config.",
            model
        )
        .into(),
        Msg::VisionPreprocessorUnresolvable { model, provider } => format!(
            "Current model \"{}\" does not support image input; the configured \
             vision_preprocessor_provider \"{}\" does not resolve (check the name \
             matches a provider/model in your config). Fix the name, or use \
             /model to switch to a vision-capable model.",
            model, provider
        )
        .into(),
        // ── --dangerously-skip-permissions / -y ──
        Msg::BypassWarningBanner =>
            "\u{26a0} --dangerously-skip-permissions is active: all tool calls are auto-approved (no permission prompts)\n".into(),
        Msg::BypassWarningHeadless =>
            "[headless] --dangerously-skip-permissions: all tool calls are auto-approved".into(),

        Msg::AdminWarningBanner =>
            "\x1b[33m\u{26a0} Warning: Running with Administrator privileges.\n   The model may have access to system files.\n   Consider running {brand} without elevation, inside a scoped working directory.\x1b[39m\n".into(),
        Msg::AdminWarningHeadless =>
            "[warning] Running with Administrator privileges -- model may have access to system files.".into(),

        Msg::CtrlCAgainToExit => "  (press Ctrl+C again to exit)\n".into(),
        Msg::EscAgainToUndo => "  (press Esc again to open Rewind)\n".into(),
        Msg::BashInputHint => "Enter to run as a bash command".into(),
        Msg::ShellModeHint => "! for shell mode".into(),
        Msg::PendingMessagesTitle =>
            "Messages to be submitted after next tool call (press esc to interrupt and send immediately)".into(),
        Msg::PendingMessagesNotSent { count } =>
            format!("{count} pending message(s) were not sent because the runtime stopped").into(),
        Msg::HintMultiLineInput =>
            "  \u{24d8} Multi-line input: end the line with `\\` then press Enter.\n    \
            Works in every terminal. (Shift / Alt / Ctrl + Enter may also work\n    \
            depending on the terminal's keyboard protocol -- try them out.)\n\n"
                .into(),

        // ── /bg (background sessions) ──
        Msg::BgHelp =>
            "  /bg                 Send current session to background and open a new foreground\n  /bg list            List background sessions\n  /bg <N>             Resume background slot N\n  /bg drop <N>        Drop background slot N\n  /bg help            Show this help\n".into(),
        Msg::BgListEmpty => "  No background sessions.\n".into(),
        Msg::BgListHeader => "  #   ID        State      Created   Summary\n".into(),
        Msg::BgListRow { slot, short_id, state, age, summary } =>
            format!("  {:<3} {:<8}  {:<9}  {:<8}  {}\n", slot, short_id, state, age, summary).into(),
        Msg::BgStateRunning => "running".into(),
        Msg::BgStateIdle => "idle".into(),
        Msg::BgStateDone => "done".into(),
        Msg::BgStateCancelled => "cancelled".into(),
        Msg::BgStateError => "error".into(),
        Msg::BgAgeNow => "now".into(),
        Msg::BgAgeMinutes { n } => format!("{n}m").into(),
        Msg::BgAgeHours { n } => format!("{n}h").into(),
        Msg::BgAgeDays { n } => format!("{n}d").into(),
        Msg::BgSlotLimitReached { max } =>
            format!("background slot limit reached ({max})").into(),
        Msg::BgBackgroundCurrent { new_id, slot, old_id, state } =>
            format!("  New foreground session [{new_id}]\n  Background: [#{slot}] {old_id} (state: {state})\n").into(),
        Msg::BgInvalidSlot { slot, available } =>
            format!("invalid background slot {slot} (available: {available})").into(),
        Msg::BgNoRuntimeClient => "background slot has no runtime client".into(),
        Msg::BgSwitchProviderTransition =>
            "/bg cannot switch the foreground while a provider transition is in progress".into(),
        Msg::BgSwitchRuntimePending =>
            "/bg cannot switch the foreground while an interactive runtime request is pending".into(),
        Msg::BgSwitchLiveSync =>
            "/bg cannot switch the foreground while live sync is attached; run /sync off first".into(),
        Msg::BgTaskFallbackName => "background task".into(),
        Msg::BgStartFailed { error } =>
            format!("background task could not be started: {error}").into(),
        Msg::BgResumed { slot, short_id } =>
            format!("  Resumed background [#{slot}] {short_id}\n").into(),
        Msg::BgPreviousForegroundMoved { slot } =>
            format!("  Previous foreground moved to [#{slot}]\n").into(),
        Msg::BgDropped { slot, short_id } =>
            format!("  Dropped background [#{slot}] {short_id}\n").into(),
        Msg::BgTaskStarted { slot, short_id } =>
            format!("  Background: [#{slot}] {short_id} (state: running)\n").into(),
        Msg::BgTaskTimedOut { secs } =>
            format!("Background task timed out after {secs}s.").into(),
        Msg::BgTaskError { error } =>
            format!("Error: {error}").into(),
        Msg::BgTaskCancelled => "Cancelled.".into(),
        Msg::BgTaskNoSummary => "Task completed (no summary text).".into(),
        // -- CLI rustcode --help i18n --
        Msg::CliAbout => "AI coding assistant in your terminal".into(),
        Msg::CliAboutLogin => "Sign in with OAuth and claim CodingPlan models in one flow".into(),
        Msg::CliAboutLoginNeutral =>
            "Managed sign-in (distribution builds only) -- this open build uses bring-your-own-key providers; configure config.toml".into(),
        Msg::CliAboutLogout => "Sign out".into(),
        Msg::CliAboutStatus => "Show current provider and sign-in status".into(),
        Msg::CliAboutUpgrade => "Upgrade rustcode in-place to the latest released version".into(),
        Msg::CliAboutRollback => "Roll back to the previous version (swap with .bak on disk)".into(),
        Msg::CliAboutMcp => "Manage MCP server entries in .mcp.json".into(),
        Msg::CliAboutDaemon => "Start the HTTP daemon for IDE integration".into(),
        Msg::CliAboutWebui => "Start the local browser webui".into(),
        Msg::CliAboutPlugin => "Manage skill/command plugins".into(),
        Msg::CliAboutUninstall => "Uninstall {brand}: remove the binary, PATH edit, and data".into(),
        Msg::CliAboutSetup => "Install seed files (skills/commands/hooks/MCP) to ~/.rustcode/".into(),
        Msg::CliAboutHooks => "Manage hooks (list, test, enable/disable)".into(),
        Msg::CliAboutHooksList => "List all loaded hooks with their status".into(),
        Msg::CliAboutHooksTest => "Test a specific hook by name".into(),
        Msg::CliAboutHooksPaths => "Show hook configuration paths".into(),
        Msg::CliAboutPluginMarketplace => "Marketplace registry operations".into(),
        Msg::CliAboutPluginInstall => "Install a plugin from a registered marketplace".into(),
        Msg::CliAboutPluginUninstall => "Uninstall a previously-installed plugin".into(),
        Msg::CliAboutPluginList => "List installed plugins".into(),
        Msg::CliAboutMarketplaceAdd => "Clone a marketplace git repo and register it locally".into(),
        Msg::CliAboutMarketplaceRemove => "Drop a registered marketplace".into(),
        Msg::CliAboutMarketplaceUpdate => "Re-pull a registered marketplace and refresh its plugin index".into(),
        Msg::CliAboutMarketplaceList => "List registered marketplaces".into(),
        Msg::CliAboutMcpAdd => "Add or replace a stdio MCP server".into(),
        Msg::CliAboutMcpAddOauth => "Add a remote OAuth MCP server by URL (any provider)".into(),
        Msg::CliAboutMcpAddGithubOauth => "Add GitHub remote MCP server using OAuth".into(),
        Msg::CliAboutMcpLogin => "Complete OAuth login for a remote MCP server".into(),
        Msg::CliAboutMcpLogout => "Remove saved OAuth credentials for a remote MCP server".into(),
        Msg::CliHelpContinue => "Continue the previous session instead of starting a new one".into(),
        Msg::CliHelpProvider => "Provider to use (overrides config default)".into(),
        Msg::CliHelpModel => "Model to use (overrides config provider model)".into(),
        Msg::CliHelpLang => "Set interface language (e.g. en, zh-CN, zh)".into(),
        Msg::CliHelpConfig => "Path to config file".into(),
        Msg::CliHelpDir => "Working directory (defaults to current directory)".into(),
        Msg::CliHelpPrompt => "Prompt to run in headless (non-interactive) mode".into(),
        Msg::CliHelpPromptFile => "Read the prompt from a file".into(),
        Msg::CliHelpVerbose => "Show tool calls, token usage, and turn summary on stderr".into(),
        Msg::CliHelpDev => "Disable auto-update for this launch".into(),
        Msg::CliHelpDangerouslySkipPermissions => "Skip all permission prompts -- auto-approve every tool call".into(),
        Msg::CliHelpForce => "Reinstall even when already on the latest version".into(),
        Msg::CliHelpPortDaemon => "Port to listen on (default: 13456)".into(),
        Msg::CliHelpIdleTimeout => "Idle-shutdown timeout in seconds; 0 disables".into(),
        Msg::CliHelpPortWebui => "Port (default: 13457)".into(),
        Msg::CliHelpHost => "Bind address (default: 127.0.0.1)".into(),
        Msg::CliHelpUninstallYes => "Skip prompts; use per-group default decisions".into(),
        Msg::CliHelpUninstallPurge => "Wipe ~/.rustcode/ entirely".into(),
        Msg::CliHelpUninstallKeepData => "Keep ~/.rustcode/ entirely".into(),
        Msg::CliHelpUninstallDryRun => "Print the plan; do nothing".into(),
        Msg::CliHelpMcpGlobal => "Write ~/.rustcode/mcp.json instead of <dir>/.mcp.json".into(),
        Msg::CliHelpMcpDir => "Directory for project .mcp.json".into(),
        Msg::CliHelpMcpName => "Server key".into(),
        Msg::CliHelpMcpUrl => "Server HTTP URL (OAuth metadata is discovered from it)".into(),
        Msg::CliHelpMcpProvider => "Optional OAuth provider hint (normally read from the server config)".into(),
        Msg::CliHelpMcpClientId => "OAuth client id".into(),
        Msg::CliHelpHooksTestName => "Hook name to test".into(),
        Msg::CliHelpPluginSpec => "e.g. plugin@marketplace".into(),
        Msg::CliHelpMarketplaceUrl => "Git URL of a marketplace repo".into(),
        Msg::CliHelpMarketplaceName => "Marketplace name".into(),
        Msg::CliAboutHelp => "Print this message or the help of the given subcommand(s)".into(),
        Msg::CliHelpMcpCommand => "Executable and arguments".into(),
        Msg::CliAboutCompletion => "Generate a shell completion script on stdout".into(),
        Msg::CliHelpCompletionShell => "Shell to generate completions for".into(),
        Msg::CliAboutResume => "Resume a session by id or name (launches the TUI on it)".into(),
        Msg::CliHelpResumeSession => "Session id or name to resume (default: the most recent session)".into(),
        Msg::CliAboutSchedule => "Manage scheduled tasks (add/list/remove/enable/disable/sync)".into(),
        Msg::CliAboutScheduleAdd => "Add a new scheduled task".into(),
        Msg::CliAboutScheduleList => "List all scheduled tasks".into(),
        Msg::CliAboutScheduleRemove => "Remove a scheduled task by id".into(),
        Msg::CliAboutScheduleEnable => "Enable a scheduled task".into(),
        Msg::CliAboutScheduleDisable => "Disable a scheduled task (it will no longer fire)".into(),
        Msg::CliAboutScheduleRun => "Run a scheduled task immediately".into(),
        Msg::CliAboutScheduleSync => "Sync OS scheduler registrations with the stored task list".into(),
        Msg::CliHelpSchedId => "Task id".into(),
        Msg::CliHelpSchedTitle => "Human-readable name for the task".into(),
        Msg::CliHelpSchedPrompt => "Prompt text to send to the agent when the task fires".into(),
        Msg::CliHelpSchedCwd => "Working directory for the agent session (defaults to current dir)".into(),
        Msg::CliHelpSchedDaily => "Run daily at HH:MM (e.g. \"09:00\")".into(),
        Msg::CliHelpSchedWeekly => "Run weekly, format N@HH:MM (N=1..7, 1=Mon)".into(),
        Msg::CliHelpSchedEvery => "Run every N minutes (e.g. \"30m\")".into(),
        Msg::CliHelpSchedHourly => "Run once per hour".into(),
        Msg::CliHelpSchedCron => "Cron expression (e.g. \"0 9 * * 1-5\")".into(),
        Msg::CliHelpSchedMode => "Permission mode: plan | accept_edits | auto".into(),
        Msg::CliHelpSchedNotify => "Notify level: off | important | all".into(),

        // ── rustcodex standalone CLI (rustcode-clix) ──
        // clap `about` strings mirror the pre-i18n derive docs byte-for-byte.
        Msg::ClixAbout => "RustCode standalone CLI (new stack)".into(),
        Msg::ClixAboutCode => "Interactive coding agent (full assembly: tools+codeintel+web+skills+mcp+session+memory).".into(),
        Msg::ClixAboutSessions => "List this project's resumable sessions.".into(),
        Msg::ClixAboutReview => "Review the local git diff and report structured findings.".into(),
        Msg::ClixSessionsNone { dir, bucket } => {
            format!("No sessions for {dir} (bucket: {bucket})").into()
        }
        Msg::ClixSessionsRow { id, name, turns, ts } => {
            // Column widths mirror the pre-i18n literal (`{:<28}` / `{:<4}`).
            format!("{id}  {name:<28} turns {turns:<4} {ts}").into()
        }
        Msg::ClixProjectDirNotFound => "project dir not found".into(),
        Msg::ClixWorkingDirNotFound => "working dir not found".into(),
        Msg::ClixRepoNotFound { path } => format!("repo not found: {path}").into(),
        Msg::ClixMissingBaseUrl => "missing base URL: pass --base-url, set $RUSTCODE_BASE_URL, or configure a provider".into(),
        Msg::ClixMissingBaseUrlReview => "missing base URL: pass --base-url, set $RUSTCODE_BASE_URL, or add base_url to the config provider".into(),
        Msg::ClixMissingModel => "missing model: pass --model, set $RUSTCODE_MODEL, or configure a provider".into(),
        Msg::ClixMissingModelReview => "missing model: pass --model, set $RUSTCODE_MODEL, or add model to the config provider".into(),
        Msg::ClixSigningGatewayCode { url } => {
            format!("provider base_url '{url}' needs RustCode's proprietary request signing, which rustcodex cannot produce -- use a plain OpenAI-compatible endpoint").into()
        }
        Msg::ClixSigningGatewayReview { url } => {
            format!("provider base_url '{url}' is a managed signing-enforced gateway, which rustcodex cannot authenticate against (it needs the proprietary request signing). Use a standard third-party provider with an explicit api_key -- select a named [providers.<name>] entry with `--provider <name>`, or set RUSTCODE_API_KEY/RUSTCODE_BASE_URL/RUSTCODE_MODEL to a plain OpenAI-compatible endpoint.").into()
        }
        Msg::ClixNoSessionToContinue => {
            "no session to continue in this project -- start one without --continue".into()
        }
        Msg::ClixConfigLoadFailed { path } => format!("failed to load config: {path}").into(),
        Msg::ClixConfigParseFailed => "failed to parse config.toml".into(),
        Msg::ClixConfigReadFailed { path } => format!("cannot read config file: {path}").into(),
        Msg::ClixConfigMalformed { path } => format!("malformed config file: {path}").into(),
        Msg::ClixPreparing { model } => format!("preparing ({model}) ...").into(),
        Msg::ClixRuntimeStartFailed => "runtime start failed".into(),
        Msg::ClixSessionNew { id } => format!("session {id} (new)").into(),
        Msg::ClixSessionResumed { id } => format!("session {id} (resumed)").into(),
        Msg::ClixTurnAbnormal { reason } => {
            format!("turn did not complete normally: {reason}").into()
        }
        Msg::ClixTurnSnapshotUnavailable { reason, error } => {
            format!("turn snapshot unavailable after {reason}: {error}").into()
        }
        Msg::ClixAgentTerminatedUnexpectedly => "agent terminated unexpectedly".into(),
        Msg::ClixInteractiveHint => {
            "interactive mode -- /help for commands, /quit to exit".into()
        }
        Msg::ClixStdinError { error } => format!("{error} -- exiting").into(),
        Msg::ClixSigintExit => {
            "(exiting -- session saved; /quit next time, or just Ctrl-C again)".into()
        }
        Msg::ClixAgentTerminatedNote => "exiting".into(),
        Msg::ClixSessionSaved { id } => {
            format!("session saved -- resume with: rustcodex code --resume {id}").into()
        }
        Msg::ClixCancelling => "cancelling ...".into(),
        Msg::ClixRetry {
            reason,
            backoff_secs,
            attempt,
            max_attempts,
        } => format!(
            "API error {reason}; retrying in {backoff_secs}s ({attempt}/{max_attempts})"
        )
        .into(),
        Msg::ClixStreamRecovered => "recovered from the interrupted stream".into(),
        Msg::ClixStreamContinuing {
            attempt,
            max_attempts,
        } => format!("safely continuing from saved progress ({attempt}/{max_attempts})").into(),
        Msg::ClixCompacting => "compacting ...".into(),
        Msg::ClixCompacted => "compacted".into(),
        Msg::ClixCompactedNoGain => "compacted -- refused (no gain)".into(),
        Msg::ClixCompactFailed { error } => format!("compact failed: {error}").into(),
        Msg::ClixTurnEnded { reason } => format!("turn ended: {reason}").into(),
        Msg::ClixToolResultChars { count } => format!("({count} chars)").into(),
        Msg::ClixToolResultCharsNamed { name, count } => {
            format!("{name} ({count} chars)").into()
        }
        Msg::ClixYoloAutoAllow { tool, args } => format!("auto-allow {tool} {args}").into(),
        Msg::ClixDiscardedTypedAhead { count } => format!(
            "(discarded {count} typed-ahead line(s) -- an approval prompt needs a fresh answer)"
        )
        .into(),
        Msg::ClixApprovalNeeded { tool, args } => {
            format!("approval needed: {tool} {args}").into()
        }
        Msg::ClixApprovalPrompt => "allow? [y = once / always = remember / N = deny]".into(),
        // Single long line with explicit \n: a `\` line-continuation would strip
        // the following lines' leading spaces (Rust string-literal rule).
        Msg::ClixSlashHelp => "  /remember [-g] <fact>   append to project (or -g: global) memory.md\n  /forget [-g] <keyword>  remove matching entries\n  /memory                 show the merged memory the model gets\n  /compact [focus]        compact the conversation (turn boundary)\n  /sessions               list this project's sessions\n  /quit                   exit (session persists)".into(),
        Msg::ClixRememberUsage => "usage: /remember [-g] <fact>".into(),
        Msg::ClixForgetUsage => "usage: /forget [-g] <keyword>".into(),
        Msg::ClixRemembered { scope } => {
            format!("remembered ({scope}) -- injected from the next session start").into()
        }
        Msg::ClixMemoryWriteFailed { error } => format!("failed to write memory: {error}").into(),
        Msg::ClixMemoryUpdateFailed { error } => format!("failed to update memory: {error}").into(),
        Msg::ClixForgetNoMatch { keyword } => format!("nothing matched '{keyword}'").into(),
        Msg::ClixForgotEntry { entry } => format!("forgot: {entry}").into(),
        Msg::ClixMemoryEmptyHint => "(memory is empty -- /remember <fact>)".into(),
        Msg::ClixCompactionRequested => "compaction requested".into(),
        Msg::ClixUnknownSlash { name } => format!(
            "unknown command /{name} -- /help (a leading '/' is command space)"
        )
        .into(),
        Msg::ClixStdinConflict { flags } => format!(
            "{flags} all read stdin; give all but one of them a file path"
        )
        .into(),
        Msg::ClixNoChanges => "No changes to review.".into(),
        Msg::ClixRulesInjected { files, chars } => format!(
            "injected for {files} changed file(s) ({chars} chars)"
        )
        .into(),
        Msg::ClixRulesNone => "no language rules matched the changed files".into(),
        Msg::ClixTraceCustomTask { chars } => format!("custom task ({chars} chars)").into(),
        Msg::ClixTraceChangedLines { lines } => format!("{lines} changed line(s)").into(),
        Msg::ClixRunning { label, model } => {
            format!("Running {label} with {model} ...").into()
        }
        Msg::ClixTraceTools { count, profile } => {
            format!("-- trace -- {count} tool call(s): {profile}").into()
        }
        Msg::ClixTraceTokens {
            prompt,
            completion,
            cached,
        } => format!("-- tokens -- prompt {prompt} / completion {completion} / cached {cached}").into(),
        Msg::ClixPassInitial => "initial pass".into(),
        Msg::ClixPassCoverage => "coverage pass".into(),
        Msg::ClixScopeDropped { dropped, files } => format!(
            "dropped {dropped} finding(s) anchored outside the {files} changed file(s)"
        )
        .into(),
        Msg::ClixCoverageSkippedFlag => "skipped -- --no-coverage".into(),
        Msg::ClixCoverageRereview { count, files } => format!(
            "{count} changed file(s) had no findings; re-reviewing: {files}"
        )
        .into(),
        Msg::ClixCoverageTrace { count, profile } => {
            format!("-- coverage trace -- {count} tool call(s): {profile}").into()
        }
        Msg::ClixCoverageRecovered { added } => {
            format!("recovered {added} finding(s) from the re-review").into()
        }
        Msg::ClixCoverageSkippedNoSignal { findings } => format!(
            "skipped -- no high-signal uncovered files after filter (first_pass_findings={findings})"
        )
        .into(),
        Msg::ClixCoverageSkippedIncomplete { reasons } => {
            format!("skipped -- initial pass incomplete ({reasons})").into()
        }
        Msg::ClixCoverageCapped { cap, dropped } => format!(
            "capped re-review to {cap} highest-priority file(s); dropped {dropped} lower-priority uncovered file(s)"
        )
        .into(),
        Msg::ClixReviewIncompleteNoFindings => {
            "Review did not complete -- no findings were collected.".into()
        }
        Msg::ClixReviewClean => "No findings -- the diff looks clean.\n".into(),
        Msg::ClixFindingsHeader {
            total,
            p0,
            p1,
            p2,
            p3,
        } => format!("{total} finding(s): {p0} P0, {p1} P1, {p2} P2, {p3} P3\n\n").into(),
        Msg::ClixReviewerSummary { text } => format!("\n-- reviewer summary --\n{text}").into(),
        Msg::ClixReviewBailIncomplete { why } => {
            format!("review did not complete ({why}): no findings collected").into()
        }
        Msg::ClixReviewEndedEarly { why, count } => format!(
            "warning: review ended early ({why}); {count} finding(s) collected before it stopped"
        )
        .into(),
        Msg::ClixTaskStdinFailed => "failed to read task from stdin".into(),
        Msg::ClixTaskFileFailed { path } => format!("failed to read task file: {path}").into(),
        Msg::ClixTaskFileEmpty { path } => format!("task file is empty: {path}").into(),
        Msg::ClixPromptStdinFailed => "failed to read system prompt from stdin".into(),
        Msg::ClixPromptFileFailed { path } => {
            format!("failed to read system prompt file: {path}").into()
        }
        Msg::ClixDiffStdinFailed => "failed to read diff from stdin".into(),
        Msg::ClixDiffFileFailed { path } => format!("failed to read diff file: {path}").into(),
        Msg::ClixGhFailed => "failed to run `gh` -- install the GitHub CLI, or pipe the diff via `--diff-file -` (e.g. for GitLab/other forges)".into(),
        Msg::ClixGhPrFailed { pr, error } => {
            format!("`gh pr diff {pr}` failed: {error}").into()
        }
        Msg::ClixGitFailed => "failed to run `git` -- is it installed and on PATH?".into(),
        Msg::ClixGitDiffFailed { error } => format!("git diff failed: {error}").into(),
        Msg::ClixSkillDirNotFound { path } => format!("--skill-dir not found: {path}").into(),
        Msg::ClixHelpCodePrompt => "One-shot prompt: run a single turn, print the answer, exit (the session is still persisted and resumable).".into(),
        Msg::ClixHelpCodeDir => "Working directory the agent's tools are scoped to.".into(),
        Msg::ClixHelpCodeResume => "Resume a session by id (see `rustcodex sessions`).".into(),
        Msg::ClixHelpCodeContinue => "Resume the most recently updated session of this project.".into(),
        Msg::ClixHelpCodeYolo => "Auto-approve every risky tool call (CI / trusted runs).".into(),
        Msg::ClixHelpCodeNoMcp => "Skip MCP server connection.".into(),
        Msg::ClixHelpCodeNoMemory => "Skip memory.md injection.".into(),
        Msg::ClixHelpCodeNoWeb => "Skip web_fetch / web_search tools.".into(),
        Msg::ClixHelpCodeModel => "Model id (overrides $RUSTCODE_MODEL).".into(),
        Msg::ClixHelpCodeApiKey => "Provider API key (overrides $RUSTCODE_API_KEY).".into(),
        Msg::ClixHelpCodeBaseUrl => "Provider base URL (overrides $RUSTCODE_BASE_URL).".into(),
        Msg::ClixHelpCodeProvider => "Named `[providers.<name>]` config entry (overrides `default_provider`).".into(),
        Msg::ClixHelpCodeConfig => "Config file path (default: ~/.rustcode/config.toml).".into(),
        Msg::ClixHelpCodeStreamTimeout => "Max seconds to wait for each stream event (liveness guard).".into(),
        Msg::ClixHelpSessionsDir => "Project directory (default: current directory).".into(),
        Msg::ClixHelpReviewRepo => "Repository root (default: current directory).".into(),
        Msg::ClixHelpReviewJson => "Emit findings as JSON instead of a human-readable report.".into(),

        // ── /usage modal ──
        Msg::UsageTabCurrent => "Current".into(),
        Msg::UsageTabOverview => "Overview".into(),
        Msg::UsageTabModels => "Models".into(),
        Msg::UsageCurrentTitle => "Rate-limit window".into(),
        Msg::UsageResetsIn { hms } => format!("Resets in {hms}").into(),
        Msg::UsageWindowHours { hours } => format!("{hours}-hour rolling window").into(),
        Msg::UsageWindowUnavailable => "Window data unavailable".into(),
        Msg::UsageStatFavorite => "Favorite model".into(),
        Msg::UsageStatTotal => "Total tokens".into(),
        Msg::UsageStatRequests => "Requests".into(),
        Msg::UsageStatActiveDays => "Active days".into(),
        Msg::UsageStatMostActive => "Most active day".into(),
        Msg::UsageStatLongestStreak => "Longest streak".into(),
        Msg::UsageStatCurrentStreak => "Current streak".into(),
        Msg::UsageHeatLess => "Less".into(),
        Msg::UsageHeatMore => "More".into(),
        Msg::UsageModelsTitle => "Per-model usage".into(),
        Msg::UsageNoData => "No usage data available".into(),
        Msg::UsageFooterHint => "← / -> or Tab switch . Ctrl+S copy . Esc close".into(),
        Msg::UsageFetchFailed { error } => format!("Failed to load usage: {error}").into(),
        Msg::UsagePlanTitle => "Plan".into(),
        Msg::UsagePlanActive => "Active".into(),
        Msg::UsagePlanExpired => "Expired".into(),
        Msg::UsagePlanClaimedExpires { claimed, expires } =>
            format!("Claimed {claimed} . Expires {expires}").into(),
        Msg::UsagePlanRemaining { remaining, total } =>
            format!("Remaining {remaining}/{total} days").into(),
        Msg::UsageCopied => "Copied to clipboard".into(),
        Msg::UsageTableModel => "Model".into(),
        Msg::UsageTableTokens => "Tokens".into(),
        Msg::UsageTableShare => "Share".into(),
        Msg::UsageMonthShort { month } => match month {
            1 => "Jan",
            2 => "Feb",
            3 => "Mar",
            4 => "Apr",
            5 => "May",
            6 => "Jun",
            7 => "Jul",
            8 => "Aug",
            9 => "Sep",
            10 => "Oct",
            11 => "Nov",
            12 => "Dec",
            _ => "",
        }
        .into(),
        Msg::UsageWeekdayShort { weekday } => match weekday {
            0 => "Sun",
            1 => "Mon",
            2 => "Tue",
            3 => "Wed",
            4 => "Thu",
            5 => "Fri",
            6 => "Sat",
            _ => "",
        }
        .into(),
        Msg::UsageTokensPerDay => "Tokens per Day".into(),
        Msg::UsageDays { n } => format!("{n} days").into(),
        Msg::UsageSparkMeta { pct, reqs, tokens } =>
            format!("{pct}%  .  {reqs} reqs  .  {tokens}").into(),
        Msg::UsageCodingPlanOnly =>
            "Managed-account usage isn't available in this build. Run /cost for this session's local token usage.".into(),

        Msg::NotifyTitleDone => "RustCode done".into(),
        Msg::NotifyTitleCancelled => "RustCode cancelled".into(),
        Msg::NotifyTitleFailed => "RustCode failed".into(),
        Msg::NotifyTitleStopped => "RustCode stopped".into(),
        Msg::NotifyStatusDone => "Done".into(),
        Msg::NotifyStatusCancelled => "Cancelled".into(),
        Msg::NotifyStatusFailed => "Failed".into(),
        Msg::NotifyStatusStopped => "Stopped".into(),
        Msg::NotifyRounds { n } => format!("{n} rounds").into(),
        Msg::NotifyTools { n } => format!("{n} tools").into(),
        Msg::NotifyApprovalTitle => "RustCode approval needed".into(),
        Msg::NotifyApprovalBody { tool } =>
            format!("{tool} is waiting for Y/A/N").into(),

        // ── TUI user-input panel ──
        Msg::UserInputTextPlaceholder => "Enter answer...".into(),
        Msg::UserInputOwnAnswer => "Type your own answer\u{2026}".into(),
        Msg::UserInputSubmitRow => "+ Submit".into(),
        Msg::UserInputSubmitLabel => "Submit".into(),
        Msg::UserInputHintSingle { n } => format!(
            "\u{2191}\u{2193} move \u{00b7} 1-{n} select \u{00b7} Enter confirm \u{00b7} Esc cancel"
        ).into(),
        Msg::UserInputHintMultiple =>
            "\u{2191}\u{2193} move \u{00b7} Space toggle \u{00b7} Enter confirm on Submit row \u{00b7} Esc cancel".into(),
        Msg::UserInputHintText =>
            "type answer \u{00b7} Enter confirm \u{00b7} Esc cancel".into(),
        Msg::UserInputBatchNav { index, total } =>
            format!("Question {index}/{total}").into(),
        Msg::UserInputReviewTitle => "Review answers".into(),
        Msg::UserInputAnswer { answer } => format!("Answer: {answer}").into(),
        Msg::UserInputUnanswered => "Unanswered".into(),
        Msg::UserInputSubmitAll { answered, total } =>
            format!("Submit all ({answered}/{total} answered)").into(),
        Msg::UserInputHintSubmit =>
            "Enter submit \u{00b7} PgUp/PgDn review \u{00b7} Shift+Tab back \u{00b7} Esc cancel".into(),
        Msg::UserInputHintBatch =>
            "Answer \u{00b7} Tab/Shift+Tab switch question \u{00b7} Enter on Submit row submits all \u{00b7} Esc cancel".into(),
        Msg::PlainAgentsStatus { finished, total, failed } =>
            format!("Agents: {finished}/{total} finished . {failed} failed").into(),
        Msg::ModelPickerEmptyNoProviders =>
            "(no models configured -- use /provider add)".into(),
        Msg::ModelPickerEmptyNoMatch => "(no models match)".into(),
        Msg::ModelPickerEmptyQuery { query } =>
            format!("(no models match \"{query}\" -- Backspace to clear)").into(),
        Msg::CliCrashHeader { info } => format!("\nRustCode crashed: {info}").into(),
        Msg::CliCrashReport =>
            "\nPlease report this crash (with the trace above) to the issue tracker of the channel you installed RustCode from.".into(),

        // ── TUI session resume / rewind ──
        Msg::SessionResumeCancelled => "Session loading cancelled".into(),
        Msg::SessionResumeCancelling => "Cancelling session loading...".into(),
        Msg::RewindNoPoints => "This session has no Rewind points yet.".into(),
        Msg::RewindCatalogLoadFailed { error } =>
            format!("Failed to load Rewind points: {error}").into(),
        Msg::RewindFailed { error } => format!("Rewind failed: {error}").into(),
        Msg::RewindScopeConversation => "conversation".into(),
        Msg::RewindScopeCode => "code".into(),
        Msg::RewindScopeConversationAndCode => "conversation and code".into(),
        Msg::RewindSuccessMain { scope, prompt } =>
            format!("\u{21a9} Rewound {scope} to before \u{201c}{prompt}\u{201d}").into(),
        Msg::RewindSuccessFiles { n } => format!(" (restored {n} files)").into(),
        Msg::RewindSuccessEnd => ".".into(),
        Msg::ImageCacheDropped { n } =>
            format!("[Image #{n}] attachment cache was lost; removed from the message").into(),
        Msg::MoreFilesHint { hidden } =>
            format!("{hidden} more files (\u{2191}/\u{2193} to scroll)").into(),
        Msg::DiffPanelFilesChanged { count } => format!("{count} files changed").into(),
        Msg::DiffPanelRenamedFrom { path } => format!("renamed from {path}").into(),
        Msg::DiffPanelNoChanges => "No changes yet".into(),
        Msg::DiffPanelTruncated => "... diff truncated".into(),
        Msg::DiffPanelTitle => "Diff".into(),
        Msg::DiffPanelEscToClose => "Esc to close".into(),
        Msg::FileViewerSelectFile => "Select file".into(),
        Msg::DiffPanelBoundedSnapshot => "Showing a bounded snapshot; some files or lines were truncated".into(),
        Msg::DiffPanelUntracked => "Untracked file; add it to the index to view a patch".into(),
        Msg::DiffPanelPatchLimit => "Patch exceeded the display limit".into(),
        Msg::DiffScopeStaged => "Staged".into(),
        Msg::DiffScopeUnstaged => "Unstaged".into(),
        Msg::DiffPanelLoading => "Loading repository changes...".into(),
        Msg::DiffPanelFooterSelect => "↑/↓ to select . Enter to view . Esc to close".into(),
        Msg::DiffPanelFooterScroll => "↑/↓ to scroll . ← to back . Esc to back".into(),
        Msg::DiffPanelWorkerStopped => "Diff worker stopped unexpectedly".into(),
        Msg::DiffPanelBinary => "Binary file; content diff is unavailable".into(),
        Msg::DiffPanelMetadataNoHunks => "Metadata changed; no text hunks".into(),
        Msg::DiffPanelInitialChanges => "Initial changes  (repository has no HEAD)".into(),
        Msg::DiffPanelUncommittedChanges => "Uncommitted changes  (git diff HEAD)".into(),
        Msg::FileViewerOpenExternal => "open external:".into(),
        Msg::FileViewerTypeToSearch => "Type to search files".into(),
        Msg::FileViewerNoMatches => "No matching files".into(),
        Msg::FileViewerFooter => "↑↓ select . Enter open . type /~ path for external file . Esc cancel".into(),
        Msg::FileViewerReadFailed => "Failed to read".into(),
        Msg::FileViewerNotRegular => "Not a regular file".into(),
        Msg::FileViewerBinary => "File appears to be binary (contains NUL bytes)".into(),
        Msg::FileViewerNotUtf8 => "File is not valid UTF-8".into(),
        Msg::FileViewerTruncatedMarker => "truncated".into(),
        Msg::FileViewerFooterBack => "↑↓/PgUp scroll . Esc back".into(),
        Msg::FileViewerFooterClose => "↑↓/PgUp scroll . Esc close".into(),
        Msg::RewindTargetHeader => "Restore the conversation to the point before...".into(),
        Msg::RewindMoreAbove { count } => format!("↑ {count} more above").into(),
        Msg::RewindCheckpoint => "  Conversation checkpoint".into(),
        Msg::RewindNoCodeChanges => "  No code changes".into(),
        Msg::RewindFilesChanged { count } => format!("{count} files changed").into(),
        Msg::RewindCurrent => "(current)".into(),
        Msg::RewindMoreBelow { count } => format!("↓ {count} more below").into(),
        Msg::RewindScopeTitle => "Rewind to before".into(),
        Msg::RewindScopeMenuConversation => "Conversation only".into(),
        Msg::RewindScopeMenuCode => "Code only".into(),
        Msg::RewindScopeMenuBoth => "Conversation and code".into(),
        Msg::RewindUnavailable => "  (unavailable)".into(),
        Msg::RewindFooterTarget => "↑/↓ select . Enter continue . Esc cancel".into(),
        Msg::RewindFooterScope => "↑/↓ select . Enter rewind . ← back . Esc cancel".into(),
        Msg::RewindStartFailed => "Could not start Rewind".into(),
        Msg::SessionPreviewLoading => "Loading preview...".into(),
        Msg::SessionPreviewUnavailable => "Preview unavailable".into(),
        Msg::PluginUninstallMarketplaceWarning { count } =>
            format!("  This will also uninstall {count} plugins from this marketplace:").into(),
        Msg::ConfigPanelTitle { shown, total } => format!("Config ({shown} / {total})").into(),
        Msg::ConfigPanelResetHint { id } => format!("Press Delete again to reset {id}").into(),
        Msg::ConfigPanelFooter =>
            "↑↓ select . Enter change . Delete reset . Esc close".into(),
        Msg::ConfigPanelRetryAttempts { model } =>
            format!("Retry attempts (current model: {model})").into(),
        Msg::ConfigPanelPolicyImmediate => "now".into(),
        Msg::ConfigPanelPolicyNextTurn => "next turn".into(),
        Msg::ConfigPanelPolicyReload => "reload".into(),
        Msg::ConfigPanelPolicyReprepare => "reprepare".into(),
        Msg::ConfigPanelPolicyRestart => "restart".into(),
        Msg::ProviderPanelAddAccountRow => "+ Add custom provider".into(),
        Msg::ProviderPanelRequiredMark => "(required)".into(),
        Msg::ProviderPanelFieldName => "Name".into(),
        Msg::ProviderPanelFieldProtocol => "Protocol".into(),
        Msg::ProviderPanelAddAccountFormHint =>
            "Tab Next  \u{2190}-> Switch protocol  \u{21b5} Save  Esc Back  (name required; add models on the Models tab)".into(),
        Msg::ProviderPanelProtocolLocked { protocol } =>
            format!("  Protocol: {protocol} (locked)").into(),
        Msg::ProviderPanelEditFormVendorLockedHint =>
            "Tab Next  \u{21b5} Save  Esc Back  (CodingPlan: only base_url can be changed)".into(),
        Msg::ProviderPanelEditFormProtocolLockedHint =>
            "Tab Next  \u{21b5} Save  Esc Back  (vendor protocol locked)".into(),
        Msg::ProviderPanelEditAccountFormHint =>
            "Tab Next  \u{2190}-> Switch protocol  \u{21b5} Save  Esc Back".into(),
        Msg::ProviderPanelProviderNotConfigured => "this provider is not configured yet".into(),
        Msg::MenuPlaceholderSearchSessions => "Search sessions...".into(),
        Msg::MenuPlaceholderSearchDirs =>
            "Search saved directories or enter a path...".into(),
        Msg::MenuPlaceholderFilter => "Type to filter...".into(),

        // ── TUI provider reload failure ──
        Msg::ProviderReloadFailed { error } =>
            format!("provider reload failed: {error}").into(),
        Msg::ProviderReloadSupersededNote =>
            "; a newer runtime generation won the transition; following the runtime owner".into(),
        Msg::ProviderRollbackFailed { error } =>
            format!("; config rollback failed: {error}").into(),

        // ── daemon live-wire errors (WebUI chat) ──
        Msg::LiveCompactFailed { error } => format!("compact failed: {error}").into(),
        Msg::LiveSetModeFailed { error } =>
            format!("failed to switch mode: {error}").into(),
        Msg::LiveSubmitFailed { error } =>
            format!("failed to send user message: {error}").into(),
        Msg::LiveProviderReloadFailed { error } =>
            format!("provider reload failed: {error}").into(),
        Msg::LiveProviderDeactivationFailed { error } =>
            format!("provider deactivation failed: {error}").into(),
        Msg::LiveSnapshotRestoreFailed { error } =>
            format!("snapshot restore failed: {error}").into(),
        Msg::LiveUndoFailed { error } => format!("undo failed: {error}").into(),
        Msg::LiveProviderNotConfigured =>
            "no provider configured -- add a third-party API key in Settings".into(),
        Msg::LiveProviderAuthRequired =>
            "provider authentication required -- check or update the API key in Settings".into(),
        Msg::LiveProviderUnsupportedBuild =>
            "this build cannot sign requests for a managed signing gateway -- use a distribution build that ships managed-signing support, or configure a standard third-party provider in Settings".into(),

        // ── daemon auth poll errors ──
        Msg::DaemonApiLoginSessionGone =>
            "Login session no longer exists; start a new login".into(),
        Msg::DaemonApiLoginPollUnavailable => "Login service is temporarily unavailable".into(),
        Msg::DaemonApiLoginExchangeFailed => "Login authorization exchange failed".into(),
        Msg::DaemonApiAuthPersistFailed => "Failed to save login credentials".into(),
        Msg::LiveApiSessionNamingFailed { error } => {
            format!("session naming failed: {error}").into()
        }
        Msg::LiveApiRuntimeStoppedEarly => "coding runtime stopped before turn terminal".into(),
        Msg::LiveApiProviderRetry {
            reason,
            backoff_secs,
            attempt,
            max_attempts,
        } => format!(
            "API error {reason}; retrying in {backoff_secs}s ({attempt}/{max_attempts})..."
        )
        .into(),
        Msg::LiveApiStreamRecovered => "recovered from the interrupted stream".into(),
        Msg::LiveApiStreamTimeout {
            attempt,
            max_attempts,
        } => format!(
            "stream timed out; safely continuing from saved progress ({attempt}/{max_attempts})"
        )
        .into(),
        Msg::LiveApiOutputLimit {
            attempt,
            max_attempts,
        } => format!(
            "output limit reached; automatically continuing ({attempt}/{max_attempts})"
        )
        .into(),
        Msg::LiveApiRuntimeStopped { reason } => {
            format!("coding runtime stopped: {reason}").into()
        }
        Msg::LiveApiRuntimeStoppedForcedSuffix => " (forced)".into(),
        Msg::LiveApiEventSerializationFailed { error } => {
            format!("live event serialization failed: {error}").into()
        }
        Msg::LiveApiEventNotObject => "live event is not a JSON object".into(),
        Msg::LiveApiStreamLagged { skipped } => {
            format!("live stream lagged by {skipped} events; reconnect").into()
        }
        Msg::LiveApiActiveTurnModelSwitch => {
            "a turn is running; stop it before switching the model".into()
        }
        Msg::LiveApiGoalConditionEmpty => "goal condition is empty".into(),
        Msg::PermissionReasonRequiresApproval => "Requires approval".into(),
        Msg::DaemonSessionSaveEarlyStopFailed { error } => format!(
            "Warning: Failed to save native session after early stop: {error}"
        )
        .into(),
        Msg::DaemonPanicHook { loc, msg } => {
            format!("[rustcode] panic at {loc}: {msg}").into()
        }

        Msg::StreamStalled => "esc to cancel".into(),
        Msg::StreamRecoveryRunning { attempt, max_attempts } => format!(
            "stream timed out; safely continuing from saved progress ({attempt}/{max_attempts})..."
        )
        .into(),
        Msg::StreamRecoverySucceeded => "[+] recovered from the interrupted stream".into(),
        Msg::OutputTruncationRunning { attempt, max_attempts } =>
            format!("Output limit reached; automatically continuing ({attempt}/{max_attempts})").into(),
        Msg::OutputTruncationHeader => "Output limit reached".into(),
        Msg::OutputTruncationQuestion =>
            "The model reached its per-response output limit and automatic continuation is still incomplete. What should happen next?".into(),
        Msg::OutputTruncationContinue => "Continue".into(),
        Msg::OutputTruncationContinueDesc =>
            "Continue from the preserved output and write the remainder incrementally".into(),
        Msg::OutputTruncationStop => "Stop".into(),
        Msg::OutputTruncationStopDesc =>
            "Keep the output produced so far and end this turn".into(),
        Msg::ConhostScrollHint =>
            "Tip: the classic Windows console is limited -- no scroll-back while a task runs, \
             and glyphs/the mascot render degraded. \x1b[1;96mWindows Terminal\x1b[0m gives the full experience."
                .into(),

        // ── rustcode-daemon startup banner + fatal lines ──
        Msg::DaemonIdleTimeout { minutes } => format!("Idle timeout: {minutes} minutes").into(),
        Msg::DaemonIdleTimeoutDisabled => "Idle timeout: disabled".into(),
        Msg::DaemonWarnNonLoopback { host } => format!(
            "Warning: binding to non-loopback address '{host}'. \
             The daemon exposes sensitive endpoints (chat, file-edit, tool-execution). \
             Ensure the network is trusted or use a reverse proxy with authentication."
        )
        .into(),
        Msg::DaemonWarnDangerousTools { env } => {
            format!("Warning: {env}=1 enables bash and write-capable daemon tools.").into()
        }
        Msg::DaemonListening { addr } => {
            format!("RustCode API server listening on http://{addr}").into()
        }
        Msg::DaemonApiEndpoints => "API endpoints:".into(),
        Msg::DaemonEpHealth => "Health check".into(),
        Msg::DaemonEpProject => "Get current working directory".into(),
        Msg::DaemonEpCd => "Change working directory (like /cd command)".into(),
        Msg::DaemonEpProjects => "List historical projects".into(),
        Msg::DaemonEpProjectSessions => "List sessions in a project".into(),
        Msg::DaemonEpSessionDetail => "Get session detail".into(),
        Msg::DaemonEpSessionDelete => "Delete a session".into(),
        Msg::DaemonEpSessionRename => "Rename a session".into(),
        Msg::DaemonEpSessionRepair => "Inspect or repair a session".into(),
        Msg::DaemonEpSessionsAll => "List all sessions (cross-project)".into(),
        Msg::DaemonEpSessionsSearch => "Search sessions by name".into(),
        Msg::DaemonEpModels => "List available models".into(),
        Msg::DaemonEpChat => "Stream chat response (SSE)".into(),
        Msg::DaemonEpConfigGet => "Get sanitized config".into(),
        Msg::DaemonEpConfigReload => "Reload config from disk".into(),
        Msg::DaemonEpProvidersList => "List providers".into(),
        Msg::DaemonEpProvidersCreate => "Create/replace provider".into(),
        Msg::DaemonEpProvidersUpdate => "Partially update provider".into(),
        Msg::DaemonEpProvidersDelete => "Delete provider".into(),
        Msg::DaemonEpProviderDefault => "Set default provider".into(),
        Msg::DaemonEpProviderThinking => "Update thinking settings".into(),
        Msg::DaemonEpSkills => "List user-invocable skills".into(),
        Msg::DaemonEpAuthStatus => "Auth status".into(),
        Msg::DaemonEpLoginStart => "Start OAuth login".into(),
        Msg::DaemonEpLoginPoll => "Poll login session".into(),
        Msg::DaemonEpLoginCancel => "Cancel login session".into(),
        Msg::DaemonEpLogout => "Logout".into(),
        Msg::DaemonEpCpSetup => "Run CodingPlan setup".into(),
        Msg::DaemonEpCpUsageSummary => "CodingPlan quota summary".into(),
        Msg::DaemonEpCpUsageDaily => "CodingPlan daily usage".into(),
        Msg::DaemonCdBodyHeading => "Change directory body:".into(),
        Msg::DaemonCdBodyHint => r#"or {"path": "-"} to go back"#.into(),
        Msg::DaemonChatBodyHeading => "Chat request body:".into(),
        Msg::DaemonFatalBind { addr, error } => {
            format!("Fatal: failed to bind to {addr}: {error}").into()
        }
        Msg::DaemonFatalServer { error } => format!("Fatal: daemon server error: {error}").into(),

        // ── rustcode-daemon HTTP API error messages ──
        Msg::DaemonApiManagedUnavailable => "This build has no managed sign-in service. Configure a third-party \
             provider with your own API key in provider settings instead."
            .into(),
        Msg::DaemonApiLoginSessionLimit => {
            "Too many login sessions are active; cancel or wait for an existing login".into()
        }
        Msg::DaemonApiLoginStartFailed => "Failed to start login".into(),
        Msg::DaemonApiLoginTaskFailed => "Login task failed".into(),
        Msg::DaemonApiInvalidLoginId => "Invalid login session ID".into(),
        Msg::DaemonApiLogoutFailed { error } => format!("Logout failed: {error}").into(),
        Msg::DaemonApiCpNotLoggedIn => "CodingPlan account is not logged in".into(),
        Msg::DaemonApiCpUsageLoadFailed => "Unable to load CodingPlan usage".into(),
        Msg::DaemonApiCdNoPrevious => "No previous directory to go back to".into(),
        Msg::DaemonApiCdNotExist { path } => format!("Directory does not exist: {path}").into(),
        Msg::DaemonApiCdNotDir { path } => format!("Not a directory: {path}").into(),
        Msg::DaemonApiCdChanged { path } => format!("Changed to {path}").into(),
        Msg::DaemonApiSessionNotFound => "Session not found".into(),
        Msg::DaemonApiSearchEmpty => "Search keyword cannot be empty".into(),
        Msg::DaemonApiSessionActive =>
            "This session is active. Switch to or create another session, then try again.".into(),
        Msg::DaemonApiDeleteNotFound => "The session was not found.".into(),
        Msg::DaemonApiDeleteInvalidId => "The session identifier is invalid.".into(),
        Msg::DaemonApiDeleteFailed =>
            "Failed to delete the session. Check the RustCode logs for details.".into(),
        Msg::DaemonApiRepairFailed =>
            "Failed to inspect or repair the session. Check the RustCode logs for details.".into(),
        Msg::DaemonApiDeleteReleaseFailed =>
            "Failed to release the current session before deleting it.".into(),
        Msg::DaemonApiRenameFailed { error } => format!("Failed to rename session: {error}").into(),
        Msg::DaemonApiMetadataNotFound => "The session metadata was not found.".into(),
        Msg::DaemonApiProjectInvalid => "The project or session identifier is invalid.".into(),
        Msg::DaemonApiSessionActiveTurn =>
            "This session has an active turn. Stop it, then try again.".into(),
        Msg::DaemonApiSessionDeleted { id } => format!("Session {id} deleted successfully").into(),
        Msg::DaemonApiSessionRenamed { id, name } =>
            format!("Session {id} renamed to '{name}'").into(),
        Msg::DaemonApiChatBusySession =>
            "This session already has an active chat operation".into(),
        Msg::DaemonApiChatBusyRequest =>
            "This request id already has an active chat operation".into(),
        Msg::DaemonApiLoginExpired => "Login session expired; start a new login".into(),
        Msg::DaemonApiLoginCancelled => "Login session was cancelled".into(),
        Msg::DaemonProvNameEmpty => "Provider name cannot be empty".into(),
        Msg::DaemonProvNameDot => "Provider name cannot be '.' or '..'".into(),
        Msg::DaemonProvNameInvalidChars =>
            "Provider name cannot contain /, \\, NUL, newline, carriage return, or tab".into(),
        Msg::CliAcpPolicyInterventionNotice =>
            "Credential protection blocked an unsafe shell operation. Complete the authenticated \
             step in a separate terminal and then ask RustCode to continue, skip the blocked \
             step, or end the task. Do not paste credentials into chat."
                .into(),
        Msg::DaemonCmdInvalidBucket => "invalid project session bucket".into(),
        Msg::DaemonCmdSessionNotFound { id } => format!("session {id} not found").into(),
        Msg::DaemonCmdProviderBuildPanicked { error } => {
            format!("provider build task panicked: {error}").into()
        }
        Msg::DaemonCmdProviderBuildFailed { error } => {
            format!("provider construction failed: {error}").into()
        }
        Msg::DaemonCmdSessionIdRequired { cmd } => {
            format!("session_id required for {cmd}").into()
        }
        Msg::DaemonCmdRememberNeedsContent => "remember needs content".into(),
        Msg::DaemonCmdForgetNeedsKeyword => "forget needs a keyword".into(),
        Msg::DaemonCmdUnknown { name } => format!("unknown command: {name}").into(),
        Msg::DaemonProvDiscoveryScheme => {
            "model discovery supports only http and https URLs".into()
        }
        Msg::DaemonProvDiscoveryNoCreds => "model discovery URL must not contain credentials".into(),
        Msg::DaemonProvInvalidModelId => {
            "model selection id is empty or contains an invalid character".into()
        }
        Msg::DaemonProvModelCountRange => "select between 1 and 100 models".into(),
        Msg::DaemonProvDupModelInRequest { model } => {
            format!("duplicate model `{model}` in request").into()
        }
        Msg::DaemonProvModelExistsInAccount { model, account } => {
            format!("model `{model}` already exists in account `{account}`").into()
        }
        Msg::DaemonProvDupSelectionInRequest { selection } => {
            format!("duplicate model selection `{selection}` in request").into()
        }
        Msg::DaemonProvSelectionExists { selection } => {
            format!("model selection `{selection}` already exists").into()
        }
        Msg::DaemonProvAccountNotFoundId { id } => {
            format!("provider account `{id}` not found").into()
        }
        Msg::DaemonProvRuntimeReadOnly => "runtime-only provider accounts cannot be modified".into(),
        Msg::DaemonProvModelEmpty => "model cannot be empty".into(),
        Msg::DaemonProvContextWindowPositive => "context_window must be greater than zero".into(),
        Msg::DaemonProvMaxTokensPositive => "max_tokens must be greater than zero".into(),
        Msg::DaemonProvAccountNotFound => "Provider account not found".into(),
        Msg::DaemonProvManagedAccount => "managed CodingPlan provider account".into(),
        Msg::DaemonProvManagedProvider => "managed CodingPlan provider".into(),
        Msg::DaemonProvModelExists { name } => {
            format!("model selection {name} already exists").into()
        }
        Msg::DaemonProvProviderExists { name } => format!("Provider '{name}' already exists").into(),
        Msg::DaemonProvAccountForModelNotFound { name } => {
            format!("account for model {name} not found").into()
        }
        Msg::DaemonProvProviderNotFound { name } => format!("Provider '{name}' not found").into(),
        Msg::DaemonProvManagedLocked { action } => {
            format!("CodingPlan providers are managed by /login and cannot be {action}").into()
        }
        Msg::DaemonProvManagedReserved => "This provider name or base URL is reserved for managed accounts, \
             which this build does not provide. Rename the provider (or change its \
             base URL) and configure it with your own api_key."
            .into(),
        Msg::DaemonProvActionModified => "modified".into(),
        Msg::DaemonProvActionReplaced => "replaced".into(),
        Msg::DaemonProvActionEdited => "edited".into(),
        Msg::DaemonProvActionDeleted => "deleted".into(),
        Msg::DaemonProvDiscoveryNoListing => {
            "This provider protocol has no supported model listing; enter the model manually".into()
        }
        Msg::DaemonProvDiscoveryTimeout => "Model discovery timed out".into(),
        Msg::DaemonProvDiscoveryTooLarge => "model listing exceeds the 4 MiB response limit".into(),
        Msg::DaemonProvDiscoveryHttpStatus { status } => {
            format!("Model endpoint returned HTTP {status}").into()
        }
        Msg::DaemonProvDiscoveryHttpStatusAuth { status } => {
            format!("Model endpoint returned HTTP {status}; check the API key").into()
        }
        Msg::DaemonProvDiscoveryUnreachable => "Could not reach the model endpoint".into(),
        Msg::DaemonProvDiscoveryOllamaParse => {
            "Ollama model listing has no valid models array".into()
        }
        Msg::DaemonProvDiscoveryParse => {
            "Model listing has no valid data array; enter the model manually".into()
        }
        Msg::DaemonProvTypeEmpty => "Provider type cannot be empty".into(),
        Msg::DaemonProvThinkingBudgetMin => "thinking_budget must be >= 1024".into(),
        Msg::DaemonProvVanished { name } => {
            format!("Provider '{name}' vanished after update").into()
        }
        Msg::DaemonApiEffortUnsupported { level, target } => {
            format!("reasoning_effort {level} not supported by {target}").into()
        }
        Msg::DaemonApiEffortUnsupportedTarget { target } => {
            format!("reasoning_effort not supported by {target}").into()
        }
        Msg::DaemonApiProviderSaveFailed { error } => {
            format!("save provider config failed: {error}").into()
        }
        Msg::DaemonChatSessionStolen { session_id } => {
            format!("chat session {session_id} became active while this turn started").into()
        }
        Msg::DaemonChatOperationInactive => "chat operation is no longer active".into(),
        Msg::DaemonApiCannotOpenFile { error } => format!("cannot open file: {error}").into(),
        Msg::DaemonApiFileResolveFailed { error } => {
            format!("file resolution task failed: {error}").into()
        }
    }
}

#[cfg(test)]
mod codingplan_crypto_tests {
    use super::*;
    use crate::i18n::Msg;

    #[test]
    fn en_official_build_required_guides_byo_or_distribution() {
        // Neutral fork: no "official releases" host. The message must still point
        // the user at a resolution -- a third-party BYO provider, or a distribution
        // that ships the managed-signing capability.
        let s = en(Msg::CpOfficialBuildRequired);
        assert!(s.contains("bring your own key"));
        assert!(s.contains("distribution"));
    }

    #[test]
    fn en_stale_clock_mentions_system_time() {
        let s = en(Msg::CpSignStaleClockSkew);
        assert!(s.to_lowercase().contains("clock") || s.to_lowercase().contains("time"));
    }

    #[test]
    fn en_replay_persisted_is_non_empty() {
        let s = en(Msg::CpSignReplayPersisted);
        assert!(!s.is_empty());
    }

    #[test]
    fn en_version_too_old_mentions_upgrade() {
        let s = en(Msg::CpSignVersionTooOld);
        assert!(s.to_lowercase().contains("upgrade") || s.to_lowercase().contains("update"));
    }

    #[test]
    fn en_upgrade_required_is_non_empty() {
        let s = en(Msg::CpUpgradeRequired);
        assert!(!s.is_empty());
    }

    #[test]
    fn en_conhost_scroll_hint_recommends_windows_terminal() {
        let s = en(Msg::ConhostScrollHint);
        assert!(s.contains("Windows Terminal"));
        assert!(s.to_lowercase().contains("scroll"));
    }
}

/// Render the outcome the caller already measured: `reload_plugins` runs
/// immediately before this toast, so the skills ARE loaded by the time it
/// prints. The previous text ("Run /reload-plugins to apply") was wrong twice
/// over -- it named a slash command that does not exist, to ask for work that
/// had already happened -- and it threw away all three counts it was handed.
fn plugin_reload_summary(loaded: usize, skipped: usize, show_details_hint: bool) -> String {
    let mut out = format!("{loaded} skill(s) loaded");
    if skipped > 0 {
        out.push_str(&format!(", {skipped} skipped"));
    }
    if show_details_hint {
        out.push_str(" (Ctrl+O for details)");
    }
    out
}
