# zh-check 报告

- 基线 (--base): `3ee655e3`
- 阈值 (--en-ratio): `0.05`
- 判定: AC-2 / AC-4 / AC-6 / AC-7b / AC-32，任一失败即 FAIL（fail-closed）

## 汇总

- 受检文件: 4
- PASS: 4
- FAIL: 0

## PASS `docs/compact-native-migration-retrospective.md`

- en=0/282 ratio=0.0000

## PASS `docs/multi-agent-collaboration-solution.md`

- en=0/179 ratio=0.0000

## PASS `docs/platform-neutralization.md`

- en=0/192 ratio=0.0000
- R5 跳过的行 (共 14 行):
  - `docs/platform-neutralization.md:38`: `    ('crates/rustcode-cli/src/main.rs:5081')`
  - `docs/platform-neutralization.md:49`: `    ('crates/rustcode-config/src/config/provider_preset.rs:303')`
  - `docs/platform-neutralization.md:75`: `    'uninstall.sh' / 'uninstall.ps1' 头部 'curl|sh' / 'irm' 厂商 URL 改本地 / 发行`
  - `docs/platform-neutralization.md:157`: `    status-glyph*.md' 等 3 个文件)里的 '🟢/🟡/🔴' 是该彩色状态点特性的规格主语义`
  - `docs/platform-neutralization.md:168`: `    ('crates/rustcode-capabilities/src/cc_hooks.rs:1144'):测试自身缺陷——同一 '&&'`
  - `docs/platform-neutralization.md:169`: `    链中两个 'grep' 共享 stdin,首个 grep 抽干管道致第二个必失败;改为先`
  - `docs/platform-neutralization.md:172`: `    ('crates/rustcode-capabilities/src/tools/read.rs:1276'):断言停留在旧 300 行分页,`
  - `docs/platform-neutralization.md:173`: `    实现已为 1500 行页('DEFAULT_READ_LIMIT','crates/rustcode-capabilities/src/`
  - `docs/platform-neutralization.md:174`: `    tools/read.rs:25');夹具改为 1600 行并对齐`
  - `docs/platform-neutralization.md:177`: `    'SpawnFailed("Text file busy (os error 26)")'(overlayfs / 容器内新写脚本`
  - `docs/platform-neutralization.md:178`: `    execve 的 ETXTBSY 竞态;Go fork/exec 内置重试而 Rust std 没有):新增`
  - `docs/platform-neutralization.md:179`: `    'process_utils::spawn_retrying_etxtbsy'(8 次退避重试,`
  - `docs/platform-neutralization.md:180`: `    'crates/rustcode-capabilities/src/process_utils.rs:204'),接入`
  - `docs/platform-neutralization.md:181`: `    'subagent::proc::ManagedChild::spawn'('.../subagent/proc.rs:146')与`

## PASS `docs/HOOK_DOC_UPDATE_SPEC.md`

- en=0/212 ratio=0.0000

## 附录 · 被 R5 跳过的行（供抽检）

共 14 行。

