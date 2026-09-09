---
kind: implementation
id: B2-atomgit
from: code-implementer
to: [code-reviewer]
feature: 2026-09-09-local-identity
status: done
decision: proceed
requires: [DESIGN-001, B1-ua-migration]
files_owned:
  - crates/rustcode-capabilities/src/atomgit/**
  - crates/rustcode-capabilities/src/lib.rs
  - crates/rustcode-capabilities/src/tools/mod.rs
  - crates/rustcode-capabilities/src/tools/bash_workspace_gate.rs
  - crates/rustcode-capabilities/Cargo.toml
  - crates/rustcode-coding/Cargo.toml
  - Cargo.lock
files_owned_extension_declared:
  - crates/rustcode-capabilities/src/tools/atomgit.rs
  - crates/rustcode-capabilities/src/tools/atomgit_bash_gate.rs
  - crates/rustcode-capabilities/src/tools/task.rs
  - crates/rustcode-capabilities/src/tools/bash.rs
  - crates/rustcode-capabilities/src/egress/mod.rs
  - crates/rustcode-coding/src/assemble.rs
  - crates/rustcode-coding/src/parts.rs
  - crates/rustcode-coding/src/persona.rs
  - crates/rustcode-coding/tests/assemble_smoke.rs
architecture_constraints:
  touches_runtime_lifecycle: false
  touches_persistence: false
  touches_cross_crate_deps: true
created: 2026-09-09
---

# B2 — 移除 `atomgit` feature 及其全部代码

## 0. 结论摘要

| 项 | 结果 |
| --- | --- |
| `grep -rn "atomgit" capabilities/src coding/src --include="*.rs"` | **0** |
| 同目录大小写不敏感残留（`AtomGit`/`Atomgit`/`ATOMGIT`） | **0** |
| `cargo check -p rustcode-capabilities --features atomgit` | `error: the package 'rustcode-capabilities' does not contain this feature: atomgit` ✅ |
| 是否留红 | **否**（`cargo check --workspace --all-targets` 0 error；capabilities 825 passed / 0 failed；coding 全绿） |
| 删除代码量 | 10 个文件、3176 行整文件删除 + 8 个文件内 cfg 分支删除，共 **-3481 行**（含他人并发改动 +50） |

## 1. 改动清单

### 1.1 整文件删除（10 个，3176 行）

| 路径 | 行数 | 说明 |
| --- | --- | --- |
| `crates/rustcode-capabilities/src/atomgit/client.rs` | 295 | AtomGit REST 客户端（HTTP 调用、`Retry` 分类） |
| `crates/rustcode-capabilities/src/atomgit/mod.rs` | 93 | 模块根 + `LiveTokenProvider`（`rustcode_auth::oauth::get_valid_token()` 调用点，`mod.rs:59`） |
| `crates/rustcode-capabilities/src/atomgit/models.rs` | 222 | Repo / PullRequest / Issue / Comment 等 DTO |
| `crates/rustcode-capabilities/src/atomgit/repo.rs` | 398 | `atomgit_repo` 实现 |
| `crates/rustcode-capabilities/src/atomgit/pr.rs` | 287 | `atomgit_pr` 实现 |
| `crates/rustcode-capabilities/src/atomgit/issue.rs` | 173 | `atomgit_issue` 实现 |
| `crates/rustcode-capabilities/src/atomgit/push_label_mw.rs` | 214 | `GitPushLabelMiddleware`（`rustcode_auth::oauth::get_valid_token()` 第二调用点，`push_label_mw.rs:57`） |
| `crates/rustcode-capabilities/src/atomgit/remote.rs` | 178 | push 后 project-label 远端探测 |
| `crates/rustcode-capabilities/src/tools/atomgit.rs` | 1190 | `atomgit_repo/pr/issue` 三个 `Tool` 适配 + `register_atomgit_tools` + `atomgit_tool_names` |
| `crates/rustcode-capabilities/src/tools/atomgit_bash_gate.rs` | 126 | `AtomgitBashGate`（拒绝经 bash 裸调 `api.atomgit.com`） |

### 1.2 `crates/rustcode-capabilities/src/lib.rs`

| 行（原） | 变更 |
| --- | --- |
| `:90` | `egress` 模块文档：`(provider, web, atomgit, mcp)` → `(provider, web, mcp)` |
| `:99-104` | `proxy` 的 `#[cfg(any(...))]` 去掉 `feature = "atomgit"`，收敛为 `#[cfg(any(feature = "provider", feature = "web", feature = "mcp"))]` |
| `:170-172` | 删除 `/// AtomGit REST tools…` 文档 + `#[cfg(feature = "atomgit")] pub mod atomgit;` |

### 1.3 `crates/rustcode-capabilities/src/tools/mod.rs`

| 行（原） | 变更 |
| --- | --- |
| `:38-42` | 删除 `pub mod atomgit;` 与 `pub mod atomgit_bash_gate;`（各带 `#[cfg(feature = "atomgit")]`） |
| `:76-81` | 删除 `pub use crate::atomgit::push_label_mw::GitPushLabelMiddleware;` 与 `crate::atomgit::{AtomgitClient, AtomgitConfig, LiveTokenProvider, StaticTokenProvider, TokenProvider}` 再导出 |
| `:87-92` | 删除 `atomgit::{atomgit_tool_names, register_atomgit_tools, Atomgit*Tool}` 与 `atomgit_bash_gate::AtomgitBashGate` 再导出 |

### 1.4 `crates/rustcode-capabilities/src/tools/bash_workspace_gate.rs`

| 行（原） | 变更 |
| --- | --- |
| `:503-533` | 删除 `command_invokes_git_subcommand()`（含其 12 行文档，原文末句「Gated on `atomgit`」）与其 `#[cfg(feature = "atomgit")]` |
| `:535-562` | 删除其私有辅助 `git_subcommand()`（仅被上面函数使用） |
| `:876-915` | 删除测试段「git-subcommand detection」：`runs_push()`、`git_subcommand_detects_push_across_shapes`、`git_subcommand_rejects_non_push` |

> 保留 `split_segments` / `tokenize` / `effective_command_index` / `command_word` —— 它们仍被破坏性-fs 扫描器使用（编译无 dead_code 警告佐证）。

### 1.5 `crates/rustcode-capabilities/src/tools/task.rs`（声明外扩，见 §3）

| 行（原） | 变更 |
| --- | --- |
| `:388-391` | 文档「…the feature-enabled AtomGit bash guard…」→ 去掉该从句 |
| `:443-444` | 删除 `#[cfg(feature = "atomgit")] mw.push(Arc::new(super::AtomgitBashGate::new()));` |
| `:3148-3151` | 测试 `child_middlewares_add_the_scope_gate_only_for_workers` 的 `base` 由双 cfg 分支收敛为 `let base = 2; // DenySensitivePaths + CredentialBashGate.` |
| `:3178-3181` | 测试 `team_middlewares_scope_explore_only_when_scope_is_declared` 同上收敛为 `base = 2` |

### 1.6 `crates/rustcode-capabilities/src/egress/mod.rs`（声明外扩）

| 行（原） | 变更 |
| --- | --- |
| `:18-19` | 模块文档列举 `atomgit` → 删除 |
| `:31` | 文档「performs outbound HTTP (`provider`, `web`, `atomgit`, `mcp`)」→ 去掉 `atomgit` |

### 1.7 `crates/rustcode-capabilities/src/tools/bash.rs`（声明外扩，纯注释）

| 行（原） | 变更 |
| --- | --- |
| `:73` | 「(the AtomGit `[PASSED]` box)」→「(a remote's `[PASSED]` push banner)」 |
| `:2628` | 「like the [PASSED] box from AtomGit push hooks」→「like a remote's [PASSED] push banner」 |

> 两处均为描述**仍存在**的 `/dev/tty` 脱离行为的注释，仅把已删除平台名泛化；`cargo fmt`/编译不受影响。

### 1.8 `crates/rustcode-capabilities/Cargo.toml`

| 行（原） | 变更 |
| --- | --- |
| `:150` | `provider` feature 去掉 `dep:rustcode-auth`（见 §2.3 确认：provider 侧已无 `rustcode_auth::` 使用点） |
| `:154` | 注释「(provider / web_fetch / web_search / atomgit / mcp)」→ 去掉 `atomgit` |
| `:165-168` | 删除 `atomgit = ["tools", "egress", "dep:reqwest", "dep:rustcode-auth"]` 及其 3 行注释 |
| `:199-200` | `cc-hooks` 注释中「(e.g. via `atomgit`)」→ 去掉 |

**保留**：`rustcode-auth = { path = "../rustcode-auth", optional = true }`（`:22`）与 `plugin` feature 的 `dep:rustcode-auth`（`:236`）—— `plugin/marketplace.rs:303/304/344/358/361/767` 仍在用，归批次 3（marketplace 无认证化）。

### 1.9 `crates/rustcode-coding/Cargo.toml`

| 行（原） | 变更 |
| --- | --- |
| `:7-11` | 删除整个 `[features]` 段（`atomgit = ["rustcode-capabilities/atomgit"]` + 2 行注释 + 空行）；该 crate 无其它 feature |

### 1.10 `crates/rustcode-coding/src/assemble.rs`（声明外扩）

| 行（原） | 变更 |
| --- | --- |
| `:130-133` | 删除 `#[cfg(feature = "atomgit")] let builder = builder.middleware(AtomgitBashGate::new());` |
| `:202-207` | 删除 `#[cfg(feature = "atomgit")] { … GitPushLabelMiddleware … }` |
| `:219-224` | `mount_coding_tools` 中删除 atomgit 注册 shadow 块 |
| `:255-277` | 删除 `register_atomgit_capabilities()` 函数及其文档 |

**刻意保留**：`mount_coding_tools` 的 `Result<MountedTools, String>` 签名未改。移除 atomgit 后该 `Err` 分支已不可达（唯一 `?` 来源消失），但收缩签名会连带改 `build_coding_agent_with` 的 infallible 契约与 `try_build_coding_agent_with`，超出本任务范围 —— 登记为 follow-up（§5）。

### 1.11 `crates/rustcode-coding/src/parts.rs`（声明外扩）

| 行（原） | 变更 |
| --- | --- |
| `:559-563` | 删除 `#[cfg(feature = "atomgit")] if opts.tools { register_atomgit_capabilities(...) }` 及 `AtomGit tool setup failed` 错误映射 |
| `:1703-1706` | `CredentialBashGate` 注释「independent of whether the optional AtomGit integration is compiled in」→ 改为无条件边界表述 |
| `:1719-1726` | 删除 `#[cfg(feature = "atomgit")] { … AtomgitBashGate … }` 中间件挂载 |
| `:1915-1926` | 删除「Ensure the repo's project label after a successful `git push`」整块（6 行注释 + `GitPushLabelMiddleware` 挂载） |
| `:2798-2812` | 删除测试 `production_prepare_exposes_atomgit_tools` |

### 1.12 `crates/rustcode-coding/src/persona.rs`（声明外扩）

| 行（原） | 变更 |
| --- | --- |
| `:296-297` | 删除 `#[cfg(feature = "atomgit")] p.push_str(ATOMGIT_TOOL_USAGE);` |
| `:388-393` | 删除 `ATOMGIT_TOOL_USAGE` 常量（`## ATOMGIT TOOLS:` 段，含 `atomgit_repo/pr/issue` 指引与 OAuth 措辞） |
| `:1560-1571` | 删除测试 `persona_prefers_atomgit_tools_without_exposing_credentials` |

### 1.13 `crates/rustcode-coding/tests/assemble_smoke.rs`（声明外扩）

| 行（原） | 变更 |
| --- | --- |
| `:53-87` | 删除 `#[cfg(feature = "atomgit")] async fn assembled_agent_rejects_raw_atomgit_api_bash()`（该 cfg 随 feature 消失后将永不编译） |

### 1.14 `Cargo.lock`

**未变化**。原因：`rustcode-auth` 仍是 `plugin` feature 的 optional 依赖，依赖边未增减，`cargo` 无需重写锁文件。（未手工编辑。）

## 2. 契约符合性对照

| 契约项 | 结论 |
| --- | --- |
| `src/atomgit/**` 整目录删除 | ✅ 8/8 文件删除 |
| `lib.rs:102` `feature = "atomgit"` cfg 删除 | ✅ `proxy` 门控已收敛 |
| `lib.rs:171` `#[cfg(feature = "atomgit")] pub mod atomgit;` 删除 | ✅ |
| `tools/mod.rs:39/41/76/78/87` 声明与注册删除 | ✅ 3 段全删（`:38-42`、`:76-81`、`:87-92`） |
| `bash_workspace_gate.rs:516/538/878/883/904` cfg 分支（含测试）删除 | ✅ 连同 `:503-562` 两个函数与 `:876-915` 测试段一并删除 |
| `capabilities/Cargo.toml:165,:168` 删除 `atomgit` feature 与注释 | ✅ |
| `coding/Cargo.toml:10` 删除 `atomgit = ["rustcode-capabilities/atomgit"]` | ✅ 整 `[features]` 段删除 |
| `rustcode-auth` optional 依赖处理 | ✅ 见下 §2.3 |
| `Cargo.lock` 由 cargo 自动更新、不手工编辑 | ✅ 无变更需要 |
| 不改契约、不改他人 files_owned | ✅ 见 §3 声明 |

### 2.3 `rustcode-auth` 使用点确认（前置检查要求的结论）

删除前 `rustcode_auth::` 在 capabilities 的全部使用点：

- `src/atomgit/mod.rs:59` —— **本任务删除**
- `src/atomgit/push_label_mw.rs:57` —— **本任务删除**
- `src/plugin/marketplace.rs:303/304/344/358/361/767` —— **仍在用**，归批次 3

删除后复核（`Grep rustcode_auth crates/rustcode-capabilities/src`）：仅剩 `plugin/marketplace.rs` 6 处。因此：

- `provider` feature 的 `dep:rustcode-auth` **已摘除**（provider 侧零使用点，与任务描述一致）；
- optional 依赖声明本身与 `plugin` 的 `dep:rustcode-auth` **保留**，否则批次 3 之前 `plugin` feature 会编译失败。已实测 `cargo check -p rustcode-capabilities --no-default-features --features "web,mcp,plugin,session,memory,notify,cc-hooks" --all-targets` 通过。

## 3. `files_owned` 偏差声明（必须声明）

任务书 `files_owned` 未覆盖以下文件，但**验收标准 `grep … | wc -l == 0` 与「移除全部代码」的语义要求必须触及它们**，故一并处理并在此显式声明：

| 文件 | 偏差理由 |
| --- | --- |
| `src/tools/atomgit.rs`、`src/tools/atomgit_bash_gate.rs` | 属于 atomgit 代码的主体（1316 行），不在 `src/atomgit/**` 内但在 `tools/` 下；不删则 grep 非 0 且残留 `mod` 声明失效 |
| `src/tools/task.rs` | 含 `#[cfg(feature = "atomgit")]` 的中间件装配与两处测试 `base` 常量；不删则 grep 非 0（且 `--check-cfg` 会产生 `unexpected cfg` 警告） |
| `src/egress/mod.rs` | 模块文档 2 处列举 `atomgit`；不删则 grep 非 0 |
| `src/tools/bash.rs` | 2 处 `AtomGit` 大写注释（大小写敏感 grep 不命中，属可选项，为不留悬空引用一并泛化） |
| `coding/src/assemble.rs`、`coding/src/parts.rs`、`coding/src/persona.rs` | `#[cfg(feature = "atomgit")]` 的工具装配 / 中间件 / 人设注入 + `register_atomgit_capabilities`；不删则 grep 非 0。STATUS.md「批次计划 2：`atomgit` feature 移除（capabilities + coding）」明确编码侧在本批次范围内 |
| `coding/tests/assemble_smoke.rs` | atomgit 专用集成测试（随 feature 消失永不编译） |

未改且不属于本任务的并发改动（已在 `git status` 中观察到，非本人修改）：`.codebuddy/artifacts/.../STATUS.md`、`crates/rustcode-config/src/{distribution.rs,lib.rs}`、`crates/rustcode-daemon/src/runtime_host.rs`、`crates/rustcode-tuix/src/version_check.rs`（批次 1 UA 迁移）。

## 4. 自验证证据（真实命令 + 输出）

```
$ cd /workspace/RustCode && cargo fmt --check
（无输出）
exit 0
```

```
$ cd /workspace/RustCode && cargo check --workspace --all-targets 2>&1 | tail -40
   ...（rustcode-daemon 7 条 `variable does not need to be mutable` 历史警告）...
    Checking rustcode v5.0.9 (/workspace/RustCode/crates/rustcode-cli)
    Finished `dev` profile [unoptimized+debuginfo] target(s) in 18.21s
exit 0

$ cargo check --workspace --all-targets 2>&1 | grep -c "^error"
0
```

```
$ cd /workspace/RustCode && cargo check -p rustcode-capabilities --features atomgit --all-targets
error: the package 'rustcode-capabilities' does not contain this feature: atomgit
```

```
$ cd /workspace/RustCode && cargo test -j 1 -p rustcode-capabilities --lib 2>&1 | tail -20
test tools::write_approval::tests::accept_edits_auto_approves_nonsensitive_but_not_sensitive ... ok
...
test tools::tests::run_bounded_yields_default_when_blocking_exceeds_timeout ... ok
test tools::task::tests::child_stream_idle_timeout_fails_the_batch_without_cancel ... ok
test tools::task::tests::hard_explore_transient_failure_retries_once_with_host_model ... ok

test result: ok. 825 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 19.74s
```
（`mcp::registry::tests::trust_key_golden_matches_core_algorithm` 未出现 —— 该用例属 `mcp` feature，非默认，本轮未编译。）

```
$ cd /workspace/RustCode && grep -rn "atomgit" crates/rustcode-capabilities/src crates/rustcode-coding/src --include="*.rs" | wc -l
0
$ grep -rn "AtomGit\|Atomgit\|ATOMGIT" crates/rustcode-capabilities/src crates/rustcode-coding/src --include="*.rs" | wc -l
0
$ grep -rn "atomgit" crates/rustcode-capabilities crates/rustcode-coding --include="*.rs" --include="*.toml" | wc -l
0
```

补充（超出任务书但已跑，用于兜底）：

```
$ cargo check -p rustcode-capabilities --no-default-features --all-targets                      → Finished, exit 0
$ cargo check -p rustcode-capabilities --no-default-features \
      --features "web,mcp,plugin,session,memory,notify,cc-hooks" --all-targets                  → Finished, exit 0
$ cargo test -j 1 -p rustcode-coding    → 430 passed / 0 failed（lib）+ 各集成二进制全 ok + 1 doctest ok
$ cargo clippy -p rustcode-capabilities -p rustcode-coding --all-targets
   → 0 error；8 条 warning 全部位于未改文件
     (config/system_proxy.rs:80、config/config/memory.rs:67、config/store.rs:224、
      capabilities/askpass/server.rs:1、session/manager.rs:3318、tools/web_search.rs:48、
      session/rewind.rs:300/338) —— 均为基线告警
```

`git diff --stat`（已删去他人并发改动的 4 个文件后，本任务 23 个文件）：

```
 crates/rustcode-capabilities/Cargo.toml                        |  12 +-
 crates/rustcode-capabilities/src/atomgit/client.rs             | 295 -----
 crates/rustcode-capabilities/src/atomgit/issue.rs              | 173 ---
 crates/rustcode-capabilities/src/atomgit/mod.rs                |  93 --
 crates/rustcode-capabilities/src/atomgit/models.rs             | 222 ----
 crates/rustcode-capabilities/src/atomgit/pr.rs                 | 287 -----
 crates/rustcode-capabilities/src/atomgit/push_label_mw.rs      | 214 ----
 crates/rustcode-capabilities/src/atomgit/remote.rs             | 178 ---
 crates/rustcode-capabilities/src/atomgit/repo.rs               | 398 -------
 crates/rustcode-capabilities/src/egress/mod.rs                 |   6 +-
 crates/rustcode-capabilities/src/lib.rs                        |  13 +-
 crates/rustcode-capabilities/src/tools/atomgit.rs              | 1190 --------------------
 crates/rustcode-capabilities/src/tools/atomgit_bash_gate.rs    | 126 ---
 crates/rustcode-capabilities/src/tools/bash.rs                 |   4 +-
 crates/rustcode-capabilities/src/tools/bash_workspace_gate.rs  | 102 --
 crates/rustcode-capabilities/src/tools/mod.rs                  |  17 -
 crates/rustcode-capabilities/src/tools/task.rs                 |  15 +-
 crates/rustcode-coding/Cargo.toml                              |   5 -
 crates/rustcode-coding/src/assemble.rs                         |  40 -
 crates/rustcode-coding/src/parts.rs                            |  45 +-
 crates/rustcode-coding/src/persona.rs                          |  22 -
 crates/rustcode-coding/tests/assemble_smoke.rs                 |  36 -
```
（工作区整体 `git diff --stat`：`27 files changed, 50 insertions(+), 3481 deletions(-)`，其中 +50 行来自批次 1 的并发改动。）

## 5. 验收标准（AC）对照

| # | AC | 如何满足 |
| --- | --- | --- |
| AC1 | `atomgit` feature 从 capabilities / coding 两处 Cargo.toml 消失 | §1.8、§1.9；`cargo check --features atomgit` 报「does not contain this feature」 |
| AC2 | `src/atomgit/**` 整目录删除 | §1.1，8/8 文件 `git status` 显示 `D` |
| AC3 | `tools/mod.rs` / `bash_workspace_gate.rs` 的 atomgit cfg 与测试全删 | §1.3、§1.4 |
| AC4 | `rustcode-auth` 依赖按使用点收敛 | §2.3：`provider` 摘除；`plugin` 与 optional 声明保留（仍在用） |
| AC5 | `cargo fmt --check` exit 0 | §4 |
| AC6 | `cargo check --workspace --all-targets` exit 0 | §4（0 error） |
| AC7 | `--features atomgit` 报 feature 不存在 | §4 |
| AC8 | `cargo test -p rustcode-capabilities --lib` 0 failed | §4（825 passed / 0 failed） |
| AC9 | `grep -rn "atomgit" capabilities/src coding/src --include="*.rs" \| wc -l` == 0 | §4（0） |
| AC10 | 文档 / i18n / 配置示例只登记不改 | §6 |

## 6. 非 Rust 侧引用登记（本批次**不处理**，供后续批次 / 文档任务认领）

### 6.1 仓库内 Rust 文件但**不是** atomgit feature（无需删，仅登记）

| file:line | 内容 | 建议 |
| --- | --- | --- |
| `crates/rustcode-config/src/config/mod.rs:2029` | 注释「motivating case: an AtomGit Qwen」 | 可泛化为「a gateway Qwen」 |
| `crates/rustcode-config/src/config/mod.rs:4044,4055,4096,4115,4117,4118,4119` | 测试夹具 provider 名 `"AtomGit-GLM-5.2"`（旧前缀兼容的遗留名） | 与批次 5/8 一并处理，或改名（需同时改 fixture JSON） |
| `crates/rustcode-tuix/src/modals/provider_panel.rs:3302` | `assert!(!ids.contains(&"atomgit".to_string()))` —— **中立性守卫断言，刻意保留** | 保留 |
| `crates/rustcode-tuix/src/render/retained.rs:23393` | 渲染夹具文本「已切换到 AtomGit-deepseek-v4-flash」 | 随 tuix 批次清理 |
| `crates/rustcode-tuix/src/event_loop/commands.rs:502` | 「Historical note: there was a `const OAUTH_PROVIDER_NAME = "AtomGit"`」 | 历史沿革注释，可随批次 5 清理 |
| `crates/rustcode-tuix/src/event_loop/commands.rs:2524` | 注释「active AtomGit provider has been torn down」 | 同批次 5 |
| `crates/rustcode-kernel/tests/fallible_stream.rs:667` | 注释「(the atomgit->DeepSeek path)」 | 泛化即可 |
| `crates/rustcode-capabilities/tests/fixtures/session_404_recovery.jsonl`、`session_p0_sprint_clean.jsonl` | 历史会话转录文本 | 夹具，建议保留（历史数据） |

### 6.2 非 Rust 文件（README / AGENTS / docs / 配置示例）

| file:line | 说明 |
| --- | --- |
| `crates/rustcode-capabilities/README.md:35` | 「`atomgit`：AtomGit REST 工具（repo / pr / issue）。」—— feature 列表，需删该行 |
| `crates/rustcode-capabilities/README.md:60` | feature 门控清单含 `atomgit` |
| `crates/rustcode-coding/README.md:81` | 「未启用 `atomgit` / `lsp` / `notify` 等 L1 feature」 |
| `AGENTS.md:58` | 出站 HTTP 单一入口规则列举 `atomgit` |
| `AGENTS.md:101, 116, 118` | 平台中立条目中把 `atomgit` 描述为「刻意保留的上游开关」——**已失效**，需改写 |
| `AGENTS.md:185, 236-238, 248, 253-255, 313, 329, 343, 383, 384, 423, 424, 431, 432, 483, 552, 555, 563` | 多轮审计沿革把 `atomgit` 记为 gate/LEAVE 项，需按「已删除」统一改写（数量最多，建议单开文档任务） |
| `docs/plans/2026-07-26-atomgit-production-tools.md` | 整篇为 atomgit 生产化设计文档 —— 建议归档 |
| `docs/platform-neutralization.md`、`docs/phase1-refactor-design.md`、`docs/phase2-subagent-status.md`、`docs/REFACTOR_SUMMARY.md`、`docs/REFACTOR_DESIGN_PHASE1.md` | 提及 `atomgit` feature |
| `docs/testing/release-v5.0.0-acceptance.md`、`docs/archive/release-v5.0.1-current-branch-change-report.md`、`docs/archive/release-v5.0.3-core-retirement-acceptance.md`、`docs/archive/coding-runtime-native-migration-design.md`、`docs/archive/coding-runtime-incremental-migration.md`、`docs/archive/2026-07-27-models-dev-pricing-design.md` | 验收/沿革文档提及 |
| `docs/plans/2026-07-26-provider-accounts-model-profiles-{plan,design}.md`、`docs/pr-descriptions/2026-04-27-vscode-extension-ui-mcp.md` | 提及 |
| `docs/UPSTREAM_RUSTCODE_LICENSE.md`、`docs/UPSTREAM_CREDITS.md`、`docs/THIRD_PARTY_NOTICES.md` | fork 归属/致谢原文 —— **MIT 合规落点，建议保留** |

未发现配置示例（`config.toml` 模板 / `.rustcode` 样例）中存在 `atomgit` 键；未发现 i18n 文案（webui `i18n.ts`、tuix 语言文件）中存在 `atomgit` 键。

## 7. 遗留风险与后续项

| 项 | 说明 | 建议负责人 |
| --- | --- | --- |
| FOLLOW-1 | `mount_coding_tools`（`coding/src/assemble.rs:213`）仍返回 `Result<MountedTools, String>`，但唯一 `?` 来源已消失 → `build_coding_agent_with` 的 `Err` 兜底分支不可达。是否收缩为 infallible 需产品决策（会改动 infallible 契约与 `try_build_coding_agent_with` 的存在意义）。本任务刻意未动，避免越界重构。 | solution-architect / 后续重构批次 |
| FOLLOW-2 | `rustcode-auth` 仍被 `plugin/marketplace.rs` 使用，`plugin` feature 保留 optional 依赖边 —— 批次 3（marketplace 无认证化）完成后才能摘除依赖声明与 crate 本身。 | 批次 3 |
| FOLLOW-3 | 非 Rust 侧引用（§6.2，尤以 `AGENTS.md` 约 20 处与 `docs/plans/2026-07-26-atomgit-production-tools.md` 为主）全部未改，需文档批次统一收口。 | 批次 8 / 文档任务 |
| FOLLOW-4 | `crates/rustcode-config/src/config/mod.rs` 的 `"AtomGit-GLM-5.2"` 旧前缀兼容测试夹具仍存在 —— 属「旧配置键仍可加载」的兼容性证据，删除前需确认无磁盘存量依赖。 | 批次 5 / 8 |
| RISK-1 | 本次删除撤销了 `AtomgitBashGate`：此前经 bash 直连 `api.atomgit.com` 会被 fail-closed 拒绝，现在该拒绝面消失（bash 对该主机不再有专项拦截）。这与「彻底不依赖外部平台」的裁决一致（不再有 typed 替代工具，保留门控会变成无法绕开的死墙），但属于**行为变更**，需在发行说明中体现。 | 编排者 / 文档批次 |
| RISK-2 | 本轮与批次 1（UA 迁移）在同一工作树并发，`config` / `daemon` / `tuix` 的 4 个文件由他人修改；`cargo check --workspace` 中的 7 条 daemon 告警来自 `legacy_convert.rs`，与本任务无关，未处理。 | — |
