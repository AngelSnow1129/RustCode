# D-35 汉化自检报告（doc-writer）· 根 `README.md` 汉化 + 删除 `README.zh-CN.md`

- 任务：D-35 / feature `2026-09-07-zh-docs-webui`
- 基线：`3ee655e3`，分支 `dev`
- 工具：`python3 scripts/check-zh-docs.py --base 3ee655e3`
- 结论：**AC-8 / AC-9a / AC-9b / AC-9c / AC-10 / AC-11 全部 PASS**，`status: done`
- 与 `03-impl/T-11.md` 的关系：见 §6（T-11 是本文件的**前置**修复，本文件是 T-11 之后的最终快照）

## 1. files_owned 与改动量

| 文件 | 处置 | `--numstat`（相对 `3ee655e3`） | `wc -l` old → new |
|---|---|---|---|
| `README.md` | 汉化 + 事实修正 | `416 458 README.md` | 877 → **835** |
| `README.zh-CN.md` | **删除** | `0 830 README.zh-CN.md` | 830 → 不存在 |

```
$ git diff 3ee655e3 --numstat -- README.md
416	458	README.md

$ git diff 3ee655e3 --numstat -- README.zh-CN.md
0	830	README.zh-CN.md
```

## 2. AC-9a / AC-9b / AC-9c：`README.zh-CN.md` 已删除

```
$ git ls-files README.zh-CN.md
（无输出，0 行）
LSFILES_EXIT=0

$ test ! -e README.zh-CN.md ; echo $?
0

$ git status --porcelain -- README.zh-CN.md
D  README.zh-CN.md
```

- **AC-9a PASS**：`git ls-files README.zh-CN.md` 输出为**空**（该文件已不在索引中）。
- **AC-9b PASS**：`test ! -e README.zh-CN.md` 为**真**（`exit=0`），工作区中已无该文件。
- **AC-9c PASS**：`README.md` 相对基线非空变更（416 增 / 458 删）。

说明：`git status --porcelain` 显示 `D `（第一列 `D` = 索引中已删除、第二列空格 = 工作区与索引一致），即删除动作已 `git rm` 落索引；**本轮（D-35 报告）未执行任何 `git add` / `git rm` / `git commit`**，该状态为既成事实的记录。

## 3. AC-10：`README.md` 的 en/total

```
$ python3 scripts/check-zh-docs.py check --base 3ee655e3 --files README.md
PASS README.md en=0/374=0.0000
  [AC-4-authorized] README.md | removed=- | added=./scripts/build-webui.sh | cause=D2

check: 受检 1，PASS 1，FAIL 0
EXIT=0
```

脚本口径复算（改动前后对照，`total` 定义见 `zh-check-D-34a.md` §1）：

| 版本 | en/total | ratio |
|---|---|---|
| 基线 `3ee655e3` | 388/409 | 0.9487 |
| 工作区（现） | **0/374** | **0.0000** |

**AC-10 PASS**。`[AC-4-authorized] ... cause=D2` 是 AC-4 的授权放行**信息条目**（非 FAIL）：`README.md` 新增了 `` `./scripts/build-webui.sh` `` 这一条 code span，无 span 被越界移除，按 `01-design-addendum.md` 的 D2 类目放行；计数 1，等于 `AC4_MAX_PER_ENTRY`。

## 4. AC-11：链接目标与语言导航

### 4.1 全部 href 目标（9 个，其中本地 7 个）

```
$ grep -oE '\]\(([^)]*)\)' README.md | sed -E 's/^\]\(//; s/\)$//' | sort -u
docs/config.example.toml
docs/ORIGINAL_LICENSE.md
./docs/security/permission-model.md
docs/telemetry.md
docs/UPSTREAM_CREDITS.md
https://agents.md/
https://rustup.rs/
LICENSE
site/docs/en/index.html
```

外链（2 个，`https://agents.md/`、`https://rustup.rs/`）不参与本地存在性判定；对其余 7 个逐一 `test -e`：

