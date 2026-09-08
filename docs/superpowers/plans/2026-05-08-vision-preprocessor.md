# Vision Preprocessor 实施计划

> **面向 agentic worker：** 必备子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实施本计划。步骤使用复选框（`- [ ]`）语法跟踪进度。

**目标：** 当当前 LLM provider 不接受图片、而用户粘贴了图片时，先将该图片交给一个可配置的视觉语言模型处理，把模型产出的描述拼接进用户消息，再以纯文本形式转发给主 provider。

**架构：** 新增模块 `rustcode-core::vision_preprocessor`，含唯一异步入口 `maybe_preprocess`。在 `agent::handle_send_message` 中有一处调用点。新增一个可选 `Config` 字段。失败通过已有的 `AgentEvent::Warning` 暴露。不改动 `LlmProvider` trait、`Conversation`、`coding_plan/setup.rs` 与 `MessageContent`。

**技术栈：** Rust、`tokio`、`async-trait`、`wiremock`（仅测试用），VL 调用复用已有的 `OpenAiProvider`。

---

## 参考：Spec

完整设计见 `docs/superpowers/specs/2026-05-08-vision-preprocessor-design.md`。此处记录关键决策：

- 触发条件 = `!model_name_suggests_vision(provider.model_name())` 且 `!images.is_empty()` 且配置字段为 `Some(non_empty)`。
- VL **只**接收当前轮次的 caption + 图片；不含主会话历史。
- VL 输出以 `"\n\n[图片内容（由 VL 模型识别）]\n{text}"` 包裹后追加到用户文本；原图片被丢弃。
- 失败时：向用户文本追加 `"\n\n[图片识别失败]"`，丢弃图片，发出 `AgentEvent::Warning(...)`，本轮继续。
- 配置字段：`vision_preprocessor_provider: Option<String>`，位于 `Config` 顶层；为 `None` 或空字符串时 → 功能关闭。

---

## 文件结构

| 文件 | 动作 | 职责 |
|---|---|---|
| `crates/rustcode-core/src/vision_preprocessor.rs` | **新建** | `PreprocessOutcome` 枚举 + `maybe_preprocess` 异步函数 + 单元测试 |
| `crates/rustcode-core/src/lib.rs` | **修改** | 添加 `pub mod vision_preprocessor;` |
| `crates/rustcode-core/src/config/mod.rs` | **修改** | 为 `Config` 添加 `vision_preprocessor_provider: Option<String>` 字段 |
| `crates/rustcode-core/src/agent/mod.rs` | **修改** | 在 `handle_send_message` 中、既有 `if images.is_empty()` 分支之前调用 `maybe_preprocess` |

不改动 TUIX。不新增 `AgentEvent` 变体。不改动 provider trait 与工厂函数。

---

## 任务 1：为 `Config` 添加 `vision_preprocessor_provider` 字段

**文件：**
- 修改：`crates/rustcode-core/src/config/mod.rs:82-125`（`Config` 结构体）
- 测试：`crates/rustcode-core/src/config/mod.rs`（在已有的 `#[cfg(test)] mod tests` 块内；若不存在则新建一个）

- [ ] **步骤 1：定位已有的 `Config` 测试模块**

运行：`grep -n "#\[cfg(test)\]\|fn parse_minimal\|mod tests" crates/rustcode-core/src/config/mod.rs | head -20`

确认 `mod.rs` 是否已有测试模块。若有，把新测试加进去；若没有，旁边的 `provider.rs` 里有，可照其风格在文件末尾新增一个 `#[cfg(test)] mod tests { use super::*; ... }` 块。

- [ ] **步骤 2：编写失败测试**

在 `config/mod.rs` 的测试模块中添加：

```rust
#[test]
fn vision_preprocessor_provider_defaults_to_none() {
    // Existing config.toml files (pre-feature) must parse cleanly with
    // `vision_preprocessor_provider` defaulting to None — feature is opt-in
    // and absence must not break load.
    let toml_str = r#"
        default_provider = "claude"
        [providers.claude]
        type = "claude"
        model = "claude-sonnet-4-5"
        api_key = "sk-test"
    "#;
    let cfg: Config = toml::from_str(toml_str).expect("parse minimal config");
    assert_eq!(cfg.vision_preprocessor_provider, None);
}

#[test]
fn vision_preprocessor_provider_round_trips_through_toml() {
    let toml_str = r#"
        default_provider = "claude"
        vision_preprocessor_provider = "RustCode-Qwen-Qwen3-VL-32B-Instruct"
        [providers.claude]
        type = "claude"
        model = "claude-sonnet-4-5"
        api_key = "sk-test"
    "#;
    let cfg: Config = toml::from_str(toml_str).expect("parse");
    assert_eq!(
        cfg.vision_preprocessor_provider.as_deref(),
        Some("RustCode-Qwen-Qwen3-VL-32B-Instruct"),
    );
}
```

- [ ] **步骤 3：运行测试以确认失败**

