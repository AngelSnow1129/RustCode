#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# RustCode 开发环境快速部署脚本（加速版）
#
# 适用系统: Huawei Cloud EulerOS / 通用 Linux (aarch64 / x86_64)
# 功能:
#   1. 安装系统依赖 (gcc / gcc-c++ 等 C 工具链)
#   2. 通过 rsproxy.cn 镜像安装 Rust 工具链 (stable, 满足 1.88+)
#   3. 配置 crates.io 国内镜像 + 并行编译 (核心加速项)
#   4. 通过 nvm 安装 Node.js 18 LTS (供 webui / npm 使用)
#   5. (可选) 经 GitHub 镜像站加速安装 opencode CLI
#   6. 验证整套环境
#
# 特性: 幂等 —— 已安装的部分会自动跳过，可安全重复运行。
#
# 用法:
#   bash scripts/dev-env-quickstart.sh
#   bash scripts/dev-env-quickstart.sh --with-release     # 额外做一次 release 编译验证
#   bash scripts/dev-env-quickstart.sh --with-opencode    # 额外安装 opencode CLI (GitHub 镜像加速)
#
# 镜像说明:
#   - Rust 工具链与 crates.io 均走 rsproxy.cn (字节) 国内镜像
#   - opencode 二进制经 GitHub 镜像站 ghfast.top 加速下载 (可改 GITHUB_MIRROR 变量)
#   - 与仓库 scripts/macos-release-windows.sh 中 RUSTCODE_USE_MIRROR=1 一致
# ─────────────────────────────────────────────────────────────────────────────
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# ── 颜色 ─────────────────────────────────────────────────────────────────────
C_RESET='\033[0m'; C_BOLD='\033[1m'
C_GREEN='\033[0;32m'; C_YELLOW='\033[0;33m'; C_CYAN='\033[0;36m'; C_RED='\033[0;31m'
info()    { echo -e "${C_CYAN}[env]${C_RESET} $*"; }
success() { echo -e "${C_GREEN}[ok]${C_RESET}  $*"; }
warn()    { echo -e "${C_YELLOW}[warn]${C_RESET} $*"; }
error()   { echo -e "${C_RED}[error]${C_RESET} $*" >&2; exit 1; }
step()    { echo -e "\n${C_BOLD}==> $*${C_RESET}"; }

WITH_RELEASE=0
WITH_OPENCODE=0
# GitHub 镜像站：用于加速 opencode 等 GitHub Release 二进制下载（可替换为其他可用镜像）
GITHUB_MIRROR="https://ghfast.top"
for a in "$@"; do
  [ "$a" = "--with-release" ] && WITH_RELEASE=1
  [ "$a" = "--with-opencode" ] && WITH_OPENCODE=1
done

# ── 0. 前置检查 ─────────────────────────────────────────────────────────────
step "环境探测"
OS="$(uname -s)"; ARCH="$(uname -m)"
case "$OS" in Linux) PLATFORM="linux" ;; *) error "仅支持 Linux，当前: $OS" ;; esac
info "平台: $PLATFORM / $ARCH"
info "项目: $PROJECT_ROOT"

# 包管理器探测
if command -v dnf &>/dev/null; then PKG="dnf";
elif command -v yum &>/dev/null; then PKG="yum";
elif command -v apt-get &>/dev/null; then PKG="apt";
else warn "未识别包管理器，跳过系统依赖安装（请手动安装 gcc/gcc-c++）"; PKG=""; fi
[ -n "$PKG" ] && info "包管理器: $PKG"

# ── 1. 系统 C 工具链 ────────────────────────────────────────────────────────
step "系统 C 工具链 (gcc / gcc-c++)"
if command -v cc &>/dev/null && command -v g++ &>/dev/null; then
  success "C 工具链已存在: $(cc --version | head -1)"
else
  info "安装 gcc / gcc-c++ ..."
  case "$PKG" in
    dnf|yum) sudo "$PKG" install -y gcc gcc-c++ ;;
    apt)     sudo apt-get update -qq && sudo apt-get install -y build-essential ;;
    *)       error "无包管理器，请手动安装 gcc/gcc-c++" ;;
  esac
  success "C 工具链安装完成"
fi

