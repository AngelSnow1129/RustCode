---
kind: task
id: TASKS-001
from: solution-architect
to: [project-manager, code-implementer]
feature: 2026-09-07-zh-docs-webui
status: ready
decision: proceed
requires: [DESIGN-001]
created: 2026-09-07
---

# TASKS-001 · 任务图（45 个任务 / 17 个批次）

> 上游：`00-requirement.md`（REQ-002, approved）、`00-decisions.md`（Q1–Q5 冻结）、`01-design.md`（DESIGN-001）。
> 冻结契约（不得擅改）：`01-design.md §4`。
>
> **角色分工**
> - 汉化任务（D-01 … D-36）→ **`doc-writer`**
> - 代码面 / 脚本任务（T-01 … T-05）→ **`code-implementer`**
> - 集成验收（T-06、T-07 中除脚本外的手工/冒烟项）→ **`test-engineer`**
>
> **所有任务的开工前置动作**（`AGENTS.md:183-185`）：记录 `git rev-parse --short HEAD` 与 worktree 状态，
> 并对每个 `files_owned` 跑一次 `git log --oneline -5 -- <file>` 确认任务未被做过或方向已变。

---

## 0. 全局约定

### 0.1 基线修订号

```bash
export ZH_BASE=3ee655e3      # 所有 check/gate 的比较基线，禁止改用 HEAD
```
> 原因：中途若分批 commit，与 `HEAD` 比较会产生假绿。只有 PM 显式决定重新基线时才可传 `--base`。

### 0.2 通用验证命令

```bash
# 汉化任务（把 <paths…> 换成本任务的 files_owned，逐路径列出）
python3 scripts/check-zh-docs.py check \
  --base "$ZH_BASE" \
  --report .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-<TASK-ID>.md \
  --files <paths…>

# 等价于自动取工作区改动（若本任务改动尚未 commit）
python3 scripts/check-zh-docs.py check --base "$ZH_BASE" --diff
```

退出码 `0` = 通过；`1` = 有 FAIL（看报告里的 offender 行号）；`2` = 用法/环境问题（**不得**当作通过）。

### 0.3 全局不变量（每个任务都受约束）

1. 只改 `files_owned` 内的路径；越界改动 ⇒ 该批 FAIL。
2. `crates/rustcode-review/rules/**`（47 个）**一个字符都不动**（Q1）。
3. `extensions/jetbrains/CHANGELOG.md`、`docs/{ORIGINAL_LICENSE,THIRD_PARTY_NOTICES,UPSTREAM_CREDITS,UPSTREAM_RUSTCODE_LICENSE}.md` **不动**（A/C 类）。
4. `AGENTS.md`、`.codebuddy/artifacts/2026-09-07-zh-docs-webui/**` **不动**（owner = SA/PM/TE）。
5. 不新增 Emoji；**不顺手清理既有 Emoji**（`AGENTS.md:244` 有界例外）。
6. 不改文件名、不改链接目标、不改 frontmatter 键名。

---

## 1. 任务列表

### 批次 0A（并行，前置；owner = `code-implementer`）

> 与汉化批次**文件集合完全不相交**（`scripts/**` + `crates/**/*.rs` vs `**/*.md`），**可与 Batch 1 同时开工**。
> 唯一依赖：Batch 1 及以后需要 T-01 产出的 `scripts/check-zh-docs.py`。

#### T-01 · 汉化验收脚本（**第一个产出，后续所有汉化批的自检工具**）

- **批次**：0A ｜ **依赖**：无 ｜ **crate**：无（仓库工具） ｜ **owner**：`code-implementer` ｜ **复杂度**：L
- **目标**：实现 `01-design.md §4.1` 冻结的 CLI 契约，使 AC-1/2/3/4/6/7/8/32 全部可脚本判定。
- **files_owned**：
  - `scripts/check-zh-docs.py`（新增）
  - `.codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/md-inventory.md`（`inventory` 输出；**仅本任务与 T-07 可写**）
- **必须实现的子命令**：`inventory` / `check`（`--files` 与 `--diff` 双模式）/`hostscan` / `gate`
- **验收标准**：
  - AC-1：`inventory` 输出清单满足 `SKIP_A + SKIP_B + SKIP_C + TODO + ZH == git ls-files '*.md' 总数`；A=4、B=47、C=1
  - AC-2：能对每个文件输出 `total / en / ratio` 与逐行 offender
  - AC-4 / AC-7b / AC-32 / AC-6：能基于 `git show $ZH_BASE:<path>` 做双轨比较并在不等时 FAIL
  - 自身可用：`python3 -m py_compile scripts/check-zh-docs.py` 通过；stdlib only
  - 自证：`python3 scripts/check-zh-docs.py check --base $ZH_BASE --files README.md` 在**未改动**时 AC-4/7/32 三项必须 PASS（防止比较逻辑本身有 bug）
- **验证命令**：
  ```bash
  python3 -m py_compile scripts/check-zh-docs.py
  python3 scripts/check-zh-docs.py inventory --base "$ZH_BASE" \
    --out .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/md-inventory.md
  python3 scripts/check-zh-docs.py check --base "$ZH_BASE" --files README.md docs/architecture.md
  python3 scripts/check-zh-docs.py hostscan --base "$ZH_BASE"
  ```

#### T-02 · 前端一键构建脚本 `scripts/build-webui.sh`

- **批次**：0A ｜ **依赖**：无 ｜ **crate**：无 ｜ **owner**：`code-implementer` ｜ **复杂度**：M
- **files_owned**：`scripts/build-webui.sh`（新增）
- **目标**：AC-12 … AC-16；E1–E8。契约见 `01-design.md §4.2`。
- **验收标准**：
  - AC-12：存在、`test -x` 为真、`bash -n scripts/build-webui.sh` 通过
  - AC-13：`PATH` 中移除 node/npm 后运行 ⇒ exit ≠ 0，stderr 含 `node` 最低版本（从 `webui/package.json` 的 `engines.node` 动态解析，解析失败回退 `22.6`），且 `test ! -e webui/dist/index.html`
  - AC-14：伪造 `node` 版本 < 22.6 ⇒ exit ≠ 0，输出含当前版本与要求版本，**不得**降级为警告继续
  - AC-15：(a) 默认重跑构建，跑完 `test -f webui/dist/index.html`；(b) `--if-missing` 且 dist 已存在 ⇒ 跳过 exit 0；(c) `--if-missing` 且 dist 不存在 ⇒ 正常构建
  - AC-16：(a) `git check-ignore -v webui/dist/index.html` 有输出且 `git status --porcelain webui/dist` 为空；(b) `grep -rn "npm\|npx" crates/*/build.rs` **0 命中**（现状已满足，**不得破坏**）；(c) `git diff "$ZH_BASE" -- .gitignore` 为空
  - E8：成功结尾必须打印 `cargo clean -p rustcode-daemon`
  - 全程无 `sudo`
- **验证命令**：
  ```bash
  bash -n scripts/build-webui.sh
  env PATH=/usr/bin:/bin bash -c 'command -v node; command -v npm'   # 预期均无输出
  env PATH=/usr/bin:/bin ./scripts/build-webui.sh; echo "exit=$?"     # 预期 != 0
  ./scripts/build-webui.sh --if-missing; echo "exit=$?"               # 视 dist 是否存在
  git check-ignore -v webui/dist/index.html; git status --porcelain webui/dist
  grep -rn "npm\|npx" crates/*/build.rs   # 预期 0 命中
  ```

#### T-03 · 默认绑定改为 `0.0.0.0`（CLI 两处 + doc comment）

- **批次**：0A ｜ **依赖**：无 ｜ **crate**：`rustcode-cli` ｜ **owner**：`code-implementer` ｜ **复杂度**：S
- **files_owned**：`crates/rustcode-cli/src/main.rs`
- **改动**（K1/K2/K3，契约见 `01-design.md §4.3`）：
  - `:1047` `default_value = "127.0.0.1"` → `"0.0.0.0"`
  - `:1045-1046` doc comment 同步为 "default 0.0.0.0 …"
  - `:1780` `host: "127.0.0.1".to_string()` → `"0.0.0.0".to_string()`
