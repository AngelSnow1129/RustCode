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
# Declared empty on purpose: the `case` arms below assign them, and the
# fail-closed checks right after make a NEW arm that forgets to assign surface
# as a clear message instead of an "os: parameter not set" crash under `set -u`.
os=""
arch=""

case "$uname_s" in
    Darwin) os="darwin" ;;
    Linux)  os="linux"  ;;
    FreeBSD) os="freebsd" ;;
    HarmonyOS) os="ohos" ;;
    # MSYS2 / MinGW / Git-Bash / Cygwin: a Unix shell running ON Windows. `uname -s` looks
    # like MSYS_NT-10.0-26100 / MINGW64_NT-... / CYGWIN_NT-.... Install the native Windows
    # `.exe` into this shell's environment (windows-specific PREFIX + suffix handled below).
    MSYS*|MINGW*|CYGWIN*) os="windows"; ext=".exe" ;;
    *) echo "Unsupported OS: $uname_s (Windows users: download the zip from the release page)"; exit 1 ;;
esac

# Belt-and-braces: reached only if an arm above was added without assigning `os`.
if [ -z "$os" ]; then
    echo "Error: OS detection failed (uname -s = '$uname_s'); add it to the case above." >&2
    exit 1
fi

case "$uname_m" in
    arm64|aarch64) arch="arm64" ;;
    x86_64|amd64)  arch="x64"   ;;
    *) echo "Unsupported arch: $uname_m"; exit 1 ;;
esac

# Belt-and-braces: reached only if an arm above was added without assigning `arch`.
if [ -z "$arch" ]; then
    echo "Error: arch detection failed (uname -m = '$uname_m'); add it to the case above." >&2
    exit 1
fi

# --- pick install dir ---
# `$HOME` is NOT exported everywhere this script runs (`env -i`, cron, systemd units,
# `docker exec sh -c 'curl…|sh'`, some CI runners). Under `set -u` a bare `$HOME`
# then aborts with "HOME: parameter not set" -- an opaque crash, same failure class
# as the DOWNLOADED bug fixed in 6c0b0a91. Fail closed with an actionable message
# instead: guess a fallback dir (e.g. `/tmp`) would install into the WRONG prefix,
# which is worse than refusing. Only call this where HOME is truly required -- the
# root /usr/local/bin branch and an explicit RUSTCODE_PREFIX need no HOME.
# `${HOME:-}` is nounset-safe (default-value expansion), so the probe itself never
# triggers `set -u`.
# $1 = the path that could not be resolved (single-quoted at the call sites so
#      the literal `$HOME/...` is shown), $2 = optional extra context line.
require_home() {
    [ -n "${HOME:-}" ] && return 0
    echo "Error: HOME is not set in this environment; cannot resolve $1." >&2
    echo "       Export HOME (e.g. export HOME=/home/you) and re-run, or install" >&2
    echo "       as root (uses /usr/local/bin, needs no HOME), or pass an explicit" >&2
    echo "       install dir: RUSTCODE_PREFIX=/opt/bin sh install.sh" >&2
    [ -z "${2:-}" ] || echo "       $2" >&2
    exit 1
}

if [ -n "${RUSTCODE_PREFIX:-}" ]; then
    PREFIX="$RUSTCODE_PREFIX"
elif [ "$os" = "ohos" ] || [ "$os" = "windows" ]; then
    # Windows shells (MSYS/Git-Bash/Cygwin) have no sudo and a system /usr/local/bin under
    # the MSYS root; install into the user's home instead (always writable, no elevation).
    require_home '$HOME/.local/bin'
    PREFIX="$HOME/.local/bin"
elif [ -w /usr/local/bin ] 2>/dev/null; then
    PREFIX="/usr/local/bin"
elif [ "$(id -u)" -eq 0 ]; then
    PREFIX="/usr/local/bin"
else
    require_home '$HOME/.local/bin'
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
# Declared empty on purpose: a new download-tool branch that forgets to set one
# of them must fail closed here, not with "_down: parameter not set" under `set -u`.
_fetch=""
_down=""
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

# Belt-and-braces: reached only if a branch above was added without assigning
# both helpers. Downloading through an empty command must never happen silently.
if [ -z "$_fetch" ] || [ -z "$_down" ]; then
    echo "Error: download tool not configured (curl/wget branch did not set it)." >&2
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

