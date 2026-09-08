---
kind: design
id: DESIGN-002
from: solution-architect
to: [project-manager]
feature: 2026-09-07-zh-docs-webui
status: approved
decision: proceed
requires: [REQ-002, DESIGN-001]
files_owned: []
architecture_constraints:
  touches_runtime_lifecycle: false
  touches_persistence: false
  touches_cross_crate_deps: false
created: 2026-09-09
---

# 01-design-addendum-d5 · `rustcode daemon` 补 `--host` 参数（缺陷 D-5）

> 本文是 `01-design.md`（DESIGN-001）与 `01-design-addendum.md` 的**补遗**，只覆盖 G5 登记的缺陷
> **D-5**（`.codebuddy/artifacts/2026-09-07-zh-docs-webui/STATUS.md:115`）。
> 裁决前提（不可推翻）：Q2-A（`rustcode webui` / `rustcode daemon` 两个 CLI 入口默认绑定 `0.0.0.0`）已通过 G1–G6，
> 本改动**不得回退该默认值**。
>
> **勘察工具说明**：本轮环境无 `Bash`，故 `git log` 未能执行。近期改动方向以
> `STATUS.md:66-67`（T-13 R1 返工 done、T-14 IPv6 回环 done）、`STATUS.md:132-142`（编排终态）
> 与源码内注释为准；所有代码结论均由 `文件:行` 直接引用，未引用处不臆测。

---

## 1. 现状（逐条可复核）

### 1.1 `rustcode daemon` 的绑定地址是硬编码的

- 枚举定义 `crates/rustcode-cli/src/main.rs:1026-1038`：`Commands::Daemon { port, client, idle_timeout }`，
  **没有 `host` 字段**。
- 分支实现 `crates/rustcode-cli/src/main.rs:1738-1742` 解构三个字段，
  在 `crates/rustcode-cli/src/main.rs:1779-1790` 构造 `ServerOpts`，其中
  `host: "0.0.0.0".to_string()`（`main.rs:1780`）为硬编码字面量。
- `Commands::Daemon` 的另一处匹配 `crates/rustcode-cli/src/main.rs:3568-3570` 是 `Commands::Daemon { .. } => unreachable!(...)`，
  用 `..` 通配，**加字段不影响**。

### 1.2 `rustcode webui` 的既有 `--host` 写法（本设计的复用模板）

`crates/rustcode-cli/src/main.rs:1040-1049`：

```rust
    /// Start the local in-process browser webui server (no separate binary needed)
    Webui {
        /// Port (default 13457; ...)
        #[arg(long, default_value_t = rustcode_daemon::WEBUI_DEFAULT_PORT)]
        port: u16,
        /// Bind address (default 0.0.0.0: reachable over LAN; use 127.0.0.1 to
        /// restrict to this machine. Token-protected only, with no TLS)
        #[arg(long, default_value = "0.0.0.0")]
        host: String,
    },
```

类型 = `String`，默认值 = `"0.0.0.0"`，**无 `value_parser`**（因此 clap 不做内容校验）。

### 1.3 CLI help 是 i18n 的，且已存在可复用的 `Msg::CliHelpHost`

- `crates/rustcode-cli/src/main.rs:353` `build_i18n_command()` 用 `Cli::command()` + 一串 `mut_arg/mut_subcommand`
  注入本地化 about/help；`main.rs:1594-1606` 显示 `--help`/`-h` 走该函数，`main.rs:1217-1220`
  显示 shell completion 也源出同一函数（`completion_command()` → `build_i18n_command()`）。
- `crates/rustcode-cli/src/main.rs:474-478` 为 `webui` 注入：
  `.mut_arg("host", |a| a.help(t(Msg::CliHelpHost).into_owned()))`。
- `crates/rustcode-cli/src/main.rs:467-473` 为 `daemon` 只注入了 `about` / `port` / `idle_timeout`，
  **缺 `host`**。
- 变体 `Msg::CliHelpHost` 已存在于 `crates/rustcode-config/src/i18n/messages.rs:3916`，两语种文案已就绪：
  - en `crates/rustcode-config/src/i18n/en.rs:2521`：`"Bind address (default: 0.0.0.0; use 127.0.0.1 for local-only)"`
  - zh_cn `crates/rustcode-config/src/i18n/zh_cn.rs:2429`：`"绑定地址（默认：0.0.0.0；改 127.0.0.1 则仅本机可访问）"`

  ⇒ **无需新增 `Msg` 变体，无需改 `en.rs` / `zh_cn.rs`**。

### 1.4 `mut_arg` 的硬耦合（关键约束）

`clap_builder-4.6.0/src/builder/command.rs:244-254`：

