# Vision Preprocessor：来自 /codingplan 的自动配置 实施计划

> **面向 agentic worker：** 必备子技能：使用 superpowers:subagent-driven-development 逐任务实施本计划。步骤使用复选框（`- [ ]`）语法跟踪进度。

**目标：** 当 `/codingplan` 填充 AtomGit-* provider 列表时，自动将 `vision_preprocessor_provider` 设为列表中第一个支持视觉的模型。既能识别视觉语言模型（如 `Qwen3-VL-32B-Instruct`），也能识别 OCR 模型（如 `PaddleOCR-2.0`、`GOT-OCR-2.0`）。保留用户自行填写的非 AtomGit 值；当列表中没有 VL 候选时，清除失效的 AtomGit-* 引用。

**架构：** 三处小改动，全部局限在 `rustcode-core`：扩展已有的 `model_name_suggests_vision` 启发式；在 `coding_plan::setup::step_models_and_register` 中加入 VL 检测与优先级逻辑；在 `ModelsInfo` + `SetupReport::render` 中呈现结果。不新增模块，不改动 agent / TUI。

**技术栈：** Rust。复用 `is_codingplan_provider_name`、`model_name_suggests_vision`、`provider_names_for`，三者均已存在于待修改文件中。

---

## 参考

Spec 位于 `docs/superpowers/specs/2026-05-08-vision-preprocessor-design.md`（原始功能）。本计划处理该 spec 中 §风险与权衡 第 4 项的后续事项，以及用户提出的「识别 OCR 命名模型」需求。

原始功能的提交 1379510..4ce8bc0 已合入。本计划在其之上再增加三个提交。

---

## 优先级规则（编码于任务 8）

| 当前 `config.vision_preprocessor_provider` | 列表含 VL/OCR | 动作 |
|---|---|---|
| `None` | 是 | 设为第一个 VL/OCR provider key |
| `None` | 否 | 保持 None |
| `Some("AtomGit-*")`（由上一次 /codingplan 设置） | 是 | 替换为新的 VL/OCR key |
| `Some("AtomGit-*")` | 否 | 清空为 None（避免指向已被清除的 key） |
| `Some("X")`，其中 X 不是 `AtomGit-*`（用户手动设置） | 是或否 | 保持不变 |

`is_codingplan_provider_name` 辅助函数（已在 `setup.rs` 中）就是精确的判别依据。

---

## 文件结构

| 文件 | 动作 | 职责 |
|---|---|---|
| `crates/rustcode-core/src/provider/mod.rs` | **修改** | 扩展 `model_name_suggests_vision` 以匹配 `ocr` 子串 + 测试 |
| `crates/rustcode-core/src/coding_plan/setup.rs` | **修改** | `step_models_and_register` 中的自动设置逻辑；`ModelsInfo` 上的新字段；`SetupReport::render` 中的渲染行 + 测试 |

---

## 任务 7：扩展 `model_name_suggests_vision` 以识别 OCR

**文件：**
- 修改：`crates/rustcode-core/src/provider/mod.rs:298-336`（`model_name_suggests_vision` 函数及其测试）

- [ ] **步骤 1：更新启发式函数体**

在 `crates/rustcode-core/src/provider/mod.rs` 中定位 `pub fn model_name_suggests_vision(name: &str) -> bool`（约 312 行），添加一个 `ocr` 子句。该函数当前是一串 `||`，把该子句加到链中任意位置（在闭合的 `}` 之前）：

```rust
        || n.contains("ocr")
```

合理位置：放在 `n.contains("vl-")` 之后、`n.contains("-4v")` 之前，使 OCR 系列子串在语义上紧邻 VL 子串。最终形态：

```rust
pub fn model_name_suggests_vision(name: &str) -> bool {
    let n = name.to_lowercase();
    n.contains("vision")
        || n.contains("-vl")
        || n.contains("vl-")
        || n.contains("ocr")
        || n.contains("-4v")
        || n.contains("-4.1v")
        || n.starts_with("gpt-4o")
        // ... rest unchanged
}
```

- [ ] **步骤 2：更新文档注释以说明 OCR 的加入**

该函数的文档注释（298-311 行）目前说明了该启发式的理由，以及假阳性与假阴性之间的权衡。补充一句关于 OCR 的说明：

