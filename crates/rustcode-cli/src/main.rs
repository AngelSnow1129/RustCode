// Swap in mimalloc on Windows -- the default HeapAlloc is the biggest single
// contributor to per-keystroke render latency (hundreds of small Line/Span
// clones per frame). No-op on macOS/Linux where the system allocator is fine.
#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Result;
use clap::{ArgGroup, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

mod headless_json;
mod schedule_cmd;
mod schedule_os;
mod vision;
use rustcode::uninstall;

// Redirect RUSTCODE_HOME to a throwaway temp dir before any test in this binary
// runs, so unit tests never persist into the developer's real `~/.rustcode`.
// An inherited shell RUSTCODE_HOME is replaced; individual tests may install
// their own temporary home after this ctor runs.
#[cfg(test)]
#[ctor::ctor]
fn _isolate_rustcode_home() {
    rustcode_kernel::test_support::isolate_home();
}

use rustcode_capabilities::mcp::{
    load_mcp_config, login_mcp_oauth, merge_http_oauth_mcp_server_into_json_file,
    merge_stdio_mcp_server_into_json_file, McpHttpAuthConfig, McpOAuthLoginOptions, McpTokenStore,
    McpTransportConfig,
};
use rustcode_capabilities::provider::{OpenAiCompatConfig, OpenAiCompatProvider};
use rustcode_config::config::Config;

/// Set to `true` at the start of `run_headless` so the panic hook and the
/// top-level error handler can skip TUI cleanup. In headless mode raw mode
/// was never enabled, so calling `disable_raw_mode` would be a wasted ioctl
/// and on Windows can panic if the console handle isn't a real TTY.
static HEADLESS_MODE: AtomicBool = AtomicBool::new(false);

/// Set once the startup synchronous upgrade check has run this launch, so the
/// post-parse detached stager can skip a redundant second `latest.json` fetch --
/// the sync path already checked (and applied, if newer) on this same launch.
static SYNC_UPGRADE_CHECKED: AtomicBool = AtomicBool::new(false);

/// Restore terminal state if (and only if) we ever entered TUI mode.
/// No-op in headless mode -- see [`HEADLESS_MODE`].
///
/// TUI mode (v4.23.2+) runs entirely in the primary screen via the
/// append-only RetainedRenderer -- we never emit `\x1b[?1049h`, so there
/// is no `LeaveAlternateScreen` counterpart to issue here.
///
/// On the GRACEFUL path mouse mode, cursor visibility, autowrap, DECSTBM
/// and the Kitty keyboard protocol are restored by `RetainedRenderer` /
/// `TerminalGuard` Drops. But the release profile sets `panic = "abort"`,
/// so on a crash NO destructor unwinds and none of those Drops run --
/// this hook is the only cleanup that executes. Disabling raw mode alone
/// left the Kitty protocol armed, so the parent shell echoed every
/// post-crash keypress as a literal `[27u` / `[99;5u` CSI-u report. We
/// therefore emit the full panic-safe restore sequence (idempotent on
/// the graceful path) before dropping raw mode.
fn notify_stop_reason(
    reason: rustcode_kernel::event::StopReason,
) -> rustcode_capabilities::notify::NotifyStopReason {
    use rustcode_capabilities::notify::NotifyStopReason as N;
    use rustcode_kernel::event::StopReason as T;
    match reason {
        T::Stopped => N::Natural,
        T::Cancelled => N::Cancelled,
        T::MaxRounds | T::MaxContinuations => N::TurnLimit,
        T::RepeatLoop | T::ToolLoopDetected => N::StepLimit,
        T::ProviderError | T::Timeout | T::PromptRejected | T::RateLimited => N::Error,
        _ => N::Error,
    }
}

fn headless_completion_exit_code(
    completion: &rustcode_coding::TurnCompletion,
    current: i32,
) -> i32 {
    match completion {
        // Snapshot failure is a failure of the completion contract itself;
        // the kernel reason inside it cannot turn the variant into success.
        rustcode_coding::TurnCompletion::SnapshotUnavailable { .. } => current.max(1),
        rustcode_coding::TurnCompletion::Completed { reason, .. } => match reason {
            rustcode_kernel::event::StopReason::Cancelled => 130,
            rustcode_kernel::event::StopReason::Stopped
            | rustcode_kernel::event::StopReason::RateLimited => current,
            _ => current.max(1),
        },
    }
}

fn headless_denial_exit_code(current: i32, had_denial: bool) -> i32 {
    if current == 0 && had_denial {
        2
    } else {
        current
    }
}

fn headless_completion_notify_reason(
    completion: &rustcode_coding::TurnCompletion,
) -> rustcode_capabilities::notify::NotifyStopReason {
    match completion {
        rustcode_coding::TurnCompletion::Completed { reason, .. } => notify_stop_reason(*reason),
        rustcode_coding::TurnCompletion::SnapshotUnavailable { .. } => {
            rustcode_capabilities::notify::NotifyStopReason::Error
        }
    }
}

fn restore_terminal_if_tui() {
    if HEADLESS_MODE.load(Ordering::Relaxed) {
        return;
    }
    rustcode_tuix::panic_restore_terminal();
    let _ = crossterm::terminal::disable_raw_mode();
}

/// Resolve the working directory at startup. **Always** uses the current
/// working directory unless the user explicitly passed `-C / --dir`.
///
/// We deliberately do **not** read `~/.rustcode/recent_dirs.txt` (or any other
/// "remembered" path). The previous implementation silently substituted the
/// first entry of recent_dirs for the user's cwd, which made commands like
/// `rustcode -p "describe this project"` operate on whatever directory the
/// TUI happened to visit last -- a violation of least surprise. recent_dirs
/// remains a TUI picker convenience only; it must never override cwd.
fn resolve_working_dir(cli_dir: Option<PathBuf>) -> PathBuf {
    if let Some(d) = cli_dir {
        std::fs::canonicalize(&d).unwrap_or(d)
    } else {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    }
}

/// The on-exit "how to resume this session" hint, mirroring codex's
/// `To continue this session, run ...`. Headless shows the `-p ... --resume <id>`
/// form (what continues a pipe run); the TUI shows the `resume <id>` subcommand.
/// Wording comes from the i18n layer (resolved locale at exit time).
fn resume_hint_line(session_id: &str, headless: bool) -> String {
    let cmd = if headless {
        format!("{BIN_NAME} -p \"...\" --resume {session_id}")
    } else {
        format!("{BIN_NAME} resume {session_id}")
    };
    rustcode_config::i18n::t(rustcode_config::i18n::Msg::CliResumeHint { cmd: &cmd }).into_owned()
}

/// What session to resume at launch, unified across `--continue`, `--resume`,
/// and the `resume` subcommand so one code path (see the launch flow) drives it.
enum ResumeSelector {
    /// The most recent session with messages (`--continue` / bare `resume`).
    Latest,
    /// An explicit id or name (`--resume <x>` / `resume <x>`).
    Specific(String),
}

/// Resolve a `--resume`/`resume <selector>` value to a concrete session id within
/// a project's scanned catalog: an exact `id` match wins; otherwise the
/// most-recently-updated session whose `name` equals the selector. `None` when
/// nothing matches. Pure over the catalog so it is unit-tested without the store.
fn resolve_in_catalog(
    catalog: &[rustcode_capabilities::session::CatalogEntry],
    selector: &str,
) -> Option<String> {
    if let Some(entry) = catalog.iter().find(|e| e.id == selector) {
        return Some(entry.id.clone());
    }
    catalog
        .iter()
        .filter(|e| e.name == selector)
        .max_by_key(|e| e.updated_at_ms)
        .map(|e| e.id.clone())
}

/// Truncate a string to at most `max_chars` *characters* (not bytes), replacing
/// any newlines with spaces and appending "..." when truncated.
///
/// Used for headless-mode log lines on stderr. **Counts characters, not bytes**,
/// so multi-byte UTF-8 (e.g. CJK) is safe -- `&s[..N]` would panic when N falls
/// inside a multi-byte char.
fn truncate_log_line(s: &str, max_chars: usize) -> String {
    let single_line: String = s.chars().map(|c| if c == '\n' { ' ' } else { c }).collect();
    if single_line.chars().count() > max_chars {
        let head: String = single_line.chars().take(max_chars).collect();
        format!("{}...", head)
    } else {
        single_line
    }
}

/// Append a streaming reasoning/thinking `chunk` to `out`, maintaining a
/// single-line `[thinking] ...` representation across many tiny deltas.
///
/// `open` tracks whether a `[thinking]` line is currently open (i.e. has a
/// prefix written but no trailing newline). The first chunk gets a fresh
/// `[thinking] ` prefix; subsequent chunks append directly. Embedded newlines
/// inside a chunk are preserved, with each non-empty new line getting its own
/// `[thinking] ` prefix so multi-line thinking stays readable.
///
/// Pulled out of `run_headless` so it can be unit-tested without spinning up
/// the agent loop. Regression target: the old per-chunk `eprintln!` produced
/// "one word per line" output for streaming reasoning models.
fn format_thinking_chunk(out: &mut String, open: &mut bool, chunk: &str) {
    if chunk.is_empty() {
        return;
    }
    if !*open {
        out.push_str("[thinking] ");
        *open = true;
    }
    let mut parts = chunk.split('\n');
    if let Some(first) = parts.next() {
        out.push_str(first);
    }
    for part in parts {
        out.push('\n');
        *open = false;
        if !part.is_empty() {
            out.push_str("[thinking] ");
            out.push_str(part);
            *open = true;
        }
    }
}

fn format_verbose_tool_chunk(chunk: &str) -> std::borrow::Cow<'_, str> {
    match chunk.strip_prefix('\u{1e}') {
        Some(progress) => std::borrow::Cow::Owned(format!("[progress] {}\n", progress.trim_end())),
        None => std::borrow::Cow::Borrowed(chunk),
    }
}

/// Close any in-flight `[thinking]` line by writing a newline if one is open.
/// Mirrors the inline `close_thinking_line` used inside `run_headless`, but
/// writes to a buffer so it can be unit-tested.
fn close_thinking_chunk(out: &mut String, open: &mut bool) {
    if *open {
        out.push('\n');
        *open = false;
    }
}

/// True if `--dev` is present in argv. Used to skip every auto-update
/// path (pre-parse `apply_pending_upgrade` and the post-parse detached
/// stager). Scanned manually because one of those paths runs before clap
/// touches argv. The flag is also declared on
/// `Cli` so `clap::Parser` accepts it without erroring after the early
/// scan.
fn is_dev_mode() -> bool {
    std::env::args().skip(1).any(|a| a == "--dev")
}

/// True when the currently-running binary's filename ends in `.bak`.
/// `self_update::replace_binary` renames the previous version to
/// `rustcode.bak` (or `rustcode.exe.bak`) during an upgrade so the user
/// can roll back. Running that backup must NOT auto-upgrade -- otherwise
/// rolling back is impossible: any launch of `.bak` would just overwrite
/// itself with the latest version again.
///
/// Defensive: if we can't read `current_exe()` for any reason, assume
/// we're the live binary (not backup) so auto-upgrade still works for
/// the common case.
fn is_running_as_backup() -> bool {
    std::env::current_exe()
        .ok()
        .as_deref()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map(|n| n.ends_with(".bak"))
        .unwrap_or(false)
}

/// Scan argv by hand to extract the value of --lang <VALUE> or --lang=VALUE.
/// This runs BEFORE clap parses the arguments, so that the i18n locale
/// can be set in time for clap to render localised --help text.
fn scan_argv_for_lang() -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1; // skip program name
    while i < args.len() {
        if args[i] == "--lang" && i + 1 < args.len() {
            return Some(args[i + 1].clone());
        }
        if let Some(val) = args[i].strip_prefix("--lang=") {
            return Some(val.to_string());
        }
        i += 1;
    }
    None
}

/// Lightweight pre-parse of the default config path only, BEFORE clap renders
/// `--help` and BEFORE the authoritative `Config` load in `run()`. Resolves
/// the three fields that clap `--help` localisation needs: `language` (for
/// locale) and `brand_name` / `oauth_provider_name` (for `{brand}` / `{oauth}`
/// placeholder substitution in help text).
///
/// Single read + parse of the default config file -- not three independent
/// scans. Env overrides (`RUSTCODE_BRAND_NAME` / `RUSTCODE_OAUTH_PROVIDER_NAME`)
/// are honoured so a `--help` launched under those env vars renders the
/// env-chosen brand, matching the post-load behaviour. Never an error path:
/// any read/parse failure falls back to defaults, matching the per-field
/// helpers it replaces.
struct PreScanConfig {
    language: Option<rustcode_tuix::i18n::Locale>,
    brand_name: String,
    oauth_provider_name: String,
}

fn scan_config_pre() -> PreScanConfig {
    let ui_default = rustcode_config::config::UiConfig::default();
    let mut result = PreScanConfig {
        language: None,
        brand_name: ui_default.brand_name,
        oauth_provider_name: ui_default.oauth_provider_name,
    };
    let path = rustcode_config::config::Config::default_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(cfg) = toml::from_str::<rustcode_config::config::Config>(&content) {
                result.language = cfg.language;
                result.brand_name = cfg.ui.brand_name;
                result.oauth_provider_name = cfg.ui.oauth_provider_name;
            }
        }
    }
    // Env overrides take precedence, matching apply_env_overrides in the full load.
    if let Ok(v) = std::env::var("RUSTCODE_BRAND_NAME") {
        if !v.trim().is_empty() {
            result.brand_name = v;
        }
    }
    if let Ok(v) = std::env::var("RUSTCODE_OAUTH_PROVIDER_NAME") {
        if !v.trim().is_empty() {
            result.oauth_provider_name = v;
        }
    }
    result
}

/// Build the top-level clap Command with i18n-localised about and help text.
/// This replaces the default Cli::parse() flow so that --help output
/// respects the current locale (set by scan_argv_for_lang above).
fn build_i18n_command() -> clap::Command {
    use rustcode_tuix::i18n::{t, Msg};

    let cmd = Cli::command();

    // Mutate the top-level about
    let cmd = cmd.about(t(Msg::CliAbout).into_owned());

    // Mutate top-level argument help texts
    let cmd = cmd
        .mut_arg("continue_last", |a| {
            a.help(t(Msg::CliHelpContinue).into_owned())
        })
        .mut_arg("provider", |a| a.help(t(Msg::CliHelpProvider).into_owned()))
        .mut_arg("model", |a| a.help(t(Msg::CliHelpModel).into_owned()))
        .mut_arg("lang", |a| a.help(t(Msg::CliHelpLang).into_owned()))
        .mut_arg("config", |a| a.help(t(Msg::CliHelpConfig).into_owned()))
        .mut_arg("dir", |a| a.help(t(Msg::CliHelpDir).into_owned()))
        .mut_arg("prompt", |a| a.help(t(Msg::CliHelpPrompt).into_owned()))
        .mut_arg("prompt_file", |a| {
            a.help(t(Msg::CliHelpPromptFile).into_owned())
        })
        .mut_arg("verbose", |a| a.help(t(Msg::CliHelpVerbose).into_owned()))
        .mut_arg("dev", |a| a.help(t(Msg::CliHelpDev).into_owned()))
        .mut_arg("dangerously_skip_permissions", |a| {
            a.help(t(Msg::CliHelpDangerouslySkipPermissions).into_owned())
        })
        .mut_arg("permission_mode", |a| {
            a.help(t(Msg::CliHelpPermissionMode).into_owned())
        });

    // Mutate subcommand about texts

    // There is no managed sign-in service and therefore no `login`/`logout`
    // subcommand at all: credentials come from `[providers.*]` in
    // ~/.rustcode/config.toml (or `--provider`), so nothing here may advertise
    // a sign-in flow. `status` is localized to say exactly that.
    cmd.mut_subcommand("status", |s| s.about(t(Msg::CliAboutStatus).into_owned()))
        .mut_subcommand("upgrade", |s| {
            s.about(t(Msg::CliAboutUpgrade).into_owned())
                .mut_arg("force", |a| a.help(t(Msg::CliHelpForce).into_owned()))
        })
        .mut_subcommand("rollback", |s| {
            s.about(t(Msg::CliAboutRollback).into_owned())
        })
        .mut_subcommand("resume", |s| {
            s.about(t(Msg::CliAboutResume).into_owned())
                .mut_arg("session", |a| {
                    a.help(t(Msg::CliHelpResumeSession).into_owned())
                })
        })
        .mut_subcommand("mcp", |s| {
            s.about(t(Msg::CliAboutMcp).into_owned())
                .mut_subcommand("add", |s| {
                    s.about(t(Msg::CliAboutMcpAdd).into_owned())
                        .mut_arg("name", |a| a.help(t(Msg::CliHelpMcpName).into_owned()))
                        .mut_arg("command", |a| {
                            a.help(t(Msg::CliHelpMcpCommand).into_owned())
                        })
                        .mut_arg("global", |a| a.help(t(Msg::CliHelpMcpGlobal).into_owned()))
                        .mut_arg("dir", |a| a.help(t(Msg::CliHelpMcpDir).into_owned()))
                })
                .mut_subcommand("add-oauth", |s| {
                    s.about(t(Msg::CliAboutMcpAddOauth).into_owned())
                        .mut_arg("url", |a| a.help(t(Msg::CliHelpMcpUrl).into_owned()))
                        .mut_arg("name", |a| a.help(t(Msg::CliHelpMcpName).into_owned()))
                        .mut_arg("global", |a| a.help(t(Msg::CliHelpMcpGlobal).into_owned()))
                        .mut_arg("dir", |a| a.help(t(Msg::CliHelpMcpDir).into_owned()))
                })
                .mut_subcommand("add-github-oauth", |s| {
                    // Retained for backward compatibility but no longer advertised:
                    // the provider-neutral `add-oauth <url>` covers any OAuth MCP server.
                    s.about(t(Msg::CliAboutMcpAddGithubOauth).into_owned())
                        .hide(true)
                        .mut_arg("name", |a| a.help(t(Msg::CliHelpMcpName).into_owned()))
                        .mut_arg("global", |a| a.help(t(Msg::CliHelpMcpGlobal).into_owned()))
                        .mut_arg("dir", |a| a.help(t(Msg::CliHelpMcpDir).into_owned()))
                })
                .mut_subcommand("login", |s| {
                    s.about(t(Msg::CliAboutMcpLogin).into_owned())
                        .mut_arg("name", |a| a.help(t(Msg::CliHelpMcpName).into_owned()))
                        .mut_arg("provider", |a| {
                            a.help(t(Msg::CliHelpMcpProvider).into_owned())
                        })
                        .mut_arg("client_id", |a| {
                            a.help(t(Msg::CliHelpMcpClientId).into_owned())
                        })
                })
                .mut_subcommand("logout", |s| {
                    s.about(t(Msg::CliAboutMcpLogout).into_owned())
                        .mut_arg("name", |a| a.help(t(Msg::CliHelpMcpName).into_owned()))
                })
        })
        .mut_subcommand("daemon", |s| {
            s.about(t(Msg::CliAboutDaemon).into_owned())
                .mut_arg("port", |a| a.help(t(Msg::CliHelpPortDaemon).into_owned()))
                .mut_arg("host", |a| a.help(t(Msg::CliHelpHost).into_owned()))
                .mut_arg("idle_timeout", |a| {
                    a.help(t(Msg::CliHelpIdleTimeout).into_owned())
                })
        })
        .mut_subcommand("webui", |s| {
            s.about(t(Msg::CliAboutWebui).into_owned())
                .mut_arg("port", |a| a.help(t(Msg::CliHelpPortWebui).into_owned()))
                .mut_arg("host", |a| a.help(t(Msg::CliHelpHost).into_owned()))
        })
        .mut_subcommand("plugin", |s| {
            s.about(t(Msg::CliAboutPlugin).into_owned())
                .mut_subcommand("marketplace", |s| {
                    s.about(t(Msg::CliAboutPluginMarketplace).into_owned())
                        .mut_subcommand("add", |s| {
                            s.about(t(Msg::CliAboutMarketplaceAdd).into_owned())
                                .mut_arg("url", |a| {
                                    a.help(t(Msg::CliHelpMarketplaceUrl).into_owned())
                                })
                        })
                        .mut_subcommand("remove", |s| {
                            s.about(t(Msg::CliAboutMarketplaceRemove).into_owned())
                                .mut_arg("name", |a| {
                                    a.help(t(Msg::CliHelpMarketplaceName).into_owned())
                                })
                        })
                        .mut_subcommand("update", |s| {
                            s.about(t(Msg::CliAboutMarketplaceUpdate).into_owned())
                                .mut_arg("name", |a| {
                                    a.help(t(Msg::CliHelpMarketplaceName).into_owned())
                                })
                        })
                        .mut_subcommand("list", |s| {
                            s.about(t(Msg::CliAboutMarketplaceList).into_owned())
                        })
                })
                .mut_subcommand("install", |s| {
                    s.about(t(Msg::CliAboutPluginInstall).into_owned())
                        .mut_arg("spec", |a| a.help(t(Msg::CliHelpPluginSpec).into_owned()))
                })
                .mut_subcommand("uninstall", |s| {
                    s.about(t(Msg::CliAboutPluginUninstall).into_owned())
                        .mut_arg("spec", |a| a.help(t(Msg::CliHelpPluginSpec).into_owned()))
                })
                .mut_subcommand("list", |s| s.about(t(Msg::CliAboutPluginList).into_owned()))
        })
        .mut_subcommand("uninstall", |s| {
            s.about(t(Msg::CliAboutUninstall).into_owned())
                .mut_arg("yes", |a| a.help(t(Msg::CliHelpUninstallYes).into_owned()))
                .mut_arg("purge", |a| {
                    a.help(t(Msg::CliHelpUninstallPurge).into_owned())
                })
                .mut_arg("keep_data", |a| {
                    a.help(t(Msg::CliHelpUninstallKeepData).into_owned())
                })
                .mut_arg("dry_run", |a| {
                    a.help(t(Msg::CliHelpUninstallDryRun).into_owned())
                })
        })
        .mut_subcommand("setup", |s| s.about(t(Msg::CliAboutSetup).into_owned()))
        .mut_subcommand("wiki", |s| {
            s.about(t(Msg::CliAboutWiki).into_owned())
                .mut_arg("path", |a| a.help("Project root to analyze (default: current directory)"))
                .mut_arg("sync", |a| a.help("Incremental sync: only update changed pages"))
                .mut_arg("watch", |a| a.help("Watch for changes and re-sync every --interval seconds"))
                .mut_arg("llm", |a| a.help("Enrich module pages with natural-language summaries via the configured LLM"))
                .mut_arg("force", |a| a.help("Force full regeneration"))
                .mut_arg("out_dir", |a| a.help("Output directory (default: <root>/.rustcode/wiki)"))
                .mut_arg("title", |a| a.help("Wiki title (default: project name)"))
                .mut_arg("exclude", |a| a.help("Extra directories to exclude (repeatable)"))
                .mut_arg("interval", |a| a.help("Watch interval in seconds (with --watch)"))
        })
        .mut_subcommand("completion", |s| {
            s.about(t(Msg::CliAboutCompletion).into_owned())
                .mut_arg("shell", |a| {
                    a.help(t(Msg::CliHelpCompletionShell).into_owned())
                })
        })
        .mut_subcommand("ide", |s| {
            s.about(t(Msg::CliAboutIde).into_owned())
                .mut_subcommand("list", |s| {
                    s.about(t(Msg::CliAboutIdeList).into_owned())
                })
                .mut_subcommand("install", |s| {
                    s.about(t(Msg::CliAboutIdeInstall).into_owned())
                        .mut_arg("ide", |a| {
                            a.help(t(Msg::CliHelpIdeInstallIde).into_owned())
                        })
                        .mut_arg("all", |a| {
                            a.help(t(Msg::CliHelpIdeInstallAll).into_owned())
                        })
                })
        })
        .mut_subcommand("hooks", |s| {
            s.about(t(Msg::CliAboutHooks).into_owned())
                .mut_subcommand("list", |s| s.about(t(Msg::CliAboutHooksList).into_owned()))
                .mut_subcommand("test", |s| {
                    s.about(t(Msg::CliAboutHooksTest).into_owned())
                        .mut_arg("name", |a| {
                            a.help(t(Msg::CliHelpHooksTestName).into_owned())
                        })
                })
                .mut_subcommand("paths", |s| {
                    s.about(t(Msg::CliAboutHooksPaths).into_owned())
                })
        })
        .mut_subcommand("schedule", |s| {
            s.about(t(Msg::CliAboutSchedule).into_owned())
                .mut_subcommand("add", |s| {
                    s.about(t(Msg::CliAboutScheduleAdd).into_owned())
                        .mut_arg("title", |a| a.help(t(Msg::CliHelpSchedTitle).into_owned()))
                        .mut_arg("prompt", |a| {
                            a.help(t(Msg::CliHelpSchedPrompt).into_owned())
                        })
                        .mut_arg("cwd", |a| a.help(t(Msg::CliHelpSchedCwd).into_owned()))
                        .mut_arg("daily", |a| a.help(t(Msg::CliHelpSchedDaily).into_owned()))
                        .mut_arg("weekly", |a| {
                            a.help(t(Msg::CliHelpSchedWeekly).into_owned())
                        })
                        .mut_arg("every", |a| a.help(t(Msg::CliHelpSchedEvery).into_owned()))
                        .mut_arg("hourly", |a| {
                            a.help(t(Msg::CliHelpSchedHourly).into_owned())
                        })
                        .mut_arg("cron", |a| a.help(t(Msg::CliHelpSchedCron).into_owned()))
                        .mut_arg("mode", |a| a.help(t(Msg::CliHelpSchedMode).into_owned()))
                        .mut_arg("notify", |a| {
                            a.help(t(Msg::CliHelpSchedNotify).into_owned())
                        })
                })
                .mut_subcommand("list", |s| {
                    s.about(t(Msg::CliAboutScheduleList).into_owned())
                })
                .mut_subcommand("remove", |s| {
                    s.about(t(Msg::CliAboutScheduleRemove).into_owned())
                        .mut_arg("id", |a| a.help(t(Msg::CliHelpSchedId).into_owned()))
                })
                .mut_subcommand("enable", |s| {
                    s.about(t(Msg::CliAboutScheduleEnable).into_owned())
                        .mut_arg("id", |a| a.help(t(Msg::CliHelpSchedId).into_owned()))
                })
                .mut_subcommand("disable", |s| {
                    s.about(t(Msg::CliAboutScheduleDisable).into_owned())
                        .mut_arg("id", |a| a.help(t(Msg::CliHelpSchedId).into_owned()))
                })
                .mut_subcommand("run", |s| {
                    s.about(t(Msg::CliAboutScheduleRun).into_owned())
                        .mut_arg("id", |a| a.help(t(Msg::CliHelpSchedId).into_owned()))
                })
                .mut_subcommand("sync", |s| {
                    s.about(t(Msg::CliAboutScheduleSync).into_owned())
                })
        })
    // NOTE: clap only instantiates the built-in `help` subcommand during
    // `build()` (i.e. at parse time), so `mut_subcommand("help", ..)` here
    // panics with "Command `help` is undefined". Its about text stays the
    // clap default; `CliAboutHelp` remains for callers that render help text.
}

/// Body of the detached upgrade-prep worker. One call to
/// `prepare_deferred_upgrade` (which fetches the manifest, downloads the
/// next version's binary if newer, verifies sha256, and writes
/// `pending.json`). On success the next parent-rustcode start will pick
/// up `pending.json` and apply. Silent: stdout/stderr are already /dev/null
/// (see `spawn_detached_upgrade_prep`), so any output would be discarded.
async fn run_prepare_upgrade_worker() -> i32 {
    let current = format!("v{}", env!("CARGO_PKG_VERSION"));
    // UpgradeEvent stream is per-byte progress; we don't surface it here
    // (parent is gone), so drain to /dev/null.
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<rustcode_updater::UpgradeEvent>();
    tokio::spawn(async move { while rx.recv().await.is_some() {} });

    match rustcode_updater::prepare_deferred_upgrade(&current, tx).await {
        Ok(_) => 0,
        Err(_) => 1,
    }
}

