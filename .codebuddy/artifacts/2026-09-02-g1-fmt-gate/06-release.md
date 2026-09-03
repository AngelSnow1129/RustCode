---
kind: release
id: RELEASE-002
from: doc-writer
to: [project-manager]
feature: 2026-09-02-g1-fmt-gate
status: done
decision: proceed
requires: [TEST-001]
files_owned:
  - .codebuddy/artifacts/2026-09-02-g1-fmt-gate/05-test-report.md
  - .codebuddy/artifacts/2026-09-02-g1-fmt-gate/06-release.md
  - .codebuddy/artifacts/2026-09-02-g1-fmt-gate/03-impl/T8-artifacts.md
created: 2026-09-02
---

# RELEASE-002 交付说明 — G1 格式门禁归零 + 7 处 locale 缺锁修复

- 基线：branch=dev commit=8e772dbf，worktree=**dirty**（29 个改动文件，含上一轮遗留与并发会话改动）
- 前置依赖：`TEST-001`（`05-test-report.md`，status=done / decision=proceed）
- 门禁：G1 pass / 编译检查 pass / G3 pass（1 个文档化已知红）/ G6 pass / G7 pass / G8 pass / G9 pass

## 0. 覆盖范围与数据来源（先读）

- [INFO] 本件覆盖 **A 组（G1 格式归零，10 文件 / 113 行）** 与 **B 组（locale 锁修复，7 处 / 3 文件）**，
  数据全部来自编排者实测，**本件未执行任何构建 / 测试 / git 命令**，未推断、未改写数字。
- [INFO] 同 feature 另有一份独立交付件 **`03-impl/T6-docs-atomcode.md`（RELEASE-001）**，
  覆盖 4 份用户可见文档的 `atomcode` 残留清理（README 中英、`docs/features.md`、
  `docs/platform-neutralization.md`）。**该件已自带四段式清单与回滚方案，本件不重复展开、不重复验证**，
  仅在文档更新清单中登记交叉引用。
- [CHECK] 本件作者对**实现状态**与**文档引用**做了只读复核（Grep/Read），结论见第 7 节与第 5 节。
- [WARN] 本件作者无 shell 工具，无法复核 `git diff --stat` / `git status`（编排者已补齐）。

---

## 1. 行为变化（Behavior Changes）

### 1.1 面向最终用户：**无**

| 组 | 改动性质 | 用户可感知变化 |
|---|---|---|
| A 组（10 文件 / 113 行） | 纯格式：空白、换行、尾随逗号增删、rustfmt `merge_derives` 合并 `#[derive]` | **无**。经 G9 证明零语义变化（6 文件剥离全空白后字符序列完全一致；4 文件差异均为零语义） |
| B 组（7 处 / 3 文件） | **仅 `#[cfg(test)]` 测试函数内新增 locale 锁**，不含任何产品代码路径 | **无**。产品运行时行为、CLI/TUI 输出、配置、协议均未变 |

### 1.2 面向贡献者与 CI：**有，且是本次交付的价值所在**

| 项 | 前 | 后 |
|---|---|---|
| G1 `cargo fmt --check` | exit=1，**19 处差异**（cli / coding / tuix / updater 四包 10 文件） | **exit=0 / 0 处差异** |
| CI（`AGENTS.md:220`，`.github/workflows/ci.yml` 的 fmt job） | 红 | 绿 |
| 7 个断言英文却未钉 locale 的测试 | 在默认 `Locale::ZhCn` 下**确定性失败**（例：实际 `"评审 · thinking"` vs 期望 `"review · thinking"`） | 持锁后**确定性转绿** |
| 全工作区存量红 | 8 | **1**（净减 7，零新增） |

### 1.3 迁移步骤

**无。** 本次交付不新增、不删除、不退役任何接口、CLI 子命令、配置项、环境变量或协议字段，
因此**不存在需要替代方案的退役接口**，调用方与用户无需任何动作。

