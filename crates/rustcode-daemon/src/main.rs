//! RustCode API Service -- standalone binary entrypoint.
//!
//! This is a thin shell around [`rustcode_daemon::run_server`]: it parses CLI
//! arguments, performs process-global bootstrap (Windows console attach, legacy
//! session migration) and then delegates to the shared server logic in the
//! `rustcode_daemon` library crate.

// On Windows, mark this binary as a GUI-subsystem application so that
// launching it from a GUI parent (e.g. VSCode extension host) does NOT
// allocate a visible console window. When launched from a terminal the
// daemon will attempt to re-attach to the parent console for stderr output.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use rustcode_daemon::{client_mode::ClientMode, run_server, ServerOpts};

/// Default idle timeout in seconds (30 minutes) before the daemon self-shuts
/// down when no client activity is observed.
const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 30 * 60;

fn parse_daemon_args() -> (String, u16, u64, ClientMode) {
    const DEFAULT_HOST: &str = "127.0.0.1";
    // Shared with `rustcode daemon --port`'s clap default, which used to carry
    // its own `13456` literal.
    const DEFAULT_PORT: u16 = rustcode_config::distribution::DAEMON_PORT;

    let mut host: Option<String> = None;
    let mut port: Option<u16> = None;
    let mut idle_timeout: Option<u64> = None;
    let mut client_mode: Option<String> = None;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--host" {
            if let Some(value) = args.next() {
                host = Some(value);
            }
            continue;
        }

        if let Some(value) = arg.strip_prefix("--host=") {
            host = Some(value.to_string());
            continue;
        }

        if arg == "--port" {
            if let Some(value) = args.next() {
                port = value.parse().ok();
            }
            continue;
        }

        if let Some(value) = arg.strip_prefix("--port=") {
            port = value.parse().ok();
            continue;
        }

        if arg == "--idle-timeout" {
            if let Some(value) = args.next() {
                idle_timeout = value.parse().ok();
            }
            continue;
        }

        if let Some(value) = arg.strip_prefix("--idle-timeout=") {
            idle_timeout = value.parse().ok();
            continue;
        }

        if arg == "--client" {
            if let Some(value) = args.next() {
                client_mode = Some(value);
            }
            continue;
        }

        if let Some(value) = arg.strip_prefix("--client=") {
            client_mode = Some(value.to_string());
            continue;
        }
    }

    // Allow env var override: RUSTCODE_DAEMON_IDLE_TIMEOUT=<seconds>
    // 0 = disabled; non-zero values are clamped to a minimum of 60s to prevent
    // accidental rapid cycling from misconfigured environments.
    let raw_timeout = idle_timeout
        .or_else(|| {
            std::env::var("RUSTCODE_DAEMON_IDLE_TIMEOUT")
                .ok()?
                .parse()
                .ok()
        })
        .unwrap_or(DEFAULT_IDLE_TIMEOUT_SECS);
    let timeout = if raw_timeout == 0 {
        0
    } else {
        raw_timeout.max(60)
    };

    // Startup default for the client identity. Per-request overrides arrive via
    // the `x-rustcode-client` header and win over this value.
    let mode = match client_mode.as_deref() {
        Some("vscode") => ClientMode::Vscode,
        Some("jetbrains") => ClientMode::Jetbrains,
        Some("webui") => ClientMode::Webui,
        Some("rustcode-air") => ClientMode::RustcodeAir,
        _ => ClientMode::Ide,
    };

    (
        host.unwrap_or_else(|| DEFAULT_HOST.to_string()),
        port.unwrap_or(DEFAULT_PORT),
        timeout,
        mode,
    )
}