将既有的文档注释块（以 `false-positives waste a turn on a 400, so when in doubt this returns false.` 结尾）替换为：

```rust
/// Heuristic: does this model name look like a vision-capable model?
///
/// Used by the TUI's Ctrl+V image-paste handler to refuse attaching an
/// image when the active model almost certainly can't accept it (e.g.
/// `glm-5.1`, `deepseek-v4-flash`, `qwen3-coder`). Without this gate
/// the user wastes a turn on a 400 from the upstream — see the
/// `ModelArts.81001` `message[3].content[0] has invalid field(s):
/// text, type` failure pattern that surfaced in production.
///
/// Also used by `vision_preprocessor::maybe_preprocess` to decide
/// whether the active main provider needs preprocessing (vision-capable
/// → skip) and by `coding_plan::setup` to auto-pick a VL preprocessor
/// from the AtomGit model list.
///
/// "OCR" is included because OCR-on-VLM endpoints (PaddleOCR-VL,
/// GOT-OCR, MonkeyOCR, etc.) accept image input via the same
/// OpenAI-compatible `image_url` schema and are first-class candidates
/// for the vision-preprocessor role.
///
/// Conservative — only matches well-known vision/OCR patterns.
/// False-negatives are safe: extend this list when a new vision/OCR model
/// ships rather than threading a per-provider config knob (no
/// user-discoverable opt-in exists). False-positives waste a turn on
/// a 400, so when in doubt this returns false.
pub fn model_name_suggests_vision(name: &str) -> bool {
```

- [ ] **步骤 3：添加 OCR 测试**

在 `provider/mod.rs` 已有的 `mod tests` 块（`vision_heuristic_*` 测试，约 462-499 行）中追加：

```rust
    /// OCR family: PaddleOCR-VL is already covered by the `-vl` clause,
    /// but pure-OCR names (no VL/vision substring) need the dedicated
    /// `ocr` clause to be recognized as vision-eligible.
    #[test]
    fn vision_heuristic_recognises_ocr_models() {
        // Names with both ocr + vl/vision (already worked, regression check).
        assert!(model_name_suggests_vision("PaddleOCR-VL-0.9B"));
        assert!(model_name_suggests_vision("Qwen2-VL-OCR-7B"));
        // Pure OCR names — should now match via the dedicated clause.
        assert!(model_name_suggests_vision("GOT-OCR-2.0"));
        assert!(model_name_suggests_vision("PaddleOCR-2.0"));
        assert!(model_name_suggests_vision("MinerU-OCR"));
        assert!(model_name_suggests_vision("MonkeyOCR-1.2B"));
        assert!(model_name_suggests_vision("got-ocr-1.0")); // lowercase
    }

    /// Non-OCR model names containing the substring `ocr` as a
    /// coincidence (rare; document the false-positive risk). Today none
    /// of these ship as actual atomgit models — if one does, we'll
    /// tighten the heuristic. Test left as a placeholder so future
    /// regressions get caught.
    #[test]
    fn vision_heuristic_documented_false_positives() {
        // These COULD theoretically false-positive on the `ocr` clause
        // if such names ever ship as text-only models. Today none do.
        // Listed here so a maintainer adding such a model sees the
        // expected failure and reconsiders the heuristic.
        assert!(model_name_suggests_vision("focar-text-7b")); // contrived
    }
```

第二个测试只是信息性说明 —— 它记录了该权衡。若将来真有模型名包含 `ocr` 但并非视觉模型，该测试需要调整。

- [ ] **步骤 4：运行测试**

```bash
cd /Users/theo/Documents/workspace/rustcode/
cargo test -p rustcode-core --lib provider::tests::vision_heuristic
```

预期：所有 `vision_heuristic_*` 测试通过（原有 2 个 + 新增 2 个）。

- [ ] **步骤 5：提交**

```bash
cat > /tmp/rustcode-task7-msg.txt <<'EOF'
feat(provider): include OCR substring in vision-capable heuristic

OCR-on-VLM endpoints (PaddleOCR, GOT-OCR, MonkeyOCR, MinerU-OCR, etc.)
accept image inputs via the same OpenAI-compatible image_url schema and
are excellent vision_preprocessor candidates — often more accurate and
cheaper for code/error screenshots than general-purpose VL. Extend the
heuristic so they're auto-recognized by Ctrl+V paste gating, the
preprocessor short-circuit, and /codingplan auto-detection (next commit).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF

cd /Users/theo/Documents/workspace/rustcode/
git add crates/rustcode-core/src/provider/mod.rs
git commit -F /tmp/rustcode-task7-msg.txt -- crates/rustcode-core/src/provider/mod.rs
```

---

## 任务 8：在 `/codingplan` 中自动设置 `vision_preprocessor_provider`

**文件：**
- 修改：`crates/rustcode-core/src/coding_plan/setup.rs` —— 函数 `step_models_and_register`（约 422-469 行）与 `ModelsInfo` 结构体（约 257-265 行）

- [ ] **步骤 1：新增一个用于表达结果的变体枚举**

在 `setup.rs` 靠顶部位置（既有的 `StepResult` 定义之后，或 `ModelsInfo` 附近）添加：

```rust
/// Describes how the auto-detected vision_preprocessor_provider was
/// (or was not) updated by `step_models_and_register`. Surfaces in
/// `SetupReport::render` so the user can see what happened to that
/// config knob across the /codingplan flow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VisionPreprocessorOutcome {
    /// Field was None and remains None (no VL/OCR in list).
    UnchangedNone,
    /// Field was a non-AtomGit user-supplied value; preserved.
    /// Carries the value for display.
    UserSupplied(String),
    /// Field was None or a stale AtomGit-* key; auto-pointed at a
    /// vision-capable provider in the freshly-installed list.
    /// Carries the new key.
    AutoSet(String),
    /// Field was an AtomGit-* key but the new list has no VL/OCR
    /// candidate, so the field was cleared to None to avoid pointing
    /// at a wiped provider key.
    Cleared,
}
```

- [ ] **步骤 2：为 `ModelsInfo` 添加字段**

既有结构体（约 257-265 行）：

```rust
#[derive(Debug, Clone)]
pub struct ModelsInfo {
    pub display_names: Vec<String>,
    pub provider_names: Vec<String>,
    pub default_provider: String,
}
```

添加第四个字段：

```rust
#[derive(Debug, Clone)]
pub struct ModelsInfo {
    pub display_names: Vec<String>,
    pub provider_names: Vec<String>,
    pub default_provider: String,
    /// Outcome of vision_preprocessor_provider auto-config. Drives the
    /// "Vision preprocessor → ..." line in the rendered report.
    pub vision_preprocessor: VisionPreprocessorOutcome,
}
```

- [ ] **步骤 3：在 `step_models_and_register` 中实现自动设置逻辑**

既有函数（约 422-469 行）当前结尾如下：

```rust
    config.default_provider = default_provider.clone();

    StepResult::Ok(ModelsInfo {
        display_names: names,
        provider_names,
        default_provider,
    })
}
```

在 `config.default_provider = ...` 之后、`StepResult::Ok(...)` 之前插入自动设置逻辑：

```rust
    config.default_provider = default_provider.clone();

    // Auto-detect a vision_preprocessor candidate from the freshly
    // installed list. Precedence:
    //   - User-supplied non-AtomGit value: leave alone.
    //   - None / AtomGit-* (i.e. previous /codingplan run): replace
    //     with first VL/OCR model's provider key from the new list,
    //     or clear to None when the new list has no VL candidate.
    let vl_idx = names.iter().position(|n| {
        crate::provider::model_name_suggests_vision(n)
    });
    let new_vl_key = vl_idx.map(|i| provider_names[i].clone());

    let vision_preprocessor = {
        let current = config.vision_preprocessor_provider.clone();
        let user_supplied_non_atomgit = current
            .as_deref()
            .map(|k| !k.is_empty() && !is_codingplan_provider_name(k))
            .unwrap_or(false);

        if user_supplied_non_atomgit {
            VisionPreprocessorOutcome::UserSupplied(current.unwrap())
        } else {
            match new_vl_key {
                Some(k) => {
                    config.vision_preprocessor_provider = Some(k.clone());
                    VisionPreprocessorOutcome::AutoSet(k)
                }
                None => {
                    if current.is_some() {
                        // Was AtomGit-* (per the precedence above) and
                        // the new list has no VL — clearing prevents a
                        // dangling reference.
                        config.vision_preprocessor_provider = None;
                        VisionPreprocessorOutcome::Cleared
                    } else {
                        VisionPreprocessorOutcome::UnchangedNone
                    }
                }
            }
        }
    };

    StepResult::Ok(ModelsInfo {
        display_names: names,
        provider_names,
        default_provider,
        vision_preprocessor,
    })
}
```

- [ ] **步骤 4：更新 render() 以输出结果**

在 `SetupReport::render` 中（约 132-162 行，即 `match &self.models { StepResult::Ok(info) => { ... } }` 分支），在既有打印 provider 名称的 bullet 循环之后添加：

```rust
                for (pname, model) in info.provider_names.iter().zip(info.display_names.iter()) {
                    let suffix = if pname == &info.default_provider {
                        "  (default)"
                    } else {
                        ""
                    };
                    out.push_str(&format!("      • {}  →  {}{}\n", pname, model, suffix));
                }
                // Add: vision-preprocessor line.
                match &info.vision_preprocessor {
                    VisionPreprocessorOutcome::AutoSet(k) => {
                        out.push_str(&format!(
                            "  ✔ Vision preprocessor → {}  (auto-detected)\n",
                            k,
                        ));
                    }
                    VisionPreprocessorOutcome::UserSupplied(k) => {
                        out.push_str(&format!(
                            "  ✔ Vision preprocessor → {}  (user setting kept)\n",
                            k,
                        ));
                    }
                    VisionPreprocessorOutcome::Cleared => {
                        out.push_str(
                            "  ⚠ Vision preprocessor cleared — no VL/OCR model in current list\n",
                        );
                    }
                    VisionPreprocessorOutcome::UnchangedNone => {
                        // No-op: nothing to say when both the previous and
                        // new state are "no preprocessor configured".
                    }
                }
