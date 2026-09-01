#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────────────
# RustCode npm package build script
# Usage:  ./scripts/build_npm_package.sh <version>
# Example: ./scripts/build_npm_package.sh 4.23.3 --dry-run
#          ./scripts/build_npm_package.sh 4.23.3
# ──────────────────────────────────────────────────────────────────────
set -euo pipefail

# Release target is operator/distributor-provided; this build ships no
# compiled-in release host. Version auto-detection uses a GitLab-v5-compatible
# contents API; binaries are fetched from a release download root.
#   RUSTCODE_RELEASE_API_HOST       API host, e.g. https://gitlab.example.com (bare host or
#                                   with a trailing /api/v5; both are normalized)
#   RUSTCODE_RELEASE_ACCESS_TOKEN   optional API token (PRIVATE-TOKEN) for private repos
#   RUSTCODE_RELEASE_OWNER          repo owner/namespace (required for auto-detect)
#   RUSTCODE_RELEASE_REPO           repo name (default: rustcode)
#   RUSTCODE_RELEASE_REF           branch/tag for version detection (default: main)
#   RUSTCODE_RELEASE_DOWNLOAD_BASE  root hosting v<ver>/rustcode-v<ver>-<os>-<arch> assets
#   JQ_URL                          optional override URL for fetching a jq binary
B="${RUSTCODE_RELEASE_API_HOST:-}"
# Accept bare host (https://gitlab.example.com) or v5 base (.../api/v5);
# normalize to the v5 base so the "$B/repos/..." paths below are correct.
if [ -n "$B" ]; then B="${B%/}"; B="${B%/api/v5}"; B="$B/api/v5"; fi
RELEASE_TOKEN="${RUSTCODE_RELEASE_ACCESS_TOKEN:-}"
RELEASE_OWNER="${RUSTCODE_RELEASE_OWNER:-}"
RELEASE_REPO="${RUSTCODE_RELEASE_REPO:-rustcode}"
RELEASE_REF="${RUSTCODE_RELEASE_REF:-main}"
DOWNLOAD_BASE="${RUSTCODE_RELEASE_DOWNLOAD_BASE:-}"

# ── version auto-detection (same helpers as packages/homebrew/scripts/package-tar-gz.sh) ──
et(){
    command -v curl &>/dev/null || return 1
    command -v jq &>/dev/null && return 0
    # Pick the jq asset for the current OS/arch (jq release asset names).
    case "$(uname -s)" in
        Darwin) j=jq-macos-$([ "$(uname -m)" = arm64 ] && echo arm64 || echo amd64) ;;
        Linux)  j=jq-linux-$([ "$(uname -m)" = aarch64 ] && echo arm64 || echo amd64) ;;
        *)      j=jq-macos-amd64 ;;
    esac
    g="https://github.com/jqlang/jq/releases/download/jq-1.7.1/$j"
    d=$(mktemp -d) || return 1; p=$d/jq
    for u in "${JQ_URL:-}" "$g"; do
        [[ $u ]] || continue
        curl -fsSL --connect-timeout 40 --retry 3 "$u" -o "$p" || continue
        # Portable byte count: `stat -f%z` is BSD/macOS-only; wc -c works on Linux too.
        s=$(wc -c < "$p" 2>/dev/null | tr -d '[:space:]'); s=${s:-0}
        [[ $s -ge 80000 ]] || { rm -f "$p"; continue; }
        chmod +x "$p" || { rm -f "$p"; continue; }
        "$p" -n . &>/dev/null || { rm -f "$p"; continue; }
        export PATH="$d:$PATH"
        command -v jq &>/dev/null && return 0
    done
    rm -rf "$d"; return 1
}

fct(){ local h=(); [ -n "$RELEASE_TOKEN" ] && h=(-H "PRIVATE-TOKEN: $RELEASE_TOKEN");
    curl -sS "${h[@]}" -H "Accept: application/json" \
    "$B/repos/$RELEASE_OWNER/$RELEASE_REPO/contents/Cargo.toml?ref=$(jq -rn --arg r "$RELEASE_REF" '$r|@uri')"; }

