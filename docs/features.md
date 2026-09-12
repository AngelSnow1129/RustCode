# RustCode 功能与能力总览

> 本文档面向使用者和二次开发者,汇总 RustCode fork 的产品能力、平台定位与
> 近期新增功能。面向 Agent 的开发约束见 `AGENTS.md`;运行时术语见 `../CONTEXT.md`;
> 架构分层见 `docs/architecture.md`;平台中立化 / 发行去厂商化 / 输出 ASCII 化等
> 工程卫生的落地明细见 `docs/platform-neutralization.md`。

## 产品定位

RustCode 是上游项目的二次开发 fork,核心诉求:

1. **产品重命名** — 产品标识统一为 `rustcode-*`(crate、二进制、命令名),配置
   目录 `~/.rustcode`,环境变量前缀 `RUSTCODE_*`。
2. **零遥测** — 删除 `rustcode-telemetry`,无任何 Sentry/PostHog/Segment/GA
   埋点 SDK 或上报调用;崩溃仅输出 stderr。
3. **平台中立** — 不硬编码任何签名网关 host,不默认注册平台专属 REST 工具;
   仅保留第三方 BYO(自带密钥)provider 配置。
4. **默认中文** — TUI/CLI 界面与 Agent 回复默认简体中文。

## 核心能力

### 1. LLM Provider 解耦(第三方 BYO)

- 统一 trait `LlmProvider`(`chat_stream` → `BoxStream<StreamEvent>`)。
- 三个适配器:Anthropic、OpenAI-compatible、Ollama。
- 工厂 `CodingProviderFactory` 按 `provider_type` 分发。
- 配置支持 `base_url` / `api_key`(含 `$VAR` 展开)/ `extra_headers` /
  `proxy` / `skip_tls_verify` / `retry_max_attempts` / `thinking_*` 等。

配置示例(`~/.rustcode/config.toml`):

```toml
default_provider = "my-provider"

[providers.my-provider]
type = "openai"
api_key = "${MY_PROVIDER_API_KEY}"
model = "your-model-id"
base_url = "https://api.example.com/v1"
context_window = 128000
```

### 2. 多 Agent 并行(子代理双车道 + 内置模板)

- **双车道信号量**:写车道(worker,默认 3)与只读车道(explore,默认 8)独立,
  排查扇出不被写预算节流。
- **内置并行模板**(`subagent.parallel_template`,默认开启):自动挂载
  `explorer`(codex/只读)、`builder`(claude-code/accept-edits)、
  `reviewer`(codex/只读)三个角色,worker 车道提到 4。
- 显式 `[[subagent.external]]` 同名条目覆盖内置角色;daemon/headless 降级
  fail-closed(禁 bypass)。
- `/config` 可实时切换(`ApplyPolicy::CapabilityReprepare`),镜像 `/think`。

```toml
[subagent]
parallel_template = true   # 默认开启,可关闭
max_concurrent = 3         # 模板开启时提升到 4
max_rounds = 200

[[subagent.external]]      # 可选:覆盖某个角色
name = "builder"
kind = "claude-code"
permission = "accept-edits"
model = "claude-sonnet-4-20250514"
```

### 3. 本地语义检索(zg / zvec-grep)

- zg 提供向量检索、BM25、混合检索与 rg 精确匹配,面向"按意图而非关键词"
  的本地检索。
- 默认本地 embedding(`local/potion-code-16m-v2`,无需 GPU),数据不外传;
  远程 embedding 需显式 `zg auth` 授权。
- 通过 MCP 接入(见 `.mcp.json.example` 模板 D),Agent 可调用语义检索。

```bash
npm install -g @zvec/zvec-grep
zg index --glob '!target' --glob '!node_modules'
zg query --human "how is theme preference restored on startup"
```

MCP 配置:

```json
{ "mcpServers": { "zg": { "command": "zg", "args": ["server", "--stdio"] } } }
```

### 4. i18n 统一字段映射

- 四端(Rust / WebUI / VS Code / JetBrains)统一 `{name}` 占位符语法与
  fallback 链:用户语言 → 产品默认(zh_CN)→ key 本身。
- 产品默认语言为简体中文;`LANG=C`/`POSIX`/空值视为"无偏好"→ 中文,
  显式不支持 locale(如 `fr_FR`)→ 英文回退。
- 权威对照源见 `docs/i18n-field-mapping.md`。

### 5. 项目 Wiki 自动生成(rustcode-wiki)

- 分析当前项目并离线产出 OpenWiki 风格 wiki：结构分析、架构图(Mermaid 依赖图)
  与逐模块文档 **100% 确定性离线生成**，任何环境都能产出。
