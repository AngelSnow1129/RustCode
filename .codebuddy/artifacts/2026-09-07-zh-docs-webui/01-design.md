---
kind: design
id: DESIGN-001
from: solution-architect
to: [project-manager]
feature: 2026-09-07-zh-docs-webui
status: approved
decision: proceed
requires: [REQ-002]
files_owned: []
architecture_constraints:
  touches_runtime_lifecycle: true
  touches_persistence: false
  touches_cross_crate_deps: false
created: 2026-09-07
---

# DESIGN-001 · 全仓 md 汉化 + WebUI 开箱可访问 + 默认绑定 0.0.0.0

> **基线**：branch `dev`，commit `3ee655e3`，worktree clean。
> **上游输入**：`00-requirement.md`（`status: approved`）、`00-decisions.md`（Q1–Q5 已冻结）。
> **本文件是只读设计产物**：未修改任何生产代码、未修改任何被汉化 md、未修改 `AGENTS.md`/`docs/**`。
> **工具说明**：本环境无 `Bash`。所有"大小 / 语种密度"数据由 `Glob`（返回字节数）与 `Grep --count`（返回命中行数）实测得出，
> 见 §1.2 与 §10.2；凡未能实测的项均显式标注"待脚本产出"，不用猜测填充。

---

## 0. 一句话结论

代码面改动 **5 个任务 / 6 个文件**（2 处 host 常量 + 1 处 doc comment + 1 处警告打印 + 1 处失实注释 + 2 个 i18n 文件 4 条文案）+ **2 个新脚本**；
文档面改动 **37 个汉化任务**，全部由**一个可脚本判定的验收脚本** `scripts/check-zh-docs.py` 逐批自检。
**批次 0（代码 + 脚本）与汉化批次文件集合两两不相交，可全速并行。**

---

## 1. 现状（实测，含对编排者盘点的两处修正）

### 1.1 代码面现状（`文件:行` 均为本次实测）

| # | 事实 | 证据 |
|---|---|---|
| C1 | `rustcode webui --host` 默认 `127.0.0.1`，且 doc comment 也写死该默认值 | `crates/rustcode-cli/src/main.rs:1047` `#[arg(long, default_value = "127.0.0.1")]`；`:1045-1046` 注释 "Bind address (default 127.0.0.1; use 0.0.0.0 to expose …)" |
| C2 | `rustcode daemon` 子命令 host 硬编码 `127.0.0.1`，且 `webui_tokens: Some(token_store)` ⇒ `enforce_token = true` | `crates/rustcode-cli/src/main.rs:1780` `host: "127.0.0.1".to_string()`；`:1784` |
| C3 | 独立 `rustcode-daemon` 二进制 `DEFAULT_HOST = 127.0.0.1`（**Q2 裁决：保持不动**） | `crates/rustcode-daemon/src/main.rs:21` |
| C4 | 非回环警告打印点 | `crates/rustcode-daemon/src/lib.rs:6345-6347` `if host != "127.0.0.1" && … { eprintln!(… Msg::DaemonWarnNonLoopback …) }` |
| C5 | 与之**直接矛盾**的注释（"Default to loopback-only for security … PR #82 …"） | `crates/rustcode-daemon/src/lib.rs:6333-6341` |
| C6 | **新发现**：紧随其后的中文注释"进程内 webui 恒为 127.0.0.1"在新默认下同样失实 | `crates/rustcode-daemon/src/lib.rs:6343-6344` |
| C7 | **新发现**：`--help` 的 `--host` 文案走 i18n，两语种都写死 `127.0.0.1`，仅用于 `webui` 子命令 | `crates/rustcode-cli/src/main.rs:474-478` `.mut_subcommand("webui", …).mut_arg("host", … Msg::CliHelpHost)`；`crates/rustcode-config/src/i18n/zh_cn.rs:2429`；`en.rs:2521` |
| C8 | **新发现（第 3 处 host 默认）**：TUI `/webui` 斜杠命令自带 `127.0.0.1` 默认，独立于 CLI | `crates/rustcode-tuix/src/event_loop/commands.rs:2188-2207`（`"127.0.0.1".to_string()`） |
| C9 | `Msg::CliWebuiNotBuilt` 文案缺 `cargo clean -p rustcode-daemon`，且与 `AGENTS.md:16` 的 `npm ci` 口径不一致 | `zh_cn.rs:1135-1136`、`en.rs:1191-1192`（均为 `npm install`） |
| C10 | 保留项：`Msg::WebuiLanWarning` / `WebuiNonLoopbackWarning` 已存在且文案完整 | `zh_cn.rs:1897-1898`、`en.rs:1960-1961` |
| C11 | `webui` 构建口径：`npm ci` + `vite build`；`engines.node >= 22.6`；`package-lock.json` 存在 | `webui/package.json:8,11,13-15`；`webui/package-lock.json` 56.7 KB |
| C12 | `webui/dist/` 已被忽略，`cargo build` 不触发 npm（AC-16b **现状已满足**） | `.gitignore:88` `dist/`；`grep npm|npx|node crates/*/build.rs` = **0 命中**（实测） |
| C13 | 无 crate 用 `include_str!` 嵌入除 review rules 以外的 md（汉化不进编译产物） | `00-requirement.md:92`（`rules.rs:110-156` 是唯一命中） |

### 1.2 文档面现状（**实测，与编排者盘点有出入，务必读完**）

全仓 md 实测 **290** 个（`Glob **/*.md`，已排除 `node_modules`/`target`）：
`docs/**` 158 · `.codebuddy/**` 48 · `crates/**` 62 · 根 5 · `{webui,docker,evals,packages,extensions}/**` 12 · `{.claude,.goals,.superpowers}/**` 5。
（`00-requirement.md:97` 记录 287；差值 3 = 本 feature 目录新增的 `00-decisions.md` / `00-requirement.md` / `STATUS.md`。**AC-1 一律以脚本产出计数为准，不硬编码。**）

**修正 1 —— "168 个纯英文 / docs/superpowers 是最大英文块" 不成立。**
用「含 Han 字符的行数 / 含任意字母的行数」实测密度（`Grep --count`），结果：