- **不得改动**：`rustcode daemon` 子命令以外的任何 host；**不动** `crates/rustcode-daemon/src/main.rs:21`
- **验收标准**：
  - AC-19（Q2-A 口径）：`grep -n '127\.0\.0\.1' crates/rustcode-cli/src/main.rs` **不再**命中 `:1047` 与 `:1780`；`crates/rustcode-daemon/src/main.rs:21` **仍为** `127.0.0.1`
  - AC-20：`rustcode webui --host 127.0.0.1` 实际监听 `127.0.0.1:<port>` 且无 `0.0.0.0:<port>`（`ss -ltnp`）
  - AC-31：`cargo fmt --check` 通过；`cargo clippy -p rustcode --all-targets` 无**新增**告警
- **验证命令**：
  ```bash
  cargo fmt --check
  cargo clippy -p rustcode --all-targets 2>&1 | tail -n 20
  cargo run -p rustcode -- webui --help | grep -A1 -- '--host'
  grep -n '127\.0\.0\.1' crates/rustcode-daemon/src/main.rs   # 预期 :21 仍命中
  ```

---

### 批次 0B（并行，前置；owner = `code-implementer`）

#### T-04 · 删除非回环警告 + 修正失实注释

- **批次**：0B ｜ **依赖**：无 ｜ **crate**：`rustcode-daemon` ｜ **owner**：`code-implementer` ｜ **复杂度**：S
- **files_owned**：`crates/rustcode-daemon/src/lib.rs`
- **改动**（K4/K5/K6）：
  - 删除 `:6345-6347` 的 `DaemonWarnNonLoopback` 打印块
  - 删除 `:6343-6344` 描述该打印的中文注释
  - 改写 `:6333-6341` 注释（删掉 "Default to loopback-only for security / PR #82" 的**失实因果叙述**，改为中性说明：默认地址由 driver 决定并强制 token，非回环风险提示由 `Msg::WebuiLanWarning`/`WebuiNonLoopbackWarning` 承担）
- **保留**：`Msg::DaemonWarnNonLoopback` 变体与两语种文案**不动**；`Msg::WebuiLanWarning`/`WebuiNonLoopbackWarning`（`zh_cn.rs:1897-1898`）**不动**
- **验收标准**：
  - AC-21：`rustcode daemon` 与独立 `rustcode-daemon` 以默认参数启动时，输出**不含** `非回环` / `non-loopback` 关键词；`grep -n "DaemonWarnNonLoopback" crates/rustcode-daemon/src/lib.rs` 在生产代码中**无调用点**
  - AC-29：`lib.rs:8854-8891` `channel_mode_tests::known_clients_interactive_on_loopback_or_token` **保持通过**，其 `assert` 无净减少
  - AC-31：`cargo fmt --check`；`cargo test -p rustcode-daemon --lib` 无新增红
- **验证命令**：
  ```bash
  cargo fmt --check
  cargo test -p rustcode-daemon --lib channel_mode_tests 2>&1 | tail -n 15
  grep -n "DaemonWarnNonLoopback" crates/rustcode-daemon/src/lib.rs   # 仅允许出现在注释/测试中
  ```

#### T-05 · i18n 文案（4 条）

- **批次**：0B ｜ **依赖**：无 ｜ **crate**：`rustcode-config` ｜ **owner**：`code-implementer` ｜ **复杂度**：S
- **files_owned**：`crates/rustcode-config/src/i18n/zh_cn.rs`、`crates/rustcode-config/src/i18n/en.rs`
- **改动**（K7–K10，精确文本见 `01-design.md §4.4`）：
  - `zh_cn.rs:1135-1136` / `en.rs:1191-1192`：`CliWebuiNotBuilt` → 指向 `scripts/build-webui.sh`、补 `cargo clean -p rustcode-daemon`、手工路径用 `npm ci`（与 `AGENTS.md:16` 对齐）
  - `zh_cn.rs:2429`：`CliHelpHost` → `绑定地址（默认：0.0.0.0；改 127.0.0.1 则仅本机可访问）`
  - `en.rs:2521`：`Bind address (default: 0.0.0.0; use 127.0.0.1 for local-only)`
- **验收标准**：
  - AC-17a/b/c/d：指向新脚本；含 `cargo clean -p rustcode-daemon`；与 `AGENTS.md:16` 口径一致（`npm ci`）；**中英文案中的命令行逐条相等**
  - AC-31：`cargo test -p rustcode-config --lib` **全量**通过（⚠ `AGENTS.md:263`：i18n 内容测试散落在 `en.rs`/`zh_cn.rs` 末尾测试模块，**不能只跑受影响功能测试**）
- **验证命令**：
  ```bash
  cargo fmt --check
  cargo test -p rustcode-config --lib 2>&1 | tail -n 25
  cargo run -p rustcode -- webui --help | grep -A1 -- '--host'
  ```

---

### 批次 1 — `docs/superpowers/plans` 超大文件（owner = `doc-writer`）

> 共同特征：**无 YAML frontmatter**；**代码块密度 高**（大量 diff/Rust/JSON 代码块与缩进代码）；中英混排密度高。
> 依赖：T-01。

#### D-01 · `2026-05-25-tuix-unified-in-app-scroll.md`（单文件专任务）
- **批次**：1 ｜ **依赖**：T-01 ｜ **复杂度**：L
- **files_owned**：`docs/superpowers/plans/2026-05-25-tuix-unified-in-app-scroll.md`（78.66 KB）
- **注意力**：无 frontmatter；代码块密度**极高**（含大段 Rust diff）；**禁止**翻译任何代码/diff 行
- **验收**：AC-2 ≤ 0.05、AC-4、AC-7b、AC-32、AC-6；报告 `zh-check-D-01.md`

#### D-02 · `2026-05-29-webui.md`（单文件专任务）
- **批次**：1 ｜ **依赖**：T-01 ｜ **复杂度**：L
- **files_owned**：`docs/superpowers/plans/2026-05-29-webui.md`（73.01 KB）
- **注意力**：无 frontmatter；代码块密度高；文中含大量接口签名（`rustcode_core::…` 路径）**一律保留原文**
- **验收**：同 D-01

#### D-03 · vision-preprocessor 双件
- **批次**：1 ｜ **依赖**：T-01 ｜ **复杂度**：L
- **files_owned**：
  - `docs/superpowers/plans/2026-05-08-vision-preprocessor.md`（40.27 KB）
  - `docs/superpowers/plans/2026-05-08-vision-preprocessor-auto-config.md`（32.62 KB）
- **注意力**：无 frontmatter；代码块密度高
- **验收**：同 D-01

### 批次 2 — `docs/superpowers/plans` 主体 A

#### D-04
- **批次**：2 ｜ **依赖**：T-01 ｜ **复杂度**：L
- **files_owned**：`docs/superpowers/plans/` 下：
  - `2026-06-29-acp-agent.md`（44.42 KB，**实测 Han=0，纯英文**）
  - `2026-06-27-v2-rate-limit-pause-resume.md`（41.12 KB）
  - `2026-07-25-round-cap-checkpoint.md`（34.54 KB）
- **注意力**：无 frontmatter；代码块密度高

#### D-05
- **批次**：2 ｜ **依赖**：T-01 ｜ **复杂度**：L
- **files_owned**：`docs/superpowers/plans/` 下：
  - `2026-07-11-persistent-todo-panel.md`（38.24 KB，**Han=6，近乎纯英文**）
  - `2026-07-13-selectable-approval.md`（34.48 KB，**Han=5**）
  - `2026-07-22-multi-question-request-user-input.md`（33.06 KB，**Han=2**）
- **注意力**：无 frontmatter；代码块密度高