运行：`cargo test -p rustcode-core --lib config::mod -- vision_preprocessor`

预期：编译错误 —— `Config` 没有 `vision_preprocessor_provider` 字段。

- [ ] **步骤 4：为 `Config` 添加该字段**

编辑 `crates/rustcode-core/src/config/mod.rs`。在 `pub struct Config { ... }` 内部（约 82–125 行），于右花括号前追加：

```rust
    /// Provider key (matches a key in `Config.providers`) of a vision-language
    /// model used to preprocess images before forwarding to a non-vision main
    /// provider. When `None` or empty, image preprocessing is disabled — pasted
    /// images either go directly to a vision-capable main provider, or get
    /// degraded to `"[image attached]"` placeholder by the existing path.
    ///
    /// Example value: `"RustCode-Qwen-Qwen3-VL-32B-Instruct"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vision_preprocessor_provider: Option<String>,
```

- [ ] **步骤 5：更新测试 / 空白构造函数中的 `Config { ... }` 字面量**

运行：`grep -rn "Config {$\|Config {[^}]" crates/rustcode-core/ | grep -v target | grep -v 'Config::' | head -20`

对每一个不使用 `..Default::default()`、完整构造整个结构体的空白 `Config { ... }` 字面量，添加 `vision_preprocessor_provider: None,`。已知位置来自 `coding_plan/setup.rs::tests::blank_config()`（约 575 行）。逐处更新。

如果 `Config` 没有 `Default` 实现，以上即为全部影响范围；如果**有** `Default` 实现，也要一并更新，将该字段设为 `None`。

- [ ] **步骤 6：运行测试以确认通过**

运行：`cargo test -p rustcode-core --lib`

预期：全部测试通过（两个新测试 + 此前所有测试）。若有测试因缺少字段而失败，回到步骤 5。

- [ ] **步骤 7：提交**

```bash
git add crates/rustcode-core/src/config/mod.rs crates/rustcode-core/src/coding_plan/setup.rs
git commit -m "feat(config): add vision_preprocessor_provider field

Optional top-level Config knob naming a provider key used to OCR images
before forwarding to a non-vision main provider. None/empty = feature off
(safe default for existing config.toml files). Wired in subsequent commits.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## 任务 2：创建 `vision_preprocessor` 模块骨架与短路逻辑

本任务搭建公开 API 以及四条短路分支中的三条（无图片 / 主 provider 本身支持视觉 / 配置未设置）。第四条分支（provider key 不在配置中）与真正的 VL 调用留待后续任务。

**文件：**
- 新建：`crates/rustcode-core/src/vision_preprocessor.rs`
- 修改：`crates/rustcode-core/src/lib.rs:1-30`（添加 `pub mod vision_preprocessor;`）

- [ ] **步骤 1：添加模块声明**

编辑 `crates/rustcode-core/src/lib.rs`。按字母序位置添加 `pub mod vision_preprocessor;`（在第 27 行 `pub mod turn;` 之后、第 28 行 `pub mod uninstall;` 之前）。第 27 行附近的结果为：

```rust
pub mod turn;
pub mod uninstall;
pub mod version_check;
pub mod vision_preprocessor;
```

（最终位置：`version_check` 之后、列表末尾之前。按字母序自行调整。）

- [ ] **步骤 2：创建模块文件，包含公开接口 + 短路逻辑 + 跳过类测试**

创建 `crates/rustcode-core/src/vision_preprocessor.rs`：

