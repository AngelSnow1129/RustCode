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
