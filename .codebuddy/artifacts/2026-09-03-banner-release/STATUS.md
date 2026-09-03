# 2026-09-03-banner-release 看板

- 当前阶段：开发（T1 完成；多平台产物部分完成，待授权扩展）
- 基线：branch=dev commit=2e5baa33 worktree=dirty
- 前序：`2026-09-03-git-wrapup`（已完成并推送，dev=2e5baa33 / main=a81fb69f）

## 用户原始诉求（逐字记录）

> 继续多agent进行推进，并尝试构建生成多平台的产物，并且readme中的图案中的atomcode图案还没有替换

拆为三条：

1. 继续用多 Agent 流水线推进
2. 尝试构建生成多平台产物
3. README 中的「图案」（字符画）里的 atomcode 未替换

## 门禁

| 门禁 | 状态 | 依据 |
|---|---|---|
| G1 需求 | **pass** | 三条诉求明确；第 3 条已定位为 figlet 字符画（非文本，grep 抓不到） |
| G2 设计 | **裁剪** | 单文件/资源级改动，无接口契约变化（依据协作方案 §9.3） |
| G3 实现 | **部分** | T1 done（README 横幅 + jetbrains 夹具 + agents 旧名）；多平台产物仅原生目标完成 |
| G4 审查 | pending | 待派发 `code-reviewer` |
| G5 测试 | pending | 待跑受影响的 jetbrains 测试（本机无 Gradle/JDK 环境验证未做） |
| G6 交付 | pending | 提交/推送需用户授权 |

## 任务

| id | 标题 | 负责人 | 状态 | 交接件 | files_owned |
|---|---|---|---|---|---|
| T1 | README 字符画 `AtomCode` → `RustCode` | 编排者（字符画对空白敏感，脚本化替换） | **done** | 本节记录 | `README.md`、`README.zh-CN.md` |
| T2 | jetbrains 测试夹具 `atomcode-daemon*` → `rustcode-daemon*` | 编排者 | **done** | 本节记录 | `extensions/jetbrains/src/test/resources/resources/bin/*/` |
| T3 | `.codebuddy/agents/` 旧 crate 名清理 | doc-writer | **done** | `03-impl/T1.md` | `.codebuddy/agents/{code-implementer,solution-architect}.md` |
| T4 | 原生 release 构建（x86_64-unknown-linux-gnu） | 编排者 | **done** | 本节记录 | `target/release/` |
| T5 | 扩展到其他平台产物 | 待授权 | **blocked** | — | — |

## T1：README 字符画（核心发现）

**为什么此前没被发现**：横幅是 ASCII art，纯文本 `grep atomcode` **命中 0**，
因此重命名时所有基于文本 grep 的门禁（G7/G8）都放过了它。

- 位置：`README.md:3-8` 与 `README.zh-CN.md`（同一横幅，全仓仅这 2 处副本）
- 解码证据：用 pyfiglet 对 21 种字体 × 9 个候选串做程序化比对，
  最佳匹配 = **figlet standard 字体的 `AtomCode`**（sim=0.8233，次优 0.8065）。
  字形特征可辨识：`|_| |_| |_|`=m、`____` + `/ ___|`=C、`__| |`…=d、`___`/`\___|`=e
