---
kind: design
id: DESIGN-GW-001
from: solution-architect
to: [project-manager]
feature: 2026-09-03-git-wrapup
status: ready
decision: proceed
requires: [REQ-ORCH-2026-09-03-GIT-INVENTORY]
files_owned: []
architecture_constraints:
  touches_runtime_lifecycle: false
  touches_persistence: false
  touches_cross_crate_deps: false
created: 2026-09-03
---

# 2026-09-03 Git 收尾执行计划（方案与契约）

## 执行摘要（不超过 10 行）

最该做的三件事：

1. **[P0] 结算 git 提交身份** —— `user.name`/`user.email` 均为 UNSET，上一轮靠 `git -c` 临时传参推送；这是后续所有 commit / merge / push 的硬前置（`GW-01`）。
2. **[P1] 清掉工作区唯一未跟踪项并入库** —— `docs/multi-agent-collaboration-solution.md`（300 行）被 `.codebuddy/rules/multi-agent-workflow.md:5` 与 `AGENTS.md:539` 活跃引用，却未入库，处于"规则引用了它、它不在版本库里"的不一致状态（`GW-02`/`GW-05`）。
3. **[P1] 把 dev 的 33 个提交（985 files / +55613 / -44000）合并进 main 并推送** —— main 自 fork 后从未前进（`merge-base main dev` = `287bff70` = main HEAD），落后 33 提交（`GW-08`/`GW-09`/`GW-10`）。

最大的两个风险：

- **[R1] CI `test` job 必然红** —— `rustcode-capabilities` 的 `mcp::registry::tests::trust_key_golden_matches_core_algorithm` 是 `AGENTS.md:226` 文档化已知红（`DefaultHasher` 跨工具链不稳定，铁律禁改）。合并后首批 CI 会红，**处置口径必须先裁决，否则会被误判为回归**。
- **[R2] 子代理通道 4 连败** —— `code-implementer` 派发累计 4 次失败（2×"No result found" + 2×idle timeout），T1 修复可能再次空转；**每个可执行任务都预置"编排者代行"回退路径与额外工时**。

---

## 1. 事实基线与假设分界（禁止把假设写成事实）

### 1.1 已完成的事实（编排者 2026-09-03 实测，权威）

| 项 | 事实 |
|---|---|
| 远端 | 唯一 `origin = https://gitcode.com/SecLab/RustCode`（fetch/push 同址）；远端分支仅 `origin/main`、`origin/dev` |
| 本地分支 | `dev = ba863a1a`（upstream `origin/dev`，ahead 0 / behind 0）；`main = 287bff70`（upstream `origin/main`，ahead 0 / behind 0） |
| 合并基 | `git merge-base main dev` = `287bff70` = main HEAD；`main` 自 fork 后从未前进 |
| 未合并量 | `dev` 领先 `main` **33 提交**，`main` 领先 `dev` **0 提交**；变更面 **985 files changed, 55613 insertions(+), 44000 deletions(-)** |
| 工作区 | 已暂存 0；未暂存修改 0；未跟踪 **仅 1 个**（`docs/multi-agent-collaboration-solution.md`，300 行）；stash 0；worktree 1；`git ls-files -u` 空（无冲突） |
| tag / reflog | tag **0 个**；reflog 干净，末次操作为 `ba863a1a commit`，无中断残留 |
| 提交身份 | `git config --get user.name` = UNSET；`git config --get user.email` = UNSET；上一轮用 `git -c user.name='rustcode-builder' -c user.email='builder@rustcode.local'` 一次性传入 |
| CI 门禁 | `.github/workflows/ci.yml`：job `fmt`（`cargo fmt --all -- --check`）、`clippy`（`cargo clippy --workspace --all-targets`，**无 `-D warnings`**，源码 TODO 注明待 ~420 条告警收敛）、`test`（`cargo test --workspace`，裸跑，无 `-j 1`、无 `--no-fail-fast`） |
| 已知红基线 | `mcp::registry::tests::trust_key_golden_matches_core_algorithm`（`AGENTS.md:226`，铁律禁改）→ CI `test` job 必然红，非新引入 |
| 本机约束 | cgroup 内存上限 8GB，`cargo test` 默认并发触发 rustc SIGBUS，本地必须 `-j 1`；daemon 测试争用固定端口 13456-13458 |
| 子代理通道 | 累计 **4 次派发失败**；已知根因之一为编译期无增量输出触发 idle timeout；缓解手段为派发前预热（本次 1.96s 缓存命中） |
| 待修缺陷 T1 | `modals::session_picker::tests::replay_keeps_current_model_window_and_ignores_accounting_only_turn_positions` 偶发红；根因链实证：`crates/rustcode-tuix/src/session.rs:119-123`（`name: format!("session-{now}")` 毫秒时间戳）→ `crates/rustcode-tuix/src/modals/session_picker.rs:912-915`（渲染进 `SessionResumedLabel`）→ `:2166-2169`（断言 `!label.contains("987")`）。用户已裁决修法：**钉定会话名**，不改断言、不改生产代码 |
| T1 现状 | 编排者已派发 `code-implementer`，返回 `No result found`（第 4 次失败）；已核验为干净 no-op（源码零改动、交接件零落盘、无残留 cargo 进程）。**T1 尚未实施** |
| 看板现状 | `2026-09-02-strip-atomcode` 停在 G1 pending；`2026-09-02-cleanup-codingplan-legacy` G6 pass 已提交 `783d48e4` 并推送，但决策日志末两行留"已暂存 21 文件 / 待用户确认是否 commit"的悬挂记录；`2026-09-02-g1-fmt-gate` G1/G2/G3 全 pass 但标题仍写"冻结"，其 2 项遗留已转移到 `2026-09-03-residual-two-items` |

