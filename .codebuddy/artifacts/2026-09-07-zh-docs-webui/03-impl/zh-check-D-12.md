# zh-check 报告

- 基线 (--base): `3ee655e3`
- 阈值 (--en-ratio): `0.05`
- 判定: AC-2 / AC-4 / AC-6 / AC-7b / AC-32，任一失败即 FAIL（fail-closed）

## 汇总

- 受检文件: 11
- PASS: 11
- FAIL: 0

## PASS `docs/superpowers/specs/2026-07-25-retire-core-provider-B-vision-design.md`

- en=1/47 ratio=0.0213
- AC-3 残留行清单 (en 行，共 1 行):
  - `docs/superpowers/specs/2026-07-25-retire-core-provider-B-vision-design.md:16`: `- **daemon** 'live_api.rs:1119 preprocess_image_caption(config, active: &dyn core::LlmProvider, …)' → 'maybe_preprocess'（core）。`

## PASS `docs/superpowers/specs/2026-07-25-retire-core-provider-C-conversation-design.md`

- en=1/47 ratio=0.0213
- AC-3 残留行清单 (en 行，共 1 行):
  - `docs/superpowers/specs/2026-07-25-retire-core-provider-C-conversation-design.md:35`: `- 'AuthoritativeTerminal.snapshot: ConversationSnapshot' → 'SessionSnapshot'（live_api.rs:72）。`

## PASS `docs/superpowers/specs/2026-07-25-retire-core-tool-ball-D-design.md`

- en=0/44 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-25-round-cap-checkpoint-design.md`

- en=0/80 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-29-user-input-background-block-design.md`

- en=0/57 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-29-webui-sync-compact-design.md`

- en=0/62 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-30-progress-signposts-preamble-design.md`

- en=0/45 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-30-workflow-intent-understanding-design.md`

- en=0/54 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-31-local-scheduled-tasks-design.md`

- en=1/52 ratio=0.0192
- AC-3 残留行清单 (en 行，共 1 行):
  - `docs/superpowers/specs/2026-07-31-local-scheduled-tasks-design.md:98`: `**DEFER**：`

## PASS `docs/superpowers/specs/2026-07-31-local-scheduled-tasks-phase2-design.md`

- en=0/59 ratio=0.0000

## PASS `docs/superpowers/specs/2026-07-31-project-memory-dir-override-design.md`

- en=0/50 ratio=0.0000

---

# D-12 汉化小结（人工撰写）

任务：把 `docs/superpowers/specs/` 下 11 篇设计文档（`files_owned`）里的**英文正文**译为简体中文。基线 `3ee655e3`，受检文件即本次 `files_owned` 的 11 篇。

**摸底结论（决定本批做法）**：这 11 篇在基线 `3ee655e3` 上**已是大段中文**，不属于「英文原文待全译」，而是铁律 §3 描述的情形。因此本次**只处理 `check` 报出的英文 offender 行**，`en/total <= 0.05` 的文件一律记 **no-op**，不改写任何已中文段落。首轮自检结果为 **10 PASS / 1 FAIL**，唯一 FAIL 是 `2026-07-31-local-scheduled-tasks-phase2-design.md`（`en=5/59=0.0847 > 0.05`）。

## 1. 每个文件改动量

