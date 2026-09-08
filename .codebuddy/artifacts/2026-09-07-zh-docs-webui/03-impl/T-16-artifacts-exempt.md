---
kind: implementation
id: T-16
from: code-implementer
to: [project-manager]
feature: 2026-09-07-zh-docs-webui
status: done
decision: proceed
requires: [T-06, T-08]
files_owned:
  - scripts/check-zh-docs.py
  - .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/T-16-artifacts-exempt.md
architecture_constraints:
  touches_runtime_lifecycle: false
  touches_persistence: false
  touches_cross_crate_deps: false
created: 2026-09-09
---

# T-16 · 把 `.codebuddy/artifacts/` 提升为门禁豁免域

> 前置裁决：用户裁决「本 feature 改动全部提交，含 `.codebuddy/artifacts/2026-09-07-zh-docs-webui/**` 交接件」；
> 本任务是其**前置条件** —— 不改判据、不放宽阈值，只把 artifacts 提升为显式豁免域，使提交（stage）前后门禁语义完全一致。

## 0. 元信息

| 项 | 值 |
|---|---|
| 分支 / HEAD | `dev` / `3ee655e3`（验证开始时 HEAD 仍等于基线） |
| 基线 `--base` | `3ee655e3` |
| 改动文件数 | 2（`scripts/check-zh-docs.py` 1 个；本报告 1 个新增） |
| `scripts/check-zh-docs.py` 改动 hunk 数 | 6（新增常量块 / AC-8 复用 / `_git` / `list_md_files` / `resolve_files` / `gate` 提示 + inventory 分母注记） |
| 未改动 | 任何 md 汉化正文、任何 Rust 源码、任何测试、`.gitignore`、`AGENTS.md` |
| 提交动作 | 已执行 `git add -A`（授权范围内）；**未 commit**；随后把 `.codebuddy/memory/` 与 `.codebuddy/teams/` 两个 agent 运行时目录移出索引（详见 §7 需 PM 裁决项） |

**为何需要本改动**：AC-1 的分母是「已跟踪 md」。本 feature 的 64 个交接件 md 在 `git add` 前属未跟踪文件、被 gate 显式排除；一旦 `git add`，它们以「基线不存在（`old_text = ""`、`is_new=True`）」身份进入分母，AC-4 会把全文 code span 全部记为 added → 大面积假 FAIL（§6 实测：撤销豁免后 `gate` 受检 295 / **FAIL 64** / 退出码 1）。

## 1. 改动 hunk 列表

行号为改动后 `scripts/check-zh-docs.py` 的行号（该文件在本 feature 中本身是新增文件，故 `git diff` 显示为整文件新增；下表只列本次 T-16 的语义改动点）。

| # | 行号 | 改前 | 改后 | 常量/函数 |
|---|---|---|---|---|
| 1 | `42-65` | （无） | 新增注释块 + `ARTIFACTS_EXEMPT_PREFIX = ".codebuddy/artifacts/"` + `def is_artifacts_exempt(path) -> bool` | `ARTIFACTS_EXEMPT_PREFIX`、`is_artifacts_exempt()` |
| 2 | `110` | `AC8_EXEMPT_PREFIX = ".codebuddy/artifacts/"` | `AC8_EXEMPT_PREFIX = ARTIFACTS_EXEMPT_PREFIX`（**值不变**，消除重复字面量） | `AC8_EXEMPT_PREFIX` |
| 3 | `152-160` | `subprocess.run(["git"] + args, ...)` | `subprocess.run(["git", "-c", "core.quotePath=false"] + args, ...)`（注释说明原因） | `_git()` |
| 4 | `191-199` | `return sorted(git_lines(["ls-files", "--", "*.md"], root))` | `return sorted(path for path in git_lines([...]) if not is_artifacts_exempt(path))` | `list_md_files()`（AC-1 分母唯一来源 → `inventory` / `gate` / `hostscan` 一致） |
| 5 | `971-991` | `return sorted(path for path in files if os.path.isfile(...))` | 追加 `and not is_artifacts_exempt(path)`；`if args.files: return list(args.files)` **保持早返回不变** | `resolve_files()`（批量候选；显式 `--files` 不受影响） |
| 6 | `1336-1353` | 仅打印「未跟踪 md 已排除：N 个」 | 追加「artifacts 豁免域 `...` 已排除：M 个（与 SKIP_A..SKIP_D 同级…）」；`untracked` 列表同步剔除豁免域成员 | `cmd_gate()` |
| 7 | `863-866` | `- 分母: git ls-files -- '*.md'（仓库当前跟踪的 md）` | 追加「已排除 artifacts 豁免域 `.codebuddy/artifacts/`（T-16）」 | `render_inventory()` |

语义要点：