#### D-06
- **批次**：2 ｜ **依赖**：T-01 ｜ **复杂度**：L
- **files_owned**：`docs/superpowers/plans/` 下：
  - `2026-04-19-tuix-retained-mode-rewrite.md`（25.86 KB）
  - `2026-07-12-github-style-diff.md`（26.21 KB，**Han=0**）
  - `2026-06-09-cache-friendly-compaction.md`（25.69 KB，**Han=2**）
  - `2026-07-31-local-scheduled-tasks-phase1.md`（23.15 KB）
- **注意力**：无 frontmatter；代码块密度高

### 批次 3 — `docs/superpowers/plans` 主体 B + 根目录 1 件

#### D-07
- **批次**：3 ｜ **依赖**：T-01 ｜ **复杂度**：M
- **files_owned**：`docs/superpowers/plans/` 下：
  - `2026-04-23-cadence-reflection.md`（19.96 KB，**Han=11**）
  - `2026-04-23-merge-current-task-into-cadence.md`（19.49 KB）
  - `2026-07-24-retire-core-conversation-tui-port.md`（19.36 KB）
  - `2026-07-31-local-scheduled-tasks-phase2.md`（18.88 KB）
  - `2026-07-22-deepseek-skill-first-reminder.md`（17.05 KB，**Han=1**）
  - `2026-07-03-terminal-status-glyph.md`（16.78 KB）
- **注意力**：无 frontmatter；代码块密度高

#### D-08
- **批次**：3 ｜ **依赖**：T-01 ｜ **复杂度**：M
- **files_owned**：`docs/superpowers/plans/` 下：
  - `2026-07-22-request-user-input-custom-and-submit-spacing.md`（21.14 KB，**Han=1**）
  - `2026-07-29-webui-sync-compact.md`（12.75 KB）
  - `2026-07-24-skills-multi-compose.md`（12.66 KB）
  - `2026-07-25-retire-core-provider-C2-conversation-transport.md`（11.88 KB）
  - `2026-07-25-retire-core-provider-A-compact.md`（10.78 KB）
  - `2026-07-24-windows-native-tls-schannel-fallback.md`（10.57 KB）
  - `2026-07-31-project-memory-dir-override.md`（10.15 KB）
  - `2026-04-23-agent-harness-principles.md`（10.15 KB）
- **注意力**：无 frontmatter；代码块密度中-高

#### D-09
- **批次**：3 ｜ **依赖**：T-01 ｜ **复杂度**：M
- **files_owned**：
  - `docs/superpowers/plans/2026-07-25-retire-core-provider-B-vision.md`（26.53 KB）
  - `docs/superpowers/plans/2026-07-06-double-esc-undo-cooldown.md`（10.92 KB）
  - `docs/superpowers/plans/2026-07-22-brainstorming-request-user-input.md`（9.95 KB，**Han=2**）
  - `docs/superpowers/plans/2026-07-25-retire-core-tool-ball-D.md`（9.08 KB）
  - `docs/superpowers/2026-07-27-release-v5.0.3-test-checklist.md`（7.02 KB，**已中文，预期 no-op 或极小改动**）
  - `docs/superpowers/plans/2026-07-22-batch-user-questions-persona-nudge.md`（6.79 KB，**Han=1**）
  - `docs/superpowers/plans/2026-04-19-tuix-ink-cell-diff.md`（6.54 KB）
- **注意力**：无 frontmatter；代码块密度中

### 批次 4 — `docs/superpowers/specs`（33 个，≈268 KB）

> 共同特征：**无 YAML frontmatter**；代码块密度**高**；**实测 27 个已是大段中文**（`Han/字母行` 密度 > 0.6），
> 预期多数只需清理英文标题/条目；若 `en_ratio ≤ 0.05` 则记 `no-op`，**不得改写已中文段落**（N-6）。

#### D-10（11 个）
- **批次**：4 ｜ **依赖**：T-01 ｜ **复杂度**：M
- **files_owned**：`docs/superpowers/specs/` 下
  - `2026-04-23-p2-doctor-review-notebook-todo-design.md`（2.43 KB，**Han=0**）
  - `2026-05-08-vision-preprocessor-design.md`（13.82 KB）
  - `2026-05-25-tuix-unified-in-app-scroll-design.md`（16.73 KB）
  - `2026-05-29-provider-add-simplify-design.md`（9.45 KB）
  - `2026-05-29-webui-design.md`（14.42 KB）
  - `2026-06-07-headless-output-format-json-design.md`（8.52 KB）
  - `2026-06-09-cache-friendly-compaction-design.md`（9.72 KB）
  - `2026-06-27-v2-rate-limit-pause-resume-design.md`（8.46 KB）
  - `2026-06-29-acp-agent-design.md`（13.72 KB，**Han=0**）
  - `2026-07-03-terminal-status-glyph-design.md`（6.79 KB）
  - `2026-07-06-double-esc-undo-cooldown-design.md`（4.45 KB）

#### D-11（11 个）
- **批次**：4 ｜ **依赖**：T-01 ｜ **复杂度**：M
- **files_owned**：`docs/superpowers/specs/` 下
  - `2026-07-11-persistent-todo-panel-design.md`（11.18 KB）
  - `2026-07-13-selectable-approval-design.md`（8.14 KB）
  - `2026-07-22-batch-user-questions-persona-nudge-design.md`（4.46 KB，**Han=0**）
  - `2026-07-22-brainstorming-request-user-input-design.md`（6.15 KB，**Han=1**）
  - `2026-07-22-deepseek-skill-first-reminder-design.md`（6.01 KB，**Han=0**）
  - `2026-07-22-multi-question-request-user-input-design.md`（8.86 KB，**Han=0**）
  - `2026-07-22-request-user-input-custom-and-submit-spacing-design.md`（6.16 KB，**Han=3**）
  - `2026-07-24-retire-core-conversation-tui-port-design.md`（7.89 KB）
  - `2026-07-24-skills-multi-compose-design.md`（6.71 KB）
  - `2026-07-24-windows-native-tls-schannel-fallback-design.md`（8.22 KB）
  - `2026-07-25-retire-core-provider-A-compact-design.md`（5.42 KB）

#### D-12（11 个）
- **批次**：4 ｜ **依赖**：T-01 ｜ **复杂度**：M
- **files_owned**：`docs/superpowers/specs/` 下
  - `2026-07-25-retire-core-provider-B-vision-design.md`（8.43 KB）
  - `2026-07-25-retire-core-provider-C-conversation-design.md`（8.07 KB）
  - `2026-07-25-retire-core-tool-ball-D-design.md`（6.08 KB）
  - `2026-07-25-round-cap-checkpoint-design.md`（10.85 KB）
  - `2026-07-29-user-input-background-block-design.md`（5.41 KB）
  - `2026-07-29-webui-sync-compact-design.md`（5.83 KB）
  - `2026-07-30-progress-signposts-preamble-design.md`（6.64 KB）
  - `2026-07-30-workflow-intent-understanding-design.md`（6.98 KB）
  - `2026-07-31-local-scheduled-tasks-design.md`（7.54 KB）
  - `2026-07-31-local-scheduled-tasks-phase2-design.md`（8.04 KB）
  - `2026-07-31-project-memory-dir-override-design.md`（6.56 KB）

### 批次 5 — `docs/plans` 大件（33 个，≈232 KB）

> **无 YAML frontmatter**；代码块密度**中**；实测 24 个 `Han=0`（纯英文）。

#### D-13（2 个大件）
- **批次**：5 ｜ **依赖**：T-01 ｜ **复杂度**：L
- **files_owned**：
  - `docs/plans/2026-08-20-code-review-deep-mode-fanout-plan.md`（39.13 KB，**Han=0**）
  - `docs/plans/2026-08-20-code-review-deep-verify-phase2-plan.md`（28.48 KB，**Han=0**）

#### D-14（3 个，provider accounts 组）
- **批次**：5 ｜ **依赖**：T-01 ｜ **复杂度**：M
- **files_owned**：
  - `docs/plans/2026-07-26-provider-accounts-model-profiles-design.md`（19.52 KB，**Han=0**）
  - `docs/plans/2026-07-26-provider-accounts-model-profiles-plan.md`（13.82 KB，**Han=0**）
  - `docs/plans/2026-08-20-code-review-deep-mode-fanout-design.md`（12.05 KB，**Han=0**）