# Order "vX.Y.Z" lines newest-first. Deliberately NOT `sort -V`: that is a GNU
# extension and this script runs on BSD/macOS `sort` too. Emit zero-padded
# numeric fields and reverse-sort lexicographically, which is well-defined for
# fixed-width numeric keys.
sort_versions_desc() {
    awk -F. '{
        v = $1; gsub(/^v/, "", v)
        printf "%09d %09d %09d\t%s\n", v + 0, $2 + 0, $3 + 0, $0
    }' | sort -r | cut -f2-
}

# Build candidate version list.
#
# BOTH sources are candidates, not authorities. The online `releases/latest`
# API can lag the repository (observed: it answered v6.1.0 while the repo
# already carried v6.2.0, because the v6.2.0 online Release did not exist yet),
# and `release/index.json` can only list versions that actually have committed
# artifacts. So collect both and order strictly by semantic version -- the
# newest version wins regardless of which source reported it.
CANDIDATES=""
if [ "$PINNED" = "1" ]; then
    CANDIDATES="$LATEST_VER"
else
    # Fetch the repo index.json (sorted newest-first by release-publish.sh).
    IDX_TMP="$TMP/index.json"
    IDX_VERS=""
    if $_fetch "$RELEASE_RAW_BASE/release/index.json?ref=$RELEASE_RAW_REF" > "$IDX_TMP" 2>/dev/null \
        && [ -s "$IDX_TMP" ] && grep -q '"version"' "$IDX_TMP"; then
        for v in $(sed -n 's/.*"version"[[:space:]]*:[[:space:]]*"\(v[0-9.]*\)".*/\1/p' "$IDX_TMP"); do
            case " $IDX_VERS " in
                *" $v "*) ;;
                *) IDX_VERS="${IDX_VERS:+$IDX_VERS }$v" ;;
            esac
        done
    fi

    # Merge the API's idea of latest into the index list (deduped).
    CANDIDATES="$IDX_VERS"
    if [ -n "$LATEST_VER" ]; then
        case " $CANDIDATES " in
            *" $LATEST_VER "*) ;;
            *) CANDIDATES="$LATEST_VER${CANDIDATES:+ $CANDIDATES}" ;;
        esac
    fi
    CANDIDATES=$(printf '%s\n' $CANDIDATES | sort_versions_desc | tr '\n' ' ' | sed 's/ *$//')

    # Diagnostics: a stale API answer used to silently win. Say so instead.
    if [ -n "$LATEST_VER" ] && [ -n "$IDX_VERS" ]; then
        IDX_TOP=$(printf '%s\n' $IDX_VERS | sort_versions_desc | head -1)
        if [ "$IDX_TOP" != "$LATEST_VER" ]; then
            echo "==> Note: latest-version API reports $LATEST_VER, repo index has $IDX_TOP; installing $IDX_TOP" >&2
        fi
    fi
fi

if [ -z "$CANDIDATES" ]; then
    echo "Error: no version resolved." >&2
    echo "       Set RUSTCODE_VERSION explicitly (e.g. RUSTCODE_VERSION=vX.Y.Z)," >&2
    echo "       or ensure RUSTCODE_RELEASE_LATEST_API / the repo release/index.json is reachable." >&2
    exit 1
fi

echo "==> Candidate versions: $CANDIDATES"

