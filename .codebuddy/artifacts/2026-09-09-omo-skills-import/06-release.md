---
kind: release
id: RELEASE-001
from: doc-writer
to: [project-manager]
feature: 2026-09-09-omo-skills-import
status: done
decision: proceed
requires: [TEST-001]
files_owned:
  - .codebuddy/artifacts/2026-09-09-omo-skills-import/01-import-plan.md
  - .codebuddy/artifacts/2026-09-09-omo-skills-import/03-impl/T-01.md
  - .codebuddy/artifacts/2026-09-09-omo-skills-import/06-release.md
  - /root/.rustcode/skills/rustcode-ast-grep/SKILL.md
  - /root/.rustcode/skills/rustcode-debugging/SKILL.md
  - /root/.rustcode/skills/rustcode-git-master/SKILL.md
  - /root/.rustcode/skills/rustcode-parallel-review/SKILL.md
  - /root/.rustcode/skills/rustcode-refactor/SKILL.md
  - /root/.rustcode/skills/rustcode-remove-ai-slops/SKILL.md
created: 2026-09-09
---

# 交付说明：RustCode 6 个原创运行时 skill + OMO 导入方案

## 1. 行为变化

**用户与调用方可感知的变化：** 在 `~/.rustcode/skills/` 新增 6 个目录式技能，由
`standard_skill_dirs`（`crates/rustcode-capabilities/src/skills/registry.rs:238`）原生加载，
无需任何配置改动、无需重启安装流程、不需要新 feature 开关。

| 技能 | 触发后可获得的能力 | 前提/依赖 |
| --- | --- | --- |
| `rustcode-ast-grep` | 按 AST 形状搜索；按证据批量改写 | `ast_grep` 工具可用；需要 `sg` 改写时要求主机已装 `ast-grep`/`sg`，否则按降级路径走 |
| `rustcode-debugging` | 取证驱动的运行时排障流程 | 全部经 `bash` + `cargo check`/`clippy`/`test`；**不依赖 LSP** |
| `rustcode-refactor` | 行为锁定在先的重构流程 | 需要可跑的测试/快照作为基线 |
| `rustcode-git-master` | 原子提交与历史调查 | rustcode 无 git 工具，全部经 `bash` 调 `git` |
| `rustcode-remove-ai-slops` | 按本仓口径的 AI 生成代码异味清理 | 对齐 `AGENTS.md` 硬约束 |
| `rustcode-parallel-review` | 多视角并行**补充**审查 | 与内置 `code_review` / `/review` 互补，非替代 |

**前后对比：**
- 前：只有内置 `code_review` 子 agent（`/review`）一条审查路径；无结构性搜索、无取证式调试、
  无原子提交、无 slop 清理的可复用流程；涉及这些任务时每次靠临时提示词，判据不一致。
- 后：上述 6 类任务有固定阶段划分、门禁判据与检查清单，且判据与 `AGENTS.md`、
  `cargo check`/`clippy`/`test` 对齐。

**迁移步骤：** 无。本次为纯新增，无删除、无退役接口、无配置键变化、无命令改名。
用户若要停用某个技能，删除对应目录即可（见 §6）。

**能力边界（重要，避免误用）：**
- `rustcode-ast-grep` 的 `ast_grep` 工具**只读**，不能靠它改写；
- `rustcode-debugging` **不提供 LSP 诊断**：`diagnostics` 工具全仓未注册，
  `lsp` 工具受 `lsp.enabled` 配置门控（`codeintel/mod.rs:129`），默认不假设可用；
- `rustcode-parallel-review` **只读**，不派 `worker` 子代理，不在本轮改代码。

## 2. 风险

| 类别 | 风险 | 等级 | 说明与缓解 |
| --- | --- | --- | --- |
| 兼容性 | 技能与内置 `code_review` / `/review` 撞车 | 中 | 已在 `rustcode-parallel-review` 正文首段用分工表写死"先跑内置、本技能叠加、不替代"；并通过改名（`review-work` -> `parallel-review`）降低语义混淆 |
| 兼容性 | `allowed-tools` 字段是否真正生效未验证 | 低 | `skill.rs:17-18` 注明其为元数据、L1 不强制，属 L2 审批策略关注面；未生效则退化为提示，不会造成功能错误 |
| 性能 | 技能文本进入上下文 | 低 | 6 个技能共 731 行；目录式技能按需注入，非常驻 |
| 数据 | 无持久化写入、无状态文件、无网络出站 | 无 | 本批不包含任何脚本或二进制 |
| 数据 | `rustcode-git-master` 涉及 git 写操作 | 中 | 已用硬性禁止清单约束：禁止 push / `--force` / `--force-with-lease` / `--amend`（未明确要求）/ `--no-verify` / `-A` / `reset --hard` / `clean -fd` / `checkout -- .`；提交前必须 `request_user_input` 确认 |
| 合规 | SUL-1.0 文本污染 | 中（已控） | 只取骨架不取正文；6 个技能均有「来源与适配」声明；`frontend`/`programming` 等重资产未引入，避免第三方 vendored 内容的许可链风险 |
| 回滚代价 | 极低 | 低 | 纯新增文件，删除目录即回滚，无状态、无迁移、无下游依赖（见 §6） |

