---
kind: implementation
id: B3-marketplace
from: code-implementer
to: [code-reviewer]
feature: 2026-09-09-local-identity
status: done
decision: proceed
requires: [本地身份改造批次 1+2]
files_owned:
  - crates/rustcode-capabilities/src/plugin/marketplace.rs
architecture_constraints:
  touches_runtime_lifecycle: false
  touches_persistence: false
  touches_cross_crate_deps: false
created: 2026-09-09
---

# B3 — `plugin marketplace` 下载改为无认证（删除平台 token 分支）

## 1. 改动清单（唯一文件：`crates/rustcode-capabilities/src/plugin/marketplace.rs`）

改动后行号 / 原行号对照，全部为删除 + 分支收敛，**无新增生产逻辑**。

| 位置（原 → 现） | 改动 |
| --- | --- |
| 原 `:283-289` → 删除 | `basic_auth_header()`（base64 编 `Authorization: Basic`）整体删除：其唯一生产者是 token 注入路径。 |
| 原 `:291-298` → 删除 | `extra_header_config()`（`http.<host>.extraHeader` 作用域配置）整体删除，同上。 |
| 原 `:300-309` → 删除 | `live_credentials()` 整体删除（`:303-304` `get_stored_auth()` / `get_valid_token()` 是全部 `rustcode_auth` 依赖的源头；删后该函数必然返回 None，按指令整函数删除）。 |
| 原 `:311-322` → 删除 | `auth_retry_args()` 整体删除（无 token 可注入 → 恒为 None）。 |
| 原 `:330-349` → 删除 | `relogin_hint()` 整体删除（`:344` `managed_login_available()` 分支消失；其唯一调用点是已被删除的“带认证重试失败”分支）。 |
| 原 `:353-366` → 现 `:291-299` | `auth_required_message()` 收敛：删除 `:358` `!managed_login_available()` 与 `:361` `get_stored_auth().is_some()` 两个分支；**只保留 `host_is_trusted(url)` 判定**——非信任域返回原 `PluginGitAuthUntrusted`（SSH / 本地 git 凭证），信任域在其后追加 `PluginReloginHintNeutral`（“本构建无托管登录服务…”），使 `host_is_trusted` 仍是下载路径中唯一的判定点（详见 §2 偏差 2）。 |
| 原 `:368-415` → 现 `:306-327` | `clone_with_optional_auth()`：删除 `run(Some(&cargs))` 认证重试分支（含失败目录清理、`PluginGitAuthRetryFailed` 文案）；现在一次无认证 clone，失败即 `auth_required_message` / `PluginGitCloneFailed`。 |
| 原 `:417-459` → 现 `:330-348` | `git_pull_ff()`：对称删除认证重试分支，一次无认证 `pull --ff-only`。 |
| 原 `:767` → 删除 | 测试内 `managed_login_available()` 分支（见 §3 用例删除）。 |

保留且未改：`find_git` / `git_command`（`GIT_TERMINAL_PROMPT=0` 等防 tty 死锁守卫）/ `is_git_auth_failure` / `git_clone` / `add_marketplace` / `update_marketplace` 等。

## 2. 契约符合性（对照任务指令）

- 删除全部 `rustcode_auth::` 调用（6 处：303、304、344、358、361、767）→ **满足**，`grep -rn "rustcode_auth" crates/rustcode-capabilities/src | wc -l` = **0**。
- “取不到 token 就返回 None” 导致整体不可达的函数整函数删除 → **满足**（`live_credentials`），并连带清理 `auth_retry_args` 及两个调用点的重试分支。
- 下载路径只保留 `host_is_trusted` 判定 → **满足**（`auth_required_message` 是下载/更新认证失败路径的唯一分支点）。
- 未动 `capabilities/Cargo.toml` → **满足**。

