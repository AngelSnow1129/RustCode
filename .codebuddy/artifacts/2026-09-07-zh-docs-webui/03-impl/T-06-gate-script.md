---
kind: implementation
id: T-06
from: code-implementer
to: [code-reviewer]
feature: 2026-09-07-zh-docs-webui
status: done
decision: escalate
requires: [T-01]
files_owned:
  - scripts/check-zh-docs.py
architecture_constraints:
  touches_runtime_lifecycle: false
  touches_persistence: false
  touches_cross_crate_deps: false
created: 2026-09-08
---

# T-06 · 门禁脚本精确化（AC-4 授权移除 + 新增 SKIP_D）

> 字段含义澄清（避免误读）：
> `status: done` = 本任务 `files_owned`（`scripts/check-zh-docs.py`）内的全部改动已完成并通过自验证；
> `decision: escalate` = **验证项 3 未达成**，根因在 `files_owned` 之外的 `.codebuddy/agents/doc-writer.md`
> （D-27 交付物），需在"修 md"与"改契约"之间由上游裁决，本任务无权二选一。详见 §5。

- 基线：`branch=dev commit=3ee655e3`，worktree dirty（本 feature 未提交，脚本本身为未跟踪新增文件）
- 改动文件数：1（`scripts/check-zh-docs.py`）
- 未改动：任何 md、任何 Rust 代码、`.gitignore`、`Cargo.toml`
- 任务 ID 冲突说明：`02-tasks.md` 中既有的 `T-06` 是"清除 md 中 `127.0.0.1` 叙述"（owner=`doc-writer`，
  files_owned = hostscan 输出文件）。本任务由编排者指派为"门禁脚本改动"，与之同名不同内容。
  本报告落盘为 `T-06-gate-script.md` 以示区分，**建议编排者后续把脚本改动任务改号（如 T-01b / T-08）**，
  否则 `04-review/T-06.md` 与本报告会指向不同交付物。

---

## 1. 改动清单

所有行号为改动后 `scripts/check-zh-docs.py` 的行号。

| # | 位置 | 变更 |
|---|---|---|
| 1 | `26` 分类桶注释 | 恒等注释改为 `SKIP_A + SKIP_B + SKIP_C + SKIP_D + TODO + ZH` |
| 2 | `30-38` 新增常量 `SKIP_D_PROMPTS` | 恰含 2 个固定成员（`evals/deepseek-v4-flash/prompts/codex-judge.md`、`.../codex-report.md`）；注释写明"与 SKIP_B 同源、固定清单禁止扫描扩充"及裁决来源 |
| 3 | `41-51` 新增常量 `AC4_ALLOWED_REMOVED` | `frozenset({"README.zh-CN.md"})`；注释写明裁决依据（STATUS.md §目标 2：汉化 `README.md`、删除 `README.zh-CN.md`）与边界（只允许消失、禁止新增、禁止越界、不按目录豁免） |
| 4 | `75` `SKIP_BUCKETS` | 追加 `"SKIP_D"`（gate 的受检集 `bucket not in SKIP_BUCKETS` 因此自动排除这 2 个文件） |
| 5 | `152` `classify()` docstring | 返回集合增列 `SKIP_D` |
| 6 | `166-167` `classify()` | 在 `SKIP_C` 判定之后、`is_owned_elsewhere` 之前新增 `if path in SKIP_D_PROMPTS: return "SKIP_D"`；命中为**全等路径匹配**，非前缀/通配 |
| 7 | `412-425` 新增 `ac4_span_diff()` | 完整（不截断）多重集差 `(removed_all, added_all)`。不复用 `multiset_diff()`：后者 `limit=10` 会截断，若越界移除落在第 11 项之后会漏判 |
| 8 | `427-432` 新增 `split_removed()` | 把 `removed` 拆成 `(授权移除, 越界移除)`，供 FAIL 详情区分二者 |
| 9 | `480-506` `check_file()` AC-4 段 | 判据由 `old_spans != new_spans → FAIL` 精确化为：`added` 非空 **或** `illegal_removed` 非空 → FAIL（详情分三行打印"越界移除 / 新增 / 授权移除"）；否则 PASS 并记 note `AC-4-allowed-removed` |
| 10 | `659` `inventory_counts()` | 计数桶新增 `"SKIP_D": 0` |
| 11 | `668` `render_inventory()` | `skip` 累加 `counts["SKIP_D"]` |
| 12 | `684-690` 恒等式串 | 改为 `SKIP_A(%d) + SKIP_B(%d) + SKIP_C(%d) + SKIP_D(%d) + TODO(%d) + ZH(%d) = %d` |
| 13 | `704-707` 桶表 | 新增 `SKIP_D` 行（期望 2，说明含裁决依据） |
| 14 | `719` 分节清单 | 新增 `("SKIP_D", "D 类 · 运行时 prompt 载荷（跳过）")` |
| 15 | `757-776` `cmd_inventory()` | stdout 汇总串新增 `SKIP_D=%d`；`ok` 条件新增 `counts["SKIP_D"] == 2` |
| 16 | `908-930` `gate_check_ac1()` | `skip` 累加 SKIP_D；期望表新增 `("SKIP_D", 2)`；PASS 详情串新增 `SKIP_D=%d` |