## 3. 验证结果

实际执行与结论（原始数据见 `03-impl/T-01.md` §3，此处不重写数据）：

1. **仓库零改动核验** —— `git status --short` / `git diff --stat`：
   已跟踪文件中仅 `README.md` 有改动，**经 mtime 与首轮基线比对确认非本次引入**
   （属前序/并行任务 `2026-09-09-readme-account-cmds`）；
   本次净新增 = 6 个 `~/.rustcode/skills/*/SKILL.md` + 本 artifacts 目录下的文档。
   `.rs` / `Cargo.toml` / `AGENTS.md` / `docs/**` / `README*` 零改动。
   **[ERROR] 判定：验证 1 的"输出为空"目标未达成**，原因与归属已逐条溯源（见 T-01 §3.1），
   属环境既有状态，非本次任务引入，不做掩盖。
2. **frontmatter 与命名静态校验** —— 复刻 `skill.rs` 的 `parse_frontmatter` 与
   `validate_skill_name`，并从 `tools/*.rs` 实时扫描注册工具名做白名单：
   **6/6 PASS，0 FAIL**（exit 0）。覆盖项：frontmatter 解析、name 合法性、name 与目录名一致、
   description 非空、allowed-tools 全部已注册、正文行数 60-200、含「来源与适配」小节、
   无 Unicode Emoji、正文引用工具名全部合法。
3. **安装就位** —— `ls -R /root/.rustcode/skills`：6 个目录各含 1 个 `SKILL.md`，共 731 行。
4. **中文文档门禁** —— `python3 scripts/check-zh-docs.py gate`：**PASS（exit 0）**；
   本次交接件落在 `.codebuddy/artifacts/` 豁免域（脚本 `ARTIFACTS_EXEMPT_PREFIX`），不进 AC-1 分母。

**测试覆盖到的入口：** 静态校验覆盖了技能加载链路的**输入侧契约**
（frontmatter 解析、命名校验、工具名白名单、文件就位）与中文文档门禁；
**未覆盖运行侧**（`SkillRegistry::load` 的实际加载与 `/` 菜单呈现）——
按本任务约束未运行任何 `cargo` 命令、未启动 runtime。

## 4. 已知未验证范围

| 项 | 状态 | 负责人建议 |
| --- | --- | --- |
| 6 个技能在**运行中的 rustcode** 能否被 `SkillRegistry` 加载并出现在 `/` 菜单 | 未验证（未 `cargo build`、未启动 runtime） | 实现方；`~/.rustcode/skills/` 是 `standard_skill_dirs` 原生路径，理论上直接生效 |
| `user-invocable` 未声明（缺省 true，全部进 `/` 菜单） | 有意留默认 | PM 决定是否需对某些技能设 `false` |
| `allowed-tools` 是否被 L2 审批策略消费 | 未验证 | 实现方确认 L2 策略 |
| `ast-grep`/`sg` 二进制在目标机是否安装 | 未验证 | 首次使用 `rustcode-ast-grep` 前确认；未装则走技能内降级路径 |
| 技能正文中的 `cargo check`/`clippy`/`test` 命令 | 未执行（本任务禁 cargo） | 使用方在各自场景验证 |
| `lsp-setup` 改造所需的 `config.toml` LSP 键名 | 未核实（本批不动工） | 实现方核对 `LspSettings` 后落笔 |
| `frontend`(2.9M) / `programming`(848K) 第三方许可链 | 未做法律澄清 | 若将来采纳必须先过合规复核 |
| 前序遗留的 `README.md` 改动与 `2026-09-09-readme-account-cmds/` 目录 | 未处理、未回滚、未提交 | 编排者 / 该任务负责人 |

## 5. 文档更新清单