/// Whether the startup-time SYNCHRONOUS upgrade path should fire. Restored (with the
/// detached stager kept alongside) after 31daa6ee removed it: the everyday startup cost
/// this guards is only a small `latest.json` fetch, so a fresh release is applied on THIS launch
/// instead of requiring two restarts. Returns false for `.bak`, dev, `RUSTCODE_PLAIN`,
/// headless `-p`, subcommands, `auto_update = false`, or offline mode.
fn should_try_sync_upgrade() -> bool {
    if is_running_as_backup() {
        return false;
    }
    if is_dev_mode() {
        return false;
    }
    // Package-managed (distro-pm / HarmonyBrew) builds: upgrades belong to the package
    // manager. Match the detached stager's gate so we don't spin up the check for a
    // guaranteed no-op (`prepare_deferred_upgrade` early-returns on pm builds anyway).
    if rustcode_updater::is_package_managed() {
        return false;
    }

    // PlainRenderer 模式（RUSTCODE_PLAIN=1）：跳过同步自更新检查。
    // 自更新用 eprintln! 直接写 stderr，和 PlainRenderer 的 stdout
    // 流式输出交错，破坏启动体验。后台异步自更新不受影响，用户仍可
    // 手动 /upgrade。
    if std::env::var("RUSTCODE_PLAIN")
        .ok()
        .filter(|v| !v.is_empty())
        .is_some()
    {
        return false;
    }

    let args: Vec<String> = std::env::args().collect();
    let any = |needle: &[&str]| {
        args.iter().skip(1).any(|a| {
            needle
                .iter()
                .any(|n| a == n || a.starts_with(&format!("{}=", n)))
        })
    };

    if any(&["-p", "--prompt", "--prompt-file"]) {
        return false;
    }
    if args.iter().skip(1).any(|a| {
        matches!(
            a.as_str(),
            "status"
                | "upgrade"
                | "rollback"
                | "uninstall"
                | "mcp"
                | "completion"
                | "--version"
                | "-V"
                | "--help"
                | "-h"
        )
    }) {
        return false;
    }

    // Load config once to honor both `auto_update = false` and `offline_mode`.
    // Runs pre-seed, so offline is resolved directly rather than via the process
    // verdict. Env wins over config; only forced On skips. Failure to load = assume
    // defaults (auto_update true, offline Off) -- fresh installs benefit.
    let path = rustcode_config::config::Config::default_path();
    let offline_mode = if path.exists() {
        if let Ok(cfg) = rustcode_config::config::Config::load(&path) {
            if !cfg.auto_update {
                return false;
            }
            cfg.offline_mode
        } else {
            rustcode_config::config::offline::OfflineMode::Off
        }
    } else {
        rustcode_config::config::offline::OfflineMode::Off
    };
    // Offline (env wins over config; only forced On skips) disables binary self-update,
    // same as auto_update=false. Works even with no config file (e.g. air-gapped container).
    if rustcode_config::config::offline::offline_resolved(
        offline_mode,
        std::env::var(rustcode_config::config::offline::RUSTCODE_OFFLINE_ENV)
            .ok()
            .as_deref(),
    ) {
        return false;
    }

    true
}

/// Synchronous startup upgrade: fetch the manifest, and if a newer version is out,
/// stage + apply it NOW and re-exec, so the user gets the new binary on THIS launch.
/// Bounded by a 120s timeout; on timeout/error it just continues (the detached stager
/// and `/upgrade` remain as fallbacks). Restored verbatim from the pre-31daa6ee path.
async fn sync_stage_and_apply_if_newer() {
    use rustcode_config::i18n::{t, Msg};
    use rustcode_updater::{self as self_update, UpgradeEvent};

    let current = format!("v{}", env!("CARGO_PKG_VERSION"));
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<UpgradeEvent>();

    // Progress consumer: renders ManifestFetched / Downloading / Verifying
    // as a single-line updating status on stderr. Percent-debounced so a
    // 15 MB download at 64 KiB chunks doesn't flood the terminal.
    let progress = tokio::spawn(async move {
        use std::io::Write;
        let mut last_pct: i32 = -1;
        while let Some(ev) = rx.recv().await {
            match ev {
                UpgradeEvent::ManifestFetched { version } => {
                    eprintln!("{}", t(Msg::CliUpgradeAvailable { version: &version }));
                }
                UpgradeEvent::Downloading { bytes, total } => {
                    let pct = ((bytes * 100).checked_div(total).unwrap_or(0)) as i32;
                    if pct != last_pct {
                        let mb = format!("{:.1}", bytes as f64 / 1_048_576.0);
                        let total_mb = format!("{:.1}", total as f64 / 1_048_576.0);
                        eprint!(
                            "{}",
                            t(Msg::CliUpgradeDownloading {
                                pct,
                                mb: &mb,
                                total_mb: &total_mb
                            })
                        );
                        let _ = std::io::stderr().flush();
                        last_pct = pct;
                    }
                }
                UpgradeEvent::Verifying => {
                    eprintln!("{}", t(Msg::CliUpgradeVerifying));
                }
                _ => {}
            }
        }
    });

    let outcome = tokio::time::timeout(
        std::time::Duration::from_secs(120),
        self_update::prepare_deferred_upgrade(&current, tx),
    )
    .await;

    // Wait briefly for the progress consumer to drain -- it closes when
    // the sender drops at the end of prepare_deferred_upgrade.
    let _ = progress.await;

    match outcome {
        Ok(Ok(Some(_staged))) => {
            // Staged successfully. Apply right now so the user gets the new
            // binary on this same invocation.
            match self_update::apply_pending_upgrade() {
                Ok(Some(applied)) => {
                    eprintln!(
                        "{}",
                        t(Msg::CliUpgradeApplying {
                            version: &applied.version
                        })
                    );
                    // Save the CURRENT version (before upgrade) so TUI can show "Upgraded old -> new"
                    std::env::set_var(UPGRADED_FROM_ENV, &current);
                    match self_update::re_exec_self(Some(&applied.exe)) {
                        Ok(_infallible) => unreachable!("re_exec_self returned Ok"),
                        Err(e) => {
                            eprintln!(
                                "{}",
                                t(Msg::CliUpgradeReexecFailed {
                                    error: &e.to_string()
                                })
                            );
                            std::env::remove_var(UPGRADED_FROM_ENV);
                        }
                    }
                }
                _ => {
                    // Stage succeeded but apply didn't -- weird, just continue.
                }
            }
        }
        Ok(Ok(None)) => {
            // Already latest, no-op.
        }
        Ok(Err(_)) | Err(_) => {
            // Network error or 120 s timeout. Don't spam the user --
            // `/upgrade` will surface the real error if they ask.
            eprintln!("{}", t(Msg::CliUpgradeCheckFailed));
        }
    }
}

/// Spawn a detached copy of this binary that runs the upgrade-prep worker
/// and exits. "Detached" means:
///   * New session on Unix (`setsid`) -- parent's Ctrl+C goes to parent's
///     foreground process group only; the child is in its own and ignores it.
///   * `CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW` on Windows, same idea
///   * stdin/stdout/stderr -> /dev/null so the child can't scribble over the
///     parent's terminal and has no reason to stay attached to it.
///
/// Does NOT wait for the child (we intentionally don't -- that would recreate
/// the cancel-on-exit problem we're trying to solve). If spawning fails we
/// just drop the error; auto-upgrade is best-effort.
fn spawn_detached_upgrade_prep() {
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => return,
    };

    let mut cmd = std::process::Command::new(&exe);
    cmd.env(INTERNAL_PREPARE_UPGRADE_ENV, "1")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.pre_exec(|| {
                // SAFETY(pre_exec): runs in the forked child before exec --
                // async-signal-safe libc ONLY. No allocation, locks, panics, or
                // non-reentrant calls, or the child can deadlock. libc::setsid() is safe.
                // Detach from parent's controlling terminal / process group.
                // Return value ignored -- setsid only fails when caller is
                // already a process group leader (not our case post-fork).
                libc::setsid();
                Ok(())
            });
        }
    }

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW);
    }

    let _ = cmd.spawn();
}

const VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("RUSTCODE_BUILD_ID"),
    env!("RUSTCODE_BUILD_DIRTY"),
    ")"
);

/// The name this binary is invoked as, taken from `[[bin]] name` rather than
/// repeated as a literal. It reaches the user in three places that must agree:
/// the `Usage:` line, the `--help` header, and the `complete -F` registration a
/// generated completion script installs. Renaming the bin used to leave all
/// three claiming the old name -- and shell completion bound to a command that
/// no longer exists.
const BIN_NAME: &str = env!("CARGO_BIN_NAME");

#[derive(Parser)]
#[command(name = BIN_NAME, version = VERSION, about = "AI coding assistant in your terminal")]
#[command(group(
    ArgGroup::new("headless_input")
        .args(["prompt", "prompt_file"])
        .multiple(false)
))]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Continue the previous session instead of starting a new one
    #[arg(short = 'c', long = "continue")]
    continue_last: bool,

    /// Resume a SPECIFIC session by id or name (vs `--continue` = the last one).
    /// Headless: `-p "..." --resume <id>`. Interactive: `--resume <id>` (no `-p`)
    /// launches the TUI resumed on it -- same as the `resume` subcommand.
    #[arg(long = "resume", value_name = "ID_OR_NAME", conflicts_with_all = ["continue_last", "ephemeral"])]
    resume: Option<String>,

    /// Provider to use (overrides config default)
    #[arg(long)]
    provider: Option<String>,

    /// Model to use (overrides config provider model)
    #[arg(long)]
    model: Option<String>,

    /// Set interface language (e.g. en, zh-CN, zh)
    #[arg(long)]
    lang: Option<String>,

    /// Path to config file
    #[arg(long, value_hint = clap::ValueHint::FilePath)]
    config: Option<PathBuf>,

    /// FIRST-RUN ONLY: seed the user's config from this file when they don't yet
    /// have one (`~/.rustcode/config.toml` absent). Copies it in once, then never
    /// touches it again -- the user owns the writable copy. On read/parse failure,
    /// falls back to normal onboarding (never blocks startup). For offline/managed
    /// deploys (e.g. a bundled `rustcode-default-config.toml` shipped next to the
    /// binary): point this at that file via the launcher. Env: `RUSTCODE_SEED_CONFIG`.
    /// No-op when the user already has a config, so it's safe to always pass.
    /// Env `RUSTCODE_SEED_CONFIG` is honored as a fallback when the flag is absent.
    #[arg(long, value_name = "PATH", value_hint = clap::ValueHint::FilePath)]
    seed_config: Option<PathBuf>,

    /// Working directory (defaults to current directory)
    #[arg(long, short = 'C', value_hint = clap::ValueHint::DirPath)]
    dir: Option<PathBuf>,

    /// Prompt to run in headless (non-interactive) mode. If omitted, launches the TUI.
    #[arg(short = 'p', long)]
    prompt: Option<String>,

    /// Read the prompt from a file (alternative to -p). Useful for long prompts
    /// that would exceed ARG_MAX or whose trailing newlines matter.
    #[arg(
        long,
        value_name = "PATH",
        conflicts_with = "prompt",
        value_hint = clap::ValueHint::FilePath
    )]
    prompt_file: Option<std::path::PathBuf>,

    /// Run without creating, resuming, or writing session state (headless only).
    #[arg(long, requires = "headless_input", conflicts_with = "continue_last")]
    ephemeral: bool,

    /// Expose no tools, MCP servers, skills tools, or child-agent tools to the model
    /// (headless only).
    #[arg(long, requires = "headless_input")]
    no_tools: bool,

    /// Headless stdout format: assistant text (default) or versioned JSON Lines.
    #[arg(
        long,
        value_enum,
        default_value_t = HeadlessOutputFormat::Text,
        requires = "headless_input"
    )]
    output_format: HeadlessOutputFormat,

    /// Show tool calls, token usage, and turn summary on stderr (headless mode only).
    /// Without this flag, headless output is the assistant reply only -- Claude Code -p style.
    #[arg(short = 'v', long)]
    verbose: bool,

    /// Disable auto-update for this launch. Skips applying any staged
    /// upgrade and skips the detached background stager. Use during
    /// local development so a
    /// fresh `cargo run` build isn't silently overwritten by the
    /// released binary.
    #[arg(long)]
    dev: bool,

    /// Skip all permission prompts -- auto-approve every tool call (bash,
    /// file edits, MCP, etc.). Equivalent to Claude Code's
    /// --dangerously-skip-permissions. The TUI shows a red [!] BYPASS
    /// badge while active. Use in CI/CD, eval harnesses, or when you
    /// trust the agent's built-in safety constraints.
    #[arg(
        short = 'y',
        long = "dangerously-skip-permissions",
        visible_alias = "yolo",
        default_value_t = false
    )]
    pub dangerously_skip_permissions: bool,

    /// Permission / sandbox mode, mirroring the common cross-tool flags:
    /// Claude Code's `--permission-mode <default|acceptEdits|plan|bypassPermissions>`,
    /// Codex's `--sandbox`, and opencode's `--mode <read-only|accept-edits|auto|bypass-permissions>`.
    /// `auto` and `bypass-permissions` are equivalent to
    /// `--dangerously-skip-permissions` / `--yolo`.
    /// Interactive runs apply the mode at startup; headless runs treat
    /// `auto` / `bypass-permissions` as auto-approve and keep the default
    /// fail-closed behavior for `default` / `accept-edits` / `plan`.
    #[arg(long = "permission-mode", value_enum, value_name = "MODE")]
    pub permission_mode: Option<PermissionModeArg>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
enum HeadlessOutputFormat {
    #[default]
    Text,
    Jsonl,
}

/// Cross-tool permission mode, merging Claude Code's `--permission-mode`,
/// Codex's `--sandbox`, and opencode's `--mode <read-only|accept-edits|auto|bypass-permissions>`.
/// The clap value names are kebab-case: `default`, `accept-edits`, `auto`,
/// `plan`, `bypass-permissions`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
enum PermissionModeArg {
    /// Prompt before every write (the fail-closed default).
    #[default]
    Default,
    /// Auto-accept file edits, still gated elsewhere.
    AcceptEdits,
    /// Full autonomy: auto-approve writes and keep looping (equivalent to
    /// --dangerously-skip-permissions / --yolo).
    Auto,
    /// Read-only planning: no writes, no approvals needed.
    Plan,
    /// Bypass approvals entirely (equivalent to `auto` / --dangerously-skip-permissions / --yolo).
    BypassPermissions,
}

impl PermissionModeArg {
    /// Map onto the runtime's execution mode. `auto` and `bypass-permissions`
    /// become `Auto` (full autonomy); the others map 1:1 onto `Build` /
    /// `AcceptEdits` / `Plan`.
    fn runtime_mode(self) -> rustcode_coding::RuntimeMode {
        match self {
            Self::Default => rustcode_coding::RuntimeMode::Build,
            Self::AcceptEdits => rustcode_coding::RuntimeMode::AcceptEdits,
            Self::Auto => rustcode_coding::RuntimeMode::Auto,
            Self::Plan => rustcode_coding::RuntimeMode::Plan,
            Self::BypassPermissions => rustcode_coding::RuntimeMode::Auto,
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Show current provider and sign-in status
    Status,
    /// Resume a session by id or name (launches the TUI on it). With no
    /// argument, resumes the most recent session. Add `-p "<prompt>"` to run
    /// headless instead. Mirrors `--resume`; the exit hint prints this form.
    Resume {
        /// Session id or name to resume (default: the most recent session).
        session: Option<String>,
    },
    /// Upgrade rustcode in-place to the latest released version
    Upgrade {
        /// Reinstall even when already on the latest version
        #[arg(long)]
        force: bool,
    },
    /// Roll back to the previous version (swap with .bak on disk)
    Rollback,
    /// Manage MCP server entries in `.mcp.json` (similar to `claude mcp add`)
    #[command(subcommand)]
    Mcp(McpCli),
    /// Start the HTTP daemon for IDE integration (VS Code extension connects to this)
    Daemon {
        /// Port to listen on
        #[arg(long, default_value_t = rustcode_config::distribution::DAEMON_PORT)]
        port: u16,
        /// Bind address (default 0.0.0.0: reachable over LAN; use 127.0.0.1 to
        /// restrict to this machine. Token-protected only, with no TLS)
        #[arg(long, default_value = "0.0.0.0")]
        host: String,
        /// Client identifier (drives LOCAL client-mode branching, not reporting;
        /// e.g. "vscode", "rustcode-air")
        #[arg(long)]
        client: Option<String>,
        /// Idle-shutdown timeout in seconds; 0 disables. Env
        /// RUSTCODE_DAEMON_IDLE_TIMEOUT overrides. Default 1800 (30 min).
        #[arg(long)]
        idle_timeout: Option<u64>,
    },
    /// Start the local in-process browser webui server (no separate binary needed)
    Webui {
        /// Port (default 13457; deliberately offset from the VSCode daemon's 13456 to avoid
        /// port clashes that cause extension 401 / no-response)
        #[arg(long, default_value_t = rustcode_daemon::WEBUI_DEFAULT_PORT)]
        port: u16,
        /// Bind address (default 0.0.0.0: reachable over LAN; use 127.0.0.1 to
        /// restrict to this machine. Token-protected only, with no TLS)
        #[arg(long, default_value = "0.0.0.0")]
        host: String,
    },
    /// Manage skill/command plugins (mirrors `claude plugin ...`).
    /// Operates on `$RUSTCODE_HOME/plugins/` shared with the TUI's `/plugin`
    /// slash command -- anything installed via either path is visible to both.
    #[command(subcommand)]
    Plugin(PluginCli),
    /// Uninstall RustCode: remove the binary, PATH edit, and (interactively)
    /// data under ~/.rustcode/. With no flags, runs interactively and asks
    /// per-group; pass --yes / --purge / --keep-data for non-interactive use.
    Uninstall {
        /// Skip prompts; use per-group default decisions
        /// (binary=yes, credentials=no, state=yes).
        #[arg(long)]
        yes: bool,
        /// Wipe ~/.rustcode/ entirely.
        #[arg(long, conflicts_with = "keep_data")]
        purge: bool,
        /// Keep ~/.rustcode/ entirely (only remove binary + PATH edit).
        #[arg(long)]
        keep_data: bool,
        /// Print the plan; do nothing.
        #[arg(long)]
        dry_run: bool,
    },
    /// Install seed files (skills/commands/hooks/MCP) to `~/.rustcode/`.
    Setup {
        /// Take over a stale lock AND force reinstall even if seeds are already present.
        #[arg(long)]
        force: bool,
    },
    /// Manage hooks (list, test, enable/disable)
    #[command(subcommand)]
    Hooks(HookCommands),
    /// Manage local scheduled tasks (add/list/remove/enable/disable).
    #[command(subcommand)]
    Schedule(schedule_cmd::ScheduleCli),
    /// Generate a project wiki (architecture diagram + module docs) and sync on change.
    Wiki(WikiArgs),
    /// Generate a shell completion script on stdout.
    Completion(CompletionCommand),
    /// Detect installed IDEs (VS Code / Cursor / JetBrains etc.) and install
    /// the matching RustCode extension.
    #[command(subcommand)]
    Ide(IdeCli),
    /// Internal: askpass helper invoked by sudo/ssh via SUDO_ASKPASS / SSH_ASKPASS.
    /// Not intended for direct user invocation.
    #[command(name = "__askpass", hide = true)]
    Askpass {
        /// The prompt string forwarded by sudo/ssh (e.g. "[sudo] password:").
        prompt: String,
    },
    /// Run as an Agent Client Protocol (ACP) agent over stdio.
    ///
    /// stdout is reserved exclusively for the ACP JSON-RPC stream.
    /// Provider and model are taken from the active configuration
    /// (same as the TUI/headless path); per-session cwd comes from
    /// the ACP client's `session/new` request.
    #[command(hide = true)]
    Acp,
}

#[derive(clap::Args)]
struct CompletionCommand {
    /// Shell to generate completions for.
    #[arg(value_enum, default_value_t = Shell::Bash)]
    shell: Shell,
}

/// `rustcode ide` — detect IDEs and install extensions.
#[derive(clap::Subcommand)]
enum IdeCli {
    /// List detected IDEs and their extension status.
    List,
    /// Install the RustCode extension into an IDE.
    Install {
        /// IDE to install into: vscode | cursor | vscodium | jetbrains
        ide: Option<String>,
        /// Install into all detected IDEs (ignores the positional).
        #[arg(long)]
        all: bool,
    },
}

/// Supported IDE kinds for detection and install.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum IdeKind {
    Vscode,
    Cursor,
    Vscodium,
    Jetbrains,
}

impl IdeKind {
    /// Display name shown in `ide list` output.
    fn display_name(self) -> &'static str {
        match self {
            Self::Vscode => "VS Code",
            Self::Cursor => "Cursor",
            Self::Vscodium => "VSCodium",
            Self::Jetbrains => "JetBrains",
        }
    }

    /// Wire tag used in CLI positional args.
    #[allow(dead_code)]
    fn tag(self) -> &'static str {
        match self {
            Self::Vscode => "vscode",
            Self::Cursor => "cursor",
            Self::Vscodium => "vscodium",
            Self::Jetbrains => "jetbrains",
        }
    }

    /// Parse a user-provided tag.
    fn from_tag(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "vscode" | "code" => Some(Self::Vscode),
            "cursor" => Some(Self::Cursor),
            "vscodium" => Some(Self::Vscodium),
            "jetbrains" | "idea" | "intellij" => Some(Self::Jetbrains),
            _ => None,
        }
    }

    /// Executable names to probe on PATH (first hit wins).
    fn exe_names(self) -> &'static [&'static str] {
        match self {
            // `code` on all platforms; Windows also has `code.cmd`.
            Self::Vscode => &["code"],
            Self::Cursor => &["cursor"],
            Self::Vscodium => &["vscodium", "codium"],
            // `idea` is the launcher script; not always on PATH.
            Self::Jetbrains => &["idea", "idea.sh"],
        }
    }

    /// The extension / plugin id to install.
    fn extension_id(self) -> &'static str {
        match self {
            // VS Code family shares the same VSIX.
            Self::Vscode | Self::Cursor | Self::Vscodium => "rustcode-tools.rustcode",
            // JetBrains plugin id from plugin.xml.
            Self::Jetbrains => "com.rustcode.jetbrains",
        }
    }

    /// Whether this IDE supports `--install-extension` (VS Code CLI) or
    /// requires manual install (JetBrains marketplace).
    fn supports_cli_install(self) -> bool {
        matches!(self, Self::Vscode | Self::Cursor | Self::Vscodium)
    }

    /// Marketplace URL for manual install guidance.
    fn marketplace_url(self) -> &'static str {
        match self {
            Self::Vscode | Self::Cursor | Self::Vscodium => {
                "https://gitcode.com/SecLab/RustCode/extensions/vscode"
            }
            Self::Jetbrains => "https://gitcode.com/SecLab/RustCode/extensions/jetbrains",
        }
    }
}

/// Probe PATH for an executable. Returns the resolved path if found.
fn which(exe: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(exe);
        if candidate.is_file() {
            return Some(candidate);
        }
        // Windows: check .exe / .cmd / .bat extensions
        #[cfg(target_os = "windows")]
        {
            for ext in &["exe", "cmd", "bat"] {
                let with_ext = dir.join(format!("{exe}.{ext}"));
                if with_ext.is_file() {
                    return Some(with_ext);
                }
            }
        }
    }
    None
}

/// Detect all installed IDEs. Returns a list of (IdeKind, resolved exe path).
fn detect_ides() -> Vec<(IdeKind, PathBuf)> {
    let mut found = Vec::new();
    for kind in [
        IdeKind::Vscode,
        IdeKind::Cursor,
        IdeKind::Vscodium,
        IdeKind::Jetbrains,
    ] {
        for exe in kind.exe_names() {
            if let Some(path) = which(exe) {
                found.push((kind, path));
                break;
            }
        }
    }
    found
}

/// Check whether the VS Code-family extension is already installed.
/// Returns true if installed (or indeterminate — treat as not-installed to be safe).
fn vscode_extension_installed(exe: &std::path::Path, ext_id: &str) -> bool {
    let output = std::process::Command::new(exe)
        .args(["--list-extensions"])
        .output();
    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout.lines().any(|line| line.trim() == ext_id)
        }
        Err(_) => false,
    }
}

/// Run `code --install-extension <id>` for the VS Code family.
fn install_vscode_extension(exe: &std::path::Path, ext_id: &str) -> Result<(), String> {
    let output = std::process::Command::new(exe)
        .args(["--install-extension", ext_id])
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let err = if !stderr.is_empty() { stderr } else { stdout };
        Err(err.trim().to_string())
    }
}

/// Handle `rustcode ide list`.
fn run_ide_list() -> Result<()> {
    let t = rustcode_config::i18n::t;
    let ides = detect_ides();
    if ides.is_empty() {
        println!("{}", t(rustcode_config::i18n::Msg::CliIdeNoneDetected));
        return Ok(());
    }
    println!("{}", t(rustcode_config::i18n::Msg::CliIdeHeader));
    for (kind, path) in &ides {
        let path_str = path.display().to_string();
        let ext_status: String = if kind.supports_cli_install() {
            if vscode_extension_installed(path, kind.extension_id()) {
                "installed".to_string()
            } else {
                "not installed".to_string()
            }
        } else {
            "n/a".to_string()
        };
        println!(
            "{}",
            t(rustcode_config::i18n::Msg::CliIdeDetected {
                ide: kind.display_name(),
                path: &path_str,
                ext: &ext_status,
            })
            .into_owned()
        );
    }
    Ok(())
}

/// Handle `rustcode ide install [--all] [ide]`.
fn run_ide_install(all: bool, ide: Option<&str>) -> Result<()> {
    let t = rustcode_config::i18n::t;

    let targets: Vec<IdeKind> = if all {
        detect_ides().into_iter().map(|(k, _)| k).collect()
    } else {
        match ide {
            Some(name) => match IdeKind::from_tag(name) {
                Some(kind) => vec![kind],
                None => {
                    println!(
                        "{}",
                        t(rustcode_config::i18n::Msg::CliIdeUnknown { ide: name })
                    );
                    anyhow::bail!("unknown ide: {name}");
                }
            },
            None => {
                // No positional and no --all: install into all detected.
                detect_ides().into_iter().map(|(k, _)| k).collect()
            }
        }
    };

    if targets.is_empty() {
        println!("{}", t(rustcode_config::i18n::Msg::CliIdeNoneDetected));
        anyhow::bail!("no ide detected");
    }

    let mut had_failure = false;
    for kind in targets {
        let name = kind.display_name();
        println!(
            "{}",
            t(rustcode_config::i18n::Msg::CliIdeInstalling { ide: name })
        );

        if !kind.supports_cli_install() {
            println!(
                "{}",
                t(rustcode_config::i18n::Msg::CliIdeManualRequired {
                    ide: name,
                    marketplace_url: kind.marketplace_url(),
                })
            );
            continue;
        }

        // Resolve exe path
        let exe_path = kind.exe_names().iter().find_map(|exe| which(exe));

        match exe_path {
            Some(exe) => {
                let ext_id = kind.extension_id();
                // Skip if already installed
                if vscode_extension_installed(&exe, ext_id) {
                    println!(
                        "{}",
                        t(rustcode_config::i18n::Msg::CliIdeInstallOk { ide: name })
                    );
                    continue;
                }
                match install_vscode_extension(&exe, ext_id) {
                    Ok(()) => println!(
                        "{}",
                        t(rustcode_config::i18n::Msg::CliIdeInstallOk { ide: name })
                    ),
                    Err(e) => {
                        let err_str = e.to_string();
                        println!(
                            "{}",
                            t(rustcode_config::i18n::Msg::CliIdeInstallFailed {
                                ide: name,
                                error: &err_str,
                            })
                            .into_owned()
                        );
                        had_failure = true;
                    }
                }
            }
            None => {
                println!(
                    "{}",
                    t(rustcode_config::i18n::Msg::CliIdeNotFound { ide: name })
                );
                had_failure = true;
            }
        }
    }
    if had_failure {
        anyhow::bail!("one or more ide installs failed");
    }
    Ok(())
}

/// `rustcode wiki` — generate / sync the project wiki.
#[derive(clap::Args)]
struct WikiArgs {
    /// Project root to analyze (default: current directory).
    path: Option<PathBuf>,
    /// Incremental sync: only update pages whose source changed (default behavior of `/wiki`).
    #[arg(long)]
    sync: bool,
    /// Watch for changes and re-sync every `--interval` seconds.
    #[arg(long)]
    watch: bool,
    /// Enrich module pages with natural-language summaries via the configured LLM.
    #[arg(long)]
    llm: bool,
    /// Force full regeneration even if nothing changed.
    #[arg(long)]
    force: bool,
    /// Provider (model-selection id) to use for `--llm` enrichment
    /// (default: the globally active provider).
    #[arg(long)]
    provider: Option<String>,
    /// Model override for `--llm` enrichment (default: the provider's model).
    #[arg(long)]
    model: Option<String>,
    /// Output directory (default: <root>/.rustcode/wiki).
    #[arg(long)]
    out_dir: Option<PathBuf>,
    /// Wiki title (default: project name).
    #[arg(long)]
    title: Option<String>,
    /// Extra directories to exclude (repeatable).
    #[arg(long = "exclude")]
    exclude: Vec<String>,
    /// Watch interval in seconds (with --watch).
    #[arg(long, default_value_t = 30)]
    interval: u64,
    /// Output language(s): `zh` and/or `en` (repeatable). Default: both.
    #[arg(long = "lang", value_name = "ZH|EN")]
    lang: Vec<String>,
    /// Assume yes: update an existing auto-generated wiki without prompting.
    #[arg(short = 'y', long = "yes")]
    yes: bool,
}

