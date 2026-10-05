#!/bin/bash
# Provision the FreeBSD cross toolchain (sysroot + clang/lld) used by the
# FreeBSD release builds in scripts/release.sh and .github/workflows/build.yml.
#
# Why a cross toolchain at all: Rust's x86_64-unknown-freebsd / aarch64-unknown-freebsd
# targets are tier 2 -- `rustup target add` gives you the std for them, but the
# final link (and the C deps compiled by cc-rs: tree-sitter, zstd, ...) still
# needs a FreeBSD sysroot (headers + libc) and a linker that understands the
# FreeBSD ELF/ABI. We build that from the official FreeBSD base tarball:
#
#   * sysroot  = an unpacked FreeBSD `base.txz` (provides /usr/include, /usr/lib)
#   * compiler = host `clang` driven in cross mode via --target=... --sysroot=...
#   * linker   = host `ld.lld` (the `lld` package) selected with -fuse-ld=lld
#
# Host clang + lld are multi-target, so no separate per-arch gcc is needed; a
# thin per-arch wrapper script just pins --target + --sysroot for cargo / cc-rs.
#
# Integrity: instead of hardcoding a sha256 we can't recompute offline, we fetch
# the release's official CHECKSUM.SHA256 from the same FreeBSD mirror and verify
# base.txz against it -- immutable, signed-by-provenance, and never a pipe.
#
# This script is the single source of truth for the FreeBSD toolchain pin; both
# scripts/release.sh (detects RUSTCODE_FREEBSD_SYSROOT + CARGO_TARGET_*_LINKER)
# and .github/workflows/build.yml call it.
#
# Usage:
#   scripts/install-freebsd-cross.sh               # install (idempotent), print export lines
#   eval "$(scripts/install-freebsd-cross.sh)"     # ... and apply them to this shell
#   scripts/install-freebsd-cross.sh --github-env  # write to $GITHUB_ENV / $GITHUB_PATH
#
# Options:
#   --dest DIR       extraction root (default: /opt, or $FREEBSD_CROSS_DEST)
#   --version VER    FreeBSD release (default: the pin below)
#   --github-env     write the environment for later CI steps instead of printing
#
# Only the eval-able env belongs on stdout; all logs go to stderr so that
# `eval "$(...)"` never tries to execute a status line.
set -euo pipefail

usage() {
    cat <<'EOF' >&2
Usage: install-freebsd-cross.sh [--dest DIR] [--version VER] [--github-env]

  --dest DIR       extraction root (default: /opt, or $FREEBSD_CROSS_DEST)
  --version VER    FreeBSD release (default: the pinned version below)
  --github-env     write the environment for later CI steps instead of printing
EOF
}

# Pin the FreeBSD release. bump only with a deliberate decision -- the base
# tarball + its CHECKSUM.SHA256 must both exist at the mirror for this version.
FREEBSD_VER="${FREEBSD_CROSS_VERSION:-14.2-RELEASE}"
MIRROR="https://download.freebsd.org/releases"

# arch -> (release dir under $MIRROR, rust triple)
#   amd64  -> FreeBSD "amd64"        tree, triple x86_64-unknown-freebsd
#   aarch64-> FreeBSD "arm64"        tree, triple aarch64-unknown-freebsd
ARCH_TRIPLE_amd64="x86_64-unknown-freebsd"
ARCH_DIR_amd64="amd64"
ARCH_TRIPLE_aarch64="aarch64-unknown-freebsd"
ARCH_DIR_aarch64="arm64"
ARCHES="amd64 aarch64"

DEST_ROOT="${FREEBSD_CROSS_DEST:-/opt}"
GITHUB_ENV_MODE=0

