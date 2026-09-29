#!/usr/bin/env bash
# 阴性对照：product-dir.sh 真能判红，也真能放过不该算的。
# 在 mktemp 里造一棵小 crates 树，量具对着它跑。
set -uo pipefail
cd "$(dirname "$0")/.."

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
src="$tmp/crates/a/src"
mkdir -p "$src" "$tmp/crates/a/tests" "$tmp/crates/atomcode-config/src"
fail=0
check() {                      # check <期望退出码> <说明>
  local want="$1" what="$2" got
  PRODUCT_DIR_ROOT="$tmp/crates" PRODUCT_DIR_BASELINE="$tmp/base" \
    bash gates/product-dir.sh >"$tmp/out" 2>&1
  got=$?
  if [ "$got" = "$want" ]; then
    printf '  \033[32mok\033[0m   %s\n' "$what"
  else
    printf '  \033[31mFAIL\033[0m %s（退出码 %s，期望 %s）\n' "$what" "$got" "$want"
    sed 's/^/        /' "$tmp/out"
    fail=1
  fi
}
put() { printf '%s\n' "$2" >"$src/$1"; }   # put <文件> <内容>

# ── 不该算的 ──
put lib.rs '#[cfg(test)]
mod declared_elsewhere;
// home.join(".atomcode") in a comment
const RULES: &str = ".atomcode.md";
const MANIFEST: &str = ".atomcode-plugin/marketplace.json";
const LABEL: &str = "com.atomcode.schedule";
#[cfg(test)]
mod tests {
    fn t() { let _ = std::path::Path::new("/home/u/.atomcode/x"); }
}'
printf 'fn t() { let _ = ".atomcode/x"; }\n' >"$tmp/crates/a/tests/it.rs"
printf 'pub const HOME_DIR_NAME: &str = ".atomcode";\n' >"$tmp/crates/atomcode-config/src/distribution.rs"
echo 0 >"$tmp/base"
check 0 "注释、测试模块、tests/、distribution.rs、.atomcode.md/.atomcode-plugin/com.atomcode 都不算"

# ── 该算的 ──
put dir.rs 'fn f(h: &std::path::Path) -> std::path::PathBuf { h.join(".atomcode") }'
check 1 "库里 join(\".atomcode\") 判红"
echo 1 >"$tmp/base"
check 0 "基线追上后持平放行"

put dir.rs 'const S: &str = "see ~/.atomcode/config.toml";'
echo 0 >"$tmp/base"
check 1 "文案里的 ~/.atomcode/ 判红"

put dir.rs '#[cfg(test)]
mod declared;
const P: &str = "under `~/.atomcode` globally";'
check 1 "#[cfg(test)] mod x; 只是声明，之后的 \`~/.atomcode\` 照样判红"

put dir.rs '#[cfg(test)]
mod tests {
    fn t() { if true { let _ = ".atomcode/x"; } }
}
fn after_the_tests(h: &std::path::Path) -> std::path::PathBuf { h.join(".atomcode") }'
check 1 "夹在文件中间的测试模块之后，生产代码照样数"

put dir.rs 'const W: &str = "%USERPROFILE%\\.atomcode\\skills";'
check 1 "Windows 写法 \\.atomcode\\ 判红"

# ── 降了要自动抬低基线 ──
rm "$src/dir.rs"
echo 5 >"$tmp/base"
check 0 "少了放行"
if [ "$(cat "$tmp/base")" = 0 ]; then
  printf '  \033[32mok\033[0m   少了之后基线降到实际数\n'
else
  printf '  \033[31mFAIL\033[0m 基线没降（仍是 %s）\n' "$(cat "$tmp/base")"
  fail=1
fi
exit $fail