```rust
    pub fn mut_arg<F>(mut self, arg_id: impl AsRef<str>, f: F) -> Self
    ...
        let a = self
            .args
            .remove_by_name(id)
            .unwrap_or_else(|| panic!("Argument `{id}` is undefined"));
```

即 `mut_arg` 在 arg id 不存在时**无条件 panic**（非 debug-only）。
⇒ **derive 字段与 `mut_arg("host", …)` 必须同一次改动内一起落地**；
只加其一会让 `rustcode --help` / `rustcode <子命令> --help` / `rustcode completion <shell>` 直接 panic。
这是本设计把改动收敛为「单文件、单任务、单 commit」的决定性理由。

### 1.5 消费者（谁在拉起 `rustcode daemon`、传了什么）

- JetBrains `extensions/jetbrains/src/main/kotlin/com/rustcode/jetbrains/daemon/RustCodeDaemonProcess.kt:61-63`
  在「非 Windows 且无打包 daemon」时回退到 CLI `rustcode daemon`；
  参数在 `RustCodeDaemonProcess.kt:113-126` 组装：`--port <port> --client jetbrains`（Windows 再加 `--idle-timeout 0`），
  **不传 `--host`**。
- VSCode `extensions/vscode/src/daemon/process.ts:369` `const portArgs = ['--port', String(port), '--client', 'vscode']`，
  分别在 `:377` / `:393` / `:410` 处拼成 `['daemon', ...portArgs]` 或直接 `portArgs`，**不传 `--host`**。
- 独立二进制 `crates/rustcode-daemon/src/main.rs:20-115` 有自己的 `parse_daemon_args()`，`DEFAULT_HOST = "127.0.0.1"`（`main.rs:21`），
  **按 `AGENTS.md:38` 明确要求不得改动**。

⇒ 新增参数是**纯 additive**：现有调用方一个字节都不用改；反过来说，若将来扩展要传 `--host`，
旧版 CLI 会 exit 2（前向兼容注意项，见 §7 R4）。

### 1.6 失败路径（本改动会走到的那一条）

`crates/rustcode-daemon/src/lib.rs:6493-6511`（`run_server`，`prebound_listener: None` 走 bind 分支）：

```rust
        None => match tokio::net::TcpListener::bind(&addr).await {
            Ok(l) => l,
            Err(e) => {
                eprintln!("{}", t(Msg::DaemonFatalBind { addr: &addr, error: &e.to_string() }));
                std::process::exit(1);
            }
        },
```

即 **bind 失败是 `std::process::exit(1)`，不返回 `Err`**，所以 `main.rs:1792-1800` 的
`Msg::CliDaemonFatal` 分支**对 bind 失败不生效**（它只在 `run_server` 返回 `Err` 时触发）。
`addr` 由 `crates/rustcode-daemon/src/lib.rs:6370` `let addr = format!("{host}:{port}");` 拼出。
`Msg::DaemonFatalBind` 的 zh 文案见 `crates/rustcode-config/src/i18n/zh_cn.rs:2994-2996`：`"致命错误：无法绑定到 {addr}：{error}"`。

对照 `rustcode webui`：`crates/rustcode-cli/src/main.rs:1803-1817` 把 `ensure_server_and_open` 的返回串
`eprintln!` 后 `return Ok(0)`；绑定失败返回 `Msg::WebuiBindFailed`（`crates/rustcode-daemon/src/lib.rs:5322-5331`），
即 **webui 绑定失败退出码为 0**，与 daemon 的 1 不同 —— 这是既有差异，本轮不动（§7 R2）。

### 1.7 风险提示判据（R1/T-14 结论，本轮不得改）

- `crates/rustcode-daemon/src/lib.rs:6381-6394`（非 quiet 分支）按 `is_loopback_bind_host(&host)` 补发
  `Msg::WebuiLanWarning`（`host == "0.0.0.0" || host == "::"`）或 `Msg::WebuiNonLoopbackWarning`（其它非回环），
  回环绑定**保持静默**。
- 谓词定义 `crates/rustcode-daemon/src/lib.rs:1300-1302`
  （`is_loopback_authority` + 裸 `::1` + `::ffff:127.0.0.1`），注释（`:1281-1299`）明写
  **该谓词只用于是否打印提示，不得用于任何鉴权判定**。
- `Msg::DaemonWarnNonLoopback` 变体按契约保留在 `crates/rustcode-config/src/i18n/messages.rs:4903`
  （注释 `:4900-4902`），**不复活**。
- `rustcode daemon` 传 `quiet: false`（`main.rs:1785`），因此该提示在此入口恒为非 quiet 路径；
  **该入口不存在 quiet 开关**，无需讨论 quiet 下是否提示。

### 1.8 测试面

