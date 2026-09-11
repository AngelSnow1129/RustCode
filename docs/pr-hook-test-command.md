# PR: 实现 `hooks test <name>` 命令

## 改动概述

实现了 `rustcode hooks test <name>` 命令，目前该命令只是一个打印 TODO 的空壳，现改为实际执行指定 hook 并展示详细结果。

## 涉及文件

| 文件 | 改动 |
|------|------|
| `crates/rustcode-cli/src/main.rs` | `rustcode hooks test` 子命令实现（`HookCommands::Test` → `handle_hooks`） |
| `crates/rustcode-capabilities/src/cc_hooks.rs` | CC 兼容 hook 的加载与单条测试执行（`load_hooks_config` / `run_hook_for_test` / 路径解析 `global_hooks_path` / `project_hooks_path`） |

> [NOTE] 原实现文档将相关改动归到 `crates/rustcode-core/src/hook/*`，但 `rustcode-core` 运行时已在本 fork 中移除，上述文件不复存在；当前 `rustcode hooks` 命令的实现位于 `rustcode-cli` 与 `rustcode_capabilities::cc_hooks`。

## 改动的价值

### 1. `rustcode hooks test <name>` 命令

**之前**：执行 `rustcode hooks test my-hook` 只会打印：
```
Testing hook: my-hook
(TODO: Implement hook testing)
```
完全不做事。

**之后**：该命令会：

- 从 `$RUSTCODE_HOME/hooks.json`（全局）和 `<项目>/.hooks.json`（项目）加载所有已配置的 Claude-Code 兼容 JSON hook（`hooks.toml` 的 script/webhook 配置属于已移除的 `rustcode-core` 体系，当前不加载）
- 按名称查找目标 hook
- 显示 hook 的完整元信息（事件类型、命令、超时时间、matcher、plugin 路径）
- 构建模拟的 `HookContext` 环境（含测试用的 session_id、tool_name、tool_args）
- 以 hook 自身配置的 timeout 执行命令，环境变量与真实运行时完全一致
  - `RUSTCODE_HOOK_EVENT` — 事件名
  - `RUSTCODE_HOOK_CONTEXT` — JSON 序列化的完整上下文
  - `RUSTCODE_TOOL_NAME` — 当前工具名
  - `CLAUDE_PLUGIN_ROOT` / `RUSTCODE_PLUGIN_ROOT` — 插件根目录
- 展示详细的执行结果：**stdout / stderr / 退出码 / 耗时 / 超时状态**
- 若指定名称未找到，列出所有可用 hook 供参考

### 2. `load_hooks_config_with_names()` 函数

在 `json_config.rs` 中新增了一个公共函数，与内部的 `load_hooks_config()` 行为完全一致，但保留 hook 的名称信息（返回 `Vec<(String, HookConfig)>` 而非 `Vec<HookConfig>`）。这为后续需要按名称操作 hook 的功能提供了基础。

## 新命令使用示例

```bash
# 测试一个名为 "check-bash" 的 hook
$ rustcode hooks test check-bash

[*] Testing Hook: check-bash
  Event:     pre_tool_use
  Command:   ./scripts/check_bash.sh
  Timeout:   10000 ms
  Matcher:   bash

[*] Result:
  Duration:  12.345ms
  Status:    [+] SUCCESS (exit code 0)
  ── stdout ──
  │ Tool check passed: bash

  [+] Hook 'check-bash' executed successfully.
```

```bash
# 查找不存在的 hook 时
$ rustcode hooks test nonexistent

[-] Hook 'nonexistent' not found.

Available hooks:
  [*] check-bash                  (event: pre_tool_use, command: ./scripts/check_bash.sh)
  [*] notify-slack                (event: post_tool_use, command: ./scripts/notify.sh)
```

```bash
# 超时场景
$ rustcode hooks test slow-hook

[*] Testing Hook: slow-hook
  Event:     pre_tool_use
  Command:   sleep 30
  Timeout:   5000 ms

[*] Result:
  ⏱ TIMEOUT after 5000 ms
  The hook command was killed because it exceeded the configured timeout.
```

## 改动行数

- `crates/rustcode-cli/src/main.rs`: `rustcode hooks test` 子命令实现（具体增删以当前代码为准）
- `crates/rustcode-capabilities/src/cc_hooks.rs`: CC 兼容 hook 加载与测试执行（具体增删以当前代码为准）

## 兼容性

- 完全向后兼容：没有修改任何现有函数的签名或行为
- `load_hooks_config()` 保持不变，只在旁边新增一个 name-preserving 版本
- 所有测试无需修改，新增代码不影响现有 test suite