| 目录 | 实测密度分布 | 结论 |
|---|---|---|
| `docs/superpowers/specs/**`（33 个，≈268 KB） | 27 个密度 > 0.6（如 `2026-06-29-acp-agent-design` 0/215、`2026-07-22-*-design` 0–3/94 仅 6 个为 EN，其余 **已是大段中文**） | **约 6 个 EN，27 个已中文** |
| `docs/superpowers/plans/**`（34 个，≈700 KB） | 密度从 0.03 到 0.94 分布（如 `2026-06-29-acp-agent` 0/715 = EN；`2026-05-29-webui` 521/1000+ = 中英混排） | **EN/MIXED 为主，是真正的主体工作量** |
| `docs/plans/**`（33 个，≈232 KB） | 24 个 Han=0（EN），6 个已中文，3 个混排 | 与编排者"26 个"接近 |
| `docs/*.md` 根级（34 个待处理，≈385 KB） | 绝大多数密度 > 0.6（已中文）；`REFACTOR_DESIGN_PHASE1.md` Han=1/44.26KB = **纯英文** | 少量 EN |
| `.codebuddy/**`（48 个） | **全部含 Han**，且 45 个中大部分密度 > 0.7（`04-review/T6.md` 4/14、`strip-atomcode/STATUS.md` 46/46） | **已基本是中文，残余工作约 900 行级别** |
| `crates/**` 非 rules（9 个） | `kernel/SPIKE.md` 0/59、`review/LANGUAGES.md` 1/79、`tuix/tests/smoke.md` 1/56 = EN；`daemon/README.md` 174/415 = 混排；其余已中文 | 3 EN + 1 混排 |
| setup-seeds（6 个） | `SKILL.md` 46/255，5 个 `references/*.md` Han = **0** | **6 个全部需要处理** |

**净效应**：真正的"从零翻译"体量显著小于 168 个文件的估计；但 **AC-2 的判据是"英文行占比 ≤ 5%"，不是"文件是否算中文"** —— 已中文文件中残留的**英文标题、英文条目、英文表格首列**同样会被 AC-2 判为 `en` 行。
因此工作量应重新表述为：**消除全部 md 中的英文散文/标题行，直到每个文件 AC-2 达标**，这与 `00-requirement.md §3.5`（中英混排以段落为最小单位统一到中文）**完全自洽**，不推翻任何已冻结裁决。

**修正 2 —— AC-2 会误伤"代码型英文行"**：`docs/**` 与 `docs/superpowers/**` 含大量 diff/日志/命令样例。AC-2 已规定剥离 fenced code 与 inline code，但**未剥离缩进代码块（4 空格缩进）与引用块 `>` 内的代码**。设计决策见 §10.1 的 `AC-2 补充规则 R5`。

### 1.3 调用方与持久化点

- 默认 host 的**唯一消费链**：`cli/src/main.rs:1047`（`webui`）→ `rustcode_daemon::run_server(ServerOpts{host,…})`（`cli/src/main.rs:1779-1790` 同构）→ `lib.rs:6342` `let addr = format!("{host}:{port}")` → `bind_scanning`。
- 该链路上**无持久化点**：host 不写 `config.toml`、不进 session 快照、不进 daemon wire DTO。故 `touches_persistence: false`。
- 与 `client_interactive_permission`（`lib.rs:1284-1294`）的耦合：`webui` 与 `rustcode daemon` 两条路径 `enforce_token = true`（`cli/src/main.rs:1784`、`lib.rs:5321`），`||` 短路 ⇒ **返回值恒为 true，approval 路径不翻转**。这正是 Q2 选 A 的安全依据，本设计直接继承，不新增任何补偿逻辑（AC-28 判定为"该路径不由 `is_loopback_authority` 单独决定"已被既有代码满足）。

---

## 2. 候选方案与取舍

### 2.1 默认 host 落地方式

| 方案 | 内容 | 取舍 |
|---|---|---|
| **A（选定）** 只改 `cli/src/main.rs` 两处常量 + 同步 doc comment + 同步 `Msg::CliHelpHost` | `webui`/`daemon` 两条进程内路径 `0.0.0.0`；`daemon/src/main.rs:21` 保持 `127.0.0.1`；TUI `/webui` 保持 `127.0.0.1` | 与 Q2 冻结裁决一致；两条路径 `enforce_token=true`，零安全回归；改动面 5 行。**代价**：CLI `rustcode webui` 与 TUI `/webui` 默认值不一致 —— 记为开放问题 O-1，靠 README 说明化解，不改代码 |
| B 三处全改 + 对独立 daemon 补 token 保护 | 需连带修改 VS Code `extensions/vscode/src/daemon/process.ts:369` 与 JetBrains `RustCodeDaemonProcess.kt:116` 的拉起参数（写入 token） | **放弃**：跨仓库改动 + 破坏 `lib.rs:8876-8890` 既有负向断言的语义基线，成本与收益严重不成比例 |
| C 三处全改不补偿 | — | **禁止**：`00-requirement.md §0.1` 已证明会导致无鉴权暴露 + `dangerously_skip_permissions=true` 双杀 |

### 2.2 汉化执行方式

| 方案 | 内容 | 取舍 |
|---|---|---|
| **A（选定）** 验收脚本先行 + 目录分片 + 单文件单 owner | 第一个任务产出 `scripts/check-zh-docs.py`，后续每批先跑脚本拿到该批的 `en` 行清单再动手 | 脚本可判定、可回归、可并行；**代价**：批次 0 有串行前置（可接受，仅 1 个文件） |
| B 人工逐文件审校 + 事后抽查 | 无脚本，靠 reviewer 目检 | **放弃**：AC-2/4/7/32 要求"脚本可判定"，168+ 文件目检不可复现 |
| C 用 LLM 一次性全量重写 | 单 agent 顺序处理 | **放弃**：单批超限；且无法保证 AC-4（标识符集合相等）与 AC-7（链接目标多重集相等） |

### 2.3 验收脚本语言

| 方案 | 取舍 |
|---|---|
| **A（选定）Python 3 单文件（stdlib only）** | Unicode 属性（`\p{Han}` 等价的类）、frontmatter 状态机、fence 状态机、多重集比较用 `re` + `collections` 一次到位；仓库已有 `python3 scripts/acp_smoke.py` / `analyze_datalogs.py` 先例，`AGENTS.md:256` 已用 `py_compile` 作脚本验证惯例。**代价**：不能沿用 `bash -n` |
| B 纯 bash + grep/sed | **放弃**：fenced code 状态机与 `sed -E` 对 `grep -oP '\x{1F300}-…'` 的 Unicode 区间支持在 macOS/BSD sed 上不一致，会产生"本地绿 / CI 红" |