- `crates/rustcode-cli/tests/shell_completion.rs` **全文 21 行**，唯一断言是
  `rustcode completion bash` 成功、stderr 为空、stdout 含 `"rustcode"`、且不产生 `RUSTCODE_HOME` 副作用。
  **无任何 flag/快照断言** ⇒ 新增 CLI 参数**不影响**该测试。
- 全仓测试代码中 `--host` / `CliHelpHost` / `Commands::Daemon` 的断言命中数为 **0**（`grep -- '--host|CliHelpHost|Commands::Daemon' **/*test*`）。
  ⇒ 无既有测试会因新增参数而变红。

---

## 2. 候选方案与取舍

### 方案 A（选中）：`String` 类型 + `default_value = "0.0.0.0"`，逐字复用 `webui` 写法

- 在 `Commands::Daemon` 加 `host: String` 字段（`#[arg(long, default_value = "0.0.0.0")]`），
  `mut_arg("host", … CliHelpHost)` 挂到 `daemon` 子命令，
  分支里把 `host` 原样传给 `ServerOpts.host`（替换 `main.rs:1780` 的字面量）。
- 改动 1 个文件、3 处（枚举字段 / `build_i18n_command` 一行 / 分支传参），零跨 crate 契约变更。

### 方案 B（放弃）：`host: IpAddr` + `value_parser`

- 放弃理由 1 —— **语义会静默变化**：`ServerOpts.host` 是 `String`（`crates/rustcode-daemon/src/lib.rs:6060`），
  `IpAddr` 往返 `to_string()` 会改写形态；`::ffff:127.0.0.1` 解析后 `Display` 为 `127.0.0.1`，
  而 `is_loopback_bind_host`（`lib.rs:1301`）是靠**字面量相等**认它的 —— 改判据即违反 §1.7 的冻结结论。
- 放弃理由 2 —— **收窄合法输入**：`is_loopback_authority`（`lib.rs:1278`）显式认 `localhost`，
  说明主机名是被支持的输入；`IpAddr` 会把它拒在解析期。
- 放弃理由 3 —— 与 `webui` 不一致，会让两个 CLI 入口的 `--host` 行为分叉（本 feature 的 Q2-A 裁决要求两个入口口径一致）。

### 方案 C（放弃）：不动 CLI，改成读环境变量 `RUSTCODE_DAEMON_HOST`

- 放弃理由 1 —— 用户已裁决「优先修 `--host` 参数」，且 D-5 的原话是"没有 `--host` 参数"
  （`STATUS.md:115`），环境变量不是等价修复。
- 放弃理由 2 —— 引入第二套「驱动端决定 bind 地址」的机制，与 `--port` / `--client` / `--idle-timeout`
  的既有风格不一致（`main.rs:1026-1038` 全部是 clap 参数）。
- 放弃理由 3 —— 该入口已有 `RUSTCODE_DAEMON_IDLE_TIMEOUT` 这类 env 覆盖（`main.rs:1762-1768`），
  再叠一层 env 会让优先级（参数 vs env vs 默认）变得难以向用户解释。

### 方案 D（放弃）：顺手把 `rustcode-daemon` 独立二进制的默认值也统一成 `0.0.0.0`

- 放弃理由 —— `AGENTS.md:38` 明文禁止："这是刻意保留的安全边界，**不要为了「统一默认值」把它们也改成 0.0.0.0**"。

---

## 3. 目标架构

**模块划分**：无新增模块。改动完全落在 `rustcode-cli` 的参数解析层（L3 driver）。

**数据流**：

```text
argv ──clap(derive + build_i18n_command)──▶ Commands::Daemon { port, host, client, idle_timeout }
   │                                              │
   │                                              ▼  main.rs:1779
   │                                    rustcode_daemon::ServerOpts { host, port, … }
   │                                              │
   │                                              ▼  lib.rs:6370
   │                                    addr = format!("{host}:{port}")
   │                                              │
   │                        ┌─────────────────────┴──────────────────────┐
   │                        ▼ lib.rs:6381-6394（非 quiet）                ▼ lib.rs:6496-6511
   │             is_loopback_bind_host(host) ?  静默 : LAN/非回环提示      TcpListener::bind(addr)
   │                                                                       └─ Err ⇒ DaemonFatalBind + exit(1)
   └─▶ --help / completion：build_i18n_command()（main.rs:353/1196）复用同一 mut_arg 注入
```

**控制流**：与现状完全一致 —— `Commands::Daemon` 仍在 `run()` 内联处理（`main.rs:1738`），
`handle_command` 的 `Commands::Daemon { .. }` 仍是 `unreachable!`（`main.rs:3568`）。
`rustcode daemon` 依然**直连 `run_server`**，不经 `ensure_server_and_open`
（所以 RI-1 的 `http://::1:PORT/` 非法 URL 问题与此入口无关，本轮不引入也不修复）。

