#!/bin/sh
# RustCode installer — curl | sh
#
#   curl -fsSL https://gitcode.com/api/v5/repos/SecLab/RustCode/raw/scripts/install.sh?ref=dev | sh
#
# Detects OS and architecture automatically, downloads the latest release
# binary from GitCode, and installs it to PATH.
#
# Env overrides (optional):
#   RUSTCODE_RELEASE_BASE        override download root
#   RUSTCODE_RELEASE_LATEST_API  override latest-version API endpoint
#   RUSTCODE_VERSION             pin a specific release tag (default: latest)
#   RUSTCODE_PREFIX              install dir (default: /usr/local/bin if writable,
#                                  else ~/.local/bin)
#   RUSTCODE_DOWNLOAD_CONCURRENCY  parallel slots for the download race
#                                  (default: 4; set 1 for sequential fallback)
#   RUSTCODE_DOWNLOAD_TIMEOUT      per-attempt download timeout in seconds
#                                  (default: 300)
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

# Release source: defaults to the GitCode repository so `curl | sh` works
# zero-config. Override via env for alternative distribution channels.
RELEASE_BASE="${RUSTCODE_RELEASE_BASE:-https://gitcode.com/SecLab/RustCode/releases/download}"
RELEASE_LATEST_API="${RUSTCODE_RELEASE_LATEST_API:-https://api.gitcode.com/api/v5/repos/SecLab/RustCode/releases/latest}"

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

# Release base is set (default or env). Strip trailing slash.
RELEASE_BASE="${RELEASE_BASE%/}"

mkdir -p "$PREFIX"

# --- download ---
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
DEST="$TMP/rustcode${ext}"

# Pick download tool: $_fetch streams a URL to stdout (for the API lookup),
# $_down saves a URL to a file (for the binary). Binary downloads race
# concurrently, so $_down is quiet (parallel progress bars would garble the
# terminal) and carries a hard per-attempt timeout: one stalled source must
# not pin its slot for the whole race.
DOWN_TIMEOUT="${RUSTCODE_DOWNLOAD_TIMEOUT:-300}"
case "$DOWN_TIMEOUT" in ''|*[!0-9]*) DOWN_TIMEOUT=300 ;; esac
if command -v curl >/dev/null 2>&1; then
    _fetch="curl -sL --connect-timeout 5 --max-time 10"
    # --speed-limit/--speed-time: abort when stalled (<1KB/s for 30s) instead of
    # burning the full --max-time on a hung source occupying a race slot.
    _down="curl -fsSL --connect-timeout 5 --speed-limit 1024 --speed-time 30 --max-time $DOWN_TIMEOUT -o"
elif command -v wget >/dev/null 2>&1; then
    _fetch="wget -qO- --timeout=10 --tries=1"
    _down="wget -q --timeout=$DOWN_TIMEOUT --read-timeout=30 --tries=1 -O"
else
    echo "Error: need curl or wget." >&2
    exit 1
fi

# --- resolve version + candidate list ---
# Layer 1 (online Release) is always tried first. Layer 2 is the repo-committed
# release/ directory (raw file URL) — it does NOT depend on the CI/CD pipeline,
# so a pipeline failure never blocks a download (see release-redundancy plan).
#
# If no explicit version is pinned, candidates are: the API "latest", then every
# version listed in release/index.json (already newest-first), de-duplicated.
# Candidate URLs are then raced in bounded concurrent waves (priority order
# preserved: the highest-priority URL yielding a real binary wins).
# GitCode's raw-file endpoint is <base>/<path>?ref=<ref> (ref as a QUERY param).
# The path-segment form (/raw/<ref>/<path>) returns the SPA HTML shell, not the file.
RELEASE_RAW_BASE="${RUSTCODE_RELEASE_RAW_BASE:-https://gitcode.com/api/v5/repos/SecLab/RustCode/raw}"
RELEASE_RAW_REF="${RUSTCODE_RELEASE_RAW_REF:-dev}"

LATEST_VER=""
PINNED=0
if [ -n "${RUSTCODE_VERSION:-}" ]; then
    LATEST_VER="$RUSTCODE_VERSION"
    PINNED=1
elif [ -n "$RELEASE_LATEST_API" ]; then
    echo "==> Detecting latest version"
    LATEST_VER=$($_fetch "$RELEASE_LATEST_API" | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
fi

# Build candidate version list.
CANDIDATES=""
if [ "$PINNED" = "1" ]; then
    CANDIDATES="$LATEST_VER"
else
    # Fetch the repo index.json (sorted newest-first by release-publish.sh).
    IDX_TMP="$TMP/index.json"
    if $_fetch "$RELEASE_RAW_BASE/release/index.json?ref=$RELEASE_RAW_REF" > "$IDX_TMP" 2>/dev/null \
        && [ -s "$IDX_TMP" ] && grep -q '"version"' "$IDX_TMP"; then
        for v in $(sed -n 's/.*"version"[[:space:]]*:[[:space:]]*"\(v[0-9.]*\)".*/\1/p' "$IDX_TMP"); do
            case " $CANDIDATES " in
                *" $v "*) ;;
                *) CANDIDATES="${CANDIDATES:+$CANDIDATES }$v" ;;
            esac
        done
    fi
    # Ensure the API latest is tried first (in case it is newer than the index).
    if [ -n "$LATEST_VER" ]; then
        case " $CANDIDATES " in
            *" $LATEST_VER "*) ;;
            *) CANDIDATES="$LATEST_VER${CANDIDATES:+ $CANDIDATES}" ;;
        esac
    fi
