<div align="center">
<pre>
   ____            _    ____          _
  |  _ \ _   _ ___| |_ / ___|___   __| | ___
  | |_) | | | / __| __| |   / _ \ / _` |/ _ \
  |  _ &lt;| |_| \__ \ |_| |__| (_) | (_| |  __/
  |_| \_\\__,_|___/\__|\____\___/ \__,_|\___|
</pre>
</div>

<p align="center">
  <strong>用 Rust 编写的开源终端 AI 编码助手</strong>
</p>

<p align="center">
  <a href="#安装">安装</a> ·
  <a href="#快速开始">快速开始</a> ·
  <a href="#功能特性">功能</a> ·
  <a href="#架构">架构</a> ·
  <a href="#开发">开发</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/version-5.0.11-blue" alt="version">
  <img src="https://img.shields.io/badge/rust-1.88%2B-orange" alt="rust">
  <img src="https://img.shields.io/badge/license-MIT-green" alt="license">
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20HarmonyOS%20PC%20%7C%20Windows-lightgrey" alt="platform">
</p>

<h3 align="center">一键安装</h3>

macOS / Linux / HarmonyOS PC（自动检测系统与架构）：

```bash
curl -fsSL https://gitcode.com/api/v5/repos/SecLab/RustCode/raw/scripts/install.sh?ref=dev | sh
```

Windows（PowerShell）：

```powershell
irm https://gitcode.com/api/v5/repos/SecLab/RustCode/raw/scripts/install.ps1?ref=dev | iex
```

或通过包管理器：

```bash
npm install -g @rustcode/rustcode      # npm
brew install --cask rustcode            # Homebrew (macOS)
```

---

> **本项目 100% 由 AI 生成。** 每一行代码、每一个架构决策的实现、每一次提交都由 AI 完成。人类开发者仅担任决策者和产品经理的角色——定义"要做什么"，而不是"怎么做"。

---

RustCode 是一款住在你终端里的 AI 编码助手。用自然语言给它一个任务，它会自动阅读代码、编辑文件、执行命令、验证结果——全程自主完成。

你可以把它理解为 Claude Code / Cursor Agent 的开源替代品，完全运行在终端里，并且可以接入任何兼容 OpenAI 接口的模型。

> **Fork 声明。** 本仓库是上游项目的二次开发 fork。相对上游：(1) 将产品重命名为 `rustcode`（crate、二进制、配置目录 `~/.rustcode`、`RUSTCODE_*` 环境变量）；(2) **完整移除**遥测/分析上报——`rustcode-telemetry` crate 及所有上报调用点均已删除；(3) **完全解除平台绑定**——不硬编码任何签名网关 host，默认不注册平台专属 REST 工具，凭据通过 `~/.rustcode/config.toml` 的 `[providers.*]` 配置（或用 TUI 内的 `/provider`），无需注册账号、无 OAuth、无外部平台；(4) **默认简体中文**——TUI/CLI 界面与 Agent 回复均默认中文（可通过 `--lang en`、配置项 `language` 或 `LANG`/`LC_ALL` 切换为英文）。原始 MIT 许可证与版权（© 2026 Yubang Xu）保留于 [docs/ORIGINAL_LICENSE.md](docs/ORIGINAL_LICENSE.md)，完整归属见 [docs/UPSTREAM_CREDITS.md](docs/UPSTREAM_CREDITS.md)。

## 功能特性

### Agent 循环

- **自主多步执行** —— 读文件、改代码、跑测试、修错误，循环直到完成
- **验证回路** —— 每次编辑后自动跑语法检查确认无误，才算任务完成
- **动态步数预算** —— 根据编辑文件数动态放宽步数上限，同时封顶以控成本
- **循环检测** —— 识别并打破重复调用同一工具的死循环
- **三层 JSON 修复** —— 修复畸形工具调用参数
- **Turn 级 datalog** —— 结构化记录每一轮工具调用，便于回放、调试和评测

### 模式与自主

- **Plan / Build 模式** —— `/plan` 切换到只读探索模式（agent 只调研、不改文件），`/build` 切回完整执行
- **目标模式** —— `/goal <text>` 设定完成条件后，agent 会一轮接一轮自动循环执行，直到目标达成
- **代码审查** —— `/review` 审查当前改动，`/review staged` 审查暂存区，`/review <base>` 对比某个基准 ref
- **后台会话** —— `/bg` 把任务放到分离的槽位执行，长任务进行时你仍可继续使用 TUI

### 内置工具

文件与 Shell：

- `read_file`、`write_file`、`edit_file`、`search_replace`
- `bash`、`grep`、`glob`、`list_directory`、`change_dir`
- `web_search`、`web_fetch`

代码图谱（语言感知的代码智能）：

- `list_symbols`、`read_symbol`、`find_references`
- `trace_callers`、`trace_callees`、`trace_chain`
- `file_deps`、`blast_radius`

自动化：

- `auto_fix` —— 自动 lint / 类型检查修复循环
- `use_skill` —— 调用用户自定义 skill

### 多模型支持

支持任何实现了 OpenAI function calling 接口的模型：

| 提供方               | Function Calling | 已验证模型                                         |
| -------------------- | :--------------: | -------------------------------------------------- |
| Claude（Anthropic）  |       支持       | Claude Sonnet 4.5/4.6、Opus 4.6                    |
| OpenAI               |       支持       | GPT-4o、GPT-4.1                                    |
| DeepSeek             |       支持       | DeepSeek V3、DeepSeek R1、DeepSeek V4              |
| 智谱（GLM）          |       支持       | GLM-4、GLM-5、GLM-5.2                              |
| 通义千问（阿里）     |       支持       | Qwen-Plus、Qwen-Max                                |
| SiliconFlow          |       支持       | 多种开源模型                                       |
| Ollama（本地）       |     部分支持     | Llama 3、Qwen2 等                                  |
| 任意 OpenAI 兼容接口 |       支持       | —                                                  |

### 会话

- **持久化会话** —— 每次对话都会保存；命令行可用 `rustcode --continue` 或 `-c` 继续上一次会话，在 TUI 内可用 `/resume` 恢复或切换
- **第三方供应商（BYO）** —— 在 `~/.rustcode/config.toml` 配置自己的 `base_url` 和 `api_key`（或用 `/provider`），无需注册账号。这是开源默认构建的使用方式
- **Headless 模式** —— `rustcode -p "..."` 非交互式跑一条 prompt，结果直接输出到 stdout（类似 Claude Code 的 `-p`）；需要确认的 `bash` 会自动批准，其他需要确认的工具会被拒绝
- **Daemon 模式** —— `rustcode-daemon` 提供 HTTP API，用于查询会话历史和 SSE 流式对话

### 终端 UI

- **实时流式输出** —— Markdown 渲染 + 语法高亮
- **代码块** —— 语言标签、行号、`base16-ocean.dark` 主题
- **多行输入** —— Shift+Enter 或 `\` + Enter 换行、高度自适应、历史记录
- **任务完成通知** —— 长任务结束后优先走终端原生通知协议，必要时回退到系统通知
- **文本选择** —— 鼠标拖选、自动滚动、复制到剪贴板
- **斜杠命令** —— `/model`、`/provider`、`/resume`、`/bg`、`/diff`、`/undo`、`/cost`、`/clear`、`/compact` 等（完整列表见下）
- **文件附加** —— 粘贴文件路径即可把内容作为上下文带入
- **Bracketed paste** —— 长文本粘贴自动折叠为紧凑的指示器
- **Skills** —— 从 skill 目录加载的用户自定义命令，像普通斜杠命令一样调用

### Web UI

- **`/webui`**（TUI 内）或 **`rustcode webui`**（命令行）会在浏览器里打开一个 Web 界面，作为终端界面之外的另一种选择——同一个 agent、同一份会话，渲染在浏览器中
- **默认绑定 0.0.0.0** —— rustcode webui 与 rustcode daemon 两个子命令默认绑定所有网卡，局域网内的其它设备也能访问；仅靠一次性 token 保护、没有 TLS。显式改回 `127.0.0.1` 即仅本机可访问
- **TUI 内的 /webui 启动路径默认仍是 127.0.0.1** —— 跨设备访问要显式加 --host 0.0.0.0（等价写法 lan）
- **无需预先配置 provider** —— 没有 provider 也能打开 Web UI；可在网页「设置」中可视化配置 provider：新增、编辑、删除、设为默认，以及发现模型
- **`/webui stop`** 停止进程内 server（之后再次 `/webui` 会重新启动）

### 安全性

- **破坏性命令检测** —— `rm -rf`、`git push --force`、`DROP TABLE` 等需要显式确认
- **按路径分层确认** —— 工作区外读取、敏感路径访问、以及所有工作区外写入会按风险等级请求确认
- **敏感文件保护** —— 系统保护路径、凭证目录、shell 配置、`.env` 文件、密钥/证书文件会触发更强的确认规则
- **Shell 绕过防护** —— `cat`、`head`、`ls`、`cp`、`mv`、`tee` 等常见 shell 文件命令会继承和文件工具一致的路径审批模型
- **按会话的权限授予** —— 单条工具模式一次授权，或设为始终允许
- **源码文件删除必须确认** —— 对代码文件执行 `rm` 从不自动放行
- **撤销** —— `/undo` 通过文件历史快照回滚上一轮的所有文件编辑

完整设计与当前边界见 [权限模型](./docs/security/permission-model.md)。

### 远程访问（隧道）

- daemon 可通过可配置的反向隧道中继把本地 webui / API 暴露到公网：环境变量 `RUSTCODE_ENABLE_TUNNEL=1` 开启（默认关闭），`RUSTCODE_TUNNEL_RELAY` 指定中继地址（如 `wss://your-relay.example.com`）。
- 在 TUI 内执行 `/tunnel` 启动本地隧道端点，打印中继 URL、当前 `access_key` 与本地回源端口；子命令：`lan`、`stop`。
- **内置中继客户端（frpc 半边）**：`RUSTCODE_ENABLE_TUNNEL=1` 且中继 URL / token 齐备时，`/tunnel` 会一并启动内置 WebSocket 隧道客户端（连中继 → 用隧道 token 鉴权 → 把中继侧入站流量转发到本地端点）；线路协议是带 stream id 的 `Open` / `Data` / `Close` 帧。配套自建中继 `rustcode-relay` 随仓库提供，见 `docs/relay.md`。
- **`/tunnel lan`（局域网模式）** —— 把隧道端点绑定到 `0.0.0.0`，同一局域网内的设备用同一个 `Authorization: Bearer <access_key>` 就能访问：不需要中继、也不需要额外依赖，可立即使用（建议在可信网络下使用）。不带子命令的 `/tunnel` 仍绑定回环地址，仅本机可达。
- 远程客户端以 `Authorization: Bearer <access_key>` 鉴权（与 daemon 静态访问密钥同一套密钥）；该隧道取代已移除的移动端 App 远程访问，无托管账号或移动端依赖。

