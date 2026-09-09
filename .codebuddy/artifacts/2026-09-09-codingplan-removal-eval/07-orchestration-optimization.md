---
kind: optimization
id: OPT-001
from: orchestrator
feature: 2026-09-09-codingplan-removal-eval
status: applied
created: 2026-09-09
supersedes: 02-tasks.md §3 并行性矩阵（仅调度与验证右配，任务内容不变）
---

# OPT-001 多 Agent 编排优化：codingplan 移除流水线

> 优化对象：本 feature 的实施流水线（`02-tasks.md`，11 批次 / 22 任务）。
> 优化目标：在不改变冻结契约（`01-design.md` 删/改/不变三表）与不改任何任务**内容**的前提下，
> 压缩关键路径轮次、削减冗余验证、消除末期返工回路。

---

## 0. 结论速览

| 指标 | 现状 | 优化后 | 变化 |
|---|---|---|---|
| 关键路径门禁轮次 | **14** | **9** | **−35.7%** |
| 工作区级 `cargo check --workspace --all-targets` | ~11 次 | **4 次** | −64% |
| 全量 `cargo test -j 1 --workspace` | 2 次（T-10 / T-31） | 2 次 | 不变（必要） |
| 离关键路径的独立任务 | 0 | **5**（T-26/27/30/28/29） | 全部移出主链 |
| 末期一次性审查的返工半径 | 最大 9 批 | **最大 1 批** | 审查左移 R1/R2 |
| 未归属文件（会导致 T-31 返工） | **7 个（含 2 处真实代码）** | 0 | 已补归属 |

**最高价值发现不是提速，而是 3 个会在 T-31 才暴露的返工回路**（§2 的 C-1/C-2/C-3）。

---

## 1. 剖析：现状流水线的性能画像

### 1.1 实测成本锚（`03-impl/T-10.md` 实测值，非估算）

| 操作 | 实测耗时 |
|---|---|
| `cargo build`（default-members，增量） | 29.62s |
| `cargo build --workspace`（增量） | 4.58s |
| `cargo fmt --check` | <1s |
| `cargo test -j 1 --workspace --no-fail-fast` | **4m50s** |
| 基线规模 | 5488 passed / 1 failed / 11 ignored；`N_RS=1100` / 66 文件 / `N_EXT=65` / `N_DOC=117` |

结论：**全量测试是唯一的重型操作**；`cargo check --workspace` 在增量态下是廉价的（秒级）。
因此"每批都跑 workspace check"的成本其实不高，而"每批都跑 crate `--lib` 测试"与"人员确认往返"才是主成本。

### 1.2 关键路径分解（现状 14 轮）

```text
B1(1) → B2(1) → B3(T-13→T-14→T-15 = 3) → B4(T-16→T-17 = 2)
      → B5(1) → B6(T-21→T-22 = 2) → B7(1) → B8(1) → B9∥B10(1) → B11(1)   = 14
```

三种串行来源：

1. **符号因果**（必须）：L3 → L2 → L1 生产方/消费方链（2→3→4→5→6→7→8）。
2. **字段耦合**（必须，但可合并为单任务）：`footer_usage` 同时出现在
   `state.rs`(T-14) / `event_loop/mod.rs`(T-15) / `event_loop/commands.rs`(T-13) —— 三任务分属三批内串行，
   本质是**同一个字段的三处引用**，拆成 3 个 Agent 轮次只增加了协调成本。
3. **调度惯性**（可消除）：B9/B10 与 Rust 主体**无编译依赖**，却被排在 B8 之后。

### 1.3 依赖方向核对（决定能否重排）

实测（`grep is_codingplan_gateway`）：

```text
rustcode-auth::gateway_crypto::is_codingplan_gateway  (L2)
        └─> rustcode_config::endpoints::is_codingplan_llm_gateway   (L1)   # gateway_crypto.rs:100-101
rustcode-capabilities::provider::is_codingplan_gateway             (L2 复刻)
        └─> gateway_crypto::is_codingplan_gateway                  # codingplan_sign.rs:97
```

推论：**L2 依赖 L1，故必须先删 L2（T-21）再删 L1（T-23）** —— 现状顺序正确，不可反转。
但这也意味着 T-23 的真实依赖是 **{T-19, T-21}**，而不是 `{T-19}`；它可与 T-22 **并行**（两者文件不相交：
T-22 = `coding/src/{provider_factory.rs,lib.rs}`，T-23 = `config/src/{endpoints.rs,config/mod.rs,...}`）。
→ 原 B7 可并入 B6b，**省 1 轮**。