### 1.2 待办假设（显式标注，未经验证不得作为结论）

| 编号 | 假设 | 验证任务 |
|---|---|---|
| `[假设-待验证] A1` | `.codebuddy/artifacts/**` 已被 git 跟踪（依据：工作区未跟踪项仅 1 个，故该目录必已入库；未直接证实） | `GW-04` 前置校验 `git ls-files --error-unmatch .codebuddy/artifacts/2026-09-02-strip-atomcode/STATUS.md` |
| `[假设-待验证] A2` | CI runner 内存规格未知，裸跑 `cargo test --workspace` 是否触发 SIGBUS 未知 | `GW-16` 观察首次 CI 结果 |
| `[假设-待验证] A3` | `origin/main` 未开启分支保护，允许直接 push | `GW-10` 的 push 结果；被拒则转 PR，禁止 force |
| `[假设-待验证] A4` | `7cdc1294 Revert` 之后 `rustcode-config` 的 clippy 修复是否完整未知 | `GW-15` 只读核对 |
| `[假设-待验证] A5` | release 链路 secrets（`RELEASE_API_HOST/OWNER/ACCESS_TOKEN`，见 `.github/workflows/build.yml:73-75`）是否已配置未知；未配置则首个 `v*` tag 会触发构建但上传失败 | `GW-13` 打 tag 前的检查 |

---

## 2. 现状盘点（含 `文件:行` 引用）

### 2.1 分支拓扑

```
287bff70 (main, origin/main, merge-base)
     \
      \-- 33 commits (2026-08-29 .. 2026-09-03) -- ba863a1a (dev, origin/dev)
```

- main 领先 dev 0 提交 → **合并不存在三方冲突面**，dev 侧树被整体采用。
- 风险不在"合并冲突"，而在"**内容回归**"：这批 985 文件的重命名迁移 + 平台中立化 + i18n 改动，在 main 上从未被构建/测试过（`[假设-待验证] A3` 之外的独立风险，见 R3）。

### 2.2 门禁与流水线

- `.github/workflows/ci.yml:35-42`：`test` job 裸跑 `cargo test --workspace`，**必然红**于 trust_key（`AGENTS.md:226`）。
- `.github/workflows/ci.yml:32-33`：`clippy` job 无 `-D warnings`，当前不会因存量告警失败。
- `.github/workflows/ci.yml:21`：`fmt` job 预期绿（`2026-09-02-g1-fmt-gate` 已将 19 处违规归零）。
- `.github/workflows/build.yml:4-6`：release 构建由 `v*` tag 触发；`build.yml:258-272` 另有 `distro-pm-check` job。
- G6（遥测 SDK）/ G7（`atomcode` in crates/scripts/.github）/ G8（`atomcode` in `docs/architecture.md`）**无 CI job**，仅本地人工门禁（`AGENTS.md:220` 同述）。