fi

if [ -z "$CANDIDATES" ]; then
    echo "Error: no version resolved." >&2
    echo "       Set RUSTCODE_VERSION explicitly (e.g. RUSTCODE_VERSION=vX.Y.Z)," >&2
    echo "       or ensure RUSTCODE_RELEASE_LATEST_API / the repo release/index.json is reachable." >&2
    exit 1
fi

echo "==> Candidate versions: $CANDIDATES"

# --- download with multi-source / multi-version fallback (concurrent race) ---
# Candidate URLs keep the documented priority order (online Release first,
# then the repo-committed release/ raw URL; newest version first), but they
# are raced in bounded waves instead of one-by-one: within a wave every
# source downloads in parallel and the highest-priority URL that yields a
# usable binary wins, so the fallback ORDER is unchanged -- only wall-clock
# latency improves, and a single stalled source can no longer block the
# rest (each attempt carries a hard timeout).
# RUSTCODE_DOWNLOAD_CONCURRENCY=1 restores strictly sequential attempts.
ATTEMPTED=""
DOWNLOADED=0

MAXPAR="${RUSTCODE_DOWNLOAD_CONCURRENCY:-4}"
case "$MAXPAR" in ''|*[!0-9]*|0) MAXPAR=4 ;; esac

# Flatten the (version x source) matrix into one URL per line, priority
# order top-to-bottom.
URLFILE="$TMP/urls.txt"
: > "$URLFILE"
for VER in $CANDIDATES; do
    BIN="rustcode-${VER}-${os}-${arch}${ext}"
    printf '%s\n' "${RELEASE_BASE%/}/${VER}/${BIN}" >> "$URLFILE"
    printf '%s\n' "${RELEASE_RAW_BASE%/}/release/${VER}/${BIN}?ref=${RELEASE_RAW_REF}" >> "$URLFILE"
done
NUML=$(grep -c . "$URLFILE" || :)
echo "==> Racing $NUML candidate URLs (up to $MAXPAR downloads in parallel)"

WAVE_START=1
while [ "$DOWNLOADED" != 1 ] && [ "$WAVE_START" -le "$NUML" ]; do
    WAVE_END=$((WAVE_START + MAXPAR - 1))
    PIDSFILE="$TMP/pids.txt"
    : > "$PIDSFILE"

    # Launch one wave: slot i downloads to $TMP/dl.i.
    i="$WAVE_START"
    while [ "$i" -le "$NUML" ] && [ "$i" -le "$WAVE_END" ]; do
        U=$(sed -n "${i}p" "$URLFILE")
        BIN=$(basename "${U%%\?*}")
        ATTEMPTED="${ATTEMPTED:+$ATTEMPTED; }$U"
        echo "==> Trying $U"
        rm -f "$TMP/dl.$i"
        $_down "$TMP/dl.$i" "$U" 2>/dev/null &
        printf '%s %s %s\n' "$i" "$!" "$BIN" >> "$PIDSFILE"
        i=$((i + 1))
    done

    # Wait in priority order: the first usable slot wins immediately and
    # the rest of the wave is stopped below.
    while IFS=" " read -r SLOT PID BIN; do
        if wait "$PID" && [ -s "$TMP/dl.$SLOT" ] \
            && ! head -c 4 "$TMP/dl.$SLOT" | grep -q "<" 2>/dev/null; then
            mv "$TMP/dl.$SLOT" "$DEST"
            echo "    -> got $BIN ($(stat -c%s "$DEST" 2>/dev/null || stat -f%z "$DEST") bytes)"
            DOWNLOADED=1
            break
        fi
        rm -f "$TMP/dl.$SLOT"
    done < "$PIDSFILE"

    if [ "$DOWNLOADED" = "1" ]; then
        # Stop the remaining (possibly still-running) wave jobs.
        while IFS=" " read -r SLOT PID BIN; do
            kill "$PID" 2>/dev/null || :
        done < "$PIDSFILE"
    fi
    wait 2>/dev/null || :
    WAVE_START=$((WAVE_END + 1))
done

if [ "$DOWNLOADED" != "1" ]; then
    echo "Error: could not download a usable rustcode binary." >&2
    echo "       OS=$os  ARCH=$arch" >&2
    echo "       Tried versions: $CANDIDATES" >&2
    echo "       Tried sources:" >&2
    echo "       $ATTEMPTED" >&2
    echo "       A pipeline failure should not block download; the repo release/" >&2
    echo "       directory may simply be empty. Populate it by running a release" >&2
    echo "       script (e.g. scripts/release.sh) on a dev host and committing it." >&2
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
