# MCP Server 推荐

MCP（Model Context Protocol）server 通过连接外部工具与服务来扩展 RustCode 的能力。

**说明**：以下是常见 MCP server。针对代码库所用的具体服务与集成，请用 Web 搜索查找对应的 MCP server。

## 配置与团队共享

**连接方式：**
1. **项目配置**（`.mcp.json`）—— 仅在该目录内可用
2. **全局配置**（`~/.rustcode/mcp.json`）—— 在所有项目中可用
3. **提交进仓库的 `.mcp.json`** —— 整个团队都可用（推荐！）

**提示**：把 `.mcp.json` 提交进 git，整个团队就能用上同一批 MCP server。

**调试**：用 `rustcode --mcp-debug` 定位配置问题。

## 文档与知识

### context7（实时文档查询）
**适用场景**：项目使用了流行库/SDK，你希望 RustCode 依据最新文档写代码

| 推荐时机 | 示例 |
|----------------|----------|
| 使用了 React、Vue、Angular | 前端框架 |
| 使用了 Express、FastAPI、Django | 后端框架 |
| 使用了 Prisma、Drizzle | ORM |
| 使用了 Stripe、Twilio、SendGrid | 第三方 API |
| 使用了 AWS SDK、Google Cloud | 云 SDK |
| 使用了 LangChain、OpenAI SDK | AI/ML 库 |

**价值**：RustCode 会抓取实时文档，而不是依赖训练数据，从而减少幻觉出来的 API 与过时写法。

---

## 浏览器与前端

### Playwright MCP（浏览器自动化）
**适用场景**：需要浏览器自动化、测试或截图的前端项目

| 推荐时机 | 示例 |
|----------------|----------|
| React/Vue/Angular 应用 | UI 组件测试 |
| 需要 E2E 测试 | 验证用户流程 |
| 视觉回归测试 | 截图对比 |
| 调试 UI 问题 | 看到用户所见 |
| 表单测试 | 多步骤工作流 |

**价值**：RustCode 可以与运行中的应用交互、截图、填写表单并验证 UI 行为。

### Puppeteer MCP（无头浏览器）
**适用场景**：无头浏览器自动化、网页抓取

| 推荐时机 | 示例 |
|----------------|----------|
| 由 HTML 生成 PDF | 生成报告 |
| 网页抓取任务 | 数据提取 |
| 无头测试 | CI 环境 |

---

## 数据库

### Supabase MCP（托管后端）
**适用场景**：用 Supabase 做后端/数据库的项目

| 推荐时机 | 示例 |
|----------------|----------|
| 检测到 Supabase 项目 | 依赖中有 `@supabase/supabase-js` |
| 需要认证 + 数据库 | 用户管理类应用 |
| 实时功能 | 数据实时同步 |

**价值**：RustCode 可以直接查询数据表、管理认证并操作 Supabase 存储。

### PostgreSQL MCP（数据库直连）
**适用场景**：直接访问 PostgreSQL 数据库

| 推荐时机 | 示例 |
|----------------|----------|
| 直接使用 PostgreSQL | 没有 ORM 层 |
| 数据库迁移 | 管理 schema |
| 数据分析任务 | 复杂查询 |
| 排查数据问题 | 查看真实数据 |

### Neon MCP（serverless Postgres 数据库）
**适用场景**：使用 Neon serverless Postgres 的用户

### Turso MCP（边缘数据库）
**适用场景**：使用 Turso/libSQL 边缘数据库的用户

---

## 版本控制与 DevOps

### GitHub MCP（仓库与协作）
**适用场景**：托管在 GitHub 上、需要 issue/PR 集成的仓库

| 推荐时机 | 示例 |
|----------------|----------|
| 仓库托管在 GitHub | `.git` 的远端指向 GitHub |
| 以 issue 驱动开发 | 在提交信息中引用 issue |
| PR 工作流 | 评审、合并操作 |
| 使用 GitHub Actions | 访问 CI/CD 流水线 |
| 发布管理 | 打标签与发布自动化 |

**价值**：RustCode 可以创建 issue、评审 PR、查看 workflow 运行状态并管理发布。

### GitLab MCP（GitLab 仓库）
**适用场景**：托管在 GitLab 上的仓库

### Linear MCP（Linear 工单）
**适用场景**：用 Linear 做 issue 追踪的团队

| 推荐时机 | 示例 |
|----------------|----------|
| 使用 Linear 工作区 | 形如 `ABC-123` 的 issue 引用 |
| 迭代规划 | 管理待办列表 |
| 从代码创建 issue | 为 TODO 自动建 issue |

---