**未改动（红线，逐项确认）**：

- `EN_RATIO_DEFAULT = 0.05` 与 AC-2 阈值判定逻辑 —— 未动。
- AC-6（frontmatter 键名集合相等）、AC-7b（链接目标多重集相等）、AC-32（Emoji 不增加）、
  AC-5（rules 无改动 / setup-seeds 有改动且无 `^[+-]name:`）、AC-8（两次 grep 0 命中）—— 未动。
- AC-7a（`:994`）与 hostscan（`:860`）内部的 `SKIP_B_RULES or SKIP_C_VERBATIM` 跳过条件 —— **故意不加 SKIP_D**：
  二者判据被红线要求保持不变；且实测这 2 个文件不出现在 hostscan 结果中（见验证 4），行为零变化。
- `check --files / --diff` 的解析逻辑 —— 未动（显式指定即显式受检，与 SKIP_A/B/C 既有行为一致；
  且这 2 个文件在工作区未改动，`git status` 无记录，不会进入 `--diff`）。

---

## 2. 契约符合性

对照 `01-design.md` 与编排者任务说明：

| 契约 | 实现 | 结论 |
|---|---|---|
| 新增模块级常量，内容**恰为** `{"README.zh-CN.md"}` | `AC4_ALLOWED_REMOVED = frozenset({"README.zh-CN.md"})`（`:51`） | 一致 |
| 判定：`added` 必须为空；`removed` 必须是该常量集合的子集 | `:485-490`：`if added or illegal_removed: FAIL` | 一致 |
| 不得放宽为"允许任意 span 移除" | 仅允许常量内成员；`frozenset` 不可变；无目录/文件豁免 | 一致 |
| 失败信息能区分"被允许移除"与"越界移除" | 三行分列：`越界移除（…判 FAIL）` / `新增（任何新增均判 FAIL）` / `授权移除（…允许消失）` | 一致 |
| SKIP_D 成员恰为 2 个、固定清单 | `SKIP_D_PROMPTS` 元组硬编码 2 条，全等匹配，无扫描 | 一致 |
| inventory 输出 `SKIP_D=2`，AC-1 恒等式 `4+47+1+2+TODO+ZH=286` 自洽 | 见验证 1 | 一致 |
| 2 个文件不再进入 gate 受检集 | `SKIP_BUCKETS` 含 SKIP_D → `cmd_gate` 的 `targets` 自动排除；受检数 234→232 | 一致 |
| 不借 SKIP_D 豁免其它文件 | 仅 2 条全等路径；`SKIP_D=2` 且恒等式仍成立（若多豁免会从 TODO/ZH 侧暴露） | 一致 |
| AC-2 阈值 0.05 不变 / AC-5、AC-6、AC-7a、AC-7b、AC-32、AC-8、hostscan 判据不变 | 见上"未改动" | 一致 |

**已声明的偏差（1 项，且唯一）**：`01-design.md:483-485` 的伪代码为
`if sorted(CODE_SPANS(old)) != sorted(CODE_SPANS(new)): FAIL`。本任务把该判据精确化为
`new ⊆ old 且 (old - new) ⊆ {"README.zh-CN.md"}`。这是**判据精确化，不是降阈值**，
理由见 §6。除此之外无其它偏差。

**为何是精确化而非降阈值（AGENTS.md 要求说明）**：

1. 触发原因是**外部事实变更**而非质量退化：本 feature 用户裁决删除 `README.zh-CN.md`
   （`STATUS.md` §目标 2），指向该文件的 code span 消失是删除动作的必然副产物。
