---
kind: impl-report
id: T7
from: code-implementer
to: code-reviewer
feature: 2026-09-02-cleanup-codingplan-legacy
status: done
decision: review
requires: [TASKS-001]
files_owned:
  - .codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/03-impl/T7-dead-code-scan.md
architecture_constraints:
  - 零代码改动（本任务只产出清单）
  - 铁律保护项即便零引用也不得删（AGENTS.md:213-214 / :338 / :530；N6）
created: 2026-09-02
---

# T7 扫描登记报告 — 死代码 / 跨 crate 重复实现（仅登记，零代码改动）

> 子 Agent 派发触发 idle timeout；本扫描由编排者直接执行（只读 `rg`）。工作区零代码改动（已用 `git status --porcelain crates/ docs/config.example.toml` 核验，与任务开始前一致）。

## 第 0 步基线

`git status --porcelain crates/ docs/config.example.toml docs/` 输出 = 与 T1 前一致（仅含 baseline dirty 中既有的用户改动，本任务未引入任何新改动）。

## 类 1：`Cp*` / `StatusCp*` i18n 变体

| 符号簇 | 引用分布（文件:命中行数） | 引用计数 | 处置 |
|---|---|---|---|
| `Cp*`（如 `StatusCp*` 等 codingplan 专有名词变体） | `tuix/render/retained.rs:1`、`tuix/event_loop/commands.rs:10`、`codingplan/src/setup.rs:31`、`cli/main.rs:1`、`daemon/lib.rs:3`、`daemon/commands.rs:9`、`daemon/api_codingplan.rs:2`、`config/i18n/{zh_cn,en,messages}.rs` | 多 | **受 AGENTS.md:338 铁律保护，不得删**（闭源发行构建 i18n 契约） |
| `StatusCp*` | `tuix/event_loop/commands.rs:9`、`config/i18n/{zh_cn,en,messages}.rs`、`daemon/commands.rs:9` | 多 | 同上受保护 |

定义集中在 `crates/rustcode-config/src/i18n/messages.rs:320-395+`（与 02-tasks T7 节所述一致）。**结论：全部为活跃引用或被铁律保留，无安全可删项。**

## 类 2：`codingplan*` 相关符号的零引用者

`rg -n "codingplan" crates/ --stats -g '!target'` 命中面极广（codingplan crate 自身、config、tuix、daemon、cli、capabilities、coding、review、kernel 测试均有引用）。
**结论：未发现「零引用且可安全删除」的 `codingplan*` 符号。** 所有符号要么活跃消费，要么受 G7/铁律（Q3/Q4）保护。S2 档不移除 B/C 类，故本类无删除项。

## 类 3：跨 crate 重复实现（tuix vs daemon）

- `crates/rustcode-tuix/src/event_loop/commands.rs:4988-5060` 与 `crates/rustcode-daemon/src/commands.rs:641-705`：CodingPlan status 渲染器逐字重复约 65 行 ×2。
- **用户裁决 Q8 = 维持现状、仅登记**：抽取为共享实现会改动 `rustcode-daemon` 对 `rustcode-codingplan` 的 optional dep 语义、触碰 feature 传递链，收益有限（仅省 ~65 行）。列为未来架构评审项，**本 feature 不做**。

## 类 4：非本 feature 的失效 `rustcode-core/src/*` 路径注释（follow-up，超出本 feature codingplan 面）

下列为指向已退役 `rustcode-core` 的失效路径注释，**不属于本次 codingplan 清理范围**（T1 仅覆盖 codingplan 面 7 处），登记供未来独立清理 feature：

| 文件:行 | 失效路径 | 类型 | 是否应修 |
|---|---|---|---|
| `crates/rustcode-tuix/src/custom_commands.rs:1` | `// crates/rustcode-core/src/commands/mod.rs` | 模块头注释 | 是（follow-up） |
| `crates/rustcode-kernel/src/agent.rs:169` | `v1 (rustcode-core/src/agent/mod.rs:3064)` | 引用注释 | 是（follow-up） |
| `crates/rustcode-kernel/tests/fallible_stream.rs:154` | `rustcode-core/src/agent/mod.rs:3064` | 引用注释 | 是（follow-up） |
| `crates/rustcode-coding/src/persona.rs:2` | `rustcode-core/src/config/prompt_sections.rs` | 模块 doc 注释 | 是（follow-up） |
| `crates/rustcode-config/src/config/instructions.rs:1` | `// crates/rustcode-core/src/config/instructions.rs` | 模块头注释 | 是（follow-up） |
| `crates/rustcode-capabilities/src/tools/bash.rs:1241` | `rustcode-core/src/tool/bash.rs` | doc 注释 | 是（follow-up） |
| `crates/rustcode-capabilities/src/skills/render.rs:4` | `rustcode-core/src/skill_render.rs` | 模块 doc 注释 | 是（follow-up） |
| `crates/rustcode-review/src/impact_plan.rs:370-372` | `crates/rustcode-core/src/agent/mod.rs` | **diff 示例原始字符串** | **否（误报）**：是 `diff --git a/...` 示例内容，非路径注释，不得改动 |

**注意**：`impact_plan.rs:370-372` 为 git diff 示例体，属有意内容，列入「不修改」白名单。

## 验收

- 报告含 4 类条目，每条带 `文件:行` + 引用计数/处置标注 ✅
- 工作区零代码改动（`git status` 与第 0 步一致）✅
- 铁律保护项（`Cp*` 族、`codingplan_crypto_tests`、`LEGACY_CODINGPLAN_PREFIX`/`prefixes_for`/`codingplan_prefixes()`/`is_codingplan_provider_name()`、`atomgit` feature、`#[cfg(feature="codingplan")]` 块）均标注「受铁律保护，不得删」✅

## 结论

status=done，decision=review。下一跳：code-reviewer（G4）。
