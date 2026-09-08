# OMO 技能导入 RustCode 的定级与改造方案

- 日期：2026-09-09
- 来源：`oh-my-openagent@4.19.4`，解包于 `/tmp/omo/package/packages/shared-skills/skills/`（17 个技能）
- 许可证事实：`/tmp/omo/package/package.json` 的 `"license": "SUL-1.0"`（Sustainable Use License，非 OSI 开源、不可再许可/不可转让），与 RustCode 的 MIT 不兼容
- 用户裁决：**不复制任何原文**，只借鉴方法骨架（阶段划分 / 门禁 / 清单 / 触发条件写法），用 RustCode 自己的语言、术语、工具重写
- 本批落地：6 个技能安装到 `/root/.rustcode/skills/`（`standard_skill_dirs` 原生加载点，见 `crates/rustcode-capabilities/src/skills/registry.rs:238`）

## 一、定级总览

| # | OMO 技能 | 体积 | 定级 | RustCode 侧对应 |
| --- | --- | --- | --- | --- |
| 1 | ast-grep | 156K | 采纳并重写 | `rustcode-ast-grep` |
| 2 | debugging | 204K | 采纳并重写 | `rustcode-debugging` |
| 3 | refactor | 28K | 采纳并重写 | `rustcode-refactor` |
| 4 | git-master | 12K | 采纳并重写 | `rustcode-git-master` |
| 5 | remove-ai-slops | 24K | 采纳并重写 | `rustcode-remove-ai-slops` |
| 6 | review-work | 32K | 采纳并重写 | `rustcode-parallel-review` |
| 7 | lsp-setup | 116K | 需改造后可行 | 见 §3.1 |
| 8 | programming | 848K | 需改造后可行（仅 rust 子集） | 见 §3.2 |
| 9 | data-scientist | 48K | 需改造后可行（低优先级） | 见 §3.3 |
| 10 | coding-agent-sessions | 148K | 不适配/放弃 | 见 §4.1 |
| 11 | init-deep | 12K | 不适配/放弃 | 见 §4.2 |
| 12 | ulw-plan | 76K | 不适配/放弃 | 见 §4.3 |
| 13 | ulw-research | 52K | 不适配/放弃 | 见 §4.3 |
| 14 | start-work | 20K | 不适配/放弃 | 见 §4.3 |
| 15 | frontend | 2.9M | 不适配/放弃 | 见 §4.4 |
| 16 | visual-qa | 116K | 不适配/放弃 | 见 §4.5 |
| 17 | ultimate-browsing | 296K | 不适配/放弃 | 见 §4.6 |

合计：采纳并重写 6、需改造后可行 3、不适配/放弃 8。

## 二、采纳并重写的 6 个（本批已落地）

统一做法：只保留方法骨架（阶段划分、先验证后修改的顺序、门禁判据、检查清单），
正文按 RustCode 工具集与 `AGENTS.md` 约束重写；每个 `SKILL.md` 末尾有「来源与适配」小节。

### 2.1 ast-grep -> `rustcode-ast-grep`

- 借鉴：先定形状再搜索、抽样核对后才改写、改写后回到结构确认与编译/测试门禁、故障排查清单。
- 改造点（**关键差异**）：OMO 版依赖 `scripts/ast_grep_helper.py` 提供 `search`/`replace`/`scan`/`validate`；
  RustCode 的 `ast_grep` 工具（`crates/rustcode-capabilities/src/tools/ast_grep.rs`）**只执行搜索，没有 rewrite 参数**。
  因此改写路径改为：`search_replace`（纯文本跨文件）/ `edit_file`（单文件）/ `parallel_edit_files`（多文件，带 `contract`）
  / `bash` 调 `sg`（主机装了 `ast-grep` 时，先 dry-run）。
- 补写：`sg` 未安装时的降级路径（`grep` 定位 + `search_replace` 改写，并在报告中声明降级）。

### 2.2 debugging -> `rustcode-debugging`

