---
kind: implementation
id: T-14
from: code-implementer
to: [code-reviewer]
feature: 2026-09-07-zh-docs-webui
status: done
decision: proceed
requires: [DESIGN-001, T-13]
files_owned:
  - crates/rustcode-daemon/src/lib.rs
  - .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/T-14-loopback-ipv6.md
architecture_constraints:
  touches_runtime_lifecycle: false
  touches_persistence: false
  touches_cross_crate_deps: false
created: 2026-09-08
---

# T-14 实现报告 · 修 `is_loopback_authority` 的 IPv6 回环误报

> 范围：按 PM 裁决**只做局部收敛**——不触碰 `is_loopback_authority` 的现有判定语义，
> 新增一个**仅服务于风险提示打印**的私有谓词 `is_loopback_bind_host`，并只把
> `run_server` 里 T-13 新加的提示分支切到该谓词。
> 本报告全部数字为本轮实测，无 TODO / 占位符。

---

## 1. 元信息

| 项 | 实测值 |
|---|---|
| 分支 | `dev` |
| 基线 / 改动前 `git rev-parse --short HEAD` | `3ee655e3`（**HEAD 仍等于基线**，改动全在未提交工作区） |
| `git status --porcelain \| wc -l` | 开工 **169** → 收工 **169**（本任务仅改 1 个已跟踪文件 + 新增 1 个报告；验证过程产生的 7 个 `core.*` 已清理，见 §4.7） |
| `git log --oneline -3 -- crates/rustcode-daemon/src/lib.rs` | `0e925aee` / `8e772dbf` / `c8b84e77` |
| 执行人 | `code-implementer`（T-14） |
| 改动文件数 | 1 个源码文件 + 1 个报告文件 |

```
$ git rev-parse --short HEAD; git status --porcelain | wc -l
3ee655e3
169
$ git log --oneline -3 -- crates/rustcode-daemon/src/lib.rs
0e925aee chore: optimize dev branch - fix clippy warnings, update repo metadata, enhance CI
8e772dbf Refactor and enhance various components in rustcode-tuix
c8b84e77 feat(i18n): platform-neutralization sweep -- inflight spinner zh-CN, webui remote-access de-vendoring
```

---

## 2. 缺陷复现证据

### 2.1 G5 报告引用

`05-test-report.md`（T-07 集成验收 G5）缺陷 **D-1** 登记：

- 现象：`is_loopback_authority("::1") == false`、`is_loopback_authority("::ffff:127.0.0.1") == false`。
- 根因：`"::1".split(':').next()` 得到**空串**，`matches!(host, ...)` 落空。
- 方向性：**过度告警（fail-safe）**，不阻塞 G5；严重度**低**。
- 归属：既有谓词缺陷；R1/T-13 的新路径继承。**PM 裁决需 architect 评估。**

### 2.2 本轮独立复现（/tmp 单文件 rustc 复刻，源码逐字复制 HEAD 版本）

`/tmp/t14-repro/repro.rs`（函数体与 `3ee655e3:crates/rustcode-daemon/src/lib.rs:1272` 逐字一致）：

```
$ cd /tmp/t14-repro && rustc -O -o repro repro.rs && ./repro; echo "exit=$?"
authority               current   expected
127.0.0.1                  true       true  ok
127.0.0.1:13456            true       true  ok
localhost                  true       true  ok
localhost:13456            true       true  ok
::1                       false       true  MISMATCH
[::1]                      true       true  ok
[::1]:13456                true       true  ok
::ffff:127.0.0.1          false       true  MISMATCH
0.0.0.0                   false      false  ok
::                        false      false  ok
192.168.1.7               false      false  ok
100.64.0.5                false      false  ok
exit=0
```

**2 处 MISMATCH**：`::1` 与 `::ffff:127.0.0.1`。`[::1]` / `[::1]:13456` 正确（走 `strip_prefix("[::1]")` 早返回分支），说明缺陷只影响**裸 IPv6 字面量**。

### 2.3 端到端复现（修复前，进程级）

把 `run_server` 的调用点临时回退为 `is_loopback_authority(&host)` 后重新构建并启动
（该回退仅为取证，已还原，见 §4.6）：

```
$ RUSTCODE_HOME=$(mktemp -d) timeout 8 ./target/debug/rustcode-daemon --host ::1 --port 13496 2>&1 | head -6
Idle timeout: 30 minutes
RustCode API server listening on http://::1:13496

[!] Bound to a non-loopback address: anyone who can reach it can get in with this token. Use only on a trusted network (no TLS).

API endpoints:
```

