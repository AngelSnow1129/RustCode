---
kind: implementation
id: B4a-daemon-auth
from: code-implementer
to: [code-reviewer]
feature: 2026-09-09-local-identity
status: done
decision: proceed
requires: [B1, B2]
files_owned:
  - crates/rustcode-daemon/src/api_auth.rs
  - crates/rustcode-daemon/src/lib.rs
  - crates/rustcode-daemon/src/login_state.rs
  - crates/rustcode-daemon/src/login_state_tests.rs
architecture_constraints:
  touches_runtime_lifecycle: false
  touches_persistence: false
  touches_cross_crate_deps: false
created: 2026-09-09
---

# B4a —— 移除 daemon 侧 OAuth 登录与平台账号面

## 结论速览

- 两条验收 grep 均为 **0**：`login_state|LoginRecord|LoginSession` = 0，`auth/login|/auth/` = 0。
- **未留红**：`cargo fmt --check` / `cargo check -p rustcode-daemon --all-targets` /
  `cargo test -j 1 -p rustcode-daemon --lib`（286 passed）/ `cargo check --workspace --all-targets`
  / `cargo clippy -p rustcode-daemon --all-targets` 全部 exit 0。
- `commands.rs` **未被本任务触碰**，且它对 `login_state` 零依赖，因此 4a 可独立编译通过；
  它残留的 `rustcode_auth::*` 平台账号调用见下文「外部引用者登记」，交 4b。

---

## 1. 改动清单

### 1.1 `crates/rustcode-daemon/src/api_auth.rs` —— 全部内容删除（704 行 → 0 行）

整个模块只承载 `/auth/*` 五条端点及其 DTO/状态映射，随端点一并清空：

| 原 file:line | 删除内容 |
| --- | --- |
| `api_auth.rs:1-21` | axum / serde / `rustcode_auth as auth` / `login_state::{...}` / `AppState, LoginSessionsStore` 全部 import |
| `api_auth.rs:23` | `MAX_LOGIN_RECORDS`（登录记录容量上限） |
| `api_auth.rs:25-43` | `LoginPollStep` / `LoginPollResult` / `LoginPollError` |
| `api_auth.rs:49-114` | 响应 DTO：`AuthStatusResponse`（含 `managed_available` / `auth_path` / `user` / `token`）、`TokenInfo`、`LoginStartResponse`、`LoginPollResponse`、`LoginStartRequest`、`default_true` |
| `api_auth.rs:68-76` | `classify_auth_status()`（logged_in / expired 分类器） |
| `api_auth.rs:120-165` | handler `auth_status()`（GET /auth/status） |
| `api_auth.rs:167-179` | `managed_login_unavailable_response()`（501 `managed_login_unavailable`） |
| `api_auth.rs:181-259` | handler `auth_login_start()`（POST /auth/login/start，含 `auth::start_login()` / `open_browser_best_effort()`） |
| `api_auth.rs:261-275` | handler `auth_login_poll()`（POST /auth/login/:login_id/poll） |
| `api_auth.rs:277-304` | handler `auth_login_cancel()`（DELETE /auth/login/:login_id） |
| `api_auth.rs:306-335` | handler `auth_logout()`（POST /auth/logout，含 `auth::logout()`） |
| `api_auth.rs:337-433` | `poll_login_session()`（`session.poll_once()` / `finish()` / `save_auth()` 的 spawn_blocking 编排） |
| `api_auth.rs:435-505` | `apply_poll_completion()` / `step_from_snapshot()` / `login_poll_response()` / `terminal_login_response()` |
| `api_auth.rs:507-536` | `cleanup_login_sessions()`（TTL 过期与记录回收） |
| `api_auth.rs:538-704` | `mod tests`：8 个登录/账号面单测（`no_credentials_is_logged_out_not_expired`、`present_and_usable_token_is_logged_in_not_expired`、`present_but_unusable_token_is_expired`、`neutral_build_reports_managed_unavailable`、`neutral_build_login_start_returns_actionable_501`、`login_start_response_has_no_runtime_protocol_selector`、`login_terminal_states_are_non_success_and_non_retryable`、`login_pending_and_authorized_states_remain_successful`、`login_retryable_failure_remains_service_unavailable`） |

### 1.2 `crates/rustcode-daemon/src/login_state.rs` —— 全部内容删除（253 行 → 0 行）

