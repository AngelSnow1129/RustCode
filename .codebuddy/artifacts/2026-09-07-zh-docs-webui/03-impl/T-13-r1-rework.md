---
kind: implementation
id: T-13-r1-rework
from: code-implementer
to: [code-reviewer]
feature: 2026-09-07-zh-docs-webui
status: done
decision: proceed
requires: [T-03, T-04, T-06, Q2, Q3]
files_owned:
  - crates/rustcode-daemon/src/lib.rs
  - scripts/check-zh-docs.py
  - crates/rustcode-daemon/tests/default_host_lock.rs
  - crates/rustcode-config/src/i18n/messages.rs
  - .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/T-13-r1-rework.md
architecture_constraints:
  touches_runtime_lifecycle: false
  touches_persistence: false
  touches_cross_crate_deps: false
created: 2026-09-09
---

# T-13 R1 返工收尾报告

本轮为 R1 返工收尾：前一轮 `rework-r1` 已把代码/脚本改动全部落盘但未产出报告。
本报告对 4 个 `files_owned` 的改动逐 hunk 核实，真实执行全部验证命令，并补齐结论。
**本轮未新增任何代码改动**——核实结论为「改动完整且正确」，无需就地修复。

---

## 0. 元信息

| 项 | 实测值 |
| --- | --- |
| 基线 `ZH_BASE` | `3ee655e3` |
| `git rev-parse --short HEAD` | `3ee655e3`（HEAD == 基线，全部改动在未提交工作区） |
| 分支 | `dev` |
| `git status --porcelain \| wc -l` | **169**（开工前与收工后一致，本轮未新增/删除文件） |

### 0.1 开工前置：`git log --oneline -3 -- <file>`

```
=== crates/rustcode-daemon/src/lib.rs ===
0e925aee chore: optimize dev branch - fix clippy warnings, update repo metadata, enhance CI
8e772dbf Refactor and enhance various components in rustcode-tuix
c8b84e77 feat(i18n): platform-neutralization sweep — inflight spinner zh-CN, webui remote-access de-vendoring

=== scripts/check-zh-docs.py ===
(空 —— 未跟踪新文件，无历史)

=== crates/rustcode-daemon/tests/default_host_lock.rs ===
(空 —— 未跟踪新文件，无历史)

=== crates/rustcode-config/src/i18n/messages.rs ===
00b99d6c feat(tuix): auto-discover models on provider save
783d48e4 chore(codingplan): clear legacy core-path comments, drop hidden CLI alias, archive stale plans
8e772dbf Refactor and enhance various components in rustcode-tuix
```

### 0.2 并发扰动记录

全程（约 4 分钟 cargo 窗口，跨 `clippy` / `test` × 3 / `build` × 2 / `check --workspace`）
**未出现一次 cargo 锁等待或 `Blocking waiting for file lock`**；无 `test-engineer` 并发扰动，
所有退出码均为首次执行结果，无重试。

### 0.3 硬约束自检

| 约束 | 结果 |
| --- | --- |
| 禁 Unicode Emoji | 通过。对 4 个 files_owned 全量扫描：`emoji hits = 1`，唯一命中 `messages.rs:3216`（`✗` U+2717，doc 注释内）。经 `git show 3ee655e3:.../messages.rs \| sed -n '3216p'` 确认为**基线既存**，不在本轮 diff 内，未触碰。 |
| 注释/日志 ASCII | 通过。本轮新增注释为 zh-CN 正文 + ASCII 标点，无全角破折号以外的新增非 ASCII 符号混入代码路径。 |
| `crates/rustcode-daemon/src/main.rs:21` `DEFAULT_HOST = "127.0.0.1"` 红线 | **未改**。`sed -n '21p'` 实测输出 `    const DEFAULT_HOST: &str = "127.0.0.1";` |
| CLI `0.0.0.0` 裁决 Q2-A 不回退 | **未回退**。`crates/rustcode-cli/src/main.rs:1047` `#[arg(long, default_value = "0.0.0.0")]`、`:1780` `host: "0.0.0.0".to_string()` 均在位 |
| 只改 files_owned | 通过。收工 `git status --porcelain \| wc -l` 仍为 169，与开工一致 |
| 未 `git add` / `git commit` | 通过。全程未执行任何 git 写操作（仅 `rev-parse` / `status` / `log` / `diff` / `show` 只读命令） |
| 未改任何 md 汉化正文 | 通过。本轮 4 个 files_owned 中无 md 文件 |

---

## 1. 逐文件逐 hunk 改动核实表

核实方式：`git diff 3ee655e3 -- <file>`（已跟踪）+ 直接读取（未跟踪）。**下表 8 项全部核实为「属实」。**

### 1.1 `crates/rustcode-daemon/src/lib.rs`