# --- FreeBSD: no prebuilt binary exists; build from source instead ---
# The release matrix publishes only linux-{x64,arm64} and windows-x64, so a
# FreeBSD download would always 404. When the Rust toolchain is present we build
# the CLI from the resolved tag; otherwise we print the exact steps and exit.
# rust-embed uses allow_missing, so a CLI-only build is fully usable (the TUI
# works); the WebUI is embedded only if Node>=22 is present (built before
# cargo build). With no Node the binary is CLI-only and `rustcode webui` prints
# a self-explanatory "not built" message.
if [ "$os" = "freebsd" ]; then
    build_freebsd_from_source() {
        VER="${LATEST_VER:-$(echo "$CANDIDATES" | awk '{print $1}')}"
        echo "==> FreeBSD has no prebuilt binary; building '$VER' from source"
        if ! command -v git >/dev/null 2>&1; then
            echo "Error: git not found. Install it: pkg install -y git" >&2
            exit 1
        fi
        if ! command -v cargo >/dev/null 2>&1; then
            echo "Error: cargo not found. Install the Rust toolchain, then re-run:" >&2
            echo "       pkg install -y rust     # or:  curl https://sh.rustup.rs -sSf | sh" >&2
            exit 1
        fi
        SRC="$TMP/src"
        echo "==> Cloning https://gitcode.com/SecLab/RustCode (tag $VER)"
        if ! git clone --depth 1 --branch "$VER" "https://gitcode.com/SecLab/RustCode" "$SRC" 2>/dev/null \
            && ! git clone --depth 1 "https://gitcode.com/SecLab/RustCode" "$SRC" 2>/dev/null; then
            echo "Error: git clone failed (need network access to gitcode.com)." >&2
            exit 1
        fi
        # Build the WebUI first (if Node>=22 is available) so rust-embed captures
        # webui/dist into the binary; otherwise CLI is built without it.
        if command -v node >/dev/null 2>&1; then
            NODE_MAJOR=$(node --version 2>/dev/null | tr -d 'v' | cut -d. -f1)
            if [ "${NODE_MAJOR:-0}" -ge 22 ]; then
                echo "==> Building WebUI frontend (node $NODE_MAJOR present)"
                if ( cd "$SRC/webui" && npm ci && npm run build ) 2>&1; then
                    echo "    -> WebUI built; it will be embedded into the binary"
                else
                    echo "Warning: WebUI build failed; binary will be CLI-only." >&2
                fi
            else
                echo "==> Node $NODE_MAJOR < 22; skipping WebUI build (CLI-only)." >&2
            fi
        else
            echo "==> Node not found; building CLI only (WebUI not embedded)." >&2
        fi
        echo "==> cargo build --release -p rustcode"
        if ! ( cd "$SRC" && cargo build --release -p rustcode ) 2>&1; then
            echo "Error: cargo build failed. Check the Rust toolchain and network, then re-run." >&2
            exit 1
        fi
        SRC_BIN="$SRC/target/release/rustcode"
        if [ ! -f "$SRC_BIN" ]; then
            echo "Error: built binary not found at $SRC_BIN" >&2
            exit 1
        fi
        cp "$SRC_BIN" "$DEST"
        echo "    -> installed $DEST (FreeBSD build from source)"
    }
    build_freebsd_from_source
    DOWNLOADED=1
fi

# --- download with multi-source / multi-version fallback (concurrent race) ---
# Candidate URLs keep the documented priority order (online Release first,
# then the repo-committed release/ raw URL; newest version first), but they
# are raced in bounded waves instead of one-by-one: within a wave every
# source downloads in parallel and the highest-priority URL that yields a
# usable binary wins, so the fallback ORDER is unchanged -- only wall-clock
# latency improves, and a single stalled source can no longer block the
# rest (each attempt carries a hard timeout).
# RUSTCODE_DOWNLOAD_CONCURRENCY=1 restores strictly sequential attempts.
# FreeBSD builds from source (handled above) and sets DOWNLOADED=1, so the
# download race below is skipped for it. Initialize for non-FreeBSD paths:
# under `set -u` an unset DOWNLOADED would abort before the first reference.
: "${DOWNLOADED:=0}"

# --- integrity: verify a downloaded binary against the repo-committed manifest ---
# `release/<ver>/manifest.json` records the expected sha256 + size for every target
# and IS committed to the repo, so it stays fetchable even when the binary itself is
# served from an online Release asset. Without this check the installer accepts any
# payload that is merely non-empty and not obviously HTML -- a truncated, corrupt or
# wrong-architecture file would be installed silently.
#
# Policy: verify whenever the expected values can be determined. If the manifest (or a
# sha256 tool) is unavailable we say so loudly rather than silently trusting the download
# -- project rule: no silent degradation. Set RUSTCODE_REQUIRE_MANIFEST=1 to make
# "cannot verify" fatal for installs that must not proceed unverified.
MF_REQUIRED="${RUSTCODE_REQUIRE_MANIFEST:-0}"

mf_path() {
    # $1 = version -> cached manifest path on stdout, or empty when unfetchable
    _v="$1"
    _mf="$TMP/manifest-$_v.json"
    if [ -s "$_mf" ]; then
        printf '%s\n' "$_mf"
        return 0
    fi
    # `${_fetch:-false}` keeps this nounset-safe: the downloader command is always
    # resolved earlier (curl/wget, else the script exits), but if it were ever unset
    # a bare `$_fetch` would reintroduce the exact failure class fixed in 6c0b0a91.
    if ${_fetch:-false} "$RELEASE_RAW_BASE/release/$_v/manifest.json?ref=$RELEASE_RAW_REF" > "$_mf" 2>/dev/null \
        && [ -s "$_mf" ] && grep -q '"binaries"' "$_mf"; then
        printf '%s\n' "$_mf"
        return 0
    fi
    rm -f "$_mf"
    return 0
}

