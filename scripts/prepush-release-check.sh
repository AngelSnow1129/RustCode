#!/usr/bin/env bash
# scripts/prepush-release-check.sh -- pre-push gate for release artifacts.
#
# Policy (user requirement): a push must carry a BUILD-PASSING artifact that is
# already COMMITTED under release/<version>/. The gate does NOT build anything:
# building inside a pre-push hook would block every push for minutes and can OOM
# a small container. It only *verifies* what the developer committed, and prints
# the exact commands needed to fix a failure.
#
# Two strengths:
#   default (on)
#       The pushed commit must contain release/<version>/manifest.json plus a
#       binary for THIS host's OS/ARCH, where <version> is read from the pushed
#       Cargo.toml ([workspace.package].version). Bumping the version therefore
#       forces a fresh, committed artifact; iterating inside one version does
#       not. Affordable: one ~40 MB artifact per release, not per push.
#   strict
#       Additionally requires the artifact to have been built from this exact
#       code. Checked as: manifest.source.sha is an ancestor of the pushed
#       commit, the only changes in between are under release/, and
#       manifest.source.dirty == false.
#
#       Note: source.sha can never EQUAL the pushed sha. The manifest is written
#       at build time and the artifact is committed afterwards, so the manifest
#       would have to contain its own commit hash. "Ancestor + no source drift in
#       between" is the strongest predicate that is actually satisfiable.
#
# Env:
#   RUSTCODE_PREPUSH_RELEASE=on|strict|off    (default: on)
#
# Usage:
#   scripts/prepush-release-check.sh [--sha <sha>] [--root <dir>] [--target <os-arch>]
#   scripts/prepush-release-check.sh --self-test
#
# Exit: 0 = gate satisfied (push may proceed), 1 = gate failed (block the push).

set -euo pipefail

SELF_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

usage() {
    cat <<'EOF'
Usage: prepush-release-check.sh [options]

  --sha <sha>        commit being pushed (default: HEAD of --root)
  --root <dir>       repository root (default: the repo containing this script)
  --target <tag>     override host OS/ARCH tag, e.g. linux-x64; 'any' accepts any
                     one committed binary of this version (used by CI)
                     (default: the host's own tag)
  --self-test        run the built-in regression suite and exit
  -h, --help         show this help

Env: RUSTCODE_PREPUSH_RELEASE=on|strict|off (default: on)
EOF
}

# ---------- pure helpers ----------

# echo the "vX.Y.Z" version from a Cargo.toml body on stdin ([workspace.package].version)
version_of() {
    awk -F'"' '
        /^\[workspace\.package\]/ { f = 1; next }
        /^\[/                     { f = 0 }
        f && /^version *=/        { print $2; exit }
    '
}

# echo "<os>-<arch>" for the current host, or nothing when unsupported.
host_target_tag() {
    local os arch
    case "$(uname -s)" in
        Darwin)                   os=darwin ;;
        Linux)                    os=linux ;;
        MINGW* | MSYS* | CYGWIN*) os=windows ;;
        *) return 0 ;;
    esac
    case "$(uname -m)" in
        x86_64 | amd64)  arch=x64 ;;
        arm64 | aarch64) arch=arm64 ;;
        *) return 0 ;;
    esac
    printf '%s-%s' "$os" "$arch"
}

# echo the binary filename recorded for <tag> in a manifest body on stdin.
# Relies on release-publish.sh emitting one binary entry per line; a compact
# single-line manifest parses the same way because the match is line-scoped.
manifest_file_for_tag() {
    local tag="$1"
    grep -F "\"$tag\":" | sed -n 's/.*"file": *"\([^"]*\)".*/\1/p' | head -1
}

# echo the first binary filename in a manifest body on stdin (used by --target any).
manifest_any_file() {
    sed -n 's/.*"file": *"\([^"]*\)".*/\1/p' | head -1
}

# echo a top-level "source" sub-field from a manifest body on stdin.
# `sha` must not be confused with the per-binary `sha256` field.
manifest_source_field() {
    case "$1" in
        sha) sed -n 's/.*"sha": *"\([^"]*\)".*/\1/p' | head -1 ;;
        dirty) sed -n 's/.*"dirty": *\(true\|false\).*/\1/p' | head -1 ;;
        *) return 1 ;;
    esac
}

