---
kind: implementation
id: T-10
from: code-implementer
to: [code-reviewer]
feature: 2026-09-09-codingplan-removal-eval
status: done
decision: proceed
requires: [REQ-001, DESIGN-001]
files_owned:
  - .codebuddy/artifacts/2026-09-09-codingplan-removal-eval/03-impl/baseline.md
  - .codebuddy/artifacts/2026-09-09-codingplan-removal-eval/03-impl/T-10.md
architecture_constraints:
  touches_runtime_lifecycle: false
  touches_persistence: false
  touches_cross_crate_deps: false
created: 2026-09-09
---

# T-10 基线留档（AC-0）—— codingplan 移除大改动的比对原点

> **只读任务。**
> 本任务**未改动任何生产文件**：`crates/**`（.rs / Cargo.toml）、`extensions/**`、
> `webui/src/**`、`docs/**`、`AGENTS.md`、根 `Cargo.toml`、`Cargo.lock` 一律未动；
> 未执行 `git add` / `git commit` / `git checkout` / `git reset` / `git clean`。
> 唯一写入物是本目录下的两个报告文件：`baseline.md`（本文件）与 `T-10.md`。
> 两者**内容完全一致**（`baseline.md` 为任务书 `files_owned` 指定名，`T-10.md` 为
> 实现报告命名约定路径），避免下游批次引用时找不到文件。

## 0. 结论摘要（TL;DR）

| 项 | 结果 |
|---|---|
| 基线 commit | `e80fb7ae4299b471f26aa658b55197be5a1ee5ac`（branch `dev`） |
| worktree | 仅一个未跟踪项 `?? .codebuddy/artifacts/2026-09-09-codingplan-removal-eval/`（本 feature 自身产物） |
| `cargo build` | exit **0**（2 条既有 dead_code 警告，非本 feature 引入） |
| `cargo build --workspace` | exit **0**（同上） |
| `cargo fmt --check` | exit **0**，**零输出** |
| `cargo test -j 1 --workspace --no-fail-fast` | exit **101**，**已完整跑完**（80 个 test target + 12 个 doc-test target），总 5488 passed / **1 failed** / 11 ignored |
| 失败集 | **仅** `mcp::registry::tests::trust_key_golden_matches_core_algorithm` |
| 是否匹配 AC-0 预期 | **完全匹配**（预期即为该唯一用例） |
| `N_RS`（`crates/`，`.rs`+`.toml` 行命中） | **1100** 行 / **66** 个文件 |
| `N_EXT`（`extensions/` + `webui/src`） | **65** 行（extensions 52 / webui/src 13；21 个文件） |
| `N_DOC`（`docs/`） | **117** 行 |
| `Cargo.lock` 中 `rustcode-codingplan` | 行号 **2692**（+ `"rustcode-codingplan-crypto"` 2724、`name = "rustcode-codingplan"` 2846、`name = "rustcode-codingplan-crypto"` 2859、2897、2968） |

**判定：AC-0 通过，可放行批次 2（T-11 / T-12）。**

---

## 1. 改动清单

本任务**零生产代码改动**。仅新增：

| 路径 | 变更说明 |
|---|---|
| `.codebuddy/artifacts/2026-09-09-codingplan-removal-eval/03-impl/baseline.md:1` | 新建：AC-0 全量基线留档（本报告正文） |
| `.codebuddy/artifacts/2026-09-09-codingplan-removal-eval/03-impl/T-10.md:1` | 新建：与 `baseline.md` 同内容的实现报告（命名约定路径） |

未触碰：`crates/**`、`extensions/**`、`webui/src/**`、`docs/**`、`AGENTS.md`、
`Cargo.toml`（根与各 crate）、`Cargo.lock`。

---

## 2. 契约符合性（对照 `00-requirement.md` AC-0 / `02-tasks.md` T-10）

