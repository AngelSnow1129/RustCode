---
kind: requirement
id: REQ-002
from: requirements-analyst
to: [solution-architect]
feature: 2026-09-07-zh-docs-webui
status: approved
decision: proceed
requires: []
files_owned:
  - 00-requirement.md
architecture_constraints:
  touches_runtime_lifecycle: true
  touches_persistence: false
  touches_cross_crate_deps: false
created: 2026-09-07
---

# REQ-002 全仓 Markdown 汉化 + WebUI 开箱可访问 + 默认绑定 0.0.0.0

> **只读分析产物。** 本轮仅落盘本文件，未修改任何生产代码 / 被汉化 md / `Cargo.toml` / `AGENTS.md` / `docs/**`。
> **基线**：branch=`dev` commit=`3ee655e3` worktree=clean（引自 `.codebuddy/artifacts/2026-09-07-zh-docs-webui/STATUS.md:4`）。
> **工具限制说明**：本环境无 `Bash`/`AskUserQuestion` 工具，凡"近期 git 历史""实际运行"类判定均未执行，一律以 `文件:行号` 静态证据替代，并在 §8 标为待裁决。**本文件所有论断均给出可复现的 Grep/Read 位置。**

---

## 0. 复核结论摘要（含对编排者事实清单的修正）

编排者 `STATUS.md:15-25` 的 9 条事实，逐条复核结果：

| # | 编排者事实 | 复核结果 | 证据 |
|---|---|---|---|
| F1 | 默认 host 3 处 | **属实** | `crates/rustcode-daemon/src/main.rs:21`、`crates/rustcode-cli/src/main.rs:1047`、`crates/rustcode-cli/src/main.rs:1779-1780` |
| F2 | 非回环警告 | **属实** | `crates/rustcode-daemon/src/lib.rs:6345-6347` |
| F3 | 前端 provider UI 已完整 | **属实** | `webui/src/components/SettingsDialogs.tsx:15,17,290,749-750,779`（`setDefaultProvider` / `discoverProviderModels` / 增删改 / 设默认 / 发现模型） |
| F4 | 前端 i18n zh/en 全量 | **属实**（文件大小未逐行核） | `webui/src/i18n.ts` |
| F5 | 启动期无 provider 预检 | **属实** | `run_server` 自 `lib.rs:6097` 起至 `AppState` 构造 `:6141-6163`，`Config::load` 失败仅 `tracing::warn!` 后回落默认（`:6115-6121`），无任何 provider 校验 |
| F6 | 前端无无-provider 阻断遮罩 | **属实（弱证据）** | `webui/src/app.tsx` 全文件 grep `provider` 0 命中 |
| F7 | 真阻塞是 `webui/dist/` 缺失 | **属实** | `.gitignore:88` `dist/` + `crates/rustcode-daemon/src/webui.rs:15-17`（`#[folder = "../../webui/dist/"]` `#[allow_missing = true]`）+ `:25-27` `is_built()` + `cli/src/main.rs:1808-1811` |
| F8 | CORS 仅放行 loopback origin | **属实**，且结论正确 | `lib.rs:1222-1227` `.allow_origin(predicate(is_loopback_origin))`；页面由 daemon 同源提供（`lib.rs:6174-6175`），同源 fetch 不受 CORS 约束 |
| F9 | `client_interactive_permission` 耦合风险 | **属实，且比描述更严重**（见下） | `lib.rs:1284-1294` |

### 0.1 对 F9 的深化复核（本需求最高风险点，务必读完）

判定函数（`crates/rustcode-daemon/src/lib.rs:1284-1294`）：

```rust
fn client_interactive_permission(client_mode: ClientMode, enforce_token: bool, bind_host: &str) -> bool {
    enforce_token
        || (matches!(client_mode, Channel | Webui | Vscode | Jetbrains)
            && is_loopback_authority(bind_host))
}
```

`is_loopback_authority` 只认 `localhost` / `127.0.0.1` / `::1`（`lib.rs:1272-1279`），**`0.0.0.0` 不在其中**。

**翻转的真实后果不是"审批变不便"，而是静默提权。** 唯一消费点 `lib.rs:4327-4328`，下游 `lib.rs:4671-4674`：

```rust
runtime_cfg.dangerously_skip_permissions = approval_mode == ApprovalMode::Auto
    || (approval_mode == ApprovalMode::Build && !interactive_permission);
```

且 `ApprovalMode` 默认就是 `Build`（`crates/rustcode-daemon/src/live_api.rs:40` `LIVE_APPROVAL_MODE` 初值；`:2797` `assert_eq!(ApprovalMode::default(), ApprovalMode::Build)`）。

即：**Build 模式 + 无交互审批方 ⇒ `dangerously_skip_permissions = true`（工具执行全量自动放行，不再征求批准）。**

按入口逐一定性（关键差异在于 `enforce_token`）：

| 入口 | `enforce_token` | 改默认 host 后是否翻转 | 证据 |
|---|---|---|---|
| `rustcode webui`（进程内 webui） | **true**（`webui_tokens: Some(tokens)`，`lib.rs:5321`）→ `:6152` | **不翻转**（`enforce_token \|\|` 短路为 true，`startup_mode: ClientMode::Webui` 见 `:5319`） | `lib.rs:5281-5348` |
| TUI `/webui` | **true**（同上） | 不翻转 | `lib.rs:5482` |
| `rustcode daemon` 子命令 | **true**（`webui_tokens: Some(token_store)`） | 不翻转 | `cli/src/main.rs:1774-1784` |
| **独立 `rustcode-daemon` 二进制** | **false**（`webui_tokens` 为 `None`） | **翻转**（`Vscode`/`Jetbrains` + 非回环 ⇒ false） | `daemon/src/main.rs` `parse_daemon_args` 未传 `webui_tokens`（`main.rs:164-181` 区间） |

**且该二进制是被 IDE 拉起的**：
- VS Code：`extensions/vscode/src/daemon/process.ts:369` → `['--port', String(port), '--client', 'vscode']`，**没有 `--host`**，因此吃 `daemon/src/main.rs:21` 的 `DEFAULT_HOST`。
- JetBrains：`extensions/jetbrains/src/main/kotlin/com/rustcode/jetbrains/daemon/RustCodeDaemonProcess.kt:116` → `["--port", ..., "--client", "jetbrains"]`，同样**没有 `--host`**。
- 两者均发送 `X-RustCode-Client` 头（`extensions/vscode/src/daemon/client.ts:170,468`；`RustCodeDaemonClient.kt:382`），故 `client_mode` 确实为 `Vscode`/`Jetbrains`。

**净效应：若把 `daemon/src/main.rs:21` 一并改成 `0.0.0.0`，则 VS Code / JetBrains 拉起的守护进程将默认在 `0.0.0.0:13456` 上无鉴权暴露 `/chat`、文件编辑与工具执行端点，同时 Build 模式因审批方缺失而 `dangerously_skip_permissions=true`。这是本次改动唯一不可接受的组合。**

### 0.2 复核中新发现的三条事实（编排者清单未列）

