# zh-check 报告

- 基线 (--base): `3ee655e3`
- 阈值 (--en-ratio): `0.05`
- 判定: AC-2 / AC-4 / AC-6 / AC-7b / AC-32，任一失败即 FAIL（fail-closed）

## 汇总

- 受检文件: 13
- PASS: 13
- FAIL: 0

## PASS `docs/plans/2026-07-28-provider-panel-ui-design.md`

- en=0/98 ratio=0.0000

## PASS `docs/plans/2026-08-23-deepseek-v4-flash-evaluation.md`

- en=0/89 ratio=0.0000

## PASS `docs/plans/2026-08-23-deepseek-v4-flash-evaluation-design.md`

- en=0/55 ratio=0.0000

## PASS `docs/plans/2026-07-28-rewind-implementation-plan.md`

- en=0/84 ratio=0.0000

## PASS `docs/plans/2026-07-28-rewind-design.md`

- en=0/59 ratio=0.0000

## PASS `docs/plans/2026-08-07-lightweight-lsp-phase-one.md`

- en=0/71 ratio=0.0000

## PASS `docs/plans/2026-07-26-model-cost-attribution.md`

- en=0/68 ratio=0.0000

## PASS `docs/plans/2026-08-17-project-input-history-design.md`

- en=0/51 ratio=0.0000

## PASS `docs/plans/2026-07-31-native-runtime-datalog.md`

- en=0/51 ratio=0.0000

## PASS `docs/plans/2026-07-24-busy-continue-fork.md`

- en=0/55 ratio=0.0000

## PASS `docs/plans/2026-08-02-session-recovery-design.md`

- en=0/45 ratio=0.0000

## PASS `docs/plans/2026-08-07-config-panel-implementation-plan.md`

- en=0/54 ratio=0.0000

## PASS `docs/plans/2026-08-10-first-stage-planning-quality-design.md`

- en=0/40 ratio=0.0000

---

# D-16 手写小结（汉化任务批次 6）

- 任务 ID：`D-16`
- 基线：`3ee655e3`
- 范围：`docs/plans/` 下 13 个文件（本批 files_owned），同目录其它文件由 D-13/D-14/D-15/D-17 并行处理，未改动。
- 执行方式：先跑基线自检拿 offender 清单，确认**全部 13 个文件均为纯英文**（en_ratio 0.88–1.00），因此全部按「全文译」处理；无 no-op 文件，也无「已中文只清 offender」的文件。

## 1. 每文件改动量

| 文件（`docs/plans/`） | 基线 en/total | 现 en/total | 基线行数 → 现行数 | 空行 前/后 | 标题数 前/后 | 代码块 前/后 | 说明 |
|---|---|---|---|---|---|---|---|
| `2026-07-28-provider-panel-ui-design.md` | 89/100 | 0/98 | 158 → 156 | 31/31 | 12/12 | 3/3 | 全文译；原文已有少量中文（`账号`/`模型`/`按 a 添加第一个 provider` 等）按铁律 8 原样保留 |
| `2026-08-23-deepseek-v4-flash-evaluation.md` | 91/92 | 0/89 | 118 → 115 | 27/27 | 8/8 | 0/0 | 全文译 |
| `2026-08-23-deepseek-v4-flash-evaluation-design.md` | 69/69 | 0/55 | 99 → 85 | 20/20 | 8/8 | 1/1 | 全文译 |
| `2026-07-28-rewind-implementation-plan.md` | 82/93 | 0/84 | 119 → 110 | 27/27 | 6/6 | 0/0 | 全文译；含 v5.0.5 状态覆盖引用块，语义逐句保留 |
| `2026-07-28-rewind-design.md` | 71/71 | 0/59 | 103 → 91 | 26/26 | 8/8 | 2/2 | 全文译 |
| `2026-08-07-lightweight-lsp-phase-one.md` | 56/60 | 0/71 | 80 → 91 | 21/21 | 6/6 | 0/0 | 全文译；原文超长单行为便于阅读拆为多行 |
| `2026-07-26-model-cost-attribution.md` | 59/60 | 0/68 | 80 → 88 | 21/21 | 6/6 | 0/0 | 全文译 |
| `2026-08-17-project-input-history-design.md` | 40/41 | 0/51 | 71 → 81 | 20/20 | 6/6 | 2/2 | 全文译 |
| `2026-07-31-native-runtime-datalog.md` | 53/58 | 0/51 | 75 → 68 | 18/18 | 5/5 | 0/0 | 全文译 |
| `2026-07-24-busy-continue-fork.md` | 38/43 | 0/55 | 60 → 72 | 18/18 | 5/5 | 0/0 | 全文译 |
| `2026-08-02-session-recovery-design.md` | 54/54 | 0/45 | 77 → 68 | 21/21 | 7/7 | 1/1 | 全文译 |
| `2026-08-07-config-panel-implementation-plan.md` | 47/50 | 0/54 | 69 → 73 | 20/20 | 6/6 | 0/0 | 全文译 |
| `2026-08-10-first-stage-planning-quality-design.md` | 47/47 | 0/40 | 55 → 48 | 9/9 | 5/5 | 0/0 | 全文译 |

汇总：13 个文件全部从 en_ratio 0.88–1.00 降到 **0.0000**，无 no-op。