### 1.4 上一 feature 成果（仅作上下文，不重复展开）

OpenRouter app 归因头改为 opt-in 默认关闭（`RUSTCODE_OPENROUTER_ATTRIBUTION`），
默认不再向第三方模型厂商外发产品身份；遥测词义残留（hook seam 注释里的 `telemetry`）清零。
该成果记录在 `docs/REFACTOR_DESIGN_PHASE1.md:357`（G6 行标注 [DONE]，**仍 dirty 未提交**）。

---

## 2. 风险（Risks）

| # | 类别 | 评估 | 依据 / 说明 |
|---|---|---|---|
| R1 | 不兼容 | **无** | 零语义变化；B 组仅测试内加锁 |
| R2 | 性能 | **无** | 不涉及生产代码路径 |
| R3 | 数据影响 | **无** | 不触碰 `~/.rustcode` 配置、会话、凭据、持久化格式 |
| R4 | 测试语义 | **低** | 7 处修复**未改任何断言**，只钉住 locale；不存在「削弱断言凑绿」 |
| R5 | **[WARN] 并发会话冲突（最高风险项）** | **中高** | 见 2.1 |
| R6 | 提交/回滚代价 | **低但非零** | 改动混在同一 dirty worktree，且 `crates/rustcode-cli/src/main.rs` 与并发会话共享，见 2.1 与第 6 节 |
| R7 | 残留已知红 | **存在，已接受** | `trust_key_golden_matches_core_algorithm` 仍红，铁律禁改；G3 判据为「逐名比对、零新增」而非「0 失败」 |
| R8 | 未修的偶发缺陷 | **低，但会复发** | `session_picker` 时间相关偶发缺陷（见 2.2）可能在未来 CI 上随机变红，与本轮无关但易被误判为回归 |
| R9 | 环境强约束 | **中**（对复跑者） | 8GB cgroup 须 `-j 1`；daemon 端口 13456–13458 并发会产生假红；复跑者忽略会得到错误结论 |
| R10 | 格式改动易被后续编辑覆盖 | **中** | rustfmt 会重排整块；并发会话在同一文件上编辑可能抵消本轮成果。**建议尽快落地提交** |

### 2.1 R5 详解：与 `2026-09-02-cleanup-codingplan-legacy` 的同树并发作业

- 该会话已落地 T1/T4/T5（其 `06-release.md` 自述 T3、T6 亦已完成）。
- **冲突点 1 —— `crates/rustcode-cli/src/main.rs` 是双方共同文件**：
  - 我方：A 组格式化了该文件（`:3363` 区域）。
  - 对方：T3 目标为删除 `Commands::Codingplan` 别名。
  - 对方决策日志曾记载该文件「非 dirty，可安全 `git checkout` 回滚」——**该前提已被我方改动推翻**，
    该文件现在是 dirty。任一方整文件 checkout 都会抹掉另一方的改动。
- **冲突点 2 —— `docs/REFACTOR_DESIGN_PHASE1.md` 双方均有改动**：
  我方上一轮（OpenRouter 归因）在 `:357` 标注 [DONE]；对方 T6 在标题后追加
  `[SUPERSEDED BY docs/phase1-refactor-design.md]` 指针行。两处改动**并存于同一 dirty 文件**。
- **[ERROR] 数据矛盾（需编排者裁决，见第 9 节 C1）**：编排者提供的数据称对方 T3「至今未执行」，
  但本件 Grep 实测 `Commands::Codingplan` 在 `crates/` 下 **0 命中**，与其 `06-release.md`
  自述「已删除，`rg` 0 命中」一致。→ 判定**该条数据已过期**，T3 极可能已执行。
  若已执行，则我方 `:3363` 格式改动是否仍保留**无任何证据**，须复跑 `cargo fmt --check` 确认。

### 2.2 R8 详解：`session_picker` 时间相关偶发缺陷（未修）