# ── 2. Rust 工具链 (rsproxy 镜像) ───────────────────────────────────────────
step "Rust 工具链 (rsproxy 镜像)"
export RUSTUP_DIST_SERVER="https://rsproxy.cn"
export RUSTUP_UPDATE_ROOT="https://rsproxy.cn/rustup"
export PATH="$HOME/.cargo/bin:$PATH"

if command -v cargo &>/dev/null; then
  success "Rust 已安装: $(rustc --version) / $(cargo --version)"
else
  info "通过 rsproxy 安装 rustup + stable ..."
  curl --proto '=https' --tlsv1.2 -sSf https://rsproxy.cn/rustup-init.sh -o /tmp/rustup-init.sh
  sh /tmp/rustup-init.sh -y --default-toolchain stable --profile minimal
  . "$HOME/.cargo/env"
  success "Rust 安装完成: $(rustc --version)"
fi

# 确保 rustfmt / clippy（开发需要）
rustup component add rustfmt clippy >/dev/null 2>&1 || warn "rustfmt/clippy 安装失败（非致命）"

# ── 3. cargo 镜像加速 + 并行编译 ────────────────────────────────────────────
step "cargo 镜像加速配置"
CARGO_CFG="$HOME/.cargo/config.toml"
MARKER="# rustcode-dev-env-accel"
# 幂等判定：只要配置中已存在 rsproxy-sparse 镜像即视为已配置，避免重复追加导致 duplicate key
if [ -f "$CARGO_CFG" ] && grep -q "rsproxy-sparse" "$CARGO_CFG" 2>/dev/null; then
  success "cargo 加速配置已存在 (rsproxy-sparse)，跳过"
else
  mkdir -p "$(dirname "$CARGO_CFG")"
  JOBS="$(nproc 2>/dev/null || echo 4)"
  info "写入 crates.io 镜像 (rsproxy.cn) + 并行 jobs=$JOBS"
  # 若文件已存在则追加，否则新建
  if [ -f "$CARGO_CFG" ]; then
    cat >> "$CARGO_CFG" <<EOF

$MARKER
[source.crates-io]
replace-with = 'rsproxy-sparse'
[source.rsproxy-sparse]
registry = "sparse+https://rsproxy.cn/index/"
[registries.rsproxy]
index = "https://rsproxy.cn/crates.io-index"
[net]
git-fetch-with-cli = false
[build]
jobs = $JOBS
EOF
  else
    cat > "$CARGO_CFG" <<EOF
$MARKER
[source.crates-io]
replace-with = 'rsproxy-sparse'
[source.rsproxy-sparse]
registry = "sparse+https://rsproxy.cn/index/"
[registries.rsproxy]
index = "https://rsproxy.cn/crates.io-index"
[net]
git-fetch-with-cli = false
[build]
jobs = $JOBS
EOF
  fi
  success "cargo 加速配置已写入 $CARGO_CFG"
fi

# ── 4. Node.js (nvm) ────────────────────────────────────────────────────────
step "Node.js (nvm, 18 LTS)"
export NVM_DIR="$HOME/.nvm"
if [ -s "$NVM_DIR/nvm.sh" ]; then
  . "$NVM_DIR/nvm.sh"
fi
if command -v node &>/dev/null && [ "$(node --version | cut -d. -f1 | tr -d v)" -ge 18 ] 2>/dev/null; then
  success "Node 已满足要求: $(node --version) / npm $(npm --version)"
else
  if [ ! -s "$NVM_DIR/nvm.sh" ]; then
    info "安装 nvm ..."
    curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
    . "$NVM_DIR/nvm.sh"
  fi
  info "安装 Node 18 LTS ..."
  nvm install 18
  nvm alias default 18
  success "Node 安装完成: $(node --version) / npm $(npm --version)"
fi