- **单一来源**：豁免只在 `list_md_files()`（AC-1 分母）与 `resolve_files()` 的**批量分支**各过滤一次；`inventory` / `gate` / `hostscan` 共用 `list_md_files()`，`check --diff` 走 `resolve_files()`，四个入口语义一致。
- **与基线无关**：判定是路径前缀，不看 `base`、不看是否已跟踪 —— 与 `SKIP_A..SKIP_D` 同级，故换任何基线都不会让交接件「回流」进分母。
- **显式 `--files` 仍可检**：`resolve_files()` 的 `if args.files: return list(args.files)` 早返回未被改动（行 `972-976` 注释也写明了这一点）。
- **不越界**：`is_artifacts_exempt()` 只用 `startswith(".codebuddy/artifacts/")`，`.codebuddy/agents/`、`.codebuddy/memory/`、`.codebuddy/teams/`、`.codebuddy/rules/` 前缀不同，一律不豁免（§5 实测）。
- **退出码语义未变**：0 = 全通过 / 1 = 有 FAIL / 2 = 用法或环境问题，未新增任何返回码分支。

## 2. 契约符合性（对照 `01-design.md` / `01-design-addendum.md`）

| 契约 | 结论 |
|---|---|
| AC-1 恒等式 `SKIP_A+SKIP_B+SKIP_C+SKIP_D+TODO+ZH == 全仓 md 总数` | 一致。分母改为「全仓 md 减 artifacts 豁免域」，恒等式在该域内仍成立（`恒等式=OK`，§4）。`01-design.md:60` 明确「**AC-1 一律以脚本产出计数为准，不硬编码**」，故分母随豁免域收缩是允许的；SKIP_A/B/C/D 的期望值 4/47/1/2 是冻结期望，实测仍为 4/47/1/2 |
| AC-2 阈值 0.05、AC-6、AC-7a、AC-7b、AC-32 判据与常量 | 未触碰，一律不变 |
| AC-4 v2（补遗 §2） | 未触碰判据；`AC4_ALLOWED_ADDED_BY_FILE` 等冻结常量一字未改。效果是把 64 个「无基线新增件」移出受检集，避免把「新文件 = 全量 added」误判为越界新增 |
| AC-8 v2（补遗 §3） | 未触碰三段结构与 A1~A4 护栏；`AC8_EXEMPT_PREFIX` 改为引用 `ARTIFACTS_EXEMPT_PREFIX`，**取值逐字不变**，段 1 正式域 / 段 2 历史域 / 段 3 兜底域行为不变（§4.3 三段输出对照） |
| 「排除集只有 `AC8_EXEMPT_PREFIX` 一个成员」 | 仍为 1 个成员；新增的是 AC-1 分母级豁免域，不是 AC-8 的第二个排除成员 |
| 禁止新增 `--allow` / `--skip` 开关 | 未新增任何命令行开关 |

**声明的偏差（1 项，属裁决的必然推论）**：豁免域是 `.codebuddy/artifacts/` 整个前缀，因此**基线已入库的 37 个历史交接件 md**（2026-09-02 等历史 feature）一并出域，AC-1 分母由 285 变为 248、全量受检由 231 变为 194。这不是放宽判据，而是「与 SKIP_A..SKIP_D 同源、按路径判定、与基线无关」的必然结果；若只豁免本 feature 目录，则实测会破坏冻结的 AC-8（§7 裁决项 A，有实测反证）。

## 3. 自验证证据（真实命令 + 输出 + 退出码）

### 3.1 提交前