- `docs/platform-neutralization.md:38`: `    ('crates/rustcode-cli/src/main.rs:5081')`
- `docs/platform-neutralization.md:49`: `    ('crates/rustcode-config/src/config/provider_preset.rs:303')`
- `docs/platform-neutralization.md:75`: `    'uninstall.sh' / 'uninstall.ps1' 头部 'curl|sh' / 'irm' 厂商 URL 改本地 / 发行`
- `docs/platform-neutralization.md:157`: `    status-glyph*.md' 等 3 个文件)里的 '🟢/🟡/🔴' 是该彩色状态点特性的规格主语义`
- `docs/platform-neutralization.md:168`: `    ('crates/rustcode-capabilities/src/cc_hooks.rs:1144'):测试自身缺陷——同一 '&&'`
- `docs/platform-neutralization.md:169`: `    链中两个 'grep' 共享 stdin,首个 grep 抽干管道致第二个必失败;改为先`
- `docs/platform-neutralization.md:172`: `    ('crates/rustcode-capabilities/src/tools/read.rs:1276'):断言停留在旧 300 行分页,`
- `docs/platform-neutralization.md:173`: `    实现已为 1500 行页('DEFAULT_READ_LIMIT','crates/rustcode-capabilities/src/`
- `docs/platform-neutralization.md:174`: `    tools/read.rs:25');夹具改为 1600 行并对齐`
- `docs/platform-neutralization.md:177`: `    'SpawnFailed("Text file busy (os error 26)")'(overlayfs / 容器内新写脚本`
- `docs/platform-neutralization.md:178`: `    execve 的 ETXTBSY 竞态;Go fork/exec 内置重试而 Rust std 没有):新增`
- `docs/platform-neutralization.md:179`: `    'process_utils::spawn_retrying_etxtbsy'(8 次退避重试,`
- `docs/platform-neutralization.md:180`: `    'crates/rustcode-capabilities/src/process_utils.rs:204'),接入`
- `docs/platform-neutralization.md:181`: `    'subagent::proc::ManagedChild::spawn'('.../subagent/proc.rs:146')与`

---

# D-20 手写小结（批次 7 · docs/ 根级 4 文件）

本小结写于**最后一次自检之后**（铁律 §5.5：`--report` 会覆盖报告，手写段必须最后追加）。

自检命令（可复现）：

```bash
python3 scripts/check-zh-docs.py check --base 3ee655e3 \
  --report .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-20.md \
  --files docs/compact-native-migration-retrospective.md \
          docs/multi-agent-collaboration-solution.md \
          docs/platform-neutralization.md \
          docs/HOOK_DOC_UPDATE_SPEC.md
```

## 1. 开工前诊断（先自检拿 offender 清单）

| 文件 | 自检前 en/total | 自检前判定 | 归类 |
|---|---|---|---|
| `docs/compact-native-migration-retrospective.md` | 13/282 = 0.0461 | PASS | 已中文，只需清 offender |
| `docs/multi-agent-collaboration-solution.md` | 0/179 = 0.0000 | PASS | 已全中文 → **no-op** |
| `docs/platform-neutralization.md` | 1/192 = 0.0052 | PASS | 已中文，只需清 offender |
| `docs/HOOK_DOC_UPDATE_SPEC.md` | 11/212 = 0.0519 | **FAIL（AC-2）** | 已中文，只需清 offender |

四个文件**均无 YAML frontmatter**（首行不是 `---`），因此 AC-6 键名集合恒等（空集）已自动满足，
`description` 无变更可列（铁律 §2.7 的逐条列示清单为空）。

四个文件在开工前**全部已是中文正文**，不存在「纯英文需全文译」的文件；本批全部工作量是
「清 offender」，完全落在铁律 §3 的适用范围内。

## 2. 各文件改动量

### 2.1 `docs/compact-native-migration-retrospective.md`

自检前 13 个 offender，**全部清除**，改动 13 行（纯行内替换，行数不变）：

