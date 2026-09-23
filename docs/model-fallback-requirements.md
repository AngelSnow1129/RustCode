# 模型回退能力需求（Model Fallback）

状态：**已实现**（L0 配置面 + L1 共享判定 + L2 运行时回退；测试见 §7.1）
日期：2026-09-22
范围：`rustcode-config` / `rustcode-coding` / `rustcode-capabilities` / 四个 driver
实施记录：见 §10（含实测契约与实现期发现的两个真实缺陷）

---

## 1. 目标

当**当前选中的模型**在运行中不可用（限流、上游 5xx、超时、鉴权失效、端点不可达等）时，
运行时应能按用户预先设定的**回退链**自动切换到下一个可用模型，并**继续完成同一个问题**，
而不是让整个回合以 `ProviderError` 终止、把可用性风险留给用户手工重配。

一句话验收：**一个模型挂了，问题还能问完。**

### 1.1 为什么现在需要

- 现有重试只覆盖「同一模型、同一端点」的**瞬时**故障（kernel `RetryReason`：`RateLimited` /
  `UpstreamUnavailable` / `Timeout` / `Network`），重试耗尽后回合即以 `StopReason::ProviderError`
  终止。用户必须手工 `/model` 换模型再重发问题，上下文与进度被迫中断。
- 端点级「鉴权失效 / 模型下线」是**非瞬时**的，重试同一个模型永远不会成功。
- 已有一处**局部**实现可作范本：`task`/`team` 子代理在「内容为零的可重试失败」时会回退到宿主
  provider（见 §2.5），但主 agent 回合没有对等能力。

### 1.2 非目标

- 不做跨 provider 的**负载均衡**或主动探测式选路（不是 router）。
- 不做模型能力的自动协商（不会因为「这个模型不支持 vision」就换另一个）。
- 不改变 `retry_max_attempts` 语义：回退不是重试；重试预算耗尽**之后**才触发回退。
- 不引入第二套 live agent 生命周期：回退必须复用
  `CodingRuntimeControl::ReassembleProvider`，不得另建 provider owner。

---

## 2. 现状取证（实现前必须成立的事实）

| # | 事实 | 位置 |
|---|------|------|
| 2.1 | 模型选择是「逻辑模型 id → account → provider + wire model」的三层解析，解析单点是 `Config::resolve_model` | `crates/rustcode-config/src/config/mod.rs:1015` |
| 2.2 | 解析产物 `ResolvedModelConfig` 是 provider 构造消费的唯一值，含 `selection_id` / `account_id` / `provider_id` / `base_url` / `api_key` / `model` 等 | `crates/rustcode-config/src/config/provider.rs:259-300` |
| 2.3 | 运行时热切换已有唯一既有命令：`CodingRuntimeControl::ReassembleProvider { generation, next, done }`，对外 `CodingRuntimeHandle::reassemble_provider` | `crates/rustcode-coding/src/runtime.rs:1513`、`:2273` |
| 2.4 | 已有「provider 不可用」的显式降级命令 `DeactivateProvider(ProviderUnavailableReason)`，原因枚举为 `NotConfigured` / `AuthenticationRequired` / `UnsupportedBuild` | `crates/rustcode-coding/src/runtime.rs:648`、`:1529` |
| 2.5 | **局部回退范本**：子代理在 `retryable_content_free_failure(&outcome)` 为真且未取消时，用 fallback provider 重跑；候选回退目标按 **provider 身份**（`Arc::ptr_eq`）判重，不按显示名 | `crates/rustcode-capabilities/src/tools/task.rs:1095-1136`、`:1641-1656` |
| 2.6 | 失败分类器已就位：`StopReason` + `outcome.provider_retryable`（结构化分类优先）+ `outcome.http_status`（`408/425/429/5xx` 回退判据） | `crates/rustcode-capabilities/src/tools/task.rs:1645-1655` |
| 2.7 | kernel 只发中性结构（`RetryReason` / `AgentNotice`，零文案）；本地化在 L2 `coding` 的 `localize_kernel_event` / `retry_reason_label` 单点完成 | `AGENTS.md`「编码约束 / I18N」节 |
| 2.8 | 每回合终止原因在运行时收口为 `StopReason::ProviderError`，并有 `provider_unavailable_reason` 单值槽（`AtomicU8` 编解码） | `crates/rustcode-coding/src/runtime.rs:1107`、`:2407`、`:2743` |
| 2.9 | 回退候选可来自配置：`Config.providers` / `Config.models` / `Config.default_model` | `crates/rustcode-config/src/config/mod.rs:284,292,297` |
| 2.10 | 当前**不存在**任何主回合级回退配置键或运行时状态（全仓 `fallback` 命中均为 TLS/UTF-8/命名的无关用法） | 全仓 grep 取证 |

---

## 3. 术语

- **回退链（fallback chain）**：一个有序的模型选择 id 列表 `[primary, fb1, fb2, ...]`，首个可用者胜出。
- **回退目标（fallback target）**：链上当前待尝试的下一个**逻辑模型 id**，经 `resolve_model` 解析。
- **可回退失败（fallback-eligible failure）**：分类为「换一个模型大概率能成功」的失败，见 §5。
- **回退事件（fallback event）**：一次「当前模型失败 → 切换到链上下一个」的转换，必须对用户可见。
- **回合连续性（turn continuity）**：回退后，用户问题、已有会话上下文与已完成工具结果**不丢失**。

---

## 4. 功能需求

### FR-1 可配置的回退链