- **N1 `startup_mode` 是死字段。** `ServerOpts.startup_mode`（`lib.rs:6042`）在 `run_server` 中永不读取（`run_server` 以 `..` 解构，`lib.rs:6109`），全仓仅 3 处写入（`lib.rs:5319`、`lib.rs:5482`、`cli/src/main.rs:1783`）与 1 处测试构造（`crates/rustcode-daemon/tests/daemon_token_auth.rs:16`）。⇒ **`--client` 启动参数对请求处理无任何影响**，客户端身份 100% 来自 `X-RustCode-Client` 请求头（`lib.rs:1244-1252`，缺省回落 `ClientMode::Ide`，而 `Ide` 不在交互白名单内）。改默认 host 时不要把"`--client` 已在启动期声明"当作安全依据。
- **N2 CLI `webui` 的现有未构建提示文案不完整。** `Msg::CliWebuiNotBuilt`（`crates/rustcode-config/src/i18n/zh_cn.rs:1135-1136`）给出 `cd webui && npm install && npm run build` + `cargo build -p rustcode`，但**缺** `cargo clean -p rustcode-daemon`，而 `AGENTS.md:16` 明确"重建前端后必须 `cargo clean -p rustcode-daemon`，cargo 不追踪 `webui/dist/` 变化"。文案也与 `AGENTS.md:16` 的 `npm ci` 不一致。
- **N3 已有回归测试锁定了"非回环 ⇒ 无交互审批"这一负向行为。** `lib.rs:8876-8890` 断言 `!client_interactive_permission(Vscode, false, "0.0.0.0")` 与 `!client_interactive_permission(Channel, false, "0.0.0.0")`。改默认值**不会**让该测试变红（它测函数而非默认值），⇒ 该测试**不能**作为"改默认 host 无回归"的证据，必须另行补测。

### 0.3 其余已核实事实

- **无 crate 通过 `include_str!` 嵌入 md**（`grep include_str!` 全仓仅命中 `crates/rustcode-review/src/rules.rs:110-156` 的 47 条）。⇒ 除 review 规则外，汉化不影响任何 Rust 编译产物内容。
- **setup-seeds 是构建期打包、运行期落地到用户 Home 的产物**：`crates/rustcode-capabilities/src/setup/seeds.rs:8` `include_bytes!(.../setup-seeds.tar.zst)`，解开到 `~/.rustcode/seeds-cache/<version>/`（`:23`），再拷进 `$RUSTCODE_HOME/skills/`（`setup/mod.rs:69,108-113`）；安装态由 `.seed-hash` 指纹判定（`setup/mod.rs:187,226-231`）。⇒ 改动这 6 个 md 会改 tar 内容哈希，**已安装用户会触发一次种子重装**（`setup/mod.rs:146-148` "seed skill updated -- reinstalling"）。
- **全仓 md 内锚点链接为 0**（`grep '\]\(#[a-z0-9-]+\)'` 全仓 `**/*.md` 0 命中）。⇒ 汉化标题**不会**打断仓内 md 互链（风险降级，但 §4.5 仍保留外部锚点检查）。
- **`README.zh-CN.md` 的唯一活引用是 `README.md:16`**（`<a href="./README.zh-CN.md">简体中文</a>`）。其余命中全在 `AGENTS.md:74,243,279` / `docs/platform-neutralization.md:143` / `docs/phase1-refactor-design.md:30,199,601` / `docs/codex-claude-analysis.md:64` —— 均为历史叙事散文，引用的是"该文件的某行内容"，**删除文件不会造成链接 404**，但也因此无法被链接检查器发现。
- **`site/`、`extensions/`、`.github/workflows/` 对 `README.zh-CN` 0 命中**；全仓无 `readme =` 出现在任何 `Cargo.toml`。
- **md 总量实测 287 个**（`Glob **/*.md`，已排除 `node_modules`/`target`），与编排者的 286 差 1（本 feature 目录新增的 `STATUS.md`）。**AC 一律用"脚本产出计数"而非硬编码数字。**
- **`webui/package.json` 要求 `node >= 22.6`**（`engines.node`），构建命令 `vite build`（无 `prebuild`），依赖安装需 `npm ci`。

---

## 1. 背景与问题

### 1.1 用户原始诉求（逐字引用）

> 将所有的文档等md进行汉化，并且多推进相应的webui，不需要设置provider就可以访问，方便在网页端可视化进行网页配置，并且webui默认使用0.0.0.0都可以访问这样也方便使用

### 1.2 现状

1. **文档语言割裂。** 全仓 287 个 md，约 168 个纯英文（含 `docs/superpowers/**` 68、`docs/plans/**`、`.codebuddy/**` 46 等历史归档）、约 60 个已是中文、6 个中英混排。而 fork 已确立**默认语言为简体中文**（`AGENTS.md:103`，`rustcode-config/src/locale.rs` 三处默认 `Locale::ZhCn`）。文档与产品默认语言不一致。
2. **README 双份。** 根 `README.md`（英文）与 `README.zh-CN.md`（中文）内容重复，长期双份维护，且 `README.md:16` 还挂着到 `README.zh-CN.md` 的导航链接。
3. **WebUI 起不来。** `webui/dist/` 被 `.gitignore:88` 的 `dist/` 忽略；`rust_embed` 用 `#[allow_missing = true]` 使编译通过（`webui.rs:15-17`），但运行期 `is_built()` 返回 false（`webui.rs:25-27`），`rustcode webui` 在 `cli/src/main.rs:1808-1811` 直接 `return Ok(1)`。前端其实**已具备完整的 provider 可视化配置能力**（`SettingsDialogs.tsx`），用户却根本到不了那一步。
4. **默认只绑回环。** 三处默认 host 均为 `127.0.0.1`，跨设备访问必须显式 `--host`。

### 1.3 为什么现在做

前三项是"已建成但用户拿不到"的交付断层；第四项是易用性。四项叠加后，本 fork 的中文用户从 `git clone` 到"在浏览器里配好 provider 开始用"之间存在硬性阻断。

---

## 2. 目标 / 非目标

### 2.1 目标

- **G1** 把全量纯英文 md（含历史归档）汉化为简体中文，且**不破坏任何代码/命令/标识符/链接**。
- **G2** 根 `README.md` 汉化为中文并成为唯一 README；删除 `README.zh-CN.md` 并清理其引用。
- **G3** WebUI 一键可构建：新增构建脚本，缺失资源时给出**可执行的**修复指引；前端产物不入库、`cargo build` 不自动跑 npm。
- **G4** 默认绑定改为 `0.0.0.0`，且不再因"默认即非回环"而每次启动打印安全警告。
- **G5** 保证"无 provider 也能打开 webui 并完成 provider 配置"这条链路端到端可用（**经复核该能力已存在，本需求主要是把它钉死为可验收行为**，见 F5/F6）。

### 2.2 非目标（显式排除，防止范围蔓延）