```console
$ git rev-parse --short HEAD
3ee655e3
$ git status --porcelain | wc -l
169
$ python3 -m py_compile scripts/check-zh-docs.py
（无输出，退出码 0）

$ python3 scripts/check-zh-docs.py gate --base 3ee655e3
gate: 未跟踪 md 已排除：1 个（不进 AC-1 分母，也不受 AC-2/4/6/7b/32 与 AC-8 段 1 约束）
  - .codebuddy/memory/2026-09-09.md

gate: artifacts 豁免域 `.codebuddy/artifacts/` 已排除：37 个（与 SKIP_A..SKIP_D 同级，按路径前缀判定、与基线无关；不进 AC-1 分母，也不受 AC-2/4/6/7b/32 与 AC-8 段 1 约束；域内文件仍是 AC-8 段 2 的历史域）

gate: base=3ee655e3

PASS AC-1 清单自洽
  SKIP_A=4 SKIP_B=47 SKIP_C=1 SKIP_D=2 TODO=27 ZH=167 total=248
PASS AC-2/3/4/6/7b/32 全量 check
  全量 check 受检 194，FAIL 0
  AC-4 授权放行合计 12 条（D1 11 / D2 1）
    .codebuddy/agents/doc-writer.md | removed=README.zh-CN.md | added=- | cause=D1
    ...（其余 11 条略）
PASS AC-5 运行时载荷
PASS AC-7a 无改名/删除
PASS AC-8 外链残留
  段 1 正式域（候选 515，已排除 .codebuddy/artifacts/）：0 命中 -> OK
  段 2 历史域（候选 37）：命中 39 处，全部位于 .codebuddy/artifacts/ -> OK（历史痕迹仍在）
  段 3 兜底域 site/.github/docs/extensions（全扩展名）：0 命中 -> OK

gate: PASS
GATE_EXIT=0

$ python3 scripts/check-zh-docs.py inventory --base 3ee655e3 | tail -n 12
| `docs/vscode-i18n-implementation-plan-2026-06-30.md` | 71 | 58 | 1.224 |  |
| `docs/webhook-implementation-summary.md` | 130 | 183 | 0.710 |  |
| `evals/deepseek-v4-flash/README.md` | 15 | 23 | 0.652 |  |
| `evals/deepseek-v4-flash/cases/agent-fixture/ARCHITECTURE.md` | 3 | 2 | 1.500 |  |
| `extensions/jetbrains/PRIVACY.md` | 24 | 15 | 1.600 |  |
| `extensions/jetbrains/README.md` | 115 | 139 | 0.827 |  |
| `extensions/jetbrains/docs/jetbrains.md` | 77 | 47 | 1.638 |  |
| `extensions/vscode/README.md` | 79 | 42 | 1.881 |  |
| `packages/npm/README.md` | 22 | 23 | 0.957 |  |
| `webui/README.md` | 20 | 29 | 0.690 |  |

inventory: total=248 SKIP_A=4 SKIP_B=47 SKIP_C=1 SKIP_D=2 TODO(EN=0,MIXED=27) ZH=167 恒等式=OK
（退出码 0）

$ python3 scripts/check-zh-docs.py check --base 3ee655e3 | tail -n 5
check: 必须指定 --files P... 或 --diff
EXIT=2        # 注：这是脚本既有约定（main() 行 1507-1508），与本次改动无关；判据未动

$ python3 scripts/check-zh-docs.py check --diff --base 3ee655e3 | tail -n 3
check: 受检 141，PASS 140，FAIL 1
EXIT=1        # 唯一 FAIL 是未跟踪的 .codebuddy/memory/2026-09-09.md（非本 feature，见 §7）

$ python3 scripts/check-zh-docs.py check --base 3ee655e3 --files .codebuddy/artifacts/2026-09-07-zh-docs-webui/STATUS.md | tail -n 2
check: 受检 1，PASS 0，FAIL 1
EXIT=1        # 显式 --files 仍可检（未被豁免静默跳过），符合实现要求 3

$ python3 scripts/check-zh-docs.py gate --base deadbeefdeadbeef
错误: --base 不是可解析的 commit: 'deadbeefdeadbeef'（git rev-parse --verify deadbeefdeadbeef^{commit} 失败: 无输出）
EXIT=2        # NIT-2 未回退
```

### 3.2 `git add -A` 之后

```console
$ git add -A
（退出码 0）
$ git status --porcelain | wc -l
236
$ git diff --cached --name-only | wc -l
236
$ git ls-files | grep -c '^\.codebuddy/artifacts/'
100           # ASCII 路径计数；含非 ASCII 文件名的 HANDOFF-汉化铁律.md 被 git 八进制转义，
              # 故该 grep 少 1。用 -c core.quotePath=false 计为 101（见下）

# —— 第一次 gate：把 agent 运行时 memory 文件一并 stage 的情形（如实记录）——
$ python3 scripts/check-zh-docs.py gate --base 3ee655e3
gate: artifacts 豁免域 `.codebuddy/artifacts/` 已排除：101 个（...）
PASS AC-1 清单自洽
  SKIP_A=4 SKIP_B=47 SKIP_C=1 SKIP_D=2 TODO=27 ZH=168 total=249
FAIL AC-2/3/4/6/7b/32 全量 check
  全量 check 受检 195，FAIL 1
    FAIL .codebuddy/memory/2026-09-09.md [AC-4] en=1/26=0.0385
PASS AC-5 运行时载荷
PASS AC-7a 无改名/删除
PASS AC-8 外链残留
gate: FAIL
GATE_EXIT=1   # 唯一 FAIL 源是 .codebuddy/memory/2026-09-09.md（agent 运行时 memory，非 artifacts、非本 feature 交付物）

# —— 把 agent 运行时状态移出索引（文件保留在工作区），再跑 ——
$ git restore --staged -- .codebuddy/memory .codebuddy/teams
（退出码 0）
$ git status --porcelain | wc -l
232
$ git diff --cached --name-only | wc -l
230
$ git ls-files | grep -c '^\.codebuddy/artifacts/'
100
$ git -c core.quotePath=false ls-files | grep -c '^\.codebuddy/artifacts/'
101           # 证明 64 个本 feature 交接件（含非 ASCII 文件名）已全部进入索引

$ python3 scripts/check-zh-docs.py gate --base 3ee655e3
gate: 未跟踪 md 已排除：1 个（不进 AC-1 分母，也不受 AC-2/4/6/7b/32 与 AC-8 段 1 约束）
  - .codebuddy/memory/2026-09-09.md

gate: artifacts 豁免域 `.codebuddy/artifacts/` 已排除：101 个（与 SKIP_A..SKIP_D 同级，按路径前缀判定、与基线无关；不进 AC-1 分母，也不受 AC-2/4/6/7b/32 与 AC-8 段 1 约束；域内文件仍是 AC-8 段 2 的历史域）

gate: base=3ee655e3

PASS AC-1 清单自洽
  SKIP_A=4 SKIP_B=47 SKIP_C=1 SKIP_D=2 TODO=27 ZH=167 total=248
PASS AC-2/3/4/6/7b/32 全量 check
  全量 check 受检 194，FAIL 0
PASS AC-5 运行时载荷
PASS AC-7a 无改名/删除
PASS AC-8 外链残留
  段 1 正式域（候选 515，已排除 .codebuddy/artifacts/）：0 命中 -> OK
  段 2 历史域（候选 101）：命中 261 处，全部位于 .codebuddy/artifacts/ -> OK（历史痕迹仍在）
  段 3 兜底域 site/.github/docs/extensions（全扩展名）：0 命中 -> OK

gate: PASS
GATE_EXIT=0

$ python3 scripts/check-zh-docs.py inventory --base 3ee655e3 | tail -n 1
inventory: total=248 SKIP_A=4 SKIP_B=47 SKIP_C=1 SKIP_D=2 TODO(EN=0,MIXED=27) ZH=167 恒等式=OK
（退出码 0）

$ python3 scripts/check-zh-docs.py check --diff --base 3ee655e3 | tail -n 1
check: 受检 141，PASS 140，FAIL 1
EXIT=1        # 与提交前逐字相同；唯一 FAIL 仍是 .codebuddy/memory/2026-09-09.md

$ python3 scripts/check-zh-docs.py check --base 3ee655e3 --files .codebuddy/artifacts/2026-09-07-zh-docs-webui/STATUS.md | tail -n 1
check: 受检 1，PASS 0，FAIL 1
EXIT=1        # 显式 --files 仍可检

$ python3 scripts/check-zh-docs.py gate --base deadbeefdeadbeef
错误: --base 不是可解析的 commit: 'deadbeefdeadbeef'（git rev-parse --verify deadbeefdeadbeef^{commit} 失败: 无输出）
EXIT=2        # NIT-2 未回退
```

