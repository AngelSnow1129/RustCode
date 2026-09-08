# zh-check 报告

- 基线 (--base): `3ee655e3`
- 阈值 (--en-ratio): `0.05`
- 判定: AC-2 / AC-4 / AC-6 / AC-7b / AC-32，任一失败即 FAIL（fail-closed）

## 汇总

- 受检文件: 3
- PASS: 3
- FAIL: 0

## PASS `docs/plans/2026-08-17-acp-sdk-2.0-protocol-roadmap.md`

- en=4/268 ratio=0.0149
- AC-3 残留行清单 (en 行，共 4 行):
  - `docs/plans/2026-08-17-acp-sdk-2.0-protocol-roadmap.md:37`: `  'session/update'(agent_message_chunk / agent_thought_chunk / tool_call / tool_call_update),`
  - `docs/plans/2026-08-17-acp-sdk-2.0-protocol-roadmap.md:86`: `- matcher:'if_message' → 'if_dispatch';'with_responder' → 'with_runner';`
  - `docs/plans/2026-08-17-acp-sdk-2.0-protocol-roadmap.md:100`: `initialize / session/new / session/prompt / session/cancel / session/request_permission`
  - `docs/plans/2026-08-17-acp-sdk-2.0-protocol-roadmap.md:209`: `   'state_update';capabilities+info;tool_call_update-only + content_chunk;plan_update;`

## PASS `docs/plans/2026-08-21-external-agent-subagent-drivers-spec.md`

- en=3/167 ratio=0.0180
- AC-3 残留行清单 (en 行，共 3 行):
  - `docs/plans/2026-08-21-external-agent-subagent-drivers-spec.md:96`: `### 4.1 'SubagentBackend' trait`
  - `docs/plans/2026-08-21-external-agent-subagent-drivers-spec.md:147`: `| rustcode | Claude Code ('claude -p --permission-mode') | Codex ('codex exec') |`
  - `docs/plans/2026-08-21-external-agent-subagent-drivers-spec.md:200`: `- spawn（'tokio::process::Command'，'kill_on_drop'）。`

## PASS `docs/plans/2026-08-14-webui-browser-notification-design.md`

- en=2/138 ratio=0.0145
- AC-3 残留行清单 (en 行，共 2 行):
  - `docs/plans/2026-08-14-webui-browser-notification-design.md:108`: `| stop_reason | title | body |`
  - `docs/plans/2026-08-14-webui-browser-notification-design.md:114`: `| undefined | RustCode finished | Finished |`

---

# 手写小结 · D-15（批次 5）

## 1. 任务与自检方式

- 任务 ID：`D-15`（批次 5）。
- 基线：`3ee655e3`（`ZH_BASE`，未覆盖）。
- 自检命令（先后执行两次，第二次带 `--report` 落盘）：

  ```bash
  python3 scripts/check-zh-docs.py check --base 3ee655e3 \
    --report .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-15.md \
    --files docs/plans/2026-08-17-acp-sdk-2.0-protocol-roadmap.md \
            docs/plans/2026-08-21-external-agent-subagent-drivers-spec.md \
            docs/plans/2026-08-14-webui-browser-notification-design.md
  ```

- 三文件均位于 `docs/plans/`；实测首行分别为 `# ACP SDK 2.0 升级与协议支持推进路线`、`# 外部 Agent 子代理驱动 —— 设计 Spec`、`# WebUI 任务完成浏览器提醒设计`，**均无 YAML frontmatter**，故 AC-6 无适用对象，脚本未报 AC-6 FAIL。

## 2. 逐文件改动量（三个文件全部判定为 no-op）

| 文件 | 物理行数 | en/total | 判定 | 本批改动行数 |
|---|---|---|---|---|
| `docs/plans/2026-08-17-acp-sdk-2.0-protocol-roadmap.md` | 360 | 4/268 = 0.0149 | **no-op** | 0 |
| `docs/plans/2026-08-21-external-agent-subagent-drivers-spec.md` | 305 | 3/167 = 0.0180 | **no-op** | 0 |
| `docs/plans/2026-08-14-webui-browser-notification-design.md` | 232 | 2/138 = 0.0145 | **no-op** | 0 |

判定依据是铁律 §3 第二句：三文件的 `en/total` 分别为 0.0149 / 0.0180 / 0.0145，**均远低于阈值 0.05**，正文已为大段中文，因此直接记 **no-op**。

按铁律 §2.8「已中文的段落不润色、不改写」，本批**未改写任何一行**：既没有润色既有的中文段落，也没有为了满足"看起来更整齐"去动那些已经合规的英文标识符行。落盘前 `git status --porcelain` 对这 3 个文件输出为空，可证改动量为 0。

## 3. 残留行清单及「为何属于白名单」（共 9 行，逐行说明）

