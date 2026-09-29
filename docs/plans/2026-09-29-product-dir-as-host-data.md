# 产品目录名是宿主的数据，不是库的常量（方案，未开工）

2026-09-29 记。基线：`release/v5.2.0` @ `e1f138e8d`（初稿基于 `900d14c5a`，其间新增的提交未涉及 `.atomcode`，
只有 `coding/src/runtime.rs` 行号挪了 6 行，已核过）。

## 问题

`distribution::HOME_DIR_NAME` 已经存在，但全仓非测试代码里仍有 40 余处直接写 `.atomcode`：
技能目录、插件路径、memory、setup、团队 worktree、工具输出落盘、敏感路径标记……
下游要改名只能逐处手改；漏一处不报错，只是静默分家。

已经发生过的分家（不是假设）：

- `capabilities/src/skills/render.rs::source_rank`：一个把配置树挪了位置的构建，把**自己**
  装的插件排到了 3 级（低于 `~/.claude`、`~/.agents`），预算一紧就被裁掉，模型当它不存在。
  现在靠「先比 `$ATOMCODE_HOME` 前缀、再兜底 `contains(".atomcode")`」补上。
- `capabilities/src/tools/sensitive_path.rs::configured_credential_markers`：字面量
  `/.atomcode` 覆盖模型会写的 `~/.atomcode/auth.toml` 形式；fork 改名后，`~/.<fork>/auth.toml`
  这种写法只剩「配置目录」那一路保护。
- `memory/store.rs` 早就加了 `ATOMCODE_PROJECT_MEMORY_DIR` 覆盖，测试里写的就是 `.myapp`——
  有人已经为这个需求单独开过一个口子。

## 判据：两类下游都要顺

1. **fork 整个仓库**的下游：改名要动几个文件？目标是 1 个。
2. **只把某个 crate 当依赖导入**的下游（`{ git = ... }`）：不 fork、不 `[patch]`、
   不设环境变量，照常调用 API 就能用自己的目录。

## 真实下游（2026-09-29 实地看过）

两类下游都存在，各一个：

**longcode（`longyuan/atomcode_longyuan`）—— fork 整仓。** 整份替换 `distribution.rs`（`04173a3f0`，12 文件），
再全仓把 `.atomcode`/AtomCode 换成 `.longcode`/LongCode（`b644aa6c4`，**239 文件**）；替换过头把规则文件
`.atomcode.md` 也改了，又撤回（`3391b33e9`，12 文件 55 处）。至今合上游 23 次，**每次合并都重做一遍**：
09-29 那次（`86a5e985f`）合并提交里手工改回的 `.longcode` 有 140 行、二十多个文件，最多的是
`memory/store.rs` 13、`sensitive_path.rs` 11、`config/memory.rs` 9、`session/manager.rs` 6、`team.rs` 5、
`model_source.rs` 5——与下方 A/B 类盘点一一对应。通知标题手改成 `"LongCode done"`（E 类）。
他们的判断与「本方案不动」一致：规则文件名保留 `.atomcode.md`；`ATOMCODE_HOME` 保留为内部变量、
另加 `LONGCODE_HOME` 别名（理由写在提交里：「上游 8 个解析函数都读它，一个都不想改」）。

**longcode-air（`longyuan/atomcode-longyuan-air`）—— 只导入 crate。** Tauri 桌面应用，
`src-tauri/Cargo.toml` 以 git 依赖导入 kernel / coding / capabilities / auth / config。它的用户目录是
`~/.longcode-air`，而库只认 `ATOMCODE_HOME`，所以：

- **每个进程入口都得先 `set_var("ATOMCODE_HOME", …)`**（`src/lib.rs` 的 `sidecar_main`、`mcp_server_main`）。
  `mcp_server_main` 的注释记着漏设的后果：`atomcode_auth::get_stored_auth()` → `Config::config_dir()`
  落到默认目录，读不到 `~/.longcode-air/auth.toml`，**表现为「未登录」，而用户明明登录了**。
  这正是「进程级注入 + 静默兜底」的失败形态：忘了注入不报错，只是悄悄读错地方。
