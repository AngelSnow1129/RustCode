---
name: setup
description: 分析代码库并推荐 RustCode 自动化配置（hooks、subagents、skills、plugins、MCP servers）。当用户询问自动化推荐、希望优化 RustCode 配置、提到改进 RustCode 工作流、询问如何为一个项目首次配置 RustCode，或想知道该使用哪些 RustCode 功能时使用。
user_invocable: true
argument_hint: "[focus area, e.g. hooks, mcp, skills, all]"
allowed_tools: Read, Glob, Grep, Bash, Write, Edit
---

# RustCode 自动化推荐器

分析代码库模式，推荐量身定制的 RustCode 自动化配置，然后**询问是否需要安装**。

## 核心原则

1. **先推荐，按请求再安装** —— 先输出推荐，再问“要我帮你装上吗？”
2. **优先使用已配置的 registry，本地兜底** —— 如果你的发行版提供了社区 skill registry CLI（例如社区的 `npx skills` 客户端），就先搜索其中的最新社区 skill；未配置此类 CLI 时回退到内置的 reference 文件
3. **友好的错误提示** —— 如果缺少工具（没有 Node.js、没有 npx），要清楚说明情况，并继续给出本地推荐
4. **每个类别 1-2 项** —— 不要堆砌，只呈现最有价值的自动化配置

## 自动化类型

| 类型 | 适用场景 | 安装方式 |
|------|----------|----------------|
| **Skills** | 打包好的经验、工作流、可重复任务 | 你的 skill registry CLI（例如 `npx skills add <pkg>`，前提是你的发行版配置了它），或创建 `.rustcode/skills/<name>/SKILL.md` |
| **Plugins** | skill、命令、agent、hook 的集合 | `/plugin marketplace add <url>` 然后 `/plugin install <name>` |
| **MCP Servers** | 外部工具集成（数据库、API、文档） | 写入 `.mcp.json`（项目根目录）或 `~/.rustcode/mcp.json`（全局） |
| **Hooks** | 工具事件触发的自动动作（格式化、lint、拦截） | 写入 `.rustcode/settings.json` |
| **Subagents** | 专门的审查者（安全、性能、无障碍） | 创建 `.rustcode/skills/<name>/SKILL.md`，内容为审查提示词 |
| **Commands** | 快捷 slash 命令（/test、/review、/deploy） | 创建 `.rustcode/commands/<name>.md` |

## 工作流

### 阶段 1：代码库分析

收集项目上下文：

```bash
# Detect project type and tools
ls -la package.json pyproject.toml Cargo.toml go.mod pom.xml build.gradle 2>/dev/null
cat package.json 2>/dev/null | head -50

# Check dependencies
cat package.json 2>/dev/null | grep -E '"(react|vue|angular|next|express|fastapi|django|prisma|supabase|stripe)"'

# Check existing RustCode config
ls -la .rustcode/ CLAUDE.md .mcp.json 2>/dev/null

# Project structure
ls -la src/ app/ lib/ tests/ components/ pages/ api/ 2>/dev/null
```

**关键指示信号：**

| 类别 | 关注什么 | 影响哪类推荐 |
|----------|------------------|---------|
| 语言/框架 | package.json、Cargo.toml、pyproject.toml | Skills、Hooks |
| 前端技术栈 | React、Vue、Angular、Next.js | Playwright MCP、前端类 skill |
| 后端技术栈 | Express、FastAPI、Django | API 文档工具 |
| 数据库 | Prisma、Supabase、裸 SQL | 数据库类 MCP server |
| 外部 API | Stripe、OpenAI、AWS SDK | context7 查文档 |
| 测试 | Jest、pytest、Playwright 配置 | 测试类 hooks/skills |
| 存在 CI/CD 配置 | GitHub Actions、GitLab CI | VCS MCP server |
| 文档模式 | OpenAPI、JSDoc、docstring | 文档类 skill |

### 阶段 2：搜索 Skills（在线 + 本地）

**按推荐类型的搜索策略：**

