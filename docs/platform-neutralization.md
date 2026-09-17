# RustCode 平台中立化与仓库卫生报告

> 本文档汇总 fork 在「平台中立 / 零遥测 / 默认中文 / 发行链路去厂商化 / 输出 ASCII
> 化」几条线上已落地的功能与工程卫生工作,作为 `AGENTS.md`「已知剩余项」中大批
> `[DONE]` 条目的正式文档。面向使用者的能力总览见 `docs/features.md`;分层架构见
> `docs/architecture.md`;面向 Agent 的硬性约束见根目录 `AGENTS.md`。
>
> 文档与代码冲突时以代码为准。状态标签统一使用 ASCII:`[DONE]` 已完成、
> `[WARN]` 需注意、`[CHECK]` 已验证门禁、`[+]/[-]` 表格是非。

## 1. 背景与原则

RustCode 是上游项目的二次开发 fork。中立化的目标是:
**构建产物与运行时不把用户导向任何签名网关、托管端点或厂商主机**,发行主机一律
由环境变量 / 运营方在构建或部署时注入。

落地时遵循四条原则:

1. **运行时无厂商默认** — 生产路径不硬编码默认 model / provider / base_url;无
   配置时 headless 直接失败并给出可操作提示,而不是"真空成功"后在网络层报错。
2. **第三方 BYO 配置保留** — `deepseek` / `openai` / `anthropic` / `qwen` /
   `ollama` 等自带密钥(Bring Your Own Key)预设是用户主动选择的第三方接入,
   不属于平台绑定,予以保留;平台绑定特指签名网关 / 托管端点。
3. **发行链路去厂商默认** — 安装器、release 上传、CI、打包脚本里不写死任何厂商
   主机 / 账号 / IP,全部改为 env 注入,缺失即 fail-closed。
4. **输出 ASCII 化** — 日志、注释、状态、文档严禁 Unicode 图形 emoji,统一使用
   ASCII 标签;TUI 排版字形按终端能力走 `glyph.rs` 政策(见第 6 节)。

## 2. 运行时与 Provider 中立

- **[DONE] headless 无 provider 预检**:无 provider 或 `--provider <不存在>` 时,
  原先 `ProviderBootstrap::Required` 会以空 `base_url` 的 OpenAI 适配器"真空成功",
  随后在 reqwest 处报含义不清的 `relative URL without a base`。现在 CLI headless
  路径在解析后若 `runtime_cfg.model.is_empty()` 即 bail,指名请求的 provider、指向
  config 路径、提示无参启动做交互配置。
  - 代码:`headless_missing_provider_message()`(`crates/rustcode-cli/src/main.rs:2789`)
  - 测试:`headless_missing_provider_message_fires_only_without_resolved_model`
    (`crates/rustcode-cli/src/main.rs:5081`)
- **[SUPERSEDED 2026-09-09] 签名网关识别整体删除**:识别器 `is_codingplan_llm_gateway`
  与其唯一开关 `RUSTCODE_CODINGPLAN_LLM_BASE_URL` 已随 codingplan 一并删除 —— 现在没有任何
  base_url 会被判为签名网关,所有 provider 一律走纯 `bearer_auth(api_key)`,该环境变量设置后
  被直接忽略、不报错。原装配说明所在的
  `crates/rustcode-codingplan/src/setup.rs:52` 连同整个 crate 已不存在,此路径仅作历史指引。
- **[DONE] 预设顺序中立**:厂商预设原先把某商业 token 转售商放在索引 0,TUI 的
  `preset_idx_by_id` 防御性 fallback(`unwrap_or(0)`)也会落到它。现把两个中立的
  `*-compatible` 自带端点预设移到注册表最前(索引 0 = `OPENAI_COMPATIBLE`),厂商
  预设随后;未知 id 由 `preset_or_compatible` 回落到 `OPENAI_COMPATIBLE`。
  - 注册表:`crates/rustcode-config/src/config/provider_preset.rs:98`
  - 锁定测试:`generic_endpoints_lead_the_registry`
    (`crates/rustcode-config/src/config/provider_preset.rs:303`)
  - TUI 回退:`crates/rustcode-tuix/src/modals/provider_panel.rs:71`
- **[DONE] atomgit REST 工具门控**:`atomgit_repo` / `pr` / `issue` 等平台 REST 工具
  与 `api.atomgit.com/api/v5` 装配、push-label 中间件整体由 `atomgit` Cargo feature
  `#[cfg(feature = "atomgit")]` 门控,默认成员不启用。这是刻意保留的上游开关,不是
  残留。
