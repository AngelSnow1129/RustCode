# 2026-09-02-g1-fmt-gate 看板

- 当前阶段：**冻结**（T1/T2/T3 均 done，G1 归零；等待并发会话 `cleanup-codingplan-legacy` 完成后恢复验证）
- 基线：branch=dev commit=8e772dbf worktree=dirty（上一轮 11 文件改动，本轮证实未触碰）

## 背景

上一轮完成 OpenRouter 归因 opt-in 与遥测词义残留清零后，唯一未达成的门禁是 **G1 格式门禁**：
`cargo fmt --check` exit=1，19 处差异分布在 4 个包（rustcode-cli / coding / tuix / updater）的 10 个文件。

`atomcode` 全库仅剩 9 处，全部位于 `README*.md` / `AGENTS.md` /
`docs/{UPSTREAM_RUSTCODE_LICENSE,UPSTREAM_CREDITS,THIRD_PARTY_NOTICES,platform-neutralization,features,REFACTOR_DESIGN_PHASE1}.md`，
性质为 MIT 归属声明与历史记录，上一轮已判定 **禁止删除**（合规风险）。O1 无剩余面，本轮不重复处理。

## 门禁

| 门禁 | 状态 | 依据 |
| G1 格式 | **pass** | `cargo fmt --check` exit=0，`^Diff in` 计数 0 |
| G2 编译 | **pass** | `cargo check -j 1 --workspace --all-targets` exit=0，0 error |
| G3 测试 | **pass**（仅 1 个文档化已知红） | `cargo test -j 1 --workspace --no-fail-fast`：**5481 passed / 1 failed**；90 个目标中 89 个全绿，唯一失败为铁律禁改的 `mcp::registry::tests::trust_key_golden_matches_core_algorithm`。详见「G3 全量验证」节 |
| G6 遥测 SDK | pass | 真实 SDK 0 命中 |
| G7 atomcode(crates/scripts/.github) | pass | 0 命中 |
| G8 atomcode(docs/architecture.md) | pass | 0 命中 |
| G9 纯格式证明 | **pass** | 见「G9 纯格式证明」节 |

## 任务

| id | 标题 | 批次 | 负责 Agent | 状态 | 返工轮次 | 交接件 |
| T1 | G1 格式门禁归零（19 处） | B1 | code-implementer（派发超时，编排者直接执行 rustfmt） | done | 0 | 本节记录 |
| T2 | 复核纯格式 + 零回归 | B2 | code-reviewer 派发失败；编排者以只读命令完成等效核验 | done | 0 | 本节记录 |
| T3 | review 缺 locale 锁修复（用户裁决「加 pin_en()」） | B3 | 编排者（经用户授权） | done | 0 | 本节记录 |
| T4 | tuix 5 + cli 1 缺 locale 锁修复 | B4 | 编排者（子代理通道 3 连败，改用直接执行） | done | 0 | 本节记录 |

**执行偏差记录（不得粉饰）**：T1/T2 的 `code-implementer` / `code-reviewer` 子代理派发
连续失败（一次 "No result found"、一次 idle timeout）。因 `cargo fmt` 是确定性格式化工具调用
而非业务代码编写，编排者直接执行，并以只读命令完成等效验证。子代理通道不稳定是环境事实。

## T1 改动范围（files_owned，10 个文件）

| 文件 | 变更行 |
|---|---|
| `crates/rustcode-cli/src/main.rs` | 3 |
| `crates/rustcode-cli/src/schedule_cmd.rs` | 7 |
| `crates/rustcode-coding/src/runtime.rs` | 3 |
| `crates/rustcode-tuix/src/event_loop/commands.rs` | 16 |
| `crates/rustcode-tuix/src/event_loop/mod.rs` | 43 |
| `crates/rustcode-tuix/src/modals/dir_picker.rs` | 19 |
| `crates/rustcode-tuix/src/modals/onboarding_wizard.rs` | 4 |
| `crates/rustcode-tuix/src/render/cell.rs` | 3 |
| `crates/rustcode-tuix/src/test_term.rs` | 7 |
| `crates/rustcode-updater/src/lib.rs` | 8 |

