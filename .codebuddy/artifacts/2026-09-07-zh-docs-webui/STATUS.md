# 2026-09-07-zh-docs-webui 看板

- 当前阶段：**已完成**（G6 `pass`；T-15 / T-16 已 `done`，R2 返工的 3 条提交判据中第 1、2 条已由本轮 doc-writer 闭环；R3 提交口径**已裁决** = 全部提交含 artifacts；**待 PM 执行 commit**）
- 基线：branch=`dev` commit=`3ee655e381d28052e428196daedd456cc6079520` worktree=**staged（未 commit）**
  - `git status --porcelain | wc -l` = **233**
    - 索引内（staged）**230**：`M` 159 / `A` 64 / `AM` 4 / `MM` 2 / `D` 1
    - 未跟踪 **3 个条目**（展开为 7 个文件）：`04-review/REVIEW-T15-T16.md`、`.codebuddy/memory/2026-09-09.md`、`.codebuddy/teams/`（目录，内含 5 个 json）
  - `git diff --cached --name-only | wc -l` = **230**；`git diff --name-only | wc -l` = **8** —— 索引后又改、需 PM 在 commit 前 `git add` 的 8 个：`03-impl/T-15.md`（§10 哨兵）、`03-impl/T-16-artifacts-exempt.md`（§9 更正）、`crates/rustcode-cli/src/main.rs`、`scripts/check-zh-docs.py`（以上为 R2 追加，非本轮产物）＋ 本轮 doc-writer 的 `00-decisions.md`（Q6）、`STATUS.md`、`06-release.md`、`AGENTS.md`（`:37`）；另需 `git add` 未跟踪的 `04-review/REVIEW-T15-T16.md`（另外 2 个未跟踪条目属 agent 运行时目录，建议**不**入库）
  - `git rev-parse HEAD` = `3ee655e381d28052e428196daedd456cc6079520` —— **HEAD 仍等于基线**，未 commit
  - 删除项：`D  README.zh-CN.md`（已落索引，未 commit）
  - `.codebuddy/memory/` 与 `.codebuddy/teams/` 已按 T-16 §7-B 用 `git restore --staged` 移出索引（**工作区文件保留**），属 agent 运行时目录，非本 feature 交付物
- 本看板数字由 doc-writer 于 2026-09-09 复跑实测，命令均可原样复现
- **数字口径已按 `00-decisions.md` Q6 变更**：AC-1 分母 = 全仓已跟踪 md 减显式豁免域 `.codebuddy/artifacts/`，故 285 → **248**、全量受检 231 → **194**

## 目标（用户已裁决，不变）

1. 全仓 markdown 汉化（168 个纯英文文件）
2. 汉化 `README.md`，删除重复的 `README.zh-CN.md`
3. WebUI 开箱可访问：构建脚本一键化 + 资源缺失可执行修复指引
4. 默认绑定 `0.0.0.0`，且不再打印非回环安全警告

## 实测现状（doc-writer 2026-09-09 复跑，口径按 Q6）