2. 被放宽的只有一个**具体、可枚举、有书面裁决支撑的字面量**，不是比例、不是计数、不是阈值：
   `AC-2` 的 0.05、`AC-32` 的 `<=`、`AC-6`/`AC-7b` 的相等判定全部保持原样。
3. 方向是**单向收紧的**：`added` 仍然零容忍（仍 FAIL），只有 `removed` 且仅当 ∈ 常量集合才放行。
   任何越界移除（例如本次实测到的 `cargo`，见 §5）依旧 FAIL —— 这正是"精确化"的证据。
4. 不可推广：没有按目录、按桶、按文件名的豁免入口；新增成员必须改常量并注明裁决，
   属契约变更，需回退 `solution-architect`。

---

## 3. 自验证证据（真实执行）

全部命令在 `/workspace/RustCode`（branch `dev`，基线 `3ee655e3`）执行。

### 验证 1 · inventory

```bash
python3 scripts/check-zh-docs.py inventory
```

- 退出码：**0**
- 关键输出：

```
SKIP_A(4) + SKIP_B(47) + SKIP_C(1) + SKIP_D(2) + TODO(33) + ZH(199) = 286 ；全仓 md 总数 = 286 ；恒等式成立

| SKIP_D | 2 | 2 | 运行时 prompt 载荷（`evals/deepseek-v4-flash/prompts/codex-{judge,report}.md`）；与 SKIP_B 同源，汉化会改变评测语义与输出解析，不汉化 |

## D 类 · 运行时 prompt 载荷（跳过）（2）
| `evals/deepseek-v4-flash/prompts/codex-judge.md` | 0 | 6 | 0.000 |  |
| `evals/deepseek-v4-flash/prompts/codex-report.md` | 0 | 12 | 0.000 |  |

inventory: total=286 SKIP_A=4 SKIP_B=47 SKIP_C=1 SKIP_D=2 TODO(EN=3,MIXED=30) ZH=199 恒等式=OK
```

- 断言：含 `SKIP_D=2` [OK]；`4+47+1+2+33+199 = 286` 恒等式成立且 exit 0 [OK]。
- 对比基线的 `TODO(EN=5,MIXED=30)`：这 2 个文件从 EN 桶移入 SKIP_D，`EN 5→3`，总数不变 [OK]。

### 验证 2 · gate

```bash
python3 scripts/check-zh-docs.py gate --base 3ee655e3
```

- 退出码：**1**（预期：仍有残留 FAIL）
- 小节判定：

```
PASS AC-1 清单自洽
  SKIP_A=4 SKIP_B=47 SKIP_C=1 SKIP_D=2 TODO=33 ZH=199 total=286
FAIL AC-2/3/4/6/7b/32 全量 check
  全量 check 受检 232，FAIL 6
PASS AC-5 运行时载荷
PASS AC-7a 无改名/删除
FAIL AC-8 外链残留
gate: FAIL
```

- 逐条 FAIL 文件（`grep -n "^    FAIL "`，6 条）：

```
    FAIL .codebuddy/agents/doc-writer.md [AC-4] en=0/41=0.0000
    FAIL README.md [AC-2] en=388/409=0.9487
    FAIL crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/mcp-servers.md [AC-2] en=103/172=0.5988
    FAIL crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/plugins-reference.md [AC-2] en=56/61=0.9180
    FAIL crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/skills-reference.md [AC-2] en=82/94=0.8723
    FAIL crates/rustcode-capabilities/assets/setup-seeds/skills/rustcode-automation-recommender/references/subagent-templates.md [AC-2] en=103/114=0.9035
```

- 与基线（8 条）的 diff（真实 `diff` 输出）：

```
4c4
<   SKIP_A=4 SKIP_B=47 SKIP_C=1 TODO=35 ZH=199 total=286
---
>   SKIP_A=4 SKIP_B=47 SKIP_C=1 SKIP_D=2 TODO=33 ZH=199 total=286
6c6
<   全量 check 受检 234，FAIL 8
---
>   全量 check 受检 232，FAIL 6
13,14d12
<     FAIL evals/deepseek-v4-flash/prompts/codex-judge.md [AC-2] en=6/6=1.0000
<     FAIL evals/deepseek-v4-flash/prompts/codex-report.md [AC-2] en=12/12=1.0000
```

