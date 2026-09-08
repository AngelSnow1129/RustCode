# zh-check 报告

- 基线 (--base): `3ee655e3`
- 阈值 (--en-ratio): `0.05`
- 判定: AC-2 / AC-4 / AC-6 / AC-7b / AC-32，任一失败即 FAIL（fail-closed）

## 汇总

- 受检文件: 1
- PASS: 1
- FAIL: 0

## PASS `docs/REFACTOR_DESIGN_PHASE1.md`

- en=0/297 ratio=0.0000

---

# 手写小结 · D-18（批次 6）

## 1. 任务与自检方式

- 任务 ID：`D-18`（批次 6），单文件专任务。
- `files_owned`：`docs/REFACTOR_DESIGN_PHASE1.md`（基线 851 行 / 44.26 KB）。
- 基线：`3ee655e3`（`ZH_BASE`，未覆盖；与当前 HEAD 同 revision）。
- 自检命令（共执行 **4 次**：改前 1 次取基线，改后 3 次，最后一次带 `--report` 落盘）：

  ```bash
  python3 scripts/check-zh-docs.py check --base 3ee655e3 \
    --report .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-18.md \
    --files docs/REFACTOR_DESIGN_PHASE1.md
  ```

- 文件首行是 `# PHASE-1：...`，**无 YAML frontmatter**，故 AC-6 无适用对象，脚本未报 AC-6 FAIL，也无 `description` 需列示（铁律 §2.7）。

## 2. 改动量（判定：全文汉化，非 no-op）

| 文件 | 基线行数 | 改后行数 | git numstat | 基线 en/total | 改后 en/total | 判定 |
|---|---|---|---|---|---|---|
| `docs/REFACTOR_DESIGN_PHASE1.md` | 851 | 844 | `+300 / -307` | 283/303 = **0.9340** | **0/297 = 0.0000** | **全文汉化** |

判定依据：基线 `en/total = 0.9340`（303 行中 283 行为英文行），远超铁律 §3 的 no-op 门槛，属**纯英文大件**，按铁律 §1 做全文汉化。844 = 851 - 7，减少的 7 行全部来自中文更紧凑导致的散文合并（铁律 §5.7 允许）；**空行数 145 与基线完全相等**，未增删任何空行，段落结构未变（铁律 §5.4）。

覆盖范围（逐节核对，无跳段）：

- §0 工程约束表（C1–C6）、§1 系统分析（1.1–1.3）、§2 OBJECTIVE-1 重命名（2.0–2.7）
- §3 OBJECTIVE-2 零遥测（3.1–3.6，含 24 文件 / 9 crate 的中和清单表与 5 条非显而易见移除项）
- §4 OBJECTIVE-3 LLM 解耦（4.1–4.7，含 G1–G8 缺口表、D5/D6 决策、SSE 对照表）
- §5 OBJECTIVE-4 许可合规（5.1–5.2）、§6 子代理执行计划（6.1–6.3）、§7 验证计划、§8 风险与 GATEWAY（D1–D10）

## 3. 保留项与本批特殊约束的落实情况

| 约束 | 落实方式 |
|---|---|
| 铁律 §2.1 不翻译 fenced code block | 15 个代码块（`text`/`rust`/`toml`/`bash`）body **逐字节相同**，已用脚本比对 `fences(base) == fences(new)` 为 `True`；fence 标记行序列也完全相同 |
| 铁律 §2.2 不翻译 inline code、不新增反引号 | 全文 code span 多重集与基线相等（AC-4 PASS，old=929 new=929）；未把任何普通词包成反引号 |
| 铁律 §2.3 保留原文 | `rustcode` / `rustcode-*` / `RUSTCODE_*` / `rustcode_telemetry` / `LlmProvider` / `ProviderKind` / `ClientMode` / `SessionMode` / `extra_headers` / `model_mapping` / `.unwrap()` 等符号、路径、命令、环境变量、HTTP 头一律原样保留 |
| 铁律 §2.4 不改链接与锚点 | 全文 `](...)` 目标集合基线为 `[]`，改后仍为 `[]`（本文件无 Markdown 链接） |
| 铁律 §2.5 不改文件名与标题层级 | 标题层级分布基线 `{h1:12, h2:9, h3:29}`，改后完全相同 |
| 铁律 §2.6 不新增 Emoji | Emoji 命中数基线 0，改后 0 |
| 铁律 §2.8 已中文段落不润色 | 唯一既存中文段是 §4.2 G6 行的 **[DONE]** 中文说明，**逐字未动**（含其半角逗号与 `\|` 转义） |
| 本批特殊说明 3：历史名引用 | 按 `AGENTS.md:7`，`rustcode` 在本文件中一律按**历史名引用**保留，**未**改写为 `rustcode-*`，也**未**反向改名；`rustcode-core` / `rustcode-bridge` 等同样原样保留 |
| 本批特殊说明 4：编号与章节号 | 决策编号 `D1`–`D10`、缺口编号 `G1`–`G8`、`OBJECTIVE-1..4`、`PHASE-1/2`、`TIP-1..4`、`§2.7` / `§3.4` / `§3.5` / `§4.3` / `§4.4` / `§4.5` / `§6.2` 的出现次数已逐个与基线比对，**全部相等**（脚本输出无 `DIFF` 行） |
| ASCII 状态标签 | `[INFO]` / `[WARN]` / `[CHECK]` / `[SECURITY]` / `[SUCCESS]` / `[DECISION Dn]` / `[GATEWAY]` / `[DELETED]` / `[STREAMING]` 全部保留原样 |

