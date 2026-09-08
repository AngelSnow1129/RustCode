---
kind: test-report
id: TEST-001
from: doc-writer
to: [project-manager]
feature: 2026-09-02-g1-fmt-gate
status: done
decision: proceed
requires: [T1, T2, T3, T4]
files_owned:
  - .codebuddy/artifacts/2026-09-02-g1-fmt-gate/05-test-report.md
created: 2026-09-02
---

# TEST-001 测试报告 — G1 格式门禁归零 + locale 缺锁修复

## 0. 本件性质与数据来源（先读）

- [INFO] **本件作者（doc-writer）未执行任何构建 / 测试 / git 命令**。全部数字与命令输出均来自编排者
  `2026-09-02-g1-fmt-gate` 会话的实测记录（看板 `STATUS.md` 与交接件
  `HANDOFF-codingplan-legacy.md`），本件按原样转录，**未推断、未改写、未补齐**。
- [INFO] 本件作者无 shell 工具，**无法**复核 `git diff --stat` / `git status` / `git stash` 相关结论
  （第 6 节的越界证明与行数表均来自编排者实测）。
- [CHECK] 本件作者对**实现状态**做了只读复核（Grep 逐项命中，见第 5.3 节），确认 7 处 locale 锁
  已真实落盘；此项为本件独立完成，非转录。
- [WARN] 门禁编号口径差异：`AGENTS.md:196-218` 定义的 **G2 是 `cargo clippy --workspace --all-targets
  -- -D warnings`**，而本轮实际执行的是 **`cargo check -j 1 --workspace --all-targets`**（仅编译检查，
  弱于 clippy）。本件按实际执行的命令记录，不把 `cargo check` 记成 G2 通过 —— 详见第 8 节 U1。

---

## 1. 验证基线

### 1.1 代码基线（编排者实测）

| 项 | 值 |
|---|---|
| 分支 | `dev` |
| 提交 | `8e772dbf` |
| 工作树 | **dirty** |
| dirty 构成 | 上一轮（OpenRouter 归因 opt-in / 遥测词义清零）遗留 11 个文件 + 并发会话 `2026-09-02-cleanup-codingplan-legacy` 写入的 6 个文件 + 本 feature 13 个文件（10 格式 + 3 锁） |
| 冻结时改动文件总数 | 29（含未跟踪：`docs/multi-agent-collaboration-solution.md`、`.codebuddy/`） |

[WARN] 基线不是干净树。所有结论以「与 dirty 基线逐名比对、零新增」为准，**不是**「绝对零失败」。

### 1.2 执行命令与参数（编排者实测，逐字）

```bash
# G1 格式门禁
cargo fmt --check                                  # exit=0，`^Diff in` 计数 0

# 编译检查（注意：非 AGENTS.md:200 定义的 clippy）
cargo check -j 1 --workspace --all-targets         # exit=0，0 error

# G3 全量测试（后台脱离会话，唯一日志，启动前确认无并发 cargo）
pgrep -c cargo                                     # 须为 0
setsid nohup cargo test -j 1 --workspace --no-fail-fast > /tmp/g3_verify.log 2>&1 &

# G6 遥测 SDK
grep -riE "sentry|posthog|mixpanel|amplitude"      # 0 真实命中

# G7 / G8
grep -rni "atomcode" crates/ scripts/ .github/     # 0 命中
grep -rni "atomcode" docs/architecture.md          # 0 命中

# G9（本 feature 自定义证明门禁，不在 AGENTS.md 门禁表内）
# 剥离全部空白字符（含换行）后比对 HEAD 与工作区的字符序列
```

### 1.3 环境约束（强制，复跑者须遵守）