### 隐私

- **零遥测** —— 本分支已删除 `rustcode-telemetry` crate 及全部上报调用点，不采集、不发送任何使用事件；详见 [docs/telemetry.md](docs/telemetry.md)，溯源见 [docs/ORIGINAL_LICENSE.md](docs/ORIGINAL_LICENSE.md) 与 [docs/UPSTREAM_CREDITS.md](docs/UPSTREAM_CREDITS.md)

### 项目 Wiki 自动生成

`rustcode-wiki` 模块提供 OpenWiki 风格的项目 wiki 自动生成：分析当前项目，离线产出结构化的项目文档，可选调用 LLM 为模块页补充自然语言摘要。

- **TUI 命令** —— 在 TUI 内输入 `/wiki`，分析当前项目并生成/同步 wiki，默认输出到 `<root>/.rustcode/wiki`。
- **CLI 子命令** —— `rustcode wiki [PATH] [--sync | --watch | --llm | --force | --out-dir <dir> | --title <t> | --exclude <dir> | --interval <秒>]`。其中 `PATH` 为**位置参数**（待分析的项目根目录，默认当前目录），其余为可选 flag：`--sync` 增量同步（仅更新变更页）、`--watch` 监听变更并按 `--interval`（默认 30 秒）重新同步、`--llm` 用已配置 Provider 充实模块页、`--force` 强制全量重建、`--out-dir` / `--title` / `--exclude` 覆盖输出目录、标题与排除目录。
- **产出物** —— 默认位于 `<root>/.rustcode/wiki`，包含：
  - `Home.md`：项目概览（模块数、文件数、代码行数、语言分布）。
  - `Architecture.md`：含 **Mermaid** 依赖图的架构页。
  - `Modules/<模块名>.md`：逐模块页（路径、类型、依赖、文件清单）。
  - `README.md`：wiki 索引（指向上述页面）。