- FR-1.1 每个模型**选择 id** 可选配 `fallback = ["<model-id>", ...]`，有序、去重、禁止自引用。
- FR-1.2 未配置时**默认不启用回退**（保持现有行为，零行为变更）。
- FR-1.3 链上不存在的 id 在解析期**显式报错**，不得静默跳过（fail-closed，对齐
  `task.rs`「拼错的显式模型必须原子失败」的既有约定，`:876-878`）。
- FR-1.4 链的最大长度设上限（建议 4）并在超限时显式报错，避免配置环导致无限切换。
- FR-1.5 支持全局默认链 `[fallback] default = [...]`，被具体模型的 `fallback` 覆盖；两者都未配则不启用。

### FR-2 触发条件

- FR-2.1 **仅**在失败被判定为「可回退」时触发，判据是**结构化分类优先**，不用字符串匹配错误文案：
  1. `outcome.provider_retryable == Some(true)` → 可回退；
  2. `Some(false)` → **不可回退**（终端失败，不得因为状态码看着像瞬时就去重放）；
  3. `None` 时回落 HTTP：`408 | 425 | 429 | 500..=599` → 可回退；
  4. `StopReason::Timeout` / `StopReason::RateLimited` → 可回退；
  5. `AuthenticationRequired` / `NotConfigured` → 可回退（换模型确有救）；
  6. 用户在回合中**已取消** → 不触发（对齐 `task.rs:1096` 的 `!child_cancel.is_cancelled()`）。
- FR-2.2 触发时机是**重试预算耗尽之后**，不是第一次失败。单模型内的瞬时故障仍由既有
  `RetryReason` 路径处理，避免放大请求量。
- FR-2.3 触发点必须是**回合级**，且发生在回合**产生任何用户可见输出之前**的纯失败路径；
  一旦该回合已有文本/工具结果产出，回退默认不触发（否则会重放并产生重复副作用）。
  未产出内容的判据复用 `task.rs:1641` 的 `text.is_empty() && tool_results.is_empty()` 语义。

### FR-3 回退执行语义（回合连续性）

- FR-3.1 回退经 `reassemble_provider` 完成，`generation` 递增，旧 generation 迟到事件必须被丢弃
  （对齐 `AGENTS.md`「Runtime 生命周期不变量」）。
- FR-3.2 回退保持：会话 id、工作目录、已持久化快照、审批 grant 中与 provider 无关的部分。
- FR-3.3 回退**不**清空会话上下文；用户问题不要求重发。
- FR-3.4 链上移动是**单调前进**的：一次回退尝试过 `fb1` 失败则试 `fb2`，不回跳到 `fb1` 反复打转；
  链耗尽后以 `ProviderError` 终止，并给出可操作报错（列出试过的模型与各自失败原因）。
- FR-3.5 回退后的 provider 必须重新走 `resolve_model`，不得复用失败实例的
  `base_url` / `api_key` / `model` 残留。
- FR-3.6 回退失败（如新 provider 也解析不了）必须**显式**以错误终止，不得静默 fresh、空 snapshot
  或假成功。

### FR-4 用户可见性

- FR-4.1 每次回退必须发一条可见通知：**哪个模型失败、为什么、正在改用哪个模型**。
- FR-4.2 通知走 i18n 目录（`rustcode-config/src/i18n/{messages,en,zh_cn}.rs`），en/zh arm 齐备，
  编译期 parity；`Msg` 变体携带 `{from}` / `{to}` / `{reason}` 字段。
- FR-4.3 失败原因用既有 `retry_reason_label(RetryReason)` 本地化标签，不新增第二套映射。
- FR-4.4 链耗尽时的终态错误必须列出链上试过的模型与其分类原因。
- FR-4.5 稳定机器 token（如 `model_fallback_started` / `model_fallback_exhausted`）随事件发出，
  供 `--json` / daemon / webui 机器通道消费；人类面文案本地化，机器面不本地化。

### FR-5 明确的**不**回退面

- FR-5.1 用户在回合中主动取消。
- FR-5.2 内容策略 / 安全拦截类终端错误。
- FR-5.3 请求本身非法（4xx 中的 400/404/422 等参数类错误）——换模型不会让非法请求变合法。
- FR-5.4 工具执行失败（与 provider 无关）。
- FR-5.5 `Auto` 模式（绝对信任）下凭据闸门相关终止——与模型可用性无关。

### FR-6 与子代理回退统一

- FR-6.1 主 agent 与 `task` / `team` 子代理应共用**同一套**「可回退失败」判定与候选去重语义，
  避免两份真相（判定函数抽到共享位置，或由 `coding` 侧统一导出）。
- FR-6.2 子代理现有行为（回退到宿主 provider）保持向后兼容；新增的显式链优先于该隐式单一回退。
- FR-6.3 子代理回退同样必须发可见事件，且不得跨 worker 写作用域（不改变 `ScopeGuard` 语义）。

### FR-7 driver 一致性

- FR-7.1 回退是 **runtime 层**能力，四个 driver（CLI headless / TUI / daemon / clix）无需各自实现，
  但都要能把回退事件渲染出来。
- FR-7.2 CLI headless 的 `--json` 通道发机器 token + 结构化字段，**不得**发本地化散文。
- FR-7.3 daemon/WebUI：回退事件经 `live_api` 投影为 `ChatEvent`，WebUI 侧文案由前端 i18n 映射
  （沿用既有「daemon 进程 locale 仅作兜底」的先例）。