---

## 4. 接口契约（冻结，实现方不得偏离）

### 4.1 参数签名与语义

| 项 | 冻结值 | 理由 / 依据 |
|---|---|---|
| 形式 | `rustcode daemon [--host <IP>]`（长选项，取值，`--host=<IP>` 等号形式由 clap 自动支持） | 与 `webui` 一致（`main.rs:1047`） |
| 字段名 / arg id | `host`（derive 字段名即 clap arg id，`mut_arg("host", …)` 依赖它） | `main.rs:477` 同构 |
| 类型 | **`String`** | §2 方案 B 的三条放弃理由；且 `ServerOpts.host: String`（`lib.rs:6060`）无需转换 |
| clap 属性 | `#[arg(long, default_value = "0.0.0.0")]`，**不写 `value_parser`** | 与 `main.rs:1047` 逐字一致 |
| 默认值 | **`"0.0.0.0"`** | Q2-A 不得回退；与 `main.rs:1780` 现有硬编码等价 |
| 字段位置 | 在 `port` 之后、`client` 之前 | 与 `Webui` 的 `port → host` 顺序一致 |
| 与 `webui --host` 是否一致 | **完全一致**（类型、默认值、help 变体、失败语义） | 用户裁决要求 |

### 4.2 精确改动（3 处，同一文件 `crates/rustcode-cli/src/main.rs`）

**(a) 枚举定义 `main.rs:1026-1038`** —— 在 `port` 字段后插入：

```rust
        /// Bind address (default 0.0.0.0: reachable over LAN; use 127.0.0.1 to
        /// restrict to this machine. Token-protected only, with no TLS)
        #[arg(long, default_value = "0.0.0.0")]
        host: String,
```

doc comment 与 `main.rs:1045-1046`（`webui` 的同名字段）**逐字相同**：它是 derive 的 fallback help，
运行时会被 §4.3 的 i18n help 覆盖，但保持与 `webui` 同源可避免两处漂移。

**(b) `build_i18n_command()` 的 `daemon` 子命令 `main.rs:467-473`** —— 追加一行（复用既有变体，不新增文案）：

```rust
    .mut_subcommand("daemon", |s| {
        s.about(t(Msg::CliAboutDaemon).into_owned())
            .mut_arg("port", |a| a.help(t(Msg::CliHelpPortDaemon).into_owned()))
            .mut_arg("host", |a| a.help(t(Msg::CliHelpHost).into_owned()))
            .mut_arg("idle_timeout", |a| {
                a.help(t(Msg::CliHelpIdleTimeout).into_owned())
            })
    })
```

**(c) 分支实现 `main.rs:1738-1742` / `main.rs:1779-1790`** —— 解构加 `host`，并把字面量换成变量：

```rust
            Commands::Daemon {
                port,
                host,
                client,
                idle_timeout,
            } => {
                …
                let res = rustcode_daemon::run_server(rustcode_daemon::ServerOpts {
                    host,                       // ← 替换原 `host: "0.0.0.0".to_string()`
                    port,
                    …
```

### 4.3 帮助文案（两语种，挂载点）

| 语种 | 确切文案 | 挂载点 |
|---|---|---|
| en | `Bind address (default: 0.0.0.0; use 127.0.0.1 for local-only)` | `crates/rustcode-config/src/i18n/en.rs:2521`，已有，不动 |
| zh_cn | `绑定地址（默认：0.0.0.0；改 127.0.0.1 则仅本机可访问）` | `crates/rustcode-config/src/i18n/zh_cn.rs:2429`，已有，不动 |
| 变体 | `Msg::CliHelpHost` | `crates/rustcode-config/src/i18n/messages.rs:3916`，已有，不动 |
| 注入 | `.mut_arg("host", \|a\| a.help(t(Msg::CliHelpHost).into_owned()))` | `crates/rustcode-cli/src/main.rs:467-473`（`daemon` 子命令闭包内） |

- **复用关系**：`daemon` 与 `webui`（`main.rs:477`）**共用同一个 `Msg::CliHelpHost` 变体与同一条 `mut_arg` 表达式**，
  因此 `rustcode daemon --help` 与 `rustcode webui --help` 的 `--host` 行**逐字相同**，两语种自动一致。
- **禁止**新增 `Msg::CliHelpHostDaemon` 之类的重复变体；**禁止**改 `en.rs` / `zh_cn.rs`。
- 无 Emoji（文案与既有实现均为纯 ASCII / 汉字 + 全角括号），符合禁 Emoji 纪律。

### 4.4 冻结不变项