| 行 | 改动前 | 改动后 | 说明 |
|---|---|---|---|
| 114 | `- core \`AgentCommand::Compact\`；` | `- core 的 \`AgentCommand::Compact\`；` | 补「的」字成中文散文 |
| 115 | `- core \`CompactionUi\` / \`CompactionUiKind\`；` | `- core 的 \`CompactionUi\` / \`CompactionUiKind\`；` | 同上 |
| 175 | `- stable handle generation；` | `- 稳定的 handle generation；` | 形容词 stable 汉化，术语 handle generation 保留 |
| 176 | `- suspend/resume；` | `- suspend/resume 机制；` | 补「机制」；suspend/resume 按术语保留 |
| 239 | `compacting/Streaming。` | `compacting/Streaming 状态。` | 补「状态」；保留原折行位置未动 |
| 383 | `\| TUI sender \| …` | `\| TUI 发送方 \| …` | sender → 发送方（与本文 §2.2「命令解析/发送方」口径一致） |
| 384 | `\| CLI/headless/clix sender \| …` | `\| CLI/headless/clix 发送方 \| …` | 同上 |
| 385 | `\| daemon/webui sender \| …` | `\| daemon/webui 发送方 \| …` | 同上 |
| 386 | `\| core command/event \| …` | `\| core 命令/事件 \| …` | command/event 汉化 |
| 387 | `\| bridge handler/converter \| …` | `\| bridge 处理器/转换器 \| …` | handler/converter 汉化 |
| 388 | `\| runtime lifecycle \| …` | `\| runtime 生命周期 \| …` | lifecycle 汉化 |
| 389 | `\| driver consumer/state \| …` | `\| driver 消费方/状态 \| …` | consumer/state 汉化 |
| 390 | `\| v1/fallback/tests \| …` | `\| v1/fallback/测试 \| …` | tests 汉化；`v1`/`fallback` 按术语保留 |

第 383–390 行是第 7.1 节「建议保存以下盘点表」的**表体空行**。`sender`/`handler`/`converter`
**未被改成反引号**（铁律 §2.2：不得把原本不在反引号里的词改成反引号），只做了自由散文汉化，
故 AC-4 的 code span 多重集不受影响。

### 2.2 `docs/multi-agent-collaboration-solution.md`

**no-op，零改动**。自检前后均为 `en=0/179 ratio=0.0000`；基线比对无 diff。
按铁律 §3 与 §2.8，已中文段落不润色、不改写，故整份文件原样保留。

### 2.3 `docs/platform-neutralization.md`

自检前 1 个 offender，**已清除**，改动 1 行：

| 行 | 改动前 | 改动后 | 说明 |
|---|---|---|---|
| 50 | `  - TUI fallback:\`crates/…/provider_panel.rs:71\`` | `  - TUI 回退:\`crates/…/provider_panel.rs:71\`` | 与同级「注册表:」「锁定测试:」标签风格对齐 |

该文件的 **R5 跳过行（14 行）未触碰**：第 38、49、75、157、168–169、172–174、177–181 行是
4 空格缩进的续行（含文件路径与日志片段），按 R5 规则本来就 skip，本批未对其加中文标点、
未改变其缩进，避免把 skip 行翻成计费行。

### 2.4 `docs/HOOK_DOC_UPDATE_SPEC.md`

自检前 11 个 offender（**唯一 FAIL 文件**），**全部清除**，改动 11 行：

| 行 | 改动前 | 改动后 | 说明 |
|---|---|---|---|
| 39 | `… \| [-] fire-and-forget \|` | `… \| [-] fire-and-forget（不阻断流程） \|` | 与本文第 180 行「fire-and-forget：结果仅用于日志」同义补注 |
| 40 | `… \| [-] fire-and-forget \|` | `… \| [-] fire-and-forget（不阻断流程） \|` | 同上 |
| 57 | `… \`PreToolExecution\` + \`PostToolExecution\` + \`PostTurn\` + \`SystemPrompt\` \|` | `… \`PreToolExecution\` 与 \`PostToolExecution\` 与 \`PostTurn\` 与 \`SystemPrompt\` \|` | 连词 `+` 汉化为「与」（`+` 在反引号外，不影响 AC-4） |
| 59 | `\| **ToolAuditLogHook** (built-in) \| 1 \| …` | `\| **ToolAuditLogHook** (内置) \| 1 \| …` | built-in → 内置 |
| 60 | `\| **TurnStatsHook** (built-in) \| 2 \| …` | `\| **TurnStatsHook** (内置) \| 2 \| …` | 同上 |
| 61 | `\| **AutoCommitHook** (built-in) \| 1 \| …` | `\| **AutoCommitHook** (内置) \| 1 \| …` | 同上 |
| 62 | `\| **SessionSummaryHook** (built-in) \| 2 \| …` | `\| **SessionSummaryHook** (内置) \| 2 \| …` | 同上 |
| 63 | `\| **ErrorReportHook** (built-in) \| 1 \| …` | `\| **ErrorReportHook** (内置) \| 1 \| …` | 同上 |
| 64 | `\| **ResponseValidationHook** (built-in) \| 1 \| …` | `\| **ResponseValidationHook** (内置) \| 1 \| …` | 同上 |
| 159 | `4. Webhook（\`load_webhook_hooks\`）` | `4. Webhook 钩子（\`load_webhook_hooks\`）` | 补「钩子」；与第 158 行「3. 内置 Hook（…）」同层 |
| 253 | `  - JSON ShellCommandHook：pre_tool_use / …` | `  - JSON ShellCommandHook：支持 pre_tool_use / …` | 补「支持」，与第 252 行「仅 …」对仗 |

