# @rustcode/rustcode（RustCode 的 npm 分发包）

[![npm 版本号](https://img.shields.io/npm/v/@rustcode/rustcode)](https://www.npmjs.com/package/@rustcode/rustcode)
[![许可证](https://img.shields.io/npm/l/@rustcode/rustcode)](https://www.npmjs.com/package/@rustcode/rustcode)

**RustCode** — 开源终端 AI 编码助手。用自然语言描述任务，自动阅读代码、编辑文件、执行命令、验证结果。

## 安装

```bash
npm install -g @rustcode/rustcode
```

安装完成后即可使用：

```bash
rustcode
```

> 安装时 npm 会自动下载匹配当前平台的预编译二进制（darwin/linux arm64+x64, windows x64, ohos arm64）。

## 使用

```bash
# 交互模式（TUI）
rustcode

# 指定项目目录
rustcode -C /path/to/project

# 指定模型
rustcode --model gpt-4o

# 非交互模式（headless）
rustcode -p "解释这个仓库的架构"

# 继续上次对话
rustcode --continue
```

## 卸载

```bash
npm uninstall -g @rustcode/rustcode

# 或使用内置卸载命令（会保留配置文件）
rustcode uninstall
```

## 版本对应

npm 版本号与 RustCode 发布版本一致，发布说明请见你的发行渠道。

## 链接

- 源码仓库与问题反馈：请通过你获取本软件的发行渠道获取（本构建不绑定固定托管平台）。
- 许可证：MIT

---

由 Rust 与 ratatui 构建而成，外加许多个深夜。