- `ServerOpts`（`crates/rustcode-daemon/src/lib.rs:6057-6087`）**契约不变**，字段、类型、注释一字不动。
- `rustcode_daemon::run_server` / `ensure_server_and_open` 签名不变。
- `is_loopback_bind_host`（`lib.rs:1300`）、`is_loopback_authority`（`lib.rs:1272`）不变。
- `crates/rustcode-daemon/src/main.rs:21` `DEFAULT_HOST = "127.0.0.1"` 不变（`AGENTS.md:38`）。
- `rustcode-tuix` 的 `/webui` 默认 `127.0.0.1`（`AGENTS.md:38` 引用 `commands.rs:2207`）不变。

---

## 5. 状态所有权

- **bind 地址的单一所有者仍是 driver**：`ServerOpts.host` 由调用方填入，`run_server` 只消费不决策
  （`lib.rs:6058-6059` 注释："Decided by the driver; this function does not warn on non-loopback binds"）。
  本次只是把 `rustcode daemon` 这个 driver 的"硬编码"升级为"用户可指定的值"，**所有者数量不变**。
- **不新增第二运行时生命周期所有者**：`rustcode daemon` 仍调用同一个 `rustcode_daemon::run_server`
  （`main.rs:1779`），没有新增 `tokio::spawn`、没有复制 server 生命周期、没有第二个 idle watchdog。
- **只读方**：`AppState.bind_host` 是 `run_server` 内部的只读派生值，被 `client_interactive_permission`（`lib.rs:1307-1317`）
  与 `GET /tunnel/status`（`lib.rs:5697`）读取。
- **跨进程协议**：本次**不改** `~/.rustcode/daemon-<port>.json` 的内容与写入时机（`lib.rs:6513-6521`），
  不触及任何跨进程 wire format。

### 5.1 已核对的两处"绑定地址影响行为"（重要，均不构成安全回退）

1. **交互式审批权限**（`client_interactive_permission`，`lib.rs:1307-1317`）：
   该入口 `webui_tokens: Some(token_store)`（`main.rs:1784`）⇒ `enforce_token = webui_tokens.is_some()` = `true`（`lib.rs:6177`）
   ⇒ 函数首个条件 `enforce_token` 恒为 `true`，**与 `bind_host` 无关**。
   ⇒ `--host 127.0.0.1` **不会**改变该入口的审批权限面（改前 `0.0.0.0` 也是同一结果）。
2. **`GET /tunnel/status` 的 `reachable`**（`lib.rs:5697`）：`!is_loopback_authority(&state.bind_host)`。
   `--host 127.0.0.1` ⇒ `reachable = false`（此前 `0.0.0.0` ⇒ `true`）。这是**符合意图的正向效果**：
   仅本机绑定时远程访问面板不应宣称可达。

---

## 6. 失败与取消语义

| 输入 | 处理层 | 行为 | 退出码 | 依据 |
|---|---|---|---|---|
| `--host` 缺值（`rustcode daemon --host`） | **clap 解析期** | 打印 clap 的用法错误（`a value is required for '--host <HOST>' but none was supplied`）并退出 | **2** | clap 默认；`webui` 同参同行为（均无 `value_parser`） |
| `--host ""`（空串） | 运行期 | `addr = ":13456"` ⇒ `TcpListener::bind` 失败 ⇒ stderr `致命错误：无法绑定到 :13456：…` | **1** | `lib.rs:6370` + `lib.rs:6498-6509`；`Msg::DaemonFatalBind` `zh_cn.rs:2994` |
| `--host 1.2.3.4:80`（含端口） | 运行期 | `addr = "1.2.3.4:80:13456"` ⇒ bind 失败，同上 | **1** | 同上 |
| `--host 999.999.999.999` / `--host not-a-host`（非法 IP / 未知主机名） | 运行期 | bind / 解析失败，同上 | **1** | 同上 |
| `--host localhost` | — | **可达的合法输入**：`is_loopback_authority` 显式认 `localhost`（`lib.rs:1278`）⇒ 绑定成功且**不告警** | 0 | `lib.rs:1278`、`lib.rs:6388` |
| `--host ::1` | — | 绑定成功；`is_loopback_bind_host("::1") == true`（`lib.rs:1301`）⇒ **不告警**。走 `run_server` 直连路径，不经 `ensure_server_and_open`，故**不产生** RI-1 的非法 URL | 0 | `lib.rs:1301`、`lib.rs:6388`；RI-1 只影响 `ensure_server_and_open`（`STATUS.md:121`） |
| `run_server` 返回 `Err`（非 bind 失败） | 运行期 | stderr `Msg::CliDaemonFatal`（`致命错误：守护进程服务器错误：{error}`）+ `return Ok(1)` | **1** | `main.rs:1792-1800`；`zh_cn.rs:850` |
| Ctrl-C / SIGTERM | 运行期 | 既有 graceful shutdown，行为不变 | — | 本轮不触碰 |