```
$ while IFS= read -r h; do t="${h%%#*}"; [ -z "$t" ] && continue; \
    if test -e "$t"; then echo "OK   $t"; else echo "MISS $t"; fi; done < <local-targets>
OK   docs/config.example.toml
OK   docs/ORIGINAL_LICENSE.md
OK   ./docs/security/permission-model.md
OK   docs/telemetry.md
OK   docs/UPSTREAM_CREDITS.md
OK   LICENSE
OK   site/docs/en/index.html
```

**7/7 全部存在，0 个 MISS。AC-11 PASS。**

### 4.2 语言导航已移除

基线的语言切换导航位于 `README.md:16`（基线）：

```
  English · <a href="./README.zh-CN.md">简体中文</a>
```

现工作区 `README.md` 中：

```
$ grep -n "README.zh-CN" README.md
（无输出）
GREP_EXIT=1

$ grep -nE 'English|简体中文' README.md
42:> **Fork 声明。** ... （4）**默认简体中文**——TUI/CLI 界面与 Agent 回复均默认中文 ...
NAV_GREP_EXIT=0
```

- `README.zh-CN` 字面在 `README.md` 中**零命中**。
- 唯一匹配「简体中文」的是 `README.md:42` 的 **Fork 声明正文**（说明产品默认语言），**不是链接、不是导航**。
- 原导航位已无中英切换链接；现有顶部导航为纯中文锚点（`README.md:15-19` 的 `#安装 / #快速开始 / #功能 / #架构 / #开发` 系列）。

## 5. AC-8：外链残留的三域口径

```
$ python3 scripts/check-zh-docs.py gate --base 3ee655e3
PASS AC-8 外链残留
  段 1 正式域（候选 515，已排除 .codebuddy/artifacts/）：0 命中 -> OK
  段 2 历史域（候选 37）：命中 39 处，全部位于 .codebuddy/artifacts/ -> OK（历史痕迹仍在）
  段 3 兜底域 site/.github/docs/extensions（全扩展名）：0 命中 -> OK
```

| 域 | 候选数 | 命中 | 判定 |
|---|---|---|---|
| 段 1 正式域（排除 `.codebuddy/artifacts/`） | 515 | **0** | OK |
| 段 2 历史域（`.codebuddy/artifacts/`） | 37 | **39 处，全部位于 `.codebuddy/artifacts/`** | OK（历史痕迹仍在） |
| 段 3 兜底域 `site/.github/docs/extensions`（全扩展名） | — | **0** | OK |

**结论：AC-8 按三域口径 PASS。** 需显式登记的口径偏差（与 `05-test-report.md` §6 缺陷 **D-2** 同源）：AC 字面表述「全仓 `README.zh-CN` 0 命中」**不成立**——全仓实为 39 处命中，但**全部**位于 `.codebuddy/artifacts/`（历史 feature 交接件），正式域为 0。本判定按 `01-design-addendum.md` 的三域口径作出，不是放宽判据。

## 6. 与 `03-impl/T-11.md` 的关系

`T-11.md` 是 D-35 的**后置修复、本文件的前置依赖**，两者是同一文件（`README.md`）上的先后两个动作：