| AC-0 要求 | 实现情况 | 偏差 |
|---|---|---|
| 7 条命令全部执行成功并留档 | 见 §3 C1–C9，全部执行完毕；原始输出与 exit code 均已贴出 | 无 |
| 失败集应只含 `mcp::registry::tests::trust_key_golden_matches_core_algorithm` | 实际失败集**恰好**为该 1 条（§4） | 无 |
| 其余红必须先记录、不得归因于本 feature | 无其余红；唯一红为 golden 哈希断言失败，**明确记录为既存失败，不归因于本 feature**（本任务零代码改动，物理上不可能引入） | 无 |
| 记录 branch / commit SHA / worktree 状态 | C1–C3 | 无 |
| 记录 `N_RS` / 文件数 / `N_EXT` / `N_DOC` / `Cargo.lock` 行号 | C6–C9 + §5 | 无 |
| 不改动生产文件 | 已遵守（§1） | 无 |

**唯一需要声明的「非偏差说明」**：任务书给的 `cargo test ... | tail -40` 只是展示方式，
本任务为得到**完整失败集**而保留了**全量日志**（6150 行，落盘于 `/tmp/T10-b4-cargo-test-workspace.log`
与 `/tmp/T10-b4b-cargo-test-workspace-exitcode.log`），§3-C5 同时给出 `tail -40` 原文与关键切片。
这不是命令替换，而是同一条命令的完整输出留档。

---

## 3. 命令原始输出与 exit code

### C1 `git rev-parse --abbrev-ref HEAD` —— exit code **0**

```
$ git rev-parse --abbrev-ref HEAD
dev
```

### C2 `git rev-parse HEAD` —— exit code **0**

```
$ git rev-parse HEAD
e80fb7ae4299b471f26aa658b55197be5a1ee5ac
```

### C3 `git status --short` —— exit code **0**

```
$ git status --short
?? .codebuddy/artifacts/2026-09-09-codingplan-removal-eval/
```

说明（**只记录，不处理、不删除**）：

- 除本 feature 自身的产物目录外，**没有其它未跟踪项**。
- 任务书提示的 `.codebuddy/artifacts/2026-09-09-omo-skills-import/` 确实存在于工作区，
  但它**已被 git 跟踪**（`git ls-files` 能列出 `01-import-plan.md`、`03-impl/T-01.md`、`03-impl/T-04.md`），
  因此不出现在 `??` 中。这是**其它 feature 的既有产物**，与本 feature 无关，未做任何改动。
- 无 `M` / `A` / `D` 条目 ⇒ 基线点无任何他人改动被我修改。

### C4 `cargo build 2>&1 | tail -5` —— exit code **0**

```
$ cargo build 2>&1 | tail -5
     = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `rustcode-capabilities` (lib) generated 1 warning
   Compiling rustcode v5.0.9 (/workspace/RustCode/crates/rustcode-cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 15.25s
```

完整日志（20 行，`/tmp/T10-b1-cargo-build.log`）中的 2 条警告均为**基线既有**、与 codingplan 无关：

```
warning: function `parse_scutil_proxy` is never used
  --> crates/rustcode-config/src/system_proxy.rs:80:15
warning: `rustcode-config` (lib) generated 1 warning
warning: function `migrate_sessions_from` is never used
    --> crates/rustcode-capabilities/src/session/manager.rs:3318:4
warning: `rustcode-capabilities` (lib) generated 1 warning
```

### C5 `cargo build --workspace 2>&1 | tail -5` —— exit code **0**

```
$ cargo build --workspace 2>&1 | tail -5
     = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `rustcode-capabilities` (lib) generated 1 warning
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.93s
```

（2.93s 是因为 C4 已把默认成员编完，`--workspace` 的其余成员此前已缓存；警告同 C4，共 2 条。）

### C6 `cargo fmt --check` —— exit code **0**，**stdout/stderr 全空（0 行）**

```
$ cargo fmt --check
$ echo $?
0
```

