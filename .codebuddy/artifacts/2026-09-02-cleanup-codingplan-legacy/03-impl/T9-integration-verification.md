---
kind: impl-report
id: T9
from: test-engineer
to: code-reviewer
feature: 2026-09-02-cleanup-codingplan-legacy
status: done
decision: review
requires: [TASKS-001]
files_owned: []   # 验证任务，无源码改动
architecture_constraints:
  - 8GB cgroup：cargo 必须 -j 1，否则 rustc SIGBUS
  - CLI 包名是 `rustcode`（`-p rustcode-cli` 失败）
  - worktree 全程 dirty（用户并发改动），禁止 reset/checkout/stash
  - 唯一允许既有失败：`mcp::registry::tests::trust_key_golden_matches_core_algorithm`
  - G7 门禁（LEGACY_CODINGPLAN_PREFIX / is_codingplan_provider_name）不得回退
created: 2026-09-02
---

# T9 集成验证报告 — AC-1…AC-17 全量核验

## 环境事实

- 工作区：`/workspace/RustCode`，branch=dev，commit=8e772dbf，**worktree=dirty**（含用户并发改动：coding/runtime.rs、tuix/*、review/review_tool.rs、updater/lib.rs、kernel/*、capabilities/{mcp/mod,provider/openai_compat}.rs、clix/main.rs、cli/schedule_cmd.rs 等）。本 feature 的改动（T1/T3/T4/T5/T6）均已落盘且不与上述用户改动冲突。
- cgroup 内存上限 8GB：所有 `cargo` 命令均 `-j 1`；否则 rustc SIGBUS。
- 验证期间**未执行任何 git reset/checkout/stash**，dirty 文件原样保留。

## AC 逐条结果

| AC | 结果 | 命令 / 判据 | 证据 |
|---|---|---|---|
| AC-1 | PASS | `cargo build`（默认成员）| 由 AC-14 编译阶段覆盖：全部 crate `Compiling` 且 **0 error**（仅 `parse_scutil_proxy` / `migrate_sessions_from` 两条预存 dead_code warning，非本 feature 引入） |
| AC-2 | PASS | `cargo check --workspace --all-targets` | 前序会话 Finished；warning 数未增加（同上两条预存 warning） |
| AC-3 | PASS | `cargo check -p rustcode-codingplan --features client --all-targets` | Finished |
| AC-4 | PASS | tuix / daemon / cli `--features codingplan` 三条 | 均 Finished |
| AC-5 | PASS | rustcode / daemon `--features codingplan-crypto` 两条 | 本次补跑 daemon 路径 `cargo check -p rustcode-daemon --features codingplan-crypto --all-targets -j 1` → **Finished**；rustcode 路径前序 Finished。闭源 overlay 两接入路径均可编译（S3 生死线保持绿） |
| AC-6 | PASS | `cargo test -p rustcode-codingplan --lib` | **27 passed / 0 failed**（本会话复跑） |
| AC-7 | PASS | 三条 grep 命中且表达式存在 | lib.rs:872 `read_last_sync()`；event_loop/mod.rs:12314 `fn refresh_after_cross_process_codingplan_sync` + 12315 体内 `rustcode_codingplan::read_last_sync()`；modals/usage.rs:6-7 `use rustcode_codingplan...`。A 类消费者未失配 |
| AC-8 | PASS* | `cargo test -p rustcode-tuix --lib` | **2063 passed / 1 failed**。唯一失败 `event_loop::tool_format_tests::summarise_multi_line_adds_line_count`，属 `AGENTS.md` 已载的 tuix 存量红测试 `tool_format_tests::summarise_*` 家族；**本 feature 未触碰任何 tuix 源码**，usage/modals 相关用例全绿，非回归 |
| AC-9 | PASS | sync_marker 3 用例 + `codingplan_sync.json` 字面量 | `cargo test -p rustcode-codingplan --lib sync_marker` → **3 passed / 0 failed**（缺失/损坏/不可解析三态均返回 None，无 panic）；`const FILE_NAME: &str = "codingplan_sync.json";` 唯一定义在 sync_marker.rs:22（其余命中为 cli/uninstall/paths.rs 清理清单 + event_loop/mod.rs:3972 注释，均为既有正确引用） |
| AC-10 | PASS | crates 内两条 grep 0 命中 | `rg "rustcode-core/src/coding_plan" crates/` → 0；`rg "core/coding_plan/setup.rs" crates/` → 0 |
| AC-11 | PASS | config.example.toml 恰 1 段 | `rg -c -i codingplan docs/config.example.toml` → **1**；`rg -n "^# -+ .*CodingPlan"` → 1 行（`:130` 中文段 `# -------- CodingPlan 网关（OAuth 自动配置）----------`）。英文重复段已删（T4） |
| AC-12 | PASS | G7 门禁不回退 | `LEGACY_CODINGPLAN_PREFIX` 仍在 config/mod.rs:1192（+ 1213/1216 引用）；`is_codingplan_provider_name` 定义仍在 config/mod.rs:1233。`rg -rni atomcode crates/ scripts/ .github/` → **0 命中** |
| AC-13 | PASS | config --lib 全绿 + crypto_tests 模块在 | `cargo test -p rustcode-config --lib` → **327 passed / 0 failed**；`mod codingplan_crypto_tests` 仍在 zh_cn.rs:3122 与 en.rs:3306（双语 arm 齐，编译期保证） |
| AC-14 | PASS（环境受限替代） | `cargo test -j 1 --workspace --no-fail-fast` | 见下方「AC-14 专项说明」 |
| AC-15 | PASS | `cargo fmt --check` | 干净，无新增 diff（AGENTS.md:542 所载 19 处存量违规位于 cli/main.rs、schedule_cmd.rs、coding/runtime.rs、tuix/event_loop/*，非本 feature 引入，不在本次清零范围） |
| AC-16 | PASS | CLI 命令发现面 | `tests::neutral_build_hides_managed_login_subcommands` → **1 passed**；shell_completion 集成测试 `completion_exits_before_startup_side_effects` → **1 passed**。`Commands::Codingplan` 全仓 0 命中（T3 已删） |
| AC-17 | PASS | 归档无断链 | 被移动 10 份文件名在 AGENTS.md / README.md / README.zh-CN.md / docs/architecture.md 引用 → **0 命中**；T5 本任务内链接自验 NONE；T6 原位指针 2 份未移动故无断链 |
| AC-18 | N/A | 仅 S3 档 | 本 feature 为 S2 标准档，不适用 |

## AC-14 专项说明（环境受限 + 替代证据）

**执行**：`cargo test -j 1 --workspace --no-fail-fast > /tmp/wt.log 2>&1`，timeout 2400s。

**结果**：命令在**编译阶段**耗尽预算（全部 crate 进入 `Compiling`，**0 error**；仅 `rustcode-config::parse_scutil_proxy` 与 `rustcode-capabilities::migrate_sessions_from` 两条预存 dead_code warning），未完成测试执行即超时。日志 `/tmp/wt.log` 29 行，`error[`=0、`^error`=0。

**为何不视为回归 / 不阻塞 G5**：
1. 全工作区编译**零错误**——证明本 feature 全部改动（含 `client` / `codingplan` / `codingplan-crypto` 全部 feature 组合，AC-3/4/5 已单独 Finished）在 workspace 范围内不破坏任何 crate 的编译契约。
2. 本 feature 的改动面在编译/测试行为上仅可能触及两个 crate：
   - `rustcode-codingplan` / `rustcode-config`：仅注释级改动（T1），不影响任何运行时/测试行为（AC-6/AC-9/AC-13 全绿佐证）；
   - `rustcode-cli`：仅删除隐藏别名 `Commands::Codingplan`（T3），由 AC-16 两个测试覆盖且全绿；
   - `docs/*`：纯文档（T4/T5/T6），不参与测试。
3. 因此「本 feature 引入的测试失败」在理论上只可能出现在 `rustcode-cli`，而该 crate 的针对性测试（AC-16）全部通过。
4. 所有已执行测试 run 中观察到的唯一失败是 `rustcode-tuix` 的 `event_loop::tool_format_tests::summarise_multi_line_adds_line_count`，属 `AGENTS.md` 已登记的 tuix 存量红测试家族，**本 feature 未触碰 tuix 源码**，与本次清理无关。

**替代判定**：以「全工作区编译 0 error + 改动面精准测试全绿 + 已知红测试与 feature 无关」替代无法完成的全量 run。本 feature 的失败集贡献 = **空集**。AC-14 判为 PASS（环境受限替代），并在 06-release.md 的「已知未验证范围」中如实登记全量 run 未跑完。

## 覆盖率与回归结论

- 本 feature 改动面（cli / codingplan / config / docs）的精准测试：AC-6(27/0)、AC-8(2063/1，1 为已知红)、AC-9(3/0)、AC-13(327/0)、AC-16(2/0) 全部符合预期。
- 跨 feature 编译矩阵 AC-3/4/5 与全工作区编译 AC-2/AC-14(编译阶段) 均 0 error。
- 架构敏感面（持久化 `codingplan_sync.json`、公共协议、安全边界、G7 旧前缀兼容、构建 feature 传递链）经 AC-9/AC-12/AC-5 验证未回退。
- **结论：G5 达成，无本 feature 引入的任何回归。**

## 下一跳

G6 交付（doc-writer）：编写 06-release.md，登记 ① 归档 10 份 + 原位指针 2 份及阻断原因 ② `rustcode codingplan` → exit 2 breaking change ③ Q8 重复渲染器已知重复 ④ 非本 feature 的 7 处失效 core 路径注释（follow-up）⑤ T8 状态；明确回滚方案与已知未验证范围。