**越界证明**：21 个改动文件总计 283 增 / 104 删；上一轮 11 文件为 230/44；
差值 53 增 / 60 删 = 113 行，恰等于上表 10 文件行数之和（3+7+3+16+43+19+4+3+7+8=113）。
故本轮只动了这 10 个文件，上一轮 11 文件未被触碰。

## G9 纯格式证明

方法：剥离**全部空白字符**（含换行）后比对 HEAD 与工作区的字符序列。
（`git diff -w` 不可用——它只忽略行内空白，无法忽略 rustfmt 的换行合并。）

- 6 个文件 `TOKENS_IDENTICAL`（纯空白/换行）：cli/main.rs、cli/schedule_cmd.rs、
  coding/runtime.rs、tuix/modals/onboarding_wizard.rs、tuix/render/cell.rs、updater/lib.rs（共 6 个文件）
- 4 个文件用 Python 做 Unicode 安全比对并定位首个差异字符，性质如下：

| 文件 | 差异性质 | 语义影响 |
|---|---|---|
| `tuix/event_loop/commands.rs` | 删除单参调用冗余尾随逗号（`t(..),)` → `t(..))`） | 无 |
| `tuix/event_loop/mod.rs` | 结构体字面量尾随逗号增删 | 无 |
| `tuix/modals/dir_picker.rs` | 多行调用末参补尾随逗号 | 无 |
| `tuix/test_term.rs` | rustfmt `merge_derives` 合并相邻 `#[derive]` | 无 |

**结论**：改动仅限空白、换行、尾随逗号与 derive 合并，零语义变化。

## G3 结果：8 个失败，逐名比对零新增

| 目标 | 失败数 | 用例 | 归因 |
|---|---|---|---|
| `rustcode --lib` | 1 | `acp::translate::tests::policy_intervention_exposes_safe_recovery_without_secret_material` | 存量红（stash 基线已证实） |
| `rustcode-capabilities --lib` | 1 | `mcp::registry::tests::trust_key_golden_matches_core_algorithm` | 文档化已知红，`AGENTS.md` 铁律禁改 |
| `rustcode-review --lib` | 1 | `review_tool::tests::review_activity_line_composes_label_findings_and_tail` | 见下「定性修正」 |
| `rustcode-tuix --lib` | 5 | `event_loop::task_render_tests::result_non_task_output_falls_back` + `tool_format_tests::summarise_*` ×4 | 存量红（stash 基线逐名一致） |

单跑复验：`rustcode-coding --lib` **430/0**、`rustcode-updater --lib` **41/0** 全绿。

### 定性修正（推翻上一轮的错误结论）

上一轮把 review 那 1 个失败判为「locale 竞态，单跑 100/0」——**该结论不准确**。

实测断言：
```
left:  "评审 · thinking"    实际（中文）
right: "review · thinking"  期望（英文）
```
该测试断言英文输出却**未持 locale 锁**。默认语言已改为中文（O5 第十轮），
因此它拿到 `Locale::ZhCn` 即红。本轮 `--test-threads=1` 与单独过滤运行均 4/4 确定性失败；
上一轮单跑之所以 100/0，是**测试顺序运气**——别的 `pin_en()` 用例抢先设了 En。

正确定性：`AGENTS.md:349` 已定义的测试缺陷（"凡断言本地化输出的测试必须持锁钉死 locale"），
确定性失败，非偶发竞态。上一轮该条目描述需以此为准。

### daemon_token_auth 单次失败归因

一次全量运行曾报 `-p rustcode-daemon --test daemon_token_auth` 失败。**是我的操作失误**：
被工具超时中断的前一次后台进程未死，与新的 `setsid` 进程**并发跑两个全量测试**，
两个 daemon 测试二进制争用固定端口（13456-13458）致偶发。该用例事后单跑 **3/3 全绿**，非代码回归。

## 环境约束

- cgroup 内存上限 8GB，`cargo test` 默认并发触发 rustc SIGBUS，**必须 `-j 1`**。
- 工具会话超时约 60–90s，长任务须用 `setsid nohup ... &` 脱离会话后轮询；
  且**同一日志路径勿被两个进程共用**（本轮因此污染过一次日志）。