- **[DONE] OAuth 与登录中立化**:loopback 回调的错误分支不再 302 跳转到厂商站点,
  改为与成功分支同构的本地中性 HTML 错误页;`/login` 已移除，改为提示用户直接配置 provider
  时提示用户直接配置 provider;`friendly_http_error` 的 403 提示去掉 `/login` 引导,
  改为"检查 API key 权限与账户状态"。
- **[DONE] 夹具与示例中立化**:测试夹具 URL 统一改 `example.com` / `127.0.0.1` 桩;
  `crates/rustcode-coding/examples/run_task.rs` 去掉 DeepSeek 默认端点,
  `RUSTCODE_BASE_URL` / `RUSTCODE_MODEL` 改为必填(未设即退出并提示)。

## 3. 发行 / 安装 / 打包 / CI 去厂商化

原则:发行主机一律由 env / 运营方注入,构建里不带厂商默认。

- **[DONE] 安装器** `scripts/install.sh`、`scripts/install.ps1`:
  - 删除硬编码的 `gitcode.com/SecLab/RustCode` 仓库地址与 `v5.0.2` 兜底 tag。
  - 改用 `RUSTCODE_RELEASE_BASE`(下载根,**必填**,未设即 fail-closed 并打印可操作
    指引)、`RUSTCODE_RELEASE_LATEST_API`(可选,JSON 的 `tag_name` 用于 latest 自动
    探测)、`RUSTCODE_VERSION`。
  - 删除 referral / invite 代码块(写 `~/.rustcode/pending_invite` + install_uuid):
    遥测 crate 删除后已无任何 Rust 读取方,属平台增长残留。
  - `install.ps1` 失实注释 `crates/rustcode-core/...` 修正为 `rustcode-cli`;
    `uninstall.sh` / `uninstall.ps1` 头部 `curl|sh` / `irm` 厂商 URL 改本地 / 发行
    渠道口径。
- **[DONE] release 上传** `.github/workflows/create_tag_release.py`:删除硬编码的
  `API_HOST=https://api.gitcode.com` 与 owner `bangxu`;改为 `--api-host` / `--owner`
  或环境变量 `RUSTCODE_RELEASE_API_HOST` / `RUSTCODE_RELEASE_OWNER` /
  `RUSTCODE_RELEASE_REPO` / `RUSTCODE_RELEASE_ACCESS_TOKEN`(GitLab-v5 兼容
  releases / `upload_url` 方言),缺失即列出缺什么并退出。
- **[DONE] GitCode 发布脚本** `scripts/gitcode_release.py`:新增,经 GitCode 官方
  OpenAPI `POST /api/v5/repos/{owner}/{repo}/releases` 创建 release 并按 GitLab-v5
  `releases/{tag}/upload_url` 方言上传附件;支持 `--dry-run` / `--attach` /
  `--file-name`;主机 / owner / repo / token 一律由 env `RUSTCODE_RELEASE_API_HOST` /
  `RUSTCODE_RELEASE_OWNER` / `RUSTCODE_RELEASE_REPO` / `RUSTCODE_RELEASE_ACCESS_TOKEN`
  或 CLI flags 注入,脚本不带任何厂商默认;unittest 见 `scripts/test_gitcode_release.py`。
- **[DONE] Gitee 流水线** `.gitee/workflow/pipelines/build-and-release.yml`:新增
  `shell@1` 步骤,在 `build@rust` 产物后从 `Cargo.toml` 派生版本号 → tag `v{version}`,
  按 `uname -m` 命名 `rustcode-{tag}-linux-{x64|arm64}` 并调用 `gitcode_release.py`
  上传;凭据 / 主机全部依赖流水线环境变量,不在 YAML 硬编码。
- **[DONE] CI** `.github/workflows/build.yml`:删除 4 处 "Add hosts" 步骤(macOS /
  linux / windows / distro-pm-check)——它们用 sudo 把 `api.gitcode.com` /
  `file.gitcode.com` 钉到厂商 IP `159.138.147.37`,属厂商 DNS 绕行;发行主机改为
  运营方注入后这些步骤无意义。上传步骤改传 `secrets.RELEASE_API_HOST` /
  `RELEASE_OWNER` / `RELEASE_ACCESS_TOKEN`。
- **[DONE] 打包脚本** `packages/npm/scripts/build_npm_package.sh`、
  `packages/homebrew/scripts/package-tar-gz.sh`:GitCode v5 API 地址 / 账号改 env
  注入,不在脚本内写死厂商主机。