- 结论：改动 2 达成（codex 两个 prompt 已移出受检集，FAIL 数 8→6）。
  **数量与任务说明的"6 个"一致，但构成与任务说明的括号描述不一致**：任务说明预期
  "`README.md` + setup-seeds 的 3 个 references + doc-writer 不再计入"，
  实测为 `README.md` + setup-seeds **4** 个 references + `doc-writer.md` 仍 FAIL（见验证 3）。
  即：`doc-writer.md` **未**如预期PASS，setup-seeds 实际是 4 个而非 3 个文件。
- 另：`AC-8 外链残留` FAIL 为**改动前既有**（基线 gate 同样 FAIL），与本任务无关，未触碰其判据。

### 验证 3 · doc-writer.md 单文件 check

```bash
python3 scripts/check-zh-docs.py check --files .codebuddy/agents/doc-writer.md
```

- 退出码：**1**（任务说明预期 0 → **未达成**，原因见 §5）
- 输出（前 5 行，AC-6-desc-changed 长行已省略）：

```
FAIL .codebuddy/agents/doc-writer.md en=0/41=0.0000
  [AC-4] AC-4 code span 多重集不等 (old=46 new=44)
  [AC-4]   越界移除（不在 AC4_ALLOWED_REMOVED 内，判 FAIL）: ['cargo']
  [AC-4]   新增（任何新增均判 FAIL）: 无
  [AC-4]   授权移除（README.zh-CN.md 已裁决删除，允许消失）: ['README.zh-CN.md']
```

- 判读：改动 1 **本身已生效** —— `README.zh-CN.md` 被正确识别为"授权移除"并单独归档，
  不再计入 FAIL 理由；剩余 FAIL 完全由越界移除的 `cargo` 引起。

### 验证 3b · 新判据两分支的合成文本验证（补充证据，不落盘、不改仓库）

用 `importlib` 载入脚本、替换 `read_work_file` 喂入合成文本（仅内存，未写任何文件）：

```
== A_only_allowed_removed -> PASS
   [note AC-4-allowed-removed] 授权移除 ['README.zh-CN.md']（README.zh-CN.md 已裁决删除）；无新增 span
== B_added_span -> FAIL
   [FAIL AC-4] AC-4 code span 多重集不等 (old=46 new=46)
   [FAIL AC-4]   越界移除（不在 AC4_ALLOWED_REMOVED 内，判 FAIL）: 无
   [FAIL AC-4]   新增（任何新增均判 FAIL）: ['FAKE-NEW-SPAN']
   [FAIL AC-4]   授权移除（README.zh-CN.md 已裁决删除，允许消失）: ['README.zh-CN.md']
```

- A：仅授权移除 → PASS（含 note）[OK]
- B：任意新增 span → 仍 FAIL [OK]（证明未放宽为"允许 span 变动"）

### 验证 4 · hostscan

```bash
python3 scripts/check-zh-docs.py hostscan | tail -20
```

- 退出码：**0**
- 输出（`tail -20`）：

```
docker/README.md
docs/superpowers/plans/2026-05-29-webui.md
docs/superpowers/specs/2026-05-29-webui-design.md
extensions/jetbrains/PRIVACY.md
extensions/jetbrains/README.md
extensions/jetbrains/docs/jetbrains.md

## 明细

docker/README.md:186: > compose 端口默认仅绑定 `127.0.0.1`（本机访问）。如需从局域网/NAS 访问，用
docker/README.md:210: 默认端口只绑定宿主机 `127.0.0.1`。仅在可信局域网中需要手机直连时，使用：
docs/superpowers/plans/2026-05-29-webui.md:44: - daemon 默认 `127.0.0.1:13456`；CLI `Commands::Daemon` 通过 re-exec `rustcode-daemon` 二进制启动（`rustcode-cli/src/main.rs:930+`）
docs/superpowers/specs/2026-05-29-webui-design.md:34: - `crates/rustcode-daemon`（axum）默认绑 `127.0.0.1:13456`，已提供：`/chat`(SSE 流式)、
docs/superpowers/specs/2026-05-29-webui-design.md:142: - server 默认只绑 `127.0.0.1`（现状已是）。
extensions/jetbrains/PRIVACY.md:19: 默认情况下，插件会连接到位于 `127.0.0.1:13456` 的 RustCode 后端。插件也可以在你的机器上启动打包的或已配置的后端进程。插件向该本地后端发送请求，以便后端运行编码智能体工作流、管理会话、与模型供应商通信，并执行经用户批准的操作。
extensions/jetbrains/README.md:241: 插件默认将后端主机设置为 `127.0.0.1`，使用后端的 HTTP API，并且不收集插件遥测数据。在发送编辑器选中内容或文件作为聊天上下文之前，会应用敏感路径分类。
extensions/jetbrains/docs/jetbrains.md:44: - 主机与端口，默认为 `127.0.0.1:13456`
extensions/jetbrains/docs/jetbrains.md:53: 默认情况下，插件与位于 `127.0.0.1` 的本地后端通信。如果你配置了其他主机，请在发送编辑器选中内容或文件作为聊天上下文之前先评估隐私与安全影响。

hostscan: 命中文件 6，命中行 9
```

