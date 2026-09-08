#!/usr/bin/env bash
# scripts/build-webui.sh -- RustCode 前端（webui）一键构建脚本
#
# 契约（冻结，见 .codebuddy/artifacts/2026-09-07-zh-docs-webui/01-design.md §4.2）：
#   set -euo pipefail
#   用法: scripts/build-webui.sh [--if-missing | -h | --help]
#   exit 0  成功（或 --if-missing 且 webui/dist/index.html 已存在 -> 打印跳过说明）
#   exit 2  用法错误 / 前置检查失败（无 node、无 npm、无 package-lock、node 版本不足）
#   exit 1  npm ci / npm run build 失败（原样透传其 stderr）
#   要求版本从 webui/package.json 的 engines.node 动态解析（解析失败回退常量 22.6）
#   成功结尾必须打印下一步：cargo clean -p rustcode-daemon
#   全程禁止 sudo（对齐 AGENTS.md:18）
#
# 说明：cargo build 不追踪 webui/dist/ 的变化，重建前端后必须清理 rustcode-daemon，
# 否则二进制仍内嵌旧 dist（AGENTS.md:16）。故本脚本在成功结尾主动打印该命令。
set -euo pipefail

# ---------------------------------------------------------------- 常量与路径

# engines.node 解析失败时的回退最低版本（契约要求）
readonly NODE_MIN_FALLBACK="22.6"

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
WEBUI_DIR="$REPO_ROOT/webui"
PKG_JSON="$WEBUI_DIR/package.json"
LOCK_FILE="$WEBUI_DIR/package-lock.json"
DIST_INDEX="$WEBUI_DIR/dist/index.html"

USAGE="用法: $(basename -- "$0") [--if-missing | -h | --help]

  (无参数)       重跑构建：npm ci && npm run build（幂等，会覆盖已有 webui/dist/）
  --if-missing   仅当 webui/dist/index.html 不存在时才构建；已存在则跳过并 exit 0
  -h, --help     显示本帮助

退出码: 0 = 成功 / 跳过; 1 = npm ci 或 npm run build 失败; 2 = 用法错误或前置检查失败"

# ---------------------------------------------------------------- 输出辅助

log() { printf '%s\n' "$*"; }
err() { printf '%s\n' "$*" >&2; }

usage_error() {
  err "[ERROR] $*"
  err "$USAGE"
  exit 2
}

# 打印 node 安装指引并以 2 退出（fail-closed：绝不降级为警告后继续）
node_install_guide() {
  err ""
  err "[INFO] 修复指引：安装 Node.js >= ${NODE_MIN} 后重试（要求来自 ${PKG_JSON} 的 engines.node）。"
  err "  - nvm（推荐，不需要 sudo）："
  err "      nvm install ${NODE_MIN} && nvm use ${NODE_MIN}"
  err "  - 官方安装包： https://nodejs.org/en/download"
  err "  - 系统包管理器： Debian/Ubuntu 见 https://github.com/nodesource/distributions ；macOS: brew install node"
  err ""
  err "[INFO] 注意：请勿使用 sudo 运行本脚本或 npm（AGENTS.md:18，sudo 会让 ~/.rustcode 出现 root 属主文件）。"
  exit 2
}

# ---------------------------------------------------------------- 版本工具

# 从 webui/package.json 的 engines.node 解析最低版本号；解析失败回退 22.6。
# 支持 ">=22.6"、"^22.6.0"、">=22.6 <24"、"22.6" 等写法；取第一个版本号。
parse_node_min() {
  local raw="" parsed=""
  if [ -f "$PKG_JSON" ]; then
    raw="$(awk '
      /"engines"[[:space:]]*:/ { in_eng = 1; next }
      in_eng && /"node"[[:space:]]*:/ {
        if (match($0, /"node"[[:space:]]*:[[:space:]]*"[^"]*"/)) {
          s = substr($0, RSTART, RLENGTH)
          sub(/^"node"[[:space:]]*:[[:space:]]*"/, "", s)
          sub(/"$/, "", s)
          print s
        }
        exit
      }
      in_eng && /\}/ { in_eng = 0 }
    ' "$PKG_JSON")" || raw=""
  fi

  if [ -n "$raw" ]; then
    case "$raw" in
      *'>'*|*'^'*|*'~'*)
        # 带比较符：取第一个数字段作为下界
        if [[ "$raw" =~ ([0-9]+(\.[0-9]+)*) ]]; then parsed="${BASH_REMATCH[1]}"; fi
        ;;
      *)
        # 无比较符：仅当整体是纯版本号时采信
        if [[ "$raw" =~ ^[0-9]+(\.[0-9]+)*$ ]]; then parsed="$raw"; fi
        ;;
    esac
  fi

  if [ -z "$parsed" ]; then
    err "[WARN] 无法从 ${PKG_JSON} 的 engines.node 解析最低版本（原始值: '${raw}'），回退为 ${NODE_MIN_FALLBACK}"
    parsed="$NODE_MIN_FALLBACK"
  fi
  printf '%s' "$parsed"
}