**结论**：`scripts/check-zh-docs.py`（实现）+ 在 `scripts/` 下**不再**新增 `.sh` 包装（避免两个真源）。`scripts/build-webui.sh` 保持 bash（与 `AGENTS.md:256` 的 `bash -n` 惯例一致）。

---

## 3. 目标架构

### 3.1 模块划分

```
scripts/
  build-webui.sh        [新增] 前端一键构建；fail-closed 前置检查；AC-12..16
  check-zh-docs.py      [新增] 汉化验收唯一判据；AC-1/2/3/4/5/6/7/8/32 + hostscan
crates/rustcode-cli/src/main.rs              2 处 host 常量 + 1 处 doc comment
crates/rustcode-daemon/src/lib.rs            -1 处警告打印 + 2 处失实注释
crates/rustcode-config/src/i18n/{zh_cn,en}.rs  4 条文案
**/*.md                                      汉化（跳过 A/B/C 三类）
```

### 3.2 数据流

```
build-webui.sh
  ├─ preflight: node 存在? >= 22.6(从 webui/package.json 动态读)? npm 存在? package-lock 存在?
  ├─ [--if-missing] webui/dist/index.html 存在 → 跳过，exit 0
  ├─ npm ci  ──fail──► exit != 0（含"离线/registry 不可达"提示），不生成半成品
  ├─ npm run build ──fail──► 原样透传 stderr，exit != 0
  └─ verify: test -f webui/dist/index.html（否则 exit != 0，E6）
             + 打印下一步 cargo clean -p rustcode-daemon
```

```
check-zh-docs.py <subcmd>
  inventory : git ls-files '*.md' → 分类 → 03-impl/md-inventory.md          (AC-1)
  check     : 对每个待检文件，取 base 版本(git show)与工作区版本做双轨计算
              ├─ AC-2  en_ratio（含逐行残留清单 → AC-3 人工抽检输入）
              ├─ AC-4  code span 多重集（inline + fenced body）相等
              ├─ AC-6  frontmatter 键名集合相等；description 变更单独列出
              ├─ AC-7  link target 多重集相等
              └─ AC-32 emoji 命中数不增加
              → exit 0/1 + 03-impl/zh-check-<task>.md
  hostscan  : 列出正文中出现 127.0.0.1/localhost 且语义为"默认绑定"的 md（供 T-06 消费）
  gate      : inventory 一致性 + 全量 check + AC-5(a)(b) + AC-7(a) + AC-8 → 集成门禁
```

### 3.3 控制流（并行关系）

```
Batch 0A ──► T-01(check-zh-docs.py) ──┐
Batch 0B ──► T-04/T-05 (.rs/.i18n) ───┼──► Batch 1..13（汉化，依赖 T-01 的脚本）
Batch 0A ──► T-02(build-webui.sh)  ───┘     每批内 ≤3 个 doc-writer 任务，files_owned 两两不相交
Batch 0A ──► T-03(cli main.rs)
                                        ──► Batch 14: T-06 host 文案同步（依赖全部汉化批）
                                        ──► Batch 15: T-07 集成门禁（依赖 T-01..T-06）
```

---

## 4. 接口契约（冻结，实现期不得擅改）

### 4.1 `scripts/check-zh-docs.py`（新增，唯一真源）

```python
#!/usr/bin/env python3
"""RustCode 中文文档验收脚本。stdlib only, 无网络, 无第三方依赖。

用法:
  python3 scripts/check-zh-docs.py inventory [--base REV] [--out PATH]
  python3 scripts/check-zh-docs.py check     [--base REV] (--files P... | --diff) [--report PATH] [--en-ratio F]
  python3 scripts/check-zh-docs.py hostscan  [--base REV]
  python3 scripts/check-zh-docs.py gate      [--base REV] [--report PATH]

退出码: 0 = 全部通过 | 1 = 至少一项 FAIL | 2 = 用法/IO 错误
默认 --base = "3ee655e3"（可用环境变量 ZH_BASE 覆盖）
"""

DEFAULT_BASE: str = "3ee655e3"
EN_RATIO_DEFAULT: float = 0.05

# 分类桶（AC-1：SKIP_A + SKIP_B + SKIP_C + TODO + ZH == 全仓 md 总数）
SKIP_A_LICENSE = ("license", "licence", "notices", "credits", "copying")  # 路径/文件名小写包含即跳过
SKIP_B_RULES   = "crates/rustcode-review/rules/"
SKIP_C_VERBATIM = ("extensions/jetbrains/CHANGELOG.md",)
# 不参与汉化但计入 ZH 桶（owner 非 doc-writer）：
OWNED_ELSEWHERE = ("AGENTS.md", "README.zh-CN.md", ".codebuddy/artifacts/2026-09-07-zh-docs-webui/")

def classify(path: str, han_lines: int, letter_lines: int) -> str:
    """返回 SKIP_A / SKIP_B / SKIP_C / EN / MIXED / ZH。
    ratio = han_lines / max(letter_lines, 1)
    ratio == 0        -> EN
    0 < ratio < 0.5   -> MIXED
    ratio >= 0.5      -> ZH      # 已中文：不许改写，只许修 en 行
    """
```

**AC-2 计算契约（逐字实现，不得简化）**

```python
FENCE_RE   = re.compile(r'^\s*(```|~~~)')
INLINE_RE  = re.compile(r'`[^`\n]*`')
URL_RE     = re.compile(r'(https?://|mailto:)\S+')
HTML_RE    = re.compile(r'<[^>]+>')
IMG_RE     = re.compile(r'!\[[^\]]*\]\([^)]*\)')
TABLE_SEP_RE = re.compile(r'^\|?[\s:\-|]+\|[\s:\-|]*$')
ASCII4_RE  = re.compile(r'[A-Za-z]{4,}')
CJK_RE     = re.compile(r'[\u3400-\u4dbf\u4e00-\u9fff\uf900-\ufaff\u3040-\u30ff]')
EMOJI_RE   = re.compile('[\U0001F300-\U0001FAFF\u2600-\u27BF\uFE0F]')

