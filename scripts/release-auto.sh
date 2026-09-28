#!/usr/bin/env bash
# scripts/release-auto.sh — automatic patch release: bump -> build -> publish ->
# commit -> tag -> (optional) remote Release.
#
# WHY THIS EXISTS
#   The whole release chain is TAG-DRIVEN (.github/workflows/build.yml fires only
#   on `push: tags: v*`) but NOTHING in the repo ever CREATES a git tag. That is
#   the root cause of "artifacts exist, no Release and no tag" (e.g. v6.2.0):
#   `gitcode_release.py --tag-name` only creates the remote Release object for a
#   tag name it is GIVEN; it never tags. This script is the missing producer.
#
# WHAT IT DOES (idempotent, fail-closed):
#   1. Skip unless the branch is `dev` and there are source changes (crates/ or
#      Cargo.toml) since the newest vX.Y.Z tag -- a release/artifact-only or
#      docs-only push must NOT consume a version number.
#   2. Bump the PATCH field of [workspace.package].version in Cargo.toml.
#   3. Cross-build the host-capable targets into dist/<version>/.
#   4. Publish into the repo-backed release/<version>/ + regenerate index.json
#      via scripts/release-publish.sh (never deletes already-committed binaries).
#   5. Commit ONLY Cargo.toml, Cargo.lock, release/ (never sweeps in other WIP).
#   6. Create the annotated git tag v<version>.
#   7. Optionally upload to GitCode + Gitee when their tokens are present.
#
# IMPORTANT GIT SEMANTICS (read before wiring this into pre-push)
#   git computes the commits to push BEFORE running the pre-push hook. A commit
#   created inside the hook therefore does NOT ride this push -- it rides the
#   NEXT one. The tag this script creates is pushed separately and explicitly
#   (see step 8), so the tag and its Release are not lost; the version bump
#   commit lands on the following `git push`. Use `git push --follow-tags` or
#   just push again to carry it.
#
# Env:
#   RUSTCODE_AUTO_RELEASE       off       disable entirely (default: on)
#   RUSTCODE_AUTO_RELEASE_PUSH  1         push the new tag to origin (default: 0)
#   RUSTCODE_AUTO_BUILD_TARGETS "..."     override the target list
#   RUSTCODE_RELEASE_ACCESS_TOKEN         GitCode PAT (optional)
#   GITEE_ACCESS_TOKEN                    Gitee PAT (optional)
#
# Usage: scripts/release-auto.sh [--dry-run]

set -euo pipefail
cd "$(dirname "$0")/.."
ROOT="$(pwd)"

DRY_RUN=0
[ "${1:-}" = "--dry-run" ] && DRY_RUN=1

[ "${RUSTCODE_AUTO_RELEASE:-on}" = "off" ] && { echo "[INFO] RUSTCODE_AUTO_RELEASE=off; skipping."; exit 0; }

say() { printf '%s\n' "$*"; }
run() { if [ "$DRY_RUN" = "1" ]; then printf '[dry-run] %s\n' "$*"; else eval "$*"; fi; }

# ---------- 0. branch guard -------------------------------------------------
BRANCH="$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo unknown)"
if [ "$BRANCH" != "dev" ]; then
  say "[INFO] auto-release only runs on dev (current: $BRANCH); skipping."
  exit 0
fi

# ---------- 1. newest tag + source-change detection -------------------------
version_of_cargo() {
  awk -F'"' '
    /^\[workspace\.package\]/ { in_section = 1; next }
    /^\[/ { in_section = 0 }
    in_section && /^version *=/ { print $2; exit }
  ' Cargo.toml
}

NEWEST_TAG="$(git tag --list 'v*' | sed 's/^v//' | sort -t. -k1,1n -k2,2n -k3,3n | tail -n1 || true)"
[ -n "$NEWEST_TAG" ] && NEWEST_TAG="v$NEWEST_TAG"

if [ -n "$NEWEST_TAG" ]; then
  CHANGED="$(git diff --name-only "$NEWEST_TAG"..HEAD -- crates/ Cargo.toml 2>/dev/null || true)"
else
  # No tag at all: treat every crates/ file in HEAD as a change.
  CHANGED="$(git ls-tree -r --name-only HEAD -- crates/ Cargo.toml 2>/dev/null || true)"
fi

if [ -z "$CHANGED" ]; then
  say "[INFO] no source change since ${NEWEST_TAG:-<no tag>}; version bump not needed."
  exit 0
fi

# ---------- 2. bump patch ---------------------------------------------------
CUR="$(version_of_cargo)"
if [ -z "$CUR" ]; then
  say "[ERROR] could not read [workspace.package].version from Cargo.toml" >&2
  exit 1
fi
MAJOR="${CUR%%.*}"
REST="${CUR#*.}"
MINOR="${REST%%.*}"
PATCH="${REST#*.}"
case "$PATCH" in
  ''|*[!0-9]*) say "[ERROR] non-numeric patch field in '$CUR'" >&2; exit 1 ;;