`LoginRecordState` 枚举（`login_state.rs:19`）、`LoginRecord<S = rustcode_auth::LoginSession>`
（`login_state.rs:69`，含 `new` / `snapshot` / `begin_poll` / `apply_poll` / `cancel` /
`expire_if_due` / `removable_at` / `fail`）、`LoginStateSnapshot`、`BeginPoll`、`ApplyPoll`、
`PollCompletion`、`LOGIN_TTL`、`LOGIN_RETRY_AFTER_MS`、`TERMINAL_RETENTION` 全部移除。

### 1.3 `crates/rustcode-daemon/src/login_state_tests.rs` —— 全部内容删除（219 行 → 0 行）

10 个 `LoginRecord` 状态机单测（`login_state_tests.rs:28-236`）全部移除。

### 1.4 `crates/rustcode-daemon/src/lib.rs` —— 10 处（+2 / −50）

| 原 file:line | 变更 |
| --- | --- |
| `lib.rs:28` | 删 `mod api_auth;` |
| `lib.rs:37-39` | 删 `mod login_state;` 与 `#[cfg(test)] mod login_state_tests;` |
| `lib.rs:80` | `routing::{delete, get, post}` → `routing::{get, post}`（`delete` 仅被 `/auth/login/:login_id` 使用） |
| `lib.rs:93` | `tokio::sync::{mpsc, watch, Mutex, RwLock}` → `{mpsc, watch, RwLock}`（`Mutex` 仅被 `login_start_lock` 使用；`lib.rs:4608` 用的是全路径 `tokio::sync::Mutex`，不受影响） |
| `lib.rs:210-213` | 删 `LoginSessionsStore` 类型别名及其 doc 注释（唯一引用 `login_state::LoginRecord` 的处） |
| `lib.rs:231-246` | 删 `coded_json_error()`（清空 api_auth 后成为死代码，编译器报 `never used`；`json_error()` 保留，`ApiError.code/retryable` 字段仍由 `delete_session_api_error` 使用） |
| `lib.rs:610-613` | 删 `AppState::login_sessions` 与 `AppState::login_start_lock` 两字段及 doc 注释 |
| `lib.rs:6142-6143` | 删生产 `AppState` 构造中的 `login_sessions` / `login_start_lock` 初始化 |
| `lib.rs:6279-6287` | 删 `// Auth API (P0)` 注释 + 5 条 `/auth/*` 路由注册 |
| `lib.rs:6423-6427` | 删 `--help` 端点清单里的 5 条 `/auth/*` 条目 |
| `lib.rs:7216-7217` | 删测试用 `chat_test_state()` 中的 `login_sessions` / `login_start_lock` 初始化 |

---

## 2. `/auth/*` 端点清单（5 条，全部删除）

| 方法 | 路径 | 原 handler | 路由注册点 | help 清单点 |
| --- | --- | --- | --- | --- |
| GET | `/auth/status` | `api_auth::auth_status` | `lib.rs:6280` | `lib.rs:6423`（`Msg::DaemonEpAuthStatus`） |
| POST | `/auth/login/start` | `api_auth::auth_login_start` | `lib.rs:6281` | `lib.rs:6424`（`Msg::DaemonEpLoginStart`） |
| POST | `/auth/login/:login_id/poll` | `api_auth::auth_login_poll` | `lib.rs:6282-6285` | `lib.rs:6425`（`Msg::DaemonEpLoginPoll`） |
| DELETE | `/auth/login/:login_id` | `api_auth::auth_login_cancel` | `lib.rs:6286` | `lib.rs:6426`（`Msg::DaemonEpLoginCancel`） |
| POST | `/auth/logout` | `api_auth::auth_logout` | `lib.rs:6287` | `lib.rs:6427`（`Msg::DaemonEpLogout`） |

注：这 5 条曾挂在 `protected` router 上（`require_webui_token` + `require_app_user_id` 两层
middleware 之内），删除后两层 `route_layer` 原样保留，其余受保护路由不受影响。

---

## 3. 前置 Grep：外部引用者登记

### 3.1 `login_state` / `LoginRecord` / `LoginSession` 的引用者

daemon 内除被删的 3 个文件外，**只有 `lib.rs:212-213` 一处**（`LoginSessionsStore` 别名），已随本任务删除。
`commands.rs` / `live_api.rs` / `runtime_host.rs` **零引用** —— 故本任务无中间态、无需 4b 兜底编译。

daemon 之外仍有引用者（**均不在 files_owned，本次未动**）：

