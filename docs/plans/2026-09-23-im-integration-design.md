# RustCode IM 接入可行性与实施方案

状态: DRAFT (评审中)
日期: 2026-09-23
范围: 新增 `rustcode-im`(IM 适配器) + `rustcode-agent-api`(编程式 API)
关联: `docs/plans/2026-09-23-continuous-agent-design.md`（IM 可作为持续工作的通知面与触发源）

---

## 0. 结论

**可实现，且本仓已具备大部分地基。** 但有三个硬约束决定了架构必须长什么样，
若忽略它们，方案会在实现到一半时撞墙（详见 §2）。

一句话架构判断：

> IM 接入的**正确落点不是 daemon 的 `/live` SSE**，而是
> **「每用户一个 `CodingRuntime` + 非流式一问一答」**。
> 前者是交互式协议（单绑定、逐字流、为浏览器设计），后者才是 IM 的天然形状。

支撑这个判断的三条实测事实：

| 事实 | 证据 | 含义 |
|------|------|------|
| `HubState.binding` 是 **`Option<BoundRuntime>`**（单绑定） | `daemon/src/live_hub.rs:120` | 一个 daemon 同时只承载**一个** live runtime；多用户会互相踢掉 |
| `ClientMode::Channel` **已存在**且已被计入可交互权限 | `daemon/src/client_mode.rs:34`（wire tag `"channel"`）；`daemon/src/lib.rs:1305` | IM 桥接的钩子**是预留好的**，非新造概念 |
| headless 已支持**一次性捕获完整回答**与**会话续接** | `run_native_headless(...) -> Result<(i32, Option<String>)>`（`cli/src/main.rs:3731/4223`，`capture` 累积）；`-c/--continue`、`--resume <ID>`（`:898/:904`） | IM 最需要的"一问一答 + 记得上下文"**已具备** |

**最重要的重构建议（避免重复造轮子）**：
`docs/agent-api-rfc.md` 已设计的 `rustcode-agent-api`（task-oriented 编程式 API）
**就是 IM 适配器需要的东西**。故 IM 接入应**建立在 agent-api 之上**，
而不是为 IM 另造一套 runtime 装配。见 §5.0。

---

## 1. 现状盘点（带证据）

### 1.1 三面 seam 齐备

| 面 | 落点 | 现状 |
|----|------|------|
| **入站** | `daemon/src/lib.rs:6441-6538` | 47 条路由，全部面向交互式客户端（webui / IDE / TUI）。**无任何 IM 回调路由** |
| 主交互通道 | `/live` SSE + 12 条 `/live/*` 命令 | 为 WebUI 设计的流式协议；`HubState.binding: Option<..>` 单绑定 |
| **同步命令通道** | `POST /command` → `CommandResult`（`commands.rs:41/682`） | **非流式**，一次返回 JSON。IM 可用的现成形状 |
| **出站** | `capabilities/src/egress/client.rs:192` `build_http_client` | 唯一工厂（信任根/代理/超时/UA）；**新增出站必须走它** |
| 出站通知 | `capabilities/src/notify.rs` | **仅终端/系统/bell**（`NotificationConfig` 的 `terminal`/`system`/`bell`），**无远端通道** |
| **鉴权** | `webui_tokens` + `enforce_token` + `access_key` | Bearer Token；`/tunnel` 刻意**不受 `no_auth` 影响**（远程接入始终要 token） |
| **公网暴露** | `crates/rustcode-tunnel/`（反向隧道中继） | `RUSTCODE_ENABLE_TUNNEL=1` 才开；`HOSTED_RELAY_ENABLED = false`（`endpoints.rs:107`）默认关闭 |
| 编程式驱动 | `acp/`（14 个模块：`dispatch`/`turn`/`permission`/`elicitation`…） | 已有的"外部程序驱动 agent"通道，但协议是 ACP，非 IM |

### 1.2 IM 的历史痕迹（说明这条路曾被走过）

搜索发现本仓上游曾有 **WeChat clawbot / OpenClaw 桥接**，且相关修复是真实的运维问题：

- `capabilities/src/process_utils.rs:5` —— "clawbot/OpenClaw) that spawns a console program… makes Windows allocate a **fresh** console window… **it flashes on the desktop every turn**"
- `capabilities/src/tools/bash.rs:242` —— "in headless/daemon mode (e.g. **the WeChat clawbot bridge**) there's no console to inherit"
- `capabilities/src/tools/mod.rs:249` —— "Critical in headless/daemon mode (e.g. the **WeChat clawbot / OpenClaw bridge**)… the '**一对话桌面就闪**' flash the user reported"

**这三点是重要情报**，不是无关注释：
1. 说明 IM 桥接**以 headless/daemon 模式运行**（印证 §2.3 的 `ClientMode` 判断）；
2. 说明 Windows 下的 console-window 闪烁是**必踩的坑**，且修复已存在
   （`suppress_console_window` / `CREATE_NO_WINDOW`）——复用即可，不要重踩；
3. 说明该形态**曾真实部署**，不是纸上推演。

### 1.3 移动端 App 的教训（已移除，需引以为戒）

`docs/features.md:188`：

> **本能力取代已移除的移动端 App 远程访问**（原 `/app <中继>` + `RUSTCODE_APP_RELAY`
> 方案，随移动端 App 一并移除）：隧道是平台中立、用户自托管中继的反向代理，
> **无任何托管账号或移动端依赖**。

**这条对 IM 接入是直接的约束**：那个方案被移除的原因据文档口径是"托管账号 + 移动端依赖"。
因此 IM 接入**必须**满足：