- **[DONE] 站点** `site/`:install 改"发行渠道获取 + `RUSTCODE_RELEASE_BASE=...`"、
  clone 改 `example.com`、npm scope 统一 `@rustcode/rustcode`、marketplace 改自有
  渠道、CDN 截图改内联 SVG / data-URI、`referral.html` 替为中性占位页,
  `search-index.{zh,en}.json` 经 `node build-search-index.mjs` 重生成。站内剩余
  `*.rustcode.dev` 导航链接是本项目自有重命名域名(品牌),保留。

## 4. Docker / 镜像中立

- **[DONE] 基础镜像**:`docker/Dockerfile-Daemon-Tosslib` 从内网
  `swr.cn-north-4.myhuaweicloud.com/...` 改为公共 `debian:bookworm-slim`。
- **[DONE] APT 源**:三个 Dockerfile 硬编码的 aliyun APT 源改为默认空、按需
  `--build-arg APT_MIRROR=<url>` 注入(默认走 Debian 官方源),
  见 `docker/Dockerfile-Daemon-Tosslib:6`。
- **[DONE] 编排与文档**:`docker-compose.yml` / `docker/README.md` 内网镜像仓库地址
  改 `your-registry.example.com/...` 占位(Docker Hub / GHCR / Harbor / 私有库均
  适用);"GitCode App" 改中性"移动端 agent 应用";`⚠️` 改 `[WARN]`。
- **[DONE] 示例配置**:`docker/config-example.toml` 默认 provider 从 openrouter 改为
  `my-provider` + `api.example.com` 模板。

## 5. 凭据与开发文档中立

- **[DONE] 疑似真实凭据清除**:`docs/dev-env-setup.md`、`DEVENV.md` 原硬编码了疑似
  真实的 `CODEARTS_CLI_AK` / `CODEARTS_CLI_SK`,已全仓清除(grep 0 命中)。
  `.gitignore` 中 `.codeartsdoer/` / `codearts.zip` 是防凭据入库的保护性忽略规则,
  保留。
- **[DONE] 开发环境文档平台中立化**:`docs/dev-env-setup.md`、`DEVENV.md` 从
  Huawei Cloud EulerOS + CodeArts 专用笔记重写为平台中立(通用 Linux/macOS),删除
  CodeArts / arkcli 等厂商云 CLI 步骤;tokenhub / hwdevspace / `openpangu` 网关示例
  改 `gateway.example.com` + `${MY_PROVIDER_API_KEY}` 环境变量注入;修正
  `cargo run -p rustcode-cli`(包名实为 `rustcode`)为 `cargo run` / `-p rustcode`。
  `docs/codex-claude-config-analysis.md` 的 tokenhub 端点一并中立化。
- **[DONE] 扩展 / 前端 / 人设夹具**:`extensions/vscode` 的 `/config` 帮助片段、
  `webui` 模型输入框 placeholder、`extensions/jetbrains` SSE 测试夹具 URL 从
  deepseek 改为 `your-model-id` / `api.example.com`;i18n 的 Base-URL 引导提示改
  `api.example.com`。
- **[CHECK] 运行时无厂商默认**:全仓排查确认生产代码路径没有硬编码默认
  model / provider / base_url 会把用户导向某厂商;无配置时 headless 预检直接 bail
  (第 2 节),预设全为 opt-in 手选。测试夹具里的厂商模型名均在 `#[test]` 内且端点为
  `example.test` / `127.0.0.1` 桩,非默认值、不影响行为。

## 6. 全仓输出 ASCII 化与 TUI 字形政策

- **原则**:严禁 Unicode 图形 emoji;状态 / 日志 / 注释 / 输出一律 ASCII 标签
  (`[INFO]` / `[WARN]` / `[ERROR]` / `[SUCCESS]` / `[CHECK]` / `[+]` / `[-]` /
  `[*]`)。
- **[DONE] 文档**:`docs/` 下约 49 个 md 的装饰性 / 状态 emoji 转 ASCII(表格
  `✅/❌ -> [+]/[-]`、完成清单 `✅ -> [x]`、banner `⚠️ -> [!]/[WARN]`、装饰
  `✨/🎉 -> [*]`),剥掉残留 U+FE0F;fenced 代码块与 TUI 字形设计稿受保护,表格
  管道数逐一核对不变。