| 编号 | 文件:行 | 改前 | 改后 | 属实 |
| --- | --- | --- | --- | --- |
| MINOR-1 | `:5275` | ``/// `host` 为绑定地址（默认 `127.0.0.1`；`0.0.0.0` 暴露到局域网/外网）。`` | ``/// `host` 为绑定地址（本函数无默认值，由调用方决定：CLI `rustcode webui` / `rustcode daemon` 默认 `0.0.0.0`；TUI `/webui` 默认 `127.0.0.1`，见设计 O-1。`0.0.0.0` 暴露到局域网/外网）。`` | 是 |
| MAJOR-1 | `:6035`（原 `:6034`） | `/// Bind host (e.g. `127.0.0.1`). Non-loopback hosts emit a security warning.` | `/// Bind host (e.g. `0.0.0.0` / `127.0.0.1`). Decided by the driver; this function does not warn on non-loopback binds.` | 是 |
| MAJOR-3 | `:6332-6350` | 旧 loopback 注释块（引 PR #82 / `tianchang fix(daemon): harden daemon chat access`）+ 实际发射 `Msg::DaemonWarnNonLoopback` 的 `if host != "127.0.0.1" && ...` 代码块 | 注释块改写为「绑定地址由 driver 决定、默认 `0.0.0.0`、非公开路由均有 token 保护但无 TLS、旧横幅因每次启动必打印而移除、风险提示改由 `WebuiLanWarning`/`WebuiNonLoopbackWarning` 承担」；原发射代码块**已删除** | 是 |
| MAJOR-3 | `:6357-6374` | （无） | 在 `run_server` 非 quiet 分支、`println!(Msg::DaemonListening)` **之后**、空行与 `Msg::DaemonApiEndpoints` **之前**，按 `is_loopback_authority(&host)` 补发：`0.0.0.0`/`::` → `Msg::WebuiLanWarning`，其余非回环 → `Msg::WebuiNonLoopbackWarning` | 是 |

行号偏移说明：任务单记为 `:6034`，实测为 `:6035`。原因同文件上方 hunk（MINOR-1 处 `+1` 行）导致后文整体下移 1 行，**非改动缺失**。

补发点位置核对（实测 `sed -n '6357,6374p'` 语义，来自 diff）：

```rust
    if !quiet {
        println!("{}", t(Msg::DaemonListening { addr: &addr }));
+       // `rustcode daemon` 与独立二进制直连 `run_server`，不经 `ensure_server_and_open`，
+       // 因此拿不到那里随访问 URL 一起打印的非回环风险提示。此处按同一判据
+       //（`is_loopback_authority`）补发，使「绑定非回环」在任何 driver 下都有用户可见
+       // 提示；回环绑定保持静默，避免每次启动刷噪音。
+       if !is_loopback_authority(&host) {
+           if host == "0.0.0.0" || host == "::" {
+               println!("{}", t(Msg::WebuiLanWarning));
+           } else {
+               println!("{}", t(Msg::WebuiNonLoopbackWarning));
+           }
+       }
        println!();
        println!("{}", t(Msg::DaemonApiEndpoints));
```

判据一致性核对：`ensure_server_and_open` 中 `:5357` `let is_wildcard = bound_host == "0.0.0.0" || bound_host == "::";`，
`:5399-5407` 用 `!is_loopback_authority(&bound_host)` + `is_wildcard` 做同样二选一。
新增分支与既有分支**判据逐字一致**，无第二套判定逻辑（未引入第二状态机）。

### 1.2 `scripts/check-zh-docs.py`（未跟踪新文件，1473 行）

| 编号 | 文件:行 | 改前 | 改后 | 属实 |
| --- | --- | --- | --- | --- |
| MAJOR-2 | `:1445-1449` | `if not getattr(args, "en_ratio", None):` 形式的真值判空 | `if getattr(args, "en_ratio", None) is None:` + 3 行注释说明「`not 0.0` 为真会把最严阈值放宽成 0.05，与 fail-closed 相反」 | 是 |
| MAJOR-2 | `:1462-1466` | （无） | `if args.en_ratio < 0: raise EnvError("--en-ratio 必须 >= 0（0 表示不允许任何纯英文行）；实际取值 %s")` | 是 |
| MINOR-5 | `:1278-1296` | （无） | `cmd_gate` 开头计算 `untracked`（`git ls-files --others --exclude-standard -- "*.md"`），非空时打印「gate: 未跟踪 md 已排除：N 个…」及前 5 个路径。**不改分母、不改退出码** | 是 |
| NIT-1 | `:7`、`:1002`、`:1408` | 无标注 | 模块 docstring 用法行、`cmd_hostscan` docstring、`hostscan` 子命令 `help=` 三处均标注「清单产出器，非门禁，恒返回 0」 | 是 |
| NIT-2 | `:1461`（`main`）、`:177-192`（`verify_base_commit` 定义） | 无校验 | `main` 中 `args.base = resolve_base(...)` 之后、`args.func(args)` 之前调用 `verify_base_commit(find_root(), args.base)`，内部执行 `git rev-parse --verify --quiet <base>^{commit}`，失败抛 `EnvError` → `main` 捕获返回 2 | 是 |

NIT-2 覆盖范围核对：`verify_base_commit` 在 `main()` 中调用，位于所有子命令分派之前，
因此 `gate` 与 `check` 两个入口**均**经过该校验（无需在 `cmd_gate`/`cmd_check` 内重复调用）。
`resolve_base` 先于校验执行（`resolve_base` → `verify_base_commit` 顺序正确，校验的是最终生效值）。

