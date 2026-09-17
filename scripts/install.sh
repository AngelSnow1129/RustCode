#!/bin/sh
# RustCode installer — curl | sh
#
#   curl -fsSL <installer-url-from-your-distribution-channel>/install.sh | sh
#
# This build ships no built-in release host. Point it at the location that
# distributes rustcode binaries for your channel:
#
#   RUSTCODE_RELEASE_BASE=https://<host>/releases/download sh install.sh
#
# Env overrides:
#   RUSTCODE_RELEASE_BASE        download root that hosts
#                                  "<tag>/rustcode-<tag>-<os>-<arch>" binaries (required)
#   RUSTCODE_RELEASE_LATEST_API  optional JSON endpoint whose "tag_name" field gives
#                                  the latest release tag (for auto-detection)
#   RUSTCODE_VERSION             release tag to install (default: latest release,
#                                  auto-detected from RUSTCODE_RELEASE_LATEST_API when set)
#   RUSTCODE_PREFIX              install dir (absolute path; default: /usr/local/bin if writable,
#                                  else ~/.local/bin). On HarmonyOS as non-root, default is ~/.local/bin.
# IMPORTANT: when changing install paths, the PATH-rc edit format, or filenames here,
# also update scripts/uninstall.sh AND
# crates/rustcode-cli/src/uninstall/paths.rs. The CI parity test guards
# the manifest, but binary path / rc-edit format are not checked.
set -eu

# --- optional arguments: inject a custom bring-your-own-key provider after a
# successful install. Maps to the [providers.<name>] block in config.toml
# (type = "openai-compatible"). Example:
#   curl -fsSL <url>/install.sh | sh -s -- \
#     --url https://my-gw.example.com/v1 --key sk-xxx --model "deepseek-v4.1-flash"
PROVIDER_NAME=""
PROVIDER_URL=""
PROVIDER_KEY=""
PROVIDER_MODEL=""
while [ $# -gt 0 ]; do
    case "$1" in
        --url|--key|--model|--provider)
            [ $# -ge 2 ] || { echo "Error: $1 requires a value" >&2; exit 1; }
            case "$1" in
                --url)      PROVIDER_URL="$2" ;;
                --key)      PROVIDER_KEY="$2" ;;
                --model)    PROVIDER_MODEL="$2" ;;
                --provider) PROVIDER_NAME="$2" ;;
            esac
            shift 2
            ;;
        *)
            echo "Error: unknown option: $1" >&2
            exit 1
            ;;
    esac
done

# `model` is a required field in config.toml's [providers.<name>] block (there is
# no sensible default for an OpenAI-compatible endpoint). Fail closed before any
# download when a provider is being injected without a model.
if [ -z "$PROVIDER_MODEL" ] && [ -n "$PROVIDER_URL$PROVIDER_KEY" ]; then
    echo "Error: --model is required when injecting a provider (--url/--key given)." >&2
    exit 1
fi

# Release source: provided by the operator/distributor via env; there is no
# compiled-in vendor host.
RELEASE_BASE="${RUSTCODE_RELEASE_BASE:-}"
RELEASE_LATEST_API="${RUSTCODE_RELEASE_LATEST_API:-}"

# --- detect platform ---
uname_s=$(uname -s)
uname_m=$(uname -m)
ext=""  # binary filename suffix; ".exe" on Windows shells (set below)

case "$uname_s" in
    Darwin) os="darwin" ;;
    Linux)  os="linux"  ;;
    HarmonyOS) os="ohos" ;;
    # MSYS2 / MinGW / Git-Bash / Cygwin: a Unix shell running ON Windows. `uname -s` looks
    # like MSYS_NT-10.0-26100 / MINGW64_NT-... / CYGWIN_NT-.... Install the native Windows
    # `.exe` into this shell's environment (windows-specific PREFIX + suffix handled below).
    MSYS*|MINGW*|CYGWIN*) os="windows"; ext=".exe" ;;
    *) echo "Unsupported OS: $uname_s (Windows users: download the zip from the release page)"; exit 1 ;;
esac

case "$uname_m" in
    arm64|aarch64) arch="arm64" ;;
    x86_64|amd64)  arch="x64"   ;;
    *) echo "Unsupported arch: $uname_m"; exit 1 ;;
esac

# --- pick install dir ---
if [ -n "${RUSTCODE_PREFIX:-}" ]; then
    PREFIX="$RUSTCODE_PREFIX"
elif [ "$os" = "ohos" ] || [ "$os" = "windows" ]; then
    # Windows shells (MSYS/Git-Bash/Cygwin) have no sudo and a system /usr/local/bin under
    # the MSYS root; install into the user's home instead (always writable, no elevation).
    PREFIX="$HOME/.local/bin"