- TUI 命令 `/wiki` 与 CLI 子命令
  `rustcode wiki [PATH] [--sync | --watch | --llm | --force | --yes | --out-dir <dir> |
  --title <t> | --exclude <dir> | --interval <秒> | --lang <zh|en>]`
  （`PATH` 为位置参数，默认当前目录）。
- 产出 `Home.md` / `Architecture.md`(含 Mermaid) / `Modules/<模块名>.md` / `README.md`，
  默认 `<root>/.rustcode/wiki`。
- 可选 `--llm`(或配置 `use_llm`)调用已配置 Provider(OpenAI 兼容网关)仅为模块页
  补充自然语言摘要，属 best-effort「充实」层，失败仅告警；摘要写入
  `<lang>/Modules/<模块>.summary.md` 侧车，后续 `sync`/`generate` 会保留，不被占位文本覆盖。
- 配置段 `[wiki]`(`auto_generate_on_init` / `out_dir` / `use_llm` / `exclude_dirs`
  / `auto_sync_interval_secs` / `langs`)可控制 `/init` 自动生成、输出目录、LLM 充实、
  TUI 后台自动同步与生成语言。`wiki` 内容默认以**中文为主**，并按 `zh`/`en` 分目录
  同时产出中英文两份文档，分别保存为 `<out>/zh/` 与 `<out>/en/`（`langs` 省略即双语言；
  也可用 CLI `--lang zh|en` 或 `[wiki] langs` 指定单一语言）。`auto_sync_interval_secs > 0`
  时 TUI 启动后周期性重新 `sync` 已存在的 wiki（仅在已生成过时生效，不凭空新建），
  变更仅在确实改动页面时提示。