即：**绑定 IPv6 回环 `::1` 却打印「绑定了非回环地址」**——误报确认。

---

## 3. 改动 hunk

仅 2 处，均在 `crates/rustcode-daemon/src/lib.rs`。

### 3.1 新增私有谓词（工作区 `lib.rs:1281-1303`，紧跟 `is_loopback_authority` 之后）

改前：无（纯新增；`is_loopback_authority` 结束于 `lib.rs:1279`，其后直接是
`client_interactive_permission` 的文档注释）。

改后（新增 23 行，含 19 行 ASCII 文档注释 + 3 行函数 + 1 空行）：

```rust
/// Loopback test used **only** to decide whether the daemon prints the
/// non-loopback risk notice (`Msg::WebuiLanWarning` / `Msg::WebuiNonLoopbackWarning`)
/// in `run_server`. It must never be used for any authorization decision.
///
/// Why a separate predicate instead of fixing `is_loopback_authority` directly:
/// `is_loopback_authority` takes an authority, so a bare IPv6 literal such as
/// `::1` falls through its `split(':').next()` branch and yields an empty host
/// (i.e. it reports `false`). That is a real bug, but `is_loopback_authority` is
/// also consumed by `client_interactive_permission`, where `true` grants local
/// known clients the right to receive interactive approval prompts. Widening
/// that predicate therefore widens a security decision, which is an architect
/// call and out of scope here.
///
/// So this helper only adds the two bare IPv6 loopback spellings that the
/// warning path can actually receive from a driver (`--host ::1`,
/// `--host ::ffff:127.0.0.1`). It is deliberately narrow: no `IpAddr` parsing
/// and no broader matching, so the security surface stays exactly as it is.
/// Under-warning (missing a notice) is the fail-safe direction here, because
/// every non-public route stays token-protected regardless of the notice.
fn is_loopback_bind_host(host: &str) -> bool {
    is_loopback_authority(host) || host == "::1" || host.eq_ignore_ascii_case("::ffff:127.0.0.1")
}
```

> 注释块经 `grep -nP "[^\x00-\x7F]"` 与 emoji 区段正则校验，**纯 ASCII、无 emoji**。
> 函数体严格等于 PM 指定的 `is_loopback_authority(host) || host == "::1" || host.eq_ignore_ascii_case("::ffff:127.0.0.1")`；
> 单行形式由 `rustfmt` 折叠（`cargo fmt --check` exit 0）。

### 3.2 `run_server` 提示分支切换谓词（工作区 `lib.rs:6388`）

改前：

```rust
        // `rustcode daemon` 与独立二进制直连 `run_server`，不经 `ensure_server_and_open`，
        // 因此拿不到那里随访问 URL 一起打印的非回环风险提示。此处按同一判据
        //（`is_loopback_authority`）补发，使「绑定非回环」在任何 driver 下都有用户可见
        // 提示；回环绑定保持静默，避免每次启动刷噪音。
        if !is_loopback_authority(&host) {
```

改后：

```rust
        // `rustcode daemon` 与独立二进制直连 `run_server`，不经 `ensure_server_and_open`，
        // 因此拿不到那里随访问 URL 一起打印的非回环风险提示。此处按同一判据
        //（`is_loopback_bind_host`，即 `is_loopback_authority` 加上裸 IPv6 回环写法）
        // 补发，使「绑定非回环」在任何 driver 下都有用户可见
        // 提示；回环绑定保持静默，避免每次启动刷噪音。
        if !is_loopback_bind_host(&host) {
```

分支体（`if host == "0.0.0.0" || host == "::"` → `Msg::WebuiLanWarning`，否则
`Msg::WebuiNonLoopbackWarning`）**一字未改**。

### 3.3 未改动项（显式声明）

- `is_loopback_authority`（`lib.rs:1272-1279`）函数体：**零改动**（§5 证明）。
- `client_interactive_permission`（`lib.rs:1307-1317`）函数体：**零改动**（§5 证明）。
- `ensure_server_and_open` 内 `lib.rs:5393/5422` 与 `lib.rs:5697` 的既有
  `is_loopback_authority` 调用点：**未动**（基线既有行为，见 §6 遗留风险 RI-1）。
