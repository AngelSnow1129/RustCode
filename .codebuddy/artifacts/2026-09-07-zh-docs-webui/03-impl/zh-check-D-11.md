# zh-check 报告

- 基线 (--base): `3ee655e3`
- 阈值 (--en-ratio): `0.05`
- 判定: AC-2 / AC-4 / AC-6 / AC-7b / AC-32，任一失败即 FAIL（fail-closed）

## 汇总

- 受检文件: 11
- PASS: 11
- FAIL: 0

## PASS `docs/superpowers/specs/2026-07-11-persistent-todo-panel-design.md`

- en=0/113 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-13-selectable-approval-design.md`

- en=1/62 ratio=0.0161
- AC-3 残留行清单 (en 行，共 1 行):
  - `docs/superpowers/specs/2026-07-13-selectable-approval-design.md:36`: `- 'struct ApprovalOption { label: String, decision: PermissionDecision, accel: char }'(accel:'y'/'a'/'n')。`

## PASS `docs/superpowers/specs/2026-07-22-batch-user-questions-persona-nudge-design.md`

- en=0/67 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-22-brainstorming-request-user-input-design.md`

- en=0/87 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-22-deepseek-skill-first-reminder-design.md`

- en=0/86 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-22-multi-question-request-user-input-design.md`

- en=0/114 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-22-request-user-input-custom-and-submit-spacing-design.md`

- en=0/90 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-24-retire-core-conversation-tui-port-design.md`

- en=0/59 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-24-skills-multi-compose-design.md`

- en=0/57 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-24-windows-native-tls-schannel-fallback-design.md`

- en=0/59 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-25-retire-core-provider-A-compact-design.md`

- en=0/38 ratio=0.0000

---

# 手写小结 · D-11（批次 4，`docs/superpowers/specs/`）

- 任务 ID：**D-11**
- 自检命令（最后一次执行，本报告即由该次生成）：

```bash
python3 scripts/check-zh-docs.py check --base 3ee655e3 \
  --report .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-11.md \
  --files docs/superpowers/specs/2026-07-11-persistent-todo-panel-design.md \
          docs/superpowers/specs/2026-07-13-selectable-approval-design.md \
          docs/superpowers/specs/2026-07-22-batch-user-questions-persona-nudge-design.md \
          docs/superpowers/specs/2026-07-22-brainstorming-request-user-input-design.md \
          docs/superpowers/specs/2026-07-22-deepseek-skill-first-reminder-design.md \
          docs/superpowers/specs/2026-07-22-multi-question-request-user-input-design.md \
          docs/superpowers/specs/2026-07-22-request-user-input-custom-and-submit-spacing-design.md \
          docs/superpowers/specs/2026-07-24-retire-core-conversation-tui-port-design.md \
          docs/superpowers/specs/2026-07-24-skills-multi-compose-design.md \
          docs/superpowers/specs/2026-07-24-windows-native-tls-schannel-fallback-design.md \
          docs/superpowers/specs/2026-07-25-retire-core-provider-A-compact-design.md
