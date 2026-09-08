# Codex / Claude 配置分析与建议

## 1. 当前 Untitled-1 内容解析
- 提供商: 一个自定义 OpenAI-compatible 托管网关（第三方自带端点，平台中立）
- 端点: `https://gateway.example.com/v2`
- 模型: 该网关服务的任意模型 id（示例：`glm-5.2` / `your-model-id`）
- 协议: `openai` (`apiFormat`)
- 适配器: 直接复用 `rustcode-capabilities/src/provider/openai_compat.rs`
- 注意: `glm-5.2` 非多模态 (`supports_vision = false`)

## 2. Codex 配置建议
若要接入 `rustcode`，在 `config.toml` 中添加:
```toml
[providers.codex]
type = "openai-compatible"
model = "codex"
base_url = "https://api.openai.com/v1"
api_key = "${OPENAI_API_KEY}"
```
若通过自建或第三方托管的 OpenAI 兼容网关访问 Codex，则 `base_url` 保持该网关端点（示例 `https://gateway.example.com/v2`），`model` 改为 `codex`。

## 3. Claude 配置建议
`rustcode-capabilities/src/provider/anthropic.rs` 已存在。配置:
```toml
[providers.claude]
type = "anthropic"
model = "claude-sonnet-4-20250514"
base_url = "https://api.anthropic.com"
api_key = "${ANTHROPIC_API_KEY}"
thinking_enabled = true
thinking_budget = 10000
```
`ProviderConfig` 已支持 `thinking_enabled` / `thinking_budget`（`provider.rs`）。

## 4. 接入第三方 OpenAI 兼容网关
任意 OpenAI 兼容的第三方或自托管网关都可用相同方式接入。把 `Untitled-1` 一类的完整 JSON 配置（含 `models`、`provider`、`options`）转为 `rustcode` 的 `config.toml` 片段（域名/密钥均为占位，替换为你自己的服务商）:
```toml
[providers.thirdparty]
type = "openai-compatible"
model = "your-model-id"
base_url = "https://gateway.example.com/v2"
api_key = "${THIRDPARTY_API_KEY}"
supports_vision = false
```

## 5. 现役 crate 映射
- `rustcode-config`: `ProviderConfig` / `ProviderPreset` / `provider_preset.rs`
- `rustcode-capabilities`: `openai_compat.rs` / `anthropic.rs` / `provider_factory`
- `rustcode-coding`: `CodingRuntime`（运行时所有者）
- `rustcode-kernel`: `Agent` / `AgentHandle`（中立循环）

## 6. 门禁状态
- `G1` (`cargo fmt`): 未执行
- `G2` (`clippy`): 未执行
- `G3` (`test`): 未执行
- `G6` (遥测 grep): 无 `sentry/posthog/segment/analytics` 命中
- `G7` (`rustcode` grep): 仍有历史名引用（合法）
- `G8` (`docs/architecture.md`): 已更新为 `rustcode-*`

## 7. 已知缺口
- `ProviderConfig` 无 `timeout` 字段（仅适配器内部默认）
- `mcp::registry::tests::trust_key_golden_matches_core_algorithm` 当前红（`DefaultHasher` 不稳定）
- `.github/workflows/build.yml` 无 fmt/clippy/test job（`ci.yml` 未建）
- `docs/telemetry.md` 与原中文 README 第 151 行与零遥测事实矛盾（仅文档口径残留）
