# DevEnv 快速上手

> RustCode 二次开发环境速查表。平台中立、零遥测；LLM 一律走你自己配置的第三方 OpenAI/Anthropic 兼容端点（自带密钥）。
> 完整说明见 [`docs/dev-env-setup.md`](docs/dev-env-setup.md)。

## 一键搭建

```bash
bash scripts/dev-env-quickstart.sh --with-opencode
```

## 已装工具

| 工具 | 命令 | 版本 |
|------|------|------|
| Node 运行时 | `node -v` | v24.20.0 |
| Rust 工具链 | `rustc -V` | 1.98.0 |
| Codex（可选） | `codex -V` | 0.151.0 |
| Claude（可选） | `claude -V` | 2.1.251 |
| opencode（可选） | `opencode -V` | 1.18.25 |

## 关键配置文件

- `~/.bashrc` — PS1=DevEnv、第三方密钥（环境变量）、PATH、镜像加速
- `~/.cargo/config.toml` — rsproxy 镜像
- `~/.config/opencode/opencode.json` — 第三方 OpenAI 兼容端点（example.com 占位）
- `~/.rustcode/config.toml` — RustCode provider 配置（第三方 BYO）

## 常用命令

```bash
cargo build                  # 构建
cargo test --workspace       # 全量测试
cargo clippy --workspace --all-targets   # lint
cargo run                    # TUI（包名是 rustcode，勿用 -p rustcode-cli）
cargo run -p rustcode -- -p "..."        # headless
```

## 模型密钥

通过环境变量注入，切勿硬编码或提交真实值：

```bash
export MY_PROVIDER_API_KEY="sk-your-own-key"   # 在 ~/.rustcode/config.toml 用 ${MY_PROVIDER_API_KEY} 引用
```

## 加速镜像（可选）

- npm: `https://registry.npmmirror.com`
- cargo: `https://rsproxy.cn`（sparse index 镜像源）
- GitHub: `https://ghfast.top` (opencode 下载)
