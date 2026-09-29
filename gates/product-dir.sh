#!/usr/bin/env bash
# project-local: 产品目录名只在宿主写一次，库里一处都不许有。
#
# 起因（真实，2026-09-29 实地看过两个下游）：
#   - longcode（fork 整仓）改名动了 239 个文件，之后每合一次上游就在合并提交里
#     手工改回一遍 —— 09-29 那次是 140 行、二十多个文件。
#   - longcode-air（只导入 crate）改不了名，只能在每个进程入口 set_var
#     ATOMCODE_HOME；漏了一个入口，登录态读错目录，界面显示「未登录」。
# 方案见 docs/plans/2026-09-29-product-dir-as-host-data.md：库收目录，不收名字。
#
# 这条闸门数的是非测试代码里的 `.atomcode` 目录字面量（后面不是 `.`、`-`、字母
# 数字下划线）。于是规则文件 `.atomcode.md`、插件清单目录 `.atomcode-plugin/`、
# launchd 的 `com.atomcode.*` 这些另有契约的名字天然不在口径里。唯一允许写它的
# 是 crates/atomcode-config/src/distribution.rs。
#
# 棘轮：数字只能降。降了自动抬低基线；升了判红并列出新增的位置。
#   PRODUCT_DIR_LIST=1 bash gates/product-dir.sh   顺带列出每一处
set -uo pipefail
cd "$(dirname "$0")/.."

ROOT=${PRODUCT_DIR_ROOT:-crates}
BASE=${PRODUCT_DIR_BASELINE:-gates/product-dir.baseline}

# 跳过 `#[cfg(test)]` 下的内联 `mod x { … }`（数花括号找它的结尾），之后接着读
# —— 测试模块常夹在文件中间。`#[cfg(test)] mod x;` 只是声明，不跳。
AWK='
function braces(s,   o, c) { o = gsub(/[{]/, "{", s); c = gsub(/[}]/, "}", s); return o - c }
skip { depth += braces($0); if (depth <= 0) skip = 0; next }
/^[[:space:]]*#\[cfg\(test\)\]/ { armed = 1; next }
armed && /^[[:space:]]*(pub(\([a-z]+\))? )?mod [a-z_0-9]+ *[{]/ {
  armed = 0; depth = braces($0); if (depth > 0) skip = 1; next
}
{ armed = 0 }
/^[[:space:]]*(\/\/|\*)/ { next }
/\.atomcode([^.a-zA-Z0-9_-]|$)/ { print f ":" NR ":" $0 }
'

hits=$(
  find "$ROOT" -name '*.rs' -path '*/src/*' \
       -not -path '*/tests/*' -not -name 'test_support.rs' -not -name 'distribution.rs' |
  sort |
  while read -r f; do awk -v f="$f" "$AWK" "$f"; done
)
n=$(printf '%s' "$hits" | grep -c . || true)

list() { [ "${PRODUCT_DIR_LIST:-0}" = 1 ] && printf '%s\n' "$hits" | sed 's/^/    /'; return 0; }

base=""
[ -f "$BASE" ] && base=$(tr -d ' \n' <"$BASE")
if [ -z "$base" ]; then
  echo "$n" >"$BASE"
  echo "  ⊙ 产品目录字面量：建立基线 ${n}"
  list
  exit 0
fi
if [ "$n" -gt "$base" ]; then
  echo "  ✗ 产品目录字面量 ${n} 处，多于基线 ${base}。库要收目录参数，名字只在 distribution.rs："
  printf '%s\n' "$hits" | sed 's/^/    /'
  exit 1
fi
if [ "$n" -lt "$base" ]; then
  echo "$n" >"$BASE"
  echo "  ✓ 产品目录字面量 ${n} 处（基线从 ${base} 降到 ${n}）"
else
  echo "  ✓ 产品目录字面量 ${n} 处（持平）"
fi
list
exit 0