### 1.3 `crates/rustcode-daemon/tests/default_host_lock.rs`（未跟踪新文件，49 行）

| 编号 | 文件:行 | 改前 | 改后 | 属实 |
| --- | --- | --- | --- | --- |
| MINOR-2 | `:3-13` | 立论未区分「有 token」与「有审批交互方」 | 立论改为：独立二进制虽 `enforce_token=true`（传入 `webui_tokens: Some(token_store)`，token 落盘 `~/.rustcode/daemon-<port>.json`，本地进程可读），但由 IDE **无人值守**拉起、**无审批交互方**，故默认仍须回环 | 是 |

**断言体零改动核对**：文件共 49 行，唯一 `#[test]` 为 `standalone_daemon_default_host_stays_loopback`，
断言 `declaration.contains(&format!("\"{EXPECTED_DEFAULT_HOST}\""))`，`EXPECTED_DEFAULT_HOST = "127.0.0.1"`。
本轮仅改 `:3-13` 的 `//!` 文档注释，**断言体一字未改**（实测确认）。

**立论事实性复核**（MINOR-2 要求引用 `webui_tokens: Some`）：
- `crates/rustcode-daemon/src/main.rs:177` 实测为 `webui_tokens: Some(token_store),` ✓
- `crates/rustcode-daemon/src/lib.rs:6154` 实测为 `enforce_token: webui_tokens.is_some(),` ✓（`Some(..)` ⇒ `true`）
⇒ 「虽 `enforce_token=true`」的立论成立。注释中未写死行号（只写 `` `src/main.rs` ``），
避免了行号漂移导致的事实错误；实测行号 177 与任务单描述的 173 有 4 行偏移，
**注释不含行号是正确的选择**。

### 1.4 `crates/rustcode-config/src/i18n/messages.rs`

| 编号 | 文件:行 | 改前 | 改后 | 属实 |
| --- | --- | --- | --- | --- |
| MINOR-4 | `:4898-4901` | （无，直接是 `/// Warning when binding...`） | 在 `DaemonWarnNonLoopback` 变体上方加 4 行注释：「已无发射点：Q3 裁决删除了 `run_server` 的启动横幅…变体与 `en.rs`/`zh_cn.rs` 两语种文案按 T-04 契约 K4 **保留**，勿当死码清理；非回环风险提示现由 `WebuiLanWarning`/`WebuiNonLoopbackWarning` 承担」 | 是 |

**纯注释性改动**：`git diff --stat` 显示 `4 +++`，0 删除，无代码语义变化。

**「已无发射点」事实复核**：
```
$ grep -rn "DaemonWarnNonLoopback" crates/
crates/rustcode-config/src/i18n/messages.rs:4900  (本注释)
crates/rustcode-config/src/i18n/messages.rs:4901  (本注释)
crates/rustcode-config/src/i18n/messages.rs:4903  (变体定义)
crates/rustcode-config/src/i18n/en.rs:3099        (match arm / 文案)
crates/rustcode-config/src/i18n/zh_cn.rs:2949     (match arm / 文案)
crates/rustcode-daemon/src/lib.rs:6339,6341       (本注释引用)
```
除注释外，**无任何构造点**（无 `Msg::DaemonWarnNonLoopback { .. }` 的 emit），注释陈述属实。

---

## 2. 每条编号的修复证据

### MAJOR-1 — 文档注释不再声称「非回环会发安全警告」

```
$ grep -rn "emit a security warning" crates/ ; echo "exit=$?"
exit=1
```
`GREP_EXIT=1`，**命中行数: 0**（`grep` 无匹配返回 1）。✓

### MAJOR-2 — `--en-ratio 0` 不得被放宽为 `EN_RATIO_DEFAULT`

单元级断言：
```
$ python3 - <<'PY'
import importlib.util, types
spec = importlib.util.spec_from_file_location("czd", "scripts/check-zh-docs.py")
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
a = types.SimpleNamespace(); a.en_ratio = 0.0
if getattr(a, "en_ratio", None) is None: a.en_ratio = m.EN_RATIO_DEFAULT
assert a.en_ratio == 0.0, a.en_ratio
print("MAJOR-2 OK: en_ratio 0.0 保持为 0.0")
print("EN_RATIO_DEFAULT =", m.EN_RATIO_DEFAULT)
PY
MAJOR-2 OK: en_ratio 0.0 保持为 0.0
EN_RATIO_DEFAULT = 0.05
MAJOR2_EXIT=0
```
✓ 关键：`EN_RATIO_DEFAULT = 0.05`，若沿用旧真值判空，`0.0` 会被替换成 `0.05`——**从「最严」放宽到「较松」，与 fail-closed 相反**。现 `is None` 判空保留 `0.0`。

端到端对照（反向 + 正向）：
```
$ python3 scripts/check-zh-docs.py check --base 3ee655e3 --files README.md --en-ratio -1
错误: --en-ratio 必须 >= 0（0 表示不允许任何纯英文行）；实际取值 -1.0
RATIO_NEG_EXIT=2                      ✓ 负值报 EnvError exit 2

$ python3 scripts/check-zh-docs.py check --base 3ee655e3 --files README.md --en-ratio 0
PASS README.md en=0/374=0.0000
  [AC-4-authorized] README.md | removed=- | added=./scripts/build-webui.sh | cause=D2
check: 受检 1，PASS 1，FAIL 0
RATIO_ZERO_EXIT=0                     ✓ 0 被当作合法最严阈值，未被误判为用法错误
```

