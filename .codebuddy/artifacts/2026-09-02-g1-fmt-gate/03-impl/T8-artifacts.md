---
kind: impl
id: T8-artifacts
from: doc-writer
to: [project-manager]
feature: 2026-09-02-g1-fmt-gate
status: done
decision: proceed
requires: [T1, T2, T3, T4]
files_owned:
  - .codebuddy/artifacts/2026-09-02-g1-fmt-gate/05-test-report.md
  - .codebuddy/artifacts/2026-09-02-g1-fmt-gate/06-release.md
  - .codebuddy/artifacts/2026-09-02-g1-fmt-gate/03-impl/T8-artifacts.md
created: 2026-09-02
---

# T8 产物生成说明 — 测试报告与发布说明

## 1. 生成了什么

| 产物 | 路径 | 内容 |
|---|---|---|
| 测试报告 | `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/05-test-report.md`（**新建**） | 验证基线、逐门禁结果、逐套件明细、存量红演进、失败项归因、零回归论证、14 项未验证范围 |
| 发布说明 | `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/06-release.md`（**新建**） | 四段式交付清单（行为变化 / 风险 / 验证结果 / 已知未验证范围）+ 回滚方案 + 文档更新清单 + 术语一致性结论 + 唯一下一步 + 5 项上报 |
| 本说明 | `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/03-impl/T8-artifacts.md`（**新建**） | 即本文件 |

- **未改动任何其他文件**：未触碰 `crates/`、两个 README、`AGENTS.md`、`docs/`、
  以及 `.codebuddy/` 下其它交接件（含 `STATUS.md`、`HANDOFF-codingplan-legacy.md`、
  `03-impl/T6-docs-atomcode.md`）。
- **未执行任何构建 / 测试 / git 命令**（按硬性约束）。

---

## 2. 采用了哪些数据（全部为编排者实测，原样转录）

### 2.1 门禁结果

| 门禁 | 数据 |
|---|---|
| G1 `cargo fmt --check` | exit=0 / 0 处差异（清理前 19 处） |
| 编译 `cargo check -j 1 --workspace --all-targets` | exit=0 / 0 error |
| G3 `cargo test -j 1 --workspace --no-fail-fast` | 5481 passed / 1 failed；90 个测试目标，89 个全绿 |
| G6 遥测 SDK | 0 真实命中（sentry / posthog / mixpanel / amplitude） |
| G7 `atomcode` in crates/ scripts/ .github/ | 0 命中 |
| G8 `atomcode` in docs/architecture.md | 0 命中 |

### 2.2 各套件明细