| 项 | 实测 | 命令 |
|---|---|---|
| 受检 md（AC-1 分母） | **248** = SKIP_A 4 + SKIP_B 47 + SKIP_C 1 + SKIP_D 2 + TODO 27 + ZH 167；恒等式 OK。**已排除 artifacts 豁免域**（旧口径全仓 285 = TODO 29 + ZH 202，差值 37 为基线已入库的历史交接件，见 `00-decisions.md` Q6） | `python3 scripts/check-zh-docs.py inventory --base 3ee655e3` |
| 分母自洽式 | `git ls-files -- '*.md'` = **349**；域内 `.codebuddy/artifacts/` = **101**（本 feature 64 + 历史 37）；349 − 101 = **248** | `git -c core.quotePath=false ls-files -- '*.md' \| grep -c '^\.codebuddy/artifacts/'` |
| 纯英文文件 | **0**（TODO = EN 0 / MIXED 27，均为代码密集文档，AC-2 实测通过） | 同上（inventory） |
| 英文散文残留 | 全量 check **受检 194 / FAIL 0** | `python3 scripts/check-zh-docs.py gate --base 3ee655e3` |
| 门禁 | **PASS**（`GATE_EXIT=0`） | 同上 |
| 排除项 | `gate` 首行两段提示：**未跟踪 md 已排除 1 个**（`.codebuddy/memory/2026-09-09.md`，agent 运行时记忆，非交付物）；**artifacts 豁免域已排除 101 个**（与 SKIP_A…SKIP_D 同级，不进分母、不受 AC-2/4/6/7b/32 与 AC-8 段 1 约束，仍是 AC-8 段 2 历史域） | 同上 |
| AC-4 授权放行 | 合计 **12 条（D1 11 / D2 1）**，无未授权差异 | 同上 |
| AC-8 三域 | 正式域 **0 命中**（候选 515，已排除 artifacts 域） / 历史域 **270 处，全部位于 `.codebuddy/artifacts/`**（候选 101） / 兜底域 **0 命中**。历史域命中数随域内交接件增长而增长（37/39 → 101/270），判定与退出码不变 | 同上 |
| 已改 md | **157** 个，`+8684 / -9840`（**汉化批次口径**，不含交接件）。若按 `git diff 3ee655e3 -- '*.md'` 全量口径（含本 feature 已入库的交接件 md）则为 **221 个文件 / +27714 / -9840**；两个数字口径不同，均实测 | `git diff --shortstat 3ee655e3 -- '*.md'`；批次构成见 `06-release.md` §5 |
| 已改源码 | 5 个：`crates/rustcode-cli/src/main.rs`、`crates/rustcode-config/src/i18n/{messages,en,zh_cn}.rs`、`crates/rustcode-daemon/src/lib.rs` | `git status --porcelain` |
| 本 feature 新增文件（**已入索引**，不再是未跟踪） | `scripts/check-zh-docs.py`、`scripts/build-webui.sh`、`crates/rustcode-daemon/tests/default_host_lock.rs`、`crates/rustcode-config/tests/cli_webui_i18n_lock.rs` | `git diff --cached --diff-filter=A --name-only` |
| hostscan | 命中文件 **6** / 命中行 **9**，**实证判定全部保留（no-op）**，恒返回 0 | `python3 scripts/check-zh-docs.py hostscan --base 3ee655e3` |
| `README.md` | `en=0/374=0.0000`（基线 `388/409=0.9487`） | `check --base 3ee655e3 --files README.md` |
| `AGENTS.md:37`（T-15 后修订） | 由「`rustcode daemon` 子命令没有 `--host` 参数」改为如实记载「同样支持 `--host`、默认 `0.0.0.0`、可显式传 `127.0.0.1` 退回仅本机」；**0 新增 code span**（不在 `AC4_ALLOWED_ADDED_BY_FILE`），`check --files AGENTS.md` **PASS / exit 0** | `python3 scripts/check-zh-docs.py check --base 3ee655e3 --files AGENTS.md` |
| 未提交改动数 | **233**（索引内 230 + 未跟踪 3 个条目）。`git add -A` 后由 169 增长而来：本 feature 交接件 md 全部入库，同时被 `ARTIFACTS_EXEMPT_PREFIX` 出域，故门禁数字不变 | `git status --porcelain \| wc -l` |

## 门禁

| 门禁 | 状态 | 依据 |
|---|---|---|
| G1 需求 | **pass** | `00-requirement.md`（REQ-002, approved）+ `00-decisions.md`（Q1–Q5 + **Q6** = AC-1 分母与 artifacts 豁免域裁决） |
| G2 设计 | **pass** | `01-design.md` + `02-tasks.md` + **`01-design-addendum.md`**（AC-4 判据重写 / AC-8 三段式 / 编号方案）+ **`01-design-addendum-d5.md`**（T-15 的 `--host` 契约，approved） |
| G3 实现 | **done** | `03-impl/*`：T-01…T-06、T-08、T-09、T-11、T-13、T-14、**T-15**（`rustcode daemon` 补 `--host`）、**T-16**（artifacts 提升为门禁豁免域）与 D-01…D-36 全部落盘 |
| G4 审查 | **approved** | `04-review/CODE-REVIEW.md`（裁决 `request-changes` → 返工）+ `03-impl/T-13-r1-rework.md`（R1 返工 8 条全部核实属实）+ **`04-review/REVIEW-T15-T16.md`**（R2：T-15 **approve**；T-16 `request-changes` 的 major-1/2 已由 `T-16-artifacts-exempt.md` §9 闭环，major-3 = `AGENTS.md:37` / 本看板 D-5 行已由本轮 doc-writer 闭环；剩余 2 条 minor 转遗留、不阻塞） |
| G5 测试 | **pass** | `05-test-report.md`（AC **29 PASS / 2 部分验证 / 2 未验证 / 0 FAIL**）+ `03-impl/zh-check-gate.md`（`gate --report` 产出）+ T-15 §10（`default_host_tests` **3 passed**） |
| G6 交付 | **pass / done** | `06-release.md`（四段式完整，本轮补 D-5/T-15 与 T-16 两节）+ `03-impl/zh-check-D-34a.md` + `03-impl/zh-check-D-35.md` + T-10b `AGENTS.md` 同步；`gate --base 3ee655e3` 复跑 **PASS / exit 0**；**无悬挂任务，唯一悬挂动作 = 待 PM 执行 commit** |

## 任务