esac
NEXT="${MAJOR}.${MINOR}.$((PATCH + 1))"
VERSION="v${NEXT}"
say "[CHECK] auto-release: ${CUR} -> ${NEXT} (source changed since ${NEWEST_TAG:-<none>})"

if [ "$DRY_RUN" = "1" ]; then
  say "[dry-run] would bump Cargo.toml, build, publish, commit and tag $VERSION"
  exit 0
fi

# Rewrite ONLY the workspace version line (crates inherit via version.workspace).
# Use `nv` (not `next`) as the awk var: some awk impls reserve `next`.
awk -v nv="$NEXT" '
  /^\[workspace\.package\]/ { in_section = 1; print; next }
  /^\[/ { in_section = 0 }
  in_section && /^version *=/ { printf "version = \"%s\"\n", nv; next }
  { print }
' Cargo.toml > Cargo.toml.new
mv Cargo.toml.new Cargo.toml

# ---------- 3. cross-build --------------------------------------------------
TARGETS="${RUSTCODE_AUTO_BUILD_TARGETS:-linux-x64 linux-arm64 windows-x64}"
say "=== Building $VERSION for: $TARGETS ==="
if [ -f webui/dist/index.html ]; then
  say "[INFO] webui/dist present; skipping frontend rebuild."
else
  "$ROOT/scripts/build-webui.sh"
fi
RUSTCODE_BUILD_TARGETS="$TARGETS" bash "$ROOT/scripts/cross-build.sh" || {
  say "[ERROR] cross-build failed; Cargo.toml has been bumped to $NEXT but nothing was tagged." >&2
  say "        Fix the build, then re-run: scripts/release-auto.sh" >&2
  exit 1
}

# ---------- 4. publish into release/ ---------------------------------------
DIST="$ROOT/dist/$VERSION"
if [ ! -d "$DIST" ]; then
  say "[ERROR] $DIST not found after cross-build." >&2
  exit 1
fi
bash "$ROOT/scripts/release-publish.sh" "$DIST" "$VERSION" dev

# ---------- 5. commit (release/ + version only) ----------------------------
git add -- Cargo.toml Cargo.lock release
if git diff --cached --quiet; then
  say "[WARN] nothing staged after publish; no commit created."
else
  git commit -m "chore(release): bump version to ${NEXT} and publish artifacts"
fi

# ---------- 6. tag ----------------------------------------------------------
if git rev-parse --verify --quiet "refs/tags/$VERSION" >/dev/null; then
  say "[WARN] tag $VERSION already exists; not re-creating."
else
  git tag -a "$VERSION" -m "Release $VERSION"
  say "[CHECK] created tag $VERSION"
fi

# ---------- 7. remote Release (GitCode + Gitee; tokens optional) -----------
# Both publishers take an explicit --tag-name: they create/update the remote
# Release object for a tag that already exists. Missing tokens are skipped
# loudly rather than failing the whole release -- the repo-backed release/
# directory remains the fallback source for install.sh.
publish_assets() {
  local script="$1" host="$2" token="$3"
  if [ -z "$token" ]; then
    say "[WARN] no token for $host; skipping remote Release (release/ fallback still works)."
    return 0
  fi
  local file
  for file in "$DIST"/rustcode-*; do
    [ -f "$file" ] || continue
    python3 "scripts/$script" \
      --tag-name "$VERSION" \
      --file-name "$(basename "$file")" \
      --file-path "$file" \
      --target-commitish "$(git rev-parse HEAD)" \
      --access-token "$token" || say "[WARN] failed to attach $(basename "$file") to $host"
  done
}
publish_assets gitcode_release.py GitCode "${RUSTCODE_RELEASE_ACCESS_TOKEN:-}"
publish_assets gitee_release.py Gitee "${GITEE_ACCESS_TOKEN:-}"

# ---------- 8. push the tag (opt-in) ---------------------------------------
if [ "${RUSTCODE_AUTO_RELEASE_PUSH:-0}" = "1" ]; then
  git push origin "$VERSION"
else
  say ""
  say "[CHECK] Tag $VERSION created locally. Push it with:"
  say "          git push origin $VERSION"
  say "        The version-bump commit rides the NEXT 'git push' (git decides"
  say "        what to push before this hook runs) — run it once more to send it."
fi