- 借鉴：假设驱动循环、先复现先取证后修改、一次只改一个变量、失败测试锁定、两轮失败换正交角度。
- 改造点：OMO 版依赖 `lsp_diagnostics` 与 `references/` 里按阶段拆分的外部文档。
  RustCode 事实核对结果：
  - `diagnostics` 工具在 `crates/rustcode-capabilities/src/codeintel/diagnostics.rs:40` 有定义，
    但**全仓没有任何注册点**，模型拿不到；
  - `lsp` 工具受配置开关控制——`codeintel/mod.rs:129` 的 `register_lsp_tool` 要求 `lsp.enabled = true` 才注册，
    默认不保证可用。
  因此全部改为 `bash` + `cargo check` / `cargo clippy` / `cargo test` + `RUST_BACKTRACE` + 日志取证；
  并行取证走 `task` 的 `explore` 只读车道。

### 2.3 refactor -> `rustcode-refactor`

- 借鉴：意图闸门 -> 并行探索 -> codemap -> 测试评估 -> 计划 -> 执行 -> 验证的顺序，以及"先补测试再改"。
- 改造点：去掉 plan agent 与 LSP 精确分析环节；探索用 `task` 的 `explore` 车道，
  分析用 `grep` + `ast_grep`；改写用 `search_replace` / `edit_file` / `parallel_edit_files`。
- 补写：跨 crate 改动前核对 `AGENTS.md` 的依赖方向（`kernel` <- `capabilities` <- `coding` <- `tuix` <- `cli`）；
  以及"测试变红时怎么判"——已有用例红=改坏了回退，红灯暴露旧缺陷=另开一次改动，不许在重构里顺手修。

### 2.4 git-master -> `rustcode-git-master`

- 借鉴：模式闸门（提交 / 历史）、原子分组、提交信息规范、历史调查命令集、禁止事项清单。
- 改造点：RustCode **没有 git 工具**，所有 git 操作一律经 `bash`。
- 补写（本仓硬性）：提交前必须用 `request_user_input` 展示文件清单与提交信息全文并取得确认；
  禁止 `git push`、`--force`、`--force-with-lease`、`--amend`（未明确要求）、`--no-verify`、
  `git reset --hard` / `git clean -fd` / `git checkout -- .`、`git commit -a` / `git add -A`；
  禁止 `sudo` 运行 git（对齐 `AGENTS.md`）。

### 2.5 remove-ai-slops -> `rustcode-remove-ai-slops`

- 借鉴：先锁定行为再清理、按类别分批处理、并行执行后过质量门禁、收尾复核的顺序。
- 改造点：类别定义改为本仓口径，直接对齐 `AGENTS.md` 硬约束——
  禁 Unicode Emoji（一律 ASCII 标签 `[INFO]`/`[CHECK]`/`[ERROR]`/`[+]`/`[-]`）、
  `thiserror` 强类型错误 + `anyhow` 顶层包装、禁裸 `unwrap()`/`expect()` 导致 panic。
- 补写：并行改走 `task` 的 `worker` 车道，且每个 worker 必须声明非重叠 `scope`；
  "先问这段代码该不该存在，再问写得好不好"（删除优先于美化）。

### 2.6 review-work -> `rustcode-parallel-review`

- 借鉴：先收集审查上下文，再按维度并行派发多个子代理，最后汇总去重并给 PASSED/FAILED 判定。
- 改造点（**最大的一处冲突，必须显式处理**）：RustCode 已有内置审查能力——
  - `code_review` 子 agent 工具，装配见 `crates/rustcode-coding/src/parts.rs:608`；
  - TUI `/review` 斜杠命令，见 `crates/rustcode-tuix/src/event_loop/commands.rs:1785`，`/review <base>` 表示 `<base>..HEAD`。
  
  因此本技能定位为**补充而非替代**，在正文首段用分工表写清三者关系，
  并规定：内置审查没跑就先跑内置，本技能结论叠加在其上；子代理一律 `explore`（只读），不派 `worker`。
- 另外：OMO 的 5 个 agent 绑定 `Oracle` / `unspecified-high` 角色档位，RustCode 无此体系，
  改为按维度（正确性与契约 / 安全与边界 / 并发与性能 / 测试与验证覆盖 / 文档与命名一致性）切分。

## 三、需改造后可行（3 个）