### C7 `cargo test -j 1 --workspace --no-fail-fast 2>&1 | tail -40` —— exit code **101**

> 命令**完整跑完，未截断、未改用子集**。共执行 80 个 test target + 12 个 doc-test target。
> 为捕获 exit code，同一命令**执行了两次**：
> - 第 1 次（`/tmp/T10-b4-cargo-test-workspace.log`，6151 行）：完整跑完；
> - 第 2 次（`/tmp/T10-b4b-cargo-test-workspace-exitcode.log`，6150 行 + 末行 `__EXIT_CODE__=101`）：完整跑完并留档 exit code。
>
> 两次结果**一致**：将 `test result:` 行去掉耗时后排序 diff ⇒ 完全相同（`IDENTICAL_RESULT_SET`）。
> 差异仅存在于**单个 test binary 内部用例的打印顺序**与耗时（`-j 1` 约束的是 target 并行度，
> 不约束 binary 内多线程用例的完成顺序），不影响任何 pass/fail 判定。

`tail -40` 原始输出（第 2 次，含我追加的 exit 标记行）：

```
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests rustcode_daemon

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests rustcode_kernel

running 3 tests
test crates/rustcode-kernel/src/conformance/mod.rs - conformance (line 19) ... ignored
test crates/rustcode-kernel/src/lib.rs - test_support (line 48) ... ignored
test crates/rustcode-kernel/src/lib.rs - test_support (line 54) ... ignored

test result: ok. 0 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests rustcode_review

running 1 test
test crates/rustcode-review/src/lib.rs - (line 17) - compile ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

   Doc-tests rustcode_tuix

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests rustcode_updater

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: 1 target failed:
    `-p rustcode-capabilities --lib`
__EXIT_CODE__=101
```

关键切片 —— 唯一失败的 target 与其 `failures:` 段（第 1 次日志 L1901–L1916）：

```
failures:

---- mcp::registry::tests::trust_key_golden_matches_core_algorithm stdout ----

thread 'mcp::registry::tests::trust_key_golden_matches_core_algorithm' (218036) panicked at crates/rustcode-capabilities/src/mcp/registry.rs:1507:9:
assertion `left == right` failed
  left: "e07a86b0ce8a1c59"
 right: "8b6a67e0b2c06dae"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    mcp::registry::tests::trust_key_golden_matches_core_algorithm

test result: FAILED. 1475 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.20s

error: test failed, to rerun pass `-p rustcode-capabilities --lib`
```

全量汇总（第 1 次日志统计）：

```
总 passed  = 5488
总 failed  = 1
总 ignored = 11
test target 数 = 80
Doc-tests target 数 = 12
```

### C8a `grep -rniI "codingplan" crates/ --include=*.rs --include=*.toml | wc -l` —— exit code **0**

```
$ grep -rniI "codingplan" crates/ --include=*.rs --include=*.toml | wc -l
1100
```

配套文件数（同一 grep 加 `-l`）：

```
$ grep -rlniI "codingplan" crates/ --include=*.rs --include=*.toml | wc -l
66
```

### C8b `grep -rniI "codingplan" extensions/ webui/src | wc -l` —— exit code **0**

```
$ grep -rniI "codingplan" extensions/ webui/src | wc -l
65
```

### C8c `grep -rniI "codingplan" docs/ | wc -l` —— exit code **0**

```
$ grep -rniI "codingplan" docs/ | wc -l
117
```

### C9 `grep -n "rustcode-codingplan" Cargo.lock` —— exit code **0**

```
$ grep -n "rustcode-codingplan" Cargo.lock
2692: "rustcode-codingplan",
2724: "rustcode-codingplan-crypto",
2846:name = "rustcode-codingplan"
2859:name = "rustcode-codingplan-crypto"
2897: "rustcode-codingplan",
2968: "rustcode-codingplan",
```