- FR-7.4 AP 通道（ACP）不受影响，除非其 `available_commands` / 事件面已有对应投影。

---

## 5. 可回退失败的判定规格（单一事实源）

```text
fn fallback_eligible(stop, provider_retryable, http_status, cancelled, produced_output) -> bool
    if cancelled            -> false          // FR-2.1.6 / FR-5.1
    if produced_output      -> false          // FR-2.3
    match stop
        Timeout | RateLimited                     -> true
        ProviderError =>
            match provider_retryable
                Some(v) -> v                   // 结构化分类权威（FR-2.1.1/2）
                None    -> matches!(http_status, Some(408|425|429) | Some(500..=599))
        _ -> false
```

要点：**任何**新增判据都必须并入该函数，禁止在调用点各写一份 `if`（对齐 `webui_no_auth_enabled`
「唯一解析入口」的既有约束风格）。

---

## 6. 配置面设计

### 6.1 配置形态（复用既有 schema，不新增顶层表）

回退链挂在**模型档**上，与其已有的 `account` 同级：

```toml
[models."deepseek-v3"]
account = "deepseek"
model   = "deepseek-chat"
fallback = ["glm-4-plus", "qwen-max"]   # 有序，首个可用者胜出

[models."glm-4-plus"]
account = "glm"
model   = "glm-4-plus"

# 可选：全局默认链，被具体模型的 fallback 覆盖
[fallback]
default = ["glm-4-plus"]
```

理由：`models` 表已是「逻辑模型 id」的权威注册点（`Config.models`，`mod.rs:292`），
`resolve_model` 已是唯一解析单点（`mod.rs:1015`）。把链挂在同一层，解析期即可完成
「链上 id 是否都存在」的 fail-closed 校验（FR-1.3），无需引入新的解析路径。

### 6.2 兼容性约束

- 6.2.1 `fallback` 必须是**可选**字段（`Option<Vec<String>>`），缺失时省略序列化，
  旧配置文件零改动可用（对齐 `ModelProfileConfig` 既有 `skip_serializing_if` 风格）。
- 6.2.2 不删除、不改名任何既有键；不迁移用户磁盘（对齐 codingplan 退役时的
  「schema 与键名完全不变」既有约定）。
- 6.2.3 支持环境变量覆盖（可选）：`RUSTCODE_MODEL_FALLBACK`（逗号分隔），
  优先级 **env > 配置**，与 `webui_no_auth_enabled` 的形状一致——但必须**只有一个解析入口**，
  禁止各 driver 自行 `std::env::var` 再判一次。

### 6.3 配置校验（写盘前 fail-closed）

| 校验 | 失败语义 |
|------|---------|
| 链上 id 在本 `Config` 中可解析 | 显式报错，不落盘、不静默跳过 |
| 链中无自引用（含间接环 `A→B→A`） | 显式报错 |
| 链长度 ≤ 上限（4） | 显式报错 |
| 链内无重复 id | 去重并在 `/config` 面给出可见提示 |

---

## 7. 验收标准

### 7.1 功能验收（每条都要有可执行证据）

1. **A-1 基本回退**：配 `primary→fb1`，桩 provider 令 primary 首请求返回 429 且重试耗尽，
   断言回合**未**以 `ProviderError` 终止、`fb1` 收到了同一个用户问题、且有回退事件发出。
2. **A-2 链式回退**：配 `primary→fb1→fb2`，令 primary 与 fb1 均失败，断言最终由 fb2 完成，
   且事件序列为 `primary→fb1`、`fb1→fb2` 两条。
3. **A-3 链耗尽**：三级全失败，断言以 `ProviderError` 显式终止，报错**列出**全部试过的模型与分类原因。
4. **A-4 不可回退面**：`provider_retryable == Some(false)` 的终端失败**不**触发回退
   （反向锁定，防止「状态码看着像瞬时」就重放）。
5. **A-5 产出后不回退**：回合已产生文本/工具结果后失败，断言**不**触发回退、不重放副作用。
6. **A-6 取消优先**：回合中取消后即使失败也不回退。
7. **A-7 回合连续性**：回退后断言会话 id、工作目录、既有会话上下文不变；用户问题无需重发。
8. **A-8 生成隔离**：回退后旧 generation 的迟到事件被丢弃，不污染新 runtime。
9. **A-9 解析 fail-closed**：链含不存在 id / 自引用 / 超长链时，配置解析显式报错。
   **实现口径**：诊断经既有 CLI 启动警告通道（`Msg::CliConfigLoadWarnings`）呈现给用户，
   见 §10.5 的 A-9 行。
10. **A-10 默认关闭**：未配 `fallback` 时行为与改动前**逐字一致**（回归基线）。
11. **A-11 driver 面**：headless `--json` 发机器 token + 结构化字段且**不含**中文散文；
    TUI/daemon/WebUI 各自能渲染回退通知。
12. **A-12 子代理兼容**：`task`/`team` 既有「回退到宿主 provider」行为不回归；
    显式链存在时优先于隐式回退。

### 7.2 门禁要求

- G1 `cargo fmt --check` 干净。
- G2 `cargo clippy --workspace --all-targets -- -D warnings` 无新增告警。
- G3 `cargo test -j 1 --workspace --no-fail-fast`（内存上限 8GB，必须 `-j 1`）。
- i18n 改动后跑**整个 crate 的 `--lib`**（内容断言散落在 `en.rs`/`zh_cn.rs` 末尾）。
- 断言本地化输出的测试必须持 `i18n::test_lock()`；断言英文的用例 `set_locale(Locale::En)`，
  但**与 locale 无关**的用例只持锁、**绝不** `set_locale`。