### 2.3 待入库文档与其引用闭环

- `.codebuddy/rules/multi-agent-workflow.md:5` → "完整方案见 `docs/multi-agent-collaboration-solution.md`"
- `AGENTS.md:539` → 同文档被正文引用
- 结论：该文档是随会话自动加载的协作规则的**上游正文**；未入库意味着任何新克隆/新机器上规则指向一个不存在的文件。

### 2.4 待修测试的根因链（已实证，非推测）

- `crates/rustcode-tuix/src/session.rs:63` — `pub type Session = TuiSession;`
- `crates/rustcode-tuix/src/session.rs:119-123` — `TuiSession::new` 生成 `name: format!("session-{now}")`，`now = now_ms()`
- `crates/rustcode-tuix/src/modals/session_picker.rs:2121` — 用例内 `let mut session = Session::new(PathBuf::from("/tmp/x"));`
- `crates/rustcode-tuix/src/modals/session_picker.rs:912-915` — `replay_session` 把 `session.name` 渲染进 `TurnSeparator` 的 `SessionResumedLabel`
- `crates/rustcode-tuix/src/modals/session_picker.rs:2166-2169` — 断言 `labels.iter().all(|label| !label.contains("987"))`（`987_654` 是"仅记账"哨兵 `total_tokens`，见 `:2145`）
- 既有同构先例：`crates/rustcode-tuix/src/session.rs:137-141` — `TuiSession::default_session` 构造后覆盖 `session.name = "default".into()`

---

## 3. 候选方案与取舍

### 3.1 决策点 Q1：`dev → main` 合并方式

| 方案 | 做法 | 取舍 |
|---|---|---|
| **A（推荐）merge commit `--no-ff`** | `git checkout main && git merge --no-ff dev` | 选它：① 在 main 上留下**一个原子批次边界**，回滚只需 `git reset --hard 287bff70`（未推送）或 `git revert -m 1 <merge-sha>`（已推送），边界清晰；② 保留 33 个提交的作者/日期/粒度，`AGENTS.md` 的"轮次"叙事依赖该历史可追溯；③ 因 behind=0，产生的树与 dev 完全一致，可用 `git diff --stat main dev` 为空来验收 |
| B fast-forward | `git merge --ff-only dev` | 放弃：历史线性但**无批次边界**，33 提交淹没在 main 主干里；回滚需靠 `git reset --hard`（未推送尚可）或逐个 revert（已推送则 33 次操作，易错）。唯一优势（线性历史）在本仓库价值不高 |
| C squash | `git merge --squash dev` | 放弃：丢掉 33 个提交的作者/日期/中间态，且把重命名迁移、平台中立化、i18n 三批不同主题压成一个不可分块，回滚粒度退化到"全有或全无" |

**变更面风险评估**：985 files / +55613 / -44000 数字大，但其中主体是 `atomcode-* → rustcode-*` 的**机械重命名**（`crates/`、`docs/` 面）与 i18n/平台中立化文案改动；behind=0 意味着**零三方冲突**。真实风险是内容回归（R3），由 `GW-08` 的全量门禁覆盖，而非靠合并策略规避。

### 3.2 决策点 Q2：CI `test` job 已知红处置

| 方案 | 做法 | 是否触碰铁律 | 评价 |
|---|---|---|---|
| **A（推荐）已知红白名单守卫** | test job 改 `cargo test --workspace --no-fail-fast`，收集失败用例名，判定"失败集 ⊆ 白名单"则 pass，否则 fail；白名单内含 `mcp::registry::tests::trust_key_golden_matches_core_algorithm` 并注释回指 `AGENTS.md:226` | **不触碰**：不改测试、不改生产实现 | 保留全部可见性，且能捕获新增回归；成本是需维护白名单文件 |
| B 修 `project_trust_key` 为稳定哈希 | 改生产实现（如固定 FNV-1a/SHA）并更新 golden 值 | **触碰**：改 golden 直接违反 `AGENTS.md:226`「不要随手改测试去凑绿」；改实现会改变既有 MCP trust 记录的计算结果，属持久化/安全边界变更 | 高风险，需独立立项 + 数据迁移评估，**不在本次收尾内做** |
| C `continue-on-error: true` | 整 job 标黄不阻断 | 不触碰测试，但违反协作方案"禁止静默降级/假成功" | **不推荐**：任何真实回归都会被标成成功 |
| D `#[ignore]` 该用例 | 测试内标注跳过 | **触碰**铁律精神（削弱测试可见性） | **不推荐** |