| 文件路径 | 变更类型 | 摘要 |
| --- | --- | --- |
| `.codebuddy/artifacts/2026-09-09-omo-skills-import/01-import-plan.md` | 新增 | OMO 17 个技能逐个三选一定级与理由；6 个采纳项的改造点；3 个需改造项的成本与阻塞；8 个放弃项的根因；工具映射表；命名映射；合规声明 |
| `.codebuddy/artifacts/2026-09-09-omo-skills-import/03-impl/T-01.md` | 新增 | 改动清单、6 技能用途与触发条件、三类验证原始输出、冲突与处理、未验证范围 |
| `.codebuddy/artifacts/2026-09-09-omo-skills-import/06-release.md` | 新增 | 本文件：四段式交付清单 + 回滚方案 + 术语一致性结论 |
| `/root/.rustcode/skills/rustcode-ast-grep/SKILL.md` | 新增 | 110 行 |
| `/root/.rustcode/skills/rustcode-debugging/SKILL.md` | 新增 | 122 行 |
| `/root/.rustcode/skills/rustcode-git-master/SKILL.md` | 新增 | 134 行 |
| `/root/.rustcode/skills/rustcode-parallel-review/SKILL.md` | 新增 | 116 行 |
| `/root/.rustcode/skills/rustcode-refactor/SKILL.md` | 新增 | 116 行 |
| `/root/.rustcode/skills/rustcode-remove-ai-slops/SKILL.md` | 新增 | 133 行 |

**已核对但未改动的现有文档：** `AGENTS.md`（核对了依赖方向、编码约束、门禁 G1-G8、术语表，
本次未发现需要同步的引用——技能为全新文件，未被任何现有文档引用）；
`README*`、`docs/**`、`CONTEXT.md` 均未引用本次新增符号，无需同步。
`README.md` 存在他人进行中的改动，按职责边界未触碰。

## 6. 回滚方案

**判定时机（满足任一即回滚）：**
1. 运行中发现某技能被 `SkillRegistry` 拒绝加载（如命名/frontmatter 在真实解析器下报错）；
2. 某技能与内置能力冲突导致误调用（例如 `parallel-review` 被当作 `code_review` 的替代）；
3. `rustcode-git-master` 的约束被观察到不足以阻止破坏性 git 操作；
4. PM 判定技能集合需要收缩。

**执行步骤（全部可逆、无状态残留）：**

```bash
# 6.1 单项回滚（停用某一个技能）
rm -rf /root/.rustcode/skills/<skill-name>

# 6.2 全量回滚（停用本批全部技能）
rm -rf /root/.rustcode/skills/rustcode-ast-grep \
       /root/.rustcode/skills/rustcode-debugging \
       /root/.rustcode/skills/rustcode-git-master \
       /root/.rustcode/skills/rustcode-parallel-review \
       /root/.rustcode/skills/rustcode-refactor \
       /root/.rustcode/skills/rustcode-remove-ai-slops

# 6.3 交接件回滚（可选；不影响运行）
rm -rf /workspace/RustCode/.codebuddy/artifacts/2026-09-09-omo-skills-import

# 6.4 验证回滚结果
ls -R /root/.rustcode/skills            # 应为空或只剩其他来源的技能目录
cd /workspace/RustCode && git status --short   # 应回到本批之前的状态
```

**回滚代价：** 无。本批不涉及代码、配置、依赖、协议、版本号的任何改动，
不产生需要清理的状态文件、缓存或持久化数据，回滚后无需重启之外的任何收尾。
**注意：** 回滚**不得**顺带处理 `README.md` 的既有改动与
`.codebuddy/artifacts/2026-09-09-readme-account-cmds/`——那是他人的过程记录。

## 7. 术语与命名一致性检查结论

- **术语**：全篇使用 `AGENTS.md` 口径——provider（未出现"供应商"）、
  `CodingRuntime`、`kernel`（仅作 crate 名/依赖方向层级引用）、skill、
  子代理（`task` 的 `explore` / `worker` 双车道）、provider 等均保持英文原样；
  已退役的 `rustcode-core` 等未在技能中写成仍可使用的能力。
- **命名**：6 个技能名统一 `rustcode-<verb>` 前缀，全部通过 `validate_skill_name`
  （1-64 字符、仅 `[a-zA-Z0-9_-/]`、不以 `/`/`-` 开头结尾、不含 `//`/`--`），
  且与所在目录名完全一致。
- **工具名**：技能中所有工具名均取自 `tools/*.rs` 里 `fn name(&self)` 的**实际注册名**；
  已纠正三处易错点：`cd` -> `change_dir`、`output_artifact` -> `fetch_output`、
  `repair` 是 JSON 修复模块而非工具（技能中未引用）。未臆造任何工具。
- **风格**：简体中文、无 Unicode Emoji、标签一律 ASCII
  （`[INFO]`/`[CHECK]`/`[ERROR]`/`[+]`/`[-]`），符合 `AGENTS.md` 的 `[STYLE]` 硬约束；
  已用脚本对 6 个 `SKILL.md` 做过 Emoji 扫描，0 命中。
- **路径引用**：统一 `文件:行` 格式，行号均在写作时核实过。