elif [ -w /usr/local/bin ] 2>/dev/null; then
    PREFIX="/usr/local/bin"
elif [ "$(id -u)" -eq 0 ]; then
    PREFIX="/usr/local/bin"
else
    PREFIX="$HOME/.local/bin"
fi

# This build ships no compiled-in release host. The download root must be
# supplied by the operator/distribution channel; fail with actionable guidance
# instead of guessing a vendor URL. This runs before creating the install dir
# so a misconfigured invocation leaves no empty directory behind.
if [ -z "$RELEASE_BASE" ]; then
    echo "Error: no release download source configured." >&2
    echo "       Set RUSTCODE_RELEASE_BASE to the directory that hosts the" >&2
    echo "       rustcode-<tag>-<os>-<arch> binaries, then re-run, e.g.:" >&2
    echo "         RUSTCODE_RELEASE_BASE=https://<your-distribution-host>/releases/download \\" >&2
    echo "           sh install.sh" >&2
    echo "       Optionally set RUSTCODE_RELEASE_LATEST_API for automatic" >&2
    echo "       latest-version detection, or pin RUSTCODE_VERSION=<tag>." >&2
    echo "       You can also download the binary for your platform directly" >&2
    echo "       from your distribution channel." >&2
    exit 1
fi
RELEASE_BASE="${RELEASE_BASE%/}"

mkdir -p "$PREFIX"

# --- download ---
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
DEST="$TMP/rustcode${ext}"

# Pick download tool: $_fetch streams a URL to stdout (for the API lookup),
# $_down saves a URL to a file (for the binary).
if command -v curl >/dev/null 2>&1; then
    _fetch="curl -sL --connect-timeout 5 --max-time 10"
    _down="curl -fL --progress-bar -o"
elif command -v wget >/dev/null 2>&1; then
    _fetch="wget -qO- --timeout=10 --tries=1"
    _down="wget --show-progress -O"
else
    echo "Error: need curl or wget." >&2
    exit 1
fi

# --- resolve version ---
# Honor RUSTCODE_VERSION if set; otherwise auto-detect the latest release tag
# from RUSTCODE_RELEASE_LATEST_API. Without either, we cannot guess a tag (this
# build has no built-in release host), so fail with guidance.
if [ -n "${RUSTCODE_VERSION:-}" ]; then
    VERSION="$RUSTCODE_VERSION"