| 文件（`docs/superpowers/specs/` 下） | 行数 base→work | 改动行（+/-，vs `3ee655e3`） | 汉化前 en/total | 汉化后 en/total | 判定 |
|---|---|---|---|---|---|
| `2026-07-25-retire-core-provider-B-vision-design.md` | 65 → 65 | **0 / 0** | 1/47 = 0.0213 | 1/47 = 0.0213 | **no-op** |
| `2026-07-25-retire-core-provider-C-conversation-design.md` | 71 → 71 | **0 / 0** | 1/47 = 0.0213 | 1/47 = 0.0213 | **no-op** |
| `2026-07-25-retire-core-tool-ball-D-design.md` | 61 → 61 | **0 / 0** | 0/44 = 0.0000 | 0/44 = 0.0000 | **no-op** |
| `2026-07-25-round-cap-checkpoint-design.md` | 138 → 138 | **0 / 0** | 0/80 = 0.0000 | 0/80 = 0.0000 | **no-op** |
| `2026-07-29-user-input-background-block-design.md` | 96 → 96 | **0 / 0** | 0/57 = 0.0000 | 0/57 = 0.0000 | **no-op** |
| `2026-07-29-webui-sync-compact-design.md` | 108 → 108 | **0 / 0** | 0/62 = 0.0000 | 0/62 = 0.0000 | **no-op** |
| `2026-07-30-progress-signposts-preamble-design.md` | 87 → 87 | **0 / 0** | 0/45 = 0.0000 | 0/45 = 0.0000 | **no-op** |
| `2026-07-30-workflow-intent-understanding-design.md` | 101 → 101 | **0 / 0** | 0/54 = 0.0000 | 0/54 = 0.0000 | **no-op** |
| `2026-07-31-local-scheduled-tasks-design.md` | 107 → 107 | **0 / 0** | 1/52 = 0.0192 | 1/52 = 0.0192 | **no-op** |
| `2026-07-31-local-scheduled-tasks-phase2-design.md` | 107 → 107 | **+5 / -5** | 5/59 = 0.0847 | **0/59 = 0.0000** | **已清理 offender 行** |
| `2026-07-31-project-memory-dir-override-design.md` | 91 → 91 | **0 / 0** | 0/50 = 0.0000 | 0/50 = 0.0000 | **no-op** |

改动量取自 `git diff --numstat 3ee655e3 -- <file>`；10 个 no-op 文件的 numstat **输出为空**（一个字符都未改动）。

**no-op 判定依据**：这 10 个文件在基线 `3ee655e3` 上首轮自检即为 `PASS`，`en/total` 最高 0.0213（B-vision / C-conversation）、次高 0.0192（local-scheduled-tasks），其余 7 个为 0.0000，全部 `<= 0.05`。按任务约定与铁律 §3「只处理 `check` 报出的英文 offender 行；若 `en/total <= 0.05` 则记 no-op，**不得**改写已中文段落」，本次保持零改动。

### 唯一改动文件 phase2 的 5 处改动（纯行内替换，+5/-5）

全部位于 `## Schedule → OS 翻译（纯函数）` 表格，做法统一为**保留英文符号 + 在其后补入中文括注**——不触碰任何反引号内的内容，也不把原本不在反引号里的词新包成反引号：

| 行号 | 改前 | 改后 |
|---|---|---|
| 62 | `\| Schedule \| launchd \| systemd OnCalendar \| schtasks \|` | `\| Schedule（调度频率） \| launchd \| systemd OnCalendar \| schtasks \|` |
| 64 | `\| Daily{HH:MM} \| StartCalendarInterval{Hour,Minute} \| …` | `\| Daily{HH:MM}（每天） \| StartCalendarInterval{Hour,Minute} \| …` |
| 65 | `\| Weekly{wd,HH:MM} \| +Weekday \| …` | `\| Weekly{wd,HH:MM}（每周） \| +Weekday \| …` |
| 66 | `\| Hourly \| StartCalendarInterval{Minute:0} \| …` | `\| Hourly（每小时） \| StartCalendarInterval{Minute:0} \| …` |
| 67 | `\| Interval{N min} \| StartInterval=N*60 \| …` | `\| Interval{N min}（间隔 N 分钟） \| StartInterval=N*60 \| …` |

术语保留说明（铁律 3）：`Schedule` 是类型名，`Daily{HH:MM}` / `Weekly{wd,HH:MM}` / `Hourly` / `Interval{N min}` / `Cron{expr}` 是枚举变体名，`StartCalendarInterval` / `StartInterval` 是 launchd plist 键名，`OnCalendar` / `OnUnitActiveSec` / `hourly` 是 systemd 键名与取值，`/SC` `/ST` `/D` `/MO` 是 `schtasks` 的 CLI flag——一律原文保留，只在行首变体名后补入中文括注，使该行含真实汉字。表格结构与列数未变，`Cron{expr}` 行（第 68 行）本就含中文，未改动。