| file:line | 内容 | 归属 |
| --- | --- | --- |
| `crates/rustcode-tuix/src/modals/onboarding_wizard.rs:394` | doc：`Live LoginSession produced by start_login()` | 后续 tuix 批次 |
| `crates/rustcode-tuix/src/modals/onboarding_wizard.rs:401` | `pending_session: Option<rustcode_auth::oauth::LoginSession>` | 后续 tuix 批次 |
| `crates/rustcode-tuix/src/modals/onboarding_wizard.rs:458` | doc 引用 `LoginSession` | 后续 tuix 批次 |
| `crates/rustcode-tuix/src/modals/onboarding_wizard.rs:481,488` | `take_pending_session() -> Option<rustcode_auth::oauth::LoginSession>` | 后续 tuix 批次 |
| `crates/rustcode-tuix/src/event_loop/oauth_poll.rs:4,5,12,34,55,63` | 整模块围绕 `LoginSession::poll_once/finish` 的后台轮询线程 | 后续 tuix 批次 |
| `crates/rustcode-tuix/src/event_loop/mod.rs:9421` | 注释 `Pull the LoginSession out of the wizard` | 后续 tuix 批次 |

### 3.2 daemon 内残留的 `rustcode_auth` 调用（不属本任务范围）

| file:line | 内容 | 归属 |
| --- | --- | --- |
| `crates/rustcode-daemon/src/commands.rs:490` | `rustcode_auth::get_stored_auth()` | **4b（/status 平台字段）** |
| `crates/rustcode-daemon/src/commands.rs:610` | `!rustcode_auth::managed_login_available()` | **4b** |
| `crates/rustcode-daemon/src/commands.rs:613` | `rustcode_auth::get_stored_auth()` | **4b** |
| `crates/rustcode-daemon/src/commands.rs:649` | `let auth = rustcode_auth::get_stored_auth();` | **4b** |
| `crates/rustcode-daemon/src/commands.rs:822` | 测试 `assert!(!rustcode_auth::managed_login_available())` | **4b** |
| `crates/rustcode-daemon/src/lib.rs:5370` | `rustcode_auth::oauth::open_browser(&local_url)` —— **打开本地 WebUI URL 的通用浏览器工具，非 OAuth 登录**。虽在我 files_owned 内，但迁移它需要为该 helper 另找归属（架构决策），不属"删登录入口"范围 | 交「整体删除 rustcode-auth」的批次；`Cargo.toml:21` 的 `rustcode-auth` 依赖因此仍需保留 |

### 3.3 其它非代码引用（未动，属文档/i18n 批次）

- `crates/rustcode-daemon/README.md`：提及 `api_auth`。
- `crates/rustcode-config/src/i18n/messages.rs`：`Msg::DaemonEpAuthStatus` / `DaemonEpLoginStart` /
  `DaemonEpLoginPoll` / `DaemonEpLoginCancel` / `DaemonEpLogout` / `DaemonApiLoginSessionLimit` /
  `DaemonApiManagedUnavailable` / `DaemonApiLoginStartFailed` / `DaemonApiLoginTaskFailed` /
  `DaemonApiInvalidLoginId` / `DaemonApiLoginSessionGone` / `DaemonApiLoginPollUnavailable` /
  `DaemonApiLoginExchangeFailed` / `DaemonApiAuthPersistFailed` / `DaemonApiLoginExpired` /
  `DaemonApiLoginCancelled` / `DaemonApiLogoutFailed` 等变体在本批后失去 daemon 消费者。
  枚举变体无 `dead_code` 告警，编译不受影响；清理留给 i18n 批次。

---

## 4. 已知偏差声明

1. **`api_auth.rs` 被清空为 0 行，而非物理 `rm`。**
   任务约束「Bash 只允许 `cargo fmt|check|test|build|clippy` 与 `grep -c`」，我没有可用的删除通道
   （`rm` / `git rm` 均超出白名单且触发审批）。`login_state.rs` / `login_state_tests.rs` /
   `api_auth.rs` 三者均已 **内容清零 + 摘除 mod 声明**，编译上等价于不存在（未声明的 `.rs`
   既不参与编译也不被 `cargo fmt` 覆盖）。
   **收尾需一条命令**（留给有删除权限的执行方）：
   ```
   git rm crates/rustcode-daemon/src/api_auth.rs \
          crates/rustcode-daemon/src/login_state.rs \
          crates/rustcode-daemon/src/login_state_tests.rs
   ```
2. **`api_auth.rs` 未在任务书中标注"整文件删除"，实际结果是整文件清空。**
   原因：该文件全部 5 个 handler 都是 `/auth/*`，删完端点后无残留代码，故等价于整文件删除。