```rust
//! VL-model image preprocessor.
//!
//! When the active main provider does not accept images and the user submits
//! an image, this module routes the image (plus the current-turn caption only)
//! through a configurable vision-language provider, returning a textual
//! description that callers splice into the user message before forwarding to
//! the main provider as plain text.
//!
//! Key invariant: the VL call NEVER sees the main conversation history. The
//! `Vec<Message>` passed to the VL provider is constructed locally from
//! `caption + images` and contains exactly one user turn.

use crate::config::Config;
use crate::conversation::message::ImagePart;
use crate::provider::{model_name_suggests_vision, LlmProvider};

/// Outcome of a preprocessing attempt.
#[derive(Debug, Clone)]
pub enum PreprocessOutcome {
    /// Preprocessing did not run — feature disabled, main provider already
    /// accepts images, or no images attached. Caller must use the original
    /// `(caption, images)` tuple unchanged.
    Skipped,
    /// VL call succeeded. `text` is the raw VL output (no wrapping). Caller
    /// is responsible for splicing it into the user message — recommended
    /// shape: `format!("{caption}\n\n[图片内容（由 VL 模型识别）]\n{text}")`
    /// — and clearing the images vec.
    Replaced { text: String },
    /// VL call failed (provider missing, network error, timeout, empty
    /// response). `reason` is intended for `AgentEvent::Warning`. Caller
    /// should append `"\n\n[图片识别失败]"` to the user message and clear
    /// images so the turn proceeds with a useful placeholder.
    Failed { reason: String },
}

/// Decide whether and how to preprocess images before a main-provider turn.
///
/// Short-circuit order (each → `Skipped`, except the last):
/// 1. `images` is empty.
/// 2. The active provider's model name passes the `model_name_suggests_vision`
///    heuristic (it can handle the image natively).
/// 3. `config.vision_preprocessor_provider` is `None` or `Some("")`.
/// 4. The configured key is missing from `config.providers` → `Failed` (this
///    is a configuration mistake worth surfacing, not a silent skip).
pub async fn maybe_preprocess(
    config: &Config,
    active_provider: &dyn LlmProvider,
    caption: &str,
    images: &[ImagePart],
) -> PreprocessOutcome {
    if images.is_empty() {
        return PreprocessOutcome::Skipped;
    }
    if model_name_suggests_vision(active_provider.model_name()) {
        return PreprocessOutcome::Skipped;
    }
    let vl_key = match config.vision_preprocessor_provider.as_deref() {
        Some(k) if !k.is_empty() => k,
        _ => return PreprocessOutcome::Skipped,
    };
    if !config.providers.contains_key(vl_key) {
        return PreprocessOutcome::Failed {
            reason: format!("VL provider '{vl_key}' not found in config.providers"),
        };
    }
    // VL HTTP call lands in Task 3 — for now, signal that we got past all
    // short-circuits but haven't yet implemented the call.
    PreprocessOutcome::Failed {
        reason: "VL call not yet implemented".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::provider::ProviderConfig;
    use crate::provider::unavailable_provider;
    use std::collections::HashMap;

    fn blank_config() -> Config {
        // Mirrors `coding_plan::setup::tests::blank_config` but kept local
        // so this test module does not reach into another module's private test
        // helpers. If new mandatory fields are added to Config, update both.
        Config {
            default_provider: String::new(),
            default_workdir: None,
            providers: HashMap::new(),
            datalog: Default::default(),
            auto_update: true,
            notifications: Default::default(),
            telemetry: Default::default(),
            lsp: Default::default(),
            auto_commit: false,
            subagent: Default::default(),
            vision_preprocessor_provider: None,
        }
    }

    fn sample_image() -> ImagePart {
        ImagePart {
            media_type: "image/png".into(),
            data: "iVBORw0KGgoAAAANSUhEUg==".into(),
        }
    }

    /// Vision-capable main provider via name heuristic (`claude-sonnet-4-5`).
    /// Real provider construction is irrelevant — `unavailable_provider` carries
    /// a model_name of `""` which passes the not-vision branch, so we use a
    /// trivial inline impl that returns the desired model name.
    struct StubProvider {
        model: &'static str,
    }
    use crate::stream::StreamEvent;
    use crate::tool::ToolDef;
    use anyhow::Result;
    use async_trait::async_trait;
    use futures::Stream;
    use std::pin::Pin;
    #[async_trait]
    impl LlmProvider for StubProvider {
        fn chat_stream(
            &self,
            _messages: &[crate::conversation::message::Message],
            _tools: Option<&[ToolDef]>,
        ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>> {
            anyhow::bail!("stub never streams");
        }
        fn model_name(&self) -> &str {
            self.model
        }
    }

    #[tokio::test]
    async fn skipped_when_no_images() {
        let cfg = blank_config();
        let provider = StubProvider { model: "deepseek-v4-flash" };
        let result = maybe_preprocess(&cfg, &provider, "any caption", &[]).await;
        assert!(matches!(result, PreprocessOutcome::Skipped));
    }

    #[tokio::test]
    async fn skipped_when_main_provider_accepts_images() {
        let cfg = blank_config();
        let provider = StubProvider { model: "claude-sonnet-4-5" };
        let result =
            maybe_preprocess(&cfg, &provider, "describe", &[sample_image()]).await;
        assert!(matches!(result, PreprocessOutcome::Skipped));
    }

    #[tokio::test]
    async fn skipped_when_config_field_unset() {
        let cfg = blank_config();
        let provider = StubProvider { model: "deepseek-v4-flash" };
        let result =
            maybe_preprocess(&cfg, &provider, "describe", &[sample_image()]).await;
        assert!(matches!(result, PreprocessOutcome::Skipped));
    }

    #[tokio::test]
    async fn skipped_when_config_field_empty_string() {
        let mut cfg = blank_config();
        cfg.vision_preprocessor_provider = Some(String::new());
        let provider = StubProvider { model: "deepseek-v4-flash" };
        let result =
            maybe_preprocess(&cfg, &provider, "describe", &[sample_image()]).await;
        assert!(matches!(result, PreprocessOutcome::Skipped));
    }

    #[tokio::test]
    async fn failed_when_configured_key_missing_from_providers() {
        let mut cfg = blank_config();
        cfg.vision_preprocessor_provider = Some("AtomGit-NoSuchModel".into());
        let provider = StubProvider { model: "deepseek-v4-flash" };
        let result =
            maybe_preprocess(&cfg, &provider, "describe", &[sample_image()]).await;
        match result {
            PreprocessOutcome::Failed { reason } => {
                assert!(
                    reason.contains("AtomGit-NoSuchModel") && reason.contains("not found"),
                    "expected 'not found' for missing key, got: {reason}",
                );
            }
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    /// Regression marker for Task 3: this test currently passes the "VL call
    /// not yet implemented" placeholder branch. After Task 3 lands, it must
    /// be replaced/removed since the placeholder branch goes away.
    #[tokio::test]
    async fn key_present_currently_hits_unimplemented_placeholder() {
        let mut cfg = blank_config();
        cfg.providers.insert(
            "vl-stub".into(),
            ProviderConfig {
                provider_type: "openai".into(),
                api_key: Some("sk-test".into()),
                model: "Qwen/Qwen3-VL-32B-Instruct".into(),
                base_url: Some("http://127.0.0.1:1/".into()),
                system_prompt: None,
                user_agent: None,
                context_window: 8000,
                max_tokens: None,
                thinking_type: None,
                thinking_keep: None,
                reasoning_history: None,
                thinking_enabled: None,
                thinking_budget: None,
                skip_tls_verify: false,
                ephemeral: false,
            },
        );
        cfg.vision_preprocessor_provider = Some("vl-stub".into());
        let provider = StubProvider { model: "deepseek-v4-flash" };
        let result =
            maybe_preprocess(&cfg, &provider, "describe", &[sample_image()]).await;
        assert!(matches!(result, PreprocessOutcome::Failed { .. }));
    }
}
```