/// Run the `rustcode wiki` subcommand: generate / sync the project wiki, optionally
/// enrich with the configured LLM, and optionally watch for changes.
async fn run_wiki_command(args: WikiArgs) -> i32 {
    use rustcode_config::i18n::{t, Msg};

    let root = args
        .path
        .clone()
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_default();
    let config =
        rustcode_config::Config::load(&rustcode_config::Config::default_path()).unwrap_or_default();

    // LLM enrichment is opt-in via `--llm` or `[wiki] use_llm`. The provider and
    // model can be chosen independently of the interactive session (flags win
    // over the `[wiki]` config, which wins over the globally active provider).
    let (use_llm, wiki_provider, wiki_model) = resolve_wiki_llm_selection(&args, &config);

    let cfg_out = config.wiki.out_dir.clone().map(PathBuf::from);
    let cfg_exclude = config.wiki.exclude_dirs.clone();
    let out_dir = if args.out_dir.is_some() {
        args.out_dir.clone()
    } else {
        cfg_out
    };
    let exclude = if !args.exclude.is_empty() {
        args.exclude.clone()
    } else {
        cfg_exclude
    };
    // Resolve output languages: explicit `--lang` flags win, then the
    // `[wiki] langs` config, then the engine default (both zh + en).
    let wiki_langs: Vec<rustcode_wiki::WikiLang> = if !args.lang.is_empty() {
        args.lang
            .iter()
            .filter_map(|s| rustcode_wiki::WikiLang::parse(s))
            .collect()
    } else if !config.wiki.langs.is_empty() {
        config
            .wiki
            .langs
            .iter()
            .filter_map(|s| rustcode_wiki::WikiLang::parse(s))
            .collect()
    } else {
        Vec::new()
    };
    let mut opts = rustcode_wiki::WikiOptions {
        root,
        out_dir,
        title: args.title.clone(),
        force: args.force,
        max_files: 20000,
        exclude_dirs: exclude,
        langs: wiki_langs,
        assume_yes: false,
    };

    let out_display = match &opts.out_dir {
        Some(d) if d.is_absolute() => d.clone(),
        Some(d) => opts.root.join(d),
        None => opts.root.join(".rustcode").join("wiki"),
    }
    .display()
    .to_string();

    // Safety guard: never clobber a directory the user created by hand.
    match rustcode_wiki::WikiEngine::precheck(&opts) {
        rustcode_wiki::WikiTargetState::Foreign => {
            eprintln!(
                "{}",
                t(Msg::WikiForeignConflict { path: &out_display }).into_owned()
            );
            return 1;
        }
        rustcode_wiki::WikiTargetState::AutoGenerated => {
            if !confirm_overwrite(&out_display, args.yes) {
                eprintln!("已取消，未改动现有 wiki。");
                return 0;
            }
            opts.assume_yes = true;
        }
        rustcode_wiki::WikiTargetState::Missing => {}
    }

    if args.watch {
        println!("{}", t(Msg::WikiGenerating).into_owned());
        loop {
            match rustcode_wiki::WikiEngine::sync(&opts) {
                Ok(res) => print_wiki_result(&res),
                Err(e) => eprintln!("wiki error: {e}"),
            }
            if use_llm {
                if let Some(provider) =
                    build_wiki_provider(&config, wiki_provider.as_deref(), wiki_model.as_deref())
                {
                    match enrich_wiki(&opts, &provider).await {
                        Ok(n) => println!("{}", t(Msg::WikiEnriched { count: n }).into_owned()),
                        Err(e) => {
                            eprintln!("{}", t(Msg::WikiEnrichSkipped { reason: &e }).into_owned())
                        }
                    }
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(args.interval)).await;
        }
    }

    let result = if args.sync {
        rustcode_wiki::WikiEngine::sync(&opts)
    } else {
        rustcode_wiki::WikiEngine::generate(&opts)
    };
    let result = match result {
        Ok(r) => r,
        Err(e) => {
            eprintln!("wiki error: {e}");
            return 1;
        }
    };
    print_wiki_result(&result);

    if use_llm {
        if let Some(provider) =
            build_wiki_provider(&config, wiki_provider.as_deref(), wiki_model.as_deref())
        {
            eprintln!("{}", t(Msg::WikiEnriching).into_owned());
            match enrich_wiki(&opts, &provider).await {
                Ok(n) => println!("{}", t(Msg::WikiEnriched { count: n }).into_owned()),
                Err(e) => eprintln!("{}", t(Msg::WikiEnrichSkipped { reason: &e }).into_owned()),
            }
        } else {
            eprintln!(
                "{}",
                t(Msg::WikiEnrichSkipped {
                    reason: "no usable provider configured"
                })
                .into_owned()
            );
        }
    }
    0
}

/// Whether a wiki run should be reported as "up to date": nothing was created,
/// updated, preserved, or removed. Extracted from [`print_wiki_result`] so the
/// removed-files path is unit-testable without capturing stdout.
fn wiki_run_is_up_to_date(res: &rustcode_wiki::WikiResult) -> bool {
    res.created.is_empty()
        && res.updated.is_empty()
        && res.preserved.is_empty()
        && res.removed.is_empty()
}

/// Human-facing lines for the files a wiki run removed (stale module pages and
/// their orphaned LLM summary sidecars). Empty when nothing was removed.
fn wiki_removed_lines(res: &rustcode_wiki::WikiResult) -> Vec<String> {
    use rustcode_config::i18n::{t, Msg};
    res.removed
        .iter()
        .map(|p| {
            t(Msg::WikiFileRemoved {
                path: &p.display().to_string(),
            })
            .into_owned()
        })
        .collect()
}

/// Print a one-line summary of a wiki run to stdout.
fn print_wiki_result(res: &rustcode_wiki::WikiResult) {
    use rustcode_config::i18n::{t, Msg};
    let changed = res.created.len() + res.updated.len();
    if wiki_run_is_up_to_date(res) {
        println!("{}", t(Msg::WikiSyncUpToDate).into_owned());
    } else {
        if changed > 0 {
            let path = res.out_dir.display().to_string();
            println!(
                "{}",
                t(Msg::WikiSummary {
                    modules: res.modules,
                    files: changed,
                    path: &path,
                })
                .into_owned()
            );
        }
        for p in &res.preserved {
            eprintln!(
                "{}",
                t(Msg::WikiFilePreserved {
                    path: &p.display().to_string()
                })
                .into_owned()
            );
        }
        for line in wiki_removed_lines(res) {
            println!("{line}");
        }
    }
}

/// Ask the user whether to update an existing auto-generated wiki. Returns `true`
/// when the user (or `--yes`) confirms. In a non-interactive context (no TTY) it
/// refuses unless `--yes` is given, so automated runs never silently overwrite.
fn confirm_overwrite(path: &str, yes: bool) -> bool {
    use std::io::{BufRead, IsTerminal, Write};
    if yes {
        return true;
    }
    if !std::io::stdin().is_terminal() {
        eprintln!("[WARN] 检测到 {path} 下已有自动生成的 wiki；非交互环境请加 --yes 以确认更新。");
        return false;
    }
    eprint!(
        "检测到 {path} 下已有由 rustcode-wiki 生成的 wiki，是否更新（将覆盖现有生成内容）？[y/N] "
    );
    let _ = std::io::stdout().flush();
    let mut line = String::new();
    if std::io::stdin().lock().read_line(&mut line).is_err() {
        return false;
    }
    let line = line.trim().to_ascii_lowercase();
    matches!(line.as_str(), "y" | "yes" | "是")
}

/// Resolve the wiki's LLM enrichment selection as
/// `(use_llm, provider, model)`.
///
/// Precedence is CLI flag > `[wiki]` config > globally active provider (the
/// last one is represented by `None`, letting `build_wiki_provider` fall back
/// to `Config::active_provider(None)`). Extracted as a pure function so the
/// precedence can be unit-tested without building an `OpenAiCompatProvider`
/// (which needs a real base URL and API key).
fn resolve_wiki_llm_selection(
    args: &WikiArgs,
    config: &rustcode_config::Config,
) -> (bool, Option<String>, Option<String>) {
    let use_llm = args.llm || config.wiki.use_llm;
    let provider = args
        .provider
        .clone()
        .or_else(|| config.wiki.provider.clone());
    let model = args.model.clone().or_else(|| config.wiki.model.clone());
    (use_llm, provider, model)
}

/// Build an OpenAI-compatible provider for wiki LLM enrichment.
///
/// `provider_name` selects a specific provider (model-selection id); `None`
/// falls back to the globally active provider. `model_override` replaces the
/// provider's configured model, so the wiki can enrich with a model of its own
/// choosing, independent of the interactive session. Returns `None` when no
/// usable provider is configured.
fn build_wiki_provider(
    config: &rustcode_config::Config,
    provider_name: Option<&str>,
    model_override: Option<&str>,
) -> Option<OpenAiCompatProvider> {
    let mut provider = config.active_provider(provider_name).ok()?;
    let base_url = provider.base_url.as_deref()?.to_string();
    let api_key = provider.resolved_api_key().unwrap_or_default();
    if let Some(m) = model_override.filter(|s| !s.trim().is_empty()) {
        provider.model = m.to_string();
    }
    let model = provider.model.clone();
    let cfg = OpenAiCompatConfig::new(api_key, base_url, model);
    OpenAiCompatProvider::new(cfg).ok()
}

/// Enrich each module page with a natural-language summary from the LLM.
///
/// Takes `&dyn LlmProvider` (not a concrete `OpenAiCompatProvider`) so the
/// enrichment logic is testable with a mock provider instead of a live network.
async fn enrich_wiki(
    opts: &rustcode_wiki::WikiOptions,
    provider: &dyn rustcode_kernel::provider::LlmProvider,
) -> Result<usize, String> {
    use futures::StreamExt;
    use rustcode_kernel::message::Message;
    use rustcode_kernel::provider::ChatOptions;
    use rustcode_kernel::stream::StreamEvent;

    let model = rustcode_wiki::WikiEngine::analyze(opts).map_err(|e| e.to_string())?;
    let out_dir = match &opts.out_dir {
        Some(d) if d.is_absolute() => d.clone(),
        Some(d) => opts.root.join(d),
        None => opts.root.join(".rustcode").join("wiki"),
    };
    // Never write sidecar summaries into a directory the user created by hand.
    if let rustcode_wiki::WikiTargetState::Foreign = rustcode_wiki::WikiEngine::precheck(opts) {
        return Err(
            "wiki 目录已存在但不是由 rustcode-wiki 生成，已停止以免覆盖你的内容。".to_string(),
        );
    }
    let langs = rustcode_wiki::resolve_langs(opts);
    let mut count = 0usize;
    let name_map = rustcode_wiki::module_file_names(&model);
    for m in &model.modules {
        for &lang in &langs {
            let prompt = rustcode_wiki::module_enrich_prompt(&model, m, lang);
            let messages = vec![Message::user(prompt)];
            let stream = provider
                .chat_stream(&messages, &[], &ChatOptions::default())
                .await
                .map_err(|e| format!("{e:?}"))?;
            let mut summary = String::new();
            let mut s = stream;
            while let Some(ev) = s.next().await {
                match ev {
                    StreamEvent::TextDelta(t) => summary.push_str(&t),
                    StreamEvent::Error(e) => return Err(format!("{e:?}")),
                    _ => {}
                }
            }
            let summary = summary.trim();
            if summary.is_empty() {
                continue;
            }
            // Persist the enriched text in a per-language sidecar (so future
            // `sync` keeps it) and rewrite the module page to embed it.
            rustcode_wiki::write_module_summary(&out_dir, &model, &name_map, lang, m, summary)
                .map_err(|e| e.to_string())?;
        }
        count += 1;
    }
    Ok(count)
}

/// Parse and serve `rustcode completion [SHELL]` before normal startup.
///
/// `Cli::try_parse` is intentionally used instead of hand-parsing argv so
/// global options and clap validation retain their canonical semantics. We
/// only surface a parse error early when argv contains the completion
/// subcommand token; every other invocation falls through to the existing
/// localized parse path.
fn try_print_shell_completion() -> bool {
    if !is_completion_invocation(std::env::args_os().skip(1)) {
        return false;
    }

    match Cli::try_parse() {
        Ok(Cli {
            command: Some(Commands::Completion(command)),
            ..
        }) => {
            print_shell_completion(command.shell, &mut std::io::stdout());
            true
        }
        Ok(_) => false,
        Err(error) => {
            // `--help`/`--version` must fall through to the i18n help
            // renderer in main() (build_i18n_command); the raw derive
            // parser above would print English doc-comment help. Any
            // other error (unknown shell, bad usage) exits here as before.
            use clap::error::ErrorKind;
            match error.kind() {
                ErrorKind::DisplayHelp
                | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
                | ErrorKind::DisplayVersion => false,
                _ => error.exit(),
            }
        }
    }
}

fn is_completion_invocation(args: impl IntoIterator<Item = std::ffi::OsString>) -> bool {
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let Some(arg) = arg.to_str() else {
            return false;
        };
        match arg {
            // `--` ends root option parsing; anything after it is input, not
            // RustCode's completion subcommand.
            "--" => return false,
            // Root flags that do not consume a value.
            "-c"
            | "--continue"
            | "-v"
            | "--verbose"
            | "--dev"
            | "-y"
            | "--dangerously-skip-permissions" => {}
            // Root options that consume the following argv item.
            "--provider" | "--model" | "--lang" | "--config" | "--seed-config" | "-C" | "--dir"
            | "-p" | "--prompt" | "--prompt-file" => {
                if args.next().is_none() {
                    return false;
                }
            }
            // Long options may carry their value after `=`.
            value
                if [
                    "--provider=",
                    "--model=",
                    "--lang=",
                    "--config=",
                    "--seed-config=",
                    "--dir=",
                    "--prompt=",
                    "--prompt-file=",
                ]
                .iter()
                .any(|prefix| value.starts_with(prefix)) => {}
            // The first root positional token is the subcommand.
            value if !value.starts_with('-') => return value == "completion",
            // Unknown/combined options belong to the canonical parser.
            _ => return false,
        }
    }
    false
}

fn completion_command() -> clap::Command {
    // Source from the i18n-built command, not the raw derive: this is where the
    // neutral build hides subcommands (and sets localized abouts), so shell
    // completion must mirror the same visibility filter as `rustcode --help`
    // -- a hidden command is one the completion script must not advertise.
    let source = build_i18n_command();
    let visible_subcommands = source
        .get_subcommands()
        .filter(|command| !command.is_hide_set())
        .cloned()
        .collect::<Vec<_>>();

    clap::Command::new(BIN_NAME)
        .version(VERSION)
        .about("AI coding assistant in your terminal")
        .args(source.get_arguments().cloned())
        .groups(source.get_groups().cloned())
        .subcommands(visible_subcommands)
}

fn print_shell_completion(shell: Shell, out: &mut dyn Write) {
    let mut command = completion_command();
    clap_complete::generate(shell, &mut command, BIN_NAME, out);
}

/// Subcommands for hooks management
#[derive(Subcommand)]
enum HookCommands {
    /// List all loaded hooks with their status
    List,
    /// Test a specific hook by name
    Test {
        /// Hook name to test
        name: String,
    },
    /// Show hook configuration paths
    Paths,
}

#[derive(Subcommand)]
enum PluginCli {
    /// Marketplace registry operations (add/remove/update/list).
    #[command(subcommand)]
    Marketplace(MarketplaceCli),
    /// Install a plugin from a registered marketplace.
    /// Spec format: `<plugin>@<marketplace>` (matches the slash command).
    Install {
        /// e.g. `ascend-model-agent-plugin@ascend-model-agent-plugin`
        spec: String,
    },
    /// Uninstall a previously-installed plugin (does not touch its marketplace).
    Uninstall {
        /// e.g. `ascend-model-agent-plugin@ascend-model-agent-plugin`
        spec: String,
    },
    /// Trust an installed plugin's hooks so they run (records the current hook-set hash).
    Trust {
        /// Plugin name (as installed), or `plugin@marketplace` for disambiguation.
        name: String,
    },
    /// Untrust an installed plugin's hooks (they stop running).
    Untrust {
        /// Plugin name (as installed), or `plugin@marketplace` for disambiguation.
        name: String,
    },
    /// List installed plugins.
    List,
}

#[derive(Subcommand)]
enum MarketplaceCli {
    /// Clone a marketplace git repo and register it locally.
    Add {
        /// Git URL (https or ssh) of a marketplace repo.
        url: String,
    },
    /// Drop a registered marketplace. Refuses if any plugin still installed.
    Remove {
        /// Marketplace name (the key shown by `marketplace list`).
        name: String,
    },
    /// Re-pull a registered marketplace and refresh its plugin index.
    Update { name: String },
    /// List registered marketplaces.
    List,
}

#[derive(Subcommand)]
enum McpCli {
    /// Add or replace a stdio MCP server (`mcpServers.<name>` with `command` + `args`)
    Add {
        /// Server key (tools appear as `mcp__<name>__...`)
        name: String,
        /// Executable and arguments, e.g. `npx @playwright/mcp@latest`
        #[arg(required = true, num_args = 1..)]
        command: Vec<String>,
        /// Write `~/.rustcode/mcp.json` instead of `<dir>/.mcp.json`
        #[arg(long)]
        global: bool,
        /// Directory for project `.mcp.json` (defaults to current directory)
        #[arg(short = 'C', long, value_hint = clap::ValueHint::DirPath)]
        dir: Option<PathBuf>,
    },
    /// Add a remote MCP server that authenticates with OAuth (provider-neutral).
    AddOauth {
        /// Server HTTP URL; OAuth metadata is discovered from the server.
        url: String,
        /// Server key (tools appear as `mcp__<name>__...`); defaults to the URL host.
        #[arg(long)]
        name: Option<String>,
        /// Write `~/.rustcode/mcp.json` instead of `<dir>/.mcp.json`
        #[arg(long)]
        global: bool,
        /// Directory for project `.mcp.json` (defaults to current directory)
        #[arg(short = 'C', long, value_hint = clap::ValueHint::DirPath)]
        dir: Option<PathBuf>,
    },
    /// Add GitHub's remote MCP server using OAuth.
    #[doc(hidden)]
    AddGithubOauth {
        /// Server key (tools appear as `mcp__<name>__...`)
        #[arg(default_value = "github")]
        name: String,
        /// Write `~/.rustcode/mcp.json` instead of `<dir>/.mcp.json`
        #[arg(long)]
        global: bool,
        /// Directory for project `.mcp.json` (defaults to current directory)
        #[arg(short = 'C', long, value_hint = clap::ValueHint::DirPath)]
        dir: Option<PathBuf>,
    },
    /// Complete OAuth login for a remote MCP server.
    Login {
        /// Server key in mcpServers.
        name: String,
        /// OAuth provider hint (optional; usually read from the server config).
        #[arg(long)]
        provider: Option<String>,
        /// OAuth client id.
        #[arg(long)]
        client_id: Option<String>,
        /// Environment variable containing the OAuth client secret.
        #[arg(long)]
        client_secret_env: Option<String>,
        /// OAuth scopes.
        #[arg(long, value_delimiter = ',')]
        scopes: Vec<String>,
    },
    /// Remove saved OAuth credentials for a remote MCP server.
    Logout {
        /// Server key in mcpServers.
        name: String,
    },
}

/// Environment variable set by this process for its re-exec'd child, so
/// the child knows which version it was just upgraded from and can show
/// a one-time "[+] Upgraded to vX.Y.Z" banner on the welcome screen.
/// The child clears this env var after reading it so grandchildren
/// (spawned tools, subprocesses) don't inherit a stale hint.
const UPGRADED_FROM_ENV: &str = "RUSTCODE_UPGRADED_FROM";

/// Env var the parent sets when spawning a detached upgrade-prep worker.
/// The child detects it at the very top of `main` and runs one
/// `prepare_deferred_upgrade` cycle in its own session (setsid'd) so the
/// parent can be Ctrl+C'd without cancelling the download.
const INTERNAL_PREPARE_UPGRADE_ENV: &str = "RUSTCODE_INTERNAL_PREPARE_UPGRADE";

fn main() {
    // Completion generation must be a pure, fast CLI operation: no helper
    // thread, Tokio runtime, log file, config read, or updater.
    // Shells may invoke completion helpers frequently, so even best-effort
    // startup work here would turn Tab into network/filesystem activity.
    if try_print_shell_completion() {
        return;
    }

    // Settle where the config tree is before anything reads it, so all eight
    // resolvers -- and every child process that inherits our environment --
    // agree by construction. Deliberately AFTER the completion fast path: that
    // branch touches no config, and the comment above asks for it to stay bare.
    rustcode_config::distribution::bootstrap_home();

    // Run the entire program on a thread with a large, explicit stack.
    // Rust gives the *main* OS thread the platform-default stack -- on
    // Windows that's only ~1 MB (vs 8 MB on Linux/macOS). The TUI event
    // loop, the synchronous OAuth work, and the rustls TLS
    // handshakes all run on it via `block_on`, and a deep call chain there
    // can overflow 1 MB. A stack overflow on Windows kills the process via
    // an OS exception (STATUS_STACK_OVERFLOW) WITHOUT a Rust panic -- so it
    // never reaches the crash-log hook and looks like a silent exit. A
    // 16 MB stack removes that platform asymmetry. (See the Windows
    // post-scan onboarding crash investigation.)
    let child = std::thread::Builder::new()
        .name("rustcode-main".into())
        .stack_size(16 * 1024 * 1024)
        .spawn(real_main)
        .expect("failed to spawn rustcode main thread");
    // Under `panic = "abort"` a panic in the child already aborts the whole
    // process, so `join` only returns an error on an abnormal thread exit;
    // mirror Rust's conventional panic exit code in that case.
    if child.join().is_err() {
        std::process::exit(101);
    }
}

fn real_main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        // Worker threads (for `tokio::spawn`ed tasks) get a generous stack
        // too -- same rationale as the main thread above.
        .thread_stack_size(8 * 1024 * 1024)
        .build()
        .expect("failed to build tokio runtime");
    rt.block_on(async_main());
}

fn merge_startup_notices(
    config_notice: Option<String>,
    session_notice: Option<String>,
) -> Option<String> {
    match (config_notice, session_notice) {
        (Some(config), Some(session)) => Some(format!("{config}\n{session}")),
        (Some(config), None) => Some(config),
        (None, Some(session)) => Some(session),
        (None, None) => None,
    }
}

async fn async_main() {
    let process_start = std::time::Instant::now();
    // Wire `tracing::` diagnostics to `<config_dir>/logs/rustcode.log` (file-only,
    // TUI-safe). Must run before anything that emits traces so nothing is lost.
    init_file_logging();
    // 一次性清除已废弃的平台凭证文件（rustcode-auth 已移除）。
    // 该文件来自已删除的 rustcode-auth，现既不被读取也无意义；启动时静默删除，
    // 让老用户机器干净。失败静默，不阻断启动。
    let legacy_auth = Config::config_dir().join("auth.toml");
    if legacy_auth.exists() {
        match std::fs::remove_file(&legacy_auth) {
            Ok(()) => tracing::info!("已清除遗留的平台凭证文件 auth.toml"),
            Err(e) => tracing::info!("跳过清理遗留 auth.toml: {e}"),
        }
    }
    // Set Windows console to UTF-8 so CJK and other multi-byte characters
    // render correctly instead of showing garbled output (mojibake).
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::Globalization::CP_UTF8;
        use windows_sys::Win32::System::Console::{
            GetConsoleCP, GetConsoleOutputCP, SetConsoleCP, SetConsoleOutputCP,
        };
        unsafe {
            SetConsoleOutputCP(CP_UTF8);
            SetConsoleCP(CP_UTF8);

            // Also check output code page, same best-effort as input.
            let actual_cp = GetConsoleCP();
            let actual_out_cp = GetConsoleOutputCP();
            if actual_cp != CP_UTF8 || actual_out_cp != CP_UTF8 {
                let _ = eprintln!(
                    "{}",
                    rustcode_config::i18n::t(rustcode_config::i18n::Msg::CliWindowsCodePage {
                        input: actual_cp,
                        output: actual_out_cp,
                    })
                );
            }
        }
    }

    // Detached upgrade-prep worker mode. The parent rustcode spawns a
    // subprocess with this env var set; that subprocess does one full
    // download + verify + `pending.json` write, then exits. Because the
    // subprocess is setsid'd (see `spawn_detached_upgrade_prep`), it
    // survives Ctrl+C / quit in the parent -- which is the whole point,
    // since the previous in-process download was tied to the parent's
    // tokio runtime and got cancelled on any quick exit.
    if std::env::var(INTERNAL_PREPARE_UPGRADE_ENV).is_ok() {
        let code = run_prepare_upgrade_worker().await;
        std::process::exit(code);
    }

    // If this invocation is the `.bak` backup binary (left behind by a
    // previous upgrade), skip all upgrade bootstrapping. `apply_pending_upgrade`
    // would rewrite ourselves with the latest version and destroy the
    // rollback target; the whole point of keeping `.bak` is for the user
    // to be able to run / keep the old version. The only upgrade path
    // still reachable from a `.bak` launch is the explicit `/upgrade`
    // slash command inside the TUI -- that's user-initiated and fine.
    let is_backup = is_running_as_backup();
    let dev_mode = is_dev_mode();
    if dev_mode {
        eprintln!(
            "{}",
            rustcode_config::i18n::t(rustcode_config::i18n::Msg::CliUpgradeDevDisabled)
        );
    }

    // Bootstrap: if a prior session staged an upgrade, apply it NOW -- before
    // we spin up tokio, the TUI, or any other heavy state. On success we
    // re-exec the new binary (Unix: same PID; Windows: child+exit). The user
    // sees one continuous "rustcode" invocation, just 100-300ms longer than
    // normal. On failure we log and carry on with the current binary; the
    // circuit-breaker in `apply_pending_upgrade` ensures a broken release
    // can't wedge this loop indefinitely.
    if !is_backup && !dev_mode {
        let stage_start = std::time::Instant::now();
        // Capture current version BEFORE applying upgrade, so we can pass it to the re-exec'd child
        let current_version = format!("v{}", env!("CARGO_PKG_VERSION"));
        match rustcode_updater::apply_pending_upgrade() {
            Ok(Some(applied)) => {
                eprintln!(
                    "{}",
                    rustcode_config::i18n::t(rustcode_config::i18n::Msg::CliUpgradeApplying {
                        version: &applied.version
                    })
                );
                // Pass the CURRENT version (before upgrade) to the re-exec'd child so the TUI
                // can surface a welcome-screen confirmation exactly once.
                std::env::set_var(UPGRADED_FROM_ENV, &current_version);
                match rustcode_updater::re_exec_self(Some(&applied.exe)) {
                    Ok(_infallible) => unreachable!("re_exec_self returned Ok"),
                    Err(e) => {
                        eprintln!(
                            "{}",
                            rustcode_config::i18n::t(
                                rustcode_config::i18n::Msg::CliUpgradeReexecFailed {
                                    error: &e.to_string()
                                }
                            )
                        );
                        std::env::remove_var(UPGRADED_FROM_ENV);
                        std::process::exit(1);
                    }
                }
            }
            Ok(None) => {
                // Nothing was staged by a prior session. Do a fresh synchronous
                // check -> stage -> apply so a newly-released version is picked up on
                // THIS launch (restores the pre-31daa6ee "one restart upgrades you"
                // behavior). The everyday no-update case is just a small latest.json
                // fetch; should_try_sync_upgrade's gates + a 120s timeout keep
                // headless / opted-out / offline runs fast, and the detached stager
                // below stays as the survives-quick-exit fallback.
                if should_try_sync_upgrade() {
                    // Remember we checked, so the post-parse detached stager doesn't
                    // re-fetch the manifest a second time this launch.
                    SYNC_UPGRADE_CHECKED.store(true, Ordering::Relaxed);
                    sync_stage_and_apply_if_newer().await;
                }
            }
            Err(e) => {
                eprintln!(
                    "{}",
                    rustcode_config::i18n::t(rustcode_config::i18n::Msg::CliUpgradeApplyFailed {
                        error: &e.to_string()
                    })
                );
            }
        }
        tracing::info!(
            target: "rustcode::startup",
            stage = "pending_upgrade",
            elapsed_ms = stage_start.elapsed().as_millis() as u64,
            total_ms = process_start.elapsed().as_millis() as u64,
            "startup stage completed"
        );
    } // end `if !is_backup`

    // Install a minimal panic hook (no network, no telemetry) that writes
    // the crash to stderr and the durable panic log.
    install_crash_panic_hook();

    match run().await {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            restore_terminal_if_tui();
            eprintln!(
                "{}",
                rustcode_config::i18n::t(rustcode_config::i18n::Msg::CliFatalError {
                    error: &format!("{e:#}")
                })
            );
            std::process::exit(1);
        }
    }
}