- **无托管账号**：不引入任何"RustCode 官方 IM 机器人"这类中心化服务；
- **用户自带凭据**：IM 的 bot token 由用户在自己的平台上申请（BYO），与 provider
  api_key 同一性质；
- **可自托管**：适配器是本地进程，可连自建网关。

若把 IM 做成"官方托管机器人"，就是重走被移除的老路。

### 1.4 配置面与 hook 面

| 项 | 现状 |
|----|------|
| config 的 IM/bot 段 | **无**（`grep` 无 `bot`/`channel`/`telegram`/`wecom` 等字段） |
| 入站 webhook 路由 | **无**（47 条路由里没有 callback/webhook/bot 形态） |
| `cc_hooks` | 只支持 **shell 命令**（`shell_command`，`cc_hooks.rs:276`），**不是 webhook** |
| 凭据保护 | `CredentialBashGate`（`tools/credential_bash_gate.rs`）拦截 bash 里的凭据字面量与 `*_WEBHOOK_URL` 读取 |

最后一行有直接后果：**IM 的 token 不得经由 bash 触碰**，必须走 config 的 `env:VAR`
展开路径。且 `CredentialBashGate` 已把 `_webhook` / `_webhook_url` 结尾的变量名
列入敏感模式（`credential_bash_gate.rs:114-115`）——IM 桥接的 webhook URL 天然命中，
这是**正确的**，不要为图方便去放宽该闸门。

### 1.5 参考实现对照：AgentCore 的 IM 渠道模型

以"AgentCore 控制台配置 IM 渠道接入钉钉/飞书/企业微信"为参照，逐条对照本仓：

| AgentCore 的前提 / 步骤 | 本仓对应物 | 可映射性 |
|------------------------|-----------|---------|
| 已在 AgentCore 创建托管 Agent（Worker） | 本地 `CodingRuntime`，**无"托管 Agent"概念** | 需改造：以 project 为 Agent 身份 |
| 默认 ServiceEndpoint 处于 READY 且**支持公网访问** | 本地进程，**默认无公网入口** | **根本差异（见下）** |
| 在 AgentCore 控制台配置 IM 渠道 | 无控制台 → 落 `config.toml` 的 `[im]` 段 | 可映射 |
| Workspace 创建/更新/删除时只能查看、不能保存 | 无 Workspace 生命周期概念 | 不适用 |
| 在 IM 平台创建机器人并获取凭证 | 同（用户自建应用） | **完全可映射（BYO）** |
| 用户单聊/群聊发消息 → 转发给 Agent → 回复回当前会话 | 同 | 可映射 |
| 每个 Agent 在每个 IM 平台**最多配置一个**机器人渠道（可同时配钉钉+飞书+企业微信） | 每 project 每平台一个绑定 | 可映射 |
| 注册或本地纳管的 Agent 不展示"外部渠道配置"入口 | 无此区分 | 不适用 |

**收敛：6 项可映射、1 项根本差异、2 项不适用。**

**那 1 项根本差异，一句话说清**：

> AgentCore 要求 Agent 具备**公网可达的托管 ServiceEndpoint**，
> 因为它的入站形态是 **webhook 回调**（平台推送到该端点）。
> 本仓是本地 CLI 进程，**没有、也不应该假设**公网入口。

**但这不是障碍，反而是本仓可以做得更好的地方** —— 见 §1.6。

### 1.6 三平台接入机制调研（决定是否真的需要公网）

| 平台 | 凭证（AgentCore 口径） | 入站机制 | 是否需要公网 |
|------|----------------------|---------|-------------|
| 钉钉 | Client ID + Client Secret | **Stream 模式（WebSocket 长连接）** | **不需要** |
| 飞书 | App ID + App Secret | 长连接（WS）或 webhook 事件订阅 | 长连接不需要 |
| 企业微信 | Bot ID + Secret | 回调 URL（webhook） | **需要** |

**钉钉的证据（一手，已证实）**：官方 `dingtalk-stream` SDK README 明示——

> 钉钉支持 **Stream 模式**接入事件推送、机器人收消息以及卡片回调……
> **相比 Webhook 模式，Stream 模式可以更简单的接入各类事件和回调。**

其示例用 `DingTalkStreamClient(credential)` + `client.register_callback_handler(...)`
+ `client.start_forever()`，凭证正是 **Client ID / Client Secret**（`app_key`/`app_secret`），
与 AgentCore 表格口径一致。这是 **WebSocket 长连接**，客户端主动外连，**无需任何公网入口**。

**置信度诚实标注（重要）**：

| 平台 | 结论 | 证据强度 |
|------|------|---------|
| 钉钉 | 支持 Stream 长连接，无需公网 | **已证实**（官方 SDK README 原文 + 示例代码） |
| 飞书 | 高置信支持长连接 | **未取得一手证据** —— 官方 SDK 的 WS 源码路径逐个 404，GitHub API 触发限流。飞书文档站为纯 JS 渲染，抓取无正文 |
| 企业微信 | 高置信为 webhook 回调（需公网） | **未验证** —— 同理未取得一手材料 |

> **[待办] 实施 P-IM1 前必须补齐飞书与企业微信的一手文档核实**，
> 不得依据"高置信"直接进入实现（本仓铁律：不臆断库/平台 API）。

**本节结论（对 AgentCore 模式的关键修正）**：

AgentCore 的"ServiceEndpoint 必须支持公网访问"是其**选择了 webhook 通用形态**的结果，
不是 IM 接入的固有要求。钉钉已证明可以纯长连接接入。因此本仓应：