---

## 2. 瓶颈与缺陷（按返工代价排序）

### C-1 [高危] 7 个文件的 codingplan 命中未被任何任务 `files_owned` 覆盖

`02-tasks.md` 的 `files_owned` 与实测 per-file manifest 对账后，以下文件**无归属**：

| 文件 | 命中 | 性质 | 判定 |
|---|---|---|---|
| `crates/rustcode-daemon/src/kernel_runtime.rs` | 2（:177-179） | **真实代码**：`#[cfg(feature="codingplan")] rate_limit_source: Some(crate::coding_plan_rate_limit_source())` 与 `#[cfg(not(..))] rate_limit_source: None` | **补入 T-11** |
| `crates/rustcode-daemon/src/api_auth.rs` | 3（:289-296） | **真实代码**：`#[cfg(feature="codingplan")] crate::api_codingplan::sync_codingplan_after_login(...)` + neutral 孪生 `let _ = client_mode;` | **补入 T-11** |
| `crates/rustcode-daemon/src/live_api.rs` | 2（:291/:2125） | 注释 | 补入 T-11（顺带中立化） |
| `crates/rustcode-tuix/src/event_loop/oauth_poll.rs` | 4（:27/43/49/90） | 注释（含 `/codingplan` 与 `pending_run_codingplan` 字样） | 补入 B3' |
| `crates/rustcode-kernel/tests/rate_limit.rs` | 2（:313/:370） | 注释 | 补入 T-24 |
| `crates/rustcode-auth/src/oauth.rs` | 2（:604/:679） | 注释 | 补入 T-21 |
| `crates/rustcode-capabilities/src/provider/openai_compat.rs` | 2（:3615/:3619） | 测试夹具字符串 `"user has no codingplan"`（模拟服务端 403 body，与 codingplan 模块无关） | **白名单保留**（AC-6 例外登记）或改中性串；建议保留并登记 |

**为什么是高危**：两处 daemon 真实代码都在 `#[cfg(feature="codingplan")]` 块内。删掉 feature 定义后这些块
**永不编译 → 不会编译红**，因此 T-11 的验证命令全部通过，缺陷会一路潜伏到 T-31 的
`grep -rniI "codingplan" crates/ | wc -l`（期望 4）才暴露，届时需在已提交 9 批之上补提交 + 重跑终验。

**处置**：已写入 §4 的 P2 批次（T-11 扩权）+ P3/P6/P7 批次（注释类）。

### C-2 [高危] T-10 未采集 clippy 基线，T-31 的 clippy 比对不可判定

`02-tasks.md:479` 要求 T-31「`cargo clippy --workspace --all-targets` 与基线逐条比对，新增 0」，
但 T-10 的命令清单（`:56-64`）**没有 clippy**，基线缺失 → 该 AC 到 T-31 时无法判定。

**处置**：已在剖析阶段后台补采（`/tmp/t10-clippy-baseline.log`），落盘后归档为
`03-impl/T-10-clippy-baseline.txt`。**新增 T-32**（见 §4 P1）。

### C-3 [中] 审查集中在末期，返工半径最大 9 批

现 `STATUS.md:49` 的 T-04 审查是一次性的、位于全部实施之后。若 R 阶段发现
「删除面偏离冻结契约」（例如多删了 `Msg::DaemonProvManagedReserved`），返工会跨越 9 个批次。

**处置**：审查左移，插入两个**非阻塞只读**审查点 R1 / R2（§4），与后续批次实施并行（只读Agent不改文件，无冲突）。

### C-4 [中] B3 三任务拆得过细，且共享同一字段

见 §1.2(2)。合并为单任务 B3' 后，**3 轮 → 1 轮**，且消除了三 Agent 对 `footer_usage` 的协作约定。

### C-5 [中] B4 两任务必须原子，拆分无收益

T-16（删 4 个模块）与 T-17（摘依赖边 + 删 crate）本就串行且必须同提交（拆开必编译失败）。
合并为 B4' 单任务单 commit，**2 轮 → 1 轮**。

### C-6 [低-中] B9/B10 被错误地放在关键路径上

- T-26（VS Code，`tsc` 验证）与 T-27（JetBrains，源码级）与 Rust 侧**零编译依赖**：
  删掉 daemon 路由只会让旧扩展运行时 404，不会让 TS/Kotlin 编译失败。