while [ $# -gt 0 ]; do
    case "$1" in
        --dest)
            [ $# -ge 2 ] || { echo "[ERROR] --dest requires a directory" >&2; exit 2; }
            DEST_ROOT="$2"
            shift 2
            ;;
        --version)
            [ $# -ge 2 ] || { echo "[ERROR] --version requires a value" >&2; exit 2; }
            FREEBSD_VER="$2"
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

CROSS_BIN="${DEST_ROOT}/freebsd-cross/bin"
SYSROOT_ROOT="${DEST_ROOT}/freebsd-sysroot"

# Host toolchain prerequisites. Multi-target clang + lld are what make this work.
if ! command -v clang >/dev/null 2>&1; then
    echo "[ERROR] clang is required (host cross compiler)." >&2
    echo "        Ubuntu/Debian: sudo apt-get install -y clang lld" >&2
    exit 1
fi
if ! command -v ld.lld >/dev/null 2>&1; then
    echo "[ERROR] ld.lld is required (the 'lld' package provides it)." >&2
    echo "        Ubuntu/Debian: sudo apt-get install -y lld" >&2
    exit 1
fi

emit_env() {
    if [ "$GITHUB_ENV_MODE" = 1 ]; then
        echo "$CROSS_BIN" >> "$GITHUB_PATH"
        {
            echo "RUSTCODE_FREEBSD_SYSROOT=${SYSROOT_ROOT}"
            echo "CARGO_TARGET_X86_64_UNKNOWN_FREEBSD_LINKER=${CROSS_BIN}/x86_64-unknown-freebsd-clang"
            echo "CC_x86_64_unknown_freebsd=${CROSS_BIN}/x86_64-unknown-freebsd-clang"
            echo "CARGO_TARGET_AARCH64_UNKNOWN_FREEBSD_LINKER=${CROSS_BIN}/aarch64-unknown-freebsd-clang"
            echo "CC_aarch64_unknown_freebsd=${CROSS_BIN}/aarch64-unknown-freebsd-clang"
        } >> "$GITHUB_ENV"
    else
        printf 'export PATH=%s:$PATH\n' "$CROSS_BIN"
        printf 'export RUSTCODE_FREEBSD_SYSROOT=%s\n' "$SYSROOT_ROOT"
        printf 'export CARGO_TARGET_X86_64_UNKNOWN_FREEBSD_LINKER=%s\n' "${CROSS_BIN}/x86_64-unknown-freebsd-clang"
        printf 'export CC_x86_64_unknown_freebsd=%s\n' "${CROSS_BIN}/x86_64-unknown-freebsd-clang"
        printf 'export CARGO_TARGET_AARCH64_UNKNOWN_FREEBSD_LINKER=%s\n' "${CROSS_BIN}/aarch64-unknown-freebsd-clang"
        printf 'export CC_aarch64_unknown_freebsd=%s\n' "${CROSS_BIN}/aarch64-unknown-freebsd-clang"
    fi
}

# Idempotent: if the sysroot for both arches + both wrappers exist, reuse as is.
ALL_PRESENT=1
for a in $ARCHES; do
    triple="$(eval echo "\$ARCH_TRIPLE_$a")"
    if [ ! -x "${CROSS_BIN}/${triple}-clang" ] || [ ! -d "${SYSROOT_ROOT}/${a}/usr/lib" ]; then
        ALL_PRESENT=0
        break
    fi
done
if [ "$ALL_PRESENT" -eq 1 ]; then
    echo "[CHECK] FreeBSD cross toolchain already installed at ${DEST_ROOT}" >&2
    emit_env
    exit 0
fi

if ! command -v xz >/dev/null 2>&1; then
    echo "[ERROR] xz is required to unpack base.txz." >&2
    echo "        Ubuntu/Debian: sudo apt-get install -y xz-utils" >&2
    exit 1
fi

# sudo only when the destination actually needs it.
if [ "$(id -u)" -eq 0 ] || [ -w "$DEST_ROOT" ] || [ -w "$(dirname "$DEST_ROOT")" ]; then
    SUDO=""
else
    SUDO="sudo"
fi

mkdir -p "$CROSS_BIN" "$SYSROOT_ROOT"

# Per-arch wrapper: pins --target + --sysroot so cargo's linker call and cc-rs's
# C compiles both use the FreeBSD headers/libs. lld handles the FreeBSD ELF.
make_wrapper() {
    local triple="$1" sysroot="$2" out="$CROSS_BIN/${triple}-clang"
    cat > "$out" <<EOF
#!/bin/sh
exec clang --target=${triple} --sysroot=${sysroot} -fuse-ld=lld -B${sysroot}/usr/lib "\$@"
EOF
    chmod +x "$out"
}

for a in $ARCHES; do
    triple="$(eval echo "\$ARCH_TRIPLE_$a")"
    rel_dir="$(eval echo "\$ARCH_DIR_$a")"
    sysroot="${SYSROOT_ROOT}/${a}"
    [ -d "${sysroot}/usr/lib" ] && continue   # already provisioned for this arch

    base_url="${MIRROR}/${rel_dir}/${FREEBSD_VER}/base.txz"
    sum_url="${MIRROR}/${rel_dir}/${FREEBSD_VER}/CHECKSUM.SHA256"

    tmp="$(mktemp -d)"
    archive="${tmp}/base.txz"
    checksum="${tmp}/CHECKSUM.SHA256"
    echo "[INFO] Downloading FreeBSD ${FREEBSD_VER} base (${rel_dir})" >&2
    curl --fail --location --silent --show-error \
        --connect-timeout 15 --max-time 600 \
        --retry 5 --retry-delay 10 --retry-all-errors \
        --output "$archive" "$base_url"
    curl --fail --location --silent --show-error \
        --connect-timeout 15 --max-time 300 \
        --retry 5 --retry-delay 10 --retry-all-errors \
        --output "$checksum" "$sum_url"

    # Verify against the official checksum file (only the base.txz line).
    echo "[INFO] Verifying base.txz integrity against official CHECKSUM.SHA256" >&2
    if command -v sha256sum >/dev/null 2>&1; then
        grep -E "\(base\.txz\)|base\.txz" "$checksum" | sed -E 's/\(base\.txz\)//' \
            | sha256sum --check --strict - >&2
    else
        echo "[ERROR] sha256sum required for integrity check." >&2
        rm -rf "$tmp"
        exit 1
    fi

    echo "[INFO] Extracting sysroot to ${sysroot}" >&2
    $SUDO mkdir -p "$sysroot"
    $SUDO tar -xJf "$archive" -C "$sysroot"
    rm -rf "$tmp"

    make_wrapper "$triple" "$sysroot"
    echo "[CHECK] installed FreeBSD sysroot + wrapper for ${triple}" >&2
done

echo "[CHECK] installed FreeBSD cross toolchain at ${DEST_ROOT}" >&2
emit_env
