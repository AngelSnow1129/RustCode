#!/bin/bash
# Provision the pinned aarch64 musl cross toolchain used by Linux ARM64 builds.
#
# Why a cross toolchain at all: cc-rs builds the C dependencies (tree-sitter,
# zstd) with a C compiler, and the distro glibc cross gcc compiles them against
# glibc headers -- the final link then dies on glibc fortify symbols
# (undefined __memcpy_chk / __fprintf_chk) because musl libc has no such
# functions. x86_64 musl only needs the distro `musl-tools` package; aarch64
# musl has no such package.
#
# Why not musl.cc (the original source): single host, connection timeouts from
# GitHub runners, and a rolling URL with no integrity guarantee piped straight
# into `tar`, so a truncated body surfaced as "tar: Child returned status 1"
# instead of as a download error. This script pins the cross-tools/musl-cross
# GitHub Release by tag + sha256 instead -- immutable, served by GitHub itself.
#
# This script is the single source of truth for that pin. Both
# .github/workflows/build.yml (Linux job) and scripts/cross-build.sh call it.
# They used to carry separate copies, which is exactly how the workflow got the
# pinned source while cross-build.sh kept the dead musl.cc URL.
#
# Usage:
#   scripts/install-musl-cross.sh               # install (idempotent), print export lines
#   eval "$(scripts/install-musl-cross.sh)"     # ... and apply them to this shell
#   scripts/install-musl-cross.sh --github-env  # write to $GITHUB_ENV / $GITHUB_PATH
#
# Options:
#   --dest DIR      extraction root (default: /opt, or $MUSL_CROSS_DEST)
#   --archive FILE  reuse an existing archive instead of downloading it
#   --github-env    write the environment for later CI steps instead of printing
#
# Only the eval-able env belongs on stdout; all logs go to stderr so that
# `eval "$(...)"` never tries to execute a status line.
set -euo pipefail

usage() {
    cat <<'EOF' >&2
Usage: install-musl-cross.sh [--dest DIR] [--archive FILE] [--github-env]

  --dest DIR      extraction root (default: /opt, or $MUSL_CROSS_DEST)
  --archive FILE  reuse an existing archive instead of downloading it
  --github-env    write the environment for later CI steps instead of printing
EOF
}

# Pin: bump TAG only together with SHA256 -- they are a pair. The asset is
# aarch64-unknown-linux-musl.tar.xz and unpacks to <dest>/aarch64-unknown-linux-musl.
MUSL_CROSS_TAG=20260823
MUSL_CROSS_SHA256=0fc483607d9ed83bdf75e7539bacc66721d7e37ca606377aed6a90cef82e45da
TRIPLE=aarch64-unknown-linux-musl
ARCHIVE_NAME="${TRIPLE}.tar.xz"
RELEASE_BASE="https://github.com/cross-tools/musl-cross/releases/download/${MUSL_CROSS_TAG}"

DEST_ROOT="${MUSL_CROSS_DEST:-/opt}"
ARCHIVE_IN=""
GITHUB_ENV_MODE=0

while [ $# -gt 0 ]; do
    case "$1" in
        --dest)
            [ $# -ge 2 ] || { echo "[ERROR] --dest requires a directory" >&2; exit 2; }
            DEST_ROOT="$2"
            shift 2
            ;;
        --archive)
            [ $# -ge 2 ] || { echo "[ERROR] --archive requires a file" >&2; exit 2; }
            ARCHIVE_IN="$2"
            shift 2
            ;;
        --github-env)
            GITHUB_ENV_MODE=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "[ERROR] unknown argument: $1" >&2
            usage
            exit 2
            ;;
    esac
done

if [ "$GITHUB_ENV_MODE" = 1 ]; then
    if [ -z "${GITHUB_ENV:-}" ] || [ -z "${GITHUB_PATH:-}" ]; then
        echo "[ERROR] --github-env requires \$GITHUB_ENV and \$GITHUB_PATH (run inside GitHub Actions)" >&2
        exit 2
    fi
fi

TOOLCHAIN_ROOT="${DEST_ROOT}/${TRIPLE}"
BIN_DIR="${TOOLCHAIN_ROOT}/bin"
GCC="${BIN_DIR}/${TRIPLE}-gcc"
AR="${BIN_DIR}/${TRIPLE}-ar"
STRIP="${BIN_DIR}/${TRIPLE}-strip"