- **测试把我们的问题原样继承过去**：自建 `TEST_ENV_LOCK`，每个用例同时设/清 `ATOMCODE_AIR_HOME` 与
  `ATOMCODE_HOME` 两个变量（`src/lib.rs` 的 `TempHome`、`src/mcp_admin.rs`）。
- **项目目录没得选**：库把 `<project>/.atomcode` 写死，air 就把自己的东西也建在那里
  （`.atomcode/conversations.json`、打包时的排除规则等十余处）——用户级叫 `.longcode-air`、
  项目级叫 `.atomcode`，不是选择，是被迫。
- 注意 **`atomcode-auth` 也在环境里解析目录**（经 `Config::config_dir()`），原盘点漏了它；
  对 air 来说它是最要紧的一处（登录态）。

## 否决过的方案（别再试）

| 方案 | 为什么不行 |
| --- | --- |
| 各处改用 `distribution::HOME_DIR_NAME` | capabilities 里 askpass、pathutil 不挂 feature，tools/skills/memory/cc-hooks 也不带 `atomcode-config`（Cargo.toml 写明精简嵌入方不被 config→telemetry 拖进 reqwest）；telemetry 在 config 之下；tui 刻意不依赖 config（「App apart」）。够不着。 |
| `.cargo/config.toml` `[env]` + `env!()` | **实测**：下游用 git 依赖导入时，我们仓库的 `.cargo/config.toml` 不生效，`env!` 直接 `could not compile`。判据 2 当场不过。 |
| `option_env!` + 默认值 | 能编过，下游也能在自己的 `[env]` 里改（实测改值会自动重编依赖）。但默认值要在每个够不着 config 的 crate 里各写一份；`option_env!` 进不了 `concat!`，文案还得另想办法。是把常量换个地方散，不是收拢。 |
| 新建零依赖叶子 crate 放名字 | 大家都能依赖到了，但库仍然「知道产品叫什么」，下游导入 crate 时照样得 fork 那个叶子。 |

四个方案都在回答「名字放哪」。问题本身问错了。

## 原则

**库收目录，不收名字。** `.atomcode` 只允许出现在宿主侧的 `distribution`；
L1/L2 的库只见到 `Path`。

大部分函数**现在就在收参数**，只是收的是「根」（家目录、项目根），然后自己在里面拼 `.atomcode`。
要改的是参数的含义——从「根」改成「产品目录本身」——不是新加一套传法：

```rust
// 现在：收项目根，自己拼名字
project_plugins_root(working_dir, scope)   // 内部 working_dir.join(".atomcode/plugins")
// 改后：收产品目录，库里没有名字
project_plugins_root(project_dir, scope)   // 内部 project_dir.join("plugins")
// 调用方（宿主）
project_plugins_root(&layout.project(&working_dir), &scope)
```

- 判据 2 自然满足：下游导入 crate 时传自己的目录，跟用任何库一样。
- 分层问题消失：capabilities/telemetry/tui 不需要依赖 config，它们手里本来就不该有名字。
- 测试不再改进程级 `ATOMCODE_HOME`（`capabilities/src/paths.rs` 的注释专门写了「这里故意不写单测，
  因为会跟并行测试抢环境变量」；`plugin/paths.rs`、`plugin/loader.rs`、`plugin/installer.rs`、
  `skills/render.rs` 的测试都在 `set_var`）。

## 形状

```rust
// atomcode-config::distribution —— 全仓唯一写出 ".atomcode" 的地方
pub const HOME_DIR_NAME: &str = ".atomcode";      // 已有
pub const PROJECT_DIR_NAME: &str = HOME_DIR_NAME; // 新增：<project>/.atomcode

/// 宿主启动时算一次，往下传。
pub struct Layout {
    /// 用户级根：$ATOMCODE_HOME，否则 ~/.atomcode（与 bootstrap_home 同一个答案）。
    pub user: PathBuf,
    /// 项目级目录名。
    pub project_dir: &'static str,
}
impl Layout {
    pub fn resolve() -> Self { … }                       // 复用 Config::config_dir
    pub fn project(&self, root: &Path) -> PathBuf { root.join(self.project_dir) }
}
```