3. **超出"删路由"字面范围的 2 处连带清理**（均在 files_owned 内、均为编译器判定的死代码，
   不清则 daemon lib 留 3 条新 warning，违反"clippy 干净"）：
   - `lib.rs:231-246` `coded_json_error()`：清空 api_auth 后 `never used`。
   - `lib.rs:80` / `lib.rs:93` 两处 import 收窄：`delete` / `Mutex` 变为 `unused import`。

---

## 5. 自验证证据（真实命令与输出）

### 5.1 `cargo fmt --check`
```
$ cd /workspace/RustCode && cargo fmt --check; echo "EXIT_FMT=$?"
EXIT_FMT=0
```
（无输出，exit 0）

### 5.2 `cargo check -p rustcode-daemon --all-targets`
```
$ cargo check -p rustcode-daemon --all-targets > /tmp/check4a.log 2>&1; echo "EXIT_CHECK=$?"; tail -3 /tmp/check4a.log
EXIT_CHECK=0

warning: `rustcode-daemon` (lib test) generated 7 warnings (run `cargo fix --lib -p rustcode-daemon --tests` to apply 7 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.26s
```
警告分布（`grep -E "^(warning|error)(\[|:)" | sort | uniq -c`）：
```
      1 warning: function `migrate_sessions_from` is never used
      1 warning: function `parse_scutil_proxy` is never used
      1 warning: `rustcode-capabilities` (lib) generated 1 warning
      1 warning: `rustcode-config` (lib) generated 1 warning
      1 warning: `rustcode-daemon` (lib test) generated 7 warnings
      7 warning: variable does not need to be mutable
```
**`rustcode-daemon` (lib) 警告数为 0**；7 条 `does not need to be mutable` 全部落在
`legacy_convert.rs:3258/3303/3329/3453/3495` 等未触碰的既有测试代码。
（对照：清理前该处为 `rustcode-daemon (lib) generated 3 warnings` ——
`coded_json_error is never used` / `unused import: delete` / `unused import: Mutex`，已在 §4.3 消除。）

### 5.3 `cargo test -j 1 -p rustcode-daemon --lib`
```
$ cargo test -j 1 -p rustcode-daemon --lib >/tmp/v3.log 2>&1; echo "EXIT_TEST=$?"; tail -4 /tmp/v3.log
EXIT_TEST=0
test tests::session_catalog_io_is_offloaded_and_single_flight ... ok

test result: ok. 286 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.00s
```
首轮（tail -25）节选：
```
test tests::stale_chat_cleanup_cannot_remove_a_replacement_operation ... ok
test tests::stop_accepts_both_session_and_request_aliases ... ok
test tests::strip_query_key_preserves_other_params ... ok
test webui::tests::serves_embedded_index ... ok
test webui::tests::unknown_path_falls_back_to_index ... ok
test tests::resolve_session_by_short_and_full_id_across_buckets ... ok
test tests::session_repair_dry_run_does_not_write_and_apply_restores_strict_load ... ok
test tests::session_catalog_io_is_offloaded_and_single_flight ... ok

test result: ok. 286 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.88s
```
（286 = 原 305 减去被删的 19 个登录/账号面单测：api_auth 9 + login_state_tests 10。）

### 5.4 两条残留 grep
```
$ grep -rn "login_state\|LoginRecord\|LoginSession" crates/rustcode-daemon/src | wc -l
0
$ grep -rn "auth/login\|/auth/" crates/rustcode-daemon/src | wc -l
0
```

### 5.5 附加：workspace 编译 + clippy
```
$ cargo check --workspace --all-targets > /tmp/ws4a.log 2>&1; echo "EXIT_WS=$?"; grep -cE "^error" /tmp/ws4a.log; tail -2 /tmp/ws4a.log
EXIT_WS=0
0
    Checking rustcode v5.0.9 (/workspace/RustCode/crates/rustcode-cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 28.95s
```
```
$ cargo clippy -p rustcode-daemon --all-targets > /tmp/clippy4a2.log 2>&1; echo "EXIT_CLIPPY=$?"
EXIT_CLIPPY=0
```
clippy 在 `rustcode-daemon/src/lib.rs` 上仅 2 条既有告警，均在未触碰代码：
`lib.rs:4772`（empty line after doc comment）、`lib.rs:4508`（too many arguments 11/7）。

