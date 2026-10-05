# RustCode 代码开发团队配置方案（面向 CodeBuddy 长期迭代）

> 适用对象：以 CodeBuddy 为主调度者的 RustCode（`SecLab/RustCode`）二次开发协作。
> 目标：把需求→设计→实现→审查→测试→交付→维护的全生命周期，映射为一套**可执行、可门禁、可复盘、可长期迭代**的团队配置。
> 所有结论均落地到代码与实测，不以臆测替代 `AGENTS.md`、原生 `team` 工具与既有 agent 约定。

## 0. 摘要

本方案定义两层角色体系 + 一条 G1–G6 门禁流水线 + 一套任务分配机制：

- **编排层（CodeBuddy agent）**：7 个既有 agent 承担需求/设计/实现/审查/测试/文档/编排，带 Bash，可跑 `cargo` 自检。
- **运行时层（RustCode 原生 `team` 工具）**：14 个内置 role，用于产品自身的 agent 运行时派发，以及编排层发起的只读扇出与限定文件编辑。
- **铁律**：任何 `Worker` 角色（原生 team）**不能运行 Bash**，故编译/测试验证一律回到主会话或带 Bash 的 agent；子代理编译必 idle timeout，长编译任务走主进程 `setsid nohup ... &` + 轮询。

---

## 1. 现状与需求分析

### 1.1 既有能力盘点

| 能力 | 落点 | 现状 |
|------|------|------|
| 原生团队派发 | `crates/rustcode-coding/src/team/tool.rs` | `team` 工具，14 角色，双通道，层级派发，`max_depth=2` |
| 角色表 | `crates/rustcode-capabilities/src/team.rs` `BUILT_IN_ROLES` | 14 项，枚举驱动 schema，禁止漂移 |
| 单子代理 | `crates/rustcode-capabilities/src/tools/task.rs` | `task` 工具，explore/worker 通道 |
| 编排 agent | `.codebuddy/agents/*.md`（7 个） | G1–G6 流水线，feature-slug 交接件 |
| 团队运行态 | `.codebuddy/teams/<id>/<name>/` | `config.json` + 消息日志 |

原生 `team` 14 角色（来自 `team.rs`）：`planner` `architect` `explorer` `implementer` `rust` `tui_ux` `reviewer` `tester` `debugger` `security` `performance` `docs_writer` `release_manager` `migration_compat`。
权限通道（来自 `TeamPermission`）：`Explore`(只读) / `Worker`(限定写入)。`Worker` 角色**必须带非空 scope、不能运行 Bash**（见 `tool.rs` 约束与 `task.rs` 的 `WorkerScopeGate`）。

### 1.2 真实约束（实测，来自 AGENTS.md 与记忆）

- **分支**：`dev` 唯一开发分支；`main` 仅上游 `upstream/main` 镜像，禁止从 dev 合并、禁止自造提交、禁止 push main。
- **内存**：cgroup 上限 **4 GiB**（AGENTS.md 写的 8GB 已过时）；链接大 debug 二进制会 OOM。绕法 `cargo build -j 1 --config 'profile.dev.package.rustcode.debug=0'`，**别用 RUSTFLAGS**。
- **磁盘**：`target/debug/incremental` + `examples` 是可弃缓存，满盘时删之恢复；保留 `deps/` 下的 `.rlib/.rmeta/.d/.o`。
- **测试**：全量必须 `cargo test -j 1 --workspace --no-fail-fast`（防 cgroup OOM + daemon 固定端口 13456-13458 争用）。
- **隔离**：`coding/tuix/daemon/capabilities/cli` 入口文件顶部 `#[ctor]` 把 `RUSTCODE_HOME` 重定向临时目录；改这些目录名须同步改。
- **门禁 G1–G6**：`cargo fmt --check` / `cargo clippy --workspace --all-targets -- -D warnings` / `cargo test --workspace` / `test-headless.sh` / `acp_smoke.py` / 无遥测 grep。另 G7 无陈旧命名、G8 中文文档门禁（`scripts/check-zh-docs.py`，AC-4 对 AGENTS.md/README 恒红维持现状；AC-32 禁 emoji；AC-7b 禁新增 markdown 链接）。
- **CLI 包名**是 `rustcode`（`-p rustcode-cli` 报错）。
- **i18n**：`Msg` 带 `'a` 且 `Copy`；动态变体用 `&'a str`，`en/zh_cn/messages` 三处同步（穷尽匹配由编译器强制）。
- **子代理编译必 idle timeout**（已记）：编译型任务派 `code-implementer` 等子代理会在 cargo 阶段挂死；纯文档（`doc-writer`）可正常派发。

