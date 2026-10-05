#!/usr/bin/env bash
# scripts/dev-common.sh — shared scaffolding for RustCode dev bootstrap scripts.
#
# Source this near the top of a dev script:
#     SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
#     # shellcheck source=dev-common.sh
#     . "$SCRIPT_DIR/dev-common.sh"
#     TAG=setup            # sets the [label] prefix printed by the helpers
#
# It defines the color helpers (info/success/warn/error/step) and a single
# detect_platform() that fills OS / ARCH / PLATFORM. Keeping these in one place
# stops setup.sh and dev-env-quickstart.sh from drifting apart.

# ── Color helpers ────────────────────────────────────────────────────────────
C_RESET='\033[0m'; C_BOLD='\033[1m'
C_GREEN='\033[0;32m'; C_YELLOW='\033[0;33m'; C_CYAN='\033[0;36m'; C_RED='\033[0;31m'

# TAG is the [label] prefix shown by the helpers; each script sets its own.
TAG="${TAG:-dev}"

info()    { echo -e "${C_CYAN}[${TAG}]${C_RESET} $*" ; }
success() { echo -e "${C_GREEN}[ok]${C_RESET}    $*" ; }
warn()    { echo -e "${C_YELLOW}[warn]${C_RESET}  $*" ; }
error()   { echo -e "${C_RED}[error]${C_RESET} $*" >&2 ; exit 1 ; }
step()    { echo -e "\n${C_BOLD}==> $*${C_RESET}" ; }

# ── OS / ARCH detection ──────────────────────────────────────────────────────
# Fills OS (uname -s), ARCH (uname -m) and PLATFORM (macos | linux | freebsd | "").
# Callers decide which platforms they support and error otherwise.
detect_platform() {
    OS="$(uname -s)"; ARCH="$(uname -m)"
    case "$OS" in
        Darwin)  PLATFORM="macos" ;;
        Linux)   PLATFORM="linux" ;;
        FreeBSD) PLATFORM="freebsd" ;;
        *)       PLATFORM="" ;;
    esac
}