结构等价性（phase2，base→work）：行数 107→107、标题数 12→12、fence 行数 4→4；11 个文件标题数与 fence 行数逐项相等（fence：0/2/0/6/2/6/6/6/4/4/2）。

## 2. 残留行清单及白名单理由

**AC-3 残留行清单（en 行）：共 3 行，分布在 3 个文件，每个文件 1 行。** 逐条理由：

1. `2026-07-25-retire-core-provider-B-vision-design.md:16`
   `- **daemon** \`live_api.rs:1119 preprocess_image_caption(config, active: &dyn core::LlmProvider, …)\` → \`maybe_preprocess\`（core）。`
   —— 剥掉两段 inline code 后，行内只剩结构性标签 `**daemon**`（`rustcode-daemon` 组件名，铁律 3 保留）与全角括号；**真正的英文正文全在反引号内**，铁律 2 禁止翻译。要消掉该 offender 只能往这句已中文的句子里**插入新的汉字**，属铁律 8 禁止的「改写已中文段落」。该文件 `en/total = 0.0213 <= 0.05`，按 §3 记 no-op。
2. `2026-07-25-retire-core-provider-C-conversation-design.md:35`
   `- \`AuthoritativeTerminal.snapshot: ConversationSnapshot\` → \`SessionSnapshot\`（live_api.rs:72）。`
   —— 同理：两段 inline code 内是类型名与方法签名（铁律 2/3 保留），反引号外只剩文件路径+行号 `live_api.rs:72`（铁律 3 保留原文）与全角括号。`en/total = 0.0213 <= 0.05`，no-op。
3. `2026-07-31-local-scheduled-tasks-design.md:98`
   `**DEFER**：`
   —— 与紧邻的 `**IN（阶段 1）**：` 配对的**英文结构性标签**（`IN` / `DEFER` 是本仓库范围章节的既有约定标记；`IN` 那行因含「阶段」二字不计 en）。它是标签而非正文，且该文件 `en/total = 0.0192 <= 0.05`，按 §3 记 no-op，未改写。

**「附录 · 被 R5 跳过的行」为 0 行** —— 11 个文件都不存在被「缩进代码块 / 引用块内裸日志」规则豁免掉的英文正文，即没有藏在豁免规则后面的漏网段落。

其余仍以英文出现的部分全部属于白名单：

1. **fenced code block**（铁律 1）：内部含注释与字符串一个字符未动；fence 行数 base→work 逐项相等（0/2/0/6/2/6/6/6/4/4/2）。例如 phase2 的架构 ASCII 图与 `OsScheduler` trait 签名、C-conversation 的 2 处 fence。
2. **inline code**（铁律 2）：未改动；也未把任何原本不在反引号里的词新包成反引号（AC-4 多重集相等可证）。
3. **保留原文的技术符号与专名**（铁律 3）：文件路径与目录名（`crates/rustcode-cli/src/schedule_os.rs`、`live_api.rs`、`lib.rs`）、crate 名（`rustcode-cli`、`rustcode-config`、`rustcode-capabilities`、`rustcode-kernel`、`rustcode-coding`）、命令与 CLI flag（`cargo test -p rustcode-cli`、`crontab -l`、`schtasks /Create`）、环境变量（`RUSTCODE_REQUEST_USER_INPUT`）、配置键与 TOML 表名、类型名/枚举变体/函数/trait 名（`OsScheduler`、`InstallState`、`ScheduleTask`、`CommandRunner`、`BashWorkspaceGate`、`ApprovalMiddleware`、`PreprocessOutcome`、`AuthoritativeTerminal`、`SessionSnapshot`）、第三方专名（launchd、systemd、schtasks、crontab、Task Scheduler、macOS、Windows、Linux、Rust、Tokio、VS Code）、版本号/分支/日期/数字（`release/v5.0.4`、`2026-07-31`）。
4. **链接**（铁律 4）：11 个文件基线与工作区的 `](...)` 链接目标**均为 0 条**，改后仍为 0 条（AC-7b 恒等）；本次无链接显示文字需要翻译。
5. **Emoji**（铁律 6）：`round-cap-checkpoint` 4→4、`user-input-background-block` 5→5，其余 9 个文件 0→0；既有的原样保留，未新增、未清理。
6. **frontmatter**（铁律 7）：11 个文件首行均不是 `---`，**均无 YAML frontmatter**，AC-6 键名集合为空集且相等，无 `description` 值改动。