**判据（为什么是"运行期报错"而不是"解析期拒绝"）**：
`webui` 的同名参数在 `main.rs:1047` 只有 `#[arg(long, default_value = "0.0.0.0")]`、**无 `value_parser`**，
clap 只拒绝"缺值"（exit 2）、不校验内容；本改动要求与 `webui` 完全一致，故 daemon 同样
**不加 `value_parser`**、同样由运行期 bind 失败兜底。**不为 daemon 单独加严** —— 那会让两个入口分叉。

**fail-closed 保证**：bind 失败发生在 `lib.rs:6496`，而 `~/.rustcode/daemon-<port>.json` 的写入在
`lib.rs:6513-6521`（bind 之后）。因此失败时**既无监听 socket、也无 token 文件**，
不会留下"扩展读到一个 token 但服务没起来"的假成功状态。

**quiet 模式**：`rustcode daemon` 恒传 `quiet: false`（`main.rs:1785`），无开关，故非回环提示恒打印；
**不存在**"quiet 下是否提示"的分支。

**禁止静默 fallback**：实现中**不得**出现"host 非法就回退 `0.0.0.0`"或"host 为空就用默认"之类的兜底；
非法值必须走到上表的 exit 1。

---

## 7. 迁移与回退

**数据格式兼容**：本改动不读写任何持久化数据，不涉及格式迁移。

**前向/后向兼容**：
- 新 CLI 收到**不带** `--host` 的旧调用（JetBrains `RustCodeDaemonProcess.kt:113-126`、
  VSCode `process.ts:369`）：默认值 `0.0.0.0` 与改前硬编码完全等价 ⇒ **零行为变化**。
- 旧 CLI 收到**带** `--host` 的新调用：clap exit 2 ⇒ 将来扩展若要传 `--host`，必须做版本门控或容错
  （本轮不改 `extensions/**`，见 §8 影响面 I-6）。

**回退步骤**（改动集中在 1 个文件、3 处、1 个 commit）：
1. `git revert <T-15 的 commit>`；
2. 或手工回退三处：删 `Commands::Daemon` 的 `host` 字段、删 `.mut_arg("host", …)` 一行、
   把 `ServerOpts { host` 改回 `host: "0.0.0.0".to_string()`。
   **三处必须同时回退**（`§1.4` 的 panic 耦合）。
3. 回退后验证：`cargo run -p rustcode -- daemon --help` 中 `--host` 行消失、且命令不 panic。

---

## 8. 架构边界核对（逐条，对照 `AGENTS.md`）

| # | 约束 | 本方案 | 结论 |
|---|---|---|---|
| 1 | 依赖只向下（`AGENTS.md:56`）；`cli` 是唯一同时依赖 tuix + daemon 的 driver | 只改 `rustcode-cli`，**零新增 crate 依赖**，不新增 `use` | ✅ |
| 2 | `rustcode-kernel` / `capabilities` / `coding` 生产依赖 core-free | 未触碰这些 crate | ✅（不涉及） |
| 3 | 禁止 capabilities 反向依赖 core / L2 / 前端 | 无 | ✅（不涉及） |
| 4 | 历史 core JSON 只允许 daemon 私有 DTO 单向导入；禁止恢复 legacy writer / core 磁盘投影 / 双向转换 | 无 | ✅（不涉及） |
| 5 | native `SessionManager/SessionMeta/SessionSnapshot` 是唯一 session 持久化模型 | 未触碰 | ✅（不涉及） |
| 6 | 不新增第二运行时生命周期所有者；目标调用链 `CLI → … → run_server` | 仍调用同一 `rustcode_daemon::run_server`（`main.rs:1779`），无新 spawn / 新 watchdog | ✅ |
| 7 | 禁止 bridge / fallback / v1-v2 兼容开关 | 无开关；非法 host 一律 exit 1，**无静默回退默认值** | ✅ |
| 8 | `AGENTS.md:37` 的 Q2-A 默认值 | 默认值仍 `0.0.0.0`，与硬编码等价 | ✅（并需同步该行文字，见 I-1） |
| 9 | `AGENTS.md:38` 独立二进制与 TUI `/webui` 保持 `127.0.0.1` | 未改动 `crates/rustcode-daemon/src/main.rs:21` 与 `commands.rs:2207` | ✅ |
| 10 | `AGENTS.md:39` 非回环提示判据 = `is_loopback_bind_host`；`Msg::DaemonWarnNonLoopback` 保留但不复活 | 未改判据、未改 `lib.rs:6381-6394`、未引用 `DaemonWarnNonLoopback` | ✅ |
| 11 | 涉及 turn completion / compaction 时先复核 `LifecycleHooks::turn_complete`，不新增重叠 hook 或第二压缩状态机 | 不涉及 | ✅（不涉及） |
| 12 | 出站 HTTP 唯一入口（`AGENTS.md:55`） | 无新增出站调用 | ✅（不涉及） |
| 13 | 门禁 `python3 scripts/check-zh-docs.py gate` 只判定全仓 `md` | 本次改 `.rs`（含英文 doc comment），**不进门禁分母**；且未新增任何 `md` 之外的受检内容 | ✅ |