- **确定性离线生成** —— 结构分析、架构图与模块文档 **100% 离线确定性产出**，任何环境都能生成；`--llm`（或配置 `use_llm`）调用已配置的 Provider（OpenAI 兼容网关，如 GLM / DeepSeek）仅为每个模块页补充自然语言摘要，属于「充实」层，**best-effort**：失败仅告警，不影响离线产物。
- **配置（`~/.rustcode/config.toml` 的 `[wiki]` 段）**：

  ```toml
  [wiki]
  auto_generate_on_init = false   # /init 时自动生成 wiki（默认关）
  out_dir = ".rustcode/wiki"      # 输出目录（相对项目根）
  use_llm = false                 # 是否用 LLM 充实模块页（等价 --llm）
  provider = "deepseek"           # 可选：充实用 Provider（省略=全局活动 Provider）
  model = "deepseek-chat"         # 可选：充实用模型（省略=该 Provider 默认模型）
  exclude_dirs = ["target"]       # 额外排除扫描的目录
  auto_sync_interval_secs = 0     # 文件变更后自动同步的轮询间隔（秒，0 = 关）
  ```

## 安装

### 一键安装（推荐）

macOS / Linux / HarmonyOS PC：

```bash
curl -fsSL https://gitcode.com/api/v5/repos/SecLab/RustCode/raw/scripts/install.sh?ref=dev | sh
```

Windows（PowerShell）：

```powershell
irm https://gitcode.com/api/v5/repos/SecLab/RustCode/raw/scripts/install.ps1?ref=dev | iex
```