def strip_frontmatter(text):  # 仅当第 1 行为 `---` 时，剥到下一个 `---`
def ac2_en_lines(text) -> tuple[int, int, list[tuple[int, str]]]:
    """返回 (total, en, offenders)。逐行：
       R1 剥 frontmatter（整块）
       R2 fence 内外状态机：fence 开/关行与其内部全部行 → skip
       R3 4 空格及以上缩进的连续行（缩进代码块）→ skip          【补充规则 R5，见 §10.1】
       R4 去 inline code / URL / HTML 标签 / 图片 / 表格分隔行
       R5 剩余为空行 → skip（不进 total）
       否则 total += 1
       if ASCII4_RE.search(l) and not CJK_RE.search(l): en += 1; offenders.append((lineno, raw))
    """
```

**AC-4 / AC-7 / AC-32 / AC-6 比较契约**

```python
def code_spans(text) -> list[str]:
    """sorted(INLINE_RE.findall(去反引号)) + sorted(fence body 行)；返回排序后的 list（多重集）"""
def link_targets(text) -> list[str]:
    """re.findall(r'\]\(([^)]*)\)') 排序后返回（多重集；只比较目标串，显示文字不参与）"""
def frontmatter_keys(text) -> list[str]:
    """frontmatter 块内 ^([A-Za-z_][A-Za-z0-9_]*): 的键名，排序后返回"""
def emoji_hits(text) -> int:
    """len(EMOJI_RE.findall(text))；与 base 版本比较，只允许 <= """
def frontmatter_desc(text) -> str | None:
    """frontmatter 的 description 值（用于 AC-6 逐条列示）"""
```

**判定与输出**

- `check` 逐文件输出 `PASS/FAIL <path> en=<n>/<total>=<ratio>`；FAIL 时附带 offender 行号与原文（AC-3 抽检输入）。
- `check --report PATH` 写 `.codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-<task>.md`。
- **AC-4 / AC-7 / AC-32 / AC-6 任一失败即 FAIL，不允许"警告后继续"**（fail-closed，对齐 `AGENTS.md:154`）。

### 4.2 `scripts/build-webui.sh`（新增）

```bash
#!/usr/bin/env bash
# 契约（冻结）：
#   set -euo pipefail
#   用法: scripts/build-webui.sh [--if-missing | -h | --help]
#   exit 0  成功（或 --if-missing 且 webui/dist/index.html 已存在 → 打印跳过说明）
#   exit 2  用法错误 / 前置检查失败（无 node、无 npm、无 package-lock、node 版本不足）
#   exit 1  npm ci / npm run build 失败（原样透传其 stderr）
#   要求版本从 webui/package.json 的 engines.node 动态解析（解析失败回退常量 22.6）
#   成功结尾必须打印下一步：cargo clean -p rustcode-daemon
#   全程禁止 sudo（对齐 AGENTS.md:18）
```

### 4.3 代码面改动契约（逐处，行号 + 替换文本）

| # | 位置 | 现状 | 改为 |
|---|---|---|---|
| K1 | `crates/rustcode-cli/src/main.rs:1047` | `#[arg(long, default_value = "127.0.0.1")]` | `#[arg(long, default_value = "0.0.0.0")]` |
| K2 | `crates/rustcode-cli/src/main.rs:1045-1046` | doc comment: `/// Bind address (default 127.0.0.1; use 0.0.0.0 to expose over LAN/public -- note it` / `/// is token-protected only, with no TLS)` | `/// Bind address (default 0.0.0.0: reachable over LAN; use 127.0.0.1 to` / `/// restrict to this machine. Token-protected only, with no TLS)` |
| K3 | `crates/rustcode-cli/src/main.rs:1780` | `host: "127.0.0.1".to_string(),` | `host: "0.0.0.0".to_string(),` |
| K4 | `crates/rustcode-daemon/src/lib.rs:6345-6347` | `if host != "127.0.0.1" … { eprintln!(…Msg::DaemonWarnNonLoopback…); }` | **整块删除**（`Msg::DaemonWarnNonLoopback` 变体与两语种文案**保留不动**，避免牵动 i18n 覆盖测试） |
| K5 | `crates/rustcode-daemon/src/lib.rs:6333-6341` | "Default to loopback-only for security … PR #82 … non-loopback address, a security warning is printed" | 改写为中性说明：默认 `0.0.0.0` 由 **driver（CLI/TUI）** 决定并强制 token；本函数不再按 host 打印警告；非回环风险提示由 `Msg::WebuiLanWarning` / `WebuiNonLoopbackWarning` 承担。**必须删掉"PR #82 / 原 loopback 默认值"的因果叙述**（已失实），不得保留为历史注脚 |
| K6 | `crates/rustcode-daemon/src/lib.rs:6343-6344` | `// 非 loopback 的安全警告即便在 quiet 模式也应输出（仅独立二进制可能触发，`<br>`// 进程内 webui 恒为 127.0.0.1）。` | 随 K4 一并删除（该注释描述的是被删代码） |
| K7 | `crates/rustcode-config/src/i18n/zh_cn.rs:1135-1136` | `Msg::CliWebuiNotBuilt` = `…cd webui && npm install && npm run build\n   cargo build -p rustcode\n` | 见下 §4.4 精确文本 |
| K8 | `crates/rustcode-config/src/i18n/en.rs:1191-1192` | 同上英文版 | 见下 §4.4 精确文本 |
| K9 | `crates/rustcode-config/src/i18n/zh_cn.rs:2429` | `"绑定地址（默认：127.0.0.1）"` | `"绑定地址（默认：0.0.0.0；改 127.0.0.1 则仅本机可访问）"` |
| K10 | `crates/rustcode-config/src/i18n/en.rs:2521` | `"Bind address (default: 127.0.0.1)"` | `"Bind address (default: 0.0.0.0; use 127.0.0.1 for local-only)"` |

**不在契约内（明确不改）**：
- `crates/rustcode-daemon/src/main.rs:21` `DEFAULT_HOST`（Q2 冻结）。
- `crates/rustcode-tuix/src/event_loop/commands.rs:2188-2207` TUI `/webui` 默认（O-1，见 §11）。
- `lib.rs:1284-1294` `client_interactive_permission`、`lib.rs:4671-4674` `dangerously_skip_permissions`、`lib.rs:8876-8890` 既有测试（**一律不动**，N4 独立立项）。
- `lib.rs:5370/5374` 浏览器地址选择（已能正确处理 `0.0.0.0` → 探测 LAN IP，失败回退 `127.0.0.1`）。
- `.gitignore`、任何 `Cargo.toml`、任何 `build.rs`、`AGENTS.md`。

### 4.4 `Msg::CliWebuiNotBuilt` 精确文案（两语种命令行逐条相等，AC-17d）

