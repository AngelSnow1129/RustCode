# 会话进度报告 — 模型回退主线与 TUI 交互修复（2026-09-23）

> 覆盖窗口：`d3cf5d92..HEAD`（23 个提交，未 push）。本文是本会话的交付总结与验证留档，
> 数据均来自 git 实测与 `/tmp/g3-*.log` 全量测试日志，非事后追记。

## 1. 总览

本会话围绕两条主线推进：**(A) 模型回退能力全链路落地**（需求文档、四层实现、五条执行
路径、开放项裁决），**(B) TUI 交互修复与防漂移收敛**（选择列表循环导航、出站单一工厂
收口、三处同构副本收敛单点）。全部工作按原子提交落地，每步经门禁验证。

| 维度 | 数值 | 来源 |
|------|------|------|
| 提交数 | 23（`d3cf5d92..HEAD`） | `git rev-list --count` |
| 改动量 | 51 文件，+5898 / -535 | `git diff --shortstat d3cf5d92..HEAD` |
| 提交类型 | 8 feat / 4 fix / 3 refactor / 7 docs / 1 style | `git log --format=%s` 分组 |
| 触及 .rs 文件 | 67 个路径 | `git log --name-only` 计数 |
| G3 全量终值 | **5635 passed / 0 failed / 12 ignored**（94 套件全绿） | `/tmp/g3-dedupe.log` |
| 分支状态 | `dev` 领先 `origin/dev` **33 个提交**（会话前 10 + 本会话 23），未 push | `git status -sb` |
| 工作区遗留 | 36 个文件，**全部属并发会话的 IM 集成 WIP**，本会话零卷入 | `git status --short` 归属核对 |

主线之外完成了三类收口：出站 HTTP 单一工厂补齐阻塞变体并清零全部手写客户端（4 处）；
候选链去重、walk-skip 列表两处同构副本收敛到单一事实源；存量 clippy warning 归零
（G2 `-D warnings` 门禁无假红风险）。

## 2. 模型回退主线（核心需求，已闭环）

需求原话：模型不可用后，能够按照设定的模型进行回退并继续使用，保持问题的可用性。
落地为 `docs/model-fallback-requirements.md`（FR-1..FR-7 / A-1..A-12 / R-1..R-7 /
§10 实施记录），实现分四层：

### 2.1 四层落点

| 层 | 提交 | 内容 |
|----|------|------|
| L0 配置 | `385e27c0` | `ModelProfileConfig.fallback: Vec<String>`（非 Option 字段，旧配置零改动可用）、`MAX_MODEL_FALLBACK_CHAIN = 4`、`validate_model_fallback_chains()`（fail-closed 报诊断）、`model_fallback_chain()`（读取路径永不失败）、`CfgDiagFallback*` i18n 五变体；`docs/config.example.toml` 补注释示例 |
| L1 共享判定 | `2bdbedf9` | `capabilities/src/fallback.rs`：`fallback_eligible` / `fallback_eligible_for_stop` 单点判定（取消不回退 / 已产出不回退 / 结构化分类优先 / 未分类仅放行瞬时 HTTP 类） |
| L2 运行时 | `2bdbedf9` | `coding/src/fallback.rs` 的 `FallbackWalk`（单调前进 + 显式耗尽）；`runtime.rs` owner 循环终态缝接线（重试预算耗尽后自发送 `ReassembleProvider` + `binding.pending_resume_prompt` 暂存问题重放，复用既有热切换路径，不建第二生命周期） |
| driver 面 | `2bdbedf9` | headless jsonl 回退 token 契约、daemon 线协议投影、ACP 刻意不投影（FR-7.4，测试锁定） |

关键语义（测试锁定）：换 provider 必须停旧 agent、原回合以 `Cancelled` 收尾、暂存问题
在新模型上重放；可观测判据 = 回退全程**零 `SessionChanged`** + `generation` 严格递增；
回退不是重试——仅在重试预算耗尽后触发。

### 2.2 校验接入生产（A-9）