#### D-15（3 个，**实测已中文，预期 no-op / 极小改动**）
- **批次**：5 ｜ **依赖**：T-01 ｜ **复杂度**：S
- **files_owned**：
  - `docs/plans/2026-08-17-acp-sdk-2.0-protocol-roadmap.md`（38.02 KB，Han 260/268）
  - `docs/plans/2026-08-21-external-agent-subagent-drivers-spec.md`（17.66 KB，Han 171/201）
  - `docs/plans/2026-08-14-webui-browser-notification-design.md`（13.19 KB，Han 153/176）
- **特别注意**：这 3 个文件**已是中文**。只处理 `check` 报出的英文 offender 行（多为英文标题/术语行）；`en_ratio ≤ 0.05` 即记 `no-op`。

### 批次 6 — `docs/plans` 小件 + `docs` 根级大件

#### D-16（13 个）
- **批次**：6 ｜ **依赖**：T-01 ｜ **复杂度**：M
- **files_owned**：`docs/plans/` 下
  - `2026-07-28-provider-panel-ui-design.md`（6.83 KB）
  - `2026-08-23-deepseek-v4-flash-evaluation.md`（5.74 KB）
  - `2026-08-23-deepseek-v4-flash-evaluation-design.md`（5.23 KB）
  - `2026-07-28-rewind-implementation-plan.md`（4.92 KB）
  - `2026-07-28-rewind-design.md`（4.47 KB）
  - `2026-08-07-lightweight-lsp-phase-one.md`（4.20 KB）
  - `2026-07-26-model-cost-attribution.md`（4.07 KB）
  - `2026-08-17-project-input-history-design.md`（4.06 KB）
  - `2026-07-31-native-runtime-datalog.md`（3.74 KB）
  - `2026-07-24-busy-continue-fork.md`（3.65 KB）
  - `2026-08-02-session-recovery-design.md`（3.48 KB）
  - `2026-08-07-config-panel-implementation-plan.md`（3.33 KB）
  - `2026-08-10-first-stage-planning-quality-design.md`（3.30 KB）

#### D-17（12 个）
- **批次**：6 ｜ **依赖**：T-01 ｜ **复杂度**：M
- **files_owned**：`docs/plans/` 下
  - `2026-07-25-subtasks-footer-panel.md`（2.75 KB）
  - `2026-08-15-webui-blocking-interaction-dock.md`（2.71 KB）
  - `2026-07-23-windows-qr-rendering.md`（2.38 KB）
  - `2026-08-07-request-user-input-review-design.md`（2.33 KB）
  - `2026-07-26-atomgit-production-tools.md`（2.21 KB）
  - `2026-07-26-busy-continue-fork-gc.md`（2.18 KB）
  - `2026-08-06-internal-continuation-compaction-design.md`（2.10 KB）
  - `2026-08-07-tasks-long-line-rendering.md`（2.00 KB）
  - `2026-08-23-headless-eval-controls.md`（1.94 KB，已中文）
  - `2026-08-02-session-recovery-implementation-plan.md`（1.68 KB）
  - `2026-08-11-todo-agent-body-projection-design.md`（1.63 KB，已中文）
  - `2026-08-11-provider-form-horizontal-editing-design.md`（1.47 KB，已中文）

#### D-18 · `REFACTOR_DESIGN_PHASE1.md`（**单文件专任务**）
- **批次**：6 ｜ **依赖**：T-01 ｜ **复杂度**：L
- **files_owned**：`docs/REFACTOR_DESIGN_PHASE1.md`（44.26 KB，**实测 Han=1，纯英文**）
- **注意力**：无 frontmatter；代码块密度高；含大量 crate 名 / 旧 `rustcode` 历史引用（`AGENTS.md:7` 规定 `rustcode` 是历史名引用）—— **历史名引用保留原文，不改写为 `rustcode-*`**

### 批次 7 — `docs` 根级

> 全部**无 YAML frontmatter**；hook/mcp/webhook 系列**代码块密度高**（JSON/bash 样例多）。

#### D-19 · `phase1-refactor-design.md`（**单文件专任务**）
- **批次**：7 ｜ **依赖**：T-01 ｜ **复杂度**：M
- **files_owned**：`docs/phase1-refactor-design.md`（44.51 KB，Han 367，已中文为主）
- **注意力**：无 frontmatter；代码块密度高；已中文段落**不润色**，只清英文 offender

#### D-20（4 个）
- **批次**：7 ｜ **依赖**：T-01 ｜ **复杂度**：M
- **files_owned**：
  - `docs/compact-native-migration-retrospective.md`（21.06 KB）
  - `docs/multi-agent-collaboration-solution.md`（18.95 KB）
  - `docs/platform-neutralization.md`（16.07 KB）
  - `docs/HOOK_DOC_UPDATE_SPEC.md`（15.85 KB）

#### D-21（6 个）
- **批次**：7 ｜ **依赖**：T-01 ｜ **复杂度**：M
- **files_owned**：
  - `docs/hook-architecture.md`（15.66 KB）
  - `docs/phase2-subagent-status.md`（15.00 KB）
  - `docs/mcp.md`（13.71 KB）
  - `docs/mcp-rmcp-feasibility.md`（13.59 KB）
  - `docs/webhook-implementation-summary.md`（7.71 KB）
  - `docs/compact-durable-checkpoint-design.md`（7.39 KB）

### 批次 8 — `docs` 根级（续）+ `docs/archive` 超大 2 件

#### D-22（7 个）
- **批次**：8 ｜ **依赖**：T-01 ｜ **复杂度**：M
- **files_owned**：
  - `docs/async-webhook-guide.md`（11.00 KB）
  - `docs/agent-api-rfc.md`（11.44 KB）
  - `docs/webhook-guide.md`（10.44 KB）
  - `docs/hook-cli-guide.md`（10.03 KB）
  - `docs/architecture.md`（9.84 KB）
  - `docs/async-webhook-summary.md`（9.83 KB）
  - `docs/hook-expansion-summary.md`（9.11 KB）

#### D-23（15 个，小件集中）
- **批次**：8 ｜ **依赖**：T-01 ｜ **复杂度**：M
- **files_owned**：
  - `docs/hooks.md`（12.30 KB，Han 22，**混排**）
  - `docs/dev-env-setup.md`（7.91 KB）
  - `docs/hook-implementation-summary.md`（7.90 KB）
  - `docs/hook-timing-complete.md`（8.30 KB）
  - `docs/target-architecture.md`（6.05 KB）
  - `docs/i18n-field-mapping.md`（5.04 KB）
  - `docs/REFACTOR_SUMMARY.md`（4.79 KB）
  - `docs/i18n-style.md`（4.75 KB）
  - `docs/vscode-i18n-implementation-plan-2026-06-30.md`（4.71 KB）
  - `docs/features.md`（4.36 KB）
  - `docs/custom-endpoint-guide.md`（3.75 KB）
  - `docs/pr-hook-test-command.md`（3.62 KB）
  - `docs/codex-claude-config-analysis.md`（2.71 KB）
  - `docs/telemetry.md`（2.26 KB）
  - `docs/acp-sdk-handler-notes.md`（1.44 KB）

#### D-24（2 个超大件）
- **批次**：8 ｜ **依赖**：T-01 ｜ **复杂度**：L
- **files_owned**：
  - `docs/archive/coding-runtime-incremental-migration.md`（56.26 KB，Han 760，已中文为主）
  - `docs/archive/session-convergence-plan.md`（55.62 KB，Han 596，已中文为主）
- **注意力**：无 frontmatter；代码块密度**高**；已中文**不润色**，只清英文 offender

### 批次 9 — `docs/archive` 余件 + `.codebuddy/agents`

#### D-25（1 个超大件）
- **批次**：9 ｜ **依赖**：T-01 ｜ **复杂度**：M
- **files_owned**：`docs/archive/coding-runtime-native-migration-design.md`（45.62 KB，Han 480）