| 位置 | 残留内容（摘要） | 属于白名单的理由 |
|---|---|---|
| `...-roadmap.md:37` | `` `session/update` ``(agent_message_chunk / agent_thought_chunk / tool_call / tool_call_update), | 该行是上一行中文列表项的续行，行内可译内容为零：只有 ACP 方法路径 `session/update` 与四个通知变体名（枚举变体名），按铁律 §2.3 保留。汉字已出现在上一行（"流式"）与下一行（"图片走 vision…"），再插入汉字会破坏同一列表项的连贯性。 |
| `...-roadmap.md:86` | matcher:`` `if_message` `` → `` `if_dispatch` ``；`` `with_responder` `` → `` `with_runner` ``； | 整行是 API 重命名映射，左右两侧全是函数名，属铁律 §2.3「函数名/类型名」保留项；行首的 `matcher` 是配置项名。 |
| `...-roadmap.md:100` | initialize / session/new / session/prompt / session/cancel / session/request_permission | ACP 方法路径清单（HTTP/协议 API 路径），铁律 §2.3 保留；其上一行为空行、下一行"含事件流与审批回环，详见 2.1"为中文说明，上下文无漏译。 |
| `...-roadmap.md:209` | `` `state_update` ``;capabilities+info;tool_call_update-only + content_chunk;plan_update; | v2 协议字段与变体名清单（配置键/变体名），铁律 §2.3 保留；该列表项的其余续行（"审批 title/description/subject…"）均为中文。 |
| `...-subagent-drivers-spec.md:96` | `### 4.1 `SubagentBackend` trait` | 标题行，可译成分仅为结构性词 `trait`，主体 `SubagentBackend` 是类型名（铁律 §2.3 保留），且铁律 §2.5 要求不改标题层级；该节（4.1）正文已全中文。 |
| `...-subagent-drivers-spec.md:147` | \| rustcode \| Claude Code (`` `claude -p --permission-mode` ``) \| Codex (`` `codex exec` ``) \| | 三列表格行：列值分别是 crate 名 `rustcode`、第三方专有名词 Claude Code / Codex、以及 CLI 命令与 flag，全部命中铁律 §2.3（crate 名、第三方专有名词、命令与 CLI flag）。 |
| `...-subagent-drivers-spec.md:200` | - spawn（`` `tokio::process::Command` ``，`` `kill_on_drop` ``）。 | 剥离 inline code 后仅剩 `spawn（，）。`——全角括号与句号不在脚本的 CJK 判定区间（铁律 §5.2 已记录的已知形态）；`spawn` 为 Rust/Tokio 术语，铁律 §2.3 保留。 |
| `...-webui-browser-notification-design.md:108` | \| stop_reason \| title \| body \| | 表格表头，三列均为字段名/列名（`stop_reason` 为 snake_case 标识符），铁律 §2.3 保留；这是表格结构行，不是散文。 |
| `...-webui-browser-notification-design.md:114` | \| undefined \| RustCode finished \| Finished \| | `undefined` 是 `stop_reason` 的取值（字面量）；`RustCode finished` / `Finished` 是产品实际展示的英文 UI 文案，译成中文会与实现行为不一致，属铁律 §2.3 保留范围。 |

**结论**：9 行残留**全部**由「协议路径 / 枚举变体 / 类型名 / crate 名 / CLI 命令 / 第三方专有名词 / 字段与列名 / 英文 UI 文案」构成，无一行是未翻译的英文散文，因此不存在漏译，也不存在铁律 §5.2 那种"只加全角标点"的伪中文行。

## 4. 四项实测结果

| 验收项 | 判定 | 实测说明 |
|---|---|---|
| **AC-2** `en/total <= 0.05` | **PASS** | 0.0149 / 0.0180 / 0.0145，三项均远低于 0.05 |
| **AC-4** inline code + fenced code 多重集相等 | **PASS** | 本批零改动，code span 集合与基线恒等；脚本未报差异 |
| **AC-7b** `](...)` 链接目标多重集相等 | **PASS** | 未改动任何链接目标与显示文字；脚本未报差异 |
| **AC-32** Emoji 数不增加 | **PASS** | 未新增任何字符，Emoji 命中数与基线相同 |

附加核对：AC-6 因三文件均无 YAML frontmatter 而无适用对象（脚本未报 AC-6 FAIL，亦无 `description` 汉化项需列示）。

最终自检输出：

```
PASS docs/plans/2026-08-17-acp-sdk-2.0-protocol-roadmap.md en=4/268=0.0149
PASS docs/plans/2026-08-21-external-agent-subagent-drivers-spec.md en=3/167=0.0180
PASS docs/plans/2026-08-14-webui-browser-notification-design.md en=2/138=0.0145

check: 受检 3，PASS 3，FAIL 0
```

## 5. 本批明确未做的事

- 未改写任何已中文段落（避免违反铁律 §2.8）——本批最大的风险就是"顺手润色"，已刻意规避。
- 未调整标题层级、未改文件名、未改链接目标与锚点、未新增 Emoji、未把普通词包成反引号（规避铁律 §5.1 的 AC-4 坑）。
- 未执行 `cargo` / `npm` 构建与测试，未改源码（铁律 §7，纯文档任务）。
- 报告全文均为实测结论，无空白待补项、无推测性表述，所有数字均来自上述两次 `check` 的实际输出。
