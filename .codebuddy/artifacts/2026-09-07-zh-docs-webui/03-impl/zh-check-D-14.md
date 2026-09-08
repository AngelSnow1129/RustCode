# zh-check 报告

- 基线 (--base): `3ee655e3`
- 阈值 (--en-ratio): `0.05`
- 判定: AC-2 / AC-4 / AC-6 / AC-7b / AC-32，任一失败即 FAIL（fail-closed）

## 汇总

- 受检文件: 3
- PASS: 3
- FAIL: 0

## PASS `docs/plans/2026-07-26-provider-accounts-model-profiles-design.md`

- en=0/164 ratio=0.0000

## PASS `docs/plans/2026-07-26-provider-accounts-model-profiles-plan.md`

- en=0/173 ratio=0.0000

## PASS `docs/plans/2026-08-20-code-review-deep-mode-fanout-design.md`

- en=0/95 ratio=0.0000

---

# D-14 手写小结（doc-writer）

## 1. 任务范围与判定

本批 3 个文件均为 `docs/plans/` 下的英文计划/设计文档，改动前 Han=0（纯英文），
**均非 no-op**，全部做了完整正文汉化。三文件**均无 YAML frontmatter**
（首行不是 `---`），因此 AC-6「frontmatter 键名集合相等」退化为两侧空集相等，
不适用；AC-6 的 `description` 汉化清单为空，无需逐条列示。

| 文件 | 改前 Han | 判定 | 改后 en/total | 译文行数（约） |
| --- | --- | --- | --- | --- |
| `docs/plans/2026-07-26-provider-accounts-model-profiles-design.md` | 0 | 完整汉化 | 0/164 | 全文 14 节 + 标题/元信息 |
| `docs/plans/2026-07-26-provider-accounts-model-profiles-plan.md` | 0 | 完整汉化 | 0/173 | 基线约束 + 任务 1–10 + 交付门禁 |
| `docs/plans/2026-08-20-code-review-deep-mode-fanout-design.md` | 0 | 完整汉化 | 0/95 | 问题/目标/架构/测试 + 第 2 阶段 |

`git diff --stat 3ee655e3` 合计 3 files changed, 517 insertions(+), 671 deletions(-)；
差异全部落在散文行，fenced code block（design 10 个、plan 18 个、fanout 1 个）
与 inline code 集合逐字节未动（见 AC-4 结论）。

## 2. 残留行清单（AC-3）

三个文件 **en=0，无任何残留英文 offender 行**，ratio 均为 0.0000，
不需要使用白名单。

补充说明（不构成残留，仅为避免后续批次误判）：有 2 处英文词残留在**中文句子内部**
（`maps in`、`(the`），因所在行同时含汉字，脚本不计为 en 行；残留原因见第 4 节。

## 3. 四项实测结果（本次最终自检）

| 项 | 阈值 | 实测 | 结论 |
| --- | --- | --- | --- |
| AC-2 en/total | ≤ 0.05 | 0/164、0/173、0/95，均 0.0000 | PASS |
| AC-4 inline+fenced code 多重集 | 与基线完全相等 | 三文件全部相等（中间曾 FAIL，已修，见第 4 节） | PASS |
| AC-7b `](...)` 链接目标多重集 | 相等 | 三文件基线均为 0 个链接，改后仍为 0 | PASS |
| AC-32 Emoji 数 | 不增加 | 三文件均为 0 → 0 | PASS |

自检命令（最终一次，报告即由该命令生成）：

```bash
python3 scripts/check-zh-docs.py check --base 3ee655e3 \
  --report .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-14.md \
  --files docs/plans/2026-07-26-provider-accounts-model-profiles-design.md \
          docs/plans/2026-07-26-provider-accounts-model-profiles-plan.md \
          docs/plans/2026-08-20-code-review-deep-mode-fanout-design.md
```

## 4. 首轮 AC-4 FAIL 的成因与修复（本批唯一踩坑，供后续批次参考）

**现象**：首轮自检 design 与 fanout 两份文件 AC-4 FAIL。
基线的 inline code 多重集里存在两个「意外 span」：
design `' , '`（即 `` `, ` `` 被当成一对反引号）、
fanout `' maps in '` 与 `' (the '`。

