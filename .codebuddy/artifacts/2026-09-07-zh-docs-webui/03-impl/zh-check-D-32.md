# zh-check 报告

- 基线 (--base): `3ee655e3`
- 阈值 (--en-ratio): `0.05`
- 判定: AC-2 / AC-4 / AC-6 / AC-7b / AC-32，任一失败即 FAIL（fail-closed）

## 汇总

- 受检文件: 9
- PASS: 9
- FAIL: 0

## PASS `crates/rustcode-daemon/README.md`

- en=2/210 ratio=0.0095
- AC-3 残留行清单 (en 行，共 2 行):
  - `crates/rustcode-daemon/README.md:1`: `# rustcode-daemon`
  - `crates/rustcode-daemon/README.md:600`: `### MCP（Model Context Protocol）`

## PASS `crates/rustcode-clix/README.md`

- en=3/82 ratio=0.0366
- AC-3 残留行清单 (en 行，共 3 行):
  - `crates/rustcode-clix/README.md:83`: `**GitHub:**`
  - `crates/rustcode-clix/README.md:171`: `C# / Swift / Objective-C / Dart / Scala / Ruby / PHP / Groovy / Lua / Perl / R / Elixir /`
  - `crates/rustcode-clix/README.md:173`: `HTML / CSS / XML / YAML / JSON / TOML / Protobuf / GraphQL / Makefile / CMake / properties /`

## PASS `crates/rustcode-kernel/SPIKE.md`

- en=0/61 ratio=0.0000
- R5 跳过的行 (共 2 行):
  - `crates/rustcode-kernel/SPIKE.md:74`: `    cargo test -p rustcode-kernel`
  - `crates/rustcode-kernel/SPIKE.md:75`: `    cargo run -p rustcode-kernel --example minimal_specialization`

## PASS `crates/rustcode-review/LANGUAGES.md`

- en=0/74 ratio=0.0000

## PASS `crates/rustcode-review/README.md`

- en=1/41 ratio=0.0244
- AC-3 残留行清单 (en 行，共 1 行):
  - `crates/rustcode-review/README.md:1`: `# rustcode-review(L2)`

## PASS `crates/rustcode-coding/README.md`

- en=1/49 ratio=0.0204
- AC-3 残留行清单 (en 行，共 1 行):
  - `crates/rustcode-coding/README.md:1`: `# rustcode-coding (L2)`

## PASS `crates/rustcode-capabilities/README.md`

- en=1/46 ratio=0.0217
- AC-3 残留行清单 (en 行，共 1 行):
  - `crates/rustcode-capabilities/README.md:1`: `# rustcode-capabilities (L1)`

## PASS `crates/rustcode-tuix/tests/smoke.md`

- en=0/55 ratio=0.0000

## PASS `crates/rustcode-kernel/README.md`

- en=1/23 ratio=0.0435
- AC-3 残留行清单 (en 行，共 1 行):
  - `crates/rustcode-kernel/README.md:1`: `# rustcode-kernel (L0)`

## 附录 · 被 R5 跳过的行（供抽检）

共 2 行。

- `crates/rustcode-kernel/SPIKE.md:74`: `    cargo test -p rustcode-kernel`
- `crates/rustcode-kernel/SPIKE.md:75`: `    cargo run -p rustcode-kernel --example minimal_specialization`

---

# D-32 手写小结（doc-writer）

- 任务：批次 11 / 汉化任务 D-32，基线 `3ee655e3`，工具 `scripts/check-zh-docs.py`。
- 命令（最后一次自检，本小结在其后追加，防止 `--report` 覆盖）：

```bash
python3 scripts/check-zh-docs.py check --base 3ee655e3 \
  --report .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-32.md \
  --files crates/rustcode-daemon/README.md crates/rustcode-clix/README.md \
  crates/rustcode-kernel/SPIKE.md crates/rustcode-review/LANGUAGES.md \
  crates/rustcode-review/README.md crates/rustcode-coding/README.md \
  crates/rustcode-capabilities/README.md crates/rustcode-tuix/tests/smoke.md \
  crates/rustcode-kernel/README.md
```

结果：**受检 9，PASS 9，FAIL 0**。

## 1. 边界声明（本批特殊说明 2）

