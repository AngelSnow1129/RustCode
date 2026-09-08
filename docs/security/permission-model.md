# 权限模型

## 概述

RustCode 使用统一的权限模型来控制对文件、目录与 shell 命令的访问。

其目标是：

- 阻止在未经确认的情况下访问当前工作目录之外的路径
- 在某个文件工具被拒绝后，阻止常见的基于 shell 的绕过手段
- 让权限行为在代码中保持显式且可追溯

本文档描述当前实现的行为。

## 设计目标

RustCode 的权限模型旨在：

- 区分常规项目工作与外部系统访问
- 区分低风险读取与高风险写入
- 在各文件工具之间应用一致的路径规则
- 对常见的 shell 文件命令应用等价的路径规则
- 为用户与维护者保留一个简单的心智模型

## 非目标

当前模型并不追求：

- 实现完整的 shell 解析器
- 静态分析解释器代码，例如 `python -c "open(...)"`
- 从 shell 变量或命令替换推断所有运行时展开的路径
- 取代沙箱机制或操作系统级的安全边界

沙箱机制与主机级权限仍然是重要的防御层。

## 核心概念

### 工作目录边界

所有路径检查都从解析被请求的路径开始，并判断它是否仍位于当前工作目录之内。

- 位于工作目录内的路径自动批准
- 位于工作目录外的路径按动作与敏感度分类

### 访问动作

RustCode 当前用三种动作建模外部路径访问：

- `Enumerate`
  目录列举、结构性探查、切换目录
- `Read`
  读取文件内容或搜索文件内容
- `Write`
  创建、编辑、覆盖、重命名或以其他方式修改文件

### 批准结果

权限检查返回三种结果之一：

- `AutoApprove`
- `RequireApproval`
- `RequireApprovalAlways`

`RequireApprovalAlways` 是更强的一种形式，用于高风险操作，例如敏感读取或工作区之外的任何写入。

## 路径批准规则

当前外部路径行为如下：

| 场景 | 结果 |
|---|---|
| 路径位于工作目录内 | `AutoApprove` |
| 工作区外、非敏感 `Enumerate` | `AutoApprove` |
| 工作区外、敏感 `Enumerate` | `RequireApprovalAlways` |
| 工作区外、非敏感 `Read` | `RequireApproval` |
| 工作区外、敏感 `Read` | `RequireApprovalAlways` |
| 工作区外、任何 `Write` | `RequireApprovalAlways` |

这意味着：

- 常规的外部目录浏览默认允许
- 常规的外部文件读取需要确认
- 敏感读取始终需要强确认
- 工作区之外的任何写入始终需要强确认

## 敏感路径分类

当前，若路径命中内置受保护系统前缀之一，则被视为敏感路径，除非它同时命中例外前缀，或者命中凭据/配置类文件的规则。

### 内置受保护前缀

当前内置受保护前缀包括：

- `/System`
- `/bin`
- `/sbin`
- `/usr`
- `/var`
- `/private/etc`
- `/private/var`
- `/etc`
- `/root`
- `/var/root`
- `/private/var/root`

### 内置例外

以下路径当前被豁免于受保护前缀规则：

- `/usr/local`
- `/private/usr/local`
- `/Applications`
- `/Library`
- `/var/folders`
- `/private/var/folders`
- `/var/tmp`
- `/private/var/tmp`

设置这些例外，是为了避免把常见的可写区域或用户所有的区域过度归类为敏感。

### Home 与类密钥路径

RustCode 也将以下路径视为敏感：

敏感 home 目录：

- `~/.ssh`
- `~/.aws`
- `~/.gnupg`
- `~/.config`

敏感文件名：

- `.bashrc`
- `.bash_profile`
- `.zshrc`
- `.zprofile`
- `.zshenv`
- `.npmrc`
- `.pypirc`
- `.env`
- `.env.local`
- `credentials`
- `config`
- `id_rsa`
- `id_dsa`
- `id_ecdsa`
- `id_ed25519`

敏感扩展名：

- `.pem`
- `.key`
- `.p12`
- `.pfx`
- `.der`
- `.crt`
- `.cer`

## 工具集成

大多数内置文件与路径工具已经使用共享的路径批准模型。

### 文件与目录工具

当前映射如下：

| 工具 | 动作 |
|---|---|
| `read_file` | `Read` |
| `grep` | `Read` |
| `find_references` | `Read` |
| `list_symbols` | `Read` |
| `read_symbol` | `Read` |
| `lsp`（第一阶段全部操作） | `Read` |
| `list_directory` | `Enumerate` |
| `glob` | `Enumerate` |
| `cd` | `Enumerate` |
| `file_dependencies` | `Enumerate` |
| `blast_radius` | `Enumerate` |
| `edit_file` | `Write` |
| `write_file` | `Write` |
| `search_replace` | `Write` |