### 3.3 决策点 Q3：git 提交身份

| 方案 | 做法 | 取舍 |
|---|---|---|
| **A（推荐）写入 git 配置** | `git config user.name '<name>' && git config user.email '<email>'`（仓库级，作用域仅本仓库） | 一次性结算；后续所有 commit/merge/push 不再依赖临时传参；新机器/CI/他人协作可复现 |
| B 继续 `git -c` 一次性传参 | 每次提交临时注入 | 放弃作为常态：每次依赖人肉记忆，历史作者易不一致；且 `git merge` 等命令同样需要身份。仅可作为 `GW-01` 未完成时的**应急通道**，且必须在看板登记为 `[WARN]` 偏差 |
| C 写入 `AGENTS.md` 作为团队约定 | 文档化传参约定 | **不推荐**：约定不能替代可执行的机器配置；且 `AGENTS.md` 属本计划禁改范围（需用户单独授权） |

### 3.4 决策点 Q4：首个 tag 与发布策略

- 当前 tag 数为 0；`Cargo.toml:30`（`[workspace.package]`）`version = "5.0.9"`。
- `.github/workflows/build.yml:4-6` 在 `v*` tag 推送时触发 macOS/Linux/Windows 三平台 release 构建 + 上传（`create_tag_release.py`，需 secrets）。
- 建议：**合并推送完成后**再决定是否打 `v5.0.9`；打 tag 前必须确认 `[假设-待验证] A5`（secrets 是否就绪），否则会得到一个"构建成功但上传失败"的半成品发布。

---

## 4. 目标状态（收尾完成后的期望快照）

```
origin/main == main == merge commit(dev 的全部内容 + 文档入库 + T1 修复 + 看板闭板)
origin/dev  == dev   == ba863a1a 之后的新提交链（文档入库 / T1 修复 / 看板闭板）
工作区: git status --short 输出为空（无未跟踪、无修改、无暂存）
git var GIT_AUTHOR_IDENT: 成功返回（身份已结算）
CI: fmt=绿, clippy=绿, test=按 Q2 裁决口径（现状为红于 trust_key）
看板: 三个历史看板闭板归档；2026-09-03-residual-two-items 的 T1/T2 转 done
```

关键顺序原则：**先在 dev 上把三项改动各自原子提交（文档 / T1 / 看板闭板），再合并进 main**。这样 main 侧只多一个 merge 节点，且每个改动可独立回滚。

---

## 5. 接口契约（本计划的冻结契约，变更须回退 `solution-architect`）

### 5.1 提交粒度契约

| 提交 | 内容 | 提交信息前缀 | 允许的文件面 |
|---|---|---|---|
| C1 | 协作方案文档入库 | `docs(collab): add multi-agent collaboration solution` | 仅 `docs/multi-agent-collaboration-solution.md` |
| C2 | 历史看板闭板归档 | `chore(board): close 2026-09-02 boards (strip-atomcode / cleanup-codingplan-legacy / g1-fmt-gate)` | 仅 `.codebuddy/artifacts/2026-09-02-{strip-atomcode,cleanup-codingplan-legacy,g1-fmt-gate}/STATUS.md` |
| C3 | T1 钉定会话名 | `fix(tuix): pin replay session name in session_picker test` | 仅 `crates/rustcode-tuix/src/modals/session_picker.rs` |
| C4 | dev → main 合并 | `merge: dev into main (33 commits, 985 files, 2026-08-29..09-03)` | merge commit，两个 parent |

- C1/C2/C3 落在 `dev`；C4 在 `main`。
- **禁止**把 C1/C2/C3 混成一个提交（回滚粒度要求）。
- **禁止**在 C4 之后对 main 做 force 更新。

### 5.2 门禁基线契约（fail-closed）

