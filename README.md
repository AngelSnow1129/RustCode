<div align="center">
<pre>
      _   _                  ____          _
     / \ | |_ ___  _ __ ___ / ___|___   __| | ___
    / _ \| __/ _ \| '_ ` _ \ |   / _ \ / _` |/ _ \
   / ___ \ || (_) | | | | | | |__| (_) | (_| |  __/
  /_/   \_\__\___/|_| |_| |_|\____\___/ \__,_|\___|
</pre>
</div>

<p align="center">
  <strong>Open-source terminal AI coding agent written in Rust</strong>
</p>

<p align="center">
  English · <a href="./README.zh-CN.md">简体中文</a>
</p>

<p align="center">
  <a href="#installation">Install</a> ·
  <a href="#quick-start">Quick Start</a> ·
  <a href="#features">Features</a> ·
  <a href="#architecture">Architecture</a> ·
  <a href="#development">Development</a> ·
  <a href="#contributing">Contributing</a> ·
  <a href="#community">Community</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/version-5.0.9-blue" alt="version">
  <img src="https://img.shields.io/badge/rust-1.88%2B-orange" alt="rust">
  <img src="https://img.shields.io/badge/license-MIT-green" alt="license">
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20HarmonyOS%20PC%20%7C%20Windows-lightgrey" alt="platform">
</p>

---

> **This project is 100% AI-generated.** Every line of code, every architectural decision's implementation, and every commit was written by AI. The human developer serves solely as the decision-maker and product manager — defining what to build, not how to build it.

---

RustCode is an AI coding agent that lives in your terminal. Give it a task in natural language, and it will read your codebase, edit files, run commands, and verify its work — autonomously.

Think of it as an open-source alternative to Claude Code / Cursor Agent, but running entirely in your terminal and connecting to any OpenAI-compatible API.

---

> **Fork notice.** This repository is a secondary-development
> fork of `atomgit_atomcode/atomcode`. Relative to upstream it (1) renames the
> product to `rustcode` (crates, binaries, config dir `~/.rustcode`, `RUSTCODE_*`
> env vars); (2) removes **all** telemetry/analytics — the `rustcode-telemetry`
> crate and every reporting call site are gone; (3) **fully decouples from any
> platform** — no host is hard-coded as a signing gateway, no platform-specific
> REST tools are registered by default, and `/login` is optional: configure a
> provider directly in `~/.rustcode/config.toml` with your own `base_url` and
> `api_key` and start coding; (4) **defaults to Simplified Chinese** for both the
> TUI/CLI interface and agent replies (override with `--lang en`, the config
> `language` field, or `LANG`/`LC_ALL`). The original MIT license and copyright (© 2026 Yubang
> Xu) are preserved in [docs/ORIGINAL_LICENSE.md](docs/ORIGINAL_LICENSE.md), with
> full attribution in [docs/UPSTREAM_CREDITS.md](docs/UPSTREAM_CREDITS.md).

## Features

### Agent Loop

- **Autonomous multi-step execution** — reads files, edits code, runs tests, fixes errors, all in a loop
- **Verification loop** — automatically verifies edits via syntax checks before declaring success
- **Dynamic step budget** — scales with the number of edited files, capped per turn to bound cost
- **Loop detection** — detects and breaks out of repetitive tool-call patterns
- **3-layer JSON repair** — recovers malformed tool-call arguments
- **Turn-level datalog** — structured per-turn logs for replay, debugging, and eval harnesses

### Modes & Autonomy

- **Plan / Build modes** — `/plan` switches to read-only exploration (the agent investigates without touching files); `/build` switches back to full execution
- **Goal mode** — `/goal <text>` sets a completion condition and the agent loops autonomously, turn after turn, until the goal is met
- **Code review** — `/review` reviews your current changes, `/review staged` the staged diff, and `/review <base>` against a base ref
- **Background sessions** — `/bg` runs work in detached slots so you can keep using the TUI while a long task progresses

### Built-in Tools

File & shell:

- `read_file`, `write_file`, `edit_file`, `search_replace`
- `bash`, `grep`, `glob`, `list_directory`, `change_dir`
- `web_search`, `web_fetch`

Code graph (language-aware code intelligence):

- `list_symbols`, `read_symbol`, `find_references`
- `trace_callers`, `trace_callees`, `trace_chain`
- `file_deps`, `blast_radius`

Automation:

- `auto_fix` — automatic lint/typecheck fix loop
- `use_skill` — invoke a user-defined skill

### Multi-Provider Support

Connect to any LLM that supports OpenAI's function-calling API:

| Provider                  | Function Calling | Tested Models                         |
| ------------------------- | :--------------: | ------------------------------------- |
| Claude (Anthropic)        |       Yes        | Claude Sonnet 4.5/4.6, Opus 4.6       |
| OpenAI                    |       Yes        | GPT-4o, GPT-4.1                       |
| DeepSeek                  |       Yes        | DeepSeek V3, DeepSeek R1, DeepSeek V4 |
| Zhipu (GLM)               |       Yes        | GLM-4, GLM-5, GLM-5.2                 |
| Qwen (Alibaba)            |       Yes        | Qwen-Plus, Qwen-Max                   |
| SiliconFlow               |       Yes        | Various open models                   |
| Ollama (local)            |     Partial      | Llama 3, Qwen2, etc.                  |
| Any OpenAI-compatible API |       Yes        | —                                     |

### Sessions & Login

- **Persistent sessions** — every conversation is saved; continue the last session with `rustcode --continue` / `-c`, or resume/switch inside the TUI with `/resume`
- **Third-party providers (BYO)** — configure your own `base_url` + `api_key` in `~/.rustcode/config.toml` (or via `/provider`); no account or signup needed. This is the default in the open-source build
- **OAuth login** — `/login` (or `rustcode login`) pairs your CLI with a managed account; **distribution builds only** (the open-source build ships no managed service)
- **SSO login** — `/login-with-sso` for SSO-enabled managed deployments (distribution builds only)
- **Headless mode** — `rustcode -p "..."` runs a single prompt non-interactively and streams the reply on stdout (Claude Code `-p` style); approval-required `bash` calls are auto-approved, while other approval-required tools are denied
- **Daemon mode** — `rustcode-daemon` exposes an HTTP API for session history and SSE streaming chat

### Terminal UI