```rust
// zh_cn.rs
"本二进制未内嵌 webui 资源。\n请先构建前端，再重新构建：\n\n   ./scripts/build-webui.sh\n   cargo clean -p rustcode-daemon\n   cargo build -p rustcode\n\n或手工执行等价步骤：\n   cd webui && npm ci && npm run build\n   cargo clean -p rustcode-daemon\n   cargo build -p rustcode\n"
// en.rs
"webui assets are not embedded in this binary.\nBuild the frontend first, then rebuild:\n\n   ./scripts/build-webui.sh\n   cargo clean -p rustcode-daemon\n   cargo build -p rustcode\n\nOr run the equivalent steps manually:\n   cd webui && npm ci && npm run build\n   cargo clean -p rustcode-daemon\n   cargo build -p rustcode\n"
```

> **AC-31 风险提示**：`AGENTS.md:263` 记录了 i18n 内容测试散落在 `en.rs`/`zh_cn.rs` 末尾的测试模块（如 `codingplan_crypto_tests`）。
> 改这 4 条文案后**必须**跑 `cargo test -p rustcode-config --lib` 全量，不能只跑受影响功能测试。

### 4.5 事件 / 命令 / 错误类型

本次**不新增**任何 `enum` / `trait` / `struct` / 事件 / 命令 / 跨进程协议字段。新增物仅两个可执行文件级入口：
- `scripts/check-zh-docs.py`（进程退出码 0/1/2，stdout 人类可读报告）
- `scripts/build-webui.sh`（进程退出码 0/1/2）
二者均为**本地开发工具**，不进产品运行时、不跨进程、不进任何 crate 的公共 API。

---

## 5. 状态所有权

| 状态 | 唯一所有者 | 只读消费者 | 生命周期 |
|---|---|---|---|
| `ServerOpts.host` | 构造方：`cli/src/main.rs`（`webui` / `daemon`）、`daemon/src/main.rs`（独立二进制） | `run_server` → `format!("{host}:{port}")` → `bind_scanning` | 进程启动期一次性构造，之后不可变；**无第二写入者** |
| 默认 host 常量 | 上表构造方各自的字面量（本次改 2 处） | — | 编译期常量 |
| `Msg::*` 文案 | `crates/rustcode-config/src/i18n/{zh_cn,en}.rs` 的 `match` 臂（一一对应，双语种必须同构） | `rustcode_config::i18n::t()` | 进程期只读 |
| 每个 md 文件 | **单文件单 owner**：同一时刻**只有一个任务的 `files_owned` 含该路径** | 验收脚本（只读）、reviewer（只读） | 批次开始 → 该批 `check` 通过 → 释放 |
| 汉化验收报告 | `scripts/check-zh-docs.py --report` | PM / reviewer | 追加写，批次内独占 |

**单一所有者保证机制**：
1. `02-tasks.md` 为每个任务给出**逐路径 `files_owned`**；同批次任务两两不相交（§7 已逐对校验）。
2. 同一 `.codebuddy/artifacts/2026-09-07-zh-docs-webui/` 目录（本 feature 交接件 + `03-impl/` 报告）**不归属任何汉化任务**，owner 为 SA/PM/TE，避免自引用冲突。
3. 禁止任何 agent 修改不属于自己 `files_owned` 的 md；发现越界改动即判该批 FAIL。

---

## 6. 失败与取消语义

| 场景 | 错误/信号 | 恢复动作 | 禁止 |
|---|---|---|---|
| `build-webui.sh` 无 node/npm | exit 2，stderr 含 `node >= 22.6` 与安装指引 | 安装后重跑 | 继续构建 / 产出半成品 dist |
| `build-webui.sh` node 版本不足 | exit 2，打印"当前 X，要求 >=22.6" | 升级 node | **降级为警告后继续**（AC-14 硬要求） |
| `npm ci` 失败（离线） | exit 1，提示离线可能性与 `--if-missing` | 联网重试 / 用已有 dist | 静默成功 |
| `vite build` 失败 | exit 1，透传 stderr | 按构建器报错修 | 吞掉错误、保留半产出 dist 当作成功 |
| dist 半产出（无 `index.html`） | exit != 0（后置校验） | 删除 `webui/dist` 重跑 | 视为已构建 |
| 汉化任务 AC-2 不达标 | `check` exit 1 + 逐行 offender | 只改 offender 行，重跑 | 放宽阈值 / 改脚本 / 把英文行塞进 code span 规避 |
| 汉化任务 AC-4 不达标（标识符变了） | `check` exit 1 | 从 base 版本把原文逐字拷回 | "顺手"规范化标识符 |
| 汉化任务 AC-7 不达标（链接变了） | `check` exit 1 | 恢复原链接目标，只改显示文字 | 改文件名 / 改锚点 |
| 汉化任务 AC-32 不达标（新增 emoji） | `check` exit 1 | 替换为 ASCII 标签（`AGENTS.md:59,172`） | 保留 emoji / "顺手"清理**既有** emoji |
| 汉化任务中途取消/换人 | 工作区文件可能处于半成品 | 接手者先跑 `check --diff` 取当前 offender 清单再继续；**基线始终是 `ZH_BASE`，不是 HEAD** | 从半成品继续而不重新基线 |
| `--host` 传非法值 | 绑定失败并报错（既有 `bind_scanning` 行为） | 修正后重跑 | 静默回落到 `0.0.0.0`（等于无意暴露） |
| `check` / `gate` 自身异常（文件缺失、git 失败） | exit 2 | 修正环境重跑 | 把异常当 PASS |

**无 pending 请求 / 无长事务**：汉化与脚本都是"读文件 → 写文件 / 读文件 → 退出码"的短操作，无跨进程的 pending 状态，故无 cancel 时期的 fail-closed 悬挂问题；唯一需要保证的是**基线固定**（`ZH_BASE`），避免中途 commit 导致基线漂移而产生假绿。

---

## 7. 迁移与回退

### 7.1 数据格式兼容

