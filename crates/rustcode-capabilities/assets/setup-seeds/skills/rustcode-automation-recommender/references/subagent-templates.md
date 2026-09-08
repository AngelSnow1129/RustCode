# Subagent 推荐

Subagent 是并行运行的专职 RustCode 实例，各自拥有独立的上下文窗口与工具权限。非常适合聚焦式的评审、分析与生成任务。

**说明**：以下是常见模式。请根据代码库具体的评审与分析需求设计自定义 subagent。

## 代码评审类 Agent

### code-reviewer（代码评审）
**适用场景**：在大型代码库上做自动化代码质量检查

| 推荐时机 | 检测依据 |
|----------------|-----------|
| 大型代码库（超过 500 个文件） | 统计文件数 |
| 代码频繁变更 | 处于活跃开发中 |
| 团队希望评审口径一致 | 关注质量 |

**价值**：在你继续工作的同时并行完成代码评审
**模型**：sonnet（质量与速度均衡）
**工具**：Read, Grep, Glob, Bash

---

### security-reviewer（安全评审）
**适用场景**：以安全为重点的代码评审

| 推荐时机 | 检测依据 |
|----------------|-----------|
| 存在认证相关代码 | `auth/`、`login`、`session` 等特征 |
| 涉及支付处理 | `stripe`、`payment`、`billing` 等特征 |
| 处理用户数据 | `user`、`profile`、`pii` 等特征 |
| 代码中出现 API key | 环境变量特征 |

**价值**：发现 OWASP 漏洞、认证问题与数据泄露风险
**模型**：sonnet
**工具**：Read, Grep, Glob（为安全起见仅只读）

---

### test-writer（编写测试）
**适用场景**：生成覆盖全面的测试

| 推荐时机 | 检测依据 |
|----------------|-----------|
| 测试覆盖率低 | 测试文件数远少于源文件 |
| 已有测试套件 | 存在 `tests/`、`__tests__/` |
| 已配置测试框架 | 依赖中有 jest、pytest、vitest |

**价值**：生成符合项目约定的测试
**模型**：sonnet
**工具**：Read, Write, Grep, Glob

---

## 专项 Agent

### api-documenter（API 文档）
**适用场景**：生成 API 文档

| 推荐时机 | 检测依据 |
|----------------|-----------|
| REST 接口 | Express 路由、FastAPI 路径 |
| GraphQL schema | `.graphql` 文件 |
| 已有 OpenAPI | `openapi.yaml`、`swagger.json` |
| API 缺少文档 | 路由没有对应文档 |

**价值**：生成 OpenAPI 规范与接口文档
**模型**：sonnet
**工具**：Read, Write, Grep, Glob

---

### performance-analyzer（性能分析）
**适用场景**：定位性能瓶颈

| 推荐时机 | 检测依据 |
|----------------|-----------|
| 数据库查询 | 使用 ORM 或裸 SQL |
| 高流量代码 | API 接口、热点路径 |
| 有性能投诉 | 用户反馈卡顿 |
| 复杂算法 | 嵌套循环、递归 |

**价值**：发现 N+1 查询、O(n^2) 算法与内存泄漏
**模型**：sonnet
**工具**：Read, Grep, Glob, Bash

---

### ui-reviewer（UI 评审）
**适用场景**：前端可访问性与 UX 评审

| 推荐时机 | 检测依据 |
|----------------|-----------|
| React/Vue/Angular | 检测到前端框架 |
| 组件库 | `components/` 目录 |
| 面向用户的 UI | 不只是 API 项目 |

**价值**：发现可访问性问题、UX 问题与响应式设计缺陷
**模型**：sonnet
**工具**：Read, Grep, Glob

---

## 工具类 Agent

### dependency-updater（依赖升级）
**适用场景**：安全地升级依赖

| 推荐时机 | 检测依据 |
|----------------|-----------|
| 依赖已过时 | `npm outdated` 有输出 |
| 安全公告 | `npm audit` 有告警 |
| 落后一个大版本 | 版本差距明显 |

**价值**：边测试边增量升级依赖
**模型**：sonnet
**工具**：Read, Write, Bash, Grep

---

### migration-helper（迁移辅助）
**适用场景**：框架或版本迁移

| 推荐时机 | 检测依据 |
|----------------|-----------|
| 需要大版本升级 | 框架版本过旧 |
| 即将出现破坏性变更 | 存在弃用告警 |
| 计划重构 | 架构调整 |

**价值**：增量规划并执行迁移
**模型**：opus（需要复杂推理）
**工具**：Read, Write, Grep, Glob, Bash

---

## 速查：检测依据 -> 推荐

| 如果看到 | 推荐 Subagent |
|------------|-------------------|
| 大型代码库 | code-reviewer |
| 认证/支付相关代码 | security-reviewer |
| 测试很少 | test-writer |
| API 路由 | api-documenter |
| 大量数据库操作 | performance-analyzer |
| 前端组件 | ui-reviewer |
| 依赖包过时 | dependency-updater |
| 框架版本过旧 | migration-helper |

---

## Subagent 的存放位置

Subagent 放在 `.rustcode/agents/` 中：

```
.rustcode/
  agents/
    code-reviewer.md
    security-reviewer.md
    test-writer.md
```

---

## 模型选择指南

| 模型 | 适用场景 | 取舍 |
|-------|----------|-----------|
| **haiku** | 简单、重复的检查 | 快且便宜，但不够深入 |
| **sonnet** | 大多数评审/分析任务 | 均衡（推荐默认） |
| **opus** | 复杂迁移、架构设计 | 深入，但更慢、更贵 |

---

## 工具权限指南

| 权限级别 | 工具 | 使用场景 |
|--------------|-------|----------|
| 只读 | Read, Grep, Glob | 评审、分析 |
| 可写 | + Write | 代码生成、文档 |
| 完全 | + Bash | 迁移、测试 |