elif [ -n "$RELEASE_LATEST_API" ]; then
    echo "==> Detecting latest version"
    VERSION=$($_fetch "$RELEASE_LATEST_API" | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
    if [ -z "$VERSION" ]; then
        echo "Error: could not determine the latest release from:" >&2
        echo "       $RELEASE_LATEST_API" >&2
        echo "       Set RUSTCODE_VERSION explicitly (e.g. RUSTCODE_VERSION=vX.Y.Z)." >&2
        exit 1
    fi
else
    echo "Error: no version specified and no release API configured." >&2
    echo "       Pin a tag with RUSTCODE_VERSION=<tag>, or set" >&2
    echo "       RUSTCODE_RELEASE_LATEST_API to a JSON endpoint that returns" >&2
    echo "       a \"tag_name\" field for automatic latest-release detection." >&2
    exit 1
fi

BIN_NAME="rustcode-${VERSION}-${os}-${arch}${ext}"
URL="${RELEASE_BASE}/${VERSION}/${BIN_NAME}"

echo "==> Downloading $BIN_NAME"
echo "    from $URL"
$_down "$DEST" "$URL"

# Sanity check: must be a real binary, not an HTML 404 page
if head -c 4 "$DEST" | grep -q "<" 2>/dev/null; then
    echo "Error: download looks like an HTML page, not a binary."
    echo "       The release may not exist for your platform, or the URL is wrong."
    echo "       URL: $URL"
    exit 1
fi

chmod +x "$DEST"

# --- install ---
TARGET="$PREFIX/rustcode${ext}"
if [ "$os" = "windows" ]; then
    # No sudo on MSYS/Git-Bash, and PREFIX is the user's own dir. A running rustcode.exe locks
    # the file on NTFS, so `mv` can fail with a lock error — surface a clear hint instead of a
    # raw `set -e` abort (mirrors install.ps1's "close any running rustcode.exe" guidance).
    echo "==> Installing to $TARGET"
    if ! mv "$DEST" "$TARGET"; then
        echo "Error: could not write $TARGET." >&2
        echo "       If rustcode is already running, close it and re-run this installer." >&2
        exit 1
    fi
elif [ -e "$TARGET" ] && [ ! -w "$TARGET" ]; then
    echo "==> Installing to $TARGET (sudo required)"
    sudo mv "$DEST" "$TARGET"
elif [ ! -w "$PREFIX" ]; then
    echo "==> Installing to $TARGET (sudo required)"
    sudo mv "$DEST" "$TARGET"
else
    echo "==> Installing to $TARGET"
    mv "$DEST" "$TARGET"
fi

# --- done ---
echo ""
echo "Installed: $TARGET"
"$TARGET" --version 2>/dev/null || true

# --- write custom provider config (optional --url/--key/--model) ---
write_custom_provider() {
    [ -n "$PROVIDER_URL$PROVIDER_KEY$PROVIDER_MODEL" ] || return 0
    NAME="${PROVIDER_NAME:-custom}"
    CFG_DIR="${RUSTCODE_HOME:-$HOME/.rustcode}"
    CFG="$CFG_DIR/config.toml"
    mkdir -p "$CFG_DIR"

    # Minimal TOML basic-string escape (backslash and double quote).
    esc_toml() { printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'; }

    # Compute this BEFORE the >> redirection below, which would otherwise
    # create the file and make a later -f always true.
    CFG_NEW=0
    [ -f "$CFG" ] || CFG_NEW=1

    # Idempotent: never clobber an existing provider of the same name.
    if [ -f "$CFG" ] && \
        { grep -Fqs "[providers.\"$NAME\"]" "$CFG" || grep -Fqs "[providers.$NAME]" "$CFG"; }; then
        echo "[INFO] provider '$NAME' already present in $CFG; skipped."
        return 0
    fi

    {
        if [ "$CFG_NEW" -eq 1 ]; then
            printf 'default_provider = "%s"\n\n' "$NAME"
        fi
        printf '[providers."%s"]\n' "$NAME"
        printf 'type = "openai-compatible"\n'
        [ -n "$PROVIDER_URL" ]   && printf 'base_url = "%s"\n' "$(esc_toml "$PROVIDER_URL")"
        [ -n "$PROVIDER_KEY" ]   && printf 'api_key = "%s"\n' "$(esc_toml "$PROVIDER_KEY")"
        [ -n "$PROVIDER_MODEL" ] && printf 'model = "%s"\n' "$(esc_toml "$PROVIDER_MODEL")"
        printf '\n'
    } >> "$CFG"

    chmod 600 "$CFG" 2>/dev/null || true
    echo "[INFO] wrote provider '$NAME' to $CFG"
    echo "[INFO] run 'rustcode' and use /provider to add more models if needed."
}
write_custom_provider

if [ "$os" = "windows" ]; then
    echo ""
    echo "Note: installed for this Unix shell (MSYS/MinGW/Git-Bash/Cygwin)."
    echo "      For a system-wide Windows install (cmd / PowerShell PATH), download"
    echo "      install.ps1 from your distribution channel and run:"
    echo "        \$env:RUSTCODE_RELEASE_BASE='https://<your-distribution-host>/releases/download'"
    echo "        powershell -ExecutionPolicy Bypass -File install.ps1"
fi

case ":$PATH:" in
    *":$PREFIX:"*) ;;
    *)
        # Auto-append PATH export to shell rc file
        LINE="export PATH=\"$PREFIX:\$PATH\""
        RC=""
        if [ -n "${ZSH_VERSION:-}" ] || [ "$(basename "${SHELL:-}")" = "zsh" ]; then
            RC="$HOME/.zshrc"
        elif [ -n "${BASH_VERSION:-}" ] || [ "$(basename "${SHELL:-}")" = "bash" ]; then
            RC="$HOME/.bashrc"
        fi

        if [ -n "$RC" ]; then
            # Match the COMPLETE export line we manage (grep -x = whole line, -F =
            # fixed string), not a bare substring of "$PREFIX". A `grep -qF "$PREFIX"`
            # false-positives when the rc merely MENTIONS the prefix as a substring
            # (e.g. an "RustCodeBackup" path), silently skipping the real PATH add.
            if [ -f "$RC" ] && grep -qxF "$LINE" "$RC" 2>/dev/null; then
                # Already present, skip
                :
            else
                echo "" >> "$RC"
                echo "# Added by RustCode installer" >> "$RC"
                echo "$LINE" >> "$RC"
                echo ""
                echo "Added $PREFIX to PATH in $RC"
            fi
            echo ""
            echo "To start using rustcode right now, run:"
            echo ""
            echo "    source $RC"
            echo ""
        else
            echo ""
            echo "Note: $PREFIX is not in your PATH. Add this line to your shell rc:"
            echo "    $LINE"
        fi
        ;;
esac