1. **首选长连接平台**（钉钉已证实）—— 零公网依赖、零隧道依赖；
2. **webhook 平台（企业微信）作为后续**，届时复用既有 `rustcode-tunnel`
   （`RUSTCODE_ENABLE_TUNNEL=1` + `RUSTCODE_TUNNEL_RELAY`，`endpoints.rs:107`
   默认 `HOSTED_RELAY_ENABLED = false`），且必须配 `Authorization: Bearer <access_key>`。

这比照搬 AgentCore 的"必须有公网端点"更适合本仓的本地/自托管定位。

### 1.7 渠道配置模型（采用 AgentCore 的约束）

AgentCore 的"**每个 Agent 在每个 IM 平台最多配置一个机器人渠道**"是良好约束，
本仓采纳，映射为：

```text
(project, platform) 唯一确定一个渠道绑定
    project  = Agent 身份（本仓无托管 Agent，用工作目录代表）
    platform = "dingtalk" | "feishu" | "wecom"
```

即：同一 project **可以同时**配钉钉 + 飞书 + 企业微信三个渠道，
但**不能**配两个钉钉渠道。配置结构（`config.toml`）：

```toml
[im]
enabled = false                      # 总开关，默认关闭（对齐 RUSTCODE_ENABLE_TUNNEL 的 gate 风格）

[im.dingtalk]
enabled = true
client_id = "env:DINGTALK_CLIENT_ID"         # BYO，走 env:VAR 展开
client_secret = "env:DINGTALK_CLIENT_SECRET"
project = "/abs/path/to/workdir"

[im.feishu]
enabled = false
app_id = "env:FEISHU_APP_ID"
app_secret = "env:FEISHU_APP_SECRET"
project = "/abs/path/to/workdir"

[im.wecom]                           # 需公网/隧道，见 §1.6
enabled = false
bot_id = "env:WECOM_BOT_ID"
secret = "env:WECOM_SECRET"
project = "/abs/path/to/workdir"
```

**三条硬约束**：

1. **凭证一律 `env:VAR` 展开**，不得明文写进 config（对齐仓库 `[SECURITY]` 约束与
   provider `api_key` 的既有做法：支持 `$VAR` / `${VAR}` / `${VAR:-default}`）。
2. **总开关默认 `false`** —— 对齐 `HOSTED_RELAY_ENABLED = false` 的 gate 风格，
   保证未启用时零行为变化。
3. **每 `(project, platform)` 唯一** —— 启动时校验，重复即拒绝启动并报错
   （不静默取最后一个，那会让用户以为配了 A 实际跑的是 B）。

---

## 2. 三个硬约束（决定架构，不可绕）

### 2.1 约束一：`live_hub` 是单绑定 —— 不能用 `/live` 承载多用户

```rust
// daemon/src/live_hub.rs:117-129
struct HubState {
    next_binding_id: u64,
    binding: Option<BoundRuntime>,      // <- 单绑定！不是 HashMap
    snapshot: Option<Arc<SessionSnapshot>>,
    ...
    turn_active: bool,
}
```

`bind_with_provider`（`:225`）在 `state.turn_active` 为真时直接返回 `Err(HubError::ActiveTurn)`。

**后果**：若 IM 适配器直接复用 `/live/message`，则：

- 用户 A 与用户 B 同时发消息 → 第二个被 `ActiveTurn` 拒绝，或**踢掉**前一个绑定；
- 这在 1 对 1 的浏览器场景是对的（一个用户一个页面），在 IM 的**多用户群聊**场景是错的。

**结论**：IM 接入**不得**共用 `/live` 的单一 hub 实例。必须走"每用户独立 runtime"
（见 §3）。

### 2.2 约束二：多用户需要"IM 用户 → 会话"映射，这是**新增持久层**

`SessionManager` 的索引维度是**项目/工作目录**，不是用户：

```rust
pub fn for_project(working_dir: &Path) -> Self        // manager.rs:968
pub fn with_root(root: impl Into<PathBuf>) -> Self    // :982
// 会话列表：list() / list_visible()，按 updated_at 排序
```

`SessionOrigin`（`:357-369`）只有 `Manual` / `Scheduled` 两个变体，
**没有"来自 IM"这一维度**（也无法区分不同 IM 用户）。

**需要新增**：

1. `SessionOrigin::Im`（或 `Channel`）变体 —— 让 IM 会话与手动/定时会话可区分；
2. 一张 **IM 身份映射表**：`(platform, chat_id/user_id, project) -> session_id`。
   这是全新的持久层，落 `rustcode-config`（与 `schedule.rs` 同性质）。

**否则会发生什么**：同一 IM 用户的连续对话会各自新建会话，Agent 失忆；
或所有 IM 用户共用同一会话，互相看到对方的内容（**隐私事故**）。

### 2.3 约束三：IM 是非流式的（或流式体验完全不同）

`/live` 的协议是 SSE 逐字流（`LiveWireEvent::TextDelta`），而 IM 的形态是：

- 多数平台**不支持编辑已发消息**（或限时/限量），不 streaming；
- 典型交互是"发一条 → 等一会 → 收到一条完整回答"；
- 长回答需**分片**（平台有单条字数上限）。

**关键使能点（已存在，无需新造）**：

