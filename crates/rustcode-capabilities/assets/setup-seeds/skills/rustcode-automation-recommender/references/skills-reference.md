# Skills 推荐

Skills 是把工作流、参考资料与最佳实践打包在一起的专家经验。在 `.rustcode/skills/<name>/SKILL.md` 中创建。RustCode 会在相关时自动调用，用户也可以直接用 `/skill-name` 调用。

部分预置 skills 可通过安装器（`rustcode setup`）获取。

**说明**：以下是常见模式。请用 Web 搜索查找与代码库所用工具/框架匹配的 skill。

---

## 安装器内置的 skills

### 内置 Skills（通过 `rustcode setup` 安装）

| Skill | 适用场景 |
|-------|----------|
| **rust-best-practices** | 符合惯例的 Rust 开发 |
| **vue-best-practices** | Vue 3 Composition API 用法 |
| **react-native-best-practices** | React Native 性能 |
| **spring-boot-engineer** | Spring Boot 3.x 应用 |
| **code-review-excellence** | 代码评审实践 |
| **kubernetes** | K8s 运维 |
| **docker** | 容器化 |
| **api-design-patterns** | REST/GraphQL API 设计 |
| **sql-optimization-patterns** | 查询优化 |
| **frontend-testing** | Vitest + RTL 测试 |

### 内置命令（通过 `rustcode setup` 安装）

| 命令 | 适用场景 |
|---------|----------|
| **/review** | 代码评审流程 |
| **/refactor** | 代码重构 |
| **/lint** | lint 流程 |
| **/clippy** | Rust clippy 检查 |
| **/cargo-test** | Rust 测试执行 |
| **/cargo-run** | Rust 构建与运行 |
| **/pytest** | Python 测试执行 |
| **/tsc-check** | TypeScript 类型检查 |
| **/changelog** | 生成 changelog |
| **/fixissue** | 修复 issue 的流程 |

---

## 插件 Skills

以插件形式分发的 skills —— 先添加你的分发渠道提供的插件 marketplace（通过配置/环境变量设置其 URL，或从你的分发渠道插件索引安装），再执行 `/plugin install <name>`。若渠道未提供具体地址，可用 `https://example.com/<your-org>/rustcode-plugins` 这样的占位 URL。

| Skill | 插件 | 安装方式 | 适用场景 |
|-------|--------|---------|----------|
| **rustcode-workflows** | rustcode-workflows | `/plugin marketplace add <your-channel-marketplace-url>` 后执行 `/plugin install rustcode-workflows@rustcode` | 规划/评审/调试/头脑风暴等工作流 skill |
| **commit-craft** | commit-craft |（同一 marketplace）`/plugin install commit-craft@rustcode` | 约定式提交信息、PR 描述、changelog |
| **git-worktree** | git-worktree |（同一 marketplace）`/plugin install git-worktree@rustcode` | `/worktree` —— 隔离的 worktree，用于并行工作 |

> **RustCode 用法与文档问答**（安装、配置、slash 命令、
> 排障等）现在由**内置的 `/guide`** subagent 提供 —— 直接运行
> `/guide <question>` 即可，无需安装插件。

---

## 自定义项目 Skills

在 `.rustcode/skills/<name>/SKILL.md` 中创建项目专属的 skill。

### Skill 目录结构

```
.rustcode/skills/
  my-skill/
    SKILL.md           # Main instructions (required)
    template.yaml      # Template to apply
    scripts/
      validate.sh    # Script to run
    examples/          # Reference examples
```

### Frontmatter 参考

```yaml
---
name: skill-name
description: What this skill does and when to use it
disable-model-invocation: true  # Only user can invoke (for side effects)
user-invocable: false           # Only RustCode can invoke (for background knowledge)
allowed-tools: Read, Grep, Glob # Restrict tool access
---
```

### 调用控制

| 设置 | 用户 | RustCode | 用途 |
|---------|------|----------|---------|
|（默认）| 是 | 是 | 通用型 skill |
| `disable-model-invocation: true` | 是 | 否 | 有副作用的操作（部署、发送） |
| `user-invocable: false` | 否 | 是 | 背景知识 |

---

## 自定义 Skill 示例

### 用 OpenAPI 模板生成 API 文档

应用 YAML 模板生成风格一致的 API 文档：

```
.rustcode/skills/api-doc/
  SKILL.md
  openapi-template.yaml
```

**SKILL.md 内容：**
```yaml
---
name: api-doc
description: Generate OpenAPI documentation for an endpoint. Use when documenting API routes.
---

Generate OpenAPI documentation for the endpoint at $ARGUMENTS.

Use the template in [openapi-template.yaml](openapi-template.yaml) as the structure.

1. Read the endpoint code
2. Extract path, method, parameters, request/response schemas
3. Fill in the template with actual values
4. Output the completed YAML
```

**openapi-template.yaml 内容：**
```yaml
paths:
  /{path}:
    {method}:
      summary: ""
      description: ""
      parameters: []
      requestBody:
        content:
          application/json:
            schema: {}
      responses:
        "200":
          description: ""
          content:
            application/json:
              schema: {}
```

---

### 带校验脚本的数据库迁移生成器

用随 skill 附带的脚本生成并校验迁移：