- **N-1** 不新增、不重写、不 Redesign WebUI 前端功能。provider 配置 UI 已存在且完整（F3），本需求**不要求**新增任何 UI 组件、不要求新增"首次运行向导"、不要求新增"无 provider 引导页"。
- **N-2** 不改 Rust 生产逻辑，除 §3 明列的 3 处默认 host 常量 + 1 处警告打印 + 1 处提示文案（N2）外，**不触碰任何 Rust 控制流**。
- **N-3** 不引入 token/TLS/反向代理等新的鉴权机制（属安全设计，超出本需求范围；§8 Q2 只要求"给出补偿方案"，不要求实现新鉴权）。
- **N-4** 不汉化 `site/**`（全部是 `.html`，不是 md）、不汉化 `extensions/**` 下的非 md 文件、不汉化代码注释、不汉化 i18n 的 `en.rs`。
- **N-5** 不汉化 A/B/C 类跳过清单（§3.1），其中 **`crates/rustcode-review/rules/*.md` 明确保留英文**。
- **N-6** 不校对、不润色、不改写已是中文的 60 个 md（唯一例外见 §3.5）。
- **N-7** 不做双语（中英并列）输出，采用**纯中文替换**（理由 §3.4）。
- **N-8** 不改 session 持久化格式、不改 daemon wire DTO、不改 `ClientMode`/`x-rustcode-client` 协议取值。
- **N-9** 不修改 `AGENTS.md`（它是 fork 长期约束，且已是中文）。
- **N-10** 不提交 `webui/dist/`、不在 `cargo build` 中触发 npm、不改 `.gitignore` 的 `dist/` 规则。
- **N-11** 不修复 N1（`startup_mode` 死字段）—— 只记录，留给独立 issue，避免与本需求耦合。

---

## 3. 汉化执行规范（强制性，逐条可验收）

### 3.1 跳过清单（不得汉化）

**A 类 — 法律 / 版权 / 第三方声明（逐字保留英文，一个字符都不动）**

判定规则：路径或文件名（大小写不敏感）含 `LICENSE`、`LICENCE`、`NOTICES`、`CREDITS`、`COPYING` 即跳过。当前确切命中：

- `docs/ORIGINAL_LICENSE.md`
- `docs/THIRD_PARTY_NOTICES.md`
- `docs/UPSTREAM_CREDITS.md`
- `docs/UPSTREAM_RUSTCODE_LICENSE.md`
- 根 `LICENSE`（无 `.md` 后缀，本身不参与；列出仅为说明遗漏风险为零）

理由：`AGENTS.md:99` 要求"新模块头部只追加本 fork 声明，**不得覆盖或删除任何既有版权行**"；许可证与第三方声明是法律文本，翻译件无法律效力且会与上游原文产生歧义。

**B 类 — 运行时 prompt 载荷（保留英文）**

- `crates/rustcode-review/rules/*.md`（47 个，文件名见 `Glob crates/rustcode-review/rules/*.md`）

**裁决：不汉化。** 理由（按权重排序）：

1. 它们是**编译期嵌入的模型输入**：`crates/rustcode-review/src/rules.rs:110-156` 用 `include_str!` 把 47 个文件逐个编进二进制，并在 `review_tool.rs:526-531` 拼进发给模型的 task（`\n\n{rules}\n\n{impact_plan}\n\n=== DIFF ===`）。改动 = 静默改变每次 code review 的模型输出行为，且**没有任何测试能发现**（`rules.rs` 的测试只断言 `render_rules_section` 的拼接形状，不断言规则正文，见 `rules.rs:379-398`）。
2. **已有官方逃生口**：`--rules-dir <dir>` 允许用 `<dir>/<name>.md` 同名覆盖任意内置规则（`rules.rs:6,228-237`）。需要中文规则的用户**无需改仓库**即可实现，成本最低、零回归风险。
3. 规则正文大量使用"N+1 query""TOCTOU""unwrap()""race"等**技术判定词**，中译后会削弱模型对模式的锚定。
4. `is_low_signal_file` 已把 `.md` 列为低信号（`rules.rs:290`），说明这些文件本身是"给模型读的规则"，不是"给人读的文档"。

> 该裁决与编排者"52 个建议跳过"的盘点自洽（47 rules + 5 法律 = 52）。若用户坚持全量，见 §8 Q1。

**C 类 — 逐字历史记录（保留英文）**

- `extensions/jetbrains/CHANGELOG.md`

裁决：不汉化。理由：逐条发布记录需与 GitHub Release Notes / 市场页逐字对应，翻译会造成版本间口径不一致且无增值。（该判定**不放进开放问题**，影响面极小且可复核；如需全量，把它从 C 类移除即可。）

### 3.2 只汉化说明文字、保留原文的对象（**白名单内一律不译**）

以下出现**在任何位置**（含正文、标题、表格单元格、列表项、图注、frontmatter 值）都**必须保持字节级原样**：

| 类别 | 示例 |
|---|---|
| 所有 fenced code block（` ``` ` / `~~~` 包裹）内的**全部内容，含注释与字符串** | ```bash cargo build -p rustcode ``` |
| 所有 inline code（`\`...\``）内的全部内容 | `client_interactive_permission` |
| 链接 URL、图片路径、`<img src>`、`mailto:` | `./docs/architecture.md` |
| HTML 标签与属性名/值 | `<a href="...">`、`<sub>` |
| YAML frontmatter 的**键名** | `name:`、`description:`、`allowed_tools:` |
| frontmatter 中作为**标识符**的值 | `name: setup`、`name: requirements-analyst`、`model: sonnet`、`tools:`、`allowed_tools:`、`user_invocable`、`argument_hint`、`enabled`、`enabledAutoRun` |
| crate 名 / 二进制名 / cargo 包名 | `rustcode-cli`、`rustcode-daemon`、`rustcode`（cli 的包名实为 `rustcode`，`AGENTS.md:13`） |
| 文件路径 / 目录名 | `crates/rustcode-daemon/src/lib.rs`、`~/.rustcode/config.toml` |
| 命令与子命令、CLI flag | `rustcode webui`、`--host`、`/provider`、`/model`、`--rules-dir` |
| 环境变量 | `RUSTCODE_HOME`、`RUSTCODE_DAEMON_IDLE_TIMEOUT` |
| 配置键 / TOML 表名 / 配置值字面量 | `[subagent]`、`max_concurrent`、`provider_type`、`base_url` |
| HTTP 方法 / API 路径 / 状态码 | `POST /chat/permission`、`GET /health`、`401` |
| `Msg` 变体名 / i18n key / 枚举变体 | `Msg::DaemonWarnNonLoopback`、`ClientMode::Webui`、`ApprovalMode::Build` |
| 工具名 / 事件名 / hook 名 / 类型名 / 函数名 | `code_review`、`CodingRuntime`、`ensure_server_and_open` |
| 产品名与命令名 | `RustCode`（不译）；`rustcode`（命令/crate 前缀，不译） |
| 第三方专有名词 | OpenAI、Anthropic、OpenRouter、Ollama、Tailscale、DeepSeek、GLM、Qwen、VS Code、JetBrains、GitHub |
| 提交类型 / 版本号 / 日期 / 数字 / 单位 | `feat:`、`v5.0.3`、`2026-09-07`、`13456` |

**允许汉化的 frontmatter 字段：仅 `description`**（它是自然语言路由提示；且 `.codebuddy/agents/requirements-analyst.md:3` 的 `description` 已是中文，存在先例）。但**必须在交付清单中逐条列出被改动的 `description`**，因为它会改变 agent/skill 的自动分派匹配行为（§8 Q5 的低风险项，不阻断）。

### 3.3 运行时载荷的两类处理（明确裁决）

| 载荷 | 是否汉化 | 裁决理由 |
|---|---|---|
| `crates/rustcode-review/rules/*.md`（47） | **否** | §3.1 B 类 |
| `crates/rustcode-capabilities/assets/setup-seeds/skills/**`（6） | **是（正文汉化）** | 见下 |

`setup-seeds` 6 个文件（`SKILL.md` + `references/{hooks-patterns,mcp-servers,plugins-reference,skills-reference,subagent-templates}.md`）**裁决：汉化正文**。理由：