howto() {
    cat >&2 <<'EOF'

  Fix it, then push again:
    scripts/release-host.sh          # build for this host + publish into release/
    git add release/                 # stage the artifacts
    git commit -m "chore(release): publish <version> artifacts"

  Switches:
    RUSTCODE_PREPUSH_RELEASE=strict  also require the artifact to come from this exact code
    RUSTCODE_PREPUSH_RELEASE=off     skip this gate entirely
    RUSTCODE_PUBLISH_COMMIT=1        let release-host.sh stage + commit release/ for you

  To bypass once (NOT recommended): git push --no-verify
EOF
}

fail_no_manifest() {
    echo "[ERROR] release gate: $1 is not committed at ${2:0:12}." >&2
    echo "        The pushed commit has no $3/ -- a version bump must ship a committed artifact." >&2
    howto
}

fail_no_target() {
    echo "[ERROR] release gate: $1 has no artifact for this host ($2)." >&2
    echo "        $3/manifest.json was found, but it lists no $2 target." >&2
    echo "        Build on this host so the artifact matches the machine you push from." >&2
    howto
}

fail_binary_missing() {
    echo "[ERROR] release gate: $1 names '$3' for $2, but that file is not committed." >&2
    echo "        The manifest is committed while the binary is not (untracked or ignored)." >&2
    howto
}

fail_stale() {
    echo "[ERROR] release gate (strict): the $2 artifact is not built from this code." >&2
    echo "        pushed commit : ${1:0:12}" >&2
    echo "        built from    : ${3:0:12}" >&2
    echo "        'built from' is not an ancestor of the pushed commit." >&2
    howto
}

fail_source_drift() {
    echo "[ERROR] release gate (strict): source changed after the $2 artifact was built." >&2
    echo "        pushed commit : ${1:0:12}" >&2
    echo "        built from    : ${3:0:12}" >&2
    echo "        Source files changed in between (only release/ may):" >&2
    printf '%s\n' "$4" | sed 's/^/          /' >&2
    echo "        Rebuild so the artifact proves that THIS commit compiles." >&2
    howto
}

fail_dirty() {
    echo "[ERROR] release gate (strict): the $1 artifact was built from a dirty tree." >&2
    echo "        Uncommitted source changes make the artifact non-reproducible," >&2
    echo "        so it cannot prove that the pushed commit compiles." >&2
    howto
}

# ---------- the gate ----------

# check_gate <root> <sha> <target-tag> <on|strict>
check_gate() {
    local root="$1" sha="$2" tag="$3" mode="$4"

    # Version comes from the PUSHED Cargo.toml, not the working tree: the gate
    # must judge the commit that is about to land, not whatever is on disk now.
    local cargo version
    if ! cargo="$(git -C "$root" show "$sha:Cargo.toml" 2>/dev/null)"; then
        echo "[ERROR] release gate: cannot read Cargo.toml at ${sha:0:12}." >&2
        return 1
    fi
    version="v$(printf '%s\n' "$cargo" | version_of)"
    if [ "$version" = "v" ]; then
        echo "[ERROR] release gate: ${sha:0:12} has no [workspace.package].version." >&2
        return 1
    fi

    local rel_dir="release/$version"
    local manifest_path="$rel_dir/manifest.json"
    local manifest
    if ! manifest="$(git -C "$root" show "$sha:$manifest_path" 2>/dev/null)"; then
        fail_no_manifest "$version" "$sha" "$rel_dir"
        return 1
    fi

    local file
    if [ "$tag" = "any" ]; then
        # CI runs on a fixed runner while the pusher's host may differ, so it
        # accepts any ONE committed binary of this version instead of a
        # host-specific one. The local pre-push hook still pins the real host.
        file="$(printf '%s\n' "$manifest" | manifest_any_file)"
    else
        file="$(printf '%s\n' "$manifest" | manifest_file_for_tag "$tag")"
    fi
    if [ -z "$file" ]; then
        fail_no_target "$version" "$tag" "$rel_dir"
        return 1
    fi

    # cat-file -e on "<sha>:<path>" proves the blob exists IN THAT COMMIT's tree;
    # a working-tree file that was never committed fails here -- that is the point.
    if ! git -C "$root" cat-file -e "$sha:$rel_dir/$file" 2>/dev/null; then
        fail_binary_missing "$version" "$tag" "$file"
        return 1
    fi

    if [ "$mode" = "strict" ]; then
        local src_sha src_dirty changed
        src_sha="$(printf '%s\n' "$manifest" | manifest_source_field sha)"
        src_dirty="$(printf '%s\n' "$manifest" | manifest_source_field dirty)"

        if [ -z "$src_sha" ] \
            || ! git -C "$root" merge-base --is-ancestor "$src_sha" "$sha" 2>/dev/null; then
            fail_stale "$sha" "$version" "$src_sha"
            return 1
        fi

        # Only the artifact commits themselves may sit between "built from" and
        # "pushed": any source change in between means the binary no longer
        # proves that the pushed commit compiles.
        changed="$(git -C "$root" diff --name-only "$src_sha" "$sha" -- . ':(exclude)release' 2>/dev/null || true)"
        if [ -n "$changed" ]; then
            fail_source_drift "$sha" "$version" "$src_sha" "$changed"
            return 1
        fi

        if [ "$src_dirty" = "true" ]; then
            fail_dirty "$version"
            return 1
        fi
    fi

    echo "[OK] release gate: $version / $tag artifact is committed at ${sha:0:12}"
    return 0
}

