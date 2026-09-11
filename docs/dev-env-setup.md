# DevEnv 开发环境配置说明

> 本文档记录 RustCode 二次开发环境的完整搭建过程，便于在新机器/新容器上快速复现。
> 适用系统：通用 Linux（x86_64 / aarch64）、macOS。
> 最后更新：2026-08-31。

RustCode 是平台中立、零遥测的分支：不绑定任何托管网关或模型厂商。
开发/运行时使用的 LLM 一律由你自己配置的**第三方** OpenAI / Anthropic 兼容端点提供（自带密钥），
任何密钥/端点都通过环境变量或配置注入，**切勿硬编码或提交真实凭据**。

## 0. 一键复现

仓库自带加速脚本，可一键完成 Rust 工具链 + cargo 镜像 + Node + opencode 的安装：

```bash
cd /workspace/RustCode
bash scripts/dev-env-quickstart.sh --with-opencode
```

脚本特性：**幂等**，已安装部分自动跳过，可安全重复运行。

## 1. 工具链清单

| 工具 | 版本 | 安装方式 | 配置位置 |
|------|------|----------|----------|
| Node.js | v24.20.0 (LTS) | nvm | `~/.nvm/` |
| npm | 11.19.0 | 随 Node | — |
| Rust | 1.98.0 | rustup (rsproxy 镜像) | `~/.cargo/env` |
| cargo | 1.98.0 | 随 Rust | `~/.cargo/config.toml` |
| gcc/g++ | 系统自带 | apt/yum/dnf | — |
| codex | 0.151.0 | `npm i -g @openai/codex`（可选，第三方 agent CLI） | `~/.codex/` |
| claude | 2.1.251 | `npm i -g @anthropic-ai/claude-code`（可选，第三方 agent CLI） | `~/.claude/` |
| opencode | 1.18.25 | 仓库脚本 (GitHub 镜像)（可选，第三方 agent CLI） | `~/.opencode/bin/` |

> codex / claude / opencode 都是可选的第三方 agent CLI，仅用于对照/调试，**不是** RustCode 的构建或运行依赖。

> **最低版本要求**：Rust 1.88+（本 quickstart 经 rustup 安装 1.98.0）；WebUI 前端构建需 Node.js >= 22.6（见 webui/package.json 的 engines 字段，本 quickstart 经 nvm 安装 v24.20.0 作为测试版本）。

## 2. 加速配置（可选，按网络环境选用）

以下镜像均为公共社区加速源，仅加速工具链/依赖下载，与任何模型服务商无关；
网络可直连官方源时可跳过本节。

### 2.1 npm 镜像

```bash
npm config set registry https://registry.npmmirror.com
```

或写入 `~/.bashrc`：
```bash
export npm_config_registry="https://registry.npmmirror.com"
```

### 2.2 Rust / cargo 镜像（rsproxy.cn）

由 `scripts/dev-env-quickstart.sh` 自动写入 `~/.cargo/config.toml`：

```toml
# rustcode-dev-env-accel
[source.crates-io]
replace-with = 'rsproxy-sparse'
[source.rsproxy-sparse]
registry = "sparse+https://rsproxy.cn/index/"
[registries.rsproxy]
index = "https://rsproxy.cn/crates.io-index"
[net]
git-fetch-with-cli = false
[build]
jobs = 16   # = nproc
```

环境变量（写入 `~/.bashrc`）：
```bash
export RUSTUP_DIST_SERVER="https://rsproxy.cn"
export RUSTUP_UPDATE_ROOT="https://rsproxy.cn/rustup"
```

### 2.3 opencode GitHub 镜像

仓库脚本通过 `https://ghfast.top` 加速下载 GitHub Release 二进制。
如需更换镜像，修改 `scripts/dev-env-quickstart.sh` 中的 `GITHUB_MIRROR` 变量。

## 3. CLI 安装命令

### 3.1 Rust 工具链（加速）

```bash
# 方式一：仓库加速脚本（推荐）
bash scripts/dev-env-quickstart.sh

# 方式二：手动 rsproxy 安装
curl --proto '=https' --tlsv1.2 -sSf https://rsproxy.cn/rustup-init.sh -o /tmp/rustup-init.sh
sh /tmp/rustup-init.sh -y --default-toolchain stable --profile minimal
. "$HOME/.cargo/env"
rustup component add rustfmt clippy
```

### 3.2 Node.js（nvm）

```bash
curl -fsSL https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash
export NVM_DIR="$HOME/.nvm" && . "$NVM_DIR/nvm.sh"
nvm install --lts
```

### 3.3 npm 全局第三方 CLI（可选，淘宝镜像加速）

```bash
npm config set registry https://registry.npmmirror.com
npm install -g @openai/codex
npm install -g @anthropic-ai/claude-code
```

### 3.4 opencode CLI（可选，仓库脚本加速）

```bash
bash scripts/dev-env-quickstart.sh --with-opencode
```