**成因**：这两处原文的反引号被**换行拆断**——`code_spans()` 的
`` `[^`\n]*` `` 不能跨行匹配，于是基线上「行尾未闭合的反引号」与「下一行开头的反引号」
被错误配成一对，把中间的英文散文（`", "`、`" maps in "`、`" (the "`）
算成了 inline code 内容。我首轮把这几段重排成单行后，反引号正确配对了，
span 集合反而与基线不等（old=239/new=240、old=130/new=132）。

**修复**：**保留原文的换行位置与反引号位置**，只翻译自由散文；
在被吞掉的内容之后补入汉字，使该行不计入 en 行。具体：

- `2026-07-26-provider-accounts-model-profiles-design.md` §14.5 第 2 条：
  恢复 `{runtime,` 处的换行，`, ` 分隔符保持 ASCII 原样，行尾补「等文件为主，」。
- `2026-08-20-code-review-deep-mode-fanout-design.md` §触发方式：
  恢复 `/review deep+verify` 与 `[scope]` 之间的换行（因此不能写成
  `` `/review deep+verify [scope]` `` 单条 inline code），
  `maps in`、`(the` 两个英文词被迫保留，随后补「关键字先于」「匹配」。
- `2026-08-20-code-review-deep-mode-fanout-design.md` §结果呈现：
  恢复 `` `verify: dropped K of M` `` 与 `candidate finding(s)` 之间的换行，
  行尾补「，用于说明剔除了多少候选发现。」。

**代价**：上述 2 个英文词（`maps in`、`(the`）作为片段残留，
是为满足 AC-4「代码 span 多重集与基线完全相等」所必须；
若后续有人把这几行重排为单行，AC-4 会立刻 FAIL，请勿「顺手整理」。

## 5. 术语与符号保留核对

按铁律 §2.3 与「本批特殊说明 3」，以下全部原样保留，未汉化、未改写：

- TOML 表名与配置键：`[providers.*]`、`[provider_accounts.*]`、`[models.*]`、
  `default_provider`、`default_model`、`base_url`、`api_key`、`context_window`、
  `capable_model`、`evaluator_provider`、`vision_preprocessor_provider`、
  `model_providers`、`user_agent`、`skip_tls_verify`、`enterprise_url`、`has_api_key`。
- provider 名/预设 ID：`aliyun`、`openai-compatible`、`anthropic-compatible`、
  `MyDeepSeek`、`corp`、各厂商名（AtomGit、Alibaba、Volcengine、Xiaomi MiMo、
  DeepSeek、Zhipu、Moonshot、MiniMax、SiliconFlow、OpenRouter、OpenAI、Anthropic、Ollama）。
- 模型 ID：`qwen3-coder-plus`、`qwen3-max`、`deepseek-chat`、`company-code-model` 等。
- 类型/函数/枚举：`ProviderConfig`、`ProviderPreset`、`ProviderAccountConfig`、
  `ModelProfileConfig`、`ResolvedModelConfig`、`Config::resolve_model`、
  `active_provider()`、`default_context_window()`、`AuthKind`、`ProviderType`、
  `ModelSource`、`ModelCatalogSource`、`resolve_tier_keys`、`merge_findings`、
  `finalize_deep_review`、`run_verify`、`build_review_agent_with`、`report_finding`、
  `ctx.cancel`、`tokio::task::JoinSet`、`tokio::select!`、`ConfigStore`（CAS）。
- 命令与路径：`/review deep`、`/review deep+verify [scope]`、`/provider`、`/model`、
  `cargo test ...`、`git commit -m "..."`、`--provider`、`--model`、`--offline`，
  以及全部 crate/文件路径（均位于 inline code 或 fenced block 内，未改动）。
- 第三方专有名词：Rust、Serde/TOML、Ratatui/crossterm、Axum、Tokio、OpenCode、
  Models.dev、Codex、GitHub Copilot、GitHub、OAuth、MCP、ACP、WebUI、TUI、CLI、
  daemon、driver、clix、headless、CodingPlan、`gitcode-assist-service`。

## 6. 未做之事（边界声明）

- 未改源码、未执行 `cargo`/`npm` 构建或测试、未提交、未打标签。
- 未改动任何文件名、标题层级、链接目标（本批 3 文件无外链）。
- 未增删 Emoji，未润色既有中文（本批改前无中文段落）。
- 只改 `files_owned` 列出的 3 个文件。