（任务书点名的 `grep -n "rustcode-codingplan" Cargo.lock` 命中 6 行；其中
`name = "rustcode-codingplan"` 的主条目在 **L2846**。6 行中除 2724/2859 是
`-crypto` 子串被同一模式带出外，其余 4 行均为 `rustcode-codingplan` 本体的依赖引用。）

---

## 4. 失败集全名清单 vs AC-0 预期

| # | 失败用例全名 | 所在 target | 失败性质 |
|---|---|---|---|
| 1 | `mcp::registry::tests::trust_key_golden_matches_core_algorithm` | `-p rustcode-capabilities --lib` | 断言失败：`left: "e07a86b0ce8a1c59"` vs `right: "8b6a67e0b2c06dae"`，位置 `crates/rustcode-capabilities/src/mcp/registry.rs:1507:9` |

- **实际失败集 = { `mcp::registry::tests::trust_key_golden_matches_core_algorithm` }**
- **AC-0 预期 = { `mcp::registry::tests::trust_key_golden_matches_core_algorithm` }**
- **结论：逐条比对，完全一致，无多余红、无缺失红。**

判断（**不归因、不改断言**）：

- 本任务零代码改动，物理上不可能引入任何失败；该红是**基线既存失败**。
- 它是 MCP server trust key 的 golden 哈希断言（期望 `8b6a67e0b2c06dae`，实测 `e07a86b0ce8a1c59`），
  涉及 `rustcode-capabilities`，与 codingplan 无符号引用关系。
- 后续批次只需保证**该红不增不减**：新增任何红都必须先记录、再判断是否与本批次相关。

---

## 5. 四个计数的具体数值

| 计数 | 命令 | 数值 |
|---|---|---|
| `N_RS` | `grep -rniI "codingplan" crates/ --include=*.rs --include=*.toml \| wc -l` | **1100**（行命中） |
| `N_RS` 文件数 | 同上加 `-l` | **66**（文件命中） |
| `N_EXT` | `grep -rniI "codingplan" extensions/ webui/src \| wc -l` | **65**（行命中；extensions 52 + webui/src 13，共 21 个文件） |
| `N_DOC` | `grep -rniI "codingplan" docs/ \| wc -l` | **117**（行命中） |
| `Cargo.lock` | `grep -n "rustcode-codingplan" Cargo.lock` | 命中 **6** 行：**2692**, 2724, **2846**, 2859, **2897**, **2968** |

`N_RS` 按 crate 分布（供下游批次各自核对自己的下降量）：

| crate | 命中文件数 | 命中行数 |
|---|---|---|
| `crates/rustcode-tuix/` | 16 | 339 |
| `crates/rustcode-codingplan/` | 7 | 266 |
| `crates/rustcode-config/` | 7 | 222 |
| `crates/rustcode-daemon/` | 10 | 143 |
| `crates/rustcode-coding/` | 8 | 45 |
| `crates/rustcode-cli/` | 4 | 40 |
| `crates/rustcode-auth/` | 3 | 21 |
| `crates/rustcode-capabilities/` | 4 | 13 |
| `crates/rustcode-kernel/` | 4 | 7 |
| `crates/rustcode-clix/` | 1 | 2 |
| `crates/rustcode-codingplan-crypto/` | 2 | 2 |
| **合计** | **66** | **1100** |

（逐 crate 求和与全量 grep 结果自洽，可作为交叉校验。）

---

## 6. target 级测试基线表（下游批次比对用）

全部 80 个 test target + 12 个 doc-test target，逐个的 `test result:` 行（第 1 次日志）：