---

## 8. 边界与风险

| # | 风险 | 处置 |
|---|------|------|
| R-1 | 回退放大请求量（用户不期望的双倍消耗） | FR-2.2：仅在重试预算耗尽后触发；FR-1.4 链长上限；可见通知让用户可预期 |
| R-2 | 回退后行为漂移（模型能力/上下文窗口不同） | 只换 provider/model，不换 persona 与工具集；`context_window` 按新模型重新解析 |
| R-3 | 产生重复副作用（工具已执行） | FR-2.3：产出后不回退；A-5 反向锁定 |
| R-4 | 旧 generation 事件污染 | FR-3.1 + A-8，复用既有 generation 守卫，不新增机制 |
| R-5 | 与 `DeactivateProvider` 语义打架 | 回退是 `DeactivateProvider` 的**上游**：先尝试链，链耗尽才降级为不可用终态 |
| R-6 | 密钥安全 | 回退目标经 `resolve_model` 取自己的 `api_key`；错误文案沿用既有 secret-safe 约束（`resolve_model_errors_are_secret_safe`） |
| R-7 | 与 kernel 零文案约束冲突 | 判定与事件在 `coding` 层收口，kernel 不含回退文案（对齐 §2.7） |

### 待用户裁决的开放项

> 2026-09-23 已由用户逐项裁决：三项均维持现状，裁决记录见 §10.6。

- **O-1 默认行为**：未配置时是否应默认启用「回退到 `default_model`」？（当前需求定为**不启用**，
  零行为变更。若希望「开箱即用」，需明确默认链的确定规则。）
- **O-2 触发时机**：是否允许在**已产出部分文本**的回合回退（会重放并让用户看到重复前缀）？
  （当前需求定为**不允许**。）
- **O-3 链的粒度**：链是「模型 id 级」还是同时支持「provider 级」（某 provider 全部模型挂掉）？
  （当前需求定为**模型 id 级**，provider 级可由用户显式列多个 id 表达。）

---

## 9. 实现落点（非约束，供实施参考）

| 层 | 落点 | 内容 |
|----|------|------|
| L0 `config` | `config/provider.rs` 的 `ModelProfileConfig` | 新增可选 `fallback` 字段 + 解析期校验 |
| L0 `config` | `config/mod.rs` | 链解析与校验；`CodingAgentConfig` 投影（`coding/config.rs:17`） |
| L0 `config` | `i18n/{messages,en,zh_cn}.rs` | 回退通知与终态文案变体（en/zh parity） |
| L2 `coding` | `runtime.rs` 回合终止路径（`StopReason::ProviderError` 收口处，`:3210`/`:3383` 附近） | 判定 + 沿链 `reassemble_provider` + 发事件 |
| L2 `coding` | `runtime.rs` 生成隔离 | 复用既有 generation 守卫，不新增机制 |
| L1 `capabilities` | `tools/task.rs:1641` 的判定函数 | 抽为共享实现，主回合与子代理共用（FR-6.1） |

**不变量（实施时必须遵守）**：

1. kernel 保持零文案、零内部依赖。
2. 出站 HTTP 仍只走 `capabilities::egress::build_http_client`（回退不新增出站路径）。
3. 不引入第二套 live agent 生命周期：`CodingRuntime` 仍是唯一 owner。
4. 不新增 `LlmClient` trait 或 core facade。
5. 纯 ASCII 标签，无 Unicode Emoji。
6. 用户可见文案一律走 i18n 目录，禁止临时双语 helper。

---

## 10. 实施记录

### 10.1 实际落点（与 §9 的差异已就地修正）

| 层 | 文件 | 内容 |
|----|------|------|
| L0 | `config/provider.rs` | `ModelProfileConfig.fallback: Vec<String>`（`Option` 之外的**空 vec 即关闭**，`skip_serializing_if = "Vec::is_empty"`）；`MAX_MODEL_FALLBACK_CHAIN = 4` |
| L0 | `config/mod.rs` | `validate_model_fallback_chains()`（存在性 / 自引用 / 环 / 超长 / 重复 → 诊断）、`model_fallback_chain(id)`（运行时读路径，裁剪去重、丢弃坏项、**永不失败**） |
| L0 | `i18n/{messages,en,zh_cn}.rs` | `CfgDiagFallback{UnknownTarget,SelfReference,Cycle,ChainTooLong,Duplicates}` 5 条 + `ModelFallback{Started,Exhausted}` 2 条，en/zh parity 由编译期强制 |
| L1 | `capabilities/fallback.rs`（**新增**） | `fallback_eligible(outcome, cancelled)` 与 `fallback_eligible_for_stop(...)`——**唯一点**判定，feature-gate-free（只依赖 kernel） |
| L1 | `capabilities/tools/task.rs` | `retryable_content_free_failure` 改为**委托**共享判定（FR-6.1），子代理行为不变 |
| L2 | `coding/fallback.rs`（**新增**） | `FallbackWalk`（单调前进 + 显式耗尽 + `failures_diagnostic()`）、`tokens::{STARTED,EXHAUSTED}`、`attempt_reason()` |
| L2 | `coding/config.rs` | `CodingRuntimeConfig.provider_fallback` / `CodingAgentConfig.provider_fallback` + `repoint_to_selection()`（按目标重新解析，杜绝失败模型的 `base_url`/`api_key`/`model` 残留） |
| L2 | `coding/runtime.rs` | owner 循环接线：`active_turn_prompt` 记录本轮问题、`turn_produced_output` / `turn_fail_status` / `turn_fail_retryable` 采集结构化失败事实、终态缝前置 `try_model_fallback()`（命中则**跳过**终止路径）、`CodingRuntimeControlReceiver::self_sender()` 自发送通道 |

