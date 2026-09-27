# 文件面安全改造设计（webui `/fs/*`）

日期: 2026-09-25
分支: dev
状态: **F1 已落地**；**F4（只读查看，限定工作目录）已落地**；下载与打包**按用户裁决推迟**（见 §0）。全部结论来自代码实测
上游需求: 在 webui 右侧增加文件管理面板（代码/文件预览、目录浏览、下载、文件夹打包）

---

## 0. 用户裁决（2026-09-25，**覆盖下方各节的开放建议**）

| 问题 | 裁决 | 影响 |
|------|------|------|
| 下载 / 打包 | **暂时不允许**。先只做"工作目录内查看" | F5（`/fs/download`）与 F6（`/fs/package`，含引入 `zip` 依赖）**推迟**；早先"引入 zip"的意向保留在案，但**不现在落地依赖** |
| 查看范围 | **只允许会话工作目录内** | 新增 `/fs/read` 一律走 `authorize_file_access`（canonicalize + 根约束 + 敏感路径），无"列工作目录外"的读取路径 |
| 越权访问 | **不得运行越权访问** | 免密（`webui_no_auth` / `enforce_token=false`）下内容类操作一律拒绝 |
| 根约束基准（Q4） | **会话工作目录** | 与 `resolve_session_workspace_file` 同源 |

**两处与既有文档行为的冲突，按本裁决处理**（上一轮提出的阻塞点）：

1. `/fs/open` 的既有注释明写"允许打开工作目录外、agent 写出的产物"。本裁决**不改 `/fs/open` 的可达范围**
   （它不返回内容，只调 GUI 打开器，且改动会打断既有产物回看），仅在读/写类端点上落实约束。
   即：**约束落在"产生内容"的地方，不落在"打开本机 GUI"的地方**。
2. `/fs/mkdir` 被 `CwdPicker`（浏览任意目录并新建文件夹）依赖。本裁决**不改其可达范围**，
   但**免密时拒绝**（写原语 + 免密常见于 `--host 0.0.0.0` 的同网部署）。

**残留风险（如实登记，非"已解决"）**：`/fs/list`、`/fs/search`、`/fs/mkdir`、`/fs/open`
在本轮后**仍可作用于工作目录之外**，因为它们服务的是"选项目目录"这一既有交互。
它们不返回文件内容（list/search 返回路径元信息，mkdir 只建目录，open 只调本机打开器），
故不是"内容外泄"面；但**"可列/可搜任意目录"本身仍是信息面**，其收束属后续独立评估，
不在本轮假装已解决。

---

## 1. 为什么先写这份设计

上一轮分析确认：文件面板本身不难（数据面 `turnArtifacts` 与 `/fs/list` 已有），
**难的是它要把 daemon 的文件系统访问面从"token 即全权"改造成可收束的边界**。
在默认 `0.0.0.0` + 明文 HTTP + 可选免密（`webui_no_auth`）的部署形态下，
新增"读内容 / 下载 / 打包"等于允许把整台机器的文件拖走。

因此本设计是**文件下载与打包的前置**，而不是面板的前置。
零新增后端的那一档面板（改动文件列表 + 目录树 + 复用聊天里已有的 `read_file` 输出）
不受本设计阻塞。

---

## 2. 现状实测（逐条来自代码，非声称）

### 2.1 四个 `/fs/*` 端点与其约束