1. 它们是**给用户读、给用户改的技能文档**，会被安装到 `$RUSTCODE_HOME/skills/`（`setup/mod.rs:69,108-113`），是产品交付面的一部分，不是纯内部 prompt。
2. 该目录下的文件本就被平台中立化改写过（`AGENTS.md:269` 记录了对 `SKILL.md` 的实质性编辑），**改动属于既有维护惯例**。
3. fork 默认中文（`AGENTS.md:103`），中文技能文档与中文用户一致。
4. 与 review rules 的关键区别：**review rules 无逃生口之外的替代**（会静默改每次 review 输出），而 skills 是用户可任意改写的本地文件，且改动可预期。

**边界（必须处理）**：
- frontmatter 的 `name: setup` **保留英文**（`SKILL.md:2`）—— 它是 skill 身份与目录名。
- 改动会使 `SEEDS_TARZST` 内容哈希变化（`seeds.rs:8,19`）⇒ 已安装用户**会触发一次种子重装**（`setup/mod.rs:136-156`）。这是预期的升级行为，不是 bug；但**若用户手工改过 `$RUSTCODE_HOME/skills/rustcode-automation-recommender/` 下的文件，会被覆盖**。实现方需在 release 说明中写明这一点。

### 3.4 双语 vs 纯中文替换

**裁决：纯中文替换（不保留英文原文段落，不做中英并列）。**

理由：

1. 双语会让 md 体积翻倍（约 1.95MB → ~4MB），且 `docs/superpowers/plans/2026-05-29-webui.md` 单文件已达 73KB，再翻倍不可维护。
2. 双语必然产生"哪一份是权威"的歧义，后续任何一次修订都必须同步两处，实际必然失同步——这正是当前 `README.md` / `README.zh-CN.md` 双份困境的复现（用户裁决 #2 本身就是在消除双份）。
3. fork 已确立中文为默认（`AGENTS.md:103`），英文原文可通过 git 历史获取，无需在正文冗余保留。

**唯一例外**：A 类法律文本（本就不译，天然保留英文原文）。

### 3.5 中英混排文件 与 已有中文文件

- **中英混排（6 个）：以段落为最小单位统一到中文。** 已是中文的段落**原样保留、不改写不润色**；英文段落按 §3.2 汉化。不在中文段落里"补译"，也不新增英文段落。
- **已有中文（约 60 个）：不校对、不润色、不改写**（非目标 N-6）。
  - **唯一例外**：其中若存在指向 `README.zh-CN.md` 的链接、或指向任何被删除文件的链接，必须修正（否则产生 404）。当前扫描显示此类引用只存在于**历史叙事散文**中（`AGENTS.md:74,243,279` 等引用的是"该文件第 N 行"，不是链接），故预期修正量 = 1（`README.md:16`）。

### 3.6 链接与锚点完整性规则

1. **不得修改任何相对链接的目标路径、锚点文本、文件名、目录名。** 只译链接的**显示文字**。
2. 全仓 md 内 `](#...)` 锚点链接实测为 **0**（§0.3），故汉化标题不会打断仓内互链。
3. **仍必须检查的外部面**（AC-8 覆盖）：`site/**/*.html`、`docs/**/*.md`、`extensions/**`、`README.md`、`.github/workflows/**` 中是否存在指向 md 标题锚点或 `README.zh-CN.md` 的引用。
4. 汉化**不得**重命名任何 md 文件（文件名是链接锚点，也是 `rules.rs` 的规则名来源）。

---

## 4. 验收标准（AC）

> 所有 AC 均为"可脚本判定"或"人工可明确判定（二值，无解释空间）"。判定命令以仓库根为 cwd；`git ls-files` 用于排除已忽略文件。

### 4.1 汉化覆盖与质量

