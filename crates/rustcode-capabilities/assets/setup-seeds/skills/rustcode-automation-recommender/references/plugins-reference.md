# 插件推荐

插件是可安装的能力集合，打包了 skills、commands、agents 与 hooks。RustCode 支持安装插件来扩展功能。

**说明**：以下是常见插件。请用 Web 搜索发现更多社区插件。

---

## 核心插件

### RustCode 官方插件

| 插件 | 适用场景 | 主要特性 |
|--------|----------|--------------|
| **rustcode** | RustCode 用法与文档问答 | 离线文档索引、安装/配置/排障解答、`/skills ask` 命令 |

### 开发与代码质量

| 插件 | 适用场景 | 主要特性 |
|--------|----------|--------------|
| **plugin-dev** | 开发 RustCode 插件 | 用于创建 skill、hook、command、agent 的 skill |
| **pr-review-toolkit** | PR 评审流程 | 专职评审 agent（代码、测试、类型） |
| **code-review** | 自动化代码评审 | 多 agent 评审并给出置信度评分 |
| **code-simplifier** | 代码重构 | 在保持功能不变的前提下简化代码 |
| **feature-dev** | 功能开发 | 由 agent 驱动的端到端功能流程 |

### Git 与工作流

| 插件 | 适用场景 | 主要特性 |
|--------|----------|--------------|
| **commit-commands** | Git 工作流 | /commit、/commit-push-pr 命令 |
| **hookify** | 自动化规则 | 从对话模式中生成 hook |

### 前端

| 插件 | 适用场景 | 主要特性 |
|--------|----------|--------------|
| **frontend-design** | UI 开发 | 生产级 UI，避免千篇一律的样式 |

### 学习与引导

| 插件 | 适用场景 | 主要特性 |
|--------|----------|--------------|
| **explanatory-output-style** | 学习 | 解释代码取舍背后的原因 |
| **learning-output-style** | 交互式学习 | 在决策点征询你的意见 |
| **security-guidance** | 安全意识 | 编辑时提示安全风险 |

### 语言服务器（LSP）

| 插件 | 语言 |
|--------|----------|
| **typescript-lsp** | TypeScript/JavaScript 语言 |
| **pyright-lsp** | Python 语言 |
| **gopls-lsp** | Go 语言 |
| **rust-analyzer-lsp** | Rust 语言 |
| **clangd-lsp** | C/C++ 语言 |
| **jdtls-lsp** | Java 语言 |
| **kotlin-lsp** | Kotlin 语言 |
| **swift-lsp** | Swift 语言 |
| **csharp-lsp** | C# 语言 |
| **php-lsp** | PHP 语言 |
| **lua-lsp** | Lua 语言 |

---

## 速查：代码库 -> 插件

| 代码库特征 | 推荐插件 |
|-----------------|-------------------|
| 任意项目（首次配置） | rustcode |
| 正在开发插件 | plugin-dev |
| 以 PR 为主的工作流 | pr-review-toolkit |
| Git 提交 | commit-commands |
| React/Vue/Angular 项目 | frontend-design |
| 想要自动化规则 | hookify |
| TypeScript 项目 | typescript-lsp |
| Python 项目 | pyright-lsp |
| Go 项目 | gopls-lsp |
| 安全敏感代码 | security-guidance |
| 学习/上手阶段 | explanatory-output-style |

---

## 何时推荐插件

**出现以下情况时推荐安装插件：**
- 用户想从共享仓库安装 RustCode 自动化能力
- 用户需要多项相关能力
- 团队希望统一工作流
- 首次配置 RustCode