| 端点 | 位置 | 现有约束 | 能力 | 越权面 |
|------|------|---------|------|--------|
| `GET /fs/list` | `lib.rs:6383` | `normalize_dir_arg`（`~` 展开）+ `canonicalize` + 去 Windows `\\?\` | 列任意目录的子目录与文件名 | **可列任意目录**（`?path=/etc`） |
| `GET /fs/search` | `lib.rs:6431` | 同上 | gitignore-aware 递归搜索（与 TUI `@` 同源） | **可搜任意目录** |
| `POST /fs/mkdir` | `lib.rs:6587` | 同上 | `create_dir_all` | **可在任意位置建目录**（写原语） |
| `POST /fs/open` | `lib.rs:6531` | `resolve_session_workspace_file` → `resolve_workspace_file` | 调系统 GUI 打开器，不回内容 | 可指向任意文件 |

上表四个端点**都在 `protected` router 内**（位于 `protected` 路由组，
并挂 `require_webui_token`），但保护强度取决于 `enforce_token`：

- `enforce_token=true`（进程内 webui / `rustcode webui`）：校验 Bearer 或 HttpOnly cookie。
- `enforce_token=false`（独立 daemon / VSCode 实例）：**中间件直接放行**（`auth_token.rs:123-126`）。
- `webui_no_auth=true`（用户显式 `--no-auth` / 配置 / env）：`enforce_token` 为 false，同上放行。

即：**免密模式下 `/fs/*` 完全无门禁**。

### 2.2 一处需要更正的历史说法

先前称"`/fs/open` 的路径解析锁在会话工作目录内"——**不准确**。
`resolve_workspace_file`（`lib.rs:6471`）实测：

```rust
let unresolved = if requested.is_absolute() { requested } else { root.join(requested) };
let target = std::fs::canonicalize(unresolved)?;
if !target.is_file() { /* reject dirs */ }
```

**没有 `target.starts_with(root)` 校验**：绝对路径直接采纳，相对路径的 `../` 也能逃逸。
危害有限（它不返回内容，只调 GUI 打开器），但"被限制在工作目录内"是错的。
新增"返回内容"的端点时**绝不能沿用这个解析函数**。

### 2.3 可复用的现成轮子

| 轮子 | 位置 | 当前用途 |
|------|------|---------|
| `path_is_sensitive` | `capabilities/src/tools/sensitive_path.rs:201` | 系统保护前缀（`/etc` `/usr` `/root` 等，含 `SYSTEM_PROTECTED_EXCEPTIONS`）+ 凭据目录（`.ssh` `.aws` `.gnupg`）+ 秘密文件名/扩展名 |
| `references_sensitive_path` | 同上 `:65` | 对原始 JSON args 做子串匹配（工具调用侧） |
| 边界惯用法 | `pathnorm.rs::path_within_root`（**F1 已落地**，统一 helper） | `canonicalize` 后 `target.starts_with(&root)`；`tools/mod.rs`、`open_file.rs`、`lsp_tool.rs` 已改用它 |
| 限流常量 | `tools/read.rs:17-36` | `MAX_IN_MEMORY_BYTES=64MiB`、`DEFAULT_READ_LIMIT=1500`、`MAX_READ_OUTPUT_BYTES=50KiB`、`MAX_LINE_LEN=2000` |
| 鉴权判据 | `AppState.enforce_token` / `webui_no_auth`（`lib.rs:614/620`） | 免密是用户主动放弃鉴权，仍需保留交互式权限 |

**注意**：`starts_with(&root)` 这个惯用法目前**没有统一 helper**，
在 `tools/mod.rs`、`open_file.rs`、`lsp_tool.rs`、`skills/render.rs` 各写各的。
本设计建议抽一个共享谓词（见 §4.1），避免第 N 份漂移。

### 2.4 部署形态（威胁模型的前提）

- webui 默认绑 `0.0.0.0`（AGENTS.md「WebUI 默认绑定地址」节），**仅 token 保护、无 TLS**。
- 入站方向**无 TLS 能力**：`Cargo.lock` 中 `axum-server` 0 命中；
  daemon 的 rustls/reqwest 仅服务**出站**（`egress/client.rs` 的 `build_http_client`）。
- 配置文件 `~/.rustcode/config.toml` 含 provider `api_key`；用户主目录含 SSH 私钥。
  这两类正是"任意文件读取"最想拿的目标。

---

## 3. 威胁模型

| 编号 | 威胁 | 现状可利用性 | 改造后目标 |
|------|------|-------------|-----------|
| T1 | 免密模式（或 `enforce_token=false`）下匿名调用 `/fs/*` | **是** | 文件**内容/下载/打包**类端点在免密时 403 |
| T2 | 路径逃逸（`../`、绝对路径）读到工作目录外 | **是**（`/fs/open` 无前缀校验；读端点若沿用即继承） | canonicalize 后前缀校验，逃逸一律拒绝 |
| T3 | 符号链接把"工作目录内的文件"指向 `/etc/passwd`、私钥 | **是**（canonicalize 已解析，但无前缀校验） | 前缀校验在 canonicalize **之后**做（顺序是硬要求） |
| T4 | 读/下载敏感路径（`.ssh`、`.aws`、`config.toml`） | **是**（即使在工作目录内也可能存在） | 叠 `path_is_sensitive` |
| T5 | 大文件/二进制打爆内存或拖垮 daemon | **是**（`/fs/list` 连 size 都不返回） | 显式大小上限 + 二进制拒绝 + `/fs/list` 补元数据 |
| T6 | 明文 HTTP 下 token 被嗅探 | **是**（无入站 TLS） | 见 §5，本设计不自建 TLS |
| T7 | `/fs/mkdir` 在任意位置写 | **是** | 收束到工作目录内（或至少叠敏感路径拒绝） |

---

## 4. 改造设计

### 4.1 新增共享边界谓词（单一事实源）

在 `capabilities` 侧新增（避免 daemon 与工具各写一份）：

```rust
/// canonicalize 之后判定 `target` 是否位于 `root` 之内。
/// 顺序是硬要求：必须先 canonicalize 再判前缀，否则符号链接可绕过（T3）。
pub fn path_within_root(root: &Path, target: &Path) -> bool
```

- 内部对两者都 canonicalize；任一步失败即返回 `false`（fail-closed）。
- Windows 上先 `strip_verbatim_path`（`\\?\` 前缀会让 `starts_with` 误判）。
- 替换 `tools/mod.rs:314`、`open_file.rs:375`、`lsp_tool.rs:118` 的散落实现。

### 4.2 文件面统一判据（单一谓词，禁止调用点各写 `if`）

借鉴本仓既有模式（`fallback_eligible*`、凭据闸门 `bypass_mode` 共享 cell 的反面教训：
判据必须集中，否则两套真相）：

```rust
pub enum FileDeny { NoAuthMode, EscapesRoot, Sensitive, TooLarge, Binary, NotRegularFile }

/// 唯一的准入判据。返回 Ok(解析后的绝对路径) 或 Err(拒绝原因)。
pub fn authorize_file_access(
    root: &Path,
    requested: &str,
    op: FileOp,          // List | Read | Download | Mkdir | Open
    policy: &FilePolicy, // 免密态 + 大小上限
) -> Result<PathBuf, FileDeny>
```

判定顺序（刻意固定，注释写明为何）：
1. 免密 + 内容类操作（Read/Download/Package）→ `NoAuthMode`（403）。
   List/Mkdir 是否也禁，见 §4.5 的开放项 Q1。
2. canonicalize → 失败即拒。
3. `path_within_root` → 否即 `EscapesRoot`（T2/T3）。
4. `path_is_sensitive` → 是即 `Sensitive`（T4）。
5. 按 op 追加：`Read`/`Download` 校验普通文件 + 大小上限 + 二进制探测（T5）。

### 4.3 `/fs/list` 返回体补元数据（T5 与前端所需）

现返回 `{path, dirs, files}`（`files` 只是文件名字符串数组）。
前端做文件树时无法判断图标、二进制、大文件。扩为对象数组：

```json
{ "path": "...", "dirs": ["..."],
  "files": [{ "name": "a.rs", "size": 1234, "is_binary": false, "truncated": false }] }
```

- `size` 来自 `metadata()`；`is_binary` 用前若干字节的 NUL/非 UTF-8 探测
  （与 `tools/read.rs` 既有探测保持一致口径，勿新造一套）。
- 单个文件超 `MAX_PREVIEW_BYTES` 时 `truncated=true`，前端据此提示"过大，仅可下载"。

### 4.4 端点级改造清单

| 端点 | 改造 |
|------|------|
| `/fs/list` | 根约束到工作目录（或保留任意列目录但叠敏感路径拒绝，见 Q1）+ 补元数据 |
| `/fs/search` | 同上根约束；搜索结果过滤敏感路径 |
| `/fs/mkdir` | **必须**根约束（写原语，T7 危害最高） |
| `/fs/open` | 改用 `authorize_file_access`（修掉 §2.2 的缺失前缀校验） |
| `/fs/read`（新） | 全部判据 + 大小上限 + 二进制拒绝；**只服务文本预览** |
| `/fs/download`（新） | 同 `/fs/read`，加 `Content-Disposition: attachment` |
| `/fs/package`（新，最贵） | 见 §4.6 |

### 4.5 免密模式策略（开放项 Q1，需裁决）

`/fs/list` 与 `/fs/mkdir` 当前被**文件选择器与 CwdPicker** 依赖，
且它们本就允许浏览工作目录之外的目录（选项目要用）。
若一刀切在免密时禁用，会打断这些既有交互。

建议（待裁决）：
- 内容类（Read/Download/Package）：免密时**一律 403**（危害不可逆）。
- 元数据类（List/Search）：免密时**保留**但叠敏感路径拒绝 + 根约束。
- 写类（Mkdir）：免密时 403（免密常见部署是 `--host 0.0.0.0` 给同网设备）。

### 4.6 文件夹打包（另议，成本与暴露面最大）

- `zip` crate **不在依赖树**（`Cargo.lock` 0 命中）；
  `tar` 仅存在于 build-dependencies 与可选 `setup` feature（解包内嵌 seeds），不可复用。
  引入 zip 与 fork「最小化依赖」倾向冲突，需用户裁决。
- 若要做，额外硬约束（在 §4.2 之外）：
  - 条目数上限 + 总字节上限（防 zip bomb）；
  - 打包时逐条目跑一遍完整判据（不能只校验根目录，软链可在深层逃逸）；
  - 流式写出，不把整个归档读进内存；
  - 拒绝 `.git`、依赖目录（`node_modules`/`target`）等噪声目录（既有 gitignore 索引可复用）。

### 4.7 TLS（本设计不自建，见 §5）

---

## 5. TLS 方案：本设计**不**在 daemon 内自建入站 TLS

理由：
- 入站 TLS 需新增 `axum-server` + rustls 依赖与证书配置面，与 fork 最小化依赖倾向冲突；
- 自签证书的"信任"要靠私有 CA 或证书钉扎，二者都要新增配置面；
  而 `rustcode-config` 里的 `skip_tls_verify` 全是**出站 provider** 语义，**不可复用**于入站。

推荐路径（按优先级）：

| 方案 | 说明 | 代价 |
|------|------|------|
| A. 反向代理（caddy / nginx / Tailscale serve） | daemon 退到 `127.0.0.1`，由反代终止 TLS | 零代码，需运维配置 |
| B. `/tunnel` wss 接入 | 已支持 `wss://` 中继 | 加密止于中继，需信任中继方 |
| C. daemon 原生 TLS | 新增依赖 + 配置面 | 最重，仅在 A/B 不可行时考虑 |

**结论**：TLS 归 A/B，本设计只保证"即使明文链路被窃听，
文件面在没有有效 token 时也不产出内容"（T1/T2/T3/T4/T5）。

---

## 6. 分期与验收（严格串行）

| 期 | 内容 | 依赖 | 验收判据 |
|----|------|------|---------|
| **F1** ✅ | `path_within_root` 共享谓词 + `authorize_file_access` 单一判据（含单测，不接端点） | 无 | 谓词单测覆盖：`../` 逃逸、绝对路径、符号链接指向外部、canonicalize 失败、`\\?\` 前缀；既有散落实现改为复用 |
| **F2'** ✅ | `/fs/mkdir` **免密拒绝** + 敏感路径拒绝；`/fs/open` 敏感路径拒绝（**不改两者可达范围**，见 §0 冲突裁定 1/2） | F1 | 免密 mkdir 被拒；工作目录内的 `.ssh` 等敏感路径被拒；CwdPicker 与产物回看不回归 |
| **F3** | `/fs/list` 根约束 + 补元数据（size/is_binary/truncated）+ 敏感路径过滤 | F1 | 返回体含元数据；敏感目录不出现在列表 |
| **F4** ✅ | `/fs/read`（文本预览）：**锁定会话工作目录** + 大小上限 + 二进制拒绝 + 免密 403 | F1 | 工作目录外/敏感/超大/二进制一律被拒；免密 403 |
| **F5** ⏸ | `/fs/download`（Content-Disposition） | F4 | **按用户裁决推迟**（§0）；落地前须先有文件名安全清洗 |
| **F6** ⏸ | `/fs/package`（zip） | F1–F5 + 引入 zip 依赖 | **按用户裁决推迟**（§0）；依赖与归档边界（条目/字节上限、逐条目判据）另行评估 |

每期结束门禁：`cargo fmt -p <crate>`、`cargo clippy -p ... --all-targets`（新增代码零告警）、
该 crate `--lib` 测试 + 末期一次 `cargo test -j 1 --workspace --no-fail-fast`。

---

## 7. 硬约束（来自 AGENTS.md，勿违反）

- 出站 HTTP 若新增，走 `egress` 单一工厂（本设计预期不涉及）。
- 新增路由 → **同轮同步 `AGENTS.md`**（强制项）。
- 本地化：错误文案走 `Msg` 三件套（en/zh parity 由编译器保证）；
  断言本地化输出的测试必须 `test_lock()`。
- **禁止**为了"统一默认值"把默认绑定改回 `127.0.0.1`（AGENTS.md 明确记载这是刻意保留的安全边界，
  但默认 `0.0.0.0` 是本设计的威胁前提）。
- 测试不得依赖真实 `~/.rustcode`（`#[ctor]` 隔离）。

---

## 8. 开放项（**已按 §0 裁决关闭**，保留原建议以备追溯）

| # | 问题 | 原建议 | 裁决结果 |
|---|------|--------|---------|
| Q1 | 免密模式下 List/Search/Mkdir 是否一并禁用 | List/Search 保留但收束；Mkdir 禁用 | **部分采纳**：Mkdir 免密禁用（F2' 已落地）；List/Search 免密仍可用（它们服务 CwdPicker，且不含内容） |
| Q2 | 是否引入 `zip` 依赖做打包 | 先不做 | **意向=引入**，但**推迟落地**（下载/打包暂不允许） |
| Q3 | TLS 走 A（反代）还是 B（tunnel wss） | A | **未答**（不阻塞本轮：本轮不新增内容外泄面） |
| Q4 | 根约束用"会话工作目录"还是"用户显式选择的根目录" | 会话工作目录 | **采纳**：会话工作目录（F4 已据此实现） |

---

## 9. 落地记录与下一步

### F1 已完成（2026-09-25）

`pathnorm::path_within_root`（无 feature 门控，供 `lsp`/`skills` 等不启用 `tools` 的 feature 复用）
+ `capabilities::fs_boundary` 的 `authorize_file_access`（`tools` feature）。单测覆盖 `../` 逃逸、
绝对路径、符号链接指向外部、canonicalize 失败、空路径、敏感路径、免密 fail-closed、超限、二进制、op 形态。
既有三处散落实现（`tools/mod.rs` 的 not-found 提示、`open_file.rs` 的 `OpenFileWorkspaceGate`、
`lsp_tool.rs`）已改为复用；`skills/render.rs::source_rank` 经核实是**排名启发式而非安全边界**，
刻意保持词法判定并加注释（设计原文把它列为第 4 处属误列，已就地更正）。

### F2' / F4 已完成（2026-09-25，按 §0 裁决）

- **F2'**：`/fs/mkdir` 免密 403 + 敏感路径拒绝（`authorize_directory_creation`，走最近已存在祖先
  canonicalize，故 `..` 按落点判定；`~` 必须先经 `normalize_dir_arg` 归一化再判，否则合法的
  `~/new-dir` 会被误拒）；`/fs/open` 加敏感路径拒绝（**刻意不查免密**——非内容/写操作）。
  两者**可达范围不变**，见 §0 的冲突裁定。
- **F4**：新增 `GET /fs/read`（文本预览）。锁定会话工作目录；`read_text_file` 一次解析一次读
  （避免校验与使用之间的 TOCTOU）；响应回显**相对**路径，不泄漏宿主绝对路径。
- i18n：新增 9 个 `DaemonApiFsDenied*` 变体（三件套 parity 由编译器保证）；daemon 边缘的
  `file_deny_message`/`file_deny_status` 是 `FileDeny` 的唯一映射点。

**门禁**：`cargo fmt --check` exit=0；clippy daemon/capabilities 均 0 警告；
daemon `--lib` **320/0**；capabilities `--lib` **868/0**（fs_boundary 专项 17/0、fs_read 专项 7/0）。

### 面板接线已完成（2026-09-25，零新增后端）

用户诉求（左=面包屑 / 中=回复区 / 右=文件浏览+预览；回复末的产物地址可点击预览）已落地：

- `webui/src/lib/filePanel.ts`（纯函数：归一化、工作区包含判定、面包屑、目录行、产物配对）
  + `filePanel.test.ts` 16 个单测。
- `webui/src/components/FilePanel.tsx`：本轮改动文件列表 + 目录树 + 面包屑 + 文本预览。
- `app.tsx` 三栏（`Sidebar | main-column | .file-panel`），面板默认关闭，窄屏（≤900px）覆盖式。
- `Chat.tsx` 新增 `onTurnArtifacts` 上报**最近一个**助手回合的产物（复用既有
  `artifactsByAssistantIndex`，不重新解析消息）。
- `api.ts` 新增 `readFile`（走 `GET /fs/read`）；i18n 新增 15 个 `filePanel.*` 双语键。
- 工作目录外的产物/V路径**照常列出但标记不可预览**，不隐藏（agent 可能确实写到了外面）。

**门禁**：`npx tsc --noEmit` 0 错；`npm test` **277/0**；`npm run build` 成功；
按铁律 `cargo clean -p rustcode-daemon` 后重编译通过（新 bundle 已嵌入）。

### 下一步（按序）

1. **F3**（可选）：`/fs/list` 补元数据（size/is_binary/truncated）+ 敏感路径过滤。前端要区分
   二进制/大文件就必须做这一步；当前前端只能靠 `/fs/read` 的报错反推二进制。
2. **F5/F6 保持推迟**（§0）：下载与打包在裁决解除前不实现；若要恢复，先补文件名安全清洗
   （F5）与归档边界（F6），再评估 zip 依赖。
3. 可选增强：`/fs/list` 的工作目录收束（当前仅 `/fs/read` 强制）——见 §0 残留风险。