| 约束 | 事实 | 后果 |
|---|---|---|
| 内存 | cgroup 上限 **8GB** | `cargo test` 默认并发触发 rustc **SIGBUS**，**必须 `-j 1`** |
| 端口 | daemon 测试占用固定端口 **13456–13458** | 两个 cargo 测试进程并发会争用并产生**假红**（`daemon_token_auth` 曾因此假红，事后单跑 3/3 全绿） |
| 会话 | 工具会话超时约 60–90s | 长任务须 `setsid nohup ... &` 脱离会话后轮询 |
| 日志 | 同一日志路径不可被两进程共用 | 本轮曾因此污染日志，出现重复的失败目标条目 |
| 包名 | `rustcode-cli` 的 crate 名是 **`rustcode`** | `-p rustcode-cli` 会失败 |

---

## 2. 逐门禁结果

| 门禁 | 命令（实际执行） | 结果 | 判定 |
|---|---|---|---|
| G1 格式 | `cargo fmt --check` | **exit=0 / 0 处差异**（清理前 19 处） | pass |
| G2（AGENTS.md:200 定义） | `cargo clippy --workspace --all-targets -- -D warnings` | **未执行** | **未验证**（见 U1） |
| 编译检查（实际执行） | `cargo check -j 1 --workspace --all-targets` | exit=0 / **0 error** | pass（弱于 G2） |
| G3 测试 | `cargo test -j 1 --workspace --no-fail-fast` | **5481 passed / 1 failed**；**90 个测试目标，89 个全绿** | pass（1 个文档化已知红） |
| G4 headless | `./scripts/test-headless.sh` | **未执行** | 未验证（见 U2） |
| G5 ACP 冒烟 | `python3 scripts/acp_smoke.py` | **未执行** | 未验证（见 U3） |
| G6 遥测 SDK | `grep -riE "sentry\|posthog\|mixpanel\|amplitude"` | **0 真实命中** | pass |
| G7 `atomcode`（crates/scripts/.github） | `grep -rni "atomcode" crates/ scripts/ .github/` | **0 命中** | pass |
| G8 `atomcode`（docs/architecture.md） | `grep -rni "atomcode" docs/architecture.md` | **0 命中** | pass |
| G9 纯格式证明（feature 自定义） | 剥离全空白后字符序列比对 + Python 定位首差异 | 6 文件完全一致 / 4 文件差异为零语义 | pass |

[INFO] G3 的「1 failed」是 **`AGENTS.md:226` 列明的文档化已知红**，不是本 feature 引入；
按门禁口径「与已知红基线逐名比对、零新增」判定 pass。

---

## 3. 逐套件明细（`cargo test -j 1 --workspace --no-fail-fast`）

| 目标 | 结果 | 修复前 | 说明 |
|---|---|---|---|
| `rustcode`(=cli) `--lib` | **116 / 0** | 115 / **1** | 修复 `acp/translate.rs` 缺锁 |
| `rustcode-tuix --lib` | **2064 / 0** | 2059 / **5** | 修复 `event_loop/mod.rs` 5 处缺锁 |
| `rustcode-review --lib` | **100 / 0** | 99 / **1** | T3，修复 `review_tool.rs` 缺锁 |
| `rustcode-coding --lib` | 430 / 0 | — | 未受影响，单跑复验亦全绿 |
| `rustcode-config --lib` | 327 / 0 | — | 未受影响 |
| `rustcode-daemon --lib` | 307 / 0 | — | 曾因双进程争端口假红，非回归（见 5.2） |
| `rustcode-updater --lib` | 41 / 0 | — | 未受影响，单跑复验亦全绿 |
| `rustcode-kernel` | 全绿 | — | 未受影响 |
| `rustcode-capabilities --lib` | 1475 / **1** | 1475 / **1** | 唯一失败 = `trust_key_golden_matches_core_algorithm`（铁律禁改） |

**合计 5481 passed / 1 failed；90 个测试目标中 89 个全绿。**

[INFO] 本轮覆盖到的入口：CLI 库（`--lib`）、TUI 库（`--lib`）、review / coding / config / daemon /
updater / kernel / capabilities 各 crate 的单元与集成目标（默认 feature，debug profile）。
未覆盖入口见第 8 节。