| 类型 | 在线 registry（仅当配置了 registry CLI） | 本地 reference 文件 |
|------|--------------------------|----------------------|
| **Skills** | [+] 若已配置 registry CLI，**先搜在线** — 社区生态有数千个 skill | [+] 补充自定义 skill 创建建议 |
| **MCP Servers** | [-] 在线 registry 不提供 MCP | [+] **只用本地** reference |
| **Hooks** | [-] 在线 registry 不提供 hooks | [+] **只用本地** reference |
| **Subagents** | [-] 在线 registry 不提供 agents | [+] **只用本地** reference |
| **Commands** | [-] 在线 registry 不提供 commands | [+] **只用本地** reference |

#### 步骤 2a：在线 skill 搜索（可选的社区 registry）

本步骤仅当用户的发行版/安装途径提供了社区 skill registry CLI 时才适用。
下面的示例使用社区的 `npx skills` 客户端；如果你的发行版提供的是其他 registry CLI
（或根本没有），就改用那一个，或直接跳到步骤 2b。

```bash
# Check if a registry CLI is available
npx --version 2>/dev/null
```

**如果 registry CLI 可用**，根据检测到的项目类型搜索相关 skill：

```bash
npx skills find <detected-language>
npx skills find <detected-framework>
npx skills find <specific-library>
```

示例：
- Rust 项目 → `npx skills find rust`
- React + Next.js 项目 → `npx skills find react nextjs`
- Python + Django 项目 → `npx skills find django`
- 用到 Docker → `npx skills find docker`

把最相关的结果（按安装量）写进你的 Skills 推荐。

**如果 npx 不可用**（未安装 Node.js），显示下面这段友好提示并继续：

```
[*] 提示：安装 Node.js 后可以使用在线 skill 搜索功能，获取社区最新推荐。
   下载：https://nodejs.org/
   目前使用内置推荐列表为您分析，功能不受影响。
```

#### 步骤 2b：本地 reference 查询（总会执行）

对**所有推荐类型**（MCP / Hooks / Subagents / Commands，以及 skill 的补充建议），都使用本地 reference 文件。无论有没有 Node.js，它们始终可用：

### 阶段 3：生成推荐

把在线搜索结果与本地 reference 知识结合起来生成推荐。

#### A. Skills 推荐

**来自在线 registry**（若可用）：
- 纳入通过 `npx skills find` 找到、安装量较高的 skill
- 给出确切的安装命令：`npx skills add <owner/repo@skill> -g -y`

**来自 reference 文件**（始终可用）：
内置模式见 [references/skills-reference.md](references/skills-reference.md)。

**建议安装的 plugin skill：**

| 代码库信号 | Skill | 安装命令 | 调用方式 |
|-----------------|-------|-----------------|------------|
| 任何项目（RustCode 使用问答） | **/guide** | 内置 —— 无需安装。直接运行 `/guide <question>`（也会自动派发）。 | 两者皆可 |

> 你的发行渠道可能提供 plugin marketplace（通过配置/环境变量设置其 URL，例如 `/plugin marketplace add https://example.com/<your-org>/rustcode-plugins`），其中提供 `rustcode-workflows`、`commit-craft`、`git-worktree` 等工作流 plugin；请从你所用发行版的 plugin 索引安装。RustCode 使用问答现在是内置的 `/guide` subagent，因此不再需要安装 plugin。

**建议创建的自定义 skill：**

| 代码库信号 | 要创建的 Skill | 调用方式 |
|-----------------|-----------------|------------|
| API 路由 | **api-doc**（OpenAPI 模板） | 两者皆可 |
| 数据库项目 | **create-migration** | 仅用户 |
| 测试套件 | **gen-test**（示例测试） | 仅用户 |
| 组件库 | **new-component**（模板） | 仅用户 |
| PR 工作流 | **pr-check**（检查清单） | 仅用户 |
| 代码风格 | **project-conventions** | 仅 RustCode |

#### B. MCP Server 推荐

详细模式见 [references/mcp-servers.md](references/mcp-servers.md)。