- G-A：`cargo fmt --all -- --check` exit=0。
- G-B：`cargo test -j 1 --workspace --no-fail-fast` 的失败集 **必须 ⊆** `{mcp::registry::tests::trust_key_golden_matches_core_algorithm}`；出现任何集合外失败即判定回归，停止后续合并动作。
- G-C：`git status --short` 输出为空（每次提交前/合并前）。
- G-D：`git var GIT_AUTHOR_IDENT` exit=0 且输出不含 `(none)`（每个写操作前）。

### 5.3 测试基线契约（数字冻结，不得改判）

`tuix --lib` 2064/0；`cli(rustcode) --lib` 116/0；`review --lib` 100/0；`coding --lib` 430/0；`config --lib` 327/0；`daemon --lib` 307/0；`updater --lib` 41/0；`capabilities --lib` 1475/1（trust_key）；全工作区 5481 passed / 1 failed。
（来源：`.codebuddy/artifacts/2026-09-02-g1-fmt-gate/STATUS.md:226-238` 与 `2026-09-03-residual-two-items/STATUS.md:71-76`。）

### 5.4 分支与远端契约

- `dev` 是持续集成分支，**不删除**；`main` 是发布分支，**不删除**。
- 远端仅 `origin/main`、`origin/dev`，**不新增远端分支**（除非 Q4 决定走发布流程）。
- 回滚锚点：`git branch backup/main-pre-merge-20260903 287bff70`（本地分支，**不用 tag**，避免污染 0-tag 命名空间，见 Q4）。

### 5.5 交接件契约

- 看板 `STATUS.md` 与交接件**只追加、不改写既有行**；可判定校验为每个被改文件的 `git diff --numstat` 删除列 = 0。
- 非 `done` 终态必须写 `decision` 与下一跳接收方，禁止悬挂。

---

## 6. 状态所有权

| 状态 | 唯一所有者 | 只读方 | 创建/销毁时机 |
|---|---|---|---|
| 工作区 / 索引 / HEAD | **用户**（编排者仅在获显式授权后代执行） | 全体 | 提交窗口内独占；同批次内多个提交任务必须串行 |
| 本地分支引用 `dev`/`main` | **用户** | 全体 | 全程存在；`backup/main-pre-merge-20260903` 于 `GW-09` 前创建、保留至合并验证通过后由用户决定是否删除 |
| 远端引用 `origin/*` | **用户** | 全体 | 仅 `GW-10` 推送时变更 |
| tag 命名空间 | **用户**（当前 0 tag） | 全体 | 仅在 Q4 裁决"打 tag"后创建 |
| `.codebuddy/artifacts/**` 看板 | **编排者(project-manager)** | 全体 | `GW-04` 独占写入三个历史看板 `STATUS.md` |
| `docs/multi-agent-collaboration-solution.md` | **doc-writer**（`GW-02` 期间）→ 用户提交后归仓库 | 全体 | `GW-02` 校验期间独占 |
| `crates/rustcode-tuix/src/modals/session_picker.rs` | **code-implementer**（`GW-03` 期间）；回退时由**编排者**接管并在看板登记 | 全体 | `GW-03` 独占；跨 crate 改动默认串行（本任务仅 1 个 crate） |
| `.github/workflows/ci.yml` | **用户**（授权后由编排者代执行） | 全体 | `GW-12` / `GW-17` 独占，两者必须串行 |
| `AGENTS.md` | **用户**（本计划不触碰） | 全体 | `GW-18` 默认 blocked |

单一所有者保障：任何 `files_owned` 在同一批次内只出现一次；同批次任务数 ≤ 3；提交窗口（HEAD/索引）在 `GW-05`/`GW-06`/`GW-07` 之间严格串行。

---

## 7. 失败与取消语义（禁止静默降级与假成功）

