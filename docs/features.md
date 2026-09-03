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