- `rustcode-cli` 的**包名是 `rustcode`**，`-p rustcode-cli` 会失败。

## T3：review 缺锁修复（用户裁决「加 pin_en()」，已完成）

- 改动：`crates/rustcode-review/src/review_tool.rs` **+1 行**，在
  `review_activity_line_composes_label_findings_and_tail` 首行加 `let _g = pin_en();`。
  **未改任何断言**，与同文件另外 10 个用例同构（helper 在 `:922`）。
- 范围核实：扫描 `:962–1028` 区间，`paths_match_*`（`:993`）断言路径匹配、
  `sort_findings_*`（`:1006`）断言输入数据，均不含本地化文本，无需加锁；
  `render_findings_*`（`:1019`）已持锁。故修复面精确到 1 处，无遗漏。
- 验证：`cargo test -p rustcode-review --lib` → **100/0**，且 `--test-threads=1`
  串行 + 默认并发共 **4/4 全绿**——证明是确定性转绿，而非又一次「顺序运气」。
- G1 复检：修改后 `cargo fmt --check` 仍 **exit=0 / 0 处差异**。

## [BLOCKER] 并发会话冲突 —— 需用户裁决

**本轮作业期间，另一编排会话正在同一 worktree 并发写入，已实测确认。**

- 证据：`ps` 抓到非本会话发起的 `cargo test -p rustcode-codingplan --lib sync_marker`；
  对应看板 `.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/STATUS.md`
  处于 **G3 实现中（T1 in_progress）**。
- 实测影响：本轮开始时 `git status` 为 11 个改动，收尾时为 **28 个**；
  新增的 6 个（`crates/rustcode-codingplan/src/{client,lib,setup,sync_marker,types}.rs`、
  `crates/rustcode-config/src/i18n/messages.rs`）**来自对方会话**，非本轮改动。
- **直接冲突点**：对方 **T3 任务**要删除 `crates/rustcode-cli/src/main.rs` 的
  `Commands::Codingplan` 别名（`:1696`、`:3573`，目前仍在，T3 尚未执行）；
  而本轮我刚格式化了同一文件的 `:3363`。其决策日志记载该文件的回滚策略是
  「非 dirty，可安全 `git checkout`」——**若对方执行回滚，会抹掉本轮格式改动**；
  反之亦然。两侧均无自动合并机制。
- 缓解事实：本轮 10 个 fmt 文件与对方已写入的 6 个文件**当前无交集**；
  对方写入后 G1 复检仍为 **exit=0 / 0 差异**（其改动亦已格式化）。
- **本轮已停止一切 worktree 写入**，等待裁决。全量 `cargo test --workspace`
  复跑因此未执行（与对方 cargo 争用 target 锁与 daemon 固定端口，会产生假红）。

### 用户裁决（2026-09-02）：**选项 1 —— 对方先跑完，我再验证**

据此本会话**即刻冻结一切 worktree 写入**，不再改动任何源码/文档（`AGENTS.md` 亦停笔），
仅保留本看板与交接件的维护。

已产出交接件：**`HANDOFF-codingplan-legacy.md`**（放在本看板目录内，
刻意不写入对方目录，避免覆盖对方正在写的 `STATUS.md`）。其中载明：
我方 11 个改动清单、冲突点与「请勿 `git checkout` 回滚 `cli/src/main.rs`」的警告、
worktree 快照、共用环境约束（`-j 1` / daemon 端口 / 日志路径）、以及恢复验证清单。

## T4：tuix 5 + cli 1 缺 locale 锁修复（已完成，零回归且**净减红**）

根因与 T3 **同类**：断言英文串却未钉 locale。实测断言
`left: "Result（3 行）"` vs `right: "Result (3 lines)"`（tuix 侧）、
`text.contains("separate terminal")` 不成立（cli，`acp/translate.rs:196`）。
依据 `AGENTS.md:231` 约定「断言英文串的测试均显式 `test_lock()` + `set_locale(En)`」。

### 修复清单（6 处，均**未改任何断言**）