| 失败场景 | 错误信号 | 恢复动作 | 禁止动作 |
|---|---|---|---|
| 缺提交身份 | `Author identity unknown` / `git var GIT_AUTHOR_IDENT` 含 `(none)` | 停止提交，回到 `GW-01` | 禁止默认改用 `git -c` 临时传参绕过（仅可作应急并登记 `[WARN]`） |
| 合并出现冲突 | `CONFLICT (content)` | 立即 `git merge --abort` → `git fetch origin` → 重新评估 → 升级用户 | 禁止在 985 文件面上手工解冲突（不可审） |
| 测试失败集超出已知红 | G-B 判定失败 | fail-closed：**停止合并**，回到 `GW-08` 定位；新失败登记为独立缺陷 | 禁止"先合并再修"；禁止改断言/改 golden 凑绿 |
| 合并预演不合预期 | `git diff --cached --stat` 与 985 files 不符 | `git merge --abort`，核对 `git rev-list --count main..dev` | 禁止带着未知差异提交合并 |
| 推送被拒（非 FF / 保护分支） | `! [rejected]` | 转 PR 路径或升级用户 | **禁止 `--force` / `--force-with-lease`** |
| 子代理派发失败（第 2 次） | `No result found` / idle timeout | 编排者代行 + 在看板登记 `[WARN]` 通道失败；第 3 次失败强制升级用户 | 禁止把空结果当成成功；禁止跳过门禁 |
| CI 出现 trust_key 之外的失败 | Actions job 失败名不属白名单 | 视为回归，升级用户，回退到 `GW-08` | 禁止加白名单掩盖 |
| 文档校验发现 `atomcode-*` 或图形 emoji | grep 命中 | 回到 `GW-02` 修正后再入库 | 禁止带着违规内容提交（违反 `AGENTS.md` ASCII 与 G7/G8 口径） |
| 看板闭板引入"删除既有行" | `git diff --numstat` 删除列 ≠ 0 | 撤销该 hunk 重做（只追加） | 禁止改写既有结论行 |

取消语义：任一批次中途取消（用户中断 / 会话超时），必须先把工作区恢复到"可判定状态"——未提交的合并用 `git merge --abort`，未验证的改动保留在 worktree 并在看板登记 `[BLOCKED]` 与下一跳，**禁止 `git checkout .` 或 `git reset --hard` 清理**（会抹掉用户既有改动）。

---

## 8. 合并与回滚方案（可执行命令）

### 8.1 合并前锚点与预演

```bash
# 0) 前置身份校验（G-D）
git var GIT_AUTHOR_IDENT

# 1) 建立回滚锚点（本地分支，不用 tag）
git branch backup/main-pre-merge-20260903 287bff70

# 2) 预演：不合意可完整撤销
git checkout main
git merge --no-commit --no-ff dev
git diff --cached --stat | tail -1          # 期望: 985 files changed, ...
git merge --abort                            # 撤销预演
git rev-parse --short HEAD                   # 期望: 287bff70
git status --short                           # 期望: 输出为空
```

### 8.2 正式合并

```bash
git checkout main
git merge --no-ff dev -m "Merge branch 'dev' into main: 2026-08-29..09-03 migration & neutralization batch (33 commits, 985 files)"
```

验收：

```bash
git cat-file -p HEAD | grep -c '^parent'    # 期望: 2
git rev-parse main^2                        # 期望: dev 的 sha
git diff --stat main dev                    # 期望: 输出为空（两棵树一致）
git rev-list --count main..dev              # 期望: 0
```

### 8.3 回滚（按是否已推送分两种）

**未推送**（推荐窗口内回滚）：

```bash
git checkout main
git reset --hard 287bff70                   # 或 git reset --hard backup/main-pre-merge-20260903
```

**已推送**（禁止 force，生成反向提交）：

```bash
git checkout main
git revert -m 1 <merge-sha>                 # -m 1 = 回到第一父（原 main）的树，等价于 287bff70
git push origin main
```

注意：`git revert -m 1` 之后若将来要重新合并 dev，必须先 `git revert <revert-sha>`（撤销撤销），否则 dev 的改动不会再次进入。**禁止**对已推送的 main 做 `reset --hard` + force push。

### 8.4 提交级回滚（C1/C2/C3 各自独立）

```bash
git revert --no-edit <C1-sha>   # 文档入库
git revert --no-edit <C2-sha>   # 看板闭板
git revert --no-edit <C3-sha>   # T1 修复
```

---

## 9. 迁移与回退（历史与数据兼容）