### 3.3 提交前 / 提交后一致性对照表

| 指标 | 提交前 | `git add -A` 后 | 一致 |
|---|---|---|---|
| `gate` 退出码 | 0 | 0 | 是 |
| `gate` AC-1 `SKIP_A` | 4 | 4 | 是 |
| `gate` AC-1 `SKIP_B` | 47 | 47 | 是 |
| `gate` AC-1 `SKIP_C` | 1 | 1 | 是 |
| `gate` AC-1 `SKIP_D` | 2 | 2 | 是 |
| `gate` AC-1 `TODO` | 27 | 27 | 是 |
| `gate` AC-1 `ZH` | 167 | 167 | 是 |
| `gate` AC-1 `total` | 248 | 248 | 是 |
| `gate` 全量 check 受检 | 194 | 194 | 是 |
| `gate` 全量 check FAIL | 0 | 0 | 是 |
| `gate` AC-4 授权放行 | 12（D1 11 / D2 1） | 12（D1 11 / D2 1） | 是 |
| `inventory` 末行 | `total=248 SKIP_A=4 SKIP_B=47 SKIP_C=1 SKIP_D=2 TODO(EN=0,MIXED=27) ZH=167 恒等式=OK` | 逐字相同 | 是 |
| `inventory` 退出码 | 0 | 0 | 是 |
| `check --diff` 受检 / PASS / FAIL | 141 / 140 / 1 | 141 / 140 / 1 | 是 |
| `check --files <artifacts 文件>` | 受检 1，FAIL 1 | 受检 1，FAIL 1 | 是 |
| `gate --base deadbeefdeadbeef` 退出码 | 2 | 2 | 是 |
| AC-8 段 1 正式域 | 候选 515，0 命中 -> OK | 候选 515，0 命中 -> OK | 是 |
| AC-8 段 3 兜底域 | 0 命中 -> OK | 0 命中 -> OK | 是 |
| AC-8 段 2 历史域（历史域大小随入库增长，判定与退出码不变） | 候选 37，命中 39 -> OK | 候选 101，命中 261 -> OK | 判定一致（PASS/PASS） |

> 注：`.codebuddy/artifacts/` 是「历史不可篡改」的历史域，段 2 的**候选数与命中数**在交接件入库后从 37/39 增长到 101/261 属于预期 —— 段 2 的判据是「必须仍有命中且命中全在豁免前缀内」，两段都 PASS、退出码不变。

## 4. 关键设计取舍的实测反证

### 4.1 为什么必须豁免整个 `.codebuddy/artifacts/` 前缀（而不是只豁免本 feature 目录）

把同一份脚本的常量改成 `".codebuddy/artifacts/2026-09-07-zh-docs-webui/"`（其余不动）后实测：

