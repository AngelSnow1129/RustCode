# Codex / Claude 配置分析与建议

## 1. 当前 Untitled-1 内容解析
- 提供商: `hwdevspace` (华为云自定义 OpenAI-compatible 网关)
- 端点: `https://tokenhub.developer.huaweicloud.com/v2`
- 模型: `glm-5.2` / `glm-5.1` / `openpangu-2.0-flash`
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
若通过 `hwdevspace` 网关访问 Codex，则 `base_url` 保持华为云端点，`model` 改为 `codex`。

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

## 4. 与现有 `hwdevspace` 的关系
`Untitled-1` 是完整 JSON 配置（含 `models`、`provider`、`options`）。建议转为 `rustcode` 的 `config.toml` 片段:
```toml
[providers.hwdevspace]
type = "openai-compatible"
model = "glm-5.2"
base_url = "https://tokenhub.developer.huaweicloud.com/v2"
api_key = "${HWDEVSPACE_API_KEY}"
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
- `G7` (`atomcode` grep): 仍有历史名引用（合法）
- `G8` (`docs/architecture.md`): 已更新为 `rustcode-*`

## 7. 已知缺口
- `ProviderConfig` 无 `timeout` 字段（仅适配器内部默认）
- `mcp::registry::tests::trust_key_golden_matches_core_algorithm` 当前红（`DefaultHasher` 不稳定）
- `.github/workflows/build.yml` 无 fmt/clippy/test job（`ci.yml` 未建）
- `docs/telemetry.md` / `README.zh-CN.md` 第 151 行与零遥测事实矛盾（仅文档口径残留）