| id | 标题 | 批次 | 负责 Agent | 状态 | 返工轮次 | 交接件 |
|---|---|---|---|---|---|---|
| T0 | 需求分析与 AC 定义 | - | requirements-analyst | done | 0 | `00-requirement.md` |
| T-01 | 汉化验收脚本 `check-zh-docs.py` | 0A | code-implementer | done | 0 | `03-impl/T-01.md` + `03-impl/md-inventory.md` |
| T-02 | `scripts/build-webui.sh` 一键构建 | 0A | code-implementer | done | 0 | `03-impl/T-02.md` |
| T-03 | 默认 host 改 `0.0.0.0`（CLI 两处） | 0A | code-implementer | done | 0 | `03-impl/T-03.md` |
| T-04 | 删除非回环警告 + 修正失实注释 | 0B | code-implementer | done | 0 | `03-impl/T-04.md` |
| T-05 | i18n 文案 4 条 | 0B | code-implementer | done | 0 | `03-impl/T-05.md` |
| D-01…D-33 | 汉化批 1–11 | 1–11 | doc-writer | done | 0 | `03-impl/zh-check-D-01.md` … `zh-check-D-33.md` |
| D-34a | setup-seeds 2 件（`SKILL.md`、`references/hooks-patterns.md`） | 12 | doc-writer | **done（报告本轮补齐）** | 0 | `03-impl/zh-check-D-34a.md`（本轮新增）+ `zh-check-D-34b.md` |
| D-34b | setup-seeds 4 件（`references/` 其余） | 12 | doc-writer | done | 0 | `03-impl/zh-check-D-34b.md` |
| D-35 | 根 `README.md` 汉化 + 删 `README.zh-CN.md` | 13 | doc-writer | **done（报告本轮补齐）** | 0 | `03-impl/zh-check-D-35.md`（本轮新增） |
| D-36 | 零星 9 件 | 13 | doc-writer | done | 0 | `03-impl/zh-check-D-36.md` |
| T-06 | hostscan：`127.0.0.1` 叙述处置 | 14 | doc-writer | done（**no-op**，9 行全部判定保留） | 0 | `03-impl/T-06-hostscan.md` |
| T-08 | 门禁脚本修正（SKIP_D 等；原误称 T-06S） | 14 | code-implementer | done | 0 | `03-impl/T-06-gate-script.md` |
| T-09 | 按补遗实施 AC-4 判据重写 + AC-8 三段式 | 15 | code-implementer | done | 0 | `03-impl/T-09.md` + `03-impl/zh-check-T-09.md` |
| T-11 | 文档侧 AC-4 回退（`README.md`、`docs/phase1-refactor-design.md`） | 15 | doc-writer | done | 0 | `03-impl/T-11.md` |
| T-07 | 全量门禁 + 集成/冒烟验收（G5） | 15 | test-engineer | **done** | 0 | `05-test-report.md` + `03-impl/zh-check-gate.md` |
| T-13 | R1 返工收尾（MAJOR-1/2/3、MINOR-1/2/4/5、NIT-1/2） | 16 | code-implementer | done | 1 | `03-impl/T-13-r1-rework.md` |
| T-14 | 修 `is_loopback_authority` 的 IPv6 回环误报 | 17 | code-implementer | done | 0 | `03-impl/T-14-loopback-ipv6.md` |
| **T-15** | `rustcode daemon` 补 `--host` 参数（缺陷 **D-5**） | 19 | code-implementer | **done**（G4 `approve`） | 1（R2 补哨兵） | `03-impl/T-15.md`（契约 `01-design-addendum-d5.md`）；`main.rs` +7/−1（4 hunk）+ R2 哨兵 +32；`default_host_tests` 2 → **3 passed**、`rustcode-daemon` **316/0**、`gate` exit 0 |
| **T-16** | `.codebuddy/artifacts/` 提升为门禁豁免域（`ARTIFACTS_EXEMPT_PREFIX`） | 19 | code-implementer | **done**（R2 已闭环） | 1（R2 报告更正 + 措辞） | `03-impl/T-16-artifacts-exempt.md`（§9 含 PM 方案 A 裁决）；分母 285 → **248**、受检 231 → **194**；`git add -A` 后 `gate` 仍 exit 0 |
| **T-10** | 补 D-34a / D-35 报告 + `06-release.md`（`AGENTS.md` 同步拆出为 T-10b） | 16/18 | doc-writer | **done** | 0 | `06-release.md` + `03-impl/zh-check-D-34a.md` + `03-impl/zh-check-D-35.md`；`AGENTS.md` 同步**不在本条 `files_owned`**，已由 T-10b 落地 |
| **T-10b** | `AGENTS.md` 同步收尾（补 `scripts/build-webui.sh` 与默认绑定 `0.0.0.0`） | 18 | doc-writer | **done** | 0 | `AGENTS.md` **纯新增 7 行**（17 / 28 / 36-39，0 删除）；`gate --base 3ee655e3` **PASS / exit 0**；记录见文末「T-10b AGENTS.md 同步记录」 |
| T-12 | **编号空置**：全量 grep `.codebuddy/artifacts/2026-09-07-zh-docs-webui/**` 对 `T-12` 为 **0 命中**，无派单、无交接件 | - | - | **n/a（不存在该任务）** | - | 无 |