### 3.1 lsp-setup（116K）

- 现状：按扩展名路由到 `references/<language>/README.md`，给 macOS/Linux/Windows 安装命令、
  `.codex/lsp-client.json` 与 `.opencode/lsp.json` 两份配置片段，附 `detect-lsp.ts` / `verify-lsp.ts`。
  实测引用：`.codex/` 47 处、`opencode` 25 处、`.omo/` 3 处。
- 可行依据（与「rustcode 无 LSP」的直觉不同，已核实）：
  - rustcode-coding 默认就带 `lsp` feature（`crates/rustcode-coding/Cargo.toml:20`）；
  - 配置面已存在：`LspSettings` / `LspServerSetting`（`crates/rustcode-coding/src/config.rs:140`）、
    `crates/rustcode-config/src/lsp_registry.rs`；
  - 注册点：`codeintel/mod.rs:129`，`lsp.enabled = true` 时挂载 `lsp` 工具。
- 要改什么：
  1. 删掉 `.codex/lsp-client.json` 与 `.opencode/lsp.json` 两套 opencode/Codex 配置写法，
     改为 RustCode 自己的 LSP 配置路径（逐项核对 `LspSettings` 字段后落笔，不臆造键名）；
  2. TypeScript 脚本（`detect-lsp.ts` / `verify-lsp.ts`）要么不引入，要么用 `bash` 等价命令替代；
  3. 语言覆盖面从 20 种收敛到本仓实际用到的（Rust 为主，其余按需）。
- 成本估计：M（约 0.5-1 天），其中配置键名核对是主要工作量。
- 阻塞项：需先确认 RustCode 的 LSP 配置在 `config.toml` 里的确切段落与键名——**未核实前不得落笔**。

### 3.2 programming（848K）

- 现状：397 行 SKILL.md + `references/{python,rust,typescript,rust-ub,go}/` + `scripts/{python,rust,typescript,go}/`。
  规范主张：严格类型、现代工具链（cargo + clippy + miri 等）、parse-don't-validate、穷尽 match、
  禁 `any`/`unwrap`/`panic`、**250 行 LOC 上限**、TDD、consumer-routed logging。
- 冲突点：**规范可能与 `AGENTS.md` 不一致**（如 250 LOC 上限本仓并未规定；`AGENTS.md` 允许测试与明确不变量处的
  `unwrap`/`expect`；日志与错误规范有本仓自己的口径）。照搬会制造第二套互相打架的规范。
- 要改什么：
  1. 只摘取 **rust 子集**（其余 4 种语言与本仓无关，直接弃）；
  2. 逐条与 `AGENTS.md` 的「编码约束（fork 硬性）」比对，冲突项一律以 `AGENTS.md` 为准并在技能里写明"以 AGENTS.md 为准"；
  3. 脚本目录（`scripts/rust/`）不引入，除非确认其内容与 `Cargo.toml` 现状匹配。
- 成本估计：M-L（约 1-2 天，主要是冲突比对）。
- 结论：价值中等（Rust 规范子集），但优先级低于本批 6 个，建议单独立项。

### 3.3 data-scientist（48K）

- 现状：DuckDB/Polars 智能选型、一律经 `uv`、必须带 numpy、明确不用 pandas。
- 冲突点：`uv` / numpy / DuckDB / Polars 都不在 RustCode 工具集内，只能经 `bash` 现装现用；
  "必须用 numpy、绝不用 pandas"这类强约束与本仓无关。
- 要改什么：改成一个薄壳——「用 `bash` 跑 `duckdb` / `polars` 命令行做本地数据探查」，
  去掉依赖注入的硬性规定与项目脚手架约定，并声明前置工具缺失时的降级。
- 成本估计：S（约 2 小时）。
- 结论：可行性没问题，但**对本仓库（Rust CLI/TUI/daemon）几乎没有使用场景**，优先级最低。

## 四、不适配 / 放弃（8 个）

### 4.1 coding-agent-sessions（148K）