# version_ge <actual> <min>：逐段数值比较，actual >= min 返回 0，否则返回 1。
# 容忍 "v" 前缀、位数不等（22.6 vs 22.13.1）与非数字后缀（如 13.1-nightly）。
version_ge() {
  local cur="${1#v}" min="${2#v}"
  local IFS=.
  # shellcheck disable=SC2206 # 需要按 IFS 做字段切分
  local -a cur_parts=($cur) min_parts=($min)
  local i n x y
  n=${#cur_parts[@]}
  if [ ${#min_parts[@]} -gt "$n" ]; then n=${#min_parts[@]}; fi
  for ((i = 0; i < n; i++)); do
    x="${cur_parts[i]:-0}"
    y="${min_parts[i]:-0}"
    x="${x%%[^0-9]*}"
    y="${y%%[^0-9]*}"
    [ -z "$x" ] && x=0
    [ -z "$y" ] && y=0
    if ((10#$x > 10#$y)); then return 0; fi
    if ((10#$x < 10#$y)); then return 1; fi
  done
  return 0
}

# ---------------------------------------------------------------- 参数解析

if [ "$#" -gt 1 ]; then
  usage_error "参数过多: $*"
fi

MODE="build"
case "${1:-}" in
  "")
    MODE="build"
    ;;
  --if-missing)
    MODE="if-missing"
    ;;
  -h | --help)
    log "$USAGE"
    exit 0
    ;;
  *)
    usage_error "未知参数: ${1}"
    ;;
esac

# ---------------------------------------------------------------- 前置检查

NODE_MIN="$(parse_node_min)"

log "[INFO] RustCode 前端构建脚本"
log "[INFO] 仓库根目录 : ${REPO_ROOT}"
log "[INFO] 前端目录   : ${WEBUI_DIR}"
log "[INFO] node 要求  : >= ${NODE_MIN}（engines.node）"
log "[INFO] 模式       : ${MODE}"

# 1) node 是否存在
if ! command -v node >/dev/null 2>&1; then
  err "[ERROR] 前置检查失败：PATH 中未找到可执行的 node。"
  node_install_guide
fi

# 2) node 版本是否达标（不足即 fail-closed，绝不降级为警告后继续）
NODE_VERSION=""
if ! NODE_VERSION="$(node --version 2>/dev/null)"; then
  NODE_VERSION=""
fi
if [ -z "$NODE_VERSION" ] || ! version_ge "$NODE_VERSION" "$NODE_MIN"; then
  err "[ERROR] 前置检查失败：node 版本不足。当前 = ${NODE_VERSION:-<无法获取>}，要求 >= ${NODE_MIN}（${PKG_JSON} engines.node）。"
  err "[ERROR] 版本不足不降级为警告，构建终止。"
  node_install_guide
fi

# 3) npm 是否存在
if ! command -v npm >/dev/null 2>&1; then
  err "[ERROR] 前置检查失败：PATH 中有 node 但未找到可执行的 npm。"
  node_install_guide
fi

# 4) package-lock.json 是否存在（npm ci 的硬前提）
if [ ! -f "$LOCK_FILE" ]; then
  err "[ERROR] 前置检查失败：缺少 ${LOCK_FILE}，npm ci 需要 lockfile。"
  err "[INFO] 请在 ${WEBUI_DIR} 下执行 npm install 生成 package-lock.json 后重试（不要用 sudo）。"
  exit 2
fi

log "[INFO] node       : $(command -v node) (${NODE_VERSION})"
log "[INFO] npm        : $(command -v npm)"

# ---------------------------------------------------------------- 跳过判定

if [ "$MODE" = "if-missing" ] && [ -f "$DIST_INDEX" ]; then
  log "[INFO] --if-missing：${DIST_INDEX} 已存在，跳过构建。"
  log "[INFO] 如需强制重跑，去掉 --if-missing 直接运行本脚本。"
  log "[INFO] 提示：继续 cargo 构建前请执行 cargo clean -p rustcode-daemon（cargo 不追踪 webui/dist/）。"
  exit 0
fi

# ---------------------------------------------------------------- 构建

# npm ci 失败：原样透传其 stderr（未重定向），并补充可执行的排查提示，退出 1。
log "[INFO] 执行 npm ci（依赖以 package-lock.json 为准，会重建 node_modules）"
if ! (cd "$WEBUI_DIR" && npm ci); then
  err ""
  err "[ERROR] npm ci 失败（原始错误见上方输出）。常见原因：离线 / registry 不可达 / package-lock.json 与 package.json 不一致。"
  if [ -f "$DIST_INDEX" ]; then
    err "[INFO] 已有构建产物 ${DIST_INDEX}；若不需要重新构建，可用 --if-missing 跳过。"
  fi
  exit 1
fi

# npm run build 失败：原样透传 vite 的 stderr，退出 1，不把半产出 dist 当成功。
log "[INFO] 执行 npm run build（vite build）"
if ! (cd "$WEBUI_DIR" && npm run build); then
  err ""
  err "[ERROR] npm run build（vite build）失败（原始错误见上方输出）。"
  err "[INFO] 未生成有效产物，请按构建器报错修复后重跑本脚本。"
  exit 1
fi

# 后置校验：E6 —— dist 半产出（无 index.html）必须判失败，不得视为已构建。
if [ ! -f "$DIST_INDEX" ]; then
  err ""
  err "[ERROR] 构建命令已返回成功，但 ${DIST_INDEX} 不存在（半产出）。"
  err "[INFO] 请检查磁盘空间后删除 ${WEBUI_DIR}/dist 并重跑本脚本。"
  exit 1
fi

# ---------------------------------------------------------------- 成功收尾

log ""
log "[SUCCESS] 前端构建完成：${DIST_INDEX}"
log ""
log "下一步（必做）：cargo 不追踪 webui/dist/ 的变化，重建前端后必须清理并重新构建 rustcode-daemon，"
log "否则二进制仍内嵌旧 dist（AGENTS.md:16）："
log ""
log "cargo clean -p rustcode-daemon"
log "cargo build"
log ""
log "注意：全程不要使用 sudo 运行本脚本或 npm（AGENTS.md:18）。"
exit 0