### MAJOR-3 — `run_server` 补发非回环提示

分支矩阵（用 `rustc` 真实编译复刻 `is_loopback_authority` + 新增分支，**不触碰仓库**）：
```
0.0.0.0        loopback=false -> WebuiLanWarning
::             loopback=false -> WebuiLanWarning
127.0.0.1      loopback=true  -> (silent)
localhost      loopback=true  -> (silent)
::1            loopback=false -> WebuiNonLoopbackWarning      ← 见 §5 遗留风险 B
[::1]          loopback=true  -> (silent)
[::1]:13456    loopback=true  -> (silent)
192.168.1.5    loopback=false -> WebuiNonLoopbackWarning
VERIFY_EXIT=0
```
运行时证据见 §3.13 / §3.14（`rustcode daemon` 有 LAN 提示；独立二进制无）。

### MINOR-1 — `ensure_server_and_open` 文档注释

见 §1.1 表格，实测 `grep -n "本函数无默认值" → 5275`。内容与代码一致：
`ensure_server_and_open` 的 `host` 是入参、无内部默认值；CLI 侧 `:1047`/`:1780` 传 `0.0.0.0`，
TUI 侧传 `127.0.0.1`。✓

### MINOR-2 — 锁定测试立论

见 §1.3。`cargo test -p rustcode-daemon` 中
`test standalone_daemon_default_host_stays_loopback ... ok`（§3.9）。✓

### MINOR-4 — `DaemonWarnNonLoopback` 保留注释

见 §1.4。「已无发射点」经 grep 复核属实；`cargo test -p rustcode-config --lib` 327 passed
（含 `i18n::tests::*` 系列，如 `current_locale_fallback_is_zh_cn`），变体保留未破坏任何 i18n 测试。✓

### MINOR-5 — gate 未跟踪 md 可见性提示

```
$ python3 scripts/check-zh-docs.py gate --base 3ee655e3 | grep "未跟踪"
```
实测 gate PASS 输出中**未出现**「未跟踪 md 已排除」行——因为当前工作区
`git ls-files --others --exclude-standard -- "*.md"` 结果为空（本轮未新建 md）。
代码路径存在且 `:1289` `if untracked:` 门控正确（空列表不打印），符合「仅打印提示、不改分母」的要求。
`GATE_OK_EXIT=0` 不受影响。✓

### NIT-1 — hostscan 恒返回 0 标注

三处标注实测在位：
```
7:  python3 scripts/check-zh-docs.py hostscan  [--base REV]   # 清单产出器，非门禁，恒返回 0
1002:    """清单产出器（供 T-06 取 files_owned），**不是门禁**：无论命中多少恒返回 0。
1408:        help="列出默认绑定语义的 127.0.0.1/localhost 叙述（清单产出器，非门禁，恒返回 0）",
```
运行时佐证：`hostscan --base 3ee655e3` 命中 6 个文件（有命中）但 **`HOSTSCAN_EXIT=0`**。✓

### NIT-2 — 非法 base 以 exit 2 报环境错误

```
$ python3 scripts/check-zh-docs.py gate --base deadbeefdeadbeef
错误: --base 不是可解析的 commit: 'deadbeefdeadbeef'（git rev-parse --verify deadbeefdeadbeef^{commit} 失败: 无输出）
GATE_BAD_EXIT=2
```
✓ 退出码 2，且**未**退化成几百条假 FAIL（与注释所述根因一致）。

---

## 3. 验证命令真实输出与退出码

### 3.1 `python3 -m py_compile scripts/check-zh-docs.py`
```
exit=0
```

### 3.2 `python3 scripts/check-zh-docs.py gate --base 3ee655e3` — 期望 exit 0
```
GATE_OK_EXIT=0
...
gate: PASS
```
（完整输出含 AC-4 授权项等明细，共约 30 行尾部已核对；末行 `gate: PASS`）

### 3.3 `python3 scripts/check-zh-docs.py gate --base deadbeefdeadbeef` — 期望 exit 2
```
GATE_BAD_EXIT=2
错误: --base 不是可解析的 commit: 'deadbeefdeadbeef'（git rev-parse --verify deadbeefdeadbeef^{commit} 失败: 无输出）
```

### 3.4 `python3 scripts/check-zh-docs.py hostscan --base 3ee655e3` — 期望 exit 0
```
HOSTSCAN_EXIT=0
hostscan: base=3ee655e3 命中 6 个文件（内容来源：工作区优先，缺失时回退基线）

## 命中文件（供 T-06 的 files_owned；T-06 需自行排除 README.md）

docker/README.md
docs/superpowers/plans/2026-05-29-webui.md
docs/superpowers/specs/2026-05-29-webui-design.md
extensions/jetbrains/PRIVACY.md
extensions/jetbrains/README.md
extensions/jetbrains/docs/jetbrains.md
```

### 3.5 MAJOR-2 单元级断言 — 见 §2，`MAJOR2_EXIT=0`