| 文件 | 用例 | 处理 |
|---|---|---|
| `tuix/event_loop/mod.rs` | `summarise_mcp_result_strips_markdown_heading` | 锁 + set En |
| 同上 | `summarise_multi_line_still_appends_count` | 锁 + set En |
| 同上 | `summarise_read_result_collapses_line_number_and_indent` | 锁 + set En |
| 同上 | `summarise_read_result_falls_back_when_not_line_numbered` | 锁 + set En |
| 同上 | `task_render_tests::result_non_task_output_falls_back` | 锁 + set En |
| 同上 | `summarise_multi_line_adds_line_count` | **只持锁，不 set locale**（见下） |
| `cli/acp/translate.rs` | `policy_intervention_exposes_safe_recovery_without_secret_material` | 锁 + set En |

### 关键教训：locale 锁分两种，用错会制造新红

`summarise_multi_line_adds_line_count` 是**刻意与 locale 无关**的测试——它用
`i18n::t(Msg::TuixFoldLinesSuffix { count: 3 })` 现算期望后缀再比对，因此要求
`summarise()` 与 `t()` 两次调用之间 **locale 保持稳定**。

首次修复后我把它连同兄弟用例一起 set 成 En，反而**新增 2 个红**：
- `summarise_multi_line_adds_line_count` —— 我增多的 En 设置翻转了全局 locale，
  使 `summarise()` 与 `t()` 拿到不同 locale 而失配。正解是**只持 `test_lock()`
  拿稳定性，绝不 set_locale**（set 会破坏其 locale 无关设计）。
- `modals::session_picker::tests::replay_keeps_current_model_window_and_ignores_accounting_only_turn_positions`
  —— **与 locale 无关的独立缺陷**，详见下节。

### 结果（净减红，无新增）

| 套件 | 修复前 | 修复后 |
|---|---|---|
| `rustcode-tuix --lib` | 2059 / **5 个失败** | **2064 / 0** |
| `rustcode --lib` | 115 / **1 个失败** | **116 / 0** |
| `rustcode-review --lib` | 99 / **1 个失败** | **100 / 0**（T3） |
| `rustcode-coding --lib` | — | 430 / 0 |
| `rustcode-updater --lib` | — | 41 / 0 |
| `rustcode-capabilities --lib` | 1475 / **1 failed** | 仍 1（`trust_key`，铁律禁改） |

**存量红由 8 个降至 1 个。**
G1 复检 `cargo fmt --check` **exit=0 / 0 差异**；本轮触碰文件严格限于自有 10 个，
`cargo fmt -p` 未波及任何额外文件。

### 新发现的独立缺陷（未修，超出本轮范围）

`modals::session_picker::tests::replay_keeps_current_model_window_and_ignores_accounting_only_turn_positions`
断言 `!label.contains("987")`（987654 为其设的「仅记账」哨兵值），但标签中渲染了
会话 ID `session-1788360798786`，该毫秒时间戳数字串**恰好含子串 "987"** 时断言失败。
**与时间相关的固有偶发缺陷**，与 locale 无关、与本轮改动无关（本轮最后一次运行该用例通过）。
修法需改哨兵值（如改判 `total_tokens` 字段而非字符串包含）或改用固定会话 ID，
涉及测试语义，**待用户裁决**。

### 子代理通道状态

T4 首次尝试派发 2 个 `code-implementer`（tuix / cli 各一，`files_owned` 不相交），
**两个均因 idle timeout 被取消**，且未落盘任何交接件、未产生任何代码改动
（已核验 `03-impl/` 目录不存在、目标文件无新改动）。cargo 编译期间无增量输出触发空闲超时。
**累计 3 次派发失败**（1 次 "No result found" + 2 次 idle timeout），通道判定为不可用，
本轮改用编排者直接执行 + 只读命令验证，未绕过任何门禁。

## G3 全量验证（2026-09-02，用户裁决「先跑全量验证锁定 G3」）

命令：`cargo test -j 1 --workspace --no-fail-fast`（后台 `setsid` 执行，唯一日志 `/tmp/g3_verify.log`，
启动前已确认 `pgrep -c cargo = 0`，规避上一轮双进程污染日志 / 争用 daemon 端口的问题）。

**结果：5481 passed / 1 failed，90 个测试目标中 89 个全绿。**