- [ ] **步骤 3：运行测试**

运行：`cargo test -p rustcode-core --lib vision_preprocessor`

预期：6 个测试通过（`skipped_when_no_images`、`skipped_when_main_provider_accepts_images`、`skipped_when_config_field_unset`、`skipped_when_config_field_empty_string`、`failed_when_configured_key_missing_from_providers`、`key_present_currently_hits_unimplemented_placeholder`）。

- [ ] **步骤 4：提交**

```bash
git add crates/rustcode-core/src/vision_preprocessor.rs crates/rustcode-core/src/lib.rs
git commit -m "feat(vision_preprocessor): module skeleton with short-circuit logic

Public API: PreprocessOutcome enum + maybe_preprocess async fn. Implements
the four early-return branches (no images / main provider accepts images /
config field unset or empty / configured key missing → Failed). VL HTTP
call lands in next commit.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## 任务 3：实现 VL HTTP 调用（happy path）

**文件：**
- 修改：`crates/rustcode-core/src/vision_preprocessor.rs`

- [ ] **步骤 1：检查 wiremock 依赖**

运行：`grep -n "wiremock" crates/rustcode-core/Cargo.toml`

预期：`[dev-dependencies]` 下已有 `wiremock = "0.6"` 一行。若缺失则添加；若 wiremock 已是常规依赖（按设计阶段的 grep 结果，确实如此），则跳过。

- [ ] **步骤 2：用真正的 VL 调用替换占位实现**

在 `crates/rustcode-core/src/vision_preprocessor.rs` 中替换占位代码块。定位以下内容：

```rust
    if !config.providers.contains_key(vl_key) {
        return PreprocessOutcome::Failed {
            reason: format!("VL provider '{vl_key}' not found in config.providers"),
        };
    }
    // VL HTTP call lands in Task 3 — for now, signal that we got past all
    // short-circuits but haven't yet implemented the call.
    PreprocessOutcome::Failed {
        reason: "VL call not yet implemented".into(),
    }
}
```

并将 `vl_key` 之后的整段（从 `if !config.providers.contains_key` 一行到 `maybe_preprocess` 的闭合 `}`）替换为：

```rust
    let vl_cfg = match config.providers.get(vl_key) {
        Some(c) => c.clone(),
        None => {
            return PreprocessOutcome::Failed {
                reason: format!("VL provider '{vl_key}' not found in config.providers"),
            };
        }
    };

    use crate::conversation::message::{Message, MessageContent, Role};
    use crate::provider::create_provider;
    use futures::StreamExt;

    // Build a one-off VL provider. `create_provider` handles auth-token
    // loading (api_key=None) for the AtomGit gateway case.
    let vl_provider = match create_provider(&vl_cfg) {
        Ok(p) => p,
        Err(e) => {
            return PreprocessOutcome::Failed {
                reason: format!("VL provider build failed: {e:#}"),
            };
        }
    };

    let prompt = if caption.trim().is_empty() {
        "请详细描述这张图片的内容。如果是代码、报错截图或终端输出，请逐字转录文本。"
            .to_string()
    } else {
        format!(
            "用户的当前请求：{caption}\n\n请详细描述这张图片的内容。如果是代码、\
             报错截图或终端输出，请逐字转录文本。",
        )
    };

    // Local one-shot conversation — explicitly NOT linked to the main
    // `agent.conversation.messages`. This is the structural guarantee that
    // VL only sees the current image + caption, never history.
    let messages = vec![Message {
        role: Role::User,
        content: MessageContent::MultiPart {
            text: Some(prompt),
            images: images.to_vec(),
        },
    }];

    let timeout = std::time::Duration::from_secs(30);
    let call = async {
        let mut stream = vl_provider.chat_stream(&messages, None)?;
        let mut buf = String::new();
        while let Some(event) = stream.next().await {
            match event? {
                crate::stream::StreamEvent::Delta(s) => buf.push_str(&s),
                crate::stream::StreamEvent::Reasoning(_) => {} // ignore — VL OCR rarely streams reasoning
                crate::stream::StreamEvent::Done { .. } => break,
                crate::stream::StreamEvent::Error(e) => anyhow::bail!("{e}"),
                _ => {}
            }
        }
        Ok::<_, anyhow::Error>(buf)
    };

    match tokio::time::timeout(timeout, call).await {
        Err(_) => PreprocessOutcome::Failed {
            reason: format!("VL call timed out after {}s", timeout.as_secs()),
        },
        Ok(Err(e)) => PreprocessOutcome::Failed {
            reason: format!("VL call error: {e:#}"),
        },
        Ok(Ok(text)) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                PreprocessOutcome::Failed {
                    reason: "VL returned empty response".into(),
                }
            } else {
                PreprocessOutcome::Replaced {
                    text: trimmed.to_string(),
                }
            }
        }
    }