### 1.3 待解决痛点

1. 编译/测试验证若随子代理派发会超时，需明确"验证回到主会话"的职责边界。
2. `Worker` 角色无 Bash，原生 `team` 派发的实现任务无法自检，须有回主会话的验收闭合。
3. 并行开发需 `files_owned` 两两不相交，避免覆盖他人 WIP（工作树常含他人未提交改动）。
4. 长生命周期迭代需记忆与门禁回归，防止重复踩坑。

---

## 2. 设计原则

1. **权限最小且 fail-closed**：只读用 `Explore`、写入用 `Worker`+scope；scope 重叠、worker 缺 scope、越权写 `.git/` 一律拒绝。
2. **验证权集中**：编译/测试/clippy/fmt 只在主会话或带 Bash 的编排 agent 执行；原生 `team` 的 `Worker` 只做文件编辑，不自检。
3. **契约冻结、所有权唯一**：设计阶段冻结接口契约；每个文件由唯一 `files_owned` 任务独占；跨 crate 默认串行。
4. **门禁驱动流转**：G1–G6 是阶段切换唯一裁判，未过门禁不进下一阶段。
5. **交接件留痕**：每个阶段产出 `.codebuddy/artifacts/<feature-slug>/` 下的结构化文件（YAML 信封 + 正文），保证可回溯、可复盘。
6. **长期可迭代**：记忆（`MEMORY.md` + 日签）记录铁律与口径；门禁作为回归基线；每次收尾写交付清单。

---

## 3. 团队角色定义

### 3.1 两层角色体系

| 层 | 载体 | 权限 | 适用 | 是否可跑 Bash |
|----|------|------|------|---------------|
| 编排层 | `.codebuddy/agents/*.md`（CodeBuddy agent） | 由 agent 自身 `tools:` 决定 | 全生命周期推进 | 是（按 agent 声明） |
| 运行时层 | 原生 `team` 14 role | `Explore` / `Worker` | 产品 agent 运行时派发、只读扇出、限定编辑 | `Explore` 可读不可写；`Worker` 不可跑 Bash |

> 对 CodeBuddy 开发 RustCode 而言，**日常驱动走编排层**（agent 带 Bash，能编译自检）；原生 `team` 工具用于：(a) 让 RustCode 自身的 kernel agent 编排子团队（保持产品能力兼容），(b) 编排层发起大规模只读勘察扇出（explorer/reviewer/security/performance），(c) 限定文件的低风险编辑（`implementer`/`rust`/`tui_ux`/`docs_writer`，但改完回主会话验收）。

### 3.2 角色—职责映射

**编排层（7 agent，来自 `.codebuddy/agents/`）**

- `project-manager`（opus，manual）：唯一消息总线、状态所有者、门禁裁判。创建 feature slug、按 G1–G6 驱动、派发并路由交接件、控并行批次与 `files_owned` 冲突。不写业务代码。
- `requirements-analyst`（sonnet）：入口。澄清目标/非目标，把模糊诉求转为可自动化验收的 AC；识别是否触碰持久化/公共协议/安全边界/运行时生命周期。不做技术选型、不写源码。
- `solution-architect`（opus）：冻结接口契约、确定单一状态所有权、失败/取消语义、拆带依赖与 `files_owned` 的任务批次。禁止改源码、禁跑构建。
- `code-implementer`（sonnet，带 Bash）：在 `files_owned` 内最小改动实现，自跑编译与测试，出含证据的实现报告。禁改契约、禁扩范围、禁提交。
- `code-reviewer`（sonnet，只读）：基于 git diff + 实现报告做分级审查，G4 门禁唯一依据。禁改源码。
- `test-engineer`（sonnet，带 Bash）：补单元/集成测试，覆盖正常/失败/取消/重试/并发路径，跑 crate 与 workspace 验证，G5 依据。禁改生产逻辑（发现问题回退 implementer）。
- `doc-writer`（sonnet，带 Bash 但禁 cargo/npm 构建）：同步架构/设计文档、CHANGELOG、四段式交付清单；允许跑 `check-zh-docs.py`。