### 10.2 实测契约（写代码前未预见，以实测为准）

1. **换 provider 必须停旧 agent，因此原回合以 `Cancelled` 收尾**。`ReassembleProvider` 臂会
   `stop_current_agent`，旧 agent 的终态是停机终态而非本轮业务终态；随后暂存的问题经
   `replay_pending_resume_prompt` 在新模型上**重放**。这是既有 `/model` 热切换的同一语义，
   **不是**回退特有的终态。因此 A-1 的可观测判据是「回合**没有**以 `ProviderError` 终结」，
   而不是「同一个 `turn_id` 继续到 `Stopped`」。
2. **回退只在「新回合」语义下成立**：链上移动是单调的，每次失败换一个模型；
   `turn_produced_output` / 失败事实在**新回合开始时重置**，但 `FallbackWalk` 跨回合保留，
   以保证 FR-3.4 的「不回跳」。
3. **目标必须先验证可构建**（`provider_factory.build`）再交出回合：否则新旧 provider 都不可用，
   回合会既无 live agent 也无终态——即挂死。此检查是 §R-3.6「显式失败而非静默 fresh」的具体落实。
4. **没有 session binding 就不可能回退**（暂存与重放都挂在 binding 上）。`SessionMode::Disabled`
   下 `try_model_fallback` 直接返回 `false`，回退静默降级为既有行为。

### 10.3 实现期发现的真实缺陷（已修）

| # | 缺陷 | 处置 |
|---|------|------|
| D-1 | 若回退目标无法构建，回合会**挂死**（无 live agent、无终态），违反「每个 accepted operation 都有终态」不变量 | 交出回合前先 `build()` 验证；失败则返回 `false` 让调用方走正常错误路径。测试 `exhausted_chain_ends_the_turn_explicitly` 锁定 |
| D-2 | clippy：`#[allow(too_many_arguments)]` 被文档注释挤成重复属性，从既有的 `replay_pending_resume_prompt` 漂移到了新函数上 | 属性归位，两处告警清零（clippy 复检仅剩既有的 `askpass/server.rs` 重复属性，非本轮引入） |
| D-3 | 新增的 7 个回退测试**缺 `RUSTCODE_HOME` 隔离**：单跑全绿、全量跑 2 个红。`RUSTCODE_HOME` 是**进程级** env，姊妹测试的 `set_var` + `TempDir` 析构会落到本测试名下，回合中途会话文件消失 | 夹具自建 `TempDir` 并把守卫返回给调用方保活，7 个测试全部加 `#[serial_test::serial(rustcode_home)]`。全量复跑 **449/0**（详见 10.4） |
| D-4 | **取消被吞掉，回退继续了一个用户已撤回的回合**（FR-2.4 被实际违反）：终态缝先 `terminal_reason.take().or_else(|| cancel_pending.then_some(Cancelled))`，若 provider 失败**已先写入** `terminal_reason`，该 `or_else` 就不会再读 `cancel_pending`；而回退判定把「是否取消」等价于 `reason == StopReason::Cancelled`，于是取消标志被静默丢弃，回退照常发生 | 在清 `cancel_pending` **之前**先捕获 `let cancelled = cancel_pending \|\| reason == StopReason::Cancelled;` 并作为显式参数传入 `try_model_fallback`。测试 `cancelling_after_a_failure_suppresses_the_failover` 锁定（该测试**先失败**才发现此缺陷，非事后补测） |
| D-5 | 子代理侧 FR-6.2「显式链优先」**根本没有接线点**：`provider_fallback` 在 `capabilities` 中 0 命中，`task` 只有隐式单宿主回退 | 本轮实现：`TaskTool` 新增 `with_chain_provider`（按 selection id 解析成有序 provider 列表）与 `with_tier_selection_ids`（让未指定 `model` 的 tier 子任务也能找到自己的链），`parts.rs` 在 coding 侧注入解析闭包；单跳重试改为**沿候选列表多跳**。测试 `explicit_chain_is_tried_before_the_implicit_host_fallback` 断言**尝试顺序** |
| D-6 | **未指定 `model` 的子任务在「宿主不参与 tier 路由」时完全没有回退**（最常见场景：用户只配了一个模型 + 一条 `fallback` 链）。实现 D-5 时我照搬了 `resolve_tier_keys` 的 `None` 语义去跳过 tier key，但该函数的 `None` 意为「两层都塌陷到宿主」——对**选 provider** 是对的，对**查链**是错的：没有 key 就查不到链，而隐式宿主兜底又只覆盖 `Explore && Hard`，且会被 provider 身份去重吃掉 ⇒ 候选列表为空 | 新增纯函数 `tier_chain_keys`（`crates/rustcode-coding/src/subagent_tiers.rs`）：路由开启时沿用 tier id，关闭时回退到**宿主 selection id**（`cfg.provider_name` 本就是 selection id）。`parts.rs` 改用它，不再 `if let Some`。测试 `chain_keys_fall_back_to_the_host_selection_when_routing_is_off`、`chain_keys_mirror_the_tier_ids_when_routing_is_on`、`model_less_non_hard_subtask_still_fails_over_along_its_tier_chain`（刻意**不装宿主 provider**，以证明是链而非兜底在起作用） |