- **无数据格式迁移**：本计划不触碰任何持久化格式（`$RUSTCODE_HOME` 下 session / codingplan_sync / trust 记录均不变）。
- **历史兼容**：合并采用 `--no-ff`，不改写既有 33 个提交的 sha，**不重写已推送历史**。所有回滚一律走"新增提交"（`revert`）而非"改写历史"。
- **禁止双向转换**：看板闭板只追加修订记录，**不修改** `2026-09-02-cleanup-codingplan-legacy/STATUS.md` 里"已暂存 21 文件 / 待用户确认"那两行的原文，而是在其后追加一条修订记录说明事实已被 `783d48e4` 取代。
- **回退步骤**：见 §8.3 / §8.4；回退后必须在 `02-tasks.md` 的「进度快照」登记实际执行的命令与结果。

---

## 10. 架构边界核对（逐条对照 `AGENTS.md`）

| 约束 | 本计划的做法 | 结论 |
|---|---|---|
| `CodingRuntime` 为唯一运行时生命周期所有者 | 本计划不触碰任何运行时代码 | 合规 |
| kernel / capabilities / coding 生产依赖 core-free，依赖方向不反转 | 无依赖变更 | 合规 |
| 不得恢复 bridge / v1-v2 开关 / legacy writer / fallback | 无代码改动（`GW-03` 仅改 `#[cfg(test)]` 夹具数据） | 合规 |
| native `SessionManager/SessionMeta/SessionSnapshot` 唯一 session 持久化模型 | 不触碰；`GW-03` 只覆盖用例内 `session.name` 字符串，不改 `TuiSession::new`（`session.rs:119-123` 保持原样），生产行为不变 | 合规 |
| `LifecycleHooks::turn_complete` / capabilities compaction 不得新增重叠 hook | 不涉及 | 合规 |
| `AGENTS.md:226` trust_key 已知红铁律禁改 | `GW-11` 候选 B/C/D 均标注触碰或不推荐；推荐候选 A 不改测试、不改实现 | 合规 |
| `AGENTS.md` 禁图形 emoji、状态一律 ASCII 标签 | 本两份文件全部使用 `[P0]/[TODO]/[WARN]/[BLOCKED]/[DONE]` 等 ASCII 标签，无图形 emoji | 合规 |
| 交接件只追加、不改写既有结论 | `GW-04` 以 `git diff --numstat` 删除列 = 0 作为硬验收 | 合规 |
| `files_owned` 重叠不得并行；单批并行 ≤ 3；跨 crate 默认串行 | 见 `02-tasks.md` §2 批次冲突校验 | 合规 |
| 未通过门禁不得推进下一阶段 | `GW-08`(G-A/G-B/G-C/G-D) 通过后才允许 `GW-09` | 合规 |
| 本次是否触碰架构敏感面 | `touches_runtime_lifecycle=false` / `touches_persistence=false` / `touches_cross_crate_deps=false` | 见信封 |

---

## 11. 风险与开放问题

| 编号 | 风险 | 等级 | 缓解 | 承载任务 |
|---|---|---|---|---|
| R1 | CI `test` job 因 trust_key 已知红必然失败，合并后首批 CI 结果会被误读为回归 | 高 | 合并**前**完成 Q2 裁决并预置白名单；在进度快照登记"预期红" | `GW-11`/`GW-12`/`GW-16` |
| R2 | 子代理通道 4 连败，T1 与文档校验可能再次空转 | 中高 | 每个可执行任务预置编排者代行；派发前预热；单任务最多 2 轮返工，第 3 次升级用户 | `GW-02`/`GW-03` |
| R3 | 985 文件批次在 main 上从未构建/测试，存在 main 侧内容回归 | 中 | 合并前跑全量 `-j 1` 门禁（G-B 逐名比对失败集） | `GW-08` |
| R4 | CI runner 规格未知，裸 `cargo test --workspace` 是否 SIGBUS 未知（`[假设-待验证] A2`） | 中 | `GW-16` 观察；若红于 SIGBUS 则给 test job 加 `-j 1`（并入 `GW-12`） | `GW-16` |
| R5 | 身份未结算导致历史作者不一致 / 无法提交 | 中 | `GW-01` 当日结算，G-D 作为每个写操作的前置 | `GW-01` |
| R6 | 0 tag + release 链路 secrets 未确认（`[假设-待验证] A5`） | 中 | 打 tag 前先确认 secrets；否则维持 0 tag 并记录 decision | `GW-13` |
| R7 | 合并后再发现 dev 侧缺陷，回滚成本 | 中 | `--no-ff` 提供单次 revert 回滚；锚点分支保留 | `GW-09` |

