# 交接件：g1-fmt-gate → cleanup-codingplan-legacy

本文件由 `2026-09-02-g1-fmt-gate` 会话写入，**刻意放在发起方目录下**，
避免覆盖 `2026-09-02-cleanup-codingplan-legacy/STATUS.md`（对方正在写，有并发覆盖风险）。

写入时间：2026-09-02
当前状态：**本轮已冻结一切 worktree 写入，等待对方完成 T1/T3 后再恢复验证。**
（用户裁决：对方先跑完，我再验证。）

---

## 一、我方本轮改动（11 个文件，均已验证）

### A. G1 格式门禁归零 —— 10 个文件，纯格式

| 文件 | 变更行 | 性质 |
|---|---|---|
| `crates/rustcode-cli/src/main.rs` | 3 | 换行合并（`:3363`） |
| `crates/rustcode-cli/src/schedule_cmd.rs` | 7 | 尾随空格 + assert 折行 |
| `crates/rustcode-coding/src/runtime.rs` | 3 | `if let` 折行（`:12466`） |
| `crates/rustcode-tuix/src/event_loop/commands.rs` | 16 | 删冗余尾随逗号 + 折行 |
| `crates/rustcode-tuix/src/event_loop/mod.rs` | 43 | 尾随逗号增删 + 折行 |
| `crates/rustcode-tuix/src/modals/dir_picker.rs` | 19 | 多行调用补尾随逗号 |
| `crates/rustcode-tuix/src/modals/onboarding_wizard.rs` | 4 | 折行 |
| `crates/rustcode-tuix/src/render/cell.rs` | 3 | 折行 |
| `crates/rustcode-tuix/src/test_term.rs` | 7 | `merge_derives` 合并 `#[derive]` + 删空行 |
| `crates/rustcode-updater/src/lib.rs` | 8 | `cfg!` 缩进 |

**纯格式已证明**：剥离全部空白字符后比对字符序列，6 个文件完全一致；
4 个文件差异经 Python 定位为尾随逗号增删与 rustfmt `merge_derives`，**零语义变化**。
（方法学提示：`git diff -w` 不可用于此证明，它只忽略行内空白，会把换行合并报成假阳性。）

### B. review 缺 locale 锁修复 —— 1 文件 +1 行

`crates/rustcode-review/src/review_tool.rs`：在
`review_activity_line_composes_label_findings_and_tail` 首行加 `let _g = pin_en();`。
**未改任何断言。** 验证 100/0，串行 + 并发共 4/4 全绿。

---

## 二、冲突点（请对方务必注意）

**`crates/rustcode-cli/src/main.rs` 是双方共同文件。**

- 我方：格式化了 `:3363`（`CodingRuntimeEvent::CompactionFinished` 的 `completion:` 折行）。
- 贵方 **T3** 要删除 `Commands::Codingplan` 别名，位于 `:1696` 与 `:3573`（截至本交接件写入时**仍在，T3 未执行**）。

**请勿对该文件执行 `git checkout` / `git restore` 回滚。**
贵方 `STATUS.md` 决策日志记载「`crates/rustcode-cli/src/main.rs` 均非 dirty，可安全
`git checkout` 回滚」——**该前提已被我方改动改变，此文件现在是 dirty**。
执行回滚会抹掉我方的格式改动（虽可重做，但会造成重复劳动与验证失效）。

建议：贵方 T3 在该文件上做**定点编辑**（只删 `Commands::Codingplan` 相关行），
不要整文件回滚、不要 `git checkout --`。

---

## 三、当前 worktree 快照（交接时实测）

- 改动文件总数：**29**
- 贵方已写入 6 个：`crates/rustcode-codingplan/src/{client,lib,setup,sync_marker,types}.rs`、
  `crates/rustcode-config/src/i18n/messages.rs`（合计 7 增 7 删，符合 T1 注释清零特征）
- 我方 11 个（上表 A+B）
- 上一轮遗留 11 个：`AGENTS.md`、`crates/rustcode-capabilities/src/{mcp/mod.rs,provider/openai_compat.rs}`、
  `crates/rustcode-clix/src/main.rs`、`crates/rustcode-kernel/src/{agent,event,hook,message}.rs`、
  `crates/rustcode-kernel/tests/{hook_a2_surface,turn_complete}.rs`、`docs/REFACTOR_DESIGN_PHASE1.md`
- 未跟踪：`docs/multi-agent-collaboration-solution.md`、`.codebuddy/`

**双方已写文件目前无交集。**

---

## 四、门禁状态（交接时实测）

| 门禁 | 状态 |
|---|---|
| G1 `cargo fmt --check` | **exit=0 / 0 处差异**（含贵方写入后复检） |
| G2 `cargo check -j 1 --workspace --all-targets` | exit=0 / 0 error |
| review --lib | **100/0**（修复后） |
| coding --lib / updater --lib | 430/0、41/0 |
| 全量 `cargo test --workspace` | **未复跑**（避免与贵方争锁/端口，见下） |

---

## 五、环境约束（双方共用，务必遵守）

- **cgroup 内存上限 8GB**：`cargo test` 默认并发会触发 rustc SIGBUS，**必须 `-j 1`**。
- **daemon 测试占用固定端口 13456–13458**：两个 cargo 测试进程并发会争用并产生**假红**。
  本轮已实测 `daemon_token_auth` 因此假红一次（事后单跑 3/3 全绿）。
  **请勿在我方跑全量测试时同时跑 cargo。**
- 工具会话超时约 60–90s；长任务用 `setsid nohup ... &` 脱离会话后轮询，
  且**同一日志路径绝不可被两个进程共用**（本轮因此污染过一次日志，出现重复的失败目标条目）。
- `rustcode-cli` 的**包名是 `rustcode`**，`-p rustcode-cli` 会失败。

---

## 六、我方恢复验证清单（贵方完成后执行）

1. `cargo fmt --check` → 须 exit=0（若贵方 T3 编辑引入新违规，请贵方 `cargo fmt -p rustcode`）
2. `cargo check -j 1 --workspace --all-targets` → 须 0 error
3. `cargo test -j 1 --workspace --no-fail-fast` → 与「已知红基线」逐名比对：
   - `rustcode-tuix --lib` ×5：`event_loop::task_render_tests::result_non_task_output_falls_back`
     + `event_loop::tool_format_tests::summarise_*` ×4
   - `rustcode --lib` ×1：`acp::translate::tests::policy_intervention_exposes_safe_recovery_without_secret_material`
   - `rustcode-capabilities --lib` ×1：`mcp::registry::tests::trust_key_golden_matches_core_algorithm`
     （文档化已知红，`AGENTS.md` 铁律禁改测试凑绿）
   - `rustcode-review --lib` 应为 **100/0**（本轮已修，不应再红）
4. 确认 `crates/rustcode-cli/src/main.rs` 同时保留：我方的 `:3363` 格式改动 + 贵方 T3 的删除结果