```rust
// cli/src/main.rs:3731 / :3771 / :3856 / :4223
pub(crate) async fn run_native_headless(...) -> Result<(i32, Option<String>)> {
    let captured = capture.then(String::new);   // capture=true 时累积完整文本
    ...
    if let Some(buffer) = captured.as_mut() { /* 逐段追加 */ }
    Ok((exit_code, captured))                    // 一次性拿到完整回答
}
```

配合 `-c / --continue`（`:898`）与 `--resume <ID>`（`:904`）做会话续接，
以及 `HeadlessOutputFormat::{Text, Jsonl}`（`:1009-1013`）选择输出形态。

**结论**：IM 的"一问一答 + 记得上下文"**不需要新机制**，直接复用 headless 的
`capture` + `--resume` 即可。这是本方案最大的省力点。

---

## 3. 目标架构

### 3.1 关键发现：daemon 有**两条**路径，只有一条是单绑定

这是本方案最重要的架构判断，必须说清，否则会选错落点：

| 路径 | 状态模型 | 并发能力 | 适合 IM？ |
|------|---------|---------|----------|
| `/live/*`（SSE） | `HubState.binding: Option<..>` | **单绑定**（`ActiveTurn` 拒绝） | **否** |
| `/chat/*` + `/command` | `ActiveChatRegistry { operations: HashMap, aliases: HashMap }` | **多会话**（按 session_id 别名单飞准入） | **是** |

证据：`daemon/src/lib.rs:401-411` 的 `ActiveChatIndex` 是 **HashMap**，
注释明写「Atomic admission and identity-aware cleanup for background `/chat` turns」、
「makes **single-flight admission** and compare-by-operation cleanup indivisible」，
并有 `SessionBusy` / `RequestBusy` 两种按会话/按请求的准入拒绝（`:422-437`）。

**这意味着**：daemon 早已为"多个后台会话并发"做好了准入控制，
IM 接入应**建立在 `/chat` 语义之上**（或直接复用 `ActiveChatRegistry`），
而**不是**去改造 `/live` 的单绑定 hub。

```text
IM 平台 (Telegram / Slack / 飞书 / 企业微信 ...)
        |  (1) 入站：webhook 回调 或 长轮询
        v
+---------------------------------------------------+
|  rustcode-im（新 crate，L3 driver）                |
|  - 每平台一个 Adapter（trait，见 §4.1）             |
|  - 身份映射： (platform, chat_id) -> session_id    |
|  - 出站长轮询/回发 走 build_http_client？见 §4.4    |
|  - 分片、去重、限流、幂等                            |
+------------------------+--------------------------+
                         | (2) 内部调用（非 SSE）
                         v
+---------------------------------------------------+
|  rustcode-agent-api（新 crate，见 §5.0）           |
|  - TaskManager: 任务队列 + 状态机 + 准入            |
|  - 每任务一个独立 CodingRuntime（单 owner）         |
|    => 避开 §2.1 的单绑定约束                        |
+------------------------+--------------------------+
                         | 复用既有装配链
                         v
        CodingRuntime（既有，唯一 owner）
                         |
                         v
        rustcode-kernel Agent（零改动）
```

### 3.2 分层归属

| 新组件 | 落点 | 层 | 理由 |
|--------|------|----|------|
| `ImIdentityMap`（身份映射持久化） | `rustcode-config` | leaf | 纯数据 + 文件存储，与 `schedule.rs` 同性质 |
| `SessionOrigin::Im` 变体 | `rustcode-capabilities` | L1 | 与既有 `SessionOrigin` 同处（`manager.rs:357`） |
| `Adapter` trait + 各平台实现 | `rustcode-im`（新） | L3 | driver 职责；`cli` 依赖它，或做成独立 bin |
| 编排（每用户 runtime 生命周期） | `rustcode-agent-api`（新） | L3 | 对齐 `agent-api-rfc.md` 既有设计 |
| IM 只读/审批转发 | `rustcode-daemon`（可选） | L3 | 若走 daemon 常驻形态 |

**硬约束**：

- `rustcode-kernel` **零改动**（L0 零产品语义不变量）；
- **不新建第二套 `CodingRuntime` owner**：IM 适配器只**调用**装配链；
- 所有出站请求走 `build_http_client`（除非 §4.4 的豁免成立）；
- 适配器**不得**引入托管账号/中心化服务（§1.3 教训）。

---

## 4. 关键机制（逐项判定可行性）

### 4.1 平台适配：trait 抽象

```rust
#[async_trait]
pub trait ImAdapter: Send + Sync {
    fn platform(&self) -> &'static str;
    /// 入站：拉取或接收一条用户消息。
    async fn next_message(&self) -> Result<Option<ImMessage>>;
    /// 出站：回发一条（已分片的）文本。
    async fn send_text(&self, chat_id: &ChatId, text: &str) -> Result<()>;
    /// 出站：可选的"正在输入"指示（多数平台不支持，默认 no-op）。
    async fn typing(&self, _chat_id: &ChatId) -> Result<()> { Ok(()) }
}
```

**可行性：高。** 这是一个薄抽象，各平台差异被收敛到 `send_text` 与消息解析。
**风险点**：不要试图把"编辑已发消息实现伪流式"做成 trait 的必备能力 ——
多数平台不支持，会逼出一堆 `NotSupported` 分支。用"分片完整性"为第一优先。

### 4.2 入站形态：长轮询 vs webhook（决定是否需要公网）

| 形态 | 是否需要公网/隧道 | 平台举例 | 复杂度 |
|------|-----------------|---------|--------|
| **长轮询 / gateway** | **不需要** | Telegram（`getUpdates`）、Discord（Gateway） | 低 |
| **webhook 回调** | **需要**（公网 IP 或隧道） | Slack、飞书、企业微信、钉钉 | 高 |

