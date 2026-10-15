#!/usr/bin/env bash
# scripts/release-publish.sh — publish a successful build into the repo-backed
# release/ directory so the download script (install.sh / install.ps1) can fall
# back to it when the CI/CD pipeline or the online Release is unavailable.
#
# Design constraints (see AGENTS.md + user release-redundancy plan):
#   * One directory per VERSION. No OS/ARCH sub-split — the binary filename
#     already encodes OS + ARCH (rustcode-<ver>-<os>-<arch>[.exe]).
#   * The dev host builds whatever it can (native + any cross linkers it has);
#     this script MERGES new binaries into release/<version>/ and NEVER deletes
#     an already-committed binary, so a partial / failed build can never clobber
#     a previously good one.
#   * Publish is atomic per file (stage in release/.tmp/<version>, then rename
#     into release/<version>) so a crashed pipeline never leaves a half-written
#     version dir behind.
#   * Emits two manifests:
#       release/<version>/manifest.json  — binaries for this version
#       release/index.json               — top-level index of all versions + targets
#   * Records build provenance in the per-version manifest
#     (`source.sha` / `source.branch` / `source.dirty` / `source.built_at`) so
#     scripts/prepush-release-check.sh can tell a fresh artifact from a stale one.
#     NOTE: this is an additive field — install.sh / install.ps1 only read
#     `binaries`, so older readers keep working.
#
# Usage (sourced or executed):
#   scripts/release-publish.sh <dist_dir> <version> [ref]
#   scripts/release-publish.sh --self-test
#
# <dist_dir> : directory containing the built rustcode-* / rustcode-daemon-* files
# <version>  : vX.Y.Z (must start with 'v')
# [ref]      : git ref recorded in index.json for the repo-raw fallback URL
#              (default: current branch, falls back to 'dev')

set -euo pipefail

RELEASE_PUBLISH_ROOT="${RELEASE_PUBLISH_ROOT:-$(cd "$(dirname "$0")/.." && pwd)}"

# ---------- semver helpers (vX.Y.Z, no prerelease) ----------
ver_split() {
    # echo "X Y Z" for vX.Y.Z
    local v="$1"
    v="${v#v}"
    printf '%s %s %s' "${v%%.*}" "$(printf '%s' "$v" | cut -d. -f2)" "$(printf '%s' "$v" | cut -d. -f3)"
}

ver_gt() {
    # ver_gt A B -> 0 if A > B
    local a1 a2 a3 b1 b2 b3
    read -r a1 a2 a3 <<<"$(ver_split "$1")"
    read -r b1 b2 b3 <<<"$(ver_split "$2")"
    a1="${a1:-0}"; a2="${a2:-0}"; a3="${a3:-0}"
    b1="${b1:-0}"; b2="${b2:-0}"; b3="${b3:-0}"
    if [ "$a1" -gt "$b1" ]; then return 0; fi
    if [ "$a1" -lt "$b1" ]; then return 1; fi
    if [ "$a2" -gt "$b2" ]; then return 0; fi
    if [ "$a2" -lt "$b2" ]; then return 1; fi
    [ "$a3" -gt "$b3" ]
}

# ---------- platform/target detection (mirror install.sh) ----------
detect_target_tag() {
    # Given the filename rustcode-<ver>-<os>-<arch>[.exe] (or rustcode-daemon-...),
    # echo "<os>-<arch>". Returns empty if it does not match the known shape.
    local f="$1"
    f="$(basename "$f")"
    case "$f" in
        rustcode-daemon-*) f="${f#rustcode-daemon-}" ;;
        rustcode-*)        f="${f#rustcode-}" ;;
        *) return 1 ;;
    esac
    f="${f%.exe}"
    # now <ver>-<os>-<arch>
    local ver="${f%%-*}"
    local rest="${f#"$ver"-}"
    local os="${rest%%-*}"
    local arch="${rest#"$os"-}"
    case "$os" in
        darwin|linux|ohos|windows|freebsd) ;;
        *) return 1 ;;
    esac
    case "$arch" in
        arm64|x64) ;;
        *) return 1 ;;
    esac
    printf '%s-%s' "$os" "$arch"
}

bin_sha256() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | awk '{print $1}'
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$1" | awk '{print $1}'
    else
        echo ""
    fi
}

bin_size() {
    if stat -f%z "$1" >/dev/null 2>&1; then
        stat -f%z "$1"
    else
        stat -c%s "$1"
    fi
}

