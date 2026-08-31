# 解除 AtomGit 平台绑定 — 平台中立化

## 目标
让 RustCode 不与任何平台绑定：用户用自己的模型嵌入使用，无 AtomGit 网关签名、无硬编码 OAuth 主机、无平台专属 REST 工具。

## 核心洞察
atomgit 集成分两类：
1. **feature 门控的**（REST 工具 `atomgit_repo/pr/issue`、bash gate、persona 提示词）——已被 `#[cfg(feature = "atomgit")]` 保护，去掉 feature 即自动消失。
2. **非门控的检测/降级路径**（`is_atomgit_gateway`、OAuth、codingplan、签名器）——这些是"检测到 atomgit 网关才走签名，否则纯 bearer"的逻辑。让检测永远返回 false → 全部自动降级为普通 bearer auth。

## 方案：分 4 步，从低风险到高风险

### 第 1 步：关闭 atomgit Cargo feature（低风险，工具层消失）
在以下 Cargo.toml 中去掉 `atomgit` feature：
- `crates/rustcode-cli/Cargo.toml:44-45`（`rustcode-coding` 和 `rustcode-capabilities` 的 features 列表去掉 `"atomgit"`）
- `crates/rustcode-clix/Cargo.toml:13,16`（同上）
- `crates/rustcode-daemon/Cargo.toml:27,30`（同上）

效果：`atomgit_repo/pr/issue` 工具不注册、`ATOMGIT_TOOL_USAGE` 提示词不编译、bash gate 不安装、`register_atomgit_capabilities` 不调用。
保留 `rustcode-coding/Cargo.toml` 的 `atomgit` feature 定义本身（让上游/其他 fork 仍可开启），只是默认成员不再启用它。

### 第 2 步：网关签名中立化（中风险，签名路径降级）
让 `is_atomgit_gateway` 不再硬编码 atomgit.com host，改为**只认用户显式配置的网关**：
- `crates/rustcode-config/src/endpoints.rs:187` `is_codingplan_llm_gateway`：去掉硬编码的 `llm-api.atomgit.com` / `pre-llm-api-cce.atomgit.com` / `api-ai.gitcode.com` 三个 host 匹配，只保留"用户通过 `RUSTCODE_CODINGPLAN_LLM_BASE_URL` 环境变量显式配置的 base_url 才算网关"这一条路径。
- 效果：没有显式配网关 → `is_atomgit_gateway` 永远返回 false → `AtomGitProviderAuthenticator` 返回 `None` → 纯 bearer auth → 用户用自己的 API key 接任何 OpenAI 兼容端点。

### 第 3 步：OAuth/登录中立化（中高风险，登录流程泛化）
当前 `/login` 硬编码 `provider: "atomgit"` 并指向 `acs.atomgit.com`。改为：
- `crates/rustcode-config/src/endpoints.rs:68` `HOSTED_PLATFORM_SERVER`：改为空或移除，让 `platform_server()` 在未配 `RUSTCODE_PLATFORM_SERVER` 时返回错误而非硬编码 atomgit。
- `crates/rustcode-auth/src/oauth.rs:609` `provider: "atomgit"`：改为从配置读取或移除该 query 参数。
- `crates/rustcode-config/src/endpoints.rs:98` `HOSTED_CODINGPLAN_PROVIDER_PREFIX: "AtomGit"` → `"RustCode"` 或移除。
- `/login` 在未配置平台服务器时，提示用户直接用 API key 配置（`~/.rustcode/config.toml` 里写 provider），而非走 OAuth。
- codingplan 子命令：在无平台绑定时降级为"请直接配置 provider"提示。

### 第 4 步：端点常量与文档清理（低风险）
- `endpoints.rs` 的 `HOSTED_CODINGPLAN_LLM_BASE_URL`、`HOSTED_RELAY_URL` 等 atomgit.com 常量：保留为"可被环境变量覆盖的默认值"，但默认值改为空串或 gitcode 对应地址，注释说明"仅用于官方托管构建，自托管用户不受影响"。
- `HOSTED_TRUSTED_DOMAINS` / `HOSTED_TLS_FALLBACK_DOMAINS`：去掉 `atomgit.com`，只留 `gitcode.com`（或改为空，信任域由用户配）。
- 文档：AGENTS.md OBJECTIVE-3 更新为"网关签名已中立化，不再硬编码 AtomGit host"；README 说明"平台中立，自带模型即用"。

## 不做的事
- **不删 `gateway_crypto.rs` / `codingplan-crypto`**：签名基础设施保留，只是不默认绑定 atomgit host。官方构建仍可用 `codingplan-crypto` feature + 环境变量启用签名。
- **不删 `rustcode-codingplan` crate**：CodingPlan 业务逻辑保留，只是默认不连 atomgit。
- **不删 OAuth 代码**：泛化而非删除——配了 `RUSTCODE_PLATFORM_SERVER` 仍可走 OAuth。
- **不删 `atomgit` feature 定义**：保留开关，让需要的人可重新启用。

## 验证
1. `cargo check --workspace` 通过（去掉 atomgit feature 后）。
2. 默认配置下启动，`is_atomgit_gateway` 对任何 URL 返回 false。
3. 用户在 config.toml 配 `[providers.openai]` + `api_key` → 可直接用，无 OAuth、无签名。
4. `RUSTCODE_PLATFORM_SERVER` + `RUSTCODE_CODINGPLAN_LLM_BASE_URL` 显式配置 → OAuth + 签名仍可用（向后兼容）。
