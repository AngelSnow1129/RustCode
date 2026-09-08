# zh-check 报告

- 基线 (--base): `3ee655e3`
- 阈值 (--en-ratio): `0.05`
- 判定: AC-2 / AC-4 / AC-6 / AC-7b / AC-32，任一失败即 FAIL（fail-closed）

## 汇总

- 受检文件: 3
- PASS: 3
- FAIL: 0

## PASS `docs/superpowers/plans/2026-07-11-persistent-todo-panel.md`

- en=0/189 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-13-selectable-approval.md`

- en=0/158 ratio=0.0000

## PASS `docs/superpowers/plans/2026-07-22-multi-question-request-user-input.md`

- en=0/157 ratio=0.0000

---

# D-05 汉化小结（人工撰写）

任务：把 `docs/superpowers/plans/` 下三个实施计划的英文正文译为简体中文，纯中文替换、不做中英并列。基线 `3ee655e3`，受检文件即本次 `files_owned` 的三篇。

## 1. 每个文件改动量

| 文件 | 总行数 | 改动行（+/-，vs `3ee655e3`）| 汉化前 en/total | 汉化后 en/total |
|---|---|---|---|---|
| `docs/superpowers/plans/2026-07-11-persistent-todo-panel.md` | 866 | +177 / -177 | 161/189 = 0.8519 | 0/189 = 0.0000 |
| `docs/superpowers/plans/2026-07-13-selectable-approval.md` | 629 | +149 / -149 | 129/155 = 0.8323 | 0/158 = 0.0000 |
| `docs/superpowers/plans/2026-07-22-multi-question-request-user-input.md` | 674 | +148 / -148 | 141/157 = 0.8981 | 0/157 = 0.0000 |

改动量取自 `git diff --numstat 3ee655e3 -- <file>`。三个文件均为**纯行内替换**：行数不变、标题层级不变，各级标题数（16/12/12）、复选框 `- [ ]` 数（47/33/31）、fence 行数（64/48/24）与基线逐项相等；文件名与计划日期未改动。

## 2. 残留行清单及白名单理由

**AC-3 残留行清单（en 行）：三个文件均为 0 行。** 上方各文件节中未打印 "AC-3 残留行清单"，正是因为 offenders 为空；被 R5 跳过的行也均为 0 行，所以不存在"被豁免掉的英文正文"。

仍以英文出现的部分全部属于白名单，理由如下：

1. **fenced code block（``` / ~~~）内全部内容** —— 铁律 1：含注释与字符串，一个字符都未改动。三文件 fence 行数与基线一致（64/48/24）。
2. **inline code（反引号）内容** —— 铁律 2。AC-2 的 R4 会先剥离 inline code 再判定，因此不会误判为 en 行；AC-4 则强制其与基线多重集相等。
3. **文件路径、目录名、crate 名、命令与 CLI flag、环境变量 `RUSTCODE_*`、配置键/TOML 表名、`Msg::Xxx` 变体名、枚举变体、函数名/类型名/工具名/hook 名、第三方专有名词（Rust、React、TS、Ollama 等）、版本号/日期/数字** —— 铁律 3，逐字保留。例如 `cargo test -p rustcode-tuix ...`、`release/v5.0.1`、`Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`。
4. **本就为中文的段落** —— 铁律 8，未润色、未改写：`2026-07-22` 中的 `提交 / Submit` 行、`Tab 切换问题` 提示行，以及执行说明里的 `**未真机**`。
5. **链接** —— 铁律 4 在本次无实际对象：三文件基线中 `](...)` 链接目标均为 0 条，改后仍为 0 条。
6. **Emoji** —— 铁律 6：既有 Emoji（文件 1 的 4 个、文件 2 的 8 个、文件 3 的 3 个，含 `☑`、`⚠`、`▌`、`▸`、`✓` 等字形字符）原样保留，未新增也未清理。

## 3. 四项实测结果

| 验收项 | 口径 | 文件 1（todo panel）| 文件 2（approval）| 文件 3（multi-question）| 结论 |
|---|---|---|---|---|---|
| **AC-2** | `en/total <= 0.05` | 0/189 = 0.0000 | 0/158 = 0.0000 | 0/157 = 0.0000 | **PASS** |
| **AC-4** | inline code + fenced code 多重集与基线相等 | 802 → 802 | 748 → 748 | 702 → 702 | **PASS** |
| **AC-7b** | `](...)` 链接目标多重集相等 | 0 → 0 | 0 → 0 | 0 → 0 | **PASS** |
| **AC-32** | Emoji 命中数不增加 | 4 → 4 | 8 → 8 | 3 → 3 | **PASS** |

附带项：**AC-6** frontmatter 键名集合 —— 三文件均无 YAML frontmatter，键名集合为空集且相等，PASS。

## 4. 复现命令与实测输出

```bash
python3 scripts/check-zh-docs.py check --base 3ee655e3 \
  --report .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-05.md \
  --files docs/superpowers/plans/2026-07-11-persistent-todo-panel.md \
          docs/superpowers/plans/2026-07-13-selectable-approval.md \
          docs/superpowers/plans/2026-07-22-multi-question-request-user-input.md
```

实测输出（三文件一起跑，退出码 0）：

```text
PASS docs/superpowers/plans/2026-07-11-persistent-todo-panel.md en=0/189=0.0000
PASS docs/superpowers/plans/2026-07-13-selectable-approval.md en=0/158=0.0000
PASS docs/superpowers/plans/2026-07-22-multi-question-request-user-input.md en=0/157=0.0000

check: 受检 3，PASS 3，FAIL 0
check: 报告已写入 .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-05.md
```

## 5. 过程记录

- 首轮自检中 `2026-07-11-persistent-todo-panel.md` 曾 **AC-4 FAIL**：我在"修改：`crates/rustcode-tuix/src/event_loop/commands.rs`（…）"一行，把基线中**不在** inline code 内的 `reset_to_new_session` 包成了反引号，使工作区比基线多出一个 code span。已改回无反引号写法（"…（reset_to_new_session，约 4295）"），复检 PASS。
- 首轮自检中 `2026-07-13-selectable-approval.md` 曾残留 1 行 en 行（第 49 行）：该行剥离 inline code 后只剩全角括号包裹的 `event_loop`，而全角括号不属于 `CJK_RE` 的判定范围，故被判为英文行。已补入中文（"（位于 event_loop 内）"），复检 en=0。
- 除上述两处外无其他 FAIL。三项改动均未触碰 `docs/` 下其他文件，也未改动任何源码或测试。