### 3.6 `grep -rn "emit a security warning" crates/` — 期望 0 命中
```
GREP_EXIT=1 (1=无命中,期望)
命中行数: 0
```

### 3.7 `cargo fmt --check`
```
FMT_EXIT=0
```
（无输出，即全工作区格式干净）

### 3.8 `cargo clippy -p rustcode-daemon -p rustcode -p rustcode-config --all-targets`
```
CLIPPY_EXIT=0
...
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 38.31s
```
warning 归属核对（47 条 `^warning:`）：
```
warning: `rustcode-capabilities` (lib) generated 11 warnings
warning: `rustcode-config` (lib) generated 3 warnings
warning: `rustcode-config` (lib test) generated 4 warnings (2 duplicates)
warning: `rustcode-daemon` (lib) generated 6 warnings
warning: `rustcode-daemon` (lib test) generated 14 warnings (6 duplicates)
warning: `rustcode` (lib) generated 1 warning
warning: `rustcode` (lib test) generated 1 warning (1 duplicate)
warning: `rustcode-tuix` (lib) generated 8 warnings
```
`crates/rustcode-daemon/src/lib.rs` 仅 2 处被点名：`:4785`（empty line after doc comment）
与 `:4521`（too many arguments 11/7）。二者均**远离**本轮改动区（5275 / 6035 / 6332-6374），
为既有 warning；本轮改动文件 `messages.rs`、`default_host_lock.rs` **零 warning**。
`rustcode-tuix` / `rustcode-cli` 的 warning 亦与本轮无关。

### 3.9 `cargo test -p rustcode-daemon`
```
TEST_DAEMON_EXIT=0

     Running unittests src/lib.rs (target/debug/deps/rustcode_daemon-c00f6f792f2e5f83)
test result: ok. 307 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.60s
     Running unittests src/main.rs
test result: ok. 0 passed; 0 failed; ...
     Running tests/daemon_token_auth.rs
test chat_requires_token_health_is_public ... ok
test result: ok. 1 passed; 0 failed; ... finished in 8.77s
     Running tests/default_host_lock.rs
test standalone_daemon_default_host_stays_loopback ... ok
test result: ok. 1 passed; 0 failed; ...
     Running tests/legacy_turn_boundary_repair.rs
test result: ok. 7 passed; 0 failed; ...
   Doc-tests rustcode_daemon
test result: ok. 0 passed; 0 failed; ...
```
**汇总：passed=316, failed=0**

要求的三组用例均在场：
- `default_host_lock` → `standalone_daemon_default_host_stays_loopback ... ok`
- `daemon_token_auth` → `chat_requires_token_health_is_public ... ok`
- `channel_mode_tests`（在 lib unittests 307 内）→
  `interactive_responder_is_required_for_prompt_required_modes` / `known_clients_interactive_on_loopback_or_token` /
  `resolve_channel_header` / `request_approval_mode_overrides_global_mode` 全 `ok`

### 3.10 `cargo test -p rustcode --bin rustcode default_host_tests`
```
TEST_CLI_EXIT=0
running 2 tests
test default_host_tests::webui_defaults_to_all_interfaces ... ok
test default_host_tests::webui_explicit_host_overrides_default ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 88 filtered out; finished in 0.00s
```

### 3.11 `cargo test -p rustcode-config --lib`
```
TEST_CFG_EXIT=0
test result: ok. 327 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
```
含 `i18n::tests::*` 系列（`authoritative_set_brand_overrides_pre_scan`、`cli_flag_wins_over_everything`、
`current_locale_fallback_is_zh_cn`、`env_explicit_english_resolves_to_en` 等）与 `i18n::en::*` 文案测试，
**i18n 内容测试全覆盖且通过**。

### 3.12 `cargo build -p rustcode`
```
BUILD_EXIT=0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 50.46s
-rwxr-xr-x 2 root root 542195928 Sep  9 01:56 target/debug/rustcode
```
另补 `cargo build -p rustcode-daemon --bin rustcode-daemon`（`BUILD_DAEMON_EXIT=0`）以确保独立二进制为最新。

### 3.13 `RUSTCODE_HOME=$(mktemp -d) timeout 10 ./target/debug/rustcode daemon --port 13499`
```
DAEMON_CLI_EXIT=124 (124=timeout 属预期，说明进程常驻)
Starting RustCode daemon on port 13499...
Press Ctrl+C to stop.
Idle timeout: 30 minutes
RustCode API server listening on http://0.0.0.0:13499

[!] Primary address is a LAN IP; only devices on the same network can reach it. For public access use a tunnel (e.g. cloudflared / Tailscale). There is no TLS, so anyone who can reach it can get in with the token.

API endpoints:
  GET    /health                                  - Health check
  ...
```
✓ **含 LAN 风险提示**，且位置正确（listening 行之后、API endpoints 之前、回环绑定不打印）。