这让 RustCode 对文件工具使用统一的路径策略，而不是每个工具各自一套临时逻辑。

## Bash 权限模型

`bash` 采用两层权限模型。

### 第 1 层：危险命令检测

部分命令被标记，是因为无论路径如何，命令本身就具有风险。

例如包括：

- 特权执行，例如 `sudo`
- 破坏性删除，例如 `rm -rf`
- 危险的网络或 shell 隧道模式
- 通过管道送入 shell 执行的远程脚本
- force-push 以及其他破坏性 VCS 操作

这些返回 `RequireApproval`。

### 第 2 层：常见 shell 文件命令的路径检查

对于常见的 shell 文件命令，RustCode 会提取其中的路径参数，并把它们映射到文件工具所使用的同一套共享路径批准模型上。

当前分类如下：

读取类命令：

- `cat`
- `head`
- `tail`
- `less`
- `more`
- `bat`
- `hexdump`
- `xxd`
- `strings`
- `file`
- `stat`
- `grep`
- `sed`
- `awk`
- `cut`
- `sort`
- `uniq`
- `wc`
- `diff`
- `patch`
- `tar`
- `unzip`
- `gunzip`
- `source`
- `.`

列举类命令：

- `ls`
- `dir`
- `tree`
- `find`

写入类命令：

- `cp`
- `mv`
- `touch`
- `mkdir`
- `rmdir`
- `rm`
- `chmod`
- `chown`
- `tee`
- `install`

该层用于阻止常见的绕过手段，例如：

- 在 `read_file` 被拒绝后改用 `cat`
- 使用 `ls` 或 `find` 探查敏感外部路径
- 使用 `cp`、`mv` 或重定向修改工作区之外的文件

### Shell wrapper 与重定向

当前实现还处理以下情况：

- `bash -c ...`
- `bash -lc ...`
- 使用 `<` 的输入重定向
- 使用 `>` 与 `>>` 的输出重定向

这使得常见的 shell 包装模式能够继承同样的路径检查。

## 明确边界：解释器代码

RustCode 当前不会对通过 shell 命令传入的解释器代码做语义检查。

例如：

```bash
cat /etc/hosts
```

与

```bash
python -c "print(open('/etc/hosts').read())"
```

会被区别对待。

前者由 shell 文件命令权限层检查。后者当前位于该层之外。

## 该边界为何存在

该边界让模型保持可落地。

若没有该边界，RustCode 将不得不：

- 解析大量脚本语言
- 理解嵌套引号与运行时字符串构造
- 从解释器语义推断文件访问
- 维护一个大得多且更不可预测的安全面

当前实现转而聚焦于：

- 文件工具
- 常见的 shell 文件命令
- 显式携带路径的 shell 语法
- 危险命令模式

## 用户体验模型

从用户视角看，当前模型应这样理解：

- 项目内工作通常是无摩擦的
- 外部文件读取会请求确认
- 外部敏感读取会以更强的方式请求确认
- 任何外部写入都会以更强的方式请求确认
- 危险 shell 命令会请求确认
- 常见的 shell 文件命令绕过手段由同一套路径规则阻断
- 解释器内部的文件访问当前位于该批准层之外

## 当前模型的优势

当前模型具备若干优势：

- 路径策略在各工具之间统一
- 常见的 shell 绕过手段被覆盖
- 批准强度是显式的
- 工作区边界与敏感度边界分别建模
- 行为可在代码中检视，而不是隐藏在零散的条件判断里

## 已知局限

当前模型也存在以下取舍：

- shell 命令覆盖是启发式的，并不完整
- 受保护路径列表与例外列表需要维护
- 平台相关的文件系统行为可能导致边界情况
- 当 shell 命令被解析而解释器代码未被解析时，用户仍可能认为某些情况不一致

## 建议的演进方向

建议的方向是在当前模型上做细化，而不是替换它。

### 保留

- 共享的路径批准核心
- `Read / Write / Enumerate` 动作模型
- `RequireApproval` 与 `RequireApprovalAlways`
- 危险 shell 命令检测
- 常见 shell 文件命令保护

### 收敛

- shell 命令解析的广度
- 建模复杂 shell 语义的压力
- 对不断增长的手写 shell 启发式规则的依赖

### 后续补充

- 可配置的敏感前缀
- 可配置的敏感 glob
- 可配置的例外
- 面向用户更清晰地说明哪些场景被覆盖、哪些未被覆盖

## 小结

RustCode 当前对文件工具与常见 shell 文件命令使用统一的基于路径的批准模型。

它保护：

- 工作区边界
- 敏感外部读取
- 全部外部写入
- 常见的基于 shell 的文件访问绕过手段
- 危险 shell 命令

它有意不尝试分析解释器代码，也不追求成为完整的 shell 安全引擎。

这让模型在常见场景上保持强健、在代码中保持显式，并具备足以持续演进的可维护性。
