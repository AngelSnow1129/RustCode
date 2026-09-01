#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Msg<'a> {
    // WelcomeWizard
    WelcomeBannerLine1,
    WelcomeBannerLine2,
    WelcomeOptionCodingPlan,
    WelcomeOptionCodingPlanHint,
    WelcomeOptionConfigureManually,
    WelcomeOptionConfigureManuallyHint,
    WelcomeOptionSkip,
    WelcomeOptionSkipHint,

    // ── /login (full setup flow) ──
    CodingPlanSetupFailed {
        error: &'a str,
    },
    /// Emitted inline by `/login` and `rustcode login` when the stored
    /// OAuth token comes back 401 from the CodingPlan API mid-flow.
    /// We re-run the OAuth dance, save the fresh token, and retry the
    /// whole setup once -- this line tells the user that's what's about
    /// to happen so the second "Open this URL in any browser..." block
    /// isn't a surprise.
    CpReauthAfter401,
    /// Neutral-build `/login`: the managed gateway client is not compiled in
    /// (the `codingplan` feature is off), so direct the operator to configure
    /// their own third-party provider instead of running a managed login.
    LoginManagedUnavailable,
    /// Emitted by the OpenAI provider when a gateway chat
    /// request returns 401 and our one automatic refresh_token attempt
    /// either failed or the retried request still came back 401. The
    /// raw server message (a gateway's bare "auth: token rejected"
    /// notice) is not useful to end users -- this replaces it with an
    /// actionable hint pointing at `/login`. Non-gateway endpoints still
    /// surface the verbatim server error so user-supplied API keys (sk-...)
    /// get the diagnostic detail.
    ChatAuthExpired,
    /// Hint appended to a login connection failure (connect/timeout): the
    /// endpoint is reachable from a browser but the client was reset -- likely a
    /// proxy/firewall path difference. Points at the actionable knobs.
    NetworkConnectHint,
    // SetupReport renderer (core/coding_plan/setup.rs)
    CpSetupHeader,
    CpLoggedIn {
        who: &'a str,
        username: &'a str,
        email: &'a str,
    },
    CpStepSkipped {
        reason: &'a str,
    },
    CpLoginFailed {
        error: &'a str,
    },
    CpClaimed {
        message: &'a str,
        plan_type: &'a str,
    },
    CpClaimSuccessFallback,
    CpAlreadyClaimed {
        reason: &'a str,
    },
    CpClaimFailed {
        error: &'a str,
    },
    /// Same as `CpClaimFailed` but with no trailing detail body.
    /// Used in the rare edge case where every tier returned success=
    /// false with an empty server message AND no transport error
    /// text -- there's nothing to put after `-- `, so the line stops
    /// at the prefix.
    CpClaimFailedBare,
    /// Per-tier cascade row -- winning tier, fresh claim. `plan` is the
    /// full plan label already including the "CodingPlan " prefix (the
    /// server's `plan_name`, e.g. "CodingPlan Pro", or "CodingPlan
    /// {tier}" fallback). Example (zh-CN): `  [+] CodingPlan Pro 生效`
    CpClaimTierSucceeded {
        plan: &'a str,
    },
    /// Per-tier cascade row -- winning tier, server reported the user
    /// already holds this tier or higher (`duplicate=true`). `plan` as
    /// above.
    CpClaimTierAlreadyHeld {
        plan: &'a str,
    },
    /// Per-tier cascade row -- tier was refused (2xx with success=
    /// false / 5xx / transport). `reason` is the server's human-
    /// readable message (e.g. `额度已满`, `暂无开放`) or a short
    /// rendering of the transport error.
    CpClaimTierFailed {
        tier: &'a str,
        reason: &'a str,
    },
    CpAddedProviders {
        accounts: usize,
        models: usize,
    },
    /// Locked-model row. `name` is expected to be pre-decorated with
    /// U+0336 combining strikethrough by the caller (see
    /// `coding_plan::setup::strikethrough`), so the template itself
    /// stays a plain `format!` and survives every renderer's CSI
    /// scrubber without needing SGR escapes.
    CpLocked {
        name: &'a str,
    },
    CpProviderRow {
        provider: &'a str,
        model: &'a str,
        default_suffix: &'a str,
    },
    CpDefaultSuffix,
    CpVisionAuto {
        kind: &'a str,
    },
    CpVisionUserSupplied {
        kind: &'a str,
    },
    CpVisionCleared,
    CpModelsSkipped {
        reason: &'a str,
    },
    CpModelsFailed {
        error: &'a str,
    },
    CpStatusHeader,
    CpPlanPending {
        plan: &'a str,
    },
    CpPlanActive {
        plan: &'a str,
        expires_at: &'a str,
        remaining_days: i32,
        total_days: i32,
    },
    CpUsageLine {
        usage: &'a str,
        reset_at: &'a str,
        duration: &'a str,
    },
    CpWindowQuotaExhausted,
    CpWindowQuotaHint {
        hint: &'a str,
    },
    CpStatusFetchSkipped {
        reason: &'a str,
    },
    CpStatusFetchFailed {
        error: &'a str,
    },
    /// Open-source build attempted to use a CodingPlan provider. The
    /// signing capability is not present in this build, so the request
    /// can't reach the LLM gateway. Surface a clear hint
    /// pointing to the official Releases page.
    CpOfficialBuildRequired,
    /// Official build, but no stored auth (or auth has empty
    /// `user.id` / `access_token`). The signing path needs these
    /// fields to derive a per-user key; without them the request
    /// can't be signed. Surface a "please run `/codingplan` to log
    /// in" hint instead of the misleading "official build required"
    /// message -- the user IS on an official build.
    CpAuthRequired,
    /// Server returned `RUSTCODE_SIG_STALE` -- the request's signed
    /// timestamp is outside the ±5min window the gateway accepts.
    /// Typically caused by an unsynced local clock.
    CpSignStaleClockSkew,
    /// Server returned `RUSTCODE_SIG_REPLAY` even after the client's
    /// one automatic retry with a fresh nonce. Surface a "please retry
    /// the command" hint -- usually self-heals on the next attempt.
    CpSignReplayPersisted,
    /// Server returned `RUSTCODE_SIG_INVALID` AND the alg_version is
    /// no longer in the server's `accepted_versions` set -- the client
    /// binary is too old. Force-upgrade hint.
    CpSignVersionTooOld,
    /// Server returned `426 Upgrade Required` -- emergency rotation
    /// playbook in progress; this build cannot continue without
    /// upgrading.
    CpUpgradeRequired,

    // i18n self-errors
    ErrUnsupportedLocale {
        input: &'a str,
    },

    // ── Status bar (build_status) ──
    StatusNoProvider,
    StatusRuntimeUnavailable,
    /// Open-source build with a gateway provider configured.
    /// Sending any chat will fail with `CpOfficialBuildRequired`; this
    /// hint surfaces the same diagnosis up-front so the user doesn't
    /// have to type a message to discover the dead-end.
    StatusOfficialBuildRequired,
    StatusUpgradeHint {
        version: &'a str,
    },
    /// Right-aligned status-row hint, HarmonyBrew variant: a newer version
    /// exists, upgrade via the package manager rather than `/upgrade`.
    StatusUpgradeHintPm {
        version: &'a str,
    },
    StatusModelNotConfigured,
    /// macOS / Linux variant: "Image in clipboard . ctrl+v to paste".
    /// Ctrl+V is intercepted by Windows Terminal / conhost before
    /// reaching rustcode, so Windows builds emit
    /// `StatusClipboardImageHintSlash` instead.
    StatusClipboardImageHint,
    /// Windows variant: "Image in clipboard . /paste". Tells the
    /// user to fall back on the `/paste` slash command, which works
    /// in every terminal regardless of host keybinds.
    StatusClipboardImageHintSlash,
    /// Lowest-priority status-row fallback: nudge the user toward the
    /// `/webui` command (browser UI) when no higher-priority hint
    /// (warnings / usage / upgrade) is competing for the slot.
    StatusWebuiHint,

    // ── /status command body ──
    StatusBody {
        model: &'a str,
        dir: &'a str,
        config: &'a str,
    },
    /// `/status` login line -- signed in, showing the account display name/username.
    StatusLoginLoggedIn {
        user: &'a str,
    },
    /// `/status` login line -- not signed in.
    StatusLoginNotSignedIn,
    StatusCpNotSignedIn,
    StatusCpFetchFailed {
        error: &'a str,
    },
    /// `/status` CodingPlan line when the fetch failed specifically because auth
    /// expired (`is_auth_expired`) -- a clear re-login prompt instead of the raw error.
    StatusCpAuthExpired,
    StatusCpNoActive,
    StatusCpLine {
        plan: &'a str,
        expires_at: &'a str,
        remaining_days: i32,
        total_days: i32,
    },
    StatusCpUsage {
        usage: &'a str,
        reset_at: &'a str,
        duration: &'a str,
    },
    StatusCpWindowExhausted,
    StatusCpWindowHint {
        hint: &'a str,
    },
    StatusInstructionFilesHeader,
    StatusInstructionScopeGlobal,
    StatusInstructionScopeProject,
    StatusInstructionScopeUser,
    StatusInstructionPresent {
        path: &'a str,
        label: &'a str,
        scope: &'a str,
    },
    StatusInstructionMissing {
        path: &'a str,
        label: &'a str,
        scope: &'a str,
    },
    StatusMemoryFilesHeader,
    StatusMemoryScopeGlobal,
    StatusMemoryScopeProject,
    StatusMemoryPresent {
        path: &'a str,
        scope: &'a str,
    },
    StatusMemoryMissing {
        path: &'a str,
        scope: &'a str,
    },

    // ── Help / commands ──
    HelpAvailableCommands,
    /// Full keyboard-shortcuts reference dumped to scrollback by the
    /// `/keys` slash command. Carries every line of the panel as a
    /// single multi-line string so translators can adjust column
    /// alignment per locale without rebuilding rows in Rust.
    KeybindingsHelp,

    // ── Provider wizard ──
    ProviderWizardHeader,
    ProviderWizardCancelled,
    ProviderMenuAdd,
    ProviderMenuAddDesc,
    ProviderMenuEdit,
    ProviderMenuEditDesc,
    ProviderMenuDelete,
    ProviderMenuDeleteDesc,
    ProviderMenuSetDefault,
    ProviderMenuSetDefaultDesc,
    ProviderImportPrompt,
    ProviderImportParsed {
        base_url: &'a str,
        type_name: &'a str,
        model: &'a str,
    },
    ProviderImportFailed,
    ProviderNoProviders,
    ProviderDeleteConfirm {
        name: &'a str,
    },
    ProviderDeleted {
        name: &'a str,
    },
    ProviderDeleteKept,
    ProviderDefaultSet {
        name: &'a str,
    },
    ProviderAdded {
        name: &'a str,
    },
    ProviderUpdated {
        name: &'a str,
    },
    ProviderStepName,
    ProviderStepType,
    ProviderStepTypeWithHint {
        current: &'a str,
    },
    ProviderStepBaseUrl,
    ProviderStepBaseUrlWithHint {
        current: &'a str,
    },
    ProviderDefaultHint,
    ProviderStepApiKey,
    ProviderStepApiKeyWithHint {
        hint: &'a str,
    },
    ProviderStepApiKeySet,
    ProviderStepApiKeyUnset,
    ProviderStepModel,
    ProviderStepModelWithHint {
        current: &'a str,
    },
    ProviderStepContextWindow {
        default: usize,
    },
    ProviderStepContextWindowWithHint {
        current: usize,
    },
    ProviderContextWindowInvalid,
    ProviderNameEmpty,
    ProviderBaseUrlEmpty,
    ProviderUnknownType,
    ProviderUnknownTypeEdit,
    ProviderModelEmpty,
    ProviderEditKeep,
    ProviderTypeInferred {
        type_name: &'a str,
    },
    ProviderStepNameDefault {
        default: &'a str,
    },
    ProviderStepProgress {
        current: usize,
        total: usize,
    },
    // ── Provider panel ──
    ProviderPanelTabAccounts,
    ProviderPanelTabModels,
    ProviderPanelEmptyAccounts,
    ProviderPanelNoMatchingAccounts,
    ProviderPanelEmptyModels,
    ProviderPanelNoMatchingModels,
    ProviderPanelLegacyBadge,
    ProviderPanelDefaultBadge,
    ProviderPanelModelCount {
        count: usize,
    },
    ProviderPanelAddModelRow,
    ProviderPanelAccountsHint,
    ProviderPanelManagedAccountHint,
    /// Neutral-build counterpart to [`Msg::ProviderPanelManagedAccountHint`]:
    /// the row collides with the reserved managed-account namespace, but this
    /// build ships no managed service -- never pitch `/login`.
    ProviderPanelManagedAccountHintNeutral,
    ProviderPanelModelsHint,
    ProviderPanelManagedModelsHint,
    /// Neutral-build counterpart to [`Msg::ProviderPanelManagedModelsHint`]
    /// (same reason; the managed hint says "managed by /login").
    ProviderPanelManagedModelsHintNeutral,
    ProviderPanelFilteredModelsHint {
        account: &'a str,
    },
    ProviderPanelModelSaved {
        model: &'a str,
    },
    ProviderPanelAddTitle,
    ProviderPanelEditAccountTitle {
        account: &'a str,
    },
    ProviderPanelAddModelTitle,
    ProviderPanelEditModelTitle,
    ProviderPanelFieldVendor,
    ProviderPanelFieldAccount,
    ProviderPanelFieldBaseUrl,
    ProviderPanelFieldApiKey,
    ProviderPanelFieldModel,
    ProviderPanelFieldVision,
    ProviderPanelVisionAuto,
    ProviderPanelVisionEnabled,
    ProviderPanelVisionDisabled,
    ProviderPanelFieldEffort,
    ProviderPanelFieldEffortLevels,
    ProviderPanelFieldWindow,
    ProviderPanelFieldMakeDefault,
    ProviderPanelSwitchHint,
    ProviderPanelEnvHint {
        env: &'a str,
    },
    ProviderPanelDefaultValue,
    ProviderPanelKeepOriginal,
    ProviderPanelProviderFormHint,
    ProviderPanelAccountFormHint,
    ProviderPanelModelFormHint,
    // ── Model picker ──
    ModelSwitched {
        provider: &'a str,
        model: &'a str,
    },
    ModelSwitchedAndDefault {
        provider: &'a str,
        model: &'a str,
    },

    // ── Session picker ──
    SessionLoadFailed {
        error: &'a str,
    },
    /// Inner error: a resume is already running (feeds SessionLoadFailed).
    SessionResumeInProgress,
    /// Inner error: preparation spawned task panicked/joined.
    SessionPrepJoinFailed {
        error: &'a str,
    },
    /// Inner error: session id missing after preparation scan.
    SessionNotFoundById {
        session_id: &'a str,
    },
    /// Inner error: preparation backstop timeout (large session/slow disk).
    SessionPrepTimeout,
    /// Fallback project title when the working dir has no basename.
    ProjectFallbackWord,
    SessionResumedLabel {
        name: &'a str,
    },
    SessionBusyForked {
        source_id: &'a str,
        fork_id: &'a str,
    },

    // ── Todo panel ──
    TodoPanelTitle,
    TodoPanelCompleted {
        n: usize,
    },
    TodoPanelMore {
        n: usize,
    },

    // ── Approval panel ──
    ApprovalAllowOnce,
    ApprovalAlwaysAllow {
        tool: &'a str,
    },
    /// "Always" for the single-file write tools, whose grant is scoped to the
    /// target's DIRECTORY (not the whole tool) -- so the label names the folder.
    ApprovalAlwaysAllowFolder,
    /// "Always" for `bash`, whose grant is scoped to THIS COMMAND (not the whole
    /// tool) -- so the label says "this command", not "Always allow bash".
    ApprovalAlwaysAllowCommand,
    ApprovalDeny,
    ApprovalHint,
    /// Header line above the interactive approval options, naming what is being
    /// approved (the `> Tool(detail)` scrollback row can be far above / hidden).
    ApprovalHeader {
        tool: &'a str,
        detail: &'a str,
    },
    /// Advisory line shown under a credential-suspected Bash approval, warning that
    /// allowing it may send secrets or sensitive content to the model provider.
    CredentialApprovalNote,

    // ── Tool result markers ──
    ToolDenied,
    /// The credential-aware Bash policy rejected a tool call. Fixed text only:
    /// never reflect the rejected command or model-controlled middleware output.
    ToolBlockedBySecurityPolicy,
    PolicyRecoveryHeader,
    PolicyRecoveryQuestion,
    PolicyRecoveryComplete,
    PolicyRecoveryCompleteDesc,
    PolicyRecoverySkip,
    PolicyRecoverySkipDesc,
    PolicyRecoveryInstructions,
    PolicyRecoveryInstructionsDesc,
    PolicyRecoveryEnd,
    PolicyRecoveryEndDesc,
    PolicyRecoverySafeInstructions,
    PolicyRecoveryCompletedLocally,
    PolicyRecoverySkippedLocally,
    PolicyRecoverySubmitError,

    // ── Execution mode ──
    CmdSwitchedAutoMode,
    CmdSwitchedAcceptEditsMode,

    SessionTimeJustNow,
    SessionTimeMinAgo {
        n: u64,
    },
    SessionTimeHourAgo {
        n: u64,
    },
    SessionTimeDayAgo {
        n: u64,
    },
    SessionMsgCount {
        count: usize,
    },
    SessionNameEmpty,
    SessionNameTooLong {
        max: usize,
    },
    SessionNameControlChars,
    SessionListFailed {
        error: &'a str,
    },
    SessionRenamed {
        old: &'a str,
        new: &'a str,
    },
    SessionSaveFailed {
        error: &'a str,
    },
    SessionDeleted {
        name: &'a str,
    },
    SessionDeleteConfirm {
        name: &'a str,
    },
    SessionDeleteFailed {
        error: &'a str,
    },
    SessionNoneSelected,
    /// Persistent footer hint in the `/resume` picker advertising the key
    /// actions (open / delete / search) so they're discoverable.
    SessionPickerHint,
    /// Title row of the `/resume` picker: current 1-based position in the
    /// filtered list, total sessions in the project, and the project name.
    SessionPickerTitle {
        n: usize,
        total: usize,
        project: &'a str,
    },
    /// Bare title of the `/resume` picker when the search box is focused --
    /// no position / total / project suffix, just the heading.
    SessionPickerTitleBare,
    /// Hint shown when the project has no sessions at all.
    SessionPickerEmptyProject,
    /// Hint shown when the filter matches no sessions (empty query).
    SessionPickerEmptyFilter,
    /// Hint shown when the filter matches no sessions for a specific query.
    SessionPickerEmptyFilterQuery {
        query: &'a str,
    },
    SessionRenameEditing {
        buffer: &'a str,
    },

    // ── Dir picker ──
    DirPickerTitle {
        n: usize,
        total: usize,
    },
    DirPickerHint,
    DirPickerEmptyPath {
        query: &'a str,
    },
    DirCurrent,
    DirNotExists {
        path: &'a str,
    },
    DirChanged {
        path: &'a str,
    },
    DirNotADirectory {
        path: &'a str,
    },
    CdHomeUnknown,
    CdNoPrevious,

    // ── Language ──
    /// Confirmation rendered to scrollback after the user picks a
    /// locale via `/language` (modal or arg). Already includes the
    /// leading "  " indent and trailing "\n" so the call site is just
    /// `renderer.render(UiLine::CommandOutput(t(Msg::LanguageSwitched
    /// { ... }).into_owned()))`.
    LanguageSwitched {
        label: &'a str,
        locale: &'a str,
    },

    // ── Idle / onboarding hints ──
    /// "type something, or press " (text before the slash)
    IdleHintPrefix,
    /// "/" (the slash shortcut itself -- kept separate for accent styling)
    IdleHintSlash,
    /// " to browse commands" (text after the slash)
    IdleHintSuffix,
    /// Complete plain-text version: "type something, or press / to browse commands"
    IdleHintFull,
    /// "/provider" command label
    IdleHintProvider,
    /// "to add a custom model" (text after /provider)
    IdleHintProviderSuffix,
    /// Complete plain-text version: "/provider  to add a custom model"
    IdleHintProviderFull,
    /// "/webui" command label
    IdleHintWebui,
    /// "open a synced session in the browser" (text after /webui)
    IdleHintWebuiSuffix,
    /// Complete plain-text version: "/webui  open a synced session in the browser"
    IdleHintWebuiFull,

    // ── Welcome screen tips ──
    /// Heading above the tips list on the welcome screen.
    WelcomeTipsHeading,
    /// Welcome tip: /login command description.
    WelcomeTipLogin,
    /// Welcome tip: /provider command description.
    WelcomeTipProvider,
    /// Welcome tip: /model command description.
    WelcomeTipModel,
    /// Welcome tip: /resume command description.
    WelcomeTipResume,
    /// Welcome tip: /setup command description.
    WelcomeTipSetup,
    /// Welcome tip: /skills command description.
    WelcomeTipSkills,
    /// Welcome tip: /plugin command description.
    WelcomeTipPlugin,
    /// Welcome tip: /webui command description.
    WelcomeTipWebui,
    /// Welcome tip: /mcp command description.
    WelcomeTipMcp,
    /// Welcome tip: /plan command description.
    WelcomeTipPlan,
    /// Welcome tip: /session command description.
    WelcomeTipSession,
    /// Welcome tip: /loop command description.
    WelcomeTipLoop,
    /// Welcome tip: /goal command description.
    WelcomeTipGoal,
    /// Welcome tip: /init command description.
    WelcomeTipInit,
    /// Welcome tip: /language command description.
    WelcomeTipLanguage,
    /// Welcome tip: /usage command description.
    WelcomeTipUsage,

    // ── Slash-command high-frequency messages ──
    CmdSwitchedPlanMode,
    CmdSwitchedBuildMode,
    CmdNewSession,
    CmdSessionTransitionPending,
    CmdSessionTransitionFailed {
        error: &'a str,
    },
    CmdCapabilityReloadFailed {
        error: &'a str,
    },
    CmdNoProviders,
    CmdSessionListLoading,
    CmdNoSessions,
    CmdUnknownCommand {
        name: &'a str,
    },
    /// /cmd with args: required but no arguments supplied.
    CmdCustomArgRequired {
        name: &'a str,
    },
    CmdLoginFailed {
        error: &'a str,
    },
    CmdLogoutDone,
    CmdLogoutFailed {
        error: &'a str,
    },
    CmdWhoamiNotSignedIn,
    /// Neutral-build counterpart to `CmdWhoamiNotSignedIn`: no managed account
    /// exists and `/login` cannot work, so the copy points at `/provider`
    /// (bring-your-own-key) instead. Also used for the provider
    /// `AuthenticationRequired` hint in a neutral build. Selected at the call
    /// site via `rustcode_auth::managed_login_available()`.
    CmdWhoamiNotSignedInNeutral,
    CmdReloadDone {
        provider: &'a str,
        model: &'a str,
    },
    CmdReloadFailed {
        error: &'a str,
    },
    CmdUndoNotSupported,
    CmdUndoDone {
        target: usize,
        last: usize,
    },
    CmdUndoDiskWarning,
    CmdUndoNoTurns,
    CmdUndoOutOfRange {
        requested: usize,
        available: usize,
    },
    CmdUndoBusy,
    /// `/rewind` rejected because a turn is running (rewind mutates history +
    /// files, so it must not race an active turn).
    CmdRewindBusy,
    /// `/rewind` (or the double-Esc gesture) couldn't open the checkpoint
    /// picker -- used as a `"{msg}: {error}"` prefix.
    CmdRewindUnavailable,
    CmdUndoBadArg,
    CmdNoChanges,
    CmdDiffTruncated,
    CmdCheckingUpdate,
    CmdNoActiveProvider,
    /// Live/sync binding requested but no model selection is configured yet.
    CmdNoModelConfigured,
    CmdProviderUnavailable,
    /// Neutral-build variant of [`CmdProviderUnavailable`]: no managed sign-in
    /// exists, so steer to `/provider` (bring-your-own-key) instead of `/login`.
    CmdProviderUnavailableNeutral,
    CmdProviderUnsupportedBuild,
    CmdProviderReloading,
    SubmitHeldUntilProviderReady,
    SubmitHeldUntilLogin,

    // ── Approval prompt ──
    ApprovalPromptAlt {
        tool: &'a str,
        detail: &'a str,
    },
    ApprovalWaitingLabel,
    ApprovalAllow,
    ApprovalAlways,

    // ── Cancelled / Error prefix ──
    Cancelled,
    ErrorPrefix {
        msg: &'a str,
    },

    // ── Upgrade messages ──
    UpgradeSuccess {
        from: &'a str,
        to: &'a str,
    },
    UpgradeManifestFetched {
        version: &'a str,
    },
    UpgradeDownloading {
        pct: i32,
        bytes: u64,
        total: u64,
    },
    UpgradeVerifying,
    UpgradeReplacing,
    UpgradeDone {
        version: &'a str,
        backup: &'a str,
    },
    UpgradeAlreadyLatest {
        current: &'a str,
        latest: &'a str,
    },
    UpgradeFailed {
        error: &'a str,
    },
    UpgradeRolledBack {
        exe: &'a str,
        backup: &'a str,
    },

    // ── CLI startup auto-upgrade (stderr progress) ──
    // Distinct shapes from the interactive `/upgrade` flow above (no
    // backup/restart prose): one-line progress for the pre-launch updater.
    /// `[*] New version available: <version>` at CLI startup.
    CliUpgradeAvailable {
        version: &'a str,
    },
    /// In-place download progress line. The template keeps the leading
    /// `\r` and trailing spaces so `eprint!` redraws overwrite the prior
    /// line; `mb`/`total_mb` are pre-formatted with one decimal.
    CliUpgradeDownloading {
        pct: i32,
        mb: &'a str,
        total_mb: &'a str,
    },
    /// `[+] Verifying sha256` once the download finishes.
    CliUpgradeVerifying,
    /// `[+] Upgrading to <version>...` right before re-exec.
    CliUpgradeApplying {
        version: &'a str,
    },
    /// Re-exec failed after a successful apply -- non-fatal; the new binary
    /// runs on the next launch.
    CliUpgradeReexecFailed {
        error: &'a str,
    },
    /// Startup update check failed (network/timeout) -- non-fatal, retried
    /// in the background.
    CliUpgradeCheckFailed,
    /// A previously-staged upgrade could not be applied -- non-fatal, the
    /// current binary keeps running.
    CliUpgradeApplyFailed {
        error: &'a str,
    },
    /// Dev builds never auto-update (informational).
    CliUpgradeDevDisabled,

    // ── CLI (non-TUI) command output ──
    /// Top-level fatal error wrapper for the `rustcode` binary exit path.
    CliFatalError {
        error: &'a str,
    },
    /// "Starting RustCode..." banner after a login setup before TUI launch.
    CliStartingAfterLogin,
    /// `rustcode daemon` startup banner.
    CliDaemonStarting {
        port: u16,
    },
    /// `rustcode daemon` Ctrl+C hint.
    CliDaemonStopHint,
    /// `rustcode daemon` in-process server fatal error.
    CliDaemonFatal {
        error: &'a str,
    },
    /// First-run login setup failed (non-fatal, falls through to TUI).
    CliLoginSetupFailed {
        error: &'a str,
    },
    /// `rustcode logout` confirmation.
    CliLoggedOut,
    /// `rustcode status` logged-in header.
    CliStatusLoggedIn {
        username: &'a str,
        id: &'a str,
    },
    /// `rustcode status` display-name line.
    CliStatusName {
        name: &'a str,
    },
    /// `rustcode status` email line.
    CliStatusEmail {
        email: &'a str,
    },
    /// `rustcode status` auth-file line (trailing newline kept in template).
    CliStatusAuthFile {
        path: &'a str,
    },
    /// `rustcode status` not-logged-in line (managed build only).
    CliStatusNotLoggedInManaged,
    /// `rustcode status` login hint (managed build only).
    CliStatusLoginHint,
    /// `rustcode status` neutral-build hint: BYO providers, no managed account.
    CliStatusHintNeutral,
    /// Neutral-build fallback replacing the managed `login` flow output.
    CliManagedLoginNotBuilt,
    /// Re-OAuth failed inside the managed setup orchestrator (feature-gated).
    CliReauthFailed {
        error: &'a str,
    },
    /// Persisting config after the managed setup flow failed (feature-gated).
    CliConfigSaveFailed {
        path: &'a str,
        error: &'a str,
    },
    /// Writing the managed sync marker failed (non-fatal; feature-gated).
    CliSyncMarkerWriteFailed {
        error: &'a str,
    },
    /// `rustcode upgrade`: manifest fetched line.
    CliUpgradeLatest {
        version: &'a str,
    },
    /// `rustcode upgrade`: download progress line (leading `\r` + trailing
    /// spaces kept so the redraw overwrites the previous line).
    CliUpgradeDownloadProgress {
        pct: i32,
        bytes: u64,
        total: u64,
    },
    /// `rustcode upgrade`: SHA256 verification line.
    CliUpgradeVerifyingSha,
    /// `rustcode upgrade`: binary replacement line.
    CliUpgradeReplacingBinary,
    /// `rustcode upgrade`: success banner.
    CliUpgradeCmdDone {
        version: &'a str,
        backup: &'a str,
    },
    /// `rustcode upgrade`: hint to run the new binary.
    CliUpgradeStartNewHint,
    /// `rustcode upgrade`: failure line (event channel).
    CliUpgradeCmdFailed {
        error: &'a str,
    },
    /// `rustcode upgrade`: driver task panicked.
    CliUpgradePanicked {
        error: &'a str,
    },
    /// `rustcode rollback`: event-channel rollback line.
    CliRollbackCmdDone {
        exe: &'a str,
        backup: &'a str,
    },
    /// `rustcode rollback`: command-result rollback line.
    CliRollbackCmdDoneTwo {
        current: &'a str,
        saved: &'a str,
    },
    /// `rustcode rollback`: hint to run the rolled-back binary.
    CliRollbackStartHint,

    // ── Headless (`-p`/`--print`) stderr lines ──
    /// Provider connection retry backoff line (non-JSONL headless mode).
    CliHeadlessProviderRetry {
        reason: &'a str,
        backoff_secs: u64,
        attempt: u32,
        max_attempts: u32,
    },
    /// 429 rate limit: kernel auto-waits and resumes.
    CliHeadlessRateAutoResume {
        secs: u64,
    },
    /// 429 rate limit (third-party provider): retry later with seconds hint;
    /// `reason` is the pre-formatted ` -- {server_message}` suffix (may be empty).
    CliHeadlessRateRetry {
        reason: &'a str,
        secs: u64,
    },
    /// 429 rate limit (third-party provider): paused, no reset time.
    CliHeadlessRatePaused {
        reason: &'a str,
    },
    /// Rate-limit window exhausted, reset time known.
    CliHeadlessRateWindowResetAt {
        reset_at: &'a str,
    },
    /// Rate-limit window exhausted, reset countdown known.
    CliHeadlessRateWindowSecs {
        secs: u64,
    },
    /// Rate-limit window exhausted, paused with no reset info.
    CliHeadlessRateWindowPaused,
    /// Headless auto-approval of a permission request.
    CliHeadlessAutoApproved {
        tool: &'a str,
    },
    /// Headless denial of a permission request (unattended, no TTY).
    CliHeadlessDenied {
        tool: &'a str,
    },
    /// `rustcode setup`: cannot read the current working directory.
    CliSetupCwdError {
        error: &'a str,
    },
    /// `rustcode setup`: setup run failed.
    CliSetupFailed {
        error: &'a str,
    },
    /// First-run `--seed-config` seed applied (sentence after the `[seed]` tag).
    CliSeedInitialized {
        path: &'a str,
        source: &'a str,
    },
    /// `--seed-config` source rejected as invalid (non-fatal).
    CliSeedInvalid {
        error: &'a str,
    },
    /// `--seed-config` seed could not be copied (non-fatal).
    CliSeedIoError {
        error: &'a str,
    },
    /// Config file loaded but some provider sections failed; the rendered
    /// notice is also surfaced inside the TUI.
    CliConfigLoadWarnings {
        path: &'a str,
        warnings: &'a str,
    },
    /// Config file failed to load entirely; defaults are used.
    CliConfigLoadFailed {
        path: &'a str,
        error: &'a str,
    },
    /// On-exit "how to resume this session" hint; `cmd` is the pre-rendered
    /// command line (`rustcode -p "..." --resume <id>` or `rustcode resume <id>`).
    CliResumeHint {
        cmd: &'a str,
    },
    /// Windows-only best-effort warning when the console code page could not
    /// be switched to UTF-8 (CJK IME may garble). Multi-line template.
    CliWindowsCodePage {
        input: u32,
        output: u32,
    },
    /// `--prompt-file` could not be read (hard exit, headless/TUI both).
    CliPromptFileReadFailed {
        path: &'a str,
        error: &'a str,
    },
    /// `--resume <id-or-name>` matched no session in this project.
    CliResumeNoMatch {
        selector: &'a str,
    },
    /// Headless start with a named `--provider` that resolves to nothing.
    CliHeadlessNoProviderNamed {
        name: &'a str,
        path: &'a str,
    },
    /// Headless start with no provider configured at all.
    CliHeadlessNoProvider {
        path: &'a str,
    },

    // ── `rustcode mcp` subcommand ──
    /// `mcp add`: stdio server registered. `name` is pre-rendered with debug quotes.
    CliMcpAdded {
        name: &'a str,
        path: &'a str,
        program: &'a str,
        args: usize,
    },
    /// `mcp add-github-oauth`: GitHub OAuth MCP server registered.
    CliMcpAddedGithub {
        name: &'a str,
        path: &'a str,
    },
    /// `mcp login`: OAuth token persisted.
    CliMcpLoginSaved {
        provider: &'a str,
        name: &'a str,
        scopes: usize,
    },
    /// `mcp logout`: a saved token was removed.
    CliMcpLogoutRemoved {
        name: &'a str,
    },
    /// `mcp logout`: no saved token existed.
    CliMcpLogoutNotFound {
        name: &'a str,
    },
    /// `mcp login`: named server absent from mcp.json.
    CliMcpServerNotFound {
        name: &'a str,
    },

    // ── `rustcode hooks` subcommand ──
    CliHooksLoadedHeader,
    CliHooksNone,
    /// Table header: event column.
    CliHooksTableEvent,
    /// Table header: count column.
    CliHooksTableCount,
    /// Table footer row label.
    CliHooksTableTotal,
    CliHooksConfigFiles,
    /// Paths listing: global hooks file line (mark printed separately).
    CliHooksPathGlobal {
        path: &'a str,
    },
    /// Paths listing: project hooks file line (mark printed separately).
    CliHooksPathProject {
        path: &'a str,
    },
    /// Paths listing: no home directory for the global file.
    CliHooksPathNoHome,
    CliHooksUntrustedHeader,
    /// Untrusted-plugin hook row; `events` is the pre-joined event list.
    CliHooksUntrustedRow {
        plugin: &'a str,
        count: usize,
        events: &'a str,
    },
    CliHooksTestNotFound {
        name: &'a str,
    },
    CliHooksTestNoneLoaded,
    CliHooksTestAvailable,
    CliHooksTesting {
        event: &'a str,
    },
    CliHooksFieldCommand {
        command: &'a str,
    },
    CliHooksFieldTimeout {
        ms: u64,
    },
    CliHooksFieldMatcher {
        matcher: &'a str,
    },
    CliHooksResultHeader,
    CliHooksDuration {
        duration: &'a str,
    },
    /// Status line: `  Status:    {label} ({detail})` with the ASCII label kept
    /// as a tag word and `detail` pre-rendered.
    CliHooksFieldStatus {
        label: &'a str,
        detail: &'a str,
    },
    CliHooksStatusSuccess,
    CliHooksStatusBlock,
    CliHooksStatusExitCode {
        code: i32,
    },
    CliHooksStatusSignal,
    CliHooksDidNotComplete {
        ms: u64,
    },
    CliHooksPathsHeader,
    CliHooksDocsHeader,
    CliHooksDocsEntry,

    // ── `rustcode plugin` / `marketplace` subcommands ──
    CliPluginMpAdded {
        name: &'a str,
        commit: &'a str,
        plugins: usize,
    },
    CliPluginMpRemoved {
        name: &'a str,
    },
    CliPluginMpUpdated {
        name: &'a str,
        commit: &'a str,
    },
    CliPluginMpNone,
    /// Marketplace list row; fields are pre-rendered (commit truncated).
    CliPluginMpRow {
        name: &'a str,
        source: &'a str,
        commit: &'a str,
        plugins: usize,
    },
    CliPluginInstalled {
        plugin: &'a str,
        marketplace: &'a str,
    },
    /// Bare-name plugin found in several marketplaces; `list` is the
    /// pre-rendered suggestion block (command lines stay English).
    CliPluginInstallAmbiguous {
        plugin: &'a str,
        list: &'a str,
    },
    CliPluginNotFound {
        plugin: &'a str,
    },
    /// Post-install notice about untrusted hooks shipped by the plugin.
    CliPluginUntrustedNotice {
        plugin: &'a str,
        count: usize,
        events: &'a str,
    },
    CliPluginUninstalled {
        plugin: &'a str,
        marketplace: &'a str,
    },
    CliPluginNotInstalled {
        plugin: &'a str,
    },
    /// Bare-name plugin installed from several marketplaces; `list` pre-rendered.
    CliPluginUninstallAmbiguous {
        plugin: &'a str,
        list: &'a str,
    },
    CliPluginNoHooks {
        name: &'a str,
    },
    CliPluginTrusted {
        count: usize,
        name: &'a str,
        events: &'a str,
    },
    /// Plugin trusted from several installations; `list` pre-rendered.
    CliPluginTrustAmbiguous {
        name: &'a str,
        list: &'a str,
    },
    CliPluginUntrusted {
        name: &'a str,
    },
    CliPluginNone,
    /// anyhow error-context prefixes (rendered as `{prefix}: {error}`).
    CliPluginErrAddMp,
    CliPluginErrRemoveMp,
    CliPluginErrUpdateMp,
    CliPluginErrInstall,
    CliPluginErrResolve,
    CliPluginErrUninstall,

    // ── `rustcode schedule` subcommand ──
    /// Argument validation errors (values pre-rendered with debug quotes).
    CliSchedDailyBad {
        value: &'a str,
    },
    CliSchedWeeklyBad {
        value: &'a str,
    },
    CliSchedWeekdayBad {
        value: &'a str,
    },
    CliSchedWeeklyTimeBad {
        value: &'a str,
    },
    CliSchedEveryBad {
        value: &'a str,
    },
    CliSchedEveryIntBad {
        value: &'a str,
    },
    CliSchedEveryZero,
    CliSchedFrequencyRequired,
    /// anyhow context wrappers (ids pre-rendered with debug quotes).
    CliSchedRemoveFailed {
        id: &'a str,
    },
    CliSchedTaskNotFound {
        id: &'a str,
    },
    CliSchedSaveFailed {
        id: &'a str,
    },
    /// OS scheduler registration failed (non-fatal; suggests `schedule sync`).
    CliSchedRegFailed {
        id: &'a str,
        error: &'a str,
    },
    CliSchedSyncInstalled {
        id: &'a str,
    },
    CliSchedSyncInstallFailed {
        id: &'a str,
        error: &'a str,
    },
    CliSchedSyncUninstalled {
        id: &'a str,
    },
    CliSchedSyncUninstallFailed {
        id: &'a str,
        error: &'a str,
    },
    CliSchedSyncDone {
        installed: usize,
        uninstalled: usize,
        errors: usize,
    },
    CliSchedAdded {
        id: &'a str,
        title: &'a str,
    },
    CliSchedNone,
    CliSchedRemoved {
        id: &'a str,
    },
    CliSchedEnabled {
        id: &'a str,
    },
    CliSchedDisabled {
        id: &'a str,
    },
    CliSchedRunSkipped {
        id: &'a str,
    },
    /// Scheduled task whose working directory is gone.
    CliSchedBadCwd {
        cwd: &'a str,
        id: &'a str,
    },
    /// `schedule list` row; state/registration pre-localized.
    CliSchedListRow {
        id: &'a str,
        title: &'a str,
        next: &'a str,
        last: &'a str,
        state: &'a str,
        reg: &'a str,
    },
    CliSchedStateOn,
    CliSchedStateOff,
    CliSchedRegRegistered,
    CliSchedRegMissing,
    CliSchedRegUnknown,

    // ── `rustcode uninstall` interactive flow ──
    CliUninstallPurgeConflict,
    CliUninstallNoTty,
    CliUninstallBinaryRequired,
    CliUninstallProcsAborted,
    CliUninstallProcsFound {
        count: usize,
    },
    CliUninstallKillPrompt,
    CliUninstallKillFailed {
        pid: u32,
        error: &'a str,
    },
    CliUninstallKillWarn {
        pid: u32,
        error: &'a str,
    },
    CliUninstallDryRun,
    CliUninstallGroup1Plan,
    CliUninstallGroup2Plan,
    CliUninstallGroup3Plan,
    CliUninstallGroup1Prompt,
    CliUninstallGroup2Prompt,
    CliUninstallGroup3Prompt,
    /// Tag word inside `[...]` for groups that will be deleted.
    CliUninstallTagWillRemove,
    /// Tag word inside `[...]` for groups that will be kept.
    CliUninstallTagKeep,
    CliUninstallIntro,
    CliUninstallGroup1Declined,
    CliUninstallSummaryHeader,
    CliUninstallContinuePrompt,
    /// Per-group proceed prompt; `suffix` is `[Y/n]` or `[y/N]`.
    CliUninstallProceedPrompt {
        suffix: &'a str,
    },
    CliUninstallActionRemove,
    CliUninstallActionKeep,
    CliUninstallLabelBinary,
    CliUninstallLabelCredentials,
    CliUninstallLabelState,
    /// Summary row: `  {action}: {count} items ({label})`.
    CliUninstallSummaryRow {
        action: &'a str,
        count: usize,
        label: &'a str,
    },
    CliUninstallResultRemoved,
    CliUninstallResultKept,
    CliUninstallResultFailed,
    CliUninstallResultBackups,
    /// `rustcode webui` / 404 body when the embedded frontend assets are absent.
    CliWebuiNotBuilt,

    // ── /config command ──
    ConfigProviderLabel {
        provider: &'a str,
        path: &'a str,
    },

    // ── /cost command ──
    CostTokenReport {
        prompt: usize,
        completion: usize,
        cached: usize,
        cache_rate: usize,
        total: usize,
    },
    CostUnattributed {
        tokens: u64,
    },

    // ── /usage command ──
    /// Shown when the user runs /usage but has no stored CodingPlan auth.
    UsageCodingPlanOnly,

    // ── /think command ──
    ThinkStatus {
        /// Whether extended thinking is currently on -- the on/off word is
        /// localized inside the template (never pass raw "enabled"/"disabled").
        enabled: bool,
        budget: u32,
        provider: &'a str,
    },
    ThinkEnabled {
        budget: u32,
    },
    ThinkDisabled,
    ThinkBudgetSet {
        n: u32,
    },
    ThinkBudgetTooSmall {
        n: u32,
    },
    ThinkBudgetUsage,
    ThinkUsage,

    // ── /remember, /forget ──
    RememberUsage,
    ForgetUsage,
    MemoryScopeGlobal,
    MemoryScopeProject,
    Remembered {
        scope: &'a str,
        content: &'a str,
    },
    AlreadyRemembered {
        scope: &'a str,
        content: &'a str,
    },
    RememberFailed {
        error: &'a str,
    },
    ForgetNoMatch {
        keyword: &'a str,
    },
    ForgotOne,
    ForgotMany {
        count: usize,
    },
    MemoryEmpty,

    // ── /team ──
    TeamNoRuns,
    TeamSummary {
        runs: usize,
        completed: usize,
        running: usize,
        failed: usize,
        stopped: usize,
    },

    // ── 通用开关词 ──
    WordOn,
    WordOff,

    // ── /schedule list ──
    ScheduleListEmpty,
    ScheduleListHeader,
    ScheduleRow {
        id: &'a str,
        title: &'a str,
        next: &'a str,
        last: &'a str,
        state: &'a str,
    },

    // ── /background ──
    BackgroundUsage,

    // ── /init ──
    InitKickoff,

    // ── /cd ──
    CdWorkingDir {
        cwd: &'a str,
    },

    // ── /diff ──
    DiffFailed {
        error: &'a str,
    },

    // ── /upgrade ──
    /// Shown when `/upgrade` (or rollback) is invoked in a HarmonyBrew-managed
    /// build: self-update is disabled, point the user at `brew upgrade`.
    UpgradePackageManaged,
    UpgradeUnknownArg {
        arg: &'a str,
    },

    // ── /skills ──
    SkillsNone,
    SkillsAvailable,
    SkillUnknown {
        name: &'a str,
    },
    SkillsLoaded {
        names: &'a str,
    },

    // ── /mcp ──
    McpReloading {
        count: usize,
    },
    McpConnecting,
    McpConnectingServer {
        name: &'a str,
    },
    McpNoServersConfigured,
    McpClearedReconnecting,
    McpClearedNoServers,
    McpToolsUsage,
    McpServersHeader,
    /// Discoverability hint appended to `/mcp` status when one or more
    /// project-source servers are withheld because the project is untrusted.
    McpBlockedTrustHint {
        count: usize,
    },
    McpReloadFailed {
        error: &'a str,
    },
    // /mcp login / logout
    McpOAuthLoginUsage,
    McpOAuthLogoutUsage,
    McpOAuthLoadConfigFailed {
        error: &'a str,
    },
    McpOAuthServerNotFound {
        server: &'a str,
    },
    McpOAuthStarting {
        server: &'a str,
    },
    McpOAuthSaved {
        provider: &'a str,
        server: &'a str,
    },
    McpOAuthFailed {
        error: &'a str,
    },
    McpOAuthTokenRemoved {
        server: &'a str,
    },
    McpOAuthNoToken {
        server: &'a str,
    },
    McpOAuthLogoutFailed {
        error: &'a str,
    },
    // /mcp trust / untrust
    McpProjectTrusted,
    McpProjectUntrusted,
    McpProjectNotTrusted,
    LspServerStarted {
        name: &'a str,
        ext: &'a str,
    },
    LspServerFailed {
        name: &'a str,
        ext: &'a str,
        error: &'a str,
    },

    // ── /worktree ──
    WorktreeUsage,
    WorktreeCreateUsage,
    WorktreeCreated {
        branch: &'a str,
        base: &'a str,
        path: &'a str,
    },
    WorktreeCreateFailed {
        error: &'a str,
    },
    WorktreeNoActive,
    WorktreeListFailed {
        error: &'a str,
    },
    WorktreeActiveHeader,
    WorktreeHasChanges,
    WorktreeClean,
    WorktreeCurrent,
    WorktreeDoneBack {
        path: &'a str,
    },
    WorktreeDoneMergeHint {
        branch: &'a str,
    },
    WorktreeNoSession,
    WorktreeCleanupUsage,
    WorktreeCleaned {
        branch: &'a str,
    },
    WorktreeCleanedSwitched {
        path: &'a str,
    },
    WorktreeCleanupUncommitted {
        branch: &'a str,
    },
    WorktreeCleanupFailed {
        error: &'a str,
    },

    // ── /help commands (custom commands subcommand) ──
    HelpCustomCommandsHeader,
    HelpCustomNone,
    HelpCustomCreateHint,
    HelpSourceGlobal,
    HelpSourceProject,

    // ── /setup ──
    /// Header line: "[+] Setup complete -- 3 installed, 1 skipped, 0 failed . 120ms"
    SetupHeader {
        installed: usize,
        skipped: usize,
        failed: usize,
        duration_ms: u64,
    },
    /// "Installed:" section label in setup report.
    SetupInstalledLabel,
    /// "Skipped:" section label in setup report.
    SetupSkippedLabel,
    /// "Failed:" section label in setup report.
    SetupFailedLabel,
    /// Per-item installed row: "  [+] skill:rustcode-automation-recommender -> /path"
    SetupInstalledRow {
        kind: &'a str,
        slug: &'a str,
        path: &'a str,
    },
    /// Per-item skipped row: "  - skill:xyz (hash match)"
    SetupSkippedRow {
        kind: &'a str,
        slug: &'a str,
        reason: &'a str,
    },
    /// Per-item failed row: "  [x] mcp:xyz -- error message"
    SetupFailedRow {
        kind: &'a str,
        slug: &'a str,
        error: &'a str,
    },
    /// "[!] Tip: Run /setup ..." -- first-run hint shown above the prompt
    /// when the project has no setup state yet.
    CmdSetupTip,
    /// "Running rustcode setup..." -- shown while setup is in progress.
    CmdSetupRunning,
    /// "Skills reloaded -- N available" -- after setup completes and skills are reloaded.
    CmdSetupSkillsReloaded {
        count: usize,
    },
    /// "setup error: {e}" -- when setup::run returns an error.
    CmdSetupError {
        error: &'a str,
    },
    /// "Running setup skill..." -- after seeds installed and skill is auto-invoked.
    CmdSetupRunningSkill,
    /// "Setup skill not found..." -- when the setup skill cannot be resolved or expanded.
    CmdSetupSkillMissing,

    // ── /plugin ──
    PluginUsage,
    PluginMarketplaceUsage,
    PluginInstallUsage,
    PluginInstallNotFound {
        plugin: &'a str,
    },
    PluginInstallAmbiguous {
        plugin: &'a str,
    },
    PluginUninstallUsage,
    PluginUninstallNotFound {
        plugin: &'a str,
    },
    PluginUninstallAmbiguous {
        plugin: &'a str,
    },
    PluginNoMarketplaces,
    PluginMarketplacesHeader,
    PluginNoInstalled,
    PluginInstalledHeader,
    PluginMarketplaceCloning {
        url: &'a str,
    },
    PluginMarketplaceRemoved {
        name: &'a str,
    },
    PluginMarketplaceRemoveFailed {
        error: &'a str,
    },
    PluginMarketplaceUpdating {
        name: &'a str,
    },
    PluginMarketplaceListFailed {
        error: &'a str,
    },
    /// Calm one-line advisory (yellow) for a NON-FATAL startup marketplace
    /// auto-update failure. `detail` is the first line of the underlying error.
    /// Replaces the red multi-line git-stderr dump that reads like a crash.
    PluginAutoUpdateSkipped {
        detail: &'a str,
    },
    /// Calm one-line advisory shown once at startup when offline mode is active.
    /// Informs the user that web tools and auto-update are disabled.
    OfflineModeActive,
    /// Startup advisory: `count` installed plugins ship UNTRUSTED hooks that will
    /// not run until the user grants trust. `names` is a comma-joined plugin-name list.
    PluginHooksUntrusted {
        count: usize,
        names: &'a str,
    },
    PluginInstalling {
        plugin: &'a str,
        marketplace: &'a str,
    },
    PluginInstallingByName {
        plugin: &'a str,
    },
    PluginAlreadyInstalled {
        id: &'a str,
    },
    // Interactive `/plugin` manager modal.
    PluginMgrBrowse,
    PluginMgrAdd,
    PluginMgrRemove,
    PluginMgrInstalled {
        count: usize,
    },
    PluginMgrInstalledMark,
    PluginMgrInstalledStatus,
    PluginMgrInstallableStatus,
    PluginMgrInstallingStatus,
    PluginMgrUpdatingStatus,
    PluginMgrHintNav,
    PluginMgrHintToggle,
    PluginMgrHintRemove,
    PluginMgrHintUninstall,
    PluginMgrHintUrl,
    PluginMgrHintPending,
    PluginMgrHintUpdating,
    PluginMgrInstallingLabel,
    PluginMgrEmptyMarketplaces,
    PluginMgrEmptyPlugins,
    PluginMgrEmptyInstalled,
    PluginMgrCloning,
    PluginMgrInstalling {
        plugin: &'a str,
    },
    PluginMgrUpdating {
        plugin: &'a str,
    },
    PluginMgrEscToCancel,
    PluginMgrRemoveMarketplaceTitle,
    PluginMgrRemoveMarketplacePrompt {
        name: &'a str,
    },
    PluginMgrRemoveMarketplaceYes,
    PluginMgrRemoveMarketplaceNo,
    PluginMgrRemoveMarketplaceHint,
    // Scope selection screen.
    PluginScopeUser,
    PluginScopeUserDesc,
    PluginScopeProject,
    PluginScopeProjectDesc,
    PluginScopeLocal,
    PluginScopeLocalDesc,
    PluginScopeHint,
    PluginScopeUserShort,
    PluginScopeProjectShort,
    PluginScopeLocalShort,
    PluginActionUninstall,
    PluginActionUninstallDesc,
    PluginActionUpdate,
    PluginActionUpdateDesc,
    PluginActionDisable,
    PluginActionDisableDesc,
    PluginActionBack,
    PluginActionBackDesc,
    PluginUninstalled {
        plugin: &'a str,
        marketplace: &'a str,
    },
    PluginUninstallFailed {
        error: &'a str,
    },
    PluginListFailed {
        error: &'a str,
    },
    PluginReloadDone {
        skills: usize,
        warnings: usize,
    },
    /// Git not found on the system -- marketplace auto-install and auto-update
    /// are disabled. Shown as a friendly hint (not an error) at startup.
    PluginGitNotFound,
    /// Marketplace `add` completion toast. Emitted by `handle_plugin_job_event`
    /// for both manual `/plugin marketplace add` and the detached
    /// startup-bootstrap auto-install. `count` is the number of plugins the
    /// marketplace exposes after cloning.
    PluginMarketplaceAdded {
        name: &'a str,
        commit: &'a str,
        count: usize,
        plugins: &'a str,
    },
    /// Marketplace `update` completion toast -- HEAD actually moved. No-op
    /// pulls (HEAD unchanged) emit no toast at all so a quiet `git pull`
    /// doesn't spam the body region.
    PluginMarketplaceUpdated {
        name: &'a str,
        commit: &'a str,
    },
    /// Plugin `install` completion toast. `skipped` counts skills that the
    /// loader rejected (bad SKILL.md frontmatter, namespace collision, etc.);
    /// `show_details_hint` flips on the trailing "(Ctrl+O for details)"
    /// nudge when warnings exist and verbose mode is off.
    PluginInstallDone {
        plugin: &'a str,
        marketplace: &'a str,
        loaded: usize,
        skipped: usize,
        show_details_hint: bool,
    },
    PluginUpdateDone {
        plugin: &'a str,
        marketplace: &'a str,
        loaded: usize,
        skipped: usize,
        show_details_hint: bool,
    },
    SetupAutoReloaded {
        skills: usize,
        warnings: usize,
    },

    // ── Plugin manager modal: tabs, rows, detail labels ──
    PluginTabAll,
    PluginTabInstalled {
        count: usize,
    },
    PluginTabMarketplaces,
    PluginAutoUninstallFailed {
        name: &'a str,
        error: &'a str,
    },
    PluginNoPluginsMatch {
        query: &'a str,
    },
    PluginNoInstalledMatch {
        query: &'a str,
    },
    PluginAddMarketplaceRow,
    PluginAddMarketplacePlus,
    PluginEnterSourceRow,
    PluginExamplesRow,
    PluginBrowseRow {
        count: usize,
    },
    PluginUpdateRow {
        date: &'a str,
    },
    PluginRemoveMarketplaceRow,
    PluginInfoHeader,
    /// Detail label; includes trailing padding so the value column aligns.
    PluginNameLabel,
    PluginMarketplaceLabel,
    PluginVersionLabel,
    PluginScopeLabel,
    PluginDescriptionLabel,
    PluginSelectScopeHeader,
    PluginManageHeader,
    PluginStatusInstalling {
        label: &'a str,
    },
    PluginAvailableCount {
        count: usize,
    },
    PluginModalInstalledHeader {
        count: usize,
    },
    PluginNoInstalledFromMarketplace,
    PluginVersionUnknown,
    // Plugin category badges (Browse rows).
    PluginCategoryGit,
    PluginCategoryLinter,
    PluginCategoryFormatter,
    PluginCategoryLanguage,
    PluginCategorySecurity,
    PluginCategoryAi,
    PluginCategoryUtility,
    PluginCategoryTool,
    PluginCategoryCompletion,

    // ── Command descriptions (for help_text dynamic lookup) ──
    CmdDescWebui,
    CmdDescSetup,
    CmdDescResume,
    CmdDescRename,
    CmdDescLogin,
    CmdDescLogout,
    CmdDescWhoami,
    CmdDescModel,
    CmdDescProvider,
    CmdDescStatus,
    CmdDescConfig,
    CmdDescReload,
    CmdDescCd,
    CmdDescInit,
    CmdDescBg,
    CmdDescBackground,
    CmdDescDiff,
    CmdDescClear,
    CmdDescSession,
    CmdDescCost,
    /// Description for the `/usage` slash command -- opens the CodingPlan usage modal.
    CmdDescUsage,
    CmdDescContext,
    CmdDescCompact,
    CmdDescRemember,
    CmdDescForget,
    CmdDescMemory,
    CmdDescMcp,
    CmdDescUndo,
    /// Description for the `/rewind` slash command -- opens the checkpoint
    /// picker (same as the double-Esc gesture) to restore an earlier point.
    CmdDescRewind,
    CmdDescWorktree,
    CmdDescUpgrade,
    CmdDescPlan,
    CmdDescBuild,
    CmdDescAuto,
    CmdDescThink,
    CmdDescEffort,
    CmdDescHelp,
    CmdDescKeys,
    CmdDescLanguage,
    CmdDescQuit,
    CmdDescSkills,
    CmdDescPlugin,
    /// Description for the `/paste` slash command -- pulls a clipboard
    /// image and attaches it as `[Image #N]`. Exists for Windows
    /// users whose Ctrl+V is swallowed by Windows Terminal / conhost
    /// before reaching the app, but works on every platform.
    CmdDescPaste,
    /// Description for the `/copy` slash command -- copies a code block from the
    /// last reply to the clipboard, or with `/copy msg` the full reply markdown.
    CmdDescCopy,
    /// `/copy`: confirmation after a code block lands on the clipboard.
    CopyOk {
        lines: usize,
        chars: usize,
    },
    /// `/copy msg`: confirmation after the full reply markdown lands on the
    /// clipboard. Distinct from `CopyOk` so the hint says "reply" not "code
    /// block" -- the user copied the whole message, not a fenced block.
    CopyOkMsg {
        lines: usize,
        chars: usize,
    },
    /// `/copy`: the last reply has no fenced code block to copy.
    CopyNoCodeBlock,
    /// `/copy msg`: the reply is empty/whitespace-only, so there is no message
    /// body to copy. Distinct from `CopyNoCodeBlock` so the hint can say
    /// "reply is empty" rather than "no code block".
    CopyMsgEmpty,
    /// `/copy N`: the requested index is out of range; `count` blocks exist.
    CopyBadIndex {
        count: usize,
    },
    /// `/copy`: the clipboard write failed (no arboard backend -- headless/SSH).
    CopyFailed,
    /// Description for the `/save` slash command -- exports the current
    /// conversation to a local markdown file.
    CmdDescSave,
    /// `/save`: the conversation was written to a file; `path` is the resolved
    /// path (display-only).
    SaveOk {
        path: &'a str,
    },
    /// `/save`: there are no conversation turns to export yet.
    SaveEmpty,
    /// `/save`: the filesystem write failed; `error` carries the underlying
    /// error message.
    SaveIoError {
        error: &'a str,
    },
    /// `/save`: the requested path's parent directory does not exist.
    SaveInvalidPath {
        path: &'a str,
    },
    /// `/save`: the target already exists and is NOT a markdown file -- refused
    /// to overwrite it (likely a typo that would clobber source/config/data).
    SaveRefuseOverwrite {
        path: &'a str,
    },
    /// Hint shown after a code block is auto-copied to clipboard (issue #699).
    CodeBlockCopied,
    /// Description for the `/guide` slash command -- asks rustcode-guide a question.
    CmdDescGuide,
    /// Description for the `/view` slash command -- opens an overlay modal showing file content.
    CmdDescView,
    /// Description for the `/app` slash command -- expose the session to the mobile App via relay.
    CmdDescApp,
    /// Description for the `/sync` slash command -- attach to a live webui session.
    CmdDescSync,
    /// Description for the `/review` slash command -- code review the current changes.
    CmdDescReview,
    /// Description for the `/goal` slash command -- set an autonomous completion goal.
    CmdDescGoal,
    /// Description for the `/proxy` slash command -- switch the outbound proxy mode.
    CmdDescProxy,
    /// Description for the `/todo` slash command -- reprint the current task list.
    CmdDescTodo,
    CmdDescTeam,
    /// Description for the `/schedule` slash command -- list local scheduled tasks.
    CmdDescSchedule,
    /// Description for the `/desktop` slash command.
    CmdDescDesktop,
    // ── /proxy picker ──
    /// Proxy picker row titles (double as the localized mode word).
    ProxyTitleFollowSystem,
    ProxyTitleDefaultProxy,
    ProxyTitleNoProxy,
    /// Proxy picker row descriptions.
    ProxyDescFollowSystem,
    ProxyDescDefaultProxy,
    ProxyDescNoProxy,
    /// Confirmation after a proxy mode switch. `mode` is the localized title.
    ProxyModeLine {
        mode: &'a str,
    },
    /// `default_proxy` summary with a count of pinned env vars.
    ProxyDefaultPinned {
        count: usize,
    },
    /// `default_proxy` summary when no env vars were captured.
    ProxyDefaultEmpty,

    /// `/desktop` -- launching the found app (`name` = app, `path` = its location).
    DesktopOpening {
        name: &'a str,
        path: &'a str,
    },
    /// `/desktop` -- app not found; point the user at the download URL.
    DesktopNotInstalled {
        url: &'a str,
    },
    /// `/desktop` -- the app was found but the OS launch call failed.
    DesktopLaunchFailed {
        path: &'a str,
        err: &'a str,
    },
    /// `/todo` output when no todowrite call exists in the transcript yet.
    TodoNoList,
    /// `/todo` header line printed before the task list.
    TodoListHeader,
    /// `/todo add` used without any task text after it.
    TodoAddUsage,
    /// `/guide` menu header: "[*] RustCode Guide -- type /guide <question>"
    GuideMenuHeader,
    /// `/guide` menu: "Common topics:" section label
    GuideMenuTopics,
    /// `/guide` menu topic: getting started
    GuideMenuGettingStarted,
    /// `/guide` menu topic: switching models
    GuideMenuSwitchModel,
    /// `/guide` menu topic: using MCP
    GuideMenuMcp,
    /// `/guide` menu topic: skills and plugins
    GuideMenuSkills,
    /// `/guide` menu topic: memory feature
    GuideMenuMemory,
    /// `/guide` menu topic: background tasks
    GuideMenuBackground,
    /// `/guide` menu topic: context management
    GuideMenuContext,
    /// `/guide` menu topic: keyboard shortcuts
    GuideMenuKeybindings,
    /// `/guide` menu topic: configuration
    GuideMenuConfig,
    /// /guide menu tip: hint for users to type a question
    GuideMenuTip,
    /// /guide menu: documentation URL
    GuideMenuDocUrl,
    /// `/guide`: ask skill install already in progress, please wait
    CmdGuideInstalling,
    /// `/guide`: ask skill not installed, triggering auto-install
    CmdGuideAutoInstall,
    /// `/guide`: auto-invoke completed, now answering
    CmdGuideAutoInvoke {
        topic: &'a str,
    },
    /// `/guide`: install succeeded but ask skill still not found
    CmdGuideSkillNotFound,
    /// `/guide`: install failed, suggest manual install
    CmdGuideInstallFailed {
        error: &'a str,
    },
    /// `/paste` failed because the clipboard holds no image. Shown
    /// in scrollback as an error line so the user isn't left
    /// wondering whether the command did anything.
    CmdPasteNoImage,
    /// `/paste` on HarmonyOS (ohos): the system clipboard is not
    /// readable at all (arboard has no ohos backend, and the
    /// `ohos-pasteboard` CLI ships only in unreleased 7.0), so "no
    /// image" is misleading -- point the user at the file-path workaround.
    CmdPasteNoImageOhos,

    // ── reasoning effort ──
    /// Rendered when the user tries to set reasoning_effort on a
    /// model that doesn't support it (only DeepSeek V4 / reasoner).
    ReasoningEffortNoEffect,

    // ── config save failed ──
    ConfigSaveFailed {
        error: &'a str,
    },

    // ── OnboardingWizard (multi-step first-run + `/welcome`). Spec:
    //    docs/superpowers/specs/2026-05-11-welcome-wizard-redesign-design.md
    OnboardingStepHeaderWelcome,
    OnboardingStepHeaderLanguage,
    OnboardingStepHeaderSetup,
    OnboardingPanelTitle,
    OnboardingIntroVersionLine {
        v: &'a str,
    },
    OnboardingIntroBullet1,
    OnboardingIntroBullet2,
    OnboardingIntroBullet3,
    /// Neutral-build replacement for `OnboardingIntroBullet3` (no managed service /
    /// free-token pitch); leads with bring-your-own-key instead.
    OnboardingIntroBullet3Neutral,
    OnboardingIntroPressEnter,
    OnboardingIntroCtrlC,
    OnboardingIntroCompactTagline,
    OnboardingLanguageTitleBilingual,
    OnboardingLanguagePrompt,
    OnboardingLanguageOptionAuto,
    OnboardingLanguageOptionEn,
    OnboardingLanguageOptionZhCn,
    OnboardingSetupTitle,
    OnboardingNavHint,
    /// Setup-step nav hint: the setup menu has a variable number of
    /// options (2 in neutral BYO builds, 3 in managed builds), so the
    /// number range must not be hard-coded.
    OnboardingSetupNavHint,
    OnboardingConfirmClear,
    CmdWelcomeDescription,

    /// Vision preprocessor success banner. Shown as a body line right
    /// after a VL turn finishes, in the form
    ///   `[+] VL recognised image, returned N chars`
    /// (English) /
    ///   `[+] VL 识别图片成功，返回 N chars`
    /// (zh-CN). The model key trails as a dim suffix in the renderer
    /// -- kept out of this message so the wrapper styling stays
    /// renderer-side.
    VisionPreprocessSuccess {
        char_count: usize,
    },

    /// VL preprocessing failed -- shown as a warning. `reason` is the underlying
    /// error; the driver restores the images so the user can retry.
    VisionPreprocessFailed {
        reason: &'a str,
    },

    /// TurnComplete separator summary, e.g.
    ///   `[+] Shipped . 3 rounds . 2 tools . 6.8s . 285 tokens`
    /// `done` is a playful completion verb from the locale's pool in
    /// tuix `state.rs` (`DONE_LABELS` for en, `DONE_LABELS_ZH` for zh);
    /// the structural words (`rounds`/`tools`/`tokens`) localise in the
    /// template below. `duration` is a pre-formatted human string (e.g.
    /// "6.8s").
    TurnSummary {
        done: &'a str,
        turn_count: usize,
        tool_call_count: usize,
        duration: &'a str,
        total_tokens: usize,
        /// Cache-hit ratio over the turn's input, if reported. `Some(n)` appends
        /// `. n% cached`; `None` appends nothing.
        cached_pct: Option<u8>,
    },

    /// Turn-end summary when the turn terminated in an error (the red
    /// error line is rendered separately, just above this). Same stats
    /// as `TurnSummary` but with a [x] marker and a neutral "stopped"
    /// label instead of a celebratory verb -- otherwise an errored turn
    /// reads as `[+] Nailed it` right under its own error message.
    TurnSummaryError {
        turn_count: usize,
        tool_call_count: usize,
        duration: &'a str,
        total_tokens: usize,
        /// Short failure cause FOLDED into the separator (`[x] 已中断：<reason> . ...`).
        /// Bound to the always-visible summary because the standalone mid-turn
        /// error line can be clobbered by a real terminal's Streaming->Idle redraw.
        /// `None` on resume replay (the reason is live-only, not persisted).
        reason: Option<&'a str>,
    },

    /// Turn-end summary for a hard local security-policy denial. This is
    /// intentionally distinct from cancellation and provider failure: the
    /// runtime stopped the turn deliberately and preserved a valid transcript.
    TurnSummaryPolicyDenied {
        turn_count: usize,
        tool_call_count: usize,
        duration: &'a str,
        total_tokens: usize,
        /// Optional driver-owned, sanitized reason folded into the terminal line.
        reason: Option<&'a str>,
    },

    // ── Live spinner footer segments ──
    // The spinner composes: `<thinking word>… <effort suffix> <queued suffix>
    // (<elapsed> · ↑ <tokens>)`. Each segment is a separate Msg so the builder
    // in tuix can splice them in fixed order; the thinking/done word pools live
    // in tuix `state.rs` (locale-indexed).
    /// Suffix after the thinking word while a non-default reasoning effort is
    /// active, e.g. ` · thinking with high effort` (en) / ` · 高强度思考` (zh).
    /// `effort` is the caller-localized level word (see tuix `effort_word`).
    SpinnerEffortSuffix {
        effort: &'a str,
    },
    /// Suffix when prompt turns are queued behind the running turn,
    /// e.g. ` · 2 queued` / ` · 2 条排队`.
    SpinnerQueuedSuffix {
        count: usize,
    },
    /// Final parenthesized spinner clock with the live output-token estimate,
    /// e.g. ` (49m44s · ↑ 12.4k tokens)`. `tokens` is pre-formatted via
    /// `fmt_tokens`; "tokens" stays untranslated to match the `/cost` convention.
    SpinnerElapsedTokens {
        elapsed: &'a str,
        tokens: &'a str,
    },
    /// Final parenthesized spinner clock before any output exists (` (12s)`).
    SpinnerElapsedOnly {
        elapsed: &'a str,
    },
    /// Sub-agent fan-out progress label stored in the spinner state,
    /// e.g. `SubAgents 2/6` / `子代理 2/6`.
    SpinnerSubAgents {
        done: usize,
        total: usize,
    },
    /// Spinner label while blocked on an interactive approval answer.
    SpinnerWaitingApproval,

    // ── Live hub / phone remote synchronization errors ──
    /// Forwarding a runtime event to the live hub failed. `error` is pre-formatted.
    LiveSyncEventFailed {
        error: &'a str,
    },
    /// Publishing the post-reload provider to the live hub failed.
    LiveSyncProviderFailed {
        error: &'a str,
    },
    /// Publishing the goal state to the live hub failed.
    LiveSyncGoalFailed {
        error: &'a str,
    },
    /// Forwarding phone-issued command output to the live hub failed.
    LiveSyncRemoteOutputFailed {
        error: &'a str,
    },
    /// Forwarding the phone-command rejection notice to the live hub failed.
    LiveSyncRemoteRejectFailed {
        error: &'a str,
    },
    /// Scrollback echo when a command was issued from the phone.
    LiveRemoteCommandEcho {
        display: &'a str,
    },
    /// Rejection notice sent back to the phone for desktop-only commands.
    LiveRemoteCommandRejected,
    /// A session switch finished without producing a session identity.
    LiveProjectionNoSessionIdentity,
    /// Capability reload returned a session identity that did not match.
    LiveProjectionUnexpectedIdentity,
    /// Updating the live capability snapshot failed. `error` is pre-formatted.
    LiveCapabilitySnapshotFailed {
        error: &'a str,
    },
    /// Decoding a persisted session during projection failed.
    LiveSessionDecodeFailed {
        session_id: &'a str,
        error: &'a str,
    },
    /// The session vanished between runtime switch and projection read.
    LiveSessionDisappeared {
        session_id: &'a str,
    },
    /// Resolving a session by id during projection failed.
    LiveSessionResolveFailed {
        session_id: &'a str,
        error: &'a str,
    },
    /// Updating the live session snapshot failed. `error` is pre-formatted.
    LiveSessionSnapshotFailed {
        error: &'a str,
    },

    // ── Renderer: status badges / agent-group headers ──
    /// Search badge prefix before the quoted query, e.g. ` Search '` / ` 搜索：'`.
    BadgeSearchPrefix,
    /// Search badge count suffix, e.g. ` 3/12 ` (numbers, locale-neutral).
    BadgeSearchCount {
        current: usize,
        total: usize,
    },
    /// History badge, e.g. ` History 2/5 ` / ` 历史 2/5 `.
    BadgeHistory {
        current: usize,
        total: usize,
    },
    /// Agent-group kind words.
    AgentGroupTeamKind,
    AgentGroupSubKind,
    /// Finished agent-group header. `marker` is the bullet glyph.
    AgentGroupFinished {
        marker: &'a str,
        kind: &'a str,
        terminal: usize,
        total: usize,
        failed: usize,
    },
    /// Running agent-group header. `marker` is the bullet glyph.
    AgentGroupRunning {
        marker: &'a str,
        kind: &'a str,
        running: usize,
        total: usize,
    },
    /// Compact sub-task progress counts appended to the tool-call header.
    SubtaskCounts {
        finished: usize,
        total: usize,
        running: usize,
        pending: usize,
    },
    /// Compact footer panel titles (leading space kept for layout).
    SubtaskPanelTeamTitle,
    SubtaskPanelSubTitle,
    /// Fallback activity word when a running sub-task reports no activity text.
    SubtaskActivityAnalyzing,
    /// Folded summary parts in the compact sub-task panel.
    SubtaskSummaryRunning {
        count: usize,
    },
    SubtaskSummaryPending {
        count: usize,
    },
    SubtaskSummaryFailed {
        count: usize,
    },
    SubtaskSummaryStopped {
        count: usize,
    },
    /// Suffix appended to the single expanded pending task identity (` · pending`).
    SubtaskPendingSuffix,
    /// Per-row sub-task state words in the agent-group panel.
    SubtaskStatePending,
    SubtaskStateRunning,
    SubtaskStateQueued,
    SubtaskStateDone,
    SubtaskStateStopped,
    SubtaskStateFailed,
    /// Todo panel header title (trailing space kept for layout) and counts.
    TodoHeaderTitle,
    TodoHeaderCounts {
        completed: usize,
        in_progress: usize,
        open: usize,
    },
    /// Todo panel fold indicator, e.g. `  +3 more…` / `  +3 项更多…`.
    TodoMoreFold {
        hidden: usize,
        ellipsis: &'a str,
    },
    /// Input-box scroll hint, e.g. ` +2 more lines ` / ` +2 行已折叠 `.
    MoreLinesHint {
        count: usize,
    },
    /// Body diff fold line, e.g. `  … +10 more lines` / `  … 还有 10 行`.
    BodyMoreLines {
        ellipsis: &'a str,
        count: usize,
    },
    /// Goal/loop footer row meta: ` · round 3 · 12s` / ` · 第 3 轮 · 12s`.
    RoundMeta {
        round: u32,
        elapsed: &'a str,
    },
    /// Narrow-width fallback without leading separator.
    RoundBare {
        round: u32,
        elapsed: &'a str,
    },
    /// Footer goal-row phase badges (paused / paused-at-cap / satisfied).
    GoalRowPausedBody,
    GoalRowPausedMeta,
    GoalRowPausedAtCapBody,
    GoalRowPausedAtCapMeta {
        round: u32,
    },
    GoalRowSatisfiedBody,
    GoalRowSatisfiedMeta,

    // ── Event-loop status / error lines ──
    /// Slash-command output sync failure (phone relay).
    SlashOutputSyncFailed {
        error: &'a str,
    },
    /// `/context` refresh could not be started.
    RefreshContextStartFailed {
        error: &'a str,
    },
    /// Context-stats refresh event returned an error.
    RefreshContextFailed {
        error: &'a str,
    },
    /// `/sync off` outcomes.
    SyncStoppedSharing,
    SyncNotActive,
    /// `/desktop` (App remote access) stop outcomes.
    AppRemoteStopped,
    AppRemoteNotRunning,
    /// Extra suffix when detach left the TUI synced.
    AppRemoteDetachSuffix {
        error: &'a str,
    },
    /// Background session projection could not be loaded for the live panel.
    BgSessionLoadFailed {
        error: &'a str,
    },
    /// `/mcp tools <server>` output lines.
    McpToolsHeader,
    McpToolsEmpty {
        status: &'a str,
    },
    McpToolsNoServer,
    /// `/team` panel control confirmations / usage.
    TeamPanelShown,
    TeamPanelHidden,
    TeamPanelCleared,
    TeamPanelUsage,
    /// Coding-plan setup internal error (two retry sites).
    InternalError {
        error: &'a str,
    },
    /// OAuth login failure hint.
    LoginFailedHint {
        reason: &'a str,
    },
    /// Mid-turn steer queued behind a running turn.
    SteerQueuedLine {
        prompt: &'a str,
    },
    /// Blank-turn notices (model produced no visible answer).
    EmptyCompletionReasoningOnly,
    EmptyCompletionNoOutput,
    /// Terminal status words for completed task-tool results.
    TaskWordFinished,
    TaskWordCompleted,
    /// Generic undo failure (RewindOutOfRange has its own handler).
    UndoFailed {
        error: &'a str,
    },
    /// Manual compaction failure.
    CompactFailed {
        error: &'a str,
    },
    /// `web_search` result source-domain summary prefix.
    WebSourcesPrefix {
        sources: &'a str,
    },
    /// Phone-relayed goal command failed on the desktop side.
    GoalExecFailed {
        error: &'a str,
    },
    // SessionSaveFailed { error } already exists near the session-management
    // arms -- reused for the phone-relay rename failure site.
    /// Done-label word for a turn that dispatched async team work.
    TurnDoneDispatched,
    /// Background session projection diagnostics (wrapped by BgSessionLoadFailed).
    BgProjectionSessionless,
    BgProjectionLoadFailed {
        bucket: &'a str,
        session_id: &'a str,
        error: &'a str,
    },
    BgProjectionDecodeFailed {
        session_id: &'a str,
        error: &'a str,
    },
    BgProjectionIdentityMismatchRuntime {
        session_id: &'a str,
        catalog: &'a str,
    },
    BgProjectionIdentityMismatchLoaded {
        expected: &'a str,
        loaded: &'a str,
    },

    // ── Code review report scaffolding (rustcode-review crate) ──
    // Model-facing text (persona prompts, verify tasks, `code_review:` error
    // prefixes) intentionally stays English; these arms cover the human-facing
    // report scaffolds and the live activity lines.
    /// Clean single-pass review with no findings.
    ReviewCompleteClean {
        changed_files: usize,
    },
    /// Single-pass review header with a finding count.
    ReviewHeader {
        findings: usize,
        changed_files: usize,
    },
    /// One finding entry in the single-pass report.
    ReviewFindingEntry {
        index: usize,
        priority: &'a str,
        confidence: &'a str,
        location: &'a str,
        title: &'a str,
    },
    /// Finding fix-suggestion line (leading `↳` glyph stays in the template).
    ReviewFixSuggestion {
        suggestion: &'a str,
    },
    /// Overflow line when findings exceed the rendered cap.
    ReviewMoreFindings {
        hidden: usize,
        shown: usize,
    },
    /// Partial-coverage report header (single pass stopped early). The
    /// `[review-incomplete]` ASCII tag is prepended by the caller (not part of
    /// this arm) as the locale-stable signal UI drivers detect.
    ReviewIncompleteHeader {
        stop: &'a str,
        findings: usize,
        changed_files: usize,
    },
    /// Optional error-reason line inside a partial-coverage report.
    ReviewIncompleteReason {
        reason: &'a str,
    },
    /// Deep review where every dimension failed (no reliable coverage).
    ReviewDeepIncomplete {
        total: usize,
        note: &'a str,
    },
    /// Deep review with zero surviving findings.
    ReviewDeepClean {
        changed_files: usize,
        completed: usize,
        total: usize,
        note: &'a str,
    },
    /// Deep review header with finding + dimension counts.
    ReviewDeepHeader {
        findings: usize,
        changed_files: usize,
        completed: usize,
        total: usize,
    },
    /// Deep review dedupe note (appended to the header).
    ReviewDeepDeduped {
        count: usize,
    },
    /// Deep review verify-pass cull note (appended to the header).
    ReviewVerifyDropped {
        count: usize,
    },
    /// Failed-dimension list line in a deep report.
    ReviewFailedDimensions {
        list: &'a str,
    },
    /// One finding entry in the deep report (with contributing-dimension list).
    ReviewDeepFindingEntry {
        index: usize,
        priority: &'a str,
        confidence: &'a str,
        location: &'a str,
        dims: &'a str,
        title: &'a str,
    },
    // Live review activity lines (ephemeral progress, marker-prefixed).
    /// Activity head for the single reviewer.
    ReviewActivityHead,
    /// Activity head for a labeled deep-mode reviewer (dimension id or verify).
    ReviewActivityHeadLabeled {
        label: &'a str,
    },
    /// Running finding count, singular.
    ReviewActivityFindingOne {
        count: u32,
    },
    /// Running finding count, plural.
    ReviewActivityFindingMany {
        count: u32,
    },
    /// Activity tail: reviewer is thinking.
    ReviewActivityThinking,
    /// Activity tail: reviewer is reporting a finding.
    ReviewActivityReporting,
    /// Activity line: diff preparation stage.
    ReviewActivityPreparing,
    /// Activity line: analysis stage with the changed-file count.
    ReviewActivityAnalyzing {
        files: usize,
    },
    /// Deep-mode stage label for the verify pass.
    ReviewStageVerify,

    // ── Agent mode badges / switch line ──
    /// Mode badge words (glyph prefix -- `⏸`/`⏵`/`>>` etc. -- stays in the
    /// renderer so its unicode-downgrade logic is untouched).
    ModeWordPlan,
    ModeWordAcceptEdits,
    ModeWordBuild,
    ModeWordAuto,
    /// Confirmation line after a mode switch, e.g.
    /// `  Switched to plan mode.` / `  已切换到计划模式。`
    ModeSwitchedLine {
        mode: &'a str,
    },

    // ── /goal verdict banners ──
    /// Goal achieved (terminal or per-round satisfaction).
    GoalMetBanner {
        reason: &'a str,
    },
    /// Goal paused at a round/time cap -- it resumes on the next Submit.
    GoalPausedBanner {
        reason: &'a str,
    },
    /// Goal ended without satisfaction (failure / explicit stop).
    GoalStoppedBanner {
        reason: &'a str,
    },

    // ── Parallel sub-agent dispatch ──
    /// Header announcing a sub-agent fan-out, e.g.
    /// `Dispatching 3 sub-agents in parallel...` / `正在并行派发 3 个子代理…`
    ParallelDispatchStart {
        count: usize,
    },
    /// Fallback word in a per-task failure line when the runtime gave no
    /// reason (`✗ path -- 1.2s . failed` / `. 失败`).
    WordFailed,
    /// Aggregate dispatch summary when every task succeeded:
    /// `● ParallelEditFiles . 3/3 ok . 12s wall` /
    /// `● 并行编辑 . 3/3 成功 . 耗时 12s`
    ParallelSummaryOk {
        ok: usize,
        total: usize,
        elapsed: &'a str,
    },
    /// Aggregate dispatch summary with failures:
    /// `● ParallelEditFiles . 2 ok . 1 fail . 12s wall` /
    /// `● 并行编辑 . 2 成功 . 1 失败 . 耗时 12s`
    ParallelSummaryFail {
        ok: usize,
        failed: usize,
        elapsed: &'a str,
    },

    // ── Verbose mode / live-output hints ──
    /// Hint inside the bash in-flight strip: press Ctrl+o to stream output.
    BashInflightCtrlOHint,
    /// Status line after toggling verbose mode on/off. `mute`/`reset` are
    /// ANSI SGR escapes injected by the caller (theme-dependent).
    VerboseOnLine {
        mute: &'a str,
        reset: &'a str,
    },
    VerboseOffLine {
        mute: &'a str,
        reset: &'a str,
    },

    // ── /effort command + slash-menu descriptions ──
    EffortLevelLow,
    EffortLevelMedium,
    EffortLevelHigh,
    EffortLevelXhigh,
    EffortLevelMax,
    /// The `default` pseudo-entry in the /effort menu.
    EffortLevelDefault,
    /// `/effort` usage with the levels THIS endpoint exposes, e.g.
    /// `  Usage: /effort low | medium | high | default\n  Shortcut: Ctrl+T\n`
    EffortUsage {
        levels: &'a str,
    },
    /// `/effort` status: current level followed by the usage block.
    EffortCurrent {
        current: &'a str,
        usage: &'a str,
    },
    /// Status-line value when the endpoint has no effort capability.
    EffortStatusUnsupported,
    /// Status-line value for the `"auto"` sentinel (capability kept, API
    /// picks the level).
    EffortStatusDefault,
    /// Confirmation after setting a level, e.g.
    /// `  o Reasoning effort set to: high\n`
    EffortSet {
        level: &'a str,
    },
    /// Confirmation after resetting to the API default.
    EffortSetDefault,

    // ── OAuth login chrome (/login + /codingplan share these) ──
    /// Header above the QR block when scanning with WeChat is the
    /// expected flow. Includes the leading "  " indent and trailing
    /// "\n\n" paragraph break that the caller used to inline.
    LoginQrHeader,
    /// Separator + URL prelude shown below the QR block when both
    /// QR and URL fallback are available. Leading "\n\n  " and
    /// trailing "\n  " are part of the template.
    LoginUrlAfterQr,
    /// QR + URL both unavailable (Unicode-incapable terminal AND a
    /// platform where URL-based login doesn't work, e.g. OHOS).
    LoginNoQrNoUrl,
    /// URL-only header when QR can't render but URL login works.
    /// Leading "  " indent and trailing "\n  " before the URL.
    LoginUrlOnly,
    /// Footer line: "Press ESC to cancel" with surrounding
    /// blank-line padding.
    LoginCancelHint,

    // ── /context report ──
    CtxUsageHeader,
    CtxUsageNoTurns,
    CtxUsageWaiting,
    CtxProvider,
    CtxCtxName,
    CtxLabelSystemPrompt,
    CtxLabelToolDefs,
    CtxLabelColdZone,
    CtxLabelMessages,
    CtxLabelFree,
    CtxMessagesInWindow {
        n: usize,
    },
    CtxSystemPromptHeader,
    CtxSystemPromptEmpty,
    /// Used in the "used/window tokens (pct)" line below the bar.
    CtxTokensSuffix,

    // ── /compact ──
    CompactNothingShort,
    CompactStarting,
    CompactInterrupted,
    CompactUnavailableDuringSync,
    CompactUnavailableDuringResync,
    LocalRuntimeRestorePending,
    LocalRuntimeRestoreTimedOut,
    CompactNothingNoSavings {
        before: &'a str,
        after: &'a str,
    },
    CompactDropped {
        messages: usize,
        before: &'a str,
        after: &'a str,
    },
    /// Footer spinner label while a compaction's LLM summary runs (slow tier).
    Compacting,
    /// Spinner label variant when the compaction summary has stalled (>20s).
    CompactingSlow,
    /// Scrollback marker for a committed drain+summarize compaction (auto or
    /// manual). `messages` = exact count summarized; `before`/`after` = raw
    /// token-estimate strings (e.g. "48.2K") -- the `~` marker is added by the
    /// i18n format string, not by the caller.
    CompactMarkDrain {
        messages: usize,
        before: &'a str,
        after: &'a str,
    },
    /// Scrollback marker for a committed in-place stub fold (tool results
    /// collapsed, no messages dropped). `saved` = raw token-estimate string;
    /// the `~` marker is added by the i18n format string, not by the caller.
    CompactMarkStub {
        saved: &'a str,
    },
    /// Acknowledgement for a MANUAL `/compact` that committed but only shaved a
    /// negligible amount (a tiny stub fold). Reads as a clean "nothing to do"
    /// so it doesn't look like a "success" the way `CompactMarkStub` does, and
    /// avoids the misleading "conversation is short" wording.
    CompactNegligibleSavings,

    // ── /goal ──
    /// The full `/goal help` usage block (header + Usage + Notes).
    GoalHelp,
    /// `/goal` / `/goal status` while a goal is active. `condition` is the goal
    /// text; `round`/`mins`/`secs` are the live progress counters.
    GoalStatus {
        condition: &'a str,
        round: u32,
        mins: u64,
        secs: u64,
    },
    /// `/goal status` (or bare `/goal`) when no goal is active.
    GoalNoActive,
    /// Confirmation line after `/goal clear` (and its aliases).
    GoalCleared,

    // ── /loop ──
    /// `/loop` / `/loop status` while a loop is active. `label` is the loop
    /// description (e.g. "30s . /foo"), `round`/`mins`/`secs` are counters.
    LoopStatus {
        label: &'a str,
        round: u32,
        mins: u64,
        secs: u64,
    },
    /// `/loop status` (or bare `/loop`) when no loop is active.
    LoopNoActive,
    /// Confirmation line after `/loop stop` (and its aliases).
    LoopCleared,
    /// Mid-loop turn-separator banner: `[*] loop round N . stats`.
    /// `round` is the 1-based round number; `stats` is the pre-formatted
    /// stats string (tools . duration . tokens . cached%).
    LoopRound {
        round: u32,
        stats: &'a str,
    },
    /// Emitted by `handle_loop_decision` when the consecutive-failure limit
    /// is reached and the interval loop auto-stops.
    LoopStopped,
    /// End-of-loop banner emitted by the `LoopUpdate { active: false }` handler
    /// when the loop ends with a non-cancellation reason.
    /// `reason` is the internal English identifier from CodingRuntime's loop controller
    /// (e.g. "completed", "round limit (10)") -- kept English as-is.
    LoopEnded {
        reason: &'a str,
    },
    /// One-line hint shown when a `/loop` is armed: the loop is a live-only
    /// construct and does NOT survive a restart/resume (persistence deferred).
    LoopNoPersistHint,
    /// Description for the `/loop` slash command (shown in `/help`).
    CmdDescLoop,

    /// Surfaced when the user pastes/attaches an image but the active
    /// model can't accept images AND no `vision_preprocessor_provider`
    /// is configured. `model` is the current model identifier.
    ModelNoImageSupport {
        model: &'a str,
    },

    /// Like `ModelNoImageSupport`, but a `vision_preprocessor_provider` IS
    /// configured -- it just doesn't resolve (typo'd / removed name). Names the
    /// offending value so the user fixes the name instead of thinking they
    /// never set it. `model` = current model; `provider` = unresolvable value.
    VisionPreprocessorUnresolvable {
        model: &'a str,
        provider: &'a str,
    },

    // ── --dangerously-skip-permissions / -y ──
    /// Scrollback warning banner when --dangerously-skip-permissions is active
    /// in TUI mode. Includes leading "[!] " and trailing "\n".
    BypassWarningBanner,
    /// Headless-mode stderr warning when --dangerously-skip-permissions is active.
    BypassWarningHeadless,

    // ── admin / root privilege warning ──
    /// TUI scrollback warning when RustCode is running as admin/root.
    /// Includes leading "[!] " and trailing "\n".
    AdminWarningBanner,
    /// Headless-mode stderr warning when running as admin/root.
    AdminWarningHeadless,

    /// Confirmation hint after the first Ctrl+C on an empty buffer.
    /// "  (press Ctrl+C again to exit)\n" -- leading indent + trailing
    /// newline are part of the template.
    CtrlCAgainToExit,

    /// Discovery hint after the first bare Esc on an empty idle buffer.
    /// A second Esc within the window rolls the conversation back a turn.
    /// "  (press Esc again to undo last turn)\n" -- leading indent +
    /// trailing newline are part of the template.
    EscAgainToUndo,

    /// Footer discoverability hint shown while the input starts with `!` -- a
    /// `!<cmd>` line runs a local shell command directly (user-invoked bash).
    BashInputHint,

    /// Footer affordance shown the instant the input is a BARE `!` (no command
    /// yet) -- signals the user has entered `!` shell mode, before `BashInputHint`
    /// ("Enter to run...") takes over once a command is typed.
    ShellModeHint,

    /// Header for the transient list of mid-turn messages waiting for the next
    /// model/tool boundary. Also documents the Esc interrupt-and-send action.
    PendingMessagesTitle,
    /// Runtime termination prevented Esc-held messages from being replayed.
    PendingMessagesNotSent {
        count: usize,
    },

    /// Startup hint shown on terminals where Kitty CSI-u keyboard
    /// disambiguation isn't available, telling the user the
    /// guaranteed-works `\<Enter>` multi-line trick. Multi-line
    /// payload with leading indent + trailing paragraph break.
    HintMultiLineInput,

    // ── /bg (background sessions) ──
    /// Help text for `/bg help`. Multi-line string with leading indent
    /// and trailing newlines baked in.
    BgHelp,
    /// Empty state for `/bg list`.
    BgListEmpty,
    /// Table header for `/bg list`. Trailing newline baked in.
    BgListHeader,
    /// Row format for `/bg list`. `state` is the localised state label,
    /// `age` is the humanised age string, `summary` is the session name.
    BgListRow {
        slot: usize,
        short_id: &'a str,
        state: &'a str,
        age: &'a str,
        summary: &'a str,
    },
    /// Localised label for `RuntimeState::Running`.
    BgStateRunning,
    /// Localised label for `RuntimeState::Idle`.
    BgStateIdle,
    /// Localised label for `RuntimeState::Done`.
    BgStateDone,
    /// Localised label for `RuntimeState::Cancelled`.
    BgStateCancelled,
    /// Localised label for `RuntimeState::Error`.
    BgStateError,
    /// Age string: less than 60 seconds.
    BgAgeNow,
    /// Age string: minutes. `n` is the number of minutes.
    BgAgeMinutes {
        n: u64,
    },
    /// Age string: hours. `n` is the number of hours.
    BgAgeHours {
        n: u64,
    },
    /// Age string: days. `n` is the number of days.
    BgAgeDays {
        n: u64,
    },
    /// Error: too many background slots. `max` is the slot limit.
    BgSlotLimitReached {
        max: usize,
    },
    /// Output after `/bg` sends the current session to background.
    /// `new_id` is the new foreground session short id,
    /// `slot` is the background slot number,
    /// `old_id` is the backgrounded session short id,
    /// `state` is the localised runtime state.
    BgBackgroundCurrent {
        new_id: &'a str,
        slot: usize,
        old_id: &'a str,
        state: &'a str,
    },
    /// Error: invalid slot number. `slot` is the requested slot,
    /// `available` is the number of available slots.
    BgInvalidSlot {
        slot: usize,
        available: usize,
    },
    /// Error: background slot has no runtime client.
    BgNoRuntimeClient,
    /// `/bg` cannot switch the foreground while a provider transition runs.
    BgSwitchProviderTransition,
    /// `/bg` cannot switch the foreground while a runtime request is pending.
    BgSwitchRuntimePending,
    /// `/bg` cannot switch the foreground while live sync is attached.
    BgSwitchLiveSync,
    /// Fallback name for a background task with an empty prompt.
    BgTaskFallbackName,
    /// `/background` task could not be started. `error` is the underlying error.
    BgStartFailed {
        error: &'a str,
    },
    /// Output after `/bg <N>` resumes a background session.
    /// `slot` is the resumed slot, `short_id` is the session short id.
    BgResumed {
        slot: usize,
        short_id: &'a str,
    },
    /// When resuming moves the previous foreground into a background slot.
    /// `slot` is the new background slot number.
    BgPreviousForegroundMoved {
        slot: usize,
    },
    /// Output after `/bg drop <N>`. `slot` is the dropped slot,
    /// `short_id` is the session short id.
    BgDropped {
        slot: usize,
        short_id: &'a str,
    },
    /// Output after `/background <task>` starts a one-shot task.
    /// `slot` is the background slot, `short_id` is the session short id.
    BgTaskStarted {
        slot: usize,
        short_id: &'a str,
    },
    /// Background task timed out. `secs` is the timeout in seconds.
    BgTaskTimedOut {
        secs: u64,
    },
    /// Background task internal error. `error` is the error message.
    BgTaskError {
        error: &'a str,
    },
    /// Background task was cancelled.
    BgTaskCancelled,
    /// Background task finished but produced no summary text.
    BgTaskNoSummary,

    // CLI rustcode --help i18n
    CliAbout,
    CliAboutLogin,
    /// Neutral-build `rustcode login` help text: managed sign-in does not
    /// exist in this build, so the about line must not pitch it; point to the
    /// bring-your-own-key provider config instead. Selected at the call site
    /// via `rustcode_auth::managed_login_available()`.
    CliAboutLoginNeutral,
    CliAboutLogout,
    CliAboutStatus,
    CliAboutUpgrade,
    CliAboutRollback,
    CliAboutMcp,
    CliAboutDaemon,
    CliAboutWebui,
    CliAboutPlugin,
    CliAboutUninstall,
    CliAboutSetup,
    CliAboutHooks,
    CliAboutHooksList,
    CliAboutHooksTest,
    CliAboutHooksPaths,
    CliAboutPluginMarketplace,
    CliAboutPluginInstall,
    CliAboutPluginUninstall,
    CliAboutPluginList,
    CliAboutMarketplaceAdd,
    CliAboutMarketplaceRemove,
    CliAboutMarketplaceUpdate,
    CliAboutMarketplaceList,
    CliAboutMcpAdd,
    CliAboutMcpAddGithubOauth,
    CliAboutMcpLogin,
    CliAboutMcpLogout,
    CliHelpContinue,
    CliHelpProvider,
    CliHelpModel,
    CliHelpLang,
    CliHelpConfig,
    CliHelpDir,
    CliHelpPrompt,
    CliHelpPromptFile,
    CliHelpVerbose,
    CliHelpDev,
    CliHelpDangerouslySkipPermissions,
    CliHelpForce,
    CliHelpPortDaemon,
    CliHelpIdleTimeout,
    CliHelpPortWebui,
    CliHelpHost,
    CliHelpUninstallYes,
    CliHelpUninstallPurge,
    CliHelpUninstallKeepData,
    CliHelpUninstallDryRun,
    CliHelpMcpGlobal,
    CliHelpMcpDir,
    CliHelpMcpName,
    CliHelpHooksTestName,
    CliHelpPluginSpec,
    CliHelpMarketplaceUrl,
    CliHelpMarketplaceName,
    CliHelpMcpCommand,
    /// About for the built-in help subcommand.
    CliAboutHelp,
    /// `rustcode resume` about line.
    CliAboutResume,
    /// `rustcode resume <session>` positional help.
    CliHelpResumeSession,
    /// `rustcode schedule` about line and its nested subcommand abouts.
    CliAboutSchedule,
    CliAboutScheduleAdd,
    CliAboutScheduleList,
    CliAboutScheduleRemove,
    CliAboutScheduleEnable,
    CliAboutScheduleDisable,
    /// Hidden `schedule run` (OS-scheduler entry point); localized for
    /// `--help` parity even though it is not advertised.
    CliAboutScheduleRun,
    CliAboutScheduleSync,
    /// `rustcode schedule ...` argument help texts.
    CliHelpSchedId,
    CliHelpSchedTitle,
    CliHelpSchedPrompt,
    CliHelpSchedCwd,
    CliHelpSchedDaily,
    CliHelpSchedWeekly,
    CliHelpSchedEvery,
    CliHelpSchedHourly,
    CliHelpSchedCron,
    CliHelpSchedMode,
    CliHelpSchedNotify,

    // ── /usage modal ──
    /// Tab label: current rate-limit window.
    UsageTabCurrent,
    /// Tab label: 60-day token/request overview.
    UsageTabOverview,
    /// Tab label: per-model breakdown.
    UsageTabModels,
    /// Title line on the Current tab ("Rate-limit window").
    UsageCurrentTitle,
    /// "Resets in HH:MM:SS". `hms` is the pre-formatted countdown string.
    UsageResetsIn {
        hms: &'a str,
    },
    /// "{hours}-hour rolling window" hint below the reset countdown.
    UsageWindowHours {
        hours: i32,
    },
    /// Shown on Current tab when window data is unavailable.
    UsageWindowUnavailable,
    /// Label: "Favorite model".
    UsageStatFavorite,
    /// Label: "Total tokens".
    UsageStatTotal,
    /// Label: "Requests".
    UsageStatRequests,
    /// Label: "Active days".
    UsageStatActiveDays,
    /// Label: "Most active day".
    UsageStatMostActive,
    /// Label: "Longest streak".
    UsageStatLongestStreak,
    /// Label: "Current streak".
    UsageStatCurrentStreak,
    /// Heat-map legend: "Less" (left side of ramp).
    UsageHeatLess,
    /// Heat-map legend: "More" (right side of ramp).
    UsageHeatMore,
    /// Title line on the Models tab.
    UsageModelsTitle,
    /// Shown when usage data is unavailable (Overview / Models tabs).
    UsageNoData,
    /// Footer navigation hint inside the /usage modal.
    UsageFooterHint,
    /// Shown when the fetch failed and we have an error string.
    UsageFetchFailed {
        error: &'a str,
    },
    /// Plan section title on the Current tab.
    UsagePlanTitle,
    /// Plan status label when active (status == 1).
    UsagePlanActive,
    /// Plan status label when expired (status != 1).
    UsagePlanExpired,
    /// "Claimed {claimed} . Expires {expires}" line.
    UsagePlanClaimedExpires {
        claimed: &'a str,
        expires: &'a str,
    },
    /// "Remaining {remaining}/{total} days" line.
    UsagePlanRemaining {
        remaining: i32,
        total: i32,
    },
    /// Brief confirmation shown after Ctrl+S copy.
    UsageCopied,

    // ── streaming liveness (rustcode-tuix spinner) ──
    /// Spinner hint shown when a streaming response has gone silent past the stall
    /// threshold. A silent stretch is OFTEN legitimate (slow first-byte prefill or
    /// long high-effort reasoning at large context), so the text makes NO judgment
    /// about speed -- labelling it "slow" reads as a malfunction ("is it stuck?")
    /// when it usually isn't. The elapsed timer already conveys duration; this adds
    /// only the one thing not otherwise surfaced mid-stream -- that esc cancels.
    StreamStalled,
    StreamRecoveryRunning {
        attempt: u32,
        max_attempts: u32,
    },
    StreamRecoverySucceeded,
    OutputTruncationRunning {
        attempt: u32,
        max_attempts: u32,
    },
    OutputTruncationHeader,
    OutputTruncationQuestion,
    OutputTruncationContinue,
    OutputTruncationContinueDesc,
    OutputTruncationStop,
    OutputTruncationStopDesc,

    // ── legacy Windows console (conhost) one-shot hint ──
    /// Shown once at startup ONLY on the classic Windows console host
    /// (`TerminalCaps::legacy_conhost`), never on Windows Terminal or any
    /// other terminal. Legacy conhost snaps the viewport back to the bottom
    /// on every write, so the live footer repaint during a running task
    /// makes scrolling up to read history impossible until the task ends.
    /// This is a conhost limitation we don't fix in-app -- the hint tells the
    /// user that scrolling resumes when the task finishes, and that Windows
    /// Terminal has no such limitation.
    ConhostScrollHint,
}