#### D-26（15 个：archive 余 + adr + security + mcp + testing + pr-descriptions）
- **批次**：9 ｜ **依赖**：T-01 ｜ **复杂度**：M
- **files_owned**：
  - `docs/testing/release-v5.0.0-acceptance.md`（16.23 KB）
  - `docs/archive/release-v5.0.3-core-retirement-acceptance.md`（13.97 KB）
  - `docs/archive/release-v5.0.1-current-branch-change-report.md`（11.76 KB）
  - `docs/archive/kernel-parity-backlog.md`（11.23 KB，Han 6，**混排**）
  - `docs/archive/live-transport-convergence-plan.md`（11.11 KB）
  - `docs/archive/v5.0.0-retire-bridge-core-progress.md`（10.69 KB）
  - `docs/security/permission-model.md`（9.17 KB，**Han=0，纯英文**）
  - `docs/pr-descriptions/2026-04-27-vscode-extension-ui-mcp.md`（7.15 KB）
  - `docs/mcp/github.md`（5.19 KB）
  - `docs/testing/windows-path-normalization.md`（4.48 KB）
  - `docs/archive/2026-07-25-provider-retry-consolidation.md`（3.96 KB）
  - `docs/adr/0002-runtime-owned-dynamic-mcp-tool-catalog.md`（3.60 KB）
  - `docs/archive/2026-07-27-models-dev-pricing-design.md`（2.28 KB）
  - `docs/adr/0003-runtime-owned-turn-execution-policy.md`（2.04 KB）
  - `docs/adr/0001-runtime-owned-session-transitions.md`（0.99 KB，**Han=2**）

#### D-27 · `.codebuddy/agents` + `.codebuddy/rules`（8 个）
- **批次**：9 ｜ **依赖**：T-01 ｜ **复杂度**：M
- **files_owned**：
  - `.codebuddy/agents/code-implementer.md`（4.96 KB）
  - `.codebuddy/agents/code-reviewer.md`（5.63 KB）
  - `.codebuddy/agents/doc-writer.md`（4.29 KB）
  - `.codebuddy/agents/project-manager.md`（6.58 KB）
  - `.codebuddy/agents/requirements-analyst.md`（4.77 KB）
  - `.codebuddy/agents/solution-architect.md`（5.83 KB）
  - `.codebuddy/agents/test-engineer.md`（5.06 KB）
  - `.codebuddy/rules/multi-agent-workflow.md`（3.97 KB）
- **⛔ 含 YAML frontmatter（全部 8 个）**：键名 `kind/id/from/to/feature/status/decision/requires/created` **一律保留英文**。
- **`description` 字段**：`requirements-analyst.md` 的 `description` **已是中文**（先例）；其余按 Q5 可汉化，但**必须在报告中逐条列出**（AC-6）。
- **代码块密度**：低。

### 批次 10 — `.codebuddy/artifacts` A

> **多数含 YAML frontmatter**（`00-` / `01-` / `02-` / `04-` / `06-` 系列与部分 `03-impl`；`STATUS.md` 多数无）。
> **实测已基本是中文**（残余英文约 900 行级别，多为英文标题与 `AC-xx` 条目首词）→ 预期大量 `no-op`。

#### D-28（19 个：`2026-09-02-cleanup-codingplan-legacy` 全目录）
- **批次**：10 ｜ **依赖**：T-01 ｜ **复杂度**：M
- **files_owned**：`.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/` 下
  - `00-requirement.md`（47.91 KB）、`01-design.md`（38.21 KB）、`02-tasks.md`（29.91 KB）、`06-release.md`（11.56 KB）、`STATUS.md`（11.83 KB）
  - `03-impl/T1.md`（4.36）、`03-impl/T3.md`（3.35）、`03-impl/T4.md`（1.77）、`03-impl/T5.md`（2.78）、`03-impl/T6.md`（1.95）、`03-impl/T7-dead-code-scan.md`（4.85）、`03-impl/T9-integration-verification.md`（7.87）
  - `04-review/T1.md`（3.54）、`04-review/T3.md`（1.63）、`04-review/T4.md`（1.07）、`04-review/T5.md`（0.98）、`04-review/T6.md`（0.86）、`04-review/T7.md`（1.23）、`04-review/T9.md`（1.77）

#### D-29（4 个：`g1-fmt-gate` 大件）
- **批次**：10 ｜ **依赖**：T-01 ｜ **复杂度**：M
- **files_owned**：`.codebuddy/artifacts/2026-09-02-g1-fmt-gate/` 下
  - `03-impl/T7-goals-superpowers.md`（26.19 KB）
  - `06-release.md`（22.53 KB）
  - `STATUS.md`（20.68 KB）
  - `05-test-report.md`（19.67 KB）

#### D-30（3 个：`g1-fmt-gate` 余件）
- **批次**：10 ｜ **依赖**：T-01 ｜ **复杂度**：S
- **files_owned**：`.codebuddy/artifacts/2026-09-02-g1-fmt-gate/` 下
  - `03-impl/T6-docs-atomcode.md`（17.72 KB）
  - `03-impl/T8-artifacts.md`（7.44 KB）
  - `HANDOFF-codingplan-legacy.md`（5.68 KB）

### 批次 11 — `.codebuddy/artifacts` B + crates 文档 + 扩展/前端文档

#### D-31（11 个）
- **批次**：11 ｜ **依赖**：T-01 ｜ **复杂度**：M
- **files_owned**：
  - `.codebuddy/artifacts/2026-09-03-git-wrapup/01-plan.md`（28.19 KB）
  - `.codebuddy/artifacts/2026-09-03-git-wrapup/02-tasks.md`（21.27 KB）
  - `.codebuddy/artifacts/2026-09-03-banner-release/STATUS.md`（19.73 KB）
  - `.codebuddy/artifacts/2026-09-03-git-wrapup/STATUS.md`（11.49 KB）
  - `.codebuddy/artifacts/2026-09-03-git-wrapup/03-impl/GW-02.md`（11.86 KB）
  - `.codebuddy/artifacts/2026-09-03-git-wrapup/03-impl/GW-03.md`（8.36 KB）
  - `.codebuddy/artifacts/2026-09-03-banner-release/03-impl/T2.md`（8.16 KB）
  - `.codebuddy/artifacts/2026-09-03-banner-release/03-impl/T1.md`（6.58 KB）
  - `.codebuddy/artifacts/2026-09-03-residual-two-items/STATUS.md`（5.70 KB）
  - `.codebuddy/artifacts/2026-09-03-doc-consistency/STATUS.md`（4.95 KB）
  - `.codebuddy/artifacts/2026-09-02-strip-atomcode/STATUS.md`（3.68 KB）

#### D-32 · crate 文档（9 个，非 rules / 非 seeds）
- **批次**：11 ｜ **依赖**：T-01 ｜ **复杂度**：M
- **files_owned**：
  - `crates/rustcode-daemon/README.md`（16.09 KB，**混排** Han 174/415）
  - `crates/rustcode-clix/README.md`（10.09 KB）
  - `crates/rustcode-kernel/SPIKE.md`（6.11 KB，**Han=0**）
  - `crates/rustcode-review/LANGUAGES.md`（6.53 KB，**Han=1**）
  - `crates/rustcode-review/README.md`（3.83 KB）
  - `crates/rustcode-coding/README.md`（3.59 KB）
  - `crates/rustcode-capabilities/README.md`（3.58 KB）
  - `crates/rustcode-tuix/tests/smoke.md`（3.08 KB，**Han=1**）
  - `crates/rustcode-kernel/README.md`（1.79 KB）
- **注意力**：**无 YAML frontmatter**；代码块/表格密度**中-高**；crate 名、命令、配置键一律保留
- **⛔ 严禁触碰** `crates/rustcode-review/rules/**`（Q1）