# ---------- self test ----------

self_test() {
    local tmp rc=0
    tmp="$(mktemp -d)"

    _fail() { echo "SELF-TEST FAIL: $1"; rc=1; }

    # _mk_repo <dir> <version> -- init a repo with a Cargo.toml; echo its sha.
    _mk_repo() {
        mkdir -p "$1"
        (
            cd "$1"
            git init -q .
            git config user.email t@example.com
            git config user.name t
            printf '[workspace]\n[workspace.package]\nversion = "%s"\n' "$2" > Cargo.toml
            git add -A
            git commit -qm base
        )
        git -C "$1" rev-parse HEAD
    }

    _commit() {
        (
            cd "$1"
            git add -A
            git commit -qm "$2" --allow-empty
        )
    }

    # _write_artifacts <repo> <src-sha> <dirty>
    _write_artifacts() {
        mkdir -p "$1/release/v1.2.3"
        printf 'ELF\x01\x00\x00\x00' > "$1/release/v1.2.3/rustcode-v1.2.3-linux-x64"
        {
            printf '{\n'
            printf '  "version": "v1.2.3",\n'
            printf '  "ref": "dev",\n'
            printf '  "source": {\n'
            printf '    "sha": "%s",\n' "$2"
            printf '    "branch": "dev",\n'
            printf '    "dirty": %s,\n' "$3"
            printf '    "built_at": "2026-01-01T00:00:00Z"\n'
            printf '  },\n'
            printf '  "binaries": {\n'
            printf '    "linux-x64": { "file": "rustcode-v1.2.3-linux-x64", "sha256": "abc", "size": 8 }\n'
            printf '  }\n'
            printf '}\n'
        } > "$1/release/v1.2.3/manifest.json"
    }

    # 1. committed artifact for the right version+target -> pass in both modes.
    local r1 b1 s1
    r1="$tmp/r1"
    b1="$(_mk_repo "$r1" 1.2.3)"
    _write_artifacts "$r1" "$b1" false
    _commit "$r1" publish
    s1="$(git -C "$r1" rev-parse HEAD)"
    check_gate "$r1" "$s1" linux-x64 on >/dev/null 2>&1 \
        || _fail "committed artifact should pass (default mode)"
    check_gate "$r1" "$s1" linux-x64 strict >/dev/null 2>&1 \
        || _fail "artifact whose only drift is release/ should pass (strict mode)"

    # 2. manifest committed but the binary itself missing -> fail.
    local r2 b2 s2
    r2="$tmp/r2"
    b2="$(_mk_repo "$r2" 1.2.3)"
    _write_artifacts "$r2" "$b2" false
    _commit "$r2" publish
    rm -f "$r2/release/v1.2.3/rustcode-v1.2.3-linux-x64"
    _commit "$r2" drop-binary
    s2="$(git -C "$r2" rev-parse HEAD)"
    if check_gate "$r2" "$s2" linux-x64 on >/dev/null 2>&1; then
        _fail "an uncommitted binary should fail"
    fi

    # 3. version bump with no artifact at all -> fail.
    local r3 s3
    r3="$tmp/r3"
    _mk_repo "$r3" 1.2.3 >/dev/null
    printf '[workspace]\n[workspace.package]\nversion = "2.0.0"\n' > "$r3/Cargo.toml"
    _commit "$r3" bump-without-artifact
    s3="$(git -C "$r3" rev-parse HEAD)"
    if check_gate "$r3" "$s3" linux-x64 on >/dev/null 2>&1; then
        _fail "a version bump without a committed artifact should fail"
    fi

    # 4. source drift after the build -> default passes, strict fails.
    local r4 b4 s4
    r4="$tmp/r4"
    b4="$(_mk_repo "$r4" 1.2.3)"
    _write_artifacts "$r4" "$b4" false
    _commit "$r4" publish
    printf '# touched after the build\n' >> "$r4/Cargo.toml"
    _commit "$r4" source-drift
    s4="$(git -C "$r4" rev-parse HEAD)"
    check_gate "$r4" "$s4" linux-x64 on >/dev/null 2>&1 \
        || _fail "default mode must accept a version-aligned artifact"
    if check_gate "$r4" "$s4" linux-x64 strict >/dev/null 2>&1; then
        _fail "strict mode must reject an artifact built before a source change"
    fi

    # 5. correct provenance but built from a dirty tree -> default passes, strict fails.
    local r5 b5 s5
    r5="$tmp/r5"
    b5="$(_mk_repo "$r5" 1.2.3)"
    _write_artifacts "$r5" "$b5" true
    _commit "$r5" publish-dirty
    s5="$(git -C "$r5" rev-parse HEAD)"
    check_gate "$r5" "$s5" linux-x64 on >/dev/null 2>&1 \
        || _fail "default mode must ignore build-time dirtiness"
    if check_gate "$r5" "$s5" linux-x64 strict >/dev/null 2>&1; then
        _fail "strict mode must reject a dirty-tree artifact"
    fi

    # 6. source.sha must not be confused with the per-binary sha256 field.
    local parsed
    parsed="$(manifest_source_field sha < "$r1/release/v1.2.3/manifest.json")"
    [ "$parsed" = "$b1" ] || _fail "source.sha parse got '$parsed', expected '$b1'"

    # 7. another platform's artifact must not satisfy this host.
    if check_gate "$r1" "$s1" darwin-arm64 on >/dev/null 2>&1; then
        _fail "a foreign target must not satisfy this host"
    fi

    # 8. the gate must also work when invoked against a mid-history commit.
    check_gate "$r1" "$s1" linux-x64 on >/dev/null 2>&1 \
        || _fail "gate should pass when run against the publish commit itself"

    # 9. --target any (CI) accepts a foreign platform, but still demands that the
    #    named binary is committed.
    check_gate "$r1" "$s1" any on >/dev/null 2>&1 \
        || _fail "--target any should accept an existing committed artifact"
    if check_gate "$r2" "$s2" any on >/dev/null 2>&1; then
        _fail "--target any must still reject an uncommitted binary"
    fi

    rm -rf "$tmp"
    if [ "$rc" -eq 0 ]; then
        echo "SELF-TEST OK"
    fi
    return "$rc"
}