**运行时层（原生 `team` 14 role，来自 `team.rs`）映射到 RustCode 开发语境**

| role | 通道 | 在 RustCode 开发中的用法 |
|------|------|--------------------------|
| `planner` | Explore | 任务分解与派发计划（等价 requirements/architect 的轻量只读版） |
| `architect` | Explore | 运行时所有权、crate 边界、协议/持久化影响勘察（只读） |
| `explorer` | Explore | 大规模代码搜索/调用链发现，扇出（上限 8） |
| `reviewer` | Explore | 只读 bug 聚焦审查（与 code-reviewer 互补，跑在 team 层） |
| `debugger` | Explore | 失败复现与根因隔离（只读） |
| `security` | Explore | 审批/密钥/scope/自动执行风险评审（只读） |
| `performance` | Explore | 并发/令牌/渲染/延迟/内存分析（只读） |
| `migration_compat` | Explore | legacy/importer/线兼容评审（只读） |
| `release_manager` | Explore | 最终验证矩阵与分支卫生检查（只读，不经 release 脚本写产物） |
| `implementer` | Worker | 限定文件聚焦编辑（无 Bash，改完回主会话验收） |
| `rust` | Worker | Rust async/trait/错误/测试相关编辑（无 Bash） |
| `tui_ux` | Worker | TUI 状态/布局/交互编辑（无 Bash） |
| `tester` | Worker | 测试文件编辑（无 Bash；实际跑测试回主会话） |
| `docs_writer` | Worker | 用户文档/评测指令编辑（无 Bash） |

### 3.3 已落地 CodeBuddy agent 清单（13 个，对应两层体系）

| 层 | Agent | 通道 | 落点文件 |
|----|-------|------|----------|
| 编排 | `project-manager` | 编排（opus, manual） | `.codebuddy/agents/project-manager.md` |
| 编排 | `requirements-analyst` | 编排（sonnet） | `.codebuddy/agents/requirements-analyst.md` |
| 编排 | `solution-architect` | 编排（opus） | `.codebuddy/agents/solution-architect.md` |
| 实现 | `code-implementer` | 实现（sonnet, 带 Bash） | `.codebuddy/agents/code-implementer.md` |
| 审查 | `code-reviewer` | 只读（sonnet） | `.codebuddy/agents/code-reviewer.md` |
| 测试 | `test-engineer` | 测试（sonnet, 带 Bash） | `.codebuddy/agents/test-engineer.md` |
| 文档 | `doc-writer` | 文档（sonnet） | `.codebuddy/agents/doc-writer.md` |
| 只读 | `explorer` | Explore（sonnet） | `.codebuddy/agents/explorer.md` |
| 只读 | `security` | Explore（sonnet） | `.codebuddy/agents/security.md` |
| 只读 | `performance` | Explore（sonnet） | `.codebuddy/agents/performance.md` |
| 只读 | `debugger` | Explore（sonnet） | `.codebuddy/agents/debugger.md` |
| 只读 | `migration-compat` | Explore（sonnet） | `.codebuddy/agents/migration-compat.md` |
| 只读 | `release-manager` | Explore（sonnet, 仅只读 git） | `.codebuddy/agents/release-manager.md` |

> 7 个编排层 agent 原已存在；本次为落地「两层角色体系」补 `explorer`/`security`/`performance`/`debugger`/`migration-compat`/`release-manager` 六个只读专家（对应原生 `team` 的 `Explore` 通道中尚未独立成 agent 的角色）。`Worker` 通道的实现类角色（`implementer`/`rust`/`tui_ux`/`tester`/`docs_writer`）由 `code-implementer` 覆盖，不重复生成。`project-manager` 已接线这 6 个专家（见其「可派发 Agent 与用途」表与步骤 2–5 的扇出时机）。

---

## 4. 分工矩阵（RACI，按生命周期）

R=负责执行，A=最终问责，C=被咨询，I=被告知。