`modals::session_picker::tests::replay_keeps_current_model_window_and_ignores_accounting_only_turn_positions`
断言 `!label.contains("987")`（`987654` 为「仅记账」哨兵值），但标签渲染了会话 ID
`session-1788360798786`，该毫秒时间戳数字串**恰好含子串 "987"** 时即失败。
与时间相关、与 locale 无关、与本轮改动无关（末次运行通过）。**待用户裁决**修法
（改判 `total_tokens` 字段，或改用固定会话 ID）。

---

## 3. 验证结果（Verification）

**完整数据以 `05-test-report.md`（TEST-001）为准，本件不重写数字，仅引用结论。**

### 3.1 实际执行的命令

```bash
cargo fmt --check                                   # G1
cargo check -j 1 --workspace --all-targets          # 编译检查（非 AGENTS.md:200 定义的 clippy）
setsid nohup cargo test -j 1 --workspace --no-fail-fast > /tmp/g3_verify.log 2>&1 &
grep -riE "sentry|posthog|mixpanel|amplitude"       # G6
grep -rni "atomcode" crates/ scripts/ .github/      # G7
grep -rni "atomcode" docs/architecture.md           # G8
```

### 3.2 门禁与套件结论

| 门禁 / 目标 | 结论 |
|---|---|
| G1 `cargo fmt --check` | **exit=0 / 0 处差异**（清理前 19 处） |
| 编译检查 `cargo check -j 1 --workspace --all-targets` | exit=0 / **0 error** |
| G3 `cargo test -j 1 --workspace --no-fail-fast` | **5481 passed / 1 failed**；**90 个测试目标，89 个全绿** |
| G6 遥测 SDK | **0 真实命中** |
| G7 / G8 `atomcode` | **0 命中 / 0 命中** |
| G9 纯格式证明 | **pass**（剥离全空白后字符序列比对：6 文件完全一致、4 文件零语义差异） |

| 套件 | 结果 |
|---|---|
| `rustcode`(=cli) `--lib` | **116 / 0**（修复前 115/1） |
| `rustcode-tuix --lib` | **2064 / 0**（修复前 2059/5） |
| `rustcode-review --lib` | **100 / 0**（修复前 99/1） |
| `rustcode-coding --lib` / `rustcode-config --lib` / `rustcode-daemon --lib` / `rustcode-updater --lib` | 430/0、327/0、307/0、41/0 |
| `rustcode-kernel` | 全绿 |
| `rustcode-capabilities --lib` | 1475 / **1** —— `trust_key_golden_matches_core_algorithm`（`AGENTS.md:226` 铁律禁改） |

### 3.3 测试覆盖到的入口（明确列出）

**已覆盖**：CLI 库单测（`--lib`）、TUI 库单测（`--lib`）、`rustcode-review`、`rustcode-coding`、
`rustcode-config`、`rustcode-daemon`、`rustcode-updater`、`rustcode-kernel`、
`rustcode-capabilities` 的单元与集成目标 —— 均为**默认 feature、debug profile、Linux x86_64**。

**未覆盖入口**：headless 端到端（G4 `scripts/test-headless.sh`）、ACP 协议冒烟（G5 `scripts/acp_smoke.py`）、
非默认 feature 组合（`codingplan` / `codingplan-crypto` / `plugin`）、release profile、
Windows / macOS、WebUI 与文档站构建。见第 4 节。

### 3.4 零回归论证要点（详见 TEST-001 第 6 节）

1. 与 stash 基线**逐名比对**：8 个基线存量红全部在册，修复后仅剩 1 个 → **本 feature 引入的新失败 = 空集**。
2. 7 处修复**未改任何断言**，只新增锁行；落盘状态经本件 Grep 逐项复核（TEST-001 5.3）。
3. A 组经 G9 证明纯格式：`git diff -w` **不可**用于该证明（会把换行合并报成 258 行假阳性）。
4. 越界证明闭合：283/104 与上一轮 230/44 的差值 = 53 增 / 60 删 = **113 行**，恰等于 A 组 10 文件行数之和。