## 3. 四项实测结果

| 文件（`docs/superpowers/specs/` 下） | AC-2 `en/total <= 0.05` | AC-4 code span 多重集 | AC-7b 链接目标多重集 | AC-32 Emoji 不增加 |
|---|---|---|---|---|
| `retire-core-provider-B-vision` | 1/47 = 0.0213 PASS | 107 → 107 相等 | 0 → 0 相等 | 0 → 0 OK |
| `retire-core-provider-C-conversation` | 1/47 = 0.0213 PASS | 91 → 91 相等 | 0 → 0 相等 | 0 → 0 OK |
| `retire-core-tool-ball-D` | 0/44 = 0.0000 PASS | 73 → 73 相等 | 0 → 0 相等 | 0 → 0 OK |
| `round-cap-checkpoint` | 0/80 = 0.0000 PASS | 157 → 157 相等 | 0 → 0 相等 | 4 → 4 OK |
| `user-input-background-block` | 0/57 = 0.0000 PASS | 83 → 83 相等 | 0 → 0 相等 | 5 → 5 OK |
| `webui-sync-compact` | 0/62 = 0.0000 PASS | 104 → 104 相等 | 0 → 0 相等 | 0 → 0 OK |
| `progress-signposts-preamble` | 0/45 = 0.0000 PASS | 64 → 64 相等 | 0 → 0 相等 | 0 → 0 OK |
| `workflow-intent-understanding` | 0/54 = 0.0000 PASS | 70 → 70 相等 | 0 → 0 相等 | 0 → 0 OK |
| `local-scheduled-tasks` | 1/52 = 0.0192 PASS | 90 → 90 相等 | 0 → 0 相等 | 0 → 0 OK |
| `local-scheduled-tasks-phase2` | **0/59 = 0.0000 PASS**（改前 0.0847 FAIL） | 111 → 111 相等 | 0 → 0 相等 | 0 → 0 OK |
| `project-memory-dir-override` | 0/50 = 0.0000 PASS | 103 → 103 相等 | 0 → 0 相等 | 0 → 0 OK |
| **结论** | **11/11 PASS** | **11/11 PASS** | **11/11 PASS** | **11/11 PASS** |

附带项：**AC-6** frontmatter 键名集合 —— 11 文件均无 frontmatter，键名集合为空集且相等，PASS。AC-4 / AC-7b / AC-32 / AC-6 的四列数值由 `scripts/check-zh-docs.py` 的 `code_spans()` / `link_targets()` / `emoji_hits()` / `frontmatter_keys()` 在基线 `3ee655e3` 与工作区两份文本上分别计算后比对得出。

## 4. 复现命令与实测输出

```bash
python3 scripts/check-zh-docs.py check --base 3ee655e3 \
  --report .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-12.md \
  --files docs/superpowers/specs/2026-07-25-retire-core-provider-B-vision-design.md \
          docs/superpowers/specs/2026-07-25-retire-core-provider-C-conversation-design.md \
          docs/superpowers/specs/2026-07-25-retire-core-tool-ball-D-design.md \
          docs/superpowers/specs/2026-07-25-round-cap-checkpoint-design.md \
          docs/superpowers/specs/2026-07-29-user-input-background-block-design.md \
          docs/superpowers/specs/2026-07-29-webui-sync-compact-design.md \
          docs/superpowers/specs/2026-07-30-progress-signposts-preamble-design.md \
          docs/superpowers/specs/2026-07-30-workflow-intent-understanding-design.md \
          docs/superpowers/specs/2026-07-31-local-scheduled-tasks-design.md \
          docs/superpowers/specs/2026-07-31-local-scheduled-tasks-phase2-design.md \
          docs/superpowers/specs/2026-07-31-project-memory-dir-override-design.md
```