**判定：先做长轮询形态。** 理由：

1. **省掉整条隧道依赖**。webhook 需要 `RUSTCODE_ENABLE_TUNNEL=1` +
   `RUSTCODE_TUNNEL_RELAY` 配齐（`endpoints.rs:107` 默认 `HOSTED_RELAY_ENABLED = false`），
   还得处理 TLS、公网可达、回调签名校验；长轮询**零额外基础设施**。
2. **符合"可自托管、无托管账号"约束**（§1.3）：长轮询是纯客户端行为，
   不需要任何一方提供公网入口。
3. webhook 形态**后续可加**，且此时隧道能力已就绪 —— 不阻塞。

### 4.3 身份映射与会话续接

新增持久层（落 `rustcode-config`，与 `schedule.rs` 同性质）：

```rust
pub struct ImBinding {
    pub platform: String,        // "telegram" | "slack" | ...
    pub chat_id: String,         // 平台侧会话/用户标识
    pub project: String,         // 绑定的工作目录
    pub session_id: String,      // 对应的 RustCode 会话
    pub created_at: i64,
    pub updated_at: i64,
}
// 存储：$RUSTCODE_HOME/im/bindings/<platform>-<hash(chat_id)>.json
```

**必须用 hash 而非原始 `chat_id` 作文件名**：`chat_id` 是不可信输入，
直接拼路径会重蹈 `schedule.rs` 已修的目录穿越问题（该处的 `valid_id` 守卫
是现成的正确先例，见 `schedule.rs:54-63`）。

配合 `SessionOrigin::Im` 变体（`manager.rs:357`），使 IM 会话可按来源过滤。

**可行性：高**，但**必须**先定一个语义（见 §8 Q2）：IM 的 `chat_id` 是**按私聊**
（一个用户一个会话）还是**按群**（一个群一个会话）映射到 session？
群映射会让**同群所有人共享 agent 上下文**——若 agent 能读到该项目的文件，
这是**隐私事故**（§2.2 已警告）。

### 4.4 出站走 egress 工厂还是豁免？

**判定：属结构性豁免（与 LLM 适配器同类）。**

依据 AGENTS.md 的两类豁免：LLM 适配器打的是**用户自供的 base_url**（BYO 语义），
不走工厂。IM 适配器**完全同构**：

- base_url 由用户配置（BYO 的 bot API 端点，或自建网关）；
- token 由用户在自己的平台申请；
- 目标是第三方服务，不是本产品的托管端点。

因此 IM 适配器可以自建 `reqwest::Client`，**但必须在代码注释里写明豁免理由**
（对齐 `openai_compat.rs` / `anthropic.rs` 的既有做法）。

**反例（不得做）**：若 IM 适配器要连**本产品的托管中继**，那就不是 BYO，
**必须**走 `build_http_client`。这条边界要写进评审清单。

### 4.5 审批的异步往返（**最硬的机制问题**）

这是 IM 接入最容易低估的一点。现场：

```rust
// daemon/src/lib.rs:1295-1310
fn client_interactive_permission(
    client_mode: ClientMode, enforce_token: bool,
    webui_no_auth: bool, bind_host: &str,
) -> bool {
    enforce_token
        || (webui_no_auth && matches!(client_mode, ClientMode::Webui))
        || (matches!(client_mode,
            ClientMode::Channel | ClientMode::Webui
            | ClientMode::Vscode | ClientMode::Jetbrains)   // <- Channel 在白名单里
            && is_loopback_authority(bind_host))
}
```

**关键事实：`ClientMode::Channel` 已被计入可交互权限。** 这不是巧合 ——
它正是为 IM/渠道桥接预留的钩子（wire tag `"channel"`，`client_mode.rs:46`）。
故 IM 适配器**必须**在请求头带 `x-rustcode-client: channel`。

**两条路径的对比（决定必须用哪条）**：

| 路径 | 审批行为 | 适合 IM？ |
|------|---------|----------|
| `run_native_headless(..., strict_unattended = true)` | **一律拒绝**被升级的调用（`main.rs:3717-3718` 直接 `return false`） | **否** —— IM 背后**有**人，只是异步 |
| `/chat/*` + `permission_bridge` + `ClientMode::Channel` | 请求进入 `pending_permissions`，等外部应答 | **是** |

**IM 审批的完整往返**（这是必须实现的）：

```text
agent 需要审批
    -> ChatEvent::PermissionRequest（live_api.rs:999 APPROVAL_KIND）
    -> 适配器格式化为 IM 消息（"是否允许执行 bash: rm -rf ...？[允许/拒绝]"）
    -> 用户回复
    -> 适配器 POST /live/permission（live_api.rs:2360）或 /chat/permission
    -> permission_bridge::deliver(session_id, request_id, value)（:59）
    -> agent 继续
```

**硬约束**：

- **必须设超时**。用户不回复时按 fail-closed 处理（拒绝），
  不得无限等待（`permission_bridge` 的 `request_timeout` 既有语义即此）。
- **不得**为省事把 IM 走成 `strict_unattended = true` —— 那会让一切危险操作静默失败，
  用户看到"agent 什么都没做"却不知原因。
- **不得**默认 `auto` 模式绕过审批（复用 `run_task()` 既有的
  "auto 降级为 accept_edits" 策略，`schedule_cmd.rs:634-638`）。