- **不涉及**任何持久化格式变更：`config.toml` schema、native `SessionManager/SessionMeta/SessionSnapshot`、daemon wire DTO、`~/.rustcode` 布局全部不动（`touches_persistence: false`）。
- setup-seeds 6 个 md 改动会使 `SEEDS_TARZST` 内容哈希变化（`crates/rustcode-capabilities/src/setup/seeds.rs:8,19`）⇒ 已安装用户触发**一次**种子重装（`setup/mod.rs:136-156`）。这是**单向升级**，不是格式迁移：
  - 不引入双向转换、不引入 legacy writer、不引入 v1/v2 开关（符合 `AGENTS.md:168`）。
  - 若用户手工改过 `$RUSTCODE_HOME/skills/rustcode-automation-recommender/**`，会被覆盖 —— 必写入 release 说明。
- **不恢复**任何历史 core JSON / core 磁盘投影（本需求完全不触及该面）。

### 7.2 回退步骤

| 范围 | 回退动作 | 影响 |
|---|---|---|
| 单个汉化批 | `git checkout -- <该批 files_owned>` | 零影响（纯文档） |
| 全部汉化 | `git checkout -- $(git ls-files '*.md')` + `git checkout -- README.zh-CN.md` | 零影响 |
| K1/K2/K3（host 常量） | `git revert`/`git checkout` 这三个 hunk | 恢复回环默认；无数据迁移 |
| K4/K5/K6（警告与注释） | 同上 | 恢复每次启动的非回环警告；无数据迁移 |
| K7–K10（i18n） | 同上 + 重跑 `cargo test -p rustcode-config --lib` | 无数据迁移 |
| 新增脚本 | `rm scripts/check-zh-docs.py scripts/build-webui.sh` | 无引用者，零影响 |
| setup-seeds | `git checkout -- crates/rustcode-capabilities/assets/setup-seeds/` | 已安装用户的 seed hash 回退，**再次**触发一次重装；可接受 |

> 汉化是**纯文档改动**，不进任何编译产物（C13），故回退永远安全；唯一带"副作用"的是 setup-seeds（触发重装），已单独成批（Batch 12 / D-34）便于独立回退。

---

## 8. 架构边界核对（逐条对照 `AGENTS.md`）

| `AGENTS.md` 约束 | 位置 | 本设计结论 |
|---|---|---|
| 依赖方向 `kernel <- capabilities <- coding <- tuix <- cli`，capabilities 禁止反向依赖 | `:49` | **合规**。改动落在 `rustcode-cli` / `rustcode-daemon` / `rustcode-config` 内部，不新增依赖边、不改方向。`rustcode-config` 是 leaf，改 i18n 文案不产生新依赖。`touches_cross_crate_deps: false` |
| `CodingRuntime` 是唯一运行时所有者，不得自建第二套 live agent 生命周期 | `:52` | **合规**。不触碰 `coding/src/runtime.rs`，不新增运行时生命周期所有者 |
| 目标调用链 `CLI/TUI/daemon/background/ACP/clix → CodingRuntime → kernel Agent` | `:52-53` | **合规**。不新增入口 |
| 启动路径三处（`spawn_native_cli_runtime` / ACP / `kernel_runtime::start_native_runtime*`） | `:53` | **合规，不触碰** |
| 出站 HTTP 唯一入口 `capabilities/src/egress/`，禁止再写 `reqwest::Client::new()` | `:48` | **合规**。本次无任何出站 HTTP |
| 运行时生命周期不变量：approval / pending request 在 cancel/reload/session switch/shutdown 时 fail-closed | `:149-156` | **合规且改善**。改动的两条路径 `enforce_token=true` ⇒ `client_interactive_permission` 恒 true ⇒ approval 不翻转；本次**不削弱**任何既有 fail-closed 行为，也不新增 fail-open 分支。既有的 `dangerously_skip_permissions` fail-open（N4）**不在本次范围**，已按 `00-decisions.md` 独立立项 |
| 涉及 turn completion / compaction 先复核 `LifecycleHooks::turn_complete`，不得新增重叠 hook 或第二压缩状态机 | `:158` | **合规**。不涉及 |
| 历史兼容面：只允许单向 importer；禁止 legacy writer / 双向转换 / 运行时 fallback / 新 core facade | `:162-168` | **合规**。本需求完全不触及 core 退役面 |
| native `SessionManager/SessionMeta/SessionSnapshot` 是唯一 session 持久化模型 | `:139`（`00-requirement.md:421` 引用） | **合规**。不改持久化 |
| 禁止 bridge / fallback / v1-v2 开关 | `:168` | **合规**。`build-webui.sh` 只有一个 `--if-missing` 开关，属"跳过已完成的幂等构建"，**不是**兼容开关或降级 fallback；不引入任何运行时分支 |
| `[STYLE]` 严禁 Unicode Emoji，日志/注释用 ASCII 标签 | `:59,172` | **合规**。AC-32 明确"不新增 emoji"；且**禁止顺手清理既有 emoji**（沿用 `AGENTS.md:244` 有界例外）。K5 注释改写一律 ASCII |
| `[SECURITY]` 禁止硬编码真实 key/token/内部端点 | `:174` | **合规**。脚本不含任何端点/凭据；`0.0.0.0` 是通配绑定地址不是端点 |
| `[ASYNC]` / `[STREAMING]` / `[ERROR]` / `[DEP]` | `:173,175-177` | **合规**。本次无 async / 无 LLM client / 无新增 `thiserror` 错误类型 / 不改 `Cargo.toml` |
| 禁止 `sudo` 运行（`~/.rustcode` root 属主） | `:18` | **合规**。`build-webui.sh` 不调用 sudo，并在输出中提示不要 sudo |
| 重建前端后必须 `cargo clean -p rustcode-daemon`（cargo 不追踪 `webui/dist/`） | `:16` | **合规且落地**：脚本成功结尾主动打印该命令；`CliWebuiNotBuilt` 文案补齐该命令（AC-17b） |
| 修改前检查（branch/commit/符号消费者/Git 历史/状态 owner/失败语义） | `:179-187` | **已履行**：§1 记录 branch/commit 与全部符号消费者与持久化点；§5 状态 owner；§6 失败语义；Git 历史因本环境无 `Bash` 未执行 —— **需在 T-03/T-04 开工前由实现者补跑 `git log --oneline -5 -- <file>`**，已在 `02-tasks.md` 列为每任务前置动作 |
| 不得修改 `AGENTS.md`（N-9） | `00-requirement.md:141` | **合规** |
| 默认中文（`Locale::ZhCn` 三处） | `:103` | **合规且强化**：汉化后文档默认语言与产品默认语言一致 |

**结论：无一条被破坏。安全边界一项（`touches_runtime_lifecycle: true`）已由 Q2 冻结裁决 + `enforce_token=true` 双重闭合，无需新增补偿逻辑。**