- 与改动前基线逐字一致（`hostscan: 命中文件 6，命中行 9`），行为未变 [OK]；
  且这 2 个 codex prompt 文件本就不在命中集内（`grep -c codex` = 0），故红线"hostscan 行为不变"成立。

### 附加冒烟 · gate --report

```bash
python3 scripts/check-zh-docs.py gate --base 3ee655e3 --report /tmp/gate_report.md
```

- 退出码：**1**（与不带 `--report` 一致）；`gate: 报告已写入 /tmp/gate_report.md`，231115 字节，写入路径未崩溃 [OK]

### 编译/静态检查

```bash
python3 -m py_compile scripts/check-zh-docs.py   # -> COMPILE_OK，退出码 0
```

（本任务为纯 Python stdlib 脚本，不涉及 Rust：未改动任何 `.rs`，故未运行 cargo。）

---

## 4. 验收标准对照

| AC / 验收项 | 结果 | 说明 |
|---|---|---|
| 改动 1：常量恰为 `{"README.zh-CN.md"}` | 满足 | `:51`，`frozenset` |
| 改动 1：`added` 空 且 `removed ⊆ 常量` 才 PASS | 满足 | `:485-506`；验证 3b 的 A/B 两支分别证明 |
| 改动 1：不得放宽为任意移除 / 不得按目录豁免 | 满足 | 越界移除 `cargo` 仍 FAIL（验证 3 实测） |
| 改动 1：FAIL 信息可区分允许/越界 | 满足 | 三行分列输出 |
| 改动 2：SKIP_D 恰 2 个、固定清单 | 满足 | `:34-38`；inventory `SKIP_D=2` |
| 改动 2：inventory 输出 `SKIP_D=2` 且 AC-1 自洽 PASS | 满足 | 验证 1，exit 0 |
| 改动 2：2 文件不进 check/gate 受检集 | 满足（gate） | 受检 234→232，FAIL 列表已无此二者；`check --files/--diff` 为显式指定路径，行为与 A/B/C 类一致未改 |
| 改动 3：AC-2 阈值 0.05 不变 | 满足 | 未触碰 |
| 改动 3：AC-5/6/7a/7b/32/8 判据不变 | 满足 | 未触碰 |
| 改动 3：hostscan 行为不变 | 满足 | 验证 4 逐字一致 |
| 验证 1（inventory 含 `SKIP_D=2` + AC-1 PASS） | **通过** | exit 0 |
| 验证 2（gate FAIL 集合变 6，codex 两件与 doc-writer 不计入） | **部分通过** | 数量 6 [OK]；codex 两件已移除 [OK]；**`doc-writer.md` 仍在列** [未达成]（§5） |
| 验证 3（`check --files doc-writer.md` PASS） | **未通过** | exit 1，越界移除 `cargo`（§5） |
| 验证 4（hostscan 可运行） | **通过** | exit 0，输出与基线一致 |

---

## 5. 遗留风险与后续项

### 5.1 需上游裁决（本任务 escalate 的唯一事由）

`.codebuddy/agents/doc-writer.md`（归属 **D-27**，批次 9；不在本任务 `files_owned`）除删掉
`README.zh-CN.md` span 外，还额外丢掉了 `cargo` 这个 code span：

- 基线（`3ee655e3`）第 65 行：`执行 \`cargo\` 等构建/测试命令`
- 工作区第 65 行：`执行 cargo/npm 构建与测试命令（**例外**：允许且应当运行 python3 scripts/check-zh-docs.py 做文档自检）`