> 编号说明（`01-design-addendum.md` §3）：`T-06` 归 hostscan；实现期新增任务从 `T-08` 顺序分配，`T-08` = 门禁脚本修正、`T-09` = 补遗脚本实施、`T-10` = 补报告 + `06-release.md`、`T-11` = 文档侧 AC-4 回退；其后按实际派单续分为 `T-13`（R1 返工）、`T-14`（IPv6 回环）、`T-15`（`rustcode daemon` 补 `--host`，缺陷 D-5）、`T-16`（artifacts 门禁豁免域）。历史文件名一律不重命名。

## 编排裁决记录（2026-09-08，保留不改）

1. **AC-4**：`README.zh-CN.md` 删除导致的 span 消失，按「removed 字面含已删文件名 + added 为最小剔除式输出」放行；`README.md` 指向 `./scripts/build-webui.sh` 的 D2 项按文件冻结登记（计数 1）。`AGENTS.md`、`docs/superpowers/plans/2026-05-29-webui.md` 直接放行；`README.md`（恢复被删的手工构建代码块）与 `docs/phase1-refactor-design.md`（改回最小剔除式）走文档侧回退。判据强度不降：任何无因果的 span 丢失仍 FAIL。
2. **AC-8**：正式域 0 命中 + 历史域（`.codebuddy/artifacts/`）反向断言「必须仍有命中」+ 兜底域不变；禁止目录豁免外溢。
3. **hostscan**：R1 历史快照 3 行、R3/R4 实证保留 6 行，R2 = 0，故零改动（`docker-compose.yml:44` 宿主侧显式写死 `127.0.0.1`，实证）。
4. **编号**：`T-06` 归 hostscan；新增从 `T-08` 顺序分配。

补充裁决（2026-09-09，G4/G5 后）：

5. **MINOR-3 不本轮改**：`lib.rs:5264-5265` 与 `lib.rs:5442-5443` 两处「VSCode daemon `enforce_token=false`」注释与 `daemon/src/main.rs:173` 矛盾，属**既存**且不在本 feature `files_owned`，已裁决登记 TODO、不本轮改（见下节第 2 条）。
6. **G5 结论**：`05-test-report.md` §8 判 `pass` 并明确「可以推进 G6」，但要求 D-1…D-4 与未验证项必须原样带入交付说明，不得表述为已验证 —— 已在 `06-release.md` §2.3 / §4 落实。

补充裁决（2026-09-09，T-15 / T-16 R2 后）：

7. **T-16 走方案 A（维持 `.codebuddy/artifacts/` 整前缀豁免）**：`ARTIFACTS_EXEMPT_PREFIX` 不收窄、`AC8_EXEMPT_PREFIX` 继续引用它（两常量不解耦）。代价（分母 285 → 248、受检 231 → 194）经契约层追认 —— 已由 doc-writer 在 `00-decisions.md` 登记为 **Q6**。复审提出的方案 B（解耦收窄，可保住 285 / 231）**未被采纳**，理由见 Q6。
8. **R3 提交口径 = 全部提交（含 artifacts）**：用户已裁决；T-16 已使 `git add -A` 后 `gate` 仍 exit 0，`00-decisions.md` Q6 已登记分母口径 ⇒ 提交前置条件全部满足，剩余动作仅 PM 执行 commit。

## 待用户裁决

1. **R3 · 提交口径 —— 已裁决（2026-09-09，用户选「② 全部提交，含 artifacts」），不再是悬挂裁决项**：
   - **裁决结果**：本 feature 改动**全部提交，含 `.codebuddy/artifacts/**` 交接件**。
   - **前置条件已具备**：T-16 已把 `.codebuddy/artifacts/` 提升为门禁豁免域（`ARTIFACTS_EXEMPT_PREFIX`），`git add -A` 后 `gate --base 3ee655e3` 仍 **exit 0 / PASS**（受检 194 / FAIL 0）；分母口径变更已由 `00-decisions.md` **Q6** 在契约层追认。
   - **当前状态**：索引内 230 项、未跟踪 3 个条目（含 `04-review/REVIEW-T15-T16.md` 与两个 agent 运行时目录），另有 4 个 R2 追加文件未重新 stage；**待 PM 执行 commit**（本轮 doc-writer 不执行 `git add` / `git commit`）。
   - 原候选 ①（不纳入提交）与 ③（提交但接受门禁红）**已作废**，保留仅作历史留档。
