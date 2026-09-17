#!/bin/bash
# Multi-arch cross-build script for RustCode releases.
#
# Builds static / cross-compiled binaries for multiple targets on a single
# Linux x86_64 runner. Each target is stripped and copied to dist/<version>/
# with a uniform name, then sha256sums.txt is generated.
#
# Targets (Linux runner, no macOS SDK needed):
#   linux-x64       x86_64-unknown-linux-musl    (musl-gcc, static)
#   linux-arm64     aarch64-unknown-linux-musl   (aarch64-linux-musl-gcc, static)
#   windows-x64     x86_64-pc-windows-gnu        (x86_64-w64-mingw32-gcc, cross)
#
# macOS targets (aarch64/x86_64-apple-darwin) require a native macOS runner
# (or osxcross + proprietary SDK) and are intentionally NOT built here —
# they are produced by .github/workflows/build.yml on macos-latest.
#
# Usage:
#   scripts/cross-build.sh                      # build all default targets
#   scripts/cross-build.sh linux-x64            # build one target
#   RUSTCODE_BUILD_TARGETS="linux-x64 linux-arm64" scripts/cross-build.sh
set -euo pipefail

cd "$(dirname "$0")/.."

# ── Version ────────────────────────────────────────────────────────────────
VERSION=$(awk -F'"' '
    /^\[workspace\.package\]/ { in_section = 1; next }
    /^\[/ { in_section = 0 }
    in_section && /^version *=/ { print "v"$2; exit }
' Cargo.toml)
if [ -z "$VERSION" ]; then
    echo "[ERROR] Could not determine version from Cargo.toml." >&2
    exit 1
fi

# ── Target registry ────────────────────────────────────────────────────────
# Each entry: SUFFIX|TRIPLE|LINKER|STRIP|INSTALL
# LINKER is the env var name (CARGO_TARGET_*_LINKER) + value, or empty for native.
# INSTALL is the apt/curl commands to provision the cross toolchain.
ALL_TARGETS=(
    "linux-x64|x86_64-unknown-linux-musl|CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc|musl-gcc|sudo apt-get update && sudo apt-get install -y musl-tools"
    "linux-arm64|aarch64-unknown-linux-musl|CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-musl-gcc|aarch64-linux-musl-strip|curl -fsSL https://musl.cc/aarch64-linux-musl-cross.tgz | sudo tar xz -C /opt && export PATH=/opt/aarch64-linux-musl-cross/bin:$PATH"
    "windows-x64|x86_64-pc-windows-gnu|CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc|x86_64-w64-mingw32-strip|sudo apt-get update && sudo apt-get install -y mingw-w64"
)

# ── Select targets ─────────────────────────────────────────────────────────
if [ $# -gt 0 ]; then
    REQUESTED="$*"
elif [ -n "${RUSTCODE_BUILD_TARGETS:-}" ]; then
    REQUESTED="$RUSTCODE_BUILD_TARGETS"
else
    REQUESTED="linux-x64 linux-arm64 windows-x64"
fi

echo "=== RustCode Cross-Build ${VERSION} ==="
echo "Requested targets: ${REQUESTED}"
echo ""

# ── WebUI check ────────────────────────────────────────────────────────────
# webui/dist is gitignored; a release without it serves 404 for /webui.
# In CI the frontend is built in a prior step; locally run build-webui.sh.
if [ -d webui ] && [ ! -f webui/dist/index.html ]; then
    if [ -n "${RUSTCODE_SKIP_WEBUI_CHECK:-}" ]; then
        echo "[WARN] webui/dist/index.html missing — binary will serve 404 for /webui." >&2
    else
        echo "[ERROR] webui/dist/index.html missing. Run scripts/build-webui.sh first," >&2
        echo "        or set RUSTCODE_SKIP_WEBUI_CHECK=1 to override (not recommended)." >&2
        exit 1
    fi
fi

# ── Output dir ─────────────────────────────────────────────────────────────
DIST="dist/${VERSION}"
mkdir -p "$DIST"

# ── Build loop ─────────────────────────────────────────────────────────────
BUILT=()
for REQUEST in $REQUESTED; do
    ENTRY=""
    for T in "${ALL_TARGETS[@]}"; do
        SUFFIX="${T%%|*}"
        if [ "$SUFFIX" = "$REQUEST" ]; then
            ENTRY="$T"
            break
        fi
    done
    if [ -z "$ENTRY" ]; then
        echo "[ERROR] Unknown target: ${REQUEST}" >&2
        echo "  Available: linux-x64 linux-arm64 windows-x64" >&2
        exit 1
    fi

    IFS='|' read -r SUFFIX TRIPLE LINKER_ENV STRIP_TOOL INSTALL_CMD <<< "$ENTRY"

    echo "[*] Building ${SUFFIX} (${TRIPLE})..."

    # Provision cross toolchain (skip if already present)
    if ! command -v "${STRIP_TOOL}" >/dev/null 2>&1; then
        echo "    Installing cross toolchain..."
        eval "$INSTALL_CMD"
    fi

    # Re-export PATH for musl.cc toolchains (needed after tar extraction)
    if [ "$SUFFIX" = "linux-arm64" ]; then
        export PATH="/opt/aarch64-linux-musl-cross/bin:${PATH}"
    fi

    # Add Rust target
    rustup target add "$TRIPLE" 2>/dev/null || true

    # Determine output file name
    if [[ "$TRIPLE" == *windows* ]]; then
        OUT_NAME="rustcode-${VERSION}-${SUFFIX}.exe"
        SRC_BIN="target/${TRIPLE}/release/rustcode.exe"
    else
        OUT_NAME="rustcode-${VERSION}-${SUFFIX}"
        SRC_BIN="target/${TRIPLE}/release/rustcode"
    fi

    # Build
    echo "    cargo build --release --target ${TRIPLE} -p rustcode"
    if [ -n "$LINKER_ENV" ]; then
        env "$LINKER_ENV" cargo build --release --target "$TRIPLE" -p rustcode
    else
        cargo build --release --target "$TRIPLE" -p rustcode
    fi

    if [ ! -f "$SRC_BIN" ]; then
        echo "[ERROR] Build succeeded but ${SRC_BIN} not found." >&2
        exit 1
    fi

    # Copy + strip
    cp "$SRC_BIN" "${DIST}/${OUT_NAME}"
    chmod +x "${DIST}/${OUT_NAME}" 2>/dev/null || true
    if command -v "${STRIP_TOOL}" >/dev/null 2>&1; then
        "${STRIP_TOOL}" "${DIST}/${OUT_NAME}" 2>/dev/null || echo "    [WARN] strip failed (non-fatal)"
    fi

    echo "    -> ${DIST}/${OUT_NAME}"
    BUILT+=("${OUT_NAME}")
done

# ── Checksums ──────────────────────────────────────────────────────────────
echo ""
echo "=== Generating sha256sums.txt ==="
cd "$DIST"
rm -f sha256sums.txt
for f in "${BUILT[@]}"; do
    sha256sum "$f" >> sha256sums.txt
done
cat sha256sums.txt

echo ""
echo "=== Done. ${#BUILT[@]} artifact(s) in ${DIST}/ ==="
ls -lh