```console
$ python3 /tmp/variantB.py gate --base 3ee655e3
PASS AC-1 清单自洽
  SKIP_A=4 SKIP_B=47 SKIP_C=1 SKIP_D=2 TODO=29 ZH=202 total=285
  全量 check 受检 231，FAIL 0
FAIL AC-8 外链残留
  段 1 正式域（候选 552，已排除 .codebuddy/artifacts/2026-09-07-zh-docs-webui/）：命中 39 处 -> FAIL，示例: ['.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/00-requirement.md:87:...']
  段 2 历史域（候选 0，前缀 .codebuddy/artifacts/2026-09-07-zh-docs-webui/）：0 命中 -> FAIL；历史交接件中的 `README.zh-CN` 记录消失：违反历史不可篡改
gate: FAIL
EXIT=1
```

结论：该变体确实能保住 `total=285 / 受检 231`（PM 期望数字），但会**双重破坏冻结的 AC-8**：37 个历史交接件被推入正式域（39 处命中），而本 feature 交接件在 `git add` 前尚未入库、历史域候选为 0。故不可行 —— 除非修改 AC-8 补遗契约（回退 `solution-architect`，本任务无权）。

### 4.2 为什么 `_git()` 要带 `-c core.quotePath=false`

`git ls-files` 默认把含非 ASCII 的路径写成八进制转义并整体加引号：

```console
$ git ls-files --others --exclude-standard -- '*.md' | grep '^"'
".codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/HANDOFF-\346\261\211\345\214\226\351\223\201\345\276\213.md"
```

该字符串以 `"` 开头，`startswith(".codebuddy/artifacts/")` 判否、`os.path.isfile()` 也判否 —— 若不加该选项，这个交接件会在 `git add` 后漏出豁免域并重演 AC-4 假 FAIL（且是静默的）。加了该选项后同一文件在 gate 中被计入豁免域（101 = 100 + 1，见 §3.2）。

## 5. `.codebuddy/agents/*.md` 未被误豁免的证据

```console
$ python3 scripts/check-zh-docs.py inventory --base 3ee655e3 | grep -c '^| `\.codebuddy/agents/'
7                     # 7 个 agents md 全部仍在 AC-1 清单内
$ python3 scripts/check-zh-docs.py inventory --base 3ee655e3 | grep -c '^| `\.codebuddy/artifacts/'
0                     # 清单表格中不再有任何 artifacts 条目（唯一命中是行 5 的分母说明文字）
$ grep -n 'codebuddy/agents' <(python3 scripts/check-zh-docs.py gate --base 3ee655e3)
13:    .codebuddy/agents/doc-writer.md | removed=README.zh-CN.md | added=- | cause=D1
                      # 仍参与 AC-4 判分并产生 D1 授权放行
$ python3 scripts/check-zh-docs.py check --base 3ee655e3 --files .codebuddy/agents/doc-writer.md | tail -n 1
check: 受检 1，PASS 1，FAIL 0
```

前缀判定自检（`is_artifacts_exempt()` 逐例实测）：

```console
.codebuddy/artifacts/2026-09-07-zh-docs-webui/STATUS.md                exempt=True
.codebuddy/artifacts/2026-09-02-cleanup-codingplan-legacy/01-design.md  exempt=True
.codebuddy/agents/doc-writer.md                                         exempt=False
.codebuddy/memory/2026-09-09.md                                         exempt=False
.codebuddy/teams/x/config.json                                          exempt=False
.codebuddy/rules/rustcode.md                                            exempt=False
AGENTS.md                                                               exempt=False
ARTIFACTS_EXEMPT_PREFIX = '.codebuddy/artifacts/'
AC8_EXEMPT_PREFIX is ARTIFACTS_EXEMPT_PREFIX = True '.codebuddy/artifacts/'
```

## 6. 回滚方案

**撤销豁免（最小动作，2 处）**：把 `list_md_files()` 行 `198` 的 `if not is_artifacts_exempt(path)` 与 `resolve_files()` 行 `988-991` 的 `and not is_artifacts_exempt(path)` 去掉即可（常量、`is_artifacts_exempt()`、gate 提示、inventory 注记可保留，仅剩提示语义）；若要完全回到改动前，再删除 hunk 1/2/3/6/7。

**撤销后提交的实测结果**（用只去掉上述两处过滤的脚本 `/tmp/noexempt.py` 在已 stage 状态下实测）：

```console
$ python3 /tmp/noexempt.py gate --base 3ee655e3
  SKIP_A=4 SKIP_B=47 SKIP_C=1 SKIP_D=2 TODO=29 ZH=266 total=349
  全量 check 受检 295，FAIL 64
