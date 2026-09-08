# 每回合执行策略由 Runtime 持有

## 背景

过去 coding persona 与 VerifyCadence 会在真实用户已明确禁止编译、测试或执行脚本的情况下，
仍然鼓励执行验证。仅靠提示词措辞并不构成执行边界。主 agent 还可以把任务委派给 worker，
而 worker 拥有独立工具栈并以自动批准运行，因此只拦截主 `bash` 会留下绕过路径。

## 决策

`CodingRuntime` 持有一个按回合的执行策略句柄。真实用户的 Submit 或 steer 会立即替换其状态；
lifecycle hook 则在 resume、compaction 或 reassembly 之后，从最新的非合成用户消息重新推导出
同样的状态。合成提醒永远不会获得授权。

该句柄在批准之前作为 middleware 安装到主 agent 上，并由 worker subagent 继承。
`rustcode-capabilities::TaskTool` 只传输通用的 worker middleware，对 coding policy 保持无感知。
只读的 explore subagent 不会收到该句柄，因为它们不挂载任何 shell 或写入类工具。

构建、测试、脚本以及全部 shell 限制是彼此独立的开关。Bash 语法由 capabilities 层用
tree-sitter 只解析一次；它暴露中性的命令调用，而由 coding 层赋予产品语义。在限制生效期间，
解析不完整会 fail-closed。

## 结论

- 用户限制在主执行与 worker 执行之间一致生效；
- 仅限制测试不会不必要地阻断编译，反之亦然；
- 常见的 shell wrapper、嵌套命令替换、带引号的分隔符以及 Windows 可执行文件形式，都依据语法
  进行分类，而不是依赖第二套手写的 shell 解析器；
- 自然语言检测仍然只是便利入口，而非完整的语言解析器；带引号的示例会被忽略，日后也可以
  在不改变 runtime/middleware 所有权边界的前提下增加显式的结构化控制；
- 在 steer 之前已经越过 middleware 的进程，无法被追溯性地阻止启动；对已在运行的工作，
  仍然以常规取消作为处理手段。
