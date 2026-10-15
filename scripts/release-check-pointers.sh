#!/usr/bin/env bash
# scripts/release-check-pointers.sh — assert that the release VERSION POINTERS
# agree with the artifacts that actually exist.
#
# Why this exists
# ---------------
# There are three independent "what is the current version" pointers, and they
# drifted apart silently:
#
#   Cargo.toml [workspace.package].version   -> 6.2.3
#   release/index.json (top entry)           -> v6.2.1   (repo raw fallback)
#   latest.json (self-update manifest)       -> v6.2.1   (TUI `/upgrade`)
#
# ...while the online Release for v6.2.3 carried all 8 platform binaries. The
# consequences were real, not cosmetic:
#   * the repo-backed fallback (install.sh layer 2) had no v6.2.3 at all, so a
#     GitCode outage silently downgraded users to v6.2.1;
#   * `/upgrade` pointed at v6.2.1, whose online Release has NO attached
#     binaries (only source archives) — a self-update that cannot succeed;
#   * scripts/release-publish.sh never wrote latest.json, so the pointer could
#     only ever be refreshed by scripts/release.sh, which cannot finish on a
#     Linux host (its macOS cross steps fail). That is the structural root
#     cause: the only writer could not run.
#
# This script turns that class of drift into a hard, local, offline-checkable
# failure so it is caught at pre-push / CI time instead of by a user.
#
# Checks (all offline unless --online is passed)
#   P1 Cargo.toml yields a vX.Y.Z version
#   P2 release/index.json exists and its TOP version equals that version
#   P3 release/<version>/manifest.json exists
#   P4 for every version in index.json: the declared `targets` set equals the
#      `binaries` key set of its manifest (no phantom, no missing)
#   P5 every `file` named by a manifest actually exists in the tree being
#      checked (a declared-but-uncommitted binary is a broken fallback URL)
#   P6 latest.json names the same version and every entry carries a real
#      64-hex sha256 + non-zero size
#   P7 (--online) the newest GitCode release that actually has attached
#      binaries is the same version
#
# Usage:
#   scripts/release-check-pointers.sh [--root DIR] [--sha SHA] [--online]
#   scripts/release-check-pointers.sh --self-test
#
#   --root DIR   repository root (default: parent of this script)
#   --sha SHA    read every file from that commit's tree instead of the
#                working tree (used by the pre-push gate, which must judge the
#                commit that is about to land, not whatever is on disk)
#   --online     additionally query the GitCode releases API (needs network +
#                python3; skipped with a warning when unavailable)

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SHA=""
ONLINE=0

# ---------- helpers ----------

# readf <repo-relative-path> -> contents on stdout (empty when absent)
readf() {
    if [ -n "$SHA" ]; then
        git -C "$ROOT" show "$SHA:$1" 2>/dev/null
    elif [ -f "$ROOT/$1" ]; then
        cat "$ROOT/$1"
    fi
}

# exists_f <repo-relative-path> -> 0 when present in the checked tree
exists_f() {
    if [ -n "$SHA" ]; then
        git -C "$ROOT" cat-file -e "$SHA:$1" 2>/dev/null
    else
        [ -f "$ROOT/$1" ]
    fi
}