声明的偏差（均为约束下的必要取舍，非行为回退）：
1. `clone_with_optional_auth` 保留第 3 个形参（改名 `_target`）：`installer.rs:165/178/312` 三处调用点不属于 `files_owned`，改签名会破坏编译；形参加 `_` 前缀避免 unused 告警，文档已说明“目标目录由 `add_args` 追加”。函数名同样因跨文件未改。
2. 信任域分支未直接复用非信任域文案，而是追加 `PluginReloginHintNeutral`：若两臂完全相同则 `host_is_trusted` 在下载路径中被架空；且 `host_is_trusted` 若彻底消失，`plugin/url.rs` 中该 `pub(crate)` 函数将只剩测试引用（已是死代码，见 §5）。文案不含 `/login`，不会把用户导向死路。
3. `Msg::PluginGitAuthExpired` / `PluginGitAuthLoginRequired` / `PluginGitAuthRetryFailed` / `PluginReloginHintManaged` 在本 crate 不再被引用（i18n 枚举与翻译属 `rustcode-config`，不在 `files_owned`，未动）。

## 3. 被删除的测试（8 条，全名）

- `plugin::marketplace::tests::basic_auth_header_encodes_user_colon_token`
- `plugin::marketplace::tests::extra_header_config_is_scoped_to_host`
- `plugin::marketplace::tests::auth_retry_args_none_when_untrusted_host`
- `plugin::marketplace::tests::auth_retry_args_none_when_not_logged_in`
- `plugin::marketplace::tests::auth_required_message_trusted_not_logged_in_suggests_login`（断言“信任域未登录 → 引导 /login”，该行为已消失）
- `plugin::marketplace::tests::auth_required_message_neutral_build_never_pitches_login`（依赖已删除的 `managed_login_available()` / `relogin_hint()`）
- `plugin::marketplace::tests::auth_required_message_trusted_logged_in_says_session_expired`（依赖 `get_stored_auth()` 分支）
- `plugin::marketplace::tests::injected_auth_header_is_not_persisted_to_git_config`（锁定的“header 不落盘”不变量随注入路径一并消失）

保留 `auth_required_message_untrusted_host_suggests_ssh_not_login`（仍成立：非信任域 → SSH、不提 /login），断言未改。`git_runs_rejects_present_but_failing_stub` 未做任何改动。

## 4. 自验证证据（真实执行）

```
$ cd /workspace/RustCode && cargo fmt --check; echo "FMT_EXIT=$?"
FMT_EXIT=0
```

```
$ cargo check --workspace --all-targets > /dev/null 2>&1; echo "CHECK_EXIT=$?"
CHECK_EXIT=0
（完整输出尾部：warning: `rustcode-daemon` (lib test) generated 7 warnings ... Finished `dev` profile ... in 56.19s）
```

```
$ cargo test -j 1 -p rustcode-capabilities --lib 2>&1 | tail -8
test tools::write_approval::tests::system_prefix_write_prompts_and_is_not_remembered ... ok
test tools::write_approval::tests::write_targets_extracts_per_tool ... ok
test tools::tests::run_bounded_yields_default_when_blocking_exceeds_timeout ... ok
test tools::task::tests::child_stream_idle_timeout_fails_the_batch_without_cancel ... ok
test tools::task::tests::hard_explore_transient_failure_retries_once_with_host_model ... ok

test result: ok. 825 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 19.98s
```

`plugin` 是 opt-in feature，上面这条默认-feature 命令不编译 marketplace 模块，故补充执行（同一命令加 `--features plugin`）：