签名变化：

```rust
// 现在                                                  // 改后
standard_skill_dirs(home, project)                      standard_skill_dirs(user_dir, project_dir)
runtime_skill_dirs(home, project)                       删除（它只是把 home/.atomcode 剥掉换成 $ATOMCODE_HOME）
runtime_skill_install_dirs(home, project)               skill_install_dirs(user_dir, project_dir)
project_plugins_root(working_dir, scope)                project_plugins_root(project_dir, scope)
MemoryStore::project(project_root)                      MemoryStore::project(project_dir)
setup::lock_dir(project_root)                           调用方传 project_dir
```

## 各 crate 要做的事

| crate | 做什么 |
| --- | --- |
| atomcode-config | `distribution` 加 `PROJECT_DIR_NAME` 与 `Layout`；config 模板注释用 `Layout` 渲染；确认 `config/memory.rs` 那份重复的路径解析还有没有消费者，没有就删 |
| atomcode-capabilities | skills / plugin / memory / setup / session 改收目录；删 `runtime_skill_dirs` 的先拼再剥；`cc_hooks` 自带的根解析并进 `paths::config_dir`；通知标题走 `{brand}`；名单类（C）等决定点 |
| atomcode-harness | team 的 worktree / 角色目录缺省值由宿主填；`atomcode_home()` 用 `Layout`；帮助文本渲染 |
| atomcode-coding | `artifacts_dir` 用常量；调 skills 改传目录；persona 渲染路径 |
| atomcode-cli / atomcode-daemon | 启动时构造一次 `Layout` 往下传；跟着改调用方 |
| atomcode-tui | `image_cache` 的目录改为注入（它刻意不依赖 config） |
| atomcode-i18n | 加 `{user_dir}` / `{project_dir}` 占位（照 `{brand}`），约 20 条文案改用占位 |
| atomcode-telemetry | 删 `default_atomcode_dir`（零调用方） |
| atomcode-tuix | 待删，只把字面量换成常量，不改签名 |
| 不动 | kernel、host-api、updater |

## 盘点（非测试代码）

### A. 已经收路径参数的 API —— 改参数语义（第 1 刀）

| 位置 | 现在 | 改后 |
| --- | --- | --- |
| `capabilities/src/skills/registry.rs:274,298,320` | `home.join(".atomcode/…")`、`project.join(".atomcode/…")`；`runtime_*` 再读 `ATOMCODE_HOME` 剥前缀 | 收 `user_dir` + `project_dir`；两个 `runtime_*` 合并掉。调用方：`coding/parts.rs:718,747`、`coding/runtime.rs:9795`、`harness/plugins/capabilities.rs:138`、`cli/tui_setup.rs:61`、`registry.rs:188` 自身 |
| `capabilities/src/plugin/paths.rs:37-38` | `working_dir.join(".atomcode/plugins")` | 收 `project_dir`。调用方：`loader.rs:297-298`、`installer.rs:572,594,623,740,749,789`、`cli/tui_plugins.rs:296`、`tuix/modals/plugin_manager.rs:386,796` |
| `capabilities/src/memory/store.rs:34,45` 与 **`config/src/config/memory.rs:32,43`** | 默认 `".atomcode"` / `".atomcode/local"` | 收 `project_dir`；`ATOMCODE_PROJECT_MEMORY_DIR` 覆盖保留。**这两份是同一逻辑的两份拷贝**，顺手确认哪份还有消费者 |
| `capabilities/src/setup/lock.rs:58`、`install.rs:34`、`state.rs:11 STATE_DIR` | `project_root.join(".atomcode")` | 收 `project_dir`；`install.rs:11,61-64` 写进 `.gitignore` 的 `.atomcode/local/` 由 `project_dir` 拼出 |
| `capabilities/src/session/manager.rs:1116` | `working_dir.join(".atomcode/local/id")` | 收 `project_dir`（`SessionManager` 构造时带进来） |
| `harness/src/plugins/team.rs:1217` | `worktrees_dir` 行配置缺省时 `repo.join(".atomcode/worktrees")` | 行配置已可注入；缺省值改由宿主在装配时填 |
| `harness/src/plugins/team.rs:1628` | `project.join(".atomcode/agents")` | 同上，`roles_dirs` 已可注入，缺省改由宿主填 |
| `coding/src/on_harness.rs:1581 artifacts_dir` | `working_dir.join(".atomcode/artifacts")` | coding 依赖 config，直接用 `PROJECT_DIR_NAME` |
| `tuix/src/custom_commands.rs:113` | `project_root.join(".atomcode/commands")` | tuix 依赖 config，直接用常量（tuix 待删，不值得改签名） |
| `capabilities/src/datalog.rs:243` | 把字符串 `"~/.atomcode/datalog"` 当「语义默认值」识别 | 默认值字符串由 `config` 的同一个常量拼出，两边比的是同一个值 |