---

## 4. 存量红演进

| 阶段 | 存量红数 | 说明 |
|---|---|---|
| 本 feature 开始（上一轮收尾） | **8** | 逐名见下 |
| 修复 review 缺锁（T3） | **7** | `review_activity_line_composes_label_findings_and_tail` 转绿 |
| 修复 tuix 5 + cli 1 缺锁（T4） | **1** | 仅剩 `trust_key_golden_matches_core_algorithm` |

**净减 7，零新增。**

### 4.1 8 个基线存量红逐名（stash 基线已证实）

| # | 目标 | 用例 | 归因 | 现状态 |
|---|---|---|---|---|
| 1 | `rustcode --lib` | `acp::translate::tests::policy_intervention_exposes_safe_recovery_without_secret_material` | 断言英文、未钉 locale | 已修（+3 行锁） |
| 2 | `rustcode-capabilities --lib` | `mcp::registry::tests::trust_key_golden_matches_core_algorithm` | `DefaultHasher` 跨工具链不稳定 | **仍红，铁律禁改** |
| 3 | `rustcode-review --lib` | `review_tool::tests::review_activity_line_composes_label_findings_and_tail` | 断言英文、未钉 locale | 已修（+1 行 `pin_en()`） |
| 4 | `rustcode-tuix --lib` | `event_loop::task_render_tests::result_non_task_output_falls_back` | 断言英文、未钉 locale | 已修（锁 + set En） |
| 5–8 | `rustcode-tuix --lib` | `event_loop::tool_format_tests::summarise_*` ×4 | 断言英文、未钉 locale | 已修（锁 + set En） |

[INFO] 上表 5–8 的 4 个 `summarise_*` 具体用例名**未逐名记录于交接件**，本件按 `STATUS.md` 原文转录。
[INFO] **推断（未验证）**：据修复清单反推，该 4 个应为
`summarise_mcp_result_strips_markdown_heading` / `summarise_multi_line_still_appends_count` /
`summarise_read_result_collapses_line_number_and_indent` /
`summarise_read_result_falls_back_when_not_line_numbered`；
同模块的 `summarise_multi_line_adds_line_count` 当时是绿的（locale 无关型），不在 8 个之内。

### 4.2 定性修正（推翻上一轮的错误结论）

上一轮把 review 那 1 个失败判为「locale 竞态，单跑 100/0」——**该结论不准确**。

```
left:  "评审 · thinking"    实际（中文）
right: "review · thinking"  期望（英文）
```

该测试断言英文输出却**未持 locale 锁**。默认语言已改为中文，因此拿到 `Locale::ZhCn` 即红。
本轮 `--test-threads=1` 与单独过滤运行均 **4/4 确定性失败**；上一轮单跑之所以 100/0，
是**测试顺序运气**——别的 `pin_en()` 用例抢先设了 En。

正确定性：`AGENTS.md:349` 已定义的测试缺陷类型（「凡断言本地化输出的测试必须持锁钉死 locale」），
**确定性失败，非偶发竞态**。上一轮该条目描述需以此为准。

---

## 5. 失败项归因

### 5.1 唯一残留失败：`trust_key_golden_matches_core_algorithm`

- 位置：`rustcode-capabilities --lib`，`mcp::registry::tests`。
- 根因：`project_trust_key` 用 `std::collections::hash_map::DefaultHasher`，**输出不保证跨工具链稳定**。
- 处置：**不修**。`AGENTS.md:226` 将其列为文档化已知红，明载「不要随手改测试去凑绿」，**铁律禁改**。
- 对门禁的含义：G3 的判据是「逐名比对、零新增」，**不是**「必须 0 失败」。

### 5.2 `daemon_token_auth` 单次失败 —— 已归因，非代码回归