- `cb808f2c`：`validate_model_fallback_chains` 此前零生产调用点，「显式报错」只在校验
  函数内部成立。现接入 `parse_disk_content_tolerant` 收尾，复用既有
  `Msg::CliConfigLoadWarnings` 启动警告通道；三个测试锁定（悬空目标可见 / 合法链零
  警告 / 追加语义不顶替既有 provider 警告）。
- 写盘侧（`ConfigStore` 落盘前拒绝**新增**坏链）由并发会话在 `store.rs` 实现，
  本会话结束时其代码与文档 hunk 均未提交（见 §7 边界）。

### 2.3 子代理五条执行路径全覆盖

| 路径 | 提交 | 接入方式 |
|------|------|---------|
| 主回合（owner 循环） | `2bdbedf9` | 终态缝 `try_model_fallback` |
| `task` 子代理 | `2bdbedf9` | `with_chain_provider` + tier selection ids；D-6 修复：单模型场景 `resolve_tier_keys` 返回 `None` 语义是「不路由」而非「无模型」，新增 `tier_chain_keys` 回退宿主 selection id，模型缺失子任务也能沿链回退 |
| `team` 成员 | `9941eeac` | `TeamRunnerFactory::with_chain_providers`，跳间重建 agent（FR-3.5 防旧 provider 残留），复用共享判定 |
| `parallel_edit_files` | `18f544e7` | `ParallelEditTool::with_chain_providers`；**诚实边界：本仓无生产构造点**（opt-in 能力面，embedder 挂载即得回退） |
| `code_review` 子 agent | `9e644b24` | `ReviewTool::with_chain_providers`；三条执行路径（single / deep fan-out / verify）统一抽 `run_review_pass`；宿主 provider 仍经 `SharedReviewProvider` 槽注入（签名网关语义保持），链是**追加**候选；已报告 finding 的 pass 不重放（评审的产出就是 finding 本身，比文本更严格） |

五条路径共用同一个 `fallback_eligible` 与同一套链解析（`parts.rs`，持有 `Config` 的层
解析后注入，capabilities 不新增 rustcode-config 依赖）。

### 2.4 开放项裁决（O-1/O-2/O-3，用户弹窗逐项裁决，已关闭）

`1e5ba93b`：三项均维持现状——O-1 不默认启用（未配链行为逐字不变，A-10）、O-2 不重放
已产出回合（A-5）、O-3 仅模型 id 级粒度。零代码变更，§10.6 逐条列出冻结证据测试名
（五条路径的 absent-chain 测试、五处 produced-output 断言、unknown-target 校验）。

### 2.5 候选链组装收敛单点

`afc6a66e` + `9ed20103`：FR-6.1 把「是否可回退」收敛到单点后，「候选列表如何组装」
仍是四处同构手写循环。收敛为 `capabilities::fallback::chain_candidates(primary,
extras)`（priority 序 + 按 provider 身份 `Arc::ptr_eq` 去重——同显示名的第二条可用
路由必须存活）；task 的「fallbacks 不含 primary」语义由 `skip(1)` 表达；review 侧本地
`review_candidates()` 整函数删除。行为零变更（63 + 11 + 105 + 7 项过滤测试全绿）。

## 3. TUI 交互修复

### 3.1 选择列表循环导航（用户需求：最上面再往上到最下面，最下面再往下到最上面）

- `7ba59a21`：新增 `modals::{step_up, step_down, step_up_by, step_down_by}` 单一实现
  （空表/单项安全、越界陈旧索引归一），接入 10+ 处模态选择列表（provider 列表、模型
  选择器、发现多选游标、目录/语言/代理选择、配置面板、插件管理、rewind 检查点+作用域、
  文件查看器、diff 文件列表、onboarding 语言与 setup 行）。**顺带修一处不一致**：流式态
  斜杠菜单不循环而空闲态循环，两处现同走 `step_*`。既有 8 个锁定「有界」的测试改写为
  锁定环绕（断言未削弱），新增 5 个 helper 单测。