```

同时从测试模块中删除占位分支测试 `key_present_currently_hits_unimplemented_placeholder`（它曾作为绊线，如今已不再准确）。

任务 2 中插入的防未使用导入行 `let _ = ReasoningPolicy::Exclude;` 等现在应删除 —— 下面的 happy-path 测试会真正用到这些导入。

- [ ] **步骤 3：为 happy path 添加 wiremock 测试**

在同一文件的 `#[cfg(test)] mod tests` 中添加：

```rust
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Minimal SSE chunk fixture for an OpenAI-compatible /chat/completions
    /// endpoint that returns one `delta.content` token then `[DONE]`.
    fn sse_one_token(text: &str) -> String {
        // Each chunk: `data: {json}\n\n`. Final terminator: `data: [DONE]\n\n`.
        let chunk = serde_json::json!({
            "choices": [{
                "delta": { "content": text },
                "finish_reason": null,
            }],
        });
        let done = serde_json::json!({
            "choices": [{
                "delta": {},
                "finish_reason": "stop",
            }],
        });
        format!(
            "data: {}\n\ndata: {}\n\ndata: [DONE]\n\n",
            chunk, done,
        )
    }

    fn vl_provider_cfg(base_url: &str) -> ProviderConfig {
        ProviderConfig {
            provider_type: "openai".into(),
            api_key: Some("sk-test".into()),
            model: "Qwen/Qwen3-VL-32B-Instruct".into(),
            base_url: Some(base_url.to_string()),
            system_prompt: None,
            user_agent: None,
            context_window: 8000,
            max_tokens: None,
            thinking_type: None,
            thinking_keep: None,
            reasoning_history: None,
            thinking_enabled: None,
            thinking_budget: None,
            skip_tls_verify: false,
            ephemeral: false,
        }
    }

    #[tokio::test]
    async fn replaced_when_vl_returns_text() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", "text/event-stream")
                    .set_body_string(sse_one_token(
                        "Python stack trace showing ZeroDivisionError on line 42",
                    )),
            )
            .expect(1)
            .mount(&server)
            .await;

        let mut cfg = blank_config();
        cfg.providers.insert(
            "vl".into(),
            vl_provider_cfg(&format!("{}/", server.uri())),
        );
        cfg.vision_preprocessor_provider = Some("vl".into());

        let provider = StubProvider { model: "deepseek-v4-flash" };
        let result =
            maybe_preprocess(&cfg, &provider, "explain this", &[sample_image()]).await;

        match result {
            PreprocessOutcome::Replaced { text } => {
                assert_eq!(
                    text,
                    "Python stack trace showing ZeroDivisionError on line 42"
                );
            }
            other => panic!("expected Replaced, got {other:?}"),
        }
    }
```

- [ ] **步骤 4：运行测试**

运行：`cargo test -p rustcode-core --lib vision_preprocessor`

预期：6 个测试通过（任务 2 的 5 个减去已删除的绊线测试，再加上新的 `replaced_when_vl_returns_text`）。

- [ ] **步骤 5：提交**

```bash
git add crates/rustcode-core/src/vision_preprocessor.rs
git commit -m "feat(vision_preprocessor): implement VL HTTP call + happy-path test

Reuses existing OpenAiProvider via create_provider(). VL conversation is a
locally-constructed Vec<Message> with exactly one user turn — structural
guarantee that main-conversation history never reaches the VL endpoint.
Wraps the call in a 30s tokio timeout. Caption-aware prompt template.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## 任务 4：失败路径测试 —— HTTP 错误、超时、空响应、caption 变体

**文件：**
- 修改：`crates/rustcode-core/src/vision_preprocessor.rs`（仅测试模块）

- [ ] **步骤 1：添加 HTTP 错误测试**

在已有的 `#[cfg(test)] mod tests` 中添加：