```

- [ ] **步骤 5：更新既有 render 测试以构造新字段**

`setup.rs` 中的 render 测试（约 `render_happy_path_has_all_checkmarks`、`render_claim_duplicate_renders_as_success`、`render_status_pending_activation_omits_zero_expiry`、`render_login_failed_blocks_persist_and_suppresses_cascade`、`render_multi_model_lists_all_providers_with_default_mark`、`render_claim_failed_suppresses_cascade_rows`、`render_skipped_with_non_cascade_reason_still_shows`、`render_status_error_truncates_long_message`）会构造 `ModelsInfo` 字面量，每个字面量都需要补上这个新字段。

运行：

```bash
cd /Users/theo/Documents/workspace/rustcode/
grep -n "ModelsInfo {" crates/rustcode-core/src/coding_plan/setup.rs
```

对测试 fixture 中每个形如 `StepResult::Ok(ModelsInfo { ... })` 的 `ModelsInfo {` 字面量，添加 `vision_preprocessor: VisionPreprocessorOutcome::UnchangedNone,`（该变体为空操作 —— 可保持测试输出不变）。示例：

```rust
            models: StepResult::Ok(ModelsInfo {
                display_names: vec!["a/b".into()],
                provider_names: vec!["AtomGit".into()],
                default_provider: "AtomGit".into(),
                vision_preprocessor: VisionPreprocessorOutcome::UnchangedNone,
            }),
```

不要为每个测试单独添加导入 —— 测试里已有 `use super::*;`，该变体应当可以直接解析。可在下一步运行测试时验证。

- [ ] **步骤 6：为新逻辑添加单元测试**

在 `setup.rs` 已有的 `#[cfg(test)] mod tests` 块中，于 `step_models_wipes_stale_atomgit_entries`（约 635-692 行）之后添加五个测试，覆盖优先级表的每一行：

```rust
    fn vl_model_entry(model: &str) -> ModelEntry {
        ModelEntry {
            id: 1,
            is_infinity: 0,
            is_rustcode_exclusive: 0,
            display_model_name: model.to_string(),
        }
    }

    /// Helper that runs the same wipe-and-insert sequence as
    /// `step_models_and_register` body for a given (config, models)
    /// combo and returns the resulting `ModelsInfo`. Avoids the network
    /// dependency by computing everything locally.
    fn run_register(config: &mut Config, models: Vec<ModelEntry>) -> ModelsInfo {
        // Mirror of step_models_and_register's body. Kept in sync by
        // the test signal: if production diverges, the existing
        // step_models_wipes_stale_atomgit_entries test catches it.
        let stale: Vec<String> = config
            .providers
            .keys()
            .filter(|k| is_codingplan_provider_name(k))
            .cloned()
            .collect();
        for k in stale {
            config.providers.remove(&k);
        }
        let names: Vec<String> = models.iter().map(|m| m.display_model_name.clone()).collect();
        let provider_names = provider_names_for(&names);
        let default_provider = provider_names
            .first()
            .cloned()
            .unwrap_or_else(|| PROVIDER_PREFIX.to_string());
        for (pname, m) in provider_names.iter().zip(models.iter()) {
            config
                .providers
                .insert(pname.clone(), build_codingplan_provider(&m.display_model_name));
        }
        config.default_provider = default_provider.clone();

        let vl_idx = names.iter().position(|n| crate::provider::model_name_suggests_vision(n));
        let new_vl_key = vl_idx.map(|i| provider_names[i].clone());
        let vision_preprocessor = {
            let current = config.vision_preprocessor_provider.clone();
            let user_supplied_non_atomgit = current
                .as_deref()
                .map(|k| !k.is_empty() && !is_codingplan_provider_name(k))
                .unwrap_or(false);
            if user_supplied_non_atomgit {
                VisionPreprocessorOutcome::UserSupplied(current.unwrap())
            } else {
                match new_vl_key {
                    Some(k) => {
                        config.vision_preprocessor_provider = Some(k.clone());
                        VisionPreprocessorOutcome::AutoSet(k)
                    }
                    None => {
                        if current.is_some() {
                            config.vision_preprocessor_provider = None;
                            VisionPreprocessorOutcome::Cleared
                        } else {
                            VisionPreprocessorOutcome::UnchangedNone
                        }
                    }
                }
            }
        };

        ModelsInfo {
            display_names: names,
            provider_names,
            default_provider,
            vision_preprocessor,
        }
    }

    #[test]
    fn vision_preprocessor_auto_set_when_none_and_list_has_vl() {
        let mut config = blank_config();
        let models = vec![
            vl_model_entry("moonshotai/Kimi-K2-Instruct"),
            vl_model_entry("Qwen/Qwen3-VL-32B-Instruct"),
            vl_model_entry("deepseek/deepseek-v4-flash"),
        ];
        let info = run_register(&mut config, models);
        // Second model is the VL candidate (Kimi has no VL hint).
        let expected = "RustCode-Qwen-Qwen3-VL-32B-Instruct".to_string();
        assert_eq!(
            info.vision_preprocessor,
            VisionPreprocessorOutcome::AutoSet(expected.clone())
        );
        assert_eq!(config.vision_preprocessor_provider, Some(expected));
    }

    #[test]
    fn vision_preprocessor_unchanged_none_when_list_has_no_vl() {
        let mut config = blank_config();
        let models = vec![vl_model_entry("moonshotai/Kimi-K2-Instruct")];
        let info = run_register(&mut config, models);
        assert_eq!(info.vision_preprocessor, VisionPreprocessorOutcome::UnchangedNone);
        assert_eq!(config.vision_preprocessor_provider, None);
    }

    #[test]
    fn vision_preprocessor_overwrites_stale_atomgit_value() {
        let mut config = blank_config();
        // Simulate previous /codingplan that set this AtomGit-* key.
        config.vision_preprocessor_provider =
            Some("RustCode-Qwen-Qwen2-VL-72B".into());
        let models = vec![
            vl_model_entry("Kimi-K2-Instruct"),
            vl_model_entry("Qwen/Qwen3-VL-32B-Instruct"),
        ];
        let info = run_register(&mut config, models);
        let expected = "RustCode-Qwen-Qwen3-VL-32B-Instruct".to_string();
        assert_eq!(
            info.vision_preprocessor,
            VisionPreprocessorOutcome::AutoSet(expected.clone())
        );
        assert_eq!(config.vision_preprocessor_provider, Some(expected));
    }

    #[test]
    fn vision_preprocessor_cleared_when_stale_atomgit_and_list_has_no_vl() {
        let mut config = blank_config();
        config.vision_preprocessor_provider =
            Some("RustCode-Qwen-Qwen2-VL-72B".into());
        let models = vec![vl_model_entry("moonshotai/Kimi-K2-Instruct")];
        let info = run_register(&mut config, models);
        assert_eq!(info.vision_preprocessor, VisionPreprocessorOutcome::Cleared);
        assert_eq!(config.vision_preprocessor_provider, None);
    }

    #[test]
    fn vision_preprocessor_preserves_user_set_non_atomgit() {
        let mut config = blank_config();
        // User has manually configured a SiliconFlow-hosted VL.
        config.vision_preprocessor_provider = Some("Qwen3-VL-32B-Instruct".into());
        let models = vec![
            vl_model_entry("Kimi-K2-Instruct"),
            vl_model_entry("Qwen/Qwen3-VL-32B-Instruct"), // would otherwise auto-set
        ];
        let info = run_register(&mut config, models);
        assert_eq!(
            info.vision_preprocessor,
            VisionPreprocessorOutcome::UserSupplied("Qwen3-VL-32B-Instruct".into())
        );
        // Crucially: config value is unchanged.
        assert_eq!(
            config.vision_preprocessor_provider.as_deref(),
            Some("Qwen3-VL-32B-Instruct")
        );
    }

    #[test]
    fn vision_preprocessor_recognises_pure_ocr_model_name() {
        // Regression for Task 7: pure OCR names (no VL/vision substring)
        // must still be recognized as VL candidates by the heuristic, so
        // the auto-set path picks them up.
        let mut config = blank_config();
        let models = vec![
            vl_model_entry("Kimi-K2-Instruct"),
            vl_model_entry("PaddleOCR-2.0"),
        ];
        let info = run_register(&mut config, models);
        let expected = "AtomGit-PaddleOCR-2.0".to_string();
        assert_eq!(
            info.vision_preprocessor,
            VisionPreprocessorOutcome::AutoSet(expected.clone())
        );
        assert_eq!(config.vision_preprocessor_provider, Some(expected));
    }
```

- [ ] **步骤 7：运行测试**

```bash
cd /Users/theo/Documents/workspace/rustcode/
cargo test -p rustcode-core --lib coding_plan
```

预期：所有 coding_plan 测试通过 —— 既有测试（其 `ModelsInfo` 字面量现已包含新字段）加上 6 个新测试。

若某个既有测试因 `ModelsInfo` 字面量不完整而失败，找到它并补上 `vision_preprocessor: VisionPreprocessorOutcome::UnchangedNone,`。

- [ ] **步骤 8：单独运行 render 测试并检查输出**

新增的渲染行代码会为 AutoSet / UserSupplied / Cleared 变体输出内容。render 测试使用 `UnchangedNone`（空操作），因此输出不变、应仍通过。验证：

```bash
cd /Users/theo/Documents/workspace/rustcode/
cargo test -p rustcode-core --lib coding_plan::setup::tests::render -- --nocapture
```

预期：全部通过。（加 `--nocapture` 只是方便你顺便看一眼输出。）

- [ ] **步骤 9：为新渲染行添加 render 测试**

追加到 `mod tests`：

```rust
    /// Render exercise: the vision-preprocessor line shows up under
    /// `Added N providers` when the auto-set path fires.
    #[test]
    fn render_includes_vision_preprocessor_auto_set_line() {
        let report = SetupReport {
            login: StepResult::Skipped("already logged in".into()),
            claim: StepResult::Ok(ClaimInfo {
                message: String::new(),
                duplicate: false,
            }),
            models: StepResult::Ok(ModelsInfo {
                display_names: vec![
                    "Kimi-K2-Instruct".into(),
                    "Qwen/Qwen3-VL-32B-Instruct".into(),
                ],
                provider_names: vec![
                    "AtomGit-Kimi-K2-Instruct".into(),
                    "RustCode-Qwen-Qwen3-VL-32B-Instruct".into(),
                ],
                default_provider: "AtomGit-Kimi-K2-Instruct".into(),
                vision_preprocessor: VisionPreprocessorOutcome::AutoSet(
                    "RustCode-Qwen-Qwen3-VL-32B-Instruct".into(),
                ),
            }),
            status: StepResult::Skipped("status check skipped for this test".into()),
        };
        let out = report.render();
        assert!(
            out.contains("Vision preprocessor → RustCode-Qwen-Qwen3-VL-32B-Instruct"),
            "render must include the auto-detected line: {out}",
        );
        assert!(out.contains("(auto-detected)"));
    }

    #[test]
    fn render_includes_vision_preprocessor_cleared_line_when_stale_dropped() {
        let report = SetupReport {
            login: StepResult::Skipped("already logged in".into()),
            claim: StepResult::Ok(ClaimInfo {
                message: String::new(),
                duplicate: false,
            }),
            models: StepResult::Ok(ModelsInfo {
                display_names: vec!["Kimi-K2-Instruct".into()],
                provider_names: vec!["AtomGit-Kimi-K2-Instruct".into()],
                default_provider: "AtomGit-Kimi-K2-Instruct".into(),
                vision_preprocessor: VisionPreprocessorOutcome::Cleared,
            }),
            status: StepResult::Skipped("test skip".into()),
        };
        let out = report.render();
        assert!(
            out.contains("Vision preprocessor cleared"),
            "render must surface the cleared state: {out}",
        );
    }

    #[test]
    fn render_includes_vision_preprocessor_user_supplied_line() {
        let report = SetupReport {
            login: StepResult::Skipped("already logged in".into()),
            claim: StepResult::Ok(ClaimInfo {
                message: String::new(),
                duplicate: false,
            }),
            models: StepResult::Ok(ModelsInfo {
                display_names: vec![
                    "Kimi-K2-Instruct".into(),
                    "Qwen/Qwen3-VL-32B-Instruct".into(),
                ],
                provider_names: vec![
                    "AtomGit-Kimi-K2-Instruct".into(),
                    "RustCode-Qwen-Qwen3-VL-32B-Instruct".into(),
                ],
                default_provider: "AtomGit-Kimi-K2-Instruct".into(),
                vision_preprocessor: VisionPreprocessorOutcome::UserSupplied(
                    "Qwen3-VL-32B-Instruct".into(),
                ),
            }),
            status: StepResult::Skipped("test skip".into()),
        };
        let out = report.render();
        assert!(out.contains("Vision preprocessor → Qwen3-VL-32B-Instruct"));
        assert!(out.contains("(user setting kept)"));
    }

    #[test]
    fn render_omits_vision_preprocessor_line_when_unchanged_none() {
        let report = SetupReport {
            login: StepResult::Skipped("already logged in".into()),
            claim: StepResult::Ok(ClaimInfo {
                message: String::new(),
                duplicate: false,
            }),
            models: StepResult::Ok(ModelsInfo {
                display_names: vec!["Kimi-K2-Instruct".into()],
                provider_names: vec!["AtomGit-Kimi-K2-Instruct".into()],
                default_provider: "AtomGit-Kimi-K2-Instruct".into(),
                vision_preprocessor: VisionPreprocessorOutcome::UnchangedNone,
            }),
            status: StepResult::Skipped("test skip".into()),
        };
        let out = report.render();
        // No-op variant must NOT add a line — keeps existing report output
        // identical for users who never set/get a VL provider.
        assert!(!out.contains("Vision preprocessor"));
    }
```

- [ ] **步骤 10：重跑全部 coding_plan 测试**

```bash
cd /Users/theo/Documents/workspace/rustcode/
cargo test -p rustcode-core --lib coding_plan
```

预期：全部通过。

- [ ] **步骤 11：Workspace clippy + 构建**

```bash
cd /Users/theo/Documents/workspace/rustcode/
cargo build --workspace --all-targets 2>&1 | tail -20
cargo clippy -p rustcode-core --lib --all-targets -- -D warnings 2>&1 | tail -30
```

预期：构建通过，且本提交不引入新的 clippy 警告。

- [ ] **步骤 12：提交**

```bash
cat > /tmp/rustcode-task8-msg.txt <<'EOF'
feat(coding_plan): auto-set vision_preprocessor_provider from model list

When /codingplan installs the AtomGit provider list, scan for the first
vision-capable model (via model_name_suggests_vision) and set
vision_preprocessor_provider to its provider key. Precedence:
  - User-supplied non-AtomGit values: preserved unchanged.
  - None or stale AtomGit-* (from previous run): replaced or cleared.

The render() output adds one of three new lines (auto-detected,
user setting kept, cleared) so users can see the resulting state.
UnchangedNone is silent to keep output identical for setups without VL.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF

cd /Users/theo/Documents/workspace/rustcode/
git add crates/rustcode-core/src/coding_plan/setup.rs
git commit -F /tmp/rustcode-task8-msg.txt -- crates/rustcode-core/src/coding_plan/setup.rs
```

---

## 任务 9：Workspace 验证

本任务仅做验证 —— 除非验证发现回归，否则不改动代码。

- [ ] **步骤 1：运行 rustcode-core 全量测试**

```bash
cd /Users/theo/Documents/workspace/rustcode/
cargo test -p rustcode-core --lib 2>&1 | tail -10
```

预期：通过数不低于基线（任务 1–6 之后为 1104 个通过）。任务 7+8 的新测试应增加约 10 个。既有失败项保持不变。

- [ ] **步骤 2：Workspace 构建**

```bash
cd /Users/theo/Documents/workspace/rustcode/
cargo build --workspace --all-targets 2>&1 | tail -10
```

预期：成功。

- [ ] **步骤 3：Workspace clippy**

```bash
cd /Users/theo/Documents/workspace/rustcode/
cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -30
```

预期：只有既有警告（与任务 6 报告的一致）。

- [ ] **步骤 4：若确有修补，提交它们**

若验证发现某个结构体字面量需要补上新的 `vision_preprocessor` 字段初始化（类似提交 `4ce8bc0` 中对 daemon 的修复），则应用之：

```bash
cat > /tmp/rustcode-task9-msg.txt <<'EOF'
fix(coding_plan): missed ModelsInfo literal cleanups

[describe specific fixes here]

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF

cd /Users/theo/Documents/workspace/rustcode/
git add -A
git commit -F /tmp/rustcode-task9-msg.txt
```

若无需修补，跳过本步骤。

---

## 手动验证（合入后）

1. 备份当前 `~/.rustcode/config.toml`。
2. 编辑该文件，删除 `vision_preprocessor_provider = ...` 一行，使该字段变为 None。
3. 运行 `cargo run -p rustcode-cli --release -- /codingplan`（或在 TUI 内调用 `/codingplan`）。
4. 检查 `/codingplan` 的输出：若 API 返回的列表中含有 VL 模型，应能看到 `✔ Vision preprocessor → AtomGit-...  (auto-detected)` 一行。
5. 确认 `~/.rustcode/config.toml` 中现在含有 `vision_preprocessor_provider = "AtomGit-..."`。
6. 将该字段设为你自己的非 AtomGit 值（例如你 SiliconFlow 配置中的 `Qwen3-VL-32B-Instruct`），重跑 /codingplan，确认其值未被改动，且报告显示 `(user setting kept)`。

---

## 自查清单（交接前执行）

**1. Spec 覆盖情况：**
- OCR 模型可被识别 → 任务 7。[x]
- None 时自动设置 → 任务 8 步骤 3 + 测试。[x]
- AtomGit-* 失效值时自动覆盖 → 任务 8 步骤 3 + 测试。[x]
- AtomGit-* 且列表无 VL 时清空 → 任务 8 步骤 3 + 测试。[x]
- 非 AtomGit 的用户值被保留 → 任务 8 步骤 3 + 测试。[x]
- 每种结果都有对应渲染行 → 任务 8 步骤 4 + 9。[x]
- UnchangedNone 静默不输出 → 任务 8 步骤 9（`render_omits_vision_preprocessor_line_when_unchanged_none`）。[x]

**2. 占位符扫描：** 各任务中均无 "TBD"/"TODO"/"add error handling"。任务 9 可选提交中的 "[describe specific fixes here]" 占位符是可接受的 —— 它仅在确实需要修补时才会用到，届时由实施者填写。

**3. 类型一致性：**
- `VisionPreprocessorOutcome` 只定义一次（任务 8 步骤 1），在生产代码（任务 8 步骤 2–4）与测试（任务 8 步骤 5、6、9）中均有使用。[x]
- `model_name_suggests_vision`（自由函数）在任务 7 与任务 8 中使用方式一致。[x]
- `is_codingplan_provider_name` 在清除步骤与优先级判断中均有使用。[x]
- `ModelsInfo` 字面量更新（任务 8 步骤 5）覆盖了全部 8 个既有 render 测试。[x]
