# RustCode for JetBrains（JetBrains 插件使用说明）

RustCode for JetBrains 把本地的 RustCode 编码智能体带入基于 IntelliJ 的 IDE。它提供原生工具窗口、编辑器操作与意图（intention）操作，用于基于聊天的编码工作流。

## 环境要求

- 与 JetBrains Marketplace 上所示插件版本兼容的 JetBrains IDE。
- 从 JetBrains Marketplace 或签名插件 ZIP 安装的 RustCode 插件。
- 一个本地 RustCode 后端。Marketplace 构建可能为受支持的平台附带打包好的后端，你也可以自行配置后端二进制路径。

## 安装

### 从 JetBrains Marketplace 安装

1. 打开 `Settings | Plugins`。
2. 搜索 `RustCode`。
3. 安装插件，并在提示时重启 IDE。

### 从签名 ZIP 安装

1. 打开 `Settings | Plugins`。
2. 选择齿轮菜单。
3. 选择 `Install Plugin from Disk...`。
4. 选择已签名的 `rustcode-jetbrains-<version>-signed.zip` 文件。
5. 在提示时重启 IDE。

## 打开 RustCode

可使用以下任一入口：

- `Tools | RustCode: Open Chat`
- 通过 Search Everywhere 执行 `RustCode: Open Chat`
- `RustCode` 工具窗口
- 编辑器右键菜单中的操作，例如 `RustCode: Explain Selection`
- 针对选中代码的 Alt+Enter 意图操作

## 配置后端

打开 `Settings | Tools | RustCode`，或执行 `RustCode: Open Settings`。

可配置项包括：

- 后端二进制路径
- 主机与端口，默认为 `127.0.0.1:13456`
- 请求超时
- 聊天字体大小
- 上下文级别
- 选中文本上下文
- 相对路径共享
- RustCode 读取文件前自动保存
- 聊天发送快捷键行为

默认情况下，插件与位于 `127.0.0.1` 的本地后端通信。如果你配置了其他主机，请在发送项目上下文之前先评估隐私与安全影响。

## 配置供应商

打开 RustCode 工具窗口，使用供应商控件添加或编辑供应商。

支持的供应商类型包括：

- OpenAI 兼容供应商
- Claude
- Ollama
- 通过供应商 base URL 接入的自定义兼容端点

供应商设置可包含供应商名称、模型名称、base URL 与 API key。在 JetBrains 插件中输入的 API key 会被发送到本地 RustCode 后端，供后端存储或用于供应商请求。

## 上下文与隐私控制

RustCode 可以把编辑器选区、附加文件、当前文件上下文与项目元数据用作编码上下文。你可以通过 RustCode 设置以及显式的编辑器操作来控制这一点。

重要控制项：

- 如果你不希望基于选区的代码上下文被发送到后端，请关闭选中文本上下文。
- 当你希望提示词包含更少的项目信息时，请使用最小上下文。
- 在向外部模型供应商发送提示词之前，请先检查附加的文件与选中的代码。
- 私钥、`.env` 文件、凭据、SSH 配置、AWS 配置、GnuPG 数据与 Terraform state 等敏感路径会被更严格地处理或直接阻止。

在配置外部模型供应商之前，请先阅读隐私政策：

`../PRIVACY.md`

本 fork 不携带任何遥测：不会收集或发送任何使用事件，崩溃报告只停留在本地 stderr。为了向后兼容较旧的启动脚本，启动参数 `--no-telemetry` 会被接受并忽略。

## 常见工作流

### 解释选中的代码

1. 在编辑器中选中代码。
2. 从编辑器右键菜单或 Search Everywhere 执行 `RustCode: Explain Selection`。
3. 在 RustCode 工具窗口中查看生成的解释。

### 修复或优化选中的代码

1. 在编辑器中选中代码。
2. 执行 `RustCode: Fix Selection` 或 `RustCode: Optimize Selection`。
3. 查看 RustCode 的响应，并在检查 diff 之后才应用改动。

### 附加文件作为上下文

1. 打开 RustCode 工具窗口。
2. 使用附加文件控件，或执行 `RustCode: Add Selection/File as Context`。
3. 发送引用该上下文的提示词。

### 查看本地改动

使用 `RustCode: Open Changes` 查看项目改动，RustCode 可在审查工作流中使用这些改动。

## 故障排查

- 如果 RustCode 无法连接，请检查设置中的后端主机与端口。
- 如果后端启动失败，请配置后端二进制路径，或单独安装 RustCode。
- 如果供应商请求失败，请检查供应商类型、模型、base URL 与 API key。
- 如果上下文缺失，请检查上下文级别与选中文本上下文设置。
- 本插件不会收集或发送遥测数据，因此没有可禁用的遥测开关；为向后兼容，启动参数 `--no-telemetry` 会被接受并忽略。

## 支持

请通过你的分发渠道对应的 issue 跟踪器反馈问题，并从同一渠道获取源代码。以下是占位 URL（请把
`<your-org>` 替换为你所在渠道的组织名）：

- Issue 跟踪器：`https://example.com/<your-org>/rustcode/issues`
- 源代码：`https://example.com/<your-org>/rustcode`