# Single source of truth for the downstream paths: the Build and Package steps
# consume these, so no caller has to reconstruct them.
emit_env() {
    if [ "$GITHUB_ENV_MODE" = 1 ]; then
        echo "$BIN_DIR" >> "$GITHUB_PATH"
        {
            echo "MUSL_AARCH64_STRIP=${STRIP}"
            echo "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=${GCC}"
            # cc-rs (tree-sitter, zstd build scripts) must pick this compiler
            # too; without it cc-rs falls back to the glibc cross gcc and the
            # final link dies on the fortify symbols described in the header.
            echo "CC_aarch64_unknown_linux_musl=${GCC}"
            echo "AR_aarch64_unknown_linux_musl=${AR}"
        } >> "$GITHUB_ENV"
    else
        printf 'export PATH=%s:$PATH\n' "$BIN_DIR"
        printf 'export MUSL_AARCH64_STRIP=%s\n' "$STRIP"
        printf 'export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=%s\n' "$GCC"
        printf 'export CC_aarch64_unknown_linux_musl=%s\n' "$GCC"
        printf 'export AR_aarch64_unknown_linux_musl=%s\n' "$AR"
    fi
}

# Idempotent: an installed toolchain that answers --version is reused as is, so
# re-runs, retries and local loops never re-download 83 MiB. Presence alone is
# not trusted -- a half-extracted tree must not pass this gate.
if [ -x "$GCC" ] && "$GCC" --version >/dev/null 2>&1; then
    echo "[CHECK] ${TRIPLE} toolchain already installed at ${TOOLCHAIN_ROOT}" >&2
    emit_env
    exit 0
fi

if ! command -v xz >/dev/null 2>&1; then
    echo "[ERROR] xz is required to unpack ${ARCHIVE_NAME}." >&2
    echo "        Ubuntu/Debian: sudo apt-get install -y xz-utils" >&2
    exit 1
fi

# sudo only when the destination actually needs it: Actions runners are not
# root, containers and dev shells usually are.
if [ "$(id -u)" -eq 0 ] || [ -w "$DEST_ROOT" ] || [ -w "$(dirname "$DEST_ROOT")" ]; then
    SUDO=""
else
    SUDO="sudo"
fi

DOWNLOAD_DIR=""
ARCHIVE=""
if [ -n "$ARCHIVE_IN" ]; then
    if [ ! -f "$ARCHIVE_IN" ]; then
        echo "[ERROR] --archive ${ARCHIVE_IN} not found" >&2
        exit 1
    fi
    ARCHIVE="$ARCHIVE_IN"
    echo "[INFO] Using local archive ${ARCHIVE}" >&2
else
    DOWNLOAD_DIR="$(mktemp -d)"
    ARCHIVE="${DOWNLOAD_DIR}/${ARCHIVE_NAME}"
    echo "[INFO] Downloading ${ARCHIVE_NAME} (cross-tools/musl-cross ${MUSL_CROSS_TAG})" >&2
    # --output writes a real file (never a pipe) and --retry-all-errors also
    # retries connection resets/timeouts, so a truncated transfer can never
    # reach tar -- the failure mode that killed the musl.cc pipeline.
    curl --fail --location --silent --show-error \
        --connect-timeout 15 --max-time 600 \
        --retry 5 --retry-delay 10 --retry-all-errors \
        --output "$ARCHIVE" "${RELEASE_BASE}/${ARCHIVE_NAME}"
fi

# Integrity gate: a short, truncated or HTML error body dies here. The check
# report goes to stderr so that stdout stays exclusively eval-able.
echo "${MUSL_CROSS_SHA256}  ${ARCHIVE}" | sha256sum --check --strict - >&2

$SUDO mkdir -p "$DEST_ROOT"
$SUDO tar -xJf "$ARCHIVE" -C "$DEST_ROOT"
if [ -n "$DOWNLOAD_DIR" ]; then
    rm -rf "$DOWNLOAD_DIR"
fi

# Smoke test: name the toolchain in the failure if the archive was unusable.
"$GCC" --version >&2

echo "[CHECK] installed ${TRIPLE} toolchain at ${TOOLCHAIN_ROOT}" >&2
emit_env
