#!/bin/sh
# RustCode uninstaller — run locally
#
#   sh scripts/uninstall.sh
#
# (Obtain this script from the same distribution channel you installed
#  rustcode from; this build ships no built-in download host.)
# Flags:
#   --yes          skip prompts (use defaults: G1=yes, G2=no, G3=yes)
#   --purge        delete everything including ~/.rustcode/
#   --keep-data    only delete binary + PATH edit
#   --dry-run      print plan, do nothing
#   --print-manifest  emit the path manifest used for parity tests, exit
#
# This is the fallback uninstaller. Prefer `rustcode uninstall` if the
# binary is still working.
set -eu

# ---- manifest (must mirror crates/rustcode-cli/src/uninstall/paths.rs) ----
RUSTCODE_GROUP2_FILES="auth.toml mcp.json config.toml RUSTCODE.md"
RUSTCODE_GROUP3_FILES="history input_history.txt recent_dirs.txt codingplan_sync.json device_id"
RUSTCODE_GROUP3_DIRS="staged telemetry plugins commands skills"
RUSTCODE_GROUP3_PREFIXES="notice."

# ---- emit-manifest mode (used by parity test) ----
if [ "${1:-}" = "--print-manifest" ]; then
    printf 'group2_files=%s\n' "$RUSTCODE_GROUP2_FILES"
    printf 'group3_files=%s\n' "$RUSTCODE_GROUP3_FILES"
    printf 'group3_dirs=%s\n' "$RUSTCODE_GROUP3_DIRS"
    printf 'group3_prefixes=%s\n' "$RUSTCODE_GROUP3_PREFIXES"
    exit 0
fi

# ---- parse flags ----
DO_YES=0; DO_PURGE=0; DO_KEEP=0; DO_DRY=0
for arg in "$@"; do
    case "$arg" in
        --yes) DO_YES=1 ;;
        --purge) DO_PURGE=1 ;;
        --keep-data) DO_KEEP=1 ;;
        --dry-run) DO_DRY=1 ;;
        *) echo "unknown flag: $arg" >&2; exit 2 ;;
    esac
done

if [ "$DO_PURGE" = 1 ] && [ "$DO_KEEP" = 1 ]; then
    echo "--purge and --keep-data conflict" >&2; exit 2
fi

# ---- detect binary ----
BIN=$(command -v rustcode 2>/dev/null || true)
if [ -z "$BIN" ]; then
    # `.exe` variants cover a Windows-shell (MSYS/MinGW/Cygwin) install — install.sh drops
    # rustcode.exe into ~/.local/bin there.
    for c in /usr/local/bin/rustcode "$HOME/.local/bin/rustcode" \
             /usr/local/bin/rustcode.exe "$HOME/.local/bin/rustcode.exe"; do
        [ -e "$c" ] && BIN=$c && break
    done
fi
if [ -z "$BIN" ] && [ -n "${RUSTCODE_BIN:-}" ]; then
    BIN=$RUSTCODE_BIN
fi
if [ -z "$BIN" ]; then
    echo "rustcode binary not found in PATH or default locations." >&2
    echo "If installed elsewhere, pass RUSTCODE_BIN=/path/to/rustcode" >&2
fi
BIN_DIR=$(dirname "$BIN" 2>/dev/null || echo "")

DATA="${RUSTCODE_HOME:-$HOME/.rustcode}"

# ---- plan ----
echo "Will remove (Group 1):"
[ -n "$BIN" ] && echo "  $BIN"
if [ -n "$BIN_DIR" ]; then
    for f in rustcode.bak .rustcode.rolling .rustcode.download .rustcode.writable-probe; do
        [ -e "$BIN_DIR/$f" ] && echo "  $BIN_DIR/$f"
    done
fi
for rc in "$HOME/.zshrc" "$HOME/.bashrc"; do
    if [ -e "$rc" ] && grep -q "Added by RustCode installer" "$rc" 2>/dev/null; then
        echo "  $rc (PATH line)"
    fi
done