2. **`lib.rs` 的 `enforce_token` 注释矛盾（既存，已裁决不本轮改）**：`crates/rustcode-daemon/src/lib.rs:5264-5265` 与 `lib.rs:5442-5443` 称「VSCode 扩展自带的守护进程以 `enforce_token=false`（不带 token）在 13456 上工作」，与 `crates/rustcode-daemon/src/main.rs:173` 的 `webui_tokens: Some(token_store)` ⇒ `enforce_token=true` 直接矛盾（CODE-REVIEW MINOR-3）。修复不在本 feature `files_owned`，**建议 PM 单开一条 TODO**；已写入 `06-release.md` §4 遗留项第 8 条。
3. **IDE 暴露面（T-06 上报，既存）**：JetBrains 插件在**非 Windows 且无打包 daemon** 时回退到 CLI `rustcode daemon`（`RustCodeDaemonProcess.kt:61-63`），而该入口默认已改为 `0.0.0.0` ⇒ IDE 用户暴露面扩大。建议单开小任务让插件显式传 `--host 127.0.0.1`。
4. **T-10 的 `AGENTS.md` 同步项 —— 已闭环（T-10b，2026-09-09，不再待裁决）**：原待裁决内容（`01-design-addendum.md` §3 把「`AGENTS.md` 同步」列在 T-10 名下，而当时派单 `files_owned` 不含 `AGENTS.md`）已由 T-10b 单派单落地，`AGENTS.md` 现记载 `scripts/build-webui.sh` 与默认绑定 `0.0.0.0`（原 `0.0.0.0` / `build-webui` 均 0 命中）。**本条保留仅作历史留档，不再是阻塞项**；改动明细与验证输出见文末「T-10b AGENTS.md 同步记录」。

## 已知未验证 / 遗留（不阻塞 G6，须原样带到后续）

### 未验证的 AC（写明所需环境）

| AC | 未验证内容 | 所需环境 |
|---|---|---|
| **AC-22** | 局域网**另一台主机**对 `<LAN-IP>:<port>` 的 `/health` 与 `/`。本机以 LAN IP `172.24.0.2:13465` 自测已通过（200/200），**跨主机未验证** | 同一二层网络的第二台主机（或另一容器 / network namespace + veth） |
| **AC-23** | 浏览器同源 fetch **无 CORS 错误**；现有证据仅为静态（`lib.rs:1222-1227` `allow_origin(predicate(is_loopback_origin))`） | 真实浏览器（DevTools）+ 已完成 `?token=` → Cookie 交接的页面 |
| **AC-25** | 无 provider 时首页**无全屏阻断遮罩**、可打开设置对话框；替代证据为 `webui/src/app.tsx` grep `provider` 0 命中 | 真实浏览器渲染（Playwright/Puppeteer） |
| **AC-26**（UI 层） | 在**设置对话框 UI** 中新建 provider 并设为默认。**后端链路已实证**（`POST /providers` 201 → `POST /providers/<id>/default` 200 → `config.toml` 落盘 → `GET /providers` 回读一致），仅 UI 交互未验证 | 真实浏览器 |

### G5 登记的缺陷（D-1…D-5，均不阻塞）

| 编号 | 一句话 | 归属 |
|---|---|---|
| **D-1** | `is_loopback_authority("::1") == false`，裸 `::1` / `::ffff:127.0.0.1` 误判为非回环 | T-14 已在提示路径局部收敛；本体交 **architect** |
| **D-2** | AC-8 字面「全仓 `README.zh-CN` 0 命中」不成立：实为 39 处命中，**全部**位于 `.codebuddy/artifacts/`，正式域 0 命中；按三域口径判 PASS | 口径登记，已写入 `06-release.md` §3 |
| **D-3** | `check-zh-docs.py` 的 AC-2 围栏剥离对**未闭合/嵌套围栏**失效（over-count，fail-safe 方向）。复现实例见 `03-impl/zh-check-D-34a.md` §5.1 | 归 **T-01** |
| **D-4** | `Msg::WebuiLanWarning` 在 `run_server` 路径下称「主地址为局域网 IP」，实际只打印 `0.0.0.0` | 待 **architect** 对 P1/P2/P3 裁决 |
| **D-5** | `rustcode daemon` 子命令**无 `--host` 参数** —— **已由 T-15 修复**：现支持 `--host`（`default_value = "0.0.0.0"`，可显式传 `127.0.0.1` 退回仅本机），G4 复审 `04-review/REVIEW-T15-T16.md` 判 **approve**；配套文档 `AGENTS.md:37` 与本行已同步 | **已闭环（T-15 / code-implementer）** |

### T-14 登记的遗留项

| 编号 | 一句话 | 归属 |
|---|---|---|
| **RI-1** | `ensure_server_and_open`（`lib.rs:5393` `open_host` 选择、`lib.rs:5422` 非回环提示）仍用旧谓词；`rustcode webui --host ::1` 仍误报且生成缺少方括号的非法 URL `http://::1:PORT/`（基线既有、不在本轮 `files_owned`） | architect（与 D-1 合并评估） |
| **RI-3** | 新谓词 `is_loopback_bind_host` 是**白名单式**：`0:0:0:0:0:0:0:1`、`127.0.0.2` 仍判非回环 → **少告警**；属 fail-safe 方向（漏提示不漏保护） | architect |
| **RI-4** | `is_loopback_bind_host` **无新增单元用例**，建议补 `::1` / `::FFFF:127.0.0.1` / `0.0.0.0` / `192.168.1.7` 四条表驱动用例 | test-engineer |

### 其它