mf_field() {
    # $1 = manifest, $2 = target (e.g. linux-x64), $3 = "sha256" | "size"
    _mf="$1"; _t="$2"; _k="$3"
    if command -v jq >/dev/null 2>&1; then
        _out=$(jq -r --arg t "$_t" --arg k "$_k" '.binaries[$t][$k] // empty' "$_mf" 2>/dev/null) || _out=""
        [ -n "$_out" ] && { printf '%s\n' "$_out"; return 0; }
    fi
    # POSIX fallback: the generator emits one compact target per line, e.g.
    #   "linux-x64": { "file": "...", "sha256": "<64 hex>", "size": 40333784 },
    _line=$(grep "\"$_t\"" "$_mf" 2>/dev/null | head -n 1)
    [ -n "$_line" ] || return 0
    case "$_k" in
        sha256) printf '%s\n' "$_line" | sed -n 's/.*"sha256"[[:space:]]*:[[:space:]]*"\([0-9a-fA-F]\{64\}\)".*/\1/p' ;;
        size)   printf '%s\n' "$_line" | sed -n 's/.*"size"[[:space:]]*:[[:space:]]*"\{0,1\}\([0-9]\{1,\}\)"\{0,1\}.*/\1/p' ;;
    esac
    return 0
}

sha256_of() {
    # $1 = file -> lowercase hex sha256 on stdout, empty when no hash tool exists
    _f="$1"
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$_f" 2>/dev/null | awk '{print $1}'
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$_f" 2>/dev/null | awk '{print $1}'
    elif command -v sha256 >/dev/null 2>&1; then
        sha256 -q "$_f" 2>/dev/null
    fi
    return 0
}

verify_against_manifest() {
    # $1 = freshly downloaded file, $2 = version, $3 = target (os-arch)
    # 0 = accept (verified, or unverifiable but allowed) | 1 = REJECT on mismatch
    _f="$1"; _v="$2"; _t="$3"
    _mf=$(mf_path "$_v")
    if [ -z "$_mf" ]; then
        if [ "$MF_REQUIRED" = "1" ]; then
            echo "    !! no release/$_v/manifest.json and RUSTCODE_REQUIRE_MANIFEST=1; refusing unverified download" >&2
            return 1
        fi
        echo "    !! WARNING: could not fetch release/$_v/manifest.json -- installing UNVERIFIED" >&2
        return 0
    fi
    _want_sha=$(mf_field "$_mf" "$_t" sha256)
    if [ -z "$_want_sha" ]; then
        if [ "$MF_REQUIRED" = "1" ]; then
            echo "    !! manifest has no sha256 for $_t on $_v; refusing unverified download" >&2
            return 1
        fi
        echo "    !! WARNING: manifest has no entry for $_t on $_v -- installing UNVERIFIED" >&2
        return 0
    fi
    _got_sha=$(sha256_of "$_f")
    if [ -z "$_got_sha" ]; then
        if [ "$MF_REQUIRED" = "1" ]; then
            echo "    !! no sha256 tool (need sha256sum/shasum/sha256); refusing unverified download" >&2
            return 1
        fi
        echo "    !! WARNING: no sha256 tool available -- installing UNVERIFIED" >&2
        return 0
    fi
    _want_size=$(mf_field "$_mf" "$_t" size)
    if [ -n "$_want_size" ]; then
        _got_size=$(wc -c < "$_f" | tr -d '[:space:]')
        if [ "$_got_size" != "$_want_size" ]; then
            echo "    !! REJECTED: $3 size $_got_size != manifest $_want_size" >&2
            return 1
        fi
    fi
    if [ "$_got_sha" != "$_want_sha" ]; then
        echo "    !! REJECTED: $3 sha256 mismatch (got $_got_sha, want $_want_sha)" >&2
        return 1
    fi
    echo "    -> verified against release/$_v/manifest.json (sha256 + size ok)"
    return 0
}

if [ "$DOWNLOADED" != "1" ]; then
ATTEMPTED=""

MAXPAR="${RUSTCODE_DOWNLOAD_CONCURRENCY:-4}"
case "$MAXPAR" in ''|*[!0-9]*|0) MAXPAR=4 ;; esac