| 阶段 | 需求 | 设计 | 实现 | 审查 | 测试 | 交付 | 维护 |
|------|------|------|------|------|------|------|------|
| 需求澄清 | A/R | C | I | I | I | I | C |
| 架构与契约冻结 | C | A/R | I | C | C | I | C |
| 任务拆分/批次 | C | R | C | I | C | I | I |
| 编码实现 | I | C | A/R | I | I | I | C |
| 代码审查(G4) | I | C | C | A/R | C | I | I |
| 测试验证(G5) | I | C | C | C | A/R | I | C |
| 文档/CHANGELOG(G6) | I | C | I | I | C | A/R | C |
| 分支/发布卫生 | I | I | I | I | I | C | A/R |
| 编译/门禁实跑 | I | I | R(主会话代跑) | I | R(主会话代跑) | C | R |

注：**编译与 G1–G6 门禁实跑的唯一执行点为主会话（或带 Bash 的 `code-implementer`/`test-engineer`）**；原生 `team` 的 `Worker` 角色不执行 Bash。

---

## 5. 协作流程

### 5.1 G1–G6 质量门禁流水线（阶段切换裁判）

```
需求(00-requirement, approved)
   │
   ▼
设计(01-design, 契约冻结 + 02-tasks, 批次划分)   ← solution-architect，禁编译
   │
   ▼
并行开发(03-impl, files_owned 内最小改动)        ← code-implementer
   │   每任务自跑 cargo check/test（主会话或带 Bash agent）
   ▼
审查(G4, 04-review/<task>.md)                     ← code-reviewer，只读裁决
   │   rework → 回 implementer；reject → 回 architect
   ▼
集成测试(G5, 05-test-report.md)                   ← test-engineer
   │
   ▼
交付(G6, 06-delivery, 四段式清单 + CHANGELOG)      ← doc-writer
   │
   ▼
发布卫生(release_manager 检查分支/验证矩阵，不经 release 脚本写产物)
```

门禁命令（来自 AGENTS.md）：
- `G1` `cargo fmt --check`（零散改用 `cargo fmt -p <包名>`，裸 `cargo fmt` 波及全仓）
- `G2` `cargo clippy --workspace --all-targets -- -D warnings`
- `G3` `cargo test -j 1 --workspace --no-fail-fast`
- `G4` `./scripts/test-headless.sh`（需先 `cargo build`）
- `G5` `python3 scripts/acp_smoke.py`
- `G6` 遥测 grep + `python3 scripts/check-zh-docs.py gate`

### 5.2 交接件契约（`.codebuddy/artifacts/<feature-slug>/`）

既有 agent 已约定 YAML 信封（`kind`/`id`/`from`/`to`/`feature`/`status`/`decision`/`requires`/`files_owned`/`architecture_constraints`）。本方案强制：
- 每个阶段产出独立文件，信封完整，`decision` 明确 `proceed|rework|block|escalate`。
- `files_owned` 在 `02-tasks.md` 冻结，并行批次两两不相交，跨 crate 串行。
- 返工满 2 轮仍被要求改 → `escalate` 升级用户，禁止第 3 轮自发返工。

### 5.3 原生 `team` 工具运行生命周期

编排层对外（或产品自身 kernel agent）派发子团队时，使用 `team` 工具的动作信封：

- `delegate`：`{"action":"delegate","tasks":[{"description","prompt","role","scope?","subagent_type?"}]}`。`role` 是通道唯一权威；`subagent_type` 仅为别名且必须与 `role` 的通道一致，否则整批报错。
- `status` / `wait`(`timeout_secs`≤300) / `result` / `stop`：用返回的 `run_id` 跟踪。
- 层级派发：子成员可再 `delegate` 至 `max_depth=2`（三层黑板树）；根→子→孙。
- 扇出上限：Explore 通道 8 并发、Worker 通道 3 并发（独立信号量，互不影响）。

### 5.4 消息总线与 hand-off 模式

- 唯一总线 = `project-manager`；成员间不跨级直连，一律经总线路由交接件。
- 既有 `.codebuddy/teams/<id>/<name>/` 的消息日志即 hand-off 记录（`from`/`to`/`type`/`content`/`timestamp`），作为仲裁与复盘证据。

---

## 6. 技术栈要求