**可行性：可实现，但这是本方案工作量最大的一块**（需新增审批消息的格式化、
回复解析、超时与幂等映射）。

### 4.6 分片、去重与幂等

| 问题 | 机制 |
|------|------|
| 回答超平台单条上限 | 在**段落边界**切分（优先 `\n\n`，退而 `\n`），避免切断代码块 |
| webhook 重投（平台会重试） | 按平台消息 id 去重，落 `dedupe` 表（有界 TTL） |
| 出站失败 | 有限次退避重试；超限则落日志并告知用户"回发失败" |
| 平台限流（429） | 复用 `capabilities` 既有退避工具；**不**新增第二套限流（对齐 AGENTS.md） |
| webhook 响应超时（平台要求快速 ack） | 收到即 ack，实际处理异步（若做 webhook 形态） |

---

## 5. 分期实施

### 5.0 前置：先做 `rustcode-agent-api`，不要为 IM 单独造 runtime

`docs/agent-api-rfc.md` 已设计好"外部程序以 task-oriented 方式驱动 agent"
（`POST /agent/tasks` → 轮询/流式 → 取结果、`TaskManager`、`WorktreeScope`、
限流、fail-closed 审批）。**IM 适配器需要的正是这些东西**：

| IM 需要 | agent-api 已设计 |
|---------|-----------------|
| 多用户并发、每用户独立 runtime | `TaskManager` + 每任务独立 `CodingRuntime` |
| 异步取结果（非 SSE） | `GET /agent/tasks/{id}` 轮询 |
| 隔离（防 IM 用户间互相干扰） | `WorktreeScope` |
| 非交互审批策略 | `AgentApprovalMode` |

**故建议**：IM 接入作为 `agent-api` 的**第一个消费者**，而非平行自造。
若先做 IM 再造 agent-api，会产生两套 runtime 准入逻辑（违反单一 owner）。

### 5.1 P-IM0 —— 身份映射地基（无平台代码，纯持久层）

| 文件 | 改动 |
|------|------|
| `crates/rustcode-config/src/im.rs`（新） | `ImBinding` + 读写（复用 `valid_id` 式守卫 + 文件名 hash） |
| `crates/rustcode-capabilities/src/session/manager.rs` | `SessionOrigin` 增加 `Im` 变体（`#[serde(default)]` 兼容既有会话） |
| `crates/rustcode-config/src/i18n/*` | 相关文案（en/zh 双语 arm 齐备） |

**验收**：`ImBinding` 往返；`chat_id` 含 `../` 时被拒绝（目录穿越回归）；
既有会话反序列化仍为 `Manual`。

### 5.2 P-IM1 —— 单平台适配器（长连接形态，最小可用）

**建议首选钉钉**（Stream 模式：WebSocket 长连接、**已证实无需公网/隧道**、
凭证正是 AgentCore 口径的 Client ID / Client Secret，见 §1.6）。
Telegram（`getUpdates` 长轮询）可作为次选 —— 同样零基础设施，
但不在目标平台清单内，价值略低。

| 文件 | 改动 |
|------|------|
| `crates/rustcode-im/src/{lib,adapter,dingtalk}.rs`（新 crate） | `ImAdapter` trait + 单平台实现（钉钉 Stream 走 WebSocket，需选型 WS 客户端依赖） |
| `crates/rustcode-im/src/bridge.rs` | 消息 → headless 调用 → 分片回发 |
| `crates/rustcode-config/src/config/im.rs` | `[im]` 配置表（平台、token `env:VAR`、绑定 project，见 §1.7） |
| `crates/rustcode-cli/src/main.rs` | `rustcode im` 子命令（前台运行适配器） |

**依赖选型提示**：钉钉 Stream 需要 WebSocket **客户端**。本仓既有 `rustcode-tunnel`
已实现 WS（`client.rs` 用 WebSocket 连中继），**优先复用它已有的 WS 依赖**，
不要引入第二套 WS 栈（对齐「最小化依赖」与「单一实现」倾向）。
具体 crate 名与版本须在实施时从 `crates/rustcode-tunnel/Cargo.toml` 读取确认，
**不得臆断**。

**关键实现点**：复用 `run_native_headless(capture = true)` 取完整回答 +
`--resume <session_id>` 续接（§2.3）；请求头带 `x-rustcode-client: channel`。

**验收**：IM 里发一句话 → 收到完整回答；再发一句 → agent **记得**上文（会话续接）；
IM token 不出现在任何日志/错误消息里。

### 5.3 P-IM2 —— 异步审批往返

| 文件 | 改动 |
|------|------|
| `crates/rustcode-im/src/approval.rs`（新） | 审批请求格式化（含工具名/参数摘要）与回复解析 |
| 同上 | 超时 fail-closed；回复幂等（重复回复不重复投递） |

**验收**：agent 请求审批 → IM 收到可读请求；回复"允许" → 工具执行；
回复"拒绝" → 工具被拒且 agent 继续；**不回复** → 超时后按拒绝处理，且**有明确提示**。

### 5.4 P-IM3 —— 多平台 + 加固（后续）

webhook 形态（需隧道）、多平台并发、群聊语义、分片优化、限流与去重加固。

### 5.5 已实现增量 —— 连通性自测（2026-09-23，对标 AgentCore 第三步"验证机器人"）