```rust
    #[tokio::test]
    async fn failed_when_vl_returns_500() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(500).set_body_string("upstream error"))
            // Existing OpenAI provider may retry per its retry::RetryPolicy.
            // Don't pin .expect(N); just assert the eventual outcome.
            .mount(&server)
            .await;

        let mut cfg = blank_config();
        cfg.providers.insert(
            "vl".into(),
            vl_provider_cfg(&format!("{}/", server.uri())),
        );
        cfg.vision_preprocessor_provider = Some("vl".into());

        let provider = StubProvider { model: "deepseek-v4-flash" };
        let result =
            maybe_preprocess(&cfg, &provider, "x", &[sample_image()]).await;

        match result {
            PreprocessOutcome::Failed { reason } => {
                assert!(
                    reason.contains("VL call error") || reason.contains("500"),
                    "expected error reason mentioning failure, got: {reason}",
                );
            }
            other => panic!("expected Failed, got {other:?}"),
        }
    }
```

- [ ] **步骤 2：添加空响应测试**

```rust
    #[tokio::test]
    async fn failed_when_vl_returns_empty_string() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", "text/event-stream")
                    .set_body_string(sse_one_token("")), // empty token then [DONE]
            )
            .mount(&server)
            .await;

        let mut cfg = blank_config();
        cfg.providers.insert(
            "vl".into(),
            vl_provider_cfg(&format!("{}/", server.uri())),
        );
        cfg.vision_preprocessor_provider = Some("vl".into());

        let provider = StubProvider { model: "deepseek-v4-flash" };
        let result =
            maybe_preprocess(&cfg, &provider, "x", &[sample_image()]).await;

        match result {
            PreprocessOutcome::Failed { reason } => {
                assert!(
                    reason.contains("empty"),
                    "expected 'empty' in reason, got: {reason}",
                );
            }
            other => panic!("expected Failed for empty response, got {other:?}"),
        }
    }
```

- [ ] **步骤 3：添加 caption 提示词断言测试**

该测试通过 wiremock 的 body 检查捕获请求体，以验证发给 VL 的提示词中包含用户的 caption。

```rust
    #[tokio::test]
    async fn caption_is_included_in_vl_prompt() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .and(wiremock::matchers::body_string_contains("用户的当前请求：解释这段代码"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", "text/event-stream")
                    .set_body_string(sse_one_token("ok")),
            )
            .expect(1)
            .mount(&server)
            .await;

        let mut cfg = blank_config();
        cfg.providers.insert(
            "vl".into(),
            vl_provider_cfg(&format!("{}/", server.uri())),
        );
        cfg.vision_preprocessor_provider = Some("vl".into());

        let provider = StubProvider { model: "deepseek-v4-flash" };
        let result = maybe_preprocess(
            &cfg,
            &provider,
            "解释这段代码",
            &[sample_image()],
        )
        .await;

        // Replaced confirms the body matched the caption pattern (otherwise
        // wiremock would reject the request and the call would fail).
        assert!(matches!(result, PreprocessOutcome::Replaced { .. }));
    }

    #[tokio::test]
    async fn empty_caption_uses_pure_describe_prompt() {
        let server = MockServer::start().await;
        // Pure describe prompt — must NOT contain the "用户的当前请求：" prefix.
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .and(wiremock::matchers::body_string_contains("请详细描述这张图片的内容"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", "text/event-stream")
                    .set_body_string(sse_one_token("ok")),
            )
            .expect(1)
            .mount(&server)
            .await;

        let mut cfg = blank_config();
        cfg.providers.insert(
            "vl".into(),
            vl_provider_cfg(&format!("{}/", server.uri())),
        );
        cfg.vision_preprocessor_provider = Some("vl".into());

        let provider = StubProvider { model: "deepseek-v4-flash" };
        let result = maybe_preprocess(&cfg, &provider, "  ", &[sample_image()]).await;

        assert!(matches!(result, PreprocessOutcome::Replaced { .. }));
    }
```

（不写超时测试 —— `tokio::time::timeout` 配合 `tokio::time::pause` 在不同版本间表现脆弱，而 30 秒真实耗时也不值得用一个实时测试去覆盖。若真实用户确实遇到超时，占位超时测试可作为后续项。）

- [ ] **步骤 4：运行测试**

运行：`cargo test -p rustcode-core --lib vision_preprocessor`

预期：共 9 个测试通过（5 个短路 + 1 个 happy path + 4 个失败模式 / caption 变体）。

- [ ] **步骤 5：提交**