`crates/rustcode-review/rules/` **未被改动、未被进入**：

```bash
git diff 3ee655e3 -- crates/rustcode-review/rules/   # 输出为空
git diff 3ee655e3 -- crates/rustcode-review/rules/ | wc -c   # 0
```

本批只处理了 `rustcode-review` 的 `README.md` 与 `LANGUAGES.md`。`LANGUAGES.md` 中出现的
`crates/rustcode-review/rules/<name>.md`、`rules/cangjie.md` 均为正文/代码块内的路径引用，
只译了周边散文，未打开、未修改任何 `rules/*.md`（共 47 个运行时 prompt 载荷文件，Q1 裁决不汉化）。

## 2. 逐文件改动量

| 文件 | 判定 | 改动量（+/- 行） | 说明 |
|---|---|---|---|
| `crates/rustcode-daemon/README.md` | **no-op** | 0 / 0 | 改动前 en=2/210=0.0095 已 ≤0.05，按铁律 §3 不改写已中文段落 |
| `crates/rustcode-clix/README.md` | **no-op** | 0 / 0 | 同上，en=3/82=0.0366；3 行残留均为专有名词/语言名 |
| `crates/rustcode-capabilities/README.md` | **no-op** | 0 / 0 | 同上，en=1/46=0.0217；唯一残留是 H1 标题里的 crate 名 |
| `crates/rustcode-coding/README.md` | 清 offender | 1 / 1 | 仅 `## Cargo features` → `## Cargo 特性`（与 kernel/review 三处同名标题统一），未触碰任何已中文段落 |
| `crates/rustcode-kernel/README.md` | 清 offender | 1 / 1 | `## Cargo features` → `## Cargo 特性`（AC-2 由 0.0870 降至 0.0435） |
| `crates/rustcode-review/README.md` | 清 offender | 2 / 2 | `## Findings` → `## 发现项（Findings）`、`## Cargo features` → `## Cargo 特性`（0.0732 → 0.0244） |
| `crates/rustcode-kernel/SPIKE.md` | **全文译** | 71 / 67 | 原文 en=54/57=0.9474；空行数 17→17 未变，仅段内重排（铁律 §5-7 允许行数变化） |
| `crates/rustcode-review/LANGUAGES.md` | **全文译** | 94 / 94 | 原文 en=63/74=0.8514；表格行数与分隔符行原样保留 |
| `crates/rustcode-tuix/tests/smoke.md` | **全文译** | 55 / 55 | 原文 en=55/55=1.0000；只改文字，未改任何断言与命令 |

## 3. 残留行清单与白名单理由

| 残留行 | 白名单理由 |
|---|---|
| `rustcode-daemon/README.md:1` `# rustcode-daemon` | crate 名，铁律 3 保留原文 |
| `rustcode-daemon/README.md:600` `### MCP（Model Context Protocol）` | 协议专有名词，铁律 3；且中文段首已给出「MCP」 |
| `rustcode-clix/README.md:83` `**GitHub:**` | 第三方平台专有名词，铁律 3 |
| `rustcode-clix/README.md:171`、`:173` 语言名清单 | 本批特殊说明 4：语言名（C#/Swift/Objective-C/…）保留英文 |
| `rustcode-review/README.md:1` `# rustcode-review(L2)` | crate 名 + 分层标签，铁律 3 |
| `rustcode-coding/README.md:1` `# rustcode-coding (L2)` | 同上 |
| `rustcode-capabilities/README.md:1` `# rustcode-capabilities (L1)` | 同上 |
| `rustcode-kernel/README.md:1` `# rustcode-kernel (L0)` | 同上 |

以上 8 行全部是「crate 名 / 第三方专有名词 / 语言名」，按铁律 3 与批次特殊说明 3、4 必须保留，
不视为漏译。所有文件 en/total 均 ≤0.05，AC-2 PASS。

## 4. 四项实测结果（逐文件）