**探针下沉（单一实现）**：`crates/rustcode-capabilities/src/im_probe.rs`（新），
feature `im = ["egress"]`（opt-in，HTTP-only，不拉 WS 栈）。含
`probe_dingtalk(client_id, client_secret, gateway)`（走 `build_http_client`
单一出站工厂）、`parse_open_response`、`urlencode`、`best_effort_local_ip`。
CLI `DingTalkAdapter::open_connection` 改为**委托**该探针并 re-export
`CHATBOT_TOPIC` / `DEFAULT_GATEWAY`，本地重复实现已删除。

**daemon 测试路由**：`POST /im/channels/test { index }`（`api_im.rs`）。
按**已保存**的配置取渠道（WebUI 测的就是将要服务的），服务端展开凭据；
响应 `{ok, platform, message, endpoint_host?}`。状态码契约：`404` 索引越界、
`400` 平台未知/凭据缺失或展开为空、`200 + ok:false` 测试失败或平台未支持。
`endpoint_host` 只含 authority（查询串被剥离，dial ticket 永不进响应）；
网关覆盖沿用 CLI 同款 `RUSTCODE_DINGTALK_GATEWAY`，禁用渠道也**可测**
（先验证凭据再开总开关是正常顺序）。

**WebUI**：渠道行新增「测试连接」按钮（`im.test/im.testing/im.testHint`
双语键），结果按行渲染（成功 `im-test-ok` / 失败 `im-test-failed`，
失败显示服务端本地化消息）；保存/删除/重排后旧判定作废
（索引不再指向同一渠道）。

**测试落点**：`im_probe` 8 个（缺失 endpoint 判 Auth 非 Transport、
secret 不入错误串等）；`api_im` 9 个（`endpoint_host` 剥查询串、
ticket 不入判定）；cli `im::` 45 个；webui 253/0。

---

## 6. 风险与失败语义

| 风险 | 严重度 | 缓解 |
|------|-------|------|
| **误用 `/live` 单绑定 hub** 导致多用户互踢 | 高 | 从一开始就走"每用户独立 runtime"（§3.1）；评审检查不得复用 `HubState` |
| **隐私泄漏**：群聊共享会话，A 看到 B 的上下文 | 高 | 默认**按私聊**映射（§4.3）；群聊模式需显式开启并显著告警 |
| **IM token 泄漏进日志/bash** | 高 | `CredentialBashGate` 已覆盖 `_webhook`/`_webhook_url`（`credential_bash_gate.rs:114-115`）；**不得放宽**；token 走 `env:VAR` 展开 |
| **审批静默拒绝**（误用 `strict_unattended`） | 高 | 必须走 `ClientMode::Channel` 交互路径（§4.5）；专项测试守住 |
| **无限等待不回复** | 中 | 审批超时 fail-closed + 明确告知用户 |
| **webhook 重投导致重复执行** | 中 | 消息 id 去重表 |
| **长任务超平台超时** | 中 | 收到即 ack + 异步回发；期间发"处理中"提示 |
| **Windows console 闪烁**（历史真实问题） | 低 | **已有修复**（`process_utils.rs` 的 `CREATE_NO_WINDOW`），复用不重踩 |
| **平台限流打爆** | 中 | 复用既有退避，不新增第二套限流 |
| **重建被移除的托管 App/账号体系** | 高 | 严禁托管账号；BYO token + 可自托管（§1.3 教训） |

---

## 7. 验证与交付