pvs(){
    local t v
    t=$(jq -er 'select(.type=="file")|.content|gsub("[[:space:]]";"")|@base64d' <<<"$1")
    v=$(awk 'BEGIN{f=0} index($0,"[workspace.package]")==1{w=1;next} substr($0,1,1)=="["{w=0}
        w&&/^version *=/{if(match($0,/"[^"]+"/)){print substr($0,RSTART+1,RLENGTH-2);f=1;exit}}
        END{exit !f}' <<<"$t")
    [[ $v ]] || exit 1; echo "$v"
}

# ── resolve version ──
if [ $# -ge 1 ] && [[ "$1" != -* ]]; then
    VERSION="$1"
    shift
else
    [ -n "$B" ] && [ -n "$RELEASE_OWNER" ] || {
        echo "Error: auto-detect needs RUSTCODE_RELEASE_API_HOST and RUSTCODE_RELEASE_OWNER,"
        echo "       or pass <version> explicitly."; exit 1; }
    et || { echo "Error: jq not available"; exit 1; }
    j=$(fct) || { echo "Error: failed to fetch Cargo.toml"; exit 1; }
    jq -e .error_code <<<"$j" &>/dev/null && { echo "Error fetching Cargo.toml: $(echo "$j" | jq -r .message)"; exit 1; }
    VERSION=$(pvs "$j") || { echo "Error: failed to parse version from Cargo.toml"; exit 1; }
fi

# Capture remaining args for npm publish (e.g. --dry-run, --otp=...)
NPM_EXTRA="${*:-}"

NPM_DIR="$(cd "$(dirname "$0")/.." && pwd)"
WORK_DIR=$(mktemp -d)
cleanup() { rm -rf "$WORK_DIR"; }
trap cleanup EXIT

PLATFORMS=(
  "darwin-arm64:darwin:arm64"
  "darwin-x64:darwin:x64"
  "linux-arm64:linux:arm64"
  "linux-x64:linux:x64"
  "win32-x64:win32:x64"
  "ohos-arm64:ohos:arm64"
)

publish_platform() {
  local tag="$1" os="$2" arch="$3"
  local dir="$WORK_DIR/$tag"
  mkdir -p "$dir/bin"

  # generate package.json dynamically — 就几行
  cat > "$dir/package.json" <<EOF
{"name":"@rustcode/rustcode","version":"${VERSION}-${tag}","os":["${os}"],"cpu":["${arch}"],"files":["bin/"]}
EOF

  # download binary
  [ -n "$DOWNLOAD_BASE" ] || { echo "Error: RUSTCODE_RELEASE_DOWNLOAD_BASE is required to fetch binaries"; exit 1; }
  local dl_os="$os"
  [ "$os" = "win32" ] && dl_os="windows"
  local bin_name="rustcode$([ "$os" = "win32" ] && echo ".exe")"
  local url="${DOWNLOAD_BASE%/}/v${VERSION}/rustcode-v${VERSION}-${dl_os}-${arch}$([ "$os" = "win32" ] && echo ".exe")"

  echo "  [*] downloading ${tag}..."
  local http_code
  # curl -f exits non-zero on HTTP 404; `|| true` keeps set -e from aborting so
  # the skip-on-missing-platform check below is reachable (matches put()/upl()).
  if [ -n "$RELEASE_TOKEN" ]; then
    http_code=$(curl -fsSL -w '%{http_code}' -H "PRIVATE-TOKEN: $RELEASE_TOKEN" --connect-timeout 30 --retry 3 "$url" -o "$dir/bin/$bin_name" 2>/dev/null) || true
  else
    http_code=$(curl -fsSL -w '%{http_code}' --connect-timeout 30 --retry 3 "$url" -o "$dir/bin/$bin_name" 2>/dev/null) || true
  fi
  if [ "$http_code" = "404" ]; then
    echo "  [!] binary not found for ${tag}, skipping"
    rm -f "$dir/bin/$bin_name"
    return 0
  fi
  chmod +x "$dir/bin/$bin_name"

  # publish
  cd "$dir"
  npm publish --registry=https://registry.npmjs.org/ --access public $NPM_EXTRA
  echo "  - @rustcode/rustcode@${VERSION}-${tag}"
}

echo ""
echo "  Publishing @rustcode/rustcode v${VERSION}"
echo ""

# 1. publish platform versions
for entry in "${PLATFORMS[@]}"; do
  IFS=: read -r tag os arch <<< "$entry"
  publish_platform "$tag" "$os" "$arch"
done

# 2. publish core
CORE_DIR="$WORK_DIR/core"
mkdir -p "$CORE_DIR/bin"
cp "$NPM_DIR/package.json" "$CORE_DIR/"
cp "$NPM_DIR/bin/rustcode.js" "$CORE_DIR/bin/"
cd "$CORE_DIR"
# Inject version + optionalDependencies dynamically (like Codex does in CI)
node -e "
var fs = require('fs');
var pkg = JSON.parse(fs.readFileSync('package.json', 'utf8'));
pkg.version = '$VERSION';
pkg.optionalDependencies = {
  '@rustcode/rustcode-darwin-arm64': 'npm:@rustcode/rustcode@$VERSION-darwin-arm64',
  '@rustcode/rustcode-darwin-x64': 'npm:@rustcode/rustcode@$VERSION-darwin-x64',
  '@rustcode/rustcode-linux-arm64': 'npm:@rustcode/rustcode@$VERSION-linux-arm64',
  '@rustcode/rustcode-linux-x64': 'npm:@rustcode/rustcode@$VERSION-linux-x64',
  '@rustcode/rustcode-win32-x64': 'npm:@rustcode/rustcode@$VERSION-win32-x64',
  '@rustcode/rustcode-ohos-arm64': 'npm:@rustcode/rustcode@$VERSION-ohos-arm64'
};
fs.writeFileSync('package.json', JSON.stringify(pkg, null, 2) + '\n');
"
npm publish --registry=https://registry.npmjs.org/ --access public $NPM_EXTRA
echo "  - @rustcode/rustcode@${VERSION} (core)"
echo ""
echo "  All done!"