- **[DONE] 扩展 / 前端**:vscode / jetbrains 扩展与 webui 共 12 文件的图形 emoji
  删除或改 ASCII 标签(`[#]/[X]/[EDIT]/[F]/[D]/[=]/[*]/[KEY]/[~]/[!]`),U+FE0F
  残基清零;JetBrains 齿轮菜单生产标签在 `RustCodeBundle*.properties` 补齐
  `gear.settings`,测试 `GearMenuLabelsTest.kt` 锁定实际标签。
- **[DONE] 顶层 / CI**:`README.md` 与原中文 README 的贡献 bullets 与捐赠行、
  `extensions/vscode/README.md` 能力 bullets、`.github/workflows/build.yml` 步骤标记
  `🔟 -> [10]` 均 ASCII 化;捐赠行顺带去掉"Coding Plan 免费"这类托管服务口径。
- **TUI 排版字形政策(不是 emoji,保留)**:平台中立化重构时 TUI 渲染串曾被机械
  ASCII 化,把 Unicode 排版字形(分隔符 `·`、省略号 `…`、输入尖括号 `❯`、菜单
  `▸`、工具圆点 `●`、spinner `◐/⠋`、状态 `⚠/ⓘ/⏸`、树形 `└`、箭头 `→/↑/↳`、
  勾叉 `✓/✗`)误改成 ASCII 占位,导致上游断言 Unicode 的红测试。修复原则遵循
  `glyph.rs`:**渲染器在 `unicode_symbols` 分支恢复 Unicode**,dumb terminal /
  legacy conhost / `LANG=C` 仍由 `glyph::downgrade_glyphs` / `ascii_for` 降级为
  ASCII(`crates/rustcode-tuix/src/glyph.rs:77` / `:21`)。`·` U+00B7 刻意不在
  downgrade 表(几乎所有终端可渲染,无条件透传)。刻意保留 ASCII 的只有替换表情
  符号的方括号标签与 config 层 tagged 模板。
- **有界例外(保留)**:
  - TUI 彩色状态点**设计稿**(`docs/superpowers/{plans,specs}/2026-07-03-terminal-
    status-glyph*.md` 等 3 个文件)里的 `🟢/🟡/🔴` 是该彩色状态点特性的规格主语义
    (实际渲染为 `●` + ANSI 色),ASCII `[+]/[-]` 无法表达颜色,故保留。
  - 排版类标记 `✓/✗/⚠(无 FE0F)/→/·/●/❯/☐/☑` 在 unicode 能力端与 TUI 字形政策
    一致,保留。
  - `tab-title-truncation.test.ts` 的 `🎉` 是被测输入。
  - `AGENTS.md` 中 backtick 包裹的字形名是字形政策文档引用。

## 7. 测试与稳定性收敛

- **[DONE] `rustcode-capabilities` lib 存量红测试**:
  - `cc_hooks::tests::turn_complete_payload_alignment`
    (`crates/rustcode-capabilities/src/cc_hooks.rs:1144`):测试自身缺陷——同一 `&&`
    链中两个 `grep` 共享 stdin,首个 grep 抽干管道致第二个必失败;改为先
    `cat > payload.json` 再对文件断言。
  - `tools::read::tests::large_{non_code,symbolless_code}_..._bounded_page`
    (`crates/rustcode-capabilities/src/tools/read.rs:1276`):断言停留在旧 300 行分页,
    实现已为 1500 行页(`DEFAULT_READ_LIMIT`,`crates/rustcode-capabilities/src/
    tools/read.rs:25`);夹具改为 1600 行并对齐
    `Continue with read_file({"limit":1500,"offset":1501})` 格式。
  - `subagent/claude_code`、`subagent/codex` 在高并发下偶发
    `SpawnFailed("Text file busy (os error 26)")`(overlayfs / 容器内新写脚本
    execve 的 ETXTBSY 竞态;Go fork/exec 内置重试而 Rust std 没有):新增
    `process_utils::spawn_retrying_etxtbsy`(8 次退避重试,
    `crates/rustcode-capabilities/src/process_utils.rs:204`),接入
    `subagent::proc::ManagedChild::spawn`(`.../subagent/proc.rs:146`)与
    `cc_hooks::run_command_hook`(`.../cc_hooks.rs:324`)。