- T-30（`docs/`）与代码完全无关，可立即开工。
- T-28（webui）只依赖 T-18 删除 `requires_login` 字段 → P5 之后即可，不必等 B8。

**处置**：全部移出主链（§4 的 F 组）。

### C-8 [中] `02-tasks.md:512` 声称「T-11 ∥ T-12 可并行」不成立

`cli/Cargo.toml` 的 `codingplan` feature 引用 `rustcode-daemon/codingplan`。T-11 删掉 daemon 的 feature 定义后，
在 T-12 落地前，**任何触及 cli 的 cargo 命令都会在 manifest 解析期报**
「feature `codingplan` in dependency `rustcode-daemon` does not exist」—— 即 T-12 在 T-11 落地前**无法自验**。

**处置**：合并为 **T-11'**（单 Agent、单 commit）。轮次数不变（仍是 1 轮），但消除了"先跑的 Agent 无法验证"的伪并行。

### C-7 [低] 每个 Agent 重复探索同一份命中清单

22 个任务各自重新 grep 定位命中点。已一次性生成共享 manifest（§5），后续派工直接引用行号，
预计每个 Agent 节省 3~6 次工具调用。

---

## 3. 优化策略

| 编号 | 策略 | 手段 | 收益 |
|---|---|---|---|
| S-1 | 合并同字段耦合任务 | T-13+14+15 → B3'；T-16+17 → B4' | −3 轮 |
| S-2 | 重排可并行任务 | T-23 并入 B6b（与 T-22 并行） | −1 轮 |
| S-3 | 移出无依赖任务 | T-26/27/30/28 → F 组离链；T-29 与 T-25 并行 | −1 轮 |
| S-4 | 验证右配 | 轮内只跑受影响 crate；workspace check 仅在 P4/P7/P8/P10 | check 次数 −64% |
| S-5 | 审查左移 | R1（P4 后）、R2（P8 后）非阻塞只读 | 返工半径 9 批 → 1 批 |
| S-6 | 上下文复用 | 共享 manifest + 每批 8 行固定摘要 | 每 Agent −3~6 次调用 |
| S-7 | 覆盖对账前置 | manifest vs `files_owned` 求差集，开工前补归属 | 消除 C-1 类末期返工 |
| S-8 | 并发车道 | 用既有双车道（worker=3 / explore=8，`AGENTS.md` OBJECTIVE-6）跑 P5/P7/F 组 | 轮内墙钟不随任务数线性增长 |

**未采用**：
- 「一次性合成 1 个 commit」——回滚粒度不可用，否决。
- 「把 B5 与 B6 合并」——B5 是"停止使用 L2 符号"、B6 是"删除 L2 本体"，合并后任一 crate 编译红都无法定位归属，且违反 `01-design.md` 的中间态可编译原则。否决。

---

## 4. 优化后的调度（supersedes `02-tasks.md` §3）

### 4.1 关键路径（9 轮）