一次全量运行曾报 `-p rustcode-daemon --test daemon_token_auth` 失败。**是操作失误**：
被工具超时中断的前一次后台进程未死，与新的 `setsid` 进程**并发跑两个全量测试**，
两个 daemon 测试二进制争用固定端口（13456–13458）致偶发。该用例事后单跑 **3/3 全绿**。

### 5.3 7 处修复的落盘状态（本件 Grep 独立复核，非转录）

| 文件 | 用例 | 复核结果 |
|---|---|---|
| `crates/rustcode-review/src/review_tool.rs:963-964` | `review_activity_line_composes_label_findings_and_tail` | [CHECK] `let _g = pin_en();` 已落地 |
| `crates/rustcode-cli/src/acp/translate.rs:189-192` | `policy_intervention_exposes_safe_recovery_without_secret_material` | [CHECK] `test_lock()` + `set_locale(Locale::En)` 已落地 |
| `crates/rustcode-tuix/src/event_loop/mod.rs:8709-8710` | `summarise_mcp_result_strips_markdown_heading` | [CHECK] 已持锁 |
| 同上 `:8841-8846` | `summarise_multi_line_adds_line_count` | [CHECK] **只持 `test_lock()`，未 set_locale**（符合 locale 无关型设计） |
| 同上 `:8905-8906` | `summarise_multi_line_still_appends_count` | [CHECK] 已持锁 |
| 同上 `:8915-8916` | `summarise_read_result_collapses_line_number_and_indent` | [CHECK] 已持锁 |
| 同上 `:8935-8936` | `summarise_read_result_falls_back_when_not_line_numbered` | [CHECK] 已持锁 |
| 同上 `:31042-31043` | `task_render_tests::result_non_task_output_falls_back` | [CHECK] 已持锁 |

[CHECK] 复核结论：**7 处修复全部在盘**，与报告一致；`summarise_multi_line_adds_line_count`
确为「只持锁、不 set_locale」，关键教训已在代码中留注释（`:8842-8845`）防复发。

### 5.4 关键教训：locale 锁分两种，用错会制造新红

`summarise_multi_line_adds_line_count` 是**刻意与 locale 无关**的测试——它用
`i18n::t(Msg::TuixFoldLinesSuffix { count: 3 })` 现算期望后缀再比对，因此要求
`summarise()` 与 `t()` 两次调用之间 **locale 保持稳定**。

首次修复时把它和兄弟用例一起 set 成 En，反而**新增 2 个红**：

1. `summarise_multi_line_adds_line_count` —— 增多的 En 设置翻转了全局 locale，
   使 `summarise()` 与 `t()` 拿到不同 locale 而失配。正解是**只持 `test_lock()`
   拿稳定性，绝不 `set_locale`**（set 会破坏其 locale 无关设计）。
2. `modals::session_picker::tests::replay_keeps_current_model_window_and_ignores_accounting_only_turn_positions`
   —— 与 locale 无关的**独立缺陷**，见 5.5。

**给断言英文的用例补锁前，必须先判明同模块是否存在 locale 无关型兄弟用例。**

### 5.5 未修的已知缺陷（非本轮引入，末次运行通过）

`modals::session_picker::tests::replay_keeps_current_model_window_and_ignores_accounting_only_turn_positions`
断言 `!label.contains("987")`（`987654` 为其「仅记账」哨兵值），但标签渲染了会话 ID
`session-1788360798786`，该毫秒时间戳数字串**恰好含子串 "987"** 时即失败。

- 性质：**与时间相关的固有偶发缺陷**，与 locale 无关、与本轮改动无关（末次运行通过）。
- 修法候选：改判 `total_tokens` 字段而非字符串包含，或改用固定会话 ID。
- 状态：**待用户裁决**（涉及测试语义，不在本 feature 范围）。
- [WARN] 未做重复运行以量化偶发率（见 U11）。

---

## 6. 零回归论证

### 6.1 论证链（四条独立证据）