echo
if [ "$DO_KEEP" = 0 ]; then
    echo "Will consider (Group 2 — credentials):"
    for f in $RUSTCODE_GROUP2_FILES; do [ -e "$DATA/$f" ] && echo "  $DATA/$f"; done
    echo
    echo "Will remove (Group 3 — state):"
    for f in $RUSTCODE_GROUP3_FILES; do [ -e "$DATA/$f" ] && echo "  $DATA/$f"; done
    for d in $RUSTCODE_GROUP3_DIRS;  do [ -e "$DATA/$d" ] && echo "  $DATA/$d"; done
    for p in $RUSTCODE_GROUP3_PREFIXES; do
        for entry in "$DATA/$p"*; do [ -e "$entry" ] && echo "  $entry"; done
    done
fi

[ "$DO_DRY" = 1 ] && exit 0

# ---- prompt ----
DO_G2=0; DO_G3=1
if [ "$DO_PURGE" = 1 ]; then
    DO_G2=1; DO_G3=1
elif [ "$DO_KEEP" = 1 ]; then
    DO_G2=0; DO_G3=0
elif [ "$DO_YES" = 0 ]; then
    if [ ! -t 0 ]; then
        echo "refusing to run interactively without a TTY; pass --yes / --purge / --keep-data / --dry-run" >&2
        exit 2
    fi
    printf "[Group 1] Remove binary and PATH edit? [Y/n]: "; read ans
    case "${ans:-Y}" in [Nn]*) echo "aborted"; exit 1 ;; esac
    printf "[Group 2] Remove credentials and global config? [y/N]: "; read ans
    case "${ans:-N}" in [Yy]*) DO_G2=1 ;; esac
    printf "[Group 3] Remove local state and extensions? [Y/n]: "; read ans
    case "${ans:-Y}" in [Nn]*) DO_G3=0 ;; esac
    printf "Continue? [y/N]: "; read ans
    case "${ans:-N}" in [Yy]*) ;; *) echo "aborted"; exit 1 ;; esac
fi

# ---- execute (state → credentials → rc → binary) ----
if [ "$DO_G3" = 1 ]; then
    for f in $RUSTCODE_GROUP3_FILES; do rm -f "$DATA/$f"; done
    for d in $RUSTCODE_GROUP3_DIRS;  do rm -rf "$DATA/$d"; done
    for p in $RUSTCODE_GROUP3_PREFIXES; do rm -f "$DATA/$p"*; done
fi
if [ "$DO_G2" = 1 ]; then
    for f in $RUSTCODE_GROUP2_FILES; do rm -f "$DATA/$f"; done
fi
rmdir "$DATA" 2>/dev/null || true

# rc cleanup
for rc in "$HOME/.zshrc" "$HOME/.bashrc"; do
    [ -e "$rc" ] || continue
    if grep -q "Added by RustCode installer" "$rc"; then
        cp "$rc" "$rc.rustcode-uninstall.bak"
        # Delete the comment line + the next non-blank line if it's an export PATH line.
        awk '
            /^# Added by RustCode installer$/ { skip=1; next }
            skip == 1 && /^export PATH=/ { skip=0; next }
            skip == 1 && /^[[:space:]]*$/ { next }
            { skip=0; print }
        ' "$rc.rustcode-uninstall.bak" > "$rc"
        echo "edited $rc (backup: $rc.rustcode-uninstall.bak)"
    fi
done

# binary
if [ -n "$BIN" ] && [ -e "$BIN" ]; then
    BIN_DIR=$(dirname "$BIN")
    if [ -w "$BIN" ] && [ -w "$BIN_DIR" ]; then
        rm -f "$BIN" "$BIN_DIR/rustcode.bak" \
              "$BIN_DIR/.rustcode.rolling" \
              "$BIN_DIR/.rustcode.download" \
              "$BIN_DIR/.rustcode.writable-probe"
    else
        sudo rm -f "$BIN" "$BIN_DIR/rustcode.bak" \
              "$BIN_DIR/.rustcode.rolling" \
              "$BIN_DIR/.rustcode.download" \
              "$BIN_DIR/.rustcode.writable-probe"
    fi
fi

echo "uninstall complete."
