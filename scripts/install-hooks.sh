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

# The pre-push hook shells out to the release gate; keep it executable too.
if [ -f "scripts/prepush-release-check.sh" ]; then
  chmod +x scripts/prepush-release-check.sh
fi

# Configure git to use the .githooks directory
git config core.hooksPath "$HOOKS_DIR"

# main is an upstream mirror (see AGENTS.md branch policy). The upstream remote
# and main's tracking relationship live in .git/config, i.e. they are not
# versioned, so make them idempotently present here.
#
# NOTE: this fork's canonical upstream repo still carries the legacy product name
# in its URL, which the CI legacy-naming gate (G7/G8) forbids inside
# scripts/. So we do NOT hardcode that URL here. The default points to THIS
# project's own repo; override with GIT_UPSTREAM_URL to track a real external
# upstream (required for branch protection against fork-authored main commits).
# The pre-push hook prints the exact command when upstream/main is missing.
UPSTREAM_URL="${GIT_UPSTREAM_URL:-https://gitcode.com/SecLab/RustCode}"
if ! git remote get-url upstream >/dev/null 2>&1; then
  git remote add upstream "$UPSTREAM_URL"
  echo "[OK] Added remote 'upstream' -> $UPSTREAM_URL"
fi
if [ -z "$(git config --get remote.upstream.tagOpt || true)" ]; then
  git config remote.upstream.tagOpt --no-tags
fi
if git rev-parse --verify --quiet refs/heads/main >/dev/null 2>&1; then
  if git rev-parse --verify --quiet refs/remotes/upstream/main >/dev/null 2>&1; then
    if git branch --set-upstream-to=upstream/main main >/dev/null 2>&1; then
      echo "[OK] main now tracks upstream/main"
    else
      echo "[WARN] could not set main to track upstream/main"
    fi
  else
    echo "[WARN] refs/remotes/upstream/main not fetched yet; run:"
    echo "       git fetch upstream main --no-tags && git branch --set-upstream-to=upstream/main main"
  fi
fi

echo "[OK] Git hooks installed."
echo "     core.hooksPath = $HOOKS_DIR"
echo ""
echo "  pre-push: blocks fork-authored commits on main (upstream-only sync)"
echo "  pre-push: requires a committed release/<version>/ artifact for this host"
echo "            (non-main branches only; main is an upstream mirror)"
echo "            (RUSTCODE_PREPUSH_RELEASE=off to skip, =strict for exact-code proof;"
echo "             scripts/prepush-release-check.sh --self-test to verify the gate)"
echo ""
echo "  To uninstall: git config --unset core.hooksPath"
echo "  To bypass:    git push --no-verify (NOT recommended)"