# Workspace version from a Cargo.toml on stdin; empty when absent.
version_of() {
    awk '
        /^[[:space:]]*\[(workspace\.)?package\][[:space:]]*$/ { inpkg = 1; next }
        /^[[:space:]]*\[/ { inpkg = 0 }
        inpkg && $1 == "version" {
            v = $3
            gsub(/[";]/, "", v)
            print v
            exit
        }
    '
}

# "X Y Z" for vX.Y.Z (missing fields -> 0)
ver_split() {
    local v="${1#v}"
    printf '%s %s %s' "${v%%.*}" "$(printf '%s' "$v" | cut -d. -f2)" "$(printf '%s' "$v" | cut -d. -f3)"
}

# ver_gt A B -> 0 when A > B
ver_gt() {
    local a1 a2 a3 b1 b2 b3
    read -r a1 a2 a3 <<<"$(ver_split "$1")"
    read -r b1 b2 b3 <<<"$(ver_split "$2")"
    a1="${a1:-0}"; a2="${a2:-0}"; a3="${a3:-0}"
    b1="${b1:-0}"; b2="${b2:-0}"; b3="${b3:-0}"
    [ "$a1" -gt "$b1" ] && return 0
    [ "$a1" -lt "$b1" ] && return 1
    [ "$a2" -gt "$b2" ] && return 0
    [ "$a2" -lt "$b2" ] && return 1
    [ "$a3" -gt "$b3" ]
}

# index_versions -> one "vX.Y.Z<TAB>target,target" per line
index_rows() {
    printf '%s\n' "$1" | sed -n 's/.*"version"[[:space:]]*:[[:space:]]*"\(v[0-9][0-9.]*\)".*"targets"[[:space:]]*:[[:space:]]*\[\([^]]*\)\].*/\1\t\2/p'
}

# manifest_targets <manifest-text> -> sorted target list, one per line
manifest_targets() {
    printf '%s\n' "$1" \
        | sed -n 's/^[[:space:]]*"\([a-z][a-z0-9]*-\(arm64\|x64\)\)":[[:space:]]*{.*/\1/p' \
        | sort
}

# manifest_files <manifest-text> -> "target<TAB>file" per line
manifest_files() {
    printf '%s\n' "$1" \
        | sed -n 's/^[[:space:]]*"\([a-z][a-z0-9]*-\(arm64\|x64\)\)":[[:space:]]*{.*"file"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1\t\3/p'
}

norm_list() { # "a","b" -> sorted "a b"
    printf '%s\n' "$1" | tr -d '"' | tr ',' '\n' | sed 's/^[[:space:]]*//; s/[[:space:]]*$//' | grep -v '^$' | sort
}

# newest GitCode release that actually carries attached binaries; "" when unknown
online_top_attached() {
    local api="${RUSTCODE_POINTERS_API:-https://api.gitcode.com/api/v5/repos/SecLab/RustCode/releases}"
    if ! command -v python3 >/dev/null 2>&1 || ! command -v curl >/dev/null 2>&1; then
        return 0
    fi
    curl -fsS --max-time 30 "$api" 2>/dev/null | python3 -c '
import json, re, sys
try:
    data = json.load(sys.stdin)
except Exception:
    sys.exit(0)
if not isinstance(data, list):
    sys.exit(0)
def key(tag):
    m = re.match(r"^v?(\d+)\.(\d+)\.(\d+)$", tag or "")
    return tuple(int(x) for x in m.groups()) if m else None
best = None
for rel in data:
    assets = [a for a in (rel.get("assets") or []) if a.get("type") == "attach"]
    if not assets:
        continue
    k = key(rel.get("tag_name"))
    if k and (best is None or k > best[0]):
        best = (k, rel.get("tag_name"))
print(best[1] if best else "")
' 2>/dev/null
}

err() { echo "[ERROR] version pointers: $*" >&2; }

# ---------- the gate ----------

check_pointers() {
    local root="$1" sha="$2" online="$3"
    ROOT="$root"; SHA="$sha"

    local rc=0

    # P1 -- version from the tree being checked
    local cargo version
    cargo="$(readf Cargo.toml)"
    version="v$(printf '%s\n' "$cargo" | version_of)"
    if [ "$version" = "v" ]; then
        err "cannot read [workspace.package].version from Cargo.toml${sha:+ at ${sha:0:12}}"
        return 1
    fi

    # P2 -- index.json exists and is not behind the version
    local index
    index="$(readf release/index.json)"
    if [ -z "$index" ]; then
        err "release/index.json is missing; run scripts/release-publish.sh <dist> $version"
        return 1
    fi
    local top=""
    local v
    while read -r v _; do
        [ -n "$v" ] || continue
        if [ -z "$top" ] || ver_gt "$v" "$top"; then top="$v"; fi
    done <<EOF
$(index_rows "$index")
EOF
    if [ -z "$top" ]; then
        err "release/index.json lists no versions"
        return 1
    fi
    if [ "$top" != "$version" ]; then
        err "Cargo.toml says $version but release/index.json tops out at $top"
        err "  the repo-backed fallback (install.sh layer 2) and every raw URL"
        err "  for $version will 404 -> users silently get $top instead."
        err "  fix: scripts/release-publish.sh <dist> $version"
        return 1
    fi

    # P3 -- this version has a manifest
    if ! exists_f "release/$version/manifest.json"; then
        err "release/$version/manifest.json is missing (not committed?)"
        return 1
    fi

    # P4/P5 -- every indexed version: targets match the manifest, files exist
    local row targets man want_t have_t missing phantom
    while IFS="$(printf '\t')" read -r row targets; do
        [ -n "$row" ] || continue
        if ! exists_f "release/$row/manifest.json"; then
            err "index.json lists $row but release/$row/manifest.json is absent"
            rc=1
            continue
        fi
        man="$(readf "release/$row/manifest.json")"
        want_t="$(norm_list "$targets")"
        have_t="$(manifest_targets "$man" | tr '\n' ' ')"
        have_t="$(printf '%s\n' "$have_t" | tr ' ' '\n' | sed 's/^[[:space:]]*//' | grep -v '^$' | sort | tr '\n' ' ')"

        missing="$(comm -23 <(printf '%s\n' "$want_t" | tr ' ' '\n' | grep -v '^$' | sort) \
                            <(printf '%s\n' "$have_t" | tr ' ' '\n' | grep -v '^$' | sort))"
        phantom="$(comm -13 <(printf '%s\n' "$want_t" | tr ' ' '\n' | grep -v '^$' | sort) \
                            <(printf '%s\n' "$have_t" | tr ' ' '\n' | grep -v '^$' | sort))"
        if [ -n "$missing" ]; then
            err "$row: index.json declares $(printf '%s' "$missing" | tr '\n' ' ') but the manifest has no such binary"
            rc=1
        fi
        if [ -n "$phantom" ]; then
            err "$row: manifest has $(printf '%s' "$phantom" | tr '\n' ' ') but index.json does not declare it"
            rc=1
        fi

        # P5: each declared binary must really be in the tree
        local t f
        while IFS="$(printf '\t')" read -r t f; do
            [ -n "$t" ] || continue
            if ! exists_f "release/$row/$f"; then
                err "$row/$t: manifest names '$f' but release/$row/$f is not in the tree (uncommitted?)"
                rc=1
            fi
        done <<EOF
$(manifest_files "$man")
EOF
    done <<EOF
$(index_rows "$index")
EOF

    # P6 -- latest.json agrees and is usable
    local latest lj_ver
    latest="$(readf latest.json)"
    if [ -z "$latest" ]; then
        err "latest.json is missing; /upgrade has no manifest to read"
        return 1
    fi
    lj_ver="$(printf '%s\n' "$latest" | sed -n 's/.*"version"[[:space:]]*:[[:space:]]*"\(v[0-9][0-9.]*\)".*/\1/p' | head -1)"
    if [ "$lj_ver" != "$version" ]; then
        err "latest.json says ${lj_ver:-<none>} but the tree is $version"
        err "  /upgrade would offer ${lj_ver:-nothing} even though $version shipped."
        err "  fix: scripts/release-publish.sh <dist> $version   (it refreshes latest.json)"
        rc=1
    fi
    # every entry must be a usable pointer: 64-hex sha256 + non-zero size
    local bad
    bad="$(printf '%s\n' "$latest" \
        | sed -n 's/^[[:space:]]*"\([a-z][a-z0-9]*-\(arm64\|x64\)\)":[[:space:]]*{.*/\1/p' \
        | while read -r t; do
              line="$(printf '%s\n' "$latest" | grep "\"$t\"" | head -1)"
              printf '%s\n' "$line" | grep -q '"sha256"[[:space:]]*:[[:space:]]*"[0-9a-fA-F]\{64\}"' || { echo "$t(sha256)"; continue; }
              printf '%s\n' "$line" | grep -q '"size"[[:space:]]*:[[:space:]]*[1-9][0-9]*' || echo "$t(size)"
          done)"
    if [ -n "$bad" ]; then
        err "latest.json has unusable entries: $(printf '%s' "$bad" | tr '\n' ' ')"
        rc=1
    fi

    # P7 -- online reality check (opt-in: needs network)
    if [ "$online" = "1" ]; then
        local otop
        otop="$(online_top_attached)"
        if [ -z "$otop" ]; then
            echo "[warn] version pointers: online check unavailable (no network/python3); skipped" >&2
        elif [ "$otop" != "$version" ]; then
            err "GitCode's newest release WITH binaries is $otop, but this tree is $version"
            err "  either publish $version's binaries online, or bump the tree to $otop"
            rc=1
        else
            echo "[OK] version pointers: online release $otop matches this tree"
        fi
    fi

    if [ "$rc" -eq 0 ]; then
        echo "[OK] version pointers: Cargo/index/latest all agree on $version${sha:+ at ${sha:0:12}}"
    fi
    return "$rc"
}

# ---------- self test ----------

self_test() {
    local tmp rc=0
    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' RETURN

    # _repo <dir> <version> : minimal repo whose release pointers are consistent
    _repo() {
        local d="$1" v="$2"
        mkdir -p "$d/release/$v"
        printf '[workspace]\n[workspace.package]\nversion = "%s"\n' "${v#v}" > "$d/Cargo.toml"
        printf 'ELF' > "$d/release/$v/rustcode-$v-linux-x64"
        cat > "$d/release/$v/manifest.json" <<EOF
{
  "version": "$v",
  "ref": "dev",
  "binaries": {
    "linux-x64": { "file": "rustcode-$v-linux-x64", "sha256": "$(printf '%064d' 0)", "size": 3 }
  }
}
EOF
        cat > "$d/release/index.json" <<EOF
{
  "updated_at": "2026-10-15T00:00:00Z",
  "ref": "dev",
  "versions": [
    { "version": "$v", "targets": ["linux-x64"] }
  ]
}
EOF
        cat > "$d/latest.json" <<EOF
{
  "version": "$v",
  "released_at": "2026-10-15T00:00:00Z",
  "binaries": {
    "linux-x64": { "sha256": "$(printf '%064d' 0)", "size": 3 }
  }
}
EOF
    }

    _ok()   { check_pointers "$1" "" 0 >/dev/null 2>&1 || { echo "SELF-TEST FAIL: $2"; rc=1; }; }
    _bad()  { check_pointers "$1" "" 0 >/dev/null 2>&1 && { echo "SELF-TEST FAIL: $2"; rc=1; }; }

    # 1. consistent tree passes
    _repo "$tmp/ok" "v6.2.3"
    _ok "$tmp/ok" "consistent tree must pass"

    # 2. latest.json behind Cargo (the v6.2.1-vs-v6.2.3 failure) must fail
    _repo "$tmp/stale-latest" "v6.2.3"
    sed -i.bak 's/"version": "v6.2.3"/"version": "v6.2.1"/' "$tmp/stale-latest/latest.json"
    rm -f "$tmp/stale-latest/latest.json.bak"
    _bad "$tmp/stale-latest" "stale latest.json must fail"

    # 3. index.json behind Cargo (repo fallback missing the version) must fail
    _repo "$tmp/stale-index" "v6.2.3"
    sed -i.bak 's/"version": "v6.2.3"/"version": "v6.2.1"/' "$tmp/stale-index/release/index.json"
    rm -f "$tmp/stale-index/release/index.json.bak"
    _bad "$tmp/stale-index" "index.json behind Cargo.toml must fail"

    # 4. index declares a target the manifest lacks (phantom fallback URL)
    _repo "$tmp/phantom" "v6.2.3"
    sed -i.bak 's/"targets": \["linux-x64"\]/"targets": ["linux-x64","darwin-arm64"]/' "$tmp/phantom/release/index.json"
    rm -f "$tmp/phantom/release/index.json.bak"
    _bad "$tmp/phantom" "phantom index target must fail"

    # 5. manifest names a binary that is not in the tree (uncommitted artifact)
    _repo "$tmp/missing-bin" "v6.2.3"
    rm -f "$tmp/missing-bin/release/v6.2.3/rustcode-v6.2.3-linux-x64"
    _bad "$tmp/missing-bin" "declared-but-missing binary must fail"

    # 6. latest.json entry without a real sha256
    _repo "$tmp/bad-sha" "v6.2.3"
    sed -i.bak 's/"sha256": "0*", "size": 3/"sha256": "deadbeef", "size": 3/' "$tmp/bad-sha/latest.json"
    rm -f "$tmp/bad-sha/latest.json.bak"
    _bad "$tmp/bad-sha" "latest.json with a fake sha256 must fail"

    # 7. --sha mode: judge the COMMIT, not the working tree
    _repo "$tmp/shamode" "v6.2.3"
    (
        cd "$tmp/shamode" || exit 1
        git init -q .
        git config user.email t@example.com
        git config user.name t
        git add -A
        git commit -qm base
    )
    local sh
    sh="$(git -C "$tmp/shamode" rev-parse HEAD)"
    # working tree drifts to a broken state; the committed tree is still good
    sed -i.bak 's/"version": "v6.2.3"/"version": "v1.0.0"/' "$tmp/shamode/latest.json"
    rm -f "$tmp/shamode/latest.json.bak"
    check_pointers "$tmp/shamode" "$sh" 0 >/dev/null 2>&1 \
        || { echo "SELF-TEST FAIL: --sha must judge the commit, not the dirty tree"; rc=1; }
    check_pointers "$tmp/shamode" "" 0 >/dev/null 2>&1 \
        && { echo "SELF-TEST FAIL: dirty working tree must fail without --sha"; rc=1; }

    [ "$rc" -eq 0 ] && echo "SELF-TEST OK"
    return "$rc"
}

# ---------- entry point ----------

while [ $# -gt 0 ]; do
    case "$1" in
        --root)   ROOT="$2"; shift 2 ;;
        --sha)    SHA="$2"; shift 2 ;;
        --online) ONLINE=1; shift ;;
        --self-test) self_test; exit $? ;;
        -h|--help)
            sed -n '2,40p' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *) echo "unknown arg: $1" >&2; exit 2 ;;
    esac
done

check_pointers "$ROOT" "$SHA" "$ONLINE"