1. **stash 基线逐名比对**：本 feature 开始时（含上一轮遗留 11 文件）全工作区基线为 **8 个失败**，
   与修复后残留的 1 个**逐名一致**；被修掉的 7 个均在基线中存在。
   → 结论：**本 feature 引入的新失败 = 空集**。
2. **断言零改动**：7 处修复只新增锁行（review +1、cli +3、tuix 5 处锁 + 1 处只锁），
   **未改任何断言、未改任何产品代码**。修复面精确到「缺锁」这一类缺陷，不具备改变业务行为的路径。
   （落盘状态由本件 5.3 节 Grep 独立复核。）
3. **G9 纯格式证明**（A 组 10 文件 / 113 行）：
   - 方法：剥离**全部空白字符（含换行）**后比对 HEAD 与工作区的字符序列。
   - 结果：**6 个文件 `TOKENS_IDENTICAL`**（纯空白/换行）——cli/main.rs、cli/schedule_cmd.rs、
     coding/runtime.rs、tuix/modals/onboarding_wizard.rs、tuix/render/cell.rs、updater/lib.rs（共 6 个文件）；
     **4 个文件**用 Python 做 Unicode 安全比对并定位首个差异字符，性质均为零语义：

     | 文件 | 差异性质 | 语义影响 |
     |---|---|---|
     | `tuix/event_loop/commands.rs` | 删除单参调用冗余尾随逗号（`t(..),)` → `t(..))`） | 无 |
     | `tuix/event_loop/mod.rs` | 结构体字面量尾随逗号增删 | 无 |
     | `tuix/modals/dir_picker.rs` | 多行调用末参补尾随逗号 | 无 |
     | `tuix/test_term.rs` | rustfmt `merge_derives` 合并相邻 `#[derive]` | 无 |

   - **方法学注记**：`git diff -w` **不能**用于此证明（只忽略行内空白，会把换行合并报成
     **258 行假阳性**）。
4. **越界证明（行数闭合）**：21 个改动文件总计 **283 增 / 104 删**；上一轮 11 文件为 230/44；
   差值 **53 增 / 60 删 = 113 行**，恰等于 A 组 10 文件行数之和
   （3+7+3+16+43+19+4+3+7+8 = **113**）。
   → 结论：本轮只动了这 10 个格式文件 + 3 个锁文件，上一轮 11 文件未被触碰。

### 6.2 修复后的门禁复检

- `cargo fmt --check`：**exit=0 / 0 处差异**（T3/T4 加锁后复检，G1 未被锁修复破坏）。
- `cargo test -p rustcode-review --lib`：**100/0**，且 `--test-threads=1` 串行 + 默认并发共
  **4/4 全绿** —— 证明是**确定性转绿**，而非又一次「顺序运气」。
- `cargo fmt -p` 未波及任何额外文件。

### 6.3 论证边界（不得过度解读）

- 上述「零回归」成立于**默认 feature、debug profile、Linux x86_64** 与**本基线 dirty 树**。
- 它**不能**外推到：非默认 feature 组合、release profile、其它操作系统、headless/ACP 端到端
  入口、WebUI/文档站（均未跑，见第 8 节）。

---

## 7. 结论

- G1 格式门禁由 **exit=1 / 19 处差异** 归零为 **exit=0 / 0 处差异**，且经 G9 证明为纯格式。
- 存量红 **8 → 1**，净减 7，**零新增失败**；唯一残留为铁律禁改的文档化已知红。
- G6 / G7 / G8 均 pass；编译检查 0 error。
- 判定：**pass（按「逐名比对、零新增」口径）**，可交付至 RELEASE-002。

---

## 8. 未验证范围（Known Unverified Scope）