**结论：本方案不破坏任何 `AGENTS.md` 架构约束，`touches_runtime_lifecycle / touches_persistence / touches_cross_crate_deps` 全为 `false`。**

---

## 9. 影响面清单

| # | 文件 / 位置 | 需做 | 归属 | 说明 |
|---|---|---|---|---|
| **I-1** | `crates/rustcode-cli/src/main.rs:1026-1038`、`:467-473`、`:1738-1742`、`:1779-1790` | **改**（3 处，见 §4.2） | `code-implementer`（T-15） | 唯一代码改动文件；必须同 commit |
| **I-2** | `AGENTS.md:37` | **改**（删掉"**rustcode daemon 子命令没有 --host 参数**，该入口无法显式改回 127.0.0.1"，改为"`rustcode daemon --host` 默认值同为 `0.0.0.0`，两个入口都可显式改回 `127.0.0.1`"；行内引用的 `main.rs:1780` 需同步为新行号或改为引用 `main.rs` 的 `host` 字段） | `project-manager`（SA 本轮 `files_owned` 不含 `AGENTS.md`；`02-tasks.md §0.3` 规则 4 亦规定 `AGENTS.md` owner = SA/PM/TE） | 全仓 `md` 中**唯一**记载"daemon 无 `--host`"的句子（全量 `grep -- '--host' *.md` 已核） |
| **I-3** | `.codebuddy/artifacts/2026-09-07-zh-docs-webui/STATUS.md:115`（D-5 行）与 `:139`（下轮建议第 2 条） | **改**（D-5 状态 → 已修；第 139 条标注已落地） | `project-manager` | 与 I-2 同批 |
| **I-4** | `crates/rustcode-cli/tests/shell_completion.rs` | **不改，只验证** | `test-engineer`（T-15 验证项） | 全文 21 行，**无 flag/快照断言**；`completion_command()` 虽源出 `build_i18n_command()`（`main.rs:1196-1215`），但本测试不断言其内容 ⇒ **shell completion 快照不受影响** |
| **I-5** | `crates/rustcode-config/src/i18n/{messages,en,zh_cn}.rs` | **不改**（复用 `Msg::CliHelpHost`） | — | 新增变体会触发 `cargo test -p rustcode-config --lib`（`AGENTS.md:263` 的 i18n 内容测试）基线变化，故刻意复用 |
| **I-6** | `extensions/jetbrains/.../RustCodeDaemonProcess.kt:113-126`、`extensions/vscode/src/daemon/process.ts:369` | **本轮不改**，仅预留 | 下轮 PM 派单 | 预留落点：JetBrains 在 `args += listOf("--port", …, "--client", "jetbrains")` 之后追加 `"--host", "127.0.0.1"`；VSCode 把 `portArgs` 扩成 `['--host', '127.0.0.1', '--port', …, '--client', …]`。**前置条件**：需处理"旧 CLI 不认 `--host` ⇒ exit 2"的版本门控 |
| **I-7** | `crates/rustcode-daemon/src/main.rs:21`（独立二进制 `DEFAULT_HOST`）与 `crates/rustcode-daemon/README.md:35` | **不改** | — | `AGENTS.md:38` 明令不得统一默认值 |
| **I-8** | `README.md:120`（"默认绑定 0.0.0.0 —— …显式改回 `127.0.0.1` 即仅本机可访问"） | **不改**（改后该描述对两个入口都成立，此前对 daemon 才是失实的） | — | 无新增失实叙述 |
| **I-9** | `docs/**` | **不改** | — | 全量 `grep` 已核：`docs/` 中无"daemon 无 `--host`"表述（`docs/testing/release-v5.0.0-acceptance.md:20/215` 只说"启动 `rustcode daemon`"，不受影响） |
| **I-10** | `crates/rustcode-daemon/src/lib.rs` | **不改** | — | `ServerOpts` / `run_server` / 谓词全部冻结不变 |

---

## 10. 验收标准（AC，6 条，均可自动化或半自动复现）

> 统一前置：`export RUSTCODE_HOME=$(mktemp -d)` 以隔离真实 `~/.rustcode`；
> `cargo build -p rustcode` 后以 `./target/debug/rustcode` 执行（下写作 `rustcode`）。