> **一次瞬时红需登记**：首轮 clippy 曾 `EXIT_CLIPPY=101`，报
> `rustcode-capabilities` (lib) `E0425: cannot find function auth_retry_args` /
> `relogin_hint`（`plugin/marketplace.rs`）。该文件**不在我的 files_owned**、我全程未编辑它，
> 且 `git status` 显示它在我开工前就已是未提交的修改态（−302 行，疑为并行批次进行中）。
> 随后 `cargo check -p rustcode-capabilities --lib` = exit 0、clippy 重跑 = exit 0，
> 确认为他人并发编辑的中间态，非本任务引入。

### 5.6 `git diff --stat`（本任务范围）
```
$ git --no-pager diff --stat -- crates/rustcode-daemon
 crates/rustcode-daemon/src/api_auth.rs          | 704 ------------------------
 crates/rustcode-daemon/src/lib.rs               |  52 +-
 crates/rustcode-daemon/src/login_state.rs       | 253 ---------
 crates/rustcode-daemon/src/login_state_tests.rs | 219 --------
 4 files changed, 2 insertions(+), 1226 deletions(-)
```
```
$ git --no-pager diff --numstat -- crates/rustcode-daemon
0	704	crates/rustcode-daemon/src/api_auth.rs
2	50	crates/rustcode-daemon/src/lib.rs
0	253	crates/rustcode-daemon/src/login_state.rs
0	219	crates/rustcode-daemon/src/login_state_tests.rs
```
全仓 `git status --porcelain` 另有 2 个**非我改动**的既存脏文件（开工前即存在，未触碰）：
`crates/rustcode-capabilities/src/plugin/marketplace.rs`、`crates/rustcode-config/src/lib.rs`。

---

## 6. 验收标准对照

| AC | 结论 |
| --- | --- |
| 删 `api_auth.rs` 中 OAuth 登录路由与处理 | ✅ 5 个 handler + 全部 DTO/状态映射/单测清零（§1.1） |
| 删 `lib.rs` 的 `/auth/*` 路由注册（5 条） | ✅ `lib.rs:6280-6287` 连同 `// Auth API (P0)` 注释删除；`--help` 清单 5 条同步删除（§2） |
| 删其 `.merge(...)` / 使用点 | ✅ 无独立 `/auth` 子 router，5 条直接挂在 `protected` 链上；`protected` 的 `.merge` 保留（其余路由需要）。使用点 = `AppState` 两字段 + 2 处构造 + `LoginSessionsStore` 别名，全部删除（§1.4） |
| `login_state.rs` 整文件删除 | ✅ 内容清零 + mod 声明删除；物理 `rm` 受 Bash 白名单阻断，见 §4.1 |
| `login_state_tests.rs` 整文件删除 | ✅ 同上 |
| 删 `mod login_state;` / `#[cfg(test)] mod login_state_tests;` | ✅ `lib.rs:37-39`；另删 `mod api_auth;`（`lib.rs:28`） |
| 不动 `commands.rs` | ✅ 零改动（`git diff` 无此文件） |
| 不动 `runtime_host.rs` | ✅ 零改动 |
| 不动 capabilities / tuix / config / i18n / 文档 / 扩展 / webui | ✅ diff 仅含 4 个 daemon 文件；capabilities/config 的脏状态为他人既有改动 |
| 随时可编译、不留红 | ✅ 每步 `cargo check` 通过，终态 fmt/check/test/workspace/clippy 全 exit 0 |
| 两条 grep = 0 | ✅ 0 / 0 |

---

## 7. 遗留风险与后续项

1. **三个空文件待物理删除**（§4.1）。责任建议：编排者或有 shell 删除权限的执行方，一条 `git rm` 即可。
2. **4b 必须处理 `commands.rs` 的 5 处 `rustcode_auth` 调用**（§3.2）。本批未产生阻塞它的中间态。
3. **`lib.rs:5370` 的 `rustcode_auth::oauth::open_browser`** 是 daemon 对 `rustcode-auth` 的最后一处
   非账号面依赖（打开本地 WebUI URL）。删除 `rustcode-auth` crate 前必须为该 helper 另立归属，
   否则 `Cargo.toml:21` 的依赖无法摘除。建议由「删除 rustcode-auth」批次统一处理。
4. **客户端契约破坏（预期）**：WebUI / VSCode 扩展若仍请求 `/auth/status` 等，现在会拿到 axum 的
   404 而非此前的 200/501。这是本批的既定语义（platform account 面整体下线），
   前端侧收口留给 webui / 扩展批次；本批未修改任何前端代码。
5. **i18n 死变体**（§3.3）：17 个 `Msg::DaemonEp*/DaemonApi*Login*` 变体在 daemon 侧已无消费者，
   不影响编译，清理留给 i18n 批次。