| 目标 | 结果 |
|---|---|
| `rustcode-cli`(`rustcode`) `--lib` | **116 / 0**（修复前 115/1） |
| `rustcode-tuix --lib` | **2064 / 0**（修复前 2059/5） |
| `rustcode-review --lib` | **100 / 0**（修复前 99/1） |
| `rustcode-coding --lib` | 430 / 0 |
| `rustcode-config --lib` | 327 / 0 |
| `rustcode-daemon` `--lib` | 307 / 0 |
| `rustcode-updater --lib` | 41 / 0 |
| `rustcode-kernel` | 全绿 |
| `rustcode-capabilities --lib` | 1475 / **1** —— `trust_key_golden_matches_core_algorithm` |

**唯一失败为 `AGENTS.md:226` 文档化已知红**（`project_trust_key` 用
`std::collections::hash_map::DefaultHasher`，输出不保证跨工具链稳定），
`AGENTS.md` 明载「不要随手改测试去凑绿」，**铁律禁改**。

### 存量红演进

| 阶段 | 存量红数 |
|---|---|
| 本 feature 开始（第二十九轮收尾） | 8 |
| 修复 review 缺锁（T3） | 7 |
| 修复 tuix 5 + cli 1 缺锁（T4） | **1**（仅 trust_key） |

**净减 7 个红，零新增失败。**

## 对方 T3 落地后的恢复验证（已完成）

`doc-writer` 在生成产物时实测 `crates/` 下 `Commands::Codingplan` **0 命中**，
上报「编排者数据已过期」——经核验属实：**对方 T3 已执行完毕**。
据此触发本会话当初冻结时约定的恢复验证，结果如下：

| 检查 | 结果 |
|---|---|
| 我的 fmt hunk(`cli/src/main.rs:3363`)是否存活 | **存活** —— 与对方 T3 的删除在同一文件共存 |
| `cli/src/main.rs` 合并后 diff | 4 增 15 删（含我的 fmt hunk） |
| G1 `cargo fmt --check` | **exit=0 / 0 差异** |
| G2 `cargo check -j 1 --workspace --all-targets` | **exit=0 / 0 错误** |
| G3 `cargo test -j 1 --workspace --no-fail-fast` | **5481 passed / 1 failed**，89 目标全绿 |
| 唯一失败 | `mcp::registry::tests::trust_key_golden_matches_core_algorithm`（已知红，铁律禁改） |

**G3 与 T3 落地前完全一致**——对方删除 `Commands::Codingplan` 未破坏任何东西，双方改动共存无损。

## 流水线检查（`.github/`，用户指定项）

- **`.github/` 无任何改动**（`git status --short -- .github/` 为空），本会话与对方会话均未触碰。
- 现有 job：`ci.yml` 三个 —— `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets`、
  `cargo test --workspace`。分别对应 G1 / G2 / G3。
- **G6/G7/G8 无 CI job**，仅为本地人工门禁（与 `AGENTS.md:220` 记载一致）。
- **影响推论**：① 本 feature 修好 19 处格式违规，**使 CI 的 `fmt` job 由红转绿**（此前必然失败）；
  ② CI 的 `test` job 跑裸 `cargo test --workspace`，而 `trust_key` 是确定性失败，
  **该 job 仍将为红**——这是既有状态，非本轮引入；③ `clippy` job 未加 `-D warnings`
  （源码内有 TODO 注明待 ~420 条存量告警收敛后再收紧），故不会失败。
- **CI 未覆盖 G6/G7/G8** 属既有缺口，是否补 job 需用户裁决（本轮未改流水线）。

## 产物清单（用户指定项）

| 路径 | 内容 |
|---|---|
| `05-test-report.md` | 验证基线、逐门禁、逐套件、存量红演进、失败归因、零回归论证、未验证范围 |
| `06-release.md` | 四段式交付清单（行为变化/风险/验证结果/已知未验证）+ 回滚方案 + 唯一下一步 |
| `03-impl/T6-docs-atomcode.md` | README 与用户文档 `atomcode` 清理说明 |
| `03-impl/T7-goals-superpowers.md` | `.goals/` `.superpowers/` 清理说明 |
| `03-impl/T8-artifacts.md` | 产物生成说明与信息缺口 |
| `HANDOFF-codingplan-legacy.md` | 给并发会话的交接件 |