| 维度 | 要求（实测） |
|------|--------------|
| 语言/Rust | workspace `edition="2021"` + `resolver="2"`；**无** `rust-toolchain`、**无** `rust-version` MSRV 约束，不臆增版本钉 |
| 构建 | `cargo build`（default-members）；CLI 包名 `rustcode`；`--locked` 安装；全量 `cargo build --workspace` |
| 门禁 | `fmt` + `clippy -D warnings` + `test -j 1` 为强制；`clippy` 须零 warning |
| WebUI | Preact，`Node>=22.6`；`npm ci` + `tsc --noEmit` + `node --test`；模块间值导入带 `.ts` 扩展名；构建后 `cargo clean -p rustcode-daemon` |
| 交叉构建 | musl 编 linux x64/arm64（`scripts/install-musl-cross.sh` 单一来源，非 musl.cc）；windows-msvc 用 `crt-static` |
| 脚本 | Python 3（`check-zh-docs.py`/`gitcode_release.py`/`acp_smoke.py`） |
| i18n | `Msg: Copy + 'a`，动态变体 `&'a str`，`en/zh_cn/messages` 三处同步 |
| 文档纪律 | 禁 emoji（AC-32）；禁新增 markdown 链接（AC-7b）；用行内 code span 表达路径 |
| 测试隔离 | 入口 `#[ctor]` 重定向 `RUSTCODE_HOME`；新 crate/二进制须自带一份 |
| 资源 | cgroup 4GiB → 链接用 `-j 1 --config 'profile.dev.package.rustcode.debug=0'`；满盘清 `target/debug/incremental`+`examples`；禁 `sudo` |

---

## 7. 任务分配机制

### 7.1 任务进入与优先级

1. 新需求 → `requirements-analyst` 产出 `00-requirement.md`（AC 可测、边界清晰、声明是否触碰持久化/协议/安全/运行时）。
2. 通过 → `solution-architect` 冻结契约 + `02-tasks.md`（任务 id、crate、`files_owned`、依赖、AC、验证命令、复杂度）。
3. `project-manager` 按批次派发；同批次 `files_owned` 两两不相交，并行任务 ≤ 3。

### 7.2 `files_owned` 与并行批次

- 每个任务在 `02-tasks.md` 声明独占写入路径；运行时 `validate_non_overlapping_worker_scopes` 对字面路径/递归 glob/常见 glob 做重叠校验，重叠即拒绝派发。
- 跨 crate 改动默认串行（避免依赖方向反转与状态所有权冲突）。
- 提交共享文件按 hunk 精挑（`git commit -m "..." -- <paths>`），避免卷走他人 WIP。

### 7.3 role / permission / scope 映射（编译型任务特殊处理）

- 只读勘察/审查/安全/性能/兼容 → `Explore` 通道（`explorer`/`reviewer`/`security`/`performance`/`migration_compat`/`debugger`/`architect`/`planner`/`release_manager`），无需 scope，扇出 8。
- 文件编辑 → `Worker` 通道（`implementer`/`rust`/`tui_ux`/`tester`/`docs_writer`），**必须带非空 scope**，并发 3，**不跑 Bash**。
- **编译/门禁自检不作为 Worker 职责**：Worker 改完回报主会话，`project-manager` 调主会话（或带 Bash 的 `code-implementer`/`test-engineer`）跑 `cargo check/test/clippy`。
- 长编译（如全量 `cargo test --workspace`）走主进程：`setsid nohup cargo test -j 1 --workspace --no-fail-fast > <唯一日志> 2>&1 &`，轮询；同一日志路径禁两进程共用。

### 7.4 实例化模板（原生 `team` delegate 示例）

只读扇出（架构+安全+性能 并行勘察，无需 scope）：

```json
{
  "action": "delegate",
  "tasks": [
    {"description": "调查运行时所有权边界", "prompt": "定位 CodingRuntime 的唯一 owner 与生命周期不变量，给出 文件:行 引用", "role": "architect"},
    {"description": "评审认证/密钥/scope 风险", "prompt": "审查本次改动涉及的令牌与路径 scope，列出越权点", "role": "security"},
    {"description": "分析并发与延迟热点", "prompt": "分析调度 tick 的并发与令牌消耗，给出热点", "role": "performance"}
  ]
}
```

限定编辑（三个 Worker，scope 两两不相交）：