- **改动仍全部未 commit**（索引内 230 / 未跟踪 3 个条目 / 未重新 stage 的 R2 追加 4 个）。R3 提交口径**已裁决**为「全部提交，含 artifacts」，交接件出域由 T-16 保障；**待 PM 执行 commit**，doc-writer 本轮不执行 `git add` / `git commit`。
- `.codebuddy/artifacts/**` 交接件：T-16 后由「未跟踪故排除」改为「**artifacts 豁免域**排除」（`gate` 首行第二段提示，当前 **101 个**）。域内文件不进 AC-1 分母、不受 AC-2/4/6/7b/32 与 AC-8 段 1 约束，**仍受 AC-7a 约束、仍是 AC-8 段 2 的历史域**（详见 `00-decisions.md` Q6）。显式 `--files` 仍可单点检。
- **`SKILL.md` 未闭合围栏（doc-writer 于 D-34a 新发现，基线既有）**：`crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/SKILL.md` 存在一个**基线即有**的未闭合 ` ```markdown ` 围栏（工作区第 192 行），既是 D-3 的可复现实例（AC-2 围栏状态机错位，JSON 模板行被误计为 en），也是**真实渲染缺陷**（其后内容被当作代码块）。已用基线与工作区双向对照确认**非本轮引入**（两侧均 32 条围栏、终态 `False`）。方向 fail-safe、不阻塞门禁；修复需改 md 正文且与 D-3 同根因，**建议下轮由 T-01 / architect 统一处理**。
- **`AGENTS.md` 反引号风格与 AC-4 冲突（待 PM 决定是否另派单）**：AC-4 要求 code span 多重集严格相等，而 `AGENTS.md` 不在 `AC4_ALLOWED_ADDED_BY_FILE`（现仅 `README.md`，`AC4_MAX_PER_ENTRY=1`），故 T-10b 新增行一律未加反引号。**本轮 T-15 引发的 `AGENTS.md:37` 修订沿用同一写法（0 新增 code span，`127.0.0.1` 等一律纯文本），`check --files AGENTS.md` 实测 PASS / exit 0**。若希望恢复反引号风格，需走补遗 v3 登记约 50 个 span 并放宽配额，属**门禁策略变更**。

### R2 复审遗留（`04-review/REVIEW-T15-T16.md`，均不阻塞提交）

| # | 内容 | 归属 / 建议 |
|---|---|---|
| **R2-1** | `scripts/check-zh-docs.py:40` 的 `OWNED_ELSEWHERE` 第 3 个成员 `.codebuddy/artifacts/2026-09-07-zh-docs-webui/` 已是**死配置**：`is_owned_elsewhere()` 的输入来自 `list_md_files()` / `cmd_hostscan()`，T-16 后两者均已排除 artifacts 域 | 复审 minor，建议下轮删除该成员或改为注释「T-16 起对 artifacts 域无效」。**未改**（PM 执行书只授权 MAJOR-1 / MAJOR-2） |
| **R2-2** | `03-impl/T-16-artifacts-exempt.md` 两处文字勘误：(a) §0 称改动 **6** hunk 而 §1 表格列 **7** 行（`gate` 提示与 `inventory` 注记同属第 6 项）；(b) §4.2 把缺 `-c core.quotePath=false` 的后果写成「**静默**」——实测 `list_md_files()` 无 `isfile` 过滤，带引号路径会进分母并在 `check_file()` 抛 `EnvError` → **exit 2（响亮失败）**，只有 `resolve_files()` 路径才是真静默漏检 | 复审 minor，需 PM 明确授权后再改（§0–§8 按裁决只追加不改写）。正确说法已记于 `T-16-artifacts-exempt.md` §9.5 第 4 条 |
| **R2-3** | 非法 host 时先打印「监听地址 http://<bad-host>:<port>」与「已绑定非回环地址…」，**之后**才 bind 失败 exit 1（`crates/rustcode-daemon/src/lib.rs:6379-6394` 打印早于 `:6498-6509` 的 `TcpListener::bind`）：输出陈述了尚未成立的事实 | 复审 nit，**判定为应修缺陷、低优先、不阻塞**；退出码 1 与 stderr `致命错误：无法绑定到 …` 均正确，无静默失败或假成功。冻结面在 `crates/rustcode-daemon`（T-15 契约 §4.4），归 **`rustcode-daemon` / architect 另派单** |

## 编排终态（2026-09-09，PM 写入）

- **G1–G6 全部达成**，看板无悬挂任务（T-12 为编号空置，非任务）。
- 终态复核（doc-writer 本轮复跑）：`git rev-parse --short HEAD` = `3ee655e3`；`git status --porcelain | wc -l` = **233**（索引内 230 + 未跟踪 3 个条目）；`python3 scripts/check-zh-docs.py gate --base 3ee655e3` = **exit 0 / PASS**（`total=248`、`受检 194 / FAIL 0`）。
- **唯一悬挂动作 = 待 PM 执行 commit**：R3 已裁决「全部提交，含 artifacts」，T-16 已提供豁免域、`00-decisions.md` Q6 已登记分母口径，门禁在 stage 前后均 exit 0。commit 前需 PM 补 stage 当前 4 个未 staged 文件与 3 个未跟踪条目中的 `04-review/REVIEW-T15-T16.md`（`.codebuddy/memory/`、`.codebuddy/teams/` 按 T-16 §7-B 已移出索引，建议**不**入库）。
- 下轮建议优先级（均不阻塞本 feature 交付）：
  1. AC-22 / AC-23 / AC-25 / AC-26 的真实浏览器与跨主机补验；
  2. ~~`rustcode daemon` 补 `--host` 参数（D-5）~~ **已由 T-15 完成**；剩余半条 = JetBrains 回退路径显式传 `127.0.0.1`（待裁决第 3 条，与 T-15 §8 O-4 的版本门控一并评估）；
  3. D-3 围栏状态机 + `SKILL.md` 未闭合围栏同根因修复；
  4. AC-27 的 mock OpenAI 兼容 provider 夹具固化为 `crates/rustcode-daemon/tests/` 下的审批回归测试（G5 建议，本轮 `files_owned` 未含测试代码）；
  5. D-1 本体（`is_loopback_authority`）与 D-4（LAN 文案 P1/P2/P3）交 architect 裁决。

## T-10b AGENTS.md 同步记录（doc-writer，2026-09-09）

派单 `files_owned`：`AGENTS.md`、`STATUS.md`、`06-release.md` 三个，全部为**纯新增/修订**，未新增或删除任何文件。

### 一、改了哪些行（改前 → 改后）

| # | 位置（改后行号） | 改前 | 改后 | 记载的事实 |
|---|---|---|---|---|
| 1 | `AGENTS.md:17`（「常用命令 → 构建」段，紧随原有 `:16` WebUI 条目） | 无此行 | 新增 1 行 | `scripts/build-webui.sh` 一键构建：等价 `cd webui && npm ci && npm run build`、可重复执行；`--if-missing` 仅 `webui/dist/index.html` 缺失时构建；成功结尾打印 `cargo clean -p rustcode-daemon`；失败/前置检查 fail-closed（缺 `node`/`npm`/`package-lock.json` 或 node 版本低于 `engines.node` → exit 2，`npm ci`/`npm run build` 失败 → exit 1，半产出 `dist` 亦判失败） |
| 2 | `AGENTS.md:28`（「常用命令 → 测试」段） | 无此行 | 新增 1 行 | `python3 scripts/check-zh-docs.py gate` 中文文档门禁用法；`check --files <path>` / `inventory` / `hostscan`（非门禁，恒返回 0） |
| 3 | `AGENTS.md:36`（新段标题行，位于「Lint / 格式」段之后、`## 架构总览` 之前） | 无此行 | 新增 1 行 | 段标题「WebUI 默认绑定地址（改这里前必读，默认值按入口而不同）」 |
| 4 | `AGENTS.md:37` | 无此行 | 新增 1 行 | CLI 两入口默认 `0.0.0.0`：`rustcode webui` 的 `--host` 默认值（`crates/rustcode-cli/src/main.rs:1047`）、`rustcode daemon` 固定传 `0.0.0.0`（`crates/rustcode-cli/src/main.rs:1780`）；`rustcode webui --host 127.0.0.1` 可退回；**`rustcode daemon` 无 `--host` 参数** |
| 5 | `AGENTS.md:38` | 无此行 | 新增 1 行 | 独立 `rustcode-daemon` 二进制（`crates/rustcode-daemon/src/main.rs:21` 的 `DEFAULT_HOST`）与 TUI `/webui`（`crates/rustcode-tuix/src/event_loop/commands.rs:2207`）**默认仍是 `127.0.0.1`**，属安全边界，不得统一改成 `0.0.0.0` |
| 6 | `AGENTS.md:39` | 无此行 | 新增 1 行 | 非回环风险提示：旧启动横幅 `Msg::DaemonWarnNonLoopback` 已从 `run_server` 移除，改由 `Msg::WebuiLanWarning`（`0.0.0.0`/`::`）与 `Msg::WebuiNonLoopbackWarning` 承担，`run_server` 非 quiet 分支（`crates/rustcode-daemon/src/lib.rs:6381`）补发；判定谓词 `is_loopback_bind_host`（`crates/rustcode-daemon/src/lib.rs:1300`，额外认 `::1` 与 `::ffff:127.0.0.1`），IPv6 回环不告警；该谓词**仅用于是否打印提示，不得用于鉴权**；变体本身按契约保留在 `crates/rustcode-config/src/i18n/messages.rs:4903`（勿当死码清理） |