即：反引号被去掉，`code_spans` 少一项。这在**任何** AC-4 严格解释下都是真实的 span 丢失，
与本 feature 的 README 裁决无关，因此不在 `AC4_ALLOWED_REMOVED` 授权范围内。

二选一，**我无权自行决定**：

- **选项 A（推荐）**：回退 D-27 的 owner 修 `.codebuddy/agents/doc-writer.md:65`，
  保留新语义但恢复 span，例如写成 `执行 \`cargo\`/\`npm\` 构建与测试命令（**例外**：…）`。
  改完后验证 2 的 FAIL 集合应降为 5（README + 4 个 setup-seeds），验证 3 转 PASS。
  **不触碰契约，不需要架构师介入。**
- **选项 B**：由 `solution-architect` 明确授权把 `cargo` 加入 `AC4_ALLOWED_REMOVED`。
  我不建议：该 span 的丢失与 README 删除无因果关系，放行为"按文件当前状态补票"，
  会打开"任何一次文档改写丢 span 都能事后追认"的口子，实质削弱 AC-4。

在裁决落地前，`gate` 与 `check --files .codebuddy/agents/doc-writer.md` 会持续 FAIL。

### 5.2 残留 FAIL 清单（均不属于本任务，不在本任务 `files_owned`）

| 文件 | 失败 AC | 归属 | 说明 |
|---|---|---|---|
| `README.md` | AC-2（en=388/409） | **D-35**（批次 13：根 README 汉化 + 删除 `README.zh-CN.md`） | 尚未汉化 |
| `.../references/mcp-servers.md` | AC-2（0.5988） | **D-34**（批次 12：setup-seeds 6 件） | 汉化未完成 |
| `.../references/plugins-reference.md` | AC-2（0.9180） | **D-34** | 同上 |
| `.../references/skills-reference.md` | AC-2（0.8723） | **D-34** | 同上 |
| `.../references/subagent-templates.md` | AC-2（0.9035） | **D-34** | 同上 |
| `.codebuddy/agents/doc-writer.md` | AC-4（`cargo` 越界移除） | **D-27**（批次 9） | 见 §5.1 |
| AC-8 外链残留（小节级 FAIL） | AC-8 | D-35 删除 `README.zh-CN.md` 后需清外链 | 改动前既已 FAIL，判据未动 |

注：任务说明中出现的 "D-34b" 在 `02-tasks.md` 中不存在，上述归属按 `02-tasks.md:568`（D-34）
与 `02-tasks.md:588`（D-35）、`02-tasks.md:477`（D-27）的 `files_owned` 实测判定。

### 5.3 其它已知风险

1. **任务 ID 冲突**：`02-tasks.md` 的 `T-06` 是 hostscan 文档改写任务，与本任务同名。
   若后续生成 `04-review/T-06.md`，需明确指向哪一份；建议编排者把脚本任务改号。
2. **脚本未纳入版本控制**：`scripts/check-zh-docs.py` 目前是未跟踪文件（`??`），
   本 feature 一旦分批 commit，需确保 T-01 + 本任务一起提交，否则基线 `3ee655e3` 上不存在该脚本。
3. **SKIP_D 与 AC-1 的耦合**：`cmd_inventory` / `gate_check_ac1` 现在硬断言 `SKIP_D == 2`。
   若将来裁决再增/减运行时 prompt 文件，必须同步改常量与该期望值，否则 inventory/gate 会 FAIL
   （这是有意的 fail-closed 设计，不是缺陷）。
4. **hostscan 未同步排除 SKIP_D**：按红线"hostscan 行为不变"保留。若这 2 个文件将来被写入
   "默认 127.0.0.1"叙述，会出现在 hostscan 列表里但不受 gate 检查 —— 届时需由编排者决定是否同步。
5. **AC-8 仍 FAIL**：根因是 `README.zh-CN` 外链残留（D-35 未完成删除/清理），
   本任务按红线未触碰其判据，故不修。

---

## 6. 结论

`scripts/check-zh-docs.py` 的两项改动均已按契约实现，红线判据零改动，
格式与语义自洽（`python3 -m py_compile` 通过；未引入新依赖；注释全 ASCII 标签、无 Emoji 新增）。
唯一未达成的验收项是验证 3，根因在 D-27 的交付文件，需上游按 §5.1 的 A/B 选项裁决。
建议：**走选项 A**。