本文件含较多 bash/JSON/TOML 示例（第 70–82、92–123、166–171、174–178 行等 fenced 块）。
本批改动**没有一行落在 fenced 代码块内**，示例代码块一个字符未动（本批特殊说明第 2 条），
`git diff` 可逐条核对。

## 3. 残留行清单与白名单判定

**残留英文行：0 行。** 四个文件的 AC-3 残留行清单在自检报告中均为空，无需要白名单豁免的行。

补充两点「刻意保留、不算残留」的说明（均已含 CJK，本来就不进 offender 清单）：

- 跨行反引号对：本批 4 个文件中**未发现**跨行反引号对（铁律 §5.6 的最隐蔽坑），
  因此不存在「为保 AC-4 而不得不留英文词」的情况。
- 铁律 §2.3 要求保留的原文（文件路径、`rustcode-*` crate 名、环境变量 `RUSTCODE_*`、
  配置键、函数名/类型名/hook 名、`Msg::Xxx`、OpenAI/Anthropic/Ollama/Rust/Tokio/VS Code、
  版本号/日期/数字）**一个未改**，例如 `AgentCommand::Compact`、`load_webhook_hooks`、
  `crates/rustcode-config/src/config/provider_preset.rs:303` 等均原样保留。

## 4. 四项实测结果

| 项 | 阈值/口径 | 实测 | 结论 |
|---|---|---|---|
| **AC-2** | `en/total <= 0.05` | 4 个文件全部 `en=0`：0/282、0/179、0/192、0/212 | **PASS**（改动前最差的 `HOOK_DOC_UPDATE_SPEC.md` 0.0519 FAIL → 现 0.0000） |
| **AC-4** | inline code + fenced code 多重集与基线完全相等 | 无 FAIL 明细，`git diff` 显示 25 行改动全在反引号之外；未新增/删除任何反引号 | **PASS** |
| **AC-7b** | `](...)` 链接目标多重集相等 | 无 FAIL 明细；本批未改动任何链接（不改链接目标与锚点，铁律 §2.4） | **PASS** |
| **AC-32** | Emoji 数不增加 | 无 FAIL 明细；本批未新增 Unicode Emoji，也未顺手清理既有 Emoji（铁律 §2.6） | **PASS** |

附带核对：

- **AC-6**：4 个文件均无 YAML frontmatter，键名集合在基线与工作区皆为空集，相等；
  `description` 无变更（无需逐条列示）。
- **行数**：`git diff --stat` 为 **25 insertions / 25 deletions**（`multi-agent-collaboration-solution.md`
  零 diff），没有增删空行、没有改变段落结构（铁律 §5.4）。
- **范围**：只改 files_owned 的 4 个文件；`docs/` 根级其它文件（D-18/D-19/D-21/D-22/D-23 负责）
  未改动；未执行 `cargo`/`npm`，未改源码（铁律 §7）。