# ---------- entry point ----------

main() {
    local sha="" root="$SELF_ROOT" tag=""
    local mode="${RUSTCODE_PREPUSH_RELEASE:-on}"

    while [ $# -gt 0 ]; do
        case "$1" in
            --sha)  sha="${2:-}"; shift 2 ;;
            --root) root="${2:-}"; shift 2 ;;
            --target) tag="${2:-}"; shift 2 ;;
            --self-test) self_test; exit $? ;;
            -h | --help) usage; exit 0 ;;
            *)
                echo "[ERROR] unknown argument: $1" >&2
                usage >&2
                exit 2
                ;;
        esac
    done

    case "$mode" in
        off | 0 | no | false | skip)
            echo "[INFO] release gate skipped (RUSTCODE_PREPUSH_RELEASE=$mode)"
            return 0
            ;;
        strict) ;;
        "" | on | 1 | yes | true) mode=on ;;
        *)
            echo "[WARN] unknown RUSTCODE_PREPUSH_RELEASE='$mode'; falling back to 'on'" >&2
            mode=on
            ;;
    esac

    if ! git -C "$root" rev-parse --git-dir >/dev/null 2>&1; then
        echo "[WARN] release gate: not a git repository ($root); skipped" >&2
        return 0
    fi

    if [ -z "$sha" ]; then
        sha="$(git -C "$root" rev-parse HEAD 2>/dev/null || true)"
    fi

    # Deleting a remote ref carries the all-zero sha: there is nothing to build.
    case "$sha" in
        "" | 0000000000000000000000000000000000000000)
            echo "[INFO] release gate: no commit to check; skipped"
            return 0
            ;;
    esac

    if [ -z "$tag" ]; then
        tag="$(host_target_tag)"
    fi
    if [ -z "$tag" ]; then
        echo "[WARN] release gate: unsupported host OS/ARCH; skipped" >&2
        return 0
    fi

    check_gate "$root" "$sha" "$tag" "$mode"
}

main "$@"
