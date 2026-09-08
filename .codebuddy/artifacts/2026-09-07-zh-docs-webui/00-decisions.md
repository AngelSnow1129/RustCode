---
kind: decisions
from: orchestrator
feature: 2026-09-07-zh-docs-webui
created: 2026-09-07
basis: 用户 4 项裁决 + 00-requirement.md §8 开放问题
---

# 编排者对 Q1–Q5 的裁决（冻结，01-design.md 必须遵守）

## Q1 — `crates/rustcode-review/rules/*.md`（47 个）是否汉化

**裁决：A — 不汉化。**

理由：
1. 用户裁决 #1 的口径是「168 个纯英文 md」。编排者原始盘点中这 47 个已被归入「52 个建议跳过」，
   **本就不在 168 之内**，故 A 与用户裁决无冲突，不需要回退到用户。
2. 它们是 `include_str!` 编译期嵌入的模型输入（`crates/rustcode-review/src/rules.rs:110-156`），
   汉化会静默改变每次 code review 的模型输出，且无任何测试可回归。
3. 已有官方逃生口 `--rules-dir <dir>`，需中文规则的用户无需改仓库。

## Q2 — 默认 0.0.0.0 覆盖哪几处

**裁决：A — 只改两处，独立 `rustcode-daemon` 二进制保持 `127.0.0.1`。**

改：
- `crates/rustcode-cli/src/main.rs:1047`（`rustcode webui --host` 默认值）
- `crates/rustcode-cli/src/main.rs:1779-1780`（`rustcode daemon` 子命令）

不改：
- `crates/rustcode-daemon/src/main.rs:21`（`DEFAULT_HOST` 保持 `127.0.0.1`）

理由：
1. **与用户原话一致**：用户说的是「webui 默认使用 0.0.0.0」，独立 daemon 二进制从未被要求改。
2. 上述两处 `enforce_token = true`，token 鉴权已生效，审批不翻转，零安全回归。
3. 独立 daemon 二进制由 VS Code（`extensions/vscode/src/daemon/process.ts:369`）与
   JetBrains（`RustCodeDaemonProcess.kt:116`）**不带 `--host`** 拉起，且 `webui_tokens=None`
   ⇒ `enforce_token=false`。若改它会同时触发「无鉴权暴露 /chat + 工具执行」与
   「Build 模式 `dangerously_skip_permissions=true` 静默放行」双杀（00-requirement.md §0.1）。
4. IDE 用户需要跨设备访问时仍可显式传 `--host`，能力不丢失。

⇒ AC-27 降级为「回归验证现状」（该入口行为不得变化），AC-28 **不需要**改动 Rust 判定逻辑。

## Q3 — 「不再警告」的范围

**裁决：A — 删除 `DaemonWarnNonLoopback` 启动横幅，保留 `Msg::WebuiLanWarning`。**

理由：
1. 用户选的是「改默认且不再警告」，其选项描述为「静默绑定，**只在日志里给访问地址**」。
2. `DaemonWarnNonLoopback`（`lib.rs:6345-6347`）在默认 `0.0.0.0` 下会每次启动必打印，
   退化成噪音 —— 这正是用户要去掉的。
3. `WebuiLanWarning`（`lib.rs:5398-5407`）承载「公网请用隧道 / 无 TLS」的可操作信息，
   且它本就随访问地址一起输出，符合用户「只在日志里给访问地址」的描述，保留不算「警告横幅」。

**连带必修**：`lib.rs:6333-6341` 的注释声称 "Default to loopback-only for security ... PR #82"，
与新默认直接矛盾，必须改写。

## Q4 — 是否保留英文入口

**裁决：B — 不保留，中文单语。**

理由：本仓库已彻底去平台化，README 内所有仓库地址均为 `example.com` 占位
（`README.md:213,724,790`），上游 `SecLab/RustCode` 链接已从发行链路删除（`AGENTS.md:224`）。
**不存在可验证访问的英文 README 地址**，选项 A 会引入死链，违反 `AGENTS.md:270` 对死链的既有处理原则。
英文原文可通过 git 历史获取。

## Q5 — setup-seeds 6 个 md 的 `description` 字段

**裁决：A — 汉化。**

理由：fork 默认中文（`AGENTS.md:103`）；`.codebuddy/agents/requirements-analyst.md:3` 的
`description` 已是中文，存在先例；skills 是用户可改写的本地文件，风险低（00-requirement.md §3.3）。

---

## 附带立项（不属本需求范围，仅记录）

- **N4**：`lib.rs:4671-4674` 在「无交互审批方」时对 `ApprovalMode::Build` 置
  `dangerously_skip_permissions = true`，属 fail-open，与 `AGENTS.md:154` 的 fail-closed 原则相悖。
- **N1**：`ServerOpts.startup_mode`（`lib.rs:6042`）是死字段，`--client` 启动参数对请求处理无效果。

两者均**不在**本次改动范围内，已在 `06-release.md` 交付清单中记录为已知未验证/待办项。