- **[DONE] `rustcode-tuix` lib 存量红测试**:从 2008 passed / 65 failed 收敛到
  **2053 passed / 0 failed**;根因即第 6 节的机械 ASCII 化误伤。测试夹具是
  `en_US.UTF-8 xterm-256color`(推得 `unicode_symbols=true`),故断言对齐到 Unicode,
  **测试未被削弱**;断言英文串的测试均显式 `i18n::test_lock()` +
  `set_locale(Locale::En)`,与默认中文改动无关。期间修复
  `event_loop/mod.rs` 测试模块机械重命名残留
  `atomgit_configcodingplan_config("model-b")`(E0425)为 `codingplan_config("model-b")`。
- **[DONE] `rustcode-review`**:100 passed / 0 failed;`review_tool.rs` 的 `·` 分隔符
  随 TUI 字形政策一并恢复。

## 8. 验证门禁

质量门禁 G1-G8 定义见根目录 `AGENTS.md`:

```text
G1  cargo fmt --check
G2  cargo clippy --workspace --all-targets
G3  cargo test --workspace
G4  ./scripts/test-headless.sh                     (需先 cargo build)
G5  python3 scripts/acp_smoke.py
G6  遥测 SDK grep(sentry/posthog/segment/...) 必须 0 命中
G7  crates/ scripts/ .github/ 无 atomcode 残留(atomgit feature / 旧前缀兼容 /
    fork 发行主页三类除外)
G8  docs/architecture.md 无 atomcode 残留
```

- **[DONE] CI 已补齐**:`.github/workflows/ci.yml` 在 push / PR 到 `main` / `dev` 时
  触发 G1(`cargo fmt --check`)、G2(`cargo clippy --workspace --all-targets`)、
  G3(`cargo test --workspace`)。G2 暂不 `-D warnings`(存量约 420 条 warning,见
  第 9 节),待收敛后收紧;`build.yml` 仍只管 release 构建。
- **测试隔离**:`coding` / `tuix` / `daemon` / `capabilities` / `cli` 入口文件顶部有
  `#[ctor]` 把 `RUSTCODE_HOME` 重定向到临时目录;新增测试不得依赖真实
  `~/.rustcode`,重命名这批目录 / 变量时须同步修改。

## 9. 仍保留的中立面与已知剩余项

- **[CHECK] `atomgit` Cargo feature** 是刻意保留的上游开关(平台 REST 工具、
  `api.atomgit.com/api/v5` 装配、push-label 中间件),默认成员不启用,不是残留。
- **[DONE] 旧 `AtomGit-*` provider 前缀兼容**(`is_codingplan_provider_name` /
  `LEGACY_CODINGPLAN_PREFIX`):此前的保留结论已被推翻 —— 两个符号已于 2026-09-09
  随 codingplan 一并移除,旧配置键不再享有前缀识别与折叠保护。
- **[CHECK] 自有品牌域名** `*.rustcode.dev` 是本项目重命名后的自有域名(品牌),
  非厂商主机,保留。
- **[WARN] 已知红测试** `mcp::registry::tests::trust_key_golden_matches_core_algorithm`
  (`crates/rustcode-capabilities/src/mcp/registry.rs:1505`):`project_trust_key` 用
  `std::collections::hash_map::DefaultHasher`,输出不保证跨工具链稳定;不要随手改
  测试去凑绿。
- **[WARN] clippy 存量**:`cargo clippy` 仍有 per-crate warnings(未用变量、命名),
  非 errors;可择机 `cargo clippy --fix` 收敛后再把 G2 收紧为 `-D warnings`。

## 事实索引(代码锚点)

| 功能 | 位置 |
|------|------|
| headless 无 provider 预检 | `crates/rustcode-cli/src/main.rs:2789` |
| 签名网关 env 识别(已于 2026-09-09 移除,以下路径与 crate 均已不存在,仅作历史指引) | `crates/rustcode-codingplan/src/setup.rs:52` |
| provider 预设注册表 / 中立顺序 | `crates/rustcode-config/src/config/provider_preset.rs:98` |
| TUI 预设 fallback | `crates/rustcode-tuix/src/modals/provider_panel.rs:71` |
| TUI 字形降级 | `crates/rustcode-tuix/src/glyph.rs:77` |
| read 分页上限(1500) | `crates/rustcode-capabilities/src/tools/read.rs:25` |
| ETXTBSY 重试 spawn | `crates/rustcode-capabilities/src/process_utils.rs:204` |
| Docker APT 镜像注入 | `docker/Dockerfile-Daemon-Tosslib:6` |
| release 上传 env 装配 | `.github/workflows/create_tag_release.py:14` / `scripts/gitcode_release.py:35` |
| 安装器发行根 env | `scripts/install.sh` / `scripts/install.ps1`(`RUSTCODE_RELEASE_BASE`) |
