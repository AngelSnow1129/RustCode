# Fork 重构总结(RustCode)

本仓库是 `https://gitcode.com/SecLab/RustCode` 的二次开发 fork。已交付三个目标:

1. [OBJECTIVE-1] 产品改名 `atomcode` -> `rustcode`(crate 名、二进制名、配置目录、环境变量)。
2. [OBJECTIVE-2] 零遥测 —— 删除 `rustcode-telemetry` crate,移除全部上报逻辑。
3. [OBJECTIVE-3] LLM provider 解耦 —— 支持自建 OpenAI/Anthropic endpoint。

## 1. 改名(rustcode)

- 13 个 crate 由 `atomcode-*` 改名为 `rustcode-*`(例如 `rustcode-cli`、`rustcode-daemon`、`rustcode-coding`、`rustcode-kernel`、`rustcode-capabilities`、`rustcode-config`)。
- 二进制:`atomcode` -> `rustcode`,`atomcode-daemon` -> `rustcode-daemon`,`atomcodex` -> `rustcodex`。
- 配置目录 `.rustcode` -> `.rustcode`;环境变量 `ATOMCODE_*` -> `RUSTCODE_*`(集中在 `rustcode-config/src/endpoints.rs`)。
- 线上契约(wire-contract)的 key 迁移到 `rustcode.*` 命名空间(不再做旧 `~/.rustcode` 会话的兼容读取),依据已批准的“全新开始”决策。
- 代码内的产品字符串已更新;OpenRouter 署名现在指向 `rustcode` / 本 fork 仓库。

## 2. 零遥测

- 已删除 `crates/rustcode-telemetry/`。
- 已在 `cli`、`daemon`、`coding`、`auth`、`clix`、`tuix`、`config` 中移除 `Telemetry`/`Event`/`CurrentContext`/`track`/`install_panic_hook` 以及 panic 上报。
- 已移除 `--no-telemetry` flag、`telemetry` CLI 子命令、`telemetry = {}` 配置段以及所有 `rustcode_telemetry::` 引用(在 `crates/*/src` 中现为 0 处)。
- 其他代码仍需要的纯非上报类型(`SessionMode`、`RepoOrigin`、`detect_repo_origin`)已迁至 `rustcode-config/src/session_mode.rs` —— 无网络、无事件。
- 崩溃处理保留只写 stderr 的 panic printer;不会有任何数据发到机器之外。

## 3. LLM provider 解耦

- Provider factory(`rustcode-coding/src/provider_factory.rs`)现在的分发映射为:
  - `claude | anthropic | anthropic-compatible` -> 走 Anthropic adapter(`/v1/messages`)
  - `ollama` -> 走 Ollama adapter
  - `openai | openai-compatible | _` -> 走 OpenAI-compatible adapter(`/v1/chat/completions`)
  (此前 `anthropic-compatible` 会落进 OpenAI 的兜底分支 —— 已修复)
- AtomGit 请求签名现在由 `is_atomgit_gateway(base_url)` 控制:只有 `atomgit`/`relay` 主机使用上游签名器;其他(自建 / 第三方)endpoint 一律使用普通的 `bearer_auth(api_key)`。
- 新增 `ProviderConfig` / `ProviderAccountConfig` / `ResolvedModelConfig` / `CodingAgentConfig` / `CodingRuntimeConfig` 字段:
  - `extra_headers: Option<HashMap<String,String>>` —— 自定义每请求头(网关鉴权/租户),两个 adapter 都生效,且从不记录到日志。
  - `proxy: Option<String>` —— 按 provider 配置的正向代理,作用于 reqwest client 构建,并遵循 `skip_tls_verify`。
- 自建 endpoint 的模板见 `docs/config.example.toml` 中的 `[providers.my-openai-gw]` / `[providers.my-anthropic-gw]`。

## 许可证与合规

- `docs/ORIGINAL_LICENSE.md` —— 放置/声明说明(不含许可证原文);上游 MIT 许可证原文位于 `docs/UPSTREAM_RUSTCODE_LICENSE.md`(Copyright (c) 2026 Yubang Xu)。
- `docs/THIRD_PARTY_NOTICES.md` —— 第三方/继承组件的声明。
- `docs/UPSTREAM_CREDITS.md` —— fork 来源与改动摘要。
- 根目录 `LICENSE` 仍为 MIT。

## 验证

```
export PATH=/root/.cargo/bin:$PATH
cargo check --workspace                 # [CHECK] passes (warnings only)
cargo test  -p rustcode-config          # [CHECK] passes
cargo test  -p rustcode-capabilities    # [CHECK] passes
cargo test  -p rustcode-coding --lib    # [CHECK] 420 passed
```

说明:
- 在受限的 CI runner 上,完整 `cargo test --workspace` 在链接最大的集成测试二进制时可能 OOM(属环境问题,不是代码错误)。建议按 crate 用 `-p` 分别运行;`--lib` 可缩小链接规模。
- `provider_factory` 的分发已由 `rustcode-coding` 单元测试覆盖;可在 `rustcode-capabilities/tests` 下补一个 mock-provider SSE 测试,断言 `openai-compatible` / `anthropic-compatible` 在自定义 `base_url` 下进入正确的 adapter。

## 手动冒烟测试(自建 endpoint)

1. 在 `~/.rustcode/config.toml` 中写入:
   ```toml
   default_provider = "my-openai-gw"
   [providers.my-openai-gw]
   type = "openai-compatible"
   api_key = "env:MY_KEY"
   model = "your-model"
   base_url = "https://your-gateway.example.com/v1"
   ```
2. `export MY_KEY=...`
3. `rustcode "explain this repo"` —— 流量应当打到你的网关,没有上游 AtomGit 签名,也没有遥测。

## 已知缺口 / 后续项

- `extensions/` 下的扩展 crate(vscode / jetbrains)仍在引用 `[RustCode]` 日志和旧的二进制名;它们不在 core 改名范围内,若 fork 要随附这些扩展,应在后续补丁中修正。
- 上游的 `docs/telemetry.md` 对本 fork 而言已过时(遥测已移除);文件仍保留在磁盘上,但不再描述实际发布行为。