## 2. 残留行清单与白名单说明

**最终自检（第二次，覆盖写报告的那一次）结果为 13 个文件 `en=0`，残留行 0 条**，因此不存在需要白名单豁免的残留。

过程中出现过 2 条临时残留，均已修掉，记录如下以备复核：

1. `2026-08-23-deepseek-v4-flash-evaluation-design.md:38` 原为 `  success、timeout、cancelled、provider error、rate limited、` —— 属于前批踩坑第 2 条（全角顿号 `、` 不在 `CJK_RE` 判定区间内，整行仍算英文行）。已改为把 `或` 提到行尾、并在下一行补 `等失败类别。`，使两行都含真实汉字。
2. `2026-08-02-session-recovery-design.md` 的 `## Daemon API` 标题 —— 纯英文标题必然算 en 行。已改为 `## Daemon 接口`（保留 `Daemon` 术语，补真实汉字）。

## 3. 前批踩坑规避情况

1. **不把普通词包成反引号**：逐文件核对，未新增任何反引号；AC-4 全绿。
2. **全角标点不算汉字**：见上 §2 第 1 条，已补真实汉字。
3. **批量替换波及代码块**：未使用批量替换；每个文件整篇重写时，代码块从基线原文原样复制。已用脚本逐文件比对 fenced block 序列，**13 个文件全部 `OK`，代码块内容逐字节未变**（见 §4 的独立核验）。
4. **空行增删**：已用脚本核对每个文件的空行数，**13 个文件空行数前后完全一致**（31/31、27/27、20/20、27/27、26/26、21/21、21/21、20/20、18/18、18/18、21/21、20/20、9/9），段落结构未变。
5. **`--report` 覆盖**：本手写小结写在**最后一次**自检之后，且此前跑过的一次自检用的是 stdout（未带 `--report`），故不会被覆盖。
6. **跨行反引号对**：开工前用脚本扫描了全部 13 个文件的反引号奇偶性，检出 1 处跨行反引号对 —— `2026-08-23-deepseek-v4-flash-evaluation.md` 第 85–86 行（`` `codex exec --json --cd <run> -o `` / ``    <file>` ``）。**已保留原折行位置与反引号位置**，只把自由散文译为中文，未重排成一行，AC-4 因此保持 PASS。
7. **行数判定**：总行数有增有减（中文更紧凑导致减少，原文超长行拆行导致增加），未增删空行；判定以 AC-2/4/7b/32 为准，均已 PASS。

## 4. 四项实测结果

| 验收项 | 结果 | 实测数据 |
|---|---|---|
| **AC-2** `en/total <= 0.05` | **PASS**（13/13） | 13 个文件全部 `en=0`，ratio 均为 `0.0000`；受检 13，PASS 13，FAIL 0 |
| **AC-4** inline code + fenced code 多重集相等 | **PASS**（13/13） | 脚本未报任何 code span 差异；另用独立脚本复核：13 个文件 fenced block 序列与基线**逐字节相同** |
| **AC-7b** `](...)` 链接目标多重集相等 | **PASS**（13/13） | 本批 13 个文件不含任何 `](...)` 链接，新旧集合均为空集，相等 |
| **AC-32** Emoji 数不增加 | **PASS**（13/13） | 未新增任何 Unicode Emoji；`provider-panel-ui-design.md` 代码块内既有的 `✓` 原样保留，未清理 |

附加核验（脚本之外，保证结构不被改写）：

- **标题层级**：13 个文件的 `#` 开头行数前后完全一致（12/12、8/8、8/8、6/6、8/8、6/6、6/6、6/6、5/5、5/5、7/7、6/6、5/5），未改标题层级。
- **YAML frontmatter**：13 个文件首行均非 `---`，无 frontmatter，故 AC-6 无键名可比较、`description` 无改动 —— **无需按 AC-6 逐条列示 description**。
- **专有名词保留**：`OpenAI`/`Anthropic`/`Ollama`/`DeepSeek`/`Volcano`/`GitHub Copilot`/`Rust`/`Tokio`/`Python`/`Codex`/`ACP`/`Clap`/`crossterm`/`serde`/`JSONL`/`TOML`/`JSON-RPC` 等术语、crate 名（`rustcode-*`）、路径、命令与 CLI flag（`--provider`/`--git-dir`/`--work-tree`/`--features`/`cargo test ...`）、环境变量（`RUSTCODE_HOME`、`RUSTCODE_*`）、配置键（`[lsp]`、`[providers.*]`、`tools.todo.eager`、`enabled=true`、`auto_detect=true`、`owner = native`、`apply: true`）、类型/函数名（`MenuKind::Plugin`、`ProviderWizard`、`build_preset_entry`、`SnapshotHook::turn_start`、`SessionManager::fork_native_session`、`SessionStoreError::SessionInUse`、`CodingRuntimeConfig` 等）、HTTP 路径（`POST /projects/:hash/sessions/:id/repair`）、版本日期数字，一律保留英文原样。

## 5. 边界声明

- 只改 `files_owned` 内的 13 个文件；未改源码、未执行 `cargo`/`npm` 构建与测试、未提交、未打标签。
- 同目录其它文件（D-13/D-14/D-15/D-17 的 files_owned）未触碰。