```
.rustcode/skills/create-migration/
  SKILL.md
  scripts/
    validate-migration.sh
```

**SKILL.md 内容：**
```yaml
---
name: create-migration
description: Create a database migration file
disable-model-invocation: true
allowed-tools: Read, Write, Bash
---

Create a migration for: $ARGUMENTS

1. Generate migration file in `migrations/` with timestamp prefix
2. Include up and down functions
3. Run validation: `bash ~/.rustcode/skills/create-migration/scripts/validate-migration.sh`
4. Report any issues found
```

**scripts/validate-migration.sh 内容：**
```bash
#!/bin/bash
# Validate migration syntax
npx prisma validate 2>&1 || echo "Validation failed"
```

---

### 带示例的测试生成器

按项目既有模式生成测试：

```
.rustcode/skills/gen-test/
  SKILL.md
  examples/
    unit-test.ts
    integration-test.ts
```

**SKILL.md 内容：**
```yaml
---
name: gen-test
description: Generate tests for a file following project conventions
disable-model-invocation: true
---

Generate tests for: $ARGUMENTS

Reference these examples for the expected patterns:
- Unit tests: [examples/unit-test.ts](examples/unit-test.ts)
- Integration tests: [examples/integration-test.ts](examples/integration-test.ts)

1. Analyze the source file
2. Identify functions/methods to test
3. Generate tests matching project conventions
4. Place in appropriate test directory
```

---

### 基于模板的组件生成器

用模板脚手架生成新组件：

```
.rustcode/skills/new-component/
  SKILL.md
  templates/
    component.tsx.template
    component.test.tsx.template
    component.stories.tsx.template
```

**SKILL.md 内容：**
```yaml
---
name: new-component
description: Scaffold a new React component with tests and stories
disable-model-invocation: true
---

Create component: $ARGUMENTS

Use templates in [templates/](templates/) directory:
1. Generate component from component.tsx.template
2. Generate tests from component.test.tsx.template
3. Generate Storybook story from component.stories.tsx.template

Replace {{ComponentName}} with the PascalCase name.
Replace {{component-name}} with the kebab-case name.
```

---

### 按清单评审 PR

按项目专属清单评审 PR：

```
.rustcode/skills/pr-check/
  SKILL.md
  checklist.md
```

**SKILL.md 内容：**
```yaml
---
name: pr-check
description: Review PR against project checklist
disable-model-invocation: true
---

## PR Context
- Diff: !`git diff HEAD~1`
- Description: !`git log -1 --format=%B`

Review against [checklist.md](checklist.md).

For each item, mark pass or fail with explanation.
```

**checklist.md 内容：**
```markdown
## PR Checklist

- [ ] Tests added for new functionality
- [ ] No console.log statements
- [ ] Error handling includes user-facing messages
- [ ] API changes are backwards compatible
- [ ] Database migrations are reversible
```

---

### 发布说明生成器

从 git 历史生成发布说明：

**SKILL.md 内容：**
```yaml
---
name: release-notes
description: Generate release notes from commits since last tag
disable-model-invocation: true
---

## Recent Changes
- Commits since last tag: !`git log $(git describe --tags --abbrev=0)..HEAD --oneline`
- Last tag: !`git describe --tags --abbrev=0`

Generate release notes:
1. Group commits by type (feat, fix, docs, etc.)
2. Write user-friendly descriptions
3. Highlight breaking changes
4. Format as markdown
```

---

### 项目约定（仅 RustCode 使用）

RustCode 自动应用的背景知识：

**SKILL.md 内容：**
```yaml
---
name: project-conventions
description: Code style and patterns for this project. Apply when writing or reviewing code.
user-invocable: false
---

## Naming Conventions
- React components: PascalCase
- Utilities: camelCase
- Constants: UPPER_SNAKE_CASE
- Files: kebab-case

## Patterns
- Use `Result<T, E>` for fallible operations, not exceptions
- Prefer composition over inheritance
- All API responses use `{ data, error, meta }` shape

## Forbidden
- No `any` types
- No `console.log` in production code
- No synchronous file I/O
```

---

### 环境搭建

用配置脚本帮新开发者完成上手：

```
.rustcode/skills/setup-dev/
  SKILL.md
  scripts/
    check-prerequisites.sh
```

**SKILL.md 内容：**
```yaml
---
name: setup-dev
description: Set up development environment for new contributors
disable-model-invocation: true
---

Set up development environment:

1. Check prerequisites: `bash scripts/check-prerequisites.sh`
2. Install dependencies: `npm install`
3. Copy environment template: `cp .env.example .env`
4. Set up database: `npm run db:setup`
5. Verify setup: `npm test`

Report any issues encountered.
```

---

## 参数模式

| 模式 | 含义 | 示例 |
|---------|---------|---------|
| `$ARGUMENTS` | 全部参数拼成的字符串 | `/deploy staging` -> "staging" |

若 skill 中没有出现 `$ARGUMENTS`，参数会以 `ARGUMENTS: <value>` 的形式追加。

## 动态上下文注入

用 `` !`command` `` 在 skill 运行前注入实时数据：

```yaml
## Current State
- Branch: !`git branch --show-current`
- Status: !`git status --short`
```

命令输出会在 RustCode 看到 skill 内容之前替换掉该占位符。