| target | 结果 |
|---|---|
| `rustcode` lib (`--lib`) | ok. **116** passed; 0 failed |
| `rustcode` bin (`src/main.rs`) | ok. **91** passed; 0 failed |
| `acp_end_to_end` | ok. 13 passed |
| `acp_initialize` | ok. 1 passed |
| `askpass_helper` | ok. 1 passed |
| `schedule_run_exit_code` | ok. 1 passed |
| `script_parity` | ok. 2 passed |
| `setup_cli` | ok. 1 passed |
| `shell_completion` | ok. 1 passed |
| `uninstall_integration` | ok. 3 passed |
| `rustcode_auth` lib | ok. 42 passed |
| **`rustcode_capabilities` lib** | **FAILED. 1475 passed; 1 failed**（唯一红） |
| `mcp_test_server` bin | ok. 0 passed |
| `anthropic_mock` | ok. 5 passed |
| `compaction_cache` | ok. 2 passed |
| `e2e` | ok. 0 passed |
| `http_mock` | ok. 12 passed |
| `mcp` | ok. 9 passed |
| `memory` | ok. 2 passed |
| `ollama_mock` | ok. 2 passed |
| `session` | ok. 4 passed |
| `session_fixture_invariants` | ok. 8 passed |
| `setup_integration` | ok. 3 passed |
| `tools_integration` | ok. 5 passed |
| `rustcodex` bin | ok. 44 passed |
| `rustcode_coding` lib | ok. 430 passed; 8 ignored |
| `assemble_smoke` | ok. 4 passed |
| `cache_prefix` (coding) | ok. 2 passed |
| `full_assembly` | ok. 1 passed |
| `overflow_recovery` | ok. 1 passed |
| `permission_grants` | ok. 1 passed |
| `plan_mode` | ok. 1 passed |
| `sensitive_path` | ok. 1 passed |
| `system_context` | ok. 1 passed |
| `team_runtime` | ok. 4 passed |
| `tool_args_repair` | ok. 3 passed |
| `verify_cadence` | ok. 4 passed |
| `rustcode_codingplan` lib | ok. **27** passed（批次 6 删除后该 target 应消失） |
| `rustcode_codingplan_crypto` lib | ok. **0** passed（批次 6 桩化后应仍为 0） |
| `rustcode_config` lib | ok. **327** passed |
| `cli_webui_i18n_lock` | ok. 3 passed |
| `config_store` | ok. 7 passed |
| `unified_prompt` | ok. 3 passed |
| **`rustcode_daemon` lib** | ok. **307** passed; 0 failed（与 T-11 AC「基线 307/0」一致 ✓） |
| `rustcode_daemon` bin | ok. 0 passed |
| `daemon_token_auth` | ok. 1 passed |
| `default_host_lock` | ok. 1 passed |
| `legacy_turn_boundary_repair` | ok. 7 passed |
| `rustcode_kernel` lib | ok. 129 passed |
| `cache_prefix` (kernel) | ok. 3 passed |
| `cancellation` | ok. 8 passed |
| `chat_options` | ok. 3 passed |
| `compaction` | ok. 15 passed |
| `conformance` | ok. 12 passed |
| `dedup` | ok. 5 passed |
| `determinism` | ok. 1 passed |
| `failure_perception` | ok. 16 passed |
| `fallible_stream` | ok. 13 passed |
| `hook_a2_surface` | ok. 7 passed |
| `hook_composition` | ok. 4 passed |
| `liveness` | ok. 9 passed |
| `multimodal` | ok. 2 passed |
| `overflow_compaction` | ok. 2 passed |
| `pre_request_guard` | ok. 2 passed |
| `rate_limit` | ok. 7 passed |
| `reasoning_persist` | ok. 4 passed |
| `resume` | ok. 7 passed |
| `spike_claims` | ok. 17 passed |
| `subagent` | ok. 3 passed |
| `tool_batch` | ok. 8 passed |
| `tool_call_streaming` | ok. 1 passed |
| `tool_loop_guard` | ok. 14 passed |
| `tool_progress` | ok. 1 passed |
| `tool_result_cap` | ok. 4 passed |
| `turn_complete` | ok. 3 passed |
| `usage_merge` | ok. 1 passed |
| `rustcode_review` lib | ok. 100 passed |
| **`rustcode_tuix` lib** | ok. **2064** passed（批次 3/4 的主要回归面） |
| `plugin_integration` | ok. 1 passed |
| `rustcode_updater` lib | ok. 41 passed |
| Doc-tests × 12 | 均 ok（kernel 3 ignored，coding 1 passed，review 1 passed，其余 0） |