```
$ cargo test -j 1 -p rustcode-capabilities --features plugin --lib 2>&1 | tail -3
test result: ok. 972 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 21.88s

$ cargo test -j 1 -p rustcode-capabilities --features plugin --lib marketplace 2>&1 | tail -20
running 13 tests
test plugin::marketplace::tests::add_marketplace_canonical_name_differs_from_url_tail ... ok
test plugin::marketplace::tests::add_marketplace_sanitizes_traversal_in_manifest_name ... ok
test plugin::installer::tests::install_external_url_dedups_with_marketplace ... ok
test plugin::marketplace::tests::add_marketplace_rejects_duplicate ... ok
test plugin::marketplace::tests::git_auth_failure_is_detected ... ok
test plugin::marketplace::tests::git_command_runs_noninteractively ... ok
test plugin::marketplace::tests::git_runs_rejects_present_but_failing_stub ... ok
test plugin::marketplace::tests::add_marketplace_single_plugin_fallback ... ok
test plugin::marketplace::tests::add_marketplace_with_manifest ... ok
test plugin::marketplace::tests::auth_required_message_untrusted_host_suggests_ssh_not_login ... ok
test plugin::state::tests::round_trips_marketplaces_file ... ok
test plugin::marketplace::tests::list_marketplaces_returns_added ... ok
test plugin::marketplace::tests::remove_marketplace_works ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 959 filtered out; finished in 0.49s
```

```
$ grep -rn "rustcode_auth" crates/rustcode-capabilities/src | wc -l
0
```

clippy（附加，非任务要求）：`cargo clippy -p rustcode-capabilities --features plugin --all-targets` 退出 0；本文件 0 条告警，capabilities 内告警仅 `plugin/url.rs:104 scheme_host_prefix`（本次新引入，见 §5）、`plugin/manifest.rs:268/269`、`askpass/server.rs:1`（后三条为基线既有）。

## 5. Cargo.toml `dep:rustcode-auth` 结论

**可摘除。** 依据：`grep -rn "rustcode_auth" crates/rustcode-capabilities/src` 计数为 **0**，capabilities 已无任何 `rustcode_auth` 使用点；`plugin` feature 的 `dep:rustcode-auth` 及 `[dependencies] rustcode-auth = { path = "../rustcode-auth", optional = true }` 均可由后续批次统一删除。本批按要求未动 Cargo.toml。

## 6. `git diff --stat`

```
$ git --no-pager diff --stat -- crates/rustcode-capabilities/src/plugin/marketplace.rs
 .../src/plugin/marketplace.rs | 304 ++-------------------
 1 file changed, 28 insertions(+), 276 deletions(-)
```

工作区整体 `git --no-pager diff --stat` 另有 6 个文件（`.codebuddy/.../STATUS.md`、`rustcode-config/src/lib.rs`、`rustcode-daemon/src/{api_auth.rs,lib.rs,login_state.rs,login_state_tests.rs}`），为本批次开始前既存的未提交改动，**非本次产生**（本次仅用 Edit 改过 marketplace.rs 一个文件）。

## 7. 遗留风险与后续项

1. **`plugin/url.rs:104` `scheme_host_prefix` 变为死代码（新 `dead_code` 告警）**：其唯一非测试调用者是已删除的 `extra_header_config`。`url.rs` 不在 `files_owned`，未改动。建议后续摘除 `rustcode-auth` 的批次同时删除该函数及其测试 `url::tests::scheme_host_prefix_strips_path`。
2. **`host_is_trusted` 仅剩错误文案用途**：下载不再因信任域而注入任何凭证，其现存价值是区分错误提示；若后续连文案也统一，则该函数与其 url.rs 测试应一并清理。
3. **行为变化（已裁决，非缺陷）**：需要平台登录的私有市场不再可用；当前构建无平台可连，实际无损失。
4. **已知环境抖动**：`plugin::marketplace::tests::git_runs_rejects_present_but_failing_stub` 为基线偶发失败用例，与本次改动无关，本次两轮运行均通过（如上输出 `ok`）。若后续复现失败，属已知环境抖动，不应归因本批次。
5. `Msg::PluginGitAuth{Expired,LoginRequired,RetryFailed}` / `PluginReloginHintManaged` 现无调用点，属 `rustcode-config` i18n 范畴，建议由删除 `rustcode-auth` 的批次评估是否一并清理枚举与翻译。
