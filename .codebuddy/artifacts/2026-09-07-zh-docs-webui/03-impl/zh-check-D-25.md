# zh-check 报告

- 基线 (--base): `3ee655e3`
- 阈值 (--en-ratio): `0.05`
- 判定: AC-2 / AC-4 / AC-6 / AC-7b / AC-32，任一失败即 FAIL（fail-closed）

## 汇总

- 受检文件: 1
- PASS: 1
- FAIL: 0

## PASS `docs/archive/coding-runtime-native-migration-design.md`

- en=0/578 ratio=0.0000

---

# D-25 手写小结（最后自检后追加）

## 1. 任务与范围

- 任务 ID：`D-25`（批次 9，单文件专任务）。
- files_owned（唯一改动文件）：`docs/archive/coding-runtime-native-migration-design.md`（45.62 KB，基线 `3ee655e3`）。
- 未改动 `docs/archive/` 下其它文件（属 D-24 / D-26 并行范围），未改动源码、未执行 `cargo`/`npm` 构建与测试。
- 文件**无 YAML frontmatter**（首行不是 `---`），因此铁律 §7 的 frontmatter 键名/`description` 逐条列示**不适用**：`AC-6` 前后键名集合均为空且相等。

## 2. 改动量（非 no-op）

改动前自检：`en=109/578 ratio=0.1886`（`AC-2 FAIL`），**大于** §3 的 `0.05` 阈值，**不是 no-op**，需要处理。

本次改动**严格限定为 `check` 报出的 109 行英文 offender**，逐行 1:1 替换，共 109 行，**无一行溢出到清单外**；已中文段落一律未润色、未改写（铁律 §3、§8）。

分类统计（合计 109 行）：

| 类别 | 行数 | 说明 |
|---|---|---|
| 章节标题（`###`） | 17 | 如 `### 8.1 Shutdown` → `### 8.1 关闭（shutdown）`；仅译标题文字，层级 `###` 保持不变 |
| 表格行 | 33 | §11.2 parity gate 必测列、§13 命令归属、§14 core 命令删除映射、§15.1 模块职责、§17.2 driver parity |
| 列表项与正文续行 | 59 | 含 3 行关联文档链接（第 13–15 行）与 `661`、`755`、`956` 等硬折行续行 |

行数与结构核验：`1116` 行 → `1116` 行（未变），空行数 `246` → `246`（未变，未增删空行），标题层数、表格竖线数、行首缩进三项均逐行比对一致。

## 3. 残留行清单与白名单说明

**残留 0 行**：最终自检 `en=0/578 ratio=0.0000`，无需任何白名单条目。

说明本批为何能做到零残留（对照前批踩坑 6）：全文奇数反引号行共 50 行，**全部是 ``` 围栏起止行**，不存在「一对反引号被硬折行拆到两行」的跨行 code span，因此无需为保全 `AC-4` 而保留英文散文。第 661、755、956 行虽是硬折行续行，但原行本就不含反引号，补入真实汉字即可，不影响 code span 多重集。

铁律 §5.2 的全角标点坑已规避：所有替换行都补入**真实汉字**（如「模式」「终结」「快照代理」），未出现只加全角括号的替换。

## 4. 四项实测结果

| 项 | 结果 | 实测值 |
|---|---|---|
| AC-2 `en/total <= 0.05` | **PASS** | `0.1886`（109/578）→ `0.0000`（0/578） |
| AC-4 inline code + fenced code 多重集相等 | **PASS** | 脚本无 FAIL 输出；另经脚本级复核：109 行改动**无一位于 fenced block 内**，且逐行反引号数量不变、全文 `code_spans()` 前后相等 |
| AC-7b `](...)` 链接目标多重集相等 | **PASS** | 第 13–15 行只**追加**中文括注（如`（压缩原生迁移复盘）`），链接目标与锚点原样保留；全文 `link_targets()` 前后相等 |
| AC-32 Emoji 数不增加 | **PASS** | 前后均为 0，未新增也未清理既有符号 |

附加一致性核验（均通过）：

- 已退役组件 `rustcode-core` 的历史引用按 `AGENTS.md:7` **保留原文**，未改写为 `rustcode-*`：全文命中数前后均为 `3`。
- 铁律 §2.3 保留项未动：crate 名（`rustcode-bridge`/`rustcode-coding`/`rustcode-capabilities`）、core 变体名（`SendMessage`/`SetMode`/`UndoToPrompt`/`SetGoal/ClearGoal`…）、函数名（`submit`/`change_directory`/`undo_to_prompt`/`respond(id,value)`）、路径（`runtime/state.rs`）、第三方专名（OpenAI、Claude、Ollama、AtomGit、TLS、MCP）。
- 铁律 §2.2 遵守：未把原本不在反引号内的词改成反引号，译文中的英文术语一律保持裸写或置于全角括号内。

## 5. 执行的命令

```bash
python3 scripts/check-zh-docs.py check --base 3ee655e3 \
  --report .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-25.md \
  --files docs/archive/coding-runtime-native-migration-design.md
```

- 改动前跑 1 次（取 offender 清单，FAIL），改动后立即复检 1 次（PASS），最终定稿后再跑 1 次并落盘本报告（PASS）。
- 报告为脚本覆盖式写入，本小结在**最后一次**自检之后追加（铁律 §5.5）。

## 6. 结论

`D-25` 判定 **PASS**，四项门禁（AC-2 / AC-4 / AC-7b / AC-32）全部通过，`AC-6` 因无 frontmatter 不适用且前后一致。无残留 TODO、占位符或与实现冲突的表述。
