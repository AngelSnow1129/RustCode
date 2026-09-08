# Telemetry(遥测)—— 已移除

```text
[STATUS] This fork ships ZERO telemetry.
[NOTE]   This page is kept only so existing links resolve. It describes a
         pipeline that no longer exists, and none of it is shipping behavior.
```

上游项目曾内置一条匿名的 usage-telemetry(使用量遥测)管线。本 fork
(`gitcode.com/SecLab/RustCode`)已将其**彻底移除**。

## 已移除的内容

- `rustcode-telemetry` crate —— 已删除;既没有目录,也没有任何依赖。
- `telemetry` CLI 子命令与 `[telemetry]` 配置段。`config.toml` 中
  遗留的 `[telemetry]` 段现在会被**静默忽略**,因此既有配置
  仍能继续加载。
- `CliOverride` 类型以及 `--no-telemetry` 的实际效果。该 flag 仍然
  **被接受但被忽略**(并在 stderr 打印一条 warning),因为较旧的 IDE
  扩展会传入它。
- 崩溃上报(crash reporting)。panic 只写入 stderr;不会有任何数据离开本机。

## 历史管线(仅供审计)

上游曾把 launch、LLM-turn、command、login 和 crash 事件收集到
本地 NDJSON 队列,再 POST 到自建 endpoint。这些代码在本仓库中
**一处都不存在**:没有 event queue,没有 sender,也没有 endpoint。
保留这段描述,只是为了让还记得旧行为的读者能够确认它确实
已经消失。

## 仍然存在、且**并非**遥测的部分

不要删除下面这些 —— 它们经常被误判为遥测:

| 项 | 它实际上是什么 |
|---|---|
| `SessionMode` / `ClientMode` | 按当前连接的是哪个客户端(IDE、webui、TUI)做本地分支判断。它驱动 token 权限与 webui 路径。wire tag 字符串被刻意保持稳定,以便较旧的扩展继续工作。 |
| `RepoOrigin` / `detect_repo_origin` | 对 git remote host 做纯字符串解析。无网络请求。 |
| turn datalog | 写入**本地**文件的结构化日志,用于调试单个 turn。 |
| update check | 针对 release manifest 的版本检查。它确实是一次网络请求,因此可配置,但不发送任何使用量事件。 |

## 如何验证

```sh
grep -rniE 'sentry|posthog|amplitude|mixpanel|opentelemetry|prometheus|statsd' \
  --include='*.rs' --include='*.toml' crates/
# [CHECK] must print nothing
```

本项目的依赖中没有任何第三方分析 SDK。历史管线是自建的,因此移除它属于
dead-code 清理,而不是对 SDK 做手术式改造。