- **Real-time streaming** with markdown rendering and syntax highlighting
- **Code blocks** with language labels, line numbers, and `base16-ocean.dark` theme
- **Multi-line input** with Shift+Enter (or `\` + Enter), auto-growing height, input history
- **Task completion notifications** — long-running tasks trigger terminal-native notifications first (kitty / WezTerm / iTerm2), falling back to OS-native alerts
- **Text selection** with mouse drag, auto-scroll, and clipboard copy
- **Slash commands** — `/model`, `/provider`, `/resume`, `/bg`, `/diff`, `/undo`, `/cost`, `/clear`, `/compact`, etc. (see table below)
- **File attachment** — paste file paths to attach content as context
- **Bracketed paste** — long paste content collapsed to a compact indicator
- **Skills** — user-defined commands loaded from your skill directory, invoked like any slash command

### Web UI

- **`/webui`** (in the TUI) or **`rustcode webui`** (CLI) launches a local browser UI as an alternative to the terminal interface — same agent, same sessions, rendered in your browser
- **Loopback only** — the server binds to `127.0.0.1` and uses a one-time token; nothing is exposed to the network
- **`/webui stop`** stops the in-process server (a later `/webui` restarts it)

### App Remote Access

- **`/app`** (in the TUI) enables mobile remote access — prints a QR code; scan it with the mobile app from any network to connect to your current session
- **Any-network reachable** — your PC connects to a public relay via a reverse WSS tunnel; the phone reaches your PC through the relay. No public IP, DDNS, or port forwarding required
- **Bidirectional real-time sync** — messages from either end appear on the other in real time (streaming replies, tool call cards, token usage)
- **Remote commands** — the phone can run `/status`, `/cost`, `/diff`, `/whoami` etc., which execute on the desktop and echo results back
- **Switch projects / sessions** — switch projects or open a history session on the phone, and the desktop follows immediately
- **Model sync** — switching models on either end keeps the other in sync
- **`/app stop`** disconnects remote access

### Safety

- **Destructive command detection** — `rm -rf`, `git push --force`, `DROP TABLE`, etc. require explicit approval
- **Path-aware confirmations** — external reads, sensitive paths, and all writes outside the workspace can require confirmation depending on risk level
- **Sensitive file protection** — protected system paths, credential directories, shell configs, `.env` files, and key/cert files receive stronger confirmation rules
- **Shell bypass protection** — common shell file commands like `cat`, `head`, `ls`, `cp`, `mv`, and `tee` inherit the same path approval model as file tools
- **Per-session permission grants** — approve once per tool pattern, or always-allow
- **Source file deletion requires approval** — `rm` on code files is never auto-approved
- **Undo** — `/undo` rolls back the last turn's file edits via file-history snapshots

See [Permission Model](./docs/security/permission-model.md) for the full design and current boundaries.

### Privacy

- [CHECK] Zero telemetry — this fork (RustCode) has removed the entire reporting
  pipeline (`rustcode-telemetry` crate deleted). No events, usage stats, or crash
  reports are sent anywhere. See [docs/telemetry.md](docs/telemetry.md) for what was
  removed, and [docs/ORIGINAL_LICENSE.md](docs/ORIGINAL_LICENSE.md)
  and [docs/UPSTREAM_CREDITS.md](docs/UPSTREAM_CREDITS.md) for provenance.

## Installation

### Installation Script (recommended)

The installer ships no built-in release host: obtain `install.ps1` (Windows
PowerShell) or `install.sh` (Linux / macOS / WSL / MSYS / Git-Bash / HarmonyOS
PC) from your distribution channel — e.g. the release assets page of the
channel you got RustCode from — and run it as follows.

For Windows PowerShell users:

```powershell
# Obtain install.ps1 from your distribution channel's release assets, then
# point it at that channel's release downloads directory:
$env:RUSTCODE_RELEASE_BASE = "https://example.com/your-host/releases/download"
./install.ps1
```

For Linux / macOS / WSL / MSYS / Git-Bash / HarmonyOS PC users:

```bash
# Obtain install.sh from your distribution channel, then point it at that
# channel's release downloads directory:
RUSTCODE_RELEASE_BASE=https://example.com/your-host/releases/download sh install.sh
```

The script downloads the prebuilt binary for the latest release
(auto-detected from the release API when your channel provides one), installs
it, and adds it to your `PATH`. Builds from a distribution channel that ships
the optional managed-signing component can use `/login` with that channel's
managed CodingPlan endpoint (see "About the optional CodingPlan gateway"
below); bring-your-own-key third-party providers need no signer.

Environment variable overrides: `RUSTCODE_RELEASE_BASE` sets the download root
that hosts the release binaries (required for the shell installer),
`RUSTCODE_VERSION` pins a release tag, and `RUSTCODE_PREFIX` picks the install
directory (see the script headers for details).

### From Source

```bash
# Clone from your distribution channel, e.g.:
git clone https://example.com/<your-org>/rustcode.git
cd rustcode
```

#### WebUI build (required for the webui feature — runs before the Rust build)

The `rustcode webui` browser UI is embedded into the binary from `webui/dist/`,
which is gitignored (not committed). The Rust build needs no Node.js toolchain
and succeeds without it, but a binary built without `webui/dist/` serves
`webui not built` for every webui page. To get a working webui, build the
frontend before the Rust build:

```bash
cd webui
npm ci
# for Windows MSYS / Git Bash users, run
# `PATH="/c/Program Files/nodejs:$PATH" npm run build`
# to use the system-installed Node.js.
npm run build    # outputs webui/dist/, embedded by the next Rust build
cd ..
```

Skip this step if you don't use the webui. The release scripts build the
frontend automatically before `cargo build`. After rebuilding the frontend,
force a daemon recompile so the new bundle is re-embedded (`cargo clean -p
rustcode-daemon`) — cargo does not track changes under `webui/dist/`. Then
build and install:

```bash
cargo install --path crates/rustcode-cli --locked
```

The binary will be generated at `target/release/rustcode` and installed to
`~/.cargo/bin/rustcode` for macOS / Linux / HarmonyOS PC and `$env:USERPROFILE/.cargo/bin/rustcode.exe`
for Windows. Make sure that `~/.cargo/bin` (or `%USERPROFILE%\.cargo\bin` on Windows) is
in your `PATH`.

To compile without installing, run:

```bash
# Builds only the CLI package (`rustcode`) — skips the standalone
# `rustcode-daemon` binary and other workspace members
cargo build --release -p rustcode
```

and the binary will be generated at `target/release/rustcode`.

### About the optional CodingPlan gateway (closed-source signer)

`crates/rustcode-codingplan-crypto/` in this repository is an open-source
placeholder. The real request-signing implementation is closed-source and is
only overlaid by distribution release pipelines that opt into the managed
CodingPlan gateway, so a self-built binary cannot sign requests to such a
managed endpoint. Binaries obtained from a distribution channel that ships
the signer (for example via the installer above or the package managers
below) include it. In practice this means:

- `/login` cannot claim a managed gateway's **free CodingPlan models** in
  self-built binaries. Signing is kept closed-source to prevent the free plan
  from being abused outside that channel's builds.
- Connecting **your own third-party providers** (bring your own key) is
  unaffected and is the default: any provider configured under `providers.*`
  in `~/.rustcode/config.toml` (DeepSeek, OpenAI, or any OpenAI-compatible
  endpoint) works without the signer.

### Package Managers

RustCode CLI can also be installed via the following package managers:

```bash
# Install using npm
npm install -g @rustcode/rustcode

# Install using Homebrew
brew install --cask rustcode
```

### Shell Completion

RustCode can generate completion scripts for Bash, Zsh, Fish, PowerShell, and
Elvish. For example:

```bash
# Bash (current session)
source <(rustcode completion bash)

# Zsh (persistent)
mkdir -p ~/.zfunc
rustcode completion zsh > ~/.zfunc/_rustcode
# Also add `fpath=(~/.zfunc $fpath)` before `compinit` in ~/.zshrc.

# Fish (persistent)
mkdir -p ~/.config/fish/completions
rustcode completion fish > ~/.config/fish/completions/rustcode.fish
```

For PowerShell, run `rustcode completion powershell | Out-String |
Invoke-Expression`. Run `rustcode completion --help` for the complete shell
list. This affects command-line completion only; inside the TUI, `Tab` completes
input and `Shift+Tab` cycles execution mode.

### Requirements

- Rust 1.88+ (for building; older Cargo versions cannot parse the current lockfile)
- An API key from any supported provider (bring your own key; or a platform
  account for `/login` with a managed gateway — the free CodingPlan models
  require a channel build that ships the signer; see "About the optional
  CodingPlan gateway" above)

### Permissions — don't run with `sudo`

Run RustCode as your **normal user**, never with `sudo`. RustCode keeps its
config, sessions, and logs under `~/.rustcode`; running once as root leaves
root-owned files there, so every later non-root start fails at runtime init with:

```
coding runtime assemble failed: Permission denied (os error 13)
```

(the message may say `prepare` instead of `assemble` — same cause.) If you hit
this, reclaim ownership and stop using `sudo`:

```bash
sudo chown -R "$(id -un):$(id -gn)" ~/.rustcode
rustcode        # start WITHOUT sudo
```

On a Linux guest, a working directory on a VirtualBox shared folder
(`/media/sf_*`, owned by `root:vboxsf`) can also trigger permission errors — add
yourself to the group with `sudo usermod -aG vboxsf "$USER"` and re-login, rather
than using `sudo`.

### Uninstall

Remove RustCode and (optionally) its data:

```bash
rustcode uninstall                # interactive: per-group prompts
rustcode uninstall --keep-data    # only remove binary + PATH edit
rustcode uninstall --purge        # remove everything, including ~/.rustcode
rustcode uninstall --dry-run      # show plan, change nothing
```

If the binary is already broken or missing, obtain `uninstall.sh` (or
`uninstall.ps1` on Windows) from your distribution channel and run it. The
uninstaller only removes the local install, so it needs no download base:

```bash
sh uninstall.sh
# Windows PowerShell: run the uninstall.ps1 obtained from your channel
./uninstall.ps1
```

By default credentials (`auth.toml`, `mcp.json`, `config.toml`, `RUSTCODE.md`) are kept; pass `--purge` to remove them too.

## Quick Start

### 1. First Run

```bash
rustcode
```

On first run, a setup wizard will guide you through configuring your LLM provider:

```
Welcome to RustCode! Let's set up your first provider.

Select provider:
  [1] Claude (Anthropic)
  [2] OpenAI
  [3] OpenAI Compatible (DeepSeek, Qwen, Zhipu, Moonshot...)
  [4] Ollama (local)
```

### 2. Configuration

Config is stored at `~/.rustcode/config.toml`. A minimal single-provider
setup looks like this:

```toml
default_provider = "deepseek"

[providers.deepseek]
type           = "openai"
api_key        = "sk-..."
model          = "deepseek-chat"
base_url       = "https://api.deepseek.com/v1"
context_window = 64000
```

You can declare multiple providers and switch between them with `/model`
or `/provider`. A **complete reference** covering Claude / OpenAI /
OpenAI-compatible endpoints (DeepSeek, GLM, SiliconFlow, OpenRouter...) /
Ollama, plus the `[datalog]` section, lives at
[`docs/config.example.toml`](docs/config.example.toml) — copy and edit the
bits you need.

After editing `config.toml` by hand, run `/reload` inside rustcode to pick
up the changes without restarting.

### 3. Start Coding

```bash
# Open in your project directory
cd your-project
rustcode

# Or specify directory
rustcode -C /path/to/project

# Or specify model
rustcode --model gpt-4o

# Headless (single prompt, reply on stdout)
rustcode -p "Explain the agent loop in this repo"

# Read prompt from file
rustcode --prompt-file task.md
```

In headless mode, approval-required `bash` calls are auto-approved and logged to stderr; other approval-required tools are denied.

Then just type what you want:

```
> Fix the login bug where users get redirected to 404 after OAuth callback

> Add a dark mode toggle to the settings page

> Refactor the database module to use connection pooling

> Write tests for the payment processing module
```

## Keybindings

### Input

| Key | Action |
|-----|--------|
| `Enter` | Send message |
| `Shift+Enter` | New line (requires Kitty keyboard protocol) |
| `Ctrl+Enter` | New line (requires Kitty keyboard protocol) |
| `Ctrl+J` | New line (when the terminal distinguishes the chord) |
| `Alt+Enter` | New line (most terminals; see compatibility note below) |
| `\` + `Enter` | New line (works on all terminals — type a `\` and press Enter; the `\` is consumed) |
| `Esc` | Clear input / Cancel stream |
| `Esc` ×2 | Undo the previous turn |
| `Up/Down` | Browse input history |
| `Tab` | Accept slash-command, skill, or file completion |
| `Shift+Tab` | Cycle to the next execution mode when no completion menu is open |
| `F2 / Shift+F2` | Switch to next / previous model (usually `Fn+F2 / Fn+Shift+F2` on Mac) |
| `Ctrl+R` | Reverse-search input history |
| `Ctrl+T` | Cycle `reasoning_effort` |
| `Ctrl+U` | Clear line |
| `Ctrl+W` | Delete word |
| `Ctrl+K` | Delete to end of line |
| `Ctrl+V / Ctrl+Alt+V` | Paste text or image from clipboard (Windows can also use `/paste`) |

> **Terminal compatibility for newline chords:**
>
> - `Shift+Enter` and `Ctrl+Enter` need a terminal that speaks the Kitty keyboard protocol — kitty, WezTerm, Alacritty, iTerm2 ≥3.5, Windows Terminal ≥1.21. Older terminals (and Windows, where rustcode doesn't enable the protocol) collapse them to plain `Enter` (which sends the message) — use `\` + `Enter`, which works everywhere.
> - RustCode enables the Kitty keyboard protocol only for known-compatible terminals. Generic web terminals such as JumpServer use legacy key reporting by default. Set `RUSTCODE_KITTY=1` to force it on or `RUSTCODE_KITTY=0` to force it off.
> - `Alt+Enter` works at the byte level on most terminals, but **Windows Terminal binds it to "toggle full screen" by default** — remove that binding under Settings → Actions to free it up.
> - Xshell does not support the Kitty protocol; in its keymap settings, map a free chord to send `ESC, Enter` (`\x1b\r`) to get the same effect, or paste multi-line text via the clipboard (bracketed paste is enabled).

> **Pasting images on Windows:**
> Windows Terminal and conhost bind `Ctrl+V` to their own `paste` action, which only forwards `CF_UNICODETEXT` from the clipboard — an image-only clipboard sends nothing, so the in-app `Ctrl+V` handler never fires. Two ways out:
>
> 1. Use **`/paste`** — the slash command pulls the clipboard image and attaches it as `[Image #N]`. Works in every terminal, including Windows Terminal, PowerShell 7, conhost, and git bash. The TUI's bottom-right hint on Windows says `Image in clipboard · /paste` automatically.
> 2. If you want `Ctrl+V` muscle memory: open Windows Terminal `settings.json` (`Ctrl+,` → "Open JSON file") and either delete the `{ "command": "paste", "keys": "ctrl+v" }` entry under `"actions"`, or rebind it to `ctrl+shift+v`. After a restart, `Ctrl+V` passes through to rustcode.
>
> Git Bash (MinTTY) doesn't intercept `Ctrl+V`, so it works there out of the box.

### Navigation

| Key                      | Action                                |
| ------------------------ | ------------------------------------- |
| `Shift+Up/Down`          | Scroll chat one line                  |
| `PageUp/PageDown`        | Scroll chat 10 lines                  |
| `Alt+Up/Down`            | Jump to previous / next message       |
| `Ctrl+Up/Down`           | Jump to previous / next user message  |
| Empty input + `Home/End` | Jump to top / bottom of conversation  |
| `Ctrl+Shift+C`           | Copy selection                        |
| `Ctrl+C`                 | Cancel operation (double-tap to exit) |

### Slash Commands

Type `/` in the TUI to browse the full list with live completion; `/help` shows commands and shortcuts.

**Sessions & workspace**

| Command              | Action                                                                                     |
| -------------------- | ------------------------------------------------------------------------------------------ |
| `/resume`            | Resume or switch session                                                                   |
| `/session`           | Start a new session                                                                        |
| `/rename <name>`     | Rename the current session                                                                 |
| `/clear`             | Start a new conversation (clears context + screen)                                         |
| `/bg`                | Background current session; subcommands: `/bg list`, `/bg <N>`, `/bg drop <N>`, `/bg help` |
| `/background <task>` | Compatibility alias: start a one-shot task in a `/bg` slot                                 |
| `/cd`                | Change working directory and start a new session                                           |
| `/worktree`          | Git worktree isolation (`create` / `list` / `done` / `cleanup`)                            |
| `/webui`             | Launch the browser webui (subcommands: `stop`, `lan`, `--host <addr>`)                     |
| `/sync`              | Attach to the live webui session (`/sync off` to detach)                                   |

**Modes, autonomy & review**

| Command        | Action                                                                            |
| -------------- | --------------------------------------------------------------------------------- |
| `/plan`        | Switch to Plan mode (read-only exploration)                                       |
| `/build`       | Switch to Build mode (full execution)                                             |
| `/goal <text>` | Set a completion goal — the agent loops autonomously until it's met               |
| `/review`      | Code review the current changes (`/review` · `/review staged` · `/review <base>`) |
| `/think`       | Control extended thinking (on / off / budget N)                                   |
| `/effort`      | DeepSeek reasoning effort control (high / max / off)                              |

**Providers & account**

| Command     | Action                                                      |
| ----------- | ----------------------------------------------------------- |
| `/model`    | Switch model / provider                                     |
| `/provider` | Manage providers (add / edit / delete)                      |
| `/proxy`    | Switch outbound proxy mode                                  |
| `/login`    | Sign in with OAuth to a managed service (distribution builds only; use `/provider` for BYO in the open-source build) |
| `/logout`   | Sign out of a managed account (distribution builds only) |
| `/whoami`   | Show the current managed-account user (distribution builds only) |
| `/status`   | Show provider status and model info (distribution builds also show the managed-account sign-in section) |

**Files, edits & context**

| Command            | Action                                                                  |
| ------------------ | ----------------------------------------------------------------------- |
| `/diff`            | Show git diff of current changes                                        |
| `/undo`            | Undo a turn's file edits (`/undo` or `/undo N`)                         |
| `/view <filepath>` | View file content in an overlay modal                                   |
| `/paste`           | Attach an image from the clipboard (Windows fallback for Ctrl+V)        |
| `/copy`            | Copy a code block from the last reply (`/copy`, `/copy N`, `/copy all`) |
| `/cost`            | Show token usage for this session                                       |
| `/context`         | Show the context budget breakdown                                       |
| `/compact`         | Compact conversation history                                            |

**Memory**

| Command            | Action                                              |
| ------------------ | --------------------------------------------------- |
| `/remember <fact>` | Save a fact to memory (`--global` for all projects) |
| `/forget <query>`  | Remove matching memories                            |
| `/memory`          | Show all saved memories                             |

**Extensions**

| Command   | Action                                                                |
| --------- | --------------------------------------------------------------------- |
| `/mcp`    | MCP server status (subcommands: `reload`, `tools`, `login`, `logout`) |
| `/plugin` | Plugin marketplace (`marketplace` / `install` / `uninstall` / `list`) |
| `/skills` | Browse loaded skills                                                  |

**Project & system**

| Command | Action |
|---------|--------|
| `/init` | Create or improve the active project instruction file, following the current language and optional custom prompt |
| `/config` | Show config path |
| `/reload` | Reload `~/.rustcode/config.toml` from disk |
| `/upgrade` | Upgrade rustcode to latest (subcommand: `rollback`) |
| `/setup` | First run: install the recommended skill and run it |
| `/welcome` | Re-run the onboarding wizard |
| `/language` | Switch display and default Git commit-message language |
| `/guide <question>` | Ask rustcode-guide how to use RustCode |
| `/keys` | Show keyboard shortcuts |
| `/help` | Show commands & shortcuts |
| `/quit`, `/exit` | Exit RustCode (or Ctrl+C ×2) |

> **Platform issues.** `/issue` has been removed. In distributions that ship
> managed-platform support, after `/login` you can ask in natural language—for
> example, “Create an issue for this bug on our platform”—and RustCode selects
> the built-in `platform_issue` tool. Reading issues is automatic; creating an
> issue or adding, editing, or deleting comments still requires approval.
>
> **Plugin commands.** Beyond the built-ins above, plugins can register their own slash commands. Add the plugin marketplace provided by your distribution channel (set the marketplace URL via config/env or install from your distribution's plugin index), then install plugins from it. For example, a channel community plugin might expose a `/wechat` command showing the community group QR code:
>
> ```text
> /plugin marketplace add https://example.com/<your-org>/rustcode-plugins
> /plugin install <plugin>@<channel>
> ```

### Custom Commands

Beyond built-ins and plugin commands, you can define your own slash commands as Markdown template files — perfect for prompt patterns you use frequently.

**Locations** (lowest to highest priority):

| Location                                                     | Scope                                                |
| ------------------------------------------------------------ | ---------------------------------------------------- |
| `$RUSTCODE_HOME/commands/` (default `~/.rustcode/commands/`) | Global — applies to every project                    |
| `<project>/.rustcode/commands/`                              | Project-level — overrides same-named global commands |
| `plugins/<name>/commands/`                                   | Plugin-contributed — installed via `/plugin install` |

**File format:**

```markdown
---
name: explain
description: Explain how a specific function or module works
args: required
---

Explain the following code in detail:

$ARGUMENTS

Cover: function signature & parameters, core business logic, data flow & side effects.
```

- **`name`** — Required. The command name; `/explain` triggers it.
- **`description`** — Optional. Shown in Tab completion.
- **`args`** — Optional. Controls argument expectation and UX:

  | Value            | Menu Enter                           | Empty-arg submit            |
  | ---------------- | ------------------------------------ | --------------------------- |
  | `none` (default) | Execute immediately                  | Accepted (substitutes `""`) |
  | `optional`       | Complete to `/name `, wait for input | Accepted                    |
  | `required`       | Complete to `/name `, wait for input | Rejected with error message |

  The template variable `$ARGUMENTS` / `${ARGUMENTS}` is always replaced with whatever the user types after the command name (empty string if nothing is typed).

- **Template body** — The prompt sent to the AI when the command is invoked. `$ARGUMENTS` or `${ARGUMENTS}` is replaced with whatever the user types after the command name.

**Example: create a code-review command**

```bash
mkdir -p .rustcode/commands

cat > .rustcode/commands/codereview.md << 'EOF'
---
name: codereview
description: Review the current git diff
args: optional
---

Review all changes in the current git diff.
If specific files are given, review only: $ARGUMENTS
EOF
```

Run `/help commands` to list all loaded custom commands.

> **Priority rule.** A custom command cannot shadow a built-in command with the same name. If a built-in `/review` already exists, a project-level `review.md` won't appear in completion or dispatch.

## Architecture

RustCode is a layered Rust workspace:

```
rustcode/
  crates/
    rustcode-kernel/        # Neutral agent loop and runtime traits
    rustcode-capabilities/  # Providers, tools, MCP, skills, sessions, memory
    rustcode-coding/        # Coding specialization and CodingRuntime lifecycle
    rustcode-review/        # Review specialization
    rustcode-tuix/          # Terminal UI
    rustcode-cli/           # TUI and headless entry point
    rustcode-daemon/        # HTTP/SSE/WebSocket transport + legacy session importer
```

The coding path is `CLI/TUI/daemon → CodingRuntime → kernel`. The retired core
agent protocol and `rustcode-bridge` are no longer part of the runtime path.

### Design Principles

1. **Tech-stack agnostic** — never hardcodes language-specific logic. Detects project type dynamically from descriptor files (`package.json`, `Cargo.toml`, `pyproject.toml`, `pom.xml`, etc.).

2. **Single runtime owner** — `CodingRuntime` owns the live coding agent, provider/session lifecycle, pending requests, snapshots, and controllers. Drivers handle input, presentation, and transport without rebuilding a second agent runtime.

3. **Tool safety** — all destructive operations require explicit user approval. Tool failures become LLM observations, never panics.

4. **Context-aware** — token-budget-aware conversation windowing, project file-tree injection, and per-turn system reminders keep the model focused without exceeding context limits.

5. **Directed dependencies** — kernel stays neutral; capabilities and coding stay free of `rustcode-core`; legacy session data is handled at an explicit compatibility boundary rather than as a runtime fallback.

## Project Instruction File

Create a `.rustcode.md` file in your project root to give RustCode persistent context:

```markdown
# Project Instructions

This is a Vue 3 + TypeScript project using Pinia for state management.

- Always use Composition API with `<script setup>`
- Use TailwindCSS for styling, no inline styles
- Run `npm run lint` after editing .vue/.ts files
```

RustCode reads this file automatically and includes it in the system prompt. RustCode also supports `AGENTS.md` (the [open standard](https://agents.md/) for AI coding agents) as an alternative — if both files exist, `.rustcode.md` takes priority.

Run `/init` to analyze the repository and create or improve the active instruction file. Its output follows the current `/language`. To append organization-specific requirements, set **Custom /init prompt file** in `/config`, or add `init_prompt_file = "prompts/init.md"` to `$RUSTCODE_HOME/config.toml`; relative paths resolve from `$RUSTCODE_HOME`.

## Development

### Prerequisites

- **Rust 1.88+** — install via [rustup](https://rustup.rs/)
- **Git**
- A supported LLM provider API key (for runtime testing)

### Build from Source

```bash
# Clone from your distribution channel, e.g.:
git clone https://example.com/<your-org>/rustcode.git
cd rustcode

# Debug build (fast compilation, slower runtime)
cargo build

# Release build (slower compilation, optimized binary)
cargo build --release
```

### Run in Development

```bash
# Run the TUI directly (debug mode)
cargo run -p rustcode-cli

# With arguments
cargo run -p rustcode-cli -- -C /path/to/project
cargo run -p rustcode-cli -- --model gpt-4o

# Headless mode
cargo run -p rustcode-cli -- -p "summarize this repo"

# Daemon (HTTP API)
cargo run -p rustcode-daemon
```

### Testing

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p rustcode-capabilities
cargo test -p rustcode-tuix

# Run a specific test
cargo test -p rustcode-capabilities test_name
```

### Useful Commands

```bash
# Check compilation without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy

# Build and install to ~/.cargo/bin
cargo install --path crates/rustcode-cli
```

## Contributing

Contributions are welcome! RustCode is in active development.

### How to Contribute

1. **Fork** the repository
2. **Clone** your fork locally (use your distribution channel's host):
   ```bash
   git clone https://example.com/<your-username>/rustcode.git
   cd rustcode
   ```
3. **Create a branch** for your change:
   ```bash
   git checkout -b feat/your-feature
   # or
   git checkout -b fix/your-bugfix
   ```
4. **Make your changes**, ensure the project builds and tests pass:
   ```bash
   cargo build && cargo test && cargo clippy
   ```
5. **Commit** with a clear message:
   ```bash
   git commit -m "feat: add xxx support"
   ```
6. **Push** and open a **Pull Request** against `main`

### Branch Naming

| Prefix      | Purpose                               |
| ----------- | ------------------------------------- |
| `feat/`     | New feature                           |
| `fix/`      | Bug fix                               |
| `refactor/` | Code refactoring (no behavior change) |
| `docs/`     | Documentation only                    |
| `chore/`    | Build, CI, tooling changes            |

### Guidelines

- Follow the project's core principles — especially **tech-stack neutrality**
  (no language/framework-specific logic in the core engine; detect via probes
  like `package.json` / `Cargo.toml` / `pom.xml` and route through adapters)
- All tool failures must be graceful — return the error as an observation to the LLM, never panic
- Destructive operations must require user approval
- Keep the system prompt compact (~1.5K tokens)
- Run `cargo fmt` and `cargo clippy` before submitting

### Where to Start

- **Add a new tool** — implement the `Tool` trait in `crates/rustcode-capabilities/src/tools/`
- **Add a new provider** — implement `LlmProvider` in `crates/rustcode-capabilities/src/provider/`
- **Improve the UI** — rendering lives in `crates/rustcode-tuix/src/render/`
- **Fix bugs** — check your distribution channel's issue tracker for open bugs

### Non-Rust Contributions

Don't know Rust? No problem! There are many ways to contribute without writing Rust code:

- **[*] Documentation** — Improve the README, fix typos, enhance the [docs site](site/docs/en/index.html), or add examples. Docs live in the root `docs/` directory, `site/docs/`, and the main README files.
- **[*] Localization & Translation** — Help translate the docs site, README, or UI strings into more languages. Check `site/docs/` for existing translations.
- **[*] Skills & Plugins** — Create new skills (Markdown + JSON, no Rust needed) that extend RustCode's capabilities, or package them for your distribution's plugin index. Skills are loaded from `~/.rustcode/skills/`.
- **[*] Bug Reports** — Found a bug? Open an issue in your distribution channel's issue tracker with clear reproduction steps, screenshots, and environment info. High-quality bug reports are invaluable.
- **[*] Test Cases & Examples** — Add test scenarios, example projects, or usage demos that help validate features and onboard new users.
- **[*] Community Support** — Help answer questions in the community group, write tutorials, or create video guides.

Every contribution, code or not, makes RustCode better for everyone. When in doubt, open an Issue or start a Discussion!

## Community

---

Scan the community QR code shared by your distribution channel (e.g. a WeChat group QR) to join the RustCode community — share feedback, report issues, and talk to other users and maintainers. The QR image asset itself is published by your channel:

<p align="center">
  <em>[ Community QR code image — your distribution channel publishes this asset ]</em>
</p>

## Donate

---

RustCode is free, open-source software that works with any third-party provider you bring your own key for. If it has saved you a bit of time, consider buying the maintainers a coffee — it keeps us motivated to keep making it better.

<p align="center">
  <em>[ Donate QR codes (Alipay / WeChat Pay) — supplied by your distribution channel ]</em>
</p>

## License

MIT License. See [LICENSE](LICENSE) for details.

---

<p align="center">
  Built with Rust, ratatui, and a lot of late nights.
</p>