# Flatten the (version x source) matrix into one URL per line, priority
# order top-to-bottom.
URLFILE="$TMP/urls.txt"
: > "$URLFILE"
for VER in $CANDIDATES; do
    BIN="rustcode-${VER}-${os}-${arch}${ext}"
    # Primary sources (online Release + repo-committed raw fallback).
    printf '%s\n' "${RELEASE_BASE%/}/${VER}/${BIN}" >> "$URLFILE"
    printf '%s\n' "${RELEASE_RAW_BASE%/}/release/${VER}/${BIN}?ref=${RELEASE_RAW_REF}" >> "$URLFILE"
    # Optional mirror acceleration: RUSTCODE_RELEASE_MIRRORS is an ordered,
    # space/comma-separated list of full download bases. Each is raced ahead of
    # the next so a fast mirror wins before a slow primary. Unset = no change.
    if [ -n "${RUSTCODE_RELEASE_MIRRORS:-}" ]; then
        # POSIX-only (this script also runs under dash via curl|sh): turn
        # commas into spaces and use unquoted word splitting to enumerate;
        # no arrays / here-strings (those are bashisms dash rejects).
        for M in $(printf '%s' "$RUSTCODE_RELEASE_MIRRORS" | tr ',' ' '); do
            [ -n "$M" ] && printf '%s\n' "${M%/}/${VER}/${BIN}" >> "$URLFILE"
        done
    fi
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
            # Recover the version this candidate belongs to: BIN is built as
            # "rustcode-<ver>-<os>-<arch><ext>" when URLFILE is flattened above.
            SLOT_VER=${BIN#rustcode-}
            SLOT_SUFFIX="-${os}-${arch}${ext}"
            SLOT_VER=${SLOT_VER%$SLOT_SUFFIX}
            if verify_against_manifest "$TMP/dl.$SLOT" "$SLOT_VER" "${os}-${arch}"; then
                mv "$TMP/dl.$SLOT" "$DEST"
                echo "    -> got $BIN ($(stat -c%s "$DEST" 2>/dev/null || stat -f%z "$DEST") bytes)"
                DOWNLOADED=1
                break
            fi
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
fi

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
# Smoke test the installed binary. This used to be `2>/dev/null || true`, which hid
# real failures: a wrong-architecture download or a missing shared library would be
# reported as a successful install. Surface it instead.
if ! VER_OUT=$("$TARGET" --version 2>&1); then
    echo "Error: $TARGET failed its --version smoke test." >&2
    echo "       Output was: $VER_OUT" >&2
    echo "       Usually this means the wrong architecture was installed, or a" >&2
    echo "       required shared library is missing. The file was still written;" >&2
    echo "       remove it and re-run with RUSTCODE_VERSION / the right target." >&2
    exit 1
fi
printf '%s\n' "$VER_OUT"

# --- write custom provider config (optional --url/--key/--model) ---
write_custom_provider() {
    [ -n "$PROVIDER_URL$PROVIDER_KEY$PROVIDER_MODEL" ] || return 0
    NAME="${PROVIDER_NAME:-custom}"
    # An explicit RUSTCODE_HOME needs no HOME at all; only the default path does.
    if [ -n "${RUSTCODE_HOME:-}" ]; then
        CFG_DIR="$RUSTCODE_HOME"
    else
        require_home '$HOME/.rustcode'
        CFG_DIR="$HOME/.rustcode"
    fi
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
            require_home '$HOME/.zshrc'
            RC="$HOME/.zshrc"
        elif [ -n "${BASH_VERSION:-}" ] || [ "$(basename "${SHELL:-}")" = "bash" ]; then
            require_home '$HOME/.bashrc'
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
                # `printf`, not `echo`: dash's builtin echo interprets backslash
                # escapes (XSI semantics, bash does not by default), so an install
                # path containing e.g. `\t` would be silently rewritten in the rc.
                printf '%s\n' "$LINE" >> "$RC"
                echo ""
                echo "Added $PREFIX to PATH in $RC"
            fi
            echo ""
            echo "To start using rustcode right now, run:"
            echo ""
            echo "    source $RC"
            echo ""
        else
            # No supported interactive rc detected (e.g. FreeBSD tcsh/sh, or a
            # non-interactive shell). Print a shell-appropriate manual line so
            # the user can add PATH themselves — the bash/zsh auto-add above
            # does not cover tcsh/csh, whose PATH syntax differs.
            SH_BN="$(basename "${SHELL:-sh}")"
            case "$SH_BN" in
                tcsh|csh) MANUAL="set path = ($PREFIX \$path)" ;;
                *)        MANUAL="export PATH=\"$PREFIX:\$PATH\"" ;;
            esac
            echo ""
            echo "Note: $PREFIX is not in your PATH. Add this line to your shell rc:"
            echo "    $MANUAL"
        fi
        ;;
esac