#### D-33 · 扩展 / 前端 / 容器 / npm 包文档（9 个）
- **批次**：11 ｜ **依赖**：T-01 ｜ **复杂度**：M
- **files_owned**：
  - `extensions/jetbrains/README.md`（14.63 KB）
  - `extensions/vscode/README.md`（5.47 KB）
  - `extensions/jetbrains/docs/jetbrains.md`（4.74 KB）
  - `extensions/jetbrains/PRIVACY.md`（3.31 KB）
  - `.claude/plans/atomgit-decouple.md`（4.44 KB）
  - `.superpowers/pr/feat-rust-tui-selection-session-preview.md`（5.04 KB）
  - `docker/README.md`（7.24 KB）
  - `webui/README.md`（2.00 KB）
  - `packages/npm/README.md`（1.33 KB）
- **⛔ 不改** `extensions/jetbrains/CHANGELOG.md`（C 类逐字历史）
- **注意力**：无 YAML frontmatter；代码块密度中

### 批次 12 — 运行时载荷（**单独一批，owner = `doc-writer`，需 PM 单独验收**）

#### D-34 · setup-seeds 6 件（**运行时载荷**）
- **批次**：12（独占）｜ **依赖**：T-01 ｜ **复杂度**：M
- **files_owned**：`crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/` 下
  - `SKILL.md`（14.33 KB）
  - `references/hooks-patterns.md`（5.64 KB）
  - `references/mcp-servers.md`（7.27 KB）
  - `references/plugins-reference.md`（3.01 KB）
  - `references/skills-reference.md`（9.79 KB）
  - `references/subagent-templates.md`（4.90 KB）
- **注意力**：
  - `SKILL.md` **含 YAML frontmatter**：`name: setup` **必须保留英文**（skill 身份/目录名）；`description` **按 Q5 汉化**，并在报告逐条列出
  - 代码块密度**高**（YAML/JSON/bash 示例）；`references/mcp-servers.md` 里的 `@sentry/*` 是**已复核的假阳性**（`AGENTS.md:255,261`），**保留不动**
- **验收（额外，AC-5）**：
  - `git diff "$ZH_BASE" -- crates/rustcode-review/rules/` **为空**
  - `git diff "$ZH_BASE" -- crates/rustcode-capabilities/assets/setup-seeds/` **非空**
  - `git diff -U0 "$ZH_BASE" -- crates/rustcode-capabilities/assets/setup-seeds/ | grep -E '^[+-]name:'` **无输出**
- **影响**：改动会改 `SEEDS_TARZST` 内容哈希（`seeds.rs:8,19`）⇒ 已安装用户触发一次种子重装，可能覆盖其手工修改 —— **必须在批次报告中写明**，供 release 说明引用

### 批次 13 — 根 README 与零星

#### D-35 · 根 `README.md` 汉化 + 删除 `README.zh-CN.md`
- **批次**：13 ｜ **依赖**：T-01、T-03（需与新默认 host 一致）｜ **复杂度**：L
- **files_owned**：`README.md`（39.51 KB，**实测 Han=1，纯英文**）、`README.zh-CN.md`（37.88 KB，**删除**）
- **改动要点**：
  1. 用 `README.zh-CN.md` 的中文内容汉化 `README.md`（纯中文替换，不并列，Q4）
  2. `git rm README.zh-CN.md`
  3. `:16` 的 `<a href="./README.zh-CN.md">简体中文</a>` 导航**移除**（Q4：不保留英文入口，不引入 `example.com` 死链）
  4. 默认绑定相关叙述改为 `0.0.0.0`，并写明 **TUI `/webui` 默认仍为 `127.0.0.1`，跨设备用 `/webui --host 0.0.0.0`**（O-1）
  5. 前端构建步骤改为 `./scripts/build-webui.sh`（对齐 AC-17）
- **注意力**：**无 YAML frontmatter**；代码块密度**高**；**徽章/HTML/`<img src>` 一律保留原样**
- **验收**：AC-9、AC-10、AC-11（所有非外链 `href` 目标 `test -e` 通过）、AC-2、AC-4、AC-7b、AC-32

#### D-36 · 零星（9 个）
- **批次**：13 ｜ **依赖**：T-01 ｜ **复杂度**：S
- **files_owned**：
  - `evals/deepseek-v4-flash/README.md`（1.54 KB）
  - `evals/deepseek-v4-flash/prompts/codex-judge.md`（0.65 KB）
  - `evals/deepseek-v4-flash/prompts/codex-report.md`（0.92 KB）
  - `evals/deepseek-v4-flash/cases/agent-fixture/ARCHITECTURE.md`（0.25 KB）
  - `.goals/rustcode-migration-finalize/goal.md`（3.57 KB）
  - `.goals/rustcode-migration-finalize/inspector-feedback-1.md`（2.15 KB）
  - `.goals/rustcode-migration-finalize/summary.md`（2.55 KB）
  - `CONTEXT.md`（3.42 KB）
  - `DEVENV.md`（1.56 KB）
- **注意力**：无 YAML frontmatter；代码块密度低。`CONTEXT.md` 的运行时领域术语（Live View / Runtime Generation / …）按 `AGENTS.md:55` **保留英文原词**，只译说明文字

### 批次 14 — 默认 host 文档同步（owner = `doc-writer`）

#### T-06 · 清除 md 中与新默认矛盾的 `127.0.0.1` 叙述
- **批次**：14 ｜ **依赖**：批次 1–13 全部完成、T-03 ｜ **复杂度**：M
- **files_owned**：`python3 scripts/check-zh-docs.py hostscan --base "$ZH_BASE"` 输出的文件列表，**排除** `README.md`（已由 D-35 处理）
- **目标**：默认改为 `0.0.0.0` 后，文档中"默认 127.0.0.1"的叙述会与行为矛盾（`00-requirement.md:395`）
- **规则**：
  - 作为**IP 字面量**出现在 inline code 中的 `127.0.0.1`（如"显式 `--host 127.0.0.1`"）**保留**
  - 仅改"默认值 = 127.0.0.1"这类**叙述**
- **验收**：AC-2、AC-4、AC-7b、AC-32；`grep -rn "default.*127\.0\.0\.1\|默认.*127\.0\.0\.1" --include=*.md .` 仅剩"显式指定"语境
- **验证命令**：
  ```bash
  python3 scripts/check-zh-docs.py hostscan --base "$ZH_BASE"
  python3 scripts/check-zh-docs.py check --base "$ZH_BASE" --diff
  ```

### 批次 15 — 集成验收（owner = `test-engineer`，`code-implementer` 配合）

#### T-07 · 全量门禁 + 人工/冒烟验收
- **批次**：15 ｜ **依赖**：T-01 … T-06 全部完成 ｜ **复杂度**：L
- **files_owned**：`.codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/**`（报告输出）、`.codebuddy/artifacts/2026-09-07-zh-docs-webui/06-release.md`（待 PM 分配）
- **A. 脚本门禁（必跑，exit 0 才算过）**：
  ```bash
  python3 scripts/check-zh-docs.py gate --base "$ZH_BASE" \
    --report .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-gate.md
  ```
  覆盖 AC-1 / 2 / 4 / 5 / 6 / 7 / 8 / 32。
- **B. 人工机械判定（无脚本，二值）**：
  - AC-3：从各批 `zh-check-*.md` 汇总残留行，抽检 20 行（不足 20 全检），100% 落入 §9.1 白名单
  - AC-8：`grep -rn "README\.zh-CN" --include=*.md --include=*.html --include=*.json --include=*.ts --include=*.kt --include=*.yml --include=*.toml .` 与 `grep -rn "README\.zh-CN" site/ .github/ docs/ extensions/` 均 **0 命中**
  - AC-9/10/11：README 三项 + `README.md` 的 AC-2 + 非外链 `href` 的 `test -e`
- **C. Rust 门禁（AC-31）**：
  ```bash
  cargo fmt --check
  cargo clippy --workspace --all-targets 2>&1 | tail -n 40     # 无新增告警（存量 ~420 豁免）
  cargo test --workspace --no-fail-fast 2>&1 | tail -n 60      # 唯一允许红测：mcp::registry::tests::trust_key_golden_matches_core_algorithm
  cargo test -p rustcode-config --lib                          # i18n 内容测试全量（AGENTS.md:263）
  cargo test -p rustcode-daemon --lib channel_mode_tests       # AC-29 保持通过
  ```