- **AC-1（跳过清单准确）** 存在交付清单文件（实现阶段产出，如 `.codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/md-inventory.md`），列出：全仓 md 总数、跳过文件（逐路径 + 归类 A/B/C）、待汉化文件（逐路径）、已中文文件。判定：`清单中 跳过数 + 待汉化数 + 已中文数 = 全仓 md 总数`，且 A 类恰好等于 §3.1 A 类 4 个路径、B 类恰好等于 47 个 `crates/rustcode-review/rules/*.md`、C 类恰好等于 `extensions/jetbrains/CHANGELOG.md`。
- **AC-2（英文残留率 ≤ 5%）** 对每个"待汉化"文件，按以下规则计算：剥离 YAML frontmatter、fenced code block、inline code、URL、HTML 标签、表格分隔行后，令 `total` = 剩余非空行数，`en` = 其中"含 ≥4 个连续 ASCII 字母且不含任何 CJK 字符"的行数。判定：**每个文件 `en / total <= 0.05`**。
- **AC-3（残留白名单）** 对 AC-2 中所有 `en > 0` 的文件，产出残留行清单（文件:行号 + 原文）。人工抽检 20 行（不足 20 则全检），**100% 必须落入 §3.2 白名单类别**，否则不通过。
- **AC-4（标识符零损坏 · 机械判定）** 对本次改动的所有 md，抽取"改动前的 inline code / fenced code 全部内容"与"改动后"做集合对比，判定：**集合完全相等**（不增不减不改）。等价脚本判定：对改动文件跑 `git show HEAD:<path> | grep -oE '`[^`]+`' | sort -u` 与改动后同命令，diff 为空。
- **AC-5（运行时载荷判定）** 判定以下两个不变式同时成立：
  - (a) `git diff HEAD -- crates/rustcode-review/rules/` **为空**；
  - (b) `git diff HEAD -- crates/rustcode-capabilities/assets/setup-seeds/` **非空，且 diff 不触及任何 frontmatter 的 `name:` 行**（判定：`git diff -U0 ... | grep -E '^[+-]name:'` 无输出）。
- **AC-6（frontmatter 完整性）** 对所有含 YAML frontmatter 的改动文件，判定：frontmatter 键名集合与改动前**逐字相同**（`git diff` 中不出现 `^[+-][a-zA-Z_]+:` 形式的键名变更）；`description` 若有改动，已在交付清单中逐条列出并注明"影响 agent/skill 自动分派匹配"。
- **AC-7（链接与文件名零变更 · 机械判定）** 对本次改动的所有 md：
  - (a) 无任何 md 文件被重命名/删除（A/C 类与 `README.zh-CN.md` 除外）；
  - (b) `git diff` 中**不出现** `^[+-].*\]\(.*\)` 形式的链接目标改动（允许仅显示文字变化：链接目标子串必须逐字保留）—— 判定方式：抽取改动前后所有 `](...)` 内的目标串为多重集合，二者相等。
- **AC-8（外部引用同步）** 判定以下 grep 全部为 **0 命中**：
  - `grep -rn "README\.zh-CN" --include=*.md --include=*.html --include=*.json --include=*.ts --include=*.kt --include=*.yml --include=*.toml .`；
  - `grep -rn "README\.zh-CN" site/ .github/ docs/ extensions/`（双重覆盖，含非上述后缀）。

### 4.2 README

- **AC-9（README 唯一化）** 判定同时成立：
  - (a) `git ls-files README.zh-CN.md` 输出为空（文件已从版本控制删除）；
  - (b) `test ! -e README.zh-CN.md` 为真；
  - (c) `README.md` 存在，`git diff` 显示其内容相对改动前**非空变更**。
- **AC-10（README 已汉化）** 对 `README.md` 按 AC-2 规则计算，判定 `en / total <= 0.05`。
- **AC-11（导航链接清理）** `README.md` 中不再出现指向 `README.zh-CN.md` 的链接；原有的中英切换导航（如 `English · 简体中文`）已移除或改为指向英文历史（git tag / 上游仓库），且**不得指向 404 地址**。判定：`README.md` 内所有 `href` 目标，非外链者逐一 `test -e` 通过。

### 4.3 WebUI 构建脚本与资源缺失指引

- **AC-12（脚本存在且可执行）** `scripts/build-webui.sh` 存在、`test -x` 为真、`bash -n` 通过（对齐 `AGENTS.md:256` 既有脚本验证惯例）。
- **AC-13（无 node/npm 时 fail-closed）** 在 `PATH` 中移除 node/npm 后运行脚本：判定 **exit code ≠ 0**，stderr 含可执行修复指引，且指引中**明确出现** `node` 最低版本要求（必须与 `webui/package.json` 的 `engines.node`（`>=22.6`）一致）。且运行后 `test ! -e webui/dist/index.html` 为真（不产生半成品）。
- **AC-14（node 版本不足时 fail-closed）** 以 `node` 版本 < 22.6 运行：判定 **exit code ≠ 0**，输出含当前版本与要求版本。**不得**降级为警告后继续。
- **AC-15（dist 行为）**
  - (a) 默认（无参数）：**重跑构建**（`npm ci` + `vite build`），幂等；跑完 `test -f webui/dist/index.html` 为真。
  - (b) 带 `--if-missing` 且 `webui/dist/index.html` 已存在：**跳过构建**，exit 0，输出说明。
  - (c) 带 `--if-missing` 且 `webui/dist/index.html` 不存在：正常构建。
- **AC-16（不入库 / 不自动 npm）**
  - (a) `git check-ignore -v webui/dist/index.html` **有输出**（被忽略）；`git status --porcelain webui/dist` 为空。
  - (b) `grep -rn "npm\|npx" crates/*/build.rs` **0 命中**（`cargo build` 不触发前端构建）。
  - (c) `.gitignore` 的 `dist/` 规则未改动（`git diff HEAD -- .gitignore` 为空）。
- **AC-17（提示文案可执行）** `Msg::CliWebuiNotBuilt`（`crates/rustcode-config/src/i18n/zh_cn.rs:1135`）的文案满足：
  - (a) 指向新脚本（出现 `scripts/build-webui.sh`）或给出等价的两步命令；
  - (b) **包含** `cargo clean -p rustcode-daemon`（修正 N2 缺失项）；
  - (c) 与 `AGENTS.md:16` 口径一致（`npm ci` 而非 `npm install`，或二者均提及）；
  - (d) 中文文案与英文文案（`en.rs` 对应条目）描述的命令步骤**完全一致**。判定：人工比对两语种文案中的命令行，逐条相等。
- **AC-18（端到端）** 在干净 worktree 上依次执行 `scripts/build-webui.sh` → `cargo clean -p rustcode-daemon` → `cargo build` → `cargo run -p rustcode -- webui`，判定：命令不因 `is_built()` 为 false 退出（`cli/src/main.rs:1808` 不触发），`curl -sSf http://127.0.0.1:13457/health` 返回 200，且 `curl -sSf http://127.0.0.1:13457/` 返回 HTML（含 `<html`）。

### 4.4 默认绑定 0.0.0.0

- **AC-19（默认值已改）** 判定：`grep -n '127\.0\.0\.1' crates/rustcode-daemon/src/main.rs crates/rustcode-cli/src/main.rs` 的命中**不再包含** `daemon/src/main.rs:21`（`DEFAULT_HOST`）、`cli/src/main.rs:1047`（`webui --host` default）、`cli/src/main.rs:1779-1780`（`daemon` 子命令硬编码）这 3 处；改为 `0.0.0.0`。
  > **范围以 §8 Q2 的裁决为准**：若 Q2 选"仅改 webui 两处"，则本 AC 只约束 `cli/src/main.rs:1047` 与 `cli/src/main.rs:1779-1780`，且 `daemon/src/main.rs:21` **必须保持 `127.0.0.1`** —— 判定条件相应取反。
- **AC-20（显式 `--host` 覆盖默认值）** 判定：`rustcode webui --host 127.0.0.1`（及 `--host=127.0.0.1`）时，进程实际监听地址**为 `127.0.0.1`**，不是 `0.0.0.0`。机械判定：`ss -ltnp`（或等价）显示监听 `127.0.0.1:<port>` 且**无** `0.0.0.0:<port>`。
- **AC-21（非回环警告不再打印）** 判定：`rustcode daemon` 与独立 `rustcode-daemon` 二进制以**默认参数**（不传 `--host`）启动时，stderr/stdout **不包含** `Msg::DaemonWarnNonLoopback` 的中文或英文文案（关键词：`非回环` / `non-loopback`）。同时 `grep -n "DaemonWarnNonLoopback" crates/rustcode-daemon/src/lib.rs` 的**生产调用点**（原 `:6345-6347`）已移除。
  > 文案 `Msg::DaemonWarnNonLoopback` 变体本身**是否删除**由 §8 Q3 决定；本 AC 只约束"不再打印"，不约束变体存废。
- **AC-22（跨设备可达）** 在局域网内另一台主机上，对运行 `rustcode webui`（默认参数）的主机执行 `curl -sSf http://<LAN-IP>:<port>/health`，判定返回 **200**；且 `curl -sSf http://<LAN-IP>:<port>/` 返回 HTML。
- **AC-23（同源页面不受 CORS 影响）** 以 `http://<LAN-IP>:<port>/` 在浏览器打开并完成 `?token=` → Cookie 交接后，页面内对同源 `/health`、`/project`、`/providers` 的 fetch 全部 2xx。判定：浏览器 DevTools Network 中**无** CORS 错误（`lib.rs:1222-1227` 的 `is_loopback_origin` 对同源请求不生效）。

### 4.5 无 provider 可访问（G5）

- **AC-24（无 provider 可启动）** 前置：`RUSTCODE_HOME` 指向空目录、`config.toml` 中**无任何 `[[providers]]`**。操作：`scripts/build-webui.sh` → `cargo run -p rustcode -- webui`。判定：进程不因 provider 缺失退出；`GET /health` 200；`GET /` 200 且返回 HTML。
- **AC-25（无阻断遮罩）** 前置同 AC-24。操作：浏览器打开首页。判定：页面**不出现**"必须先配置 provider 才能继续"类全屏阻断；可正常打开设置对话框（`webui/src/components/SettingsDialogs.tsx`）并看到 provider 列表（空态文案允许，如"暂无 provider"）。
- **AC-26（可在网页端完成配置）** 前置同 AC-24。操作：在设置对话框新建 provider（填 name / base_url / api_key / model）→ 设为默认。判定：`~/.rustcode/config.toml` 中出现对应的 `[[providers]]` 条目与 `is_default`（或等价字段）；`GET /providers` 返回该条目；无需重启 CLI/TUI。

### 4.6 耦合风险补偿（最高优先级 AC）

- **AC-27（审批不静默 bypass）** 对**独立 `rustcode-daemon` 二进制**以默认参数（模拟 VS Code 拉起：`--port 13456 --client vscode`，不传 `--host`）启动，用带 `X-RustCode-Client: vscode` 的请求在 **Build 模式**下发 `POST /chat`（触发一次需要审批的工具调用）。判定：**必须满足其一**：
  - (a) 该 turn 走交互审批路径（注册了 permission responder，事件流中出现 permission 请求事件，**工具未自动执行**）；或
  - (b) 明确拒绝服务并返回可机读错误（非 5xx 静默成功）。
  **判定为不通过的情形**：工具在未获批准的情况下被自动执行（即 `dangerously_skip_permissions` 为 true 且无审批事件）。
  > 依据：`lib.rs:4671-4674` + `live_api.rs:40`（默认 `Build`）。**该 AC 是本次改动的回归闸门，不得跳过。**
- **AC-28（非回环无鉴权判定被显式覆盖）** 判定：`lib.rs` 的 `client_interactive_permission`（`:1284-1294`）在"默认 host 已改为 `0.0.0.0`"的前提下，对 `ClientMode::{Vscode,Jetbrains,Channel,Webui}` 且 `enforce_token == false` 的组合，其返回值**不再由 `is_loopback_authority(bind_host)` 单独决定**——即存在显式补偿（改判定逻辑 / 或该路径改为 token 保护 / 或该入口不跟随默认 host）。补偿方案由 `01-design.md` 给出，本 AC 只判"存在且覆盖到上述 4 个 ClientMode"。
- **AC-29（现有测试不被削弱）** `crates/rustcode-daemon` 的 `channel_mode_tests::known_clients_interactive_on_loopback_or_token`（`lib.rs:8854-8891`）**保持通过**，且其断言内容不被删改（判定：`git diff` 中该函数体内无 `^[+-]\s*assert` 净减少）。若补偿改变了语义，必须**新增**测试而非修改该测试。
- **AC-30（默认 host 判定有测试锁定）** 新增至少一个单测，断言"独立 daemon 二进制解析默认参数得到的 host"等于预期常量（防止后续再次漂移）。判定：`cargo test -p rustcode-daemon` 中出现该测试名且通过。

### 4.7 全局门禁

- **AC-31** `cargo fmt --check` 通过（G1）；`cargo clippy --workspace --all-targets` 无**新增**告警（G2，存量 ~420 条豁免）；`cargo test --workspace --no-fail-fast` 结果中，**唯一允许的红测**是 `AGENTS.md:226` 已文档化的 `mcp::registry::tests::trust_key_golden_matches_core_algorithm`（G3）。
- **AC-32** 汉化改动**不得**引入新的 Unicode Emoji（`AGENTS.md:59,172`）。判定：对本次改动的所有 md 跑 `grep -nP '[\x{1F300}-\x{1FAFF}\x{2600}-\x{27BF}\x{FE0F}]'`，命中数 **不增加**（相对 `git show HEAD:<path>` 的基线）。**不得**"顺手"清理既有 emoji（沿用 `AGENTS.md:244` 的有界例外）。
- **AC-33** webui 前端回归：`cd webui && npx tsc --noEmit` 0 错、`npm test` 全通过、`npm run build` 成功（对齐 `AGENTS.md:332` 既有验证口径）。

---

## 5. 异常与非功能场景

| # | 场景 | 期望行为 | 依据 / 备注 |
|---|---|---|---|
| E1 | 构建脚本在无 node/npm 环境运行 | exit ≠ 0；stderr 给出含 `node >= 22.6` 的安装指引；**不创建** `webui/dist/` | AC-13；`webui/package.json` `engines.node` |
| E2 | node 版本 < 22.6 | exit ≠ 0，打印当前版本与要求版本，不降级继续 | AC-14 |
| E3 | `npm ci` 失败（离线 / registry 不可达） | exit ≠ 0；提示包含"离线"可能性与 `--if-missing`（若 dist 已存在）；不静默成功 | 新增行为 |
| E4 | `vite build` 失败（TS 错 / OOM） | exit ≠ 0；原样透传构建器 stderr；**不清空**已存在的旧 `webui/dist/` 之外的东西 | 新增行为 |
| E5 | `webui/dist/index.html` 已存在 | 默认重跑覆盖；`--if-missing` 跳过 | AC-15 |
| E6 | 磁盘空间不足导致 dist 半产出 | 脚本以非 0 退出；`rustcode webui` 因 `is_built()` 仍返回合理结果（dist 存在但 index.html 缺失 ⇒ 视为未构建） | `webui.rs:25-27` 以 `index.html` 判定 |
| E7 | `rustcode webui` 在 dist 缺失时 | 保持现状：exit 1 + 打印 `Msg::CliWebuiNotBuilt`，且文案指向新脚本并含 `cargo clean -p rustcode-daemon` | `cli/src/main.rs:1808-1811`；AC-17 |
| E8 | 重建前端后忘记 `cargo clean -p rustcode-daemon` | 二进制仍嵌旧 dist；脚本在成功结尾**主动提示**该命令 | `AGENTS.md:16`；AC-17(b) |
| E9 | `--host` 显式指定（`127.0.0.1` / 具体 IP / `::1`） | 完全覆盖默认值，绑定到指定地址 | AC-20 |
| E10 | `--host` 传非法值（非 IP/主机名、空串） | 绑定失败并给出可读错误；**不得**静默回落到 `0.0.0.0`（否则等于无意暴露） | 新增判定；`bind_scanning` 路径 |
| E11 | 端口被占用 | 沿用现有向上扫描（`bind_scanning(host, port, 100)`，`lib.rs:5298`）；改用实际端口生成 URL | 已有行为，不变 |
| E12 | 默认 `0.0.0.0` 下 token 鉴权（进程内 webui / `rustcode daemon`） | token 仍强制（`enforce_token=true`，`lib.rs:6152`）；URL 中的一次性 token → HttpOnly Cookie 交接不变 | `lib.rs:6165-6175` |
| E13 | 默认 `0.0.0.0` 下**独立 daemon 二进制**（`enforce_token=false`） | **不得**在默认路径下发生"无鉴权暴露 + 审批静默 bypass"的叠加。由设计阶段裁决（§8 Q2） | AC-27 / AC-28 |
| E14 | 交互审批不可用时（非回环 + 无 token） | 不得静默 `dangerously_skip_permissions=true`；须注册审批方或显式拒绝 | `lib.rs:4671-4674`；AC-27 |
| E15 | 无 `X-RustCode-Client` 头的请求 | `client_mode` 回落 `ClientMode::Ide`（`lib.rs:1250`），本就不在交互白名单；行为与改动前一致，不得变 | N1 |
| E16 | 主机无局域网 IPv4（`primary_lan_ipv4()` 返回 None） | 浏览器打开地址回退 `127.0.0.1`（`lib.rs:5369-5377`）；不得打开 `0.0.0.0`（不可路由） | 已有逻辑，需回归 |
| E17 | 汉化后 md 内相对链接 / 锚点 | 链接目标与文件名零变更（AC-7）；全仓 `](#...)` 现为 0（§0.3）；外部 `site/`/CI 引用需扫（AC-8） | |
| E18 | `README.zh-CN.md` 删除后的引用 | 唯一活引用 `README.md:16` 已清理；`AGENTS.md`/`docs/**` 中的命中均为历史叙事散文（非链接），**无需改**，但不得新引入 | AC-8；§0.3 |
| E19 | 浏览器从 `localhost` 访问而 daemon 绑 `0.0.0.0` | 可访问（`0.0.0.0` 监听集合含回环）；Cookie 交接正常 | AC-22 的互补场景 |
| E20 | 公网/不可信网络下默认 `0.0.0.0` | 至少保留一处风险提示（§8 Q3 推荐保留 `WebuiLanWarning`）；**不提供 TLS**，须提示用隧道/反向代理 | `zh_cn.rs:1897-1898` |
| E21 | seed skills 内容变更导致 hash 变化 | 已安装用户触发一次重装，可能覆盖用户手工修改 | `setup/mod.rs:136-156`；release 说明写明 |
| E22 | 并发/大规模并行汉化（多 agent 同时写同一 md） | 单文件单 owner，禁止并发写同一文件；分批提交，每批可独立通过 AC-2 | 流程约束 |
| E23 | 已安装用户升级后 seed 重装失败（权限/磁盘） | 重装失败须报错而非静默；不得破坏既有 `$RUSTCODE_HOME` | |
| E24 | 权限不足（`~/.rustcode` 出现 root 属主） | 沿用铁律：禁止 `sudo` 运行，脚本须检测并提示 | `AGENTS.md:18`、`AGENTS.md:269` |

---

## 6. 受影响范围（初步）

### 6.1 汉化（文档面，无代码行为变更）

| 目录 | 说明 |
|---|---|
| `docs/**` | 含 `docs/superpowers/**`（68，历史 plans/specs）、`docs/plans/**`、`docs/archive/**`、`docs/adr/**`、根级 30+ 篇 |
| `.codebuddy/**` | `agents/*.md`（7，含 YAML frontmatter）、`rules/*.md`、`artifacts/**`（历史交接件，46 个中的绝大部分） |
| `crates/*/README.md`、`crates/rustcode-kernel/SPIKE.md`、`crates/rustcode-review/{README,LANGUAGES}.md`、`crates/rustcode-clix/README.md` | |
| `webui/README.md`、`docker/README.md`、`extensions/jetbrains/README.md` 等 | |
| 根 `README.md` | 汉化（G2） |
| `crates/rustcode-capabilities/assets/setup-seeds/skills/**`（6） | 汉化正文，**运行时载荷**（§3.3） |
| `.claude/plans/**`、`.goals/**`、`.superpowers/**` | 历史归档，全量纳入 |

### 6.2 代码面（预期极小）

| 文件:行 | 改动 | 备注 |
|---|---|---|
| `crates/rustcode-daemon/src/main.rs:21` | `DEFAULT_HOST` → `0.0.0.0`（**取决于 §8 Q2**） | 独立 daemon 二进制 |
| `crates/rustcode-cli/src/main.rs:1047` | `webui --host` default → `0.0.0.0` | 进程内 webui |
| `crates/rustcode-cli/src/main.rs:1779-1780` | `daemon` 子命令 `host` → `0.0.0.0`（**取决于 §8 Q2**） | 进程内 daemon |
| `crates/rustcode-cli/src/main.rs:1045-1046` | `--host` 的 doc comment 同步（现写 "default 127.0.0.1; use 0.0.0.0 to expose..."） | 必须同步，否则 `--help` 失实 |
| `crates/rustcode-daemon/src/lib.rs:6345-6347` | 移除/条件化 `DaemonWarnNonLoopback` 打印 | 取决于 §8 Q3 |
| `crates/rustcode-daemon/src/lib.rs:6333-6341` | 该段注释声称"Default to loopback-only for security"并引用 PR #82，与新的 `0.0.0.0` 默认**直接矛盾**，须改写 | 一致性必修 |
| `crates/rustcode-daemon/src/lib.rs:1284-1294` + `:4327-4328` + `:4671-4674` | **可能需要**补偿（AC-27/28） | 由 `01-design.md` 定 |
| `crates/rustcode-config/src/i18n/zh_cn.rs:1135-1136`、`en.rs` 对应条目 | `CliWebuiNotBuilt` 文案补 `cargo clean -p rustcode-daemon` 并指向新脚本 | N2 |
| `docs/**` 中提到默认 `127.0.0.1` 的章节 | 需同步（否则文档与行为矛盾） | 实现期 grep 兜底 |
| **新增** `scripts/build-webui.sh` | 一键构建脚本 | AC-12 |
| `AGENTS.md:16` | 可补一句"或用 `scripts/build-webui.sh`" | **可选**；本需求不强制改 `AGENTS.md`（N-9） |

### 6.3 影响的入口

| 入口 | 是否受影响 | 说明 |
|---|---|---|
| CLI（`rustcode webui` / `rustcode daemon`） | **是** | 默认 host、`--help` 文案、未构建提示 |
| daemon（独立二进制 `rustcode-daemon`） | **是（高风险）** | DEFAULT_HOST；被 IDE 拉起且无 token |
| TUI（`/webui`） | 部分 | `ensure_server_and_open` 传的 host 来源需确认是否跟随新默认；`enforce_token=true` 故审批不翻转 |
| headless / background | 否 | |
| ACP | 否 | |
| clix | 否 | |
| VS Code 扩展 | **是（间接）** | 拉起 daemon 时不传 `--host`，吃默认（`process.ts:369`） |
| JetBrains 插件 | **是（间接）** | 同上（`RustCodeDaemonProcess.kt:116`） |
| `site/` 文档站 | 否（但需扫 `README.zh-CN` 引用） | |

---

## 7. 架构敏感面标记

> 逐条对照 `AGENTS.md:181` 要求的"持久化 / 公共协议 / 安全边界 / 运行时生命周期"检查面。

| 敏感面 | 是否触碰 | 结论与依据 |
|---|---|---|
| **持久化格式** | **否** | 不改 session 持久化模型（`AGENTS.md:139` native `SessionManager/SessionMeta/SessionSnapshot` 唯一模型）、不改 `config.toml` schema、不改 daemon wire DTO、不改 `~/.rustcode` 布局。setup-seeds 的 `.seed-hash` 变化只触发重装，**不改格式**（`setup/mod.rs:187,226-231`）。故信封 `touches_persistence: false`。 |
| **公共协议** | **否** | `x-rustcode-client` 头取值、`ClientMode` 枚举、`/chat*`、`/providers` 等路径与 HTTP 方法均不变。CORS 策略不改（`lib.rs:1222-1227`）。 |
| **安全边界** | **是（本次最高风险）** | ① 默认绑定从 loopback 放宽到 `0.0.0.0`；② 独立 daemon 二进制路径 `enforce_token=false`（`lib.rs:6152`），无鉴权暴露 `/chat` + 文件编辑 + 工具执行；③ 见下"运行时生命周期"的审批翻转，二者叠加 = 无鉴权 + 自动放行的工具执行。**必须在 `01-design.md` 出具补偿方案。** 依据：`lib.rs:6177-6178`（"独立 daemon/VSCode（enforce_token=false）中间件直接放行"）、`daemon/src/main.rs:21`、`extensions/vscode/src/daemon/process.ts:369`。 |
| **运行时生命周期** | **是** | `client_interactive_permission` 的翻转直接改变 approval 路径（`lib.rs:4327-4328` → `:4630` `registered_permission_responder` → `:4671-4674` `dangerously_skip_permissions`）。`AGENTS.md:149-156` 明确把 **approval** 列为 Runtime 生命周期不变量检查项，且要求"pending approval 在 cancel/reload/session switch/shutdown 时 fail-closed"。当前代码在"无审批方"时是 **fail-open（静默放行）**，与本 fork 的 fail-closed 原则相悖 —— 无论本次是否改 host，该点都值得独立 issue（记为 **N4**，不属本需求范围）。故信封 `touches_runtime_lifecycle: true`。 |
| **跨 crate 依赖方向** | **否** | 改动落在 `rustcode-cli` / `rustcode-daemon` / `rustcode-config` 内，不新增依赖边、不反向依赖（`AGENTS.md:49` 方向不变）。故信封 `touches_cross_crate_deps: false`。 |
| **兼容面退役 / core 回流** | **否** | 本需求不触及 `rustcode-core` 遗留、不引入 bridge / v1/v2 开关 / fallback，符合 `AGENTS.md:168`。 |
| **运行时 prompt 载荷** | **是（弱）** | setup-seeds 6 个 md 汉化会改变注入内容（§3.3）；review rules 47 个**明确不汉化**以规避。 |

**强制架构评审项**：安全边界 + 运行时生命周期两条为 true，`01-design.md` 必须就 §8 Q2 给出显式裁决并附 AC-27/28/29 的落地方式。

---

## 8. 未决歧义（含推荐项，需编排者/用户裁决）

> 本环境无 `AskUserQuestion` 工具，无法在落盘前逐条征询；以下 5 项按"是否阻断流水线"排序，**Q1–Q3 必须在 `01-design.md` 阶段裁决**。

**Q1 — `crates/rustcode-review/rules/*.md`（47）到底汉不汉化？**
与用户裁决 #1「全量 168 个纯英文 md」的字面表述存在张力（编排者盘点的 52 个"建议跳过"里含这 47 个，故它们**不在** 168 之内）。

- **A（推荐）不汉化**，理由见 §3.1 B 类：编译期 `include_str!` 载荷、无测试可回归、已有 `--rules-dir` 逃生口。
- B 汉化正文（保留全部技术标识符），并要求交付前人工抽检 ≥5 种语言的真实 review 输出，确认 finding 质量未退化。
- C 只汉化"说明性段落"，保留所有带技术判定词的条目 —— **不推荐**：切分标准主观，无法机械验收。

> 影响：Q1 直接决定 AC-1（清单）与 AC-5(a) 的期望值。

**Q2 — 默认 `0.0.0.0` 覆盖哪几处？（安全边界 + 审批翻转，最高优先级）**

- **A（推荐）只改进程内 webui / `rustcode daemon` 两处**（`cli/src/main.rs:1047` 与 `:1779-1780`），**`daemon/src/main.rs:21` 保持 `127.0.0.1`**。
  理由：用户原话是"webui 默认使用 0.0.0.0"；这两处 `enforce_token=true`，有 token 保护、审批不翻转、零安全回归；而独立 daemon 二进制被 IDE 无参数拉起且无 token，改它会同时触发"无鉴权暴露"+"审批静默 bypass"双杀（§0.1）。IDE 用户需要跨设备时仍可显式 `--host`。
- B 三处全改，并对独立 daemon 路径做补偿：把该路径也改为 token 保护（`webui_tokens: Some(...)`），或把 `client_interactive_permission` 改为不再依赖 `is_loopback_authority`（例如改判"客户端已通过任意鉴权"）。
  **此路径必须连带修改 IDE 扩展的连接方式（写入 token），属跨仓库改动，成本高。**
- C 三处全改且不补偿 —— **禁止**：会导致 §0.1 的严重回归。

> 影响：Q2 决定 AC-19 的判定方向、AC-27/28 是否需要改动 Rust 逻辑。若选 A，AC-27 只需"回归验证现状"；若选 B，AC-28 是硬需求。

**Q3 — "不再打印非回环安全警告"的范围？**

- **A（推荐）只删独立 daemon 启动横幅的 `DaemonWarnNonLoopback`**（`lib.rs:6345-6347`），**保留** `Msg::WebuiLanWarning` / `WebuiNonLoopbackWarning`（`lib.rs:5398-5407`，文案见 `zh_cn.rs:1897-1898`）。
  理由：默认改 `0.0.0.0` 后，`DaemonWarnNonLoopback` 会**每次启动必打印**，变成噪音，正是用户想去掉的；而 `WebuiLanWarning` 承载"公网请用隧道 / 无 TLS"的可操作信息，是 webui 暴露面唯一的风险提示，删掉后默认 `0.0.0.0` 将**完全无任何安全提示**，与 `AGENTS.md:174` `[SECURITY]` 约束冲突。
- B 两处都删 —— 不推荐（同上）。
- C 两处都删，但在 `README.md` / `docs/` 中用中文固定章节承载同等风险说明 —— 可接受，但需额外 AC 约束该章节存在且链接可达。

> 另需同步：删 `DaemonWarnNonLoopback` 后，`lib.rs:6333-6341` 的注释（"Default to loopback-only for security … PR #82 …"）与新的 `0.0.0.0` 默认**直接矛盾**，必须改写（否则文档与代码失一致）。

**Q4 — 汉化后是否需要在显著位置保留英文入口？**
删除 `README.zh-CN.md` 后，非中文读者只剩中文 `README.md`。

- **A（推荐）** `README.md` 顶部保留一行指向英文版的链接，英文版指向**上游仓库 README 或本仓库某个英文 tag 的渲染链接**（必须是可访问的真实地址，不得指向 `example.com` 死链 —— 参见 `AGENTS.md:270` 对死链的既有处理）。
- B 不留英文入口（中文单语）。

> 影响：AC-11 的判定内容。

**Q5 — setup-seeds 6 个 md 的 `description` 字段是否汉化？**
`SKILL.md:3` 的 `description` 是模型/工具据以判断"何时调用该 skill"的路由提示。

- **A（推荐）汉化**：fork 默认中文（`AGENTS.md:103`），中文用户以中文描述触发；且 `.codebuddy/agents/requirements-analyst.md:3` 的 `description` 已是中文，有先例。
- B 保留英文：零行为变化，但技能触发对中文用户不友好。

> 风险等级：低（skills 是用户可改的本地文件）。影响 AC-6。

---

## 9. 交付与回报

- 本文件为**只读分析产物**，未修改任何生产代码 / 被汉化 md / `docs/**` / `AGENTS.md`。
- 状态：`status: approved`、`decision: proceed`。
- 开放问题：**5 条**（Q1–Q5），其中 **Q1 / Q2 / Q3 必须在 `01-design.md` 阶段裁决**；Q2 为安全边界 + 运行时生命周期双敏感项，需架构评审。
- 架构敏感面结论：
  - `touches_persistence` = **false**（§7）
  - `touches_cross_crate_deps` = **false**（§7）
  - `touches_runtime_lifecycle` = **true**（approval 路径翻转，`lib.rs:4671-4674`）
  - **安全边界 = 是**（默认 `0.0.0.0` × 独立 daemon 无 token × 审批 fail-open）→ 强制架构评审
- 复核中新发现的、超出本需求范围但建议独立立项的项：
  - **N4**：`lib.rs:4671-4674` 在"无交互审批方"时对 `ApprovalMode::Build` 置 `dangerously_skip_permissions = true`，属 fail-open，与 `AGENTS.md:154` 的 fail-closed 原则相悖。
  - **N1**：`ServerOpts.startup_mode` 为死字段，`--client` 启动参数无实际效果。
  - **N2**：`Msg::CliWebuiNotBuilt` 缺 `cargo clean -p rustcode-daemon`（本需求 AC-17 顺带修正）。