async fn run() -> Result<i32> {
    let run_start = std::time::Instant::now();
    // -- Pre-parse --lang to set locale BEFORE clap renders --help --
    // clap help text is generated during parse, so we must resolve
    // the locale first (from --lang flag, env vars, or config) so that
    // the i18n system is ready when clap calls our dynamic about/help closures.
    let pre_lang = scan_argv_for_lang();
    // Single pre-scan of the default config path: resolves language (for
    // locale) AND brand/oauth names (for --help placeholder substitution)
    // in one read + parse, not three independent scans.
    let pre_scan = scan_config_pre();
    let pre_locale =
        rustcode_tuix::i18n::resolve_initial_locale(pre_lang.as_deref(), pre_scan.language);
    rustcode_tuix::i18n::set_locale(pre_locale);

    // Build the clap Command with i18n-injected about/help text, then parse.
    // Check if --help or -h was requested by scanning argv.
    // If so, render the i18n-localised help via our custom Command and exit.
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        // --help renders BEFORE the authoritative Config load, so settle the
        // brand from the pre-scan (default config path + env) just for this
        // branch. The normal launch path does NOT call set_brand here -- it
        // waits for the authoritative Config load in run() so --config
        // / --seed-config custom paths surface the real brand, not the
        // default-path pre-scan value.
        rustcode_config::i18n::set_brand(&pre_scan.brand_name, &pre_scan.oauth_provider_name);
        let help_cmd = build_i18n_command();
        help_cmd
            .try_get_matches_from(std::env::args_os())
            .unwrap_or_else(|e| e.exit());
        // Unreachable: e.exit() prints the localised help and exits.
    }

    // No --help was passed. Parse normally to get the Cli struct.
    let cli = Cli::parse();

    // Merge the cross-tool permission flags into one effective posture.
    // `--yolo` / `--dangerously-skip-permissions` and
    // `--permission-mode bypass-permissions` all mean "auto-approve everything";
    // the remaining `--permission-mode` values select a run-time execution mode
    // applied at startup (and mirrored by TUI re-spawns).
    let skip_permissions = cli.dangerously_skip_permissions
        || matches!(
            cli.permission_mode,
            Some(PermissionModeArg::Auto | PermissionModeArg::BypassPermissions)
        );
    let initial_mode = match cli.permission_mode {
        Some(mode) => mode.runtime_mode(),
        None if cli.dangerously_skip_permissions => rustcode_coding::RuntimeMode::Auto,
        None => rustcode_coding::RuntimeMode::Build,
    };

    // ── Askpass early exit ────────────────────────────────────────────────────
    // Handle `rustcode __askpass <prompt>` before ANY TUI/daemon setup.
    // sudo/ssh invoke this helper synchronously; it must not spawn async
    // runtimes, perform network I/O, or open a terminal.
    if let Some(Commands::Askpass { prompt }) = &cli.command {
        #[cfg(unix)]
        {
            use std::path::Path;
            let sock = std::env::var("RUSTCODE_ASKPASS_SOCK").ok();
            let token = std::env::var("RUSTCODE_ASKPASS_TOKEN").ok();
            match (sock, token) {
                (Some(s), Some(t)) => {
                    match rustcode::askpass::run_askpass(prompt, Path::new(&s), &t) {
                        Some(pw) => {
                            use std::io::Write;
                            print!("{pw}");
                            let _ = std::io::stdout().flush();
                            return Ok(0);
                        }
                        None => return Ok(1),
                    }
                }
                _ => return Ok(1),
            }
        }
        #[cfg(not(unix))]
        {
            let _ = prompt;
            return Ok(1);
        }
    }
    // ── End askpass early exit ────────────────────────────────────────────────

    let is_admin = rustcode_capabilities::process_utils::is_running_as_admin();

    // ── Early config / offline seed ─────────────────────────────────────────
    // Load config early (before subcommand dispatch) so we can seed the
    // offline verdict + note before any tool/persona/provider assembly.
    // Failure to load config is non-fatal.
    let config_path_for_tel = cli.config.clone().unwrap_or_else(Config::default_path);
    let early_config = if config_path_for_tel.exists() {
        Config::load(&config_path_for_tel).ok()
    } else {
        None
    };

    // Seed the offline verdict + note ONCE from config + env, before any
    // tool/provider assembly.
    rustcode_config::config::offline::seed_offline_from_config(early_config.as_ref());
    // ── End early config / offline seed ──────────────────────────────────────

    // Handle subcommands. Most are self-contained (`handle_command` runs
    // and exits); a few (e.g. `Resume`) fall through to the TUI/headless
    // launch below.

    let force_verbose = false;
    // Capture the resume intent BEFORE the dispatch below moves `cli.command`.
    // `--resume`/`--continue` are plain flags; the `resume` subcommand is NOT a
    // terminal command -- it falls through to the normal TUI/headless launch with
    // this selector applied (see the launch flow).
    let resume_selector: Option<ResumeSelector> = if let Some(sel) = cli.resume.clone() {
        Some(ResumeSelector::Specific(sel))
    } else if let Some(Commands::Resume { session }) = &cli.command {
        Some(match session {
            Some(s) => ResumeSelector::Specific(s.clone()),
            None => ResumeSelector::Latest,
        })
    } else if cli.continue_last {
        Some(ResumeSelector::Latest)
    } else {
        None
    };
    if let Some(cmd) = cli.command {
        match cmd {
            // Not a terminal command: fall through to the TUI/headless launch with
            // `resume_selector` already captured above (interactive by default; add
            // `-p` to run headless resumed).
            Commands::Resume { .. } => {}
            Commands::Wiki(args) => {
                HEADLESS_MODE.store(true, Ordering::Relaxed);
                let code = run_wiki_command(args).await;
                return Ok(code);
            }
            Commands::Daemon {
                port,
                host,
                client,
                idle_timeout,
            } => {
                HEADLESS_MODE.store(true, Ordering::Relaxed);
                eprintln!(
                    "{}",
                    rustcode_config::i18n::t(rustcode_config::i18n::Msg::CliDaemonStarting {
                        port
                    })
                );
                eprintln!(
                    "{}",
                    rustcode_config::i18n::t(rustcode_config::i18n::Msg::CliDaemonStopHint)
                );
                // Run the bundled server IN-PROCESS (same `run_server` the webui uses),
                // instead of re-exec'ing into a separate `rustcode-daemon` binary that
                // may not be installed. This is an equivalent daemon entrypoint to the
                // standalone `rustcode-daemon` binary, so it MUST enforce the local token
                // identically (mirror of `rustcode-daemon/src/main.rs`): mint/resolve a
                // token, enable enforcement, and write `~/.rustcode/daemon-<port>.json`.
                // Otherwise `rustcode daemon` would be an unauthenticated bypass of the
                // very surface the token guards.
                let idle = idle_timeout
                    .or_else(|| {
                        std::env::var("RUSTCODE_DAEMON_IDLE_TIMEOUT")
                            .ok()
                            .and_then(|s| s.parse().ok())
                    })
                    .unwrap_or(30 * 60);
                // `--client` is NOT telemetry: the daemon uses it for permission
                // enforcement and the Webui interactive-input path.
                let startup_mode = rustcode_daemon::client_mode::resolve_client_mode(
                    client.as_deref().unwrap_or("ide"),
                );
                let token_store = rustcode_daemon::auth_token::WebuiTokenStore::new();
                let daemon_token = rustcode_daemon::resolve_daemon_token(
                    std::env::var("RUSTCODE_DAEMON_TOKEN").ok(),
                    &token_store,
                );
                let res = rustcode_daemon::run_server(rustcode_daemon::ServerOpts {
                    host,
                    port,
                    idle_timeout_secs: idle,
                    startup_mode,
                    webui_tokens: Some(token_store),
                    quiet: false,
                    working_dir_override: None,
                    prebound_listener: None,
                    daemon_token_file: Some(daemon_token),
                })
                .await;
                if let Err(e) = res {
                    eprintln!(
                        "{}",
                        rustcode_config::i18n::t(rustcode_config::i18n::Msg::CliDaemonFatal {
                            error: &format!("{e:#}")
                        })
                    );
                    return Ok(1);
                }
                return Ok(0);
            }
            Commands::Webui { port, host } => {
                HEADLESS_MODE.store(true, Ordering::Relaxed);
                // Fail before binding a port and opening a browser: without the
                // assets every page would be a 404, and the cause is a build
                // step, not anything the running server can recover from.
                if !rustcode_daemon::webui::is_built() {
                    eprint!("{}", rustcode_daemon::webui::not_built_help());
                    return Ok(1);
                }
                let msg = rustcode_daemon::ensure_server_and_open(&host, port, false).await;
                eprintln!("{msg}");
                // server 是后台 task；保持进程存活直到用户 Ctrl+C
                let _ = tokio::signal::ctrl_c().await;
                return Ok(0);
            }
            Commands::Setup { force } => {
                HEADLESS_MODE.store(true, Ordering::Relaxed);
                let exit_code = run_setup_command(force);
                return Ok(exit_code);
            }
            Commands::Acp => {
                // stdout is the ACP JSON-RPC channel -- no banner or diagnostic output here.
                HEADLESS_MODE.store(true, Ordering::Relaxed);
                // Load config the same way the TUI path does so provider/model resolution
                // is identical (honors --provider, --model, and config.toml).
                let config_path = cli.config.clone().unwrap_or_else(Config::default_path);
                let mut config = if config_path.exists() {
                    Config::load(&config_path).unwrap_or_default()
                } else {
                    Config::default()
                };
                apply_cli_runtime_overrides(
                    &mut config,
                    cli.provider.as_deref(),
                    cli.model.as_deref(),
                );
                let working_dir = resolve_working_dir(cli.dir.clone());
                let runtime_cfg = runtime_config_from(
                    &config,
                    &working_dir,
                    cli.provider.as_deref(),
                    skip_permissions,
                    // ACP sessions are interactive: approval prompts park until the
                    // client answers (not fail-closed like headless -p).
                    true,
                );
                // Honor `--dangerously-skip-permissions`: auto-approve kernel approval
                // requests in the turn loop instead of round-tripping to the client.
                let auto_approve = runtime_cfg.dangerously_skip_permissions;
                // The factory creates and binds a distinct authenticated provider for
                // every ACP session. Sharing one pre-built provider would pin gateway
                // session affinity to whichever session bound it first.
                let provider_factory = rustcode_daemon::coding_provider_factory();
                let engine = rustcode::acp::engine::EngineConfig::from_coding_config(
                    runtime_cfg.agent_config(),
                );
                // Session config option catalog (`session/set_config_option`):
                // a `mode` select over the kernel execution modes, a
                // `reasoning_effort` select mirroring the TUI `/effort` ladder,
                // and a `model` select over the configured model catalog. The
                // resolvers re-resolve a selected value into a kernel config for
                // the live runtime's provider reload.
                use agent_client_protocol::schema::v1::{
                    SessionConfigOption, SessionConfigOptionCategory, SessionConfigSelectOption,
                };
                let session_config_options: Vec<SessionConfigOption> = {
                    let mut options: Vec<SessionConfigOption> = Vec::new();
                    // Execution mode: always available, same ids/names as the
                    // `modes` advertised in the session setup response.
                    let modes = rustcode::acp::options::session_modes();
                    options.push(
                        SessionConfigOption::select(
                            rustcode::acp::options::MODE_CONFIG_ID,
                            "Mode",
                            rustcode_coding::RuntimeMode::Build.wire(),
                            modes
                                .iter()
                                .map(|m| {
                                    SessionConfigSelectOption::new(
                                        m.id.0.as_ref().to_string(),
                                        m.name.clone(),
                                    )
                                })
                                .collect::<Vec<_>>(),
                        )
                        .category(SessionConfigOptionCategory::Mode),
                    );
                    // Reasoning effort: always available; the active provider
                    // adapter decides whether the tier has an effect.
                    let current_effort = config
                        .effective_model_selection()
                        .and_then(|sel| config.provider_config_for_selection(&sel))
                        .and_then(|provider| provider.reasoning_effort.clone())
                        .unwrap_or_else(|| "off".to_string());
                    let effort_tiers: [(&str, &str); 3] = [
                        ("off", "Off (API default)"),
                        ("high", "High"),
                        ("max", "Max"),
                    ];
                    options.push(
                        SessionConfigOption::select(
                            rustcode::acp::options::REASONING_EFFORT_CONFIG_ID,
                            "Reasoning effort",
                            current_effort,
                            effort_tiers
                                .into_iter()
                                .map(|(value, name)| {
                                    SessionConfigSelectOption::new(
                                        value.to_string(),
                                        name.to_string(),
                                    )
                                })
                                .collect::<Vec<_>>(),
                        )
                        .category(SessionConfigOptionCategory::ThoughtLevel),
                    );
                    // Model selector: only when a model catalog is configured.
                    let mut ids: Vec<String> = config.logical_models().keys().cloned().collect();
                    ids.sort();
                    if !ids.is_empty() {
                        let current = config
                            .effective_model_selection()
                            .filter(|id| ids.contains(id))
                            .unwrap_or_else(|| ids[0].clone());
                        options.push(
                            SessionConfigOption::select(
                                rustcode::acp::options::MODEL_CONFIG_ID,
                                "Model",
                                current,
                                ids.into_iter()
                                    .map(|id| SessionConfigSelectOption::new(id.clone(), id))
                                    .collect::<Vec<_>>(),
                            )
                            .category(SessionConfigOptionCategory::Model),
                        );
                    }
                    options
                };
                let session_model_resolver: Option<Arc<rustcode::acp::SessionModelResolver>> =
                    if config.logical_models().is_empty() {
                        None
                    } else {
                        let base = config.clone();
                        let provider = cli.provider.clone();
                        let dir = working_dir.clone();
                        let skip = skip_permissions;
                        let resolver: Arc<rustcode::acp::SessionModelResolver> = Arc::new(
                            move |model: &str| -> Option<rustcode_coding::CodingAgentConfig> {
                                let mut cfg = base.clone();
                                cfg.default_model = Some(model.to_string());
                                let runtime = runtime_config_from(
                                    &cfg,
                                    &dir,
                                    provider.as_deref(),
                                    skip,
                                    true,
                                );
                                if runtime.model.is_empty() {
                                    return None;
                                }
                                Some(runtime.agent_config())
                            },
                        );
                        Some(resolver)
                    };
                let session_effort_resolver: Option<Arc<rustcode::acp::SessionModelResolver>> =
                    Some({
                        let base = config.clone();
                        let provider = cli.provider.clone();
                        let dir = working_dir.clone();
                        let skip = skip_permissions;
                        let resolver: Arc<rustcode::acp::SessionModelResolver> = Arc::new(
                            move |effort: &str| -> Option<rustcode_coding::CodingAgentConfig> {
                                let mut cfg = base.clone();
                                let selection = cfg.effective_model_selection()?;
                                cfg.update_selection_reasoning(&selection, |fields| {
                                    *fields.reasoning_effort = match effort {
                                        "off" => None,
                                        other => Some(other.to_string()),
                                    };
                                });
                                let runtime = runtime_config_from(
                                    &cfg,
                                    &dir,
                                    provider.as_deref(),
                                    skip,
                                    true,
                                );
                                if runtime.model.is_empty() {
                                    return None;
                                }
                                Some(runtime.agent_config())
                            },
                        );
                        resolver
                    });
                return rustcode::acp::serve_stdio(rustcode::acp::AcpServeOptions {
                    engine: Some(engine),
                    provider_factory: Some(provider_factory),
                    auto_approve,
                    session_config_options,
                    session_model_resolver,
                    session_effort_resolver,
                })
                .await
                .map(|_| 0);
            }
            Commands::Schedule(sub) => {
                // Handled here (not via the generic `other` arm) so the executor's
                // exit code survives: `schedule run` returns 0/1/130 and the OS
                // scheduler keys failure detection on it. The generic arm collapses
                // every Ok to 0, which would report every scheduled run as success.
                HEADLESS_MODE.store(true, Ordering::Relaxed);
                let code = schedule_cmd::handle_schedule(sub).await?;
                return Ok(code);
            }
            other => {
                let result = handle_command(other).await.map(|_| 0);
                return result;
            }
        }
    }

    // Default: start TUI

    let config_path = cli.config.clone().unwrap_or_else(Config::default_path);

    // FIRST-RUN seed for offline / managed deploys (e.g. a government intranet
    // that ships a bundled default config): if the user has no config yet and a
    // `--seed-config <path>` (or `RUSTCODE_SEED_CONFIG` env) source is given, copy
    // it into place once. No-op when a config already exists, so it's safe for the
    // launcher to always pass. Any failure is non-fatal -> normal onboarding.
    let seed_source = cli.seed_config.clone().or_else(|| {
        std::env::var_os("RUSTCODE_SEED_CONFIG")
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
    });
    match rustcode_config::config::Config::seed_user_config(&config_path, seed_source.as_deref()) {
        rustcode_config::config::SeedOutcome::Seeded => {
            if let Some(src) = seed_source.as_deref() {
                eprintln!(
                    "[seed] {}",
                    rustcode_config::i18n::t(rustcode_config::i18n::Msg::CliSeedInitialized {
                        path: &config_path.display().to_string(),
                        source: &src.display().to_string(),
                    })
                );
            }
        }
        rustcode_config::config::SeedOutcome::Invalid(e) => {
            eprintln!(
                "{}",
                rustcode_config::i18n::t(rustcode_config::i18n::Msg::CliSeedInvalid {
                    error: &e.to_string(),
                })
            );
        }
        rustcode_config::config::SeedOutcome::IoError(e) => {
            eprintln!(
                "{}",
                rustcode_config::i18n::t(rustcode_config::i18n::Msg::CliSeedIoError {
                    error: &e.to_string(),
                })
            );
        }
        // AlreadyConfigured / NoSource -> nothing to do, stay quiet.
        _ => {}
    }

    let config_load_start = std::time::Instant::now();
    let (mut config, config_startup_notice) = if config_path.exists() {
        match Config::load_with_diagnostics(&config_path) {
            Ok((config, warnings)) if warnings.is_empty() => (config, None),
            Ok((config, warnings)) => {
                let warning_list = warnings
                    .iter()
                    .map(|warning| format!("  - {warning}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                let notice =
                    rustcode_config::i18n::t(rustcode_config::i18n::Msg::CliConfigLoadWarnings {
                        path: &config_path.display().to_string(),
                        warnings: &warning_list,
                    })
                    .into_owned();
                eprintln!("{notice}");
                (config, Some(notice))
            }
            Err(error) => {
                let notice =
                    rustcode_config::i18n::t(rustcode_config::i18n::Msg::CliConfigLoadFailed {
                        path: &config_path.display().to_string(),
                        error: &error.to_string(),
                    })
                    .into_owned();
                eprintln!("{notice}");
                (Config::default(), Some(notice))
            }
        }
    } else {
        // No config yet -- TUI Welcome screen will guide first-run setup
        (Config::default(), None)
    };
    tracing::info!(
        target: "rustcode::startup",
        stage = "config_load",
        elapsed_ms = config_load_start.elapsed().as_millis() as u64,
        total_ms = run_start.elapsed().as_millis() as u64,
        "startup stage completed"
    );
    rustcode_config::proxy::apply_process_proxy_config(&config.network.proxy);

    // ── i18n locale ──
    // Locale was already pre-resolved above (before clap parse) so --help
    // could be localised. Re-resolve with the full config (which may specify
    // a language key) to honour config-over-env priority.
    if config.language.is_some() {
        let locale =
            rustcode_tuix::i18n::resolve_initial_locale(cli.lang.as_deref(), config.language);
        rustcode_tuix::i18n::set_locale(locale);
    }

    // ── i18n brand/OAuth names ──
    // Settle the distribution's brand + OAuth provider display names into the
    // i18n placeholder cache BEFORE any `t()` render, so the first visible
    // string (e.g. the Welcome banner) already shows the configured brand.
    // Idempotent (`OnceLock` keeps the first value); a mid-session `/reload`
    // does NOT flip the brand, matching `theme`'s startup-only semantics.
    rustcode_config::i18n::set_brand(&config.ui.brand_name, &config.ui.oauth_provider_name);

    // ── Plugin marketplace bootstrap + post-upgrade refresh ──
    //
    // Two best-effort hooks (auto-install default skills marketplace
    // on first startup, `git pull` every installed marketplace after a
    // self-upgrade) used to fire here synchronously, blocking the
    // input box for 1-3s on a warm path (and 5-10s on first clone).
    // Both now run as a detached `spawn_blocking` from inside
    // `rustcode_tuix::run` after the skill registry is constructed --
    // see lib.rs near `spawn_plugin_bootstrap`. Newly-installed skills
    // are picked up by a `skill_registry.reload()` + wake pulse the
    // background task fires on completion, so the slash menu refreshes
    // without a restart.

    apply_cli_runtime_overrides(&mut config, cli.provider.as_deref(), cli.model.as_deref());

    let working_dir = resolve_working_dir(cli.dir.clone());

    // Determine if we're running in headless mode BEFORE loading MCP.
    // Headless mode requires MCP tools immediately; TUI can load them in background.
    let is_headless = cli.prompt.is_some() || cli.prompt_file.is_some();

    // Continue the previous session only when the user explicitly opts
    // in via `-c` / `--continue`. Bare `rustcode` starts a fresh
    // session -- no auto-resume, no scrollback replay. Users who want to
    // pick a specific older session can still use `/resume` inside the
    // TUI.
    let resume_session_id = match &resume_selector {
        Some(ResumeSelector::Specific(sel)) => {
            let catalog = rustcode_daemon::legacy_convert::catalog_for_project(&working_dir)?;
            match resolve_in_catalog(&catalog, sel) {
                Some(id) => Some(id),
                None => anyhow::bail!(
                    "{}",
                    rustcode_config::i18n::t(rustcode_config::i18n::Msg::CliResumeNoMatch {
                        selector: &format!("{sel:?}"),
                    })
                ),
            }
        }
        Some(ResumeSelector::Latest) => {
            rustcode_daemon::legacy_convert::catalog_for_project(&working_dir)?
                .into_iter()
                .find(|entry| entry.message_count > 0)
                .map(|entry| entry.id)
        }
        None => None,
    };

    // The native runtime builds its own provider + tools from
    // config, so the `tool_registry`/`tool_context` assembled above are no longer
    // wired to an agent loop; they remain constructed (unchanged lifetime) pending
    // a follow-up cleanup.
    let mut runtime_cfg = runtime_config_from(
        &config,
        &working_dir,
        cli.provider.as_deref(),
        skip_permissions,
        // Interactive (TUI) ⇒ approvals park until answered; headless (`-p`) keeps the
        // fail-closed timeout so an unanswered approval can't park the run forever.
        !is_headless,
    );
    runtime_cfg.next_prompt_suggestions = !is_headless;
    let model_name = runtime_cfg.model.clone();
    // Headless (`-p`) has no onboarding wizard. If no provider/model resolved -- an
    // empty config with no third-party provider, or a `--provider` name that matches
    // nothing -- fail fast with an actionable message instead of building an OpenAI
    // adapter with an empty base_url and dying deep in reqwest with a cryptic
    // "relative URL without a base". This mirrors the TUI's `model.is_empty()`
    // onboarding trigger, turned into a hard error for the non-interactive path.
    if is_headless {
        if let Some(message) =
            headless_missing_provider_message(&runtime_cfg, cli.provider.as_deref())
        {
            anyhow::bail!("{message}");
        }
    }
    let provider_bootstrap = if is_headless {
        rustcode_coding::ProviderBootstrap::Required
    } else {
        interactive_provider_bootstrap(&runtime_cfg)
    };
    let runtime_start = std::time::Instant::now();
    let (native_runtime, native_coding_cfg, continued_session) = spawn_native_cli_runtime(
        &runtime_cfg,
        resume_session_id,
        provider_bootstrap,
        cli.ephemeral,
        cli.no_tools,
        !is_headless,
        // TUI-only: the interactive checkpoint replaces the hard round-cap
        // error. Headless (`-p`) keeps the fail-closed hard error (no picker).
        !is_headless,
        initial_mode,
    )
    .await?;
    // The active session id (fresh or resumed) for the on-exit resume hint,
    // captured before the runtime is moved into the headless/TUI arms below.
    // `None` for an ephemeral run (no persisted session -> nothing to resume).
    let active_session_id: Option<String> = native_runtime.session.as_ref().map(|s| s.id.clone());
    tracing::info!(
        target: "rustcode::startup",
        stage = "runtime_start",
        elapsed_ms = runtime_start.elapsed().as_millis() as u64,
        total_ms = run_start.elapsed().as_millis() as u64,
        "startup stage completed"
    );
    // TUI replay remains a presentation projection during S4; runtime resume above
    // has already converged and loaded the native snapshot under one lease.
    let resume_project_bucket =
        rustcode_capabilities::session::SessionManager::project_hash(&working_dir);
    let session_to_continue = match continued_session
        .as_ref()
        .map(|session| session.id.as_str())
    {
        Some(id) => rustcode_daemon::legacy_convert::load_catalog_session_view_in_project(
            &resume_project_bucket,
            id,
        )?,
        None => None,
    }
    .map(rustcode_tuix::session::Session::from_catalog_view)
    .transpose()?;
    let session_startup_notice = continued_session
        .as_ref()
        .and_then(|session| {
            session
                .forked_from
                .as_deref()
                .map(|source_id| (source_id, session.id.as_str()))
        })
        .map(|(source_id, fork_id)| {
            rustcode_tuix::i18n::t(rustcode_tuix::i18n::Msg::SessionBusyForked {
                source_id,
                fork_id,
            })
            .into_owned()
        });
    let startup_notice = merge_startup_notices(config_startup_notice, session_startup_notice);
    let (mut native_headless_runtime, mut native_tui_runtime) = if is_headless {
        (Some(native_runtime), None)
    } else {
        (None, Some((native_runtime, native_coding_cfg)))
    };

    // Spawner for in-TUI session switches (/session, /bg, disk /resume): each one
    // builds a fresh native runtime from the CURRENT in-process config. A launch-time
    // `--provider` is applied to that config above, so the override remains authoritative
    // for live views and subsequent runtime respawns without being persisted to disk.
    let runtime_spawn_override: rustcode_tuix::RuntimeSpawnOverride = {
        // Capture the bypass flag so in-TUI re-spawns also honor
        // --dangerously-skip-permissions -- not just the launch handle.
        let skip_perms = skip_permissions;
        std::sync::Arc::new(
            move |config: &rustcode_config::config::Config,
                  working_dir: &std::path::Path,
                  session: &rustcode_tuix::session::Session| {
                let mut runtime_cfg = runtime_config_from(
                    config,
                    working_dir,
                    None,
                    skip_perms,
                    // In-TUI re-spawns (/session, /bg, /resume) are always interactive.
                    true,
                );
                // TUI-only: a `max_rounds` hit becomes the interactive
                // continue/stop checkpoint (the render arm lives in the TUI
                // event loop). `spawn_deferred_tui_runtime` is only ever the
                // in-TUI respawn factory, so this is never a headless path.
                runtime_cfg.round_cap_checkpoint = true;
                runtime_cfg.next_prompt_suggestions = true;
                spawn_deferred_tui_runtime(runtime_cfg, session)
            },
        )
    };

    // Resolve effective prompt: --prompt-file reads from disk; -p is inline.
    // clap's conflicts_with ensures `-p` and `--prompt-file` can't both be given.
    let effective_prompt: Option<String> = match (cli.prompt.as_ref(), cli.prompt_file.as_ref()) {
        (Some(p), None) => Some(p.clone()),
        (None, Some(path)) => match std::fs::read_to_string(path) {
            Ok(s) => Some(s),
            Err(e) => {
                eprintln!(
                    "{}",
                    rustcode_config::i18n::t(rustcode_config::i18n::Msg::CliPromptFileReadFailed {
                        path: &path.display().to_string(),
                        error: &e.to_string(),
                    })
                );
                std::process::exit(2);
            }
        },
        (None, None) => None,
        (Some(_), Some(_)) => unreachable!("clap conflicts_with prevents this"),
    };

    // Build the session-scope context: repo_origin, mode.
    // session_id and account_id are managed on the session binding directly via
    // set_session_id() / set_account_id(). Seed account_id from stored auth so
    // events from this session correlate to the user even before any explicit
    // login action this run; login()/logout() update it later as needed.
    // mode: Headless when a prompt is supplied (-p / --prompt-file);
    //       Tui when the user launches the interactive terminal UI.
    let result = async {
        // Headless mode: -p / --prompt-file triggers non-interactive execution.
        let exit_code = if let Some(prompt) = effective_prompt {
            let verbose = cli.verbose || force_verbose;
            let capture = false;
            // Don't `?`-propagate here: an error must still fall through to the
            // end of the session so its Result is returned. Capture the Result
            // and let it bubble up only after the run completes. The run routes
            // through the native runtime handle.
            let notifications_cfg = config.notifications.clone();
            let engine_runtime = native_headless_runtime
                .take()
                .expect("native headless runtime built above");
            let result = match run_native_headless(
                notifications_cfg,
                engine_runtime,
                prompt,
                runtime_cfg.provider_name.clone(),
                runtime_cfg.model.clone(),
                verbose,
                cli.output_format,
                capture,
                working_dir.clone(),
                skip_permissions,
                is_admin,
                false, // strict_unattended=false: preserve -p behaviour exactly
            )
            .await
            {
                Err(e) => Err(e),
                Ok((ec, _captured)) => Ok::<i32, anyhow::Error>(ec),
            };
            // Codex-style discovery hint on STDERR so the piped stdout stays the
            // clean assistant reply. Skipped for ephemeral runs (no session id).
            if let Some(id) = &active_session_id {
                eprintln!("\n{}", resume_hint_line(id, true));
            }
            result
        } else {
            // Fire-and-forget: spawn a setsid'd subprocess to stage the next
            // release if one is out. Detached so a Ctrl+C in this parent doesn't
            // also kill the download -- that was the whole reason "exit and come
            // back" wasn't picking up v_next on short sessions. Only armed when
            // the user hasn't opted out via `auto_update = false` AND we're not
            // running as `rustcode.bak` (backup should stay pinned; see the
            // `is_running_as_backup` guard up top).
            // In distro-pm (HarmonyBrew) builds the package manager owns
            // upgrades, so skip spawning the detached prep process entirely --
            // `prepare_deferred_upgrade` would no-op anyway.
            if config.auto_update
                && !is_running_as_backup()
                && !cli.dev
                && !rustcode_updater::is_package_managed()
                // Skip when the startup synchronous path already checked (and applied,
                // if newer) this launch -- otherwise both fetch `latest.json`. The
                // detached stager stays as the fallback for launches where the sync
                // path was skipped (e.g. RUSTCODE_PLAIN).
                && !SYNC_UPGRADE_CHECKED.load(Ordering::Relaxed)
            {
                spawn_detached_upgrade_prep();
            }

            // Redirect fd 2 -> $RUSTCODE_HOME/stderr.log before the TUI takes
            // ownership of the terminal. NSPasteboard deprecation warnings
            // (arboard clipboard polling, ~1.5 s interval) and any other
            // rogue C-lib stderr writes would otherwise land at the raw-mode
            // cursor position, painting into the input box.
            //
            // Only fires here -- the TUI branch. Headless (-p/--prompt-file)
            // leaves stderr pointing at the real terminal so the user sees
            // actual errors in their shell/CI output.
            redirect_stderr_to_log_file();

            // The runtime task already runs the engine behind this handle; the TUI
            // drives it. In-TUI /session and /resume spawn via
            // runtime_spawn_override.
            let (runtime, coding_cfg) = native_tui_runtime
                .take()
                .expect("native TUI runtime built above");
            let provider_selection = coding_cfg.provider_name.clone();
            let tui_runtime = into_tui_native_runtime(runtime, coding_cfg);
            // Same as the headless arm: don't `?` -- a TUI run that ends in an
            // error must still reach the shutdown/flush below. Ok(()) -> exit 0;
            // the error propagates only after the shutdown flush completes.
            // A running session owns its resolved provider/model. Shared config
            // changes only define the default for sessions opened afterwards;
            // they must not retarget an already-open runtime.
            let provider_selection_mode = rustcode_tuix::ProviderSelectionMode::Pinned;
            tracing::info!(
                target: "rustcode::startup",
                stage = "tui_enter",
                total_ms = run_start.elapsed().as_millis() as u64,
                "handing control to TUI"
            );
            let tui_result = match rustcode_tuix::run(
                config,
                provider_selection,
                model_name,
                provider_selection_mode,
                rustcode_config::ConfigStore::new(config_path.clone()),
                tui_runtime,
                runtime_spawn_override,
                working_dir,
                session_to_continue,
                startup_notice,
                skip_permissions,
                is_admin,
            )
            .await
            {
                Ok(()) => Ok(0),
                Err(e) => Err(e),
            };
            // Codex-style resume hint to STDOUT, after the TUI restored the
            // terminal, so `rustcode resume <id>` is discoverable on exit.
            if let Some(id) = &active_session_id {
                println!("\n{}", resume_hint_line(id, false));
            }
            tui_result
        };

        exit_code
    }
    .await;

    result
}

fn spawn_deferred_tui_runtime(
    cfg: rustcode_coding::CodingRuntimeConfig,
    session: &rustcode_tuix::session::Session,
) -> rustcode_tuix::SpawnedRuntime {
    let session_id = session.id.as_str().to_string();
    let snapshot = session.to_conversation_snapshot();
    // Base agent config for the VL preprocessor's one-off provider builds,
    // derived from this runtime's config before `cfg` is moved into the spawn.
    let vl_base = cfg.agent_config();
    let (native_control, mut events, runtime_state) =
        rustcode_daemon::spawn_native_runtime_for_session_deferred_with_preprocessor(
            cfg,
            session_id.clone(),
            snapshot,
            Some(std::sync::Arc::new(
                crate::vision::VlImagePreprocessor::new(
                    rustcode_daemon::coding_provider_factory(),
                    vl_base,
                ),
            )),
        );
    let control = rustcode_tuix::RuntimeControl::deferred(native_control, runtime_state);
    let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(async move {
        while let Some(event) = events.recv().await {
            if event_tx
                .send(rustcode_tuix::RuntimeEventPayload::SequencedNative(event))
                .is_err()
            {
                break;
            }
        }
    });
    rustcode_tuix::SpawnedRuntime {
        endpoint: rustcode_tuix::RuntimeEndpoint { native: control },
        event_rx,
        session_id: Some(session_id),
    }
}

fn into_tui_native_runtime(
    runtime: rustcode_coding::CodingRuntime,
    _coding_cfg: rustcode_coding::CodingAgentConfig,
) -> rustcode_tuix::SpawnedRuntime {
    let session_id = runtime.session.as_ref().map(|session| session.id.clone());
    let rustcode_coding::CodingRuntime {
        handle,
        mut events,
        task,
        ..
    } = runtime;
    let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
    let control =
        rustcode_tuix::RuntimeControl::ready_with_event_tx(handle.clone(), event_tx.clone());
    let control_for_events = control.clone();
    tokio::spawn(async move {
        while let Some(event) = events.recv().await {
            if event_tx
                .send(rustcode_tuix::RuntimeEventPayload::SequencedNative(event))
                .is_err()
            {
                break;
            }
        }
        let _ = task.await;
        control_for_events.detach_delivery_event_tx();
    });
    rustcode_tuix::SpawnedRuntime {
        endpoint: rustcode_tuix::RuntimeEndpoint { native: control },
        event_rx,
        session_id,
    }
}

/// On macOS the NSPasteboard runtime prints deprecation warnings to
/// stderr when arboard calls into AppKit (via clipboard polling for
/// the "ctrl+v to paste image" hint). In raw mode, stderr shares the
/// TTY with the TUI paint stream, so those warnings paint into the
/// input box at whatever cursor row happens to be active. Other libs
/// (LSP, MCP shells) can leak the same way.
///
/// Redirect fd 2 to `$RUSTCODE_HOME/stderr.log` once we know we're
/// entering interactive TUI mode. plain / headless / piped paths
/// don't call this -- they want stderr to reach the terminal so the
/// user sees real errors.
///
/// Best-effort: if the home dir can't be created or the file can't
/// be opened, do nothing and let stderr leak (the original bug); we
/// don't want to take down rustcode startup because logging failed.
#[cfg(unix)]
fn redirect_stderr_to_log_file() {
    use std::os::unix::io::AsRawFd;
    // Both halves come from `distribution`, which is what `HOME_ENV`'s doc
    // comment asks of the resolvers it enumerates -- this is one of them. The
    // fallback is rarely taken (`bootstrap_home` has normally set the variable
    // already), and that is exactly why a literal here is a liability: the one
    // path that reaches it is the one nobody exercises.
    let home_env = std::env::var_os(rustcode_config::distribution::HOME_ENV);
    let Some(home) = home_env
        .map(std::path::PathBuf::from)
        .or_else(|| dirs::home_dir().map(|h| h.join(rustcode_config::distribution::HOME_DIR_NAME)))
    else {
        return;
    };
    if std::fs::create_dir_all(&home).is_err() {
        return;
    }
    let Ok(file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(home.join("stderr.log"))
    else {
        return;
    };
    // Write a session marker so users can see in stderr.log where
    // each rustcode session starts -- helps separate one run's noise
    // from another's when grepping for actual problems.
    // Use epoch seconds (std::time only -- no chrono dep needed).
    let epoch_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let marker = format!("\n--- rustcode session start (unix={epoch_secs}) ---\n");
    let _ = std::io::Write::write_all(&mut std::io::BufWriter::new(&file), marker.as_bytes());
    // SAFETY: dup2 swaps the file descriptor table entry for fd 2
    // to point at `file`'s underlying fd. This is a standard, safe
    // operation; the worst case (dup2 fails) is the redirect doesn't
    // happen and we log nothing -- same as the no-redirect baseline.
    unsafe {
        libc::dup2(file.as_raw_fd(), libc::STDERR_FILENO);
    }
    // Intentionally keep `file` alive via the dup2 -- the kernel
    // holds a reference to the underlying inode, so even after
    // `file` is dropped, fd 2 stays pointing at the same file.
    // No need to std::mem::forget.
}

#[cfg(not(unix))]
fn redirect_stderr_to_log_file() {
    // Windows: NSPasteboard is mac-only; arboard on Windows uses
    // OpenClipboard which doesn't NSLog. Not a known leak path.
    // No-op for now; revisit if a similar Windows issue surfaces.
}

/// The persistent tracing log path: `<config_dir>/logs/rustcode.log`. Pure so the
/// join rule is unit-testable; the config dir is resolved by `Config::config_dir()`
/// (which is `RUSTCODE_HOME`- AND sudo-aware via `real_home_dir`), so the log lands
/// next to config/sessions instead of diverging under `sudo` -- plain `dirs::home_dir()`
/// there points at root's home, where the user would never find the log.
fn rustcode_log_path(config_dir: std::path::PathBuf) -> std::path::PathBuf {
    config_dir.join("logs").join("rustcode.log")
}

/// The default `RUST_LOG` directive when the env var is unset: `info` for everything,
/// with the chatty transport crates pinned to `warn` so the log stays about rustcode.
const DEFAULT_LOG_DIRECTIVES: &str =
    "info,hyper=warn,hyper_util=warn,h2=warn,rustls=warn,reqwest=warn,tower=warn,mio=warn";

/// Roll size cap for the tracing log. Above this the live file is rotated to a single
/// `.old` generation, bounding on-disk usage at ~2x this (an always-on `info` log
/// would otherwise append forever across every session).
const LOG_ROTATE_BYTES: u64 = 5 * 1024 * 1024;

/// If the log already exceeds [`LOG_ROTATE_BYTES`], move it aside to `<path>.old`
/// (single generation) so the live file restarts small. Best-effort; errors ignored.
fn rotate_log_if_large(path: &std::path::Path) {
    if std::fs::metadata(path).map(|m| m.len()).unwrap_or(0) > LOG_ROTATE_BYTES {
        let mut old = path.as_os_str().to_owned();
        old.push(".old"); // rustcode.log -> rustcode.log.old
        let _ = std::fs::rename(path, std::path::PathBuf::from(old));
    }
}

/// Install a global tracing subscriber that writes to `<config_dir>/logs/rustcode.log`.
///
/// The whole workspace emits `tracing::` diagnostics but historically installed NO
/// subscriber, so every line (including the `rustcode-label:` middleware trace) went
/// to the no-op dispatcher and vanished. This wires them to a file.
///
/// FILE-ONLY BY DESIGN: the TUI owns the terminal, and the stderr redirect only runs
/// in the detached-daemon path -- writing tracing output to real stderr would corrupt
/// the interactive display. So we always write to our own file handle, never stderr.
///
/// Fail-open: any error (can't create dir/file, subscriber already set) leaves the
/// process running with logging simply disabled. Called once, early.
fn init_file_logging() {
    use std::io::Write as _;
    let path = rustcode_log_path(Config::config_dir());
    if let Some(parent) = path.parent() {
        if std::fs::create_dir_all(parent).is_err() {
            return;
        }
    }
    rotate_log_if_large(&path);
    let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    else {
        return;
    };
    // Session marker so users can tell one run's lines from another's when grepping.
    let epoch_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let _ = writeln!(file, "--- rustcode session start (unix={epoch_secs}) ---");

    let filter = std::env::var("RUST_LOG")
        .ok()
        .and_then(|v| tracing_subscriber::EnvFilter::try_new(v).ok())
        .unwrap_or_else(|| tracing_subscriber::EnvFilter::new(DEFAULT_LOG_DIRECTIVES));

    // `Mutex<File>` is tracing-subscriber's OWN built-in `MakeWriter`: its
    // `MutexGuardWriter` holds the lock for the WHOLE formatted event, so concurrent
    // traces from tokio workers can't interleave mid-line (a per-`write()` re-lock
    // would). No custom writer needed.
    let _ = tracing_subscriber::fmt()
        .with_ansi(false) // file, not a terminal
        .with_env_filter(filter)
        .with_writer(std::sync::Mutex::new(file))
        .try_init(); // Err only if a subscriber is already set -- fine, ignore.
}

/// Apply launch-time provider/model overrides to the process-owned config.
///
/// The runtime already resolves `--provider` directly, but TUI respawns and live views
/// read `config.default_provider`. Keeping those two sources different makes the footer
/// correct while synchronized WebUI tabs expose and reload the wrong provider. This is
/// deliberately in-memory only; persistence remains owned by `/model` and WebUI settings.
fn apply_cli_runtime_overrides(
    config: &mut rustcode_config::config::Config,
    provider_override: Option<&str>,
    model_override: Option<&str>,
) {
    let provider_name = provider_override
        .map(str::to_string)
        // Fall back to the canonical active selection (new-schema `default_model`
        // then legacy `default_provider`), not `default_provider` alone.
        .or_else(|| config.effective_model_selection())
        .unwrap_or_default();
    // Route the optional `--model` override to whichever schema holds the
    // selection (legacy `[providers.*]` or new-schema `[models.*]`); bail if the
    // selection is unknown so `--provider bogus` is a no-op as before.
    if let Some(model) = model_override {
        if let Some(p) = config.providers.get_mut(&provider_name) {
            p.model = model.to_string();
        } else if let Some(m) = config.models.get_mut(&provider_name) {
            m.model = model.to_string();
        } else {
            return;
        }
    } else if !config.selection_exists(&provider_name) {
        return;
    }
    if provider_override.is_some() {
        // `default_model` is canonical (`effective_model_selection` prefers it);
        // sync the legacy field too so a new-schema override takes effect.
        config.default_model = Some(provider_name.clone());
        config.default_provider = provider_name;
    }
}

/// Derive the coding runtime config from the current config + working dir.
/// Shared by the initial runtime and the in-TUI runtime-spawn override so
/// both resolve the provider identically.
///
/// `provider_override` is the `--provider` flag: it must flow through the SAME
/// `active_provider` resolution (honor the override, fall back when the default
/// points to a deleted section). Reading `default_provider` directly silently
/// ignored `--provider`, so a headless `--provider X` run picked the config
/// default instead of X.
pub(crate) fn runtime_config_from(
    config: &rustcode_config::config::Config,
    working_dir: &std::path::Path,
    provider_override: Option<&str>,
    dangerously_skip_permissions: bool,
    interactive: bool,
) -> rustcode_coding::CodingRuntimeConfig {
    let mut runtime = rustcode_coding::CodingRuntimeConfig::from_config(
        config,
        working_dir,
        provider_override,
        dangerously_skip_permissions,
        interactive,
    );
    // The process locale has already resolved CLI `--lang` > config > env.
    runtime.preferred_language = Some(rustcode_tuix::i18n::current_locale());
    runtime
}

/// Select the interactive startup mode through the same unified provider/model
/// resolution result that will be assembled by the runtime.
///
/// Inspecting the raw config here would either see only the legacy schema or
/// repeat resolution with subtly different fallback behavior. `runtime_cfg`
/// already owns the exact resolved provider/model for this spawn.
fn interactive_provider_bootstrap(
    runtime_cfg: &rustcode_coding::CodingRuntimeConfig,
) -> rustcode_coding::ProviderBootstrap {
    if !runtime_cfg.model.is_empty() {
        rustcode_coding::ProviderBootstrap::RecoverAuthentication
    } else {
        rustcode_coding::ProviderBootstrap::Unavailable(
            rustcode_coding::ProviderUnavailableReason::NotConfigured,
        )
    }
}

/// Headless pre-flight: returns an actionable error message when no provider/model
/// resolved (empty config, or a `--provider` name that matches nothing). Returns
/// `None` when a provider resolved and the run should proceed.
///
/// The non-interactive path cannot fall back to the onboarding wizard, so the
/// `model.is_empty()` condition that sends the TUI to onboarding becomes a hard,
/// clearly-worded error here -- avoiding a cryptic reqwest "relative URL without a
/// base" from an OpenAI adapter built with an empty `base_url`.
pub(crate) fn headless_missing_provider_message(
    runtime_cfg: &rustcode_coding::CodingRuntimeConfig,
    requested: Option<&str>,
) -> Option<String> {
    if !runtime_cfg.model.is_empty() {
        return None;
    }
    let path = rustcode_config::config::Config::default_path();
    let path = path.display().to_string();
    Some(match requested {
        Some(name) => {
            rustcode_config::i18n::t(rustcode_config::i18n::Msg::CliHeadlessNoProviderNamed {
                name,
                path: &path,
            })
            .into_owned()
        }
        None => rustcode_config::i18n::t(rustcode_config::i18n::Msg::CliHeadlessNoProvider {
            path: &path,
        })
        .into_owned(),
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn spawn_native_cli_runtime(
    cfg: &rustcode_coding::CodingRuntimeConfig,
    resume_session_id: Option<String>,
    bootstrap: rustcode_coding::ProviderBootstrap,
    ephemeral: bool,
    no_tools: bool,
    fork_on_session_in_use: bool,
    // TUI-only opt-in: turn a `max_rounds` hit into the interactive
    // continue/stop checkpoint. The checkpoint render arm lives in the TUI
    // event loop, so headless (`-p`) callers pass `false` -- otherwise the
    // kernel would emit a checkpoint Request with no requester and fail-closed.
    round_cap_checkpoint: bool,
    // Initial run-time execution mode derived from `--permission-mode` (or
    // `--dangerously-skip-permissions`). Applied to the live runtime right
    // after startup; `Build` is the fail-closed default.
    initial_mode: rustcode_coding::RuntimeMode,
) -> anyhow::Result<(
    rustcode_coding::CodingRuntime,
    rustcode_coding::CodingAgentConfig,
    Option<ContinuedCliSession>,
)> {
    let mut agent = cfg.agent_config();
    agent.round_cap_checkpoint = round_cap_checkpoint;
    let (session, imported_lease, continued_session) = if ephemeral {
        (rustcode_coding::SessionMode::Disabled, None, None)
    } else {
        match resume_session_id {
            Some(id) => {
                let manager =
                    rustcode_capabilities::session::SessionManager::for_project(&agent.working_dir);
                match manager.acquire_lease(&id) {
                    Ok(lease) => {
                        rustcode_daemon::legacy_convert::converge_session(&manager, &lease)?;
                        (
                            rustcode_coding::SessionMode::Resume(id.clone()),
                            Some(lease),
                            Some(ContinuedCliSession {
                                id,
                                forked_from: None,
                            }),
                        )
                    }
                    Err(error) if should_fork_busy_continue(fork_on_session_in_use, &error) => {
                        let fork_id = uuid::Uuid::new_v4().to_string();
                        let (forked, lease) = manager.fork_native_session(
                            &id,
                            &fork_id,
                            rustcode_capabilities::session::now_ms(),
                        )?;
                        (
                            rustcode_coding::SessionMode::Resume(forked.meta.id.clone()),
                            Some(lease),
                            Some(ContinuedCliSession {
                                id: forked.meta.id,
                                forked_from: Some(id),
                            }),
                        )
                    }
                    Err(error) => return Err(error.into()),
                }
            }
            None => (rustcode_coding::SessionMode::Fresh, None, None),
        }
    };
    // External-agent subagents (Claude Code / Codex) from `[[subagent.external]]`.
    // Interactive TUI ⇒ dangerous (`bypass`) modes are allowed to be configured
    // (each risky call is still approval-gated); headless/daemon paths pass none.
    let external_subagents = cfg
        .subagent_config
        .as_ref()
        .map(|c| rustcode_coding::parts::resolve_external_subagents(&c.subagent, true))
        .unwrap_or_default();
    let prepare = rustcode_coding::PrepareOptions {
        subagents: rustcode_coding::SubagentPolicy::Enabled,
        session,
        tools: !no_tools,
        skill_dirs: no_tools.then(Vec::new),
        plugin_skill_dirs: if no_tools {
            Vec::new()
        } else {
            rustcode_daemon::gather_plugin_skill_dirs_for(&cfg.working_dir)
        },
        mcp: cfg.mcp && !no_tools,
        external_subagents: if no_tools {
            Vec::new()
        } else {
            external_subagents
        },
        memory: !no_tools,
        web: !no_tools,
        review: !no_tools,
        request_user_input: !no_tools,
        ..rustcode_coding::PrepareOptions::default()
    };
    let start = rustcode_coding::CodingRuntimeStart {
        agent: agent.clone(),
        prepare,
        provider_factory: rustcode_daemon::coding_provider_factory(),
        plugin_hooks: rustcode_daemon::installed_plugin_hook_source(),
        // Restore the TUI's VL image recognition (dropped when the legacy
        // bridge was retired): convert images to text for non-vision models
        // inside the async turn, so it never blocks the UI. See `vision`.
        image_preprocessor: Some(std::sync::Arc::new(
            crate::vision::VlImagePreprocessor::new(
                rustcode_daemon::coding_provider_factory(),
                agent.clone(),
            ),
        )),
    };
    let runtime = match imported_lease {
        Some(lease) => {
            rustcode_coding::CodingRuntime::start_with_session_lease(start, bootstrap, lease).await
        }
        None => rustcode_coding::CodingRuntime::start_with_bootstrap(start, bootstrap).await,
    }
    .map_err(anyhow::Error::new)?;
    runtime
        .handle
        .set_mode(initial_mode)
        .await
        .map_err(anyhow::Error::new)?;
    Ok((runtime, agent, continued_session))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContinuedCliSession {
    id: String,
    forked_from: Option<String>,
}

fn should_fork_busy_continue(
    fork_on_session_in_use: bool,
    error: &rustcode_capabilities::session::SessionStoreError,
) -> bool {
    fork_on_session_in_use
        && matches!(
            error,
            rustcode_capabilities::session::SessionStoreError::SessionInUse { .. }
        )
}

/// Decide whether to auto-approve a headless approval request.
///
/// A request only reaches this point when a gate (BashWorkspaceGate /
/// ApprovalMiddleware) already escalated the tool call -- i.e. it is NOT
/// trivially safe.  `-p` (skip_permissions) blanket-approves bash; scheduled
/// runs (strict_unattended=true) refuse everything because no human is present
/// to vet a destructive or out-of-workspace command.
pub(crate) fn headless_auto_approve(
    strict_unattended: bool,
    skip_permissions: bool,
    tool: &str,
) -> bool {
    if strict_unattended {
        return false; // scheduled: deny everything that was escalated to approval
    }
    skip_permissions || tool == "bash" // -p: current behaviour unchanged
}

fn write_headless_json_event(event: &headless_json::HeadlessEvent) -> io::Result<()> {
    let stdout = io::stdout();
    let mut output = stdout.lock();
    output.write_all(&headless_json::line(event)?)?;
    output.flush()
}

#[allow(clippy::too_many_arguments)] // 12 params thread the headless session config into the runtime; callers are few and stable
pub(crate) async fn run_native_headless(
    notifications_cfg: rustcode_config::config::NotificationConfig,
    runtime: rustcode_coding::CodingRuntime,
    prompt: String,
    provider_name: String,
    model_name: String,
    verbose: bool,
    output_format: HeadlessOutputFormat,
    capture: bool,
    working_dir: PathBuf,
    skip_permissions: bool,
    is_admin: bool,
    strict_unattended: bool,
) -> Result<(i32, Option<String>)> {
    use rustcode_capabilities::tools::{ApprovalRequest, ApprovalResponse, APPROVAL_KIND};
    use rustcode_coding::{CodingRuntimeEvent, TurnCompletion, UserInput};
    use rustcode_config::i18n::{t, Msg};
    use rustcode_kernel::event::{AgentEvent as KernelEvent, StopReason};

    HEADLESS_MODE.store(true, Ordering::Relaxed);
    if skip_permissions {
        eprintln!(
            "{}",
            rustcode_config::i18n::t(rustcode_config::i18n::Msg::BypassWarningHeadless)
        );
    }
    if is_admin {
        eprintln!(
            "{}",
            rustcode_config::i18n::t(rustcode_config::i18n::Msg::AdminWarningHeadless)
        );
    }

    let rustcode_coding::CodingRuntime {
        handle,
        mut events,
        task,
        ..
    } = runtime;
    let jsonl = output_format == HeadlessOutputFormat::Jsonl;
    let captured = capture.then(String::new);
    if jsonl {
        write_headless_json_event(&headless_json::HeadlessEvent::RunStarted {
            schema_version: 1,
            provider: provider_name,
            model: model_name,
        })?;
    }
    if let Err(error) = handle
        .wait_mcp_ready(rustcode_capabilities::mcp::CONNECT_TIMEOUT)
        .await
    {
        if jsonl {
            let message = error.to_string();
            write_headless_json_event(&headless_json::HeadlessEvent::Error {
                message: message.clone(),
                http_status: None,
                code: Some("mcp_not_ready".into()),
                retryable: None,
            })?;
            write_headless_json_event(&headless_json::HeadlessEvent::RunFailed {
                exit_code: 1,
                message,
            })?;
            let _ = handle.shutdown().await;
            let _ = task.await;
            return Ok((1, captured));
        }
        let _ = handle.shutdown().await;
        let _ = task.await;
        return Err(anyhow::Error::new(error));
    }
    let started = std::time::Instant::now();
    if let Err(error) = handle.submit(UserInput::from(prompt)).await {
        if jsonl {
            let message = error.to_string();
            write_headless_json_event(&headless_json::HeadlessEvent::Error {
                message: message.clone(),
                http_status: None,
                code: Some("submit_failed".into()),
                retryable: None,
            })?;
            write_headless_json_event(&headless_json::HeadlessEvent::RunFailed {
                exit_code: 1,
                message,
            })?;
            let _ = handle.shutdown().await;
            let _ = task.await;
            return Ok((1, captured));
        }
        let _ = handle.shutdown().await;
        let _ = task.await;
        return Err(anyhow::Error::new(error));
    }
    let mut captured = captured;
    let mut last_text_ended_with_newline = true;
    let mut tool_calls = 0usize;
    let mut total_tokens = 0usize;
    let mut prompt_tokens = 0usize;
    let mut completion_tokens = 0usize;
    let mut cached_tokens = 0usize;
    let mut rounds = 0usize;
    let mut ttft_ms: Option<u64> = None;
    let mut had_denial = false;
    let mut exit_code = 0;
    let mut saw_turn_terminal = false;
    let mut thinking_line_open = false;

    fn close_native_thinking(open: &mut bool) {
        let mut output = String::new();
        close_thinking_chunk(&mut output, open);
        if !output.is_empty() {
            eprint!("{output}");
            let _ = io::stderr().flush();
        }
    }

    while let Some(envelope) = events.recv().await {
        match envelope.event {
            CodingRuntimeEvent::Agent(KernelEvent::TextDelta(text)) => {
                ttft_ms.get_or_insert_with(|| started.elapsed().as_millis() as u64);
                close_native_thinking(&mut thinking_line_open);
                if !text.is_empty() {
                    last_text_ended_with_newline = text.ends_with('\n');
                }
                if let Some(buffer) = captured.as_mut() {
                    buffer.push_str(&text);
                }
                if jsonl {
                    write_headless_json_event(&headless_json::HeadlessEvent::MessageDelta {
                        text,
                    })?;
                } else {
                    print!("{text}");
                    io::stdout().flush()?;
                }
            }
            CodingRuntimeEvent::Agent(KernelEvent::Reasoning(text)) => {
                ttft_ms.get_or_insert_with(|| started.elapsed().as_millis() as u64);
                if jsonl {
                    write_headless_json_event(&headless_json::HeadlessEvent::ReasoningDelta {
                        text,
                    })?;
                } else if verbose {
                    let mut output = String::new();
                    format_thinking_chunk(&mut output, &mut thinking_line_open, &text);
                    eprint!("{output}");
                    let _ = io::stderr().flush();
                }
            }
            CodingRuntimeEvent::Agent(KernelEvent::ToolStarted { call }) => {
                ttft_ms.get_or_insert_with(|| started.elapsed().as_millis() as u64);
                close_native_thinking(&mut thinking_line_open);
                tool_calls += 1;
                if jsonl {
                    write_headless_json_event(&headless_json::HeadlessEvent::ToolStarted {
                        id: call.id,
                        name: call.name,
                        arguments: call.arguments,
                    })?;
                } else if verbose {
                    eprintln!(
                        "[tool-> {}] {}",
                        call.name,
                        truncate_log_line(&call.arguments, 120)
                    );
                }
            }
            CodingRuntimeEvent::Agent(KernelEvent::ToolResult { result }) => {
                close_native_thinking(&mut thinking_line_open);
                if jsonl {
                    write_headless_json_event(&headless_json::HeadlessEvent::ToolCompleted {
                        id: result.call_id,
                        content: result.content,
                        is_error: result.is_error,
                    })?;
                } else if verbose {
                    eprintln!(
                        "[tool← {}] {} chars",
                        if result.is_error { "error" } else { "ok" },
                        format_verbose_tool_chunk(&result.content).chars().count()
                    );
                }
            }
            CodingRuntimeEvent::Agent(KernelEvent::Usage(meta)) => {
                rounds += 1;
                total_tokens = total_tokens
                    .saturating_add((meta.tokens.prompt + meta.tokens.completion) as usize);
                prompt_tokens = prompt_tokens.saturating_add(meta.tokens.prompt as usize);
                completion_tokens =
                    completion_tokens.saturating_add(meta.tokens.completion as usize);
                cached_tokens = cached_tokens.saturating_add(meta.tokens.cached as usize);
                if jsonl {
                    write_headless_json_event(&headless_json::HeadlessEvent::Usage {
                        round: meta.round,
                        turn_id: meta.turn_id,
                        request_id: meta.request_id,
                        prompt_tokens: meta.tokens.prompt,
                        completion_tokens: meta.tokens.completion,
                        cached_tokens: meta.tokens.cached,
                        elapsed_ms: meta.elapsed_ms,
                        reasoning_elapsed_ms: meta.reasoning_elapsed_ms,
                    })?;
                } else if verbose {
                    eprintln!(
                        "[tokens] prompt={} completion={} cached={}",
                        meta.tokens.prompt, meta.tokens.completion, meta.tokens.cached
                    );
                }
            }
            CodingRuntimeEvent::Agent(KernelEvent::Error {
                message,
                http_status,
                code,
                retryable,
            }) => {
                close_native_thinking(&mut thinking_line_open);
                if jsonl {
                    write_headless_json_event(&headless_json::HeadlessEvent::Error {
                        message,
                        http_status,
                        code,
                        retryable,
                    })?;
                } else {
                    eprintln!("[error] {message}");
                }
                exit_code = 1;
            }
            CodingRuntimeEvent::Agent(KernelEvent::StreamRecovery {
                attempt,
                max_attempts,
                recovered,
            }) if jsonl => {
                write_headless_json_event(&headless_json::HeadlessEvent::Retry {
                    kind: "stream_recovery".into(),
                    attempt,
                    max_attempts,
                    recovered: Some(recovered),
                    backoff_secs: None,
                    reason: None,
                })?;
            }
            CodingRuntimeEvent::Agent(KernelEvent::OutputTruncationRecovery {
                attempt,
                max_attempts,
            }) if jsonl => {
                write_headless_json_event(&headless_json::HeadlessEvent::Retry {
                    kind: "output_truncation".into(),
                    attempt,
                    max_attempts,
                    recovered: None,
                    backoff_secs: None,
                    reason: None,
                })?;
            }
            CodingRuntimeEvent::Agent(KernelEvent::ProviderRetry {
                attempt,
                max_attempts,
                backoff_secs,
                reason,
            }) => {
                if jsonl {
                    write_headless_json_event(&headless_json::HeadlessEvent::Retry {
                        kind: "provider_open".into(),
                        attempt,
                        max_attempts,
                        recovered: None,
                        backoff_secs: Some(backoff_secs),
                        reason: Some(reason.as_token().to_string()),
                    })?;
                } else {
                    eprintln!(
                        "{}",
                        t(Msg::CliHeadlessProviderRetry {
                            reason: &rustcode_coding::retry_reason_label(reason),
                            backoff_secs,
                            attempt,
                            max_attempts,
                        })
                    );
                }
            }
            CodingRuntimeEvent::Agent(KernelEvent::Warning(message))
            | CodingRuntimeEvent::ControllerWarning(message)
            | CodingRuntimeEvent::PersistenceWarning(message) => {
                if jsonl {
                    write_headless_json_event(&headless_json::HeadlessEvent::Warning { message })?;
                } else {
                    eprintln!("[warning] {message}")
                }
            }
            CodingRuntimeEvent::Agent(KernelEvent::RateLimited {
                reset_at_display,
                reset_label,
                secs_until_reset,
                auto_resuming,
                server_message,
            }) => {
                if jsonl {
                    write_headless_json_event(&headless_json::HeadlessEvent::RateLimit {
                        reset_at: reset_at_display,
                        reset_label,
                        seconds_until_reset: secs_until_reset,
                        auto_resuming,
                        server_message,
                    })?;
                    continue;
                }
                let is_quota_window = !reset_at_display.is_empty() || !reset_label.is_empty();
                if auto_resuming {
                    eprintln!(
                        "[rate-limited] {}",
                        t(Msg::CliHeadlessRateAutoResume {
                            secs: secs_until_reset.unwrap_or(0),
                        })
                    );
                } else if !is_quota_window {
                    let reason = match server_message.as_deref() {
                        Some(message) if !message.trim().is_empty() => {
                            format!(" -- {}", message.trim())
                        }
                        _ => String::new(),
                    };
                    match secs_until_reset {
                        Some(seconds) => eprintln!(
                            "[rate-limited] {}",
                            t(Msg::CliHeadlessRateRetry {
                                reason: &reason,
                                secs: seconds,
                            })
                        ),
                        None => eprintln!(
                            "[rate-limited] {}",
                            t(Msg::CliHeadlessRatePaused { reason: &reason })
                        ),
                    }
                } else if !reset_at_display.is_empty() {
                    eprintln!(
                        "[rate-limited] {}",
                        t(Msg::CliHeadlessRateWindowResetAt {
                            reset_at: &reset_at_display,
                        })
                    );
                } else if let Some(seconds) = secs_until_reset {
                    eprintln!(
                        "[rate-limited] {}",
                        t(Msg::CliHeadlessRateWindowSecs { secs: seconds })
                    );
                } else {
                    eprintln!("[rate-limited] {}", t(Msg::CliHeadlessRateWindowPaused));
                }
            }
            CodingRuntimeEvent::Request(request) => {
                let response = if request.kind == APPROVAL_KIND {
                    serde_json::from_value::<ApprovalRequest>(request.payload)
                        .ok()
                        .map(|approval| {
                            if headless_auto_approve(
                                strict_unattended,
                                skip_permissions,
                                &approval.tool,
                            ) {
                                eprintln!(
                                    "[headless] {}",
                                    t(Msg::CliHeadlessAutoApproved {
                                        tool: &approval.tool,
                                    })
                                );
                                ApprovalResponse::allow()
                            } else {
                                had_denial = true;
                                eprintln!(
                                    "[denied] {}",
                                    t(Msg::CliHeadlessDenied {
                                        tool: &approval.tool,
                                    })
                                );
                                ApprovalResponse::deny()
                            }
                        })
                } else {
                    None
                };
                let value = response
                    .and_then(|response| serde_json::to_value(response).ok())
                    .unwrap_or(serde_json::Value::Null);
                let _ = handle.respond(request.id, value).await;
            }
            CodingRuntimeEvent::CompactionFinished {
                completion: rustcode_coding::runtime::CompactionCompletion::Completed(outcome),
            } if outcome.committed => {
                eprintln!(
                    "[compact] {}",
                    rustcode_config::i18n::format_compaction_mark(
                        outcome.removed_messages,
                        outcome.estimated_tokens_before,
                        outcome.estimated_tokens_after
                    )
                );
            }
            CodingRuntimeEvent::CompactionFinished { .. } => {}
            CodingRuntimeEvent::TurnFinished(completion) => {
                saw_turn_terminal = true;
                close_native_thinking(&mut thinking_line_open);
                let reason = match &completion {
                    TurnCompletion::Completed { reason, .. }
                    | TurnCompletion::SnapshotUnavailable { reason, .. } => *reason,
                };
                if !jsonl && !last_text_ended_with_newline {
                    println!();
                    io::stdout().flush()?;
                }
                rustcode_capabilities::notify::notify_turn_finished(
                    &notifications_cfg,
                    rustcode_capabilities::notify::TurnNotification {
                        duration: started.elapsed(),
                        turn_count: rounds,
                        tool_call_count: tool_calls,
                        total_tokens: Some(total_tokens),
                        stop_reason: headless_completion_notify_reason(&completion),
                        working_dir: Some(&working_dir),
                    },
                );
                if verbose {
                    eprintln!(
                        "[done] {:.1}s tokens={} turns={} tool_calls={}{}",
                        started.elapsed().as_secs_f64(),
                        rustcode_config::i18n::fmt_tokens(total_tokens),
                        rounds,
                        tool_calls,
                        match &completion {
                            TurnCompletion::Completed { .. } if reason == StopReason::Stopped =>
                                String::new(),
                            TurnCompletion::Completed { .. } => {
                                format!(" stopped={reason:?}")
                            }
                            TurnCompletion::SnapshotUnavailable { error, .. } => format!(
                                " completion=SnapshotUnavailable reason={reason:?} error={}",
                                error.message
                            ),
                        }
                    );
                }
                exit_code = headless_completion_exit_code(&completion, exit_code);
                exit_code = headless_denial_exit_code(exit_code, had_denial);
                if jsonl {
                    let snapshot_error = match &completion {
                        TurnCompletion::Completed { .. } => None,
                        TurnCompletion::SnapshotUnavailable { error, .. } => {
                            Some(error.message.clone())
                        }
                    };
                    write_headless_json_event(&headless_json::HeadlessEvent::TurnCompleted {
                        stop_reason: reason,
                        exit_code,
                        duration_ms: started.elapsed().as_millis() as u64,
                        rounds,
                        tool_calls,
                        total_tokens,
                        prompt_tokens,
                        completion_tokens,
                        cached_tokens,
                        cache_hit_rate: if prompt_tokens == 0 {
                            None
                        } else {
                            Some(cached_tokens as f64 / prompt_tokens as f64)
                        },
                        ttft_ms,
                        snapshot_error,
                    })?;
                }
                break;
            }
            CodingRuntimeEvent::RuntimeStopped(_) => {
                exit_code = exit_code.max(1);
                break;
            }
            _ => {}
        }
    }
    if !saw_turn_terminal {
        exit_code = exit_code.max(1);
        if jsonl {
            write_headless_json_event(&headless_json::HeadlessEvent::RunFailed {
                exit_code,
                message: "runtime stopped before a turn terminal".into(),
            })?;
        }
    }
    let _ = handle.shutdown().await;
    let _ = task.await;
    Ok((exit_code, captured))
}

/// Drive `rustcode_capabilities::setup::run` end-to-end and return the CLI exit code
/// (0 on success, 1 on any setup error). `setup::run` is synchronous; we
/// run it directly since `Commands::Setup` already runs outside the TUI loop.
fn run_setup_command(force: bool) -> i32 {
    use rustcode_capabilities::setup;

    let project_root = match std::env::current_dir() {
        Ok(p) => p,
        Err(e) => {
            eprintln!(
                "{}",
                rustcode_config::i18n::t(rustcode_config::i18n::Msg::CliSetupCwdError {
                    error: &e.to_string(),
                })
            );
            return 1;
        }
    };
    let mut opts = setup::RunOptions::new(project_root);
    opts.force = force;

    match setup::run(opts) {
        Ok(report) => {
            println!("{}", report.render_cli());
            0
        }
        Err(e) => {
            eprintln!(
                "{}",
                rustcode_config::i18n::t(rustcode_config::i18n::Msg::CliSetupFailed {
                    error: &e.to_string(),
                })
            );
            1
        }
    }
}

/// Handle the subcommands that run to completion and exit; the ones `run()`
/// intercepts inline (Resume, Daemon, Webui, Setup) never reach this match.
async fn handle_command(cmd: Commands) -> Result<()> {
    // Subcommands never enter TUI, so tell the panic hook to skip terminal
    // cleanup -- otherwise `disable_raw_mode` panics on Windows with
    // "initial console mode not set" because raw mode was never enabled.
    HEADLESS_MODE.store(true, Ordering::Relaxed);

    match cmd {
        Commands::Resume { .. } => {
            // Resume falls through to the TUI/headless launch in run() before
            // handle_command is reached; kept defensive.
            unreachable!("Resume is handled inline in run() before handle_command")
        }
        Commands::Status => {
            // There is no stored platform identity to print, so `status`
            // reports where credentials actually come from instead of
            // inventing a sign-in state.
            println!(
                "{}",
                rustcode_config::i18n::t(rustcode_config::i18n::Msg::CliStatusHintNeutral)
            );
            Ok(())
        }
        Commands::Upgrade { force } => run_upgrade_cli(force).await,
        Commands::Rollback => run_rollback_cli(),
        Commands::Uninstall {
            yes,
            purge,
            keep_data,
            dry_run,
        } => uninstall::run(uninstall::Args {
            yes,
            purge,
            keep_data,
            dry_run,
        }),
        Commands::Daemon { .. } => {
            unreachable!("Daemon is handled inline in run() before handle_command")
        }
        Commands::Webui { .. } => {
            unreachable!("Webui is handled inline in run() before handle_command")
        }
        Commands::Setup { .. } => {
            unreachable!("Setup is handled inline in run() before handle_command")
        }
        Commands::Wiki(_) => {
            unreachable!("Wiki is handled inline in run() before handle_command")
        }
        Commands::Plugin(sub) => handle_plugin_cli(sub),
        Commands::Mcp(McpCli::Add {
            name,
            command,
            global,
            dir,
        }) => {
            let base = resolve_working_dir(dir);
            let path = if global {
                Config::config_dir().join("mcp.json")
            } else {
                base.join(".mcp.json")
            };
            let program = command
                .first()
                .expect("clap ensures at least one command token")
                .clone();
            let args: Vec<String> = command.into_iter().skip(1).collect();
            merge_stdio_mcp_server_into_json_file(&path, &name, &program, &args)?;
            println!(
                "{}",
                rustcode_config::i18n::t(rustcode_config::i18n::Msg::CliMcpAdded {
                    name: &format!("{name:?}"),
                    path: &path.display().to_string(),
                    program: &program,
                    args: args.len(),
                })
            );
            Ok(())
        }
        Commands::Mcp(McpCli::AddOauth {
            url,
            name,
            global,
            dir,
        }) => {
            let base = resolve_working_dir(dir);
            let path = if global {
                Config::config_dir().join("mcp.json")
            } else {
                base.join(".mcp.json")
            };
            // Default the server key to the URL host so the command is one-shot;
            // fall back to a neutral key if the URL cannot be parsed.
            let name = name
                .filter(|n| !n.trim().is_empty())
                .or_else(|| {
                    url::Url::parse(&url)
                        .ok()
                        .and_then(|u| u.host_str().map(|h| h.to_string()))
                        .filter(|h| !h.is_empty())
                })
                .unwrap_or_else(|| "remote".to_string());
            // Empty provider => provider-neutral OAuth (generic RFC-8414 metadata
            // discovery at login). No vendor is baked into the default command.
            merge_http_oauth_mcp_server_into_json_file(&path, &name, &url, "")?;
            println!(
                "{}",
                rustcode_config::i18n::t(rustcode_config::i18n::Msg::CliMcpAddedOauth {
                    name: &format!("{name:?}"),
                    path: &path.display().to_string(),
                    url: &url,
                })
            );
            Ok(())
        }
        Commands::Mcp(McpCli::AddGithubOauth { name, global, dir }) => {
            let base = resolve_working_dir(dir);
            let path = if global {
                Config::config_dir().join("mcp.json")
            } else {
                base.join(".mcp.json")
            };
            merge_http_oauth_mcp_server_into_json_file(
                &path,
                &name,
                "https://api.githubcopilot.com/mcp/",
                "github",
            )?;
            println!(
                "{}",
                rustcode_config::i18n::t(rustcode_config::i18n::Msg::CliMcpAddedGithub {
                    name: &format!("{name:?}"),
                    path: &path.display().to_string(),
                })
            );
            Ok(())
        }
        Commands::Mcp(McpCli::Login {
            name,
            provider,
            client_id,
            client_secret_env,
            scopes,
        }) => {
            let configs = load_mcp_config(&std::env::current_dir()?)?;
            let server = configs
                .into_iter()
                .find(|config| config.name == name)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "{}",
                        rustcode_config::i18n::t(
                            rustcode_config::i18n::Msg::CliMcpServerNotFound {
                                name: &format!("{name:?}"),
                            }
                        )
                    )
                })?;
            let is_github_server = matches!(
                &server.config,
                McpTransportConfig::Http {
                    auth: Some(McpHttpAuthConfig::OAuth(auth)),
                    ..
                } if auth.provider.as_deref() == Some("github")
            );
            // GitHub's classic OAuth flow is a per-server interop path, driven by
            // the server's own config (or an explicit `--provider github` hint) --
            // never a vendor default for a bare `mcp login`.
            let github_mode = is_github_server || provider.as_deref() == Some("github");
            let client_id = client_id.or_else(|| {
                if github_mode {
                    std::env::var("RUSTCODE_GITHUB_MCP_CLIENT_ID").ok()
                } else {
                    None
                }
            });
            let token = login_mcp_oauth(
                &server,
                McpOAuthLoginOptions {
                    client_id,
                    client_secret_env,
                    scopes,
                },
            )?;
            println!(
                "{}",
                rustcode_config::i18n::t(rustcode_config::i18n::Msg::CliMcpLoginSaved {
                    provider: &token.provider,
                    name: &format!("{name:?}"),
                    scopes: token.scopes.len(),
                })
            );
            Ok(())
        }
        Commands::Mcp(McpCli::Logout { name }) => {
            let removed = McpTokenStore::default().delete_token(&name)?;
            let name_dbg = format!("{name:?}");
            if removed {
                println!(
                    "{}",
                    rustcode_config::i18n::t(rustcode_config::i18n::Msg::CliMcpLogoutRemoved {
                        name: &name_dbg,
                    })
                );
            } else {
                println!(
                    "{}",
                    rustcode_config::i18n::t(rustcode_config::i18n::Msg::CliMcpLogoutNotFound {
                        name: &name_dbg
                    })
                );
            }
            Ok(())
        }
        Commands::Acp => {
            unreachable!("Acp is handled inline in run() before handle_command")
        }
        Commands::Completion(_) => {
            unreachable!("completion is handled before runtime startup")
        }
        Commands::Ide(sub) => match sub {
            IdeCli::List => run_ide_list(),
            IdeCli::Install { ide, all } => run_ide_install(all, ide.as_deref()),
        },
        Commands::Hooks(subcmd) => handle_hooks(subcmd).await,
        Commands::Schedule(_) => {
            unreachable!("Schedule is handled inline in run() so its exit code survives")
        }
        Commands::Askpass { .. } => {
            unreachable!("__askpass is handled early in run() before handle_command")
        }
    }
}

/// Handle hooks subcommands.
///
/// Reports and tests the CC-compatible external hooks the LIVE runtime actually
/// runs (`rustcode_capabilities::cc_hooks`: `$RUSTCODE_HOME/hooks.json` +
/// `<project>/.hooks.json`). The legacy v1 engine (TOML script / webhook / built-in
/// hooks) no longer fires at runtime, so it is intentionally not surfaced here.
async fn handle_hooks(cmd: HookCommands) -> Result<()> {
    use rustcode_capabilities::cc_hooks::{
        global_hooks_path, load_hooks_config, project_hooks_path, run_hook_for_test, HookEvent,
    };
    use rustcode_config::i18n::{t, Msg};
    HEADLESS_MODE.store(true, Ordering::Relaxed);

    let cwd = std::env::current_dir().unwrap_or_default();

    // CC hook event -> its display / payload name.
    fn event_name(e: HookEvent) -> &'static str {
        match e {
            HookEvent::PreToolUse => "PreToolUse",
            HookEvent::PostToolUse => "PostToolUse",
            HookEvent::PostToolUseFailure => "PostToolUseFailure",
            HookEvent::SessionStart => "SessionStart",
            HookEvent::SessionEnd => "SessionEnd",
            HookEvent::UserPromptSubmit => "UserPromptSubmit",
            HookEvent::Stop => "Stop",
            HookEvent::StopFailure => "StopFailure",
        }
    }

    // Display the EXACT files cc_hooks loads -- via cc_hooks' own resolver, not
    // `Config::config_dir()` (which is sudo-aware and would diverge from what the
    // hook loader actually reads under `sudo`, turning the diagnostic into a lie).
    let project_hooks = project_hooks_path(&cwd);
    let print_paths = || {
        match global_hooks_path() {
            Some(g) => {
                let mark = if g.exists() { "[+]" } else { "[x]" };
                println!(
                    "  {} {}",
                    mark,
                    t(Msg::CliHooksPathGlobal {
                        path: &g.display().to_string()
                    })
                );
            }
            None => println!("  [x] {}", t(Msg::CliHooksPathNoHome)),
        }
        let p = if project_hooks.exists() { "[+]" } else { "[x]" };
        println!(
            "  {} {}",
            p,
            t(Msg::CliHooksPathProject {
                path: &project_hooks.display().to_string()
            })
        );
    };

    match cmd {
        HookCommands::List => {
            let hooks = load_hooks_config(&cwd);
            println!("{}", t(Msg::CliHooksLoadedHeader));
            println!("─────────────────────────────────────────────");
            if hooks.is_empty() {
                println!("{}", t(Msg::CliHooksNone));
            } else {
                let mut by_event: std::collections::BTreeMap<&str, usize> =
                    std::collections::BTreeMap::new();
                for h in &hooks {
                    *by_event.entry(event_name(h.event)).or_insert(0) += 1;
                }
                println!(
                    "  {:<20} {:>5}",
                    t(Msg::CliHooksTableEvent),
                    t(Msg::CliHooksTableCount)
                );
                println!("  {:<20} {:>5}", "─".repeat(20), "─".repeat(5));
                for (ev, n) in &by_event {
                    println!("  {:<20} {:>5}", ev, n);
                }
                println!("  {:<20} {:>5}", "─".repeat(20), "─".repeat(5));
                println!("  {:<20} {:>5}", t(Msg::CliHooksTableTotal), hooks.len());
            }
            println!("{}", t(Msg::CliHooksConfigFiles));
            println!("─────────────────────────────────────────────");
            print_paths();
            println!();

            let untrusted: Vec<_> =
                rustcode_capabilities::plugin::installed_plugin_hook_trust_status()
                    .into_iter()
                    .filter(|s| !s.trusted)
                    .collect();
            if !untrusted.is_empty() {
                println!("{}", t(Msg::CliHooksUntrustedHeader));
                for s in &untrusted {
                    println!(
                        "{}",
                        t(Msg::CliHooksUntrustedRow {
                            plugin: &s.plugin,
                            count: s.hook_count,
                            events: &s.events.join(", "),
                        })
                    );
                }
                println!();
            }
            Ok(())
        }
        HookCommands::Test { name } => {
            let hooks = load_hooks_config(&cwd);
            // cc_hooks hooks carry no name -- match by event name or a command substring.
            let found = hooks.iter().find(|h| {
                event_name(h.event).eq_ignore_ascii_case(&name) || h.command.contains(&name)
            });
            match found {
                None => {
                    println!("{}", t(Msg::CliHooksTestNotFound { name: &name }));
                    if hooks.is_empty() {
                        println!("{}", t(Msg::CliHooksTestNoneLoaded));
                    } else {
                        println!("{}", t(Msg::CliHooksTestAvailable));
                        for h in &hooks {
                            println!("  [-] {:<16} {}", event_name(h.event), h.command);
                        }
                    }
                }
                Some(hook) => {
                    println!(
                        "{}",
                        t(Msg::CliHooksTesting {
                            event: event_name(hook.event)
                        })
                    );
                    println!(
                        "{}",
                        t(Msg::CliHooksFieldCommand {
                            command: &hook.command
                        })
                    );
                    println!(
                        "{}",
                        t(Msg::CliHooksFieldTimeout {
                            ms: hook.timeout_ms
                        })
                    );
                    if let Some(ref m) = hook.matcher {
                        println!("{}", t(Msg::CliHooksFieldMatcher { matcher: m }));
                    }
                    println!();
                    // CC stdin payload -- event-shaped to MATCH what the live runtime pipes
                    // (see cc_hooks lifecycle methods): only tool events carry tool fields,
                    // PostToolUse carries `tool_response`, UserPromptSubmit carries `prompt`.
                    let sid = "test-session-0000";
                    let cwd_s = cwd.display().to_string();
                    let payload = match hook.event {
                        HookEvent::PreToolUse => serde_json::json!({
                            "session_id": sid, "hook_event_name": "PreToolUse", "cwd": cwd_s,
                            "tool_name": "bash", "tool_input": { "command": "echo hello" },
                        }),
                        HookEvent::PostToolUse => serde_json::json!({
                            "session_id": sid, "hook_event_name": "PostToolUse", "cwd": cwd_s,
                            "tool_name": "bash", "tool_response": "hello\n",
                        }),
                        HookEvent::PostToolUseFailure => serde_json::json!({
                            "session_id": sid, "hook_event_name": "PostToolUseFailure", "cwd": cwd_s,
                            "tool_name": "bash", "tool_response": "command failed\n",
                        }),
                        HookEvent::UserPromptSubmit => serde_json::json!({
                            "session_id": sid, "hook_event_name": "UserPromptSubmit",
                            "cwd": cwd_s, "prompt": "test prompt",
                        }),
                        HookEvent::SessionStart => serde_json::json!({
                            "session_id": sid, "hook_event_name": "SessionStart", "cwd": cwd_s,
                        }),
                        HookEvent::SessionEnd => serde_json::json!({
                            "session_id": sid, "hook_event_name": "SessionEnd", "cwd": cwd_s,
                        }),
                        HookEvent::Stop => serde_json::json!({
                            "session_id": sid, "transcript_path": null,
                            "hook_event_name": "Stop", "stop_hook_active": false,
                            "stop_reason": "Stopped", "cwd": cwd_s,
                        }),
                        HookEvent::StopFailure => serde_json::json!({
                            "session_id": sid, "transcript_path": null,
                            "hook_event_name": "StopFailure", "stop_hook_active": false,
                            "stop_reason": "ProviderError", "cwd": cwd_s,
                        }),
                    };
                    let start = std::time::Instant::now();
                    match run_hook_for_test(hook, &payload).await {
                        Some(out) => {
                            println!("{}", t(Msg::CliHooksResultHeader));
                            println!(
                                "{}",
                                t(Msg::CliHooksDuration {
                                    duration: &format!("{:?}", start.elapsed())
                                })
                            );
                            // CC exit-code contract: 0 = ok, 2 = DELIBERATE block (not a
                            // failure), other/signal = the hook broke.
                            let label: &str = match out.exit_code {
                                Some(0) => "[+] SUCCESS",
                                Some(2) => "[!] BLOCK",
                                Some(_) | None => "[x] FAILURE",
                            };
                            let detail = match out.exit_code {
                                Some(0) => t(Msg::CliHooksStatusSuccess).into_owned(),
                                Some(2) => t(Msg::CliHooksStatusBlock).into_owned(),
                                Some(c) => t(Msg::CliHooksStatusExitCode { code: c }).into_owned(),
                                None => t(Msg::CliHooksStatusSignal).into_owned(),
                            };
                            println!(
                                "{}",
                                t(Msg::CliHooksFieldStatus {
                                    label,
                                    detail: &detail
                                })
                            );
                            if !out.stdout.is_empty() {
                                println!("  ── stdout ──");
                                for l in out.stdout.trim_end().lines() {
                                    println!("  │ {}", l);
                                }
                            }
                            if !out.stderr.is_empty() {
                                println!("  ── stderr ──");
                                for l in out.stderr.trim_end().lines() {
                                    println!("  │ {}", l);
                                }
                            }
                        }
                        None => {
                            println!("{}", t(Msg::CliHooksResultHeader));
                            println!(
                                "{}",
                                t(Msg::CliHooksDidNotComplete {
                                    ms: hook.timeout_ms
                                })
                            );
                        }
                    }
                }
            }
            Ok(())
        }
        HookCommands::Paths => {
            println!("{}", t(Msg::CliHooksPathsHeader));
            println!("─────────────────────────────────────────────");
            print_paths();
            println!("{}", t(Msg::CliHooksDocsHeader));
            println!("─────────────────────────────────────────────");
            println!("{}", t(Msg::CliHooksDocsEntry));
            println!();
            Ok(())
        }
    }
}

/// Dispatch `rustcode plugin ...` subcommands. Each branch calls the same
/// `rustcode_capabilities::plugin::*` API the TUI's `/plugin` slash command uses, so
/// CLI installs and TUI installs share state under `$RUSTCODE_HOME/plugins/`.
fn handle_plugin_cli(sub: PluginCli) -> Result<()> {
    use rustcode_capabilities::plugin::{installer, marketplace};
    use rustcode_config::i18n::{t, Msg};
    /// `{prefix}: {error}` with the localized anyhow context prefix.
    fn ctx(prefix: &str, e: impl std::fmt::Display) -> anyhow::Error {
        anyhow::anyhow!("{prefix}: {e:#}")
    }
    match sub {
        PluginCli::Marketplace(MarketplaceCli::Add { url }) => {
            let info = marketplace::add_marketplace(&url)
                .map_err(|e| ctx(&t(Msg::CliPluginErrAddMp), e))?;
            println!(
                "{}",
                t(Msg::CliPluginMpAdded {
                    name: &info.name,
                    commit: &info.git_commit[..7.min(info.git_commit.len())],
                    plugins: info.plugins.len(),
                })
            );
            Ok(())
        }
        PluginCli::Marketplace(MarketplaceCli::Remove { name }) => {
            marketplace::remove_marketplace(&name)
                .map_err(|e| ctx(&t(Msg::CliPluginErrRemoveMp), e))?;
            println!("{}", t(Msg::CliPluginMpRemoved { name: &name }));
            Ok(())
        }
        PluginCli::Marketplace(MarketplaceCli::Update { name }) => {
            let info = marketplace::update_marketplace(&name)
                .map_err(|e| ctx(&t(Msg::CliPluginErrUpdateMp), e))?;
            println!(
                "{}",
                t(Msg::CliPluginMpUpdated {
                    name: &info.name,
                    commit: &info.git_commit[..7.min(info.git_commit.len())],
                })
            );
            Ok(())
        }
        PluginCli::Marketplace(MarketplaceCli::List) => {
            let items = marketplace::list_marketplaces()?;
            if items.is_empty() {
                println!("{}", t(Msg::CliPluginMpNone));
            } else {
                for m in items {
                    println!(
                        "{}",
                        t(Msg::CliPluginMpRow {
                            name: &m.name,
                            source: &m.source,
                            commit: &m.git_commit[..7.min(m.git_commit.len())],
                            plugins: m.plugins.len(),
                        })
                    );
                }
            }
            Ok(())
        }
        PluginCli::Install { spec } => {
            let installed_plugin_name: String;
            match parse_plugin_spec(&spec)? {
                PluginSpec::Qualified {
                    plugin,
                    marketplace: mp,
                } => {
                    let info = installer::install(
                        &plugin,
                        &mp,
                        rustcode_capabilities::plugin::InstallScope::User,
                    )
                    .map_err(|e| ctx(&t(Msg::CliPluginErrInstall), e))?;
                    println!(
                        "{}",
                        t(Msg::CliPluginInstalled {
                            plugin: &info.plugin,
                            marketplace: &info.marketplace,
                        })
                    );
                    installed_plugin_name = info.plugin;
                }
                PluginSpec::Bare { plugin } => {
                    match installer::resolve_plugin_marketplace(&plugin)
                        .map_err(|e| ctx(&t(Msg::CliPluginErrResolve), e))?
                    {
                        matches if matches.len() == 1 => {
                            let m = &matches[0];
                            let mp = m.marketplace.clone();
                            let resolved_plugin = m.plugin.clone();
                            let info = installer::install(
                                &resolved_plugin,
                                &mp,
                                rustcode_capabilities::plugin::InstallScope::User,
                            )
                            .map_err(|e| ctx(&t(Msg::CliPluginErrInstall), e))?;
                            println!(
                                "{}",
                                t(Msg::CliPluginInstalled {
                                    plugin: &info.plugin,
                                    marketplace: &info.marketplace,
                                })
                            );
                            installed_plugin_name = info.plugin;
                        }
                        matches if matches.len() > 1 => {
                            let mut list = String::new();
                            for m in &matches {
                                list.push_str(&format!(
                                    "  rustcode plugin install {}@{}\n",
                                    m.plugin, m.marketplace
                                ));
                            }
                            anyhow::bail!(
                                "{}",
                                t(Msg::CliPluginInstallAmbiguous {
                                    plugin: &plugin,
                                    list: &list,
                                })
                                .trim_end()
                            );
                        }
                        _ => {
                            anyhow::bail!("{}", t(Msg::CliPluginNotFound { plugin: &plugin }));
                        }
                    }
                }
            }
            // Surface untrusted hooks for the freshly-installed plugin only -- they
            // will NOT run until the user trusts them (loaded-code trust gate).
            // Filtered by `info.plugin` (the canonical plugin name returned by the
            // installer) so pre-existing untrusted plugins don't produce spurious output.
            for s in rustcode_capabilities::plugin::installed_plugin_hook_trust_status() {
                if !s.trusted && s.plugin == installed_plugin_name {
                    println!(
                        "{}",
                        t(Msg::CliPluginUntrustedNotice {
                            plugin: &s.plugin,
                            count: s.hook_count,
                            events: &s.events.join(", "),
                        })
                    );
                }
            }
            Ok(())
        }
        PluginCli::Uninstall { spec } => {
            match parse_plugin_spec(&spec)? {
                PluginSpec::Qualified {
                    plugin,
                    marketplace: mp,
                } => {
                    installer::uninstall(
                        &plugin,
                        &mp,
                        rustcode_capabilities::plugin::InstallScope::User,
                    )
                    .map_err(|e| ctx(&t(Msg::CliPluginErrUninstall), e))?;
                    println!(
                        "{}",
                        t(Msg::CliPluginUninstalled {
                            plugin: &plugin,
                            marketplace: &mp,
                        })
                    );
                }
                PluginSpec::Bare { plugin } => {
                    let installed = installer::list_installed().unwrap_or_default();
                    let matches: Vec<_> = installed
                        .into_iter()
                        .filter(|p| {
                            p.plugin == plugin
                                || p.plugin
                                    == rustcode_capabilities::plugin::marketplace::sanitize_name(
                                        &plugin,
                                    )
                        })
                        .collect();
                    match matches.len() {
                        0 => {
                            anyhow::bail!("{}", t(Msg::CliPluginNotInstalled { plugin: &plugin }))
                        }
                        1 => {
                            let p = &matches[0];
                            installer::uninstall(&p.plugin, &p.marketplace, p.scope.clone())
                                .map_err(|e| ctx(&t(Msg::CliPluginErrUninstall), e))?;
                            println!(
                                "{}",
                                t(Msg::CliPluginUninstalled {
                                    plugin: &p.plugin,
                                    marketplace: &p.marketplace,
                                })
                            );
                        }
                        _ => {
                            let mut list = String::new();
                            for p in &matches {
                                list.push_str(&format!(
                                    "  {}@{} ({})\n",
                                    p.plugin, p.marketplace, p.scope
                                ));
                            }
                            anyhow::bail!(
                                "{}",
                                t(Msg::CliPluginUninstallAmbiguous {
                                    plugin: &plugin,
                                    list: &list,
                                })
                            );
                        }
                    }
                }
            }
            Ok(())
        }
        PluginCli::Trust { name } => {
            let status = rustcode_capabilities::plugin::installed_plugin_hook_trust_status();
            let matches: Vec<_> = if name.contains('@') {
                status.iter().filter(|s| s.plugin_id == name).collect()
            } else {
                status.iter().filter(|s| s.plugin == name).collect()
            };
            match matches.as_slice() {
                [] => anyhow::bail!("{}", t(Msg::CliPluginNoHooks { name: &name })),
                [s] => {
                    rustcode_capabilities::plugin::hook_trust::trust(&s.plugin_id, &s.hash)?;
                    println!(
                        "{}",
                        t(Msg::CliPluginTrusted {
                            count: s.hook_count,
                            name: &name,
                            events: &s.events.join(", "),
                        })
                    );
                }
                many => {
                    let mut list = String::new();
                    for s in many {
                        list.push_str(&format!("  {}@{} ({})\n", s.plugin, s.marketplace, s.scope));
                    }
                    anyhow::bail!(
                        "{}",
                        t(Msg::CliPluginTrustAmbiguous {
                            name: &name,
                            list: &list,
                        })
                    );
                }
            }
            Ok(())
        }
        PluginCli::Untrust { name } => {
            let status = rustcode_capabilities::plugin::installed_plugin_hook_trust_status();
            let matches: Vec<_> = if name.contains('@') {
                status.iter().filter(|s| s.plugin_id == name).collect()
            } else {
                status.iter().filter(|s| s.plugin == name).collect()
            };
            match matches.as_slice() {
                [] => anyhow::bail!("{}", t(Msg::CliPluginNoHooks { name: &name })),
                [s] => {
                    rustcode_capabilities::plugin::hook_trust::untrust(&s.plugin_id)?;
                    println!("{}", t(Msg::CliPluginUntrusted { name: &name }));
                }
                many => {
                    let mut list = String::new();
                    for s in many {
                        list.push_str(&format!("  {}@{} ({})\n", s.plugin, s.marketplace, s.scope));
                    }
                    anyhow::bail!(
                        "{}",
                        t(Msg::CliPluginTrustAmbiguous {
                            name: &name,
                            list: &list,
                        })
                    );
                }
            }
            Ok(())
        }
        PluginCli::List => {
            let items = installer::list_installed()?;
            if items.is_empty() {
                println!("{}", t(Msg::CliPluginNone));
            } else {
                for p in items {
                    println!("  {}@{}  {}", p.plugin, p.marketplace, p.plugin_dir);
                }
            }
            Ok(())
        }
    }
}

/// Parsed argument for `rustcode plugin install/uninstall`.
/// Supports both `plugin@marketplace` (fully qualified) and bare
/// `plugin` (resolved across all marketplaces).
enum PluginSpec {
    /// Explicit `plugin@marketplace` -- use as-is.
    Qualified { plugin: String, marketplace: String },
    /// Bare plugin name -- needs marketplace resolution.
    Bare { plugin: String },
}

/// Parse a plugin spec string. Accepts both `plugin@marketplace` and
/// bare `plugin` (resolved across all registered marketplaces).
fn parse_plugin_spec(s: &str) -> Result<PluginSpec> {
    use rustcode_config::i18n::{t, Msg};
    let s = s.trim();
    if s.is_empty() {
        anyhow::bail!("{}", t(Msg::CliPluginSpecEmpty));
    }
    if let Some((plugin, mp)) = s.split_once('@') {
        if plugin.trim().is_empty() || mp.trim().is_empty() {
            anyhow::bail!("{}", t(Msg::CliPluginSpecPartEmpty { spec: s }));
        }
        Ok(PluginSpec::Qualified {
            plugin: plugin.trim().to_string(),
            marketplace: mp.trim().to_string(),
        })
    } else {
        Ok(PluginSpec::Bare {
            plugin: s.to_string(),
        })
    }
}

/// Extract current + latest version from the updater's `ALREADY_LATEST`
/// error body. The shape is a fixed English contract owned by
/// `rustcode-updater` (the TUI's `parse_already_latest_versions` parses
/// the same string):
///   `already on {current} (latest is {latest}). Pass --force to reinstall.`
/// Returns `None` if the format ever drifts; the caller uses "?"
/// placeholders so the localized sentence still renders cleanly.
fn parse_already_latest_versions(s: &str) -> Option<(&str, &str)> {
    let after_on = s.strip_prefix("already on ")?;
    let (current, rest) = after_on.split_once(" (latest is ")?;
    let latest = rest.strip_suffix(". Pass --force to reinstall.")?;
    let latest = latest.strip_suffix(')')?;
    Some((current, latest))
}

/// CLI (non-TUI) upgrade driver -- prints progress to stdout and
/// success/error messages the same way `install.sh` does.
async fn run_upgrade_cli(force: bool) -> Result<()> {
    use rustcode_config::i18n::{t, Msg};
    use rustcode_updater::{self as self_update, UpgradeEvent, ALREADY_LATEST};

    let current = format!("v{}", env!("CARGO_PKG_VERSION"));
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<UpgradeEvent>();

    // Spawn the driver; consume events on the main task so stdout
    // writes don't interleave unpredictably with the upgrade work.
    let driver = tokio::spawn(self_update::run_upgrade(current.clone(), force, tx));

    let mut last_pct: i32 = -1;
    while let Some(ev) = rx.recv().await {
        match ev {
            UpgradeEvent::ManifestFetched { version } => {
                println!("{}", t(Msg::CliUpgradeLatest { version: &version }));
            }
            UpgradeEvent::Downloading { bytes, total } => {
                // Debounce to whole percents so we don't spam stdout --
                // piping the CLI through `tee` with 10k updates is no
                // fun for anyone.
                let pct = ((bytes * 100).checked_div(total).unwrap_or(0)) as i32;
                if pct != last_pct {
                    print!(
                        "{}",
                        t(Msg::CliUpgradeDownloadProgress { pct, bytes, total })
                    );
                    io::stdout().flush().ok();
                    last_pct = pct;
                }
            }
            UpgradeEvent::Verifying => {
                println!("{}", t(Msg::CliUpgradeVerifyingSha));
            }
            UpgradeEvent::Replacing => {
                println!("{}", t(Msg::CliUpgradeReplacingBinary));
            }
            UpgradeEvent::Done {
                version,
                backup,
                exe: _,
            } => {
                let backup = backup.display().to_string();
                println!(
                    "{}",
                    t(Msg::CliUpgradeCmdDone {
                        version: &version,
                        backup: &backup
                    })
                );
                println!("{}", t(Msg::CliUpgradeStartNewHint));
            }
            // CLI path never spawns a rollback via this channel and the
            // driver below translates errors into the returned Result
            // (not a Failed event) -- these arms exist only to keep the
            // match exhaustive if the TUI path ever reuses this code.
            UpgradeEvent::Failed(msg) => {
                if msg.contains(rustcode_updater::PACKAGE_MANAGED) {
                    println!(
                        "\n{}",
                        rustcode_config::i18n::t(rustcode_config::i18n::Msg::UpgradePackageManaged)
                    );
                } else {
                    eprintln!("{}", t(Msg::CliUpgradeCmdFailed { error: &msg }));
                }
            }
            UpgradeEvent::RolledBack { exe, backup } => {
                let exe = exe.display().to_string();
                let backup = backup.display().to_string();
                println!(
                    "{}",
                    t(Msg::CliRollbackCmdDone {
                        exe: &exe,
                        backup: &backup
                    })
                );
            }
        }
    }

    match driver.await {
        Ok(Ok(_summary)) => Ok(()),
        Ok(Err(e)) => {
            let msg = format!("{:#}", e);
            if msg.contains(rustcode_updater::PACKAGE_MANAGED) {
                println!(
                    "{}",
                    rustcode_config::i18n::t(rustcode_config::i18n::Msg::UpgradePackageManaged)
                );
                Ok(())
            } else if msg.contains(ALREADY_LATEST) {
                // Friendly path -- not an error, just "nothing to do".
                // The updater's body is fixed-format English (the TUI
                // parses the same shape in parse_already_latest_versions);
                // extract the versions and render the localized sentence
                // instead of pasting English prose.
                let body = msg.replace(&format!("{}: ", ALREADY_LATEST), "");
                let (current, latest) = parse_already_latest_versions(&body).unwrap_or(("?", "?"));
                println!("  {}", t(Msg::UpgradeAlreadyLatest { current, latest }));
                Ok(())
            } else {
                Err(e)
            }
        }
        Err(e) => Err(anyhow::anyhow!(
            "{}",
            t(Msg::CliUpgradePanicked {
                error: &e.to_string()
            })
        )),
    }
}

fn run_rollback_cli() -> Result<()> {
    let summary = match rustcode_updater::run_rollback() {
        Ok(s) => s,
        Err(e) => {
            let msg = format!("{:#}", e);
            if msg.contains(rustcode_updater::PACKAGE_MANAGED) {
                println!(
                    "{}",
                    rustcode_config::i18n::t(rustcode_config::i18n::Msg::UpgradePackageManaged)
                );
                return Ok(());
            }
            return Err(e);
        }
    };
    use rustcode_config::i18n::{t, Msg};
    let current = summary.exe.display().to_string();
    let saved = summary.backup.display().to_string();
    println!(
        "{}",
        t(Msg::CliRollbackCmdDoneTwo {
            current: &current,
            saved: &saved
        })
    );
    println!("{}", t(Msg::CliRollbackStartHint));
    Ok(())
}

/// Guard so the two-link panic-hook chain (installed hook + install-aware
/// hook that chains to it) writes the crash log exactly once.
static CRASH_LOGGED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Synchronously append a panic's location + message + backtrace to
/// `$RUSTCODE_HOME/logs/panic.log`, then `flush` + `sync_all` so the bytes are
/// durable **before** the hook returns and the runtime calls `abort()`
/// (`panic = "abort"` in the release profile).
///
/// Why this exists: with `abort`, stderr is lost when the terminal window
/// closes (Windows resize crash), and any other reporting path similarly
/// fails to flush. A blocking, fsync'd file write is the only sink that
/// survives a crash. Best-effort: every step swallows errors so the hook
/// never re-panics.
fn write_crash_log(info: &std::panic::PanicHookInfo<'_>) {
    use std::io::Write;
    use std::sync::atomic::Ordering;
    if CRASH_LOGGED.swap(true, Ordering::SeqCst) {
        return;
    }
    // Same `logs/` dir as [`rustcode_log_path`], so it must resolve the same
    // way: that one goes through `Config::config_dir()`, this one used to hard-
    // code `~/.rustcode`, and with `$RUSTCODE_HOME` set the two split into
    // different trees -- `rustcode.log` where the user configured it, the crash
    // report somewhere they never look.
    //
    // This also drops the old give-up-if-no-home arm: with nothing resolvable
    // the report now lands in a cwd-relative dir, matching every other path the
    // process writes, rather than being silently discarded.
    let dir = rustcode_config::config::Config::config_dir().join("logs");
    let _ = std::fs::create_dir_all(&dir);
    let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("panic.log"))
    else {
        return;
    };
    let loc = info
        .location()
        .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
        .unwrap_or_else(|| "unknown".into());
    let msg = info
        .payload()
        .downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| info.payload().downcast_ref::<String>().cloned())
        .unwrap_or_default();
    let thread = std::thread::current()
        .name()
        .unwrap_or("unknown")
        .to_string();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // `force_capture` ignores RUST_BACKTRACE and always resolves frames.
    let bt = std::backtrace::Backtrace::force_capture();
    let _ = writeln!(f, "\n==== RustCode panic @ unix:{ts} thread:{thread} ====");
    let _ = writeln!(f, "location: {loc}");
    let _ = writeln!(f, "message : {msg}");
    let _ = writeln!(f, "backtrace:\n{bt}");
    let _ = f.flush();
    let _ = f.sync_all();
}