### 3.5 关键教训（必须随交付传承）

**locale 锁分两种，用错会制造新红。** `summarise_multi_line_adds_line_count` 是**刻意与 locale 无关**
的测试（用 `i18n::t()` 现算期望后缀再比对），要求两次调用间 locale 稳定。首次修复时把它和兄弟用例
一起 set 成 En，反而**新增 2 个红**。正解是**只持 `test_lock()` 取稳定性，绝不 `set_locale`**。

> **给断言英文的用例补锁前，必须先判明同模块是否存在 locale 无关型兄弟用例。**
> 该教训已在代码中留注释防复发（`crates/rustcode-tuix/src/event_loop/mod.rs:8842-8845`）。

---

## 4. 已知未验证 / 未做范围（Known Unverified Scope）

| # | 未覆盖 / 未做 | 建议负责人 |
|---|---|---|
| N1 | **`cargo clippy` 全量未跑**（`AGENTS.md:200` 的 G2 真义；本轮只跑了 `cargo check`） | 编排者：跑 `cargo clippy --workspace --all-targets` 与存量 warning 基线比对 |
| N2 | **G4 `./scripts/test-headless.sh` 未跑**（需先 `cargo build`） | 编排者 |
| N3 | **G5 `python3 scripts/acp_smoke.py` 未跑** | 编排者 |
| N4 | **非默认 feature 组合未验证**（`codingplan` / `codingplan-crypto` / `plugin`） | 编排者：`--features` 矩阵 `cargo check` |
| N5 | **仅 Linux x86_64 + debug profile**；Windows / macOS / `--release` 未验证 | 各平台 CI / 发布流程 |
| N6 | **WebUI `npm test` 与 `site/` 构建未跑**（本 feature 零前端改动，但 T6 文档改动可能影响搜索索引） | 站点维护者：`cd site && node build-search-index.mjs` |
| N7 | **G6 未取 `AGENTS.md:204` 的完整正则**（缺 `segment` / `google-analytics` / `googletagmanager`）；已知假阳性须人工判定 | 编排者 |
| N8 | **并发会话 T3 状态未确认**（数据称未执行，实测 0 命中 → 数据过期）；由此无法确认 `cli/src/main.rs` 的 `:3363` 格式改动是否仍保留 | 编排者：`git status` / `git log` + 复跑 `cargo fmt --check` |
| N9 | **本件作者无 shell**：`git diff --stat` / `git status` 未复核（编排者已补齐） | 编排者（一条只读命令） |
| N10 | **`session_picker` 偶发缺陷未修、未量化偶发率** | 用户裁决修法 → 测试维护者补重复运行 |
| N11 | **测试目标数 89 → 90 的差异未逐项归因** | 编排者 |
| N12 | **全部 29 个 dirty 文件未提交**：「测试通过」不等于「已合入」 | 编排者 / 用户（见第 8 节） |

**遗留后续项（不在本 feature 范围）**：
1. `trust_key_golden_matches_core_algorithm`：需换跨工具链稳定哈希（如 `std::hash::DefaultHasher` → 固定算法）
   才能转绿，属实现决策，**待用户裁决**，铁律禁止改测试凑绿。
2. `session_picker` 时间相关偶发缺陷修法裁决（见 N10）。
3. 子代理通道可用性：本轮累计 **3 次派发失败**（1 次无响应 + 2 次 idle timeout，因 cargo 编译期间
   无增量输出触发空闲超时）；`doc-writer` 因不跑 cargo 而派发成功。
   **结论：涉编译任务慎用子代理。** 建议编排者记录为环境事实，不重试同类派发。

---

## 5. 文档更新清单