默认语区（zh-CN）下的中文原文（额外实跑 `LC_ALL= LANG= LANGUAGE=`，`EXIT=124`）：
```
正在端口 13497 上启动 RustCode 守护进程...
按 Ctrl+C 停止。
空闲超时：30 分钟
RustCode API 服务已启动，监听地址 http://0.0.0.0:13497

[!] 主地址为局域网 IP，仅同一网络内的设备可访问；公网访问请用隧道（如 cloudflared / Tailscale）。无 TLS，凡能访问者凭 token 即可进入。

API 端点：
```
（本机 `LANG=en_US.UTF-8` 故首次输出为英文；`rustcode-config` 的
`i18n::tests::current_locale_fallback_is_zh_cn` 确认无语区时回落 zh-CN。）

### 3.14 `RUSTCODE_HOME=$(mktemp -d) timeout 10 ./target/debug/rustcode-daemon --port 13498`
```
DAEMON_BIN_EXIT=124 (124=timeout 预期)
Idle timeout: 30 minutes
RustCode API server listening on http://127.0.0.1:13498

API endpoints:
  GET    /health                                  - Health check
  ...
--- LAN 提示命中检查 ---
0
grep_exit=1
```
✓ 独立二进制走 `127.0.0.1`，**无** LAN / 非回环提示（grep 命中 0）。与 MINOR-2 立论、Q2 红线一致。

### 3.15 附加：`cargo check --workspace --all-targets`（跨 crate 影响面）
```
CHECK_WS_EXIT=0
=== 错误 ===
0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.41s
```
理由：本轮触及 `rustcode-config`（共享 crate，i18n 消息枚举为公共协议面），故补做 workspace 级校验。

### 3.16 测试计数汇总

| 套件 | passed | failed |
| --- | --- | --- |
| `cargo test -p rustcode-daemon` | 316 | 0 |
| `cargo test -p rustcode --bin rustcode default_host_tests` | 2 | 0 |
| `cargo test -p rustcode-config --lib` | 327 | 0 |
| **合计** | **645** | **0** |

---

## 4. `git diff --stat 3ee655e3 -- <files_owned>`

```
$ git diff --stat 3ee655e3 -- crates/rustcode-daemon/src/lib.rs \
    crates/rustcode-config/src/i18n/messages.rs \
    scripts/check-zh-docs.py crates/rustcode-daemon/tests/default_host_lock.rs

 crates/rustcode-config/src/i18n/messages.rs |  4 +++
 crates/rustcode-daemon/src/lib.rs           | 43 ++++++++++++++++++-----------
 2 files changed, 31 insertions(+), 16 deletions(-)
```

未跟踪（无 diff，按整文件计）：

| 文件 | 行数 | 状态 |
| --- | --- | --- |
| `scripts/check-zh-docs.py` | 1473 | 新增未跟踪 |
| `crates/rustcode-daemon/tests/default_host_lock.rs` | 49 | 新增未跟踪 |

```
$ git status --porcelain -- scripts/check-zh-docs.py crates/rustcode-daemon/tests/default_host_lock.rs
?? crates/rustcode-daemon/tests/default_host_lock.rs
?? scripts/check-zh-docs.py
```

工作区总改动数：开工 169 → 收工 **169**（未增未减）。

---

## 5. 遗留判断

### 遗留判断 A：`Msg::WebuiLanWarning` 文案 vs `0.0.0.0` 通配绑定

**问题**：中文文案「主地址为**局域网 IP**，仅同一网络内的设备可访问；公网访问请用隧道…」
（en 同义）。当绑定 `0.0.0.0`（通配所有网卡）时，该断言为**假**：监听面可能包含公网网卡，
且端口转发/DMZ 会进一步放大暴露面。

**判定：构成误导性提示（是），但本轮不认定为需就地修复的安全问题（否）。**

理由：

1. **不改变任何访问控制，也不新增暴露面。** 实际暴露完全由已批准裁决 Q2-A（CLI 默认 `0.0.0.0`）
   决定，与文案无关。文案只影响用户对既有暴露面的**认知**。
2. **关键风险信息仍在。** 同一条消息含「无 TLS，凡能访问者凭 token 即可进入」——读者仍被明确告知
   「可达 ⇒ 凭 token 即可进入」。误导集中在前半句的范围断言，而非整体风险缺失。
3. **完整修复不在 `files_owned` 内。** 新增 `Msg::DaemonBindWildcard` 变体必须同时改
   `crates/rustcode-config/src/i18n/en.rs` 与 `zh_cn.rs`（新增 match arm），二者归 T-04 / i18n owner，
   本任务硬约束禁止触碰。且新增变体会触发 i18n 完整性契约检查，需架构确认。
4. **回退到 Q3 之前的横幅也不是「更优基线」。** Q3 是**刻意**移除 `Msg::DaemonWarnNonLoopback`
   启动横幅的（默认 `0.0.0.0` 下每次启动必打印 ⇒ 退化为噪音，用户脱敏）。原样恢复违反 Q3 裁决。

**实测到的一个加重因素（如实记录）**：在 `run_server` 这条路径上，用户看到的地址是
`http://0.0.0.0:13497`（通配），而消息却说「主地址为局域网 IP」——`ensure_server_and_open`
路径中「主地址」指其探测并打印的 `lan_ip`，而 `run_server` 路径**根本没打印任何局域网 IP**。
即该消息在 `run_server` 路径下不仅范围断言为假，其「主地址」指代对象也未出现在输出中。
这提高了修复价值，但不改变上述 4 条判定（仍是认知层问题，非访问控制问题）。