---

## Q6 — AC-1 分母与 `.codebuddy/artifacts/` 门禁豁免域（2026-09-09 追加，契约层追认）

> 裁决来源：`04-review/REVIEW-T15-T16.md` §2「可推进提交的判据」第 1 条 与 §4 复审点 11(d)（PM 采纳**方案 A**）；
> 实现依据：`03-impl/T-16-artifacts-exempt.md`（§9.3 PM 最终裁决、§9.5 第 1 项交由契约层登记）。
> 本节为**追加**，Q1–Q5 未作任何改动。

**裁决**：AC-1 的分母 = 全仓已跟踪 md **减去显式豁免域** `.codebuddy/artifacts/`。域内 md 的约束状态：

| 约束 | 域内是否适用 | 依据 |
|---|---|---|
| AC-1 分母 | **不适用**（不进分母） | `list_md_files()` 按路径前缀过滤（`scripts/check-zh-docs.py:196-199`） |
| AC-2 / AC-4 / AC-6 / AC-7b / AC-32 | **不适用** | `resolve_files()` 批量分支同一过滤（`:985-991`）；`args.files` 早返回（`:974-975`）未动，显式 `--files` 仍可单点检。AC-3 的残留学检样本取自 AC-2 结果，故同样不覆盖域内 |
| AC-8 段 1（正式域） | **不适用** | 段 1 候选生成时即排除该前缀 |
| AC-8 段 2（历史域） | **适用** | 段 2 的反向断言「`README.zh-CN` 命中必须仍有、且必须全部位于域内」**以该域为样本**；去掉它段 2 会失效 |
| AC-7a（改名/删除） | **适用** | `gate_check_ac7a()` 未加任何豁免 |

**数字变化（实测，命令可原样复现）**：

- AC-1 分母 **285 → 248**；全量 check 受检 **231 → 194**（`python3 scripts/check-zh-docs.py gate --base 3ee655e3`）。
- 差值 **37** 的构成：**基线 `3ee655e3` 已入库的历史 artifacts 交接件 md** —— `2026-09-02-cleanup-codingplan-legacy` 19、`2026-09-02-g1-fmt-gate` 7、`2026-09-03-git-wrapup` 5、`2026-09-03-banner-release` 3、`2026-09-02-strip-atomcode` 1、`2026-09-03-doc-consistency` 1、`2026-09-03-residual-two-items` 1（共 7 个历史 feature 目录，合计 37）。**不是**本 feature 产物。
- 自洽式：`git ls-files -- '*.md'` = **349**，域内 **101**（= 本 feature 64 + 历史 37），349 − 101 = **248**，与脚本输出分母逐字相符。
- **豁免未掩盖失败的实证**：`03-impl/T-16-artifacts-exempt.md` §9.2 的解耦变体实测 `受检 231 / FAIL 0`，即这 37 个历史件在当前判据下**无一失败**；出域不隐藏任何既有失败。

**为何选方案 A（整前缀豁免）而非法案 B（收窄到本 feature 目录）**：

1. **与 `SKIP_A`…`SKIP_D` 同源**：按路径前缀判定、与基线无关、与是否已跟踪无关，对未来**所有** feature 一致，不必每 feature 改常量。
2. **方案 B 会重踩同一个坑**：下一个 feature 的交接件仍会以「新文件无基线（`old_text = ""`、`is_new=True`）→ AC-4 把全文 code span 记为 added」触发大面积假 FAIL；`03-impl/T-16-artifacts-exempt.md` §6 实测撤销豁免后为 `受检 295 / FAIL 64 / exit 1`。
3. **域内并未彻底脱离门禁**：AC-7a 与 AC-8 段 2 仍覆盖（见上表），且域内当前全部 PASS。
4. 代价（分母 285 → 248、受检 231 → 194）已显式记录，并由本节在契约层追认。

**口径取代关系（遇到旧表述一律以本节为准）**：

- `01-design.md:60` 的「**AC-1 一律以脚本产出计数为准，不硬编码**」**继续有效**。本裁决改变的是分母的**集合定义**（排除显式豁免域），不是数字来源，二者不冲突。
- `00-requirement.md:259` 的 AC-1 恒等式文字含「全仓 md 总数」字样；自本裁决起一律按「**受检 md 总数（已排除 artifacts 豁免域）**」理解，即 `SKIP_A + SKIP_B + SKIP_C + SKIP_D + TODO + ZH = 受检 md 总数`（当前 4 + 47 + 1 + 2 + 27 + 167 = 248）。**本文件不修改 `00-requirement.md`**。
- `scripts/check-zh-docs.py` 的 `render_inventory()` 输出与 `:27` 注释已随 T-16 §9.4 同步为「受检 md 总数（已排除 artifacts 豁免域）」，脚本内「全仓 md 总数」零残留（`grep -rn "全仓 md 总数" scripts/check-zh-docs.py` 无输出）。