| 文件路径 | 变更类型 | 摘要 |
|---|---|---|
| `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/05-test-report.md` | **新增** | TEST-001：验证基线、逐门禁/逐套件结果、存量红演进、失败归因、零回归论证、14 项未验证范围 |
| `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/06-release.md` | **新增** | 本件：四段式交付清单、回滚方案、唯一下一步、术语一致性结论 |
| `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/03-impl/T8-artifacts.md` | **新增** | 产物生成说明：采用了哪些数据、独立复核了什么、信息缺口清单 |
| `.codebuddy/artifacts/2026-09-02-g1-fmt-gate/03-impl/T6-docs-atomcode.md` | 既有（RELEASE-001） | 同 feature 的文档侧交付件，自带四段式清单与回滚方案；本件仅登记交叉引用，**不重复验证** |

**由 T6 交付件登记、本件不重复验证的产品文档改动**（如需复核请查阅 `T6-docs-atomcode.md` 第 7 节）：
`README.md`（修订）、`README.zh-CN.md`（修订）、`docs/features.md`（修订）、
`docs/platform-neutralization.md`（修订）。

[CHECK] 本件复核：`AGENTS.md:282` 与 `AGENTS.md:539` 已带 `[SUPERSEDED 2026-09-02,见第三十二轮]`
标记，与 T6 的清理结果一致 —— **T6 交付件第 5 节 U1（「AGENTS.md 两处过程记录已过时」）现已闭环**，
无需再上报。

[INFO] 本仓库**不存在 `CHANGELOG`**，故无 CHANGELOG 条目需要更新。

[INFO] 本 feature **未改动** `docs/` 下任何技术文档（A/B 两组均为源码改动；
`docs/REFACTOR_DESIGN_PHASE1.md` 的修改属上一 feature，非本轮）。

---

## 6. 回滚方案（Rollback）

### 6.1 判定时机（任一出现即回滚）

1. 复跑 `cargo fmt --check` **非 exit=0**（说明 A 组未真正归零或被并发编辑抵消）。
2. 复跑 `cargo test -j 1 --workspace --no-fail-fast` 出现 `trust_key_golden_matches_core_algorithm`
   **之外**的新失败，且逐名比对确认由本 feature 引入。
3. 与并发会话在 `crates/rustcode-cli/src/main.rs` 上的改动**无法逐 hunk 共存**。
4. 合规/审计要求回退任一 A 组文件的格式（概率极低，A 组为零语义格式）。

### 6.2 回滚步骤（须由编排者执行；本件作者无 git 写权限）

```bash
# 0) 前置：确认无 cargo 在跑，避免 target 锁 / daemon 端口争用造成假象
pgrep -c cargo                      # 必须为 0

# 1) 先存补丁，保留可重放能力
git diff > /tmp/g1-fmt-gate-all.patch                  # 全量留档（含并发会话改动，仅作快照，勿整体回用）
git diff -- crates/rustcode-cli/src/schedule_cmd.rs \
            crates/rustcode-coding/src/runtime.rs \
            crates/rustcode-tuix/src/event_loop/commands.rs \
            crates/rustcode-tuix/src/event_loop/mod.rs \
            crates/rustcode-tuix/src/modals/dir_picker.rs \
            crates/rustcode-tuix/src/modals/onboarding_wizard.rs \
            crates/rustcode-tuix/src/render/cell.rs \
            crates/rustcode-tuix/src/test_term.rs \
            crates/rustcode-updater/src/lib.rs > /tmp/g1-fmt-gate-fmt.patch
git diff -- crates/rustcode-review/src/review_tool.rs \
            crates/rustcode-cli/src/acp/translate.rs \
            crates/rustcode-tuix/src/event_loop/mod.rs > /tmp/g1-fmt-gate-locks.patch

# 2a) B 组（locale 锁）回滚：反打补丁即可（+9 行量级）
git apply -R /tmp/g1-fmt-gate-locks.patch

# 2b) A 组（纯格式）回滚：逐文件 checkout —— 但见 6.3 安全边界
git checkout -- crates/rustcode-cli/src/schedule_cmd.rs crates/rustcode-coding/src/runtime.rs \
                crates/rustcode-tuix/src/event_loop/commands.rs \
                crates/rustcode-tuix/src/modals/dir_picker.rs \
                crates/rustcode-tuix/src/modals/onboarding_wizard.rs \
                crates/rustcode-tuix/src/render/cell.rs \
                crates/rustcode-tuix/src/test_term.rs \
                crates/rustcode-updater/src/lib.rs
# 注意：crates/rustcode-tuix/src/event_loop/mod.rs 同时属于 A、B 两组。
#       先反打 locks 补丁，再 checkout 该文件，顺序不可颠倒；或改为逐 hunk 人工反向编辑。

# 3) 复核
cargo fmt --check
cargo test -j 1 --workspace --no-fail-fast
```