| 代码库信号 | 推荐的 MCP Server |
|-----------------|------------------------|
| 使用了流行库 | **context7** - 实时文档查询 |
| 前端且需要 UI 测试 | **Playwright** - 浏览器自动化 |
| 使用了 Supabase | **Supabase MCP** - 数据库操作 |
| PostgreSQL/MySQL | **Database MCP** - 查询与 schema |
| GitHub/GitLab 仓库 | **VCS MCP** - Issue、PR/MR |
| Docker 容器 | **Docker MCP** - 容器管理 |

#### C. Hooks 推荐

配置写法见 [references/hooks-patterns.md](references/hooks-patterns.md)。

| 代码库信号 | 推荐的 Hook |
|-----------------|------------------|
| 配置了 Prettier | PostToolUse：编辑后自动格式化 |
| 配置了 ESLint/Ruff | PostToolUse：编辑后自动 lint |
| TypeScript 项目 | PostToolUse：编辑后做类型检查 |
| 存在测试目录 | PostToolUse：运行相关测试 |
| 存在 `.env` 文件 | PreToolUse：拦截 `.env` 编辑 |
| 存在 lock 文件 | PreToolUse：拦截 lock 文件编辑 |

#### D. Subagent 推荐

模板见 [references/subagent-templates.md](references/subagent-templates.md)。

在 RustCode 中，subagent 以带有专门审查提示词的 skill 形式实现：

| 代码库信号 | 推荐的 Subagent |
|-----------------|---------------------|
| 大型代码库（>500 个文件） | **code-reviewer** |
| 认证/支付代码 | **security-reviewer** |
| API 项目 | **api-documenter** |
| 性能敏感 | **performance-analyzer** |
| 前端偏重 | **ui-reviewer**（无障碍） |

### 阶段 4：输出报告 + 询问是否安装

把推荐清晰地排版输出，然后**询问是否需要安装**。

```markdown
## RustCode Automation Recommendations

### Codebase Profile
- **Type**: [detected language/runtime]
- **Framework**: [detected framework]
- **Key Libraries**: [relevant libraries]

---

### Skills

#### [skill name]
**Why**: [specific reason]
**Install**: `npx skills add <owner/repo@skill> -g -y`
**Source**: [online registry / custom creation]

---

### MCP Servers

#### [server name]
**Why**: [specific reason]
**Config**:
```json
{
  "mcpServers": {
    "[name]": {
      "command": "npx",
      "args": ["-y", "@package/name"]
    }
  }
}
```

---

### [*] Hooks

#### [hook name]
**Why**: [specific reason]
**Config**: Add to `.rustcode/settings.json`

---

### Subagents

#### [agent name]
**Why**: [specific reason]
**Create**: `.rustcode/skills/[name]/SKILL.md`

---

**要我帮你安装这些推荐吗？** 你可以说：
- "全部装上" — 安装所有推荐
- "装 skills" — 只装推荐的 skills
- "装 [具体名称]" — 装指定项
- "先不装" — 只看推荐，稍后自行安装
```

### 阶段 5：执行安装（用户同意后）

用户同意安装后，按类型执行相应动作：

**Skills（来自在线 registry，前提是你的发行版配置了 registry CLI）：**
```bash
# example using the community `skills` CLI; use your distribution's registry CLI if different
npx skills add <owner/repo@skill> -g -y
```

如果 registry 命令失败，显示：
```
[!] 安装失败。可能原因：
1. 未安装 registry CLI（如 Node.js/npx）— 参考你的发行版文档
2. 网络问题 — 检查网络连接
3. 包名错误 — 在你所用 skill registry 的页面搜索确认

你也可以手动安装：
1. 在你所用 skill registry 的页面查找该 skill
2. 复制 SKILL.md 内容
3. 创建 ~/.rustcode/skills/[name]/SKILL.md
```

**Skills（自定义创建）：**
用 Write 工具按推荐内容创建 `.rustcode/skills/<name>/SKILL.md`。

