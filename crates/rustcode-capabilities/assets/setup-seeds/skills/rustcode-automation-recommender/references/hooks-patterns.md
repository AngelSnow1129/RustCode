# Hooks 推荐

Hooks 会在 RustCode 事件发生时自动执行命令。它们非常适合用于强制约束，以及需要稳定执行的自动化。

**说明**：以下是常见模式。对这里没列出的工具/框架，请用 Web 搜索查找相应的 hooks，以便给用户推荐最合适的。

## 自动格式化 Hooks

### Prettier（JavaScript/TypeScript 格式化）
| 检测依据 | 是否存在 |
|-----------|-------------|
| `.prettierrc`、`.prettierrc.json`、`prettier.config.js` | 是 |

**推荐**：在 Edit/Write 上挂 PostToolUse hook，自动格式化
**价值**：代码始终保持格式化，无需操心

### ESLint（JavaScript/TypeScript lint）
| 检测依据 | 是否存在 |
|-----------|-------------|
| `.eslintrc`、`.eslintrc.json`、`eslint.config.js` | 是 |

**推荐**：在 Edit/Write 上挂 PostToolUse hook，自动修复
**价值**：lint 错误会被自动修复

### Black/isort（Python 格式化）
| 检测依据 | 是否存在 |
|-----------|-------------|
| `pyproject.toml` 中含 black/isort、`.black`、`setup.cfg` | 是 |

**推荐**：挂 PostToolUse hook 格式化 Python 文件
**价值**：Python 格式保持一致

### Ruff（Python lint + 格式化）
| 检测依据 | 是否存在 |
|-----------|-------------|
| `ruff.toml`、含 `[tool.ruff]` 的 `pyproject.toml` | 是 |

**推荐**：挂 PostToolUse hook 做 lint + 格式化
**价值**：快速、全面的 Python lint

### gofmt（Go 格式化）
| 检测依据 | 是否存在 |
|-----------|-------------|
| `go.mod` | 是 |

**推荐**：挂 PostToolUse hook 运行 gofmt
**价值**：标准的 Go 格式

### rustfmt（Rust 格式化）
| 检测依据 | 是否存在 |
|-----------|-------------|
| `Cargo.toml` | 是 |

**推荐**：挂 PostToolUse hook 运行 rustfmt
**价值**：标准的 Rust 格式

---

## 类型检查 Hooks

### TypeScript 类型检查
| 检测依据 | 是否存在 |
|-----------|-------------|
| `tsconfig.json` | 是 |

**推荐**：挂 PostToolUse hook 运行 tsc --noEmit
**价值**：立刻发现类型错误

### mypy/pyright（Python 类型检查）
| 检测依据 | 是否存在 |
|-----------|-------------|
| `mypy.ini`、`pyrightconfig.json`、含 mypy 的 pyproject.toml | 是 |

**推荐**：挂 PostToolUse hook 做类型检查
**价值**：发现 Python 中的类型错误

---

## 保护类 Hooks

### 拦截敏感文件编辑
| 检测依据 | 存在迹象 |
|-----------|-------------|
| `.env`、`.env.local`、`.env.production` | 环境文件 |
| `credentials.json`、`secrets.yaml` | 密钥文件 |
| `.git/` 目录 | Git 内部文件 |

**推荐**：挂 PreToolUse hook，拦截对这些路径的 Edit/Write
**价值**：避免意外泄露密钥或损坏 git

### 拦截 lock 文件编辑
| 检测依据 | 存在迹象 |
|-----------|-------------|
| `package-lock.json`、`yarn.lock`、`pnpm-lock.yaml` | JS lock 文件 |
| `Cargo.lock`、`poetry.lock`、`Pipfile.lock` | 其他 lock 文件 |

**推荐**：挂 PreToolUse hook，拦截直接编辑
**价值**：lock 文件只应通过包管理器变更

---

## 测试运行 Hooks

### Jest（JavaScript/TypeScript 测试）
| 检测依据 | 存在迹象 |
|-----------|-------------|
| `jest.config.js`、package.json 中的 `jest` | 已配置 Jest |
| `__tests__/`、`*.test.ts`、`*.spec.ts` | 存在测试文件 |

**推荐**：挂 PostToolUse hook，编辑后运行相关测试
**价值**：改动后立刻得到测试反馈

### pytest（Python 测试）
| 检测依据 | 存在迹象 |
|-----------|-------------|
| `pytest.ini`、含 pytest 的 `pyproject.toml` | 已配置 pytest |
| `tests/`、`test_*.py` | 存在测试文件 |

**推荐**：挂 PostToolUse hook，对改动的文件运行 pytest
**价值**：立刻得到测试反馈

---

## 速查：检测依据 -> 推荐

| 如果看到 | 推荐的 Hook |
|------------|-------------------|
| Prettier 配置 | 在 Edit/Write 时自动格式化 |
| ESLint 配置 | 在 Edit/Write 时自动 lint |
| Ruff/Black 配置 | 自动格式化 Python |
| tsconfig.json | 在编辑时做类型检查 |
| 测试目录 | 在编辑时运行相关测试 |
| .env 文件 | 拦截 .env 编辑 |
| lock 文件 | 拦截 lock 文件编辑 |
| Go 项目 | 在编辑时运行 gofmt |
| Rust 项目 | 在编辑时运行 rustfmt |

---

## 通知类 Hooks

通知类 hooks 在 RustCode 发出通知时运行。用 matcher 按通知类型过滤。

### 权限提醒
| Matcher | 用途 |
|---------|----------|
| `permission_prompt` | 在 RustCode 请求权限时提醒 |

**推荐**：播放声音、发送桌面通知，或记录权限请求
**价值**：并行处理多件事时也不会漏掉权限提示

### 闲置通知
| Matcher | 用途 |
|---------|----------|
| `idle_prompt` | 在 RustCode 等待输入时提醒（闲置 60 秒以上） |

**推荐**：在 RustCode 需要你注意时播放声音或发送通知
**价值**：知道 RustCode 何时在等你的输入

### 配置示例

```json
{
  "hooks": {
    "Notification": [
      {
        "matcher": "permission_prompt",
        "hooks": [
          {
            "type": "command",
            "command": "afplay /System/Library/Sounds/Ping.aiff"
          }
        ]
      },
      {
        "matcher": "idle_prompt",
        "hooks": [
          {
            "type": "command",
            "command": "osascript -e 'display notification \"RustCode is waiting\" with title \"RustCode\"'"
          }
        ]
      }
    ]
  }
}
```

### 可用 Matcher

| Matcher | 触发时机 |
|---------|---------------|
| `permission_prompt` | RustCode 需要某个工具的权限 |
| `idle_prompt` | RustCode 正在等待输入（60 秒以上） |
| `auth_success` | 认证成功 |
| `elicitation_dialog` | MCP 工具需要输入 |

---

## Hooks 的存放位置

Hooks 写在 `.rustcode/settings.json` 中：

```
.rustcode/
  settings.json  <- Hook configurations here
```

如果 `.rustcode/` 目录还不存在，建议创建它。