`rustcode`(=cli) `--lib` 116/0（修复前 115/1）；`rustcode-tuix --lib` 2064/0（修复前 2059/5）；
`rustcode-review --lib` 100/0（修复前 99/1）；`rustcode-coding --lib` 430/0；
`rustcode-config --lib` 327/0；`rustcode-daemon --lib` 307/0；`rustcode-updater --lib` 41/0；
`rustcode-kernel` 全绿；`rustcode-capabilities --lib` 1475/**1**。

### 2.3 其它采用项

- 唯一失败：`mcp::registry::tests::trust_key_golden_matches_core_algorithm`（`DefaultHasher`
  跨工具链不稳定；`AGENTS.md:226` 铁律禁改）。
- 存量红演进：8 → 7 → 1，净减 7，零新增。
- A 组：10 文件 / 113 行纯格式；G9 证明（6 文件 `TOKENS_IDENTICAL`、4 文件零语义差异）；
  `git diff -w` 不可用于该证明（258 行假阳性）。
- B 组：7 处 locale 锁修复，未改任何断言。
- 关键教训：`summarise_multi_line_adds_line_count` 为 locale 无关型，**只持 `test_lock()`、不 set_locale**。
- 未修缺陷：`session_picker` 时间相关偶发（末次通过）。
- 并发会话风险：`2026-09-02-cleanup-codingplan-legacy` 同树作业、共享 `cli/src/main.rs`
  与 `docs/REFACTOR_DESIGN_PHASE1.md`。
- 环境约束：8GB cgroup 须 `-j 1`；daemon 端口 13456–13458；`setsid nohup`；日志路径不可共用。
- 子代理通道：累计 3 次派发失败；`doc-writer` 因不跑 cargo 而派发成功。
- 上一 feature 成果（仅作上下文）：OpenRouter 归因头 opt-in 默认关闭、遥测词义残留清零。

---

## 3. 本件独立复核到的实现状态（Grep / Read，非转录）

| # | 复核项 | 结果 |
|---|---|---|
| 1 | `crates/rustcode-review/src/review_tool.rs:963-964` | `let _g = pin_en();` 已落地 |
| 2 | `crates/rustcode-cli/src/acp/translate.rs:189-192` | `test_lock()` + `set_locale(Locale::En)` 已落地 |
| 3 | `crates/rustcode-tuix/src/event_loop/mod.rs` | 6 处锁全部落地：`:8710`、`:8846`、`:8906`、`:8916`、`:8936`、`:31043` |
| 4 | 同上 `:8841-8846`（locale 无关型） | **只持锁、未 set_locale**，且 `:8842-8845` 有防复发注释 |
| 5 | `AGENTS.md:196-218` | 门禁定义确认：**G2 = `cargo clippy -- -D warnings`**，与本轮执行的 `cargo check` 不同 → 已标注为未验证项 |
| 6 | `AGENTS.md:226` | trust_key 已知红原文确认（「不要随手改测试去凑绿」） |
| 7 | `AGENTS.md:231` / `:349` | locale 锁约定原文确认，与关键教训表述一致 |
| 8 | `AGENTS.md:282` / `:539` | 已带 `[SUPERSEDED 2026-09-02,见第三十二轮]` → **T6 交付件 U1 已闭环** |
| 9 | README.md:48-49 | T6 改动已落地（无 `atomgit_atomcode`） |
| 10 | `docs/features.md:113-114` | G7/G8 门禁定义行原样保留 |
| 11 | `Commands::Codingplan` in `crates/` | **0 命中** → 并发会话 T3 极可能已执行（与编排者数据矛盾，已上报 C1） |
| 12 | `CHANGELOG*` | 仓库根**不存在** CHANGELOG |

---

## 4. 信息缺口（Gap List）

| # | 缺口 | 说明 | 补齐方式 |
|---|---|---|---|
| G-1 | **`Commands::Codingplan` 状态矛盾** | 编排者数据称对方 T3「至今未执行」，本件 Grep 实测 0 命中，对方 `06-release.md` 自述已删 | 编排者 `git status` / `git log`；并复跑 `cargo fmt --check` 确认 `cli/src/main.rs` |
| G-2 | **`cli/src/main.rs:3363` 格式改动是否仍保留** | 无 shell、禁止跑 `cargo fmt`，本件无法核验 | 编排者复跑 `cargo fmt --check`（一条命令） |
| G-3 | **8 个基线存量红中 tuix 的 4 个 `summarise_*` 具体用例名** | 交接件只写 `summarise_* ×4`，未逐名记录 | 本件已按修复清单**推断**4 个名字并明确标注为推断（TEST-001 4.1） |
| G-4 | **`git diff --stat` / `git status`** | 本会话无 shell 工具 | 编排者已补齐（越界证明与 113 行闭合） |
| G-5 | **G6 完整正则未跑** | 本轮只验证 4 个 SDK 名，缺 `segment` / `google-analytics` / `googletagmanager` | 编排者按 `AGENTS.md:204` 复跑并人工判定假阳性 |
| G-6 | **G4 / G5 端到端入口** | 本轮未跑 | 编排者 |
| G-7 | **非默认 feature / 非 Linux / release profile** | 本轮未跑 | 见 06-release N4/N5 |
| G-8 | **`session_picker` 偶发率未量化** | 未做重复运行 | 用户裁决修法后补 |
| G-9 | **测试目标数 89 → 90 的差值未归因** | 历史记录为 89 | 编排者对比目标清单 |
| G-10 | **工具行为异常（记录备查）** | 本会话 Grep 的 `outputMode: count` 对单文件/目录返回 0，而 `outputMode: content` 有命中（同一 pattern、同一 path）。已全部改用 content 模式复核，结论不受影响 | 后续使用者：优先用 content 模式取数 |

---

## 5. 未做的事（明确声明）

- 未执行任何 `cargo` / `git` / `python3` / `sh` 命令。
- 未把未验证内容写成已验证：所有非编排者实测的推论均标注为「推断（未验证）」或列入缺口表。
- 未改写 `AGENTS.md` 的门禁定义；发现差异（G2 = clippy vs 实跑 check）后**只标注、不修改**。
- 未删除或改写任何他人的过程记录（T6 交付件、`STATUS.md`、`HANDOFF` 均原样保留）。
- 未重复验证 T6 的 4 份文档改动（该件自带交付与回滚说明，本件仅交叉引用）。

---

## 6. 结论

`status: done`、`decision: proceed`。两份产物已落盘并通过四段式完整性自检；
编排者需处置的唯一高优先项：**C1（并发会话 T3 状态与 `cli/src/main.rs` 格式 hunk 是否保留）**，
其余为 N1–N12 的补跑与 C2–C5 的裁决。