### 6.3 安全边界（硬约束）

- **[ERROR] 严禁** `git reset --hard` / `git stash` / `git checkout .` / `git checkout -- <目录>`：
  worktree 含**上一轮遗留 11 个文件**与**并发会话改动**，blanket 回退会造成不可逆丢失。
- **[ERROR] `crates/rustcode-cli/src/main.rs` 禁止整文件 `git checkout --`**：
  该文件与并发会话 T3 共享，整文件回退会抹掉对方（或我方）的改动。
  改为 `git diff -- crates/rustcode-cli/src/main.rs` 后**逐 hunk 人工反向编辑**。
- **[ERROR] `docs/REFACTOR_DESIGN_PHASE1.md` 禁止 `git checkout --`**：
  该文件 dirty（我方上一轮 [DONE] 标注 + 对方 T6 追加的 SUPERSEDED 指针），只能逐 hunk 处理。
- [WARN] 执行任何 checkout 前先 `git diff -- <file>` 确认该文件**只含本 feature 改动**；
  若已被并发会话覆盖，必须人工 diff 还原对应 hunk。

### 6.4 回滚代价与可预期后果

- **代价：低。** 全部为格式与测试锁，无数据迁移、无配置变更、无协议变更、无构建产物依赖。
- **可预期后果**（不是新故障，回滚前须知情）：
  - A 组回滚 → G1 重新变红（19 处差异），CI fmt job 转红，产品行为不变。
  - B 组回滚 → 7 个用例在默认 `Locale::ZhCn` 下**重新确定性失败**（存量红回到 8）。

---

## 7. 术语与命名一致性检查结论

| 检查项 | 结论 |
|---|---|
| 门禁编号 | `AGENTS.md:196-218` 定义 G1–G8。本件已标注差异：**本轮「G2」实为 `cargo check`，非 AGENTS.md 的 `cargo clippy -- -D warnings`**；「G9 纯格式证明」为本 feature 自定义门禁，**不在 AGENTS.md 门禁表内**。未改写 `AGENTS.md` |
| 包名 | `rustcode-cli` 目录的 crate 名是 **`rustcode`**；报告中一律写作 `rustcode --lib` 并注明 `(=cli)`，与 `AGENTS.md:13`（及 `:547`）约束一致 |
| ASCII 化 | 本件与 TEST-001 全文**零 Unicode emoji**；状态标签仅用 `[INFO]` / `[CHECK]` / `[WARN]` / `[ERROR]` / `[DONE]` |
| 退役组件 | 本次交付**不涉及**任何退役组件；未把 `atomcode-*` 或已退役 core 写成仍可使用的能力 |
| 文档化已知红 | `trust_key_golden_matches_core_algorithm` 在两件中均表述为「`AGENTS.md:226` 列明、铁律禁改」，与 `AGENTS.md:226` 原文一致，未写成待修项 |
| locale 锁术语 | 严格区分 `i18n::test_lock()`（取稳定性，**不**改语言）与 `set_locale(Locale::En)`（钉语言），与 `AGENTS.md:231` / `:349` 的既定约定一致 |
| 架构边界 | 本 feature 不涉及 `AGENTS.md` 的单一状态所有权 / 依赖方向 / core 退役结论，**无需校准** |
| 上游 slug 一致性 | [CHECK] 本件 Grep 复核：`atomgit_atomcode` 现仅存于 `AGENTS.md`（历史记录，已带 [SUPERSEDED] 标记）、`docs/UPSTREAM_CREDITS.md:6`、`docs/UPSTREAM_RUSTCODE_LICENSE.md:4`、`docs/THIRD_PARTY_NOTICES.md:8`（MIT 合规落点）、`docs/REFACTOR_DESIGN_PHASE1.md:357`（历史分析引用）；两个 README 与 `docs/{features,platform-neutralization}.md` 正文已清零 |
| 中英一致 | 本次交付无面向用户的中英文案改动；T6 侧的两 README 语义对齐由其自有交付件登记 |