另：`> [SUPERSEDED BY docs/phase1-refactor-design.md]` 这一取代标记**逐字保留**，仅在其后追加中文说明「本文档已被取代」，以免破坏可能的外部检索。

## 4. 残留行清单：0 行（无需白名单）

最终自检 `en=0/297`，即 AC-2  offender 行为 **0**，**不存在任何残留英文行**，因此本批**没有需要按铁律 §5.2 论证的白名单项**。

需说明的是，脚本 `total` 从 303 降到 297，不是因为删了内容，而是两类行不再计入分母：

- 原先独立成行的英文片段被并入带中文的上一行；
- 表格行在剥离 inline code 后仅剩分隔符（无 4 字母以上 ASCII 串）时按 R4 规则不进分母。

这两类都已在上一节的结构校验中被 `fence 一致 / 空行数一致 / 标题层级一致` 覆盖，不存在漏译。

## 5. 四项实测结果

| 验收项 | 判定 | 实测说明 |
|---|---|---|
| **AC-2** `en/total <= 0.05` | **PASS** | `0/297 = 0.0000`（基线 0.9340） |
| **AC-4** inline code + fenced code 多重集相等 | **PASS** | 基线 929 条、改后 929 条，脚本未报差异（old=929 new=929） |
| **AC-7b** `](...)` 链接目标多重集相等 | **PASS** | 两侧均为空集；本文件本就没有 Markdown 链接 |
| **AC-32** Emoji 数不增加 | **PASS** | 基线 0 -> 改后 0 |

附加核对：AC-6 无适用对象（无 YAML frontmatter），脚本未报 AC-6 FAIL。

最终自检输出：

```
PASS docs/REFACTOR_DESIGN_PHASE1.md en=0/297=0.0000

check: 受检 1，PASS 1，FAIL 0
```

## 6. 过程中的两次修正（如实记录）

1. **第一次自检 AC-4 FAIL**（old=929 new=931）：两处误改反引号。
   - §4.1 更正段把基线的一个长 span `is_atomgit_gateway == rustcode_config::endpoints::is_codingplan_llm_gateway` 拆成了两个 span；已改回**单一 span**。
   - §6.2 `[WARN]` 段把基线里**不在反引号内**的 `Cargo.toml vs`（铁律 §5.1 的同类坑）误加了反引号；已去掉。
   第二次自检后 AC-4 转为 PASS。
2. **第二次自检 AC-2 虽已 PASS（0.0337）但有 10 行残留**：§2.5 的 `(ACP cursor)` / `(header)`、§4.1 的 `Ollama adapter` / `Env proxy` / `webpki backstop`、§4.6 的 `(policy-gated)` / `(whole)` / `(live)` / `last non-null` / `mid-stream chunk error`。这些是**可译的英文散文**（不是标识符），按铁律 §1 全部补译，第三次自检降到 `0/297 = 0.0000`。

## 7. 本批明确未做的事

- 未改动任何 fenced code block 内容、未改文件名、未改标题层级、未改链接目标、未新增 Emoji。
- 未执行 `cargo` / `npm` 构建与测试，未改任何源码（铁律 §7，纯文档任务）。
- 未汉化 `[INFO]` 等 ASCII 状态标签本身（它们是约束 C1 规定的字面标签，铁律 §2.3 保留项）。
- 未把 `rustcode` 历史名引用"现代化"成 `rustcode-*`（本批特殊说明 3 明确禁止）。
- 报告全文为实测结论，无任何占位符或空白待补项，所有数字均来自上述 4 次 `check` 与 2 次结构比对脚本的实际输出。