## 云基础设施

### AWS MCP（AWS 基础设施管理）
**适用场景**：AWS 基础设施管理

| 推荐时机 | 示例 |
|----------------|----------|
| 依赖中含 AWS SDK | `@aws-sdk/*` 包 |
| 基础设施即代码 | Terraform、CDK、SAM |
| Lambda 开发 | 无服务器函数 |
| 使用 S3、DynamoDB | 云数据服务 |

### Cloudflare MCP（Workers、Pages、R2、D1 服务）
**适用场景**：Cloudflare Workers、Pages、R2、D1

| 推荐时机 | 示例 |
|----------------|----------|
| 使用 Cloudflare Workers | 边缘函数 |
| Pages 部署 | 静态站点托管 |
| R2 存储 | 对象存储 |
| D1 数据库 | 边缘 SQL 数据库 |

### Vercel MCP（部署与配置）
**适用场景**：Vercel 部署与配置

---

## 监控与可观测性

### Sentry MCP（错误追踪）
**适用场景**：错误追踪与调试

| 推荐时机 | 示例 |
|----------------|----------|
| 已配置 Sentry | 依赖中有 `@sentry/*` |
| 生产环境调试 | 排查错误 |
| 错误模式 | 归类相似问题 |
| 发布追踪 | 把部署与错误关联起来 |

**价值**：RustCode 可以排查 Sentry 问题、定位根因并给出修复建议。

### Datadog MCP（APM、日志与指标）
**适用场景**：APM、日志与指标

---

## 沟通协作

### Slack MCP（Slack 工作区集成）
**适用场景**：Slack 工作区集成

| 推荐时机 | 示例 |
|----------------|----------|
| 团队使用 Slack | 发送消息通知 |
| 部署通知 | 告警频道 |
| 故障响应 | 发布进展更新 |

### Notion MCP（文档工作区）
**适用场景**：用 Notion 工作区存放文档

| 推荐时机 | 示例 |
|----------------|----------|
| 用 Notion 写文档 | 读取/更新页面 |
| 知识库 | 检索文档 |
| 会议记录 | 生成摘要 |

---

## 文件与数据

### Filesystem MCP（增强文件操作）
**适用场景**：超出内置工具能力的进阶文件操作

| 推荐时机 | 示例 |
|----------------|----------|
| 复杂文件操作 | 批量处理 |
| 监听文件 | 监控变更 |
| 高级搜索 | 自定义模式 |

### Memory MCP（跨会话记忆）
**适用场景**：跨会话的持久化记忆

| 推荐时机 | 示例 |
|----------------|----------|
| 长期项目 | 记住上下文 |
| 用户偏好 | 保存设置 |
| 学习模式 | 沉淀知识 |

**价值**：RustCode 能在多次对话之间记住项目上下文、决策与模式。

---

## 容器与 DevOps

### Docker MCP（容器管理）
**适用场景**：容器管理

| 推荐时机 | 示例 |
|----------------|----------|
| 存在 Docker Compose 文件 | 容器编排 |
| 存在 Dockerfile | 构建镜像 |
| 容器调试 | 查看日志、exec 进入 |

### Kubernetes MCP（集群管理）
**适用场景**：Kubernetes 集群管理

| 推荐时机 | 示例 |
|----------------|----------|
| K8s manifest | 部署、扩缩 Pod |
| Helm chart | 包管理 |
| 集群调试 | Pod 日志与状态 |

---

## AI 与 ML

### Exa MCP（Web 搜索与调研）
**适用场景**：Web 搜索与资料调研

| 推荐时机 | 示例 |
|----------------|----------|
| 调研任务 | 查找最新信息 |
| 竞品分析 | 市场调研 |
| 文档缺失 | 查找示例 |

---

## 速查：检测依据 -> 推荐的 MCP server

| 检测依据 | 推荐的 MCP server |
|----------|-------------------|
| 使用了热门 npm 包 | context7 |
| React/Vue/Next.js 项目 | Playwright MCP |
| 依赖含 `@supabase/supabase-js` | Supabase MCP |
| 依赖含 `pg` 或 `postgres` | PostgreSQL MCP |
| 远端是 GitHub | GitHub MCP |
| 存在 `.linear` 或 Linear 引用 | Linear MCP |
| 依赖含 `@aws-sdk/*` | AWS MCP |
| 依赖含 `@sentry/*` | Sentry MCP |
| 存在 `docker-compose.yml` | Docker MCP |
| 存在 Slack webhook URL | Slack MCP |
| 依赖含 `@anthropic-ai/sdk` | 用 context7 查 Anthropic 文档 |