/// Install a minimal panic hook that writes the crash to stderr and the
/// durable panic log (no network, no telemetry). Replaces the default
/// std hook so crashes are reported cleanly even before any subsystem
/// is initialized.
fn install_crash_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        write_crash_log(info);
        restore_terminal_if_tui();
        eprintln!(
            "{}",
            rustcode_config::i18n::t(rustcode_config::i18n::Msg::CliCrashHeader {
                info: &info.to_string()
            })
        );
        if let Some(location) = info.location() {
            eprintln!(
                "  at {}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            );
        }
        eprintln!(
            "{}",
            rustcode_config::i18n::t(rustcode_config::i18n::Msg::CliCrashReport)
        );
    }));
}

#[cfg(test)]
mod tests {
    use super::{
        apply_cli_runtime_overrides, build_i18n_command, close_thinking_chunk,
        format_thinking_chunk, format_verbose_tool_chunk, headless_completion_exit_code,
        headless_completion_notify_reason, headless_denial_exit_code,
        headless_missing_provider_message, interactive_provider_bootstrap,
        is_completion_invocation, merge_startup_notices, print_shell_completion,
        resolve_in_catalog, resolve_wiki_llm_selection, resolve_working_dir, resume_hint_line,
        runtime_config_from, rustcode_log_path, should_fork_busy_continue, truncate_log_line,
        wiki_removed_lines, wiki_run_is_up_to_date, Cli, Commands, HeadlessOutputFormat, WikiArgs,
        DEFAULT_LOG_DIRECTIVES,
    };
    use clap::Parser;
    use clap_complete::Shell;
    use std::path::PathBuf;