---

## 9. 汉化执行统一规范（下游逐字遵守）

> 本节是 `00-requirement.md §3` 的**可执行收敛版**。冲突时以本节为准（本节只做了"更严格/更可判定"的收敛，未放宽任何一条）。

### 9.1 白名单：以下对象出现**在任何位置**都必须字节级原样保留

1. 所有 fenced code block（` ``` ` / `~~~`）内的**全部内容**，含注释与字符串。
2. 所有 inline code（`` `…` ``）内的全部内容。
3. 链接 URL、图片路径、`<img src>`、`mailto:`（**只译显示文字**）。
4. HTML 标签与属性名/值。
5. YAML frontmatter 的**键名**。
6. frontmatter 中作为**标识符**的值：`name`、`model`、`tools`、`allowed_tools`、`user_invocable`、`argument_hint`、`enabled`、`enabledAutoRun`。
7. crate 名 / 二进制名 / cargo 包名（`rustcode-cli`、`rustcode-daemon`、包名 `rustcode`、`rustcodex`）。
8. 文件路径 / 目录名 / 环境变量 / 配置键 / TOML 表名 / 配置值字面量。
9. 命令与子命令、CLI flag（`rustcode webui`、`--host`、`/provider`、`/model`、`--rules-dir`）。
10. HTTP 方法 / API 路径 / 状态码。`Msg` 变体名 / i18n key / 枚举变体。工具名 / 事件名 / hook 名 / 类型名 / 函数名。
11. 产品名 `RustCode`（不译）、命令/crate 前缀 `rustcode`（不译）。第三方专有名词（OpenAI、Anthropic、OpenRouter、Ollama、Tailscale、DeepSeek、GLM、Qwen、VS Code、JetBrains、GitHub）。
12. 提交类型 / 版本号 / 日期 / 数字 / 单位 / IP 字面量（含 `127.0.0.1`、`0.0.0.0`）。

**唯一可译的 frontmatter 字段：`description`**（Q5 裁决为汉化）。但**必须在批次报告中逐条列出被改动的 `description` 及其路径**（AC-6）。

### 9.2 链接与文件名规则

- 不得修改任何相对链接的**目标路径 / 锚点 / 文件名 / 目录名**；只译显示文字。
- 不得重命名或删除任何 md（例外：`README.zh-CN.md`）。
- 全仓 `](#...)` 锚点链接实测为 0 ⇒ 汉化标题不会打断仓内互链；但仍受 AC-7 多重集校验。
- 链接目标为空或为外链（`http(s)://`）时**一律不改**。

### 9.3 禁 Emoji

- 不得新增任何 Unicode Emoji（`AGENTS.md:59,172`）。
- **注意陷阱**：AC-32 的判定区间 `\u2600-\u27BF` 包含 `☑ ✓ ✗ ⚠` 等常见符号，这些在 `docs/superpowers/**` 中**已存在**（如 `☑ 当前任务 · N/M`）。
  规则是**命中数不增加**，因此：保留既有符号（不清理），新增时不引入任何该区间字符 —— 需要打勾/警示时一律用 ASCII（`[x]`、`[!]`、`[*]`）。

### 9.4 混排与已中文文件

- 已是中文的段落**原样保留、不润色、不改写**（N-6）。
- 英文段落/标题/条目按 §9.1 汉化；**以段落为最小单位**，不做半句翻译。
- 若某文件经 `check` 计算 `en_ratio <= 0.05`，**该文件判为 no-op：一个字都不改**，只需在批次报告中记录 `no-op`。

### 9.5 术语表（统一译法，避免各批不一致）

| 英文 | 中文 |
|---|---|
| provider | provider（不译） |
| skill / plugin / command plugin | 技能 / 插件 |
| subagent | 子代理 |
| session / turn / compaction | 会话 / 轮次 / 压缩 |
| approval / permission | 审批 / 权限 |
| daemon | daemon（不译） |
| seed / setup-seeds | 种子 / 种子包 |
| checkpoint / snapshot | 检查点 / 快照 |
| fallback | 回退 |
| runtime / generation | 运行时 / 代（generation 需与 `CONTEXT.md` 术语一致） |

---

## 10. 验收脚本设计

### 10.1 AC-2 补充规则 R5（缩进代码块）

AC-2 原文只剥离 fenced code 与 inline code。实测 `docs/superpowers/**` 与 `docs/archive/**` 大量使用 **4 空格缩进代码块**与**引用块内的日志样例**，这些内容不是散文，若计入 `total`/`en` 会：
- 使纯中文文件因英文日志而误判 FAIL（假红）；
- 诱导 agent 去"翻译日志"，违反 §9.1。

**R5（补充，更严格且更安全）**：连续 4 空格（或 1 个 tab）缩进行、以及 `>` 引用块内紧邻 fenced code 的裸日志行，**不计入 `total` 也不计入 `en`**。
代价：缩进排版下的**真英文散文**被漏检。缓解：R5 只作用于"该行不含任何中文标点（`，。：；、`）"的行，且 `gate` 全量跑时会额外输出一份"被 R5 跳过的行"附录，供 reviewer 抽检。

### 10.2 判定逻辑伪代码（AC-2 / AC-4 / AC-7 / AC-32）

```
function CHECK_FILE(path, base_rev):
    old = git_show(base_rev, path)  or  ""   # 新增文件：old = ""，AC-4/7/32 退化为"与空集合比较"
    new = read(path)

    # ---- AC-2 ----
    total, en, offenders = AC2(new)
    ratio = en / total if total else 0
    if ratio > EN_RATIO: FAIL(path, "AC-2", offenders)

    # ---- AC-4：代码 span 多重集相等 ----
    if sorted(CODE_SPANS(old)) != sorted(CODE_SPANS(new)):
        FAIL(path, "AC-4", symmetric_difference)

    # ---- AC-7(b)：链接目标多重集相等 ----
    if sorted(LINK_TARGETS(old)) != sorted(LINK_TARGETS(new)):
        FAIL(path, "AC-7b", symmetric_difference)

    # ---- AC-32：emoji 不增加 ----
    if EMOJI(new) > EMOJI(old): FAIL(path, "AC-32", new_hits - old_hits)

    # ---- AC-6：frontmatter 键名集合相等 ----
    if sorted(FM_KEYS(old)) != sorted(FM_KEYS(new)): FAIL(path, "AC-6", key_diff)
    if FM_DESC(old) != FM_DESC(new): REPORT(path, "AC-6-desc-changed")   # 需人工确认，不直接 FAIL

    return PASS

function CODE_SPANS(text):
    out = []
    for m in INLINE_RE.finditer(text): out.append(m.group(0).strip('`'))
    inside = false
    for line in text.split('\n'):
        if FENCE_RE.match(line): inside = !inside; continue
        if inside: out.append(line)          # fenced body 原样入集合
    return out

function LINK_TARGETS(text):
    return [t for t in re.findall(r'\]\(([^)]*)\)', text)]
```

### 10.3 `gate`（Batch 15 集成门禁）判定表

| 检查 | 判据 | AC |
|---|---|---|
| 清单自洽 | `SKIP_A + SKIP_B + SKIP_C + TODO + ZH == git ls-files '*.md' 总数`；A=4、B=47、C=1 | AC-1 |
| 英文残留 | 全量 `check` 无 FAIL | AC-2 / AC-3 |
| 标识符 | 全量 AC-4 无 FAIL | AC-4 |
| 运行时载荷 | `git diff $BASE -- crates/rustcode-review/rules/` **空**；`git diff $BASE -- crates/rustcode-capabilities/assets/setup-seeds/` **非空** 且 `git diff -U0 … \| grep -E '^[+-]name:'` **无输出** | AC-5 |
| frontmatter | 全量 AC-6 无 FAIL；`description` 变更已列清单 | AC-6 |
| 链接 | 无改名/删除（除 `README.zh-CN.md`）；全量 AC-7b 无 FAIL | AC-7 |
| 外链残留 | `grep -rn "README\.zh-CN" --include={md,html,json,ts,kt,yml,toml} .` + `grep -rn README.zh-CN site/ .github/ docs/ extensions/` 均 **0 命中** | AC-8 |
| README | `git ls-files README.zh-CN.md` 空；`test ! -e README.zh-CN.md`；`README.md` 有非空变更；`README.md` 的 AC-2 ≤ 0.05；其 `href` 非外链者 `test -e` 通过 | AC-9 / 10 / 11 |
| Emoji | 全量 AC-32 无新增 | AC-32 |

> AC-12..18（构建脚本）、AC-19..23（绑定与可达）、AC-24..26（无 provider 可访问）、AC-27..30（审批回归）**不在 `gate` 内**：
> 前者由 `bash -n` + 手工/冒烟验证（AC-13/14/15 需要操纵 PATH 与 node 版本，脚本自证不可靠）；后者需要真实网络与浏览器，归 `test-engineer` 的人工验收清单（见 `02-tasks.md` T-07）。

---

## 11. 风险与开放问题

### 11.1 风险（按严重度）

| # | 风险 | 等级 | 缓解 |
|---|---|---|---|
| R1 | **AC-2 判据过严导致"翻译英文标题"工作量被低估**：已中文文件的英文 H1/H2 会被判 `en`，若一个 60 行文件有 4 个英文标题即 6.7% > 5% | 高 | 每批开工前先跑 `check` 拿 offender 清单再动手；**只改 offender 行**；no-op 文件不改。已中文文件的改动量实测很小（`.codebuddy` 全目录残余约 900 行级别） |
| R2 | i18n 文案改动触发 `rustcode-config` 内容测试红（`AGENTS.md:263` 先例） | 中 | T-05 强制 `cargo test -p rustcode-config --lib` 全量，不只跑受影响测试 |
| R3 | setup-seeds 改动触发已安装用户种子重装并覆盖其手工修改 | 中 | 单独成批（D-34）；release 说明写明；`SKILL.md` 的 `name:` 行不动（AC-5b 卡死） |
| R4 | 多批并行时同一 md 被两个 agent 写 | 中 | `files_owned` 两两不相交（§7 已校验）；`gate` 的 AC-7(a) 会兜底发现删除/改名 |
| R5 | `check` 基线漂移（中途 commit 导致与 HEAD 比较变假绿） | 中 | 基线固定为 `ZH_BASE=3ee655e3`，**不随 HEAD 走**；仅可用 `--base` 显式覆盖 |
| R6 | 大文件（78 KB / 73 KB）单任务超限 | 中 | 已单独成任务（D-01 / D-02），不强求 12–20 文件/批 |
| R7 | `build-webui.sh` 在无 npm 的 CI 上被误当作构建依赖 | 低 | 不改任何 `build.rs`（AC-16b 现状已 0 命中），脚本不进 cargo 构建图 |
| R8 | R5（缩进代码块）漏检真英文散文 | 低 | `gate` 输出"被 R5 跳过的行"附录供抽检 |

### 11.2 开放问题

- **O-1（不阻断，需 PM 知会）**：TUI `/webui`（`crates/rustcode-tuix/src/event_loop/commands.rs:2188-2207`）默认仍是 `127.0.0.1`，与 CLI `rustcode webui` 的新默认 `0.0.0.0` 不一致。
  **本设计不改它**（Q2 冻结只授权两处；TUI 是本机交互路径，`enforce_token=true`，无安全回归）。
  处理：在 `README.md`（D-35）与 `docs/` 的 webui 章节（T-06）写明"TUI `/webui` 默认仅本机，跨设备用 `/webui --host 0.0.0.0`"。
- **O-2（不阻断）**：`docs` 根级还有 3 个文件（`REFACTOR_SUMMARY.md`、`telemetry.md`、`acp-sdk-handler-notes.md`）语种密度未实测到 —— 由 `inventory` 子命令在 T-01 首次运行时定档，不改变批次归属（均已在 D-23 的 `files_owned` 内）。
- **O-3（不阻断）**：全仓 md 实测 290 个，`00-requirement.md:97` 记录 287，差 3 = 本 feature 目录的 3 个交接件。**以脚本计数为准**，不回填硬编码数字到任何 AC。
- **O-4（不阻断，与 N4 同源）**：`lib.rs:4671-4674` 的 `dangerously_skip_permissions` fail-open 与 `AGENTS.md:154` 冲突，已按 `00-decisions.md` 独立立项，本设计不动。

### 11.3 回滚触发条件

`gate` 连续两轮 FAIL 且根因指向 §9 规范本身（而非单批疏漏）→ 上报编排者，按 §7.2 分范围回退；**禁止**为让 `gate` 变绿而修改 `scripts/check-zh-docs.py` 的判据或阈值。