is_html() {
    # true if the file looks like an HTML error page (first byte is '<' == 0x3c)
    local c
    c="$(head -c 1 "$1" 2>/dev/null | od -An -tx1 | tr -d ' \n')"
    [ "$c" = "3c" ]
}

# ---------- root latest.json (self-update manifest) ----------
# Shape (read by the TUI `/upgrade` command):
#   { "version": "vX.Y.Z", "released_at": "<ISO8601 Z>",
#     "binaries": { "<os>-<arch>": { "sha256": "<64 hex>", "size": N } } }
#
# HISTORY / why this lives here: latest.json used to be emitted ONLY by
# scripts/release.sh, the full-matrix script. That script cannot complete on a
# Linux host (its macOS cross steps fail), so in practice latest.json froze at
# whatever version a macOS host last published -- while newer releases kept
# shipping to the online Release. `/upgrade` then pointed users at a version
# whose online Release carried no binaries at all. Every publish now refreshes
# it from the binaries that just landed, so the pointer cannot drift again.
#
# Only targets present in release/<version>/manifest.json are emitted, so the
# manifest can never advertise a 404.
latest_json_binaries() {
    # $1 = manifest.json path -> lines "<target> <sha256> <size>"
    local man="$1"
    [ -f "$man" ] || return 1
    sed -n 's/^[[:space:]]*"\([a-z][a-z0-9]*-\(arm64\|x64\)\)":[[:space:]]*{.*"sha256":[[:space:]]*"\([0-9a-fA-F]\{64\}\)".*"size":[[:space:]]*\([0-9]\{1,\}\).*$/\1 \3 \4/p' "$man"
}

update_latest_json() {
    local version="$1"
    local rel_dir="$2"
    local man="$rel_dir/$version/manifest.json"
    if [ ! -f "$man" ]; then
        echo "[publish] WARNING: no $version/manifest.json; latest.json left untouched" >&2
        return 1
    fi
    local entries
    entries="$(latest_json_binaries "$man")"
    if [ -z "$entries" ]; then
        echo "[publish] WARNING: $version/manifest.json has no binaries; latest.json left untouched" >&2
        return 1
    fi

    local staged="$rel_dir/latest.json.tmp"
    {
        printf '{\n'
        printf '  "version": "%s",\n' "$version"
        printf '  "released_at": "%s",\n' "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
        printf '  "binaries": {\n'
        local first=1 t s sz
        # while-read over a here-string (not a pipe) so `first` survives the loop.
        while read -r t s sz; do
            [ -n "$t" ] || continue
            if [ "$first" -eq 0 ]; then printf ',\n'; fi
            printf '    "%s": { "sha256": "%s", "size": %s }' "$t" "$s" "$sz"
            first=0
        done <<EOF
$entries
EOF
        printf '\n  }\n'
        printf '}\n'
    } > "$staged"
    mv "$staged" "$RELEASE_PUBLISH_ROOT/latest.json"
    echo "[publish] latest.json -> $version ($(printf '%s\n' "$entries" | wc -l | tr -d ' ') targets)"
    return 0
}