```json
{
  "action": "delegate",
  "tasks": [
    {"description": "改 CLI 子命令参数", "prompt": "在 crates/rustcode-cli/src/main.rs 内按契约加 --no-auth 参数", "role": "rust", "scope": ["crates/rustcode-cli/src/main.rs"]},
    {"description": "改 TUI 渲染", "prompt": "在 crates/rustcode-tuix/src/render/ 内修复布局", "role": "tui_ux", "scope": ["crates/rustcode-tuix/src/render/**"]},
    {"description": "补测试", "prompt": "在 crates/rustcode-cli/tests/ 内加单测", "role": "tester", "scope": ["crates/rustcode-cli/tests/**"]}
  ]
}
```

---

## 8. 适配现有架构与约束

- **分支纪律**：所有改动只在 `dev`；`main` 仅上游同步；发布 tag 从 `dev` 打。团队运行时绝不写 `main`。
- **依赖方向**：`kernel`/`capabilities`/`coding` 生产依赖 core-free；`capabilities` 不反向依赖 core/L2/前端；新增抽象须过 `solution-architect` 架构边界核对。
- **运行时生命周期不变量**：submit/steer/cancel/approval/reload/session/compact/undo 等事件须由单一 `CodingRuntime` owner 管理；迟到事件不得污染 replacement runtime。
- **持久化模型**：native `SessionManager/SessionMeta/SessionSnapshot` 唯一；历史 core JSON 仅 daemon 私有 DTO 单向导入，禁双向转换/legacy writer。
- **资源约束**：编译全部 `-j 1`；链接 OOM 用 debug=0 配置；满盘清缓存；禁 `sudo`。
- **门禁回归**：每次提交前主会话实跑 G1–G3；`Worker` 编辑后由主会话补跑，不把门禁推给无 Bash 角色。
- **文档纪律**：交付文档禁 emoji、禁新增 markdown 链接，用 code span。

---

## 9. 长期可迭代保障

1. **记忆系统**：每日进度写 `.codebuddy/memory/YYYY-MM-DD.md`；跨会话铁律（包名、i18n 三表、子代理编译超时、cgroup 4GiB）入 `MEMORY.md`。开工前读 `MEMORY.md` + 当日/前一日日签。
2. **门禁即回归基线**：G1–G6 全绿作为每轮收尾硬指标；任何门禁变动须同步改 `AGENTS.md`（维护规则强制）。
3. **口径铁律**：覆盖率先看文档覆盖率而非字符比；定位字段先声明作用域（文档内/跨文档），`page` 与 `doc_id` 同现；口径声明作为开工前置。
4. **复盘与交付清单**：每轮完成写四段式交付（行为变化/风险/验证结果/已知未验证范围），并标注未验证范围，禁止编造已验证结论。
5. **孤儿与漂移**：定期 `git ls-files` 核对入库状态（已跟踪文件不受 `.gitignore` 约束）；清理 `target` 缓存与孤儿 artifact。

---

## 10. 落地清单与启动步骤

1. **确认团队运行态已存在**：`.codebuddy/teams/<id>/<name>/config.json` 记载 `workspacePath=/workspace/RustCode`、成员与 `sessionId`；新会话沿用既有团队而非重建。
2. **启动一个特性**：`project-manager` 创建 `feature-slug`，按 §5.1 驱动 G1–G6；首任务 `requirements-analyst` 出 `00-requirement.md`。
3. **拆批派发**：`solution-architect` 出 `01-design.md`+`02-tasks.md`；`project-manager` 校验 `files_owned` 不相交后并行派发（≤3）。
4. **编译托管**：`code-implementer`/`test-engineer` 带 Bash 自跑；若需全量测试，主会话 `setsid nohup cargo test -j 1 --workspace --no-fail-fast > /tmp/<slug>.log 2>&1 &` 轮询。
5. **只读扇出可选**：大规模勘察用原生 `team` 的 `Explore` 通道（示例见 §7.4），不占 Bash 配额。
6. **收尾**：`doc-writer` 出 G6 交付 + CHANGELOG；`release_manager` 核对分支卫生（不写 release 产物）；写当日记忆；仅 `dev` 提交。

> 编排层 7 个 agent 原已存在；本次为落地「两层角色体系」补 6 个只读专家（共 13 个），对应原生 `team` 的 `Explore` 通道中尚未独立成 agent 的角色；不改 `BUILT_IN_ROLES`（14 项已覆盖开发语境）。核心是明确两层如何接线、为何验证权集中、约束如何落地，使 CodeBuddy 能持续、门禁驱动、可复盘地推进 RustCode 开发。