**MCP Servers 的安装：**
用 Write/Edit 工具把 server 配置写入项目根目录的 `.mcp.json`（**不要**写 `.rustcode/mcp.json` —— 加载器不读它），或写入全局的 `~/.rustcode/mcp.json`：
```bash
# Read existing config
cat .mcp.json 2>/dev/null || echo '{}'
# Merge new server config
```

**Hooks 的安装：**
用 Write/Edit 工具把 hook 配置写入 `.rustcode/settings.json`：
```bash
cat .rustcode/settings.json 2>/dev/null || echo '{}'
```

**Subagents（以 skill 形式）：**
用 Write 工具按 subagent 模板中的审查提示词创建 `.rustcode/skills/<name>/SKILL.md`。

**Commands 的创建：**
用 Write 工具创建 `.rustcode/commands/<name>.md`。

**安装完成后**，向用户确认：
```
[SUCCESS] 安装完成！

已安装：
  [+] skill: [name] — [source]
  [+] mcp: [name] — 写入 .mcp.json（重启 RustCode 后生效）
  [+] hook: [name] — 写入 .rustcode/settings.json
  [+] subagent: [name] — 创建 .rustcode/skills/[name]/SKILL.md

[*] MCP 服务需要重启 RustCode 后生效。
[*] 输入 /help 查看新增的 slash commands。
```

## 决策框架

### 何时推荐 MCP Servers
- 需要集成外部服务（数据库、API）
- 需要查询库/SDK 的文档
- 需要浏览器自动化或测试
- 需要接入团队工具（GitHub、GitLab、Linear、Slack）

### 何时推荐 Skills
- 频繁重复的提示词或工作流
- 带参数的项目专属任务
- 需要把模板或脚本套用到任务上
- 用 `/skill-name` 调用的快捷动作

**调用控制：**
- `disable_model_invocation: true` —— 仅用户（用于有副作用的动作：deploy、commit）
- `user_invocable: false` —— 仅 RustCode（用于背景知识）
- 默认 —— 两者皆可调用

### 何时推荐 Hooks
- 编辑后重复执行的动作（格式化、lint）
- 保护规则（拦截敏感文件编辑）
- 校验检查（测试、类型检查）

### 何时推荐 Subagents
- 需要专门能力（安全、性能）
- 审查工作流
- 后台质量检查

## 错误处理

### Node.js / npx 不可用
```
[*] 提示：当前环境未安装 Node.js，在线 skill 搜索功能不可用。
   安装 Node.js 后可使用 `npx skills find` 搜索社区 skill。
   下载：https://nodejs.org/
   
   目前使用内置推荐列表为您分析，功能不受影响。
```

### 在线 skill 搜索无结果
```
[i] 在线 skill 库中暂无 [keyword] 相关的社区 skill。
   已使用内置推荐列表为您分析。
   你也可以稍后在你所用 skill registry 的页面浏览所有可用 skill。
```

### 在线 skill 安装失败
```
[!] skill 安装失败：[error message]

可能原因：
1. 网络连接问题 — 检查代理设置
2. 权限不足 — 检查目录权限与属主（切勿用 sudo 写入 ~/.rustcode，root 属主文件会导致后续运行初始化失败）
3. 包已下架 — 在你所用 skill registry 的页面确认包是否可用

替代方案：手动创建 skill 文件
  mkdir -p ~/.rustcode/skills/[name]
  # 然后将 SKILL.md 内容粘贴进去
```

### .rustcode 目录不可写
```
[!] 无法写入 .rustcode/ 目录。

请检查目录权限：
  ls -la .rustcode/
  
或手动创建：
  mkdir -p .rustcode/skills .rustcode/commands
```

## 配置提示

### MCP Server 配置
- **团队共享**：把 `.mcp.json` 提交进仓库，整个团队就都能用上同一批 MCP server
- **调试**：用 `--mcp-debug` 参数定位配置问题

### Hooks 的权限
在 `.rustcode/settings.json` 中配置允许的工具：
```json
{
  "permissions": {
    "allow": ["Edit", "Write", "Bash(npm test:*)", "Bash(git commit:*)"]
  }
}
```