    fn catalog_entry(
        id: &str,
        name: &str,
        updated_at_ms: i64,
    ) -> rustcode_capabilities::session::CatalogEntry {
        rustcode_capabilities::session::CatalogEntry {
            id: id.to_string(),
            name: name.to_string(),
            fork_root_id: None,
            project_bucket: "0123456789abcdef".to_string(),
            working_dir: PathBuf::from("/w"),
            created_at_ms: 0,
            updated_at_ms,
            message_count: 1,
            turn_count: 1,
            presence: rustcode_capabilities::session::CatalogPresence::NativeOnly,
        }
    }

    #[test]
    fn resolve_in_catalog_prefers_exact_id_then_most_recent_name() {
        let catalog = vec![
            catalog_entry("aaa", "review", 100),
            catalog_entry("bbb", "review", 300), // newer duplicate name
            catalog_entry("ccc", "deploy", 200),
        ];
        // Exact id wins even when a name also matches something.
        assert_eq!(resolve_in_catalog(&catalog, "ccc").as_deref(), Some("ccc"));
        // Ambiguous name resolves to the most-recently-updated session.
        assert_eq!(
            resolve_in_catalog(&catalog, "review").as_deref(),
            Some("bbb")
        );
        // Unique name.
        assert_eq!(
            resolve_in_catalog(&catalog, "deploy").as_deref(),
            Some("ccc")
        );
        // No match.
        assert_eq!(resolve_in_catalog(&catalog, "nope"), None);
        assert_eq!(resolve_in_catalog(&[], "review"), None);
    }

