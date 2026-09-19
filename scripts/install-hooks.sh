#!/usr/bin/env bash
# Install git hooks for branch protection.
#
# This script configures git to use the .githooks directory for hooks,
# enabling the pre-push branch protection hook that prevents direct
# commits to the main branch.
#
# Usage: ./scripts/install-hooks.sh
set -euo pipefail

cd "$(dirname "$0")/.."

HOOKS_DIR=".githooks"

if [ ! -d "$HOOKS_DIR" ]; then
  echo "[ERROR] $HOOKS_DIR directory not found" >&2
  exit 1
fi

# Make sure hook scripts are executable
chmod +x "$HOOKS_DIR"/* 2>/dev/null || true

# Configure git to use the .githooks directory
git config core.hooksPath "$HOOKS_DIR"

echo "[OK] Git hooks installed."
echo "     core.hooksPath = $HOOKS_DIR"
echo ""
echo "  pre-push: blocks direct commits to main (dev-only development)"
echo ""
echo "  To uninstall: git config --unset core.hooksPath"
echo "  To bypass:    git push --no-verify (NOT recommended)"