| 文件 | AC-2 (≤0.05) | AC-4 code span 多重集 | AC-7b 链接目标多重集 | AC-32 emoji |
|---|---|---|---|---|
| `rustcode-daemon/README.md` | PASS 2/210=0.0095 | PASS（493，与基线相等） | PASS（1，相等） | PASS 0→0 |
| `rustcode-clix/README.md` | PASS 3/82=0.0366 | PASS（175，相等） | PASS（2，相等） | PASS 0→0 |
| `rustcode-kernel/SPIKE.md` | PASS 0/61=0.0000 | PASS（75，相等） | PASS（0，相等） | PASS 0→0 |
| `rustcode-review/LANGUAGES.md` | PASS 0/74=0.0000 | PASS（150，相等） | PASS（0，相等） | PASS 0→0 |
| `rustcode-review/README.md` | PASS 1/41=0.0244 | PASS（101，相等） | PASS（4，相等） | PASS 0→0 |
| `rustcode-coding/README.md` | PASS 1/49=0.0204 | PASS（60，相等） | PASS（1，相等） | PASS 0→0 |
| `rustcode-capabilities/README.md` | PASS 1/46=0.0217 | PASS（63，相等） | PASS（0，相等） | PASS 0→0 |
| `rustcode-tuix/tests/smoke.md` | PASS 0/55=0.0000 | PASS（31，相等） | PASS（0，相等） | PASS 5→5 |
| `rustcode-kernel/README.md` | PASS 1/23=0.0435 | PASS（38，相等） | PASS（0，相等） | PASS 0→0 |

- AC-6：本批 9 个文件**均无 YAML frontmatter**（首行不是 `---`），不存在键名或 `description` 变更，无需逐条列示。
- 未新增任何 Unicode Emoji；`smoke.md` 原有的 `❯`、`✓`、`✗`（脚本 AC-32 命中区间内）原样保留，计数 5→5。

## 5. 关键取舍（便于复核）

1. **表格行必须带汉字**：`LANGUAGES.md` 的行在剥离 inline code 后只剩语言名（如 `Rust`），
   仍被 AC-2 判为英文行；因此表格首列统一采用「**英文原名 + 中文类别词**」
   （`Rust 语言`、`HTML 文档`、`Gradle 构建文件`、`Terraform 基础设施`）。
   语言名、规则名（`arkts`/`mapper_dao_xml`…）与文件模式全部原样保留在 inline code 中，
   未增删一个反引号。`Cangjie (仓颉)` 原文已含中文，未改动。
2. **`SPIKE.md` 的跨行/裸写符号**：`Message.meta`、`meta.round`、`Snapshot`、`Err`、
   `ToolStarted`、`user_prompt_submit` 等原文**未**包在反引号里，译文同样**未**加反引号，
   避免 AC-4 FAIL；`SPIKE.md:74-75` 的 4 空格缩进命令块按 R5 跳过，未翻译、未改缩进。
3. **`## Cargo features` → `## Cargo 特性`**：保留工具名 Cargo（铁律 3），译出通用词 features。
   三个 crate README 的同名标题一并对齐。已全仓确认无 `#cargo-features` / `#findings` 锚点引用。
4. **`smoke.md` 位于 `tests/` 但属文档**：只改显示文字，`cargo test -p rustcode-tuix`、
   `cargo clippy`、79 个用例数、ANSI 转义串、`~/.rustcode/history` 等全部逐字保留。

## 6. 未完成 / 需后续留意

- 无遗留 TODO、无占位符。
- `crates/rustcode-coding/README.md` 的 `## Cargo features` 本属「已达标文件的 residual offender」，
  本次为术语一致性一并改掉；若编排者希望 PASS 文件严格零改动，可单独回退这一行（回退后仍 PASS 0.0408）。
- 未执行 `cargo`/`npm` 构建与测试（纯文档任务，铁律 §7）。

## 7. 关于本报告文件自身

`03-impl/zh-check-D-32.md` 是**脚本生成的验收产物**（不属于 D-32 的 files_owned），其
AC-2 英文行全部来自脚本自动段（`PASS: 9` / `FAIL: 0` / `## PASS \`<path>\`` / `en=N/M ratio=`），
手写小结部分无英文残留行。这与前批同类产物一致：D-27 `0.1961`、D-29 `0.1111`、D-32 `0.1786`。
若编排者要求产物文件也满足 AC-2，需要改脚本模板而非本批手写内容，故此处上报、不擅自改写。