**建议（按优先级）**：

- **P1（推荐，需 architect + i18n owner）**：新增 `Msg::DaemonBindWildcard`（无参数或带 `addr`），
  文案改为不宣称范围，例如「已绑定通配地址 `0.0.0.0`：本机所有网卡（可能含公网网卡/端口转发）
  均可访问；无 TLS，凡可达者凭 token 即可进入。仅本机使用请加 `--host 127.0.0.1`」。
  由 `run_server` 与 `ensure_server_and_open` 的 wildcard 分支统一发射。owner：T-04 i18n。
- **P2（过渡，1 行，需 architect 批准）**：在 `run_server` 的 wildcard 分支改发
  `Msg::WebuiNonLoopbackWarning`（「已绑定非回环地址：凡能访问该地址者凭此 token 即可进入，
  请仅在可信网络使用（无 TLS）」）。该文案对 `0.0.0.0` **完全准确且不宣称范围**。
  代价：与 `ensure_server_and_open` 的 wildcard 分支不一致，且丢失「隧道」提示。
  **本轮未执行**——属对评审指令中既定分支映射的偏离，未经批准不擅自改。
- **P3（现状）**：维持现状 + 本文档记录。

**责任建议**：交 `solution-architect` 裁定 P1 / P2 / P3；若选 P1 需同步派单给 i18n（`en.rs` / `zh_cn.rs`）owner。

### 遗留风险 B：`is_loopback_authority("::1") == false`（既有谓词缺陷，本轮新路径继承）

`crates/rustcode-daemon/src/lib.rs:1272` 的 `is_loopback_authority` 处理了 `[::1]` 括号形式，
但对**裸 `::1`** 走 `authority.split(':').next()` 得到空串 `""`，`matches!("", ...)` 为 false ⇒
裸 `::1` 被判为**非回环**（`rustc` 实测复刻已确认，见 §2 MAJOR-3 分支矩阵）。

影响：

- 本轮新增的 `run_server` 分支对 `--host ::1` 会打印 `WebuiNonLoopbackWarning`（**误报**）。
  Q3 之前的旧代码用字面量 `host != "::1"` 显式排除了 `::1`，故这是本轮引入的一处**轻微回退**
  （噪音方向，非静默方向）。
- 同一谓词已被 `ensure_server_and_open`（`:5399`）与 `client_interactive_permission`（`:1288`）使用，
  说明该缺陷在 `ensure_server_and_open` 中**已存在**。

**为何本轮不修**：修 `is_loopback_authority` 会连带改变 `client_interactive_permission` 的行为——
该函数决定「已知客户端在回环上是否可接收交互式审批」，把 `::1` 从 false 改为 true 会**放宽权限面**
（`lib.rs:6154` 附近的安全语义）。这属安全相关行为变更，超出 T-13（文档/i18n 返工）范围，
须 architect 裁决，不在本轮擅自动手。

方向性说明：误报是**过度告警**（fail-safe 方向），不会造成「本该告警却静默」，
故不阻塞本轮结论。

**建议**：单独开单修 `is_loopback_authority`（正确解析 `::1` / `[::1]` / `::ffff:127.0.0.1`），
并同步评估 `client_interactive_permission` 的放宽影响。owner：architect + daemon owner。

### 遗留风险 C：`Msg::DaemonWarnNonLoopback` 已无发射点

已按 MINOR-4 加注释锁定（契约 K4 保留）。风险是未来有人当死码清理该变体及其双语文案；
注释已明确「勿当死码清理」。若后续 P1 采纳 `Msg::DaemonBindWildcard`，可一并评估是否
复用/替换该变体。**无需本轮动作**。

---

## 6. 回滚方案

**前提**：全部改动均未 `git add` / `git commit`，HEAD 仍为 `3ee655e3`。回滚不涉及 reflog。

| 文件 | 类型 | 回滚方式 | 可安全回退 |
| --- | --- | --- | --- |
| `crates/rustcode-config/src/i18n/messages.rs` | **纯注释**（`+4` 行，0 删除） | `git checkout 3ee655e3 -- crates/rustcode-config/src/i18n/messages.rs` | 是。零语义影响，唯一代价是丢失「勿当死码清理」警示（见风险 C） |
| `crates/rustcode-daemon/src/lib.rs` `:5275` | **纯注释**（MINOR-1） | 手工还原该行；或整文件 `git checkout 3ee655e3 -- <file>`（会连带回滚 MAJOR-1/MAJOR-3） | 是（单独还原时）。误还原的后果仅是文档与 CLI 默认 `0.0.0.0` 不一致，无运行时影响 |
| `crates/rustcode-daemon/src/lib.rs` `:6035` | **纯注释**（MAJOR-1） | 同上 | 是。但会重新引入「Non-loopback hosts emit a security warning」的**不实陈述**，不建议单独回退 |
| `crates/rustcode-daemon/src/lib.rs` `:6332-6350` | **注释改写 + 删除发射代码**（MAJOR-3 前半） | `git checkout 3ee655e3 -- <file>` | 否（与 `:6357-6374` 强耦合，见下） |
| `crates/rustcode-daemon/src/lib.rs` `:6357-6374` | **新增行为代码**（MAJOR-3 后半） | 删除该 `if !is_loopback_authority(&host) { ... }` 块 | 需成对回退 |
| `scripts/check-zh-docs.py` | 未跟踪新文件 | `rm scripts/check-zh-docs.py` | 是（整体移除）。无其他文件引用它（T-06 报告已产出） |
| `crates/rustcode-daemon/tests/default_host_lock.rs` | 未跟踪新文件 | `rm crates/rustcode-daemon/tests/default_host_lock.rs` | 是。但会**失去 Q2 红线的机器化守卫**，不建议 |

