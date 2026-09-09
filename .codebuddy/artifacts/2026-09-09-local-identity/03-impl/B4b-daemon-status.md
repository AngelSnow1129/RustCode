---
kind: implementation
id: B4b-daemon-status
from: code-implementer
to: [code-reviewer]
feature: 2026-09-09-local-identity
status: done
decision: proceed
requires: [DESIGN-001]
files_owned:
  - crates/rustcode-daemon/src/commands.rs
architecture_constraints:
  touches_runtime_lifecycle: false
  touches_persistence: false
  touches_cross_crate_deps: false
created: 2026-09-09
---

# B4b — 清除 rustcode-daemon/src/commands.rs 的平台账号 / 登录态字段

## 1. 改动清单

唯一改动文件：`crates/rustcode-daemon/src/commands.rs`（+2 / −90）。

| 位置（改动后 / 原行号） | 变更 |
|---|---|
| :65 DTO（原 :68） | 删除 `CommandResult::Whoami` 变体（`logged_in` / `username` / `name` / `email` 四个平台账号字段随变体整体移除） |
| :68 DTO（原 :74） | 删除 `CommandResult::Status` 的 `logged_in: bool`、`username: Option<String>` |
| 原 :489 `exec_whoami()` | 整函数删除（原 :490 `rustcode_auth::get_stored_auth()` 随之消失） |
| 原 :783 分派 arm | 删除 `"whoami" => exec_whoami(),`；`whoami` 落回 `other =>` 分支，返回显式 `Msg::DaemonCmdUnknown` 错误 |
| 原 :587 `render_login_line()` | 删除（登录行渲染，仅服务于 stored auth） |
| 原 :595 `format_login_identity()` | 删除（平台 name(username) 拼接） |
| 原 :605 `render_login_line_from_stored_auth()` | 删除（原 :610 `managed_login_available()` 与原 :613 `get_stored_auth()` 两处调用随之消失） |
| :562 `assemble_status()`（原 :622） | 签名由 `(login, body, proxy, instructions)` 收缩为 `(body, proxy, instructions)`，去掉恒空的 login 段拼接 |
| 原 :649 `exec_status()` | 删除 `let auth = rustcode_auth::get_stored_auth();` |
| :599 / :605 `exec_status()` | `assemble_status` 调用去掉首参；`CommandResult::Status` 构造去掉 `logged_in` / `username` |
| 原 :818 测试用例 | 删除 `tests::neutral_status_omits_managed_login_line`（含原 :822 `managed_login_available()` 调用） |

**被删测试用例全名**：`crates/rustcode-daemon/src/commands.rs::tests::neutral_status_omits_managed_login_line`

删除理由：该用例断言对象（`render_login_line_from_stored_auth`、登录行存在性、`managed_login_available`）全部被移除，无法编译；按「删用例，不改断言」处理，未改写其断言、未弱化其余用例断言。

**决策说明（需 review 确认）**：`CommandResult::Whoami` 的全部字段均为平台账号字段；删空后若保留空变体 + 恒 `Ok` 返回即构成「假成功」，违反显式错误原则，故连同 `exec_whoami` 与分派 arm 一并移除，使 daemon `whoami` 返回显式 unknown-command 错误。未新增 i18n 文案（i18n 属 rustcode-config，不在 files_owned）。**未**为 `whoami` 发明本地身份语义，属后续批次设计范围。

## 2. 契约符合性

- 未修改任何已冻结契约。
- `CommandResult` 为 `pub(crate)`，删字段无跨 crate 编译影响（已由 `cargo check --workspace --all-targets` 证明）。
- 未恢复 bridge / v1-v2 开关 / core session 磁盘模型 / 任何 fallback。
- 无持久化、无 runtime 生命周期、无跨 crate 依赖方向变更。
- 唯一偏差：删除 daemon `whoami` 命令入口（见上「决策说明」），是「删除由这些调用支撑的平台账号字段」的直接后果。

## 3. 自验证证据

命令 1：

    $ cargo fmt --check
    exit=0（无输出）

命令 2：

    $ cargo check --workspace --all-targets
    exit=0
        ...
        warning: `rustcode-daemon` (lib test) generated 7 warnings
        Finished `dev` profile [unoptimized + debuginfo] target(s) in 35.70s

7 条 warning 全部位于 `crates/rustcode-daemon/src/legacy_convert.rs`（unused mut），为基线既有、非本次引入；`commands.rs` 在 check 与 clippy 输出中命中 0 次。