下游最常被引用的三个基线数字：**daemon lib 307/0**、**tuix lib 2064/0**、**codingplan lib 27/0**。

---

## 7. 执行环境

```
cargo 1.93.0 (083ac5135 2025-12-15)
rustc 1.93.0 (254b59607 2026-01-19)
host 248ba91ddd66
date 2026-09-09T04:25:53Z (UTC)
workspace /workspace/RustCode, branch dev, HEAD e80fb7ae4299b471f26aa658b55197be5a1ee5ac
```

资源：磁盘 181G 可用（overlay 256G，已用 75G）；内存 39.7G 总量 / 32.2G 可用。
`target/` 41G（其中 `target/debug/incremental` 20G）。**未清理 incremental**（磁盘充足，无需清理）。
未使用 `sudo`（遵守 `AGENTS.md:19`）；`cargo test` 一律带 `-j 1`。

---

## 8. 遗留风险与后续项

1. **既存红 `trust_key_golden_matches_core_algorithm` 未修复**（本任务职责外）。
   建议负责人：MCP / capabilities 的 owner 独立排查（golden 值与当前算法不一致，
   可能是历史上算法变更未同步 golden）。后续 10 个批次**只比对、不修复**，
   任何新增红都必须在当批报告中显式列出。
2. **`target/` 已达 41G**。后续 10 个批次会反复重编译，建议在磁盘告警时（而非现在）
   清理 `target/debug/incremental`。当前无需处理。
3. **两次运行日志已落盘于 `/tmp`**（`T10-b1`…`T10-b4b`），`/tmp` 非持久存储；
   下游若需复核，可在本 commit `e80fb7a` 上重跑本报告 §3 的命令复现。
4. **`Cargo.lock` 行号会在后续批次变动**：T-17 / T-21 摘除 crate 后 `Cargo.lock` 会被重写，
   §5 的 6 个行号只对基线 commit 有效，不得跨批次当作绝对坐标使用。
5. **并发注意**：本任务只读，但下游批次若并行执行，均依赖本文件；**不得修改本文件**，
   只能追加引用。

## 9. 自验证证据索引

| 命令 | 日志文件 | exit code |
|---|---|---|
| `git rev-parse --abbrev-ref HEAD` / `git rev-parse HEAD` / `git status --short` | （直接回显，见 §3 C1–C3） | 0 / 0 / 0 |
| `cargo build` | `/tmp/T10-b1-cargo-build.log` | 0 |
| `cargo build --workspace` | `/tmp/T10-b2-cargo-build-workspace.log` | 0 |
| `cargo fmt --check` | `/tmp/T10-b3-cargo-fmt-check.log`（0 行） | 0 |
| `cargo test -j 1 --workspace --no-fail-fast`（第 1 次） | `/tmp/T10-b4-cargo-test-workspace.log`（6151 行） | 未捕获（见下） |
| `cargo test -j 1 --workspace --no-fail-fast`（第 2 次） | `/tmp/T10-b4b-cargo-test-workspace-exitcode.log`（6150 行 + `__EXIT_CODE__=101`） | **101** |

**关于第 1 次未捕获 exit code 的如实说明**：第 1 次以 `nohup ... &` 方式启动（为规避工具调用超时），
`$?` 未被保存。为获得权威 exit code，**完整重跑了同一条命令**（第 2 次，全程未截断），
并在日志末行写入 `__EXIT_CODE__=101`。两次运行的 `test result:` 集合（去耗时后排序 diff）完全相同，
故 §3-C7 的 exit code **101** 与全量失败集均来自**完整执行**，非推断。