- 文件规模：**664 行 → 671 行（净 +7，0 删除）**；`grep -c "" AGENTS.md` = **671**。
- 关键字命中（派单要求的自检）：`grep -n "0.0.0.0\|build-webui" AGENTS.md` 命中 **17 / 37 / 38 / 39** 共 4 行（改前为 **0 命中**）。
- **刻意未改**：`AGENTS.md` 原 `:322` 的「`127.0.0.1:13456 默认`」（位于「刻意保留/LEAVE 项（留档）」，因上方 +7 行位移至 **`:329`**）；`AGENTS.md:16` 既有 `webui/dist` + `cargo clean -p rustcode-daemon` 约定未重复添加；既有事实陈述（`rustcode` 产品身份、零遥测守卫清单、`[OBJECTIVE-*]`、`DEPENDENCY` 方向、Runtime 生命周期不变量）与既有段落结构**均未改动**。
- `git diff --stat 3ee655e3 -- AGENTS.md` 显示 `10 insertions(+), 3 deletions(-)`：其中 **7 行为本次新增**，另 3 删 3 增是**他人 D-35 批次**把 `README.zh-CN.md` 改写为「原中文 README」（现 `AGENTS.md:81` / `:250` / `:286`），非本轮引入。

### 二、写法约束（需 PM 知情，非阻塞）