- 根因：正文围绕 **opencode 的 message/part 存储**与 **Codex 的 rollout JSONL / state SQLite** 展开
  （实测 `opencode` 57 处、`codex` 27 处、`session_*` 25 处），还覆盖十来种第三方 CLI 的私有目录布局。
  这些格式在 RustCode 全不存在，照搬即全错。
- 能力已被原生覆盖：RustCode 有原生 SessionManager 与 `/resume`、`/bg` 斜杠命令
  （`crates/rustcode-tuix/src/event_loop/mod.rs:15242`、`:17584`），会话数据在 `~/.rustcode/sessions/`。
- 替代：需要查自己的历史会话时，直接用 `/resume` + `bash` 查 `~/.rustcode/sessions/`，不需要技能。
- 放弃。

### 4.2 init-deep（12K）

- 根因：产物是**分层生成的 AGENTS.md 知识库**（实测 `AGENTS.md` 22 处引用），与本仓
  `AGENTS.md` 的维护规则直接冲突——该文件开头即写明「文件内容变更时同步更新，不得滞后」，
  且是面向 Agent 的长期约束文档，不允许由技能批量生成/覆写；生成层级 `AGENTS.md` 后
  RustCode 的 skill registry 也不会消费它们。
- 放弃。若将来确有需求，改造方向只能是「生成 `docs/` 下的模块说明，不碰 `AGENTS.md`」。

### 4.3 ulw-plan（76K）/ ulw-research（52K）/ start-work（20K）

- 根因：三个技能绑定 OMO 私有工作流概念——`Prometheus`（规划顾问）、`Boulder`（状态机）、
  `Metis` / `Momus`（plan-gated reviewer）、`Sisyphus`、`.omo/plans/` 目录与 Stop-hook 续跑机制。
  实测：`.omo/` 33+12+1 处、`Prometheus` 17 处、`Boulder` 9 处、`Metis` 9 处、`Momus` 5 处。
  这些在 RustCode 全无对应物，技能主体逻辑（状态文件读写、reviewer 解锁条件）无法平移。
- 可复用部分：骨架仍有价值——「先探索、只问探索解决不了的岔路、等明确批准、产出一份决策完整的计划」
  与 RustCode 的 `docs/plans/YYYY-MM-DD-*.md` 约定兼容，可作为后续**原创**技能的骨架来源，但不属于本批。
- 放弃（本批）。

### 4.4 frontend（2.9M）

- 根因（三重）：
  1. **重资产**：2.9MB、`references/ui-ux-db/data/` 下 24 个 CSV 知识库 + 4 个 ruleset 路由，对 RustCode（Rust CLI/TUI/daemon，无 Web 前端主业）性价比极低；
  2. **第三方许可链不明**：`references/designpowers/vendor/` 下 vendored 了大量第三方 agent/skill 内容，
     在 SUL-1.0 之外还叠加了第三方条款，引入即产生无法在本批澄清的合规风险；
  3. **技术栈不匹配**：React / Lighthouse / Core Web Vitals / Playwright 场景与仓库现状无关
     （`webui/` 是既有前端，不由 agent 日常改动）。
- 放弃。若将来确需，可只摘「不含第三方来源」的检查清单条目，成本 M，且必须先过合规复核。

### 4.5 visual-qa（116K）

- 根因：核心取证手段是**截图 + 像素 diff**，自带 `scripts/`（png-decode / image-diff / tui-grid / ansi，均 `.ts`/.mjs），
  并依赖 Playwright / agent-browser / dev-browser 与 CDP。RustCode **没有浏览器/截图工具**，
  全仓 `crates/` 下无 playwright/截图能力；`.ts`/`.mjs` 脚本还需要 Node 运行时。
- 价值场景有限：唯一沾边的是 TUI 对齐检查，但同样没有取证手段。
- 放弃。

### 4.6 ultimate-browsing（296K）

- 根因：整套 TIER 1/1.5/2 依赖本地 Python（curl_cffi TLS  impersonation、yt-dlp）、Jina Reader、
  Playwright 真 Chrome、CloakBrowser stealth Chromium 与 CDP 点击/表单/截图/cookie 登录。
  RustCode 侧只有 `web_fetch` / `web_search`，且 `AGENTS.md` 规定**出站 HTTP 只有一个入口**
  （`capabilities/src/egress/` 的 `egress::client::build_http_client`）——该技能"绕过 WAF/反爬"的定位
  与本仓的出站治理和安全边界约束直接冲突。