> **D-6 的教训**：一个函数返回 `None` 时，**必须问清它否定的到底是什么**。`resolve_tier_keys`
> 的 `None` 是「不要路由」，不是「没有模型选择」；我把它当成后者直接跳过了接线，于是把
> 「无路由」错误地实现成了「无回退」。语义借用是危险的——尤其当借用方的用途（查链）和原用途
> （选 provider）只字面相似时。

> **D-4 的教训**：`a.take().or_else(|| b.then_some(x))` 把两个互斥来源折叠成一个 `Option` 时，
> 「先到者胜」会让**后到但更重要**的信号（用户取消）静默消失。这类折叠要看**优先级**是否需要
> 独立表达，而不是只问 `Option` 最终是否有值。
>
> **D-3 的教训值得复用**：单跑通过、全量失败，且失败点与断言逻辑无关时，先怀疑**进程级共享状态**
> （env、固定端口、工作目录），而不是去改断言。这里改断言只会掩盖真因。

### 10.4 验证基线

- `cargo fmt --check` → exit 0
- `cargo check --workspace --all-targets` → Finished，0 error
- `cargo check -p rustcode-capabilities --no-default-features --features tools` → Finished，
  **0 error**。这条是刻意的**分层门禁**：FR-6.2 的链解析留在 coding 侧、capabilities 只接收
  已构建的 provider，故新增 `with_chain_provider` 后 `tools`-only 构建仍不拖 `rustcode-config`
- `cargo clippy -p rustcode-coding -p rustcode-capabilities -p rustcode-config -p rustcode --all-targets`
  → 本轮改动**零**新增告警（逐条核对 `-->` 指向，无一处落在 `fallback.rs` / `headless_json.rs` /
  `task.rs` / `parts.rs` / `runtime.rs` / `translate.rs`）
- `cargo test -p rustcode-config --lib` → **332 passed / 0 failed**（含 4 条回退文案断言 +
  3 条 A-9 诊断通道断言）
- `cargo test -p rustcode-coding --lib` → **457 passed / 0 failed / 8 ignored**（含 8 个回退端到端
  测试、`tier_chain_keys` 的 2 条单测，以及本轮新增的 5 条 team 成员回退断言）
- `cargo test -p rustcode-capabilities --lib` → **850 passed / 0 failed**；其中 `tools::task`
  → **42 passed / 0 failed**（原 40 条 + FR-6.2 顺序断言 + 模型缺失子任务的 tier 链断言）、
  `tools::parallel_edit` → **14 passed / 0 failed**（原 11 条 + 本轮 3 条链断言）
- `cargo test -p rustcode --lib` → **117 passed / 0 failed**；`--bins` → **102 passed / 0 failed**
  （含 A-11 的 jsonl 契约断言与 ACP 投影边界断言）
- `cargo test -p rustcode-daemon --lib` → **291 passed / 0 failed**（含回退通知线协议投影断言）
- `cargo test -j 1 --workspace --no-fail-fast` → **5540 passed / 0 failed / 12 ignored**（94 套件全绿）

### 10.5 端到端覆盖现状

已由本轮的端到端测试覆盖（不再是边界）：

| 验收项 | 测试 | 断言要点 |
|--------|------|---------|
| A-1 基本回退 | `transient_provider_failure_fails_over_and_replays_the_question` | 回退可见（Warning）、走 reassemble 路径、**恰好 1 次**换模型、回合**未**以 `ProviderError` 终结且到达 `Stopped` |
| A-2 链式回退 | `chained_fallback_walks_to_the_next_target_in_order` | 两次换模型的**顺序**为 `primary→fb1→fb2`（顺序断言，非仅计数）、`swap_stops == 2`、第三个模型完成回合 |
| A-3 链耗尽/不可构建目标 | `exhausted_chain_ends_the_turn_explicitly` | 目标不可构建时**显式终结**而非挂死 |
| A-4 不可回退面（结构化） | `terminal_provider_classification_suppresses_fallback` | `retryable == Some(false)` 压过 HTTP 503，不回退 |
| A-5 产出后不回退 | `produced_output_suppresses_fallback` | 已产出文本 → 不回退、以真实 `ProviderError` 终结 |
| A-7 回合连续性 | `failover_preserves_the_session_and_advances_the_generation` | 回退全程 **零 `SessionChanged`**（`SessionChanged` 只在新建/切换会话时发出，两处 emit 均不在 reassemble 臂内）→ 换的是 provider，不是会话 |
| A-8 生成隔离 | 同上 | `handle.status().generation` 严格递增 → 被替换 agent 的迟到事件落入旧 generation，由既有守卫过滤 |
| A-10 默认关闭 | `absent_chain_keeps_the_single_model_behaviour` | 无链 = 零行为变更 |
| A-11 机器通道 | `model_fallback_token_is_carried_verbatim_in_the_code_field`、`model_fallback_tokens_are_stable_ascii_identifiers` | token **逐字**进 `code` 字段、锁定字面值、限定 `model_fallback_` 命名空间、纯 ASCII |
| A-11 人类面文案 | `{en,zh}_fallback_notice_names_both_models_and_the_reason`、`{en,zh}_exhausted_notice_lists_every_attempt` | 中英各自点名**两个**模型 + 失败原因；回退措辞不得读作终结（反向锁定） |
| A-11 人类面投递（daemon） | `native_live_projector_carries_a_failover_notice_verbatim` | 回退通知以 `warning` 类型上线且**逐字**透传（含中文），不被吞掉、不被改成终态错误 |
| **A-6 取消优先** | `cancelling_after_a_failure_suppresses_the_failover` | 失败已记录、用户在收尾快照在途时取消 → **零**回退 Warning、`swap_stops == 0`。该测试**先失败**，据此发现并修掉 D-4 |
| **A-12 子代理显式链优先** | `explicit_chain_is_tried_before_the_implicit_host_fallback` | 断言**尝试顺序**为 `capable → chain-hop-1 → chain-hop-2`：显式链优先，且链还能应答时**不得**触碰隐式宿主兜底 |
| **A-12 补：tier 链键** | `chain_keys_fall_back_to_the_host_selection_when_routing_is_off`、`chain_keys_mirror_the_tier_ids_when_routing_is_on`、`model_less_non_hard_subtask_still_fails_over_along_its_tier_chain` | 无路由时链键塌陷到**宿主 selection id**（而非无键）；有路由时用 tier id；模型缺失且非 `Explore && Hard` 的子任务仍能靠链恢复（测试**不装宿主 provider**，故证明是链在起作用） |