实测输出（11 个文件一起跑，退出码 0）：

```text
PASS docs/superpowers/specs/2026-07-25-retire-core-provider-B-vision-design.md en=1/47=0.0213
PASS docs/superpowers/specs/2026-07-25-retire-core-provider-C-conversation-design.md en=1/47=0.0213
PASS docs/superpowers/specs/2026-07-25-retire-core-tool-ball-D-design.md en=0/44=0.0000
PASS docs/superpowers/specs/2026-07-25-round-cap-checkpoint-design.md en=0/80=0.0000
PASS docs/superpowers/specs/2026-07-29-user-input-background-block-design.md en=0/57=0.0000
PASS docs/superpowers/specs/2026-07-29-webui-sync-compact-design.md en=0/62=0.0000
PASS docs/superpowers/specs/2026-07-30-progress-signposts-preamble-design.md en=0/45=0.0000
PASS docs/superpowers/specs/2026-07-30-workflow-intent-understanding-design.md en=0/54=0.0000
PASS docs/superpowers/specs/2026-07-31-local-scheduled-tasks-design.md en=1/52=0.0192
PASS docs/superpowers/specs/2026-07-31-local-scheduled-tasks-phase2-design.md en=0/59=0.0000
PASS docs/superpowers/specs/2026-07-31-project-memory-dir-override-design.md en=0/50=0.0000

check: 受检 11，PASS 11，FAIL 0
check: 报告已写入 .codebuddy/artifacts/2026-09-07-zh-docs-webui/03-impl/zh-check-D-12.md
```

## 5. 过程记录与踩坑

- **摸底先于动手**：先跑首轮 `check` 摸清 11 篇的成色，确认「10 PASS / 1 FAIL」后才动手，避免对已中文文件做无谓改写。唯一需要处理的 offender 集中在 phase2 的同一张表内（5 行连续）。
- **`--en-ratio 0` 不生效**：脚本里 `if not getattr(args, "en_ratio", None)` 把 `0.0` 当假值回落到默认 `0.05`，因此用 `--en-ratio 0` 想强制列出全部 offender 会静默失效。改用 `--en-ratio 0.0001` 才拿到 B-vision / C-conversation / local-scheduled-tasks 三处的 offender 行号。
- **全角括号陷阱（前批实测的坑）**：本次 5 处补入的都是**真实汉字**（调度频率 / 每天 / 每周 / 每小时 / 间隔 N 分钟），不是只加全角标点，因此改后 `en` 由 5 直接归 0，无残留。
- **未新包反引号**：phase2 表格里 `Daily{HH:MM}` 等变体名原本就不在反引号里，改后依然不在；AC-4 多重集 111→111 相等可证（若误包会立即 FAIL）。
- **行数口径**：改动为纯行内替换，`git diff --numstat` 显示 `5  5`，无多插/少删空行；phase2 行数 107→107、标题数 12→12、fence 行数 4→4。
- **范围**：本次只改 `files_owned` 内**唯一需要改的 1 个文件**（`2026-07-31-local-scheduled-tasks-phase2-design.md`），其余 10 个 `files_owned` 文件 `git diff --numstat` 均为空。
- **同目录并发改动说明**：`docs/superpowers/specs/` 是多个汉化批次（D-10 及以后）共享的目录，本次执行期间该目录下另有若干文件相对基线 `3ee655e3` 存在 diff（执行本批自检时实测为 11 个，例如 `2026-04-23-p2-doctor-review-notebook-todo-design.md`、`2026-05-08-vision-preprocessor-design.md`、`2026-05-25-tuix-unified-in-app-scroll-design.md`、`2026-06-07-headless-output-format-json-design.md`、`2026-06-29-acp-agent-design.md`、`2026-07-22-*`、`2026-07-24-windows-native-tls-schannel-fallback-design.md` 等）。它们**不在本批 `files_owned` 内，属于其它批次 / 其它并发执行者，D-12 未触碰**；该清单随其它批次推进会变化，故此处不逐一列举为固定数字。
- **未执行**任何 `cargo` / `npm` 构建与测试，未改任何源码。