| 轮 | 内容 | 任务 | 并发 | 轮末验证 |
|---|---|---|---|---|
| **P1** | 补 clippy 基线 | T-32（新增） | 1 | 日志归档；`CLIPPY_EXIT` 与告警清单留档 |
| **P2** | L3 网络侧（daemon + cli），**单 Agent 单 commit** | **T-11'**（= T-11 + T-12 合并，见 C-8） | 1 | `cargo check --workspace --all-targets`；`-p rustcode-daemon --lib`；`-p rustcode --lib -- completion`；两条 feature 不存在断言；`grep -rn codingplan crates/rustcode-daemon/src crates/rustcode-cli/src \| wc -l` = 2（`uninstall/paths.rs` 白名单） |
| **P3** | tuix 消费点整体剥离（合并原 T-13/14/15，+`oauth_poll.rs`） | **T-13'**（单 Agent，8 文件） | 1 | `cargo check -p rustcode-tuix --all-targets`；`cargo test -j 1 -p rustcode-tuix --lib`；`grep -rniI codingplan crates/rustcode-tuix/src/lib.rs state.rs event_loop/commands.rs render sanitize.rs` = 0 |
| **P4** | tuix 模块删除 + 摘依赖边 + 删 crate + `Cargo.lock`（**原子单 commit**） | **T-16'**（= T-16 + T-17） | 1 | `cargo check --workspace --all-targets`；`cargo metadata > /dev/null`；`grep -n rustcode-codingplan Cargo.lock` = 0；`cargo check -p rustcode-codingplan` 报 package 不存在 |
| **R1** | **只读审查点**（与 P5 并行，不阻塞） | code-reviewer | 1 | 产出分级问题清单；`changes_requested` 则 P5 暂停 |
| **P5** | L2 消费点先改 | T-18 ∥ T-19 ∥ T-20 | 3 | 每 crate `check --all-targets` + `--lib`；三个 `grep` 归零断言 |
| **P6** | L2 生产方删除 + L4 闭源桩 | T-21（+`auth/src/oauth.rs` 注释） | 1 | `cargo check --workspace --all-targets`；auth/capabilities `--lib`；crypto feature 不存在断言 |
| **P7** | L2 收尾 + L1 生产方删除（原 B7 并入） | T-22 ∥ **T-23** | 2 | `cargo check --workspace --all-targets`；`config --lib`（含 4 个新增测试）；`coding --lib` |
| **P8** | i18n 三件套 ∥ `AGENTS.md`（原 B10 拆分，与 T-25 并行） | T-25 ∥ **T-29** | 2 | `cargo check --workspace --all-targets`；`config --lib` 全绿；`python3 scripts/check-zh-docs.py gate` exit 0 |
| **R2** | **只读审查点**（与 P9 并行，不阻塞） | code-reviewer | 1 | 复核 i18n 三语 arm parity 与 AGENTS.md 条文一致性 |
| **P9** | 全量验证与交付 | T-31 | 1 | AC-1…AC-15 全部命令；clippy 与 P1 基线比对新增 0；失败集 ⊆ T-10 基线 |

### 4.2 离关键路径（F 组，任意时点并行，用各自工具链）

| 任务 | 最早可开工 | 依赖 | 说明 |
|---|---|---|---|
| **T-30** `docs/` | **立即** | 无 | 与代码零耦合 |
| **T-27** JetBrains | **立即** | 无 | 无 JDK/gradle，仅源码级验证 |
| **T-26** VS Code | **立即** | 无 | `tsc --noEmit` + `test:webview`（基线 11/11） |
| **T-28** webui | P5 之后 | T-18（删 `requires_login`） | `npm ci && build` + `cargo clean -p rustcode-daemon` |

> F 组不与 Rust 主体争用 `target/` 锁（`cargo` 会串行化，但前端任务不调 cargo，T-28 的 `cargo clean` 需错开 P6/P7 的 check）。

### 4.3 必须整批 squash 的组合（回滚原子性）

| 轮 | 必须同 commit | 理由 |
|---|---|---|
| P2 | T-11 + T-12 | T-12 删 `cli/Cargo.toml` 的 `rustcode-daemon/codingplan` 传递引用；若 T-11 未同步删掉 daemon 的 feature 定义 → workspace 解析失败 |
| P4 | T-16' 全部 | 删模块 + 摘依赖边 + 删 crate + `Cargo.lock` 分离即编译失败 |
| P8 | T-25 三件套 | 穷尽 match，缺一语即编译失败 |

### 4.4 默认策略（前次提问未获答复，按推荐项执行，如不同意请覆盖）

- **提交**：每轮 1 个 commit，**不 push**；P2/P4/P8 按 §4.3 squash。
- **中断**：任一轮编译红且一轮返工未修复 → **立即停下汇报**，保留现场不提交。
- **回滚**：反向 `git revert`（P9 → P1），§4.3 三组整批回滚。

---

## 5. 共享 manifest（S-6，已生成）

`crates/` 下 per-file 命中数（降序，仅列 >0）：

