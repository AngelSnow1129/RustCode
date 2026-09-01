#!/bin/sh
[ -n "${BASH_VERSION:-}" ] || exec /bin/bash "$0" "$@"
set -euo pipefail

# Package release binaries into tar.gz archives for Homebrew-cask.
#
# Release target is operator/distributor-provided; this build ships no
# compiled-in release host. The API is a GitLab-v5-compatible release API
# (contents + releases/<tag>/upload_url endpoints).
#
# Environment:
#   RUSTCODE_RELEASE_API_HOST       API host, e.g. https://gitlab.example.com (bare host or
#                                   with a trailing /api/v5; both are normalized) (required)
#   RUSTCODE_RELEASE_ACCESS_TOKEN   release API access token (required)
#   RUSTCODE_RELEASE_OWNER          repo owner/namespace (required)
#   RUSTCODE_RELEASE_REPO           repo name (default: rustcode)
#   RUSTCODE_RELEASE_REF            branch/tag for version detection (default: main)
#   RUSTCODE_RELEASE_VERSION        override version (skip API detection)
#   RUSTCODE_RELEASE_DOWNLOAD_BASE  root hosting v<tag>/rustcode-v<tag>-<plat> binaries (required)
#   JQ_URL                          optional override URL for fetching a jq binary

: "${RUSTCODE_RELEASE_API_HOST:?RUSTCODE_RELEASE_API_HOST is required (GitLab-v5 API base)}"
: "${RUSTCODE_RELEASE_ACCESS_TOKEN:?RUSTCODE_RELEASE_ACCESS_TOKEN is required}"
: "${RUSTCODE_RELEASE_OWNER:?RUSTCODE_RELEASE_OWNER is required}"
: "${RUSTCODE_RELEASE_DOWNLOAD_BASE:?RUSTCODE_RELEASE_DOWNLOAD_BASE is required}"

# Accept bare host (https://gitlab.example.com) or v5 base (.../api/v5);
# normalize to the v5 base so the "$B/repos/..." paths below are correct.
B="${RUSTCODE_RELEASE_API_HOST%/}"
B="${B%/api/v5}"
B="$B/api/v5"
T="$RUSTCODE_RELEASE_ACCESS_TOKEN"
o="$RUSTCODE_RELEASE_OWNER"
r="${RUSTCODE_RELEASE_REPO:-rustcode}"
ref="${RUSTCODE_RELEASE_REF:-main}"
DL_ROOT="${RUSTCODE_RELEASE_DOWNLOAD_BASE%/}"

# ── helpers (same pattern as ci-release scripts) ──
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

fct(){ curl -sS -H "PRIVATE-TOKEN: $T" -H "Accept: application/json" \
    "$B/repos/$o/$r/contents/Cargo.toml?ref=$(jq -rn --arg r "$ref" '$r|@uri')"; }

pvs(){
    local t v
    t=$(jq -er 'select(.type=="file")|.content|gsub("[[:space:]]";"")|@base64d' <<<"$1")
    v=$(awk 'BEGIN{f=0} index($0,"[workspace.package]")==1{w=1;next} substr($0,1,1)=="["{w=0}
        w&&/^version *=/{if(match($0,/"[^"]+"/)){print substr($0,RSTART+1,RLENGTH-2);f=1;exit}}
        END{exit !f}' <<<"$t")
    [[ $v ]] || exit 1; echo "$v"
}

put(){
    local j="$1" f="$2" u o c a=() h
    u=$(jq -r '.url' <<<"$j")
    h=$(mktemp) || exit 1
    jq -r '.headers|to_entries[]|(.key+": "+(.value|tostring))' <<<"$j" > "$h"
    a=(); while IFS= read -r l; do [[ $l ]] && a+=(-H "$l"); done < "$h"
    rm -f "$h"
    o=$(mktemp) || exit 1
    c=$(curl -sS -o "$o" -w '%{http_code}' -X PUT "$u" "${a[@]}" --data-binary "@$f") || :
    [[ $c -ge 200 && $c -le 299 ]] && { rm -f "$o"; return 0; }
    rm -f "$o"; exit 1
}

upl(){
    local tg="$1" fn="$2" bin="$3" u z c R
    fn="rustcode-${tg}-${fn}"
    u="$B/repos/$o/$r/releases/${tg}/upload_url?access_token=$T&file_name=$fn"
    z=$(mktemp) || exit 1
    c=$(curl -sS -o "$z" -w '%{http_code}' -X GET "$u" \
        -H "PRIVATE-TOKEN:$T" -H "Accept: application/json") || :
    R=$(<"$z"); rm -f "$z"
    [[ $c =~ ^2[0-9][0-9]$ ]] || exit 1
    put "$R" "$bin"
}

# ── version ──
et || exit 1

if [ -n "${RUSTCODE_RELEASE_VERSION:-}" ]; then
    tag="v${RUSTCODE_RELEASE_VERSION#v}"
else
    j=$(fct) || { echo "Error: failed to fetch Cargo.toml"; exit 1; }
    jq -e .error_code <<<"$j" &>/dev/null && { echo "Error fetching Cargo.toml: $(echo "$j" | jq -r .message)"; exit 1; }
    ver=$(pvs "$j") || { echo "Error: failed to parse version from Cargo.toml"; exit 1; }
    tag="v$ver"
fi

case "$tag" in v[0-9]*) ;; *) echo "Invalid version: $tag"; exit 1 ;; esac
echo "==> package-tar-gz.sh ${tag}"
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT

# ── platforms ──
PLATFORMS="darwin-arm64 darwin-x64 linux-arm64 linux-x64"
DOWNLOAD_BASE="${DL_ROOT}/${tag}"

# ── download raw binaries ──
echo "[1/4] Downloading raw binaries ..."
for plat in $PLATFORMS; do
    fn="rustcode-${tag}-${plat}"
    echo "       ${fn} ..."
    curl -fL --connect-timeout 30 --retry 3 -o "${WORK}/${fn}" "${DOWNLOAD_BASE}/${fn}"
done

# ── package tar.gz ──
echo "[2/4] Packaging tar.gz archives ..."
for plat in $PLATFORMS; do
    src="rustcode-${tag}-${plat}"
    dst="${src}.tar.gz"
    cp "${WORK}/${src}" "${WORK}/rustcode"
    tar czf "${WORK}/${dst}" -C "$WORK" "rustcode"
    rm "${WORK}/rustcode"
    echo "       ${dst}"
done

# ── upload to release ──
echo "[3/4] Uploading tar.gz to release ..."
for plat in $PLATFORMS; do
    fn="rustcode-${tag}-${plat}.tar.gz"
    echo "       ${fn} ..."
    upl "$tag" "${plat}.tar.gz" "${WORK}/${fn}"
    echo "       done"
done

# ── output checksums for update-cask.sh ──
echo ""
echo "Done. Checksums:"
shasum -a 256 "${WORK}"/rustcode-*.tar.gz