- `194c3787`：审计发现上一轮漏接 `state.rs` 的回合内面板——三面板行为不一致
  （`ApprovalPanel` 既有循环，`UserInputPanel` / `RoundCapPanel` 仍有界）。统一改走
  `step_*`（含 multiple 模式 5 行环与 Other 行环形导航），三面板从此共用一份实现。
- **刻意保持有界**（循环在那里是 bug 不是特性）：文本光标、滚动视口、Home/End、
  过滤钳位、Enter 落行。
- `session_picker` 的环形包含搜索框（框 → 首个会话 → … → 回到框）。

### 3.2 模型发现入口补齐

`cf2d9af6`：账号行回车 = 钻入 + 后台拉取模型列表（`start_discovery_for_account` 共享
助手），新增列表刷新行（`REFRESH_MODELS_ROW`），无端点时给可见提示而非静默；传输设置
（UA/代理/跳过 TLS 校验）从已提交账号读取。修复两处自引入测试缺陷：夹具改本地 mock
server（不真连 `api.deepseek.com`）、死端口替代永久 `join()`。

## 4. 单点收敛(主线外收口)

### 4.1 出站 HTTP 单一工厂补齐阻塞变体

- `d9dfe190`:新增 `egress-blocking` feature 与 `build_blocking_http_client`,与异步侧共用同一
  `HttpClientSpec` 字段映射、同一份信任根收集与 webpki 兜底重试;TUI 模型发现(OAuth 登录/
  刷新同属离开 async 运行时的一次性调用)不再手写 `reqwest::blocking::Client`。
- `8a7d839c`:MCP OAuth 与 OpenRouter 两处阻塞客户端接入同一工厂。至此全仓手写客户端清零,
  仅存两类结构性豁免(leaf crate 无法依赖 capabilities;LLM 适配器打用户自供 base_url)。

### 4.2 walk-skip 列表收敛

- `6e4a6e80`:`fallback_eligible` 的 walk-skip 列表在 capabilities 两处各持一份同构副本,
  收敛到 `pathutil` 单一事实源(与 §2.5 的 `chain_candidates` 同一精神:先收敛判定单点,
  再消灭同构副本)。

## 5. 门禁清理

| 提交 | 问题 | 处置 |
|------|------|------|
| `2dae564b` | openrouter 注册测试含恒真断言(P \|\| !P),`overly_complex_bool_expr` 在 G2 `-D warnings` 下为 error | 删除该断言;"存在即可"的意图由上一行 `.expect(...)` 覆盖,校验不削弱 |
| `1bfba3db` | tuix/capabilities 存量 clippy warning(`else { if }` 收缩、De Morgan、重复 `cfg` 属性等) | 全部语义等价机械变换收敛归零,防 feature 组合/工具链升级后撞 `-D` 假红 |
| `70f43469` | parallel_edit 链序测试并发交织 flake(连续重复段折叠在共享尝试日志下假红) | 改首现序去重:每个子代理按链序走,任一模型首现必早于链上其后模型,任意交织下确定等于链序;连跑 5 次 14/0 稳定 |

## 6. G3 全量验证

`cargo test -j 1 --workspace --no-fail-fast`(8GB cgroup 强制 `-j 1`,且 daemon 固定端口
13456-13458 要求启动前确认无并发测试),日志 `/tmp/g3-dedupe.log`(2026-09-23 17:47):

| 指标 | 数值 |
|------|------|
| passed | 5635 |
| failed | 0 |
| ignored | 12(联网门控) |
| 汇总行 | 94 条 `test result: ok`,0 条 FAILED |

## 7. 边界与如实登记

- **FR-5 写盘侧**:本会话结束时,写侧门禁由并发会话在 `store.rs` 实现**且未提交**
  (§2.2 只落地了读侧启动警告通道)。2026-09-27 交接核验:`store::tests` 三个新测试
  (`a_newly_broken_chain_is_rejected_and_never_reaches_the_disk` /
  `a_valid_chain_persists_normally` / `unrelated_writes_pass_on_chainless_and_prebroken_configs`)
  全绿,与需求文档 A-9 补注一致。