FAIL AC-2/3/4/6/7b/32 全量 check
gate: FAIL
NOEXEMPT_GATE_EXIT=1
$ grep -c 'FAIL \.codebuddy/artifacts/2026-09-07' <输出>
64                    # 64 条 FAIL 全部是本 feature 交接件（含非 ASCII 文件名的 HANDOFF-汉化铁律.md）
```

即：撤销豁免后执行 `git commit`，门禁会从 PASS 变为 **FAIL（退出码 1）**，受检 295、FAIL 64，全部源于「新增文件无基线 → AC-4 把全文 code span 记为 added」的假 FAIL；AC-8 三段不受影响（段 1 仍 0 命中），因为 AC-8 的排除前缀是独立常量。

## 7. 遗留风险与需 PM 裁决项

**A（需裁决，但不影响本次交付）**：AC-1 分母 285 → 248、全量受检 231 → 194，差值 37 全部是**基线已入库的历史交接件**（`.codebuddy/artifacts/2026-09-02-*` 等）。若 PM 要求分母必须仍是 285，唯一实现路径是收窄豁免前缀到本 feature 目录，而实测会破坏冻结的 AC-8（§4.1）→ 那必须先改 AC-8 契约并回退 `solution-architect`。本人判断：保持整前缀豁免（与 SKIP_A..SKIP_D 同级、与基线无关）才是裁决的文字与意图，且 `01-design.md:60` 已写明 AC-1 计数「一律以脚本产出为准，不硬编码」。

**B（需裁决，已按最保守方式处理）**：`git add -A` 会把两个 **agent 运行时目录**一并入库：

- `.codebuddy/memory/2026-09-09.md`：本会话的 agent 记忆日志，非本 feature 交付物；入库后因「基线不存在」触发 AC-4 FAIL，使 `gate` 退出码变 1（§3.2 第一段实测：受检 195 / FAIL 1）。
- `.codebuddy/teams/dd199334a20c45409617b7b84e89f4b5/zh-final/*.json`：agent 团队运行时状态（无 md，不影响门禁）。

本人已用 `git restore --staged -- .codebuddy/memory .codebuddy/teams` 把两者移出索引（**工作区文件保留，未删除**），使门禁保持 PASS 与 staged 状态。若 PM 决定要把它们一并提交，则需要新的裁决（把 `.codebuddy/memory/` 也纳入豁免域，或单独放行该文件的 AC-4），本任务不擅自扩域。

**C（遗留）**：`check --diff` 的 FAIL 1 是 `.codebuddy/memory/2026-09-09.md`，提交前后恒为 1，与本次改动无关（改动前该子命令为受检 219 / FAIL 63）；`gate` 不受其影响（未跟踪 → 不在分母）。

**D（已知非阻塞）**：AC-8 段 2 历史域候选/命中数会随交接件入库增长（37/39 → 101/261）。这是「历史域」设计的本意，判定与退出码不变；若希望该数字也冻结，需要另开裁决。

## 8. 结论

`status: done` / `decision: proceed`。

- 提交前后 `gate` 退出码均为 **0**，`inventory` 恒等式与 `check` 受检/FAIL 数字**逐项完全相同**（§3.3）；
- 显式 `--files` 仍可检；NIT-2（非法 base → 2）未回退；
- 全部改动在 `files_owned` 内（1 个脚本 + 本报告），未改任何汉化正文、源码、测试；
- 最终状态：**staged 230 个文件**，`git status --porcelain | wc -l` = **232**（差值 2 为未跟踪的 `.codebuddy/memory/`、` .codebuddy/teams/`），**未 commit**，等 PM 统一提交。

---

## 9. R2 复审更正（2026-09-09，追加）

> **本章是追加的「已更正」说明，不修改 §0–§8 任何既有文字。**
> §4.1 与 §7-A 中关于「收窄豁免前缀」的**排他性/唯一性**论断已被实测证伪；凡与本章冲突处，**以本章为准**。
> 更正依据：`04-review/REVIEW-T15-T16.md` §3 问题表 major 第 1 条、§4 复审点 11(c)、§7.4。

### 9.1 被更正的原文（照录，未删改）

- §4.1 结论段：「结论：该变体确实能保住 `total=285 / 受检 231`（PM 期望数字），但会**双重破坏冻结的 AC-8** … 故**不可行** —— 除非修改 AC-8 补遗契约（回退 `solution-architect`，本任务无权）。」
- §7-A：「若 PM 要求分母必须仍是 285，**唯一实现路径**是收窄豁免前缀到本 feature 目录，而实测会破坏冻结的 AC-8（§4.1）→ 那必须先改 AC-8 契约并回退 `solution-architect`。」

**错误性质**：这两句断言的不是「某方案会失败」，而是「**不存在**别的方案」。作此断言前，我只测了**耦合变体**——收窄 `ARTIFACTS_EXEMPT_PREFIX` 时把 `:110` 的 `AC8_EXEMPT_PREFIX = ARTIFACTS_EXEMPT_PREFIX` 一并带窄，AC-8 三段因此被破坏。但 AC-1 的**分母豁免域**与 AC-8 的**排除前缀**是两个不同作用域的常量，**可以解耦**，而解耦后的组合我从未实测。**结论下早了。**

### 9.2 更正内容（第二条可行路径存在，已由我本轮独立复现）

**解耦变体定义**：`ARTIFACTS_EXEMPT_PREFIX` 收窄为 `.codebuddy/artifacts/2026-09-07-zh-docs-webui/`（只管 AC-1 分母），`AC8_EXEMPT_PREFIX` 恢复为**独立字面量** `.codebuddy/artifacts/`（AC-8 三段取值逐字不变）。

我本轮在 `/tmp` 复制脚本并只改这两个常量后实测（**未改动仓库内任何文件**，仅 `/tmp/t16_narrow.py`）：

```console
$ sed -n '60p;110p' /tmp/t16_narrow.py
ARTIFACTS_EXEMPT_PREFIX = ".codebuddy/artifacts/2026-09-07-zh-docs-webui/"
AC8_EXEMPT_PREFIX = ".codebuddy/artifacts/"  # 唯一豁免成员（复用 artifacts 豁免域常量，值不变）

$ python3 /tmp/t16_narrow.py gate --base 3ee655e3
gate: artifacts 豁免域 `.codebuddy/artifacts/2026-09-07-zh-docs-webui/` 已排除：64 个（…）
PASS AC-1 清单自洽
  SKIP_A=4 SKIP_B=47 SKIP_C=1 SKIP_D=2 TODO=29 ZH=202 total=285
PASS AC-2/3/4/6/7b/32 全量 check
  全量 check 受检 231，FAIL 0
  段 1 正式域（候选 515，已排除 .codebuddy/artifacts/）：0 命中 -> OK
  段 2 历史域（候选 101）：命中 264 处，全部位于 .codebuddy/artifacts/ -> OK（历史痕迹仍在）
  段 3 兜底域 site/.github/docs/extensions（全扩展名）：0 命中 -> OK
gate: PASS
NARROW_GATE_EXIT=0
```

**更正后的结论**：

1. **存在第二条可行路径**（即复审所称方案 B）：`total=285`、`受检 231`、AC-8 三段逐字不变、`gate` exit 0 / PASS，**且不需要回退 `solution-architect` 改 AC-8 契约**。
2. 因此 §4.1 的「不可行」与 §7-A 的「唯一实现路径是先改 AC-8 契约」均为**错误论断，即日作废**；不得被后续任何报告或派单引用。
3. §4.1 保留的**有效部分**：耦合变体（两个常量一起收窄）确实会双重破坏 AC-8——那是真实的实测结果，作废的只是由它推出的排他性结论。

### 9.3 PM 最终裁决：方案 A（维持 `.codebuddy/artifacts/` 整前缀豁免）

PM 已在 R2 返工执行书中裁决 **采纳方案 A**：维持整前缀豁免，`ARTIFACTS_EXEMPT_PREFIX` **不收窄**、`AC8_EXEMPT_PREFIX` 继续引用它（两常量**不**解耦）。理由（据复审 §4 复审点 11(d) 与 PM 裁决）：

- **(a) 与 SKIP 域一致、对未来所有 feature 一致**：豁免是「按路径前缀判定、与基线无关」，与 `SKIP_A..SKIP_D` 同源；收窄到单 feature 目录，下一个 feature 的交接件会重踩「新文件无基线 → AC-4 全文记为 added」的大面积假 FAIL（§6 实测：撤销豁免 `受检 295 / FAIL 64`）。
- **(b) 域内文件当前全部 PASS，豁免未掩盖失败**：解耦变体实测 `受检 231 / FAIL 0`，即 37 个历史交接件（101 − 64 = 37）在当前判据下**无一失败**；整前缀方案把它们一并出域，没有掩盖任何既有失败。
- **(c) 该域并未彻底脱离约束**：域内文件仍是 AC-8 段 2 的「历史域」（段 2 反向断言 `README.zh-CN` 命中必须仍在、命中须全部位于域内），且 AC-7a 的改名/删除判定未加豁免。

**代价（必须显式记录，不得省略）**：AC-1 分母由 **285 变为 248**、全量 check 受检由 **231 变为 194**（差值 37 = 基线已入库的历史交接件）。这是契约层范围变更，**须由契约层追认** —— 建议由 PM 指派 `doc-writer` 在 `.codebuddy/artifacts/2026-09-07-zh-docs-webui/00-decisions.md` 登记一条裁决：

> AC-1 分母 = 全仓 md 减显式豁免域 `.codebuddy/artifacts/`；域内文件不进分母、不受 AC-2/4/6/7b/32 与 AC-8 段 1 约束，仍为 AC-8 段 2 历史域、仍受 AC-7a 约束。

该项**不在本任务 `files_owned`**（需改 `00-decisions.md`），本任务未擅自登记。

### 9.4 本轮（R2）随裁决一并落地的代码/文案改动

| # | 文件 | 改动 | 对应复审项 |
|---|---|---|---|
| 1 | `scripts/check-zh-docs.py:872` | 恒等式行「全仓 md 总数 = %d」→「受检 md 总数（已排除 artifacts 豁免域） = %d」（**只改文案**：格式参数个数与顺序、计数逻辑、判据、阈值一律未动） | major 第 2 条的一半 |
| 2 | `scripts/check-zh-docs.py:27` | 同一措辞的注释（AC-1 桶定义）同步改为「受检 md 总数（已排除 artifacts 豁免域）」 | 同上（保持注释与输出一致） |
| 3 | 本文件 | 追加本章（§9） | major 第 1 条 |

改后实测（改文案不影响任何计数）：

```console
$ python3 -m py_compile scripts/check-zh-docs.py
PYCOMPILE_EXIT=0

$ python3 scripts/check-zh-docs.py gate --base 3ee655e3
PASS AC-1 清单自洽
  SKIP_A=4 SKIP_B=47 SKIP_C=1 SKIP_D=2 TODO=27 ZH=167 total=248
PASS AC-2/3/4/6/7b/32 全量 check
  全量 check 受检 194，FAIL 0
PASS AC-8 外链残留
  段 1 正式域（候选 515，已排除 .codebuddy/artifacts/）：0 命中 -> OK
  段 2 历史域（候选 101）：命中 264 处，全部位于 .codebuddy/artifacts/ -> OK
  段 3 兜底域 site/.github/docs/extensions（全扩展名）：0 命中 -> OK
gate: PASS
GATE_EXIT=0

$ python3 scripts/check-zh-docs.py inventory --base 3ee655e3 | grep -n "受检 md 总数"
10:SKIP_A(4) + SKIP_B(47) + SKIP_C(1) + SKIP_D(2) + TODO(27) + ZH(167) = 248 ；受检 md 总数（已排除 artifacts 豁免域） = 248 ；恒等式成立

$ python3 scripts/check-zh-docs.py inventory --base 3ee655e3 | tail -n 1
inventory: total=248 SKIP_A=4 SKIP_B=47 SKIP_C=1 SKIP_D=2 TODO(EN=0,MIXED=27) ZH=167 恒等式=OK

$ grep -rn "全仓 md 总数" scripts/check-zh-docs.py
（无输出，exit 1 —— 旧措辞零残留）
```

**口径自洽性复核**：`git ls-files -- '*.md'` = **349**，其中 `.codebuddy/artifacts/` 域 **101**（`git -c core.quotePath=false` 计数），349 − 101 = **248** = 脚本输出的分母，与改动后的措辞「已排除 artifacts 豁免域」完全相符（改前打印「全仓 md 总数 = 248」与真实 349 自相矛盾，即复审 major 第 2 条所指）。

> 自指注记（如实披露）：上面这段 gate 输出是「本轮代码/文案改动完成后、本 §9 与 `T-15.md` §10 追加**之前**」那一次运行（段 2 命中 **264**）；两份报告追加完成后复跑，同一数字变为**命中 265**（追加正文中引用了 `README.zh-CN` 字样），其余逐字相同、`GATE_EXIT=0` 不变。这是 AC-8 段 2「历史域命中数随域内内容增长」的既有设计（同 §3.3 表中已记录的 37/39 → 101/261 增长），**判定（全部位于域内 -> OK）与退出码 0 不变**。

### 9.5 仍待 PM / 契约层完成（不在本任务 `files_owned`，本轮未做）

1. `00-decisions.md` 登记 §9.3 的新分母定义（**建议 owner：`doc-writer`，由 PM 派单**）——这是复审「可推进提交」的第 1 条判据。
2. `AGENTS.md:37` 与 `STATUS.md:115` 关于「`rustcode daemon` 没有 `--host`」的失实文字（复审 major 第 3 条，归 PM / T-15 之外）。
3. 复审 minor：`OWNED_ELSEWHERE` 第 3 个成员（`.codebuddy/artifacts/2026-09-07-zh-docs-webui/`）已为死配置——**本轮未改**，因 PM 执行书只授权 MAJOR-1 / MAJOR-2 两项；保留原状不影响任何判据（`is_owned_elsewhere()` 的输入已不含 artifacts 域）。建议随下轮一并处理。
4. 复审 minor：§1 hunk 计数写成 6 而表格列 7 行、§4.2 把 quotePath 后果写成「静默」——**文字勘误未在本轮改**（§0–§8 按裁决只追加不改写）；如需订正，请 PM 明确授权后再动。正确说法在此记录在案：改动点是 6 处，`gate` 提示与 `inventory` 注记同属第 6 项；`list_md_files()` 路径并非静默，带引号路径会进入分母并在 `check_file()` 抛 `EnvError` → **exit 2（响亮失败）**，只有 `resolve_files()` 路径（有 `isfile` 过滤）才是真静默漏检。

### 9.6 状态

T-16 在 `files_owned` 内的返工项（MAJOR-1 报告更正、MAJOR-2 措辞）**已完成**；剩余两条判据（§9.5 第 1、2 项）属 PM / doc-writer 的 `files_owned`，本任务不越界。`gate --base 3ee655e3` 仍 exit 0 / PASS，未 commit。