- **D. WebUI 构建与端到端（AC-12/13/14/15/16/17/18/22/23/24/25/26）**：
  ```bash
  bash -n scripts/build-webui.sh
  ./scripts/build-webui.sh && cargo clean -p rustcode-daemon && cargo build
  RUSTCODE_HOME=$(mktemp -d) cargo run -p rustcode -- webui     # 无 provider 也不得退出（AC-24）
  curl -sSf http://127.0.0.1:13457/health                       # 200
  curl -sSf http://127.0.0.1:13457/ | grep -c '<html'           # >=1
  # AC-22/23：局域网另一台主机对 <LAN-IP>:<port> 重复上述两项 + 浏览器同源 fetch 无 CORS 错误
  # AC-25/26：浏览器打开设置对话框，新建 provider → 设默认 → 校验 config.toml 与 GET /providers
  ```
- **E. 审批回归闸门（AC-27/28/29/30，最高优先级，不得跳过）**：
  - 独立 `rustcode-daemon` 二进制以 `--port 13456 --client vscode`（**不传 `--host`**）启动，带 `X-RustCode-Client: vscode` 在 Build 模式发 `POST /chat` 触发需审批工具调用
  - **判定不通过** = 工具未获批准即自动执行（`dangerously_skip_permissions=true` 且无审批事件）
  - AC-29：`channel_mode_tests::known_clients_interactive_on_loopback_or_token` 保持通过且断言无净减少
  - AC-30：`cargo test -p rustcode-daemon` 中出现"独立 daemon 默认 host 常量"锁定测试（**若 T-03 未新增，需在此补齐并回退给 `code-implementer`**）
- **F. 前端回归（AC-33）**：
  ```bash
  cd webui && npx tsc --noEmit && npm test && npm run build
  ```

### 批次 16 — D-5：`rustcode daemon` 补 `--host` 参数

> 上游契约：`01-design-addendum-d5.md`（DESIGN-002，已冻结，**实现方不得偏离**）。
> 本批修 G5 登记的缺陷 **D-5**（`STATUS.md:115`）。**不得**回退 Q2-A 的 `0.0.0.0` 默认值。
>
> **硬耦合（必读）**：`clap` 的 `Command::mut_arg` 在 arg id 不存在时**无条件 panic**
> （`clap_builder-4.6.0/src/builder/command.rs:244-254`：
> `.unwrap_or_else(|| panic!("Argument `{id}` is undefined"))`）。
> 故「加 derive 字段」与「加 `mut_arg("host", …)`」**必须在同一次改动、同一个 commit 内一起落地**，
> 否则 `rustcode --help` / `rustcode <任一子命令> --help` / `rustcode completion <shell>` 会直接 panic。

#### T-15 · `rustcode daemon` 补 `--host` 参数（缺陷 D-5）
- **批次**：16（独占）｜ **依赖**：无（不依赖任何汉化批；但**不得**与 T-03 并发改 `crates/rustcode-cli/src/main.rs`）｜ **crate**：`rustcode-cli` ｜ **复杂度**：S
- **owner（代码）**：`code-implementer`
- **owner（文档 / 看板）**：`project-manager`（`AGENTS.md` 与 `STATUS.md` 按 §0.3 规则 4 不归 `code-implementer`）
- **目标**：让 `rustcode daemon` 入口的用户能显式退回 `127.0.0.1`，同时保持默认值 `0.0.0.0`（Q2-A 不回退）。
- **files_owned（代码，`code-implementer` 独占）**：
  - `crates/rustcode-cli/src/main.rs`
- **files_owned（文档 / 看板，`project-manager` 独占，限指定行）**：
  - `AGENTS.md`（**仅第 37 行**）
  - `.codebuddy/artifacts/2026-09-07-zh-docs-webui/STATUS.md`（**仅第 115 行** D-5 行、**第 139 行**下轮建议第 2 条）
- **改动（3 处，精确代码见 `01-design-addendum-d5.md §4.2`）**：
  1. `main.rs:1026-1038` `Commands::Daemon`：在 `port` 字段之后插入
     ```rust
        /// Bind address (default 0.0.0.0: reachable over LAN; use 127.0.0.1 to
        /// restrict to this machine. Token-protected only, with no TLS)
        #[arg(long, default_value = "0.0.0.0")]
        host: String,
     ```
     doc comment 与 `main.rs:1045-1046`（`webui` 同名字段）**逐字相同**。
  2. `main.rs:467-473` 的 `.mut_subcommand("daemon", …)` 闭包内追加
     ```rust
            .mut_arg("host", |a| a.help(t(Msg::CliHelpHost).into_owned()))
     ```
     **复用**既有变体（`messages.rs:3916` / `en.rs:2521` / `zh_cn.rs:2429`），**不新增 `Msg`**。
  3. `main.rs:1738-1742` 解构加 `host`；`main.rs:1779-1790` 的 `ServerOpts { host: "0.0.0.0".to_string(), … }` 改为 `host,`。
- **不得改动（越界即 FAIL）**：
  - `crates/rustcode-daemon/**`（含 `main.rs:21` 的 `DEFAULT_HOST = "127.0.0.1"`，`AGENTS.md:38` 明令不得统一）
  - `crates/rustcode-config/src/i18n/**`（复用 `Msg::CliHelpHost`，不新增变体）
  - `ServerOpts`、`run_server`、`is_loopback_bind_host`、`is_loopback_authority`、`ensure_server_and_open` 全部**一字不动**
  - `extensions/**`（本轮只预留，落点见设计 §9 I-6）
  - `docs/**`、`README.md`（已核：无"daemon 无 `--host`"表述；`README.md:120` 改后即为真）
  - **不得**复活 `Msg::DaemonWarnNonLoopback`、**不得**改动非回环提示判据
- **验收标准**：AC-D5-1 … AC-D5-6（`01-design-addendum-d5.md §10`），逐条如下：
  - **AC-D5-1**：`rustcode daemon --help` 含 `--host <HOST>`；默认 locale 下含 `绑定地址（默认：0.0.0.0；改 127.0.0.1 则仅本机可访问）`；`rustcode --lang en daemon --help` 含 `Bind address (default: 0.0.0.0; use 127.0.0.1 for local-only)`；且与 `rustcode webui --help` 的 `--host` 行 help 文本**逐字相等**
  - **AC-D5-2**：不传 `--host` 时 stdout 含 `监听地址 http://0.0.0.0:<PORT>` 且**打印** `主地址为局域网 IP`（与基线等价）
  - **AC-D5-3**：`--host 127.0.0.1` 时 stdout 含 `监听地址 http://127.0.0.1:<PORT>`，且**不含** `主地址为局域网 IP` / `已绑定非回环地址`；`curl -sSf http://127.0.0.1:<PORT>/health` = 200
  - **AC-D5-4**：`rustcode daemon --host`（缺值）⇒ 退出码 **2**
  - **AC-D5-5**：`--host ""`、`--host 1.2.3.4:80` ⇒ 退出码 **1**，stderr 含 `致命错误：无法绑定到`，且 `$RUSTCODE_HOME/daemon-<PORT>.json` **不存在**
  - **AC-D5-6**：`cargo fmt --check`；`cargo clippy --workspace --all-targets` 无新增告警；`cargo test -p rustcode-cli --test shell_completion`、`cargo test -p rustcode-config --lib`、`cargo test -p rustcode-daemon --lib channel_mode_tests` 均通过；`python3 scripts/check-zh-docs.py gate --base 3ee655e3` 仍 exit 0