- `validate_provider_accounts_and_models` 仍刻意不接写盘门控(无需求、交互式流程合法保存
  中间态),边界记录在 `store.rs` 注释。
- 会话结束时工作区遗留的 36 个文件属并发会话 IM WIP,本会话零卷入;该批工作已于后续会话
  以 IM 系列提交落地(`9e939206..224f7607`),不再滞留工作区。

## 8. 提交清单(23,旧 -> 新)

| # | 提交 | 类型 | 主题 |
|---|------|------|------|
| 1 | `385e27c0` | feat(config) | 新增模型回退链配置面与校验 |
| 2 | `2bdbedf9` | feat(coding) | 回合级模型回退与共享回退判定 |
| 3 | `cf2d9af6` | feat(tuix) | 账号行回车获取模型列表并新增刷新入口 |
| 4 | `fb77774a` | docs | 新增模型回退能力需求与实施记录 |
| 5 | `7ba59a21` | feat(tuix) | 选择列表导航首尾循环,去掉上下死路 |
| 6 | `cb808f2c` | fix(config) | 回退链校验结果接入启动警告通道 |
| 7 | `9941eeac` | feat(coding) | team 成员支持模型回退链 |
| 8 | `18f544e7` | feat(capabilities) | parallel_edit_files 子代理支持模型回退链 |
| 9 | `ab69b464` | docs | 回退需求文档同步 team/parallel_edit 覆盖现状 |
| 10 | `45f65be0` | docs(agents) | 补记回退共享判定的三个子代理调用方 |
| 11 | `194c3787` | fix(tuix) | 回合内三个交互面板统一为循环导航 |
| 12 | `9e644b24` | feat(review) | code_review 子 agent 支持模型回退链 |
| 13 | `a7e1641e` | docs | 回退需求文档同步 code_review 覆盖现状 |
| 14 | `4b5a9474` | docs(agents) | 补记 code_review 成为回退判定的第四个子代理调用方 |
| 15 | `1e5ba93b` | docs | 回退开放项 O-1/O-2/O-3 已由用户裁决关闭 |
| 16 | `2dae564b` | fix(tuix) | 删除 openrouter 注册测试中的恒真断言 |
| 17 | `70f43469` | fix(capabilities) | parallel_edit 链序测试按首现序去重,修并发交织 flake |
| 18 | `d9dfe190` | feat(capabilities) | egress 阻塞变体接入单一工厂,TUI 发现不再手写客户端 |
| 19 | `afc6a66e` | refactor(capabilities) | 候选链去重收敛到 chain_candidates 单点 |
| 20 | `9ed20103` | docs | 回退文档注记同步——候选去重已收敛到 chain_candidates |
| 21 | `8a7d839c` | refactor(capabilities) | MCP OAuth 与 OpenRouter 阻塞客户端接入 egress 单一工厂 |
| 22 | `1bfba3db` | style | tuix/capabilities 存量 clippy warning 收敛归零 |
| 23 | `6e4a6e80` | refactor(capabilities) | walk-skip 列表收敛到 pathutil 单一事实源 |

## 9. 现状补注(2026-09-27,非本会话窗口)

- `d3cf5d92..HEAD` 现含 **40** 个提交:本文的 23 个 + 后续会话 17 个(IM 渠道接入与 setup
  向导、schedule P0-P2、文件面安全判据与 webui 文件面板、v6.1.0 linux-arm64 补齐、v6.2.0
  版本号)。
- `dev` 现领先 `origin/dev` **4** 个提交未 push:`16f1b420` / `54283b91` / `5720296c` /
  `08890df5`(§1 表中"33 个提交未 push"为会话当时状态,origin/dev 此后已前移)。
- 工作区当前改动 5 文件:store.rs FR-5 写侧门禁(已验证)、install.sh 下载并发竞速
  (`bash -n` + stub curl 功能走查通过:并发波次、优先级胜出、串行回退、SPA HTML 壳拒收)、
  README 下载环境变量文档(与 install.sh 实现核对一致)、`.gitignore`(+`.omo/`)、本文档。