| **A-9 诊断通道** | `fallback_chain_diagnostics_reach_the_startup_warning_channel`、`a_valid_fallback_chain_adds_no_startup_warning`、`chain_warnings_are_appended_to_provider_load_warnings` | 坏链的 `CfgDiagFallback*` 诊断确实经 `load_with_diagnostics` 的 `Vec<String>` 抵达启动警告通道（CLI 后续以 `Msg::CliConfigLoadWarnings` 打印）；合法链零新增警告；既有 `[providers.*]` 隔离警告不被顶替（**追加**语义） |

仍未覆盖（诚实边界）：

- **A-9 诊断通道已接通（本轮补）**：`validate_model_fallback_chains` 此前**没有生产调用点**
  （全仓 grep 只命中定义、文档注释与 `#[cfg(test)]`），回退链的问题因此无法呈现给用户——
  「显式报错」只在校验函数内部成立。现已在 `Config::parse_disk_content_tolerant` 的收尾处
  追加 `warnings.extend(config.validate_model_fallback_chains())`，复用既有
  `Msg::CliConfigLoadWarnings` 通道（`crates/rustcode-cli/src/main.rs` 的配置加载分支打印），
  与 `[providers.*]` 段隔离信息、`default_provider` 回退提示同源。
  - 三个测试锁定：`fallback_chain_diagnostics_reach_the_startup_warning_channel`（悬空目标
    确实出现在启动警告里）、`a_valid_fallback_chain_adds_no_startup_warning`（合法链零新增
    警告，通道不误报）、`chain_warnings_are_appended_to_provider_load_warnings`（**追加**语义
    ——既有 provider 警告不被顶替、无提前返回）。
  - **仍未覆盖（诚实边界）**：这不等于「不落盘」。`Config::save` → `ConfigStore::replace` →
    `persist_locked` 全链路仍然**不调**校验，故用户可以把一条坏链写进磁盘；本轮的落点是
    「写盘后重新加载时**显式报出来**」，而非「写入时拒绝」。`validate_provider_accounts_and_models`
    同样是零生产调用点——这是与既有 provider 校验同形的**存量架构缺口**，非本轮引入，
    也非本需求范围。
  - `load_with_diagnostics` 的通道本身**不是**缺陷（它确实可用且已被 CLI 消费）；本轮补的是
    「链校验结果没进这条通道」。