脚本自动检测系统与架构，下载最新版预编译二进制并写入 `PATH`。
也可从 [Release 页面](https://gitcode.com/SecLab/RustCode/releases) 手动下载。

环境变量覆盖项（可选，脚本已内置 GitCode 默认值）：

- `RUSTCODE_RELEASE_BASE` —— 覆盖下载根目录
- `RUSTCODE_RELEASE_LATEST_API` —— 覆盖最新版本探测 API
- `RUSTCODE_VERSION` —— 固定某个发布版本（如 `v5.0.11`），未设则自动探测最新
- `RUSTCODE_PREFIX` —— 安装目录（详见脚本头部注释）

安装器还支持在安装成功后注入一个自定义 BYO provider（对应 `config.toml` 的
`[providers.<name>]`、`type = "openai-compatible"`）——在脚本后追加
`--url <base_url> --key <api_key> --model <model>`（可选 `--provider <name>`，
默认 `custom`）即可在安装时一并写入配置；提供了 `--url`/`--key` 但缺 `--model`
会 fail-closed 退出（`model` 是该 provider 的必填字段，无合理默认）。例如：

```bash
curl -fsSL https://gitcode.com/api/v5/repos/SecLab/RustCode/raw/scripts/install.sh?ref=dev | sh -s -- \
  --url https://my-gw.example.com/v1 --key sk-xxx --model deepseek-v4.1-flash
```

PowerShell 侧同名参数为 `-Url` / `-Key` / `-Model` / `-Provider`。

### 发布资产

每个 Release 附带以下预编译二进制(命名格式 `rustcode-<tag>-<os>-<arch>[.exe]`):

| 资产名后缀 | 目标 Triple | 说明 |
|---|---|---|
| `linux-x64` | `x86_64-unknown-linux-musl` | 静态链接,任意 Linux 发行版 |
| `linux-arm64` | `aarch64-unknown-linux-musl` | 静态链接,ARM 服务器 / 树莓派 |
| `windows-x64.exe` | `x86_64-pc-windows-gnu` | Windows 10 / 11 x64 |
| `darwin-x64` | `x86_64-apple-darwin` | Intel Mac(GitHub Actions 构建) |
| `darwin-arm64` | `aarch64-apple-darwin` | Apple Silicon(GitHub Actions 构建) |

每个 Release 还附带 `sha256sums.txt` 校验文件。下载后建议校验:

```bash
sha256sum -c sha256sums.txt --ignore-missing
```

### 从源码构建

```bash
# Clone from your distribution channel, e.g.:
git clone https://gitcode.com/SecLab/RustCode.git
cd rustcode
```

#### WebUI 构建（使用 webui 功能时需要 —— 在 Rust 构建之前进行）

`rustcode webui` 浏览器 UI 从 `webui/dist/` 嵌入二进制，该目录被 gitignore、不入库。
Rust 构建本身不需要 Node.js 工具链，缺少该目录也能编译通过，但构建出的二进制在所有
webui 页面上都会返回 `webui not built`。如需可用的 webui，请在 Rust 构建之前先构建前端：

```bash
./scripts/build-webui.sh
```

该脚本会依次执行 npm ci 与 vite build，产物输出 webui/dist/，并在成功结尾打印下一步
cargo clean -p rustcode-daemon。不使用 webui 可跳过这一步（发布脚本会在 `cargo build` 前自动调用该脚本）。
或手工执行等价步骤：

```bash
cd webui
npm ci
# for Windows MSYS / Git Bash users, run
# `PATH="/c/Program Files/nodejs:$PATH" npm run build`
# to use the system-installed Node.js.
npm run build    # outputs webui/dist/, embedded by the next Rust build
cd ..
```

注意 cargo build 不会触发 npm，也不跟踪 `webui/dist/` 的变化：重新构建前端后，必须执行上面那条 (`cargo clean -p
rustcode-daemon`) — cargo does not track changes under `webui/dist/`，需重编 daemon 才能嵌入新 bundle。然后构建并安装：

```bash
cargo install --path crates/rustcode-cli --locked
```

编译产物位于 `target/release/rustcode`。在 macOS / Linux / HarmonyOS PC 其被安装到 `~/.cargo/bin/rustcode`，
在 Windows 系统上其被安装到 `$env:USERPROFILE/.cargo/bin/rustcode.exe`。请确保 `~/.cargo/bin`
（或 `%USERPROFILE%\.cargo\bin`）已经被添加到 `PATH` 环境变量中。

如果只想要编译，不要安装，运行：

```bash
# Builds only the CLI package (`rustcode`) — skips the standalone
# `rustcode-daemon` binary and other workspace members
cargo build --release -p rustcode
```

编译产物会在 `target/release/rustcode` 生成。

本地交叉构建多架构发布资产(Linux x64 runner):

```bash
# 先构建 webui(见上文),然后:
scripts/build-webui.sh

# 构建全部默认 target(linux-x64 / linux-arm64 / windows-x64):
scripts/cross-build.sh

# 或只构建单个 target:
scripts/cross-build.sh linux-arm64

# 产物输出到 dist/v<version>/,含 sha256sums.txt
```

macOS 资产需在 macOS 上原生构建,使用 `scripts/macos-release-linux.sh`(Intel)或
`scripts/macos-release-windows.sh`(Apple Silicon),或通过 GitHub Actions 的
`macos-latest` runner。

### 关于曾经可选的托管网关（已于 2026-09-09 移除）

托管订阅套餐网关及其闭源签名层已于 2026-09-09 一并移除：占位 crate
`crates/rustcode-codingplan-crypto/`、对应的 Cargo feature 与分发渠道覆盖注入机制均已删除，
本仓库不再包含任何托管端点。实际影响：

- 构建来源（自行构建或渠道构建）不再造成能力差异，全部走本地身份体系。
- 连接**你自己的第三方 API 提供商**（自带密钥 / BYO）是唯一方式，且行为不变：在
  `~/.rustcode/config.toml` 的 `providers.*` 下配置的任意提供商（DeepSeek、OpenAI 或任意
  OpenAI 兼容端点）都可直接使用。

### 包管理器安装

除了从源码构建外，RustCode CLI 也可以通过以下包管理器安装：

```bash
# Install using npm
npm install -g @rustcode/rustcode
# Install using Homebrew
brew install --cask rustcode
```

### Shell 补全

RustCode 可为 Bash、Zsh、Fish、PowerShell 和 Elvish 生成补全脚本。例如：

```bash
# Bash (current session)
source <(rustcode completion bash)
# Zsh (persistent)
mkdir -p ~/.zfunc
rustcode completion zsh > ~/.zfunc/_rustcode
# Also add `fpath=(~/.zfunc $fpath)` before `compinit` in ~/.zshrc.

# Fish (persistent)
mkdir -p ~/.config/fish/completions
rustcode completion fish > ~/.config/fish/completions/rustcode.fish
```

PowerShell 可运行 `rustcode completion powershell | Out-String |
Invoke-Expression`. Run `rustcode completion --help` 可查看完整 Shell 列表。该能力只作用于
外部命令行；TUI 内仍由 `Tab` 完成输入补全、`Shift+Tab` 切换执行模式。

### 依赖

- **Rust 1.88+** —— 用于构建；更旧的 Cargo 无法解析当前 lock 文件（`rustup update stable` 即可升级）
- **Node.js >= 22.6** —— **仅当需要从源码构建 WebUI 时**需要（`webui/package.json` 的 `engines.node` 约束；运行预编译二进制或纯 TUI 不需要 Node）。推荐 LTS
- 任一支持的模型提供方的 **API Key**（自带密钥 / BYO；托管网关已于 2026-09-09 移除，见上文「关于曾经可选的托管网关」）
- **不要用 `sudo` 运行**（见下方「权限」小节）

### 权限 —— 不要用 `sudo` 启动

请用**普通用户**运行 RustCode，切勿 `sudo`。RustCode 把配置、会话、日志都放在
`~/.rustcode`；一旦用 root 跑过一次，就会在那里留下 root 属主的文件，之后非 root
启动会在运行时初始化阶段报错：

```
coding runtime assemble failed: Permission denied (os error 13)
```

（提示里可能是 `prepare` 而非 `assemble`——同一个原因。）遇到这种情况，把属主收回
并停止使用 `sudo`：

```bash
sudo chown -R "$(id -un):$(id -gn)" ~/.rustcode
rustcode        # start WITHOUT sudo
```

在 Linux 客户机上，工作目录若在 VirtualBox 共享文件夹（`/media/sf_*`，属主
`root:vboxsf`）也会导致权限错误——用 `sudo usermod -aG vboxsf "$USER"` 把自己加进
该组（重新登录后生效），而不是用 `sudo`。

### 卸载

移除 RustCode 及（可选）其数据：

```bash
rustcode uninstall                # interactive: per-group prompts
rustcode uninstall --keep-data    # only remove binary + PATH edit
rustcode uninstall --purge        # remove everything, including ~/.rustcode
rustcode uninstall --dry-run      # show plan, change nothing
```

二进制已损坏或丢失时，可从你的分发渠道获取 `uninstall.sh`（Windows 为
`uninstall.ps1`）后运行。卸载脚本只删除本地安装，不需要下载根目录：

```bash
sh uninstall.sh
# Windows PowerShell: run the uninstall.ps1 obtained from your channel
./uninstall.ps1
```

默认保留凭据（`mcp.json`、`mcp_auth.toml`、`config.toml`、`RUSTCODE.md`），传 `--purge` 才会一起清除。

## 快速开始

### 1. 首次运行

```bash
rustcode
```

首次运行会有一个向导帮你配置模型：

```
Welcome to RustCode! Let's set up your first provider.

Select provider:
  [1] Claude (Anthropic)
  [2] OpenAI
  [3] OpenAI Compatible (DeepSeek, Qwen, Zhipu, Moonshot...)
  [4] Ollama (local)
```

### 2. 配置

配置文件位于 `~/.rustcode/config.toml`，最小单 provider 样例：

```toml
default_provider = "deepseek"

[providers.deepseek]
type           = "openai"
api_key        = "sk-..."
model          = "deepseek-chat"
base_url       = "https://api.deepseek.com/v1"
context_window = 64000
```

可以配置多个 provider，用 `/model` 或 `/provider` 切换。完整示例
（涵盖 Claude / OpenAI / OpenAI-兼容 endpoint 如 DeepSeek / GLM /
SiliconFlow / OpenRouter / Ollama，以及 `[datalog]` 段）见
[`docs/config.example.toml`](docs/config.example.toml)——拷出来按需改。

手动改完 `config.toml` 后，在 rustcode 里执行 `/reload` 重新加载配置，
不用重启。

### 3. 开始编码

```bash
# Open in your project directory
cd your-project
rustcode

# Or specify directory
rustcode -C /path/to/project

# Or specify model
rustcode --model gpt-4o

# Headless (single prompt, reply on stdout)
rustcode -p "Explain the agent loop in this repo"

# Read prompt from file
rustcode --prompt-file task.md
```

在 headless 模式下，需要确认的 `bash` 会自动批准并写到 stderr；其他需要确认的工具会被拒绝。

然后直接用自然语言描述你想做的事：

```
> Fix the login bug where users get redirected to 404 after OAuth callback

> Add a dark mode toggle to the settings page

> Refactor the database module to use connection pooling

> Write tests for the payment processing module
```

## 快捷键

### 输入

| 键位 | 动作 |
|-----|--------|
| `Enter` | 发送消息 |
| `Shift+Enter` | 换行（需要终端支持 Kitty 键盘协议） |
| `Ctrl+Enter` | 换行（需要终端支持 Kitty 键盘协议） |
| `Ctrl+J` | 换行（终端能区分该组合键时） |
| `Alt+Enter` | 换行（多数终端可用，见下方兼容性说明） |
| `\` + `Enter` | 换行（所有终端通用——输入一个 `\` 后按回车，`\` 会被自动删除） |
| `Esc` | 清空输入 / 取消流式输出 |
| `Esc` ×2 | 撤销上一轮 |
| `Up/Down` | 浏览输入历史 |
| `Tab` | 接受斜杠命令、Skill 或文件补全 |
| `Shift+Tab` | 无补全菜单时切换到下一个执行模式 |
| `F2 / Shift+F2` | 切换下一个 / 上一个模型（Mac 通常按 `Fn+F2 / Fn+Shift+F2`） |
| `Ctrl+R` | 反向搜索输入历史 |
| `Ctrl+T` | 切换 `reasoning_effort` |
| `Ctrl+U` | 清空当前行 |
| `Ctrl+W` | 删除一个单词 |
| `Ctrl+K` | 删除到行尾 |
| `Ctrl+V / Ctrl+Alt+V` | 从剪贴板粘贴文本或图片（Windows 也可用 `/paste`） |

> **换行快捷键的终端兼容性：**
>
> - `Shift+Enter`、`Ctrl+Enter` 需要终端支持 Kitty 键盘协议 — kitty、WezTerm、Alacritty、iTerm2 ≥3.5、Windows Terminal ≥1.21。不支持的终端（以及 Windows，rustcode 在其上不启用该协议）会把它们退化成普通 `Enter`（直接发送消息）—— 请改用 `\` + `Enter`，它在所有终端都生效。
> - RustCode 仅在明确兼容的终端中自动启用 Kitty 键盘协议。JumpServer 等通用 WebTerminal 默认使用传统键盘上报；可通过 `RUSTCODE_KITTY=1` 强制开启，或用 `RUSTCODE_KITTY=0` 强制关闭。
> - `Alt+Enter` 在多数终端的字节层面就能工作，但 **Windows Terminal 默认把它绑给"切换全屏"** — 在 设置 → 操作 中删掉那条绑定即可释放。
> - Xshell 不支持 Kitty 协议；可在键盘映射设置中把某个空闲组合映射为发送 `ESC, Enter`（`\x1b\r`）达到同样效果，或直接从剪贴板粘贴多行文本（已启用 bracketed paste）。

> **Windows 下粘贴图片：**
> Windows Terminal 和 conhost 默认把 `Ctrl+V` 绑给它们自己的 `paste` action — 这个 action 只会从剪贴板读 `CF_UNICODETEXT`，剪贴板上只有图片时它什么都不会发，应用里的 `Ctrl+V` 处理器根本收不到事件。两种解法：
>
> 1. 使用 **`/paste`** —— 这个斜杠命令直接读取剪贴板图片并以 `[Image #N]` 的形式附加到输入框，在 Windows Terminal、PowerShell 7、conhost、git bash 等所有终端里都能正常工作。Windows 版的 TUI 右下角会自动显示 `Image in clipboard · /paste` 提示（中文界面下为「剪贴板有图片 · /paste 粘贴」）。
> 2. 若想保留 `Ctrl+V` 的肌肉记忆：打开 Windows Terminal 的 `settings.json`（`Ctrl+,` → 右下角"打开 JSON 文件"），在 `"actions"` 数组里删掉 `{ "command": "paste", "keys": "ctrl+v" }`，或把它改绑到 `ctrl+shift+v`。重启 Windows Terminal 后，`Ctrl+V` 就能透传给 rustcode 了。
>
> Git Bash（MinTTY）不拦截 `Ctrl+V`，开箱即用。

### 导航

| 键位                | 动作                         |
| ------------------- | ---------------------------- |
| `Shift+Up/Down`     | 滚动聊天区（一行）           |
| `PageUp/PageDown`   | 滚动聊天区（10 行）          |
| `Alt+Up/Down`       | 跳到上一条 / 下一条消息      |
| `Ctrl+Up/Down`      | 跳到上一条 / 下一条用户消息  |
| 空输入时 `Home/End` | 跳到对话顶部 / 底部          |
| `Ctrl+Shift+C`      | 复制选中内容                 |
| `Ctrl+C`            | 取消当前操作（连按两次退出） |

### 斜杠命令

在 TUI 中输入 `/` 即可浏览完整列表并实时补全；`/help` 会列出命令与快捷键。

**会话与工作区**

| 命令                 | 动作                                                                          |
| -------------------- | ----------------------------------------------------------------------------- |
| `/resume`            | 恢复或切换会话                                                                |
| `/session`           | 创建新会话                                                                    |
| `/rename <name>`     | 重命名当前会话                                                                |
| `/clear`             | 开始新对话（清空上下文与屏幕）                                                |
| `/bg`                | 将当前会话放到后台；子命令：`/bg list`、`/bg <N>`、`/bg drop <N>`、`/bg help` |
| `/background <task>` | 兼容入口：在 `/bg` 槽位中启动一次性后台任务                                   |
| `/cd`                | 切换工作目录并开启新建对话                                                    |
| `/worktree`          | Git worktree 隔离（`create` / `list` / `done` / `cleanup`）                   |
| `/webui`             | 启动浏览器 webui（子命令：`stop`、`lan`、`--host <addr>`）                    |
| `/sync`              | 连接到实时 webui 会话（`/sync off` 断开）                                     |

**模式、自主与审查**

| 命令           | 动作                                                                |
| -------------- | ------------------------------------------------------------------- |
| `/plan`        | 切换到 Plan 模式（只读探索）                                        |
| `/build`       | 切换到 Build 模式（完整执行）                                       |
| `/goal <text>` | 设置完成目标——agent 自动循环执行直到条件满足                        |
| `/review`      | 代码审查当前改动（`/review` · `/review staged` · `/review <base>`） |
| `/think`       | 控制深度思考（on / off / budget N）                                 |
| `/effort`      | DeepSeek 推理努力控制（high / max / off）                           |

**Provider 与账号**

| 命令        | 动作                                                                            |
| ----------- | ------------------------------------------------------------------------------- |
| `/model`    | 切换模型 / provider                                                             |
| `/provider` | 管理 provider（添加 / 编辑 / 删除）                                             |
| `/proxy`    | 切换出站代理模式                                                                |
| `/status`   | 查看 provider、模型与上下文文件状态                                             |

**文件、编辑与上下文**

| 命令               | 动作                                                         |
| ------------------ | ------------------------------------------------------------ |
| `/diff`            | 显示当前修改的 git diff                                      |
| `/undo`            | 撤销某一轮的文件编辑（`/undo` 或 `/undo N`）                 |
| `/view <filepath>` | 在浮层窗口中查看文件内容                                     |
| `/paste`           | 从剪贴板粘贴图片（Windows 下 Ctrl+V 被终端拦截时的备用入口） |
| `/copy`            | 从上一条回复复制代码块（`/copy`、`/copy N`、`/copy all`）    |
| `/cost`            | 显示本次会话的 token 消耗                                    |
| `/context`         | 查看上下文预算占用明细                                       |
| `/compact`         | 压缩对话历史                                                 |

**记忆**

| 命令               | 动作                                      |
| ------------------ | ----------------------------------------- |
| `/remember <fact>` | 保存一条记忆（`--global` 对所有项目生效） |
| `/forget <query>` | 删除匹配的记忆                            |
| `/memory`          | 查看所有已保存的记忆                      |

**扩展**

| 命令      | 动作                                                         |
| --------- | ------------------------------------------------------------ |
| `/mcp`    | MCP 服务状态（子命令：`reload`、`tools`、`login`、`logout`） |
| `/plugin` | 插件市场（`marketplace` / `install` / `uninstall` / `list`） |
| `/skills` | 浏览已加载的 skills                                          |

**项目与系统**

| 命令 | 动作 |
|---------|--------|
| `/init` | 按当前语言及可选自定义提示词，创建或完善当前生效的项目指令文件 |
| `/wiki` | 分析当前项目并生成/同步项目 wiki（架构图 + 模块文档），默认输出到 `<root>/.rustcode/wiki` |
| `/config` | 显示配置文件路径 |
| `/reload` | 从磁盘重新加载 `~/.rustcode/config.toml` |
| `/upgrade` | 升级 rustcode 到最新版（子命令：`rollback`） |
| `/setup` | 首次运行：安装推荐 skill 并执行 |
| `/welcome` | 重新运行引导向导 |
| `/language` | 切换显示语言及默认 Git 提交消息语言 |
| `/guide <question>` | 向 rustcode-guide 询问使用方式 |
| `/keys` | 查看键盘快捷键 |
| `/help` | 查看命令与快捷键 |
| `/quit`、`/exit` | 退出 RustCode（或连按 Ctrl+C） |

> **平台 Issue**：`/issue` 与内置的 `platform_issue` 工具均已移除。上游的托管平台 Issue 功能不在本开源构建中提供。
>
> **插件命令**：除了上面的内置命令，插件还能注册自己的斜杠命令。先添加你的分发渠道提供的插件市场（通过配置/环境变量设置市场 URL，或从分发渠道的插件索引安装），再从中安装插件：
>
> ```text
> /plugin marketplace add https://gitcode.com/SecLab/rustcode-plugins
> /plugin install <plugin>@<channel>
> ```

### 自定义命令

除了内置命令和插件命令，你还可以通过 Markdown（.md）模板文件定义自己的斜杠命令，适用于频繁使用的提示词。

**存放位置**（按优先级从低到高）：

| 位置                                                         | 作用域                                  |
| ------------------------------------------------------------ | --------------------------------------- |
| `$RUSTCODE_HOME/commands/`（默认为 `~/.rustcode/commands/`） | 全局 —— 所有项目生效                    |
| `<project>/.rustcode/commands/`                              | 项目级 —— 覆盖同名的全局命令            |
| `plugins/<name>/commands/`                                   | 插件贡献 —— 通过 `/plugin install` 安装 |

**文件格式**：

```markdown
---
name: explain
description: Explain how a specific function or module works
args: required
---

Explain the following code in detail:

$ARGUMENTS

Cover: function signature & parameters, core business logic, data flow & side effects.
```

- **`name`** —— 必填。命令名，输入 `/explain` 触发。
- **`description`** —— 可选。Tab 补全时显示。
- **`args`** —— 可选。控制参数期望与交互行为：

  | 值             | 菜单 Enter 行为           | 空参提交               |
  | -------------- | ------------------------- | ---------------------- |
  | `none`（默认） | 立即执行                  | 允许（替换为 `""`）       |
  | `optional`     | 补全到 `/name `，等待输入 | 允许                   |
  | `required`     | 补全到 `/name `，等待输入 | 拒绝并提示错误         |

  模板变量 `$ARGUMENTS` / `${ARGUMENTS}` 始终替换为用户在命令名后输入的内容（未输入则为空字符串）。

- **模板正文** —— 输入命令后发送给 AI 的提示词。`$ARGUMENTS` 或 `${ARGUMENTS}` 会被替换为用户输入的命令参数。

**示例：创建一个审查命令**

```bash
mkdir -p .rustcode/commands

cat > .rustcode/commands/codereview.md << 'EOF'
---
name: codereview
description: Review the current git diff
args: optional
---

Review all changes in the current git diff.
If specific files are given, review only: $ARGUMENTS
EOF
```

输入 `/help commands` 可查看所有已加载的自定义命令。

> **优先级规则**：自定义命令名不能覆盖同名内置命令。如果内置已有 `/review`，项目级自定义的 `review.md` 不会出现在补全菜单中，也不会被 dispatch。

## 架构

RustCode 是一个分层的 Rust workspace：

```
rustcode/
  crates/
    rustcode-kernel/        # L0 中立 Agent 循环、运行时 trait 与执行边界
    rustcode-capabilities/  # L1 Providers、工具、MCP、skills、session、memory
    rustcode-coding/        # L2 Coding 特化与 CodingRuntime 生命周期
    rustcode-review/        # L2/L3 独立的代码审查 agent
    rustcode-config/        # leaf 配置模型、加载与产品配置策略
    rustcode-updater/       # service 安装包与版本更新能力
    rustcode-tuix/          # L3 终端 UI（ratatui）
    rustcode-cli/           # L3 TUI 与 headless 入口（二进制 `rustcode`）
    rustcode-daemon/        # L3 HTTP/SSE/WebSocket 传输 + 历史 session 单向导入
    rustcode-clix/          # L3 独立 `code`/`review` 命令行驱动（二进制 `rustcodex`）
```

> 工作区 `members = ["crates/*"]`，`default-members` 仅含 `rustcode-cli` / `rustcode-daemon` / `rustcode-tuix`；`rustcode-clix`、`rustcode-review`、`rustcode-config`、`rustcode-updater` 不在默认构建目标内，需用 `-p` 显式指定（如 `cargo build -p rustcode-clix`）。

coding 主调用链是 `CLI/TUI/daemon → CodingRuntime → kernel`。已经退役的 core agent
协议和 `rustcode-bridge` 不再位于运行时路径中。

### 设计原则

1. **技术栈无关** —— 核心引擎不硬编码任何特定语言的逻辑，通过 `package.json`、`Cargo.toml`、`pyproject.toml`、`pom.xml` 等描述文件动态探测项目类型。

2. **单一运行时所有者** —— `CodingRuntime` 统一拥有 live coding agent、provider/session 生命周期、pending request、snapshot 和 controller。driver 只负责输入、展示和传输，不重建第二套 agent runtime。

3. **工具安全** —— 所有破坏性操作必须经用户显式确认。工具失败会作为 observation 返回给模型，绝不 panic。

4. **上下文感知** —— token 预算感知的会话窗口、项目文件树注入、每轮系统提醒，在不超出上下文限制的同时让模型保持专注。

5. **依赖单向** —— kernel 保持中立；capabilities 与 coding 不依赖 `rustcode-core`；历史 session 数据只在显式兼容边界处理，不作为 runtime fallback。

## 项目指令文件

在项目根目录创建 `.rustcode.md` 文件，给 RustCode 提供持久化上下文：

```markdown
# Project Instructions

This is a Vue 3 + TypeScript project using Pinia for state management.

- Always use Composition API with `<script setup>`
- Use TailwindCSS for styling, no inline styles
- Run `npm run lint` after editing .vue/.ts files
```

RustCode 会自动读取这个文件并注入到系统提示中。RustCode 也支持 `AGENTS.md`（AI 编程代理的[开放标准](https://agents.md/)）作为替代——如果两个文件同时存在，`.rustcode.md` 优先。

运行 `/init` 可分析仓库并创建或完善当前生效的项目指令文件，生成语言跟随当前 `/language`。如需追加团队自定义要求，可在 `/config` 中设置“自定义 /init 提示词文件”，或在 `$RUSTCODE_HOME/config.toml` 中添加 `init_prompt_file = "prompts/init.md"`；相对路径基于 `$RUSTCODE_HOME` 解析。

## 开发

### 前置条件

- **Rust 1.88+** —— 通过 [rustup](https://rustup.rs/) 安装
- **Git**
- 任一支持的模型 API Key（用于运行时测试）

### 从源码构建

```bash
# Clone from your distribution channel, e.g.:
git clone https://gitcode.com/SecLab/RustCode.git
cd rustcode

# Debug build (fast compilation, slower runtime)
cargo build

# Release build (slower compilation, optimized binary)
cargo build --release
```

### 开发时运行

```bash
# Run the TUI directly (debug mode)
cargo run -p rustcode

# With arguments
cargo run -p rustcode -- -C /path/to/project
cargo run -p rustcode -- --model gpt-4o

# Headless mode
cargo run -p rustcode -- -p "summarize this repo"

# Daemon (HTTP API)
cargo run -p rustcode-daemon
```

### 测试

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p rustcode-capabilities
cargo test -p rustcode-tuix

# Run a specific test
cargo test -p rustcode-capabilities test_name
```

### 常用命令

```bash
# Check compilation without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy

# Build and install to ~/.cargo/bin
cargo install --path crates/rustcode-cli
```

## 打赏

---

RustCode 是免费的开源软件，可搭配任意你自带密钥的第三方服务商使用。如果它帮你省下了一点时间，欢迎请维护者喝杯咖啡，让我们更有动力把它做下去。

<p align="center">
  <em>[ 赞赏码图片（支付宝 / 微信支付）—— 由你的分发渠道提供 ]</em>
</p>

## 常见问题与故障排查

### WebUI 打开后空白 / 提示 `webui not built`

`webui/dist/` 由前端构建产出且被 gitignore，Rust 构建本身不触发 `npm`。如果没构建前端就编译，所有 webui 页面都会返回 `webui not built`。修复：

```bash
./scripts/build-webui.sh          # 等价于 cd webui && npm ci && npm run build
cargo clean -p rustcode-daemon     # 必须：让 daemon 重新嵌入新 bundle
cargo build --release -p rustcode  # 重新构建
```

`scripts/build-webui.sh --if-missing` 仅在 `webui/dist/index.html` 不存在时才构建，可重复执行。

### 启动报 `Permission denied (os error 13)` / `coding runtime assemble failed`

几乎都是用 `sudo` 跑过一次，在 `~/.rustcode` 留下了 root 属主文件。收回属主并**永远用普通用户**运行：

```bash
sudo chown -R "$(id -un):$(id -gn)" ~/.rustcode
rustcode        # 不要加 sudo
```

Linux 客户机若工作目录在 VirtualBox 共享文件夹（`/media/sf_*`），改用 `sudo usermod -aG vboxsf "$USER"` 加组，而非 `sudo`。

### 改了 `~/.rustcode/config.toml` 不生效

在 TUI 内执行 `/reload` 即可热加载，无需重启；或重启进程。配置文件路径可用 `/config` 查看。

### Headless 模式下某些工具被拒绝

`rustcode -p "..."` 中，需要确认的 `bash` 会自动批准并写 stderr；**其他需要确认的工具会被拒绝**（例如会改文件的写操作）。这是设计内的安全行为，不是 bug。

### 换行快捷键（Shift+Enter / Ctrl+Enter）没反应，直接发送了消息

这两个组合依赖终端支持 Kitty 键盘协议（kitty、WezTerm、Alacritty、iTerm2 ≥3.5、Windows Terminal ≥1.21）。不支持的终端会退化成普通 `Enter`。**通用解法：输入一个 `\` 再按 `Enter`**（`\` 会被自动删除），在所有终端都生效。详见上方「换行快捷键的终端兼容性」。

### 怎么接入自己的模型（BYO）

- 编辑 `~/.rustcode/config.toml` 的 `[providers.*]`（最小样例见上方「配置」）；或
- 在 TUI 内用 `/provider` 可视化添加；或
- 首次运行的无参数向导会引导你填写。

本分支不内置任何网关或账号，**所有** provider 都走你自己的第三方密钥，无需注册、无 OAuth、无外部平台。

### 界面 / Agent 回复是英文，想用中文

默认即简体中文。若显示英文，多半是环境变量 `LANG`/`LC_ALL` 显式设成了不支持的 locale（如 `fr_FR`），会回退英文；改为 `zh_CN.UTF-8` 或清空该变量即可。也可显式 `--lang zh` 或 TUI 内 `/language`。

### 编译报错 `failed to parse lock file` 或解析失败

Cargo 版本过旧，无法解析当前 `Cargo.lock`。升级工具链：`rustup update stable`，要求 Rust 1.88+。

### Windows 下 `Ctrl+V` 粘贴不了图片

Windows Terminal / conhost 把 `Ctrl+V` 绑给了自身 `paste`（只读文本），应用收不到图片事件。改用 **`/paste`** 斜杠命令，在所有终端都正常；或在 Windows Terminal 设置里解除 `ctrl+v` 绑定。

### WebUI 局域网可访问但不安全

`rustcode webui` 与 `rustcode daemon` 默认绑定 `0.0.0.0`，仅靠一次性 token 保护、**无 TLS**。暴露到公网有风险，需要远程访问时建议改 `127.0.0.1` 后用 SSH 隧道，或仅本机使用。TUI 内的 `/webui` 默认仍是 `127.0.0.1`。

### `cargo install` 与 `cargo build` 产物名字

`rustcode-cli` 的**包名是 `rustcode`**，因此 `-p rustcode-cli` 会失败；发布版 CLI 用 `cargo install --path crates/rustcode-cli --locked` 或 `cargo build --release -p rustcode`，产物都是 `target/release/rustcode`。独立 `code`/`review` 命令行驱动是 `rustcode-clix`（二进制 `rustcodex`），不在默认成员中。

## 许可证

MIT License。详见 [LICENSE](LICENSE)。

---

<p align="center">
  用 Rust、ratatui 以及无数个深夜构建而成。
</p>