| 项 | T-11（前置，已 done） | D-35（本文件，最终快照） |
|---|---|---|
| 时间 | 2026-09-08 22:53 | 2026-09-09（本轮记录） |
| 动作 | 删除 `README.md` 中 **2 个 fenced 代码块内部的空行**（删除前行号 258、270） | 汉化 + 删 `README.zh-CN.md` + 四处理事实修正（在 T-11 之前完成） |
| 目的 | 修 AC-4：`code_spans()` 把围栏行自身也计为空 span，新增的 ```` ```bash ```` 块（承载 `./scripts/build-webui.sh`）使围栏行 +2，需删 2 行围栏内空行把 `''` 计数拉回基线 89 | 满足 AC-8…AC-11 |
| 结果 | `README.md` AC-4 由 FAIL 转 PASS，授权明细恰为 1 条 `cause=D2 / added=./scripts/build-webui.sh` | 本文件 §3 复现的正是这条明细 |
| 是否改动非空行 | 否（仅删 2 个空行） | — |

**关键归因差异（以实测为准，已同步 T-11 §4）**：T-11 实测推翻了「多出 2 个空行」的推测——两侧围栏体内空行数**同为 29**，差值 2 实际来自**围栏行数 +2**（工作区 31 个围栏块 / 62 条围栏行 vs 基线 30 个 / 60 条）。因 AC-4 只比较 `''` 的多重集计数、不区分来源，删 2 行围栏内空行同样可把计数拉平，故**改法不变、仅归因不同**。本文件沿用 T-11 的实测归因。

T-11 的回滚警示同样适用于 D-35：**不要**用 `git checkout 3ee655e3 -- README.md` 回滚 `README.md`——该文件整体属汉化改动，整文件回滚会丢失全部成果（含四处理事实修正）。

## 7. 四处理事实修正的落点（逐条实测）

| # | 事实修正 | 落点 | 正文摘录 |
|---|---|---|---|
| 1 | 构建步骤指向 `./scripts/build-webui.sh` | `README.md:200`（```bash 块内） | `./scripts/build-webui.sh` |
| 2 | 默认 host 由 `127.0.0.1` 改为 `0.0.0.0` | `README.md:120` | 「**默认绑定 0.0.0.0** —— `rustcode webui` 与 `rustcode daemon` 两个子命令默认绑定所有网卡……仅靠一次性 token 保护、没有 TLS。显式改回 `127.0.0.1` 即仅本机可访问」 |
| 3 | TUI 内 `/webui` 默认**仍**是 `127.0.0.1`（不随 CLI 变更） | `README.md:121` | 「**TUI 内的 /webui 启动路径默认仍是 127.0.0.1** —— 跨设备访问要显式加 `--host 0.0.0.0`（等价写法 `lan`）」 |
| 4 | WebUI 无需预配 provider、可在网页端配置 | `README.md:122` | 「**无需预先配置 provider** —— 没有 provider 也能打开 Web UI；可在网页「设置」中可视化配置 provider：新增、编辑、删除、设为默认，以及发现模型」 |

配套的两处 `cargo clean -p rustcode-daemon` 约定也已在正文中：`README.md:204`（脚本说明）与 `README.md:217-218`（手工构建步骤后的注意事项）。

三条 `0.0.0.0` / `127.0.0.1` 相关表述与实现一致，实测对照：

- `crates/rustcode-cli/src/main.rs:1047` `#[arg(long, default_value = "0.0.0.0")]`、`crates/rustcode-cli/src/main.rs:1780` `host: "0.0.0.0".to_string()` → 对应修正 2。
- `crates/rustcode-daemon/src/main.rs:21` `const DEFAULT_HOST: &str = "127.0.0.1"`（独立二进制）→ 与修正 3 同向，且被 `crates/rustcode-daemon/tests/default_host_lock.rs` 测试锁定（AC-30）。

## 8. 术语与命名一致性检查结论

- `rustcode` / `RustCode`、crate 名（`rustcode-cli`、`rustcode-daemon`、`rustcode-config`）、路径（`~/.rustcode/config.toml`、`webui/dist/`）、环境变量（`RUSTCODE_*`）、斜杠命令（`/webui`、`/provider`、`/sync`）、脚本名（`scripts/build-webui.sh`）均保持英文原样。
- 术语与全仓一致：`provider`、`token`、`hook`、`subagent`、`skill`、`plugin`、`MCP`、`LSP`、`BYO`、`TUI`、`WebUI` 未作本土化改写。
- 未新增 Emoji（AC-32 在 `gate` 的 231 文件全量 check 中 0 FAIL）。
- 链接目标未变（AC-7b 在 `gate` 中 0 FAIL）；唯一新增的 code span 是 D2 授权项 `./scripts/build-webui.sh`。
- `README.zh-CN` 字样在 `README.md` 中零命中（§4.2）。

## 9. 风险与未验证范围

### 9.1 风险