#[tokio::main]
async fn main() {
    // Before any config read. The daemon is launched by the VS Code extension
    // rather than by the CLI, so it resolves the tree on its own.
    rustcode_config::distribution::bootstrap_home();

    // Locale: the daemon renders its startup banner and API error messages
    // through t(). It has no --lang flag, so resolve from config `language`
    // then LC_* env (default Simplified Chinese, like the other binaries).
    // A missing/malformed config simply falls through to the default.
    // 全量加载 Config（而非仅取 language），供下方静态访问密钥解析复用。
    let loaded_config =
        rustcode_config::config::Config::load(&rustcode_config::config::Config::default_path())
            .ok();
    {
        let language = loaded_config.as_ref().and_then(|c| c.language);
        rustcode_config::i18n::set_locale(rustcode_config::i18n::resolve_initial_locale(
            None, language,
        ));
    }

    // On Windows, when built as a GUI-subsystem binary (windows_subsystem = "windows"),
    // there is no default console. If launched from a terminal (cmd.exe / PowerShell),
    // re-attach to the parent's console so eprintln!/tracing output is visible.
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
        unsafe {
            AttachConsole(ATTACH_PARENT_PROCESS);
        }
    }

    // Ensure legacy sessions (macOS pre-v4.16 ~/Library/Application Support/rustcode/sessions)
    // are migrated to the canonical location ($RUSTCODE_HOME/sessions) before any handler reads it.
    if let Err(error) = rustcode_capabilities::session::SessionManager::migrate_from_legacy() {
        tracing::warn!("[session] Failed to migrate legacy sessions: {error}");
    }

    let (host, port, idle_timeout_secs, startup_mode) = parse_daemon_args();

    let token_store = rustcode_daemon::auth_token::WebuiTokenStore::new();
    // WebUI 访问密钥优先级链（与 `resolve_daemon_token` 内部顺序一致）：
    //   1. RUSTCODE_DAEMON_TOKEN —— 桥接令牌：主进程拉起本 daemon 时显式下发，必须优先。
    //      否则主进程 shell 里一旦设了 RUSTCODE_ACCESS_KEY 就会连不上自己拉起的 daemon。
    //   2. RUSTCODE_ACCESS_KEY —— 独立 `rustcode-daemon` 部署时由用户设定，用于保护 WebUI。
    //   3. config.access_key —— 持久化配置。
    //   4. 以上皆空 -> 随机 mint 一次性 token。
    // 解析到的密钥会登记进 WebuiTokenStore 并写入 daemon-<port>.json。
    let daemon_token = rustcode_daemon::resolve_daemon_token(
        std::env::var("RUSTCODE_DAEMON_TOKEN").ok(),
        std::env::var("RUSTCODE_ACCESS_KEY").ok(),
        loaded_config.as_ref().and_then(|c| c.access_key.clone()),
        &token_store,
    );
    // 免密开关（本二进制没有 --no-auth 参数，走配置 `webui_no_auth` 或环境变量
    // RUSTCODE_WEBUI_NO_AUTH）。开启后上面解析出的密钥仍然登记进 store、token 文件
    // 照写，但中间件不再校验 —— IDE 客户端不带头也能连。
    let webui_no_auth =
        rustcode_config::config::webui_no_auth_enabled(false, loaded_config.as_ref());
    // 免密时上面那把密钥守不住端口（仍登记、仍写文件，但不校验）—— 说清楚，
    // 免得运维以为配了 access_key 就安全。
    if webui_no_auth
        && rustcode_config::config::webui_no_auth_masks_access_key(loaded_config.as_ref())
    {
        println!(
            "{}",
            rustcode_config::i18n::t(rustcode_config::i18n::Msg::WebuiNoAuthKeyIgnored)
        );
    }

    if let Err(e) = run_server(ServerOpts {
        host,
        port,
        idle_timeout_secs,
        startup_mode,
        webui_tokens: Some(token_store),
        webui_no_auth,
        // 独立二进制：保留完整启动横幅。
        quiet: false,
        // 独立二进制 / VSCode：沿用 config 的 default_workdir，不覆盖。
        working_dir_override: None,
        // 独立二进制自行 bind host:port，不预绑定。
        prebound_listener: None,
        daemon_token_file: Some(daemon_token),
    })
    .await
    {
        eprintln!(
            "{}",
            rustcode_config::i18n::t(rustcode_config::i18n::Msg::DaemonFatalServer {
                error: &format!("{e:#}")
            })
        );
        std::process::exit(1);
    }
}
