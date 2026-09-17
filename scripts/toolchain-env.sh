#!/bin/bash
# Local RustCode dev toolchain (second-dev only, NOT part of CI/release).
# The sandbox lacks system gcc/glibc-devel (/etc is read-only), so Zig provides
# the C compiler and linker. Overrides:
#   RUSTCODE_ZIG           path to zig executable
#   RUSTCODE_TOOLCHAIN_DIR directory where zig-cc/zig-ld wrappers live
set -e

RUSTCODE_ZIG="${RUSTCODE_ZIG:-$(command -v zig || echo /workspace/zig/zig)}"
TOOLCHAIN_DIR="${RUSTCODE_TOOLCHAIN_DIR:-/workspace}"

cat > "$TOOLCHAIN_DIR/zig-cc" <<EOF
#!/bin/bash
args=()
for a in "\$@"; do
  case "\$a" in
    --target=*)
      t="\${a#--target=}"
      args+=("-target" "\${t/-unknown-linux-gnu/-linux-gnu}")
      ;;
    *) args+=("\$a") ;;
  esac
done
exec "$RUSTCODE_ZIG" cc "\${args[@]}"
EOF

cat > "$TOOLCHAIN_DIR/zig-ld" <<EOF
#!/bin/bash
args=()
skip=0
for a in "\$@"; do
  if [ \$skip -eq 1 ]; then skip=0; continue; fi
  case "\$a" in
    -z|-soname|-finstrument-functions) skip=1; continue ;;
    --version-script=*|--no-undefined-version) continue ;;
    --as-needed|--no-as-needed|-Bstatic|-Bdynamic|--gc-sections|--eh-frame-hdr|-pie|--pie|--strip-all|-s) continue ;;
    -zrelro|-znow|-znoexecstack|-zdefs|-ztext) continue ;;
    --fix-cortex-a53-843419|-favor-align) continue ;;
    *) args+=("\$a") ;;
  esac
done
exec "$RUSTCODE_ZIG" cc "\${args[@]}"
EOF

chmod +x "$TOOLCHAIN_DIR/zig-cc" "$TOOLCHAIN_DIR/zig-ld"

export PATH="$HOME/.cargo/bin:$PATH"
export CC="$TOOLCHAIN_DIR/zig-cc"
export CXX="$RUSTCODE_ZIG c++"
export AR="$RUSTCODE_ZIG ar"
export RANLIB="$RUSTCODE_ZIG ranlib"
export RUSTFLAGS="-C linker=$TOOLCHAIN_DIR/zig-ld"

echo "[CHECK] toolchain ready: zig=$RUSTCODE_ZIG"
echo "[INFO] source this file, then run cargo build/check/test"