门禁沿用仓库标准（G1–G5，见 AGENTS.md 门禁节）：

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -j 1 --workspace --no-fail-fast      # 必须 -j 1
./scripts/test-headless.sh                       # 需先 cargo build
python3 scripts/acp_smoke.py
```

**新增测试清单**：

| 期 | 必增测试 |
|----|---------|
| P-IM0 | `im_binding_roundtrip`、`im_binding_rejects_path_traversal_chat_id`（关键）、`session_origin_im_roundtrips_and_defaults_manual` |
| P-IM1 | `adapter_chunks_long_reply_at_paragraph_boundary`、`bridge_resumes_same_session_for_same_chat`（关键）、`im_token_never_appears_in_errors` |
| P-IM2 | `approval_timeout_fails_closed`（关键）、`approval_reply_is_idempotent`、`im_uses_channel_client_mode` |
| P-IM3 | `webhook_dedupe_drops_replayed_message`、`per_chat_isolation_under_concurrency`（关键） |
| 5.5 连通性自测（已实现） | `missing_endpoint_is_an_auth_error_not_transport`（关键）、`error_display_never_embeds_the_client_secret`、`endpoint_host_never_carries_a_ticket_into_the_verdict`（关键） |

**locale 锁约定**：断言本地化文案的测试必须持 `i18n::test_lock()`；
注意 `summarise_*` 类测试**刻意与 locale 无关**，只持锁、**不得** `set_locale`。

---

## 8. 开放决策（需用户裁决）

| # | 决策 | 选项 | 倾向 |
|---|------|------|------|
| Q1 | 首发平台 | (a) 钉钉（Stream 长连接，已证实无需公网）(b) 飞书（长连接，待核实）(c) 企业微信（webhook，需隧道）(d) Telegram（长轮询，无需公网） | **(a)** —— 唯一有**一手证据**支持"零公网"的平台，且正是目标平台之一 |
| Q2 | `chat_id` 映射语义 | (a) 按私聊（一人一会话）(b) 按群（一群一会话）(c) 可配，默认私聊 | **(c)** —— 但默认必须是私聊，群聊需显式开启（隐私） |
| Q3 | 是否先做 `rustcode-agent-api`（§5.0） | (a) 先做 agent-api 再让 IM 消费 (b) IM 直连现有 headless 装配 | **(a)** —— 避免两套 runtime 准入逻辑 |
| Q4 | 审批默认策略 | (a) 超时 fail-closed 拒绝 (b) 超时降级为 `accept_edits` 继续 | **(a)** —— 与仓库既有的 fail-closed 不变量一致 |
| Q5 | 入站形态优先级 | (a) 长连接优先，webhook 后置 (b) 先做通用 webhook 框架 | **(a)** —— 长连接零基础设施；webhook 需隧道且要处理签名/重放 |

---

## 9. WebUI IM 栏目（查看 IM 记录）

### 9.1 现状

IM 接入已实现（`rustcode im` 子命令 + `rustcode_config::im_store`）。但**记录仅存在于磁盘**
（`$RUSTCODE_HOME/im/bindings/*.json`），**没有任何 HTTP 查询端点**，WebUI 无法读取。
WebUI 侧 `.omo`/MCP/扩展等栏目通过 `fetch('/mcp/status', ...)` 这类只读路由取数据。

### 9.2 数据现实（设计必须基于存在的数据）

`ImBinding` 现有字段（**没有独立的运行台账 / run ledger**）：

```rust
pub struct ImBinding {
    pub platform: String,    // "dingtalk" | "feishu" | "wecom"
    pub chat_id: String,     // 平台会话 id（不可读，已 hash 为文件名）
    pub project: String,     // 绑定的工作目录（= agent 身份）
    pub session_id: String,  // RustCode 会话 id（续会话用）
    pub created_at: i64,     // epoch 秒
    pub updated_at: i64,     // 最近一条消息时间
}
```

**结论**：当前可展示的"记录"只有**绑定记录**（哪个聊天驱动哪个项目、对应哪个 session、活跃时间）。
用户要的"查看相应的 IM 记录"在当前数据模型下 = 查看绑定记录 + 下钻到对应 session 的对话历史
（会话历史本身已存在 `SessionManager`/`SessionSnapshot`，不是 IM 新数据）。
**不在本期范围**：逐条消息级 run record（那需新增 `ImRunRecord`，属 P-IM1.5，见 §9.7）。

### 9.3 栏目与面包屑设计

在 WebUI 侧边栏新增 **IM 栏目**（参照 MCP 栏目的 lazy-load + 计数徽标模式），
三级面包屑下钻：

```text
IM (根)                          <- 列出所有已配置的 platform（dingtalk/feishu/wecom），带计数徽标
 |- dingtalk                     <- 该平台下所有 binding（去重：按 (platform, project)）
 |    |- /path/to/project        <- 该项目下该平台的每个 chat 绑定
 |         |- <session_id>       <- 点击进入该 session 的对话历史（复用既有会话查看器）
```

面包屑层级：**平台 → 项目 → 会话**。

### 9.4 后端只读路由（新增）

仿 `/mcp/status`（`lib.rs:5070` + `lib.rs:6533`）在 `webui.rs` 或 `lib.rs` 增加只读路由
（**仅 GET，无副作用，无写入**——对齐 §6 的"WebUI 调度只读路由"先例）：

| 路由 | 返回 | 复用 |
|------|------|------|
| `GET /im/status` | `{ enabled, channels: [{platform, project, chat_id, session_id, created_at, updated_at}] }` | `im_store::list()` |
| `GET /im/bindings?platform=dingtalk` | 该平台全部绑定 | `im_store::list()` 过滤 |
| `GET /im/overview` | 按平台聚合的计数（驱动徽标） | `im_store::list()` 聚合 |

**鉴权**：复用既有 `enforce_token` 中间件（WebUI 端已带 token 头）。
**一致性**：数据从 `im_store` 实时读取，不缓存——绑定是低频写入，实时读无性能问题。

### 9.5 前端组件

仿 MCP 栏目的 lazy-load 模式（`Sidebar.tsx:1079` 的 `renderMcpMenu`）：

- `fetch('/im/status')` 懒加载，加载时显示 `sidebar.imLoading`；
- 三级下钻用本地 state 维护 `selectedPlatform / selectedProject / selectedSession`，
  不是独立路由（对齐 MCP 用 popover 而非页面跳转的做法，避免改动 router）；
- 叶子节点点击 → 调既有 `onSelect(session)`（或等价 handler）进入该 session 对话视图；
- 面包屑 UI 复用既有 `.breadcrumb` 样式类（若有），否则新增 `im-breadcrumb`。

### 9.6 i18n

新增（en/zh 双语 arm 齐备，编译期 parity）：

- `sidebar.im` / `sidebar.imLoading` / `sidebar.imEmpty`
- `im.platformColumn` / `im.projectColumn` / `im.sessionColumn`
- `im.bindingSince` / `im.lastActive` / `im.noBindings`

### 9.7 范围边界（明确不做）

- **不做** IM 消息级 run ledger（需新增 `ImRunRecord` + `schedule` 同款台账，属 P-IM1.5，单独成期）；
- **不做** WebUI 侧创建/删除绑定（保持"只读查看"；创建仍走 `config.toml`，对齐 §1.7 的 BYO 配置面）；
- **不做** 审批交互（那属 P-IM2）。

### 9.8 门禁

```bash
# 后端
cargo fmt --check -p rustcode-daemon
cargo clippy -p rustcode-daemon --all-targets -- -D warnings
cargo test -p rustcode-daemon --lib    # 新增路由测试：/im/status 返回绑定、空时为空数组

# 前端
cd webui && npm run typecheck && npm test
# 构建后须重嵌 dist（cargo clean -p rustcode-daemon）以便 daemon 二进制携带新 bundle
```