- `daemon/src/main.rs:21` `DEFAULT_HOST = "127.0.0.1"` 红线：**未动**。
- CLI `main.rs:1047` / `:1780` 的 `0.0.0.0` 默认值：**未动**。
- 无新增依赖、无 `IpAddr` 解析、无跨 crate 改动、无持久化/生命周期改动。

---

## 4. 验证命令的真实输出与退出码

### 4.1 `cargo fmt --check`

```
$ cargo fmt --check > /tmp/t14-fmt.txt 2>&1; echo "exit=$?"; cat /tmp/t14-fmt.txt
exit=0
```

（首次执行时 exit=1，唯一 diff 是新增函数的三行 `||` 被 rustfmt 折叠为一行；
按 rustfmt 建议改后重跑为 exit=0。全仓其它文件**无**格式 diff。）

### 4.2 `cargo clippy -p rustcode-daemon --all-targets`

```
$ cargo clippy -p rustcode-daemon --all-targets > /tmp/t14-clippy.txt 2>&1; echo "exit=$?"
exit=0
$ sed -r 's/\x1B\[[0-9;]*[mK]//g' /tmp/t14-clippy.txt | tail -n 20
     |
     = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.93.0/index.html#too_many_arguments

warning: `rustcode-daemon` (lib) generated 6 warnings
warning: field assignment outside of initializer for an instance created with Default::default()
    --> crates/rustcode-daemon/src/live_api.rs:2681:9
     |
2681 |         config.default_provider = "default-prov".to_string();
     |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
     |
note: consider initializing the variable with `rustcode_config::Config { default_provider: "default-prov".to_string(), ..Default::default() }` and removing relevant reassignments
    --> crates/rustcode-daemon/src/live_api.rs:2680:9
     |
2680 |         let mut config = Config::default();
     |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
     = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.93.0/index.html#field_reassign_with_default
     = note: `#[warn(clippy::field_reassign_with_default)]` on by default