- **双语策略示例**（默认双语言，中文为主）：

  ```text
  <root>/.rustcode/wiki/
  ├── manifest.json          # 工具生成的清单（源文件 sha256 + 各生成文件内容哈希）
  ├── zh/                    # 中文（主文档）
  │   ├── Home.md
  │   ├── Architecture.md    # 含 ```mermaid 依赖图
  │   ├── README.md
  │   └── Modules/<模块名>.md
  └── en/                    # 英文
      ├── Home.md
      ├── Architecture.md
      ├── README.md
      └── Modules/<模块名>.md
  ```

  常用命令：

  ```bash
  rustcode wiki                       # 默认双语言生成到 .rustcode/wiki
  rustcode wiki --lang en             # 仅英文
  rustcode wiki --sync                # 增量同步（源码变化才重写页面）
  rustcode wiki --watch --interval 60 # 监听并每 60s 同步
  rustcode wiki --llm                 # 额外用 LLM 充实模块摘要（需已配置 Provider）
  rustcode wiki --yes                 # 对已存在的本工具 wiki 跳过交互确认直接更新
  rustcode wiki --force               # 强制全量重写，覆盖用户手动修改（见下）
  ```

- **wiki 写入安全护栏**：生成/同步前先 `precheck` 目标目录。`manifest.json` 中
  `generator == "rustcode-wiki"`（或旧版 `version`）才视为本工具生成的 wiki；其余情况
  （目录不存在/为空 → 安全生成；目录存在但非本工具生成 → **拒绝覆盖**，保护用户内容，
  提示改用其它 `--out-dir` 或先手动删除）。对已存在的本工具 wiki 二次更新前会**再次确认**
  （CLI 交互环境询问 `[y/N]`，非交互环境需 `--yes`；TUI 中以显式 `/wiki` 作为确认；
  后台自动同步仅维护本工具已有 wiki，遇外来目录自动跳过）。
- **手动修改保留（默认安全）**：对已生成 `.md` 的手动编辑，默认在后续 `sync`/`generate`
  中**被保留而非覆盖**——引擎比对 `manifest.json` 中记录的文件内容哈希，发现页面被改动即跳过
  该文件，并在 CLI 输出 `已保留你对 <path> 的手动修改（使用 --force 可覆盖）` 提示。
  传入 `--force` 才会以重新生成的内容覆盖你的修改（同时用于「强制全量重写」）。
  LLM 摘要侧车 `<lang>/Modules/<模块>.summary.md` 与网页本体分离，始终按用户提供内容保留。

### 6. 守护进程客户端鉴权（静态访问密钥）

- daemon 的 HTTP/SSE API 采用**静态访问密钥**鉴权，不再有任何托管登录 / 托管账号体系（`/auth/*` 登录流程与移动端 App 均已移除）。
- 客户端在请求头中以 `Authorization: Bearer <key>` 携带密钥；密钥来源为配置文件 `~/.rustcode/config.toml` 的静态访问密钥字段 `access_key`（本变更已实现，详见 `docs/config.example.toml`），或环境变量 `RUSTCODE_ACCESS_KEY` / `RUSTCODE_DAEMON_TOKEN`。
- 这是本地身份体系的一部分：密钥由用户自行持有，不依赖任何外部平台、OAuth 或第三方账号；`access_key` 与 `RUSTCODE_ACCESS_KEY` / `RUSTCODE_DAEMON_TOKEN` 的优先级以 `docs/config.example.toml` 的配置契约为准。

### 7. 远程访问（frp 风格反向隧道）

- daemon 可通过可配置的**反向隧道中继**把本地 webui / HTTP·SSE API 暴露到公网：由环境变量 `RUSTCODE_ENABLE_TUNNEL=1` 开启（默认关闭），并由 `RUSTCODE_TUNNEL_RELAY` 指定中继地址（如 `wss://your-relay.example.com`）。开启后 daemon 作为反向代理，把中继转发来的请求回源到本地 webui / API。
- 远程客户端以 `Authorization: Bearer <access_key>` 鉴权（与第 6 节静态访问密钥同一套密钥），未携带有效 Bearer 的远程请求被拒绝。
- TUI 内执行 `/tunnel` 命令启动本地隧道端点，并在终端打印：中继 URL（`RUSTCODE_TUNNEL_RELAY`）、当前 `access_key`、以及本地回源端口。子命令：`lan`（见「`/tunnel lan` 局域网模式」）、`stop`（停止隧道端点）。
- **本能力取代已移除的移动端 App 远程访问**（原 `/app <中继>` + `RUSTCODE_APP_RELAY` 方案，随移动端 App 一并移除）：隧道是平台中立、用户自托管中继的反向代理，无任何托管账号或移动端依赖。

#### 内置隧道客户端（frpc 半边）—— 设计已定，**尚未实现**

- 目标形态：当 `RUSTCODE_ENABLE_TUNNEL=1` **且** `RUSTCODE_TUNNEL_RELAY` 已设置时，`/tunnel` 在启动本地端点的同时启动内置 WebSocket 隧道客户端（frp 的 frpc 半边）：连接中继、用隧道 token 鉴权，并把中继侧入站流量转发到本地端点——无需再额外部署/运行外部客户端进程。
- 线路协议已定：`Open` / `Data` / `Close` 三种帧，均携带 stream id（`Open` 建立流、`Data` 双向搬运字节、`Close` 关闭流）。中继只按 stream id 转发字节、不解析应用数据，因此任何中继都可据此独立实现。
- **当前状态：未实现**（daemon 尚未引入 WebSocket 客户端依赖）。现阶段远程接入的可行做法是：① 同网直接用 `/tunnel lan`（见下）；② 把你自己的 frp / 外部客户端指向 `/tunnel` 打印的本地回源端口。

#### `/tunnel lan`（局域网模式）

- `/tunnel lan` 把隧道端点绑定到 `0.0.0.0`，同一局域网内的其它设备用同一套 `Authorization: Bearer <access_key>` 即可访问 webui / API。
- **不需要中继，也不需要任何额外依赖**（不依赖上面的内置 WebSocket 客户端），可立即使用；不带子命令的 `/tunnel` 仍绑定回环地址，仅本机可达。
- 绑定 `0.0.0.0` 会把端点暴露给整个局域网，请仅在可信网络中使用，并确保 `access_key` 是足够强的随机密钥。

#### UDP 打洞（未来 P2P 模式）

- 未来可选模式：借助 STUN 风格信令服务器 + UDP 打洞，让两端点对点直连，绕过中继以降低延迟。该 P2P 模式**当前不在范围内**，仅作为下一步规划记录，不在本变更实现。

## 本地检索工具(内置)

| 工具 | 用途 |
|------|------|
| `grep` | 精确符号/字符串匹配 |
| `glob` | 按文件名模式查找 |
| `read_file` | 读取文件或任意片段(offset/limit/ranges) |
| `zg`(MCP 可选) | 语义/意图检索、BM25、混合检索、rg |

## 质量门禁(G1-G8)

```text
G1  cargo fmt --check
G2  cargo clippy --workspace --all-targets
G3  cargo test --workspace
G4  ./scripts/test-headless.sh
G5  python3 scripts/acp_smoke.py
G6  遥测 SDK grep 必须 0 命中
G7  crates/scripts/.github 无 atomcode 残留
G8  docs/architecture.md 无 atomcode 残留
```

## 已知项

- `mcp::registry::tests::trust_key_golden_matches_core_algorithm` 当前为红:
  `DefaultHasher` 输出不保证跨工具链稳定,勿改测试凑绿。
- `cargo clippy` 仍有 per-crate 存量 warning(非 errors),可择机收敛。