## `.goals/` `.superpowers/` 清理结果

- `.goals/`：2 文件 3 增 3 删；**残留 6 处全为 G7/G8 门禁定义与验收证据（必留）**。
  `inspector-feedback-1.md` 零改动（全文仅含门禁验收证据）。
- `.superpowers/`：**残留 0**。修正 4 处失效命令（`atomcode-tuix/capabilities/daemon` → `rustcode-*`，
  以及任务书未点名的第 4 处 `cargo check -p atomcode` → `-p rustcode`，映射依据
  `crates/rustcode-cli/Cargo.toml:2` 的 `name = "rustcode"`，非臆测）。
- **[CORRECTION] 子代理判断有误已纠正**：它认为 `.superpowers/` 被 gitignore 忽略、改动不会入库。
  实测 `git ls-files` 显示该文件**已被跟踪**，`git status` 显示 ` M`——**会进入提交**
  （已跟踪文件不受 gitignore 约束）。是否保留该修改需用户决定。

### 恢复条件（对方 T1/T3 完成后）

1. `cargo fmt --check` → 须 exit=0；若对方 T3 编辑引入新违规，由对方 `cargo fmt -p rustcode`
2. `cargo check -j 1 --workspace --all-targets` → 须 0 error
3. `cargo test -j 1 --workspace --no-fail-fast` → 与已知红基线逐名比对，零新增；
   `rustcode-review --lib` 须为 **100/0**（本轮已修，不应再红）
4. 确认 `crates/rustcode-cli/src/main.rs` 同时保留我方 `:3363` 格式改动与对方 T3 的删除结果

**冻结时实测快照**：改动文件 29 个；无 cargo 进程在跑；
对方 T3 目标 `Commands::Codingplan` 仍在 `:1696`/`:3573`（未执行）；
对方 T1 已落 6 文件 7 增 7 删；G1 仍 exit=0 / 0 差异。

## [CLOSED] 闭板记录（2026-09-03，只追加，不改写既有行）

- **结论：G1/G2/G3 全通过，T1–T4 全 done，冻结解除。**
- 门禁终态：

| 门禁 | 终态 | 依据 |
|---|---|---|
| G1 格式 | pass | `cargo fmt --check` exit=0，19 处差异已归零（本看板使 CI 的 `fmt` job 由红转绿） |
| G2 编译 | pass | `cargo check -j 1 --workspace --all-targets` 0 error |
| G3 测试 | pass | `cargo test -j 1 --workspace --no-fail-fast` = **5481 passed / 1 failed**，90 个目标中 89 个全绿；唯一失败为铁律禁改的 `trust_key_golden_matches_core_algorithm` |
| G6–G9 | pass | 遥测 SDK 0 命中；`atomcode` 在 crates/scripts/.github 与 docs/architecture.md 均 0 命中；纯格式证明 10 文件零语义变化 |

- 「冻结」状态解除依据：当初冻结是为规避与并发会话 `cleanup-codingplan-legacy` 的文件冲突
  （直接冲突点为 `crates/rustcode-cli/src/main.rs`）。该会话已完成并入库（`783d48e4`），
  恢复验证四项全部通过：我方 fmt hunk 存活 / `fmt --check` exit=0 / `check` 0 error /
  测试与冻结前逐名一致（5481/1）。双方改动共存无损。
- **本看板登记的两项遗留已转移并完成**：

| 遗留项 | 转移路径 | 终态 |
|---|---|---|
| `docs/multi-agent-collaboration-solution.md` 入库 | → `2026-09-03-residual-two-items` T2 → `2026-09-03-git-wrapup` **GW-02** | done（doc-writer 校验，修正 1 处悬空示例路径） |
| session_picker 偶发红修复 | → `2026-09-03-residual-two-items` T1 → `2026-09-03-git-wrapup` **GW-03** | done（钉定会话名；tuix 2064/0 + 20/20 确定性循环 + fmt exit=0） |

- **承接看板**：`2026-09-03-git-wrapup`。