```text
setup.rs(193)  tuix/event_loop/commands.rs(99)  tuix/event_loop/mod.rs(83)  config/config/mod.rs(77)
tuix/event_loop/monitor.rs(59)  daemon/api_codingplan.rs(57)  config/i18n/zh_cn.rs(40)
config/endpoints.rs(40)  config/i18n/en.rs(39)  codingplan/types.rs(38)  tuix/modals/onboarding_wizard.rs(32)
daemon/lib.rs(27)  cli/main.rs(26)  config/i18n/messages.rs(24)  coding/rate_limit.rs(20)
codingplan/client.rs(19)  tuix/modals/provider_panel.rs(18)  auth/gateway_crypto.rs(17)
daemon/commands.rs(14)  daemon/api_config.rs(14)  tuix/event_loop/usage_monitor.rs(12)
daemon/runtime_host.rs(11)  cli/Cargo.toml(11)  tuix/modals/usage.rs(9)  coding/provider_factory.rs(8)
daemon/api_provider.rs(7)  coding/config.rs(7)  codingplan/lib.rs(7)  capabilities/provider/codingplan_sign.rs(7)
tuix/lib.rs(6)  tuix/Cargo.toml(6)  daemon/Cargo.toml(6)  coding/parts.rs(5)  codingplan/sync_marker.rs(5)
tuix/event_loop/oauth_poll.rs(4)*  tuix/render/retained.rs(3)  tuix/render/mod.rs(3)  daemon/api_auth.rs(3)*
codingplan/Cargo.toml(3)  capabilities/provider/mod.rs(3)  tuix/commands.rs(2)  kernel/tests/rate_limit.rs(2)*
kernel/hook.rs(2)  kernel/event.rs(2)  daemon/live_api.rs(2)*  daemon/kernel_runtime.rs(2)*
coding/lib.rs(2)  clix/main.rs(2)  cli/uninstall/paths.rs(2)†  capabilities/provider/openai_compat.rs(2)†
auth/oauth.rs(2)*  auth/Cargo.toml(2)  余 1 命中：tuix/sanitize.rs  tuix/render/qr.rs  tuix/render/cell.rs
kernel/agent.rs  config/tls.rs  config/config/provider.rs  coding/skill_first.rs  coding/runtime.rs  coding/persona.rs
```

图例：`*` = 原任务图未归属（C-1，已在 §4 补入）｜`†` = 白名单保留（`uninstall/paths.rs` 按 Q6 裁决；`openai_compat.rs` 为 403 测试夹具串）。

合计 `N_RS=1100` / 66 文件，与 `03-impl/T-10.md` §5.1 一致。

---

## 6. 验证矩阵（替代 `02-tasks.md` 各批的验证命令）

| 层级 | 命令 | 何时跑 |
|---|---|---|
| 每轮必跑 | `cargo fmt --check`（<1s） | P1–P9 |
| 轮内 | `cargo check -p <受影响 crate> --all-targets` + `cargo test -j 1 -p <crate> --lib` | P2/P3/P5/P6/P7 |
| 关卡 | `cargo check --workspace --all-targets` | **P2 / P4 / P7 / P8 / P9**（原为每批） |
| 终验 | `cargo build && cargo build --workspace` | P9 |
| 终验 | `cargo clippy --workspace --all-targets`（与 P1 基线逐条比对，新增 0） | P9 |
| 终验 | `cargo test -j 1 --workspace --no-fail-fast`（失败集 ⊆ T-10 基线 1 条） | P9 |
| 终验 | AC-4 七条 feature/package 不存在断言 + AC-6 残留 grep + AC-7/8 completion + `check-zh-docs.py gate` | P9 |

> `-j 1` 强制（`AGENTS.md:549`，cgroup 8GB）；日志路径每轮唯一（`AGENTS.md:561`）。

---

## 7. 风险与缓解（优化引入的新风险）

| 风险 | 说明 | 缓解 |
|---|---|---|
| 合并后单 Agent diff 过大（P3 约 8 文件 / 180+ 命中） | 长上下文下易漏臂 | 用 manifest 精确给行号；P3 结束即跑 workspace check（唯一一个额外加测的轮次） |
| T-25 ∥ T-29 并行 | `AGENTS.md` 按"计划终态"书写，若 T-25 实际删除面偏离则条文失真 | P9 增加一条复核：`grep -n "codingplan\|CodingPlan" AGENTS.md` 与实际删除面逐条对账 |
| F 组与主体争用 cargo 锁 | T-28 的 `cargo clean -p rustcode-daemon` 会让后续 check 重链 | 把 T-28 安排在 P7 之后、P9 之前单独窗口 |
| 审查左移的 R1/R2 若与实施轮并发 | 只读，但读的是"刚提交的上一轮"，若实施轮已改动同文件则结论过期 | R1 只审 P4 提交范围；R2 只审 P8 提交范围；均不跨越未提交改动 |

---

## 8. 未验证范围（沿用 `01-design.md`）

- JetBrains 侧**无 JDK/gradle**，只能源码级验证（`AGENTS.md:318`），不声称编译通过。
- `cargo clippy` 基线正在后台补采，若日志中 exit≠0 需先判定是否属存量告警，再写入 P1。
- 本优化只改调度与验证配给，**未做任何生产代码改动**。