    #[test]
    fn resume_hint_line_forms_match_headless_vs_tui_and_language() {
        // Default locale is ZhCn; pin explicitly so the English assertions hold.
        let _g = rustcode_config::i18n::test_lock();
        rustcode_config::i18n::set_locale(rustcode_config::i18n::Locale::En);
        // Headless continues a pipe run with `-p ... --resume <id>`.
        assert_eq!(
            resume_hint_line("abc", true),
            "To resume this session, run: rustcode -p \"...\" --resume abc"
        );
        // TUI shows the `resume <id>` subcommand (codex parity).
        assert_eq!(
            resume_hint_line("abc", false),
            "To resume this session, run: rustcode resume abc"
        );
        // Chinese locale.
        rustcode_config::i18n::set_locale(rustcode_config::i18n::Locale::ZhCn);
        assert_eq!(
            resume_hint_line("abc", false),
            "继续此会话，运行：rustcode resume abc"
        );
    }

    #[test]
    fn resume_flag_and_subcommand_parse() {
        // `-p "..." --resume <id>` -> headless resume.
        let c = Cli::try_parse_from(["rustcode", "-p", "hi", "--resume", "sid"]).unwrap();
        assert_eq!(c.resume.as_deref(), Some("sid"));
        // `resume <id>` subcommand.
        let c = Cli::try_parse_from(["rustcode", "resume", "sid"]).unwrap();
        assert!(matches!(
            c.command,
            Some(Commands::Resume { session: Some(s) }) if s == "sid"
        ));
        // bare `resume` -> most recent.
        let c = Cli::try_parse_from(["rustcode", "resume"]).unwrap();
        assert!(matches!(
            c.command,
            Some(Commands::Resume { session: None })
        ));
        // `--resume` conflicts with `--continue`.
        assert!(Cli::try_parse_from(["rustcode", "-p", "hi", "-c", "--resume", "x"]).is_err());
    }

    #[test]
    fn completion_subcommand_defaults_to_bash_and_accepts_all_supported_shells() {
        let default = Cli::try_parse_from(["rustcode", "completion"]).unwrap();
        assert!(matches!(
            default.command,
            Some(Commands::Completion(command)) if command.shell == Shell::Bash
        ));

        for shell in ["bash", "zsh", "fish", "powershell", "elvish"] {
            let parsed = Cli::try_parse_from(["rustcode", "completion", shell]).unwrap();
            assert!(matches!(parsed.command, Some(Commands::Completion(_))));
        }
    }

    #[test]
    fn headless_eval_flags_accept_inline_or_file_prompts() {
        let inline =
            Cli::try_parse_from(["rustcode", "-p", "answer", "--ephemeral", "--no-tools"]).unwrap();
        assert!(inline.ephemeral);
        assert!(inline.no_tools);

        let jsonl =
            Cli::try_parse_from(["rustcode", "-p", "answer", "--output-format", "jsonl"]).unwrap();
        assert_eq!(jsonl.output_format, HeadlessOutputFormat::Jsonl);

        let file =
            Cli::try_parse_from(["rustcode", "--prompt-file", "prompt.txt", "--no-tools"]).unwrap();
        assert!(file.no_tools);
    }

    #[test]
    fn headless_eval_flags_reject_interactive_and_resume_combinations() {
        assert!(Cli::try_parse_from(["rustcode", "--ephemeral"]).is_err());
        assert!(Cli::try_parse_from(["rustcode", "--no-tools"]).is_err());
        assert!(Cli::try_parse_from(["rustcode", "--output-format", "jsonl"]).is_err());
        assert!(Cli::try_parse_from(["rustcode", "-p", "answer", "-c", "--ephemeral"]).is_err());
    }

    #[test]
    fn denied_headless_turn_reports_the_process_exit_code_in_its_terminal() {
        assert_eq!(headless_denial_exit_code(0, true), 2);
        assert_eq!(headless_denial_exit_code(1, true), 1);
        assert_eq!(headless_denial_exit_code(0, false), 0);
    }

    #[test]
    fn completion_scripts_cover_all_supported_shells() {
        for shell in [
            Shell::Bash,
            Shell::Zsh,
            Shell::Fish,
            Shell::PowerShell,
            Shell::Elvish,
        ] {
            let mut output = Vec::new();
            print_shell_completion(shell, &mut output);
            let script = String::from_utf8(output).unwrap();
            assert!(!script.is_empty(), "{shell:?} script should not be empty");
            assert!(
                script.contains("rustcode"),
                "{shell:?} script should target rustcode"
            );
            assert!(
                script.contains("completion"),
                "{shell:?} script should include the completion command"
            );
            assert!(
                !script.contains("__askpass"),
                "{shell:?} script should not expose the internal askpass helper"
            );
            assert!(
                !script.contains("codingplan"),
                "{shell:?} script should not expose deprecated hidden aliases"
            );
            // There is no managed `login`/`logout` in this build; the only
            // auth commands that may appear are the unrelated `mcp login`/
            // `mcp logout` OAuth pair. A flat substring
            // check cannot separate the two, so track the enclosing context
            // (bash function headers, zsh curcontext, elvish/powershell map
            // paths) and require every mention to sit in an mcp context.
            let mut context_marker = String::new();
            for line in script.lines() {
                let is_context_marker = line.contains("curcontext=")
                    || line.contains("()") && line.contains('{')
                    || line.contains("&'rustcode;")
                    || line.contains("'rustcode;") && line.contains('{');
                if is_context_marker {
                    context_marker = line.to_string();
                }
                if line.contains("login") || line.contains("logout") {
                    let mcp_context = line.to_lowercase().contains("mcp")
                        || line.contains("github-oauth")
                        || context_marker.to_lowercase().contains("mcp");
                    assert!(
                        mcp_context,
                        "{shell:?} advertises managed login/logout outside `mcp`\n\
                         line: {line}\ncontext: {context_marker}"
                    );
                }
            }
            assert!(
                !script.contains("acp"),
                "{shell:?} script should not expose internal protocol commands"
            );
        }
    }

    #[test]
    fn neutral_build_has_no_managed_login_subcommands() {
        // There is no managed sign-in service, so `rustcode login` / `rustcode
        // logout` must not exist at all -- neither in `--help` nor in shell
        // completion. A hidden-but-runnable variant would be a dead end that
        // only pretends to sign the user in or out.
        let cmd = build_i18n_command();
        assert!(
            cmd.find_subcommand("login").is_none(),
            "`rustcode login` must not exist in a neutral build"
        );
        assert!(
            cmd.find_subcommand("logout").is_none(),
            "`rustcode logout` must not exist in a neutral build"
        );
        // `status` stays visible -- in a neutral build it prints the BYO hint.
        assert!(!cmd
            .find_subcommand("status")
            .expect("status subcommand")
            .is_hide_set());
    }

    #[test]
    fn completion_early_exit_only_matches_the_root_subcommand() {
        let invocation =
            |args: &[&str]| is_completion_invocation(args.iter().map(std::ffi::OsString::from));

        assert!(invocation(&["completion", "zsh"]));
        assert!(invocation(&[
            "--config",
            "/tmp/config.toml",
            "completion",
            "fish"
        ]));
        assert!(invocation(&[
            "--config",
            "/tmp/config.toml",
            "completion",
            "bash"
        ]));
        assert!(!invocation(&["--provider", "completion"]));
        assert!(!invocation(&["-p", "completion"]));
        assert!(!invocation(&["mcp", "add", "completion", "server"]));
        assert!(!invocation(&["--", "completion"]));
    }