### 关键耦合警告（MAJOR-3 必须成对回退）

`lib.rs` 的两个 hunk 是**同一逻辑改动的两半**：

- 前半（`:6332-6350`）删除了旧的 `if host != "127.0.0.1" && ... { eprintln!(DaemonWarnNonLoopback) }`；
- 后半（`:6357-6374`）新增了 `if !is_loopback_authority(&host) { ... }` 补发。

**只回退后半** ⇒ `rustcode daemon --host 0.0.0.0` 完全静默、无任何非回环提示（**最危险**，
静默暴露，违反「失败路径必须显式错误 / 禁止静默」约束）。
**只回退前半** ⇒ 旧横幅与新增提示并存，重复告警。

因此 MAJOR-3 只能 **整体回退**（`git checkout 3ee655e3 -- crates/rustcode-daemon/src/lib.rs`
后重放其余 hunk）或 **整体保留**。

### 推荐的最小回滚集

若评审只否定某一项，按此顺序选：

1. 仅否定 MINOR-4 → 回退 `messages.rs`（安全，纯注释）。
2. 仅否定 MINOR-1/MAJOR-1 → 手工还原两处注释（安全，纯注释）。
3. 仅否定 MINOR-5/NIT-1/NIT-2 → 编辑 `scripts/check-zh-docs.py` 对应片段；NIT-2 移除后
   `gate --base <bad>` 会退化为大量假 FAIL 且退出码 1（不再是 2），**已知后果**。
4. 否定 MAJOR-2 → 不建议单独回退：会重新引入「`--en-ratio 0` 被静默放宽为 0.05」，
   与 fail-closed 相悖（§2 有 `EN_RATIO_DEFAULT = 0.05` 实测佐证）。
5. 否定 MAJOR-3 → 必须按上表**成对/整体**回退 `lib.rs`，不可拆。

---

## 7. 结论

**status: `done`** / **decision: `proceed`**

- 前一轮落盘的 8 项改动（MAJOR-1/2/3、MINOR-1/2/4/5、NIT-1/2）**逐 hunk 核实全部属实**，
  `files_owned` 内无遗漏、无越界，本轮**零新增代码改动**、零就地修复。
- 15 项验证命令**全部按判据通过**（脚本类 6 项 + cargo 类 7 项 + 运行时进程 2 项，
  另加 workspace 级 check 1 项）：`gate` 合法基线 exit 0 / 非法基线 exit 2、`hostscan` exit 0、
  MAJOR-2 断言 exit 0、`emit a security warning` 0 命中、`cargo fmt --check` exit 0、
  `clippy` exit 0、**测试 645 passed / 0 failed**、workspace check 0 error。
- 运行时行为符合裁决：`rustcode daemon` 绑定 `0.0.0.0` **含 LAN 风险提示**；
  独立 `rustcode-daemon` 绑定 `127.0.0.1` **无提示**。
- 硬约束全部守住：`DEFAULT_HOST = "127.0.0.1"` 红线未动、CLI `0.0.0.0` 裁决未回退、
  未 `git add`/`commit`、工作区改动数 169 → 169、未改任何 md 汉化正文、
  files_owned 无新增 Emoji（唯一命中为基线既存的 `messages.rs:3216`）。
- 格式与 clippy 干净（本轮改动文件零 warning）。

### G4 判定

**G4 可判定为 `approved`。**

依据：G4 关注「非回环绑定是否有用户可见提示、回环绑定是否保持静默」——
§3.13 / §3.14 两条真实进程启动输出直接覆盖该判据，且方向正确（暴露时提示、回环时静默）。
遗留判断 A（LAN 文案范围断言不精确）与遗留风险 B（裸 `::1` 误判）**均不改变 G4 的判据成立性**：
A 是提示措辞的精确性问题（风险信息仍在），B 是过度告警方向（fail-safe）。
二者已按 §5 记录并给出 owner 与修复路径，不构成 G4 的阻塞项。

### 需编排者转派的事项

1. 遗留判断 A → `solution-architect` 裁定 P1（新增 `Msg::DaemonBindWildcard`，需 i18n owner 配改
   `en.rs`/`zh_cn.rs`）/ P2（`run_server` wildcard 改发 `WebuiNonLoopbackWarning`，1 行）/ P3（现状）。
2. 遗留风险 B → 单独开单修 `is_loopback_authority`，并评估对 `client_interactive_permission`
   的权限放宽影响（安全相关，需 architect）。
3. 遗留风险 C → 仅备案，保持 MINOR-4 注释即可。