命令 3：

    $ cargo test -j 1 -p rustcode-daemon --lib
    exit=0
        ...
        test tests::session_catalog_io_is_offloaded_and_single_flight ... ok

        test result: ok. 285 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.78s

基线 286 − 删除的 1 个用例 = 285，0 failed。

命令 4：

    $ grep -rn "rustcode_auth" crates/rustcode-daemon/src | wc -l
    1

剩余 1 处为 `crates/rustcode-daemon/src/lib.rs:5370` 的 `rustcode_auth::oauth::open_browser(&local_url)`（通用「打开浏览器」工具，非 OAuth，后续批次迁移，本次未动）。

附加验证（非清单要求）：`cargo clippy -p rustcode-daemon --all-targets` → exit=0，commands.rs 命中 0 条。

diff 统计：

    $ git diff --stat -- crates/rustcode-daemon/src/commands.rs
     crates/rustcode-daemon/src/commands.rs | 92 +---------------------------------
     1 file changed, 2 insertions(+), 90 deletions(-)

仅 1 个文件。

## 4. 验收标准对照

| AC | 结论 |
|---|---|
| 5 处 `rustcode_auth::` 使用点清零 | 满足：`get_stored_auth()` x3、`managed_login_available()` x2 全部随其宿主函数 / 用例删除；实测 grep 计数 1（仅 lib.rs:5370 open_browser） |
| 平台账号 / 登录态 / 套餐字段删除 | 满足：`Whoami.logged_in/username/name/email`、`Status.logged_in/username` 全删（声明在本文件） |
| 删失效测试、不削弱无关断言 | 满足：仅删 `neutral_status_omits_managed_login_line` 一条；其余 285 条用例及断言零改动 |
| 不改其它文件 | 满足：`git diff --stat` 仅 1 个文件；未触碰 lib.rs / Cargo.toml / i18n |
| 随时可编译 | 满足：每删一组即 `cargo check -p rustcode-daemon`（--all-targets），中间红状态已当场修完 |
| 前两条命令 exit 0 | 满足：`cargo fmt --check` exit=0，`cargo check --workspace --all-targets` exit=0 |

## 5. 遗留风险与后续项

**其它文件中的待处理项（本次未改，登记给后续批次）**

| file:line | 内容 | 建议负责人 |
|---|---|---|
| `crates/rustcode-daemon/src/lib.rs:5370` | `rustcode_auth::oauth::open_browser(&local_url)` —— 通用「打开浏览器」工具，非 OAuth；需迁移到 capabilities / config 后才能断开依赖 | 后续批次（迁移 open_browser） |
| `crates/rustcode-daemon/Cargo.toml:21` | `rustcode-auth = { path = "../rustcode-auth" }` 依赖，因 lib.rs:5370 仍需保留 | 后续批次（随 open_browser 迁移一并移除） |
| `crates/rustcode-tuix/src/event_loop/commands.rs:8651` | `assert!(!rustcode_auth::managed_login_available());`（TUI 侧测试） | 后续批次（TUI 清理），不在本 files_owned |
| `crates/rustcode-tuix/src/modals/onboarding_wizard.rs:338-339` | `managed_login_available()` 转发封装 | 后续批次（TUI 清理） |
| `crates/rustcode-tuix/src/commands.rs:35` | `MANAGED_ONLY_COMMANDS` 含 `login/logout/whoami/usage` | 后续批次（TUI 清理） |
| `crates/rustcode-cli/src/main.rs:389,4790` | `auth::managed_login_available()` 及 `neutral_build_hides_managed_login_subcommands` 用例 | 后续批次（CLI 清理） |

**风险**

1. daemon 的 `whoami` 命令入口被移除，调用方将收到 unknown-command 显式错误（而非静默空结果）。若产品期望 `whoami` 改为输出本地身份，需由 solution-architect 定义契约后另开任务，本任务不得自造语义。
2. `CommandResult::Status` 的 JSON 少了 `logged_in` / `username`；全仓 grep（.rs/.js/.ts/.html，排除 rustcode-auth 自身）确认无任何消费方，仅 `rustcode-tuix/src/event_loop/oauth_poll.rs:89` 有一条注释提及 `is_logged_in()`，非消费点。
3. i18n `Msg::StatusLoginLoggedIn` / `StatusLoginNotSignedIn` / `CmdWhoamiNotSignedIn*` 现由 TUI 侧继续使用（daemon 侧不再引用）；`rustcode-auth` crate 整体删除批次需做最终清理。