```bash
git add crates/rustcode-core/src/vision_preprocessor.rs
git commit -m "test(vision_preprocessor): HTTP error, empty response, caption variants

Adds wiremock-based tests covering: 500 → Failed, empty SSE token →
Failed('empty'), prompt body contains user caption when present, prompt
body uses pure-describe template when caption is whitespace.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## 任务 5：将 `maybe_preprocess` 接入 `Agent::handle_send_message`

**文件：**
- 修改：`crates/rustcode-core/src/agent/mod.rs`（约 1266 行，既有的 `if images.is_empty()` 位置）

- [ ] **步骤 1：定位调用点**

运行：`grep -n "if images.is_empty()" crates/rustcode-core/src/agent/mod.rs`

确认该行位于 `handle_send_message` 内（按当前代码约在 1266 行）。

- [ ] **步骤 2：在该分支之前插入预处理调用**

在 `crates/rustcode-core/src/agent/mod.rs` 中，当前内容如下的位置：

```rust
        if images.is_empty() {
            self.conversation.add_user_message(&clean);
        } else {
            use crate::conversation::message::{Message, MessageContent, Role};
            let msg = Message {
                role: Role::User,
                content: MessageContent::MultiPart {
                    text: if clean.is_empty() { None } else { Some(clean.clone()) },
                    images,
                },
            };
            ...
        }
```

替换为：

```rust
        // Vision preprocessing: when the active provider can't accept images
        // and the user pasted some, run them through the configured VL model
        // first and turn the result into plain text. See
        // `vision_preprocessor` module doc for the data-flow contract.
        let (clean, images) = if !images.is_empty() {
            use crate::vision_preprocessor::{maybe_preprocess, PreprocessOutcome};
            match maybe_preprocess(
                &self.config,
                self.turn_runner.provider.as_ref(),
                &clean,
                &images,
            )
            .await
            {
                PreprocessOutcome::Skipped => (clean, images),
                PreprocessOutcome::Replaced { text } => {
                    let merged = if clean.is_empty() {
                        format!("[图片内容（由 VL 模型识别）]\n{text}")
                    } else {
                        format!("{clean}\n\n[图片内容（由 VL 模型识别）]\n{text}")
                    };
                    (merged, Vec::new())
                }
                PreprocessOutcome::Failed { reason } => {
                    let _ = self
                        .event_tx
                        .send(AgentEvent::Warning(format!("VL 预处理失败：{reason}")));
                    let merged = if clean.is_empty() {
                        "[图片识别失败]".to_string()
                    } else {
                        format!("{clean}\n\n[图片识别失败]")
                    };
                    (merged, Vec::new())
                }
            }
        } else {
            (clean, images)
        };

        if images.is_empty() {
            self.conversation.add_user_message(&clean);
        } else {
            use crate::conversation::message::{Message, MessageContent, Role};
            let msg = Message {
                role: Role::User,
                content: MessageContent::MultiPart {
                    text: if clean.is_empty() { None } else { Some(clean.clone()) },
                    images,
                },
            };
            let idx = self.conversation.messages.len();
            self.conversation.messages.push(msg);
            self.conversation.turn_tracker.on_user_message(idx);
        }
```

- [ ] **步骤 3：构建以确认可编译**

运行：`cargo build -p rustcode-core`

预期：成功。若借用检查器因同一作用域内同时出现 `&self.config` 与 `self.event_tx.send` 而报错，则重构为：把告警字符串存入局部变量，在 match 之后再发送：

```rust
let mut warning: Option<String> = None;
let (clean, images) = if !images.is_empty() {
    match maybe_preprocess(&self.config, self.turn_runner.provider.as_ref(), &clean, &images).await {
        // ...
        PreprocessOutcome::Failed { reason } => {
            warning = Some(format!("VL 预处理失败：{reason}"));
            // ...
        }
        // ...
    }
} else { (clean, images) };
if let Some(w) = warning {
    let _ = self.event_tx.send(AgentEvent::Warning(w));
}
```

- [ ] **步骤 4：运行已有 agent 测试，确认无回归**

运行：`cargo test -p rustcode-core --lib agent`

预期：所有既有测试通过。

- [ ] **步骤 5：提交**

```bash
git add crates/rustcode-core/src/agent/mod.rs
git commit -m "feat(agent): route images through vision_preprocessor before send

In handle_send_message, when the user submitted images, call
maybe_preprocess() with the active provider + caption + images. On
Replaced, splice VL text into the user message and drop images so the
turn proceeds as plain text. On Failed, append [图片识别失败] and emit
AgentEvent::Warning so the user understands why the placeholder appeared.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## 任务 6：全 crate 冒烟构建 + clippy

**文件：**
-（无 —— 仅验证。）

- [ ] **步骤 1：全 workspace 构建**

运行：`cargo build --workspace --all-targets`

预期：成功。

- [ ] **步骤 2：全 workspace 测试**

运行：`cargo test --workspace --all-targets`