# ---------- the publish action ----------
publish_release() {
    local dist_dir="$1"
    local version="$2"
    local ref="${3:-}"

    if [ ! -d "$dist_dir" ]; then
        echo "[publish] dist dir not found: $dist_dir" >&2
        return 1
    fi
    case "$version" in
        v[0-9]*) ;;
        *)
            echo "[publish] refusing non-vX.Y.Z version: '$version'" >&2
            return 1
            ;;
    esac
    if [ -z "$ref" ]; then
        ref="$(git -C "$RELEASE_PUBLISH_ROOT" rev-parse --abbrev-ref HEAD 2>/dev/null || true)"
        ref="${ref:-dev}"
    fi

    # Source provenance for the pre-push gate (scripts/prepush-release-check.sh):
    # which commit produced these binaries, and whether the source tree was dirty
    # at build time. A dirty build is NOT reproducible, so the strict gate refuses
    # to treat it as proof that "this commit compiles".
    #
    # release/ itself is excluded from the dirty probe -- publishing is precisely
    # what makes it dirty (fresh untracked binaries), so counting it would make
    # every build look dirty.
    local src_sha="" src_branch="" src_dirty="false" built_at
    # RUSTCODE_PUBLISH_SRC_SHA pins the provenance commit. Needed when binaries
    # built elsewhere (e.g. a CI runner, or a detached worktree at the release
    # tag) are published after dev has already moved on: without it the manifest
    # would claim "built from HEAD", which is a lie the strict pre-push gate
    # would rightly reject.
    src_sha="${RUSTCODE_PUBLISH_SRC_SHA:-$(git -C "$RELEASE_PUBLISH_ROOT" rev-parse HEAD 2>/dev/null || true)}"
    src_branch="$(git -C "$RELEASE_PUBLISH_ROOT" rev-parse --abbrev-ref HEAD 2>/dev/null || true)"
    if [ -n "$src_sha" ]; then
        if [ -n "$(git -C "$RELEASE_PUBLISH_ROOT" status --porcelain -- . ':(exclude)release' 2>/dev/null || true)" ]; then
            src_dirty="true"
        fi
    fi
    built_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

    local REL_DIR="$RELEASE_PUBLISH_ROOT/release"
    local TMP_DIR="$REL_DIR/.tmp/$version"
    mkdir -p "$REL_DIR" "$TMP_DIR"

    # Collect valid built binaries from dist_dir.
    local found=0
    local f
    for f in "$dist_dir"/rustcode-"$version"-* "$dist_dir"/rustcode-daemon-"$version"-*; do
        [ -e "$f" ] || continue
        [ -f "$f" ] || continue
        local sz; sz="$(bin_size "$f")"
        if [ "${sz:-0}" -le 0 ] || is_html "$f"; then
            echo "[publish] skipping invalid binary: $(basename "$f") (size=$sz)" >&2
            continue
        fi
        local tag; tag="$(detect_target_tag "$(basename "$f")")"
        if [ -z "$tag" ]; then
            echo "[publish] skipping unrecognized name: $(basename "$f")" >&2
            continue
        fi
        cp "$f" "$TMP_DIR/$(basename "$f")"
        echo "[publish] + $(basename "$f") ($tag)"
        found=1
    done

    if [ "$found" -eq 0 ]; then
        rm -rf "$TMP_DIR"
        echo "[publish] no valid binaries found in $dist_dir; nothing to publish." >&2
        return 1
    fi

    # Merge temp binaries into the real version dir (atomic per-file rename).
    mkdir -p "$REL_DIR/$version"
    local bf
    for bf in "$TMP_DIR"/*; do
        [ -e "$bf" ] || continue
        mv "$bf" "$REL_DIR/$version/$(basename "$bf")"
    done
    rm -rf "$TMP_DIR"

    # Regenerate this version's manifest.json (over whatever was there).
    (
        cd "$REL_DIR/$version"
        local man="manifest.json"
        {
            printf '{\n'
            printf '  "version": "%s",\n' "$version"
            printf '  "ref": "%s",\n' "$ref"
            printf '  "source": {\n'
            printf '    "sha": "%s",\n' "$src_sha"
            printf '    "branch": "%s",\n' "$src_branch"
            printf '    "dirty": %s,\n' "$src_dirty"
            printf '    "built_at": "%s"\n' "$built_at"
            printf '  },\n'
            printf '  "binaries": {\n'
            local first=1
            local bin
            for bin in rustcode-* rustcode-daemon-*; do
                [ -e "$bin" ] || continue
                [ -f "$bin" ] || continue
                local tag; tag="$(detect_target_tag "$bin")"
                [ -n "$tag" ] || continue
                local s size
                s="$(bin_sha256 "$bin")"
                size="$(bin_size "$bin")"
                if [ "$first" -eq 0 ]; then printf ',\n'; fi
                printf '    "%s": { "file": "%s", "sha256": "%s", "size": %s }' "$tag" "$bin" "$s" "$size"
                first=0
            done
            printf '\n  }\n'
            printf '}\n'
        } > "$man.tmp"
        mv "$man.tmp" "$man"
    )

    # Regenerate top-level index.json (sorted newest-first).
    (
        cd "$REL_DIR"
        local versions_list=()
        local d
        for d in v[0-9]*.[0-9]*.[0-9]*; do
            [ -d "$d" ] || continue
            [ -f "$d/manifest.json" ] || continue
            # A directory carrying ONLY manifest.json (no binary was ever
            # committed) must not be indexed. install.sh turns every index
            # entry into a raw fallback URL, so indexing it advertises URLs
            # that all 404 -- a fallback that only looks available. Observed:
            # release/v5.1.0 declared 6 targets with zero committed binaries.
            local has_bin=0 b
            for b in "$d"/rustcode-*; do
                [ -f "$b" ] || continue
                has_bin=1
                break
            done
            [ "$has_bin" -eq 1 ] || continue
            versions_list+=("$d")
        done
        # sort descending
        local sorted=()
        local v
        for v in "${versions_list[@]}"; do
            local insert=1
            local i
            for i in "${!sorted[@]}"; do
                if ver_gt "$v" "${sorted[$i]}"; then
                    sorted=("${sorted[@]:0:$i}" "$v" "${sorted[@]:$i}")
                    insert=0
                    break
                fi
            done
            if [ "$insert" -eq 1 ]; then sorted+=("$v"); fi
        done

        {
            printf '{\n'
            printf '  "updated_at": "%s",\n' "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
            printf '  "ref": "%s",\n' "$ref"
            printf '  "versions": [\n'
            local first=1
            for v in "${sorted[@]}"; do
                # extract targets + released_at from manifest
                local targets
                targets="$(grep -oE '"[a-z]+-(arm64|x64)":' "$v/manifest.json" 2>/dev/null \
                    | sed -E 's/[":]//g' | paste -sd',' - || true)"
                targets="${targets//,/\",\"}"
                if [ "$first" -eq 0 ]; then printf ',\n'; fi
                printf '    { "version": "%s", "targets": ["%s"] }' "$v" "$targets"
                first=0
            done
            printf '\n  ]\n'
            printf '}\n'
        } > index.json.tmp
        mv index.json.tmp index.json
    )

    # Refresh the root self-update manifest (see update_latest_json's header for
    # why this must happen on every publish, not just the full-matrix script).
    # Opt out with RUSTCODE_PUBLISH_NO_LATEST=1 (e.g. publishing an old/partial
    # version must never move the pointer backwards).
    if [ "${RUSTCODE_PUBLISH_NO_LATEST:-0}" = "1" ]; then
        echo "[publish] latest.json refresh skipped (RUSTCODE_PUBLISH_NO_LATEST=1)"
    elif ! update_latest_json "$version" "$REL_DIR"; then
        return 1
    fi

    echo "[publish] release/$version published; index updated."
    return 0
}

# ---------- self test ----------
self_test() {
    local tmp
    tmp="$(mktemp -d)"
    local fake_dist="$tmp/dist"
    local old_root="$RELEASE_PUBLISH_ROOT"
    RELEASE_PUBLISH_ROOT="$tmp/repo"
    mkdir -p "$RELEASE_PUBLISH_ROOT" "$fake_dist"

    # Fake binaries (real ELF-ish bytes; not HTML, non-empty).
    printf 'ELF\x01\x00\x00\x00' > "$fake_dist/rustcode-v1.0.0-linux-x64"
    printf 'ELF\x01\x00\x00\x00' > "$fake_dist/rustcode-daemon-v1.0.0-linux-x64"
    # FreeBSD must be recognized as a valid target (no .exe, ELF-like).
    printf 'ELF\x01\x00\x00\x00' > "$fake_dist/rustcode-v1.0.0-freebsd-x64"

    publish_release "$fake_dist" "v1.0.0" "dev" || { echo "SELF-TEST FAIL: publish v1.0.0"; return 1; }

    # A second, partial publish (only windows) must MERGE, not clobber linux.
    local fake_dist2="$tmp/dist2"
    mkdir -p "$fake_dist2"
    printf 'MZ\x00' > "$fake_dist2/rustcode-v1.0.0-windows-x64.exe"
    publish_release "$fake_dist2" "v1.0.0" "dev" || { echo "SELF-TEST FAIL: merge publish"; return 1; }

    [ -f "$RELEASE_PUBLISH_ROOT/release/v1.0.0/rustcode-v1.0.0-linux-x64" ] || { echo "SELF-TEST FAIL: linux clobbered"; return 1; }
    [ -f "$RELEASE_PUBLISH_ROOT/release/v1.0.0/rustcode-v1.0.0-windows-x64.exe" ] || { echo "SELF-TEST FAIL: windows missing"; return 1; }
    [ -f "$RELEASE_PUBLISH_ROOT/release/v1.0.0/rustcode-v1.0.0-freebsd-x64" ] || { echo "SELF-TEST FAIL: freebsd not published"; return 1; }

    # Reject an invalid (HTML) binary — must not be published.
    local fake_dist3="$tmp/dist3"
    mkdir -p "$fake_dist3"
    printf '<html>404</html>' > "$fake_dist3/rustcode-v1.0.0-darwin-arm64"
    if publish_release "$fake_dist3" "v1.0.0" "dev" 2>/dev/null; then
        [ -f "$RELEASE_PUBLISH_ROOT/release/v1.0.0/rustcode-v1.0.0-darwin-arm64" ] && { echo "SELF-TEST FAIL: html published"; return 1; }
    fi

    # Version ordering in index.json.
    local fake_dist4="$tmp/dist4"
    mkdir -p "$fake_dist4"
    printf 'ELF' > "$fake_dist4/rustcode-v1.9.0-linux-x64"
    printf 'ELF' > "$fake_dist4/rustcode-v1.10.0-linux-x64"
    publish_release "$fake_dist4" "v1.9.0" "dev"
    publish_release "$fake_dist4" "v1.10.0" "dev"
    local top
    top="$(grep -oE '"version": "v[0-9.]+"' "$RELEASE_PUBLISH_ROOT/release/index.json" | head -1 | grep -oE 'v[0-9.]+')"
    [ "$top" = "v1.10.0" ] || { echo "SELF-TEST FAIL: order wrong (got $top)"; return 1; }

    # No duplicate version entries in index.json (regression: explicit arg + glob).
    local dups
    dups="$(grep -oE '"version": "v[0-9.]+"' "$RELEASE_PUBLISH_ROOT/release/index.json" | sort | uniq -d)"
    [ -z "$dups" ] || { echo "SELF-TEST FAIL: duplicate index entries: $dups"; return 1; }

    # A version dir holding only manifest.json (no committed binary) must never
    # be indexed: install.sh turns each index entry into raw fallback URLs, so
    # indexing it advertises URLs that all 404.
    mkdir -p "$RELEASE_PUBLISH_ROOT/release/v0.9.0"
    printf '{\n  "version": "v0.9.0",\n  "binaries": {}\n}\n' > "$RELEASE_PUBLISH_ROOT/release/v0.9.0/manifest.json"
    publish_release "$fake_dist4" "v1.10.0" "dev" >/dev/null 2>&1
    if grep -q '"v0.9.0"' "$RELEASE_PUBLISH_ROOT/release/index.json"; then
        echo "SELF-TEST FAIL: binary-less v0.9.0 got indexed"; return 1
    fi

    # latest.json must have been refreshed to the newest published version.
    # Regression guard: this manifest used to be written only by the full-matrix
    # script and silently froze while releases kept shipping.
    [ -f "$RELEASE_PUBLISH_ROOT/latest.json" ] || { echo "SELF-TEST FAIL: latest.json not written"; return 1; }
    local lj_ver
    lj_ver="$(grep -oE '"version": "v[0-9.]+"' "$RELEASE_PUBLISH_ROOT/latest.json" | head -1 | grep -oE 'v[0-9.]+')"
    [ "$lj_ver" = "v1.10.0" ] || { echo "SELF-TEST FAIL: latest.json is $lj_ver, want v1.10.0"; return 1; }
    # ...and it must carry the sha256 of the binary it points at.
    grep -q '"sha256": "[0-9a-f]\{64\}"' "$RELEASE_PUBLISH_ROOT/latest.json" \
        || { echo "SELF-TEST FAIL: latest.json has no sha256"; return 1; }

    # Publishing an OLD version must not move the pointer backwards.
    RUSTCODE_PUBLISH_NO_LATEST=1 publish_release "$fake_dist4" "v1.9.0" "dev" >/dev/null 2>&1
    lj_ver="$(grep -oE '"version": "v[0-9.]+"' "$RELEASE_PUBLISH_ROOT/latest.json" | head -1 | grep -oE 'v[0-9.]+')"
    [ "$lj_ver" = "v1.10.0" ] || { echo "SELF-TEST FAIL: NO_LATEST moved pointer to $lj_ver"; return 1; }

    echo "SELF-TEST OK"
    rm -rf "$tmp"
    RELEASE_PUBLISH_ROOT="$old_root"
    return 0
}

if [ "${1:-}" = "--self-test" ]; then
    self_test
    exit $?
fi

if [ $# -lt 2 ]; then
    echo "Usage: $0 <dist_dir> <version> [ref]" >&2
    echo "       $0 --self-test" >&2
    exit 1
fi

publish_release "$1" "$2" "${3:-}"