- 放弃。

## 五、工具映射表（OMO 依赖 -> RustCode 替代方案）

| OMO 依赖 | RustCode 替代 | 说明 |
| --- | --- | --- |
| `call_omo_agent` | `task` | 参数：`subagent_type: explore`（只读）/ `worker`（可写），`difficulty: simple`（fast 模型）/ `hard`（capable 模型）；整批同步返回，无轮询 |
| `lsp_diagnostics` | `bash` + `cargo check` / `cargo clippy` / `cargo test` | `diagnostics` 工具已定义但全仓未注册，拿不到；`lsp` 工具受 `lsp.enabled` 配置门控，不做默认假设 |
| `lsp_goto_definition` / `lsp_find_references` / `lsp_rename` / `lsp_symbols` / `lsp_status` | `grep` / `ast_grep` / `task` 派 `explore` 子代理 | 结构性查找优先 `ast_grep`（只读） |
| `background_output`（轮询后台任务） | 不需要：`task` 整批同步返回 | 不要臆造轮询工具 |
| `session_*` / 会话存储检索 | 原生 SessionManager + `/resume` + `/bg` + `bash` 查 `~/.rustcode/sessions/` | OMO 的 opencode/Codex 存储格式不适用 |
| `deep` / `unspecified-high` 档位 | `task` 的 `difficulty: hard` / `simple`；`role` 可选 architect/reviewer/tester/rust/tui_ux | 无 Oracle/Metis/Momus 角色体系 |
| `oracle` | 无对应：用 `task` 派 `explore` 子代理 + 明确 prompt | 换用"维度切分"而非"角色切分" |
| TodoWrite | `todowrite` | 受 `RUSTCODE_TODO` 环境变量门控 |
| TodoWrite 之外的 `.omo/plans/` / Boulder 状态 | `docs/plans/YYYY-MM-DD-*.md` + `todowrite` | 计划类文档沿用该命名 |
| `scripts/ast_grep_helper.py`（replace/scan/validate） | `ast_grep`（只读搜索）+ `search_replace` / `parallel_edit_files` / `bash` 调 `sg` | `ast_grep` 无 rewrite 参数 |
| 5 个并行 agent 一批 | `task` 一批 3-5 个，prompt 务必简短 | 整批一个 JSON 载荷，长 prompt 会被拒；`explore` 并发上限 8、`worker` 上限 3 |
| Playwright / 浏览器 / 截图 / CDP | 无 | 相关技能全部放弃 |
| `uv` / numpy / DuckDB / Polars | 无原生工具，仅可经 `bash` 现装现用 | 见 §3.3 |

## 六、命名映射与校验

| OMO 名 | RustCode 名 | 校验 |
| --- | --- | --- |
| ast-grep | `rustcode-ast-grep` | 通过 `validate_skill_name` |
| debugging | `rustcode-debugging` | 同上 |
| refactor | `rustcode-refactor` | 同上 |
| git-master | `rustcode-git-master` | 同上 |
| remove-ai-slops | `rustcode-remove-ai-slops` | 同上 |
| review-work | `rustcode-parallel-review` | 同上；改名是为了避免与内置 `code_review` 撞名 |

命名规则来源：`crates/rustcode-capabilities/src/skills/skill.rs:297` `validate_skill_name`
——长度 1-64、仅 `[a-zA-Z0-9_-/]`、不以 `/` 或 `-` 开头结尾、不含 `//` 或 `--`。

## 七、合规声明

本批次未复制任何 SUL-1.0 文本：6 个技能只借鉴了 oh-my-openagent 同名技能的流程骨架
（阶段划分、先验证后修改的顺序、门禁判据、检查清单写法），正文均为 RustCode 依据自身工具集
与 `AGENTS.md` 约束原创重写，且每个 `SKILL.md` 末尾的「来源与适配」小节均已声明该事实。
被判定为「需改造后可行」的 3 个技能尚未动工，后续若实施同样只取骨架、不取原文。