warning: `rustcode-daemon` (lib test) generated 14 warnings (6 duplicates) (run `cargo clippy --fix --lib -p rustcode-daemon --tests` to apply 7 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 22.65s
```

**退出码 0，无 error。**（最终态复跑：`cargo clippy -p rustcode-daemon --all-targets`
`clippy_exit=0`，`grep -cE "^error"` = **0**。） 警告均为既有（`rustcode-config` / `rustcode-capabilities` /
`live_api.rs:2680-2681` 等）。我新增代码**零警告**：

```
$ sed -r 's/\x1B\[[0-9;]*[mK]//g' /tmp/t14-clippy.txt | grep -oE "crates/rustcode-daemon/src/lib\.rs:[0-9]+" | sort -u
crates/rustcode-daemon/src/lib.rs:4544
crates/rustcode-daemon/src/lib.rs:4808
```

两处均远离本次改动区（`1281-1303` / `6388`）。

### 4.3 `cargo test -p rustcode-daemon`

```
$ cargo test -p rustcode-daemon > /tmp/t14-test-final.txt 2>&1; echo "test_exit=$?"
test_exit=0
$ grep -E "^     Running|^test result" /tmp/t14-test-final.txt
     Running unittests src/lib.rs (target/debug/deps/rustcode_daemon-c00f6f792f2e5f83)
test result: ok. 307 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.37s
     Running unittests src/main.rs (target/debug/deps/rustcode_daemon-76c511278bb5c1fe)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/daemon_token_auth.rs (target/debug/deps/daemon_token_auth-c24f62f3bbfc6376)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.56s
     Running tests/default_host_lock.rs (target/debug/deps/default_host_lock-41211267cc16cc50)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/legacy_turn_boundary_repair.rs (target/debug/deps/legacy_turn_boundary_repair-59b95b13a645a98e)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s   # Doc-tests
$ echo "TOTAL_PASSED=$(grep -oE '^test result: ok\. [0-9]+ passed' /tmp/t14-test-final.txt | grep -oE '[0-9]+' | paste -sd+ | bc)"
TOTAL_PASSED=316
$ grep -c 'FAILED\|panicked' /tmp/t14-test-final.txt
0
```

**316 passed / 0 failed / 0 ignored，与基线计数一致。**
点名用例均在：

```
$ grep -n "known_clients_interactive_on_loopback_or_token" /tmp/t14-test-final.txt
124:test channel_mode_tests::known_clients_interactive_on_loopback_or_token ... ok
```

`daemon_token_auth`（1 passed）与 `default_host_lock`（1 passed）两个集成目标均通过。

### 4.4 `cargo build -p rustcode-daemon -p rustcode`

```
$ cargo build -p rustcode-daemon -p rustcode > /tmp/t14-build.txt 2>&1; echo "exit=$?"
exit=0
$ tail -5 /tmp/t14-build.txt
warning: `rustcode-capabilities` (lib) generated 1 warning
   Compiling rustcode-daemon v5.0.9 (/workspace/RustCode/crates/rustcode-daemon)
   Compiling rustcode-tuix v5.0.9 (/workspace/RustCode/crates/rustcode-tuix)
   Compiling rustcode v5.0.9 (/workspace/RustCode/crates/rustcode-cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 22.70s
```

### 4.5 行为验证：五个 host 各起进程

> 说明：`rustcode daemon` CLI 子命令**没有** `--host` 参数（已知 D-5，
> `crates/rustcode-cli/src/main.rs:467-472` 只有 `port` / `idle_timeout`），
> 因此 `0.0.0.0` 用 CLI 默认路径、其余 host 用**独立二进制 `rustcode-daemon`**
> （其 `parse_daemon_args()` 在 `main.rs:33-43` 支持 `--host`）端到端验证。
> 两者都走同一个 `run_server`，覆盖到同一个改动分支。

**A. `rustcode daemon`（CLI 默认 0.0.0.0）→ 应含 LAN 提示**

```
$ RUSTCODE_HOME=$(mktemp -d) timeout 8 ./target/debug/rustcode daemon --port 13491 2>&1 | head -8
Starting RustCode daemon on port 13491...
Press Ctrl+C to stop.
Idle timeout: 30 minutes
RustCode API server listening on http://0.0.0.0:13491

[!] Primary address is a LAN IP; only devices on the same network can reach it. For public access use a tunnel (e.g. cloudflared / Tailscale). There is no TLS, so anyone who can reach it can get in with the token.

API endpoints:
```

✅ 含 `Msg::WebuiLanWarning`（未回归，T-13 行为保留）。

**B. `rustcode-daemon --port 13492`（默认 127.0.0.1）→ 不应含提示**

```
$ RUSTCODE_HOME=$(mktemp -d) timeout 8 ./target/debug/rustcode-daemon --port 13492 2>&1 | head -8
Idle timeout: 30 minutes
RustCode API server listening on http://127.0.0.1:13492

API endpoints:
  GET    /health                                  - Health check
  GET    /project                                 - Get current working directory
  POST   /cd                                      - Change working directory (like /cd command)
  GET    /projects                                - List historical projects
```

✅ 静默。

**C. `rustcode-daemon --host ::1 --port 13493` → 修复后不应含提示**

```
$ RUSTCODE_HOME=$(mktemp -d) timeout 8 ./target/debug/rustcode-daemon --host ::1 --port 13493 2>&1 | head -8
Idle timeout: 30 minutes
RustCode API server listening on http://::1:13493

API endpoints:
  GET    /health                                  - Health check
  GET    /project                                 - Get current working directory
  POST   /cd                                      - Change working directory (like /cd command)
  GET    /projects                                - List historical projects
```

✅ 误报消除（对照 §2.3 修复前同样的命令行会打印 `[!] Bound to a non-loopback address: ...`）。

**D. `rustcode-daemon --host ::ffff:127.0.0.1 --port 13494` → 不应含提示**

```
$ RUSTCODE_HOME=$(mktemp -d) timeout 8 ./target/debug/rustcode-daemon --host ::ffff:127.0.0.1 --port 13494 2>&1 | head -8
Idle timeout: 30 minutes
RustCode API server listening on http://::ffff:127.0.0.1:13494

API endpoints:
  GET    /health                                  - Health check
  GET    /project                                 - Get current working directory
  POST   /cd                                      - Change working directory (like /cd command)
  GET    /projects                                - List historical projects
```

✅ 静默。

**E. 控制组 `rustcode-daemon --host 192.168.1.7 --port 13495` → 必须仍告警**

```
$ RUSTCODE_HOME=$(mktemp -d) timeout 8 ./target/debug/rustcode-daemon --host 192.168.1.7 --port 13495 2>&1 | head -8
Idle timeout: 30 minutes
RustCode API server listening on http://192.168.1.7:13495

[!] Bound to a non-loopback address: anyone who can reach it can get in with this token. Use only on a trusted network (no TLS).

API endpoints:
  GET    /health                                  - Health check
  GET    /project                                 - Get current working directory
  POST   /cd                                      - Change working directory (like /cd command)
  GET    /projects                                - List historical projects
```

✅ 真实非回环仍告警——**补集收敛没有把告警面整体关掉**。

### 4.6 取证用的临时回退与还原（可复现）

为取得 §2.3 的「修复前」端到端证据，仅把调用点一行 sed 回退、构建、运行，再整文件还原：

```
$ cp crates/rustcode-daemon/src/lib.rs /tmp/t14-lib-fixed.rs && md5sum /tmp/t14-lib-fixed.rs
c5ba16f91ad6050a95b1063050147bf1  /tmp/t14-lib-fixed.rs
$ sed -i 's/if !is_loopback_bind_host(&host) {/if !is_loopback_authority(\&host) {/' crates/rustcode-daemon/src/lib.rs
$ cargo build -p rustcode-daemon   # build_exit=0
$ RUSTCODE_HOME=$(mktemp -d) timeout 8 ./target/debug/rustcode-daemon --host ::1 --port 13496 2>&1 | head -6
   -> 打印 "[!] Bound to a non-loopback address: ..."（见 §2.3）
$ cp /tmp/t14-lib-fixed.rs crates/rustcode-daemon/src/lib.rs && md5sum crates/rustcode-daemon/src/lib.rs
c5ba16f91ad6050a95b1063050147bf1  crates/rustcode-daemon/src/lib.rs   # 与回退前逐字节一致
$ cargo build -p rustcode-daemon -p rustcode   # build_exit=0
$ RUSTCODE_HOME=$(mktemp -d) timeout 8 ./target/debug/rustcode-daemon --host ::1 --port 13497 2>&1 | head -6
Idle timeout: 30 minutes
RustCode API server listening on http://::1:13497

API endpoints:
  GET    /health                                  - Health check
  GET    /project                                 - Get current working directory
```

§4.3 的 316 passed 是在**还原后**（md5 `c5ba16f9...`，即最终态）跑的。

### 4.7 单文件 rustc 复刻：新谓词行为（可独立复现）

`/tmp/t14-repro/after.rs` —— 两个谓词均从当前 `lib.rs` **逐字复制**：

```
$ cd /tmp/t14-repro && rustc -O -o after after.rs && ./after; echo "exit=$?"
host                 is_loopback_authority  is_loopback_bind_host      warn?
127.0.0.1                          true                   true      false  ok
127.0.0.1:13456                    true                   true      false  ok
localhost                          true                   true      false  ok
localhost:13456                    true                   true      false  ok
[::1]                              true                   true      false  ok
[::1]:13456                        true                   true      false  ok
::1                               false                   true      false  ok
::ffff:127.0.0.1                  false                   true      false  ok
::FFFF:127.0.0.1                  false                   true      false  ok
0.0.0.0                           false                  false       true  ok
::                                false                  false       true  ok
192.168.1.7                       false                  false       true  ok
100.64.0.5                        false                  false       true  ok

mismatches = 0
exit=0
```

要点：

- `::1` → 新谓词 **true**（旧谓词 false）；
- `::ffff:127.0.0.1` 与大写 `::FFFF:127.0.0.1` → 新谓词 **true**（`eq_ignore_ascii_case` 生效）；
- `is_loopback_authority` 列与 §2.2 完全一致 —— **旧谓词输出一行未变**；
- `0.0.0.0` / `::` / `192.168.1.7` / `100.64.0.5` → 新谓词仍 false，告警面未缩小。

### 4.8 工作区卫生

`timeout 8` 杀进程时 daemon 会落 `core.*`（7 个，共约 126 MB，`file` 确认来自
`./target/debug/rustcode daemon --port ...`）。这是**既有信号行为**，与本次改动无关，
但属我验证过程产生的污染，已清理：

```
$ rm -f core.433776 core.434506 core.434515 core.435050 core.435058 core.437679 core.440146
$ git status --porcelain | wc -l
169
$ git status --porcelain | grep -c 'core\.'
0
```

未执行任何 `git add` / `git commit`。

---

## 5. 回归证据：`is_loopback_authority` / `client_interactive_permission` 零改动

### 5.1 逐函数抽取 + diff + md5（HEAD vs 工作区）

```
$ git show 3ee655e3:crates/rustcode-daemon/src/lib.rs > /tmp/t14-head-lib.rs
$ extract 'fn is_loopback_authority' /tmp/t14-head-lib.rs        > /tmp/t14-fn-isloop-head.txt
$ extract 'fn is_loopback_authority' crates/rustcode-daemon/src/lib.rs > /tmp/t14-fn-isloop-work.txt
$ diff /tmp/t14-fn-isloop-head.txt /tmp/t14-fn-isloop-work.txt && echo "IDENTICAL"
IDENTICAL (exit=0)

$ extract 'fn client_interactive_permission' /tmp/t14-head-lib.rs        > /tmp/t14-fn-cip-head.txt
$ extract 'fn client_interactive_permission' crates/rustcode-daemon/src/lib.rs > /tmp/t14-fn-cip-work.txt
$ diff /tmp/t14-fn-cip-head.txt /tmp/t14-fn-cip-work.txt && echo "IDENTICAL"
IDENTICAL (exit=0)

$ md5sum /tmp/t14-fn-isloop-head.txt /tmp/t14-fn-isloop-work.txt /tmp/t14-fn-cip-head.txt /tmp/t14-fn-cip-work.txt
80e8814c6e2a821b61306f929b2c8eb0  /tmp/t14-fn-isloop-head.txt
80e8814c6e2a821b61306f929b2c8eb0  /tmp/t14-fn-isloop-work.txt
3dd9c125e13dd8d5cf3b7b5494addb0b  /tmp/t14-fn-cip-head.txt
3dd9c125e13dd8d5cf3b7b5494addb0b  /tmp/t14-fn-cip-work.txt
```

**两个函数与基线逐字节相同（md5 相同）。**

### 5.2 hunk 层面：改动是纯插入，落在两函数之间

```
$ git diff 3ee655e3 -U0 -- crates/rustcode-daemon/src/lib.rs | grep -E "^@@"
@@ -1280,0 +1281,23 @@ fn is_loopback_authority(authority: &str) -> bool {
@@ -5275 +5298,2 @@ pub const WEBUI_DEFAULT_PORT: u16 = rustcode_config::distribution::WEBUI_PORT;
@@ -6034 +6058,2 @@ pub struct ServerOpts {
@@ -6333,9 +6358,12 @@ pub async fn run_server(opts: ServerOpts) -> anyhow::Result<()> {
@@ -6343,5 +6370,0 @@ pub async fn run_server(opts: ServerOpts) -> anyhow::Result<()> {
@@ -6359,0 +6383,12 @@ pub async fn run_server(opts: ServerOpts) -> anyhow::Result<()> {
```

- `@@ -1280,0 +1281,23 @@`：**纯插入**（`-1280,0` = 基线侧删除 0 行），
  且 hunk header 显示上下文函数就是 `fn is_loopback_authority` —— 插入点紧接其闭合括号之后，
  **没有改动该函数任何一行**。
- 其它 hunk 均属 T-13（默认绑定 0.0.0.0 + 横幅改造），非本任务。
- 无任何 hunk 覆盖 `client_interactive_permission` 所在行区间。

### 5.3 diff 中出现的 `is_loopback_authority` 字样全部只在注释/新函数体内

```
$ git diff 3ee655e3 -- crates/rustcode-daemon/src/lib.rs | grep -E "^[+-]" | grep -E "is_loopback_authority|client_interactive_permission"
+/// Why a separate predicate instead of fixing `is_loopback_authority` directly:
+/// `is_loopback_authority` takes an authority, so a bare IPv6 literal such as
+/// `::1` falls through its `split(':').next()` branch and yields an empty host
+/// (i.e. it reports `false`). That is a real bug, but `is_loopback_authority` is
+/// also consumed by `client_interactive_permission`, where `true` grants local
+    is_loopback_authority(host) || host == "::1" || host.eq_ignore_ascii_case("::ffff:127.0.0.1")
+        //（`is_loopback_bind_host`，即 `is_loopback_authority` 加上裸 IPv6 回环写法）
```

即：5 行文档注释 + 1 行新函数体（**调用**旧谓词，非修改）+ 1 行中文注释。
**没有任何 `-`（删除）行**触及这两个函数。

### 5.4 行为侧交叉验证

`cargo test -p rustcode-daemon` 中
`channel_mode_tests::known_clients_interactive_on_loopback_or_token ... ok`（§4.3）——
该用例正是锁定 `client_interactive_permission` 的 loopback 分支，断言无净减少、行为不变。

---

## 6. 验收标准对照

| 验收项 | 要求 | 如何满足 | 证据 |
|---|---|---|---|
| AC-1 | 不修改 `is_loopback_authority` 函数体现有判定语义 | 函数体零改动，md5 与基线相同 | §5.1 / §5.2 |
| AC-2 | 新增私有函数，逻辑恰为 `is_loopback_authority(host) \|\| host == "::1" \|\| host.eq_ignore_ascii_case("::ffff:127.0.0.1")`，只允许这三个补集 | 严格三选或；无 `IpAddr` 解析、无更宽匹配 | §3.1 / §4.7 |
| AC-3 | `run_server` 提示分支改用新谓词 | `lib.rs:6388` 由 `!is_loopback_authority(&host)` 改为 `!is_loopback_bind_host(&host)` | §3.2 |
| AC-4 | 新函数上方注释说明：为何不改 `is_loopback_authority`（它服务 `client_interactive_permission` 的安全判定）、本函数仅用于是否打印风险提示 | 19 行文档注释明确写出两点 | §3.1 |
| AC-5 | 注释与代码禁 Unicode Emoji、ASCII | `grep -nP "[^\x00-\x7F]"` 与 emoji 正则均 0 命中 | §3.1 |
| AC-6 | 只改 `files_owned` 的 1 个文件 + 新增 1 个报告 | `git status --porcelain` 中 daemon 侧仅 `M crates/rustcode-daemon/src/lib.rs` | §1 / §4.8 |
| AC-7 | 不 `git add` / `git commit`；不改 md 汉化正文 | 全程未执行；未改任何 md 正文 | §1 / §4.8 |
| AC-8 | `daemon/src/main.rs:21` `DEFAULT_HOST="127.0.0.1"` 红线不动 | 未触碰该文件 | §3.3 |
| AC-9 | CLI `:1047`/`:1780` 的 `0.0.0.0` 不回退 | 未触碰 CLI 源码；行为验证 A 仍绑定 0.0.0.0 | §3.3 / §4.5-A |
| AC-10 | `cargo fmt --check` 干净 | exit=0 | §4.1 |
| AC-11 | `cargo clippy -p rustcode-daemon --all-targets` 通过且无新增警告 | exit=0；警告位置 4544/4808，均不在改动区 | §4.2 |
| AC-12 | `cargo test -p rustcode-daemon` 仍 316 passed / 0 failed（含 `default_host_lock`、`daemon_token_auth`、`channel_mode_tests`） | 316 passed / 0 failed，三个点名目标均 ok | §4.3 |
| AC-13 | `cargo build -p rustcode-daemon -p rustcode` 通过 | exit=0 | §4.4 |
| AC-14 | 三个 host 行为验证（0.0.0.0 含 LAN 提示、127.0.0.1 不含、::1 不含）+ `::1` 在新判定下 loopback=true 的可复现证据 | A/B/C/D/E 五组进程级实测 + `/tmp` rustc 复刻 + 修复前/后对照 | §4.5 / §4.6 / §4.7 |
| AC-15 | 回归证据：两个安全相关函数零改动 | diff 空 + md5 相同 + hunk 纯插入 | §5 |

**所有 AC 满足，无未声明偏差。**

### 契约符合性（`01-design.md` 对照）

| 契约点 | 实现一致性 |
|---|---|
| 绑定地址由 driver 决定，`run_server` 不内置默认值 | 未改；仅改提示谓词 |
| 非回环风险提示复用 `Msg::WebuiLanWarning` / `Msg::WebuiNonLoopbackWarning` | 未改；分支体一字未动 |
| 回环绑定静默、非回环可见告警 | 强化：`::1` / `::ffff:127.0.0.1` 由误告警改为静默 |
| 安全判定收敛于 `client_interactive_permission` | 未触碰，其输入谓词输出逐字节不变 |
| 依赖方向 / core-free / 无 bridge / 无 fallback | 未引入任何新依赖或新抽象 |

**偏差：无。**

---

## 7. 回滚方案

### 7.1 回滚步骤（改动完全自包含）

1. 删除 `crates/rustcode-daemon/src/lib.rs:1281-1303` 的
   `is_loopback_bind_host` 及其文档注释块（纯新增，删除后与基线该处完全一致）。
2. 把 `lib.rs:6388` 的 `if !is_loopback_bind_host(&host) {` 改回
   `if !is_loopback_authority(&host) {`，并把上面第 7 行中文注释里的
   `is_loopback_bind_host`，即 `is_loopback_authority` 加上裸 IPv6 回环写法
   改回 `is_loopback_authority`。
3. （可选）删除本报告文件。

因本任务**未新增任何测试、未改任何其它文件、未引入依赖、未改持久化或状态机**，
回滚后 `crates/rustcode-daemon/src/lib.rs` 回到 T-14 之前的确切内容（即 T-13 收尾态）。
可用 `md5sum` 与 `/tmp/t14-lib-fixed.rs`（`c5ba16f91ad6050a95b1063050147bf1`）比对确认还原/回滚是否正确。

磁盘 / 配置 / session / token 全部不受影响：**零数据面改动**，回滚无迁移成本。

### 7.2 回滚后后果

- 回到 T-13 收尾态：`--host ::1` 与 `--host ::ffff:127.0.0.1` 会**继续误报**
  「绑定了非回环地址」风险提示（§2.3 实证）。
- 方向是**过度告警（fail-safe）**：不会「本该告警却静默」，安全面上偏保守；
  所有非公开路由仍由 token 保护，故**不阻塞发布**。
- `client_interactive_permission` 的权限面**不因回滚而变化**（本任务与回滚都不触碰它）。
- 已通过的全部验证（fmt / clippy / 316 tests / build）保持通过，因为回滚只是撤销新增代码。

---

## 8. 遗留风险与后续项

| 编号 | 风险 / 后续项 | 说明 | 建议负责人 |
|---|---|---|---|
| RI-1 | `ensure_server_and_open` 的 `lib.rs:5393`（`open_host` 选择）与 `lib.rs:5422`（非回环提示）仍用 `is_loopback_authority` | 这两处是**基线既有**行为，不在 T-14 `files_owned` 收敛范围内。后果：`rustcode webui --host ::1` 仍会（a）误报非回环提示，（b）生成 `http://::1:PORT/?token=...` 这种**缺少方括号的非法 URL**（`open_host` 取到 `bound_host` 原值）。**本任务未加重也未修复。** | architect（与 D-1 合并评估） |
| RI-2 | `is_loopback_authority` 本体缺陷未修 | PM 明令本任务不改。G5 D-1 建议的 `normalize_authority` + `IpAddr` 解析方案连同 `client_interactive_permission` 权限面影响一并裁决 | architect（PM 已裁决需其评估） |
| RI-3 | 谓词补集是白名单式、非解析式 | 只覆盖 `::1` 与 `::ffff:127.0.0.1` 两种写法；`0:0:0:0:0:0:0:1`、`127.0.0.2`（同属 127/8）等仍判为非回环 → **少告警**。这是 fail-safe 方向（漏提示不漏保护），且符合 PM「不要引入 `IpAddr` 解析或更宽匹配」的硬约束 | architect（如需全覆盖再议） |
| RI-4 | 无新增 `#[cfg(test)]` 单元用例 | 按派单，测试归 `test-engineer`；本任务用 `/tmp` rustc 复刻 + 进程级实测给出证据。建议后续为 `is_loopback_bind_host` 补 `::1` / `::FFFF:127.0.0.1` / `0.0.0.0` / `192.168.1.7` 四条表驱动用例 | test-engineer |
| RI-5 | `timeout` 终止 daemon 会落 `core.*` | 验证过程中产生 7 个 core（约 126 MB），已清理至 `git status` 恢复 169。属既有信号处理行为，与本次改动无关 | 如需，另开任务查 SIGTERM 处理 |

---

## 9. 结论

**`status: done`，`decision: proceed`。**

- 缺陷已按 PM 裁决**局部收敛**修复：`run_server` 的风险提示分支不再对 IPv6 回环误报，
  `is_loopback_authority` 与 `client_interactive_permission` **逐字节零改动**（md5 佐证）。
- `cargo fmt --check` exit 0；`cargo clippy -p rustcode-daemon --all-targets` exit 0 且无新增警告；
  `cargo test -p rustcode-daemon` **316 passed / 0 failed**（`default_host_lock`、
  `daemon_token_auth`、`channel_mode_tests::known_clients_interactive_on_loopback_or_token` 均 ok）；
  `cargo build -p rustcode-daemon -p rustcode` exit 0。
- 行为：0.0.0.0 → LAN 提示（保留）；127.0.0.1 → 静默；**::1 → 静默（修复前会误报，已取证对照）**；
  `::ffff:127.0.0.1` → 静默；192.168.1.7 → 仍告警（控制组）。
- 改动文件数：**1**（`crates/rustcode-daemon/src/lib.rs`）+ 新增 1 个报告；工作区条目数 169 → 169。