| # | 未覆盖项 | 影响 | 建议负责人 / 补齐方式 |
|---|---|---|---|
| U1 | **`cargo clippy` 全量未跑**（`AGENTS.md:200` 的 G2 真义）。本轮只跑了 `cargo check`（编译检查，不产生 lint） | 无法排除本轮改动引入新 clippy warning；`AGENTS.md:220` 记有约 420 条存量 warning（`:225` 记 clippy 仍有 per-crate warnings，非 errors） | 编排者：在更大内存环境跑 `cargo clippy --workspace --all-targets`（不加 `-D warnings`），与存量基线比对 |
| U2 | **G4 `./scripts/test-headless.sh` 未跑**（需先 `cargo build`） | headless 端到端入口未覆盖 | 编排者：`cargo build` 后执行该脚本 |
| U3 | **G5 `python3 scripts/acp_smoke.py` 未跑** | ACP 协议冒烟入口未覆盖 | 编排者 |
| U4 | **非默认 feature 组合未验证**（`codingplan` / `codingplan-crypto` / `plugin` 等） | A 组格式改动涉及 cli/coding/tuix/updater；`codingplan` 门控代码路径未编译验证 | 编排者：`cargo check --workspace --all-targets --features <matrix>` |
| U5 | **仅 Linux x86_64**；Windows / macOS 未验证 | 平台相关路径（updater、shell、路径规范化）未覆盖 | 各平台 CI（`build.yml` 已有多平台 job） |
| U6 | **仅 debug profile**；`--release` 构建与测试未跑 | release 下 `debug_assert!`/溢出行为未覆盖 | 发布流程 |
| U7 | **WebUI `npm test` / 站点构建未跑** | 与本 feature 无直接关联（零前端改动），但 T6 文档改动可能影响 `site/` 搜索索引 | 站点维护者：`cd site && node build-search-index.mjs` 重生成（若索引由 md 生成） |
| U8 | **daemon 固定端口假红风险**：本次全量跑前已 `pgrep -c cargo = 0`，但无法排除非 cargo 进程占用 13456–13458 | 理论上仍可能污染 daemon 结果 | 复跑者：跑前确认端口空闲 |
| U9 | **本件作者无 shell**：`git diff --stat` / `git status` / `git stash` 均无法复核 | 越界证明（6.1 第 4 条）与 113 行闭合为编排者实测，本件未独立验证 | 编排者已补齐（见 6.1）；如仍需二次确认，`git diff --stat` 一条命令即可 |
| U10 | **并发会话 `2026-09-02-cleanup-codingplan-legacy` 的 T3 状态存疑**：编排者数据称「至今未执行」，但本件 Grep 实测 `Commands::Codingplan` 在 `crates/` 下 **0 命中**，且其 `06-release.md` 自述 T3 已完成 | 若 T3 已执行，`crates/rustcode-cli/src/main.rs` 被双方先后编辑，本轮 `:3363` 格式改动是否保留**无证据**；G1 结论需重跑确认 | 编排者：`git status` / `git log` 确认，并复跑 `cargo fmt --check` |
| U11 | **`session_picker` 偶发缺陷未量化**：未做 N 次重复运行测其偶发率 | 未来 CI 上可能随机变红，与本轮改动无关但会被误判为回归 | 测试维护者：待用户裁决修法后补重复运行 |
| U12 | **测试目标数 89 → 90 的差异未逐项归因**（`AGENTS.md:360` 记载 89 个测试二进制） | 无法确认新增的 1 个目标是什么；不影响「零新增失败」结论（逐名比对已覆盖） | 编排者：对比两次运行的目标清单 |
| U13 | **G6 的正则未取 AGENTS.md:204 的完整集合**（缺 `segment` / `google-analytics` / `googletagmanager`） | 本轮只验证了四个 SDK 名；完整集合下的已知假阳性（`AGENTS.md:206`：英文单词 `segment`、seed skill 里的 "Sentry MCP"）需人工判定 | 编排者：按 `AGENTS.md:204` 完整正则复跑并逐条人工判定 |
| U14 | **提交态未产生**：29 个 dirty 文件全部未提交，本 feature 改动与上一轮遗留、并发会话改动共存 | 「测试通过」不等于「已合入」 | 见 RELEASE-002 第 6/8 节 |