新增行中的命令、路径、符号**一律未加反引号**（行内 code span）。原因：AC-4 要求 code span 多重集严格相等，`AGENTS.md` **不在** `AC4_ALLOWED_ADDED_BY_FILE`（当前仅 `README.md` 一项，且 `AC4_MAX_PER_ENTRY = 1`）内，任何新增 code span 都判「越界新增」→ `check` FAIL。带反引号的初版实测 `FAIL ... 越界新增 51 条`；改为纯文本后 `added` 为空、`en` 仍为 0（AC-2 判据为「含 4+ 字母 ASCII 词且**不含 CJK**」，新增行均含中文）→ PASS。本轮禁止修改 `scripts/check-zh-docs.py`，故未登记 D2 放行。**若 PM 希望 `AGENTS.md` 恢复反引号风格，需另派单走补遗 v3 登记 `AC4_ALLOWED_ADDED_BY_FILE['AGENTS.md']`（约 50 个 span，与配额上限 1 冲突，需一并放宽），属门禁策略变更，不在本轮授权内。**

### 三、真实执行的验证（原样复现）

```console
$ git rev-parse --short HEAD
3ee655e3
(EXIT=0)

$ python3 scripts/check-zh-docs.py check --base 3ee655e3 --files AGENTS.md
PASS AGENTS.md en=0/546=0.0000
  [AC-4-authorized] AGENTS.md | removed=README.zh-CN.md | added=- | cause=D1
  [AC-4-authorized] AGENTS.md | removed=README.zh-CN.md | added=- | cause=D1
  [AC-4-authorized] AGENTS.md | removed=README.zh-CN.md:94 | added=- | cause=D1

check: 受检 1，PASS 1，FAIL 0
(EXIT=0)

$ python3 scripts/check-zh-docs.py gate --base 3ee655e3
gate: 未跟踪 md 已排除：61 个（不进 AC-1 分母，也不受 AC-2/4/6/7b/32 与 AC-8 段 1 约束）
gate: base=3ee655e3
PASS AC-1 清单自洽            SKIP_A=4 SKIP_B=47 SKIP_C=1 SKIP_D=2 TODO=29 ZH=202 total=285
PASS AC-2/3/4/6/7b/32 全量 check   全量 check 受检 231，FAIL 0；AC-4 授权放行合计 12 条（D1 11 / D2 1）
PASS AC-5 运行时载荷
PASS AC-7a 无改名/删除
PASS AC-8 外链残留（段 1 正式域 0 命中 / 段 2 历史域 39 处全在 .codebuddy/artifacts/ / 段 3 兜底域 0 命中）
gate: PASS
(GATE_EXIT=0)

$ git diff --stat 3ee655e3 -- AGENTS.md
 AGENTS.md | 13 ++++++++++---
 1 file changed, 10 insertions(+), 3 deletions(-)

$ git status --porcelain | wc -l
169

$ grep -n "0\.0\.0\.0\|build-webui" AGENTS.md
17 / 37 / 38 / 39（4 行命中）

$ grep -c "" AGENTS.md
671
```

结论：`HEAD` 仍为基线 `3ee655e3`；`gate` **exit 0**（未打红）；工作区未提交改动数**仍为 169**（只改了 `files_owned` 的 3 个文件，无新增/删除文件）。

> **快照口径注记（2026-09-09 契约登记轮追加）**：本节全部数字是 **T-10b 当时**的快照（未跟踪 61 / `total=285` / `受检 231` / `git status` 169 / `AGENTS.md` 671 行）。此后 T-15 / T-16 改变了分母口径与工作区状态，**当前值**见本文件顶部「实测现状」表与「R2 复审遗留」段：`total=248`、`受检 194 / FAIL 0`、未跟踪 1、`git status --porcelain | wc -l` = **233**、`AGENTS.md` 行数仍为 **671**（本轮只重写 `:37` 一行，1 insertion / 1 deletion）。本节按裁决原样保留，不改写。