- 处置：用脚本（`/tmp/fix_banner.py`）替换 `<pre>` 块，**不手工转录**（艺术字含大量
  `\` 与反引号，且对空格敏感；手工 Edit 有此前越界事故的前车之鉴）
- HTML 转义：`<pre>` 是裸 HTML，新横幅第 4 行含 `<`（`|  _ <`），已转义为 `&lt;`
- **验证（三重）**：
  1. 与 `pyfiglet("RustCode")` 输出**逐字节一致**（True / True）
  2. 反向相似度：RustCode **1.0**，AtomCode 仅 0.176
  3. 旧字形特征 `|_| |_| |_|` 在两个文件中均已消失

## T2：jetbrains 测试夹具（连带发现的真实缺陷）

`RustCodeDaemonProcess.kt:90/154/183` 用 `executableName("rustcode-daemon")` 拼出
`resources/bin/$platformDir/rustcode-daemon[.exe]` 去查捆绑 daemon；
但磁盘上的夹具仍叫 `atomcode-daemon*` → **查找必然落空**。
因路径是动态拼接，文本 grep 同样抓不到。

- 已 `git mv` 5 个文件（darwin-arm64 / darwin-x64 / linux-arm64 / linux-x64 / win32-x64）
- 目录名与代码 `platformDir()` 的 5 个取值**完全对应**（已逐项核对）
- 夹具实为 20 字节 ASCII 占位文件（内容 `test bundled daemon`），非真二进制
- 生产代码零引用（仅测试夹具面）

## T3：`.codebuddy/agents/` 旧 crate 名（编排者复核属实）

`.codebuddy/` 自 2026-09-03 起纳入版本控制后，Agent 定义里的旧 crate 名变成**会误导子 Agent 的活性缺陷**
（子 Agent 按 `atomcode-*` 去 `-p` 必然失败）。

- `code-implementer.md:74`（6 处）、`solution-architect.md:96,99`（3+1 处）→ 全部改 `rustcode-*`
- 复验：`grep -rn atomcode .codebuddy/agents/` → **0 命中**；diff 为 3 行纯 token 替换（3+3）
- 刻意保留：`.codebuddy/artifacts/**` 的历史记录、`docs/UPSTREAM_*` 与 `LICENSE` 的 MIT 归属声明

## T4：原生 release 产物（已完成）

| 产物 | 大小 | 类型 |
|---|---|---|
| `target/release/rustcode` | 32,701,552 B | ELF 64-bit LSB pie, x86-64, dynamically linked |
| `target/release/rustcode-daemon` | 27,304,112 B | 同上 |

- `cargo build --release -j 2`（default-members）→ **exit=0**
- 冒烟：`./target/release/rustcode --version` → `rustcode 5.0.9 (2e5baa33+dirty)`
- `rustcode-tuix` 是 lib，无独立二进制，故产物为 2 个

## T5：多平台扩展的可行性结论（实测，不含推测）

**硬约束 1 —— rustup 仍缺失**：`/root/.cargo/bin/{cargo,cargo-clippy,cargo-fmt}` 均为指向
已删除 `rustup` 的悬空链接；全部构建走工具链绝对路径
`/root/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/{cargo,rustc}` + PATH 注入。
`rustup target add` 因此不可用 —— **这是无法扩展目标平台的第一道门**。

**硬约束 2 —— 本机仅安装 1 个目标**：`rustlib/` 下只有 `x86_64-unknown-linux-gnu`。

**交叉工具实测**：`musl-gcc` / `zig` / `cross` / `cargo-zigbuild` / `gcc-aarch64-linux-gnu` /
`x86_64-w64-mingw32-gcc` **全部 MISSING**；仅 `docker` 可用且 **daemon 可达**（`docker ps` 正常）。

**网络**：`static.rust-lang.org` HTTP 200（rustup 可恢复）；内存 252GB / 可用 241GB（无 8GB 限制）。

**各平台可行性**：

| 平台 | 本机可行 | 需要什么 |
|---|---|---|
| x86_64-unknown-linux-gnu | **已完成** | — |
| x86_64/aarch64-unknown-linux-musl | 待授权 | rustup + musl-gcc，或 docker `messense/rust-musl-cross` |
| aarch64-unknown-linux-gnu | 待授权 | rustup + `gcc-aarch64-linux-gnu` |
| x86_64-pc-windows-gnu | 待授权 | rustup + `mingw-w64` |
| **aarch64/x86_64-apple-darwin** | **不可行** | 需 macOS SDK；`build.yml` 的 macOS 产物跑在 `macos-latest` 原生 runner 上 |
| **x86_64/aarch64-pc-windows-msvc** | **不可行** | 需 MSVC；`build.yml` 跑在 `windows-latest` 上 |

**结论**：本机最多补齐 4 个 Linux/Windows-gnu 目标；**macOS 与 Windows-msvc 产物只能由 CI 产出**。

## 待用户裁决 / 授权

| 项 | 说明 |
|---|---|
| 是否恢复 rustup 并安装交叉工具链 | 系统级改动，编排者**未执行**，需你授权 |
| 走 apt 装工具链 还是 docker 镜像 | 两条路，见汇报 |
| 是否提交并推送本次改动 | 4 类改动待提交（README×2、jetbrains 重命名×5、agents×2、新看板） |

## 决策日志

| 时间 | 决策 | 依据 |
|---|---|---|
| 2026-09-03 | 字符画用脚本化替换而非手工 Edit | 艺术字对空白敏感 + 此前 Edit 越界事故 |
| 2026-09-03 | 新横幅第 4 行 `<` 转义为 `&lt;` | `<pre>` 是裸 HTML，字面 `<` 可能被解析为标签起始 |
| 2026-09-03 | 原生构建用 `-j 2` 而非 `-j 1` | 实测可用内存 241GB，AGENTS.md 记载的 8GB cgroup 限制在当前环境不成立 |
| 2026-09-03 | T3 派发 doc-writer 而非编排者代行 | 纯文档 token 替换，doc-writer 通道此前验证可用（对比 code-implementer 5 连败） |
| 2026-09-03 | 不擅自恢复 rustup / 装系统包 | 系统级改动超出编排者权限，须显式授权 |

---

# [IN PROGRESS] 多平台产物（用户已授权恢复 rustup + apt 装交叉工具链）

## 环境恢复（2026-09-03，已授权执行）

| 项 | 结果 |
|---|---|
| rustup | **已恢复** —— `rustup 1.29.1 (d95a37b6a 2026-08-13)`；`stable-x86_64-unknown-linux-gnu` 自动识别为 active/default（原有工具链未重装） |
| `cargo` 悬空链接 | **已修复** —— `/root/.cargo/bin/cargo` 现在能正常解析，`cargo 1.93.0` 可直接调用，**不再需要工具链绝对路径** |
| musl-tools | `musl-gcc` → `/usr/bin/musl-gcc` |
| gcc-aarch64-linux-gnu | `aarch64-linux-gnu-gcc` → `/usr/bin/aarch64-linux-gnu-gcc` |
| mingw-w64 | `x86_64-w64-mingw32-gcc` → `/usr/bin/x86_64-w64-mingw32-gcc` |
| `apt-get install` | `install_exit=0` |

**副作用（正面）**：此前所有构建/测试命令都必须走
`/root/.rustup/toolchains/.../bin/cargo` + PATH 注入；**该变通即日起不再需要**。
这条应回写到 `AGENTS.md` 的环境约束段（待用户决定是否改 `AGENTS.md`）。

## 已安装的编译目标

`rustup target list --installed`：
`x86_64-unknown-linux-gnu`（原有）、`x86_64-unknown-linux-musl`、`aarch64-unknown-linux-gnu`、
`aarch64-unknown-linux-musl`、`x86_64-pc-windows-gnu`。

## 构建结果（按可行性排序，逐个执行，失败不阻断后续）

| 目标 | 状态 | 产物 |
|---|---|---|
| `x86_64-unknown-linux-gnu` | **done** | `rustcode` 32,701,552 B；`rustcode-daemon` 27,304,112 B（ELF x86-64，dynamically linked） |
| `x86_64-unknown-linux-musl` | **done** | `rustcode` 32,851,064 B；`rustcode-daemon` 27,448,888 B（ELF x86-64，**static-pie linked**，完全静态） |
| `aarch64-unknown-linux-gnu` | **done**（首次失败，补装 `libc6-dev-arm64-cross` 后重试成功，2m20s） | `rustcode` 29,791,176 B；`rustcode-daemon` 25,654,680 B（**ELF ARM aarch64**） |
| `x86_64-pc-windows-gnu` | **done** | `rustcode.exe` 30,920,192 B；`rustcode-daemon.exe` 26,045,440 B（**PE32+ x86-64 for MS Windows**；daemon 为 GUI 子系统） |
| `aarch64-unknown-linux-musl` | **failed** | `failed to find tool "aarch64-linux-musl-gcc"` —— apt 无 aarch64 版 musl 交叉工具链，`musl-tools` 只提供 x86_64 的 `musl-gcc`。**未强推**，避免继续装外部工具链扩大系统改动面 |
| `*-apple-darwin` / `*-pc-windows-msvc` | **不可行** | 需 macOS SDK / MSVC，只能由 `build.yml` 在 `macos-latest` / `windows-latest` 原生 runner 产出 |

日志：`/tmp/multi_target.log`；分目标日志 `/tmp/relbuild/<target>.log`。
构建脚本 `/tmp/multi_target_build.sh`（`-j 4`；实测内存 252GB / 可用 241GB，无 8GB cgroup 限制）。

## 已推送的四次原子提交（2026-09-03）

| 提交 | sha | 内容 |
|---|---|---|
| C1 | `98616b0f` | `docs(readme)` README 字符画 → RustCode（2 files / +10 / -10） |
| C2 | `fac2cc11` | `fix(jetbrains)` 夹具重命名（5 files，R100 纯重命名，0 增 0 删） |
| C3 | `20e36caf` | `chore(agents)` 旧 crate 名清理（2 files / +3 / -3） |
| C4 | `3d8466c0` | `chore(board)` 本看板入库 |

推送：`2e5baa33..3d8466c0 dev -> dev`，远端 Hooks `[PASSED]`；工作区干净。

## [WARN] 本轮执行偏差

1. **C1 首次提交误带入已暂存的重命名**（`git commit` 不带路径会提交整个索引）。
   已用 `git reset --soft HEAD~1` 回退（**未动工作区**），改用路径限定提交 `git commit -- <paths>` 纠正。
2. **纠正过程中暴露又一个隐藏约束**：`extensions/jetbrains/.gitignore:7` 的 `bin/` 规则忽略了该目录，
   这些夹具在版本库里只是因为历史上被强制加入。回退暂存后新文件名变回「未跟踪且被忽略」，
   需用 `git add -fA` 才能重新暂存重命名（已验证 5 个均识别为 R100）。
3. **aarch64 首次构建失败**，根因 `bits/libc-header-start.h: No such file or directory`
   —— `gcc-aarch64-linux-gnu` 抓的是宿主机 `/usr/include` 而非 aarch64 sysroot。
   补装 `libc6-dev-arm64-cross`（提供 `/usr/aarch64-linux-gnu/include`）后重试成功。
   **教训**：装交叉编译器 ≠ 装齐交叉 libc 头文件，两者是不同包。

## 冒烟验证

| 命令 | 输出 |
|---|---|
| `./target/release/rustcode --version` | `rustcode 5.0.9 (2e5baa33+dirty)`（该产物建于提交前，故带 `+dirty`） |
| `./target/x86_64-unknown-linux-musl/release/rustcode --version` | `rustcode 5.0.9 (3d8466c0)`（提交后构建，干净） |
| `./target/release/rustcode-daemon` | `RustCode API server listening on http://127.0.0.1:13456`（daemon 正常拉起，后因空闲超时中止） |

aarch64 / Windows 产物**只做了 `file` 架构校验，未在本机执行**（架构不匹配，无法运行）——属已知未验证范围。

## T2：产物分析 + 运行测试（已完成，报告见 `03-impl/T2.md`）

### 静态分析结论（8 个产物）

- **musl 版为 `static-pie linked`，零外部依赖** —— 跨发行版可移植性最佳，Linux 分发首选
- 原生 gnu 依赖 `libgcc_s.so.1` / `libm.so.6` / `libc.so.6`；aarch64 依赖 aarch64 glibc
- Windows：CLI 为 **CUI 控制台子系统**，daemon 为 **GUI 子系统**（符合后台服务定位）
- 加固属性三者一致：**PIE + Full RELRO（BIND_NOW）+ NX**，符号已剥离（无 `.debug_*`、无 `.symtab`）
- Stack canary：gnu 产物检出引用（CLI 206 / daemon 86）；musl 与 aarch64 检出 0，
  但**产物已剥离符号，该计数不可靠，不作结论**

### 运行测试结果（原生 gnu + musl，均通过）

| 测试 | 原生 gnu | musl |
|---|---|---|
| `--version` / `--help` | exit=0 | exit=0 |
| **ACP stdio 端到端** `scripts/acp_smoke.py` | **SMOKE OK, exit=0** | **SMOKE OK, exit=0** |
| daemon 启动监听 13456 | 成功 | 成功 |
| `GET /health` | **200** + 完整 JSON | **200** + 完整 JSON |
| `GET /project` `/projects` | 401 | 401（**正确**：令牌鉴权 fail-closed） |
| `GET /`（根路径） | 404 | 404（**符合预期**：前端未构建，AGENTS.md 已记载） |

ACP 冒烟覆盖 v1 + v2 双协议全流程，含**失败路径**（`resume` 不存在 id → `Invalid params`）。
两版 `/health` 的 `binary_hash` 不同，证明是各自独立构建的可执行文件。

### 未执行项（诚实标注）

aarch64 与 Windows 产物**仅做静态分析，未运行**：本机无 `qemu-user-static` 与 `wine`，
**未擅自安装**（系统级改动，未授权）。
跳过：`scripts/test-headless.sh`（硬编码 `target/debug/rustcode`，本轮只有 release 产物）；
依赖 LLM provider 凭据的联网用例。

### [WARN] 构建溯源问题 —— 已处置

发现原生 gnu 产物版本串为 `rustcode 5.0.9 (2e5baa33+dirty)`：该产物构建于本轮四次提交**之前**，
与源码状态不同步。已在提交后**重建**（`Finished in 1m 20s`），版本串现为干净的
**`rustcode 5.0.9 (73efc258)`**。musl / aarch64 / Windows 产物本就在提交后构建，版本串干净。

### 观测到的行为（非缺陷，供参考）

`rustcode-daemon --help` **不打印帮助而是直接启动服务器**（与 `--version` 行为不同）。
该 daemon 不消费 `--help`，直接走默认启动路径。属 CLI 一致性小问题，未改动。

## [ERROR] 并发会话事件（2026-09-03 19:13）

作业期间检测到**另一个编排会话向同一 worktree 提交**：

- 提交 `73efc258 docs(consistency): fix rustcode-cli commands and stale rebrand facts`
  作者同为 `rustcode-builder <builder@rustcode.local>`，但**非本会话产出**
- 改动 12 文件（README × 2、`docs/` 若干、`site/index.html`、`scripts/linux-release-linux.sh`），
  内容为文档一致性修正（`-p rustcode-cli` → `-p rustcode`、旧品牌事实、`.atom` → `.rustcode` 等）
- **与我改动的 `README.md` / `README.zh-CN.md` 存在文件重叠**
- **已验证未破坏我的成果**：两个 README 横幅仍与 `pyfiglet('RustCode')` 逐字节一致，
  旧字形 `|_| |_| |_|` 无残留。双方改动互补且无冲突（其提交是我已推送提交的快进子节点）
- 该提交将随本看板提交一并推送（**仅快进，无 force**）

此现象与 `2026-09-02-g1-fmt-gate` 看板记载的并发会话冲突属同类，属环境事实。

## 需要回写的环境事实（待用户决定是否改 `AGENTS.md`）

`AGENTS.md` 记载「cgroup 内存上限 8GB，`cargo test` 必须 `-j 1`」。本轮实测：

- `free -m` 显示 **252GB 总量 / 241GB 可用**，未观察到 8GB cgroup 限制
- 因此 release 构建用 `-j 4` 全程正常；本轮全工作区测试是用 `-j 1` 跑的（沿用旧约束，偏保守但未出错）
- rustup 已恢复，`cargo` 直连可用，此前「工具链绝对路径 + PATH 注入」的变通**已不需要**

`AGENTS.md` 属受保护文件（GW-18 默认 blocked），是否更新需你单独授权。
