# zh-check 报告

- 基线 (--base): `3ee655e3`
- 阈值 (--en-ratio): `0.05`
- 判定: AC-2 / AC-4 / AC-6 / AC-7b / AC-32，任一失败即 FAIL（fail-closed）

## 汇总

- 受检文件: 1
- PASS: 1
- FAIL: 0

## PASS `docs/phase1-refactor-design.md`

- en=0/296 ratio=0.0000

---

# D-19 手写小结（在最后一次自检之后追加）

## 1. 任务与文件

- 任务：D-19（批次 7），单文件专任务。
- `files_owned`：`docs/phase1-refactor-design.md`（45580 字节 / 688 行；无 YAML frontmatter）。
- 并行批次 D-18 处理的 `docs/REFACTOR_DESIGN_PHASE1.md` 与本文内容相关但非同一文件，本次**未**触碰。

## 2. 改动量判定：**no-op（0 行改动）**

按汉化铁律 §3「若文件已是大段中文，只处理 `check` 报出的英文 offender 行；`en/total <= 0.05` 则记 no-op，不得改写已中文段落」：

- 首次自检即报 `en=0/296`，`ratio=0.0000` —— **check 报出的英文 offender 行为 0 条**。
- 因为没有任何 offender 行，本批**不存在需要改的行**，故对正文**一字未改**。
- 实测佐证：`git diff 3ee655e3 -- docs/phase1-refactor-design.md` 为空，`git status --porcelain` 为空，工作区文件与基线逐字节相同（均为 688 行 / 45580 字节）。

判定为 **no-op**，符合铁律 §3，且正是本批特殊说明第 3 条所要规避的反面（"顺手润色"）。

## 3. 为排除"假绿"所做的三重核验

自检给出 `en=0` 后，我没有直接采信，而是独立复核了三种可能导致脚本漏判的情形：

1. **围栏未闭合导致大段内容被整体跳过**：统计以三个反引号开头的行共 38 行 = 19 个围栏块，开闭配对完整，无落单围栏。故不存在"半个文件被当作代码块跳过"的假绿。
2. **R5 跳过是否吞掉了散文**：脚本报告 R5（缩进代码块 / 含围栏的引用块内裸日志行）跳过 **0 行**，附录为空。
3. **inline code 掩码是否掩盖了英文散文**：自写脚本在剔除围栏与 inline code 后，对全文做了两轮独立扫描：
   - 「含 >= 2 个英文单词」的行共 **99 行** —— 逐行核对，全部是**中文为主**的句子中夹带受铁律 §3 保护的原样符号：crate 名（`rustcode-*`）、文件路径、命令行 flag（`--no-telemetry`）、环境变量（`RUSTCODE_*`）、TOML 键、类型/函数名、HTTP 方法等。**这些不得翻译**。
   - 「剔除 inline code 后完全不含 CJK 且非空」的行共 **38 行** —— 实为 19 行表格分隔行（形如 `|---|---|`）、11 行分隔线（`---`）、以及 8 行纯路径 / inline code 行（250、251、442、461、481、524、525、528）。**均非英文散文**。其中 442 / 461 / 481 是中文长句硬折行的续行（尾部只余 `、）、。` 等标点，按铁律 §5.6 不得重新折行，维持原样。

结论：`en=0` 是**真实结果**，不是脚本漏判。

## 4. 残留行清单与白名单理由

**check 报出的 offender 行：0 条**（无需列出）。

我额外自查出上述 99 + 38 行含英文字符的行，**全部判定为白名单、一行未改**，理由统一如下：

- 它们是铁律 §3 明令**保留原文**的符号（路径、目录名、crate 名 `rustcode-*`、命令与 CLI flag、环境变量 `RUSTCODE_*`、配置键/TOML 表名、HTTP 方法与 API 路径、枚举变体、函数名/类型名、第三方专有名词、版本号/日期/数字）；
- 或本身不含任何自然语言（表格分隔行、分隔线、纯 inline code 路径行）；
- 或属于铁律 §5.6 所述**跨行反引号对 / 硬折行**结构，重排会直接触发 AC-4 FAIL，代价远大于收益。

其中决策编号（D1）、`§` 章节号、`OBJECTIVE-*` 编号、crate 名均按要求**原样保留**——其它文档按号引用本文件，改动会破坏交叉引用。

## 5. 四项实测结果

| 项 | 实测值 | 判定 |
|---|---|---|
| AC-2 | `en=0 / total=296`，`ratio=0.0000` <= `0.05` | **PASS** |
| AC-4 | code span 多重集大小 **691**（inline code + 围栏正文行）；工作区与基线为同一文件，集合恒等 | **PASS** |
| AC-7b | `](...)` 链接目标多重集大小 **0**（本文件无 markdown 链接，所有引用均为 inline code 形式的路径） | **PASS** |
| AC-32 | Emoji 命中数 **0**，与基线相等（未新增、也未清除） | **PASS** |

补充：AC-6 不适用 —— 本文件首行为 `# PHASE-1 ...` 而非 `---`，`frontmatter_block` 返回 **0 行**，无 YAML frontmatter，故无 frontmatter 键名与 `description` 的变更。

脚本总判定：`check: 受检 1，PASS 1，FAIL 0`，退出码 0。

## 6. 复核方式

```bash
python3 scripts/check-zh-docs.py check --base 3ee655e3 \
  --report .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-19.md \
  --files docs/phase1-refactor-design.md
```