- **子代理链的覆盖现状（1、2、3 已全部覆盖）**：
  1. **[已覆盖] `team` 成员** —— 此前 `rustcode-coding/src/team/runner.rs` 自建 agent 后直接
     `run_to_completion`，既无显式链也无隐式宿主回退，一个模型抖动就会让整次委派失败。
     现 `TeamRunnerFactory` 新增 `with_chain_providers(...)`：`run()` 按
     `[本 tier provider, 该 tier 的显式链...]` 去重成候选列表后逐跳尝试，跳与跳之间
     **重建 agent**（不复用失败 attempt 的 builder，杜绝旧 provider 残留，对应 FR-3.5）；
     每跳发一条 `Msg::ModelFallbackStarted`，链走完仍失败则发 `ModelFallbackExhausted`
     并列出每次尝试的 `model: reason`（复用既有 i18n 变体，不新增文案）。链由
     `parts.rs` 用既有的 `tier_chain_keys` 解析后注入，`capabilities` 不新增 `rustcode-config`
     依赖。测试：`member_walks_its_chain_in_order_and_stops_at_the_first_success`（**顺序**断言）、
     `exhausted_chain_reports_every_attempt`、`an_absent_chain_keeps_the_single_attempt_behaviour`
     （A-10）、`a_terminal_failure_does_not_walk_the_chain`（FR-5.3）、
     `a_member_that_produced_output_is_never_replayed`（A-5）。
  2. **[已覆盖] `parallel_edit_files`** —— `ParallelEditTool` 新增 `with_chain_providers(...)`：
     每个文件的子代理在**未产出任何内容**的失败后沿链换模型重试，规则与主回合、`task` 共用
     同一个 `fallback_eligible`（FR-6.1）。测试：`a_failed_child_walks_the_chain_in_order`、
     `an_absent_chain_attempts_only_the_primary`、`a_child_that_produced_output_is_never_replayed`。
     **诚实边界**：该工具在本仓**没有任何生产构造点**（全仓 `ParallelEditTool::new` 只出现在
     它自己的 `#[cfg(test)]` 里；模块文档即写明它是 opt-in、"constructed by the embedder"）。
     故本轮补的是**能力面**——embedder 一旦把它挂起来就自带回退；本仓既有的 CLI/TUI/daemon
     路径不会因此改变行为（它们根本不注册该工具）。
  3. **[已覆盖] `code_review` 子 agent** —— 它经 `ReviewTool` 进程内直连 kernel
     （`rustcode-review/src/review_tool.rs` 的 `Agent::run_to_completion`），不经过 owner 循环，
     因而复用宿主那台失败 provider，一个模型抖动就让整个评审失败。现 `ReviewTool` 新增
     `with_chain_providers(...)`：三个执行路径（single / deep fan-out / verify）统一改走
     `run_review_pass(...)`，按 `[宿主 provider, 宿主模型的显式链...]`（按 provider **身份**
     去重）逐跳尝试；换跳判据仍是共享的 `fallback_eligible`（已报告过 finding 的 pass
     **不重放**，否则会重复报告——评审工具的「产出」就是 finding 本身，比文本更严格）。
     宿主 provider 的取法不变（assemble 时经 `SharedReviewProvider` 槽注入，签名网关语义
     保持），链只是**追加**的候选。链由 `parts.rs` 用宿主 selection id 解析后注入，
     `rustcode-review` 不新增 `rustcode-config` 依赖。测试：`review_walks_its_chain_in_order`
     （**顺序**断言）、`an_absent_review_chain_attempts_only_the_host_provider`（A-10）、
     `a_review_that_reported_findings_is_never_replayed`（A-5）、
     `a_terminal_review_failure_is_not_eligible_for_fallback`（FR-5.3 + 取消不重放）。
     每跳经 `Msg::ModelFallbackStarted` 发到评审的 activity 通道（`REVIEW_ACTIVITY_MARKER`
     前缀，与既有进度行同面），复用既有 i18n 变体，不新增文案。
  - 注：`team` 的成员**已不再**与 `task` 各写一份候选去重逻辑——两处都按 provider **身份**
    （`Arc::ptr_eq`）去重，而非显示名：两个不同 provider 合法地可能暴露同一个原始模型名。
    这条规则目前是两处同构实现（编码上未抽公共函数，因两者候选来源与 tier 语义不同），
    若要进一步收敛应抽到 `capabilities` 的共享位置。
- **ACP 通道 = 机制已继承、事件面刻意不投影**：ACP 回合由同一 `CodingRuntime` 拥有
  （`rustcode-cli/src/acp/engine.rs` 的 `CodingRuntime::start` → `acp/turn.rs` 用
  `CodingRuntimeHandle` 驱动），故**模型中断可被吸收**；但协议侧无 advisory 通道，
  `AgentEvent::Warning` 在 `acp/translate.rs::event_to_update` 中**刻意不投影**（FR-7.4）。
  该决定由 `failover_advisory_is_deliberately_not_projected_to_acp_clients` 锁定：
  将来若改为投影，属**范围变更**而非修 bug。
- **子代理回退覆盖至此闭环**：主回合（owner 循环）、`task`、`team` 成员、
  `parallel_edit_files`、`code_review` 五条执行路径全部接通同一个
  `fallback_eligible` 与同一套链解析（`parts.rs`），没有任何进程内直连 kernel
  的子 agent 仍留在单次尝试语义上（在本仓已知的执行路径范围内）。

### 10.6 开放项裁决（2026-09-23，用户逐项裁决）

§8 的三个开放项已于 2026-09-23 经用户弹窗逐项裁决，三项均**维持现状**，开放项就此关闭：

- **O-1 默认行为 = 不默认启用**：回退仅在用户显式配置 `fallback` 链后生效；未配置时
  行为与无回退能力逐字相同（A-10）。不引入「默认回退到 `default_model`」的开箱行为。
- **O-2 触发时机 = 不允许重放已产出回合**：已产出文本或工具结果的回合终态后不换模型
  重放（A-5），避免重复副作用与用户可见的重复前缀。
- **O-3 链粒度 = 仅模型 id 级**：链由模型 selection id 组成；provider 级故障由用户显式
  列出多个 id 表达，不引入 provider 维度的跳转。

冻结证据（既有测试锁定，无需新增代码）：
- O-1 —— 五条执行路径各有 absent-chain 测试：`absent_chain_keeps_the_single_model_behaviour`
  （主回合）、`explicit_chain_is_tried_before_the_implicit_host_fallback` 与
  `model_less_non_hard_subtask_still_fails_over_along_its_tier_chain`（task）、
  `an_absent_chain_keeps_the_single_attempt_behaviour`（team）、
  `an_absent_chain_attempts_only_the_primary`（parallel_edit）、
  `an_absent_review_chain_attempts_only_the_host_provider`（code_review）。
- O-2 —— 各层 produced-output 不重放断言：`produced_output_suppresses_fallback`（主回合）、
  `a_member_that_produced_output_is_never_replayed`（team）、
  `a_child_that_produced_output_is_never_replayed`（parallel_edit）、
  `a_review_that_reported_findings_is_never_replayed`（code_review）、
  `produced_output_blocks_fallback`（共享判定单点）。
- O-3 —— `fallback_chain_flags_an_unknown_target`（链目标必须是已注册的模型 id）。