# ── 5. (可选) opencode CLI (GitHub 镜像加速) ────────────────────────────────
if [ "$WITH_OPENCODE" = "1" ]; then
  step "opencode CLI (GitHub 镜像: $GITHUB_MIRROR)"
  OC_BIN="$HOME/.opencode/bin/opencode"
  if [ -x "$OC_BIN" ]; then
    success "opencode 已存在: $($OC_BIN --version 2>/dev/null || echo unknown)"
  else
    info "查询最新 release ..."
    OC_VER="$(curl -fsS --http1.1 --connect-timeout 15 "https://api.github.com/repos/anomalyco/opencode/releases/latest" 2>/dev/null | grep -oE '"tag_name": "v[^"]*"' | head -1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+')"
    [ -z "$OC_VER" ] && OC_VER="1.18.25"   # 查询失败时的兜底版本
    info "目标版本: v$OC_VER"
    case "$ARCH" in
      aarch64) OC_ARCH="arm64" ;;
      x86_64)  OC_ARCH="x64"   ;;
      *)       warn "未知架构 $ARCH，跳过 opencode 安装"; OC_ARCH="" ;;
    esac
    if [ -n "$OC_ARCH" ]; then
      OC_URL="https://github.com/anomalyco/opencode/releases/download/v${OC_VER}/opencode-linux-${OC_ARCH}.tar.gz"
      OC_TMP="$(mktemp -d)"
      info "经镜像下载: $GITHUB_MIRROR/${OC_URL}"
      if curl -fsS --http1.1 --connect-timeout 20 -o "$OC_TMP/oc.tar.gz" "$GITHUB_MIRROR/${OC_URL}"; then
        mkdir -p "$HOME/.opencode/bin"
        tar -xzf "$OC_TMP/oc.tar.gz" -C "$OC_TMP"
        mv "$OC_TMP/opencode" "$OC_BIN" && chmod +x "$OC_BIN"
        # 写入 PATH（幂等）
        grep -q 'opencode/bin' "$HOME/.bashrc" 2>/dev/null || \
          printf '\n# opencode\nexport PATH="$HOME/.opencode/bin:$PATH"\n' >> "$HOME/.bashrc"
        success "opencode 安装完成: $($OC_BIN --version 2>/dev/null)"
      else
        warn "opencode 下载失败（镜像不可达），可稍后手动执行: bash <(curl -fsSL https://opencode.ai/install)"
      fi
      rm -rf "$OC_TMP"
    fi
  fi
fi

# ── 6. 验证 ─────────────────────────────────────────────────────────────────
step "环境验证"
echo "  rustc : $(rustc --version 2>/dev/null || echo MISSING)"
echo "  cargo : $(cargo --version 2>/dev/null || echo MISSING)"
echo "  cc    : $(cc --version 2>/dev/null | head -1 || echo MISSING)"
echo "  node  : $(node --version 2>/dev/null || echo MISSING)"
echo "  npm   : $(npm --version 2>/dev/null || echo MISSING)"
echo "  opencode: $(opencode --version 2>/dev/null || echo MISSING)"

# 用临时项目验证镜像 + 编译链路
info "用临时 crate 验证 crates.io 镜像可达 ..."
TMPD="$(mktemp -d)"
trap 'rm -rf "$TMPD"' EXIT
cat > "$TMPD/Cargo.toml" <<'TOML'
[package]
name = "envcheck"
version = "0.1.0"
edition = "2021"
[dependencies]
libc = "0.2"
TOML
mkdir -p "$TMPD/src" && echo 'fn main(){}' > "$TMPD/src/main.rs"
( cd "$TMPD" && cargo fetch -q 2>&1 | tail -3 ) \
  && success "crates.io 镜像拉取成功" \
  || warn "crates.io 镜像拉取异常，请检查网络"

# ── 6. (可选) release 编译验证 ──────────────────────────────────────────────
if [ "$WITH_RELEASE" = "1" ]; then
  step "Release 编译验证 (cargo build --release)"
  cd "$PROJECT_ROOT"
  cargo build --release 2>&1 | tail -5
  success "Release 编译完成"
fi

echo ""
echo -e "${C_GREEN}${C_BOLD}开发环境就绪 [SUCCESS]${C_RESET}"
echo ""
echo "  快速开始:"
echo "    cargo run                          # 运行 TUI (debug)"
echo "    cargo run -p rustcode -- -p \"...\"  # headless 模式"
echo "    cargo test                         # 跑测试"
echo "    cargo clippy                       # lint"
echo "    opencode                          # 启动 opencode (第三方 agent CLI)"
echo ""
echo "  注意: 新开终端会自动加载 Rust/Node/opencode；当前 shell 请先执行:"
echo "    . \"\$HOME/.cargo/env\""