- **验证命令**：
  ```bash
  # 0) 环境隔离
  export RUSTCODE_HOME=$(mktemp -d)
  cargo build -p rustcode
  BIN=./target/debug/rustcode

  # 1) AC-D5-1：help 两语种 + 与 webui 一致
  $BIN daemon --help | grep -F -- '--host <HOST>'
  $BIN daemon --help | grep -F '绑定地址（默认：0.0.0.0；改 127.0.0.1 则仅本机可访问）'
  $BIN --lang en daemon --help | grep -F 'Bind address (default: 0.0.0.0; use 127.0.0.1 for local-only)'
  diff <($BIN daemon --help | grep -A1 -- '--host') <($BIN webui --help | grep -A1 -- '--host')   # 应无差异
  # 防 panic（mut_arg 硬耦合）：以下三条都不得 panic
  $BIN --help >/dev/null; $BIN completion bash >/dev/null; $BIN daemon --help >/dev/null

  # 2) AC-D5-2：默认值不回退
  ($BIN daemon --port 13456 > /tmp/d5-default.log 2>&1 &) ; sleep 3
  grep -F '监听地址 http://0.0.0.0:13456' /tmp/d5-default.log
  grep -F '主地址为局域网 IP' /tmp/d5-default.log
  pkill -f 'rustcode daemon' || true

  # 3) AC-D5-3：显式回环生效且不告警
  ($BIN daemon --host 127.0.0.1 --port 13456 > /tmp/d5-lo.log 2>&1 &) ; sleep 3
  grep -F '监听地址 http://127.0.0.1:13456' /tmp/d5-lo.log
  ! grep -F '主地址为局域网 IP' /tmp/d5-lo.log && ! grep -F '已绑定非回环地址' /tmp/d5-lo.log
  curl -sSf http://127.0.0.1:13456/health      # 期望 200
  pkill -f 'rustcode daemon' || true

  # 4) AC-D5-4：解析期拒绝缺值
  $BIN daemon --host; echo "exit=$?"           # 期望 2

  # 5) AC-D5-5：运行期 fail-closed
  $BIN daemon --host '' --port 13456; echo "exit=$?"          # 期望 1，stderr 含「致命错误：无法绑定到」
  $BIN daemon --host 1.2.3.4:80 --port 13456; echo "exit=$?"  # 期望 1
  test ! -e "$RUSTCODE_HOME/daemon-13456.json"                 # 无 token 文件

  # 6) AC-D5-6：门禁与既有测试
  cargo fmt --check
  cargo clippy --workspace --all-targets 2>&1 | tail -n 40
  cargo test -p rustcode-cli --test shell_completion 2>&1 | tail -n 10
  cargo test -p rustcode-config --lib 2>&1 | tail -n 10
  cargo test -p rustcode-daemon --lib channel_mode_tests 2>&1 | tail -n 10
  python3 scripts/check-zh-docs.py gate --base 3ee655e3
  ```
- **回退**：`git revert` 本任务 commit 即可；手工回退时必须**三处同时**撤回（见设计 §7）。
- **报告**：实现与验收记录写入 `.codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/T-15.md`（`code-implementer` 写，属其 `files_owned` 扩展，需 PM 追加授权）。

---

## 2. 并行批次总览

| 批次 | 任务 | `files_owned` 不相交校验 | 可并行 |
|---|---|---|---|
| **0A** | T-01（`scripts/check-zh-docs.py` + `03-impl/md-inventory.md`）／T-02（`scripts/build-webui.sh`）／T-03（`crates/rustcode-cli/src/main.rs`） | ✅ 三者路径完全不同；且均为 `scripts/**` 或 `*.rs`，与所有 `*.md` 汉化任务不相交 | 是（3 个） |
| **0B** | T-04（`crates/rustcode-daemon/src/lib.rs`）／T-05（`crates/rustcode-config/src/i18n/{zh_cn,en}.rs`） | ✅ 分属不同 crate | 是（2 个） |
| **1** | D-01／D-02／D-03 | ✅ 逐路径不相交 | 是（3） |
| **2** | D-04／D-05／D-06 | ✅ | 是（3） |
| **3** | D-07／D-08／D-09 | ✅（D-09 含 `docs/superpowers/` 根 1 件，其余任务无） | 是（3） |
| **4** | D-10／D-11／D-12 | ✅ specs 33 个三分，无重叠 | 是（3） |
| **5** | D-13／D-14／D-15 | ✅ | 是（3） |
| **6** | D-16／D-17／D-18 | ✅ | 是（3） |
| **7** | D-19／D-20／D-21 | ✅ | 是（3） |
| **8** | D-22／D-23／D-24 | ✅ | 是（3） |
| **9** | D-25／D-26／D-27 | ✅（D-26 在 `docs/**`，D-27 在 `.codebuddy/**`） | 是（3） |
| **10** | D-28／D-29／D-30 | ✅（三条不同子目录/不同文件） | 是（3） |
| **11** | D-31／D-32／D-33 | ✅（`.codebuddy/artifacts` ／ `crates/**` ／ `extensions,webui,docker,packages,.claude,.superpowers`） | 是（3） |
| **12** | D-34（独占） | ✅ 运行时载荷单独一批 | 否（1） |
| **13** | D-35／D-36 | ✅ | 是（2） |
| **14** | T-06（独占） | 依赖全部汉化批完成 | 否（1） |
| **15** | T-07（独占） | 依赖 T-01…T-06 | 否（1） |
| **16** | T-15（独占，D-5 补 `--host`） | 代码面仅 `crates/rustcode-cli/src/main.rs`；文档面仅 `AGENTS.md:37` 与 `STATUS.md:115/139`。**与所有 `*.md` 汉化批不相交**（汉化批已完结），与批次 0A–15 亦无重叠 | 否（1） |

**跨批次并行**：0A / 0B 与批次 1–13 **可同时进行**（文件集合不相交）。
**跨 crate 改动**：T-03 / T-04 / T-05 分属 `rustcode-cli` / `rustcode-daemon` / `rustcode-config`，无共享文件，故未强制串行；若实现期发现 i18n 与 CLI 文案存在交叉引用，由 PM 决定将 T-05 提前。
**T-15 的串行约束**：`crates/rustcode-cli/src/main.rs` 与 T-03 是同一文件，二者**必须串行**（先 T-03 后 T-15）；批次 16 排在最后一批，天然满足。

---

## 3. 集成顺序说明

```
1) T-01 先落地（阻塞所有汉化批）—— 没有验收脚本的汉化批不得开工。
2) T-02 / T-03 / T-04 / T-05 与批次 1 同时启动（不相交）。
3) 批次 1 → 13 顺序推进；每批完成后立刻跑本批 check（不等集成），
   FAIL 只回退本批的 files_owned，不动他批。
4) D-34（setup-seeds）单独一批：改动前记录 $RUSTCODE_HOME 现状，
   改动后确认 AC-5(b)（非空 diff 且无 ^[+-]name:）。
5) D-35（README）在 T-03 之后（需与 0.0.0.0 默认一致），且在 D-36 同批内完成；
   删除 README.zh-CN.md 后立即跑 AC-8 的两次 grep（0 命中）。
6) T-06 在所有汉化批之后：hostscan 只扫"叙述性 127.0.0.1"，不动 IP 字面量。
7) T-07 集成：先 gate（脚本），再 AC-3 人工抽检，再 cargo 三门禁，
   最后 AC-18/22/23/24/25/26 的端到端与 AC-27/28/29/30 的审批闸门。
8) 合并顺序建议：0A → 0B →（汉化批按 1..13 逐批 merge）→ 14 → 15。
   每个汉化批可独立成一个 commit，便于 §7.2 的按批回退。
9) T-15（D-5，批次 16）在 T-03 之后、且在 T-07 的 cargo 门禁之后再落地：
   - 代码面（`crates/rustcode-cli/src/main.rs`）与 T-03 同文件 ⇒ 必须串行，不得并发；
   - 三处改动（derive 字段 / mut_arg / 分支传参）必须在**同一个 commit** 内，
     否则 `mut_arg` 会 panic（见批次 16 的「硬耦合」说明）；
   - 随后由 PM 同步 `AGENTS.md:37` 与 `STATUS.md:115/139`；
   - 最后补跑一次 `python3 scripts/check-zh-docs.py gate --base 3ee655e3`
     与 `cargo test -p rustcode-cli --test shell_completion` 收尾。
```