或手动（GitHub 镜像）：
```bash
curl -fsSL https://ghfast.top/https://github.com/anomalyco/opencode/releases/download/v1.18.25/opencode-linux-arm64.tar.gz -o /tmp/oc.tar.gz
mkdir -p ~/.opencode/bin && tar -xzf /tmp/oc.tar.gz -C ~/.opencode/bin
```

## 4. 模型服务商凭据（第三方自带密钥）

RustCode 不内置任何网关或模型账号。运行时所需的密钥一律从环境变量读取，
在 `config.toml` 中用 `env:VAR` / `${VAR}` / `${VAR:-default}` 展开引用：

```bash
# 写入 ~/.bashrc（值来自你在第三方服务商控制台创建的密钥，切勿提交真实值）
export MY_PROVIDER_API_KEY="sk-your-own-key"
```

> [!WARNING]
> [WARN] 安全提示：密钥以环境变量方式注入，仅用于本地开发机；不要把真实密钥写进任何文件并提交到仓库。
> 清除某个导出：`sed -i '/^export MY_PROVIDER_API_KEY=/d' ~/.bashrc`

验证 RustCode 能读到配置：
```bash
# 无任何 provider 时，headless 模式会给出可操作的报错并指向配置路径
cargo run -p rustcode -- -p "hi"
# TUI 交互引导（无参数启动）会引导你填写第三方 provider
cargo run
```

## 5. opencode 第三方模型配置（可选）

配置文件：`~/.config/opencode/opencode.json`

接入任意**第三方** OpenAI 兼容端点（示例域名 `example.com`，替换为你的服务商地址）：

```json
{
  "model": "thirdparty/your-model-id",
  "provider": {
    "thirdparty": {
      "name": "thirdparty",
      "npm": "@ai-sdk/openai-compatible",
      "options": {
        "apiKey": "{env:MY_PROVIDER_API_KEY}",
        "baseURL": "https://api.example.com/v1"
      },
      "models": {
        "your-model-id": { "id": "your-model-id", "limit": { "context": 128000, "output": 8192 } }
      }
    }
  }
}
```

密钥从环境变量读取，不要把明文 key 写进该 JSON。

## 6. 终端提示符（PS1 = DevEnv）

写入 `~/.bashrc`，将容器默认的 `root@<hash>` 替换为清爽的 `DevEnv`：

```bash
export PS1='[\[\e[1;36m\]DevEnv\[\e[0m\] \[\e[1;33m\]\w\[\e[0m\]]\$ '
```

效果：
```
[DevEnv /workspace/RustCode]#
```

- `DevEnv` 青色加粗
- 当前路径 `\w` 黄色加粗
- root 用 `#`，普通用户用 `$`

## 7. ~/.bashrc 完整追加段

以下为新增到 `~/.bashrc` 末尾的全部内容（nvm/cargo 段由各自安装器写入）：

```bash
# ── DevEnv: 终端提示符与工具链集成 ──────────────────────────────────────────
export PS1='[\[\e[1;36m\]DevEnv\[\e[0m\] \[\e[1;33m\]\w\[\e[0m\]]\$ '

# npm 国内镜像加速（淘宝源，可选）
export npm_config_registry="https://registry.npmmirror.com"

# Rust / cargo 镜像加速（rsproxy.cn，可选）
export RUSTUP_DIST_SERVER="https://rsproxy.cn"
export RUSTUP_UPDATE_ROOT="https://rsproxy.cn/rustup"

# 第三方模型服务商密钥（值由你自己填写，切勿提交真实值）
export MY_PROVIDER_API_KEY="sk-your-own-key"

# ── opencode CLI（GitHub 镜像加速安装，可选）──────────────────────────────
export PATH="$HOME/.opencode/bin:$PATH"
```

## 8. 验证

```bash
source ~/.bashrc

# 工具链
node --version      # v24.20.0
npm --version       # 11.19.0
rustc --version     # rustc 1.98.0
cargo --version     # cargo 1.98.0

# 可选的第三方 agent CLI
codex --version     # codex-cli 0.151.0
claude --version    # 2.1.251 (Claude Code)
opencode --version  # 1.18.25
```

## 9. RustCode 常用命令

```bash
cargo build                              # 构建 default-members（CLI/daemon/tuix）
cargo build --release -p rustcode        # 发布版 CLI（包名是 rustcode）
cargo test                               # 默认成员测试
cargo test --workspace                   # 全量测试
cargo clippy --workspace --all-targets   # lint
cargo fmt                                # 格式化
cargo run                                # 运行 TUI (debug)
cargo run -p rustcode -- -p "..."        # headless 模式
```

> 注意：`rustcode-cli` 目录的**包名是 `rustcode`**，用 `-p rustcode-cli` 会失败；运行 TUI 直接 `cargo run` 即可。
> 禁止 `sudo` 运行——`~/.rustcode` 一旦出现 root 属主文件，非 root 启动会失败。