### B. 从环境里解析「用户级根」的函数 —— 字面量只在兜底分支里（第 2 刀，见决定点）

`$ATOMCODE_HOME` 本身已经是宿主注入的运行时数据：`cli/main.rs`、`daemon/main.rs`
在启动第一件事调 `bootstrap_home()` 把它写进环境。这些函数里的 `.atomcode` 只在「变量没设」时走到：

| 位置 | 备注 |
| --- | --- |
| `capabilities/src/paths.rs:17 config_dir()` | capabilities 内的唯一解析点，mcp（`mcp/util.rs`，~10 处）、memory、session、`tools/grep.rs:206`、`tools/sensitive_path.rs:141,299`、`provider/mod.rs:68` 都经它 |
| `capabilities/src/cc_hooks.rs:284` | 自己又写了一份，应并进 `paths::config_dir` |
| `capabilities/src/askpass/server.rs:60` | 刻意用 `$HOME` 而非 `$ATOMCODE_HOME`（`SUN_LEN` 限制），名字仍是产品的 |
| `harness/src/model_source.rs:441 atomcode_home()` | harness 依赖 config，可直接用 `Layout`/常量 |
| `tui/src/image_cache.rs:23` | tui 刻意不依赖 config，只能收注入 |
| `telemetry/src/identity.rs:62 default_atomcode_dir()` | **零调用方**，直接删 |

### C. 「已知 agent 目录」名单 —— 名单里本来就混着别家产品

| 位置 | 用途 |
| --- | --- |
| `capabilities/src/pathutil.rs:14 SKIP_DIRS` | 代码智能/遍历跳过 |
| `capabilities/src/file_index.rs:25 ALWAYS_INDEX_DIRS` | 文件选择器总是索引 |
| `capabilities/src/tools/sensitive_path.rs:137` | 凭据标记（安全相关，见开头） |
| `capabilities/src/skills/render.rs:72 source_rank` | 插件排序兜底 |

这几处挂在工具调用路径上，要么把 `project_dir` 带进 `ToolContext`，要么跟 B 一起走同一个注入点。

### D. 给人/给模型看的文案

- i18n：`product/{en,zh_cn}.rs`、`screen/{en,zh_cn}.rs` 里 `~/.atomcode/plugins`、`.atomcode/commands`、
  `~/.atomcode/config.toml`、`~/.atomcode/mcp.json` 等约 20 条 → 加 `{user_dir}` / `{project_dir}` 占位，
  走 `runtime::substitute_placeholders`（与 `{brand}` 同一条路，`set_brand` 旁加一个 setter）。
  好处：设了 `ATOMCODE_HOME` 的人看到的是自己真实的目录。
- `coding/src/persona.rs:746`（系统提示里说配置在 `~/.atomcode`）、`harness/src/launch.rs:460-488`
  帮助文本、`harness/src/plugins/llm.rs:723`、`harness/src/model_source.rs:360`、
  `config/src/config/mod.rs` 生成的 config 模板注释、`daemon/src/lib.rs` 启动报错提示、
  `cli/src/acp/commands.rs:609` → 生成时用 `Layout` 渲染。persona 渲染结果在一台机器上是稳定的，
  不破坏前缀缓存。
- `tuix` 里的几处（`commands.rs`、`event_loop/mod.rs` 的技能安装提示）：tuix 待删，只改用常量。