    #[test]
    fn log_path_is_logs_subdir_of_config_dir() {
        // The log lives at `<config_dir>/logs/rustcode.log`; RUSTCODE_HOME/sudo
        // resolution is `Config::config_dir()`'s job (covered by its own tests), so
        // this pins only the join rule.
        assert_eq!(
            rustcode_log_path(PathBuf::from("/Users/x/.rustcode")),
            PathBuf::from("/Users/x/.rustcode/logs/rustcode.log")
        );
    }

    #[test]
    fn default_log_directives_parse() {
        // A malformed default would silently disable ALL logging via the fallback
        // path in `init_file_logging`; pin that it is a valid EnvFilter directive.
        assert!(tracing_subscriber::EnvFilter::try_new(DEFAULT_LOG_DIRECTIVES).is_ok());
    }

    #[test]
    fn config_and_session_startup_notices_are_both_preserved() {
        assert_eq!(
            merge_startup_notices(
                Some("bad provider ignored".into()),
                Some("busy session forked".into())
            )
            .as_deref(),
            Some("bad provider ignored\nbusy session forked")
        );
    }

    #[test]
    fn busy_continue_forks_only_for_interactive_session_contention() {
        let busy = rustcode_capabilities::session::SessionStoreError::SessionInUse {
            id: "source".into(),
            path: PathBuf::from("/sessions/source.lease"),
        };
        let missing = rustcode_capabilities::session::SessionStoreError::NotFound {
            path: PathBuf::from("/sessions/source.meta"),
        };

        assert!(should_fork_busy_continue(true, &busy));
        assert!(!should_fork_busy_continue(false, &busy));
        assert!(!should_fork_busy_continue(true, &missing));
    }

    #[test]
    fn verbose_tool_chunk_strips_ephemeral_activity_marker() {
        assert_eq!(
            format_verbose_tool_chunk("\u{1e}review . 2 findings . read_file"),
            "[progress] review . 2 findings . read_file\n"
        );
    }

    #[test]
    fn snapshot_unavailable_is_headless_failure_even_when_reason_is_stopped() {
        let completion = rustcode_coding::TurnCompletion::SnapshotUnavailable {
            turn_id: 1,
            reason: rustcode_kernel::event::StopReason::Stopped,
            error: rustcode_coding::RuntimeSnapshotError {
                message: "snapshot failed".into(),
            },
            stats: Default::default(),
        };

        assert_eq!(headless_completion_exit_code(&completion, 0), 1);
        assert_eq!(
            headless_completion_notify_reason(&completion),
            rustcode_capabilities::notify::NotifyStopReason::Error
        );
    }

    #[test]
    fn headless_missing_provider_message_fires_only_without_resolved_model() {
        // Default locale is ZhCn; the assertions below match English copy.
        let _g = rustcode_config::i18n::test_lock();
        rustcode_config::i18n::set_locale(rustcode_config::i18n::Locale::En);
        let wd = PathBuf::from("/tmp/x");

        // Empty config + a bogus `--provider` name: nothing resolves, so headless
        // must get an actionable "Provider not found" error (T5b) rather than a
        // cryptic reqwest "relative URL without a base".
        let empty: rustcode_config::config::Config = toml::from_str("").unwrap();
        let empty_cfg = runtime_config_from(&empty, &wd, None, false, false);
        let named = headless_missing_provider_message(&empty_cfg, Some("__nonexistent_qa_probe__"))
            .expect("empty config with a named --provider yields an error");
        assert!(named.contains("Provider not found"), "got: {named}");
        assert!(
            named.contains("__nonexistent_qa_probe__"),
            "message should name the requested provider: {named}"
        );

        // Empty config, no `--provider`: the no-provider branch still flags it.
        let unnamed = headless_missing_provider_message(&empty_cfg, None)
            .expect("empty config without a provider yields an error");
        assert!(
            unnamed.to_ascii_lowercase().contains("provider"),
            "got: {unnamed}"
        );

        // A configured third-party provider resolves a model -> run proceeds.
        let toml_str = r#"
            default_provider = "p"
            [providers.p]
            type = "openai"
            api_key = "sk-x"
            model = "m"
            base_url = "https://api.example.com/v1"
        "#;
        let cfg: rustcode_config::config::Config = toml::from_str(toml_str).unwrap();
        let resolved = runtime_config_from(&cfg, &wd, None, false, false);
        assert!(headless_missing_provider_message(&resolved, None).is_none());
        assert!(headless_missing_provider_message(&resolved, Some("p")).is_none());
    }

    #[test]
    fn runtime_config_honors_provider_override() {
        // Regression: engine-v2 headless `--provider X` was silently ignored --
        // runtime_config_from read `default_provider` directly instead of routing
        // through `active_provider`, so the runtime picked the config default
        // (e.g. a gateway needing a signer this build lacks) and a
        // `--provider direct` run hit the wrong endpoint and failed.
        let toml_str = r#"
            default_provider = "gateway"

            [providers.gateway]
            type = "openai"
            model = "gw-model"
            base_url = "https://gateway.test.example/v1"

            [providers.direct]
            type = "openai"
            api_key = "sk-direct"
            model = "direct-model"
            base_url = "https://direct.example.com/v1"
            reasoning_history = "exclude"
        "#;
        let config: rustcode_config::config::Config = toml::from_str(toml_str).unwrap();
        let wd = PathBuf::from("/tmp/x");

        // No override -> the config default (gateway), no reasoning_history set.
        let def = runtime_config_from(&config, &wd, None, false, false);
        assert_eq!(def.base_url, "https://gateway.test.example/v1");
        assert_eq!(def.model, "gw-model");
        assert_eq!(def.provider_name, "gateway");
        assert_eq!(def.reasoning_history, None);

        // `--provider direct` -> that provider's endpoint/model/key + its per-provider
        // reasoning_history override, NOT the default.
        let ov = runtime_config_from(&config, &wd, Some("direct"), false, false);
        assert_eq!(ov.base_url, "https://direct.example.com/v1");
        assert_eq!(ov.model, "direct-model");
        assert_eq!(ov.provider_name, "direct");
        assert_eq!(ov.api_key, "sk-direct");
        assert_eq!(ov.reasoning_history.as_deref(), Some("exclude"));
    }

    #[test]
    fn interactive_restart_recovers_new_schema_provider() {
        let config: rustcode_config::config::Config = toml::from_str(
            r#"
                default_model = "taotoken/glm_for_coding"

                [provider_accounts.taotoken]
                provider = "openai"
                base_url = "https://example.test/v1"
                api_key = "test"

                [models."taotoken/glm_for_coding"]
                account = "taotoken"
                model = "glm_for_coding"
                context_window = 131072
            "#,
        )
        .unwrap();
        let runtime_cfg =
            runtime_config_from(&config, std::path::Path::new("/tmp"), None, false, true);

        assert!(config.providers.is_empty(), "legacy table stays empty");
        assert_eq!(
            interactive_provider_bootstrap(&runtime_cfg),
            rustcode_coding::ProviderBootstrap::RecoverAuthentication
        );
        let empty_runtime_cfg = runtime_config_from(
            &rustcode_config::config::Config::default(),
            std::path::Path::new("/tmp"),
            None,
            false,
            true,
        );
        assert_eq!(
            interactive_provider_bootstrap(&empty_runtime_cfg),
            rustcode_coding::ProviderBootstrap::Unavailable(
                rustcode_coding::ProviderUnavailableReason::NotConfigured
            )
        );
    }

    #[test]
    fn interactive_bootstrap_uses_runtime_fallback_resolution() {
        let config: rustcode_config::config::Config = toml::from_str(
            r#"
                default_model = "broken"

                [provider_accounts.missing]
                provider = "openai"

                [models.broken]
                account = "unknown"
                model = "broken-model"
                context_window = 131072

                [provider_accounts.usable]
                provider = "openai"
                base_url = "https://example.test/v1"
                api_key = "test"

                [models.usable]
                account = "usable"
                model = "fallback-model"
                context_window = 131072
            "#,
        )
        .unwrap();
        let runtime_cfg =
            runtime_config_from(&config, std::path::Path::new("/tmp"), None, false, true);

        assert_eq!(runtime_cfg.model, "fallback-model");
        assert_eq!(
            interactive_provider_bootstrap(&runtime_cfg),
            rustcode_coding::ProviderBootstrap::RecoverAuthentication
        );
    }

    #[test]
    fn round_cap_checkpoint_default_off_and_propagates_to_agent_config() {
        // Guard against C1 regression: the interactive checkpoint is TUI-only,
        // opted in at the TUI spawn sites by flipping `round_cap_checkpoint` on
        // the runtime config. A direct test of the async CLI factory
        // (`spawn_native_cli_runtime`/`spawn_deferred_tui_runtime`) isn't
        // feasible here -- both need a live runtime + provider bootstrap -- so we
        // pin the propagation seam those sites rely on: the field defaults off
        // and `agent_config()` copies it through to the kernel-facing config.
        let toml_str = r#"
            default_provider = "p"
            [providers.p]
            type = "openai"
            model = "m"
            api_key = "k"
            base_url = "https://example.test/v1"
        "#;
        let config: rustcode_config::config::Config = toml::from_str(toml_str).unwrap();
        let wd = PathBuf::from("/tmp/x");

        // Default (headless / ACP / daemon behavior): checkpoint stays off.
        let mut cfg = runtime_config_from(&config, &wd, None, false, true);
        assert!(!cfg.round_cap_checkpoint, "defaults off");
        assert!(
            !cfg.agent_config().round_cap_checkpoint,
            "off config yields off agent"
        );

        // TUI opt-in: the flag flows through agent_config() to the kernel.
        cfg.round_cap_checkpoint = true;
        assert!(
            cfg.agent_config().round_cap_checkpoint,
            "TUI opt-in reaches the agent config"
        );
    }

    #[test]
    fn interactive_provider_override_becomes_the_in_process_default() {
        let mut config: rustcode_config::config::Config = toml::from_str(
            r#"
                default_provider = "gateway"

                [providers.gateway]
                type = "openai"
                model = "gw-model"

                [providers.direct]
                type = "openai"
                model = "direct-model"
            "#,
        )
        .unwrap();

        apply_cli_runtime_overrides(&mut config, Some("direct"), None);

        assert_eq!(config.default_provider, "direct");
    }

    #[test]
    fn ascii_short_unchanged() {
        assert_eq!(truncate_log_line("hello", 10), "hello");
    }

    #[test]
    fn ascii_long_truncated_with_ellipsis() {
        assert_eq!(truncate_log_line("0123456789abcdef", 10), "0123456789...");
    }

    #[test]
    fn newlines_become_spaces() {
        assert_eq!(truncate_log_line("a\nb\nc", 10), "a b c");
    }

    #[test]
    fn mixed_ascii_cjk_truncates_at_char_boundary() {
        // 8 chars: ['a','b','c','计','算','d','e','f']; max 5 -> "abc计算..."
        assert_eq!(truncate_log_line("abc计算def", 5), "abc计算...");
    }

    /// Regression test for panic at `crates/rustcode-cli/src/main.rs:272:42`:
    /// "byte index 500 is not a char boundary; it is inside '计' (bytes 498..501)".
    /// Triggered when ToolCallResult output was a CJK-heavy string > 500 bytes
    /// and the old code did `trimmed[..500]` (byte slice). Pure CJK at 3 bytes
    /// per char means almost any 500-byte cut lands inside a multi-byte char.
    #[test]
    fn cjk_truncation_does_not_panic() {
        let s: String = "计算".repeat(500); // 1000 chars, 3000 bytes
        let result = truncate_log_line(&s, 500);
        assert_eq!(result.chars().count(), 503); // 500 + "..."
        assert!(result.ends_with("..."));
    }

    /// Regression test for cwd-override bug: when no `-C` is given, working dir
    /// must equal `std::env::current_dir()`. Old code silently substituted the
    /// first line of `~/.rustcode/recent_dirs.txt`, breaking `rustcode -p` from
    /// any directory that wasn't the TUI's last-visited project.
    #[test]
    fn resolve_working_dir_uses_cwd_when_no_cli_dir() {
        let expected = std::env::current_dir().unwrap();
        assert_eq!(resolve_working_dir(None), expected);
    }

    #[test]
    fn resolve_working_dir_honors_cli_dir() {
        let temp = std::env::temp_dir();
        let canon = std::fs::canonicalize(&temp).unwrap_or(temp.clone());
        assert_eq!(resolve_working_dir(Some(temp)), canon);
    }

    #[test]
    fn resolve_working_dir_falls_back_to_input_when_canonicalize_fails() {
        // Use a non-existent path so canonicalize() returns Err and the
        // function falls back to the raw input rather than panicking.
        let bogus = PathBuf::from("/nonexistent/rustcode-test-path-xyzzy");
        assert_eq!(resolve_working_dir(Some(bogus.clone())), bogus);
    }

    /// Verify that std::fs::read_to_string reads a temp file correctly,
    /// which is the core of --prompt-file. This is a unit-level stand-in for
    /// the integration test (full CLI parse requires a running provider).
    #[test]
    fn prompt_file_read_preserves_trailing_newline() {
        use std::io::Write as _;
        let path = std::env::temp_dir().join("rustcode_test_prompt_file.txt");
        let content = "fix the bug\n";
        {
            let mut f = std::fs::File::create(&path).unwrap();
            f.write_all(content.as_bytes()).unwrap();
        }
        let read_back = std::fs::read_to_string(&path).unwrap();
        std::fs::remove_file(&path).ok();
        assert_eq!(
            read_back, content,
            "--prompt-file must preserve trailing newline (unlike bash $(...))"
        );
    }

    /// End-to-end enrichment with a *mock* `LlmProvider` (no network): confirms
    /// `enrich_wiki` calls the provider, persists the summary sidecar, and rewrites
    /// the module page to embed it. Closes the R2 gap (LLM path was previously
    /// only best-effort and untested).
    #[tokio::test]
    async fn wiki_enrich_writes_summary_via_mock_provider() {
        use async_trait::async_trait;
        use futures::stream::{self, BoxStream};
        use rustcode_kernel::message::Message;
        use rustcode_kernel::provider::ChatOptions;
        use rustcode_kernel::stream::{ProviderError, StreamEvent};
        use rustcode_kernel::tool::ToolDef;

        struct StaticProvider(&'static str);
        #[async_trait]
        impl rustcode_kernel::provider::LlmProvider for StaticProvider {
            fn model_name(&self) -> &str {
                "static"
            }
            async fn chat_stream(
                &self,
                _messages: &[Message],
                _tools: &[ToolDef],
                _options: &ChatOptions,
            ) -> Result<BoxStream<'static, StreamEvent>, ProviderError> {
                let text = self.0;
                Ok(Box::pin(stream::iter(vec![
                    StreamEvent::TextDelta(text.to_string()),
                    StreamEvent::Done { truncated: false },
                ])))
            }
        }

        // A minimal crate under a unique temp root.
        let root =
            std::env::temp_dir().join(format!("rustcode_wiki_enrich_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        std::fs::write(root.join("src/lib.rs"), "pub fn hi() {}\n").unwrap();

        let opts = rustcode_wiki::WikiOptions {
            root: root.clone(),
            out_dir: None,
            title: None,
            force: false,
            max_files: 10_000,
            exclude_dirs: vec![],
            langs: vec![],
            assume_yes: true,
        };
        rustcode_wiki::WikiEngine::generate(&opts).expect("generate");

        let count = super::enrich_wiki(&opts, &StaticProvider("MOCK_SUMMARY_XYZ"))
            .await
            .expect("enrich");
        assert!(count >= 1, "expected at least one module enriched");

        let page = std::fs::read_to_string(root.join(".rustcode/wiki/zh/Modules/demo.md"))
            .expect("read page");
        assert!(
            page.contains("MOCK_SUMMARY_XYZ"),
            "module page should embed the mock summary, got: {page}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    // ---- format_thinking_chunk / close_thinking_chunk ----
    //
    // Regression suite for the "one word per line" bug in headless verbose
    // output. The old `eprintln!("[thinking] {}", text)` printed a fresh
    // prefix + trailing newline for every streaming chunk, so a streaming
    // reasoning model produced output like:
    //
    //   [thinking] are
    //   [thinking] already
    //   [thinking] configured
    //
    // The new formatter must keep a single line open across many tiny
    // chunks until something else (text, tool call, turn complete, etc.)
    // explicitly closes it.

    /// Streaming many single-token chunks must produce ONE line, not N.
    #[test]
    fn thinking_chunks_stream_onto_single_line() {
        let mut buf = String::new();
        let mut open = false;
        for tok in ["are", " already", " configured", " and", " what"] {
            format_thinking_chunk(&mut buf, &mut open, tok);
        }
        assert_eq!(buf, "[thinking] are already configured and what");
        assert!(open, "line should remain open until something closes it");
    }

    /// A non-reasoning event must close the line with a single newline.
    #[test]
    fn close_appends_newline_and_clears_open() {
        let mut buf = String::new();
        let mut open = false;
        format_thinking_chunk(&mut buf, &mut open, "hello");
        close_thinking_chunk(&mut buf, &mut open);
        assert_eq!(buf, "[thinking] hello\n");
        assert!(!open);
        // Closing again is a no-op (idempotent).
        close_thinking_chunk(&mut buf, &mut open);
        assert_eq!(buf, "[thinking] hello\n");
    }

    /// Embedded newlines inside a chunk must produce a re-prefixed next line.
    #[test]
    fn embedded_newline_reprefixes_next_line() {
        let mut buf = String::new();
        let mut open = false;
        format_thinking_chunk(&mut buf, &mut open, "first line\nsecond line");
        assert_eq!(buf, "[thinking] first line\n[thinking] second line");
        assert!(open);
    }

    /// A chunk ending with `\n` closes the line; the next chunk must
    /// re-introduce the `[thinking] ` prefix.
    #[test]
    fn trailing_newline_closes_and_next_chunk_reprefixes() {
        let mut buf = String::new();
        let mut open = false;
        format_thinking_chunk(&mut buf, &mut open, "para1\n");
        assert!(!open, "trailing newline should close the line");
        format_thinking_chunk(&mut buf, &mut open, "para2");
        assert_eq!(buf, "[thinking] para1\n[thinking] para2");
        assert!(open);
    }

    /// Empty chunks must be skipped without emitting a stray prefix.
    #[test]
    fn empty_chunk_is_noop() {
        let mut buf = String::new();
        let mut open = false;
        format_thinking_chunk(&mut buf, &mut open, "");
        assert_eq!(buf, "");
        assert!(!open);
        // Still no prefix after empty input.
        format_thinking_chunk(&mut buf, &mut open, "x");
        assert_eq!(buf, "[thinking] x");
    }

    /// CJK content (common in Chinese reasoning models) must not break the
    /// single-line invariant -- every char-level chunk just appends.
    #[test]
    fn cjk_chunks_stream_correctly() {
        let mut buf = String::new();
        let mut open = false;
        for tok in ["先", "看", "看", "你", "当前的", "环境"] {
            format_thinking_chunk(&mut buf, &mut open, tok);
        }
        assert_eq!(buf, "[thinking] 先看看你当前的环境");
    }

    /// Simulated end-to-end event sequence: thinking deltas, then a tool
    /// call. The tool call must appear on its OWN line, not mashed onto
    /// the tail of the thinking text.
    #[test]
    fn thinking_followed_by_tool_call_is_separated() {
        let mut buf = String::new();
        let mut open = false;
        format_thinking_chunk(&mut buf, &mut open, "I should");
        format_thinking_chunk(&mut buf, &mut open, " check");
        format_thinking_chunk(&mut buf, &mut open, " the file");
        // Now a non-reasoning event arrives -> close, then emit it.
        close_thinking_chunk(&mut buf, &mut open);
        buf.push_str("[tool-> read_file]\n");
        assert_eq!(
            buf,
            "[thinking] I should check the file\n[tool-> read_file]\n"
        );
    }

    // --- wiki LLM selection precedence -----------------------------------
    //
    // `resolve_wiki_llm_selection` is the pure core of `run_wiki_command`'s
    // "flag > `[wiki]` config > globally active provider" contract. The
    // provider itself (`build_wiki_provider`) needs a real base URL / API key
    // to construct, so the precedence is asserted here instead.

    /// Minimal `WikiArgs` with everything off/empty.
    fn wiki_args() -> WikiArgs {
        WikiArgs {
            path: None,
            sync: false,
            watch: false,
            llm: false,
            force: false,
            provider: None,
            model: None,
            out_dir: None,
            title: None,
            exclude: Vec::new(),
            interval: 30,
            lang: Vec::new(),
            yes: false,
        }
    }

    /// `Config::default()` with the `[wiki]` LLM knobs set as given.
    fn config_with_wiki_llm(
        use_llm: bool,
        provider: Option<&str>,
        model: Option<&str>,
    ) -> rustcode_config::Config {
        let mut config = rustcode_config::Config::default();
        config.wiki.use_llm = use_llm;
        config.wiki.provider = provider.map(str::to_string);
        config.wiki.model = model.map(str::to_string);
        config
    }

    /// CLI `--provider` / `--model` win over `[wiki] provider` / `[wiki] model`.
    #[test]
    fn wiki_llm_selection_flags_override_wiki_config() {
        let mut args = wiki_args();
        args.provider = Some("flag-provider".to_string());
        args.model = Some("flag-model".to_string());
        let config = config_with_wiki_llm(false, Some("cfg-provider"), Some("cfg-model"));

        let (use_llm, provider, model) = resolve_wiki_llm_selection(&args, &config);
        assert!(
            !use_llm,
            "no --llm and [wiki].use_llm=false -> enrichment stays off"
        );
        assert_eq!(provider.as_deref(), Some("flag-provider"));
        assert_eq!(model.as_deref(), Some("flag-model"));
    }

    /// With no flags, the `[wiki]` section supplies both provider and model
    /// (and turns enrichment on).
    #[test]
    fn wiki_llm_selection_uses_wiki_config_without_flags() {
        let args = wiki_args();
        let config = config_with_wiki_llm(true, Some("cfg-provider"), Some("cfg-model"));

        let (use_llm, provider, model) = resolve_wiki_llm_selection(&args, &config);
        assert!(use_llm);
        assert_eq!(provider.as_deref(), Some("cfg-provider"));
        assert_eq!(model.as_deref(), Some("cfg-model"));
    }

    /// Nothing configured anywhere -> enrichment off and `None` provider /
    /// model, so `build_wiki_provider` falls back to the globally active
    /// provider instead of pinning an override.
    #[test]
    fn wiki_llm_selection_defaults_to_off_and_no_override() {
        let args = wiki_args();
        let config = rustcode_config::Config::default();

        let (use_llm, provider, model) = resolve_wiki_llm_selection(&args, &config);
        assert!(!use_llm);
        assert_eq!(provider, None);
        assert_eq!(model, None);
    }

    /// `[wiki] use_llm = true` alone enables enrichment, with no provider or
    /// model override layered on top.
    #[test]
    fn wiki_llm_selection_use_llm_comes_from_config() {
        let args = wiki_args();
        let config = config_with_wiki_llm(true, None, None);

        let (use_llm, provider, model) = resolve_wiki_llm_selection(&args, &config);
        assert!(use_llm);
        assert_eq!(provider, None);
        assert_eq!(model, None);
    }

    /// Flags and config mix field-by-field: a flag on one field must not
    /// discard the configured value of the other.
    #[test]
    fn wiki_llm_selection_mixes_flag_provider_with_config_model() {
        let mut args = wiki_args();
        args.llm = true;
        args.provider = Some("flag-provider".to_string());
        let config = config_with_wiki_llm(false, Some("cfg-provider"), Some("cfg-model"));

        let (use_llm, provider, model) = resolve_wiki_llm_selection(&args, &config);
        assert!(use_llm, "--llm alone is enough to enable enrichment");
        assert_eq!(provider.as_deref(), Some("flag-provider"));
        assert_eq!(model.as_deref(), Some("cfg-model"));
    }

    /// GAP ① regression: a run whose only effect was removing stale files must
    /// NOT be reported as up to date, and each removed file must surface on its
    /// own line (naming the file) so module deletions are visible on the CLI.
    #[test]
    fn wiki_removed_files_are_surfaced_and_not_up_to_date() {
        let gone = PathBuf::from("/w/.rustcode/wiki/zh/Modules/gone.md");
        let run = rustcode_wiki::WikiResult {
            out_dir: PathBuf::from("/w/.rustcode/wiki"),
            modules: 1,
            created: Vec::new(),
            updated: Vec::new(),
            unchanged: 0,
            removed: vec![gone.clone()],
            preserved: Vec::new(),
            diagram: String::new(),
        };

        assert!(
            !wiki_run_is_up_to_date(&run),
            "a run with removals must not be reported as up to date"
        );
        let lines = wiki_removed_lines(&run);
        assert_eq!(lines.len(), 1, "one line per removed file, got {lines:?}");
        assert!(
            lines[0].contains("gone.md"),
            "the removed line must name the file, got {:?}",
            lines[0]
        );

        // An entirely empty run stays on the up-to-date path.
        let clean = rustcode_wiki::WikiResult {
            removed: Vec::new(),
            ..run
        };
        assert!(
            wiki_run_is_up_to_date(&clean),
            "a run with no created/updated/preserved/removed work is up to date"
        );
        assert!(
            wiki_removed_lines(&clean).is_empty(),
            "no removals => no removal lines"
        );
    }
}

/// T-03 锁定测试：`rustcode webui` 的绑定地址默认值，以及显式 `--host` 对默认值的覆盖。
///
/// 默认值从 `127.0.0.1` 改成 `0.0.0.0` 是**对外暴露面**的变更：默认值的含义变了，
/// 而"显式指定必须赢过默认值"这条不变式一旦破掉，`--host 127.0.0.1` 就会静默失效。
/// 这里在参数解析层把两件事钉死，作为 AC-19 / AC-20 的回归哨兵。
#[cfg(test)]
mod default_host_tests {
    use super::{Cli, Commands};
    use clap::Parser;

    /// 解析 `rustcode webui <args...>`，返回最终生效的绑定地址。
    fn webui_host(args: &[&str]) -> String {
        let argv: Vec<&str> = ["rustcode", "webui"]
            .into_iter()
            .chain(args.iter().copied())
            .collect();
        let cli = Cli::try_parse_from(argv).expect("`rustcode webui` 参数解析失败");
        match cli.command {
            Some(Commands::Webui { host, .. }) => host,
            Some(_) => panic!("解析出了非 webui 子命令"),
            None => panic!("未解析出任何子命令"),
        }
    }

    /// 不带 `--host`：默认绑定 `0.0.0.0`（T-03 K1，AC-19 的正向断言）。
    #[test]
    fn webui_defaults_to_all_interfaces() {
        assert_eq!(
            webui_host(&[]),
            "0.0.0.0",
            "`rustcode webui` 不带 --host 时必须默认绑定 0.0.0.0"
        );
    }

    /// 显式 `--host` 必须覆盖默认值（AC-20 的参数解析侧）。
    #[test]
    fn webui_explicit_host_overrides_default() {
        assert_eq!(
            webui_host(&["--host", "127.0.0.1"]),
            "127.0.0.1",
            "显式 --host 127.0.0.1 必须覆盖默认值，把服务收回本机"
        );
        assert_eq!(
            webui_host(&["--host", "0.0.0.0"]),
            "0.0.0.0",
            "显式给出与默认值相同的值时行为不变"
        );
        assert_eq!(webui_host(&["--host", "192.168.1.7"]), "192.168.1.7");
    }

    /// 解析 `rustcode daemon <args...>`，返回最终生效的绑定地址。
    ///
    /// 与 `webui_host()` 对称：T-15 之前 `daemon` 没有 `--host`，绑定地址是
    /// `ServerOpts { host: "0.0.0.0" }` 的硬编码字面量，改坏它没有任何解析层
    /// 哨兵会拦。T-15 把它提升为带默认值的参数后，默认值就是唯一剩下的
    /// 「不传参也生效」的暴露面，必须在解析层钉死。
    fn daemon_host(args: &[&str]) -> String {
        let argv: Vec<&str> = ["rustcode", "daemon"]
            .into_iter()
            .chain(args.iter().copied())
            .collect();
        let cli = Cli::try_parse_from(argv).expect("`rustcode daemon` 参数解析失败");
        match cli.command {
            Some(Commands::Daemon { host, .. }) => host,
            Some(_) => panic!("解析出了非 daemon 子命令"),
            None => panic!("未解析出任何子命令"),
        }
    }

    /// 不带 `--host`：默认绑定 `0.0.0.0`（T-15 的 Q2-A 冻结默认值，与 `webui` 同源）。
    ///
    /// 只断言默认值：显式 `--host <ip>` 的取值由调用方自行负责，且 Q2-A 冻结的是
    /// 「默认暴露面」，故这里不锁定任何非默认值。
    #[test]
    fn daemon_defaults_to_all_interfaces() {
        assert_eq!(
            daemon_host(&[]),
            "0.0.0.0",
            "`rustcode daemon` 不带 --host 时必须默认绑定 0.0.0.0"
        );
    }
}