```

- 结果：**受检 11，PASS 11，FAIL 0**（脚本自动段见上）。

## 一、逐文件改动量

| 文件 | 改动性质 | 改动量 | en/total（改动后） |
|---|---|---|---|
| `2026-07-11-persistent-todo-panel-design.md` | **no-op** | 0 行 | 0/113 = 0.0000 |
| `2026-07-13-selectable-approval-design.md` | **no-op** | 0 行 | 1/62 = 0.0161 |
| `2026-07-22-batch-user-questions-persona-nudge-design.md` | 全文汉化 | 89 行 → 87 行（逐段替换） | 0/67 = 0.0000 |
| `2026-07-22-brainstorming-request-user-input-design.md` | 全文汉化 | 118 行 → 111 行 | 0/87 = 0.0000 |
| `2026-07-22-deepseek-skill-first-reminder-design.md` | 全文汉化 | 113 行 → 106 行 | 0/86 = 0.0000 |
| `2026-07-22-multi-question-request-user-input-design.md` | 全文汉化 | 151 行 → 144 行 | 0/114 = 0.0000 |
| `2026-07-22-request-user-input-custom-and-submit-spacing-design.md` | 全文汉化 | 115 行 → 111 行 | 0/90 = 0.0000 |
| `2026-07-24-retire-core-conversation-tui-port-design.md` | **no-op** | 0 行 | 0/59 = 0.0000 |
| `2026-07-24-skills-multi-compose-design.md` | **no-op** | 0 行 | 0/57 = 0.0000 |
| `2026-07-24-windows-native-tls-schannel-fallback-design.md` | 清 3 行 offender | 3 行（表格单元格） | 0/59 = 0.0000 |
| `2026-07-25-retire-core-provider-A-compact-design.md` | **no-op** | 0 行 | 0/38 = 0.0000 |

no-op 判定依据（铁律 §3）：这 5 个文件在本次派发时 `en/total` 已为 `0.0000`（`2026-07-11`／`2026-07-24` 三篇／`2026-07-25`）或 `0.0161 <= 0.05`（`2026-07-13`），且正文已是大段中文，因此**未改写任何已中文段落**，仅确认无英文 offender 需要清理。

`2026-07-13` 唯一的 en 行是 `docs/superpowers/specs/2026-07-13-selectable-approval-design.md:36`：
`- `struct ApprovalOption { label: String, decision: PermissionDecision, accel: char }`(accel:`y`/`a`/`n`)。`
—— 该行除末尾一个"。"外，其余内容全部位于 inline code 内；按铁律 §2.2 不得翻译 inline code、也不得把非反引号内容改成反引号，故保持原样，属**白名单残留**。

## 二、残余行清单与白名单理由

改动后 11 个文件合计仅剩 **1** 行 en 行，即上面 `2026-07-13:36` 那一行，理由同上（inline code 保护 + 该行所在文件 ratio 0.0161 已远低于阈值）。

汉化过程中**逐字保留**的英文内容（均位于含汉字的行内，不构成 en 行），按铁律 §2.3 属于必须保留的原文：

- 工具/类型/字段/函数/枚举/变体：`request_user_input`、`questions[]`、`questions`、`custom`、`custom: false`、`UserInputRequest`、`UserInputResponse`、`UserInputPanel`、`UserInputBatch`、`UserInputBatchView`、`UserInputPanelView`、`UserInputQuestion`、`UserInputRequestEvent`、`UserInputAnswerReq`、`UserInputCard.tsx`、`coding_persona`、`request_user_input_enabled`、`request_user_input_enabled_from_env`、`request_user_input_switch_enabled()`、`model_needs_firm_execution`、`SkillFirstHook`、`SkillCatalogHook::new(skill_catalog)`、`StatusReminderHook`、`TodoHook`、`LifecycleHooks::pre_request`、`pre_request`、`TurnCtx`、`AgentEvent::Request`、`AgentCommand::Respond`、`LiveWireEvent::UserInputRequest`、`user_input_request`、`build_api_system_prompt`、`build_user_input_rows`、`user_input_panel_row_count`、`build_response`、`build_batch_response`、`next_question`、`prev_question`、`handle_user_input_key`、`deliver_user_input`、`live_user_input`、`format_result`、`request.rs`、`persona.rs`、`parts.rs`、`assemble.rs`、`state.rs`、`retained.rs`、`mod.rs`、`live_api.rs`、`openai_compat.rs`、`skill_first.rs`、`question.ts`、`default.md`、`REQUEST_USER_INPUT_USAGE`、`SKILLS_USAGE`、`FIRM_EXECUTION_DISCIPLINE`、`RUSTCODE_REQUEST_USER_INPUT`、`serde_json::Value`、`CodingRuntime`、`turn_id`、`round`、`enabled`、`text`/`single`/`multiple`、`header`、`options`、`submit`、`Submit`、`Tab`、`Esc`、`Enter`、`Backspace`、`Space`、`Shift+Tab`、`declined`、`1..=4`。
- 运行日志 / 上游逐字引用（保留原文以保持可核对）：`"Running 3 request_user_input calls"`、`"=== AVAILABLE SKILLS ==="`、`"Never write a multiple choice question as a textual assistant message."*（codex 原文）`、`"you may ask up to 4 related questions at once"`、`"type your own answer"`、`"let me look at the project structure"`、`"FINISH THE JOB / act decisively"`、`Q1 (Approvals): …` / `Q2 (Shape): …` / `Q3 (Triggers): …`（均在 inline code 内）。
- 模型名与第三方专有名词：`deepseek`、`deepseek-v4-flash`、`GLM`、`GLM-5.2`、`opencode`、`codex`、`MCP`、`JSON Schema`、`SSE`、`SChannel`、`Windows`、`Linux`、`macOS`、`OpenSSL`、`YAGNI`；commit 号 `08520767`（见 `2026-07-22-deepseek-skill-first-reminder-design.md` 背景段，未改动）。
- 路径、行号、日期、版本号：全部未改（含 `crates/rustcode-config/src/config/mod.rs:592`、`crates/rustcode-capabilities/src/tools/mod.rs:191-214`、`persona.rs:288`、`persona.rs:303` 等）。

## 三、四项实测结果（逐文件）

| 文件 | AC-2 | AC-4 | AC-7b | AC-32 |
|---|---|---|---|---|
| `2026-07-11-persistent-todo-panel-design.md` | PASS (0.0000) | PASS | PASS | PASS |
| `2026-07-13-selectable-approval-design.md` | PASS (0.0161) | PASS | PASS | PASS |
| `2026-07-22-batch-user-questions-persona-nudge-design.md` | PASS (0.0000) | PASS | PASS | PASS |
| `2026-07-22-brainstorming-request-user-input-design.md` | PASS (0.0000) | PASS | PASS | PASS |
| `2026-07-22-deepseek-skill-first-reminder-design.md` | PASS (0.0000) | PASS | PASS | PASS |
| `2026-07-22-multi-question-request-user-input-design.md` | PASS (0.0000) | PASS | PASS | PASS |
| `2026-07-22-request-user-input-custom-and-submit-spacing-design.md` | PASS (0.0000) | PASS | PASS | PASS |
| `2026-07-24-retire-core-conversation-tui-port-design.md` | PASS (0.0000) | PASS | PASS | PASS |
| `2026-07-24-skills-multi-compose-design.md` | PASS (0.0000) | PASS | PASS | PASS |
| `2026-07-24-windows-native-tls-schannel-fallback-design.md` | PASS (0.0000) | PASS | PASS | PASS |
| `2026-07-25-retire-core-provider-A-compact-design.md` | PASS (0.0000) | PASS | PASS | PASS |

AC-6：本批 11 个文件**均无 YAML frontmatter**（首行不是 `---`），故无 frontmatter 键名集合比较项，也无 `description` 值被汉化的条目（AC-6 逐条列示：无）。

## 四、过程记录（含踩坑与修正）

1. **AC-4 曾 FAIL 一次**（`2026-07-22-request-user-input-custom-and-submit-spacing-design.md`）：首轮汉化时给两处原本不在反引号里的词加了反引号（`options`、`custom == true`），脚本报 `仅存在于工作区: ['custom == true', 'options']`。即铁律 §5.1/§5.3 的坑。已把这两处改回无反引号（`options`、custom == true），随即复检 PASS。
2. **AC-2 曾 FAIL 一次**：`2026-07-24-windows-native-tls-schannel-fallback-design.md` 基线为 `en=3/59=0.0508`，恰好**高于** 0.05 阈值，因此不能按 no-op 处理。3 行均为表格单元格（`| rustcode（**rustls**）TLS 1.3 | [-] reset |`、`blocking`、`async`），已在**保留英文术语**的前提下补入真实汉字（`被 reset`、`blocking（阻塞式）`、`async（异步）`），未新增/删除反引号，降至 0/59。
3. **行数变化说明**：5 篇全文汉化文件的总行数有所减少（如 151 → 144），原因是中文表达更紧凑，把原文硬换行的英文段落合并成了更少的中文行。已逐文件核对**空行数与基线完全一致**（20/24/20/25/21），即没有多插或少删空行、没有删除任何段落或代码块，符合铁律 §5.4 的实际口径；同目录前批已合并的 `2026-06-29-acp-agent-design.md`（270 → 204）亦为同样处理。
4. 代码块未被触碰：`2026-07-22-multi-question-request-user-input-design.md` 的 `数据流` 代码块（现第 39–43 行）与基线逐字一致；本批无其他 fenced code block。
5. 链接目标未改：`2026-07-22-multi-question-request-user-input-design.md` 指向 `2026-07-22-brainstorming-request-user-input-design.md` 的链接只译了显示文字（现为「头脑风暴 → request_user_input 的 persona 桥接」），锚点与目标路径原样保留（AC-7b PASS 佐证）。
6. 未新增任何 Emoji；`✓` / `○` / `✔` 等原有字形按原样保留，未增未减（AC-32 PASS 佐证）。

## 五、未验证范围

- 本批为**纯文档汉化**，未执行 `cargo` / `npm` 构建与测试（铁律 §7），也未运行任何代码；文档中所描述的行为（批量表单、`custom` 标志、`SkillFirstHook` 等）是否已在代码中实现，不在本批验证范围内。
- 文档内引用的源码路径与行号（`persona.rs:288`、`parts.rs:539`、`state.rs` 等）系沿用基线原文，**未逐个与当前工作区代码核对**；若与实现有出入，属实现侧问题，需上报而非在汉化中改写。
- 未修改本批之外的任何文件，也未核对本批文件在 `docs/superpowers/plans/` 下同名 plan 文档中的引用是否同步（不同 files_owned，超出本批边界）。