开放问题（需用户裁决，按紧急度排序）：

- **Q3 git 身份配置方式**（推荐 A：写入 git 配置）—— 阻塞所有写操作，Day0 必须裁决。
- **Q1 合并方式**（推荐 A：`--no-ff` merge commit）—— 阻塞 `GW-09`。
- **Q2 CI `test` job 已知红处置**（推荐 A：白名单守卫；B/C/D 分别触碰铁律或构成静默降级）—— 阻塞 `GW-12`。
- **Q4 是否打首个 tag**（`v5.0.9` 对齐 `Cargo.toml:30`）—— 阻塞 `GW-13`。
- **Q5 是否补 G6/G7/G8 的 CI job**（`AGENTS.md:220` 记载为既有缺口）—— 阻塞 `GW-17`。
- **Q6 是否解除 `AGENTS.md:278` 表述同步任务 T8 的 blocked**（当前默认不执行）—— 阻塞 `GW-18`。

---

## 12. 定期更新机制（摘要，落地见 `02-tasks.md` 的 `GW-14`）

- **更新频率**：每工作日 1 次（收尾时）；每个 P0/P1 任务完成时**即时**追加；每周一做一次全量复核。
- **更新人**：编排者(project-manager)（涉及用户授权事项由用户确认后由编排者代记）。
- **更新载体**：本 `01-plan.md` 的「修订记录」小节（方案层偏差）+ `02-tasks.md` 的「进度快照」小节（执行层状态）；两个小节**均只追加**。
- **修订规则**：计划与实际不符时，追加一行 `时间 / 偏差描述 / 原因 / 新决定 / 受影响任务 / 新截止时间`，**不改写既有行**；截止时间不可达时升级用户裁决并同步调整依赖链；连续 2 次偏差触发一次计划重排。

---

## 修订记录（只追加，不改写既有行）

| 时间 | 修订内容 | 依据 | 修订人 |
|---|---|---|---|
| 2026-09-03 | 初版冻结：18 个任务 / 9 个批次 / 6 个待裁决项 | 编排者 2026-09-03 Git 盘点事实包 + 只读勘查 | solution-architect |
| 2026-09-03 | **修订 1（§1.1 假设 A1 证伪）**：A1「`.codebuddy/artifacts/**` 已被 git 跟踪」**不成立**。实测 `.gitignore:131` 排除整个 `.codebuddy/`，该目录零文件被跟踪，导致 GW-04 闭板改动无提交内容、且文档 §9.1「随仓库共享给团队」为失实表述。已升级用户，裁决 **CB=解除整个 `.codebuddy/` 忽略**。据此 GW-04 的 `files_owned` 追加 `.gitignore`，提交粒度契约 §5.1 由 C1/C2/C3 扩展为 C1–C4(dev)+C5(merge) | `git check-ignore -v` / `git ls-files .codebuddy` 实测；用户 2026-09-03 裁决 CB | project-manager |
| 2026-09-03 | **修订 2（执行环境）**：`rustup` 于 09:18 被外部删除，`/root/.cargo/bin/cargo` 变悬空链接。全部构建/验证改用工具链绝对路径 `/root/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/{cargo,rustc,cargo-fmt}` + PATH 注入完成。§8 的命令示例在 rustup 恢复前需按此方式调用 | `/root/.cargo/bin` mtime=09:18；`ls /root/.cargo/bin/rustup` → No such file | project-manager |
| 2026-09-03 | **修订 3（Q1/Q2/Q3 裁决落地）**：Q1=`--no-ff`（→ `a81fb69f`，2 parent，`git diff --stat main dev` 空）；Q2=白名单守卫（→ `f3489055`，三场景本地等价验证）；Q3=写入仓库级 git config（非全局，沿用 `rustcode-builder`）。GW-01/02/03/04/08/09/10/12 全部 done；GW-15/16/17/18 与 GW-13 未执行，见 `STATUS.md` 未完事项 | 用户 2026-09-03 四项裁决；实测命令输出 | project-manager |
