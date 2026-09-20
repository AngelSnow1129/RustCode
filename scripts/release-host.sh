#!/usr/bin/env bash
# scripts/release-host.sh — build ONE version for the CURRENT host's OS/ARCH and
# publish it into the repo-backed release/ directory.
#
# Why this exists: scripts/release.sh cross-compiles the full matrix but is
# macOS-cross-centric (it builds apple-darwin first and `set -e` aborts a Linux
# host there). A dev host should be able to produce a complete, runnable build
# of its OWN environment with one command; this script does exactly that and
# then hands off to scripts/release-publish.sh.
#
# Env:
#   RUSTCODE_VERSION        override the version (default: [workspace.package].version)
#   RUSTCODE_INCLUDE_DAEMON 1 to also build + publish rustcode-daemon
#   RUSTCODE_BUILD_WEBUI    0 to skip the embedded-UI build (default 1)
#
# Usage: scripts/release-host.sh

set -euo pipefail
cd "$(dirname "$0")/.."
ROOT="$(pwd)"

# --- version (same source of truth as release.sh) ---
VERSION="${RUSTCODE_VERSION:-}"
if [ -z "$VERSION" ]; then
    CV="$(awk -F'"' '/^\[workspace\.package\]/{f=1;next} /^\[/{f=0} f&&/^version *=/{print $2; exit}' Cargo.toml)"
    [ -n "$CV" ] && VERSION="v${CV}"
fi
case "$VERSION" in
    v[0-9]*) ;;
    *) echo "Refusing non-vX.Y.Z version: '$VERSION' (set RUSTCODE_VERSION=vX.Y.Z)" >&2; exit 1 ;;
esac

# --- host OS/ARCH -> release target tag ---
OS_RAW="$(uname -s)"
ARCH_RAW="$(uname -m)"
case "$OS_RAW" in
    Darwin) OS=darwin ;;
    Linux)  OS=linux ;;
    MINGW*|MSYS*|CYGWIN*) OS=windows ;;
    *) echo "Unsupported host OS: $OS_RAW" >&2; exit 1 ;;
esac
case "$ARCH_RAW" in
    x86_64|amd64) ARCH=x64 ;;
    arm64|aarch64) ARCH=arm64 ;;
    *) echo "Unsupported host ARCH: $ARCH_RAW" >&2; exit 1 ;;
esac
EXT=""
[ "$OS" = "windows" ] && EXT=".exe"
TARGET_TAG="${OS}-${ARCH}"
echo "=== Host release ${VERSION} for ${TARGET_TAG} ==="

# --- embedded webui (rust-embed allow_missing would silently ship an empty UI) ---
if [ "${RUSTCODE_BUILD_WEBUI:-1}" = "1" ] && [ ! -f webui/dist/index.html ]; then
    echo "Building webui frontend..."
    "$ROOT/scripts/build-webui.sh"
fi

DIST="dist/${VERSION}"
mkdir -p "$DIST"

echo "=== cargo build --release -p rustcode ==="
cargo build --release -p rustcode
cp "target/release/rustcode${EXT}" "${DIST}/rustcode-${VERSION}-${TARGET_TAG}${EXT}"
echo "  -> ${DIST}/rustcode-${VERSION}-${TARGET_TAG}${EXT}"
[ -x "${DIST}/rustcode-${VERSION}-${TARGET_TAG}${EXT}" ] || [ -f "${DIST}/rustcode-${VERSION}-${TARGET_TAG}${EXT}" ]

if [ "${RUSTCODE_INCLUDE_DAEMON:-0}" = "1" ]; then
    echo "=== cargo build --release -p rustcode-daemon ==="
    cargo build --release -p rustcode-daemon
    cp "target/release/rustcode-daemon${EXT}" "${DIST}/rustcode-daemon-${VERSION}-${TARGET_TAG}${EXT}"
    echo "  -> ${DIST}/rustcode-daemon-${VERSION}-${TARGET_TAG}${EXT}"
fi

# --- publish into the repo-backed release/ (fallback source) ---
echo ""
echo "=== Publishing to release/ (repo fallback) ==="
"$ROOT/scripts/release-publish.sh" "$ROOT/$DIST" "$VERSION"
