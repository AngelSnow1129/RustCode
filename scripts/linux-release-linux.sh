#!/bin/bash
# Linux Native Build Script
# 在 Linux 服务器上编译 Linux 原生版本
set -e

# Always run from project root
cd "$(dirname "$0")/.."

# Derive version from Cargo.toml [workspace.package].version, matching the
# other release scripts (release.sh / release-daemon.sh / macos-release-*.sh).
# git tag is no longer the source of truth to avoid drift between tag and crate.
VERSION=$(awk -F'"' '
    /^\[workspace\.package\]/ { in_section = 1; next }
    /^\[/ { in_section = 0 }
    in_section && /^version *=/ { print "v"$2; exit }
' Cargo.toml)
if [ -z "$VERSION" ]; then
    echo "Could not determine version from Cargo.toml. Set [workspace.package].version."
    exit 1
fi

# Detect architecture
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64)
        TARGET="x86_64-unknown-linux-gnu"
        SUFFIX="linux-x64"
        ;;
    aarch64)
        TARGET="aarch64-unknown-linux-gnu"
        SUFFIX="linux-arm64"
        ;;
    *)
        echo "Unsupported architecture: ${ARCH}"
        exit 1
esac

DIST="dist/${VERSION}"
mkdir -p "$DIST"

echo "=== RustCode Linux Release ${VERSION} ==="
echo "Target: ${TARGET}"
echo "Architecture: ${ARCH}"
echo ""

# Build the embedded webui frontend so the binary embeds the latest UI.
#
# Fatal, not a warning: webui/dist is gitignored, so there is no committed
# copy to fall back on. Skipping this step produces a binary whose /webui
# serves 404 — a defect nobody sees until the release is installed.
if [ ! -d webui ]; then
  echo "error: webui/ is missing; cannot build the embedded UI" >&2
  exit 1
fi
if ! command -v npm >/dev/null 2>&1; then
  echo "error: npm not found, and webui/dist is gitignored — there is nothing" >&2
  echo "       to fall back on. A release built now would serve 404 for /webui." >&2
  exit 1
fi
echo "Building webui frontend..."
(cd webui && npm ci && npm run build)
# `rust-embed` accepts a missing/empty folder silently (allow_missing), so a
# half-failed frontend build would otherwise ship as an empty UI.
if [ ! -f webui/dist/index.html ]; then
  echo "error: webui build produced no dist/index.html" >&2
  exit 1
fi
echo ""

# Build
echo "[1/2] Building ${TARGET} (native)..."
cargo build --release --target "$TARGET"

# Copy binaries
echo "[2/2] Copying artifacts..."
cp "target/${TARGET}/release/rustcode" "${DIST}/rustcode-${VERSION}-${SUFFIX}"
cp "target/${TARGET}/release/rustcode-daemon" "${DIST}/rustcode-daemon-${VERSION}-${SUFFIX}"
echo "  -> ${DIST}/rustcode-${VERSION}-${SUFFIX}"
echo "  -> ${DIST}/rustcode-daemon-${VERSION}-${SUFFIX}"

# Package
echo ""
echo "=== Packaging ==="
cd "$DIST"
rm -f *${SUFFIX}*.tar.gz 2>/dev/null
for f in rustcode-*${SUFFIX} rustcode-daemon-*${SUFFIX}; do
    [ -f "$f" ] || continue
    chmod +x "$f"
    tar czf "${f}.tar.gz" "$f"
    echo "  -> ${f}.tar.gz"
done

# SHA256
echo ""
echo "=== SHA256 ==="
sha256sum *${SUFFIX}*.tar.gz | tee -a checksums.txt
echo ""
echo "Done. Release artifacts:"
ls -lh *${SUFFIX}*.tar.gz