1. **AC-8 字面口径不成立（低风险，已登记为 G5 缺陷 D-2）**：全仓 `README.zh-CN` 仍有 39 处命中，全部在 `.codebuddy/artifacts/` 历史交接件中。若后续验收按字面口径（全仓 0 命中）复核，需先引用本文件 §5 与 `01-design-addendum.md` 的三域定义，否则会被误判为 FAIL。
2. **`README.md` AC-4 依赖「围栏内空行数」这一脆弱不变量（中风险）**：T-11 通过删 2 个围栏内空行把 `''` 计数拉平。今后任何人向 `README.md` 增删代码块或围栏内空行，都会立刻打破该平衡并触发 AC-4 FAIL。建议后续把这条约束写进 `AGENTS.md`（**不在本轮 `files_owned`**）。
3. **删除 `README.zh-CN.md` 的外链影响（低风险）**：站点 / 其它仓库若有指向 `README.zh-CN.md` 的外链会 404。本仓 `README.md` 内已零引用；仓库外引用无法在本仓验证。
4. **内容准确性依赖人工（中风险）**：四处理事实修正（§7）已与实现逐条比对，但 `README.md` 全文 835 行中其余描述未逐条与代码核对，属于本 feature 的既有验收边界（AC-3 残留行抽检 20/20 白名单，非全文语义复核）。

### 9.2 未验证范围

- 未验证 `README.md` 在 GitHub / 站点渲染下的**锚点跳转**是否全部可用（仅验证了本地文件存在性 `test -e`；`#安装` 类中文锚点属 GitHub 自动生成，未实测）。
- 未验证 `README.zh-CN.md` 删除后，`.codebuddy/artifacts/**` 之外的历史引用（如已发布 release note、外部文档站）是否残留指向。
- 未执行 `cargo` / `npm`，故 §7 的实现侧行号（`main.rs:1047` 等）通过 `grep` 静态核对，未做构建级验证（构建级验证见 `05-test-report.md` §4.C）。

## 10. 回滚方案

两个文件回滚代价**不对称**，需分开处理：

**A. 恢复 `README.zh-CN.md`（低代价）**

```bash
git checkout 3ee655e3 -- README.zh-CN.md
```

后果：AC-9a / AC-9b 立刻 FAIL；`README.md` 若无对应导航回链，会出现「孤立的中英双 README」。回滚后需同步在 `README.md` 恢复语言导航。

**B. 回滚 `README.md`（高代价，不建议整文件回滚）**

`README.md` 的汉化、四处理事实修正、T-11 的 2 空行删除**交织在同一文件**，`git checkout 3ee655e3 -- README.md` 会一次性丢失全部成果。若只需撤回其中一项，按 T-11 §9 的最小回滚集执行：

1. 仅撤回 T-11 → 在 `README.md` 的 `npm install -g @rustcode/rustcode` 与 `# Install using Homebrew` 之间、以及 `source <(rustcode completion bash)` 与 `# Zsh (persistent)` 之间各补回 1 个空行；复跑 `check --base 3ee655e3 --files README.md` 确认明细变化。
2. 仅撤回某一条事实修正 → 编辑 `README.md:120-122` 对应行；复跑 `check` 确认 en 仍为 0。
3. 仅撤回本报告 → 删除 `03-impl/zh-check-D-35.md`（未跟踪新文件，不影响任何门禁口径）。

**判定时机**：复跑 `python3 scripts/check-zh-docs.py check --base 3ee655e3 --files README.md` —— 只要 `en=0/374=0.0000` 保持且退出码为 0，即 `README.md` 本体未被破坏；只要 `gate` 的 AC-8 段 1 仍为「0 命中」，即三域口径未被打破。

## 11. 纪律确认

- 本轮 `files_owned` 仅本报告文件；**未修改 `README.md`、未恢复 `README.zh-CN.md`、未修改任何源码/测试/脚本**。
- 未执行 `cargo` / `npm`；未执行 `git add` / `git rm` / `git commit`；HEAD 仍为 `3ee655e3`。
- 所有数字均为本机实测，可由上文贴出的命令原样复现；本文件无 TODO / 占位符。