- **AC-D5-1（参数存在 + help 两语种 + 与 webui 一致）**
  `rustcode daemon --help` 输出含 `--host <HOST>`；在默认 locale（zh_cn）下含
  `绑定地址（默认：0.0.0.0；改 127.0.0.1 则仅本机可访问）`；
  且 `rustcode daemon --help` 与 `rustcode webui --help` 中 `--host` 那一行的 help 文本**逐字相等**；
  `rustcode --lang en daemon --help` 含 `Bind address (default: 0.0.0.0; use 127.0.0.1 for local-only)`。

- **AC-D5-2（默认值不回退，行为与基线一致）**
  不传 `--host` 启动：`rustcode daemon --port <PORT>` 的 stdout 含
  `RustCode API 服务已启动，监听地址 http://0.0.0.0:<PORT>`，且**打印** `Msg::WebuiLanWarning`
  （含 `主地址为局域网 IP`）。⇒ 与改前硬编码 `0.0.0.0` 完全等价。

- **AC-D5-3（显式回环生效且不告警）**
  `rustcode daemon --host 127.0.0.1 --port <PORT>`：stdout 含
  `监听地址 http://127.0.0.1:<PORT>`；stdout **不含** `主地址为局域网 IP`、**不含** `已绑定非回环地址`；
  且 `curl -sSf http://127.0.0.1:<PORT>/health` 返回 200。

- **AC-D5-4（解析期拒绝缺值）**
  `rustcode daemon --host` ⇒ 退出码 **2**，stderr 含 clap 的
  `a value is required for '--host <HOST>' but none was supplied`。

- **AC-D5-5（运行期 fail-closed）**
  `rustcode daemon --host "" --port <PORT>` 与 `rustcode daemon --host 1.2.3.4:80 --port <PORT>`：
  退出码 **1**，stderr 含 `致命错误：无法绑定到`；
  且 `$RUSTCODE_HOME/daemon-<PORT>.json` **不存在**（无监听、无 token 文件，无假成功）。

- **AC-D5-6（既有测试与 lint 不回退）**
  `cargo fmt --check` 通过；`cargo clippy --workspace --all-targets` 无新增告警；
  `cargo test -p rustcode-cli --test shell_completion` 通过（I-4）；
  `cargo test -p rustcode-config --lib` 通过（无新增 `Msg` 变体，I-5）；
  `cargo test -p rustcode-daemon --lib channel_mode_tests` 通过（`AGENTS.md` 的 AC-29 面）；
  `python3 scripts/check-zh-docs.py gate --base 3ee655e3` 仍 exit 0（本次未改受检 `md`）。

---

## 11. 风险与开放问题

- **R1（高，已用设计消解）**：`mut_arg` 对不存在的 arg id 无条件 panic（§1.4）。
  ⇒ 消解方式：3 处改动收敛为**单文件、单任务、单 commit**，并在 T-15 验收中把
  `rustcode --help` / `rustcode daemon --help` / `rustcode completion bash` 三者都跑一遍防 panic。
- **R2（中，报 PM 裁决）**：`daemon` 与 `webui` 的**绑定失败退出码不一致**（daemon 1 / webui 0，§1.6）。
  属既有行为，本轮**不统一**；是否另派单把 `webui` 的 bind 失败也改为非 0，需 PM 裁决。
- **R3（低，已知不加剧）**：D-4（`Msg::WebuiLanWarning` 的"主地址为局域网 IP"在 `run_server` 路径下
  只打印 `0.0.0.0`，`STATUS.md:114`）。`--host 127.0.0.1` 时不触发该文案，故本改动**不加剧** D-4；
  默认路径下的失实表述仍待 architect 对 P1/P2/P3 裁决（`STATUS.md:142`）。
- **R4（中，下轮前置）**：扩展侧传 `--host` 的**前向兼容**（旧 CLI exit 2，I-6）。本轮不改 `extensions/**`。
- **R5（低，已核对无影响）**：D-1 本体 / RI-1（`STATUS.md:111/121`）不受本改动影响：
  `rustcode daemon` 不走 `ensure_server_and_open`，故不产生 `http://::1:PORT/`；
  `--host ::1` 时 `is_loopback_bind_host` 返回 true（`lib.rs:1301`）⇒ 正确静默。
  另据 §5.1，`client_interactive_permission` 因 `enforce_token=true` 而与 bind host 无关，
  故 `--host 127.0.0.1` **不扩大**审批权限面。
- **R6（低）**：`--host localhost` 是可解析主机名（`lib.rs:1278` 显式认它），依赖本机 DNS/hosts；
  属既有 `webui` 行为，本轮保持一致，不改。