### E. 品牌名（不是目录名）

- `capabilities/src/notify.rs:243-247,273,371`：`"AtomCode done"` 等通知标题 → 走已有的
  `{brand}`（`atomcode_config::i18n::substitute_placeholders`，notify feature 本来就带 config），
  跟随 `ui.brand_name`。

### F. 本方案不动

| 项 | 理由 |
| --- | --- |
| `.atomcode.md` / `.atomcode.user.md` / `ATOMCODE.md` 指令文件名（`config/instructions.rs`、`capabilities/instructions.rs`） | 另一份契约（项目里提交的文件名），改它影响已有用户仓库；需要的话单开一刀，同样按「库收名单」处理 |
| `.atomcode-plugin/` 清单目录 | 插件生态格式，与 marketplace 上已发布的仓库兼容 |
| `com.atomcode.schedule.*` launchd label | 与已安装的系统计划任务兼容 |
| `UPDATE_TEMP_PREFIX` | 已在 `distribution` |
| `ATOMCODE_HOME` 等环境变量**名** | 运行时覆盖维度，另议 |

## 决定点：用户级根（B、C）怎么进到够不着 config 的库

A、D、E 不需要做这个决定就能先做。B/C 有三条路：

1. **全程显式传参**：`paths::config_dir()` 删掉，mcp/tools/provider/session 的构造都带 `user_dir`。
   最干净，但 mcp 那 ~10 处是自由函数，tools 要进 `ToolContext`，改动面最大。
2. **进程级注入点**：capabilities 提供 `paths::install(user_dir, project_dir)`，宿主启动时调一次
   （与 `askpass::set_env` 同一种形状）；没装时退回读 `$ATOMCODE_HOME`，再退回 `~/.atomcode`。
   只剩这一个兜底字面量 + 一条判据断言它等于 `distribution::HOME_DIR_NAME`（capabilities 的
   dev-dependency 里本来就有 config）。改动小，代价是一个全局。
3. **1 与 2 分期**：先做 2 把字面量收到一处，再把高频路径（tools、session）逐步改成显式传参。

**定为 1。** 初稿倾向 3，看过 air 之后推翻：进程级注入点就是 air 今天手上那根 `set_var("ATOMCODE_HOME")`
换个名字——漏调一个入口不报错、悄悄读错目录（air 的「未登录」），一个进程只能有一个值，测试照样要锁。
显式传参让「忘了给目录」变成编译错误。范围在初稿之外加上 `atomcode-auth`（登录态存储）与
`atomcode-config` 里被库调用的 `Config::config_dir()`：库这一侧改为收目录，`Config::config_dir()`
只留给宿主。

## 顺序与判据

1. `distribution` 加 `PROJECT_DIR_NAME` 与 `Layout`；宿主（cli、daemon、tui 的 host 侧）各构造一次。
2. A 类逐个改签名，每改一个 API 跑该 crate 的测试；capabilities 的 `session`/`setup`/`plugin`
   挂在非默认 feature 下，合之前必须跑 `bash gates/compile.sh`（AGENTS.md 那张表的第二行就是这么烂的）。
3. D、E。
4. 按决定点做 B、C。
5. telemetry 删 `default_atomcode_dir`。

判据（按 AGENTS.md 要求，先摘掉被测代码证伪一次）：

- **名字只在一处**：一条测试/闸门扫 `crates/*/src/**/*.rs`，非测试、非注释代码里出现 `".atomcode"`
  字面量的只能是 `distribution.rs`（外加决定点 2 的那一个兜底）。白名单写 F 类。
  这条是棘轮：数字只能降不能升。
- **下游改名能用**：把 `Layout` 换成 `.forkcode` 构造一个 coding runtime，断言 skills、plugins、
  memory、session 标记、artifacts 全落在 `.forkcode` 下，`.atomcode` 目录一个都没被创建。
- **行为不变**：默认 `Layout` 下既有测试不改期望值（只把期望里的 `".atomcode"` 换成从常量拼）。
- **通知跟品牌走**：`set_brand("Forkcode", …)` 后通知标题为 `"Forkcode done"`。