---

## 8. 唯一下一步（Single Next Step）

**[CHECK] 唯一建议下一步**：与并发会话 `2026-09-02-cleanup-codingplan-legacy` 完成
`crates/rustcode-cli/src/main.rs` 的冲突对齐 —— 确认该文件的「我方 `:3363` 格式 hunk」与
「对方 T3 删除 `Commands::Codingplan` 的 hunk」**同时保留**（用 `git diff -- crates/rustcode-cli/src/main.rs`
逐 hunk 核验；若已丢失则重跑 `cargo fmt -p rustcode` 补回），并复跑 `cargo fmt --check`
与 `cargo test -j 1 --workspace --no-fail-fast` 与 TEST-001 的已知红基线逐名比对。

该步完成后即可按建议标题将本 feature 的 13 个文件（A 组 10 + B 组 3）作为**一次独立 commit** 提交：
`chore(fmt): clear G1 fmt gate and pin locale in 7 i18n tests`；
提交须用 `git add` 精确暂存，**禁止 `git add -A`**（会夹带上一轮遗留与并发会话改动）。

---

## 9. 上报项（需编排者 / 用户裁决，本件不自行决定）

| # | 级别 | 事项 |
|---|---|---|
| C1 | **高** | **并发会话 T3 状态矛盾**：编排者数据称「至今未执行」，但本件 Grep 实测 `Commands::Codingplan` 在 `crates/` 下 **0 命中**，且对方 `06-release.md` 自述已删除。请编排者以 `git status` / `git log` 确认，并复跑 G1 确认我方 `:3363` 格式改动是否保留（见 N8 / R5） |
| C2 | 中 | **`trust_key` 已知红**：是否改用跨工具链稳定的哈希实现，需用户裁决。铁律禁止改测试凑绿，本 feature 不处理 |
| C3 | 中 | **`session_picker` 时间相关偶发缺陷**：修法（改判 `total_tokens` 字段 vs 改用固定会话 ID）需用户裁决 |
| C4 | 低 | **提交分组**：T6 的 4 份文档改动与本件 A/B 两组是否合入同一 commit，请编排者决定 |
| C5 | 低 | **子代理通道**：本轮累计 3 次派发失败（涉 cargo 编译任务），建议记录为环境事实并在同类任务中改用直接执行 |

---

## 10. 结论

`status: done`，`decision: proceed`。

G1 格式门禁归零（19 → 0 处差异）并已证明为纯格式；7 处 locale 缺锁修复使存量红 **8 → 1**（净减 7、
零新增）；G6/G7/G8 全绿；编译 0 error。唯一残留失败为 `AGENTS.md:226` 铁律禁改的文档化已知红。

**唯一阻塞项是 C1（并发会话 T3 状态与我方 `main.rs` 格式 hunk 是否保留）** —— 该步不解决，
不建议提交；解决后即可按第 8 节的建议标题独立提交。