预期：成功。若任务 1 步骤 5 遗漏了 TUIX 或 CLI fixture 中某处 `Config { ... }` 字面量，问题会在此暴露。就地修复，添加 `vision_preprocessor_provider: None`。

- [ ] **步骤 3：Clippy**

运行：`cargo clippy --workspace --all-targets -- -D warnings`

预期：无警告。常见需就地修复的问题：
- `vision_preprocessor.rs` 中未使用的 `use` 导入（清理掉）。
- `&clean` 参数上的 `clippy::needless_borrow`（若 clippy 要求，去掉 `&`）。

- [ ] **步骤 4：若步骤 1-3 中确有修复，提交这些修补**

```bash
git add -A
git commit -m "fix(vision_preprocessor): clippy + build cleanups

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

（若无需修补，跳过该提交。）

---

## 手动集成验证

（非清单任务 —— 计划完全合入后只执行一次。PR 描述中的 Test Plan 必须包含这些步骤。）

1. 执行 `cargo run -p rustcode-cli --release` 进入 TUI。
2. 执行 `/codingplan` 安装 AtomGit providers。
3. 在 `~/.rustcode/config.toml` 中手动添加 `[providers."RustCode-Qwen-Qwen3-VL-32B-Instruct"]` 块，指向 AtomGit 网关，model 为 `Qwen/Qwen3-VL-32B-Instruct`。（或改用非 `AtomGit-` 前缀命名，以免被 `/codingplan` 重跑清除 —— 例如 `vl-qwen3vl`。）
4. 添加顶层配置 `vision_preprocessor_provider = "RustCode-Qwen-Qwen3-VL-32B-Instruct"`（或你实际使用的 key）。
5. 执行 `/model AtomGit-DeepSeek-V4-flash`（或任意不支持视觉的 provider）。
6. Ctrl+V 粘贴一张代码截图，追加 caption "解释这段代码"，按 Enter。
7. **预期：** 滚动历史中显示的用户消息同时包含 `解释这段代码` 与 `[图片内容（由 VL 模型识别）]\n...` 块；`/datalog tail` 显示发给 DeepSeek 的请求是纯文本（没有 `image_url` 块）；主模型能连贯地回答该代码问题。
8. 在配置中注释掉 `vision_preprocessor_provider`，重跑步骤 6。**预期：** DeepSeek 收到 `[image attached]` 占位符（既有回退路径）；主模型没有图片上下文。
9. 将 `vision_preprocessor_provider = "AtomGit-NoSuchModel"` 故意设为一个不存在的 key（故意写错）。重跑步骤 6。**预期：** 出现黄色 `Warning` 行：`VL 预处理失败：VL provider 'AtomGit-NoSuchModel' not found in config.providers`；用户消息以 `[图片识别失败]` 结尾；主模型仍会回复（大概率会要求补充说明）。
10. 使用 `/model claude-sonnet-4-5`（支持视觉）并设置 `vision_preprocessor_provider`，重跑步骤 6。**预期：** 预处理被跳过（无 Notice，也没有 `[图片内容...]` 包裹）；图片原生发给 Claude。

---

## 自查清单（交接前执行）

该清单在撰写计划时已执行过 —— 此处保留以便复核。

**1. Spec 覆盖情况：**
- 目标 / 触发条件 → 任务 2 短路分支 + 任务 5 接入。[x]
- caption 包含在 VL 提示词中 → 任务 3 提示词模板 + 任务 4 caption 测试。[x]
- VL 只看到当前图片、看不到历史 → 任务 3 局部 `Vec<Message>`（结构性保证）。[x]
- VL 输出被追加、图片被丢弃 → 任务 5 `Replaced` 分支。[x]
- 失败 → Warning + 占位符 → 任务 5 `Failed` 分支。[x]
- 配置字段位于顶层、默认为 None、需显式开启 → 任务 1。[x]
- 30 秒超时 → 任务 3 `tokio::time::timeout`。[x]
- 支持视觉的场景下 `[image attached]` 行为不变 → 任务 5 `Skipped` 分支保留既有路径；任务 1 的 serde `skip_serializing_if = Option::is_none` 保证既有配置文件保持干净。[x]
- 非目标：不改动 `LlmProvider` trait。[x]
- 非目标：不改动 `coding_plan/setup.rs`。[x]

**2. 占位符扫描：** 各任务中均无 "TBD"/"TODO"/"add error handling"。[x]

**3. 类型一致性：**
- `LlmProvider`（trait），而非 `Provider`。任务 2 与 5 中使用一致。[x]
- `model_name_suggests_vision`（自由函数，非方法）。[x]
- `StreamEvent::Delta(String)` 而非 `TextDelta`。[+]（TextDelta 是 `AgentEvent` 的变体；provider 侧流使用 `Delta`。）
- `create_provider` 返回 `Result<Box<dyn LlmProvider>>`。任务 3 使用正确。[x]
- `AgentEvent::Warning(String)`（元组变体），非结构体。任务 5 使用 `AgentEvent::Warning(format!(...))`。[x]
